// SPDX-License-Identifier: GPL-3.0-or-later
//! Postfach-Modus (Testkonzept Stufe 3): das Testprogramm prüft sich selbst (Makros aus `test.inc`) und schreibt das Ergebnis in ein
//! Postfach im RAM; der Läufer startet das Programm, wartet auf das Ende und liest nur Speicher.
//!
//! Warten: Hat der Adapter Haltepunkte und steht im Programm die Endlosschleife von `test_end` (`jmp *`), hält der Läufer dort an
//! (schnell, auch bei atari800, das sich nicht anhalten lässt). Sonst, oder mit `RunOptions::poll`, liest er das Postfach in Abständen
//! von 100 ms (VICE hält bei jedem Befehl an und läuft danach weiter) bis zum Zeitlimit.
//!
//! Postfach (8 Byte, Adresse `testMailbox` der Zieldaten): +0 Status (1 läuft, 2 bestanden, 3 fehlgeschlagen), +1 Anzahl der Prüfungen,
//! +2 Nummer der ersten fehlgeschlagenen (0 = keine), +3 Ist-Wert, +4 Sollwert, +5 Art (1 A, 2 X, 3 Y, 4 Speicher).
use super::model::{Level, MailboxCase, Outcome, RegSnap, TestFile};
use super::runner::RunOptions;
use super::static_checks::Prepared;
use crate::cond::{Ctx, Expr};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use vc_emu::Backend;

pub const MAILBOX_LEN: usize = 8;
pub const STATUS_RUNNING: u8 = 1;
pub const STATUS_PASS: u8 = 2;
pub const STATUS_FAIL: u8 = 3;

/// Startet einen Emulator mit dem Testprogramm (autostart bzw. Laden), ohne es anzuhalten
pub type ProgramLauncher<'a> = dyn FnMut(&Path) -> Result<Box<dyn Backend>, String> + 'a;

/// Inhalt des Postfachs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mailbox { pub status: u8, pub count: u8, pub first_failed: u8, pub actual: u8, pub expected: u8, pub kind: u8 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass { checks: u8 },
    Fail { number: u8, of: u8, kind: u8, actual: u8, expected: u8 },
    /// Das Programm ist durchgelaufen, hat aber keine Prüfung ausgeführt
    NoChecks,
    /// Status passt nicht zu einem Ende (läuft noch, nie gestartet oder das Postfach wurde überschrieben)
    Unfinished(u8),
}

impl Mailbox {
    pub fn parse(b: &[u8]) -> Option<Mailbox> { if b.len() < 6 { None } else { Some(Mailbox { status: b[0], count: b[1], first_failed: b[2], actual: b[3], expected: b[4], kind: b[5] }) } }
    pub fn verdict(&self) -> Verdict {
        match self.status {
            STATUS_PASS if self.count == 0 => Verdict::NoChecks,
            STATUS_PASS => Verdict::Pass { checks: self.count },
            STATUS_FAIL => Verdict::Fail { number: self.first_failed, of: self.count, kind: self.kind, actual: self.actual, expected: self.expected },
            s => Verdict::Unfinished(s),
        }
    }
}

fn kind_name(k: u8) -> &'static str { match k { 1 => "assert_a_eq (A)", 2 => "assert_x_eq (X)", 3 => "assert_y_eq (Y)", 4 => "assert_mem_eq (Speicher)", _ => "unbekannte Art" } }

/// Adresse der Endlosschleife von `test_end`: `jmp` auf die eigene Adresse in einer Befehlszeile des Programms (die letzte)
pub fn find_done_address(pre: &Prepared) -> Option<u16> {
    let img = pre.image.as_ref().ok()?;
    pre.dbg.lines.iter().filter(|l| l.cycles.is_some() && l.len == 3 && img.get(l.addr) == Some(0x4C)
        && img.get(l.addr.wrapping_add(1)).zip(img.get(l.addr.wrapping_add(2))).map_or(false, |(lo, hi)| u16::from_le_bytes([lo, hi]) == l.addr)).map(|l| l.addr).last()
}

