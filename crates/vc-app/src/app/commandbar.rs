// SPDX-License-Identifier: GPL-3.0-or-later
//! Titelzeile (macOS: Fenstertitel; sonst die Menüzeile) und Befehlsleiste: eine Zeile, drei Gruppen.
//! Links Projekt, Mitte Ausführen (Ziel, Starten, Debuggen, Bauen), rechts Suche, Hinweis „ungespeichert“ und das Segment Code | Debug.
use super::*;
use crate::nativemenu::{self, Cmd as MenuCmd, M};
use crate::ui::icons::{self, Icon};
use crate::ui::tokens::{t, tint};

/// Wie viel von der Befehlsleiste bei der gegebenen Breite sichtbar ist. Bei Platzmangel zuerst die Knopftexte, dann das Suchfeld
/// (nur Symbol), zuletzt das Projekt (nur Name). Nichts bricht in eine zweite Zeile um.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub(super) enum Fit { Full, NoLabels, SearchIcon, Collapsed }

pub(super) fn fit_for_width(w: f32) -> Fit {
    if w >= layout::WIDE_W { Fit::Full } else if w >= layout::MEDIUM_W { Fit::NoLabels } else if w >= 800.0 { Fit::SearchIcon } else { Fit::Collapsed }
}

/// Zustand der Mittelgruppe
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RunUi { Idle, Building, DebugRunning, DebugHalted }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Look { Plain, Run, Debug, Primary, Stop }

/// Knopf der Befehlsleiste: Symbol, optional Text und Kürzel. Ohne Text nur das Symbol (quadratisch).
fn cmd_button(ui: &mut egui::Ui, icon: Icon, label: &str, kbd: &str, look: Look, enabled: bool, tip: &str) -> egui::Response {
    let k = t();
    let (fg, fill, stroke) = match look {
        Look::Plain => (k.fg, k.bg2, k.line),
        Look::Run => (k.gr, tint(k.gr), k.gr.gamma_multiply(0.5)),
        Look::Debug => (k.am, tint(k.am), k.am.gamma_multiply(0.5)),
        Look::Primary => (k.on_cy, k.cy, k.cy),
        Look::Stop => (k.rd, tint(k.rd), k.rd.gamma_multiply(0.5)),
    };
    let dim = if enabled { 1.0 } else { 0.4 };
    let font = egui::FontId::new(13.0, crate::fonts::ui_strong());
    let lg = (!label.is_empty()).then(|| ui.painter().layout_no_wrap(label.to_string(), font, fg));
    let kg = (!label.is_empty() && !kbd.is_empty()).then(|| ui.painter().layout_no_wrap(kbd.to_string(), egui::FontId::monospace(10.0), fg.gamma_multiply(0.7)));
    let w = match &lg { None => 30.0, Some(l) => 12.0 + 16.0 + 6.0 + l.size().x + kg.as_ref().map_or(0.0, |g| 8.0 + g.size().x) + 12.0 };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), if enabled { egui::Sense::click() } else { egui::Sense::hover() });
    let hover = enabled && resp.hovered();
    let p = ui.painter();
    p.rect_filled(rect, 5.0, if hover { fill.gamma_multiply(1.0).lerp_to_gamma(fg, 0.12) } else { fill }.gamma_multiply(dim));
    p.rect_stroke(rect, 5.0, egui::Stroke::new(1.0, stroke.gamma_multiply(dim)), egui::StrokeKind::Inside);
    let mut x = rect.left() + if lg.is_some() { 12.0 } else { 7.0 };
    icons::paint(p, egui::pos2(x + 8.0, rect.center().y), 16.0, icon, fg.gamma_multiply(dim));
    x += 16.0 + 6.0;
    if let Some(l) = lg { let sz = l.size(); p.galley(egui::pos2(x, rect.center().y - sz.y / 2.0), l, fg.gamma_multiply(dim)); x += sz.x + 8.0; }
    if let Some(g) = kg { let sz = g.size(); p.galley(egui::pos2(x, rect.center().y - sz.y / 2.0 + 1.0), g, fg.gamma_multiply(0.7 * dim)); }
    let resp = if enabled && !tip.is_empty() { resp.on_hover_text(tip) } else { resp };
    resp.on_hover_cursor(if enabled { egui::CursorIcon::PointingHand } else { egui::CursorIcon::Default })
}

