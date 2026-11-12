// SPDX-License-Identifier: GPL-3.0-or-later
//! Audio-Strom: 770 Byte je Paket = 2 Byte Zähler (u16 LE) + 768 Byte = 192 Stereo-Abtastwerte (je 16 Bit mit Vorzeichen, little-endian, links zuerst).
//! Beim Start kommt zusätzlich ein Paket von nur 2 Byte (so beginnt jeder Strom); das ist ein Steuerpaket ([`PacketError::Control`]).
use super::PacketError;

pub const PACKET_LEN: usize = 770;
pub const PAYLOAD_LEN: usize = 768;
pub const FRAMES_PER_PACKET: usize = 192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioPacket<'a> { pub seq: u16, /// 768 Byte
    pub data: &'a [u8] }

impl AudioPacket<'_> {
    /// Stereo-Abtastwerte (links, rechts) als 16-Bit-Zahlen
    pub fn frames(&self) -> impl Iterator<Item = (i16, i16)> + '_ {
        self.data.chunks_exact(4).map(|c| (i16::from_le_bytes([c[0], c[1]]), i16::from_le_bytes([c[2], c[3]])))
    }
}

pub fn parse(p: &[u8]) -> Result<AudioPacket<'_>, PacketError> {
    if p.len() == 2 { return Err(PacketError::Control); }
    if p.len() != PACKET_LEN { return Err(PacketError::Length { got: p.len(), want: PACKET_LEN }); }
    Ok(AudioPacket { seq: u16::from_le_bytes([p[0], p[1]]), data: &p[2..2 + PAYLOAD_LEN] })
}


/// Abtastrate des Stroms (Hz): Die Ultimate sendet 192 Frames je Paket, 250,2 Pakete/s (PAL) bzw. 249,7 Pakete/s (NTSC)
pub const RATE_PAL: f64 = 47_982.886;
pub const RATE_NTSC: f64 = 47_940.0;

/// 16-Bit-Wert nach −1,0 … <1,0
#[inline]
pub fn to_f32(v: i16) -> f32 { v as f32 / 32768.0 }

/// Stereo-Frame (f32)
pub type Frame = (f32, f32);

/// Wandelt ein Paket in f32-Frames (links, rechts); Fehler wie [`parse`].
pub fn decode(p: &[u8], out: &mut Vec<Frame>) -> Result<u16, PacketError> {
    let a = parse(p)?;
    out.extend(a.frames().map(|(l, r)| (to_f32(l), to_f32(r))));
    Ok(a.seq)
}

/// Kubische Interpolation (Catmull-Rom) zwischen `b` und `c` an der Stelle `t` (0..1); `a` davor, `d` danach
#[inline]
pub fn cubic(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let (t2, t3) = (t * t, t * t * t);
    0.5 * ((2.0 * b) + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3)
}

/// Zähler des Puffers
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerStats {
    /// Pakete, die als Lücke aufgefüllt wurden (Anzahl der fehlenden Pakete)
    pub filled_packets: u64,
    /// Unterläufe (Puffer leer, neu aufgefüllt)
    pub underruns: u32,
    /// Überläufe (Puffer zu voll, Anfang verworfen)
    pub overruns: u32,
    /// Aktueller Füllstand in Millisekunden (geglättet)
    pub fill_ms: f64,
    /// Aktuelles Wandlungsverhältnis relativ zum Nennwert (1,0 = nominal)
    pub ratio: f64,
}

/// Ringpuffer mit Neuabtastung zwischen dem Takt der Ultimate und dem der Soundkarte.
///
/// Eingang: [`push`](Self::push) mit Paketen in Quellrate. Ausgang: [`pull`](Self::pull) füllt Frames in Geräterate.
/// Zielfüllung ~60 ms; aus der (geglätteten) Abweichung wird das Verhältnis um höchstens ±0,5 % nachgeführt.
/// Ausgabe beginnt erst bei halber Zielfüllung. Fehlende Pakete werden durch eine Überblendung ersetzt, die die Länge erhält.
pub struct StreamPlayer {
    q: std::collections::VecDeque<Frame>,
    /// Position des nächsten Ausgabeframes in `q` (Index mit Bruchteil; `q[0]` ist der Vorgänger)
    pos: f64,
    src_rate: f64,
    dst_rate: f64,
    target: usize,
    started: bool,
    fill_avg: f64,
    ratio: f64,
    seq: Option<u16>,
    last: Frame,
    stats: PlayerStats,
}

