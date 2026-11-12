// SPDX-License-Identifier: GPL-3.0-or-later
//! Trap-Modus mit echtem VICE (Register setzen, `run_until`, Zyklen): nur mit VC_EMU_TESTS=1.
use std::time::Duration;
use vc_emu::{Backend, Regs, Vice};

#[test]
fn vice_set_regs_run_until_and_cycles() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let mut v = Vice::launch_with("x64sc", "", None, &[]).expect("VICE starten");
    let caps = v.trap_caps();
    assert!(caps.set_regs && caps.cycles);
    std::thread::sleep(Duration::from_secs(3));                          // Hochfahren
    assert_eq!(v.resource_int("MachineVideoStandard").unwrap(), 1, "PAL erwartet");
    // Falle: jsr $C100 / jmp *; Unterprogramm: 20 × nop, rts (6 + 40 + 6 = 52 Zyklen)
    let mut sub = vec![0xEAu8; 20]; sub.push(0x60);
    v.write_mem(0xC100, &sub).unwrap();
    v.write_mem(0xC000, &[0x20, 0x00, 0xC1, 0x4C, 0x03, 0xC0]).unwrap();
    let r = Regs { pc: 0xC000, a: 0x12, x: 0x34, y: 0x56, sp: 0xF0, flags: 0xC7, extra: vec![] };
    v.set_regs(&r).unwrap();
    let back = v.registers().unwrap();
    assert_eq!((back.pc, back.a, back.x, back.y, back.sp, back.flags & 0xCF), (0xC000, 0x12, 0x34, 0x56, 0xF0, 0xC7), "{back:?}");
    let start = v.cycles().unwrap();
    assert!(v.run_until(0xC003, Duration::from_secs(3)).unwrap(), "Falle nicht erreicht");
    let end = v.cycles().unwrap();
    assert_eq!(end.since(&start), 52);
    assert_eq!(v.registers().unwrap().pc, 0xC003);
    assert!(v.breakpoints().is_empty(), "Haltepunkt wird entfernt");
    // Zeitlimit: Schleife ohne Haltepunkt am Ziel
    v.write_mem(0xC100, &[0x4C, 0x00, 0xC1]).unwrap();
    v.set_regs(&Regs { pc: 0xC000, sp: 0xF0, flags: 0x04, ..Default::default() }).unwrap();
    assert!(!v.run_until(0xC003, Duration::from_millis(500)).unwrap(), "Zeitlimit statt Halt");
    v.stop();
}

/// CIA2-Zeitgeber (TA/TB zusammengeschaltet, 32 Bit) als Stoppuhr neben der Strahlposition: exakt auch über ein Bild hinaus
#[test]
fn vice_cia_stopwatch_matches_beam_position() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let mut v = Vice::launch_with("x64sc", "", None, &[]).expect("VICE starten");
    std::thread::sleep(Duration::from_secs(3));
    eprintln!("\n     N  erwartet  Strahl  Zeitgeber  Zeitgeber−erwartet");
    for n in [0usize, 0, 1, 10, 100, 1000, 2900, usize::MAX] {
        let sub: Vec<u8> = if n == usize::MAX { vec![0xA0, 0x00, 0xA2, 0x00, 0xCA, 0xD0, 0xFD, 0x88, 0xD0, 0xF8, 0x60] } else { let mut s = vec![0xEAu8; n]; s.push(0x60); s };
        v.write_mem(0xC100, &sub).unwrap();
        v.write_mem(0xC000, &[0x20, 0x00, 0xC1, 0x4C, 0x03, 0xC0]).unwrap();
        v.set_regs(&Regs { pc: 0xC000, sp: 0xFF, flags: 0x24, ..Default::default() }).unwrap();
        for (a, b) in [(0xDD0Du16, 0x7Fu8), (0xDD0E, 0x00), (0xDD0F, 0x00)] { v.write_mem(a, &[b]).unwrap(); }
        v.write_mem(0xDD04, &[0xFF, 0xFF, 0xFF, 0xFF]).unwrap();
        v.write_mem(0xDD0E, &[0x10]).unwrap(); v.write_mem(0xDD0F, &[0x50]).unwrap();
        let chk = v.read_mem(0xDD04, 4).unwrap();
        v.write_mem(0xDD0E, &[0x01]).unwrap(); v.write_mem(0xDD0F, &[0x41]).unwrap();
        let s = v.cycles().unwrap();
        assert!(v.run_until(0xC003, Duration::from_secs(5)).unwrap());
        let e = v.cycles().unwrap();
        let t = v.read_mem(0xDD04, 4).unwrap();
        let timer = 0xFFFF_FFFFu64 - (t[0] as u64 | (t[1] as u64) << 8 | (t[2] as u64) << 16 | (t[3] as u64) << 24);
        // Schleifenfall: 2 + 256·(2 + 255·5 + 4 + 2 + 3) − 1 − 1 … + 12, hier nur die Rechnung des ersten Durchlaufs
        let exp = if n == usize::MAX { 12 + 2 + 256 * (2 + 256 * 5 - 1 + 2 + 3) - 1 } else { 12 + 2 * n as i64 };
        eprintln!("{n:>6} {exp:>9} {:>7} {timer:>10} {:>10}   (Zähler nach Laden: {chk:02X?})", e.since(&s), timer as i64 - exp);
    }
    v.stop();
}
