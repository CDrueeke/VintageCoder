// SPDX-License-Identifier: GPL-3.0-or-later
//! Werkzeuge des Wiederaufrufs: Export (ohne Passwort, mit Rückfrage zu Pfaden), Import (zeigt dasselbe Diff, schreibt erst nach Bestätigung)
//! und Zurücksetzen je Bereich (Hardware und Passwörter nur nach ausdrücklicher Bestätigung).
use super::{Env, Out, SetupWizard};
use crate::theme::View;
use crate::ui::dialog::{self, Act, Dlg};
use crate::ui::tokens::t;
use eframe::egui::RichText;
use vc_ide::settings::Settings;
use vc_ide::setup::{self, Draft, ExportOpts, ResetArea, Topic};

pub enum Side {
    None,
    Export { opts: ExportOpts, outside: Vec<String>, msg: String },
    Import { msg: String },
    Reset { area: ResetArea, with_hw: bool },
}

impl Side {
    pub fn export_for(s: &Settings) -> Side { Side::Export { opts: ExportOpts::default(), outside: setup::outside_paths(s), msg: String::new() } }
}

impl SetupWizard {
    /// Öffnet den Export aus dem laufenden Assistenten (z. B. auf der Fertig-Seite)
    pub(crate) fn open_export(&mut self) { self.side = Side::export_for(&self.cur.settings); }

