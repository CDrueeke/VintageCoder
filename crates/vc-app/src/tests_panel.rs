// SPDX-License-Identifier: GPL-3.0-or-later
//! Reiter „Tests“ (Testkonzept T3): Explorer mit Gruppen und Zuständen, Kopfleiste, Detail mit Erwartung, Registern und Zyklen.
//! Das Ergebnismodell und der Lauf kommen aus `vc_ide::tests::suite`; hier steht nur die Ansicht und der Arbeits-Thread des Laufs.
//! Aktionen, die das Dock, Editoren oder den Debugger betreffen, gibt `show` als `Act` an die Anwendung zurück.
use crate::ui::{tokens::t, widgets};
use eframe::egui::{self, pos2, vec2, Color32, RichText, Sense, Stroke};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use vc_ide::tests::suite::{self, Event, Hardware, Only, RunRequest, Suite};
use vc_ide::tests::{Level, Outcome, State, TestCase, TestFile};

/// Was die Ansicht von der Anwendung verlangt
#[derive(Debug, Clone, PartialEq)]
pub enum Act {
    Run(Only),
    Stop,
    /// „Diesen Test debuggen“: Testdatei und Testname
    Debug(PathBuf, String),
    /// „Zur Quelle“: absoluter oder projektrelativer Pfad und 1-basierte Zeile
    Source(String, usize),
}

pub struct TestsState {
    pub suite: Suite,
    /// Zählt jede Änderung der Ergebnisse (die Anwendung aktualisiert damit die Marken der Editoren)
    pub version: u64,
    rx: Option<Receiver<Event>>,
    cancel: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// Installierte Emulatoren (Kennung, Name) und die Wahl (`None`: nur statische Prüfungen)
    pub backends: Vec<(String, String)>,
    pub backend: Option<String>,
    filter: String,
    only_failed: bool,
    sel: Option<(PathBuf, String)>,
    /// Zwischenspeicher für die Setup-Zeile des gewählten Tests: (Schlüssel, Text)
    setup: Option<((PathBuf, String), String)>,
    started: Option<Instant>,
    /// Wartet auf die Sicherheitsabfrage (Mockup 3): der Lauf auf Hardware, der nach „Auf Hardware testen“ startet
    confirm: Option<Only>,
}

impl Default for TestsState {
    fn default() -> Self {
        let backends = suite::available_backends();
        let backend = backends.first().map(|b| b.0.clone());
        TestsState { suite: Suite::default(), version: 0, rx: None, cancel: Arc::new(AtomicBool::new(false)), thread: None, backends, backend, filter: String::new(), only_failed: false, sel: None, setup: None, started: None, confirm: None }
    }
}

impl TestsState {
    /// „Tests 13/16“ für die Statusleiste
    pub fn status_text(&self) -> Option<String> { self.suite.status_text() }
    pub fn failed_count(&self) -> usize { self.suite.failed_count() }
    /// Test im Fußpanel „Tests“ vorwählen (Klick in der Seitenleiste)
    pub fn select(&mut self, file: PathBuf, name: String) { self.sel = Some((file, name)); }
    pub fn is_running(&self) -> bool { self.suite.running }

    /// Kennung (`hw:<Maschine>`) des gewählten Geräts, falls „Läuft auf“ Hardware ist
    pub fn hardware_selected(&self) -> Option<String> { self.backend.as_deref().filter(|b| b.starts_with("hw:")).map(String::from) }
    /// Ein Emulator (kein Gerät) ist gewählt: nur dann gibt es „Debuggen“
    fn emulator_selected(&self) -> bool { self.backend.as_deref().map_or(false, |b| !b.starts_with("hw:")) }

    /// Sicherheitsabfrage bei jedem Lauf auf Hardware: ein Lauf wird nicht gestartet, sondern vorgemerkt und im Dialog bestätigt
    fn gate(&mut self, act: Option<Act>) -> Option<Act> {
        match act {
            Some(Act::Run(only)) if self.hardware_selected().is_some() => { self.confirm = Some(only); None }
            a => a,
        }
    }

