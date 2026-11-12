// SPDX-License-Identifier: GPL-3.0-or-later
//! Strukturiertes BASIC (do/loop/exit, proc/call, elseif) auf C128 (BASIC 7) und Plus/4 (BASIC 3.5) in echtem VICE:
//! nur mit VC_EMU_TESTS=1 (öffnet Emulatorfenster).
use std::path::PathBuf;
use std::time::Duration;
use vc_basic::{compile, Dialect, Options};
use vc_emu::{Backend, Vice};

/// Ordner mit den (nicht im Repository liegenden) `.sbas`-Beispielen: Umgebungsvariable `VC_SBAS_EXAMPLES`; ohne sie überspringen sich die Tests
fn sbas_dir() -> PathBuf { std::env::var("VC_SBAS_EXAMPLES").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt")) }

const SRC: &str = "total = 0
n = 0
do
  n = n + 1
  call add(n)
  if n >= 10 then
    exit
  endif
loop
poke 12032, total - 50
x = 3
if x = 1 then
  poke 12033, 1
elseif x = 3 then
  poke 12033, 33
else
  poke 12033, 99
endif
k = 0
repeat
  k = k + 1
  if k = 4 then
    exit
  endif
until k = 100
poke 12034, k
@done:
goto @done
proc add(v)
  total = total + v
endproc
";

fn run(system: &str, dialect: &str) {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = Dialect::load(&data, dialect).unwrap();
    let c = compile(SRC, &d, &Options::new("t.sbas")).unwrap_or_else(|e| panic!("{e:?}"));
    let dir = std::env::temp_dir().join(format!("vc_basic_live_{system}"));
    std::fs::create_dir_all(&dir).unwrap();
    let prg = dir.join("t.prg");
    std::fs::write(&prg, &c.prg).unwrap();
    let mut v = Vice::launch_system(system, prg.to_str().unwrap(), None).expect("VICE starten");
    std::thread::sleep(Duration::from_secs(25));
    v.pause().unwrap();
    let got = v.read_mem(12032, 3).unwrap();
    if std::env::var("VC_DUMP").is_ok() {
        let scr = v.read_mem(if system == "xplus4" { 0x0C00 } else { 0x0400 }, 400).unwrap();
        let txt: String = scr.chunks(40).map(|l| l.iter().map(|b| { let c = b & 0x7f; if c < 32 { (c + 64) as char } else { c as char } }).collect::<String>() + "\n").collect();
        eprintln!("{system} Bildschirm:\n{txt}");
    }
    v.stop();
    assert_eq!(got, vec![5, 33, 4], "{system}: Ergebnis im Speicher");
}

#[test]
fn structured_basic_runs_on_c128_and_plus4() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    run("x128", "basic7");
    run("xplus4", "basic35");
}

