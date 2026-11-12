// SPDX-License-Identifier: GPL-3.0-or-later
//! Ultimate-REST in der IDE gegen einen Scheinserver: Verbindungstest mit `/v1/info`, Module starten (`run_crt`), Programme (`run_prg`).
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use vc_ide::settings::{test_machine, Machine};

fn server(product: Option<&'static str>) -> (String, Arc<Mutex<Vec<(String, usize)>>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port();
    let log = Arc::new(Mutex::new(vec![])); let lg = log.clone();
    std::thread::spawn(move || for s in l.incoming() {
        let mut s = s.unwrap(); let (mut buf, mut tmp) = (vec![], [0u8; 8192]);
        let (p, cl) = loop {
            let n = s.read(&mut tmp).unwrap(); if n == 0 { break (0, 0); } buf.extend_from_slice(&tmp[..n]);
            if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..p]).to_ascii_lowercase();
                let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
                if buf.len() >= p + 4 + cl { break (p, cl); }
            }
        };
        if p == 0 { continue; }
        let head = String::from_utf8_lossy(&buf[..p]).into_owned(); let mut f = head.lines().next().unwrap().split_whitespace();
        let (m, t) = (f.next().unwrap().to_string(), f.next().unwrap().to_string());
        lg.lock().unwrap().push((format!("{m} {t}"), cl));
        let (status, body): (&str, Vec<u8>) = match (t.as_str(), product) {
            ("/v1/info", Some(pr)) => ("200 OK", format!("{{ \"product\" : \"{pr}\", \"firmware_version\" : \"3.14d\", \"hostname\" : \"Ultimate-64-II-79F667\", \"core_version\" : \"1.49\", \"errors\" : [] }}").into_bytes()),
            ("/v1/info", None) => ("404 Not Found", vec![]),      // ältere Firmware ohne /v1/info
            ("/v1/version", _) => ("200 OK", b"{\"version\":\"0.1\",\"errors\":[]}".to_vec()),
            _ => ("200 OK", vec![]),
        };
        s.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes()).unwrap(); s.write_all(&body).unwrap();
    });
    (format!("127.0.0.1:{port}"), log)
}

fn machine(host: &str) -> Machine {
    let mut m: Machine = serde_json::from_str(r#"{"id":"u1","kind":"ultimate64"}"#).unwrap();
    m.host = host.into(); m.name = "Testgerät".into(); m.run_after = true; m.reset_before = false; m
}

#[test]
fn connection_test_shows_product_firmware_and_hostname() {
    let (host, _) = server(Some("Ultimate 64-II"));
    assert_eq!(test_machine(&machine(&host)).unwrap(), "verbunden: Ultimate 64-II, Firmware 3.14d, Host Ultimate-64-II-79F667");
    // Firmware ohne /v1/info: bisherige Meldung mit der Versionsantwort
    let (host, _) = server(None);
    let t = test_machine(&machine(&host)).unwrap();
    assert!(t.starts_with("verbunden, Firmware-Antwort:") && t.contains("0.1"), "{t}");
}

#[test]
fn modules_start_with_run_crt_and_other_files_are_refused() {
    let (host, log) = server(Some("Ultimate 64-II"));
    let dir = std::env::temp_dir().join(format!("vc-crt-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
    let crt = dir.join("spiel.CRT"); std::fs::write(&crt, vec![0x43u8; 64]).unwrap();
    let prg = dir.join("spiel.prg"); std::fs::write(&prg, [1u8, 8, 0, 0]).unwrap();
    let d64 = dir.join("spiel.d64"); std::fs::write(&d64, [0u8; 8]).unwrap();
    let m = machine(&host);
    let t = vc_ide::launch::run_on_machine(&m, &crt, "c64").unwrap();
    assert!(t.contains("Modul gestartet (64 Bytes)"), "{t}");
    vc_ide::launch::run_on_machine(&m, &prg, "c64").unwrap();
    let e = vc_ide::launch::run_on_machine(&m, &d64, "c64").unwrap_err();
    assert!(e.contains(".prg") && e.contains(".crt"), "{e}");
    assert!(vc_ide::launch::run_on_machine(&m, &crt, "atari8").is_err());
    let log = log.lock().unwrap().clone();
    assert!(log.contains(&("POST /v1/runners:run_crt".to_string(), 64)), "{log:?}");
    assert!(log.contains(&("POST /v1/runners:run_prg".to_string(), 4)), "{log:?}");
    assert_eq!(log.iter().filter(|l| l.0.contains("runners")).count(), 2, "Diskette und falsches Ziel starten nichts: {log:?}");
    std::fs::remove_dir_all(&dir).ok();
}

/// Gegen echte Hardware (setzt die Maschine zurück und startet das Modul): `U64_HOST=… U64_CRT=pfad.crt cargo test -p vc-ide --test ultimate_rest real_ -- --ignored --nocapture`
#[test]
#[ignore]
fn real_ultimate_starts_a_module() {
    let (Ok(host), Ok(crt)) = (std::env::var("U64_HOST"), std::env::var("U64_CRT")) else { return };
    let m = machine(&host);
    println!("{}", test_machine(&m).unwrap());
    println!("{}", vc_ide::launch::run_on_machine(&m, std::path::Path::new(&crt), "c64").unwrap());
}
