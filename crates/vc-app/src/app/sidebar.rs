// SPDX-License-Identifier: GPL-3.0-or-later
//! Seitenleiste: der Inhalt folgt dem Symbol der Aktivitätsleiste (Projekt, Suche, Tests). Das gewählte Symbol wird je Arbeitsplatz gemerkt.
//! Dazu der Zustand der Schubladen schmaler Fenster (`SideUi`) und die Befehle, die Zonen öffnen und schließen.
//!
//! „Referenz“ hat hier keinen eigenen Inhalt: das Symbol öffnet den Reiter im Inspektor, die Seitenleiste bleibt auf dem zuletzt gewählten Eintrag.
use super::zones::panel_header;
use super::*;
use crate::project::Group;
use crate::projsearch::{self, SearchState};
use crate::ui::tokens::{t, tint};
use vc_ide::tests::suite::Only;

/// Was die Seitenleiste zeigt
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum SideView { #[default] Project, Search, Tests }

impl SideView {
    pub fn title(self) -> &'static str { match self { SideView::Project => "Projekt", SideView::Search => "Suche", SideView::Tests => "Tests" } }
}

/// Zustand der Seitenleiste, der nicht zum Arbeitsplatz gehört: Projektsuche, geöffnete Schublade, zuletzt bekannte Fensterbreite
#[derive(Default)]
pub(super) struct SideUi {
    pub search: SearchState,
    /// Eingeklappte Zone (Seitenleiste oder Inspektor), deren Schublade gerade über dem Editor liegt
    pub drawer: Option<Zone>,
    pub win_w: f32,
    root: Option<PathBuf>,
}

impl SideUi {
    /// Beim Projektwechsel die Treffer des alten Projekts verwerfen
    pub fn sync_project(&mut self, root: Option<&PathBuf>) {
        if self.root.as_ref() != root {
            self.root = root.cloned();
            self.search.cancel();
            self.search.hits.clear(); self.search.for_query = None; self.search.summary = Default::default();
        }
    }
}

/// Klick auf ein Symbol der Aktivitätsleiste: (neuer Inhalt, Seitenleiste sichtbar). Das Symbol des gezeigten Inhalts blendet die
/// Seitenleiste aus, jedes andere zeigt sie mit dem neuen Inhalt.
pub(super) fn activity_click(cur: SideView, visible: bool, clicked: SideView) -> (SideView, bool) {
    if clicked == cur && visible { (cur, false) } else { (clicked, true) }
}

/// Kopf der Seitenleiste und Inhalt (für die angedockte Seitenleiste und ihre Schublade)
pub(super) fn side_content(ui: &mut egui::Ui, viewer: &mut Viewer, view: SideView, search: &mut SearchState) {
    let k = t();
    let r = ui.max_rect();
    ui.painter().vline(r.right(), r.y_range(), egui::Stroke::new(1.0, k.line));
    match view {
        SideView::Project => {
            project_header(ui, viewer.st);
            egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| { let mut tab = Tab::Project; viewer.ui(ui, &mut tab); });
        }
        SideView::Search => {
            panel_header(ui, "Suche");
            egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 6)).show(ui, |ui| viewer.st.side_search_ui(ui, search));
        }
        SideView::Tests => {
            panel_header(ui, "Tests");
            egui::Frame::new().inner_margin(egui::Margin::symmetric(8, 6)).show(ui, |ui| viewer.st.side_tests_ui(ui));
        }
    }
}

