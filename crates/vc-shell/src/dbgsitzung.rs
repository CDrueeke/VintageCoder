// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Sitzung der Oberfläche (E6): startet den Arbeits-Thread aus `debug.rs`, gleicht Haltepunkte des Editors ab (Zeile ↔ Adresse über den letzten Bau),
//! liefert das Zustandsabbild als JSON (`debug_state`) und nimmt Befehle an (`debug_cmd`). In der Prüfbrücke (`VC_TEST_NO_SPAWN`) läuft statt eines Emulators
//! eine Attrappe (`VC_TEST_FAKE_DEBUG=1`), die ein Programm nur durch die Länge der Befehle abläuft.
use crate::debug::{Cmd, Debugger, Snap};
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Sitzung {
    pub dbg: Debugger, pub info: crate::bau::Info, bps_an: HashSet<u16>, run_to: Option<(u16, bool)>, fehler_seq: u32, notizen: usize,
    reg_alt: Option<[u16; 6]>, reg_neu: Option<[u16; 6]>, meldungen: Vec<(bool, String)>, root: PathBuf, data: PathBuf, system: String, test: bool, frame_seq_gesendet: u64,
}

fn adressen(info: &crate::bau::Info, bps: &HashMap<String, Vec<usize>>) -> HashSet<u16> {
    bps.iter().flat_map(|(rel, zeilen)| zeilen.iter().filter_map(|l| info.adresse_von(rel, *l))).collect()
}
fn bps_aus(a: &Value) -> HashMap<String, Vec<usize>> {
    a["bps"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_array().map(|z| z.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect()).unwrap_or_default())).collect()).unwrap_or_default()
}

