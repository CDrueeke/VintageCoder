// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Sitzung der IDE: ein laufender Emulator-Adapter, Disassembler und Haltepunkte (Ausführung, Datenzugriff, Bedingung, Ignorieren).
use crate::cond::{Ctx, Expr};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use vc_asm::disassemble;
use vc_emu::{Backend, Hit, Regs, RunState, WatchKind};
use vc_target::Cpu;

pub struct LaunchSpec { pub artifact: String, pub target: String, pub adapter: String, pub host: Option<String>, pub password: Option<String>, pub entry: Option<u16>, /** zusätzliche Emulator-Argumente (z. B. `-memory 8k` beim VC-20) */ pub extra_args: Vec<String> }
pub type Launcher = Box<dyn Fn(&LaunchSpec) -> Result<Box<dyn Backend>, String> + Send + Sync>;

/// Haltepunkt der Oberfläche (der Adapter kennt nur Adressen bzw. Bereiche; Bedingung und Ignorieren wertet die Sitzung aus).
#[derive(Debug, Clone)]
pub struct Point { pub id: u32, pub kind: String, pub start: u16, pub end: u16, pub cond: Option<Expr>, pub ignore: u32, pub hits: u32, watch_id: Option<u32> }

static SESSION_IDS: AtomicU64 = AtomicU64::new(1);

pub struct Session {
    pub backend: Box<dyn Backend>, pub cpu: Cpu, pub target: String, pub id: u64,
    points: Vec<Point>, next_point: u32, symbols: HashMap<String, i64>, user_stop: bool, last_stop: Option<u32>, cond_error: Option<String>,
}

/// Einsprungadresse aus dem BASIC-Starter (`SYS nnnn` in der ersten Zeile eines PRG); `None`, wenn keiner vorhanden ist
pub fn sys_target(prg: &[u8]) -> Option<u16> {
    let body = prg.get(2..)?; let i = body.iter().take(12).position(|b| *b == 0x9E)?;
    let digits: String = body[i + 1..].iter().skip_while(|b| **b == b' ').take_while(|b| b.is_ascii_digit()).map(|b| *b as char).collect();
    digits.parse().ok()
}

