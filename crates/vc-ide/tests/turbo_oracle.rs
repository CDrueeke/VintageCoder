// SPDX-License-Identifier: GPL-3.0-or-later
//! Orakel Turbo BASIC XL starten (E2): Projekt bauen → `build/<name>.atr` in atari800 booten (ohne eingebautes BASIC, XL-ROM) → Turbo BASIC XL lädt
//! `AUTORUN.BAS` selbst und führt es aus → gelesen wird der Bildschirmspeicher.
//! Nur mit `VC_EMU_TESTS=1` und der Diskette des Inhabers (`VC_TURBO_ATR` oder `assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr`, nicht im
//! Repository), atari800 und seinem XL-ROM in ~/.atari800.cfg; immer mit `--test-threads=1` (je Test startet ein Emulator). Die Diskette wird nur kopiert.
//! Altirra ist hier NICHT geprüft: zu prüfen in einer Windows-Sitzung (Handprüfliste).
use std::path::PathBuf;
use std::time::Duration;
use vc_emu::{Atari800, Backend};
use vc_ide::build::Ctx;
use vc_ide::projfile::{build_project, ProjectFile};

fn owner_disk() -> Option<PathBuf> {
    std::env::var("VC_TURBO_ATR").map(PathBuf::from).ok().or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().map(|a| a.join("assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr")).find(|p| p.exists()))
}
fn enabled() -> bool { std::env::var("VC_EMU_TESTS").is_ok() }

fn screen_lines(mem: &[u8]) -> Vec<String> {
    mem.chunks(40).map(|l| l.iter().map(|&b| match b & 0x7F { c @ 0x00..=0x3F => (c + 0x20) as char, c @ 0x40..=0x5F => (c - 0x40) as char, c @ 0x60..=0x7F => c as char, _ => '?' }).collect::<String>().trim_end().to_string()).collect()
}

/// Bootet die Diskette in atari800 und wartet, bis der Tastaturtreiber nach einem Zeichen fragt (Programmende, `READY`); liefert die Bildschirmzeilen
fn boot_and_read(atr: &std::path::Path) -> Option<Vec<String>> {
    let mut a = Atari800::launch(atr.to_str().unwrap(), None).expect("atari800 starten");
    let t = a.read_mem(0xE424, 2).unwrap();
    let kget = (t[0] as u16 | (t[1] as u16) << 8).wrapping_add(1);       // Einsprung „Zeichen lesen“ des K:-Treibers
    let hit = a.run_until(kget, Duration::from_secs(90)).expect("run_until");
    let sav = a.read_mem(0x58, 2).unwrap();
    let lines = screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap());
    a.stop();
    hit.then(|| lines.into_iter().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
}

fn project(name: &str, source: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vc_turbo_oracle_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("prog.vcprj"), r#"{ "format": 1, "project": {"name": "Orakel"}, "targets": [{ "id": "atari8", "system": "atari8", "model": "800xl", "variant": "pal", "cpu": "sally" }],
  "sources": [{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["prog.tur"] }], "main": "prog.tur" }"#).unwrap();
    std::fs::write(root.join("prog.tur"), source).unwrap();
    root
}

fn build(root: &std::path::Path) -> serde_json::Value {
    let data = vc_ide::find_data_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap();
    build_project(&Ctx { root, data: &data }, &ProjectFile::load(&vc_ide::projfile::find(root).unwrap()).unwrap(), None)
}

#[test]
fn autorun_bas_laeuft_im_echten_turbo_basic_xl() {
    if !enabled() { return; }
    let Some(orig) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let root = project("klein", "PRINT \"AUTORUN OK\"\nEXEC hallo\nPRINT \"ENDE\"\nEND\nPROC hallo\n  PRINT \"AUS PROC\"\nENDPROC\n");
    vc_ide::turbo::provide(&root, &orig, &Default::default()).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    let lines = boot_and_read(&root.join(r["turbo"]["file"].as_str().unwrap())).expect("Programmende nicht erreicht");
    eprintln!("{lines:#?}");
    assert_eq!(lines.iter().map(String::as_str).collect::<Vec<_>>(), ["AUTORUN OK", "AUS PROC", "ENDE", "READY"], "Konzept Abschnitt 3, Lauf 2");
}

