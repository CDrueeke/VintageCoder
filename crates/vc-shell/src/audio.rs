// SPDX-License-Identifier: GPL-3.0-or-later
//! Audio-Ausgabe des Trackers (E5.2): ein cpal-Ausgabestrom rendert den Sequenzer `synth::Engine` (vc-ide-model). Der Strom lebt in einem eigenen Thread (cpal-Ströme sind nicht überall `Send`)
//! und wird erst beim ersten Abspielen oder Vorhören geöffnet. Im Testlauf (`VC_TEST_NO_SPAWN`) wird nie ein Gerät geöffnet: der Sequenzer läuft nur im Speicher und wird bei jeder Abfrage
//! um die vergangene Zeit weitergerechnet, damit Position und Pegel prüfbar sind.
use serde_json::{json, Value};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use vc_ide_model::synth::{Engine, Instr, SongData};

/// Größerer Ausgabepuffer als der Standard (oft nur 512 Frames = 10 ms): eine kurze Verzögerung des Audio-Threads ergäbe sonst sofort ein Knacken
const PUFFER: u32 = 2048;

struct Audio { engine: Arc<Mutex<Engine>>, strom: Mutex<Option<Result<(), String>>>, zuletzt: Mutex<Instant> }

fn audio() -> &'static Audio { static A: OnceLock<Audio> = OnceLock::new(); A.get_or_init(|| Audio { engine: Arc::new(Mutex::new(Engine::default())), strom: Mutex::new(None), zuletzt: Mutex::new(Instant::now()) }) }

fn oeffnen(engine: Arc<Mutex<Engine>>) -> Result<(), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::spawn(move || {
        let bauen = || -> Result<cpal::Stream, String> {
            let host = cpal::default_host();
            let dev = host.default_output_device().ok_or_else(|| vc_i18n::t!("audio-kein-audio-ausgabegeraet-gefunden").to_string())?;
            let cfg = dev.default_output_config().map_err(|e| e.to_string())?;
            let mut config: cpal::StreamConfig = cfg.clone().into();
            let versuch = |config: cpal::StreamConfig| -> Result<cpal::Stream, String> {
                let (ch, sr) = (config.channels as usize, u32::from(config.sample_rate) as f32); let engine = engine.clone(); let mut mono: Vec<f32> = vec![];
                let s = dev.build_output_stream(config, move |data: &mut [f32], _| {
                    let frames = data.len() / ch.max(1); mono.resize(frames, 0.0);
                    if let Ok(mut e) = engine.lock() { e.render(&mut mono, sr) } else { mono.iter_mut().for_each(|s| *s = 0.0) }
                    for (i, f) in data.chunks_mut(ch.max(1)).enumerate() { for s in f.iter_mut() { *s = mono[i] } }
                }, |e| eprintln!("Audio-Fehler: {e}"), None).map_err(|e| e.to_string())?;
                s.play().map_err(|e| e.to_string())?; Ok(s)
            };
            if let cpal::SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
                let mut gross = config.clone(); gross.buffer_size = cpal::BufferSize::Fixed(PUFFER.clamp(*min, *max));
                if let Ok(s) = versuch(gross) { return Ok(s) }
            }
            config.buffer_size = cpal::BufferSize::Default; versuch(config)
        };
        match bauen() { Ok(stream) => { let _ = tx.send(Ok(())); loop { std::thread::park(); let _ = &stream; } } Err(e) => { let _ = tx.send(Err(e)); } }
    });
    rx.recv().unwrap_or_else(|_| Err("Audio-Thread beendet".into()))
}

/// Ausgabegerät bei Bedarf öffnen; im Testlauf gilt es als vorhanden. Ein Fehler wird gemerkt (kein erneutes Öffnen bei jedem Ton)
fn bereit() -> Result<(), String> {
    if crate::bau::gesperrt() { return Ok(()) }
    let a = audio(); let mut s = a.strom.lock().unwrap_or_else(|e| e.into_inner());
    if s.is_none() { *s = Some(oeffnen(a.engine.clone())); }
    s.clone().unwrap()
}

