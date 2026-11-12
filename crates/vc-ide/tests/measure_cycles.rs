// SPDX-License-Identifier: GPL-3.0-or-later
//! Zyklenmessung als Prüfung (`"mode": "measure"`): Urteil gegen Budget, Paketverlust → ungültig, Läufer mit Mess-Ersatz.
//! Die Pakete sind echte Aufzeichnungen (`vc-emu/tests/data/debug.bin`, Ultimate 64 II, „6510 Only“): Die Warteschleife `$0A95`
//! (LDA abs 4 + BEQ genommen 3) braucht 7 Zyklen je Durchlauf, 50 bei einer Bad Line, 1 beim IRQ-Einsprung (doppelter Opcode-Abruf).
//! Gegen echte Hardware: `U64_HOST=… cargo test -p vc-ide --test measure_cycles -- --ignored --nocapture` (Gerät wird zurückgesetzt).
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use vc_emu::streams::trace::{Analyzer, MeasureSpec, OpTable};
use vc_emu::streams::RxMeta;
use vc_ide::build::{build, Ctx};
use vc_ide::tests::cycles::{judge, run_measure, Measured};
use vc_ide::tests::runner::RunOptions;
use vc_ide::tests::suite::{run, Only, RunRequest, Suite};
use vc_ide::tests::{Level, MeasureCase, Prepared, State, TestFile};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn data() -> PathBuf { root().join("data") }
fn packets() -> Vec<Vec<u8>> {
    let b = std::fs::read(root().join("crates/vc-emu/tests/data/debug.bin")).unwrap();
    let (mut o, mut out) = (0, vec![]);
    while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; out.push(b[o + 2..o + 2 + n].to_vec()); o += 2 + n; }
    out
}

/// Aufgezeichnete Pakete durch den Analysator (wie der Empfangsthread); `lose`: dieses Paket fehlt
fn measured(from: u16, to: u16, lose: Option<usize>) -> Measured {
    let mut an = Analyzer::new(OpTable::empty());
    let id = an.add_measure(MeasureSpec { name: "m".into(), from, to });
    for (i, p) in packets().iter().enumerate() {
        if lose == Some(i) { continue; }
        let seq = u16::from_le_bytes([p[0], p[1]]);
        an.feed(p, &RxMeta { seq, lost_before: (lose.map_or(false, |l| i == l + 1)) as u16, late: false });
    }
    Measured { result: an.measure(id).unwrap(), summary: an.summary() }
}

fn case(max: Option<&str>, min: Option<&str>, level: Level) -> MeasureCase {
    MeasureCase { name: "Schleife".into(), from: "$0A95".into(), to: "$0A95".into(), max: max.map(Into::into), min: min.map(Into::into), level, timeout_ms: 5000 }
}

#[test]
fn recorded_packets_pass_and_fail_against_the_budget() {
    // $0A95 → $0A98: LDA abs (4 Zyklen), dann der Abruf des BEQ: 4 Zyklen, bei Bad Line +43
    let m = measured(0x0A95, 0x0A98, None);
    let (lo, hi) = (m.result.min.unwrap() as i64, m.result.max.unwrap() as i64);
    assert!(m.result.completed > 100 && lo == 4 && hi > 4, "{:?}", m.result);
    // Die größten Werte enthalten Raster-IRQ und Bad Line (die Hardware misst, was der Bus sieht): hier 801 Zyklen
    let c = case(None, None, Level::Error);
    let o = judge(&c, 0x0A95, 0x0A98, Some(hi + 100), None, &m, 5000, false);
    assert_eq!(o.state, State::Passed, "{o:?}");
    assert!(o.measured.as_deref().unwrap().contains(&format!("{lo} bis {hi} Zyklen")), "{:?}", o.measured);
    // Budget genau auf dem Höchstwert: bestanden, einen Zyklus darunter: fehlgeschlagen (Bad Line und IRQ sind enthalten)
    assert_eq!(judge(&c, 0x0A95, 0x0A98, Some(hi), None, &m, 5000, false).state, State::Passed);
    let f = judge(&c, 0x0A95, 0x0A98, Some(hi - 1), None, &m, 5000, false);
    assert!(f.state == State::Failed && f.is_error() && f.detail.contains("überschreiten das Budget") && f.measured.is_some(), "{f:?}");
    // level hint: nur Hinweis; --strict macht daraus einen Fehler
    let h = case(Some("5"), None, Level::Hint);
    let o = judge(&h, 0x0A95, 0x0A98, Some(5), None, &m, 5000, false); assert!(o.state == State::Failed && !o.is_error(), "{o:?}");
    assert!(judge(&h, 0x0A95, 0x0A98, Some(5), None, &m, 5000, true).is_error());
    // Untergrenze
    let o = judge(&c, 0x0A95, 0x0A98, None, Some(lo + 1), &m, 5000, false); assert!(o.is_error() && o.detail.contains("unterschreiten"), "{o:?}");
    assert_eq!(judge(&c, 0x0A95, 0x0A98, None, Some(lo), &m, 5000, false).state, State::Passed);
}

