// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien per `#include` im nativen BASIC (Konzept „Include von Medien“, Abschnitt 3): Provider-Haken, Zeilenzuordnung, Altpfad, einmaliges
//! Einbinden, Nummerierung (`#numbering`), `#if` um das Include, Bytegleichheit zum Altpfad je Dialekt, Platzbedarf, Lesereihenfolge, Umwandlung.
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vc_basic::include::{IncludeHooks, IncludeProvider, LegacyHint};
use vc_basic::{compile_plain_atari, compile_plain_cbm, expand_includes_with, labels, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-basic-media-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("graphics")).unwrap();
    std::fs::create_dir_all(d.join("build/export")).unwrap();
    d
}
/// Erzeugter Text eines Mediums (wie `vc_media::export_data_labeled`): 64 Byte → sechs DATA-Zeilen
fn sprite(n: usize) -> String { vc_media::export_data_labeled("hero", &(0..n).map(|i| (i * 3 % 256) as u8).collect::<Vec<_>>()) }

/// Provider für `*.vcsprite` (Test: feste Bytes), `*.vcsong` (Fehler: kein DATA), sonst gewöhnliche Datei
fn provider(n: usize) -> IncludeProvider {
    Arc::new(move |p: &Path| match p.extension().and_then(|e| e.to_str()) {
        Some("vcsprite") => Some(if p.file_name().unwrap().to_str().unwrap().starts_with("fehlt") { Err("Medium nicht gefunden".into()) } else { Ok(sprite(n)) }),
        Some("vcsong") => Some(Err("Song nicht als BASIC-DATA exportierbar".into())),
        _ => None,
    })
}
fn hooks(n: usize) -> IncludeHooks {
    let legacy: LegacyHint = Arc::new(|p: &Path| p.to_string_lossy().contains("build/export/hero.data.bas").then(|| "graphics/hero.vcsprite".to_string()));
    IncludeHooks { provider: Some(provider(n)), legacy_hint: Some(legacy) }
}

#[test]
fn medium_is_included_and_lines_map_to_the_medium() {
    let d = tmp("map");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let e = expand_includes_with("PRINT 1\n#include \"graphics/hero.vcsprite\"\nREAD A\n", &main, &hooks(64)).unwrap();
    assert_eq!(e.media.len(), 1);
    assert!(e.text.contains("@hero:") && e.text.contains("DATA 0,3,6,9"), "{}", e.text);
    let medium = d.join("graphics/hero.vcsprite").to_string_lossy().into_owned();
    // Zeile 2 des Gesamttexts = Zeile 1 des erzeugten Textes (Kommentar), die letzte Zeile ist wieder die Hauptdatei
    assert_eq!(e.map[1], (medium.clone(), 1));
    assert_eq!(e.map.iter().filter(|(f, _)| *f == medium).count(), 8);
    assert_eq!(e.map.last().unwrap(), &(main, 3));
    assert!(e.warnings.is_empty());
}

#[test]
fn second_include_of_the_same_medium_is_skipped_silently() {
    let d = tmp("once");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let e = expand_includes_with("#include \"graphics/hero.vcsprite\"\n#include \"graphics/hero.vcsprite\"\n", &main, &hooks(64)).unwrap();
    assert_eq!(e.text.matches("@hero:").count(), 1);
    assert_eq!(e.media.len(), 1);
}

