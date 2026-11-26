// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Projekt kopieren …“ (Stufe 4b): neuer Name, Ort und was mitkopiert wird; das Kopieren läuft im Hintergrund
//! (`vc_ide::copyproject`), die Oberfläche bleibt bedienbar. Beispielprojekte lassen sich so gefahrlos ausprobieren.
use crate::setui::{abschnitt, klein};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{checkbox, column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::path::{Path, PathBuf};
use vc_ide::copyproject::{self, Options, Report};

#[derive(Clone, Debug)]
pub enum KopAkt { Name(String), Ort, OrtGewaehlt(Option<PathBuf>), Build(bool), Git(bool), Danach(bool), Ok, Fertig(Result<Report, String>, bool), Abbruch }

pub struct KopierDlg { pub quelle: PathBuf, pub name: String, pub ort: PathBuf, pub mit_build: bool, pub mit_git: bool, pub danach: bool, pub fehler: Option<String>, pub info: Option<(usize, u64)>, pub laeuft: bool }

/// Erster freier Name „<Name> Kopie“, „<Name> Kopie 2“ … neben dem Quellordner
pub fn vorschlag(quelle: &Path, ort: &Path) -> String {
    let basis = quelle.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| vc_i18n::t!("kopierdlg-projekt").into());
    let erst = vc_i18n::t!("kopierdlg-basis-kopie", basis = &basis); if !ort.join(&erst).exists() { return erst }
    (2..1000).map(|i| vc_i18n::t!("kopierdlg-erst-i", erst = &erst, i = &i)).find(|n| !ort.join(n).exists()).unwrap_or(erst)
}
pub fn menschlich(b: u64) -> String { if b < 10_000 { vc_i18n::t!("kopierdlg-b-byte", b = &b) } else if b < 10_000_000 { vc_i18n::t!("kopierdlg-p1-kb", p1 = vc_i18n::num((b as f64 / 1000.0) as f64, 0)) } else { vc_i18n::t!("kopierdlg-p1-mb", p1 = vc_i18n::num((b as f64 / 1_000_000.0) as f64, 1)) } }

impl KopierDlg {
    pub fn neu(quelle: PathBuf) -> KopierDlg { let ort = quelle.parent().map(Path::to_path_buf).unwrap_or_default(); let name = vorschlag(&quelle, &ort); let mut d = KopierDlg { quelle, name, ort, mit_build: false, mit_git: false, danach: true, fehler: None, info: None, laeuft: false }; d.berechnen(); d }
    pub fn optionen(&self) -> Options { Options { with_build: self.mit_build, with_git: self.mit_git } }
    pub fn berechnen(&mut self) { self.info = copyproject::plan(&self.quelle, &self.optionen()).ok() }
    /// Warum „Kopieren“ noch nicht geht (None = es geht)
    pub fn problem(&self) -> Option<String> {
        if let Err(e) = copyproject::validate_name(&self.name) { return Some(e) }
        let ziel = self.ort.join(self.name.trim());
        if ziel.exists() && !(ziel.is_dir() && std::fs::read_dir(&ziel).is_ok_and(|mut d| d.next().is_none())) { return Some(vc_i18n::t!("kopierdlg-p1-gibt-es-schon", p1 = vc_ide_model::recent::pretty(&ziel))) }
        if ziel.starts_with(&self.quelle) { return Some(vc_i18n::t!("kopierdlg-das-ziel-laege-im-quellordner").into()) }
        None
    }
}

