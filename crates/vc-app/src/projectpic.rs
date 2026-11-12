// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektbild (`projektbild.png` im Projektordner) als Textur: für die Liste „Zuletzt geöffnet“ und die Projekt-Einstellungen.
//! Die Datei wird nur gelesen, wenn sie neu oder geändert ist (Änderungszeit, höchstens alle 2 Sekunden geprüft).
use eframe::egui;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

#[derive(Default)]
pub struct Thumb {
    tex: Option<egui::TextureHandle>,
    stamp: Option<SystemTime>,
    checked: Option<Instant>,
}

impl Thumb {
    /// Textur des Projektbilds im Ordner `root` (Größe in Punkten dazu) oder `None`, wenn es keines gibt
    pub fn get(&mut self, ctx: &egui::Context, root: &Path) -> Option<(&egui::TextureHandle, egui::Vec2)> {
        if self.checked.map_or(true, |t| t.elapsed() > Duration::from_secs(2)) {
            self.checked = Some(Instant::now());
            let file = vc_ide::livebild::preview_path(root);
            let stamp = std::fs::metadata(&file).and_then(|m| m.modified()).ok();
            if stamp != self.stamp || (stamp.is_some() && self.tex.is_none()) {
                self.stamp = stamp;
                self.tex = stamp.and_then(|_| image::open(&file).ok()).map(|img| {
                    let img = img.to_rgba8();
                    let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
                    ctx.load_texture("projektbild", ci, egui::TextureOptions::LINEAR)
                });
            }
        }
        self.tex.as_ref().map(|t| (t, t.size_vec2()))
    }
    /// Beim nächsten Aufruf neu prüfen (nach Speichern oder Löschen)
    pub fn refresh(&mut self) { self.checked = None; }
}

/// Größe, in die ein Bild passt (Seitenverhältnis bleibt)
pub fn fit(size: egui::Vec2, max: egui::Vec2) -> egui::Vec2 { let s = (max.x / size.x).min(max.y / size.y); size * s }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_keeps_aspect() {
        let r = fit(egui::vec2(384.0, 272.0), egui::vec2(112.0, 80.0));
        assert!((r.x / r.y - 384.0 / 272.0).abs() < 0.01 && r.x <= 112.0 && r.y <= 80.0);
    }
    #[test]
    fn missing_file_gives_none() {
        let ctx = egui::Context::default();
        let mut t = Thumb::default();
        assert!(t.get(&ctx, Path::new("/nonexistent-vc-dir")).is_none());
    }
}
