// SPDX-License-Identifier: GPL-3.0-or-later
//! Fujisan-Adapter (Atari 8 Bit): Fujisan ist ein Qt-Emulator auf Basis von libatari800 (https://github.com/pedgarcia/fujisan, MIT)
//! mit einem TCP-Server (JSON-Zeilen, Standardport 6502, läuft nach dem Start von selbst). Der Adapter startet Fujisan als eigenen Prozess
//! (oder hängt sich an eine laufende Instanz) und steuert es über diesen Server.
//!
//! Gemessene Eigenschaften (Fujisan 2.0.5, Windows; siehe das Konzept „Fujisan“):
//! - Haltepunkte (mindestens 80), Einzelschritt, Schritt über, Pause und Fortsetzen per Befehl.
//! - **Kein Ereignis beim Treffer eines Haltepunkts**: der Adapter fragt `status.get_state` ab (`running`).
//! - Speicher lesen: höchstens 256 Byte je Aufruf; schreiben: **ein Byte je Aufruf** (andere Felder werden stillschweigend als 0 geschrieben).
//! - Register lesen ja, **setzen nein**; keine Zyklenzählung, keine Datenhaltepunkte.
//! - `debug.load_xex_for_debug` lädt ein `.xex` und hält am Einsprung an.
//!
//! Ergänzungen im Adapter:
//! - **Register setzen** (für die Tests im Trap-Modus): Fujisan hat keinen Befehl dafür. `set_regs` schreibt einen kurzen Vorspann
//!   (`LDX #sp / TXS / LDA #flags / PHA / LDA #a / LDX #x / LDY #y / PLP / JMP pc`) an den aktuellen PC, führt ihn mit neun Einzelschritten
//!   aus und stellt die überschriebenen Bytes wieder her. Voraussetzung: der PC steht im RAM (nach `launch_bare` und nach jeder Falle der Fall).
//!   Greift zufällig ein NMI (Bildwechsel) in diese neun Befehle, bleibt das Ergebnis richtig (der Stapel wird vom NMI nur darunter benutzt).
//! - **Bild**: `screen.capture` schreibt eine PCX-Datei (8 Bit, RLE, Palette am Ende); der Adapter liest sie, wandelt in RGBA und löscht sie.
use crate::{find_program, Backend, Caps, Hit, Regs, RunState, TrapCaps};
use crate::streams::video::Frame;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Standardport des TCP-Servers von Fujisan
pub const DEFAULT_PORT: u16 = 6502;

/// Größte Länge, die `debug.read_memory` je Aufruf liefert
const READ_CHUNK: usize = 256;

pub struct Fujisan { child: Option<Child>, rd: BufReader<TcpStream>, wr: TcpStream, next_id: u64, bps: Vec<u16>, running: bool }

/// `"$A9"` oder `"A9"` → Zahl
fn hex(v: &Value) -> Option<u32> {
    let s = v.as_str()?;
    u32::from_str_radix(s.trim().trim_start_matches('$'), 16).ok()
}

impl Fujisan {
    /// Startet Fujisan (oder nutzt eine laufende Instanz auf dem Port) und lädt `xex` mit Halt am Einsprung.
    /// `program`: Pfad zu `Fujisan(.exe)`; leer: automatisch suchen.
    pub fn launch(xex: &str, program: Option<&str>, port: u16) -> Result<Fujisan, String> {
        let mut f = Self::open(program, port)?;
        f.load(xex)?;
        Ok(f)
    }

