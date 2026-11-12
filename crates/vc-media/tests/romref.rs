// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeichensatz-Verweise auf ROMs und Format 2 – mit synthetischen ROM-Daten (keine echten ROMs nötig).
//! Der ROM-Dienst ist prozessweit; alle Schritte stehen deshalb in einem Test.
use serde_json::{json, Value};
use vc_media::romref::{self, RomRef};
use vc_roms::RomEnv;

fn synthetic_chargen() -> Vec<u8> { (0..4096).map(|i| ((i * 7 + i / 8) % 251) as u8).collect() }

fn bits(b: u8) -> String { format!("{b:08b}") }

#[test]
fn verweis_auflösen_überschreiben_speichern_und_fehlendes_rom() {
    let dir = std::env::temp_dir().join(format!("vc-media-romref-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("c64")).unwrap();
    let rom = synthetic_chargen();
    std::fs::write(dir.join("c64/chargen.bin"), &rom).unwrap();
    romref::configure(RomEnv { vc_roms: Some(dir.clone()), ..Default::default() });

    // 1. Verweis lesen: Satz „upper“ = erste 2048 Byte, „lower“ = zweite Hälfte
    let file = json!({ "format": 2, "kind": "charset", "profile": "c64.char.hires", "name": "t", "base": "rom:c64/chargen#lower", "overrides": { "1": ["11111111","00000000","11111111","00000000","11111111","00000000","11111111","00000000"] } });
    let m = vc_media::parse_media(&file.to_string()).unwrap();
    let frames = m.root["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 256);
    assert_eq!(frames[0]["rows"][3], json!(bits(rom[2048 + 3])));
    assert_eq!(frames[1]["rows"][0], json!("11111111"), "override gewinnt");
    assert_eq!(frames[2]["rows"][5], json!(bits(rom[2048 + 16 + 5])));
    assert_eq!(m.root["rom_status"], json!("unknown_revision"), "synthetische Datei ist nicht in der Tabelle");

    // 2. Speichern: nur die Änderung steht in der Datei, nie die ROM-Glyphen
    let mut edited = m.root.clone();
    edited["frames"][2]["rows"][0] = json!("10101010");
    let saved = romref::for_saving(&edited);
    assert_eq!(saved["format"], json!(2));
    assert!(saved.get("frames").is_none() && saved.get("rom_status").is_none());
    let o = saved["overrides"].as_object().unwrap();
    assert_eq!(o.len(), 2, "Zeichen 1 (alt) und 2 (neu): {o:?}");
    assert_eq!(o["2"][0], json!("10101010"));
    assert!(!saved.to_string().contains(&bits(rom[2048])), "keine Original-Zeile in der Datei");
    // Rundlauf
    let again = vc_media::parse_media(&saved.to_string()).unwrap();
    assert_eq!(again.root["frames"], edited["frames"]);

    // 3. Format 1 mit altem `base` (Kennung statt Verweis) bleibt unberührt
    let old = json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "alt", "base": "c64_standard_upper", "frames": [] });
    assert_eq!(romref::for_saving(&old), old);

    // 4. MEGA65-Vollfarbe aus dem C64-Satz: Bit → "01"
    let fcm = json!({ "format": 2, "kind": "charset", "profile": "mega65.char.fcm", "name": "m", "base": "rom:c64/chargen#upper" });
    let m = vc_media::parse_media(&fcm.to_string()).unwrap();
    let expect: String = bits(rom[0]).chars().map(|c| if c == '1' { "01" } else { "00" }).collect();
    assert_eq!(m.root["frames"][0]["rows"][0], json!(expect));

    // 5. Profil passt nicht zum Satz
    let bad = json!({ "format": 2, "kind": "charset", "profile": "atari.char.antic2", "name": "x", "base": "rom:c64/chargen#upper" });
    assert!(vc_media::parse_media(&bad.to_string()).unwrap_err().contains("passt nicht"));

    // 6. Quellenliste: gefundenes ROM mit seinen Sätzen
    let list = romref::with_store(|s| romref::choices(s, "c64.char.hires"));
    let refs: Vec<&str> = list.iter().map(|c| c.reference.as_str()).collect();
    assert!(refs.contains(&"rom:c64/chargen#upper") && refs.contains(&"rom:c64/chargen#lower"), "{refs:?}");
    assert!(list.iter().find(|c| c.reference == "rom:c64/chargen#upper").unwrap().status.usable());

    // 7. Revision, die nicht vorliegt: Fehler „fehlt“, kein stilles Ersetzen
    let r = RomRef::parse("rom:c64/chargen@906143-02#upper").unwrap();
    assert!(romref::with_store(|s| romref::read_set(s, &r)).is_err());

    // 8. ROM fehlt ganz: eingebaute Ersatzschrift (gekennzeichnet); Speichern schreibt trotzdem nur die eigenen Änderungen
    std::fs::remove_file(dir.join("c64/chargen.bin")).unwrap();
    romref::rescan();
    let fm = json!({ "format": 2, "kind": "charset", "profile": "c64.char.hires", "name": "t", "base": "rom:c64/chargen#upper", "overrides": { "65": ["11110000","11110000","11110000","11110000","00001111","00001111","00001111","00001111"] } });
    let m = vc_media::parse_media(&fm.to_string()).unwrap();
    assert_eq!(m.root["frames"].as_array().unwrap().len(), 256);
    assert_eq!(m.root["rom_status"], json!("replacement"));
    assert!(m.root["rom_note"].as_str().unwrap().contains("Ersatzschrift"));
    assert_ne!(m.root["frames"][1]["rows"], json!(vec!["00000000"; 8]), "A ist gezeichnet");
    assert_eq!(m.root["frames"][65]["rows"][0], json!("11110000"));
    let saved = romref::for_saving(&m.root);
    assert_eq!(saved["overrides"].as_object().unwrap().len(), 1, "nur die eigene Änderung");
    assert!(!saved.to_string().contains("rom_note"));
    // 9. ausdrücklich verlangte Revision fehlt: kein stiller Ersatz, leeres Raster mit Hinweis
    let pinned = json!({ "format": 2, "kind": "charset", "profile": "c64.char.hires", "name": "t", "base": "rom:c64/chargen@906143-02#upper" });
    let m = vc_media::parse_media(&pinned.to_string()).unwrap();
    assert!(m.root["rom_missing"].as_str().unwrap().contains("906143-02"));
    assert_eq!(m.root["frames"][1]["rows"][0], json!("00000000"));
    // 10. Quellenliste ohne ROM: Ersatzschrift ist wählbar
    let list = romref::with_store(|s| romref::choices(s, "c64.char.hires"));
    assert!(list.iter().any(|c| c.reference == "rom:c64/chargen#upper" && c.status == vc_media::Status::Replacement));
    let _ = std::fs::remove_dir_all(&dir);
    let _: Value = saved;
}
