// SPDX-License-Identifier: GPL-3.0-or-later
//! Meldungen des Assemblers: Code, Schlüssel und Parameter (nicht Wortlaut); dazu Katalogprüfung (alle Schlüssel in de und en, deutscher Wortlaut unverändert).
use std::path::PathBuf;
use vc_asm::{assemble, Options, Policy};
use vc_i18n::Lang;
use vc_target::load_cpu;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn first(src: &str) -> vc_core::Diagnostic {
    let cpu = load_cpu(&data(), "6510").unwrap();
    let mut o = Options::new("t.asm", Policy::Deny);
    o.include_dirs.push(data());
    assemble(src, &cpu, &o).unwrap_err().remove(0)
}
fn warn(src: &str) -> vc_core::Diagnostic {
    let cpu = load_cpu(&data(), "6510").unwrap();
    assemble(src, &cpu, &Options::new("t.asm", Policy::Warn)).unwrap().warnings.remove(0)
}

/// (Quelltext, Code, Schlüssel, deutscher Text)
const ERRORS: &[(&str, &str, &str, &str)] = &[
    (".org $1000\n foo\n", "E205", "diag-asm-e205", "unbekannter Befehl 'FOO'"),
    (".org $1000\n lda\n", "E206", "diag-asm-e206-operand-needed", "LDA braucht einen Operanden"),
    (".org $1000\n lda (1,2,3)\n", "E206", "diag-asm-e206-indirect", "ungültiger indirekter Operand"),
    (".org $1000\n lda 1,2,3\n", "E206", "diag-asm-e206-operand", "ungültiger Operand"),
    (".org $1000\n lda [1\n", "E206", "diag-asm-e206-ptr32", "ungültiger 32-Bit-Zeiger"),
    (".org $1000\n lda #1/0\n", "E202", "diag-asm-e202-expression", "Division durch 0 in '1/0'"),
    (".org $1000\n lda #nix\n", "E202", "diag-asm-e202-expression", "unbekanntes Symbol 'nix' in 'nix'"),
    (".org $1000\n lda #(1\n", "E202", "diag-asm-e202-expression", "')' fehlt in '(1'"),
    (".org $1000\n lda #min()\n", "E202", "diag-asm-e202-expression", "min() erwartet mindestens 1 Argument(e), erhalten 0 in 'min()'"),
    (".org $1000\n lda #atan2(1)\n", "E202", "diag-asm-e202-expression", "atan2() erwartet 2 Argument(e), erhalten 1 in 'atan2(1)'"),
    (".org $1000\n lda #[1,2]\n", "E202", "diag-asm-e202-list-for-number", "Liste statt Zahl in '[1,2]' (Elemente mit [i] wählen oder .byte/.word für die ganze Liste)"),
    (".org $1000\n.for i in 5\n.next\n", "E202", "diag-asm-e202-for-in-not-list", ".for … in erwartet eine Liste oder range(…), nicht '5'"),
    (".org $1000\n.byte 300\n", "E203", "diag-asm-e203-value-range", "Wert 300 außerhalb -128..255"),
    (".org $1000\n.word 70000\n", "E203", "diag-asm-e203-value-16bit", "Wert 70000 außerhalb 16 Bit"),
    (".org 70000\n", "E203", "diag-asm-e203-org-range", ".org außerhalb 0..65535"),
    (".org $1000\n.fill -1\n", "E203", "diag-asm-e203-fill-length", ".fill: ungültige Länge"),
    (".org $1000\n.repeat -1\n.endrepeat\n", "E203", "diag-asm-e203-repeat-count", ".repeat: ungültige Anzahl"),
    (".org $1000\n.basicstub 2061, 70000\n", "E203", "diag-asm-e203-basicstub-line", ".basicstub: Zeilennummer 70000 außerhalb 0–63999"),
    (".org $1000\nx = 1\nx = 2\n", "E204", "diag-asm-e204-symbol", "Symbol 'x' doppelt definiert"),
    (".org $1000\n.function sin(x) = x\n", "E204", "diag-asm-e204-builtin-function", "'sin' ist eine eingebaute Funktion"),
    (".org $1000\n.assert 1 == 2\n", "E207", "diag-asm-e207-assert", "Assertion fehlgeschlagen"),
    (".org $1000\n.assert 1 == 2, zu klein\n", "E207", "diag-asm-e207-assert-message", "Assertion fehlgeschlagen: zu klein"),
    (".org $1000\n.error \"Speicher voll\"\n", "E207", "diag-asm-e207-user", "Speicher voll"),
    (".org $1000\n.foo\n", "E208", "diag-asm-e208-unknown-directive", "unbekannte Direktive '.foo'"),
    (".org $1000\n.petscii abc\n", "E208", "diag-asm-e208-string-expected", "Zeichenkette in Anführungszeichen erwartet"),
    (".org $1000\n.petscii \"{nix}\"\n", "E208", "diag-asm-e208-token-unknown", "unbekanntes Sonderzeichen {nix}"),
    (".org $1000\n.petscii \"{down\"\n", "E208", "diag-asm-e208-token-unclosed", "'{' ohne schließendes '}' (für ein einzelnes '{' schreibe '{{')"),
    (".org $1000\n.screen \"{down}\"\n", "E208", "diag-asm-e208-token-names", "{down}: Namen gibt es nur mit .petscii/.atascii oder .encoding \"petscii\"/\"atascii\" (hier nur {$hh} und {nnn})"),
    (".org $1000\n.petscii \"{down*0}\"\n", "E208", "diag-asm-e208-token-repeat", "{down*0}: Wiederholung 1 bis 255 erwartet"),
    (".org $1000\n.for 1\n.next\n", "E208", "diag-asm-e208-for-syntax", ".for erwartet: .for anfang, bedingung, schritt (z. B. .for i = 0, i < 8, i += 1)"),
    (".org $1000\n.include foo\n", "E208", "diag-asm-e208-include-name", ".include erwartet einen Dateinamen in Anführungszeichen oder <spitzen Klammern>"),
    (".org $1000\n.macro m\n", "E210", "diag-asm-e210-macro-unclosed", ".macro ohne .endmacro"),
    (".org $1000\n.if 1\n", "E210", "diag-asm-e210-if-unclosed", ".if ohne .endif"),
    (".org $1000\n.endif\n", "E210", "diag-asm-e210-unexpected", "unerwartetes .endif"),
    (".org $1000\n.break\n", "E210", "diag-asm-e210-outside-loop", ".break außerhalb einer Schleife"),
    (".org $1000\n.include \"gibtesnicht.inc\"\n", "E212", "diag-asm-e212-include-missing", "Include-Datei 'gibtesnicht.inc' nicht gefunden"),
    (".org $1000\n{ x\n}\n", "E213", "diag-asm-e213-after-open", "nach '{' darf nichts in derselben Zeile stehen"),
    (".org $1000\n}\n", "E213", "diag-asm-e213-unmatched-close", "'}' ohne öffnendes '{'"),
    (".org $1000\n lda -\n", "E214", "diag-asm-e214-before", "kein passendes '-'-Label davor"),
    (".org $1000\n lda.z (1,x)\n", "E215", "diag-asm-e215-suffix", "Suffix .z gilt nur für Zeropage- und absolute Adressierung"),
    (".org $1000\n.set i = 1\n", "E216", "diag-asm-e216", "Variable 'i' ist nicht mit .var angelegt"),
    ("nop\n", "E300", "diag-asm-e300-code-before-org", "Code vor .org"),
    (".org $1000\n.bank 0\n.org $1000\n nop\n.org $1000\n nop\n", "E301", "diag-asm-e301-overlap", "Segmente überlappen: $1000 und $1000"),
    (".org $1000\n slo $10\n", "E201", "diag-asm-e201-illegal-denied", "undokumentierter Opcode SLO (illegals=deny)"),
    (".org $1000\n.org $fffe\n nop\n nop\n nop\n", "E301", "diag-asm-e301-beyond-ffff", "Segment bei $FFFE überschreitet $FFFF"),
];

