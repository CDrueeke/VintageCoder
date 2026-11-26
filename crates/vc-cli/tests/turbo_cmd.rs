// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc turbo …`, `vcc build` für Turbo-Projekte und `vcc disk atr --base` (Turbo-BASIC-Konzept, E0/E1) mit selbst gebauten Abbildern.
//! Meldungen werden auf Englisch geprüft (`--lang en`): so bleibt der Test von der Systemsprache unabhängig und prüft zugleich den englischen Katalog.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use vc_disk::Atr;

fn vcc(args: &[&str], dir: &Path) -> Output {
    let mut a = vec!["--lang", "en"]; a.extend_from_slice(args);
    Command::new(env!("CARGO_BIN_EXE_vcc")).args(a).current_dir(dir).env("VC_DATA", Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")).env("VC_LANG", "en").stdin(std::process::Stdio::null()).output().unwrap()
}
fn out(o: &Output) -> String { format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)) }
fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vcc-turbo-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
fn bytes(n: usize, seed: u8) -> Vec<u8> { (0..n).map(|i| (i as u8).wrapping_mul(11).wrapping_add(seed)).collect() }

fn fake_disk() -> Atr {
    let mut a = Atr::new_dos2();
    a.dos2_add("DOS.SYS", &bytes(4625, 1)).unwrap();
    a.dos2_add("DUP.SYS", &bytes(5126, 2)).unwrap();
    let mut auto = bytes(6000, 3);
    auto[50..50 + 33].copy_from_slice(b"TURBO-BASIC XL 1.5 (c) 1985 Frank");
    a.dos2_add("AUTORUN.SYS", &auto).unwrap();
    a.dos2_add("MAGIC.TUR", &bytes(125 * 50, 5)).unwrap();
    a.dos2_add("COMPILER.COM", &bytes(125 * 80, 6)).unwrap();
    for n in ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS", "MAGIC.TUR", "COMPILER.COM"] { a.dos2_set_locked(n, true).unwrap(); }
    a
}

fn project(name: &str) -> PathBuf {
    let root = tmp(name);
    std::fs::write(root.join("p.vcprj"), r#"{ "format": 1, "project": {"name": "T"}, "targets": [{ "id": "atari8", "system": "atari8", "cpu": "sally" }],
  "sources": [{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["p.tur"] }], "main": "p.tur" }"#).unwrap();
    std::fs::write(root.join("p.tur"), "PRINT \"HI\"\n").unwrap();
    root
}

#[test]
fn turbo_check_meldet_kennung_und_exitcode() {
    let d = tmp("check");
    std::fs::write(d.join("t.atr"), fake_disk().to_bytes()).unwrap();
    std::fs::write(d.join("leer.atr"), Atr::new_dos2().to_bytes()).unwrap();
    let o = vcc(&["turbo", "check", "t.atr"], &d);
    let t = out(&o);
    assert!(o.status.success() && t.contains("Turbo BASIC XL") && t.contains("1.5") && t.contains("sectors free"), "{t}");
    let o = vcc(&["turbo", "check", "t.atr", "--json"], &d);
    let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(j["turbo"], true); assert_eq!(j["version"], "1.5"); assert_eq!(j["dos2"], true);
    let o = vcc(&["turbo", "check", "leer.atr"], &d);
    assert_eq!(o.status.code(), Some(1)); assert!(out(&o).contains("does not look like Turbo BASIC XL"), "{}", out(&o));
    assert_eq!(vcc(&["turbo", "check", "gibts.atr"], &d).status.code(), Some(1));
    assert_eq!(vcc(&["turbo"], &d).status.code(), Some(2));
    assert_eq!(vcc(&["turbo", "check", "--nope"], &d).status.code(), Some(2));
}

