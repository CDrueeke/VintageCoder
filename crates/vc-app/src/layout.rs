// SPDX-License-Identifier: GPL-3.0-or-later
//! Festes Gerüst der Oberfläche: sechs Zonen (Befehlsleiste, Aktivitätsleiste und Seitenleiste, Editorbereich, Inspektor,
//! Fußpanel, Statusleiste).
//!
//! Dieses Modul kennt keine Panels der Anwendung. Es liefert
//! * die Maße und Grenzen der Zonen und das, was gespeichert wird: **nur Zahlen und Schalter** (`LayoutFile`), nie ein Reiterbaum,
//! * `ZoneTabs` mit einer schlanken Reiterleiste für Seitenleiste, Inspektor und Fußpanel (umsortieren und schließen, nichts verlässt die Zone),
//! * Hilfen für den Editorbereich, der als einziger ein `egui_dock` benutzt und sich genau einmal teilen lässt.
use eframe::egui::{self, WidgetText};
use egui_dock::{DockState, NodeIndex};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---- Maße ---------------------------------------------------------------------------------------------------------------------------

/// Titelzeile (macOS: eigene Zeile mit Ampelknöpfen, sonst die Menüzeile)
pub const TITLE_H: f32 = 32.0;
pub const COMMAND_H: f32 = 46.0;
pub const ACTIVITY_W: f32 = 46.0;
pub const STATUS_H: f32 = 24.0;
/// Der Editorbereich behält mindestens diese Breite
pub const EDITOR_MIN_W: f32 = 480.0;
/// Standard und Grenzen der verstellbaren Zonen
pub const SIDE_W: Range = Range { def: 260.0, min: 180.0, max: 420.0 };
pub const INSPECTOR_W: Range = Range { def: 300.0, min: 240.0, max: 480.0 };
pub const FOOTER_H: Range = Range { def: 220.0, min: 120.0, max: f32::INFINITY };
/// Das Fußpanel darf höchstens so viel der Fensterhöhe belegen
pub const FOOTER_MAX_FRAC: f32 = 0.6;

#[derive(Clone, Copy)]
pub struct Range { pub def: f32, pub min: f32, pub max: f32 }

// ---- Gespeicherte Größen und Schalter ---------------------------------------------------------------------------------------------

/// Größen und Sichtbarkeit der Zonen eines Arbeitsplatzes. Nur Zahlen und Schalter.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workplace {
    pub side_w: f32, pub inspector_w: f32, pub footer_h: f32,
    pub side: bool, pub inspector: bool, pub footer: bool,
}

impl Default for Workplace {
    fn default() -> Self { Workplace { side_w: SIDE_W.def, inspector_w: INSPECTOR_W.def, footer_h: FOOTER_H.def, side: true, inspector: true, footer: true } }
}

fn finite_or(v: f32, def: f32) -> f32 { if v.is_finite() { v } else { def } }

// ---- Breitenklassen (L5): welche Zonen angedockt bleiben und welche zur Schublade werden ---------------------------------------------

/// Ab dieser Fensterbreite ist alles angedockt
pub const WIDE_W: f32 = 1280.0;
/// Ab dieser Breite bleibt die Seitenleiste angedockt, darunter wird auch sie zur Schublade
pub const MEDIUM_W: f32 = 1000.0;
/// Breite des Streifens, den eine eingeklappte Zone übrig lässt (senkrechte Beschriftung)
pub const STRIP_W: f32 = 22.0;

/// Welche Zonen bei einer Fensterbreite als Schublade statt angedockt erscheinen
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Docking { pub side_drawer: bool, pub inspector_drawer: bool }

/// ab 1280 px alles angedockt; 1000 bis 1279 px Inspektor als Schublade; darunter auch die Seitenleiste
pub fn docking(win_w: f32) -> Docking { Docking { side_drawer: win_w < MEDIUM_W, inspector_drawer: win_w < WIDE_W } }

#[cfg(test)]
/// Breite, die ein angedocktes Gerüst bei `win_w` für Aktivitätsleiste, Streifen und Zonen belegt (ohne Editorbereich)
pub fn docked_width(win_w: f32, wp: &Workplace, hide: bool) -> f32 {
    let d = docking(win_w);
    let zone = |on: bool, drawer: bool, w: f32| if hide || !on { 0.0 } else if drawer { STRIP_W } else { w };
    ACTIVITY_W + zone(wp.side, d.side_drawer, wp.side_w) + zone(wp.inspector, d.inspector_drawer, wp.inspector_w)
}

/// Breite einer Schublade: ihre gewohnte Zonenbreite, aber so, dass links und rechts noch ein Rand des Editors bleibt
pub fn drawer_width(zone_w: f32, win_w: f32) -> f32 { zone_w.min((win_w - ACTIVITY_W - STRIP_W - 40.0).max(SIDE_W.min)) }

impl LayoutFile {
    /// Größen und Schalter des Arbeitsplatzes (`debug`: Arbeitsplatz Debug)
    pub fn workplace(&self, debug: bool) -> &Workplace { if debug { &self.debug } else { &self.code } }
    pub fn workplace_mut(&mut self, debug: bool) -> &mut Workplace { if debug { &mut self.debug } else { &mut self.code } }
}

