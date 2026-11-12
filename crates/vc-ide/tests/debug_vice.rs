// SPDX-License-Identifier: GPL-3.0-or-later
//! Haltepunkte mit Bedingung und Datenhaltepunkte gegen echtes VICE (öffnet ein Emulatorfenster): nur mit VC_EMU_TESTS=1.
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vc_emu::{Backend, Vice};
use vc_ide::debug::Session;

fn session() -> Session {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let prg = root.join("build/border.prg"); assert!(prg.exists(), "build/border.prg fehlt (vcc asm examples/c64/border/border.asm -o build/border.prg)");
    let cpu = vc_target::load_cpu(&root.join("data"), "6510").unwrap();
    let v: Box<dyn Backend> = Box::new(Vice::launch(prg.to_str().unwrap(), None).expect("VICE starten"));
    Session::new(v, cpu, "c64".into(), HashMap::new())
}
fn wait_stopped(s: &mut Session, secs: u64) -> serde_json::Value {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end { let st = s.state(); if st["state"] == "stopped" { return st; } std::thread::sleep(Duration::from_millis(20)); }
    panic!("hält nicht an")
}

#[test]
fn vice_conditions_and_data_watchpoints() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let mut s = session();
    // Ausführungs-Haltepunkt auf `inc $d020` ($081A) mit Bedingung: nur wenn X (zuletzt gelesene Rahmenfarbe) == 5
    s.set_breakpoint(0x081A, true, Some("(x & 15) == 5"), 0).unwrap();
    s.cont().unwrap();
    let st = wait_stopped(&mut s, 30);
    assert_eq!((st["regs"]["pc"].as_u64(), st["regs"]["x"].as_u64().map(|x| x & 15)), (Some(0x081A), Some(5)), "{st}");
    assert!(st["points"][0]["hits"].as_u64().unwrap() >= 1);
    // Datenhaltepunkt: Schreibzugriff auf $D020, Bedingung auf den Speicherinhalt
    let id = s.set_watch(0xD020, 0xD020, "write", Some("([$d020] & 15) == 9"), 0).unwrap();
    s.set_breakpoint(0x081A, false, None, 0).unwrap();
    s.cont().unwrap();
    let st = wait_stopped(&mut s, 30);
    assert_eq!(st["stoppedAt"], id, "{st}");
    let mem = s.backend.read_mem(0xD020, 1).unwrap()[0] & 15; assert_eq!(mem, 9);
    assert_eq!(st["regs"]["pc"], 0x081D, "Stopp hinter dem schreibenden INC");
    s.backend.stop();
}

#[test]
fn atari800_condition_on_the_single_breakpoint() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let xex = root.join("build/colorbars.xex"); assert!(xex.exists(), "build/colorbars.xex fehlt (vcc asm examples/atari/colorbars/colorbars.asm --target atari8 --format xex)");
    let labels = std::fs::read_to_string(root.join("build/colorbars.vs")).unwrap();
    let wsync: u16 = u16::from_str_radix(&labels.lines().find_map(|l| l.strip_prefix("al C:").and_then(|r| r.strip_suffix(" .start_line"))).expect("Label .start_line").to_string(), 16).unwrap() + 3;
    let cpu = vc_target::load_cpu(&root.join("data"), "sally").unwrap();
    let v: Box<dyn Backend> = Box::new(vc_emu::Atari800::launch(xex.to_str().unwrap(), None).expect("atari800 starten"));
    let mut s = Session::new(v, cpu, "atari8".into(), HashMap::new());
    s.set_breakpoint(wsync, true, Some("x >= 20 && x < 40"), 0).unwrap();
    s.cont().unwrap();
    let st = wait_stopped(&mut s, 60);
    let x = st["regs"]["x"].as_u64().unwrap();
    assert_eq!(st["regs"]["pc"].as_u64(), Some(wsync as u64), "{st}");
    assert!((20..40).contains(&x), "X = {x}: Bedingung nicht beachtet");
    assert!(st["points"][0]["hits"].as_u64().unwrap() >= 1);
    s.backend.stop();
}

