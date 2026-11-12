// SPDX-License-Identifier: GPL-3.0-or-later
//! WAV-Aufnahme des Audio-Stroms (PCM, 16 Bit, stereo, 44-Byte-Kopf) und ein Leser dafür, dazu der Vergleich zweier Signale.
use super::audio::{Frame, FRAMES_PER_PACKET};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Schreibt eine WAV-Datei fortlaufend; die Längenfelder im Kopf werden bei [`finish`](Self::finish) (und beim Verwerfen) nachgetragen.
pub struct WavWriter { f: Option<File>, path: PathBuf, data_len: u64, rate: u32 }

fn header(rate: u32, data_len: u32) -> [u8; 44] {
    let mut h = [0u8; 44];
    h[0..4].copy_from_slice(b"RIFF"); h[4..8].copy_from_slice(&(36 + data_len).to_le_bytes()); h[8..12].copy_from_slice(b"WAVE");
    h[12..16].copy_from_slice(b"fmt "); h[16..20].copy_from_slice(&16u32.to_le_bytes()); h[20..22].copy_from_slice(&1u16.to_le_bytes());     // PCM
    h[22..24].copy_from_slice(&2u16.to_le_bytes()); h[24..28].copy_from_slice(&rate.to_le_bytes()); h[28..32].copy_from_slice(&(rate * 4).to_le_bytes());
    h[32..34].copy_from_slice(&4u16.to_le_bytes()); h[34..36].copy_from_slice(&16u16.to_le_bytes());
    h[36..40].copy_from_slice(b"data"); h[40..44].copy_from_slice(&data_len.to_le_bytes());
    h
}

impl WavWriter {
    pub fn create(path: &Path, rate: u32) -> Result<WavWriter, String> {
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
        let mut f = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
        f.write_all(&header(rate, 0)).map_err(|e| e.to_string())?;
        Ok(WavWriter { f: Some(f), path: path.to_path_buf(), data_len: 0, rate })
    }
    pub fn path(&self) -> &Path { &self.path }
    pub fn rate(&self) -> u32 { self.rate }
    pub fn frames(&self) -> u64 { self.data_len / 4 }
    pub fn seconds(&self) -> f64 { self.frames() as f64 / self.rate as f64 }
    /// Frames anhängen (links, rechts als 16-Bit-Werte); Fehler beim Schreiben beenden die Aufnahme
    pub fn write(&mut self, frames: impl Iterator<Item = (i16, i16)>) -> Result<(), String> {
        let Some(f) = self.f.as_mut() else { return Err("Aufnahme beendet".into()) };
        let mut buf = Vec::with_capacity(FRAMES_PER_PACKET * 4);
        for (l, r) in frames { buf.extend(l.to_le_bytes()); buf.extend(r.to_le_bytes()); }
        // Dateien über 4 GiB sind in einem WAV nicht darstellbar
        if self.data_len + buf.len() as u64 > u32::MAX as u64 - 64 { return Err("WAV-Datei voll (4 GiB)".into()); }
        f.write_all(&buf).map_err(|e| e.to_string())?;
        self.data_len += buf.len() as u64;
        Ok(())
    }
    /// Kopf nachtragen und schließen
    pub fn finish(&mut self) -> Result<(), String> {
        let Some(mut f) = self.f.take() else { return Ok(()) };
        f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        f.write_all(&header(self.rate, self.data_len as u32)).map_err(|e| e.to_string())?;
        f.flush().map_err(|e| e.to_string())
    }
}
impl Drop for WavWriter { fn drop(&mut self) { let _ = self.finish(); } }

/// WAV (PCM, 16 Bit, 1 oder 2 Kanäle) lesen: (Rate, Frames als f32; Mono wird verdoppelt)
pub fn read_wav(b: &[u8]) -> Result<(u32, Vec<Frame>), String> {
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" { return Err("keine WAV-Datei".into()); }
    let (mut o, mut fmt, mut data) = (12usize, None, None);
    while o + 8 <= b.len() {
        let id = &b[o..o + 4]; let n = u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
        let body = &b[(o + 8).min(b.len())..(o + 8 + n).min(b.len())];
        if id == b"fmt " && body.len() >= 16 { fmt = Some((u16::from_le_bytes([body[0], body[1]]), u16::from_le_bytes([body[2], body[3]]), u32::from_le_bytes([body[4], body[5], body[6], body[7]]), u16::from_le_bytes([body[14], body[15]]))); }
        if id == b"data" { data = Some(body); break; }
        o += 8 + n + (n & 1);
    }
    let (tag, ch, rate, bits) = fmt.ok_or("fmt-Block fehlt")?; let data = data.ok_or("data-Block fehlt")?;
    if tag != 1 || bits != 16 || !(1..=2).contains(&ch) { return Err(format!("nur PCM mit 16 Bit und 1 oder 2 Kanälen (hier Format {tag}, {bits} Bit, {ch} Kanäle)")); }
    if rate == 0 { return Err("Abtastrate 0".into()); }
    let s = |i: usize| i16::from_le_bytes([data[i], data[i + 1]]) as f32 / 32768.0;
    let frames = if ch == 2 { data.chunks_exact(4).enumerate().map(|(k, _)| (s(k * 4), s(k * 4 + 2))).collect() } else { data.chunks_exact(2).enumerate().map(|(k, _)| (s(k * 2), s(k * 2))).collect() };
    Ok((rate, frames))
}

