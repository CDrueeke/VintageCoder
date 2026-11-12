// SPDX-License-Identifier: GPL-3.0-or-later
//! Anzeigen des Trackers: Pegel, Oszilloskop, Spektrum und Tastatur. Sie zeigen das Signal des eingebauten Synthesizers (Hörprobe), nicht den echten Chip.
use crate::synth::VizSnap;
use eframe::egui::{self, pos2, vec2, Color32, Rect, Sense, Stroke};
use std::time::Instant;

pub const BANDS: usize = 24;
pub const FFT_N: usize = 2048;
/// Farben der Stimmen (Tastatur, Spitzenmarken)
pub const VOICE_COLORS: [Color32; 8] = [
    Color32::from_rgb(0x35, 0xc7, 0x59), Color32::from_rgb(0x4f, 0x9d, 0xff), Color32::from_rgb(0xf0, 0xc3, 0x3c), Color32::from_rgb(0xf7, 0x8c, 0x6c),
    Color32::from_rgb(0xc7, 0x92, 0xea), Color32::from_rgb(0xff, 0x8f, 0xc8), Color32::from_rgb(0x5a, 0xe0, 0xe0), Color32::from_rgb(0xef, 0x6b, 0x6b),
];

/// Einstellungen der Anzeige
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VizOpts { pub vu: bool, pub scope: bool, pub spectrum: bool, pub peaks: bool, pub flash: bool, pub follow: bool, pub keys: bool, pub fall: f32 }
impl Default for VizOpts { fn default() -> Self { VizOpts { vu: true, scope: true, spectrum: true, peaks: true, flash: true, follow: true, keys: true, fall: 0.5 } } }

/// Abfallzeit der Balken von voll bis leer in Sekunden (Regler 0..1)
fn fall_secs(fall: f32) -> f32 { 0.25 + fall.clamp(0.0, 1.0) * 1.75 }

/// Pegel (linear 0..1) → Anzeigewert 0..1 auf einer dB-Skala (−48 dB … 0 dB)
pub fn level_of(peak: f32) -> f32 { if peak <= 0.0 { 0.0 } else { ((20.0 * peak.log10() + 48.0) / 48.0).clamp(0.0, 1.0) } }

// ------------------------------------------------------------------------ Spektrum
/// In-place-FFT (Radix 2, `re`/`im` gleich lang, Potenz von 2)
pub fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len(); let mut j = 0;
    for i in 1..n { let mut bit = n >> 1; while j & bit != 0 { j ^= bit; bit >>= 1; } j ^= bit; if i < j { re.swap(i, j); im.swap(i, j); } }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32; let (wr, wi) = (ang.cos(), ang.sin());
        for s in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (s + k, s + k + len / 2);
                let (tr, ti) = (re[b] * cr - im[b] * ci, re[b] * ci + im[b] * cr);
                re[b] = re[a] - tr; im[b] = im[a] - ti; re[a] += tr; im[a] += ti;
                let nr = cr * wr - ci * wi; ci = cr * wi + ci * wr; cr = nr;
            }
        }
        len <<= 1;
    }
}

/// Bandpegel 0..1 (24 Bänder logarithmisch von 60 Hz bis 16 kHz) aus den letzten `FFT_N` Samples der Summe
pub fn spectrum(mix: &[f32], sr: f32) -> [f32; BANDS] {
    let mut out = [0.0f32; BANDS];
    if mix.len() < FFT_N || sr <= 0.0 { return out; }
    let x = &mix[mix.len() - FFT_N..];
    let (mut re, mut im): (Vec<f32>, Vec<f32>) = (x.iter().enumerate().map(|(i, v)| v * (0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / FFT_N as f32).cos())).collect(), vec![0.0; FFT_N]);
    fft(&mut re, &mut im);
    let hz_per_bin = sr / FFT_N as f32; let (lo, hi) = (60.0f32, 16000.0f32.min(sr / 2.0 - 1.0));
    for (b, o) in out.iter_mut().enumerate() {
        let (f0, f1) = (lo * (hi / lo).powf(b as f32 / BANDS as f32), lo * (hi / lo).powf((b + 1) as f32 / BANDS as f32));
        let (i0, i1) = (((f0 / hz_per_bin).floor() as usize).max(1), ((f1 / hz_per_bin).ceil() as usize).max(2).min(FFT_N / 2));
        let mag = (i0..i1.max(i0 + 1)).map(|i| (re[i] * re[i] + im[i] * im[i]).sqrt()).fold(0.0f32, f32::max) / (FFT_N as f32 / 4.0);   // Amplitude eines Sinus
        *o = if mag <= 1e-6 { 0.0 } else { ((20.0 * mag.log10() + 66.0) / 60.0).clamp(0.0, 1.0) };
    }
    out
}

