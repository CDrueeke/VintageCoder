// SPDX-License-Identifier: GPL-3.0-or-later
//! Dienst „Ströme“ gegen eine Schein-Ultimate: eine REST-Gegenstelle nimmt `:start?ip=` an und sendet echte (aufgezeichnete) Pakete per UDP an den
//! gemeldeten Empfänger, wahlweise mit Verlusten, Umordnung und Pausen. Hardware: `U64_HOST=… cargo test -p vc-ide --test streams -- --ignored --nocapture`
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vc_emu::streams::{channel_sink, StreamKind};
use vc_ide::streams::{StreamOptions, StreamService, StreamState};

/// Ablauf eines Sendevorgangs der Schein-Ultimate
#[derive(Clone, Debug)]
struct Script { /// Pakete, danach Stille
    count: usize, /// Pakete (Index), die nicht gesendet werden
    skip: Vec<usize>, /// je zwei benachbarte Pakete vertauschen
    swap: bool }
impl Script { fn steady() -> Script { Script { count: usize::MAX, skip: vec![], swap: false } } fn silent() -> Script { Script { count: 0, skip: vec![], swap: false } } fn then_silent(n: usize) -> Script { Script { count: n, skip: vec![], swap: false } } }

#[derive(Default)]
struct State { log: Vec<String>, scripts: HashMap<&'static str, Vec<Script>>, senders: HashMap<&'static str, Arc<AtomicBool>> }

