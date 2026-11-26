// SPDX-License-Identifier: GPL-3.0-or-later
//! Syntaxprüfung (Stufe 1 und 2): je Code ein positiver und ein negativer Test, Abschalten, Einbindung in den Bau.
use std::path::{Path, PathBuf};
use vc_basic::check::{check_cbm, CheckOptions};
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn run(src: &str, id: &str) -> Vec<(&'static str, usize)> {
    let d = Dialect::load(&data(), id).unwrap();
    check_cbm(src, "t.bas", &d, &CheckOptions::default()).iter().map(|x| (x.code, x.line)).collect()
}
fn codes(src: &str, id: &str) -> Vec<&'static str> { run(src, id).into_iter().map(|c| c.0).collect() }
fn has(src: &str, code: &str) -> bool { codes(src, "cbm_v2").contains(&code) }

#[test]
fn b150_keyword_in_name() {
    assert!(has("10 SCORE=5\n", "B150"));
    assert!(has("10 TOTAL=5\n", "B150"));
    assert!(has("10 ORDER=1\n", "B150"));
    // gewöhnlicher kompakter Code ist in Ordnung
    for ok in ["10 FORI=1TO10:NEXTI\n", "10 IFA=1THENPRINTA\n", "10 PRINTA$:GOTO10\n", "10 A=PEEK(53280)AND15\n", "10 ONXGOTO20,30\n20 END\n30 END\n"] { assert!(!has(ok, "B150"), "{ok}"); }
    // mit Leerzeichen und in Zeichenketten nicht
    assert!(!has("10 SC = 1: S$=\"SCORE\"\n", "B150"));
    // in #longnames sind Namen ganze Wörter
    assert!(!has("#longnames\n10 score=5\n", "B150"));
}

#[test]
fn b151_for_next() {
    assert_eq!(codes("10 FOR I=1 TO 3\n20 PRINT I\n", "cbm_v2"), ["B151"]);
    assert_eq!(codes("10 NEXT I\n", "cbm_v2"), ["B151"]);
    assert!(codes("10 FOR I=1 TO 3:FOR J=1 TO 2\n20 NEXT J,I\n", "cbm_v2").is_empty());
    assert!(codes("10 FOR I=1 TO 3\n20 NEXT\n", "cbm_v2").is_empty());
    assert_eq!(run("10 FOR I=1 TO 3\n20 FOR J=1 TO 2\n30 NEXT J\n", "cbm_v2"), [("B151", 1)], "I bleibt offen");
    assert_eq!(codes("10 FOR I=1 TO 3\n20 NEXT K\n", "cbm_v2"), ["B151", "B151"], "NEXT K ohne FOR K, FOR I offen");
}

#[test]
fn b152_gosub_return() {
    assert!(has("10 GOSUB 100\n20 END\n100 PRINT\n", "B152"));
    assert!(has("10 PRINT\n20 RETURN\n", "B152"));
    assert!(!has("10 GOSUB 100\n20 END\n100 RETURN\n", "B152"));
}

#[test]
fn b153_missing_targets_only_in_number_mode() {
    assert_eq!(run("10 GOTO 20\n30 END\n", "cbm_v2"), [("B153", 1)]);
    assert_eq!(run("10 IF A THEN 99\n20 ON A GOTO 10,20,77\n", "cbm_v2"), [("B153", 1), ("B153", 2)]);
    assert!(codes("10 GOTO 20\n20 END\n", "cbm_v2").is_empty());
    assert!(codes("GOTO 20\nEND\n", "cbm_v2").is_empty(), "im Labelmodus gibt es keine feste Nummer");
    assert!(codes("10 GOTO 10+X\n", "cbm_v2").is_empty(), "berechnet");
}

#[test]
fn b154_variables_not_told_apart() {
    assert_eq!(run("10 COUNT=1\n20 COUNTER=2\n", "cbm_v2"), [("B154", 2)]);
    assert!(codes("10 COUNT=1\n20 COUNT=COUNT+1\n", "cbm_v2").is_empty(), "derselbe Name");
    assert!(codes("10 AB=1\n20 AC=2\n", "cbm_v2").is_empty(), "höchstens zwei Zeichen: unterscheidbar");
    assert!(codes("10 CNT$=\"A\"\n20 CNT=1\n", "cbm_v2").is_empty(), "Typkennzeichen unterscheidet");
    assert!(codes("#longnames\n10 counter=1\n20 count=2\n", "cbm_v2").is_empty());
    // BASIC 65 beachtet ebenfalls zwei Zeichen
    assert!(codes("10 AAA=1\n20 AAB=2\n", "basic65").contains(&"B154"));
}

#[test]
fn b155_open_string_hint() {
    assert_eq!(codes("10 PRINT \"HALLO\n", "cbm_v2"), ["B155"]);
    assert!(codes("10 PRINT \"HALLO\"\n", "cbm_v2").is_empty());
}

