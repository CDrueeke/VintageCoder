// SPDX-License-Identifier: GPL-3.0-or-later
//! Läufer des Trap-Modus ohne Emulator: ein Scheinadapter liest die Falle aus dem Speicher und führt je Aufrufziel eine vorgegebene
//! „Wirkung“ aus (kein 6502-Simulator: das Unterprogramm selbst wird nicht ausgeführt). Geprüft werden Falle, Setup, Auswertung der
//! Erwartungen, Zyklenbudget, Zeitüberschreitung und dass immer nur ein Emulator lebt.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use vc_emu::{Backend, Cycles, Regs, RunState, TrapCaps};
use vc_ide::tests::runner::{default_trap, run_tests, trap_code, RunOptions};
use vc_ide::tests::static_checks::{Dbg, Image};
use vc_ide::tests::{Level, Outcome, Prepared, State, TestFile};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

type Effect = Arc<dyn Fn(&mut Vec<u8>, &mut Regs) -> Option<u64> + Send + Sync>;      // Zyklen des Unterprogramms; None = kehrt nie zurück

struct Shared { alive: AtomicUsize, max_alive: AtomicUsize, launches: AtomicUsize, stops: AtomicUsize }

struct Fake {
    mem: Vec<u8>, regs: Regs, beam: u64, timer_start: Option<u64>,
    effects: HashMap<u16, Effect>, caps: TrapCaps, sh: Arc<Shared>, stopped: bool,
    pending_call: Option<u16>,
}

impl Fake {
    fn new(effects: HashMap<u16, Effect>, caps: TrapCaps, sh: Arc<Shared>) -> Fake {
        let n = sh.alive.fetch_add(1, Ordering::SeqCst) + 1;
        sh.max_alive.fetch_max(n, Ordering::SeqCst);
        sh.launches.fetch_add(1, Ordering::SeqCst);
        Fake { mem: vec![0; 0x10000], regs: Regs { sp: 0xFF, ..Default::default() }, beam: 1000, timer_start: None, effects, caps, sh, stopped: false, pending_call: None }
    }
    fn word(&self, a: u16) -> u16 { self.mem[a as usize] as u16 | (self.mem[a as usize + 1] as u16) << 8 }
    /// Liest die Falle ab PC: optional Vorspann (`A2 sp 9A`), dann `jsr ziel` oder (rti) `jmp ziel`; führt die Wirkung aus und liefert die Haltadresse
    fn execute_trap(&mut self) -> Option<u16> {
        let mut pc = self.regs.pc;
        if self.mem[pc as usize] == 0xA2 {                     // Vorspann (rts: 13 Bytes, rti: 20 Bytes)
            let rti = self.mem[pc as usize + 3] == 0xA9 && self.mem[pc as usize + 5] == 0x48 && self.mem[pc as usize + 8] == 0x48;
            let (sp, p, a, x, y) = if rti { (self.mem[pc as usize + 1], self.mem[pc as usize + 10], self.mem[pc as usize + 14], self.mem[pc as usize + 16], self.mem[pc as usize + 18]) }
                                   else { (self.mem[pc as usize + 1], self.mem[pc as usize + 4], self.mem[pc as usize + 7], self.mem[pc as usize + 9], self.mem[pc as usize + 11]) };
            self.regs = Regs { pc, a, x, y, sp, flags: p, extra: vec![] };
            if self.pending_call.is_none() { self.pending_call = Some(pc.wrapping_add(if rti { 20 } else { 13 })); return self.pending_call; }
            pc = self.pending_call.take().unwrap();
        }
        let op = self.mem[pc as usize];
        let target = self.word(pc + 1);
        let (cost, halt) = match op { 0x20 => (6, pc + 3), 0x4C => (3, pc + 3), x => panic!("unerwartete Falle: Opcode {x:02X} bei {pc:04X}") };
        let eff = self.effects.get(&target).unwrap_or_else(|| panic!("keine Wirkung für Ziel {target:04X}")).clone();
        let mut regs = self.regs.clone();
        match eff(&mut self.mem, &mut regs) {
            Some(c) => { self.beam += cost + c; self.regs = regs; self.regs.pc = halt; Some(halt) }
            None => { self.regs.pc = target; None }
        }
    }
}

