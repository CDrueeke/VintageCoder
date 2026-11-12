// SPDX-License-Identifier: GPL-3.0-or-later
//! Studios für Grafik und Klang: je Editorart ein Studio mit mehreren Tabs (gleiche Art), standardmäßig als Reiter im Editorbereich,
//! auf Wunsch als eigenes Betriebssystem-Fenster.
use crate::audio::Audio;
use crate::bitmap::{MediaEd, Bitmap, Env, Palettes, Requests};
use crate::palette::PaletteEd;
use crate::screen::ScreenEd;
use crate::screenbank::BankEd;
use crate::tiles::TileEd;
use crate::tracker::{PlayEd, TrackerEd};
use crate::project::Group;
use eframe::egui::{self, ViewportBuilder, ViewportCommand, ViewportId};
use egui_dock::widgets::tab_viewer::OnCloseResponse;
use egui_dock::{AllowedSplits, DockArea, DockState, Style, TabViewer};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Anzahl der gespeicherten Selbsttest-Screenshots (nur `VC_SHOT`)
pub static SHOTS: AtomicUsize = AtomicUsize::new(0);

pub struct MediaDoc { pub rel: String, pub kind: String, pub profile: String, pub name: String, pub json: String, pub bitmap: Option<Box<dyn MediaEd>>, pub note: String }

