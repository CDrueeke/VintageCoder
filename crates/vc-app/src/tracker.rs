// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker-Editor (Songs für SID/POKEY) und PLAY-Tracker (MEGA65) mit Hörprobe über die Software-Synthese.
use crate::bitmap::{export_media, MediaEd, Env};
use crate::synth::{self, fmt_cell, parse_cell, Instr, SongData, Wave};
use crate::ui::tokens::{t, tint};
use crate::ui::widgets;
use eframe::egui::{self, pos2, vec2, Color32, Rect, RichText, Sense, Stroke};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

// ------------------------------------------------------------------------ Eingabe
struct Cursor { row: usize, ch: usize, oct: u8, ins: usize }

enum Act { Set(usize, usize, String, Option<(u8, usize)>), Clear(usize, usize), Toggle, Undo, Redo, Save, Moved }

fn piano(key: egui::Key) -> Option<u8> {
    use egui::Key::*;
    Some(match key { Z => 0, S => 1, X => 2, D => 3, C => 4, V => 5, G => 6, B => 7, H => 8, N => 9, J => 10, M => 11,
        Q => 12, Num2 => 13, W => 14, Num3 => 15, E => 16, R => 17, Num5 => 18, T => 19, Num6 => 20, Y => 21, Num7 => 22, U => 23, I => 24, _ => return None })
}

fn handle_keys(cur: &mut Cursor, rows: usize, cols: usize, n_ins: usize, events: &[egui::Event]) -> Vec<Act> {
    use egui::Key;
    let mut out = vec![];
    for e in events {
        let egui::Event::Key { key, pressed: true, modifiers, .. } = e else { continue };
        if modifiers.command {
            match key { Key::S => out.push(Act::Save), Key::Z => out.push(if modifiers.shift { Act::Redo } else { Act::Undo }), _ => {} }
            continue;
        }
        match key {
            Key::ArrowDown => { cur.row = (cur.row + 1) % rows; out.push(Act::Moved); }
            Key::ArrowUp => { cur.row = (cur.row + rows - 1) % rows; out.push(Act::Moved); }
            Key::ArrowRight => { cur.ch = (cur.ch + 1) % cols; out.push(Act::Moved); }
            Key::ArrowLeft => { cur.ch = (cur.ch + cols - 1) % cols; out.push(Act::Moved); }
            Key::PageDown => { cur.row = (cur.row + 8).min(rows - 1); out.push(Act::Moved); }
            Key::PageUp => { cur.row = cur.row.saturating_sub(8); out.push(Act::Moved); }
            Key::Home => { cur.row = 0; out.push(Act::Moved); }
            Key::End => { cur.row = rows - 1; out.push(Act::Moved); }
            Key::Space => out.push(Act::Toggle),
            Key::F9 => cur.oct = cur.oct.saturating_sub(1),
            Key::F10 => cur.oct = (cur.oct + 1).min(7),
            Key::OpenBracket => cur.ins = cur.ins.saturating_sub(1),
            Key::CloseBracket => cur.ins = (cur.ins + 1).min(n_ins.saturating_sub(1)),
            Key::Delete | Key::Backspace | Key::Period => { out.push(Act::Clear(cur.row, cur.ch)); cur.row = (cur.row + 1) % rows; out.push(Act::Moved); }
            Key::Num1 => { out.push(Act::Set(cur.row, cur.ch, "=== --".into(), None)); cur.row = (cur.row + 1) % rows; out.push(Act::Moved); }
            k => if let Some(semi) = piano(*k) {
                let note = cur.oct as u32 * 12 + semi as u32 + 1;
                if note < 255 { out.push(Act::Set(cur.row, cur.ch, fmt_cell(note as u8, cur.ins as u8), Some((note as u8, cur.ins)))); cur.row = (cur.row + 1) % rows; out.push(Act::Moved); }
            },
        }
    }
    out
}

// ------------------------------------------------------------------------ Raster-Widget
struct GridView<'a> { id: egui::Id, rows: usize, cols: usize, head: &'a dyn Fn(usize) -> String, cell: &'a dyn Fn(usize, usize) -> String, play_row: Option<usize>, cur: (usize, usize), scroll_to: bool,
    /// Spaltenbreite, Höhe des Kanalkopfs (0 = ohne) und dessen Zeichner; `flash` liefert die Helligkeit 0..1 für das Aufleuchten einer Zelle
    cw: f32, head_h: f32, head_draw: Option<&'a dyn Fn(&mut egui::Ui, usize, Rect)>, flash: Option<&'a dyn Fn(usize, usize) -> f32> }

/// Zeichnet das Raster; Kopfzeile bleibt oben stehen. Gibt (angeklickte Zelle, Tastenereignisse) zurück.
fn grid(ui: &mut egui::Ui, g: GridView) -> (Option<(usize, usize)>, Vec<egui::Event>) {
    let (rh, cw, nw) = (20.0, g.cw, 34.0);
    let mono = egui::FontId::monospace(13.0); let k = t(); let dim = k.mute;
    let mut clicked = None; let mut events = vec![];
    egui::ScrollArea::horizontal().id_salt((g.id, "h")).auto_shrink([false, false]).show(ui, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let (hr, _) = ui.allocate_exact_size(vec2(nw + cw * g.cols as f32, rh + 4.0 + g.head_h), Sense::hover());
            let p = ui.painter_at(hr);
            if let (Some(draw), true) = (g.head_draw, g.head_h > 0.0) { for c in 0..g.cols { draw(ui, c, Rect::from_min_size(pos2(hr.left() + nw + c as f32 * cw + 2.0, hr.top() + 2.0), vec2(cw - 4.0, g.head_h - 4.0))); } }
            for c in 0..g.cols { p.text(pos2(hr.left() + nw + c as f32 * cw + cw / 2.0, hr.bottom() - (rh + 4.0) / 2.0), egui::Align2::CENTER_CENTER, if g.head_h > 0.0 { String::new() } else { (g.head)(c) }, egui::FontId::proportional(12.0), dim); }
            p.line_segment([hr.left_bottom(), hr.right_bottom()], Stroke::new(1.0, k.line));
            egui::ScrollArea::vertical().id_salt((g.id, "v")).auto_shrink([false, false]).show(ui, |ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(nw + cw * g.cols as f32, rh * g.rows as f32), Sense::click_and_drag());
                let p = ui.painter_at(rect.expand(1.0));
                for r in 0..g.rows {
                    let y = rect.top() + r as f32 * rh; let row_rect = Rect::from_min_size(pos2(rect.left(), y), vec2(rect.width(), rh));
                    if g.play_row == Some(r) { p.rect_filled(row_rect, 0.0, k.cy.gamma_multiply(0.30)); p.rect_filled(Rect::from_min_size(row_rect.min, vec2(3.0, rh)), 0.0, k.cy); }
                    else if r % 4 == 0 { p.rect_filled(row_rect, 0.0, k.bg2); }
                    p.text(pos2(rect.left() + nw - 6.0, y + rh / 2.0), egui::Align2::RIGHT_CENTER, format!("{:02X}", r), mono.clone(), if r % 4 == 0 { k.mute } else { k.dim });
                    for c in 0..g.cols {
                        let x = rect.left() + nw + c as f32 * cw; let t = (g.cell)(r, c);
                        let cr = Rect::from_min_size(pos2(x, y), vec2(cw, rh));
                        let mut flashing = false;
                        if let Some(f) = g.flash { let a = f(r, c); if a > 0.01 { flashing = a > 0.05; p.rect_filled(cr.shrink2(vec2(6.0, 2.0)), 3.0, k.am.gamma_multiply(a * 0.45)); } }
                        if g.cur == (r, c) { p.rect_filled(cr.shrink2(vec2(2.0, 0.0)), 3.0, tint(k.cy)); p.rect_stroke(cr.shrink2(vec2(2.0, 0.0)), 3.0, Stroke::new(1.0, k.cy), egui::StrokeKind::Inside); }
                        let (empty, stop) = (t.starts_with("---"), t.starts_with("==="));
                        if empty || stop {
                            p.text(cr.center(), egui::Align2::CENTER_CENTER, t, mono.clone(), if stop { k.am } else { k.dim });
                        } else {
                            // Note in Textfarbe, Instrument in Cyan: beide Teile um die Mitte der Zelle
                            let (note, rest) = t.split_at(t.char_indices().nth(3).map_or(t.len(), |(i, _)| i));
                            let chw = p.layout_no_wrap("0".into(), mono.clone(), k.fg).size().x;
                            let x0 = cr.center().x - t.chars().count() as f32 * chw / 2.0;
                            p.text(pos2(x0, cr.center().y), egui::Align2::LEFT_CENTER, note, mono.clone(), k.fg);
                            p.text(pos2(x0 + 3.0 * chw, cr.center().y), egui::Align2::LEFT_CENTER, rest, mono.clone(), if flashing { k.fg } else { k.cy });   // auf dem Aufleuchten ist Cyan nicht lesbar
                        }
                        if c > 0 { p.line_segment([cr.left_top(), cr.left_bottom()], Stroke::new(1.0, k.line)); }
                    }
                }
                if resp.clicked() || resp.drag_started() {
                    resp.request_focus();
                    if let Some(pt) = resp.interact_pointer_pos() { let (c, r) = (((pt.x - rect.left() - nw) / cw).floor(), ((pt.y - rect.top()) / rh).floor()); if c >= 0.0 && r >= 0.0 && (c as usize) < g.cols && (r as usize) < g.rows { clicked = Some((r as usize, c as usize)); } }
                }
                ui.memory_mut(|m| m.set_focus_lock_filter(resp.id, egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, tab: true, escape: false }));
                if resp.has_focus() { events = ui.input(|i| i.events.clone()); }
                if g.scroll_to { let y = rect.top() + g.cur.0 as f32 * rh; ui.scroll_to_rect(Rect::from_min_size(pos2(rect.left(), y - rh), vec2(10.0, rh * 3.0)), None); }
            });
        });
    });
    (clicked, events)
}