/// Kopfzeile „Projekt“ mit dem Menü „+“ (neue Dateien aller Arten) und dem Menü „⋯“
fn project_header(ui: &mut egui::Ui, st: &mut State) {
    let k = t();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, k.bg1);
    ui.painter().hline(rect.x_range(), rect.bottom(), egui::Stroke::new(1.0, k.line));
    ui.painter().text(egui::pos2(rect.left() + 12.0, rect.center().y), egui::Align2::LEFT_CENTER, "Projekt", egui::FontId::new(11.5, crate::fonts::ui_strong()), k.mute);
    let mut row = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(egui::vec2(4.0, 0.0))).layout(egui::Layout::right_to_left(egui::Align::Center)));
    row.spacing_mut().item_spacing.x = 2.0;
    row.spacing_mut().button_padding = egui::vec2(6.0, 1.0);
    row.menu_button(egui::RichText::new("…").strong().color(k.mute), |ui| {
        if ui.button("Projekt-Einstellungen …").clicked() { st.open_projed = true; ui.close(); }
        if ui.button("Diskettenimage öffnen …").on_hover_text("D64, D71, D81, ATR öffnen und Dateien ins Projekt übernehmen").clicked() { st.pick_image_request = true; ui.close(); }
        ui.separator();
        if ui.button("Dateien neu ordnen …").on_hover_text("Dateien nach der Projektstruktur (layout, Regeln, Kategorien) umsortieren; Pfade werden nachgeführt").clicked() { st.reorg_request = true; ui.close(); }
        if ui.button("Neu einlesen").clicked() { if let Some(pr) = &mut st.project { pr.rescan(); } ui.close(); }
        if ui.button("Im Dateimanager zeigen").clicked() { st.tree_act = Some(TreeAct::Reveal(String::new())); ui.close(); }
    }).response.on_hover_text("Weitere Befehle");
    row.menu_button(egui::RichText::new("+").strong().size(15.0).color(k.cy), |ui| {
        if ui.button("Quelltext …").on_hover_text("Neue .asm- oder .bas-Datei").clicked() { st.tree_act = Some(TreeAct::NewFile(String::new())); ui.close(); }
        if ui.button("Ordner …").on_hover_text("Neuen Ordner anlegen (auch verschachtelt: res/bin/fonts)").clicked() { st.tree_act = Some(TreeAct::NewFolder(String::new())); ui.close(); }
        let media = crate::studios::any_media();
        if media { ui.separator(); }
        for (g, label) in [(Group::Sprite, "Sprite …"), (Group::Charset, "Zeichensatz …"), (Group::Screen, "Bildschirm …"), (Group::Tiles, "Kacheln …"), (Group::Palette, "Palette …"), (Group::Song, "Song …"), (Group::Play, "PLAY …")] {
            if crate::studios::group_enabled(g) && ui.button(label).clicked() { st.tree_act = Some(TreeAct::NewMedia(g)); ui.close(); }
        }
        ui.separator();
        if ui.button("Test …").on_hover_text("Neue Testdatei (.vctest)").clicked() { st.tree_act = Some(TreeAct::NewTest); ui.close(); }
    }).response.on_hover_text("Neue Datei anlegen");
}

/// Gerüst einer neuen Testdatei: Ziel und Programm des Projekts, eine statische Prüfung, noch keine Emulator-Tests
pub(super) fn test_template_json(target: &str, program: &str) -> String {
    let v = serde_json::json!({ "kind": "test", "target": target, "program": program, "static": [{ "check": "overlap" }], "tests": [] });
    format!("{}\n", serde_json::to_string_pretty(&v).unwrap_or_default())
}

impl App {
    /// Inhalt einer neuen Testdatei aus dem Projekt (erstes Ziel, Startprogramm)
    pub(super) fn test_template(&self) -> String {
        let pr = self.st.project.as_ref();
        let target = pr.and_then(|p| p.systems.first().cloned()).unwrap_or_else(|| vc_target::default_target_or_c64(self.st.data.as_deref()));
        let stem = pr.and_then(|p| p.pf.as_ref().and_then(|pf| pf.main_file()).map(|m| vc_media::naming::stem(&m).to_string())).unwrap_or_else(|| "main".into());
        test_template_json(&target, &format!("build/{stem}.prg"))
    }

    /// „In Projekt suchen“ (Cmd/Strg+Umschalt+F): Seitenleiste zeigen, Suche wählen, Eingabefeld fokussieren
    pub(super) fn show_project_search(&mut self) {
        self.zen = false;
        self.zones_mut().view = SideView::Search;
        self.wp().side = true;
        self.side.search.focus = true;
        self.open_drawer_if_narrow(Zone::Side);
    }

