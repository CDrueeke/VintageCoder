// SPDX-License-Identifier: GPL-3.0-or-later
//! Schritte „Editor“ (Schrift, Adressspalte, feste Kürzel) und „Projekte“ (Projektordner, Beispiele, erstes Projekt, Kettentest).
use crate::setup_wizard::{widgets, Env, First, SetupWizard};
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip, section, system_color};
use eframe::egui::{self, RichText};
use std::time::{Duration, Instant};
use vc_ide::setup::{self, Mode, ProjectsDirState};

impl SetupWizard {
    pub(crate) fn step_editor(&mut self, ui: &mut egui::Ui, env: &mut Env) {
        let k = t();
        section(ui, "Tastenbelegung", "Die Kürzel sind fest; Anpassen ist noch nicht möglich.", |_| {});
        card(ui, false, |ui| {
            ui.horizontal(|ui| {
                chip(ui, "Standard", k.cy);
                chip(ui, "Wie moderne Editoren: braucht anpassbare Kürzel", k.dim);
                chip(ui, "Klassisch: braucht anpassbare Kürzel", k.dim);
            });
            ui.label(RichText::new(format!("Plattformrichtig: {} statt {}.", if cfg!(target_os = "macos") { "Cmd" } else { "Strg" }, if cfg!(target_os = "macos") { "Strg" } else { "Cmd" })).small().color(k.mute));
        });
        ui.add_space(6.0);
        crate::settings_ui::pages::keys(ui);
        ui.add_space(8.0);
        let mut senv = crate::settings_ui::Env { view: env.view, show_addr: env.show_addr, data: env.data, project: None };
        crate::settings_ui::pages::editor(ui, &mut senv);
        ui.label(RichText::new("Vervollständigung öffnet mit Strg+Leertaste.").small().color(k.mute));
    }

