// SPDX-License-Identifier: GPL-3.0-or-later
//! T0 des Testkonzepts: Vorversuch mit echten Emulatoren (Register setzen, Falle `jsr ziel` + `jmp *`, Zyklen aus LIN/CYC).
//! Läuft nur mit `VC_EMU_TESTS=1` (VICE x64sc und atari800 installiert); es läuft immer nur ein Emulator zugleich.
//! Eigene kleine Protokollclients, damit `vc-emu` für den Versuch unverändert bleibt.
//! Mit `-- --nocapture --test-threads=1` zeigt der Lauf die Messtabellen.
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ONE_EMULATOR: Mutex<()> = Mutex::new(());
fn enabled() -> bool { std::env::var("VC_EMU_TESTS").is_ok() }

// ---------------- VICE (Binärmonitor) ----------------

struct Vice { #[allow(dead_code)] port: u16, child: Child, s: TcpStream, rx: Receiver<(u8, u8, u32, Vec<u8>)>, id: u32, names: HashMap<u8, String>, ids: HashMap<String, u8> }

impl Vice {
    fn start() -> Vice {
        let exe = vc_emu::find_program("x64sc", &["/Applications/vice-arm64-gtk3-3.10/bin", "/opt/homebrew/bin"]).expect("x64sc nicht gefunden");
        let port = vc_emu::free_port();
        let child = Command::new(exe).args(["-binarymonitor", "-binarymonitoraddress", &format!("ip4://127.0.0.1:{port}"), "+confirmonexit"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let mut s = None;
        for _ in 0..80 { if let Ok(x) = TcpStream::connect(("127.0.0.1", port)) { s = Some(x); break; } std::thread::sleep(Duration::from_millis(250)); }
        let s = s.expect("Monitor nicht erreichbar");
        let mut r = s.try_clone().unwrap();
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut h = [0u8; 12];
            while r.read_exact(&mut h).is_ok() {
                let len = u32::from_le_bytes([h[2], h[3], h[4], h[5]]) as usize;
                let mut b = vec![0u8; len];
                if r.read_exact(&mut b).is_err() { break; }
                if tx.send((h[6], h[7], u32::from_le_bytes([h[8], h[9], h[10], h[11]]), b)).is_err() { break; }
            }
        });
        let mut v = Vice { port, child, s, rx, id: 1, names: HashMap::new(), ids: HashMap::new() };
        let b = v.cmd(0x83, &[0]);
        let n = u16::from_le_bytes([b[0], b[1]]) as usize; let mut p = 2;
        for _ in 0..n { let sz = b[p] as usize; let nl = b[p + 3] as usize; let name = String::from_utf8_lossy(&b[p + 4..p + 4 + nl]).to_string(); v.names.insert(b[p + 1], name.clone()); v.ids.insert(name, b[p + 1]); p += 1 + sz; }
        v
    }
    fn send(&mut self, code: u8, body: &[u8]) -> u32 {
        let id = self.id; self.id += 1;
        let mut m = vec![2, 2]; m.extend((body.len() as u32).to_le_bytes()); m.extend(id.to_le_bytes()); m.push(code); m.extend_from_slice(body);
        self.s.write_all(&m).unwrap(); id
    }
    /// Befehl senden und die Antwort mit gleicher Kennung abwarten; Ereignisse (Kennung $FFFFFFFF) werden dabei verworfen
    fn cmd(&mut self, code: u8, body: &[u8]) -> Vec<u8> {
        let id = self.send(code, body);
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            let (_, err, rid, b) = self.rx.recv_timeout(end.saturating_duration_since(Instant::now())).unwrap_or_else(|_| panic!("keine Antwort auf 0x{code:02X}"));
            if rid == id { assert_eq!(err, 0, "VICE-Befehl 0x{code:02X}: Fehler 0x{err:02X}"); return b; }
        }
    }
    fn regs(&mut self) -> HashMap<String, u16> {
        let b = self.cmd(0x31, &[0]);
        let n = u16::from_le_bytes([b[0], b[1]]) as usize; let mut p = 2; let mut m = HashMap::new();
        for _ in 0..n { let sz = b[p] as usize; m.insert(self.names[&b[p + 1]].clone(), u16::from_le_bytes([b[p + 2], b[p + 3]])); p += 1 + sz; }
        m
    }
    fn set_regs(&mut self, r: &[(&str, u16)]) {
        let mut body = vec![0u8]; body.extend((r.len() as u16).to_le_bytes());
        for (n, v) in r { body.extend([3, self.ids[*n]]); body.extend(v.to_le_bytes()); }
        self.cmd(0x32, &body);
    }
    fn write(&mut self, addr: u16, data: &[u8]) {
        let mut body = vec![0u8]; body.extend(addr.to_le_bytes()); body.extend((addr + data.len() as u16 - 1).to_le_bytes()); body.push(0); body.extend([0, 0]); body.extend_from_slice(data);
        self.cmd(0x02, &body);
    }
    fn read(&mut self, addr: u16, len: u16) -> Vec<u8> {
        let mut body = vec![0u8]; body.extend(addr.to_le_bytes()); body.extend((addr + len - 1).to_le_bytes()); body.push(0); body.extend([0, 0]);
        let b = self.cmd(0x01, &body); let n = u16::from_le_bytes([b[0], b[1]]) as usize; b[2..2 + n].to_vec()
    }
    fn set_bp(&mut self, addr: u16) -> u32 {
        let mut body = vec![]; body.extend(addr.to_le_bytes()); body.extend(addr.to_le_bytes()); body.extend([1, 1, 4, 0]);
        let b = self.cmd(0x12, &body); u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }
    fn del_bp(&mut self, n: u32) { self.cmd(0x13, &n.to_le_bytes()); }
    /// Weiterlaufen und auf das Anhalten warten (Ereignis 0x62 „gestoppt“ mit PC); `None` bei Zeitüberschreitung
    fn run_until_stop(&mut self, limit: Duration) -> Option<u16> {
        self.send(0xAA, &[]);
        let end = Instant::now() + limit;
        loop {
            let left = end.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) { Ok((0x62, _, 0xFFFF_FFFF, b)) => return Some(u16::from_le_bytes([b[0], b[1]])), Ok(m) => { if std::env::var("VC_VICE_TRACE").is_ok() { eprintln!("Ereignis 0x{:02X} Fehler {} Id {:X} {:02X?}", m.0, m.1, m.2, m.3); } } Err(_) => return None }
        }
    }
}
/// Beenden: Unter macOS ist `bin/x64sc` ein Shell-Skript, das VICE als Kindprozess startet – `child.kill()` trifft nur das Skript und
/// lässt VICE weiterlaufen (Befund T0). Deshalb zusätzlich über die eindeutige Monitor-Adresse beenden.
impl Drop for Vice { fn drop(&mut self) {
    let _ = self.child.kill(); let _ = self.child.wait();
    #[cfg(unix)] { let _ = Command::new("pkill").args(["-f", &format!("binarymonitoraddress ip4://127.0.0.1:{}", self.port)]).status(); }
} }

