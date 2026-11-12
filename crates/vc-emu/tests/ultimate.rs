// SPDX-License-Identifier: GPL-3.0-or-later
//! Ultimate-64-Adapter gegen einen kleinen Test-HTTP-Server (Passwort, readmem/writemem, pause/resume, run_prg, chunked-Antworten).
//! Gegen echte Hardware: `U64_HOST=192.168.x.y cargo test -p vc-emu --test ultimate -- --ignored --nocapture`
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use vc_emu::{Backend, RunState, Ultimate64};

type Log = Arc<Mutex<Vec<(String, String, usize, Option<String>)>>>;

fn server() -> (String, Log) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port();
    let log: Log = Arc::new(Mutex::new(vec![])); let lg = log.clone();
    std::thread::spawn(move || {
        let mut ram = vec![0u8; 0x10000]; for (i, b) in ram.iter_mut().enumerate() { *b = i as u8; }
        for s in l.incoming() {
            let mut s = s.unwrap(); let mut buf = vec![]; let mut tmp = [0u8; 4096];
            let (head_end, len) = loop {
                let n = s.read(&mut tmp).unwrap(); buf.extend_from_slice(&tmp[..n]);
                if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..p]).to_ascii_lowercase();
                    let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
                    if buf.len() >= p + 4 + cl { break (p, cl); }
                }
            };
            let head = String::from_utf8_lossy(&buf[..head_end]).into_owned(); let body = buf[head_end + 4..head_end + 4 + len].to_vec();
            let mut first = head.lines().next().unwrap().split_whitespace(); let (method, target) = (first.next().unwrap().to_string(), first.next().unwrap().to_string());
            let pw = head.lines().find_map(|l| l.strip_prefix("X-Password: ")).map(String::from);
            lg.lock().unwrap().push((method.clone(), target.clone(), body.len(), pw.clone()));
            let (path, q) = target.split_once('?').unwrap_or((&target, ""));
            let arg = |k: &str| q.split('&').find_map(|p| p.strip_prefix(&format!("{k}="))).map(String::from);
            if pw.as_deref() != Some("geheim") { s.write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n").unwrap(); continue; }
            let out: Vec<u8> = match path {
                "/v1/version" => b"{\"version\":\"0.1\",\"errors\":[]}".to_vec(),
                "/v1/info" => b"{ \"product\" : \"Ultimate 64-II\", \"firmware_version\" : \"3.14d\", \"fpga_version\" : \"122\", \"core_version\" : \"1.49\", \"hostname\" : \"Ultimate-64-II-79F667\", \"unique_id\" : \"B95100\", \"errors\" : [  ] }".to_vec(),
                "/v1/machine:debugreg" => b"{\"value\" : \"2A\", \"errors\" : []}".to_vec(),
                "/v1/machine:readmem" => { let a = usize::from_str_radix(&arg("address").unwrap(), 16).unwrap(); let n: usize = arg("length").unwrap().parse().unwrap(); ram[a..a + n].to_vec() }
                "/v1/machine:writemem" => { let a = usize::from_str_radix(&arg("address").unwrap(), 16).unwrap(); let d: Vec<u8> = match arg("data") { Some(h) => (0..h.len() / 2).map(|i| u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap()).collect(), None => body.clone() }; ram[a..a + d.len()].copy_from_slice(&d); vec![] }
                _ => vec![],
            };
            if path == "/v1/machine:readmem" && out.len() == 3 { // chunked-Test
                s.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n").unwrap(); s.write_all(format!("{:x}\r\n", out.len()).as_bytes()).unwrap(); s.write_all(&out).unwrap(); s.write_all(b"\r\n0\r\n\r\n").unwrap();
            } else { s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", out.len()).as_bytes()).unwrap(); s.write_all(&out).unwrap(); }
        }
    });
    (format!("127.0.0.1:{port}"), log)
}

