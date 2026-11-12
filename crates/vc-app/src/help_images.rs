// SPDX-License-Identifier: GPL-3.0-or-later
//! Handbuchbilder in der Hilfe: Zustimmung, Laden im Hintergrund (`vc_ide::help_images`), Texturen erst bei Bedarf (sichtbarer Platzhalter),
//! Platzhalter mit Alt-Text und Unterschrift solange das Bild fehlt, Vergrößern per Klick. Fehler werden angezeigt, nie als Absturz.
use crate::ui::tokens::t as tok;
use eframe::egui::{self, RichText};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use vc_ide::help_images::{self as hi, Consent};

enum Slot {
    Loading,
    Ready(egui::TextureHandle),
    /// Fehlertext (Netz, Datei, Format); „Erneut versuchen“ löscht den Eintrag
    Failed(String),
    /// nicht geladen, weil keine Zustimmung vorliegt und das Bild nicht im Zwischenspeicher ist
    Skipped,
}

type Done = (String, Result<egui::ColorImage, String>);

pub struct HelpImages {
    /// Einstellung (`help.images`); `session` gilt nur bis zum Programmende
    pub consent: Consent,
    session: Option<bool>,
    slots: HashMap<String, Slot>,
    tx: Sender<Done>,
    rx: Receiver<Done>,
    /// vergrößert angezeigtes Bild
    pub zoom: Option<String>,
}

impl Default for HelpImages {
    fn default() -> Self {
        let (tx, rx) = channel();
        HelpImages { consent: Consent::from_setting(&vc_ide::settings::get().help.images), session: None, slots: HashMap::new(), tx, rx, zoom: None }
    }
}

/// Alle Bildkennungen eines Markdown-Textes, die das Handbuch kennt (unbekannte lässt die Hilfe weg)
pub fn known_ids(md: &str) -> Vec<String> {
    md.lines().filter_map(|l| crate::markdown::parse_image(l)).map(|(_, id, _)| id).filter(|id| vc_ide::help::handbook_image_known(id)).collect()
}

impl HelpImages {
    /// Einstellung neu lesen (Einstellungsseite oder Hilfe haben sie geändert); eine Änderung gibt zurückgestellte Bilder frei
    pub fn sync(&mut self) {
        let c = Consent::from_setting(&vc_ide::settings::get().help.images);
        if c != self.consent { self.consent = c; self.session = None; self.slots.retain(|_, s| !matches!(s, Slot::Skipped)); }
    }
    fn allowed(&self) -> bool { !hi::is_remote(&hi::base()) || self.consent == Consent::Always || self.session == Some(true) }
    /// Muss gefragt werden, bevor dieser Text angezeigt wird? (Bilder daraus kommen aus dem Netz und liegen nicht im Zwischenspeicher)
    pub fn needs_prompt(&self, md: &str) -> bool {
        if self.consent != Consent::Ask || self.session.is_some() { return false; }
        let ids = known_ids(md);
        !ids.is_empty() && hi::is_remote(&hi::base()) && ids.iter().any(|id| !self.slots.contains_key(id) && !hi::is_cached(id))
    }
    /// Für diese Sitzung laden (`true`) bzw. nicht laden (`false`)
    pub fn answer_session(&mut self, allow: bool) { self.session = Some(allow); self.slots.retain(|_, s| !matches!(s, Slot::Skipped)); }
    pub fn base_host(&self) -> String { let b = hi::base(); b.split("://").nth(1).unwrap_or(&b).split('/').next().unwrap_or("").to_string() }

    /// Fertige Hintergrundergebnisse abholen und Texturen anlegen
    pub fn poll(&mut self, ctx: &egui::Context) {
        while let Ok((id, r)) = self.rx.try_recv() {
            let slot = match r { Ok(img) => Slot::Ready(ctx.load_texture(format!("handbuch-{id}"), img, egui::TextureOptions::LINEAR)), Err(e) => Slot::Failed(e) };
            self.slots.insert(id, slot);
        }
    }

