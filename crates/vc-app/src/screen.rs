// SPDX-License-Identifier: GPL-3.0-or-later
//! Editor für Textbildschirme (der Kachel-Editor steht in `tiles.rs`) samt gemeinsamem Rahmen `Common` (Zeichen- und Farbwahl, Werkzeuge, Zeichensatz-Auswahl).
use crate::bitmap::{MediaEd, Env, Palettes};
use crate::ui::studio;
use eframe::egui::{self, pos2, vec2, Color32, ColorImage, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Zeichensatz-Glyphen als Texturen (weiße Pixel, Einfärbung beim Zeichnen).
/// Mehrfarb-Zeichensätze (4 Pixel je Zeile, Werte 0–3) bekommen zusätzlich je Pixelwert 1–3 eine Maske
/// (doppelt breit), damit jeder Wert mit seiner Farbe gezeichnet werden kann; `tex` zeigt dann die Hires-Deutung
/// derselben Bytes (so zeigt der C64 ein Zeichen, dessen Farb-RAM-Wert unter 8 liegt).
pub(crate) struct Glyphs { pub(crate) tex: Vec<TextureHandle>, layers: Vec<[Option<TextureHandle>; 3]>, pub(crate) mc: bool, mc1: usize, mc2: usize, pf3: Option<usize> }

/// Farben der Mehrfarbwerte, falls ein Bildschirm/Kachelsatz eigene hat (`None`: die des Zeichensatzes):
/// `mc1`, `mc2` = Pixelwerte 1 und 2, `pf3` = Farbe der Zeichen mit Bit 7 am Atari (COLPF3 statt COLPF2)
#[derive(Clone, Copy, Default)]
pub(crate) struct McCols { pub(crate) mc1: Option<usize>, pub(crate) mc2: Option<usize>, pub(crate) pf3: Option<usize> }
impl McCols {
    pub(crate) fn from_colors(c: &Value) -> McCols { let g = |k: &str| c[k].as_u64().map(|v| v as usize); McCols { mc1: g("mc1"), mc2: g("mc2"), pf3: g("pf3") } }
}

impl Glyphs {
    /// Zeichen `code` in `r` zeichnen; `cell` = Farbe der Zelle (Farb-RAM bzw. Vordergrund), `mc` = Farben der Mehrfarbwerte 1 und 2
    /// (`None`: die des Zeichensatzes), `alpha` = Deckkraft. Ergebnis `false`, wenn es das Zeichen nicht gibt.
    /// `hi` = Zeichenbit 7 (Atari ANTIC 4: Pixelwert 3 in COLPF3 statt COLPF2); `code` ist dann der Code ohne Bit 7.
    pub(crate) fn paint(&self, p: &egui::Painter, pal: &Palettes, target: &str, code: u8, r: Rect, cell: usize, mc: McCols, hi: bool, alpha: f32) -> bool {
        let Some(t) = self.tex.get(code as usize) else { return false };
        let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let vic2 = matches!(target, "c64" | "mega65" | "vic20");
        // VIC-II/VIC: Mehrfarbe nur, wenn Bit 3 der Farbe gesetzt ist (dann zählen die unteren 3 Bit); TED/ANTIC 4: immer
        if self.mc && (!vic2 || cell >= 8) {
            let c3 = if vic2 { cell & 7 } else if hi { mc.pf3.or(self.pf3).unwrap_or(cell) } else { cell };
            if let Some(l) = self.layers.get(code as usize) {
                for (k, idx) in [(0usize, mc.mc1.unwrap_or(self.mc1)), (1, mc.mc2.unwrap_or(self.mc2)), (2, c3)] {
                    if let Some(m) = &l[k] { p.image(m.id(), r, full, pal.color(target, idx).gamma_multiply(alpha)); }
                }
            }
        } else {
            p.image(t.id(), r, full, pal.color(target, cell).gamma_multiply(alpha));
        }
        true
    }
}

pub(crate) fn load_charset(root: &Path, rel: &str, ctx: &egui::Context) -> Option<Glyphs> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(root.join(rel)).ok()?).ok()?;
    let frames = v["frames"].as_array()?;
    let all: Vec<Vec<String>> = frames.iter().map(|f| f["rows"].as_array().map(|r| r.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()).collect();
    let prof = v["profile"].as_str().unwrap_or("");
    let mc = prof.contains("multicolor") || prof.contains("antic4") || all.iter().any(|r| r.iter().any(|s| { let n = s.chars().count(); n > 0 && n <= 4 }));
    let on = [255u8; 4];
    let mut tex = Vec::new(); let mut layers = Vec::new();
    for (i, rows) in all.iter().enumerate() {
        let mut px = vec![0u8; 8 * 8 * 4];
        let mut lay: [Vec<u8>; 3] = [vec![0u8; 256], vec![0u8; 256], vec![0u8; 256]];
        let mut used = [false; 3];
        for (y, r) in rows.iter().take(8).enumerate() {
            if mc {
                for (x, c) in r.chars().take(4).enumerate() {
                    let val = c.to_digit(4).unwrap_or(0) as usize;
                    if val == 0 { continue; }
                    for dx in 0..2 {
                        let o = (y * 8 + x * 2 + dx) * 4;
                        if (dx == 0 && val & 2 != 0) || (dx == 1 && val & 1 != 0) { px[o..o + 4].copy_from_slice(&on); }   // Hires-Deutung
                        lay[val - 1][o..o + 4].copy_from_slice(&on);
                    }
                    used[val - 1] = true;
                }
            } else {
                for (x, c) in r.chars().take(8).enumerate() { if c != '0' { let o = (y * 8 + x) * 4; px[o..o + 4].copy_from_slice(&on); } }
            }
        }
        tex.push(ctx.load_texture(format!("glyph-{rel}-{i}"), ColorImage::from_rgba_unmultiplied([8, 8], &px), TextureOptions::NEAREST));
        let mut ls: [Option<TextureHandle>; 3] = [None, None, None];
        if mc { for k in 0..3 { if used[k] { ls[k] = Some(ctx.load_texture(format!("glyph-{rel}-{i}-m{k}"), ColorImage::from_rgba_unmultiplied([8, 8], &lay[k]), TextureOptions::NEAREST)); } } }
        layers.push(ls);
    }
    let col = |k: &str, d: usize| v["colors"][k].as_u64().map_or(d, |x| x as usize);
    Some(Glyphs { tex, layers, mc, mc1: col("mc1", 2), mc2: col("mc2", 5), pf3: v["colors"]["pf3"].as_u64().map(|x| x as usize) })
}

