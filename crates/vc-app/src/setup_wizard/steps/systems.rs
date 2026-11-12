// SPDX-License-Identifier: GPL-3.0-or-later
//! Schritte „Zielsysteme“ (Kacheln mit Silhouetten, Erkennungschip) und „Toolchain“ (Erkennung, Installationshinweise zum Kopieren).
use crate::setup_wizard::{widgets, SetupWizard};
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip, system_color};
use eframe::egui::{self, RichText};
use vc_ide::setup;
use vc_ide::targets::SYSTEMS;

impl SetupWizard {
    pub(crate) fn step_systems(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let mut toggle: Option<(&'static str, bool)> = None;
        let cols = (((ui.available_width() + 10.0) / 280.0).floor() as usize).max(1);
        let mut groups: Vec<&'static str> = vec![];
        let systems = vc_ide::targets::installed_systems(self.data_dir.as_deref());
        for info in &systems { if !groups.contains(&info.group) { groups.push(info.group); } }
        for group in groups {
            let members: Vec<_> = systems.iter().filter(|s| s.group == group).collect();
            let on_count = members.iter().filter(|s| self.draft.settings.target(s.id).is_some()).count();
            ui.add_space(4.0);
            ui.label(RichText::new(format!("{} · {on_count} von {}", group.to_uppercase(), members.len())).monospace().small().color(k.dim));
            ui.add_space(2.0);
            for row in members.chunks(cols) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                    for info in row {
                        let on = self.draft.settings.target(info.id).is_some();
                        let hidden = self.draft.settings.hidden_targets.iter().any(|h| h.system == info.id);
                        let kind = setup::offered_kinds(info.id).first().map(|k| k.id).unwrap_or("");
                        let found = if kind.is_empty() { None } else { self.found(kind) };
                        let r = widgets::choice_card(ui, on, 270.0, |ui| {
                            ui.horizontal(|ui| {
                                widgets::silhouette(ui, info.id, egui::vec2(64.0, 42.0));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(info.name).strong().color(system_color(info.id).lerp_to_gamma(k.fg, 0.5)));
                                    ui.label(RichText::new(info.cpu).small().color(k.mute));
                                });
                            });
                            ui.horizontal(|ui| {
                                let mut v = on;
                                if ui.checkbox(&mut v, if on { "verwenden" } else { "nicht verwenden" }).changed() { toggle = Some((info.id, v)); }
                                if kind.is_empty() { chip(ui, "kein Emulator hier", k.dim); }
                                else if found.is_some() { chip(ui, &format!("{} gefunden", vc_ide::targets::kind_label(kind).split(' ').next().unwrap_or("")), k.gr); }
                                else { chip(ui, &format!("{} fehlt", vc_ide::targets::kind_label(kind).split(' ').next().unwrap_or("")), k.am); }
                            });
                            if hidden && !on { ui.label(RichText::new("ausgeblendet · Einträge bleiben erhalten").small().color(k.mute)); }
                        });
                        if r.clicked() && toggle.is_none() { toggle = Some((info.id, !on)); }
                    }
                });
                ui.add_space(6.0);
            }
        }
        if let Some((id, on)) = toggle { setup::set_system(&mut self.draft.settings, id, on); self.refresh_status(true); }
        if self.draft.settings.targets.is_empty() { ui.add_space(6.0); widgets::note(ui, "!", "Ohne System lässt sich nichts bauen: wähle mindestens eines.", k.am); }
        else if self.recall { ui.add_space(6.0); ui.label(RichText::new("Abgewählte Systeme werden ausgeblendet; ihre Emulator- und Hardwareeinträge bleiben erhalten (die Zusammenfassung zeigt sie als „Entfernt“).").small().color(k.mute)); }
    }

    pub(crate) fn step_toolchain(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let rows = setup::toolchain_rows(&self.systems());
        if self.tool.is_empty() || !rows.iter().any(|r| r.kind == self.tool) { self.tool = rows.iter().find(|r| !r.builtin && self.found(&r.kind).is_none()).or_else(|| rows.iter().find(|r| !r.builtin)).map(|r| r.kind.clone()).unwrap_or_default(); }
        card(ui, false, |ui| {
            egui::Grid::new("tool_grid").num_columns(5).spacing([16.0, 6.0]).striped(true).show(ui, |ui| {
                for h in ["Programm", "Für", "Status", "Fundort", ""] { ui.label(RichText::new(h.to_uppercase()).monospace().small().color(k.dim)); }
                ui.end_row();
                for r in &rows {
                    let found = if r.builtin { Some("eingebaut".to_string()) } else { self.configured_or_found(&r.kind) };
                    ui.label(RichText::new(&r.name).strong());
                    ui.label(RichText::new(r.used_for.join(", ")).small().color(k.mute));
                    if r.builtin { chip(ui, "eingebaut", k.gr); } else if found.is_some() { chip(ui, "✓ gefunden", k.gr); } else { chip(ui, "– fehlt", k.am); }
                    ui.add(egui::Label::new(RichText::new(found.clone().unwrap_or_else(|| "nicht gefunden".into())).monospace().small().color(if found.is_some() { k.fg } else { k.mute })).truncate());
                    if !r.builtin && ui.selectable_label(self.tool == r.kind, if found.is_some() { "Details" } else { "Hinweis" }).clicked() { self.tool = r.kind.clone(); }
                    ui.end_row();
                }
                ui.label(RichText::new("Java").color(k.dim)); ui.label(RichText::new("–").color(k.dim)); chip(ui, "nicht nötig", k.dim); ui.label(RichText::new("Der Assembler ist eingebaut (vc-asm).").small().color(k.dim)); ui.label(""); ui.end_row();
            });
        });
        ui.add_space(8.0);
        let Some(row) = rows.iter().find(|r| r.kind == self.tool).cloned() else { return };
        self.install_hint(ui, &row);
    }

    /// Eingetragener Pfad (wenn gültig) oder Fundort der automatischen Suche
    fn configured_or_found(&mut self, kind: &str) -> Option<String> {
        let configured = self.draft.settings.targets.iter().flat_map(|t| t.emulators.iter()).find(|e| e.kind == kind && !e.path.trim().is_empty()).map(|e| e.path.clone());
        match configured { Some(p) => vc_ide::settings::check_program(&p).ok().map(|_| vc_ide::settings::program_exe(&p).to_string_lossy().into_owned()), None => self.found(kind) }
    }

    fn install_hint(&mut self, ui: &mut egui::Ui, row: &setup::ToolRow) {
        let k = t();
        let found = self.configured_or_found(&row.kind).is_some();
        card(ui, false, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}: {}", row.name, if found { "gefunden" } else { "Installationshinweis" })).strong());
            });
            ui.label(RichText::new("Der Assistent installiert nichts. Kopiere den Befehl und führe ihn selbst in einem Terminal aus.").small().color(k.mute));
            let mut os = self.tab_os.clone();
            crate::ui::widgets::segmented(ui, &mut os, &[("macos", "macOS"), ("windows", "Windows"), ("linux", "Linux")]);
            self.tab_os = os.clone();
            let hs: Vec<setup::Hint> = setup::hints_for(&self.hints, &row.kind, &os).into_iter().cloned().collect();
            if os == "linux" && hs.len() > 1 {
                let opts: Vec<(&str, &str)> = hs.iter().map(|h| (h.package_manager.as_str(), match h.package_manager.as_str() { "apt" => "apt", "dnf" => "dnf", "zypper" => "zypper", "flatpak" => "Flatpak", p => p })).collect();
                if !hs.iter().any(|h| h.package_manager == self.tab_pm) { self.tab_pm = hs.first().map(|h| h.package_manager.clone()).unwrap_or_default(); }
                let mut pm = self.tab_pm.clone();
                crate::ui::widgets::segmented(ui, &mut pm, &opts);
                self.tab_pm = pm;
                if let Some(d) = self.distro_pm { ui.label(RichText::new(format!("Erkannte Verteilung: Paketmanager {d}")).small().color(k.mute)); }
            }
            let hint = if os == "linux" && hs.len() > 1 { hs.iter().find(|h| h.package_manager == self.tab_pm).or(hs.first()) } else { hs.first() };
            match hint {
                Some(h) => {
                    if !h.command.is_empty() {
                        ui.horizontal(|ui| {
                            let mut c = h.command.clone();
                            ui.add(egui::TextEdit::singleline(&mut c).font(egui::TextStyle::Monospace).desired_width(ui.available_width() - 110.0));
                            if ui.button("Kopieren").clicked() { ui.ctx().copy_text(h.command.clone()); }
                        });
                    }
                    if !h.note.is_empty() { ui.label(RichText::new(&h.note).small().color(k.mute)); }
                    if !h.url.is_empty() && ui.button("Webseite im Browser öffnen").on_hover_text(&h.url).clicked() { ui.ctx().open_url(egui::OpenUrl::new_tab(&h.url)); }
                }
                None => { ui.label(RichText::new("Für dieses Betriebssystem gibt es keinen Hinweis; bitte auf der Webseite des Programms nachsehen.").small().color(k.mute)); }
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Pfad wählen …").clicked() { if let Some(p) = rfd::FileDialog::new().pick_file() { self.set_tool_path(&row.kind, p.to_string_lossy().into_owned()); } }
                if ui.button("Erneut suchen").clicked() { self.forget_found(); }
                if !found && ui.button(format!("Ohne {} fortfahren", row.name.split(' ').next().unwrap_or(&row.name))).on_hover_text("Betroffene Systeme lassen sich bauen, aber nicht starten").clicked() { self.skipped.insert(vc_ide::setup::Topic::Toolchain); self.idx = (self.idx + 1).min(self.steps.len() - 1); self.reached = self.reached.max(self.idx); }
            });
            if cfg!(target_os = "macos") && os == "macos" { ui.label(RichText::new("Nicht signierte Programme blockiert macOS beim ersten Start: Rechtsklick ▸ Öffnen.").small().color(k.mute)); }
            ui.collapsing(RichText::new("Gesucht in").small(), |ui| { for p in setup::search_places(&row.kind) { ui.label(RichText::new(p).monospace().small().color(k.mute)); } });
        });
    }

    /// Pfad für alle Emulatoren der Sorte im Entwurf setzen; gibt es keinen, bekommt das erste passende System einen
    fn set_tool_path(&mut self, kind: &str, path: String) {
        let mut any = false;
        for t in &mut self.draft.settings.targets { for e in t.emulators.iter_mut().filter(|e| e.kind == kind) { e.path = path.clone(); any = true; } }
        if !any {
            let sys = SYSTEMS.iter().find(|s| s.kinds().iter().any(|k| k.id == kind) && self.draft.settings.target(s.id).is_some()).map(|s| s.id);
            if let Some(t) = sys.and_then(|s| self.draft.settings.target_mut(s)) { let id = t.add_emulator(kind); if let Some(e) = t.emulators.iter_mut().find(|e| e.id == id) { e.path = path; } }
        }
        self.forget_found(); self.refresh_status(true);
    }
}
