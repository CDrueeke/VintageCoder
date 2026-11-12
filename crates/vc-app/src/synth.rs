// SPDX-License-Identifier: GPL-3.0-or-later
//! Software-Synthese für die Hörprobe der Tracker (SID- und POKEY-ähnlich, keine Chip-Emulation).
//! Takt der Hüllkurven: SID-Zeiten wie in der Web-Version (ADSR in ms), POKEY mit Lautstärke je 50-Hz-Bild.
use serde_json::Value;

pub const SID_A: [f32; 16] = [2.0, 8.0, 16.0, 24.0, 38.0, 56.0, 68.0, 80.0, 100.0, 250.0, 500.0, 800.0, 1000.0, 3000.0, 5000.0, 8000.0];
pub const SID_D: [f32; 16] = [6.0, 24.0, 48.0, 72.0, 114.0, 168.0, 204.0, 240.0, 300.0, 750.0, 1500.0, 2400.0, 3000.0, 9000.0, 15000.0, 24000.0];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wave { Triangle, Saw, Pulse, Noise }
impl Wave {
    pub fn parse(s: &str) -> Wave { match s { "saw" => Wave::Saw, "pulse" => Wave::Pulse, "noise" => Wave::Noise, _ => Wave::Triangle } }
    pub fn key(self) -> &'static str { match self { Wave::Triangle => "triangle", Wave::Saw => "saw", Wave::Pulse => "pulse", Wave::Noise => "noise" } }
    pub fn label(self) -> &'static str { match self { Wave::Triangle => "Dreieck", Wave::Saw => "Sägezahn", Wave::Pulse => "Rechteck/Puls", Wave::Noise => "Rauschen" } }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instr {
    Sid { wave: Wave, pw: f32, a: usize, d: usize, s: usize, r: usize },
    Pokey { dist: String, env: Vec<u8>, loop_at: usize },
}

impl Instr {
    pub fn from_json(v: &Value, pokey: bool) -> Instr {
        if pokey {
            Instr::Pokey { dist: v["dist"].as_str().unwrap_or("pure").into(), env: v["env"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0).min(15) as u8).collect()).unwrap_or_else(|| vec![15]), loop_at: v["loop"].as_u64().unwrap_or(15) as usize }
        } else {
            let n = |k: &str, d: u64| v[k].as_u64().unwrap_or(d).min(15) as usize;
            Instr::Sid { wave: Wave::parse(v["wave"].as_str().unwrap_or("pulse")), pw: v["pw"].as_f64().unwrap_or(2048.0) as f32 / 4096.0, a: n("a", 0), d: n("d", 9), s: n("s", 8), r: n("r", 4) }
        }
    }
}

pub fn hz(note: u8) -> f32 { 440.0 * 2f32.powf((note as f32 - 58.0) / 12.0) }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage { Attack, Decay, Release, Off }

#[derive(Clone)]
pub struct Voice {
    ins: Option<Instr>, phase: f32, freq: f32, level: f32, stage: Stage, pos: usize, noise: f32, rng: u32, held: f32,
    /// POKEY: Wellenform und Tastverhältnis der gewählten Verzerrung (einmal beim Start bestimmt, damit `sample` nichts kopieren muss)
    pk_wave: Wave, pk_duty: f32,
}

impl Default for Voice { fn default() -> Voice { Voice { ins: None, phase: 0.0, freq: 440.0, level: 0.0, stage: Stage::Off, pos: 0, noise: 0.0, rng: 0x1234_5678, held: 0.0, pk_wave: Wave::Pulse, pk_duty: 0.5 } } }

