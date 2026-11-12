// SPDX-License-Identifier: GPL-3.0-or-later
//! Konfigurationsassistent (Ersteinrichtung und Wiederaufruf): ein Assistent, zwei Wege (Schnelleinrichtung mit 7 Schritten, „Alles selbst
//! einstellen“ mit 12), dazu Teil-Assistenten. Er arbeitet auf einem Entwurf im Speicher (`vc_ide::setup::Draft`), zeigt am Ende die
//! Änderungsliste und schreibt erst bei „Übernehmen“ (Sicherung `settings.json.bak`).
//!
//! Aufbau: `mod.rs` (Zustand, Navigation, Fenster), `rail.rs` (Schritt- und Fortschrittsleiste), `steps/` (ein Abschnitt je Schritt),
//! `side.rs` (Export, Import, Zurücksetzen), `widgets.rs` (Silhouetten, Tastenkappe, Bildschirm). Die Logik steht in `vc_ide::setup`;
//! Emulator-, Hardware-, ROM- und Editor-Seiten kommen aus `settings_ui` (keine zweite Fassung). Der Assistent führt nie Befehle zur
//! Installation aus, lädt nichts herunter und öffnet den Browser nur auf Klick.
use crate::theme::View;
use crate::ui::tokens::t;
use eframe::egui::{self, RichText};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use vc_ide::setup::{self, Applied, Badge, Change, Draft, Finding, Hint, Kind, Mode, Part, Paths, Problem, Snapshot, Topic, WizardState};

mod rail;
mod side;
mod steps;
pub mod widgets;
#[cfg(test)]
mod tests;

/// Was die Anwendung nach einem Bild tun soll
pub enum Out {
    None,
    /// Assistent beendet (Fenster schließen); die Darstellung ist bereits in `env.view` zurückgesetzt, falls nichts übernommen wurde
    Close,
    /// Übernommen: Einstellungen und Darstellung gelten jetzt (Oberfläche neu laden)
    Applied(View),
    /// Rückgängig gemacht
    Undone(View),
    OpenProject(PathBuf),
    NewProject,
    Help(&'static str),
}

/// Umgebung des Assistenten
pub struct Env<'a> { pub view: &'a mut View, pub show_addr: &'a mut bool, pub data: Option<&'a Path> }

/// Wie der Assistent gestartet wird
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Launch { FirstRun, Recall, Part(Part), Export, Import, Reset }

/// Das erste Projekt (Schritt „Projekte“, Vollweg)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum First { Example(String), New, Later }

pub struct SetupWizard {
    pub(crate) mode: Mode,
    pub(crate) steps: Vec<Topic>,
    pub(crate) idx: usize,
    /// weitester erreichter Schritt (Sprung nur zu erreichten)
    pub(crate) reached: usize,
    pub(crate) paths: Paths,
    pub(crate) cur: Snapshot,
    pub(crate) draft: Draft,
    pub(crate) wiz: WizardState,
    pub(crate) skipped: BTreeSet<Topic>,
    pub(crate) recall: bool,
    /// nur ein Dialog (Export/Import/Zurücksetzen), kein Schrittfenster
    pub(crate) tool_only: bool,
    pub(crate) st: crate::settings_ui::state::UiState,
    pub(crate) roms: crate::settings_ui::roms::RomsUi,
    pub(crate) changes: Vec<Change>,
    pub(crate) deselected: BTreeSet<String>,
    pub(crate) findings: Vec<Finding>,
    pub(crate) problems: Vec<Problem>,
    pub(crate) badges: std::collections::BTreeMap<Topic, Badge>,
    pub(crate) status_at: Option<Instant>,
    status_for: Option<Draft>,
    pub(crate) load_error: Option<String>,
    pub(crate) confirm_later: bool,
    pub(crate) apply_error: Option<String>,
    pub(crate) applied: Option<Applied>,
    pub(crate) notes: Vec<String>,
    pub(crate) view_start: View,
    pub(crate) hints: Vec<Hint>,
    pub(crate) tool: String,
    pub(crate) tab_os: String,
    pub(crate) tab_pm: String,
    det: HashMap<String, (Instant, Option<String>)>,
    pub(crate) has_hardware: bool,
    pub(crate) first: First,
    pub(crate) proj_check: Option<(String, Result<setup::ProjectsDirState, String>, Instant)>,
    pub(crate) side: side::Side,
    pub(crate) install_dir: Option<PathBuf>,
    /// Datenordner (nur gelieferte Systeme werden angeboten)
    pub(crate) data_dir: Option<PathBuf>,
    /// Selbsttest (Bilder): nichts schreiben
    pub(crate) shot: bool,
    pub(crate) distro_pm: Option<&'static str>,
    /// Übernommene Einstellungen im Prozess wirksam machen (nur im Programm, nicht in Tests)
    pub(crate) sync_memory: bool,
}

