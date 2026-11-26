// SPDX-License-Identifier: GPL-3.0-or-later
//! Natives BASIC im IDE-Bau: Zielsystem (`#if SYSTEM_…`), Syntaxprüfung (B15x), Nummerierung und „kompakt“ aus dem Projekt, `#asm`.
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn proj(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-bp-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}
fn run(d: &std::path::Path, f: &str, t: &str) -> serde_json::Value { build(&Ctx { root: d, data: &data() }, f, Some(t)).unwrap() }
fn listing(d: &std::path::Path, stem: &str) -> String { std::fs::read_to_string(d.join(format!("build/{stem}.classic.txt"))).unwrap().to_ascii_uppercase() }

#[test]
fn if_system_greift_je_zielsystem() {
    let d = proj("sys");
    std::fs::write(d.join("a.bas"), "#if SYSTEM_C64\nPRINT \"C64\"\n#else\nPRINT \"ANDERES\"\n#endif\n").unwrap();
    assert_eq!(run(&d, "a.bas", "c64")["ok"], true);
    assert!(listing(&d, "a").contains("C64"));
    assert_eq!(run(&d, "a.bas", "c128")["ok"], true);
    assert!(listing(&d, "a").contains("ANDERES"));
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn syntaxpruefung_meldet_warnungen_und_laesst_sich_abschalten() {
    let d = proj("chk");
    std::fs::write(d.join("a.bas"), "10 FOR I=1 TO 3\n20 PRINT I\n30 END\n").unwrap();
    let out = run(&d, "a.bas", "c64");
    assert_eq!(out["ok"], true, "{out}");
    let codes: Vec<&str> = out["diagnostics"].as_array().unwrap().iter().filter_map(|x| x["code"].as_str()).collect();
    assert!(codes.contains(&"B151"), "{out}");
    // Einzelner Code abgeschaltet
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"disabled":["B151"]}}"#).unwrap();
    let out = run(&d, "a.bas", "c64");
    assert!(!out["diagnostics"].as_array().unwrap().iter().any(|x| x["code"] == "B151"), "{out}");
    // Prüfung ganz aus
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"check":false}}"#).unwrap();
    assert!(run(&d, "a.bas", "c64")["diagnostics"].as_array().unwrap().is_empty());
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn nummerierung_und_kompakt_kommen_aus_dem_projekt() {
    let d = proj("num");
    std::fs::write(d.join("a.bas"), "PRINT \"A\"\nPRINT \"B\"\n").unwrap();
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"start":100,"step":5}}"#).unwrap();
    assert_eq!(run(&d, "a.bas", "c64")["ok"], true);
    let l = listing(&d, "a");
    assert!(l.contains("100 PRINT") && l.contains("105 PRINT"), "{l}");
    // ungültige Werte: B191
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"start":100,"step":0}}"#).unwrap();
    let bad = run(&d, "a.bas", "c64");
    assert_eq!(bad["ok"], false);
    assert_eq!(bad["diagnostics"][0]["code"], "B191", "{bad}");
    // zu wenig Platz: B192
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"start":10,"step":40000}}"#).unwrap();
    std::fs::write(d.join("a.bas"), "PRINT 1\nPRINT 2\nPRINT 3\n").unwrap();
    assert_eq!(run(&d, "a.bas", "c64")["diagnostics"][0]["code"], "B192");
    // kompakt: weniger Bytes
    std::fs::write(d.join("a.bas"), "10 IF A = 1 THEN PRINT \"X\" : GOTO 10\n").unwrap();
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"}}"#).unwrap();
    let normal = run(&d, "a.bas", "c64")["size"].as_u64().unwrap();
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"compact":true}}"#).unwrap();
    let kompakt = run(&d, "a.bas", "c64")["size"].as_u64().unwrap();
    assert!(kompakt < normal, "{kompakt} >= {normal}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn blob_wird_hinter_den_basic_text_gelegt_und_marke_ersetzt() {
    let d = proj("blob");
    std::fs::write(d.join("code.asm"), ".org asm_base\nstart: lda #1\n       rts\n").unwrap();
    std::fs::write(d.join("a.bas"), "#asm \"code.asm\"\nSYS asm.start\n").unwrap();
    let out = run(&d, "a.bas", "c64");
    assert_eq!(out["ok"], true, "{out}");
    let prg = std::fs::read(d.join("build/a.prg")).unwrap();
    assert_eq!(&prg[prg.len() - 3..], &[0xA9, 0x01, 0x60]);
    let l = listing(&d, "a");
    assert!(l.contains("SYS"), "{l}");
    assert!(!l.contains("ASM."), "{l}");
    assert!(out["log"].to_string().contains("code.asm"), "{}", out["log"]);
    // fehlende Datei: Fehler mit B114
    std::fs::write(d.join("b.bas"), "#asm \"fehlt.asm\"\nSYS asm.start\n").unwrap();
    let bad = run(&d, "b.bas", "c64");
    assert_eq!(bad["ok"], false);
    assert_eq!(bad["diagnostics"][0]["code"], "B114", "{bad}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn b150_je_name_nur_eine_meldung() {
    let d = proj("b150");
    std::fs::write(d.join("a.bas"), "10 SCORE=SCORE+1\n20 PRINT SCORE\n30 SCORE=0\n").unwrap();
    let out = run(&d, "a.bas", "c64");
    let b150: Vec<&serde_json::Value> = out["diagnostics"].as_array().unwrap().iter().filter(|x| x["code"] == "B150").collect();
    assert_eq!(b150.len(), 1, "{out}");
    assert_eq!(b150[0]["line"], 1);
    assert!(b150[0]["message"].as_str().unwrap().contains("2, 3"), "{}", b150[0]);
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn medium_im_include_wird_zu_data_zeilen_und_altpfad_meldet_b194() {
    let d = proj("medium");
    std::fs::create_dir_all(d.join("graphics")).unwrap();
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/sprite_demo/graphics/hero.vcsprite"), d.join("graphics/hero.vcsprite")).unwrap();
    std::fs::write(d.join("a.bas"), "10 RESTORE\n20 READ A\n30 END\n#include \"graphics/hero.vcsprite\"\n").unwrap();
    let out = run(&d, "a.bas", "c64");
    assert_eq!(out["ok"], true, "{out}");
    assert!(listing(&d, "a").contains("DATA"), "{}", listing(&d, "a"));
    assert!(out["log"].to_string().contains("hero.vcsprite"), "{}", out["log"]);
    // Altpfad: geht weiter, meldet aber B194 mit dem Medium
    std::fs::create_dir_all(d.join("build/export")).unwrap();
    std::fs::write(d.join("build/export/hero.data.bas"), "@hero:\nDATA 1,2\n").unwrap();
    std::fs::write(d.join("b.bas"), "10 RESTORE\n20 END\n#include \"build/export/hero.data.bas\"\n").unwrap();
    let alt = run(&d, "b.bas", "c64");
    assert!(alt["diagnostics"].as_array().unwrap().iter().any(|x| x["code"] == "B194"), "{alt}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn export_stufe_laeuft_nur_bei_altpfad_der_gebauten_datei() {
    let d = proj("export");
    std::fs::create_dir_all(d.join("graphics")).unwrap();
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/sprite_demo/graphics/hero.vcsprite"), d.join("graphics/hero.vcsprite")).unwrap();
    // eine andere Datei im Projekt nennt den Altpfad; die gebaute nicht
    std::fs::write(d.join("alt.bas"), "10 END\n#include \"build/export/hero.data.bas\"\n").unwrap();
    std::fs::write(d.join("neu.bas"), "10 RESTORE\n20 END\n#include \"graphics/hero.vcsprite\"\n").unwrap();
    let out = run(&d, "neu.bas", "c64");
    assert_eq!(out["ok"], true, "{out}");
    assert!(!d.join("build/export/hero.data.bas").exists() && !d.join("build/export/hero.asm").exists(), "keine Exportdateien für ein Programm ohne Altpfad");
    std::fs::remove_dir_all(&d).ok();
}
