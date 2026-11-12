// SPDX-License-Identifier: GPL-3.0-or-later
//! Die festen Zonen unter der Befehlsleiste: Aktivitätsleiste, Seitenleiste, Inspektor, Fußpanel und der Editorbereich.
//! Die Inhalte der Panels sind dieselben wie zuvor (`Viewer::ui`); hier werden sie nur in die Zonen gehängt.
use super::*;
use super::sidebar::SideView;
use crate::ui::icons::{self, Icon};
use crate::ui::tokens::{t, tint};

/// Klick in der Aktivitätsleiste
enum ActivityAct { View(SideView), Reference, Settings }

impl App {
    /// Die sechs Zonen zeichnen. Ohne Projekt entfallen Seitenleiste, Inspektor und Fußpanel; „Maximieren“ blendet sie zeitweise aus.
    /// In schmalen Fenstern (`layout::docking`) stehen Seitenleiste und Inspektor als Streifen da und öffnen als Schublade über dem Editor.
    pub(super) fn show_zones(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.update_context();
        // Invariante: jeder Reiter höchstens einmal (Editorbereiche, Zonen)
        layout::dedupe_tabs(&mut self.dock);
        for z in self.zones.iter_mut() { layout::dedupe_zone(&mut z.side); layout::dedupe_zone(&mut z.inspector); layout::dedupe_zone(&mut z.studio); layout::dedupe_zone(&mut z.footer); }
        let k = t();
        let has_project = self.st.project.is_some();
        let win = ctx.content_rect().size();
        self.side.win_w = win.x;
        self.side.sync_project(self.st.project.as_ref().map(|p| &p.root));
        let dk = layout::docking(win.x);
        let wp = self.layout_wp().clamped(Some(win));
        let hide = self.zen || !has_project;
        let (show_side, show_insp, show_foot) = (wp.side && !hide, wp.inspector && !hide, wp.footer && !hide);
        // angedockt oder als Streifen mit Schublade
        let (side_dock, side_strip) = (show_side && !dk.side_drawer, show_side && dk.side_drawer);
        let (insp_dock, insp_strip) = (show_insp && !dk.inspector_drawer, show_insp && dk.inspector_drawer);
        if !side_strip && self.side.drawer == Some(Zone::Side) || !insp_strip && self.side.drawer == Some(Zone::Inspector) { self.side.drawer = None; }
        let drawer = self.side.drawer;
        // die angedockten Zonen teilen sich nur den Platz, den die Streifen übrig lassen
        let wp_room = layout::Workplace { side: side_dock, inspector: insp_dock, ..wp };
        let (gen, mode) = (self.layout_gen, self.mode_ix());
        let view = self.zones[mode].view;
        let side_open = side_dock || (side_strip && drawer == Some(Zone::Side));
        let tests_failed = self.st.tests.failed_count();
        // Zustand der Aktivitätsleiste vor dem Zeichnen sammeln
        let insp_open = insp_dock || (insp_strip && drawer == Some(Zone::Inspector));
        let active = [side_open && view == SideView::Project, side_open && view == SideView::Search, side_open && view == SideView::Tests, insp_open && self.zones[mode].insp().active_tab() == Some(&Tab::Ref), self.dock.find_tab(&Tab::Settings).is_some()];
        let mut act: Option<ActivityAct> = None;
        let (root, data) = (self.st.project.as_ref().map(|p| p.root.clone()).unwrap_or_default(), self.st.data.clone().unwrap_or_default());
        let split = layout::is_split(&self.dock);
        let App { dock, zones, st, wins, pal, audio, reqs, side, .. } = self;
        let mut viewer = Viewer { split, st, studios: wins, pal, audio, reqs, root, data };
        let (mut side_w, mut insp_w, mut foot_h) = (None, None, None);
        let (mut strip_side, mut strip_insp, mut close_drawer) = (false, false, false);

        layout::activity_panel(ui, |ui| {
            let r = ui.max_rect();
            ui.painter().vline(r.right(), r.y_range(), egui::Stroke::new(1.0, k.line));
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            let items: [(Icon, &str, ActivityAct); 4] = [(Icon::Folder, "Projekt", ActivityAct::View(SideView::Project)), (Icon::Search, "Suche im Projekt (Strg/Cmd+Umschalt+F)", ActivityAct::View(SideView::Search)), (Icon::Flask, "Tests", ActivityAct::View(SideView::Tests)), (Icon::Book, "Referenz", ActivityAct::Reference)];
            ui.add_space(4.0);
            for (i, (icon, tip, a)) in items.into_iter().enumerate() {
                if activity_button(ui, icon, tip, active[i], if i == 2 { tests_failed } else { 0 }).clicked() { act = Some(a); }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                if activity_button(ui, Icon::Gear, "Einstellungen", active[4], 0).clicked() { act = Some(ActivityAct::Settings); }
            });
        });

