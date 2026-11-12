// SPDX-License-Identifier: GPL-3.0-or-later
use serde_json::json;
use std::path::PathBuf;
use vc_media::*;

fn profiles() -> Vec<Profile> { load_profiles(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")).unwrap() }
fn prof(id: &str) -> Profile { profiles().into_iter().find(|p| p.id == id).unwrap() }

#[test]
fn fcm_charset_uses_two_hex_digits_per_pixel() {
    let p = prof("mega65.char.fcm");
    let mut v = new_media(&p, "fcm");
    assert_eq!(v["frames"][0]["rows"][0].as_str().unwrap().len(), 16);
    v["frames"][0]["rows"][0] = json!("00ff1020000000ab");
    let b = encode(&parse_media(&v.to_string()).unwrap(), &p).unwrap();
    assert_eq!(b.len(), 256 * 64);
    assert_eq!(&b[..8], &[0x00, 0xff, 0x10, 0x20, 0x00, 0x00, 0x00, 0xab]);
}

#[test]
fn palette_is_encoded_in_register_form() {
    let p = prof("mega65.palette");
    let mut v = new_media(&p, "pal");
    v["entries"][5] = json!("#123456");
    let parts = encode_palette(&parse_media(&v.to_string()).unwrap()).unwrap();
    assert_eq!(parts.len(), 3);
    assert!(parts.iter().all(|(_, b)| b.len() == 256));
    // Halbbytes vertauscht: $12 -> $21 (rot), $34 -> $43, $56 -> $65
    assert_eq!((parts[0].1[5], parts[1].1[5], parts[2].1[5]), (0x21, 0x43, 0x65));
}

#[test]
fn play_media_becomes_basic_program() {
    let p = prof("mega65.play");
    let mut v = new_media(&p, "lied");
    v["steps"][0][0] = json!("C-4 00");   // Viertel C, dann E (Achtel), Pause
    v["steps"][4][0] = json!("E-4 01");
    v["steps"][6][0] = json!("===");
    v["steps"][8][1] = json!("A-3 02");   // Stimme 2 mit Pausenvorlauf
    v["steps"][3 + 8][1] = json!("---");
    let t = play::to_basic(&v, "lied").unwrap();
    assert!(t.contains("TEMPO 8"), "{t}");
    assert!(t.contains("ENVELOPE 0,0,9,8,4,0,2048"), "{t}");
    assert!(t.contains("PLAY \"V1T0O4QCT1IER") || t.contains("PLAY \"V1T0O4QC"), "{t}");
    assert!(t.contains("V2"), "{t}");
    assert!(t.lines().all(|l| l.split(' ').next().unwrap().parse::<u32>().is_ok()));
    // zu hohe Oktave wird gemeldet
    v["steps"][0][0] = json!("C-7 00");
    assert!(play::to_basic(&v, "lied").unwrap_err().contains("Oktave"));
}

#[test]
fn media_must_match_the_project_systems() {
    let root = std::env::temp_dir().join(format!("vc-sys-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("p.vcprj"), r#"{"format":1,"targets":[{"id":"c64","system":"c64"}]}"#).unwrap();
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let e = project::create(&root, &data, "x", "mega65.play", "assets").unwrap_err();
    assert!(e.contains("mega65") && e.contains("c64"), "{e}");
    project::create(&root, &data, "x", "c64.sprite.hires", "assets").unwrap();
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn wizard_creates_buildable_projects_for_every_template_and_system() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let list = wizard::templates(&data).unwrap();
    for t in list["templates"].as_array().unwrap() {
        for s in t["systems"].as_array().unwrap() {
            let (tid, sid) = (t["id"].as_str().unwrap(), s.as_str().unwrap());
            let dir = std::env::temp_dir().join(format!("vc-wiz-{}-{tid}-{sid}", std::process::id()));
            let r = wizard::create(&data, &dir, "mein-spiel", &[sid.to_string()], tid).unwrap();
            let proj: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("mein-spiel.vcprj")).unwrap()).unwrap();
            assert_eq!(proj["targets"][0]["system"], sid);
            for f in r["files"].as_array().unwrap() { assert!(dir.join(f.as_str().unwrap()).is_file(), "{tid}/{sid}: {f}"); }
            assert!(wizard::create(&data, &dir, "x", &[sid.to_string()], tid).unwrap_err().contains("schon ein Projekt"));
            std::fs::remove_dir_all(&dir).ok();
        }
    }
    let d = std::env::temp_dir().join(format!("vc-wiz-bad-{}", std::process::id()));
    assert!(wizard::create(&data, &d, "a b", &["c64".into()], "asm-hello").is_err());
    assert!(wizard::create(&data, &d, "ok", &["mega65".into()], "cartridge").unwrap_err().contains("gibt es nicht"));
}

#[test]
fn example_tune_exists_as_song_and_as_play_media() {
    // Das Beispiel-Musikstück des Trackers (tune4, vier SIDs) gibt es im MEGA65-Beispielprojekt auch für den PLAY-Tracker: sechs Stimmen, gleiche Hüllkurven
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/mega65/play_demo/music");
    let song: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("../../music_demo/music/tune4.vcsong")).unwrap()).unwrap();
    let play: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("tune_play.vcplay")).unwrap()).unwrap();
    assert_eq!(play["voices"], 6);
    assert_eq!(play["instruments"], song["instruments"]);
    let t = play::to_basic(&play, "tune_play").unwrap();
    for v in 1..=6 { assert!(t.contains(&format!("PLAY \"V{v}")), "Stimme {v} fehlt:\n{t}"); }
    assert!(t.lines().all(|l| l.len() <= 160), "BASIC-Zeile zu lang:\n{t}");
}
