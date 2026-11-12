// SPDX-License-Identifier: GPL-3.0-or-later
use serde_json::{json, Value};
use std::path::PathBuf;
use vc_media::{encode, load_profiles, parse_media, profile, project, sprite};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn blank_hires() -> Vec<String> { (0..21).map(|_| "0".repeat(24)).collect() }
fn blank_mc() -> Vec<String> { (0..21).map(|_| "0".repeat(12)).collect() }

/// Hires-Sprite mit einem eingemischten Multicolor-Frame und einer Animation
fn mixed() -> Value {
    let (mut h, mut m) = (blank_hires(), blank_mc());
    h[0] = format!("1111{}", "0".repeat(20)); m[0] = format!("123{}", "0".repeat(9));
    json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "held", "colors": { "sprite": 7, "mc1": 2, "mc2": 6 },
        "frames": [ { "rows": h.clone() }, { "mode": "multicolor", "rows": m.clone() }, { "rows": h }, { "mode": "multicolor", "rows": m } ],
        "animations": [ { "name": "lauf", "mode": "pingpong", "steps": [ { "frame": 0, "delay": 4 }, { "frame": 1, "delay": 3 }, { "frame": 2, "delay": 5 } ] },
                        { "name": "boom", "mode": "once", "steps": [ { "frame": 3, "delay": 8 } ] } ] })
}

#[test]
fn encode_mixes_hires_and_multicolor_frames() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sprite.hires").unwrap();
    let a = parse_media(&mixed().to_string()).unwrap();
    let bytes = encode(&a, p).unwrap();
    assert_eq!(bytes.len(), 4 * 64);
    assert_eq!(&bytes[0..3], &[0xF0, 0x00, 0x00], "Frame 0 hires: vier Pixel links");
    assert_eq!(&bytes[64..67], &[0x6C, 0x00, 0x00], "Frame 1 multicolor: Werte 1,2,3 = 01 10 11");
    assert_eq!(&bytes[128..131], &[0xF0, 0x00, 0x00]);
    // falsche Zeilenbreite für den Modus wird gemeldet
    let mut bad = mixed(); bad["frames"][1]["rows"][0] = json!("1".repeat(24));
    assert!(encode(&parse_media(&bad.to_string()).unwrap(), p).unwrap_err().contains("Frame 1"));
}

#[test]
fn mode_conversion_keeps_shape() {
    let hires = vec![vec![1, 1, 0, 0, 0, 1], vec![0, 0, 0, 0, 0, 0]];
    let mc = sprite::hires_to_mc(&hires);
    assert_eq!(mc, vec![vec![2, 0, 2], vec![0, 0, 0]]);   // Wert 2 = Sprite-Farbe (Bitpaar 10)
    assert_eq!(sprite::mc_to_hires(&vec![vec![1, 2, 0, 3]]), vec![vec![1, 1, 1, 1, 0, 0, 1, 1]]);
}

#[test]
fn animation_expansion_timing_and_index_shifts() {
    let mut v = mixed();
    let anims = sprite::parse_animations(&v).unwrap();
    assert_eq!(anims.len(), 2);
    // hin und zurück: 0,1,2,1 – Endbilder nicht doppelt
    assert_eq!(anims[0].expanded(), vec![(0, 4), (1, 3), (2, 5), (1, 3)]);
    assert_eq!(anims[0].total_ticks(), 15);
    assert_eq!((anims[0].step_at(0.0), anims[0].step_at(0.09), anims[0].step_at(0.15), anims[0].step_at(0.29), anims[0].step_at(0.31)), (0, 1, 2, 3, 0));
    // einmal: bleibt auf dem letzten Schritt
    assert_eq!(anims[1].step_at(100.0), 0);
    // Frame an Position 1 einfügen: alle Frames ab 1 rücken
    sprite::animations_after_insert(&mut v, 1);
    assert_eq!(sprite::parse_animations(&json!({ "frames": [0,1,2,3,4], "animations": v["animations"] })).unwrap()[0].steps, vec![(0, 4), (2, 3), (3, 5)]);
    // Frame 2 löschen: Schritt entfällt, spätere rücken nach
    sprite::animations_after_remove(&mut v, 2);
    assert_eq!(v["animations"][0]["steps"].as_array().unwrap().len(), 2);
    // Fehler: unbekanntes Frame, doppelter Name, Dauer
    assert!(sprite::parse_animations(&json!({ "frames": [{}], "animations": [{ "name": "a", "steps": [{ "frame": 3 }] }] })).unwrap_err().contains("gibt es nicht"));
    assert!(sprite::parse_animations(&json!({ "frames": [{}], "animations": [{ "name": "a", "steps": [] }, { "name": "a", "steps": [] }] })).unwrap_err().contains("doppelt"));
    assert!(sprite::parse_animations(&json!({ "frames": [{}], "animations": [{ "name": "a", "steps": [{ "frame": 0, "delay": 0 }] }] })).unwrap_err().contains("Dauer"));
    assert!(sprite::parse_animations(&json!({ "frames": [{}], "animations": [{ "name": "1x", "steps": [] }] })).unwrap_err().contains("ungültig"));
}

#[test]
fn export_writes_mode_and_animation_tables() {
    let root = std::env::temp_dir().join(format!("vc_spr_{}", std::process::id())); std::fs::create_dir_all(root.join("graphics")).unwrap();
    std::fs::write(root.join("graphics/held.vcsprite"), mixed().to_string()).unwrap();
    let r = project::export(&root, &data(), "graphics/held.vcsprite", "asm", None, None, 10, 10).unwrap();
    assert!(r["ok"] == true);
    let asm = &std::fs::read_to_string(root.join("build/export/held.asm")).unwrap();
    assert!(asm.contains(".const held_count = 4"), "{asm}");
    assert!(asm.contains(".const held_multicolor_mask = $0a"), "Frames 1 und 3 sind Multicolor: {asm}");
    assert!(!asm.contains("_anim_") && !asm.contains("held_modes"), "Tabellen stehen nicht in der Datei der Sprite-Daten");
    // Tabellen in eigener Datei
    let t = &std::fs::read_to_string(root.join("build/export/held.anim.asm")).unwrap();
    assert!(t.contains("held_modes:\n        .byte 0,1,0,1"), "{t}");
    assert!(t.contains("held_anim_lauf_ptr:\n        .byte ((held & $3fff) / 64) + 0,((held & $3fff) / 64) + 1,((held & $3fff) / 64) + 2,((held & $3fff) / 64) + 1"), "{t}");
    assert!(t.contains("held_anim_lauf_delay:\n        .byte 4,3,5,3") && t.contains(".const held_anim_lauf_len = 4"), "{t}");
    assert!(t.contains(".const held_anim_boom_mode = 1"), "{t}");
    // ohne Animationen und ohne gemischte Frames entsteht keine zweite Datei und die Daten bleiben wie vorher
    let plain = serde_json::json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "schlicht", "colors": { "sprite": 7 }, "frames": [ { "rows": blank_hires() } ] });
    std::fs::write(root.join("graphics/schlicht.vcsprite"), plain.to_string()).unwrap();
    project::export(&root, &data(), "graphics/schlicht.vcsprite", "asm", None, None, 10, 10).unwrap();
    assert!(!root.join("build/export/schlicht.anim.asm").exists());
    std::fs::remove_dir_all(&root).ok();
}
