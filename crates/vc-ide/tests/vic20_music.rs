// SPDX-License-Identifier: GPL-3.0-or-later
//! VC-20: Spielroutine für VIC-Songs (examples/vic20/music_demo). Das Beispiel wird gebaut (Medien werden dabei exportiert) und im 6502-Modell
//! bildweise abgespielt; geprüft werden die Schreibzugriffe auf $900A-$900E.
use std::path::{Path, PathBuf};
use vc_ide::build::{build, Ctx};
use vc_media::cpu6502::{Bus, Cpu};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn demo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/vic20/music_demo") }

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let n = e.file_name(); if n == "build" { continue; }
        if e.path().is_dir() { copy_dir(&e.path(), &to.join(&n)); } else { std::fs::copy(e.path(), to.join(&n)).unwrap(); }
    }
}

struct M { ram: Vec<u8>, log: Vec<(u16, u8)> }
impl Bus for M {
    fn read(&mut self, a: u16) -> u8 { self.ram[a as usize] }
    fn write(&mut self, a: u16, v: u8) { if (0x900A..=0x900E).contains(&a) { self.log.push((a, v)); } self.ram[a as usize] = v; }
}

fn label(vs: &str, name: &str) -> u16 {
    let l = vs.lines().find(|l| l.ends_with(&format!(" .{name}"))).unwrap_or_else(|| panic!("Label {name} fehlt"));
    u16::from_str_radix(&l[5..9], 16).unwrap()
}

/// Erwarteter Registerwert ($80 | N) nach f = Takt * m / (256 * (127 - N)) (in VICE gemessen), Ton auf den Umfang der Stimme gefaltet
fn expect(note: u32, m: f64) -> u8 {
    let clock = 1108405.0;
    let mut f = 440.0 * 2f64.powf((note as f64 - 58.0) / 12.0);
    while clock * m / (256.0 * f) > 126.5 { f *= 2.0; }
    while clock * m / (256.0 * f) < 0.5 { f /= 2.0; }
    let k = (clock * m / (256.0 * f)).round().clamp(1.0, 127.0) as u8;
    0x80 | (127 - k)
}

#[test]
fn vic_player_plays_the_tune() {
    let dir = std::env::temp_dir().join(format!("vc-vic20music-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&demo(), &dir);
    let out = build(&Ctx { root: &dir, data: &data() }, "music_demo.asm", Some("vic20")).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    let prg = std::fs::read(dir.join(out["artifact"].as_str().unwrap())).unwrap();
    let vs = std::fs::read_to_string(dir.join("build/music_demo.vs")).unwrap();
    let (start, lp, play) = (label(&vs, "start"), label(&vs, "loop"), label(&vs, "music_play"));

    let mut m = M { ram: vec![0; 65536], log: vec![] };
    let load = prg[0] as usize | (prg[1] as usize) << 8;
    m.ram[load..load + prg.len() - 2].copy_from_slice(&prg[2..]);
    m.ram[0x900E] = 0xA0;   // Hilfsfarbe muss erhalten bleiben
    let mut cpu = Cpu::new(); cpu.pc = start; cpu.sp = 0xF0;
    for _ in 0..100_000 { if cpu.pc == lp { break; } cpu.step(&mut m).unwrap(); }
    assert_eq!(cpu.pc, lp, "Initialisierung endet in der Schleife");
    assert_eq!(m.ram[0x900E], 0xAF, "Lautstärke 15, Hilfsfarbe bleibt");

    // Je Bild music_play aufrufen (Rücksprung in eine Falle bei $0200) und die Register nach dem Bild festhalten
    let mut frames: Vec<[u8; 4]> = vec![];
    let mut regs = [0u8; 4];
    for _ in 0..200 {
        cpu.pc = play; cpu.sp = 0xF0; cpu.push(&mut m, 0x01); cpu.push(&mut m, 0xFF);
        m.log.clear();
        for _ in 0..100_000 { if cpu.pc == 0x0200 { break; } cpu.step(&mut m).unwrap(); }
        assert_eq!(cpu.pc, 0x0200, "music_play kehrt zurück");
        for (a, v) in &m.log { if *a < 0x900E { regs[(*a - 0x900A) as usize] = *v; } }
        frames.push(regs);
    }
    // Zeile 0 (Bild 0): Bass C-3 (Note 37), Alt E-4 (Note 53), Sopran E-5 (Note 65) laufen, das Rauschen noch nicht
    assert_eq!(frames[0][0], expect(37, 1.0), "Bass C-3");
    assert_eq!(frames[0][1], expect(53, 2.0), "Alt E-4");
    assert_eq!(frames[0][2], expect(65, 4.0), "Sopran E-5");
    assert_eq!(frames[0][3], 0, "Rauschen noch aus");
    // Die Hülle des Instruments 0 hat 8 Schritte mit Lautstärke, danach ist die Stimme still (Bild 8)
    assert_ne!(frames[7][2], 0);
    assert_eq!(frames[8][2], 0, "Sopran nach der Hülle aus");
    // Zeile 1 beginnt nach 6 Bildern; Zeile 4 (Bild 24): Sopran G-5 (Note 68) spielt neu
    assert_eq!(frames[24][2], expect(68, 4.0), "Sopran G-5 in Zeile 4");
    // Rauschen: Zeile 2 (Bild 12) mit Instrument 2 (zwei Schritte)
    assert_ne!(frames[12][3], 0); assert_ne!(frames[13][3], 0); assert_eq!(frames[14][3], 0);
    // Nach 32 Zeilen (192 Bilder) beginnt der Song von vorn
    assert_eq!(frames[192 + 0][0], expect(37, 1.0), "Wiederholung");
    if std::env::var("VC_KEEP").is_err() { std::fs::remove_dir_all(&dir).ok(); } else { eprintln!("PRG: {}", dir.join(out["artifact"].as_str().unwrap()).display()); }
}