/// ASCII → Bildschirmcode des Zielsystems
pub(crate) fn screen_code(target: &str, ch: char) -> u8 {
    let a = ch as u32;
    if target == "atari8" { return if (32..96).contains(&a) { (a - 32) as u8 } else if a < 32 { (a + 64) as u8 } else { (a & 127) as u8 }; }
    if ch.is_ascii_lowercase() { return (a - 96) as u8; }
    if ch.is_ascii_uppercase() { return (a - 64) as u8; }
    (a & 127) as u8
}
fn code_char(target: &str, code: u8) -> char {
    if target == "atari8" { return char::from_u32(code as u32 + 32).filter(|c| c.is_ascii_graphic()).unwrap_or('·'); }
    match code { 1..=26 => (b'A' + code - 1) as char, 32..=63 => code as char, 0 => '@', _ => '·' }
}

pub(crate) struct Common { pub(crate) exp: Option<crate::exportdlg::ExportDlg>, pub(crate) target: String, pub(crate) rel: String, pub(crate) root: PathBuf, pub(crate) media: Value, pub(crate) charset: Option<Glyphs>, pub(crate) charsets: Vec<String>, pub(crate) undo: Vec<String>, pub(crate) redo: Vec<String>, pub(crate) saved: String, pub(crate) abs: PathBuf, pub(crate) status: String, pub(crate) glyph_ctx: egui::Context }