impl MediaDoc {
    pub fn load(ctx: &egui::Context, root: &std::path::Path, rel: &str, data: &std::path::Path) -> Result<MediaDoc, String> {
        let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("{rel}: ungültiges JSON ({e})"))?;
        let kind = v["kind"].as_str().unwrap_or("").to_string();
        let (mut bitmap, mut note) = (None, String::new());
        let made: Option<Result<Box<dyn MediaEd>, String>> = match kind.as_str() {
            "sprite" | "pmg" | "charset" => Some(Bitmap::open(root, rel, data).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "screen" if vc_media::tiles::is_bank(&v) => Some(BankEd::open(ctx, root, rel, data).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "screen" => Some(ScreenEd::open(ctx, root, rel, data).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "song" => Some(TrackerEd::open(root, rel, data).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "play" => Some(PlayEd::open(root, rel).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "palette" => Some(PaletteEd::open(root, rel).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            "tilemap" | "tileset" => Some(TileEd::open(ctx, root, rel, data).map(|b| Box::new(b) as Box<dyn MediaEd>)),
            _ => None,
        };
        if let Some(r) = made { match r { Ok(b) => bitmap = Some(b), Err(e) => note = e } }
        Ok(MediaDoc { rel: rel.into(), kind, profile: v["profile"].as_str().unwrap_or("").into(), name: v["name"].as_str().unwrap_or("").into(), json: text, bitmap, note })
    }
}

/// Wunsch eines Studios, den Ort zu wechseln (setzen Knöpfe und Menüs, die App führt ihn aus)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move { Embed, Detach }

pub struct MediaWindow { pub group: Group, pub dock: DockState<String>, pub docs: Vec<MediaDoc>, pub closed: bool, pub embedded: bool, pub want: Option<Move> }

/// Studios, die beim Öffnen eingebettet werden sollen (Merkliste der laufenden Sitzung)
/// Studios sind standardmäßig Reiter im Editorbereich; gemerkt werden nur die Arten, die der Nutzer als eigenes Fenster gelöst hat
static DETACHED_PREF: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());
pub fn prefers_embed(g: Group) -> bool { DETACHED_PREF.lock().map_or(true, |v| !v.contains(&(g as u8))) }
pub fn set_prefers_embed(g: Group, on: bool) { if let Ok(mut v) = DETACHED_PREF.lock() { v.retain(|x| *x != g as u8); if !on { v.push(g as u8); } } }
/// Die als Fenster bevorzugten Arten (zum Speichern) und deren Wiederherstellung
pub fn detached_prefs() -> Vec<u8> { DETACHED_PREF.lock().map(|v| v.clone()).unwrap_or_default() }
pub fn set_detached_prefs(v: &[u8]) { if let Ok(mut d) = DETACHED_PREF.lock() { *d = v.to_vec(); } }
/// Letzte Lage gelöster Studio-Fenster (Gruppe, x, y, Breite, Höhe); wird mit `layout.json` gespeichert
static RECTS: std::sync::Mutex<Vec<[f32; 5]>> = std::sync::Mutex::new(Vec::new());
pub fn rects() -> Vec<[f32; 5]> { RECTS.lock().map(|v| v.clone()).unwrap_or_default() }
pub fn set_rects(v: &[[f32; 5]]) { if let Ok(mut r) = RECTS.lock() { *r = v.iter().copied().filter(|a| rect_ok(a)).collect(); } }
/// Eine gemerkte Lage taugt nur, wenn sie endlich, nicht winzig und nicht weit außerhalb jedes üblichen Bildschirms liegt
pub fn rect_ok(a: &[f32; 5]) -> bool { a.iter().all(|x| x.is_finite()) && a[3] >= 240.0 && a[4] >= 160.0 && a[3] <= 8000.0 && a[4] <= 6000.0 && a[1] > -4000.0 && a[1] < 8000.0 && a[2] > -200.0 && a[2] < 6000.0 }
pub fn saved_rect(g: Group) -> Option<[f32; 5]> { RECTS.lock().ok().and_then(|v| v.iter().find(|a| a[0] as u8 == g as u8).copied()).filter(rect_ok) }
pub fn remember_rect(g: Group, pos: [f32; 2], size: [f32; 2]) {
    let a = [g as u8 as f32, pos[0].round(), pos[1].round(), size[0].round(), size[1].round()];
    if !rect_ok(&a) { return; }
    if let Ok(mut v) = RECTS.lock() { match v.iter_mut().find(|r| r[0] == a[0]) { Some(r) => *r = a, None => v.push(a) } }
}
pub fn group_from_u8(n: u8) -> Group { Group::ALL.iter().copied().find(|g| *g as u8 == n).unwrap_or(Group::Other) }

pub fn window_title(g: Group) -> &'static str { crate::studios::def_for_group(g).map_or("Grafik und Klang", |d| d.title) }
/// Nur diese Arten bekommen ein eigenes Fenster (Tabelle `studios::STUDIOS`); ob das Studio im Release ist, fragt `studios::group_enabled`
pub fn is_media_group(g: Group) -> bool { crate::studios::def_for_group(g).is_some() }

struct Viewer<'a> { docs: &'a mut Vec<MediaDoc>, group: Group, env: &'a Env<'a> }

impl TabViewer for Viewer<'_> {
    type Tab = String;
    fn id(&mut self, tab: &mut String) -> egui::Id { egui::Id::new(("mediatab", tab.as_str())) }
    fn title(&mut self, tab: &mut String) -> egui::WidgetText {
        let dirty = self.docs.iter().find(|d| &d.rel == tab).and_then(|d| d.bitmap.as_ref()).map_or(false, |b| b.dirty());
        format!("{}{}", vc_media::naming::stem(tab), if dirty { " *" } else { "" }).into()
    }
    fn closeable(&mut self, _tab: &mut String) -> bool { true }
    fn on_close(&mut self, tab: &mut String) -> OnCloseResponse { self.docs.retain(|d| &d.rel != tab); OnCloseResponse::Close }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut String) {
        let Some(d) = self.docs.iter_mut().find(|d| &d.rel == tab) else { return };
        if let Some(b) = d.bitmap.as_mut() { b.ui(ui, self.env); return; }
        ui.add_space(6.0);
        ui.heading(&d.name);
        ui.label(egui::RichText::new(format!("{} · Profil {}", window_title(self.group), d.profile)).weak());
        ui.add_space(8.0);
        ui.group(|ui| { ui.label(if d.note.is_empty() { "Der Editor für diese Art folgt in einer späteren Etappe.".to_string() } else { d.note.clone() }); });
        ui.add_space(6.0);
        egui::CollapsingHeader::new("Rohdaten (JSON)").show(ui, |ui| {
            egui::ScrollArea::both().max_height(ui.available_height().max(120.0)).show(ui, |ui| { ui.add(egui::Label::new(egui::RichText::new(d.json.chars().take(4000).collect::<String>()).monospace().small()).extend()); });
        });
    }
}

impl MediaWindow {
    pub fn new(group: Group) -> MediaWindow { MediaWindow { group, dock: DockState::new(vec![]), docs: vec![], closed: false, embedded: false, want: None } }
    pub fn id(&self) -> ViewportId { ViewportId::from_hash_of(("media", self.group as u8)) }

    pub fn open(&mut self, doc: MediaDoc) {
        let rel = doc.rel.clone();
        if let Some(loc) = self.dock.find_tab(&rel) { let _ = self.dock.set_active_tab(loc); return; }
        self.docs.push(doc);
        if self.dock.iter_all_tabs().next().is_none() { self.dock = DockState::new(vec![rel]); } else { self.dock.push_to_focused_leaf(rel); }
    }