/// MEGA65 im Xemu über den UART-Monitor: nur mit VC_EMU_TESTS=1, VC_XEMU (Pfad zu xmega65) und VC_XEMU_SD (Ordner mit MEGA65.ROM).
#[test]
fn mega65_in_xemu_breakpoint_step_and_memory() {
    if std::env::var("VC_EMU_TESTS").is_err() || std::env::var("VC_XEMU").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1, VC_XEMU, VC_XEMU_SD setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bin = std::fs::read(root.join("build/cputest.bin")).expect("build/cputest.bin fehlt (vcc asm examples/mega65/cputest/cputest.asm --target mega65 --format bin)");
    let labels = std::fs::read_to_string(root.join("build/cputest.vs")).unwrap();
    let addr = |name: &str| u16::from_str_radix(labels.lines().find_map(|l| l.strip_prefix("al C:").and_then(|r| r.strip_suffix(&format!(" .{name}")))).unwrap(), 16).unwrap();
    let mut prg = vec![0x00, 0x20]; prg.extend(&bin);
    let mut m = vc_emu::Mega65::launch_xemu(None, &std::env::var("VC_XEMU_SD").unwrap()).expect("Xemu starten");
    m.load_prg(&prg, 0x2000).unwrap();
    assert_eq!(m.registers().unwrap().pc, 0x2000);
    assert_eq!(m.read_mem(0x2000, 4).unwrap(), bin[..4].to_vec());
    // Einzelschritt: sei / cle / see / ldz #$12
    for _ in 0..4 { m.step().unwrap(); }
    let r = m.registers().unwrap(); assert_eq!(r.pc, 0x2005, "{r:?}"); assert!(r.extra.iter().any(|(n, v)| n == "Z" && *v == 0x12), "{r:?}");
    // Haltepunkt am Ende, dann laufen lassen; Ergebnis aus dem Speicher
    let done = addr("done"); m.set_breakpoint(done).unwrap(); m.cont().unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    while m.state() == vc_emu::RunState::Running && Instant::now() < end { std::thread::sleep(Duration::from_millis(50)); }
    assert_eq!(m.state(), vc_emu::RunState::Stopped); assert_eq!(m.registers().unwrap().pc, done);
    assert_eq!(m.read_mem(0x84, 4).unwrap(), vec![0x79, 0x56, 0x34, 0x12], "INQ");
    assert_eq!(m.read_mem(0x6A, 1).unwrap(), vec![0xC0], "ASR");
    m.write_mem(0x0400, &[1, 2, 3]).unwrap(); assert_eq!(m.read_mem(0x0400, 3).unwrap(), vec![1, 2, 3]);
    m.stop();
}

/// Schreib-Beobachtung im MEGA65-Adapter (Einzelschritt-Emulation): Halt, sobald sich $84-$87 ändern (INQ im CPU-Test)
#[test]
fn mega65_in_xemu_write_watch() {
    if std::env::var("VC_EMU_TESTS").is_err() || std::env::var("VC_XEMU").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1, VC_XEMU, VC_XEMU_SD setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bin = std::fs::read(root.join("build/cputest.bin")).expect("build/cputest.bin fehlt");
    let mut prg = vec![0x00, 0x20]; prg.extend(&bin);
    let mut m = vc_emu::Mega65::launch_xemu(None, &std::env::var("VC_XEMU_SD").unwrap()).expect("Xemu starten");
    m.load_prg(&prg, 0x2000).unwrap();
    assert!(m.caps().watch);
    assert!(m.set_watch(0x84, 0x87, vc_emu::WatchKind::Read).is_err());
    let id = m.set_watch(0x84, 0x87, vc_emu::WatchKind::Write).unwrap();
    m.cont().unwrap();
    let end = Instant::now() + Duration::from_secs(60);
    while m.state() == vc_emu::RunState::Running && Instant::now() < end { std::thread::sleep(Duration::from_millis(5)); }
    assert_eq!(m.state(), vc_emu::RunState::Stopped);
    assert_eq!(m.last_hit(), Some(vc_emu::Hit::Watch(id)));
    assert_ne!(m.read_mem(0x84, 4).unwrap(), vec![0, 0, 0, 0]);
    m.clear_watch(id).unwrap();
    m.stop();
}

