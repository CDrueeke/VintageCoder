// SPDX-License-Identifier: GPL-3.0-or-later
//! Übersetzt die Tokens in egui-Einstellungen (Flächen, Rahmen, Ecken, Schriftgrößen) für Dunkel und Hell.
use super::tokens::{Tokens, DARK, LIGHT};
use crate::fonts;
use eframe::egui::{self, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Theme};

fn visuals(t: &Tokens, dark: bool) -> egui::Visuals {
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    let r = CornerRadius::same(4);
    v.panel_fill = t.bg1;
    v.window_fill = t.bg3;
    v.window_stroke = Stroke::new(1.0, t.cy.gamma_multiply(0.30));
    v.window_corner_radius = CornerRadius::same(8);
    v.menu_corner_radius = r;
    v.extreme_bg_color = t.bg0;
    v.faint_bg_color = t.bg2;
    v.code_bg_color = t.bg2;
    v.hyperlink_color = t.cy;
    v.warn_fg_color = t.am;
    v.error_fg_color = t.rd;
    v.override_text_color = None;
    v.selection.bg_fill = t.cy.gamma_multiply(0.28);
    v.selection.stroke = Stroke::new(1.0, t.cy);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.bg1;
    w.noninteractive.weak_bg_fill = t.bg1;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.line);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.fg);
    w.inactive.bg_fill = t.line.lerp_to_gamma(t.mute, 0.3);   // Schieberbahn, Häkchenfeld: sichtbar auch auf Karten (Fläche bg2)
    w.inactive.weak_bg_fill = t.bg2;
    w.inactive.bg_stroke = Stroke::new(1.0, t.line);
    w.inactive.fg_stroke = Stroke::new(1.0, t.fg);
    w.hovered.bg_fill = t.bg3;
    w.hovered.weak_bg_fill = t.bg3;
    w.hovered.bg_stroke = Stroke::new(1.0, t.cy.gamma_multiply(0.7));
    w.hovered.fg_stroke = Stroke::new(1.5, t.cy);
    w.active.bg_fill = t.cy.gamma_multiply(0.22);
    w.active.weak_bg_fill = t.cy.gamma_multiply(0.22);
    w.active.bg_stroke = Stroke::new(1.0, t.cy);
    // egui nimmt den „starken“ Text aus der aktiven Widget-Farbe: Textfarbe, nicht Akzent
    w.active.fg_stroke = Stroke::new(1.5, t.fg);
    w.open.bg_fill = t.bg3;
    w.open.weak_bg_fill = t.bg3;
    w.open.bg_stroke = Stroke::new(1.0, t.line);
    for ws in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] { ws.corner_radius = r; }
    v
}

/// Einmal beim Start: Visuals beider Schemas und Schriftgrößen. Die Schriftgröße der Festbreitenschrift setzt `View::apply`.
pub fn install(ctx: &egui::Context) {
    ctx.set_visuals_of(Theme::Dark, visuals(&DARK, true));
    ctx.set_visuals_of(Theme::Light, visuals(&LIGHT, false));
    for theme in [Theme::Dark, Theme::Light] {
        ctx.style_mut_of(theme, |s| {
            s.text_styles.insert(TextStyle::Body, FontId::new(13.0, FontFamily::Proportional));
            s.text_styles.insert(TextStyle::Button, FontId::new(13.0, FontFamily::Proportional));
            s.text_styles.insert(TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
            s.text_styles.insert(TextStyle::Heading, FontId::new(18.0, fonts::ui_strong()));
            s.spacing.item_spacing = egui::vec2(8.0, 6.0);
            s.spacing.button_padding = egui::vec2(10.0, 4.0);
            s.spacing.interact_size.y = 24.0;
        });
    }
}

/// Reiterleisten der Dock-Flächen im Stil der Studie: dichte Reiter (28 px), aktiver Reiter mit leichtem Cyan-Grund und Cyan-Schrift
/// (der Unterstrich kommt von `dock_tab_overlay`), die Schließen-Kreuze zeichnet `dock_tab_overlay` nur bei Hover und auf dem aktiven Reiter.
pub fn dock(style: &mut egui_dock::Style) {
    use eframe::egui::{Color32, CornerRadius};
    let k = super::tokens::t();
    style.tab_bar.height = 28.0;
    style.tab_bar.bg_fill = k.bg1;
    style.tab_bar.hline_color = k.line;
    style.tab_bar.corner_radius = CornerRadius::ZERO;
    style.main_surface_border_rounding = CornerRadius::ZERO;
    style.main_surface_border_stroke = egui::Stroke::NONE;
    style.tab.spacing = 0.0;
    let tab = |bg: Color32, fg: Color32| egui_dock::TabInteractionStyle { outline_color: Color32::TRANSPARENT, corner_radius: CornerRadius::ZERO, bg_fill: bg, text_color: fg };
    let active_bg = k.bg1.lerp_to_gamma(k.cy, 0.12);
    style.tab.active = tab(active_bg, k.cy);
    style.tab.focused = tab(active_bg, k.cy);
    style.tab.active_with_kb_focus = tab(active_bg, k.cy);
    style.tab.focused_with_kb_focus = tab(active_bg, k.cy);
    style.tab.inactive = tab(k.bg1, k.mute);
    style.tab.inactive_with_kb_focus = tab(k.bg1, k.mute);
    style.tab.hovered = tab(k.bg2, k.fg);
    style.tab.tab_body.stroke = egui::Stroke::NONE;
    style.tab.tab_body.corner_radius = CornerRadius::ZERO;
    style.tab.tab_body.bg_fill = k.bg0;
    style.buttons.close_tab_color = Color32::TRANSPARENT;
    style.buttons.close_tab_active_color = k.fg;
    style.buttons.close_tab_bg_fill = k.bg3;
}

/// Zusätze auf einem Reiter nach dem Zeichnen: Cyan-Unterstrich beim aktiven, Schließen-Kreuz bei Hover und beim aktiven (`closeable`)
pub fn dock_tab_overlay(r: &egui::Response, active: bool, closeable: bool) {
    let k = super::tokens::t();
    let p = r.ctx.layer_painter(r.layer_id);
    if active { p.rect_filled(egui::Rect::from_min_max(egui::pos2(r.rect.left(), r.rect.bottom() - 2.0), r.rect.right_bottom()), 0.0, k.cy); }
    if closeable && (active || r.hovered()) {
        let c = egui::Rect::from_center_size(egui::pos2(r.rect.right() - 12.0, r.rect.center().y), egui::vec2(9.0, 9.0));
        let s = egui::Stroke::new(1.0, if active { k.cy } else { k.mute });
        p.line_segment([c.left_top(), c.right_bottom()], s);
        p.line_segment([c.left_bottom(), c.right_top()], s);
    }
}
