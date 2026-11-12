// SPDX-License-Identifier: GPL-3.0-or-later
//! atari800-Adapter über den Text-Monitor (stdin/stdout). Einschränkungen des Monitors: nur ein Ausführungs-Haltepunkt,
//! kein Pausieren per Befehl (im Emulatorfenster F8 drücken – das Ereignis wird erkannt).
use crate::{find_program, Backend, Cycles, Hit, Regs, RunState, TrapCaps};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const SENTINEL: &str = "32639";                       // Ausgabe von `dec 7f7f`

struct Shared { lines: Mutex<Vec<String>>, cv: Condvar, running: AtomicBool }

pub struct Atari800 { child: Child, stdin: ChildStdin, sh: Arc<Shared>, bp: Option<u16> }

fn strip_ansi(s: &str) -> String {
    let (mut out, mut esc) = (String::new(), false);
    for c in s.chars() { if esc { if c.is_ascii_alphabetic() { esc = false; } } else if c == '\x1b' { esc = true; } else { out.push(c); } }
    out
}

impl Atari800 {
    /// Startet atari800 im Monitor. `.atr` wird als Diskette gebootet, alles andere per `-run` geladen. Der Emulator steht zunächst.
    pub fn launch(file: &str, program: Option<&str>) -> Result<Atari800, String> { Self::launch_with(file, program, None, &[]) }

    /// Wie `launch`, mit Betriebssystem-ROM (XL/XE) und zusätzlichen Argumenten (Einstellungen des Benutzers).
    pub fn launch_with(file: &str, program: Option<&str>, os_rom: Option<&str>, extra: &[String]) -> Result<Atari800, String> { Self::start(Some(file), program, os_rom, extra) }

    /// Startet atari800 im Monitor ohne Programm (für die Tests im Trap-Modus: der Läufer schreibt Programm und Falle selbst in den Speicher).
    /// Der Emulator steht vor dem ersten Befehl: das Betriebssystem ist nicht hochgefahren.
    pub fn launch_bare(program: Option<&str>, os_rom: Option<&str>, extra: &[String]) -> Result<Atari800, String> { Self::start(None, program, os_rom, extra) }

