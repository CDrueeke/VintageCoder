// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildstrom mit dem Fake-Emulator: Takt, Auslassen statt Aufstauen, Fehler, sauberes Beenden. Läuft ohne echten Emulator.
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vc_emu::fake::{Fake, FakeProbe};
use vc_emu::framestream::{share, FrameStream, NextError, StreamConfig, StreamFrame};

/// Zeitmessende Tests laufen nacheinander (gegenseitige Störung bei parallelem Lauf, Debug-Bau)
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> { SERIAL.lock().unwrap_or_else(|e| e.into_inner()) }

fn fake(f: impl FnOnce(&mut Fake)) -> (vc_emu::framestream::SharedBackend, Arc<FakeProbe>) {
    let mut b = Fake::new();
    f(&mut b);
    let probe = b.probe.clone();
    (share(Box::new(b)), probe)
}

/// Liest bis `dur` verstrichen ist alle Bilder
fn collect(s: &FrameStream, dur: Duration) -> Vec<StreamFrame> {
    let end = Instant::now() + dur;
    let mut v = vec![];
    while Instant::now() < end { if let Ok(f) = s.next(Duration::from_millis(200).min(end.saturating_duration_since(Instant::now()).max(Duration::from_millis(1)))) { v.push(f); } }
    v
}

#[test]
fn steady_rate_without_gaps() {
    let _g = serial();
    let (b, _) = fake(|_| {});
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let v = collect(&s, Duration::from_secs(2));
    let stats = s.stop();
    assert!(v.len() >= 85 && v.len() <= 101, "{} Bilder in 2 s", v.len());
    assert!(v.iter().all(|f| f.dropped == 0), "unerwartete Lücken");
    assert!(v.windows(2).all(|w| w[1].seq == w[0].seq + 1));
    let f = &v[0];
    assert_eq!((f.width, f.height, f.stride, f.data.len()), (64, 48, 256, 64 * 48 * 4));
    // gleichmäßig: Abstände nahe 20 ms
    let gaps: Vec<f64> = v.windows(2).map(|w| (w[1].timestamp - w[0].timestamp).as_secs_f64() * 1000.0).collect();
    let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
    let dev = (gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len() as f64).sqrt();
    assert!((mean - 20.0).abs() < 2.5, "mittlerer Abstand {mean:.2} ms");
    assert!(dev < 6.0, "Streuung {dev:.2} ms");
    assert_eq!(stats.dropped, 0);
}

#[test]
fn slow_grab_skips_instead_of_bursting() {
    let _g = serial();
    // Abruf dauert 45 ms bei 20 ms Takt: gut 20 Bilder pro Sekunde, Rest ausgelassen, nie schneller als der Takt
    let (b, _) = fake(|f| f.grab_delay = Duration::from_millis(45));
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let v = collect(&s, Duration::from_secs(2));
    let stats = s.stop();
    assert!(v.len() >= 28 && v.len() <= 45, "{} Bilder", v.len());
    assert!(v.windows(2).all(|w| (w[1].timestamp - w[0].timestamp) >= Duration::from_millis(40)), "Nachholstoß");
    let lost: u64 = v.iter().map(|f| f.dropped as u64).sum();
    assert!(lost > 0);
    // Lückenfrei zählbar: gelieferte Bilder + gemeldete Auslassungen = Nummernbereich
    let span = v.last().unwrap().seq - v[0].seq + 1;
    assert_eq!(span, v.len() as u64 + v.iter().skip(1).map(|f| f.dropped as u64).sum::<u64>());
    assert!(stats.dropped >= lost);
}

#[test]
fn slow_consumer_gets_latest_and_drops_are_counted() {
    let _g = serial();
    let (b, _) = fake(|_| {});
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let mut got: Vec<StreamFrame> = vec![];
    let end = Instant::now() + Duration::from_secs(2);
    while Instant::now() < end { if let Ok(f) = s.next(Duration::from_millis(500)) { got.push(f); } std::thread::sleep(Duration::from_millis(100)); }
    s.stop();
    assert!(got.len() <= 22 && got.len() >= 12, "{} Bilder", got.len());
    // jede Lücke der Nummern steht in `dropped`
    for w in got.windows(2) { assert_eq!(w[1].seq - w[0].seq - 1, w[1].dropped as u64, "Lücke nicht gemeldet: {} → {}", w[0].seq, w[1].seq); }
    // es wird nichts aufgestaut: das Bild ist höchstens ein Takt alt
    assert!(got.iter().skip(1).all(|f| f.dropped >= 3));
}

