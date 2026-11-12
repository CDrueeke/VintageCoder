// SPDX-License-Identifier: GPL-3.0-or-later
//! `"defines"` im Target der Projektdatei: Konstanten wie bei `vcc asm -D NAME=Zahl` (z. B. DEBUG für `.if DEBUG`).
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};
use vc_media::wizard;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn defines_of_the_project_target_reach_the_assembler() {
    let dir = std::env::temp_dir().join(format!("vc-defs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    wizard::create(&data(), &dir, "probe", &["c64".to_string()], "asm-hello").unwrap();
    let src = dir.join("src/c64/main.asm");
    let text = std::fs::read_to_string(&src).unwrap();
    std::fs::write(&src, text.replace("start:", ".if DEBUG\n        lda #1\n.endif\nstart:")).unwrap();
    // ohne Eintrag: DEBUG ist unbekannt
    let out = build(&Ctx { root: &dir, data: &data() }, "src/c64/main.asm", Some("c64")).unwrap();
    assert_eq!(out["ok"], false, "{out}");
    // mit "defines": DEBUG ist bekannt (Zahl und true/false)
    for (val, bytes_more) in [(serde_json::json!(1), true), (serde_json::json!(false), false)] {
        let f = dir.join("probe.vcprj");
        let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        v["targets"][0]["defines"] = serde_json::json!({ "DEBUG": val });
        std::fs::write(&f, serde_json::to_string_pretty(&v).unwrap()).unwrap();
        let out = build(&Ctx { root: &dir, data: &data() }, "src/c64/main.asm", Some("c64")).unwrap();
        assert_eq!(out["ok"], true, "{out}");
        let prg = std::fs::read(dir.join("build/main.prg")).unwrap();
        assert_eq!(prg.windows(2).any(|w| w == [0xA9, 0x01]), bytes_more, "DEBUG={val}: lda #1 {}", if bytes_more { "fehlt" } else { "unerwartet da" });
    }
    assert_eq!(vc_target::project_defines(&dir, "c64"), vec![("DEBUG".to_string(), 0)]);
    assert!(vc_target::project_defines(&dir, "atari8").is_empty());
    std::fs::remove_dir_all(&dir).ok();
}
