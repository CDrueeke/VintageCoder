// SPDX-License-Identifier: GPL-3.0-or-later
//! Dünne Schicht zwischen Oberfläche und Betriebssystem: startet die blockierenden Aufrufe aus `vc_ide::platform` in
//! Hintergrund-Threads und meldet das Ergebnis als [`Event`](super::state::Event) zurück.
use super::state::{emu_key, hw_key, Event, Probe, UiState};
use eframe::egui;
use std::time::Duration;
use vc_ide::platform;
use vc_ide::settings::Machine;
use vc_ide::targets::EmulatorCfg;

fn spawn(st: &mut UiState, ctx: &egui::Context, key: String, work: impl FnOnce() -> Result<String, String> + Send + 'static) {
    st.probes.insert(key.clone(), Probe::Running);
    let (tx, ctx) = (st.tx.clone(), ctx.clone());
    std::thread::spawn(move || { let r = work(); let _ = tx.send(Event::Probe(key, r)); ctx.request_repaint(); });
}

/// Test-Start: der Emulator wird mit seinen Argumenten gestartet und nach kurzer Zeit wieder beendet
pub fn test_emulator(st: &mut UiState, ctx: &egui::Context, e: &EmulatorCfg) {
    let e = e.clone();
    spawn(st, ctx, emu_key(&e.id), move || platform::test_launch(&e));
}

/// Lädt und installiert einen Emulator portabel (nur nach Bestätigung in der Oberfläche); das Ergebnis ist der Pfad des Programms
pub fn install_emulator(st: &mut UiState, ctx: &egui::Context, id: &str) {
    let id = id.to_string();
    spawn(st, ctx, super::install::install_key(&id), move || vc_ide::emulators::install(&id, &mut |_| {}));
}

/// Verbindungstest (Ping bzw. Handshake) mit Zeitlimit
pub fn test_hardware(st: &mut UiState, ctx: &egui::Context, m: &Machine) {
    let m = m.clone();
    spawn(st, ctx, hw_key(&m.id), move || platform::test_hardware(&m, Duration::from_secs(12)).map(|(t, d)| format!("{t} ({} ms)", d.as_millis().max(1))));
}

pub fn scan_ports(st: &mut UiState, ctx: &egui::Context) {
    if st.scanning { return; }
    st.scanning = true;
    let (tx, ctx) = (st.tx.clone(), ctx.clone());
    std::thread::spawn(move || { let _ = tx.send(Event::Ports(platform::scan_serial_ports())); ctx.request_repaint(); });
}

/// Verbindungstest einer MEGA65 über eine bestimmte Verbindung (USB oder Ethernet; Ethernet UNGEPRÜFT)
pub fn test_hardware_link(st: &mut UiState, ctx: &egui::Context, m: &Machine, link: vc_ide::mega65net::Link) {
    let m = m.clone();
    spawn(st, ctx, hw_key(&m.id), move || platform::test_hardware_link(&m, link, Duration::from_secs(30)));
}
