// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenpass: `$hex` und `%binär` im nativen BASIC (Konzept „BASIC-Zahlenformate“, Stufe 0).
use std::path::PathBuf;
use vc_basic::numbers::{scan_line, Lexicon, NumberOptions};
use vc_basic::{compile_plain_atari, compile_plain_cbm, Dialect, PlainOptions};
use vc_basic::atari::AtariDialect;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn opts() -> PlainOptions { PlainOptions::new("t.bas", 63999) }
fn cbm(id: &str, src: &str) -> Result<vc_basic::PlainCbm, Vec<vc_core::Diagnostic>> { compile_plain_cbm(src, &Dialect::load(&data(), id).unwrap(), &opts(), 0x0801) }
fn atari(id: &str, src: &str) -> Result<vc_basic::PlainAtari, Vec<vc_core::Diagnostic>> { compile_plain_atari(src, &AtariDialect::load(&data(), id).unwrap(), &PlainOptions::new("t.bas", 32767)) }
fn codes(r: Result<impl Sized, Vec<vc_core::Diagnostic>>) -> Vec<&'static str> { match r { Ok(_) => vec![], Err(e) => e.iter().map(|d| d.code).collect() } }
/// Ausgabe von `convert_numbers` für CBM V2 (ohne die späteren Schritte)
fn conv(id: &str, src: &str) -> Result<String, Vec<vc_core::Diagnostic>> {
    let d = Dialect::load(&data(), id).unwrap();
    vc_basic::numbers::convert_numbers(src, &Lexicon::Cbm(&d), &NumberOptions::default(), "t.bas").map(|c| c.text)
}

#[test]
fn issue_example_becomes_decimal_and_builds_like_decimal() {
    let a = cbm("cbm_v2", "POKE $D020,%00001110\n").unwrap();
    let b = cbm("cbm_v2", "POKE 53280,14\n").unwrap();
    assert_eq!(a.prg, b.prg, "Wertgleichheit: dieselbe PRG");
    assert!(a.listing.to_ascii_uppercase().contains("POKE 53280,14"), "{}", a.listing);
    assert_eq!(a.numbers_converted, 2);
}

#[test]
fn all_cbm_dialects_get_decimal_by_default() {
    for id in ["cbm_v2", "basic35", "basic7", "basic65"] {
        let a = cbm(id, "A=$FF+%101\n").unwrap();
        let b = cbm(id, "A=255+5\n").unwrap();
        assert_eq!(a.prg, b.prg, "{id}");
    }
}

#[test]
fn basic65_can_keep_literals_as_typed() {
    let d = Dialect::load(&data(), "basic65").unwrap();
    let mut o = opts();
    o.numbers.native = Some(true);
    let r = compile_plain_cbm("A=$d0_20+%0000_1110\n", &d, &o, 0x2001).unwrap();
    assert!(r.listing.to_ascii_uppercase().contains("A=$D020+%00001110"), "{}", r.listing);
    // CBM V2 kennt keine Literale: die Einstellung ändert dort nichts
    let v2 = Dialect::load(&data(), "cbm_v2").unwrap();
    let r = compile_plain_cbm("A=$D020\n", &v2, &o, 0x0801).unwrap();
    assert!(r.listing.to_ascii_uppercase().contains("A=53280"), "{}", r.listing);
}