impl SetupWizard {
    pub fn new(launch: Launch, env: &Env) -> Option<SetupWizard> {
        let mut w = SetupWizard::with_paths(launch, env, Paths::current()?);
        w.sync_memory = true;
        Some(w)
    }

    /// Wie [`SetupWizard::new`] mit festen Dateiorten; die Einstellungen im Prozess bleiben unberührt (Tests)
    pub(crate) fn with_paths(launch: Launch, env: &Env, paths: Paths) -> SetupWizard {
        let (cur, load_error) = match setup::load_snapshot(&paths) { Ok(s) => (s, None), Err(e) => (Snapshot { settings: Default::default(), view: setup::ViewPrefs::load(&paths.view), disk_hash: 0 }, Some(e)) };
        let wiz = WizardState::load(&paths.wizard);
        let recall = launch != Launch::FirstRun;
        let mut draft = if recall { Draft::from_snapshot(&cur) } else { Draft::for_first_run(&cur) };
        if !recall && draft.settings.projects_dir.trim().is_empty() { draft.settings.projects_dir = setup::effective_projects_dir(&draft.settings); }
        draft.view = env.view.to_prefs();
        draft.examples.source = env.data.and_then(|d| d.parent()).map(|p| p.join("examples")).filter(|p| p.is_dir());
        draft.examples.copy = draft.examples.source.is_some() && !recall;
        let mode = match launch { Launch::Part(p) => Mode::Part(p), Launch::Recall | Launch::Export | Launch::Import | Launch::Reset => Mode::Full, Launch::FirstRun => Mode::Quick };
        let mut st = crate::settings_ui::state::UiState::new(); st.offline = true;
        let hints = env.data.map(setup::load_hints).unwrap_or_default();
        let distro_pm = if cfg!(target_os = "linux") { std::fs::read_to_string("/etc/os-release").ok().and_then(|t| setup::package_manager_from_os_release(&t)) } else { None };
        let mut w = SetupWizard {
            steps: mode.steps(), mode, idx: 0, reached: 0, paths, cur, draft, wiz, skipped: BTreeSet::new(), recall, tool_only: false, st, roms: Default::default(), changes: vec![], deselected: BTreeSet::new(),
            findings: vec![], problems: vec![], badges: Default::default(), status_at: None, status_for: None, load_error, confirm_later: false, apply_error: None, applied: None, notes: vec![], view_start: *env.view,
            hints, tool: String::new(), tab_os: setup::os_id().into(), tab_pm: distro_pm.unwrap_or("").into(), det: HashMap::new(), has_hardware: false, first: First::Later, proj_check: None, side: side::Side::None,
            sync_memory: false, install_dir: env.data.and_then(|d| d.parent()).map(Path::to_path_buf), data_dir: env.data.map(Path::to_path_buf), shot: std::env::var("VC_SHOT").is_ok(), distro_pm,
        };
        w.has_hardware = !w.draft.settings.machines.is_empty();
        if let Some(s) = w.draft.settings.targets.first() { w.st.system = s.system.clone(); }
        // Teil-Assistenten und Werkzeuge: Welcome entfällt
        match launch {
            Launch::Export => { w.tool_only = true; w.side = side::Side::export_for(&w.draft.settings); }
            Launch::Import => { w.tool_only = true; w.side = side::Side::Import { msg: String::new() }; }
            Launch::Reset => { w.tool_only = true; w.side = side::Side::Reset { area: setup::ResetArea::All, with_hw: false }; }
            _ => {}
        }
        w.refresh_status(true);
        // Selbsttest (Bilder): VC_WIZ_STEP=<Schrittkennung oder Nummer>, VC_WIZ_MODE=quick|full|recall
        if w.shot { w.shot_setup(); }
        w
    }

