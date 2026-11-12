// SPDX-License-Identifier: GPL-3.0-or-later
//! Schrittleiste (links, mit Gruppen, Kreisen und Statusmarken) und Fortschrittsleiste (je Schritt ein Segment).
use super::{widgets, Nav, SetupWizard};
use crate::ui::tokens::t;
use eframe::egui::{self, RichText};
use vc_ide::setup::{Badge, Mode, Topic};

pub fn show(ui: &mut egui::Ui, w: &mut SetupWizard) -> Option<Nav> {
    let k = t();
    let mut nav = None;
    widgets::stripe(ui, ui.available_width());
    ui.add_space(8.0);
    ui.label(RichText::new("VintageCoder").font(egui::FontId::new(16.0, crate::fonts::ui_strong())).color(k.fg));
    ui.label(RichText::new(if w.recall { "Konfigurationsassistent" } else { "Einrichtung" }).small().color(k.mute));
    ui.add_space(10.0);
    let mut group = "";
    let quick = w.mode == Mode::Quick;
    let mut auto_hidden = false;
    for (i, topic) in w.steps.clone().into_iter().enumerate() {
        if topic.group() != group { group = topic.group(); ui.add_space(6.0); ui.label(RichText::new(group.to_uppercase()).monospace().small().color(k.dim)); }
        if let Some(r) = step_row(ui, w, i, topic) { nav = Some(r); }
    }
    if quick {
        // Schnelleinrichtung: die Schritte 5 bis 9 laufen automatisch und stehen in einer Zeile
        if !auto_hidden { auto_hidden = true; }
        ui.add_space(8.0);
        ui.label(RichText::new("Automatisch erledigt: Emulatoren, ROMs, Hardware, Startziel, Editor").small().color(k.mute));
        if ui.link(RichText::new("Doch ansehen").small()).clicked() { w.mode = Mode::Full; w.steps = w.mode.steps(); w.jump_to(Topic::Emulators); }
    }
    let _ = auto_hidden;
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(6.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Bleibt auf diesem Rechner").strong().small().color(k.gr));
            ui.label(RichText::new("Kein Netzwerk, keine Telemetrie, kein Download ohne dein Ja.").small().color(k.mute));
        });
    });
    nav
}

fn step_row(ui: &mut egui::Ui, w: &SetupWizard, i: usize, topic: Topic) -> Option<Nav> {
    let k = t();
    let current = i == w.idx;
    let done = i < w.idx || (w.reached >= i && !current && i < w.reached);
    let badge = w.badges.get(&topic).copied().unwrap_or(Badge::Unchanged);
    let skipped = w.skipped.contains(&topic);
    let (sym, col) = if skipped { widgets::badge_symbol(Badge::Skipped, false) } else if w.recall || done { widgets::badge_symbol(if matches!(topic, Topic::Welcome | Topic::Summary | Topic::Done) { Badge::Unchanged } else { badge }, done || w.recall) } else { ("", k.dim) };
    let reachable = i <= w.reached || w.recall;
    let h = 26.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), h), if reachable { egui::Sense::click() } else { egui::Sense::hover() });
    let p = ui.painter();
    if current {
        // Phosphor-Glühen: zwei dünne, durchscheinende Rahmen um den aktuellen Schritt
        p.rect_filled(rect, 4.0, crate::ui::tokens::tint(k.cy));
        p.rect_stroke(rect.expand(1.5), 5.0, egui::Stroke::new(1.0, k.cy.gamma_multiply(0.25)), egui::StrokeKind::Outside);
        p.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, k.cy.gamma_multiply(0.6)), egui::StrokeKind::Inside);
    } else if resp.hovered() && reachable { p.rect_filled(rect, 4.0, k.bg2); }
    let c = egui::pos2(rect.left() + 14.0, rect.center().y);
    p.circle(c, 9.0, if current { k.cy } else { k.bg3 }, egui::Stroke::new(1.0, if current { k.cy } else { col.gamma_multiply(0.8) }));
    let txt = if sym.is_empty() || current { (i + 1).to_string() } else { sym.to_string() };
    p.text(c, egui::Align2::CENTER_CENTER, txt, egui::FontId::monospace(11.0), if current { k.on_cy } else if sym.is_empty() { k.mute } else { col });
    p.text(egui::pos2(rect.left() + 32.0, rect.center().y), egui::Align2::LEFT_CENTER, topic.title(), egui::FontId::new(13.0, if current { crate::fonts::ui_strong() } else { egui::FontFamily::Proportional }), if current { k.fg } else if reachable { k.fg } else { k.mute });
    if w.recall && !current && badge != Badge::Unchanged && !skipped {
        let text = match badge { Badge::New => "neu", Badge::Changed => "geändert", Badge::Problem => "Problem", _ => "" };
        if !text.is_empty() { p.text(egui::pos2(rect.right() - 6.0, rect.center().y), egui::Align2::RIGHT_CENTER, text, egui::FontId::monospace(10.0), col); }
    }
    if resp.clicked() { return Some(Nav::Jump(i)); }
    None
}

/// Fortschrittsleiste: je Schritt ein Segment; das Segment des aktuellen Schrittes füllt sich weich
pub fn progress(ui: &mut egui::Ui, w: &SetupWizard) {
    let k = t();
    let n = w.steps.len().max(1);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 6.0), egui::Sense::hover());
    let gap = 3.0;
    let sw = (rect.width() - gap * (n as f32 - 1.0)) / n as f32;
    for i in 0..n {
        let r = egui::Rect::from_min_size(rect.min + egui::vec2(i as f32 * (sw + gap), 0.0), egui::vec2(sw, 6.0));
        ui.painter().rect_filled(r, 2.0, k.line);
        let fill = ui.ctx().animate_value_with_time(egui::Id::new(("setup_prog", i)), if i < w.idx { 1.0 } else if i == w.idx { 0.5 } else { 0.0 }, 0.15);
        if fill > 0.0 { ui.painter().rect_filled(egui::Rect::from_min_size(r.min, egui::vec2(sw * fill, 6.0)), 2.0, if i < w.idx { k.gr } else { k.cy }); }
    }
}