impl Workplace {
    /// Werte in die erlaubten Grenzen bringen (kaputte oder alte Dateien können die Oberfläche nicht verderben).
    /// `win`: Fenstergröße, wenn bekannt; dann gilt auch die Obergrenze des Fußpanels und die Mindestbreite des Editorbereichs.
    pub fn clamped(mut self, win: Option<egui::Vec2>) -> Self {
        self.side_w = finite_or(self.side_w, SIDE_W.def).clamp(SIDE_W.min, SIDE_W.max);
        self.inspector_w = finite_or(self.inspector_w, INSPECTOR_W.def).clamp(INSPECTOR_W.min, INSPECTOR_W.max);
        self.footer_h = finite_or(self.footer_h, FOOTER_H.def).max(FOOTER_H.min);
        if let Some(w) = win {
            self.footer_h = self.footer_h.min((w.y * FOOTER_MAX_FRAC).max(FOOTER_H.min));
            // Schubladen belegen nur ihren Streifen; nur angedockte Zonen verteilen sich den Rest
            let d = docking(w.x);
            let (side_on, insp_on) = (self.side && !d.side_drawer, self.inspector && !d.inspector_drawer);
            let strips = (if self.side && d.side_drawer { STRIP_W } else { 0.0 }) + (if self.inspector && d.inspector_drawer { STRIP_W } else { 0.0 });
            let room = w.x - ACTIVITY_W - EDITOR_MIN_W - strips;
            // ist es eng, gibt zuerst der Inspektor nach, dann die Seitenleiste (nie unter ihre Mindestbreite)
            if side_on && insp_on && self.side_w + self.inspector_w > room {
                self.inspector_w = (room - self.side_w).max(INSPECTOR_W.min).min(self.inspector_w);
                if self.side_w + self.inspector_w > room { self.side_w = (room - self.inspector_w).max(SIDE_W.min).min(self.side_w); }
            } else if side_on && self.side_w > room { self.side_w = room.max(SIDE_W.min).min(self.side_w); }
            else if insp_on && self.inspector_w > room { self.inspector_w = room.max(INSPECTOR_W.min).min(self.inspector_w); }
        }
        self
    }
}

/// Inhalt von `layout.json`: je Arbeitsplatz die Zonen, dazu die Studios, die der Nutzer als eigenes Fenster bevorzugt.
/// Bewusst ohne Reiter: was in den Zonen steht, bestimmt allein die Anwendung.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutFile {
    pub version: u32,
    pub code: Workplace,
    pub debug: Workplace,
    pub detached_studios: Vec<u8>,
    /// Letzte Lage gelöster Studio-Fenster: Gruppe, x, y, Breite, Höhe
    pub studio_rects: Vec<[f32; 5]>,
}

/// Format der Datei; bei einer anderen Zahl wird die Datei ignoriert
pub const VERSION: u32 = 1;

impl Default for LayoutFile {
    fn default() -> Self { LayoutFile { version: VERSION, code: Workplace::default(), debug: Workplace::default(), detached_studios: vec![], studio_rects: vec![] } }
}

impl LayoutFile {
    pub fn parse(text: &str) -> LayoutFile {
        match serde_json::from_str::<LayoutFile>(text) {
            Ok(l) if l.version == VERSION => LayoutFile { code: l.code.clamped(None), debug: l.debug.clamped(None), ..l },
            _ => LayoutFile::default(),
        }
    }
    pub fn to_json(&self) -> String { serde_json::to_string_pretty(self).unwrap_or_default() }
}

/// Die Datei im Konfigurationsordner. Die alten `layout_code.json` und `layout_debug.json` werden nicht mehr gelesen.
pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("layout.json")) }

pub fn load_from(path: &Path) -> LayoutFile { std::fs::read_to_string(path).map(|t| LayoutFile::parse(&t)).unwrap_or_default() }

pub fn save_to(path: &Path, l: &LayoutFile) -> bool {
    if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
    std::fs::write(path, format!("{}\n", l.to_json())).is_ok()
}

// ---- Reiter der festen Zonen ------------------------------------------------------------------------------------------------------

/// Reiter einer Zone (Seitenleiste, Inspektor, Fußpanel): eine Liste und der aktive Eintrag. Nichts davon wird gespeichert.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneTabs<T> { pub tabs: Vec<T>, pub active: usize }

impl<T: Clone + PartialEq> ZoneTabs<T> {
    pub fn new(tabs: Vec<T>) -> Self { ZoneTabs { tabs, active: 0 } }
    pub fn active_tab(&self) -> Option<&T> { self.tabs.get(self.active) }
    /// Reiter nach vorn holen; fehlt er (wurde geschlossen), wird er hinten wieder angelegt
    pub fn show(&mut self, t: &T) {
        match self.tabs.iter().position(|x| x == t) { Some(i) => self.active = i, None => { self.tabs.push(t.clone()); self.active = self.tabs.len() - 1; } }
    }
    pub fn close(&mut self, i: usize) {
        if i >= self.tabs.len() { return; }
        self.tabs.remove(i);
        if self.active > i || self.active >= self.tabs.len() { self.active = self.active.saturating_sub(1).min(self.tabs.len().saturating_sub(1)); }
    }
    /// Reiter `from` an die Stelle `to` schieben; der aktive Reiter bleibt derselbe
    pub fn move_tab(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to { return; }
        let cur = self.tabs.get(self.active).cloned();
        let t = self.tabs.remove(from); self.tabs.insert(to, t);
        if let Some(c) = cur { if let Some(i) = self.tabs.iter().position(|x| *x == c) { self.active = i; } }
    }
}

