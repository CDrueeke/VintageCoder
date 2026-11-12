// SPDX-License-Identifier: GPL-3.0-or-later
//! JUnit-XML-Ausgabe: Zuordnung der Zustände, Maskierung und Zähler.
use vc_ide::tests::junit::{render, SuiteResult};
use vc_ide::tests::{Level, Outcome};

#[test]
fn maps_states_and_escapes() {
    let outcomes = vec![
        Outcome::passed("ok <1>", Some("48 bis 52".into())),
        Outcome::passed("ohne Messwert", None),
        Outcome::failed("mul16 \"300\"", Level::Error, "Erwartung „a & b“ ist nicht erfüllt\nzweite Zeile").at("src/m.asm", 50),
        Outcome::failed("Budget", Level::Hint, "19 Zyklen gemessen, erlaubt: höchstens 18").with_measured("19 Zyklen"),
        Outcome::timed_out("hängt", "nicht zurück"),
        Outcome::skipped("später", "braucht den Emulator"),
        Outcome::skipped("Steuerzeichen\u{1}", "Grund\u{2}"),
    ];
    let x = render(&[SuiteResult { name: "a.vctest".into(), outcomes, millis: 1234 }, SuiteResult { name: "Statische Prüfungen".into(), outcomes: vec![Outcome::passed("Größe", None)], millis: 5 }]);
    assert!(x.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"vcc test\" tests=\"8\" failures=\"1\" errors=\"1\" skipped=\"2\" time=\"1.239\">"), "{x}");
    assert!(x.contains("<testsuite name=\"a.vctest\" tests=\"7\" failures=\"1\" errors=\"1\" skipped=\"2\" time=\"1.234\">"));
    assert!(x.contains("name=\"ok &lt;1&gt;\"") && x.contains("<system-out>48 bis 52</system-out>"));
    assert!(x.contains("<testcase classname=\"a.vctest\" name=\"ohne Messwert\"/>"), "bestandener Fall ohne Inhalt ist selbstschließend");
    assert!(x.contains("name=\"mul16 &quot;300&quot;\"") && x.contains("<failure message=\"Erwartung „a &amp; b“ ist nicht erfüllt\" type=\"fehlgeschlagen\">src/m.asm:50\nErwartung"), "{x}");
    assert!(x.contains("<system-out>HINWEIS: 19 Zyklen gemessen, erlaubt: höchstens 18 (19 Zyklen)</system-out>"), "Hinweis ist kein Fehlschlag");
    assert!(x.contains("<error message=\"nicht zurück\" type=\"Zeitüberschreitung\">"));
    assert!(x.contains("<skipped message=\"braucht den Emulator\"/>") && x.contains("name=\"Steuerzeichen\"") && x.contains("<skipped message=\"Grund\"/>"), "{x}");
    assert!(!x.contains('\u{1}') && !x.contains('\u{2}'));
    assert_eq!(x.matches("<testcase").count(), 8);
    assert_eq!(x.matches("</testsuite>").count(), 2);
}