/// Senkrechter Trenner zwischen den Gruppen
fn group_sep(ui: &mut egui::Ui) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(9.0, 24.0), egui::Sense::hover());
    ui.painter().line_segment([egui::pos2(r.center().x, r.top() + 2.0), egui::pos2(r.center().x, r.bottom() - 2.0)], egui::Stroke::new(1.0, t().line));
}

/// Einträge einer Menütabelle als egui-Menü; liefert den gewählten Befehl
fn menu_items(ui: &mut egui::Ui, items: &[M], ms: &nativemenu::MenuState, mac: bool, recent: &[String]) -> Option<MenuCmd> {
    let mut out = None;
    for m in items {
        match m {
            M::Sep => { ui.separator(); }
            M::Item(c) => {
                let i = nativemenu::info(*c);
                let keys = nativemenu::keys_text(i.keys, mac);
                let b = egui::Button::new(i.label).shortcut_text(keys);
                if ui.add_enabled(nativemenu::enabled(*c, ms), b).clicked() { out = Some(*c); ui.close(); }
            }
            M::Check(c) => {
                let i = nativemenu::info(*c);
                let mut on = nativemenu::checked(*c, ms);
                let keys = nativemenu::keys_text(i.keys, mac);
                let label = if keys.is_empty() { i.label.to_string() } else { format!("{}   ({keys})", i.label) };
                if ui.add_enabled(nativemenu::enabled(*c, ms), egui::Checkbox::new(&mut on, label)).clicked() { out = Some(*c); ui.close(); }
            }
            M::Sub(name, sub) => { ui.menu_button(*name, |ui| { if let Some(c) = menu_items(ui, &nativemenu::visible_items(sub, &vc_ide::features::features()), ms, mac, recent) { out = Some(c); } }); }
            M::Recent => { ui.menu_button("Zuletzt geöffnet", |ui| {
                if recent.is_empty() { ui.add_enabled(false, egui::Button::new("Keine")); }
                for (i, n) in recent.iter().enumerate() { if ui.button(n).clicked() { out = Some(MenuCmd::Recent(i as u8)); ui.close(); } }
            }); }
        }
    }
    out
}

impl App {
    /// Namen der zuletzt geöffneten Projekte für das Menü (ohne fehlende), gleiche Reihenfolge wie `menu_cmd(Recent(i))`
    pub(super) fn recent_names(&self) -> Vec<String> { self.st.welcome.recent.iter().filter(|r| !r.missing).take(10).map(|r| r.name.clone()).collect() }

    /// Titel des Fensters: „Projekt — Datei“, im Debug-Arbeitsplatz mit Zusatz
    fn window_title(&mut self) -> String {
        let Some(pr) = &self.st.project else { return "VintageCoder".into() };
        let name = pr.name.clone();
        let mut s = match self.active_doc() { Some(d) => format!("{name} — {d}"), None => name };
        if self.mode == Mode::Debug { s.push_str(" · Debug"); }
        s
    }