// ------------------------------------------------------------------------ Zustand
/// Geglättete Anzeigewerte (Abfall, Spitzenmarken); wird je Bild mit den Daten der Engine aktualisiert
pub struct VizState {
    pub level: Vec<f32>, pub hold: Vec<f32>, hold_t: Vec<f32>,
    pub master: f32, pub master_hold: f32, master_t: f32,
    pub spec: [f32; BANDS], pub spec_hold: [f32; BANDS], spec_t: [f32; BANDS],
    pub snap: VizSnap, last: Option<Instant>,
    /// Zeitpunkt des letzten Zeilenwechsels (für das Aufleuchten neuer Töne)
    pub row_since: Instant, last_pos: (usize, usize),
}
impl Default for VizState {
    fn default() -> Self { VizState { level: vec![], hold: vec![], hold_t: vec![], master: 0.0, master_hold: 0.0, master_t: 0.0, spec: [0.0; BANDS], spec_hold: [0.0; BANDS], spec_t: [0.0; BANDS], snap: VizSnap::default(), last: None, row_since: Instant::now(), last_pos: (usize::MAX, 0) } }
}

fn follow(level: &mut f32, hold: &mut f32, t: &mut f32, target: f32, dt: f32, fall: f32) {
    if target >= *level { *level = target; } else { *level = (*level - dt / fall).max(target.min(*level)).max(0.0); }
    if target >= *hold { *hold = target; *t = 0.0; } else { *t += dt; if *t > 0.9 { *hold = (*hold - dt * 0.7).max(*level); } }
}

impl VizState {
    /// Neue Daten der Engine einarbeiten; `pos` = (Reihenfolge, Zeile) der Wiedergabe
    pub fn update(&mut self, snap: VizSnap, pos: Option<(usize, usize)>, opts: &VizOpts) {
        let now = Instant::now(); let dt = self.last.map_or(0.016, |t| now.duration_since(t).as_secs_f32()).clamp(0.001, 0.25); self.last = Some(now);
        let fall = fall_secs(opts.fall);
        let n = snap.peaks.len().max(snap.notes.len());
        self.level.resize(n, 0.0); self.hold.resize(n, 0.0); self.hold_t.resize(n, 0.0);
        for i in 0..n { let target = level_of(snap.peaks.get(i).copied().unwrap_or(0.0)); let (l, h, t) = (&mut self.level[i], &mut self.hold[i], &mut self.hold_t[i]); follow(l, h, t, target, dt, fall); }
        follow(&mut self.master, &mut self.master_hold, &mut self.master_t, level_of(snap.master_peak * 1.8), dt, fall);
        let target = spectrum(&snap.mix, snap.sr);
        for b in 0..BANDS { let (l, h, t) = (&mut self.spec[b], &mut self.spec_hold[b], &mut self.spec_t[b]); follow(l, h, t, target[b], dt, fall); }
        if let Some(p) = pos { if p != self.last_pos { self.last_pos = p; self.row_since = now; } } else { self.last_pos = (usize::MAX, 0); }
        // Wellenformen nur ersetzen, wenn die Engine welche geliefert hat (sonst die letzten behalten, damit nichts flackert)
        if !snap.ch_wave.is_empty() || !snap.mix.is_empty() { self.snap = snap; } else { self.snap.notes.clear(); }
    }
    /// Helligkeit 0..1 für das Aufleuchten einer Zelle, in der gerade ein Ton einsetzt (klingt über ~0,35 s aus)
    pub fn flash(&self) -> f32 { (1.0 - self.row_since.elapsed().as_secs_f32() / 0.35).clamp(0.0, 1.0) }
}

