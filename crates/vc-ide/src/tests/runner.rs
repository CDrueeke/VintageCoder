// SPDX-License-Identifier: GPL-3.0-or-later
//! Läufer für den Trap-Modus (Testkonzept Stufe 2): Programm in den Emulator laden, in freiem RAM eine Falle anlegen
//! (`jsr ziel` + `jmp *`), Setup schreiben, bis zur Falle laufen lassen und die Erwartungen auswerten.
//!
//! Regeln aus dem Vorversuch T0:
//! - Das I-Flag ist beim Start immer gesetzt; sonst erreicht VICE die Falle nicht zuverlässig. Ein Test, der `fi: 0` verlangt, wird übersprungen.
//! - Kann der Adapter keine Flags setzen (`TrapCaps::set_regs` fehlt oder `force_prologue`), setzt ein Vorspann in der Falle sie
//!   (`ldx #sp / txs / lda #P / pha / lda #A / ldx #X / ldy #Y / plp`). Dann gibt es zwei Halte (am `jsr`, am `jmp *`).
//! - Zyklen: Stoppuhr aus den zusammengeschalteten CIA2-Zeitgebern (32 Bit, zählt Systemzyklen, C64: $DD04-$DD07), vor dem Aufruf gestartet und
//!   bei der Rückkehr gelesen, abzüglich des Aufrufbefehls (`jsr` 6, `jmp` 3). Die Strahlposition (Rasterzeile/Zyklus) prüft sie gegen
//!   (Zeitgeber mod Zyklen je Bild == Strahldifferenz); stimmen beide nicht überein (der erste Lauf nach dem Hochfahren ist um 2 Zyklen
//!   daneben), wird der Test wiederholt, danach gilt „nicht messbar“. Allein die Strahlposition wiederholt sich je Bild und taugt nicht.
//! - Ohne Zeitgeber (atari800): Strahlposition (Zeile/Zyklus) bei den beiden Halten am `jsr` und am `jmp *`, abzüglich des Aufrufbefehls; das Ergebnis
//!   heißt „Strahlzeit“ (mit Speicherauffrischung und DMA) und gilt nur unter einem Bild. Ohne Zyklenmessung (Xemu): „nicht messbar“, Test übersprungen.
//! - Zeitüberschreitung: der Test gilt als „hängt“, der Emulator wird beendet und für den nächsten Test neu gestartet.
use super::model::{Level, Outcome, RegSnap, TestCase, TestFile};
use super::static_checks::Prepared;
use crate::cond::{Ctx, Expr};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use vc_emu::{Backend, Regs};

/// Basisadresse der CIA, deren Zeitgeber A/B die Stoppuhr bilden; `None`: für das Ziel nicht vorgesehen
fn stopwatch_base(target: &str) -> Option<u16> { match target { "c64" => Some(0xDD00), _ => None } }
const FLAG_BITS: [(&str, u8); 6] = [("fn", 0x80), ("fv", 0x40), ("fd", 0x08), ("fi", 0x04), ("fz", 0x02), ("fc", 0x01)];

#[derive(Clone)]
pub struct RunOptions {
    /// Hinweise (Zyklenbudget ohne eigene Angabe) werden zu Fehlern
    pub strict: bool,
    /// Wartezeit nach dem Emulatorstart (Hochfahren des Rechners), bevor Speicher geschrieben wird
    pub boot_wait: Duration,
    /// Vorspann statt direktem Setzen der Register verwenden (auch wenn der Adapter es könnte)
    pub force_prologue: bool,
    /// Nur Tests, deren Name diesen Text enthält
    pub filter: Option<String>,
    /// Nur Tests mit genau diesen Namen (Wiederholung fehlgeschlagener Tests)
    pub only_names: Option<Vec<String>>,
    /// Abbruch von außen (wird zwischen den Tests geprüft); die übrigen Tests gelten als „übersprungen“
    pub cancel: Option<Arc<AtomicBool>>,
    /// Wird nach jedem Ergebnis aufgerufen (Fortschritt für die Oberfläche)
    pub on_outcome: Option<Arc<dyn Fn(&Outcome) + Send + Sync>>,
    /// Postfach-Modus: das Postfach in Abständen lesen, auch wenn ein Haltepunkt am Programmende möglich wäre
    pub poll: bool,
}