    /// Erste Zeile: unter macOS die eigene Titelzeile (Fenstertitel, Ziehfläche), sonst die Menüzeile mit den Menüs aus `nativemenu::MENUS`.
    /// Die Menüleiste am Bildschirmrand (macOS) kommt aus derselben Tabelle.
    pub(super) fn title_row(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, ms: nativemenu::MenuState, do_open: &mut bool, want_convert: &mut bool) {
        let k = t();
        let title = self.window_title();
        let recent = self.recent_names();
        let mut chosen: Option<MenuCmd> = None;
        egui::Panel::top("titlerow").exact_size(layout::TITLE_H).frame(egui::Frame::new().fill(k.bg1).inner_margin(egui::Margin::symmetric(8, 0))).show_inside(ui, |ui| {
            if crate::ui::titlebar::active() {
                let r = ui.max_rect();
                crate::ui::titlebar::drag_area(ui, r, egui::Id::new("main-titlebar-drag"));
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, title, egui::FontId::new(12.5, egui::FontFamily::Proportional), k.mute);
            } else {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(8.0, 3.0);
                    for (name, items) in &nativemenu::visible_menus(&vc_ide::features::features()) {
                        ui.menu_button(*name, |ui| {
                            if let Some(c) = menu_items(ui, items, &ms, false, &recent) { chosen = Some(c); }
                            if *name == "Fenster" { if let Some(c) = self.window_menu_extras(ctx, ui) { chosen = Some(c); } }
                        });
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { crate::winctl::controls(ui); });
                });
            }
            let r = ui.max_rect();
            ui.painter().hline(r.x_range(), r.bottom(), egui::Stroke::new(1.0, k.line));
        });
        if let Some(c) = chosen { self.menu_cmd(ctx, c, do_open, want_convert); }
    }

    /// Fenster-Menü außerhalb des Macs: Studios einbetten oder lösen, eigene Fensterknöpfe
    fn window_menu_extras(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<MenuCmd> {
        if !self.wins.is_empty() { ui.separator(); ui.label(egui::RichText::new("Studios").weak()); }
        for w in &mut self.wins {
            ui.horizontal(|ui| {
                if ui.button(windows::window_title(w.group)).clicked() { if w.embedded { self.st.show_tab = Some(Tab::Studio(w.group as u8)); } else { ctx.send_viewport_cmd_to(w.id(), egui::ViewportCommand::Focus); } ui.close(); }
                if ui.small_button(if w.embedded { "lösen" } else { "einbetten" }).on_hover_text(if w.embedded { "Als eigenes Fenster zeigen" } else { "Als Reiter ins Hauptfenster holen" }).clicked() { w.want = Some(if w.embedded { windows::Move::Detach } else { windows::Move::Embed }); ui.close(); }
            });
        }
        if !crate::ui::titlebar::active() {
            ui.separator();
            let mut own = crate::winctl::enabled();
            if ui.checkbox(&mut own, "Eigene Fensterknöpfe").on_hover_text("Minimieren, Maximieren, Schließen und Größe ändern vom Programm gezeichnet; für Fenstermanager ohne Titelleiste").changed() { crate::winctl::set_enabled(own); }
        }
        None
    }

    /// Das Auswahlfeld „Auf: C64 PAL → VICE x64sc“: Ziel (System) und Startziel (Emulator oder Maschine) in einem. Rückgabe: (Feldtext, Systemkennung)
    fn run_target_text(&mut self) -> (String, Option<String>) {
        let Some(pr) = &self.st.project else { return ("kein Projekt".into(), None) };
        let sys = if self.st.target == "auto" { pr.systems.first().cloned() } else { Some(self.st.target.clone()) };
        let Some(sys) = sys else { return ("kein Ziel".into(), None) };
        let variant = pr.pf.as_ref().and_then(|pf| pf.targets.iter().find(|t| t.id == sys || t.system == sys)).map(|t| t.variant.to_uppercase()).unwrap_or_default();
        let left = if variant.is_empty() { crate::recent::short_name(&sys).to_string() } else { format!("{} {variant}", crate::recent::short_name(&sys)) };
        let machines = vc_ide::settings::effective().machines;
        let right = match machines.iter().find(|m| m.id == self.st.run_on) {
            Some(m) => if m.name.is_empty() { m.id.clone() } else { m.name.clone() },
            None => self.st.emulator_status(&sys).map(|e| e.0).unwrap_or_else(|| "Emulator".into()),
        };
        (format!("{left} → {right}"), Some(sys))
    }

    pub(super) fn command_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let k = t();
        let has_project = self.st.project.is_some();
        let state = match &self.st.dbg {
            _ if self.st.building.is_some() => RunUi::Building,
            Some(d) => { let sn = d.snapshot(); if sn.running || sn.starting { RunUi::DebugRunning } else { RunUi::DebugHalted } }
            None => RunUi::Idle,
        };
        let idle = self.st.building.is_none() && has_project && self.st.dbg.is_none();
        let dirty = self.st.docs.iter().any(|d| d.dirty());
        let (target_text, _sys) = self.run_target_text();
        let width = ui.available_width();
        let fit = fit_for_width(width);
        let labels = fit == Fit::Full;
        let mac = cfg!(target_os = "macos");
        let mut act: Option<BarAct> = None;
        egui::Panel::top("commandbar").exact_size(layout::COMMAND_H).frame(egui::Frame::new().fill(k.bg1).inner_margin(egui::Margin::symmetric(10, 0))).show_inside(ui, |ui| {
            let bar = ui.max_rect();
            ui.painter().hline(bar.x_range(), bar.bottom(), egui::Stroke::new(1.0, k.line));
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                // ---- links: Projekt ----
                let name = self.st.project.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Kein Projekt".into());
                let recent: Vec<(std::path::PathBuf, String)> = self.st.welcome.recent.iter().filter(|r| !r.missing).take(8).map(|r| (r.path.clone(), r.name.clone())).collect();
                egui::ComboBox::from_id_salt("projectsel").selected_text(egui::RichText::new(name).font(egui::FontId::new(13.0, crate::fonts::ui_strong()))).width(if fit == Fit::Collapsed { 110.0 } else { 150.0 }).show_ui(ui, |ui| {
                    if !recent.is_empty() { ui.label(egui::RichText::new("Zuletzt geöffnet").weak().small()); }
                    for (p, n) in &recent { if ui.selectable_label(false, n).on_hover_text(p.to_string_lossy()).clicked() { act = Some(BarAct::OpenRecent(p.clone())); } }
                    if !recent.is_empty() { ui.separator(); }
                    if ui.button("Neues Projekt …").clicked() { act = Some(BarAct::Menu(MenuCmd::NewProject)); ui.close(); }
                    if ui.button("Projekt öffnen …").clicked() { act = Some(BarAct::Menu(MenuCmd::OpenProject)); ui.close(); }
                    if ui.add_enabled(has_project, egui::Button::new("Projekt-Einstellungen …")).clicked() { act = Some(BarAct::Menu(MenuCmd::ProjectSettings)); ui.close(); }
                });
                if fit != Fit::Collapsed { if let Some(pr) = &self.st.project {
                    let sys = if self.st.target == "auto" { pr.systems.first().cloned() } else { Some(self.st.target.clone()) };
                    if let Some(sys) = sys {
                        let variant = pr.pf.as_ref().and_then(|pf| pf.targets.iter().find(|t| t.id == sys || t.system == sys)).map(|t| t.variant.clone()).unwrap_or_default();
                        crate::shell::target_chip(ui, &sys, &variant);
                    }
                } }
                group_sep(ui);
                // ---- Mitte: Ausführen ----
                let locked = matches!(state, RunUi::DebugRunning | RunUi::DebugHalted);
                ui.add_enabled_ui(has_project && !locked, |ui| {
                    let systems = self.st.project.as_ref().map(|p| p.systems.clone()).unwrap_or_default();
                    let machines = vc_ide::settings::effective().machines;
                    let name_of = |m: &vc_ide::settings::Machine| if m.name.is_empty() { m.id.clone() } else { m.name.clone() };
                    let shown = if labels { format!("Auf: {target_text}") } else { target_text.clone() };
                    egui::ComboBox::from_id_salt("runtarget").selected_text(shown).width(if labels { 268.0 } else { 200.0 }).show_ui(ui, |ui| {
                        ui.label(egui::RichText::new("Ziel").weak().small());
                        ui.selectable_value(&mut self.st.target, "auto".into(), "automatisch");
                        for s in systems { ui.selectable_value(&mut self.st.target, s.clone(), s); }
                        ui.separator();
                        ui.label(egui::RichText::new("Läuft auf").weak().small());
                        if ui.selectable_label(self.st.run_on.is_empty(), "Standard des Systems").on_hover_text("Das Startziel, das in den Einstellungen für das System gewählt ist").clicked() { self.st.run_on.clear(); act = Some(BarAct::SavePrefs); }
                        if ui.selectable_label(self.st.run_on == "@emulator", "Emulator").clicked() { self.st.run_on = "@emulator".into(); act = Some(BarAct::SavePrefs); }
                        for m in &machines { if ui.selectable_label(self.st.run_on == m.id, name_of(m)).clicked() { self.st.run_on = m.id.clone(); act = Some(BarAct::SavePrefs); } }
                        ui.separator();
                        if ui.button("Maschinen und Emulatoren verwalten …").clicked() { act = Some(BarAct::Menu(MenuCmd::Settings)); ui.close(); }
                    });
                });
                // Ziel-Feld und Aktionen gehören zusammen: kein Trenner dazwischen
                match state {
                    RunUi::Idle | RunUi::Building => {
                        let on = idle;
                        if cmd_button(ui, Icon::Play, if labels { "Starten" } else { "" }, if labels { "F5" } else { "" }, Look::Run, on, "Starten (F5)").clicked() { act = Some(BarAct::Menu(MenuCmd::Run)); }
                        let can_dbg = vc_ide::features::features().any_debugger();
                        if cmd_button(ui, Icon::Bug, if labels { "Debuggen" } else { "" }, if labels { "F6" } else { "" }, Look::Debug, on && can_dbg, if can_dbg { "Debuggen (F6)" } else { "In diesem Release ist keine Anbindung mit Debugger enthalten" }).clicked() { act = Some(BarAct::Menu(MenuCmd::Debug)); }
                        if state == RunUi::Building {
                            self.building_button(ui, labels);
                            if cmd_button(ui, Icon::Stop, "", "", Look::Stop, true, "Bau abbrechen (das Ergebnis wird verworfen)").clicked() { act = Some(BarAct::CancelBuild); }
                        } else if cmd_button(ui, Icon::Hammer, if labels { "Bauen" } else { "" }, if labels { "F7" } else { "" }, Look::Primary, idle, "Bauen (F7)").clicked() { act = Some(BarAct::Menu(MenuCmd::Build)); }
                    }
                    RunUi::DebugRunning | RunUi::DebugHalted => {
                        let (sn_pc, can_pause, starting, step_block) = match &self.st.dbg { Some(d) => { let sn = d.snapshot(); (sn.pc, sn.can_pause, sn.starting, sn.step_block.clone()) } None => (0, false, false, None) };
                        let running = state == RunUi::DebugRunning;
                        let chip = if starting { "● startet …".to_string() } else if running { "● läuft".to_string() } else { format!("● angehalten · ${sn_pc:04X}") };
                        crate::ui::widgets::chip(ui, &chip, if running { k.gr } else { k.am });
                        if running {
                            if cmd_button(ui, Icon::Pause, if labels { "Pause" } else { "" }, "", Look::Plain, can_pause, "Anhalten").clicked() { act = Some(BarAct::Menu(MenuCmd::DbgPause)); }
                        } else if cmd_button(ui, Icon::Cont, if labels { "Weiter" } else { "" }, if labels { "F8" } else { "" }, Look::Plain, true, "Weiter (F8)").clicked() { act = Some(BarAct::Menu(MenuCmd::DbgCont)); }
                        let can_step = !running && !starting && step_block.is_none();
                        let why = step_block.clone().unwrap_or_default();
                        if cmd_button(ui, Icon::StepInto, if labels { "Schritt" } else { "" }, if labels { "F11" } else { "" }, Look::Plain, can_step, &format!("Schritt (F11){}", if why.is_empty() { String::new() } else { format!(" – gesperrt: {why}") })).clicked() { act = Some(BarAct::Menu(MenuCmd::DbgStepInto)); }
                        if cmd_button(ui, Icon::StepOver, if labels { "Über" } else { "" }, if labels { "F10" } else { "" }, Look::Plain, can_step, "Über (F10)").clicked() { act = Some(BarAct::Menu(MenuCmd::DbgStepOver)); }
                        if cmd_button(ui, Icon::StepOut, if labels { "Raus" } else { "" }, if labels { "⇧F11" } else { "" }, Look::Plain, can_step, "Raus: bis zum Rücksprung (Umschalt+F11)").clicked() { act = Some(BarAct::Menu(MenuCmd::DbgStepOut)); }
                        // Nächste BASIC-Zeile: nur wenn der Programmzähler in einem konvertierten BASIC-Programm steht
                        let next_bas = if !running && !starting { let maps = self.b2a_maps(); self.st.info.as_ref().and_then(|i| { let sym = |n: &str| i.symbols.get(n).copied(); maps.iter().find_map(|m| m.next_line_addr(&sym, sn_pc)) }) } else { None };
                        if let Some(a) = next_bas { if ui.button("BASIC-Zeile weiter").on_hover_text("Bis zum Anfang der nächsten BASIC-Zeile ausführen (konvertiertes Programm)").clicked() { act = Some(BarAct::NextBasic(a)); } }
                        if cmd_button(ui, Icon::Stop, if labels { "Stop" } else { "" }, "", Look::Stop, true, "Debug-Sitzung beenden").clicked() { act = Some(BarAct::Menu(MenuCmd::DbgStop)); }
                    }
                }
                // ---- rechts: Suche, Hinweis, Segment (von rechts nach links aufgebaut) ----
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    // Segment Code | Debug
                    let mut mode = None;
                    egui::Frame::new().stroke(egui::Stroke::new(1.0, k.line)).corner_radius(5.0).inner_margin(egui::Margin::same(1)).show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        ui.horizontal(|ui| {
                            for (m, text) in [(Mode::Debug, "Debug"), (Mode::Code, "Code")] {   // von rechts nach links: Code steht links
                                let on = self.mode == m;
                                let label = if on { egui::RichText::new(text).strong().color(k.cy) } else { egui::RichText::new(text).color(k.mute) };
                                let b = egui::Button::new(label).fill(if on { tint(k.cy) } else { egui::Color32::TRANSPARENT }).stroke(egui::Stroke::NONE).corner_radius(4.0).min_size(egui::vec2(52.0, 24.0));
                                if ui.add_enabled(has_project || on, b).on_hover_text(match m { Mode::Code => "Arbeitsplatz Code: Gliederung, Memory Map, Bau-Protokoll", Mode::Debug => "Arbeitsplatz Debug: Register, Disassembler, Speicher" }).clicked() { mode = Some(m); }
                            }
                        });
                    });
                    if let Some(m) = mode { act = Some(BarAct::Mode(m)); }
                    // Suche
                    let kbd = if mac { "⌘K" } else { "Strg+K" };
                    if fit >= Fit::SearchIcon {
                        if cmd_button(ui, Icon::Search, "", "", Look::Plain, true, &format!("Befehl oder Datei suchen ({kbd})")).clicked() { act = Some(BarAct::Menu(MenuCmd::Palette)); }
                    } else {
                        let w = if labels { 250.0 } else { 190.0 };
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 28.0), egui::Sense::click());
                        let p = ui.painter();
                        p.rect_filled(rect, 5.0, if resp.hovered() { k.bg3 } else { k.bg0 });
                        p.rect_stroke(rect, 5.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
                        icons::paint(p, egui::pos2(rect.left() + 15.0, rect.center().y), 14.0, Icon::Search, k.dim);
                        p.text(egui::pos2(rect.left() + 30.0, rect.center().y), egui::Align2::LEFT_CENTER, if labels { "Befehl oder Datei suchen …" } else { "Suchen …" }, egui::FontId::proportional(12.5), k.dim);
                        p.text(egui::pos2(rect.right() - 8.0, rect.center().y), egui::Align2::RIGHT_CENTER, kbd, egui::FontId::monospace(10.5), k.dim);
                        if resp.on_hover_cursor(egui::CursorIcon::Text).clicked() { act = Some(BarAct::Menu(MenuCmd::Palette)); }
                    }
                    // Hinweis ungespeichert
                    if dirty { ui.label(egui::RichText::new("● ungespeichert").monospace().size(11.0).color(k.am)).on_hover_text("Speichern: Strg/Cmd+S. Vor dem Bauen wird ohnehin gespeichert."); }
                });
            });
        });
        match act {
            Some(BarAct::Menu(c)) => { let (mut o, mut w) = (false, false); self.menu_cmd(ctx, c, &mut o, &mut w); if o { self.open_project_dialog(); } }
            Some(BarAct::OpenRecent(p)) => self.open_project_path(p),
            Some(BarAct::SavePrefs) => save_prefs(self.st.show_addr, &self.st.run_on),
            Some(BarAct::CancelBuild) => { self.st.building = None; self.st.status = "Bau abgebrochen".into(); self.st.log(Level::Warn, "Bau abgebrochen – ein bereits laufender Übersetzungsschritt endet im Hintergrund, sein Ergebnis wird verworfen"); }
            Some(BarAct::NextBasic(a)) => { if let Some(d) = &self.st.dbg { d.send(DbgCmd::AddPoint { kind: "exec".into(), start: a, end: a, cond: None, ignore: 0 }); d.send(DbgCmd::Cont); self.st.run_to = Some((a, false)); } }
            Some(BarAct::Mode(m)) => self.switch_mode(m),
            None => {}
        }
    }

    /// Der Bauen-Knopf während eines Baus: Text „Baue …“ mit laufendem Balken (der Bau meldet keinen Fortschritt in Prozent)
    fn building_button(&mut self, ui: &mut egui::Ui, labels: bool) {
        let k = t();
        let w = if labels { 150.0 } else { 70.0 };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 28.0), egui::Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, 5.0, k.cy);
        let time = ui.input(|i| i.time) as f32;
        let bar = egui::Rect::from_min_size(egui::pos2(rect.left() + 10.0, rect.bottom() - 7.0), egui::vec2(rect.width() - 20.0, 3.0));
        p.rect_filled(bar, 1.5, k.on_cy.gamma_multiply(0.25));
        let seg = bar.width() * 0.35; let x = bar.left() + (bar.width() - seg) * (0.5 + 0.5 * (time * 3.0).sin());
        p.rect_filled(egui::Rect::from_min_size(egui::pos2(x, bar.top()), egui::vec2(seg, 3.0)), 1.5, k.on_cy);
        p.text(egui::pos2(rect.center().x, rect.top() + 10.0), egui::Align2::CENTER_CENTER, "Baue …", egui::FontId::new(13.0, crate::fonts::ui_strong()), k.on_cy);
        ui.ctx().request_repaint();
    }

    /// Dateidialog „Projekt öffnen“
    pub(super) fn open_project_dialog(&mut self) {
        let mut dlg = rfd::FileDialog::new().add_filter("Retro-Projekt (.vcprj)", &["vcprj"]).set_title("Projekt öffnen");
        if let Some(dir) = self.open_start_dir() { dlg = dlg.set_directory(dir); }
        if let Some(f) = dlg.pick_file() { self.open_project_path(f); }
    }
}

/// Was die Befehlsleiste im letzten Bild ausgelöst hat
enum BarAct { Menu(MenuCmd), OpenRecent(std::path::PathBuf), SavePrefs, CancelBuild, NextBasic(u16), Mode(Mode) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn befehlsleiste_spart_zuerst_texte_dann_suchfeld_dann_projekt() {
        assert_eq!(fit_for_width(1400.0), Fit::Full);
        assert_eq!(fit_for_width(1280.0), Fit::Full);
        assert_eq!(fit_for_width(1279.0), Fit::NoLabels, "1000 bis 1279 px: ohne Knopftexte");
        assert_eq!(fit_for_width(1000.0), Fit::NoLabels);
        assert_eq!(fit_for_width(999.0), Fit::SearchIcon, "unter 1000 px: Suchfeld wird Symbol");
        assert_eq!(fit_for_width(950.0), Fit::SearchIcon);
        assert_eq!(fit_for_width(799.0), Fit::Collapsed);
        // je schmaler, desto weiter die Verdichtung – nie zurück
        let mut last = Fit::Full;
        for w in (600..1500).rev().step_by(10) { let f = fit_for_width(w as f32); assert!(f >= last, "{w}: {f:?} < {last:?}"); last = f; }
    }
}
