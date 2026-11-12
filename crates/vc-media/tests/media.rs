// SPDX-License-Identifier: GPL-3.0-or-later
use serde_json::{json, Value};
use std::path::PathBuf;
use vc_media::*;

fn profiles() -> Vec<Profile> { load_profiles(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")).unwrap() }
fn data_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn media(v: &Value) -> Media { parse_media(&v.to_string()).unwrap() }

#[test]
fn profiles_load() {
    let p = profiles();
    assert_eq!(p.len(), 40);
    assert!(profile(&p, "vic20.char.multicolor").is_ok() && profile(&p, "vic20.char.hires16").is_ok());
    assert!(profile(&p, "vic20.char.hires").is_ok());
    assert!(profile(&p, "plus4.ted.song").is_ok());
    let mc = profile(&p, "c64.sprite.multicolor").unwrap();
    assert_eq!((mc.width, mc.height, mc.bpp, mc.block), (12, 21, 2, 64));
    assert!(profile(&p, "nix").is_err());
}

#[test]
fn sprite_hires_and_multicolor_encoding() {
    let p = profiles();
    let hi = profile(&p, "c64.sprite.hires").unwrap();
    let mut a = new_media(hi, "t");
    a["frames"][0]["rows"][0] = json!("111111110000000011111111");
    a["frames"][0]["rows"][20] = json!("100000000000000000000001");
    let b = encode(&media(&a), hi).unwrap();
    assert_eq!(b.len(), 64);
    assert_eq!(&b[..3], &[0xFF, 0x00, 0xFF]);
    assert_eq!(&b[60..63], &[0x80, 0x00, 0x01]);
    assert_eq!(b[63], 0, "Padding-Byte");
    let mc = profile(&p, "c64.sprite.multicolor").unwrap();
    let mut m = new_media(mc, "t");
    m["frames"][0]["rows"][0] = json!("012301230123");
    m["frames"][0]["rows"][1] = json!("333333333333");
    let b = encode(&media(&m), mc).unwrap();
    assert_eq!(&b[..3], &[0x1B, 0x1B, 0x1B]);
    assert_eq!(&b[3..6], &[0xFF, 0xFF, 0xFF]);
}

#[test]
fn decode_is_the_inverse_of_encode() {
    let p = profiles();
    for id in ["c64.sprite.hires", "c64.sprite.multicolor", "c64.char.hires", "c64.char.multicolor", "atari.char.antic2", "atari.pmg.single"] {
        let pr = profile(&p, id).unwrap();
        let bytes: Vec<u8> = (0..4096).map(|i| (i * 37 + 11) as u8).collect();
        let used = pr.width * pr.bpp / 8 * pr.height;
        let block = if pr.block > 0 { pr.block } else { used };
        let n = if pr.kind == "charset" { pr.count } else { 3 };
        // gültige Rohdaten: Nutzbytes zufällig, Rest 0
        let mut raw = vec![0u8; n * block];
        for i in 0..n { for j in 0..used { raw[i * block + j] = bytes[(i * block + j) % bytes.len()]; } }
        let a = decode(pr, &raw, "x").unwrap();
        assert_eq!(encode(&media(&a), pr).unwrap(), raw, "{id}");
    }
}

#[test]
fn charset_and_pmg_layout() {
    let p = profiles();
    let ch = profile(&p, "c64.char.hires").unwrap();
    let mut a = new_media(ch, "font");
    let glyph_a = ["00111100", "01100110", "11000011", "11000011", "11111111", "11000011", "11000011", "00000000"];
    a["frames"][1]["rows"] = json!(glyph_a.iter().map(|r| r.replace('1', "1")).collect::<Vec<_>>());
    let b = encode(&media(&a), ch).unwrap();
    assert_eq!(b.len(), 2048);
    assert_eq!(&b[8..16], &[0x3C, 0x66, 0xC3, 0xC3, 0xFF, 0xC3, 0xC3, 0x00]);
    let pm = profile(&p, "atari.pmg.single").unwrap();
    let mut s = new_media(pm, "ship");
    s["frames"][2]["rows"][0] = json!("00011000");
    s["frames"][2]["rows"][31] = json!("10000001");
    let b = encode(&media(&s), pm).unwrap();
    assert_eq!(b.len(), 4 * 256, "4 Spieler zu je 256 Bytes");
    assert_eq!(b[2 * 256], 0x18);
    assert_eq!(b[2 * 256 + 31], 0x81);
    s["height"] = json!(16);                                         // kleinere Höhe: Zeilenzahl muss passen
    assert!(encode(&media(&s), pm).is_err());
}

#[test]
fn invalid_media_are_rejected() {
    let p = profiles();
    let hi = profile(&p, "c64.sprite.hires").unwrap();
    let mut a = new_media(hi, "t");
    a["frames"][0]["rows"][3] = json!("1111");
    assert!(encode(&media(&a), hi).unwrap_err().contains("Pixel"));
    let mut b = new_media(hi, "t");
    b["frames"][0]["rows"][3] = json!("200000000000000000000000");
    assert!(encode(&media(&b), hi).unwrap_err().contains("ungültig"));
    assert!(parse_media("{\"format\": 2}").is_err() && parse_media("kein json").is_err());
}

#[test]
fn screens_and_tilemaps() {
    let p = profiles();
    let c = profile(&p, "c64.screen.text").unwrap();
    let mut s = new_media(c, "title");
    s["chars"][0][0] = json!(8);
    s["attrs"][24][39] = json!(7);
    let (chars, attrs) = encode_screen(&media(&s), c).unwrap();
    assert_eq!((chars.len(), attrs.len(), chars[0], attrs[999]), (1000, 1000, 8, 7));
    let at = profile(&p, "atari.screen.antic2").unwrap();
    let (c2, a2) = encode_screen(&media(&new_media(at, "x")), at).unwrap();
    assert_eq!((c2.len(), a2.len()), (960, 0));
    let t = profile(&p, "c64.tilemap.2x2").unwrap();
    let mut m = new_media(t, "level");
    m["tiles"] = json!([{ "name": "leer", "chars": [32, 32, 32, 32], "color": 1 }, { "name": "stein", "chars": [100, 101, 102, 103], "color": 12 }]);
    m["map"][0][0] = json!(1);
    let (tc, map, col) = encode_tilemap(&media(&m), t).unwrap();
    assert_eq!((tc, map.len(), map[0], col), (vec![32, 32, 32, 32, 100, 101, 102, 103], 240, 1, vec![1, 12]));
    m["map"][0][1] = json!(5);
    assert!(encode_tilemap(&media(&m), t).unwrap_err().contains("Kachel 5"));
}

#[test]
fn exporters() {
    let asm = export_asm("hero_spr", &[1, 2, 255], 2);
    assert_eq!(asm, "; erzeugt von VintageCoder – nicht von Hand ändern\nhero_spr:\n        .byte $01,$02\n        .byte $ff\n.const hero_spr_len = 3\n");
    assert_eq!(export_prg(0x2000, &[9, 8]), vec![0x00, 0x20, 9, 8]);
    assert_eq!(export_data(&(0..14).collect::<Vec<u8>>(), 1000, 10), "1000 DATA 0,1,2,3,4,5,6,7,8,9,10,11\n1010 DATA 12,13\n");
    assert_eq!(export_data_labeled("hero", &(0..14).collect::<Vec<u8>>()), "REM erzeugt von VintageCoder: 14 Bytes\n@hero:\nDATA 0,1,2,3,4,5,6,7,8,9,10,11\nDATA 12,13\n");
    assert_eq!(export_inc("font", 2048, Some(0x3000)), "; erzeugt von VintageCoder\n.const font_len = 2048\n.const font_addr = $3000\n");
}


#[test]
fn standard_charsets_are_imported_per_system() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let profiles = vc_media::load_profiles(&data).unwrap();
    // C64-ähnliche 2048 Bytes: Zeichen 0 = "@", Zeichen 1 = senkrechter Strich
    let mut rom = vec![0u8; 4096]; rom[..8].copy_from_slice(&[0x3c, 0x66, 0x6e, 0x6e, 0x60, 0x62, 0x3c, 0x00]); rom[8..16].copy_from_slice(&[0x18; 8]); rom[0x800 + 8..0x800 + 16].copy_from_slice(&[0xff; 8]);
    let c64 = vc_media::profile(&profiles, "c64.char.hires").unwrap();
    let a = vc_media::import_rom(c64, &rom, 0, "x").unwrap();
    assert_eq!(a["frames"].as_array().unwrap().len(), 256); assert_eq!(a["frames"][0]["rows"][0], "00111100"); assert_eq!(a["frames"][1]["rows"][3], "00011000");
    assert_eq!(vc_media::import_rom(c64, &rom, 0x800, "x").unwrap()["frames"][1]["rows"][0], "11111111", "zweiter Satz ab $800");
    // Atari: 128 Zeichen ab Offset
    let mut atari_rom = vec![0u8; 16384]; atari_rom[0x2008..0x2010].copy_from_slice(&[0x18; 8]);
    let atari = vc_media::profile(&profiles, "atari.char.antic2").unwrap();
    let b = vc_media::import_rom(atari, &atari_rom, 0x2000, "y").unwrap();
    assert_eq!(b["frames"].as_array().unwrap().len(), 128); assert_eq!(b["frames"][1]["rows"][0], "00011000");
    // MEGA65-Vollfarbe: gesetzte Pixel = Farbe 01, zwei Hex-Ziffern je Pixel
    let m65 = vc_media::profile(&profiles, "mega65.char.fcm").unwrap();
    let c = vc_media::import_rom(m65, &rom, 0, "z").unwrap();
    assert_eq!(c["frames"][0]["rows"][0], "0000010101010000"); assert_eq!(c["frames"].as_array().unwrap().len(), 256);
    assert!(vc_media::import_rom(c64, &rom[..100], 0, "x").is_err() && vc_media::import_rom(c64, &rom, 99999, "x").is_err());
    assert_eq!(vc_media::project::rom_candidates(16384)[0].1, "atari.char.antic2");
}


#[test]
fn charset_png_sheet_roundtrip() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let profiles = vc_media::load_profiles(&data).unwrap();
    let p = vc_media::profile(&profiles, "c64.char.hires").unwrap();
    let mut rom = vec![0u8; 2048]; rom[..8].copy_from_slice(&[0x3c, 0x66, 0x6e, 0x6e, 0x60, 0x62, 0x3c, 0x00]); rom[255 * 8..].copy_from_slice(&[0xaa; 8]);
    let a = vc_media::import_rom(p, &rom, 0, "c").unwrap();
    let png = vc_media::export_sheet(p, &a).unwrap();
    let b = vc_media::import_sheet(p, &png, "c", false).unwrap();
    assert_eq!(a["frames"], b["frames"], "Export und Import ergeben dasselbe");
    assert_ne!(vc_media::import_sheet(p, &png, "c", true).unwrap()["frames"], a["frames"], "invertiert");
    assert!(vc_media::import_sheet(p, &png[..20], "c", false).is_err());
    assert!(vc_media::import_sheet(vc_media::profile(&profiles, "c64.sprite.hires").unwrap(), &png, "c", false).is_err());
}


