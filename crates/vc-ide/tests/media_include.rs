// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien als Include im Bau (Konzept „Include von Medien“, Teil C): gleiche Bytes wie über build/export, kein build/export, Neubau bei Änderung, Altpfad-Hinweis.
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn project(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vc_mi_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(root.join("graphics")).unwrap();
    let hires: Vec<String> = (0..21).map(|_| "0".repeat(24)).collect(); let mc: Vec<String> = (0..21).map(|_| "0".repeat(12)).collect();
    let sprite = serde_json::json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "held", "colors": { "sprite": 7 },
        "frames": [ { "rows": hires }, { "mode": "multicolor", "rows": mc }, { "rows": hires } ],
        "animations": [ { "name": "lauf", "mode": "pingpong", "steps": [ { "frame": 0, "delay": 4 }, { "frame": 1, "delay": 3 }, { "frame": 2, "delay": 5 } ] } ] });
    std::fs::write(root.join("graphics/held.vcsprite"), sprite.to_string()).unwrap();
    root
}
fn run(root: &PathBuf, src: &str) -> serde_json::Value {
    std::fs::write(root.join("main.asm"), src).unwrap();
    build(&Ctx { root, data: &data() }, "main.asm", None).unwrap()
}

#[test]
fn include_der_mediendatei_gibt_dieselben_bytes_wie_der_alte_pfad() {
    let root = project("bytes");
    let neu = run(&root, ".target c64\n.org $2000\n.include \"graphics/held.vcsprite\"\n");
    assert_eq!(neu["ok"], true, "{neu}");
    let prg_neu = std::fs::read(root.join("build/main.prg")).unwrap();
    assert!(!root.join("build/export").exists(), "ohne alten Pfad entsteht kein build/export");
    assert!(root.join("build/.media-cache/held.asm").is_file());
    let alt = run(&root, ".target c64\n.org $2000\n.include \"build/export/held.asm\"\n.include \"build/export/held.anim.asm\"\n");
    assert_eq!(alt["ok"], true, "{alt}");
    assert_eq!(std::fs::read(root.join("build/main.prg")).unwrap(), prg_neu, "Include der Mediendatei = alter Export, Byte für Byte");
    assert!(root.join("build/export/held.asm").is_file(), "der alte Pfad braucht den Export weiter (Übergang)");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn alter_pfad_warnt_mit_dem_medium() {
    let root = project("w208");
    let v = run(&root, ".target c64\n.org $2000\n.include \"build/export/held.asm\"\n");
    assert_eq!(v["ok"], true, "{v}");
    let msgs = v.to_string();
    assert!(msgs.contains("W208") && msgs.contains("graphics/held.vcsprite"), "{msgs}");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn geaendertes_medium_baut_neu_und_zweiter_lauf_nutzt_den_zwischenspeicher() {
    let root = project("neubau");
    let src = ".target c64\n.org $2000\n.include \"graphics/held.vcsprite\"\n";
    let v = run(&root, src);
    assert!(v.to_string().contains("Medium umgewandelt"), "{v}");
    let a = std::fs::read(root.join("build/main.prg")).unwrap();
    let v2 = run(&root, src);
    assert!(v2.to_string().contains("Zwischenspeicher"), "{v2}");
    let p = root.join("graphics/held.vcsprite");
    let mut m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    m["frames"][0]["rows"][0] = serde_json::json!(format!("1{}", "0".repeat(23)));
    std::fs::write(&p, m.to_string()).unwrap();
    run(&root, src);
    assert_ne!(std::fs::read(root.join("build/main.prg")).unwrap(), a, "geändertes Medium → anderes Programm");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn medium_eines_anderen_systems_ist_ein_fehler_mit_hinweis() {
    let root = project("ziel");
    let v = run(&root, ".target atari8\n.org $2000\n.include \"graphics/held.vcsprite\"\n");
    assert_eq!(v["ok"], false, "{v}");
    let s = v.to_string();
    assert!(s.contains("E219") && s.contains("atari8") && s.contains("c64"), "{s}");
    std::fs::remove_dir_all(&root).ok();
}