impl Default for RunOptions { fn default() -> Self { RunOptions { strict: false, boot_wait: Duration::from_secs(3), force_prologue: false, filter: None, only_names: None, cancel: None, on_outcome: None, poll: false } } }

/// Startet einen Emulator mit dem Zielrechner, ohne Programm (das schreibt der Läufer in den Speicher)
pub type Launcher<'a> = dyn FnMut() -> Result<Box<dyn Backend>, String> + 'a;

/// Standardadresse der Falle je Ziel (freies RAM, das Testprogramme nicht belegen); `None`: im Test mit `"trap"` angeben
/// atari8: Seite 6 (`$0600`, frei für Anwenderprogramme); mega65: `$1F00` (unterhalb des BASIC-Textes, im Versuch mit Xemu beschreibbar)
pub fn default_trap(target: &str) -> Option<u16> { match target { "c64" => Some(0xC000), "atari8" => Some(0x0600), "mega65" => Some(0x1F00), _ => None } }

/// Falle im freien RAM
#[derive(Debug, Clone, PartialEq)]
pub struct Trap {
    pub code: Vec<u8>,
    /// Offset des Aufrufbefehls (`jsr ziel` bzw. bei `rti` `jmp ziel`)
    pub call_off: u16,
    /// Offset des `jmp *`, an dem der Läufer hält
    pub halt_off: u16,
    /// Zyklen des Aufrufbefehls (`jsr` 6, `jmp` 3): gehören nicht zum Unterprogramm
    pub call_cost: u32,
}

/// Bytes der Falle. Mit `prologue` setzt ein Vorspann Stapelzeiger, Flags und Register (für Adapter ohne Flag-Zugriff); bei `rti`
/// ist der Vorspann Pflicht, er legt zusätzlich den Rücksprungrahmen (Rückkehradresse = `jmp *`, Statusbyte) auf den Stapel.
pub fn trap_code(call: u16, base: u16, rti: bool, prologue: Option<&Regs>) -> Trap {
    let mut c: Vec<u8> = vec![];
    if rti {
        let r = prologue.expect("rti braucht den Vorspann");
        let halt = base.wrapping_add(23);
        c.extend([0xA2, r.sp, 0x9A, 0xA9, (halt >> 8) as u8, 0x48, 0xA9, halt as u8, 0x48, 0xA9, r.flags, 0x48, 0x48, 0xA9, r.a, 0xA2, r.x, 0xA0, r.y, 0x28]);
        let call_off = c.len() as u16;
        c.extend([0x4C, call as u8, (call >> 8) as u8]);
        let halt_off = c.len() as u16;
        c.extend([0x4C, halt as u8, (halt >> 8) as u8]);
        return Trap { code: c, call_off, halt_off, call_cost: 3 };
    }
    if let Some(r) = prologue { c.extend([0xA2, r.sp, 0x9A, 0xA9, r.flags, 0x48, 0xA9, r.a, 0xA2, r.x, 0xA0, r.y, 0x28]); }
    let call_off = c.len() as u16;
    c.extend([0x20, call as u8, (call >> 8) as u8]);
    let halt_off = c.len() as u16;
    let at = base.wrapping_add(halt_off);
    c.extend([0x4C, at as u8, (at >> 8) as u8]);
    Trap { code: c, call_off, halt_off, call_cost: 6 }
}

struct DynCtx<'a> { regs: &'a Regs, backend: &'a mut dyn Backend, cycles: &'a Result<i64, String> }

