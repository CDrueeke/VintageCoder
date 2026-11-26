// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildstrom (Kernarbeit K1): das Emulatorbild in gleichmäßigem Takt statt per Einzelabfrage.
//!
//! Ein Arbeitsfaden holt im festen Takt (`StreamConfig::rate`) über [`Backend::grab_frame`] das Bild und liefert es als [`StreamFrame`]
//! (RGBA, Größe, Zeilenabstand, Bildnummer, Zeitstempel, Auslass-Hinweis). **Auslassen statt Aufstauen:** dauert ein Abruf länger als
//! ein Takt oder liest der Verbraucher zu langsam, werden Bilder verworfen und gezählt ([`StreamFrame::dropped`]); es gibt keine
//! Warteschlange und keinen Nachholstoß. Der Strom ist für jeden Adapter mit `can_grab_frame()` nutzbar (VICE, Fake-Backend, später
//! weitere Emulatoren); er teilt sich den Emulator über [`SharedBackend`] mit dem Rest der Anwendung (Debugger usw.).
//!
//! Zwei Arten, den Strom zu lesen: [`FrameStream::start`] (Verbraucher fragt mit [`FrameStream::next`] das jeweils neueste Bild ab) und
//! [`FrameStream::start_with`] (Rückruf auf dem Arbeitsfaden). Beenden: [`FrameStream::stop`] oder Wegwerfen; beides wartet, bis der
//! Arbeitsfaden fertig ist, und der Emulator läuft danach weiter (ein Abruf setzt die Maschine immer fort).
use crate::Backend;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Der Emulator, geteilt zwischen Bildstrom und der übrigen Anwendung
pub type SharedBackend = Arc<Mutex<Box<dyn Backend>>>;

/// Verpackt einen Adapter zum Teilen mit einem Bildstrom
pub fn share(b: Box<dyn Backend>) -> SharedBackend { Arc::new(Mutex::new(b)) }

/// Kleinste und größte erlaubte Bildrate (Bilder pro Sekunde)
pub const MIN_RATE: f64 = 1.0;
pub const MAX_RATE: f64 = 100.0;
/// So viele Fehlschläge des Abrufs hintereinander beenden den Strom
const MAX_CONSECUTIVE_ERRORS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamConfig {
    /// Zielrate in Bildern pro Sekunde (PAL: 50, NTSC: 60); erlaubt `MIN_RATE..=MAX_RATE`
    pub rate: f64,
    /// `false` (Standard): Bilder, die sich gegenüber dem letzten gelieferten nicht geändert haben (stehende Maschine, stehender Bildschirm),
    /// werden nicht geliefert, sondern nur in [`StreamStats::unchanged`] gezählt; das erste Bild kommt immer.
    pub deliver_unchanged: bool,
}

impl Default for StreamConfig { fn default() -> Self { StreamConfig { rate: 50.0, deliver_unchanged: false } } }

/// Pixelformat der gelieferten Bilder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8 Bit je Kanal in der Reihenfolge R, G, B, A (A immer 255)
    Rgba8,
}

/// Ein Bild des Stroms
#[derive(Debug, Clone, PartialEq)]
pub struct StreamFrame {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    /// Bytes je Zeile (hier `width * 4`, ohne Füllbytes)
    pub stride: usize,
    /// Bilddaten, `stride * height` Bytes
    pub data: Vec<u8>,
    /// Laufende Nummer des Taktes, aus dem das Bild stammt (beginnt bei 1); Lücken zeigen ausgelassene Takte
    pub seq: u64,
    /// Zeit seit dem Start des Stroms, zu der der Abruf fertig war
    pub timestamp: Duration,
    /// Seit dem zuletzt gelieferten Bild ausgelassene Bilder (zu langsamer Abruf oder Verbraucher); 0 = lückenlos
    pub dropped: u32,
    /// Unterscheidet sich der Inhalt vom zuletzt gelieferten Bild? (Beim ersten Bild `true`)
    pub changed: bool,
}

/// Fehler des Stroms, maschinenlesbar mit Katalogschlüssel (`diag-emu-stream-…`) und Parametern, dazu der Text in der Sprache beim Entstehen
#[derive(Debug, Clone, PartialEq)]
pub struct StreamError {
    pub key: &'static str,
    pub args: Vec<(&'static str, String)>,
    pub message: String,
}

impl std::fmt::Display for StreamError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.message) } }
impl std::error::Error for StreamError {}

/// Antwort von [`FrameStream::next`], wenn kein Bild kam
#[derive(Debug, Clone, PartialEq)]
pub enum NextError {
    /// Innerhalb der Frist kam kein neues Bild; der Strom läuft weiter
    Timeout,
    /// Der Strom ist zu Ende (`Some`: wegen eines Fehlers, `None`: gestoppt)
    Ended(Option<StreamError>),
}

