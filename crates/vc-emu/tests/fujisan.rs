// SPDX-License-Identifier: GPL-3.0-or-later
//! Lebendtest gegen ein echtes Fujisan. Übersprungen, wenn Fujisan (Umgebung `VC_FUJISAN_EXE` oder automatische Suche) oder das gebaute
//! Beispiel `examples/atari/jumprun/build/jumprun.xex` fehlt. Läuft nicht, wenn auf Port 6502 schon etwas anderes lauscht.
//! Alle Tests teilen sich den Port: `cargo test -p vc-emu --test fujisan -- --test-threads=1`.
use std::time::{Duration, Instant};
use vc_emu::{fujisan, Backend, Fujisan, RunState};

#[test]
fn fujisan_haltepunkt_schritt_speicher() {
    let exe = std::env::var("VC_FUJISAN_EXE").ok().or_else(fujisan::find_exe);
    let xex = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/atari/jumprun/build/jumprun.xex");
    let Some(exe) = exe else { eprintln!("übersprungen: Fujisan nicht gefunden"); return; };
    if !xex.is_file() { eprintln!("übersprungen: {} fehlt (vcc build examples/atari/jumprun)", xex.display()); return; }
    let mut f = Fujisan::launch(xex.to_str().unwrap(), Some(&exe), fujisan::DEFAULT_PORT).expect("Fujisan starten");
    assert_eq!(f.state(), RunState::Stopped);
    assert_eq!(f.registers().unwrap().pc, 0x2000, "nach dem Laden steht der PC am Einsprung");
    // Haltepunkt auf `vbi` (jumprun.sym.txt: $21F3) trifft in jedem Bild
    f.set_breakpoint(0x21F3).unwrap();
    f.cont().unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    while f.state() == RunState::Running && Instant::now() < end { std::thread::sleep(Duration::from_millis(10)); }
    assert_eq!(f.state(), RunState::Stopped);
    assert_eq!(f.registers().unwrap().pc, 0x21F3);
    assert!(matches!(f.last_hit(), Some(vc_emu::Hit::Exec(0x21F3))));
    // Befehlsbytes ab dem Haltepunkt: LDA #$01 / STA $21EC / JSR $6003
    assert_eq!(f.read_mem(0x21F3, 8).unwrap(), [0xA9, 0x01, 0x8D, 0xEC, 0x21, 0x20, 0x03, 0x60]);
    f.step().unwrap();
    assert_eq!(f.registers().unwrap().pc, 0x21F5);
    // Lesen über mehrere Stücke, Schreiben und Zurücklesen im freien Speicher ($0600)
    assert_eq!(f.read_mem(0x2000, 700).unwrap().len(), 700);
    f.write_mem(0x0600, &[1, 2, 3]).unwrap();
    assert_eq!(f.read_mem(0x0600, 3).unwrap(), [1, 2, 3]);
    f.clear_breakpoint(0x21F3).unwrap();
    f.stop();
}

fn fujisan_exe() -> Option<String> { std::env::var("VC_FUJISAN_EXE").ok().or_else(fujisan::find_exe) }

/// Register setzen über den Vorspann: Programm im RAM, PC/A/X/Y/SP/Flags setzen, laufen lassen, Ergebnis prüfen
#[test]
fn fujisan_register_setzen_und_laufen_lassen() {
    let Some(exe) = fujisan_exe() else { eprintln!("übersprungen: Fujisan nicht gefunden"); return; };
    let mut f = Fujisan::launch_bare(Some(&exe), fujisan::DEFAULT_PORT).expect("Fujisan starten");
    assert_eq!(f.state(), RunState::Stopped);
    assert_eq!(f.registers().unwrap().pc, 0x0600);
    // $0700: ADC #$01 / STA $0680 / STX $0681 / STY $0682 / PHP / PLA / STA $0683 / TSX / STX $0684 / JMP $0700+14
    let code: [u8; 24] = [0x69, 0x01, 0x8D, 0x80, 0x06, 0x8E, 0x81, 0x06, 0x8C, 0x82, 0x06, 0x08, 0x68, 0x8D, 0x83, 0x06, 0xBA, 0x8E, 0x84, 0x06, 0x4C, 0x14, 0x07, 0x00];
    f.write_mem(0x0700, &code).unwrap();
    let r = vc_emu::Regs { pc: 0x0700, a: 0x41, x: 0x12, y: 0x34, sp: 0xF0, flags: 0x01 | 0x20, extra: vec![] };   // Carry gesetzt: ADC liefert $43
    assert!(f.trap_caps().set_regs);
    f.set_regs(&r).unwrap();
    assert_eq!(f.registers().unwrap().pc, 0x0700);
    f.set_breakpoint(0x0714).unwrap();
    f.cont().unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    while f.state() == RunState::Running && Instant::now() < end { std::thread::sleep(Duration::from_millis(10)); }
    assert_eq!(f.registers().unwrap().pc, 0x0714);
    let m = f.read_mem(0x0680, 5).unwrap();
    assert_eq!(m[0], 0x43, "A + 1 + Carry");
    assert_eq!((m[1], m[2]), (0x12, 0x34));
    assert_eq!(m[3] & 0x21, 0x20, "Carry nach ADC gelöscht, Flag 5 gesetzt");
    assert_eq!(m[4], 0xF0, "Stapelzeiger");
    // zweiter Durchlauf an derselben Stelle (PC steht auf der Falle selbst)
    f.clear_breakpoint(0x0714).unwrap();
    f.set_regs(&vc_emu::Regs { pc: 0x0700, a: 0x00, x: 0x01, y: 0x02, sp: 0xFF, flags: 0x24, extra: vec![] }).unwrap();
    assert_eq!(f.read_mem(0x0714, 3).unwrap(), [0x4C, 0x14, 0x07], "überschriebene Bytes sind wiederhergestellt");
    f.stop();
}

/// Bild über `screen.capture` (PCX → RGBA)
#[test]
fn fujisan_bild_abholen() {
    let Some(exe) = fujisan_exe() else { eprintln!("übersprungen: Fujisan nicht gefunden"); return; };
    let mut f = Fujisan::launch_bare(Some(&exe), fujisan::DEFAULT_PORT).expect("Fujisan starten");
    assert!(f.can_grab_frame());
    let fr = f.grab_frame().unwrap();
    assert_eq!((fr.width, fr.height), (336, 240));
    assert_eq!(fr.rgba.len(), 336 * 240 * 4);
    assert!(fr.rgba.chunks(4).any(|p| p[..3] != fr.rgba[..3]), "Bild ist nicht einfarbig");
    f.stop();
}
