// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_basic::{compile, parse_prg, to_classic_text, Dialect, Options};

/// Ordner mit den (nicht im Repository liegenden) `.sbas`-Beispielen: Umgebungsvariable `VC_SBAS_EXAMPLES`; ohne sie überspringen sich die Tests
fn sbas_dir() -> PathBuf { std::env::var("VC_SBAS_EXAMPLES").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt")) }

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn fixture(n: &str) -> Vec<u8> { std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(n)).unwrap() }

#[test]
fn dialect_has_prefix_tokens() {
    let d = Dialect::load(&data(), "basic65").unwrap();
    assert_eq!(d.load_addr, 0x2001);
    assert_eq!(d.token_bytes("END"), Some(vec![0x80])); assert_eq!(d.token_bytes("BORDER"), Some(vec![0xFE, 0x3C])); assert_eq!(d.token_bytes("POT"), Some(vec![0xCE, 0x02]));
    assert_eq!(d.token_bytes("~"), Some(vec![0xFF]));
    assert_eq!(d.keyword_at(&[0xFE, 0x3C, 0x20], 0), Some(("BORDER", 2)));
    assert_eq!(d.keyword_at(&[0x99], 0), Some(("PRINT", 1)));
}

/// Byte für Byte wie `petcat -w65` (VICE 3.10): längster Treffer gewinnt (FOREGROUND statt FOR + EGROUND), Präfix-Tokens $CE/$FE
#[test]
fn classic_tokenizer_matches_petcat_w65() {
    let d = Dialect::load(&data(), "basic65").unwrap();
    let text = String::from_utf8(fixture("basic65_sample.bas")).unwrap();
    let lines = vc_basic::tokenize_text(&text, &d, "t.bas").unwrap();
    let prg = vc_basic::build_prg(&lines, d.load_addr);
    assert_eq!(prg, fixture("basic65_sample.prg"));
    // und zurück
    let back = to_classic_text(&parse_prg(&prg).unwrap(), &d);
    assert!(back.contains("10 border 6:background 0:foreground 1") && back.contains("50 dopen#1,\"daten\",w:print#1,\"x\":dclose#1") && back.contains("90 dma 0,1,2,3:vsync 100"), "{back}");
}

#[test]
fn structured_basic65_compiles() {
    let d = Dialect::load(&data(), "basic65").unwrap();
    let example = sbas_dir().join("examples/mega65/hello_basic/hello.sbas");
    if !example.is_file() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let src = std::fs::read_to_string(&example).unwrap();
    let c = compile(&src, &d, &Options::new("hello.sbas")).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(&c.prg[..2], &[0x01, 0x20]);
    let text = to_classic_text(&parse_prg(&c.prg).unwrap(), &d);
    assert!(text.contains("border6") && text.contains("background0") && text.contains("foreground1") && text.contains("vsync100"), "{text}");
}

/// C128 (BASIC 7.0, Ladeadresse $1C01) und Plus/4 (BASIC 3.5, $1001): byte-identisch zu `petcat -w70` bzw. `-w3`
#[test]
fn classic_tokenizer_matches_petcat_for_basic7_and_basic35() {
    for (id, load, name) in [("basic7", 0x1c01u16, "basic7_sample"), ("basic35", 0x1001, "basic35_sample")] {
        let d = Dialect::load(&data(), id).unwrap();
        assert_eq!(d.load_addr, load, "{id}");
        let text = String::from_utf8(fixture(&format!("{name}.bas"))).unwrap();
        let lines = vc_basic::tokenize_text(&text, &d, "t.bas").unwrap();
        assert_eq!(vc_basic::build_prg(&lines, d.load_addr), fixture(&format!("{name}.prg")), "{id}");
    }
}