/// Standard-Starter: VICE für `c64`, atari800 für `atari8`, die Ultimate-Hardware über `adapter = "ultimate64"`.
pub fn default_launcher() -> Launcher {
    Box::new(|s: &LaunchSpec| -> Result<Box<dyn Backend>, String> {
        if s.adapter == "ultimate64" {
            let host = s.host.as_deref().filter(|h| !h.is_empty()).ok_or("Ultimate 64: Adresse (host) fehlt")?;
            let mut u = vc_emu::Ultimate64::connect(host, s.password.as_deref())?;
            if s.target != "c64" { return Err("Die Ultimate-Hardware spielt nur C64-Programme".into()); }
            let prg = std::fs::read(&s.artifact).map_err(|e| format!("{}: {e}", s.artifact))?;
            u.run_prg(&prg)?;
            return Ok(Box::new(u));
        }
        if s.adapter == "mega65" {
            // `host`: "xemu" (startet Xemu: VC_XEMU, VC_XEMU_SD = Ordner mit MEGA65.ROM), "unix:/pfad" (laufender Xemu mit -uartmon) oder "/dev/cu.usbserial-…" (Hardware)
            let spec = s.host.as_deref().unwrap_or("xemu");
            let mut m = if spec == "xemu" || spec.is_empty() {
                let xemu = crate::settings::effective().emulator_for("mega65");
                let art = std::path::Path::new(&s.artifact); let root = crate::launch::project_root_of(art);
                let xargs = xemu.debug_args(&crate::args::ArgCtx::from_launch(&root, art, &s.target, ""))?;
                let rp = crate::roms::plan("xemu", &crate::roms::store(Some(&root), Some(&xemu)), &xargs);
                if let Some(e) = rp.error { return Err(e); }
                let rom = rp.xemu_rom.ok_or("MEGA65-ROM nicht gefunden")?.to_string_lossy().into_owned();
                vc_emu::Mega65::launch_xemu_rom(xemu.path.as_deref(), &rom, &xargs)?
            } else { vc_emu::Mega65::connect(spec)? };
            let prg = std::fs::read(&s.artifact).map_err(|e| format!("{}: {e}", s.artifact))?;
            let load = prg.get(0).copied().unwrap_or(1) as u16 | (prg.get(1).copied().unwrap_or(0x20) as u16) << 8;
            m.load_prg(&prg, s.entry.or_else(|| sys_target(&prg)).unwrap_or(load))?;
            return Ok(Box::new(m));
        }
        let st = crate::settings::effective();
        // Platzhalter und Katalogauswahl wie beim normalen Start; was der Debug-Start selbst setzt, fällt weg
        let art = std::path::Path::new(&s.artifact);
        let root = crate::launch::project_root_of(art);
        let ctx = crate::args::ArgCtx::from_launch(&root, art, &s.target, &crate::launch::atari_variant(&root));
        match s.target.as_str() {
            "c64" | "c128" | "plus4" | "c16" => {
                let vice = st.vice_for(&s.target);
                crate::emulators::require(&vice.kind)?;
                let user = vice.debug_args(&ctx)?;
                let mut a = crate::roms::plan(&vice.kind, &crate::roms::store(Some(&root), Some(&vice)), &user).args;
                a.extend(user);
                crate::emulators::launch_backend(&vice.kind, &vc_emu::Launch { kind: &vice.kind, file: Some(&s.artifact), exe: vice.path.as_deref(), args: &a, ..Default::default() })
            }
            "vic20" => {
                let vice = st.vice_for("vic20");
                crate::emulators::require(&vice.kind)?;
                let user = vice.debug_args(&ctx)?;
                let mut a = s.extra_args.clone(); a.extend(crate::roms::plan(&vice.kind, &crate::roms::store(Some(&root), Some(&vice)), &user).args); a.extend(user);
                // Steckmodul (.a0, .20, …): als Modul einlegen statt per Autostart laden
                let art = match crate::launch::vic20_cart_arg(std::path::Path::new(&s.artifact)) { Some(c) => { a.extend(c); "" } None => s.artifact.as_str() };
                crate::emulators::launch_backend(&vice.kind, &vc_emu::Launch { kind: &vice.kind, file: Some(art), exe: vice.path.as_deref(), args: &a, ..Default::default() })
            }
            "atari8" => {
                // Fujisan (TCP, mehrere Haltepunkte, Pause per Befehl) ist der Standard, wenn gefunden und das Programm ein .xex ist; sonst atari800
                let want = st.emulators.fujisan.debug_for_atari.trim().to_ascii_lowercase();
                let is_xex = s.artifact.to_ascii_lowercase().ends_with(".xex");
                let fuji = if want == "atari800" || !crate::emulators::enabled("fujisan") { None } else { crate::settings::program_path("fujisan").or_else(|| crate::launch::detect("fujisan")) };
                if let Some(exe) = fuji.filter(|_| is_xex) {
                    let port = std::env::var("VC_FUJISAN_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(vc_emu::fujisan::DEFAULT_PORT);
                    return crate::emulators::launch_backend("fujisan", &vc_emu::Launch { kind: "fujisan", file: Some(&s.artifact), exe: Some(&exe), port, ..Default::default() });
                }
                if want == "fujisan" && crate::emulators::enabled("fujisan") { return Err(if !is_xex { "Der Fujisan-Debugger lädt nur .xex-Dateien".into() } else { format!("Fujisan nicht gefunden ({})", vc_emu::fujisan::install_hint()) }); }
                crate::emulators::require("atari800")?;
                let a8 = st.emulator_of_kind("atari8", "atari800");
                let user = a8.debug_args(&ctx)?;
                let mut a = crate::roms::plan("atari800", &crate::roms::store(Some(&root), Some(&a8)), &user).args;
                a.extend(user);
                crate::emulators::launch_backend("atari800", &vc_emu::Launch { kind: "atari800", file: Some(&s.artifact), exe: a8.path.as_deref(), args: &a, ..Default::default() })
            }
            t => Err(format!("kein Debug-Adapter für Target '{t}'")),
        }
    })
}

fn regs_json(r: &Regs) -> Value {
    json!({ "pc": r.pc, "a": r.a, "x": r.x, "y": r.y, "sp": r.sp, "flags": r.flags, "extra": r.extra.iter().map(|(n, v)| json!({ "name": n, "value": v })).collect::<Vec<_>>() })
}

struct EvalCtx<'a> { regs: &'a Regs, backend: &'a mut dyn Backend }
impl Ctx for EvalCtx<'_> {
    fn reg(&self, name: &str) -> Option<i64> {
        let r = self.regs;
        Some(match name { "a" => r.a as i64, "x" => r.x as i64, "y" => r.y as i64, "sp" | "s" => r.sp as i64, "pc" => r.pc as i64, "p" | "flags" => r.flags as i64,
            "fn" => (r.flags >> 7 & 1) as i64, "fv" => (r.flags >> 6 & 1) as i64, "fb" => (r.flags >> 4 & 1) as i64, "fd" => (r.flags >> 3 & 1) as i64, "fi" => (r.flags >> 2 & 1) as i64, "fz" => (r.flags >> 1 & 1) as i64, "fc" => (r.flags & 1) as i64,
            other => return r.extra.iter().find(|(n, _)| n.eq_ignore_ascii_case(other)).map(|(_, v)| *v as i64) })
    }
    fn peek(&mut self, addr: u16) -> Result<u8, String> { self.backend.read_mem(addr, 1)?.first().copied().ok_or_else(|| "Speicher nicht lesbar".to_string()) }
}

