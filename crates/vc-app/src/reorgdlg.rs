// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Dateien neu ordnen“: Vorschlag je Datei (abwählbar), Optionen, danach der Bericht mit Prüfbau und „Rückgängig“.
use crate::ui::dialog::{self, Act, Dlg};
use crate::ui::tokens::t;
use eframe::egui::{self, RichText};
use std::path::Path;
use vc_ide::projfile::ProjectFile;
use vc_ide::reorganize::{self, Check, Move, Opts, Report};

/// Meldungen des Hintergrundthreads, der verschiebt und den Prüfbau macht
pub enum Msg { Progress(String), Done(Result<Report, String>) }

pub enum Stage { Plan, /** läuft im Hintergrund: aktueller Schritt */ Running(String), Done(Report) }

pub struct ReorgDlg {
    pub moves: Vec<(bool, Move)>,
    pub subdirs: bool,
    /// Vorher und nachher bauen und die Ergebnisse vergleichen
    pub check: bool,
    /// Pfade in `.include`/`.incbin` der Quelltexte anpassen
    pub rewrite: bool,
    pub stage: Stage,
    /// (Dateien, Quelltexte mit geänderten Pfaden) der aktuellen Auswahl
    info: (usize, usize),
    stale: bool,
}

impl ReorgDlg {
    pub fn new(root: &Path, pf: &ProjectFile) -> ReorgDlg {
        let mut d = ReorgDlg { moves: vec![], subdirs: false, check: true, rewrite: true, stage: Stage::Plan, info: (0, 0), stale: true };
        d.replan(root, pf);
        d
    }
    fn replan(&mut self, root: &Path, pf: &ProjectFile) {
        self.moves = reorganize::plan(root, pf, &Opts { subdirs: self.subdirs }).into_iter().map(|m| (!m.conflict, m)).collect();
        self.stale = true;
    }
    /// Die angehakten Verschiebungen ohne Konflikte
    pub fn selected(&self) -> Vec<Move> { self.moves.iter().filter(|(on, m)| *on && !m.conflict).map(|(_, m)| m.clone()).collect() }

