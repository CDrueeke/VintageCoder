// SPDX-License-Identifier: GPL-3.0-or-later
//! Notenkonstanten und Frequenztabellen (`data/sys/<system>/notes.inc`, erzeugt von `tools/gen_notes.py`):
//! jede Datei baut in PAL und NTSC fehlerfrei, die Konstanten stimmen mit dem Tracker überein, und Stichproben der Tabellen
//! (A4 = 440 Hz, C4, tiefste/höchste Note) entsprechen den physikalischen Formeln.
use std::collections::BTreeMap;
use std::path::PathBuf;
use vc_asm::{assemble, Options, Output, Policy};
use vc_target::load_cpu;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// (Systemordner, CPU-Kennung, Origin)
const SYSTEMS: [(&str, &str, u16); 6] = [("c64", "6510", 0x0801), ("c128", "8502", 0x1c01), ("mega65", "45gs02", 0x2001), ("plus4", "7501", 0x1001), ("vic20", "nmos6502", 0x1201), ("atari8bit", "sally", 0x2000)];

fn build(sys: &str, ntsc: bool, extra: &str) -> Output {
    let (_, cpu, org) = *SYSTEMS.iter().find(|s| s.0 == sys).unwrap();
    let cpu = load_cpu(&data(), cpu).unwrap();
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    if ntsc { o.defines.push(("NTSC".into(), 1)); }
    let src = format!(".org ${org:04x}\n#include <{sys}/notes.inc>\n{extra}");
    assemble(&src, &cpu, &o).unwrap_or_else(|e| panic!("{sys} (NTSC={ntsc}): {e:?}"))
}

/// Byte an der Adresse `addr` aus den Segmenten.
fn byte(out: &Output, addr: i64) -> u8 {
    for s in &out.segments {
        let a = addr - s.start as i64;
        if a >= 0 && (a as usize) < s.bytes.len() { return s.bytes[a as usize]; }
    }
    panic!("Adresse ${addr:04x} nicht belegt");
}
fn tab(out: &Output, label: &str, note: i64) -> u8 { byte(out, out.symbols[label] + note) }
fn sym(out: &Output, name: &str) -> i64 { *out.symbols.get(name).unwrap_or_else(|| panic!("Symbol {name} fehlt")) }
fn near(a: i64, b: i64) -> bool { (a - b).abs() <= 1 }

#[test]
fn every_file_builds_pal_and_ntsc() {
    for (sys, _, _) in SYSTEMS {
        for ntsc in [false, true] { build(sys, ntsc, ""); }
        // nur Konstanten: keine Tabelle
        let cpu = load_cpu(&data(), SYSTEMS.iter().find(|s| s.0 == sys).unwrap().1).unwrap();
        let mut o = Options::new("t.asm", Policy::Allow);
        o.include_dirs.push(data().join("sys"));
        o.defines.push(("NOTES_NO_TABLE".into(), 1));
        let out = assemble(&format!(".org $2000\n#include <{sys}/notes.inc>\n nop\n#include <{sys}/notes.inc>\n"), &cpu, &o).unwrap_or_else(|e| panic!("{sys}: {e:?}"));
        assert_eq!(out.segments[0].bytes, vec![0xEA], "{sys}: nur Konstanten, doppeltes Einbinden harmlos");
    }
}

#[test]
fn constants_follow_the_tracker_numbering() {
    for (sys, _, _) in SYSTEMS {
        let out = build(sys, false, "");
        let want: BTreeMap<&str, i64> = [("NOTE_C0", 1), ("NOTE_CS0", 2), ("NOTE_DB0", 2), ("NOTE_B0", 12), ("NOTE_C1", 13), ("NOTE_C4", 49), ("NOTE_CS4", 50), ("NOTE_DB4", 50),
            ("NOTE_A4", 58), ("NOTE_AS4", 59), ("NOTE_BB4", 59), ("NOTE_B4", 60), ("NOTE_AS7", 95), ("NOTE_B7", 96), ("NOTE_HOLD", 0), ("NOTE_REST", 96), ("NOTE_FIRST", 1), ("NOTE_LAST", 96)].into_iter().collect();
        for (k, v) in want { assert_eq!(sym(&out, k), v, "{sys}: {k}"); }
    }
}