#[test]
fn an_exact_single_value_is_shown_as_one_number() {
    // Schleife ohne Bad Line und IRQ in einem Ausschnitt der Aufzeichnung gibt es nicht garantiert; also direkt aus den Werten prüfen
    let mut m = measured(0x0A95, 0x0A98, None);
    m.result.min = Some(8); m.result.max = Some(8); m.result.completed = 1;
    let o = judge(&case(Some("10"), None, Level::Error), 0x0A95, 0x0A98, Some(10), None, &m, 5000, false);
    assert_eq!((o.state, o.measured.as_deref()), (State::Passed, Some("8 Zyklen")), "{o:?}");
}

#[test]
fn packet_loss_makes_the_measurement_invalid_and_never_guesses() {
    // Ein langes Messfenster (IRQ bis IRQ-Ende ist hier nicht nötig): A = B nur einmal pro Epoche erreichbar machen, indem früh Pakete fehlen.
    // Zuerst: alle Messungen über die Lücke sind verworfen, Messungen danach laufen weiter (Epoche 1)
    let m = measured(0x0A95, 0x0A98, Some(10));
    assert_eq!(m.summary.lost_packets, 1); assert!(m.result.completed > 0, "nach der Lücke wird wieder gemessen");
    // Läuft die Messung, wenn das Paket fehlt, ins Leere (nichts abgeschlossen), ist das Urteil „ungültig“, nicht bestanden und nicht fehlgeschlagen
    let mut v = m.clone(); v.result.completed = 0; v.result.min = None; v.result.max = None; v.result.last = None; v.result.invalid = 1; v.result.running = false;
    let c = case(Some("1000"), None, Level::Error);
    let o = judge(&c, 0x0A95, 0x0A98, Some(1000), None, &v, 5000, false);
    assert!(o.state == State::Skipped && !o.is_error() && o.detail.starts_with("ungültig: Paketverlust") && o.detail.contains("nichts geschätzt"), "{o:?}");
    // --strict: ungültig ist ein Fehler
    assert!(judge(&c, 0x0A95, 0x0A98, Some(1000), None, &v, 5000, true).is_error());
    // keine Pakete: ebenfalls ungültig (Firewall), nicht „B nie erreicht“
    let mut none = v.clone(); none.summary.packets = 0; none.result.invalid = 0;
    let o = judge(&c, 0x0A95, 0x0A98, Some(1000), None, &none, 5000, false);
    assert!(o.state == State::Skipped && o.detail.contains("keine Pakete"), "{o:?}");
    // Pakete da, nichts verloren, B nie erreicht: Zeitüberschreitung
    let mut late = v.clone(); late.result.invalid = 0;
    let o = judge(&c, 0x0A95, 0x0A98, Some(1000), None, &late, 5000, false);
    assert!(o.state == State::TimedOut && o.detail.contains("nie erreicht"), "{o:?}");
    late.result.running = true;
    assert!(judge(&c, 0x0A95, 0x0A98, Some(1000), None, &late, 5000, false).detail.contains("danach nicht"));
    // eine gültige Messung neben verworfenen zählt und nennt die verworfenen
    let mut ok = measured(0x0A95, 0x0A98, None); ok.result.invalid = 2;
    let o = judge(&c, 0x0A95, 0x0A98, Some(1000), None, &ok, 5000, false);
    assert!(o.state == State::Passed && o.measured.as_deref().unwrap().contains("2 weitere Messung(en)"), "{o:?}");
}

