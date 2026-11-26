// SPDX-License-Identifier: GPL-3.0-or-later
//! Startseite (Stufe 4c, Mockup 3): Neu/Öffnen/Beispiel, zuletzt geöffnete Projekte (anheften, entfernen), erkannte Emulatoren und Hardware,
//! Einstellungen/Assistent/Feedback, Studios und Schnellreferenz. Sie erscheint im Dokumentenfeld, wenn nichts offen ist, und als Reiter „Start“
//! (Befehl „Startseite“). Es werden nur echte Werte gezeigt: Emulatoren werden gesucht (kein Netz), Hardware gilt als „ungeprüft“, bis getestet wird.
//! Der Hardware-Test läuft nur über `aussen`.
use crate::{aussen, flaeche, knopf, App, Msg, Pal};
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vc_ide::platform::{self, Presence};
use vc_ide::targets::{kind_label, EmulatorCfg};
use vc_ide_model::recent::{self, Recent};
use vc_ide::setup;

#[derive(Clone, Debug)]
pub enum StartAkt { Neu, Beispiele, Pin(usize), Weg(usize), Testen(String), ProbeErg(String, Result<String, String>), Hilfe(&'static str), StudioNeu(crate::NeuArt), Kopieren(PathBuf), Assistent, BannerWeg, Feedback }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hinweis { Unvollstaendig, Angebot }

#[derive(Clone, Debug, PartialEq)]
pub enum Probe { Laeuft, Ok, Fehler(String) }

#[derive(Default)]
pub struct StartZ {
    pub banner: Option<Hinweis>, pub recent: Vec<Recent>, pub beispiele_offen: bool, pub proben: HashMap<String, Probe>,
    pub werkzeuge: RefCell<Option<(Instant, Vec<(&'static str, Presence)>)>>,
}

fn s(a: StartAkt) -> Msg { Msg::StartSeite(a) }
fn chip<'a>(c: Color, t: impl Into<String>) -> Element<'a, Msg> { let t = t.into(); container(text(t).size(11.5).color(c)).padding([2, 8]).style(move |_| container::Style { border: Border { color: c, width: 1.0, radius: 10.0.into() }, background: Some(Color { a: 0.12, ..c }.into()), ..Default::default() }).into() }
fn tafel<'a>(pal: Pal, inhalt: Element<'a, Msg>) -> Element<'a, Msg> { container(inhalt).padding(14).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into() }
fn klein<'a>(pal: Pal, t: impl Into<String>) -> Element<'a, Msg> { text(t.into()).size(11.5).color(pal.mute).into() }
fn titel<'a>(pal: Pal, t: impl AsRef<str>) -> Element<'a, Msg> { text(t.as_ref().to_string()).size(15).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into() }

impl App {
    /// Zuletzt geöffnete Projekte einlesen (nie im Testlauf; dort bleibt die Liste leer, bis ein Test sie füllt)
    pub(crate) fn start_laden(&mut self) { if !cfg!(test) { self.start.recent = recent::load(Some(&self.data)); self.start.banner = self.hinweis_ermitteln() } }
    /// Hinweis oben auf der Startseite: Einrichtung unvollständig, oder einmal „Neu: Konfigurationsassistent“ bei bestehenden Einstellungen
    pub(crate) fn hinweis_ermitteln(&self) -> Option<Hinweis> {
        let paths = self.wiz_pfade()?; let st = setup::WizardState::load(&paths.wizard); if st.dismissed.iter().any(|d| d == "banner") { return None }
        if st.incomplete() { Some(Hinweis::Unvollstaendig) } else if setup::start_state(&paths) == setup::Start::OfferOnce { Some(Hinweis::Angebot) } else { None }
    }

    fn beispiele(&self) -> Vec<(String, PathBuf)> {
        let Some(dir) = self.data.parent().map(|p| p.join("examples")).filter(|p| p.is_dir()) else { return vec![] };
        let mut v = vec![];
        for sys in std::fs::read_dir(&dir).into_iter().flatten().flatten().filter(|e| e.path().is_dir()) {
            for p in std::fs::read_dir(sys.path()).into_iter().flatten().flatten().filter(|e| e.path().is_dir()) {
                if std::fs::read_dir(p.path()).into_iter().flatten().flatten().any(|f| f.path().extension().is_some_and(|x| x == "vcprj")) { v.push((format!("{} / {}", sys.file_name().to_string_lossy(), p.file_name().to_string_lossy()), p.path())) }
            }
        }
        v.sort(); v
    }

    pub(crate) fn werkzeuge(&self) -> Vec<(&'static str, Presence)> {
        if let Some((t, v)) = &*self.start.werkzeuge.borrow() { if t.elapsed() < Duration::from_secs(5) { return v.clone() } }
        let (mut gesehen, mut out): (Vec<&'static str>, Vec<(&'static str, Presence)>) = (vec![], vec![]);
        let set = &self.satz.gespeichert;
        for sys in vc_ide::targets::installed_systems(Some(&self.data)) { for k in sys.kinds() {
            if gesehen.contains(&k.id) || !vc_ide::emulators::enabled(k.id) { continue }
            gesehen.push(k.id);
            let pfad = set.target(sys.id).and_then(|t| t.emulators.iter().find(|e| e.kind == k.id)).map(|e| e.path.clone()).unwrap_or_default();
            out.push((k.id, platform::presence(&EmulatorCfg { id: k.id.into(), kind: k.id.into(), path: pfad, ..Default::default() })));
        } }
        *self.start.werkzeuge.borrow_mut() = Some((Instant::now(), out.clone())); out
    }

    pub(crate) fn start_akt(&mut self, a: StartAkt) -> Task<Msg> {
        match a {
            StartAkt::Neu => self.neuprojekt_oeffnen(),
            StartAkt::Beispiele => self.start.beispiele_offen = !self.start.beispiele_offen,
            StartAkt::Pin(i) => recent::toggle_pin(&mut self.start.recent, i),
            StartAkt::Weg(i) => recent::remove(&mut self.start.recent, i),
            StartAkt::Testen(id) => {
                if let Some(m) = self.satz.gespeichert.machine(&id).cloned() {
                    self.start.proben.insert(id.clone(), Probe::Laeuft);
                    return Task::perform(async move { tokio::task::spawn_blocking(move || aussen::hardware_test(&m, None)).await.unwrap_or_else(|e| Err(e.to_string())) }, move |r| Msg::StartSeite(StartAkt::ProbeErg(id.clone(), r)));
                }
            }
            StartAkt::ProbeErg(id, r) => { self.start.proben.insert(id, match r { Ok(_) => Probe::Ok, Err(e) => Probe::Fehler(e) }); }
            StartAkt::Hilfe(id) => { let _ = self.update_innen(Msg::PanelWahl(crate::PanelSeite::Hilfe)); return self.update_innen(Msg::HilfeOeffne(id.to_string())) }
            StartAkt::StudioNeu(a) => { let o = self.ziel.take().unwrap_or_default(); return self.update_innen(Msg::BaumNeu(o, a)) }
            StartAkt::Kopieren(p) => self.kopier = Some(crate::kopierdlg::KopierDlg::neu(p)),
            StartAkt::Assistent => self.wiz_oeffnen(None),
            StartAkt::BannerWeg => { if let Some(p) = self.wiz_pfade() { let mut st = setup::WizardState::load(&p.wizard); st.dismiss("banner"); let _ = st.save(&p.wizard); } self.start.banner = None }
            StartAkt::Feedback => return self.info_akt(crate::infodlg::InfoAkt::FeedbackAuf),
        }
        Task::none()
    }

    pub(crate) fn startseite(&self, pal: Pal) -> Element<'_, Msg> {
        let st = &self.start;
        let hat_projekt = !self.root.as_os_str().is_empty() && self.proj.pf.is_some();
        // Kopf mit Hauptaktionen
        let beispiele = self.beispiele();
        let kopf = tafel(pal, column![
            row![image(self.logos[2].clone()).height(54), column![row![text(vc_i18n::t!("startui-vintagecoder")).size(20).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), chip(pal.accent, concat!("v", env!("CARGO_PKG_VERSION")))].spacing(10).align_y(Alignment::Center).wrap(), klein(pal, vc_i18n::t!("startui-entwicklungsumgebung-fuer-die-6502-familie-commodore"))].spacing(4)].spacing(14).align_y(Alignment::Center),
            row![knopf(pal, vc_i18n::t!("startui-neues-projekt"), Some(s(StartAkt::Neu)), pal.text, true), knopf(pal, vc_i18n::t!("startui-projekt-oeffnen"), Some(Msg::Befehl(vc_gui_ui::befehle::Cmd::ProjektOeffnen)), pal.text, false), knopf(pal, vc_i18n::t!("startui-beispiel-oeffnen"), (!beispiele.is_empty()).then(|| s(StartAkt::Beispiele)), pal.text, false), knopf(pal, vc_i18n::t!("startui-einrichten"), Some(Msg::Befehl(vc_gui_ui::befehle::Cmd::Einstellungen)), pal.text, false)].spacing(8).wrap().vertical_spacing(8),
        ].spacing(12).into());
        let mut links = column![].spacing(12);
        if let Some(h) = &st.banner { let (t, b) = match h { Hinweis::Unvollstaendig => (vc_i18n::t!("startui-einrichtung-unvollstaendig-ein-paar-schritte-sind"), vc_i18n::t!("startui-fertig-einrichten")), Hinweis::Angebot => (vc_i18n::t!("startui-neu-konfigurationsassistent-er-prueft-deine-einrichtung"), vc_i18n::t!("startui-assistent-oeffnen")) }; links = links.push(tafel(pal, row![text(t).size(13).color(pal.text), iced::widget::space::horizontal(), knopf(pal, b, Some(s(StartAkt::Assistent)), pal.text, true), knopf(pal, "✕", Some(s(StartAkt::BannerWeg)), pal.text, false)].spacing(8).align_y(Alignment::Center).wrap().into())) }
        links = links.push(kopf);
        if st.beispiele_offen { links = links.push(tafel(pal, column![titel(pal, vc_i18n::t!("startui-beispielprojekte")), column(beispiele.into_iter().map(|(n, p)| button(text(n).size(13).color(pal.text)).width(Length::Fill).padding([5, 8]).on_press(Msg::Geoeffnet(Some(p))).style(move |_, h| button::Style { background: matches!(h, button::Status::Hovered).then(|| pal.sel.into()), border: Border { radius: 5.0.into(), ..Default::default() }, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>()).spacing(2)].spacing(6).into())) }
        // Zuletzt geöffnet
        let jetzt = recent::now();
        let mut liste = column![row![titel(pal, vc_i18n::t!("startui-zuletzt-geoeffnete-projekte")), chip(pal.mute, st.recent.len().to_string())].spacing(8).align_y(Alignment::Center)].spacing(8);
        if st.recent.is_empty() { liste = liste.push(klein(pal, vc_i18n::t!("startui-noch-kein-projekt-geoeffnet-neues-projekt"))) }
        for (i, r) in st.recent.iter().enumerate() {
            let mut marken = row![].spacing(6);
            if r.missing { marken = marken.push(chip(pal.err, vc_i18n::t!("startui-ordner-fehlt"))) }
            for (sy, v) in &r.info.systems { marken = marken.push(chip(pal.accent, if v.is_empty() { recent::short_name(sy).to_string() } else { format!("{} {}", recent::short_name(sy), v.to_uppercase()) })) }
            for l in &r.info.langs { marken = marken.push(chip(pal.warn, l.clone())) }
            if let Some(e) = &r.info.emulator { marken = marken.push(chip(pal.mute, e.clone())) }
            let name = button(text(r.name.clone()).size(14).font(vc_gui_ui::fonts::UI_FETT).color(if r.missing { pal.mute } else { pal.text })).padding([0, 0]).on_press_maybe((!r.missing).then(|| Msg::Geoeffnet(Some(r.path.clone())))).style(|_, _| button::Style::default());
            let aktion = row![text(recent::ago(r.last, jetzt)).size(11.5).color(pal.mute), knopf(pal, if r.pinned { "★" } else { "☆" }, Some(s(StartAkt::Pin(i))), pal.text, false), knopf(pal, "×", Some(s(StartAkt::Weg(i))), pal.text, false), knopf(pal, vc_i18n::t!("startui-kopieren"), (!r.missing).then(|| s(StartAkt::Kopieren(r.path.clone()))), pal.text, false), knopf(pal, vc_i18n::t!("startui-oeffnen"), (!r.missing).then(|| Msg::Geoeffnet(Some(r.path.clone()))), pal.text, false)].spacing(6).align_y(Alignment::Center).wrap();
            let bild: Option<Element<Msg>> = (!r.missing && !r.info.no_preview && r.path.join("projektbild.png").is_file()).then(|| image(image::Handle::from_path(r.path.join("projektbild.png"))).height(72).into());
            let kopf_zeile: Element<Msg> = row![name, iced::widget::space::horizontal(), aktion].spacing(8).align_y(Alignment::Center).wrap().into();
            let text_spalte = column![kopf_zeile, marken.wrap().vertical_spacing(4), text(recent::pretty(&r.path)).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute)].spacing(5).width(Length::Fill);
            liste = liste.push(tafel(pal, match bild { Some(b) => row![b, text_spalte].spacing(12).align_y(Alignment::Center).into(), None => text_spalte.into() }));
        }
        links = links.push(liste);
        // Studios
        let arten = self.studio_arten();
        if !arten.is_empty() {
            links = links.push(titel(pal, vc_i18n::t!("startui-studios"))).push(klein(pal, if hat_projekt { vc_i18n::t!("startui-legt-eine-neue-datei-im-geoeffneten") } else { vc_i18n::t!("startui-studios-arbeiten-auf-dateien-eines-projekts") }));
            links = links.push(row(arten.into_iter().map(|a| { let n = crate::NeuArt::Studio(a); knopf(pal, &n.name(), hat_projekt.then(|| s(StartAkt::StudioNeu(n))), pal.text, false) }).collect::<Vec<_>>()).spacing(8).wrap().vertical_spacing(8));
        }
        // rechts: Toolchain und Hardware, Referenz
        let mut tc = column![titel(pal, vc_i18n::t!("startui-toolchain-und-hardware")), text(vc_i18n::t!("startui-erkannte-emulatoren")).size(10.5).color(pal.mute)].spacing(5);
        for (k, p) in self.werkzeuge() {
            tc = tc.push(row![text("●").size(11).color(if p.found { pal.ok } else { pal.mute }), text(kind_label(k)).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text), iced::widget::space::horizontal(), if p.found { chip(pal.ok, vc_i18n::t!("startui-bereit")) } else if k == "altirra" && !cfg!(windows) { chip(pal.mute, vc_i18n::t!("startui-nur-windows")) } else { chip(pal.mute, vc_i18n::t!("startui-nicht-gefunden")) }].spacing(6).align_y(Alignment::Center));
        }
        tc = tc.push(text(vc_i18n::t!("startui-echte-hardware")).size(10.5).color(pal.mute));
        let ms = &self.satz.gespeichert.machines;
        if ms.is_empty() { tc = tc.push(klein(pal, vc_i18n::t!("startui-keine-hardware-eingetragen"))) }
        for m in ms {
            let p = st.proben.get(&m.id); let adr = if m.kind == "ultimate64" { m.host.clone() } else { m.device.clone() };
            let (farbe, c) = match p { Some(Probe::Ok) => (pal.ok, chip(pal.ok, "online")), Some(Probe::Fehler(_)) => (pal.err, chip(pal.err, vc_i18n::t!("startui-nicht-erreichbar"))), Some(Probe::Laeuft) => (pal.warn, chip(pal.warn, vc_i18n::t!("startui-pruefe"))), None => (pal.mute, chip(pal.mute, vc_i18n::t!("startui-ungeprueft"))) };
            tc = tc.push(row![text("●").size(11).color(farbe), text(format!("{} · {}", m.display_name(), if adr.is_empty() { "–".into() } else { adr })).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text), iced::widget::space::horizontal(), c, knopf(pal, vc_i18n::t!("startui-testen"), (p != Some(&Probe::Laeuft)).then(|| s(StartAkt::Testen(m.id.clone()))), pal.text, false)].spacing(6).align_y(Alignment::Center).wrap());
            if let Some(Probe::Fehler(e)) = p { tc = tc.push(text(e.clone()).size(11.5).color(pal.err)) }
        }
        tc = tc.push(row![knopf(pal, vc_i18n::t!("startui-einstellungen-oeffnen"), Some(Msg::Befehl(vc_gui_ui::befehle::Cmd::Einstellungen)), pal.text, false), knopf(pal, vc_i18n::t!("startui-konfigurationsassistent"), Some(s(StartAkt::Assistent)), pal.text, false), knopf(pal, vc_i18n::t!("startui-feedback-senden"), Some(s(StartAkt::Feedback)), pal.text, false)].spacing(6).wrap().vertical_spacing(6));
        let mut ref_ = column![titel(pal, vc_i18n::t!("startui-schnellreferenz"))].spacing(4);
        for (id, t) in [("start", vc_i18n::t!("startui-ueberblick")), ("settings", vc_i18n::t!("startui-einstellungen")), ("assembler", vc_i18n::t!("startui-assembler")), ("basic", "BASIC".to_string()), ("debugger", vc_i18n::t!("startui-debugger")), ("disks", vc_i18n::t!("startui-disk-studio")), ("shortcuts", vc_i18n::t!("startui-tastenkuerzel"))] {
            if !vc_ide::help::visible(id) { continue }
            ref_ = ref_.push(button(text(t).size(13).color(pal.accent)).padding([1, 0]).on_press(s(StartAkt::Hilfe(id))).style(|_, _| button::Style::default()));
        }
        let rechts = column![tafel(pal, tc.into()), tafel(pal, ref_.into())].spacing(12);
        let breit = self.fenster.width / self.einst.zoom >= 1100.0;
        let inhalt: Element<Msg> = if breit { row![container(links).width(Length::FillPortion(3)), container(rechts).width(Length::FillPortion(2))].spacing(16).into() } else { column![links, rechts].spacing(16).into() };
        scrollable(container(inhalt).padding(20)).height(Length::Fill).into()
    }
}