impl Ctx for DynCtx<'_> {
    fn reg(&self, name: &str) -> Option<i64> {
        let r = self.regs;
        if name == "cycles" { return self.cycles.as_ref().ok().copied(); }
        if let Some((_, bit)) = FLAG_BITS.iter().find(|(n, _)| *n == name) { return Some((r.flags & bit != 0) as i64); }
        Some(match name { "a" => r.a as i64, "x" => r.x as i64, "y" => r.y as i64, "sp" | "s" => r.sp as i64, "pc" => r.pc as i64, "p" | "flags" => r.flags as i64, "fb" => (r.flags >> 4 & 1) as i64,
            other => return r.extra.iter().find(|(n, _)| n.eq_ignore_ascii_case(other)).map(|(_, v)| *v as i64) })
    }
    fn peek(&mut self, addr: u16) -> Result<u8, String> { self.backend.read_mem(addr, 1)?.first().copied().ok_or_else(|| "Speicher nicht lesbar".to_string()) }
}

fn reg_line(r: &Regs) -> String {
    let f: String = "NV-BDIZC".chars().enumerate().map(|(i, c)| if r.flags & (0x80 >> i) != 0 { c } else { c.to_ascii_lowercase() }).collect();
    format!("A=${:02X} X=${:02X} Y=${:02X} SP=${:02X} P={f} (${:02X}) PC=${:04X}", r.a, r.x, r.y, r.sp, r.flags, r.pc)
}

fn num(src: &str, syms: &HashMap<String, i64>) -> Result<i64, String> {
    struct NoCtx;
    impl Ctx for NoCtx { fn reg(&self, _: &str) -> Option<i64> { None } fn peek(&mut self, _: u16) -> Result<u8, String> { Err("Speicher ist hier nicht lesbar".into()) } }
    Expr::parse(src, syms).map_err(|e| format!("„{src}“: {e}"))?.eval(&mut NoCtx).map_err(|e| format!("„{src}“: {e}"))
}
fn byte(src: &str, syms: &HashMap<String, i64>, what: &str) -> Result<u8, String> {
    let v = num(src, syms)?;
    if !(-128..=255).contains(&v) { return Err(format!("{what}: {v} passt nicht in ein Byte")); }
    Ok(v as u8)
}

/// Adresse der Falle und Bereiche des Programmabbilds; `Err`: Grund, warum sich die Tests hier nicht ausführen lassen
pub fn resolve_trap(tf: &TestFile, pre: &Prepared) -> Result<(u16, Vec<(u16, Vec<u8>)>), String> {
    let image = pre.image.as_ref().map_err(|e| format!("Programmabbild nicht lesbar: {e}"))?;
    let syms = &pre.dbg.symbols;
    let trap = match tf.trap.as_deref() {
        Some(src) => match num(src, syms) { Ok(v) if (0..=0xFFFF).contains(&v) => v as u16, Ok(v) => return Err(format!("trap: Adresse {v} außerhalb 0..65535")), Err(e) => return Err(format!("trap: {e}")) },
        None => default_trap(&tf.target).ok_or_else(|| format!("für das Ziel „{}“ ist keine Trap-Adresse bekannt: im Test \"trap\" angeben", tf.target))?,
    };
    let runs = image.runs();
    // Die Falle (höchstens 27 Bytes) darf nicht im Programm liegen
    if runs.iter().any(|(s, v)| (*s as u32) < trap as u32 + 27 && trap as u32 <= *s as u32 + v.len() as u32) {
        return Err(format!("die Falle bei ${trap:04X} liegt im Programm: im Test \"trap\" eine freie Adresse angeben"));
    }
    Ok((trap, runs))
}