#[test]
fn unchanged_frames_are_not_delivered() {
    let _g = serial();
    let (b, _) = fake(|f| f.frozen = true);
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: false }).unwrap();
    let v = collect(&s, Duration::from_millis(600));
    let stats = s.stop();
    assert_eq!(v.len(), 1);
    assert!(v[0].changed);
    assert!(stats.unchanged >= 20, "{stats:?}");
    // mit deliver_unchanged kommen alle, `changed` ist dann falsch
    let (b, _) = fake(|f| f.frozen = true);
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let v = collect(&s, Duration::from_millis(400));
    s.stop();
    assert!(v.len() > 10 && v[0].changed && v.iter().skip(1).all(|f| !f.changed));
}

#[test]
fn config_errors_have_catalog_keys() {
    let _g = serial();
    let (b, _) = fake(|_| {});
    for rate in [0.0, 0.5, 150.0, f64::NAN] {
        let e = FrameStream::start(b.clone(), StreamConfig { rate, deliver_unchanged: false }).err().unwrap();
        assert_eq!(e.key, "diag-emu-stream-bad-rate");
        assert!(e.args.iter().any(|(k, _)| *k == "rate") && !e.message.is_empty() && vc_i18n::has_key(e.key));
    }
    let (b, _) = fake(|f| f.can_grab = false);
    let e = FrameStream::start(b, StreamConfig::default()).err().unwrap();
    assert_eq!(e.key, "diag-emu-stream-no-frames");
}

#[test]
fn repeated_grab_errors_end_the_stream_with_error() {
    let _g = serial();
    let (b, _) = fake(|f| { f.fail_at = Some(5); f.fail_from_on = true; });
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let mut n = 0;
    let e = loop {
        match s.next(Duration::from_secs(3)) { Ok(_) => n += 1, Err(NextError::Ended(e)) => break e, Err(NextError::Timeout) => panic!("kein Ende") }
    };
    let e = e.expect("Fehler erwartet");
    assert_eq!(e.key, "diag-emu-stream-grab-failed");
    assert_eq!(e.args[0].0, "detail");
    assert!(n >= 1 && n <= 4);
    assert!(!s.is_running() || s.next(Duration::from_millis(100)).is_err());
}

#[test]
fn single_grab_error_is_survived() {
    let _g = serial();
    let (b, _) = fake(|f| f.fail_at = Some(3));
    let s = FrameStream::start(b, StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    let v = collect(&s, Duration::from_millis(500));
    let stats = s.stop();
    assert!(v.len() > 15);
    assert_eq!(stats.errors, 1);
}

#[test]
fn stop_is_clean_machine_runs_on_and_emulator_survives() {
    let _g = serial();
    let (b, probe) = fake(|f| f.grab_delay = Duration::from_millis(5));
    let s = FrameStream::start(b.clone(), StreamConfig::default()).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let t = Instant::now();
    s.stop();
    assert!(t.elapsed() < Duration::from_millis(500), "Beenden dauerte {:?}", t.elapsed());
    assert_eq!(probe.stopped.load(Ordering::SeqCst), 0, "Maschine blieb angehalten");
    assert_eq!(probe.stops.load(Ordering::SeqCst), 0, "Strom darf den Emulator nicht beenden");
    let n = probe.grabs.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(probe.grabs.load(Ordering::SeqCst), n, "Faden läuft nach stop weiter");
    // Wegwerfen beendet ebenfalls
    let s = FrameStream::start(b, StreamConfig::default()).unwrap();
    drop(s);
}

#[test]
fn callback_receives_frames_and_end() {
    let _g = serial();
    let (b, _) = fake(|_| {});
    let got = Arc::new(Mutex::new(vec![]));
    let ended = Arc::new(Mutex::new(None));
    let (g2, e2) = (got.clone(), ended.clone());
    let s = FrameStream::start_with(b, StreamConfig { rate: 25.0, deliver_unchanged: true }, move |f| g2.lock().unwrap().push(f.seq), move |e| *e2.lock().unwrap() = Some(e)).unwrap();
    std::thread::sleep(Duration::from_millis(1000));
    s.stop();
    let n = got.lock().unwrap().len();
    assert!(n >= 20 && n <= 26, "{n} Rückrufe");
    assert_eq!(*ended.lock().unwrap(), Some(None));
}

#[test]
fn stream_shares_the_emulator_with_other_users() {
    let _g = serial();
    let (b, _) = fake(|_| {});
    let s = FrameStream::start(b.clone(), StreamConfig { rate: 50.0, deliver_unchanged: true }).unwrap();
    for _ in 0..20 { let mut g = b.lock().unwrap(); g.registers().unwrap(); drop(g); std::thread::sleep(Duration::from_millis(10)); }
    let v = collect(&s, Duration::from_millis(300));
    assert!(v.len() > 5);
}
