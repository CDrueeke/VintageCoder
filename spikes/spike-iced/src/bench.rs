// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: eingebauter Messlauf. `--bench tippen|scrollen|canvas|idle|alles` fährt ein Szenario Bild für Bild, schreibt `spike-messung.txt` und beendet sich.
use crate::{App, Msg, Tab, UPDATE_NS, VIEW_NS};
use crate::widget::{DRAW_N, DRAW_NS};
use iced::Task;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind { Start, Idle, Tippen, Scrollen, Canvas }
pub struct Bench { kinds: Vec<Kind>, i: usize, frames: usize, last: Option<Instant>, gaps: Vec<f64>, base: (u64, u64, u64, u64), results: Vec<String>, first: bool, wait_until: Option<Instant> }
const FRAMES: usize = 400;

impl Bench {
    pub fn new(k: &str) -> Bench {
        let kinds = match k { "idle" => vec![Kind::Idle], "tippen" => vec![Kind::Tippen], "scrollen" => vec![Kind::Scrollen], "canvas" => vec![Kind::Canvas], _ => vec![Kind::Tippen, Kind::Scrollen, Kind::Canvas] };
        Bench { kinds, i: 0, frames: 0, last: None, gaps: vec![], base: (0, 0, 0, 0), results: vec![], first: true, wait_until: None }
    }
    pub fn needs_big(&self) -> bool { self.kinds.iter().any(|k| matches!(k, Kind::Tippen | Kind::Scrollen)) }
    fn cur(&self) -> Option<Kind> { self.kinds.get(self.i).copied() }
}
fn snap() -> (u64, u64, u64, u64) { (UPDATE_NS.load(Relaxed), VIEW_NS.load(Relaxed), DRAW_NS.load(Relaxed), DRAW_N.load(Relaxed)) }
fn pct(v: &mut Vec<f64>, p: f64) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[((v.len() as f64 - 1.0) * p) as usize] }

pub fn on_frame(app: &mut App, now: Instant) -> Task<Msg> {
    if app.first_frame_exit && app.bench.is_none() {
        let ms = crate::START.get().map(|s| s.elapsed().as_secs_f64() * 1000.0).unwrap_or(0.0);
        println!("Startzeit bis zum ersten Bild: {ms:.0} ms"); return iced::exit()
    }
    let Some(mut b) = app.bench.take() else { return Task::none() };
    if b.first { b.first = false; let ms = crate::START.get().map(|s| s.elapsed().as_secs_f64() * 1000.0).unwrap_or(0.0); b.results.push(format!("Start bis erstes Bild: {ms:.0} ms")); b.wait_until = Some(now + std::time::Duration::from_millis(600)); }
    if let Some(t) = b.wait_until { if now < t { app.bench = Some(b); return Task::none() } b.wait_until = None; b.base = snap(); b.last = None; b.frames = 0; b.gaps.clear(); }
    let Some(kind) = b.cur() else { return finish(app, b) };
    // Szenario voranbringen
    match kind {
        Kind::Tippen => { if let Some(n) = app.cur_name() { if n != "gross.asm" { app.open_tab(Tab::File("gross.asm".into())) } } app.apply_ed(crate::widget::EdMsg::Insert(if b.frames % 40 == 39 { "\n".into() } else { "a".into() })); }
        Kind::Scrollen => { if let Some(e) = app.cur() { e.scroll_by(if b.frames % 200 < 100 { 7.0 } else { -7.0 }) } }
        Kind::Canvas => { app.open_tab(Tab::Studio); let (x, y) = (b.frames % 24, (b.frames / 24) % 21); app.studio.pix[y * 24 + x] = (b.frames % 4) as u8; app.studio.cache.clear(); }
        Kind::Idle | Kind::Start => {}
    }
    if let Some(l) = b.last { b.gaps.push(now.duration_since(l).as_secs_f64() * 1000.0) } b.last = Some(now); b.frames += 1;
    if b.frames >= FRAMES {
        let s = snap(); let n = b.frames as f64; let mut g = std::mem::take(&mut b.gaps);
        let line = format!("{:?}: Bildabstand Mittel {:.2} ms, p95 {:.2} ms, max {:.2} ms | CPU je Bild: update {:.0} µs, view {:.0} µs, draw {:.0} µs ({} draw-Aufrufe)", kind, g.iter().sum::<f64>() / g.len() as f64, pct(&mut g, 0.95), pct(&mut g, 1.0), (s.0 - b.base.0) as f64 / n / 1000.0, (s.1 - b.base.1) as f64 / n / 1000.0, (s.2 - b.base.2) as f64 / (s.3 - b.base.3).max(1) as f64 / 1000.0, s.3 - b.base.3);
        println!("{line}"); b.results.push(line); b.i += 1; b.wait_until = Some(now + std::time::Duration::from_millis(300));
    }
    app.bench = Some(b); Task::none()
}
fn finish(app: &mut App, b: Bench) -> Task<Msg> {
    let _ = std::fs::write("spike-messung.txt", b.results.join("\n") + "\n"); let _ = app; iced::exit()
}
