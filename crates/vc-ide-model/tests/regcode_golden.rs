// SPDX-License-Identifier: GPL-3.0-or-later
//! Goldene Tests des BASIC-Codegenerators (`vc_ide::regcode`, Konzept POKE-Designer Abschnitt 5): je Dialekt eine Sollwertdatei unter
//! `tests/golden/regcode/<dialekt>.txt`. Zusätzlich: Idempotenz gegen den Formatierer, fehlerfreier Bau, Hex gleich Dezimal in den
//! Bytes, kein `$` in der Dezimalausgabe. Die Daten unter `data/` werden nur gelesen. Neu schreiben der Sollwerte nur mit
//! `VC_UPDATE_GOLDEN=1` (nach Prüfung des Unterschieds).
use std::path::PathBuf;
use vc_basic::PlainOptions;
use vc_ide::chipmap::ChipMap;
use vc_ide::regcode::*;
use vc_ide_model::pretty::native_basic_line;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn golden(d: &str) -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/regcode").join(format!("{d}.txt")) }

struct Case { name: &'static str, ws: Vec<RegWrite>, o: Options }

fn w(m: &ChipMap, label: &str, assign: &[(&str, u32)], full: Option<u8>) -> RegWrite { write_from_map(m, label, assign, full).unwrap() }

fn cbm_cases(m: &ChipMap) -> Vec<Case> {
    let o = Options::default();
    let sid = |l: &str, v: u8| w(m, l, &[], Some(v));
    let mut locked = w(m, "vic2.ctrl1", &[], Some(1)); locked.lock = Some("Bankumschaltung".into());
    vec![
        Case { name: "bit_setzen", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 1)], None)], o: o.clone() },
        Case { name: "bit_loeschen", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 0)], None)], o: o.clone() },
        Case { name: "setzen_und_loeschen", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 0), ("DEN", 1)], None)], o: o.clone() },
        Case { name: "feld_mehrbit", ws: vec![w(m, "vic2.ctrl1", &[("YSCROLL", 3)], None)], o: o.clone() },
        Case { name: "ganzes_byte", ws: vec![w(m, "vic2.ctrl1", &[], Some(59))], o: o.clone() },
        Case { name: "schreibregister_teil_warnt", ws: vec![{ let mut x = w(m, "sid.mode_vol", &[], Some(15)); x.mask = 0x0F; x.fields = vec![("VOL".into(), 15)]; x }], o: o.clone() },
        Case { name: "ohne_kommentare_ohne_marken", ws: vec![w(m, "vic2.border", &[], Some(6)), w(m, "vic2.bg0", &[], Some(0))], o: Options { comments: Comment::Off, markers: false, ..o.clone() } },
        Case { name: "sid_init_mit_variable", ws: vec![sid("sid.mode_vol", 15), sid("sid.v1_ad", 9), sid("sid.v1_sr", 240), sid("sid.v1_freq_lo", 103), sid("sid.v1_freq_hi", 17), sid("sid.v1_ctrl", 17)], o: o.clone() },
        Case { name: "sprites_variable_mehrere_je_zeile", ws: vec![w(m, "vic2.spena", &[], Some(7)), w(m, "vic2.sp0col", &[], Some(1)), w(m, "vic2.sp1col", &[], Some(2)), w(m, "vic2.sp2col", &[], Some(3)), w(m, "vic2.sp0x", &[], Some(100)), w(m, "vic2.sp0y", &[], Some(120)), w(m, "vic2.spmc", &[], Some(4))], o: o.clone() },
        Case { name: "sprites_je_zeile_eine", ws: vec![w(m, "vic2.spena", &[], Some(7)), w(m, "vic2.sp0col", &[], Some(1)), w(m, "vic2.sp1col", &[], Some(2)), w(m, "vic2.sp2col", &[], Some(3)), w(m, "vic2.sp0x", &[], Some(100))], o: Options { per_line: false, ..o.clone() } },
        Case { name: "wenige_ohne_variable", ws: vec![w(m, "vic2.spena", &[], Some(7)), w(m, "vic2.sp0col", &[], Some(1))], o: o.clone() },
        Case { name: "variable_belegt_waehlt_anderen_namen", ws: vec![w(m, "vic2.spena", &[], Some(7)), w(m, "vic2.sp0col", &[], Some(1))], o: Options { vars: Vars::Always, taken: vec!["v".into()], ..o.clone() } },
        Case { name: "hex_zahlen", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 1)], None), w(m, "vic2.border", &[], Some(6))], o: Options { numbers: Numbers::Hex, ..o.clone() } },
        Case { name: "zeilennummern_und_rem", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 1)], None), w(m, "vic2.border", &[], Some(6))], o: Options { line_numbers: Some((100, 10)), ..o.clone() } },
        Case { name: "rem_ohne_nummern", ws: vec![w(m, "vic2.border", &[], Some(6))], o: Options { comments: Comment::Rem, ..o.clone() } },
        Case { name: "andere_bits_nicht_erhalten", ws: vec![w(m, "vic2.ctrl1", &[("BMM", 1)], None)], o: Options { keep_others: false, ..o.clone() } },
        Case { name: "gesperrt_ohne_bestaetigung", ws: vec![locked.clone()], o: o.clone() },
        Case { name: "gesperrt_mit_bestaetigung", ws: vec![locked], o: Options { allow_locked: true, ..o.clone() } },
    ]
}

