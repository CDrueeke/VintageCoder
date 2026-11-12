// SPDX-License-Identifier: GPL-3.0-or-later
//! Audio-Ausgabe der Hörprobe (cpal): ein Ausgabestrom, der den `Engine`-Sequenzer rendert. Wird erst beim ersten Abspielen geöffnet.
use crate::synth::{Engine, Instr, SongData};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vc_emu::streams::audio::{AudioHub, RATE_PAL};

/// Woher der Ton kommt: eingebauter Synthesizer oder Audio-Strom der Ultimate (echter Chip)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source { Synth, Ultimate }

/// Anzeigedaten des Ultimate-Tons (Summe, in Geräterate): Ringpuffer für Oszilloskop und Spektrum und Spitzenpegel
struct UViz { mix: Vec<f32>, w: usize, peak: f32, sr: f32 }
impl Default for UViz { fn default() -> Self { UViz { mix: vec![0.0; crate::synth::RING], w: 0, peak: 0.0, sr: 48000.0 } } }

pub struct Audio {
    engine: Arc<Mutex<Engine>>, stream: RefCell<Option<cpal::Stream>>, error: RefCell<Option<String>>,
    /// `true` = der Rückruf holt den Ton aus `hub` statt aus `engine`
    ultimate: Arc<AtomicBool>, hub: Arc<Mutex<AudioHub>>, uviz: Arc<Mutex<UViz>>,
}

impl Audio {
    pub fn new() -> Audio {
        Audio { engine: Arc::new(Mutex::new(Engine::default())), stream: RefCell::new(None), error: RefCell::new(None), ultimate: Arc::new(AtomicBool::new(false)),
                hub: Arc::new(Mutex::new(AudioHub::new(RATE_PAL, 48000.0))), uviz: Default::default() }
    }
    pub fn source(&self) -> Source { if self.ultimate.load(Ordering::Relaxed) { Source::Ultimate } else { Source::Synth } }
    /// Quelle umschalten; beim Wechsel zum Synthesizer wird der Puffer geleert. Öffnet bei Bedarf das Ausgabegerät.
    pub fn set_source(&self, s: Source) {
        if s == Source::Ultimate { self.stop(); self.ensure(); }
        self.ultimate.store(s == Source::Ultimate, Ordering::Relaxed);
        if s == Source::Synth { if let Ok(mut h) = self.hub.lock() { h.player.reset(); } }
    }
    /// Gemeinsamer Zustand für den Empfänger (Puffer und Aufnahme)
    pub fn hub(&self) -> Arc<Mutex<AudioHub>> { self.hub.clone() }
    /// Puffer des Ultimate-Tons: (Füllstand ms, Lücken gefüllt, Unterläufe, Überläufe, Verhältnis)
    pub fn buffer_stats(&self) -> vc_emu::streams::audio::PlayerStats { self.hub.lock().map(|h| h.player.stats()).unwrap_or_default() }
    /// Aufnahme läuft: (Sekunden, Datei)
    pub fn recording(&self) -> Option<(f64, std::path::PathBuf)> { self.hub.lock().ok().and_then(|h| h.rec.as_ref().map(|w| (w.seconds(), w.path().to_path_buf()))) }
    pub fn start_recording(&self, path: &std::path::Path, rate: u32) -> Result<(), String> {
        let w = vc_emu::streams::wav::WavWriter::create(path, rate)?;
        let mut h = self.hub.lock().map_err(|_| "Puffer gesperrt".to_string())?; h.rec_error = None; h.rec = Some(w); Ok(())
    }
    /// Aufnahme beenden; liefert Datei und Länge in Sekunden
    pub fn stop_recording(&self) -> Option<(std::path::PathBuf, f64)> {
        let mut h = self.hub.lock().ok()?; let mut w = h.rec.take()?; let _ = w.finish(); Some((w.path().to_path_buf(), w.seconds()))
    }
    /// Fehler der Aufnahme (einmalig)
    pub fn take_recording_error(&self) -> Option<String> { self.hub.lock().ok().and_then(|mut h| h.rec_error.take()) }

    fn ensure(&self) -> bool {
        if self.stream.borrow().is_some() { return true; }
        if self.error.borrow().is_some() { return false; }
        match self.open() { Ok(s) => { *self.stream.borrow_mut() = Some(s); true } Err(e) => { *self.error.borrow_mut() = Some(e); false } }
    }
    /// Größerer Ausgabepuffer als der Standard (oft nur 512 Frames = 10 ms): Eine kurze Verzögerung des Audio-Threads, etwa bei ausgelastetem Rechner,
    /// ergäbe sonst sofort ein Knacken. 2048 Frames (rund 43 ms) sind für die Hörprobe noch unmerklich träge.
    const BUFFER_FRAMES: u32 = 2048;