/// Schlanke Reiterleiste (26 px). `title` liefert die Beschriftung, `closable` ob ein Reiter geschlossen werden darf.
/// Reiter lassen sich innerhalb der Leiste ziehen (umsortieren), per Mittelklick oder Kontextmenü schließen.
pub fn tab_bar<T: Clone + PartialEq>(ui: &mut egui::Ui, id: &str, zt: &mut ZoneTabs<T>, mut title: impl FnMut(&T) -> WidgetText, closable: bool) {
    let k = crate::ui::tokens::t();
    let (bar, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
    ui.painter().rect_filled(bar, 0.0, k.bg1);
    ui.painter().line_segment([bar.left_bottom(), bar.right_bottom()], egui::Stroke::new(1.0, k.line));
    let mut x = bar.left() + 4.0;
    let mut rects: Vec<egui::Rect> = vec![];
    let mut clicked = None; let mut close = None; let mut drag_end = None; let mut shift: Option<(usize, isize)> = None;
    let n = zt.tabs.len();
    for i in 0..n {
        let wt: WidgetText = title(&zt.tabs[i]);
        let galley = wt.into_galley(ui, Some(egui::TextWrapMode::Extend), f32::INFINITY, egui::TextStyle::Button);
        let w = galley.size().x + 22.0;
        let rect = egui::Rect::from_min_size(egui::pos2(x, bar.top()), egui::vec2(w, bar.height()));
        x += w;
        rects.push(rect);
        if rect.right() > bar.right() { continue; }   // passt nicht mehr hinein: abgeschnitten (Reihenfolge ändern oder Fenster vergrößern)
        let resp = ui.interact(rect, egui::Id::new((id, "tab", i)), egui::Sense::click_and_drag());
        let on = i == zt.active;
        if on { ui.painter().rect_filled(rect, 0.0, crate::ui::tokens::tint(k.cy).gamma_multiply(0.7)); }
        else if resp.hovered() { ui.painter().rect_filled(rect, 0.0, k.bg2); }
        if on { ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - 2.0), egui::vec2(rect.width(), 2.0)), 0.0, k.cy); }
        let col = if on { k.cy } else { k.mute };
        ui.painter().galley(egui::pos2(rect.left() + 11.0, rect.center().y - galley.size().y / 2.0), galley, col);
        if resp.clicked() { clicked = Some(i); }
        if closable && resp.middle_clicked() { close = Some(i); }
        if resp.drag_stopped() { drag_end = Some(i); }
        resp.context_menu(|ui| {
            if ui.add_enabled(i > 0, egui::Button::new("Nach links")).clicked() { shift = Some((i, -1)); ui.close(); }
            if ui.add_enabled(i + 1 < n, egui::Button::new("Nach rechts")).clicked() { shift = Some((i, 1)); ui.close(); }
            if closable { ui.separator(); if ui.button("Schließen").clicked() { close = Some(i); ui.close(); } }
        });
    }
    if let Some(i) = clicked { zt.active = i; }
    if let (Some(from), Some(p)) = (drag_end, ui.ctx().pointer_latest_pos()) {
        if let Some(to) = target_index(&rects, p.x) { zt.move_tab(from, to); }
    }
    if let Some((i, d)) = shift { let to = (i as isize + d).clamp(0, n as isize - 1) as usize; zt.move_tab(i, to); }
    if let Some(i) = close { zt.close(i); }
}

/// Auf welche Stelle zeigt `x`? Links vor allen Reitern: erste, rechts dahinter: letzte.
pub fn target_index(rects: &[egui::Rect], x: f32) -> Option<usize> {
    if rects.is_empty() { return None; }
    if x < rects[0].left() { return Some(0); }
    rects.iter().position(|r| x >= r.left() && x < r.right()).or(Some(rects.len() - 1))
}

// ---- Zonen als egui-Panels --------------------------------------------------------------------------------------------------------

/// Rahmen eines Zonenpanels: Grundfläche, ein Trennstrich zum Editorbereich
fn frame() -> egui::Frame { egui::Frame::new().fill(crate::ui::tokens::t().bg1).inner_margin(egui::Margin::ZERO) }

/// Obere Zeilen und Statusleiste sind feste Panels der Anwendung; hier die verstellbaren Zonen. Die Reihenfolge der Aufrufe
/// ist die der egui-Panels: Aktivitätsleiste, Seitenleiste, Inspektor, Fußpanel, danach der Editorbereich als CentralPanel.
pub fn activity_panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Panel::left("zone-activity").exact_size(ACTIVITY_W).resizable(false).frame(frame()).show_inside(ui, |ui| add(ui));
}

