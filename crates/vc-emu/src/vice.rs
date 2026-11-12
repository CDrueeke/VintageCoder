// SPDX-License-Identifier: GPL-3.0-or-later
//! VICE-Adapter über das Binary-Monitor-Protokoll (API v2, Port frei wählbar).
use crate::streams::video::Frame;
use crate::{find_program, free_port, Backend, Caps, Cycles, Hit, Regs, RunState, TrapCaps, WatchKind};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Reply = (u8, u8, Vec<u8>);

pub struct Vice {
    child: Child,
    /// Monitor-Adresse (`ip4://127.0.0.1:<port>`), eindeutig je Instanz: Kennzeichen zum Beenden aller Prozesse dieser Instanz
    marker: String,
    stream: TcpStream,
    pending: Arc<Mutex<HashMap<u32, Sender<Reply>>>>,
    running: Arc<AtomicBool>,
    stop_pc: Arc<AtomicU32>,
    last_cp: Arc<AtomicU32>,
    watches: Vec<(u32, u16, u16, WatchKind)>,
    next_id: u32,
    reg_names: HashMap<u8, (String, u8)>,
    bps: Vec<(u16, u32)>,
    banks: Option<HashMap<String, u16>>,
    /// VICE-Version (Haupt, Neben) laut Monitor; `None`, wenn VICE sie nicht meldet
    version: Option<(u8, u8)>,
    /// Programmname (`x64sc`, `x128`, …): Zyklenmessung gibt es nur für `x64sc`
    system: String,
    /// (Zyklen je Zeile, Zeilen je Bild), beim ersten Bedarf aus der Videonorm des Emulators bestimmt
    video: Option<(u32, u32)>,
    /// Bildquelle beim C128: `true` = VIC-II (Standard), `false` = VDC (80 Spalten)
    vic_source: bool,
    frame_no: u16,
}

const MON_SIZE: usize = 12;

impl Vice {
    /// Startet `x64sc` (oder `program`), verbindet sich und lädt `prg` per Autostart. Der Emulator läuft danach.
    pub fn launch(prg: &str, program: Option<&str>) -> Result<Vice, String> { Self::launch_system("x64sc", prg, program) }

    /// Wie `launch`, aber mit wählbarem VICE-Programm (`x64sc`, `x128`, `xplus4`).
    pub fn launch_system(name: &str, prg: &str, program: Option<&str>) -> Result<Vice, String> { Self::launch_with(name, prg, program, &[]) }