struct MCtx<'a> { regs: Option<RegSnap>, backend: &'a mut dyn Backend }
impl Ctx for MCtx<'_> {
    fn reg(&self, name: &str) -> Option<i64> {
        let r = self.regs?;
        Some(match name { "a" => r.a as i64, "x" => r.x as i64, "y" => r.y as i64, "sp" | "s" => r.sp as i64, "pc" => r.pc as i64, "p" | "flags" => r.flags as i64,
            "fn" => (r.flags >> 7 & 1) as i64, "fv" => (r.flags >> 6 & 1) as i64, "fb" => (r.flags >> 4 & 1) as i64, "fd" => (r.flags >> 3 & 1) as i64, "fi" => (r.flags >> 2 & 1) as i64, "fz" => (r.flags >> 1 & 1) as i64, "fc" => (r.flags & 1) as i64, _ => return None })
    }
    fn peek(&mut self, addr: u16) -> Result<u8, String> { self.backend.read_mem(addr, 1)?.first().copied().ok_or_else(|| "Speicher nicht lesbar".to_string()) }
}

/// Wartet auf das Ende des Testprogramms und liefert das Postfach; `None`: Zeitlimit (dann das letzte gelesene Postfach, falls lesbar)
fn wait_for_end(emu: &mut dyn Backend, mb: u16, done: Option<u16>, timeout: Duration, poll: bool) -> Result<(bool, Option<Vec<u8>>), String> {
    if !poll && emu.caps().breakpoints {
        if let Some(a) = done {
            let reached = emu.run_until(a, timeout)?;
            // nach dem Zeitlimit kann das Postfach noch lesbar sein (VICE); atari800 läuft dann und antwortet nicht
            return Ok((reached, emu.read_mem(mb, MAILBOX_LEN).ok()));
        }
    }
    let end = Instant::now() + timeout;
    let mut last = None;
    // Hat die Hardware ein Debug-Register (Ultimate 64, `$D7FF`), das `test_end` beschreibt, ist das der schnelle Weg: ein Byte statt des Postfachs.
    // Es gilt nur als Hinweis: zählt das Postfach nicht als fertig (veralteter Wert), wird weitergewartet; geht es nicht (Fehler), bleibt es beim Postfach.
    let mut use_reg = !poll;
    let mut round = 0u32;
    while Instant::now() < end {
        std::thread::sleep(Duration::from_millis(if use_reg { 20 } else { 100 }));
        round += 1;
        if use_reg {
            match emu.debug_reg() {
                None | Some(Err(_)) => use_reg = false,
                // Status noch nicht gemeldet: nur gelegentlich (jede Sekunde) das Postfach prüfen, falls das Programm das Register nie beschreibt
                Some(Ok(v)) if v != STATUS_PASS && v != STATUS_FAIL && round % 50 != 0 => continue,
                Some(Ok(_)) => {}
            }
        }
        let b = emu.read_mem(mb, MAILBOX_LEN)?;           // hält VICE an
        let fin = matches!(b.first(), Some(&STATUS_PASS) | Some(&STATUS_FAIL));
        last = Some(b);
        if fin { return Ok((true, last)); }
        if emu.state() == vc_emu::RunState::Stopped { let _ = emu.cont(); }      // VICE hält bei jedem Befehl an; die Hardware läuft beim Lesen weiter
    }
    Ok((false, last))
}

