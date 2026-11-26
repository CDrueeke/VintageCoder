// SPDX-License-Identifier: GPL-3.0-or-later
//! `#asm` im nativen BASIC: Platzierung hinter dem Text (Adresse wird stabil), DATA-Zeilen, `asm.name`, Fehler.
use std::path::{Path, PathBuf};
use vc_basic::blob::{asm_refs, compile_plain_atari_blobs, compile_plain_cbm_blobs, find_blobs, Assembled, Placement};
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// Ersatz für den Assembler: Code `LDA #$01 / RTS` ab `addr`, Marken `start` (= addr) und `ende` (= addr+3)
fn fake(reqs: &[vc_basic::BlobReq], addr: usize, strict: bool) -> Result<Assembled, Vec<vc_core::Diagnostic>> {
    assert!(!reqs.is_empty()); let _ = strict;
    let bytes = vec![0xA9, 0x01, 0x60, 0x00];
    Ok(Assembled { symbols: vec![("start".into(), addr as i64), ("ende".into(), addr as i64 + 3)], chunks: vec![(addr as u16, bytes.clone())], bytes })
}

#[test]
fn directive_and_references_are_found() {
    let src = "#asm \"player.asm\" with \"song.asm\"\n  #ASM \"a.asm\"\nSYS asm.START:POKE 1,asm.ende\nPRINT \"asm.zz\":REM asm.yy\nDATA asm.xx\n";
    let b = find_blobs(src, "m.bas").unwrap();
    assert_eq!(b.len(), 2);
    assert_eq!((b[0].file.as_str(), b[0].with.as_slice(), b[0].src.1), ("player.asm", &["song.asm".to_string()][..], 1));
    assert_eq!(b[1].src.1, 2);
    assert_eq!(asm_refs(src), ["start", "ende"]);
    assert_eq!(find_blobs("#asm player.asm\n", "m.bas").err().unwrap()[0].code, "B177");
    assert!(find_blobs("#asms\nPRINT\n", "m.bas").unwrap().is_empty());
}

#[test]
fn code_behind_the_text_with_a_stable_address() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "#asm \"p.asm\"\nSYS asm.start\nPOKE 53280,asm.ende\n";
    let r = compile_plain_cbm_blobs(src, &d, &PlainOptions::new("m.bas", 63999), d.load_addr, Placement::Behind, &mut fake).unwrap();
    let prg = &r.built.prg;
    // Programm ohne Maschinencode, mit der Adresse von Hand eingesetzt, ist der Anfang des PRG
    let by_hand = compile_plain_cbm(&format!("SYS {}\nPOKE 53280,{}\n", r.base, r.base + 3), &d, &PlainOptions::new("m.bas", 63999), d.load_addr).unwrap().prg;
    assert_eq!(&prg[..by_hand.len()], &by_hand[..]);
    assert_eq!(&prg[by_hand.len()..], &[0xA9, 0x01, 0x60, 0x00]);
    assert_eq!(r.base, d.load_addr as usize + by_hand.len() - 2, "Code beginnt direkt hinter dem BASIC-Text");
    assert_eq!(r.size, 4);
}

#[test]
fn works_in_number_mode_and_with_define() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "#define BORDER 53280\n#asm \"p.asm\"\n10 POKE BORDER,1\n20 SYS asm.start\n";
    let r = compile_plain_cbm_blobs(src, &d, &PlainOptions::new("m.bas", 63999), d.load_addr, Placement::Behind, &mut fake).unwrap();
    let by_hand = compile_plain_cbm(&format!("10 POKE 53280,1\n20 SYS {}\n", r.base), &d, &PlainOptions::new("m.bas", 63999), d.load_addr).unwrap().prg;
    assert_eq!(&r.built.prg[..by_hand.len()], &by_hand[..]);
}

#[test]
fn code_as_data_lines_on_systems_without_protection() {
    let d = Dialect::load(&data(), "basic65").unwrap();
    let src = "#asm \"p.asm\"\nRESTORE @asm_data\nSYS asm.start\n";
    let r = compile_plain_cbm_blobs(src, &d, &PlainOptions::new("m.bas", 63999), 0x2001, Placement::Data(Some(0xC000)), &mut fake).unwrap();
    assert_eq!(r.base, 0xC000);
    let by_hand = compile_plain_cbm("RESTORE @asm_data\nSYS 49152\n@asm_data: DATA 49152,4\nDATA 169,1,96,0\nDATA 0\n", &d, &PlainOptions::new("m.bas", 63999), 0x2001).unwrap().prg;
    assert_eq!(r.built.prg, by_hand);
    assert!(r.behind.is_empty());
}

#[test]
fn atari_puts_the_code_below_a000_on_a_page_boundary() {
    let d = vc_basic::atari::AtariDialect::load(&data(), "atari_basic").unwrap();
    let mut seen = vec![];
    let mut asm = |reqs: &[vc_basic::BlobReq], addr: usize, strict: bool| { seen.push(addr); fake(reqs, addr, strict) };
    let r = compile_plain_atari_blobs("#asm \"p.asm\"\nPOKE 106,asm.base_page\nX=USR(asm.start)\n", &d, &PlainOptions::new("m.bas", 32767), &mut asm).unwrap();
    assert_eq!(r.base, 0x9F00);
    assert_eq!(seen, [0x6000, 0x9F00]);
    let by_hand = vc_basic::compile_plain_atari("POKE 106,159\nX=USR(40704)\n@asm_data: DATA 40704,4\nDATA 169,1,96,0\nDATA 0\n", &d, &PlainOptions::new("m.bas", 32767)).unwrap().file;
    assert_eq!(r.built.file, by_hand);
}

#[test]
fn errors() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let o = PlainOptions::new("m.bas", 63999);
    // unbekannte Marke
    let e = compile_plain_cbm_blobs("#asm \"p.asm\"\nSYS asm.nirgends\n", &d, &o, d.load_addr, Placement::Behind, &mut fake).err().unwrap();
    assert_eq!(e[0].code, "B177");
    assert_eq!(e[0].arg_text("name"), "nirgends");
    // Nummernbereich der DATA-Zeilen
    let e = compile_plain_cbm_blobs("#asm \"p.asm\"\n63990 SYS asm.start\n", &d, &o, d.load_addr, Placement::Data(Some(0xC000)), &mut fake).err().unwrap();
    assert_eq!(e[0].code, "B173");
    // ohne Anbindung des Assemblers
    let e = compile_plain_cbm("#asm \"p.asm\"\nPRINT\n", &d, &o, d.load_addr).err().unwrap();
    assert_eq!(e[0].code, "B177");
    // ohne #asm ist es der gewöhnliche Bau
    let r = compile_plain_cbm_blobs("PRINT 1\n", &d, &o, d.load_addr, Placement::Behind, &mut fake).unwrap();
    assert_eq!(r.built.prg, compile_plain_cbm("PRINT 1\n", &d, &o, d.load_addr).unwrap().prg);
}

#[test]
fn old_name_blob_is_gone() {
    assert!(vc_basic::blob::is_blob_directive("#asm \"a.asm\""));
    assert!(!vc_basic::blob::is_blob_directive("#blob \"a.asm\""));
}

#[test]
fn old_name_gives_a_helpful_error() {
    let e = compile_plain_cbm("#blob \"a.asm\"\nPRINT 1\n", &vc_basic::Dialect::load(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"), "cbm_v2").unwrap(), &PlainOptions::new("m.bas", 63999), 0x0801).err().expect("Fehler");
    assert_eq!((e[0].code, e[0].line), ("B177", 1));
    assert!(e[0].message.contains("#asm"), "{}", e[0].message);
}
