// SPDX-License-Identifier: GPL-3.0-or-later
//! Rückfrage beim Ablegen von Dateien im Projekt: je Datei Name, erkannte Art, Kategorie, Zielordner und – bei einem Namenskonflikt –
//! Umbenennen, Ersetzen oder Überspringen. „Für alle übernehmen“ gibt die Wahl der ersten Datei an alle weiter.
use crate::ui::dialog::{self, Act, Dlg};
use crate::ui::tokens::t;
use eframe::egui::{self, RichText};
use std::path::{Path, PathBuf};
use vc_ide::importfiles::{self, Category, Conflict, Kind, Plan, Target};

pub struct Row {
    pub on: bool,
    pub plan: Plan,
    pub kind: Kind,
    /// Stammt aus einem abgelegten Ordner: Unterordner (`pack/sub`), unter dem die Datei im Projekt landet
    pub struct_dir: Option<String>,
    /// Zustand des Ziels (zwischengespeichert, solange Name und Ordner gleich bleiben)
    state: Option<(Target, (String, String))>,
}

pub struct ImportDlg {
    pub rows: Vec<Row>,
    /// Wahl der ersten Datei für alle übernehmen
    pub all: bool,
    /// Zielordner steht fest (auf einen Ordner des Projektbaums abgelegt)
    pub fixed_dir: Option<String>,
    /// Quelltext, in den die Dateien nach der Übernahme per `.include`/`.incbin` eingefügt werden (auf den Editor abgelegt)
    pub insert_into: Option<String>,
    /// Zielordner je Kategorie aus der Projektdatei (`layout`)
    pub layout: std::collections::BTreeMap<String, String>,
    /// Regeln nach Dateimuster (`rules`), vor `layout`
    pub rules: Vec<vc_ide::projfile::Rule>,
    /// Ordnerstruktur abgelegter Ordner beibehalten
    pub keep_tree: bool,
    /// Dateien, die wegen der Obergrenze nicht in der Liste stehen
    pub over: usize,
    /// Einzelne Dateien zeigen (bei vielen Dateien zunächst nur die Zusammenfassung)
    pub details: bool,
}

/// Obergrenze der Dateien aus abgelegten Ordnern
const MAX_FILES: usize = 5000;
/// Ab so vielen Dateien zeigt der Dialog zunächst eine Zusammenfassung statt Einzelzeilen
const SUMMARY_MIN: usize = 40;
/// So viele Zeilen zeigt die Einzelansicht höchstens
const DETAIL_MAX: usize = 300;

impl ImportDlg {
    pub fn new(paths: Vec<PathBuf>, fixed_dir: Option<String>, insert_into: Option<String>, layout: std::collections::BTreeMap<String, String>, rules: Vec<vc_ide::projfile::Rule>) -> Option<ImportDlg> {
        let mut d = ImportDlg { rows: vec![], all: false, fixed_dir, insert_into, layout, rules, keep_tree: true, over: 0, details: true };
        d.add(paths);
        d.details = d.rows.len() <= SUMMARY_MIN;
        if d.rows.is_empty() { None } else { Some(d) }
    }

    /// Weitere Dateien in die Liste aufnehmen (Ordner und doppelte Pfade bleiben draußen)
    pub fn add(&mut self, paths: Vec<PathBuf>) {
        // Ordner werden rekursiv aufgelöst: jede Datei merkt ihren Unterordner (`ordner/unter`)
        let mut files: Vec<(PathBuf, Option<String>)> = vec![];
        for p in paths {
            if p.is_dir() {
                let room = MAX_FILES.saturating_sub(self.rows.len() + files.len());
                let (v, over) = importfiles::walk_folder(&p, room);
                self.over += over;
                files.extend(v.into_iter().map(|(f, sub)| (f, Some(sub))));
            } else { files.push((p, None)); }
        }
        for (p, sub) in files {
            if self.rows.iter().any(|r| r.plan.src == p) { continue; }
            if self.rows.len() >= MAX_FILES { self.over += 1; continue; }
            let mut plan = Plan::suggest_with(&p, &self.layout, &self.rules);
            let base = self.fixed_dir.clone();
            let struct_dir = sub.map(|s| importfiles::join_rel(base.as_deref().unwrap_or(""), &s));
            if let Some(dir) = &base { plan.dir = dir.clone(); }
            if let (Some(sd), true) = (&struct_dir, self.keep_tree) { plan.dir = sd.clone(); }
            let kind = plan.kind();
            self.rows.push(Row { on: true, plan, kind, struct_dir, state: None });
        }
    }

