// SPDX-License-Identifier: GPL-3.0-or-later
//! Messung: wie oft liefert `grab_frame` (Einzelabfrage) ein Bild? Nur zur Handmessung gegen echtes VICE.
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
fn main() {
    let mut v = Vice::launch("", None).expect("VICE");
    eprintln!("Zustand nach Start: {:?}", v.state());
    v.cont().unwrap();
    std::thread::sleep(Duration::from_secs(4));
    let t0 = Instant::now();
    let mut ts = vec![];
    while t0.elapsed() < Duration::from_secs(5) { let t = Instant::now(); v.grab_frame().unwrap(); ts.push(t.elapsed().as_secs_f64() * 1000.0); }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eprintln!("{} Bilder in 5 s = {:.1}/s, Median {:.1} ms, p95 {:.1} ms", ts.len(), ts.len() as f64 / 5.0, ts[ts.len() / 2], ts[ts.len() * 95 / 100]);
    v.stop();
}