/// Zähler des Stroms (Momentaufnahme)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StreamStats {
    /// Takte, in denen ein Bild abgerufen wurde
    pub grabbed: u64,
    /// Bilder, die der Verbraucher bekommen hat
    pub delivered: u64,
    /// Ausgelassene Bilder (Takte übersprungen oder vom Verbraucher nicht abgeholt)
    pub dropped: u64,
    /// Nicht gelieferte, unveränderte Bilder
    pub unchanged: u64,
    /// Fehlgeschlagene Abrufe
    pub errors: u64,
    /// Dauer des letzten Abrufs und Mittel über alle (Millisekunden); in dieser Zeit hält ein laufender VICE an
    pub last_grab_ms: f64,
    pub mean_grab_ms: f64,
    pub max_grab_ms: f64,
}

#[derive(Default)]
struct Slot { frame: Option<StreamFrame>, ended: Option<Option<StreamError>> }

struct Shared {
    slot: Mutex<Slot>,
    ready: Condvar,
    stop: Mutex<bool>,
    stop_cv: Condvar,
    stats: Mutex<StreamStats>,
}

pub struct FrameStream {
    shared: Arc<Shared>,
    thread: Option<std::thread::JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

fn err(key: &'static str, args: Vec<(&'static str, String)>, message: String) -> StreamError { StreamError { key, args, message } }

fn check_config(cfg: &StreamConfig) -> Result<(), StreamError> {
    if !(cfg.rate >= MIN_RATE && cfg.rate <= MAX_RATE) {
        return Err(err("diag-emu-stream-bad-rate", vec![("rate", cfg.rate.to_string()), ("min", MIN_RATE.to_string()), ("max", MAX_RATE.to_string())],
            vc_i18n::t!("diag-emu-stream-bad-rate", rate = cfg.rate, min = MIN_RATE, max = MAX_RATE)));
    }
    Ok(())
}

/// Schneller Inhaltsvergleich (FNV-artig über 8-Byte-Blöcke); gleiche Hashes bei verschiedenen Bildern sind vernachlässigbar selten
fn hash_bytes(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut it = b.chunks_exact(8);
    for c in &mut it { h = (h ^ u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]])).wrapping_mul(0x0000_0100_0000_01b3).rotate_left(5); }
    for &x in it.remainder() { h = (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3); }
    h
}

impl FrameStream {
    /// Strom starten; der Verbraucher holt mit [`FrameStream::next`] jeweils das neueste Bild (ältere, nicht abgeholte werden ausgelassen).
    pub fn start(backend: SharedBackend, cfg: StreamConfig) -> Result<FrameStream, StreamError> {
        let sh = Arc::new(Shared { slot: Mutex::new(Slot::default()), ready: Condvar::new(), stop: Mutex::new(false), stop_cv: Condvar::new(), stats: Mutex::new(StreamStats::default()) });
        let sh2 = sh.clone();
        Self::spawn(backend, cfg, sh, Box::new(move |ev| {
            let mut s = sh2.slot.lock().unwrap();
            match ev {
                Event::Frame(mut f) => {
                    // ein noch nicht abgeholtes Bild wird ersetzt: es zählt als ausgelassen, seine eigenen Auslassungen gehen auf das neue über
                    if let Some(old) = s.frame.take() { f.dropped = f.dropped.saturating_add(old.dropped).saturating_add(1); sh2.stats.lock().unwrap().dropped += 1; }
                    s.frame = Some(f);
                }
                Event::Ended(e) => s.ended = Some(e),
            }
            sh2.ready.notify_all();
        }))
    }

    /// Strom starten mit Rückruf: `sink` läuft auf dem Arbeitsfaden für jedes gelieferte Bild. Ein langsamer Rückruf verzögert den
    /// nächsten Takt; verpasste Takte werden ausgelassen (`dropped`). Das Ende (Fehler oder Stopp) meldet `on_end`.
    pub fn start_with(backend: SharedBackend, cfg: StreamConfig, mut sink: impl FnMut(StreamFrame) + Send + 'static, mut on_end: impl FnMut(Option<StreamError>) + Send + 'static) -> Result<FrameStream, StreamError> {
        let sh = Arc::new(Shared { slot: Mutex::new(Slot::default()), ready: Condvar::new(), stop: Mutex::new(false), stop_cv: Condvar::new(), stats: Mutex::new(StreamStats::default()) });
        Self::spawn(backend, cfg, sh, Box::new(move |ev| match ev { Event::Frame(f) => sink(f), Event::Ended(e) => on_end(e) }))
    }

    fn spawn(backend: SharedBackend, cfg: StreamConfig, sh: Arc<Shared>, mut out: Box<dyn FnMut(Event) + Send>) -> Result<FrameStream, StreamError> {
        check_config(&cfg)?;
        {
            let b = backend.lock().unwrap();
            if !b.can_grab_frame() { return Err(err("diag-emu-stream-no-frames", vec![], vc_i18n::t!("diag-emu-stream-no-frames"))); }
        }
        let stopped = Arc::new(AtomicBool::new(false));
        let (sh2, st2) = (sh.clone(), stopped.clone());
        let th = std::thread::Builder::new().name("vc-frame-stream".into()).spawn(move || run(backend, cfg, sh2, st2, &mut *out))
            .map_err(|e| err("diag-emu-stream-thread", vec![("detail", e.to_string())], vc_i18n::t!("diag-emu-stream-thread", detail = e.to_string())))?;
        Ok(FrameStream { shared: sh, thread: Some(th), stopped })
    }