// ------------------------------------------------------------------------ Vergleich

/// Zahlenmaß des Vergleichs zweier Signale (nur Hilfe, keine Prüfung)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comparison {
    /// Um so viele Sekunden setzt B (Aufnahme) später ein als A (Synthesizer); negativ = früher
    pub lag_secs: f64,
    /// Korrelation der Hüllkurven bei dieser Ausrichtung (−1 … 1)
    pub corr: f64,
    /// Effektivpegel von A und B im überlappenden Teil (dBFS) und B − A
    pub rms_a_db: f64, pub rms_b_db: f64, pub diff_db: f64,
    /// Länge des überlappenden Teils
    pub overlap_secs: f64,
}

/// Hüllkurve: Effektivwert in Fenstern von `hop` Samples
pub fn envelope(x: &[f32], hop: usize) -> Vec<f32> {
    x.chunks(hop.max(1)).map(|c| (c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32).sqrt()).collect()
}

/// Normierte Kreuzkorrelation der Hüllkurven; liefert (Versatz in Fenstern von B gegen A, Korrelation) für Versätze bis `max_lag`.
pub fn align(a: &[f32], b: &[f32], max_lag: usize) -> Option<(isize, f64)> {
    let mut best: Option<(isize, f64)> = None;
    for lag in -(max_lag as isize)..=(max_lag as isize) {
        // b[i + lag] gehört zu a[i]
        let (i0, i1) = (0.max(-lag), (a.len() as isize).min(b.len() as isize - lag));
        if i1 - i0 < (a.len().min(b.len()) as isize / 2).max(8) { continue; }   // zu kleine Überlappung täuscht hohe Korrelation vor
        let n = (i1 - i0) as f64;
        let (mut sa, mut sb, mut saa, mut sbb, mut sab) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for i in i0..i1 { let (x, y) = (a[i as usize] as f64, b[(i + lag) as usize] as f64); sa += x; sb += y; saa += x * x; sbb += y * y; sab += x * y; }
        let (va, vb) = (saa - sa * sa / n, sbb - sb * sb / n);
        if va <= 1e-12 || vb <= 1e-12 { continue; }
        let c = (sab - sa * sb / n) / (va * vb).sqrt();
        if best.map_or(true, |(_, bc)| c > bc) { best = Some((lag as isize, c)); }
    }
    best
}

fn rms_db(x: &[f32]) -> f64 { if x.is_empty() { return f64::NEG_INFINITY; } let m = x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len() as f64; 10.0 * m.max(1e-20).log10() }

/// Vergleich zweier Mono-Signale gleicher Rate: Ausrichtung (bis ±`max_lag_secs`) per Kreuzkorrelation der Hüllkurven (10-ms-Fenster),
/// dazu Pegelunterschied im überlappenden Teil.
pub fn compare(a: &[f32], b: &[f32], rate: f64, max_lag_secs: f64) -> Option<Comparison> {
    let hop = (rate * 0.01) as usize;
    let (ea, eb) = (envelope(a, hop), envelope(b, hop));
    let (lag, corr) = align(&ea, &eb, (max_lag_secs / 0.01) as usize)?;
    let shift = lag as i64 * hop as i64;          // B-Index = A-Index + shift
    let (i0, i1) = (0.max(-shift), (a.len() as i64).min(b.len() as i64 - shift));
    if i1 <= i0 { return None; }
    let (sa, sb) = (&a[i0 as usize..i1 as usize], &b[(i0 + shift) as usize..(i1 + shift) as usize]);
    let (ra, rb) = (rms_db(sa), rms_db(sb));
    Some(Comparison { lag_secs: shift as f64 / rate, corr, rms_a_db: ra, rms_b_db: rb, diff_db: rb - ra, overlap_secs: (i1 - i0) as f64 / rate })
}

