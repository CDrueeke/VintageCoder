// SPDX-License-Identifier: GPL-3.0-or-later
//! Postfach-Modus (Testkonzept Stufe 3): Auswertung des Postfachs und Läufer mit Scheinadapter (ohne Emulator) sowie mit echtem VICE
//! und atari800 (nur mit VC_EMU_TESTS=1).
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use vc_emu::{Backend, Regs, RunState};
use vc_ide::build::{build, Ctx};
use vc_ide::tests::mailbox::{find_done_address, run_mailbox, Mailbox, Verdict};
use vc_ide::tests::runner::RunOptions;
use vc_ide::tests::suite::launch_program;
use vc_ide::tests::{MailboxCase, Outcome, Prepared, State, TestFile, TestMode};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn mailbox_verdicts() {
    let m = |b: [u8; 6]| Mailbox::parse(&b).unwrap().verdict();
    assert_eq!(m([2, 4, 0, 0, 0, 0]), Verdict::Pass { checks: 4 });
    assert_eq!(m([2, 0, 0, 0, 0, 0]), Verdict::NoChecks, "ohne Prüfung gilt nie als bestanden");
    assert_eq!(m([3, 5, 3, 0x7F, 0x10, 1]), Verdict::Fail { number: 3, of: 5, kind: 1, actual: 0x7F, expected: 0x10 });
    assert_eq!(m([1, 2, 0, 0, 0, 0]), Verdict::Unfinished(1));
    assert_eq!(m([0xFF, 0, 0, 0, 0, 0]), Verdict::Unfinished(0xFF));
    assert!(Mailbox::parse(&[2, 1, 0]).is_none());
}

