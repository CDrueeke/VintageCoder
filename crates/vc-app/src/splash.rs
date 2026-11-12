// SPDX-License-Identifier: GPL-3.0-or-later
//! Startbildschirm („Splash“): Name, Version, Hinweis auf den Alpha-Stand und Haftungsausschluss.
//! Er liegt beim Programmstart über der ganzen Oberfläche und verschwindet erst nach einer Bestätigung
//! (Knopf, Eingabe-, Leer- oder Escape-Taste). Er ruft nichts aus dem Netz ab.
use eframe::egui::{self, Align, Color32, Layout, RichText};

/// App-Symbol (wie im Programmpaket), beim Bau eingebettet
const ICON: &[u8] = include_bytes!("../../../installer/macos/VintageCoder-1024.png");

/// Kurzer Hinweis zum Entwicklungsstand
pub const ALPHA_TITLE: &str = "Alpha-Version: nicht für wichtige Arbeit gedacht";
pub const ALPHA_TEXT: &str = "VintageCoder befindet sich im Alpha-Stadium und enthält mit hoher Wahrscheinlichkeit noch viele Fehler. \
Funktionen können unvollständig sein, sich ändern oder ausfallen; das Programm kann abstürzen.";

/// Haftungsausschluss (sinngemäß GPL-3.0-or-later, Abschnitte 15 und 16)
pub const DISCLAIMER_TITLE: &str = "Haftungsausschluss";
pub const DISCLAIMER_TEXT: &str = "Die Software wird „wie besehen“ und ohne jede Gewährleistung bereitgestellt. \
Die Nutzung erfolgt auf eigene Gefahr. Der Autor und die Mitwirkenden haften nicht für Datenverlust, beschädigte oder überschriebene Projekte, \
Disketten- und Speicherabbilder, Schäden an angeschlossener Hardware oder Emulatoren sowie für sonstige direkte oder indirekte Schäden, \
soweit das Gesetz das zulässt. Sichere deine Projekte und Daten regelmäßig (zum Beispiel mit Git oder einer Datensicherung), \
bevor du VintageCoder verwendest, besonders bevor du Programme oder Abbilder auf echte Geräte überträgst.";

/// Zustand des Startbildschirms
pub struct Splash { pub open: bool, /** Haken „nicht mehr anzeigen“ (gilt nur für die laufende Version) */ pub hide_next: bool }

fn file() -> Option<std::path::PathBuf> { Some(vc_core::userdirs::config_dir()?.join("splash.json")) }

/// Sagt die Datei, dass der Bildschirm für genau diese Version ausgeblendet ist?
fn hidden_for(text: Option<&str>, version: &str) -> bool {
    text.and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok()).map_or(false, |j| j["hidden_for_version"].as_str() == Some(version))
}

impl Splash {
    /// Beim Programmstart sichtbar, außer der Nutzer hat ihn für diese Version abgewählt; eine neue Version zeigt ihn immer wieder
    pub fn new() -> Self {
        let text = file().and_then(|f| std::fs::read_to_string(f).ok());
        Splash { open: !hidden_for(text.as_deref(), crate::about::version()), hide_next: false }
    }

    /// Schließen; mit gesetztem Haken wird die laufende Version gemerkt (Selbsttests schreiben nichts)
    fn close(&mut self) {
        self.open = false;
        if self.hide_next && std::env::var("VC_SHOT").is_err() {
            if let Some(f) = file() {
                if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
                let _ = std::fs::write(f, format!("{}\n", serde_json::json!({ "hidden_for_version": crate::about::version() })));
            }
        }
    }
}

impl Default for Splash { fn default() -> Self { Self::new() } }

/// Symbol als Textur (einmal dekodiert, dann aus dem Speicher der Oberfläche)
fn icon(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("splash-icon");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) { return Some(t); }
    let img = image::load_from_memory(ICON).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let t = ctx.load_texture("splash-icon", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

/// Fünf schmale Farbbalken in den Farben des Commodore-Regenbogens als Markenzeichen
fn stripes(ui: &mut egui::Ui, width: f32) {
    const C: [Color32; 5] = [Color32::from_rgb(0xD2, 0x40, 0x3C), Color32::from_rgb(0xE8, 0x89, 0x2B), Color32::from_rgb(0xF2, 0xD3, 0x3B), Color32::from_rgb(0x5B, 0xB8, 0x5B), Color32::from_rgb(0x4A, 0x8F, 0xD9)];
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, 6.0), egui::Sense::hover());
    let w = r.width() / C.len() as f32;
    for (i, c) in C.iter().enumerate() {
        ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(r.left() + w * i as f32, r.top()), egui::vec2(w + 0.5, r.height())), 0.0, *c);
    }
}

/// Hinweiskasten mit farbigem Rand und Überschrift
fn notice(ui: &mut egui::Ui, color: Color32, title: &str, text: &str) {
    egui::Frame::new().fill(color.gamma_multiply(0.10)).stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.7))).corner_radius(6.0).inner_margin(egui::Margin::symmetric(14, 10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(title).strong().color(color).size(14.0));
        ui.add_space(3.0);
        ui.label(RichText::new(text).size(13.0));
    });
}

