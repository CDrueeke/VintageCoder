// SPDX-License-Identifier: GPL-3.0-or-later
//! Test gegen echtes atari800 (öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1.
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vc_emu::{Atari800, Backend, RunState};

#[test]
fn atari800_breakpoint_step_and_memory() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let xex = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../build/colorbars.xex");
    assert!(xex.exists(), "build/colorbars.xex fehlt");
    let mut a = Atari800::launch(xex.to_str().unwrap(), None).expect("atari800 starten");
    assert_eq!(a.state(), RunState::Stopped);
    a.set_breakpoint(0x2005).unwrap();                                    // .line: ldx antic.vcount
    a.cont().unwrap();
    assert_eq!(a.state(), RunState::Running);
    let end = Instant::now() + Duration::from_secs(30);
    while a.state() == RunState::Running && Instant::now() < end { std::thread::sleep(Duration::from_millis(50)); }
    assert_eq!(a.state(), RunState::Stopped, "Haltepunkt nicht erreicht");
    let r = a.registers().unwrap();
    assert_eq!(r.pc, 0x2005, "{r:?}");
    assert!(r.extra.iter().any(|(n, _)| n == "LIN"));
    assert_eq!(a.read_mem(0x2000, 6).unwrap(), vec![0xA9, 0x00, 0x8D, 0x0E, 0xD4, 0xAE]);
    a.step().unwrap();
    assert_eq!(a.registers().unwrap().pc, 0x2008);
    a.write_mem(0x0600, &[0x11, 0x22, 0x33]).unwrap();
    assert_eq!(a.read_mem(0x0600, 3).unwrap(), vec![0x11, 0x22, 0x33]);
    assert!(a.pause().is_err() && !a.can_pause());
    a.clear_breakpoint(0x2005).unwrap();
    assert!(a.breakpoints().is_empty());
    a.stop();
}
