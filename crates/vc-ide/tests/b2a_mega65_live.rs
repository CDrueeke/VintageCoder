// SPDX-License-Identifier: GPL-3.0-or-later
//! basic2asm für das MEGA65 im echten Xemu (öffnet ein Emulatorfenster). NUR MIT VC_EMU_TESTS=1, einzeln (`--test-threads=1`).
//!
//! STAND: unter Windows in Xemu bestanden (beide Tests). Auf dem Mac startet Xemu mit `-prg` unzuverlässig (beendet sich teils von selbst) und das Laden über den
//! Monitor stürzte beim ersten Versuch mit „Broken pipe“ ab; dieser Weg (nicht Windows) ist weiter ungeprüft. Windows: Xemu startet und führt das Programm selbst
//! aus (`-prg … -prgmode 65`), der Test wartet, hält an und liest den Bildschirm ($0800, 80 Spalten). Ohne Audiogerät `SDL_AUDIODRIVER=dummy` setzen, sonst beendet sich Xemu.
//!
//! Tests:
//!  * `probe_zeropage`: misst, welche Nullseitenbytes KERNAL und Interrupt im Lauf ändern (Block der Laufzeit: `data/sys/mega65/b2a/rt_zp.asm`, $40-$53).
//!  * `programs_run_on_mega65`: übersetzte BASIC-Programme (Ganzzahl, Zeichenketten) laufen und schreiben den erwarteten Text auf den Bildschirm.
use std::path::PathBuf;
use std::time::Duration;
use vc_emu::{Backend, Mega65};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
/// Nur unter Windows (dort läuft Xemu mit `-prg` verlässlich); auf dem Mac bleibt der Startweg ungeklärt, die Tests würden dort scheitern. `VC_M65_LIVE=1` erzwingt sie überall.
fn enabled() -> bool { std::env::var("VC_EMU_TESTS").is_ok() && (cfg!(windows) || std::env::var("VC_M65_LIVE").is_ok()) }

fn assemble(asm: &str) -> vc_asm::Output {
    let cpu = vc_target::load_cpu(&data(), "45gs02").unwrap();
    let mut ao = vc_asm::Options::new("p.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    vc_asm::assemble(asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")))
}

/// Startet Xemu mit dem Programm. Windows: Xemu lädt und startet selbst (`-prg … -prgmode 65`, wie `launch.rs`); sonst: über den Monitor laden und am SYS-Ziel starten.
fn start(prg: &[u8], entry: u16, name: &str) -> Mega65 {
    let sd = Mega65::xemu_sd_dir().expect("MEGA65.ROM nicht gefunden (VC_XEMU_SD setzen)");
    let rom = Mega65::rom_in(&sd).unwrap();
    let dir = std::env::temp_dir().join("vc_m65_b2a"); std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(name);
    std::fs::write(&f, prg).unwrap();
    #[cfg(windows)]
    {
        let _ = entry;
        Mega65::launch_xemu_rom(None, &rom, &["-prg".into(), f.to_string_lossy().into_owned(), "-prgmode".into(), "65".into()]).expect("Xemu starten")
    }
    #[cfg(not(windows))]
    {
        let _ = &f;
        let mut m = Mega65::launch_xemu_rom(None, &rom, &[]).expect("Xemu starten");
        std::thread::sleep(Duration::from_secs(12));   // Hypervisor und BASIC fertig starten lassen
        m.load_prg(prg, entry).expect("laden");
        m.cont().expect("fortsetzen");
        m
    }
}

/// Wartet, bis das Programm das Ende erreicht (Haltepunkt) oder die Zeit abläuft
fn wait_for(m: &mut Mega65, addr: u16, secs: u64) -> bool {
    #[cfg(windows)] { let _ = m.pause(); }
    m.run_until(addr, Duration::from_secs(secs)).unwrap_or(false)
}