/// Zyklen zwischen zwei Messpunkten aus Rasterzeile und Zyklus in der Zeile (`per_line` = Zyklen je Zeile, `lines` = Zeilen je Bild)
fn delta(a: (u16, u16), b: (u16, u16), per_line: i64, lines: i64) -> i64 {
    let (la, lb) = (a.0 as i64, b.0 as i64);
    let dl = (lb - la).rem_euclid(lines);
    dl * per_line + (b.1 as i64 - a.1 as i64)
}

#[test]
fn vice_registers_trap_and_cycles() {
    if !enabled() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let _g = ONE_EMULATOR.lock().unwrap_or_else(|e| e.into_inner());
    let mut v = Vice::start();
    std::thread::sleep(Duration::from_secs(3));                     // Hochfahren (KERNAL), danach liegt $C000 frei
    let mut names: Vec<&String> = v.ids.keys().collect(); names.sort();
    eprintln!("VICE-Register: {names:?}");

    // (1) Register setzen und zurücklesen
    v.set_regs(&[("A", 0x12), ("X", 0x34), ("Y", 0x56), ("SP", 0xF0), ("FL", 0xC7), ("PC", 0xC000)]);
    let r = v.regs();
    eprintln!("nach Setzen: A={:02X} X={:02X} Y={:02X} SP={:02X} FL={:02X} PC={:04X} LIN={:?} CYC={:?}", r["A"], r["X"], r["Y"], r["SP"], r["FL"], r["PC"], r.get("LIN"), r.get("CYC"));
    assert_eq!((r["A"], r["X"], r["Y"], r["SP"], r["PC"]), (0x12, 0x34, 0x56, 0xF0, 0xC000));
    assert_eq!(r["FL"] & 0xCF, 0xC7, "Flags N V Z C direkt setzbar");

    // (2) Falle: C000 `jsr C100`, C003 `jmp C003`; Unterprogramm legt P (php/pla) in $C200 ab
    v.write(0xC000, &[0x20, 0x00, 0xC1, 0x4C, 0x03, 0xC0]);
    v.write(0xC100, &[0x08, 0x68, 0x8D, 0x00, 0xC2, 0x60]);       // php; pla; sta $C200; rts
    let bp = v.set_bp(0xC003);
    eprintln!("Haltepunkt Nr. {bp}; Liste (0x14): {:02X?}", v.cmd(0x14, &[]));
        let t = Instant::now();
    v.set_regs(&[("FL", 0xC7), ("A", 0x12), ("SP", 0xF0), ("PC", 0xC000)]);     // I gesetzt: sonst springt VICE sofort in den KERNAL-IRQ
    eprintln!("Speicher C000: {:02X?}  C100: {:02X?}", v.read(0xC000, 6), v.read(0xC100, 6));
    let pc = match v.run_until_stop(Duration::from_secs(3)) { Some(p) => p, None => { v.send(0x81, &[]); std::thread::sleep(Duration::from_millis(300)); let r = v.regs(); panic!("Falle nicht erreicht: PC={:04X} SP={:02X} FL={:02X} Speicher C200={:02X?}", r["PC"], r["SP"], r["FL"], v.read(0xC200, 1)) } };
    eprintln!("Falle erreicht: PC={pc:04X} nach {:?}", t.elapsed());
    assert_eq!(pc, 0xC003);
    let r = v.regs();
    let pushed = v.read(0xC200, 1)[0];
    eprintln!("Flags beim Aufruf (php): ${pushed:02X} (gesetzt waren $C7), A={:02X}", r["A"]);
    assert_eq!(pushed & 0xCF, 0xC7, "Flags vor dem Aufruf kommen im Unterprogramm an");
    assert_eq!(r["PC"], 0xC003);

    // (3) Zyklen aus LIN/CYC: Unterprogramm mit N nop (2 Zyklen) → jsr 6 + 2N + rts 6; I-Flag gesetzt (keine IRQs)
    eprintln!("\n  N   erwartet   gemessen   Δ    LIN/CYC Start → Halt");
    let mut worst = 0i64;
    for n in [0usize, 1, 10, 30, 100, 200, 500, 1000, 2000] {
        let mut code = vec![0xEAu8; n]; code.push(0x60);
        v.write(0xC110, &code);
        v.write(0xC000, &[0x20, 0x10, 0xC1]);
        v.set_regs(&[("FL", 0x04), ("SP", 0xF0), ("PC", 0xC000)]);
        let a = v.regs();
        let (s_lin, s_cyc) = (a["LIN"], a["CYC"]);
        let tw = Instant::now();
        let hit = v.run_until_stop(Duration::from_secs(5));
        let wall = tw.elapsed();
        assert_eq!(hit, Some(0xC003), "N={n}");
        let b = v.regs();
        let (per_line, lines) = (63, 312);
        let got = delta((s_lin, s_cyc), (b["LIN"], b["CYC"]), per_line, lines);
        let expect = 12 + 2 * n as i64;
        eprintln!("{n:>5} {expect:>9} {got:>10} {:>4}   {s_lin}/{s_cyc} → {}/{}   Wartezeit {:?}", got - expect, b["LIN"], b["CYC"], wall);
        if expect < 312 * 63 - 200 { worst = worst.max((got - expect).abs()); }
    }
    eprintln!("größte Abweichung: {worst} Zyklen");
    assert!(worst <= 1, "Messung auf ±1 Zyklus genau, solange die Dauer unter einem Bild bleibt");
    v.del_bp(bp);

    // (4) Zeitüberschreitung: Falle ohne Haltepunkt wird nie erreicht
    v.write(0xC110, &[0x4C, 0x10, 0xC1]);                          // jmp * im Unterprogramm
    v.set_regs(&[("PC", 0xC000)]);
    assert_eq!(v.run_until_stop(Duration::from_millis(800)), None, "ohne Halt läuft VICE weiter, der Läufer bricht nach dem Zeitlimit ab");
    v.send(0x81, &[]);                                              // Ping hält VICE an (jeder Befehl hält an)
}