/// Führt alle Tests (`tests` der Testdatei) im Emulator aus. Der Emulator wird nur nach Bedarf gestartet und am Ende beendet.
pub fn run_tests(tf: &TestFile, pre: &Prepared, ro: &RunOptions, launch: &mut Launcher) -> Vec<Outcome> {
    let mut out = vec![];
    let mut cases: Vec<TestCase> = vec![];
    for (i, t) in tf.tests.iter().enumerate() {
        match TestCase::parse(i, t) { Ok(c) => if ro.filter.as_ref().map_or(true, |f| c.name.contains(f.as_str())) && ro.only_names.as_ref().map_or(true, |n| n.contains(&c.name)) { cases.push(c) }, Err(e) => out.push(Outcome::failed(t.get("name").and_then(|n| n.as_str()).unwrap_or("Test"), Level::Error, e)) }
    }
    if cases.is_empty() { return out; }
    let skip_all = |out: &mut Vec<Outcome>, cases: &[TestCase], why: String| for c in cases { out.push(Outcome::skipped(c.name.clone(), why.clone())); };
    let (trap, runs) = match resolve_trap(tf, pre) { Ok(x) => x, Err(why) => { skip_all(&mut out, &cases, why); return out; } };
    let mut emu: Option<Box<dyn Backend>> = None;
    let mut broken: Option<String> = None;
    let emit = |out: &mut Vec<Outcome>, o: Outcome| { if let Some(f) = &ro.on_outcome { f(&o); } out.push(o); };
    for c in &cases {
        if ro.cancel.as_ref().map_or(false, |f| f.load(Ordering::SeqCst)) { emit(&mut out, Outcome::skipped(c.name.clone(), "abgebrochen")); continue; }
        if let Some(why) = &broken { emit(&mut out, Outcome::skipped(c.name.clone(), why.clone())); continue; }
        if tf.isolate { if let Some(mut e) = emu.take() { e.stop(); } }
        if emu.is_none() {
            match launch() {
                Ok(e) => { emu = Some(e); std::thread::sleep(ro.boot_wait); }
                Err(e) => { let why = format!("Emulator nicht startbar: {e}"); emit(&mut out, Outcome::skipped(c.name.clone(), why.clone())); broken = Some(why); continue; }
            }
        }
        let e = emu.as_mut().unwrap();
        let (o, restart) = run_one(e.as_mut(), c, pre, &runs, trap, ro);
        emit(&mut out, o);
        if restart { if let Some(mut e) = emu.take() { e.stop(); } }
    }
    if let Some(mut e) = emu.take() { e.stop(); }
    out
}

