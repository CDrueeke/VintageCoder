// SPDX-License-Identifier: GPL-3.0-or-later
//! Schritte „Willkommen“ (Weg wählen, im Wiederaufruf: Stand, Neufunde, Werkzeuge), „Design“ und „Fertig“.
use crate::setup_wizard::{side, widgets, Env, First, Out, SetupWizard};
use crate::theme::Scheme;
use crate::ui::tokens::{contrast_rating, t, Tokens, DARK, LIGHT};
use crate::ui::widgets::{card, chip, section};
use eframe::egui::{self, RichText};
use vc_ide::setup::{self, Mode, Part, Topic};

impl SetupWizard {
    pub(crate) fn step_welcome(&mut self, ui: &mut egui::Ui) {
        let k = t();
        if !self.recall {
            let w = ((ui.available_width() - 16.0) / 2.0).max(280.0);
            ui.horizontal_top(|ui| {
                for (mode, title, sub, steps, mins, rec) in [(Mode::Quick, "Schnelleinrichtung", "Systeme, Toolchain und Projektordner; der Rest läuft mit sinnvollen Voreinstellungen.", "7 Schritte", "ca. 2 Minuten", true), (Mode::Full, "Alles selbst einstellen", "Zusätzlich Parameter, ROMs, echte Hardware, Startziel F5, Editor und erstes Projekt.", "12 Schritte", "ca. 6 Minuten", false)] {
                    let sel = self.mode == mode;
                    let r = widgets::choice_card(ui, sel, w, |ui| {
                        ui.horizontal(|ui| { ui.label(RichText::new(title).font(egui::FontId::new(16.0, crate::fonts::ui_strong())).color(k.fg)); if rec { chip(ui, "empfohlen", k.gr); } });
                        ui.label(RichText::new(sub).color(k.mute));
                        ui.add_space(4.0);
                        ui.label(RichText::new(format!("{steps} · {mins}")).monospace().small().color(k.cy));
                        ui.label(RichText::new(if sel { "● gewählt" } else { "○ wählen" }).small().color(if sel { k.cy } else { k.mute }));
                    });
                    if r.clicked() { self.mode = mode; self.steps = mode.steps(); }
                }
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (head, text) in [("Nichts wird gesendet", "Kein Netzwerkzugriff, keine Telemetrie."), ("Kein Download ohne Ja", "Der Assistent installiert und lädt nichts."), ("Alles ist änderbar", "Auch später: Menü Hilfe ▸ Konfigurationsassistent.")] {
                    egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(12)).show(ui, |ui| { ui.set_width(210.0); ui.vertical(|ui| { ui.label(RichText::new(format!("✓ {head}")).strong().color(k.gr)); ui.label(RichText::new(text).small().color(k.mute)); }); });
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new("Oberfläche: Deutsch (ein Sprachwähler folgt, sobald es eine zweite Sprache gibt).").small().color(k.mute));
            ui.add_space(10.0);
            widgets::ready_screen(ui, &["**** VINTAGECODER ****", "", "READY."], 360.0, !ui.ctx().input(|i| i.time).fract().lt(&0.5));
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
            return;
        }
        self.welcome_recall(ui);
    }

    fn welcome_recall(&mut self, ui: &mut egui::Ui) {
        let k = t();
        section(ui, "Aktuelle Konfiguration", "Gespeicherte Werte stehen oben und führen. Neues wird nur vorgeschlagen.", |_| {});
        card(ui, false, |ui| {
            egui::Grid::new("recall_info").num_columns(2).spacing([14.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new("Quelle").color(k.mute)); ui.label(RichText::new(self.paths.settings.display().to_string()).monospace().small()); ui.end_row();
                ui.label(RichText::new("Format").color(k.mute)); ui.label(format!("Version {}", self.cur.settings.schema_version.max(setup::SCHEMA_VERSION))); ui.end_row();
                ui.label(RichText::new("Letzter Lauf").color(k.mute)); ui.label(if self.wiz.last_run.is_empty() { "noch keiner".to_string() } else { self.wiz.last_run.clone() }); ui.end_row();
                ui.label(RichText::new("Systeme").color(k.mute)); ui.label(self.systems().iter().filter_map(|s| vc_ide::targets::system(s).map(|i| i.name)).collect::<Vec<_>>().join(", ")); ui.end_row();
            });
        });
        // Probleme
        if !self.problems.is_empty() {
            ui.add_space(8.0);
            card(ui, false, |ui| {
                ui.label(RichText::new("Probleme im gespeicherten Stand").strong().color(k.rd));
                for p in self.problems.clone() {
                    ui.horizontal(|ui| { ui.label(RichText::new("! ").strong().color(k.rd)); ui.label(&p.text); if ui.small_button("Schritt öffnen").clicked() { self.jump_to(p.topic); } });
                }
            });
        }
        // Neufunde: nur vorschlagen
        if !self.findings.is_empty() {
            ui.add_space(8.0);
            card(ui, false, |ui| {
                ui.label(RichText::new("Neu gefunden").strong().color(k.cy));
                ui.label(RichText::new("Nichts wird still übernommen. „Übernehmen“ legt den Eintrag in den Entwurf; geschrieben wird erst in der Zusammenfassung.").small().color(k.mute));
                for f in self.findings.clone() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("+ ").strong().color(k.cy)); ui.label(&f.text);
                        if ui.small_button("Übernehmen").clicked() { setup::accept_finding(&mut self.draft, &f); self.refresh_status(true); }
                        if ui.small_button("Ignorieren").on_hover_text("Kommt nicht wieder").clicked() { self.wiz.dismiss(&f.id); if !self.shot { let _ = self.wiz.save(&self.paths.wizard); } self.refresh_status(true); }
                    });
                }
            });
        }
        ui.add_space(8.0);
        section(ui, "Nur einen Teil einrichten", "Alles andere bleibt nachweislich unverändert.", |_| {});
        ui.horizontal_wrapped(|ui| {
            for (p, label) in [(Part::Roms, "Nur ROMs"), (Part::Emulators, "Nur Emulatoren"), (Part::Hardware, "Nur echte Hardware"), (Part::DesignEditor, "Nur Design und Editor")] {
                if ui.button(label).clicked() { self.mode = Mode::Part(p); self.steps = self.mode.steps(); self.idx = 0; self.reached = 0; self.refresh_status(true); }
            }
        });
        ui.add_space(8.0);
        section(ui, "Sichern, übertragen, zurücksetzen", "", |_| {});
        ui.horizontal_wrapped(|ui| {
            if ui.button("Konfiguration exportieren …").clicked() { self.open_export(); }
            if ui.button("Konfiguration importieren …").clicked() { self.side = side::Side::Import { msg: String::new() }; }
            if ui.button("Auf Standard zurücksetzen …").clicked() { self.side = side::Side::Reset { area: setup::ResetArea::All, with_hw: false }; }
        });
        if self.mode == Mode::Full { ui.add_space(6.0); ui.label(RichText::new("Mit „Weiter“ gehst du alle Schritte durch; Schritte ohne Handlungsbedarf bestätigst du mit einem Klick.").small().color(k.mute)); }
    }

    pub(crate) fn step_design(&mut self, ui: &mut egui::Ui, env: &mut Env) {
        let k = t();
        ui.horizontal_top(|ui| {
            for (sc, label, sub) in [(Scheme::Dark, "Dunkel", "Voreinstellung"), (Scheme::Light, "Hell", "für helle Räume"), (Scheme::System, "System", "folgt dem Betriebssystem, sonst Dunkel")] {
                let sel = env.view.scheme == sc;
                let r = widgets::choice_card(ui, sel, 210.0, |ui| {
                    scheme_preview(ui, sc);
                    ui.label(RichText::new(label).strong().color(k.fg));
                    ui.label(RichText::new(sub).small().color(k.mute));
                });
                if r.clicked() { env.view.scheme = sc; }
            }
        });
        ui.add_space(10.0);
        card(ui, false, |ui| {
            ui.horizontal(|ui| { ui.label("Oberfläche"); crate::theme::zoom_slider(ui, &mut env.view.zoom); });
            ui.horizontal(|ui| { ui.label("Editorschrift"); ui.add(egui::Slider::new(&mut env.view.font, crate::theme::FONT_RANGE.0..=crate::theme::FONT_RANGE.1).step_by(1.0).suffix(" pt")); });
            ui.label(RichText::new("Schriftart: die eingebaute (weitere Schriften sind nicht vorgesehen). Status steht immer mit Symbol und Text, nie nur als Farbe.").small().color(k.mute));
        });
        ui.add_space(8.0);
        card(ui, false, |ui| {
            ui.label(RichText::new("Vorschau").small().color(k.mute));
            let m = |s: &str, c| RichText::new(s).monospace().size(env.view.font).color(c);
            let sy = crate::ui::tokens::syntax();
            ui.horizontal(|ui| { ui.label(m("start:", sy.label)); ui.label(m("lda", sy.opcode)); ui.label(m("#$00", sy.number)); ui.label(m("; Rahmen schwarz", sy.comment)); });
            ui.horizontal(|ui| { ui.label(m("        sta", sy.opcode)); ui.label(m("$d020", sy.number)); });
        });
        ui.add_space(8.0);
        let dark_now = match env.view.scheme { Scheme::Dark => true, Scheme::Light => false, Scheme::System => ui.ctx().global_style().visuals.dark_mode };
        let tok: &Tokens = if dark_now { &DARK } else { &LIGHT };
        card(ui, false, |ui| {
            ui.label(RichText::new(format!("Lesbarkeit ({})", if dark_now { "dunkel" } else { "hell" })).strong());
            for (name, c) in [("Text auf Fläche", tok.fg), ("Gedämpfter Text", tok.mute), ("Primärfarbe", tok.cy), ("Hinweisfarbe", tok.am), ("Fehlerfarbe", tok.rd), ("Erfolgsfarbe", tok.gr)] {
                let (ratio, text, col) = contrast_rating(c, tok.bg3);
                ui.horizontal(|ui| { ui.label(RichText::new(name).color(c)); ui.label(RichText::new(format!("{ratio:.1}:1")).monospace().small()); ui.label(RichText::new(text).small().strong().color(match col { 0 => k.gr, 1 => k.am, _ => k.rd })); });
            }
        });
        ui.add_space(6.0);
        ui.label(RichText::new("„Bewegung reduzieren“: der Assistent animiert nur sparsam (Fortschrittsleiste, Cursor); ein eigener Schalter ist noch nicht vorgesehen.").small().color(k.mute));
    }

    pub(crate) fn step_done(&mut self, ui: &mut egui::Ui, out: &mut Out) {
        let k = t();
        if let Some(e) = &self.apply_error { widgets::note(ui, "!", e, k.rd); }
        for n in self.notes.clone() { widgets::note(ui, "!", &n, k.am); }
        // Systeme: bereit oder teilweise
        ui.horizontal_wrapped(|ui| {
            for sys in self.systems() {
                let Some(info) = vc_ide::targets::system(&sys) else { continue };
                let emu = self.draft.settings.target(&sys).and_then(|t| t.standard_emulator()).cloned();
                let found = emu.as_ref().map_or(false, |e| if e.path.trim().is_empty() { self.found(&e.kind).is_some() } else { vc_ide::settings::check_program(&e.path).is_ok() });
                card(ui, false, |ui| {
                    ui.set_width(200.0);
                    ui.horizontal(|ui| { widgets::silhouette(ui, &sys, egui::vec2(46.0, 30.0)); ui.label(RichText::new(info.name).strong()); });
                    if found { widgets::note(ui, "✓", "bereit", k.gr); } else { widgets::note(ui, "–", "teilweise: Emulator fehlt, bauen geht", k.am); }
                });
            }
        });
        ui.add_space(8.0);
        let w = ((ui.available_width() - 30.0) / 4.0).max(150.0);
        ui.horizontal_top(|ui| {
            let ex = self.first_example_path();
            if widgets::choice_card(ui, false, w, |ui| { ui.label(RichText::new("Beispiel starten").strong()); ui.label(RichText::new(if ex.is_some() { "Öffnet ein Beispiel; F5 startet es." } else { "Kein Beispiel gefunden." }).small().color(k.mute)); }).clicked() { if let Some(p) = ex { *out = Out::OpenProject(p); } }
            if widgets::choice_card(ui, false, w, |ui| { ui.label(RichText::new("Neues Projekt").strong()); ui.label(RichText::new("Der Projekt-Assistent öffnet sich.").small().color(k.mute)); }).clicked() { *out = Out::NewProject; }
            if widgets::choice_card(ui, false, w, |ui| { ui.label(RichText::new("Handbuch öffnen").strong()); ui.label(RichText::new("Überblick und Einstellungen.").small().color(k.mute)); }).clicked() { *out = Out::Help("start"); }
            if widgets::choice_card(ui, false, w, |ui| { ui.label(RichText::new("Assistent erneut starten").strong()); ui.label(RichText::new("Mit Statusmarken je Schritt.").small().color(k.mute)); }).clicked() {
                self.recall = true; self.mode = Mode::Full; self.steps = self.mode.steps(); self.idx = 0; self.reached = 0; self.applied = None; self.draft = setup::Draft { examples: self.draft.examples.clone(), ..setup::Draft::from_snapshot(&self.cur) }; self.refresh_status(true);
            }
        });
        ui.add_space(8.0);
        card(ui, false, |ui| {
            ui.label(RichText::new("Konfiguration sichern").strong());
            ui.label(RichText::new("Der Export enthält keine Passwörter, außer du wählst es; vorher wirst du gefragt.").small().color(k.mute));
            if ui.button("Exportieren …").clicked() { self.open_export(); }
        });
        if let Some(a) = &self.applied { ui.add_space(6.0); ui.label(RichText::new(format!("Gesichert in {}. „Rückgängig“ (unten) stellt den Stand bis zum Neustart wieder her.", a.backup.as_ref().map_or("–".to_string(), |b| b.display().to_string()))).small().color(k.mute)); }
        ui.add_space(8.0);
        widgets::ready_screen(ui, &["READY."], 300.0, !ui.ctx().input(|i| i.time).fract().lt(&0.5));
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    }

    /// Pfad des ersten kopierten (sonst mitgelieferten) Beispiels der gewählten Systeme
    pub(crate) fn first_example_path(&self) -> Option<std::path::PathBuf> {
        let sys = self.systems();
        let root = self.draft.examples.source.as_ref()?;
        let dest = setup::examples_dest_in(&self.draft.examples.dest, &setup::effective_projects_dir(&self.cur.settings));
        let pick = match &self.first { First::Example(id) => id.clone(), _ => String::new() };
        let items = setup::list_examples(root, &sys);
        let it = items.iter().find(|i| i.id() == pick).or_else(|| items.iter().find(|i| i.name == "jumprun")).or_else(|| items.first())?;
        let copy = dest.join(setup::example_dir_name(&it.system)).join(&it.name);
        Some(if copy.is_dir() { copy } else { it.src.clone() })
    }
}

