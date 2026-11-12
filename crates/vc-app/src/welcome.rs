// SPDX-License-Identifier: GPL-3.0-or-later
//! Willkommensseite: Neu/Öffnen, zuletzt geöffnete Projekte, erkannte Emulatoren und Hardware, Studios, Schnellreferenz.
//! Zeigt nur echte Werte: Emulatoren werden gesucht (kein Netzwerk), Hardware gilt als „ungeprüft“, bis der Nutzer testet.
use crate::recent::{self, Recent};
use crate::ui::tokens::t;
use crate::ui::widgets::{badge, card, chip, system_color};
use eframe::egui::{self, RichText};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use vc_ide::platform::{self, Presence};
use vc_ide::targets::{kind_label, EmulatorCfg};

pub enum Act { Feedback, New, Open, OpenExample, OpenPath(PathBuf), Settings, Help(&'static str), NewMedia, Roms, /** Konfigurationsassistent öffnen (Wiederaufruf) */ Wizard, /** Hinweis: `true` = ausblenden, `false` = Assistent öffnen */ Banner(bool) }

/// Hinweis oben auf der Willkommensseite (Konfigurationsassistent)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Banner {
    /// Der Assistent wurde begonnen und ist nicht fertig
    Incomplete,
    /// Bestehende Einstellungen, neue Programmversion: einmal anbieten
    OfferOnce,
    /// Seit dem letzten Lauf wurde ein Programm gefunden
    News,
}

enum Probe { Running, Ok, Err(String) }

pub struct WelcomeState {
    pub recent: Vec<Recent>,
    pub banner: Option<Banner>,
    tools: Vec<(&'static str, Presence)>,
    tools_at: Option<Instant>,
    /// Gelieferte Systeme (nur deren Emulatoren werden geprüft)
    systems: Vec<&'static vc_ide::targets::SystemInfo>,
    probes: HashMap<String, Probe>,
    /// Projektbilder der Liste „Zuletzt geöffnet“ (je Projektordner)
    pics: HashMap<PathBuf, crate::projectpic::Thumb>,
    tx: Sender<(String, Result<String, String>)>,
    rx: Receiver<(String, Result<String, String>)>,
}

impl WelcomeState {
    pub fn new(data: Option<&std::path::Path>) -> WelcomeState {
        let (tx, rx) = channel();
        WelcomeState { recent: recent::load(data), banner: None, tools: vec![], tools_at: None, systems: vc_ide::targets::installed_systems(data), probes: HashMap::new(), pics: HashMap::new(), tx, rx }
    }

    /// Emulator-Sorten in der Reihenfolge der Systeme, je Sorte einmal
    fn refresh_tools(&mut self) {
        if self.tools_at.map_or(false, |t| t.elapsed() < Duration::from_secs(5)) { return; }
        let settings = vc_ide::settings::get();
        let mut seen: Vec<&'static str> = vec![];
        let mut out = vec![];
        for s in &self.systems { for k in s.kinds() {
            if seen.contains(&k.id) || !vc_ide::emulators::enabled(k.id) { continue; }
            seen.push(k.id);
            let path = settings.emulator_of_kind(s.id, k.id).path.unwrap_or_default();
            out.push((k.id, platform::presence(&EmulatorCfg { id: k.id.into(), kind: k.id.into(), path, ..Default::default() })));
        } }
        self.tools = out; self.tools_at = Some(Instant::now());
    }