impl Backend for Fake {
    fn name(&self) -> &str { "Schein" }
    fn trap_caps(&self) -> TrapCaps { self.caps }
    fn state(&mut self) -> RunState { if self.stopped { RunState::Stopped } else { RunState::Running } }
    fn registers(&mut self) -> Result<Regs, String> { Ok(self.regs.clone()) }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        if addr == 0xDD04 {
            let t = 0xFFFF_FFFFu64 - self.timer_start.map_or(0, |s| self.beam - s);
            return Ok((t as u32).to_le_bytes()[..len.min(4)].to_vec());
        }
        Ok(self.mem[addr as usize..addr as usize + len].to_vec())
    }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        if addr == 0xDD0F && data == [0x41] { self.timer_start = Some(self.beam); }
        self.mem[addr as usize..addr as usize + data.len()].copy_from_slice(data); Ok(())
    }
    fn set_regs(&mut self, r: &Regs) -> Result<(), String> { if !self.caps.set_regs && false { return Err("nein".into()); } self.regs = r.clone(); self.pending_call = None; Ok(()) }
    fn cycles(&mut self) -> Result<Cycles, String> { if self.caps.cycles { Ok(Cycles { line: ((self.beam / 63) % 312) as u32, cycle: (self.beam % 63) as u32, per_line: 63, lines: 312 }) } else { Err("keine Zyklen".into()) } }
    fn run_until(&mut self, addr: u16, _t: Duration) -> Result<bool, String> {
        match self.execute_trap() { Some(h) => { self.stopped = true; Ok(h == addr) } None => Ok(false) }
    }
    fn step(&mut self) -> Result<(), String> { Ok(()) }
    fn cont(&mut self) -> Result<(), String> { Ok(()) }
    fn can_pause(&self) -> bool { false }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn set_breakpoint(&mut self, _a: u16) -> Result<(), String> { Ok(()) }
    fn clear_breakpoint(&mut self, _a: u16) -> Result<(), String> { Ok(()) }
    fn breakpoints(&self) -> Vec<u16> { vec![] }
    fn stop(&mut self) { self.sh.stops.fetch_add(1, Ordering::SeqCst); }
}
impl Drop for Fake { fn drop(&mut self) { self.sh.alive.fetch_sub(1, Ordering::SeqCst); } }

fn effect(f: impl Fn(&mut Vec<u8>, &mut Regs) -> Option<u64> + Send + Sync + 'static) -> Effect { Arc::new(f) }

/// Programm: `inc_fb` ($0810) erhöht $FB (3 Zyklen mehr als rts), `hang` ($0820) kehrt nie zurück
struct Env { pre: Prepared, effects: HashMap<u16, Effect>, sh: Arc<Shared> }

fn env() -> Env { env_target("c64") }

fn env_target(id: &str) -> Env {
    let d = data();
    let target = vc_target::load_target(&d, id).unwrap();
    let cpu = vc_target::load_cpu(&d, &target.cpu).unwrap();
    let mut dbg = Dbg::default();
    for (n, v) in [("inc_fb", 0x0810), ("hang", 0x0820), ("irq", 0x0830), ("flags_probe", 0x0840)] { dbg.symbols.insert(n.to_string(), v); }
    let image = Image::from_runs(&[(0x0801, &[0x60; 0x80])]);
    let mut effects: HashMap<u16, Effect> = HashMap::new();
    effects.insert(0x0810, effect(|m, r| { m[0xFB] = m[0xFB].wrapping_add(1); r.a = m[0xFB]; Some(5 + 6) }));          // inc zp (5) + rts (6)
    effects.insert(0x0820, effect(|_, _| None));
    effects.insert(0x0830, effect(|m, r| { m[0xD020] = m[0xFB]; r.a = 0x55; Some(20) }));
    effects.insert(0x0840, effect(|_, r| { r.flags = 0x80 | 0x24; Some(8) }));
    Env { pre: Prepared { target, cpu, program: PathBuf::from("p.prg"), dbg, image: Ok(image) }, effects, sh: Arc::new(Shared { alive: AtomicUsize::new(0), max_alive: AtomicUsize::new(0), launches: AtomicUsize::new(0), stops: AtomicUsize::new(0) }) }
}

fn run(e: &Env, json: &str, caps: TrapCaps, ro: &RunOptions) -> Vec<Outcome> {
    let tf = TestFile::parse(Path::new("x.vctest"), json).unwrap();
    let (effects, sh) = (e.effects.clone(), e.sh.clone());
    let mut launch = move || -> Result<Box<dyn Backend>, String> { Ok(Box::new(Fake::new(effects.clone(), caps, sh.clone()))) };
    run_tests(&tf, &e.pre, &RunOptions { boot_wait: Duration::ZERO, ..ro.clone() }, &mut launch)
}
fn full() -> TrapCaps { TrapCaps { set_regs: true, cycles: true } }
fn file(tests: &str) -> String { format!(r#"{{ "target": "c64", "program": "p.prg", "tests": {tests} }}"#) }

#[test]
fn passing_test_measures_cycles_without_the_call_instruction() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "inc", "setup": { "mem": { "$fb": 41 } }, "call": "inc_fb", "expect": [ "[$fb] == 42", "a == 42", "cycles == 11", "fi" ], "cycles": { "max": 11 } } ]"#), full(), &RunOptions::default());
    assert_eq!(o.len(), 1);
    assert_eq!((o[0].state, o[0].measured.as_deref()), (State::Passed, Some("11 Zyklen")), "{:?}", o[0]);
    assert_eq!(e.sh.launches.load(Ordering::SeqCst), 1);
    assert_eq!(e.sh.stops.load(Ordering::SeqCst), 1, "Emulator wird am Ende beendet");
}

