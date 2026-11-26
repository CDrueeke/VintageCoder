// SPDX-License-Identifier: GPL-3.0-or-later
//! Über, Feedback und Update (Stufe 4d). Über zeigt Name, Version, Lizenz, Urheber, Links und die Drittanbieter-Hinweise; es ruft von selbst nichts aus dem Netz ab.
//! Feedback bereitet eine Mail vor (`vc_ide::feedback`): gesendet wird erst im Mailprogramm. Update fragt nur auf Wunsch (oder nach der gewählten Häufigkeit) die
//! Versionsliste ab, lädt und installiert nichts; der Link zur Releaseseite ist im Programm fest. Alles Äußere (Browser, Mail, Netz) läuft über `aussen`.
use crate::setui::{abschnitt, karte, klein, mono, SetAkt};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{checkbox, column, container, image, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Task};
use vc_ide::feedback::{self, Mail};
use vc_ide::update::{self, Evaluation, Report, Version};

pub const WEBSITE: &str = "https://vintagecoder.drueeke.de";
pub const REPOSITORY: &str = "https://github.com/CDrueeke/VintageCoder";
pub const LIZENZ: &str = "GPL-3.0-or-later";
pub const HINWEISE: &str = include_str!("../../../THIRD-PARTY-NOTICES.md");

#[derive(Clone, Debug)]
pub enum InfoAkt { UeberAuf, UeberZu, FeedbackAuf, FeedbackZu, Mail, Kopieren, DebugLog(bool), Pruefen, Ergebnis(Box<Report>), Ansehen, Spaeter, Ueberspringen, UpdateZu, UpdateAuf, Hinweise, Herkunft, Web(&'static str) }

#[derive(Default)]
pub struct InfoZ { pub ueber: bool, pub hinweise: bool, pub herkunft: bool, pub feedback: Option<(Mail, Option<(bool, String)>)>, pub update_offen: bool, pub eval: Option<Evaluation>, pub laeuft: bool, pub stand: update::State, pub letzte: Option<Report> }

pub fn version() -> &'static str { env!("CARGO_PKG_VERSION") }

fn i(a: InfoAkt) -> Msg { Msg::Info(a) }
fn modal<'a>(pal: Pal, inhalt: Element<'a, Msg>, breite: f32, zu: Msg) -> Element<'a, Msg> {
    let karte = container(inhalt).width(breite).max_height(720).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
    let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() })).on_press(zu);
    iced::widget::stack![hg, container(karte).center(Length::Fill)].into()
}

