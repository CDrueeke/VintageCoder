// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildabruf und Texteingabe gegen echtes VICE (x64sc, öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1.
//! `VC_FRAME_PPM=<Datei>` schreibt das erste Bild zusätzlich als PPM (zum Ansehen).
use std::time::{Duration, Instant};
use vc_emu::{ascii_to_petscii, find_program, Backend, RunState, Vice};

#[test]
fn vice_frame_while_running_and_stopped_and_typed_text() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    if find_program("x64sc", &["/Applications/vice-arm64-gtk3-3.10/bin", "/opt/homebrew/bin", "/usr/local/bin"]).is_none() { eprintln!("übersprungen (x64sc nicht gefunden)"); return; }
    let mut v = Vice::launch("", None).expect("VICE starten");
    assert!(v.can_grab_frame() && v.can_type_text());
    v.cont().unwrap();                                                  // nach dem Verbinden wartet VICE im Monitor
    std::thread::sleep(Duration::from_secs(5));                         // BASIC-Startbild abwarten
    // 1. laufende Maschine: Bild holen, danach läuft sie weiter
    let f = v.grab_frame().expect("Bild");
    assert_eq!((f.width, f.height), (504, 312));
    assert_eq!(f.rgba.len(), 504 * 312 * 4);
    if let Ok(p) = std::env::var("VC_FRAME_PPM") {
        let mut o = format!("P6\n{} {}\n255\n", f.width, f.height).into_bytes();
        for p in f.rgba.chunks(4) { o.extend_from_slice(&p[..3]); }
        std::fs::write(p, o).unwrap();
    }
    assert!(f.rgba.chunks(4).any(|p| p[..3] != f.rgba[..3]), "Bild ist einfarbig");
    assert_eq!(v.state(), RunState::Running, "der Bildabruf muss die Maschine fortsetzen");
    let (j0, j1) = { let a = v.read_mem(0xA2, 1).unwrap()[0]; std::thread::sleep(Duration::from_millis(500)); (a, v.read_mem(0xA2, 1).unwrap()[0]) };
    let _ = (j0, j1);
    // 2. Text eingeben, während sie läuft
    v.cont().unwrap();
    v.type_text(&ascii_to_petscii("print 123\n")).unwrap();
    assert_eq!(v.state(), RunState::Running);
    std::thread::sleep(Duration::from_millis(1500));
    v.pause().unwrap();
    assert_eq!(v.state(), RunState::Stopped);
    let screen = v.read_mem(0x0400, 1000).unwrap();
    assert!(screen.windows(3).any(|w| w == [0x31, 0x32, 0x33]), "„123“ nicht im Bildschirmspeicher");
    // 3. angehaltene Maschine bleibt angehalten
    let t = Instant::now();
    let g = v.grab_frame().unwrap();
    eprintln!("Bild bei Halt: {:?}", t.elapsed());
    assert_eq!(v.state(), RunState::Stopped);
    assert!(g.frame_no > f.frame_no);
    // 4. Zeitlupen-Live: Bild, 20 ms warten, Bild … – die Maschine läuft dazwischen weiter und danach noch
    v.cont().unwrap();
    let jiffies = |v: &mut Vice| { let b = v.read_mem(0xA0, 3).unwrap(); b[0] as u32 * 65536 + b[1] as u32 * 256 + b[2] as u32 };
    let _ = v.grab_frame().unwrap();
    let (j0, t0) = (jiffies(&mut v), Instant::now());   // der Jiffy-Zähler lesende Befehl hielt an: Zähler und Uhr zusammen messen
    v.cont().unwrap();
    let mut n = 0;
    while t0.elapsed() < Duration::from_secs(3) { v.grab_frame().unwrap(); n += 1; std::thread::sleep(Duration::from_millis(20)); }
    assert_eq!(v.state(), RunState::Running);
    v.pause().unwrap();
    let tempo = (jiffies(&mut v) - j0) as f64 / 60.0 / t0.elapsed().as_secs_f64();
    eprintln!("Zeitlupe: {n} Bilder in 3 s, Tempo {:.0} %", tempo * 100.0);
    assert!(tempo > 0.2, "Tempo {tempo:.2} zu gering");
    v.stop();
}

/// Bildgrößen der übrigen VICE-Programme (VC-20, Plus/4, C128 mit VIC-II und VDC) laut Messungen
#[test]
fn vice_frame_sizes_of_other_systems() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    for (prog, sizes) in [("xvic", vec![(568, 284)]), ("xplus4", vec![(384, 288)]), ("x128", vec![(384, 272), (856, 288)])] {
        if find_program(prog, &["/Applications/vice-arm64-gtk3-3.10/bin", "/opt/homebrew/bin", "/usr/local/bin"]).is_none() { eprintln!("{prog} nicht gefunden"); continue; }
        let mut v = Vice::launch_system(prog, "", None).expect("VICE starten");
        v.cont().unwrap();
        std::thread::sleep(Duration::from_secs(4));
        let srcs = v.frame_sources().len();
        for (i, (w, h)) in sizes.iter().enumerate() {
            v.set_frame_source(i);
            let f = v.grab_frame().unwrap_or_else(|e| panic!("{prog} Quelle {i}: {e}"));
            assert_eq!((f.width, f.height), (*w, *h), "{prog} Quelle {i}");
            if i == 0 { assert!(f.rgba.chunks(4).any(|p| p[..3] != f.rgba[..3]), "{prog}: Bild einfarbig"); }   // der VDC ist ohne 80-Spalten-Modus leer
            assert_eq!(v.state(), RunState::Running);
        }
        assert_eq!(srcs, if prog == "x128" { 2 } else { 0 });
        v.stop();
    }
}
