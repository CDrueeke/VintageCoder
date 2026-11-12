// SPDX-License-Identifier: GPL-3.0-or-later
//! Postfach-Modus auf Hardware (Testkonzept T6) gegen einen Scheinserver für die Ultimate-REST-Schnittstelle: kein echtes Gerät.
//! Gegen echte Hardware (Gerät muss ausdrücklich genannt werden; lädt ein Programm, setzt zurück):
//! `U64_HOST=192.168.x.y [U64_PASSWORD=…] cargo test -p vc-ide --test hardware -- --ignored --nocapture`
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use vc_ide::build::{build, Ctx};
use vc_ide::tests::suite::{run, Hardware, Only, RunRequest, Suite};
use vc_ide::tests::State;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// Scheingerät: Postfach bei $033C liefert nacheinander die Einträge von `script` (der letzte bleibt); jede Anfrage steht im Protokoll
fn fake_u64(password: &'static str, script: Vec<[u8; 8]>) -> (String, Arc<Mutex<Vec<String>>>) { fake_u64_with(password, script, None) }

/// Gerätedaten des Scheingeräts: Produkt für `/v1/info` und die Werte, die `machine:debugreg` nacheinander liefert (der letzte bleibt);
/// `None` bei den Werten: das Gerät antwortet auf `debugreg` mit 501 (wie die Ultimate II+)
type Dev = (&'static str, Option<Vec<u8>>);

fn fake_u64_with(password: &'static str, script: Vec<[u8; 8]>, dev: Option<Dev>) -> (String, Arc<Mutex<Vec<String>>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port();
    let log = Arc::new(Mutex::new(vec![])); let lg = log.clone();
    std::thread::spawn(move || {
        let mut reads = 0usize; let mut regs = 0usize;
        for s in l.incoming() {
            let mut s = s.unwrap(); let (mut buf, mut tmp) = (vec![], [0u8; 8192]);
            let (head_end, len) = loop {
                let n = s.read(&mut tmp).unwrap(); if n == 0 { break (0, 0); } buf.extend_from_slice(&tmp[..n]);
                if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..p]).to_ascii_lowercase();
                    let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
                    if buf.len() >= p + 4 + cl { break (p, cl); }
                }
            };
            if head_end == 0 { continue; }
            let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
            let _ = len;
            let mut first = head.lines().next().unwrap().split_whitespace(); let (method, target) = (first.next().unwrap().to_string(), first.next().unwrap().to_string());
            let pw = head.lines().find_map(|l| l.strip_prefix("X-Password: ")).map(String::from);
            lg.lock().unwrap().push(format!("{method} {target} pw={}", pw.as_deref().unwrap_or("-")));
            if pw.as_deref() != Some(password) { s.write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n").unwrap(); continue; }
            let (path, q) = target.split_once('?').unwrap_or((&target, ""));
            if path == "/v1/machine:debugreg" {
                match dev.as_ref().and_then(|d| d.1.as_ref()) {
                    Some(v) => { let b = format!("{{\"value\" : \"{:02X}\", \"errors\" : []}}", v[regs.min(v.len() - 1)]); regs += 1; s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}", b.len()).as_bytes()).unwrap(); }
                    None => s.write_all(b"HTTP/1.1 501 Not Implemented\r\nConnection: close\r\n\r\n").unwrap(),
                }
                continue;
            }
            let out: Vec<u8> = match path {
                "/v1/info" if dev.is_some() => format!("{{ \"product\" : \"{}\", \"firmware_version\" : \"3.14d\", \"hostname\" : \"Test\", \"core_version\" : \"1.49\", \"errors\" : [] }}", dev.as_ref().unwrap().0).into_bytes(),
                "/v1/version" => b"{\"version\":\"0.1\",\"errors\":[]}".to_vec(),
                "/v1/machine:readmem" => {
                    let a = usize::from_str_radix(q.split('&').find_map(|p| p.strip_prefix("address=")).unwrap(), 16).unwrap();
                    let n: usize = q.split('&').find_map(|p| p.strip_prefix("length=")).unwrap().parse().unwrap();
                    if a == 0x033C { let v = script[reads.min(script.len() - 1)]; reads += 1; v[..n.min(8)].to_vec() } else { vec![0; n] }
                }
                _ => vec![],
            };
            s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", out.len()).as_bytes()).unwrap(); s.write_all(&out).unwrap();
        }
    });
    (format!("127.0.0.1:{port}"), log)
}

const MB_ASM: &str = include_str!("../../../examples/c64/tests_demo/tests_mailbox.asm");
const MB_TESTS: &str = include_str!("../../../examples/c64/tests_demo/tests_mailbox.vctest");
const TRAP_ASM: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.asm");
const TRAP_TESTS: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.vctest");

