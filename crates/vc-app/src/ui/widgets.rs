// SPDX-License-Identifier: GPL-3.0-or-later
//! Wiederverwendbare Bausteine des Designsystems (ohne Wissen über Einstellungen oder Projekte).
use super::tokens::{t, tint};
use eframe::egui::{self, Color32, RichText};

/// Kennfarbe eines Systems
pub fn system_color(id: &str) -> Color32 {
    match id { "c64" => Color32::from_rgb(0x48, 0x6c, 0xb4), "c128" => Color32::from_rgb(0x3a, 0x8f, 0x9a), "plus4" | "c16" => Color32::from_rgb(0x4f, 0x9a, 0x5c), "vic20" => Color32::from_rgb(0x8a, 0x6a, 0xb0),
        "atari8" => Color32::from_rgb(0xb0, 0x6a, 0x30), "mega65" => Color32::from_rgb(0x84, 0x4c, 0xc0), _ => Color32::GRAY }
}

/// Quadratisches Symbol mit Kürzel
pub fn badge(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size * 1.25, size), egui::Sense::hover());
    ui.painter().rect_filled(rect, 6.0, color);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::monospace(size * 0.42), Color32::WHITE);
}

/// Kleine Marke mit Farbe (Status, „STANDARD“ …)
pub fn chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new().fill(tint(color)).stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.30))).corner_radius(99.0).inner_margin(egui::Margin::symmetric(7, 1)).show(ui, |ui| {
        ui.add(egui::Label::new(RichText::new(text).small().strong().color(color)).extend());
    });
}

/// Kompakte Marke ohne Ring (Zielsystem in der Befehlsleiste): leicht getönte Fläche, kleine Rundung, feste Höhe wie die Nachbarfelder
pub fn tag(ui: &mut egui::Ui, text: &str, color: Color32) {
    let k = crate::ui::tokens::t();
    let fg = color.lerp_to_gamma(k.fg, 0.35);
    let (h, galley) = (TAG_H, ui.painter().layout_no_wrap(text.to_string(), egui::FontId::new(12.0, crate::fonts::ui_strong()), fg));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(galley.size().x + 14.0, h), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, color.gamma_multiply(0.18));
    ui.painter().galley(egui::pos2(rect.left() + 7.0, rect.center().y - galley.size().y / 2.0), galley, fg);
}

/// Höhe von `tag`, gleich der Höhe der Felder der Befehlsleiste
pub const TAG_H: f32 = 24.0;

/// Marke für Adressen, Opcodes und Zähler (Festbreitenschrift)
#[allow(dead_code)]   // Baustein für Debugger, Memory Map und Studios (Phasen 5 bis 7 des Designkonzepts)
pub fn mono_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new().fill(tint(color)).stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.30))).corner_radius(99.0).inner_margin(egui::Margin::symmetric(7, 1)).show(ui, |ui| {
        ui.add(egui::Label::new(RichText::new(text).small().monospace().color(color)).extend());
    });
}

/// Karte um einen Block zusammengehöriger Felder
pub fn card<R>(ui: &mut egui::Ui, selected: bool, body: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let k = t();
    let stroke = if selected { egui::Stroke::new(1.0, k.cy.gamma_multiply(0.6)) } else { egui::Stroke::new(1.0, k.line) };
    egui::Frame::new().fill(k.bg2).stroke(stroke).corner_radius(4.0).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui)
    }).inner
}

/// Abschnittsüberschrift mit Erklärung; `right` füllt die rechte Seite (z. B. „Hinzufügen“)
pub fn section(ui: &mut egui::Ui, title: &str, hint: &str, right: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| { ui.label(RichText::new(title).font(egui::FontId::new(15.0, crate::fonts::ui_strong()))); ui.label(RichText::new(hint).weak().small()); });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), right);
    });
    ui.add_space(4.0);
}

/// Beschriftung für Formularzeilen (feste Breite)
pub fn label(ui: &mut egui::Ui, text: &str) {
    ui.allocate_ui_with_layout(egui::vec2(110.0, 22.0), egui::Layout::left_to_right(egui::Align::Center), |ui| { ui.label(text); });
}