const ASM: &str = ".target c64\n.cpu 6510 { illegals = deny }\n.org $0801\n.basicstub start\nstart: sei\nwarte: lda $d012\n       beq warte\nende:  jmp ende\n";

fn project(tag: &str, tests: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-mc-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("m.asm"), ASM).unwrap();
    std::fs::write(dir.join("m.vctest"), format!(r#"{{ "kind": "test", "target": "c64", "program": "build/m.prg", "mode": "measure", "tests": {tests} }}"#)).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "m.asm", Some("c64")).unwrap()["ok"], true);
    dir
}

#[test]
fn the_runner_resolves_symbols_and_stops_after_a_device_error() {
    let dir = project("run", r#"[
        { "name": "ok", "from": "warte", "to": "ende", "max": 50 },
        { "name": "zu eng", "from": "warte", "to": "ende", "max": "4+1", "level": "hint" },
        { "name": "kaputt", "from": "gibtsnicht", "to": "ende", "max": 5 },
        { "name": "ohne Grenze", "from": "warte", "to": "ende" }
    ]"#);
    let tf = TestFile::load(&dir.join("m.vctest")).unwrap();
    let pre = Prepared::load(&tf, &data()).unwrap();
    let warte = pre.dbg.symbols["warte"] as u16; let ende = pre.dbg.symbols["ende"] as u16;
    let mut seen = vec![];
    let mut m = |f: u16, t: u16, _: Duration| -> Result<Measured, String> { seen.push((f, t)); Ok(Measured { result: { let mut r = measured(0x0A95, 0x0A98, None).result; r.min = Some(9); r.max = Some(9); r.completed = 1; r }, summary: measured(0x0A95, 0x0A98, None).summary }) };
    let o = run_measure(&tf, &pre, &RunOptions::default(), &mut m);
    assert_eq!(seen, vec![(warte, ende), (warte, ende)], "gemessen wird nur, was sich auswerten lässt");
    assert_eq!(o.len(), 4, "{o:?}");
    assert_eq!((o[0].state, o[0].measured.as_deref()), (State::Passed, Some("9 Zyklen")), "{:?}", o[0]);
    assert!(o[1].state == State::Failed && !o[1].is_error() && o[1].detail.contains("Budget von 5"), "{:?}", o[1]);
    assert!(o[2].is_error() && o[2].detail.contains("gibtsnicht"), "{:?}", o[2]);
    assert!(o[3].is_error() && o[3].detail.contains("prüft nichts"), "{:?}", o[3]);
    // Gerät nicht erreichbar: die erste Messung meldet den Grund, die übrigen werden übersprungen
    let mut bad = |_: u16, _: u16, _: Duration| -> Result<Measured, String> { Err("keine Verbindung".into()) };
    let o = run_measure(&tf, &pre, &RunOptions::default(), &mut bad);
    assert!(o[0].state == State::Skipped && o[0].detail.contains("keine Verbindung"), "{:?}", o[0]);
    assert!(o[1].state == State::Skipped && o[1].detail.contains("keine Verbindung"), "{:?}", o[1]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn without_hardware_the_suite_skips_measurements() {
    let dir = project("skip", r#"[ { "name": "ok", "from": "warte", "to": "ende", "max": 50 } ]"#);
    let req = RunRequest { root: dir.clone(), data: data(), backend: Some("vice".into()), strict: false, only: Only::All, hardware: None };
    let (tx, rx) = channel();
    run(&req, Arc::new(AtomicBool::new(false)), &tx); drop(tx);
    let mut s = Suite::default(); s.begin(&req.only);
    for e in rx.iter() { s.apply(e); }
    let g = s.groups.iter().find(|g| g.title == "m.vctest").unwrap();
    assert!(g.mailbox, "kein Debuggen mit Falle");
    assert!(g.items[0].outcome.state == State::Skipped && g.items[0].outcome.detail.contains("Ultimate 64"), "{:?}", g.items[0]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn mega65_has_no_debug_stream() {
    let dir = project("m65", r#"[ { "name": "ok", "from": "warte", "to": "ende", "max": 50 } ]"#);
    let tf = TestFile::load(&dir.join("m.vctest")).unwrap();
    let pre = Prepared::load(&tf, &data()).unwrap();
    let mut m = vc_ide::tests::cycles::hardware_measurer(vc_ide::tests::suite::Hardware::Mega65 { spec: "serial:/dev/null".into(), name: String::new() }, "c64".into(), pre.program.clone(), Default::default());
    let o = run_measure(&tf, &pre, &RunOptions::default(), &mut m);
    assert!(o[0].state == State::Skipped && o[0].detail.contains("nur an der Ultimate 64"), "{:?}", o[0]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn model_parses_and_rejects() {
    let p = |s: &str| MeasureCase::parse(0, &serde_json::from_str(s).unwrap());
    let c = p(r#"{ "name": "x", "from": "$1000", "to": 4112, "max": "$40", "min": 3 }"#).unwrap();
    assert_eq!((c.from.as_str(), c.to.as_str(), c.max.as_deref(), c.min.as_deref(), c.level, c.timeout_ms), ("$1000", "4112", Some("$40"), Some("3"), Level::Error, 20000));
    assert!(p(r#"{ "to": 1, "max": 2 }"#).unwrap_err().contains("\"from\" fehlt"));
    assert!(p(r#"{ "from": 1, "max": 2 }"#).unwrap_err().contains("\"to\" fehlt"));
    assert!(p(r#"{ "from": 1, "to": 2 }"#).unwrap_err().contains("prüft nichts"));
    assert!(p(r#"{ "from": 1, "to": 2, "max": 3, "level": "x" }"#).unwrap_err().contains("level"));
    assert!(TestFile::parse(std::path::Path::new("a.vctest"), r#"{"target":"c64","program":"a.prg","mode":"measure"}"#).is_ok());
    assert!(TestFile::parse(std::path::Path::new("a.vctest"), r#"{"target":"c64","program":"a.prg","mode":"x"}"#).unwrap_err().contains("measure"));
}

/// Echtes Gerät (Ultimate 64 II, „6510 Only“ am Gerät eingestellt): setzt zurück, lädt und startet das Programm, liest nur den Strom.
/// Das Beispiel `examples/c64/tests_cycles` misst die Warteschleife `messen` bis `ende`: genau 1284 Zyklen (Bildschirm aus, I-Flag gesetzt).
#[test]
#[ignore]
fn hardware_measures_the_wait_loop() {
    let Ok(host) = std::env::var("U64_HOST") else { eprintln!("U64_HOST nicht gesetzt"); return };
    let dir = std::env::temp_dir().join(format!("vc-mc-hw-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tests_cycles.asm"), include_str!("../../../examples/c64/tests_cycles/tests_cycles.asm")).unwrap();
    std::fs::write(dir.join("tests_cycles.vctest"), include_str!("../../../examples/c64/tests_cycles/tests_cycles.vctest")).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "tests_cycles.asm", Some("c64")).unwrap()["ok"], true);
    let tf = TestFile::load(&dir.join("tests_cycles.vctest")).unwrap();
    let pre = Prepared::load(&tf, &data()).unwrap();
    let hw = vc_ide::tests::suite::Hardware::Ultimate { host, password: std::env::var("U64_PASSWORD").ok(), name: String::new() };
    let mut m = vc_ide::tests::cycles::hardware_measurer(hw, "c64".into(), pre.program.clone(), vc_ide::streams::StreamOptions::from_settings(&vc_ide::settings::effective()));
    let o = run_measure(&tf, &pre, &RunOptions::default(), &mut m);
    println!("{o:#?}");
    assert_eq!((o[0].state, o[0].measured.as_deref()), (State::Passed, Some("1284 Zyklen")), "{:?}", o[0]);
    assert!(o[1].state == State::Failed && !o[1].is_error(), "{:?}", o[1]);
    std::fs::remove_dir_all(&dir).ok();
}
