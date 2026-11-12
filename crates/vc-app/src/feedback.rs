// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Feedback senden“: zeigt Betreff und Text der vorbereiteten Mail (Angaben aus `vc_ide::feedback`) zur Kontrolle.
//! Gesendet wird nichts: „Im Mailprogramm öffnen“ übergibt den `mailto:`-Link an das Standard-Mailprogramm, dort schickt der Anwender selbst ab.
use eframe::egui::{self, RichText};
use vc_ide::feedback::{self, Mail};

/// Offener Dialog: die Mail wird beim Öffnen einmal gebildet (Angaben ändern sich während des Dialogs nicht)
pub struct Dlg { mail: Mail, status: Option<(bool, String)> }

impl Dlg {
    /// `target`: Zielsystem des offenen Projekts (kein Projektname)
    pub fn new(target: Option<&str>) -> Dlg { Dlg { mail: feedback::current(target), status: None } }
}

/// Ordner im Dateimanager öffnen
pub fn open_folder(dir: &std::path::Path) {
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(dir).spawn(); }
    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(dir).spawn(); }
    #[cfg(all(unix, not(target_os = "macos")))] { let _ = std::process::Command::new("xdg-open").arg(dir).spawn(); }
}

/// Dialog zeichnen; schließt sich bei „Abbrechen“ oder dem Schließknopf des Fensters (`dlg` wird `None`)
pub fn show(ctx: &egui::Context, dlg: &mut Option<Dlg>) {
    let Some(d) = dlg.as_mut() else { return };
    let mut open = true;
    let mut close = false;
    egui::Window::new("Feedback senden").open(&mut open).collapsible(false).default_size([560.0, 580.0]).show(ctx, |ui| {
        ui.label(format!("Die Mail geht an {}. Sie wird nur vorbereitet: gesendet wird erst, wenn du in deinem Mailprogramm auf „Senden“ klickst; hier wird nichts ins Netz geschickt.", feedback::ADDRESS));
        ui.add_space(4.0);
        ui.label("Beschreibe den Fehler in deinem Mailprogramm in der leeren Zeile über den Angaben; Screenshots und Dateien hängst du dort selbst an.");
        ui.add_space(8.0);
        ui.label(RichText::new("Betreff").strong());
        ui.add(egui::TextEdit::singleline(&mut d.mail.subject.as_str()).desired_width(f32::INFINITY));
        ui.add_space(6.0);
        ui.label(RichText::new("Text").strong());
        egui::ScrollArea::vertical().max_height(280.0).id_salt("feedback-body").show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut d.mail.body.as_str()).desired_width(f32::INFINITY).desired_rows(10).font(egui::TextStyle::Monospace));
        });
        if !d.mail.dropped.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Gekürzt: wegen der Längengrenze von Mail-Links fehlen {}. „In Zwischenablage kopieren“ enthält alles.", d.mail.dropped.join(", "))).color(crate::theme::warn()));
        }
        if d.mail.too_long { ui.label(RichText::new("Der Link ist trotz Kürzung sehr lang; manche Mailprogramme schneiden ihn ab. Nimm dann die Zwischenablage.").color(crate::theme::warn())); }
        ui.add_space(4.0);
        ui.label(RichText::new("Pfade in deinem persönlichen Ordner stehen als ~; Projekt- und Dateinamen kommen nicht vor.").weak().small());
        ui.add_space(6.0);
        ui.label(RichText::new("Fehler schwer zu beschreiben? Schalte in den Einstellungen → Allgemein das Debug-Protokoll ein, wiederhole den Fehler und hänge die Datei an:").small());
        ui.horizontal(|ui| {
            let mut on = vc_ide::debuglog::enabled();
            if ui.checkbox(&mut on, "Debug-Protokoll schreiben").changed() { let mut s = vc_ide::settings::get(); s.debug_log = on; let _ = vc_ide::settings::set(s); vc_ide::debuglog::set_enabled(on); }
            if ui.button("Protokoll zeigen").on_hover_text("Öffnet den Ordner mit der Protokolldatei; angehängt wird nichts automatisch").clicked() {
                match vc_ide::debuglog::dir() { Some(dir) => { let _ = std::fs::create_dir_all(&dir); open_folder(&dir); } None => d.status = Some((false, "Kein Konfigurationsordner gefunden.".to_string())) }
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Im Mailprogramm öffnen").clicked() {
                d.status = Some(match feedback::open_mailto(&d.mail.url) {
                    Ok(()) => (true, "Mailprogramm geöffnet. Gesendet wird erst dort.".to_string()),
                    Err(e) => (false, format!("Mailprogramm nicht gestartet ({e}). Nimm „In Zwischenablage kopieren“.")),
                });
            }
            if ui.button("In Zwischenablage kopieren").clicked() { ui.ctx().copy_text(d.mail.clipboard()); d.status = Some((true, "In die Zwischenablage kopiert.".to_string())); }
            if ui.button("Abbrechen").clicked() { close = true; }
        });
        if let Some((ok, t)) = &d.status { ui.add_space(4.0); ui.label(RichText::new(t).color(if *ok { crate::theme::ok() } else { crate::theme::warn() })); }
    });
    if !open || close { *dlg = None; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_laesst_sich_zeichnen_und_schliessen() {
        let ctx = egui::Context::default();
        let mut dlg = Some(Dlg::new(Some("c64")));
        for _ in 0..3 { let mut o = ctx.run_ui(Default::default(), |ui| show(ui.ctx(), &mut dlg)); o.textures_delta.clear(); }
        assert!(dlg.is_some());
        assert!(dlg.as_ref().unwrap().mail.body.contains("Zielsystem: c64"));
    }
}