impl App {
    /// Zustand der Update-Prüfung beim Start lesen: gemerkte Liste, Zeitpunkte; und, wenn fällig und erlaubt, im Hintergrund prüfen
    pub(crate) fn update_start(&mut self) -> Task<Msg> {
        if cfg!(test) { return Task::none() }
        self.upd.stand = update::State::load();
        if !vc_ide::features::features().update() { return Task::none() }
        self.upd.eval = update::cached(Version::installed());
        let prefs = self.satz.gespeichert.updates.clone();
        if update::auto_check_allowed(&vc_ide::features::features(), &prefs, &self.upd.stand, update::now_secs()) { return self.update_pruefen() }
        Task::none()
    }
    fn update_pruefen(&mut self) -> Task<Msg> {
        if self.upd.laeuft { return Task::none() }
        self.upd.laeuft = true; let src = update::source();
        Task::perform(async move { tokio::task::spawn_blocking(move || aussen::update_pruefen(&src, Version::installed())).await.ok() }, |r| match r { Some(r) => Msg::Info(InfoAkt::Ergebnis(Box::new(r))), None => Msg::Info(InfoAkt::UeberZu) })
    }
    /// Hinweis in der Statusleiste: neue Version, nicht übersprungen und nicht in der Ruhezeit
    pub(crate) fn update_hinweis(&self) -> Option<Version> {
        let ev = self.upd.eval.as_ref()?; if !update::badge_visible(ev, &self.satz.gespeichert.updates, &self.upd.stand, update::now_secs()) { return None } ev.update.as_ref().map(|u| u.version)
    }
    pub(crate) fn info_akt(&mut self, a: InfoAkt) -> Task<Msg> {
        match a {
            InfoAkt::UeberAuf => { self.upd.ueber = true; if !cfg!(test) { self.upd.stand = update::State::load() } }
            InfoAkt::UeberZu => { self.upd.ueber = false; self.upd.hinweise = false; self.upd.herkunft = false }
            InfoAkt::Herkunft => self.upd.herkunft = !self.upd.herkunft,
            InfoAkt::Hinweise => { self.upd.hinweise = !self.upd.hinweise; self.md_grenze = crate::mdansicht::BLOECKE }
            InfoAkt::FeedbackAuf => { let ziel = vc_ide_model::recent::short_name(&self.system_id()).to_string(); let _ = ziel; self.upd.feedback = Some((feedback::current(Some(&self.system_id())), None)) }
            InfoAkt::FeedbackZu => self.upd.feedback = None,
            InfoAkt::Mail => { if let Some((m, st)) = &mut self.upd.feedback { *st = Some(match aussen::mail_oeffnen(&m.url) { Ok(()) => (true, vc_i18n::t!("infodlg-mailprogramm-geoeffnet-gesendet-wird-erst-dort").into()), Err(e) => (false, vc_i18n::t!("infodlg-mailprogramm-nicht-gestartet-e-nimm-in", e = &e)) }) } }
            InfoAkt::Kopieren => { if let Some((m, st)) = &mut self.upd.feedback { *st = Some((true, vc_i18n::t!("infodlg-in-die-zwischenablage-kopiert").into())); return iced::clipboard::write(m.clipboard()) } }
            InfoAkt::DebugLog(an) => { self.satz.akt(SetAkt::DebugLog(an)) }
            InfoAkt::Pruefen => { self.upd.update_offen = false; return self.update_pruefen() }
            InfoAkt::Ergebnis(r) => {
                self.upd.laeuft = false; if !cfg!(test) { update::record(&r, &mut self.upd.stand) } else { self.upd.stand.last_result = match &r.result { Ok(e) => e.update.as_ref().map_or(vc_i18n::t!("update-aktuell"), |u| vc_i18n::t!("infodlg-version-p1-verfuegbar", p1 = u.version.to_string())), Err(e) => vc_i18n::t!("infodlg-fehler-e", e = &e) }; self.upd.stand.last_attempt = r.at }
                if let Ok(ev) = &r.result { self.upd.eval = Some(ev.clone()); if ev.update.is_some() { self.upd.update_offen = true } }
                self.upd.letzte = Some(*r);
            }
            InfoAkt::Ansehen => { let url = self.upd.eval.as_ref().and_then(|e| e.update.as_ref()).map(|u| u.url.clone()).unwrap_or_else(|| update::release_url(None)); aussen::url_oeffnen(&url) }
            InfoAkt::Spaeter => { self.upd.stand.snooze_until = update::now_secs() + update::SNOOZE_SECS; if !cfg!(test) { let _ = self.upd.stand.save(); } self.upd.update_offen = false }
            InfoAkt::Ueberspringen => { if let Some(v) = self.upd.eval.as_ref().and_then(|e| e.update.as_ref()).map(|u| u.version.to_string()) { self.satz.einzeln_pub(move |s| s.updates.skipped = v.clone()) } self.upd.update_offen = false }
            InfoAkt::UpdateZu => self.upd.update_offen = false,
            InfoAkt::UpdateAuf => self.upd.update_offen = true,
            InfoAkt::Web(u) => aussen::url_oeffnen(u),
        }
        Task::none()
    }

