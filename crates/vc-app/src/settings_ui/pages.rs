// SPDX-License-Identifier: GPL-3.0-or-later
//! Weitere Seiten der Einstellungen: Allgemein, Toolchain, Editor, Tastenkürzel. Sie zeigen und ändern nur echte Werte;
//! was sich (noch) nicht einstellen lässt, steht als Anzeige da und nennt, wo es gilt.
use super::Act;
use crate::theme::{Scheme, View};
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip, section};
use eframe::egui::{self, RichText};
use std::path::Path;

/// Alles, was die Seiten außerhalb des Einstellungsentwurfs brauchen
pub struct Env<'a> {
    pub view: &'a mut View,
    pub show_addr: &'a mut bool,
    pub data: Option<&'a Path>,
    /// geöffnetes Projekt: Name, Auto-Export an?
    pub project: Option<(String, bool)>,
}

fn row_resp(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui) -> egui::Response) -> egui::Response {
    ui.horizontal(|ui| { crate::ui::widgets::label(ui, label); body(ui) }).inner
}

fn row(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| { crate::ui::widgets::label(ui, label); body(ui); });
}

pub fn general(ui: &mut egui::Ui, env: &mut Env, help: &mut vc_ide::settings::HelpPrefs, saved: &mut vc_ide::settings::HelpPrefs) {
    section(ui, "Darstellung", "Gilt für die ganze Oberfläche und wirkt sofort.", |_| {});
    card(ui, false, |ui| {
        let mut scheme = match env.view.scheme { Scheme::Dark => "dark", Scheme::Light => "light", Scheme::System => "system" }.to_string();
        row(ui, "Farbschema", |ui| { if crate::ui::widgets::segmented(ui, &mut scheme, &[("dark", "Dunkel"), ("light", "Hell"), ("system", "System")]) { env.view.scheme = match scheme.as_str() { "light" => Scheme::Light, "system" => Scheme::System, _ => Scheme::Dark }; } });
        row(ui, "Oberfläche", |ui| {
            crate::theme::zoom_slider(ui, &mut env.view.zoom)
                .on_hover_text("Größe der gesamten Oberfläche; auch mit Strg/Cmd + und −, zurück mit Strg/Cmd 0");
        });
        if ui.button("Darstellung zurücksetzen").clicked() { *env.view = View::default(); }
    });
    ui.add_space(12.0);
    section(ui, "Einrichtung", "Der Konfigurationsassistent prüft die Einrichtung und ändert nur, was du bestätigst.", |_| {});
    card(ui, false, |ui| {
        let last = vc_ide::setup::Paths::current().map(|p| vc_ide::setup::WizardState::load(&p.wizard).last_run).filter(|d| !d.is_empty());
        row(ui, "Assistent", |ui| {
            if ui.button("Assistent starten").clicked() { ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("setup_request"), true)); }
            ui.label(RichText::new(match last { Some(d) => format!("zuletzt ausgeführt: {d}"), None => "noch nicht ausgeführt".to_string() }).small().weak());
        });
    });
    ui.add_space(12.0);
    section(ui, "Fehlersuche", "Das Debug-Protokoll schreibt Befehle, Meldungen, Fehler und Abstürze in eine Datei auf deinem Rechner; nichts wird gesendet. Die Datei hängst du bei Bedarf selbst an eine Feedback-Mail.", |_| {});
    card(ui, false, |ui| {
        let mut cur = vc_ide::settings::get();
        let env_on = std::env::var("VC_DEBUG_LOG").ok().filter(|v| !v.trim().is_empty());
        let mut on = if env_on.is_some() { vc_ide::debuglog::enabled() } else { cur.debug_log };
        row(ui, "Debug-Protokoll", |ui| { ui.add_enabled(env_on.is_none(), egui::Checkbox::new(&mut on, "Protokolldatei schreiben")); });
        if env_on.is_none() && on != cur.debug_log { cur.debug_log = on; let _ = vc_ide::settings::set(cur); vc_ide::debuglog::set_enabled(on); }
        if let Some(v) = &env_on { ui.label(RichText::new(format!("Umgebungsvariable VC_DEBUG_LOG überstimmt die Einstellung: {v}")).small().color(t().am)); }
        if let Some(p) = vc_ide::debuglog::path() { ui.label(RichText::new(format!("Datei: {}", p.display())).small().monospace().weak()); }
        ui.label(RichText::new("Pfade im persönlichen Ordner stehen als ~, Passwörter und Tokens werden geschwärzt. Dateinamen und Fehlermeldungen deines Projekts können vorkommen: vor dem Weitergeben ansehen. Ab 5 MB beginnt eine neue Datei (die alte bleibt als .log.1).").small().weak());
        if ui.button("Ordner öffnen").clicked() { if let Some(d) = vc_ide::debuglog::dir() { let _ = std::fs::create_dir_all(&d); crate::feedback::open_folder(&d); } }
    });
    ui.add_space(12.0);
    section(ui, "Hilfe", "Die Bilder des Handbuchs liegen im Internet und werden erst nach deiner Zustimmung geladen; die Wahl wirkt sofort und wird gespeichert.", |_| {});
    card(ui, false, |ui| {
        let mut sel = if help.images.is_empty() { "ask" } else { help.images.as_str() }.to_string();
        let sel_before = sel.clone();
        row(ui, "Bilder laden", |ui| { crate::ui::widgets::segmented(ui, &mut sel, &[("ask", "Fragen"), ("always", "Immer"), ("never", "Nie")]); });
        let mut changed = sel != sel_before;
        if changed { help.images = if sel == "ask" { String::new() } else { sel }; }
        let r = row_resp(ui, "Adresse", |ui| ui.add(egui::TextEdit::singleline(&mut help.images_url).hint_text(vc_ide::help_images::DEFAULT_BASE).desired_width(420.0)));
        if r.lost_focus() && help.images_url != saved.images_url { changed = true; }
        if let Ok(v) = std::env::var("VC_HELP_IMAGES") { ui.label(RichText::new(format!("Umgebungsvariable VC_HELP_IMAGES überstimmt die Adresse: {v}")).small().color(t().am)); }
        ui.label(RichText::new("Ein Ordner statt einer Webadresse (z. B. docs/handbuch/bilder) wird gelesen statt geladen. Beim Laden wird nur die Adresse des Bildes abgerufen; geladene Bilder bleiben im Zwischenspeicher.").small().weak());
        if let Some(d) = vc_ide::help_images::cache_dir() { ui.label(RichText::new(format!("Zwischenspeicher: {}", d.display())).small().monospace().weak()); }
        if changed {
            // nur dieses Feld speichern, nicht den übrigen (womöglich ungespeicherten) Entwurf
            let mut cur = vc_ide::settings::get(); cur.help = help.clone();
            let _ = vc_ide::settings::set(cur);
            *saved = help.clone();
        }
    });
    ui.add_space(12.0);
    section(ui, "Dateien", "Wo VintageCoder Einstellungen ablegt.", |_| {});
    card(ui, false, |ui| {
        let dir = vc_core::userdirs::config_dir().map(|d| d.display().to_string()).unwrap_or_else(|| "–".into());
        row(ui, "Ordner", |ui| { ui.label(RichText::new(dir).monospace().small()); });
        row(ui, "Einstellungen", |ui| { ui.label(RichText::new(vc_ide::settings::file().map(|f| f.display().to_string()).unwrap_or_else(|| "–".into())).monospace().small()); });
        ui.label(RichText::new("Die Einstellungsdatei enthält die Passwörter der Hardware und ist nur für dich lesbar.").small().weak());
    });
}