// ------------------------------------------------------------------------ Zeichnen
/// Pegelfarbe: grün, ab 62 % bernstein, ab 86 % rot (Tokens, in Hell und Dunkel passend)
fn seg_color(frac: f32) -> Color32 { let k = crate::ui::tokens::t(); if frac <= 0.62 { k.gr } else if frac <= 0.86 { k.am } else { k.rd } }

/// Senkrechter LED-Balken
pub fn vbar(p: &egui::Painter, r: Rect, level: f32, hold: Option<f32>, n: usize) {
    let gap = 1.5; let sh = ((r.height() - gap * (n as f32 - 1.0)) / n as f32).max(1.0);
    for i in 0..n {
        let frac = (i + 1) as f32 / n as f32; let y = r.bottom() - (i + 1) as f32 * sh - i as f32 * gap;
        let c = seg_color(frac); let on = frac <= level + 1e-4;
        p.rect_filled(Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), sh)), 1.0, if on { c } else { c.gamma_multiply(0.14) });
    }
    if let Some(h) = hold.filter(|h| *h > 0.01) { let i = ((h * n as f32).ceil() as usize).clamp(1, n) - 1; let y = r.bottom() - (i + 1) as f32 * sh - i as f32 * gap; p.rect_filled(Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), sh)), 1.0, seg_color((i + 1) as f32 / n as f32)); }
}

/// Waagerechter LED-Balken
pub fn hbar(p: &egui::Painter, r: Rect, level: f32, hold: Option<f32>, n: usize) {
    let gap = 1.5; let sw = ((r.width() - gap * (n as f32 - 1.0)) / n as f32).max(1.0);
    for i in 0..n {
        let frac = (i + 1) as f32 / n as f32; let x = r.left() + i as f32 * (sw + gap);
        let c = seg_color(frac); let on = frac <= level + 1e-4;
        p.rect_filled(Rect::from_min_size(pos2(x, r.top()), vec2(sw, r.height())), 1.0, if on { c } else { c.gamma_multiply(0.14) });
    }
    if let Some(h) = hold.filter(|h| *h > 0.01) { let i = ((h * n as f32).ceil() as usize).clamp(1, n) - 1; p.rect_filled(Rect::from_min_size(pos2(r.left() + i as f32 * (sw + gap), r.top()), vec2(sw, r.height())), 1.0, seg_color((i + 1) as f32 / n as f32)); }
}

/// Oszilloskop: ab einem aufsteigenden Nulldurchgang gezeichnet, damit die Kurve steht; `gain` skaliert die Höhe
pub fn scope(p: &egui::Painter, r: Rect, wave: &[f32], gain: f32, color: Color32) {
    let k = crate::ui::tokens::t();
    p.rect_filled(r, 3.0, k.bg0); p.rect_stroke(r, 3.0, Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
    let grid = Stroke::new(1.0, k.line.gamma_multiply(0.7));
    for k in 1..4 { let x = r.left() + r.width() * k as f32 / 4.0; p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], grid); }
    p.line_segment([pos2(r.left(), r.center().y), pos2(r.right(), r.center().y)], grid);
    if wave.len() < 64 { return; }
    let show = ((r.width() * 1.5) as usize).clamp(64, wave.len() / 2);       // dargestellte Samples
    let start = (0..wave.len() - show).find(|&i| wave[i] <= 0.0 && wave[i + 1] > 0.0).unwrap_or(0);
    let pts: Vec<_> = (0..r.width() as usize).map(|x| { let i = start + x * show / r.width().max(1.0) as usize; pos2(r.left() + x as f32, r.center().y - wave[i.min(wave.len() - 1)] * gain * r.height() * 0.46) }).collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.4, color)));
}

/// Spektrum: 24 senkrechte LED-Balken mit Spitzenmarken
pub fn spectrum_view(p: &egui::Painter, r: Rect, st: &VizState, peaks: bool) {
    p.rect_filled(r, 3.0, crate::ui::tokens::t().bg0);
    let gw = r.width() / BANDS as f32;
    for b in 0..BANDS { vbar(p, Rect::from_min_size(pos2(r.left() + b as f32 * gw + 1.0, r.top() + 2.0), vec2((gw - 2.0).max(2.0), r.height() - 4.0)), st.spec[b], peaks.then_some(st.spec_hold[b]), 16); }
}

