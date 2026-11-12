// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt-Wizard: dreispaltiger Dialog (Zielsystem, Vorlage, Details). Die Vorlagen kommen aus `data/templates`; angelegt wird mit
//! `vc_media::wizard::create_with`, danach ergänzt `finish` die Projektdatei (Videonorm, VC-20-Speicher, BASIC-Dialekt, Diskette)
//! und legt auf Wunsch README und Git-Repository an. Weicht der gewählte Emulator oder das Startziel von den Einstellungen ab,
//! steht die Abweichung als Überschreibung in der Projektdatei (sonst nichts). Es gibt nur unseren Assembler, daher keine Auswahl eines Cross-Assemblers.
use crate::recent::{dialect_label, target_dialect};
use crate::ui::tokens::t;
use crate::ui::widgets::{badge, card, chip, segmented, system_color};
use eframe::egui::{self, RichText};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::targets;

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Lang { Asm, Basic }

/// Eine wählbare Vorlage (Vorlage oder deren Variante)
#[derive(Clone)]
struct Card { template: String, variant: Option<String>, name: String, description: String, lang: Lang, systems: Vec<String>, files: Vec<String> }

pub struct Wizard {
    data: PathBuf,
    cards: Vec<Card>,
    pub name: String,
    pub parent: PathBuf,
    system: String,
    extra: Vec<String>,
    lang: Lang,
    card: Option<usize>,
    pal: String,
    memory: String,
    dialect: String,
    disk: String,
    /// Gewählter Emulator des Systems (Kennung aus den Einstellungen) und Startziel (leer = Emulator, sonst Maschine); beim Anlegen nur bei Abweichung ins Projekt geschrieben
    emulator: String,
    start_on: String,
    git: bool,
    readme: bool,
    /// Projektstruktur-Vorlagen (`data/templates/structures.json`) und die gewählte (Index; 0 = „Einfach“, ändert nichts)
    structures: Vec<vc_ide::structure::Structure>,
    structure: usize,
}

pub enum Out { None, Cancel, Created(PathBuf, Vec<String>) }

fn strs(v: &Value) -> Vec<String> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }

fn read_json(p: &Path) -> Option<Value> { serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok() }

/// Diskettenformate, die zu einem System passen
fn disk_formats(system: &str) -> &'static [&'static str] {
    match system { "c128" => &["d64", "d71", "d81"], "atari8" => &["atr"], "mega65" => &["d81"], _ => &["d64"] }
}

impl Wizard {
    pub fn new(data: &Path, parent: PathBuf) -> Option<Wizard> {
        let idx = read_json(&data.join("templates/index.json"))?;
        let mut cards = vec![];
        for t in idx["templates"].as_array()? {
            let mut push = |name: String, desc: String, id: Option<String>, node: &Value| {
                let lang = if node["sources"].as_array().and_then(|s| s.first()).map_or("asm", |s| s["kind"].as_str().unwrap_or("asm")) == "basic" { Lang::Basic } else { Lang::Asm };
                let files = node["files"].as_array().map(|f| f.iter().filter_map(|x| x["to"].as_str().map(String::from)).collect()).unwrap_or_default();
                cards.push(Card { template: t["id"].as_str().unwrap_or("").into(), variant: id, name, description: desc, lang, systems: strs(&node["systems"]), files });
            };
            push(t["name"].as_str().unwrap_or("").into(), t["description"].as_str().unwrap_or("").into(), None, t);
            for v in t["variants"].as_array().into_iter().flatten() { push(v["name"].as_str().unwrap_or("").into(), v["description"].as_str().unwrap_or("").into(), v["id"].as_str().map(String::from), v); }
        }
        let mut w = Wizard { data: data.to_path_buf(), cards, name: "meinprojekt".into(), parent, system: vc_target::default_target_or_c64(Some(data)), extra: vec![], lang: Lang::Asm, card: None, pal: "pal".into(), memory: String::new(), dialect: String::new(), disk: String::new(), emulator: String::new(), start_on: String::new(), git: false, readme: true, structures: vc_ide::structure::list_all(data), structure: 0 };
        w.sync();
        Some(w)
    }

    /// Selbsttest (Screenshots): Zielsystem und Sprache vorwählen
    pub fn demo(&mut self, system: &str, basic: bool) { self.system = system.into(); self.lang = if basic { Lang::Basic } else { Lang::Asm }; self.card = None; self.sync(); }