    /// Ist die Zone beim zuletzt gesehenen Fenster eingeklappt (Streifen statt angedockt)?
    pub(super) fn zone_is_drawer(&self, z: Zone) -> bool {
        if self.side.win_w <= 0.0 { return false; }
        let d = layout::docking(self.side.win_w);
        match z { Zone::Side => d.side_drawer, Zone::Inspector => d.inspector_drawer, Zone::Footer => false }
    }

    /// Eine eingeklappte Zone als Schublade über den Editor legen (angedockte Zonen bleiben, wie sie sind)
    pub(super) fn open_drawer_if_narrow(&mut self, z: Zone) { if self.zone_is_drawer(z) { self.side.drawer = Some(z); } }

    /// Menü „Ansicht“ und Tastenkürzel: angedockt blendet die Zone ein oder aus, als Streifen öffnet und schließt es die Schublade
    pub(super) fn toggle_zone(&mut self, ctx: &egui::Context, z: Zone) {
        self.zen = false;
        self.side.win_w = ctx.content_rect().width();
        let on = { let w = self.wp(); match z { Zone::Side => w.side, Zone::Inspector => w.inspector, Zone::Footer => w.footer } };
        if self.zone_is_drawer(z) && on {
            self.side.drawer = if self.side.drawer == Some(z) { None } else { Some(z) };
        } else {
            { let w = self.wp(); match z { Zone::Side => w.side = !on, Zone::Inspector => w.inspector = !on, Zone::Footer => w.footer = !on } }
            if !on { self.open_drawer_if_narrow(z); } else if self.side.drawer == Some(z) { self.side.drawer = None; }
        }
    }
}

impl State {
    /// Was die Projektsuche durchsucht: Textdateien des Projekts, offene ungespeicherte Dokumente mit ihrem aktuellen Text
    fn search_params(&self, ss: &SearchState) -> Option<projsearch::Params> {
        let pr = self.project.as_ref()?;
        let entries: Vec<(String, Group)> = pr.entries.iter().map(|e| (e.rel.clone(), e.group)).collect();
        let overrides = self.docs.iter().filter(|d| d.dirty()).map(|d| (d.rel.clone(), d.text.clone())).collect();
        Some(projsearch::Params { root: pr.root.clone(), files: projsearch::candidate_files(&entries), overrides, text: ss.text.clone(), case: ss.case, limit: projsearch::MAX_HITS })
    }

