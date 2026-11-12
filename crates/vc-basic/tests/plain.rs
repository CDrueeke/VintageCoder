// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_basic::{compile_plain_atari, compile_plain_cbm, number_lines, tokenize_text, build_prg, Dialect, PlainOptions};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn opts() -> PlainOptions { PlainOptions::new("t.bas", 63999) }

#[test]
fn numbered_text_passes_through_unchanged() {
    let src = "10 print \"hi\"\n20 goto 10\n";
    let n = number_lines(src, &opts()).unwrap();
    assert_eq!(n.text, src);
    assert!(n.labels.is_empty());
}

#[test]
fn unnumbered_lines_get_numbers_and_labels_are_resolved() {
    let src = "; Kommentar\nprint \"start\"\n@loop:\nprint \"x@nicht\"\ngoto @loop\nif a=1 then @fertig\nrem @ignoriert\ndata @roh:1,2 : on a gosub @loop,@fertig\n@fertig: end\n";
    let n = number_lines(src, &opts()).unwrap();
    // Labelzeilen und Kommentare bleiben leer (Zeilenangaben stimmen mit der Quelle überein)
    let want = "\n10 print \"start\"\n\n20 print \"x@nicht\"\n30 goto 20\n40 if a=1 then 70\n50 rem @ignoriert\n60 data @roh:1,2 : on a gosub 20,70\n70 end\n";
    assert_eq!(n.text, want, "{}", n.text);
    assert_eq!(n.labels, vec![("loop".to_string(), 20), ("fertig".to_string(), 70)]);
}

#[test]
fn explicit_numbers_mix_with_automatic_ones() {
    let n = number_lines("100 a=1\nb=2\n200 c=3\nd=4\n", &opts()).unwrap();
    assert_eq!(n.text, "100 a=1\n110 b=2\n200 c=3\n210 d=4\n");
    let e = number_lines("100 a=1\n50 b=2\n", &opts()).unwrap_err();
    assert!(e[0].message.contains("nicht größer"), "{:?}", e[0].message);
}

#[test]
fn label_errors_are_reported_with_source_line() {
    let e = number_lines("goto @weg\n@a:\n@a: end\n", &opts()).unwrap_err();
    let msgs: Vec<(usize, String)> = e.iter().map(|d| (d.line, d.message.clone())).collect();
    assert!(msgs.iter().any(|(l, m)| *l == 1 && m.contains("@weg")), "{msgs:?}");
    assert!(msgs.iter().any(|(l, m)| *l == 3 && m.contains("doppelt")), "{msgs:?}");
    let e = number_lines("print 1\n@ende:\n", &opts()).unwrap_err();
    assert!(e[0].message.contains("ohne Anweisung"));
}

#[test]
fn cbm_labels_give_the_same_prg_as_hand_numbered_source() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let with_labels = "print \"{clr}hallo\"\n@l:\na=a+1:print a\nif a<10 then @l\nend\n";
    let numbered = "10 print \"{clr}hallo\"\n20 a=a+1:print a\n30 if a<10 then 20\n40 end\n";
    let c = compile_plain_cbm(with_labels, &d, &opts(), 0x0801).unwrap();
    assert_eq!(c.prg, build_prg(&tokenize_text(numbered, &d, "n.bas").unwrap(), 0x0801));
    assert_eq!(c.labels, vec![("l".to_string(), 20)]);
    assert!(c.listing.to_ascii_uppercase().contains("30 IF A<10 THEN 20"), "{}", c.listing);
}

#[test]
fn atari_labels_give_the_same_file_as_hand_numbered_source() {
    let d = vc_basic::atari::AtariDialect::load(&data(), "atari_basic").unwrap();
    let o = PlainOptions::new("t.bas", 32767);
    let a = compile_plain_atari("graphics 0\n@loop:\nposition 2,2:? \"HI\"\ngoto @loop\n", &d, &o).unwrap();
    let b = vc_basic::atari::tokenize_text("10 graphics 0\n20 position 2,2:? \"HI\"\n30 goto 20\n", &d, "n.bas").unwrap().save_file(&d);
    assert_eq!(a.file, b);
}

#[test]
fn auto_line_numbers_preview_matches_the_real_numbering() {
    let src = "; Kommentar\nprint 1\n@loop:\nprint 2\n\n100 print 3\nprint 4\n@a: @b: goto @loop\n";
    let pre = vc_basic::auto_line_numbers(src, 10, 10);
    assert_eq!(pre, vec![None, Some(10), None, Some(20), None, None, Some(110), Some(120)]);
    let n = number_lines(src, &opts()).unwrap();
    let real: Vec<Option<u32>> = n.text.lines().map(|l| l.split(' ').next().and_then(|d| d.parse().ok())).collect();
    for (p, r) in pre.iter().zip(&real) { if let Some(p) = p { assert_eq!(Some(*p), *r); } }
}