/// Klaviatur (3 Oktaven ab C-2): leuchtende Tasten in der Farbe der Stimme; liefert die angeklickte Note (Tracker-Zählung, C-0 = 1)
pub fn keyboard(ui: &mut egui::Ui, size: egui::Vec2, notes: &[Option<u8>]) -> Option<u8> {
    let first = 2 * 12 + 1u8; let keys = 36usize; // C-2 … B-4
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let p = ui.painter_at(rect);
    let white: Vec<usize> = (0..keys).filter(|k| ![1, 3, 6, 8, 10].contains(&(k % 12))).collect();
    let ww = rect.width() / white.len() as f32; let wh = rect.height();
    let color_of = |note: u8| notes.iter().enumerate().find(|(_, n)| **n == Some(note)).map(|(c, _)| VOICE_COLORS[c % VOICE_COLORS.len()]);
    let mut hit = None; let click = if resp.clicked() { resp.interact_pointer_pos() } else { None };
    // weiße Tasten
    for (i, k) in white.iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.left() + i as f32 * ww, rect.top()), vec2(ww - 1.0, wh));
        p.rect_filled(r, 2.0, color_of(first + *k as u8).unwrap_or(Color32::from_rgb(0xe9, 0xec, 0xf2))); p.rect_stroke(r, 2.0, Stroke::new(1.0, crate::theme::grid(80)), egui::StrokeKind::Inside);
    }
    // schwarze Tasten (liegen über den weißen)
    let mut blacks = vec![];
    for k in 0..keys { if [1, 3, 6, 8, 10].contains(&(k % 12)) { let wi = white.iter().position(|w| *w == k - 1).unwrap_or(0); blacks.push((k, Rect::from_min_size(pos2(rect.left() + (wi as f32 + 1.0) * ww - ww * 0.3, rect.top()), vec2(ww * 0.6, wh * 0.6)))); } }
    for (k, r) in &blacks { p.rect_filled(*r, 2.0, color_of(first + *k as u8).unwrap_or(Color32::from_rgb(0x20, 0x24, 0x2d))); }
    if let Some(pt) = click {
        if let Some((k, _)) = blacks.iter().find(|(_, r)| r.contains(pt)) { hit = Some(first + *k as u8); }
        else if let Some(i) = white.iter().enumerate().find(|(i, _)| Rect::from_min_size(pos2(rect.left() + *i as f32 * ww, rect.top()), vec2(ww, wh)).contains(pt)).map(|(_, k)| *k) { hit = Some(first + i as u8); }
    }
    hit
}

// ------------------------------------------------------------------------ Bausteine für die Editoren
/// Einstellungen der Anzeige (gelten für alle Tracker-Fenster der Sitzung)
pub fn opts(ctx: &egui::Context) -> VizOpts { ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(egui::Id::new("viz-opts"), VizOpts::default)) }
pub fn set_opts(ctx: &egui::Context, o: VizOpts) { ctx.data_mut(|d| d.insert_temp(egui::Id::new("viz-opts"), o)); }

/// Höhe des Kanalkopfs mit Pegel und Oszilloskop (0 = ohne)
pub fn head_height(o: &VizOpts) -> f32 { if o.vu || o.scope { 78.0 } else { 0.0 } }

/// Kleiner Schalter an fester Stelle (ohne den Cursor des Layouts zu verschieben: `Ui::put` würde ihn zurücksetzen)
fn mini_button(ui: &mut egui::Ui, r: Rect, label: &str, on: bool, tip: &str, id: (&str, usize)) -> bool {
    let resp = ui.interact(r, ui.id().with(id), Sense::click());
    let fill = if on { crate::theme::accent() } else if resp.hovered() { ui.visuals().widgets.hovered.weak_bg_fill } else { ui.visuals().widgets.inactive.weak_bg_fill };
    ui.painter().rect_filled(r, 3.0, fill);
    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(11.0), if on { Color32::WHITE } else { ui.visuals().text_color() });
    resp.on_hover_text(tip).clicked()
}