    fn shot_setup(&mut self) {
        if let Ok(m) = std::env::var("VC_WIZ_MODE") {
            match m.as_str() { "quick" => { self.mode = Mode::Quick; self.recall = false; } "full" => { self.mode = Mode::Full; self.recall = false; } "recall" => { self.mode = Mode::Full; self.recall = true; }
                "roms" => { self.mode = Mode::Part(Part::Roms); self.recall = true; } _ => {} }
            self.steps = self.mode.steps();
        }
        if let Ok(s) = std::env::var("VC_WIZ_STEP") {
            let want = Topic::from_id(&s).or_else(|| s.parse::<usize>().ok().and_then(|n| self.steps.get(n.saturating_sub(1)).copied()));
            if let Some(i) = want.and_then(|t| self.steps.iter().position(|x| *x == t)) { self.idx = i; self.reached = i; }
        }
        if let Ok(a) = std::env::var("VC_WIZ_SYSTEMS") { for s in a.split(',') { setup::set_system(&mut self.draft.settings, s, true); } }
        if std::env::var("VC_WIZ_HW").is_ok() { self.has_hardware = true; }
        if let Ok(k) = std::env::var("VC_WIZ_TOOL") { self.tool = k; }
        self.refresh_status(true);
    }

    pub(crate) fn topic(&self) -> Topic { self.steps.get(self.idx).copied().unwrap_or(Topic::Welcome) }

    /// Fundort einer Emulator-Sorte (Dateisystem und PATH); das Ergebnis gilt drei Sekunden
    pub(crate) fn found(&mut self, kind: &str) -> Option<String> {
        if let Some((t, v)) = self.det.get(kind) { if t.elapsed() < Duration::from_secs(3) { return v.clone(); } }
        let v = vc_ide::platform::discover(kind);
        self.det.insert(kind.into(), (Instant::now(), v.clone()));
        v
    }
    pub(crate) fn forget_found(&mut self) { self.det.clear(); self.st.forget_presence(); }

    /// Systeme des Entwurfs
    pub(crate) fn systems(&self) -> Vec<String> { setup::visible_systems(&self.draft.settings) }

    /// Änderungsliste, Statusmarken und Probleme neu berechnen (bei Änderung des Entwurfs oder alle zwei Sekunden)
    pub(crate) fn refresh_status(&mut self, force: bool) {
        let stale = self.status_at.map_or(true, |t| t.elapsed() > Duration::from_secs(2));
        if !force && !stale && self.status_for.as_ref() == Some(&self.draft) { return; }
        let mut ch = setup::diff(&self.cur, &self.draft);
        for c in &mut ch { if self.deselected.contains(&c.key) { c.selected = false; } }
        if let Mode::Part(p) = self.mode { setup::only_topics(&mut ch, part_topics(p)); }
        self.problems = setup::problems(&self.cur.settings);
        let discover = |k: &str| vc_ide::platform::discover(k);
        self.findings = if self.recall { setup::findings(&self.draft.settings, &self.wiz, &discover) } else { vec![] };
        self.badges = setup::badges(&ch, &self.problems, &self.findings, &self.skipped);
        self.changes = ch;
        self.status_for = Some(self.draft.clone());
        self.status_at = Some(Instant::now());
    }

    pub(crate) fn selected_keys(&self) -> BTreeSet<String> { self.changes.iter().filter(|c| c.selected && c.kind != Kind::Same).map(|c| c.key.clone()).collect() }

    /// Gründe, warum „Weiter“ gesperrt ist (leer = frei)
    pub(crate) fn blocked(&self) -> Option<String> {
        match self.topic() {
            Topic::Systems if self.draft.settings.targets.is_empty() => Some("Ohne System lässt sich nichts bauen: wähle mindestens eines.".into()),
            Topic::Projects => match setup::check_projects_dir(&setup::effective_projects_dir(&self.draft.settings), self.install_dir.as_deref()) { Err(e) if !self.draft.settings.projects_dir.trim().is_empty() => Some(e), _ => None },
            Topic::Summary if self.load_error.is_some() => Some("Die Einstellungsdatei ist nicht lesbar; es wird nichts geschrieben.".into()),
            _ => None,
        }
    }

    /// Themen, die noch offen sind (für „Später fortsetzen“): übersprungene und noch nicht erreichte Schritte des Weges
    pub(crate) fn later_topics(&self) -> Vec<Topic> {
        let mut v: Vec<Topic> = self.steps.iter().enumerate().filter(|(i, t)| (*i > self.idx || self.skipped.contains(t)) && !matches!(t, Topic::Welcome | Topic::Summary | Topic::Done)).map(|(_, t)| *t).collect();
        if let Some(c) = Some(self.topic()).filter(|t| !matches!(t, Topic::Welcome | Topic::Summary | Topic::Done)) { if !v.contains(&c) { v.insert(0, c); } }
        v
    }