// ------------------------------------------------------------------------ gemeinsames Modell
struct Model { rel: String, abs: PathBuf, media: Value, undo: Vec<String>, redo: Vec<String>, saved: String, status: String, exp: Option<crate::exportdlg::ExportDlg> }
impl Model {
    fn open(root: &Path, rel: &str) -> Result<Model, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| format!("{rel}: {e}"))?).map_err(|e| format!("{rel}: {e}"))?;
        Ok(Model { rel: rel.into(), abs, saved: media.to_string(), media, undo: vec![], redo: vec![], status: String::new(), exp: None })
    }
    fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    fn push_undo(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 200 { self.undo.remove(0); } self.redo.clear(); }
    fn do_undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; } } }
    fn do_redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; } } }
    fn save(&mut self) -> Result<(), String> { std::fs::write(&self.abs, vc_media::project::format_media(&self.media)).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = "gespeichert".into(); Ok(()) }
    fn toolbar(&mut self, ui: &mut egui::Ui, env: &Env) {
        if ui.button("Speichern").clicked() { if let Err(e) = self.save() { self.status = e; } }
        if ui.button("Exportieren …").on_hover_text("Format, Adresse und Vorschau wählen, dann nach build/export/ schreiben").clicked() || crate::exportdlg::wanted() {
            let (kind, name) = (self.media["kind"].as_str().unwrap_or("").to_string(), self.media["name"].as_str().unwrap_or("export").to_string());
            crate::exportdlg::open(&mut self.exp, &self.rel, &kind, &name);
        }
                if ui.button("In Code einfügen").on_hover_text("Exportiert und fügt .include in den zuletzt benutzten Quelltext ein").clicked() { match self.save() { Ok(()) => { export_media(env, &self.rel.clone(), self.media["kind"].as_str().unwrap_or("")); env.reqs.insert_asset(&self.rel); } Err(e) => self.status = e } }
        if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo")).clicked() { self.do_undo(); }
        if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo")).clicked() { self.do_redo(); }
    }
    /// Export-Dialog zeigen und seine Entscheidung ausführen (am Ende von `ui` aufrufen)
    fn export_dialog(&mut self, ui: &mut egui::Ui, env: &Env, target: &str) {
        if self.exp.is_none() { return; }
        let text = vc_media::project::format_media(&self.media);
        let mut exp = self.exp.take();
        let rel = self.rel.clone();
        if let Some(st) = crate::exportdlg::drive(&mut exp, ui, env, &rel, &text, target, || self.save()) { self.status = st; }
        self.exp = exp;
    }
    fn instruments(&self, pokey: bool) -> Vec<Instr> { self.media["instruments"].as_array().map(|a| a.iter().map(|i| Instr::from_json(i, pokey)).collect()).unwrap_or_default() }
}

/// Hüllkurve zeichnen (SID: ADSR-Verlauf als Kurve, POKEY: 16 Lautstärkebalken mit Wiederholungsstelle)
fn envelope_plot(ui: &mut egui::Ui, ins: &Instr) {
    let k = t();
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width().min(300.0), 84.0), Sense::hover());
    let p = ui.painter_at(r); p.rect_filled(r, 4.0, k.bg0); p.rect_stroke(r, 4.0, Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
    let line = Stroke::new(2.0, k.cy);
    match ins {
        Instr::Sid { a, d, s, r: rel, .. } => {
            let (ta, td, tr) = ((synth::SID_A[*a]).max(2.0), synth::SID_D[*d].max(6.0), synth::SID_D[*rel].max(6.0));
            let total = ta + td * 1.2 + 120.0 + tr; let x = |t: f32| r.left() + 6.0 + t / total * (r.width() - 12.0); let y = |l: f32| r.bottom() - 6.0 - l * (r.height() - 12.0);
            let sus = *s as f32 / 15.0;
            let pts = vec![pos2(x(0.0), y(0.0)), pos2(x(ta), y(1.0)), pos2(x(ta + td), y(sus)), pos2(x(ta + td * 1.2 + 120.0), y(sus)), pos2(x(total), y(0.0))];
            let mut fill = pts.clone(); fill.push(pos2(x(total), y(0.0))); fill.push(pos2(x(0.0), y(0.0)));
            p.add(egui::Shape::convex_polygon(fill, tint(k.cy), Stroke::NONE));
            p.add(egui::Shape::line(pts, line));
            for (tx, name) in [(x(ta / 2.0), "A"), (x(ta + td / 2.0), "D"), (x(ta + td * 1.2 + 60.0), "S"), (x(total - tr / 2.0), "R")] { p.text(pos2(tx, r.top() + 9.0), egui::Align2::CENTER_CENTER, name, egui::FontId::monospace(10.0), k.dim); }
        }
        Instr::Pokey { env, loop_at, .. } => {
            let w = (r.width() - 12.0) / 16.0;
            for (i, v) in env.iter().take(16).enumerate() {
                let h = *v as f32 / 15.0 * (r.height() - 22.0);
                let col = if i >= *loop_at { k.cy } else { k.cy.gamma_multiply(0.55) };
                p.rect_filled(Rect::from_min_size(pos2(r.left() + 6.0 + i as f32 * w, r.bottom() - 6.0 - h), vec2(w - 1.5, h)), 1.0, col);
            }
            if *loop_at < 16 {
                let x = r.left() + 6.0 + *loop_at as f32 * w - 0.5;
                p.line_segment([pos2(x, r.top() + 4.0), pos2(x, r.bottom() - 4.0)], Stroke::new(1.0, k.am));
                p.text(pos2(x + 3.0, r.top() + 8.0), egui::Align2::LEFT_CENTER, "↺ ab hier", egui::FontId::monospace(10.0), k.am);
            }
        }
    }
}

