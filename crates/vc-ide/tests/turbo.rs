// SPDX-License-Identifier: GPL-3.0-or-later
//! Turbo BASIC XL starten, Etappen E0–E2 (Prüfung, Bereitstellen, Bau; Konzept „Turbo BASIC XL starten“): selbst gebaute Abbilder, keine Fremddatei im Repository.
//! Die Tests mit der Diskette des Inhabers (`VC_TURBO_ATR` oder `assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr`) arbeiten nur auf Kopien und
//! überspringen sich ohne Abbild. Der Emulator-Orakeltest steht in `turbo_oracle.rs`.
use std::path::{Path, PathBuf};
use vc_disk::Atr;
use vc_ide::build::Ctx;
use vc_ide::projfile::{build_project, ProjectFile};
use vc_ide::turbo::{self, TurboErr};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc_turbo_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}
fn data() -> PathBuf { vc_ide::find_data_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap() }
fn bytes(n: usize, seed: u8) -> Vec<u8> { (0..n).map(|i| (i as u8).wrapping_mul(13).wrapping_add(seed)).collect() }

/// Selbst gebaute Diskette im Aufbau der Turbo-Diskette: Systemdateien (geschützt), `AUTORUN.SYS` mit Kennung, Beispieldateien; `free` Sektoren bleiben frei
fn fake_disk(free: usize, with_signature: bool) -> Atr {
    let mut a = Atr::new_dos2();
    a.dos2_add("DOS.SYS", &bytes(4625, 1)).unwrap();
    a.dos2_add("DUP.SYS", &bytes(5126, 2)).unwrap();
    let mut auto = bytes(6000, 3);
    if with_signature { auto[100..100 + 33].copy_from_slice(b"TURBO-BASIC XL 1.5 (c) 1985 Frank"); }
    a.dos2_add("AUTORUN.SYS", &auto).unwrap();
    a.dos2_add("BEISPIEL.TUR", &bytes(125 * 60, 4)).unwrap();
    a.dos2_add("MAGIC.TUR", &bytes(125 * 50, 5)).unwrap();
    let fill = a.free_sectors() - free;
    if fill > 0 { a.dos2_add("FUELLER.DAT", &bytes(125 * fill, 6)).unwrap(); }
    for n in ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS", "BEISPIEL.TUR", "MAGIC.TUR"] { a.dos2_set_locked(n, true).unwrap(); }
    assert_eq!(a.free_sectors(), free);
    a
}

fn project(name: &str, turbo: Option<&str>) -> PathBuf {
    let root = tmp(name);
    let t = turbo.map(|t| format!(",\n  \"turbo\": {t}")).unwrap_or_default();
    std::fs::write(root.join("prog.vcprj"), format!(r#"{{ "format": 1, "project": {{"name": "Turbo-Test"}},
  "targets": [{{ "id": "atari8", "system": "atari8", "model": "800xl", "variant": "pal", "cpu": "sally" }}],
  "sources": [{{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["prog.tur"] }}],
  "main": "prog.tur"{t} }}"#)).unwrap();
    std::fs::write(root.join("prog.tur"), "PRINT \"AUTORUN OK\"\nEXEC hallo\nEND\nPROC hallo\n  PRINT \"AUS PROC\"\nENDPROC\n").unwrap();
    root
}

fn load(root: &Path) -> ProjectFile { ProjectFile::load(&vc_ide::projfile::find(root).unwrap()).unwrap() }
fn build(root: &Path) -> serde_json::Value { let d = data(); build_project(&Ctx { root, data: &d }, &load(root), None) }

fn owner_disk() -> Option<PathBuf> {
    std::env::var("VC_TURBO_ATR").map(PathBuf::from).ok().or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().map(|a| a.join("assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr")).find(|p| p.exists()))
}

// ---- Prüfung ----

#[test]
fn pruefung_erkennt_die_kennung_und_die_version() {
    let r = turbo::check_bytes(&fake_disk(100, true).to_bytes()).unwrap();
    assert!(r.dos2 && r.is_turbo && r.problems.is_empty(), "{r:?}");
    assert_eq!(r.version.as_deref(), Some("1.5"));
    assert_eq!(r.free_sectors, 100);
    assert!(r.known.is_none(), "selbst gebautes Abbild ist nicht in der Liste");
    assert!(!r.has_program);
    let text = r.lines().join("\n");
    assert!(text.contains("1.5") && text.contains("100") && text.contains(&r.sha256), "{text}");
    let r = turbo::check_bytes(&fake_disk(100, false).to_bytes()).unwrap();
    assert!(!r.is_turbo && r.version.is_none());
    assert!(turbo::check_bytes(b"kein ATR, viel zu kurz").is_err());
    assert!(turbo::check_bytes(&Atr::blank().to_bytes()).map(|r| !r.dos2 && !r.is_turbo).unwrap());
}