#[test]
fn lexical_table() {
    let rows: &[(&str, &str)] = &[
        ("10 PRINT $FF", "10 PRINT 255"),
        ("10 PRINT$FF", "10 PRINT 255"),                      // Schlüsselwort, dann Zahl (mit Leerzeichen)
        ("10 A$=\"X\":B%=1", "10 A$=\"X\":B%=1"),               // Typkennzeichen bleiben
        ("10 PRINT A$;$10;B%", "10 PRINT A$;16;B%"),
        ("10 X=PEEK($D020)AND%1111", "10 X=PEEK(53280)AND 15"),
        ("10 PRINT \"$FF %101\"", "10 PRINT \"$FF %101\""),   // Zeichenkette
        ("10 REM $FF %1", "10 REM $FF %1"),
        ("10 POKE 1,$1E3", "10 POKE 1,483"),                  // nach $ sind E und A–F Ziffern
        ("10 X=1E3+$F", "10 X=1E3+15"),
        ("10 X=-$FF", "10 X=-255"),
        ("10 POKE $d020,%1", "10 POKE 53280,1"),
        ("10 X=$D0_20", "10 X=53280"),
        ("10 X=%0000_1110", "10 X=14"),
        ("10 X=$FFFFFFFF", "10 X=4294967295"),
        ("10 X=%00000000000000000000000000000001", "10 X=1"),
    ];
    for (src, want) in rows { assert_eq!(conv("cbm_v2", &format!("{src}\n")).unwrap_or_else(|e| panic!("{src}: {e:?}")).trim_end(), *want, "{src}"); }
}

#[test]
fn data_elements_are_converted_but_text_is_not() {
    let t = conv("cbm_v2", "10 DATA $FF,%1010, 7 ,\"$5\",$5x,hallo:POKE $10,1\n").unwrap();
    assert_eq!(t.trim_end(), "10 DATA 255,10, 7 ,\"$5\",$5x,hallo:POKE 16,1");
    // Einstellung aus
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let o = NumberOptions { data: false, ..Default::default() };
    let c = vc_basic::numbers::convert_numbers("10 DATA $FF\n", &Lexicon::Cbm(&d), &o, "t").unwrap();
    assert_eq!(c.text.trim_end(), "10 DATA $FF");
}

#[test]
fn errors_have_codes() {
    assert_eq!(codes(cbm("cbm_v2", "POKE $,1\n")), vec!["B132"]);
    assert_eq!(codes(cbm("cbm_v2", "POKE %,1\n")), vec!["B132"]);
    assert_eq!(codes(cbm("cbm_v2", "X=$G\n")), vec!["B130"]);
    assert_eq!(codes(cbm("cbm_v2", "X=%102\n")), vec!["B131"]);
    assert_eq!(codes(cbm("cbm_v2", "X=%2\n")), vec!["B131"]);
    assert_eq!(codes(cbm("cbm_v2", "X=$FFFFFFFFF\n")), vec!["B133"]);
    assert_eq!(codes(cbm("cbm_v2", &format!("X=%{}\n", "1".repeat(33)))), vec!["B133"]);
    assert_eq!(codes(cbm("cbm_v2", "X=$_1\n")), vec!["B130"]);
    assert_eq!(codes(cbm("cbm_v2", "X=$1__2\n")), vec!["B130"]);
    assert_eq!(codes(cbm("cbm_v2", "GOTO $0100\n")), vec!["B138"]);
    assert_eq!(codes(cbm("cbm_v2", "ON X GOSUB 10,%101\n")), vec!["B138"]);
    assert_eq!(codes(cbm("cbm_v2", "IF A THEN $10\n")), vec!["B138"]);
    // Fehler tragen die Quellzeile
    let e = cbm("cbm_v2", "A=1\nB=2\nPOKE $G,1\n").err().unwrap();
    assert_eq!(e[0].line, 3);
}

#[test]
fn keyword_adjacent_hex_warns_and_0x_hints() {
    let r = cbm("cbm_v2", "X=$FFAND$0F\n");
    // `$FFA` + `ND`: ungültig (der Fall ist im Konzept beschrieben)
    assert!(r.is_err());
    let r = cbm("cbm_v2", "X=$FF OR$0F\nY=$10TO\n").unwrap_or_else(|e| panic!("{e:?}"));
    assert!(r.warnings.iter().all(|w| w.code == "B136"), "{:?}", r.warnings);
    let r = cbm("cbm_v2", "X=0xFF\n").unwrap();
    assert_eq!(r.warnings[0].code, "B139");
}

#[test]
fn line_targets_stay_decimal_and_numbering_is_unaffected() {
    let r = cbm("cbm_v2", "@a:\nPOKE $D020,%1\nGOTO @a\n").unwrap();
    let l = r.listing.to_ascii_uppercase();
    assert!(l.contains("10 POKE 53280,1") && l.contains("20 GOTO 10"), "{}", r.listing);
}