/// Ziel-Füllung in Sekunden
pub const TARGET_SECS: f64 = 0.060;
/// Größte Abweichung des Verhältnisses vom Nennwert
pub const MAX_RATIO_DEV: f64 = 0.005;
/// Höchstzahl fehlender Pakete, die noch als Lücke gefüllt werden (darüber: Neustart des Stroms, Puffer neu füllen)
pub const MAX_FILL_PACKETS: u16 = 100;

impl StreamPlayer {
    pub fn new(src_rate: f64, dst_rate: f64) -> StreamPlayer {
        StreamPlayer { q: Default::default(), pos: 1.0, src_rate, dst_rate, target: (src_rate * TARGET_SECS) as usize, started: false, fill_avg: 0.0, ratio: 1.0, seq: None, last: (0.0, 0.0), stats: PlayerStats { ratio: 1.0, ..Default::default() } }
    }
    pub fn src_rate(&self) -> f64 { self.src_rate }
    pub fn dst_rate(&self) -> f64 { self.dst_rate }
    pub fn set_dst_rate(&mut self, r: f64) { self.dst_rate = r; }
    pub fn started(&self) -> bool { self.started }
    /// Füllstand in Quellframes vor der Leseposition
    pub fn fill(&self) -> f64 { self.q.len() as f64 - self.pos }
    pub fn stats(&self) -> PlayerStats { let mut s = self.stats; s.fill_ms = self.fill_avg / self.src_rate * 1000.0; s.ratio = self.ratio; s }
    /// Alles verwerfen (neuer Strom)
    pub fn reset(&mut self) { self.q.clear(); self.pos = 1.0; self.started = false; self.fill_avg = 0.0; self.ratio = 1.0; self.seq = None; }

    /// Ein Paket einreihen. `seq` ist der Zähler des Pakets, `frames` dessen Inhalt.
    pub fn push(&mut self, seq: u16, frames: &[Frame]) {
        if frames.is_empty() { return; }
        if let Some(prev) = self.seq {
            let d = seq.wrapping_sub(prev.wrapping_add(1));
            if d != 0 && d < 0x8000 {
                if d <= MAX_FILL_PACKETS {
                    // Lücke: gleich lange Überblendung vom letzten zum ersten neuen Frame, Füllstand bleibt stetig
                    let n = d as usize * frames.len();
                    let (a, b) = (self.last, frames[0]);
                    for i in 0..n { let t = (i + 1) as f32 / (n + 1) as f32; self.q.push_back((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)); }
                    self.stats.filled_packets += d as u64;
                } else { self.reset(); }
            } else if d != 0 { return; }          // verspätet oder doppelt
        }
        self.seq = Some(seq);
        if self.q.is_empty() { self.q.push_back(frames[0]); self.pos = 1.0; }
        self.q.extend(frames.iter().copied());
        self.last = *frames.last().unwrap();
        let cap = self.target * 4;
        if self.q.len() as f64 - self.pos > cap as f64 {
            let drop = (self.q.len() as f64 - self.pos) as usize - self.target;
            for _ in 0..drop { self.q.pop_front(); }
            self.pos = (self.pos - drop as f64).max(1.0);
            if self.q.len() < 3 { self.reset(); }
            self.stats.overruns += 1;
        }
        if !self.started && self.fill() >= self.target as f64 / 2.0 { self.started = true; self.fill_avg = self.fill(); }
    }

    /// Ausgabeframes in Geräterate holen; nicht Verfügbares wird still (und der Puffer neu aufgefüllt).
    pub fn pull(&mut self, out: &mut [Frame]) {
        if !self.started { out.fill((0.0, 0.0)); return; }
        let base = self.src_rate / self.dst_rate;
        // Füllstand glätten (~0,4 s), Abweichung ergibt das Verhältnis
        let alpha = 1.0 - (-(out.len() as f64) / self.dst_rate / 0.4).exp();
        self.fill_avg += (self.fill() - self.fill_avg) * alpha;
        let err = (self.fill_avg - self.target as f64) / self.target as f64;
        self.ratio = 1.0 + (err * 0.05).clamp(-MAX_RATIO_DEV, MAX_RATIO_DEV);
        let step = base * self.ratio;
        for (k, o) in out.iter_mut().enumerate() {
            let i = self.pos.floor() as usize;
            if i + 2 >= self.q.len() {
                // leer: Rest still, neu füllen
                for r in out[k..].iter_mut() { *r = (0.0, 0.0); }
                self.started = false; self.stats.underruns += 1;
                if let Some(l) = self.q.back().copied() { self.q.clear(); self.q.push_back(l); self.pos = 1.0; }
                return;
            }
            let t = (self.pos - i as f64) as f32;
            let (a, b, c, d) = (self.q[i - 1], self.q[i], self.q[i + 1], self.q[i + 2]);
            *o = (cubic(a.0, b.0, c.0, d.0, t), cubic(a.1, b.1, c.1, d.1, t));
            self.pos += step;
            while self.pos >= 2.0 { self.q.pop_front(); self.pos -= 1.0; }
        }
    }
}