impl Common {
    pub(crate) fn new(ctx: &egui::Context, root: &Path, rel: &str, data: &Path) -> Result<Common, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| format!("{rel}: {e}"))?).map_err(|e| format!("{rel}: {e}"))?;
        let profiles = vc_media::load_profiles(data)?;
        let p = vc_media::profile(&profiles, media["profile"].as_str().unwrap_or(""))?;
        let target = p.target.clone();
        let charset = media["charset"].as_str().and_then(|c| load_charset(root, c, ctx));
        let mut c = Common { exp: None, target, rel: rel.into(), root: root.into(), charset, charsets: vec![], undo: vec![], redo: vec![], saved: media.to_string(), abs, status: String::new(), media, glyph_ctx: ctx.clone() };
        c.charsets = scan_charsets(root, &c.target, &profiles);
        Ok(c)
    }
    pub(crate) fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    pub(crate) fn push_undo(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear(); }
    pub(crate) fn undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; } } }
    pub(crate) fn redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; } } }
    pub(crate) fn save(&mut self) -> Result<(), String> {
        std::fs::write(&self.abs, vc_media::project::format_media(&self.media)).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = "gespeichert".into(); Ok(())
    }
    pub(crate) fn set_charset(&mut self, rel: Option<String>) {
        self.push_undo(); self.media["charset"] = rel.clone().map_or(Value::Null, Value::from);
        self.charset = rel.and_then(|c| load_charset(&self.root, &c, &self.glyph_ctx));
    }
    pub(crate) fn cnum(&self, key: &str, d: usize) -> usize { self.media["colors"][key].as_u64().map(|v| v as usize).unwrap_or(d) }

    /// Mehrfarbwerte 1 und 2 des Bildschirms (falls er eigene Farben hat), sonst die des Zeichensatzes
    pub(crate) fn mc_cols(&self) -> McCols { McCols::from_colors(&self.media["colors"]) }

    /// Zeichen in `r` mit Vordergrundfarbe (Index, bei C64 = Farb-RAM-Wert) und Hintergrundfarbe zeichnen
    pub(crate) fn glyph(&self, p: &egui::Painter, pal: &Palettes, code: u8, r: Rect, fgi: usize, bg: Option<Color32>) {
        if let Some(b) = bg { p.rect_filled(r, 0.0, b); }
        // Atari: Codes ab 128 sind die Zeichen 0-127 mit Bit 7 (zweite Farbgruppe)
        let (code, hi) = if self.target == "atari8" && code >= 128 { (code & 127, true) } else { (code, false) };
        let drawn = self.charset.as_ref().map_or(false, |g| g.paint(p, pal, &self.target, code, r, fgi, self.mc_cols(), hi, 1.0));
        if !drawn { p.text(r.center(), egui::Align2::CENTER_CENTER, code_char(&self.target, code), egui::FontId::monospace(r.height() * 0.8), pal.color(&self.target, fgi)); }
    }

    /// Speichern, Rückgängig, Wiederholen (mit den Tastenkürzeln)
    pub(crate) fn file_tools(&mut self, ui: &mut egui::Ui) {
        let (s, z, sz) = ui.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::S), i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z), i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)));
        if s { if let Err(e) = self.save() { self.status = e; } } if z { self.undo(); } if sz { self.redo(); }
        if ui.button("Speichern").on_hover_text("Strg/Cmd+S").clicked() { if let Err(e) = self.save() { self.status = e; } }
        if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo")).on_hover_text("Rückgängig (Strg/Cmd+Z)").clicked() { self.undo(); }
        if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo")).on_hover_text("Wiederholen").clicked() { self.redo(); }
        ui.separator();
    }

    pub(crate) fn target_label(&self) -> String { crate::recent::short_name(&self.target).to_string() }

    /// Kopfzeile des Studios: Dateiname, System, Profil
    pub(crate) fn head(&self, ui: &mut egui::Ui) {
        let file = self.rel.rsplit('/').next().unwrap_or(&self.rel);
        let chips = vec![(self.target_label(), crate::ui::widgets::system_color(&self.target).gamma_multiply(1.4)), (self.media["profile"].as_str().unwrap_or("").to_string(), crate::ui::tokens::t().cy)];
        studio::head(ui, &self.rel, &studio::Head { file, dirty: self.dirty(), chips: &chips });
    }

    /// Fuß mit Export-Streifen und Status; zeigt auch den Export-Dialog
    pub(crate) fn foot(&mut self, ui: &mut egui::Ui, env: &Env, info: &str) {
        let out_rel = format!("build/export/{}.asm", self.media["name"].as_str().unwrap_or("export"));
        let fresh = std::fs::metadata(env.root.join(&out_rel)).and_then(|m| m.modified()).ok().zip(std::fs::metadata(&self.abs).and_then(|m| m.modified()).ok()).map(|(o, i)| o >= i);
        let atari = self.target == "atari8";
        let info = format!("Ziel: {}{} · {info}", self.target_label(), if atari { if crate::bitmap::ATARI_NTSC.load(std::sync::atomic::Ordering::Relaxed) { " NTSC" } else { " PAL" } } else { "" });
        let act = studio::foot(ui, &self.rel, &studio::Foot { formats: &["ASM", "INC", "BIN", "PRG", "DATA"], out: fresh.map(|f| (out_rel.as_str(), f && !self.dirty())), info: &info, status: &self.status });
        if act == studio::FootAct::OpenExport { let (rel, kind, name) = (self.rel.clone(), self.media["kind"].as_str().unwrap_or("").to_string(), self.media["name"].as_str().unwrap_or("export").to_string()); crate::exportdlg::open(&mut self.exp, &rel, &kind, &name); }
        let text = vc_media::project::format_media(&self.media);
        let target = format!("Ziel: {}", self.target_label());
        let mut exp = self.exp.take();
        let rel = self.rel.clone();
        if let Some(st) = crate::exportdlg::drive(&mut exp, ui, env, &rel, &text, &target, || self.save()) { self.status = st; }
        self.exp = exp;
    }

    /// Videonorm-Schalter (nur Atari): PAL oder NTSC für die Farbtabelle
    pub(crate) fn video_switch(&self, ui: &mut egui::Ui) {
        if self.target != "atari8" { return; }
        studio::caption(ui, "Videonorm");
        let mut v = if crate::bitmap::ATARI_NTSC.load(std::sync::atomic::Ordering::Relaxed) { "ntsc".to_string() } else { "pal".to_string() };
        if crate::ui::widgets::segmented(ui, &mut v, &[("pal", "PAL"), ("ntsc", "NTSC")]) { crate::bitmap::ATARI_NTSC.store(v == "ntsc", std::sync::atomic::Ordering::Relaxed); }
    }

    pub(crate) fn charset_combo(&mut self, ui: &mut egui::Ui) {
        let cur = self.media["charset"].as_str().unwrap_or("").to_string();
        let mut pick: Option<Option<String>> = None;
        egui::ComboBox::from_id_salt(("cs", &self.rel)).width(180.0).selected_text(if cur.is_empty() { "Ersatzschrift".into() } else { cur.clone() }).show_ui(ui, |ui| {
            if ui.selectable_label(cur.is_empty(), "Ersatzschrift").clicked() { pick = Some(None); }
            for c in &self.charsets { if ui.selectable_label(&cur == c, c).clicked() { pick = Some(Some(c.clone())); } }
        });
        if let Some(p) = pick { self.set_charset(p); }
    }

    pub(crate) fn char_picker(&self, ui: &mut egui::Ui, ch: &mut u8, pal: &Palettes, fg: usize, bg: usize) {
        let n = if self.target == "atari8" { 128 } else { 256 }; let s = 18.0;
        // Mehrfarb-Zeichensatz am C64: die Auswahl zeigt die Zeichen mehrfarbig (Farb-RAM-Bit 3 gesetzt)
        let fgp = if matches!(self.target.as_str(), "c64" | "mega65" | "vic20") && self.charset.as_ref().map_or(false, |g| g.mc) { fg | 8 } else { fg };
        ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
        for row in 0..(n + 15) / 16 { ui.horizontal(|ui| { for i in row * 16..((row + 1) * 16).min(n) {
            let (r, resp) = ui.allocate_exact_size(vec2(s, s), Sense::click());
            self.glyph(&ui.painter_at(r), pal, i as u8, r.shrink(1.0), fgp, Some(pal.color(&self.target, bg)));
            if i as u8 == *ch { ui.painter().rect_stroke(r, 1.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside); }
            if resp.clicked() { *ch = i as u8; }
        } }); }
    }

    pub(crate) fn swatches(&self, ui: &mut egui::Ui, pal: &Palettes, cur: usize) -> Option<usize> {
        let mut out = None; let atari = self.target == "atari8"; let (n, per, sz) = if atari { (128, 16, 11.0) } else { (16, 8, 20.0) };
        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
        for row in 0..n / per { ui.horizontal(|ui| { for k in 0..per {
            let idx = if atari { (row * per + k) * 2 } else { row * per + k };
            let (r, resp) = ui.allocate_exact_size(vec2(sz, sz), Sense::click());
            ui.painter().rect_filled(r, 2.0, pal.color(&self.target, idx));
            ui.painter().rect_stroke(r, 2.0, Stroke::new(1.0, crate::theme::dim().gamma_multiply(0.8)), egui::StrokeKind::Inside); // Rand, sonst ist Weiß im hellen Theme unsichtbar
            if idx == cur { ui.painter().rect_stroke(r, 2.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Outside); }
            if resp.clicked() { out = Some(idx); }
        } }); }
        out
    }
}

