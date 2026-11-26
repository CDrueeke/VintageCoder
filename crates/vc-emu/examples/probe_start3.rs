// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: Anlaufstörung. Startet VICE, sendet nach `wait_ms` ein `cont` und prüft nach 3 s, ob das Programm läuft
//! (PC im Programm, nicht-schwarzes Bild). Nur Handmessung.
use std::time::Duration;
use vc_emu::{Backend, Vice};
fn main() {
    let prg = std::env::args().nth(1).expect("PRG");
    let wait: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let do_cont = std::env::args().nth(3).map_or(true, |s| s != "nocont");
    let mut v = Vice::launch(&prg, None).expect("VICE");
    let st0 = v.state();
    std::thread::sleep(Duration::from_millis(wait));
    if do_cont { v.cont().unwrap(); }
    std::thread::sleep(Duration::from_secs(3));
    let st1 = v.state();
    let a = v.registers().unwrap().pc;               // hält an
    v.cont().unwrap(); std::thread::sleep(Duration::from_millis(300));
    let f = v.grab_frame().unwrap();
    let b = v.registers().unwrap().pc;
    let nonblack = f.rgba.chunks(4).filter(|p| p[..3] != [0, 0, 0]).count();
    println!("wait {wait} ms, cont {do_cont}: Zustand nach launch {st0:?}, nach 3 s {st1:?}, PC {a:04X} dann {b:04X}, nicht-schwarz {nonblack} => {}", if (0x0800..0x0900).contains(&a) { "LÄUFT" } else { "HÄNGT/ANDERS" });
    v.stop();
}