/// Seitenleiste. Liefert die tatsächliche Breite (zum Merken).
pub fn side_panel(ui: &mut egui::Ui, wp: &Workplace, gen: u32, add: impl FnOnce(&mut egui::Ui)) -> f32 {
    let win = ui.ctx().content_rect().size();
    let room = (win.x - ACTIVITY_W - EDITOR_MIN_W - if wp.inspector { wp.inspector_w } else { 0.0 }).max(SIDE_W.min);
    let r = egui::Panel::left(egui::Id::new(("zone-side", gen))).resizable(true).default_size(wp.side_w).size_range(SIDE_W.min..=SIDE_W.max.min(room)).frame(frame()).show_inside(ui, |ui| add(ui));
    r.response.rect.width()
}

pub fn inspector_panel(ui: &mut egui::Ui, wp: &Workplace, gen: u32, add: impl FnOnce(&mut egui::Ui)) -> f32 {
    let win = ui.ctx().content_rect().size();
    let room = (win.x - ACTIVITY_W - EDITOR_MIN_W - if wp.side { wp.side_w } else { 0.0 }).max(INSPECTOR_W.min);
    let r = egui::Panel::right(egui::Id::new(("zone-inspector", gen))).resizable(true).default_size(wp.inspector_w).size_range(INSPECTOR_W.min..=INSPECTOR_W.max.min(room)).frame(frame()).show_inside(ui, |ui| add(ui));
    r.response.rect.width()
}

pub fn footer_panel(ui: &mut egui::Ui, wp: &Workplace, gen: u32, add: impl FnOnce(&mut egui::Ui)) -> f32 {
    let max = (ui.ctx().content_rect().height() * FOOTER_MAX_FRAC).max(FOOTER_H.min);
    let r = egui::Panel::bottom(egui::Id::new(("zone-footer", gen))).resizable(true).default_size(wp.footer_h).size_range(FOOTER_H.min..=max).frame(frame()).show_inside(ui, |ui| add(ui));
    r.response.rect.height()
}

/// Nach dem Zeichnen: gemessene Größen übernehmen. Liefert, ob sich etwas geändert hat.
pub fn adopt(wp: &mut Workplace, side: Option<f32>, inspector: Option<f32>, footer: Option<f32>) -> bool {
    let before = *wp;
    if let Some(w) = side { wp.side_w = w.clamp(SIDE_W.min, SIDE_W.max); }
    if let Some(w) = inspector { wp.inspector_w = w.clamp(INSPECTOR_W.min, INSPECTOR_W.max); }
    if let Some(h) = footer { wp.footer_h = h.max(FOOTER_H.min); }
    // Rundung der Pixel ausgleichen: Änderungen unter einem halben Pixel zählen nicht
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    if near(before.side_w, wp.side_w) { wp.side_w = before.side_w; }
    if near(before.inspector_w, wp.inspector_w) { wp.inspector_w = before.inspector_w; }
    if near(before.footer_h, wp.footer_h) { wp.footer_h = before.footer_h; }
    *wp != before
}

// ---- Dateien öffnen ----------------------------------------------------------------------------------------------------------------

/// Wie eine Datei geöffnet wird
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Open {
    /// Einfachklick im Baum: Vorschau-Reiter (kursiv), eine weitere Vorschau ersetzt ihn
    Preview,
    /// Doppelklick, Befehle, Sprünge: fester Reiter im aktiven Editorbereich oder Sprung zum vorhandenen
    Fixed,
    /// Cmd/Strg+Klick: im zweiten Editorbereich (wird angelegt)
    Second,
}

/// Was beim Öffnen passiert: welche Vorschau geschlossen wird, welche danach gilt, ob der zweite Bereich gemeint ist
#[derive(Debug, PartialEq, Eq)]
pub struct OpenPlan { pub close: Option<String>, pub preview: Option<String>, pub second: bool }

/// `cur`: aktuelle Vorschau, `cur_dirty`: sie hat ungespeicherte Änderungen (dann wird sie nicht ersetzt), `is_open`: `rel` ist schon offen
pub fn plan_open(cur: Option<&str>, cur_dirty: bool, is_open: bool, rel: &str, how: Open) -> OpenPlan {
    let keep = cur.filter(|c| *c != rel).map(String::from);
    match how {
        Open::Preview if is_open => OpenPlan { close: None, preview: cur.map(String::from), second: false },
        Open::Preview => OpenPlan { close: keep.clone().filter(|_| !cur_dirty), preview: Some(rel.to_string()), second: false },
        Open::Fixed => OpenPlan { close: None, preview: keep, second: false },
        Open::Second => OpenPlan { close: None, preview: keep, second: true },
    }
}

// ---- Editorbereich: höchstens eine Teilung --------------------------------------------------------------------------------------

/// Anzahl der Bereiche (Blätter) im Editorbereich
pub fn areas<T>(dock: &DockState<T>) -> usize { dock.main_surface().iter().filter(|n| n.is_leaf()).count() }

/// Ist der Editorbereich geteilt?
pub fn is_split<T>(dock: &DockState<T>) -> bool { areas(dock) >= 2 }

/// Was `DockArea` an Teilungen anbieten darf: nur solange noch nicht geteilt ist, und nur links/rechts
pub fn allowed_splits<T>(dock: &DockState<T>) -> egui_dock::AllowedSplits { if is_split(dock) { egui_dock::AllowedSplits::None } else { egui_dock::AllowedSplits::LeftRightOnly } }