#[test]
fn b156_argument_count() {
    assert_eq!(codes("10 POKE 53280\n", "cbm_v2"), ["B156"]);
    assert_eq!(codes("10 POKE 1,2,3\n", "cbm_v2"), ["B156"]);
    assert!(codes("10 POKE 1,2:POKE 3,4\n", "cbm_v2").is_empty());
    assert!(codes("10 POKE A(1,2),3\n", "cbm_v2").is_empty(), "Komma in Klammern");
    assert_eq!(codes("10 A$=LEFT$(B$)\n", "cbm_v2"), ["B156"]);
    assert!(codes("10 A$=MID$(B$,2)+MID$(B$,2,3)\n", "cbm_v2").is_empty());
    assert!(codes("10 OPEN 1,8,15,\"I\"\n20 LOAD\"X\",8,1\n30 SYS 49152\n", "cbm_v2").is_empty());
    assert!(codes("10 IF A THEN POKE 1,2 ELSE POKE 3,4\n", "basic65").is_empty());
    assert_eq!(codes("10 POKE 1\n", "basic65"), ["B156"]);
    assert!(codes("10 POKE 1\n", "basic35").is_empty(), "keine syntax-Angaben");
}

#[test]
fn b157_statement_of_another_dialect() {
    assert_eq!(codes("10 GRAPHIC 1,1\n", "cbm_v2"), ["B157"]);
    assert_eq!(codes("10 IF A THEN SCNCLR\n", "cbm_v2"), ["B157"]);
    assert!(codes("10 GRAPHIC 1,1\n", "basic65").is_empty());
    assert!(codes("10 PRINT \"GRAPHIC\"\n20 REM GRAPHIC\n", "cbm_v2").is_empty());
    assert!(codes("10 COLOR=5\n", "cbm_v2").iter().all(|c| *c != "B157"), "Zuweisung an eine Variable");
}

#[test]
fn b159_function_without_parentheses() {
    assert_eq!(codes("10 A=RND\n", "cbm_v2"), ["B159"]);
    assert!(codes("10 A=RND(1)+ABS(-1)+LEN(\"X\")\n", "cbm_v2").is_empty());
    assert_eq!(codes("10 PRINT SPC(1,2)\n", "cbm_v2"), ["B156"]);
}

#[test]
fn b167_quote_is_no_comment_for_commodore() {
    assert_eq!(codes("10 PRINT 1 ' ERKLAERUNG\n", "cbm_v2"), ["B167"]);
    assert!(codes("10 PRINT \"IT'S\"\n", "cbm_v2").is_empty());
    assert!(codes("10 REM IT'S\n", "cbm_v2").is_empty());
}

#[test]
fn disabling_and_positions() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "10 SCORE=5\n20 PRINT \"X\n";
    let all = check_cbm(src, "t.bas", &d, &CheckOptions::default());
    assert_eq!(all.iter().map(|x| x.code).collect::<Vec<_>>(), ["B150", "B155"]);
    let r = &all[0].range;
    assert_eq!((r.line, r.col, r.end_col), (1, 4, 9), "Spalten 1-basiert auf der Quellzeile: SCORE");
    let o = CheckOptions { disabled: vec!["b150".into()], off: false };
    assert_eq!(check_cbm(src, "t.bas", &d, &o).iter().map(|x| x.code).collect::<Vec<_>>(), ["B155"]);
    assert!(check_cbm(src, "t.bas", &d, &CheckOptions { off: true, ..Default::default() }).is_empty());
    assert!(all.iter().all(|x| !x.is_error()), "die Prüfung meldet nur Warnungen");
}

#[test]
fn build_reports_warnings_and_can_switch_them_off() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let r = compile_plain_cbm("10 SCORE=5\n", &d, &PlainOptions::new("t.bas", 63999), d.load_addr).unwrap();
    assert!(r.warnings.iter().any(|w| w.code == "B150"));
    let mut o = PlainOptions::new("t.bas", 63999);
    o.check.off = true;
    assert!(compile_plain_cbm("10 SCORE=5\n", &d, &o, d.load_addr).unwrap().warnings.is_empty());
}

#[test]
fn examples_are_clean() {
    let ex = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for (f, id) in [("c64/basic_native/sterne.bas", "cbm_v2"), ("c64/basic_langenamen/punkte.bas", "cbm_v2"), ("pet/basic_hello/hello.bas", "cbm_v2")]  {
        let Ok(src) = std::fs::read_to_string(ex.join(f)) else { continue };
        let d = Dialect::load(&data(), id).unwrap();
        let ds = check_cbm(&src, f, &d, &CheckOptions::default());
        assert!(ds.is_empty(), "{f}: {:?}", ds.iter().map(|x| (x.code, x.line, x.message.clone())).collect::<Vec<_>>());
    }
}

#[test]
fn label_names_are_not_variables() {
    // `@hauptschleife` enthält IF, `@weiter` enthält ITE …: Label, keine Variablen
    assert!(codes("@hauptschleife: PRINT 1\nGOTO @hauptschleife\nIF A THEN @neustart\n@neustart: END\n", "cbm_v2").is_empty());
    assert!(codes("@count: A=1\n@counter: B=2\n", "cbm_v2").is_empty(), "Labels zählen nicht für B154");
}