#[test]
fn errors_carry_code_key_and_german_text() {
    vc_i18n::set_thread_language(Some(Lang::De));
    for (src, code, key, de) in ERRORS {
        let d = first(src);
        assert_eq!((d.code, d.key), (*code, *key), "{src:?}: {}", d.message);
        assert_eq!(&d.message, de, "{src:?}");
        assert!(vc_i18n::has_key_in(Lang::En, key), "{key} fehlt im Englischen");
        if !key.ends_with("-user") { assert_ne!(d.text_in(Lang::En), d.text_in(Lang::De), "{key}: englischer Text gleich dem deutschen"); }
        assert!(!d.text_in(Lang::En).contains("⟦"), "{key}");
    }
}

#[test]
fn parameters_are_available_by_name() {
    let d = first(".org $1000\nx = 1\nx = 2\n");
    assert_eq!(d.arg_text("name"), "x");
    let d = first(".org $1000\n.byte 300\n");
    assert_eq!((d.arg_text("value"), d.arg_text("lo"), d.arg_text("hi")), ("300".into(), "-128".into(), "255".into()));
    // verschachtelte Meldung: E202 trägt die innere Meldung als Parameter
    let d = first(".org $1000\n lda #1/0\n");
    assert_eq!(d.arg_text("expr"), "1/0");
    match d.msg().arg("inner") {
        Some(vc_core::MsgArg::Msg(m)) => assert_eq!(m.key, "diag-asm-e202-div-zero"),
        other => panic!("{other:?}"),
    }
    let d = first(".org $1000\n.org $fffe\n nop\n nop\n nop\n");
    assert_eq!(d.arg_text("start"), "FFFE");
}

