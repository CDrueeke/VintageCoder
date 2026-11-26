// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: Startverhalten von VICE (Zustand nach Autostart, Verlauf des ersten `cont`). Nur Handmessung.
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
fn main() {
    let prg = std::env::args().nth(1).expect("PRG");
    let t = Instant::now();
    let mut v = Vice::launch(&prg, None).expect("VICE");
    eprintln!("[{:?}] nach launch: {:?}", t.elapsed(), v.state());
    for i in 0..6 {
        std::thread::sleep(Duration::from_millis(700));
        let st = v.state();
        let r = v.registers().unwrap();
        let ln = r.extra.iter().find(|(n, _)| n == "LIN").map(|x| x.1);
        eprintln!("[{:?}] Probe {i}: Zustand vorher {:?}, PC {:04X}, LIN {:?}", t.elapsed(), st, r.pc, ln);
        if i == 2 { eprintln!("  -> cont"); v.cont().unwrap(); }
    }
    v.stop();
}
