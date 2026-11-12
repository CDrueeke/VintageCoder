// SPDX-License-Identifier: GPL-3.0-or-later
//! Datenströme: Decoder gegen aufgezeichnete Pakete der Ultimate 64 II und der Empfänger gegen einen UDP-Sender auf dem Loopback
//! (mit Verlusten, Umordnung, Pausen, Steuer- und Fehlpaketen). Hardware: `U64_HOST=… cargo test -p vc-emu --test streams -- --ignored --nocapture`
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vc_emu::streams::{audio, channel_sink, debug, video, Delivery, PacketError, ReceiverConfig, StreamKind, StreamReceiver};

/// Datei mit Paketen laden (je u16 Länge little-endian + Bytes)
pub fn read_packets(name: &str) -> Vec<Vec<u8>> {
    let b = std::fs::read(format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let (mut o, mut out) = (0, vec![]);
    while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; out.push(b[o + 2..o + 2 + n].to_vec()); o += 2 + n; }
    assert_eq!(o, b.len(), "Datei {name} abgeschnitten");
    out
}

#[test]
fn real_video_packets_form_a_frame() {
    let ps = read_packets("video.bin"); assert_eq!(ps.len(), 72);
    let hs: Vec<_> = ps.iter().map(|p| video::parse(p).unwrap().header).collect();
    for (i, h) in hs.iter().take(68).enumerate() {
        assert_eq!(h.line as usize, i * 4, "Zeile von Paket {i}"); assert_eq!(h.frame, hs[0].frame, "ein Bild");
        assert_eq!(h.last, i == 67, "nur das 68. Paket ist das letzte"); assert_eq!(h.seq, hs[0].seq.wrapping_add(i as u16));
    }
    assert_eq!(hs[68].line, 0); assert_eq!(hs[68].frame, hs[0].frame.wrapping_add(1), "nächstes Bild");
    assert!(ps.iter().all(|p| p.len() == video::PACKET_LEN));
    assert!(ps.iter().take(68).any(|p| video::parse(p).unwrap().pixels.iter().any(|&b| b != 0)), "Bild enthält Pixel ungleich 0");
    assert!(ps.iter().all(|p| StreamKind::Video.check(p).is_ok()));
}

#[test]
fn real_audio_packets_decode() {
    let ps = read_packets("audio.bin"); assert_eq!(ps.len(), 49);
    let mut prev = None; let mut n = 0;
    for p in &ps {
        let a = audio::parse(p).unwrap();
        if let Some(q) = prev { assert_eq!(a.seq, u16::wrapping_add(q, 1), "lückenlose Zähler"); } prev = Some(a.seq);
        n += a.frames().count();
    }
    assert_eq!(n, 49 * 192);
}

#[test]
fn real_debug_packets_decode() {
    let ps = read_packets("debug.bin"); assert_eq!(ps.len(), 61);
    let mut prev = None; let mut short = 0;
    for p in &ps {
        let d = debug::parse(p).unwrap();
        if let Some(q) = prev { assert_eq!(d.seq, u16::wrapping_add(q, 1)); } prev = Some(d.seq);
        assert_eq!(d.entries().count(), (p.len() - 4) / 4);
        if p.len() != debug::PACKET_LEN { short += 1; }
    }
    assert!(short >= 1, "die Aufnahme enthält kürzere Pakete");
}

#[test]
fn control_packets_and_garbage() {
    for k in StreamKind::ALL { assert_eq!(k.check(&[0, 0]), Err(PacketError::Control), "{k:?}"); assert!(k.check(&[]).is_err()); assert!(k.check(&[1]).is_err()); assert!(k.check(&vec![0xFF; 9999]).is_err()); }
    // jede Länge bis über das Maximum: nie Panik
    let junk: Vec<u8> = (0..2000).map(|i| (i * 7) as u8).collect();
    for n in 0..=junk.len() { for k in StreamKind::ALL { let _ = k.check(&junk[..n]); } }
}

fn rx_cfg(kind: StreamKind) -> ReceiverConfig { ReceiverConfig { kind, delivery: Delivery::Unicast { local: Ipv4Addr::LOCALHOST, port: 0 }, source: None, rcvbuf: 0 } }
fn with_seq(p: &[u8], seq: u16) -> Vec<u8> { let mut v = p.to_vec(); v[..2].copy_from_slice(&seq.to_le_bytes()); v }
fn wait(mut f: impl FnMut() -> bool) -> bool { let t = Instant::now(); while t.elapsed() < Duration::from_secs(3) { if f() { return true; } std::thread::sleep(Duration::from_millis(5)); } false }