    /// Startet Fujisan für die Tests im Trap-Modus: lädt ein winziges Programm (`JMP *` bei `$0600`), damit der PC im RAM steht und angehalten ist.
    /// Das Betriebssystem ist hochgefahren (anders als bei atari800); Programm und Falle schreibt der Läufer selbst in den Speicher.
    pub fn launch_bare(program: Option<&str>, port: u16) -> Result<Fujisan, String> {
        // Ein vorheriger Testlauf wird gerade beendet (der Läufer startet den Emulator für einen neuen Fall, bevor der alte verschwunden ist):
        // zuerst abwarten, damit wir uns nicht an die sterbende Instanz hängen. Bleibt sie bestehen, ist es die des Anwenders.
        let end = Instant::now() + Duration::from_secs(5);
        while Instant::now() < end && Self::connect(port, None).is_ok() { std::thread::sleep(Duration::from_millis(100)); }
        let dir = std::env::temp_dir().join(format!("vc-fujisan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let x = dir.join("idle.xex");
        std::fs::write(&x, IDLE_XEX).map_err(|e| e.to_string())?;
        let r = Self::launch(&x.to_string_lossy(), program, port);
        let _ = std::fs::remove_dir_all(&dir);
        r
    }

    /// Verbindet mit Fujisan, ohne ein Programm zu laden (wie `launch`, sonst gleich).
    pub fn open(program: Option<&str>, port: u16) -> Result<Fujisan, String> {
        if let Ok(f) = Self::connect(port, None) { return Ok(f); }
        let exe = program.filter(|p| !p.is_empty()).map(String::from).or_else(find_exe).ok_or_else(|| format!("Fujisan nicht gefunden ({})", install_hint()))?;
        let dir = std::path::Path::new(&exe).parent().map(|p| p.to_path_buf());
        let mut cmd = Command::new(&exe);
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        if let Some(d) = dir.filter(|d| !d.as_os_str().is_empty()) { cmd.current_dir(d); }
        let mut child = cmd.spawn().map_err(|e| format!("{exe}: {e}"))?;
        let end = Instant::now() + Duration::from_secs(20);
        loop {
            match Self::connect(port, None) {
                Ok(mut f) => { f.child = Some(child); return Ok(f); }
                Err(e) => {
                    if let Ok(Some(st)) = child.try_wait() { return Err(format!("Fujisan wurde beendet ({st}), bevor der TCP-Server antwortete")); }
                    if Instant::now() > end { let _ = child.kill(); return Err(format!("Fujisan antwortet auf Port {port} nicht: {e}")); }
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
    }

    /// Verbindung zu einem laufenden TCP-Server; prüft die Begrüßung (`event: connected`, `emulator: Fujisan`).
    pub fn connect(port: u16, child: Option<Child>) -> Result<Fujisan, String> {
        let addr = ("127.0.0.1", port).to_socket_addrs().map_err(|e| e.to_string())?.next().ok_or("keine Adresse")?;
        let s = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).map_err(|e| e.to_string())?;
        Self::from_stream(s, child)
    }

    fn from_stream(s: TcpStream, child: Option<Child>) -> Result<Fujisan, String> {
        s.set_read_timeout(Some(Duration::from_secs(8))).map_err(|e| e.to_string())?;
        s.set_nodelay(true).ok();
        let wr = s.try_clone().map_err(|e| e.to_string())?;
        let mut f = Fujisan { child, rd: BufReader::new(s), wr, next_id: 0, bps: vec![], running: false };
        let hello = f.read_line()?;
        if hello["event"] != "connected" || hello["data"]["emulator"] != "Fujisan" { return Err(format!("auf dem Port antwortet kein Fujisan: {hello}")); }
        Ok(f)
    }

    fn read_line(&mut self) -> Result<Value, String> {
        let mut line = String::new();
        loop {
            line.clear();
            match self.rd.read_line(&mut line) {
                Ok(0) => return Err("Fujisan hat die Verbindung geschlossen".into()),
                Ok(_) => { if line.trim().is_empty() { continue; } return serde_json::from_str(line.trim()).map_err(|e| format!("Antwort von Fujisan nicht lesbar: {e}")); }
                Err(e) => return Err(format!("Fujisan antwortet nicht: {e}")),
            }
        }
    }

    /// Sendet einen Befehl und wartet auf die Antwort mit derselben Kennung (Ereignisse dazwischen werden übergangen).
    fn cmd(&mut self, name: &str, params: Option<Value>) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id.to_string();
        let mut m = json!({ "command": name, "id": id });
        if let Some(p) = params { m["params"] = p; }
        if std::env::var_os("VC_FUJISAN_TRACE").is_some() { eprintln!("fujisan> {m}"); }
        writeln!(self.wr, "{m}").and_then(|_| self.wr.flush()).map_err(|e| format!("Fujisan: {e}"))?;
        loop {
            let r = self.read_line()?;
            if r["type"] != "response" || r["id"].as_str() != Some(id.as_str()) { continue; }
            if r["status"] == "success" { return Ok(r["result"].clone()); }
            return Err(format!("{name}: {}", r["error"].as_str().unwrap_or("Fehler")));
        }
    }

    /// Lädt ein `.xex` und hält am Einsprung an. Vorhandene Haltepunkte der Sitzung werden erneut gesetzt.
    pub fn load(&mut self, xex: &str) -> Result<(), String> {
        if !xex.to_ascii_lowercase().ends_with(".xex") { return Err("Der Fujisan-Debugger lädt nur .xex-Dateien (Diskettenabbilder und Module bitte mit atari800 debuggen)".into()); }
        let abs = std::fs::canonicalize(xex).map(|p| p.to_string_lossy().trim_start_matches(r"\\?\").to_string()).unwrap_or_else(|_| xex.to_string());
        self.cmd("debug.load_xex_for_debug", Some(json!({ "path": abs })))?;
        self.running = false;
        let bps = std::mem::take(&mut self.bps);
        for a in bps { self.set_breakpoint(a)?; }
        Ok(())
    }

    fn get_running(&mut self) -> Result<bool, String> { Ok(self.cmd("status.get_state", None)?["running"].as_bool().unwrap_or(false)) }
}

/// Leeres Programm für `launch_bare`: ein Segment `$0600` mit `JMP $0600` und `RUNAD` = `$0600`
const IDLE_XEX: [u8; 15] = [0xFF, 0xFF, 0x00, 0x06, 0x02, 0x06, 0x4C, 0x00, 0x06, 0xE0, 0x02, 0xE1, 0x02, 0x00, 0x06];

/// Wandelt eine PCX-Datei (8 Bit, eine Ebene, RLE, Palette mit Kennbyte `0x0C` am Ende) in RGBA
pub fn decode_pcx(d: &[u8]) -> Result<Frame, String> {
    if d.len() < 128 + 769 || d[0] != 0x0A || d[2] != 1 || d[3] != 8 || d[65] != 1 { return Err("PCX-Datei nicht lesbar (erwartet: 8 Bit, eine Ebene, RLE)".into()); }
    let u16at = |i: usize| u16::from_le_bytes([d[i], d[i + 1]]) as usize;
    let (w, h, bpl) = (u16at(8) + 1 - u16at(4), u16at(10) + 1 - u16at(6), u16at(66));
    let pal = &d[d.len() - 768..];
    if d[d.len() - 769] != 0x0C || bpl < w { return Err("PCX-Datei: Palette fehlt".into()); }
    let mut idx = Vec::with_capacity(bpl * h);
    let mut i = 128;
    while idx.len() < bpl * h && i < d.len() - 769 {
        let b = d[i]; i += 1;
        if b >= 0xC0 { let n = (b & 0x3F) as usize; let v = *d.get(i).ok_or("PCX-Datei abgeschnitten")?; i += 1; idx.extend(std::iter::repeat(v).take(n)); } else { idx.push(b); }
    }
    if idx.len() < bpl * h { return Err("PCX-Datei abgeschnitten".into()); }
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h { for x in 0..w { let c = idx[y * bpl + x] as usize * 3; rgba.extend_from_slice(&[pal[c], pal[c + 1], pal[c + 2], 255]); } }
    Ok(Frame { width: w, height: h, rgba, frame_no: 0 })
}

/// Fujisan in den üblichen Ordnern suchen: eigene Installation von VintageCoder, `PATH`, Programme-Ordner
pub fn find_exe() -> Option<String> {
    find_program("Fujisan", &["/opt/fujisan", "/opt/fujisan/bin", "/Applications/Fujisan.app/Contents/MacOS"]).or_else(|| find_program("fujisan", &[]))
}

/// Von VintageCoder geladene Fujisan-Version (neueste), falls vorhanden
pub fn installed_exe() -> Option<String> { crate::install::find_exe("fujisan", "Fujisan").map(|p| p.to_string_lossy().into_owned()) }

pub fn install_hint() -> String {
    "Fujisan von https://github.com/pedgarcia/fujisan/releases installieren oder mit „vcc emulator install fujisan“ laden".into()
}

impl Backend for Fujisan {
    fn name(&self) -> &str { "Fujisan" }
    fn caps(&self) -> Caps { Caps { registers: true, step: true, breakpoints: true, watch: false } }
    /// Alle Register und Flags (über den Vorspann), keine Zyklenzählung
    fn trap_caps(&self) -> TrapCaps { TrapCaps { set_regs: true, cycles: false } }
    fn set_regs(&mut self, r: &Regs) -> Result<(), String> {
        if self.running { return Err("Emulator läuft – zuerst anhalten".into()); }
        let pc = self.registers()?.pc;
        if pc > 0xFFF0 { return Err("PC liegt zu nah am Speicherende für den Vorspann".into()); }
        let stub = [0xA2, r.sp, 0x9A, 0xA9, r.flags, 0x48, 0xA9, r.a, 0xA2, r.x, 0xA0, r.y, 0x28, 0x4C, r.pc as u8, (r.pc >> 8) as u8];
        let saved = self.read_mem(pc, stub.len())?;
        self.write_mem(pc, &stub)?;
        let run = (|| { for _ in 0..9 { self.step()?; } Ok::<(), String>(()) })();
        self.write_mem(pc, &saved)?;
        run?;
        let now = self.registers()?;
        if now.pc != r.pc { return Err(format!("Register setzen fehlgeschlagen: PC ist ${:04X} statt ${:04X} (Vorspann im ROM?)", now.pc, r.pc)); }
        Ok(())
    }
    fn can_grab_frame(&self) -> bool { true }
    fn grab_frame(&mut self) -> Result<Frame, String> {
        let res = self.cmd("screen.capture", None)?;
        let file = res["filename"].as_str().ok_or("screen.capture nennt keine Datei")?.to_string();
        let data = std::fs::read(&file).map_err(|e| format!("{file}: {e}"));
        let _ = std::fs::remove_file(&file);
        decode_pcx(&data?)
    }
    fn state(&mut self) -> RunState {
        match self.get_running() { Ok(true) => { self.running = true; RunState::Running } Ok(false) => { self.running = false; RunState::Stopped } Err(_) => { self.running = false; RunState::Stopped } }
    }
    fn registers(&mut self) -> Result<Regs, String> {
        let r = self.cmd("debug.get_registers", None)?;
        let g = |k: &str| hex(&r[k]).ok_or_else(|| format!("Register {k} nicht lesbar"));
        Ok(Regs { pc: g("pc")? as u16, a: g("a")? as u8, x: g("x")? as u8, y: g("y")? as u8, sp: g("s")? as u8, flags: g("p")? as u8, extra: vec![] })
    }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let (mut out, mut a) = (Vec::with_capacity(len), addr as usize);
        let end = (addr as usize + len).min(0x10000);
        while a < end {
            let n = (end - a).min(READ_CHUNK);
            let r = self.cmd("debug.read_memory", Some(json!({ "address": a, "length": n })))?;
            let bytes: Vec<u8> = r["data"].as_array().ok_or("Speicher nicht lesbar")?.iter().filter_map(|v| hex(v).map(|x| x as u8)).collect();
            if bytes.len() != n { return Err(format!("Fujisan lieferte {} statt {n} Byte ab ${a:04X}", bytes.len())); }
            out.extend(bytes);
            a += n;
        }
        Ok(out)
    }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        // Fujisan schreibt je Aufruf genau ein Byte (Felder `data`/`values` werden ignoriert und als 0 geschrieben) und bearbeitet etwa 50 Befehle
        // je Sekunde, auch gebündelt. Lesen geht in 256-Byte-Stücken (etwa 7 KB/s): bei größeren Blöcken, die der Läufer vor jedem Test neu
        // aufspielt, wird deshalb erst gelesen und nur geschrieben, was sich unterscheidet.
        if addr as usize + data.len() > 0x10000 { return Err("Schreiben über $FFFF hinaus".into()); }
        let current = if data.len() > 16 { Some(self.read_mem(addr, data.len())?) } else { None };
        for (i, b) in data.iter().enumerate() {
            if current.as_ref().map_or(false, |c| c[i] == *b) { continue; }
            self.cmd("debug.write_memory", Some(json!({ "address": addr as usize + i, "value": *b })))?;
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> { self.cmd("debug.step_into", None).map(|_| ()) }
    fn cont(&mut self) -> Result<(), String> {
        if self.running { return Ok(()); }
        self.cmd("debug.resume", None)?;
        self.running = true;
        Ok(())
    }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> {
        match self.cmd("debug.pause", None) {
            Ok(_) => { self.running = false; Ok(()) }
            Err(e) if e.contains("already paused") => { self.running = false; Ok(()) }
            Err(e) => Err(e),
        }
    }
    fn set_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        self.cmd("debug.add_breakpoint", Some(json!({ "address": addr })))?;
        if !self.bps.contains(&addr) { self.bps.push(addr); }
        Ok(())
    }
    fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String> {
        self.cmd("debug.remove_breakpoint", Some(json!({ "address": addr })))?;
        self.bps.retain(|a| *a != addr);
        Ok(())
    }
    fn breakpoints(&self) -> Vec<u16> { self.bps.clone() }
    fn last_hit(&mut self) -> Option<Hit> { let pc = self.registers().ok()?.pc; self.bps.contains(&pc).then_some(Hit::Exec(pc)) }
    fn stop(&mut self) {
        if let Some(mut c) = self.child.take() { let _ = c.kill(); let _ = c.wait(); }
    }
}

impl Drop for Fujisan { fn drop(&mut self) { self.stop(); } }

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// Kleiner Ersatz für den TCP-Server von Fujisan (nur die Befehle, die der Adapter nutzt)
    fn fake_server() -> u16 {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (s, _) = l.accept().unwrap();
            let mut w = s.try_clone().unwrap();
            writeln!(w, r#"{{"data":{{"emulator":"Fujisan","version":"1.0"}},"event":"connected","type":"event"}}"#).unwrap();
            let mut mem = vec![0u8; 0x10000];
            let (mut running, mut pc) = (false, 0x2000u16);
            for line in BufReader::new(s).lines() {
                let Ok(line) = line else { break };
                let m: Value = serde_json::from_str(&line).unwrap();
                let (id, c, p) = (m["id"].as_str().unwrap().to_string(), m["command"].as_str().unwrap().to_string(), m["params"].clone());
                let ok = |r: Value| json!({ "id": id, "result": r, "status": "success", "type": "response" });
                let reply = match c.as_str() {
                    "debug.load_xex_for_debug" => { pc = 0x2000; ok(json!({ "entry_point": "$2000", "loaded": true, "paused_after_load": true })) }
                    "status.get_state" => ok(json!({ "running": running, "pc": format!("${pc:04X}") })),
                    "debug.get_registers" => ok(json!({ "pc": format!("${pc:04X}"), "a": "$07", "x": "$0C", "y": "$FF", "s": "$FB", "p": "$30" })),
                    "debug.read_memory" => {
                        let (a, n) = (p["address"].as_u64().unwrap() as usize, p["length"].as_u64().unwrap() as usize);
                        if n > 256 || a + n > 0x10000 { json!({ "id": id, "error": "Invalid memory length or address range", "status": "error", "type": "response" }) }
                        else { ok(json!({ "address": format!("${a:04X}"), "data": mem[a..a + n].iter().map(|b| format!("${b:02X}")).collect::<Vec<_>>(), "length": n })) }
                    }
                    "debug.write_memory" => { mem[p["address"].as_u64().unwrap() as usize] = p["value"].as_u64().unwrap() as u8; ok(json!({ "written": true })) }
                    "debug.add_breakpoint" | "debug.remove_breakpoint" => ok(json!({})),
                    "debug.step_into" => { pc += 2; ok(json!({ "stepped": true, "pc": format!("${pc:04X}") })) }
                    "debug.resume" => { running = true; ok(json!({ "resumed": true })) }
                    "debug.pause" => if running { running = false; ok(json!({ "paused": true })) } else { json!({ "id": id, "error": "Emulation is already paused", "status": "error", "type": "response" }) },
                    _ => json!({ "id": id, "error": format!("Unknown command: {c}"), "status": "error", "type": "response" }),
                };
                // Ereignis vor der Antwort: der Adapter muss es übergehen
                if c == "debug.resume" { writeln!(w, r#"{{"data":{{"resumed":true}},"event":"debug_resumed","type":"event"}}"#).unwrap(); }
                writeln!(w, "{reply}").unwrap();
            }
        });
        port
    }

    #[test]
    fn protokoll_gegen_den_ersatzserver() {
        let mut f = Fujisan::connect(fake_server(), None).unwrap();
        let dir = std::env::temp_dir().join("vc-fujisan-test");
        std::fs::create_dir_all(&dir).unwrap();
        let x = dir.join("t.xex");
        std::fs::write(&x, [0xFF, 0xFF]).unwrap();
        f.load(x.to_str().unwrap()).unwrap();
        assert_eq!(f.state(), RunState::Stopped);
        let r = f.registers().unwrap();
        assert_eq!((r.pc, r.a, r.x, r.y, r.sp, r.flags), (0x2000, 7, 0x0C, 0xFF, 0xFB, 0x30));
        // Schreiben je Byte, Lesen in Stücken zu 256 Byte (hier über die Grenze von zwei Stücken)
        let data: Vec<u8> = (0..600u32).map(|i| (i % 251) as u8).collect();
        f.write_mem(0x3000, &data).unwrap();
        assert_eq!(f.read_mem(0x3000, 600).unwrap(), data);
        assert_eq!(f.read_mem(0xFFF0, 100).unwrap().len(), 16, "über $FFFF hinaus wird abgeschnitten");
        f.set_breakpoint(0x2004).unwrap();
        assert_eq!(f.breakpoints(), [0x2004]);
        f.step().unwrap();
        assert_eq!(f.registers().unwrap().pc, 0x2002);
        f.cont().unwrap();
        assert_eq!(f.state(), RunState::Running);
        f.pause().unwrap();
        f.pause().unwrap();                                    // „already paused“ ist kein Fehler
        assert_eq!(f.state(), RunState::Stopped);
        f.clear_breakpoint(0x2004).unwrap();
        assert!(f.breakpoints().is_empty());
    }

    #[test]
    fn nur_xex_und_nur_fujisan() {
        let mut f = Fujisan::connect(fake_server(), None).unwrap();
        assert!(f.load("spiel.atr").unwrap_err().contains(".xex"));
        // ein Server, der sich nicht als Fujisan meldet
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || { let (mut s, _) = l.accept().unwrap(); writeln!(s, "{{\"event\":\"hallo\"}}").unwrap(); std::thread::sleep(Duration::from_millis(200)); });
        assert!(Fujisan::connect(port, None).err().unwrap().contains("kein Fujisan"));
    }

    #[test]
    fn pcx_wird_zu_rgba() {
        // 3 × 2 Punkte: Zeile 1 = 0,1,1 (Lauf der Länge 2 für Index 1), Zeile 2 = 2,2,2 (Lauf 3); Zeilenbreite 4 (gerade)
        let mut d = vec![0x0A, 5, 1, 8, 0, 0, 0, 0, 2, 0, 1, 0];
        d.resize(65, 0); d.extend([1, 4, 0]); d.resize(128, 0);
        d.extend([0x00, 0xC2, 0x01, 0x00, 0xC3, 0x02, 0x00]);
        d.push(0x0C);
        let mut pal = vec![0u8; 768]; pal[3..6].copy_from_slice(&[10, 20, 30]); pal[6..9].copy_from_slice(&[40, 50, 60]);
        d.extend(pal);
        let f = decode_pcx(&d).unwrap();
        assert_eq!((f.width, f.height), (3, 2));
        assert_eq!(&f.rgba[..12], [0, 0, 0, 255, 10, 20, 30, 255, 10, 20, 30, 255]);
        assert_eq!(&f.rgba[12..16], [40, 50, 60, 255]);
        assert!(decode_pcx(&d[..100]).is_err());
    }
}
