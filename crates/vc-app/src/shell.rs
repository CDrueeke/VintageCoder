// SPDX-License-Identifier: GPL-3.0-or-later
//! Rahmen der Oberfläche im Designsystem: Modus-Tabzeile, Statusleiste, Aktionsknöpfe der Werkzeugleiste und Zeilen des Projektbaums.
//! Reine Darstellung; die Anwendung (`app.rs`) liefert die Werte und führt die Aktionen aus.
use crate::project::Group;
use crate::ui::tokens::{t, tint};
use eframe::egui::{self, Color32, RichText};

/// Version des Assemblers: `vc-asm` und die Oberfläche haben dieselbe Arbeitsbereich-Version
pub fn assembler_label() -> String { format!("vcasm {}", env!("CARGO_PKG_VERSION")) }

/// Ein Eintrag der Modus-Tabzeile
pub struct ModeTab { pub label: &'static str, pub active: bool, pub open: bool, pub count: Option<usize>, pub hint: String }

/// Zeichnet die Modus-Tabzeile; liefert den angeklickten Eintrag. `active` = Arbeitsmodus der Oberfläche, `open` = eigenes Fenster offen.
pub fn mode_bar(ui: &mut egui::Ui, tabs: &[ModeTab]) -> Option<usize> { mode_bar_ex(ui, tabs, None).0 }

/// Wie `mode_bar`, dazu: Kontextmenü je Eintrag (`menu`) und ein Eintrag, der nach unten aus der Zeile gezogen und losgelassen wurde
/// (Rückgabe: angeklickt, losgelassen unterhalb der Zeile)
pub fn mode_bar_ex(ui: &mut egui::Ui, tabs: &[ModeTab], mut menu: Option<&mut dyn FnMut(usize, &mut egui::Ui)>) -> (Option<usize>, Option<usize>) {
    let k = t();
    let mut clicked = None;
    let mut dropped = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (i, tab) in tabs.iter().enumerate() {
            let text = RichText::new(tab.label).font(egui::FontId::new(12.5, if tab.active { crate::fonts::ui_strong() } else { egui::FontFamily::Proportional }));
            let galley = ui.painter().layout_no_wrap(tab.label.to_string(), egui::FontId::new(12.5, egui::FontFamily::Proportional), k.fg);
            let count_w = tab.count.map_or(0.0, |c| 8.0 + 7.0 * c.to_string().len() as f32 + 6.0);
            let w = galley.size().x + 26.0 + count_w;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), if menu.is_some() { egui::Sense::click_and_drag() } else { egui::Sense::click() });
            if resp.drag_stopped() { if let Some(pp) = ui.ctx().pointer_latest_pos() { if pp.y > rect.bottom() + 24.0 { dropped = Some(i); } } }
            if let Some(m) = menu.as_deref_mut() { resp.context_menu(|ui| m(i, ui)); }
            let p = ui.painter();
            if tab.active { p.rect_filled(rect, 0.0, tint(k.cy).gamma_multiply(0.7)); }
            else if resp.hovered() { p.rect_filled(rect, 0.0, k.bg2); }
            let line = if tab.active { k.cy } else if tab.open { k.gr } else { Color32::TRANSPARENT };
            p.rect_filled(egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - 2.0), egui::vec2(rect.width(), 2.0)), 0.0, line);
            let col = if tab.active { k.cy } else if tab.count == Some(0) { k.dim } else { k.mute };
            let _ = text;
            p.text(egui::pos2(rect.left() + 13.0, rect.center().y), egui::Align2::LEFT_CENTER, tab.label, egui::FontId::new(12.5, if tab.active { crate::fonts::ui_strong() } else { egui::FontFamily::Proportional }), col);
            if let Some(c) = tab.count {
                let cx = rect.left() + 13.0 + galley.size().x + 8.0;
                let r = egui::Rect::from_min_size(egui::pos2(cx, rect.center().y - 7.0), egui::vec2(7.0 * c.to_string().len() as f32 + 6.0, 14.0));
                p.rect_filled(r, 7.0, k.bg3);
                p.text(r.center(), egui::Align2::CENTER_CENTER, c.to_string(), egui::FontId::monospace(9.5), k.mute);
            }
            let resp = if tab.hint.is_empty() { resp } else { resp.on_hover_text(&tab.hint) };
            if resp.clicked() { clicked = Some(i); }
        }
    });
    (clicked, dropped)
}