/// Windows: Xemu startet das Programm selbst und es ist beim Anhalten meist schon zu Ende (zurück bei READY). Daher: warten, anhalten, Bildschirm lesen;
/// enthält er `want` noch nicht, weiterlaufen lassen und erneut lesen (bis `tries` mal).
#[cfg(windows)]
fn screen_when(m: &mut Mega65, want: &str, tries: u32) -> Vec<String> {
    let mut lines = vec![];
    std::thread::sleep(Duration::from_secs(14));
    for _ in 0..tries {
        let _ = m.pause();
        let scr = m.read_mem(0x0800, 2000).unwrap();
        lines = screen_text(&scr, 80).into_iter().filter(|l| !l.is_empty()).collect();
        if lines.join("
").contains(want) { break; }
        let _ = m.cont();
        std::thread::sleep(Duration::from_secs(4));
    }
    lines
}

#[test]
fn probe_zeropage() {
    if !enabled() { return; }
    let asm = "
.target mega65
.cpu 45gs02 { illegals = deny }
#include <mega65/kernal.asm>
.org $2001
.basicstub start
start:  ldx #0
.a:     lda $00,x
        sta $1E00,x
        inx
        bne .a
        lda #65
        jsr CHROUT
        lda #13
        jsr CHROUT
        jsr GETIN
        sec
        jsr PLOT
        jsr RDTIM
        lda #0
        ldx #0
        ldy #0
        jsr SETTIM
        ldy #0
        ldx #0
.w:     inx
        bne .w
        iny
        bne .w
        ldx #0
.b:     lda $00,x
        sta $1F00,x
        inx
        bne .b
done:   jmp done
";
    let out = assemble(asm);
    let entry = out.symbols["start"] as u16;
    let done = out.symbols["done"] as u16;
    let mut m = start(&vc_asm::to_prg(&out).unwrap(), entry, "PROBE.PRG");
    assert!(wait_for(&mut m, done, 60), "Programmende nicht erreicht");
    let a = m.read_mem(0x1E00, 256).unwrap();
    let b = m.read_mem(0x1F00, 256).unwrap();
    let diff: Vec<String> = (0..256).filter(|&i| a[i] != b[i]).map(|i| format!("${i:02X}:{:02X}->{:02X}", a[i], b[i])).collect();
    eprintln!("Nullseite geändert: {}", diff.join(" "));
    // Der Block der Laufzeit ($40-$53) darf sich nicht geändert haben
    let hit: Vec<&String> = diff.iter().filter(|d| { let i = usize::from_str_radix(&d[1..3], 16).unwrap(); (0x40..0x54).contains(&i) }).collect();
    assert!(hit.is_empty(), "KERNAL/Interrupt ändern den Block $40-$53: {hit:?}");
    m.stop();
}

/// Bildschirmcodes → Text (Großbuchstaben, Ziffern und Satzzeichen); Leerzeichen am Zeilenende entfallen
fn screen_text(mem: &[u8], width: usize) -> Vec<String> {
    mem.chunks(width).map(|l| l.iter().map(|&b| match b & 0x7F { c @ 0x01..=0x1A => (c + 0x40) as char, c @ 0x41..=0x5A => c as char, c @ 0x20..=0x3F => c as char, 0 => '@', _ => '?' }).collect::<String>().trim_end().to_string()).collect()
}

const PROGRAMS: &[(&str, &str, &str)] = &[
    ("ganz", "10 X=5:Y=7\n20 PRINT X*Y;X+Y\n30 FOR I=1 TO 3:PRINT I;:NEXT I\n40 PRINT\n50 PRINT \"HALLO\"\n", "35 12\n1 2 3\nHALLO"),
    ("text", "10 A$=\"ABC\":B$=\"DEF\":C$=A$+B$\n20 PRINT C$;LEN(C$)\n30 PRINT LEFT$(C$,2);MID$(C$,3,2);RIGHT$(C$,1)\n", "ABCDEF 6\nABCDF"),
];

#[test]
fn programs_run_on_mega65() {
    if !enabled() { return; }
    use vc_basic::b2a::{convert, B2aOptions};
    for (name, src, want) in PROGRAMS {
        let d = vc_basic::Dialect::load(&data(), "basic65").unwrap();
        let mut o = B2aOptions::new("t.bas"); o.target = "mega65".into();
        let c = convert(src, &d, &o).unwrap_or_else(|e| panic!("{e:?}"));
        let out = assemble(&c.asm);
        let entry = out.symbols["b2a_start"] as u16;
        let end = out.symbols["rt_end"] as u16;
        let mut m = start(&vc_asm::to_prg(&out).unwrap(), entry, "T.PRG");
        #[cfg(windows)]
        let (hit, lines) = { let l = screen_when(&mut m, want.lines().last().unwrap_or(""), 5); let ok = l.iter().any(|x| x.contains("READY")); (ok, l) };
        #[cfg(not(windows))]
        let (hit, lines) = {
            let hit = wait_for(&mut m, end, 60);
            // Bildschirm: 80 Spalten ab $0800 (C65-Modus, VIC-IV-Text); bei 40 Spalten die Breite anpassen
            let scr = m.read_mem(0x0800, 2000).unwrap();
            (hit, screen_text(&scr, 80).into_iter().filter(|l| !l.is_empty()).collect::<Vec<String>>())
        };
        #[cfg(windows)] { let _ = end; }
        if let Ok(scr) = m.read_mem(0x0800, 2000) { let raw: Vec<String> = scr.chunks(80).filter(|c| c.iter().any(|&b| b != 0x20)).map(|c| c.iter().take(24).map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")).collect(); eprintln!("{name} roh: {raw:?}"); }
        m.stop();
        assert!(hit, "{name}: Programmende nicht erreicht; Bildschirm: {lines:?}");
        // BASIC setzt vor positive Zahlen ein Leerzeichen („ 35  12“): Leerzeichenfolgen vergleichen wir als eines.
        // Großbuchstaben des Quelltexts liegen als PETSCII $C1-$DA vor, auf dem Bildschirm als $41-$5A.
        let norm = |t: &str| t.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join("\n");
        let (text, want) = (norm(&lines.join("\n")), norm(want));
        assert!(text.contains(&want), "{name}: erwartet {want:?}, Bildschirm: {lines:?}");
    }
}