    fn filtered(&self) -> Vec<usize> { self.cards.iter().enumerate().filter(|(_, c)| c.lang == self.lang && c.systems.contains(&self.system)).map(|(i, _)| i).collect() }

    /// Auswahl nach einer Änderung gültig halten
    fn sync(&mut self) {
        let f = self.filtered();
        if !self.card.map_or(false, |c| f.contains(&c)) { self.card = f.first().copied(); }
        self.extra.retain(|e| *e != self.system && self.card.map_or(false, |c| self.cards[c].systems.contains(e)));
        if self.dialect.is_empty() || !self.dialects().iter().any(|d| *d == self.dialect) { self.dialect = self.dialects().first().cloned().unwrap_or_default(); }
        if !disk_formats(&self.system).contains(&self.disk.as_str()) { self.disk.clear(); }
        self.sync_target_choice(&vc_ide::settings::get());
        if self.system == "vic20" && self.memory.is_empty() { self.memory = self.vic20_memory().first().map(|m| m.0.clone()).unwrap_or_default(); }
    }

    /// Emulator und Startziel gültig halten: nach einem Systemwechsel (oder ungültiger Wahl) gelten wieder die Vorgaben der Einstellungen
    fn sync_target_choice(&mut self, st: &vc_ide::settings::Settings) {
        let cfg = st.target(&self.system);
        if !cfg.map_or(false, |t| t.emulators.iter().any(|e| e.id == self.emulator)) { self.emulator = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.clone()).unwrap_or_default(); }
        if !self.start_on.is_empty() && st.machines_for(&self.system).all(|m| m.id != self.start_on) { self.start_on.clear(); }
    }

    fn dialects(&self) -> Vec<String> {
        let mut v: Vec<String> = target_dialect(&self.data, &self.system).into_iter().collect();
        if self.system == "atari8" { v.push("turbo_basic_xl".into()); }
        v
    }

    fn vic20_memory(&self) -> Vec<(String, String)> {
        let Some(t) = read_json(&self.data.join("targets/vic20.target.json")) else { return vec![] };
        let default = t["memory"]["default"].as_str().unwrap_or("").to_string();
        let mut v: Vec<(String, String)> = t["memory"]["configs"].as_object().map(|o| o.iter().map(|(k, c)| (k.clone(), c["label"].as_str().unwrap_or(k).to_string())).collect()).unwrap_or_default();
        if let Some(i) = v.iter().position(|x| x.0 == default) { let d = v.remove(i); v.insert(0, d); }
        v
    }

    fn name_ok(&self) -> bool { !self.name.is_empty() && self.name.len() <= 48 && self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') }

    fn systems(&self) -> Vec<String> { let mut v = vec![self.system.clone()]; v.extend(self.extra.iter().cloned()); v }

    /// Dateistruktur-Vorschau
    fn preview(&self) -> Vec<String> {
        let mut files = vec![format!("{}.vcprj", self.name)];
        if let Some(c) = self.card { for s in self.systems() { for f in &self.cards[c].files { files.push(f.replace("{system}", &s)); } } }
        if let Some(st) = self.structures.get(self.structure) { for d in st.dirs_for(&self.systems()) { if !files.iter().any(|f| f.starts_with(&format!("{d}/"))) { files.push(format!("{d}/")); } } }
        if self.readme { files.push("README.md".into()); }
        if self.git { files.push(".gitignore".into()); }
        files.push("build/".into());
        files
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Out {
        let k = t();
        let screen = ctx.content_rect().size();
        let (w, h) = ((screen.x - 80.0).clamp(760.0, 1180.0), (screen.y - 80.0).clamp(480.0, 720.0));
        let mut out = Out::None;
        egui::Area::new(egui::Id::new("wizard_dim")).order(egui::Order::Foreground).fixed_pos(egui::Pos2::ZERO).show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(screen, egui::Sense::click_and_drag());
            ui.painter().rect_filled(r, 0.0, egui::Color32::from_black_alpha(150));
        });
        egui::Window::new("Neues Projekt").title_bar(false).collapsible(false).resizable(false).order(egui::Order::Tooltip).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).fixed_size([w, h])
            .frame(egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.3))).corner_radius(8.0).inner_margin(egui::Margin::same(0)))
            .show(ctx, |ui| {
                ui.set_min_size(egui::vec2(w, h));
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
                // Kopf
                egui::Frame::new().inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, "+", 34.0, k.cy.gamma_multiply(0.5));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| { ui.label(RichText::new("Neues Projekt erstellen").font(egui::FontId::new(18.0, crate::fonts::ui_strong()))); chip(ui, "WIZARD", k.cy); });
                            ui.label(RichText::new("Zielsystem, Vorlage und Details wählen").small().color(k.mute));
                        });
                    });
                });
                ui.separator();
                let body_h = h - 150.0;
                let (c1, c3) = (260.0, 340.0);
                let c2 = w - c1 - c3 - 2.0 * 16.0 - 24.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    // Spalte 1: Zielsystem
                    column(ui, c1, body_h, |ui| { self.col_system(ui); });
                    ui.separator();
                    column(ui, c2, body_h, |ui| { self.col_templates(ui); });
                    ui.separator();
                    column(ui, c3, body_h, |ui| { self.col_details(ui); });
                });
                ui.separator();
                // Fuß
                egui::Frame::new().inner_margin(egui::Margin::symmetric(16, 8)).show(ui, |ui| {
                    // Hinweisband: fehlende ROMs für das gewählte Zielsystem (nur wenn etwas fehlt); der Assistent öffnet über das Hauptfenster
                    if crate::rom_ui::rom_band(ui, &[self.system.as_str()]) { ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("rom_setup_request"), true)); }
                    ui.horizontal(|ui| {
                        let var = if self.system == "mega65" { String::new() } else { format!(" {}", self.pal.to_uppercase()) };
                        ui.label(RichText::new("●").color(k.gr).small());
                        ui.label(RichText::new(format!("Ziel: {}{}", crate::recent::short_name(&self.system), var)).monospace().small());
                        ui.label(RichText::new(format!("· {}", if self.lang == Lang::Asm { "Assembler (vcasm)".to_string() } else { dialect_label(&self.dialect).to_string() })).monospace().small().color(k.mute));
                        if let Some(c) = self.card { ui.label(RichText::new(format!("· {}", self.cards[c].name)).monospace().small().color(k.mute)); }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let ok = self.name_ok() && self.card.is_some();
                            let btn = egui::Button::new(RichText::new("Projekt erstellen und öffnen").strong().color(k.on_cy)).fill(if ok { k.cy } else { k.cy.gamma_multiply(0.3) }).stroke(egui::Stroke::NONE).min_size(egui::vec2(0.0, 30.0));
                            if ui.add_enabled(ok, btn).clicked() {
                                match self.create() { Ok((dir, notes)) => out = Out::Created(dir, notes), Err(e) => { ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("wizard_err"), e)); } }
                            }
                            if ui.add(egui::Button::new("Abbrechen").min_size(egui::vec2(0.0, 30.0))).clicked() { out = Out::Cancel; }
                            if let Some(e) = ui.ctx().data(|d| d.get_temp::<String>(egui::Id::new("wizard_err"))) { ui.label(RichText::new(e).color(k.rd).small()); }
                        });
                    });
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { out = Out::Cancel; }
        out
    }

    fn col_system(&mut self, ui: &mut egui::Ui) {
        let k = t();
        ui.label(RichText::new("ZIELSYSTEM").small().strong().color(k.mute));
        let before = self.system.clone();
        for s in targets::installed_systems(Some(&self.data)) {
            let on = self.system == s.id;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 50.0), egui::Sense::click());
            let p = ui.painter();
            if on { p.rect_filled(rect, 6.0, k.cy.gamma_multiply(0.16)); p.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, k.cy.gamma_multiply(0.5)), egui::StrokeKind::Inside); }
            else if resp.hovered() { p.rect_filled(rect, 6.0, k.bg2); }
            let b = egui::Rect::from_min_size(rect.min + egui::vec2(8.0, 12.0), egui::vec2(40.0, 26.0));
            p.rect_filled(b, 5.0, system_color(s.id));
            p.text(b.center(), egui::Align2::CENTER_CENTER, s.badge, egui::FontId::monospace(12.0), egui::Color32::WHITE);
            p.text(rect.min + egui::vec2(58.0, 14.0), egui::Align2::LEFT_CENTER, s.name, egui::FontId::proportional(13.5), k.fg);
            p.text(rect.min + egui::vec2(58.0, 34.0), egui::Align2::LEFT_CENTER, s.cpu, egui::FontId::proportional(10.5), k.mute);
            if resp.clicked() { self.system = s.id.into(); }
        }
        if self.system != before { self.sync(); }
        ui.add_space(8.0);
        let valid = vc_target::load_target(&self.data, &self.system).is_ok();
        card(ui, false, |ui| {
            ui.horizontal(|ui| { ui.label(RichText::new("Hardware-Profil").color(k.mute)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if valid { chip(ui, "gültig", k.gr); } else { chip(ui, "fehlt", k.rd); } }); });
            if let Ok(t) = vc_target::load_target(&self.data, &self.system) { ui.label(RichText::new(format!("CPU {} · {}", t.cpu, t.chips.join(", "))).monospace().small().color(k.mute)); }
        });
    }

    fn col_templates(&mut self, ui: &mut egui::Ui) {
        let k = t();
        ui.horizontal(|ui| {
            ui.label(RichText::new("VORLAGE").small().strong().color(k.mute));
            let mut l = if self.lang == Lang::Asm { "asm".to_string() } else { "basic".to_string() };
            if segmented(ui, &mut l, &[("asm", "Assembler"), ("basic", "BASIC")]) { self.lang = if l == "asm" { Lang::Asm } else { Lang::Basic }; self.sync(); }
        });
        let f = self.filtered();
        if f.is_empty() { ui.add_space(10.0); ui.label(RichText::new("Für diese Kombination aus Zielsystem und Sprache gibt es keine Vorlage.").color(k.mute)); }
        for i in f {
            let c = self.cards[i].clone();
            let on = self.card == Some(i);
            let rect = card(ui, on, |ui| {
                ui.horizontal(|ui| { ui.label(RichText::new(&c.name).strong()); chip(ui, if c.lang == Lang::Asm { "vcasm" } else { "BASIC" }, k.pu); });
                ui.label(RichText::new(&c.description).small().color(k.mute));
                ui.horizontal_wrapped(|ui| { for file in &c.files { ui.label(RichText::new(file.replace("{system}", &self.system)).monospace().small().color(k.dim)); } });
                ui.min_rect()
            }).expand(12.0);
            if ui.interact(rect, egui::Id::new(("tpl", i)), egui::Sense::click()).clicked() { self.card = Some(i); }
        }
        if self.lang == Lang::Basic {
            ui.add_space(6.0);
            ui.label(RichText::new("BASIC wird im Dialekt des Zielsystems geschrieben, ohne Zeilennummern und mit Sprungmarken (@name). Beim Bauen entsteht daraus das native Programm.").small().color(k.mute));
        }
    }

    fn col_details(&mut self, ui: &mut egui::Ui) {
        let k = t();
        ui.label(RichText::new("PROJEKT-DETAILS").small().strong().color(k.mute));
        ui.label(RichText::new("Projektname").small().color(k.mute));
        ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
        if !self.name_ok() { ui.label(RichText::new("Nur Buchstaben, Ziffern, _ und -").small().color(k.rd)); }
        ui.label(RichText::new("Speicherort").small().color(k.mute));
        ui.horizontal(|ui| {
            let mut s = crate::recent::pretty(&self.parent);
            ui.add(egui::TextEdit::singleline(&mut s).desired_width(ui.available_width() - 80.0).font(egui::TextStyle::Monospace).interactive(false));
            if ui.button("Wählen …").clicked() { if let Some(f) = rfd::FileDialog::new().set_directory(&self.parent).pick_folder() { self.parent = f; } }
        });
        if self.lang == Lang::Basic {
            ui.label(RichText::new("BASIC-Dialekt").small().color(k.mute));
            let opts = self.dialects();
            if opts.len() > 1 {
                let list: Vec<(&str, &str)> = opts.iter().map(|d| (d.as_str(), dialect_label(d))).collect();
                segmented(ui, &mut self.dialect, &list);
            } else if let Some(d) = opts.first() { chip(ui, dialect_label(d), k.pu); }
        }
        if self.system != "mega65" {
            ui.label(RichText::new("Videonorm").small().color(k.mute));
            segmented(ui, &mut self.pal, &[("pal", "PAL"), ("ntsc", "NTSC")]);
        }
        if self.system == "vic20" {
            ui.label(RichText::new("Speicherausbau").small().color(k.mute));
            let list = self.vic20_memory();
            egui::ComboBox::from_id_salt("wiz_mem").selected_text(list.iter().find(|m| m.0 == self.memory).map_or("–", |m| m.1.as_str())).width(ui.available_width() - 8.0).show_ui(ui, |ui| { for (id, label) in &list { ui.selectable_value(&mut self.memory, id.clone(), label.as_str()); } });
        }
        // Emulator und Startziel: Vorgabe sind die Einstellungen; eine Abweichung wird im Projekt gespeichert (nur dort wirksam)
        let st = vc_ide::settings::get();
        let cfg = st.target(&self.system);
        let global_emu = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.clone()).unwrap_or_default();
        let global_start = cfg.map(|t| t.start_on.clone()).filter(|id| st.machine(id).is_some()).unwrap_or_default();
        ui.label(RichText::new("Standard-Emulator").small().color(k.mute));
        match cfg.filter(|t| !t.emulators.is_empty()) {
            Some(t) => {
                let name_of = |id: &str| t.emulators.iter().find(|e| e.id == id).map_or("–".to_string(), |e| e.display_name().to_string());
                egui::ComboBox::from_id_salt("wiz_emu").selected_text(name_of(&self.emulator)).width(ui.available_width() - 8.0).show_ui(ui, |ui| {
                    for e in &t.emulators { ui.selectable_value(&mut self.emulator, e.id.clone(), if e.id == global_emu { format!("{} (Einstellungen)", e.display_name()) } else { e.display_name().to_string() }); }
                });
                if self.emulator != global_emu { ui.label(RichText::new("weicht von den Einstellungen ab: wird in der Projektdatei gespeichert").small().color(k.am)); }
            }
            None => { ui.horizontal(|ui| { chip(ui, targets::kind_label(&st.emulator_for(&self.system).kind), k.cy); ui.label(RichText::new("aus den Einstellungen").small().color(k.dim)); }); }
        }
        // Homepage des gewählten Emulators; fehlt das Programm, steht der Weg zur Installation daneben
        {
            let kind = cfg.and_then(|t| t.emulators.iter().find(|e| e.id == self.emulator)).map(|e| e.kind.clone()).unwrap_or_else(|| st.emulator_for(&self.system).kind.clone());
            if let Some(en) = vc_ide::emulators::for_kind(&kind) {
                ui.horizontal_wrapped(|ui| {
                    ui.hyperlink_to(format!("Homepage von {}", en.name), &en.homepage).on_hover_text(&en.homepage);
                    if vc_ide::launch::detect(&kind).is_none() && cfg.and_then(|t| t.emulators.iter().find(|e| e.id == self.emulator)).map_or(true, |e| e.path.trim().is_empty()) {
                        ui.label(RichText::new(if en.package().is_some() { "nicht gefunden – unter Einstellungen portabel installieren" } else { "nicht gefunden – siehe Homepage" }).small().color(k.am));
                    }
                });
            }
        }
        ui.label(RichText::new("Startziel (F5)").small().color(k.mute));
        {
            let mut opts: Vec<(String, String)> = vec![(String::new(), "Emulator".into())];
            for m in st.machines_for(&self.system) { opts.push((m.id.clone(), format!("{} (Hardware)", m.display_name()))); }
            let list: Vec<(&str, &str)> = opts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
            segmented(ui, &mut self.start_on, &list);
            if self.start_on != global_start { ui.label(RichText::new("weicht von den Einstellungen ab: wird in der Projektdatei gespeichert").small().color(k.am)); }
        }
        ui.label(RichText::new("Diskette").small().color(k.mute));
        ui.horizontal(|ui| {
            let mut opts: Vec<(&str, String)> = vec![("", "keine".into())];
            for f in disk_formats(&self.system) { opts.push((f, f.to_uppercase())); }
            let list: Vec<(&str, &str)> = opts.iter().map(|(a, b)| (*a, b.as_str())).collect();
            segmented(ui, &mut self.disk, &list);
        });
        if !self.structures.is_empty() {
            ui.label(RichText::new("Projektstruktur").small().color(k.mute));
            let cur = self.structures.get(self.structure).map(|s| s.name.clone()).unwrap_or_default();
            egui::ComboBox::from_id_salt("wiz_structure").width(220.0).selected_text(cur).show_ui(ui, |ui| {
                for (i, s) in self.structures.iter().enumerate() { ui.selectable_value(&mut self.structure, i, &s.name); }
            });
            if let Some(s) = self.structures.get(self.structure) {
                ui.label(RichText::new(&s.description).small().color(k.dim));
                if !s.layout.is_empty() { ui.label(RichText::new("Abgelegte Dateien: Zielordner je Kategorie stehen in der Projektdatei (layout).").small().color(k.dim)); }
            }
        }
        ui.checkbox(&mut self.readme, "README.md mit Bauhinweisen anlegen");
        ui.checkbox(&mut self.git, "Git-Repository anlegen (.gitignore für build/ und .vcbackup/)");
        // weitere Systeme
        if let Some(c) = self.card {
            let others: Vec<&targets::SystemInfo> = targets::installed_systems(Some(&self.data)).into_iter().filter(|s| s.id != self.system && self.cards[c].systems.iter().any(|x| x == s.id)).collect();
            if !others.is_empty() {
                egui::CollapsingHeader::new(RichText::new("Weitere Zielsysteme").small().color(k.mute)).show(ui, |ui| {
                    for s in others { let mut on = self.extra.iter().any(|x| x == s.id); if ui.checkbox(&mut on, s.name).changed() { if on { self.extra.push(s.id.into()); } else { self.extra.retain(|x| x != s.id); } } }
                });
            }
        }
        ui.add_space(4.0);
        card(ui, false, |ui| {
            let files = self.preview();
            ui.horizontal(|ui| { ui.label(RichText::new("Dateistruktur").small().color(k.mute)); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { chip(ui, &format!("{} Dateien", files.iter().filter(|f| !f.ends_with('/')).count()), k.cy); }); });
            ui.label(RichText::new(format!("{}/", self.name)).monospace().small().color(k.cy));
            // als Baum: Ordner vor Dateien, eingerückt
            let notes: std::collections::BTreeMap<String, String> = [("build".to_string(), "entsteht beim Bauen".to_string())].into_iter().collect();
            for l in vc_ide::structure::preview_tree(&files, &notes, &|_| true) {
                ui.horizontal(|ui| {
                    ui.add_space(l.depth as f32 * 14.0);
                    ui.label(RichText::new(if l.dir { format!("{}/", l.name) } else { l.name.clone() }).monospace().small().color(if l.dir { k.fg } else { k.mute }));
                    if !l.note.is_empty() { ui.label(RichText::new(&l.note).small().color(k.dim)); }
                });
            }
        });
    }

    /// Projekt anlegen und die Projektdatei ergänzen; liefert den Projektordner und Hinweise (z. B. Git fehlt)
    fn create(&self) -> Result<(PathBuf, Vec<String>), String> { self.create_with(&vc_ide::settings::get()) }

    /// Wie [`Wizard::create`] mit den angegebenen globalen Einstellungen (für Tests)
    fn create_with(&self, st: &vc_ide::settings::Settings) -> Result<(PathBuf, Vec<String>), String> {
        let c = &self.cards[self.card.ok_or("keine Vorlage gewählt")?];
        let dir = self.parent.join(&self.name);
        let systems = self.systems();
        vc_media::wizard::create_with(&self.data, &dir, &self.name, &systems, &c.template, c.variant.as_deref())?;
        let mut notes = vec![];
        let file = dir.join(format!("{}.vcprj", self.name));
        let mut pf = vc_ide::projfile::ProjectFile::load(&file)?;
        for t in &mut pf.targets {
            let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
            if sys != "mega65" { t.variant = self.pal.clone(); }
            if sys == "vic20" && !self.memory.is_empty() { t.extra.insert("memory".into(), json!(self.memory)); }
        }
        if self.lang == Lang::Basic {
            for s in &mut pf.sources {
                if s.kind != "basic" { continue; }
                let sys = s.targets.first().cloned().unwrap_or_else(|| self.system.clone());
                if target_dialect(&self.data, &sys).as_deref() != Some(self.dialect.as_str()) && sys == self.system { s.dialect = self.dialect.clone(); }
            }
        }
        pf.set_overrides_if_differs(&self.system, st, &self.emulator, &self.start_on);
        if !self.disk.is_empty() { pf.disk = Some(vc_ide::projfile::DiskSpec { format: self.disk.clone(), ..Default::default() }); }
        if let Some(st) = self.structures.get(self.structure) { st.apply(&dir, &mut pf, &systems, self.git)?; }
        pf.save(&file)?;
        if self.readme {
            let sys = targets::system(&self.system).map_or(self.system.as_str(), |s| s.name);
            let text = format!("# {}\n\nProjekt für {} ({}), angelegt mit VintageCoder.\n\n- Bauen: F7 (oder `vcc build .`)\n- Starten: F5\n- Debuggen: F6\n", self.name, sys, c.name);
            std::fs::write(dir.join("README.md"), text).map_err(|e| e.to_string())?;
        }
        if self.git {
            std::fs::write(dir.join(".gitignore"), "build/\n.vcbackup/\n").map_err(|e| e.to_string())?;
            match std::process::Command::new("git").arg("init").current_dir(&dir).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status() {
                Ok(s) if s.success() => {}
                _ => notes.push("git init ist fehlgeschlagen (ist Git installiert?). .gitignore wurde trotzdem angelegt.".to_string()),
            }
        }
        Ok((dir, notes))
    }
}

