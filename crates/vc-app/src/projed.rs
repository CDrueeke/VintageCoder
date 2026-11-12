// SPDX-License-Identifier: GPL-3.0-or-later
//! Grafischer Projekteditor: bearbeitet die `.vcprj` (Name, Targets, Quellen mit Reihenfolge, Startprogramm, Diskette).
use eframe::egui::{self, Color32, RichText};
use std::path::Path;
use vc_ide::projfile::{DiskSpec, ProjectFile, Source, TargetDef};

pub struct ProjEdit { pub draft: ProjectFile, pub saved: ProjectFile, pub new_target: String, /** Rückmeldung zu „Ordner anlegen“ im Abschnitt Projektstruktur */ pub note: String, /** Name für „Als Vorlage speichern“ */ pub tpl_name: String, /** Baumvorschau der Projektstruktur (Schlüssel = Stand von layout, rules, folders) */ preview: (String, Vec<vc_ide::structure::PreviewLine>), /** Projektbild (Vorschau in den Einstellungen) */ pic: crate::projectpic::Thumb, /** Rückmeldung zum Projektbild */ pic_note: String }

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Act { None, Save, SaveBuild, Revert }

/// Umgebung des Editors: bekannte Targets (ID, CPU) aus der Datenbank und Dateien des Projekts
pub struct Env<'a> { pub targets: &'a [(String, String)], pub files: &'a [String], pub dialects: &'a [String], pub structures: &'a [vc_ide::structure::Structure] }

impl ProjEdit {
    pub fn new(pf: ProjectFile) -> ProjEdit { ProjEdit { saved: pf.clone(), draft: pf, new_target: String::new(), note: String::new(), tpl_name: String::new(), preview: Default::default(), pic: Default::default(), pic_note: String::new() } }
    pub fn dirty(&self) -> bool { self.draft != self.saved }
}

fn emulator_for(id: &str) -> &'static str { match id { "atari8" => "atari800", "mega65" => "xemu", _ => "vice" } }
fn kind_matches(kind: &str, f: &str) -> bool {
    let l = f.to_ascii_lowercase();
    match kind { "basic" => l.ends_with(".sbas") || l.ends_with(".bas") || l.ends_with(".lst") || l.ends_with(".tur"), _ => l.ends_with(".asm") || l.ends_with(".s") }
}
fn is_entry_candidate(f: &str) -> bool { !f.starts_with("build/") && !f.to_ascii_lowercase().ends_with(".inc") && (kind_matches("asm", f) || kind_matches("basic", f)) }

pub fn show(ui: &mut egui::Ui, ed: &mut ProjEdit, env: &Env, root: &Path) -> Act {
    egui::Frame::new().inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| show_inner(ui, ed, env, root)).inner
}