/// Pfadfeld mit Durchsuchen-Knopf; `folder` wählt einen Ordner statt einer Datei. Liefert, ob der Wert geändert wurde.
pub fn path_field(ui: &mut egui::Ui, value: &mut String, hint: &str, folder: bool) -> bool {
    let mut changed = false;
    let w = (ui.available_width() - 330.0).clamp(160.0, 520.0);
    changed |= ui.add(egui::TextEdit::singleline(value).desired_width(w).hint_text(hint).font(egui::TextStyle::Monospace)).changed();
    if ui.button("Durchsuchen …").clicked() {
        let picked = if folder { rfd::FileDialog::new().pick_folder() } else { rfd::FileDialog::new().pick_file() };
        if let Some(p) = picked { *value = p.to_string_lossy().into_owned(); changed = true; }
    }
    changed
}

/// Segmentschalter (genau ein Eintrag aktiv). `options`: (Kennung, Beschriftung). Liefert, ob die Auswahl wechselte.
pub fn segmented(ui: &mut egui::Ui, current: &mut String, options: &[(&str, &str)]) -> bool {
    let k = t();
    let mut changed = false;
    egui::Frame::new().stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(1)).show(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 1.0;
        ui.horizontal(|ui| {
            for (id, text) in options {
                let on = current == id;
                let label = if on { RichText::new(*text).strong().color(k.cy) } else { RichText::new(*text).color(k.mute) };
                let btn = egui::Button::new(label).fill(if on { tint(k.cy) } else { Color32::TRANSPARENT }).stroke(egui::Stroke::NONE).corner_radius(3.0);
                if ui.add(btn).clicked() && !on { *current = id.to_string(); changed = true; }
            }
        });
    });
    changed
}

/// Bitfeld-Schalter: eine Reihe benannter Bits (höchstes Bit links), Klick schaltet um. `bits`: (Bitnummer, Name, Hinweis).
/// Dient Registern und Speicherumschaltung (LORAM/HIRAM/CHAREN, Sprite-Flags, Statusbits). Liefert, ob der Wert geändert wurde.
#[allow(dead_code)]   // Baustein für Debugger, Memory Map und Studios (Phasen 5 bis 7 des Designkonzepts)
pub fn bitfield(ui: &mut egui::Ui, value: &mut u8, bits: &[(u8, &str, &str)]) -> bool {
    let k = t();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let mut sorted: Vec<&(u8, &str, &str)> = bits.iter().collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0));
        for (n, name, hint) in sorted.into_iter().map(|b| (b.0, b.1, b.2)) {
            let on = *value & (1 << n) != 0;
            let text = RichText::new(name).monospace().small().strong().color(if on { k.gr } else { k.dim });
            let btn = egui::Button::new(text).fill(if on { tint(k.gr) } else { k.bg2 }).stroke(egui::Stroke::new(1.0, if on { k.gr.gamma_multiply(0.45) } else { k.line })).corner_radius(4.0);
            let r = ui.add(btn).on_hover_text(format!("Bit {n}: {hint}"));
            if r.clicked() { *value ^= 1 << n; changed = true; }
        }
    });
    changed
}

/// Registerkachel: großer Wert über kleiner Beschriftung; die Antwort reagiert auf Klicks (Kontextmenü, Kopieren)
pub fn reg_tile(ui: &mut egui::Ui, name: &str, value: &str, enabled: bool) -> egui::Response {
    let k = t();
    let col = if enabled { k.gr } else { k.dim };
    // Breite nach Inhalt: `vertical_centered` füllt sonst die ganze Zeile und schiebt die übrigen Kacheln hinaus
    let (fv, fs) = (egui::FontId::new(15.0, crate::fonts::mono_strong()), egui::TextStyle::Small.resolve(ui.style()));
    let w = ui.fonts_mut(|f| f.layout_no_wrap(value.to_string(), fv, col).size().x.max(f.layout_no_wrap(name.to_string(), fs, col).size().x));
    let r = egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
        ui.set_width(w.max(22.0));
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(value).font(egui::FontId::new(15.0, crate::fonts::mono_strong())).color(col));
            ui.label(RichText::new(name).small().color(k.dim));
        });
    });
    ui.interact(r.response.rect, ui.id().with(("regtile", name)), egui::Sense::click())
}

#[cfg(test)]
mod tag_tests {
    #[test]
    fn marke_hat_hoehe_der_nachbarfelder() { assert_eq!(super::TAG_H, 24.0); }
}