/// Gemeinsamer Zustand zwischen Empfangsthread und Ausgabe: der Puffer und, auf Knopfdruck, die WAV-Aufnahme.
pub struct AudioHub {
    pub player: StreamPlayer,
    pub rec: Option<super::wav::WavWriter>,
    /// Fehler der Aufnahme (Platte voll …); die Aufnahme ist dann beendet
    pub rec_error: Option<String>,
    /// Pakete insgesamt, die in den Puffer kamen
    pub packets: u64,
}

impl AudioHub {
    pub fn new(src_rate: f64, dst_rate: f64) -> AudioHub { AudioHub { player: StreamPlayer::new(src_rate, dst_rate), rec: None, rec_error: None, packets: 0 } }
    /// Ein gültiges Paket verarbeiten (Aufnahme in Quellrate, vor der Neuabtastung; Lücken als Stille, damit die Zeit stimmt)
    pub fn on_packet(&mut self, data: &[u8], meta: super::RxMeta) {
        if meta.late { return; }
        let Ok(a) = parse(data) else { return };
        let frames: Vec<Frame> = a.frames().map(|(l, r)| (to_f32(l), to_f32(r))).collect();
        self.player.push(a.seq, &frames); self.packets += 1;
        if let Some(w) = self.rec.as_mut() {
            let gap = (meta.lost_before as usize).min(MAX_FILL_PACKETS as usize) * FRAMES_PER_PACKET;
            if let Err(e) = w.write(std::iter::repeat((0i16, 0i16)).take(gap).chain(a.frames())) { self.rec_error = Some(e); self.rec = None; }
        }
    }
}