// ---------------- atari800 (Textmonitor) ----------------

struct A8 { child: Child, stdin: std::process::ChildStdin, rx: Receiver<String> }

impl A8 {
    fn start() -> A8 {
        let exe = vc_emu::find_program("atari800", &["/opt/homebrew/bin", "/usr/local/bin"]).expect("atari800 nicht gefunden");
        let mut child = Command::new(exe).args(["-xl", "-pal", "-no-autosave-config", "-nobasic", "-monitor"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let stdin = child.stdin.take().unwrap(); let out = child.stdout.take().unwrap();
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut r = BufReader::new(out); let mut b = String::new();
            while r.read_line(&mut b).map_or(0, |n| n) > 0 {
                let mut l = String::new(); let mut esc = false;
                for c in b.trim_end().chars() { if esc { if c.is_ascii_alphabetic() { esc = false; } } else if c == '\x1b' { esc = true; } else { l.push(c); } }
                b.clear(); if tx.send(l).is_err() { break; }
            }
        });
        let mut a = A8 { child, stdin, rx };
        a.wait(|l| l.contains(" PC=") && l.contains("A="), 15).expect("Monitor meldet sich nicht");
        a
    }
    fn say(&mut self, c: &str) { writeln!(self.stdin, "{c}").unwrap(); self.stdin.flush().unwrap(); }
    fn wait(&mut self, pred: impl Fn(&str) -> bool, secs: u64) -> Option<String> {
        let end = Instant::now() + Duration::from_secs(secs);
        loop { match self.rx.recv_timeout(end.saturating_duration_since(Instant::now())) { Ok(l) if pred(&l) => return Some(l), Ok(_) => {} Err(_) => return None } }
    }
    fn drain(&mut self) { while self.rx.try_recv().is_ok() {} }
    /// `show` → Zeile, Zyklus, Register-Zeile
    fn show(&mut self) -> (u32, u32, String) {
        self.drain(); self.say("show");
        let l = self.wait(|l| l.contains(" PC=") && l.contains("A="), 5).expect("show ohne Antwort");
        let mut n = l.split_whitespace().take(2).map(|w| w.parse::<u32>().unwrap());
        (n.next().unwrap(), n.next().unwrap(), l)
    }
    fn cont_until_break(&mut self, secs: u64) -> bool { self.drain(); self.say("cont"); self.wait(|l| l.starts_with("(breakpoint at"), secs).is_some() }
    fn mem(&mut self, addr: u16, len: usize) -> Vec<u8> {
        self.drain(); self.say(&format!("m {addr:04x}"));
        let l = self.wait(|l| l.starts_with(&format!("{addr:04X}:")), 5).expect("m ohne Antwort");
        l[5..].split_whitespace().take(len).map(|h| u8::from_str_radix(h, 16).unwrap()).collect()
    }
}
impl Drop for A8 { fn drop(&mut self) { let _ = self.child.kill(); let _ = self.child.wait(); } }