impl Session {
    pub fn new(backend: Box<dyn Backend>, cpu: Cpu, target: String, symbols: HashMap<String, i64>) -> Session {
        Session { backend, cpu, target, id: SESSION_IDS.fetch_add(1, Ordering::SeqCst), points: vec![], next_point: 1, symbols, user_stop: false, last_stop: None, cond_error: None }
    }

    pub fn points_json(&self) -> Vec<Value> {
        self.points.iter().map(|p| json!({ "id": p.id, "kind": p.kind, "start": p.start, "end": p.end, "cond": p.cond.as_ref().map(|c| c.source.clone()), "ignore": p.ignore, "hits": p.hits })).collect()
    }

    pub fn state(&mut self) -> Value {
        // Bedingte Haltepunkte, die nicht zutreffen, nicht als Stopp zeigen (der Emulator kann nach dem Fortsetzen sofort wieder halten)
        for _ in 0..256 { self.tick(); if self.user_stop || self.points.is_empty() || self.backend.state() == RunState::Running { break; } }
        let caps = self.backend.caps();
        let running = self.backend.state() == RunState::Running;
        let mut v = json!({ "active": true, "backend": self.backend.name(), "state": if running { "running" } else { "stopped" }, "canPause": self.backend.can_pause(),
                            "breakpoints": self.backend.breakpoints(), "points": self.points_json(), "stoppedAt": self.last_stop,
                            "caps": { "registers": caps.registers, "step": caps.step, "breakpoints": caps.breakpoints, "watch": caps.watch } });
        if let Some(e) = &self.cond_error { v["condError"] = json!(e); }
        if !running && caps.registers {
            match self.backend.registers() { Ok(r) => v["regs"] = regs_json(&r), Err(e) => v["error"] = json!(e) }
        }
        v
    }