/// Abspielknopf: Cyan gefüllt, beim Spielen „Stopp“ in Rot
fn play_button(playing: bool, label: &str) -> egui::Button<'static> {
    let k = t();
    if playing { egui::Button::new(RichText::new("■ Stopp").strong().color(Color32::WHITE)).fill(k.rd) } else { egui::Button::new(RichText::new(label.to_string()).strong().color(k.on_cy)).fill(k.cy) }
}

/// Kopfzeile: Chip-Marke (Chip, Stimmen, Format), Dateiname und Änderungsmarke
fn header(ui: &mut egui::Ui, rel: &str, dirty: bool, chips: &[(String, egui::Color32)]) {
    let k = t();
    ui.horizontal_wrapped(|ui| {
        for (text, c) in chips { widgets::chip(ui, text, *c); }
        ui.label(RichText::new(rel.rsplit('/').next().unwrap_or(rel)).font(egui::FontId::new(14.0, crate::fonts::ui_strong())));
        if dirty { widgets::chip(ui, "geändert", k.am); }
    });
}

/// Export-Streifen: ein Knopf je Format
fn export_strip(ui: &mut egui::Ui, env: &Env, rel: &str, formats: &[(&str, &str)], status: &mut String) {
    let k = t();
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Export").strong().color(k.mute));
        for (fmt, hint) in formats {
            if ui.button(RichText::new(fmt.to_uppercase()).monospace()).on_hover_text(*hint).clicked() {
                match vc_media::project::export(env.root, env.data, rel, fmt, None, None, 10, 10) {
                    Ok(v) => {
                        let files: Vec<String> = v["files"].as_array().map(|a| a.iter().filter_map(|x| x["path"].as_str().map(String::from)).collect()).unwrap_or_default();
                        *status = format!("{}: {}", fmt.to_uppercase(), files.join(", "));
                        env.reqs.info(format!("Export {rel}: {}", files.join(", ")));
                    }
                    Err(e) => { *status = e.clone(); env.reqs.error(format!("Export {rel}: {e}")); }
                }
            }
        }
        if !status.is_empty() { ui.label(RichText::new(status.as_str()).small().color(k.dim)); }
    });
}

/// Instrument bearbeiten; gibt true zurück, wenn sich etwas geändert hat
fn instrument_editor(ui: &mut egui::Ui, v: &mut Value, pokey: bool) -> bool {
    let mut ch = false;
    let mut name = v["name"].as_str().unwrap_or("").to_string();
    if ui.text_edit_singleline(&mut name).changed() { v["name"] = json!(name); ch = true; }
    if pokey {
        let cur = v["dist"].as_str().unwrap_or("pure").to_string();
        egui::ComboBox::from_id_salt("dist").selected_text(match cur.as_str() { "pure" => "Reiner Ton", "noise" => "Rauschen", "buzz" => "Buzz (4-Bit)", "gritty" => "Gritty (5-Bit)", _ => "Bass (5+4-Bit)" }).show_ui(ui, |ui| {
            for (k, l) in [("pure", "Reiner Ton"), ("noise", "Rauschen"), ("buzz", "Buzz (4-Bit)"), ("gritty", "Gritty (5-Bit)"), ("bass", "Bass (5+4-Bit)")] { if ui.selectable_label(cur == k, l).clicked() { v["dist"] = json!(k); ch = true; } }
        });
        ui.label("Lautstärke je Bild (0–15)");
        let mut env: Vec<u64> = v["env"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0)).collect()).unwrap_or_else(|| vec![15; 16]); env.resize(16, 0);
        ui.horizontal_wrapped(|ui| { for x in env.iter_mut() { if ui.add(egui::DragValue::new(x).range(0..=15)).changed() { ch = true; } } });
        v["env"] = json!(env);
        let mut lp = v["loop"].as_u64().unwrap_or(15); if ui.add(egui::Slider::new(&mut lp, 0..=15).text("Wiederholung ab")).changed() { v["loop"] = json!(lp); ch = true; }
    } else {
        let cur = Wave::parse(v["wave"].as_str().unwrap_or("pulse"));
        egui::ComboBox::from_id_salt("wave").selected_text(cur.label()).show_ui(ui, |ui| { for w in [Wave::Triangle, Wave::Saw, Wave::Pulse, Wave::Noise] { if ui.selectable_label(cur == w, w.label()).clicked() { v["wave"] = json!(w.key()); ch = true; } } });
        let mut pw = v["pw"].as_u64().unwrap_or(2048); if ui.add(egui::Slider::new(&mut pw, 0..=4095).text("Pulsbreite")).changed() { v["pw"] = json!(pw); ch = true; }
        for (k, l) in [("a", "Attack"), ("d", "Decay"), ("s", "Sustain"), ("r", "Release")] { let mut x = v[k].as_u64().unwrap_or(0); if ui.add(egui::Slider::new(&mut x, 0..=15).text(l)).changed() { v[k] = json!(x); ch = true; } }
    }
    ch
}

