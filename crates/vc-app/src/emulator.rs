// SPDX-License-Identifier: GPL-3.0-or-later
//! Reiter „Emulator“ (Debugger): das Bild des laufenden VICE-Emulators in der IDE, daneben Speicher und Texteingabe.
//!
//! Das Bild kommt über den Monitor des Emulators (`display get`), nicht aus einem eingebetteten Fenster:
//! - **Bild beim Halt** (Standard): bei jedem Halt (Haltepunkt, Einzelschritt, Pause) wird das Bild geholt.
//! - **Zeitlupen-Live** (auf Knopfdruck): der Arbeits-Thread des Debuggers hält die Maschine alle paar Millisekunden an, holt das Bild und
//!   setzt sie fort. Die Maschine läuft dabei mit etwa 40–80 % ihres Tempos und ohne Ton.
//!
//! Zustand und Befehle liegen in `debug.rs` (Zustandsabbild `Snap`, Befehle `Frame`, `FrameMode`, `FrameSource`, `Text`).
use crate::debug::{Cmd, Debugger, Snap};
use crate::ui::tokens::{t, tint};
use crate::ui::widgets;
use eframe::egui::{self, Color32, RichText};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vc_emu::streams::video::Frame;

#[derive(Default)]
pub struct EmuState {
    tex: Option<egui::TextureHandle>,
    tex_seq: u64,
    frame: Option<Arc<Frame>>,
    /// Zeitlupen-Live gewählt
    pub slow: bool,
    /// zuletzt an den Debugger gemeldeter Modus samt Kennung der Sitzung
    sent: Option<(usize, bool, bool)>,
    text: String,
    source: usize,
    msg: Option<(bool, String)>,
    /// letzter Zeitpunkt, zu dem der Reiter gezeichnet wurde (Aufpasser: Zeitlupe endet, wenn der Reiter nicht mehr zu sehen ist)
    seen: Option<Instant>,
}

enum Act { Grab, Save, Copy, Send, SetPic }

/// Kennung einer Sitzung (Adresse des geteilten Zustands)
fn session_id(d: &Debugger) -> usize { Arc::as_ptr(&d.snap) as *const () as usize }

impl EmuState {
    /// Schaltet Bildabruf und Zeitlupe aus (Reiter geschlossen oder Sitzung weg)
    pub fn stop(&mut self, d: Option<&Debugger>) {
        self.slow = false;
        if let (Some(d), Some(_)) = (d, self.sent.take()) { d.send(Cmd::FrameMode { auto: false, slow: false }); }
        self.sent = None;
    }

    /// Jede Sitzung beginnt ohne Bild und ohne Zeitlupe
    fn reset_for_new_session(&mut self) { self.frame = None; self.tex = None; self.tex_seq = 0; self.slow = false; self.source = 0; self.msg = None; }

    /// Aufpasser (jeden Zeichendurchgang): ist der Reiter nicht mehr zu sehen, endet die Zeitlupe, damit die Maschine nicht unbemerkt gebremst wird
    pub fn watchdog(&mut self, d: Option<&Debugger>) {
        let Some(d) = d else { return };
        if self.slow && self.seen.map_or(true, |t| t.elapsed() > Duration::from_millis(700)) {
            self.slow = false;
            if let Some((id, auto, _)) = self.sent { if id == session_id(d) { d.send(Cmd::FrameMode { auto, slow: false }); self.sent = Some((id, auto, false)); } }
        }
    }

    pub fn active(&self) -> bool { self.slow }