impl Voice {
    pub fn active(&self) -> bool { self.stage != Stage::Off }
    pub fn start(&mut self, ins: &Instr, note: u8) {
        let mut f = hz(note);
        if let Instr::Pokey { .. } = ins { let div = ((1_773_447.0 / 28.0 / (2.0 * f)).round() - 1.0).clamp(0.0, 255.0); f = 1_773_447.0 / 28.0 / (2.0 * (div + 1.0)); }
        self.ins = Some(ins.clone()); self.freq = f; self.phase = 0.0; self.pos = 0; self.stage = Stage::Attack;
        self.level = if matches!(ins, Instr::Pokey { .. }) { 1.0 } else { 0.0 };
        if let Instr::Pokey { env, dist, .. } = ins {
            self.held = env.first().copied().unwrap_or(15) as f32 / 15.0 * 0.9;
            (self.pk_wave, self.pk_duty) = match dist.as_str() { "noise" | "gritty" => (Wave::Noise, 0.5), "buzz" => (Wave::Pulse, 0.25), "bass" => (Wave::Pulse, 0.12), _ => (Wave::Pulse, 0.5) };
        }
    }
    pub fn release(&mut self) { if self.stage != Stage::Off { self.stage = Stage::Release; } }
    /// Einmal je 50-Hz-Bild (nur POKEY-Hüllkurve)
    pub fn tick(&mut self) {
        if let (Some(Instr::Pokey { env, loop_at, .. }), true) = (&self.ins, self.stage != Stage::Release && self.stage != Stage::Off) {
            self.held = env[self.pos.min(env.len() - 1)] as f32 / 15.0 * 0.9; self.pos += 1; if self.pos >= 16 { self.pos = *loop_at; }
        }
    }
    fn rand(&mut self) -> f32 { self.rng ^= self.rng << 13; self.rng ^= self.rng >> 17; self.rng ^= self.rng << 5; (self.rng as f32 / u32::MAX as f32) * 2.0 - 1.0 }

    pub fn sample(&mut self, sr: f32) -> f32 {
        // Kein Kopieren des Instruments je Sample: nur die (kopierbaren) Zahlen herausnehmen, sonst gäbe es beim POKEY Speicherzuweisungen im Audio-Thread
        let sid = match &self.ins { Some(Instr::Sid { wave, pw, a, d, s, r }) => Some((*wave, *pw, *a, *d, *s, *r)), Some(Instr::Pokey { .. }) => None, None => return 0.0 };
        if self.stage == Stage::Off { return 0.0; }
        let (wave, duty, gain) = match sid {
            Some((wave, pw, a, d, s, r)) => {
                let (sus, tau_d, tau_r) = (s as f32 / 15.0, SID_D[d] / 3000.0, SID_D[r] / 3000.0);
                match self.stage {
                    Stage::Attack => { self.level += 1.0 / ((SID_A[a] / 1000.0).max(0.0005) * sr); if self.level >= 1.0 { self.level = 1.0; self.stage = Stage::Decay; } }
                    Stage::Decay => { self.level += (sus - self.level) * (1.0 - (-1.0 / (tau_d.max(0.0005) * sr)).exp()); }
                    Stage::Release => { self.level += (0.0 - self.level) * (1.0 - (-1.0 / (tau_r.max(0.0005) * sr)).exp()); if self.level < 0.001 { self.stage = Stage::Off; } }
                    Stage::Off => {}
                }
                (wave, pw.clamp(0.02, 0.98), self.level)
            }
            None => {
                if self.stage == Stage::Release { self.held *= 1.0 - 1.0 / (0.005 * sr); if self.held < 0.001 { self.stage = Stage::Off; } }
                (self.pk_wave, self.pk_duty, self.held)
            }
        };
        let inc = self.freq / sr; self.phase += inc; let wrapped = self.phase >= 1.0; if wrapped { self.phase -= 1.0; }
        let p = self.phase;
        let raw = match wave {
            Wave::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
            Wave::Saw => 2.0 * p - 1.0,
            Wave::Pulse => if p < duty { 1.0 } else { -1.0 },
            Wave::Noise => { if wrapped || self.noise == 0.0 { self.noise = self.rand(); } self.noise }
        };
        raw * gain
    }
}

