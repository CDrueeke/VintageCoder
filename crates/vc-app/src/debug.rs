// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Sitzung in einem Arbeits-Thread: startet den Emulator über `vc_ide::debug`, nimmt Befehle entgegen und veröffentlicht Zustandsabbilder.
use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use vc_emu::streams::video::Frame;
use vc_ide::debug::{default_launcher, LaunchSpec, Session};

pub enum Cmd { Chips(Vec<(u16, u16)>), Bp(u16, bool), Cont, Pause, Step(&'static str), Mem(u16), SaveCharset { addr: u16, profile: String, name: String, root: std::path::PathBuf, data: std::path::PathBuf }, AddPoint { kind: String, start: u16, end: u16, cond: Option<String>, ignore: u32 }, DelPoint(u32), Watches(Vec<String>), SnapDetect(String, std::path::PathBuf), Frame, FrameMode { auto: bool, slow: bool }, FrameSource(usize), Text(String), SnapSave { #[allow(dead_code)] system: String, det: crate::snapshot::Detected, opts: crate::snapshot::Opts, root: std::path::PathBuf, data: std::path::PathBuf }, Stop }

#[derive(Clone, Default)]
pub struct PointInfo { pub id: u32, pub kind: String, pub start: u16, pub end: u16, pub cond: Option<String>, pub ignore: u32, pub hits: u32 }

#[derive(Clone, Default)]
pub struct DisLine { pub addr: u16, pub bytes: Vec<u8>, pub text: String, pub illegal: bool,
    /// Taktzyklen laut Befehlstabelle (0 = unbekannt); `extra`: kommt bei Seitenwechsel bzw. genommenem Sprung hinzu
    pub cycles: u8, pub extra: bool }

/// Was der Adapter laut `Backend::caps()` kann
#[derive(Clone, Copy, Default)]
pub struct CapsInfo { pub registers: bool, pub step: bool, pub breakpoints: bool, pub watch: bool }

#[derive(Clone, Default)]
pub struct Snap {
    pub starting: bool, pub ended: bool, pub error: Option<String>, pub error_seq: u32, pub step_block: Option<String>, pub running: bool, pub backend: String,
    pub pc: u16, pub a: u8, pub x: u8, pub y: u8, pub sp: u8, pub flags: u8, pub extra: Vec<(String, i64)>, pub have_regs: bool,
    pub disasm: Vec<DisLine>, pub mem_addr: u16, pub mem: Vec<u8>, pub breakpoints: Vec<u16>, pub can_pause: bool, pub stopped_at: Option<u32>, pub cond_error: Option<String>,
    pub notes: Vec<(bool, String)>, pub points: Vec<PointInfo>, pub watch: Vec<(String, Result<i64, String>)>,
    pub detected: Option<Result<crate::snapshot::Detected, String>>,
    pub caps: CapsInfo,
    /// vorheriger Speicherabzug derselben Adresse (für die Änderungsmarkierung)
    pub mem_prev: Vec<u8>,
    /// gelesene Chip-Fenster: (Startadresse, Bytes); nur im angehaltenen Zustand
    pub chips: Vec<(u16, Vec<u8>)>,
    /// Bild des Emulators (nur Adapter mit `grab_frame`): neuestes Bild, laufende Nummer, letzter Fehler, Bildquellen (C128: VIC-II/VDC)
    pub frame: Option<Arc<Frame>>, pub frame_seq: u64, pub frame_err: Option<String>, pub can_frame: bool, pub can_text: bool, pub frame_sources: Vec<String>,
}

/// Was der Start eines Emulators liefert
struct Launched { backend: Box<dyn vc_emu::Backend>, cpu: vc_target::Cpu, target: String, symbols: HashMap<String, i64>, autorun: bool, halt_bp: Option<u16> }

/// Holt das Bild des Emulators und legt es im Zustandsabbild ab (Fehler als Text); der Adapter setzt eine laufende Maschine selbst fort
fn grab_and_publish(b: &mut dyn vc_emu::Backend, publish: &dyn Fn(&dyn Fn(&mut Snap))) {
    match b.grab_frame() {
        Ok(f) => { let f = Arc::new(f); publish(&|s| { s.frame = Some(f.clone()); s.frame_seq += 1; s.frame_err = None; }); }
        Err(e) => publish(&|s| s.frame_err = Some(e.clone())),
    }
}

pub struct Debugger { tx: Sender<Cmd>, pub snap: Arc<Mutex<Snap>> }

impl Debugger {
    pub fn start(spec: LaunchSpec, cpu: vc_target::Cpu, symbols: HashMap<String, i64>, bps: Vec<u16>, ctx: egui::Context) -> Debugger {
        let target = spec.target.clone();
        Self::start_with(Box::new(move || default_launcher()(&spec).map(|backend| Launched { backend, cpu, target, symbols, autorun: true, halt_bp: None })), bps, ctx)
    }

    /// „Diesen Test debuggen“: Emulator mit vorbereiteter Falle, angehalten am Anfang der Falle (kein Selbstlauf); Haltepunkt am `jmp *` der Falle
    pub fn start_test(file: std::path::PathBuf, data: std::path::PathBuf, test: String, ctx: egui::Context) -> Debugger {
        Self::start_with(Box::new(move || {
            let p = vc_ide::tests::suite::prepare_debug(&file, &data, &test)?;
            Ok(Launched { backend: p.backend, cpu: p.cpu, target: p.target, symbols: p.symbols, autorun: false, halt_bp: Some(p.info.halt) })
        }), vec![], ctx)
    }

    fn start_with(make: Box<dyn FnOnce() -> Result<Launched, String> + Send>, bps: Vec<u16>, ctx: egui::Context) -> Debugger {
        let (tx, rx) = channel::<Cmd>(); let snap = Arc::new(Mutex::new(Snap { starting: true, mem_addr: 0x0400, ..Default::default() })); let s2 = snap.clone();
        std::thread::spawn(move || {
            let publish = |f: &dyn Fn(&mut Snap)| { if let Ok(mut g) = s2.lock() { f(&mut g); } ctx.request_repaint(); };
            let Launched { backend, cpu, target, symbols, autorun, halt_bp } = match make() { Ok(l) => l, Err(e) => { publish(&|s| { s.starting = false; s.ended = true; s.error = Some(e.clone()); }); return; } };
            let cpu_t = cpu.clone();
            let mut sess = Session::new(backend, cpu, target, symbols);
            let caps = sess.backend.caps();
            let step_block = sess.backend.step_blocked();
            let (can_frame, can_text, sources) = (sess.backend.can_grab_frame(), sess.backend.can_type_text(), sess.backend.frame_sources());
            publish(&|s| { s.can_frame = can_frame; s.can_text = can_text; s.frame_sources = sources.iter().map(|x| x.to_string()).collect(); });
            publish(&|s| { s.step_block = step_block.clone(); s.caps = CapsInfo { registers: caps.registers, step: caps.step, breakpoints: caps.breakpoints, watch: caps.watch }; });
            let mut chip_wins: Vec<(u16, u16)> = vec![];
            for a in bps { let _ = sess.set_breakpoint(a, true, None, 0); }
            if let Some(a) = halt_bp { let _ = sess.set_breakpoint(a, true, None, 0); }
            if autorun { let _ = sess.cont(); }   // der Emulator wartet nach dem Verbinden; das Programm läuft los (Testfall: angehalten an der Falle)
            publish(&|s| s.starting = false);
            let mut watches: Vec<String> = vec![];
            // Bild: `frame_on` = Reiter „Emulator“ offen (Bild bei jedem Halt), `slow` = Zeitlupen-Live (Bild in kurzen Abständen, die Maschine läuft dazwischen)
            let (mut frame_on, mut slow) = (false, false);
            let mut mem_addr = 0x0400u16; let mut last: (bool, u16) = (true, 0xFFFF); let mut dirty = true;
            loop {
                match rx.recv_timeout(Duration::from_millis(if slow { 20 } else { 100 })) {
                    Ok(cmd) => { dirty = true; match cmd {
                        Cmd::Bp(a, on) => { let was_running = sess.backend.state() == vc_emu::RunState::Running; if let Err(e) = sess.set_breakpoint(a, on, None, 0) { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } else if was_running { let _ = sess.cont(); } }
                        Cmd::Cont => { if let Err(e) = sess.cont() { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } }
                        Cmd::Pause => { if let Err(e) = sess.pause() { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } }
                        Cmd::Step(k) => { if let Err(e) = sess.step(k) { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } }
                        Cmd::Mem(a) => mem_addr = a,
                        Cmd::Chips(w) => chip_wins = w,
                        Cmd::AddPoint { kind, start, end, cond, ignore } => {
                            let r = if kind == "exec" { sess.set_breakpoint(start, true, cond.as_deref(), ignore) } else { sess.set_watch(start, end, &kind, cond.as_deref(), ignore).map(|_| ()) };
                            if let Err(e) = r { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); }
                        }
                        Cmd::SaveCharset { addr, profile, name, root, data } => {
                            let r = (|| -> Result<String, String> {
                                let profiles = vc_media::load_profiles(&data)?; let p = vc_media::profile(&profiles, &profile)?;
                                let len = p.count * if p.block > 0 { p.block } else { p.width * p.height * p.bpp / 8 };
                                if addr as usize + len > 0x10000 { return Err(format!("{len} Bytes ab ${addr:04X} reichen über das Speicherende")); }
                                let bytes = sess.backend.read_mem(addr, len)?;
                                let a = vc_media::decode(p, &bytes, &name)?;
                                let rel = format!("graphics/dump/{name}.vcchar"); let out = root.join(&rel);
                                if out.exists() { return Err(format!("{rel} existiert bereits")); }
                                std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
                                std::fs::write(&out, vc_media::project::format_media(&a)).map_err(|e| e.to_string())?; Ok(rel)
                            })();
                            publish(&|s| s.notes.push(match &r { Ok(rel) => (true, format!("Zeichensatz aus dem Speicher gespeichert: {rel}")), Err(e) => (false, format!("Zeichensatz speichern: {e}")) }));
                        }
                        Cmd::SnapDetect(system, data) => {
                            let pal = crate::bitmap::Palettes::load(&data);
                            let r = crate::snapshot::detect_with_preview(&system, &mut crate::snapshot::BackendMem(&mut *sess.backend), &pal);
                            publish(&|s| s.detected = Some(r.clone()));
                        }
                        Cmd::SnapSave { system: _, det, opts, root, data } => {
                            let r = (|| -> Result<Vec<String>, String> {
                                let profiles = vc_media::load_profiles(&data)?;
                                let items = crate::snapshot::build_media(&det, &opts, &mut crate::snapshot::BackendMem(&mut *sess.backend), &profiles)?;
                                let mut names = vec![];
                                for (rel, v) in items {
                                    let out = root.join(&rel); std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
                                    std::fs::write(&out, vc_media::project::format_media(&v)).map_err(|e| e.to_string())?; names.push(rel);
                                }
                                Ok(names)
                            })();
                            publish(&|s| match &r { Ok(n) => for x in n { s.notes.push((true, format!("Momentaufnahme gespeichert: {x}"))) }, Err(e) => s.notes.push((false, format!("Momentaufnahme: {e}"))) });
                        }
                        Cmd::DelPoint(id) => { if let Err(e) = sess.clear_point(id) { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } }
                        Cmd::Watches(w) => watches = w,
                        Cmd::Frame => if can_frame { grab_and_publish(&mut *sess.backend, &publish); },
                        Cmd::FrameMode { auto, slow: sl } => { frame_on = auto && can_frame; slow = sl && can_frame; }
                        Cmd::FrameSource(i) => { sess.backend.set_frame_source(i); if can_frame && (frame_on || slow) { grab_and_publish(&mut *sess.backend, &publish); } }
                        Cmd::Text(t) => { if let Err(e) = sess.backend.type_text(&vc_emu::ascii_to_petscii(&t)) { publish(&|s| { s.error = Some(e.clone()); s.error_seq += 1; }); } }
                        Cmd::Stop => break,
                    } }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
                let st = sess.state();
                let running = st["state"] == "running";
                let pc = st["regs"]["pc"].as_u64().unwrap_or(0xFFFF) as u16;
                if slow && running { grab_and_publish(&mut *sess.backend, &publish); }         // Zeitlupen-Live: kurz anhalten, Bild holen, fortsetzen
                if !dirty && (running, pc) == last { continue; }
                last = (running, pc); dirty = false;
                let (dis, mem) = if running { (None, None) } else {
                    (sess.disasm(pc, 18).ok().map(|v| v["lines"].as_array().map(|a| a.iter().map(|l| DisLine { addr: l["addr"].as_u64().unwrap_or(0) as u16, bytes: l["bytes"].as_array().map(|b| b.iter().map(|x| x.as_u64().unwrap_or(0) as u8).collect()).unwrap_or_default(), text: l["text"].as_str().unwrap_or("").into(), illegal: l["illegal"].as_bool().unwrap_or(false), cycles: 0, extra: false }).map(|mut d| { if let Some(o) = d.bytes.first().and_then(|b| cpu_t.by_opcode(*b)) { d.cycles = o.cycles; d.extra = o.page_cross || o.branch; } d }).collect::<Vec<_>>()).unwrap_or_default()),
                     sess.backend.read_mem(mem_addr, 256).ok())
                };
                let chips: Vec<(u16, Vec<u8>)> = if running || !caps.registers { vec![] } else { chip_wins.iter().filter_map(|(a, n)| sess.backend.read_mem(*a, *n as usize).ok().map(|b| (*a, b))).collect() };
                let regs = &st["regs"];
                let points: Vec<PointInfo> = sess.points_json().iter().map(|p| PointInfo { id: p["id"].as_u64().unwrap_or(0) as u32, kind: p["kind"].as_str().unwrap_or("").into(), start: p["start"].as_u64().unwrap_or(0) as u16, end: p["end"].as_u64().unwrap_or(0) as u16, cond: p["cond"].as_str().map(String::from), ignore: p["ignore"].as_u64().unwrap_or(0) as u32, hits: p["hits"].as_u64().unwrap_or(0) as u32 }).collect();
                let wv: Vec<(String, Result<i64, String>)> = if running { vec![] } else { watches.iter().map(|w| (w.clone(), sess.eval(w))).collect() };
                publish(&|s| {
                    s.points = points.clone(); if !running { s.watch = wv.clone(); }
                    s.running = running; s.backend = st["backend"].as_str().unwrap_or("").into(); s.can_pause = st["canPause"].as_bool().unwrap_or(false);
                    s.breakpoints = st["breakpoints"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|v| v as u16)).collect()).unwrap_or_default();
                    s.stopped_at = st["stoppedAt"].as_u64().map(|v| v as u32); s.cond_error = st["condError"].as_str().map(String::from);
                    if regs.is_object() { s.have_regs = true; s.pc = pc; s.a = regs["a"].as_u64().unwrap_or(0) as u8; s.x = regs["x"].as_u64().unwrap_or(0) as u8; s.y = regs["y"].as_u64().unwrap_or(0) as u8; s.sp = regs["sp"].as_u64().unwrap_or(0) as u8; s.flags = regs["flags"].as_u64().unwrap_or(0) as u8;
                        s.extra = regs["extra"].as_array().map(|a| a.iter().map(|e| (e["name"].as_str().unwrap_or("").to_string(), e["value"].as_i64().unwrap_or(0))).collect()).unwrap_or_default(); }
                    if let Some(d) = &dis { s.disasm = d.clone(); }
                    if let Some(m) = &mem { s.mem_prev = if s.mem_addr == mem_addr && s.mem.len() == m.len() { std::mem::take(&mut s.mem) } else { vec![] }; s.mem = m.clone(); s.mem_addr = mem_addr; }
                    if !running { s.chips = chips.clone(); }
                });
                if !running && frame_on && !slow { grab_and_publish(&mut *sess.backend, &publish); }       // Bild beim Halt
            }
            sess.backend.stop();
            publish(&|s| { s.ended = true; s.running = false; });
        });
        Debugger { tx, snap }
    }
    /// Selbsttest ohne Emulator: festes Zustandsabbild, Befehle werden verworfen
    pub fn demo(snap: Snap) -> Debugger { let (tx, _rx) = channel::<Cmd>(); Debugger { tx, snap: Arc::new(Mutex::new(snap)) } }
    pub fn send(&self, c: Cmd) { let _ = self.tx.send(c); }
    pub fn snapshot(&self) -> Snap { self.snap.lock().map(|s| s.clone()).unwrap_or_default() }
}

/// Befehlsgeber für das Beenden beim Fensterschließen
impl Drop for Debugger { fn drop(&mut self) { let _ = self.tx.send(Cmd::Stop); } }
