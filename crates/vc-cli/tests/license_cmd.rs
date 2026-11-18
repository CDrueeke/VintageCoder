// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc build --license-report` und `--deny-license`: Bericht (Text, JSON, build/LICENSES.txt) und Sperre.
use std::path::{Path, PathBuf};
use std::process::Command;

fn vcc() -> Command { let mut c = Command::new(env!("CARGO_BIN_EXE_vcc")); c.env("VC_DATA", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")); c }
fn example(rel: &str) -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples").join(rel) }
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let n = e.file_name(); if n == "build" { continue; }
        if e.path().is_dir() { copy_dir(&e.path(), &to.join(&n)); } else { std::fs::copy(e.path(), to.join(&n)).unwrap(); }
    }
}
fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-liccli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    copy_dir(&example("atari/music_demo"), &d);
    d
}

#[test]
fn build_prints_note_and_report_writes_licenses_file() {
    let d = tmp("rep");
    let o = vcc().args(["build", d.to_str().unwrap(), "--license-report"]).output().unwrap();
    let (out, err) = (String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    assert!(o.status.success(), "{out}{err}");
    assert!(err.contains("Hinweis (L001)") && err.contains("GPL-3.0-only"), "{err}");
    assert!(out.contains("rmtplayr.asm") && out.contains("Copyleft"), "{out}");
    let f = std::fs::read_to_string(d.join("build/LICENSES.txt")).unwrap();
    assert!(f.contains("GPL-3.0-only") && f.contains("Radek"), "{f}");
    let o = vcc().args(["build", d.to_str().unwrap(), "--license-report", "--json"]).output().unwrap();
    let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(j["programs"][0]["files"].as_array().unwrap().iter().any(|f| f["license"] == "GPL-3.0-only"));
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn deny_license_option_fails_the_build() {
    let d = tmp("deny");
    let o = vcc().args(["build", d.to_str().unwrap(), "--deny-license", "GPL-3.0-only,GPL-3.0-or-later"]).output().unwrap();
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("L002"), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!d.join("build/music_demo.xex").exists());
    // ohne die Sperre geht es
    assert!(vcc().args(["build", d.to_str().unwrap()]).output().unwrap().status.success());
    std::fs::remove_dir_all(&d).ok();
}