#[test]
fn failed_expectation_names_actual_value_and_state() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "falsch", "setup": { "mem": { "$fb": 1 }, "x": 9 }, "call": "inc_fb", "expect": [ "[$fb] == 5" ] } ]"#), full(), &RunOptions::default());
    assert_eq!(o[0].state, State::Failed);
    assert!(o[0].is_error());
    assert!(o[0].detail.contains("linke Seite = 2") && o[0].detail.contains("rechte Seite = 5"), "{}", o[0].detail);
    assert!(o[0].detail.contains("X=$09") && o[0].detail.contains("SP=$FF"), "Registerzustand: {}", o[0].detail);
}

#[test]
fn cycle_budget_is_a_hint_unless_error_or_strict() {
    let e = env();
    let t = |level: &str| file(&format!(r#"[ {{ "name": "b", "call": "inc_fb", "cycles": {{ "max": 10 {level} }} }} ]"#));
    let o = run(&e, &t(""), full(), &RunOptions::default());
    assert_eq!((o[0].state, o[0].level), (State::Failed, Level::Hint), "{:?}", o[0]);
    assert!(!o[0].is_error() && o[0].detail.contains("11 Zyklen gemessen") && o[0].detail.contains("1 zu viel"));
    assert!(run(&e, &t(r#", "level": "error""#), full(), &RunOptions::default())[0].is_error());
    assert!(run(&e, &t(""), full(), &RunOptions { strict: true, ..Default::default() })[0].is_error());
    assert_eq!(run(&e, &file(r#"[ { "name": "ok", "call": "inc_fb", "cycles": { "max": 11 } } ]"#), full(), &RunOptions::default())[0].state, State::Passed);
}

#[test]
fn timeout_restarts_the_emulator_and_later_tests_still_run() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "hängt", "call": "hang", "expect": [ "a == 0" ], "timeoutMs": 50 }, { "name": "danach", "call": "inc_fb", "expect": [ "[$fb] == 1" ] } ]"#), full(), &RunOptions::default());
    assert_eq!(o[0].state, State::TimedOut, "{:?}", o[0]);
    assert!(o[0].is_error() && o[0].detail.contains("nicht zurück"));
    assert_eq!(o[1].state, State::Passed, "{:?}", o[1]);
    assert_eq!(e.sh.launches.load(Ordering::SeqCst), 2, "nach der Zeitüberschreitung neu gestartet");
    assert_eq!(e.sh.max_alive.load(Ordering::SeqCst), 1, "immer nur ein Emulator");
    assert_eq!(e.sh.stops.load(Ordering::SeqCst), 2);
}

#[test]
fn one_emulator_per_file_unless_isolate() {
    let e = env();
    let two = r#"[ { "name": "a", "call": "inc_fb", "expect": [ "[$fb] == 1" ] }, { "name": "b", "call": "inc_fb", "expect": [ "[$fb] == 1" ] } ]"#;
    run(&e, &file(two), full(), &RunOptions::default());
    assert_eq!(e.sh.launches.load(Ordering::SeqCst), 1);
    let e = env();
    let json = format!(r#"{{ "target": "c64", "program": "p.prg", "isolate": true, "tests": {two} }}"#);
    let o = run(&e, &json, full(), &RunOptions::default());
    assert!(o.iter().all(|x| x.state == State::Passed), "{o:?}");
    assert_eq!((e.sh.launches.load(Ordering::SeqCst), e.sh.max_alive.load(Ordering::SeqCst)), (2, 1), "jeder Test startet neu, nie zwei zugleich");
}

#[test]
fn cycles_not_measurable_is_skipped_never_passed() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "z", "call": "inc_fb", "expect": [ "cycles < 100" ] }, { "name": "ohne Zyklen", "setup": { "mem": { "$fb": 0 } }, "call": "inc_fb", "expect": [ "[$fb] == 1" ] } ]"#), TrapCaps { set_regs: true, cycles: false }, &RunOptions::default());
    assert_eq!(o[0].state, State::Skipped, "{:?}", o[0]);
    assert!(o[0].detail.contains("misst keine Zyklen"), "{}", o[0].detail);
    assert_eq!(o[1].state, State::Passed);
}

#[test]
fn prologue_sets_flags_and_registers_for_adapters_without_register_access() {
    let e = env();
    let json = file(r#"[ { "name": "rts", "setup": { "mem": { "$fb": 6 }, "a": 3, "x": 4, "y": 5, "sp": "$f0", "fc": 1, "fn": 1 }, "call": "inc_fb", "expect": [ "[$fb] == 7", "cycles == 11" ] },
        { "name": "rti", "setup": { "mem": { "$fb": 6 }, "fz": 1 }, "call": "irq", "return": "rti", "expect": [ "([$d020]) == 6", "a == $55", "cycles == 20" ] } ]"#);
    for ro in [RunOptions { force_prologue: true, ..Default::default() }, RunOptions::default()] {
        let o = run(&e, &json, if ro.force_prologue { TrapCaps { set_regs: false, cycles: true } } else { full() }, &ro);
        assert!(o.iter().all(|x| x.state == State::Passed), "{o:?}");
    }
}

#[test]
fn trap_bytes() {
    let r = Regs { a: 0x11, x: 0x22, y: 0x33, sp: 0xF0, flags: 0xC5, ..Default::default() };
    let t = trap_code(0x1234, 0xC000, false, None);
    assert_eq!((t.code.clone(), t.call_off, t.halt_off, t.call_cost), (vec![0x20, 0x34, 0x12, 0x4C, 0x03, 0xC0], 0, 3, 6));
    let t = trap_code(0x1234, 0xC000, false, Some(&r));
    assert_eq!(t.code, vec![0xA2, 0xF0, 0x9A, 0xA9, 0xC5, 0x48, 0xA9, 0x11, 0xA2, 0x22, 0xA0, 0x33, 0x28, 0x20, 0x34, 0x12, 0x4C, 0x10, 0xC0]);
    assert_eq!((t.call_off, t.halt_off), (13, 16));
    // rti: Rücksprungrahmen (Rückkehradresse = Haltebefehl), Statusbyte doppelt (plp holt eines), dann jmp ziel
    let t = trap_code(0x1234, 0xC000, true, Some(&r));
    assert_eq!(t.code, vec![0xA2, 0xF0, 0x9A, 0xA9, 0xC0, 0x48, 0xA9, 0x17, 0x48, 0xA9, 0xC5, 0x48, 0x48, 0xA9, 0x11, 0xA2, 0x22, 0xA0, 0x33, 0x28, 0x4C, 0x34, 0x12, 0x4C, 0x17, 0xC0]);
    assert_eq!((t.call_off, t.halt_off, t.call_cost), (20, 23, 3));
    assert_eq!(default_trap("c64"), Some(0xC000));
    assert_eq!((default_trap("atari8"), default_trap("mega65"), default_trap("c128")), (Some(0x0600), Some(0x1F00), None));
}

#[test]
fn skips_with_reason_instead_of_guessing() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "I", "setup": { "fi": 0 }, "call": "inc_fb", "expect": [ "a == 1" ] } ]"#), full(), &RunOptions::default());
    assert_eq!(o[0].state, State::Skipped); assert!(o[0].detail.contains("I-Flag"), "{}", o[0].detail);
    // Falle im Programm
    let o = run(&e, r#"{ "target": "c64", "program": "p.prg", "trap": "$0820", "tests": [ { "name": "T", "call": "inc_fb", "expect": [ "a == 1" ] } ] }"#, full(), &RunOptions::default());
    assert_eq!(o[0].state, State::Skipped); assert!(o[0].detail.contains("liegt im Programm"), "{}", o[0].detail);
    // unbekanntes Ziel ohne Trap-Adresse
    let mut e2 = env();
    e2.pre.target = vc_target::load_target(&data(), "c128").unwrap();
    let o = run(&e2, r#"{ "target": "c128", "program": "p.prg", "tests": [ { "name": "A", "call": "inc_fb", "expect": [ "a == 1" ] } ] }"#, full(), &RunOptions::default());
    assert_eq!(o[0].state, State::Skipped); assert!(o[0].detail.contains("keine Trap-Adresse"), "{}", o[0].detail);
    // Emulator nicht startbar
    let tf = TestFile::parse(Path::new("x.vctest"), &file(r#"[ { "name": "N", "call": "inc_fb", "expect": [ "a == 1" ] }, { "name": "M", "call": "inc_fb", "expect": [ "a == 1" ] } ]"#)).unwrap();
    let o = run_tests(&tf, &e.pre, &RunOptions::default(), &mut || Err("nicht installiert".to_string()));
    assert!(o.iter().all(|x| x.state == State::Skipped && x.detail.contains("nicht installiert")), "{o:?}");
}

#[test]
fn malformed_tests_fail_with_a_message() {
    let e = env();
    let o = run(&e, &file(r#"[ { "name": "ohne call", "expect": [ "a == 1" ] }, { "name": "prüft nichts", "call": "inc_fb" }, { "name": "falsches Setup", "call": "inc_fb", "setup": { "q": 1 }, "expect": [ "a == 1" ] },
        { "name": "falscher Ausdruck", "call": "inc_fb", "expect": [ "a ==" ] }, { "name": "unbekanntes Ziel", "call": "gibtsnicht", "expect": [ "a == 1" ] } ]"#), full(), &RunOptions::default());
    assert!(o.iter().all(|x| x.is_error()), "{o:?}");
    assert!(o[0].detail.contains("call") && o[1].detail.contains("prüft nichts") && o[2].detail.contains("setup.q") && o[3].detail.contains("expect") && o[4].detail.contains("call"), "{o:?}");
    let filtered = run(&e, &file(r#"[ { "name": "eins", "call": "inc_fb", "expect": [ "a == 1" ] }, { "name": "zwei", "call": "inc_fb", "expect": [ "a == 1" ] } ]"#), full(), &RunOptions { filter: Some("zw".into()), ..Default::default() });
    assert_eq!(filtered.len(), 1); assert_eq!(filtered[0].name, "zwei");
}

#[test]
fn without_cycle_timer_the_beam_time_is_reported_as_such() {
    // atari8: keine Stoppuhr aus Zeitgebern, aber Zeile/Zyklus: Strahlzeit zwischen den beiden Halten, gekennzeichnet
    let e = env_target("atari8");
    assert_eq!((default_trap("atari8"), default_trap("mega65")), (Some(0x0600), Some(0x1F00)));
    let caps = TrapCaps { set_regs: false, cycles: true };
    let json = r#"{ "target": "atari8", "program": "p.xex", "tests": [
        { "name": "inc", "setup": { "mem": { "$fb": 41 } }, "call": "inc_fb", "expect": [ "[$fb] == 42", "cycles == 11" ], "cycles": { "max": 11 } },
        { "name": "zu teuer", "call": "inc_fb", "cycles": { "max": 5 } } ] }"#;
    let o = run(&e, json, caps, &RunOptions::default());
    assert_eq!((o[0].state, o[0].measured.as_deref()), (State::Passed, Some("11 Zyklen (Strahlzeit)")), "{:?}", o[0]);
    assert_eq!(o[1].state, State::Failed);
    assert!(o[1].detail.contains("11 Zyklen gemessen (Strahlzeit") && o[1].detail.contains("Speicherauffrischung"), "{}", o[1].detail);
    assert_eq!(o[1].measured.as_deref(), Some("11 Zyklen (Strahlzeit)"));
}

#[test]
fn beam_time_close_to_a_frame_is_not_measurable() {
    let mut e = env_target("atari8");
    // Unterprogramm mit fast einem Bild Dauer (35 568 Zyklen je Bild bei 114 × 312, hier das Schein-Maß 63 × 312 = 19 656)
    e.effects.insert(0x0810, effect(|_, _| Some(15_000)));
    let json = r#"{ "target": "atari8", "program": "p.xex", "tests": [ { "name": "lang", "call": "inc_fb", "expect": [ "cycles > 0" ] } ] }"#;
    let o = run(&e, json, TrapCaps { set_regs: false, cycles: true }, &RunOptions::default());
    assert_eq!(o[0].state, State::Skipped, "{:?}", o[0]);
    assert!(o[0].detail.contains("nahe an einem Bild"), "{}", o[0].detail);
}
