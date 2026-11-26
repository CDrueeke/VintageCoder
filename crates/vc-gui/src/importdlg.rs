// SPDX-License-Identifier: GPL-3.0-or-later
//! Rückfrage beim Ablegen von Dateien im Projekt (Fenster oder Baum): je Datei Name, erkannte Art, Kategorie und – bei einem Namenskonflikt –
//! Umbenennen, Ersetzen oder Überspringen. Das Modell (Erkennung, Zielordner, Übernahme) ist `vc_ide::importfiles`.
use super::*;
use std::path::Path;
use vc_ide::importfiles::{self, Category, Conflict, Plan, Target};

pub struct ImportZeile { pub an: bool, pub plan: Plan }
pub struct ImportDialog { pub zeilen: Vec<ImportZeile>, pub ueber: usize }

const MAX: usize = 200;
const KATEGORIEN: [Category; 6] = [Category::Source, Category::Text, Category::Binary, Category::Graphics, Category::Charset, Category::Studio];

/// Wird die Zeile bei „übernehmen“ wirklich etwas tun? Neu und „Name belegt“ (außer „Überspringen“) ja; gleicher Inhalt am Ziel oder Datei liegt schon dort: nie
pub fn uebernehmbar(root: &Path, z: &ImportZeile) -> bool { match importfiles::target_state(root, &z.plan) { Target::Free => true, Target::Clash => z.plan.conflict != Conflict::Skip, Target::Same | Target::InPlace => false } }
/// Anzahl der Zeilen, die bei „übernehmen“ wirklich etwas tun (angehakt und übernehmbar)
pub fn wirksam(root: &Path, d: &ImportDialog) -> usize { d.zeilen.iter().filter(|z| z.an && uebernehmbar(root, z)).count() }

impl ImportDialog {
    /// Vorgabe der Haken nach dem Zustand am Ziel: nur Übernehmbares ist angehakt (Name belegt: Vorgabe „Umbenennen“)
    /// Alle Dateien kommen in den im Baum gewählten Ordner (leer = Projektordner); keine Regeln nach Dateiart
    pub fn in_ordner(&mut self, ordner: &str) { for z in &mut self.zeilen { z.plan.dir = ordner.trim_matches('/').to_string() } }
    pub fn vorgaben(&mut self, root: &Path) { for z in &mut self.zeilen { z.an = uebernehmbar(root, z) } }
    pub fn neu(pfade: Vec<PathBuf>, layout: &std::collections::BTreeMap<String, String>) -> Option<ImportDialog> {
        let mut dateien: Vec<PathBuf> = vec![]; let mut ueber = 0;
        for p in pfade { if p.is_dir() { let (liste, mehr) = importfiles::walk_folder(&p, MAX); dateien.extend(liste.into_iter().map(|(f, _)| f)); ueber += mehr } else { dateien.push(p) } }
        if dateien.len() > MAX { ueber += dateien.len() - MAX; dateien.truncate(MAX) }
        let zeilen: Vec<_> = dateien.iter().filter(|p| p.is_file()).map(|p| ImportZeile { an: true, plan: Plan::suggest_in(p, layout) }).collect();
        if zeilen.is_empty() { None } else { Some(ImportDialog { zeilen, ueber }) }
    }
}

