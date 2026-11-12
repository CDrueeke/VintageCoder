// SPDX-License-Identifier: GPL-3.0-or-later
//! VC-20: Speicherausbau (Projekteinstellung `memory`) bestimmt BASIC-Start, Bildschirm- und Farb-RAM.
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};
use vc_media::wizard;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn project(tag: &str, template: &str, memory: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-vic20-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    wizard::create(&data(), &dir, "probe", &["vic20".to_string()], template).unwrap();
    if let Some(m) = memory {
        let f = dir.join("probe.vcprj");
        let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        v["targets"][0]["memory"] = m.into();
        std::fs::write(&f, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    }
    dir
}

#[test]
fn memory_configs_of_the_target() {
    let mut t = vc_target::load_target(&data(), "vic20").unwrap();
    assert_eq!(t.cpu, "nmos6502"); assert_eq!(t.basic_load_addr, Some(0x1001));
    assert_eq!(t.memory.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["unexpanded", "3k", "8k", "16k", "24k", "35k"]);
    t.apply_memory(None).unwrap(); assert_eq!(t.basic_load_addr, Some(0x1001));
    t.apply_memory(Some("3k")).unwrap(); assert_eq!(t.basic_load_addr, Some(0x0401));
    t.apply_memory(Some("8k")).unwrap(); assert_eq!(t.basic_load_addr, Some(0x1201));
    let syms = t.memory_symbols();
    assert!(syms.contains(&("MEM_SCREEN_RAM".into(), 0x1000)) && syms.contains(&("MEM_COLOR_RAM".into(), 0x9400)), "{syms:?}");
    assert!(t.apply_memory(Some("1m")).unwrap_err().contains("unbekannter Speicherausbau"));
    // Systeme ohne Ausbaustufen bleiben unberührt
    let mut c64 = vc_target::load_target(&data(), "c64").unwrap();
    c64.apply_memory(Some("8k")).unwrap(); assert_eq!(c64.basic_load_addr, Some(0x0801)); assert!(c64.memory_symbols().is_empty());
}

#[test]
fn assembler_project_follows_the_memory_setting() {
    // Das Vorlagenprogramm setzt `.org $1001`; die Systemdatei liefert je Ausbau andere Konstanten
    let probe = "\n#include <vic20/kernal.asm>\n.org BASIC_START\n.basicstub start\nstart: lda #COL_WHITE\n sta COLOR_RAM\n sta SCREEN_RAM\n rts\n";
    for (mem, load, screen, color) in [(None, 0x1001u16, 0x1E00u16, 0x9600u16), (Some("3k"), 0x0401, 0x1E00, 0x9600), (Some("8k"), 0x1201, 0x1000, 0x9400)] {
        let dir = project(mem.unwrap_or("default"), "empty", mem);
        std::fs::write(dir.join("probe.asm"), format!(".target vic20{probe}")).unwrap();
        let out = build(&Ctx { root: &dir, data: &data() }, "probe.asm", Some("vic20")).unwrap();
        assert_eq!(out["ok"], true, "{mem:?}: {out}");
        let prg = std::fs::read(dir.join(out["artifact"].as_str().unwrap())).unwrap();
        assert_eq!(u16::from_le_bytes([prg[0], prg[1]]), load, "{mem:?}: Ladeadresse");
        let sta: Vec<u16> = prg.windows(3).filter(|w| w[0] == 0x8D).map(|w| u16::from_le_bytes([w[1], w[2]])).collect();
        assert_eq!(sta, vec![color, screen], "{mem:?}: Farb-/Bildschirm-RAM");
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn basic_project_uses_the_basic_start_of_the_memory_setting() {
    for (mem, load) in [(None, 0x1001u16), (Some("8k"), 0x1201), (Some("3k"), 0x0401)] {
        let dir = project(&format!("bas-{}", mem.unwrap_or("default")), "basic-hello", mem);
        let rel = "src/vic20/main.bas";
        let out = build(&Ctx { root: &dir, data: &data() }, rel, Some("vic20")).unwrap();
        assert_eq!(out["ok"], true, "{mem:?}: {out}");
        let prg = std::fs::read(dir.join(out["artifact"].as_str().unwrap())).unwrap();
        assert_eq!(u16::from_le_bytes([prg[0], prg[1]]), load, "{mem:?}");
        // die erste Zeilenverknüpfung zeigt hinter die Startadresse
        let link = u16::from_le_bytes([prg[2], prg[3]]);
        assert!(link > load && link < load + 200, "{mem:?}: Verknüpfung ${link:04X}");
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn cartridge_files_start_as_modules_in_vice() {
    use std::path::Path;
    for (f, flag) in [("x.a0", "-cartA"), ("x.20", "-cart2"), ("x.40", "-cart4"), ("x.60", "-cart6"), ("X.A0", "-cartA")] {
        assert_eq!(vc_ide::launch::vic20_cart_arg(Path::new(f)).map(|a| a[0].clone()).as_deref(), Some(flag), "{f}");
    }
    assert!(vc_ide::launch::vic20_cart_arg(Path::new("x.prg")).is_none() && vc_ide::launch::vic20_cart_arg(Path::new("x")).is_none());
}

#[test]
fn cartridge_template_builds_a_module_image() {
    let dir = project("cart", "cartridge", None);
    let out = build(&Ctx { root: &dir, data: &data() }, "src/vic20/module.asm", Some("vic20")).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["artifact"], "build/module.a0");
    let img = std::fs::read(dir.join("build/module.a0")).unwrap();
    assert_eq!(img.len(), 8192); assert_eq!(&img[4..9], &[0x41, 0x30, 0xC3, 0xC2, 0xCD]);
    std::fs::remove_dir_all(&dir).ok();
}