    /// Wartet bis `timeout` auf das neueste noch nicht abgeholte Bild
    pub fn next(&self, timeout: Duration) -> Result<StreamFrame, NextError> {
        let end = Instant::now() + timeout;
        let mut s = self.shared.slot.lock().unwrap();
        loop {
            if let Some(f) = s.frame.take() { return Ok(f); }
            if let Some(e) = &s.ended { return Err(NextError::Ended(e.clone())); }
            let now = Instant::now();
            if now >= end { return Err(NextError::Timeout); }
            s = self.shared.ready.wait_timeout(s, end - now).unwrap().0;
        }
    }

    /// Das neueste Bild, falls eines wartet (ohne zu warten)
    pub fn try_latest(&self) -> Option<StreamFrame> { self.shared.slot.lock().unwrap().frame.take() }

    pub fn stats(&self) -> StreamStats { *self.shared.stats.lock().unwrap() }

    /// Läuft der Arbeitsfaden noch?
    pub fn is_running(&self) -> bool { self.thread.as_ref().map_or(false, |t| !t.is_finished()) }

    /// Beendet den Strom und wartet auf den Arbeitsfaden (höchstens ein laufender Abruf). Der Emulator bleibt am Leben und läuft weiter.
    pub fn stop(mut self) -> StreamStats { self.shutdown(); self.stats() }

    fn shutdown(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        *self.shared.stop.lock().unwrap() = true;
        self.shared.stop_cv.notify_all();
        if let Some(t) = self.thread.take() { let _ = t.join(); }
    }
}

impl Drop for FrameStream { fn drop(&mut self) { self.shutdown(); } }

enum Event { Frame(StreamFrame), Ended(Option<StreamError>) }

fn run(backend: SharedBackend, cfg: StreamConfig, sh: Arc<Shared>, stopped: Arc<AtomicBool>, out: &mut dyn FnMut(Event)) {
    let period = Duration::from_secs_f64(1.0 / cfg.rate);
    let start = Instant::now();
    let mut next = start;
    let mut seq: u64 = 0;
    let mut dropped_pending: u64 = 0;
    let mut last_hash: Option<u64> = None;
    let mut consecutive_errors = 0u32;
    let mut grab_sum = 0.0f64;
    let finish = |out: &mut dyn FnMut(Event), e: Option<StreamError>| out(Event::Ended(e));
    loop {
        // bis zum Takt warten (unterbrechbar)
        {
            let mut g = sh.stop.lock().unwrap();
            loop {
                if *g { drop(g); return finish(out, None); }
                let now = Instant::now();
                if now >= next { break; }
                g = sh.stop_cv.wait_timeout(g, next - now).unwrap().0;
            }
        }
        // verpasste Takte auslassen statt nachholen
        let now = Instant::now();
        if now > next + period {
            let skipped = ((now - next).as_secs_f64() / period.as_secs_f64()) as u64;
            next += period * skipped as u32;
            seq += skipped;
            dropped_pending += skipped;
            sh.stats.lock().unwrap().dropped += skipped;
        }
        next += period;
        seq += 1;
        let t0 = Instant::now();
        let res = { let mut b = backend.lock().unwrap(); b.grab_frame() };
        let grab_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if stopped.load(Ordering::SeqCst) { return finish(out, None); }
        match res {
            Err(e) => {
                consecutive_errors += 1;
                sh.stats.lock().unwrap().errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return finish(out, Some(err("diag-emu-stream-grab-failed", vec![("detail", e.clone())], vc_i18n::t!("diag-emu-stream-grab-failed", detail = e))));
                }
            }
            Ok(f) => {
                consecutive_errors = 0;
                let h = hash_bytes(&f.rgba);
                let changed = last_hash != Some(h);
                {
                    let mut st = sh.stats.lock().unwrap();
                    st.grabbed += 1;
                    st.last_grab_ms = grab_ms;
                    grab_sum += grab_ms;
                    st.mean_grab_ms = grab_sum / st.grabbed as f64;
                    if grab_ms > st.max_grab_ms { st.max_grab_ms = grab_ms; }
                    if !changed && !cfg.deliver_unchanged { st.unchanged += 1; continue; }
                    st.delivered += 1;
                }
                last_hash = Some(h);
                let frame = StreamFrame { width: f.width, height: f.height, format: PixelFormat::Rgba8, stride: f.width * 4, data: f.rgba, seq, timestamp: start.elapsed(), dropped: std::mem::take(&mut dropped_pending).min(u32::MAX as u64) as u32, changed };
                out(Event::Frame(frame));
            }
        }
    }
}
