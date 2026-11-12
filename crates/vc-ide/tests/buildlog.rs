// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::{Path, PathBuf};
use vc_ide::build::{build, Ctx};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_bl_{name}_{}", std::process::id())); std::fs::create_dir_all(&d).unwrap(); d }

fn stage<'a>(v: &'a serde_json::Value, title: &str) -> &'a serde_json::Value {
    v["log"]["stages"].as_array().unwrap().iter().find(|s| s["title"] == title).unwrap_or_else(|| panic!("Stufe '{title}' fehlt: {}", v["log"]))
}
fn files(st: &serde_json::Value) -> Vec<String> { st["files"].as_array().unwrap().iter().map(|f| f["path"].as_str().unwrap().to_string()).collect() }

#[test]
fn assembler_build_logs_every_stage_and_intermediate_file() {
    let root = tmp("asm");
    std::fs::write(root.join("lib.inc"), ".const LIVES = 3\n.macro border(c)\n lda #c\n sta $d020\n.endmacro\n").unwrap();
    std::fs::write(root.join("main.asm"), ".target c64\n.org $0801\n.basicstub start\n.include \"lib.inc\"\nstart: lda #LIVES\n border 5\n jmp *\n").unwrap();
    let v = build(&Ctx { root: &root, data: &data() }, "main.asm", None).unwrap();
    assert_eq!(v["ok"], true);
    let titles: Vec<&str> = v["log"]["stages"].as_array().unwrap().iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Vorbereitung", "Medien exportieren", "Assemblieren", "Ausgabe schreiben", "Zwischendateien"]);
    let asm = stage(&v, "Assemblieren");
    let lines: Vec<&str> = asm["lines"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
    assert!(lines.iter().any(|l| l.contains("2 Datei(en)")), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("Makros: 1 (BORDER)")), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("Durchläufe:")), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("Segment $0801")), "{lines:?}");
    assert_eq!(files(asm), ["lib.inc"], "Include als Datei des Protokolls");
    // jede im Protokoll genannte Datei gibt es wirklich, die Zwischendateien sind lesbar
    let mid = files(stage(&v, "Zwischendateien"));
    for n in ["main.expanded.asm", "main.lst", "main.sym.txt", "main.map.txt", "main.vs", "main.vcdbg"] { assert!(mid.contains(&format!("build/{n}")), "{n} fehlt in {mid:?}"); }
    for st in v["log"]["stages"].as_array().unwrap() { for f in st["files"].as_array().unwrap() { assert!(Path::new(f["abs"].as_str().unwrap()).exists(), "{f}"); } }
    let ex = std::fs::read_to_string(root.join("build/main.expanded.asm")).unwrap();
    assert!(ex.contains("; ===== lib.inc =====") && ex.contains(".const LIVES = 3") && !ex.contains(".include"), "{ex}");
    assert!(std::fs::read_to_string(root.join("build/main.sym.txt")).unwrap().contains("LIVES"));
    assert!(std::fs::read_to_string(root.join("build/main.map.txt")).unwrap().contains("$0801"));
    assert_eq!(files(stage(&v, "Ausgabe schreiben")), ["build/main.prg"]);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn failed_build_logs_the_error_stage() {
    let root = tmp("fail");
    std::fs::write(root.join("bad.asm"), ".target c64\n.org $0801\n lda #nix\n").unwrap();
    let v = build(&Ctx { root: &root, data: &data() }, "bad.asm", None).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["log"]["status"], "error");
    let st = stage(&v, "Assemblieren");
    assert_eq!(st["status"], "error");
    assert!(st["lines"].as_array().unwrap().iter().any(|l| l.as_str().unwrap().starts_with("Fehler bad.asm:3")), "{st}");
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn sbas_is_switched_off_and_native_basic_builds_without_line_numbers() {
    let root = tmp("bas");
    std::fs::write(root.join("old.sbas"), "print 1\n").unwrap();
    let e = build(&Ctx { root: &root, data: &data() }, "old.sbas", Some("c64")).unwrap_err();
    assert!(e.contains("abgeschaltet"), "{e}");
    std::fs::write(root.join("prog.bas"), "for i = 1 to 3\n print i\nnext i\n@ende:\ngoto @ende\n").unwrap();
    let v = build(&Ctx { root: &root, data: &data() }, "prog.bas", Some("c64")).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let classic = std::fs::read_to_string(root.join("build/prog.classic.txt")).unwrap();
    assert!(classic.to_lowercase().contains("for") && classic.contains("goto 50") || classic.to_lowercase().contains("goto"), "{classic}");
    std::fs::remove_dir_all(&root).ok();
}
