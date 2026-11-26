// SPDX-License-Identifier: GPL-3.0-or-later
//! Test-Explorer (Testkonzept T3) der neuen Oberfläche: Zustand und Arbeits-Thread des Laufs. Ergebnismodell und Lauf kommen aus
//! `vc_ide::tests::suite`; die Ansicht steht in `main.rs` (Schublade „Tests“).
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use vc_ide::tests::suite::{self, Event, Only, RunRequest, Suite};
use vc_ide::tests::{Level, Outcome, State, TestFile};

pub struct TestLauf {
    pub suite: Suite,
    /// Zählt jede Änderung der Ergebnisse
    pub version: u64,
    rx: Option<Receiver<Event>>,
    cancel: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// Installierte Emulatoren (Kennung, Name) und die Wahl (`None`: nur statische Prüfungen)
    pub backends: Vec<(String, String)>,
    pub backend: Option<String>,
    pub filter: String,
    pub nur_fehler: bool,
    pub sel: Option<(PathBuf, String)>,
    pub strikt: bool,
    pub gestartet: Option<Instant>,
}

impl Default for TestLauf {
    fn default() -> Self {
        let backends = if cfg!(test) { vec![] } else { suite::available_backends() };
        let backend = backends.first().map(|b| b.0.clone());
        TestLauf { suite: Suite::default(), version: 0, rx: None, cancel: Arc::new(AtomicBool::new(false)), thread: None, backends, backend, filter: String::new(), nur_fehler: false, sel: None, strikt: false, gestartet: None }
    }
}

impl TestLauf {
    pub fn laeuft(&self) -> bool { self.suite.running }
    /// „Tests 13/16“ für die Statusleiste
    pub fn status_text(&self) -> Option<String> { self.suite.status_text() }
    /// Nächster Emulator in der Wahl (Umschalten); am Ende „nur statische Prüfungen“
    pub fn backend_weiter(&mut self) {
        let i = self.backend.as_ref().and_then(|b| self.backends.iter().position(|x| &x.0 == b));
        self.backend = match i { None => self.backends.first().map(|b| b.0.clone()), Some(i) => self.backends.get(i + 1).map(|b| b.0.clone()) };
    }
    pub fn backend_name(&self) -> String {
        self.backend.as_ref().and_then(|b| self.backends.iter().find(|x| &x.0 == b)).map(|b| b.1.clone()).unwrap_or_else(|| vc_i18n::t!("testlauf-nur-statische-pruefungen").into())
    }
    /// Startet einen Lauf im Arbeits-Thread (ein laufender wird zuvor abgebrochen)
    pub fn start(&mut self, root: PathBuf, data: PathBuf, only: Only) {
        self.stop_blockierend();
        let req = RunRequest { root, data, backend: self.backend.clone(), strict: self.strikt, only: only.clone(), hardware: None };
        let (tx, rx) = channel();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        self.suite.begin(&only); self.version += 1; self.rx = Some(rx); self.gestartet = Some(Instant::now());
        self.thread = Some(std::thread::spawn(move || suite::run(&req, cancel, &tx)));
    }
    /// Bricht einen laufenden Lauf ab und wartet (höchstens 10 s), bis der Emulator beendet ist – es läuft immer nur ein Emulator
    pub fn stop_blockierend(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        if let Some(h) = self.thread.take() {
            let ende = Instant::now() + Duration::from_secs(10);
            while !h.is_finished() && Instant::now() < ende { std::thread::sleep(Duration::from_millis(40)); }
            if h.is_finished() { let _ = h.join(); }
        }
        self.abholen(); self.rx = None;
        if self.suite.running { self.suite.running = false; self.version += 1 }
    }
    pub fn abbrechen(&mut self) { self.cancel.store(true, Ordering::SeqCst) }
    /// Ereignisse des Laufs übernehmen (vom Zeitgeber aufgerufen)
    pub fn abholen(&mut self) {
        let Some(rx) = &self.rx else { return };
        let mut geaendert = false;
        loop {
            match rx.try_recv() {
                Ok(e) => { self.suite.apply(e); geaendert = true }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => { if self.suite.running { self.suite.apply(Event::Error(vc_i18n::t!("testlauf-der-testlauf-wurde-unerwartet-beendet").into())); geaendert = true } self.rx = None; break }
            }
        }
        if geaendert { self.version += 1 }
    }
    /// Setup-Zeile des Tests (Aufruf, Speicher, Register) aus der Testdatei, wenn sie sich ermitteln lässt
    pub fn setup_zeile(file: &std::path::Path, name: &str) -> Option<String> {
        let tf = TestFile::load(file).ok()?;
        let (i, raw) = tf.tests.iter().enumerate().find(|(_, t)| t.get("name").and_then(|n| n.as_str()) == Some(name))?;
        let c = vc_ide::tests::TestCase::parse(i, raw).ok()?;
        let mut teile: Vec<String> = c.mem.iter().map(|(a, v)| format!("[{a}]={}", v.join(","))).collect();
        teile.extend(c.regs.iter().map(|(r, v)| format!("{}={v}", r.to_uppercase())));
        teile.extend(c.flags.iter().map(|(f, an)| format!("{}={}", f.to_uppercase(), *an as u8)));
        Some(vc_i18n::t!("testlauf-setup-p1-aufruf-p2-p3", p1 = if teile.is_empty() { vc_i18n::t!("testlauf-keines") } else { teile.join(" ") }, p2 = &c.call, p3 = if c.rti { format!(" {}", vc_i18n::t!("testlauf-rti")) } else { String::new() }))
    }
}

/// Anzeigetext des Zustands
pub fn zustand_text(o: &Outcome) -> String {
    match (o.state, o.level) { (State::Passed, _) => vc_i18n::t!("testlauf-bestanden"), (State::Failed, Level::Error) => vc_i18n::t!("testlauf-fehlgeschlagen"), (State::Failed, Level::Hint) => vc_i18n::t!("testlauf-hinweis"), (State::TimedOut, _) => vc_i18n::t!("testlauf-zeitueberschreitung"), (State::Skipped, _) => vc_i18n::t!("testlauf-uebersprungen") }
}
/// Zeichen vor dem Namen: gefüllt (bestanden, fehlgeschlagen, Hinweis), Uhr (Zeitüberschreitung), Ring (übersprungen)
pub fn zustand_zeichen(o: &Outcome) -> &'static str { match o.state { State::Skipped => "○", State::TimedOut => "◷", _ => "●" } }
pub fn dauer(ms: u64) -> String { if ms < 1000 { vc_i18n::t!("testlauf-ms-ms", ms = &ms) } else { format!("{:.1} s", ms as f64 / 1000.0).replace('.', ",") } }