    /// Wie `launch_system`, mit zusätzlichen Kommandozeilenargumenten (Einstellungen des Benutzers).
    pub fn launch_with(name: &str, prg: &str, program: Option<&str>, extra: &[String]) -> Result<Vice, String> {
        let exe = program.map(String::from).or_else(|| find_program(name, &["/Applications/vice-arm64-gtk3-3.10/bin", "/Applications/vice-arm64-gtk3-3.9/bin", "/opt/homebrew/bin", "/usr/local/bin"]))
            .ok_or(format!("{name} (VICE) nicht gefunden"))?;
        let port = free_port();
        // Windows: Der Autostart-Befehl des Binär-Monitors lässt VICE 3.9 abstürzen (Zugriffsverletzung, Exitcode 0xC0000005), deshalb
        // wird das Programm dort über die Kommandozeile (`-autostart`) geladen; der Monitor ist schon vor dem Programmstart verbunden.
        let cli_autostart: Vec<String> = if cfg!(windows) && !prg.is_empty() { vec!["-autostart".into(), prg.to_string()] } else { vec![] };
        let marker = format!("ip4://127.0.0.1:{port}");
        let child = Command::new(&exe).args(["-binarymonitor", "-binarymonitoraddress", &marker, "+confirmonexit"]).args(&cli_autostart).args(extra)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| format!("{exe}: {e}"))?;
        let mut stream = None;
        for _ in 0..80 { if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) { stream = Some(s); break; } std::thread::sleep(Duration::from_millis(250)); }
        let Some(stream) = stream else { let mut c = child; terminate(&mut c, &marker); return Err("VICE-Monitor nicht erreichbar".into()) };
        let pending: Arc<Mutex<HashMap<u32, Sender<Reply>>>> = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));
        let stop_pc = Arc::new(AtomicU32::new(0));
        let last_cp = Arc::new(AtomicU32::new(0));
        let mut rs = stream.try_clone().map_err(|e| e.to_string())?;
        let (p2, r2, s2, c2) = (pending.clone(), running.clone(), stop_pc.clone(), last_cp.clone());
        let trace = std::env::var("VC_VICE_TRACE").is_ok();   // Diagnose: alle Antworten und Ereignisse des Monitors auf stderr
        std::thread::spawn(move || {
            let mut h = [0u8; MON_SIZE];
            while rs.read_exact(&mut h).is_ok() {
                let len = u32::from_le_bytes([h[2], h[3], h[4], h[5]]) as usize;
                let (rtype, err, id) = (h[6], h[7], u32::from_le_bytes([h[8], h[9], h[10], h[11]]));
                let mut body = vec![0u8; len];
                if rs.read_exact(&mut body).is_err() { break; }
                if trace { eprintln!("VICE <- Typ 0x{rtype:02X} Fehler 0x{err:02X} Id {id:#X} ({len} Bytes)"); }
                if id == 0xFFFF_FFFF {
                    match rtype { 0x62 if body.len() >= 2 => { s2.store(u16::from_le_bytes([body[0], body[1]]) as u32, Ordering::SeqCst); r2.store(false, Ordering::SeqCst); } 0x63 => { c2.store(0, Ordering::SeqCst); r2.store(true, Ordering::SeqCst); }
                        0x11 if body.len() >= 5 && body[4] == 1 => c2.store(u32::from_le_bytes([body[0], body[1], body[2], body[3]]), Ordering::SeqCst),   // Haltepunkt getroffen
                        _ => {} }
                } else if let Some(tx) = p2.lock().unwrap().remove(&id) { let _ = tx.send((rtype, err, body)); }
            }
            r2.store(false, Ordering::SeqCst);
        });
        let mut v = Vice { child, marker, stream, pending, running, stop_pc, last_cp, watches: vec![], next_id: 1, reg_names: HashMap::new(), bps: vec![], banks: None, version: None, system: name.to_string(), video: None, vic_source: true, frame_no: 0 };
        v.ping()?;
        // Version erfragen (VICE-Info 0x85: Länge, Versionsbytes Haupt/Neben/Bau/…); ohne Antwort bleibt sie unbekannt
        if let Ok(b) = v.cmd(0x85, &[], Duration::from_secs(5)) { if b.len() >= 3 && b[0] >= 2 { v.version = Some((b[1], b[2])); } }
        if std::env::var("VC_VICE_TRACE").is_ok() { eprintln!("VICE-Version laut Monitor: {:?}", v.version); }
        v.load_reg_names()?;
        // Ohne Programmdatei (z. B. Steckmodul über `extra`) gibt es nichts zu laden
        if !prg.is_empty() && cli_autostart.is_empty() {
            let mut body = vec![1u8, 0, 0, prg.len() as u8];
            body.extend_from_slice(prg.as_bytes());
            v.cmd(0xDD, &body, Duration::from_secs(20))?;
        }
        Ok(v)
    }

    fn cmd(&mut self, code: u8, body: &[u8], timeout: Duration) -> Result<Vec<u8>, String> {
        let id = self.next_id;
        self.next_id += 1;
        let (tx, rx): (Sender<Reply>, Receiver<Reply>) = channel();
        self.pending.lock().unwrap().insert(id, tx);
        let mut msg = vec![2, 2];
        msg.extend((body.len() as u32).to_le_bytes());
        msg.extend(id.to_le_bytes());
        msg.push(code);
        msg.extend_from_slice(body);
        if std::env::var("VC_VICE_TRACE").is_ok() { eprintln!("VICE -> Befehl 0x{code:02X} Id {id} ({} Bytes)", body.len()); }
        self.stream.write_all(&msg).map_err(|e| e.to_string())?;
        let (_, err, b) = rx.recv_timeout(timeout).map_err(|_| format!("Zeitüberschreitung bei VICE-Befehl 0x{code:02X}"))?;
        if err != 0 { return Err(format!("VICE-Befehl 0x{code:02X}: Fehlercode 0x{err:02X}")); }
        Ok(b)
    }
    fn ping(&mut self) -> Result<(), String> { self.cmd(0x81, &[], Duration::from_secs(5)).map(|_| ()) }
    /// Speichersichten des Rechners (Name → Kennung), z. B. `ram`, `rom`, `io`, `cpu`
    fn load_banks(&mut self) -> Result<(), String> {
        let b = self.cmd(0x82, &[0], Duration::from_secs(5))?;
        let n = u16::from_le_bytes([b[0], b[1]]) as usize; let mut p = 2; let mut m = HashMap::new();
        for _ in 0..n { let sz = b[p] as usize; let id = u16::from_le_bytes([b[p + 1], b[p + 2]]); let nl = b[p + 3] as usize; m.insert(String::from_utf8_lossy(&b[p + 4..p + 4 + nl]).to_lowercase(), id); p += 1 + sz; }
        self.banks = Some(m); Ok(())
    }
    fn read_bank(&mut self, bank: u16, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(len); let mut a = addr as usize;
        while out.len() < len && a <= 0xFFFF {
            let end = (a + len - out.len() - 1).min(0xFFFF);
            let mut body = vec![0u8]; body.extend((a as u16).to_le_bytes()); body.extend((end as u16).to_le_bytes()); body.push(0); body.extend(bank.to_le_bytes());
            let b = self.cmd(0x01, &body, Duration::from_secs(5))?; let n = u16::from_le_bytes([b[0], b[1]]) as usize;
            out.extend_from_slice(&b[2..2 + n]); a = end + 1;
        }
        Ok(out)
    }

    /// Ganzzahl-Ressource des Emulators (Befehl 0x51), z. B. `MachineVideoStandard`
    pub fn resource_int(&mut self, name: &str) -> Result<i64, String> {
        let mut body = vec![name.len() as u8]; body.extend_from_slice(name.as_bytes());
        let b = self.cmd(0x51, &body, Duration::from_secs(5))?;
        if b.len() < 2 || b[0] != 1 { return Err(format!("Ressource {name} ist keine Zahl")); }
        let n = (b[1] as usize).min(4);
        if b.len() < 2 + n { return Err(format!("Ressource {name}: Antwort zu kurz")); }
        Ok(b[2..2 + n].iter().enumerate().fold(0i64, |acc, (i, x)| acc | (*x as i64) << (8 * i)))
    }

    /// Rohbild (Befehl 0x84) und Palette (0x91) als RGBA. Der Aufrufer sorgt dafür, dass die Maschine danach wieder läuft (siehe `grab_frame`).
    fn display_rgba(&mut self) -> Result<Frame, String> {
        let vic = self.vic_source as u8;
        let img = self.cmd(0x84, &[vic, 0], Duration::from_secs(5))?;
        let pal = self.cmd(0x91, &[vic], Duration::from_secs(5))?;
        let mut f = parse_display(&img, &parse_palette(&pal)?)?;
        self.frame_no = self.frame_no.wrapping_add(1);
        f.frame_no = self.frame_no;
        Ok(f)
    }

    fn load_reg_names(&mut self) -> Result<(), String> {
        let b = self.cmd(0x83, &[0], Duration::from_secs(5))?;
        let n = u16::from_le_bytes([b[0], b[1]]) as usize;
        let mut p = 2;
        for _ in 0..n {
            let sz = b[p] as usize;
            let (id, bits, nl) = (b[p + 1], b[p + 2], b[p + 3] as usize);
            self.reg_names.insert(id, (String::from_utf8_lossy(&b[p + 4..p + 4 + nl]).into_owned(), bits));
            p += 1 + sz;
        }
        Ok(())
    }
    fn wait_stopped(&self, ms: u64) -> bool {
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end { if !self.running.load(Ordering::SeqCst) { return true; } std::thread::sleep(Duration::from_millis(10)); }
        false
    }
}

