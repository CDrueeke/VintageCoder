// SPDX-License-Identifier: GPL-3.0-or-later
//! Quelltexte mit BOM, CRLF oder Windows-1252 bauen genauso wie die Normalform (UTF-8 ohne BOM, LF).
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};

fn build_bytes(name: &str, ext: &str, bytes: &[u8], target: &str) -> serde_json::Value {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = manifest.join("../../data");
    let dir = std::env::temp_dir().join(format!("vc-tf-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("main.{ext}")), bytes).unwrap();
    let out = build(&Ctx { root: &dir, data: &data }, &format!("main.{ext}"), Some(target)).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    out
}

#[test]
fn assembler_accepts_bom_crlf_and_windows_1252() {
    let norm = "; Gr\u{fc}\u{df}e\n.target c64\n.org $0801\nlda #$01\nrts\n";
    let ok = build_bytes("asm0", "asm", norm.as_bytes(), "c64");
    assert_eq!(ok["ok"], true, "{ok}");
    let mut bom = b"\xEF\xBB\xBF".to_vec(); bom.extend_from_slice(norm.replace('\n', "\r\n").as_bytes());
    let b = build_bytes("asm1", "asm", &bom, "c64");
    assert_eq!(b["ok"], true, "{b}");
    let mut cp = b"; Gr\xFC\xDFe\r\n".to_vec(); cp.extend_from_slice(b".target c64\r\n.org $0801\r\nlda #$01\r\nrts\r\n");
    let c = build_bytes("asm2", "asm", &cp, "c64");
    assert_eq!(c["ok"], true, "{c}");
}

#[test]
fn basic_accepts_bom_and_crlf() {
    let b = build_bytes("bas1", "bas", b"\xEF\xBB\xBF10 PRINT \"HALLO\"\r\n20 END\r\n", "c64");
    assert_eq!(b["ok"], true, "{b}");
}