#[test]
fn warnings_carry_code_and_key() {
    vc_i18n::set_thread_language(Some(Lang::De));
    let d = warn(".org $1000\n lda #3.5\n");
    assert_eq!((d.code, d.key), ("W202", "diag-asm-w202"));
    assert_eq!((d.arg_text("value"), d.arg_text("to")), ("3.5".into(), "3".into()));
    assert_eq!(d.message, "Kommazahl 3.5 auf 3 abgeschnitten (int(), round() oder floor() verwenden) in '3.5'");
    let d = warn(".org $1000\n slo $10\n");
    assert_eq!((d.code, d.key, d.arg_text("mn").as_str()), ("W104", "diag-asm-w104", "SLO"));
    let d = warn(".org $1000\n.cycles t max=1\n lda #1\n lda #1\n.endcycles\n");
    assert_eq!((d.code, d.key), ("W201", "diag-asm-w201"));
    assert_eq!(d.message, "Zyklenbudget 't' überschritten: worst-case 4 > 1 Zyklen");
}

/// Alle im Quelltext verwendeten Schlüssel stehen in beiden Katalogen
#[test]
fn every_key_in_the_sources_exists_in_both_catalogs() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut n = 0;
    for f in std::fs::read_dir(dir).unwrap().flatten() {
        let text = std::fs::read_to_string(f.path()).unwrap();
        for (i, _) in text.match_indices("\"diag-asm-").chain(text.match_indices("\"asm-")) {
            let rest = &text[i + 1..];
            let key = &rest[..rest.find('"').unwrap()];
            assert!(vc_i18n::has_key_in(Lang::De, key) && vc_i18n::has_key_in(Lang::En, key), "{}: Schlüssel {key} fehlt im Katalog", f.path().display());
            n += 1;
        }
    }
    assert!(n > 100, "{n}");
}

/// Große Zahlen erscheinen in beiden Sprachen ohne Tausendertrennung
#[test]
fn large_numbers_have_no_grouping() {
    let d = first(".org $1000\n.word 70000\n");
    assert_eq!(d.text_in(Lang::En), "value 70000 outside 16 bits");
    assert_eq!(d.text_in(Lang::De), "Wert 70000 außerhalb 16 Bit");
}