    fn open(&self) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let dev = host.default_output_device().ok_or("kein Audio-Ausgabegerät gefunden")?;
        let cfg = dev.default_output_config().map_err(|e| e.to_string())?;
        let mut config: cpal::StreamConfig = cfg.clone().into();
        if let cpal::SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
            let frames = Self::BUFFER_FRAMES.clamp(*min, *max);
            let mut big = config.clone(); big.buffer_size = cpal::BufferSize::Fixed(frames);
            // Geht der große Puffer nicht, mit dem Standard weitermachen
            match self.build(&dev, big) { Ok(s) => return Ok(s), Err(e) => eprintln!("Audio: Puffer von {frames} Frames nicht möglich ({e}), Standardgröße"), }
        }
        config.buffer_size = cpal::BufferSize::Default;
        self.build(&dev, config)
    }
    fn build(&self, dev: &cpal::Device, config: cpal::StreamConfig) -> Result<cpal::Stream, String> {
        let (ch, sr) = (config.channels as usize, u32::from(config.sample_rate) as f32);
        let engine = self.engine.clone(); let mut mono: Vec<f32> = vec![];
        let (ult, hub, uviz) = (self.ultimate.clone(), self.hub.clone(), self.uviz.clone()); let mut stereo: Vec<vc_emu::streams::audio::Frame> = vec![];
        let stream = dev.build_output_stream(config, move |data: &mut [f32], _| {
            let frames = data.len() / ch.max(1);
            if ult.load(Ordering::Relaxed) {
                stereo.resize(frames, (0.0, 0.0));
                if let Ok(mut h) = hub.lock() { h.player.set_dst_rate(sr as f64); h.player.pull(&mut stereo); } else { stereo.fill((0.0, 0.0)); }
                if let Ok(mut v) = uviz.lock() {
                    v.sr = sr; let n = v.mix.len();
                    for f in &stereo { let m = (f.0 + f.1) * 0.5; let w = v.w; v.mix[w] = m; v.w = (w + 1) % n; v.peak = v.peak.max(f.0.abs().max(f.1.abs())); }
                }
                for (o, f) in data.chunks_mut(ch.max(1)).zip(&stereo) {
                    match o.len() { 1 => o[0] = (f.0 + f.1) * 0.5, _ => { o[0] = f.0; o[1] = f.1; for x in o[2..].iter_mut() { *x = 0.0; } } }
                }
                return;
            }
            mono.resize(frames, 0.0);
            if let Ok(mut e) = engine.lock() { e.render(&mut mono, sr); } else { mono.iter_mut().for_each(|s| *s = 0.0); }
            for (i, f) in data.chunks_mut(ch.max(1)).enumerate() { for s in f.iter_mut() { *s = mono[i]; } }
        }, |e| eprintln!("Audio-Fehler: {e}"), None).map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(stream)
    }
    pub fn error(&self) -> Option<String> { self.error.borrow().clone() }
    pub fn play(&self, song: SongData, ord: usize, row: usize) { if self.ensure() { if let Ok(mut e) = self.engine.lock() { e.play(song, ord, row); } } }
    pub fn stop(&self) { if let Ok(mut e) = self.engine.lock() { e.stop(); } }
    pub fn audition(&self, ins: &Instr, note: u8) { if self.ensure() { if let Ok(mut e) = self.engine.lock() { e.audition(ins, note); } } }
    pub fn playing(&self) -> bool { if self.source() == Source::Ultimate { self.hub.lock().map_or(false, |h| h.player.started()) } else { self.engine.lock().map_or(false, |e| e.playing) } }
    pub fn set_mute(&self, ch: usize, on: bool) { if let Ok(mut e) = self.engine.lock() { e.set_mute(ch, on); } }
    pub fn set_solo(&self, ch: usize, on: bool) { if let Ok(mut e) = self.engine.lock() { e.set_solo(ch, on); } }
    /// (stumm, solo) je Stimme
    pub fn switches(&self, ch: usize) -> (bool, bool) { self.engine.lock().map_or((false, false), |e| (e.muted(ch), e.soloed(ch))) }
    /// Anzeigedaten für Pegel, Oszilloskop und Spektrum (leer, solange nicht gespielt wurde)
    pub fn viz(&self, scope: usize, mix: usize) -> crate::synth::VizSnap {
        if self.source() == Source::Ultimate {
            // Nur die Summe: keine Stimmen, keine Noten
            let Ok(mut v) = self.uviz.lock() else { return Default::default() };
            let n = mix.min(v.mix.len()); let w = v.w; let len = v.mix.len();
            let tail: Vec<f32> = (0..n).map(|i| v.mix[(w + len - n + i) % len]).collect();
            let peak = std::mem::take(&mut v.peak);
            return crate::synth::VizSnap { sr: v.sr, master_peak: peak * 0.55, mix: tail, ..Default::default() };   // 0,55: Anzeige rechnet mit dem Faktor 1,8 (Synthesizer-Pegel)
        }
        self.engine.lock().map(|mut e| e.viz(scope, mix)).unwrap_or_default() }
    pub fn pos(&self) -> (usize, usize) { self.engine.lock().map_or((0, 0), |e| e.pos) }
}
