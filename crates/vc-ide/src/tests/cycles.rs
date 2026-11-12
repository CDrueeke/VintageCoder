// SPDX-License-Identifier: GPL-3.0-or-later
//! Zyklenmessung auf der Hardware als Prüfung (`"mode": "measure"`): Zyklen vom ersten Zugriff auf A bis zum ersten danach auf B,
//! gemessen mit dem Debug-Strom der Ultimate 64 (Modus „6510 Only“, `vc_emu::streams::trace`), gewertet gegen `max` und/oder `min`.
//!
//! Ergebnis: bestanden oder fehlgeschlagen mit gemessenen Zyklen. Was sich nicht sauber messen lässt, wird **nie geschätzt**:
//! Paketverlust (die Messung über die Lücke ist ungültig) und fehlende Pakete sind „übersprungen“ mit Grund „ungültig …“
//! (mit `--strict` ein Fehler), ein nie erreichtes B ist eine Zeitüberschreitung.
//!
//! Ablauf je Messung: Debug-Strom starten (stoppt den Bildstrom), Programm laden und starten (Reset, Laden, RUN), warten, Strom stoppen.
//! Es wird keine Konfiguration des Geräts geändert und nichts im Flash gespeichert. Der Modus „6510 Only“ muss am Gerät eingestellt sein.
//! Die Messung enthält jeden Interrupt und jede Bad Line, die zwischen A und B auftritt.
use super::model::{Level, MeasureCase, Outcome, TestFile};
use super::runner::RunOptions;
use super::static_checks::Prepared;
use super::suite::{launch_hardware, Hardware};
use crate::cond::{Ctx, Expr};
use crate::streams::StreamOptions;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;
use vc_emu::streams::trace::{MeasureResult, Summary};
use vc_emu::Backend;

/// Ergebnis einer Messung samt Zustand des Stroms
#[derive(Debug, Clone)]
pub struct Measured { pub result: MeasureResult, pub summary: Summary }

/// Misst die Zyklen von `from` bis `to` (frischer Programmstart, Zeitlimit für die Messung nach dem Start). `Err`: Gerät nicht erreichbar,
/// Programm nicht startbar, Modus nicht unterstützt.
pub type Measurer<'a> = dyn FnMut(u16, u16, Duration) -> Result<Measured, String> + 'a;

struct NoCtx;
impl Ctx for NoCtx {
    fn reg(&self, _: &str) -> Option<i64> { None }
    fn peek(&mut self, _: u16) -> Result<u8, String> { Err("Speicher ist bei einer Hardware-Messung nicht lesbar".into()) }
}

fn eval(src: &str, pre: &Prepared) -> Result<i64, String> { Expr::parse(src, &pre.dbg.symbols).and_then(|e| e.eval(&mut NoCtx)) }
fn addr(src: &str, pre: &Prepared) -> Result<u16, String> {
    let v = eval(src, pre).map_err(|e| format!("„{src}“: {e}"))?;
    u16::try_from(v).map_err(|_| format!("„{src}“ liegt mit {v} außerhalb von $0000..$FFFF"))
}

fn cycles_text(r: &MeasureResult) -> String {
    match (r.min, r.max) {
        (Some(a), Some(b)) if a != b => format!("{a} bis {b} Zyklen ({} Messungen)", r.completed),
        (Some(a), _) => format!("{a} Zyklen"),
        _ => String::new(),
    }
}

/// Urteil über eine Messung. `strict`: Hinweise und ungültige Messungen werden zu Fehlern.
pub fn judge(case: &MeasureCase, from: u16, to: u16, max: Option<i64>, min: Option<i64>, m: &Measured, timeout_ms: u64, strict: bool) -> Outcome {
    let (r, s) = (&m.result, &m.summary);
    let name = case.name.clone();
    let span = format!("${from:04X} bis ${to:04X}");
    if r.completed == 0 {
        let lost = if s.lost_packets > 0 { format!(", {} Paket(e) verloren", s.lost_packets) } else { String::new() };
        let why = if s.packets == 0 { Some("ungültig: keine Pakete des Debug-Stroms empfangen (Firewall für eingehendes UDP, Modus „6510 Only“ am Gerät, Netzwerk?); nichts geschätzt".to_string()) }
            else if r.invalid > 0 { Some(format!("ungültig: Paketverlust während der Messung {span} ({} Messung(en) verworfen{lost}); nichts geschätzt, Lauf wiederholen", r.invalid)) }
            else { None };
        return match why {
            Some(w) if strict => Outcome::failed(name, Level::Error, format!("{w} (--strict)")),
            Some(w) => Outcome::skipped(name, w),
            None => Outcome::timed_out(name, if r.running { format!("{span} wurde in {timeout_ms} ms nicht durchlaufen: ${from:04X} erreicht, ${to:04X} danach nicht") } else { format!("{span} wurde in {timeout_ms} ms nicht durchlaufen: ${from:04X} nie erreicht") }),
        };
    }
    let (lo, hi) = (r.min.unwrap_or(0), r.max.unwrap_or(0));
    let mut bad = vec![];
    if let Some(mx) = max { if hi as i64 > mx { bad.push(format!("{hi} Zyklen von {span} überschreiten das Budget von {mx}")); } }
    if let Some(mn) = min { if (lo as i64) < mn { bad.push(format!("{lo} Zyklen von {span} unterschreiten die Untergrenze von {mn}")); } }
    let extra = if r.invalid > 0 { format!(" ({} weitere Messung(en) wegen Paketverlust ungültig und verworfen)", r.invalid) } else { String::new() };
    if bad.is_empty() { return Outcome::passed(name, Some(format!("{}{extra}", cycles_text(r)))); }
    let level = if strict { Level::Error } else { case.level };
    Outcome::failed(name, level, format!("{}{extra}", bad.join("; "))).with_measured(cycles_text(r))
}

