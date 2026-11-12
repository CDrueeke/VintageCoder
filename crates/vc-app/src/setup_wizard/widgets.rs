// SPDX-License-Identifier: GPL-3.0-or-later
//! Bausteine des Konfigurationsassistenten: Gerätesilhouetten (mit dem Painter gezeichnet, keine Bilder), C64-Streifen, Tastenkappe,
//! blauer `READY.`-Bildschirm, Überzeile und Statusmarken. Farben kommen aus den Tokens; der Status ist nie nur Farbe (Symbol und Text).
use crate::ui::tokens::t;
use crate::ui::widgets::system_color;
use eframe::egui::{self, Color32, RichText};
use vc_ide::setup::Badge;

/// Die fünf Farben des „C64-Streifens“ (Rot bis Blau)
pub const STRIPE: [Color32; 5] = [Color32::from_rgb(0xd0, 0x4a, 0x3a), Color32::from_rgb(0xe8, 0x8e, 0x2c), Color32::from_rgb(0xe6, 0xc8, 0x3a), Color32::from_rgb(0x4f, 0xb0, 0x5c), Color32::from_rgb(0x48, 0x6c, 0xb4)];

/// Blauer Bildschirm des C64
pub const C64_BLUE: Color32 = Color32::from_rgb(0x40, 0x31, 0x8d);
pub const C64_BORDER: Color32 = Color32::from_rgb(0x88, 0x7e, 0xcb);

pub fn stripe(ui: &mut egui::Ui, width: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 5.0), egui::Sense::hover());
    let w = rect.width() / STRIPE.len() as f32;
    for (i, c) in STRIPE.iter().enumerate() { ui.painter().rect_filled(egui::Rect::from_min_size(rect.min + egui::vec2(w * i as f32, 0.0), egui::vec2(w + 0.5, 5.0)), 0.0, *c); }
}

/// Überzeile in Versalien (`SCHRITT 3 VON 12 · SYSTEME`)
pub fn overline(ui: &mut egui::Ui, text: &str) { ui.label(RichText::new(text.to_uppercase()).monospace().small().color(t().cy)); }

/// Gerätesilhouette eines Systems (Rechteck und Tasten, in der Farbe des Systems)
pub fn silhouette(ui: &mut egui::Ui, system: &str, size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter_at(rect);
    let col = system_color(system);
    let body = rect.shrink2(egui::vec2(size.x * 0.06, size.y * 0.16));
    let dark = col.gamma_multiply(0.55);
    match system {
        "atari8" => {
            p.rect_filled(body, 4.0, Color32::from_gray(0x2a));
            for i in 0..4 { p.rect_filled(egui::Rect::from_min_size(egui::pos2(body.left() + 4.0 + i as f32 * 9.0, body.top() + 3.0), egui::vec2(7.0, 4.0)), 1.0, col); }
        }
        "mega65" => { p.rect_filled(body, 4.0, Color32::from_gray(0x3a)); p.rect_filled(egui::Rect::from_min_size(body.left_bottom() - egui::vec2(0.0, 5.0), egui::vec2(body.width(), 5.0)), 0.0, col); }
        "vic20" => { p.rect_filled(body, 4.0, Color32::from_rgb(0xd8, 0xcf, 0xb8)); p.rect_filled(egui::Rect::from_min_size(body.min, egui::vec2(body.width(), 4.0)), 0.0, col); }
        "c128" => { p.rect_filled(body, 4.0, Color32::from_rgb(0xc8, 0xc0, 0xaa)); p.rect_filled(egui::Rect::from_min_size(body.min + egui::vec2(body.width() * 0.7, 3.0), egui::vec2(body.width() * 0.25, 5.0)), 1.0, dark); }
        "plus4" | "c16" => { p.rect_filled(body, 4.0, Color32::from_rgb(0xb8, 0xb8, 0xb0)); p.rect_filled(egui::Rect::from_min_size(body.min + egui::vec2(3.0, 3.0), egui::vec2(body.width() * 0.4, 3.0)), 1.0, col); }
        _ => { p.rect_filled(body, 4.0, Color32::from_rgb(0xd0, 0xc8, 0xb0)); p.rect_filled(egui::Rect::from_min_size(body.min + egui::vec2(4.0, 3.0), egui::vec2(body.width() * 0.5, 3.0)), 1.0, col); }
    }
    // Tasten
    let rows = 3;
    for r in 0..rows {
        let y = body.top() + body.height() * (0.38 + 0.17 * r as f32);
        let n = 11 - r;
        for i in 0..n { p.rect_filled(egui::Rect::from_min_size(egui::pos2(body.left() + 5.0 + i as f32 * (body.width() - 10.0) / n as f32, y), egui::vec2((body.width() - 10.0) / n as f32 - 2.0, 3.5)), 1.0, dark); }
    }
}

