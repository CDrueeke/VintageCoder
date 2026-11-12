// SPDX-License-Identifier: GPL-3.0-or-later
//! Abschnitt „Konfigurierte Emulatoren“: eine Karte je Emulator mit Pfad, Argumenten, Statusanzeige und Aktionen.
use super::{services, state::{emu_key, Probe, UiState}, widgets};
use eframe::egui::{self, RichText};
use vc_ide::emulators::{self as emu_cat, enabled};
use vc_ide::platform;
use vc_ide::settings::Settings;
use vc_ide::targets::{self, EmulatorCfg};

enum Action { Standard(String), Remove(String), Test(EmulatorCfg), Detect(String) }

/// ROM-Feld der Karte aus dem Katalog (`rom`): Beschriftung, Hinweis, Ordner statt Datei
fn rom_label(kind: &str) -> Option<(&'static str, &'static str, bool)> {
    emu_cat::for_kind(kind)?.rom.as_ref().map(|r| (r.label.as_str(), r.hint.as_str(), r.folder))
}

pub fn show(ui: &mut egui::Ui, s: &mut Settings, st: &mut UiState) {
    let Some(info) = targets::system(&st.system) else { return };
    let mut add_kind: Option<&'static str> = None;
    // nur Sorten, deren Anbindung im Release gewählt ist; Einträge der übrigen bleiben in der Datei, werden aber nicht gezeigt
    let kinds: Vec<&'static targets::EmuKind> = info.kinds().iter().filter(|k| enabled(k.id)).collect();
    widgets::section(ui, "Konfigurierte Emulatoren", "Programm, Startargumente und Standard-Emulator dieses Systems. Leere Pfade werden automatisch gesucht (PATH und übliche Ordner).", |ui| {
        ui.add_enabled_ui(!kinds.is_empty(), |ui| { ui.menu_button("+  Weiteren Emulator hinzufügen", |ui| { for k in &kinds { if ui.button(k.label).clicked() { add_kind = Some(k.id); ui.close(); } } }); });
    });
    let mut action = None;
    let ctx = ui.ctx().clone();
    let Some(t) = s.target_mut(info.id) else { return };
    if let Some(k) = add_kind { t.add_emulator(k); }
    let count = t.emulators.iter().filter(|e| enabled(&e.kind)).count();
    if kinds.is_empty() { ui.label(RichText::new("In diesem Release ist keine Emulator-Anbindung für dieses System enthalten.").weak()); }
    else if count == 0 { ui.label(RichText::new("Kein Emulator eingetragen. Ohne Emulator lässt sich dieses System nur auf Hardware starten.").weak()); }
    for e in t.emulators.iter_mut().filter(|e| enabled(&e.kind)) {
        widgets::card(ui, e.standard && count > 1, |ui| {
            let presence = st.presence(e);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut e.name).hint_text("Name").desired_width(210.0));
                if kinds.len() > 1 {
                    egui::ComboBox::from_id_salt(("emu_kind", &e.id)).selected_text(targets::kind_label(&e.kind)).show_ui(ui, |ui| { for k in &kinds { ui.selectable_value(&mut e.kind, k.id.to_string(), k.label); } });
                } else { ui.label(RichText::new(targets::kind_label(&e.kind)).weak()); }
                if e.standard { widgets::chip(ui, "STANDARD", crate::theme::accent()); }
                if presence.found { widgets::chip(ui, if presence.auto { "Automatisch erkannt" } else { "Gefunden" }, crate::theme::ok()); } else { widgets::chip(ui, "Fehlt", crate::theme::bad()); }
            });
            if let Some(en) = emu_cat::for_kind(&e.kind) { super::install::links(ui, en, &e.kind, presence.found, &mut e.path, st); }
            ui.add_space(4.0);
            egui::Grid::new(("emu_grid", &e.id)).num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                widgets::label(ui, "Pfad");
                ui.horizontal(|ui| {
                    let hint = if presence.auto && presence.found { format!("automatisch: {}", presence.text) } else { "leer = automatisch suchen".to_string() };
                    widgets::path_field(ui, &mut e.path, &hint, false);
                    if ui.button("Automatisch suchen").on_hover_text("Trägt den Pfad ein, den die Suche in PATH und den üblichen Ordnern findet").clicked() { action = Some(Action::Detect(e.id.clone())); }
                });
                ui.end_row();
                if let Some((lbl, hint, folder)) = rom_label(&e.kind) {
                    widgets::label(ui, lbl);
                    ui.horizontal(|ui| {
                        widgets::path_field(ui, &mut e.rom, hint, folder);
                        let v = e.rom.trim();
                        if !v.is_empty() {
                            let p = std::path::PathBuf::from(vc_ide::settings::expand_home(v));
                            if (folder && p.is_dir()) || (!folder && p.is_file()) { ui.label(RichText::new("vorhanden").color(crate::theme::ok()).small()); } else { ui.label(RichText::new("nicht gefunden").color(crate::theme::bad()).small()); }
                        }
                    });
                    ui.end_row();
                }
            });
            if !presence.found { ui.label(RichText::new(&presence.text).small().color(crate::theme::bad())); }
            ui.add_space(4.0);
            let program = if e.path.trim().is_empty() { e.kind.clone() } else { e.path.trim().to_string() };
            super::args::show(ui, e, info.id, &program, st);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let key = emu_key(&e.id);
                let busy = st.running(&key);
                if ui.add_enabled(!busy && presence.found, egui::Button::new(if busy { "startet …" } else { "▶  Test-Start" })).on_hover_text("Startet den Emulator mit diesen Parametern und beendet ihn nach kurzer Zeit wieder").clicked() { action = Some(Action::Test(e.clone())); }
                if ui.add_enabled(!e.standard, egui::Button::new("Als Standard setzen")).clicked() { action = Some(Action::Standard(e.id.clone())); }
                if ui.button("Löschen").clicked() { action = Some(Action::Remove(e.id.clone())); }
                match st.probe(&key) {
                    Some(Probe::Ok(t)) => { ui.label(RichText::new(format!("OK: {t}")).color(crate::theme::ok()).small()); }
                    Some(Probe::Err(t)) => { ui.label(RichText::new(t).color(crate::theme::bad()).small()); }
                    _ => {}
                }
            });
        });
        ui.add_space(8.0);
    }
    match action {
        Some(Action::Test(e)) => services::test_emulator(st, &ctx, &e),
        Some(Action::Standard(id)) => t.set_standard(&id),
        Some(Action::Remove(id)) => { t.remove_emulator(&id); st.probes.remove(&emu_key(&id)); }
        Some(Action::Detect(id)) => { if let Some(e) = t.emulators.iter_mut().find(|e| e.id == id) { if let Some(p) = platform::discover(&e.kind) { e.path = p; st.forget_presence(); } } }
        None => {}
    }
    if info.id == "atari8" && !st.offline && enabled("fujisan") { super::install::fujisan_card(ui, s, st); }
}
