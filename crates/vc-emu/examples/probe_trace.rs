// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnose: welche Ereignisse löst ein Bildabruf aus (mit VC_VICE_TRACE=1 ansehen). Nur Handmessung.
use std::time::Duration;
use vc_emu::{Backend, Vice};
fn main() {
    let mut v = Vice::launch(&std::env::args().nth(1).unwrap(), None).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    eprintln!("=== grab_frame x2"); v.grab_frame().unwrap(); v.grab_frame().unwrap();
    eprintln!("=== read_mem"); v.read_mem(0x400, 4).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    v.stop();
}