    pub fn running(&self) -> bool { self.probes.values().any(|p| matches!(p, Probe::Running)) }
}

fn mod_key() -> &'static str { if cfg!(target_os = "macos") { "Cmd" } else { "Strg" } }

fn primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let k = t();
    ui.add(egui::Button::new(RichText::new(text).strong().color(k.on_cy)).fill(k.cy).stroke(egui::Stroke::NONE).min_size(egui::vec2(0.0, 30.0)))
}

pub fn show(ui: &mut egui::Ui, w: &mut WelcomeState, data: Option<&std::path::Path>, has_project: bool) -> Option<Act> {
    while let Ok((id, r)) = w.rx.try_recv() { w.probes.insert(id, match r { Ok(_) => Probe::Ok, Err(e) => Probe::Err(e) }); }
    if w.running() { ui.ctx().request_repaint_after(Duration::from_millis(200)); }
    w.refresh_tools();
    let k = t();
    let mut act: Option<Act> = None;
    let examples = data.and_then(|d| d.parent()).map(|p| p.join("examples")).filter(|p| p.is_dir());

    egui::Panel::right("welcome_right").exact_size(330.0).resizable(false).show_inside(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(14.0);
            card(ui, false, |ui| {
                ui.horizontal(|ui| { ui.label(RichText::new("Toolchain und Hardware").font(egui::FontId::new(14.0, crate::fonts::ui_strong()))); });
                ui.add_space(4.0);
                ui.label(RichText::new("ERKANNTE EMULATOREN").small().color(k.dim));
                for (kind, p) in &w.tools {
                    ui.horizontal(|ui| {
                        let col = if p.found { k.gr } else { k.dim };
                        ui.label(RichText::new("●").color(col).small());
                        ui.label(RichText::new(kind_label(kind)).monospace().small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if p.found { chip(ui, "bereit", k.gr); } else if *kind == "altirra" && !cfg!(windows) { chip(ui, "nur Windows", k.dim); } else { chip(ui, "nicht gefunden", k.dim); }
                        });
                    }).response.on_hover_text(&p.text);
                }
                ui.add_space(6.0);
                ui.label(RichText::new("ECHTE HARDWARE").small().color(k.dim));
                let machines = vc_ide::settings::effective().machines;
                if machines.is_empty() { ui.label(RichText::new("Keine Hardware eingetragen").weak().small()); }
                for m in &machines {
                    let state = match w.probes.get(&m.id) { Some(Probe::Ok) => 1, Some(Probe::Err(_)) => 2, Some(Probe::Running) => 3, None => 0 };
                    let col = match state { 1 => k.gr, 2 => k.rd, 3 => k.am, _ => k.dim };
                    let addr = if m.kind == "ultimate64" { m.host.clone() } else { m.device.clone() };
                    let text = format!("{} · {}", m.display_name(), if addr.is_empty() { "–".into() } else { addr });
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            match state {
                                1 => { chip(ui, "online", k.gr); }
                                2 => { chip(ui, "nicht erreichbar", k.rd); }
                                3 => { chip(ui, "prüfe …", k.am); }
                                _ => {
                                    if ui.small_button("Testen").on_hover_text("Verbindung jetzt prüfen (beim Start findet kein Netzwerkzugriff statt)").clicked() {
                                        w.probes.insert(m.id.clone(), Probe::Running);
                                        let (tx, ctx, mm) = (w.tx.clone(), ui.ctx().clone(), m.clone());
                                        std::thread::spawn(move || { let r = platform::test_hardware(&mm, Duration::from_secs(12)).map(|(t, d)| format!("{t} ({} ms)", d.as_millis().max(1))); let _ = tx.send((mm.id.clone(), r)); ctx.request_repaint(); });
                                    }
                                    chip(ui, "ungeprüft", k.dim);
                                }
                            }
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                ui.label(RichText::new("●").color(col).small());
                                ui.add(egui::Label::new(RichText::new(&text).monospace().small()).truncate());
                            });
                        });
                    });
                    if let Some(Probe::Err(e)) = w.probes.get(&m.id) { ui.label(RichText::new(e).small().color(k.rd)); }
                }
                ui.add_space(8.0);
                if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("Einstellungen öffnen")).clicked() { act = Some(Act::Settings); }
                if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("Konfigurationsassistent")).on_hover_text("Einrichtung prüfen oder wiederholen; geändert wird nur, was du bestätigst").clicked() { act = Some(Act::Wizard); }
                if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("Feedback senden …")).on_hover_text("Fehler oder Wunsch per E-Mail melden; zeigt vorher, was in der Mail steht").clicked() { act = Some(Act::Feedback); }
            });
            ui.add_space(10.0);
            if crate::rom_ui::rom_card(ui) { act = Some(Act::Roms); }
            ui.add_space(10.0);
            card(ui, false, |ui| {
                ui.label(RichText::new("Schnellreferenz").font(egui::FontId::new(14.0, crate::fonts::ui_strong())));
                ui.add_space(4.0);
                for (id, title) in [("start", "Überblick"), ("settings", "Einstellungen"), ("assembler", "Assembler"), ("basic", "BASIC"), ("debugger", "Debugger"), ("disks", "Disk-Studio"), ("shortcuts", "Tastenkürzel")] {
                    if !vc_ide::help::visible(id) { continue; }
                    if ui.link(title).clicked() { act = Some(Act::Help(id)); }
                }
            });
        });
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(14.0);
            if let Some(b) = w.banner {
                egui::Frame::new().fill(crate::ui::tokens::tint(k.cy)).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.4))).corner_radius(8.0).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (text, button) = match b {
                            Banner::Incomplete => ("Einrichtung unvollständig: ein paar Schritte sind noch offen.", "Fertig einrichten"),
                            Banner::OfferOnce => ("Neu: Konfigurationsassistent. Er prüft deine Einrichtung und ändert nur, was du bestätigst.", "Assistent öffnen"),
                            Banner::News => ("Neues erkannt: seit dem letzten Mal wurde ein Programm gefunden.", "Einrichtung prüfen"),
                        };
                        ui.label(RichText::new(text).color(k.fg));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").on_hover_text("Ausblenden").clicked() { act = Some(Act::Banner(true)); }
                            if primary(ui, button).clicked() { act = Some(Act::Banner(false)); }
                        });
                    });
                });
                ui.add_space(10.0);
            }
            // Banner
            egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(8.0).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    badge(ui, "VC", 54.0, k.cy.gamma_multiply(0.5));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| { ui.label(RichText::new("VintageCoder").font(egui::FontId::new(24.0, crate::fonts::ui_strong()))); chip(ui, concat!("v", env!("CARGO_PKG_VERSION")), k.cy); });
                        ui.label(RichText::new("Entwicklungsumgebung für die 6502-Familie: Commodore und Atari, Assembler und BASIC, Emulator und echte Hardware.").color(k.mute));
                    });
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if primary(ui, "Neues Projekt").clicked() { act = Some(Act::New); }
                    if ui.add(egui::Button::new(format!("Projekt öffnen  ({}+O)", mod_key())).min_size(egui::vec2(0.0, 30.0))).clicked() { act = Some(Act::Open); }
                    if ui.add_enabled(examples.is_some(), egui::Button::new("Beispiel öffnen").min_size(egui::vec2(0.0, 30.0))).on_disabled_hover_text("Dieses Release enthält keine Beispielprojekte.").clicked() { act = Some(Act::OpenExample); }
                });
            });
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Zuletzt geöffnete Projekte").font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
                chip(ui, &w.recent.len().to_string(), k.dim);
            });
            ui.add_space(4.0);
            if w.recent.is_empty() { ui.label(RichText::new("Noch kein Projekt geöffnet. „Neues Projekt“ legt eines an, „Beispiel öffnen“ zeigt fertige Programme.").color(k.mute)); }
            let now = recent::now();
            let (mut remove, mut pin) = (None::<usize>, None::<usize>);
            for (i, r) in w.recent.iter().enumerate() {
                card(ui, false, |ui| {
                    ui.horizontal(|ui| {
                        // Projektbild (projektbild.png im Projektordner), falls vorhanden und gewünscht
                        let ctx = ui.ctx().clone();
                        if !r.missing && !r.info.no_preview {
                            if let Some((tex, size)) = w.pics.entry(r.path.clone()).or_default().get(&ctx, &r.path) {
                                let s = crate::projectpic::fit(size, egui::vec2(112.0, 80.0));
                                ui.add(egui::Image::new((tex.id(), s)).corner_radius(4.0)).on_hover_text("Projektbild (projektbild.png)");
                            }
                        }
                        let left = (ui.available_width() - 250.0).max(160.0);
                        ui.vertical(|ui| {
                            ui.set_width(left);
                            ui.horizontal_wrapped(|ui| {
                                let name = ui.add(egui::Label::new(RichText::new(&r.name).font(egui::FontId::new(14.0, crate::fonts::ui_strong())).color(if r.missing { k.dim } else { k.fg })).sense(egui::Sense::click()));
                                if name.clicked() && !r.missing { act = Some(Act::OpenPath(r.path.clone())); }
                                if r.missing { chip(ui, "Ordner fehlt", k.rd); }
                                for (s, v) in &r.info.systems { chip(ui, &if v.is_empty() { recent::short_name(s).to_string() } else { format!("{} {}", recent::short_name(s), v.to_uppercase()) }, system_color(s).gamma_multiply(1.4)); }
                                for l in &r.info.langs { chip(ui, l, k.pu); }
                            });
                            ui.horizontal(|ui| {
                                if let Some(e) = &r.info.emulator { chip(ui, e, k.dim); }
                                ui.add(egui::Label::new(RichText::new(recent::pretty(&r.path)).monospace().small().color(k.mute)).truncate());
                            });
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("×").on_hover_text("Aus der Liste entfernen").clicked() { remove = Some(i); }
                            if ui.small_button(if r.pinned { "★" } else { "☆" }).on_hover_text(if r.pinned { "Anheftung lösen" } else { "Anheften" }).clicked() { pin = Some(i); }
                            if !r.missing && ui.button("Öffnen").clicked() { act = Some(Act::OpenPath(r.path.clone())); }
                            ui.label(RichText::new(recent::ago(r.last, now)).small().color(k.dim));
                        });
                    });
                });
                ui.add_space(6.0);
            }
            if let Some(i) = pin { recent::toggle_pin(&mut w.recent, i); }
            if let Some(i) = remove { recent::remove(&mut w.recent, i); }
            ui.add_space(10.0);
            let studio_buttons: Vec<(&str, &str)> = [("sprite", "Sprite/PMG", "C64 hires und multicolor, Atari Player/Missile"), ("char", "Zeichensatz", "8×8-Zeichen, ROM-Import"), ("screen", "Bildschirm", "Textbildschirme mit Farbe"), ("tiles", "Kacheln", "Kachelkarten"), ("tracker", "Tracker", "SID, POKEY, PLAY"), ("palette", "Palette", "Farbpaletten")]
                .into_iter().filter(|(id, _, _)| crate::studios::enabled(id)).map(|(_, n, s)| (n, s)).collect();
            if !studio_buttons.is_empty() {
            ui.label(RichText::new("Studios").font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
            ui.label(RichText::new(if has_project { "Legt eine neue Datei im geöffneten Projekt an." } else { "Studios arbeiten auf Dateien eines Projekts. Zuerst ein Projekt öffnen." }).small().color(k.mute));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for (name, sub) in studio_buttons {
                    let r = ui.add_enabled(has_project, egui::Button::new(RichText::new(name).strong()).min_size(egui::vec2(120.0, 28.0))).on_hover_text(sub).on_disabled_hover_text("Zuerst ein Projekt öffnen");
                    if r.clicked() { act = Some(Act::NewMedia); }
                }
            });
            }
        });
    });
    act
}
