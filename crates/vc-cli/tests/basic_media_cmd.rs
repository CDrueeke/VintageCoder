// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc basic` mit Medien: `#include "graphics/hero.vcsprite"` ist bytegleich zum Altpfad `build/export/hero.data.bas`, Fehler und Altpfad-Hinweis,
//! Umwandlung schreibt Medien nie zurück.
use std::path::{Path, PathBuf};
use std::process::Command;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn vcc() -> Command { let mut c = Command::new(env!("CARGO_BIN_EXE_vcc")); c.env("VC_LANG", "de").env("VC_DATA", data()); c }
fn proj(tag: &str) -> PathBuf {
    let r = std::env::temp_dir().join(format!("vc-cli-basic-media-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r); std::fs::create_dir_all(&r).unwrap(); r
}
fn sprite(root: &Path) -> PathBuf {
    let v = vc_media::project::create(root, &data(), "hero", "c64.sprite.hires", "").unwrap();
    PathBuf::from(v["path"].as_str().unwrap())
}
const BODY: &str = "for i=0 to 63:read a:poke 832+i,a:next i\n";

fn build(root: &Path, file: &str, target: &str) -> (bool, String, Vec<u8>) {
    let o = vcc().current_dir(root).args(["basic", file, "--target", target, "-o", "out.prg"]).output().unwrap();
    (o.status.success(), String::from_utf8_lossy(&o.stderr).into_owned() + &String::from_utf8_lossy(&o.stdout), std::fs::read(root.join("out.prg")).unwrap_or_default())
}

#[test]
fn medium_include_is_byte_identical_to_the_legacy_path() {
    let root = proj("bytes");
    let rel = sprite(&root);
    vc_media::project::export(&root, &data(), rel.to_str().unwrap(), "data", None, None, 1000, 10).unwrap();
    assert!(root.join("build/export/hero.data.bas").is_file());
    std::fs::write(root.join("neu.bas"), format!("#include \"graphics/hero.vcsprite\"\n{BODY}")).unwrap();
    std::fs::write(root.join("alt.bas"), format!("#include \"build/export/hero.data.bas\"\n{BODY}")).unwrap();
    let (ok1, out1, prg1) = build(&root, "neu.bas", "c64");
    assert!(ok1, "{out1}");
    assert!(!out1.contains("B194"), "{out1}");
    let (ok2, out2, prg2) = build(&root, "alt.bas", "c64");
    assert!(ok2, "{out2}");
    assert!(out2.contains("B194") && out2.contains("graphics/hero.vcsprite"), "Altpfad-Hinweis mit Quelle: {out2}");
    assert!(prg1.len() > 200, "DATA-Zeilen im Programm");
    assert_eq!(prg1, prg2, "Medium und Altpfad liefern dasselbe Programm");
    assert!(root.join("build/.media-cache").is_dir(), "Zwischenspeicher");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn media_errors_are_reported_with_include_line() {
    let root = proj("err");
    sprite(&root);
    std::fs::write(root.join("a.bas"), format!("{BODY}#include \"graphics/fehlt.vcsprite\"\n")).unwrap();
    let (ok, out, _) = build(&root, "a.bas", "c64");
    assert!(!ok && out.contains("B193") && out.contains("fehlt.vcsprite"), "{out}");
    // Profil für den C64, Ziel VC-20: Fehler statt stillem Umwandeln
    std::fs::write(root.join("b.bas"), format!("#include \"graphics/hero.vcsprite\"\n{BODY}")).unwrap();
    let (ok, out, _) = build(&root, "b.bas", "vic20");
    assert!(!ok && out.contains("B193"), "{out}");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn conversion_never_writes_the_medium_back() {
    let root = proj("conv");
    let rel = sprite(&root);
    let before = std::fs::read(root.join(&rel)).unwrap();
    std::fs::write(root.join("m.bas"), "10 restore 20\n20 read a\n#include \"graphics/hero.vcsprite\"\n").unwrap();
    let o = vcc().current_dir(&root).args(["basic", "m.bas", "--to-labels", "--write", "--target", "c128"]).output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr).into_owned() + &String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success(), "{err}");
    assert_eq!(std::fs::read(root.join(&rel)).unwrap(), before, "Mediendatei unverändert");
    assert!(!err.contains("hero.vcsprite: "), "{err}");
    let m = std::fs::read_to_string(root.join("m.bas")).unwrap();
    assert!(m.contains("#include \"graphics/hero.vcsprite\""), "{m}");
    std::fs::remove_dir_all(&root).ok();
}
