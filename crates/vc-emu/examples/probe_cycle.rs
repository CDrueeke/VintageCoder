// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: Wirkung von Anhalten und Fortsetzen allein auf das Emulationstempo (ohne Bildabruf). Nur Handmessung.
//! Aufruf: probe_cycle <prg> <takt_ms> <haltezeit_ms>
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
fn jiffies(b: &mut Vice) -> u32 { let m = b.read_mem(0xA0, 3).unwrap(); m[0] as u32 * 65536 + m[1] as u32 * 256 + m[2] as u32 }
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (period, hold): (u64, u64) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let mut v = Vice::launch(&a[1], None).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    let j0 = jiffies(&mut v); v.cont().unwrap(); let t0 = Instant::now();
    let mut n = 0;
    while t0.elapsed() < Duration::from_secs(8) {
        let t = Instant::now();
        v.registers().unwrap(); std::thread::sleep(Duration::from_millis(hold)); v.cont().unwrap(); n += 1;
        std::thread::sleep(Duration::from_millis(period).saturating_sub(t.elapsed()));
    }
    let j1 = jiffies(&mut v); let dt = t0.elapsed().as_secs_f64();
    eprintln!("Takt {period} ms, Halt {hold} ms: {n} Zyklen, Tempo {:.0} %", (j1 - j0) as f64 / 60.2 / dt * 100.0);
    v.stop();
}
