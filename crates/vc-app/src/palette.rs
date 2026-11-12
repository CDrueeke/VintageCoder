// SPDX-License-Identifier: GPL-3.0-or-later
//! Paletten-Editor (MEGA65: 256 Farben, 8 Bit je Kanal).
use crate::bitmap::{export_media, MediaEd, Env};
use crate::ui::studio;
use eframe::egui::{self, vec2, Color32, RichText, Sense, Stroke};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub struct PaletteEd { rel: String, abs: PathBuf, media: Value, sel: usize, undo: Vec<String>, redo: Vec<String>, saved: String, status: String, hex: String }

fn hex_to_rgb(s: &str) -> Option<[u8; 3]> {
    let t = s.trim().trim_start_matches('#');
    (t.len() == 6).then(|| u32::from_str_radix(t, 16).ok()).flatten().map(|v| [(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

impl PaletteEd {
    pub fn open(root: &Path, rel: &str) -> Result<PaletteEd, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| format!("{rel}: {e}"))?).map_err(|e| format!("{rel}: {e}"))?;
        if media["entries"].as_array().map_or(true, |a| a.is_empty()) { return Err(format!("{rel}: keine Palettenwerte")); }
        Ok(PaletteEd { rel: rel.into(), abs, saved: media.to_string(), media, sel: 0, undo: vec![], redo: vec![], status: String::new(), hex: String::new() })
    }
    fn entry(&self, i: usize) -> String { self.media["entries"][i].as_str().unwrap_or("#000000").to_string() }
    fn push_undo(&mut self) { self.undo.push(self.media.to_string()); self.redo.clear(); }
    fn set(&mut self, i: usize, hex: String) { self.push_undo(); self.media["entries"][i] = json!(hex); }
}

impl MediaEd for PaletteEd {
    fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    fn save(&mut self) -> Result<(), String> {
        std::fs::write(&self.abs, vc_media::project::format_media(&self.media)).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = "gespeichert".into(); Ok(())
    }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let (s, z) = ui.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::S), i.modifiers.command && i.key_pressed(egui::Key::Z)));
        if s { let _ = self.save(); }
        if z { if let Some(u) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&u) { self.media = v; } } }
        ui.horizontal_wrapped(|ui| {
            if crate::shell::action_button(ui, "Speichern", crate::shell::Kind::Primary, true).clicked() { if let Err(e) = self.save() { self.status = e; } }
            if ui.button("Exportieren").clicked() { match self.save() { Ok(()) => export_media(env, &self.rel, "palette"), Err(e) => self.status = e } }
                if ui.button("In Code einfügen").on_hover_text("Exportiert und fügt .include in den zuletzt benutzten Quelltext ein").clicked() { match self.save() { Ok(()) => { export_media(env, &self.rel, "palette"); env.reqs.insert_asset(&self.rel); } Err(e) => self.status = e } }
            if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo")).clicked() { if let Some(u) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&u) { self.media = v; } } }
            if ui.button("Standardpalette").clicked() { self.push_undo(); self.media["entries"] = json!(vc_media::default_palette()); }
            if !self.status.is_empty() { ui.label(RichText::new(&self.status).weak()); }
        });
        ui.separator();
        // Schmal (halber Editorbereich): Eigenschaften unter dem Raster; die Felder wachsen mit dem Platz
        let (avail_w, narrow) = (ui.available_width(), ui.available_width() < 560.0);
        let sw = studio::fit_zoom(vec2(avail_w - if narrow { 8.0 } else { 300.0 }, 1e6), vec2(16.0, 16.0), vec2(0.0, 0.0), 16.0, 40.0) - 2.0;
        let sw = sw.max(16.0);
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            if narrow { ui.vertical(|ui| { self.swatch_grid(ui, sw); ui.add_space(8.0); self.entry_card(ui); }); }
            else { ui.horizontal_top(|ui| { self.swatch_grid(ui, sw); ui.add_space(14.0); self.entry_card(ui); }); }
        });
    }
}

impl PaletteEd {
    fn swatch_grid(&mut self, ui: &mut egui::Ui, sw: f32) {
        ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                let n = self.media["entries"].as_array().map_or(0, |a| a.len());
                for row in 0..(n + 15) / 16 { ui.horizontal(|ui| { for i in row * 16..((row + 1) * 16).min(n) {
                    let (r, resp) = ui.allocate_exact_size(vec2(sw, sw), Sense::click());
                    let c = hex_to_rgb(&self.entry(i)).map_or(Color32::GRAY, |c| Color32::from_rgb(c[0], c[1], c[2]));
                    ui.painter().rect_filled(r, 2.0, c);
                    if i == self.sel { ui.painter().rect_stroke(r, 2.0, Stroke::new(2.0, crate::ui::tokens::t().cy), egui::StrokeKind::Outside); }
                    if resp.clicked() { self.sel = i; self.hex = self.entry(i); }
                    resp.on_hover_text(format!("{i} ($ {:02X}) {}", i, self.entry(i)));
                } }); }
        });
    }

    fn entry_card(&mut self, ui: &mut egui::Ui) {
        crate::ui::widgets::card(ui, false, |ui| {
                ui.set_max_width(260.0);
                ui.label(RichText::new(format!("Eintrag {} ($ {:02X})", self.sel, self.sel)).font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
                let mut rgb = hex_to_rgb(&self.entry(self.sel)).unwrap_or([0; 3]);
                if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() { self.set(self.sel, format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])); self.hex = self.entry(self.sel); }
                if self.hex.is_empty() { self.hex = self.entry(self.sel); }
                let r = ui.text_edit_singleline(&mut self.hex);
                if r.lost_focus() { if let Some(c) = hex_to_rgb(&self.hex) { self.set(self.sel, format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])); } self.hex = self.entry(self.sel); }
                ui.add_space(6.0);
                ui.label(RichText::new("Die MEGA65-Palette hat 8 Bit je Kanal. Der Export kümmert sich um die vertauschten Halbbytes der Register $D100/$D200/$D300.").weak().small());
        });
    }
}
