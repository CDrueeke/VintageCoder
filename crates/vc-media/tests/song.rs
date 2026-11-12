// SPDX-License-Identifier: GPL-3.0-or-later
use vc_media::{song, *};
use std::path::PathBuf;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn notes_roundtrip() {
    assert_eq!(song::parse_cell("C-0 00").unwrap(), (1, 0));
    assert_eq!(song::parse_cell("A-4 0F").unwrap(), (58, 15));
    assert_eq!(song::parse_cell("C#3 --").unwrap(), (38, song::INS_KEEP));
    assert_eq!(song::parse_cell("=== --").unwrap(), (song::NOTE_OFF, song::INS_KEEP));
    assert_eq!(song::parse_cell("--- --").unwrap(), (0, song::INS_KEEP));
    for n in 1..=96u8 { assert_eq!(song::parse_cell(&song::format_cell(n, 3)).unwrap(), (n, 3)); }
    assert!(song::parse_cell("H-4 00").is_err() && song::parse_cell("C-9 00").is_err() && song::parse_cell("C-4 zz").is_err());
}

#[test]
fn encodes_sid_and_pokey_songs() {
    let profiles = load_profiles(&data()).unwrap();
    for (id, ch, rec) in [("c64.sid.song", 3usize, 8usize), ("atari.pokey.song", 4, 32)] {
        let p = profile(&profiles, id).unwrap(); let mut a = new_media(p, "t");
        a["patterns"][0]["rows"][0][0] = "C-4 01".into(); a["patterns"][0]["rows"][1][0] = "=== --".into();
        let parts = song::encode(&a, p).unwrap();
        assert_eq!(parts[0], ("order".to_string(), vec![0, 0xFF, 0]));
        assert_eq!(parts[1].1.len(), 32 * ch * 2);
        assert_eq!(&parts[1].1[..2], &[49, 1]);
        assert_eq!(&parts[1].1[ch * 2..ch * 2 + 2], &[song::NOTE_OFF, song::INS_KEEP]);
        assert_eq!(parts[2].1.len(), 2 * rec);
        let bad = { let mut b = a.clone(); b["patterns"][0]["rows"][0][0] = "C-4 09".into(); b };
        assert!(song::encode(&bad, p).unwrap_err().contains("Instrument 09"));
    }
}

#[test]
fn sid_instrument_bytes() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "c64.sid.song").unwrap();
    let mut a = new_media(p, "t"); a["instruments"][0] = serde_json::json!({ "name": "x", "wave": "saw", "pw": 0x0A12, "a": 1, "d": 2, "s": 3, "r": 4 });
    let ins = &song::encode(&a, p).unwrap()[2].1;
    assert_eq!(&ins[..5], &[0x20, 0x12, 0x34, 0x12, 0x0A]);
}

#[test]
fn pokey_song_becomes_an_rmt_module() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "atari.pokey.song").unwrap();
    let mut a = new_media(p, "t");
    a["patterns"][0]["rows"][0][0] = "C-3 00".into();   // RMT-Note 0
    a["patterns"][0]["rows"][4][0] = "=== --".into();
    a["patterns"][0]["rows"][8][1] = "C-4 01".into();
    let m = song::rmt_build(&a, p).unwrap();
    assert_eq!(&m.bytes[..4], b"RMT4");
    assert_eq!((m.bytes[4], m.bytes[5], m.bytes[6]), (32, 6, 1));          // Tracklänge, Takt, Instrumentgeschwindigkeit
    let f = m.to_file(0x4000);
    assert_eq!(&f[..4], &[0xFF, 0xFF, 0x00, 0x40]);
    let end = f[4] as usize | (f[5] as usize) << 8; assert_eq!(end, 0x4000 + m.bytes.len() - 1);
    let w = |o: usize| f[6 + o] as usize | (f[7 + o] as usize) << 8;
    let (tis, tlo, thi, sng) = (w(8), w(10), w(12), w(14));
    assert!(tis > 0x4000 && tlo > tis && thi > tlo && sng > thi, "Zeiger aufsteigend: {tis:x} {tlo:x} {thi:x} {sng:x}");
    // Track der Stimme 0: Note C-3 mit Lautstärke 15 (Instrument 0), 3 Leerzeilen, Note aus, 27 Leerzeilen
    let t0 = (f[6 + (tlo - 0x4000)] as usize) | (f[6 + (thi - 0x4000)] as usize) << 8;
    let tr = &f[6 + (t0 - 0x4000)..];
    assert_eq!(&tr[..7], &[0 | 3 << 6, 0 << 2 | 3, 62 | 3 << 6, 61, 0, 62, 27][..], "{:02x?}", &tr[..8]);
    // Songzeile 0 und Sprungzeile am Ende (Schleife auf Zeile 0)
    let goto = 6 + (sng - 0x4000) + 4;
    assert_eq!(&f[goto..goto + 4], &[0xFE, 0, sng as u8, (sng >> 8) as u8]);
    let asm = m.to_asm("tune_rmt"); assert!(asm.contains("tune_rmt:") && asm.contains(".word tune_rmt+16") && asm.contains("<(tune_rmt+"));
    a["patterns"][0]["rows"][1][0] = "C-2 00".into();
    assert!(song::rmt_build(&a, p).unwrap_err().contains("POKEY-Bereich"));
}

