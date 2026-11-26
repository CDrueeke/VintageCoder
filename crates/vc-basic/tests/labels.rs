// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeilennummern ↔ Labels: Umwandlung, Gedächtnis (Markerzeilen), Neu-Nummerieren, Modus, Bedeutungstreue (byte-gleiche Programme).
use std::path::{Path, PathBuf};
use vc_basic::dialects::{self, AnyDialect, DialectInfo};
use vc_basic::labels::{self, Cx, Mode, Origin, Unit};
use vc_basic::{compile_plain_atari, compile_plain_cbm, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn dia(id: &str) -> (AnyDialect, DialectInfo) { (AnyDialect::load(&data(), id).unwrap(), dialects::info(&data(), id).unwrap()) }
fn nl(src: &str, id: &str) -> Result<String, Vec<vc_core::Diagnostic>> { let (d, i) = dia(id); labels::to_labels_text(src, &Cx::new(&d, &i)) }
fn nn(src: &str, id: &str) -> Result<String, Vec<vc_core::Diagnostic>> { let (d, i) = dia(id); labels::to_numbers_text(src, &Cx::new(&d, &i)) }
fn codes(r: &Result<String, Vec<vc_core::Diagnostic>>) -> Vec<&'static str> { r.as_ref().err().map(|e| e.iter().map(|d| d.code).collect()).unwrap_or_default() }

const LOOP: &str = "10 REM HAUPTSCHLEIFE\n20 FOR I=1 TO 10\n30 PRINT I\n40 NEXT I\n50 GOTO 20\n";

#[test]
fn numbers_to_labels_example_from_the_concept() {
    // Anhang B: nur die angesprungene Zeile bekommt ein Label, der Name stammt aus dem Kommentar davor; REM bleibt Programmzeile
    assert_eq!(nl(LOOP, "cbm_v2").unwrap(), "REM HAUPTSCHLEIFE\n@hauptschleife: FOR I=1 TO 10\nPRINT I\nNEXT I\nGOTO @hauptschleife\n");
}

#[test]
fn labels_to_numbers_remembers_names_as_markers() {
    let l = nl(LOOP, "cbm_v2").unwrap();
    assert_eq!(nn(&l, "cbm_v2").unwrap(), "10 REM HAUPTSCHLEIFE\n;@hauptschleife\n20 FOR I=1 TO 10\n30 PRINT I\n40 NEXT I\n50 GOTO 20\n");
}

#[test]
fn manual_names_survive_any_number_of_switches() {
    let src = "10 PRINT \"A\"\n;@schleife\n20 FOR I=1 TO 3\n30 NEXT I\n40 GOTO 20\n;@ende\n50 END\n";
    // `ende` wird nicht angesprungen, bleibt aber erhalten, weil der Name manuell vergeben ist
    let mut t = src.to_string();
    for _ in 0..3 {
        let l = nl(&t, "cbm_v2").unwrap();
        assert!(l.contains("@schleife: FOR I=1 TO 3") && l.contains("@ende: END") && l.contains("GOTO @schleife"), "{l}");
        t = nn(&l, "cbm_v2").unwrap();
    }
    assert_eq!(t, "10 PRINT \"A\"\n;@schleife\n20 FOR I=1 TO 3\n30 NEXT I\n40 GOTO 20\n;@ende\n50 END\n");
}

#[test]
fn existing_labels_are_never_changed_and_names_not_reused() {
    let src = ";@start\n10 PRINT 1\n20 PRINT 2\n;@z300\n30 PRINT 3\n300 GOTO 10\n";
    // Marker `start` (manuell) bleibt; die Zeile 20 wird nicht angesprungen; Zeile 30 heißt schon z300
    let src = src.replace("300 GOTO 10", "40 GOTO 10\n50 GOTO 30");
    let (d, i) = dia("cbm_v2");
    let c = labels::to_labels(&[Unit { file: "a.bas".into(), text: src }], &Cx::new(&d, &i)).unwrap();
    let t = &c.units[0].text;
    assert!(t.contains("@start: PRINT 1") && t.contains("@z300: PRINT 3") && t.contains("GOTO @start") && t.contains("GOTO @z300"), "{t}");
    assert!(c.new_labels.is_empty(), "nichts automatisch erzeugt: {:?}", c.new_labels);
    assert_eq!(c.kept_labels.len(), 2);
    // neu erzeugter Name darf einen vorhandenen nicht doppeln
    let c2 = labels::to_labels(&[Unit { file: "a.bas".into(), text: ";@z20\n10 PRINT 1\n20 GOTO 10\n30 GOTO 20\n".into() }], &Cx::new(&d, &i)).unwrap();
    assert_eq!(c2.units[0].text, "@z20: PRINT 1\n@z20_2: GOTO @z20\nGOTO @z20_2\n");
    assert_eq!(c2.new_labels[0].name, "z20_2");
    assert_eq!(c2.new_labels[0].origin, Origin::Auto);
}

#[test]
fn computed_targets_abort_the_conversion() {
    for line in ["20 GOTO A*10", "20 ON X GOTO 10,A", "20 GOSUB X", "20 GOTO 10+X", "20 IF A THEN GOTO PEEK(5)", "20 GOTO (X)"] {
        let r = nl(&format!("10 PRINT\n{line}\n"), "cbm_v2");
        assert_eq!(codes(&r), ["B161", "B162"], "{line}: {:?}", r.as_ref().err().map(|e| e.iter().map(|d| d.message.clone()).collect::<Vec<_>>()));
    }
    // Zeichenketten, REM und DATA enthalten keine Sprünge
    let t = nl("10 PRINT \"GOTO X\":REM GOTO X\n20 DATA GOTO,GOSUB\n30 GOTO 10\n", "cbm_v2").unwrap();
    assert!(t.contains("@z10: PRINT \"GOTO X\":REM GOTO X") && t.contains("DATA GOTO,GOSUB") && t.contains("GOTO @z10"), "{t}");
    // berechnet beim Neu-Nummerieren: nur Warnung
    let (d, i) = dia("cbm_v2");
    let c = labels::renumber(&[Unit { file: "a".into(), text: "10 GOTO A*10\n".into() }], &Cx::new(&d, &i), None).unwrap();
    assert_eq!(c.warnings[0].code, "B162");
}

#[test]
fn all_jump_forms_are_recognised() {
    let src = "10 PRINT\n20 IF A THEN 10\n30 ON A GOTO 10,20,30\n40 GOSUB 10:GOTO 20\n50 PRINT \"X\"\n60 IF A THEN PRINT 1:GOTO 50\n";
    let t = nl(src, "cbm_v2").unwrap();
    assert!(t.contains("THEN @z10") && t.contains("ON A GOTO @z10,@z20,@z30") && t.contains("GOSUB @z10:GOTO @z20") && t.contains("PRINT 1:GOTO @z50"), "{t}");
    // BASIC 7/65: ELSE, RESTORE, TRAP, RESUME, GO TO
    let src = "10 DATA 1\n20 TRAP 90\n30 RESTORE 10\n40 IF A THEN 10 ELSE 30\n50 GO TO 40\n60 RESUME NEXT\n70 RUN\n90 PRINT\n";
    let t = nl(src, "basic65").unwrap();
    assert!(t.contains("TRAP @z90") && t.contains("RESTORE @z10") && t.contains("THEN @z10 ELSE @z30") && t.contains("GO TO @z40") && t.contains("RESUME NEXT"), "{t}");
}

#[test]
fn missing_targets_are_warned_and_left() {
    let (d, i) = dia("cbm_v2");
    let c = labels::to_labels(&[Unit { file: "a".into(), text: "10 GOTO 999\n".into() }], &Cx::new(&d, &i)).unwrap();
    assert_eq!(c.warnings[0].code, "B153");
    assert!(c.units[0].text.contains("GOTO 999"));
}

#[test]
fn renumber_adjusts_targets_and_keeps_markers_with_their_lines() {
    let (d, i) = dia("cbm_v2");
    let src = ";@a\n5 PRINT 1\n7 GOTO 5\n100 GOSUB 7\n";
    let c = labels::renumber(&[Unit { file: "a".into(), text: src.into() }], &Cx::new(&d, &i).numbering(100, 5), None).unwrap();
    assert_eq!(c.units[0].text, ";@a\n100 PRINT 1\n105 GOTO 100\n110 GOSUB 105\n");
    // Bereich
    let c = labels::renumber(&[Unit { file: "a".into(), text: "10 A\n20 GOTO 40\n30 B\n40 C\n100 GOTO 20\n".into() }], &Cx::new(&d, &i).numbering(21, 1), Some((20, 40))).unwrap();
    assert_eq!(c.units[0].text, "10 A\n21 GOTO 23\n22 B\n23 C\n100 GOTO 21\n");
    let c = labels::renumber(&[Unit { file: "a".into(), text: "10 A\n20 B\n30 C\n".into() }], &Cx::new(&d, &i).numbering(11, 1), Some((20, 30))).unwrap();
    assert_eq!(c.units[0].text, "10 A\n11 B\n12 C\n");
    let e = labels::renumber(&[Unit { file: "a".into(), text: "10 A\n20 B\n30 C\n".into() }], &Cx::new(&d, &i).numbering(10, 1), Some((20, 30)));
    assert_eq!(e.err().unwrap()[0].code, "B161");
    // Höchstnummer
    let e = labels::renumber(&[Unit { file: "a".into(), text: "10 A\n20 B\n".into() }], &Cx::new(&d, &i).numbering(63990, 100), None);
    assert_eq!(e.err().unwrap()[0].code, "B192");
}

#[test]
fn mode_detection_and_diagnostics() {
    let (d, i) = dia("cbm_v2");
    let cx = Cx::new(&d, &i);
    assert_eq!(labels::detect_mode_text("10 PRINT\n20 GOTO 10\n", &cx), Mode::Numbers);
    assert_eq!(labels::detect_mode_text("@a: PRINT\nGOTO @a\n", &cx), Mode::Labels);
    assert_eq!(labels::detect_mode_text("10 PRINT\nGOTO @a\n", &cx), Mode::Mixed);
    assert_eq!(labels::detect_mode_text("; nur Kommentar\n#longnames\n", &cx), Mode::Undetermined);
    assert_eq!(labels::detect_mode_text("PRINT \"@a\"\nREM @x\n", &cx), Mode::Undetermined, "in Zeichenketten und REM nichts");
    let ds = labels::check_modes(&[Unit { file: "m.bas".into(), text: "10 PRINT\n@a: PRINT\nGOTO @a\n".into() }], &cx);
    assert_eq!(ds.iter().map(|d| d.code).collect::<Vec<_>>(), ["B160"]);
    assert_eq!(ds[0].line, 2);
    let ds = labels::check_modes(&[Unit { file: "m.bas".into(), text: ";@a\n10 PRINT\n;@a\n20 PRINT\n;@1x\n;@verwaist\n".into() }], &cx);
    assert_eq!(ds.iter().map(|d| d.code).collect::<Vec<_>>(), ["B168", "B168", "B165"], "doppelt, ungültig, verwaist");
}

#[test]
fn runtime_line_warning() {
    let (d, i) = dia("basic65");
    let c = labels::to_labels(&[Unit { file: "a".into(), text: "10 PRINT EL\n20 GOTO 10\n".into() }], &Cx::new(&d, &i)).unwrap();
    assert!(c.warnings.iter().any(|w| w.code == "B163"));
    let (d, i) = dia("cbm_v2");
    let c = labels::to_labels(&[Unit { file: "a".into(), text: "10 PRINT EL\n20 GOTO 10\n".into() }], &Cx::new(&d, &i)).unwrap();
    assert!(c.warnings.iter().all(|w| w.code != "B163"), "BASIC V2 kennt EL nicht");
}

#[test]
fn indentation_moves_with_the_line() {
    assert_eq!(nl("10 FOR I=1 TO 3\n20    PRINT I\n30 NEXT I\n", "cbm_v2").unwrap(), "FOR I=1 TO 3\n   PRINT I\nNEXT I\n");
    assert_eq!(nn("FOR I=1 TO 3\n   PRINT I\nNEXT I\n", "cbm_v2").unwrap(), "10 FOR I=1 TO 3\n20    PRINT I\n30 NEXT I\n");
}

#[test]
fn includes_share_labels_and_numbers() {
    let dir = std::env::temp_dir().join(format!("vc_lbl_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/u.bas"), "@unter: PRINT \"U\"\nRETURN\n").unwrap();
    let main = dir.join("m.bas").to_string_lossy().into_owned();
    let text = "GOSUB @unter\n#include \"lib/u.bas\"\nGOTO @unter\n";
    let units = labels::load_set(&main, text).unwrap();
    assert_eq!(units.len(), 2);
    let (d, i) = dia("cbm_v2");
    let c = labels::to_numbers(&units, &Cx::new(&d, &i), None).unwrap();
    assert_eq!(c.units[0].text, "10 GOSUB 20\n#include \"lib/u.bas\"\n40 GOTO 20\n");
    assert_eq!(c.units[1].text, ";@unter\n20 PRINT \"U\"\n30 RETURN\n");
    assert!(c.changes.iter().any(|ch| ch.file.ends_with("u.bas")));
    // zurück
    let back = labels::to_labels(&c.units, &Cx::new(&d, &i)).unwrap();
    assert_eq!(back.units[0].text, text);
    assert_eq!(back.units[1].text, "@unter: PRINT \"U\"\nRETURN\n");
    // Modi
    let ds = labels::check_modes(&[Unit { file: main.clone(), text: "10 PRINT\n#include \"lib/u.bas\"\n".into() }, Unit { file: "lib/u.bas".into(), text: "@a: PRINT\n".into() }], &Cx::new(&d, &i));
    assert!(ds.iter().any(|x| x.code == "B166"));
    // fehlende Datei
    assert_eq!(labels::load_set(&main, "#include \"fehlt.bas\"\n").err().unwrap()[0].code, "B120");
    let _ = std::fs::remove_dir_all(&dir);
}

// ───────── Bedeutungstreue: dasselbe Programm vor und nach der Umwandlung ─────────

fn prg(src: &str, id: &str) -> Vec<u8> {
    let d = vc_basic::Dialect::load(&data(), id).unwrap();
    compile_plain_cbm(src, &d, &PlainOptions::new("t.bas", 63999), d.load_addr).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).prg
}
fn sav(src: &str, id: &str) -> Vec<u8> {
    let d = vc_basic::atari::AtariDialect::load(&data(), id).unwrap();
    compile_plain_atari(src, &d, &PlainOptions::new("t.bas", 32767)).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).file
}
fn build(src: &str, id: &str) -> Vec<u8> { if dia(id).1.family == "atari" { sav(src, id) } else { prg(src, id) } }

