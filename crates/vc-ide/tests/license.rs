// SPDX-License-Identifier: GPL-3.0-or-later
//! Lizenzhinweis beim Übersetzen: music_demo (Atari, RMT-Player unter GPL) löst L001 aus, prallball (MIT-0) nicht;
//! `licenses.deny` in der Projektdatei lässt den Bau scheitern, `allow` unterdrückt den Hinweis.
use std::path::{Path, PathBuf};
use vc_ide::build::{build, Ctx};
use vc_ide::projfile::ProjectFile;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn example(rel: &str) -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples").join(rel) }

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let n = e.file_name(); if n == "build" { continue; }
        if e.path().is_dir() { copy_dir(&e.path(), &to.join(&n)); } else { std::fs::copy(e.path(), to.join(&n)).unwrap(); }
    }
}

fn tmp(name: &str, ex: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-lic-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(ex), &dir);
    dir
}

fn codes(v: &serde_json::Value) -> Vec<String> { v["diagnostics"].as_array().unwrap().iter().map(|d| d["code"].as_str().unwrap_or("").to_string()).collect() }

#[test]
fn rmt_player_gives_a_copyleft_note_but_no_error() {
    let dir = tmp("md", "atari/music_demo");
    let v = build(&Ctx { root: &dir, data: &data() }, "music_demo.asm", None).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let d = v["diagnostics"].as_array().unwrap().iter().find(|d| d["code"] == "L001").expect("L001 fehlt");
    assert_eq!(d["severity"], "info");
    let m = d["message"].as_str().unwrap();
    assert!(m.contains("GPL-3.0-only") && m.contains("rmtplayr.asm") && m.contains("Radek"), "{m}");
    let lic = v["licenses"].as_array().unwrap();
    assert!(lic.iter().any(|l| l["file"].as_str().unwrap().ends_with("rmt.inc") && l["copyleft"] == true));
    // das Bauprotokoll nennt den Hinweis
    let log = v["log"].to_string();
    assert!(log.contains("Hinweis (L001)"), "{log}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn mit0_example_gives_no_note_and_lists_the_system_files() {
    let dir = tmp("pb", "c64/prallball");
    let v = build(&Ctx { root: &dir, data: &data() }, "prallball.asm", None).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(!codes(&v).contains(&"L001".to_string()), "{:?}", codes(&v));
    let lic = v["licenses"].as_array().unwrap();
    assert!(!lic.is_empty() && lic.iter().all(|l| l["license"] == "MIT-0" && l["copyleft"] == false), "{lic:?}");
    assert!(lic.iter().any(|l| l["file"].as_str().unwrap().starts_with("data/sys/")), "Systemdateien stehen im Bericht");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn deny_in_project_file_fails_the_build_and_allow_silences_the_note() {
    let dir = tmp("deny", "atari/music_demo");
    let file = vc_ide::projfile::find(&dir).unwrap();
    let mut pf = ProjectFile::load(&file).unwrap();
    pf.licenses.deny = vec!["GPL-3.0-only".into(), "GPL-3.0-or-later".into()];
    pf.save(&file).unwrap();
    let v = build(&Ctx { root: &dir, data: &data() }, "music_demo.asm", None).unwrap();
    assert_eq!(v["ok"], false, "{v}");
    assert!(codes(&v).contains(&"L002".to_string()));
    assert!(!dir.join("build/music_demo.xex").exists(), "bei gesperrter Lizenz entsteht kein Ergebnis");
    // allow unterdrückt nur den Hinweis (und hebt die Sperre nicht auf, wenn die Lizenz nicht gesperrt ist)
    pf.licenses.deny.clear();
    pf.licenses.allow = vec!["GPL-3.0-only".into()];
    pf.save(&file).unwrap();
    let v = build(&Ctx { root: &dir, data: &data() }, "music_demo.asm", None).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(!codes(&v).contains(&"L001".to_string()));
    assert!(v["licenses"].as_array().unwrap().iter().any(|l| l["license"] == "GPL-3.0-only"), "der Bericht führt sie weiter auf");
    std::fs::remove_dir_all(&dir).ok();
}
