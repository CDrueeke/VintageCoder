// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: Anlaufstörung nachstellen (Abfrageschleife wie im Tauri-Spike, sofort nach `launch`). Nur Handmessung.
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
fn main() {
    let prg = std::env::args().nth(1).expect("PRG");
    let delay: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut v = Vice::launch(&prg, None).expect("VICE");
    std::thread::sleep(Duration::from_millis(delay));
    let t = Instant::now();
    let mut n = 0;
    let mut cont_sent = 0;
    while t.elapsed() < Duration::from_secs(6) {
        let f = v.grab_frame().unwrap();
        if n % 4 == 0 {
            let st = v.state();
            let r = v.registers().unwrap();
            if st != vc_emu::RunState::Running { v.cont().unwrap(); cont_sent += 1; }
            let nonblack = f.rgba.chunks(4).filter(|p| p[..3] != [0, 0, 0]).count();
            if n % 16 == 0 { eprintln!("[{:?}] n={n} Zustand {:?} PC {:04X} nicht-schwarze Punkte {nonblack} border-rgb {:?}", t.elapsed(), st, r.pc, &f.rgba[..3]); }
        }
        n += 1;
        std::thread::sleep(Duration::from_millis(60));
    }
    eprintln!("cont gesendet: {cont_sent}");
    v.stop();
}