pub(crate) fn scan_charsets(root: &Path, target: &str, profiles: &[vc_media::Profile]) -> Vec<String> {
    let mut out = vec![];
    fn walk(root: &Path, dir: &Path, depth: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with('.') || n == "build" || n == "target" { continue; }
            if p.is_dir() { if depth < 5 { walk(root, &p, depth + 1, out); } } else if vc_media::naming::is_data_file(&n) { if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")); } } }
    }
    walk(root, root, 0, &mut out);
    out.retain(|rel| std::fs::read_to_string(root.join(rel)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).map_or(false, |v| v["kind"] == "charset" && profiles.iter().any(|p| Some(p.id.as_str()) == v["profile"].as_str() && p.target == target)));
    out.sort(); out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool { Pen, Text, Fill, Rect, Pick, Erase }

// ------------------------------------------------------------------------ Textbildschirm
pub struct ScreenEd { decomp: Option<crate::screenbank::DecompDlg>, profiles: Vec<vc_media::Profile>, c: Common, w: usize, h: usize, color_ram: bool, tool: Tool, ch: u8, col: usize, cur: (usize, usize), cell: f32, fit: bool, avail: egui::Vec2, drag: Option<(usize, usize)> }

impl ScreenEd {
    pub fn open(ctx: &egui::Context, root: &Path, rel: &str, data: &Path) -> Result<ScreenEd, String> {
        let c = Common::new(ctx, root, rel, data)?;
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, c.media["profile"].as_str().unwrap_or(""))?;
        let col = c.cnum("fg", 1);
        Ok(ScreenEd { decomp: None, profiles: profiles.clone(), w: p.width, h: p.height, color_ram: p.raw["colorRam"].as_bool().unwrap_or(false), tool: Tool::Pen, ch: 1, col, cur: (0, 0), cell: 18.0, fit: true, avail: egui::Vec2::ZERO, drag: None, c })
    }
    #[cfg(test)] pub(crate) fn open_decomp(&mut self) { self.decomp = Some(crate::screenbank::DecompDlg::new()); }
    #[cfg(test)] pub(crate) fn has_decomp(&self) -> bool { self.decomp.is_some() }
    fn code(&self, x: usize, y: usize) -> u8 { self.c.media["chars"][y][x].as_u64().unwrap_or(32) as u8 }
    fn put(&mut self, x: usize, y: usize, code: u8, col: usize) { self.c.media["chars"][y][x] = json!(code); if self.color_ram { self.c.media["attrs"][y][x] = json!(col); } }
    fn flood(&mut self, x: usize, y: usize, code: u8) {
        let from = self.code(x, y); if from == code { return; } let mut st = vec![(x, y)];
        while let Some((px, py)) = st.pop() { if self.code(px, py) != from { continue; } let col = self.col; self.put(px, py, code, col);
            if px > 0 { st.push((px - 1, py)); } if py > 0 { st.push((px, py - 1)); } if px + 1 < self.w { st.push((px + 1, py)); } if py + 1 < self.h { st.push((px, py + 1)); } }
    }
    fn rect(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
        let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1)); let mut v = vec![];
        for x in x0..=x1 { v.push((x, y0)); v.push((x, y1)); } for y in y0 + 1..y1 { v.push((x0, y)); v.push((x1, y)); } v
    }
}

