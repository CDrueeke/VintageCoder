// SPDX-License-Identifier: GPL-3.0-or-later
//! Einrückung (nur im Quelltext, beim Bauen entfernt) und Bauoption „kompakt“ (Leerzeichen innerhalb der Anweisung, mit Prüfung).
use std::path::{Path, PathBuf};
use vc_basic::compact::{compact_line, same_tokens};
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn build(src: &str, id: &str, compact: bool) -> Vec<u8> {
    let d = Dialect::load(&data(), id).unwrap();
    let mut o = PlainOptions::new("t.bas", 63999); o.compact = compact;
    compile_plain_cbm(src, &d, &o, d.load_addr).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).prg
}

#[test]
fn indentation_is_removed_when_building() {
    let plain = "10 FOR I=1 TO 3\n20 PRINT I\n30 NEXT I\n";
    let indented = "10 FOR I=1 TO 3\n20    PRINT I\n30 NEXT I\n";
    assert_eq!(build(plain, "cbm_v2", false), build(indented, "cbm_v2", false));
    let labels = "@a: FOR I=1 TO 3\n    PRINT I\nNEXT I\n";
    let labels_flat = "@a: FOR I=1 TO 3\nPRINT I\nNEXT I\n";
    assert_eq!(build(labels, "cbm_v2", false), build(labels_flat, "cbm_v2", false));
    // auch hinter einer Zeilennummer mit Label
    assert_eq!(build("10 @x:    PRINT 1\n", "cbm_v2", false), build("10 PRINT 1\n", "cbm_v2", false));
}

#[test]
fn compact_removes_spaces_inside_statements_as_measured() {
    let spaced = "10 T = TI : FOR I = 1 TO 3000 : A = A + 1 : B = A * 2 : NEXT I : PRINT \"JIFFIES\" ; TI - T\n";
    let compact_by_hand = "10 T=TI:FORI=1TO3000:A=A+1:B=A*2:NEXTI:PRINT\"JIFFIES\";TI-T\n";
    assert_eq!(build(spaced, "cbm_v2", true), build(compact_by_hand, "cbm_v2", false));
    assert_eq!(build(spaced, "cbm_v2", false).len() - build(spaced, "cbm_v2", true).len(), 31, "jedes entfernte Leerzeichen spart ein Byte (Messung B0: 84 → 54 Bytes)");
    // Standard ist aus
    assert_ne!(build(spaced, "cbm_v2", false), build(spaced, "cbm_v2", true));
}

#[test]
fn compact_keeps_strings_rem_data_and_glued_keywords() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    assert_eq!(compact_line("PRINT \"A  B\" : A$ = \"X Y\"", &d), "PRINT\"A  B\":A$=\"X Y\"");
    assert_eq!(compact_line("X = 1 : REM  zwei  Leerzeichen", &d), "X=1:REM  zwei  Leerzeichen");
    assert_eq!(compact_line("DATA 1, 2 ,3 : PRINT 5", &d), "DATA 1, 2 ,3 :PRINT5");
    assert_eq!(compact_line("GO TO 100", &d), "GO TO100", "GOTO wäre ein anderes Token: das Leerzeichen hinter GO bleibt");
    assert_eq!(compact_line("PRINT {clr} \"X\"", &d), "PRINT{clr}\"X\"");
}

#[test]
fn compact_never_changes_the_tokens() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    for src in ["IF A = 1 THEN PRINT \"JA\" ELSE_", "ON X GOTO 10 , 20 , 30", "FOR I = A TO B STEP - 1", "A = B OR C AND NOT D", "PRINT TAB( 5 ) ; SPC( 2 ) ; A$", "OPEN 1 , 8 , 15 , \"I\"", "X = INT ( RND ( 1 ) * 6 ) + 1", "DEF FN A ( X ) = X * 2", "S$ = LEFT$ ( A$ , 3 ) + MID$ ( A$ , 2 , 1 )", "IF A < B AND B > C OR D = E THEN 100"] {
        let orig = vc_basic::tokenize_line(src, &d).unwrap();
        let c = compact_line(src, &d);
        let new = vc_basic::tokenize_line(&c, &d).unwrap();
        assert!(same_tokens(&orig, &new, &d), "{src} → {c}");
        assert!(new.len() <= orig.len());
    }
}

#[test]
fn compact_examples_build_with_equal_tokens() {
    let ex = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for f in ["c64/basic_native/sterne.bas", "c64/basic_langenamen/punkte.bas", "pet/basic_hello/hello.bas"] {
        let Ok(src) = std::fs::read_to_string(ex.join(f)) else { continue };
        let d = Dialect::load(&data(), "cbm_v2").unwrap();
        let a = vc_basic::parse_prg(&build(&src, "cbm_v2", false)).unwrap();
        let b = vc_basic::parse_prg(&build(&src, "cbm_v2", true)).unwrap();
        assert_eq!(a.lines.len(), b.lines.len());
        for (x, y) in a.lines.iter().zip(&b.lines) { assert_eq!(x.0, y.0); assert!(same_tokens(&x.1, &y.1, &d), "{f} Zeile {}", x.0); }
        assert!(build(&src, "cbm_v2", true).len() <= build(&src, "cbm_v2", false).len());
    }
}