/// Der zweite Bereich (rechts), falls geteilt
fn second<T>(dock: &DockState<T>) -> Option<egui_dock::NodePath> {
    dock.iter_leaves().map(|(p, _)| p).filter(|p| p.surface == egui_dock::SurfaceIndex::main()).nth(1)
}

/// Reiter in den zweiten Bereich legen; der Bereich wird angelegt, falls nötig. Ein vorhandener Reiter wird verschoben.
pub fn open_in_second<T: Clone + PartialEq>(dock: &mut DockState<T>, tab: T) {
    if let Some(loc) = dock.find_tab(&tab) { dock.remove_tab(loc); }
    match areas(dock) {
        0 => dock.push_to_focused_leaf(tab),
        1 => {
            let node = dock.main_surface().iter().enumerate().find(|(_, n)| n.is_leaf()).map(|(i, _)| NodeIndex(i)).unwrap_or(NodeIndex::root());
            let [_, new] = dock.main_surface_mut().split_right(node, 0.5, vec![tab]);
            dock.main_surface_mut().set_focused_node(new);
        }
        _ => { if let Some(p) = second(dock) { dock.set_focused_node_and_surface(p); } dock.push_to_focused_leaf(tab); }
    }
}

/// Teilung aufheben: alle Reiter des zweiten Bereichs wandern in den ersten
pub fn unsplit<T: Clone + PartialEq>(dock: &mut DockState<T>) {
    let Some(p) = second(dock) else { return };
    let tabs: Vec<T> = dock.leaf(p).map(|l| l.tabs.clone()).unwrap_or_default();
    for t in &tabs { if let Some(loc) = dock.find_tab(t) { dock.remove_tab(loc); } }
    for t in tabs { dock.push_to_focused_leaf(t); }
}

// ---- Jeder Reiter höchstens einmal ---------------------------------------------------------------------------------------------

/// Reiter öffnen oder fokussieren: ein vorhandener Reiter wird nach vorn geholt (im Bereich, in dem er liegt), sonst im aktiven
/// Bereich angelegt. Gibt es ihn schon mehrfach, bleibt nur der erste. Rückgabe: `true`, wenn er neu angelegt wurde.
pub fn show_unique<T: Clone + PartialEq>(dock: &mut DockState<T>, tab: T) -> bool {
    dedupe_tabs(dock);
    match dock.find_tab(&tab) {
        Some(l) => { let _ = dock.set_active_tab(l); dock.set_focused_node_and_surface(egui_dock::NodePath { surface: l.surface, node: l.node }); false }
        None => { dock.push_to_focused_leaf(tab); true }
    }
}

/// Doppelte Reiter entfernen (der erste bleibt, auch über Bereiche und gelöste Oberflächen hinweg). Rückgabe: Anzahl entfernter.
/// Wird jedes Bild aufgerufen und nach dem Laden/Wiederverbinden; ohne Doppelte kostet das nur einen Durchlauf.
pub fn dedupe_tabs<T: Clone + PartialEq>(dock: &mut DockState<T>) -> usize {
    let mut seen: Vec<T> = vec![];
    let mut dups: Vec<egui_dock::TabPath> = vec![];
    for (path, t) in dock.iter_all_tabs() { if seen.contains(t) { dups.push(path); } else { seen.push(t.clone()); } }
    // von hinten entfernen, damit die Indizes der übrigen gültig bleiben
    for path in dups.iter().rev() { dock.remove_tab(*path); }
    dups.len()
}

