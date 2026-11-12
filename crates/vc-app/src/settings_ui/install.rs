// SPDX-License-Identifier: GPL-3.0-or-later
//! Homepage, Lizenz, Quelle (Version) und portable Installation je Emulator (Katalog `data/emulators/catalog.json`), dazu die Karte für Fujisan
//! (Debugger). Heruntergeladen wird nur nach Klick auf „Jetzt laden“; die Programme landen unterhalb von VintageCoder
//! (`emulators/<Emulator>/<Version>`). Mehrere Versionen eines Emulators liegen nebeneinander; die Karte wählt eine über den Pfad.
use super::{services, state::{Probe, UiState}, widgets};
use eframe::egui::{self, RichText};
use vc_ide::emulators::{self, Entry};
use vc_ide::settings::Settings;

fn mb(b: u64) -> f64 { b as f64 / 1_048_576.0 }

pub fn install_key(id: &str) -> String { format!("inst:{id}") }

/// Zeilen unter dem Kopf einer Emulator-Karte: Homepage, Lizenz, Rolle; Quelle (Version) wählen; weitere Version portabel installieren.
/// `path` ist das Pfadfeld der Karte: leer = automatisch, sonst der gewählte Pfad.
pub fn links(ui: &mut egui::Ui, en: &'static Entry, kind: &str, found: bool, path: &mut String, st: &mut UiState) {
    let ctx = ui.ctx().clone();
    let key = install_key(&en.id);
    ui.horizontal_wrapped(|ui| {
        ui.hyperlink_to("Homepage", &en.homepage).on_hover_text(&en.homepage);
        ui.label(RichText::new(format!("· {} · {}", en.license, en.role)).small().weak());
    });

    // Quelle: automatisch, jede portabel installierte Version, das System
    let srcs = st.sources(kind);
    if !srcs.is_empty() {
        let typed = path.trim().to_string();
        let label = if typed.is_empty() { "automatisch".to_string() } else { srcs.iter().find(|s| s.path == typed).map_or("eigener Pfad".to_string(), |s| s.label.clone()) };
        ui.horizontal(|ui| {
            ui.label(RichText::new("Quelle").small().weak());
            egui::ComboBox::from_id_salt(("emu_src", kind, &en.id)).selected_text(label).show_ui(ui, |ui| {
                if ui.selectable_label(typed.is_empty(), "automatisch (System, sonst neueste portable Version)").clicked() { path.clear(); st.forget_presence(); }
                for s in &srcs {
                    if ui.selectable_label(typed == s.path, format!("{}  –  {}", s.label, s.path)).clicked() { *path = s.path.clone(); st.forget_presence(); }
                }
            });
        });
    }

    if st.offline { return; }   // Konfigurationsassistent: kein Herunterladen, keine Installation
    if st.running(&key) { ui.label(RichText::new("lädt und installiert …").small().color(crate::theme::accent())); return; }

    // weitere Version portabel installieren
    let have: Vec<String> = vc_emu::install::versions(&en.id).into_iter().map(|(v, _)| v).collect();
    let todo: Vec<&vc_ide::emulators::Pkg> = en.packages_here().into_iter().filter(|p| !have.contains(&p.version)).collect();
    if let Some(first) = todo.first() {
        let chosen = st.install_version.get(&en.id).filter(|v| todo.iter().any(|p| &p.version == *v)).cloned().unwrap_or_else(|| first.version.clone());
        let pkg = todo.iter().find(|p| p.version == chosen).copied().unwrap_or(first);
        let spec = format!("{}@{}", en.id, pkg.version);
        if st.install_confirm.as_deref() == Some(spec.as_str()) {
            let to = emulators::install_dir(&en.id).map(|d| d.join(&pkg.version).display().to_string()).unwrap_or_default();
            ui.label(RichText::new(format!("Lädt {} {} ({:.1} MB) von {}, prüft die SHA-256-Summe und packt es nach {to} aus.", en.name, pkg.version, mb(pkg.size), pkg.url.split('/').nth(2).unwrap_or(""))).small());
            ui.horizontal(|ui| {
                if ui.button("Jetzt laden").clicked() { services::install_emulator(st, &ctx, &spec); st.install_confirm = None; }
                if ui.button("Abbrechen").clicked() { st.install_confirm = None; }
            });
        } else {
            ui.horizontal(|ui| {
                if todo.len() > 1 {
                    egui::ComboBox::from_id_salt(("emu_ver", &en.id, kind)).selected_text(chosen.clone()).show_ui(ui, |ui| {
                        for p in &todo { if ui.selectable_label(p.version == chosen, format!("{} ({:.1} MB)", p.version, mb(p.size))).clicked() { st.install_version.insert(en.id.clone(), p.version.clone()); } }
                    });
                }
                let text = if have.is_empty() && !found { format!("Portabel installieren ({} {:.1} MB) …", pkg.version, mb(pkg.size)) } else { format!("Version {} portabel dazuinstallieren …", pkg.version) };
                if ui.button(text).on_hover_text("Lädt eine feste Version von der Seite des Projekts, prüft die Prüfsumme und installiert sie unterhalb von VintageCoder; vorhandene Versionen bleiben").clicked() { st.install_confirm = Some(spec); }
            });
        }
    } else if en.packages_here().is_empty() && !found {
        let hint = en.manual_hint().unwrap_or("Für dieses System gibt es kein automatisches Paket.");
        ui.horizontal_wrapped(|ui| { ui.label(RichText::new(hint).small().weak()); ui.hyperlink_to("Download-Seite", &en.download_page); });
    }
    match st.probe(&key) {
        Some(Probe::Ok(t)) => { ui.label(RichText::new(format!("Installiert: {t}")).color(crate::theme::ok()).small()); }
        Some(Probe::Err(t)) => { ui.label(RichText::new(t).color(crate::theme::bad()).small()); }
        _ => {}
    }
}

