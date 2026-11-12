// SPDX-License-Identifier: GPL-3.0-or-later
//! Live-Bild: Bildstrom der Ultimate zu fertigen Bildern zusammensetzen, als PNG speichern und einzelne Aufnahmen holen
//! (Screenshot als Dienstfunktion für Tests und Dokumentation, `vcc streams shot`).
//!
//! [`VideoFeed`] hängt an einem [`StreamService`] (Bild) und wird von der Oberfläche regelmäßig mit [`VideoFeed::pump`] abgefragt;
//! es liefert nur das neueste Bild, ältere werden verworfen.
use crate::streams::{StreamOptions, StreamService, StreamState};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use vc_emu::streams::video::{Frame, FrameAssembler};
use vc_emu::streams::{channel_sink, RawPacket, Sink, StreamKind};

pub struct VideoFeed {
    rx: Receiver<RawPacket>,
    asm: FrameAssembler,
    /// Pakete, die der Decoder abgelehnt hat
    pub bad: u64,
}

impl VideoFeed {
    /// Neuer Zuhörer samt Sink für `StreamService::start(StreamKind::Video, sink)`
    pub fn new() -> (VideoFeed, Sink) {
        let (sink, rx) = channel_sink(20_000);
        (VideoFeed { rx, asm: FrameAssembler::new(), bad: 0 }, sink)
    }
    /// Wartende Pakete verarbeiten; liefert das neueste vollständige Bild (ältere fallen weg). Höchstens `max` Pakete je Aufruf.
    pub fn pump(&mut self, max: usize) -> Option<Frame> {
        let mut newest = None;
        for _ in 0..max {
            let Ok(p) = self.rx.try_recv() else { break };
            match self.asm.push(&p.data) { Ok(Some(f)) => newest = Some(f), Ok(None) => {}, Err(_) => self.bad += 1 }
        }
        newest
    }
    pub fn frames_done(&self) -> u64 { self.asm.frames_done() }
}

/// Bild als PNG-Bytes
pub fn png_bytes(f: &Frame) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut out);
    image::ImageEncoder::write_image(enc, &f.rgba, f.width as u32, f.height as u32, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
    Ok(out)
}

pub fn save_png(f: &Frame, path: &Path) -> Result<(), String> {
    if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
    std::fs::write(path, png_bytes(f)?).map_err(|e| format!("{}: {e}", path.display()))
}

/// Schneidet den gezeichneten Bereich aus einem Emulatorbild: der Rahmen aus nicht schwarzen Punkten. Ist er kleiner als 40 % der Fläche
/// (z. B. fast alles schwarz), bleibt das Bild unverändert.
pub fn visible_area(f: &Frame) -> Frame {
    let (w, h) = (f.width, f.height);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h { for x in 0..w { let p = &f.rgba[(y * w + x) * 4..]; if p[0] | p[1] | p[2] != 0 { x0 = x0.min(x); x1 = x1.max(x + 1); y0 = y0.min(y); y1 = y1.max(y + 1); } } }
    if x1 <= x0 || y1 <= y0 || (x1 - x0) * (y1 - y0) * 10 < w * h * 4 { return f.clone(); }
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut rgba = Vec::with_capacity(cw * ch * 4);
    for y in y0..y1 { rgba.extend_from_slice(&f.rgba[(y * w + x0) * 4..(y * w + x1) * 4]); }
    Frame { width: cw, height: ch, rgba, frame_no: f.frame_no }
}

/// Projektbild: `<Projekt>/projektbild.png`
pub fn preview_path(project_root: &Path) -> PathBuf { project_root.join(crate::projfile::PREVIEW_FILE) }

/// Legt das Emulatorbild (auf den gezeichneten Bereich zugeschnitten) als Projektbild ab und ersetzt ein vorhandenes
pub fn save_project_preview(project_root: &Path, f: &Frame) -> Result<PathBuf, String> {
    let path = preview_path(project_root);
    save_png(&visible_area(f), &path)?;
    Ok(path)
}