#[test]
fn new_charset_starts_with_the_systems_standard_set_and_can_be_blank() {
    let data = data_dir();
    synthetic_roms();
    let root = std::env::temp_dir().join(format!("vc_newcs_{}", std::process::id())); let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("p.vcprj"), r#"{"format":1,"project":{"name":"t"},"targets":[{"id":"c64","system":"c64"},{"id":"atari8","system":"atari8"}],"sources":[]}"#).unwrap();
    // C64: „Großbuchstaben und Grafik“ ist der Einschaltzustand
    let r = project::create(&root, &data, "font", "c64.char.hires", "graphics").unwrap();
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(r["path"].as_str().unwrap())).unwrap()).unwrap();
    assert_eq!((v["format"].clone(), v["base"].clone()), (2.into(), "rom:c64/chargen#upper".into()));
    assert!(v.get("frames").is_none(), "die Verweisdatei enthält keine Glyphen");
    let std = project::std_charset_media("c64.char.hires", "rom:c64/chargen#upper").unwrap();
    let loaded = parse_media(&v.to_string()).unwrap();
    assert_eq!(loaded.root["frames"], std["frames"]);
    assert_eq!(v["name"], "font");
    assert!(loaded.root["frames"].as_array().unwrap().iter().any(|f| f["rows"].as_array().unwrap().iter().any(|r| r.as_str().unwrap().contains('1'))), "nicht leer");
    // Atari: der Standardsatz; mehrere Sätze zur Auswahl, der Standard zuerst
    let r = project::create(&root, &data, "afont", "atari.char.antic2", "graphics").unwrap();
    let a: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(r["path"].as_str().unwrap())).unwrap()).unwrap();
    assert_eq!(a["base"], "rom:atari8/os#standard");
    let sets = project::std_charsets_for_profile(&data, "atari.char.antic2", true);
    assert!(sets.len() >= 2, "{sets:?}");   // OS XL/XE: Standard und International, OS B: Standard
    assert_eq!(project::std_charsets_for_profile(&data, "c64.char.hires", true).len(), 2);   // upper, lower
    // leer auf Wunsch
    let r = project::create_with(&root, &data, "leer", "c64.char.hires", "graphics", true).unwrap();
    let b: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(r["path"].as_str().unwrap())).unwrap()).unwrap();
    assert!(b.get("base").is_none());
    assert!(b["frames"].as_array().unwrap().iter().all(|f| f["rows"].as_array().unwrap().iter().all(|r| r.as_str().unwrap().chars().all(|c| c == '0'))));
}