    /// Startet einen Lauf im Arbeits-Thread (ein laufender wird zuvor abgebrochen)
    pub fn start(&mut self, root: PathBuf, data: PathBuf, only: Only, strict: bool, ctx: &egui::Context) {
        self.stop_blocking();
        let hardware = match self.hardware_selected() {
            None => None,
            Some(id) => match hardware_machines().into_iter().find(|(i, _)| *i == id) {
                Some((_, h)) => Some(h),
                None => { self.suite.error = Some("Das gewählte Gerät steht nicht mehr in den Einstellungen.".into()); self.version += 1; return; }
            },
        };
        let req = RunRequest { root, data, backend: if hardware.is_some() { None } else { self.backend.clone() }, strict, only: only.clone(), hardware };
        let (tx, rx) = channel();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        self.suite.begin(&only);
        self.version += 1;
        self.rx = Some(rx);
        self.started = Some(Instant::now());
        let ctx = ctx.clone();
        self.thread = Some(std::thread::spawn(move || { suite::run(&req, cancel, &tx); ctx.request_repaint(); }));
    }

    /// Bricht einen laufenden Lauf ab und wartet (höchstens 10 s), bis der Emulator beendet ist – es läuft immer nur ein Emulator
    pub fn stop_blocking(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
        if let Some(h) = self.thread.take() {
            let end = Instant::now() + Duration::from_secs(10);
            while !h.is_finished() && Instant::now() < end { std::thread::sleep(Duration::from_millis(40)); }
            if h.is_finished() { let _ = h.join(); }
        }
        self.poll();
        self.rx = None;
        if self.suite.running { self.suite.running = false; self.version += 1; }
    }

    /// Ereignisse des Laufs übernehmen (jedes Bild aufrufen)
    pub fn poll(&mut self) {
        let Some(rx) = &self.rx else { return };
        let mut changed = false;
        loop {
            match rx.try_recv() {
                Ok(e) => { self.suite.apply(e); changed = true; }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => { if self.suite.running { self.suite.apply(Event::Error("der Testlauf wurde unerwartet beendet".into())); changed = true; } self.rx = None; break; }
            }
        }
        if changed { self.version += 1; }
    }
}

/// Maschinen aus den Einstellungen, die als Testgerät in Frage kommen: (Kennung `hw:<id>`, Hardware)
fn hardware_machines() -> Vec<(String, Hardware)> {
    vc_ide::settings::effective().machines.iter().filter_map(|m| {
        let id = format!("hw:{}", m.id);
        match m.kind.as_str() {
            "ultimate64" if !m.host.trim().is_empty() => Some((id, Hardware::Ultimate { host: m.host.trim().to_string(), password: Some(m.password.clone()).filter(|p| !p.is_empty()), name: m.display_name().to_string() })),
            "mega65" => vc_ide::settings::mega65_spec(m).ok().map(|spec| (id, Hardware::Mega65 { spec, name: m.display_name().to_string() })),
            _ => None,
        }
    }).collect()
}

fn widgets_tint(c: Color32) -> Color32 { crate::ui::tokens::tint(c) }

fn state_color(o: &Outcome) -> Color32 {
    let k = t();
    match (o.state, o.level) { (State::Passed, _) => k.gr, (State::Failed, Level::Error) => k.rd, (State::Failed, Level::Hint) | (State::TimedOut, _) => k.am, (State::Skipped, _) => k.dim }
}

fn state_label(o: &Outcome) -> &'static str {
    match (o.state, o.level) { (State::Passed, _) => "bestanden", (State::Failed, Level::Error) => "fehlgeschlagen", (State::Failed, Level::Hint) => "Hinweis", (State::TimedOut, _) => "Zeitüberschreitung", (State::Skipped, _) => "übersprungen" }
}

/// Statuspunkt: gefüllt (bestanden, fehlgeschlagen, Hinweis), Uhr (Zeitüberschreitung), Ring (übersprungen)
pub fn dot(p: &egui::Painter, c: egui::Pos2, o: &Outcome) {
    let col = state_color(o);
    match o.state {
        State::Skipped => { p.circle_stroke(c, 4.5, Stroke::new(1.4, col)); }
        State::TimedOut => { p.circle_stroke(c, 5.0, Stroke::new(1.4, col)); p.line_segment([c, c + vec2(0.0, -3.2)], Stroke::new(1.3, col)); p.line_segment([c, c + vec2(2.6, 1.0)], Stroke::new(1.3, col)); }
        _ => { p.circle_filled(c, 4.5, col); }
    }
}