/// Führt die Postfach-Läufe der Testdatei aus: je Eintrag in `tests` ein frisch gestarteter Emulator mit dem Programm
pub fn run_mailbox(tf: &TestFile, pre: &Prepared, ro: &RunOptions, launch: &mut ProgramLauncher) -> Vec<Outcome> {
    let mut out = vec![];
    let mut cases = vec![];
    for (i, t) in tf.tests.iter().enumerate() {
        match MailboxCase::parse(i, t) {
            Ok(c) => if ro.filter.as_ref().map_or(true, |f| c.name.contains(f.as_str())) && ro.only_names.as_ref().map_or(true, |n| n.contains(&c.name)) { cases.push(c) },
            Err(e) => out.push(Outcome::failed(t.get("name").and_then(|n| n.as_str()).unwrap_or("Test"), Level::Error, e)),
        }
    }
    let emit = |out: &mut Vec<Outcome>, o: Outcome| { if let Some(f) = &ro.on_outcome { f(&o); } out.push(o); };
    let skip_all = |out: &mut Vec<Outcome>, cases: &[MailboxCase], why: String| for c in cases { emit(out, Outcome::skipped(c.name.clone(), why.clone())); };
    let Some(mb) = pre.target.test_mailbox else { skip_all(&mut out, &cases, format!("für das Ziel „{}“ ist in den Zieldaten kein Postfach (testMailbox) festgelegt", tf.target)); return out; };
    let done = find_done_address(pre);
    // Eine Quellstelle gibt es nicht: die Endlosschleife steht im Makro `test_end` (test.inc), nicht im Testprogramm
    let with_loc = |o: Outcome| o;
    let mut broken: Option<String> = None;
    for c in &cases {
        if ro.cancel.as_ref().map_or(false, |f| f.load(Ordering::SeqCst)) { emit(&mut out, Outcome::skipped(c.name.clone(), "abgebrochen")); continue; }
        if let Some(why) = &broken { emit(&mut out, Outcome::skipped(c.name.clone(), why.clone())); continue; }
        let mut emu = match launch(&pre.program) {
            Ok(e) => e,
            Err(e) => { let why = format!("Emulator nicht startbar: {e}"); emit(&mut out, Outcome::skipped(c.name.clone(), why.clone())); broken = Some(why); continue; }
        };
        let timeout = Duration::from_millis(c.timeout_ms);
        let waited = wait_for_end(emu.as_mut(), mb, done, timeout, ro.poll);
        let o = match waited {
            Err(e) => with_loc(Outcome::failed(c.name.clone(), Level::Error, format!("Postfach nicht lesbar: {e}"))),
            Ok((false, last)) => {
                let st = last.as_deref().and_then(Mailbox::parse).map_or("unbekannt".to_string(), |m| format!("Status {}, {} Prüfung(en) bisher", m.status, m.count));
                let hint = if last.as_deref().and_then(|b| b.first()).map_or(false, |s| *s == 0) { " (Status 0: test_begin wurde nie ausgeführt, das Programm startet nicht oder hängt vorher)" } else { "" };
                with_loc(Outcome::timed_out(c.name.clone(), format!("das Testprogramm endete nach {} ms nicht: {st}{hint}; der Emulator wird beendet", c.timeout_ms)))
            }
            Ok((true, bytes)) => {
                let m = bytes.as_deref().and_then(Mailbox::parse);
                match m.map(|m| m.verdict()) {
                    None => with_loc(Outcome::failed(c.name.clone(), Level::Error, "Postfach unvollständig gelesen")),
                    Some(Verdict::Fail { number, of, kind, actual, expected }) => with_loc(Outcome::failed(c.name.clone(), Level::Error, format!("Prüfung Nr. {number} von {of} fehlgeschlagen ({}): Ist-Wert {actual} (${actual:02X}), erwartet {expected} (${expected:02X})", kind_name(kind)))),
                    Some(Verdict::NoChecks) => with_loc(Outcome::failed(c.name.clone(), Level::Error, "das Programm lief durch, hat aber keine Prüfung ausgeführt (assert_… fehlen)")),
                    Some(Verdict::Unfinished(s)) => with_loc(Outcome::failed(c.name.clone(), Level::Error, format!("Postfach zeigt nach dem Ende Status ${s:02X}: das Programm hat das Postfach überschrieben oder test_end nicht erreicht"))),
                    Some(Verdict::Pass { checks }) => {
                        // zusätzliche Erwartungen über Speicher und Register nach dem Ende
                        let regs = emu.registers().ok().map(|r| RegSnap { pc: r.pc, a: r.a, x: r.x, y: r.y, sp: r.sp, flags: r.flags });
                        let mut bad: Option<String> = None;
                        for src in &c.expect {
                            let r = Expr::parse(src, &pre.dbg.symbols).and_then(|e| e.holds(&mut MCtx { regs, backend: emu.as_mut() }));
                            match r { Ok(true) => {} Ok(false) => { bad.get_or_insert(format!("Erwartung „{src}“ ist nach dem Ende nicht erfüllt")); } Err(e) => { bad.get_or_insert(format!("Erwartung „{src}“ nicht auswertbar: {e}")); } }
                        }
                        match bad {
                            Some(b) => with_loc(Outcome::failed(c.name.clone(), Level::Error, format!("alle {checks} Prüfungen des Programms bestanden, aber {b}"))),
                            None => with_loc(Outcome::passed(c.name.clone(), Some(format!("{checks} Prüfung{}", if checks == 1 { "" } else { "en" })))),
                        }
                    }
                }
            }
        };
        emu.stop();
        emit(&mut out, o);
    }
    out
}