fn hexs(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ") }

#[test]
fn atari800_registers_trap_and_cycles() {
    if !enabled() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let _g = ONE_EMULATOR.lock().unwrap_or_else(|e| e.into_inner());
    let mut a = A8::start();

    // (1) Register setzen: A X Y S PC und die Flags D, I gehen; N V Z C werden vom Monitor angenommen, aber beim Fortsetzen verworfen
    a.say("c 0600 20 10 06 4c 03 06");                              // jsr $0610; jmp $0603
    a.say("c 0610 08 68 8d 00 07 60");                              // php; pla; sta $0700; rts
    a.say("bpc 0603");
    for c in ["setpc 0600", "seta 77", "setx 12", "sety 34", "sets f0", "seti 1", "setd 1", "setn 1", "setv 1", "setz 1", "setc 1"] { a.say(c); }
    let (_, _, l) = a.show();
    eprintln!("nach set…: {l}");
    assert!(l.contains("A=77 X=12 Y=34 S=F0") && l.contains("PC=0600"), "{l}");
    assert!(a.cont_until_break(10), "Falle nicht erreicht");
    let (_, _, l) = a.show();
    let pushed = a.mem(0x0700, 1)[0];
    eprintln!("nach der Falle: {l}; php im Unterprogramm legte ${pushed:02X} ab (gesetzt: N V D I Z C)");
    assert!(l.contains("A=") && l.contains("X=12 Y=34"), "{l}");
    assert_eq!(pushed & 0x0C, 0x0C, "D und I kommen an");
    assert_eq!(pushed & 0xC3, 0, "N V Z C gehen beim Fortsetzen verloren (Befund atari800 7.2.1)");

    // (2) Ausweg: Flags im Vorspann der Falle setzen (lda #P; pha; … plp), dann jsr; Haltepunkt zuerst am jsr, danach am jmp
    a.say("c 0600 a9 c3 48 a9 77 a2 12 a0 34 28 20 10 06 4c 0d 06");
    a.say("setpc 0600"); a.say("bpc 060a");
    assert!(a.cont_until_break(10));
    let (l1, c1, l_start) = a.show();
    a.say("bpc 060d");
    assert!(a.cont_until_break(10));
    let (l2, c2, l_end) = a.show();
    let pushed = a.mem(0x0700, 1)[0];
    eprintln!("Vorspann: Start {l_start}\n          Halt  {l_end}\n          php legte ${pushed:02X} ab (gesetzt: $C3)");
    assert_eq!(pushed & 0xCF, 0xC3, "Flags N V Z C kommen über den Vorspann an");
    let got = (l2 as i64 - l1 as i64) * 114 + (c2 as i64 - c1 as i64);
    eprintln!("Zyklen php 3 + pla 4 + sta 4 + rts 6 + jsr 6 = 23, gemessen {got} (Zeile {l1}/{c1} → {l2}/{c2})");
    assert_eq!(got, 23);

    // (3) Messreihe: N nop; die Differenz enthält Speicherauffrischung und ANTIC-DMA (gestohlene Zyklen), ist also Strahlzeit
    eprintln!("\n  N  erwartet  gemessen  Δ");
    let mut deltas = vec![];
    for n in [0usize, 10, 40, 100, 300] {
        let mut code = vec![0xEAu8; n]; code.push(0x60);
        a.say("c 0600 a9 04 48 28 20 10 06 4c 07 06");             // lda #$04; pha; plp; jsr $0610; jmp $0607 … Zieladresse unten korrigiert
        // Adresse des jmp: 0600 a9 04 | 0602 48 | 0603 28 | 0604 20 10 06 | 0607 4c 07 06
        for (i, ch) in code.chunks(16).enumerate() { a.say(&format!("c {:04x} {}", 0x0610 + i * 16, hexs(ch))); }
        a.say("setpc 0600"); a.say("bpc 0604");
        assert!(a.cont_until_break(10));
        let (s_lin, s_cyc, _) = a.show();
        a.say("bpc 0607");
        assert!(a.cont_until_break(10));
        let (e_lin, e_cyc, _) = a.show();
        let got = (e_lin as i64 - s_lin as i64).rem_euclid(312) * 114 + (e_cyc as i64 - s_cyc as i64);
        let exp = 12 + 2 * n as i64;
        eprintln!("{n:>4} {exp:>9} {got:>9} {:>4}   {s_lin}/{s_cyc} → {e_lin}/{e_cyc}", got - exp);
        deltas.push((n, exp, got));
    }
    assert!(deltas.iter().all(|(_, e, g)| g >= e), "gemessen ≥ erwartet: DMA und Auffrischung kommen hinzu, nie weniger");

    // (4) Zeitlimit: ohne Haltepunkt keine Meldung; der Läufer beendet den Emulator
    a.say("c 0610 4c 10 06"); a.say("setpc 0600"); a.say("bpc ffff");
    assert!(!a.cont_until_break(2), "ohne Haltepunkt kommt keine Meldung");
}

/// Nebenbefund: Was passiert in der Falle, wenn das I-Flag beim Start gelöscht ist (KERNAL-IRQ läuft mit)?
#[test]
fn vice_trap_with_interrupts_enabled() {
    if !enabled() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let _g = ONE_EMULATOR.lock().unwrap_or_else(|e| e.into_inner());
    let mut v = Vice::start();
    std::thread::sleep(Duration::from_secs(3));
    v.write(0xC000, &[0x20, 0x10, 0xC1, 0x4C, 0x03, 0xC0]);
    v.write(0xC110, &[0xEA, 0xEA, 0x60]);
    let bp = v.set_bp(0xC003);
    for (name, fl, sp) in [("I gelöscht, SP=F0", 0x00u16, 0xF0u16), ("I gelöscht, SP=FF", 0x00, 0xFF), ("I gelöscht, CIA1-Timer-IRQ aus", 0x00, 0xF0)] {
        if name.contains("CIA1") { v.write(0xDC0D, &[0x7F]); }
        v.set_regs(&[("FL", fl), ("SP", sp), ("PC", 0xC000)]);
        let hit = v.run_until_stop(Duration::from_secs(2));
        let r = { if hit.is_none() { v.send(0x81, &[]); std::thread::sleep(Duration::from_millis(300)); } v.regs() };
        eprintln!("{name}: Halt an der Falle: {:?}; danach PC={:04X} SP={:02X} FL={:02X}", hit.map(|p| format!("{p:04X}")), r["PC"], r["SP"], r["FL"]);
    }
    v.del_bp(bp);
}
