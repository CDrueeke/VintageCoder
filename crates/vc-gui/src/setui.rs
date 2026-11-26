// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungsseite der neuen Oberfläche (Stufe 4a): Seiten Allgemein, Zielsysteme und Hardware, ROMs, Toolchain, Editor, Tastenkürzel.
//! Die Werte (`vc_ide::settings::Settings`) werden in einem Entwurf bearbeitet; „Speichern“ schreibt, „Verwerfen“ stellt den gespeicherten
//! Stand wieder her. Auswahlen, die sofort wirken sollen (Debug-Protokoll, Bilder laden, Update-Häufigkeit), speichern einzeln.
//! Im Testlauf geht nie etwas in die echte Einstellungsdatei: sie liegt dann in einem Temp-Ordner (`Satz::neu`).
use crate::{argsui, flaeche, knopf, romui, zielui, App, Msg, Pal};
use iced::Task;
use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
use iced::{Alignment, Border, Element, Length};
use vc_gui_ui::befehle;
use vc_ide::settings::Settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seite { Allgemein, Ziele, Roms, Toolchain, Editor, Kuerzel }
/// Titel einer Einstellungsseite in der gewählten Sprache
pub fn seite_titel(s: Seite) -> String {
    match s {
        Seite::Allgemein => vc_i18n::t!("setui-seite-allgemein"), Seite::Ziele => vc_i18n::t!("setui-seite-zielsysteme-und-hardware"), Seite::Roms => vc_i18n::t!("setui-seite-roms"),
        Seite::Toolchain => vc_i18n::t!("setui-seite-toolchain"), Seite::Editor => vc_i18n::t!("setui-seite-editor"), Seite::Kuerzel => vc_i18n::t!("setui-seite-tastenkuerzel"),
    }
}

pub const SEITEN: [(Seite, &str); 6] = [(Seite::Allgemein, "Allgemein"), (Seite::Ziele, "Zielsysteme und Hardware"), (Seite::Roms, "ROMs"), (Seite::Toolchain, "Toolchain"), (Seite::Editor, "Editor"), (Seite::Kuerzel, "Tastenkürzel")];

#[derive(Clone, Debug)]
pub enum SetAkt {
    Seite(Seite), Speichern, Verwerfen,
    /// Sprache der Oberfläche: `auto`, `de` oder `en` (wirkt sofort, wird gespeichert)
    Sprache(String),
    DebugLog(bool), HilfeBilder(String), HilfeUrl(String), UpdPruefen(String), UpdUrl(String), UpdSkipZurueck,
    AsmArgs(String), KuerzelSuche(String), OrdnerZeigen, Rom(romui::RomAkt), Ziel(zielui::ZielAkt), Arg(argsui::ArgAkt), ScratchSicherung(bool),
}

pub struct Satz { pub seite: Seite, pub entwurf: Settings, pub gespeichert: Settings, pub datei: Option<std::path::PathBuf>, pub meldung: Option<(bool, String)>, pub kuerzel_suche: String, pub roms: romui::RomZ, pub ziel: zielui::ZielZ, /** umgeschaltete Auf-/Zu-Zustände der Parametereditoren */ pub args_offen: std::collections::HashMap<String, bool>, pub arg_fokus: Option<(String, String)>, pub scratch_an: bool }