fn new_instrument(pokey: bool, n: usize) -> Value {
    if pokey { json!({ "name": format!("neu{n}"), "dist": "pure", "env": [15, 14, 13, 12, 11, 10, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8], "loop": 15 }) } else { json!({ "name": format!("neu{n}"), "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 }) }
}

// ------------------------------------------------------------------------ Song-Tracker
pub struct TrackerEd { listen: crate::listen_ui::ListenPanel, chip: String, export_status: String, viz: crate::viz::VizState, hz: f32, dlg: Option<(bool, String)>, m: Model, pokey: bool, src_label: String, sid: Option<(String, u16, u16, u32)>, channels: usize, cur: Cursor, pi: usize, oi: usize, sel_ins: usize, follow: Option<(usize, usize)>, scroll: bool, rmt_file: Option<PathBuf> }

impl TrackerEd {
    pub fn open(root: &Path, rel: &str, data: &Path) -> Result<TrackerEd, String> {
        let m = Model::open(root, rel)?;
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, m.media["profile"].as_str().unwrap_or(""))?;
        let channels = p.raw["channels"].as_u64().unwrap_or(3) as usize; let pokey = matches!(p.raw["chip"].as_str(), Some("pokey" | "ted" | "vic"));
        Ok(TrackerEd { listen: Default::default(), chip: p.raw["chip"].as_str().unwrap_or("sid").to_string(), export_status: String::new(), viz: Default::default(), hz: if p.raw["chip"] == "sid" && std::env::var("VC_NTSC").is_ok() { 60.0 } else { 50.0 }, dlg: None, m, pokey, src_label: String::new(), sid: None, channels, cur: Cursor { row: 0, ch: 0, oct: 3, ins: 0 }, pi: 0, oi: 0, sel_ins: 0, follow: None, scroll: false, rmt_file: None })
    }
    fn rows(&self) -> usize { self.m.media["rows"].as_u64().unwrap_or(32) as usize }
    fn cell(&self, p: usize, r: usize, c: usize) -> String { self.m.media["patterns"][p]["rows"][r][c].as_str().unwrap_or("--- --").to_string() }
    fn npat(&self) -> usize { self.m.media["patterns"].as_array().map_or(0, |a| a.len()) }
    fn order(&self) -> Vec<usize> { self.m.media["order"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as usize).collect()).unwrap_or_default() }

    pub(crate) fn song(&self, only_pattern: bool) -> SongData {
        let patterns = self.m.media["patterns"].as_array().map(|ps| ps.iter().map(|p| p["rows"].as_array().map(|rows| rows.iter().map(|r| r.as_array().map(|cs| cs.iter().map(|c| parse_cell(c.as_str().unwrap_or(""))).collect()).unwrap_or_default()).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        SongData { pokey: self.pokey, channels: self.channels, rows: self.rows(), order: if only_pattern { vec![self.pi] } else { self.order() }, loop_at: if only_pattern { 0 } else { self.m.media["loop"].as_u64().unwrap_or(0) as usize },
            patterns, instruments: self.m.instruments(self.pokey), row_secs: self.m.media["speed"].as_f64().unwrap_or(6.0) as f32 / self.hz, frame_hz: self.hz }
    }
    fn set_rows(&mut self, n: usize) {
        self.m.push_undo();
        if let Some(ps) = self.m.media["patterns"].as_array_mut() { for p in ps { if let Some(rows) = p["rows"].as_array_mut() { while rows.len() < n { rows.push(json!(vec!["--- --"; self.channels])); } rows.truncate(n); } } }
        self.m.media["rows"] = json!(n); self.cur.row = self.cur.row.min(n - 1);
    }
}

impl MediaEd for TrackerEd {
    fn dirty(&self) -> bool { self.m.dirty() }
    fn save(&mut self) -> Result<(), String> { self.m.save() }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let ult = env.audio.source() == crate::audio::Source::Ultimate;
        let playing = env.audio.playing() && !ult;      // Mitlaufen und Positionsanzeige gehören dem Synthesizer
        // Selbsttest: VC_TRK_PLAY=1 startet den Song einmal von selbst (Bildschirmfotos der Anzeigen)
        { use std::sync::atomic::{AtomicBool, Ordering}; static AUTOPLAY: AtomicBool = AtomicBool::new(false);
          if std::env::var("VC_TRK_PLAY").is_ok() && !AUTOPLAY.swap(true, Ordering::SeqCst) { env.audio.play(self.song(false), 0, 0); } }
        let mut vopts = crate::viz::opts(ui.ctx());
        {
            let snap = env.audio.viz(1024, crate::viz::FFT_N);
            self.viz.update(snap, playing.then(|| env.audio.pos()), &vopts);
            if playing || ult || self.viz.level.iter().any(|l| *l > 0.005) || self.viz.master > 0.005 { ui.ctx().request_repaint(); }
        }
        if playing { let (o, r) = env.audio.pos(); let ord = self.order(); self.follow = ord.get(o).map(|p| (*p, r)); if let Some((p, _)) = self.follow { if self.pi != p || self.oi != o { self.pi = p; self.oi = o; } } ui.ctx().request_repaint(); self.scroll = true; } else { self.follow = None; }
        let rows = self.rows();
        let k = t();
        let chips: Vec<(String, Color32)> = match self.chip.as_str() {
            "sid" => vec![(if self.channels == 3 { format!("SID 6581 · {} Stimmen", self.channels) } else { format!("SID · {} Stimmen", self.channels) }, k.cy)],
            "pokey" => vec![(format!("POKEY · {} Stimmen", self.channels), k.am), ("RMT4".into(), k.gr)],
            c => vec![(format!("{} · {} Stimmen", c.to_uppercase(), self.channels), k.cy)],
        };
        egui::Panel::top(egui::Id::new(("tk-top", &self.m.rel))).show_inside(ui, |ui| { header(ui, &self.m.rel, self.m.dirty(), &chips); ui.horizontal_wrapped(|ui| {
            self.m.toolbar(ui, env);
            {
                let clicked = ui.button("Konvertieren von …").on_hover_text("Song aus einem anderen System ins Projekt holen und für diesen Chip umwandeln: Song (SID ↔ POKEY), GoatTracker-Datei (.sng) oder SID-Datei (.sid, Aufnahme)").clicked();
                if let Some(f) = pick_or_dialog(clicked, "convert", || rfd::FileDialog::new().add_filter("SID-Datei (.sid), GoatTracker (.sng) oder Song (.vcsong)", &["sng", "sid", "vcsong"]).set_title("Song aus anderem System wählen").pick_file()) {
                    let fname = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    let name = vc_media::naming::stem(&fname).trim_end_matches(".sng").trim_end_matches(".sid").replace(' ', "_");
                    self.sid = None;
                    if fname.to_lowercase().ends_with(".sid") {
                        match std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| vc_media::sid::info(&b).map(|x| x.0)) {
                            Ok(i) => { self.sid = Some((format!("{} – {} ({} Subtunes)", i.title, i.author, i.songs), i.songs, i.start_song, 30)); }
                            Err(e) => { env.reqs.error(format!("{fname}: {e}")); return; }
                        }
                    }
                    self.src_label = if self.sid.is_some() { "SID-Datei (Aufnahme der Spielroutine)".into() } else if fname.to_lowercase().ends_with(".sng") { "GoatTracker (SID)".into() } else {
                        match std::fs::read_to_string(&f).ok().and_then(|t| vc_media::parse_media(&t).ok()).map(|a| a.profile) { Some(p) if p.contains("pokey") => "POKEY (Atari)".into(), Some(p) if p.contains("sid") => "SID (C64)".into(), _ => "unbekannt".into() }
                    };
                    self.dlg = Some((false, name)); self.rmt_file = Some(f);
                }
            }
            { let clicked = ui.button("RMT importieren …").on_hover_text("RMT-Modul (Raster Music Tracker) als POKEY-Song laden").clicked(); if let Some(f) = pick_or_dialog(clicked, "rmt", || rfd::FileDialog::new().add_filter("RMT-Modul", &["rmt"]).pick_file()) { self.dlg = Some((true, f.file_stem().map(|n| n.to_string_lossy().replace(' ', "_")).unwrap_or_default())); self.rmt_file = Some(f); } }
            ui.separator();
            if ui.add_enabled(!ult, play_button(playing, "▶ Song")).on_disabled_hover_text("Quelle ist die Ultimate: Zum Abspielen „Auf Hardware hören“ benutzen oder die Quelle auf Synthesizer stellen").clicked() { if playing { env.audio.stop(); } else { env.audio.play(self.song(false), self.oi, 0); } }
            if ui.add_enabled(!playing && !ult, egui::Button::new("▶ Pattern")).clicked() { env.audio.play(self.song(true), 0, 0); }
            ui.separator();
            let mut sp = self.m.media["speed"].as_u64().unwrap_or(6); ui.label("Takt"); if ui.add(egui::DragValue::new(&mut sp).range(1..=31)).changed() { self.m.push_undo(); self.m.media["speed"] = json!(sp); }
            let mut rw = rows; ui.label("Zeilen"); if ui.add(egui::DragValue::new(&mut rw).range(1..=128)).changed() { self.set_rows(rw); }
            ui.label(format!("Oktave {}", self.cur.oct));
            ui.label("Bildrate"); egui::ComboBox::from_id_salt("hz").selected_text(format!("{:.0} Hz", self.hz)).show_ui(ui, |ui| { ui.selectable_value(&mut self.hz, 50.0, "50 Hz (PAL)"); ui.selectable_value(&mut self.hz, 60.0, "60 Hz (NTSC)"); });
            if let Some(e) = env.audio.error() { ui.colored_label(crate::highlight::PAL_ERR, format!("Audio: {e}")); }
            if !self.m.status.is_empty() { ui.label(RichText::new(&self.m.status).weak()); }
            if vopts.vu { ui.separator(); crate::viz::master_meter(ui, &self.viz, &vopts); }
        }); });
        {
            let (rel, name) = (self.m.rel.clone(), self.m.media["name"].as_str().unwrap_or("lied").to_string());
            let (profile, chip, channels, hz) = (self.m.media["profile"].as_str().unwrap_or("").to_string(), self.chip.clone(), self.channels, self.hz);
            let mut ln = std::mem::take(&mut self.listen);
            let json = || vc_media::project::format_media(&self.m.media); let song = || self.song(false);
            let info = crate::listen_ui::SongInfo { root: env.root, data: env.data, name: &name, profile: &profile, chip: &chip, channels, hz, json: &json, song: &song };
            egui::Panel::top(egui::Id::new(("tk-listen", &rel))).show_inside(ui, |ui| { ln.show(ui, env.audio, &info); });
            self.listen = ln;
        }
        if let Some((is_rmt, mut name)) = self.dlg.take() {
            let mut keep = true; let mut go = false;
            egui::Window::new(if is_rmt { "RMT-Modul importieren" } else { "Song aus anderem System übernehmen" }).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| { ui.label("Neuer Name"); ui.text_edit_singleline(&mut name); });
                if !is_rmt {
                    let (src, dst) = (self.src_label.clone(), if self.pokey { "POKEY (Atari)" } else { "SID (C64)" });
                    ui.label(format!("Quelle: {}", self.rmt_file.as_ref().map(|f| f.display().to_string()).unwrap_or_default()));
                    ui.label(format!("{src}  →  {dst}"));
                    if let Some((txt, songs, song, secs)) = &mut self.sid {
                        ui.label(RichText::new(txt.as_str()).strong());
                        ui.horizontal(|ui| { ui.label("Subtune"); ui.add(egui::DragValue::new(song).range(1..=*songs)); ui.label("Länge (Sekunden)"); ui.add(egui::DragValue::new(secs).range(5..=600)); });
                        ui.label(RichText::new("Die Spielroutine läuft in einem 6502-Kern; aufgezeichnet werden Gate, Tonhöhe, Wellenform, ADSR und Pulsbreite. Vibrato, Filter und Effekte entfallen.").weak().small());
                    }
                }
                ui.label(RichText::new(if is_rmt { "RMT4, Mono, mit Ladeadresse. Effekte, Tabellen und Lautstärken werden nicht übernommen." } else { "Der Song wird als neue Datei im Projekt angelegt. Noten und Struktur bleiben; Instrumente werden angenähert." }).weak().small());
                ui.horizontal(|ui| { if ui.button("OK").clicked() { go = true; keep = false; } if ui.button("Abbrechen").clicked() { keep = false; } });
            });
            if go {
                let dir = self.m.rel.rsplit_once('/').map(|x| x.0.to_string()).unwrap_or_else(|| ".".into());
                let dir = if dir == "." { "music".to_string() } else { dir };
                let res = if is_rmt {
                    (|| -> Result<(String, Vec<String>), String> {
                        let f = self.rmt_file.take().ok_or("keine Datei gewählt")?; let bytes = std::fs::read(&f).map_err(|e| format!("{}: {e}", f.display()))?;
                        let profiles = vc_media::load_profiles(env.data)?; let p = vc_media::profile(&profiles, "atari.pokey.song")?;
                        let (v, w) = vc_media::song::rmt_import(&bytes, p, &name)?;
                        let rel = format!("{dir}/{name}.vcsong"); let out = env.root.join(&rel);
                        if out.exists() { return Err(format!("{rel} existiert bereits")); }
                        std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?; std::fs::write(&out, vc_media::project::format_media(&v)).map_err(|e| e.to_string())?; Ok((rel, w))
                    })()
                } else {
                    (|| -> Result<(String, Vec<String>), String> {
                        let f = self.rmt_file.take().ok_or("keine Datei gewählt")?;
                        let to = self.m.media["profile"].as_str().ok_or("Profil fehlt")?.to_string();
                        let opts = self.sid.as_ref().map(|(_, _, song, secs)| vc_media::sid::SidOpts { song: *song, seconds: *secs }).unwrap_or_default();
                        let r = vc_media::project::import_song_with(env.root, env.data, &f, &to, &name, &dir, &opts)?;
                        Ok((r["path"].as_str().unwrap_or("").to_string(), r["warnings"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()))
                    })()
                };
                match res { Ok((rel, warns)) => { for w in warns { env.reqs.warn(format!("Hinweis: {w}")); } env.reqs.info(format!("Angelegt: {rel}")); env.reqs.open(&rel); } Err(e) => env.reqs.error(e) }
            }
            if keep { self.dlg = Some((is_rmt, name)); }
        }
        { let rel = self.m.rel.clone(); crate::ui::studio::left(ui, &rel, 190.0, |ui| { egui::ScrollArea::vertical().show(ui, |ui| {
            widgets::section(ui, "Reihenfolge", "↺ = Wiederholung ab", |_| {});
            let order = self.order();
            let loop_at = self.m.media["loop"].as_u64().unwrap_or(0) as usize;
            for (i, p) in order.iter().enumerate() {
                let playing_here = self.follow.is_some() && self.oi == i;
                let text = RichText::new(format!("{}{i:02}  Pattern {p:02}", if i == loop_at { "↺ " } else { "   " })).monospace().color(if i == loop_at { t().am } else if playing_here { t().cy } else { t().fg });
                if ui.selectable_label(self.oi == i, text).on_hover_text(if i == loop_at { "Hier beginnt die Wiederholung" } else { "" }).clicked() { self.oi = i; self.pi = *p; }
            }
            ui.horizontal(|ui| {
                if ui.button("+").on_hover_text("Aktuelles Pattern anhängen").clicked() { self.m.push_undo(); if let Some(o) = self.m.media["order"].as_array_mut() { o.push(json!(self.pi)); } }
                if ui.add_enabled(order.len() > 1, egui::Button::new("−")).clicked() { self.m.push_undo(); let oi = self.oi.min(order.len() - 1); if let Some(o) = self.m.media["order"].as_array_mut() { o.remove(oi); } self.oi = oi.saturating_sub(1); }
                if ui.button("Setzen").on_hover_text("Eintrag auf das aktuelle Pattern setzen").clicked() && self.oi < order.len() { self.m.push_undo(); self.m.media["order"][self.oi] = json!(self.pi); }
            });
            let mut lp = self.m.media["loop"].as_u64().unwrap_or(0); ui.horizontal(|ui| { ui.label("Wiederholung ab"); if ui.add(egui::DragValue::new(&mut lp).range(0..=order.len().saturating_sub(1))).changed() { self.m.push_undo(); self.m.media["loop"] = json!(lp); } });
            ui.separator(); widgets::section(ui, "Patterns", "Bausteine der Reihenfolge", |_| {});
            for p in 0..self.npat() { if ui.selectable_label(self.pi == p, format!("Pattern {p:02}")).clicked() { self.pi = p; } }
            ui.horizontal(|ui| {
                if ui.button("+ Neu").clicked() { self.m.push_undo(); let rows = self.rows(); let ch = self.channels; if let Some(ps) = self.m.media["patterns"].as_array_mut() { ps.push(json!({ "rows": (0..rows).map(|_| vec!["--- --"; ch]).collect::<Vec<_>>() })); } self.pi = self.npat() - 1; }
                if ui.button("Dupl.").clicked() { self.m.push_undo(); let c = self.m.media["patterns"][self.pi].clone(); if let Some(ps) = self.m.media["patterns"].as_array_mut() { ps.push(c); } self.pi = self.npat() - 1; }
            });
            ui.separator(); crate::viz::side_views(ui, &self.viz, &vopts); crate::viz::settings_ui(ui, &mut vopts);
        }); }); }
        {
            let formats: Vec<(&str, &str)> = if self.pokey && self.chip == "pokey" { vec![("asm", "RMT-Modul als Assembler mit verschiebbaren Zeigern"), ("rmt", "RMT-Modul als Datei (Ladeadresse $4000)")] }
                else { vec![("asm", "Daten und Konstanten als Assembler"), ("inc", "Include-Datei mit Größen"), ("bin", "Rohdaten")] };
            let rel = self.m.rel.clone();
            egui::Panel::bottom(egui::Id::new(("tk-export", &self.m.rel))).show_inside(ui, |ui| { export_strip(ui, env, &rel, &formats, &mut self.export_status); });
        }
        if vopts.keys {
            egui::Panel::bottom(egui::Id::new(("tk-keys", &self.m.rel))).show_inside(ui, |ui| {
                let h = 54.0; let w = ui.available_width().min(760.0);
                if let Some(note) = crate::viz::keyboard(ui, vec2(w, h), &self.viz.snap.notes) { if let Some(ins) = self.m.instruments(self.pokey).get(self.cur.ins) { env.audio.audition(ins, note); } }
            });
        }
        { let rel = self.m.rel.clone(); crate::ui::studio::right(ui, &rel, 260.0, |ui| { egui::ScrollArea::vertical().show(ui, |ui| {
            widgets::section(ui, "Instrumente", if self.pokey { "Verzerrung und Lautstärke je Bild" } else { "Wellenform, Pulsbreite und ADSR" }, |_| {});
            let n = self.m.media["instruments"].as_array().map_or(0, |a| a.len());
            for i in 0..n { let name = self.m.media["instruments"][i]["name"].as_str().unwrap_or(""); if ui.selectable_label(self.sel_ins == i, format!("{i:02X}  {name}")).clicked() { self.sel_ins = i; self.cur.ins = i; } }
            ui.horizontal(|ui| {
                if n < 32 && ui.button("+").clicked() { self.m.push_undo(); let v = new_instrument(self.pokey, n); if let Some(a) = self.m.media["instruments"].as_array_mut() { a.push(v); } self.sel_ins = n; }
                if ui.button("Dupl.").clicked() && n > 0 && n < 32 { self.m.push_undo(); let v = self.m.media["instruments"][self.sel_ins].clone(); if let Some(a) = self.m.media["instruments"].as_array_mut() { a.push(v); } self.sel_ins = n; }
            });
            if n > 0 {
                self.sel_ins = self.sel_ins.min(n - 1); ui.separator();
                let before = self.m.media["instruments"][self.sel_ins].clone(); let mut v = before.clone();
                if widgets::card(ui, false, |ui| instrument_editor(ui, &mut v, self.pokey)) { self.m.undo.push({ let mut a = self.m.media.clone(); a["instruments"][self.sel_ins] = before; a.to_string() }); self.m.redo.clear(); self.m.media["instruments"][self.sel_ins] = v; }
                let ins = Instr::from_json(&self.m.media["instruments"][self.sel_ins], self.pokey); envelope_plot(ui, &ins);
                if ui.button("Hörprobe (C-4)").clicked() { env.audio.audition(&ins, 49); }
            }
        }); }); }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let (id, chn, pi) = (egui::Id::new(("tk-grid", &self.m.rel)), self.channels, self.pi);
            let cells: Vec<Vec<String>> = (0..rows).map(|r| (0..chn).map(|c| self.cell(pi, r, c)).collect()).collect();
            let play_row = self.follow.filter(|(p, _)| *p == pi).map(|(_, r)| r);
            let play_row_shown = play_row.filter(|_| vopts.follow);
            let head_h = crate::viz::head_height(&vopts);
            let flash = |r: usize, c: usize| -> f32 { if vopts.flash && play_row == Some(r) && !cells[r][c].starts_with("---") && !cells[r][c].starts_with("===") { self.viz.flash() } else { 0.0 } };
            let draw_head = |ui: &mut egui::Ui, c: usize, rect: Rect| crate::viz::channel_head(ui, &self.viz, &vopts, env.audio, c, rect, &format!("Stimme {}", c + 1));
            let (clicked, events) = grid(ui, GridView { id, rows, cols: chn, head: &|c| format!("Stimme {}", c + 1), cell: &|r, c| cells[r][c].clone(), play_row: play_row_shown, cur: (self.cur.row, self.cur.ch), scroll_to: self.scroll,
                cw: if head_h > 0.0 { 124.0 } else { 86.0 }, head_h, head_draw: Some(&draw_head), flash: Some(&flash) });
            self.scroll = false;
            if let Some((r, c)) = clicked { self.cur.row = r; self.cur.ch = c; }
            let n_ins = self.m.media["instruments"].as_array().map_or(0, |a| a.len());
            let insts = self.m.instruments(self.pokey);
            for act in handle_keys(&mut self.cur, rows, chn, n_ins, &events) {
                match act {
                    Act::Set(r, c, t, aud) => { self.m.push_undo(); self.m.media["patterns"][pi]["rows"][r][c] = json!(t); if let Some((note, i)) = aud { if let Some(ins) = insts.get(i) { env.audio.audition(ins, note); } } }
                    Act::Clear(r, c) => { self.m.push_undo(); self.m.media["patterns"][pi]["rows"][r][c] = json!("--- --"); }
                    Act::Toggle => { if playing { env.audio.stop(); } else { env.audio.play(self.song(false), self.oi, 0); } }
                    Act::Undo => self.m.do_undo(), Act::Redo => self.m.do_redo(),
                    Act::Save => { if let Err(e) = self.m.save() { self.m.status = e; } }
                    Act::Moved => self.scroll = true,
                }
            }
            self.sel_ins = self.cur.ins.min(n_ins.saturating_sub(1));
        });
        self.m.export_dialog(ui, env, &format!("Ziel: {}", self.chip.to_uppercase()));
        crate::viz::set_opts(ui.ctx(), vopts);
    }
}

