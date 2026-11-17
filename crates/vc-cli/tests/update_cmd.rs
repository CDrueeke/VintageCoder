// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc update` ohne echtes Netz: die Versionsliste kommt aus einem lokalen Ordner (`VC_UPDATE_URL`), Befund und Protokoll liegen in einem Wegwerf-Ordner.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vcc-update-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn vcc(dir: &Path, feed: &str, release: Option<&str>, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vcc"));
    c.args(args).env("VC_DATA", data()).env("VC_UPDATE_URL", feed).env("VC_UPDATE_DIR", dir.join("updates")).env("VC_SETTINGS", dir.join("settings.json")).env("HOME", dir).env("XDG_CONFIG_HOME", dir.join("cfg")).current_dir(dir);
    if let Some(r) = release { c.env("VC_RELEASE", r); }
    c.output().unwrap()
}
fn feed(dir: &Path, latest: &str, yanked: &str) -> String {
    let f = dir.join("feed"); std::fs::create_dir_all(&f).unwrap();
    let v = env!("CARGO_PKG_VERSION");
    let alt = if latest == v { String::new() } else { format!(r#",{{"version":"{v}","title":"{v}","notes":"alt"}}"#) };
    std::fs::write(f.join("update.json"), format!(r#"{{"format":1,"sequence":1,"channel":"stable","published":"2030-01-01T00:00:00Z","latest":"{latest}","yanked":[{yanked}],"releases":[
        {{"version":"{latest}","title":"{latest} (Test)","notes":"Neu: [Seite](https://x.example/) und ![Bild](https://x.example/b.png)"}}{alt}]}}"#)).unwrap();
    f.to_str().unwrap().to_string()
}
fn out(o: &Output) -> String { format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)) }

#[test]
fn update_verfuegbar_endet_mit_10_und_zeigt_festen_link() {
    let d = tmp("neu"); let f = feed(&d, "99.0.0", "");
    let o = vcc(&d, &f, None, &["update", "--notes"]);
    let t = out(&o);
    assert_eq!(o.status.code(), Some(10), "{t}");
    assert!(t.contains("Neu: Version 99.0.0") && t.contains("https://github.com/CDrueeke/VintageCoder/releases/tag/v99.0.0"), "{t}");
    assert!(t.contains("Neu: Seite (https://x.example/) und Bild") && !t.contains("!["), "Links und Bilder nur als Text: {t}");
    assert!(d.join("updates/log.txt").is_file() && d.join("updates/state.json").is_file());
}

#[test]
fn aktuell_endet_mit_0_und_ohne_option_wird_geprueft() {
    let d = tmp("aktuell"); let f = feed(&d, env!("CARGO_PKG_VERSION"), "");
    let o = vcc(&d, &f, None, &["update"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(out(&o).contains("Aktuell"));
}

#[test]
fn json_und_offline() {
    let d = tmp("json"); let f = feed(&d, "99.0.0", "");
    let o = vcc(&d, &f, None, &["update", "--json"]);
    assert_eq!(o.status.code(), Some(10));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!((v["update_available"].as_bool(), v["version"].as_str(), v["url"].as_str()), (Some(true), Some("99.0.0"), Some("https://github.com/CDrueeke/VintageCoder/releases/tag/v99.0.0")));
    // ohne Netz: Quelle zeigt ins Leere, --offline liest nur den gespeicherten Befund
    let o = vcc(&d, "/gibt/es/nicht", None, &["update", "--offline"]);
    assert_eq!(o.status.code(), Some(10), "{}", out(&o));
    assert!(out(&o).contains("99.0.0"));
    // frischer Ordner ohne Befund
    let d2 = tmp("leer"); let o = vcc(&d2, "/gibt/es/nicht", None, &["update", "--offline"]);
    assert_eq!(o.status.code(), Some(0)); assert!(out(&o).contains("Noch keine Prüfung"));
}

#[test]
fn fehler_zurueckgezogen_und_aufruf() {
    let d = tmp("fehler");
    let o = vcc(&d, "/gibt/es/nicht", None, &["update"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let f = feed(&d, "99.0.0", &format!("\"{}\"", env!("CARGO_PKG_VERSION")));
    let o = vcc(&d, &f, None, &["update"]);
    assert!(out(&o).contains("zurückgezogen"), "{}", out(&o));
    let o = vcc(&d, &f, None, &["update", "--installieren"]);
    assert_eq!(o.status.code(), Some(2), "unbekannte Option: {}", out(&o));
}

#[test]
fn ohne_ide_update_im_release_gibt_es_keine_pruefung() {
    let d = tmp("ohne"); let f = feed(&d, "99.0.0", "");
    let rel = d.join("release.json"); std::fs::write(&rel, r#"{"format":1,"update":false}"#).unwrap();
    let o = vcc(&d, &f, Some(rel.to_str().unwrap()), &["update"]);
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(out(&o).contains("keine Update-Prüfung"));
    assert!(!d.join("updates").exists(), "nichts geschrieben, nichts abgefragt");
}

#[test]
fn andere_befehle_weisen_nie_auf_updates_hin() {
    let d = tmp("still"); let f = feed(&d, "99.0.0", "");
    let o = vcc(&d, &f, None, &["--version"]);
    assert_eq!(o.status.code(), Some(0)); assert!(!out(&o).contains("Update") && !out(&o).contains("99.0.0"));
    assert!(!d.join("updates").exists());
}