    /// Tabelle der mitgelieferten eigenen Zeichensätze: Urheber, Lizenz, Revision, Prüfstand und Freigabe (aus `data/charsets/index.json` und dem `origin` jeder Datei)
    fn herkunft_karte(&self, pal: Pal) -> Element<'_, Msg> {
        let mut zeilen: Vec<Element<Msg>> = vec![text(vc_i18n::t!("infodlg-eigene-zeichensaetze-vc-standard")).size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(),
            klein(pal, vc_i18n::t!("infodlg-entwuerfe-des-vintagecoder-projekts-keine-nachbildung"))];
        let saetze = vc_media::project::eigene_saetze(&self.data, "");
        if saetze.is_empty() { zeilen.push(klein(pal, vc_i18n::t!("infodlg-keine-eigenen-zeichensaetze-gefunden-data-charsets"))) }
        for e in saetze {
            let o: serde_json::Value = std::fs::read_to_string(&e.datei).ok().and_then(|t| serde_json::from_str(&t).ok()).map(|v: serde_json::Value| v["origin"].clone()).unwrap_or_default();
            let (pruef, frei) = (o["check"]["result"].as_str().unwrap_or("offen").to_string(), o["freigabe"].as_str().unwrap_or("offen").to_string());
            zeilen.push(row![text(e.name.clone()).size(12.5).color(pal.text).width(Length::FillPortion(5)), text(vc_i18n::t!("infodlg-p1-rev-p2", p1 = &e.lizenz, p2 = &e.revision)).size(12).color(pal.mute).width(Length::FillPortion(3)), text(vc_i18n::t!("infodlg-pruefung-pruef-freigabe-frei-p3", pruef = &pruef, frei = &frei, p3 = if pruef != "bestanden" && frei != "offen" { format!(" {}", vc_i18n::t!("infodlg-abstandspruefung-nicht-bestanden-vom-inhaber-akzeptiert")) } else { String::new() })).size(12).color(if frei == "offen" { pal.warn } else { pal.ok }).width(Length::FillPortion(4))].spacing(8).into());
        }
        karte(pal, zeilen)
    }
    pub(crate) fn info_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let z = &self.upd;
        if let Some((m, st)) = &z.feedback {
            let mut c = column![abschnitt(pal, vc_i18n::t!("infodlg-feedback-senden"), &vc_i18n::t!("infodlg-die-mail-geht-an-p1-sie", p1 = feedback::ADDRESS)),
                klein(pal, vc_i18n::t!("infodlg-beschreibe-den-fehler-in-deinem-mailprogramm")),
                text(vc_i18n::t!("infodlg-betreff")).size(12).color(pal.mute), mono(pal, m.subject.clone()), text(vc_i18n::t!("infodlg-text")).size(12).color(pal.mute), scrollable(text(m.body.clone()).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text)).height(Length::Fixed(240.0))].spacing(8);
            if !m.dropped.is_empty() { c = c.push(text(vc_i18n::t!("infodlg-gekuerzt-wegen-der-laengengrenze-von-mail", p1 = m.dropped.join(", "))).size(11.5).color(pal.warn)) }
            if m.too_long { c = c.push(text(vc_i18n::t!("infodlg-der-link-ist-trotz-kuerzung-sehr")).size(11.5).color(pal.warn)) }
            c = c.push(klein(pal, vc_i18n::t!("infodlg-pfade-in-deinem-persoenlichen-ordner-stehen")));
            c = c.push(klein(pal, vc_i18n::t!("infodlg-fehler-schwer-zu-beschreiben-schalte-das")));
            c = c.push(checkbox(self.satz.gespeichert.debug_log).label(vc_i18n::t!("infodlg-debug-protokoll-schreiben")).on_toggle(|b| i(InfoAkt::DebugLog(b))).size(16).text_size(13));
            c = c.push(row![knopf(pal, vc_i18n::t!("infodlg-im-mailprogramm-oeffnen"), Some(i(InfoAkt::Mail)), pal.text, true), knopf(pal, vc_i18n::t!("infodlg-in-zwischenablage-kopieren"), Some(i(InfoAkt::Kopieren)), pal.text, false), knopf(pal, vc_i18n::t!("infodlg-abbrechen"), Some(i(InfoAkt::FeedbackZu)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
            if let Some((ok, t)) = st { c = c.push(text(t.clone()).size(12).color(if *ok { pal.ok } else { pal.warn })) }
            return Some(modal(pal, c.into(), 620.0, i(InfoAkt::FeedbackZu)));
        }
        if z.update_offen {
            if let Some(u) = z.eval.as_ref().and_then(|e| e.update.as_ref()) {
                let mut c = column![abschnitt(pal, &vc_i18n::t!("infodlg-neue-version-p1", p1 = u.version.to_string()), &vc_i18n::t!("infodlg-installiert-ist-p1-vintagecoder-laedt-und", p1 = Version::installed().to_string())), scrollable(text(u.notes.clone()).size(12.5).color(pal.text)).height(Length::Fixed(260.0))].spacing(10);
                if u.project_format_warning { c = c.push(text(vc_i18n::t!("infodlg-achtung-die-neue-version-schreibt-ein")).size(12).color(pal.warn)) }
                c = c.push(row![knopf(pal, vc_i18n::t!("infodlg-ansehen"), Some(i(InfoAkt::Ansehen)), pal.text, true), knopf(pal, vc_i18n::t!("infodlg-spaeter-erinnern"), Some(i(InfoAkt::Spaeter)), pal.text, false), knopf(pal, vc_i18n::t!("infodlg-diese-version-ueberspringen"), Some(i(InfoAkt::Ueberspringen)), pal.text, false), knopf(pal, vc_i18n::t!("infodlg-schliessen"), Some(i(InfoAkt::UpdateZu)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
                return Some(modal(pal, c.into(), 600.0, i(InfoAkt::UpdateZu)));
            }
        }
        if z.ueber {
            let upd: Element<Msg> = if vc_ide::features::features().update() {
                let (inst, gesehen) = (Version::installed().to_string(), z.eval.as_ref().map(|e| e.latest.to_string()).unwrap_or_default());
                karte(pal, vec![text(vc_i18n::t!("infodlg-updates")).size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), klein(pal, vc_i18n::t!("infodlg-installiert-inst-p2", inst = &inst, p2 = match z.eval.as_ref().and_then(|e| e.update.as_ref()) { Some(u) => format!("   {}", vc_i18n::t!("infodlg-neuere-version-p1", p1 = &u.version)), None if !gesehen.is_empty() => format!("   {}", vc_i18n::t!("infodlg-aktuell")).to_string(), None => String::new() })),
                    klein(pal, vc_i18n::t!("infodlg-zuletzt-geprueft-p1-p2", p1 = if z.stand.last_check == 0 { vc_i18n::t!("infodlg-noch-nie").to_string() } else { update::format_time(z.stand.last_check) + " UTC" }, p2 = if z.stand.last_result.is_empty() { String::new() } else { format!(" ({})", z.stand.last_result) })),
                    row![knopf(pal, if z.laeuft { vc_i18n::t!("infodlg-pruefe") } else { vc_i18n::t!("infodlg-jetzt-pruefen") }, (!z.laeuft).then(|| i(InfoAkt::Pruefen)), pal.text, false), knopf(pal, vc_i18n::t!("infodlg-aenderungen-ansehen"), z.eval.as_ref().is_some_and(|e| e.update.is_some()).then(|| i(InfoAkt::Ansehen)), pal.text, false)].spacing(8).wrap().vertical_spacing(6).into(),
                    klein(pal, vc_i18n::t!("infodlg-jetzt-pruefen-fragt-die-versionsliste-update"))])
            } else { text("").into() };
            let mut c = column![row![image(self.logos[2].clone()).height(54), column![text(vc_i18n::t!("infodlg-vintagecoder-p1", p1 = version())).size(20).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), klein(pal, vc_i18n::t!("infodlg-entwicklungsumgebung-fuer-retro-computer"))].spacing(2)].spacing(14).align_y(Alignment::Center),
                klein(pal, vc_i18n::t!("infodlg-assembler-basic-grafik-und-klang-editoren")),
                text(vc_i18n::t!("infodlg-enthaelt-wahrscheinlich-noch-viele-fehler")).size(12).color(pal.warn)].spacing(10);
            c = c.push(klein(pal, vc_ide::license::notice_short()));
            c = c.push(karte(pal, vec![row![text(vc_i18n::t!("infodlg-lizenz")).size(12.5).color(pal.mute).width(90), text(LIZENZ).size(13)].into(), row![text(vc_i18n::t!("infodlg-autor")).size(12.5).color(pal.mute).width(90), text(vc_i18n::t!("infodlg-christian-drueeke")).size(13)].into(), 
                row![text(vc_i18n::t!("infodlg-dank")).size(12.5).color(pal.mute).width(90), text(vc_i18n::t!("infodlg-an-die-community-n3rdroom-discord")).size(13)].into(),
                row![text(vc_i18n::t!("infodlg-webseite")).size(12.5).color(pal.mute).width(90), knopf(pal, WEBSITE, Some(i(InfoAkt::Web(WEBSITE))), pal.accent, false)].spacing(0).into(), row![text(vc_i18n::t!("infodlg-quelltext")).size(12.5).color(pal.mute).width(90), knopf(pal, REPOSITORY, Some(i(InfoAkt::Web(REPOSITORY))), pal.accent, false)].spacing(0).into()]));
            c = c.push(upd);
            c = c.push(row![knopf(pal, vc_i18n::t!("infodlg-feedback-senden-2"), Some(i(InfoAkt::FeedbackAuf)), pal.text, false), knopf(pal, if z.herkunft { vc_i18n::t!("infodlg-herkunft-und-lizenzen-ausblenden") } else { vc_i18n::t!("infodlg-herkunft-und-lizenzen") }, Some(i(InfoAkt::Herkunft)), pal.text, false), knopf(pal, if z.hinweise { vc_i18n::t!("infodlg-drittanbieter-hinweise-ausblenden") } else { vc_i18n::t!("infodlg-drittanbieter-hinweise") }, Some(i(InfoAkt::Hinweise)), pal.text, false), iced::widget::space::horizontal(), knopf(pal, vc_i18n::t!("infodlg-schliessen"), Some(i(InfoAkt::UeberZu)), pal.text, true)].spacing(8).wrap().vertical_spacing(6));
            if z.herkunft { c = c.push(self.herkunft_karte(pal)) }
            if z.hinweise { let doc = self.md_doc(HINWEISE); c = c.push(container(scrollable(crate::mdansicht::ansicht(pal, &doc, self.md_grenze, Some(Msg::MdMehr))).height(Length::Fixed(260.0))).padding(8).style(move |_| crate::flaeche(pal.bg, pal.line))) }
            return Some(modal(pal, scrollable(c).into(), 640.0, i(InfoAkt::UeberZu)));
        }
        None
    }
}
use crate::aussen;