impl Satz {
    pub fn neu() -> Satz {
        // Testlauf: eigene Datei im Temp-Ordner, nie die echte Einstellungsdatei
        let datei = if cfg!(test) { Some(std::env::temp_dir().join(format!("vcgui-einstellungen-{}-{:?}.json", std::process::id(), std::thread::current().id()))) } else { vc_ide::settings::file() };
        let s = if cfg!(test) { Settings::default() } else { datei.as_deref().and_then(|d| Settings::load_from(d).ok()).unwrap_or_default() };
        let mut s = s; s.migrate_legacy();
        Satz { seite: Seite::Allgemein, gespeichert: s.clone(), entwurf: s, datei, meldung: None, kuerzel_suche: String::new(), roms: Default::default(), ziel: Default::default(), args_offen: Default::default(), arg_fokus: None, scratch_an: if cfg!(test) { true } else { vc_ide_model::scratchsave::enabled() } }
    }
    pub fn geaendert(&self) -> bool { self.entwurf != self.gespeichert }
    /// Schreibt `s` (im Betrieb über die gemeinsame Einstellungsstelle der IDE, im Test nur in die Temp-Datei)
    fn schreiben(&mut self, s: Settings) -> Result<(), String> {
        if cfg!(test) { match &self.datei { Some(d) => s.save_atomic(d), None => Ok(()) } } else { vc_ide::settings::set(s) }
    }
    /// Alles speichern
    pub fn speichern(&mut self) { let s = self.entwurf.clone(); self.meldung = Some(match self.schreiben(s.clone()) { Ok(()) => { self.gespeichert = s; (true, vc_i18n::t!("setui-gespeichert").into()) } Err(e) => (false, vc_i18n::t!("setui-speichern-fehlgeschlagen-e", e = &e)) }) }
    /// Nur diese Änderung speichern, ohne anderen ungespeicherten Entwurf mitzunehmen
    pub fn einzeln_pub(&mut self, f: impl Fn(&mut Settings)) { self.einzeln(f) }
    fn einzeln(&mut self, f: impl Fn(&mut Settings)) {
        let mut g = self.gespeichert.clone(); f(&mut g); f(&mut self.entwurf);
        self.meldung = match self.schreiben(g.clone()) { Ok(()) => { self.gespeichert = g; None } Err(e) => Some((false, vc_i18n::t!("setui-speichern-fehlgeschlagen-e", e = &e))) };
    }
    pub fn akt(&mut self, a: SetAkt) {
        match a {
            SetAkt::Seite(s) => { self.seite = s; self.meldung = None }
            SetAkt::Speichern => self.speichern(),
            SetAkt::Verwerfen => { self.entwurf = self.gespeichert.clone(); self.meldung = None }
            SetAkt::Sprache(c) => self.einzeln(|s| s.language = if c == "auto" { String::new() } else { c.clone() }),
            SetAkt::DebugLog(an) => { self.einzeln(|s| s.debug_log = an); if !cfg!(test) { vc_ide::debuglog::set_enabled(an) } }
            SetAkt::HilfeBilder(w) => self.einzeln(|s| s.help.images = if w == "ask" { String::new() } else { w.clone() }),
            SetAkt::HilfeUrl(u) => self.entwurf.help.images_url = u,
            SetAkt::UpdPruefen(w) => self.einzeln(|s| s.updates.check = w.clone()),
            SetAkt::UpdUrl(u) => self.entwurf.updates.manifest_url = u,
            SetAkt::UpdSkipZurueck => self.einzeln(|s| s.updates.skipped.clear()),
            SetAkt::AsmArgs(a) => self.entwurf.assembler.args = a,
            SetAkt::KuerzelSuche(q) => self.kuerzel_suche = q,
            SetAkt::Rom(_) | SetAkt::Ziel(_) | SetAkt::Arg(_) => {}
            SetAkt::ScratchSicherung(an) => { if !cfg!(test) { vc_ide_model::scratchsave::set_enabled(an) } self.scratch_an = an }
            SetAkt::OrdnerZeigen => { if let Some(d) = vc_core::userdirs::config_dir() { crate::aussen::ausfuehren(crate::aussen::Aufruf::ImOrdnerZeigen(d.join("settings.json"))) } }
        }
    }
}

impl Drop for Satz { fn drop(&mut self) { if cfg!(test) { if let Some(d) = &self.datei { let _ = std::fs::remove_file(d); } } } }