fn project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-hw-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (n, c) in [("tests_mailbox.asm", MB_ASM), ("tests_mailbox.vctest", MB_TESTS), ("tests_demo.asm", TRAP_ASM), ("tests_demo.vctest", TRAP_TESTS)] { std::fs::write(dir.join(n), c).unwrap(); }
    for a in ["tests_mailbox.asm", "tests_demo.asm"] { assert_eq!(build(&Ctx { root: &dir, data: &data() }, a, Some("c64")).unwrap()["ok"], true); }
    dir
}

fn go(dir: &PathBuf, hw: Hardware) -> Suite {
    let req = RunRequest { root: dir.clone(), data: data(), backend: None, strict: false, only: Only::All, hardware: Some(hw) };
    let (tx, rx) = channel();
    run(&req, Arc::new(AtomicBool::new(false)), &tx);
    drop(tx);
    let mut s = Suite::default(); s.begin(&req.only);
    for e in rx.iter() { s.apply(e); }
    s
}

fn ultimate(host: &str, pw: &str) -> Hardware { Hardware::Ultimate { host: host.into(), password: Some(pw.into()), name: "Testgerät".into() } }
fn group<'a>(s: &'a Suite, name: &str) -> &'a vc_ide::tests::suite::Group { s.groups.iter().find(|g| g.title == name).unwrap_or_else(|| panic!("keine Gruppe {name}: {:?}", s.groups.iter().map(|g| &g.title).collect::<Vec<_>>())) }