#[test]
fn build_ohne_diskette_hinweis_mit_strict_fehler_dann_bereitstellen_und_bauen() {
    let root = project("ablauf");
    let src = tmp("ablauf-src").join("Turbo.atr");
    std::fs::write(&src, fake_disk().to_bytes()).unwrap();
    let before = std::fs::read(&src).unwrap();
    let o = vcc(&["build"], &root);
    assert!(o.status.success(), "{}", out(&o));
    assert!(out(&o).contains("needs a Turbo BASIC XL disk") && out(&o).contains("vcc turbo provide"), "{}", out(&o));
    let o = vcc(&["build", "--strict"], &root);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    // bereitstellen mit Verkleinern: ohne Terminal und ohne --yes wird abgebrochen und nichts verändert
    let o = vcc(&["turbo", "provide", src.to_str().unwrap(), ".", "--reduce"], &root);
    assert_eq!(o.status.code(), Some(1)); assert!(out(&o).contains("Cancelled"), "{}", out(&o));
    assert!(!root.join("turbo").exists());
    let o = vcc(&["turbo", "provide", src.to_str().unwrap(), ".", "--reduce", "--yes"], &root);
    let t = out(&o);
    assert!(o.status.success() && t.contains("Copy reduced: 2 files removed") && t.contains("original stays unchanged") && t.contains(".gitignore"), "{t}");
    assert_eq!(std::fs::read(&src).unwrap(), before);
    let o = vcc(&["build", "--strict"], &root);
    let t = out(&o);
    assert!(o.status.success() && t.contains("build/p.atr") && t.contains("AUTORUN.BAS"), "{t}");
    let o = vcc(&["disk", "list", "build/p.atr"], &root);
    let t = out(&o);
    assert!(o.status.success() && t.contains("AUTORUN.BAS") && !t.contains("COMPILER.COM"), "{t}");
    // Kopie bereits verkleinert: nichts zu entfernen
    let o = vcc(&["turbo", "reduce", "--yes"], &root);
    assert!(o.status.success() && out(&o).contains("nothing to remove"), "{}", out(&o));
    // starten ohne Projektdatei-Diskette meldet sich verständlich
    let o = vcc(&["turbo", "run", "/gibt/es/nicht"], &root);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn platzmangel_im_bau_nennt_den_weg() {
    let root = project("platz");
    let mut d = fake_disk();
    d.dos2_add("FUELLER.DAT", &bytes(125 * (d.free_sectors() - 3), 1)).unwrap();
    std::fs::create_dir_all(root.join("turbo")).unwrap();
    std::fs::write(root.join("turbo/T.atr"), d.to_bytes()).unwrap();
    std::fs::write(root.join("p.vcprj"), std::fs::read_to_string(root.join("p.vcprj")).unwrap().replace("\"main\": \"p.tur\"", "\"main\": \"p.tur\", \"turbo\": { \"disk\": \"turbo/T.atr\", \"files\": [\"gross.dat\"] }")).unwrap();
    std::fs::write(root.join("gross.dat"), bytes(125 * 20, 1)).unwrap();
    let o = vcc(&["build"], &root);
    assert_eq!(o.status.code(), Some(1));
    assert!(out(&o).contains("sectors, but only 3 are free") && out(&o).contains("vcc turbo reduce"), "{}", out(&o));
    assert!(vcc(&["turbo", "reduce", "--yes"], &root).status.success());
    let o = vcc(&["build"], &root);
    assert!(o.status.success(), "{}", out(&o));
}

#[test]
fn disk_atr_base_ersetzt_loescht_und_laesst_die_quelle_unveraendert() {
    let d = tmp("base");
    std::fs::write(d.join("in.atr"), fake_disk().to_bytes()).unwrap();
    std::fs::write(d.join("neu.dat"), bytes(300, 7)).unwrap();
    std::fs::write(d.join("autorun.bas"), bytes(200, 8)).unwrap();
    let before = std::fs::read(d.join("in.atr")).unwrap();
    // geschützte Datei: ohne --force Fehler, kein Ergebnis
    let o = vcc(&["disk", "atr", "-o", "out.atr", "--base", "in.atr", "--delete", "COMPILER.COM"], &d);
    assert_eq!(o.status.code(), Some(1)); assert!(out(&o).contains("write-protected"), "{}", out(&o));
    assert!(!d.join("out.atr").exists());
    let o = vcc(&["disk", "atr", "-o", "out.atr", "--base", "in.atr", "--delete", "COMPILER.COM", "--delete", "magic.tur", "--put", "autorun.bas", "--put", "neu.dat:NEU.DAT", "--force"], &d);
    assert!(o.status.success(), "{}", out(&o));
    let a = Atr::from_bytes(&std::fs::read(d.join("out.atr")).unwrap()).unwrap();
    assert!(a.dos2_read("COMPILER.COM").is_err() && a.dos2_read("MAGIC.TUR").is_err());
    assert_eq!(a.dos2_read("AUTORUN.BAS").unwrap(), bytes(200, 8));
    assert_eq!(a.dos2_read("NEU.DAT").unwrap(), bytes(300, 7));
    assert!(a.dos2_check_split().0.is_empty());
    assert_eq!(std::fs::read(d.join("in.atr")).unwrap(), before, "Ausgangsabbild unverändert");
    // ersetzen
    std::fs::write(d.join("neu.dat"), bytes(10, 1)).unwrap();
    assert!(vcc(&["disk", "atr", "-o", "out2.atr", "--base", "out.atr", "--put", "neu.dat:NEU.DAT"], &d).status.success());
    assert_eq!(Atr::from_bytes(&std::fs::read(d.join("out2.atr")).unwrap()).unwrap().dos2_read("NEU.DAT").unwrap(), bytes(10, 1));
    // Fehlerfälle
    assert_eq!(vcc(&["disk", "atr", "-o", "in.atr", "--base", "in.atr", "--put", "neu.dat"], &d).status.code(), Some(1), "Quelle darf nicht das Ziel sein");
    assert_eq!(vcc(&["disk", "atr", "-o", "x.atr", "--put", "neu.dat"], &d).status.code(), Some(1), "--put braucht --base");
    assert_eq!(vcc(&["disk", "atr", "-o", "x.atr", "--base", "in.atr", "--boot", "a.xex"], &d).status.code(), Some(1));
    // ohne --base bleibt --add wie bisher
    assert!(vcc(&["disk", "atr", "-o", "neu.atr", "--add", "neu.dat"], &d).status.success());
}