pub fn starten(sh: &crate::cmds::Shell, root: PathBuf, data: PathBuf, a: &Value) -> Result<Value, Fehler> {
    let info = sh.bau.info.lock().unwrap_or_else(|e| e.into_inner()).clone().ok_or_else(|| Fehler::neu("debug-kein-bau"))?;
    beenden(sh);
    let bps = adressen(&info, &bps_aus(a)); let watches: Vec<String> = a["watches"].as_array().map(|w| w.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let cpu = vc_target::load_target(&data, &info.target).and_then(|t| vc_target::load_cpu(&data, &t.cpu)).map_err(|e| Fehler::mit("debug-fehler", json!({ "detail": e })))?;
    let dbg = if crate::bau::gesperrt() {
        #[cfg(feature = "testbridge")]
        { if std::env::var_os("VC_TEST_FAKE_DEBUG").is_some() { Debugger::start_backend(Box::new(attrappe::Attrappe::neu(&info, cpu.clone())), cpu, info.target.clone(), info.symbols.clone(), bps.iter().copied().collect(), false) } else { return Err(Fehler::neu("debug-testlauf")) } }
        #[cfg(not(feature = "testbridge"))]
        { let _ = (cpu, bps); return Err(Fehler::neu("debug-testlauf")) }
    } else {
        vc_ide::launch::stop_running();
        let run_on = sh.bau.ziel.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let machine = vc_ide::settings::effective().start_machine(&run_on, &info.target).cloned();
        if let Some(m) = machine.as_ref().filter(|m| m.kind == "mega65" && m.link_for(vc_ide::mega65net::Action::Debug) == vc_ide::mega65net::Link::Eth) { return Err(Fehler::mit("debug-nur-usb", json!({ "name": m.display_name() }))) }
        let (adapter, host, password): (String, Option<String>, Option<String>) = match machine {
            Some(m) if m.kind == "ultimate64" => ("ultimate64".into(), Some(m.host.clone()), Some(m.password.clone())),
            Some(m) if m.kind == "mega65" => ("mega65".into(), vc_ide::settings::mega65_spec(&m).ok(), None),
            _ => (if info.target == "mega65" { "mega65".into() } else { "emulator".into() }, if info.target == "mega65" { Some("xemu".into()) } else { None }, None),
        };
        let spec = vc_ide::debug::LaunchSpec { artifact: info.artifact.to_string_lossy().into_owned(), target: info.target.clone(), adapter, host, password, entry: if info.target == "mega65" { info.run } else { None }, extra_args: vc_ide::launch::project_args(&data, &root, &info.target) };
        Debugger::start(spec, cpu, info.symbols.clone(), bps.iter().copied().collect())
    };
    if !watches.is_empty() { dbg.send(Cmd::Watches(watches)) }
    let system = info.target.clone();
    let s = Sitzung { dbg, info, bps_an: bps, run_to: None, fehler_seq: 0, notizen: 0, reg_alt: None, reg_neu: None, meldungen: vec![], root, data, system, test: false, frame_seq_gesendet: 0 };
    *sh.debug.lock().unwrap_or_else(|e| e.into_inner()) = Some(s);
    Ok(json!({ "ok": true }))
}

/// „Diesen Test debuggen“ (E6): Emulator mit vorbereiteter Falle, angehalten am Anfang der Falle. Im Test (Prüfbrücke mit Attrappe) startet eine Attrappe ohne Emulator.
pub fn starten_test(sh: &crate::cmds::Shell, root: PathBuf, data: PathBuf, a: &Value) -> Result<Value, Fehler> {
    let rel = a["file"].as_str().unwrap_or(""); let test = a["test"].as_str().unwrap_or("").to_string();
    let file = crate::dateien::absolut(&root, rel)?;
    beenden(sh);
    let target = vc_ide::tests::TestFile::load(&file).map(|t| t.target).unwrap_or_else(|_| "c64".into());
    let cpu = vc_target::load_target(&data, &target).and_then(|t| vc_target::load_cpu(&data, &t.cpu)).map_err(|e| Fehler::mit("debug-fehler", json!({ "detail": e })))?;
    let info = crate::bau::Info { target: target.clone(), ..Default::default() };
    let dbg = if crate::bau::gesperrt() {
        #[cfg(feature = "testbridge")]
        { if std::env::var_os("VC_TEST_FAKE_DEBUG").is_some() { Debugger::start_backend(Box::new(attrappe::Attrappe::neu(&info, cpu.clone())), cpu, target.clone(), Default::default(), vec![], false) } else { return Err(Fehler::neu("debug-testlauf")) } }
        #[cfg(not(feature = "testbridge"))]
        { let _ = (cpu, &test); return Err(Fehler::neu("debug-testlauf")) }
    } else { vc_ide::launch::stop_running(); Debugger::start_test(file, data.clone(), test) };
    let s = Sitzung { dbg, info, bps_an: Default::default(), run_to: None, fehler_seq: 0, notizen: 0, reg_alt: None, reg_neu: None, meldungen: vec![], root, data, system: target, test: true, frame_seq_gesendet: 0 };
    *sh.debug.lock().unwrap_or_else(|e| e.into_inner()) = Some(s);
    Ok(json!({ "ok": true }))
}

pub fn beenden(sh: &crate::cmds::Shell) -> bool { sh.debug.lock().unwrap_or_else(|e| e.into_inner()).take().is_some() }

fn zahl16(a: &Value, k: &str) -> Result<u16, Fehler> { a[k].as_u64().filter(|v| *v <= 0xFFFF).map(|v| v as u16).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k }))) }