/// Große Tastenkappe (F5)
pub fn keycap(ui: &mut egui::Ui, text: &str) {
    let k = t();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 54.0), egui::Sense::hover());
    ui.painter().rect_filled(rect.translate(egui::vec2(0.0, 3.0)), 8.0, k.line);
    ui.painter().rect(rect, 8.0, k.bg3, egui::Stroke::new(1.5, k.cy), egui::StrokeKind::Inside);
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::monospace(22.0), k.fg);
}

/// Blauer C64-Bildschirm mit Text und Cursor
pub fn ready_screen(ui: &mut egui::Ui, lines: &[&str], width: f32, cursor_on: bool) {
    let h = 18.0 * (lines.len() as f32 + 1.0) + 28.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, h), egui::Sense::hover());
    ui.painter().rect_filled(rect, 6.0, C64_BORDER);
    let inner = rect.shrink(10.0);
    ui.painter().rect_filled(inner, 2.0, C64_BLUE);
    let mut y = inner.top() + 6.0;
    for l in lines { ui.painter().text(egui::pos2(inner.left() + 8.0, y), egui::Align2::LEFT_TOP, l, egui::FontId::monospace(14.0), C64_BORDER); y += 18.0; }
    if cursor_on { ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(inner.left() + 8.0, y), egui::vec2(9.0, 14.0)), 0.0, C64_BORDER); }
}

/// Symbol und Farbe eines Schrittes in der Leiste
pub fn badge_symbol(b: Badge, done: bool) -> (&'static str, Color32) {
    let k = t();
    match b { Badge::Problem => ("!", k.rd), Badge::Skipped => ("–", k.dim), Badge::Changed => ("~", k.am), Badge::New => ("+", k.cy), Badge::Unchanged => if done { ("✓", k.gr) } else { ("", k.dim) } }
}

/// Meldung mit Symbol, Text und Farbe in einer Zeile (Warnung, Fehler, Erfolg)
pub fn note(ui: &mut egui::Ui, symbol: &str, text: &str, color: Color32) {
    ui.horizontal_wrapped(|ui| { ui.label(RichText::new(symbol).strong().color(color)); ui.label(RichText::new(text).small().color(color)); });
}

/// Eine Karte, die sich wie ein Knopf wählen lässt; `body` zeichnet den Inhalt. Liefert die Antwort (für Klicks).
pub fn choice_card(ui: &mut egui::Ui, selected: bool, width: f32, body: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    let k = t();
    let stroke = if selected { egui::Stroke::new(2.0, k.cy) } else { egui::Stroke::new(1.0, k.line) };
    let r = egui::Frame::new().fill(if selected { crate::ui::tokens::tint(k.cy) } else { k.bg2 }).stroke(stroke).corner_radius(6.0).inner_margin(egui::Margin::same(12)).show(ui, |ui| { ui.set_width(width - 26.0); ui.vertical(body); });
    ui.interact(r.response.rect, ui.id().with(("choice", r.response.rect.min.x as i32, r.response.rect.min.y as i32)), egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand)
}
