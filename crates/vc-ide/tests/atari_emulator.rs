// SPDX-License-Identifier: GPL-3.0-or-later
//! Atari-Programme starten in Altirra, wenn vorhanden (Standard), sonst in atari800; die Einstellung kann atari800 erzwingen.
use vc_ide::launch::altirra_choice;
use vc_ide::settings::{set, Settings};

#[test]
fn altirra_is_the_default_for_atari_unless_atari800_is_chosen() {
    let dir = std::env::temp_dir().join(format!("vc_atari_emu_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("VC_SETTINGS", dir.join("settings.json"));
    let fake = dir.join("Altirra64.exe");
    std::fs::write(&fake, "").unwrap();

    // Pfad eingetragen, keine Wahl: Altirra
    let mut s = Settings::default();
    s.emulators.altirra.path = fake.to_string_lossy().into_owned();
    set(s.clone()).unwrap();
    assert_eq!(altirra_choice().as_deref(), fake.to_str());

    // ausdrücklich Altirra: ebenso
    s.emulators.altirra.default_for_atari = "altirra".into();
    set(s.clone()).unwrap();
    assert_eq!(altirra_choice().as_deref(), fake.to_str());

    // atari800 erzwungen: kein Altirra
    s.emulators.altirra.default_for_atari = "atari800".into();
    set(s.clone()).unwrap();
    assert!(altirra_choice().is_none());

    // Der Eintrag in der Datei überlebt Speichern und Laden (Einstellung auch in `settings.json`)
    let text = std::fs::read_to_string(dir.join("settings.json")).unwrap();
    assert!(text.contains("default_for_atari") && text.contains("atari800"), "{text}");
    eprintln!("Altirra automatisch gefunden: {:?}", vc_ide::launch::detect("altirra"));
    std::fs::remove_dir_all(&dir).ok();
}
