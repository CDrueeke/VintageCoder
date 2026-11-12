// SPDX-License-Identifier: GPL-3.0-or-later
//! Seite „Über VintageCoder“: Name, Version (aus Cargo), Lizenz, Urheber, Dank, Projektseite und die Drittanbieter-Hinweise.
//! Die Seite ist ein Fenster über der Oberfläche; sie ruft nichts aus dem Netz ab (der Link öffnet nur auf Klick).
use eframe::egui::{self, RichText};

/// Projektseite und Quelltext-Repository (ein Klick öffnet sie im Browser, es gibt keinen Abruf)
pub const WEBSITE: &str = "https://vintagecoder.drueeke.de";
pub const REPOSITORY: &str = "https://github.com/CDrueeke/VintageCoder";
/// Logo von drueeke.de (helle Schrift für dunklen Grund, mit Transparenz; im hellen Schema dunkel eingefärbt)
const LOGO: &[u8] = include_bytes!("../assets/drueeke-logo-light.png");
pub const LICENSE: &str = "GPL-3.0-or-later";
/// Inhalt von `THIRD-PARTY-NOTICES.md`, beim Bau eingebettet
pub const NOTICES: &str = include_str!("../../../THIRD-PARTY-NOTICES.md");

/// Versionsnummer aus dem Cargo-Paket
pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }

/// Überschrift mit Version, z. B. „VintageCoder 0.1.0“
pub fn title_line() -> String { format!("VintageCoder {}", version()) }

/// Fenster zeigen; `open` wird beim Schließen falsch. Gibt zurück, ob „Feedback senden …“ angeklickt wurde.
pub fn show(ctx: &egui::Context, open: &mut bool) -> bool {
    let mut feedback = false;
    egui::Window::new("Über VintageCoder").open(open).collapsible(false).default_size([520.0, 460.0]).show(ctx, |ui| feedback = body(ui));
    feedback
}

/// Logo als Textur (einmal dekodiert, dann aus dem Speicher der Oberfläche)
fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("about-logo");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) { return Some(t); }
    let img = image::load_from_memory(LOGO).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let t = ctx.load_texture("drueeke-logo", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

fn body(ui: &mut egui::Ui) -> bool {
    if let Some(t) = logo(ui.ctx()) {
        let tint = if ui.visuals().dark_mode { egui::Color32::WHITE } else { egui::Color32::from_rgb(0x2b, 0x33, 0x3a) };
        ui.vertical_centered(|ui| { ui.add(egui::Image::new(&t).fit_to_exact_size(egui::vec2(133.0, 105.0)).tint(tint)); });
        ui.add_space(4.0);
    }
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title_line()).strong().size(24.0));
    });
    ui.add_space(6.0);
    ui.label("Entwicklungsumgebung für Retro-Computer: Assembler, BASIC, Grafik- und Klang-Editoren, Debugger und Emulator-Anbindung für C64, Atari, MEGA65 und weitere Systeme.");
    ui.add_space(6.0);
    ui.label(RichText::new("Alpha-Version: enthält wahrscheinlich noch viele Fehler. Keine Gewährleistung, keine Haftung für Datenverlust (siehe Lizenz).").color(crate::theme::warn()));
    ui.add_space(8.0);
    egui::Grid::new("about-grid").num_columns(2).spacing([12.0, 4.0]).show(ui, |ui| {
        ui.label(RichText::new("Lizenz").weak()); ui.label(LICENSE); ui.end_row();
        ui.label(RichText::new("Autor").weak()); ui.label("Christian Drüeke"); ui.end_row();
        ui.label(RichText::new("Co-Autor").weak()); ui.label("Claude Code (Anthropic)"); ui.end_row();
        ui.label(RichText::new("Dank").weak()); ui.label("an die Community „N3rdroom“ (Discord)"); ui.end_row();
        ui.label(RichText::new("Webseite").weak()); ui.hyperlink_to(WEBSITE, WEBSITE); ui.end_row();
        ui.label(RichText::new("Quelltext").weak()); ui.hyperlink_to(REPOSITORY, REPOSITORY); ui.end_row();
    });
    ui.add_space(8.0);
    let feedback = ui.button("Feedback senden …").on_hover_text("Fehler oder Wunsch per E-Mail melden; zeigt vorher, was in der Mail steht").clicked();
    ui.add_space(4.0);
    egui::CollapsingHeader::new("Drittanbieter-Hinweise (THIRD-PARTY-NOTICES)").show(ui, |ui| {
        egui::ScrollArea::vertical().max_height(260.0).auto_shrink([false, true]).show(ui, |ui| { let _ = crate::markdown::render(ui, NOTICES); });
    });
    feedback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_kommt_aus_cargo() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert!(title_line().starts_with("VintageCoder ") && title_line().ends_with(version()));
        assert!(version().split('.').count() == 3);
    }

    #[test]
    fn hinweise_sind_eingebettet() { assert!(NOTICES.len() > 100); assert!(image::load_from_memory(LOGO).is_ok()); }

    #[test]
    fn seite_laesst_sich_zeichnen() {
        let ctx = egui::Context::default();
        let mut open = true;
        for _ in 0..3 { let mut o = ctx.run_ui(Default::default(), |ui| { assert!(!show(ui.ctx(), &mut open)); }); o.textures_delta.clear(); }
        assert!(open);
    }
}