/// C128 (x128) und Plus/4 (xplus4): Vorlagenprogramm bauen, im VICE starten und am `inc` der Schleife anhalten.
#[test]
fn vice_c128_and_plus4_breakpoint() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let data = root.join("data");
    for (sys, prog, cpu_id) in [("c128", "x128", "8502"), ("plus4", "xplus4", "7501")] {
        let src = std::fs::read_to_string(data.join(format!("templates/asm-hello/{sys}.asm"))).unwrap().replace("{{name}}", "t");
        let tgt = vc_target::load_target(&data, sys).unwrap();
        let cpu = vc_target::load_cpu(&data, &tgt.cpu).unwrap();
        assert_eq!(tgt.cpu, cpu_id);
        let mut o = vc_asm::Options::new(format!("{sys}.asm"), vc_asm::Policy::Warn);
        o.predefined = vc_target::chip_symbols(&data, &tgt).unwrap();
        let out = vc_asm::assemble(&src, &cpu, &o).expect("assemblieren");
        let prg = vc_asm::to_prg(&out).unwrap();
        let path = std::env::temp_dir().join(format!("vc_{sys}_test.prg")); std::fs::write(&path, &prg).unwrap();
        let inc = prg[2..].iter().position(|&b| b == 0xEE).expect("INC") as u16 + u16::from_le_bytes([prg[0], prg[1]]);
        let v: Box<dyn Backend> = Box::new(Vice::launch_system(prog, path.to_str().unwrap(), None).unwrap_or_else(|e| panic!("{prog}: {e}")));
        let mut s = Session::new(v, cpu, sys.into(), HashMap::new());
        s.set_breakpoint(inc, true, None, 0).unwrap();
        s.cont().unwrap();
        let st = wait_stopped(&mut s, 60);
        assert_eq!(st["regs"]["pc"].as_u64(), Some(inc as u64), "{sys}: {st}");
        s.backend.stop();
    }
}

/// Das C64-Musikbeispiel läuft mit der Bildrate (PAL 50 Hz): bei Takt 6 rund 8,3 Zeilen je Sekunde (nur mit VC_EMU_TESTS=1, öffnet VICE)
#[test]
fn c64_music_demo_runs_at_frame_rate() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/music_demo");
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let b = vc_ide::build::build(&vc_ide::build::Ctx { root: &root, data: &data }, "music_demo.asm", None).unwrap();
    assert_eq!(b["ok"], true, "{b}");
    let sym = |n: &str| b["symbols"].as_array().unwrap().iter().find(|s| s["name"] == n).and_then(|s| s["value"].as_u64()).unwrap_or_else(|| panic!("Symbol {n}")) as u16;
    let (row, ord) = (sym("mus_row"), sym("mus_ord"));
    let art = root.join(b["artifact"].as_str().unwrap());
    let mut v = Vice::launch(art.to_str().unwrap(), None).expect("VICE starten");
    v.cont().ok();
    std::thread::sleep(Duration::from_secs(4));
    let pos = |v: &mut Vice| -> u32 { v.pause().unwrap(); std::thread::sleep(Duration::from_millis(300)); let r = v.read_mem(row, 1).unwrap()[0] as u32; let o = v.read_mem(ord, 1).unwrap()[0] as u32; v.cont().unwrap(); o * 32 + r };
    let t0 = std::time::Instant::now(); let a = pos(&mut v);
    std::thread::sleep(Duration::from_secs(6)); let c = pos(&mut v); let dt = t0.elapsed().as_secs_f64();
    let rows = ((c + 128 - a) % 128) as f64;   // 4 Positionen × 32 Zeilen
    let rate = rows / dt;
    v.stop();
    assert!((rate - 50.0 / 6.0).abs() < 0.9, "Zeilen je Sekunde {rate} (erwartet {:.2})", 50.0 / 6.0);
}

/// Speichersichten des C64 in VICE: unter ROM und E/A liegt eigenes RAM (nur mit VC_EMU_TESTS=1)
#[test]
fn vice_reads_ram_under_rom_and_io() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let prg = root.join("build/border.prg"); assert!(prg.exists(), "build/border.prg fehlt");
    let mut v = Vice::launch(prg.to_str().unwrap(), None).expect("VICE starten");
    std::thread::sleep(Duration::from_secs(3)); v.pause().ok(); std::thread::sleep(Duration::from_millis(300));
    let cpu = v.read_mem(0xE000, 8).unwrap(); let ram = v.read_space("ram", 0xE000, 8).unwrap();
    let io = v.read_space("io", 0xD020, 1).unwrap();
    v.stop();
    assert_ne!(cpu, ram, "KERNAL-ROM und RAM darunter unterscheiden sich: {cpu:?} {ram:?}");
    assert_eq!(io.len(), 1);
}

