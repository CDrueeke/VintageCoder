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
            InfoAkt::Mail => { if let Some((m, st)) = &mut self.upd.feedback { *st = Some(match aussen::mail_oeffnen(&m.url) { Ok(()) => (true, "Mailprogramm geöffnet. Gesendet wird erst dort.".into()), Err(e) => (false, format!("Mailprogramm nicht gestartet ({e}). Nimm „In Zwischenablage kopieren“.")) }) } }
            InfoAkt::Kopieren => { if let Some((m, st)) = &mut self.upd.feedback { *st = Some((true, "In die Zwischenablage kopiert.".into())); return iced::clipboard::write(m.clipboard()) } }
            InfoAkt::DebugLog(an) => { self.satz.akt(SetAkt::DebugLog(an)) }
            InfoAkt::Pruefen => { self.upd.update_offen = false; return self.update_pruefen() }
            InfoAkt::Ergebnis(r) => {
                self.upd.laeuft = false; if !cfg!(test) { update::record(&r, &mut self.upd.stand) } else { self.upd.stand.last_result = match &r.result { Ok(e) => e.update.as_ref().map_or("aktuell".to_string(), |u| format!("Version {} verfügbar", u.version)), Err(e) => format!("Fehler: {e}") }; self.upd.stand.last_attempt = r.at }
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
        let mut zeilen: Vec<Element<Msg>> = vec![text("Eigene Zeichensätze (VC-Standard)").size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(),
            klein(pal, "Entwürfe des VintageCoder-Projekts, keine Nachbildung von Original-ROMs. Lizenz CC0-1.0: frei nutzbar und weitergebbar. Jede Datei trägt ihre Herkunft im Block „origin“.")];
        let saetze = vc_media::project::eigene_saetze(&self.data, "");
        if saetze.is_empty() { zeilen.push(klein(pal, "Keine eigenen Zeichensätze gefunden (data/charsets/).")) }
        for e in saetze {
            let o: serde_json::Value = std::fs::read_to_string(&e.datei).ok().and_then(|t| serde_json::from_str(&t).ok()).map(|v: serde_json::Value| v["origin"].clone()).unwrap_or_default();
            let (pruef, frei) = (o["check"]["result"].as_str().unwrap_or("offen").to_string(), o["freigabe"].as_str().unwrap_or("offen").to_string());
            zeilen.push(row![text(e.name.clone()).size(12.5).color(pal.text).width(Length::FillPortion(5)), text(format!("{} · Rev. {}", e.lizenz, e.revision)).size(12).color(pal.mute).width(Length::FillPortion(3)), text(format!("Prüfung: {pruef} · Freigabe: {frei}{}", if pruef != "bestanden" && frei != "offen" { " (Abstandsprüfung nicht bestanden, vom Inhaber akzeptiert)" } else { "" })).size(12).color(if frei == "offen" { pal.warn } else { pal.ok }).width(Length::FillPortion(4))].spacing(8).into());
        }
        karte(pal, zeilen)
    }
    pub(crate) fn info_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let z = &self.upd;
        if let Some((m, st)) = &z.feedback {
            let mut c = column![abschnitt(pal, "Feedback senden", &format!("Die Mail geht an {}. Sie wird nur vorbereitet: gesendet wird erst, wenn du in deinem Mailprogramm auf „Senden“ klickst; hier wird nichts ins Netz geschickt.", feedback::ADDRESS)),
                klein(pal, "Beschreibe den Fehler in deinem Mailprogramm in der leeren Zeile über den Angaben; Screenshots und Dateien hängst du dort selbst an."),
                text("Betreff").size(12).color(pal.mute), mono(pal, m.subject.clone()), text("Text").size(12).color(pal.mute), scrollable(text(m.body.clone()).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text)).height(Length::Fixed(240.0))].spacing(8);
            if !m.dropped.is_empty() { c = c.push(text(format!("Gekürzt: wegen der Längengrenze von Mail-Links fehlen {}. „In Zwischenablage kopieren“ enthält alles.", m.dropped.join(", "))).size(11.5).color(pal.warn)) }
            if m.too_long { c = c.push(text("Der Link ist trotz Kürzung sehr lang; manche Mailprogramme schneiden ihn ab. Nimm dann die Zwischenablage.").size(11.5).color(pal.warn)) }
            c = c.push(klein(pal, "Pfade in deinem persönlichen Ordner stehen als ~; Projekt- und Dateinamen kommen nicht vor."));
            c = c.push(klein(pal, "Fehler schwer zu beschreiben? Schalte das Debug-Protokoll ein, wiederhole den Fehler und hänge die Datei an:"));
            c = c.push(checkbox(self.satz.gespeichert.debug_log).label("Debug-Protokoll schreiben").on_toggle(|b| i(InfoAkt::DebugLog(b))).size(16).text_size(13));
            c = c.push(row![knopf(pal, "Im Mailprogramm öffnen", Some(i(InfoAkt::Mail)), pal.text, true), knopf(pal, "In Zwischenablage kopieren", Some(i(InfoAkt::Kopieren)), pal.text, false), knopf(pal, "Abbrechen", Some(i(InfoAkt::FeedbackZu)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
            if let Some((ok, t)) = st { c = c.push(text(t.clone()).size(12).color(if *ok { pal.ok } else { pal.warn })) }
            return Some(modal(pal, c.into(), 620.0, i(InfoAkt::FeedbackZu)));
        }
        if z.update_offen {
            if let Some(u) = z.eval.as_ref().and_then(|e| e.update.as_ref()) {
                let mut c = column![abschnitt(pal, &format!("Neue Version {}", u.version), &format!("Installiert ist {}. VintageCoder lädt und installiert nichts selbst: „Ansehen“ öffnet die Releaseseite im Browser.", Version::installed())), scrollable(text(u.notes.clone()).size(12.5).color(pal.text)).height(Length::Fixed(260.0))].spacing(10);
                if u.project_format_warning { c = c.push(text("Achtung: Die neue Version schreibt ein neueres Projektformat; ältere Versionen können solche Projekte danach nicht mehr lesen.").size(12).color(pal.warn)) }
                c = c.push(row![knopf(pal, "Ansehen …", Some(i(InfoAkt::Ansehen)), pal.text, true), knopf(pal, "Später erinnern", Some(i(InfoAkt::Spaeter)), pal.text, false), knopf(pal, "Diese Version überspringen", Some(i(InfoAkt::Ueberspringen)), pal.text, false), knopf(pal, "Schließen", Some(i(InfoAkt::UpdateZu)), pal.text, false)].spacing(8).wrap().vertical_spacing(6));
                return Some(modal(pal, c.into(), 600.0, i(InfoAkt::UpdateZu)));
            }
        }
        if z.ueber {
            let upd: Element<Msg> = if vc_ide::features::features().update() {
                let (inst, gesehen) = (Version::installed(), z.eval.as_ref().map(|e| e.latest.to_string()).unwrap_or_default());
                karte(pal, vec![text("Updates").size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), klein(pal, format!("Installiert: {inst}{}", match z.eval.as_ref().and_then(|e| e.update.as_ref()) { Some(u) => format!("   ·   Neuere Version: {}", u.version), None if !gesehen.is_empty() => "   ·   aktuell".to_string(), None => String::new() })),
                    klein(pal, format!("Zuletzt geprüft: {}{}", if z.stand.last_check == 0 { "noch nie".to_string() } else { update::format_time(z.stand.last_check) + " UTC" }, if z.stand.last_result.is_empty() { String::new() } else { format!(" ({})", z.stand.last_result) })),
                    row![knopf(pal, if z.laeuft { "Prüfe …" } else { "Jetzt prüfen" }, (!z.laeuft).then(|| i(InfoAkt::Pruefen)), pal.text, false), knopf(pal, "Änderungen ansehen", z.eval.as_ref().is_some_and(|e| e.update.is_some()).then(|| i(InfoAkt::Ansehen)), pal.text, false)].spacing(8).wrap().vertical_spacing(6).into(),
                    klein(pal, "„Jetzt prüfen“ fragt die Versionsliste (update.json) bei github.com ab: GitHub sieht deine IP-Adresse und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Ohne deine Zustimmung geht nie etwas ins Netz.")])
            } else { text("").into() };
            let mut c = column![row![image(self.logos[2].clone()).height(54), column![text(format!("VintageCoder {}", version())).size(20).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), klein(pal, "Entwicklungsumgebung für Retro-Computer")].spacing(2)].spacing(14).align_y(Alignment::Center),
                klein(pal, "Assembler, BASIC, Grafik- und Klang-Editoren, Debugger und Emulator-Anbindung für C64, Atari, MEGA65 und weitere Systeme."),
                text("Alpha-Version: enthält wahrscheinlich noch viele Fehler. Keine Gewährleistung, keine Haftung für Datenverlust (siehe Lizenz).").size(12).color(pal.warn)].spacing(10);
            c = c.push(karte(pal, vec![row![text("Lizenz").size(12.5).color(pal.mute).width(90), text(LIZENZ).size(13)].into(), row![text("Autor").size(12.5).color(pal.mute).width(90), text("Christian Drüeke").size(13)].into(), 
                row![text("Dank").size(12.5).color(pal.mute).width(90), text("an die Community „N3rdroom“ (Discord)").size(13)].into(),
                row![text("Webseite").size(12.5).color(pal.mute).width(90), knopf(pal, WEBSITE, Some(i(InfoAkt::Web(WEBSITE))), pal.accent, false)].spacing(0).into(), row![text("Quelltext").size(12.5).color(pal.mute).width(90), knopf(pal, REPOSITORY, Some(i(InfoAkt::Web(REPOSITORY))), pal.accent, false)].spacing(0).into()]));
            c = c.push(upd);
            c = c.push(row![knopf(pal, "Feedback senden …", Some(i(InfoAkt::FeedbackAuf)), pal.text, false), knopf(pal, if z.herkunft { "Herkunft und Lizenzen ausblenden" } else { "Herkunft und Lizenzen" }, Some(i(InfoAkt::Herkunft)), pal.text, false), knopf(pal, if z.hinweise { "Drittanbieter-Hinweise ausblenden" } else { "Drittanbieter-Hinweise" }, Some(i(InfoAkt::Hinweise)), pal.text, false), iced::widget::space::horizontal(), knopf(pal, "Schließen", Some(i(InfoAkt::UeberZu)), pal.text, true)].spacing(8).wrap().vertical_spacing(6));
            if z.herkunft { c = c.push(self.herkunft_karte(pal)) }
            if z.hinweise { let doc = self.md_doc(HINWEISE); c = c.push(container(scrollable(crate::mdansicht::ansicht(pal, &doc, self.md_grenze, Some(Msg::MdMehr))).height(Length::Fixed(260.0))).padding(8).style(move |_| crate::flaeche(pal.bg, pal.line))) }
            return Some(modal(pal, scrollable(c).into(), 640.0, i(InfoAkt::UeberZu)));
        }
        None
    }
}
use crate::aussen;
