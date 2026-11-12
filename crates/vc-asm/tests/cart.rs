// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_asm::{assemble, to_bin, to_car, to_crt, to_prg, to_rom, to_vic20_cart, to_xex, Options, Output, Policy};
use vc_target::load_cpu;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn build(cpu: &str, src: &str) -> Result<Output, String> {
    assemble(src, &load_cpu(&data(), cpu).unwrap(), &Options::new("t.asm", Policy::Allow)).map_err(|e| format!("{} {}", e[0].code, e[0].message))
}
fn be32(b: &[u8], o: usize) -> u32 { u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }

#[test]
fn banks_have_separate_address_spaces() {
    // dieselbe Adresse in zwei Bänken ist kein Überlapp; innerhalb einer Bank schon
    let o = build("6510", ".bank 0\n.org $8000\n.byte 1,2\n.bank 1\n.org $8000\n.byte 3\n.bank 0\n.byte 9\n").unwrap();
    let b = |n: u32| o.segments.iter().filter(|s| s.bank == n).flat_map(|s| s.bytes.clone()).collect::<Vec<u8>>();
    assert_eq!((b(0), b(1)), (vec![1, 2, 9], vec![3]), "Bank 0 setzt hinter der letzten Position fort");
    assert!(build("6510", ".bank 0\n.org $8000\n.byte 1,2\n.org $8001\n.byte 5\n").unwrap_err().starts_with("E301"));
    assert!(to_prg(&o).unwrap_err().contains("Bänke") && to_bin(&o).is_err() && to_xex(&o).is_err());
    assert!(build("6510", ".bank 2000\n").unwrap_err().starts_with("E203"));
}

#[test]
fn named_segments_check_their_range() {
    let ok = build("6510", ".segment \"CODE\", $8000, $8003\n.org $8000\n.byte 1,2,3,4\n").unwrap();
    assert_eq!(ok.segments[0].name, "CODE");
    assert!(build("6510", ".segment \"CODE\", $8000, $8003\n.org $8000\n.byte 1,2,3,4,5\n").unwrap_err().contains("CODE"));
    assert!(build("6510", ".segment \"DATA\", $9000, $9fff\n.org $8000\n.byte 1\n").unwrap_err().starts_with("E301"));
    // unter .pseudopc zählt die echte Adresse
    assert!(build("6510", ".segment \"ROM\", $8000, $8fff\n.org $8000\n.pseudopc $c000\n nop\n.endpseudopc\n").is_ok());
    assert!(build("6510", ".segment CODE\n").unwrap_err().starts_with("E208"));
}

#[test]
fn c64_crt_normal_and_banked() {
    let n = build("6510", ".cartridge \"normal\", \"Test\"\n.org $8000\n.word $8009,$8009\n.byte $c3,$c2,$cd,$38,$30\n rts\n").unwrap();
    let f = to_crt(&n, None).unwrap();
    assert_eq!(&f[..16], b"C64 CARTRIDGE   "); assert_eq!(be32(&f, 16), 0x40);
    assert_eq!((f[0x16], f[0x17], f[0x18], f[0x19]), (0, 0, 0, 1), "Hardware 0, EXROM 0, GAME 1 (8K)");
    assert_eq!(&f[0x20..0x24], b"TEST"); assert_eq!(&f[0x40..0x44], b"CHIP");
    assert_eq!(be32(&f, 0x44), 16 + 0x2000); assert_eq!(f.len(), 0x40 + 16 + 0x2000);
    assert_eq!(&f[0x48..0x4C], &[0, 0, 0, 0], "Typ ROM, Bank 0"); assert_eq!(&f[0x4C..0x50], &[0x80, 0, 0x20, 0], "Adresse $8000, Größe $2000");
    assert_eq!(f[0x50 + 4], 0xC3, "Kennung CBM80 bei $8004"); assert_eq!(f[0x50 + 0x1FFF], 0xFF, "Rest ist $FF");
    let big = build("6510", ".org $8000\n.byte 1\n.org $a000\n.byte 2\n").unwrap();
    let f = to_crt(&big, Some("normal")).unwrap(); assert_eq!((f[0x18], f[0x19]), (0, 0), "16K: EXROM 0, GAME 0"); assert_eq!(f.len(), 0x40 + 16 + 0x4000);
    assert!(to_crt(&build("6510", ".org $c000\n nop\n").unwrap(), None).is_err());
    // Magic Desk / Ocean: Bänke werden auf die gültige Größe aufgefüllt
    let b = build("6510", ".bank 0\n.org $8000\n.byte 1\n.bank 5\n.org $8000\n.byte 2\n").unwrap();
    let md = to_crt(&b, Some("magicdesk")).unwrap(); assert_eq!((md[0x16], md[0x17]), (0, 19)); assert_eq!(md.len(), 0x40 + 8 * (16 + 0x2000), "6 Bänke → 8");
    let oc = to_crt(&b, Some("ocean")).unwrap(); assert_eq!(oc.len(), 0x40 + 16 * (16 + 0x2000), "mindestens 128 KB"); assert_eq!((oc[0x16], oc[0x17], oc[0x18], oc[0x19]), (0, 5, 0, 0));
    assert!(to_crt(&b, Some("normal")).is_err() && to_crt(&b, Some("xyz")).is_err());
    // cartconv (VICE) prüft die Dateien, wenn vorhanden
    let c = PathBuf::from("/Applications/vice-arm64-gtk3-3.10/bin/cartconv");
    if c.exists() {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")); std::fs::create_dir_all(&dir).unwrap(); std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in [("md.crt", &md), ("oc.crt", &oc), ("n.crt", &to_crt(&n, None).unwrap())] {
            let p = dir.join(name); std::fs::write(&p, bytes).unwrap();
            let r = Command::new(&c).args(["-c", p.to_str().unwrap()]).output().unwrap();
            let text = String::from_utf8_lossy(&r.stdout).to_string() + &String::from_utf8_lossy(&r.stderr);
            assert!(r.status.success() && !text.to_lowercase().contains("error") && !text.to_lowercase().contains("warning"), "{name}: {text}");
        }
    }
}

