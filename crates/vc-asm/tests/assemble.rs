// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_asm::{assemble, to_prg, Options, Policy};
use vc_target::{load_cpu, Cpu, Mode};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn cpu() -> Cpu { load_cpu(&data(), "6510").unwrap() }
fn opts(p: Policy) -> Options { Options::new("test.asm", p) }

#[test]
fn opcode_table_is_complete() {
    let c = cpu();
    let mut seen = [false; 256];
    for op in &c.ops { assert!(!seen[op.opcode as usize], "doppelt: {:02X}", op.opcode); seen[op.opcode as usize] = true; }
    assert!(seen.iter().all(|s| *s), "Tabelle hat Lücken");
    assert_eq!(c.ops.iter().filter(|o| o.group == "official").count(), 151);
}

/// Jede kanonische (Mnemonic, Modus)-Kombination muss mit der erwarteten Länge und dem richtigen Opcode assembliert werden.
#[test]
fn every_instruction_roundtrips() {
    let c = cpu();
    for op in c.canonical_ops() {
        let operand = match op.mode {
            Mode::Imp => "", Mode::Acc => " a", Mode::Imm => " #$12", Mode::Zp => " $12", Mode::Zpx => " $12,x", Mode::Zpy => " $12,y",
            Mode::Abs => " $1234", Mode::Abx => " $1234,x", Mode::Aby => " $1234,y", Mode::Ind => " ($1234)",
            Mode::Izx => " ($12,x)", Mode::Izy => " ($12),y", Mode::Rel => " *+2", _ => unreachable!("nur 6502-Modi"),
        };
        let src = format!(".org $1000\n {}{}\n", op.mnemonic, operand);
        let out = assemble(&src, &c, &opts(Policy::Allow)).unwrap_or_else(|e| panic!("{src:?}: {e:?}"));
        let bytes = &out.segments[0].bytes;
        assert_eq!(bytes[0], op.opcode, "{src:?}");
        assert_eq!(bytes.len(), op.bytes as usize, "{src:?}");
    }
}

#[test]
fn basic_stub_and_loop_match_known_bytes() {
    let src = ".org $0801\n.basicstub start\nstart: lda #$00\n sta $d020\n jmp start\n";
    let out = assemble(src, &cpu(), &opts(Policy::Warn)).unwrap();
    let prg = to_prg(&out).unwrap();
    assert_eq!(prg, vec![0x01, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x9E, 0x32, 0x30, 0x36, 0x31, 0x00, 0x00, 0x00,
                         0xA9, 0x00, 0x8D, 0x20, 0xD0, 0x4C, 0x0D, 0x08]);
}

#[test]
fn forward_references_and_zero_page_shrinking() {
    let src = ".org $c000\n lda zpvar\n lda later\n bne done\n nop\ndone: rts\nlater = $20\nzpvar = $fb\n";
    let out = assemble(src, &cpu(), &opts(Policy::Warn)).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0xA5, 0xFB, 0xA5, 0x20, 0xD0, 0x01, 0xEA, 0x60]);
}

#[test]
fn illegal_policy() {
    let src = ".org $1000\n lax $d012\n";
    assert!(assemble(src, &cpu(), &opts(Policy::Deny)).is_err());
    let out = assemble(src, &cpu(), &opts(Policy::Warn)).unwrap();
    assert_eq!(out.warnings[0].code, "W104");
    let out = assemble(src, &cpu(), &opts(Policy::Allow)).unwrap();
    assert!(out.warnings.is_empty());
    let local = ".org $1000\n.cpu 6510 { illegals = deny }\n lax $d012\n";
    assert!(assemble(local, &cpu(), &opts(Policy::Allow)).is_err());
    // offizielle CPU ohne Illegals
    let strict = load_cpu(&data(), "nmos6502").unwrap();
    let e = assemble(src, &strict, &opts(Policy::Allow)).unwrap_err();
    assert_eq!(e[0].code, "E201");
}

#[test]
fn local_labels_expressions_and_data() {
    let src = ".org $2000\nA: ldx #0\n.l: inx\n bne .l\nB: ldx #1\n.l: inx\n bne .l\n.byte <A, >A, 1+2*3, \"hi\"\n.word B\n.fill 2, $ff\n";
    let out = assemble(src, &cpu(), &opts(Policy::Warn)).unwrap();
    assert_eq!(&out.segments[0].bytes[..], &[0xA2,0,0xE8,0xD0,0xFD, 0xA2,1,0xE8,0xD0,0xFD, 0x00,0x20,7,b'h',b'i', 0x05,0x20, 0xFF,0xFF]);
}

#[test]
fn errors_are_reported() {
    for (src, code) in [(".org $1000\n foo\n", "E205"), (".org $1000\n lda #300\n", "E203"), (".org $1000\n lda nowhere\n", "E202"),
                        (".org $1000\n bne $2000\n", "E203"), (" nop\n", "E300"), (".org $1000\nx: nop\nx: nop\n", "E204")] {
        let e = assemble(src, &cpu(), &opts(Policy::Warn)).unwrap_err();
        assert!(e.iter().any(|d| d.code == code), "{src:?} -> {e:?}");
    }
}