    fn start(&mut self, ctx: &egui::Context, id: &str) {
        let allow = self.allowed();
        // Bild im Zwischenspeicher oder lokal: Lesen kostet kein Netz; sonst nur mit Zustimmung
        if !allow && !hi::is_cached(id) { self.slots.insert(id.to_string(), Slot::Skipped); return; }
        self.slots.insert(id.to_string(), Slot::Loading);
        let (tx, ctx, id) = (self.tx.clone(), ctx.clone(), id.to_string());
        std::thread::spawn(move || {
            let r = (|| -> Result<egui::ColorImage, String> {
                let bytes = match hi::cached(&id) { Some(b) => b, None if allow => hi::fetch(&id)?, None => return Err("nicht geladen".into()) };
                let img = image::load_from_memory(&bytes).map_err(|e| format!("{id}: {e}"))?.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                Ok(egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
            })();
            let _ = tx.send((id, r));
            ctx.request_repaint();
        });
    }

    fn placeholder(&mut self, ui: &mut egui::Ui, id: &str, alt: &str, status: &str, retry: bool) {
        let k = tok();
        let r = egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(6.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
            ui.set_width(ui.available_width().min(760.0));
            ui.set_min_height(70.0);
            ui.label(RichText::new(format!("Bild: {alt}")).color(k.mute));
            ui.label(RichText::new(status).small().weak());
            if retry && ui.small_button("Erneut versuchen").clicked() { self.slots.remove(id); }
        });
        // erst laden, wenn der Platzhalter im sichtbaren Bereich liegt
        if !self.slots.contains_key(id) && ui.is_rect_visible(r.response.rect) { self.start(ui.ctx(), id); }
    }

    /// Das vergrößerte Bild (Klick oder Esc schließt)
    pub fn zoom_ui(&mut self, ctx: &egui::Context) {
        let Some(id) = self.zoom.clone() else { return };
        let Some(Slot::Ready(tex)) = self.slots.get(&id) else { self.zoom = None; return };
        let caption = vc_ide::help::handbook_images().iter().find(|(i, _, _)| *i == id).map(|(_, _, c)| c.clone()).unwrap_or_default();
        let screen = ctx.content_rect().size();
        let ts = tex.size_vec2();
        let scale = ((screen.x * 0.9) / ts.x).min((screen.y * 0.8) / ts.y).min(2.0);
        let m = egui::Modal::new(egui::Id::new("help_zoom")).show(ctx, |ui| {
            let r = ui.add(egui::Image::new((tex.id(), ts * scale)).sense(egui::Sense::click()));
            if !caption.is_empty() { ui.label(RichText::new(&caption).small().weak()); }
            r.clicked()
        });
        if m.inner || m.should_close() { self.zoom = None; }
    }
}