    fn goto(&mut self, i: usize) { self.idx = i.min(self.steps.len().saturating_sub(1)); self.reached = self.reached.max(self.idx); self.apply_error = None; self.refresh_status(true); }

    /// Zu einem Schritt springen (Wechsel des Weges, wenn er im aktuellen fehlt)
    pub(crate) fn jump_to(&mut self, t: Topic) {
        if !self.steps.contains(&t) { self.mode = Mode::Full; self.steps = self.mode.steps(); }
        if let Some(i) = self.steps.iter().position(|x| *x == t) { self.goto(i); }
    }

    /// „Später fortsetzen“: Stand der Schritte merken (keine Eingaben, keine Zugangsdaten), Darstellung zurücksetzen
    fn save_later(&mut self) {
        if self.shot { return; }
        let later = self.later_topics();
        let step = self.topic();
        for t in later { self.wiz.mark_later(t); }
        self.wiz.resume_step = step.id().into(); self.wiz.mode = self.mode.id().into();
        let _ = self.wiz.save(&self.paths.wizard);
    }

    /// Nach „Übernehmen“ oder „Abschließen“: `wizard.json` fortschreiben
    fn finish_state(&mut self) {
        for t in Topic::ALL { if !self.skipped.contains(&t) && self.steps.contains(&t) { self.wiz.clear_later(t); } }
        for t in self.skipped.clone() { self.wiz.mark_later(t); }
        self.wiz.first_run_done = true; self.wiz.last_run = setup::today(); self.wiz.mode = self.mode.id().into(); self.wiz.resume_step.clear();
        self.wiz.known_programs = setup::found_kinds(&|k| vc_ide::platform::discover(k));
        if let Some(a) = &self.applied { for e in &a.copy.copied { if !self.wiz.examples_copied.contains(e) { self.wiz.examples_copied.push(e.clone()); } } }
        if !self.shot { let _ = self.wiz.save(&self.paths.wizard); }
    }

    fn do_apply(&mut self) -> Option<View> {
        if self.shot { self.apply_error = Some("Selbsttest: es wird nichts geschrieben".into()); return None; }
        let keys = self.selected_keys();
        match setup::apply_to(&self.paths, &self.cur, &self.draft, &|k| keys.contains(k)) {
            Ok(a) => {
                if self.sync_memory { setup::apply_replace_memory(&a); }
                self.notes = a.notes.clone();
                let view = View::from_prefs(&a.view);
                self.cur = setup::load_snapshot(&self.paths).unwrap_or_else(|_| Snapshot { settings: a.settings.clone(), view: a.view.clone(), disk_hash: 0 });
                self.draft = Draft { settings: a.settings.clone(), view: a.view.clone(), examples: setup::ExamplesCfg { copy: false, excluded: self.draft.examples.excluded.clone(), source: self.draft.examples.source.clone(), dest: self.draft.examples.dest.clone() } };
                self.applied = Some(a);
                self.deselected.clear();
                self.apply_error = None;
                self.finish_state();
                self.refresh_status(true);
                Some(view)
            }
            Err(e) => { self.apply_error = Some(e); None }
        }
    }