/// Werte der Statusleiste
#[derive(Default)]
pub struct Status {
    pub target: Option<String>,
    pub lang: Option<String>,
    pub cursor: Option<(usize, usize)>,
    pub message: String,
    /// Emulator des Ziels: Name und ob das Programm gefunden wurde
    pub emulator: Option<(String, bool)>,
    /// Debug-Sitzung: Text
    pub debug: Option<String>,
    /// Grund, warum der Einzelschritt gesperrt ist (volle Begründung; die Leiste zeigt nur einen Kurzhinweis)
    pub step_block: Option<String>,
    /// Ergebnis des letzten Testlaufs: Text („Tests 13/16“) und ob es Fehlschläge gibt (rot, sonst grün)
    pub tests: Option<(String, bool)>,
}

/// Zeichnet die Statusleiste; liefert, ob die Marke der Tests angeklickt wurde
pub fn status_bar(ui: &mut egui::Ui, s: &Status) -> bool {
    let k = t();
    let mut tests_clicked = false;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        let mono = |text: String, col: Color32| RichText::new(text).monospace().size(11.0).color(col);
        if let Some(tg) = &s.target { ui.label(mono(format!("Ziel: {tg}"), k.cy)); }
        ui.label(mono(assembler_label(), k.mute));
        if let Some((l, c)) = s.cursor { ui.label(mono(format!("Zeile {l}, Spalte {c}"), k.mute)); }
        if let Some(l) = &s.lang { ui.label(mono(l.clone(), k.mute)); }
        if let Some(d) = &s.debug { ui.label(mono(d.clone(), k.am)); }
        if let Some(why) = &s.step_block { ui.label(mono("Einzelschritt gesperrt".into(), k.am)).on_hover_text(why.as_str()); }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            if let Some((name, found)) = &s.emulator {
                ui.label(mono(format!("{name} {}", if *found { "bereit" } else { "fehlt" }), if *found { k.gr } else { k.rd }));
                ui.label(RichText::new("●").size(10.0).color(if *found { k.gr } else { k.rd }));
            }
            ui.label(mono(s.message.clone(), k.mute));
            if let Some((text, failed)) = &s.tests {
                if ui.add(egui::Label::new(mono(text.clone(), if *failed { k.rd } else { k.gr })).sense(egui::Sense::click())).on_hover_text("Tests öffnen").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { tests_clicked = true; }
            }
        });
    });
    tests_clicked
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind { Primary }

/// Hauptknopf in Dialogen und Studios (cyan)
pub fn action_button(ui: &mut egui::Ui, text: &str, kind: Kind, enabled: bool) -> egui::Response {
    let k = t();
    let (fg, fill, stroke) = match kind {
        Kind::Primary => (k.on_cy, k.cy, k.cy),
    };
    ui.add_enabled(enabled, egui::Button::new(RichText::new(text).color(fg)).fill(fill).stroke(egui::Stroke::new(1.0, stroke)))
}

/// Kennfarbe der Dateiart im Projektbaum
pub fn group_color(g: Group) -> Color32 {
    let k = t();
    match g { Group::Asm => k.cy, Group::Basic => k.am, Group::Song | Group::Play => k.gr, Group::Sprite => k.pu, Group::Charset => Color32::from_rgb(0x3a, 0x8f, 0x9a), Group::Screen => Color32::from_rgb(0x48, 0x6c, 0xb4), Group::Tiles => Color32::from_rgb(0xb0, 0x6a, 0x30), Group::Palette => Color32::from_rgb(0xc0, 0x5a, 0x8c), Group::Text => k.mute, Group::Binary => Color32::from_rgb(0x9a, 0x82, 0x5e), Group::Other => k.dim }
}

/// Zeile im Projektbaum: Art-Marke, Name, Cyan-Streifen für die geöffnete Datei, ★ für das Startprogramm
pub fn tree_row(ui: &mut egui::Ui, g: Group, name: &str, active: bool, is_main: bool) -> egui::Response {
    let k = t();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click_and_drag());
    let p = ui.painter();
    if active { p.rect_filled(rect, 0.0, tint(k.cy).gamma_multiply(0.8)); p.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())), 0.0, k.cy); }
    else if resp.hovered() { p.rect_filled(rect, 0.0, k.bg2); }
    let c = group_color(g);
    let badge = egui::Rect::from_min_size(egui::pos2(rect.left() + 8.0, rect.center().y - 7.0), egui::vec2(30.0, 14.0));
    p.rect_filled(badge, 3.0, tint(c));
    p.text(badge.center(), egui::Align2::CENTER_CENTER, g.tag(), egui::FontId::monospace(9.0), c);
    let label = if is_main { format!("★ {name}") } else { name.to_string() };
    p.text(egui::pos2(badge.right() + 8.0, rect.center().y), egui::Align2::LEFT_CENTER, label, egui::FontId::new(12.0, if is_main { crate::fonts::ui_strong() } else { egui::FontFamily::Proportional }), if active { k.fg } else { k.mute });
    resp
}