    pub fn show(&mut self, ctx: &egui::Context, root: &Path, pf: &ProjectFile) -> Act {
        let k = t();
        if self.stale { let s = self.selected(); self.info = (s.len(), reorganize::rewrites(root, &s).len()); self.stale = false; }
        let mut replan = false; let mut changed = false;
        let act = match &self.stage {
            Stage::Plan => {
                let n = self.info.0;
                let ok = n > 0;
                let sub = format!("{n} von {} Dateien · {} Quelltext(e) mit angepassten Pfaden", self.moves.len(), self.info.1);
                let (moves, subdirs, check, rewrite, texts) = (&mut self.moves, &mut self.subdirs, &mut self.check, &mut self.rewrite, self.info.1);
                dialog::show(ctx, &Dlg::new("Dateien neu ordnen", "Verschieben").sub(&sub), |ui| {
                    ui.label(RichText::new("Dateien kommen nach den Zielordnern aus den Projekt-Einstellungen (Projektstruktur). Programme, Studio-Dateien, Tests und README bleiben. Pfade in .include/.incbin und die Projektdatei werden nachgeführt; vorher entsteht eine Sicherung in .vcbackup/.").small().color(k.mute));
                    ui.add_space(4.0);
                    if ui.checkbox(subdirs, "Auch Dateien in Unterordnern umordnen").changed() { replan = true; }
                    ui.add_enabled(texts > 0, egui::Checkbox::new(rewrite, format!("Code anpassen: Pfade in {texts} Quelltext(en) nachführen (.include / .incbin)")));
                    if texts > 0 && !*rewrite { ui.label(RichText::new("Ohne Anpassung zeigen die Pfade nach dem Verschieben ins Leere; der Prüfbau meldet das.").small().color(k.am)); }
                    ui.checkbox(check, "Prüfbau: vorher und nachher bauen, Ergebnisse vergleichen (im Hintergrund)");
                    ui.add_space(4.0);
                    if moves.is_empty() { ui.label("Alles liegt schon am richtigen Ort."); }
                    egui::ScrollArea::vertical().max_height(320.0).auto_shrink([true, true]).show(ui, |ui| {
                        egui::Grid::new("reorg_rows").num_columns(4).spacing([10.0, 4.0]).striped(true).show(ui, |ui| {
                            for (on, m) in moves.iter_mut() {
                                if ui.add_enabled(!m.conflict, egui::Checkbox::without_text(on)).changed() { changed = true; }
                                ui.label(RichText::new(&m.from).monospace().small());
                                ui.label(RichText::new(format!("→ {}", m.to)).monospace().small().color(k.cy));
                                if m.conflict { crate::ui::widgets::chip(ui, "Ziel belegt", k.am); } else { ui.label(RichText::new(m.label).small().color(k.dim)); }
                                ui.end_row();
                            }
                        });
                    });
                    ok
                })
            }
            Stage::Running(step) => {
                let step = step.clone();
                let a = dialog::show(ctx, &Dlg { primary: "", secondary: "Läuft …", ..Dlg::new("Dateien werden geordnet", "") }, |ui| {
                    ui.horizontal(|ui| { ui.spinner(); ui.label(&step); });
                    ui.label(RichText::new("Bitte warten – der Prüfbau kann bei großen Projekten etwas dauern.").small().color(k.mute));
                    true
                });
                // Abbrechen gibt es nicht: die Dateien werden schon verschoben
                let _ = a; Act::None
            }
            Stage::Done(r) => {
                let (title_ok, lines): (bool, Vec<String>) = match &r.check {
                    Check::Same(n) => (true, vec![format!("Prüfbau: {n} Ergebnisdatei(en) unverändert.")]),
                    Check::Skipped(why) => (true, vec![format!("Prüfbau übersprungen: {why}")]),
                    Check::Different(v) => (false, vec![format!("Das Ergebnis weicht ab: {}", v.join(", ")), "Empfehlung: rückgängig machen und die Pfade prüfen.".into()]),
                };
                let dlg = if title_ok { Dlg { primary: "", secondary: "Schließen", ..Dlg::new("Neu geordnet", "") } } else { Dlg { secondary: "Behalten", ..Dlg::new("Neu geordnet", "Rückgängig") } };
                let sub = format!("{} Datei(en) verschoben · Sicherung {}", r.moved.len(), r.backup.file_name().map(|n| format!(".vcbackup/{}", n.to_string_lossy())).unwrap_or_default());
                let dlg = Dlg { sub: &sub, ..dlg };
                dialog::show(ctx, &dlg, |ui| {
                    for l in &lines { ui.label(RichText::new(l).color(if title_ok { k.gr } else { crate::theme::bad() })); }
                    if !r.rewritten.is_empty() { ui.add_space(4.0); ui.label(RichText::new("Angepasste Quelltexte").small().color(k.mute)); for (f, n) in &r.rewritten { ui.label(RichText::new(format!("{f}  ({n} Pfad{})", if *n == 1 { "" } else { "e" })).monospace().small()); } }
                    true
                })
            }
        };
        if replan { self.replan(root, pf); }
        if changed { self.stale = true; }
        act
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auswahl_ohne_konflikte_und_abgewaehlte() {
        let root = std::env::temp_dir().join(format!("vc_reorgdlg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
        for f in ["a.bin", "b.bin"] { std::fs::write(root.join(f), [0u8, 1]).unwrap(); }
        std::fs::create_dir_all(root.join("data")).unwrap(); std::fs::write(root.join("data/a.bin"), [9u8]).unwrap();
        let mut pf = ProjectFile::default(); pf.layout.insert("binary".into(), "data".into());
        let mut d = ReorgDlg::new(&root, &pf);
        assert_eq!(d.moves.len(), 2);
        assert!(d.moves.iter().any(|(on, m)| m.from == "a.bin" && m.conflict && !on), "Konflikt ist abgewählt");
        assert_eq!(d.selected().len(), 1);
        d.moves.iter_mut().for_each(|(on, _)| *on = false);
        assert!(d.selected().is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
