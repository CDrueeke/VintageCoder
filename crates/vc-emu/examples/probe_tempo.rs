// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: Emulationstempo von VICE ohne Bildstrom (Jiffy-Zähler), als Vergleichswert. Nur Handmessung.
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
fn jiffies(b: &mut Vice) -> u32 { let m = b.read_mem(0xA0, 3).unwrap(); m[0] as u32 * 65536 + m[1] as u32 * 256 + m[2] as u32 }
fn main() {
    let mut v = Vice::launch(&std::env::args().nth(1).unwrap(), None).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    for round in 0..3 {
        let j0 = jiffies(&mut v); v.cont().unwrap(); let t0 = Instant::now();
        std::thread::sleep(Duration::from_secs(4));
        let j1 = jiffies(&mut v); let dt = t0.elapsed().as_secs_f64();
        v.cont().unwrap();
        eprintln!("Runde {round}: {} Jiffies in {dt:.2} s = {:.1} /s", j1 - j0, (j1 - j0) as f64 / dt);
    }
    v.stop();
}
