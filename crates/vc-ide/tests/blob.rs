// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_basic::{Dialect, Options};

/// Ordner mit den (nicht im Repository liegenden) `.sbas`-Beispielen: Umgebungsvariable `VC_SBAS_EXAMPLES`; ohne sie überspringen sich die Tests
fn sbas_dir() -> PathBuf { std::env::var("VC_SBAS_EXAMPLES").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt")) }

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn demo() -> PathBuf { sbas_dir().join("examples/multi/music_demo") }

fn build(sys: &str, dialect: &str, src: &str) -> Result<vc_ide::blob::Built, Vec<vc_core::Diagnostic>> {
    let d = Dialect::load(&data(), dialect).unwrap();
    let tgt = vc_target::load_target(&data(), sys).unwrap();
    let mut o = Options::new(demo().join("main.sbas").to_string_lossy());
    o.include_dirs.push(demo());
    o.system = Some(sys.into());
    o.load_addr = tgt.basic_load_addr;
    vc_ide::blob::compile_with_blobs(src, &d, &o, &demo(), &data(), &tgt)
}

#[test]
fn blob_follows_the_basic_text_and_symbols_become_constants() {
    if !demo().is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    // ohne Export gibt es die Song-Datei nicht: einmal erzeugen
    assert!(demo().join("build/export/tune.asm").exists() && demo().join("build/export/tune_ted.asm").exists(), "vorher: vcc media export music/tune.vcsong und music/tune_ted.vcsong (im Ordner examples/multi/music_demo)");
    let src = std::fs::read_to_string(demo().join("main.sbas")).unwrap();
    for (sys, dialect, load) in [("c64", "cbm_v2", 0x0801usize), ("c128", "basic7", 0x1C01), ("plus4", "basic35", 0x1001)] {
        let b = build(sys, dialect, &src).unwrap_or_else(|e| panic!("{sys}: {e:?}"));
        assert_eq!(b.blobs.len(), 1);
        let info = &b.blobs[0];
        // Der Code beginnt unmittelbar hinter dem BASIC-Programm (Ladeadresse + Länge ohne die zwei Kopfbytes)
        assert_eq!(info.addr as usize, load + b.compiled.prg.len() - 2, "{sys}");
        assert_eq!(b.prg.len(), b.compiled.prg.len() + info.size, "{sys}");
        // `sys bas_music_start` steht im Programm als Zahl: die Adresse des Einsprungs (erste Marke am Anfang des Blobs)
        let text = vc_basic::to_classic_text(&vc_basic::parse_prg(&b.compiled.prg).unwrap(), &d(dialect));
        assert!(text.contains(&format!("sys{}", info.addr)), "{sys}: {text}");
    }
}
fn d(id: &str) -> Dialect { Dialect::load(&data(), id).unwrap() }

#[test]
fn atari_blob_becomes_data_lines_above_the_program() {
    if !demo().is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    assert!(demo().join("build/export/atune.asm").exists(), "vorher: vcc media export music/atune.vcsong im Ordner examples/multi/music_demo");
    let src = std::fs::read_to_string(demo().join("main.sbas")).unwrap();
    let d = vc_basic::atari::AtariDialect::load(&data(), "atari_basic").unwrap();
    let tgt = vc_target::load_target(&data(), "atari8").unwrap();
    let mut o = vc_basic::AtariOptions::new(demo().join("main.sbas").to_string_lossy());
    o.include_dirs.push(demo());
    o.system = Some("atari8".into());
    let b = vc_ide::blob::compile_atari_with_blobs(&src, &d, &o, &demo(), &data(), &tgt).unwrap_or_else(|e| panic!("{e:?}"));
    let info = &b.blobs[0];
    // oben im Speicher, auf einer Seitengrenze, unter $A000; RAMTOP wird auf diese Seite gesetzt
    assert!(info.addr % 256 == 0 && info.addr as usize + info.size <= 0xA000, "{:04X}+{}", info.addr, info.size);
    let l = &b.compiled.listing;
    assert!(l.contains(&format!("POKE 106,{}", info.addr >> 8)), "{l}");
    assert!(l.contains("DATA 0\n") || l.trim_end().ends_with("DATA 0"), "Ende-Marke fehlt");
    assert!(l.lines().filter(|x| x.contains(" DATA ")).count() > 100, "zu wenige DATA-Zeilen");
    // Programm ohne blob bleibt unverändert (kein DATA)
    let mut o2 = vc_basic::AtariOptions::new("t.sbas");
    o2.system = Some("atari8".into());
    let p = vc_ide::blob::compile_atari_with_blobs("print 1\n", &d, &o2, &demo(), &data(), &tgt).unwrap();
    assert!(p.blobs.is_empty() && !p.compiled.listing.contains("DATA"));
}

#[test]
fn mega65_blob_is_delivered_as_data_lines_at_a_fixed_address() {
    if !demo().is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    assert!(demo().join("build/export/tune.asm").exists(), "vorher: vcc media export music/tune.vcsong");
    let src = std::fs::read_to_string(demo().join("main.sbas")).unwrap();
    let b = build("mega65", "basic65", &src).unwrap_or_else(|e| panic!("{e:?}"));
    let info = &b.blobs[0];
    assert_eq!(info.addr as usize, vc_ide::blob::MEGA65_BLOB_BASE);
    // Das PRG enthält nur das BASIC-Programm samt DATA-Zeilen, keinen Maschinencode dahinter
    assert_eq!(b.prg, b.compiled.prg);
    let text = vc_basic::to_classic_text(&vc_basic::parse_prg(&b.prg).unwrap(), &d("basic65"));
    assert!(text.contains(&format!("data{},", info.addr)) && text.lines().filter(|l| l.contains(" data") || l.contains("data")).count() > 60, "{text}");
    let sys: Vec<usize> = text.match_indices("sys").filter_map(|(i, _)| text[i + 3..].chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()).collect();
    assert!(sys.len() == 2 && sys.iter().all(|a| *a >= info.addr as usize && *a < info.addr as usize + info.size), "Einsprünge {sys:?} liegen nicht im Code: {text}");
}

#[test]
fn programs_without_blob_build_as_before() {
    if !demo().is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let b = build("mega65", "basic65", "print 1\n").unwrap();
    assert!(b.blobs.is_empty());
    assert_eq!(b.prg, b.compiled.prg);
}

#[test]
fn blob_errors_are_reported() {
    if !demo().is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let e = build("c64", "cbm_v2", "blob \"gibt_es_nicht.asm\"\n").err().unwrap();
    assert_eq!(e[0].code, "B114");
    let e = build("c64", "cbm_v2", "blob gibt_es_nicht.asm\n").err().unwrap();
    assert_eq!(e[0].code, "B113");
}