impl App {
    pub(crate) fn kopieren_oeffnen(&mut self) { if self.proj.pf.is_none() { self.log.push(vc_i18n::t!("kopierdlg-projekt-kopieren-erst-ein-projekt-oeffnen").into()); return } self.kopier = Some(KopierDlg::neu(self.root.clone())) }
    pub(crate) fn kop_akt(&mut self, a: KopAkt) -> Task<Msg> {
        let Some(d) = self.kopier.as_mut() else { return Task::none() };
        match a {
            KopAkt::Name(n) => { d.name = n; d.fehler = None }
            KopAkt::Ort => { let start = d.ort.clone(); return Task::perform(async move { rfd::AsyncFileDialog::new().set_directory(start).pick_folder().await.map(|f| f.path().to_path_buf()) }, |p| Msg::Kopieren(KopAkt::OrtGewaehlt(p))) }
            KopAkt::OrtGewaehlt(p) => { if let Some(p) = p { d.ort = p; d.fehler = None } }
            KopAkt::Build(b) => { d.mit_build = b; d.berechnen() }
            KopAkt::Git(b) => { d.mit_git = b; d.berechnen() }
            KopAkt::Danach(b) => d.danach = b,
            KopAkt::Abbruch => { if !d.laeuft { self.kopier = None } }
            KopAkt::Ok => {
                if d.problem().is_some() || d.laeuft { return Task::none() }
                d.laeuft = true; d.fehler = None;
                let (q, o, n, op, danach) = (d.quelle.clone(), d.ort.clone(), d.name.trim().to_string(), d.optionen(), d.danach);
                return Task::perform(async move { tokio::task::spawn_blocking(move || copyproject::copy_project(&q, &o, &n, &op)).await.unwrap_or_else(|e| Err(e.to_string())) }, move |r| Msg::Kopieren(KopAkt::Fertig(r, danach)))
            }
            KopAkt::Fertig(r, danach) => match r {
                Ok(rep) => { self.kopier = None; self.log.push(vc_i18n::t!("kopierdlg-projekt-kopiert-p1-p2-dateien-p3", p1 = rep.new_root.display(), p2 = &rep.files, p3 = menschlich(rep.bytes))); for s in rep.skipped.iter().take(5) { self.log.push(vc_i18n::t!("kopierdlg-uebersprungen-s", s = &s)) } if danach { return self.update_innen(Msg::Geoeffnet(Some(rep.new_root))) } }
                Err(e) => { d.laeuft = false; d.fehler = Some(e) }
            },
        }
        Task::none()
    }
    pub(crate) fn kopier_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.kopier.as_ref()?; let k = |a: KopAkt| Msg::Kopieren(a);
        let problem = d.problem();
        let mut c = column![abschnitt(pal, vc_i18n::t!("kopierdlg-projekt-kopieren"), &vc_i18n::t!("kopierdlg-quelle-p1", p1 = vc_ide_model::recent::pretty(&d.quelle))), text(vc_i18n::t!("kopierdlg-name-der-kopie")).size(11.5).color(pal.mute),
            text_input(&vc_i18n::t!("kopierdlg-name"), &d.name).on_input(move |n| Msg::Kopieren(KopAkt::Name(n))).font(vc_gui_ui::fonts::MONO).size(13).padding(6), text(vc_i18n::t!("kopierdlg-speicherort")).size(11.5).color(pal.mute),
            row![text(vc_ide_model::recent::pretty(&d.ort)).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text).width(Length::Fill), knopf(pal, vc_i18n::t!("kopierdlg-waehlen"), (!d.laeuft).then(|| k(KopAkt::Ort)), pal.text, false)].spacing(8).align_y(Alignment::Center),
            checkbox(d.mit_build).label(vc_i18n::t!("kopierdlg-bauordner-build-mitkopieren")).on_toggle(move |b| Msg::Kopieren(KopAkt::Build(b))).size(16).text_size(13),
            checkbox(d.mit_git).label(vc_i18n::t!("kopierdlg-versionsgeschichte-git-mitkopieren")).on_toggle(move |b| Msg::Kopieren(KopAkt::Git(b))).size(16).text_size(13),
            checkbox(d.danach).label(vc_i18n::t!("kopierdlg-danach-die-kopie-oeffnen")).on_toggle(move |b| Msg::Kopieren(KopAkt::Danach(b))).size(16).text_size(13)].spacing(8);
        if let Some((n, b)) = d.info { c = c.push(klein(pal, vc_i18n::t!("kopierdlg-n-dateien-p2-werden-kopiert", n = &n, p2 = menschlich(b)))) }
        if let Some(p) = &problem { c = c.push(text(p.clone()).size(11.5).color(pal.err)) }
        if let Some(e) = &d.fehler { c = c.push(text(e.clone()).size(11.5).color(pal.err)) }
        c = c.push(row![if d.laeuft { text(vc_i18n::t!("kopierdlg-kopiert")).size(13).color(pal.mute) } else { text("").size(13) }, iced::widget::space::horizontal(), knopf(pal, vc_i18n::t!("kopierdlg-abbrechen"), (!d.laeuft).then(|| k(KopAkt::Abbruch)), pal.text, false), knopf(pal, vc_i18n::t!("kopierdlg-kopieren"), (problem.is_none() && !d.laeuft).then(|| k(KopAkt::Ok)), pal.text, true)].spacing(8).align_y(Alignment::Center));
        let karte = container(c).width(560).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() }));
        Some(iced::widget::stack![hg, container(karte).center(Length::Fill)].into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn namensvorschlag_nimmt_den_ersten_freien_namen() {
        let basis = std::env::temp_dir().join(format!("vcgui-kopier-{}", std::process::id())); let _ = std::fs::remove_dir_all(&basis); std::fs::create_dir_all(basis.join("Spiel")).unwrap(); let q = basis.join("Spiel");
        assert_eq!(vorschlag(&q, &basis), "Spiel Kopie"); std::fs::create_dir_all(basis.join("Spiel Kopie")).unwrap(); assert_eq!(vorschlag(&q, &basis), "Spiel Kopie 2"); let _ = std::fs::remove_dir_all(&basis);
    }
    #[test] fn groessen_lesbar() { assert_eq!(menschlich(512), "512 Byte"); assert_eq!(menschlich(250_000), "250 KB"); assert_eq!(menschlich(12_500_000), "12,5 MB", "Zahlenformat der Sprache: Komma im Deutschen"); vc_i18n::set_thread_language(Some(vc_i18n::Lang::En)); assert_eq!(menschlich(12_500_000), "12.5 MB"); vc_i18n::set_thread_language(None); }
}
