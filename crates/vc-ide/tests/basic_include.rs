// SPDX-License-Identifier: GPL-3.0-or-later
//! Natives BASIC: `#include` bindet exportierte DATA-Zeilen eines Sprites ein (Build exportiert automatisch).
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};

#[test]
fn basic_includes_exported_data_lines() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = manifest.join("../../data");
    let dir = std::env::temp_dir().join(format!("vc-bi-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("graphics")).unwrap();
    std::fs::copy(manifest.join("../../examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("graphics/hero.vcsprite")).unwrap();
    std::fs::write(dir.join("main.bas"), "REM test\nRESTORE @hero\nFOR I=1 TO 3\nREAD A\nPRINT A\nNEXT I\nEND\n#include \"build/export/hero.data.bas\"\n").unwrap();
    let out = build(&Ctx { root: &dir, data: &data }, "main.bas", Some("c64")).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    let listing = std::fs::read_to_string(dir.join("build/main.classic.txt")).unwrap().to_ascii_uppercase();
    assert!(listing.contains("DATA"), "{listing}");
    assert!(listing.contains("RESTORE "), "{listing}");
    // fehlende Datei: Fehler zeigt auf die Zeile der #include-Anweisung
    std::fs::write(dir.join("bad.bas"), "PRINT 1\n#include \"fehlt.bas\"\n").unwrap();
    let bad = build(&Ctx { root: &dir, data: &data }, "bad.bas", Some("c64")).unwrap();
    assert_eq!(bad["ok"], false);
    assert_eq!(bad["diagnostics"][0]["line"], 2, "{bad}");
    assert_eq!(bad["diagnostics"][0]["code"], "B120");
    std::fs::remove_dir_all(&dir).ok();
}