pub fn toolchain(ui: &mut egui::Ui, env: &mut Env, asm: &mut vc_ide::settings::AsmCfg, focus: &mut Option<(String, String)>) -> Act {
    let k = t();
    let mut act = Act::None;
    section(ui, "Assembler", "VintageCoder hat einen eigenen Assembler; es wird kein externer Assembler benötigt.", |_| {});
    card(ui, false, |ui| {
        row(ui, "Assembler", |ui| { chip(ui, &crate::shell::assembler_label(), k.gr); ui.label(RichText::new("eingebaut (vc-asm)").small().weak()); });
        row(ui, "Include-Suche", |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("1. Ordner des Projekts").small());
                ui.label(RichText::new("2. Zusätzliche Ordner (unten: Include-Ordner, -I)").small());
                ui.horizontal(|ui| { ui.label(RichText::new("3. Systemdateien:").small()); ui.label(RichText::new(env.data.map_or("–".to_string(), |d| d.join("sys").display().to_string())).small().monospace().color(k.mute)); });
            });
        });
        ui.add_space(6.0);
        let spec = super::args::Spec { owner: "asm", title: "Assembler-Optionen beim Bauen", cat: vc_ide::args::catalog_for("vcc-asm"), ctx: vc_ide::args::ArgCtx::example("c64"), program: "vcc asm", hint: "z. B. -I /pfad/zu/includes -D DEBUG=1", reserved_note: false };
        super::args::editor(ui, &spec, &mut asm.opts, &mut asm.args, focus);
        ui.label(RichText::new("Gilt für das Bauen in der IDE (nicht für vcc auf der Kommandozeile). Konstanten aus der Projektdatei kommen zuerst, diese danach.").small().weak());
    });
    ui.add_space(12.0);
    section(ui, "BASIC-Dialekt je Zielsystem", "Das BASIC eines Projekts folgt dem Dialekt seines Zielsystems; je Quelle lässt er sich in der Projektdatei ändern.", |_| {});
    card(ui, false, |ui| {
        for s in vc_ide::targets::installed_systems(env.data) {
            let d = env.data.and_then(|d| crate::recent::target_dialect(d, s.id));
            row(ui, s.name, |ui| {
                match d { Some(d) => { chip(ui, crate::recent::dialect_label(&d), k.pu); } None => { chip(ui, "unbekannt", k.dim); } }
                if s.id == "atari8" { chip(ui, "auch Turbo BASIC XL", k.pu); }
            });
        }
        ui.label(RichText::new("BASIC (.bas, Zeilennummern optional) gibt es für alle Dialekte.").small().weak());
    });
    ui.add_space(12.0);
    section(ui, "Export beim Bauen", "Sprites, Zeichensätze und Songs werden vor dem Assemblieren nach build/export/ geschrieben, wenn sie veraltet sind.", |_| {});
    card(ui, false, |ui| {
        match &env.project {
            Some((name, on)) => {
                row(ui, "Projekt", |ui| { ui.label(RichText::new(name).strong()); chip(ui, if *on { "Auto-Export an" } else { "Auto-Export aus" }, if *on { k.gr } else { k.dim }); });
                ui.label(RichText::new("Gilt je Projekt und steht in dessen Einstellungen.").small().weak());
                if ui.button("Projekt-Einstellungen öffnen").clicked() { act = Act::OpenProjectSettings; }
            }
            None => { ui.label(RichText::new("Gilt je Projekt. Ohne geöffnetes Projekt gibt es nichts anzuzeigen.").weak()); }
        }
    });
    act
}

