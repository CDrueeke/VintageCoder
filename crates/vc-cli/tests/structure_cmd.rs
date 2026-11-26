// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc new --structure`, `vcc add` und `vcc organize` Ende zu Ende an einem frischen Projekt.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
/// Persönlicher Ordner für den Prozess: Tests schreiben nie in den echten Konfigurationsordner (z. B. Liste der Umordnungs-Sicherungen)
fn vcc(dir: &Path, args: &[&str]) -> Output {
    let home = dir.join("home");
    Command::new(env!("CARGO_BIN_EXE_vcc")).args(args).env("VC_LANG", "de").env("VC_DATA", data())
        .env("HOME", &home).env("USERPROFILE", &home).env("APPDATA", &home).env("XDG_CONFIG_HOME", home.join("xdg")).current_dir(dir).output().unwrap()
}
fn text(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr) }

#[test]
fn struktur_add_und_organize() {
    let base = std::env::temp_dir().join(format!("vc-cli-struct-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (proj, ext) = (base.join("p"), base.join("ext"));
    std::fs::create_dir_all(ext.join("pack/sub")).unwrap();
    std::fs::write(ext.join("font.bin"), [0u8, 1, 2]).unwrap();
    std::fs::write(ext.join("pack/n.txt"), "hi").unwrap();
    std::fs::write(ext.join("pack/sub/b.bin"), [0u8, 2]).unwrap();
    let o = vcc(&base, &["new", proj.to_str().unwrap(), "--target", "c64", "--structure", "gross"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(proj.join("res/fonts").is_dir());
    let pf = std::fs::read_to_string(proj.join("p.vcprj")).unwrap();
    assert!(pf.contains("\"layout\"") && pf.contains("res/bin") && pf.contains("\"folders\""), "{pf}");
    // unbekannte Struktur
    assert!(!vcc(&base, &["new", base.join("q").to_str().unwrap(), "--structure", "gibtsnicht"]).status.success());
    // Trockenlauf ändert nichts
    let o = vcc(&base, &["add", ext.join("font.bin").to_str().unwrap(), ext.join("pack").to_str().unwrap(), "--into", proj.to_str().unwrap(), "--dry-run"]);
    assert!(o.status.success() && !proj.join("res/bin/font.bin").exists(), "{}", text(&o));
    // Übernehmen: Datei nach layout, Ordner mit Struktur, Einträge in assets
    let o = vcc(&base, &["add", ext.join("font.bin").to_str().unwrap(), ext.join("pack").to_str().unwrap(), "--into", proj.to_str().unwrap(), "--description", "Test"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(proj.join("res/bin/font.bin").is_file() && proj.join("pack/sub/b.bin").is_file() && proj.join("pack/n.txt").is_file(), "{}", text(&o));
    let pf = std::fs::read_to_string(proj.join("p.vcprj")).unwrap();
    assert!(pf.contains("pack/sub/b.bin") && pf.contains("\"description\": \"Test\""), "{pf}");
    // zweiter Lauf: gleiche Dateien sind „vorhanden“, nichts doppelt
    let o = vcc(&base, &["add", ext.join("font.bin").to_str().unwrap(), "--into", proj.to_str().unwrap()]);
    assert!(text(&o).contains("vorhanden"), "{}", text(&o));
    assert_eq!(std::fs::read_to_string(proj.join("p.vcprj")).unwrap().matches("res/bin/font.bin").count(), 1);
    // organize: nichts im Projektordner, Unterordner mit --subdirs
    let o = vcc(&base, &["organize", proj.to_str().unwrap(), "--dry-run", "--subdirs"]);
    let t0 = text(&o);
    assert!(o.status.success(), "{t0}");
    // ohne Terminal und ohne --yes wird nichts geändert
    let o = vcc(&base, &["organize", proj.to_str().unwrap(), "--subdirs"]);
    assert!(!o.status.success() && text(&o).contains("--yes") && proj.join("pack/sub/b.bin").is_file(), "{}", text(&o));
    let o = vcc(&base, &["organize", proj.to_str().unwrap(), "--dry-run", "--subdirs"]);
    let t = text(&o);
    assert!(t.contains("pack/sub/b.bin -> res/bin/b.bin"), "{t}");
    let _ = std::fs::remove_dir_all(&base);
}

/// Die Strukturen heißen englisch (`large`); die früheren deutschen Namen (`gross`) bleiben als Alias gültig
#[test]
fn struktur_englische_kennungen_und_deutscher_alias() {
    let base = std::env::temp_dir().join(format!("vc-cli-struct-alias-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    for (name, id) in [("a", "large"), ("b", "gross"), ("c", "multisystem"), ("d", "mehrsystem")] {
        let p = base.join(name);
        let o = vcc(&base, &["new", p.to_str().unwrap(), "--target", "c64", "--structure", id]);
        assert!(o.status.success(), "{id}: {}", text(&o));
    }
    // beide Schreibweisen ergeben dieselben Ordner
    assert_eq!(base.join("a").join("src").is_dir(), base.join("b").join("src").is_dir());
    assert_eq!(base.join("c").join("res").is_dir(), base.join("d").join("res").is_dir());
    let l = text(&vcc(&base, &["new", "--list"]));
    assert!(l.contains("simple") && l.contains("large") && l.contains("multisystem") && !l.contains("mehrsystem"), "{l}");
    std::fs::remove_dir_all(&base).ok();
}