/// Sprite-Schnittstelle (examples/multi/sprite_demo/hal) auf C64, C128 und Plus/4 in echtem VICE.
#[test]
fn sprite_interface_on_c64_c128_plus4() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = sbas_dir().join("examples/multi/sprite_demo");
    if !root.is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    // Formdaten aus dem Export: 64 Bytes
    let exp = std::fs::read_to_string(root.join("build/export/ship.data.sbas")).expect("vorher exportieren: vcc media export graphics/ship.vcsprite --format sbas");
    let shape: Vec<u8> = exp.lines().filter(|l| l.starts_with("data ")).flat_map(|l| l[5..].split(',').map(|v| v.trim().parse::<u8>().unwrap()).collect::<Vec<_>>()).collect();
    assert_eq!(shape.len(), 64);
    let ones: u32 = shape.iter().map(|b| b.count_ones()).sum();
    let src = "include \"hal/sprite_{system}.sbas\"\ninclude \"build/export/ship.data.sbas\"\ncall sprite_init\nrestore\ncall sprite_load(0)\ncall sprite_color(0, 7)\ncall sprite_move(0, 100, 80)\ncall sprite_show(0, 1)\ncall sprite_move(0, 150, 90)\n@e:\ngoto @e\n";
    for (system, dialect) in [("x64sc", "cbm_v2"), ("x128", "basic7"), ("xplus4", "basic35")] {
        let sys = match system { "x64sc" => "c64", "x128" => "c128", _ => "plus4" };
        let d = Dialect::load(&data, dialect).unwrap();
        let mut o = Options::new(root.join("main.sbas").to_string_lossy());
        o.include_dirs.push(root.clone());
        o.system = Some(sys.into());
        let c = compile(src, &d, &o).unwrap_or_else(|e| panic!("{sys}: {e:?}"));
        let f = std::env::temp_dir().join(format!("vc_sprite_{sys}.prg"));
        std::fs::write(&f, &c.prg).unwrap();
        let mut v = Vice::launch_system(system, f.to_str().unwrap(), None).expect("VICE starten");
        std::thread::sleep(Duration::from_secs(30));
        v.pause().unwrap();
        match sys {
            "plus4" => {
                let bm = v.read_mem(0x2000, 8000).unwrap();
                let set: u32 = bm.iter().map(|b| b.count_ones()).sum();
                // genau eine Kopie sichtbar (altes Bild wurde im XOR-Modus gelöscht)
                assert_eq!(set, ones, "plus4: gesetzte Pixel im Bitmap");
                // an der neuen Stelle (x 150 = Spalte 18, y 90 = Zelle 11, Zeile 2)
                let at = (90 / 8) * 320 + (150 / 8) * 8;
                assert!(bm[at..at + 40].iter().any(|b| *b != 0), "plus4: kein Bild bei (150, 90)");
            }
            _ => {
                let io = v.read_space("io", 0xD000, 48).unwrap();
                if std::env::var("VC_DUMP").is_ok() {
                    let scr = v.read_mem(0x0400, 1000).unwrap();
                    let txt: String = scr.chunks(40).map(|l| l.iter().map(|b| { let c = b & 0x7f; if c < 32 { (c + 64) as char } else { c as char } }).collect::<String>().trim_end().to_string() + "|").collect();
                    eprintln!("DUMP {sys}: io={:02X?} schirm={txt}", &io[..0x28]);
                }
                assert_eq!(io[0x15] & 1, 1, "{sys}: Sprite 0 an");
                assert_eq!(io[0x27] & 15, 7, "{sys}: Farbe");
                assert_eq!(io[1], 140, "{sys}: y = 90 + 50");
                assert_eq!(io[0], 174, "{sys}: x = 150 + 24");
                if sys == "c64" {
                    assert_eq!(v.read_mem(0x3E00, 64).unwrap(), shape, "c64: Form im Speicher");
                    assert_eq!(v.read_mem(2040, 1).unwrap()[0], 248, "c64: Sprite-Zeiger");
                }
            }
        }
        v.stop();
    }
}

/// Musik aus BASIC (Maschinencode hinter dem BASIC-Text, Raster-IRQ) auf C64 und C128 in echtem VICE.
#[test]
fn music_from_basic_on_c64_and_c128() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = sbas_dir().join("examples/multi/music_demo");
    if !root.is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let src = std::fs::read_to_string(root.join("main.sbas")).unwrap();
    for (vice, sys, dialect) in [("x64sc", "c64", "cbm_v2"), ("x128", "c128", "basic7")] {
        let d = Dialect::load(&data, dialect).unwrap();
        let tgt = vc_target::load_target(&data, sys).unwrap();
        let mut o = Options::new(root.join("main.sbas").to_string_lossy());
        o.include_dirs.push(root.clone());
        o.system = Some(sys.into());
        o.load_addr = tgt.basic_load_addr;
        let built = vc_ide::blob::compile_with_blobs(&src, &d, &o, &root, &data, &tgt).unwrap_or_else(|e| panic!("{sys}: {e:?}"));
        let b = &built.blobs[0];
        let f = std::env::temp_dir().join(format!("vc_music_{sys}.prg"));
        std::fs::write(&f, &built.prg).unwrap();
        let mut v = Vice::launch_system(vice, f.to_str().unwrap(), None).expect("VICE starten");
        std::thread::sleep(Duration::from_secs(30));
        v.pause().unwrap();
        let vec = v.read_mem(0x0314, 2).unwrap();
        let irq = vec[0] as usize | (vec[1] as usize) << 8;
        assert!(irq >= b.addr as usize && irq < b.addr as usize + b.size, "{sys}: IRQ-Vektor ${irq:04X} nicht im Maschinencode ${:04X}+{}", b.addr, b.size);
        let sid = v.read_space("io", 0xD400, 25).unwrap();
        assert_eq!(sid[24] & 15, 15, "{sys}: Lautstärke");
        let voices: u32 = [0usize, 7, 14].iter().map(|o| sid[*o] as u32 | (sid[o + 1] as u32) << 8).sum();
        assert!(voices > 0, "{sys}: keine Stimme klingt (SID: {sid:02X?})");
        // BASIC läuft weiter: der Zähler steht auf dem Bildschirm
        let scr = v.read_mem(0x0400, 1000).unwrap();
        let txt: String = scr.iter().map(|b| { let c = b & 0x7f; if c < 32 { (c + 64) as char } else { c as char } }).collect();
        assert!(txt.chars().any(|c| c.is_ascii_digit()) && !txt.contains("ERROR"), "{sys}: Bildschirm {txt:?}");
        // und später ändert sich die Stimme weiter (der Song spielt)
        let before = sid.clone();
        v.cont().unwrap();
        std::thread::sleep(Duration::from_secs(4));
        v.pause().unwrap();
        let after = v.read_space("io", 0xD400, 25).unwrap();
        v.stop();
        assert_ne!(before[..21], after[..21], "{sys}: SID-Register ändern sich nicht (Player läuft nicht)");
    }
}

