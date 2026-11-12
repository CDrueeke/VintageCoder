// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Sitzung der IDE mit Fujisan als Atari-Debugger (Standardwahl). Nur mit `VC_FUJISAN_DIR` (Ordner mit ausgepacktem Fujisan,
//! z. B. nach `vcc emulator install fujisan`) und dem gebauten `examples/atari/jumprun/build/jumprun.xex`; sonst übersprungen.
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vc_ide::debug::{default_launcher, LaunchSpec, Session};

fn wait_stopped(s: &mut Session, secs: u64) -> serde_json::Value {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end { let st = s.state(); if st["state"] == "stopped" { return st; } std::thread::sleep(Duration::from_millis(20)); }
    panic!("hält nicht an")
}

#[test]
fn fujisan_ist_der_standard_atari_debugger() {
    if std::env::var_os("VC_FUJISAN_DIR").is_none() { eprintln!("übersprungen (VC_FUJISAN_DIR setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let xex = root.join("examples/atari/jumprun/build/jumprun.xex");
    if !xex.is_file() { eprintln!("übersprungen: {} fehlt", xex.display()); return; }
    let spec = LaunchSpec { artifact: xex.to_string_lossy().into_owned(), target: "atari8".into(), adapter: String::new(), host: None, password: None, entry: None, extra_args: vec![] };
    let backend = default_launcher()(&spec).expect("Debug-Start");
    assert_eq!(backend.name(), "Fujisan");
    let cpu = vc_target::load_cpu(&root.join("data"), "sally").unwrap();
    let mut s = Session::new(backend, cpu, "atari8".into(), HashMap::new());
    let st = s.state();
    assert_eq!(st["state"], "stopped");
    assert_eq!(st["regs"]["pc"].as_u64(), Some(0x2000), "{st}");
    // mehrere Haltepunkte zugleich (atari800 kennt nur einen)
    s.set_breakpoint(0x21F3, true, None, 0).unwrap();
    s.set_breakpoint(0x2201, true, None, 0).unwrap();
    assert_eq!(s.backend.breakpoints().len(), 2);
    s.cont().unwrap();
    let st = wait_stopped(&mut s, 10);
    assert!(matches!(st["regs"]["pc"].as_u64(), Some(0x21F3) | Some(0x2201)), "{st}");
    // Einzelschritt bis zum JSR bei $21F8, dann „Über“ → $21FB
    s.set_breakpoint(0x2201, false, None, 0).unwrap();
    if st["regs"]["pc"].as_u64() != Some(0x21F3) { s.cont().unwrap(); wait_stopped(&mut s, 10); }
    s.step("into").unwrap(); s.step("into").unwrap();
    assert_eq!(s.state()["regs"]["pc"].as_u64(), Some(0x21F8));
    s.step("over").unwrap();
    assert_eq!(s.state()["regs"]["pc"].as_u64(), Some(0x21FB));
    s.backend.stop();
}