    /// Zeichnet das Fenster; `shot` = Pfad für den Selbsttest-Screenshot
    pub fn show(&mut self, ctx: &egui::Context, project: &str, shot: Option<&str>, pal: &Palettes, audio: &Audio, reqs: &Requests, root: &std::path::Path, data: &std::path::Path) {
        let env = Env { pal, audio, reqs, root, data };
        if self.embedded { if self.dock.iter_all_tabs().next().is_none() { self.closed = true; } return; }
        let Self { group, dock, docs, closed, want, .. } = self;
        // Tracker brauchen Platz für Kanalköpfe mit Pegel und Oszilloskop, Seitenleiste und Tastatur; Kachel- und Screen-Studio für Palette, Fläche und Eigenschaften nebeneinander
        let size = if matches!(*group, Group::Song | Group::Play | Group::Screen | Group::Tiles) { [1100.0, 760.0] } else { [620.0, 480.0] };
        let mut builder = crate::ui::titlebar::decorate(ViewportBuilder::default().with_title(format!("{} – {}", window_title(*group), project)).with_inner_size(size)).with_min_inner_size(if matches!(*group, Group::Song | Group::Play) { [640.0, 460.0] } else { [480.0, 360.0] });
        if let Some(r) = saved_rect(*group) { builder = builder.with_inner_size([r[3], r[4]]).with_position([r[1], r[2]]); }
        ctx.show_viewport_immediate(ViewportId::from_hash_of(("media", *group as u8)), builder, |ui, _class| {
            if ui.input(|i| i.viewport().close_requested()) { *closed = true; return; }
            if let (Some(o), Some(n)) = ui.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect)) { remember_rect(*group, [o.min.x, o.min.y], [n.width(), n.height()]); }
            if crate::ui::titlebar::active() {
                crate::ui::titlebar::sync_theme(ui.ctx());
                let file = dock.find_active_focused().map(|(_, t)| t.clone()).unwrap_or_default();
                crate::ui::titlebar::bar(ui, &format!("{:?}", group), crate::ui::titlebar::LEFT_INSET, &file, window_title(*group), crate::shell::group_color(*group));
            }
            egui::Panel::top(egui::Id::new(("studio-actions", *group as u8))).exact_size(24.0).frame(egui::Frame::new().fill(crate::ui::tokens::t().bg1).inner_margin(egui::Margin::symmetric(8, 0))).show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.small_button("In Hauptfenster einbetten").on_hover_text("Das Studio als Reiter ins Hauptfenster holen (Fenster, Debugger usw. bleiben daneben sichtbar)").clicked() { *want = Some(Move::Embed); }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| crate::winctl::controls(ui));
                });
            });
            egui::CentralPanel::default().frame(egui::Frame::NONE).show_inside(ui, |ui| dock_ui(dock, docs, *group, &env, ui));
            crate::winctl::edges(ui.ctx());
            if let Some(path) = shot { save_shot(ui, path, &format!("{:?}", group).to_lowercase()); }
        });
        if dock.iter_all_tabs().next().is_none() { *closed = true; }
    }
}

/// Die Tabs eines Studios (gleiche Art) – im eigenen Fenster wie eingebettet im Hauptfenster
fn dock_ui(dock: &mut DockState<String>, docs: &mut Vec<MediaDoc>, group: Group, env: &Env, ui: &mut egui::Ui) {
    let style = Style::from_egui(ui.style().as_ref());
    DockArea::new(dock).id(egui::Id::new(("mediadock", group as u8))).style(style).show_add_buttons(false).allowed_splits(AllowedSplits::None).show_inside(ui, &mut Viewer { docs, group, env });
}

impl MediaWindow {
    /// Zeichnet das Studio eingebettet in einen Bereich des Hauptfensters (mit Knopf zum Lösen)
    pub fn show_embedded(&mut self, ui: &mut egui::Ui, pal: &Palettes, audio: &Audio, reqs: &Requests, root: &std::path::Path, data: &std::path::Path) {
        let env = Env { pal, audio, reqs, root, data };
        ui.horizontal(|ui| {
            if ui.small_button("In eigenem Fenster lösen").on_hover_text("Das Studio wieder als eigenes Betriebssystem-Fenster zeigen").clicked() { self.want = Some(Move::Detach); }
        });
        dock_ui(&mut self.dock, &mut self.docs, self.group, &env, ui);
    }
}