#[test]
fn mailbox_test_runs_on_the_device_and_trap_tests_are_skipped() {
    let dir = project("ok");
    let (host, log) = fake_u64("geheim", vec![[0; 8], [1, 1, 0, 0, 0, 0, 0, 0], [2, 4, 0, 0, 0, 0, 0, 0]]);
    let s = go(&dir, ultimate(&host, "geheim"));
    let m = group(&s, "tests_mailbox.vctest");
    assert_eq!(m.backend.as_deref(), Some(format!("Ultimate 64 „Testgerät“ ({host}) · Postfach").as_str()));
    assert_eq!((m.items[0].outcome.state, m.items[0].outcome.measured.as_deref()), (State::Passed, Some("4 Prüfungen")), "{:?}", m.items[0]);
    let t = group(&s, "tests_demo.vctest");
    assert!(t.items.len() == 4 && t.items.iter().all(|i| i.outcome.state == State::Skipped && i.outcome.detail.contains("keine CPU-Register")), "{:?}", t.items);
    let log = log.lock().unwrap().clone();
    // genau ein Programmstart, Passwort immer mitgeschickt, nie Reset oder Pause, am Ende fortgesetzt (Gerät nicht angehalten zurücklassen)
    assert_eq!(log.iter().filter(|l| l.contains("runners:run_prg")).count(), 1, "{log:?}");
    assert!(log.iter().all(|l| l.ends_with("pw=geheim")), "{log:?}");
    assert!(!log.iter().any(|l| l.contains("machine:reset") || l.contains("machine:pause") || l.contains("writemem")), "{log:?}");
    assert!(log.last().unwrap().contains("machine:resume"), "{log:?}");
    assert_eq!(log.iter().filter(|l| l.contains("readmem")).count(), 3, "Postfach wurde dreimal gelesen: {log:?}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn ultimate_64_reports_the_result_through_the_debug_register() {
    let dir = project("reg");
    // Register meldet erst „läuft“, dann „bestanden“; das Postfach wird nur einmal gelesen, wenn das Register fertig meldet
    let (host, log) = fake_u64_with("geheim", vec![[2, 4, 0, 0, 0, 0, 0, 0]], Some(("Ultimate 64-II", Some(vec![0, 1, 1, 2]))));
    let s = go(&dir, ultimate(&host, "geheim"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert_eq!((o.state, o.measured.as_deref()), (State::Passed, Some("4 Prüfungen")), "{o:?}");
    let log = log.lock().unwrap().clone();
    assert!(log.iter().filter(|l| l.contains("debugreg")).count() >= 4, "{log:?}");
    assert_eq!(log.iter().filter(|l| l.contains("readmem")).count(), 1, "Postfach nur nach dem Register gelesen: {log:?}");
    assert!(log.iter().any(|l| l.contains("/v1/info")) && !log.iter().any(|l| l.contains("writemem") || l.contains("machine:reset") || l.contains("debugreg?")), "{log:?}");
    // Fehlschlag über das Register: Einzelheiten kommen weiterhin aus dem Postfach
    let (host, _) = fake_u64_with("geheim", vec![[3, 5, 2, 0x11, 0x22, 1, 0, 0]], Some(("Ultimate 64-II", Some(vec![3]))));
    let s = go(&dir, ultimate(&host, "geheim"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert!(o.is_error() && o.detail.contains("Nr. 2 von 5"), "{o:?}");
    // veralteter Registerwert (2 von einem früheren Lauf), Postfach sagt noch „läuft“: weiter warten, dann bestanden
    let (host, _) = fake_u64_with("geheim", vec![[1, 1, 0, 0, 0, 0, 0, 0], [2, 4, 0, 0, 0, 0, 0, 0]], Some(("Ultimate 64-II", Some(vec![2]))));
    let s = go(&dir, ultimate(&host, "geheim"));
    assert_eq!(group(&s, "tests_mailbox.vctest").items[0].outcome.state, State::Passed);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn without_a_debug_register_the_runner_polls_the_mailbox() {
    let dir = project("noreg");
    // Ultimate II+ (kein „Ultimate 64“): debugreg wird nicht einmal angefragt
    let (host, log) = fake_u64_with("geheim", vec![[1, 1, 0, 0, 0, 0, 0, 0], [2, 4, 0, 0, 0, 0, 0, 0]], Some(("Ultimate II+", None)));
    let s = go(&dir, ultimate(&host, "geheim"));
    assert_eq!(group(&s, "tests_mailbox.vctest").items[0].outcome.state, State::Passed);
    assert!(!log.lock().unwrap().iter().any(|l| l.contains("debugreg")));
    // Ultimate 64, aber die Firmware antwortet auf debugreg mit 501: Rückfall auf das Postfach
    let (host, log) = fake_u64_with("geheim", vec![[2, 4, 0, 0, 0, 0, 0, 0]], Some(("Ultimate 64-II", None)));
    let s = go(&dir, ultimate(&host, "geheim"));
    assert_eq!(group(&s, "tests_mailbox.vctest").items[0].outcome.state, State::Passed);
    assert_eq!(log.lock().unwrap().iter().filter(|l| l.contains("debugreg")).count(), 1);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn failure_timeout_and_wrong_password_on_the_device() {
    let dir = project("bad");
    // fehlgeschlagene Prüfung auf dem Gerät
    let (host, _) = fake_u64("geheim", vec![[3, 5, 2, 0x11, 0x22, 1, 0, 0]]);
    let s = go(&dir, ultimate(&host, "geheim"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert!(o.is_error() && o.detail.contains("Nr. 2 von 5") && o.detail.contains("Ist-Wert 17"), "{o:?}");
    // das Programm endet nie: Zeitüberschreitung, Gerät danach fortgesetzt
    let (host, log) = fake_u64("geheim", vec![[1, 1, 0, 0, 0, 0, 0, 0]]);
    std::fs::write(dir.join("tests_mailbox.vctest"), MB_TESTS.replace("20000", "300")).unwrap();
    let s = go(&dir, ultimate(&host, "geheim"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert_eq!(o.state, State::TimedOut, "{o:?}");
    assert!(log.lock().unwrap().last().unwrap().contains("machine:resume"));
    // falsches Passwort: übersprungen mit Grund, kein Programmstart
    let (host, log) = fake_u64("geheim", vec![[2, 1, 0, 0, 0, 0, 0, 0]]);
    let s = go(&dir, ultimate(&host, "falsch"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert!(o.state == State::Skipped && o.detail.contains("403"), "{o:?}");
    assert!(!log.lock().unwrap().iter().any(|l| l.contains("run_prg")));
    // Gerät nicht erreichbar
    let s = go(&dir, ultimate("127.0.0.1:1", "x"));
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    assert!(o.state == State::Skipped && o.detail.contains("Emulator nicht startbar"), "{o:?}");
    // Zeitlimit zählt in echter Zeit: 300 ms
    std::fs::remove_dir_all(&dir).ok();
}

/// Nur mit ausdrücklich genanntem Gerät (`U64_HOST`) und `--ignored`: lädt das Beispielprogramm, liest das Postfach, setzt das Gerät fort.
#[test]
#[ignore]
fn real_ultimate_runs_the_mailbox_example() {
    let host = std::env::var("U64_HOST").expect("U64_HOST (Adresse der Ultimate) angeben");
    let pw = std::env::var("U64_PASSWORD").ok().filter(|p| !p.is_empty());
    let dir = project("real");
    let s = go(&dir, Hardware::Ultimate { host, password: pw, name: "Hardware".into() });
    let o = &group(&s, "tests_mailbox.vctest").items[0].outcome;
    eprintln!("{o:?}");
    assert_eq!(o.state, State::Passed, "{o:?}");
    std::fs::remove_dir_all(&dir).ok();
}