fn scheme_preview(ui: &mut egui::Ui, sc: Scheme) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(180.0, 64.0), egui::Sense::hover());
    let p = ui.painter();
    let draw = |r: egui::Rect, k: &Tokens| {
        p.rect_filled(r, 4.0, k.bg1);
        p.rect_filled(egui::Rect::from_min_size(r.min + egui::vec2(6.0, 6.0), egui::vec2(r.width() * 0.4, 8.0)), 2.0, k.cy);
        p.rect_filled(egui::Rect::from_min_size(r.min + egui::vec2(6.0, 20.0), egui::vec2(r.width() - 12.0, 14.0)), 2.0, k.bg2);
        p.text(r.min + egui::vec2(10.0, 27.0), egui::Align2::LEFT_CENTER, "lda #$00", egui::FontId::monospace(10.0), k.fg);
        p.rect_filled(egui::Rect::from_min_size(r.min + egui::vec2(6.0, 40.0), egui::vec2(r.width() * 0.25, 8.0)), 2.0, k.gr);
        p.rect_filled(egui::Rect::from_min_size(r.min + egui::vec2(r.width() * 0.25 + 12.0, 40.0), egui::vec2(r.width() * 0.2, 8.0)), 2.0, k.am);
    };
    match sc {
        Scheme::Dark => draw(rect, &DARK),
        Scheme::Light => draw(rect, &LIGHT),
        Scheme::System => { let (l, r) = (egui::Rect::from_min_max(rect.min, egui::pos2(rect.center().x, rect.max.y)), egui::Rect::from_min_max(egui::pos2(rect.center().x, rect.min.y), rect.max)); draw(rect, &DARK); p.with_clip_rect(r).rect_filled(r, 0.0, LIGHT.bg1); let _ = l; p.with_clip_rect(r).text(r.min + egui::vec2(10.0, 27.0), egui::Align2::LEFT_CENTER, "lda", egui::FontId::monospace(10.0), LIGHT.fg); }
    }
}

#[allow(dead_code)]
fn _unused(_: Topic) {}
