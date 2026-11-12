// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc test`: Ausgabe und Exit-Code der statischen Prüfungen am Beispielprojekt `examples/c64/tests_demo` (ohne Emulator).
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn vcc(dir: &Path, args: &[&str]) -> Output { Command::new(env!("CARGO_BIN_EXE_vcc")).args(args).env("VC_DATA", data()).current_dir(dir).output().unwrap() }
fn text(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr) }

#[test]
fn vcc_test_reports_and_sets_exit_code() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/tests_demo");
    let dir = std::env::temp_dir().join(format!("vc-cli-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["tests_demo.asm", "tests_mailbox.asm", "tests_demo.vcprj", "tests_demo.vctest"] { std::fs::copy(src.join(f), dir.join(f)).unwrap(); }
    // ohne Bauen: Fehler mit Hinweis
    let o = vcc(&dir, &["test"]);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    assert!(text(&o).contains("zuerst bauen"), "{}", text(&o));
    assert!(vcc(&dir, &["build", "."]).status.success());
    // gebaut: grün, der Hinweis zum Zyklenbudget zählt nicht als Fehler
    let o = vcc(&dir, &["test", "."]);
    let t = text(&o);
    assert_eq!(o.status.code(), Some(0), "{t}");
    assert!(t.contains("OK       Zyklen irq_handler ≤ 24  20") && t.contains("HINWEIS  Zyklen next_color ≤ 18") && t.contains("nicht bestimmbar: Schleife"), "{t}");
    assert!(t.contains("ÜBERSPR. next_color: 15 wird 0") && t.contains("--backend vice"), "Emulator-Tests ohne --backend: übersprungen mit Grund\n{t}");
    // --strict: der Hinweis wird ein Fehler
    let o = vcc(&dir, &["test", "--strict"]);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    assert!(text(&o).contains("FEHLER   Zyklen next_color ≤ 18") && text(&o).contains("(--strict)"), "{}", text(&o));
    // einzelne Datei; falsche Eingaben
    assert_eq!(vcc(&dir, &["test", "tests_demo.vctest"]).status.code(), Some(0));
    assert_eq!(vcc(&dir, &["test", "gibtsnicht"]).status.code(), Some(1));
    let o = vcc(&dir, &["test", "--backend", "quatsch"]);
    assert_eq!(o.status.code(), Some(1)); assert!(text(&o).contains("u64:HOST"), "{}", text(&o));
    // atari800 und Xemu kennen nur ihr Ziel: ein c64-Test wird mit Grund übersprungen, ohne dass ein Emulator startet
    for (b, ziel) in [("atari800", "atari8"), ("xemu", "mega65")] {
        let o = vcc(&dir, &["test", "--backend", b]);
        assert!(text(&o).contains(&format!("kennt nur das Ziel {ziel}")) && text(&o).contains("ÜBERSPR."), "{b}: {}", text(&o));
    }
    // leerer Ordner: keine Testdateien, kein Fehler
    let empty = dir.join("leer"); std::fs::create_dir_all(&empty).unwrap();
    let o = vcc(&empty, &["test"]);
    assert_eq!(o.status.code(), Some(0)); assert!(text(&o).contains("Keine Testdateien"));
    std::fs::remove_dir_all(&dir).ok();
}

// ---------- Hardware (Scheingerät für die Ultimate-REST-Schnittstelle), JUnit ----------

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// Scheingerät: Postfach ($033C) meldet zuerst „läuft“, dann „bestanden, 4 Prüfungen“; jede Anfrage steht im Protokoll
fn fake_u64(password: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port();
    let log = Arc::new(Mutex::new(vec![])); let lg = log.clone();
    std::thread::spawn(move || {
        let mut reads = 0;
        for s in l.incoming() {
            let mut s = s.unwrap(); let (mut buf, mut tmp) = (vec![], [0u8; 8192]);
            let (end, _) = loop {
                let n = s.read(&mut tmp).unwrap(); if n == 0 { break (0, 0); } buf.extend_from_slice(&tmp[..n]);
                if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..p]).to_ascii_lowercase();
                    let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
                    if buf.len() >= p + 4 + cl { break (p, cl); }
                }
            };
            if end == 0 { continue; }
            let head = String::from_utf8_lossy(&buf[..end]).into_owned();
            let target = head.lines().next().unwrap().split_whitespace().nth(1).unwrap().to_string();
            let pw = head.lines().find_map(|l| l.strip_prefix("X-Password: ")).map(String::from);
            lg.lock().unwrap().push(format!("{target} pw={}", pw.as_deref().unwrap_or("-")));
            if pw.as_deref() != Some(password) { s.write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n").unwrap(); continue; }
            let (path, q) = target.split_once('?').unwrap_or((&target, ""));
            let out: Vec<u8> = match path {
                "/v1/version" => b"{\"version\":\"0.1\"}".to_vec(),
                "/v1/machine:readmem" => { let a = q.split('&').find_map(|p| p.strip_prefix("address=")).unwrap(); let n: usize = q.split('&').find_map(|p| p.strip_prefix("length=")).unwrap().parse().unwrap();
                    if a == "033C" { reads += 1; let v = if reads < 2 { [1u8, 1, 0, 0, 0, 0, 0, 0] } else { [2, 4, 0, 0, 0, 0, 0, 0] }; v[..n.min(8)].to_vec() } else { vec![0; n] } }
                _ => vec![],
            };
            s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", out.len()).as_bytes()).unwrap(); s.write_all(&out).unwrap();
        }
    });
    (format!("127.0.0.1:{port}"), log)
}

