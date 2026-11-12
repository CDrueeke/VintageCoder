// SPDX-License-Identifier: GPL-3.0-or-later
//! Test gegen echtes VICE (öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1.
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vc_emu::{Backend, RunState, Vice};

fn wait_stopped(b: &mut dyn Backend, secs: u64) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end { if b.state() == RunState::Stopped { return true; } std::thread::sleep(Duration::from_millis(50)); }
    false
}

#[test]
fn vice_breakpoint_step_memory_and_pause() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let prg = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/border.prg");
    assert!(prg.exists(), "build/border.prg fehlt (vcc asm examples/c64/border/border.asm -o build/border.prg)");
    let mut v = Vice::launch(prg.to_str().unwrap(), None).expect("VICE starten");
    v.set_breakpoint(0x080D).unwrap();
    assert_eq!(v.breakpoints(), vec![0x080D]);
    v.cont().unwrap();
    assert!(wait_stopped(&mut v, 20), "Haltepunkt bei $080D nicht erreicht");
    let r = v.registers().unwrap();
    assert_eq!(r.pc, 0x080D, "{r:?}");
    assert!(r.extra.iter().any(|(n, _)| n == "LIN") && r.extra.iter().any(|(n, _)| n == "CYC"));
    assert_eq!(v.read_mem(0x080D, 8).unwrap(), vec![0x78, 0xA9, 0x00, 0x8D, 0x21, 0xD0, 0xA9, 0x80]);   // sei / lda #0 / sta $d021 / lda #$80
    v.step().unwrap();
    assert_eq!(v.registers().unwrap().pc, 0x080E);
    v.step().unwrap();
    let r2 = v.registers().unwrap();
    assert_eq!((r2.pc, r2.a), (0x0810, 0));
    v.write_mem(0x0400, &[1, 2, 3]).unwrap();
    assert_eq!(v.read_mem(0x0400, 3).unwrap(), vec![1, 2, 3]);
    v.clear_breakpoint(0x080D).unwrap();
    assert!(v.breakpoints().is_empty());
    v.cont().unwrap();
    assert_eq!(v.state(), RunState::Running);
    std::thread::sleep(Duration::from_millis(800));
    v.pause().unwrap();
    assert_eq!(v.state(), RunState::Stopped);
    let r3 = v.registers().unwrap();
    assert!((0x0810..=0x0830).contains(&r3.pc), "PC {:04X} nicht in der Schleife", r3.pc);
    v.stop();
}
