// SPDX-License-Identifier: GPL-3.0-or-later
//! Schritte „Emulatoren“, „ROMs“, „Echte Hardware“ und „Startziel F5“. Emulator-, ROM- und Hardware-Seiten sind die der Einstellungen
//! (`settings_ui`), nur ohne Installationskarten: der Assistent lädt nichts herunter.
use crate::settings_ui::state::{hw_key, Probe};
use crate::setup_wizard::{widgets, SetupWizard};
use crate::ui::tokens::t;
use crate::ui::widgets::{badge, card, chip, system_color};
use eframe::egui::{self, RichText};
use vc_ide::targets::{self, SYSTEMS};

impl SetupWizard {
    /// Liste der eingerichteten Systeme links; setzt `st.system`
    fn system_list(&mut self, ui: &mut egui::Ui) {
        let k = t();
        if self.draft.settings.target(&self.st.system).is_none() { self.st.system = self.draft.settings.targets.first().map(|t| t.system.clone()).unwrap_or_default(); }
        for tcfg in self.draft.settings.released().targets.clone() {     // Sicht des Releases: Emulatoren abgewählter Anbindungen zählen nicht
            let Some(info) = targets::system(&tcfg.system) else { continue };
            let sel = self.st.system == tcfg.system;
            let std_emu = tcfg.standard_emulator().cloned();
            let ok = std_emu.as_ref().map_or(false, |e| self.st.presence(e).found);
            let r = widgets::choice_card(ui, sel, 190.0, |ui| {
                ui.horizontal(|ui| { badge(ui, info.badge, 22.0, system_color(info.id)); ui.label(RichText::new(info.name).strong().color(k.fg)); });
                ui.label(RichText::new(if ok { "✓ Emulator bereit" } else if std_emu.is_some() { "– Emulator fehlt" } else { "– kein Emulator" }).small().color(if ok { k.gr } else { k.am }));
            });
            if r.clicked() { self.st.system = tcfg.system.clone(); }
        }
    }

