// SPDX-License-Identifier: GPL-3.0-or-later
use vc_media::{load_profiles, profile, sid};
use std::path::PathBuf;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn reg(n: u8) -> u16 { (440.0 * 2f64.powf((n as f64 - 58.0) / 12.0) * 16_777_216.0 / 985_248.0).round() as u16 }

/// PSID v2 mit einer kleinen Spielroutine: Stimme 1 spielt C-4, A-4, E-4, C-5 je 6 Bilder (5 Bilder Gate an, 1 Bild aus)
fn psid(songs: u16) -> Vec<u8> {
    let mut h = b"PSID".to_vec();
    h.extend([0, 2, 0, 0x7C, 0x10, 0x00, 0x10, 0x00, 0x10, 0x20]);             // Version 2, Datenbeginn $7C, Laden/Init $1000, Play $1020
    h.extend(songs.to_be_bytes()); h.extend([0, 1, 0, 0, 0, 0]);               // Songs, Startsong 1, Speed 0 (Bildtakt)
    for s in ["Testlied", "Tester", "2026"] { let mut b = s.as_bytes().to_vec(); b.resize(32, 0); h.extend(b); }
    h.extend([0, 4, 0, 0, 0, 0]);                                         // Flags: PAL, Startseite, Länge, 2./3. SID
    assert_eq!(h.len(), 0x7C);
    let mut m = vec![0u8; 0x180];
    // init: lda #$0F sta $D418 ; lda #$09 sta $D405 ; lda #$F0 sta $D406 ; lda #0 sta $FB ; rts
    let init = [0xA9, 0x0F, 0x8D, 0x18, 0xD4, 0xA9, 0x09, 0x8D, 0x05, 0xD4, 0xA9, 0xF0, 0x8D, 0x06, 0xD4, 0xA9, 0x00, 0x85, 0xFB, 0x60];
    m[..init.len()].copy_from_slice(&init);
    // play: ldx $FB ; lda $1100,x sta $D400 ; lda $1130,x sta $D401 ; lda $1160,x sta $D404 ; inc $FB ; rts
    let play = [0xA6, 0xFB, 0xBD, 0x00, 0x11, 0x8D, 0x00, 0xD4, 0xBD, 0x30, 0x11, 0x8D, 0x01, 0xD4, 0xBD, 0x60, 0x11, 0x8D, 0x04, 0xD4, 0xE6, 0xFB, 0x60];
    m[0x20..0x20 + play.len()].copy_from_slice(&play);
    for (k, n) in [49u8, 58, 53, 61].iter().enumerate() {
        for f in 0..6 { let i = k * 6 + f; let r = reg(*n); m[0x100 + i] = r as u8; m[0x130 + i] = (r >> 8) as u8; m[0x160 + i] = if f < 5 { 0x41 } else { 0x40 }; }
    }
    h.extend(m); h
}

#[test]
fn reads_psid_header() {
    let (i, off) = sid::info(&psid(3)).unwrap();
    assert_eq!((i.title.as_str(), i.author.as_str(), i.songs, i.start_song, i.play, i.rsid, i.ntsc, off), ("Testlied", "Tester", 3, 1, 0x1020, false, false, 0x7C));
    assert!(sid::info(b"MThd0000").is_err());
}

#[test]
fn records_player_into_song() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sid.song").unwrap();
    let (v, warn) = sid::import(&psid(1), p, "rec", &sid::SidOpts { song: 0, seconds: 2 }).unwrap();
    assert_eq!(v["speed"], 6);
    let r = &v["patterns"][0]["rows"];
    assert_eq!(r[0], serde_json::json!(["C-4 00", "--- --", "--- --"]));
    assert_eq!((r[1][0].as_str(), r[2][0].as_str(), r[3][0].as_str()), (Some("A-4 00"), Some("E-4 00"), Some("C-5 00")));
    assert_eq!(r[4][0], "--- --");
    let i = &v["instruments"][0];
    assert_eq!((i["wave"].as_str(), i["a"].as_u64(), i["d"].as_u64(), i["s"].as_u64(), i["r"].as_u64()), (Some("pulse"), Some(0), Some(9), Some(15), Some(0)));
    assert!(warn[0].contains("Testlied") && warn[0].contains("Annäherung"), "{warn:?}");
}

#[test]
fn rejects_bad_input() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sid.song").unwrap();
    assert!(sid::import(&psid(1), p, "x", &sid::SidOpts { song: 2, seconds: 1 }).unwrap_err().contains("Subtune 2"));
    // Stille Datei: init und play tun nichts (nur RTS) -> keine Töne
    let mut s = psid(1); for b in s[0x7C..].iter_mut() { *b = 0; } s[0x7C] = 0x60; s[0x7C + 0x20] = 0x60;
    assert!(sid::import(&s, p, "x", &sid::SidOpts::default()).unwrap_err().contains("keine Töne"));
    // Endlosschleife in der Spielroutine -> Zeitüberschreitung
    let mut s = psid(1); s[0x7C + 0x20..0x7C + 0x23].copy_from_slice(&[0x4C, 0x20, 0x10]);
    assert!(sid::import(&s, p, "x", &sid::SidOpts { song: 0, seconds: 1 }).unwrap_err().contains("Zeitüberschreitung"));
}

#[test]
fn import_song_from_sid_file_for_both_chips() {
    let dir = std::env::temp_dir().join(format!("vc_sid_{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("fremd.sid"); std::fs::write(&src, psid(1)).unwrap();
    let proj = dir.join("proj"); std::fs::create_dir_all(&proj).unwrap();
    let opts = sid::SidOpts { song: 1, seconds: 2 };
    let r = vc_media::project::import_song_with(&proj, &data(), &src, "c64.sid.song", "aus_sid", "", &opts).unwrap();
    assert_eq!((r["path"].as_str(), r["from"].as_str()), (Some("music/aus_sid.vcsong"), Some("sid")));
    // auch im POKEY-Tracker: SID-Aufnahme -> POKEY
    let r = vc_media::project::import_song_with(&proj, &data(), &src, "atari.pokey.song", "aus_sid_p", "", &opts).unwrap();
    let t = std::fs::read_to_string(proj.join(r["path"].as_str().unwrap())).unwrap();
    assert_eq!(vc_media::parse_media(&t).unwrap().profile, "atari.pokey.song");
    std::fs::remove_dir_all(&dir).ok();
}