/// Mono-Mischung
pub fn mono(f: &[Frame]) -> Vec<f32> { f.iter().map(|(l, r)| (l + r) * 0.5).collect() }

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-wav-{}-{name}", std::process::id())); let _ = std::fs::remove_file(&d); d }

    #[test]
    fn writes_a_valid_44_byte_header_and_reads_back() {
        let p = tmp("a.wav");
        { let mut w = WavWriter::create(&p, 47983).unwrap();
          w.write((0..1000).map(|i| (i as i16, -(i as i16)))).unwrap(); w.write(std::iter::once((i16::MIN, i16::MAX))).unwrap();
          assert_eq!(w.frames(), 1001); w.finish().unwrap(); w.finish().unwrap(); }
        let b = std::fs::read(&p).unwrap();
        assert_eq!(b.len(), 44 + 4004); assert_eq!(&b[0..4], b"RIFF"); assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 36 + 4004);
        assert_eq!(u32::from_le_bytes(b[24..28].try_into().unwrap()), 47983); assert_eq!(u32::from_le_bytes(b[28..32].try_into().unwrap()), 47983 * 4);
        assert_eq!(u32::from_le_bytes(b[40..44].try_into().unwrap()), 4004);
        let (rate, f) = read_wav(&b).unwrap(); assert_eq!(rate, 47983); assert_eq!(f.len(), 1001);
        assert_eq!(f[1000], (-1.0, 32767.0 / 32768.0)); assert!((f[10].0 - 10.0 / 32768.0).abs() < 1e-9 && (f[10].1 + 10.0 / 32768.0).abs() < 1e-9);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn drop_completes_the_header() {
        let p = tmp("b.wav");
        { let mut w = WavWriter::create(&p, 44100).unwrap(); w.write(std::iter::repeat((1, 2)).take(10)).unwrap(); }
        let (r, f) = read_wav(&std::fs::read(&p).unwrap()).unwrap(); assert_eq!((r, f.len()), (44100, 10));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn reader_rejects_garbage() {
        assert!(read_wav(&[]).is_err()); assert!(read_wav(b"RIFF\0\0\0\0WAVE").is_err());
        let mut h = header(48000, 0).to_vec(); h[34] = 8; assert!(read_wav(&h).unwrap_err().contains("16 Bit"));
        let mut h = header(48000, 0).to_vec(); h[22] = 6; assert!(read_wav(&h).is_err());
        let mut h = header(48000, 400).to_vec(); h.extend([0; 10]); assert!(read_wav(&h).is_ok(), "abgeschnittene Daten werden gelesen, soweit vorhanden");
        for n in 0..=44 { let _ = read_wav(&header(48000, 8)[..n]); }
    }

    /// Rauschartiges, aber wiederholbares Signal mit wechselnder Lautstärke
    fn song(n: usize, rate: f64) -> Vec<f32> {
        let mut s = 12345u32;
        (0..n).map(|i| { s = s.wrapping_mul(1664525).wrapping_add(1013904223); let noise = (s >> 16) as f32 / 32768.0 - 1.0;
            let t = i as f64 / rate; let env = if (t * 2.0) as i64 % 3 == 0 { 0.8 } else if (t * 2.0) as i64 % 3 == 1 { 0.1 } else { 0.4 };
            let tone = (2.0 * std::f64::consts::PI * 440.0 * t).sin() as f32; env as f32 * (0.7 * tone + 0.3 * noise) }).collect()
    }

    #[test]
    fn alignment_finds_the_shift_and_level_difference() {
        let rate = 48000.0; let a = song(rate as usize * 6, rate);
        for lag_ms in [0i64, 130, 770, -400] {
            let shift = lag_ms * 48; let gain = 0.5f32;
            let b: Vec<f32> = (0..a.len() as i64).map(|i| { let j = i - shift; if j >= 0 && (j as usize) < a.len() { a[j as usize] * gain } else { 0.0 } }).collect();
            let c = compare(&a, &b, rate, 2.0).unwrap();
            assert!((c.lag_secs - lag_ms as f64 / 1000.0).abs() <= 0.011, "{lag_ms}: {c:?}");
            assert!(c.corr > 0.95, "{c:?}"); assert!((c.diff_db + 6.02).abs() < 0.5, "{c:?}"); assert!(c.overlap_secs > 4.0);
        }
    }

    #[test]
    fn compare_handles_silence_and_empty() {
        assert!(compare(&[], &[], 48000.0, 1.0).is_none());
        assert!(compare(&vec![0.0; 48000], &vec![0.0; 48000], 48000.0, 1.0).is_none(), "keine Hüllkurve zum Vergleichen");
        let a = song(48000, 48000.0); assert!(compare(&a, &a, 48000.0, 1.0).unwrap().corr > 0.999);
    }
}