fn seconds(ms: u64) -> String { if ms < 1000 { format!("{ms} ms") } else { format!("{:.1} s", ms as f64 / 1000.0).replace('.', ",") } }

/// Registerstand als Kacheln und Flags
fn regs_ui(ui: &mut egui::Ui, r: &vc_ide::tests::RegSnap) {
    ui.horizontal_wrapped(|ui| {
        for (n, v) in [("A", format!("${:02X}", r.a)), ("X", format!("${:02X}", r.x)), ("Y", format!("${:02X}", r.y)), ("SP", format!("${:02X}", r.sp)), ("PC", format!("${:04X}", r.pc))] { widgets::reg_tile(ui, n, &v, true); }
    });
    let flags: String = "NV-BDIZC".chars().enumerate().map(|(i, c)| if r.flags & (0x80 >> i) != 0 { c } else { '·' }).collect();
    ui.label(RichText::new(flags).monospace().color(t().gr));
}

impl TestsState {
    /// Setup-Zeile des Tests (Aufruf, Speicher, Register) aus der Testdatei
    fn setup_line(&mut self, file: &Path, name: &str) -> Option<String> {
        let key = (file.to_path_buf(), name.to_string());
        if let Some((k, s)) = &self.setup { if *k == key { return Some(s.clone()); } }
        let tf = TestFile::load(file).ok()?;
        let (i, raw) = tf.tests.iter().enumerate().find(|(_, t)| t.get("name").and_then(|n| n.as_str()) == Some(name))?;
        let c = TestCase::parse(i, raw).ok()?;
        let mut parts: Vec<String> = c.mem.iter().map(|(a, v)| format!("[{a}]={}", v.join(","))).collect();
        parts.extend(c.regs.iter().map(|(r, v)| format!("{}={v}", r.to_uppercase())));
        parts.extend(c.flags.iter().map(|(f, on)| format!("{}={}", f.to_uppercase(), *on as u8)));
        let s = format!("Setup: {} · Aufruf {}{}", if parts.is_empty() { "keines".to_string() } else { parts.join(" ") }, c.call, if c.rti { " (rti)" } else { "" });
        self.setup = Some((key, s.clone()));
        Some(s)
    }

    /// Zeichnet den Reiter. `can_run`: ein Projekt ist offen und der Datenordner gefunden.
    pub fn show(&mut self, ui: &mut egui::Ui, can_run: bool) -> Option<Act> {
        let a = self.show_main(ui, can_run);
        let mut act = self.gate(a);
        if let Some(only) = self.confirm.clone() {
            let hw = self.hardware_selected().and_then(|id| hardware_machines().into_iter().find(|(i, _)| *i == id)).map(|x| x.1);
            match hw.map_or(Some(false), |h| self.confirm_dialog(ui.ctx(), &h)) {
                Some(true) => { self.confirm = None; act = Some(Act::Run(only)); }
                Some(false) => self.confirm = None,
                None => {}
            }
        }
        act
    }