#[test]
fn rmt_export_import_roundtrip() {
    let profiles = load_profiles(&data()).unwrap(); let p = profile(&profiles, "atari.pokey.song").unwrap();
    let mut a = new_media(p, "t");
    a["patterns"][0]["rows"][0][0] = "C-4 00".into(); a["patterns"][0]["rows"][2][0] = "=== --".into(); a["patterns"][0]["rows"][5][3] = "G-5 01".into();
    a["patterns"].as_array_mut().unwrap().push(serde_json::json!({ "rows": (0..32).map(|_| vec!["--- --"; 4]).collect::<Vec<_>>() }));
    a["patterns"][1]["rows"][7][1] = "D#3 01".into();
    a["order"] = serde_json::json!([0, 1, 0]); a["loop"] = serde_json::json!(1); a["speed"] = serde_json::json!(4);
    let file = song::rmt_build(&a, p).unwrap().to_file(0x4000);
    let (b, warn) = song::rmt_import(&file, p, "t").unwrap();
    assert!(warn.is_empty(), "{warn:?}");
    for k in ["speed", "rows", "order", "loop", "patterns"] { assert_eq!(a[k], b[k], "{k}"); }
    for (x, y) in a["instruments"].as_array().unwrap().iter().zip(b["instruments"].as_array().unwrap()) { assert_eq!((&x["env"], &x["dist"], &x["loop"]), (&y["env"], &y["dist"], &y["loop"])); }
    assert!(song::rmt_import(b"RMT4", p, "t").is_err() && song::rmt_import(&[0xFF, 0xFF, 0, 0x40, 3, 0x40, b'R', b'M', b'T', b'8'], p, "t").unwrap_err().contains("RMT8"));
}

#[test]
fn converts_between_sid_and_pokey() {
    let profiles = load_profiles(&data()).unwrap(); let sid = profile(&profiles, "c64.sid.song").unwrap(); let pok = profile(&profiles, "atari.pokey.song").unwrap();
    let mut a = new_media(sid, "t");
    a["patterns"][0]["rows"][0][0] = "C-4 00".into(); a["patterns"][0]["rows"][0][1] = "C-2 01".into(); a["patterns"][0]["rows"][4][2] = "=== --".into();
    let (b, warn) = song::convert(&a, sid, pok).unwrap();
    assert_eq!(b["profile"], "atari.pokey.song");
    assert_eq!(b["patterns"][0]["rows"][0], serde_json::json!(["C-4 00", "C-3 01", "--- --", "--- --"]), "Bass wird oktaviert");
    assert_eq!(b["patterns"][0]["rows"][4][2], "=== --");
    assert!(warn.iter().any(|w| w.contains("unter C-3")), "{warn:?}");
    assert_eq!(b["instruments"][0]["env"].as_array().unwrap().len(), 16);
    assert!(song::rmt_build(&b, pok).is_ok(), "Ergebnis lässt sich als RMT-Modul exportieren");
    let (c, warn2) = song::convert(&b, pok, sid).unwrap();      // zurück: vierte Stimme (leer) entfällt
    assert_eq!(c["patterns"][0]["rows"][0].as_array().unwrap().len(), 3);
    assert!(warn2.iter().any(|w| w.contains("Stimme 4")), "{warn2:?}");
    assert!(song::encode(&c, sid).is_ok());
    assert!(song::convert(&a, sid, sid).is_err());
}