/// Übernimmt ein beliebiges Bild (PNG, JPEG …) als Projektbild
pub fn import_project_preview(project_root: &Path, src: &Path) -> Result<PathBuf, String> {
    let img = image::open(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let path = preview_path(project_root);
    img.save_with_format(&path, image::ImageFormat::Png).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Löscht das Projektbild (`false`: es gab keines)
pub fn delete_project_preview(project_root: &Path) -> Result<bool, String> {
    let p = preview_path(project_root);
    if !p.is_file() { return Ok(false); }
    std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(true)
}

/// Dateiname `live-bild-JJJJ-MM-TT_HHMMSS.png` (UTC) für Sekunden seit 1970
pub fn shot_name(unix_secs: u64) -> String { shot_name_with("live-bild", unix_secs) }

/// Dateiname `<Vorsilbe>-JJJJ-MM-TT_HHMMSS.png` (UTC)
pub fn shot_name_with(prefix: &str, unix_secs: u64) -> String {
    let (days, rem) = (unix_secs / 86400, unix_secs % 86400);
    // bürgerliches Datum aus Tageszahl (Algorithmus nach H. Hinnant)
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{prefix}-{y:04}-{m:02}-{d:02}_{:02}{:02}{:02}.png", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Zielpfad für „Live-Bild speichern“: `<Projekt>/aufnahmen/live-bild-….png`
pub fn shot_path(project_root: &Path) -> PathBuf { shot_path_with(project_root, "live-bild") }

/// Zielpfad `<Projekt>/aufnahmen/<Vorsilbe>-….png`
pub fn shot_path_with(project_root: &Path, prefix: &str) -> PathBuf {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    project_root.join("aufnahmen").join(shot_name_with(prefix, now))
}

/// Ein Bild von der Ultimate holen: Bildstrom starten, bis zum ersten vollständigen Bild warten, Strom stoppen.
/// `wait` begrenzt die Wartezeit nach dem Start. Fehlertexte kommen aus dem Dienst (kein LAN-Link, keine Pakete/Firewall).
pub fn grab_frame(host: &str, password: Option<&str>, opts: StreamOptions, wait: Duration) -> Result<Frame, String> {
    let mut svc = StreamService::new(host, password, opts);
    let (mut feed, sink) = VideoFeed::new();
    svc.start(StreamKind::Video, sink)?;
    let t0 = Instant::now();
    let r = loop {
        std::thread::sleep(Duration::from_millis(20));
        svc.poll();
        if let Some(f) = feed.pump(10_000) { break Ok(f); }
        if let StreamState::Error(e) = svc.state(StreamKind::Video) { break Err(e.clone()); }
        if t0.elapsed() > wait { break Err("Kein vollständiges Bild innerhalb der Wartezeit empfangen.".into()); }
    };
    svc.stop_all();
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_use_utc_time() {
        assert_eq!(shot_name(0), "live-bild-1970-01-01_000000.png");
        assert_eq!(shot_name(1_794_146_701), "live-bild-2026-11-08_140501.png");
        assert_eq!(shot_name(951_782_400 + 86399), "live-bild-2000-02-29_235959.png");
        assert_eq!(shot_name_with("emulator-bild", 0), "emulator-bild-1970-01-01_000000.png");
    }
    #[test]
    fn visible_area_crops_the_drawn_frame() {
        let (w, h) = (10usize, 8usize);
        let mut rgba = vec![0u8; w * h * 4];
        for y in 1..7 { for x in 1..9 { let i = (y * w + x) * 4; rgba[i..i + 4].copy_from_slice(&[10, 20, 30, 255]); } }
        let f = Frame { width: w, height: h, rgba, frame_no: 1 };
        let c = visible_area(&f);
        assert_eq!((c.width, c.height), (8, 6));
        assert!(c.rgba.chunks(4).all(|p| p[..3] == [10, 20, 30]));
        let mut tiny = f.clone(); tiny.rgba.iter_mut().for_each(|b| *b = 0); tiny.rgba[0] = 9;
        assert_eq!((visible_area(&tiny).width, visible_area(&tiny).height), (w, h), "fast leeres Bild bleibt");
    }
    #[test]
    fn project_preview_roundtrip() {
        let dir = std::env::temp_dir().join(format!("vc-preview-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = Frame { width: 2, height: 2, rgba: vec![200; 16], frame_no: 0 };
        let p = save_project_preview(&dir, &f).unwrap();
        assert_eq!(p, dir.join("projektbild.png")); assert!(p.is_file());
        assert!(delete_project_preview(&dir).unwrap() && !p.exists());
        assert!(!delete_project_preview(&dir).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }
    #[test]
    fn feed_assembles_real_packets_and_encodes_png() {
        let b = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../vc-emu/tests/data/video.bin")).unwrap();
        let (mut feed, sink) = VideoFeed::new();
        let meta = vc_emu::streams::RxMeta { seq: 0, lost_before: 0, late: false };
        let mut o = 0;
        while o + 2 <= b.len() { let n = u16::from_le_bytes([b[o], b[o + 1]]) as usize; sink(&b[o + 2..o + 2 + n], meta); o += 2 + n; }
        let f = feed.pump(1000).expect("Bild");
        assert_eq!((f.width, f.height), (384, 272));
        let png = png_bytes(&f).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        let back = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(back.as_raw(), &f.rgba);
    }
}
