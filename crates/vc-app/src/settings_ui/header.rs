// SPDX-License-Identifier: GPL-3.0-or-later
//! Kopf der Detailansicht: System, Kurzinfo, Status und Schnellwahl des Startziels (F5).
use super::state::UiState;
use super::widgets::{badge, chip, system_color};
use eframe::egui::{self, RichText};
use vc_ide::settings::Settings;
use vc_ide::targets;

pub fn show(ui: &mut egui::Ui, s: &mut Settings, st: &mut UiState) {
    let Some(info) = targets::system(&st.system) else { return };
    // Sicht des Releases: Emulatoren und Maschinen abgewählter Anbindungen zählen hier nicht (die Datei behält sie)
    let std_emu = s.released().target(info.id).and_then(|t| t.standard_emulator()).cloned();
    let mut remove = false;
    ui.horizontal(|ui| {
        badge(ui, info.badge, 44.0, system_color(info.id));
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(info.name).heading());
                match &std_emu {
                    Some(e) => { let p = st.presence(e); if p.found { chip(ui, "Emulator bereit", crate::theme::ok()); } else { chip(ui, "Emulator fehlt", crate::theme::bad()); } }
                    None => chip(ui, "Kein Emulator", crate::theme::warn()),
                }
            });
            ui.label(RichText::new(info.cpu).weak().small());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("...", |ui| { if ui.button("Zielsystem deaktivieren").on_hover_text("Blendet das System überall aus (Projekte, F5, Listen). Emulator- und Hardwareeinträge bleiben erhalten; über „Zielsystem hinzufügen“ lässt es sich wieder aktivieren").clicked() { remove = true; ui.close(); } });
        });
    });
    ui.add_space(6.0);
    // Schnellwahl: Standard-Startziel (Emulator oder eine Maschine dieses Systems)
    let machines: Vec<(String, String)> = s.machines_for(info.id).filter(|m| vc_ide::emulators::device_enabled(&m.kind)).map(|m| (m.id.clone(), if m.kind == "mega65" { format!("{} · {}", m.display_name(), m.link_for(vc_ide::mega65net::Action::Start).label()) } else { m.display_name().to_string() })).collect();
    let mut start_on = s.target(info.id).map(|t| t.start_on.clone()).unwrap_or_default();
    if !start_on.is_empty() && !machines.iter().any(|(id, _)| *id == start_on) { start_on.clear(); }
    let before = start_on.clone();
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Startziel (F5):").weak());
        let emu_name = std_emu.as_ref().map_or("kein Emulator".to_string(), |e| e.display_name().to_string());
        let mut options: Vec<(String, String)> = vec![("".into(), format!("Emulator ({emu_name})"))];
        for (id, name) in &machines { options.push((id.clone(), format!("{name} (Hardware)"))); }
        let opts: Vec<(&str, &str)> = options.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        crate::ui::widgets::segmented(ui, &mut start_on, &opts);
        if machines.is_empty() { ui.label(RichText::new("– keine Hardware eingetragen").weak().small()); }
    });
    // Überschreibung durch das geöffnete Projekt (nur anzeigen, wenn eine für dieses System besteht)
    if let Some(o) = st.project.as_ref().and_then(|v| v.iter().find(|o| o.system == info.id)).cloned() {
        let emu = s.target(info.id).and_then(|t| t.emulators.iter().find(|e| e.id == o.default_emulator)).map(|e| e.display_name().to_string());
        let start = match o.start_on.as_str() { "" => None, "@emulator" => Some("Emulator".to_string()), id => s.machine(id).map(|m| m.display_name().to_string()) };
        let mut parts = vec![];
        if let Some(e) = emu { parts.push(format!("Standard-Emulator: {e}")); }
        if let Some(t) = start { parts.push(format!("Startziel: {t}")); }
        if !parts.is_empty() {
            ui.horizontal_wrapped(|ui| {
                chip(ui, "Überschrieben für dieses Projekt", crate::theme::warn());
                ui.label(RichText::new(parts.join(" · ")).small());
                if ui.small_button("Überschreibung entfernen").on_hover_text("Löscht die Angaben aus der Projektdatei; danach gelten wieder die Einstellungen hier").clicked() { st.clear_override = Some(info.id.to_string()); }
            });
        }
    }
    if start_on != before { if let Some(t) = s.target_mut(info.id) { t.start_on = start_on; } }
    if remove { vc_ide::setup::set_system(s, info.id, false); }
    ui.add_space(4.0);
    ui.separator();
}