#[test]
fn talks_rest_to_the_device() {
    let (host, log) = server();
    assert!(Ultimate64::connect(&host, None).unwrap_err().contains("403"), "ohne Passwort");
    let mut u = Ultimate64::connect(&host, Some("geheim")).unwrap();
    assert!(u.version().unwrap().contains("0.1"));
    let c = u.caps(); assert!(!c.registers && !c.step && !c.breakpoints && !c.watch);
    assert_eq!(u.read_mem(0x0400, 4).unwrap(), vec![0x00, 0x01, 0x02, 0x03]);
    assert_eq!(u.read_mem(0x00FE, 3).unwrap(), vec![0xFE, 0xFF, 0x00], "chunked-Antwort");
    assert_eq!(u.read_mem(0x1000, 5000).unwrap().len(), 5000, "mehrere Anfragen à 4096");
    u.write_mem(0x2000, &[1, 2, 3]).unwrap();                              // kurz → PUT mit data=
    u.write_mem(0x3000, &vec![7u8; 300]).unwrap();                         // lang → POST mit Rumpf
    assert_eq!(u.read_mem(0x2000, 3).unwrap(), vec![1, 2, 3]); assert_eq!(u.read_mem(0x3000, 300).unwrap(), vec![7u8; 300]);
    assert_eq!(u.state(), RunState::Running);
    u.pause().unwrap(); assert_eq!(u.state(), RunState::Stopped); u.cont().unwrap(); assert_eq!(u.state(), RunState::Running);
    u.run_prg(&[1, 8, 0, 0]).unwrap();
    u.run_crt(b"C64 CARTRIDGE   ").unwrap();
    let i = u.info().unwrap();
    assert_eq!((i.product.as_str(), i.firmware_version.as_str(), i.hostname.as_str(), i.core_version.as_str()), ("Ultimate 64-II", "3.14d", "Ultimate-64-II-79F667", "1.49"));
    assert_eq!(u.ultimate_caps(), vc_emu::UltimateCaps { streams: true, machine_input: true, debugreg: true, poweroff: true });
    assert_eq!(u.debugreg().unwrap(), 0x2A); assert_eq!(u.debug_reg(), Some(Ok(0x2A)));
    assert!(u.registers().is_err() && u.step().is_err() && u.set_breakpoint(0x0800).is_err() && u.set_watch(0, 1, vc_emu::WatchKind::Write).is_err());
    let l = log.lock().unwrap();
    assert!(l.iter().any(|r| r.0 == "POST" && r.1 == "/v1/runners:run_crt" && r.2 == 16), "run_crt mit Anhang");
    assert!(l.iter().any(|r| r.0 == "GET" && r.1 == "/v1/info"));
    assert!(l.iter().any(|r| r.0 == "PUT" && r.1 == "/v1/machine:writemem?address=2000&data=010203"));
    assert!(l.iter().any(|r| r.0 == "POST" && r.1 == "/v1/machine:writemem?address=3000" && r.2 == 300));
    assert!(l.iter().any(|r| r.0 == "PUT" && r.1 == "/v1/machine:pause") && l.iter().any(|r| r.0 == "POST" && r.1 == "/v1/runners:run_prg" && r.2 == 4));
    assert!(l.iter().filter(|r| r.1 != "/v1/version" || r.3.is_some()).all(|r| r.3.as_deref() == Some("geheim")));
}

#[test]
#[ignore]
fn real_hardware() {
    let Ok(host) = std::env::var("U64_HOST") else { return };
    let mut u = Ultimate64::connect(&host, std::env::var("U64_PASSWORD").ok().as_deref()).unwrap();
    println!("Version: {}", u.version().unwrap());
    let m = u.read_mem(0x0400, 16).unwrap(); println!("$0400: {m:02x?}");
    u.pause().unwrap(); let a = u.read_mem(0xD020, 1).unwrap(); u.cont().unwrap(); println!("$D020 = {a:02x?}");
}

#[test]
fn capabilities_follow_the_product_and_assume_nothing() {
    use vc_emu::{UltimateCaps, UltimateInfo};
    let caps = |p: &str| UltimateInfo { product: p.into(), ..Default::default() }.caps();
    let all = UltimateCaps { streams: true, machine_input: true, debugreg: true, poweroff: true };
    for p in ["Ultimate 64", "Ultimate 64-II", "ultimate-64 elite", "Ultimate 64 Elite-II"] { assert_eq!(caps(p), all, "{p}"); }
    for p in ["Ultimate II+", "Ultimate-II+L", "", "Etwas Anderes", "U64"] { assert_eq!(caps(p), UltimateCaps::default(), "{p}"); }
}

#[test]
fn debug_register_answers_are_parsed_defensively() {
    // über die öffentliche Schnittstelle: ein Gerät, das etwas Unlesbares liefert, ergibt einen Fehler, keinen Absturz
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || for s in l.incoming() {
        let mut s = s.unwrap(); let mut b = [0u8; 2048]; let n = s.read(&mut b).unwrap(); let req = String::from_utf8_lossy(&b[..n]).into_owned();
        let body: &[u8] = if req.contains("/v1/info") { b"{\"product\":\"Ultimate 64\"}" } else if req.contains("debugreg") { b"nix" } else { b"" };
        s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes()).unwrap(); s.write_all(body).unwrap();
    });
    let mut u = Ultimate64::connect(&format!("127.0.0.1:{port}"), None).unwrap();
    assert!(u.debugreg().unwrap_err().contains("nicht lesbar"));
    assert!(matches!(u.debug_reg(), Some(Err(_))));
}

/// Gegen echte Hardware, nur lesend (`GET /v1/info`): `U64_HOST=… cargo test -p vc-emu --test ultimate real_info -- --ignored --nocapture`
#[test]
#[ignore]
fn real_info() {
    let Ok(host) = std::env::var("U64_HOST") else { return };
    let u = Ultimate64::connect(&host, std::env::var("U64_PASSWORD").ok().as_deref()).unwrap();
    let i = u.info().unwrap(); println!("{i:?} -> {:?}", i.caps());
}
