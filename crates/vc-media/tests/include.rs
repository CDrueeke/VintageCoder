// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien als Include (Konzept „Include von Medien“, Teil A): `resolve_include` je Medienart und Format, Zwischenspeicher, Fehler.
use std::path::{Path, PathBuf};
use vc_media::include::{is_media_path, resolve_include, IncFormat, IncludeError, CACHE_DIR};
use vc_media::project;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn proj(tag: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("vc_inc_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r); std::fs::create_dir_all(&r).unwrap(); r
}
fn make(root: &Path, name: &str, profile: &str) -> PathBuf {
    let v = project::create(root, &data(), name, profile, "").unwrap();
    PathBuf::from(v["path"].as_str().unwrap())
}

#[test]
fn erkennt_medienendungen() {
    for p in ["graphics/hero.vcsprite", "g/a.vcpmg", "g/f.vcchar", "g/t.vctiles", "g/s.vcscreen", "music/m.vcsong", "music/p.vcplay", "g/c.vcpal", "assets/x.asset.json"] { assert!(is_media_path(p), "{p}"); }
    for p in ["main.asm", "lib/a.inc", "x.bas", "build/export/hero.asm"] { assert!(!is_media_path(p), "{p}"); }
}

#[test]
fn sprite_asm_und_data_mit_marken() {
    let root = proj("sprite");
    let rel = make(&root, "hero", "c64.sprite.hires");
    let a = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!a.cached);
    assert!(a.text.contains("hero:"), "Marke hero fehlt:\n{}", a.text);
    assert!(a.text.contains("hero_"), "Konstanten fehlen");
    assert_eq!(a.source, root.join(&rel));
    assert_eq!(a.deps, vec![root.join(&rel)]);
    let d = resolve_include(&root, &data(), &rel, IncFormat::Data, None).unwrap();
    assert!(d.text.contains("@hero:") && d.text.contains("DATA "), "{}", d.text);
    // nichts landet in build/export
    assert!(!root.join("build/export").exists(), "Include darf build/export nicht anlegen");
    assert!(root.join(CACHE_DIR).join("hero.asm").is_file() && root.join(CACHE_DIR).join("hero.data.bas").is_file());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ergebnis_gleich_dem_export() {
    let root = proj("gleich");
    let rel = make(&root, "hero", "c64.sprite.hires");
    let inc = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    project::export(&root, &data(), rel.to_str().unwrap(), "asm", None, None, 1000, 10).unwrap();
    let alt = std::fs::read_to_string(root.join("build/export/hero.asm")).unwrap();
    assert_eq!(inc.text, alt, "Include und Export müssen denselben Text liefern (Animationstabellen folgen im selben Text)");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn zwischenspeicher_treffer_und_neuer_inhalt() {
    let root = proj("cache");
    let rel = make(&root, "hero", "c64.sprite.hires");
    let a = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    let b = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!a.cached && b.cached && a.text == b.text);
    // Inhalt ändern → neuer Text, alter Stand wird verworfen
    let p = root.join(&rel);
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let row = v["frames"][0]["rows"][0].as_str().unwrap().to_string();
    v["frames"][0]["rows"][0] = serde_json::json!(format!("1{}", &row[1..]));
    std::fs::write(&p, v.to_string()).unwrap();
    let c = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!c.cached && c.text != a.text, "geänderte Mediendatei muss neu umgewandelt werden");
    let n = std::fs::read_dir(root.join(CACHE_DIR)).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with("hero-")).count();
    assert_eq!(n, 1, "alter Zwischenspeicher-Stand bleibt nicht liegen");
    // Zwischenspeicher löschen ist unschädlich
    std::fs::remove_dir_all(root.join(CACHE_DIR)).unwrap();
    let d = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!d.cached && d.text == c.text);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn alle_arten_asm() {
    let root = proj("arten");
    for (n, pr) in [("pm", "atari.pmg.single"), ("font", "c64.char.hires"), ("title", "c64.screen.text"), ("tune", "c64.sid.song")] {
        let sys = if pr.starts_with("atari") { "atari8" } else { "c64" };
        let _ = sys;
        // Projekt ohne .vcprj: keine Einschränkung des Ziels
        let rel = make(&root, n, pr);
        let r = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap_or_else(|e| panic!("{pr}: {e}"));
        assert!(!r.text.trim().is_empty(), "{pr}: leer");
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn pokey_song_und_play_und_palette() {
    let root = proj("songs");
    let rel = make(&root, "music1", "atari.pokey.song");
    let r = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!r.text.is_empty());
    let rel = make(&root, "pal1", "mega65.palette");
    let r = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert!(!r.text.is_empty());
    let rel = make(&root, "pl", "mega65.play");
    let r = resolve_include(&root, &data(), &rel, IncFormat::Data, None).unwrap();
    assert!(!r.text.is_empty(), "PLAY liefert ein BASIC-Stück");
    // PLAY als Assembler ist kein gültiger Export: Fehler mit Hinweis, kein leeres Ergebnis
    let e = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap_err();
    assert!(matches!(e, IncludeError::Failed(_)), "{e:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn kacheln_und_screen_bank_haengen_von_anderen_medien_ab() {
    let root = proj("kacheln");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/jumprun/graphics");
    std::fs::create_dir_all(root.join("graphics")).unwrap();
    for f in ["chars.vcchar", "hg.vctiles", "levels.vcscreen"] { std::fs::copy(src.join(f), root.join("graphics").join(f)).unwrap(); }
    let set = resolve_include(&root, &data(), Path::new("graphics/hg.vctiles"), IncFormat::Asm, None).unwrap();
    assert!(!set.text.is_empty());
    let bank = resolve_include(&root, &data(), Path::new("graphics/levels.vcscreen"), IncFormat::Asm, None).unwrap();
    assert!(bank.deps.len() >= 3 && bank.deps.contains(&root.join("graphics/hg.vctiles")), "Bank hängt vom Kachelsatz ab: {:?}", bank.deps);
    // Zweiter Aufruf: Treffer. Kachelsatz ändern: Bank wird neu umgewandelt
    assert!(resolve_include(&root, &data(), Path::new("graphics/levels.vcscreen"), IncFormat::Asm, None).unwrap().cached);
    let p = root.join("graphics/hg.vctiles");
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    v["tiles"][0]["name"] = serde_json::json!("umbenannt");
    std::fs::write(&p, v.to_string()).unwrap();
    assert!(!resolve_include(&root, &data(), Path::new("graphics/levels.vcscreen"), IncFormat::Asm, None).unwrap().cached);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn fehler_nicht_gefunden_und_kein_medium() {
    let root = proj("fehler");
    let e = resolve_include(&root, &data(), Path::new("graphics/fehlt.vcsprite"), IncFormat::Asm, None).unwrap_err();
    assert!(matches!(e, IncludeError::NotFound(_)) && e.to_string().contains("graphics/fehlt.vcsprite"), "{e:?}");
    std::fs::write(root.join("kaputt.vcsprite"), "kein json").unwrap();
    let e = resolve_include(&root, &data(), Path::new("kaputt.vcsprite"), IncFormat::Asm, None).unwrap_err();
    assert!(matches!(e, IncludeError::Failed(_)) && e.to_string().contains("kaputt.vcsprite"), "{e:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn profil_passt_nicht_zum_ziel() {
    let root = proj("ziel");
    let rel = make(&root, "hero", "c64.sprite.hires");
    // ausdrückliches Ziel des Baus
    let e = resolve_include(&root, &data(), &rel, IncFormat::Asm, Some("atari8")).unwrap_err();
    assert!(matches!(e, IncludeError::Failed(_)) && e.to_string().contains("atari8") && e.to_string().contains("c64"), "{e:?}");
    assert!(resolve_include(&root, &data(), &rel, IncFormat::Asm, Some("c64")).is_ok());
    // Ziele aus der Projektdatei
    std::fs::write(root.join("p.vcprj"), r#"{"targets":[{"system":"atari8"}]}"#).unwrap();
    let e = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap_err();
    assert!(matches!(e, IncludeError::Failed(_)), "{e:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn absoluter_pfad_und_projektrelativer_pfad_gleich() {
    let root = proj("abs");
    let rel = make(&root, "hero", "c64.sprite.hires");
    let a = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    let b = resolve_include(&root, &data(), &root.join(&rel), IncFormat::Asm, None).unwrap();
    assert_eq!(a.text, b.text);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn speichermodus_memory_schreibt_nichts_ins_projekt() {
    use vc_media::include::{resolve_include_with, CacheMode};
    let root = proj("memory");
    let rel = make(&root, "hero", "c64.sprite.hires");
    let a = resolve_include_with(&root, &data(), &rel, IncFormat::Asm, None, CacheMode::Memory).unwrap();
    let b = resolve_include_with(&root, &data(), &rel, IncFormat::Asm, None, CacheMode::Memory).unwrap();
    assert!(!a.cached && b.cached && a.text == b.text);
    assert!(!root.join("build").exists(), "Memory-Modus legt nichts im Projektordner an");
    // dasselbe Ergebnis wie im Plattenmodus
    let c = resolve_include(&root, &data(), &rel, IncFormat::Asm, None).unwrap();
    assert_eq!(a.text, c.text);
    // kein Arbeitsordner bleibt im Projekt liegen
    assert!(std::fs::read_dir(root.join(CACHE_DIR)).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().starts_with(".work")));
    let _ = std::fs::remove_dir_all(&root);
}