pub fn editor(ui: &mut egui::Ui, env: &mut Env) {
    section(ui, "Editor", "Gilt für alle Quelltexte und wirkt sofort.", |_| {});
    card(ui, false, |ui| {
        row(ui, "Schriftgröße", |ui| { ui.add(egui::Slider::new(&mut env.view.font, crate::theme::FONT_RANGE.0..=crate::theme::FONT_RANGE.1).step_by(1.0).suffix(" pt")); });
        row(ui, "Adressen", |ui| { ui.checkbox(env.show_addr, "Adressspalte im Editor (BASIC: Zeilennummern)").on_hover_text("Assembler: Speicheradresse jeder Zeile, vom Sprachdienst laufend berechnet."); });
        let mut hk = crate::bitmap::HOTKEYS.load(std::sync::atomic::Ordering::Relaxed);
        row(ui, "Grafik-Editoren", |ui| { if ui.checkbox(&mut hk, "Einzeltasten (D Stift, E Radierer, F Füllen …)").changed() { crate::bitmap::HOTKEYS.store(hk, std::sync::atomic::Ordering::Relaxed); } });
    });
}

pub fn keys(ui: &mut egui::Ui) {
    let m = if cfg!(target_os = "macos") { "Cmd" } else { "Strg" };
    section(ui, "Tastenkürzel", "Die Kürzel sind fest; Anpassen ist noch nicht möglich.", |_| {});
    let rows: Vec<(String, &str)> = vec![
        ("F5".into(), "Startprogramm bauen und starten"), ("F6".into(), "Bauen und Debuggen"), ("F7".into(), "Projekt bauen (mit Umschalt: aktive Datei)"),
        ("F8".into(), "Debugger: Weiter"), ("F10".into(), "Debugger: Über"), ("F11".into(), "Debugger: Schritt"), ("F1".into(), "Hilfe zum Wort am Cursor"),
        (format!("{m}+S"), "Speichern"), (format!("{m}+O"), "Projekt öffnen"), (format!("{m}+,"), "Einstellungen"), (format!("{m}+ +/−/0"), "Oberfläche vergrößern, verkleinern, zurücksetzen"),
        ("Strg+Leertaste".into(), "Vervollständigung im Editor"),
    ];
    card(ui, false, |ui| {
        egui::Grid::new("keys_grid").num_columns(2).spacing([24.0, 6.0]).show(ui, |ui| {
            for (key, what) in rows { ui.label(RichText::new(key).monospace().strong()); ui.label(what); ui.end_row(); }
        });
    });
}