    fn parse_cond(&self, cond: Option<&str>) -> Result<Option<Expr>, String> {
        match cond.map(str::trim).filter(|c| !c.is_empty()) { None => Ok(None), Some(c) => Expr::parse(c, &self.symbols).map(Some).map_err(|e| format!("Bedingung '{c}': {e}")) }
    }
    fn pause_for_edit(&mut self) -> Result<(), String> {
        if self.backend.state() == RunState::Running { self.backend.pause().map_err(|e| format!("Haltepunkte nur bei angehaltenem Emulator: {e}"))?; self.user_stop = true; }
        Ok(())
    }

    /// Ausführungs-Haltepunkt setzen/entfernen (ein Punkt je Adresse; erneutes Setzen aktualisiert Bedingung und Ignorieren).
    pub fn set_breakpoint(&mut self, addr: u16, on: bool, cond: Option<&str>, ignore: u32) -> Result<(), String> {
        let cond = if on { self.parse_cond(cond)? } else { None };
        self.pause_for_edit()?;
        if on {
            self.backend.set_breakpoint(addr)?;
            match self.points.iter_mut().find(|p| p.kind == "exec" && p.start == addr) {
                Some(p) => { p.cond = cond; p.ignore = ignore; }
                None => { let id = self.next_point; self.next_point += 1; self.points.push(Point { id, kind: "exec".into(), start: addr, end: addr, cond, ignore, hits: 0, watch_id: None }); }
            }
        } else { self.backend.clear_breakpoint(addr)?; self.points.retain(|p| !(p.kind == "exec" && p.start == addr)); }
        Ok(())
    }

    /// Datenhaltepunkt (`read`, `write`, `access`) auf `start..=end`
    pub fn set_watch(&mut self, start: u16, end: u16, kind: &str, cond: Option<&str>, ignore: u32) -> Result<u32, String> {
        if end < start { return Err("Ende liegt vor dem Anfang".into()); }
        let wk = match kind { "read" => WatchKind::Read, "write" => WatchKind::Write, "access" => WatchKind::Access, k => return Err(format!("Art '{k}' unbekannt (read, write, access)")) };
        let cond = self.parse_cond(cond)?;
        self.pause_for_edit()?;
        let wid = self.backend.set_watch(start, end, wk)?;
        let id = self.next_point; self.next_point += 1;
        self.points.push(Point { id, kind: kind.into(), start, end, cond, ignore, hits: 0, watch_id: Some(wid) });
        Ok(id)
    }
    pub fn clear_point(&mut self, id: u32) -> Result<(), String> {
        let p = self.points.iter().find(|p| p.id == id).cloned().ok_or("Haltepunkt unbekannt")?;
        self.pause_for_edit()?;
        match p.watch_id { Some(w) => self.backend.clear_watch(w)?, None => self.backend.clear_breakpoint(p.start)? }
        self.points.retain(|q| q.id != id); Ok(())
    }

    /// Wertet einen Ausdruck (Register, Symbole, `[adresse]`, Operatoren) im angehaltenen Zustand aus
    pub fn eval(&mut self, src: &str) -> Result<i64, String> {
        let e = Expr::parse(src, &self.symbols)?; let regs = self.backend.registers()?;
        e.eval(&mut EvalCtx { regs: &regs, backend: &mut *self.backend })
    }

    pub fn cont(&mut self) -> Result<(), String> { self.user_stop = false; self.last_stop = None; self.cond_error = None; self.backend.cont() }
    pub fn pause(&mut self) -> Result<(), String> { self.user_stop = true; self.backend.pause() }