#[test]
fn ted_song_exports_two_voices_with_envelope_instruments() {
    let p = profiles();
    let pr = profile(&p, "plus4.ted.song").unwrap();
    let song = vc_media::song::new_song(pr, "t");
    let parts = vc_media::song::encode(&song, pr).unwrap();
    let get = |n: &str| parts.iter().find(|(k, _)| k == n).unwrap().1.clone();
    assert_eq!(get("patterns").len(), 32 * 2 * 2, "32 Zeilen × 2 Stimmen × (Note, Instrument)");
    assert_eq!(get("ins").len(), 2 * 32, "Instrumente wie beim POKEY: 32 Bytes je Satz");
    assert_eq!(vc_media::song::consts("t", &song, pr, false).lines().find(|l| l.contains("channels")).unwrap(), ".const t_channels = 2");
}

#[test]
fn vic20_charset_profiles() {
    let p = profiles();
    // 8×8 einfarbig: 256 × 8 Bytes; Mehrfarb 4×8: je Zeile vier Pixel zu 2 Bit (00, 01, 10, 11 = $1B); 8×16: 256 × 16 Bytes
    for (id, len) in [("vic20.char.hires", 2048), ("vic20.char.multicolor", 2048), ("vic20.char.hires16", 4096)] {
        let pr = profile(&p, id).unwrap();
        assert_eq!(encode(&media(&new_media(pr, "t")), pr).unwrap().len(), len, "{id}");
    }
    let mc = profile(&p, "vic20.char.multicolor").unwrap();
    assert_eq!(mc.raw["values"], json!(["Hintergrund", "Rahmen", "Zeichenfarbe", "Hilfsfarbe"]));
    let mut m = new_media(mc, "t");
    m["frames"][0]["rows"][0] = json!("0123");
    assert_eq!(encode(&media(&m), mc).unwrap()[0], 0x1B);
    // Neuer Zeichensatz: einfarbig mit dem VC-20-Zeichen-ROM, die anderen leer
    let data = data_dir();
    synthetic_roms();
    let root = std::env::temp_dir().join(format!("vc_vic20cs_{}", std::process::id())); let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("p.vcprj"), r#"{"format":1,"project":{"name":"t"},"targets":[{"id":"vic20","system":"vic20"}],"sources":[]}"#).unwrap();
    let r = project::create(&root, &data, "font", "vic20.char.hires", "graphics").unwrap();
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(r["path"].as_str().unwrap())).unwrap()).unwrap();
    assert_eq!(v["base"], "rom:vic20/chargen#upper");
    assert_eq!(project::std_charsets_for_profile(&data, "vic20.char.hires", true).len(), 2);
    let r = project::create(&root, &data, "mc", "vic20.char.multicolor", "graphics").unwrap();
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(r["path"].as_str().unwrap())).unwrap()).unwrap();
    assert!(v.get("base").is_none());
    // PNG-Palette des VC-20: Farbe 3 ist Cyan (nicht das C64-Cyan)
    let pal = vc_media::import::palette(&data, "vic20", "pal").unwrap();
    assert_eq!(pal.len(), 16); assert_eq!(pal[3].1, [0xa0, 0xfe, 0xf8]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn vic20_screen_profile() {
    let p = profiles();
    let sp = profile(&p, "vic20.screen.text").unwrap();
    assert_eq!((sp.width, sp.height, sp.target.as_str()), (22, 23, "vic20"));
    let mut m = new_media(sp, "title");
    m["chars"][0][0] = json!(1); m["chars"][22][21] = json!(255);
    let (chars, attrs) = vc_media::encode_screen(&media(&m), sp).unwrap();
    assert_eq!((chars.len(), attrs.len()), (22 * 23, 22 * 23));
    assert_eq!((chars[0], chars[22 * 23 - 1]), (1, 255));
    assert!(attrs.iter().all(|a| *a == 0), "Standard-Zeichenfarbe schwarz");
    assert_eq!(m["colors"], json!({ "border": 3, "bg": 1, "fg": 0 }));
}

#[test]
fn vic20_tilemap_profile() {
    let p = profiles();
    let t = profile(&p, "vic20.tilemap.2x2").unwrap();
    assert_eq!((t.target.as_str(), t.width, t.height), ("vic20", 11, 11));
    let mut m = new_media(t, "level");
    m["tiles"] = json!([{ "name": "leer", "chars": [32, 32, 32, 32], "color": 1 }, { "name": "stein", "chars": [100, 101, 102, 103], "color": 2 }]);
    m["map"][10][10] = json!(1);
    let (tc, map, col) = encode_tilemap(&media(&m), t).unwrap();
    assert_eq!((tc.len(), map.len(), map[120], col), (8, 121, 1, vec![1, 2]));
}

#[test]
fn vic_song_exports_four_voices_with_envelope_instruments() {
    let p = profiles();
    let pr = profile(&p, "vic20.vic.song").unwrap();
    assert_eq!(pr.target, "vic20");
    let song = vc_media::song::new_song(pr, "t");
    let parts = vc_media::song::encode(&song, pr).unwrap();
    let get = |n: &str| parts.iter().find(|(k, _)| k == n).unwrap().1.clone();
    assert_eq!(get("patterns").len(), 32 * 4 * 2, "32 Zeilen × 4 Stimmen × (Note, Instrument)");
    assert_eq!(get("ins").len(), 2 * 32, "Instrumente wie beim POKEY: 32 Bytes je Satz");
    assert_eq!(vc_media::song::consts("t", &song, pr, false).lines().find(|l| l.contains("channels")).unwrap(), ".const t_channels = 4");
}

/// Synthetische ROM-Dateien (richtige Größe, Muster) für die Zeichensatz-Tests; beide Tests laufen im selben Prozess und teilen sich den ROM-Dienst
fn synthetic_roms() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let dir = std::env::temp_dir().join(format!("vc_media_roms_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (rel, size) in [("c64/chargen.bin", 4096), ("vic20/chargen.bin", 4096), ("atari8/os-xl.rom", 16384), ("atari8/os-b.rom", 10240)] {
            let p = dir.join(rel); std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, (0..size).map(|i| (i % 253 + 1) as u8).collect::<Vec<u8>>()).unwrap();
        }
        vc_media::romref::configure(vc_roms::RomEnv { vc_roms: Some(dir), ..Default::default() });
    });
}