impl App {
    /// Abgelegte Dateien: außerhalb des Projekts → Rückfrage, innerhalb → öffnen
    pub(crate) fn abgelegt(&mut self, p: PathBuf) {
        if self.disk_ablegen(&p) { return }
        if p.is_dir() && std::fs::read_dir(&p).is_ok_and(|mut d| d.any(|e| e.is_ok_and(|e| e.file_name().to_string_lossy().ends_with(".vcprj")))) || p.extension().is_some_and(|e| e == "vcprj") { self.projekt_oeffnen(p); return }
        let drin = p.canonicalize().ok().zip(self.root.canonicalize().ok()).is_some_and(|(a, r)| a.starts_with(&r));
        if drin || p.is_file() && self.root.as_os_str().is_empty() { self.oeffne_pfad(p); return }
        match &mut self.import {
            Some(d) => { if let Some(mut n) = ImportDialog::neu(vec![p], &Default::default()) { n.in_ordner(&self.baum_ziel); n.vorgaben(&self.root); d.zeilen.extend(n.zeilen) } }
            None => { self.import = ImportDialog::neu(vec![p], &Default::default()); let ziel = self.baum_ziel.clone(); if let Some(d) = &mut self.import { d.in_ordner(&ziel); d.vorgaben(&self.root) } self.ed_aktiv = false }
        }
    }
    pub(crate) fn import_ausfuehren(&mut self) {
        let Some(d) = self.import.take() else { return };
        if wirksam(&self.root, &d) == 0 { self.log.push(vc_i18n::t!("importdlg-nichts-zu-uebernehmen-alle-dateien-liegen").into()); return }
        let mut erste: Option<String> = None; let (mut ok, mut fehler) = (0, 0);
        for z in d.zeilen.iter().filter(|z| z.an && uebernehmbar(&self.root, z)) {
            match importfiles::import(&self.root, self.proj.pf.as_mut(), &z.plan) {
                Ok(o) => { ok += 1; if let Some(r) = o.rel() { self.log.push(vc_i18n::t!("importdlg-uebernommen-r", r = &r)); if erste.is_none() { erste = Some(r.to_string()) } } else if let importfiles::Outcome::Skipped(r) = &o { self.log.push(vc_i18n::t!("importdlg-uebersprungen-r", r = &r)) } }
                Err(e) => { fehler += 1; self.log.push(e) }
            }
        }
        if let (Some(pf), Some(f)) = (&self.proj.pf, self.proj.file.clone()) { if let Err(e) = pf.save(&self.root.join(f)) { self.log.push(e) } }
        self.projekt_neu_laden(); self.log.push(vc_i18n::t!("importdlg-ok-datei-en-uebernommen-p2", ok = &ok, p2 = if fehler > 0 { vc_i18n::t!("importdlg-fehler-fehler", fehler = &fehler) } else { String::new() }));
        if let Some(r) = erste { self.oeffne(&r) }
    }
    pub(crate) fn import_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.import.as_ref()?; let mono = vc_gui_ui::fonts::MONO;
        let mut liste = iced::widget::column![].spacing(8);
        for (i, z) in d.zeilen.iter().enumerate().take(12) {
            let zustand = importfiles::target_state(&self.root, &z.plan);
            let art = z.plan.kind().label(); let pfad = importfiles::join_rel(&z.plan.dir, &z.plan.name);
            let konflikt = |name: &str, c: Conflict| -> Element<Msg> { let aktiv = z.plan.conflict == c; button(text(name.to_string()).size(11).color(pal.text)).padding([2, 8]).on_press(Msg::ImportKonflikt(i, c)).style(move |_, _| button::Style { background: Some(if aktiv { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if aktiv { pal.accent } else { pal.line }, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }).into() };
            let (txt, farbe) = match zustand { Target::Free => (vc_i18n::t!("importdlg-neu"), pal.ok), Target::Clash => (vc_i18n::t!("importdlg-name-belegt-anderer-inhalt").to_string(), pal.warn), Target::Same => (vc_i18n::t!("importdlg-schon-vorhanden-gleicher-inhalt-wird-nicht").to_string(), pal.mute), Target::InPlace => (vc_i18n::t!("importdlg-liegt-schon-dort-wird-nicht-uebernommen").to_string(), pal.mute) };
            let kann = uebernehmbar(&self.root, z); let mut zeile = iced::widget::row![iced::widget::checkbox(z.an && kann).on_toggle_maybe(kann.then_some(move |b| Msg::ImportAn(i, b))).size(14), text_input(&vc_i18n::t!("importdlg-name"), &z.plan.name).on_input(move |n| Msg::ImportName(i, n)).size(12).padding(3).width(190).font(mono), text(art).size(11.5).color(pal.mute).width(110),
                crate::pfadfeld::pfadfeld(pal, &vc_i18n::t!("importdlg-ordner-im-projekt"), &z.plan.dir, crate::pfadfeld::PfadArt::ProjektOrdner, Length::Fixed(150.0), move |o| Msg::ImportOrdner(i, o)),
                text(vc_i18n::t!("importdlg-pfad", pfad = &pfad)).size(11.5).font(mono).color(pal.mute), space::horizontal(), text(txt).size(11.5).color(farbe)].spacing(8).align_y(Alignment::Center);
            if zustand == Target::Clash { zeile = zeile.push(konflikt(&vc_i18n::t!("importdlg-umbenennen"), Conflict::Rename)).push(konflikt(&vc_i18n::t!("importdlg-ersetzen"), Conflict::Replace)).push(konflikt(&vc_i18n::t!("importdlg-ueberspringen"), Conflict::Skip)) }
            if matches!(zustand, Target::Same | Target::InPlace) { zeile = zeile.push(button(text(vc_i18n::t!("importdlg-als-kopie-unter-neuem-namen")).size(11).color(pal.text)).padding([2, 8]).on_press(Msg::ImportKopie(i)).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() })) }
            liste = liste.push(zeile);
        }
        let mehr = d.zeilen.len().saturating_sub(12) + d.ueber;
        let n = wirksam(&self.root, d); let moeglich = d.zeilen.iter().any(|z| uebernehmbar(&self.root, z));
        let mut c = iced::widget::column![text(t::import_titel()).size(16).color(pal.text), text(t::import_hinweis()).size(12).color(pal.mute), liste].spacing(10);
        if mehr > 0 { c = c.push(text(vc_i18n::t!("importdlg-und-mehr-weitere-datei-en-die", mehr = &mehr)).size(11.5).color(pal.mute)) }
        c = c.push(iced::widget::row![knopf(pal, &vc_i18n::t!("importdlg-n-uebernehmen", n = &n), (n > 0).then_some(Msg::ImportOk), pal.text, true), knopf(pal, &t::baum_abbruch(), Some(Msg::ImportAbbruch), pal.text, false)].spacing(8));
        if !moeglich { c = c.push(text(vc_i18n::t!("importdlg-nichts-zu-uebernehmen-alle-dateien-liegen")).size(11.5).color(pal.mute)) }
        let karte = container(c).width(900).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() }));
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
}