#[test]
fn receiver_counts_loss_reorder_pause_and_rejects() {
    let (sink, rx_ch) = channel_sink(1000);
    let rx = StreamReceiver::start(rx_cfg(StreamKind::Audio), sink).unwrap();
    assert!(rx.recv_buffer() > 0);
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap(); let to = rx.local_addr();
    let base = read_packets("audio.bin");
    assert_eq!(rx.stats().since_last, None);
    tx.send_to(&[0, 0], to).unwrap();                                         // Steuerpaket
    tx.send_to(&[1, 2, 3], to).unwrap();                                      // Müll
    tx.send_to(&vec![0u8; 769], to).unwrap();                                 // falsche Länge
    for s in [10u16, 11, 12] { tx.send_to(&with_seq(&base[0], s), to).unwrap(); }
    tx.send_to(&with_seq(&base[0], 15), to).unwrap();                         // 13 und 14 fehlen
    tx.send_to(&with_seq(&base[0], 13), to).unwrap();                         // 13 kommt verspätet
    std::thread::sleep(Duration::from_millis(150));                           // Pause
    for s in [16u16, 17] { tx.send_to(&with_seq(&base[0], s), to).unwrap(); }
    assert!(wait(|| rx.stats().packets == 7));
    let st = rx.stats();
    assert_eq!((st.packets, st.lost, st.late, st.rejected, st.control), (7, 1, 1, 2, 1), "{st:?}");
    assert_eq!(st.bytes, 7 * 770); assert!(st.since_last.unwrap() < Duration::from_secs(1));
    assert!((st.loss_percent() - 100.0 / 8.0).abs() < 0.01);
    let got: Vec<_> = rx_ch.try_iter().map(|p| (p.meta.seq, p.meta.lost_before, p.meta.late)).collect();
    assert_eq!(got, vec![(10, 0, false), (11, 0, false), (12, 0, false), (15, 2, false), (13, 0, true), (16, 0, false), (17, 0, false)], "{got:?}");
}

#[test]
fn receiver_stops_cleanly_and_frees_the_port() {
    let (sink, _rx) = channel_sink(10);
    let rx = StreamReceiver::start(rx_cfg(StreamKind::Debug), sink).unwrap();
    let addr = rx.local_addr(); assert_ne!(addr.port(), 0);
    let t = Instant::now(); rx.stop(); assert!(t.elapsed() < Duration::from_secs(2), "Thread beendet sich zügig");
    UdpSocket::bind(addr).expect("Port nach dem Beenden frei");
}

#[test]
fn bind_errors_are_readable() {
    let (sink, _rx) = channel_sink(10);
    let first = StreamReceiver::start(rx_cfg(StreamKind::Video), sink.clone()).unwrap();
    let port = first.local_addr().port();
    let busy = ReceiverConfig { delivery: Delivery::Unicast { local: Ipv4Addr::LOCALHOST, port }, ..rx_cfg(StreamKind::Video) };
    let e = StreamReceiver::start(busy, sink).unwrap_err();
    assert!(e.contains("belegt") && e.contains(&port.to_string()), "{e}");
}

#[test]
fn receiver_filters_by_source_and_shares_callback() {
    let seen = Arc::new(Mutex::new(0usize)); let s2 = seen.clone();
    let sink: vc_emu::streams::Sink = Arc::new(move |_, _| { *s2.lock().unwrap() += 1; });
    let cfg = ReceiverConfig { source: Some(Ipv4Addr::new(10, 9, 8, 7)), ..rx_cfg(StreamKind::Audio) };
    let rx = StreamReceiver::start(cfg, sink).unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
    tx.send_to(&with_seq(&read_packets("audio.bin")[0], 1), rx.local_addr()).unwrap();
    assert!(wait(|| rx.stats().foreign == 1)); assert_eq!(*seen.lock().unwrap(), 0); assert_eq!(rx.stats().packets, 0);
}

