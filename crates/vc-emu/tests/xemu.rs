// SPDX-License-Identifier: GPL-3.0-or-later
//! Test gegen echtes Xemu (öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1. Braucht xmega65 (VC_XEMU oder PATH/Programme)
//! und MEGA65.ROM (VC_XEMU_SD, ~/.config/vintagecoder/mega65 oder %APPDATA%\xemu-lgb\mega65).
//! Prüft den Monitor-Anschluss (Unix-Socket bzw. unter Windows TCP), das Laden eines Programms und das Lesen und Schreiben von Speicher.
use vc_emu::{Backend, Mega65};

#[test]
fn xemu_monitor_loads_and_reads_memory() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let sd = Mega65::xemu_sd_dir().expect("MEGA65.ROM nicht gefunden (VC_XEMU_SD setzen)");
    let mut m = Mega65::launch_xemu(None, &sd).expect("Xemu starten");
    // Programm laden (Ladeadresse $3000, Schleife `jmp *`): der PC steht danach auf dem Einsprung
    m.load_prg(&[0x00, 0x30, 0x4C, 0x00, 0x30], 0x3000).unwrap();
    let r = m.registers().unwrap();
    assert_eq!(r.pc, 0x3000, "{r:?}");
    // Die Sicht der CPU zeigt bei $2000–$7FFF das ROM-BASIC (C65-Abbildung), darunter liegt das RAM: nur in einem nicht überdeckten Bereich
    // kommt das Geschriebene zurück ($1F00 liegt unterhalb des BASIC-Textes)
    let data = [1u8, 2, 3, 4];
    m.write_mem(0x1F00, &data).unwrap();
    assert_eq!(m.read_mem(0x1F00, 4).unwrap(), data.to_vec(), "RAM bei $1F00");
    m.stop();
}