// ---------------------------------------------------------------------------------------------- Bausteine
pub(crate) fn abschnitt<'a>(pal: Pal, titel: impl AsRef<str>, hinweis: impl AsRef<str>) -> Element<'a, Msg> {
    column![text(titel.as_ref().to_string()).size(15).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(hinweis.as_ref().to_string()).size(12).color(pal.mute)].spacing(3).into()
}
pub(crate) fn karte<'a>(pal: Pal, inhalt: Vec<Element<'a, Msg>>) -> Element<'a, Msg> {
    container(column(inhalt).spacing(10)).padding(14).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
}
fn zeile<'a>(pal: Pal, titel: impl AsRef<str>, inhalt: Element<'a, Msg>) -> Element<'a, Msg> {
    row![text(titel.as_ref().to_string()).size(13).color(pal.text).width(170), container(inhalt).width(Length::Fill)].spacing(12).align_y(Alignment::Start).into()
}
pub(crate) fn klein<'a>(pal: Pal, s: impl Into<String>) -> Element<'a, Msg> { text(s.into()).size(11.5).color(pal.mute).into() }
pub(crate) fn mono<'a>(pal: Pal, s: impl Into<String>) -> Element<'a, Msg> { text(s.into()).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute).into() }
pub(crate) fn wahl<'a>(pal: Pal, optionen: &[(&str, &str)], aktuell: &str, m: impl Fn(String) -> Msg) -> Element<'a, Msg> {
    row(optionen.iter().map(|(w, l)| { let on = *w == aktuell; button(text(l.to_string()).size(13).color(pal.text)).padding([5, 12]).on_press(m(w.to_string())).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6).into()
}
pub(crate) fn feld_klein<'a>(pal: Pal, hinweis: impl AsRef<str>, wert: &str, m: impl Fn(String) -> Msg + 'a) -> Element<'a, Msg> {
    let _ = pal; text_input(hinweis.as_ref(), wert).on_input(m).size(13).padding(6).width(Length::Fill).into()
}