#[test]
fn local_ip_towards_loopback() {
    assert_eq!(vc_emu::streams::local_ip_towards("127.0.0.1").unwrap(), Ipv4Addr::LOCALHOST);
    assert_eq!(vc_emu::streams::local_ip_towards("127.0.0.1:8080").unwrap(), Ipv4Addr::LOCALHOST);
    assert!(vc_emu::streams::local_ip_towards("::1x").is_err());
}

/// Gegen echte Hardware (nur Strom-Aufrufe, danach gestoppt): Pakete kommen an und lassen sich dekodieren
#[test]
#[ignore]
fn hardware_streams_arrive_and_decode() {
    let host = std::env::var("U64_HOST").expect("U64_HOST setzen");
    let pw = std::env::var("U64_PASSWORD").ok();
    let dev = vc_emu::Ultimate64::connect(&host, pw.as_deref()).unwrap();
    let local = vc_emu::streams::local_ip_towards(&host).unwrap();
    for kind in [StreamKind::Video, StreamKind::Audio, StreamKind::Debug] {
        let (sink, ch) = channel_sink(100_000);
        let rx = StreamReceiver::start(ReceiverConfig { kind, delivery: Delivery::Unicast { local, port: 0 }, source: None, rcvbuf: 0 }, sink).unwrap();
        let r = dev.stream_start(kind, local, rx.local_addr().port());
        let ok = r.is_ok() && wait(|| rx.stats().packets >= 200);
        let _ = dev.stream_stop(kind);
        r.unwrap();
        assert!(ok, "{kind:?}: zu wenige Pakete: {:?}", rx.stats());
        let st = rx.stats(); println!("{kind:?}: {} Pakete, {} fehlen, {} verworfen", st.packets, st.lost, st.rejected);
        assert_eq!(st.rejected, 0, "{kind:?}: Pakete mit unplausiblem Kopf");
        for p in ch.try_iter() { assert!(kind.check(&p.data).is_ok()); }
    }
}

#[test]
fn real_video_packets_assemble_to_pal_frame() {
    use vc_emu::streams::video::FrameAssembler;
    let ps = read_packets("video.bin");
    let mut a = FrameAssembler::new();
    let mut frames = vec![];
    for p in &ps { if let Some(f) = a.push(p).unwrap() { frames.push(f); } }
    assert!(!frames.is_empty());
    let f = &frames[0];
    assert_eq!((f.width, f.height), (384, 272));
    assert!(f.rgba.chunks(4).all(|px| px[3] == 255));
    assert!(f.rgba.chunks(4).any(|px| px[..3] != f.rgba[..3]), "Bild nicht einfarbig");
}

#[test]
fn real_audio_goes_through_player_and_wav() {
    use vc_emu::streams::{audio::{decode, StreamPlayer, RATE_PAL}, wav};
    let ps = read_packets("audio.bin");
    let mut all = vec![]; let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
    for p in &ps { let mut f = vec![]; let seq = decode(p, &mut f).unwrap(); all.extend(f.iter().copied()); pl.push(seq, &f); }
    assert_eq!(all.len(), 49 * 192);
    assert!(all.iter().all(|(l, r)| l.abs() <= 1.0 && r.abs() <= 1.0));
    assert!(pl.started(), "49 Pakete (196 ms) reichen zum Start");
    let mut out = vec![(0.0f32, 0.0f32); 4410]; pl.pull(&mut out);
    let peak = out.iter().map(|f| f.0.abs().max(f.1.abs())).fold(0.0f32, f32::max);
    let src_peak = all.iter().map(|f| f.0.abs().max(f.1.abs())).fold(0.0f32, f32::max);
    assert!(peak <= src_peak * 1.3 + 1e-3, "kubische Neuabtastung überschwingt kaum: {peak} gegen {src_peak}");
    // WAV: Quelldaten unverändert hinein und wieder heraus
    let path = std::env::temp_dir().join(format!("vc-real-{}.wav", std::process::id()));
    { let mut w = wav::WavWriter::create(&path, 47983).unwrap(); for p in &ps { w.write(audio::parse(p).unwrap().frames()).unwrap(); } }
    let (rate, back) = wav::read_wav(&std::fs::read(&path).unwrap()).unwrap(); let _ = std::fs::remove_file(&path);
    assert_eq!(rate, 47983); assert_eq!(back, all);
}