/// Anhang D des Konzepts: ein Programm von über 20 KB passt nur auf die verkleinerte Kopie und läuft dort
#[test]
fn grosses_programm_laeuft_auf_der_verkleinerten_kopie() {
    if !enabled() { return; }
    let Some(orig) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let mut src = String::new();
    for i in 0..260 { src.push_str(&format!("x={}\n", (0..9).map(|k| format!("{}.{:06}", i % 90 + k, i * 7919 + k * 31)).collect::<Vec<_>>().join("+"))); }
    src.push_str("PRINT \"GROSS OK\"\n");
    let root = project("gross", &src);
    vc_ide::turbo::provide(&root, &orig, &Default::default()).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], false, "auf dem Original (103 Sektoren frei) passt es nicht: {r}");
    assert!(r["turbo_error"].as_str().unwrap().contains("vcc turbo reduce"));
    vc_ide::turbo::reduce_project_copy(&root).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    assert!(r["turbo"]["size"].as_u64().unwrap() > 103 * 125, "Programm größer als der freie Platz des Originals");
    let lines = boot_and_read(&root.join(r["turbo"]["file"].as_str().unwrap())).expect("Programmende nicht erreicht");
    eprintln!("{lines:#?}");
    assert_eq!(lines.iter().map(String::as_str).collect::<Vec<_>>(), ["GROSS OK", "READY"]);
}

/// Beispiel `examples/atari/turbo_basic/balken.tur`: nach dem Start stehen 16 Balken in Rot, Grün, Blau im Wechsel auf dem Bildschirm (Grafikstufe 7,
/// 10 Punkte je Balken) und bleiben stehen, bis eine Taste gedrückt wird. Gelesen wird die erste Bildzeile.
#[test]
fn beispiel_balken_zeigt_farbige_balken() {
    if !enabled() { return; }
    let Some(orig) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let ex = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/atari/turbo_basic");
    let root = project("balken", &std::fs::read_to_string(ex.join("balken.tur")).unwrap());
    vc_ide::turbo::provide(&root, &orig, &Default::default()).unwrap();
    let r = build(&root);
    assert_eq!(r["ok"], true, "{r}");
    let mut a = Atari800::launch(root.join(r["turbo"]["file"].as_str().unwrap()).to_str().unwrap(), None).expect("atari800 starten");
    a.set_breakpoint(0xE45F).unwrap();                                    // SYSVBV: einmal je Bild; 1500 Bilder = 30 s (Boot, 16 Balken zu je 0,5 s, danach Warten auf eine Taste)
    for _ in 0..1500 { a.run_until(0xE45F, Duration::from_secs(5)).unwrap(); }
    let dl = a.read_mem(0x0230, 2).unwrap();
    let d = dl[0] as u16 | (dl[1] as u16) << 8;
    let lms = a.read_mem(d + 4, 2).unwrap();
    let row = a.read_mem(lms[0] as u16 | (lms[1] as u16) << 8, 40).unwrap();
    let colors = a.read_mem(0x02C4, 3).unwrap();
    a.stop();
    let pixels: Vec<u8> = row.iter().flat_map(|b| (0..4).map(move |k| (b >> (6 - 2 * k)) & 3)).collect();
    assert_eq!(pixels, (0..160).map(|x| (x / 10 % 3 + 1) as u8).collect::<Vec<u8>>(), "16 Balken in den Farben 1, 2, 3 im Wechsel");
    assert_eq!(colors, [0x46, 0xCE, 0x84], "Rot, Grün (hell), Blau (dunkel): Register 1 und 2 färben auch das Textfenster, die Schrift muss heller sein als der Hintergrund");
}
