// SPDX-License-Identifier: GPL-3.0-or-later
//! Trap-Modus mit echtem VICE (x64sc): Register, Flags, gemessene Zyklen (auch über ein Bild hinaus), Zeitüberschreitung und Neustart.
//! Nur mit VC_EMU_TESTS=1 (öffnet ein Emulatorfenster). Läuft einmal mit direkt gesetzten Registern, einmal mit dem Vorspann.
use std::path::PathBuf;
use std::process::Command;
use vc_ide::build::{build, Ctx};
use vc_ide::tests::runner::{run_tests, RunOptions};
use vc_ide::tests::{Level, Outcome, Prepared, State, TestFile};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

const ASM: &str = "\
.target c64
.cpu 6510 { illegals = deny }
.org $0801
.basicstub s
s:      rts
add:    clc
        adc $fb
        sta $fb
        rts
flagprobe: php
        pla
        sta $fc
        rts
hang:   jmp hang
isr:    inc $d020
        rti
longloop: ldy #0
.o:     ldx #0
.i:     dex
        bne .i
        dey
        bne .o
        rts
";

const TESTS: &str = r#"{ "target": "c64", "program": "build/p.prg", "tests": [
  { "name": "add: 5 + 3", "setup": { "mem": { "$fb": 3 }, "a": 5 }, "call": "add", "expect": [ "[$fb] == 8", "!fc", "!fz" ], "cycles": { "max": 14 } },
  { "name": "add: Übertrag", "setup": { "mem": { "$fb": 100 }, "a": 200 }, "call": "add", "expect": [ "[$fb] == 44", "fc", "cycles == 14" ] },
  { "name": "Flags kommen an", "setup": { "fn": 1, "fz": 1, "fc": 1, "fd": 1 }, "call": "flagprobe", "expect": [ "([$fc] & $CB) == $8B", "cycles == 16" ] },
  { "name": "falsche Erwartung", "setup": { "mem": { "$fb": 3 }, "a": 5 }, "call": "add", "expect": [ "[$fb] == 9" ] },
  { "name": "Budget zu klein", "setup": { "mem": { "$fb": 3 }, "a": 5 }, "call": "add", "cycles": { "max": 10 } },
  { "name": "hängt", "call": "hang", "expect": [ "a == 0" ], "timeoutMs": 400 },
  { "name": "nach dem Neustart", "setup": { "mem": { "$fb": 1 }, "a": 1 }, "call": "add", "expect": [ "[$fb] == 2" ] },
  { "name": "rti: Interruptroutine", "setup": { "mem": { "$d020": 3 } }, "call": "isr", "return": "rti", "expect": [ "([$d020] & 15) == 4", "cycles == 12" ] },
  { "name": "mehr als ein Bild", "call": "longloop", "expect": [ "cycles == 329223", "x == 0", "y == 0" ], "timeoutMs": 5000 }
] }"#;

fn alive() -> bool { Command::new("pgrep").args(["-f", "[b]inarymonitoraddress"]).stdout(std::process::Stdio::null()).status().map_or(false, |s| s.success()) }
fn by<'a>(o: &'a [Outcome], n: &str) -> &'a Outcome { o.iter().find(|x| x.name == n).unwrap_or_else(|| panic!("{n} fehlt in {o:?}")) }

#[test]
fn vice_trap_mode_end_to_end() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    assert!(!alive(), "es läuft schon ein Emulator");
    let dir = std::env::temp_dir().join(format!("vc-trap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("p.asm"), ASM).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "p.asm", Some("c64")).unwrap()["ok"], true);
    std::fs::write(dir.join("t.vctest"), TESTS).unwrap();
    let tf = TestFile::load(&dir.join("t.vctest")).unwrap();
    let pre = Prepared::load(&tf, &data()).unwrap();
    for force_prologue in [false, true] {
        let ro = RunOptions { force_prologue, ..Default::default() };
        let mut launch = || vc_emu::Vice::launch_with("x64sc", "", None, &[]).map(|v| Box::new(v) as Box<dyn vc_emu::Backend>);
        let o = run_tests(&tf, &pre, &ro, &mut launch);
        let m = if force_prologue { "Vorspann" } else { "direkt" };
        eprintln!("--- {m}"); for x in &o { eprintln!("{:?} {} {:?} {}", x.state, x.name, x.measured, x.detail); }
        assert!(!alive(), "{m}: Emulator läuft nach dem Lauf noch");
        assert_eq!(by(&o, "add: 5 + 3").state, State::Passed, "{m}");
        assert_eq!(by(&o, "add: 5 + 3").measured.as_deref(), Some("14 Zyklen"), "{m}");
        assert_eq!(by(&o, "add: Übertrag").state, State::Passed, "{m}");
        assert_eq!(by(&o, "Flags kommen an").state, State::Passed, "{m}: {:?}", by(&o, "Flags kommen an"));
        let f = by(&o, "falsche Erwartung");
        assert!(f.is_error() && f.detail.contains("linke Seite = 8") && f.detail.contains("rechte Seite = 9") && f.detail.contains("A=$"), "{m}: {}", f.detail);
        let b = by(&o, "Budget zu klein");
        assert_eq!((b.state, b.level), (State::Failed, Level::Hint), "{m}"); assert!(b.detail.contains("14 Zyklen gemessen"), "{}", b.detail);
        assert_eq!(by(&o, "hängt").state, State::TimedOut, "{m}");
        assert_eq!(by(&o, "nach dem Neustart").state, State::Passed, "{m}: Neustart nach Zeitüberschreitung");
        assert_eq!(by(&o, "rti: Interruptroutine").state, State::Passed, "{m}: {:?}", by(&o, "rti: Interruptroutine"));
        assert_eq!(by(&o, "mehr als ein Bild").state, State::Passed, "{m}: {:?}", by(&o, "mehr als ein Bild"));
    }
    std::fs::remove_dir_all(&dir).ok();
}