pub fn fehler() -> Option<String> { audio().strom.lock().unwrap_or_else(|e| e.into_inner()).clone().and_then(|r| r.err()) }
/// Abspielen ab (Eintrag, Zeile); Fehler: Text, wenn kein Ausgabegerät da ist
pub fn spielen(song: SongData, ord: usize, zeile: usize, stumm: &[bool], solo: &[bool]) -> Result<(), String> {
    bereit()?;
    let a = audio(); *a.zuletzt.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
    if let Ok(mut e) = a.engine.lock() { e.play(song, ord, zeile); for (c, m) in stumm.iter().enumerate() { e.set_mute(c, *m) } for (c, s) in solo.iter().enumerate() { e.set_solo(c, *s) } }
    Ok(())
}
/// Änderungen am Song während der Wiedergabe übernehmen (Position bleibt)
pub fn aktualisieren(song: SongData) { if let Ok(mut e) = audio().engine.lock() { e.update_song(song) } }
pub fn stoppen() { if let Ok(mut e) = audio().engine.lock() { e.stop() } }
pub fn vorhoeren(ins: &Instr, note: u8) { if bereit().is_ok() { if let Ok(mut e) = audio().engine.lock() { e.audition(ins, note) } } }
pub fn spielt() -> bool { audio().engine.lock().is_ok_and(|e| e.playing) }
pub fn stumm_solo(c: usize, stumm: bool, solo: bool) { if let Ok(mut e) = audio().engine.lock() { e.set_mute(c, stumm); e.set_solo(c, solo) } }

/// Zustand für die Oberfläche: spielt, Position, Spitzenpegel je Stimme seit der letzten Abfrage, klingende Noten
pub fn abfragen() -> Value {
    let a = audio();
    if crate::bau::gesperrt() {
        // ohne Gerät: um die verstrichene Zeit weiterrechnen (höchstens eine Sekunde je Abfrage)
        let jetzt = Instant::now(); let dt = { let mut z = a.zuletzt.lock().unwrap_or_else(|e| e.into_inner()); let d = jetzt.duration_since(*z); *z = jetzt; d.as_secs_f32().min(1.0) };
        let frames = (dt * 44100.0) as usize; if frames > 0 { if let Ok(mut e) = a.engine.lock() { let mut puffer = vec![0.0f32; frames]; e.render(&mut puffer, 44100.0) } }
    }
    let Ok(mut e) = a.engine.lock() else { return json!({ "playing": false }) };
    let (playing, pos) = (e.playing, e.pos); let snap = e.viz(1, 1);
    json!({ "playing": playing, "ord": pos.0, "row": pos.1, "peaks": snap.peaks, "master": snap.master_peak, "notes": snap.notes, "error": fehler() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_ide_model::synth::INS_KEEP;
    fn song() -> SongData {
        let ins = Instr::from_json(&json!({ "name": "t", "wave": "pulse", "a": 0, "d": 4, "s": 8, "r": 4 }), false);
        let mut rows = vec![vec![(0u8, INS_KEEP); 2]; 4]; rows[0][0] = (40, 0);
        SongData { pokey: false, channels: 2, rows: 4, order: vec![0], loop_at: usize::MAX, patterns: vec![rows], instruments: vec![ins], row_secs: 0.05, frame_hz: 50.0 }
    }
    #[test] fn abspielen_rechnet_im_testlauf_weiter_und_endet() {
        spielen(song(), 0, 0, &[], &[]).unwrap(); assert!(spielt());
        std::thread::sleep(std::time::Duration::from_millis(60)); let v = abfragen(); assert_eq!(v["playing"], true); assert!(v["row"].as_u64().unwrap() >= 1, "{v}");
        std::thread::sleep(std::time::Duration::from_millis(400)); let v = abfragen(); assert_eq!(v["playing"], false, "Muster einmal, dann Ende: {v}");
        stoppen();
    }
}