fn roundtrip(src: &str, id: &str) {
    let original = build(src, id);
    let n = nn(src, id).unwrap();
    assert_eq!(build(&n, id), original, "Labels → Nummern ändert das Programm ({id})\n{n}");
    let l = nl(&n, id).unwrap();
    assert_eq!(build(&l, id), original, "Nummern → Labels ändert das Programm ({id})\n{l}");
    assert_eq!(nn(&l, id).unwrap(), n, "Rundlauf nicht stabil ({id})");
    assert_eq!(labels::detect_mode_text(&n, &Cx::new(&dia(id).0, &dia(id).1)), Mode::Numbers);
    assert_ne!(labels::detect_mode_text(&l, &Cx::new(&dia(id).0, &dia(id).1)), Mode::Mixed);
    assert_ne!(labels::detect_mode_text(&l, &Cx::new(&dia(id).0, &dia(id).1)), Mode::Numbers);
}

#[test]
fn examples_run_identically_after_conversion() {
    let ex = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let cases = [("c64/basic_native/sterne.bas", "cbm_v2"), ("c64/basic_langenamen/punkte.bas", "cbm_v2"), ("pet/basic_hello/hello.bas", "cbm_v2"),
                 ("atari/basic_native/farben.bas", "atari_basic"), ("atari/turbo_basic/balken.tur", "turbo_basic_xl")];
    for (f, id) in cases {
        let Ok(src) = std::fs::read_to_string(ex.join(f)) else { continue };
        roundtrip(&src, id);
    }
}

