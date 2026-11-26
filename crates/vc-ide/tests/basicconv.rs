// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC umwandeln im IDE-Kern: Labels ↔ Nummern, Neu nummerieren, Modus, Einbindungen, Fehler bei berechnetem Sprungziel.
use std::path::{Path, PathBuf};
use vc_ide::basicconv::{apply, run, Kind, Req};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn proj(tag: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-bc-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
fn req<'a>(root: &'a Path, data: &'a Path, text: &'a str, kind: Kind) -> Req<'a> { Req { root, data, rel: "a.bas", text, kind, range: None, with_includes: true } }

#[test]
fn nummern_zu_labels_und_zurueck_mit_vorschau() {
    let (d, dt) = (proj("rt"), data());
    let src = "10 PRINT \"A\"\n20 GOTO 40\n30 PRINT \"B\"\n40 END\n";
    let v = run(&req(&d, &dt, src, Kind::Labels));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["mode"], "numbers");
    assert!(!v["changes"].as_array().unwrap().is_empty());
    assert!(v["newLabels"].as_array().unwrap().len() >= 1);
    let labels_text = v["text"].as_str().unwrap().to_string();
    assert!(labels_text.contains('@') && !labels_text.contains("GOTO 40"), "{labels_text}");
    // zurück zu Nummern: Ausgangsprogramm (Nummern 10/10)
    let back = run(&req(&d, &dt, &labels_text, Kind::Numbers));
    assert_eq!(back["ok"], true, "{back}");
    assert_eq!(back["mode"], "labels");
    assert_eq!(back["text"].as_str().unwrap().lines().filter(|l| l.starts_with(|c: char| c.is_ascii_digit())).count(), 4, "{}", back["text"]);
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn neu_nummerieren_mit_projektwerten_und_bereich() {
    let (d, dt) = (proj("rn"), data());
    std::fs::write(d.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"basic":{"start":100,"step":5}}"#).unwrap();
    let src = "1 PRINT 1\n2 GOTO 4\n3 PRINT 2\n4 END\n";
    let v = run(&req(&d, &dt, src, Kind::Renumber));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["numbering"]["start"], 100);
    let t = v["text"].as_str().unwrap();
    assert!(t.starts_with("100 PRINT 1") && t.contains("GOTO 115"), "{t}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn berechnetes_sprungziel_bricht_ab_und_gemischt_wird_gemeldet() {
    let (d, dt) = (proj("fehler"), data());
    let v = run(&req(&d, &dt, "10 A=1\n20 GOTO A*10\n30 END\n", Kind::Labels));
    assert_eq!(v["ok"], false, "{v}");
    assert_eq!(v["diagnostics"][0]["code"], "B161");
    assert!(v["diagnostics"].as_array().unwrap().iter().any(|d| d["code"] == "B162" && d["line"] == 2), "{v}");
    let m = run(&req(&d, &dt, "10 PRINT 1\n@x: PRINT 2\n", Kind::Mode));
    // auch beim Umwandeln eines gemischten Programms erscheint B160 in den Warnungen (Anzeige des Dialogs)
    let g = run(&req(&d, &dt, "10 PRINT 1\n@x: PRINT 2\n", Kind::Labels));
    if g["ok"] == true { assert!(g["warnings"].as_array().unwrap().iter().any(|w| w["code"] == "B160"), "{g}"); }
    assert_eq!(m["ok"], true);
    assert_eq!(m["mode"], "mixed");
    assert!(m["warnings"].as_array().unwrap().iter().any(|w| w["code"] == "B160"), "{m}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn einbindungen_werden_mit_umgewandelt_und_geschrieben() {
    let (d, dt) = (proj("inc"), data());
    std::fs::write(d.join("lib.bas"), "100 PRINT \"LIB\"\n110 RETURN\n").unwrap();
    let src = "10 GOSUB 100\n20 END\n#include \"lib.bas\"\n";
    let pv = run(&req(&d, &dt, src, Kind::Labels));
    assert_eq!(pv["ok"], true, "{pv}");
    assert_eq!(pv["includes"], 1);
    assert!(pv["files"].as_array().unwrap().iter().any(|f| f["file"] == "lib.bas" && f["changed"] == true), "{pv}");
    // Vorschau schreibt nichts
    assert!(std::fs::read_to_string(d.join("lib.bas")).unwrap().starts_with("100 PRINT"));
    let ap = apply(&req(&d, &dt, src, Kind::Labels));
    assert_eq!(ap["ok"], true, "{ap}");
    assert_eq!(ap["written"][0], "lib.bas");
    assert!(!std::fs::read_to_string(d.join("lib.bas")).unwrap().starts_with("100 PRINT"));
    // ohne Einbindungen: nur die Datei selbst
    let mut r = req(&d, &dt, src, Kind::Mode); r.with_includes = false;
    assert_eq!(run(&r)["ok"], true);
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn medien_im_include_werden_nicht_zurueckgeschrieben() {
    let (d, dt) = (proj("medium"), data());
    std::fs::create_dir_all(d.join("graphics")).unwrap();
    let sprite = d.join("graphics/hero.vcsprite");
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/sprite_demo/graphics/hero.vcsprite"), &sprite).unwrap();
    let before = std::fs::read(&sprite).unwrap();
    let src = "10 RESTORE\n20 READ A\n30 GOTO 20\n#include \"graphics/hero.vcsprite\"\n";
    let v = run(&req(&d, &dt, src, Kind::Labels));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["media"], 1);
    assert_eq!(v["includes"], 0, "Medien zählen nicht als eingebundene Dateien");
    assert!(v["files"].as_array().unwrap().iter().all(|f| !f["file"].as_str().unwrap().ends_with("vcsprite")), "{v}");
    let ap = apply(&req(&d, &dt, src, Kind::Labels));
    assert_eq!(ap["ok"], true, "{ap}");
    assert_eq!(ap["written"].as_array().unwrap().len(), 0, "{ap}");
    assert_eq!(std::fs::read(&sprite).unwrap(), before);
    std::fs::remove_dir_all(&d).ok();
}
