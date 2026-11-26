// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Zeichensatz hinzufügen“ mit drei Reitern: **Standardzeichensatz** (Verweis auf dein ROM oder Kopie, dazu die eigenen Sätze VC-Standard), **aus ROM** (nur ROM-Dateien des Anwenders,
//! die er selbst wählt) und **aus PNG-Raster**. Die Arbeit machen `vc_media::project` und `vc_media::wizard`; hier stehen nur Auswahl und Ausführung.
use crate::setui::{abschnitt, klein};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZTab { Standard, Rom, Png }

#[derive(Clone, Debug)]
pub enum ZAkt { Tab(ZTab), Name(String), Std(usize), Einbetten(bool), RomWaehlen, RomGewaehlt(Option<PathBuf>), RomSatz(usize), PngWaehlen, PngGewaehlt(Option<PathBuf>), Profil(usize), Invert(bool), Ok, Abbruch }

/// Eintrag der Standard-Liste: Anzeige, Quelle (Verweis auf ein ROM oder ein eigener Satz) und Profil
#[derive(Clone, Debug)]
pub struct StdEintrag { pub text: String, pub verweis: Option<(String, String)>, pub eigen: Option<String>, pub vorhanden: bool }

pub struct ZeichenDlg {
    pub tab: ZTab, pub name: String, pub meldung: String,
    pub liste: Vec<StdEintrag>, pub std_wahl: usize, pub einbetten: bool,
    pub rom_pfad: Option<PathBuf>, pub erkennung: Option<vc_media::project::RomRecognition>, pub rom_satz: usize,
    pub png_pfad: Option<PathBuf>, pub profile: Vec<(String, String)>, pub profil: usize, pub invert: bool,
}

impl ZeichenDlg {
    fn neu(tab: ZTab, data: &std::path::Path, systeme: &[String]) -> ZeichenDlg {
        let passt = |s: &str| systeme.is_empty() || systeme.iter().any(|x| x == s || (x == "c16" && s == "plus4"));
        let mut liste: Vec<StdEintrag> = vec![];
        for c in vc_media::project::std_charset_list(data).into_iter().filter(|c| passt(&c.system)) { liste.push(StdEintrag { text: vc_i18n::t!("zeichendlg-p1-dein-rom", p1 = &c.label), vorhanden: c.status != vc_roms::Status::Missing, verweis: Some((c.reference, c.profile)), eigen: None }) }
        for e in vc_media::project::eigene_saetze(data, "").into_iter().filter(|e| passt(&e.system)) { liste.push(StdEintrag { text: vc_i18n::t!("zeichendlg-vc-standard-p1-eigen-cc0", p1 = &e.name), verweis: None, eigen: Some(e.id), vorhanden: true }) }
        let profile: Vec<(String, String)> = vc_media::load_profiles(data).unwrap_or_default().into_iter().filter(|p| p.kind == "charset" && passt(&p.target)).map(|p| (p.id.clone(), p.name.clone())).collect();
        ZeichenDlg { tab, name: String::new(), meldung: String::new(), liste, std_wahl: 0, einbetten: false, rom_pfad: None, erkennung: None, rom_satz: 0, png_pfad: None, profile, profil: 0, invert: false }
    }
    /// Warum „Hinzufügen“ nicht geht (None = es geht)
    pub fn problem(&self) -> Option<String> {
        let name_ok = !self.name.is_empty() && self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        match self.tab {
            ZTab::Standard => if self.liste.is_empty() { Some(vc_i18n::t!("zeichendlg-fuer-dieses-projekt-gibt-es-keinen").into()) } else { None },
            ZTab::Rom => if self.rom_pfad.is_none() { Some(vc_i18n::t!("zeichendlg-waehle-eine-rom-datei").into()) } else if self.erkennung.as_ref().is_none_or(|e| e.sets.is_empty()) { Some(vc_i18n::t!("zeichendlg-in-dieser-datei-wurde-kein-zeichensatz").into()) } else if !name_ok { Some(vc_i18n::t!("zeichendlg-name-nur-buchstaben-ziffern-und").into()) } else { None },
            ZTab::Png => if self.png_pfad.is_none() { Some(vc_i18n::t!("zeichendlg-waehle-ein-png-bild").into()) } else if self.profile.is_empty() { Some(vc_i18n::t!("zeichendlg-kein-zeichensatzprofil-fuer-dieses-projekt").into()) } else if !name_ok { Some(vc_i18n::t!("zeichendlg-name-nur-buchstaben-ziffern-und").into()) } else { None },
        }
    }
}

