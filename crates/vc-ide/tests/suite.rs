// SPDX-License-Identifier: GPL-3.0-or-later
//! Ergebnismodell und Lauf für den Tests-Reiter (T3), ohne Oberfläche: Zähler, Filter, Zeilenmarken, Ereignisse eines Laufs.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use vc_ide::build::{build, Ctx};
use vc_ide::tests::suite::{run, same_file, Event, Group, Item, Only, RunRequest, Suite, STATIC_TITLE};
use vc_ide::tests::{Level, Outcome, State};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn item(file: &str, o: Outcome) -> Item { Item { file: PathBuf::from(file), outcome: o } }

fn sample() -> Suite {
    let root = "/p";
    let mut s = Suite::default();
    s.groups.push(Group { title: STATIC_TITLE.into(), items: vec![
        item("/p/a.vctest", Outcome::passed("Größe Main ≤ $2000", Some("$1F3C".into()))),
        item("/p/a.vctest", Outcome::passed("Zyklen irq_handler ≤ 63", Some("48 bis 52".into())).at("/p/src/irq.asm", 41)),
        item("/p/a.vctest", Outcome::failed("Zyklen mul16 ≤ 100", Level::Hint, "im ungünstigsten Fall 120 Zyklen").at("/p/src/irq.asm", 41)),
    ], ..Default::default() });
    s.groups.push(Group { title: "a.vctest".into(), file: Some(PathBuf::from("/p/a.vctest")), backend: Some("VICE x64sc".into()), items: vec![
        item("/p/a.vctest", Outcome::passed("div16: 1000 / 8", Some("196 Zyklen".into())).at("/p/src/math.asm", 7)),
        item("/p/a.vctest", Outcome::failed("mul16: 300 × 300", Level::Error, "Erwartung „peekw($0200) == 90000“ ist nicht erfüllt").at("/p/src/math.asm", 50)),
        item("/p/a.vctest", Outcome::timed_out("hängt", "nicht zurück").at("/p/src/math.asm", 60)),
        item("/p/a.vctest", Outcome::skipped("sqrt8: Zeitverhalten", "Zyklen nicht messbar")),
    ], ..Default::default() });
    let _ = root;
    s
}

#[test]
fn counts_status_and_filters() {
    let s = sample();
    let c = s.counts();
    assert_eq!((c.passed, c.failed, c.timed_out, c.skipped, c.hints, c.total()), (3, 1, 1, 1, 1, 7));
    assert_eq!(s.failed_count(), 2, "Fehlschlag und Zeitüberschreitung; Hinweise zählen nicht");
    assert_eq!(s.status_text().as_deref(), Some("Tests 3/7"));
    assert_eq!(Suite::default().status_text(), None);
    assert_eq!(s.visible("", false).len(), 7);
    assert_eq!(s.visible("MUL16", false), vec![(0, 2), (1, 1)], "Filter ohne Beachtung der Schreibweise");
    assert_eq!(s.visible("nicht messbar", false), vec![(1, 3)], "Filter sucht auch im Grund");
    assert_eq!(s.visible("", true), vec![(1, 1), (1, 2)], "nur Fehlgeschlagene und Zeitüberschreitungen");
    assert_eq!(s.failed_tests(), vec![(PathBuf::from("/p/a.vctest"), "mul16: 300 × 300".to_string()), (PathBuf::from("/p/a.vctest"), "hängt".to_string())]);
}

#[test]
fn marks_for_a_source_file_pick_the_worst_result_per_line() {
    let s = sample();
    let root = Path::new("/p");
    let m = s.marks_for(root, "src/irq.asm");
    assert_eq!(m.len(), 1, "{m:?}");
    assert_eq!((m[0].line, m[0].state, m[0].level), (40, State::Failed, Level::Hint), "Hinweis schlägt OK auf derselben Zeile");
    assert!(m[0].text.starts_with("Hinweis"), "{}", m[0].text);
    let m = s.marks_for(root, "src/math.asm");
    assert_eq!(m.iter().map(|x| (x.line, x.state)).collect::<Vec<_>>(), vec![(6, State::Passed), (49, State::Failed), (59, State::TimedOut)]);
    assert_eq!(m[0].text, "OK 196 Zyklen");
    assert!(s.marks_for(root, "src/andere.asm").is_empty());
    assert!(same_file(root, "/p/x/y.asm", "x/y.asm") && same_file(root, "x/y.asm", "x/y.asm") && same_file(root, "./x/y.asm", "x/y.asm") && !same_file(root, "/q/x/y.asm", "x/y.asm"));
}