    fn start(file: Option<&str>, program: Option<&str>, os_rom: Option<&str>, extra: &[String]) -> Result<Atari800, String> {
        let exe = program.map(String::from).or_else(|| find_program("atari800", &["/opt/homebrew/bin", "/usr/local/bin"])).ok_or_else(|| format!("atari800 nicht gefunden ({})", crate::install_hint("atari800")))?;
        let mut args: Vec<String> = ["-xl", "-pal", "-no-autosave-config", "-nobasic", "-monitor"].iter().map(|s| s.to_string()).collect();
        if let Some(r) = os_rom.filter(|r| !r.is_empty()) { args.push("-xlxe_rom".into()); args.push(r.into()); }
        args.extend(extra.iter().cloned());
        if let Some(file) = file {
            let ext = |x: &str| Path::new(file).extension().map_or(false, |e| e.eq_ignore_ascii_case(x));
            if ext("atr") { args.push(file.into()); }
            else if ext("car") || ext("rom") { args.push("-cart".into()); args.push(file.into()); }
            else { if file.to_ascii_lowercase().ends_with(".bas") { args.retain(|a| a != "-nobasic"); args.push("-basic".into()); } args.push("-run".into()); args.push(file.into()); }
        }
        let mut child = Command::new(&exe).args(&args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e| format!("{exe}: {e}"))?;
        let stdin = child.stdin.take().unwrap();
        let out = child.stdout.take().unwrap();
        let sh = Arc::new(Shared { lines: Mutex::new(vec![]), cv: Condvar::new(), running: AtomicBool::new(false) });
        let s2 = sh.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(out);
            let mut buf = String::new();
            while r.read_line(&mut buf).map_or(0, |n| n) > 0 {
                let line = strip_ansi(buf.trim_end());
                buf.clear();
                if s2.running.load(Ordering::SeqCst) && (line.starts_with("(breakpoint at") || (line.contains(" PC=") && line.contains("A="))) { s2.running.store(false, Ordering::SeqCst); }
                s2.lines.lock().unwrap().push(line);
                s2.cv.notify_all();
            }
            s2.running.store(false, Ordering::SeqCst);
        });
        let mut a = Atari800 { child, stdin, sh, bp: None };
        // Warten, bis der Monitor bereit ist (Startausgabe mit Register-Zeile)
        let end = Instant::now() + Duration::from_secs(15);
        loop {
            if a.sh.lines.lock().unwrap().iter().any(|l| l.contains(" PC=") && l.contains("A=")) { break; }
            if Instant::now() > end { let _ = a.child.kill(); return Err("atari800-Monitor meldet sich nicht".into()); }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(a)
    }

    fn exec(&mut self, cmd: &str) -> Result<Vec<String>, String> {
        if self.sh.running.load(Ordering::SeqCst) { return Err("Emulator läuft – zuerst anhalten (Haltepunkt oder F8 im Emulatorfenster)".into()); }
        self.sh.lines.lock().unwrap().clear();
        write!(self.stdin, "{cmd}\ndec 7f7f\n").and_then(|_| self.stdin.flush()).map_err(|e| e.to_string())?;
        let end = Instant::now() + Duration::from_secs(8);
        let mut g = self.sh.lines.lock().unwrap();
        loop {
            if let Some(i) = g.iter().position(|l| l.contains(SENTINEL)) { return Ok(g[..i].to_vec()); }
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() { return Err(format!("atari800 antwortet nicht auf '{cmd}'")); }
            g = self.sh.cv.wait_timeout(g, left).unwrap().0;
        }
    }

    fn parse_regs(line: &str) -> Option<Regs> {
        let f = |key: &str| -> Option<&str> { let i = line.find(key)? + key.len(); Some(line[i..].split_whitespace().next()?.trim_end_matches(':')) };
        let hex = |s: &str| u32::from_str_radix(s, 16).ok();
        let flags = f("P=")?.chars().enumerate().fold(0u8, |acc, (i, c)| if c != '-' { acc | (0x80 >> i) } else { acc });
        let mut nums = line.split_whitespace().take(2).filter_map(|w| w.parse::<u32>().ok());
        let (lin, cyc) = (nums.next(), nums.next());
        let mut r = Regs { pc: hex(f("PC=")?)? as u16, a: hex(f("A=")?)? as u8, x: hex(f("X=")?)? as u8, y: hex(f("Y=")?)? as u8, sp: hex(f("S=")?)? as u8, flags, extra: vec![] };
        if let (Some(l), Some(c)) = (lin, cyc) { r.extra.push(("LIN".into(), l)); r.extra.push(("CYC".into(), c)); }
        Some(r)
    }
}

/// Monitorbefehle, die `regs` setzen. Wirksam sind PC, A, X, Y, S und die Flags D und I; N, V, Z und C zeigt der Monitor zwar an,
/// verwirft sie aber beim Fortsetzen (Vorversuch T0): sie kommen über den Vorspann der Falle an, nicht von hier.
pub fn set_commands(r: &Regs) -> Vec<String> {
    vec![format!("setpc {:04x}", r.pc), format!("seta {:02x}", r.a), format!("setx {:02x}", r.x), format!("sety {:02x}", r.y), format!("sets {:02x}", r.sp),
         format!("setd {}", (r.flags >> 3) & 1), format!("seti {}", (r.flags >> 2) & 1)]
}

/// Maße des Bildes: 114 Zyklen je Zeile, 312 Zeilen (PAL; der Adapter startet atari800 immer mit `-pal`)
const PER_LINE: u32 = 114;
const LINES: u32 = 312;

/// Strahlposition aus den Zusatzwerten `LIN`/`CYC` der Registerzeile
pub fn beam(r: &Regs) -> Result<Cycles, String> {
    let get = |n: &str| r.extra.iter().find(|(k, _)| k == n).map(|(_, v)| *v);
    match (get("LIN"), get("CYC")) { (Some(line), Some(cycle)) => Ok(Cycles { line, cycle, per_line: PER_LINE, lines: LINES }), _ => Err("atari800 meldet Zeile und Zyklus nicht".into()) }
}

