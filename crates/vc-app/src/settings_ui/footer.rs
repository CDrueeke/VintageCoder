// SPDX-License-Identifier: GPL-3.0-or-later
//! Fußzeile: Speicherort, Änderungsstatus sowie „Zurücksetzen“ und „Änderungen anwenden“.
use super::Act;
use eframe::egui::{self, RichText};

pub fn show(ui: &mut egui::Ui, dirty: bool) -> Act {
    let mut act = Act::None;
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let file = vc_ide::settings::file().map(|f| f.display().to_string()).unwrap_or_else(|| "–".into());
        ui.label(RichText::new("Gespeichert in").weak().small());
        ui.label(RichText::new(file).monospace().small()).on_hover_text("Gilt für alle Projekte. Die Datei enthält die Passwörter und ist nur für dich lesbar.");
        if dirty { ui.label(RichText::new("●  ungespeicherte Änderungen").small().color(crate::theme::warn())); }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_enabled(dirty, egui::Button::new(RichText::new("Änderungen anwenden").strong()).fill(crate::theme::accent().gamma_multiply(if dirty { 0.55 } else { 0.15 }))).clicked() { act = Act::Save; }
            if ui.add_enabled(dirty, egui::Button::new("Zurücksetzen")).clicked() { act = Act::Revert; }
        });
    });
    ui.add_space(6.0);
    act
}