#[test]
fn events_replace_and_extend_results() {
    let mut s = sample();
    let f = PathBuf::from("/p/a.vctest");
    // einzelner Test wird wiederholt: Ergebnis ersetzt, die übrigen bleiben
    s.begin(&Only::Test(f.clone(), "mul16: 300 × 300".into()));
    assert!(s.running && s.groups.len() == 2);
    s.apply(Event::FileStarted { file: f.clone(), title: "a.vctest".into(), backend: Some("VICE x64sc".into()), expected: 1, mailbox: false });
    assert_eq!(s.groups[1].expected, 1);
    s.apply(Event::Outcome { file: f.clone(), outcome: Outcome::passed("mul16: 300 × 300", Some("412 Zyklen".into())) });
    s.apply(Event::FileDone { file: f.clone(), millis: 1200 });
    s.apply(Event::Done { millis: 1300 });
    assert!(!s.running && s.last_millis == Some(1300) && s.groups[1].millis == 1200 && s.groups[1].expected == 0);
    assert_eq!(s.groups[1].items.len(), 4);
    assert_eq!(s.groups[1].items[1].outcome.state, State::Passed);
    // neuer Name wird angehängt, unbekannte Datei ignoriert
    s.apply(Event::Outcome { file: f.clone(), outcome: Outcome::passed("neu", None) });
    s.apply(Event::Outcome { file: PathBuf::from("/p/x.vctest"), outcome: Outcome::passed("fremd", None) });
    assert_eq!(s.groups[1].items.len(), 5);
    // statische Gruppe wird ersetzt
    s.apply(Event::Static(Group { title: STATIC_TITLE.into(), items: vec![item("/p/a.vctest", Outcome::passed("nur eins", None))], ..Default::default() }));
    assert_eq!(s.groups[0].items.len(), 1);
    // vollständiger Lauf beginnt frisch
    s.begin(&Only::All);
    assert!(s.groups.is_empty() && s.running);
    s.apply(Event::Error("kaputt".into()));
    assert!(!s.running && s.error.as_deref() == Some("kaputt"));
}

const ASM: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.asm");
const TESTS: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.vctest");

fn project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-suite-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tests_demo.asm"), ASM).unwrap();
    std::fs::write(dir.join("tests_demo.vctest"), TESTS).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "tests_demo.asm", Some("c64")).unwrap()["ok"], true);
    dir
}

fn collect(req: &RunRequest, cancel: bool) -> (Vec<Event>, Suite) {
    let (tx, rx) = channel();
    run(req, Arc::new(AtomicBool::new(cancel)), &tx);
    drop(tx);
    let ev: Vec<Event> = rx.iter().collect();
    let mut s = Suite::default(); s.begin(&req.only);
    for e in ev.iter().cloned() { s.apply(e); }
    (ev, s)
}

#[test]
fn run_without_emulator_reports_static_checks_and_skips_the_rest_with_reason() {
    let dir = project("static");
    let req = RunRequest { root: dir.clone(), data: data(), backend: None, strict: false, only: Only::All, hardware: None };
    let (ev, s) = collect(&req, false);
    assert!(matches!(ev.first(), Some(Event::Static(_))) && matches!(ev.last(), Some(Event::Done { .. })), "{ev:?}");
    let st = &s.groups[0];
    assert_eq!(st.title, STATIC_TITLE);
    assert!(st.items.iter().any(|i| i.outcome.name == "Zyklen irq_handler ≤ 24" && i.outcome.measured.as_deref() == Some("20")));
    let g = &s.groups[1];
    assert_eq!((g.title.as_str(), g.backend.clone(), g.items.len()), ("tests_demo.vctest", None, 4));
    assert!(g.items.iter().all(|i| i.outcome.state == State::Skipped && i.outcome.detail.contains("kein Emulator gewählt")), "{:?}", g.items);
    assert_eq!(s.failed_count(), 0);
    // Marke an der Zeile des Labels irq_handler
    let m = s.marks_for(&dir, "tests_demo.asm");
    assert!(m.iter().any(|x| x.line == 49 && x.text.starts_with("OK")), "{m:?}");
    // abgebrochen: keine Emulator-Gruppen, aber die statischen Prüfungen stehen
    let (ev, s) = collect(&req, true);
    assert!(s.groups.len() == 1 && matches!(ev.last(), Some(Event::Done { .. })), "{ev:?}");
    // Einzeltest: nur die Datei, übrige Ergebnisse bleiben
    let one = RunRequest { only: Only::Test(dir.join("tests_demo.vctest"), "next_color: 3 wird 4".into()), ..req.clone() };
    let (_, s1) = collect(&one, false);
    assert_eq!(s1.groups[1].items.len(), 1);
    assert_eq!(s1.groups[1].items[0].outcome.name, "next_color: 3 wird 4");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn run_with_missing_build_marks_tests_skipped_and_static_failed() {
    let dir = std::env::temp_dir().join(format!("vc-suite-nobuild-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tests_demo.vctest"), TESTS).unwrap();
    let req = RunRequest { root: dir.clone(), data: data(), backend: Some("vice".into()), strict: false, only: Only::All, hardware: None };
    let (_, s) = collect(&req, false);
    assert!(s.groups[0].items.iter().any(|i| i.outcome.is_error() && i.outcome.detail.contains("zuerst bauen")), "{:?}", s.groups[0]);
    assert!(s.groups[1].items.iter().all(|i| i.outcome.state == State::Skipped && i.outcome.detail.contains("nicht gebaut")), "{:?}", s.groups[1].items);
    assert!(s.groups[1].backend.as_deref() == Some("VICE x64sc"));
    std::fs::remove_dir_all(&dir).ok();
}

/// Mit echtem VICE (öffnet ein Fenster): ein Einzeltest und die Wiederholung der Fehlgeschlagenen
#[test]
fn run_with_vice() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let dir = project("vice");
    let f = dir.join("tests_demo.vctest");
    let req = RunRequest { root: dir.clone(), data: data(), backend: Some("vice".into()), strict: false, only: Only::Test(f.clone(), "next_color: 3 wird 4".into()), hardware: None };
    let (_, s) = collect(&req, false);
    let g = &s.groups[1];
    assert_eq!(g.items.len(), 1, "{g:?}");
    assert_eq!((g.items[0].outcome.state, g.items[0].outcome.measured.as_deref()), (State::Passed, Some("19 Zyklen")), "{:?}", g.items[0]);
    assert_eq!(g.backend.as_deref(), Some("VICE x64sc"));
    let _ = Ordering::SeqCst;
    std::fs::remove_dir_all(&dir).ok();
}