#[test]
fn literals_in_include_free_longnames_and_labels_work_together() {
    let r = cbm("cbm_v2", "#longnames\nrand = 3\nPOKE $D020,rand\nPOKE $D021,%1010\n").unwrap();
    assert!(r.listing.contains("53280") && r.listing.contains("53281,10"), "{}", r.listing);
}

#[test]
fn atari_hex_and_binary_become_decimal() {
    let a = atari("atari_basic", "POKE $2C8,%1110\n").unwrap();
    let b = atari("atari_basic", "POKE 712,14\n").unwrap();
    assert_eq!(a.file, b.file);
    // früher las Atari `%0`..`%3` als Konstante und `%00001110` still falsch: jetzt Binärzahl
    assert_eq!(atari("atari_basic", "X=%00001110\n").unwrap().file, atari("atari_basic", "X=14\n").unwrap().file);
    assert_eq!(codes(atari("atari_basic", "X=%2\n")), vec!["B131"]);
    assert_eq!(atari("atari_basic", "X=%1\n").unwrap().file, atari("atari_basic", "X=1\n").unwrap().file);
    assert_eq!(atari("atari_basic", "PRINT$FF\n").unwrap().file, atari("atari_basic", "PRINT 255\n").unwrap().file);
    assert_eq!(atari("atari_basic", "A$=STR$($10)\n").unwrap().file, atari("atari_basic", "A$=STR$(16)\n").unwrap().file);
}

#[test]
fn turbo_keeps_hex_and_converts_binary() {
    let a = atari("turbo_basic_xl", "POKE $2C8,%00001110\n").unwrap();
    assert!(a.listing.contains("$2C8") || a.listing.contains("$2c8"), "{}", a.listing);
    assert!(a.listing.contains("14"), "{}", a.listing);
    // Einzelziffern bleiben Konstanten (Token per0..per3), ab zwei Ziffern Binärzahl
    for (src, want) in [("X=%2\n", "X=%2"), ("X=%3\n", "X=%3"), ("X=%0\n", "X=%0"), ("X=%10\n", "X=2")] {
        let r = atari("turbo_basic_xl", src).unwrap();
        assert!(r.listing.contains(want), "{src}: {}", r.listing);
    }
    // Hex als Dezimalzahl, falls gewünscht
    let d = AtariDialect::load(&data(), "turbo_basic_xl").unwrap();
    let mut o = PlainOptions::new("t.bas", 32767);
    o.numbers.native = Some(false);
    let r = compile_plain_atari("X=$FF\n", &d, &o).unwrap();
    assert!(r.listing.contains("255"), "{}", r.listing);
}

#[test]
fn cbm_and_atari_scanners_agree_on_a_shared_corpus() {
    let cb = Dialect::load(&data(), "cbm_v2").unwrap();
    let at = AtariDialect::load(&data(), "atari_basic").unwrap();
    let o = NumberOptions::default();
    for line in ["10 POKE $D020,%1010", "10 A$=\"$FF\":PRINT $10;A$", "10 X=$FF+%11-A%", "10 DATA $1,%10,x", "10 IF X=$10 THEN PRINT %11"] {
        let a: Vec<_> = scan_line(line, &Lexicon::Cbm(&cb), &o).lits.iter().map(|l| (l.start, l.end, l.value)).collect();
        let b: Vec<_> = scan_line(line, &Lexicon::Atari(&at), &o).lits.iter().map(|l| (l.start, l.end, l.value)).collect();
        assert_eq!(a, b, "{line}");
    }
}