    /// Ein Bild des Assistenten. Liefert, was die Anwendung tun soll.
    pub fn show(&mut self, ctx: &egui::Context, env: &mut Env) -> Out {
        if self.st.poll() { ctx.request_repaint_after(Duration::from_millis(200)); }
        // Enter, das ein Eingabefeld bestätigt (Fokus geht im selben Bild verloren), darf nicht auch „Weiter“ auslösen
        let typing_at_start = ctx.egui_wants_keyboard_input();
        // Darstellung wirkt sofort; der Entwurf folgt ihr
        self.draft.view = env.view.to_prefs();
        self.refresh_status(false);
        let mut out = Out::None;
        if self.tool_only {
            if let Some(o) = self.show_side(ctx, env) { return o; }
            return Out::None;
        }
        // abgedunkelte Fläche unter dem Fenster; sie fängt Klicks ab (modal)
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("setup_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_black_alpha(170));
        });
        let size = egui::vec2((screen.width() - 48.0).clamp(820.0, 1180.0), (screen.height() - 48.0).clamp(520.0, 780.0));
        let mut nav = Nav::None;
        egui::Window::new("Konfigurationsassistent").id(egui::Id::new("setup_wizard_win")).title_bar(false).collapsible(false).resizable(false).order(egui::Order::Foreground)
            .frame(crate::ui::dialog::frame().inner_margin(egui::Margin::ZERO)).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).fixed_size(size).show(ctx, |ui| {
            ui.set_min_size(size);
            egui::Panel::left("setup_rail").exact_size(244.0).resizable(false).frame(egui::Frame::new().fill(t().bg1).inner_margin(egui::Margin::same(14))).show_inside(ui, |ui| { if let Some(n) = rail::show(ui, self) { nav = n; } });
            egui::Panel::bottom("setup_foot").frame(egui::Frame::new().fill(t().bg2).inner_margin(egui::Margin::symmetric(18, 10))).show_inside(ui, |ui| { let n = self.footer(ui); if n != Nav::None { nav = n; } });
            egui::CentralPanel::default().frame(egui::Frame::new().fill(t().bg3).inner_margin(egui::Margin::symmetric(22, 14))).show_inside(ui, |ui| {
                self.header(ui);
                ui.add_space(6.0);
                egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt(("setup_body", self.idx)).show(ui, |ui| { ui.set_width(ui.available_width() - 6.0); self.body(ui, env, &mut out); });
            });
        });
        if self.confirm_later { if let Some(n) = self.show_confirm_later(ctx) { nav = n; } }
        if let Some(o) = self.show_side(ctx, env) { return o; }
        // Tastatur: Enter = Weiter, Alt+Pfeil links = Zurück, Esc = Später (fragt nach)
        if nav == Nav::None && !self.confirm_later && matches!(self.side, side::Side::None) {
            let (enter, back, esc) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::Escape)));
            if esc { self.confirm_later = true; }
            else if back { nav = Nav::Back; }
            else if enter && !typing_at_start && !ctx.egui_wants_keyboard_input() { nav = if self.topic() == Topic::Summary { Nav::Apply } else { Nav::Next }; }
        }
        if !matches!(out, Out::None) { return out; }
        match nav {
            Nav::None => {}
            Nav::Back => { if self.idx > 0 { let i = self.idx - 1; self.goto(i); } }
            Nav::Next => { if self.blocked().is_none() { self.skipped.remove(&self.topic()); if self.idx + 1 < self.steps.len() { let i = self.idx + 1; self.goto(i); } } }
            Nav::Skip => { let t = self.topic(); self.skipped.insert(t); if self.idx + 1 < self.steps.len() { let i = self.idx + 1; self.goto(i); } }
            Nav::Jump(i) => self.goto(i),
            Nav::Later => { self.save_later(); *env.view = self.view_start; return Out::Close; }
            Nav::Stay => { self.confirm_later = false; }
            Nav::Apply => {
                if self.changes.iter().any(|c| c.selected && c.kind != Kind::Same) {
                    if let Some(v) = self.do_apply() { *env.view = v; if self.idx + 1 < self.steps.len() { let i = self.idx + 1; self.goto(i); } return Out::Applied(v); }
                } else { self.finish_state(); if self.idx + 1 < self.steps.len() { let i = self.idx + 1; self.goto(i); } else { return Out::Close; } }
            }
            Nav::Close => { if self.applied.is_none() && !matches!(self.topic(), Topic::Done) { *env.view = self.view_start; } return Out::Close; }
            Nav::Undo => {
                if let Some(a) = self.applied.take() {
                    match setup::undo_to(&self.paths, &a) {
                        Ok(()) => {
                            let before = setup::load_snapshot(&self.paths).unwrap_or_else(|_| self.cur.clone());
                            if self.sync_memory { setup::apply_replace_memory_settings(before.settings.clone()); }
                            let v = View::from_prefs(&before.view); *env.view = v;
                            self.cur = before; self.draft = Draft::from_snapshot(&self.cur); self.refresh_status(true);
                            return Out::Undone(v);
                        }
                        Err(e) => { self.apply_error = Some(e); self.applied = Some(a); }
                    }
                }
            }
        }
        out
    }

    fn show_confirm_later(&mut self, ctx: &egui::Context) -> Option<Nav> {
        let mut nav = None;
        let a = crate::ui::dialog::show(ctx, &crate::ui::dialog::Dlg { title: "Später fortsetzen?", sub: "", chips: &[], primary: "Später fortsetzen", secondary: "Weitermachen" }, |ui| {
            ui.label("Der Assistent merkt sich nur den Schritt und die offenen Themen, nicht deine Eingaben und keine Zugangsdaten.");
            ui.label("Bisher ist nichts in deine Einstellungen geschrieben worden.");
            true
        });
        match a { crate::ui::dialog::Act::Primary => nav = Some(Nav::Later), crate::ui::dialog::Act::Secondary => nav = Some(Nav::Stay), _ => {} }
        nav
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let (n, total) = (self.idx + 1, self.steps.len());
        let topic = self.topic();
        let mode = match self.mode { Mode::Quick => "Schnelleinrichtung", Mode::Full => "Alles selbst einstellen", Mode::Part(_) => "Teil-Assistent" };
        widgets::overline(ui, &format!("Schritt {n} von {total} · {} · {mode}", topic.group()));
        ui.label(RichText::new(topic_title(topic, self.recall)).font(egui::FontId::new(24.0, crate::fonts::ui_strong())).color(t().fg));
        ui.label(RichText::new(topic_lead(topic, self.recall)).color(t().mute));
        ui.add_space(4.0);
        rail::progress(ui, self);
    }

    fn body(&mut self, ui: &mut egui::Ui, env: &mut Env, out: &mut Out) {
        if let Some(e) = self.load_error.clone() { if self.topic() == Topic::Welcome { self.load_error_card(ui, &e); return; } }
        match self.topic() {
            Topic::Welcome => self.step_welcome(ui),
            Topic::Design => self.step_design(ui, env),
            Topic::Systems => self.step_systems(ui),
            Topic::Toolchain => self.step_toolchain(ui),
            Topic::Emulators => self.step_emulators(ui),
            Topic::Roms => self.step_roms(ui),
            Topic::Hardware => self.step_hardware(ui),
            Topic::StartTarget => self.step_start(ui),
            Topic::Editor => self.step_editor(ui, env),
            Topic::Projects => self.step_projects(ui),
            Topic::Summary => self.step_summary(ui),
            Topic::Done => self.step_done(ui, out),
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui) -> Nav {
        let k = t();
        let mut nav = Nav::None;
        let topic = self.topic();
        let last = self.idx + 1 >= self.steps.len();
        ui.horizontal(|ui| {
            if topic != Topic::Done && ui.button("Später fortsetzen").on_hover_text("Esc").clicked() { self.confirm_later = true; }
            ui.add_space(8.0);
            if let Some(reason) = self.blocked() { ui.add(egui::Label::new(RichText::new(format!("! {reason}")).small().color(k.am)).truncate()); }
            else { ui.add(egui::Label::new(RichText::new(footer_hint(topic)).small().color(k.mute)).truncate()); }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if topic == Topic::Done || (last && topic != Topic::Summary) {
                    if crate::shell::action_button(ui, "Fertig", crate::shell::Kind::Primary, true).clicked() { nav = Nav::Close; }
                    if self.applied.is_some() && ui.button("Rückgängig").on_hover_text("Stellt Einstellungen und Beispiele wieder her (bis zum Neustart)").clicked() { nav = Nav::Undo; }
                    return;
                }
                if topic == Topic::Summary {
                    let any = self.changes.iter().any(|c| c.selected && c.kind != Kind::Same);
                    let label = if any { "Übernehmen" } else { "Ohne Änderung abschließen" };
                    if crate::shell::action_button(ui, label, crate::shell::Kind::Primary, self.blocked().is_none()).clicked() { nav = Nav::Apply; }
                } else if crate::shell::action_button(ui, "Weiter", crate::shell::Kind::Primary, self.blocked().is_none()).on_hover_text("Enter").clicked() { nav = Nav::Next; }
                if !matches!(topic, Topic::Welcome | Topic::Summary) && ui.button("Überspringen").clicked() { nav = Nav::Skip; }
                if self.idx > 0 && ui.button("Zurück").on_hover_text("Alt+Pfeil links").clicked() { nav = Nav::Back; }
            });
        });
        nav
    }

    fn load_error_card(&mut self, ui: &mut egui::Ui, err: &str) {
        let k = t();
        crate::ui::widgets::card(ui, false, |ui| {
            widgets::note(ui, "!", "Die Einstellungsdatei lässt sich nicht lesen. Der Assistent überschreibt sie nie ohne Sicherung.", k.rd);
            ui.label(RichText::new(err).monospace().small());
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Neu beginnen (Sicherung der alten Datei anlegen)").clicked() {
                    match setup::backup_broken(&self.paths) {
                        Ok(b) => { self.notes.push(format!("Sicherung angelegt: {}", b.display())); self.load_error = None; self.cur.disk_hash = std::fs::read(&self.paths.settings).map_or(0, |b| setup::hash_of(&b)); self.draft = Draft::for_first_run(&self.cur); self.refresh_status(true); }
                        Err(e) => self.apply_error = Some(e),
                    }
                }
            });
            if let Some(e) = &self.apply_error { widgets::note(ui, "!", e, k.rd); }
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Nav { None, Back, Next, Skip, Jump(usize), Later, Stay, Apply, Close, Undo }