#[test]
fn atari_car_and_xegs() {
    let o = build("sally", ".org $a000\nstart: nop\n jmp start\n.run start\n").unwrap();
    let f = to_car(&o, None).unwrap();
    assert_eq!(&f[..4], b"CART"); assert_eq!(be32(&f, 4), 1); assert_eq!(f.len(), 16 + 0x2000);
    let rom = &f[16..]; assert_eq!(be32(&f, 8), rom.iter().map(|b| *b as u32).sum::<u32>(), "Prüfsumme");
    assert_eq!(&rom[0x1FF9..], &[0x60, 0x00, 0xA0, 0x00, 0x04, 0xF9, 0xBF], "RTS, Run, Flag, Option, Init");
    assert_eq!(to_rom(&o, None).unwrap(), rom);
    assert_eq!(be32(&to_car(&build("sally", ".org $8000\n nop\n.org $a000\n nop\n.run $a000\n").unwrap(), None).unwrap(), 4), 2, "16K");
    assert!(to_car(&build("sally", ".org $a000\n nop\n").unwrap(), None).unwrap_err().contains("Startadresse"));
    // eigene Vektoren werden nicht überschrieben
    let own = build("sally", ".org $a000\n nop\n.org $bffa\n.word $a000\n.byte 0,0\n.word $a000\n").unwrap();
    assert_eq!(&to_rom(&own, None).unwrap()[0x1FFA..0x1FFC], &[0x00, 0xA0]);
    // XEGS: vier Bänke, die letzte fest bei $A000
    let x = build("sally", ".cartridge \"xegs\"\n.bank 0\n.org $8100\n.byte 7\n.bank 3\n.org $a000\nstart: nop\n.run start\n").unwrap();
    let f = to_car(&x, None).unwrap(); assert_eq!(be32(&f, 4), 12); assert_eq!(f.len(), 16 + 4 * 0x2000);
    assert_eq!(f[16 + 0x100], 7); assert_eq!(f[16 + 3 * 0x2000], 0xEA, "feste Bank zuletzt");
    let big = build("sally", ".bank 0\n.org $8000\n nop\n.bank 7\n.org $a000\n.run $a000\n nop\n").unwrap();
    assert_eq!(be32(&to_car(&big, Some("xegs")).unwrap(), 4), 13, "8 Bänke = 64 KB");
}