impl Backend for Vice {
    fn name(&self) -> &str { "VICE (x64sc)" }
    fn state(&mut self) -> RunState { if self.running.load(Ordering::SeqCst) { RunState::Running } else { RunState::Stopped } }
    fn registers(&mut self) -> Result<Regs, String> {
        let b = self.cmd(0x31, &[0], Duration::from_secs(5))?;
        let n = u16::from_le_bytes([b[0], b[1]]) as usize;
        let (mut p, mut r) = (2, Regs::default());
        for _ in 0..n {
            let sz = b[p] as usize;
            let (id, val) = (b[p + 1], u16::from_le_bytes([b[p + 2], b[p + 3]]));
            match self.reg_names.get(&id).map(|(n, _)| n.as_str()) {
                Some("PC") => r.pc = val, Some("A") => r.a = val as u8, Some("X") => r.x = val as u8, Some("Y") => r.y = val as u8,
                Some("SP") => r.sp = val as u8, Some("FL") => r.flags = val as u8,
                Some(other) => r.extra.push((other.to_string(), val as u32)),
                None => {}
            }
            p += 1 + sz;
        }
        self.running.store(false, Ordering::SeqCst);        // jeder Befehl hält VICE an
        Ok(r)
    }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(len);
        let mut a = addr as usize;
        while out.len() < len && a <= 0xFFFF {
            let end = (a + len - out.len() - 1).min(0xFFFF);
            let mut body = vec![0u8];
            body.extend((a as u16).to_le_bytes()); body.extend((end as u16).to_le_bytes()); body.push(0); body.extend([0, 0]);
            let b = self.cmd(0x01, &body, Duration::from_secs(5))?;
            let n = u16::from_le_bytes([b[0], b[1]]) as usize;
            out.extend_from_slice(&b[2..2 + n]);
            a = end + 1;
        }
        Ok(out)
    }
    fn read_space(&mut self, space: &str, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        if self.banks.is_none() { self.load_banks()?; }
        let id = self.banks.as_ref().and_then(|m| m.get(space).copied());
        match id { Some(b) => self.read_bank(b, addr, len), None => self.read_mem(addr, len) }
    }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        if data.is_empty() { return Ok(()); }
        let mut body = vec![0u8];
        body.extend(addr.to_le_bytes()); body.extend((addr + data.len() as u16 - 1).to_le_bytes()); body.push(0); body.extend([0, 0]);
        body.extend_from_slice(data);
        self.cmd(0x02, &body, Duration::from_secs(5)).map(|_| ())
    }
    fn step_blocked(&self) -> Option<String> { step_block_reason(cfg!(windows), self.version) }
    fn step(&mut self) -> Result<(), String> {
        if let Some(why) = self.step_blocked() { return Err(why); }
        self.cmd(0x71, &[0, 1, 0], Duration::from_secs(5)).map(|_| ())
    }
    fn cont(&mut self) -> Result<(), String> { self.running.store(true, Ordering::SeqCst); self.cmd(0xAA, &[], Duration::from_secs(5)).map(|_| ()) }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> {
        self.cmd(0x81, &[], Duration::from_secs(5))?;                        // jeder Befehl hält VICE an
        if self.wait_stopped(1500) { Ok(()) } else { self.running.store(false, Ordering::SeqCst); Ok(()) }
    }
    fn set_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        if self.bps.iter().any(|(a, _)| *a == addr) { return Ok(()); }
        let mut body = vec![];
        body.extend(addr.to_le_bytes()); body.extend(addr.to_le_bytes()); body.extend([1, 1, 4, 0]);     // stoppen, aktiv, exec, nicht temporär
        let b = self.cmd(0x12, &body, Duration::from_secs(5))?;
        let n = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        self.bps.push((addr, n));
        Ok(())
    }
    fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        if let Some(i) = self.bps.iter().position(|(a, _)| *a == addr) {
            let n = self.bps[i].1;
            self.cmd(0x13, &n.to_le_bytes(), Duration::from_secs(5))?;
            self.bps.remove(i);
        }
        Ok(())
    }
    fn breakpoints(&self) -> Vec<u16> { self.bps.iter().map(|(a, _)| *a).collect() }
    fn caps(&self) -> Caps { Caps { registers: true, step: true, breakpoints: true, watch: true } }
    fn trap_caps(&self) -> TrapCaps { TrapCaps { set_regs: true, cycles: self.system == "x64sc" } }
    fn set_regs(&mut self, r: &Regs) -> Result<(), String> {
        let ids: HashMap<&str, u8> = self.reg_names.iter().map(|(id, (n, _))| (n.as_str(), *id)).collect();
        let items = [("PC", r.pc), ("A", r.a as u16), ("X", r.x as u16), ("Y", r.y as u16), ("SP", r.sp as u16), ("FL", r.flags as u16)];
        let mut body = vec![0u8]; body.extend((items.len() as u16).to_le_bytes());
        for (n, v) in items { let id = *ids.get(n).ok_or_else(|| format!("VICE kennt das Register {n} nicht"))?; body.extend([3, id]); body.extend(v.to_le_bytes()); }
        self.cmd(0x32, &body, Duration::from_secs(5)).map(|_| ())
    }
    /// Zeile und Zyklus aus `LIN`/`CYC`; Maße nach Videonorm (PAL 63×312; andere Videonormen sind ungeprüft)
    fn cycles(&mut self) -> Result<Cycles, String> {
        if self.system != "x64sc" { return Err(format!("Zyklenmessung ist nur für x64sc geprüft, nicht für {}", self.system)); }
        let (per_line, lines) = match self.video {
            Some(v) => v,
            None => {
                // VICE 3.10: 1 = PAL (gemessen). NTSC (2), altes NTSC (3) und PAL-N (4) sind nicht gemessen, deshalb keine Zahlen raten.
                let v = match self.resource_int("MachineVideoStandard")? { 1 => (63, 312), x => return Err(format!("Zyklenmessung nur für PAL gemessen (Videonorm {x})")) };
                self.video = Some(v); v
            }
        };
        let r = self.registers()?;
        let get = |n: &str| r.extra.iter().find(|(k, _)| k == n).map(|(_, v)| *v);
        match (get("LIN"), get("CYC")) { (Some(line), Some(cycle)) => Ok(Cycles { line, cycle, per_line, lines }), _ => Err("VICE meldet LIN/CYC nicht".into()) }
    }
    fn set_watch(&mut self, start: u16, end: u16, kind: WatchKind) -> Result<u32, String> {
        let op = match kind { WatchKind::Read => 1u8, WatchKind::Write => 2, WatchKind::Access => 3 };      // 1 = Laden, 2 = Speichern
        let mut body = vec![]; body.extend(start.to_le_bytes()); body.extend(end.to_le_bytes()); body.extend([1, 1, op, 0]);
        let b = self.cmd(0x12, &body, Duration::from_secs(5))?;
        let n = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        self.watches.push((n, start, end, kind));
        Ok(n)
    }
    fn clear_watch(&mut self, id: u32) -> Result<(), String> {
        if let Some(i) = self.watches.iter().position(|w| w.0 == id) { self.cmd(0x13, &id.to_le_bytes(), Duration::from_secs(5))?; self.watches.remove(i); }
        Ok(())
    }
    fn last_hit(&mut self) -> Option<Hit> {
        let n = self.last_cp.load(Ordering::SeqCst); if n == 0 { return None; }
        self.bps.iter().find(|(_, c)| *c == n).map(|(a, _)| Hit::Exec(*a)).or_else(|| self.watches.iter().find(|w| w.0 == n).map(|w| Hit::Watch(w.0)))
    }
    fn can_grab_frame(&self) -> bool { true }
    fn frame_sources(&self) -> Vec<&'static str> { if self.system == "x128" { vec!["VIC-II (40 Spalten)", "VDC (80 Spalten)"] } else { vec![] } }
    fn set_frame_source(&mut self, index: usize) { self.vic_source = index == 0; }
    fn grab_frame(&mut self) -> Result<Frame, String> {
        // Jeder Monitorbefehl hält VICE an: lief die Maschine, wird sie danach fortgesetzt (auch bei einem Fehler)
        let was_running = self.running.load(Ordering::SeqCst);
        let r = self.display_rgba();
        if was_running { self.cont()?; }
        r
    }
    fn can_type_text(&self) -> bool { true }
    fn type_text(&mut self, petscii: &[u8]) -> Result<(), String> {
        if petscii.is_empty() { return Ok(()); }
        let n = petscii.len().min(255);
        let mut body = vec![n as u8]; body.extend_from_slice(&petscii[..n]);
        let was_running = self.running.load(Ordering::SeqCst);
        let r = self.cmd(0x72, &body, Duration::from_secs(5)).map(|_| ());
        if was_running { self.cont()?; }
        r
    }
    fn stop(&mut self) { terminate(&mut self.child, &self.marker); }
}