impl App {
    /// Aktion der Einstellungsseite; die Seite „ROMs“ liest ihren Stand beim ersten Öffnen im Hintergrund
    pub(crate) fn set_akt(&mut self, a: SetAkt) -> Task<Msg> {
        match a {
            SetAkt::Rom(r) => self.rom_akt(r),
            SetAkt::Ziel(a) => self.ziel_akt(a),
            SetAkt::Arg(x) => { self.arg_akt(x); Task::none() }
            SetAkt::Sprache(c) => { self.satz.akt(SetAkt::Sprache(c)); self.sprache_geaendert(); Task::none() }
            SetAkt::Seite(Seite::Ziele) => { self.satz.akt(SetAkt::Seite(Seite::Ziele)); self.ziel_oeffnen() }
            SetAkt::Seite(Seite::Roms) => { self.satz.akt(SetAkt::Seite(Seite::Roms)); if self.satz.roms.stand.is_none() && !self.satz.roms.laedt { self.rom_laden() } else { Task::none() } }
            a => { self.satz.akt(a); Task::none() }
        }
    }
    pub(crate) fn einstellungen_ansicht(&self, pal: Pal) -> Element<'_, Msg> {
        let z = &self.satz; let sa = |s: Seite| Msg::Set(SetAkt::Seite(s));
        let reiter = row(SEITEN.iter().map(|(s, l)| { let on = z.seite == *s; button(text(seite_titel(*s)).size(13).color(pal.text)).padding([6, 12]).on_press(sa(*s)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6);
        let inhalt: Element<Msg> = match z.seite { Seite::Allgemein => self.seite_allgemein(pal), Seite::Toolchain => self.seite_toolchain(pal), Seite::Editor => self.seite_editor(pal), Seite::Kuerzel => self.seite_kuerzel(pal), Seite::Roms => self.seite_roms(pal), Seite::Ziele => self.seite_ziele(pal) };
        let fuss: Element<Msg> = if z.geaendert() || z.meldung.is_some() {
            let msg = z.meldung.clone().map(|(ok, t)| text(t).size(12.5).color(if ok { pal.ok } else { pal.err }));
            let mut r = row![].spacing(10).align_y(Alignment::Center);
            if z.geaendert() { r = r.push(text(vc_i18n::t!("setui-ungespeicherte-aenderungen")).size(12.5).color(pal.warn)).push(space::horizontal()).push(knopf(pal, vc_i18n::t!("setui-verwerfen"), Some(Msg::Set(SetAkt::Verwerfen)), pal.text, false)).push(knopf(pal, vc_i18n::t!("setui-speichern"), Some(Msg::Set(SetAkt::Speichern)), pal.text, true)) }
            if let Some(m) = msg { r = r.push(m) }
            container(r.wrap().vertical_spacing(6)).padding([8, 14]).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
        } else { space::vertical().height(0).into() };
        column![container(column![text(vc_gui_ui::texte::einstellungen()).size(20).color(pal.text), reiter].spacing(10)).padding([14, 20]).width(Length::Fill), scrollable(container(inhalt).padding([6, 20]).max_width(1000)).height(Length::Fill), fuss].into()
    }
    fn seite_allgemein(&self, pal: Pal) -> Element<'_, Msg> {
        let (e, g) = (&self.satz.entwurf, &self.einst);
        let sc = g.schema;
        let sprache = self.sprache_karte(pal, e.language.as_str(), |w| Msg::Set(SetAkt::Sprache(w)));
        let darst = karte(pal, vec![
            zeile(pal, &vc_gui_ui::texte::farbschema(), row![
                self.seg(pal, vc_gui_ui::texte::dunkel(), sc == crate::einstellungen::Schema::Dunkel, Msg::SetSchema(crate::einstellungen::Schema::Dunkel)), self.seg(pal, vc_gui_ui::texte::hell(), sc == crate::einstellungen::Schema::Hell, Msg::SetSchema(crate::einstellungen::Schema::Hell)),
                self.seg(pal, vc_gui_ui::texte::system(), sc == crate::einstellungen::Schema::System, Msg::SetSchema(crate::einstellungen::Schema::System)), self.seg(pal, vc_gui_ui::texte::kontrast(), sc == crate::einstellungen::Schema::Kontrast, Msg::SetSchema(crate::einstellungen::Schema::Kontrast))].spacing(6).wrap().vertical_spacing(6).into()),
            zeile(pal, &vc_gui_ui::texte::oberflaeche(), row![knopf(pal, "−", Some(Msg::SetZoom(g.zoom - 0.1)), pal.text, false), text(format!("{:.0} %", g.zoom * 100.0)).size(13).color(pal.text), knopf(pal, "+", Some(Msg::SetZoom(g.zoom + 0.1)), pal.text, false)].spacing(10).align_y(Alignment::Center).into()),
            klein(pal, &befehle::plattform_text(&vc_i18n::t!("setui-groesse-der-ganzen-oberflaeche-auch-mit"))),
        ]);
        let letzter = vc_ide::setup::Paths::current().map(|p| vc_ide::setup::WizardState::load(&p.wizard).last_run).filter(|d| !d.is_empty());
        let konfig = karte(pal, vec![zeile(pal, vc_i18n::t!("setui-konfiguration"), row![knopf(pal, vc_i18n::t!("setui-exportieren"), Some(Msg::KonfigWahl(crate::konfig::KArt::Export)), pal.text, false), knopf(pal, vc_i18n::t!("setui-importieren"), Some(Msg::KonfigWahl(crate::konfig::KArt::Import)), pal.text, false), knopf(pal, vc_i18n::t!("setui-zuruecksetzen"), Some(Msg::KonfigResetFrage), pal.text, false)].spacing(8).wrap().vertical_spacing(8).into()),
            klein(pal, vc_i18n::t!("setui-exportieren-schreibt-die-einstellungen-der-ide"))]);
        let einr = karte(pal, vec![zeile(pal, vc_i18n::t!("setui-assistent"), row![knopf(pal, vc_i18n::t!("setui-assistent-starten"), Some(Msg::Befehl(vc_gui_ui::befehle::Cmd::Assistent)), pal.text, false), klein(pal, match letzter { Some(d) => vc_i18n::t!("setui-zuletzt-ausgefuehrt-d", d = &d), None => vc_i18n::t!("setui-noch-nicht-ausgefuehrt").into() })].spacing(10).align_y(Alignment::Center).wrap().into())]);
        let env_log = std::env::var("VC_DEBUG_LOG").ok().filter(|v| !v.trim().is_empty());
        let an = if env_log.is_some() { vc_ide::debuglog::enabled() } else { e.debug_log };
        let mut f = vec![zeile(pal, vc_i18n::t!("setui-debug-protokoll"), wahl(pal, &[("an", &vc_i18n::t!("setui-protokolldatei-schreiben")), ("aus", &vc_i18n::t!("setui-aus"))], if an { "an" } else { "aus" }, |w| Msg::Set(SetAkt::DebugLog(w == "an"))))];
        if let Some(v) = &env_log { f.push(text(vc_i18n::t!("setui-umgebungsvariable-vc-debug-log-ueberstimmt-die", v = &v)).size(11.5).color(pal.warn).into()) }
        if let Some(p) = vc_ide::debuglog::path() { f.push(mono(pal, vc_i18n::t!("setui-datei-p1", p1 = p.display()))) }
        f.push(klein(pal, vc_i18n::t!("setui-pfade-im-persoenlichen-ordner-stehen-als")));
        let fehler = karte(pal, f);
        let bilder = if e.help.images.is_empty() { "ask" } else { e.help.images.as_str() };
        let mut h = vec![zeile(pal, vc_i18n::t!("setui-bilder-laden"), wahl(pal, &[("ask", &vc_i18n::t!("setui-fragen")), ("always", &vc_i18n::t!("setui-immer")), ("never", &vc_i18n::t!("setui-nie"))], bilder, |w| Msg::Set(SetAkt::HilfeBilder(w)))), zeile(pal, vc_i18n::t!("setui-adresse"), feld_klein(pal, vc_ide::help_images::DEFAULT_BASE, &e.help.images_url, |s| Msg::Set(SetAkt::HilfeUrl(s))))];
        if let Ok(v) = std::env::var("VC_HELP_IMAGES") { h.push(text(vc_i18n::t!("setui-umgebungsvariable-vc-help-images-ueberstimmt-die", v = &v)).size(11.5).color(pal.warn).into()) }
        h.push(klein(pal, vc_i18n::t!("setui-ein-ordner-statt-einer-webadresse-wird")));
        if let Some(d) = vc_ide::help_images::cache_dir() { h.push(mono(pal, vc_i18n::t!("setui-zwischenspeicher-p1", p1 = d.display()))) }
        let hilfe = karte(pal, h);
        let mut spalte = column![abschnitt(pal, vc_i18n::t!("settings-language-title"), vc_i18n::t!("settings-language-hint")), sprache, abschnitt(pal, vc_i18n::t!("setui-darstellung"), vc_i18n::t!("setui-gilt-fuer-die-ganze-oberflaeche-und")), darst, abschnitt(pal, vc_i18n::t!("setui-konfiguration-sichern"), vc_i18n::t!("setui-einstellungen-in-eine-datei-schreiben-von")), konfig, abschnitt(pal, vc_i18n::t!("setui-einrichtung"), vc_i18n::t!("setui-der-konfigurationsassistent-prueft-die-einrichtung-und")), einr,
            abschnitt(pal, vc_i18n::t!("setui-fehlersuche"), vc_i18n::t!("setui-das-debug-protokoll-schreibt-befehle-meldungen")), fehler,
            abschnitt(pal, vc_i18n::t!("setui-hilfe"), vc_i18n::t!("setui-die-bilder-des-handbuchs-liegen-im")), hilfe].spacing(8);
        if vc_ide::features::features().update() {
            let u = &e.updates; let v = if u.check.is_empty() { "" } else { u.check.as_str() };
            let mut k = vec![zeile(pal, vc_i18n::t!("setui-nach-updates-suchen"), wahl(pal, &[("never", &vc_i18n::t!("setui-nur-auf-wunsch")), ("daily", &vc_i18n::t!("setui-taeglich-beim-start")), ("weekly", &vc_i18n::t!("setui-woechentlich-beim-start"))], v, |w| Msg::Set(SetAkt::UpdPruefen(w))))];
            if u.check.is_empty() { k.push(text(vc_i18n::t!("setui-noch-nicht-entschieden-es-wird-nichts")).size(11.5).color(pal.warn).into()) }
            k.push(zeile(pal, vc_i18n::t!("setui-kanal"), text(vc_i18n::t!("setui-stabil-nur-fertige-versionen")).size(13).color(pal.text).into()));
            k.push(zeile(pal, vc_i18n::t!("setui-jetzt-pruefen"), row![knopf(pal, if self.upd.laeuft { vc_i18n::t!("setui-pruefe") } else { vc_i18n::t!("setui-jetzt-pruefen") }, (!self.upd.laeuft).then(|| Msg::Befehl(vc_gui_ui::befehle::Cmd::UpdatePruefen)), pal.text, false), klein(pal, if self.upd.stand.last_result.is_empty() { vc_i18n::t!("setui-noch-nicht-geprueft").to_string() } else { self.upd.stand.last_result.clone() })].spacing(10).align_y(Alignment::Center).wrap().into()));
            if !u.skipped.is_empty() { k.push(zeile(pal, vc_i18n::t!("setui-uebersprungen"), row![text(vc_i18n::t!("setui-version-p1", p1 = &u.skipped)).size(13).color(pal.text), knopf(pal, vc_i18n::t!("setui-zuruecknehmen"), Some(Msg::Set(SetAkt::UpdSkipZurueck)), pal.text, false)].spacing(10).align_y(Alignment::Center).into())) }
            k.push(zeile(pal, vc_i18n::t!("setui-adresse"), feld_klein(pal, vc_ide::update::FEED_URL, &u.manifest_url, |s| Msg::Set(SetAkt::UpdUrl(s)))));
            if let Ok(v) = std::env::var("VC_UPDATE_URL") { k.push(text(vc_i18n::t!("setui-umgebungsvariable-vc-update-url-ueberstimmt-die", v = &v)).size(11.5).color(pal.warn).into()) }
            k.push(klein(pal, vc_i18n::t!("setui-die-adresse-betrifft-nur-die-versionsliste")));
            spalte = spalte.push(abschnitt(pal, vc_i18n::t!("setui-updates"), vc_i18n::t!("setui-vintagecoder-kann-nachsehen-ob-es-eine"))).push(karte(pal, k));
        }
        let dir = vc_core::userdirs::config_dir().map(|d| d.display().to_string()).unwrap_or_else(|| "–".into());
        spalte = spalte.push(abschnitt(pal, vc_i18n::t!("setui-dateien"), vc_i18n::t!("setui-wo-vintagecoder-einstellungen-ablegt"))).push(karte(pal, vec![zeile(pal, vc_i18n::t!("setui-ordner"), mono(pal, dir)), zeile(pal, vc_i18n::t!("setui-einstellungen"), mono(pal, vc_ide::settings::file().map(|f| f.display().to_string()).unwrap_or_else(|| "–".into()))),
            klein(pal, vc_i18n::t!("setui-die-einstellungsdatei-enthaelt-die-passwoerter-der")), row![knopf(pal, vc_i18n::t!("setui-im-ordner-zeigen"), Some(Msg::Set(SetAkt::OrdnerZeigen)), pal.text, false)].into()]));
        spalte.padding([0, 0]).into()
    }
    fn seg(&self, pal: Pal, name: impl Into<String>, on: bool, m: Msg) -> Element<'_, Msg> {
        button(text(name.into()).size(13).color(pal.text)).padding([5, 12]).on_press(m).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into()
    }
    fn seite_toolchain(&self, pal: Pal) -> Element<'_, Msg> {
        let e = &self.satz.entwurf;
        let asm = karte(pal, vec![
            zeile(pal, vc_i18n::t!("setui-assembler"), row![text(vc_i18n::t!("setui-eingebaut-vc-asm")).size(13).color(pal.text)].into()),
            zeile(pal, vc_i18n::t!("setui-include-suche"), column![klein(pal, vc_i18n::t!("setui-1-ordner-des-projekts")), klein(pal, vc_i18n::t!("setui-2-zusaetzliche-ordner-unten-weitere-parameter")), mono(pal, vc_i18n::t!("setui-3-systemdateien-p1", p1 = self.data.join("sys").display()))].spacing(2).into()),
            self.args_editor(pal, "asm", &vc_i18n::t!("setui-assembler-optionen-beim-bauen"), vc_ide::args::catalog_for("vcc-asm"), vc_ide::args::ArgCtx::example("c64"), &vc_i18n::t!("setui-vcc-asm"), &vc_i18n::t!("setui-z-b-i-pfad-zu-includes"), &e.assembler.opts, &e.assembler.args, false),
            klein(pal, vc_i18n::t!("setui-gilt-fuer-das-bauen-in-der")),
        ]);
        let mut d = vec![];
        for s in vc_ide::targets::installed_systems(Some(&self.data)) {
            let dia = vc_ide_model::recent::target_dialect(&self.data, s.id).map(|x| vc_ide_model::recent::dialect_label(&x).to_string()).unwrap_or_else(|| "unbekannt".into());
            d.push(zeile(pal, s.name(), text(if s.id == "atari8" { format!("{dia} · {}", vc_i18n::t!("setui-auch-turbo-basic-xl")) } else { dia }).size(13).color(pal.text).into()));
        }
        d.push(klein(pal, vc_i18n::t!("setui-basic-bas-zeilennummern-optional-gibt-es")));
        let ex = karte(pal, vec![zeile(pal, vc_i18n::t!("setui-projekt"), text(if self.proj.name.is_empty() { vc_i18n::t!("setui-kein-projekt-geoeffnet").to_string() } else { vc_i18n::t!("setui-p1-auto-export-in-den-projekt", p1 = &self.proj.name) }).size(13).color(pal.text).into()), klein(pal, vc_i18n::t!("setui-gilt-je-projekt-und-steht-in"))]);
        column![abschnitt(pal, vc_i18n::t!("setui-assembler"), vc_i18n::t!("setui-vintagecoder-hat-einen-eigenen-assembler-es")), asm,
            abschnitt(pal, vc_i18n::t!("setui-basic-dialekt-je-zielsystem"), vc_i18n::t!("setui-das-basic-eines-projekts-folgt-dem")), karte(pal, d),
            abschnitt(pal, vc_i18n::t!("setui-export-beim-bauen"), vc_i18n::t!("setui-sprites-zeichensaetze-und-songs-werden-vor")), ex].spacing(8).into()
    }
    pub(crate) fn seite_editor(&self, pal: Pal) -> Element<'_, Msg> {
        let g = &self.einst;
        let k = karte(pal, vec![
            zeile(pal, &vc_gui_ui::texte::editor_schrift(), row![knopf(pal, "−", Some(Msg::SetSchrift(g.editor_schrift - 1.0)), pal.text, false), text(vc_i18n::t!("setui-p1-pt", p1 = vc_i18n::num((g.editor_schrift) as f64, 0))).size(13).color(pal.text), knopf(pal, "+", Some(Msg::SetSchrift(g.editor_schrift + 1.0)), pal.text, false)].spacing(10).align_y(Alignment::Center).into()),
            zeile(pal, &vc_gui_ui::texte::umbruch(), wahl(pal, &[("aus", &vc_i18n::t!("setui-aus")), ("an", &vc_i18n::t!("setui-an"))], if g.umbruch { "an" } else { "aus" }, |w| Msg::SetUmbruch(w == "an"))),
            zeile(pal, vc_i18n::t!("setui-menueleiste-anzeigen"), if cfg!(target_os = "macos") { klein(pal, vc_i18n::t!("setui-unter-macos-steht-die-menueleiste-des")) } else { wahl(pal, &[("an", &vc_i18n::t!("setui-an")), ("aus", &vc_i18n::t!("setui-aus"))], if g.menueleiste { "an" } else { "aus" }, |w| Msg::SetMenueleiste(w == "an")) }),
            zeile(pal, vc_i18n::t!("setui-zeile-beim-verlassen-formatieren"), wahl(pal, &[("an", &vc_i18n::t!("setui-an")), ("aus", &vc_i18n::t!("setui-aus"))], if g.pretty_zeile { "an" } else { "aus" }, |w| Msg::SetPretty(w == "an"))),
            klein(pal, vc_i18n::t!("setui-assembler-und-basic-leerraeume-und-kommentarspalte")),
            zeile(pal, vc_i18n::t!("setui-einzeltasten-der-studios"), wahl(pal, &[("an", &vc_i18n::t!("setui-an")), ("aus", &vc_i18n::t!("setui-aus"))], if g.studio_einzeltasten { "an" } else { "aus" }, |w| Msg::SetStudioTasten(w == "an"))),
            zeile(pal, vc_i18n::t!("setui-kuerzel-diagnose"), wahl(pal, &[("aus", &vc_i18n::t!("setui-aus")), ("an", &vc_i18n::t!("setui-an"))], if g.kuerzel_diagnose { "an" } else { "aus" }, |w| Msg::SetDiagnose(w == "an"))),
            klein(pal, vc_i18n::t!("setui-schreibt-zu-jedem-kuerzel-taste-umschalttasten")),
            zeile(pal, vc_i18n::t!("setui-tastaturlayout"), wahl(pal, &[("auto", &vc_i18n::t!("setui-automatisch")), ("qwertz", "QWERTZ"), ("qwerty", "QWERTY"), ("azerty", "AZERTY")], match g.tastaturlayout { crate::einstellungen::Tastaturlayout::Auto => "auto", crate::einstellungen::Tastaturlayout::Qwertz => "qwertz", crate::einstellungen::Tastaturlayout::Qwerty => "qwerty", crate::einstellungen::Tastaturlayout::Azerty => "azerty" }, |w| Msg::SetLayout(match w.as_str() { "qwertz" => crate::einstellungen::Tastaturlayout::Qwertz, "qwerty" => crate::einstellungen::Tastaturlayout::Qwerty, "azerty" => crate::einstellungen::Tastaturlayout::Azerty, _ => crate::einstellungen::Tastaturlayout::Auto }))),
            klein(pal, vc_i18n::t!("setui-die-klaviertasten-des-trackers-gelten-nach", p1 = crate::tastatur::name(self.layout()))),
            zeile(pal, vc_i18n::t!("setui-tasten-auf-der-klaviatur"), wahl(pal, &[("an", &vc_i18n::t!("setui-an")), ("aus", &vc_i18n::t!("setui-aus"))], if g.tasten_auf_klaviatur { "an" } else { "aus" }, |w| Msg::SetKlavierTasten(w == "an"))),
            klein(pal, vc_i18n::t!("setui-buchstaben-und-zifferntasten-in-den-studios")),
            zeile(pal, vc_i18n::t!("setui-startbild"), wahl(pal, &[("an", &vc_i18n::t!("setui-an")), ("aus", &vc_i18n::t!("setui-aus"))], if g.splash { "an" } else { "aus" }, |w| Msg::SetSplash(w == "an"))),
            zeile(pal, vc_i18n::t!("setui-takt-in-der-statusleiste"), wahl(pal, &[("aus", &vc_i18n::t!("setui-aus")), ("an", &vc_i18n::t!("setui-an"))], if g.takt_anzeige { "an" } else { "aus" }, |w| Msg::SetTakt(w == "an"))),
            klein(pal, vc_i18n::t!("setui-zeigt-unten-takt-norm-pal-ntsc")),
            zeile(pal, vc_i18n::t!("setui-sicherung"), wahl(pal, &[("an", &vc_i18n::t!("setui-unbenannte-reiter-nach-einem-absturz-wiederherstellen")), ("aus", &vc_i18n::t!("setui-aus"))], if self.satz.scratch_an { "an" } else { "aus" }, |w| Msg::Set(SetAkt::ScratchSicherung(w == "an")))),
            klein(pal, vc_i18n::t!("setui-der-inhalt-unbenannter-reiter-wird-alle")),
            klein(pal, vc_i18n::t!("setui-lange-zeilen-werden-an-der-sichtbaren")),
        ]);
        column![abschnitt(pal, vc_i18n::t!("setui-editor"), vc_i18n::t!("setui-gilt-fuer-alle-quelltexte-und-wirkt")), k].spacing(8).into()
    }
    fn seite_kuerzel(&self, pal: Pal) -> Element<'_, Msg> {
        let q = self.satz.kuerzel_suche.to_lowercase();
        let mut gruppen: Vec<(&str, Vec<(String, String)>)> = vec![];
        for i in befehle::BEFEHLE.iter().filter(|i| !i.tasten.is_empty()) {
            let t = befehle::fuer_anzeige(i.tasten, cfg!(target_os = "macos"));
            if !q.is_empty() && !i.name().to_lowercase().contains(&q) && !t.to_lowercase().contains(&q) && !i.menue.to_lowercase().contains(&q) { continue }
            match gruppen.iter_mut().find(|(m, _)| *m == i.menue) { Some((_, v)) => v.push((i.name(), t)), None => gruppen.push((i.menue, vec![(i.name(), t)])) }
        }
        let mut c = column![abschnitt(pal, vc_i18n::t!("setui-tastenkuerzel"), vc_i18n::t!("setui-die-kuerzel-sind-fest-anpassen-ist")),
            text_input(&vc_i18n::t!("setui-befehl-oder-kuerzel-suchen"), &self.satz.kuerzel_suche).on_input(|s| Msg::Set(SetAkt::KuerzelSuche(s))).size(13).padding(6).width(320)].spacing(8);
        if gruppen.is_empty() { c = c.push(klein(pal, vc_i18n::t!("setui-nichts-gefunden"))) }
        for (m, v) in gruppen {
            c = c.push(text(befehle::menue_titel(m)).size(12).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute));
            c = c.push(karte(pal, v.into_iter().map(|(n, t)| row![text(n.to_string()).size(13).color(pal.text).width(Length::Fill), text(t).size(12.5).font(vc_gui_ui::fonts::MONO).color(pal.text)].spacing(12).into()).collect()));
        }
        c.into()
    }
}
