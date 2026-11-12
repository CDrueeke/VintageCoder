// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn exported_sprite_tables_assemble_and_hold_the_right_pointers() {
    let root = std::env::temp_dir().join(format!("vc_sx_{}", std::process::id())); std::fs::create_dir_all(root.join("graphics")).unwrap();
    let hires: Vec<String> = (0..21).map(|_| "0".repeat(24)).collect(); let mc: Vec<String> = (0..21).map(|_| "0".repeat(12)).collect();
    let sprite = serde_json::json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "held", "colors": { "sprite": 7 },
        "frames": [ { "rows": hires }, { "mode": "multicolor", "rows": mc }, { "rows": hires } ],
        "animations": [ { "name": "lauf", "mode": "pingpong", "steps": [ { "frame": 0, "delay": 4 }, { "frame": 1, "delay": 3 }, { "frame": 2, "delay": 5 } ] } ] });
    std::fs::write(root.join("graphics/held.vcsprite"), sprite.to_string()).unwrap();
    // Sprite-Daten ab $2000 (Blocknummer $80), danach die Tabellen
    std::fs::write(root.join("main.asm"), ".target c64\n.org $2000\n.include \"build/export/held.asm\"\n.include \"build/export/held.anim.asm\"\n").unwrap();
    let v = build(&Ctx { root: &root, data: &data() }, "main.asm", None).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let prg = std::fs::read(root.join("build/main.prg")).unwrap();
    assert_eq!(&prg[0..2], &[0x00, 0x20]);
    let body = &prg[2..];
    // 3 Frames × 64 Byte, dann _modes (3 Byte), dann Zeiger (4 Schritte) und Dauern (4 Schritte)
    assert_eq!(&body[192..195], &[0, 1, 0], "Modus je Frame");
    assert_eq!(&body[195..199], &[0x80, 0x81, 0x82, 0x81], "Sprite-Zeiger: $2000/64 = $80 … (hin und zurück)");
    assert_eq!(&body[199..203], &[4, 3, 5, 3], "Dauern");
    std::fs::remove_dir_all(&root).ok();
}