fn atari_cases(m: &ChipMap, id: &str) -> Vec<Case> {
    let o = Options::default();
    let sh = |l: &str, a: &[(&str, u32)], f: Option<u8>| { let mut x = w(m, l, a, f); assert!(x.shadow.is_some(), "{l}: kein Schatten in den Daten"); x.access = Access::W; x };
    let mut v = vec![
        Case { name: "farbe_ueber_schatten", ws: vec![sh("gtia.colbk", &[], Some(36))], o: o.clone() },
        Case { name: "farbe_direkt_ins_hardwareregister", ws: vec![sh("gtia.colbk", &[], Some(36))], o: Options { atari_shadow: false, ..o.clone() } },
        Case { name: "dma_feld_ueber_schatten", ws: vec![sh("antic.dmactl", &[("PFW", 2)], None)], o: o.clone() },
        Case { name: "dma_feld_und_bit", ws: vec![sh("antic.dmactl", &[("PFW", 2), ("MISSILE", 1)], None)], o: o.clone() },
        Case { name: "direkt_schreibregister_teilmaske_warnt", ws: vec![sh("antic.dmactl", &[("PFW", 2)], None)], o: Options { atari_shadow: false, ..o.clone() } },
        Case { name: "gruppe_farben", ws: vec![sh("gtia.colpf0", &[], Some(40)), sh("gtia.colpf1", &[], Some(202)), sh("gtia.colpf2", &[], Some(148)), sh("gtia.colbk", &[], Some(0))], o: o.clone() },
        Case { name: "hex_zahlen", ws: vec![sh("gtia.colbk", &[], Some(36))], o: Options { numbers: Numbers::Hex, ..o.clone() } },
        Case { name: "zeilennummern", ws: vec![sh("antic.dmactl", &[("PFW", 2)], None), sh("gtia.colbk", &[], Some(36))], o: Options { line_numbers: Some((10, 10)), ..o.clone() } },
    ];
    let _ = id;
    v.retain(|c| !c.ws.is_empty());
    v
}

fn render(cases: &[Case], d: &Dialect) -> String {
    let mut out = String::new();
    for c in cases {
        let g = generate(&c.ws, d, &c.o);
        out.push_str(&format!("=== {}\n", c.name));
        if !g.lines.is_empty() { out.push_str(&g.text()); }
        for (r, why) in &g.locked { out.push_str(&format!("!! gesperrt: {r} ({why})\n")); }
        for wn in &g.warnings { out.push_str(&format!("!! {wn}\n")); }
    }
    out
}

fn check_dialect(id: &str, target: &str) {
    let map = ChipMap::load(&data(), target);
    let d = Dialect::load(&data(), id).unwrap();
    let cases = if d.atari { atari_cases(&map, id) } else { cbm_cases(&map) };
    let got = render(&cases, &d);
    let path = golden(id);
    if std::env::var("VC_UPDATE_GOLDEN").is_ok() { std::fs::write(&path, &got).unwrap(); }
    let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Sollwerte fehlen: {} (mit VC_UPDATE_GOLDEN=1 anlegen)", path.display()));
    assert_eq!(got, want, "Abweichung gegen {} (nach Prüfung mit VC_UPDATE_GOLDEN=1 übernehmen)", path.display());

    // Eigenschaften je Fall
    for c in &cases {
        let g = generate(&c.ws, &d, &c.o);
        let text = g.text();
        // idempotent: der Formatierer ändert die Ausgabe nicht
        for l in &g.lines { assert_eq!(&native_basic_line(l), l, "{id}/{}: Formatierer ändert `{l}`", c.name); }
        // Zeilenlänge
        for l in &g.lines { assert!(l.chars().count() <= c.o.max_len.min(d.max_line_len) + 8, "{id}/{}: zu lang: {l}", c.name); }
        if g.lines.is_empty() { continue; }
        // ohne Hex kein `$` in Zeilen mit Code
        if c.o.numbers == Numbers::Decimal { assert!(!text.lines().filter(|l| !l.trim_start().starts_with(';') && !l.contains("rem ")).any(|l| l.contains('$')), "{id}/{}: $ in Dezimalausgabe: {text}", c.name); }
        // baut ohne Meldung; Hex gibt dieselben Bytes wie Dezimal
        let bytes = |src: &str| -> Vec<u8> {
            let o = PlainOptions::new("t.bas", 63999);
            if d.atari { let ad = vc_basic::atari::AtariDialect::load(&data(), id).unwrap(); vc_basic::compile_plain_atari(src, &ad, &o).unwrap_or_else(|e| panic!("{id}/{}: {:?}\n{src}", c.name, e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).file }
            else { let cd = vc_basic::Dialect::load(&data(), id).unwrap(); vc_basic::compile_plain_cbm(src, &cd, &o, 0x0801).unwrap_or_else(|e| panic!("{id}/{}: {:?}\n{src}", c.name, e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).prg }
        };
        let b1 = bytes(&text);
        let dec = generate(&c.ws, &d, &Options { numbers: Numbers::Decimal, ..c.o.clone() });
        let b2 = bytes(&dec.text());
        // Turbo-BASIC XL behält `$` als eigene Konstante (Entscheidung 2 des Zahlen-Konzepts): dort nur „baut“
        if id != "turbo_basic_xl" { assert_eq!(b1, b2, "{id}/{}: Hex und Dezimal ergeben verschiedene Bytes", c.name); }
    }
}

#[test] fn golden_cbm_v2() { check_dialect("cbm_v2", "c64"); }
#[test] fn golden_basic35() { check_dialect("basic35", "c64"); }
#[test] fn golden_basic7() { check_dialect("basic7", "c64"); }
#[test] fn golden_basic65() { check_dialect("basic65", "c64"); }
#[test] fn golden_atari_basic() { check_dialect("atari_basic", "atari8"); }
#[test] fn golden_turbo_basic_xl() { check_dialect("turbo_basic_xl", "atari8"); }