struct Mock { host: String, st: Arc<Mutex<State>>, no_link: Arc<AtomicBool>, product: Arc<Mutex<&'static str>> }

fn template(kind: &str) -> Vec<u8> {
    let b = std::fs::read(format!("{}/../vc-emu/tests/data/{kind}.bin", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let n = u16::from_le_bytes([b[0], b[1]]) as usize; b[2..2 + n].to_vec()
}

fn name_of(k: &str) -> &'static str { match k { "video" => "video", "audio" => "audio", _ => "debug" } }

fn mock() -> Mock {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let host = format!("127.0.0.1:{}", l.local_addr().unwrap().port());
    let st: Arc<Mutex<State>> = Default::default(); let no_link = Arc::new(AtomicBool::new(false)); let product = Arc::new(Mutex::new("Ultimate 64-II"));
    let (s2, n2, p2) = (st.clone(), no_link.clone(), product.clone());
    std::thread::spawn(move || {
        for c in l.incoming() {
            let Ok(mut c) = c else { continue };
            let mut buf = vec![0u8; 4096]; let mut n = 0;
            while !buf[..n].windows(4).any(|w| w == b"\r\n\r\n") { match c.read(&mut buf[n..]) { Ok(0) | Err(_) => break, Ok(k) => n += k } }
            let head = String::from_utf8_lossy(&buf[..n]).into_owned();
            let mut it = head.lines().next().unwrap_or("").split_whitespace(); let (method, target) = (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string());
            s2.lock().unwrap().log.push(format!("{method} {target}"));
            let (path, q) = target.split_once('?').unwrap_or((&target, ""));
            let body = match path {
                "/v1/version" => "{\"version\":\"0.1\",\"errors\":[]}".to_string(),
                "/v1/info" => format!("{{ \"product\" : \"{}\", \"firmware_version\" : \"3.14d\", \"errors\" : [ ] }}", p2.lock().unwrap()),
                p if p.starts_with("/v1/streams/") && p.ends_with(":start") => {
                    let kind = name_of(&p["/v1/streams/".len()..p.len() - 6]);
                    if n2.load(Relaxed) { "{ \"errors\" : [ \"No Operational Network Interface\" ] }".to_string() } else {
                        let dest = q.strip_prefix("ip=").unwrap_or("").to_string();
                        let script = { let mut g = s2.lock().unwrap(); let v = g.scripts.entry(kind).or_default(); if v.is_empty() { Script::steady() } else { v.remove(0) } };
                        let stop = Arc::new(AtomicBool::new(false));
                        if let Some(old) = s2.lock().unwrap().senders.insert(kind, stop.clone()) { old.store(true, Relaxed); }
                        let t = template(kind);
                        std::thread::spawn(move || send(&dest, t, script, stop));
                        "{ \"errors\" : [ ] }".to_string()
                    }
                }
                p if p.starts_with("/v1/streams/") && p.ends_with(":stop") => {
                    let kind = name_of(&p["/v1/streams/".len()..p.len() - 5]);
                    if let Some(s) = s2.lock().unwrap().senders.remove(kind) { s.store(true, Relaxed); }
                    "{ \"errors\" : [ ] }".to_string()
                }
                _ => "{ \"errors\" : [ ] }".to_string(),
            };
            let _ = c.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes());
        }
    });
    Mock { host, st, no_link, product }
}

fn send(dest: &str, tpl: Vec<u8>, sc: Script, stop: Arc<AtomicBool>) {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    let _ = s.send_to(&[0, 0], dest);                                    // Startpaket wie an der Hardware
    let mut i = 0usize; let mut held: Option<Vec<u8>> = None;
    while !stop.load(Relaxed) && i < sc.count {
        let mut p = tpl.clone(); p[..2].copy_from_slice(&(100u16.wrapping_add(i as u16)).to_le_bytes());
        if !sc.skip.contains(&i) {
            if sc.swap { match held.take() { None => held = Some(p), Some(h) => { let _ = s.send_to(&p, dest); let _ = s.send_to(&h, dest); } } } else { let _ = s.send_to(&p, dest); }
        }
        i += 1; std::thread::sleep(Duration::from_millis(2));
    }
    while !stop.load(Relaxed) { std::thread::sleep(Duration::from_millis(10)); }   // Stille, bis :stop
}

impl Mock {
    fn script(&self, kind: &'static str, v: Vec<Script>) { self.st.lock().unwrap().scripts.insert(kind, v); }
    fn log(&self) -> Vec<String> { self.st.lock().unwrap().log.clone() }
    fn svc(&self, timeout_ms: u64) -> StreamService { StreamService::new(&self.host, None, StreamOptions { timeout: Duration::from_millis(timeout_ms), ..Default::default() }) }
}

fn until(svc: &mut StreamService, secs: u64, mut f: impl FnMut(&StreamService) -> bool) -> bool {
    let t = Instant::now(); while t.elapsed() < Duration::from_secs(secs) { svc.poll(); if f(svc) { return true; } std::thread::sleep(Duration::from_millis(20)); } false
}
fn is_running(s: &StreamService, k: StreamKind) -> bool { matches!(s.state(k), StreamState::Running | StreamState::Lossy) }
fn count(log: &[String], what: &str) -> usize { log.iter().filter(|l| l.contains(what)).count() }

#[test]
fn start_runs_delivers_raw_packets_and_stop_stops() {
    let m = mock(); let mut svc = m.svc(2000);
    let (sink, rx) = channel_sink(10_000);
    assert_eq!(*svc.state(StreamKind::Video), StreamState::Off);
    svc.start(StreamKind::Video, sink).unwrap();
    assert_eq!(*svc.state(StreamKind::Video), StreamState::Starting);
    assert!(until(&mut svc, 3, |s| is_running(s, StreamKind::Video)), "{:?}", svc.state(StreamKind::Video));
    let st = svc.status(StreamKind::Video); let target = st.target.unwrap();
    assert_eq!(*target.ip(), Ipv4Addr::LOCALHOST); assert_ne!(target.port(), 0);
    assert!(m.log().contains(&format!("PUT /v1/streams/video:start?ip=127.0.0.1:{}", target.port())));
    assert!(st.packets > 0 && st.rejected == 0);
    let p = rx.recv_timeout(Duration::from_secs(1)).unwrap(); assert_eq!(p.data.len(), 780); assert!(StreamKind::Video.check(&p.data).is_ok());
    svc.stop(StreamKind::Video);
    assert_eq!(*svc.state(StreamKind::Video), StreamState::Off);
    assert!(m.log().contains(&"PUT /v1/streams/video:stop".to_string()));
    // nur Strom-Aufrufe und Geräteabfragen: keine Konfiguration, kein Flash
    assert!(m.log().iter().all(|l| l.contains("/v1/streams/") || l.ends_with("/v1/version") || l.ends_with("/v1/info")), "{:?}", m.log());
}

#[test]
fn counterpart_stream_is_stopped_first() {
    let m = mock(); let mut svc = m.svc(2000);
    let (s1, _r1) = channel_sink(100); let (s2, _r2) = channel_sink(100); let (s3, _r3) = channel_sink(100);
    svc.start(StreamKind::Video, s1).unwrap(); svc.start(StreamKind::Audio, s3).unwrap();
    svc.start(StreamKind::Debug, s2).unwrap();
    let log = m.log();
    let stop_video = log.iter().position(|l| l == "PUT /v1/streams/video:stop").expect("Bild gestoppt");
    let start_debug = log.iter().position(|l| l.starts_with("PUT /v1/streams/debug:start")).unwrap();
    assert!(stop_video < start_debug, "{log:?}");
    assert_eq!(*svc.state(StreamKind::Video), StreamState::Off); assert!(svc.is_active(StreamKind::Audio), "Ton läuft weiter"); assert!(svc.is_active(StreamKind::Debug));
    let (s4, _r4) = channel_sink(100); svc.start(StreamKind::Video, s4).unwrap();
    assert_eq!(*svc.state(StreamKind::Debug), StreamState::Off, "und umgekehrt");
}

#[test]
fn no_packets_gives_firewall_hint_and_stops() {
    let m = mock(); m.script("video", vec![Script::silent()]); let mut svc = m.svc(300);
    let (sink, _rx) = channel_sink(10); svc.start(StreamKind::Video, sink).unwrap();
    assert!(until(&mut svc, 3, |s| matches!(s.state(StreamKind::Video), StreamState::Error(_))));
    let StreamState::Error(e) = svc.state(StreamKind::Video).clone() else { unreachable!() };
    assert!(e.contains("keine Pakete") && e.contains("Firewall") && e.contains("Netzwerkprofil") && e.contains("127.0.0.1"), "{e}");
    assert!(!svc.is_active(StreamKind::Video)); assert!(m.log().contains(&"PUT /v1/streams/video:stop".to_string()), "Strom wird beendet");
}

#[test]
fn no_lan_link_is_translated() {
    let m = mock(); m.no_link.store(true, Relaxed); let mut svc = m.svc(300);
    let (sink, _rx) = channel_sink(10);
    let e = svc.start(StreamKind::Audio, sink).unwrap_err();
    assert!(e.starts_with("Kein LAN-Link") && e.contains("WLAN"), "{e}");
    assert_eq!(*svc.state(StreamKind::Audio), StreamState::Error(e)); assert!(!svc.is_active(StreamKind::Audio));
}

#[test]
fn device_without_streams_is_refused() {
    let m = mock(); *m.product.lock().unwrap() = "Ultimate II+"; let mut svc = m.svc(300);
    let (sink, _rx) = channel_sink(10);
    let e = svc.start(StreamKind::Video, sink).unwrap_err(); assert!(e.contains("Ultimate II+") && e.contains("Ultimate 64"), "{e}");
    assert_eq!(count(&m.log(), ":start"), 0, "kein Strom-Aufruf");
}

#[test]
fn unreachable_device_is_an_error_not_a_panic() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let host = format!("127.0.0.1:{}", l.local_addr().unwrap().port()); drop(l);
    let mut svc = StreamService::new(&host, None, StreamOptions::default());
    let (sink, _rx) = channel_sink(10);
    let e = svc.start(StreamKind::Video, sink).unwrap_err(); assert!(e.contains("nicht erreichbar"), "{e}");
    svc.stop_all(); svc.poll();
}

#[test]
fn restarts_once_after_a_break_then_runs_again() {
    let m = mock(); m.script("audio", vec![Script::then_silent(40), Script::steady()]); let mut svc = m.svc(400);
    let (sink, rx) = channel_sink(100_000); svc.start(StreamKind::Audio, sink).unwrap();
    assert!(until(&mut svc, 3, |s| is_running(s, StreamKind::Audio)));
    assert!(until(&mut svc, 5, |s| s.status(StreamKind::Audio).restarts == 1), "Neustart");
    assert!(until(&mut svc, 5, |s| is_running(s, StreamKind::Audio) && s.status(StreamKind::Audio).since_last.map_or(false, |d| d < Duration::from_millis(200)) && s.status(StreamKind::Audio).packets > 0), "{:?}", svc.state(StreamKind::Audio));
    let log = m.log(); assert_eq!(count(&log, "audio:start"), 2); assert!(count(&log, "audio:stop") >= 1);
    assert!(rx.try_iter().count() > 40, "der Sink bleibt über den Neustart erhalten");
    // ein weiterer Abbruch später darf wieder einmal neu starten: Skript ist erschöpft → steady, daher kein weiterer Test
}

#[test]
fn second_break_ends_in_an_error() {
    let m = mock(); m.script("audio", vec![Script::then_silent(40), Script::silent()]); let mut svc = m.svc(300);
    let (sink, _rx) = channel_sink(100_000); svc.start(StreamKind::Audio, sink).unwrap();
    assert!(until(&mut svc, 8, |s| matches!(s.state(StreamKind::Audio), StreamState::Error(_))));
    let StreamState::Error(e) = svc.state(StreamKind::Audio).clone() else { unreachable!() };
    assert!(e.contains("ausgefallen") && e.contains("Neustart"), "{e}");
    assert_eq!(svc.status(StreamKind::Audio).restarts, 1); assert_eq!(count(&m.log(), "audio:start"), 2, "genau ein Neustartversuch");
    assert!(!svc.is_active(StreamKind::Audio));
}

#[test]
fn break_without_recovery_after_running_reports_failure() {
    let m = mock(); m.script("video", vec![Script::then_silent(30), Script::then_silent(0)]); let mut svc = m.svc(300);
    let (sink, _rx) = channel_sink(10_000); svc.start(StreamKind::Video, sink).unwrap();
    assert!(until(&mut svc, 3, |s| is_running(s, StreamKind::Video)));
    assert!(until(&mut svc, 8, |s| matches!(s.state(StreamKind::Video), StreamState::Error(_))));
}

#[test]
fn loss_and_reordering_are_counted() {
    let m = mock();
    m.script("debug", vec![Script { count: 200, skip: (0..200).filter(|i| i % 10 == 3).collect(), swap: false }]);
    let mut svc = m.svc(2000); let (sink, rx) = channel_sink(10_000);
    svc.start(StreamKind::Debug, sink).unwrap();
    assert!(until(&mut svc, 5, |s| s.status(StreamKind::Debug).packets >= 180), "{:?}", svc.status(StreamKind::Debug));
    svc.poll(); assert_eq!(*svc.state(StreamKind::Debug), StreamState::Lossy);
    let st = svc.status(StreamKind::Debug);
    assert_eq!(st.lost, 20, "jedes zehnte Paket fehlt (UDP auf Loopback verliert sonst nichts): {st:?}");
    assert!(st.loss_percent > 5.0 && st.loss_percent < 15.0, "{st:?}");
    assert!(rx.try_iter().any(|p| p.meta.lost_before == 1));
    svc.stop_all();
    // Umordnung
    let m = mock(); m.script("audio", vec![Script { count: 100, skip: vec![], swap: true }]);
    let mut svc = m.svc(2000); let (sink, _rx) = channel_sink(10_000); svc.start(StreamKind::Audio, sink).unwrap();
    assert!(until(&mut svc, 5, |s| s.status(StreamKind::Audio).packets >= 90));
    std::thread::sleep(Duration::from_millis(100)); svc.poll();
    let st = svc.status(StreamKind::Audio); assert!(st.lost <= 1, "nachgelieferte Pakete zählen nicht als Verlust: {st:?}"); assert!(is_running(&svc, StreamKind::Audio));
}

#[test]
fn drop_stops_all_streams() {
    let m = mock();
    { let mut svc = m.svc(2000); let (a, _ra) = channel_sink(10); let (b, _rb) = channel_sink(10); svc.start(StreamKind::Video, a).unwrap(); svc.start(StreamKind::Audio, b).unwrap(); }
    let log = m.log(); assert!(log.contains(&"PUT /v1/streams/video:stop".to_string()) && log.contains(&"PUT /v1/streams/audio:stop".to_string()), "{log:?}");
    assert!(m.st.lock().unwrap().senders.is_empty());
}

#[test]
fn multicast_uses_the_default_group_as_target() {
    let m = mock();
    let mut svc = StreamService::new(&m.host, None, StreamOptions { multicast: true, ..Default::default() });
    let (sink, _rx) = channel_sink(10);
    // Auf Loopback gibt es keine Multicast-Schnittstelle: Der Beitritt darf scheitern, ein Strom-Aufruf darf dann nicht erfolgen.
    match svc.start(StreamKind::Video, sink) {
        Ok(()) => assert!(m.log().contains(&"PUT /v1/streams/video:start?ip=239.0.1.64:11000".to_string()), "{:?}", m.log()),
        Err(e) => { assert!(e.contains("239.0.1.64") || e.contains("Multicast") || e.contains("belegt") || e.contains("Berechtigung"), "{e}"); assert_eq!(count(&m.log(), ":start"), 0); }
    }
}

#[test]
fn fixed_base_port_is_used() {
    let probe = UdpSocket::bind("127.0.0.1:0").unwrap(); let base = probe.local_addr().unwrap().port(); drop(probe);
    let m = mock(); let mut svc = StreamService::new(&m.host, None, StreamOptions { base_port: Some(base), ..Default::default() });
    let (sink, _rx) = channel_sink(10);
    match svc.start(StreamKind::Video, sink) { Ok(()) => assert_eq!(svc.status(StreamKind::Video).target.unwrap().port(), base), Err(e) => assert!(e.contains("belegt"), "{e}") }
}

/// Gegen echte Hardware: Bild und Ton starten, bis zum Laufen warten, stoppen. Nur Strom-Aufrufe, nichts wird gespeichert oder konfiguriert.
#[test]
#[ignore]
fn hardware_service_runs_and_stops() {
    let host = std::env::var("U64_HOST").expect("U64_HOST setzen");
    let pw = std::env::var("U64_PASSWORD").ok();
    let mut svc = StreamService::new(&host, pw.as_deref(), StreamOptions::default());
    let (a, _ra) = channel_sink(100_000); let (b, _rb) = channel_sink(100_000);
    let r1 = svc.start(StreamKind::Video, a); let r2 = svc.start(StreamKind::Audio, b);
    let ok = r1.is_ok() && r2.is_ok() && until(&mut svc, 5, |s| is_running(s, StreamKind::Video) && is_running(s, StreamKind::Audio));
    for k in [StreamKind::Video, StreamKind::Audio] { let s = svc.status(k); println!("{k:?}: {:?} {} Pakete, {:.0}/s, {:.2} Mbit/s, {} fehlen", s.state, s.packets, s.packets_per_s, s.mbit_per_s, s.lost); }
    svc.stop_all();
    r1.unwrap(); r2.unwrap(); assert!(ok);
}

#[test]
fn firewall_advice_adds_the_windows_block_rule_hint_only_on_windows() {
    use vc_ide::streams::{firewall_advice, no_packets_hint_os};
    let w = firewall_advice("vintagecoder", true);
    assert!(w.contains("Privat einstufen") && w.contains("Eingehend-Blockregel für vintagecoder.exe") && w.contains("Zulassen ändern oder löschen") && w.contains("neu starten"), "{w}");
    let o = firewall_advice("vintagecoder", false);
    assert!(!o.contains("Blockregel") && o.contains("Netzwerkprofil „Privat“"), "{o}");
    // im Gesamttext steht der Hinweis an Punkt 1, die übrigen Punkte bleiben
    let t = no_packets_hint_os(vc_emu::streams::StreamKind::Video, None, false, true);
    assert!(t.contains("1. Firewall:") && t.contains("Blockregel") && t.contains("2. Netz:") && t.contains("4. Hilft nichts"), "{t}");
    assert!(!no_packets_hint_os(vc_emu::streams::StreamKind::Video, None, true, false).contains("Blockregel"));
}