/// Ein Test; zweiter Wert: Emulator ist in unbestimmtem Zustand und muss neu gestartet werden
fn run_one(emu: &mut dyn Backend, c: &TestCase, pre: &Prepared, runs: &[(u16, Vec<u8>)], trap: u16, ro: &RunOptions) -> (Outcome, bool) {
    let syms = &pre.dbg.symbols;
    let name = c.name.clone();
    let fail = |e: String| (Outcome::failed(name.clone(), Level::Error, e), false);
    let target = match num(&c.call, syms) { Ok(v) if (0..=0xFFFF).contains(&v) => v as u16, Ok(v) => return fail(format!("call: Adresse {v} außerhalb 0..65535")), Err(e) => return fail(format!("call: {e}")) };
    let loc = pre.dbg.lines.iter().find(|l| l.addr == target && l.cycles.is_some()).map(|l| (l.file.clone(), l.line));
    let with_loc = |o: Outcome| match &loc { Some((f, l)) => o.at(f, *l), None => o };
    // Register vorbereiten; das I-Flag bleibt gesetzt
    let mut r = Regs { sp: 0xFF, flags: 0x24, ..Default::default() };
    for (k, src) in &c.regs {
        match byte(src, syms, k) { Ok(v) => match k.as_str() { "a" => r.a = v, "x" => r.x = v, "y" => r.y = v, "sp" => r.sp = v, _ => r.flags = v | 0x24 }, Err(e) => return fail(e) }
    }
    for (k, on) in &c.flags {
        let bit = FLAG_BITS.iter().find(|(n, _)| n == k).map(|(_, b)| *b).unwrap_or(0);
        if bit == 0x04 && !on { return (Outcome::skipped(name, "der Test verlangt ein gelöschtes I-Flag: im Trap-Modus laufen Tests mit gesetztem I-Flag (sonst erreicht VICE die Falle nicht zuverlässig)"), false); }
        if *on { r.flags |= bit } else { r.flags &= !bit }
    }
    let mut attempt = 0;
    // Zyklen ohne Zeitgeber: Strahlzeit zwischen den beiden Halten (atari800); sie enthält Speicherauffrischung und DMA
    let mut strahl = false;
    let cycles: Result<i64, String> = loop {
        // Speicher: Programm neu schreiben (stellt Code und Daten her), dann das Setup des Tests
        let io = (|| -> Result<(), String> {
            for (s, v) in runs { emu.write_mem(*s, v)?; }
            for (addr, vals) in &c.mem {
                let a = num(addr, syms).map_err(|e| format!("setup.mem {e}"))?;
                if !(0..=0xFFFF).contains(&a) { return Err(format!("setup.mem: Adresse {a} außerhalb 0..65535")); }
                let bytes = vals.iter().map(|v| byte(v, syms, "setup.mem")).collect::<Result<Vec<_>, _>>()?;
                emu.write_mem(a as u16, &bytes)?;
            }
            Ok(())
        })();
        if let Err(e) = io { return fail(e); }
        let caps = emu.trap_caps();
        let prologue = ro.force_prologue || !caps.set_regs || c.rti;
        let t = trap_code(target, trap, c.rti, if prologue { Some(&r) } else { None });
        let (code, call_at, jmp_at) = (&t.code, trap.wrapping_add(t.call_off), trap.wrapping_add(t.halt_off));
        let timeout = Duration::from_millis(c.timeout_ms);
        let want_cycles = c.cycles_max.is_some() || c.expect.iter().any(|e| e.contains("cycles"));
        let hang = |what: &str| (with_loc(Outcome::timed_out(name.clone(), format!("{what} nach {} ms nicht erreicht: das Unterprogramm kehrt nicht zurück; der Emulator wird neu gestartet", c.timeout_ms))), true);
        if let Err(e) = emu.write_mem(trap, code) { return fail(e); }
        let start_regs = if prologue { Regs { pc: trap, sp: 0xFF, flags: 0x24, ..Default::default() } } else { Regs { pc: trap, ..r.clone() } };
        if let Err(e) = emu.set_regs(&start_regs) { return fail(e); }
        if prologue { match emu.run_until(call_at, timeout) { Ok(true) => {} Ok(false) => return hang("der Vorspann der Falle"), Err(e) => return fail(e) } }
            let mut cycles: Result<i64, String> = Err("nicht angefordert".into());
        let sw = if want_cycles { stopwatch_base(&pre.target.id).filter(|_| caps.cycles) } else { None };
        let start_pos = match sw {
            Some(base) => {
                // CIA-Zeitgeber A und B (B zählt die Unterläufe von A) auf $FFFFFFFF laden und gemeinsam starten; Unterbrechungen der CIA aus
                let go = (|| -> Result<(), String> {
                    for (a, b) in [(base + 0x0D, 0x7Fu8), (base + 0x0E, 0x00), (base + 0x0F, 0x00)] { emu.write_mem(a, &[b])?; }
                    emu.write_mem(base + 4, &[0xFF; 4])?;
                    emu.write_mem(base + 0x0E, &[0x10])?; emu.write_mem(base + 0x0F, &[0x50])?;        // laden (B zählt A-Unterläufe)
                    emu.write_mem(base + 0x0E, &[0x01])?; emu.write_mem(base + 0x0F, &[0x41])?;        // starten
                    Ok(())
                })();
                Some(go.and_then(|_| emu.cycles()))
            }
            None if want_cycles && caps.cycles => { strahl = true; Some(emu.cycles()) }
            None if want_cycles => Some(Err("dieser Emulator misst keine Zyklen".to_string())),
            None => None,
        };
        match emu.run_until(jmp_at, timeout) { Ok(true) => {} Ok(false) => return hang("die Rückkehr"), Err(e) => return fail(e) }
        if let Some(start) = start_pos {
            cycles = match start.and_then(|s| emu.cycles().map(|e| (s, e))) {
                Err(e) => Err(format!("nicht messbar: {e}")),
                Ok((s, e)) if strahl => {
                    // Nur die Strahlposition: gültig, solange weniger als ein Bild vergeht (sie wiederholt sich je Bild)
                    let beam = e.since(&s);
                    if beam * 4 >= e.frame() * 3 { Err(format!("nicht messbar: Strahlzeit {beam} liegt nahe an einem Bild ({} Zyklen), die Position wiederholt sich", e.frame())) }
                    else if beam < t.call_cost as u64 { Err(format!("nicht messbar: {beam} Zyklen sind weniger als der Aufrufbefehl")) }
                    else { Ok(beam as i64 - t.call_cost as i64) }
                }
                Ok((s, e)) => {
                    let beam = e.since(&s);
                    match emu.read_mem(sw.unwrap() + 4, 4) {
                        Err(er) => Err(format!("nicht messbar: {er}")),
                        Ok(tm) if tm.len() == 4 => {
                            let timer = 0xFFFF_FFFFu64 - u32::from_le_bytes([tm[0], tm[1], tm[2], tm[3]]) as u64;
                            if timer % e.frame() != beam { Err(format!("nicht messbar: Zeitgeber ({timer} Zyklen) und Strahlposition ({beam} mod {}) stimmen nicht überein (Strahlposition)", e.frame())) }
                            else if timer < t.call_cost as u64 { Err(format!("nicht messbar: {timer} Zyklen sind weniger als der Aufrufbefehl")) }
                            else { Ok(timer as i64 - t.call_cost as i64) }
                        }
                        Ok(_) => Err("nicht messbar: Zeitgeber nicht lesbar".into()),
                    }
                }
            };
        }
        // Die Wartezeit bis zum Halt schwankt (VICE bedient den Monitor bis zu ein Bild später): bei „zu lange“ denselben Test wiederholen,
            // die gemessene Differenz ist ja jedes Mal dieselbe, nur die Gültigkeitsprüfung kann beim nächsten Versuch bestehen
            if matches!(&cycles, Err(e) if e.contains("Strahlposition")) && attempt < 4 { attempt += 1; continue; }
            break cycles;
        };
    let regs = match emu.registers() { Ok(x) => x, Err(e) => return fail(e) };
    let snap = RegSnap { pc: regs.pc, a: regs.a, x: regs.x, y: regs.y, sp: regs.sp, flags: regs.flags };
    let with_loc = |o: Outcome| with_loc(o).with_regs(snap);
    let mut first_fail: Option<String> = None;
    let mut unmeasurable: Option<String> = None;
    for src in &c.expect {
        let expr = match Expr::parse(src, syms) { Ok(x) => x, Err(e) => return fail(format!("expect „{src}“: {e}")) };
        let mut ctx = DynCtx { regs: &regs, backend: emu, cycles: &cycles };
        match expr.holds(&mut ctx) {
            Ok(true) => {}
            Ok(false) => {
                let mut d = format!("Erwartung „{src}“ ist nicht erfüllt");
                if let Some((l, op, rr)) = expr.comparison() {
                    let (lv, rv) = (l.eval(&mut ctx), rr.eval(&mut ctx));
                    if let (Ok(lv), Ok(rv)) = (lv, rv) { d.push_str(&format!(": linke Seite = {lv} (${:X}), rechte Seite = {rv} (${:X}), verlangt wurde {op}", lv & 0xFFFF_FFFF, rv & 0xFFFF_FFFF)); }
                }
                first_fail.get_or_insert(d);
            }
            Err(e) => match (&cycles, src.contains("cycles")) {
                (Err(why), true) => { unmeasurable.get_or_insert(format!("„{src}“: Zyklen {why}")); }
                _ => { first_fail.get_or_insert(format!("Erwartung „{src}“ nicht auswertbar: {e}")); }
            },
        }
    }
    let state = format!("Zustand bei der Rückkehr: {}", reg_line(&regs));
    if let Some(d) = first_fail { return (with_loc(Outcome::failed(name, Level::Error, format!("{d}. {state}"))), false); }
    // Zyklenbudget
    let mut budget_note: Option<Outcome> = None;
    if let Some(m) = &c.cycles_max {
        let limit = match num(m, syms) { Ok(v) => v, Err(e) => return fail(format!("cycles.max: {e}")) };
        match &cycles {
            Ok(n) if *n > limit => {
                let lvl = if ro.strict { Level::Error } else { c.level.unwrap_or(Level::Hint) };
                let sfx = if strahl { " (Strahlzeit: mit Speicherauffrischung und DMA, nie unter den CPU-Zyklen)" } else { "" };
                budget_note = Some(with_loc(Outcome::failed(name.clone(), lvl, format!("{n} Zyklen gemessen{sfx}, erlaubt: höchstens {limit} – {} zu viel", n - limit)).with_measured(format!("{n} Zyklen{}", if strahl { " (Strahlzeit)" } else { "" }))));
            }
            Ok(_) => {}
            Err(why) => { unmeasurable.get_or_insert(format!("Zyklenbudget: Zyklen {why}")); }
        }
    }
    if let Some(o) = budget_note { return (o, false); }
    if let Some(why) = unmeasurable { return (with_loc(Outcome::skipped(name, why)), false); }
    let mut ok = Outcome::passed(name, cycles.as_ref().ok().map(|n| format!("{n} Zyklen{}", if strahl { " (Strahlzeit)" } else { "" })));
    ok.detail = String::new();
    (with_loc(ok), false)
}

