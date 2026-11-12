// SPDX-License-Identifier: GPL-3.0-or-later
//! Trap-Modus mit echtem atari800 (Atari 8 Bit, PAL): Register, Flags über den Vorspann, Strahlzeit, Zeitüberschreitung und Neustart.
//! Nur mit VC_EMU_TESTS=1 (öffnet ein Emulatorfenster); es darf kein anderer atari800 laufen.
use std::path::PathBuf;
use std::process::Command;
use vc_ide::build::{build, Ctx};
use vc_ide::tests::runner::{run_tests, RunOptions};
use vc_ide::tests::{Outcome, Prepared, State, TestFile};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

const ASM: &str = "\
.target atari8
.cpu sally { illegals = deny }
.org $2000
start:  rts
add:    clc
        adc $e0
        sta $e0
        rts
flagprobe: php
        pla
        sta $e1
        rts
hang:   jmp hang
isr:    inc $e2
        rti
";

const TESTS: &str = r#"{ "target": "atari8", "program": "build/p.xex", "tests": [
  { "name": "add: 5 + 3", "setup": { "mem": { "$e0": 3 }, "a": 5 }, "call": "add", "expect": [ "[$e0] == 8", "!fc", "!fz", "cycles >= 14", "cycles < 40" ] },
  { "name": "add: Übertrag", "setup": { "mem": { "$e0": 100 }, "a": 200 }, "call": "add", "expect": [ "[$e0] == 44", "fc" ] },
  { "name": "Flags kommen an", "setup": { "fn": 1, "fz": 1, "fc": 1, "fd": 1 }, "call": "flagprobe", "expect": [ "([$e1] & $CB) == $8B" ] },
  { "name": "falsche Erwartung", "setup": { "mem": { "$e0": 3 }, "a": 5 }, "call": "add", "expect": [ "[$e0] == 9" ] },
  { "name": "hängt", "call": "hang", "expect": [ "a == 0" ], "timeoutMs": 600 },
  { "name": "nach dem Neustart", "setup": { "mem": { "$e0": 1 }, "a": 1 }, "call": "add", "expect": [ "[$e0] == 2" ] },
  { "name": "rti: Interruptroutine", "setup": { "mem": { "$e2": 3 } }, "call": "isr", "return": "rti", "expect": [ "[$e2] == 4" ] }
] }"#;

fn alive() -> bool { Command::new("pgrep").args(["-x", "atari800"]).stdout(std::process::Stdio::null()).status().map_or(false, |s| s.success()) }
fn by<'a>(o: &'a [Outcome], n: &str) -> &'a Outcome { o.iter().find(|x| x.name == n).unwrap_or_else(|| panic!("{n} fehlt in {o:?}")) }

#[test]
fn atari800_trap_mode_end_to_end() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    assert!(!alive(), "es läuft schon ein atari800");
    let dir = std::env::temp_dir().join(format!("vc-trap-a8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("p.asm"), ASM).unwrap();
    assert_eq!(build(&Ctx { root: &dir, data: &data() }, "p.asm", Some("atari8")).unwrap()["ok"], true);
    std::fs::write(dir.join("t.vctest"), TESTS).unwrap();
    let tf = TestFile::load(&dir.join("t.vctest")).unwrap();
    let pre = Prepared::load(&tf, &data()).unwrap();
    let ro = RunOptions { boot_wait: std::time::Duration::ZERO, ..Default::default() };
    let mut launch = || vc_emu::Atari800::launch_bare(None, None, &[]).map(|v| Box::new(v) as Box<dyn vc_emu::Backend>);
    let o = run_tests(&tf, &pre, &ro, &mut launch);
    for x in &o { eprintln!("{:?} {} {:?} {}", x.state, x.name, x.measured, x.detail); }
    assert!(!alive(), "atari800 läuft nach dem Lauf noch");
    let p = by(&o, "add: 5 + 3");
    assert_eq!(p.state, State::Passed, "{p:?}");
    assert!(p.measured.as_deref().map_or(false, |m| m.ends_with("(Strahlzeit)")), "{:?}", p.measured);
    assert_eq!(by(&o, "add: Übertrag").state, State::Passed);
    assert_eq!(by(&o, "Flags kommen an").state, State::Passed, "{:?}", by(&o, "Flags kommen an"));
    let f = by(&o, "falsche Erwartung");
    assert!(f.is_error() && f.detail.contains("linke Seite = 8") && f.detail.contains("rechte Seite = 9"), "{}", f.detail);
    assert_eq!(by(&o, "hängt").state, State::TimedOut);
    assert_eq!(by(&o, "nach dem Neustart").state, State::Passed, "Neustart nach Zeitüberschreitung");
    assert_eq!(by(&o, "rti: Interruptroutine").state, State::Passed, "{:?}", by(&o, "rti: Interruptroutine"));
    std::fs::remove_dir_all(&dir).ok();
}