/// Karte „Fujisan“ auf der Seite des Atari-Systems: Pfad, Wahl des Debuggers, Homepage, Version und Installation
pub fn fujisan_card(ui: &mut egui::Ui, s: &mut Settings, st: &mut UiState) {
    let Some(en) = emulators::find("fujisan") else { return };
    ui.add_space(6.0);
    widgets::section(ui, "Fujisan (Debugger für Atari-Programme)", "Atari-Emulator mit TCP-Fernsteuerung. Der Debugger nimmt ihn für .xex-Programme, wenn er gefunden wird; sonst atari800.", |_| {});
    let fj = &mut s.emulators.fujisan;
    let typed = fj.path.trim().to_string();
    let found_path = if typed.is_empty() { vc_ide::launch::detect("fujisan") } else { Some(vc_ide::settings::expand_home(&typed)).filter(|p| std::path::Path::new(p).is_file()) };
    widgets::card(ui, false, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Fujisan").strong());
            if found_path.is_some() { widgets::chip(ui, if typed.is_empty() { "Automatisch erkannt" } else { "Gefunden" }, crate::theme::ok()); } else { widgets::chip(ui, "Fehlt", crate::theme::bad()); }
        });
        ui.add_space(4.0);
        egui::Grid::new("fujisan_grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            widgets::label(ui, "Pfad");
            let hint = match &found_path { Some(p) if typed.is_empty() => format!("automatisch: {p}"), _ => "leer = automatisch suchen".to_string() };
            widgets::path_field(ui, &mut fj.path, &hint, false);
            ui.end_row();
            widgets::label(ui, "Debugger");
            let label = match fj.debug_for_atari.trim() { "fujisan" => "immer Fujisan", "atari800" => "immer atari800", _ => "automatisch (Fujisan, wenn gefunden)" };
            egui::ComboBox::from_id_salt("fujisan_debug").selected_text(label).show_ui(ui, |ui| {
                ui.selectable_value(&mut fj.debug_for_atari, String::new(), "automatisch (Fujisan, wenn gefunden)");
                ui.selectable_value(&mut fj.debug_for_atari, "fujisan".to_string(), "immer Fujisan");
                ui.selectable_value(&mut fj.debug_for_atari, "atari800".to_string(), "immer atari800");
            });
            ui.end_row();
        });
        links(ui, en, "fujisan", found_path.is_some(), &mut fj.path, st);
    });
}
