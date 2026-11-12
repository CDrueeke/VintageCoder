// SPDX-License-Identifier: GPL-3.0-or-later
use vc_media::{import, png, *};
use serde_json::json;
use std::path::PathBuf;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn decodes_real_compressed_png() {
    let img = png::decode(&std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/atari800-window.png")).unwrap()).unwrap();
    assert_eq!((img.w, img.h), (336, 272));
    let i = (58 * img.w + 168) * 4; // weisse Markerzeile der Farbtafel
    assert!(img.rgba[i] > 240 && img.rgba[i + 1] > 240 && img.rgba[i + 2] > 240);
}

#[test]
fn encode_decode_roundtrip() {
    let px: Vec<u8> = (0..4 * 5 * 4).map(|i| (i * 7) as u8).collect();
    let img = png::decode(&png::encode_rgba(4, 5, &px)).unwrap();
    assert_eq!(img.rgba, px);
}

#[test]
fn measured_atari_palette_is_complete() {
    for sys in ["pal", "ntsc"] { let p = import::palette(&data(), "atari8", sys).unwrap(); assert_eq!(p.len(), 128); assert_eq!(p[1].0, 2); }
    assert_eq!(import::palette(&data(), "c64", "pal").unwrap().len(), 16);
}

#[test]
fn imports_multicolor_sprite_and_reports_conflict() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sprite.multicolor").unwrap();
    let mut media = new_media(p, "t");
    // 24x21 Bild: links rot, Mitte grün, rechts blau, ganz rechts gelb (4 Farben > 3 darstellbar), oben links transparent
    let (w, h) = (24usize, 21usize); let mut rgba = Vec::new();
    for y in 0..h { for x in 0..w {
        let c = match x { 0..=5 => [0x81, 0x33, 0x38, 255], 6..=11 => [0x56, 0xac, 0x4d, 255], 12..=17 => [0x2e, 0x2c, 0x9b, 255], _ => [0xed, 0xf1, 0x71, 255] };
        rgba.extend_from_slice(&if y < 2 && x < 2 { [0, 0, 0, 0] } else { c });
    } }
    let r = import::import_png(&png::encode_rgba(w, h, &rgba), &mut media, p, &data(), 0, true, "pal").unwrap();
    assert_eq!(r["imageColors"], 4); assert_eq!(r["allowed"], 3); assert_eq!(r["conflict"], true);
    let rows = media["frames"][0]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 21); assert_eq!(rows[0].as_str().unwrap().len(), 12);
    assert!(rows[0].as_str().unwrap().starts_with('0') && !rows[5].as_str().unwrap().starts_with('0'));
    assert!(encode(&parse_media(&media.to_string()).unwrap(), p).is_ok());
}

#[test]
fn imports_atari_pmg_with_ntsc_palette() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "atari.pmg.single").unwrap();
    let mut media = new_media(p, "t"); let (w, h) = (8usize, 32usize);
    let rgba: Vec<u8> = (0..w * h).flat_map(|i| if (i % w) < 4 { [255, 255, 255, 255] } else { [0, 0, 0, 0] }).collect();
    let r = import::import_png(&png::encode_rgba(w, h, &rgba), &mut media, p, &data(), 1, true, "ntsc").unwrap();
    assert_eq!(r["conflict"], false);
    assert_eq!(media["frames"][1]["rows"][0], json!("11110000"));
}
