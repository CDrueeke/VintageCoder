// SPDX-License-Identifier: GPL-3.0-or-later
use serde_json::{json, Value};
use std::path::PathBuf;
use vc_media::spriteimport::{self as si, SprOpts};
use vc_media::{load_profiles, profile, project};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn tmp(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_si_{n}_{}", std::process::id())); std::fs::create_dir_all(&d).unwrap(); d }

/// Sprite-Block (63 Datenbytes + Attribut): Zeile 0 = `row0`, sonst leer
fn block(row0: [u8; 3], attr: u8) -> Vec<u8> { let mut b = vec![0u8; 64]; b[..3].copy_from_slice(&row0); b[63] = attr; b }

/// SPD v1: Kopf (SPD, 1, Sprites-1, Animationen-1, Hintergrund, Multicolor 1, 2), danach die Blöcke
fn spd_v1() -> Vec<u8> {
    let mut f = b"SPD".to_vec(); f.extend([1, 1, 0, 0, 5, 8]);              // 2 Sprites, 1 Animation, Farben 0/5/8
    f.extend(block([0xF0, 0x0F, 0xAA], 0x07));                            // hires, Farbe 7
    f.extend(block([0x1B, 0xE4, 0x00], 0x80 | 0x02));                     // multicolor, Farbe 2
    f.extend([0, 0, 1, 0]); f                                             // Animationsinfo am Ende
}
/// SPD v5: 20 Byte Kopf (Flags, Sprites u16, Kacheln u16, Animationen, Kachelanimationen, Kachelgröße, Farben, Überlappungen)
fn spd_v5() -> Vec<u8> {
    let mut f = b"SPD".to_vec(); f.push(5);
    f.extend([0, 2, 0, 0, 0, 1, 0, 1, 1]); f.extend([0, 5, 8]); f.extend([0, 0, 0, 0]);
    assert_eq!(f.len(), 20);
    f.extend(block([0xF0, 0x0F, 0xAA], 0x07)); f.extend(block([0x1B, 0xE4, 0x00], 0x80 | 0x02)); f
}

#[test]
fn spd_versions_and_modes() {
    let profiles = load_profiles(&data()).unwrap(); let hires = profile(&profiles, "c64.sprite.hires").unwrap();
    for (name, bytes) in [("v1", spd_v1()), ("v5", spd_v5())] {
        let i = si::spd_info(&bytes).unwrap(); assert_eq!(i.sprites, 2, "{name}");
        let (sprites, colors, warn) = si::read_spd(&bytes).unwrap();
        assert_eq!(sprites.len(), 2); assert!(!sprites[0].mc && sprites[1].mc, "{name}");
        assert_eq!(&sprites[0].rows[0][..8], &[1, 1, 1, 1, 0, 0, 0, 0]); assert_eq!(&sprites[0].rows[0][8..16], &[0, 0, 0, 0, 1, 1, 1, 1]);
        assert_eq!(&sprites[1].rows[0][..8], &[0, 1, 2, 3, 3, 2, 1, 0]);
        assert_eq!(colors, [7, 5, 8], "Sprite-Farbe des ersten Sprites, Multicolor 1/2 aus dem Kopf");
        assert!(warn.iter().any(|w| w.contains("verschiedene Farben")), "{warn:?}");
        // als Asset: hires ist das Profil, das Multicolor-Frame bekommt `mode`
        let m = si::c64_media(&sprites, colors, hires, "held");
        assert!(m["frames"][0]["mode"].is_null()); assert_eq!(m["frames"][1]["mode"], "multicolor");
        assert_eq!(m["frames"][1]["rows"][0], "0123321000"[..].to_string() + "00");
        assert_eq!(m["colors"], json!({ "sprite": 7, "mc1": 5, "mc2": 8 }));
    }
    // Fehler
    assert!(si::spd_info(b"XYZ123456").is_err());
    let mut v9 = spd_v1(); v9[3] = 9; assert!(si::spd_info(&v9).unwrap_err().contains("Version 9"));
    let short = spd_v1()[..80].to_vec(); assert!(si::spd_info(&short).unwrap_err().contains("zu kurz"));
}

#[test]
fn raw_blocks_prg_and_offsets() {
    let mut raw = vec![]; for k in 0..3u8 { raw.extend(block([k + 1, 0, 0], 0)); }
    let s = si::read_raw(&raw, &SprOpts::default(), false).unwrap(); assert_eq!(s.len(), 3);
    assert_eq!(&s[2].rows[0][..8], &[0, 0, 0, 0, 0, 0, 1, 1]);
    assert_eq!(si::read_raw(&raw, &SprOpts { offset: 64, count: 1, ..Default::default() }, false).unwrap().len(), 1);
    assert!(si::read_raw(&raw, &SprOpts { count: 9, ..Default::default() }, false).unwrap_err().contains("nur 3"));
    assert!(si::read_raw(&raw, &SprOpts { offset: 190, ..Default::default() }, false).unwrap_err().contains("weniger als 64"));
    // Modus erzwingen: Multicolor liest 12 Pixel
    assert_eq!(si::read_raw(&raw, &SprOpts { mc: Some(true), ..Default::default() }, false).unwrap()[0].rows[0].len(), 12);
}

#[test]
fn c64_pmg_conversion_roundtrip() {
    let profiles = load_profiles(&data()).unwrap(); let pmg = profile(&profiles, "atari.pmg.single").unwrap();
    let mut rows = vec![vec![0u8; 24]; 21]; for y in 0..21 { for x in 0..24 { if x == y || x == 23 - y || y == 0 { rows[y][x] = 1; } } }
    let spr = si::C64Spr { mc: false, rows: rows.clone() };
    let mut warn = vec![];
    let m = si::c64_to_pmg(&spr, pmg, "p", &mut warn);
    assert_eq!(m["kind"], "pmg"); assert_eq!(m["frames"].as_array().unwrap().len(), 4);
    assert_eq!(m["frames"][0]["rows"][0], "11111111"); assert_eq!(m["frames"][3]["rows"][0], "00000000");
    assert_eq!(m["frames"][0]["rows"].as_array().unwrap().len(), 32, "auf die PMG-Höhe aufgefüllt");
    // zurück: Player 0..2 nebeneinander, auf 21 Zeilen gekürzt
    let mut w2 = vec![]; let back = si::pmg_to_c64(&m, &mut w2).unwrap();
    assert_eq!(back.rows, rows); assert!(!back.mc);
    // Multicolor wird beim Wechsel zu Hires gedoppelt und gemeldet
    let mc = si::C64Spr { mc: true, rows: { let mut r = vec![vec![0u8; 12]; 21]; r[0][0] = 2; r[0][1] = 3; r } };
    let mut w3 = vec![]; let m2 = si::c64_to_pmg(&mc, pmg, "p", &mut w3);
    assert_eq!(m2["frames"][0]["rows"][0], "11110000"); assert!(w3.iter().any(|w| w.contains("Mehrfarbigkeit")), "{w3:?}");
}

#[test]
fn import_sprite_into_the_project_for_c64_and_atari() {
    let (proj, src) = (tmp("proj"), tmp("src"));
    std::fs::write(src.join("fremd.spd"), spd_v1()).unwrap();
    let r = project::import_sprite(&proj, &data(), &src.join("fremd.spd"), "c64.sprite.multicolor", "aus_spd", "", &SprOpts::default()).unwrap();
    assert_eq!((r["path"].as_str(), r["from"].as_str(), r["count"].as_u64()), (Some("graphics/aus_spd.vcsprite"), Some("spd"), Some(2)));
    // Atari-Ziel: nur ein Sprite (hier das zweite, Multicolor)
    let r2 = project::import_sprite(&proj, &data(), &src.join("fremd.spd"), "atari.pmg.single", "aus_spd_pmg", "", &SprOpts { frame: 1, ..Default::default() }).unwrap();
    assert_eq!(r2["path"], "graphics/aus_spd_pmg.vcpmg");
    let w: Vec<&str> = r2["warnings"].as_array().unwrap().iter().filter_map(|x| x.as_str()).collect();
    assert!(w.iter().any(|x| x.contains("2 Sprites")) && w.iter().any(|x| x.contains("Mehrfarbigkeit")), "{w:?}");
    // PMG-Asset zurück in den C64 und PRG-Rohdaten (2 Byte Ladeadresse werden übersprungen)
    let r3 = project::import_sprite(&proj, &data(), &proj.join("graphics/aus_spd_pmg.vcpmg"), "c64.sprite.hires", "zurueck", "", &SprOpts::default()).unwrap();
    assert_eq!(r3["count"], 1);
    let mut prg = vec![0x00, 0x20]; prg.extend(block([0x80, 0, 0], 0)); prg.extend(block([0x40, 0, 0], 0));
    std::fs::write(src.join("daten.prg"), prg).unwrap();
    let r4 = project::import_sprite(&proj, &data(), &src.join("daten.prg"), "c64.sprite.hires", "aus_prg", "", &SprOpts { mc: Some(false), ..Default::default() }).unwrap();
    assert_eq!(r4["count"], 2);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(proj.join("graphics/aus_prg.vcsprite")).unwrap()).unwrap();
    assert_eq!(v["frames"][0]["rows"][0], "100000000000000000000000"); assert_eq!(v["frames"][1]["rows"][0], "010000000000000000000000");
    // vorhandene Datei, falsche Art, unbekanntes Sprite
    assert!(project::import_sprite(&proj, &data(), &src.join("fremd.spd"), "c64.sprite.hires", "aus_spd", "", &SprOpts::default()).is_err());
    assert!(project::import_sprite(&proj, &data(), &src.join("fremd.spd"), "c64.char.hires", "x", "", &SprOpts::default()).unwrap_err().contains("kein Sprite-Profil"));
    assert!(project::import_sprite(&proj, &data(), &src.join("fremd.spd"), "atari.pmg.single", "y", "", &SprOpts { frame: 5, ..Default::default() }).unwrap_err().contains("gibt es nicht"));
    std::fs::remove_dir_all(&proj).ok(); std::fs::remove_dir_all(&src).ok();
}