    /// Zeichnet den offenen Dialog; liefert `Out::Close`, wenn der Assistent nur für den Dialog geöffnet war und der Dialog endet
    pub(crate) fn show_side(&mut self, ctx: &eframe::egui::Context, env: &mut Env) -> Option<Out> {
        let tool_only = self.tool_only;
        let mut close = false;
        let side = std::mem::replace(&mut self.side, Side::None);
        let next = match side {
            Side::None => Side::None,
            Side::Export { mut opts, outside, mut msg } => {
                let act = dialog::show(ctx, &Dlg { title: "Konfiguration exportieren", sub: "vintagecoder-config.json", chips: &[], primary: "Speichern unter …", secondary: "Schließen" }, |ui| {
                    let k = t();
                    ui.label("Die Datei enthält Systeme, Emulatoren, Hardware und die Darstellung. Pfade im Projektordner werden relativ geschrieben.");
                    ui.add_space(6.0);
                    ui.checkbox(&mut opts.passwords, "Netzwerkpasswörter der Maschinen einschließen");
                    if opts.passwords { super::widgets::note(ui, "!", "Dann steht das Passwort im Klartext in der Datei. Gib sie nicht weiter.", k.am); } else { ui.label(RichText::new("Voreinstellung: ohne Passwörter.").small().color(k.mute)); }
                    if !outside.is_empty() {
                        ui.add_space(6.0);
                        ui.checkbox(&mut opts.outside_paths, format!("{} Pfade außerhalb des Projektordners mitnehmen", outside.len()));
                        for p in outside.iter().take(6) { ui.label(RichText::new(p).monospace().small().color(k.mute)); }
                        if !opts.outside_paths { ui.label(RichText::new("Ohne Haken bleiben sie leer; auf dem anderen Rechner wird dann automatisch gesucht.").small().color(k.mute)); }
                    }
                    if !msg.is_empty() { ui.add_space(6.0); ui.label(RichText::new(&msg).small().color(k.gr)); }
                    true
                });
                match act {
                    Act::Primary => {
                        if let Some(p) = rfd::FileDialog::new().set_file_name("vintagecoder-config.json").add_filter("VintageCoder-Konfiguration", &["json"]).save_file() {
                            let j = setup::export_json(&self.cur.settings, &self.cur.view, opts);
                            msg = match vc_ide::settings::write_atomic(&p, format!("{}\n", serde_json::to_string_pretty(&j).unwrap_or_default()).as_bytes()) { Ok(()) => format!("Gespeichert: {}", p.display()), Err(e) => format!("Fehler: {e}") };
                        }
                        Side::Export { opts, outside, msg }
                    }
                    Act::Secondary => { close = true; Side::None }
                    Act::None => Side::Export { opts, outside, msg },
                }
            }
            Side::Import { mut msg } => {
                let act = dialog::show(ctx, &Dlg { title: "Konfiguration importieren", sub: "", chips: &[], primary: "Datei wählen …", secondary: "Abbrechen" }, |ui| {
                    ui.label("Du wählst eine Exportdatei. Danach siehst du dieselbe Änderungsliste wie bei jeder Änderung; geschrieben wird erst nach „Übernehmen“.");
                    ui.label(RichText::new("Pfade, die es auf diesem Rechner nicht gibt, werden gemeldet.").small().color(t().mute));
                    if !msg.is_empty() { super::widgets::note(ui, "!", &msg, t().rd); }
                    true
                });
                match act {
                    Act::Primary => {
                        let Some(p) = rfd::FileDialog::new().add_filter("VintageCoder-Konfiguration", &["json"]).pick_file() else { self.side = Side::Import { msg }; return None };
                        match std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display())).and_then(|t| setup::import_json(&t, &self.cur)) {
                            Ok(imp) => {
                                let examples = self.draft.examples.clone();
                                self.draft = Draft { examples, ..imp.draft };
                                *env.view = View::from_prefs(&self.draft.view);
                                self.notes = imp.path_notes;
                                self.steps = vec![Topic::Summary, Topic::Done]; self.idx = 0; self.reached = 0; self.recall = true; self.tool_only = false;
                                self.refresh_status(true);
                                Side::None
                            }
                            Err(e) => { msg = e; Side::Import { msg } }
                        }
                    }
                    Act::Secondary => { close = true; Side::None }
                    Act::None => Side::Import { msg },
                }
            }
            Side::Reset { mut area, mut with_hw } => {
                let act = dialog::show(ctx, &Dlg { title: "Auf Standard zurücksetzen", sub: "", chips: &[], primary: "Weiter zur Zusammenfassung", secondary: "Abbrechen" }, |ui| {
                    let k = t();
                    ui.label("Du wählst einen Bereich. Danach siehst du, was sich ändert; zurückgesetzt wird erst nach „Übernehmen“, mit Sicherung settings.json.bak.");
                    ui.add_space(6.0);
                    for (a, label) in [(ResetArea::Design, "Design (Schema, Größe, Schrift)"), (ResetArea::Roms, "ROM-Einstellungen"), (ResetArea::Emulators, "Emulatoren (Pfade, Parameter, Assembler-Optionen)"), (ResetArea::Hardware, "Echte Hardware (Maschinen und Passwörter)"), (ResetArea::All, "Alles")] {
                        ui.radio_value(&mut area, a, label);
                    }
                    if matches!(area, ResetArea::Hardware | ResetArea::All) {
                        ui.add_space(6.0);
                        ui.checkbox(&mut with_hw, if area == ResetArea::All { "auch Hardware-Einträge und Passwörter löschen" } else { "Ja, Hardware-Einträge und Passwörter löschen" });
                        super::widgets::note(ui, "!", "Gelöschte Passwörter lassen sich nur aus settings.json.bak zurückholen.", k.am);
                    }
                    area != ResetArea::Hardware || with_hw
                });
                match act {
                    Act::Primary => {
                        setup::reset_area(&mut self.draft, area);
                        if area == ResetArea::All && with_hw { setup::reset_area(&mut self.draft, ResetArea::Hardware); }
                        if matches!(area, ResetArea::Design | ResetArea::All) { *env.view = View::from_prefs(&self.draft.view); }
                        self.steps = vec![Topic::Summary, Topic::Done]; self.idx = 0; self.reached = 0; self.recall = true; self.tool_only = false;
                        self.refresh_status(true);
                        Side::None
                    }
                    Act::Secondary => { close = true; Side::None }
                    Act::None => Side::Reset { area, with_hw },
                }
            }
        };
        // Der Dialog darf sich nicht selbst überschreiben, wenn der Import ihn beendet hat
        if matches!(self.side, Side::None) { self.side = next; }
        if close && tool_only { return Some(Out::Close); }
        None
    }
}