/// VC-20 (xvic): Vorlagenprogramm im Grundausbau und mit +8 KB bauen (BASIC-Start $1001 bzw. $1201), im VICE starten und am Rasterwart-Zyklus anhalten.
#[test]
fn vice_vic20_memory_configs() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    for (mem, org) in [("unexpanded", 0x1001u16), ("8k", 0x1201)] {
        let mut tgt = vc_target::load_target(&data, "vic20").unwrap();
        tgt.apply_memory(Some(mem)).unwrap();
        assert_eq!(tgt.basic_load_addr, Some(org));
        let src = std::fs::read_to_string(data.join("templates/asm-hello/vic20.asm")).unwrap().replace("{{name}}", "t").replace(".org $1001", &format!(".org ${org:04x}"));
        let cpu = vc_target::load_cpu(&data, &tgt.cpu).unwrap();
        let mut o = vc_asm::Options::new("vic20.asm", vc_asm::Policy::Warn);
        o.predefined = vc_target::chip_symbols(&data, &tgt).unwrap();
        o.predefined.extend(tgt.memory_symbols());
        let out = vc_asm::assemble(&src, &cpu, &o).expect("assemblieren");
        let prg = vc_asm::to_prg(&out).unwrap();
        assert_eq!(u16::from_le_bytes([prg[0], prg[1]]), org);
        let path = std::env::temp_dir().join(format!("vc_vic20_{mem}.prg")); std::fs::write(&path, &prg).unwrap();
        let sta = prg[2..].iter().position(|&b| b == 0x8D).expect("STA") as u16 + org;   // Schreiben der Rahmenfarbe
        let extra = vc_ide::launch::project_args(&data, std::path::Path::new("/nonexistent"), "vic20");
        assert!(extra.is_empty() || extra[0] == "-memory");
        let v: Box<dyn Backend> = Box::new(Vice::launch_with("xvic", path.to_str().unwrap(), None, &["-memory".to_string(), tgt.memory_config().unwrap().vice.clone()]).unwrap_or_else(|e| panic!("xvic: {e}")));
        let mut s = Session::new(v, cpu, "vic20".into(), HashMap::new());
        s.set_breakpoint(sta, true, None, 0).unwrap();
        s.cont().unwrap();
        let st = wait_stopped(&mut s, 60);
        assert_eq!(st["regs"]["pc"].as_u64(), Some(sta as u64), "{mem}: {st}");
        s.backend.stop();
    }
}

/// VC-20-Steckmodul: Vorlage bauen (.a0), per Debug-Starter als Modul einlegen, das Modul initialisiert das System und schreibt auf den Bildschirm.
#[test]
fn vice_vic20_cartridge_template() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let dir = std::env::temp_dir().join(format!("vc-vic20cart-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir);
    vc_media::wizard::create(&data, &dir, "probe", &["vic20".to_string()], "cartridge").unwrap();
    let b = vc_ide::build::build(&vc_ide::build::Ctx { root: &dir, data: &data }, "src/vic20/module.asm", Some("vic20")).unwrap();
    assert_eq!(b["ok"], true, "{b}");
    let art = dir.join(b["artifact"].as_str().unwrap());
    assert_eq!(art.extension().unwrap(), "a0"); assert_eq!(std::fs::metadata(&art).unwrap().len(), 8192);
    let msg = b["symbols"].as_array().unwrap().iter().find(|s| s["name"] == "msg").and_then(|s| s["value"].as_u64()).unwrap() as u16;
    let spec = vc_ide::debug::LaunchSpec { artifact: art.to_string_lossy().into_owned(), target: "vic20".into(), adapter: String::new(), host: None, password: None, entry: None, extra_args: vec![] };
    let backend = (vc_ide::debug::default_launcher())(&spec).expect("xvic mit Modul starten");
    let cpu = vc_target::load_cpu(&data, "nmos6502").unwrap();
    let mut s = Session::new(backend, cpu, "vic20".into(), HashMap::new());
    s.set_breakpoint(msg - 3, true, None, 0).unwrap();      // die Schleife `jmp .done` hinter der Ausgabe
    s.cont().unwrap();
    let st = wait_stopped(&mut s, 60);
    assert_eq!(st["regs"]["pc"].as_u64(), Some((msg - 3) as u64), "{st}");
    let screen = s.backend.read_mem(0x1E00, 22).unwrap();
    assert_eq!(&screen[..5], &[8, 1, 12, 12, 15], "HALLO auf dem Bildschirm: {screen:?}");
    s.backend.stop();
}
