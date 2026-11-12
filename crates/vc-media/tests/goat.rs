// SPDX-License-Identifier: GPL-3.0-or-later
use vc_media::{goat, load_profiles, profile, project};
use std::path::PathBuf;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn ins(ad: u8, sr: u8, wave: u8, pulse: u8, name: &str) -> Vec<u8> {
    let mut v = vec![ad, sr, wave, pulse, 0, 0, 0, 2, 9];
    let mut n = name.as_bytes().to_vec(); n.resize(16, 0); v.extend(n); v
}
fn table(left: &[u8], right: &[u8]) -> Vec<u8> { let mut v = vec![left.len() as u8]; v.extend(left); v.extend(right); v }
fn pattern(rows: &[[u8; 4]]) -> Vec<u8> { let mut v = vec![rows.len() as u8 + 1]; for r in rows { v.extend(r); } v.extend([0xFF, 0, 0, 0]); v }
fn orderlist(items: &[u8]) -> Vec<u8> { let mut v = vec![items.len() as u8 - 1]; v.extend(items); v }

/// Kleiner Song im GTS5-Format: 3 Stimmen mit eigenen Orderlisten, 2 Instrumente, 2 Patterns à 8 Zeilen
fn sng(chn: usize) -> Vec<u8> {
    let mut f = b"GTS5".to_vec();
    f.extend(vec![0u8; 96]); f.push(1);
    let lists: [&[u8]; 3] = [&[0, 0xFF, 0], &[0xF2, 1, 0xFF, 0], &[0xD1, 0, 0xFF, 0]];
    for c in 0..chn { f.extend(orderlist(lists.get(c).copied().unwrap_or(&[0, 0xFF, 0]))); }
    f.push(2);
    f.extend(ins(0x09, 0xA4, 1, 1, "Lead")); f.extend(ins(0x00, 0xF0, 3, 0, ""));
    f.extend(table(&[0x41, 0xFF, 0x21, 0xFF], &[0x80, 0, 0x80, 0]));       // Wellentabelle
    f.extend(table(&[0x86, 0xFF], &[0x40, 0]));                            // Pulstabelle
    f.extend(table(&[], &[])); f.extend(table(&[], &[]));
    f.push(2);
    f.extend(pattern(&[[0x90, 1, 0xF, 4], [0xBD, 0, 0, 0], [0x94, 2, 0, 0], [0xBD, 0, 0, 0], [0xBE, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0]]));
    f.extend(pattern(&[[0x8B, 1, 0, 0], [0xBD, 0, 1, 5], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0], [0xBD, 0, 0, 0]]));
    f
}

#[test]
fn imports_goattracker_song() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sid.song").unwrap();
    let (v, warn) = goat::import(&sng(3), p, "gt").unwrap();
    assert_eq!((v["speed"].as_u64(), v["rows"].as_u64(), v["loop"].as_u64()), (Some(4), Some(8), Some(0)));
    // Stimme 1 und 3 spielen Pattern 0 (Stimme 3 doppelt), Stimme 2 Pattern 1 um 2 Halbtöne höher
    assert_eq!(v["order"], serde_json::json!([0, 1]));
    let r = &v["patterns"][0]["rows"];
    assert_eq!(r[0], serde_json::json!(["C-4 00", "A-3 00", "C-4 00"]));
    assert_eq!(r[1][0], "--- --"); assert_eq!(r[2][0], "E-4 01"); assert_eq!(r[4][0], "=== --");
    // zweiter Block: nur Stimme 3 (wiederholtes Pattern 0), die anderen sind zu Ende
    assert_eq!(v["patterns"][1]["rows"][0], serde_json::json!(["--- --", "--- --", "C-4 00"]));
    let i = &v["instruments"];
    assert_eq!((i[0]["name"].as_str(), i[0]["wave"].as_str(), i[0]["pw"].as_u64()), (Some("Lead"), Some("pulse"), Some(1600)));
    assert_eq!((i[0]["a"].as_u64(), i[0]["d"].as_u64(), i[0]["s"].as_u64(), i[0]["r"].as_u64()), (Some(0), Some(9), Some(10), Some(4)));
    assert_eq!((i[1]["name"].as_str(), i[1]["wave"].as_str(), i[1]["pw"].as_u64()), (Some("Instrument 02"), Some("saw"), Some(2048)));
    assert!(warn.iter().any(|w| w.contains("Befehl 1XY")), "{warn:?}");
    assert!(warn.iter().any(|w| w.contains("unterschiedlich lang")), "{warn:?}");
}

#[test]
fn goattracker_errors_and_stereo() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sid.song").unwrap();
    assert!(goat::import(b"GTS2....", p, "x").unwrap_err().contains("alte GoatTracker"));
    assert!(goat::import(b"RIFF", p, "x").unwrap_err().contains("keine GoatTracker-Datei"));
    let mut t = sng(3); t.truncate(t.len() - 5);
    assert!(goat::import(&t, p, "x").is_err());
    // Stereo-Datei (6 Orderlisten): erkannt, nur die ersten drei Stimmen
    let (v, w) = goat::import(&sng(6), p, "x").unwrap();
    assert_eq!(v["patterns"][0]["rows"][0].as_array().unwrap().len(), 3);
    assert!(w.iter().any(|m| m.contains("Stereo")), "{w:?}");
}

#[test]
fn import_song_from_sng_for_both_chips() {
    let dir = std::env::temp_dir().join(format!("vc_gt_{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("fremd.sng"); std::fs::write(&src, sng(3)).unwrap();
    let proj = dir.join("proj"); std::fs::create_dir_all(&proj).unwrap();
    let r = project::import_song(&proj, &data(), &src, "c64.sid.song", "gt_sid", "assets").unwrap();
    assert_eq!((r["path"].as_str(), r["from"].as_str()), (Some("music/gt_sid.vcsong"), Some("goattracker")));
    // auch für den POKEY-Tracker: GoatTracker -> SID -> POKEY
    let r = project::import_song(&proj, &data(), &src, "atari.pokey.song", "gt_pokey", "assets").unwrap();
    let t = std::fs::read_to_string(proj.join(r["path"].as_str().unwrap())).unwrap();
    assert!(vc_media::parse_media(&t).unwrap().profile == "atari.pokey.song");
    assert!(project::import_song(&proj, &data(), &src, "c64.sid.song", "gt_sid", "assets").is_err());
    std::fs::remove_dir_all(&dir).ok();
}