fn show_inner(ui: &mut egui::Ui, ed: &mut ProjEdit, env: &Env, root: &Path) -> Act {
    let mut act = Act::None;
    let problems = ed.draft.validate(root);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Projekt-Einstellungen").font(egui::FontId::new(18.0, crate::fonts::ui_strong())).color(crate::ui::tokens::t().fg));
        if ed.dirty() { ui.label(RichText::new("● ungespeichert").color(crate::theme::warn())); }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if crate::shell::action_button(ui, "Speichern & Projekt bauen", crate::shell::Kind::Primary, ed.dirty() && problems.is_empty()).clicked() { act = Act::SaveBuild; }
            if ui.add_enabled(ed.dirty(), egui::Button::new("Speichern")).clicked() { act = Act::Save; }
            if ui.add_enabled(ed.dirty(), egui::Button::new("Zurücksetzen")).clicked() { act = Act::Revert; }
        });
    });
    if !problems.is_empty() {
        crate::ui::widgets::card(ui, false, |ui| { for p in &problems { ui.label(RichText::new(format!("Achtung: {p}")).color(crate::theme::bad())); } });
    }
    ui.separator();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let pf = &mut ed.draft;
        let (pic, pic_note) = (&mut ed.pic, &mut ed.pic_note);
        // --- Allgemein ---
        egui::CollapsingHeader::new(RichText::new("Allgemein").font(egui::FontId::new(15.0, crate::fonts::ui_strong()))).default_open(true).show(ui, |ui| {
            egui::Grid::new("pe_general").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut pf.project.name).desired_width(360.0)); ui.end_row();
                ui.label("Beschreibung"); ui.add(egui::TextEdit::multiline(&mut pf.project.description).desired_rows(2).desired_width(360.0)); ui.end_row();
            });
            ui.checkbox(&mut pf.restore_tabs, "Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen").on_hover_text("Merkt sich die offenen Reiter und den aktiven Reiter pro Rechner (nicht in der .vcprj) und öffnet sie beim nächsten Öffnen des Projekts wieder");
            // Projektbild: projektbild.png im Projektordner; Austauschen und Löschen wirken sofort (nicht erst mit „Speichern“)
            let mut want = !pf.no_preview;
            if ui.checkbox(&mut want, "Projektbild verwenden (projektbild.png)").on_hover_text("Beim ersten Halt im Debugger wird das Emulatorbild automatisch als Projektbild abgelegt, falls es noch keines gibt. Es erscheint in „Zuletzt geöffnete Projekte“ und wird mit dem Projekt eingecheckt.").changed() { pf.no_preview = !want; }
            if want {
                ui.horizontal(|ui| {
                    let ctx = ui.ctx().clone();
                    let has = match pic.get(&ctx, root) {
                        Some((tex, size)) => { let s = crate::projectpic::fit(size, egui::vec2(160.0, 112.0)); ui.add(egui::Image::new((tex.id(), s)).corner_radius(4.0)); true }
                        None => { ui.label(RichText::new("noch kein Projektbild").weak()); false }
                    };
                    ui.vertical(|ui| {
                        if ui.button("Bild wählen …").on_hover_text("Ersetzt das Projektbild durch eine Bilddatei (PNG, JPEG)").clicked() {
                            if let Some(f) = rfd::FileDialog::new().set_title("Projektbild wählen").add_filter("Bild", &["png", "jpg", "jpeg", "gif", "bmp"]).pick_file() {
                                *pic_note = match vc_ide::livebild::import_project_preview(root, &f) { Ok(_) => "Projektbild ersetzt.".into(), Err(e) => format!("Fehlgeschlagen: {e}") };
                                pic.refresh();
                            }
                        }
                        if ui.add_enabled(has, egui::Button::new("Löschen")).on_hover_text("Löscht projektbild.png; beim nächsten Halt im Debugger wird ein neues angelegt, solange das Projektbild verwendet wird").clicked() {
                            *pic_note = match vc_ide::livebild::delete_project_preview(root) { Ok(_) => "Projektbild gelöscht.".into(), Err(e) => format!("Fehlgeschlagen: {e}") };
                            pic.refresh();
                        }
                        ui.label(RichText::new("Aus dem laufenden Emulator: Reiter „Emulator“ ▸ „Als Projektbild setzen“.").small().weak());
                        if !pic_note.is_empty() { ui.label(RichText::new(pic_note.as_str()).small()); }
                    });
                });
            }
        });
        // --- Targets ---
        egui::CollapsingHeader::new(RichText::new(format!("Targets ({})", pf.targets.len())).font(egui::FontId::new(15.0, crate::fonts::ui_strong()))).id_salt("pe_targets").default_open(true).show(ui, |ui| {
            let mut remove: Option<usize> = None;
            for (i, t) in pf.targets.iter_mut().enumerate() {
                crate::ui::widgets::card(ui, false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&t.id).strong().monospace());
                        if let Some((_, cpu)) = env.targets.iter().find(|(id, _)| *id == t.id) { ui.label(RichText::new(format!("CPU {cpu}")).weak().small()); }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("Entfernen").clicked() { remove = Some(i); } });
                    });
                    egui::Grid::new(("pe_t", i)).num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                        ui.label("Videonorm"); egui::ComboBox::from_id_salt(("pe_var", i)).selected_text(if t.variant.is_empty() { "Standard" } else { t.variant.as_str() }).show_ui(ui, |ui| { for v in ["", "pal", "ntsc"] { ui.selectable_value(&mut t.variant, v.to_string(), if v.is_empty() { "Standard" } else { v }); } }); ui.end_row();
                        if t.id == "vic20" || t.system == "vic20" {
                            // Speicherausbau des VC-20 (siehe data/targets/vic20.target.json): bestimmt BASIC-Start, Bildschirm-/Farb-RAM und den Emulator-Ausbau
                            const MEM: [(&str, &str); 7] = [("", "Standard (Grundgerät)"), ("unexpanded", "Grundgerät (5 KB)"), ("3k", "+3 KB"), ("8k", "+8 KB"), ("16k", "+16 KB"), ("24k", "+24 KB"), ("35k", "voll ausgebaut (35 KB)")];
                            let cur = t.extra.get("memory").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            ui.label("Speicherausbau");
                            egui::ComboBox::from_id_salt(("pe_mem", i)).selected_text(MEM.iter().find(|m| m.0 == cur).map_or(cur.as_str(), |m| m.1)).show_ui(ui, |ui| {
                                for (id, lab) in MEM { if ui.selectable_label(cur == id, lab).clicked() { if id.is_empty() { t.extra.remove("memory"); } else { t.extra.insert("memory".into(), serde_json::Value::String(id.into())); } } }
                            });
                            ui.end_row();
                        }
                        // Überschreibungen der globalen Einstellungen (nur für dieses Projekt); settings.json bleibt maßgeblich
                        {
                            let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
                            let g = vc_ide::settings::get().released();
                            let emus: Vec<(String, String)> = g.target(&sys).map(|c| c.emulators.iter().map(|e| (e.id.clone(), e.display_name().to_string())).collect()).unwrap_or_default();
                            let cur = emus.iter().find(|e| e.0 == t.default_emulator).map_or(if t.default_emulator.is_empty() { "wie in den Einstellungen" } else { "(unbekannt, gilt nicht)" }, |e| e.1.as_str()).to_string();
                            ui.label("Standard-Emulator");
                            egui::ComboBox::from_id_salt(("pe_defemu", i)).selected_text(cur).show_ui(ui, |ui| {
                                ui.selectable_value(&mut t.default_emulator, String::new(), "wie in den Einstellungen");
                                for (id, name) in &emus { ui.selectable_value(&mut t.default_emulator, id.clone(), name.as_str()); }
                            }).response.on_hover_text("Überschreibt nur für dieses Projekt, welcher Emulator des Systems beim Starten und Debuggen gilt");
                            ui.end_row();
                            let machines: Vec<(String, String)> = g.machines_for(&sys).map(|m| (m.id.clone(), m.display_name().to_string())).collect();
                            let cur = match t.start_on.as_str() { "" => "wie in den Einstellungen".to_string(), "@emulator" => "Emulator".to_string(), id => machines.iter().find(|m| m.0 == id).map_or("(unbekannt, gilt nicht)".to_string(), |m| m.1.clone()) };
                            ui.label("Startziel (F5)");
                            egui::ComboBox::from_id_salt(("pe_starton", i)).selected_text(cur).show_ui(ui, |ui| {
                                ui.selectable_value(&mut t.start_on, String::new(), "wie in den Einstellungen");
                                ui.selectable_value(&mut t.start_on, "@emulator".to_string(), "Emulator");
                                for (id, name) in &machines { ui.selectable_value(&mut t.start_on, id.clone(), format!("{name} (Hardware)")); }
                            }).response.on_hover_text("Überschreibt nur für dieses Projekt, wo F5 startet. Zugangsdaten bleiben in den globalen Einstellungen");
                            ui.end_row();
                        }
                        ui.label("Undokumentierte Befehle"); egui::ComboBox::from_id_salt(("pe_ill", i)).selected_text(if t.illegals.is_empty() { "Standard" } else { t.illegals.as_str() }).show_ui(ui, |ui| { for v in ["", "deny", "warn", "allow"] { ui.selectable_value(&mut t.illegals, v.to_string(), if v.is_empty() { "Standard" } else { v }); } }); ui.end_row();
                        ui.label("Emulator"); egui::ComboBox::from_id_salt(("pe_emu", i)).selected_text(if t.emulator.is_empty() { "Standard" } else { t.emulator.as_str() }).show_ui(ui, |ui| { for v in ["", "vice", "atari800", "xemu"] { ui.selectable_value(&mut t.emulator, v.to_string(), if v.is_empty() { "Standard" } else { v }); } }); ui.end_row();
                    });
                });
            }
            if let Some(i) = remove { pf.targets.remove(i); }
            let free: Vec<&(String, String)> = env.targets.iter().filter(|(id, _)| !pf.targets.iter().any(|t| &t.id == id)).collect();
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("pe_addtarget").selected_text(if ed.new_target.is_empty() { "Target hinzufügen …" } else { ed.new_target.as_str() }).show_ui(ui, |ui| { for (id, cpu) in &free { ui.selectable_value(&mut ed.new_target, id.clone(), format!("{id}  (CPU {cpu})")); } });
                if ui.add_enabled(!ed.new_target.is_empty(), egui::Button::new("Hinzufügen")).clicked() {
                    let id = std::mem::take(&mut ed.new_target);
                    let cpu = env.targets.iter().find(|(i, _)| *i == id).map(|t| t.1.clone()).unwrap_or_default();
                    pf.targets.push(TargetDef { system: id.clone(), emulator: emulator_for(&id).into(), cpu, id, ..Default::default() });
                }
            });
        });
        // --- Quellen ---
        let all_sources: Vec<String> = pf.source_files().into_iter().map(|s| s.0).collect();
        egui::CollapsingHeader::new(RichText::new(format!("Quellen und Startprogramm ({} Programme)", all_sources.len())).font(egui::FontId::new(15.0, crate::fonts::ui_strong()))).id_salt("pe_sources").default_open(true).show(ui, |ui| {
            ui.label(RichText::new("Alle Programme werden bei „Projekt bauen“ in dieser Reihenfolge gebaut. ★ = Startprogramm (F5 / F6).").weak().small());
            let target_ids: Vec<String> = pf.targets.iter().map(|t| t.id.clone()).collect();
            let main = pf.main_file();
            let mut set_main: Option<String> = None; let mut del_group: Option<usize> = None;
            let mut mv: Option<(usize, usize, i32)> = None; let mut rm: Option<(usize, usize)> = None; let mut add: Option<(usize, String)> = None;
            for (gi, g) in pf.sources.iter_mut().enumerate() {
                crate::ui::widgets::card(ui, false, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Art"); egui::ComboBox::from_id_salt(("pe_kind", gi)).width(90.0).selected_text(if g.kind == "basic" { "BASIC" } else { "Assembler" }).show_ui(ui, |ui| { ui.selectable_value(&mut g.kind, "asm".into(), "Assembler"); ui.selectable_value(&mut g.kind, "basic".into(), "BASIC"); });
                        ui.label("Target");
                        let cur = g.targets.first().cloned().unwrap_or_default();
                        egui::ComboBox::from_id_salt(("pe_gt", gi)).width(90.0).selected_text(if cur.is_empty() { "—".to_string() } else { cur.clone() }).show_ui(ui, |ui| { for t in &target_ids { if ui.selectable_label(&cur == t, t).clicked() { g.targets = vec![t.clone()]; } } });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("Gruppe löschen").clicked() { del_group = Some(gi); } });
                    });
                    if g.kind == "basic" {
                        ui.horizontal(|ui| {
                            ui.label("BASIC-Dialekt");
                            egui::ComboBox::from_id_salt(("pe_dia", gi)).width(160.0).selected_text(if g.dialect.is_empty() { "Standard des Targets".to_string() } else { g.dialect.clone() }).show_ui(ui, |ui| {
                                ui.selectable_value(&mut g.dialect, String::new(), "Standard des Targets");
                                for d in env.dialects { ui.selectable_value(&mut g.dialect, d.clone(), d.as_str()); }
                            });
                            ui.label(RichText::new("natives BASIC (.bas/.lst/.tur): Zeilennummern optional, Sprungmarken @name").weak().small());
                        });
                        // Konvertierung BASIC → Assembler beim Bauen (basic2asm)
                        let mut on = g.convert.is_some();
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut on, "Beim Bauen nach Assembler konvertieren").on_hover_text("Erst basic2asm, dann wird das Ergebnis gebaut; die .bas bleibt die Quelle").changed() {
                                g.convert = if on { Some(vc_ide::projfile::ConvertSpec { track_line: true, ..Default::default() }) } else { None };
                            }
                        });
                        if let Some(cv) = &mut g.convert {
                            ui.horizontal(|ui| {
                                ui.label("Laufzeit für");
                                let shown = if cv.target.is_empty() { "Target der Gruppe".to_string() } else { cv.target.to_uppercase() };
                                egui::ComboBox::from_id_salt(("pe_cvt", gi)).width(130.0).selected_text(shown).show_ui(ui, |ui| {
                                    ui.selectable_value(&mut cv.target, String::new(), "Target der Gruppe");
                                    for t in vc_ide::b2a::targets() { ui.selectable_value(&mut cv.target, t.to_string(), t.to_uppercase()); }
                                });
                                ui.label("Ausgabe"); ui.add(egui::TextEdit::singleline(&mut cv.out).hint_text("Standard: <name>_asm.asm").desired_width(190.0));
                            });
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut cv.int_div, "Ganzzahl-Division"); ui.checkbox(&mut cv.float, "Fließkomma erzwingen");
                                ui.checkbox(&mut cv.track_line, "Zeilenverfolgung"); ui.checkbox(&mut cv.no_rem, "REM weglassen");
                                ui.checkbox(&mut cv.inline, "Stil inline"); ui.checkbox(&mut cv.embed, "Laufzeit einbetten"); ui.checkbox(&mut cv.turbo, "Turbo BASIC XL");
                            });
                        }
                    }
                    let n = g.files.len();
                    for (fi, f) in g.files.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let is_main = main.as_deref() == Some(f.as_str());
                            if ui.add(egui::Button::new(RichText::new(if is_main { "★" } else { "☆" }).color(if is_main { crate::theme::warn() } else { Color32::GRAY })).frame(false)).on_hover_text("Als Startprogramm festlegen").clicked() { set_main = Some(f.clone()); }
                            let missing = !root.join(f).is_file();
                            ui.label(if missing { RichText::new(f).color(crate::theme::bad()) } else if is_main { RichText::new(f).strong() } else { RichText::new(f) });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("Entf.").on_hover_text("Aus dem Projekt entfernen (die Datei bleibt erhalten)").clicked() { rm = Some((gi, fi)); }
                                if ui.add_enabled(fi + 1 < n, egui::Button::new("Ab").small()).clicked() { mv = Some((gi, fi, 1)); }
                                if ui.add_enabled(fi > 0, egui::Button::new("Auf").small()).clicked() { mv = Some((gi, fi, -1)); }
                            });
                        });
                    }
                    let cands: Vec<&String> = env.files.iter().filter(|f| is_entry_candidate(f) && kind_matches(&g.kind, f) && !all_sources.contains(*f)).collect();
                    egui::ComboBox::from_id_salt(("pe_addfile", gi)).selected_text("Datei hinzufügen …").show_ui(ui, |ui| { for c in &cands { if ui.selectable_label(false, c.as_str()).clicked() { add = Some((gi, (*c).clone())); } } if cands.is_empty() { ui.label(RichText::new("keine weiteren Dateien dieser Art").weak()); } });
                });
            }
            if let Some((g, f, d)) = mv { let to = (f as i32 + d) as usize; pf.sources[g].files.swap(f, to); }
            if let Some((g, f)) = rm { let name = pf.sources[g].files.remove(f); if pf.main.as_deref() == Some(name.as_str()) { pf.main = None; } if let Some(d) = &mut pf.disk { d.files.retain(|x| *x != name); } }
            if let Some((g, f)) = add { pf.sources[g].files.push(f); }
            if let Some(g) = del_group { for name in pf.sources[g].files.clone() { if pf.main.as_deref() == Some(name.as_str()) { pf.main = None; } } pf.sources.remove(g); }
            if let Some(m) = set_main { pf.main = Some(m); }
            if ui.button("Quellgruppe hinzufügen").clicked() { pf.sources.push(Source { targets: pf.targets.first().map(|t| vec![t.id.clone()]).unwrap_or_default(), kind: "asm".into(), files: vec![] , ..Default::default() }); }
        });
        // --- Projektstruktur ---
        egui::CollapsingHeader::new(RichText::new("Projektstruktur").font(egui::FontId::new(15.0, crate::fonts::ui_strong()))).id_salt("pe_struct").default_open(!pf.layout.is_empty() || !pf.rules.is_empty()).show(ui, |ui| {
            use vc_ide::importfiles::{valid_dir, Category};
            ui.label(RichText::new("Wohin kommen abgelegte Dateien? Gilt für Drag and Drop in die IDE; leer = Vorschlag der IDE. Dateien an anderer Stelle bleiben gültig.").weak().small());
            let counts: std::collections::HashMap<String, usize> = pf.assets.iter().fold(Default::default(), |mut m, a| { *m.entry(a.category.clone()).or_default() += 1; m });
            egui::Grid::new("pe_layout").num_columns(3).spacing([10.0, 4.0]).show(ui, |ui| {
                for h in ["Kategorie", "Zielordner", "Dateien"] { ui.label(RichText::new(h).small().weak()); }
                ui.end_row();
                for c in Category::CHOICES.iter().copied().chain([Category::Studio]) {
                    ui.label(c.label());
                    let mut d = pf.layout.get(c.id()).cloned().unwrap_or_default();
                    let bad = !d.is_empty() && !valid_dir(&d);
                    let mut te = egui::TextEdit::singleline(&mut d).hint_text(if c.dir().is_empty() { "Projektordner" } else { c.dir() }).desired_width(200.0);
                    if bad { te = te.text_color(crate::theme::bad()); }
                    if ui.add_sized([200.0, ui.spacing().interact_size.y], te).changed() { if d.trim().is_empty() { pf.layout.remove(c.id()); } else { pf.layout.insert(c.id().to_string(), d); } }
                    ui.label(RichText::new(counts.get(c.id()).copied().unwrap_or(0).to_string()).weak().small());
                    ui.end_row();
                }
            });
            ui.add_space(4.0);
            ui.label(RichText::new("Regeln (die erste passende gilt, vor der Kategorie): * und ? im Dateinamen").weak().small());
            let mut rm: Option<usize> = None;
            egui::Grid::new("pe_rules").num_columns(4).spacing([10.0, 4.0]).show(ui, |ui| {
                for (i, r) in pf.rules.iter_mut().enumerate() {
                    ui.add_sized([110.0, ui.spacing().interact_size.y], egui::TextEdit::singleline(&mut r.pattern).hint_text("*.sid"));
                    let cur = Category::from_id(&r.category).map_or("wie erkannt", |c| c.label());
                    egui::ComboBox::from_id_salt(("pe_rule_cat", i)).width(140.0).selected_text(cur).show_ui(ui, |ui| {
                        ui.selectable_value(&mut r.category, String::new(), "wie erkannt");
                        for c in Category::CHOICES { ui.selectable_value(&mut r.category, c.id().to_string(), c.label()); }
                    });
                    let bad = !r.dir.is_empty() && !valid_dir(&r.dir);
                    let mut te = egui::TextEdit::singleline(&mut r.dir).hint_text("Ordner der Kategorie").desired_width(160.0);
                    if bad { te = te.text_color(crate::theme::bad()); }
                    ui.add_sized([160.0, ui.spacing().interact_size.y], te);
                    if ui.small_button("Entf.").clicked() { rm = Some(i); }
                    ui.end_row();
                }
            });
            if let Some(i) = rm { pf.rules.remove(i); }
            ui.horizontal(|ui| {
                if ui.button("+ Regel").clicked() { pf.rules.push(Default::default()); }
                if !env.structures.is_empty() {
                    egui::ComboBox::from_id_salt("pe_struct_tpl").selected_text("Vorlage laden …").show_ui(ui, |ui| {
                        for s in env.structures { if ui.selectable_label(false, if s.user { format!("{} (eigene)", s.name) } else { s.name.clone() }).on_hover_text(&s.description).clicked() { pf.layout = s.layout.clone(); ed.note = format!("Zielordner der Vorlage „{}“ übernommen (Ordner mit „Ordner anlegen“).", s.name); } }
                    });
                }
                if ui.button("Ordner anlegen").on_hover_text("Fehlende Zielordner aus Kategorien und Regeln im Projekt anlegen").clicked() {
                    let dirs: Vec<String> = pf.layout.values().cloned().chain(pf.rules.iter().map(|r| r.dir.clone())).map(|d| d.trim_matches('/').to_string()).filter(|d| !d.is_empty() && valid_dir(d)).collect();
                    let mut made = 0;
                    for d in &dirs { if !root.join(d).exists() && std::fs::create_dir_all(root.join(d)).is_ok() { made += 1; } }
                    ed.note = format!("{made} Ordner angelegt.");
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut ed.tpl_name).hint_text("Name der Vorlage").desired_width(160.0));
                if ui.add_enabled(!ed.tpl_name.trim().is_empty(), egui::Button::new("Als Vorlage speichern")).on_hover_text("Ordner, Zielordner und Rollen dieses Projekts als eigene Vorlage ablegen (für neue Projekte im Wizard)").clicked() {
                    let s = vc_ide::structure::from_project(root, pf, &ed.tpl_name);
                    ed.note = match vc_ide::structure::user_file().ok_or_else(|| "kein Konfigurationsordner".to_string()).and_then(|f| vc_ide::structure::save_user(&f, &s).map(|_| f)) {
                        Ok(f) => format!("Vorlage „{}“ gespeichert ({}).", s.name, f.display()),
                        Err(e) => format!("Speichern fehlgeschlagen: {e}"),
                    };
                }
            });
            if !ed.note.is_empty() { ui.label(RichText::new(&ed.note).small().color(crate::ui::tokens::t().gr)); }
            // Baumvorschau: vorhandene und vorgesehene Ordner mit Rolle und Zuordnung
            let key = serde_json::to_string(&(&pf.layout, &pf.rules, &pf.folders)).unwrap_or_default();
            if ed.preview.0 != key { ed.preview = (key, vc_ide::structure::preview_project(root, pf)); }
            ui.add_space(4.0);
            ui.label(RichText::new("Vorschau").small().weak());
            egui::Frame::new().fill(crate::ui::tokens::t().bg2).corner_radius(4.0).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                if ed.preview.1.is_empty() { ui.label(RichText::new("noch keine Ordner").small().weak()); }
                for l in ed.preview.1.iter().take(60) {
                    ui.horizontal(|ui| {
                        ui.add_space(l.depth as f32 * 14.0);
                        ui.label(RichText::new(format!("{}/", l.name)).monospace().small().color(if l.new { crate::ui::tokens::t().dim } else { crate::ui::tokens::t().fg }));
                        if l.new { ui.label(RichText::new("neu").small().color(crate::ui::tokens::t().cy)); }
                        if !l.note.is_empty() { ui.label(RichText::new(&l.note).small().weak()); }
                    });
                }
                if ed.preview.1.len() > 60 { ui.label(RichText::new(format!("… {} weitere", ed.preview.1.len() - 60)).small().weak()); }
            });
        });
        // --- Diskette ---
        egui::CollapsingHeader::new(RichText::new("Diskette / Datenträger").font(egui::FontId::new(15.0, crate::fonts::ui_strong()))).id_salt("pe_disk").default_open(pf.disk.is_some()).show(ui, |ui| {
            let mut on = pf.disk.is_some();
            if ui.checkbox(&mut on, "Beim Projektbau ein Diskettenabbild erzeugen").changed() {
                pf.disk = if on { Some(DiskSpec { format: match pf.targets.first().map(|t| t.id.as_str()) { Some("atari8") => "atr", Some("mega65") => "d81", _ => "d64" }.into(), ..Default::default() }) } else { None };
            }
            let srcs: Vec<String> = pf.source_files().into_iter().map(|s| s.0).collect();
            if let Some(d) = &mut pf.disk {
                egui::Grid::new("pe_diskgrid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.label("Format"); egui::ComboBox::from_id_salt("pe_dfmt").selected_text(match d.format.as_str() { "d64" => "D64 (C64/1541)", "d71" => "D71 (1571)", "d81" => "D81 (1581, MEGA65)", "atr" => "ATR (Atari, DOS 2)", o => o }).show_ui(ui, |ui| {
                        for (v, l) in [("d64", "D64 (C64/1541)"), ("d71", "D71 (1571)"), ("d81", "D81 (1581, MEGA65)"), ("atr", "ATR (Atari, DOS 2)")] { ui.selectable_value(&mut d.format, v.into(), l); }
                    }); ui.end_row();
                    ui.label("Diskettenname"); ui.add(egui::TextEdit::singleline(&mut d.name).hint_text("Projektname").desired_width(260.0)); ui.end_row();
                    ui.label("Ausgabedatei"); ui.add(egui::TextEdit::singleline(&mut d.file).hint_text(format!("build/<Ordnername>.{}", d.format)).desired_width(260.0)); ui.end_row();
                });
                ui.label("Programme auf der Diskette (nichts angehakt = alle):");
                for s in &srcs {
                    let mut chk = d.files.contains(s);
                    if ui.checkbox(&mut chk, s.as_str()).changed() { if chk { d.files.push(s.clone()); } else { d.files.retain(|x| x != s); } }
                }
                // Auswahl in Projektreihenfolge halten
                d.files.sort_by_key(|f| srcs.iter().position(|s| s == f).unwrap_or(usize::MAX));
            }
        });
    });
    act
}