#[test]
fn mailbox_case_and_mode_parsing() {
    let tf = |s: &str| TestFile::parse(Path::new("x.vctest"), s);
    assert_eq!(tf(r#"{"target":"c64","program":"a","mode":"mailbox"}"#).unwrap().mode, TestMode::Mailbox);
    assert_eq!(tf(r#"{"target":"c64","program":"a"}"#).unwrap().mode, TestMode::Trap);
    assert!(tf(r#"{"target":"c64","program":"a","mode":"kaputt"}"#).unwrap_err().contains("mailbox"));
    let c = MailboxCase::parse(0, &serde_json::json!({ "name": "x", "expect": ["[$fb] == 1"], "timeoutMs": 500 })).unwrap();
    assert_eq!((c.name.as_str(), c.expect.len(), c.timeout_ms), ("x", 1, 500));
    assert_eq!(MailboxCase::parse(2, &serde_json::json!({})).unwrap().timeout_ms, 15000);
    assert!(MailboxCase::parse(0, &serde_json::json!({ "timeoutMs": 0 })).is_err());
    assert!(MailboxCase::parse(0, &serde_json::json!({ "expect": "a" })).is_err());
}

// ---------- Programm bauen ----------

fn project(tag: &str, target: &str, name: &str, asm: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("vc-mbox-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), asm).unwrap();
    let out = build(&Ctx { root: &dir, data: &data() }, name, Some(target)).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    let artifact = dir.join(out["artifact"].as_str().unwrap());
    (dir, artifact)
}

const C64_PASS: &str = "\
.target c64\n.cpu 6510 { illegals = deny }\n.include \"c64/test.inc\"\n.org $0801\n.basicstub start\n\
start:  sei\n        test_begin\n        lda #21\n        jsr double\n        assert_a_eq 42\n        ldx #5\n        inx\n        assert_x_eq 6\n        assert_a_eq 42\n        ldy #9\n        assert_y_eq 9\n\
        lda #$7f\n        sta $fb\n        assert_mem_eq $fb, $7f\n        test_end\ndouble: asl\n        rts\n";

fn tf(dir: &Path, json: &str) -> TestFile { let p = dir.join("t.vctest"); std::fs::write(&p, json).unwrap(); TestFile::load(&p).unwrap() }

#[test]
fn test_inc_assembles_and_the_done_loop_is_found() {
    let (dir, art) = project("asm", "c64", "p.asm", C64_PASS);
    let tf = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{}", "mode": "mailbox", "tests": [ {{ "name": "t" }} ] }}"#, art.strip_prefix(&dir).unwrap().display()));
    let pre = Prepared::load(&tf, &data()).unwrap();
    assert_eq!(pre.target.test_mailbox, Some(0x033C));
    let done = find_done_address(&pre).expect("Endlosschleife von test_end");
    let img = pre.image.as_ref().unwrap();
    assert_eq!((img.get(done), img.get(done + 1), img.get(done + 2)), (Some(0x4C), Some(done as u8), Some((done >> 8) as u8)));
    // Postfach-Adressen der drei Ziele; ohne Eintrag (plus4, vic20, mega65) kein Postfach
    for (t, a) in [("c64", Some(0x033C)), ("c128", Some(0x0B00)), ("atari8", Some(0x0600)), ("plus4", None), ("vic20", None), ("mega65", None)] {
        assert_eq!(vc_target::load_target(&data(), t).unwrap().test_mailbox, a, "{t}");
    }
    // Debug-Register der Ultimate 64 ($D7FF): nur im C64-Ziel; test_begin und test_end schreiben dorthin (sta $D7FF = 8D FF D7), sonst nirgends
    for t in ["c64", "c128", "atari8", "plus4", "vic20", "mega65"] {
        assert_eq!(vc_target::load_target(&data(), t).unwrap().test_debug_reg, if t == "c64" { Some(0xD7FF) } else { None }, "{t}");
    }
    let img = pre.image.as_ref().unwrap(); let mut n = 0;
    for a in 0..0xFFFDu32 { if (img.get(a as u16), img.get(a as u16 + 1), img.get(a as u16 + 2)) == (Some(0x8D), Some(0xFF), Some(0xD7)) { n += 1; } }
    assert_eq!(n, 2, "sta $D7FF in test_begin und test_end");
    std::fs::remove_dir_all(&dir).ok();
}

// ---------- Scheinadapter ----------

struct Fake { mb: Vec<Vec<u8>>, reads: Arc<AtomicUsize>, reach: bool, alive: Arc<AtomicUsize> }
impl Fake {
    fn new(mb: Vec<Vec<u8>>, reach: bool, reads: Arc<AtomicUsize>, alive: Arc<AtomicUsize>) -> Fake { alive.fetch_add(1, Ordering::SeqCst); Fake { mb, reads, reach, alive } }
}
impl Drop for Fake { fn drop(&mut self) { self.alive.fetch_sub(1, Ordering::SeqCst); } }
impl Backend for Fake {
    fn name(&self) -> &str { "Schein" }
    fn state(&mut self) -> RunState { RunState::Stopped }
    fn registers(&mut self) -> Result<Regs, String> { Ok(Regs { pc: 0x0900, a: 7, ..Default::default() }) }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        if addr == 0x033C { let n = self.reads.fetch_add(1, Ordering::SeqCst); let v = &self.mb[n.min(self.mb.len() - 1)]; return Ok(v[..len.min(v.len())].to_vec()); }
        Ok(vec![if addr == 0x00FB { 1 } else { 0 }; len])
    }
    fn write_mem(&mut self, _: u16, _: &[u8]) -> Result<(), String> { Ok(()) }
    fn step(&mut self) -> Result<(), String> { Ok(()) }
    fn cont(&mut self) -> Result<(), String> { Ok(()) }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn run_until(&mut self, _: u16, _: Duration) -> Result<bool, String> { Ok(self.reach) }
    fn set_breakpoint(&mut self, _: u16) -> Result<(), String> { Ok(()) }
    fn clear_breakpoint(&mut self, _: u16) -> Result<(), String> { Ok(()) }
    fn breakpoints(&self) -> Vec<u16> { vec![] }
    fn stop(&mut self) {}
}

fn run_fake(tf: &TestFile, pre: &Prepared, mb: Vec<Vec<u8>>, reach: bool, ro: RunOptions) -> (Vec<Outcome>, usize, usize) {
    let (reads, alive, max) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (r2, a2, m2) = (reads.clone(), alive.clone(), max.clone());
    let mut launch = move |_: &Path| -> Result<Box<dyn Backend>, String> {
        let f = Fake::new(mb.clone(), reach, r2.clone(), a2.clone());
        m2.fetch_max(a2.load(Ordering::SeqCst), Ordering::SeqCst);
        Ok(Box::new(f))
    };
    let out = run_mailbox(tf, pre, &ro, &mut launch);
    (out, reads.load(Ordering::SeqCst), max.load(Ordering::SeqCst))
}

#[test]
fn runner_reports_pass_fail_nochecks_timeout_and_extra_expectations() {
    let (dir, art) = project("fake", "c64", "p.asm", C64_PASS);
    let prog = art.strip_prefix(&dir).unwrap().display().to_string();
    let t = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "t", "expect": [ "[$fb] == 1" ] }} ] }}"#));
    let pre = Prepared::load(&t, &data()).unwrap();
    let ro = RunOptions { boot_wait: Duration::ZERO, ..Default::default() };
    let go = |mb: Vec<Vec<u8>>, reach: bool, ro: &RunOptions| run_fake(&t, &pre, mb, reach, ro.clone()).0.remove(0);
    let ok = go(vec![vec![2, 4, 0, 0, 0, 0, 0, 0]], true, &ro);
    assert_eq!((ok.state, ok.measured.as_deref()), (State::Passed, Some("4 Prüfungen")), "{ok:?}");
    let f = go(vec![vec![3, 5, 3, 0x7F, 0x10, 2, 0, 0]], true, &ro);
    assert!(f.is_error() && f.detail.contains("Nr. 3 von 5") && f.detail.contains("assert_x_eq") && f.detail.contains("Ist-Wert 127 ($7F)") && f.detail.contains("erwartet 16 ($10)"), "{}", f.detail);
    let n = go(vec![vec![2, 0, 0, 0, 0, 0, 0, 0]], true, &ro);
    assert!(n.is_error() && n.detail.contains("keine Prüfung"), "{}", n.detail);
    let u = go(vec![vec![0xAA, 1, 0, 0, 0, 0, 0, 0]], true, &ro);
    assert!(u.is_error() && u.detail.contains("überschrieben"), "{}", u.detail);
    let to = go(vec![vec![0, 0, 0, 0, 0, 0, 0, 0]], false, &ro);
    assert_eq!(to.state, State::TimedOut); assert!(to.detail.contains("test_begin wurde nie ausgeführt"), "{}", to.detail);
    let to = go(vec![vec![1, 2, 0, 0, 0, 0, 0, 0]], false, &ro);
    assert!(to.detail.contains("Status 1, 2 Prüfung"), "{}", to.detail);
    // zusätzliche Erwartung nach dem Ende
    let bad = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "t", "expect": [ "[$fb] == 9" ] }} ] }}"#));
    let o = run_fake(&bad, &pre, vec![vec![2, 4, 0, 0, 0, 0, 0, 0]], true, ro.clone()).0.remove(0);
    assert!(o.is_error() && o.detail.contains("alle 4 Prüfungen") && o.detail.contains("[$fb] == 9"), "{}", o.detail);
    // Abfragemodus: Postfach in Abständen lesen, bis ein Ende dasteht
    let (out, reads, max_alive) = run_fake(&t, &pre, vec![vec![0; 8], vec![1, 1, 0, 0, 0, 0, 0, 0], vec![2, 4, 0, 0, 0, 0, 0, 0]], true, RunOptions { poll: true, ..ro.clone() });
    assert_eq!(out[0].state, State::Passed, "{:?}", out[0]);
    assert_eq!(reads, 3, "drei Lesevorgänge bis zum Ende");
    assert_eq!(max_alive, 1, "immer nur ein Emulator");
    // kein Ende im Abfragemodus: Zeitüberschreitung
    let t2 = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "t", "timeoutMs": 250 }} ] }}"#));
    let (out, _, _) = run_fake(&t2, &pre, vec![vec![1, 1, 0, 0, 0, 0, 0, 0]], true, RunOptions { poll: true, ..ro.clone() });
    assert_eq!(out[0].state, State::TimedOut, "{:?}", out[0]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn runner_skips_without_mailbox_or_emulator() {
    let (dir, art) = project("skip", "c64", "p.asm", C64_PASS);
    let prog = art.strip_prefix(&dir).unwrap().display().to_string();
    let t = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "a" }}, {{ "name": "b" }} ] }}"#));
    let mut pre = Prepared::load(&t, &data()).unwrap();
    let ro = RunOptions { boot_wait: Duration::ZERO, ..Default::default() };
    let o = run_mailbox(&t, &pre, &ro, &mut |_: &Path| Err("nicht installiert".to_string()));
    assert!(o.len() == 2 && o.iter().all(|x| x.state == State::Skipped && x.detail.contains("nicht installiert")), "{o:?}");
    pre.target.test_mailbox = None;
    let o = run_mailbox(&t, &pre, &ro, &mut |_: &Path| Err("egal".to_string()));
    assert!(o.iter().all(|x| x.state == State::Skipped && x.detail.contains("testMailbox")), "{o:?}");
    // Abbruch von außen
    pre.target.test_mailbox = Some(0x033C);
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let o = run_mailbox(&t, &pre, &RunOptions { cancel: Some(cancel), ..ro }, &mut |_: &Path| Err("x".to_string()));
    assert!(o.iter().all(|x| x.detail == "abgebrochen"), "{o:?}");
    std::fs::remove_dir_all(&dir).ok();
}