/// „Diesen Test debuggen“ mit echtem VICE: angehalten am Anfang der Falle, danach läuft der Test bis zum `jmp *` und liefert das Ergebnis
#[test]
fn prepare_debug_arms_the_trap_with_vice() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let dir = project("debug");
    let mut plan = vc_ide::tests::suite::prepare_debug(&dir.join("tests_demo.vctest"), &data(), "next_color: 15 wird 0").unwrap();
    assert_eq!(plan.target, "c64");
    assert!(plan.symbols.contains_key("next_color"));
    assert_eq!((plan.info.trap, plan.info.call, plan.info.halt, plan.info.prologue), (0xC000, 0xC000, 0xC003, false));
    let r = plan.backend.registers().unwrap();
    assert_eq!(r.pc, 0xC000, "angehalten am Anfang der Falle");
    assert_eq!(plan.backend.read_mem(0xFB, 1).unwrap(), vec![15], "Setup geschrieben");
    assert!(plan.backend.run_until(plan.info.halt, std::time::Duration::from_secs(3)).unwrap());
    assert_eq!(plan.backend.read_mem(0xFB, 1).unwrap(), vec![0], "next_color hat 15 auf 0 gesetzt");
    assert!(vc_ide::tests::suite::prepare_debug(&dir.join("tests_demo.vctest"), &data(), "gibt es nicht").is_err());
    plan.backend.stop();
    std::fs::remove_dir_all(&dir).ok();
}

const MB_ASM: &str = include_str!("../../../examples/c64/tests_demo/tests_mailbox.asm");
const MB_TESTS: &str = include_str!("../../../examples/c64/tests_demo/tests_mailbox.vctest");

fn mailbox_project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-suite-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tests_mailbox.asm"), MB_ASM).unwrap();
    std::fs::write(dir.join("tests_mailbox.vctest"), MB_TESTS).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "tests_mailbox.asm", Some("c64")).unwrap()["ok"], true);
    dir
}

#[test]
fn mailbox_files_form_their_own_group_and_cannot_be_debugged_with_a_trap() {
    let dir = mailbox_project("mbox");
    let req = RunRequest { root: dir.clone(), data: data(), backend: None, strict: false, only: Only::All, hardware: None };
    let (_, s) = collect(&req, false);
    let g = &s.groups[1];
    assert!(g.mailbox && g.file.is_some(), "{g:?}");
    assert_eq!(g.items.len(), 1);
    assert!(g.items[0].outcome.state == State::Skipped && g.items[0].outcome.detail.contains("kein Emulator"), "{:?}", g.items[0]);
    let e = vc_ide::tests::suite::prepare_debug(&dir.join("tests_mailbox.vctest"), &data(), "double, plus3 und Speicher (Selbsttest)").err().expect("Fehler erwartet");
    assert!(e.contains("Postfach-Modus"), "{e}");
    std::fs::remove_dir_all(&dir).ok();
}

/// Mit echtem VICE: das Beispiel „tests_mailbox“ bestätigt sein Postfach-Ergebnis
#[test]
fn mailbox_example_with_vice() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let dir = mailbox_project("mboxvice");
    let req = RunRequest { root: dir.clone(), data: data(), backend: Some("vice".into()), strict: false, only: Only::All, hardware: None };
    let (_, s) = collect(&req, false);
    let g = &s.groups[1];
    assert_eq!(g.backend.as_deref(), Some("VICE x64sc · Postfach"));
    assert_eq!((g.items[0].outcome.state, g.items[0].outcome.measured.as_deref()), (State::Passed, Some("4 Prüfungen")), "{:?}", g.items[0]);
    std::fs::remove_dir_all(&dir).ok();
}