/// Marke „C64 PAL“ für das Ziel eines Projekts
pub fn target_chip(ui: &mut egui::Ui, system: &str, variant: &str) {
    let txt = if variant.is_empty() || system == "mega65" && variant.is_empty() { crate::recent::short_name(system).to_string() } else { format!("{} {}", crate::recent::short_name(system), variant.to_uppercase()) };
    crate::ui::widgets::tag(ui, &txt, crate::ui::widgets::system_color(system));
}

/// Neuzeichnen nach Start, Größenänderung, Maximieren/Wiederherstellen und Fokuswechsel. Unter Windows blieb das Fenster teils weiß
/// (nach dem Maximieren, einmal auch Sekunden nach dem Start), bis ein Eingabeereignis kam. Deshalb wird nach jeder solchen Änderung
/// eine Weile lang zehnmal je Sekunde neu gezeichnet (nach dem Start länger); danach ruht die Oberfläche wieder, es gibt keine Dauerlast.
#[derive(Default)]
pub struct RepaintGuard { size: egui::Vec2, maximized: bool, focused: bool, started: bool, until: f64 }

/// So lange wird nach dem Start nachgezeichnet (Sekunden)
const START_SECS: f64 = 20.0;
/// … und nach einer Änderung von Größe, Zustand oder Fokus
const CHANGE_SECS: f64 = 5.0;
/// Abstand der Nachzeichnungen
pub const REPAINT_EVERY: std::time::Duration = std::time::Duration::from_millis(100);

impl RepaintGuard {
    /// Meldet den Fensterzustand zur Zeit `now` (Sekunden); `true`: es muss (weiter) neu gezeichnet werden
    pub fn observe(&mut self, now: f64, size: egui::Vec2, maximized: bool, focused: bool) -> bool {
        let secs = if !self.started { START_SECS } else if size != self.size || maximized != self.maximized || focused != self.focused { CHANGE_SECS } else { 0.0 };
        if secs > 0.0 { self.until = self.until.max(now + secs); }
        self.started = true; self.size = size; self.maximized = maximized; self.focused = focused;
        now < self.until
    }

    pub fn tick(&mut self, ctx: &egui::Context) {
        let (now, size, max, focus) = ctx.input(|i| (i.time, i.viewport().inner_rect.map_or(i.content_rect().size(), |r| r.size()), i.viewport().maximized.unwrap_or(false), i.viewport().focused.unwrap_or(false)));
        if self.observe(now, size, max, focus) { ctx.request_repaint_after(REPAINT_EVERY); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repaints_for_a_while_after_start_and_after_changes_then_rests() {
        let mut g = RepaintGuard::default();
        let s = egui::vec2(1400.0, 860.0);
        assert!(g.observe(0.0, s, false, true), "Start: sofort");
        assert!(g.observe(15.0, s, false, true), "nach dem Start noch eine Weile");
        assert!(!g.observe(21.0, s, false, true), "danach Ruhe, kein Dauer-Neuzeichnen");
        assert!(g.observe(30.0, s, true, true), "Maximieren löst erneut aus");
        assert!(g.observe(34.0, s, true, true));
        assert!(!g.observe(36.0, s, true, true));
        assert!(g.observe(40.0, egui::vec2(1920.0, 1080.0), true, true), "neue Größe");
        assert!(g.observe(50.0, egui::vec2(1920.0, 1080.0), true, false), "Fokuswechsel");
        assert!(!g.observe(56.0, egui::vec2(1920.0, 1080.0), true, false));
    }
}

/// Punkt am rechten Rand einer Zeile des Projektbaums: die Datei hat ungespeicherte Änderungen
pub fn dirty_dot(ui: &egui::Ui, resp: &egui::Response, dirty: bool) {
    if dirty { ui.painter().circle_filled(egui::pos2(resp.rect.right() - 10.0, resp.rect.center().y), 3.5, crate::ui::tokens::t().am); }
}
