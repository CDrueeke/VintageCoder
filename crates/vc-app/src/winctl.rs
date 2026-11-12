// SPDX-License-Identifier: GPL-3.0-or-later
//! Eigene Fenstersteuerung (Minimieren, Maximieren, Schließen, Größe ändern) für Systeme, deren Fenstermanager keine Titelleiste
//! liefert (z. B. Tiling- oder Minimal-Setups unter Linux). Plattformneutral, aber unter macOS nie aktiv (dort bleibt die Ampel).
//! Ob der Fenstermanager dekoriert, lässt sich nicht verlässlich erkennen; deshalb schaltet der Nutzer die Steuerung im Menü
//! „Fenster“ zu. Die Wahl liegt in `fenster.json` im Konfigurationsordner.
use eframe::egui::{self, pos2, vec2, Rect, ResizeDirection, ViewportCommand};
use std::sync::atomic::{AtomicBool, Ordering};

static OWN: AtomicBool = AtomicBool::new(false);
/// Breite der Griffzone am Fensterrand
pub const BORDER: f32 = 5.0;

fn file() -> Option<std::path::PathBuf> { Some(vc_core::userdirs::config_dir()?.join("fenster.json")) }

/// Gespeicherte Wahl laden (beim Start)
pub fn load() {
    if std::env::var("VC_SHOT").is_ok() { return; }
    let v: Option<serde_json::Value> = file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str(&t).ok());
    OWN.store(v.and_then(|v| v["own_controls"].as_bool()).unwrap_or(false), Ordering::Relaxed);
}

/// Wahl setzen und speichern
pub fn set_enabled(on: bool) {
    OWN.store(on, Ordering::Relaxed);
    if std::env::var("VC_SHOT").is_ok() { return; }
    if let Some(f) = file() { if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); } let _ = std::fs::write(f, format!("{}\n", serde_json::json!({ "own_controls": on }))); }
}

/// Ob die Knöpfe und Griffzonen gezeichnet werden: nur wenn gewählt und nicht unter macOS
pub fn enabled() -> bool { !crate::ui::titlebar::active() && OWN.load(Ordering::Relaxed) }

/// Richtung zum Größenändern für eine Zeigerposition im Fenster (Ecken vor Kanten); `None` im Inneren
pub fn resize_dir(p: egui::Pos2, r: Rect, b: f32) -> Option<ResizeDirection> {
    if !r.contains(p) { return None; }
    let (l, rt, t, bt) = (p.x - r.left() < b, r.right() - p.x < b, p.y - r.top() < b, r.bottom() - p.y < b);
    use ResizeDirection::*;
    match (l, rt, t, bt) {
        (true, _, true, _) => Some(NorthWest), (_, true, true, _) => Some(NorthEast),
        (true, _, _, true) => Some(SouthWest), (_, true, _, true) => Some(SouthEast),
        (true, ..) => Some(West), (_, true, ..) => Some(East), (_, _, true, _) => Some(North), (.., true) => Some(South),
        _ => None,
    }
}

fn cursor(d: ResizeDirection) -> egui::CursorIcon {
    use ResizeDirection::*;
    match d { North => egui::CursorIcon::ResizeNorth, South => egui::CursorIcon::ResizeSouth, East => egui::CursorIcon::ResizeEast, West => egui::CursorIcon::ResizeWest, NorthEast => egui::CursorIcon::ResizeNorthEast, NorthWest => egui::CursorIcon::ResizeNorthWest, SouthEast => egui::CursorIcon::ResizeSouthEast, SouthWest => egui::CursorIcon::ResizeSouthWest }
}