/// Themen, die ein Teil-Assistent ändern darf (alles andere bleibt nachweislich unverändert)
pub fn part_topics(p: Part) -> &'static [Topic] {
    match p { Part::Roms => &[Topic::Roms, Topic::Summary], Part::Emulators => &[Topic::Emulators, Topic::Toolchain, Topic::Systems, Topic::Summary], Part::Hardware => &[Topic::Hardware, Topic::StartTarget, Topic::Summary], Part::DesignEditor => &[Topic::Design, Topic::Editor, Topic::Summary] }
}

fn topic_title(t: Topic, recall: bool) -> &'static str {
    match t {
        Topic::Welcome => if recall { "Konfiguration prüfen" } else { "Willkommen bei VintageCoder" },
        Topic::Design => "Design und Größe", Topic::Systems => "Welche Systeme nutzt du?", Topic::Toolchain => "Toolchain: was ist installiert?", Topic::Emulators => "Emulatoren und Probestart",
        Topic::Roms => "ROMs", Topic::Hardware => "Echte Hardware", Topic::StartTarget => "Wohin startet F5?", Topic::Editor => "Editor und Tasten",
        Topic::Projects => "Projekte und Beispiele", Topic::Summary => "Zusammenfassung: was wird geändert?", Topic::Done => "Fertig",
    }
}