#[test]
fn sid_tables_c64_c128_mega65() {
    for sys in ["c64", "c128", "mega65"] {
        for (ntsc, clock) in [(false, 985248.0f64), (true, 1022727.0)] {
            let out = build(sys, ntsc, "");
            let fn_of = |n: i64| (440.0 * 2f64.powf((n - 58) as f64 / 12.0) * 16777216.0 / clock).round().min(65535.0) as i64;
            let get = |n: i64| tab(&out, "note_sid_lo", n) as i64 | (tab(&out, "note_sid_hi", n) as i64) << 8;
            for n in [1, 49, 58, 95] { assert!(near(get(n), fn_of(n)), "{sys} ntsc={ntsc} Note {n}: {} statt {}", get(n), fn_of(n)); }
            // bekannte Werte: A4 PAL = $1CD6 (440 Hz * 2^24 / 985248 = 7493,3)
            if !ntsc { assert_eq!(get(58), 7493); assert_eq!(get(49), 4455); }
            // Obergrenze: PAL kappt B7 bei 65535, NTSC (höherer Takt, kleinerer Zähler) passt gerade noch
            let hi = sym(&out, "NOTE_TABLE_HIGHEST");
            assert_eq!(hi, if ntsc { 96 } else { 95 }, "{sys} ntsc={ntsc}");
            assert!(near(get(hi), fn_of(hi)) && get(hi) < 65535 && get(hi) > 60000);
            if !ntsc { assert_eq!(get(96), 65535, "B7 liegt über dem Zählerumfang und ist gekappt"); }
            // Konstante und Tabellenwert: Oktave höher = doppelter Wert
            assert!(near(get(sym(&out, "NOTE_A4") + 12), 2 * get(sym(&out, "NOTE_A4"))));
        }
    }
}

#[test]
fn ted_table_plus4() {
    for (ntsc, clock) in [(false, 886724.0f64), (true, 894886.0)] {
        let out = build("plus4", ntsc, "");
        let get = |n: i64| tab(&out, "note_ted_lo", n) as i64 | (tab(&out, "note_ted_hi", n) as i64) << 8;
        let f = |n: i64| 440.0 * 2f64.powf((n - 58) as f64 / 12.0);
        let n_of = |n: i64| (1024.0 - clock / 8.0 / f(n)).round() as i64;
        for n in [sym(&out, "NOTE_C3"), sym(&out, "NOTE_C4"), sym(&out, "NOTE_A4"), 96] { assert!(near(get(n), n_of(n)), "ntsc={ntsc} Note {n}"); }
        if !ntsc { assert_eq!(get(58), 772, "A4 PAL: N = 1024 - 110840,5 / 440"); }
        let lowest = sym(&out, "NOTE_TABLE_LOWEST");
        assert!(near(get(lowest), n_of(lowest)) && lowest > 1);
        assert!(get(lowest - 1) > 0 && get(1) > 0, "gefaltete Töne bleiben gültig");
    }
}

#[test]
fn vic20_tables() {
    for ntsc in [false, true] {
        let out = build("vic20", ntsc, "");
        let clock: f64 = if ntsc { 1022727.0 } else { 1108405.0 };
        for (label, m) in [("note_vic_sopr", 4.0f64), ("note_vic_alto", 2.0), ("note_vic_bass", 1.0)] {
            // A4 liegt bei allen drei Stimmen im Bereich: N = 127 - round(Takt * m / (256 * 440))
            let k = (clock * m / (256.0 * 440.0)).round() as u8;
            assert_eq!(tab(&out, label, 58), 0x80 | (127 - k), "{label} ntsc={ntsc}");
            assert!(tab(&out, label, 1) >= 0x80 && tab(&out, label, 96) >= 0x80);
        }
    }
}

#[test]
fn pokey_tables_atari() {
    for (ntsc, clock) in [(false, 1773447.0f64), (true, 1789790.0)] {
        let out = build("atari8bit", ntsc, "");
        let f = |n: i64| 440.0 * 2f64.powf((n - 58) as f64 / 12.0);
        let n8 = |n: i64| (clock / 28.0 / (2.0 * f(n))).round() as i64 - 1;
        let n16 = |n: i64| (clock / (2.0 * f(n))).round() as i64 - 7;
        for n in [sym(&out, "NOTE_C3"), sym(&out, "NOTE_C4"), sym(&out, "NOTE_A4"), 96] { assert_eq!(tab(&out, "note_pokey8", n) as i64, n8(n), "8 Bit ntsc={ntsc} Note {n}"); }
        let low = sym(&out, "NOTE_POKEY8_LOWEST");
        assert!(n8(low) <= 255 && n8(low - 1) > 255, "Untergrenze 8 Bit");
        assert_eq!(tab(&out, "note_pokey8", low - 1), 0xff);
        for n in [1, 49, 58, 96] {
            let v = tab(&out, "note_pokey16_lo", n) as i64 | (tab(&out, "note_pokey16_hi", n) as i64) << 8;
            assert!(near(v, n16(n)), "16 Bit ntsc={ntsc} Note {n}: {v} statt {}", n16(n));
        }
    }
}

#[test]
fn constants_usable_in_code() {
    // Tabellenindex mit Konstante und Arithmetik (Quinte, Oktave)
    let out = build("c64", false, "start: ldx #NOTE_C4 + 7\n lda note_sid_lo,x\n ldy #NOTE_A4\n");
    assert_eq!(byte(&out, sym(&out, "start")), 0xA2);
    assert_eq!(byte(&out, sym(&out, "start") + 1), 56);
}

#[test]
fn checked_in_files_match_generator() {
    // `tools/gen_notes.py --check`: Abweichung von den eingecheckten Dateien = Fehler (übersprungen, wenn kein python3 da ist)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(out) = std::process::Command::new("python3").arg(root.join("tools/gen_notes.py")).arg("--check").output() else { eprintln!("python3 fehlt – übersprungen"); return };
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}