#[test]
fn vcc_test_on_hardware_needs_explicit_device_confirmation_and_writes_junit() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/tests_demo");
    let dir = std::env::temp_dir().join(format!("vc-cli-hw-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["tests_demo.asm", "tests_mailbox.asm", "tests_demo.vcprj", "tests_demo.vctest", "tests_mailbox.vctest"] { std::fs::copy(src.join(f), dir.join(f)).unwrap(); }
    assert!(vcc(&dir, &["build", "."]).status.success());
    let (host, log) = fake_u64("geheim");
    let backend = format!("u64:{host}");
    // ohne Adresse oder ohne Bestätigung: Abbruch, es wird nichts angesprochen
    let o = vcc(&dir, &["test", ".", "--backend", "u64"]);
    assert_eq!(o.status.code(), Some(1)); assert!(text(&o).contains("Adresse"), "{}", text(&o));
    let o = vcc(&dir, &["test", ".", "--backend", &backend, "--password", "geheim"]);
    assert_eq!(o.status.code(), Some(1)); assert!(text(&o).contains("--yes") && text(&o).contains("ACHTUNG"), "{}", text(&o));
    assert!(log.lock().unwrap().is_empty(), "ohne Bestätigung keine Verbindung: {:?}", log.lock().unwrap());
    // mit --yes: Postfach-Test läuft auf dem Gerät, Trap-Tests werden übersprungen, JUnit entsteht
    let junit = dir.join("out.xml");
    let o = vcc(&dir, &["test", ".", "--backend", &backend, "--password", "geheim", "--yes", "--junit", junit.to_str().unwrap()]);
    let t = text(&o);
    assert_eq!(o.status.code(), Some(0), "{t}");
    assert!(t.contains("OK       double, plus3 und Speicher (Selbsttest)  4 Prüfungen") && t.contains("keine CPU-Register"), "{t}");
    let x = std::fs::read_to_string(&junit).unwrap();
    assert!(x.contains("<testsuite name=\"tests_mailbox.vctest\"") && x.contains("<testcase classname=\"tests_mailbox.vctest\" name=\"double, plus3 und Speicher (Selbsttest)\"") && x.contains("<skipped message=\"Trap-Modus nicht verfügbar"), "{x}");
    assert!(x.starts_with("<?xml") && x.trim_end().ends_with("</testsuites>"));
    let log = log.lock().unwrap().clone();
    assert!(log.iter().all(|l| l.ends_with("pw=geheim")) && log.iter().any(|l| l.contains("run_prg")) && log.last().unwrap().contains("machine:resume"), "{log:?}");
    // falsches Passwort: die Postfach-Tests werden übersprungen (Exit-Code 0), nichts wird gestartet
    let (host2, log2) = fake_u64("geheim");
    let o = vcc(&dir, &["test", "tests_mailbox.vctest", "--backend", &format!("u64:{host2}"), "--password", "falsch", "--yes"]);
    assert!(text(&o).contains("403"), "{}", text(&o));
    assert!(!log2.lock().unwrap().iter().any(|l| l.contains("run_prg")));
    std::fs::remove_dir_all(&dir).ok();
}