/// Palette (Antwort auf 0x91): Anzahl (u16), je Eintrag Länge (3) und R, G, B
fn parse_palette(b: &[u8]) -> Result<Vec<[u8; 3]>, String> {
    if b.len() < 2 { return Err("VICE-Palette: Antwort zu kurz".into()); }
    let n = u16::from_le_bytes([b[0], b[1]]) as usize;
    let mut out = Vec::with_capacity(n); let mut p = 2;
    for _ in 0..n {
        let sz = *b.get(p).ok_or("VICE-Palette: abgeschnitten")? as usize;
        if sz < 3 || p + 1 + sz > b.len() { return Err("VICE-Palette: Eintrag unvollständig".into()); }
        out.push([b[p + 1], b[p + 2], b[p + 3]]); p += 1 + sz;
    }
    Ok(out)
}

/// Bild (Antwort auf 0x84, Format 0 = 8 Bit je Punkt, Farbindex) mit Palette zu RGBA. Kopf: Länge des festen Teils (u32), Breite, Höhe,
/// X-/Y-Versatz und Breite/Höhe des Innenbereichs (je u16), Bits je Punkt (u8), Länge des Puffers (u32), dann die Punkte.
/// VICE 3.10 liefert den Puffer um 4 Bytes kürzer als im Kopf angegeben: der Rest (die letzten Punkte der letzten Zeile) bleibt Index 0.
fn parse_display(b: &[u8], pal: &[[u8; 3]]) -> Result<Frame, String> {
    if b.len() < 21 { return Err("VICE-Bild: Antwort zu kurz".into()); }
    let u16at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]) as usize;
    let info = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
    let (w, h, bpp) = (u16at(4), u16at(6), b[16]);
    if info < 13 { return Err(format!("VICE-Bild: Kopflänge {info} unbekannt")); }
    if bpp != 8 { return Err(format!("VICE-Bild: {bpp} Bit je Punkt werden nicht unterstützt")); }
    if w == 0 || h == 0 || w > 4096 || h > 4096 { return Err(format!("VICE-Bild: Größe {w}×{h} unplausibel")); }
    if pal.is_empty() { return Err("VICE-Palette ist leer".into()); }
    let start = 4 + info + 4;
    let data = b.get(start..).ok_or("VICE-Bild: Puffer fehlt")?;
    if data.len() + 16 < w * h { return Err(format!("VICE-Bild: Puffer zu kurz ({} von {} Bytes)", data.len(), w * h)); }
    let mut rgba = Vec::with_capacity(w * h * 4);
    for i in 0..w * h {
        let c = pal[data.get(i).copied().unwrap_or(0) as usize % pal.len()];
        rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
    }
    Ok(Frame { width: w, height: h, rgba, frame_no: 0 })
}