/// Kanalkopf: Name, Stumm/Solo, Oszilloskop und zwei Pegelbalken der Stimme `c`
pub fn channel_head(ui: &mut egui::Ui, st: &VizState, o: &VizOpts, audio: &crate::audio::Audio, c: usize, rect: Rect, name: &str) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 5.0, ui.visuals().faint_bg_color); p.rect_stroke(rect, 5.0, Stroke::new(1.0, crate::theme::grid(60)), egui::StrokeKind::Inside);
    p.text(pos2(rect.left() + 8.0, rect.top() + 10.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(12.0), ui.visuals().text_color());
    if audio.source() == crate::audio::Source::Ultimate {
        // Der Strom ist die gemischte Ausgabe: Stimmen einzeln gibt es nicht
        let k = crate::ui::tokens::t();
        let body = Rect::from_min_max(pos2(rect.left() + 6.0, rect.top() + 22.0), pos2(rect.right() - 6.0, rect.bottom() - 6.0));
        p.rect_filled(body, 3.0, k.bg0.gamma_multiply(0.6)); p.rect_stroke(body, 3.0, Stroke::new(1.0, k.line.gamma_multiply(0.6)), egui::StrokeKind::Inside);
        p.text(body.center() - vec2(0.0, 6.0), egui::Align2::CENTER_CENTER, "–", egui::FontId::proportional(16.0), k.dim);
        p.text(body.center() + vec2(0.0, 9.0), egui::Align2::CENTER_CENTER, "nur Summe", egui::FontId::proportional(10.0), k.dim);
        return;
    }
    let (m, so) = audio.switches(c);
    let (mr, sr) = (Rect::from_min_size(pos2(rect.right() - 44.0, rect.top() + 3.0), vec2(19.0, 15.0)), Rect::from_min_size(pos2(rect.right() - 23.0, rect.top() + 3.0), vec2(19.0, 15.0)));
    if mini_button(ui, mr, "M", m, "Stimme stumm schalten", ("viz-m", c)) { audio.set_mute(c, !m); }
    if mini_button(ui, sr, "S", so, "Nur diese Stimme hören (Solo)", ("viz-s", c)) { audio.set_solo(c, !so); }
    let body = Rect::from_min_max(pos2(rect.left() + 6.0, rect.top() + 22.0), pos2(rect.right() - 6.0, rect.bottom() - 6.0));
    let bars_w = if o.vu { 2.0 * 12.0 + 4.0 } else { 0.0 };
    if o.scope { let sr = Rect::from_min_max(body.min, pos2(body.right() - if o.vu { bars_w + 4.0 } else { 0.0 }, body.bottom())); scope(&p, sr, st.snap.ch_wave.get(c).map_or(&[][..], |w| w.as_slice()), 1.0, VOICE_COLORS[c % VOICE_COLORS.len()]); }
    if o.vu {
        let lv = st.level.get(c).copied().unwrap_or(0.0); let hold = st.hold.get(c).copied().unwrap_or(0.0);
        let x0 = body.right() - bars_w;
        vbar(&p, Rect::from_min_size(pos2(x0, body.top()), vec2(12.0, body.height())), lv, o.peaks.then_some(hold), 12);
        vbar(&p, Rect::from_min_size(pos2(x0 + 16.0, body.top()), vec2(12.0, body.height())), (lv - 0.04).max(0.0), o.peaks.then_some((hold - 0.03).max(0.0)), 12);
    }
}

/// Master-Pegel (links/rechts) für die Werkzeugleiste
pub fn master_meter(ui: &mut egui::Ui, st: &VizState, o: &VizOpts) {
    let (r, _) = ui.allocate_exact_size(vec2(96.0, 22.0), Sense::hover());
    let p = ui.painter_at(r);
    hbar(&p, Rect::from_min_size(r.min + vec2(0.0, 2.0), vec2(r.width(), 7.0)), st.master, o.peaks.then_some(st.master_hold), 16);
    hbar(&p, Rect::from_min_size(r.min + vec2(0.0, 12.0), vec2(r.width(), 7.0)), (st.master - 0.03).max(0.0), o.peaks.then_some((st.master_hold - 0.02).max(0.0)), 16);
}

