// SPDX-License-Identifier: GPL-3.0-or-later
//! MEGA65 über den UART-Monitor: die Hardware (USB-Seriell, 2 Mbit/s) und der Emulator Xemu (`-uartmon <Socket>`) sprechen dasselbe
//! zeilenweise Textprotokoll: `r` Register, `M adr` 256 Bytes lesen, `s adr bytes…` schreiben (28-Bit-Adressen, `777xxxx` = Sicht der CPU),
//! `t1`/`t0` anhalten/laufen, `t` Einzelschritt, `b adr` Haltepunkt, `g adr` PC setzen. Jede Antwort endet mit einer Zeile `.`.
//! Einschränkungen: ein Haltepunkt; Datenhaltepunkte nur als Schreib-Beobachtung per Einzelschritt (langsam, erkennt Wertänderungen);
//! der Monitor meldet keinen Halt von selbst, der Adapter erkennt ihn am PC.
use crate::{find_program, Backend, Caps, Hit, Regs, RunState, TrapCaps, WatchKind};
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Verbindung zum Monitor: Unix-Socket (Xemu auf macOS/Linux), TCP (Xemu auf Windows: `-uartmon :Port`) oder serielle Schnittstelle (Hardware)
enum Io { #[cfg(unix)] Unix(UnixStream), Tcp(TcpStream), Serial(std::fs::File) }
impl Io {
    fn write_all(&mut self, b: &[u8]) -> std::io::Result<()> { match self { #[cfg(unix)] Io::Unix(s) => s.write_all(b), Io::Tcp(s) => s.write_all(b), Io::Serial(f) => { f.write_all(b)?; f.flush() } } }
    /// Übrig gebliebene Ausgabe verwerfen (nur Sockets: nicht blockierend)
    fn drain(&mut self) {
        let mut b = [0u8; 4096];
        match self {
            #[cfg(unix)] Io::Unix(s) => { let _ = s.set_nonblocking(true); while matches!(s.read(&mut b), Ok(n) if n > 0) {} let _ = s.set_nonblocking(false); }
            Io::Tcp(s) => { let _ = s.set_nonblocking(true); while matches!(s.read(&mut b), Ok(n) if n > 0) {} let _ = s.set_nonblocking(false); }
            Io::Serial(_) => {}
        }
    }
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { match self { #[cfg(unix)] Io::Unix(s) => s.read(buf), Io::Tcp(s) => s.read(buf), Io::Serial(f) => f.read(buf) } }
    /// Gegenstelle ist Xemu (Zeilenende LF); die Hardware erwartet CR
    fn is_emulator(&self) -> bool { !matches!(self, Io::Serial(_)) }
}

pub struct Mega65 { /** Halt nach `run_until`: PC-Stand nach dem Befehl am Haltepunkt */ expect_pc: Option<u16>, io: Io, child: Option<Child>, paused: bool, bp: Option<u16>, label: String, watches: Vec<Watch>, next_watch: u32, emulating: bool, hit: Option<Hit> }
/// Beobachteter Bereich mit letztem bekannten Inhalt
struct Watch { id: u32, start: u16, end: u16, snap: Vec<u8> }

fn hex_ok(s: &str) -> Option<Vec<u8>> { (0..s.len() / 2).map(|i| u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()).collect() }

impl Mega65 {
    /// `spec`: `unix:/pfad/socket` (Xemu auf macOS/Linux), `tcp:host:port` (Xemu auf Windows, gestartet mit `-uartmon :port`),
    /// `/dev/cu.usbserial-…[@baud]` oder `serial:/dev/…[@baud]` (Hardware)
    pub fn connect(spec: &str) -> Result<Mega65, String> {
        let (io, label) = if let Some(a) = spec.strip_prefix("tcp:") {
            let s = TcpStream::connect(a).map_err(|e| format!("{a}: {e}"))?; s.set_read_timeout(Some(Duration::from_millis(300))).ok(); s.set_nodelay(true).ok();
            (Io::Tcp(s), "MEGA65 (Xemu)".to_string())
        } else if let Some(p) = spec.strip_prefix("unix:") {
            #[cfg(unix)]
            { let s = UnixStream::connect(p).map_err(|e| format!("{p}: {e}"))?; s.set_read_timeout(Some(Duration::from_millis(300))).ok(); (Io::Unix(s), "MEGA65 (Xemu)".to_string()) }
            #[cfg(not(unix))]
            { return Err(format!("{p}: Unix-Sockets gibt es unter Windows nicht – tcp:127.0.0.1:Port verwenden")); }
        } else {
            let path = spec.strip_prefix("serial:").unwrap_or(spec); let (dev, baud) = path.split_once('@').unwrap_or((path, "2000000"));
            // Schnittstelle: roh, 8N1, kein Echo; Lesen kehrt nach 0,3 s ohne Daten zurück (VMIN=0, VTIME=3)
            let ok = Command::new("stty").args([if cfg!(target_os = "linux") { "-F" } else { "-f" }, dev, baud, "raw", "-echo", "cs8", "-cstopb", "-parenb", "min", "0", "time", "3"]).stdout(Stdio::null()).stderr(Stdio::null()).status().map_err(|e| format!("stty: {e}"))?;
            if !ok.success() { return Err(format!("{dev}: Schnittstelle lässt sich nicht einstellen")); }
            (Io::Serial(std::fs::OpenOptions::new().read(true).write(true).open(dev).map_err(|e| format!("{dev}: {e}"))?), "MEGA65 (Hardware)".to_string())
        };
        let mut m = Mega65 { expect_pc: None, io, child: None, paused: false, bp: None, label, watches: vec![], next_watch: 1, emulating: false, hit: None };
        m.send("r", Duration::from_secs(3)).map_err(|e| format!("keine Antwort des Monitors: {e}"))?;
        Ok(m)
    }

    /// Ordner mit der ROM-Datei `MEGA65.ROM` für Xemu: `VC_XEMU_SD`, sonst `~/.config/vintagecoder/mega65/` (wenn dort eine `MEGA65.ROM` liegt)
    pub fn xemu_sd_dir() -> Option<String> {
        if let Ok(d) = std::env::var("VC_XEMU_SD") { return Some(d); }
        let has_rom = |dir: &std::path::Path| std::fs::read_dir(dir).map_or(false, |rd| rd.flatten().any(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case("mega65.rom")));
        let mut dirs = vec![];
        if let Ok(h) = std::env::var("HOME") { dirs.push(std::path::PathBuf::from(h).join(".config/vintagecoder/mega65")); }
        // Linux: XDG-Konfigurationsordner und die Daten von Xemu selbst (~/.local/share/xemu-lgb/mega65, bzw. $XDG_DATA_HOME)
        #[cfg(target_os = "linux")]
        {
            let abs = |v: &str| std::env::var_os(v).map(std::path::PathBuf::from).filter(|p| p.is_absolute());
            if let Some(x) = abs("XDG_CONFIG_HOME") { dirs.push(x.join("vintagecoder/mega65")); }
            if let Some(x) = abs("XDG_DATA_HOME") { dirs.push(x.join("xemu-lgb/mega65")); }
            if let Ok(h) = std::env::var("HOME") { dirs.push(std::path::PathBuf::from(h).join(".local/share/xemu-lgb/mega65")); }
        }
        // Windows: Daten von Xemu selbst (%APPDATA%\xemu-lgb\mega65) und eigener Ordner unter %APPDATA%
        if let Ok(a) = std::env::var("APPDATA") { dirs.push(std::path::PathBuf::from(&a).join("vintagecoder").join("mega65")); dirs.push(std::path::PathBuf::from(a).join("xemu-lgb").join("mega65")); }
        dirs.into_iter().find(|d| has_rom(d)).map(|d| d.to_string_lossy().into_owned())
    }

    fn child_status(&mut self) -> Result<Option<std::process::ExitStatus>, String> { match &mut self.child { Some(c) => c.try_wait().map_err(|e| e.to_string()), None => Ok(None) } }

    /// Pfad der ROM-Datei (`MEGA65.ROM`, Groß-/Kleinschreibung egal) im Ordner `dir`
    pub fn rom_in(dir: &str) -> Option<String> {
        std::fs::read_dir(dir).ok()?.flatten().find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case("mega65.rom")).map(|e| e.path().to_string_lossy().into_owned())
    }

    /// Startet Xemu mit UART-Monitor (`sd_dir` enthält `MEGA65.ROM`) und verbindet sich.
    /// Übergibt den Xemu-Prozess an den Aufrufer (der ihn dann selbst beendet); `Drop` lässt ihn danach in Ruhe.
    pub fn take_child(&mut self) -> Option<Child> { self.child.take() }

    pub fn launch_xemu(exe: Option<&str>, sd_dir: &str) -> Result<Mega65, String> { Self::launch_xemu_with(exe, sd_dir, &[]) }

    /// Wie `launch_xemu`, mit zusätzlichen Kommandozeilenargumenten (Einstellungen des Benutzers).
    pub fn launch_xemu_with(exe: Option<&str>, sd_dir: &str, extra: &[String]) -> Result<Mega65, String> {
        let rom = Self::rom_in(sd_dir).ok_or_else(|| format!("{sd_dir}: keine MEGA65.ROM gefunden"))?;
        Self::launch_xemu_rom(exe, &rom, extra)
    }

    /// Wie `launch_xemu_with`, mit der ROM-Datei selbst (aus dem ROM-Verzeichnis, `-rom <datei>`)
    pub fn launch_xemu_rom(exe: Option<&str>, rom: &str, extra: &[String]) -> Result<Mega65, String> {
        let exe = exe.map(String::from).or_else(|| std::env::var("VC_XEMU").ok()).or_else(|| find_program("xmega65", &["/opt/homebrew/bin", "/usr/local/bin", "/Applications/xmega65.app/Contents/MacOS", "/Applications/Xemu"])).ok_or("xmega65 (Xemu) nicht gefunden – VC_XEMU setzen")?;
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0); let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let sock = std::env::temp_dir().join(format!("vc_xemu_{}_{n}.sock", std::process::id())); let _ = std::fs::remove_file(&sock);
        let log = std::env::temp_dir().join(format!("vc_xemu_{}_{n}.log", std::process::id()));
        let logf = std::fs::File::create(&log).map_err(|e| format!("{}: {e}", log.display()))?;
        // Monitor-Anschluss: Unix-Socket (macOS/Linux) bzw. TCP-Port (Windows: Xemu kennt dort nur `-uartmon :Port` und lauscht auf allen Schnittstellen,
        // deshalb ein zufälliger freier Port je Start und nur für die Dauer der Sitzung)
        #[cfg(unix)]
        let (uart_arg, spec) = (sock.to_string_lossy().into_owned(), format!("unix:{}", sock.display()));
        #[cfg(not(unix))]
        let (uart_arg, spec) = { let port = crate::free_port(); (format!(":{port}"), format!("tcp:127.0.0.1:{port}")) };
        let mut child = Command::new(&exe).args(["-besure", "-rom", rom]).args(std::env::var("VC_XEMU_ARGS").unwrap_or_default().split_whitespace()).args(extra).arg("-uartmon").arg(&uart_arg)
            .stdin(Stdio::null()).stdout(logf.try_clone().map_err(|e| e.to_string())?).stderr(logf).spawn().map_err(|e| format!("{exe}: {e}"))?;
        let tail = |log: &std::path::Path| -> String { let t = std::fs::read_to_string(log).unwrap_or_default(); let l: Vec<&str> = t.lines().collect(); l[l.len().saturating_sub(6)..].join(" | ") };
        let mut tries = 0;
        #[cfg(unix)]
        for _ in 0..300 {
            if sock.exists() { break; }
            if let Ok(Some(st)) = child.try_wait() { return Err(format!("Xemu beendet sich ({st}): {} (Protokoll: {})", tail(&log), log.display())); }
            std::thread::sleep(Duration::from_millis(100));
        }
        // Beim Start antwortet der Monitor erst verzögert: verbinden und einige Male neu versuchen
        // (Windows: der Port lauscht erst nach einigen Sekunden, ein Fehlversuch dauert dort nur Millisekunden)
        let max_tries = if cfg!(unix) { 6 } else { 80 };
        let mut m = loop {
            match Mega65::connect(&spec) {
                Ok(m) => break m,
                Err(e) => {
                    if let Ok(Some(st)) = child.try_wait() { return Err(format!("Xemu beendet sich ({st}): {} (Protokoll: {})", tail(&log), log.display())); }
                    tries += 1; if tries > max_tries { let _ = child.kill(); return Err(format!("{e} – Xemu-Protokoll: {} ({})", tail(&log), log.display())); }
                    if !cfg!(unix) { std::thread::sleep(Duration::from_millis(250)); }
                }
            }
        };
        m.child = Some(child);
        // Start (Hypervisor, ROM) abwarten: vorher stürzt Xemu bei Einzelschritten ab (die Protokolldatei wird gepuffert, taugt also nicht als Signal)
        for _ in 0..40 { if let Ok(Some(st)) = m.child_status() { return Err(format!("Xemu beendet sich ({st}): {} (Protokoll: {})", tail(&log), log.display())); } std::thread::sleep(Duration::from_millis(250)); }
        std::thread::sleep(Duration::from_secs(3));
        Ok(m)
    }

    /// Antwort lesen: bis zur Zeile `.` oder Zeitablauf
    fn send(&mut self, cmd: &str, timeout: Duration) -> Result<String, String> {
        let eol = if self.io.is_emulator() { "\n" } else { "\r" };      // Xemu: Zeilenende LF; Hardware: CR
        self.io.drain();
        self.io.write_all(format!("{cmd}{eol}").as_bytes()).map_err(|e| e.to_string())?;
        let (mut out, end, mut buf) = (Vec::<u8>::new(), Instant::now() + timeout, [0u8; 4096]);
        loop {
            match self.io.read(&mut buf) { Ok(0) => {} Ok(n) => out.extend_from_slice(&buf[..n]), Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {} Err(e) => return Err(e.to_string()) }
            let s = String::from_utf8_lossy(&out);
            if s.lines().any(|l| l.trim() == ".") { return Ok(s.into_owned()); }
            if Instant::now() > end { return Err(format!("keine Antwort auf '{cmd}'")); }
        }
    }

    /// Programm laden (`prg` mit zwei Bytes Ladeadresse) und den PC auf `entry` setzen; die Maschine bleibt angehalten.
    pub fn load_prg(&mut self, prg: &[u8], entry: u16) -> Result<(), String> {
        if prg.len() < 3 { return Err("PRG zu kurz".into()); }
        self.pause()?; let addr = prg[0] as usize | (prg[1] as usize) << 8;
        self.write_mem(addr as u16, &prg[2..])?;
        self.send(&format!("g {entry:x}"), Duration::from_secs(3))?; Ok(())
    }

    fn parse_regs(text: &str) -> Option<Regs> {
        let mut it = text.lines(); it.find(|l| l.trim_start().starts_with("PC "))?;
        let f: Vec<&str> = it.next()?.split_whitespace().collect(); if f.len() < 11 { return None; }
        let h = |s: &str| u32::from_str_radix(s, 16).ok();
        Some(Regs { pc: h(f[0])? as u16, a: h(f[1])? as u8, x: h(f[2])? as u8, y: h(f[3])? as u8, sp: h(f[6])? as u8, flags: h(f[10])? as u8,
                    extra: vec![("Z".into(), h(f[4])?), ("B".into(), h(f[5])?), ("SPH".into(), h(f[6])? >> 8), ("MAPH".into(), h(f[7])?), ("MAPL".into(), h(f[8])?)] })
    }
}

/// PC nach dem Haltepunkt-Befehl bei `addr`: Xemu hält erst **nach** dem Befehl an. `jsr` und `jmp` landen am Ziel
/// (bei `jmp *` wieder auf `addr`), alle anderen Befehle liegen höchstens 4 Bytes dahinter (`None`).
pub fn pc_after(code: &[u8]) -> Option<u16> {
    match code { [0x20 | 0x4C, lo, hi, ..] => Some(*lo as u16 | (*hi as u16) << 8), _ => None }
}

impl Mega65 {
    /// Schreib-Beobachtung: höchstens ~40 ms lang einzelschrittweise ausführen und nach jedem Schritt die Bereiche vergleichen
    fn watch_steps(&mut self) -> RunState {
        let end = Instant::now() + Duration::from_millis(40);
        while Instant::now() < end {
            if self.send("t", Duration::from_secs(8)).is_err() { self.emulating = false; self.paused = true; return RunState::Stopped; }
            for i in 0..self.watches.len() {
                let (a, n) = (self.watches[i].start, self.watches[i].end as usize - self.watches[i].start as usize + 1);
                let Ok(now) = self.read_mem(a, n) else { continue };
                if now != self.watches[i].snap { self.watches[i].snap = now; self.hit = Some(Hit::Watch(self.watches[i].id)); self.emulating = false; self.paused = true; return RunState::Stopped; }
            }
            if let Some(bp) = self.bp { if self.registers().map_or(false, |r| r.pc == bp) { self.emulating = false; self.paused = true; return RunState::Stopped; } }
        }
        RunState::Running
    }
}

impl Backend for Mega65 {
    fn name(&self) -> &str { &self.label }
    /// Der UART-Monitor setzt nur den PC (`g`) und kennt keine Zyklen: Register und Flags setzt der Vorspann der Falle
    fn trap_caps(&self) -> TrapCaps { TrapCaps { set_regs: false, cycles: false } }
    fn set_regs(&mut self, r: &Regs) -> Result<(), String> { if !self.paused { self.pause()?; } self.send(&format!("g {:x}", r.pc), Duration::from_secs(3)).map(|_| ()) }
    /// Läuft bis `addr`; Xemu meldet den Halt erst nach dem Befehl am Haltepunkt (bei `jsr` am Ziel, bei `jmp *` wieder an `addr`)
    fn run_until(&mut self, addr: u16, timeout: Duration) -> Result<bool, String> {
        let after = self.read_mem(addr, 3).ok().and_then(|c| pc_after(&c));
        self.set_breakpoint(addr)?;
        self.expect_pc = after;
        let res = (|| {
            self.cont()?;
            let end = Instant::now() + timeout;
            while Instant::now() < end {
                if self.state() == RunState::Stopped { let pc = self.registers()?.pc; return Ok(match after { Some(e) => pc == e, None => pc.wrapping_sub(addr) <= 4 }); }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(false)
        })();
        self.expect_pc = None;
        if res == Ok(false) { let _ = self.pause(); }
        let _ = self.clear_breakpoint(addr);
        res
    }
    fn caps(&self) -> Caps { Caps { registers: true, step: true, breakpoints: true, watch: true } }
    fn set_watch(&mut self, start: u16, end: u16, kind: WatchKind) -> Result<u32, String> {
        if kind != WatchKind::Write { return Err("MEGA65-Monitor: nur Schreib-Haltepunkte (Wertänderung) per Einzelschritt-Beobachtung".into()); }
        if end as usize - start as usize >= 256 { return Err("Bereich höchstens 256 Bytes".into()); }
        if self.watches.len() >= 4 { return Err("höchstens 4 beobachtete Bereiche".into()); }
        let snap = self.read_mem(start, end as usize - start as usize + 1)?;
        let id = self.next_watch; self.next_watch += 1; self.watches.push(Watch { id, start, end, snap }); Ok(id)
    }
    fn clear_watch(&mut self, id: u32) -> Result<(), String> { self.watches.retain(|w| w.id != id); Ok(()) }
    fn last_hit(&mut self) -> Option<Hit> { self.hit.take() }
    fn state(&mut self) -> RunState {
        if self.paused { return RunState::Stopped; }
        if self.emulating { return self.watch_steps(); }
        if let Some(bp) = self.bp {          // Halt am Haltepunkt: der PC bleibt dort stehen
            // Xemu hält erst nach dem Befehl am Haltepunkt an: der PC steht dann auf dem folgenden Befehl (höchstens 4 Bytes dahinter)
            let expect = self.expect_pc;
            let near = |pc: u16| match expect { Some(e) => pc == e, None => pc.wrapping_sub(bp) <= 4 };
            if let Ok(r) = self.registers() { if near(r.pc) { std::thread::sleep(Duration::from_millis(15)); if self.registers().map_or(false, |r2| r2.pc == r.pc) { self.paused = true; self.hit = Some(Hit::Exec(bp)); return RunState::Stopped; } } }
        }
        RunState::Running
    }
    fn registers(&mut self) -> Result<Regs, String> { let t = self.send("r", Duration::from_secs(3))?; Mega65::parse_regs(&t).ok_or_else(|| format!("Register nicht lesbar: {t:?}")) }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(len); let mut a = addr as usize;
        while out.len() < len && a <= 0xFFFF {
            let t = self.send(&format!("M 777{a:04x}"), Duration::from_secs(5))?; let mut got = vec![];
            for l in t.lines() { if let Some(rest) = l.trim().strip_prefix(':') { if let Some((_, h)) = rest.split_once(':') { if let Some(b) = hex_ok(h.trim()) { got.extend(b); } } } }
            if got.is_empty() { return Err("Speicher nicht lesbar".into()); }
            let take = got.len().min(len - out.len()).min(0x10000 - a); out.extend_from_slice(&got[..take]); a += got.len();
        }
        Ok(out)
    }
    /// Schreibt Speicher in Blöcken: Xemu nimmt lange Zeilen an (Eingabepuffer 4 KB, ein Befehl je Bild, rund 40 ms), die echte Hardware
    /// hat eine kurze Eingabezeile und bekommt weiterhin 24 Bytes je Befehl. Ein Block ohne Antwort wird einmal wiederholt
    /// (Schreiben ist wiederholbar), damit ein kurzer Aussetzer des Emulators nicht den ganzen Start kostet.
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        let block = if self.io.is_emulator() { 200 } else { 24 };
        for (i, c) in data.chunks(block).enumerate() {
            let bytes: Vec<String> = c.iter().map(|b| format!("{b:02x}")).collect();
            let cmd = format!("s 777{:04x} {}", addr as usize + i * block, bytes.join(" "));
            let t = match self.send(&cmd, Duration::from_secs(5)) { Ok(t) => t, Err(_) => self.send(&cmd, Duration::from_secs(10))? };
            if t.to_lowercase().contains("error") { return Err(format!("Schreiben fehlgeschlagen: {t:?}")); }
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> { if !self.paused { return Err("zuerst anhalten".into()); } self.send("t", Duration::from_secs(8)).map(|_| ()) }
    fn cont(&mut self) -> Result<(), String> {
        if self.watches.is_empty() { self.send("t0", Duration::from_secs(3))?; } else { self.emulating = true; self.hit = None; }     // Beobachtung: Maschine bleibt im Einzelschritt
        self.paused = false; Ok(())
    }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> { self.emulating = false; self.send("t1", Duration::from_secs(3))?; self.paused = true; Ok(()) }
    fn set_breakpoint(&mut self, addr: u16) -> Result<(), String> { self.send(&format!("b {addr:x}"), Duration::from_secs(3))?; self.bp = Some(addr); Ok(()) }
    fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String> { if self.bp == Some(addr) { self.send("b ffff", Duration::from_secs(3))?; self.bp = None; } Ok(()) }
    fn breakpoints(&self) -> Vec<u16> { self.bp.into_iter().collect() }
    fn stop(&mut self) {
        if self.child.is_none() { let _ = self.send("t0", Duration::from_secs(1)); }       // echte Hardware nicht angehalten zurücklassen
        if let Some(mut c) = self.child.take() { let _ = c.kill(); let _ = c.wait(); }
    }
}
impl Drop for Mega65 { fn drop(&mut self) { self.stop(); } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halt_nach_dem_befehl_am_haltepunkt() {
        assert_eq!(pc_after(&[0x20, 0x34, 0x12]), Some(0x1234), "jsr: Ziel");
        assert_eq!(pc_after(&[0x4C, 0x00, 0x20]), Some(0x2000), "jmp * landet wieder auf der eigenen Adresse");
        assert_eq!(pc_after(&[0xA9, 0x01, 0x60]), None);
        assert_eq!(pc_after(&[0x20, 0x34]), None);
    }
}