// ------------------------------------------------------------------------ PLAY-Tracker
pub struct PlayEd { export_status: String, viz: crate::viz::VizState, m: Model, voices: usize, cur: Cursor, sel_ins: usize, follow: Option<usize>, scroll: bool }

impl PlayEd {
    pub fn open(root: &Path, rel: &str) -> Result<PlayEd, String> {
        let m = Model::open(root, rel)?; let voices = m.media["voices"].as_u64().unwrap_or(6) as usize;
        Ok(PlayEd { export_status: String::new(), viz: Default::default(), m, voices, cur: Cursor { row: 0, ch: 0, oct: 3, ins: 0 }, sel_ins: 0, follow: None, scroll: false })
    }
    fn steps(&self) -> usize { self.m.media["steps"].as_array().map_or(0, |a| a.len()) }
    fn song(&self) -> SongData {
        let rows: Vec<Vec<(u8, u8)>> = self.m.media["steps"].as_array().map(|ss| ss.iter().map(|r| r.as_array().map(|cs| cs.iter().map(|c| parse_cell(c.as_str().unwrap_or(""))).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        SongData { pokey: false, channels: self.voices, rows: rows.len().max(1), order: vec![0], loop_at: 0, patterns: vec![rows], instruments: self.m.instruments(false), row_secs: 1.0 / self.m.media["tempo"].as_f64().unwrap_or(8.0).max(1.0) as f32, frame_hz: 50.0 }
    }
}

impl MediaEd for PlayEd {
    fn dirty(&self) -> bool { self.m.dirty() }
    fn save(&mut self) -> Result<(), String> { self.m.save() }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let playing = env.audio.playing();
        let mut vopts = crate::viz::opts(ui.ctx());
        {
            let snap = env.audio.viz(1024, crate::viz::FFT_N);
            self.viz.update(snap, playing.then(|| env.audio.pos()), &vopts);
            if playing || self.viz.level.iter().any(|l| *l > 0.005) || self.viz.master > 0.005 { ui.ctx().request_repaint(); }
        }
        if playing { self.follow = Some(env.audio.pos().1); ui.ctx().request_repaint(); self.scroll = true; } else { self.follow = None; }
        let rows = self.steps().max(1);
        let chips = vec![(format!("MEGA65 PLAY · {} Stimmen", self.voices), t().pu)];
        egui::Panel::top(egui::Id::new(("pl-top", &self.m.rel))).show_inside(ui, |ui| { header(ui, &self.m.rel, self.m.dirty(), &chips); ui.horizontal_wrapped(|ui| {
            self.m.toolbar(ui, env); ui.separator();
            if ui.add(play_button(playing, "▶ Abspielen")).clicked() { if playing { env.audio.stop(); } else { env.audio.play(self.song(), 0, 0); } }
            let mut t = self.m.media["tempo"].as_u64().unwrap_or(8); ui.label("Tempo"); if ui.add(egui::DragValue::new(&mut t).range(1..=255)).changed() { self.m.push_undo(); self.m.media["tempo"] = json!(t); }
            let mut v = self.m.media["volume"].as_u64().unwrap_or(8); ui.label("Lautstärke"); if ui.add(egui::DragValue::new(&mut v).range(0..=15)).changed() { self.m.push_undo(); self.m.media["volume"] = json!(v); }
            ui.label(format!("Oktave {}", self.cur.oct));
            if let Some(e) = env.audio.error() { ui.colored_label(crate::highlight::PAL_ERR, format!("Audio: {e}")); }
            if !self.m.status.is_empty() { ui.label(RichText::new(&self.m.status).weak()); }
            if vopts.vu { ui.separator(); crate::viz::master_meter(ui, &self.viz, &vopts); }
        }); });
        {
            let rel = self.m.rel.clone();
            egui::Panel::bottom(egui::Id::new(("pl-export", &self.m.rel))).show_inside(ui, |ui| { export_strip(ui, env, &rel, &[("bas", "PLAY-Befehle als BASIC-Programm")], &mut self.export_status); });
        }
        if vopts.keys {
            egui::Panel::bottom(egui::Id::new(("pl-keys", &self.m.rel))).show_inside(ui, |ui| {
                let w = ui.available_width().min(760.0);
                if let Some(note) = crate::viz::keyboard(ui, vec2(w, 54.0), &self.viz.snap.notes) { if let Some(ins) = self.m.instruments(false).get(self.cur.ins) { env.audio.audition(ins, note); } }
            });
        }
        { let rel = self.m.rel.clone(); crate::ui::studio::right(ui, &rel, 260.0, |ui| { egui::ScrollArea::vertical().show(ui, |ui| {
            widgets::section(ui, "Hüllkurven", "ENVELOPE 0–9 im PLAY-Befehl", |_| {});
            let n = self.m.media["instruments"].as_array().map_or(0, |a| a.len());
            for i in 0..n { let name = self.m.media["instruments"][i]["name"].as_str().unwrap_or(""); if ui.selectable_label(self.sel_ins == i, format!("{i}  {name}")).clicked() { self.sel_ins = i; self.cur.ins = i; } }
            if n < 10 && ui.button("+ Hüllkurve").clicked() { self.m.push_undo(); let v = new_instrument(false, n); if let Some(a) = self.m.media["instruments"].as_array_mut() { a.push(v); } self.sel_ins = n; }
            if n > 0 {
                self.sel_ins = self.sel_ins.min(n - 1); ui.separator();
                let before = self.m.media["instruments"][self.sel_ins].clone(); let mut v = before.clone();
                if widgets::card(ui, false, |ui| instrument_editor(ui, &mut v, false)) { self.m.undo.push({ let mut a = self.m.media.clone(); a["instruments"][self.sel_ins] = before; a.to_string() }); self.m.redo.clear(); self.m.media["instruments"][self.sel_ins] = v; }
                let ins = Instr::from_json(&self.m.media["instruments"][self.sel_ins], false); envelope_plot(ui, &ins);
                if ui.button("Hörprobe (C-4)").clicked() { env.audio.audition(&ins, 49); }
            }
            ui.separator(); crate::viz::side_views(ui, &self.viz, &vopts); crate::viz::settings_ui(ui, &mut vopts);
        }); }); }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let (id, vo) = (egui::Id::new(("pl-grid", &self.m.rel)), self.voices);
            let cells: Vec<Vec<String>> = (0..rows).map(|r| (0..vo).map(|c| self.m.media["steps"][r][c].as_str().unwrap_or("--- --").to_string()).collect()).collect();
            let head_h = crate::viz::head_height(&vopts);
            let flash = |r: usize, c: usize| -> f32 { if vopts.flash && self.follow == Some(r) && !cells[r][c].starts_with("---") && !cells[r][c].starts_with("===") { self.viz.flash() } else { 0.0 } };
            let draw_head = |ui: &mut egui::Ui, c: usize, rect: Rect| crate::viz::channel_head(ui, &self.viz, &vopts, env.audio, c, rect, &format!("Stimme {}", c + 1));
            let (clicked, events) = grid(ui, GridView { id, rows, cols: vo, head: &|c| format!("Stimme {}", c + 1), cell: &|r, c| cells[r][c].clone(), play_row: self.follow.filter(|_| vopts.follow), cur: (self.cur.row, self.cur.ch), scroll_to: self.scroll,
                cw: if head_h > 0.0 { 124.0 } else { 86.0 }, head_h, head_draw: Some(&draw_head), flash: Some(&flash) });
            self.scroll = false;
            if let Some((r, c)) = clicked { self.cur.row = r; self.cur.ch = c; }
            let n_ins = self.m.media["instruments"].as_array().map_or(0, |a| a.len()); let insts = self.m.instruments(false);
            for act in handle_keys(&mut self.cur, rows, vo, n_ins, &events) {
                match act {
                    Act::Set(r, c, t, aud) => { self.m.push_undo(); self.m.media["steps"][r][c] = json!(t); if let Some((note, i)) = aud { if let Some(ins) = insts.get(i) { env.audio.audition(ins, note); } } }
                    Act::Clear(r, c) => { self.m.push_undo(); self.m.media["steps"][r][c] = json!("--- --"); }
                    Act::Toggle => { if playing { env.audio.stop(); } else { env.audio.play(self.song(), 0, 0); } }
                    Act::Undo => self.m.do_undo(), Act::Redo => self.m.do_redo(),
                    Act::Save => { if let Err(e) = self.m.save() { self.m.status = e; } }
                    Act::Moved => self.scroll = true,
                }
            }
            self.sel_ins = self.cur.ins.min(n_ins.saturating_sub(1));
        });
        self.m.export_dialog(ui, env, "Ziel: MEGA65 PLAY");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../") }

    #[test]
    fn song_and_play_load_edit_save_and_render() {
        let data = root().join("data"); let dir = std::env::temp_dir().join(format!("vc-tk-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
        for (src, dst) in [("examples/c64/music_demo/music/tune.vcsong", "tune.asset.json"), ("examples/mega65/play_demo/music/tune_play.vcplay", "play.asset.json"), ("examples/atari/music_demo/music/tune_pokey.vcsong", "pk.asset.json")] { std::fs::copy(root().join(src), dir.join(dst)).unwrap(); }
        let mut t = TrackerEd::open(&dir, "tune.asset.json", &data).unwrap();
        assert_eq!((t.channels, t.rows(), t.npat()), (3, 32, 2));
        let song = t.song(false); assert_eq!(song.order, vec![0, 1, 0, 1]); assert_eq!(song.instruments.len(), t.m.media["instruments"].as_array().unwrap().len());
        let mut e = synth::Engine::default(); e.play(song, 0, 0); let mut buf = vec![0.0; 44100]; e.render(&mut buf, 44100.0);
        assert!(buf.iter().any(|v| v.abs() > 0.02), "Song klingt"); assert!(e.pos.1 > 0);
        t.m.push_undo(); t.m.media["patterns"][0]["rows"][1][0] = json!("E-4 00"); assert!(t.dirty()); t.save().unwrap();
        assert_eq!(TrackerEd::open(&dir, "tune.asset.json", &data).unwrap().cell(0, 1, 0), "E-4 00");
        t.set_rows(16); assert_eq!(t.rows(), 16); assert_eq!(t.m.media["patterns"][0]["rows"].as_array().unwrap().len(), 16);
        let pk = TrackerEd::open(&dir, "pk.asset.json", &data).unwrap(); assert!(pk.pokey); assert!(matches!(pk.song(true).instruments[0], Instr::Pokey { .. }));
        let p = PlayEd::open(&dir, "play.asset.json").unwrap(); assert_eq!((p.voices, p.steps()), (6, 64));
        let s = p.song(); assert_eq!(s.patterns[0].len(), 64); assert!((s.row_secs - 0.125).abs() < 1e-6);
        let mut cur = Cursor { row: 0, ch: 0, oct: 3, ins: 1 };
        let acts = handle_keys(&mut cur, 32, 3, 4, &[egui::Event::Key { key: egui::Key::C, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]);
        assert!(matches!(&acts[0], Act::Set(0, 0, t, Some((41, 1))) if t == "E-3 01"), "C = Halbton 4 in Oktave 3"); assert_eq!(cur.row, 1);
        // Konvertieren SID -> POKEY erzeugt einen eigenen Song, den der Tracker wieder lädt
        let r = vc_media::project::convert_song(&dir, &data, "tune.asset.json", "atari.pokey.song", "tune_p", ".").unwrap();
        let conv = TrackerEd::open(&dir, r["path"].as_str().unwrap(), &data).unwrap(); assert!(conv.pokey); assert_eq!(conv.channels, 4);
        // Import aus einem anderen System: Quelle liegt außerhalb des Projekts (anderer Ordner), Ziel ist das Profil des offenen Trackers
        let other = std::env::temp_dir().join(format!("vc_tk_other_{}", std::process::id())); std::fs::create_dir_all(&other).unwrap();
        std::fs::copy(dir.join(r["path"].as_str().unwrap()), other.join("fremd.asset.json")).unwrap();      // POKEY-Song des "anderen" Systems
        let imp = vc_media::project::import_song(&dir, &data, &other.join("fremd.asset.json"), "c64.sid.song", "aus_atari", "assets").unwrap();
        assert_eq!(imp["from"], "atari.pokey.song"); assert_eq!(imp["path"], "music/aus_atari.vcsong");
        let back = TrackerEd::open(&dir, "music/aus_atari.vcsong", &data).unwrap(); assert!(!back.pokey); assert_eq!(back.channels, 3);
        assert!(vc_media::project::import_song(&dir, &data, &other.join("fremd.asset.json"), "c64.sid.song", "aus_atari", "assets").is_err(), "vorhandenes Media wird nicht überschrieben");
        std::fs::remove_dir_all(&other).ok();
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tracker_zeichnet_klein_und_oeffnet_export() {
        let data = root().join("data"); let dir = std::env::temp_dir().join(format!("vc-tk-small-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
        for (src, dst) in [("examples/c64/music_demo/music/tune.vcsong", "tune.asset.json"), ("examples/mega65/play_demo/music/tune_play.vcplay", "play.asset.json"), ("examples/atari/music_demo/music/tune_pokey.vcsong", "pk.asset.json")] { std::fs::copy(root().join(src), dir.join(dst)).unwrap(); }
        let pal = crate::bitmap::Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        let run = |ui_fn: &mut dyn FnMut(&mut egui::Ui), w: f32, h: f32| { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| ui_fn(ui)); o.textures_delta.clear(); } };
        for (w, h) in [(330.0, 500.0), (640.0, 480.0), (1100.0, 760.0)] {
            let mut t = TrackerEd::open(&dir, "tune.asset.json", &data).unwrap();
            let mut pk = TrackerEd::open(&dir, "pk.asset.json", &data).unwrap();
            let mut p = PlayEd::open(&dir, "play.asset.json").unwrap();
            for m in [&mut t.m, &mut pk.m, &mut p.m] { crate::exportdlg::open(&mut m.exp, &m.rel.clone(), "song", "tune"); assert!(m.exp.is_some()); }
            run(&mut |ui| MediaEd::ui(&mut t, ui, &env), w, h);
            run(&mut |ui| MediaEd::ui(&mut pk, ui, &env), w, h);
            run(&mut |ui| MediaEd::ui(&mut p, ui, &env), w, h);
            assert!(t.m.exp.is_some() && pk.m.exp.is_some() && p.m.exp.is_some(), "Export-Dialog bleibt offen ({w}x{h})");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn kopf_huellkurve_und_knopf_zeichnen() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let sid = Instr::from_json(&json!({ "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 }), false);
        let pk = Instr::from_json(&json!({ "dist": "buzz", "env": [15, 14, 13, 12, 11, 10, 9, 8, 8, 8, 8, 8, 8, 8, 8, 8], "loop": 8 }), true);
        assert!(matches!(&pk, Instr::Pokey { loop_at: 8, .. }));
        for _ in 0..2 {
            let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
                header(ui, "music/tune.vcsong", true, &[("POKEY · 4 Stimmen".into(), t().am), ("RMT4".into(), t().gr)]);
                envelope_plot(ui, &sid); envelope_plot(ui, &pk);
                ui.add(play_button(false, "▶ Song")); ui.add(play_button(true, "▶ Song"));
            });
            o.textures_delta.clear();
        }
    }
}

/// Selbsttest (Screenshots): `VC_TRK_DLG=convert|rmt` löst „Konvertieren von …“ bzw. „RMT importieren …“ einmal selbst aus, `VC_PICK` ersetzt die Dateiwahl
fn pick_or_dialog(clicked: bool, which: &str, dialog: impl FnOnce() -> Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if std::env::var("VC_SHOT").is_ok() && std::env::var("VC_TRK_DLG").as_deref() == Ok(which) {
        if let Ok(f) = std::env::var("VC_PICK") { if !DONE.swap(true, std::sync::atomic::Ordering::SeqCst) { return Some(std::path::PathBuf::from(f)); } }
    }
    if clicked { dialog() } else { None }
}