/// Selbsttest: Screenshot dieses Fensters anfordern und als `<Pfad>_<name>.png` speichern (die Antwort kommt in diesem Fenster an)
pub fn save_shot(ui: &egui::Ui, path: &str, name: &str) {
    let ctx = ui.ctx(); let vid = ctx.viewport_id();
    let n = ui.memory_mut(|m| { let k = egui::Id::new(("shot-asked", vid)); let n: u32 = m.data.get_temp(k).unwrap_or(0); m.data.insert_temp(k, n + 1); n });
    let at: u32 = std::env::var("VC_SHOT_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    if n == at { ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())); }
    let imgs: Vec<_> = ctx.input(|i| i.events.iter().filter_map(|e| if let egui::Event::Screenshot { image, .. } = e { Some(image.clone()) } else { None }).collect());
    for img in imgs {
        let (w, h) = (img.width() as u32, img.height() as u32);
        let raw: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
        let file = path.trim_end_matches(".png").to_string() + "_" + name + ".png";
        if let Some(buf) = image::RgbaImage::from_raw(w, h, raw) { let _ = buf.save(&file); }
        SHOTS.fetch_add(1, Ordering::SeqCst);
    }
    ctx.request_repaint();
}

/// Selbsttest: speichert alle eingehenden Screenshots; `names` ordnet Fenster-IDs den Dateinamen zu
pub fn save_shot_root(ctx: &egui::Context, path: &str, names: &[(ViewportId, String)]) {
    let n = ctx.data_mut(|d| { let k = egui::Id::new("root-frames"); let n: u32 = d.get_temp(k).unwrap_or(0); d.insert_temp(k, n + 1); n });
    let at: u32 = std::env::var("VC_SHOT_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    if n == at { ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default())); }
    let imgs: Vec<_> = ctx.input(|i| i.events.iter().filter_map(|e| if let egui::Event::Screenshot { image, viewport_id, .. } = e { Some((*viewport_id, image.clone())) } else { None }).collect());
    for (vid, img) in imgs {
        let name = names.iter().find(|(i, _)| *i == vid).map(|(_, n)| n.clone()).unwrap_or_else(|| "main".into());
        let (w, h) = (img.width() as u32, img.height() as u32);
        let raw: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
        let file = if name == "main" { path.to_string() } else { path.trim_end_matches(".png").to_string() + "_" + &name + ".png" };
        if let Some(buf) = image::RgbaImage::from_raw(w, h, raw) { let _ = buf.save(&file); }
        SHOTS.fetch_add(1, Ordering::SeqCst);
    }
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merkliste_und_gruppen() {
        assert!(prefers_embed(Group::Tiles) && prefers_embed(Group::Palette), "Standard: eingebettet");
        set_prefers_embed(Group::Tiles, false);
        assert!(!prefers_embed(Group::Tiles) && prefers_embed(Group::Palette));
        assert_eq!(detached_prefs(), vec![Group::Tiles as u8]);
        set_prefers_embed(Group::Tiles, true);
        assert!(prefers_embed(Group::Tiles));
        for g in Group::ALL { assert_eq!(group_from_u8(g as u8), g); }
    }

    #[test]
    fn gemerkte_lage_wird_geprueft() {
        assert!(rect_ok(&[2.0, 100.0, 80.0, 800.0, 600.0]));
        assert!(!rect_ok(&[2.0, 100.0, 80.0, 50.0, 600.0]), "zu schmal");
        assert!(!rect_ok(&[2.0, 99999.0, 80.0, 800.0, 600.0]), "weit außerhalb");
        assert!(!rect_ok(&[2.0, f32::NAN, 80.0, 800.0, 600.0]));
        remember_rect(Group::Tiles, [10.0, 20.0], [700.0, 500.0]);
        remember_rect(Group::Tiles, [30.0, 40.0], [710.0, 510.0]);
        assert_eq!(saved_rect(Group::Tiles), Some([Group::Tiles as u8 as f32, 30.0, 40.0, 710.0, 510.0]));
        assert_eq!(rects().iter().filter(|r| r[0] == Group::Tiles as u8 as f32).count(), 1);
    }

    #[test]
    fn neues_studio_ist_ein_fenster_ohne_wunsch() {
        let w = MediaWindow::new(Group::Sprite);
        assert!(!w.embedded && w.want.is_none() && !w.closed);
    }
}