impl MediaEd for ScreenEd {
    fn dirty(&self) -> bool { self.c.dirty() }
    fn save(&mut self) -> Result<(), String> { self.c.save() }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let pal = env.pal;
        let id = self.c.rel.clone();
        self.c.head(ui);
        studio::tools(ui, &id, |ui| {
            self.c.file_tools(ui);
            { for (t, n) in [(Tool::Pen, "Zeichen"), (Tool::Text, "Text"), (Tool::Fill, "Füllen"), (Tool::Rect, "Rahmen"), (Tool::Pick, "Pipette"), (Tool::Erase, "Löschen")] { ui.selectable_value(&mut self.tool, t, n); } ui.separator(); ui.label("Zoom"); if ui.add(egui::Slider::new(&mut self.cell, 8.0..=32.0).show_value(false)).changed() { self.fit = false; } ui.checkbox(&mut self.fit, "Anpassen").on_hover_text("Zoom automatisch an den freien Platz anpassen (Strg+Mausrad oder Regler stellt ihn fest)"); ui.separator();
                if ui.button("In Kacheln zerlegen …").on_hover_text("Gleiche Blöcke zu Kacheln machen: Kachelsatz und Screen-Bank mit Vorschau der Speichergröße").clicked() { self.decomp = Some(crate::screenbank::DecompDlg::new()); } }
        });
        if let Some(mut d) = self.decomp.take() {
            if let Ok(p) = vc_media::profile(&self.profiles, self.c.media["profile"].as_str().unwrap_or("")) { if d.show(ui.ctx(), &self.c.media, p, &self.profiles, &self.c.root, &self.c.rel) { self.decomp = Some(d); } }
        }
        let info = format!("{}×{} · Zoom {:.0} %", self.w, self.h, self.cell / 18.0 * 100.0);
        self.c.foot(ui, env, &info);
        studio::right(ui, &id, 320.0, |ui| {
            self.c.video_switch(ui);
            studio::caption(ui, "Zeichensatz"); self.c.charset_combo(ui);
            ui.label(RichText::new(format!("Zeichen {} ($ {:02X})", self.ch, self.ch)).strong());
            let (fg, bg) = (if self.color_ram { self.col } else { self.c.cnum("fg", 14) }, self.c.cnum("bg", 6)); self.c.char_picker(ui, &mut self.ch, pal, fg, bg);
            if self.color_ram { ui.label(RichText::new("Zeichenfarbe").strong()); if let Some(i) = self.c.swatches(ui, pal, self.col) { self.col = i; } }
            studio::caption(ui, "Hintergrund"); if let Some(i) = self.c.swatches(ui, pal, self.c.cnum("bg", 6)) { self.c.push_undo(); self.c.media["colors"]["bg"] = json!(i); }
            if matches!(self.c.target.as_str(), "c64" | "vic20") { studio::caption(ui, "Rahmen"); if let Some(i) = self.c.swatches(ui, pal, self.c.cnum("border", 14)) { self.c.push_undo(); self.c.media["colors"]["border"] = json!(i); } }
            else { ui.label(RichText::new("Vordergrund (Helligkeit)").strong()); if let Some(i) = self.c.swatches(ui, pal, self.c.cnum("fg", 14)) { self.c.push_undo(); self.c.media["colors"]["fg"] = json!(i); } }
        });
        { let a = ui.available_size(); if a != self.avail { self.avail = a; ui.ctx().request_repaint(); } }
        studio::center(ui, |ui| {
            let pad = if matches!(self.c.target.as_str(), "c64" | "vic20") { 16.0 } else { 8.0 };
            if self.fit { self.cell = studio::fit_zoom(self.avail, egui::vec2(self.w as f32, self.h as f32), egui::vec2(pad * 2.0 + 24.0, pad * 2.0 + 48.0), 8.0, 32.0); }
            let cz = self.cell;
            let (outer, resp) = ui.allocate_exact_size(vec2(self.w as f32 * cz + pad * 2.0, self.h as f32 * cz + pad * 2.0), Sense::click_and_drag());
            if resp.hovered() { let z = ui.input(|i| i.zoom_delta()); if z != 1.0 { self.cell = (self.cell * z).clamp(8.0, 32.0); self.fit = false; } }
            let p = ui.painter_at(outer); let inner = outer.shrink(pad);
            let border = self.c.cnum("border", self.c.cnum("bg", 6)); p.rect_filled(outer, 0.0, pal.color(&self.c.target, border));
            p.rect_filled(inner, 0.0, pal.color(&self.c.target, self.c.cnum("bg", 6)));
            for y in 0..self.h { for x in 0..self.w {
                let code = self.code(x, y); let r = Rect::from_min_size(pos2(inner.left() + x as f32 * cz, inner.top() + y as f32 * cz), vec2(cz, cz));
                let fgi = if self.color_ram { self.c.media["attrs"][y][x].as_u64().map(|v| v as usize).unwrap_or(1) } else { self.c.cnum("fg", 14) };
                if code != 32 || self.c.charset.is_some() { self.c.glyph(&p, pal, code, r, fgi, None); }
            } }
            let (gw, gh) = (self.w, self.h);
            let cell_at = |pt: egui::Pos2| -> Option<(usize, usize)> { if !inner.contains(pt) { return None; } let (x, y) = (((pt.x - inner.left()) / cz) as usize, ((pt.y - inner.top()) / cz) as usize); (x < gw && y < gh).then_some((x, y)) };
            if let Some(c) = resp.interact_pointer_pos().and_then(&cell_at) { if resp.drag_started() || resp.clicked() || resp.secondary_clicked() {
                self.cur = c; resp.request_focus();
                match self.tool {
                    Tool::Pick => { self.ch = self.code(c.0, c.1); if self.color_ram { self.col = self.c.media["attrs"][c.1][c.0].as_u64().unwrap_or(1) as usize; } self.tool = Tool::Pen; }
                    Tool::Text => {}
                    Tool::Fill => { self.c.push_undo(); let code = self.ch; self.flood(c.0, c.1, code); }
                    Tool::Rect => { self.drag = Some(c); }
                    _ => { self.c.push_undo(); let code = if self.tool == Tool::Erase || resp.secondary_clicked() { 32 } else { self.ch }; let col = self.col; self.put(c.0, c.1, code, col); }
                }
            } else if resp.dragged() && matches!(self.tool, Tool::Pen | Tool::Erase) { self.cur = c; let code = if self.tool == Tool::Erase || resp.dragged_by(egui::PointerButton::Secondary) { 32 } else { self.ch }; let col = self.col; self.put(c.0, c.1, code, col); } }
            if resp.drag_stopped() && self.tool == Tool::Rect { if let (Some(a), Some(b)) = (self.drag.take(), resp.interact_pointer_pos().and_then(&cell_at)) { self.c.push_undo(); let (code, col) = (self.ch, self.col); for (x, y) in Self::rect(a, b) { self.put(x, y, code, col); } } }
            let cr = Rect::from_min_size(pos2(inner.left() + self.cur.0 as f32 * cz, inner.top() + self.cur.1 as f32 * cz), vec2(cz, cz)); p.rect_stroke(cr, 0.0, Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Inside);
            if resp.has_focus() { ui.input(|i| for e in &i.events { if let egui::Event::Text(t) = e { if self.tool == Tool::Text { for ch in t.chars() { self.c.push_undo(); let (x, y) = self.cur; let col = self.col; self.put(x, y, screen_code(&self.c.target, ch), col); self.cur = if x + 1 < self.w { (x + 1, y) } else { (0, (y + 1).min(self.h - 1)) }; } } } }); }
            ui.label(RichText::new(format!("{}×{} · Spalte {}, Zeile {} · Code {}", self.w, self.h, self.cur.0, self.cur.1, self.code(self.cur.0, self.cur.1))).weak().small());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../") }

    #[test]
    fn screen_open_edit_save() {
        let (data, ctx) = (root().join("data"), egui::Context::default());
        let dir = std::env::temp_dir().join(format!("vc-sc-{}", std::process::id())); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        for (f, t) in [("font", "vcchar"), ("title", "vcscreen")] { std::fs::copy(root().join(format!("examples/c64/gfx_demo/graphics/{f}.{t}")), dir.join(format!("graphics/{f}.{t}"))).unwrap(); }
        let mut s = ScreenEd::open(&ctx, &dir, "graphics/title.vcscreen", &data).unwrap();
        assert_eq!((s.w, s.h), (40, 25)); assert!(s.c.charset.is_some(), "Zeichensatz aus dem Projekt geladen"); assert_eq!(s.c.charsets, vec!["graphics/font.vcchar".to_string()]);
        s.c.push_undo(); s.put(3, 4, 7, 5); s.flood(10, 10, 9); assert!(s.dirty()); s.save().unwrap();
        let s2 = ScreenEd::open(&ctx, &dir, "graphics/title.vcscreen", &data).unwrap();
        assert_eq!(s2.code(3, 4), 7); assert_eq!(s2.c.media["attrs"][4][3], 5);
        assert_eq!(screen_code("c64", 'a'), 1); assert_eq!(screen_code("atari8", 'A'), 33);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn studios_zeichnen_mit_exportdialog() {
        let data = root().join("data");
        let pal = Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let tmp = std::env::temp_dir().join(format!("vc_screen_ui_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &tmp, data: &data };
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        let sc = vc_media::project::create(&tmp, &data, "title", "c64.screen.text", "").unwrap()["path"].as_str().unwrap().to_string();
        let frame = |ui_fn: &mut dyn FnMut(&mut egui::Ui)| { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))), ..Default::default() }, |ui| ui_fn(ui)); o.textures_delta.clear(); } };
        let mut a = ScreenEd::open(&ctx, &tmp, &sc, &data).unwrap();
        a.c.exp = Some(crate::exportdlg::ExportDlg::new(&sc, "screen", "title"));
        frame(&mut |ui| MediaEd::ui(&mut a, ui, &env));
        // kleine Fenster: Dialog bleibt offen, nichts bricht
        for (w, h) in [(330.0, 500.0), (640.0, 480.0)] {
            let small = |ui_fn: &mut dyn FnMut(&mut egui::Ui)| { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| ui_fn(ui)); o.textures_delta.clear(); } };
            small(&mut |ui| MediaEd::ui(&mut a, ui, &env));
            assert!(a.c.exp.is_some(), "Export-Dialog offen bei {w}x{h}");
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