#[test]
fn liste_bekannter_abbilder_enthaelt_die_diskette_des_inhabers() {
    let k = turbo::known_list();
    assert!(k.iter().any(|d| d.sha256 == "814ebeb7a2cd84cfa8d5366c97d94ae7504a5bfe80523bd1a295b3cf1cfb9f5f" && d.version == "1.5" && d.size == 92176), "{k:?}");
    assert!(k.iter().all(|d| d.sha256.len() == 64 && d.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())));
    assert_eq!(turbo::sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn pruefung_der_diskette_des_inhabers() {
    let Some(p) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let r = turbo::check_file(&p).unwrap();
    assert!(r.is_turbo && r.problems.is_empty(), "{r:?}");
    assert_eq!(r.version.as_deref(), Some("1.5"));
    assert_eq!(r.known.as_ref().map(|k| k.version.as_str()), Some("1.5"));
    assert_eq!(r.free_sectors, 103);
    assert_eq!(r.entries.len(), 13, "Anhang A: 12 Dateien, PUTGET.TUR steht zweimal im Verzeichnis (zweiter Eintrag nie abgeschlossen)");
    assert_eq!(r.hints.len(), 2, "{:?}", r.hints);
    assert!(r.lines().iter().any(|l| l.contains("103")));
}

// ---- Bau ----

#[test]
fn bau_schreibt_autorun_bas_bitgleich_und_die_zusatzdateien() {
    let root = project("bau", Some(r#"{ "disk": "turbo/T.atr", "files": ["daten/hallo.dat:HALLO.DAT", "daten/zwei.dat"] }"#));
    std::fs::create_dir_all(root.join("turbo")).unwrap(); std::fs::create_dir_all(root.join("daten")).unwrap();
    std::fs::write(root.join("turbo/T.atr"), fake_disk(300, true).to_bytes()).unwrap();
    std::fs::write(root.join("daten/hallo.dat"), bytes(500, 9)).unwrap();
    std::fs::write(root.join("daten/zwei.dat"), bytes(10, 8)).unwrap();
    let orig = std::fs::read(root.join("turbo/T.atr")).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["turbo"]["file"], "build/prog.atr");
    assert_eq!(r["turbo"]["program"], "AUTORUN.BAS");
    let tok = std::fs::read(root.join("build/prog.bas")).unwrap();
    let a = Atr::from_bytes(&std::fs::read(root.join("build/prog.atr")).unwrap()).unwrap();
    assert_eq!(a.dos2_read("AUTORUN.BAS").unwrap(), tok, "AUTORUN.BAS bit-gleich zum Ergebnis des Tokenizers");
    assert_eq!(a.dos2_read("HALLO.DAT").unwrap(), bytes(500, 9));
    assert_eq!(a.dos2_read("ZWEI.DAT").unwrap(), bytes(10, 8));
    assert!(a.dos2_check_split().0.is_empty());
    assert_eq!(a.free_sectors() as u64, r["turbo"]["free_sectors"].as_u64().unwrap());
    assert_eq!(a.free_sectors(), 300 - tok.len().div_ceil(125) - 4 - 1);
    // Systemdateien der Kopie bit-gleich
    let src = Atr::from_bytes(&orig).unwrap();
    for n in ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS", "MAGIC.TUR"] { assert_eq!(a.dos2_read(n).unwrap(), src.dos2_read(n).unwrap(), "{n}"); }
    assert_eq!(std::fs::read(root.join("turbo/T.atr")).unwrap(), orig, "die Kopie im Projekt wird beim Bauen nie verändert");
    assert!(r["turbo"]["line"].as_str().unwrap().contains("AUTORUN.BAS"));
}

#[test]
fn vorhandenes_autorun_bas_wird_ersetzt_geschuetztes_nicht() {
    let mut d = fake_disk(300, true);
    d.dos2_add("AUTORUN.BAS", &bytes(1000, 7)).unwrap();
    let root = project("ersetzen", Some(r#"{ "disk": "turbo/T.atr" }"#));
    std::fs::create_dir_all(root.join("turbo")).unwrap();
    std::fs::write(root.join("turbo/T.atr"), d.to_bytes()).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    let a = Atr::from_bytes(&std::fs::read(root.join("build/prog.atr")).unwrap()).unwrap();
    assert_eq!(a.dos2_read("AUTORUN.BAS").unwrap(), std::fs::read(root.join("build/prog.bas")).unwrap());
    assert_eq!(a.dos2_entries().iter().filter(|e| e.name == "AUTORUN.BAS").count(), 1);
    // geschützt: Fehler mit Meldung, kein Abbild
    d.dos2_set_locked("AUTORUN.BAS", true).unwrap();
    std::fs::write(root.join("turbo/T.atr"), d.to_bytes()).unwrap();
    let _ = std::fs::remove_file(root.join("build/prog.atr"));
    let r = build(&root);
    assert_eq!(r["ok"], false); assert!(r["turbo_error"].as_str().unwrap().contains("AUTORUN.BAS"), "{r}");
    assert!(!root.join("build/prog.atr").exists());
}

#[test]
fn fehlende_diskette_ist_ein_hinweis_und_kein_fehler() {
    for t in [None, Some(r#"{ "disk": "turbo/gibtsnicht.atr" }"#)] {
        let root = project("fehlt", t);
        let r = build(&root);
        assert_eq!(r["ok"], true, "{r}");
        assert!(r.get("turbo").is_none() || r["turbo"].is_null());
        let m = r["turbo_missing"].as_str().expect("Hinweis").to_string();
        assert!(m.contains("Turbo") && m.contains("vcc turbo provide"), "{m}");
        assert!(root.join("build/prog.bas").is_file(), "die .bas wird trotzdem gebaut");
        assert!(!root.join("build/prog.atr").exists());
    }
}

#[test]
fn platzmangel_meldet_den_bedarf_und_den_weg_ueber_das_verkleinern() {
    let root = project("platz", Some(r#"{ "disk": "turbo/T.atr", "files": ["gross.dat:GROSS.DAT"] }"#));
    std::fs::create_dir_all(root.join("turbo")).unwrap();
    std::fs::write(root.join("turbo/T.atr"), fake_disk(30, true).to_bytes()).unwrap();
    std::fs::write(root.join("gross.dat"), bytes(125 * 120, 1)).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], false);
    let m = r["turbo_error"].as_str().unwrap();
    assert!(m.contains("12") && m.contains("30") && m.contains("vcc turbo reduce"), "{m}");
    assert!(!root.join("build/prog.atr").exists());
    // verkleinern (T2): nur die Projektkopie; danach passt es
    let removed = turbo::reduce_project_copy(&root).unwrap();
    assert!(removed.contains(&"MAGIC.TUR".to_string()) && removed.contains(&"FUELLER.DAT".to_string()), "{removed:?}");
    let t = load(&root).turbo.unwrap();
    assert!(t.reduced);
    let c = Atr::from_bytes(&std::fs::read(root.join("turbo/T.atr")).unwrap()).unwrap();
    assert_eq!(c.dos2_entries().iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS"]);
    assert!(c.dos2_check_split().0.is_empty());
    assert_eq!(turbo::sha256_hex(&std::fs::read(root.join("turbo/T.atr")).unwrap()), t.sha256, "neue Prüfsumme der Kopie notiert");
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    // zu groß auch nach dem Verkleinern: Hinweis auf größeres Abbild
    std::fs::write(root.join("gross.dat"), bytes(125 * 700, 1)).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], false);
    assert!(r["turbo_error"].as_str().unwrap().contains("130 KB"), "{r}");
}

#[test]
fn veraenderte_kopie_und_unbekanntes_abbild_geben_hinweise() {
    let root = project("hinweise", Some(r#"{ "disk": "turbo/T.atr", "sha256": "0000" }"#));
    std::fs::create_dir_all(root.join("turbo")).unwrap();
    std::fs::write(root.join("turbo/T.atr"), fake_disk(300, false).to_bytes()).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    let notes = r["turbo"]["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2, "{notes:?}");
}

#[test]
fn kein_turbo_projekt_baut_keine_diskette() {
    let root = tmp("anders");
    std::fs::write(root.join("a.bas"), "10 PRINT 1\n").unwrap();
    std::fs::write(root.join("a.vcprj"), r#"{ "format": 1, "project": {"name": "x"}, "targets": [{"id":"atari8","system":"atari8"}], "sources": [{"targets":["atari8"],"kind":"basic","files":["a.bas"]}], "main": "a.bas" }"#).unwrap();
    assert!(!turbo::is_turbo_project(&load(&root)));
    let r = build(&root);
    assert!(r.get("turbo").is_none() && r.get("turbo_missing").is_none(), "{r}");
    assert!(turbo::start(&root, &data()).is_err());
}

#[test]
fn projektdatei_prueft_namen_und_behaelt_das_feld() {
    let root = project("pf", Some(r#"{ "disk": "turbo/T.atr", "program": "autorun.bas", "files": ["a.dat:zu_langer_name.bas", "b.dat:AUTORUN.BAS"] }"#));
    let (errs, warns) = load(&root).validate_getrennt(&root);
    let e = errs.join("\n");
    assert!(e.contains("ZU_LANGER_NAME.BAS") && e.contains("AUTORUN.BAS"), "{e}");
    assert_eq!(warns.iter().filter(|w| w.contains(".dat")).count(), 2, "{warns:?}");
    // Feld bleibt beim Speichern erhalten, alte Projekte ohne Feld bleiben unverändert
    let pf = load(&root);
    let back: ProjectFile = serde_json::from_str(&serde_json::to_string(&pf).unwrap()).unwrap();
    assert_eq!(back.turbo, pf.turbo);
    let ohne = load(&project("pf2", None));
    assert!(ohne.turbo.is_none() && !serde_json::to_string(&ohne).unwrap().contains("\"turbo\":"));
}

// ---- Bereitstellen und Verkleinern ----

#[test]
fn bereitstellen_kopiert_traegt_ein_und_laesst_das_original_unberuehrt() {
    let root = project("prov", None);
    let src_dir = tmp("prov_src");
    let src = src_dir.join("Meine_Turbo.atr");
    std::fs::write(&src, fake_disk(100, true).to_bytes()).unwrap();
    let before = std::fs::read(&src).unwrap();
    let p = turbo::provide(&root, &src, &Default::default()).unwrap();
    assert_eq!(p.disk, "turbo/Meine_Turbo.atr");
    assert_eq!(std::fs::read(root.join(&p.disk)).unwrap(), before, "Kopie bit-gleich (nicht verkleinert)");
    assert_eq!(std::fs::read(&src).unwrap(), before, "Original unverändert");
    assert!(p.gitignore_added);
    assert!(std::fs::read_to_string(root.join(".gitignore")).unwrap().lines().any(|l| l == "turbo/*.atr"));
    let t = load(&root).turbo.unwrap();
    assert_eq!((t.disk.as_str(), t.sha256.as_str(), t.reduced), ("turbo/Meine_Turbo.atr", turbo::sha256_hex(&before).as_str(), false));
    // zweites Mal: Kopie gibt es schon; mit replace geht es, .gitignore wird nicht doppelt beschrieben
    assert!(turbo::provide(&root, &src, &Default::default()).unwrap_err().contains("Meine_Turbo.atr"));
    let p2 = turbo::provide(&root, &src, &turbo::ProvideOpts { replace: true, ..Default::default() }).unwrap();
    assert!(!p2.gitignore_added);
    assert_eq!(std::fs::read_to_string(root.join(".gitignore")).unwrap().matches("turbo/*.atr").count(), 1);
    // die Quelle darf nicht die Kopie selbst sein
    assert!(turbo::provide(&root, &root.join(&p.disk), &turbo::ProvideOpts { replace: true, ..Default::default() }).is_err());
    // danach baut das Projekt
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    assert!(r["turbo"]["version"] == "1.5");
}

#[test]
fn bereitstellen_mit_verkleinern_und_gitignore_bleibt_erhalten() {
    let root = project("prov2", None);
    std::fs::write(root.join(".gitignore"), "build/\n*.log").unwrap();
    let src = tmp("prov2_src").join("T.atr");
    std::fs::write(&src, fake_disk(100, true).to_bytes()).unwrap();
    let p = turbo::provide(&root, &src, &turbo::ProvideOpts { reduce: true, ..Default::default() }).unwrap();
    assert!(p.removed.contains(&"MAGIC.TUR".to_string()));
    assert!(p.free_sectors > 100);
    let g = std::fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(g.starts_with("build/\n*.log\n") && g.contains("turbo/*.atr"), "{g}");
    assert_eq!(Atr::from_bytes(&std::fs::read(&src).unwrap()).unwrap().free_sectors(), 100, "Original nie verkleinert");
    assert!(load(&root).turbo.unwrap().reduced);
}

#[test]
fn bereitstellen_warnt_bei_fremden_abbildern_erst_mit_anyway() {
    let root = project("prov3", None);
    let src = tmp("prov3_src").join("Fremd.atr");
    std::fs::write(&src, fake_disk(100, false).to_bytes()).unwrap();
    let e = turbo::provide(&root, &src, &Default::default()).unwrap_err();
    assert!(e.contains("Turbo BASIC XL"), "{e}");
    assert!(!root.join("turbo").exists() && load(&root).turbo.is_none(), "nichts geschrieben");
    turbo::provide(&root, &src, &turbo::ProvideOpts { anyway: true, ..Default::default() }).unwrap();
    // Verkleinern ohne die drei Systemdateien: Fehler
    let mut a = fake_disk(100, true);
    a.dos2_remove("DUP.SYS", true).unwrap();
    assert!(turbo::reduce(&mut a).unwrap_err().contains("DUP.SYS"));
    // kein ATR: Fehler des Containers
    let junk = tmp("prov3_junk").join("x.atr"); std::fs::write(&junk, b"nichts").unwrap();
    assert!(turbo::provide(&root, &junk, &turbo::ProvideOpts { anyway: true, replace: true, ..Default::default() }).is_err());
}

#[test]
fn mit_der_diskette_des_inhabers_bereitstellen_reduzieren_und_bauen() {
    let Some(orig) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let orig_bytes = std::fs::read(&orig).unwrap();
    let root = project("owner", None);
    let p = turbo::provide(&root, &orig, &Default::default()).unwrap();
    assert!(p.report.known.is_some());
    assert_eq!(p.free_sectors, 103);
    // 103 Sektoren frei: ein großes Programm passt nicht, mit Hinweis auf das Verkleinern
    let mut pf = load(&root);
    pf.turbo.as_mut().unwrap().files = vec!["gross.dat:GROSS.DAT".into()];
    pf.save(&vc_ide::projfile::find(&root).unwrap()).unwrap();
    std::fs::write(root.join("gross.dat"), bytes(125 * 200, 1)).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], false);
    assert!(r["turbo_error"].as_str().unwrap().contains("vcc turbo reduce"), "{r}");
    let removed = turbo::reduce_project_copy(&root).unwrap();
    assert_eq!(removed.len(), 9, "{removed:?}");
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    let a = Atr::from_bytes(&std::fs::read(root.join("build/prog.atr")).unwrap()).unwrap();
    assert_eq!(a.dos2_read("GROSS.DAT").unwrap(), bytes(125 * 200, 1));
    assert!(a.dos2_check_split().0.is_empty());
    assert_eq!(std::fs::read(&orig).unwrap(), orig_bytes, "Original unverändert");
}

// ---- Start (Kommandozeile des Emulators; der Emulator selbst wird im Orakeltest gestartet) ----

#[test]
fn start_bootet_die_diskette_ohne_eingebautes_basic() {
    let (cmd, args) = match vc_ide::launch::insert_disk_command(Path::new("/x/build/prog.atr"), "atari8", "pal") {
        Ok(x) => x,
        Err(e) => { eprintln!("kein atari800 gefunden, übersprungen: {e}"); return }
    };
    let line = args.join(" ");
    if cmd.to_ascii_lowercase().contains("altirra") { assert!(line.contains("/nobasic") && line.contains("/disk"), "{line}"); }
    else {
        assert!(args.contains(&"-nobasic".to_string()) && args.contains(&"-xl".to_string()), "{line}");
        assert_eq!(args.last().map(String::as_str), Some("/x/build/prog.atr"), "das Abbild kommt in Laufwerk 1: {line}");
        assert!(!args.contains(&"-run".to_string()) && !args.contains(&"-basic".to_string()));
    }
    let (_, ntsc) = vc_ide::launch::insert_disk_command(Path::new("/x/p.atr"), "atari8", "ntsc").unwrap();
    assert!(ntsc.contains(&"-ntsc".to_string()) || ntsc.contains(&"/ntsc".to_string()));
}

#[test]
fn start_ohne_diskette_meldet_bereitstellen() {
    let root = project("start_fehlt", None);
    let e = turbo::start(&root, &data()).unwrap_err();
    assert!(e.contains("Turbo") && e.contains("vcc turbo provide"), "{e}");
    let _: Option<TurboErr> = None;
}

#[test]
fn vorschau_der_reduzierung_veraendert_nichts() {
    let bytes = fake_disk(100, true).to_bytes();
    let (names, free) = turbo::reduce_preview(&bytes).expect("verkleinerbar");
    assert!(names.contains(&"MAGIC.TUR".to_string()) && !names.contains(&"DOS.SYS".to_string()), "{names:?}");
    assert!(free > 100, "danach mehr frei: {free}");
    assert_eq!(turbo::check_bytes(&bytes).unwrap().free_sectors, 100, "die Vorschau ändert die Bytes nicht");
    assert!(turbo::reduce_preview(&Atr::new_dos2().to_bytes()).is_none(), "ohne Systemdateien keine Verkleinerung");
}
