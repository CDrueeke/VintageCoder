// SPDX-License-Identifier: GPL-3.0-or-later
//! Fake-Emulator für Tests (Bildstrom und Oberfläche ohne echten Emulator): liefert ein 64×48-Bild, dessen Inhalt mit jedem Bild wechselt.
//! Hält wie VICE an, solange ein Abruf dauert, und setzt danach fort; Abrufdauer, Fehler und stehende Maschine sind einstellbar.
use crate::streams::video::Frame;
use crate::{Backend, Regs, RunState};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Zähler, die der Test von außen liest (der Emulator selbst liegt hinter dem [`crate::framestream::SharedBackend`])
#[derive(Debug, Default)]
pub struct FakeProbe {
    /// Anzahl der Bildabrufe
    pub grabs: AtomicU64,
    /// Anzahl der Fortsetzungen (`cont`)
    pub conts: AtomicU64,
    /// 1, solange die Maschine angehalten ist
    pub stopped: AtomicU64,
    /// Aufrufe von `stop()`
    pub stops: AtomicU64,
}

pub struct Fake {
    pub probe: Arc<FakeProbe>,
    /// So lange dauert ein Abruf (die Maschine steht währenddessen)
    pub grab_delay: Duration,
    /// Abrufe mit dieser Nummer (ab 1) schlagen fehl; alle danach ebenso, wenn `fail_from_on` gilt
    pub fail_at: Option<u64>,
    pub fail_from_on: bool,
    /// `true`: die Maschine steht dauerhaft (Bild ändert sich nicht)
    pub frozen: bool,
    /// `false`: der Adapter kann kein Bild liefern (`can_grab_frame()` ist falsch)
    pub can_grab: bool,
    running: bool,
    n: u64,
}

impl Fake {
    pub fn new() -> Fake { Fake { probe: Arc::new(FakeProbe::default()), grab_delay: Duration::ZERO, fail_at: None, fail_from_on: false, frozen: false, can_grab: true, running: true, n: 0 } }
    pub const WIDTH: usize = 64;
    pub const HEIGHT: usize = 48;
}

impl Default for Fake { fn default() -> Self { Fake::new() } }

impl Backend for Fake {
    fn name(&self) -> &str { "Fake" }
    fn state(&mut self) -> RunState { if self.running { RunState::Running } else { RunState::Stopped } }
    fn registers(&mut self) -> Result<Regs, String> { Ok(Regs::default()) }
    fn read_mem(&mut self, _addr: u16, len: usize) -> Result<Vec<u8>, String> { Ok(vec![0; len]) }
    fn write_mem(&mut self, _addr: u16, _data: &[u8]) -> Result<(), String> { Ok(()) }
    fn step(&mut self) -> Result<(), String> { Ok(()) }
    fn cont(&mut self) -> Result<(), String> { self.running = true; self.probe.conts.fetch_add(1, Ordering::SeqCst); self.probe.stopped.store(0, Ordering::SeqCst); Ok(()) }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> { self.running = false; self.probe.stopped.store(1, Ordering::SeqCst); Ok(()) }
    fn set_breakpoint(&mut self, _addr: u16) -> Result<(), String> { Ok(()) }
    fn clear_breakpoint(&mut self, _addr: u16) -> Result<(), String> { Ok(()) }
    fn breakpoints(&self) -> Vec<u16> { vec![] }
    fn can_grab_frame(&self) -> bool { self.can_grab }
    fn grab_frame(&mut self) -> Result<Frame, String> {
        let k = self.probe.grabs.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail_at.map_or(false, |f| k == f || (self.fail_from_on && k > f)) { return Err("fake: Abruf fehlgeschlagen".into()); }
        let was_running = self.running;
        self.probe.stopped.store(1, Ordering::SeqCst);
        if !self.grab_delay.is_zero() { std::thread::sleep(self.grab_delay); }
        if !self.frozen && was_running { self.n += 1; }
        let v = (self.n % 251) as u8;
        let mut rgba = Vec::with_capacity(Self::WIDTH * Self::HEIGHT * 4);
        for y in 0..Self::HEIGHT { for x in 0..Self::WIDTH { rgba.extend_from_slice(&[v, (x * 4) as u8, (y * 5) as u8, 255]); } }
        if was_running { self.probe.stopped.store(0, Ordering::SeqCst); }
        Ok(Frame { width: Self::WIDTH, height: Self::HEIGHT, rgba, frame_no: self.n as u16 })
    }
    fn stop(&mut self) { self.probe.stops.fetch_add(1, Ordering::SeqCst); }
}
