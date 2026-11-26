// SPDX-License-Identifier: GPL-3.0-or-later
//! Meldungen tragen Schlüssel und Parameter und erscheinen in beiden Sprachen (i18n I2).
use std::path::PathBuf;
use vc_basic::{compile, number_lines, tokenize_text, Dialect, Options, PlainOptions};
use vc_core::Diagnostic;
use vc_i18n::Lang;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// Einige fehlerhafte Quelltexte quer durch die Module; jede Diagnose muss einen Katalogschlüssel `diag-basic-…` haben
fn samples() -> Vec<Diagnostic> {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let mut all = vec![];
    all.extend(tokenize_text("10 PRINT {nix}\n99999 END\nPRINT\n20 PRINT \"é\"\n", &d, "t.bas").unwrap_err());
    all.extend(number_lines("100 a=1\n50 b=2\ngoto @weg\n", &PlainOptions::new("t.bas", 63999)).unwrap_err());
    all.extend(compile("else\nendif\nwend\nloop\nconst X\nproc\ncall nix\nwhile 1\n@a:\n@a:\nprint\n", &d, &Options::new("t.sbas")).err().unwrap());
    all.extend(compile("#if foo\nprint 1\n", &d, &Options::new("t.sbas")).err().unwrap());
    all
}

#[test]
fn every_message_has_a_catalog_key_and_both_languages_differ() {
    let all = samples();
    assert!(all.len() >= 8, "{all:?}");
    for x in &all {
        assert!(x.key.starts_with("diag-basic-"), "{}: {}", x.code, x.message);
        assert!(vc_i18n::has_key_in(Lang::De, x.key) && vc_i18n::has_key_in(Lang::En, x.key), "{}", x.key);
        let (de, en) = (x.text_in(Lang::De), x.text_in(Lang::En));
        assert!(!de.contains("⟦") && !en.contains("⟦"), "{}", x.key);
        assert_eq!(x.message, de, "Standardsprache der Tests ist Deutsch");
    }
}

#[test]
fn parameters_and_text_in_english() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    // CBM erlaubt bis 63999
    let e = tokenize_text("64000 PRINT\n", &d, "t.bas").unwrap_err();
    assert_eq!((e[0].code, e[0].key, e[0].arg_text("num").as_str()), ("B100", "diag-basic-b100-line-number-max-cbm", "64000"));
    assert_eq!(e[0].text_in(Lang::De), "Zeilennummer 64000 größer als 63999");
    assert_eq!(e[0].text_in(Lang::En), "line number 64000 is greater than 63999");
}

#[test]
fn english_text_and_json_of_a_label_error() {
    let e = number_lines("goto @weg\n", &PlainOptions::new("t.bas", 63999)).unwrap_err();
    assert_eq!(e[0].text_in(Lang::En), "label @weg is not defined");
    assert_eq!(e[0].to_json(Lang::En)["args"]["name"], "weg");
}