#[test]
fn media_failures_are_b193_and_plain_files_still_work() {
    let d = tmp("err");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let e = expand_includes_with("#include \"graphics/fehlt.vcsprite\"\n", &main, &hooks(64)).unwrap_err();
    assert_eq!(e[0].code, "B193");
    assert!(e[0].message.contains("Medium nicht gefunden") && e[0].message.contains("fehlt.vcsprite"), "{}", e[0].message);
    assert_eq!((e[0].file.as_str(), e[0].line), (main.as_str(), 1));
    let e = expand_includes_with("PRINT 1\n#include \"musik/t.vcsong\"\n", &main, &hooks(64)).unwrap_err();
    assert_eq!((e[0].code, e[0].line), ("B193", 2));
    assert!(e[0].message.contains("nicht als BASIC-DATA"));
    std::fs::write(d.join("lib.bas"), "DATA 1\n").unwrap();
    assert!(expand_includes_with("#include \"lib.bas\"\n", &main, &hooks(64)).is_ok());
    let e = expand_includes_with("#include \"nirgends.bas\"\n", &main, &hooks(64)).unwrap_err();
    assert_eq!(e[0].code, "B120");
}

#[test]
fn legacy_path_warns_with_the_source_and_still_builds() {
    let d = tmp("legacy");
    std::fs::write(d.join("build/export/hero.data.bas"), sprite(64)).unwrap();
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let e = expand_includes_with("#include \"build/export/hero.data.bas\"\n", &main, &hooks(64)).unwrap();
    assert_eq!(e.warnings.len(), 1);
    assert_eq!(e.warnings[0].code, "B194");
    assert!(e.warnings[0].message.contains("graphics/hero.vcsprite"), "{}", e.warnings[0].message);
    assert!(e.media.is_empty());
}

const PROG: &str = "#include \"graphics/hero.vcsprite\"\nrestore @hero\nfor i=0 to 63:read a:poke 832+i,a:next\n";
const PROG_OLD: &str = "#include \"build/export/hero.data.bas\"\nrestore @hero\nfor i=0 to 63:read a:poke 832+i,a:next\n";

fn cbm_prg(id: &str, prog: &str, d: &Path, h: &IncludeHooks, num: Option<(u32, u32)>) -> Vec<u8> {
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let e = expand_includes_with(prog, &main, h).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>()));
    let dia = Dialect::load(&data(), id).unwrap();
    let mut o = PlainOptions::new(&main, 63999);
    if let Some((s, t)) = num { o.start = s; o.step = t; }
    o.media = e.media.len();
    compile_plain_cbm(&e.text, &dia, &o, dia.load_addr).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).prg
}

/// Bytegleich zum Altpfad (gleicher Text aus `build/export`) in allen Commodore-Dialekten; das Medium trägt keine eigenen Nummern
#[test]
fn medium_and_legacy_path_give_identical_programs_in_every_cbm_dialect() {
    let d = tmp("bytes");
    std::fs::write(d.join("build/export/hero.data.bas"), sprite(64)).unwrap();
    for id in ["cbm_v2", "vic_basic_2", "basic7", "basic35", "basic65"] {
        let new = cbm_prg(id, PROG.replace("restore @hero\n", "").as_str(), &d, &hooks(64), None);
        let old = cbm_prg(id, PROG_OLD.replace("restore @hero\n", "").as_str(), &d, &hooks(64), None);
        assert_eq!(new, old, "{id}");
        assert!(new.len() > 100);
    }
    // Dialekte mit RESTORE auf eine Zeile: auch mit Marke
    for id in ["basic7", "basic35", "basic65"] {
        assert_eq!(cbm_prg(id, PROG, &d, &hooks(64), None), cbm_prg(id, PROG_OLD, &d, &hooks(64), None), "{id}");
    }
}

#[test]
fn numbering_applies_to_the_medium_lines() {
    let d = tmp("numbering");
    let a = cbm_prg("basic7", PROG, &d, &hooks(64), Some((100, 5)));
    let list = vc_basic::to_classic_text(&vc_basic::parse_prg(&a).unwrap(), &Dialect::load(&data(), "basic7").unwrap());
    let list = list.to_lowercase();
    assert!(list.starts_with("100 rem") && list.contains("\n105 data 0,3,6") && list.contains("\n135 restore 105"), "{list}");
    let b = cbm_prg("basic7", PROG, &d, &hooks(64), None);
    assert_ne!(a, b, "andere Nummern, andere Bytes");
}

