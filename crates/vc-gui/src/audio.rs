// SPDX-License-Identifier: GPL-3.0-or-later
//! Audio-Ausgabe der Hörprobe: ein cpal-Ausgabestrom, der den Sequenzer `synth::Engine` rendert. Der Strom wird erst beim ersten Abspielen geöffnet.
//! Im Testlauf (und mit `VC_TEST_NO_SPAWN=1`) wird nie ein Gerät geöffnet: der Sequenzer läuft nur im Speicher (Zustand und Position sind prüfbar,
//! `schritt` rechnet ihn von Hand weiter) und jeder Aufruf wird über `aussen` aufgezeichnet.
use crate::aussen::{self, Aufruf};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use vc_ide_model::synth::{Engine, Instr, SongData, VizSnap};

pub struct Audio { engine: Arc<Mutex<Engine>>, stream: RefCell<Option<cpal::Stream>>, fehler: RefCell<Option<String>> }

impl Default for Audio { fn default() -> Audio { Audio::neu() } }

impl Audio {
    pub fn neu() -> Audio { Audio { engine: Arc::new(Mutex::new(Engine::default())), stream: RefCell::new(None), fehler: RefCell::new(None) } }

    /// Ausgabegerät bei Bedarf öffnen; im Testlauf gilt es als vorhanden, ohne etwas zu öffnen
    fn bereit(&self) -> bool {
        if aussen::gesperrt() { return true }
        if self.stream.borrow().is_some() { return true }
        if self.fehler.borrow().is_some() { return false }
        match self.oeffnen() { Ok(s) => { *self.stream.borrow_mut() = Some(s); true } Err(e) => { *self.fehler.borrow_mut() = Some(e); false } }
    }
    /// Größerer Ausgabepuffer als der Standard (oft nur 512 Frames = 10 ms): eine kurze Verzögerung des Audio-Threads ergäbe sonst sofort ein Knacken
    const PUFFER: u32 = 2048;
    fn oeffnen(&self) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let dev = host.default_output_device().ok_or(vc_i18n::t!("audio-kein-audio-ausgabegeraet-gefunden"))?;
        let cfg = dev.default_output_config().map_err(|e| e.to_string())?;
        let mut config: cpal::StreamConfig = cfg.clone().into();
        if let cpal::SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
            let frames = Self::PUFFER.clamp(*min, *max); let mut gross = config.clone(); gross.buffer_size = cpal::BufferSize::Fixed(frames);
            if let Ok(s) = self.bauen(&dev, gross) { return Ok(s) }
        }
        config.buffer_size = cpal::BufferSize::Default; self.bauen(&dev, config)
    }
    fn bauen(&self, dev: &cpal::Device, config: cpal::StreamConfig) -> Result<cpal::Stream, String> {
        let (ch, sr) = (config.channels as usize, u32::from(config.sample_rate) as f32); let engine = self.engine.clone(); let mut mono: Vec<f32> = vec![];
        let stream = dev.build_output_stream(config, move |data: &mut [f32], _| {
            let frames = data.len() / ch.max(1); mono.resize(frames, 0.0);
            if let Ok(mut e) = engine.lock() { e.render(&mut mono, sr) } else { mono.iter_mut().for_each(|s| *s = 0.0) }
            for (i, f) in data.chunks_mut(ch.max(1)).enumerate() { for s in f.iter_mut() { *s = mono[i] } }
        }, |e| eprintln!("Audio-Fehler: {e}"), None).map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?; Ok(stream)
    }

    pub fn fehler(&self) -> Option<String> { self.fehler.borrow().clone() }
    pub fn abspielen(&self, song: SongData, ord: usize, zeile: usize, stumm: &[bool], solo: &[bool]) {
        aussen::aufzeichnen(Aufruf::AudioStart { ord, zeile });
        if !self.bereit() { return }
        if let Ok(mut e) = self.engine.lock() { e.play(song, ord, zeile); for (c, m) in stumm.iter().enumerate() { e.set_mute(c, *m) } for (c, s) in solo.iter().enumerate() { e.set_solo(c, *s) } }
    }
    /// Änderungen am Song während der Wiedergabe übernehmen (Momentaufnahme ersetzen, Position bleibt)
    pub fn aktualisieren(&self, song: SongData) { aussen::aufzeichnen(Aufruf::AudioAktualisieren); if self.bereit() { if let Ok(mut e) = self.engine.lock() { e.update_song(song) } } }
    pub fn stoppen(&self) { aussen::aufzeichnen(Aufruf::AudioStopp); if let Ok(mut e) = self.engine.lock() { e.stop() } }
    pub fn vorhoeren(&self, ins: &Instr, note: u8) { aussen::aufzeichnen(Aufruf::AudioVorhoeren { note }); if self.bereit() { if let Ok(mut e) = self.engine.lock() { e.audition(ins, note) } } }
    pub fn spielt(&self) -> bool { self.engine.lock().is_ok_and(|e| e.playing) }
    /// (Eintrag der Reihenfolge, Zeile) der Wiedergabe
    pub fn position(&self) -> (usize, usize) { self.engine.lock().map_or((0, 0), |e| e.pos) }
    pub fn stumm(&self, c: usize, an: bool) { if let Ok(mut e) = self.engine.lock() { e.set_mute(c, an) } }
    pub fn solo(&self, c: usize, an: bool) { if let Ok(mut e) = self.engine.lock() { e.set_solo(c, an) } }
    pub fn stumm_ist(&self, c: usize) -> bool { self.engine.lock().is_ok_and(|e| e.muted(c)) }
    pub fn solo_ist(&self, c: usize) -> bool { self.engine.lock().is_ok_and(|e| e.soloed(c)) }
    /// Pegel seit der letzten Abfrage
    pub fn pegel(&self) -> VizSnap { self.engine.lock().map(|mut e| e.viz(1, 1)).unwrap_or_default() }
    /// Nur Tests: Sequenzer ohne Gerät um `frames` Samples weiterrechnen
    #[cfg(test)]
    pub fn schritt(&self, frames: usize) { let mut puffer = vec![0.0f32; frames]; if let Ok(mut e) = self.engine.lock() { e.render(&mut puffer, 44100.0) } }
}