/// Beendet den gestarteten Prozess und alle Prozesse, die seine Kommandozeile mit `marker` tragen. Unter macOS (App-Installation) ist
/// `bin/x64sc` ein Shell-Skript, das VICE als Kindprozess startet: `Child::kill()` träfe nur das Skript und VICE liefe weiter
/// (belegt den Monitor-Port, zweiter Emulator neben dem nächsten Start).
pub(crate) fn terminate(child: &mut Child, marker: &str) {
    let _ = child.kill();
    let _ = child.wait();
    #[cfg(unix)]
    if !marker.is_empty() { let _ = Command::new("pkill").args(["-KILL", "-f", marker]).stdout(Stdio::null()).stderr(Stdio::null()).status(); }
    #[cfg(not(unix))]
    let _ = marker;
}

impl Drop for Vice { fn drop(&mut self) { self.stop(); } }

impl Vice { pub fn last_stop_pc(&self) -> u16 { self.stop_pc.load(Ordering::SeqCst) as u16 } }

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn alive(marker: &str) -> bool { Command::new("pgrep").args(["-f", marker]).stdout(Stdio::null()).status().map_or(false, |s| s.success()) }

    /// Wie `bin/x64sc` unter macOS: ein Skript startet das eigentliche Programm als Kindprozess; `kill()` allein ließe es laufen.
    #[test]
    fn terminate_also_ends_children_of_a_launcher_script() {
        let marker = format!("vc-terminate-probe-{}", std::process::id());
        let mut child = Command::new("bash").args(["-c", "(exec -a \"$0-kind\" sleep 60) & wait", &marker]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while !alive(&marker) && Instant::now() < end { std::thread::sleep(Duration::from_millis(50)); }
        assert!(alive(&marker), "Probeprozess läuft nicht");
        terminate(&mut child, &marker);
        std::thread::sleep(Duration::from_millis(200));
        assert!(!alive(&marker), "Kindprozess des Startskripts läuft noch");
    }
}