pub fn befehl(sh: &crate::cmds::Shell, a: &Value) -> Result<Value, Fehler> {
    let mut g = sh.debug.lock().unwrap_or_else(|e| e.into_inner()); let s = g.as_mut().ok_or_else(|| Fehler::neu("debug-keine-sitzung"))?;
    match a["kind"].as_str().unwrap_or("") {
        "cont" => s.dbg.send(Cmd::Cont), "pause" => s.dbg.send(Cmd::Pause),
        "over" => s.dbg.send(Cmd::Step("over")), "into" => s.dbg.send(Cmd::Step("into")), "out" => s.dbg.send(Cmd::Step("out")),
        "mem" => {
            // Adresse: Hex mit/ohne `$`, oder Symbol der gebauten Datei
            let t = a["text"].as_str().unwrap_or("");
            match crate::debug::speicher_adresse(t, &s.info.symbols) { Ok(ad) => s.dbg.send(Cmd::Mem(ad)), Err(e) => return Err(Fehler::mit("debug-fehler", json!({ "detail": e }))) }
        }
        "mem_addr" => s.dbg.send(Cmd::Mem(zahl16(a, "addr")?)),
        "point" => {
            let kind = a["pkind"].as_str().unwrap_or("exec").to_string(); let start = zahl16(a, "start")?; let end = a["end"].as_u64().map(|x| x as u16).unwrap_or(start);
            let cond = a["cond"].as_str().map(str::trim).filter(|c| !c.is_empty()).map(String::from); let ignore = a["ignore"].as_u64().unwrap_or(0) as u32;
            s.dbg.send(Cmd::AddPoint { kind, start, end, cond, ignore });
        }
        "delpoint" => s.dbg.send(Cmd::DelPoint(a["id"].as_u64().unwrap_or(0) as u32)),
        "watches" => s.dbg.send(Cmd::Watches(a["list"].as_array().map(|w| w.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default())),
        "chips" => s.dbg.send(Cmd::Chips(a["wins"].as_array().map(|w| w.iter().filter_map(|x| Some((x[0].as_u64()? as u16, x[1].as_u64()? as u16))).collect()).unwrap_or_default())),
        "frame" => s.dbg.send(Cmd::Frame),
        "framemode" => s.dbg.send(Cmd::FrameMode { auto: a["auto"].as_bool().unwrap_or(false), slow: a["slow"].as_bool().unwrap_or(false) }),
        "framesource" => s.dbg.send(Cmd::FrameSource(a["index"].as_u64().unwrap_or(0) as usize)),
        "text" => s.dbg.send(Cmd::Text(a["text"].as_str().unwrap_or("").to_string())),
        "snapdetect" => { let sys = s.system.clone(); s.dbg.send(Cmd::SnapDetect(sys)) }
        "snapsave" => {
            let sn = s.dbg.snapshot();
            let Some(Ok(det)) = sn.detected else { return Err(Fehler::neu("debug-keine-erkennung")) };
            let name: String = a["name"].as_str().unwrap_or("").trim().chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
            let name = if name.is_empty() { "momentaufnahme".to_string() } else { name };
            let opts = vc_ide::snapshot::Opts { screen: det.supported.then_some(det.screen), charset: det.charset, rom: a["rom"].as_bool().unwrap_or(false) && det.rom_off.is_some(), sprites: a["sprites"].as_bool().unwrap_or(false) && !det.sprites.is_empty(), tiles: a["tiles"].as_bool().unwrap_or(false), name };
            s.dbg.send(Cmd::SnapSave { det, opts, root: s.root.clone(), data: s.data.clone() });
        }
        "runto" => {
            let (rel, zeile) = (a["rel"].as_str().unwrap_or(""), a["line"].as_u64().unwrap_or(0) as usize);
            match s.info.adresse_von(rel, zeile) { None => return Err(Fehler::mit("debug-zeile-ohne-code", json!({ "zeile": zeile }))), Some(ad) => { s.dbg.send(Cmd::AddPoint { kind: "exec".into(), start: ad, end: ad, cond: None, ignore: 0 }); s.dbg.send(Cmd::Cont); s.run_to = Some((ad, false)) } }
        }
        "bps" => {
            let want = adressen(&s.info, &bps_aus(a)); let starting = s.dbg.snapshot().starting;
            if !starting { for ad in want.difference(&s.bps_an) { s.dbg.send(Cmd::Bp(*ad, true)) } for ad in s.bps_an.difference(&want) { s.dbg.send(Cmd::Bp(*ad, false)) } s.bps_an = want; }
        }
        _ => return Err(Fehler::neu("op-ungueltig")),
    }
    Ok(Value::Null)
}

fn hex(v: u16, n: usize) -> String { format!("{v:0n$X}") }

fn erkennung(d: &Result<vc_ide::snapshot::Detected, String>) -> Value {
    match d { Err(e) => json!({ "error": e }), Ok(d) => json!({ "system": d.system, "mode": d.mode, "screen": d.screen, "charset": d.charset, "charsetMulticolor": d.charset_multicolor, "romOff": d.rom_off, "supported": d.supported, "sprites": d.sprites.len(), "notes": d.notes }) }
}

/// Zustandsabbild für die Oberfläche; gleicht nebenbei Fehlermeldungen, „bis hierher laufen“ und die Registervergleiche ab
pub fn zustand(sh: &crate::cmds::Shell) -> Value {
    let mut g = sh.debug.lock().unwrap_or_else(|e| e.into_inner()); let Some(s) = g.as_mut() else { return json!({ "active": false }) };
    let sn: Snap = s.dbg.snapshot();
    if sn.error_seq != s.fehler_seq { s.fehler_seq = sn.error_seq; if let Some(e) = &sn.error { s.meldungen.push((false, e.clone())) } }
    else if sn.error_seq == 0 && sn.ended { if let Some(e) = &sn.error { s.meldungen.push((false, e.clone())) } }
    while s.notizen < sn.notes.len() { s.meldungen.push(sn.notes[s.notizen].clone()); s.notizen += 1 }
    if let Some((ad, gesehen)) = s.run_to {
        if sn.running { s.run_to = Some((ad, true)) }
        else if gesehen && !sn.starting { for p in sn.points.iter().filter(|p| p.kind == "exec" && p.start == ad && p.end == ad && !s.bps_an.contains(&ad)) { s.dbg.send(Cmd::DelPoint(p.id)) } s.run_to = None }
    }
    if sn.have_regs && !sn.running { let jetzt = [sn.pc, sn.a as u16, sn.x as u16, sn.y as u16, sn.sp as u16, sn.flags as u16]; if s.reg_neu != Some(jetzt) { s.reg_alt = s.reg_neu; s.reg_neu = Some(jetzt) } }
    let ort = (!sn.running && sn.have_regs).then(|| s.info.zeile_bei(sn.pc).map(|(f, n)| json!({ "file": f, "line": n }))).flatten();
    let disasm: Vec<Value> = if sn.running { vec![] } else { sn.disasm.iter().map(|l| {
        let z = s.info.zeile_bei(l.addr);
        json!({ "addr": l.addr, "bytes": l.bytes, "text": l.text, "illegal": l.illegal, "cycles": l.cycles, "extra": l.extra, "label": s.info.marke_bei(l.addr), "line": z.map(|(f, n)| json!({ "file": f, "line": n })), "bp": sn.points.iter().any(|p| p.kind == "exec" && p.start == l.addr) || s.bps_an.contains(&l.addr) })
    }).collect() };
    let meld = std::mem::take(&mut s.meldungen);
    let ended = sn.ended;
    let out = json!({
        "active": true, "starting": sn.starting, "ended": ended, "running": sn.running, "haveRegs": sn.have_regs, "backend": sn.backend, "canPause": sn.can_pause, "stepBlock": sn.step_block, "error": sn.error, "condError": sn.cond_error,
        "caps": { "registers": sn.caps.registers, "step": sn.caps.step, "breakpoints": sn.caps.breakpoints, "watch": sn.caps.watch },
        "regs": { "pc": sn.pc, "a": sn.a, "x": sn.x, "y": sn.y, "sp": sn.sp, "flags": sn.flags, "extra": sn.extra.iter().map(|(n, v)| json!({ "name": n, "value": v })).collect::<Vec<_>>(), "prev": s.reg_alt },
        "ort": ort, "disasm": disasm, "stack": sn.stack, "points": sn.points.iter().map(|p| json!({ "id": p.id, "kind": p.kind, "start": p.start, "end": p.end, "cond": p.cond, "ignore": p.ignore, "hits": p.hits })).collect::<Vec<_>>(),
        "mem": { "addr": sn.mem_addr, "bytes": sn.mem, "prev": sn.mem_prev },
        "watch": sn.watch.iter().map(|(e, r)| match r { Ok(v) => json!({ "expr": e, "value": v }), Err(m) => json!({ "expr": e, "error": m }) }).collect::<Vec<_>>(),
        "chips": sn.chips.iter().map(|(a, b)| json!({ "addr": a, "bytes": b })).collect::<Vec<_>>(),
        "detected": sn.detected.as_ref().map(erkennung),
        "frame": { "can": sn.can_frame, "seq": sn.frame_seq, "error": sn.frame_err, "sources": sn.frame_sources, "canText": sn.can_text },
        "meldungen": meld.iter().map(|(ok, t)| json!({ "ok": ok, "text": t })).collect::<Vec<_>>(),
        "test": s.test,
    });
    if ended { *g = None; }
    out
}

/// Bild des Emulators als PNG-Datenadresse (nur wenn neuer als `seq`)
pub fn bild(sh: &crate::cmds::Shell, seq: u64) -> Value {
    let mut g = sh.debug.lock().unwrap_or_else(|e| e.into_inner()); let Some(s) = g.as_mut() else { return Value::Null };
    let sn = s.dbg.snapshot();
    let Some(f) = sn.frame.as_ref().filter(|_| sn.frame_seq != seq) else { return Value::Null };
    s.frame_seq_gesendet = sn.frame_seq;
    json!({ "seq": sn.frame_seq, "w": f.width, "h": f.height, "img": crate::kacheln::png_adresse(f.width, f.height, &f.rgba) })
}

#[cfg(feature = "testbridge")]
pub mod attrappe {
    //! Attrappe eines Emulators für die Prüfbrücke: Speicher mit dem gebauten Programm, Befehle werden nur über ihre Länge abgelaufen (A zählt hoch, X/Y wechseln),
    //! „Weiter“ läuft nach 200 ms bis zum nächsten Haltepunkt (ohne Haltepunkt bleibt sie laufend).
    use std::time::Instant;
    use vc_emu::{Backend, Regs, RunState};
    pub struct Attrappe { mem: Vec<u8>, regs: Regs, running: Option<Instant>, bps: Vec<u16>, cpu: vc_target::Cpu }
    impl Attrappe {
        pub fn neu(info: &crate::bau::Info, cpu: vc_target::Cpu) -> Attrappe {
            let mut mem = vec![0u8; 0x10000]; let bytes = std::fs::read(&info.artifact).unwrap_or_default();
            // PRG: zwei Byte Ladeadresse; sonst ab dem Start der ersten Segmente belegen
            let (start, daten) = if bytes.len() > 2 { (u16::from_le_bytes([bytes[0], bytes[1]]) as usize, &bytes[2..]) } else { (0x0801, &bytes[..]) };
            for (i, b) in daten.iter().enumerate() { if start + i < 0x10000 { mem[start + i] = *b } }
            let pc = info.run.unwrap_or(start as u16);
            Attrappe { mem, regs: Regs { pc, a: 0, x: 0, y: 0, sp: 0xF8, flags: 0x24, ..Default::default() }, running: None, bps: vec![], cpu }
        }
        fn laenge(&self, pc: u16) -> u16 { self.cpu.by_opcode(self.mem[pc as usize]).map_or(1, |o| o.bytes as u16).max(1) }
        fn schritt(&mut self) { let l = self.laenge(self.regs.pc); self.regs.pc = self.regs.pc.wrapping_add(l); self.regs.a = self.regs.a.wrapping_add(1); self.regs.x ^= 0x03; self.regs.flags ^= 0x02; }
    }
    impl Backend for Attrappe {
        fn name(&self) -> &str { "Attrappe" }
        fn state(&mut self) -> RunState {
            if let Some(t) = self.running { if t.elapsed().as_millis() > 200 && !self.bps.is_empty() { for _ in 0..200 { self.schritt(); if self.bps.contains(&self.regs.pc) { self.running = None; break } } } }
            if self.running.is_some() { RunState::Running } else { RunState::Stopped }
        }
        fn registers(&mut self) -> Result<Regs, String> { Ok(self.regs.clone()) }
        fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> { Ok((0..len).map(|i| self.mem[(addr as usize + i) & 0xFFFF]).collect()) }
        fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> { for (i, b) in data.iter().enumerate() { self.mem[(addr as usize + i) & 0xFFFF] = *b } Ok(()) }
        fn step(&mut self) -> Result<(), String> { self.schritt(); Ok(()) }
        fn cont(&mut self) -> Result<(), String> { self.running = Some(Instant::now()); Ok(()) }
        fn can_pause(&self) -> bool { true }
        fn pause(&mut self) -> Result<(), String> { self.running = None; Ok(()) }
        fn set_breakpoint(&mut self, addr: u16) -> Result<(), String> { if !self.bps.contains(&addr) { self.bps.push(addr) } Ok(()) }
        fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String> { self.bps.retain(|b| *b != addr); Ok(()) }
        fn breakpoints(&self) -> Vec<u16> { self.bps.clone() }
        fn can_grab_frame(&self) -> bool { true }
        fn grab_frame(&mut self) -> Result<vc_emu::streams::video::Frame, String> {
            let (w, h) = (64usize, 48usize); let n = self.regs.a as usize; let mut rgba = vec![255u8; w * h * 4];
            for y in 0..h { for x in 0..w { let i = (y * w + x) * 4; rgba[i] = ((x * 4 + n * 7) & 0xFF) as u8; rgba[i + 1] = (y * 5) as u8; rgba[i + 2] = 0x80; } }
            Ok(vc_emu::streams::video::Frame { width: w, height: h, rgba, frame_no: self.regs.a as u16 })
        }
        fn can_type_text(&self) -> bool { true }
        fn type_text(&mut self, _petscii: &[u8]) -> Result<(), String> { Ok(()) }
        fn stop(&mut self) {}
    }
}