        if side_dock {
            side_w = Some(layout::side_panel(ui, &wp_room, gen, |ui| sidebar::side_content(ui, &mut viewer, view, &mut side.search)));
        } else if side_strip {
            strip_side = narrow::strip(ui, "zone-side-strip", true, view.title(), drawer == Some(Zone::Side));
        }
        if insp_dock {
            insp_w = Some(layout::inspector_panel(ui, &wp_room, gen, |ui| {
                let r = ui.max_rect();
                ui.painter().vline(r.left(), r.y_range(), egui::Stroke::new(1.0, k.line));
                let sid = format!("insp{mode}{}", if zones[mode].studio_on { "s" } else { "" });
                zone_body(ui, &mut viewer, zones[mode].insp(), &sid);
            }));
        } else if insp_strip {
            strip_insp = narrow::strip(ui, "zone-inspector-strip", false, "Inspektor", drawer == Some(Zone::Inspector));
        }
        if show_foot {
            foot_h = Some(layout::footer_panel(ui, &wp, gen, |ui| {
                let r = ui.max_rect();
                ui.painter().hline(r.x_range(), r.top(), egui::Stroke::new(1.0, k.line));
                zone_body(ui, &mut viewer, &mut zones[mode].footer, &format!("foot{mode}"));
            }));
        }
        // Editorbereich: der einzige Ort mit egui_dock (Reiter und eine Teilung); Schubladen schmaler Fenster liegen darüber
        egui::CentralPanel::default().frame(egui::Frame::NONE).show_inside(ui, |ui| {
            let area = ui.max_rect();
            if layout::areas(dock) == 0 {
                ui.centered_and_justified(|ui| { ui.label(egui::RichText::new(if has_project { "Keine Datei offen.\nDoppelklick im Projektbaum öffnet eine Datei, Strg/Cmd+K sucht." } else { "" }).color(k.dim)); });
            } else {
                let mut style = Style::from_egui(ui.style().as_ref());
                crate::ui::style::dock(&mut style);
                let splits = layout::allowed_splits(dock);
                DockArea::new(dock).style(style).allowed_splits(splits).show_leaf_close_all_buttons(false).show_leaf_collapse_buttons(false).show_inside(ui, &mut viewer);
            }
            let drawer_rect = match drawer {
                Some(Zone::Side) if side_strip => Some(narrow::drawer(ctx, area, true, wp.side_w, "drawer-side", |ui| sidebar::side_content(ui, &mut viewer, view, &mut side.search))),
                Some(Zone::Inspector) if insp_strip => Some(narrow::drawer(ctx, area, false, wp.inspector_w, "drawer-inspector", |ui| {
                    let r = ui.max_rect();
                    ui.painter().vline(r.left(), r.y_range(), egui::Stroke::new(1.0, k.line));
                    zone_body(ui, &mut viewer, &mut zones[mode].inspector, &format!("insp-drawer{mode}"));
                })),
                _ => None,
            };
            if let Some(dr) = drawer_rect {
                // Klick in den Editor daneben oder Esc (ohne fokussiertes Feld) schließt die Schublade
                let pressed_outside = ctx.input(|i| i.pointer.any_pressed()) && ctx.pointer_latest_pos().map_or(false, |p| area.contains(p) && !dr.contains(p));
                let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape)) && ctx.memory(|m| m.focused().is_none());
                if pressed_outside || esc { close_drawer = true; }
            }
        });

        // Eine Datei aus der Schublade geöffnet (Projektbaum, Suchtreffer): die Schublade macht den Platz wieder frei
        let opened = viewer.st.open_request.is_some() || viewer.st.preview_request.is_some() || viewer.st.second_request.is_some();
        let _ = viewer;
        crate::exportdlg::WANT.store(false, std::sync::atomic::Ordering::Relaxed);   // eine Export-Anforderung gilt nur für dieses Bild
        if !hide && layout::adopt(self.wp(), side_w, insp_w, foot_h) { /* gemerkt; save_layout schreibt alle paar Sekunden */ }
        if strip_side { self.side.drawer = if drawer == Some(Zone::Side) { None } else { Some(Zone::Side) }; }
        if strip_insp { self.side.drawer = if drawer == Some(Zone::Inspector) { None } else { Some(Zone::Inspector) }; }
        if close_drawer || (opened && drawer == Some(Zone::Side)) { self.side.drawer = None; }
        match act {
            Some(ActivityAct::View(v)) => {
                let (nv, vis) = sidebar::activity_click(view, side_open, v);
                self.zones[mode].view = nv;
                self.zen = false;
                if vis {
                    self.wp().side = true;
                    if v == SideView::Search { self.side.search.focus = true; }
                    if dk.side_drawer { self.side.drawer = Some(Zone::Side); }
                } else if dk.side_drawer { self.side.drawer = None; } else { self.wp().side = false; }
            }
            Some(ActivityAct::Reference) => self.reveal(Tab::Ref),
            Some(ActivityAct::Settings) => self.open_settings(),
            None => {}
        }
    }
}