/// Zeichensätze aus ROM-Verweisen: mit synthetischen ROM-Dateien (richtige Größe, Muster) für alle Systeme – keine echten ROMs nötig
#[test]
fn standard_charsets_come_from_rom_references() {
    let profiles = load_profiles(&data()).unwrap();
    let rom_dir = std::env::temp_dir().join(format!("vc_std_roms_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&rom_dir);
    for (rel, size) in [("c64/chargen.bin", 4096), ("c128/chargen.bin", 8192), ("vic20/chargen.bin", 4096), ("plus4/kernal.bin", 16384), ("atari8/os-xl.rom", 16384), ("atari8/os-b.rom", 10240)] {
        let p = rom_dir.join(rel); std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, (0..size).map(|i| (i % 253 + 1) as u8).collect::<Vec<u8>>()).unwrap();
    }
    vc_media::romref::configure(vc_roms::RomEnv { vc_roms: Some(rom_dir.clone()), ..Default::default() });
    let list = project::std_charset_list(&data());
    assert!(list.len() >= 8, "{} Zeichensatzquellen", list.len());
    let dir = std::env::temp_dir().join(format!("vc_std_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
    let mut done = std::collections::HashSet::new();
    for c in list.iter().filter(|c| c.status != vc_media::Status::Missing) {
        assert!(!c.label.is_empty() && profiles.iter().any(|p| p.id == c.profile && p.target == c.system), "{}: Profil {} passt nicht zu {}", c.reference, c.profile, c.system);
        let p = profile(&profiles, &c.profile).unwrap();
        if !done.insert((c.reference.clone(), c.profile.clone())) { continue; }
        // Verweis (Format 2) und Kopie (Format 1) in getrennten Ordnern, damit gleiche Namen nicht kollidieren
        for (copy, sub) in [(false, "ref"), (true, "copy")] {
            let d = dir.join(sub); std::fs::create_dir_all(&d).unwrap();
            let name = format!("{}-{}", c.profile, c.reference.replace(['/', ':', '#', '@'], "_"));
            let r = project::add_std_charset(&d, &c.reference, &c.profile, "graphics", copy);
            let r = match r { Ok(r) => r, Err(e) if e.contains("existiert bereits") => continue, Err(e) => panic!("{name}: {e}") };
            let text = std::fs::read_to_string(d.join(r["path"].as_str().unwrap())).unwrap();
            if !copy { assert!(text.contains("\"base\"") && !text.contains("\"frames\""), "{name}: Verweisdatei enthält Glyphen:\n{text}"); }
            let a = parse_media(&text).unwrap();
            assert_eq!((a.kind.as_str(), a.profile.as_str()), ("charset", c.profile.as_str()));
            let frames = a.root["frames"].as_array().unwrap();
            assert_eq!(frames.len(), p.count, "{name}: Zeichenzahl");
            assert!(frames.iter().all(|f| f["rows"].as_array().map_or(false, |r| r.len() == p.height)), "{name}: Zeilenzahl je Zeichen");
            assert!(frames.iter().any(|f| f["rows"].as_array().unwrap().iter().any(|r| r.as_str().unwrap().contains('1'))), "{name}: leerer Satz");
        }
    }
    // vorhandene Datei bleibt, unbekannter Verweis ist ein Fehler
    let r0 = &list.iter().find(|c| c.status != vc_media::Status::Missing).unwrap();
    let again = project::add_std_charset(&dir.join("ref"), &r0.reference, &r0.profile, "graphics", false);
    assert!(again.is_err(), "vorhandene Datei bleibt");
    assert!(project::add_std_charset(&dir, "gibt_es_nicht", "c64.char.hires", "", false).is_err());
    std::fs::remove_dir_all(&dir).ok(); std::fs::remove_dir_all(&rom_dir).ok();
}