    /// Wirkt die Wahl der ersten Datei auf alle (nach jeder Änderung aufrufen)
    fn sync_all(&mut self) {
        if !self.all { return; }
        let Some((cat, dir, conflict)) = self.rows.first().map(|r| (r.plan.category, r.plan.dir.clone(), r.plan.conflict)) else { return };
        let keep = self.keep_tree;
        for r in self.rows.iter_mut().skip(1) { r.plan.category = cat; if !(keep && r.struct_dir.is_some()) { r.plan.dir = dir.clone(); } r.plan.conflict = conflict; }
    }

    /// Sind alle angehakten Zeilen gültig und ist mindestens eine angehakt?
    pub fn valid(&self) -> bool {
        self.rows.iter().any(|r| r.on) && self.rows.iter().filter(|r| r.on).all(|r| importfiles::valid_name(&r.plan.name) && importfiles::valid_dir(&r.plan.dir))
    }

    /// Die angehakten Pläne
    pub fn plans(&self) -> Vec<Plan> { self.rows.iter().filter(|r| r.on).map(|r| r.plan.clone()).collect() }

    pub fn show(&mut self, ctx: &egui::Context, root: &Path) -> Act {
        let k = t();
        let n = self.rows.len();
        let sub = match (&self.insert_into, &self.fixed_dir) {
            (Some(d), _) => format!("{n} Datei{} · danach in {d} einfügen", if n == 1 { "" } else { "en" }),
            (None, Some(f)) => format!("{n} Datei{} · Ordner {}", if n == 1 { "" } else { "en" }, if f.is_empty() { "Projektwurzel".to_string() } else { format!("{f}/") }),
            _ => format!("{n} Datei{}", if n == 1 { "" } else { "en" }),
        };
        let valid = self.valid();
        let wide = n <= SUMMARY_MIN || self.details;
        let mut changed = false;
        let act = dialog::show(ctx, &Dlg::new("Dateien ins Projekt übernehmen", "Übernehmen").sub(&sub), |ui| {
            // Die Tabelle braucht Platz: Name, Art, Kategorie, Ordner und Ziel nebeneinander
            if wide { ui.set_min_width(800.0); }
            ui.label(RichText::new("Die Dateien werden ins Projekt kopiert und mit ihrer Kategorie in der Projektdatei vermerkt. Binärdaten lassen sich im Quelltext mit .incbin einbinden.").small().color(k.mute));
            ui.add_space(4.0);
            if self.rows.iter().any(|r| r.struct_dir.is_some()) {
                if ui.checkbox(&mut self.keep_tree, "Ordnerstruktur abgelegter Ordner beibehalten").changed() {
                    for r in self.rows.iter_mut() {
                        if let Some(sd) = &r.struct_dir { r.plan.dir = if self.keep_tree { sd.clone() } else { self.fixed_dir.clone().unwrap_or_else(|| importfiles::dir_for(&self.layout, r.plan.category)) }; }
                    }
                    changed = true;
                }
            }
            if self.over > 0 { ui.colored_label(crate::theme::warn(), format!("{} weitere Datei(en) nicht aufgenommen (Obergrenze {MAX_FILES}) – den Ordner in Teilen ablegen.", self.over)); }
            let compact = n > SUMMARY_MIN && !self.details;
            if n > SUMMARY_MIN { ui.checkbox(&mut self.details, format!("Einzelne Dateien anzeigen (die ersten {DETAIL_MAX})")); }
            // Ziele prüfen (nur bei Änderung von Name oder Ordner neu)
            for r in self.rows.iter_mut() {
                let key = (r.plan.name.clone(), r.plan.dir.clone());
                if r.state.as_ref().map_or(true, |s| s.1 != key) {
                    let st = if importfiles::valid_name(&r.plan.name) && importfiles::valid_dir(&r.plan.dir) { importfiles::target_state(root, &r.plan) } else { Target::Free };
                    r.state = Some((st, key));
                }
            }
            if compact {
                // Zusammenfassung: Dateien je Kategorie, Namenskonflikte, gemeinsame Lösung
                let mut per: Vec<(Category, usize)> = vec![];
                for r in self.rows.iter().filter(|r| r.on) { match per.iter_mut().find(|x| x.0 == r.plan.category) { Some(x) => x.1 += 1, None => per.push((r.plan.category, 1)) } }
                let clashes = self.rows.iter().filter(|r| r.on && r.state.as_ref().map_or(false, |s| s.0 == Target::Clash)).count();
                let same = self.rows.iter().filter(|r| r.on && r.state.as_ref().map_or(false, |s| matches!(s.0, Target::Same | Target::InPlace))).count();
                egui::Grid::new("import_sum").num_columns(2).spacing([14.0, 4.0]).show(ui, |ui| {
                    for (c, cnt) in &per { ui.label(c.label()); ui.label(RichText::new(format!("{cnt} Dateien")).color(k.mute)); ui.end_row(); }
                });
                ui.label(RichText::new(if self.keep_tree && self.rows.iter().any(|r| r.struct_dir.is_some()) { "Ordnerstruktur bleibt erhalten; Kategorie und Zielordner je Datei wie vorgeschlagen." } else { "Kategorie und Zielordner je Datei wie vorgeschlagen." }).small().color(k.dim));
                if same > 0 { ui.label(RichText::new(format!("{same} Datei(en) sind schon mit gleichem Inhalt vorhanden und werden nicht noch einmal kopiert.")).small().color(k.mute)); }
                if clashes > 0 {
                    ui.horizontal(|ui| {
                        ui.colored_label(k.am, format!("{clashes} Namenskonflikt(e):"));
                        let cur = self.rows.iter().find(|r| r.state.as_ref().map_or(false, |s| s.0 == Target::Clash)).map_or(Conflict::Rename, |r| r.plan.conflict);
                        let mut sel = cur;
                        egui::ComboBox::from_id_salt("imp_conf_all").width(150.0).selected_text(match sel { Conflict::Rename => "umbenennen (_2 …)", Conflict::Replace => "ersetzen", Conflict::Skip => "überspringen" }).show_ui(ui, |ui| {
                            ui.selectable_value(&mut sel, Conflict::Rename, "Umbenennen (_2 …)");
                            ui.selectable_value(&mut sel, Conflict::Replace, "Ersetzen");
                            ui.selectable_value(&mut sel, Conflict::Skip, "Überspringen");
                        });
                        if sel != cur { for r in self.rows.iter_mut() { r.plan.conflict = sel; } }
                    });
                }
            }
            if !compact && n > 1 { if ui.checkbox(&mut self.all, "Für alle übernehmen (Kategorie, Ordner und Konfliktlösung der ersten Datei)").changed() { changed = true; } }
            ui.add_space(4.0);
            let all = self.all;
            let fixed = self.fixed_dir.is_some();
            let (layout, keep) = (self.layout.clone(), self.keep_tree);
            if !compact { egui::ScrollArea::vertical().max_height(340.0).auto_shrink([true, true]).show(ui, |ui| {
                egui::Grid::new("import_rows").num_columns(6).spacing([10.0, 6.0]).striped(true).show(ui, |ui| {
                    for h in ["", "Name", "Art", "Kategorie", "Ordner", "Ziel"] { ui.label(RichText::new(h).small().color(k.dim)); }
                    ui.end_row();
                    for (i, r) in self.rows.iter_mut().enumerate().take(DETAIL_MAX) {
                        let follower = all && i > 0;
                        ui.checkbox(&mut r.on, "").on_hover_text("Diese Datei übernehmen");
                        ui.add_enabled_ui(r.on, |ui| {
                            let bad = !importfiles::valid_name(&r.plan.name);
                            let mut te = egui::TextEdit::singleline(&mut r.plan.name).desired_width(150.0);
                            if bad { te = te.text_color(crate::theme::bad()); }
                            // in Tabellen zählt nur add_sized: desired_width wird in einer Grid-Spalte auf die bisherige Spaltenbreite gekürzt
                            let h = ui.spacing().interact_size.y;
                            ui.add_sized([150.0, h], te).on_hover_text(r.plan.src.to_string_lossy());
                        });
                        ui.label(RichText::new(r.kind.label()).small().color(k.mute));
                        ui.add_enabled_ui(r.on && !follower, |ui| {
                            let before = r.plan.category;
                            egui::ComboBox::from_id_salt(("imp_cat", i)).width(130.0).selected_text(r.plan.category.label()).show_ui(ui, |ui| {
                                for c in Category::CHOICES { ui.selectable_value(&mut r.plan.category, c, c.label()); }
                                // Studio-Dateien kennt der Projektbaum schon; nur als Auswahl, wenn so erkannt
                                if r.kind == Kind::Studio { ui.selectable_value(&mut r.plan.category, Category::Studio, Category::Studio.label()); }
                            });
                            if r.plan.category != before {
                                changed = true;
                                // Ordner mitführen, solange er noch der Vorschlag der alten Kategorie ist
                                if !fixed && !(keep && r.struct_dir.is_some()) && r.plan.dir == importfiles::dir_for(&layout, before) { r.plan.dir = importfiles::dir_for(&layout, r.plan.category); }
                            }
                        });
                        ui.add_enabled_ui(r.on && !follower, |ui| {
                            let bad = !importfiles::valid_dir(&r.plan.dir);
                            let mut te = egui::TextEdit::singleline(&mut r.plan.dir).desired_width(110.0).hint_text("Projektwurzel");
                            if bad { te = te.text_color(crate::theme::bad()); }
                            let h = ui.spacing().interact_size.y;
                            if ui.add_sized([120.0, h], te).changed() { changed = true; }
                        });
                        // Ziel: frei, vorhanden (gleich/anders) und was dann geschieht
                        let state = r.state.as_ref().map_or(Target::Free, |s| s.0);
                        ui.add_enabled_ui(r.on, |ui| match state {
                            Target::Free => { ui.label(RichText::new("neu").small().color(k.gr)); }
                            Target::Same | Target::InPlace => { ui.label(RichText::new("schon vorhanden (gleich)").small().color(k.mute)); }
                            Target::Clash => {
                                ui.add_enabled_ui(!follower, |ui| {
                                    let before = r.plan.conflict;
                                    egui::ComboBox::from_id_salt(("imp_conf", i)).width(150.0).selected_text(match r.plan.conflict { Conflict::Rename => "Name ist belegt: umbenennen", Conflict::Replace => "ersetzen", Conflict::Skip => "überspringen" }).show_ui(ui, |ui| {
                                        ui.selectable_value(&mut r.plan.conflict, Conflict::Rename, "Umbenennen (_2 …)");
                                        ui.selectable_value(&mut r.plan.conflict, Conflict::Replace, "Ersetzen");
                                        ui.selectable_value(&mut r.plan.conflict, Conflict::Skip, "Überspringen");
                                    });
                                    if r.plan.conflict != before { changed = true; }
                                });
                            }
                        });
                        ui.end_row();
                    }
                });
            }); if n > DETAIL_MAX { ui.label(RichText::new(format!("… und {} weitere Dateien (werden mit den vorgeschlagenen Einstellungen übernommen).", n - DETAIL_MAX)).small().color(k.dim)); } }
            if !self.rows.iter().all(|r| r.on) || self.rows.iter().any(|r| !importfiles::valid_name(&r.plan.name) || !importfiles::valid_dir(&r.plan.dir)) {
                if !valid { ui.colored_label(crate::theme::bad(), "Name oder Ordner ungültig (kein Pfad, kein „..“)."); }
            }
            valid
        });
        if changed { self.sync_all(); }
        // „Für alle“ eben angehakt: sofort angleichen
        if self.all { self.sync_all(); }
        act
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuer_alle_uebernimmt_die_wahl_der_ersten_zeile() {
        let d = std::env::temp_dir().join(format!("vc_impdlg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap();
        for n in ["a.bin", "b.txt", "c.asm"] { std::fs::write(d.join(n), [0u8, 1, 2]).unwrap(); }
        std::fs::create_dir_all(d.join("ordner")).unwrap();
        let mut dlg = ImportDlg::new(vec![d.join("a.bin"), d.join("b.txt"), d.join("c.asm"), d.join("ordner"), d.join("a.bin")], None, None, Default::default(), vec![]).unwrap();
        assert_eq!(dlg.rows.len(), 3, "leerer Ordner und Doppelte entfallen");
        assert_eq!(dlg.rows[1].plan.category, Category::Text);
        dlg.rows[0].plan.category = Category::Graphics; dlg.rows[0].plan.dir = "gfx".into();
        dlg.all = true; dlg.sync_all();
        assert!(dlg.rows.iter().all(|r| r.plan.category == Category::Graphics && r.plan.dir == "gfx"));
        assert!(dlg.valid());
        dlg.rows[2].plan.dir = "../x".into();
        dlg.all = false;
        assert!(!dlg.valid());
        for r in &mut dlg.rows { r.on = false; }
        assert!(!dlg.valid() && dlg.plans().is_empty());
        let fixed = ImportDlg::new(vec![d.join("a.bin")], Some("lib".into()), None, Default::default(), vec![]).unwrap();
        assert_eq!(fixed.rows[0].plan.dir, "lib");
        assert!(ImportDlg::new(vec![d.join("ordner")], None, None, Default::default(), vec![]).is_none(), "leerer Ordner: nichts zu übernehmen");
        // Ordner ablegen: Struktur bleibt, im Baum-Ordner darunter
        std::fs::create_dir_all(d.join("pack/sub")).unwrap();
        std::fs::write(d.join("pack/x.bin"), [0u8, 1]).unwrap(); std::fs::write(d.join("pack/sub/y.txt"), "hi").unwrap();
        let mut f = ImportDlg::new(vec![d.join("pack")], Some("res".into()), None, Default::default(), vec![]).unwrap();
        let mut dirs: Vec<(String, String)> = f.rows.iter().map(|r| (r.plan.name.clone(), r.plan.dir.clone())).collect();
        dirs.sort();
        assert_eq!(dirs, vec![("x.bin".to_string(), "res/pack".to_string()), ("y.txt".to_string(), "res/pack/sub".to_string())]);
        f.keep_tree = false;
        for r in f.rows.iter_mut() { if let Some(_) = &r.struct_dir { r.plan.dir = "res".into(); } }
        assert!(f.rows.iter().all(|r| r.plan.dir == "res"));
        let mut lay = std::collections::BTreeMap::new(); lay.insert("binary".to_string(), "res/bin".to_string());
        let g = ImportDlg::new(vec![d.join("a.bin")], None, None, lay, vec![]).unwrap();
        assert_eq!(g.rows[0].plan.dir, "res/bin", "Projektstruktur bestimmt den Vorschlag");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn viele_dateien_starten_in_der_zusammenfassung() {
        let d = std::env::temp_dir().join(format!("vc_impdlg_many_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("pack")).unwrap();
        for i in 0..(SUMMARY_MIN + 5) { std::fs::write(d.join("pack").join(format!("f{i}.bin")), [0u8, i as u8]).unwrap(); }
        let few = ImportDlg::new(vec![d.join("pack/f0.bin")], None, None, Default::default(), vec![]).unwrap();
        assert!(few.details, "wenige Dateien: Einzelansicht");
        let many = ImportDlg::new(vec![d.join("pack")], None, None, Default::default(), vec![]).unwrap();
        assert_eq!(many.rows.len(), SUMMARY_MIN + 5);
        assert!(!many.details, "viele Dateien: Zusammenfassung");
        assert!(many.valid());
        let _ = std::fs::remove_dir_all(&d);
    }
}
