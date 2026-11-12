// SPDX-License-Identifier: GPL-3.0-or-later
//! Zustand der Einstellungsseite, der nicht zu den Einstellungen selbst gehört: Auswahl, Filter, laufende und fertige Proben
//! (Test-Start, Verbindungstest), Ergebnis des Port-Scans. Hintergrund-Ergebnisse kommen als [`Event`] über einen Kanal.
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use vc_ide::platform::{self, Presence};
use vc_ide::targets::EmulatorCfg;

/// Ergebnis einer Hintergrundarbeit
pub enum Event {
    /// Test-Start bzw. Verbindungstest (Schlüssel siehe [`emu_key`], [`hw_key`])
    Probe(String, Result<String, String>),
    Ports(Vec<String>),
}

#[derive(Clone)]
pub enum Probe { Running, Ok(String), Err(String) }

pub fn emu_key(id: &str) -> String { format!("emu:{id}") }
pub fn hw_key(id: &str) -> String { format!("hw:{id}") }

pub struct UiState {
    /// Ausgewähltes Zielsystem (Kennung)
    pub system: String,
    pub filter: String,
    pub probes: HashMap<String, Probe>,
    pub ports: Vec<String>,
    /// Überschreibungen des geöffneten Projekts (`None` = kein Projekt offen)
    pub project: Option<Vec<vc_ide::targets::ProjectOverride>>,
    /// Auf „Überschreibung entfernen“ geklickt: System; `app.rs` ändert die Projektdatei
    pub clear_override: Option<String>,
    pub scanning: bool,
    /// Konfigurationsassistent: nichts herunterladen oder installieren (die Installationskarten entfallen)
    pub offline: bool,
    /// Emulator (Kennung), dessen Installation gerade bestätigt werden soll (vor dem Herunterladen)
    pub install_confirm: Option<String>,
    /// gewählte Version je Emulator für „portabel installieren“
    pub install_version: HashMap<String, String>,
    sources: HashMap<String, (Instant, Vec<vc_ide::emulators::Source>)>,
    pub show_pw: HashSet<String>,
    /// zuletzt bearbeitetes Parameterfeld (Emulator, Parameter; leer = freies Feld) für „Variable einfügen“
    pub focus: Option<(String, String)>,
    presence: HashMap<String, (Instant, String, Presence)>,
    pub tx: Sender<Event>,
    rx: Receiver<Event>,
}

impl UiState {
    pub fn new() -> UiState {
        let (tx, rx) = channel();
        UiState { system: String::new(), filter: String::new(), probes: HashMap::new(), ports: platform::scan_serial_ports(), project: None, clear_override: None, scanning: false, offline: false, install_confirm: None, install_version: HashMap::new(), sources: HashMap::new(), show_pw: HashSet::new(), focus: None, presence: HashMap::new(), tx, rx }
    }

    /// Holt fertige Hintergrundergebnisse ab; liefert, ob noch etwas läuft
    pub fn poll(&mut self) -> bool {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Probe(key, r) => { self.probes.insert(key, match r { Ok(t) => Probe::Ok(t), Err(e) => Probe::Err(e) }); }
                Event::Ports(p) => { self.ports = p; self.scanning = false; }
            }
        }
        self.scanning || self.probes.values().any(|p| matches!(p, Probe::Running))
    }

    pub fn probe(&self, key: &str) -> Option<&Probe> { self.probes.get(key) }
    pub fn running(&self, key: &str) -> bool { matches!(self.probes.get(key), Some(Probe::Running)) }

    /// Programm des Emulators gefunden? Das Ergebnis gilt zwei Sekunden, damit nicht in jedem Bild das Dateisystem durchsucht wird.
    pub fn presence(&mut self, e: &EmulatorCfg) -> Presence {
        let sig = format!("{}|{}", e.kind, e.path);
        if let Some((t, s, p)) = self.presence.get(&e.id) { if *s == sig && t.elapsed() < Duration::from_secs(2) { return p.clone(); } }
        let p = platform::presence(e);
        self.presence.insert(e.id.clone(), (Instant::now(), sig, p.clone()));
        p
    }
    pub fn forget_presence(&mut self) { self.presence.clear(); self.sources.clear(); }

    /// Quellen (portable Versionen, System) einer Emulator-Sorte; das Ergebnis gilt zwei Sekunden
    pub fn sources(&mut self, kind: &str) -> Vec<vc_ide::emulators::Source> {
        if let Some((t, v)) = self.sources.get(kind) { if t.elapsed() < Duration::from_secs(2) { return v.clone(); } }
        let v = vc_ide::emulators::sources(kind);
        self.sources.insert(kind.to_string(), (Instant::now(), v.clone()));
        v
    }
}