// ---------- echte Emulatoren ----------

fn enabled() -> bool { std::env::var("VC_EMU_TESTS").is_ok() }
fn no_vice() -> bool { !std::process::Command::new("pgrep").args(["-f", "[b]inarymonitoraddress"]).stdout(std::process::Stdio::null()).status().map_or(false, |s| s.success()) }

#[test]
fn vice_runs_pass_fail_and_hang() {
    if !enabled() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    assert!(no_vice(), "es läuft schon ein Emulator");
    let fail = C64_PASS.replace("assert_x_eq 6", "assert_x_eq 7");
    let hang = "\
.target c64\n.include \"c64/test.inc\"\n.org $0801\n.basicstub start\nstart:  sei\n        test_begin\n        lda #1\n        assert_a_eq 1\nforever: jmp forever\n        test_end\n";
    for (tag, asm, expect_state, needle) in [("pass", C64_PASS.to_string(), State::Passed, "5 Prüfungen"), ("fail", fail, State::Failed, "Nr. 2 von 5"), ("hang", hang.to_string(), State::TimedOut, "Status 1, 1 Prüfung")] {
        let (dir, art) = project(&format!("vice-{tag}"), "c64", "p.asm", &asm);
        let prog = art.strip_prefix(&dir).unwrap().display().to_string();
        let t = tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "{tag}", "timeoutMs": 15000 }} ] }}"#));
        let pre = Prepared::load(&t, &data()).unwrap();
        for poll in [false, true] {
            if tag == "hang" && !poll { continue; }                       // ohne Endschleife gibt es keinen Haltepunkt: der Hang läuft im Abfragemodus
            let ro = RunOptions { poll: poll || tag == "hang", boot_wait: Duration::ZERO, ..Default::default() };
            let ro = if tag == "hang" { RunOptions { ..ro } } else { ro };
            let t2 = if tag == "hang" { tf(&dir, &format!(r#"{{ "target": "c64", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "{tag}", "timeoutMs": 9000 }} ] }}"#)) } else { t.clone() };
            let o = run_mailbox(&t2, &pre, &ro, &mut |p: &Path| launch_program("vice", "c64", p)).remove(0);
            eprintln!("{tag} poll={poll}: {:?} {:?} {}", o.state, o.measured, o.detail);
            assert_eq!(o.state, expect_state, "{tag} poll={poll}: {o:?}");
            assert!(o.measured.as_deref().unwrap_or("").contains(needle) || o.detail.contains(needle), "{tag}: {o:?}");
            assert!(no_vice(), "{tag}: Emulator läuft nach dem Lauf noch");
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[test]
fn atari800_runs_pass_and_fail() {
    if !enabled() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let pass = "\
.target atari8\n.include \"atari8bit/test.inc\"\n.org $2000\nstart:  test_begin\n        lda #3\n        jsr twice\n        assert_a_eq 6\n        ldx #1\n        assert_x_eq 1\n        test_end\ntwice:  asl\n        rts\n.run start\n";
    let fail = pass.replace("assert_a_eq 6", "assert_a_eq 7");
    for (tag, asm, state, needle) in [("pass", pass.to_string(), State::Passed, "2 Prüfungen"), ("fail", fail, State::Failed, "Nr. 1 von 2")] {
        let (dir, art) = project(&format!("a8-{tag}"), "atari8", "p.asm", &asm);
        let prog = art.strip_prefix(&dir).unwrap().display().to_string();
        let t = tf(&dir, &format!(r#"{{ "target": "atari8", "program": "{prog}", "mode": "mailbox", "tests": [ {{ "name": "{tag}", "timeoutMs": 15000 }} ] }}"#));
        let pre = Prepared::load(&t, &data()).unwrap();
        assert_eq!(pre.target.test_mailbox, Some(0x0600));
        let o = run_mailbox(&t, &pre, &RunOptions { boot_wait: Duration::ZERO, ..Default::default() }, &mut |p: &Path| launch_program("atari800", "atari8", p)).remove(0);
        eprintln!("atari {tag}: {:?} {:?} {}", o.state, o.measured, o.detail);
        assert_eq!(o.state, state, "{o:?}");
        assert!(o.measured.as_deref().unwrap_or("").contains(needle) || o.detail.contains(needle), "{o:?}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