fn column(ui: &mut egui::Ui, w: f32, h: f32, body: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(egui::vec2(w + 24.0, h), egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::ScrollArea::vertical().id_salt(("wizcol", w as i32)).max_height(h).auto_shrink([false, false]).show(ui, |ui| {
            egui::Frame::new().inner_margin(egui::Margin::symmetric(12, 10)).show(ui, |ui| { ui.set_width(w - 12.0); body(ui); });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> PathBuf { vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data/") }
    fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_wiz_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test]
    fn templates_are_filtered_by_target_and_language() {
        let mut w = Wizard::new(&data(), tmp("f")).unwrap();
        w.demo("c64", false);
        let names: Vec<String> = w.filtered().iter().map(|&i| w.cards[i].name.clone()).collect();
        assert!(names.iter().any(|n| n.contains("Steckmodul")), "{names:?}");
        w.demo("mega65", false);
        assert!(!w.filtered().iter().any(|&i| w.cards[i].variant.as_deref() == Some("cartridge")), "kein Steckmodul für MEGA65");
        w.demo("atari8", true);
        assert!(!w.filtered().is_empty() && w.filtered().iter().all(|&i| w.cards[i].lang == Lang::Basic));
        assert_eq!(w.dialects(), vec!["atari_basic".to_string(), "turbo_basic_xl".to_string()]);
        w.demo("c128", true);
        assert_eq!(w.dialects(), vec!["basic7".to_string()]);
    }

    #[test]
    fn creates_project_with_norm_memory_dialect_disk_readme_and_gitignore() {
        let parent = tmp("c");
        let mut w = Wizard::new(&data(), parent.clone()).unwrap();
        w.name = "vc20test".into();
        w.demo("vic20", false);
        w.pal = "ntsc".into(); w.memory = "16k".into(); w.disk = "d64".into(); w.readme = true; w.git = false;
        let (dir, _) = w.create().unwrap();
        let pf = vc_ide::projfile::ProjectFile::load(&dir.join("vc20test.vcprj")).unwrap();
        assert_eq!(pf.targets[0].variant, "ntsc");
        assert_eq!(pf.targets[0].extra["memory"], "16k");
        assert_eq!(pf.disk.as_ref().unwrap().format, "d64");
        assert!(dir.join("README.md").is_file() && !dir.join(".gitignore").exists());
        // BASIC mit Turbo BASIC XL
        let mut w = Wizard::new(&data(), parent).unwrap();
        w.name = "tbx".into(); w.demo("atari8", true); w.dialect = "turbo_basic_xl".into(); w.git = true; w.readme = false;
        let (dir, _) = w.create().unwrap();
        let pf = vc_ide::projfile::ProjectFile::load(&dir.join("tbx.vcprj")).unwrap();
        assert!(pf.sources.iter().any(|s| s.kind == "basic" && s.dialect == "turbo_basic_xl"), "{:?}", pf.sources);
        assert_eq!(std::fs::read_to_string(dir.join(".gitignore")).unwrap(), "build/\n.vcbackup/\n");
        // doppelt anlegen scheitert
        assert!(w.create().is_err());
    }

    fn settings_with_machine() -> vc_ide::settings::Settings {
        let mut s = vc_ide::settings::Settings::default();
        s.add_target("c64");
        s.target_mut("c64").unwrap().add_emulator("x64");
        s.add_machine("c64", "ultimate64");
        s
    }

    #[test]
    fn project_file_gets_overrides_only_when_the_choice_differs() {
        let st = settings_with_machine();
        let parent = tmp("o");
        // Vorgaben unverändert: keine Überschreibung
        let mut w = Wizard::new(&data(), parent.clone()).unwrap();
        w.name = "ohne".into(); w.demo("c64", false);
        w.sync_target_choice(&st);
        assert_eq!(w.emulator, st.target("c64").unwrap().standard_emulator().unwrap().id);
        let (dir, _) = w.create_with(&st).unwrap();
        let text = std::fs::read_to_string(dir.join("ohne.vcprj")).unwrap();
        assert!(!text.contains("default_emulator") && !text.contains("start_on"), "{text}");
        // anderer Emulator und Hardware als Startziel
        let mut w = Wizard::new(&data(), parent).unwrap();
        w.name = "mit".into(); w.demo("c64", false);
        w.sync_target_choice(&st);
        let other = st.target("c64").unwrap().emulators.iter().find(|e| !e.standard).unwrap().id.clone();
        w.emulator = other.clone(); w.start_on = "m1".into();
        let (dir, _) = w.create_with(&st).unwrap();
        let pf = vc_ide::projfile::ProjectFile::load(&dir.join("mit.vcprj")).unwrap();
        assert_eq!(pf.format, 1);
        assert_eq!(pf.overrides(), vec![vc_ide::targets::ProjectOverride { system: "c64".into(), default_emulator: other, start_on: "m1".into() }]);
        // wirksam: Start und Debugger sehen die Überschreibung
        let eff = st.with_overrides(&pf.overrides());
        assert_eq!(eff.emulator_for("c64").kind, "x64");
        assert_eq!(eff.start_machine("", "c64").map(|m| m.id.as_str()), Some("m1"));
    }

    #[test]
    fn choice_resets_when_the_system_changes() {
        let st = settings_with_machine();
        let mut w = Wizard::new(&data(), tmp("r")).unwrap();
        w.demo("c64", false);
        w.emulator = "weg".into(); w.start_on = "m9".into();
        w.sync_target_choice(&st);
        assert_eq!(w.emulator, st.target("c64").unwrap().standard_emulator().unwrap().id);
        assert!(w.start_on.is_empty());
        w.system = "atari8".into();   // dort nichts eingerichtet
        w.sync_target_choice(&st);
        assert!(w.emulator.is_empty());
    }

    #[test]
    fn structure_creates_folders_and_layout() {
        let parent = tmp("struct");
        let mut w = Wizard::new(&data(), parent.clone()).unwrap();
        w.name = "gross".into(); w.git = true;
        assert_eq!(w.structure, 0, "Vorgabe: Einfach");
        let (dir0, _) = { let mut w0 = Wizard::new(&data(), parent.clone()).unwrap(); w0.name = "klein".into(); w0.create_with(&vc_ide::settings::Settings::default()).unwrap() };
        assert!(!dir0.join("lib").exists() && vc_ide::projfile::ProjectFile::load(&dir0.join("klein.vcprj")).unwrap().layout.is_empty());
        w.structure = w.structures.iter().position(|s| s.id == "gross").unwrap();
        assert!(w.preview().iter().any(|f| f == "res/fonts/"), "Vorschau nennt die Ordner");
        let (dir, _) = w.create_with(&vc_ide::settings::Settings::default()).unwrap();
        assert!(dir.join("res/fonts/.gitkeep").is_file() && dir.join("tests").is_dir());
        let pf = vc_ide::projfile::ProjectFile::load(&dir.join("gross.vcprj")).unwrap();
        assert_eq!(pf.layout.get("charset").map(String::as_str), Some("res/fonts"));
        // Mehrsystem: Ordner je gewähltem System
        let mut m = Wizard::new(&data(), parent.clone()).unwrap();
        m.name = "multi".into(); m.structure = m.structures.iter().position(|s| s.id == "mehrsystem").unwrap();
        m.demo("c64", false);
        if let Some(c) = m.card { if m.cards[c].systems.iter().any(|x| x == "plus4") { m.extra.push("plus4".into()); } }
        let (d2, _) = m.create_with(&vc_ide::settings::Settings::default()).unwrap();
        assert!(d2.join("res/c64").is_dir() && (m.extra.is_empty() || d2.join("res/plus4").is_dir()));
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn invalid_names_are_rejected() {
        let mut w = Wizard::new(&data(), tmp("n")).unwrap();
        w.name = "mein projekt".into(); assert!(!w.name_ok());
        w.name = "".into(); assert!(!w.name_ok());
        w.name = "ok_1-x".into(); assert!(w.name_ok());
    }
}