#[test]
fn easyflash_actionreplay_and_ultimax() {
    // EasyFlash: Bank 0 mit Reset-Vektoren (ROMH bei $E000), Bank 1 nur ROML
    let src = ".bank 0\n.org $8000\n.byte 1\n.org $e000\nreset: jmp reset\n.org $fffc\n.word reset\n.bank 1\n.org $8000\n.byte 2\n";
    let b = build("6510", src).unwrap();
    let ef = to_crt(&b, Some("easyflash")).unwrap();
    assert_eq!((ef[0x16], ef[0x17], ef[0x18], ef[0x19]), (0, 32, 1, 0), "Hardware 32, EXROM 1, GAME 0");
    assert_eq!(ef.len(), 0x40 + 4 * (16 + 0x2000), "zwei Bänke × (ROML + ROMH)");
    assert_eq!(&ef[0x48..0x4C], &[0, 2, 0, 0], "Typ Flash, Bank 0");
    assert_eq!(&ef[0x4C..0x50], &[0x80, 0, 0x20, 0]);
    let second = 0x40 + 16 + 0x2000;
    assert_eq!(&ef[second + 0x0C..second + 0x10], &[0xA0, 0, 0x20, 0], "ROMH im CRT bei $A000");
    assert_eq!(ef[second + 0x10], 0x4C, "jmp reset ($E000)");
    assert!(to_crt(&build("6510", ".bank 0\n.org $8000\n.byte 1\n").unwrap(), Some("easyflash")).unwrap_err().contains("ROMH"));
    // Action Replay: immer vier Bänke, EXROM 0 / GAME 1
    let ar = to_crt(&build("6510", ".bank 2\n.org $8000\n.byte 7\n").unwrap(), Some("actionreplay")).unwrap();
    assert_eq!((ar[0x16], ar[0x17], ar[0x18], ar[0x19]), (0, 1, 0, 1)); assert_eq!(ar.len(), 0x40 + 4 * (16 + 0x2000));
    assert!(to_crt(&build("6510", ".bank 4\n.org $8000\n.byte 7\n").unwrap(), Some("actionreplay")).is_err());
    // Ultimax: ROML + ROMH, EXROM 1 / GAME 0
    let ul = to_crt(&build("6510", ".org $8000\n.byte 1\n.org $e000\nr: jmp r\n.org $fffc\n.word r\n").unwrap(), Some("ultimax")).unwrap();
    assert_eq!((ul[0x16], ul[0x17], ul[0x18], ul[0x19]), (0, 0, 1, 0)); assert_eq!(ul.len(), 0x40 + 2 * (16 + 0x2000));
    assert!(to_crt(&build("6510", ".org $8000\n.byte 1\n").unwrap(), Some("ultimax")).is_err());
    // cartconv (VICE) prüft die Dateien, wenn vorhanden
    let c = PathBuf::from("/Applications/vice-arm64-gtk3-3.10/bin/cartconv");
    if c.exists() {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        for (name, bytes) in [("ef.crt", &ef), ("ar.crt", &ar), ("ul.crt", &ul)] {
            let p = dir.join(name); std::fs::write(&p, bytes).unwrap();
            let r = Command::new(&c).args(["-f", p.to_str().unwrap()]).output().unwrap();
            let text = String::from_utf8_lossy(&r.stdout).to_string() + &String::from_utf8_lossy(&r.stderr);
            assert!(r.status.success() && !text.to_lowercase().contains("error"), "{name}: {text}");
            if name == "ef.crt" { assert!(text.contains("EasyFlash") && text.contains("ultimax") && text.contains("FLASH"), "{text}"); }
            if name == "ar.crt" { assert!(text.contains("Action Replay") && text.contains("8k Game"), "{text}"); }
        }
    }
}

#[test]
fn vic20_cartridge_images() {
    let sig = ".word cold, cold\n.byte $41,$30,$c3,$c2,$cd\ncold: rts\n";
    let o = build("nmos6502", &format!(".cartridge \"a0\", \"T\"\n.org $A000\n{sig}")).unwrap();
    let (ext, img) = to_vic20_cart(&o, None).unwrap();
    assert_eq!(ext, "a0"); assert_eq!(img.len(), 0x2000);
    assert_eq!(&img[..11], &[0x09, 0xA0, 0x09, 0xA0, 0x41, 0x30, 0xC3, 0xC2, 0xCD, 0x60, 0xFF], "Vektoren, Kennung A0CBM, Code, danach $FF");
    // Kennung fehlt: Fehler (sonst startet das System das Modul nicht)
    let bad = build("nmos6502", ".org $A000\n.word 0,0\n.byte 1,2,3,4,5\n").unwrap();
    assert!(to_vic20_cart(&bad, None).unwrap_err().contains("A0CBM"));
    // andere Blöcke: keine Kennung nötig, Endung = Block
    for (kind, org) in [("20", 0x2000), ("40", 0x4000), ("60", 0x6000)] {
        let o = build("nmos6502", &format!(".org ${org:04X}\nrts\n")).unwrap();
        let (ext, img) = to_vic20_cart(&o, Some(kind)).unwrap();
        assert_eq!((ext, img.len(), img[0]), (kind, 0x2000, 0x60));
    }
    // Code im falschen Block, unbekannter Typ, Bänke
    let o = build("nmos6502", ".org $4000\nrts\n").unwrap();
    assert!(to_vic20_cart(&o, Some("20")).is_err() && to_vic20_cart(&o, Some("77")).unwrap_err().contains("unbekannter"));
    let o = build("nmos6502", ".bank 1\n.org $A000\nrts\n").unwrap();
    assert!(to_vic20_cart(&o, Some("a0")).unwrap_err().contains("Bänke"));
}
