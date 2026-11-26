// SPDX-License-Identifier: GPL-3.0-or-later
//! Start und Schrittweite der Zeilennummern: Standard 10/10, Direktive `#numbering`, Projektwerte, Bereich, Überlauf (B190–B192).
use std::path::{Path, PathBuf};
use vc_basic::dialects::{self, AnyDialect};
use vc_basic::labels::{self, Cx, Unit};
use vc_basic::numbering::{self, Numbering};
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn prg(src: &str, start: u32, step: u32) -> Result<Vec<u8>, Vec<vc_core::Diagnostic>> {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let mut o = PlainOptions::new("t.bas", 63999); (o.start, o.step) = (start, step);
    compile_plain_cbm(src, &d, &o, d.load_addr).map(|r| r.prg)
}
fn line_numbers(prg: &[u8]) -> Vec<u16> { vc_basic::parse_prg(prg).unwrap().lines.iter().map(|l| l.0).collect() }
fn to_numbers(src: &str, n: Numbering, range: Option<(u32, u32)>) -> Result<String, Vec<vc_core::Diagnostic>> {
    let (d, i) = (AnyDialect::load(&data(), "cbm_v2").unwrap(), dialects::info(&data(), "cbm_v2").unwrap());
    labels::to_numbers(&[Unit { file: "t.bas".into(), text: src.into() }], &Cx::new(&d, &i).numbering(n.start, n.step), range).map(|mut c| c.units.remove(0).text)
}

#[test]
fn default_is_ten_and_ten() {
    assert_eq!(Numbering::default(), Numbering::new(10, 10));
    assert_eq!(line_numbers(&prg("PRINT 1\nPRINT 2\nPRINT 3\n", 10, 10).unwrap()), [10, 20, 30]);
    assert_eq!(line_numbers(&prg("PRINT 1\nPRINT 2\nPRINT 3\n", 100, 5).unwrap()), [100, 105, 110]);
    assert_eq!(Numbering::from_json(&serde_json::json!({})), Numbering::new(10, 10));
    let n = Numbering::from_json(&serde_json::json!({ "start": 100, "step": 5 }));
    assert_eq!(n, Numbering::new(100, 5));
    assert_eq!(Numbering::from_json(&n.to_json()), n);
}

#[test]
fn file_directive_beats_the_project_value() {
    let p = Numbering::new(10, 10);
    assert_eq!(numbering::resolve("PRINT\n", "a", p, 63999).unwrap(), p);
    assert_eq!(numbering::resolve("#numbering start=100 step=5\nPRINT\n", "a", p, 63999).unwrap(), Numbering::new(100, 5));
    assert_eq!(numbering::resolve("#NUMBERING step=2\n", "a", p, 63999).unwrap(), Numbering::new(10, 2), "Teilangabe ergänzt den Projektwert");
    assert_eq!(numbering::directive_line(Numbering::new(100, 5)), "#numbering start=100 step=5");
    // Bau: die Direktive gilt, die Zeile wird nicht zum Programm
    assert_eq!(line_numbers(&prg("#numbering start=100 step=5\nPRINT 1\nPRINT 2\n", 10, 10).unwrap()), [100, 105]);
    // auch bei der Umwandlung
    let (d, i) = (AnyDialect::load(&data(), "cbm_v2").unwrap(), dialects::info(&data(), "cbm_v2").unwrap());
    let _ = (&d, &i);
    let t = to_numbers("#numbering start=100 step=5\nPRINT 1\nPRINT 2\n", numbering::resolve("#numbering start=100 step=5\n", "a", p, 63999).unwrap(), None).unwrap();
    assert_eq!(t, "#numbering start=100 step=5\n100 PRINT 1\n105 PRINT 2\n");
}

#[test]
fn invalid_values_are_reported() {
    let p = Numbering::default();
    for bad in ["#numbering\n", "#numbering start=x\n", "#numbering stride=5\n", "#numbering start\n"] { assert_eq!(numbering::resolve(bad, "a", p, 63999).unwrap_err()[0].code, "B190", "{bad}"); }
    assert_eq!(numbering::resolve("#numbering step=0\n", "a", p, 63999).unwrap_err()[0].code, "B191");
    assert_eq!(numbering::resolve("#numbering start=70000\n", "a", p, 63999).unwrap_err()[0].code, "B191");
    assert_eq!(numbering::resolve("#numbering start=40000\n", "a", p, 32767).unwrap_err()[0].code, "B191", "Grenze des Dialekts");
    assert!(numbering::resolve("#numbering#x\n", "a", p, 63999).is_ok(), "keine Direktive");
}

#[test]
fn overflow_suggests_a_smaller_step() {
    let src: String = (0..700).map(|i| format!("PRINT {i}\n")).collect();
    let e = prg(&src, 10, 100).err().unwrap();
    assert_eq!(e[0].code, "B192");
    assert_eq!(e[0].arg_text("suggestion"), "91", "(63999 - 10) / 699");
    assert!(prg(&src, 10, 91).is_ok());
    // Umwandlung Labels → Nummern
    let e = to_numbers(&src, Numbering::new(10, 100), None).err().unwrap();
    assert_eq!(e[0].code, "B192");
    assert!(to_numbers(&src, Numbering::new(10, 91), None).is_ok());
    // die Nummern der Zeilen mit eigener Nummer zählen nicht mit
    assert!(prg("10 PRINT\n20 PRINT\n", 63990, 100).is_ok());
}

#[test]
fn range_renumbering_uses_its_own_step() {
    let src = "10 A\n20 GOTO 40\n30 B\n40 C\n100 GOTO 20\n";
    assert_eq!(to_numbers(src, Numbering::new(21, 1), Some((20, 40))).unwrap(), "10 A\n21 GOTO 23\n22 B\n23 C\n100 GOTO 21\n");
    assert_eq!(to_numbers(src, Numbering::new(50, 5), Some((20, 40))).unwrap(), "10 A\n50 GOTO 60\n55 B\n60 C\n100 GOTO 50\n");
    assert_eq!(to_numbers(src, Numbering::new(95, 10), Some((20, 40))).err().unwrap()[0].code, "B161", "würde die Reihenfolge ändern");
    assert_eq!(to_numbers(src, Numbering::new(63999, 10), Some((20, 40))).err().unwrap()[0].code, "B161", "zuerst fällt die Reihenfolge auf");
}

#[test]
fn directive_line_is_ignored_by_the_editor_preview() {
    assert_eq!(vc_basic::auto_line_numbers("#numbering start=100 step=5\nPRINT\nPRINT\n", 100, 5), [None, Some(100), Some(105)]);
}

/// Zeilen mit Umlauten an der Schnittstelle der Direktivenerkennung dürfen nicht abstürzen (Troxx-Beispiele)
#[test]
fn directive_detection_survives_multibyte_text() {
    for l in ["REM Spielfeld löschen", "#bläh x", "ä#numbering", "10 PRINT \"ÄÖÜ\"", "ÄÖÜÄÖ", "; äöüäö"] {
        assert!(!vc_basic::numbering::is_directive(l), "{l}");
        assert!(!vc_basic::blob::is_blob_directive(l), "{l}");
    }
}
