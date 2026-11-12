// SPDX-License-Identifier: GPL-3.0-or-later
//! Schmale Fenster (L5): eine eingeklappte Zone lässt einen schmalen Streifen mit senkrechter Beschriftung stehen; ein Klick darauf
//! (oder das Tastenkürzel der Zone) öffnet sie als Schublade über dem Editor. Die Regeln, wann eingeklappt wird, stehen in `layout::docking`.
use super::*;
use crate::ui::tokens::{t, tint};

/// Streifen einer eingeklappten Zone (links neben dem Editor oder rechts davon). Liefert, ob er angeklickt wurde.
pub(super) fn strip(ui: &mut egui::Ui, id: &str, left: bool, label: &str, open: bool) -> bool {
    let k = t();
    let panel = if left { egui::Panel::left(id.to_string()) } else { egui::Panel::right(id.to_string()) };
    let mut clicked = false;
    panel.exact_size(layout::STRIP_W).resizable(false).frame(egui::Frame::new().fill(k.bg1)).show_inside(ui, |ui| {
        let r = ui.max_rect();
        let resp = ui.allocate_rect(r, egui::Sense::click());
        if open { ui.painter().rect_filled(r, 0.0, tint(k.cy).gamma_multiply(0.7)); } else if resp.hovered() { ui.painter().rect_filled(r, 0.0, k.bg2); }
        let edge = if left { r.right() } else { r.left() };
        ui.painter().vline(edge, r.y_range(), egui::Stroke::new(1.0, k.line));
        let col = if open { k.cy } else if resp.hovered() { k.fg } else { k.mute };
        let g = ui.painter().layout_no_wrap(label.to_string(), egui::FontId::new(11.5, crate::fonts::ui_strong()), col);
        // Text von unten nach oben lesbar: um −90° gedreht, der Anfang des Textes unten
        let pos = egui::pos2(r.center().x - g.size().y / 2.0, r.top() + 12.0 + g.size().x);
        ui.painter().add(egui::epaint::TextShape::new(pos, g, col).with_angle(-std::f32::consts::FRAC_PI_2));
        clicked = resp.on_hover_text(format!("{label} öffnen")).on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
    });
    clicked
}

/// Schublade über dem Editorbereich `area` (links oder rechts bündig, volle Höhe). Liefert ihr Rechteck, damit ein Klick daneben sie schließen kann.
pub(super) fn drawer(ctx: &egui::Context, area: egui::Rect, left: bool, zone_w: f32, id: &str, add: impl FnOnce(&mut egui::Ui)) -> egui::Rect {
    let k = t();
    let w = layout::drawer_width(zone_w, ctx.content_rect().width());
    let h = area.height();
    let pos = if left { area.left_top() } else { egui::pos2(area.right() - w, area.top()) };
    let shadow = egui::Shadow { offset: [if left { 3 } else { -3 }, 0], blur: 14, spread: 0, color: egui::Color32::from_black_alpha(90) };
    egui::Area::new(egui::Id::new(id)).order(egui::Order::Foreground).fixed_pos(pos).constrain(false).show(ctx, |ui| {
        egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.line)).shadow(shadow).show(ui, |ui| {
            ui.set_min_size(egui::vec2(w, h)); ui.set_max_size(egui::vec2(w, h));
            add(ui);
        });
    }).response.rect
}