/// Dasselbe für eine Zonenleiste
pub fn dedupe_zone<T: Clone + PartialEq>(zt: &mut ZoneTabs<T>) {
    let active = zt.tabs.get(zt.active).cloned();
    let mut seen: Vec<T> = vec![];
    zt.tabs.retain(|t| if seen.contains(t) { false } else { seen.push(t.clone()); true });
    zt.active = active.and_then(|a| zt.tabs.iter().position(|x| *x == a)).unwrap_or(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standardwerte_entsprechen_dem_konzept() {
        let w = Workplace::default();
        assert_eq!((w.side_w, w.inspector_w, w.footer_h), (260.0, 300.0, 220.0));
        assert!(w.side && w.inspector && w.footer);
        assert_eq!((COMMAND_H, ACTIVITY_W, STATUS_H, EDITOR_MIN_W), (46.0, 46.0, 24.0, 480.0));
    }

    #[test]
    fn grenzen_halten_auch_bei_kaputten_werten() {
        let w = Workplace { side_w: 9999.0, inspector_w: -5.0, footer_h: f32::NAN, ..Default::default() }.clamped(None);
        assert_eq!((w.side_w, w.inspector_w, w.footer_h), (420.0, 240.0, 220.0));
        let w = Workplace { footer_h: 5000.0, ..Default::default() }.clamped(Some(egui::vec2(1400.0, 800.0)));
        assert!((w.footer_h - 480.0).abs() < 0.01);   // 60 % der Höhe
    }

    #[test]
    fn schmales_fenster_laesst_dem_editor_480_px() {
        // bei jeder Breite: Aktivitätsleiste, Streifen und angedockte Zonen lassen dem Editorbereich mindestens 480 px
        for w in (700..2000).step_by(20) {
            let win = w as f32;
            let wp = Workplace { side_w: 420.0, inspector_w: 480.0, ..Default::default() }.clamped(Some(egui::vec2(win, 800.0)));
            let editor = win - docked_width(win, &wp, false);
            assert!(editor >= EDITOR_MIN_W - 0.01, "{win}: Editor {editor} px, {wp:?}");
            assert!(wp.side_w >= SIDE_W.min && wp.inspector_w >= INSPECTOR_W.min);
        }
        // Seitenleiste und Inspektor angedockt (1280 px): beide behalten ihre Mindestbreite
        let w = Workplace { side_w: 420.0, inspector_w: 480.0, ..Default::default() }.clamped(Some(egui::vec2(1280.0, 800.0)));
        assert!(w.side_w >= SIDE_W.min && w.inspector_w >= INSPECTOR_W.min && 1280.0 - docked_width(1280.0, &w, false) >= EDITOR_MIN_W - 0.01);
    }

    #[test]
    fn breitenklassen_nach_dem_konzept() {
        let d = |w| docking(w);
        assert_eq!(d(1400.0), Docking { side_drawer: false, inspector_drawer: false });
        assert_eq!(d(1280.0), Docking { side_drawer: false, inspector_drawer: false }, "ab 1280 px alles sichtbar");
        assert_eq!(d(1279.0), Docking { side_drawer: false, inspector_drawer: true }, "1000 bis 1279 px: Inspektor als Schublade");
        assert_eq!(d(1000.0), Docking { side_drawer: false, inspector_drawer: true });
        assert_eq!(d(999.0), Docking { side_drawer: true, inspector_drawer: true }, "unter 1000 px: auch die Seitenleiste");
        assert_eq!(d(640.0), Docking { side_drawer: true, inspector_drawer: true });
        // je schmaler, desto mehr Schubladen – nie zurück
        let mut last = (false, false);
        for w in (500..1500).rev().step_by(7) { let x = d(w as f32); assert!(x.side_drawer >= last.0 && x.inspector_drawer >= last.1, "{w}"); last = (x.side_drawer, x.inspector_drawer); }
    }

    #[test]
    fn belegte_breite_beruecksichtigt_streifen_und_ausgeblendete_zonen() {
        let wp = Workplace::default();
        assert_eq!(docked_width(1400.0, &wp, false), ACTIVITY_W + 260.0 + 300.0);
        assert_eq!(docked_width(1100.0, &wp, false), ACTIVITY_W + 260.0 + STRIP_W);
        assert_eq!(docked_width(900.0, &wp, false), ACTIVITY_W + 2.0 * STRIP_W);
        assert_eq!(docked_width(900.0, &Workplace { side: false, ..wp }, false), ACTIVITY_W + STRIP_W, "ausgeblendet: kein Streifen");
        assert_eq!(docked_width(900.0, &wp, true), ACTIVITY_W, "Maximieren blendet alles aus");
        // Schubladen bleiben schmaler als das Fenster
        assert_eq!(drawer_width(300.0, 1100.0), 300.0);
        assert!(drawer_width(420.0, 600.0) <= 600.0 - ACTIVITY_W - STRIP_W - 40.0 + 0.01);
    }

    #[test]
    fn arbeitsplaetze_speichern_groessen_und_sichtbarkeit_getrennt() {
        let mut l = LayoutFile::default();
        { let c = l.workplace_mut(false); c.side_w = 300.0; c.footer = false; }
        { let d = l.workplace_mut(true); d.inspector_w = 400.0; d.side = false; }
        assert_eq!((l.code.side_w, l.code.footer, l.code.inspector_w, l.code.side), (300.0, false, 300.0, true));
        assert_eq!((l.debug.inspector_w, l.debug.side, l.debug.side_w, l.debug.footer), (400.0, false, 260.0, true));
        assert_eq!(l.workplace(false), &l.code);
        assert_eq!(l.workplace(true), &l.debug);
        // über Speichern und Laden bleiben sie getrennt
        let back = LayoutFile::parse(&l.to_json());
        assert_eq!(back, l);
        // ein Wechsel des Arbeitsplatzes ändert nichts an den gespeicherten Werten des anderen
        let before = l.clone();
        let _ = l.workplace(true); let _ = l.workplace(false);
        assert_eq!(l, before);
    }

    #[test]
    fn gespeichert_werden_nur_zahlen_und_schalter() {
        let l = LayoutFile { detached_studios: vec![4, 6], code: Workplace { side: false, side_w: 300.0, ..Default::default() }, ..Default::default() };
        let v: serde_json::Value = serde_json::from_str(&l.to_json()).unwrap();
        fn only_scalars(v: &serde_json::Value) {
            match v {
                serde_json::Value::Number(_) | serde_json::Value::Bool(_) => {}
                serde_json::Value::Object(m) => m.values().for_each(only_scalars),
                serde_json::Value::Array(a) => a.iter().for_each(only_scalars),
                other => panic!("kein Zahlen- oder Schalterwert: {other}"),
            }
        }
        only_scalars(&v);
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert!(!keys.iter().any(|k| k.contains("tab") && *k != "detached_studios" || k.contains("dock")), "{keys:?}");
    }

    #[test]
    fn speichern_und_laden_rundet_ab_und_fehler_ergeben_standard() {
        let dir = std::env::temp_dir().join(format!("vc_layout_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let f = dir.join("layout.json");
        assert_eq!(load_from(&f), LayoutFile::default(), "fehlende Datei: Standard");
        let mut l = LayoutFile::default(); l.debug.footer = false; l.debug.inspector_w = 333.0; l.code.side_w = 200.0;
        assert!(save_to(&f, &l));
        assert_eq!(load_from(&f), l);
        std::fs::write(&f, "kein json").unwrap();
        assert_eq!(load_from(&f), LayoutFile::default(), "kaputte Datei: Standard");
        // altes Format (Reiterbaum der Dock-Zonen) oder falsche Version wird ignoriert
        std::fs::write(&f, r#"{"dock":{"surfaces":[]},"center":{"surface":0,"node":1}}"#).unwrap();
        assert_eq!(load_from(&f), LayoutFile::default());
        std::fs::write(&f, r#"{"version":99,"code":{"side_w":300.0}}"#).unwrap();
        assert_eq!(load_from(&f), LayoutFile::default());
        // fehlende Felder bekommen Standardwerte, Werte außerhalb der Grenzen werden geklemmt
        std::fs::write(&f, r#"{"version":1,"code":{"side_w":1000.0}}"#).unwrap();
        let g = load_from(&f);
        assert_eq!((g.code.side_w, g.code.inspector_w, g.debug), (420.0, 300.0, Workplace::default()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zuruecksetzen_stellt_die_standardgroessen_her() {
        let mut l = LayoutFile::default();
        l.code = Workplace { side_w: 400.0, footer: false, ..Default::default() };
        l.code = Workplace::default();
        assert_eq!(l, LayoutFile::default());
    }

    #[test]
    fn die_alten_layoutdateien_heissen_anders() {
        let f = file().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).unwrap_or_else(|| "layout.json".into());
        assert_eq!(f, "layout.json");
    }

    #[test]
    fn adopt_meldet_nur_echte_aenderungen() {
        let mut w = Workplace::default();
        assert!(!adopt(&mut w, Some(260.2), Some(299.9), Some(220.0)));
        assert!(adopt(&mut w, Some(300.0), None, None) && w.side_w == 300.0);
        assert!(adopt(&mut w, None, None, Some(50.0)) && w.footer_h == FOOTER_H.min);
    }

    #[test]
    fn zonenreiter_zeigen_schliessen_und_umsortieren() {
        let mut z = ZoneTabs::new(vec!["a", "b", "c"]);
        z.show(&"c"); assert_eq!(z.active_tab(), Some(&"c"));
        z.move_tab(2, 0); assert_eq!((z.tabs.clone(), z.active_tab().copied()), (vec!["c", "a", "b"], Some("c")));
        z.close(0); assert_eq!(z.tabs, vec!["a", "b"]); assert!(z.active_tab().is_some());
        z.show(&"c"); assert_eq!(z.tabs, vec!["a", "b", "c"], "ein geschlossener Reiter kommt wieder");
        z.close(5); assert_eq!(z.tabs.len(), 3);
        let mut e = ZoneTabs::new(vec!["x"]); e.close(0); assert!(e.active_tab().is_none()); e.close(0);
    }

    #[test]
    fn ziehziel_in_der_reiterleiste() {
        let r = |a: f32, b: f32| egui::Rect::from_min_max(egui::pos2(a, 0.0), egui::pos2(b, 20.0));
        let v = [r(0.0, 50.0), r(50.0, 120.0), r(120.0, 160.0)];
        assert_eq!(target_index(&v, -10.0), Some(0));
        assert_eq!(target_index(&v, 60.0), Some(1));
        assert_eq!(target_index(&v, 500.0), Some(2));
        assert_eq!(target_index(&[], 5.0), None);
    }

    #[test]
    fn oeffnen_regeln() {
        let p = |cur: Option<&str>, dirty, open, rel, how| plan_open(cur, dirty, open, rel, how);
        // Einfachklick: neue Vorschau ersetzt die alte
        assert_eq!(p(Some("a.asm"), false, false, "b.asm", Open::Preview), OpenPlan { close: Some("a.asm".into()), preview: Some("b.asm".into()), second: false });
        // ... aber nicht, wenn die alte bearbeitet wurde (sie bleibt dann als fester Reiter)
        assert_eq!(p(Some("a.asm"), true, false, "b.asm", Open::Preview).close, None);
        // Einfachklick auf eine schon offene Datei ändert nichts
        assert_eq!(p(Some("a.asm"), false, true, "b.asm", Open::Preview), OpenPlan { close: None, preview: Some("a.asm".into()), second: false });
        // Doppelklick macht die Vorschau fest, andere Vorschau bleibt
        assert_eq!(p(Some("a.asm"), false, true, "a.asm", Open::Fixed).preview, None);
        assert_eq!(p(Some("a.asm"), false, false, "b.asm", Open::Fixed).preview, Some("a.asm".into()));
        // Cmd/Strg+Klick: zweiter Bereich, nie eine Vorschau
        let s = p(None, false, false, "c.asm", Open::Second);
        assert!(s.second && s.preview.is_none() && s.close.is_none());
    }

    // ---- Prototyp: Teilung des Editorbereichs mit egui_dock ----

    #[test]
    fn editorbereich_laesst_sich_genau_einmal_teilen() {
        let mut d = DockState::new(vec!["a.asm", "b.asm"]);
        assert_eq!(areas(&d), 1);
        assert_eq!(allowed_splits(&d), egui_dock::AllowedSplits::LeftRightOnly);
        open_in_second(&mut d, "b.asm");
        assert_eq!(areas(&d), 2);
        assert_eq!(allowed_splits(&d), egui_dock::AllowedSplits::None, "keine dritte Teilung");
        let in_second = |d: &DockState<&str>, t: &str| second(d).map_or(false, |p| d.leaf(p).map_or(false, |l| l.tabs.contains(&t)));
        assert!(in_second(&d, "b.asm") && !in_second(&d, "a.asm"));
        // weitere Dateien mit Cmd+Klick landen ebenfalls rechts, ohne neue Teilung
        open_in_second(&mut d, "c.asm");
        assert_eq!(areas(&d), 2);
        assert!(in_second(&d, "c.asm"));
        // Reiter zwischen den Hälften ziehen = move_tab
        let src = d.find_tab(&"c.asm").unwrap();
        let dst = d.focused_leaf().unwrap();
        let _ = dst;
        let first = d.iter_leaves().map(|(p, _)| p).next().unwrap();
        d.move_tab(src, egui_dock::TabDestination::Node(first, egui_dock::TabInsert::Append));
        assert!(!in_second(&d, "c.asm") && areas(&d) == 2);
        // Teilung aufheben: nichts geht verloren
        unsplit(&mut d);
        assert_eq!(areas(&d), 1);
        let mut all: Vec<&str> = d.iter_all_tabs().map(|(_, t)| *t).collect(); all.sort();
        assert_eq!(all, vec!["a.asm", "b.asm", "c.asm"]);
        assert_eq!(allowed_splits(&d), egui_dock::AllowedSplits::LeftRightOnly);
    }

    #[test]
    fn leerer_editorbereich_nimmt_wieder_reiter_auf() {
        let mut d: DockState<&str> = DockState::new(vec!["x"]);
        let l = d.find_tab(&"x").unwrap(); d.remove_tab(l);
        assert_eq!(areas(&d), 0);
        d.push_to_focused_leaf("y");
        assert_eq!(areas(&d), 1);
        let mut d2: DockState<&str> = DockState::new(vec!["x"]);
        let l = d2.find_tab(&"x").unwrap(); d2.remove_tab(l);
        open_in_second(&mut d2, "z");
        assert_eq!(areas(&d2), 1, "ohne Bereich entsteht erst einmal einer");
    }

    #[test]
    fn jeder_reiter_nur_einmal() {
        // öffnen: vorhandener Reiter wird fokussiert statt verdoppelt
        let mut d = DockState::new(vec!["welcome", "a"]);
        assert!(!show_unique(&mut d, "welcome"));
        assert!(show_unique(&mut d, "b"));
        assert!(!show_unique(&mut d, "b"));
        assert_eq!(d.iter_all_tabs().count(), 3);
        // Duplikate (Laden, Wiederverbinden) werden bereinigt, der erste bleibt
        let mut d = DockState::new(vec!["welcome", "a", "welcome", "a", "settings", "welcome"]);
        assert_eq!(dedupe_tabs(&mut d), 3);
        let all: Vec<&str> = d.iter_all_tabs().map(|(_, t)| *t).collect();
        assert_eq!(all, vec!["welcome", "a", "settings"]);
        assert_eq!(dedupe_tabs(&mut d), 0);
    }

    #[test]
    fn doppelte_reiter_ueber_beide_editorbereiche() {
        let mut d = DockState::new(vec!["a", "b"]);
        open_in_second(&mut d, "c");
        assert_eq!(areas(&d), 2);
        // ein Duplikat „von außen“ im zweiten Bereich (z. B. wieder angedocktes Fenster)
        d.push_to_focused_leaf("a");
        assert_eq!(dedupe_tabs(&mut d), 1);
        assert_eq!(d.iter_all_tabs().filter(|(_, t)| **t == "a").count(), 1);
        // der zweite Bereich verschiebt statt zu verdoppeln
        open_in_second(&mut d, "b");
        assert_eq!(d.iter_all_tabs().filter(|(_, t)| **t == "b").count(), 1);
        // ein Bereich, der nur aus Duplikaten bestand, verschwindet ohne Rest
        let mut d = DockState::new(vec!["a"]);
        open_in_second(&mut d, "x");
        let l = d.find_tab(&"x").unwrap(); d.remove_tab(l);
        d.push_to_focused_leaf("a");
        dedupe_tabs(&mut d);
        assert_eq!(d.iter_all_tabs().count(), 1);
    }

    #[test]
    fn zonenleiste_ohne_doppelte_behaelt_den_aktiven() {
        let mut z = ZoneTabs::new(vec!["x", "y", "x", "z", "y"]);
        z.active = 3; // „z“
        dedupe_zone(&mut z);
        assert_eq!(z.tabs, vec!["x", "y", "z"]);
        assert_eq!(z.active_tab(), Some(&"z"));
    }
}
