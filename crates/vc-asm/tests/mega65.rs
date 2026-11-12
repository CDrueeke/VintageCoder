// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_asm::{assemble, disassemble, Options, Policy};
use vc_target::{load_cpu, Cpu};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn cpu() -> Cpu { load_cpu(&data(), "45gs02").unwrap() }
fn asm(src: &str) -> Vec<u8> {
    let out = assemble(&format!(".org $2000\n{src}\n"), &cpu(), &Options::new("t.asm", Policy::Deny)).unwrap_or_else(|e| panic!("{src}: {e:?}"));
    out.segments[0].bytes.clone()
}
fn dis(b: &[u8]) -> Vec<String> { disassemble(&cpu(), b, 0x2000).into_iter().map(|l| l.text).collect() }
fn err(src: &str) -> String { match assemble(&format!(".org $2000\n{src}\n"), &cpu(), &Options::new("t.asm", Policy::Deny)) { Err(e) => e[0].code.to_string(), Ok(_) => "ok".into() } }

#[test]
fn table_is_complete_and_unique() {
    let c = cpu();
    assert_eq!(c.ops.len(), 256);
    assert!((0..=255u8).all(|o| c.by_opcode(o).is_some()));
    assert_eq!(c.nop32, Some(0xEA)); assert_eq!(c.quad_prefix, vec![0x42, 0x42]);
}

#[test]
fn ce02_and_mega65_instructions() {
    assert_eq!(asm("lda #$42\nldx #1\nldz #$00\ntab\ntba\ntaz\ntza\nmap\neom\ncle\nsee\nneg\nasr\ninz\ndez\nphz\nplz\ntsy\ntys"),
               vec![0xA9, 0x42, 0xA2, 1, 0xA3, 0, 0x5B, 0x7B, 0x4B, 0x6B, 0x5C, 0xEA, 0x02, 0x03, 0x42, 0x43, 0x1B, 0x3B, 0xDB, 0xFB, 0x0B, 0x2B]);
    assert_eq!(asm("lda ($10),z\nsta ($10,sp),y\nlda ($20,sp),y\nlda ($10),y\nlda ($10,x)"), vec![0xB2, 0x10, 0x82, 0x10, 0xE2, 0x20, 0xB1, 0x10, 0xA1, 0x10]);
    assert_eq!(asm("phw #$1234\nphw $1234\ninw $10\ndew $10\nasw $1234\nrow $1234\nrts #5\njsr ($1234)\njsr ($1234,x)\njmp ($1234,x)"),
               vec![0xF4, 0x34, 0x12, 0xFC, 0x34, 0x12, 0xE3, 0x10, 0xC3, 0x10, 0xCB, 0x34, 0x12, 0xEB, 0x34, 0x12, 0x62, 5, 0x22, 0x34, 0x12, 0x23, 0x34, 0x12, 0x7C, 0x34, 0x12]);
    assert_eq!(asm("stz $10\nstz $1000\nstz $1000,x\ncpz #1\ntsb $10\ntrb $1000\nrmb3 $10\nsmb7 $10"), vec![0x64, 0x10, 0x9C, 0, 0x10, 0x9E, 0, 0x10, 0xC2, 1, 0x04, 0x10, 0x1C, 0, 0x10, 0x37, 0x10, 0xF7, 0x10]);
}

#[test]
fn branches_short_long_and_bit() {
    // kurz: relativ zum Ende des Befehls; lang: relativ zu Opcode+2; BBR/BBS: zp, Ziel relativ zu Opcode+3
    assert_eq!(asm("start: bne start"), vec![0xD0, 0xFE]);
    assert_eq!(asm("start: lbne start"), vec![0xD3, 0xFE, 0xFF]);
    assert_eq!(asm("lbra there\nnop\nthere: rts"), vec![0x83, 0x02, 0x00, 0xEA, 0x60]);
    assert_eq!(asm("start: bsr start"), vec![0x63, 0xFE, 0xFF]);
    assert_eq!(asm("start: bbr0 $10,start\nbbs7 $20,start"), vec![0x0F, 0x10, 0xFD, 0xFF, 0x20, 0xFA]);
    assert_eq!(err("bne far\n.fill 200\nfar: nop"), "E203", "kurzer Sprung zu weit");
    assert_eq!(asm("lbne far\n.fill 200\nfar: nop")[..3], [0xD3, 0xC9, 0x00]);
}

