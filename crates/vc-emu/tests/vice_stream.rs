// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildstrom gegen echtes VICE (x64sc, öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1.
//! Prüft Anlauf (Strom sofort nach dem Start, mehrfach), Rate, Bildwechsel, Tempo und sauberes Beenden.
use std::time::{Duration, Instant};
use vc_emu::framestream::{share, FrameStream, StreamConfig};
use vc_emu::{find_program, Backend, RunState, Vice};

/// BASIC-Starter `SYS 2061` und `INC $D020 / JMP` (Rahmenfarbe wechselt ständig)
fn write_prg() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("vc-stream-test-{}.prg", std::process::id()));
    std::fs::write(&p, [0x01, 0x08, 0x0b, 0x08, 0x0a, 0x00, 0x9e, 0x32, 0x30, 0x36, 0x31, 0x00, 0x00, 0x00, 0xEE, 0x20, 0xD0, 0x4C, 0x0D, 0x08]).unwrap();
    p
}
fn jiffies(b: &mut dyn Backend) -> u32 { let m = b.read_mem(0xA0, 3).unwrap(); m[0] as u32 * 65536 + m[1] as u32 * 256 + m[2] as u32 }

#[test]
fn stream_from_real_vice_start_rate_tempo_and_clean_stop() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    if find_program("x64sc", &["/Applications/vice-arm64-gtk3-3.10/bin", "/opt/homebrew/bin", "/usr/local/bin"]).is_none() { eprintln!("übersprungen (x64sc nicht gefunden)"); return; }
    let prg = write_prg();
    // Anlauf mehrfach: Strom sofort nach dem Start, ohne Wartezeit und ohne ein `cont` von außen
    for round in 0..3 {
        let v = Vice::launch(prg.to_str().unwrap(), None).expect("VICE starten");
        let be = share(Box::new(v));
        let s = FrameStream::start(be.clone(), StreamConfig { rate: 50.0, deliver_unchanged: false }).unwrap();
        // Das Programm muss anlaufen: irgendwann kommen verschiedene Bilder (Rahmenfarbe wechselt), spätestens nach 8 s
        let t = Instant::now();
        let mut changed = 0;
        while t.elapsed() < Duration::from_secs(8) && changed < 10 { if let Ok(f) = s.next(Duration::from_millis(500)) { if f.changed { changed += 1; } } }
        assert!(changed >= 10, "Runde {round}: Programm lief nicht an (nur {changed} verschiedene Bilder)");
        if round < 2 { s.stop(); be.lock().unwrap().stop(); continue; }
        // letzte Runde: Rate und Tempo messen
        let (j0, t0) = { let mut g = be.lock().unwrap(); let j = jiffies(&mut **g); g.cont().unwrap(); (j, Instant::now()) };
        let mut n = 0;
        let mut dropped = 0u64;
        while t0.elapsed() < Duration::from_secs(5) { if let Ok(f) = s.next(Duration::from_millis(500)) { n += 1; dropped += f.dropped as u64; } }
        let wall = t0.elapsed().as_secs_f64();
        let stats = s.stop();
        let (j1, t1) = { let mut g = be.lock().unwrap(); let j = jiffies(&mut **g); (j, Instant::now()) };
        let tempo = (j1 - j0) as f64 / 60.2 / (t1 - t0).as_secs_f64();
        eprintln!("{n} Bilder in {wall:.1} s = {:.1}/s, ausgelassen {dropped}, Tempo {:.0} %, Abruf {:.1} ms", n as f64 / wall, tempo * 100.0, stats.mean_grab_ms);
        assert!(n as f64 / wall >= 25.0, "Rate zu niedrig: {:.1}/s", n as f64 / wall);
        assert!(tempo > 0.8 && tempo < 1.25, "Emulationstempo {:.0} %", tempo * 100.0);
        // nach dem Beenden des Stroms läuft die Maschine weiter
        let mut g = be.lock().unwrap();
        g.cont().unwrap();
        assert_eq!(g.state(), RunState::Running);
        g.stop();
    }
    let _ = std::fs::remove_file(prg);
}