/// Sink für den Empfänger, die Pakete in den gemeinsamen Zustand einreiht
pub fn hub_sink(hub: std::sync::Arc<std::sync::Mutex<AudioHub>>) -> super::Sink {
    std::sync::Arc::new(move |d: &[u8], m: super::RxMeta| { if let Ok(mut h) = hub.lock() { h.on_packet(d, m); } })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_frames() {
        let mut p = vec![0u8; PACKET_LEN]; p[0] = 5; p[1] = 1;
        p[2..6].copy_from_slice(&[0x00, 0x80, 0xFF, 0x7F]);
        let a = parse(&p).unwrap(); assert_eq!(a.seq, 261);
        assert_eq!(a.frames().count(), FRAMES_PER_PACKET);
        assert_eq!(a.frames().next(), Some((i16::MIN, i16::MAX)));
    }
    #[test]
    fn rejects_and_flags_control() {
        assert_eq!(parse(&[1, 0]).unwrap_err(), PacketError::Control);
        assert!(matches!(parse(&[]), Err(PacketError::Length { got: 0, .. })));
        assert!(matches!(parse(&[1]), Err(PacketError::Length { got: 1, .. })));
        assert!(matches!(parse(&vec![0; 769]), Err(PacketError::Length { .. })));
        assert!(matches!(parse(&vec![0; 771]), Err(PacketError::Length { .. })));
    }

    /// Simulation: Quelle mit `src_hz`, Gerät mit `dst_hz` (Nennwerte 47983 → 44100), jedes `drop_every`-te Paket geht verloren.
    /// Liefert (größte Sprunghöhe der Ausgabe nach dem Start, kleinster/größter Füllstand in ms, Zähler, Anzahl Ausgabeframes).
    fn simulate(src_hz: f64, dst_hz: f64, secs: f64, drop_every: usize) -> (f32, f64, f64, PlayerStats, usize) {
        let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
        let (freq, amp) = (440.0f64, 0.5f64);
        let sig = |i: u64| (amp * (2.0 * std::f64::consts::PI * freq * i as f64 / src_hz).sin()) as f32;
        let (mut t, mut next_pkt, mut pkt) = (0.0f64, 0.0f64, 0u64);
        let chunk = 512usize; let mut out = vec![(0.0f32, 0.0f32); chunk];
        let (mut maxjump, mut lo, mut hi, mut n) = (0.0f32, f64::MAX, 0.0f64, 0usize);
        let mut prev: Option<f32> = None;
        while t < secs {
            while next_pkt <= t {
                if drop_every == 0 || pkt as usize % drop_every != drop_every - 1 {
                    let frames: Vec<Frame> = (0..192).map(|k| { let v = sig(pkt * 192 + k); (v, v) }).collect();
                    pl.push(pkt as u16, &frames);
                }
                pkt += 1; next_pkt += 192.0 / src_hz;
            }
            pl.pull(&mut out); t += chunk as f64 / dst_hz;
            if pl.started() && t > 12.0 {
                lo = lo.min(pl.stats().fill_ms); hi = hi.max(pl.stats().fill_ms);
                for o in &out { if let Some(p) = prev { maxjump = maxjump.max((o.0 - p).abs()); } prev = Some(o.0); n += 1; }
            } else { prev = None; }
        }
        (maxjump, lo, hi, pl.stats(), n)
    }

    #[test]
    fn nominal_rates_keep_fill_and_stay_smooth() {
        let (jump, lo, hi, st, n) = simulate(RATE_PAL, 44100.0, 25.0, 0);
        assert!(n > 20 * 44100 / 2, "es wurde gespielt");
        assert!(jump < 0.04, "Sprunghöhe {jump}");          // Sinus 440 Hz bei 44,1 kHz, Amplitude 0,5: höchstens ~0,031
        assert!(lo > 40.0 && hi < 80.0, "Füllstand {lo}..{hi} ms");
        assert_eq!((st.underruns, st.overruns), (0, 0));
    }

    #[test]
    fn shifted_rates_are_followed_within_half_percent() {
        for dev in [-0.004, -0.001, 0.001, 0.004] {
            // Geräte-Uhr weicht um `dev` ab: Quelle schneller/langsamer
            let (jump, lo, hi, st, _) = simulate(RATE_PAL * (1.0 + dev), 44100.0, 40.0, 0);
            assert!(jump < 0.045, "{dev}: Sprunghöhe {jump}");
            assert!(lo > 30.0 && hi < 95.0, "{dev}: Füllstand {lo}..{hi} ms");
            assert_eq!((st.underruns, st.overruns), (0, 0), "{dev}");
            assert!((st.ratio - 1.0).abs() <= MAX_RATIO_DEV + 1e-12);
        }
        // Gerät läuft anders als nominal (48 kHz statt 44,1 kHz Nennwert wird nicht angenommen; hier 44,1 kHz ±0,3 %)
        for dev in [-0.003, 0.003] {
            let (jump, lo, hi, st, _) = simulate(RATE_PAL, 44100.0 * (1.0 + dev), 40.0, 0);
            assert!(jump < 0.04 && lo > 30.0 && hi < 95.0, "{dev}: {jump} {lo}..{hi}");
            assert_eq!((st.underruns, st.overruns), (0, 0));
        }
    }

    #[test]
    fn losses_are_filled_without_level_jump() {
        let (jump, lo, hi, st, _) = simulate(RATE_PAL, 44100.0, 25.0, 40);
        assert!(st.filled_packets > 100, "{}", st.filled_packets);
        assert!(jump < 0.12, "Sprunghöhe {jump}");        // Überblendung über eine Paketlänge, keine Stufe
        assert!(lo > 40.0 && hi < 80.0, "Füllstand {lo}..{hi} ms");
        assert_eq!((st.underruns, st.overruns), (0, 0));
    }

    #[test]
    fn waits_for_half_fill_before_output() {
        let mut pl = StreamPlayer::new(RATE_PAL, 48000.0);
        let mut out = [(1.0f32, 1.0f32); 64];
        let pkt = vec![(0.25f32, 0.25f32); 192];
        for i in 0..14u16 { pl.push(i, &pkt); }          // 14 × 4 ms = 56 ms ≥ 30 ms
        assert!(pl.started());
        let mut pl = StreamPlayer::new(RATE_PAL, 48000.0);
        for i in 0..6u16 { pl.push(i, &pkt); }           // 24 ms < 30 ms
        assert!(!pl.started()); pl.pull(&mut out); assert!(out.iter().all(|f| *f == (0.0, 0.0)), "vor dem Start still");
        pl.push(6, &pkt); pl.push(7, &pkt); assert!(pl.started());
    }

    #[test]
    fn underrun_goes_quiet_and_refills() {
        let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
        let pkt = vec![(0.5f32, -0.5f32); 192];
        for i in 0..20u16 { pl.push(i, &pkt); }
        let mut out = vec![(0.0f32, 0.0f32); 4096]; pl.pull(&mut out);
        assert_eq!(pl.stats().underruns, 1); assert!(!pl.started());
        assert!(out[0].0 > 0.4 && out[0].1 < -0.4, "Anfang wird gespielt: {:?}", out[0]);
        assert_eq!(*out.last().unwrap(), (0.0, 0.0));
        for i in 20..34u16 { pl.push(i, &pkt); }
        assert!(pl.started(), "nach dem Auffüllen geht es weiter");
    }

    #[test]
    fn late_duplicates_and_huge_gaps() {
        let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
        let pkt = vec![(0.1f32, 0.1f32); 192];
        pl.push(10, &pkt); pl.push(11, &pkt);
        let n = pl.q.len(); pl.push(11, &pkt); pl.push(9, &pkt); assert_eq!(pl.q.len(), n, "doppelt/verspätet verworfen");
        pl.push(11u16.wrapping_add(500), &pkt); assert_eq!(pl.stats().filled_packets, 0, "zu große Lücke: kein Auffüllen, Neustart");
        assert!(pl.q.len() <= 193);
        // Überlauf der Zähler
        let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
        pl.push(0xFFFF, &pkt); pl.push(0, &pkt); assert_eq!(pl.stats().filled_packets, 0);
        pl.push(2, &pkt); assert_eq!(pl.stats().filled_packets, 1);
    }

    #[test]
    fn overfull_buffer_is_trimmed() {
        let mut pl = StreamPlayer::new(RATE_PAL, 44100.0);
        let pkt = vec![(0.1f32, 0.1f32); 192];
        for i in 0..400u16 { pl.push(i, &pkt); }
        assert!(pl.fill() < pl.target as f64 * 4.0 + 193.0, "Füllstand begrenzt: {}", pl.fill());
        assert!(pl.stats().overruns > 0);
    }

    #[test]
    fn cubic_is_exact_on_lines_and_hits_nodes() {
        for t in [0.0f32, 0.25, 0.5, 1.0] { assert!((cubic(0.0, 1.0, 2.0, 3.0, t) - (1.0 + t)).abs() < 1e-6); }
        assert_eq!(cubic(5.0, 7.0, 9.0, 1.0, 0.0), 7.0); assert_eq!(cubic(5.0, 7.0, 9.0, 1.0, 1.0), 9.0);
    }

    #[test]
    fn decode_converts_to_float() {
        let mut p = vec![0u8; PACKET_LEN]; p[0] = 1; p[2..6].copy_from_slice(&[0x00, 0x80, 0xFF, 0x7F]);
        let mut v = vec![]; assert_eq!(decode(&p, &mut v), Ok(1));
        assert_eq!(v.len(), 192); assert_eq!(v[0].0, -1.0); assert!((v[0].1 - 0.99997).abs() < 1e-4);
        assert!(decode(&[1, 0], &mut v).is_err()); assert_eq!(v.len(), 192, "bei Fehler nichts angehängt");
    }

    #[test]
    fn hub_feeds_player_and_records_gaps_as_silence() {
        use super::super::{wav, RxMeta};
        let path = std::env::temp_dir().join(format!("vc-hub-{}.wav", std::process::id()));
        let mut h = AudioHub::new(RATE_PAL, 44100.0);
        h.rec = Some(wav::WavWriter::create(&path, 47983).unwrap());
        let mut p = vec![0u8; PACKET_LEN]; for (i, b) in p[2..].iter_mut().enumerate() { *b = (i % 200) as u8 + 1; }
        for (seq, lost) in [(0u16, 0u16), (1, 0), (4, 2)] { p[0] = seq as u8; h.on_packet(&p, RxMeta { seq, lost_before: lost, late: false }); }
        h.on_packet(&p, RxMeta { seq: 2, lost_before: 0, late: true });      // verspätet: ignoriert
        h.on_packet(&[1, 2, 3], RxMeta { seq: 9, lost_before: 0, late: false });  // ungültig: ignoriert
        assert_eq!(h.packets, 3); assert_eq!(h.player.stats().filled_packets, 2);
        assert_eq!(h.rec.as_ref().unwrap().frames(), 5 * 192, "3 Pakete + 2 Lücken");
        h.rec = None;
        let (_, f) = wav::read_wav(&std::fs::read(&path).unwrap()).unwrap(); let _ = std::fs::remove_file(&path);
        assert!(f[2 * 192..4 * 192].iter().all(|x| *x == (0.0, 0.0)) && f[4 * 192] != (0.0, 0.0));
    }
}