    pub(crate) fn step_projects(&mut self, ui: &mut egui::Ui) {
        let k = t();
        section(ui, "Projektordner", "Hier liegen deine Projekte; der Projekt-Assistent schlägt ihn als Ort vor.", |_| {});
        card(ui, false, |ui| {
            let mut dir = if self.draft.settings.projects_dir.trim().is_empty() { String::new() } else { self.draft.settings.projects_dir.clone() };
            let hint = setup::effective_projects_dir(&self.cur.settings);
            ui.horizontal(|ui| {
                ui.label("Basisverzeichnis");
                if crate::ui::widgets::path_field(ui, &mut dir, &hint, true) { self.draft.settings.projects_dir = dir.clone(); }
            });
            let eff = setup::effective_projects_dir(&self.draft.settings);
            let state = self.projects_dir_state(&eff);
            ui.horizontal(|ui| {
                match &state { Ok(ProjectsDirState::Writable) => { chip(ui, "✓ schreibbar", k.gr); } Ok(ProjectsDirState::WillCreate) => { chip(ui, "+ Ordner wird neu angelegt", k.cy); } Err(e) => { chip(ui, "! ungültig", k.rd); ui.label(RichText::new(e).small().color(k.rd)); } }
                if self.draft.settings.projects_dir.trim().is_empty() { ui.label(RichText::new("Vorschlag, noch nicht gespeichert").small().color(k.mute)); }
            });
            if eff.starts_with("//") || eff.starts_with("/Volumes/") || eff.starts_with("\\\\") { widgets::note(ui, "i", "Der Ordner liegt vermutlich auf einem Netzlaufwerk; das kann langsam sein.", k.am); }
            if self.recall { ui.label(RichText::new("Den Ordner zu ändern verschiebt nichts; nur die Voreinstellung für neue Projekte ändert sich.").small().color(k.mute)); }
        });
        ui.add_space(8.0);
        section(ui, "Beispiele", "Die mitgelieferten Beispiele kopieren, standardmäßig nach <Projektordner>/Beispiele (Zielordner frei wählbar). Kopiert wird erst bei „Übernehmen“, vorhandene Dateien bleiben unberührt.", |_| {});
        card(ui, false, |ui| {
            let has = self.draft.examples.source.is_some();
            ui.add_enabled(has, egui::Checkbox::new(&mut self.draft.examples.copy, "Mitgelieferte Beispiele kopieren")).on_disabled_hover_text("Der Ordner examples/ wurde neben der Installation nicht gefunden");
            if !has { ui.label(RichText::new("Beispiele nicht gefunden (Ordner examples/ fehlt).").small().color(k.am)); return; }
            let all = self.draft.examples.source.as_ref().map(|r| setup::list_examples(r, &self.systems())).unwrap_or_default();
            ui.horizontal_wrapped(|ui| { for s in self.systems() { if let Some(i) = vc_ide::targets::system(&s) { let n = all.iter().filter(|e| e.system == s).count(); chip(ui, &format!("{} · {n}", i.badge), system_color(&s)); } } });
            let base = setup::effective_projects_dir(&self.draft.settings);
            let mut custom = self.draft.examples.dest.clone();
            ui.horizontal(|ui| {
                ui.label("Zielordner");
                if crate::ui::widgets::path_field(ui, &mut custom, &setup::examples_dest(&base).to_string_lossy(), true) { self.draft.examples.dest = custom.clone(); }
            });
            let dest = setup::examples_dest_in(&self.draft.examples.dest, &base);
            let missing = all.iter().filter(|i| !self.draft.examples.excluded.contains(&i.id()) && !dest.join(setup::example_dir_name(&i.system)).join(&i.name).exists()).count();
            ui.label(RichText::new(format!("{} Beispiele für deine Systeme; {missing} fehlen im Zielordner{}.", all.len(), if self.recall { " (erneutes Kopieren ergänzt nur Fehlendes)" } else { "" })).small().color(k.mute));
            ui.collapsing("Auswahl …", |ui| {
                for it in &all {
                    let mut on = !self.draft.examples.excluded.contains(&it.id());
                    if ui.checkbox(&mut on, it.id()).changed() { if on { self.draft.examples.excluded.remove(&it.id()); } else { self.draft.examples.excluded.insert(it.id()); } }
                }
            });
        });
        if self.mode != Mode::Full { return; }
        ui.add_space(8.0);
        section(ui, "Erstes Projekt", "Der Wertmoment: etwas läuft. Die Auswahl gilt nach „Fertig“.", |_| {});
        let examples: Vec<setup::ExampleItem> = self.draft.examples.source.as_ref().map(|r| setup::list_examples(r, &self.systems())).unwrap_or_default();
        let w = ((ui.available_width() - 20.0) / 3.0).max(190.0);
        ui.horizontal_top(|ui| {
            let ex_sel = matches!(self.first, First::Example(_));
            if widgets::choice_card(ui, ex_sel, w, |ui| {
                ui.horizontal(|ui| { ui.label(RichText::new("Beispiel öffnen").strong()); chip(ui, "empfohlen", k.gr); });
                if let First::Example(cur) = &mut self.first {
                    egui::ComboBox::from_id_salt("first_example").selected_text(cur.clone()).show_ui(ui, |ui| { for e in &examples { ui.selectable_value(cur, e.id(), e.id()); } });
                } else { ui.label(RichText::new("z. B. c64/jumprun").small().color(k.mute)); }
            }).clicked() && !ex_sel { self.first = First::Example(examples.iter().find(|e| e.name == "jumprun").or(examples.first()).map(|e| e.id()).unwrap_or_default()); }
            if widgets::choice_card(ui, self.first == First::New, w, |ui| { ui.label(RichText::new("Leeres Projekt anlegen").strong()); ui.label(RichText::new("Öffnet danach den Projekt-Assistenten.").small().color(k.mute)); }).clicked() { self.first = First::New; }
            if widgets::choice_card(ui, self.first == First::Later, w, |ui| { ui.label(RichText::new("Später").strong()); ui.label(RichText::new("Nur einrichten.").small().color(k.mute)); }).clicked() { self.first = First::Later; }
        });
        ui.add_space(6.0);
        card(ui, false, |ui| {
            ui.label(RichText::new("Kettentest").strong());
            ui.label(RichText::new("Der Assistent baut und startet nichts von selbst. Nach „Fertig“ öffnet das Beispiel; dann prüfst du die Kette selbst:").small().color(k.mute));
            let copy = self.draft.examples.copy;
            for (n, text, ok) in [("①", "Beispiel kopieren (bei „Übernehmen“)", copy), ("②", "Assemblieren (F7)", false), ("③", "Im Standard-Emulator starten (F5)", false), ("④", "Bild und Ton prüfen (du bestätigst)", false)] {
                ui.horizontal(|ui| { ui.label(RichText::new(n).monospace().color(if ok { k.gr } else { k.mute })); ui.label(RichText::new(text).color(if ok { k.fg } else { k.mute })); });
            }
        });
    }

    /// Prüfung des Projektordners, höchstens jede halbe Sekunde (Dateisystem)
    pub(crate) fn projects_dir_state(&mut self, dir: &str) -> Result<ProjectsDirState, String> {
        if let Some((d, r, at)) = &self.proj_check { if d == dir && at.elapsed() < Duration::from_millis(500) { return r.clone(); } }
        let r = setup::check_projects_dir(dir, self.install_dir.as_deref());
        self.proj_check = Some((dir.to_string(), r.clone(), Instant::now()));
        r
    }
}
