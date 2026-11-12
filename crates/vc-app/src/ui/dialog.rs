// SPDX-License-Identifier: GPL-3.0-or-later
//! Einheitlicher Rahmen für Dialoge: Fläche `bg3` mit Cyan-Rand (30 %) und 8 px Ecken, Kopf mit Titel, Untertitel und Marken,
//! Fuß mit Primär- und Sekundärknopf. Esc löst den Sekundärknopf aus, Enter den Primärknopf, wenn die Eingaben gültig sind.
use super::tokens::t;
use super::widgets::chip;
use eframe::egui::{self, Color32, RichText};

/// Was der Nutzer im Dialog gewählt hat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act { None, Primary, Secondary }

/// Kopf und Fuß eines Dialogs
pub struct Dlg<'a> {
    pub title: &'a str,
    /// Zeile unter dem Titel (Datei, Ordner …); leer = keine
    pub sub: &'a str,
    pub chips: &'a [(String, Color32)],
    pub primary: &'a str,
    pub secondary: &'a str,
}

impl<'a> Dlg<'a> {
    pub fn new(title: &'a str, primary: &'a str) -> Self { Dlg { title, sub: "", chips: &[], primary, secondary: "Abbrechen" } }
    pub fn sub(mut self, s: &'a str) -> Self { self.sub = s; self }
    pub fn chips(mut self, c: &'a [(String, Color32)]) -> Self { self.chips = c; self }
}

/// Wirkt eine Taste? Esc gewinnt immer; Enter nur bei gültiger Eingabe (`ok`).
pub fn key_act(esc: bool, enter: bool, ok: bool) -> Act {
    if esc { Act::Secondary } else if enter && ok { Act::Primary } else { Act::None }
}

/// Rahmen des Dialogfensters
pub fn frame() -> egui::Frame {
    let k = t();
    egui::Frame::new().fill(k.bg3).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.30))).corner_radius(8.0).inner_margin(egui::Margin::same(16))
        .shadow(egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(90) })
}

/// Zeigt den Dialog in der Bildmitte. `body` zeichnet die Felder und liefert, ob der Primärknopf gelten darf.
pub fn show(ctx: &egui::Context, d: &Dlg, body: impl FnOnce(&mut egui::Ui) -> bool) -> Act {
    let k = t();
    let mut act = Act::None;
    let (esc, enter) = ctx.input(|i| (i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Enter)));
    let mut ok = true;
    egui::Window::new(d.title).id(egui::Id::new(("dlg", d.title))).title_bar(false).collapsible(false).resizable(false).frame(frame())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        ui.set_min_width(340.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(d.title).font(egui::FontId::new(16.0, crate::fonts::ui_strong())).color(k.fg));
            for (text, c) in d.chips { chip(ui, text, *c); }
        });
        if !d.sub.is_empty() { ui.add(egui::Label::new(RichText::new(d.sub).small().color(k.mute)).wrap()); }
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);
        ok = body(ui);
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);
        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 26.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !d.primary.is_empty() && crate::shell::action_button(ui, d.primary, crate::shell::Kind::Primary, ok).clicked() { act = Act::Primary; }
            if ui.button(d.secondary).clicked() { act = Act::Secondary; }
        });
    });
    if act == Act::None { act = key_act(esc, enter && !d.primary.is_empty(), ok); }
    act
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_gewinnt_und_enter_braucht_gueltige_eingabe() {
        assert_eq!(key_act(true, true, true), Act::Secondary);
        assert_eq!(key_act(false, true, true), Act::Primary);
        assert_eq!(key_act(false, true, false), Act::None);
        assert_eq!(key_act(false, false, true), Act::None);
    }

    #[test]
    fn dialog_zeichnet_und_meldet_knopf() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut seen_ok = false;
        for _ in 0..2 {
            let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0))), ..Default::default() }, |ui| {
                let a = show(ui.ctx(), &Dlg::new("Test", "OK").sub("unter"), |ui| { ui.label("Inhalt"); seen_ok = true; true });
                assert_eq!(a, Act::None);
            });
            o.textures_delta.clear();
        }
        assert!(seen_ok);
    }
}