impl App {
    pub(crate) fn zeichen_oeffnen(&mut self, tab: ZTab) {
        if self.proj.pf.is_none() { self.log.push(vc_i18n::t!("zeichendlg-zeichensatz-hinzufuegen-erst-ein-projekt-oeffnen").into()); return }
        let systeme: Vec<String> = self.proj.pf.as_ref().map(|p| p.targets.iter().map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).collect()).unwrap_or_default();
        self.zeichen = Some(ZeichenDlg::neu(tab, &self.data, &systeme));
    }
    pub(crate) fn zeichen_akt(&mut self, a: ZAkt) -> Task<Msg> {
        let data = self.data.clone();
        let Some(d) = self.zeichen.as_mut() else { return Task::none() };
        match a {
            ZAkt::Tab(t) => { d.tab = t; d.meldung.clear() }
            ZAkt::Name(n) => d.name = n,
            ZAkt::Std(i) => d.std_wahl = i,
            ZAkt::Einbetten(b) => d.einbetten = b,
            ZAkt::RomWaehlen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title(vc_i18n::t!("zeichendlg-rom-datei-waehlen-nur-deine-eigenen")).pick_file().await.map(|f| f.path().to_path_buf()) }, |p| Msg::Zeichen(ZAkt::RomGewaehlt(p))),
            ZAkt::RomGewaehlt(p) => { if let Some(p) = p { d.erkennung = Some(vc_media::project::recognize_rom(&p)); if d.name.is_empty() { d.name = p.file_stem().map(|s| s.to_string_lossy().replace(['.', ' '], "_")).unwrap_or_default() } d.rom_satz = 0; d.rom_pfad = Some(p) } }
            ZAkt::RomSatz(i) => d.rom_satz = i,
            ZAkt::PngWaehlen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title(vc_i18n::t!("zeichendlg-png-raster-waehlen")).add_filter("PNG", &["png"]).pick_file().await.map(|f| f.path().to_path_buf()) }, |p| Msg::Zeichen(ZAkt::PngGewaehlt(p))),
            ZAkt::PngGewaehlt(p) => { if let Some(p) = p { if d.name.is_empty() { d.name = p.file_stem().map(|s| s.to_string_lossy().replace(['.', ' '], "_")).unwrap_or_default() } d.png_pfad = Some(p) } }
            ZAkt::Profil(i) => d.profil = i,
            ZAkt::Invert(b) => d.invert = b,
            ZAkt::Abbruch => self.zeichen = None,
            ZAkt::Ok => {
                if d.problem().is_some() { return Task::none() }
                let root = self.root.clone();
                let erg: Result<String, String> = match d.tab {
                    ZTab::Standard => { let e = d.liste[d.std_wahl.min(d.liste.len() - 1)].clone(); match (e.verweis, e.eigen) {
                        (Some((r, p)), _) => vc_media::project::add_std_charset(&root, &r, &p, "graphics", d.einbetten).map(|v| v["path"].as_str().unwrap_or("").to_string()),
                        (_, Some(id)) => vc_media::wizard::eigenen_satz_kopieren(&data, &root, &id), _ => Err(vc_i18n::t!("zeichendlg-nichts-gewaehlt").into()) } }
                    ZTab::Rom => { let (Some(f), Some(er)) = (d.rom_pfad.clone(), d.erkennung.clone()) else { return Task::none() }; let (_, prof, off) = er.sets[d.rom_satz.min(er.sets.len() - 1)].clone(); vc_media::project::import_rom(&root, &data, &f, &prof, off, &d.name, "graphics").map(|v| v["path"].as_str().unwrap_or("").to_string()) }
                    ZTab::Png => { let Some(f) = d.png_pfad.clone() else { return Task::none() }; let prof = d.profile[d.profil.min(d.profile.len() - 1)].0.clone(); vc_media::project::import_sheet(&root, &data, &f, &prof, d.invert, &d.name, "graphics").map(|v| v["path"].as_str().unwrap_or("").to_string()) }
                };
                match erg {
                    Ok(rel) => { self.log.push(vc_i18n::t!("zeichendlg-zeichensatz-hinzugefuegt-rel", rel = &rel)); self.zeichen = None; self.projekt_neu_laden(); self.oeffne(&rel) }
                    Err(e) => { if let Some(d) = &mut self.zeichen { d.meldung = e } }
                }
            }
        }
        Task::none()
    }

    pub(crate) fn zeichen_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.zeichen.as_ref()?; let z = |a: ZAkt| Msg::Zeichen(a);
        let seg = |name: &str, aktiv: bool, m: Msg| -> Element<Msg> { iced::widget::button(text(name.to_string()).size(12).color(pal.text)).padding([4, 10]).on_press(m).style(move |_, _| iced::widget::button::Style { background: Some(if aktiv { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() };
        let tabs = row![seg(&vc_i18n::t!("zeichendlg-standardzeichensatz"), d.tab == ZTab::Standard, z(ZAkt::Tab(ZTab::Standard))), seg(&vc_i18n::t!("zeichendlg-aus-rom"), d.tab == ZTab::Rom, z(ZAkt::Tab(ZTab::Rom))), seg(&vc_i18n::t!("zeichendlg-aus-png-raster"), d.tab == ZTab::Png, z(ZAkt::Tab(ZTab::Png)))].spacing(6);
        let name = row![text(vc_i18n::t!("zeichendlg-name")).size(11.5).color(pal.mute), text_input("name", &d.name).on_input(move |n| Msg::Zeichen(ZAkt::Name(n))).font(vc_gui_ui::fonts::MONO).size(13).padding(5).width(200)].spacing(8).align_y(Alignment::Center);
        let koerper: Element<Msg> = match d.tab {
            ZTab::Standard => {
                let mut l = column![].spacing(3);
                for (i, e) in d.liste.iter().enumerate() { l = l.push(seg(&format!("{}{}", e.text, if e.vorhanden { String::new() } else { format!("  {}", vc_i18n::t!("zeichendlg-rom-fehlt-verweis-trotzdem-moeglich")) }), d.std_wahl == i, z(ZAkt::Std(i)))) }
                column![klein(pal, vc_i18n::t!("zeichendlg-ein-verweis-nimmt-die-schrift-aus")), container(scrollable(l).height(Length::Fixed(190.0))).padding(4), checkbox(d.einbetten).label(vc_i18n::t!("zeichendlg-zeichen-stattdessen-einbetten-nur-rom-saetze")).on_toggle(move |b| Msg::Zeichen(ZAkt::Einbetten(b))).size(16).text_size(12)].spacing(8).into()
            }
            ZTab::Rom => {
                let mut c = column![klein(pal, vc_i18n::t!("zeichendlg-liest-den-zeichensatz-aus-einer-rom")), row![text(d.rom_pfad.as_ref().map(|p| vc_ide_model::recent::pretty(p)).unwrap_or_else(|| vc_i18n::t!("zeichendlg-keine-datei-gewaehlt").into())).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text).width(Length::Fill), knopf(pal, vc_i18n::t!("zeichendlg-waehlen"), Some(z(ZAkt::RomWaehlen)), pal.text, false)].spacing(8).align_y(Alignment::Center)].spacing(8);
                if let Some(e) = &d.erkennung { c = c.push(text(match &e.label { Some(l) => vc_i18n::t!("zeichendlg-erkannt-l", l = &l), None => vc_i18n::t!("zeichendlg-nicht-erkannt-vorschlaege-nach-dateigroesse").to_string() }).size(11.5).color(pal.mute)); for (i, (beschr, prof, off)) in e.sets.iter().enumerate() { c = c.push(seg(&vc_i18n::t!("zeichendlg-beschr-prof-ab-off", beschr = &beschr, prof = &prof, off = format!("{:04X}", off)), d.rom_satz == i, z(ZAkt::RomSatz(i)))) } }
                c.into()
            }
            ZTab::Png => {
                let mut c = column![klein(pal, vc_i18n::t!("zeichendlg-16-zeichen-je-zeile-in-leserichtung")), row![text(d.png_pfad.as_ref().map(|p| vc_ide_model::recent::pretty(p)).unwrap_or_else(|| vc_i18n::t!("zeichendlg-keine-datei-gewaehlt").into())).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text).width(Length::Fill), knopf(pal, vc_i18n::t!("zeichendlg-waehlen"), Some(z(ZAkt::PngWaehlen)), pal.text, false)].spacing(8).align_y(Alignment::Center)].spacing(8);
                let p = row(d.profile.iter().enumerate().map(|(i, (_, n))| seg(n, d.profil == i, z(ZAkt::Profil(i)))).collect::<Vec<_>>()).spacing(4).wrap();
                c = c.push(text(vc_i18n::t!("zeichendlg-profil")).size(11.5).color(pal.mute)).push(p).push(checkbox(d.invert).label(vc_i18n::t!("zeichendlg-dunkle-pixel-sind-gesetzt-statt-heller")).on_toggle(move |b| Msg::Zeichen(ZAkt::Invert(b))).size(16).text_size(12));
                c.into()
            }
        };
        let problem = d.problem();
        let mut c = column![abschnitt(pal, vc_i18n::t!("zeichendlg-zeichensatz-hinzufuegen"), vc_i18n::t!("zeichendlg-legt-eine-zeichensatzdatei-im-ordner-graphics")), tabs].spacing(10);
        if d.tab != ZTab::Standard { c = c.push(name) }
        c = c.push(koerper);
        if let Some(p) = &problem { c = c.push(text(p.clone()).size(11.5).color(pal.warn)) }
        if !d.meldung.is_empty() { c = c.push(text(d.meldung.clone()).size(11.5).color(pal.err)) }
        c = c.push(row![iced::widget::space::horizontal(), knopf(pal, vc_i18n::t!("zeichendlg-abbrechen"), Some(z(ZAkt::Abbruch)), pal.text, false), knopf(pal, vc_i18n::t!("zeichendlg-hinzufuegen"), problem.is_none().then(|| z(ZAkt::Ok)), pal.text, true)].spacing(8));
        let karte = container(c).width(620).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() }));
        Some(iced::widget::stack![hg, container(karte).center(Length::Fill)].into())
    }
}