/// Griffzonen am Rand des aktuellen Fensters: Zeiger ändert sich, Ziehen ändert die Größe. Am Ende eines Bildes aufrufen.
pub fn edges(ctx: &egui::Context) {
    if !enabled() { return; }
    let (full, max, pos, pressed, rect) = ctx.input(|i| { let v = i.viewport(); (v.fullscreen.unwrap_or(false), v.maximized.unwrap_or(false), i.pointer.hover_pos(), i.pointer.primary_pressed(), i.viewport_rect()) });
    if full || max { return; }
    let Some(d) = pos.and_then(|p| resize_dir(p, rect, BORDER)) else { return };
    ctx.set_cursor_icon(cursor(d));
    if pressed { ctx.send_viewport_cmd(ViewportCommand::BeginResize(d)); }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind { Minimize, Maximize, Close }

fn button(ui: &mut egui::Ui, kind: Kind, maximized: bool) -> egui::Response {
    let k = crate::ui::tokens::t();
    let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 24.0), egui::Sense::click());
    let hot = if kind == Kind::Close { k.rd } else { k.bg3 };
    if resp.hovered() { ui.painter().rect_filled(rect, 3.0, hot); }
    let (c, s) = (rect.center(), egui::Stroke::new(1.3, if resp.hovered() && kind == Kind::Close { egui::Color32::WHITE } else { k.fg }));
    let p = ui.painter();
    match kind {
        Kind::Minimize => { p.line_segment([pos2(c.x - 5.0, c.y + 3.0), pos2(c.x + 5.0, c.y + 3.0)], s); }
        Kind::Maximize if maximized => { p.rect_stroke(Rect::from_center_size(pos2(c.x - 1.0, c.y + 1.0), vec2(8.0, 8.0)), 0.0, s, egui::StrokeKind::Inside); p.line_segment([pos2(c.x - 3.0, c.y - 4.0), pos2(c.x + 5.0, c.y - 4.0)], s); p.line_segment([pos2(c.x + 5.0, c.y - 4.0), pos2(c.x + 5.0, c.y + 3.0)], s); }
        Kind::Maximize => { p.rect_stroke(Rect::from_center_size(c, vec2(10.0, 9.0)), 0.0, s, egui::StrokeKind::Inside); }
        Kind::Close => { p.line_segment([pos2(c.x - 4.5, c.y - 4.5), pos2(c.x + 4.5, c.y + 4.5)], s); p.line_segment([pos2(c.x - 4.5, c.y + 4.5), pos2(c.x + 4.5, c.y - 4.5)], s); }
    }
    resp.on_hover_text(match kind { Kind::Minimize => "Minimieren", Kind::Maximize => if maximized { "Wiederherstellen" } else { "Maximieren" }, Kind::Close => "Schließen" })
}

/// Befehle für einen Knopf, gegeben den Maximiert-Zustand
pub fn command(kind: Kind, maximized: bool) -> ViewportCommand {
    match kind { Kind::Minimize => ViewportCommand::Minimized(true), Kind::Maximize => ViewportCommand::Maximized(!maximized), Kind::Close => ViewportCommand::Close }
}

/// Die drei Knöpfe in einem `right_to_left`-Layout (Schließen ganz rechts); tut nichts, wenn die Steuerung aus ist
pub fn controls(ui: &mut egui::Ui) {
    if !enabled() { return; }
    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
    for kind in [Kind::Close, Kind::Maximize, Kind::Minimize] {
        if button(ui, kind, max).clicked() { ui.ctx().send_viewport_cmd(command(kind, max)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn griffzonen() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        assert_eq!(resize_dir(pos2(200.0, 150.0), r, BORDER), None);
        assert_eq!(resize_dir(pos2(2.0, 150.0), r, BORDER), Some(ResizeDirection::West));
        assert_eq!(resize_dir(pos2(398.0, 150.0), r, BORDER), Some(ResizeDirection::East));
        assert_eq!(resize_dir(pos2(200.0, 1.0), r, BORDER), Some(ResizeDirection::North));
        assert_eq!(resize_dir(pos2(200.0, 299.0), r, BORDER), Some(ResizeDirection::South));
        assert_eq!(resize_dir(pos2(1.0, 1.0), r, BORDER), Some(ResizeDirection::NorthWest));
        assert_eq!(resize_dir(pos2(399.0, 299.0), r, BORDER), Some(ResizeDirection::SouthEast));
        assert_eq!(resize_dir(pos2(-1.0, 10.0), r, BORDER), None);
    }

    #[test]
    fn befehle() {
        assert!(matches!(command(Kind::Minimize, false), ViewportCommand::Minimized(true)));
        assert!(matches!(command(Kind::Maximize, false), ViewportCommand::Maximized(true)));
        assert!(matches!(command(Kind::Maximize, true), ViewportCommand::Maximized(false)));
        assert!(matches!(command(Kind::Close, false), ViewportCommand::Close));
    }

    #[test]
    fn unter_macos_nie_aktiv() {
        OWN.store(true, Ordering::Relaxed);
        assert_eq!(enabled(), !cfg!(target_os = "macos"));
        OWN.store(false, Ordering::Relaxed);
    }
}