    /// Wertet nach einem Halt die Bedingungen aus und setzt den Lauf fort, wenn kein Haltepunkt „zutrifft“.
    pub fn tick(&mut self) {
        if self.user_stop || self.points.is_empty() || self.backend.state() == RunState::Running { return; }
        let Ok(regs) = self.backend.registers() else { self.user_stop = true; return };
        let hit = self.backend.last_hit();
        let matched: Vec<usize> = self.points.iter().enumerate().filter(|(_, p)| match (hit, p.watch_id) {
            (Some(Hit::Watch(w)), Some(pw)) => w == pw,
            (Some(Hit::Exec(a)), None) => a == p.start,
            (None, None) => p.start == regs.pc,
            _ => false,
        }).map(|(i, _)| i).collect();
        if matched.is_empty() { self.user_stop = true; return; }          // anderer Grund (Pause, Einzelschritt, BRK …)
        let mut stop = false;
        for i in matched {
            self.points[i].hits += 1;
            if self.points[i].hits <= self.points[i].ignore { continue; }
            let verdict = match self.points[i].cond.clone() {
                None => Ok(true),
                Some(c) => c.holds(&mut EvalCtx { regs: &regs, backend: &mut *self.backend }),
            };
            match verdict { Ok(true) => { stop = true; self.last_stop = Some(self.points[i].id); } Ok(false) => {} Err(e) => { stop = true; self.last_stop = Some(self.points[i].id); self.cond_error = Some(format!("Bedingung: {e}")); } }
        }
        if stop { self.user_stop = true; } else { let _ = self.backend.cont(); }
    }

    fn wait_stopped(&mut self, ms: u64) -> bool {
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end { if self.backend.state() == RunState::Stopped { return true; } std::thread::sleep(Duration::from_millis(20)); }
        false
    }

    /// `into`: Einzelschritt; `over`: JSR wird übersprungen (temporärer Haltepunkt hinter dem Aufruf).
    pub fn step(&mut self, kind: &str) -> Result<(), String> {
        if !self.backend.caps().step { return Err("Dieser Adapter unterstützt keinen Einzelschritt".into()); }
        if let Some(why) = self.backend.step_blocked() { return Err(why); }
        if self.backend.state() == RunState::Running { return Err("Emulator läuft – zuerst anhalten".into()); }
        self.user_stop = true;
        if kind == "out" { return self.step_out(); }
        if kind == "over" {
            let pc = self.backend.registers()?.pc;
            if self.backend.read_mem(pc, 1)?.first() == Some(&0x20) {
                let ret = pc.wrapping_add(3);
                let temp = !self.backend.breakpoints().contains(&ret);
                if temp { self.backend.set_breakpoint(ret)?; }
                self.backend.cont()?;
                let ok = self.wait_stopped(5000);
                if temp { if !ok { self.backend.pause().ok(); } self.backend.clear_breakpoint(ret).ok(); }
                return if ok { Ok(()) } else { Err("Unterprogramm kehrt nicht zurück (innerhalb von 5 s)".into()) };
            }
        }
        self.backend.step()
    }

    /// `out` („Raus“): bis der aktuelle Unterprogramm-/Interrupt-Rücksprung (RTS/RTI) ausgeführt ist. Aufrufe darin (JSR) werden wie bei `over`
    /// übersprungen, alles andere in Einzelschritten; nach 10 s ohne Rücksprung (Endlosschleife, Warten auf Eingabe) Abbruch mit Meldung.
    fn step_out(&mut self) -> Result<(), String> {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            let pc = self.backend.registers()?.pc;
            let op = self.backend.read_mem(pc, 1)?.first().copied().unwrap_or(0);
            if op == 0x20 { self.step("over")?; } else { self.backend.step()?; }
            if op == 0x60 || op == 0x40 { return Ok(()); }
            if Instant::now() > end { return Err("Kein Rücksprung innerhalb von 10 s (Schleife im Unterprogramm?) – Haltepunkt am Rücksprung setzen".into()); }
        }
    }

    pub fn disasm(&mut self, addr: u16, count: usize) -> Result<Value, String> {
        let bytes = self.backend.read_mem(addr, (count * 3 + 3).min(0x10000 - addr as usize))?;
        let lines: Vec<Value> = disassemble(&self.cpu, &bytes, addr).into_iter().take(count)
            .map(|l| json!({ "addr": l.addr, "bytes": l.bytes, "text": l.text, "illegal": l.illegal })).collect();
        Ok(json!({ "lines": lines }))
    }
}