#[test]
fn if_around_the_include_decides_whether_the_medium_is_built() {
    let d = tmp("if");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let dia = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "#if SYSTEM_C64\n#include \"graphics/hero.vcsprite\"\n#endif\nPRINT 1\n";
    let build = |sys: &str| {
        let e = expand_includes_with(src, &main, &hooks(64)).unwrap();
        let mut o = PlainOptions::new(&main, 63999); o.system = Some(sys.into());
        compile_plain_cbm(&e.text, &dia, &o, dia.load_addr).unwrap().prg
    };
    let plain = { let mut o = PlainOptions::new(&main, 63999); o.system = Some("vic20".into()); compile_plain_cbm("PRINT 1\n", &dia, &o, dia.load_addr).unwrap().prg };
    assert!(build("c64").len() > plain.len());
    assert_eq!(build("vic20"), plain, "nur PRINT 1 ohne Medium");
}

#[test]
fn program_too_big_warns_before_loading() {
    let d = tmp("big");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let dia = Dialect::load(&data(), "cbm_v2").unwrap();
    let run = |n: usize, sys: &str| {
        let e = expand_includes_with("#include \"graphics/hero.vcsprite\"\n", &main, &hooks(n)).unwrap();
        let mut o = PlainOptions::new(&main, 63999); o.system = Some(sys.into());
        compile_plain_cbm(&e.text, &dia, &o, dia.load_addr).unwrap().warnings.iter().map(|w| w.code).collect::<Vec<_>>()
    };
    assert!(!run(64, "c64").contains(&"B195"));
    assert!(run(8000, "vic20").contains(&"B195"), "8000 Byte DATA sprengen die 3583 Byte des unerweiterten VC-20");
    assert!(!run(8000, "c64").contains(&"B195"));
    assert!(run(30000, "c64").contains(&"B195"));
    assert!(!run(30000, "mega65").contains(&"B195"), "unbekanntes System: keine Prüfung");
}

#[test]
fn restore_label_with_several_media_warns_in_v2_only() {
    let d = tmp("v2");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let prog = "#include \"graphics/hero.vcsprite\"\n#include \"graphics/zwei.vcsprite\"\nrestore @hero\nread a\n";
    let codes = |id: &str, prog: &str| {
        let e = expand_includes_with(prog, &main, &hooks(64)).unwrap();
        let dia = Dialect::load(&data(), id).unwrap();
        let mut o = PlainOptions::new(&main, 63999); o.media = e.media.len();
        // zwei gleichnamige Marken wären ein Fehler: zweites Medium unter eigenem Namen
        let text = e.text.replacen("@hero:", "@zwei:", 2).replacen("@zwei:", "@hero:", 1);
        compile_plain_cbm(&text, &dia, &o, dia.load_addr).map(|r| r.warnings.iter().map(|w| w.code).collect::<Vec<_>>())
    };
    assert!(codes("cbm_v2", prog).unwrap().contains(&"B196"));
    assert!(!codes("basic7", prog).unwrap().contains(&"B196"));
    assert!(!codes("cbm_v2", "#include \"graphics/hero.vcsprite\"\nrestore @hero\n").unwrap().contains(&"B196"), "ein Medium: kein Reihenfolgeproblem");
}