    /// Mockup 3: „Test auf echter Hardware starten?“ – `Some(true)` bestätigt, `Some(false)` bricht ab
    fn confirm_dialog(&mut self, ctx: &egui::Context, hw: &Hardware) -> Option<bool> {
        let k = t();
        let mut result = None;
        egui::Window::new("Test auf echter Hardware starten?").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0)).show(ctx, |ui| {
            ui.set_max_width(460.0);
            widgets::card(ui, false, |ui| {
                ui.label(RichText::new(hw.label()).strong());
                ui.label(RichText::new(match hw { Hardware::Ultimate { host, .. } => format!("Ultimate 64 / II+ · REST · {host}"), Hardware::Mega65 { spec, .. } => format!("MEGA65 · UART · {spec} (ungeprüft, Gerät war offline)") }).small().color(k.dim));
                ui.label(RichText::new("Die Verbindung wird beim Start geprüft.").small().color(k.dim));
            });
            ui.add_space(6.0);
            egui::Frame::new().fill(widgets_tint(k.am)).stroke(Stroke::new(1.0, k.am.gamma_multiply(0.4))).corner_radius(4.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                ui.label(RichText::new("Achtung").strong().color(k.am));
                ui.label("Der laufende Zustand des Geräts geht verloren: der Test setzt es zurück, lädt ein Programm und schreibt in den Speicher (Postfach im RAM, Programm ab $0801). Gespeicherte Daten im RAM bleiben nicht erhalten.");
                ui.label("Jeder Test wird nach seinem Zeitlimit (Standard 15 s) abgebrochen; danach läuft das Gerät weiter. Es laufen nur Postfach-Tests, Trap-Tests werden übersprungen.");
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Abbrechen").clicked() { result = Some(false); }
                if ui.add(egui::Button::new(RichText::new("Auf Hardware testen").color(k.on_cy)).fill(k.cy)).clicked() { result = Some(true); }
            });
        });
        result
    }

    fn show_main(&mut self, ui: &mut egui::Ui, can_run: bool) -> Option<Act> {
        let k = t();
        let mut act: Option<Act> = None;
        let running = self.suite.running;
        // ---- Kopfleiste ----
        ui.horizontal_wrapped(|ui| {
            if running {
                if ui.button("■ Stopp").clicked() { act = Some(Act::Stop); }
            } else if ui.add_enabled(can_run, egui::Button::new(RichText::new("▶ Alle ausführen").color(k.on_cy)).fill(k.cy)).on_hover_text(if can_run { "statische Prüfungen und Tests im Emulator" } else { "erst ein Projekt öffnen und bauen" }).clicked() { act = Some(Act::Run(Only::All)); }
            let failed = self.suite.failed_tests();
            if ui.add_enabled(can_run && !running && !failed.is_empty(), egui::Button::new("Fehlgeschlagene")).on_hover_text("nur die fehlgeschlagenen Tests im Emulator wiederholen").clicked() { act = Some(Act::Run(Only::Failed(failed))); }
            let dbg_target = self.sel.clone().filter(|(f, n)| !running && self.suite.groups.iter().any(|g| g.file.as_ref() == Some(f) && !g.mailbox && g.items.iter().any(|i| &i.outcome.name == n)));
            if ui.add_enabled(dbg_target.is_some() && self.emulator_selected(), egui::Button::new("Debuggen")).on_hover_text("den gewählten Test im Debugger starten, angehalten an der Falle").clicked() { if let Some((f, n)) = dbg_target { act = Some(Act::Debug(f, n)); } }
            ui.separator();
            ui.label(RichText::new("Läuft auf:").color(k.dim));
            let cur = match &self.backend { None => "nur statisch".to_string(), Some(b) if b.starts_with("hw:") => hardware_machines().into_iter().find(|(i, _)| i == b).map_or("Gerät fehlt".to_string(), |(_, h)| h.label()), Some(b) => self.backends.iter().find(|x| &x.0 == b).map_or(b.clone(), |x| x.1.clone()) };
            egui::ComboBox::from_id_salt("tests_backend").selected_text(cur).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.backend, None, "nur statisch (ohne Emulator)");
                for (id, name) in self.backends.clone() { ui.selectable_value(&mut self.backend, Some(id), name); }
                let hw = hardware_machines();
                if !hw.is_empty() { ui.separator(); ui.label(RichText::new("Echte Hardware (nur Postfach-Tests, mit Sicherheitsabfrage)").small().color(k.dim)); }
                for (id, h) in hw { ui.selectable_value(&mut self.backend, Some(id), h.label()); }
            });
        });
        let c = self.suite.counts();
        ui.horizontal_wrapped(|ui| {
            if self.suite.is_empty() && !running { ui.label(RichText::new("Noch kein Lauf.").color(k.dim)); }
            if c.passed > 0 { widgets::chip(ui, &format!("{} bestanden", c.passed), k.gr); }
            if c.failed > 0 { widgets::chip(ui, &format!("{} fehlgeschlagen", c.failed), k.rd); }
            if c.timed_out > 0 { widgets::chip(ui, &format!("{} Zeitüberschreitung", c.timed_out), k.am); }
            if c.hints > 0 { widgets::chip(ui, &format!("{} Hinweis{}", c.hints, if c.hints == 1 { "" } else { "e" }), k.am); }
            if c.skipped > 0 { widgets::chip(ui, &format!("{} übersprungen", c.skipped), k.dim); }
            if running { ui.spinner(); ui.label(RichText::new(format!("läuft … {}", self.started.map_or(String::new(), |s| seconds(s.elapsed().as_millis() as u64)))).color(k.am)); ui.ctx().request_repaint_after(Duration::from_millis(150)); }
            else if let Some(ms) = self.suite.last_millis { ui.label(RichText::new(format!("Lauf in {}", seconds(ms))).color(k.dim)); }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter …").desired_width(150.0));
                ui.checkbox(&mut self.only_failed, "nur Fehler");
            });
        });
        if let Some(e) = &self.suite.error { ui.colored_label(k.rd, e); }
        ui.separator();
        if self.suite.groups.is_empty() {
            ui.add_space(6.0);
            ui.label(RichText::new("Tests liegen als *.vctest im Projekt (Größe, Zeropage, Zyklen, Aufrufe im Emulator). Zuerst bauen, dann „Alle ausführen“.").color(k.dim));
            return act;
        }
        // ---- Explorer links, Detail rechts ----
        let h = ui.available_height().max(120.0);
        let total_w = ui.available_width();
        let lw = (total_w * 0.55).clamp(260.0, (total_w - 240.0).max(260.0));
        let visible = self.suite.visible(&self.filter, self.only_failed);
        let mut clicked: Option<(PathBuf, String)> = None;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(lw, h), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().id_salt("tests_list").auto_shrink([false, false]).show(ui, |ui| {
                    for (gi, g) in self.suite.groups.iter().enumerate() {
                        let rows: Vec<usize> = visible.iter().filter(|(a, _)| *a == gi).map(|(_, b)| *b).collect();
                        if rows.is_empty() && (!self.filter.trim().is_empty() || self.only_failed) { continue; }
                        let gc = g.counts();
                        let ok_n = gc.passed;
                        let head = format!("{} · {}/{}{}", g.title, ok_n, g.items.len().max(g.expected), if g.millis > 0 { format!(" · {}", seconds(g.millis)) } else { String::new() });
                        let mut label = RichText::new(head).strong();
                        if gc.failed + gc.timed_out > 0 { label = label.color(k.rd); }
                        egui::CollapsingHeader::new(label).id_salt(("tests_group", gi)).default_open(true).show(ui, |ui| {
                            if let Some(b) = &g.backend { ui.label(RichText::new(format!("Emulator: {b}")).small().color(k.dim)); }
                            if g.expected > g.items.len() { ui.label(RichText::new(format!("läuft … {} von {}", g.items.len(), g.expected)).small().color(k.am)); }
                            for ii in rows {
                                let it = &g.items[ii];
                                let o = &it.outcome;
                                let key = (it.file.clone(), o.name.clone());
                                let selected = self.sel.as_ref() == Some(&key);
                                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 21.0), Sense::click());
                                if selected { ui.painter().rect_filled(rect, 3.0, k.bg3); } else if resp.hovered() { ui.painter().rect_filled(rect, 3.0, k.bg2); }
                                let p = ui.painter_at(rect);
                                dot(&p, pos2(rect.left() + 11.0, rect.center().y), o);
                                let font = egui::TextStyle::Body.resolve(ui.style());
                                let meas = o.measured.clone().unwrap_or_default();
                                let mw = if meas.is_empty() { 0.0 } else { p.layout_no_wrap(meas.clone(), egui::TextStyle::Monospace.resolve(ui.style()), k.dim).size().x + 10.0 };
                                if !meas.is_empty() { p.text(pos2(rect.right() - 6.0, rect.center().y), egui::Align2::RIGHT_CENTER, &meas, egui::TextStyle::Monospace.resolve(ui.style()), if o.state == State::Passed { k.gr } else { k.am }); }
                                let clip = egui::Rect::from_min_max(rect.min, pos2(rect.right() - mw - 4.0, rect.max.y));
                                ui.painter_at(clip).text(pos2(rect.left() + 24.0, rect.center().y), egui::Align2::LEFT_CENTER, &o.name, font, if selected { k.fg } else { crate::theme::label_gray() });
                                if resp.clicked() { clicked = Some(key); }
                                if o.state == State::Skipped && !o.detail.is_empty() {
                                    ui.horizontal(|ui| { ui.add_space(24.0); ui.add(egui::Label::new(RichText::new(&o.detail).small().color(k.dim)).wrap()); });
                                }
                                resp.context_menu(|ui| {
                                    if g.file.is_some() {
                                        if ui.add_enabled(!running && can_run, egui::Button::new("Nur diesen Test ausführen")).clicked() { act = Some(Act::Run(Only::Test(it.file.clone(), o.name.clone()))); ui.close(); }
                                        if ui.add_enabled(!running && self.emulator_selected() && !g.mailbox, egui::Button::new("Diesen Test debuggen")).on_disabled_hover_text("Im Postfach-Modus prüft sich das Programm selbst: normal debuggen (F6)").clicked() { act = Some(Act::Debug(it.file.clone(), o.name.clone())); ui.close(); }
                                    } else if ui.add_enabled(!running && can_run, egui::Button::new("Prüfungen erneut ausführen")).clicked() { act = Some(Act::Run(Only::All)); ui.close(); }
                                    if let Some((f, l)) = &o.location { if ui.button("Zur Quelle").clicked() { act = Some(Act::Source(f.clone(), *l)); ui.close(); } }
                                    if ui.button("Meldung kopieren").clicked() { ui.ctx().copy_text(format!("{} – {}: {}", o.name, state_label(o), o.detail)); ui.close(); }
                                });
                            }
                        });
                    }
                });
            });
            ui.separator();
            ui.allocate_ui_with_layout(vec2(ui.available_width(), h), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().id_salt("tests_detail").auto_shrink([false, false]).show(ui, |ui| {
                    let sel = self.sel.clone();
                    let found = sel.as_ref().and_then(|(f, n)| self.suite.groups.iter().find_map(|g| g.items.iter().find(|i| &i.file == f && &i.outcome.name == n).map(|i| (g.file.is_some(), g.mailbox, g.backend.as_deref().map_or(false, |b| b.ends_with("· Zyklen")), i.clone()))));
                    let Some((dynamic, mailbox, measure, it)) = found else { ui.label(RichText::new("Test wählen, um Erwartung, Ist-Wert und Register zu sehen.").color(k.dim)); return; };
                    let o = &it.outcome;
                    ui.horizontal_wrapped(|ui| { ui.label(RichText::new(&o.name).strong().size(14.0)); widgets::chip(ui, state_label(o), state_color(o)); });
                    if let Some((f, l)) = &o.location {
                        let short = Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| f.clone());
                        if ui.link(format!("{short}:{l}  ·  Zur Quelle")).clicked() { act = Some(Act::Source(f.clone(), *l)); }
                    }
                    ui.add_space(4.0);
                    let (what, body) = match o.state {
                        State::Skipped => ("Grund", o.detail.as_str()),
                        State::Passed => ("Ergebnis", ""),
                        State::TimedOut => ("Zeitüberschreitung", o.detail.as_str()),
                        State::Failed => ("Erwartung und Ist-Wert", o.detail.as_str()),
                    };
                    let body = body.split(". Zustand bei der Rückkehr").next().unwrap_or(body);
                    if !body.is_empty() { ui.label(RichText::new(what).small().color(k.dim)); ui.add(egui::Label::new(RichText::new(body).monospace()).wrap()); }
                    if let Some(m) = &o.measured { ui.add_space(4.0); ui.label(RichText::new("Zyklen / Messwert").small().color(k.dim)); ui.label(RichText::new(m).monospace().color(k.gr)); }
                    if let Some(r) = &o.regs { ui.add_space(4.0); ui.label(RichText::new("Zustand bei der Rückkehr").small().color(k.dim)); regs_ui(ui, r); }
                    if mailbox { ui.add_space(4.0); ui.add(egui::Label::new(RichText::new(if measure { "Zyklenmessung auf der Ultimate 64: Debug-Strom, Zyklen vom ersten Zugriff auf A bis zum ersten danach auf B (mit Interrupts und Bad Lines). Bei Paketverlust ist die Messung ungültig, nie geschätzt." } else { "Postfach-Modus: das Testprogramm prüft sich selbst (test.inc) und schreibt das Ergebnis ins Postfach im RAM; der Läufer liest nur Speicher." }).small().color(k.dim)).wrap()); }
                    if dynamic && !mailbox { if let Some(s) = self.setup_line(&it.file, &o.name) { ui.add_space(4.0); ui.add(egui::Label::new(RichText::new(s).small().monospace().color(k.dim)).wrap()); } }
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if dynamic {
                            if ui.add_enabled(!running && self.emulator_selected() && !mailbox, egui::Button::new("Diesen Test debuggen")).clicked() { act = Some(Act::Debug(it.file.clone(), o.name.clone())); }
                            if ui.add_enabled(!running && can_run, egui::Button::new("Nur diesen Test ausführen")).clicked() { act = Some(Act::Run(Only::Test(it.file.clone(), o.name.clone()))); }
                        }
                    });
                });
            });
        });
        if let Some(c) = clicked { self.sel = Some(c); }
        act
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_ide::tests::suite::{Group, Item};

    fn item(file: &str, o: Outcome) -> Item { Item { file: PathBuf::from(file), outcome: o } }

    fn sample() -> TestsState {
        let mut s = TestsState::default();
        s.suite.groups.push(Group { title: "Statische Prüfungen".into(), items: vec![
            item("/p/a.vctest", Outcome::passed("Größe Main ≤ $2000", Some("$1F3C".into()))),
            item("/p/a.vctest", Outcome::skipped("Zyklen clear_text ≤ 100", "nicht bestimmbar: Schleife bei $082F")),
        ], ..Default::default() });
        let regs = vc_ide::tests::RegSnap { pc: 0xC003, a: 0, x: 0, y: 0, sp: 0xFF, flags: 0x24 };
        s.suite.groups.push(Group { title: "a.vctest".into(), file: Some(PathBuf::from("/p/a.vctest")), backend: Some("VICE x64sc".into()), millis: 1800, expected: 6, mailbox: false, items: vec![
            item("/p/a.vctest", Outcome::passed("div16: 1000 / 8", Some("196 Zyklen".into())).at("/p/src/math.asm", 7).with_regs(regs)),
            item("/p/a.vctest", Outcome::failed("mul16: 300 × 300", Level::Error, "Erwartung „peekw($0200) == 90000“ ist nicht erfüllt: linke Seite = 24464 ($5F90), rechte Seite = 90000 ($15F90). Zustand bei der Rückkehr: A=$00").at("/p/src/math.asm", 50).with_regs(regs)),
            item("/p/a.vctest", Outcome::timed_out("hängt", "die Rückkehr nach 400 ms nicht erreicht")),
            item("/p/a.vctest", Outcome::failed("Budget", Level::Hint, "19 Zyklen gemessen, erlaubt: höchstens 18")),
        ] });
        s
    }

    /// Zeichnet den Reiter einige Bilder lang (kein Absturz, auch bei leerem Ergebnis, laufendem Lauf, gewähltem Test und schmalem Platz)
    fn draw(st: &mut TestsState, w: f32, h: f32) -> Option<Act> {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut act = None;
        for _ in 0..3 {
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| { if let Some(a) = st.show(ui, true) { act = Some(a); } });
            out.textures_delta.clear();
        }
        act
    }

    #[test]
    fn draws_without_panicking_in_all_states() {
        let mut empty = TestsState::default();
        assert_eq!(draw(&mut empty, 900.0, 300.0), None);
        let mut s = sample();
        assert_eq!(draw(&mut s, 1200.0, 400.0), None);
        s.sel = Some((PathBuf::from("/p/a.vctest"), "mul16: 300 × 300".into()));
        assert_eq!(draw(&mut s, 1200.0, 400.0), None);
        s.sel = Some((PathBuf::from("/p/a.vctest"), "Größe Main ≤ $2000".into()));
        s.filter = "mul".into(); s.only_failed = true; s.suite.running = true;
        assert_eq!(draw(&mut s, 520.0, 160.0), None, "schmaler Platz");
        assert_eq!(s.suite.status_text().as_deref(), Some("Tests 2/6"));
        assert_eq!(s.failed_count(), 2);
    }

    #[test]
    fn mailbox_group_draws_and_blocks_trap_debugging() {
        let mut s = TestsState::default();
        s.suite.groups.push(Group { title: "m.vctest".into(), file: Some(PathBuf::from("/p/m.vctest")), backend: Some("VICE x64sc · Postfach".into()), mailbox: true, items: vec![
            item("/p/m.vctest", Outcome::passed("Selbsttest", Some("4 Prüfungen".into()))),
            item("/p/m.vctest", Outcome::failed("kaputt", Level::Error, "Prüfung Nr. 2 von 5 fehlgeschlagen (assert_x_eq (X)): Ist-Wert 6 ($06), erwartet 7 ($07)")),
        ], ..Default::default() });
        s.sel = Some((PathBuf::from("/p/m.vctest"), "kaputt".into()));
        assert_eq!(draw(&mut s, 1000.0, 300.0), None);
        s.sel = Some((PathBuf::from("/p/m.vctest"), "Selbsttest".into()));
        assert_eq!(draw(&mut s, 1000.0, 300.0), None);
    }

    #[test]
    fn runs_on_hardware_always_wait_for_the_confirmation() {
        let mut s = TestsState::default();
        s.backend = Some("hw:u64".into());
        assert_eq!(s.hardware_selected().as_deref(), Some("hw:u64"));
        assert!(!s.emulator_selected(), "auf Hardware gibt es kein Debuggen mit Falle");
        // jeder Lauf (alle, Fehlgeschlagene, einzelner Test) wird nur vorgemerkt
        for only in [Only::All, Only::File(PathBuf::from("/p/a.vctest")), Only::Test(PathBuf::from("/p/a.vctest"), "x".into()), Only::Failed(vec![])] {
            s.confirm = None;
            assert_eq!(s.gate(Some(Act::Run(only.clone()))), None);
            assert_eq!(s.confirm, Some(only));
        }
        // andere Aktionen laufen ungehindert durch
        assert_eq!(s.gate(Some(Act::Stop)), Some(Act::Stop));
        assert_eq!(s.gate(None), None);
        // Emulator und „nur statisch“ fragen nicht
        for b in [Some("vice".to_string()), None] {
            s.backend = b; s.confirm = None;
            assert_eq!(s.gate(Some(Act::Run(Only::All))), Some(Act::Run(Only::All)));
            assert_eq!(s.confirm, None);
        }
        // ein Gerät, das nicht (mehr) in den Einstellungen steht, startet nie
        s.backend = Some("hw:gibt-es-nicht".into());
        s.start(PathBuf::from("/p"), PathBuf::from("/d"), Only::All, false, &egui::Context::default());
        assert!(s.suite.error.as_deref().unwrap_or("").contains("nicht mehr in den Einstellungen"), "{:?}", s.suite.error);
        assert!(!s.suite.running);
    }

    #[test]
    fn confirmation_dialog_draws_for_both_kinds_of_hardware() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut s = TestsState::default();
        for hw in [Hardware::Ultimate { host: "192.168.1.50".into(), password: None, name: "Wohnzimmer".into() }, Hardware::Mega65 { spec: "serial:/dev/cu.usbserial-1@2000000".into(), name: String::new() }] {
            let mut r = None;
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, 600.0))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| { r = s.confirm_dialog(ui.ctx(), &hw); });
            out.textures_delta.clear();
            assert_eq!(r, None, "ohne Klick bleibt der Dialog offen");
        }
        assert!(Hardware::Ultimate { host: "h".into(), password: None, name: String::new() }.label().starts_with("Ultimate 64"));
    }

    #[test]
    fn seconds_and_labels() {
        assert_eq!(seconds(850), "850 ms");
        assert_eq!(seconds(1800), "1,8 s");
        let o = Outcome::failed("x", Level::Hint, "");
        assert_eq!(state_label(&o), "Hinweis");
        assert_eq!(state_label(&Outcome::timed_out("x", "")), "Zeitüberschreitung");
        assert_eq!(state_label(&Outcome::skipped("x", "")), "übersprungen");
    }
}
