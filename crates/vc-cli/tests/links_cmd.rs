// SPDX-License-Identifier: GPL-3.0-or-later
//! Anbindungen und Release-Auswahl: mit einer `release.json`, die nur VICE wählt, kennt `vcc` nichts anderes; ohne Auswahl bleibt alles.
use std::path::PathBuf;
use std::process::{Command, Output};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn text(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr) }

/// `release`: Inhalt einer `release.json` oder `None` (keine Auswahl: `VC_RELEASE=all`)
fn vcc(tag: &str, release: Option<&str>, args: &[&str]) -> Output {
    let dir = std::env::temp_dir().join(format!("vc-cli-links-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sel = match release { Some(t) => { let p = dir.join("release.json"); std::fs::write(&p, t).unwrap(); p.to_string_lossy().into_owned() } None => "all".into() };
    // eigene Konfiguration und Datenordner: keine installierten Emulatoren, keine Einstellungen des Benutzers
    Command::new(env!("CARGO_BIN_EXE_vcc")).args(args).env("VC_DATA", data()).env("VC_RELEASE", sel).env("VC_SETTINGS", dir.join("settings.json")).env("HOME", &dir).env("XDG_CONFIG_HOME", dir.join("cfg")).env("XDG_DATA_HOME", dir.join("data")).current_dir(&dir).output().unwrap()
}

const NUR_VICE: &str = r#"{"format":1,"ide":false,"links":["vice"]}"#;

#[test]
fn emulator_list_zeigt_nur_gewaehlte() {
    let o = vcc("list", Some(NUR_VICE), &["emulator", "list"]);
    let t = text(&o);
    assert!(o.status.success() && t.contains("VICE"), "{t}");
    for name in ["atari800", "Altirra", "Xemu", "Fujisan", "Ultimate", "MEGA65"] { assert!(!t.contains(name), "{name} dürfte nicht erscheinen:\n{t}"); }
    let o = vcc("status", Some(NUR_VICE), &["emulator", "status"]);
    let t = text(&o);
    assert!(t.contains("VICE") && !t.contains("atari800") && !t.contains("Xemu"), "{t}");
}

#[test]
fn abgewaehlte_anbindung_meldet_nicht_im_paket() {
    for args in [&["emulator", "status", "atari800"][..], &["emulator", "install", "xemu", "--dry-run"], &["emulator", "install", "altirra"]] {
        let o = vcc("fehler", Some(NUR_VICE), args);
        assert_eq!(o.status.code(), Some(1), "{args:?}: {}", text(&o));
        assert!(text(&o).contains("nicht im Paket enthalten"), "{args:?}: {}", text(&o));
    }
    let o = vcc("backend", Some(NUR_VICE), &["test", ".", "--backend", "atari800"]);
    assert!(!o.status.success() && text(&o).contains("nicht im Paket enthalten"), "{}", text(&o));
    let o = vcc("u64", Some(NUR_VICE), &["test", ".", "--backend", "u64:192.168.1.50"]);
    assert!(!o.status.success() && text(&o).contains("nicht im Paket enthalten"), "{}", text(&o));
    let o = vcc("streams", Some(NUR_VICE), &["streams", "listen"]);
    assert!(!o.status.success() && text(&o).contains("nicht im Paket enthalten"), "{}", text(&o));
    // das Gewählte geht weiter (Trockenlauf, ohne Download)
    let o = vcc("vice", Some(NUR_VICE), &["emulator", "install", "vice", "--dry-run"]);
    assert!(!text(&o).contains("nicht im Paket enthalten"), "{}", text(&o));
}

#[test]
fn ohne_auswahl_bleibt_alles_wie_heute() {
    let o = vcc("alle", None, &["emulator", "list"]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    for name in ["VICE", "atari800", "Altirra", "Xemu", "Fujisan", "Ultimate", "MEGA65"] { assert!(t.contains(name), "{name} fehlt:\n{t}"); }
    let o = vcc("alle3", None, &["emulator", "status", "ultimate"]);
    assert!(o.status.success() && text(&o).contains("Ultimate"), "{}", text(&o));
    let o = vcc("nurultimate", Some(r#"{"format":1,"links":["ultimate"]}"#), &["emulator", "list"]);
    assert!(text(&o).contains("Ultimate") && !text(&o).contains("VICE") && !text(&o).contains("MEGA65"), "{}", text(&o));
    let o = vcc("alle2", None, &["emulator", "status", "atari800"]);
    assert!(o.status.success(), "{}", text(&o));
}