/// Startbildschirm zeichnen; schließt sich (`open = false`) auf Knopf oder Taste
pub fn show(ctx: &egui::Context, s: &mut Splash) {
    if !s.open { return; }
    let k = crate::ui::tokens::t();
    let screen = ctx.content_rect();
    // Hintergrund: deckt die Oberfläche vollständig ab und fängt alle Klicks
    egui::Area::new(egui::Id::new("splash-bg")).order(egui::Order::Tooltip).fixed_pos(screen.min).show(ctx, |ui| {
        let (r, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click_and_drag());
        ui.painter().rect_filled(r, 0.0, k.bg0);
    });
    let width = (screen.width() - 40.0).clamp(320.0, 680.0);
    egui::Area::new(egui::Id::new("splash-card")).order(egui::Order::Tooltip).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(10.0).inner_margin(egui::Margin::same(0))
            .shadow(egui::Shadow { offset: [0, 10], blur: 36, spread: 4, color: Color32::from_black_alpha(140) }).show(ui, |ui| {
                ui.set_width(width);
                stripes(ui, width);
                let mut content = |ui: &mut egui::Ui| {
                    egui::Frame::new().inner_margin(egui::Margin::symmetric(26, 20)).show(ui, |ui| {
                        ui.set_width(width - 52.0);
                        ui.horizontal(|ui| {
                            if let Some(t) = icon(ui.ctx()) { ui.add(egui::Image::new(&t).fit_to_exact_size(egui::vec2(88.0, 88.0))); }
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.add_space(4.0);
                                ui.label(RichText::new("VintageCoder").size(34.0).strong().color(k.fg));
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("Version {}", crate::about::version())).size(14.0).color(k.mute));
                                    ui.add_space(4.0);
                                    egui::Frame::new().fill(k.am).corner_radius(4.0).inner_margin(egui::Margin::symmetric(8, 1)).show(ui, |ui| {
                                        ui.label(RichText::new("ALPHA").strong().size(13.0).color(Color32::from_rgb(0x1a, 0x14, 0x00)));
                                    });
                                });
                                ui.label(RichText::new("Entwicklungsumgebung für Retro-Computer").size(13.0).color(k.mute));
                            });
                        });
                        ui.add_space(14.0);
                        notice(ui, k.am, ALPHA_TITLE, ALPHA_TEXT);
                        ui.add_space(10.0);
                        notice(ui, k.cy, DISCLAIMER_TITLE, DISCLAIMER_TEXT);
                        ui.add_space(12.0);
                        ui.checkbox(&mut s.hide_next, RichText::new("Beim nächsten Start nicht mehr anzeigen (bei einer neuen Version erscheint er wieder)").size(13.0));
                        ui.add_space(10.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let go = ui.add(egui::Button::new(RichText::new("Verstanden, los geht’s").strong().size(14.0)).min_size(egui::vec2(190.0, 32.0)));
                            if go.clicked() { s.close(); }
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("Lizenz {}", crate::about::LICENSE)).size(12.0).color(k.dim));
                        });
                    });
                };
                if screen.height() < 700.0 { egui::ScrollArea::vertical().max_height((screen.height() - 80.0).max(200.0)).auto_shrink([false, true]).show(ui, content); } else { content(ui); }
            });
    });
    if ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Escape)) { s.close(); }
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hinweise_nennen_alpha_und_haftung() {
        assert!(ALPHA_TEXT.contains("Alpha") && ALPHA_TEXT.contains("Fehler"));
        assert!(DISCLAIMER_TEXT.contains("Datenverlust") && DISCLAIMER_TEXT.contains("Gewährleistung"));
        assert!(image::load_from_memory(ICON).is_ok());
    }

    #[test]
    fn ausblenden_gilt_nur_fuer_die_gemerkte_version() {
        assert!(hidden_for(Some(r#"{"hidden_for_version":"0.1.0"}"#), "0.1.0"));
        assert!(!hidden_for(Some(r#"{"hidden_for_version":"0.1.0"}"#), "0.2.0"));
        assert!(!hidden_for(None, "0.1.0") && !hidden_for(Some("kaputt"), "0.1.0") && !hidden_for(Some("{}"), "0.1.0"));
    }

    #[test]
    fn zeichnet_und_bleibt_offen_ohne_eingabe() {
        let ctx = egui::Context::default();
        let mut s = Splash { open: true, hide_next: false };
        for _ in 0..3 { let mut o = ctx.run_ui(Default::default(), |ui| show(ui.ctx(), &mut s)); o.textures_delta.clear(); }
        assert!(s.open);
    }

    #[test]
    fn geschlossen_zeichnet_nichts() {
        let ctx = egui::Context::default();
        let mut s = Splash { open: false, hide_next: false };
        let mut o = ctx.run_ui(Default::default(), |ui| show(ui.ctx(), &mut s));
        o.textures_delta.clear();
        assert!(!s.open && o.shapes.is_empty());
    }
}