/// Song-Daten für den Sequenzer
#[derive(Clone)]
pub struct SongData {
    #[allow(dead_code)]
    pub pokey: bool,
    pub channels: usize,
    pub rows: usize,
    pub order: Vec<usize>,
    pub loop_at: usize,
    pub patterns: Vec<Vec<Vec<(u8, u8)>>>,   // Pattern → Zeile → Kanal → (Note, Instrument; 0xFF = behalten)
    pub instruments: Vec<Instr>,
    /// Dauer einer Zeile in Sekunden
    pub row_secs: f32,
    /// Bildrate des Zielsystems (Takt der POKEY-Hüllkurven): 50 (PAL) oder 60 (NTSC)
    pub frame_hz: f32,
}

pub const NOTE_OFF: u8 = 0xFF;
pub const INS_KEEP: u8 = 0xFF;

pub fn parse_cell(c: &str) -> (u8, u8) {
    let t: Vec<&str> = c.split_whitespace().collect();
    let s = t.first().copied().unwrap_or("---");
    let note = if s == "===" { NOTE_OFF } else if s == "---" || s == "..." { 0 } else {
        let names = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];
        names.iter().position(|n| s.len() >= 3 && s[..2].eq_ignore_ascii_case(n)).map_or(0, |k| k as u8 + 12 * s[2..].parse::<u8>().unwrap_or(0) + 1)
    };
    let ins = match t.get(1) { Some(x) if *x != "--" && *x != ".." => u8::from_str_radix(x, 16).unwrap_or(INS_KEEP), _ => INS_KEEP };
    (note, ins)
}
pub fn note_name(n: u8) -> String {
    if n == 0 { "---".into() } else if n == NOTE_OFF { "===".into() } else { format!("{}{}", ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"][((n - 1) % 12) as usize], (n - 1) / 12) }
}
pub fn fmt_cell(n: u8, i: u8) -> String { format!("{} {}", note_name(n), if i == INS_KEEP { "--".to_string() } else { format!("{:02X}", i) }) }

/// Mischt Stimmen und läuft als Sequenzer (Takt über `row_secs`)
pub struct Engine {
    voices: Vec<Voice>, song: Option<SongData>, pub playing: bool, ord: usize, row: usize, samples_in_row: f32, frame_acc: f32, cur_ins: Vec<usize>,
    pub pos: (usize, usize), master: f32, audition: Vec<(usize, f32)>, row_started: bool,
    /// Anzeigedaten (Pegel, Wellenform) und Stumm-/Solo-Schalter je Stimme
    viz: Viz, mute: Vec<bool>, solo: Vec<bool>, voice_note: Vec<u8>, sr: f32,
}

/// Länge der Ringpuffer für Oszilloskop und Spektrum (Samples)
pub const RING: usize = 4096;

/// Mitgeschnittene Signale für die Anzeige: je Stimme eine Wellenform und ein Spitzenpegel, dazu die Summe
#[derive(Default)]
struct Viz { ch: Vec<Vec<f32>>, mix: Vec<f32>, w: usize, peak: Vec<f32>, master_peak: f32 }

/// Momentaufnahme der Anzeigedaten (Wellenformen in zeitlicher Reihenfolge, Pegel seit der letzten Abfrage)
#[derive(Clone, Default, Debug)]
pub struct VizSnap {
    pub sr: f32,
    /// Spitzenpegel je Stimme (0..1) seit der letzten Abfrage
    pub peaks: Vec<f32>,
    pub master_peak: f32,
    /// letzte `len` Samples je Stimme (Amplitude −1..1)
    pub ch_wave: Vec<Vec<f32>>,
    pub mix: Vec<f32>,
    /// gerade klingende Note je Stimme
    pub notes: Vec<Option<u8>>,
}

impl Default for Engine { fn default() -> Engine { Engine { voices: vec![Voice::default(); 16], song: None, playing: false, ord: 0, row: 0, samples_in_row: 0.0, frame_acc: 0.0, cur_ins: vec![], pos: (0, 0), master: 0.18, audition: vec![], row_started: false, viz: Viz::default(), mute: vec![], solo: vec![], voice_note: vec![0; 16], sr: 44100.0 } } }

impl Engine {
    pub fn play(&mut self, song: SongData, ord: usize, row: usize) {
        for v in &mut self.voices { v.release(); }
        self.voices.resize(song.channels.max(16), Voice::default());
        self.viz.ch = vec![vec![0.0; RING]; song.channels]; self.viz.peak = vec![0.0; song.channels]; if self.viz.mix.len() != RING { self.viz.mix = vec![0.0; RING]; }
        self.mute.resize(song.channels, false); self.solo.resize(song.channels, false);
        self.cur_ins = vec![0; song.channels]; self.ord = ord; self.row = row; self.samples_in_row = 0.0; self.frame_acc = 0.0; self.row_started = false;
        self.song = Some(song); self.playing = true; self.pos = (ord, row);
    }
    pub fn stop(&mut self) { self.playing = false; for v in &mut self.voices { v.release(); } }
    /// Stummschalten und Solo je Stimme (wirkt auf Klang und Anzeige)
    pub fn set_mute(&mut self, ch: usize, on: bool) { if self.mute.len() <= ch { self.mute.resize(ch + 1, false); } self.mute[ch] = on; }
    pub fn set_solo(&mut self, ch: usize, on: bool) { if self.solo.len() <= ch { self.solo.resize(ch + 1, false); } self.solo[ch] = on; }
    pub fn muted(&self, ch: usize) -> bool { self.mute.get(ch).copied().unwrap_or(false) }
    pub fn soloed(&self, ch: usize) -> bool { self.solo.get(ch).copied().unwrap_or(false) }
    /// Anzeigedaten holen; die Spitzenpegel beginnen danach von vorn. `scope` = Samples je Stimme, `mix` = Samples der Summe.
    pub fn viz(&mut self, scope: usize, mix: usize) -> VizSnap {
        let w = self.viz.w;
        let tail = |buf: &Vec<f32>, n: usize| -> Vec<f32> { let n = n.min(buf.len()); (0..n).map(|i| buf[(w + buf.len() - n + i) % buf.len()]).collect() };
        let snap = VizSnap {
            sr: self.sr, peaks: self.viz.peak.clone(), master_peak: self.viz.master_peak,
            ch_wave: self.viz.ch.iter().map(|b| tail(b, scope)).collect(), mix: if self.viz.mix.is_empty() { vec![] } else { tail(&self.viz.mix, mix) },
            notes: (0..self.viz.ch.len()).map(|i| (self.voices.get(i).map_or(false, |v| v.active())).then(|| self.voice_note.get(i).copied().unwrap_or(0))).collect(),
        };
        self.viz.peak.iter_mut().for_each(|p| *p = 0.0); self.viz.master_peak = 0.0;
        snap
    }
    /// Einzelton zur Eingabe-Rückmeldung
    pub fn audition(&mut self, ins: &Instr, note: u8) {
        let slot = self.voices.len() - 1; self.voices[slot].start(ins, note); self.audition.push((slot, 0.25));
    }
    fn step(&mut self) {
        // Song kurz herausnehmen statt zu kopieren: step() läuft im Audio-Thread, eine tiefe Kopie je Zeile wäre eine große Speicherzuweisung
        let Some(song) = self.song.take() else { return };
        if self.row == 0 && self.ord >= song.order.len() { self.ord = song.loop_at.min(song.order.len().saturating_sub(1)); }
        let Some(&p) = song.order.get(self.ord) else { self.playing = false; self.song = Some(song); return };
        self.pos = (self.ord, self.row);
        if let Some(cells) = song.patterns.get(p).and_then(|pat| pat.get(self.row)) {
            for (v, &(n, i)) in cells.iter().enumerate().take(song.channels) {
                if i != INS_KEEP { self.cur_ins[v] = i as usize; }
                if n == 0 { continue; }
                if n == NOTE_OFF { self.voices[v].release(); } else if let Some(ins) = song.instruments.get(self.cur_ins[v]) { self.voices[v].start(ins, n); self.voice_note[v] = n; }
            }
        }
        self.row += 1; if self.row >= song.rows { self.row = 0; self.ord += 1; }
        self.song = Some(song);
    }
    pub fn render(&mut self, out: &mut [f32], sr: f32) {
        let frame = sr / self.song.as_ref().map_or(50.0, |s| s.frame_hz).max(1.0);
        for o in out.iter_mut() {
            if self.playing {
                let row_len = self.song.as_ref().map_or(sr, |s| s.row_secs * sr);
                if self.samples_in_row <= 0.0 { self.step(); self.samples_in_row += row_len; }
                self.samples_in_row -= 1.0;
                if self.playing { self.frame_acc += 1.0; if self.frame_acc >= frame { self.frame_acc -= frame; for v in &mut self.voices { v.tick(); } } }
            }
            let nch = self.viz.ch.len(); let any_solo = self.solo.iter().any(|b| *b);
            let mut s = 0.0;
            for (i, v) in self.voices.iter_mut().enumerate() {
                let x = if v.active() { v.sample(sr) } else { 0.0 };
                if i < nch {
                    let audible = !self.mute.get(i).copied().unwrap_or(false) && (!any_solo || self.solo.get(i).copied().unwrap_or(false));
                    let x = if audible { x } else { 0.0 };
                    self.viz.ch[i][self.viz.w] = x; if x.abs() > self.viz.peak[i] { self.viz.peak[i] = x.abs(); }
                    s += x;
                } else { s += x; }
            }
            *o = (s * self.master).clamp(-1.0, 1.0);
            if self.viz.mix.len() == RING { self.viz.mix[self.viz.w] = *o; if o.abs() > self.viz.master_peak { self.viz.master_peak = o.abs(); } }
            self.viz.w = (self.viz.w + 1) % RING;
        }
        self.sr = sr;
        let dt = out.len() as f32 / sr;
        let rel: Vec<usize> = self.audition.iter_mut().filter_map(|(slot, t)| { *t -= dt; (*t <= 0.0).then_some(*slot) }).collect();
        for slot in rel { self.voices[slot].release(); }
        self.audition.retain(|(_, t)| *t > 0.0);
        let _ = self.row_started;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sid(wave: Wave) -> Instr { Instr::Sid { wave, pw: 0.5, a: 0, d: 9, s: 8, r: 4 } }

    fn zero_crossings(b: &[f32]) -> usize { b.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count() }

    #[test]
    fn pitch_and_envelope() {
        let sr = 44100.0; let mut e = Engine::default();
        e.audition(&sid(Wave::Saw), 58);                     // A-4 = 440 Hz
        let mut buf = vec![0.0; 22050]; e.render(&mut buf, sr);
        let hzs = zero_crossings(&buf[2000..]) as f32 / ((buf.len() - 2000) as f32 / sr);
        assert!((hzs - 440.0).abs() < 8.0, "Frequenz {hzs}");
        assert!(buf.iter().any(|v| v.abs() > 0.02));
        // nach Ende der Einzeltonzeit klingt der Ton aus
        let mut tail = vec![0.0; 44100]; e.render(&mut tail, sr); assert!(tail[40000..].iter().all(|v| v.abs() < 0.01));
    }

    #[test]
    fn sequencer_plays_rows_and_loops() {
        let song = SongData { pokey: false, channels: 2, rows: 4, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(49, 0), (0, INS_KEEP)], vec![(0, INS_KEEP); 2], vec![(NOTE_OFF, INS_KEEP), (53, 0)], vec![(0, INS_KEEP); 2]]], instruments: vec![sid(Wave::Pulse)], row_secs: 0.1, frame_hz: 50.0 };
        let mut e = Engine::default(); e.play(song, 0, 0);
        let mut b = vec![0.0; 4410]; e.render(&mut b, 44100.0);
        assert!(b.iter().any(|v| v.abs() > 0.02), "erste Zeile erzeugt Ton");
        assert_eq!(e.pos.1, 0);
        let mut b2 = vec![0.0; 4410 * 3]; e.render(&mut b2, 44100.0);
        assert!(e.pos.1 >= 2, "Zeile fortgeschritten: {:?}", e.pos);
        let mut b3 = vec![0.0; 44100]; e.render(&mut b3, 44100.0); assert!(e.playing, "Wiederholung");
    }

    #[test]
    fn viz_levels_waves_mute_and_solo() {
        let song = SongData { pokey: false, channels: 2, rows: 4, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(49, 0), (53, 0)], vec![(0, INS_KEEP); 2], vec![(0, INS_KEEP); 2], vec![(0, INS_KEEP); 2]]], instruments: vec![sid(Wave::Saw)], row_secs: 0.5, frame_hz: 50.0 };
        let mut e = Engine::default(); e.play(song, 0, 0);
        let mut buf = vec![0.0; 8820]; e.render(&mut buf, 44100.0);
        let v = e.viz(1024, 2048);
        assert_eq!((v.peaks.len(), v.ch_wave.len(), v.ch_wave[0].len(), v.mix.len()), (2, 2, 1024, 2048));
        assert!(v.peaks[0] > 0.2 && v.peaks[1] > 0.2, "beide Stimmen haben Pegel: {:?}", v.peaks);
        assert!(v.ch_wave[0].iter().any(|x| x.abs() > 0.1) && v.mix.iter().any(|x| x.abs() > 0.02));
        assert_eq!(v.notes, vec![Some(49), Some(53)], "klingende Noten");
        assert_eq!(v.sr, 44100.0);
        // Die Spitzenpegel beginnen nach der Abfrage von vorn
        let mut quiet = vec![0.0; 1]; e.stop(); for _ in 0..200 { e.render(&mut buf, 44100.0); } e.render(&mut quiet, 44100.0);
        let _ = e.viz(16, 16);
        // Stumm und Solo wirken auf Klang und Anzeige
        let song2 = SongData { pokey: false, channels: 2, rows: 4, order: vec![0], loop_at: 0, patterns: vec![vec![vec![(49, 0), (53, 0)], vec![(0, INS_KEEP); 2], vec![(0, INS_KEEP); 2], vec![(0, INS_KEEP); 2]]], instruments: vec![sid(Wave::Saw)], row_secs: 0.5, frame_hz: 50.0 };
        e.play(song2, 0, 0); e.set_mute(0, true);
        e.render(&mut buf, 44100.0); let v = e.viz(1024, 2048);
        assert_eq!(v.peaks[0], 0.0, "stumme Stimme zeigt nichts"); assert!(v.peaks[1] > 0.2);
        e.set_mute(0, false); e.set_solo(0, true); let _ = e.viz(16, 16);
        e.render(&mut buf, 44100.0); let v = e.viz(1024, 2048);
        assert!(v.peaks[0] > 0.2 && v.peaks[1] == 0.0, "Solo: nur Stimme 1: {:?}", v.peaks);
        assert!(e.soloed(0) && !e.muted(0));
    }

    #[test]
    fn pokey_voice_and_cells() {
        let ins = Instr::Pokey { dist: "pure".into(), env: vec![15; 16], loop_at: 15 };
        let mut v = Voice::default(); v.start(&ins, 49); let mut e = 0.0; for _ in 0..2000 { e += v.sample(44100.0).abs(); } assert!(e > 10.0);
        assert_eq!(parse_cell("C-4 00"), (49, 0)); assert_eq!(parse_cell("F#3 0A"), (43, 10)); assert_eq!(parse_cell("=== --"), (NOTE_OFF, INS_KEEP)); assert_eq!(fmt_cell(49, 0), "C-4 00");
    }
}