#[test]
fn synthetic_programs_roundtrip_in_every_dialect() {
    let prog = "; Haupt\nREM START\nPRINT \"A\"\n@loop: FOR I=1 TO 3\nON I GOSUB @a,@b\nNEXT I\nIF X THEN @ende\nGOTO @loop\n@a: PRINT 1: RETURN\n@b: PRINT 2: RETURN\n@ende: END\n";
    for id in ["cbm_v2", "vic_basic_2", "basic35", "basic7", "basic65"] { roundtrip(prog, id); }
    let atari = "GRAPHICS 0\n@loop: FOR I=1 TO 3\nON I GOSUB @a,@b\nNEXT I\nIF X THEN @ende\nTRAP @ende\nGOTO @loop\n@a: PRINT 1: RETURN\n@b: PRINT 2: RETURN\n@ende: END\n";
    for id in ["atari_basic", "turbo_basic_xl"] { roundtrip(atari, id); }
}

#[test]
fn irregular_numbering_gives_the_renumbered_program() {
    // Nummern 7, 9, 200: die Rückumwandlung nummeriert nach Projektvorgabe neu (B6); gleich dem direkten Neu-Nummerieren
    let src = "7 PRINT 1\n9 GOTO 200\n200 PRINT 2\n210 GOTO 7\n";
    let (d, i) = dia("cbm_v2");
    let cx = Cx::new(&d, &i);
    let direct = labels::to_numbers_text(src, &cx).unwrap();
    let via = nn(&nl(src, "cbm_v2").unwrap(), "cbm_v2").unwrap();
    assert_eq!(prg(&via, "cbm_v2"), prg(&direct, "cbm_v2"));
    assert_eq!(direct, "10 PRINT 1\n20 GOTO 30\n30 PRINT 2\n40 GOTO 10\n");
}