impl Backend for Atari800 {
    /// Register: alles außer N V Z C (siehe [`set_commands`])
    fn trap_caps(&self) -> TrapCaps { TrapCaps { set_regs: false, cycles: true } }
    fn set_regs(&mut self, r: &Regs) -> Result<(), String> { for c in set_commands(r) { self.exec(&c)?; } Ok(()) }
    /// Strahlzeit, nicht reine CPU-Zyklen: enthält Speicherauffrischung (9 Zyklen je Zeile) und ANTIC-DMA
    fn cycles(&mut self) -> Result<Cycles, String> { let r = self.registers()?; beam(&r) }
    fn name(&self) -> &str { "atari800" }
    fn state(&mut self) -> RunState { if self.sh.running.load(Ordering::SeqCst) { RunState::Running } else { RunState::Stopped } }
    fn registers(&mut self) -> Result<Regs, String> {
        let lines = self.exec("show")?;
        lines.iter().find_map(|l| Self::parse_regs(l)).ok_or_else(|| format!("Register nicht lesbar: {lines:?}"))
    }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let (mut out, mut a) = (Vec::with_capacity(len), addr as usize);
        while out.len() < len && a <= 0xFFFF {
            let lines = self.exec(&format!("m {a:04x}"))?;
            let mut got = vec![];
            for l in &lines {
                let Some((head, rest)) = l.split_once(':') else { continue };
                if head.len() != 4 || u16::from_str_radix(head, 16).is_err() { continue; }
                got.extend(rest.split_whitespace().take(16).filter_map(|h| u8::from_str_radix(h, 16).ok()));
            }
            if got.is_empty() { return Err("Speicher nicht lesbar".into()); }
            let take = got.len().min(len - out.len()).min(0x10000 - a);
            out.extend_from_slice(&got[..take]);
            a += got.len();
        }
        Ok(out)
    }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        for (i, chunk) in data.chunks(16).enumerate() {
            let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
            self.exec(&format!("c {:04x} {}", addr as usize + i * 16, hex.join(" ")))?;
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> { self.exec("g").map(|_| ()) }
    fn cont(&mut self) -> Result<(), String> {
        if self.sh.running.load(Ordering::SeqCst) { return Ok(()); }
        self.sh.lines.lock().unwrap().clear();
        self.sh.running.store(true, Ordering::SeqCst);
        writeln!(self.stdin, "cont").and_then(|_| self.stdin.flush()).map_err(|e| e.to_string())
    }
    fn can_pause(&self) -> bool { false }
    fn pause(&mut self) -> Result<(), String> { Err("atari800 lässt sich nicht per Befehl anhalten – im Emulatorfenster F8 drücken".into()) }
    fn set_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        self.exec(&format!("bpc {addr:04x}"))?;
        self.bp = Some(addr);
        Ok(())
    }
    fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        if self.bp == Some(addr) { self.exec("bpc ffff")?; self.bp = None; }      // `$FFFF` dient als „kein Haltepunkt“
        Ok(())
    }
    fn breakpoints(&self) -> Vec<u16> { self.bp.into_iter().collect() }
    fn last_hit(&mut self) -> Option<Hit> { let pc = self.registers().ok()?.pc; (self.bp == Some(pc)).then_some(Hit::Exec(pc)) }
    fn stop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); }
}

impl Drop for Atari800 { fn drop(&mut self) { self.stop(); } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitorbefehle_setzen_register_und_flags_d_i() {
        let r = Regs { pc: 0x0600, a: 0x77, x: 0x12, y: 0x34, sp: 0xF0, flags: 0x2C, extra: vec![] };
        assert_eq!(set_commands(&r), ["setpc 0600", "seta 77", "setx 12", "sety 34", "sets f0", "setd 1", "seti 1"]);
        assert_eq!(set_commands(&Regs { flags: 0x20, ..r })[5..], ["setd 0", "seti 0"]);
    }

    #[test]
    fn strahlposition_aus_der_registerzeile() {
        let r = Atari800::parse_regs("  3  47 A=77 X=12 Y=34 S=F0 P=--------  PC=0600: A9 00").unwrap();
        let c = beam(&r).unwrap();
        assert_eq!((c.line, c.cycle, c.per_line, c.lines), (3, 47, 114, 312));
        // Strahlzeit über eine Zeilengrenze
        let (a, b) = (Cycles { line: 0, cycle: 100, ..c }, Cycles { line: 1, cycle: 20, ..c });
        assert_eq!(b.since(&a), 34);
        assert!(beam(&Regs::default()).is_err());
    }
}