/// Musik aus BASIC auf dem Plus/4 (TED, 2 Stimmen) in echtem VICE.
#[test]
fn music_from_basic_on_plus4() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = sbas_dir().join("examples/multi/music_demo");
    if !root.is_dir() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let src = std::fs::read_to_string(root.join("main.sbas")).unwrap();
    let d = Dialect::load(&data, "basic35").unwrap();
    let tgt = vc_target::load_target(&data, "plus4").unwrap();
    let mut o = Options::new(root.join("main.sbas").to_string_lossy());
    o.include_dirs.push(root.clone());
    o.system = Some("plus4".into());
    o.load_addr = tgt.basic_load_addr;
    let built = vc_ide::blob::compile_with_blobs(&src, &d, &o, &root, &data, &tgt).unwrap_or_else(|e| panic!("{e:?}"));
    let b = &built.blobs[0];
    let frames_at = b.syms.iter().find(|(n, _)| n == "mus_frames").unwrap().1 as u16;
    let f = std::env::temp_dir().join("vc_music_plus4.prg");
    std::fs::write(&f, &built.prg).unwrap();
    let mut v = Vice::launch_system("xplus4", f.to_str().unwrap(), None).expect("VICE starten");
    std::thread::sleep(Duration::from_secs(30));
    v.pause().unwrap();
    let vec = v.read_mem(0x0314, 2).unwrap();
    let irq = vec[0] as usize | (vec[1] as usize) << 8;
    assert!(irq >= b.addr as usize && irq < b.addr as usize + b.size, "IRQ-Vektor ${irq:04X} nicht im Maschinencode ${:04X}+{}", b.addr, b.size);
    let fr = |v: &mut Vice| { let m = v.read_mem(frames_at, 2).unwrap(); m[0] as u32 | (m[1] as u32) << 8 };
    let f0 = fr(&mut v);
    // mehrfach kurz laufen lassen und die TED-Register lesen: es müssen Töne mit Lautstärke und wechselnden Frequenzen vorkommen
    let (mut freqs, mut loud) = (std::collections::BTreeSet::new(), 0);
    for _ in 0..16 {
        v.cont().unwrap();
        std::thread::sleep(Duration::from_millis(350));
        v.pause().unwrap();
        let r = v.read_mem(0xFF0E, 5).unwrap();        // FF0E V1lo, FF0F V2lo, FF10 V2hi, FF11 ctrl, FF12 V1hi
        let ctrl = r[3];
        if ctrl & 0x0F > 0 && ctrl & 0x30 != 0 { loud += 1; }
        freqs.insert((r[0], r[4] & 3));
    }
    let fa = fr(&mut v);
    v.cont().unwrap();
    let t = std::time::Instant::now();
    std::thread::sleep(Duration::from_secs(6));
    v.pause().unwrap();
    let secs = t.elapsed().as_secs_f64();
    let fb = fr(&mut v);
    let rate = fb.wrapping_sub(fa) as f64 / secs;
    eprintln!("PLUS4 Takt: {rate:.1} Aufrufe/s");
    assert!((45.0..=65.0).contains(&rate), "plus4: Player-Takt {rate:.1}/s statt ein Aufruf je Bild (50/60 Hz)");
    let f1 = fr(&mut v);
    v.stop();
    assert!(loud >= 3, "plus4: zu selten laut ({loud} von 16)");
    assert!(freqs.len() >= 3, "plus4: Frequenzen ändern sich nicht ({freqs:?})");
    eprintln!("PLUS4 Aufrufe {f0} -> {f1}, Frequenzen {freqs:?}, laut {loud}");
}