#[test]
fn quad_and_32_bit_pointers() {
    assert_eq!(asm("lda [$10],z\nsta [$20],z\nadc [$30],z"), vec![0xEA, 0xB2, 0x10, 0xEA, 0x92, 0x20, 0xEA, 0x72, 0x30]);
    assert_eq!(asm("ldq $10\nldq $1234\nstq $10\nstq $1234\nadcq $10\nsbcq $1234\ncpq $10\nandq $10\norq $10\neorq $10\nbitq $10"),
               vec![0x42, 0x42, 0xA5, 0x10, 0x42, 0x42, 0xAD, 0x34, 0x12, 0x42, 0x42, 0x85, 0x10, 0x42, 0x42, 0x8D, 0x34, 0x12, 0x42, 0x42, 0x65, 0x10, 0x42, 0x42, 0xED, 0x34, 0x12,
                    0x42, 0x42, 0xC5, 0x10, 0x42, 0x42, 0x25, 0x10, 0x42, 0x42, 0x05, 0x10, 0x42, 0x42, 0x45, 0x10, 0x42, 0x42, 0x24, 0x10]);
    assert_eq!(asm("aslq q\nlsrq a\nrolq $10\nrorq $1234,x\ninq\ndeq $10\nasrq $10,x\nnegq"), // Akkumulator-/Q-Operand
               vec![0x42, 0x42, 0x0A, 0x42, 0x42, 0x4A, 0x42, 0x42, 0x26, 0x10, 0x42, 0x42, 0x7E, 0x34, 0x12, 0x42, 0x42, 0x1A, 0x42, 0x42, 0xC6, 0x10, 0x42, 0x42, 0x54, 0x10, 0x42, 0x42, 0x42]);
    assert_eq!(asm("ldq ($10),z\nldq [$10],z\nstq ($10)\nstq [$10]"), vec![0x42, 0x42, 0xB2, 0x10, 0x42, 0x42, 0xEA, 0xB2, 0x10, 0x42, 0x42, 0x92, 0x10, 0x42, 0x42, 0xEA, 0x92, 0x10]);
    assert_eq!(err("ldq #1"), "E206", "kein Q-Immediate");
    // Q gibt es nur auf der 45GS02
    let n = load_cpu(&data(), "6510").unwrap();
    assert!(assemble(".org $2000\n ldq $10\n", &n, &Options::new("t.asm", Policy::Allow)).is_err());
}

#[test]
fn disassembler_round_trip() {
    let src = "lda #$42\nldz #$00\nlda ($10),z\nsta ($10,sp),y\nlda [$10],z\nphw #$1234\njsr ($1234,x)\nldq $1234\nstq [$10],z\naslq q\ninq\nbbr3 $10,there\nlbra there\nthere: neg\nasr\neom\nrts #5";
    let b = asm(src);
    assert_eq!(dis(&b), vec!["lda #$42", "ldz #$00", "lda ($10),z", "sta ($10,sp),y", "lda [$10],z", "phw #$1234", "jsr ($1234,x)", "ldq $1234", "stq [$10],z", "aslq q", "inq", "bbr3 $10,$2027", "lbra $2027", "neg", "asr", "eom", "rts #$05"]);
}

#[test]
fn branchlong_jmp_keeps_far_branches_6502_compatible() {
    let far = |mode: &str| { let src = format!(".org $2000\n .branchlong {mode}\n beq far\n .fill 200, $ea\nfar: nop\n"); assemble(&src, &cpu(), &Options::new("t.asm", Policy::Deny)).unwrap().segments[0].bytes.clone() };
    // auto: 45GS02 verlängert mit dem langen Verzweigungsbefehl LBEQ (3 Byte); jmp: BNE *+5 ; JMP far wie auf dem 6502
    assert_eq!(far("auto").len(), 3 + 200 + 1);
    let b = far("jmp");
    assert_eq!(&b[..5], &[0xD0, 0x03, 0x4C, 0xCD, 0x20]);
    assert_eq!(b.len(), 5 + 200 + 1);
}