fn topic_lead(t: Topic, recall: bool) -> &'static str {
    match t {
        Topic::Welcome => if recall { "Dein gespeicherter Stand steht oben und führt. Geändert wird nur, was du in der Zusammenfassung bestätigst." } else { "In wenigen Minuten vom Start bis „F5 startet mein Programm“. Alles bleibt auf diesem Rechner." },
        Topic::Design => "Farbschema, Größe und Schrift; wirkt sofort. Du kannst es später in den Einstellungen ändern.",
        Topic::Systems => "Nur die gewählten Systeme erscheinen in der Oberfläche. Abgewählte lassen sich später wieder einblenden.",
        Topic::Toolchain => "Der Assistent zeigt, was gefunden wurde, und installiert nie etwas selbst.",
        Topic::Emulators => "Je System ein Standard-Emulator für F5; der Probestart prüft, ob er läuft.",
        Topic::Roms => "VintageCoder liefert keine Original-ROMs mit und lädt keine herunter; freie Ersatz-ROMs genügen oft.",
        Topic::Hardware => "Ultimate 64 / II+ und MEGA65 eintragen. Verbindungen werden nur auf Klick getestet.",
        Topic::StartTarget => "F5 baut und startet: auf dem Emulator oder auf einer eingetragenen Maschine.",
        Topic::Editor => "Schrift, Adressspalte und Tasten. Die Tastenkürzel sind fest.",
        Topic::Projects => "Wo deine Projekte liegen, und ob die mitgelieferten Beispiele dorthin kopiert werden.",
        Topic::Summary => "Letzte Kontrolle. Jede Änderung lässt sich abwählen; geschrieben wird erst bei „Übernehmen“.",
        Topic::Done => "Die Einrichtung ist abgeschlossen.",
    }
}

fn footer_hint(t: Topic) -> &'static str {
    match t { Topic::Summary => "Vor dem Schreiben entsteht settings.json.bak.", Topic::StartTarget => "F6 (Debuggen) startet immer im Emulator.", Topic::Hardware => "Passwörter erscheinen nie im Klartext in Liste oder Export.", _ => "Nichts wird gesendet. Überspringen ist jederzeit möglich." }
}