/// Spektrum und Summen-Oszilloskop (Seitenleiste)
pub fn side_views(ui: &mut egui::Ui, st: &VizState, o: &VizOpts) {
    if o.spectrum {
        ui.label(egui::RichText::new("Spektrum").strong());
        let w = ui.available_width().max(100.0);
        let (r, _) = ui.allocate_exact_size(vec2(w, 96.0), Sense::hover()); spectrum_view(&ui.painter_at(r), r, st, o.peaks);
    }
    if o.scope {
        ui.label(egui::RichText::new("Oszilloskop (Summe)").strong());
        let w = ui.available_width().max(100.0);
        let (r, _) = ui.allocate_exact_size(vec2(w, 60.0), Sense::hover()); scope(&ui.painter_at(r), r, &st.snap.mix, 1.8, crate::ui::tokens::t().cy);
    }
}

/// Einstellungen der Anzeige als Ausklappbereich
pub fn settings_ui(ui: &mut egui::Ui, o: &mut VizOpts) {
    egui::CollapsingHeader::new("Anzeige").id_salt("viz-settings").default_open(false).show(ui, |ui| {
        ui.checkbox(&mut o.vu, "Pegel je Stimme"); ui.checkbox(&mut o.scope, "Oszilloskop"); ui.checkbox(&mut o.spectrum, "Spektrum");
        ui.checkbox(&mut o.peaks, "Spitzenmarken halten"); ui.checkbox(&mut o.flash, "Neue Töne aufleuchten lassen"); ui.checkbox(&mut o.follow, "Mitlaufende Zeile hervorheben"); ui.checkbox(&mut o.keys, "Tastatur");
        ui.horizontal(|ui| { ui.label("Abfall"); ui.add(egui::Slider::new(&mut o.fall, 0.0..=1.0).show_value(false)); });
        ui.label(egui::RichText::new("Zeigt das Signal des Tracker-Synthesizers, nicht den Chip.").weak().small());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_finds_a_sine_in_the_right_band() {
        let sr = 44100.0;
        for hz in [120.0f32, 1000.0, 5000.0] {
            let x: Vec<f32> = (0..FFT_N).map(|i| 0.5 * (2.0 * std::f32::consts::PI * hz * i as f32 / sr).sin()).collect();
            let s = spectrum(&x, sr);
            let top = s.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap();
            let band = (0..BANDS).find(|b| { let (f0, f1) = (60.0 * (16000.0f32 / 60.0).powf(*b as f32 / 24.0), 60.0 * (16000.0f32 / 60.0).powf((*b + 1) as f32 / 24.0)); hz >= f0 && hz < f1 }).unwrap();
            assert!((top as i32 - band as i32).abs() <= 1, "{hz} Hz: Band {top}, erwartet {band}");
            assert!(s[top] > 0.7, "Pegel {}", s[top]);
        }
        assert!(spectrum(&vec![0.0; FFT_N], sr).iter().all(|v| *v == 0.0));
    }

    #[test]
    fn level_scale_and_decay() {
        assert_eq!(level_of(0.0), 0.0); assert!((level_of(1.0) - 1.0).abs() < 1e-6); assert!(level_of(0.1) > level_of(0.01));
        let mut st = VizState::default(); let opts = VizOpts::default();
        st.update(VizSnap { peaks: vec![1.0, 0.0], master_peak: 0.5, notes: vec![None, None], ..Default::default() }, None, &opts);
        assert!((st.level[0] - 1.0).abs() < 1e-6 && st.level[1] == 0.0);
        std::thread::sleep(std::time::Duration::from_millis(40));
        st.update(VizSnap { peaks: vec![0.0, 0.0], notes: vec![None, None], ..Default::default() }, None, &opts);
        assert!(st.level[0] < 1.0 && st.level[0] > 0.5, "Pegel fällt langsam: {}", st.level[0]);
        assert!((st.hold[0] - 1.0).abs() < 1e-6, "Spitzenmarke bleibt zunächst stehen");
    }
}