/// Symbol der Aktivitätsleiste (46 × 40 px); `badge` > 0 zeigt eine rote Zahl
fn activity_button(ui: &mut egui::Ui, icon: Icon, tip: &str, on: bool, badge: usize) -> egui::Response {
    let k = t();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(layout::ACTIVITY_W, 40.0), egui::Sense::click());
    let p = ui.painter();
    if on { p.rect_filled(rect, 0.0, tint(k.cy).gamma_multiply(0.7)); p.rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())), 0.0, k.cy); }
    else if resp.hovered() { p.rect_filled(rect, 0.0, k.bg2); }
    icons::paint(p, rect.center(), 20.0, icon, if on { k.cy } else if resp.hovered() { k.fg } else { k.mute });
    if badge > 0 {
        let r = egui::Rect::from_center_size(rect.center() + egui::vec2(10.0, -10.0), egui::vec2(14.0, 14.0));
        p.circle_filled(r.center(), 7.0, k.rd);
        p.text(r.center(), egui::Align2::CENTER_CENTER, badge.min(9).to_string(), egui::FontId::monospace(9.0), egui::Color32::WHITE);
    }
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Kopfzeile eines Panels ohne Reiter (26 px)
pub(super) fn panel_header(ui: &mut egui::Ui, title: &str) {
    let k = t();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, k.bg1);
    ui.painter().hline(rect.x_range(), rect.bottom(), egui::Stroke::new(1.0, k.line));
    ui.painter().text(egui::pos2(rect.left() + 12.0, rect.center().y), egui::Align2::LEFT_CENTER, title, egui::FontId::new(11.5, crate::fonts::ui_strong()), k.mute);
}

/// Inspektor und Fußpanel: schlanke Reiterleiste, darunter der Inhalt des aktiven Reiters
fn zone_body(ui: &mut egui::Ui, viewer: &mut Viewer, zt: &mut ZoneTabs<Tab>, id: &str) {
    let k = t();
    ui.spacing_mut().item_spacing.y = 0.0;
    layout::tab_bar(ui, id, zt, |tab| { let mut tt = tab.clone(); viewer.title(&mut tt) }, true);
    let Some(mut tab) = zt.active_tab().cloned() else {
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Keine Panels. „Ansicht ▸ Layout zurücksetzen“ stellt sie wieder her.").small().color(k.dim));
        return;
    };
    egui::Frame::new().inner_margin(egui::Margin::same(6)).show(ui, |ui| {
        egui::ScrollArea::both().id_salt((id, "body")).auto_shrink([false, false]).show(ui, |ui| { ui.spacing_mut().item_spacing.y = 6.0; viewer.ui(ui, &mut tab); });
    });
}