/// Wo die Falle für „Diesen Test debuggen“ liegt
#[derive(Debug, Clone, PartialEq)]
pub struct ArmInfo {
    pub trap: u16,
    /// Adresse des Aufrufbefehls und des abschließenden `jmp *`
    pub call: u16,
    pub halt: u16,
    /// Adresse des Unterprogramms
    pub target: u16,
    pub prologue: bool,
}

/// Bereitet einen Test im (laufenden, angehaltenen) Emulator vor, ohne ihn auszuführen: Programm und Setup schreiben, Falle anlegen,
/// PC auf die Falle setzen. Danach übernimmt der Debugger (Einzelschritt, Haltepunkt am `jmp *` der Falle).
pub fn arm_test(emu: &mut dyn Backend, tf: &TestFile, pre: &Prepared, test: &str, ro: &RunOptions) -> Result<ArmInfo, String> {
    let (i, raw) = tf.tests.iter().enumerate().find(|(_, t)| t.get("name").and_then(|n| n.as_str()) == Some(test)).ok_or_else(|| format!("Test „{test}“ nicht gefunden"))?;
    let c = TestCase::parse(i, raw)?;
    let (trap, runs) = resolve_trap(tf, pre)?;
    let syms = &pre.dbg.symbols;
    let target = num(&c.call, syms).map_err(|e| format!("call: {e}"))?;
    if !(0..=0xFFFF).contains(&target) { return Err(format!("call: Adresse {target} außerhalb 0..65535")); }
    let mut r = Regs { sp: 0xFF, flags: 0x24, ..Default::default() };
    for (k, src) in &c.regs { let v = byte(src, syms, k)?; match k.as_str() { "a" => r.a = v, "x" => r.x = v, "y" => r.y = v, "sp" => r.sp = v, _ => r.flags = v | 0x24 } }
    for (k, on) in &c.flags {
        let bit = FLAG_BITS.iter().find(|(n, _)| n == k).map(|(_, b)| *b).unwrap_or(0);
        if *on { r.flags |= bit } else { r.flags &= !bit }
    }
    for (s, v) in &runs { emu.write_mem(*s, v)?; }
    for (addr, vals) in &c.mem {
        let a = num(addr, syms).map_err(|e| format!("setup.mem {e}"))?;
        if !(0..=0xFFFF).contains(&a) { return Err(format!("setup.mem: Adresse {a} außerhalb 0..65535")); }
        let bytes = vals.iter().map(|v| byte(v, syms, "setup.mem")).collect::<Result<Vec<_>, _>>()?;
        emu.write_mem(a as u16, &bytes)?;
    }
    let caps = emu.trap_caps();
    let prologue = ro.force_prologue || !caps.set_regs || c.rti;
    let t = trap_code(target as u16, trap, c.rti, if prologue { Some(&r) } else { None });
    emu.write_mem(trap, &t.code)?;
    let start = if prologue { Regs { pc: trap, sp: 0xFF, flags: 0x24, ..Default::default() } } else { Regs { pc: trap, ..r } };
    emu.set_regs(&start)?;
    Ok(ArmInfo { trap, call: trap.wrapping_add(t.call_off), halt: trap.wrapping_add(t.halt_off), target: target as u16, prologue })
}