impl crate::markdown::Images for HelpImages {
    fn image(&mut self, ui: &mut egui::Ui, id: &str, alt: &str, caption: &str) {
        if !vc_ide::help::handbook_image_known(id) { return; }   // noch nicht aufgenommen: weglassen
        ui.add_space(4.0);
        match self.slots.get(id) {
            Some(Slot::Ready(tex)) => {
                let ts = tex.size_vec2();
                let w = ui.available_width().min(760.0).min(ts.x);
                let r = ui.add(egui::Image::new((tex.id(), egui::vec2(w, w * ts.y / ts.x))).corner_radius(4.0).sense(egui::Sense::click()));
                let r = r.on_hover_text(format!("{alt}\n(Klick: vergrößern)")).on_hover_cursor(egui::CursorIcon::PointingHand);
                if r.clicked() { self.zoom = Some(id.to_string()); }
            }
            Some(Slot::Loading) => { self.placeholder(ui, id, alt, "wird geladen …", false); }
            Some(Slot::Failed(e)) => { let e = e.clone(); self.placeholder(ui, id, alt, &format!("nicht verfügbar: {e}"), true); }
            Some(Slot::Skipped) => { self.placeholder(ui, id, alt, "nicht geladen (Bilder aus dem Internet laden: siehe Hinweis oben bzw. Einstellungen > Allgemein > Hilfe)", false); }
            None => { self.placeholder(ui, id, alt, "", false); }
        }
        if !caption.is_empty() { ui.label(RichText::new(caption).small().italics().weak()); }
        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kapitel mit Bildern, Hinweisboxen, Listen und Links durch den echten Renderer (ohne Fenster): Bilder werden aus dem lokalen Ordner geladen,
    /// fehlende Kennungen weggelassen, es gibt keinen Absturz; ein zweiter Lauf mit unerreichbarer Adresse zeigt nur Platzhalter.
    #[test]
    fn kapitel_laesst_sich_ohne_fenster_zeichnen() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/handbuch/bilder");
        std::env::set_var("VC_HELP_IMAGES", &dir);
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let md = vc_ide::help::topic("kapitel:erstes-projekt").unwrap()["markdown"].as_str().unwrap().to_string();
        let wanted = known_ids(&md);
        assert!(!wanted.is_empty());
        let mut imgs = HelpImages::default();
        let mut clicked = None;
        for _ in 0..400 {
            imgs.poll(&ctx);
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 20000.0))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| { egui::CentralPanel::default().show_inside(ui, |ui| { if let Some(k) = crate::markdown::render_with(ui, &md, Some(&mut imgs)) { clicked = Some(k); } }); });
            out.textures_delta.clear();
            if wanted.iter().all(|id| matches!(imgs.slots.get(id), Some(Slot::Ready(_)))) { break; }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        for id in &wanted { assert!(matches!(imgs.slots.get(id), Some(Slot::Ready(_))), "Bild {id} nicht geladen"); }
        assert!(clicked.is_none() && !imgs.needs_prompt(&md), "lokaler Ordner braucht keine Zustimmung");
        // offline: Adresse nicht erreichbar → erst Nachfrage, ohne Zustimmung nur Platzhalter, mit Zustimmung Fehlerplatzhalter (kein Absturz)
        let tmp = std::env::temp_dir().join(format!("vc-help-ui-{}", std::process::id()));
        std::env::set_var("VC_HELP_IMAGES", "https://nicht-erreichbar.invalid/bilder");
        std::env::set_var("VC_HELP_CACHE", &tmp);
        let frame = |ctx: &egui::Context, imgs: &mut HelpImages| {
            imgs.poll(ctx);
            let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 20000.0))), ..Default::default() };
            let mut out = ctx.run_ui(input, |ui| { egui::CentralPanel::default().show_inside(ui, |ui| { let _ = crate::markdown::render_with(ui, &md, Some(imgs)); }); });
            out.textures_delta.clear();
        };
        let mut off = HelpImages::default(); off.consent = Consent::Ask;
        assert!(off.needs_prompt(&md));
        off.answer_session(false);
        assert!(!off.needs_prompt(&md));
        for _ in 0..5 { frame(&ctx, &mut off); }
        assert!(wanted.iter().all(|id| matches!(off.slots.get(id), Some(Slot::Skipped))), "ohne Zustimmung wird nichts geladen");
        let mut on = HelpImages::default(); on.consent = Consent::Always;
        for _ in 0..400 {
            frame(&ctx, &mut on);
            if wanted.iter().all(|id| matches!(on.slots.get(id), Some(Slot::Failed(_)))) { break; }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        for id in &wanted { assert!(matches!(on.slots.get(id), Some(Slot::Failed(_))), "Bild {id}: Fehlerplatzhalter erwartet"); }
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::remove_var("VC_HELP_IMAGES"); std::env::remove_var("VC_HELP_CACHE");
    }

    #[test]
    fn bekannte_kennungen_ohne_fehlende() {
        let md = "![A](bild:ui-gesamt \"x\")\n![B](bild:gibt-es-nicht-xyz \"y\")\nText\n";
        assert_eq!(known_ids(md), vec!["ui-gesamt".to_string()]);
    }
}