    pub(crate) fn step_emulators(&mut self, ui: &mut egui::Ui) {
        if self.draft.settings.targets.is_empty() { ui.label("Wähle zuerst im Schritt „Zielsysteme“ mindestens ein System."); return; }
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| { ui.set_width(200.0); self.system_list(ui); });
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                crate::settings_ui::emulators::show(ui, &mut self.draft.settings, &mut self.st);
            });
        });
    }

    pub(crate) fn step_roms(&mut self, ui: &mut egui::Ui) {
        let k = t();
        card(ui, false, |ui| {
            widgets::note(ui, "i", "Dateien werden am Inhalt erkannt (Größe, Prüfsumme), nicht am Namen. Kopiert wird nur auf Klick; es gibt keinen ROM-Download.", k.cy);
            ui.label(RichText::new("Hinweis: ROM-Dateien, die du hier importierst, werden sofort in den ROM-Ordner kopiert (nicht erst bei „Übernehmen“). Das ist keine Rechtsberatung: Original-ROMs gehören dem Rechteinhaber.").small().color(k.mute));
        });
        ui.add_space(6.0);
        let mut saved = self.cur.settings.roms.clone();
        crate::settings_ui::roms::show(ui, &mut self.roms, &mut self.draft.settings.roms, &mut saved);
    }

    pub(crate) fn step_hardware(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let had = !self.draft.settings.machines.is_empty();
        card(ui, false, |ui| {
            ui.checkbox(&mut self.has_hardware, "Ich habe echte Hardware anzuschließen (Ultimate 64 / II+, MEGA65)");
            if !self.has_hardware { ui.label(RichText::new(if had { "Die vorhandenen Einträge bleiben unverändert." } else { "Der Schritt gilt als erledigt; Hardware lässt sich später über den Teil-Assistenten oder die Einstellungen eintragen." }).small().color(k.mute)); }
        });
        if !self.has_hardware { return; }
        ui.add_space(6.0);
        let sys: Vec<&'static targets::SystemInfo> = SYSTEMS.iter().filter(|s| s.hardware().iter().any(|h| vc_ide::emulators::device_enabled(h)) && self.draft.settings.target(s.id).is_some()).collect();
        if sys.is_empty() { ui.label("Echte Hardware gibt es für C64 (Ultimate) und MEGA65; wähle eines dieser Systeme im Schritt „Zielsysteme“."); return; }
        for info in sys {
            self.st.system = info.id.to_string();
            ui.horizontal(|ui| { badge(ui, info.badge, 24.0, system_color(info.id)); ui.label(RichText::new(info.name).font(egui::FontId::new(16.0, crate::fonts::ui_strong()))); });
            crate::settings_ui::hardware::show(ui, &mut self.draft.settings, &mut self.st);
            ui.add_space(8.0);
        }
        card(ui, false, |ui| {
            ui.label(RichText::new("Ablage des Passworts").strong());
            ui.label(RichText::new("Das Netzwerkpasswort liegt in settings.json (Rechte 0600, nur für dich lesbar). Es erscheint nie im Klartext in der Änderungsliste und fehlt im Export, solange du ihn nicht ausdrücklich wählst. Die Ablage im Schlüsselbund des Betriebssystems ist noch nicht umgesetzt.").small().color(k.mute));
        });
        if cfg!(target_os = "linux") && self.draft.settings.machines.iter().any(|m| m.kind == "mega65") {
            ui.add_space(6.0);
            card(ui, false, |ui| {
                ui.label(RichText::new("Linux: Zugriff auf die serielle Schnittstelle").strong());
                ui.label(RichText::new("Dein Benutzer braucht die Gruppe dialout (auf manchen Verteilungen uucp). Kopiere den Befehl und führe ihn selbst aus, danach ab- und wieder anmelden. VintageCoder führt ihn nie aus.").small().color(k.mute));
                if let Some(h) = self.hints.iter().find(|h| h.program == "serial" && h.os == "linux") {
                    ui.horizontal(|ui| { let mut c = h.command.clone(); ui.add(egui::TextEdit::singleline(&mut c).font(egui::TextStyle::Monospace).desired_width(320.0)); if ui.button("Kopieren").clicked() { ui.ctx().copy_text(h.command.clone()); } });
                }
            });
        }
    }

    pub(crate) fn step_start(&mut self, ui: &mut egui::Ui) {
        let k = t();
        if self.draft.settings.targets.is_empty() { ui.label("Wähle zuerst im Schritt „Zielsysteme“ mindestens ein System."); return; }
        ui.horizontal(|ui| {
            widgets::keycap(ui, "F5");
            ui.vertical(|ui| {
                ui.label(RichText::new("① Bauen  →  ② Starten auf dem gewählten Ziel").font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
                ui.label(RichText::new("Pro System genau ein Ziel: der Standard-Emulator oder eine eingetragene Maschine.").small().color(k.mute));
            });
        });
        ui.add_space(8.0);
        for tcfg in self.draft.settings.released().targets.clone() {
            let Some(info) = targets::system(&tcfg.system) else { continue };
            let std_emu = tcfg.standard_emulator().cloned();
            let emu_ok = std_emu.as_ref().map_or(false, |e| self.st.presence(e).found);
            let machines: Vec<(String, String, String)> = self.draft.settings.machines_for(info.id).filter(|m| vc_ide::emulators::device_enabled(&m.kind)).map(|m| (m.id.clone(), m.display_name().to_string(), m.kind.clone())).collect();
            let mut start = tcfg.start_on.clone();
            if !start.is_empty() && !machines.iter().any(|(id, _, _)| *id == start) { start.clear(); }
            let before = start.clone();
            card(ui, false, |ui| {
                ui.horizontal(|ui| { badge(ui, info.badge, 22.0, system_color(info.id)); ui.label(RichText::new(info.name).strong()); });
                ui.horizontal(|ui| {
                    ui.radio_value(&mut start, String::new(), format!("Emulator ({})", std_emu.as_ref().map_or("keiner".to_string(), |e| e.display_name().to_string())));
                    if emu_ok { chip(ui, "✓ bereit", k.gr); } else { chip(ui, "– fehlt", k.am); }
                });
                for (id, name, _kind) in &machines {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut start, id.clone(), format!("{name} (Hardware)"));
                        match self.st.probe(&hw_key(id)) { Some(Probe::Ok(_)) => chip(ui, "● online", k.gr), Some(Probe::Err(_)) => chip(ui, "! nicht erreichbar", k.rd), Some(Probe::Running) => chip(ui, "prüfe …", k.am), None => chip(ui, "○ ungeprüft", k.dim) };
                    });
                }
                if !start.is_empty() && !matches!(self.st.probe(&hw_key(&start)), Some(Probe::Ok(_))) { widgets::note(ui, "i", "Die Maschine ist ungeprüft oder offline; F5 meldet dann einen Fehler. Gesperrt wird nichts.", k.am); }
                if machines.is_empty() { ui.label(RichText::new("Keine Maschine eingetragen: F5 startet im Emulator.").small().color(k.mute)); }
            });
            if start != before { if let Some(t) = self.draft.settings.target_mut(info.id) { t.start_on = start; } }
            ui.add_space(4.0);
        }
    }
}