    /// Seitenleiste „Suche“: Suchfeld, Groß-/Kleinschreibung, Ergebnisse nach Datei gruppiert; ein Klick springt zur Zeile
    pub(super) fn side_search_ui(&mut self, ui: &mut egui::Ui, ss: &mut SearchState) {
        let k = t();
        ss.poll();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let r = ui.add(egui::TextEdit::singleline(&mut ss.text).hint_text("In Projektdateien suchen …").desired_width((ui.available_width() - 34.0).max(60.0)));
            if ss.focus { r.request_focus(); ss.focus = false; }
            if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) { if ss.running { ss.cancel(); } else { ss.text.clear(); } }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { ss.for_query = None; r.request_focus(); }   // Eingabe: noch einmal suchen
            let aa = egui::Button::new(egui::RichText::new("Aa").monospace().color(if ss.case { k.cy } else { k.mute })).selected(ss.case);
            if ui.add(aa).on_hover_text("Groß-/Kleinschreibung beachten").clicked() { ss.case = !ss.case; }
        });
        if ss.stale() { if let Some(p) = self.search_params(ss) { ss.start(p, &self.ctx); } }
        // Zustandszeile
        let groups = projsearch::grouped(&ss.hits);
        let (n, ng) = (ss.hits.len(), groups.len());
        let mut want_cancel = false;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if ss.text.is_empty() { ui.label(egui::RichText::new("Sucht in Quelltext, BASIC und Textdateien des Projekts.").small().color(k.dim)); }
            else if ss.running {
                ui.spinner();
                ui.label(egui::RichText::new(format!("läuft … {n} Treffer")).small().color(k.am));
                if ui.small_button("Abbrechen").clicked() { want_cancel = true; }
            } else if ss.summary.cancelled { ui.label(egui::RichText::new(format!("abgebrochen · {n} Treffer")).small().color(k.dim)); }
            else if n == 0 { ui.label(egui::RichText::new("Keine Treffer.").small().color(k.dim)); }
            else if ss.summary.truncated { ui.label(egui::RichText::new(format!("mehr als {n} Treffer in {ng} Dateien – Suchbegriff eingrenzen")).small().color(k.am)); }
            else { ui.label(egui::RichText::new(format!("{n} Treffer in {ng} Datei{}", if ng == 1 { "" } else { "en" })).small().color(k.dim)); }
        });
        ui.add_space(2.0);
        // Zeilen: Dateikopf (einklappbar) und Treffer
        enum Row<'a> { File(&'a str, usize), Hit(&'a projsearch::Hit) }
        let mut rows: Vec<Row> = vec![];
        for (rel, hs) in &groups {
            rows.push(Row::File(rel, hs.len()));
            if !ss.collapsed.contains(*rel) { for h in hs { rows.push(Row::Hit(h)); } }
        }
        let (mut toggle, mut jump): (Option<String>, Option<(String, usize)>) = (None, None);
        egui::ScrollArea::vertical().id_salt("side_search").auto_shrink([false, false]).show_rows(ui, 20.0, rows.len(), |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for i in range {
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), egui::Sense::click());
                if resp.hovered() { ui.painter().rect_filled(rect, 3.0, k.bg2); }
                match &rows[i] {
                    Row::File(rel, count) => {
                        let open = !ss.collapsed.contains(*rel);
                        let (dir, name) = rel.rsplit_once('/').map_or(("", *rel), |(d, n)| (d, n));
                        let p = ui.painter_at(rect);
                        p.text(egui::pos2(rect.left() + 8.0, rect.center().y), egui::Align2::LEFT_CENTER, if open { "▾" } else { "▸" }, egui::FontId::proportional(11.0), k.mute);
                        let g = p.layout_no_wrap(name.to_string(), egui::FontId::new(12.0, crate::fonts::ui_strong()), k.fg);
                        let w = g.size().x;
                        p.galley(egui::pos2(rect.left() + 22.0, rect.center().y - g.size().y / 2.0), g, k.fg);
                        if !dir.is_empty() { p.text(egui::pos2(rect.left() + 22.0 + w + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, dir, egui::FontId::proportional(11.0), k.dim); }
                        p.text(egui::pos2(rect.right() - 8.0, rect.center().y), egui::Align2::RIGHT_CENTER, count.to_string(), egui::FontId::monospace(10.5), k.dim);
                        if resp.clicked() { toggle = Some(rel.to_string()); }
                    }
                    Row::Hit(h) => {
                        let p = ui.painter_at(rect);
                        p.text(egui::pos2(rect.left() + 46.0, rect.center().y), egui::Align2::RIGHT_CENTER, h.line.to_string(), egui::FontId::monospace(10.5), k.dim);
                        let font = egui::FontId::monospace(11.5);
                        let (a, m, b) = (h.text.get(..h.start).unwrap_or(""), h.text.get(h.start..h.end).unwrap_or(""), h.text.get(h.end..).unwrap_or(""));
                        let mut job = egui::text::LayoutJob::default();
                        let fmt = |color, bg| egui::text::TextFormat { font_id: font.clone(), color, background: bg, ..Default::default() };
                        if a.is_empty() && m.is_empty() && b.is_empty() { job.append(&h.text, 0.0, fmt(k.mute, egui::Color32::TRANSPARENT)); }
                        else { job.append(a, 0.0, fmt(k.mute, egui::Color32::TRANSPARENT)); job.append(m, 0.0, fmt(k.fg, tint(k.am))); job.append(b, 0.0, fmt(k.mute, egui::Color32::TRANSPARENT)); }
                        job.wrap = egui::text::TextWrapping { max_width: (rect.width() - 56.0).max(20.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
                        let g = p.layout_job(job);
                        p.galley(egui::pos2(rect.left() + 54.0, rect.center().y - g.size().y / 2.0), g, k.mute);
                        if resp.clicked() { jump = Some((h.rel.clone(), h.line)); }
                        resp.on_hover_text(format!("{}:{}", h.rel, h.line));
                    }
                }
            }
        });
        if want_cancel { ss.cancel(); }
        if let Some(r) = toggle { if !ss.collapsed.remove(&r) { ss.collapsed.insert(r); } }
        if let Some((rel, line)) = jump { self.open_request = Some(rel.clone()); if line > 0 { self.pending_reveal = Some((rel, line - 1)); } }
    }

    /// Seitenleiste „Tests“: Tests des Projekts mit letztem Ergebnis, „Alle ausführen“ und je Datei oder Test ein ▶.
    /// Das ausführliche Ergebnis bleibt im Fußpanel „Tests“.
    pub(super) fn side_tests_ui(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let root = self.project.as_ref().map(|p| p.root.clone());
        let can_run = root.is_some() && self.data.is_some();
        let running = self.tests.is_running();
        let mut act: Option<crate::tests_panel::Act> = None;
        let mut show_footer = false;
        let mut select: Option<(PathBuf, String)> = None;
        ui.horizontal(|ui| {
            if running { if ui.button("■ Stopp").clicked() { act = Some(crate::tests_panel::Act::Stop); } }
            else if ui.add_enabled(can_run, egui::Button::new(egui::RichText::new("▶ Alle ausführen").color(k.on_cy)).fill(k.cy)).on_hover_text("statische Prüfungen und Tests im Emulator").clicked() { act = Some(crate::tests_panel::Act::Run(Only::All)); }
            if ui.small_button("Ergebnis …").on_hover_text("Fußpanel „Tests“ mit Erwartung, Registern und Zyklen").clicked() { show_footer = true; }
        });
        let c = self.tests.suite.counts();
        ui.horizontal_wrapped(|ui| {
            if self.tests.suite.is_empty() && !running { ui.label(egui::RichText::new("Noch kein Lauf.").small().color(k.dim)); }
            if c.passed > 0 { crate::ui::widgets::chip(ui, &format!("{} bestanden", c.passed), k.gr); }
            if c.failed > 0 { crate::ui::widgets::chip(ui, &format!("{} fehlgeschlagen", c.failed), k.rd); }
            if c.timed_out > 0 { crate::ui::widgets::chip(ui, &format!("{} Zeitüberschreitung", c.timed_out), k.am); }
            if c.skipped > 0 { crate::ui::widgets::chip(ui, &format!("{} übersprungen", c.skipped), k.dim); }
            if running { ui.spinner(); }
        });
        ui.add_space(2.0);
        // Testdateien, die noch in keinem Lauf vorkamen
        let known: HashSet<String> = match &root { Some(r) => self.tests.suite.groups.iter().filter_map(|g| g.file.as_ref().map(|f| vc_ide::tests::suite::rel_to(r, &f.to_string_lossy()))).collect(), None => HashSet::new() };
        let unrun: Vec<String> = self.project.as_ref().map(|p| p.entries.iter().filter(|e| e.rel.ends_with(".vctest") && !known.contains(&e.rel)).map(|e| e.rel.clone()).collect()).unwrap_or_default();
        if self.tests.suite.groups.is_empty() && unrun.is_empty() {
            ui.label(egui::RichText::new("Keine Tests im Projekt. Im „+“-Menü des Projekts lässt sich eine Testdatei (.vctest) anlegen.").small().color(k.dim));
        }
        egui::ScrollArea::vertical().id_salt("side_tests").auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (gi, g) in self.tests.suite.groups.iter().enumerate() {
                let gc = g.counts();
                let id = ui.make_persistent_id(("side_tests_group", gi, &g.title));
                let state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
                state.show_header(ui, |ui| {
                    let mut t = egui::RichText::new(format!("{} · {}/{}", g.title, gc.passed, g.items.len().max(g.expected))).strong();
                    if gc.failed + gc.timed_out > 0 { t = t.color(k.rd); }
                    ui.label(t);
                    if let Some(f) = &g.file {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add_enabled(can_run && !running, egui::Button::new("▶").small().frame(false)).on_hover_text("Diese Testdatei ausführen").clicked() { act = Some(crate::tests_panel::Act::Run(Only::File(f.clone()))); }
                        });
                    }
                }).body(|ui| {
                    for it in &g.items {
                        let o = &it.outcome;
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), egui::Sense::click());
                        if resp.hovered() { ui.painter().rect_filled(rect, 3.0, k.bg2); }
                        let p = ui.painter_at(rect);
                        crate::tests_panel::dot(&p, egui::pos2(rect.left() + 9.0, rect.center().y), o);
                        let play = (g.file.is_some() && can_run && !running).then(|| egui::Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), egui::vec2(18.0, 18.0)));
                        let right = play.map_or(rect.right(), |r| r.left()) - 4.0;
                        ui.painter_at(egui::Rect::from_min_max(rect.min, egui::pos2(right, rect.max.y))).text(egui::pos2(rect.left() + 20.0, rect.center().y), egui::Align2::LEFT_CENTER, &o.name, egui::FontId::proportional(12.0), crate::theme::label_gray());
                        if let Some(pr) = play {
                            if resp.hovered() || resp.contains_pointer() {
                                let r = ui.interact(pr, ui.make_persistent_id(("side_test_play", &it.file, &o.name)), egui::Sense::click());
                                crate::ui::icons::paint(ui.painter(), pr.center(), 12.0, crate::ui::icons::Icon::Play, if r.hovered() { k.gr } else { k.mute });
                                if r.on_hover_text("Nur diesen Test ausführen").clicked() { act = Some(crate::tests_panel::Act::Run(Only::Test(it.file.clone(), o.name.clone()))); }
                            }
                        }
                        if resp.clicked() { select = Some((it.file.clone(), o.name.clone())); }
                        resp.on_hover_text(format!("{} – {}", o.name, if o.detail.is_empty() { "Ergebnis im Fußpanel „Tests“".to_string() } else { o.detail.clone() }));
                    }
                });
            }
            for rel in &unrun {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(rel).strong());
                    ui.label(egui::RichText::new("noch nicht ausgeführt").small().color(k.dim));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(can_run && !running, egui::Button::new("▶").small().frame(false)).on_hover_text("Diese Testdatei ausführen").clicked() {
                            if let Some(r) = &root { act = Some(crate::tests_panel::Act::Run(Only::File(r.join(rel)))); }
                        }
                    });
                });
            }
        });
        if let Some((f, n)) = select { self.tests.select(f, n); show_footer = true; }
        if show_footer { self.show_tab = Some(Tab::Tests); }
        if act.is_some() { self.tests_act = act; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aktivitaetsleiste_wechselt_den_inhalt_und_blendet_aus() {
        // anderes Symbol: Inhalt wechseln und Seitenleiste zeigen
        assert_eq!(activity_click(SideView::Project, true, SideView::Search), (SideView::Search, true));
        assert_eq!(activity_click(SideView::Project, false, SideView::Tests), (SideView::Tests, true));
        // dasselbe Symbol: ausblenden; war sie schon aus, wieder einblenden
        assert_eq!(activity_click(SideView::Search, true, SideView::Search), (SideView::Search, false));
        assert_eq!(activity_click(SideView::Search, false, SideView::Search), (SideView::Search, true));
    }

    #[test]
    fn gemerkter_inhalt_je_arbeitsplatz() {
        let mut z = [Zones::defaults(Mode::Code), Zones::defaults(Mode::Debug)];
        assert_eq!((z[0].view, z[1].view), (SideView::Project, SideView::Project));
        z[1].view = SideView::Tests;
        assert_eq!((z[0].view, z[1].view), (SideView::Project, SideView::Tests), "der andere Arbeitsplatz behält seine Wahl");
        assert_eq!(SideView::Search.title(), "Suche");
    }

    #[test]
    fn testvorlage_ist_gueltiges_json_mit_ziel_und_programm() {
        let v: serde_json::Value = serde_json::from_str(&test_template_json("c64", "build/main.prg")).unwrap();
        assert_eq!((v["kind"].as_str(), v["target"].as_str(), v["program"].as_str()), (Some("test"), Some("c64"), Some("build/main.prg")));
        assert!(v["tests"].as_array().map_or(false, |a| a.is_empty()));
    }
}
