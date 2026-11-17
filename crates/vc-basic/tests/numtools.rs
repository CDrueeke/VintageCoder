// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenwerkzeuge (Stufen 3 bis 6): Umwandeln, Wertsuche, Bit-Rechnung, Hex-Vorschläge, Rechner.
use std::path::PathBuf;
use vc_basic::numbers::{Lexicon, NumberOptions, Role};
use vc_basic::numtools::*;
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn cbm() -> Dialect { Dialect::load(&data(), "cbm_v2").unwrap() }
fn conv(text: &str, t: Target, subsets: Subsets) -> Converted {
    let d = cbm();
    let c = ConvertOptions { target: t, subsets, ..Default::default() };
    convert_lines(text, 0, text.split('\n').count() - 1, None, &c, &Lexicon::Cbm(&d), &NumberOptions::default())
}

#[test]
fn decimal_to_hex_uses_width_by_role() {
    let r = conv("10 POKE 53280,14:POKE 53281,6\n", Target::Hex, Subsets::ALL);
    assert_eq!(r.text, "10 POKE $D020,$0E:POKE $D021,$06\n".replace("$0E", "$0E").replace("$06", "$06"), "{}", r.text);
    assert_eq!(r.changes.len(), 4);
    // Adresse unter 256 bekommt vier Ziffern, Wert zwei
    assert_eq!(conv("POKE 5,5\n", Target::Hex, Subsets::ALL).text, "POKE $0005,$05\n");
}

#[test]
fn by_use_makes_addresses_hex_values_binary_and_the_rest_decimal() {
    let r = conv("POKE $D020,$0E\nX=$FF AND 15\nDATA $10,2\n", Target::ByUse, Subsets::ALL);
    assert_eq!(r.text, "POKE $D020,%00001110\nX=255 AND 15\nDATA 16,2\n", "{}", r.text);
}

#[test]
fn subsets_limit_what_is_converted() {
    let r = conv("POKE 53280,14\nX=255\nDATA 1,2\n", Target::Hex, Subsets::ADDRESSES);
    assert_eq!(r.text, "POKE $D020,14\nX=255\nDATA 1,2\n");
    let r = conv("POKE 53280,14\nX=255\nDATA 1,2\n", Target::Hex, Subsets { addresses: false, values: false, exprs: false, data: true });
    assert_eq!(r.text, "POKE 53280,14\nX=255\nDATA $01,$02\n");
}

#[test]
fn strings_comments_line_numbers_targets_and_labels_stay() {
    let src = "10 PRINT \"POKE 53280,14\":REM POKE 53280,14\n20 GOTO 10\n@x:\nA=ON 5\n30 DATA 7,text 5,\"9\",8\nIF A THEN 10\nON A GOSUB 10,20\nX=1.5+2E3\nY=A-5\nZ=-5\n";
    let r = conv(src, Target::Hex, Subsets::ALL);
    assert_eq!(r.text, "10 PRINT \"POKE 53280,14\":REM POKE 53280,14\n20 GOTO 10\n@x:\nA=ON $05\n30 DATA $07,text 5,\"9\",$08\nIF A THEN 10\nON A GOSUB 10,20\nX=1.5+2E3\nY=A-$05\nZ=-5\n", "{}", r.text);
}

#[test]
fn keeping_width_between_hex_and_binary() {
    let r = conv("X=$000E\n", Target::Bin, Subsets::ALL);
    assert_eq!(r.text, "X=%0000000000001110\n");
    let r = conv("X=%0000000000001110\n", Target::Hex, Subsets::ALL);
    assert_eq!(r.text, "X=$000E\n");
    let d = cbm();
    let c = ConvertOptions { target: Target::Bin, keep_zeros: false, groups: true, ..Default::default() };
    let r = convert_lines("X=14\n", 0, 0, None, &c, &Lexicon::Cbm(&d), &NumberOptions::default());
    assert_eq!(r.text, "X=%0000_1110\n");
}

#[test]
fn line_and_selection_scopes() {
    let d = cbm();
    let c = ConvertOptions::default();
    let text = "POKE 1000,1\nPOKE 2000,2\nPOKE 3000,3\n";
    let r = convert_lines(text, 1, 1, None, &c, &Lexicon::Cbm(&d), &NumberOptions::default());
    assert_eq!(r.text, "POKE 1000,1\nPOKE $07D0,$02\nPOKE 3000,3\n");
    assert_eq!(r.lines_changed(), 1);
    // Auswahl: nur Zahlen, die ganz darin liegen (hier 2000, nicht die 2 dahinter)
    let a = text.find("2000").unwrap();
    let r = convert_lines(text, 0, 2, Some((a, a + 4)), &c, &Lexicon::Cbm(&d), &NumberOptions::default());
    assert_eq!(r.text, "POKE 1000,1\nPOKE $07D0,2\nPOKE 3000,3\n");
}

#[test]
fn conversion_keeps_the_built_program_identical() {
    let d = cbm();
    let src = "POKE 53280,14:POKE 53281,%00000110\nX=PEEK($D012) AND 15\nDATA 1,2,$FF,%1010\nSYS 49152\n";
    let build = |t: &str| compile_plain_cbm(t, &d, &PlainOptions::new("t.bas", 63999), 0x0801).unwrap().prg;
    let before = build(src);
    for t in [Target::Dec, Target::Hex, Target::Bin, Target::ByUse] {
        let r = conv(src, t, Subsets::ALL);
        assert_eq!(build(&r.text), before, "{t:?}: {}", r.text);
        // zweimal angewandt ändert nichts mehr
        assert_eq!(conv(&r.text, t, Subsets::ALL).text, r.text, "{t:?}");
    }
}