#[test]
fn fuzz_keeps_strings_and_comments_and_never_panics() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let alphabet: Vec<char> = "$%0123456789ABCDEFG_\" :,()-+".chars().collect();
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut next = || { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed };
    for _ in 0..3000 {
        let len = (next() % 24) as usize;
        let mut s: String = (0..len).map(|_| alphabet[(next() % alphabet.len() as u64) as usize]).collect();
        if next() % 3 == 0 { s.push_str(" REM $FF %1"); }
        let line = format!("10 {s}\n");
        if let Ok(c) = vc_basic::numbers::convert_numbers(&line, &Lexicon::Cbm(&d), &NumberOptions::default(), "t") {
            // Text in Anführungszeichen und Kommentare unverändert
            let strings = |t: &str| t.split('"').enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, x)| x.to_string()).collect::<Vec<_>>();
            if line.matches('"').count() % 2 == 0 { assert_eq!(strings(&line), strings(&c.text), "{line}"); }
            if let Some(p) = line.find("REM") { assert!(c.text.ends_with(&line[p..]), "{line} -> {}", c.text); }
        }
    }
}

fn warn_codes(r: Result<vc_basic::PlainCbm, Vec<vc_core::Diagnostic>>) -> Vec<&'static str> { r.unwrap_or_else(|e| panic!("{e:?}")).warnings.iter().map(|w| w.code).collect() }

#[test]
fn range_warnings_for_addresses_and_poke_values() {
    assert_eq!(warn_codes(cbm("cbm_v2", "POKE 70000,1\n")), vec!["B134"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "POKE $10000,1\n")), vec!["B134"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "POKE 53280,256\n")), vec!["B135"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "POKE 53280,%100000000\n")), vec!["B135"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "POKE 53280,-1\n")), vec!["B135"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "X=PEEK(65536)\n")), vec!["B134"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "SYS 65536\n")), vec!["B134"]);
    assert_eq!(warn_codes(cbm("basic7", "WAIT 53265,256\n")), vec!["B135"]);
    assert_eq!(warn_codes(cbm("cbm_v2", "A%=40000\n")), vec!["B137"]);
    // gültige Werte, Ausdrücke und Variablen bleiben still
    assert!(warn_codes(cbm("cbm_v2", "POKE 53280,255\nPOKE A+1,B\nPOKE 65535,0\nA%=32767\nX=PEEK(A)\nPOKE 2*40000,1\nSYS 64738\nB%=-32768\nPOKE 53280,1.5\n")).is_empty());
    // Atari: USR
    let a = atari("atari_basic", "X=USR(70000)\n").unwrap();
    assert_eq!(a.warnings.iter().map(|w| w.code).collect::<Vec<_>>(), vec!["B134"]);
    // Text und DATA bleiben still
    assert!(warn_codes(cbm("cbm_v2", "PRINT \"POKE 70000,1\"\nDATA POKE 70000,1\nREM POKE 70000,1\n")).is_empty());
}

#[test]
fn strict_mode_makes_range_problems_errors_and_off_silences_them() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let mut o = opts();
    o.numbers.ranges = vc_basic::numbers::RangeCheck::Error;
    let e = compile_plain_cbm("POKE 53280,300\n", &d, &o, 0x0801).err().unwrap();
    assert_eq!(e[0].code, "B135");
    o.numbers.ranges = vc_basic::numbers::RangeCheck::Off;
    assert!(compile_plain_cbm("POKE 53280,300\n", &d, &o, 0x0801).unwrap().warnings.is_empty());
}

#[test]
fn classic_path_warns_but_does_not_convert() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let w = vc_basic::numbers::classic_warnings("10 POKE $D020,%1\n20 A$=\"$FF\":PRINT A$\n", &Lexicon::Cbm(&d), "t.lst");
    assert_eq!(w.iter().map(|x| (x.code, x.line)).collect::<Vec<_>>(), vec![("B140", 1), ("B140", 1)]);
    // BASIC 65 liest beides selbst: keine Warnung
    let d65 = Dialect::load(&data(), "basic65").unwrap();
    assert!(vc_basic::numbers::classic_warnings("10 POKE $D020,%1\n", &Lexicon::Cbm(&d65), "t.lst").is_empty());
    // tokenisiert wird unverändert (klassisch)
    assert!(vc_basic::tokenize_text("10 POKE $D020,1\n", &d, "t.lst").is_ok());
}
