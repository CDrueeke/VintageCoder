// SPDX-License-Identifier: GPL-3.0-or-later
//! Schritt „Zusammenfassung“: Änderungsliste (Neu / Geändert / Entfernt / Unverändert), je Eintrag abwählbar, dazu Zielorte und Sicherheitsnetz.
//! Passwörter stehen nie im Klartext in der Liste (`vc_ide::setup::diff` maskiert sie).
use crate::setup_wizard::{widgets, SetupWizard};
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip};
use eframe::egui::{self, Color32, RichText};
use vc_ide::setup::{self, Change, Kind};

fn kind_style(kind: Kind) -> (&'static str, &'static str, Color32) {
    let k = t();
    match kind { Kind::New => ("+", "Neu", k.cy), Kind::Changed => ("~", "Geändert", k.am), Kind::Removed => ("−", "Entfernt", k.rd), Kind::Same => ("=", "Unverändert", k.mute) }
}

impl SetupWizard {
    pub(crate) fn step_summary(&mut self, ui: &mut egui::Ui) {
        let k = t();
        if let Some(e) = &self.apply_error { widgets::note(ui, "!", e, k.rd); ui.add_space(4.0); }
        let keys = self.selected_keys();
        let (_, _, merge_notes) = setup::merge(&self.cur, &self.draft, &|k| keys.contains(k));
        for n in merge_notes.iter().chain(self.notes.iter()) { widgets::note(ui, "!", n, k.am); }
        let total = self.changes.iter().filter(|c| c.kind != Kind::Same).count();
        let mut toggle: Option<(String, bool)> = None;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width((ui.available_width() - 280.0).max(360.0));
                if total == 0 { card(ui, false, |ui| { ui.label(RichText::new("Nichts zu ändern. Deine Einstellungen bleiben, wie sie sind.").color(k.mute)); }); }
                for kind in [Kind::New, Kind::Changed, Kind::Removed] {
                    let items: Vec<Change> = self.changes.iter().filter(|c| c.kind == kind).cloned().collect();
                    if items.is_empty() { continue; }
                    let (sym, name, col) = kind_style(kind);
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{sym} {name} ({})", items.len())).strong().color(col));
                    for c in items { if let Some(t) = change_row(ui, &c, col) { toggle = Some(t); } }
                }
                let same: Vec<Change> = self.changes.iter().filter(|c| c.kind == Kind::Same && !matches!(c.key.as_str(), "streams" | "assembler" | "roms:use_foreign")).cloned().collect();
                if !same.is_empty() {
                    ui.add_space(6.0);
                    ui.collapsing(RichText::new(format!("= Unverändert ({})", same.len())).color(k.mute), |ui| {
                        for c in same { ui.horizontal(|ui| { ui.label(RichText::new(c.area).monospace().small().color(k.dim)); ui.label(RichText::new(&c.label).color(k.mute)); if !c.new.is_empty() { ui.label(RichText::new(&c.new).small().color(k.dim)); } else if !c.old.is_empty() { ui.label(RichText::new(&c.old).small().color(k.dim)); } }); }
                    });
                }
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.set_width(260.0);
                card(ui, false, |ui| {
                    ui.label(RichText::new("Wird geschrieben nach").strong());
                    ui.label(RichText::new(self.paths.settings.display().to_string()).monospace().small());
                    if keys.iter().any(|k| k.starts_with("view:")) { ui.label(RichText::new(self.paths.view.display().to_string()).monospace().small()); }
                    if keys.contains("examples") { ui.label(RichText::new(setup::examples_dest_in(&self.draft.examples.dest, &setup::effective_projects_dir(&self.draft.settings)).display().to_string()).monospace().small()); }
                    ui.label(RichText::new(self.paths.wizard.display().to_string()).monospace().small().color(k.mute));
                });
                ui.add_space(6.0);
                card(ui, false, |ui| {
                    ui.label(RichText::new("Sicherheitsnetz").strong());
                    for line in [format!("Sicherung: {}", self.paths.backup().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()), "Rückgängig bis zum Neustart".to_string(), "Passwörter verborgen".to_string(), "Export jederzeit möglich".to_string()] { ui.label(RichText::new(format!("✓ {line}")).small().color(k.gr)); }
                    ui.label(RichText::new("Schreiben ist atomar: bei einem Fehler bleibt alles wie vorher.").small().color(k.mute));
                });
                if self.recall { ui.add_space(6.0); chip(ui, "Wiederaufruf", k.cy); }
            });
        });
        if let Some((key, on)) = toggle { if on { self.deselected.remove(&key); } else { self.deselected.insert(key); } self.refresh_status(true); }
    }
}

/// Eine Zeile der Änderungsliste mit Haken; liefert (Schlüssel, neuer Zustand) bei Klick
fn change_row(ui: &mut egui::Ui, c: &Change, col: Color32) -> Option<(String, bool)> {
    let k = t();
    let mut on = c.selected;
    let mut out = None;
    ui.horizontal_wrapped(|ui| {
        if ui.checkbox(&mut on, "").changed() { out = Some((c.key.clone(), on)); }
        ui.label(RichText::new(c.area).monospace().small().color(k.dim));
        ui.label(RichText::new(&c.label).color(if on { k.fg } else { k.mute }));
        if !c.old.is_empty() && !c.new.is_empty() { ui.label(RichText::new(format!("{} → {}", c.old, c.new)).small().color(col)); }
        else if !c.new.is_empty() { ui.label(RichText::new(&c.new).small().color(col)); }
        else if !c.old.is_empty() { ui.label(RichText::new(&c.old).small().color(col)); }
        if c.sensitive { ui.label(RichText::new("Passwort verborgen").small().color(k.mute)); }
        if !on { ui.label(RichText::new("– nicht übernehmen").small().color(k.dim)); }
    });
    if !c.note.is_empty() { ui.label(RichText::new(format!("      {}", c.note)).small().color(k.mute)); }
    out
}