#[test]
fn atari_and_turbo_include_media_like_the_legacy_path() {
    let d = tmp("atari");
    std::fs::write(d.join("build/export/hero.data.bas"), sprite(64)).unwrap();
    let main = d.join("main.bas").to_string_lossy().into_owned();
    for id in ["atari_basic", "turbo_basic_xl"] {
        let dia = vc_basic::atari::AtariDialect::load(&data(), id).unwrap();
        let go = |prog: &str| {
            let e = expand_includes_with(prog, &main, &hooks(64)).unwrap();
            let mut o = PlainOptions::new(&main, 32767); o.media = e.media.len();
            compile_plain_atari(&e.text, &dia, &o).unwrap_or_else(|e| panic!("{id}: {:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).file
        };
        let body = "restore @hero\nfor i=0 to 63:read a:poke 1536+i,a:next i\n";
        assert_eq!(go(&format!("#include \"graphics/hero.vcsprite\"\n{body}")), go(&format!("#include \"build/export/hero.data.bas\"\n{body}")), "{id}");
    }
}

#[test]
fn conversion_sees_media_labels_and_round_trips() {
    let d = tmp("conv");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let units = labels::load_set_with(&main, PROG, &hooks(64)).unwrap();
    assert_eq!(units.len(), 2);
    let (_, media) = vc_basic::include::load_set_media(&main, PROG, &hooks(64)).unwrap();
    assert_eq!(media, [d.join("graphics/hero.vcsprite").to_string_lossy().into_owned()], "Medien sind schreibgeschützt");
    assert_eq!(units[1].file, d.join("graphics/hero.vcsprite").to_string_lossy());
    assert!(units[1].text.contains("@hero:"));
    let dia = vc_basic::dialects::AnyDialect::load(&data(), "basic7").unwrap();
    let info = vc_basic::dialects::info(&data(), "basic7").unwrap();
    let cx = labels::Cx::new(&dia, &info);
    let conv = labels::to_numbers(&units, &cx, None).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>()));
    let text: String = conv.units.iter().map(|u| u.text.clone()).collect();
    assert!(text.contains("restore 20") && text.contains("20 DATA 0,3,6,9") && text.contains(";@hero"), "{text}");
}

/// Nummernmodus (Konzept 7.8): `<marke>_line` ist die Zeilennummer der Marke; bei mehrteiligen Medien je Teil `<medium>_<teil>_line`
#[test]
fn line_constants_for_media_labels() {
    let d = tmp("lineconst");
    let main = d.join("main.bas").to_string_lossy().into_owned();
    let dia = Dialect::load(&data(), "cbm_v2").unwrap();
    let multi: IncludeProvider = Arc::new(|p: &Path| (p.extension().and_then(|e| e.to_str()) == Some("vcchar")).then(|| Ok(
        "@font_chars:\nDATA 1,2\n@font_attrs:\nDATA 3\n".to_string())));
    let h = IncludeHooks { provider: Some(multi), legacy_hint: None };
    let src = "10 RESTORE font_attrs_line\n20 READ A\n30 RESTORE font_chars_line:READ B\n40 PRINT \"font_chars_line\"\n50 X$=\"A\":PRINT FONT_LINE\n#include \"gfx/font.vcchar\"\n";
    let e = expand_includes_with(src, &main, &h).unwrap();
    let mut o = PlainOptions::new(&main, 63999); o.media = e.media.len();
    let r = compile_plain_cbm(&e.text, &dia, &o, dia.load_addr).unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings.iter().map(|w| (w.code, w.message.clone())).collect::<Vec<_>>());
    let text = vc_basic::to_classic_text(&vc_basic::parse_prg(&r.prg).unwrap(), &dia).to_lowercase();
    assert!(text.contains("10 restore 70") && text.contains("30 restore 60:read b"), "{text}");
    assert!(text.contains("\"font_chars_line\"") && text.contains("print font_line"), "in Zeichenketten und unbekannten Namen bleibt es: {text}");
    assert_eq!(r.labels, [("font_chars".to_string(), 60), ("font_attrs".to_string(), 70)]);
    // ohne Medien bleibt der Name eine gewöhnliche Variable
    let mut o2 = o.clone(); o2.media = 0;
    let t2 = vc_basic::to_classic_text(&vc_basic::parse_prg(&compile_plain_cbm(&e.text, &dia, &o2, dia.load_addr).unwrap().prg).unwrap(), &dia).to_lowercase();
    assert!(t2.contains("restore font_attrs_line"), "{t2}");
}