/// Führt die Messungen der Testdatei aus (`"mode": "measure"`): je Eintrag in `tests` ein frischer Programmstart und eine Messung.
pub fn run_measure(tf: &TestFile, pre: &Prepared, ro: &RunOptions, measure: &mut Measurer) -> Vec<Outcome> {
    let mut out = vec![];
    let emit = |out: &mut Vec<Outcome>, o: Outcome| { if let Some(f) = &ro.on_outcome { f(&o); } out.push(o); };
    let mut broken: Option<String> = None;
    for (i, t) in tf.tests.iter().enumerate() {
        let case = match MeasureCase::parse(i, t) {
            Ok(c) => c,
            Err(e) => { let n = t.get("name").and_then(|n| n.as_str()).unwrap_or("Messung").to_string(); emit(&mut out, Outcome::failed(n, Level::Error, e)); continue; }
        };
        if !(ro.filter.as_ref().map_or(true, |f| case.name.contains(f.as_str())) && ro.only_names.as_ref().map_or(true, |n| n.contains(&case.name))) { continue; }
        if ro.cancel.as_ref().map_or(false, |f| f.load(Ordering::SeqCst)) { emit(&mut out, Outcome::skipped(case.name.clone(), "abgebrochen")); continue; }
        if let Some(why) = &broken { emit(&mut out, Outcome::skipped(case.name.clone(), why.clone())); continue; }
        let (from, to) = match (addr(&case.from, pre), addr(&case.to, pre)) { (Ok(a), Ok(b)) => (a, b), (Err(e), _) | (_, Err(e)) => { emit(&mut out, Outcome::failed(case.name.clone(), Level::Error, format!("Adresse nicht auswertbar: {e}"))); continue; } };
        let lim = |s: &Option<String>| -> Result<Option<i64>, String> { s.as_ref().map(|x| eval(x, pre).map_err(|e| format!("„{x}“: {e}"))).transpose() };
        let (max, min) = match (lim(&case.max), lim(&case.min)) { (Ok(a), Ok(b)) => (a, b), (Err(e), _) | (_, Err(e)) => { emit(&mut out, Outcome::failed(case.name.clone(), Level::Error, format!("Grenzwert nicht auswertbar: {e}"))); continue; } };
        let o = match measure(from, to, Duration::from_millis(case.timeout_ms)) {
            Ok(m) => judge(&case, from, to, max, min, &m, case.timeout_ms, ro.strict),
            Err(e) => { let why = format!("Messung nicht möglich: {e}"); broken = Some(why.clone()); Outcome::skipped(case.name.clone(), why) }
        };
        emit(&mut out, o);
    }
    out
}

/// Messer für echte Hardware: Debug-Strom der Ultimate 64 starten, danach Programm laden und starten. Sonst Fehler (MEGA65 hat keinen Debug-Strom).
pub fn hardware_measurer(hw: Hardware, target: String, program: std::path::PathBuf, opts: StreamOptions) -> impl FnMut(u16, u16, Duration) -> Result<Measured, String> {
    move |from, to, timeout| {
        let Hardware::Ultimate { host, password, .. } = &hw else { return Err("Der Debug-Strom (Zyklenmessung) ist nur an der Ultimate 64 verfügbar".into()) };
        let mut dev: Option<Box<dyn Backend>> = None;
        let r = crate::trace::measure_cycles_with(host.trim(), password.as_deref(), opts.clone(), from, to, timeout, || { dev = Some(launch_hardware(&hw, &target, Path::new(&program))?); Ok(()) });
        if let Some(mut d) = dev { d.stop(); }          // Rechner nicht angehalten zurücklassen
        r.map(|(result, summary)| Measured { result, summary })
    }
}