    fn pull_texture(&mut self, ctx: &egui::Context, sn: &Snap) {
        if sn.frame_seq != self.tex_seq {
            self.tex_seq = sn.frame_seq;
            if let Some(f) = &sn.frame {
                let img = egui::ColorImage::from_rgba_unmultiplied([f.width, f.height], &f.rgba);
                match &mut self.tex { Some(t) => t.set(img, egui::TextureOptions::NEAREST), None => self.tex = Some(ctx.load_texture("emulator-bild", img, egui::TextureOptions::NEAREST)) }
                self.frame = Some(f.clone());
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, d: Option<&Debugger>, root: Option<&Path>, mem_input: &mut String) {
        let k = t();
        self.seen = Some(Instant::now());
        let Some(d) = d else {
            if self.sent.is_some() { self.reset_for_new_session(); self.sent = None; }
            widgets::card(ui, false, |ui| {
                ui.label(RichText::new("Keine Emulator-Sitzung").strong());
                ui.label(RichText::new("Dieser Reiter zeigt das Bild des Emulators. Starte dein Programm mit „Debuggen“ (F6): VICE startet, und hier erscheint sein Bild, sobald die Maschine angehalten ist.").color(k.mute));
            });
            return;
        };
        let sn = d.snapshot();
        let sid = session_id(d);
        if self.sent.map_or(false, |(id, _, _)| id != sid) { self.reset_for_new_session(); self.sent = None; }
        if sn.starting { ui.label("Emulator startet …"); ui.ctx().request_repaint_after(Duration::from_millis(200)); return; }
        if sn.ended { ui.weak("Die Emulator-Sitzung ist beendet."); return; }
        if !sn.can_frame {
            widgets::card(ui, false, |ui| {
                ui.label(RichText::new("Dieser Emulator liefert kein Bild").strong());
                ui.label(RichText::new(format!("{} lässt sich nicht nach dem Bild fragen. Das Bild gibt es für VICE (C64, C128, VC-20, Plus/4); Atari und MEGA65 folgen nicht in dieser Ausbaustufe.", if sn.backend.is_empty() { "Der Emulator" } else { &sn.backend })).color(k.mute));
            });
            return;
        }
        // Modus beim Debugger anmelden (Bild beim Halt immer, Zeitlupe nur auf Wunsch)
        let want = (sid, true, self.slow);
        if self.sent != Some(want) { d.send(Cmd::FrameMode { auto: true, slow: self.slow }); self.sent = Some(want); }
        self.pull_texture(ui.ctx(), &sn);

        let mut act: Option<Act> = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Emulator").font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
            if !sn.backend.is_empty() { ui.label(RichText::new(format!("· {}", sn.backend)).color(k.dim)); }
            if self.slow { widgets::chip(ui, "Zeitlupen-Live", k.am); }
            else if sn.running { widgets::chip(ui, "läuft · Bild vom letzten Halt", k.dim); }
            else { widgets::chip(ui, "angehalten · Bild aktuell", k.gr); }
            ui.separator();
            if ui.button("Bild holen").on_hover_text("Holt das Bild jetzt (hält eine laufende Maschine kurz an und setzt sie fort)").clicked() { act = Some(Act::Grab); }
            let label = if self.slow { "Zeitlupen-Live beenden" } else { "Zeitlupen-Live" };
            if ui.selectable_label(self.slow, label).on_hover_text("Bild laufend aktualisieren: hält die Maschine alle paar Millisekunden an, sie läuft mit etwa 40–80 % Tempo und ohne Ton. Eine stehende Maschine wird dafür fortgesetzt.").clicked() {
                self.slow = !self.slow;
                if self.slow && !sn.running { d.send(Cmd::Cont); }      // eine stehende Maschine läuft los, sonst gäbe es nichts zu sehen
            }
            if sn.frame_sources.len() > 1 {
                let before = self.source;
                egui::ComboBox::from_id_salt("emu_source").selected_text(sn.frame_sources.get(self.source).cloned().unwrap_or_default()).show_ui(ui, |ui| { for (i, n) in sn.frame_sources.iter().enumerate() { ui.selectable_value(&mut self.source, i, n.as_str()); } });
                if before != self.source { d.send(Cmd::FrameSource(self.source)); }
            }
            let has = self.frame.is_some();
            if ui.add_enabled(has, egui::Button::new("Als PNG speichern")).on_hover_text("PNG im Projekt unter aufnahmen/").clicked() { act = Some(Act::Save); }
            if ui.add_enabled(has, egui::Button::new("In Zwischenablage")).clicked() { act = Some(Act::Copy); }
            if ui.add_enabled(has && root.is_some(), egui::Button::new("Als Projektbild setzen")).on_hover_text("Speichert das Bild (zugeschnitten) als projektbild.png im Projekt; es erscheint in „Zuletzt geöffnete Projekte“. Ein vorhandenes wird ersetzt.").clicked() { act = Some(Act::SetPic); }
        });
        if self.slow {
            ui.label(RichText::new("Zeitlupen-Live hält die Maschine immer wieder kurz an: sie läuft mit etwa 40–80 % ihres Tempos und ohne Ton. Zum Spielen ist das nicht gedacht; zum Beobachten von Änderungen reicht es.").small().color(k.am));
        }
        ui.add_space(4.0);

        let wide = ui.available_width() > 700.0;
        let area = ui.available_rect_before_wrap();
        let side_w = 330.0;
        let (img_area, side_area) = if wide { (egui::Rect::from_min_max(area.min, egui::pos2(area.right() - side_w - 12.0, area.bottom())), Some(egui::Rect::from_min_max(egui::pos2(area.right() - side_w, area.top()), area.max))) } else { (area, None) };
        self.image_area(ui, img_area, &sn);
        if let Some(sa) = side_area {
            ui.scope_builder(egui::UiBuilder::new().max_rect(sa), |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { self.side(ui, d, &sn, mem_input, &mut act); });
            });
        }
        ui.allocate_rect(area, egui::Sense::hover());

        match act {
            Some(Act::Grab) => { self.msg = None; d.send(Cmd::Frame); }
            Some(Act::Save) => self.msg = Some(self.save(root)),
            Some(Act::SetPic) => self.msg = Some(self.set_project_pic(root)),
            Some(Act::Copy) => self.msg = Some(self.copy(ui.ctx())),
            Some(Act::Send) => { let txt = std::mem::take(&mut self.text); self.msg = Some((true, if sn.running { "Text eingespeist.".into() } else { "Text eingespeist: die Maschine steht, er wird beim Fortsetzen eingegeben.".into() })); d.send(Cmd::Text(txt)); }
            None => {}
        }
        if self.slow { ui.ctx().request_repaint_after(Duration::from_millis(100)); }
    }

    fn image_area(&self, ui: &mut egui::Ui, rect: egui::Rect, sn: &Snap) {
        let k = t();
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 4.0, k.bg0);
        p.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside);
        let inner = rect.shrink(8.0);
        if let (Some(tex), Some(f)) = (&self.tex, &self.frame) {
            let (w, h) = (f.width as f32, f.height as f32);
            if inner.width() > 8.0 && inner.height() > 8.0 {
                let scale = (inner.width() / w).min(inner.height() / h);
                let r = egui::Rect::from_center_size(inner.center(), egui::vec2(w * scale, h * scale));
                p.image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
            }
            if sn.running && !self.slow { p.text(rect.left_top() + egui::vec2(14.0, 12.0), egui::Align2::LEFT_TOP, "Stand beim letzten Halt – die Maschine läuft", egui::FontId::proportional(12.0), k.am); }
        } else {
            let txt = if let Some(e) = &sn.frame_err { format!("Kein Bild: {e}") } else if sn.running { "Die Maschine läuft. Das Bild erscheint beim nächsten Halt\noder mit „Bild holen“ bzw. „Zeitlupen-Live“.".to_string() } else { "Warte auf das Bild …".to_string() };
            p.text(rect.center(), egui::Align2::CENTER_CENTER, txt, egui::FontId::proportional(14.0), k.dim);
        }
    }

    fn side(&mut self, ui: &mut egui::Ui, d: &Debugger, sn: &Snap, mem_input: &mut String, act: &mut Option<Act>) {
        let k = t();
        if let Some(f) = &self.frame { ui.label(RichText::new(format!("Bild: {} × {} Punkte · Nr. {}", f.width, f.height, sn.frame_seq)).small().color(k.dim)); }
        if let Some(e) = &sn.frame_err { ui.label(RichText::new(e).small().color(k.rd)); }
        if let Some((ok, text)) = &self.msg { ui.label(RichText::new(text).small().color(if *ok { k.gr } else { k.rd })); }
        ui.add_space(6.0);
        if sn.can_text {
            widgets::card(ui, false, |ui| {
                ui.label(RichText::new("Text einspeisen").strong());
                ui.add(egui::TextEdit::multiline(&mut self.text).desired_rows(2).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace).hint_text("z. B. RUN"));
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.text.is_empty(), egui::Button::new("Einspeisen")).on_hover_text("Schreibt den Text in den Tastaturpuffer; ein Zeilenumbruch ist RETURN").clicked() { *act = Some(Act::Send); }
                    if ui.add_enabled(true, egui::Button::new("RETURN")).clicked() { self.text.push('\n'); }
                });
                ui.label(RichText::new("Nur Text, zum Beispiel BASIC-Zeilen oder RUN. Zum Spielen reicht das nicht: Tastatur und Joystick werden nicht weitergegeben. Buchstaben kommen als Großbuchstaben an. Steht die Maschine, wird der Text beim Fortsetzen eingegeben.").small().color(k.dim));
            });
            ui.add_space(6.0);
        }
        widgets::card(ui, false, |ui| {
            ui.label(RichText::new("Speicher").strong());
            ui.horizontal(|ui| {
                ui.label("ab $");
                let r = ui.add(egui::TextEdit::singleline(mem_input).desired_width(52.0).font(egui::TextStyle::Monospace));
                let go = (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Lesen").clicked();
                if go { if let Ok(a) = u16::from_str_radix(mem_input.trim().trim_start_matches('$'), 16) { d.send(Cmd::Mem(a)); } }
                if ui.button("−256").clicked() { let a = sn.mem_addr.wrapping_sub(256); d.send(Cmd::Mem(a)); *mem_input = format!("{a:04X}"); }
                if ui.button("+256").clicked() { let a = sn.mem_addr.wrapping_add(256); d.send(Cmd::Mem(a)); *mem_input = format!("{a:04X}"); }
            });
            if sn.running && sn.mem.is_empty() { ui.label(RichText::new("Der Speicher wird beim nächsten Halt gelesen.").small().color(k.dim)); }
            else if sn.running { ui.label(RichText::new("Stand beim letzten Halt").small().color(k.am)); }
            egui::ScrollArea::vertical().id_salt("emu_mem").max_height(280.0).auto_shrink([false, true]).show(ui, |ui| {
                for (row, chunk) in sn.mem.chunks(16).enumerate() {
                    ui.add(egui::Label::new(crate::debugger::mem_job(sn.mem_addr.wrapping_add((row * 16) as u16), chunk, &sn.mem_prev, row)).extend());
                }
            });
        });
        let _ = tint;
    }

    fn save(&self, root: Option<&Path>) -> (bool, String) {
        let Some(f) = &self.frame else { return (false, "Noch kein Bild vorhanden.".into()) };
        let Some(root) = root else { return (false, "Zum Speichern zuerst ein Projekt öffnen.".into()) };
        let path = vc_ide::livebild::shot_path_with(root, "emulator-bild");
        match vc_ide::livebild::save_png(f, &path) {
            Ok(()) => (true, format!("Gespeichert: {}", path.strip_prefix(root).unwrap_or(&path).display())),
            Err(e) => (false, format!("Speichern fehlgeschlagen: {e}")),
        }
    }

    fn set_project_pic(&self, root: Option<&Path>) -> (bool, String) {
        let Some(f) = &self.frame else { return (false, "Noch kein Bild vorhanden.".into()) };
        let Some(root) = root else { return (false, "Zuerst ein Projekt öffnen.".into()) };
        match vc_ide::livebild::save_project_preview(root, f) {
            Ok(_) => (true, format!("Projektbild gesetzt: {}", vc_ide::projfile::PREVIEW_FILE)),
            Err(e) => (false, format!("Projektbild speichern fehlgeschlagen: {e}")),
        }
    }

    fn copy(&self, ctx: &egui::Context) -> (bool, String) {
        let Some(f) = &self.frame else { return (false, "Noch kein Bild vorhanden.".into()) };
        ctx.copy_image(egui::ColorImage::from_rgba_unmultiplied([f.width, f.height], &f.rgba));
        (true, "Bild in die Zwischenablage kopiert.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2, Rect};

    fn snap(frame: bool, running: bool) -> Snap {
        let mut sn = crate::debugger::demo_snap("vice");
        sn.can_frame = true; sn.can_text = true; sn.running = running; sn.starting = false;
        if frame { sn.frame = Some(Arc::new(Frame { width: 4, height: 3, rgba: vec![200; 4 * 3 * 4], frame_no: 1 })); sn.frame_seq = 1; }
        sn
    }

    /// Zeichentest in allen Zuständen: ohne Sitzung, ohne Bild, mit Bild (angehalten/läuft), Zeitlupe, schmal und breit
    #[test]
    fn emulator_zeichnet_in_allen_zustaenden() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        for (w, h) in [(330.0, 300.0), (1000.0, 420.0)] {
            for variant in ["keine", "ohne-bild", "bild", "laeuft", "zeitlupe", "kein-bild-moeglich"] {
                let mut s = EmuState::default();
                let dbg = match variant {
                    "keine" => None,
                    "ohne-bild" => Some(Debugger::demo(snap(false, false))),
                    "laeuft" => Some(Debugger::demo(snap(true, true))),
                    "kein-bild-moeglich" => { let mut sn = snap(false, false); sn.can_frame = false; Some(Debugger::demo(sn)) }
                    _ => Some(Debugger::demo(snap(true, false))),
                };
                if variant == "zeitlupe" { s.slow = true; }
                let mut mem = "0400".to_string();
                for _ in 0..3 {
                    let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| s.ui(ui, dbg.as_ref(), None, &mut mem));
                    o.textures_delta.clear();
                }
                if matches!(variant, "bild" | "laeuft" | "zeitlupe") { assert!(s.tex.is_some() && s.frame.is_some(), "Bild als Textur ({variant})"); }
                else { assert!(s.tex.is_none(), "kein Bild ({variant})"); }
            }
        }
    }

    #[test]
    fn watchdog_ends_slow_mode_when_tab_is_hidden() {
        let d = Debugger::demo(snap(true, true));
        let mut s = EmuState::default();
        s.slow = true; s.sent = Some((session_id(&d), true, true));
        s.seen = Some(Instant::now()); s.watchdog(Some(&d));
        assert!(s.slow, "Reiter eben noch sichtbar");
        s.seen = Some(Instant::now() - Duration::from_secs(2)); s.watchdog(Some(&d));
        assert!(!s.slow);
        assert_eq!(s.sent.map(|x| x.2), Some(false));
    }

    #[test]
    fn stop_switches_everything_off() {
        let d = Debugger::demo(snap(true, true));
        let mut s = EmuState::default();
        s.slow = true; s.sent = Some((session_id(&d), true, true));
        s.stop(Some(&d));
        assert!(!s.slow && s.sent.is_none());
    }
}