#[test]
fn parse_and_find_by_value() {
    assert_eq!(parse_number("$D020"), Some(53280));
    assert_eq!(parse_number(" %1101_0000 "), Some(208));
    assert_eq!(parse_number("-5"), Some(-5));
    assert_eq!(parse_number("$"), None);
    assert_eq!(parse_number("12a"), None);
    assert_eq!(parse_number("$_1"), None);
    let d = cbm();
    let o = NumberOptions::default();
    let hits = find_value("10 POKE 53280,14:A$=\"$D020\":POKE $D020,%1101000000100000:REM 53280", 53280, &Lexicon::Cbm(&d), &o);
    assert_eq!(hits.len(), 3, "{hits:?}");
    assert_eq!(hits[0].radix, 10);
    assert_eq!(hits[1].radix, 16);
    assert_eq!(hits[2].radix, 2);
    // Zeilennummer zählt nicht
    assert!(find_value("53280 PRINT 1", 53280, &Lexicon::Cbm(&d), &o).is_empty());
    // Ersetzen behält die Schreibweise
    let n = &hits[1];
    assert_eq!(reformat_like(n, 0xD021, true), "$D021");
    assert_eq!(reformat_like(&hits[0], 53281, true), "53281");
    let b = find_value("X=%00001110", 14, &Lexicon::Cbm(&d), &o);
    assert_eq!(reformat_like(&b[0], 5, true), "%00000101");
}

#[test]
fn bit_arithmetic() {
    let mut b = Bits::new(8, 0b0000_1110);
    assert_eq!(b.bin_text(true), "%0000_1110");
    assert_eq!(b.hex_text(true), "$0E");
    b.toggle(0); b.toggle(7);
    assert_eq!(b.value, 0b1000_1111);
    b.invert(); assert_eq!(b.value, 0b0111_0000);
    b.shift_left(); assert_eq!(b.value, 0b1110_0000);
    b.shift_left(); assert_eq!(b.value, 0b1100_0000, "Schieben verliert Bits oben");
    b.shift_right(); assert_eq!(b.value, 0b0110_0000);
    b.set_field(0, 2, 5); assert_eq!(b.field(0, 2), 5); assert_eq!(b.value, 0b0110_0101);
    b.clear(); assert_eq!(b.value, 0); b.set_all(); assert_eq!(b.value, 255);
    let mut w = Bits::new(16, 0x12AB);
    w.swap_bytes(); assert_eq!(w.value, 0xAB12);
    assert_eq!(w.hex_text(true), "$AB12");
    w.resize(8); assert_eq!(w.value, 0x12);
    assert_eq!(Bits::new(8, 300).value, 44, "Wert wird auf die Breite beschnitten");
    assert_eq!(Bits::fit(255), 8); assert_eq!(Bits::fit(256), 16); assert_eq!(Bits::fit(70000), 32);
    let mut x = Bits::new(32, 1); x.shift_left(); assert_eq!(x.value, 2);
    assert_eq!(Bits::new(16, 0x00FF).source_text(2, true, false), "%0000000011111111");
    assert_eq!(Bits::new(8, 14).source_text(10, true, false), "14");
}

#[test]
fn import_suggestions_only_for_known_address_arguments() {
    let d = cbm();
    let src = "10 POKE 53280,14\n20 X=PEEK(53281)+53280\n30 SYS 49152\n40 DATA 53280,2\n50 PRINT \"53280\"\n60 POKE 53280+1,2\n70 POKE 1000,255\n80 REM POKE 53280,1\n90 POKE 53280,1000\n";
    let known = |v: u32| (0xD000..=0xD02E).contains(&v);
    let s = suggest_hex(src, &Lexicon::Cbm(&d), &NumberOptions::default(), &known);
    let got: Vec<(usize, &str, &str)> = s.iter().map(|x| (x.line + 1, x.from.as_str(), x.to.as_str())).collect();
    assert_eq!(got, vec![(1, "53280", "$D020"), (2, "53281", "$D021"), (9, "53280", "$D020")], "{got:?}");
    // einzeln abwählbar, nie blind: ohne Auswahl bleibt der Text gleich
    assert_eq!(apply_suggestions(src, &s, &[false, false, false]), src);
    let t = apply_suggestions(src, &s, &[true, false, true]);
    assert!(t.contains("10 POKE $D020,14") && t.contains("PEEK(53281)") && t.contains("90 POKE $D020,1000"), "{t}");
    // Werte und Zahlen anderer Argumente werden nie vorgeschlagen
    assert!(s.iter().all(|x| x.value != 1000));
    let _ = Role::Address;
}

#[test]
fn calculator() {
    assert_eq!(calc("$D000 + 32"), Ok(0xD020));
    assert_eq!(calc("%1010 OR %0101"), Ok(15));
    assert_eq!(calc("(1 << 4) | 3"), Err("unerwartetes Zeichen '|'".into()));
    assert_eq!(calc("(1 << 4) OR 3"), Ok(19));
    assert_eq!(calc("255 AND $0F XOR 1"), Ok(14));
    assert_eq!(calc("2 + 3 * 4"), Ok(14));
    assert_eq!(calc("-5 + 2"), Ok(-3));
    assert_eq!(calc("256 >> 4"), Ok(16));
    assert_eq!(calc("10 / 3"), Ok(3));
    assert!(calc("1 / 0").is_err());
    assert!(calc("(1").is_err());
    assert!(calc("").is_err());
    assert!(calc("foo").is_err());
    assert!(calc("1 +").is_err());
}