/// VICE 3.9 für Windows stürzt beim Einzelschritt über den Monitor ab (Zugriffsverletzung): der Schritt wird vorher abgefangen
fn step_block_reason(windows: bool, version: Option<(u8, u8)>) -> Option<String> {
    let (a, b) = version?;
    if windows && (a, b) < (3, 10) { Some(format!("VICE {a}.{b} stürzt unter Windows beim Einzelschritt ab – bitte VICE 3.10 oder neuer verwenden")) } else { None }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    fn display(w: u16, h: u16, px: &[u8]) -> Vec<u8> {
        let mut b = 13u32.to_le_bytes().to_vec();
        for v in [w, h, 0, 0, w, h] { b.extend(v.to_le_bytes()); }
        b.push(8); b.extend((w as u32 * h as u32).to_le_bytes()); b.extend_from_slice(px); b
    }
    #[test]
    fn palette_is_read() {
        let b = [2, 0, 3, 1, 2, 3, 3, 255, 0, 128];
        assert_eq!(parse_palette(&b).unwrap(), vec![[1, 2, 3], [255, 0, 128]]);
        assert!(parse_palette(&[2, 0, 3, 1, 2, 3]).is_err());
        assert!(parse_palette(&[]).is_err());
    }
    #[test]
    fn display_becomes_rgba() {
        let f = parse_display(&display(2, 2, &[0, 1, 1, 0]), &[[0, 0, 0], [255, 255, 255]]).unwrap();
        assert_eq!((f.width, f.height), (2, 2));
        assert_eq!(f.rgba, vec![0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255]);
    }
    #[test]
    fn short_buffer_from_vice_is_tolerated_but_not_garbage() {
        let pal = [[9, 9, 9], [1, 1, 1]];
        let full: Vec<u8> = vec![1; 100];
        assert_eq!(parse_display(&display(10, 10, &full[..96]), &pal).unwrap().rgba.len(), 400);   // 4 Bytes fehlen wie bei VICE 3.10
        assert!(parse_display(&display(10, 10, &full[..40]), &pal).is_err());
    }
    #[test]
    fn broken_headers_are_rejected() {
        let pal = [[0, 0, 0]];
        assert!(parse_display(&[0; 5], &pal).is_err());
        let mut b = display(2, 2, &[0; 4]); b[16] = 4; assert!(parse_display(&b, &pal).unwrap_err().contains("4 Bit"));
        let mut b = display(2, 2, &[0; 4]); b[4] = 0; b[5] = 0; assert!(parse_display(&b, &pal).is_err());
        assert!(parse_display(&display(2, 2, &[0; 4]), &[]).is_err());
    }
    #[test]
    fn petscii_conversion() {
        use crate::ascii_to_petscii as p;
        assert_eq!(p("print\"vc\"\n"), b"PRINT\"VC\"\r");
        assert_eq!(p("Ab€1"), b"AB1");
        assert_eq!(p(&"x".repeat(400)).len(), 255);
    }
}

#[cfg(test)]
mod step_block_tests {
    use super::step_block_reason;
    #[test]
    fn old_vice_on_windows_cannot_step() {
        assert!(step_block_reason(true, Some((3, 9))).unwrap().contains("3.9"));
        assert!(step_block_reason(true, Some((3, 10))).is_none());
        assert!(step_block_reason(false, Some((3, 9))).is_none());
        assert!(step_block_reason(true, None).is_none());
    }
}
