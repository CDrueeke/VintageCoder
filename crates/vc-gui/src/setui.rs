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
pub const SEITEN: [(Seite, &str); 6] = [(Seite::Allgemein, "Allgemein"), (Seite::Ziele, "Zielsysteme und Hardware"), (Seite::Roms, "ROMs"), (Seite::Toolchain, "Toolchain"), (Seite::Editor, "Editor"), (Seite::Kuerzel, "Tastenkürzel")];

#[derive(Clone, Debug)]
pub enum SetAkt {
    Seite(Seite), Speichern, Verwerfen,
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
    pub fn speichern(&mut self) { let s = self.entwurf.clone(); self.meldung = Some(match self.schreiben(s.clone()) { Ok(()) => { self.gespeichert = s; (true, "Gespeichert.".into()) } Err(e) => (false, format!("Speichern fehlgeschlagen: {e}")) }) }
    /// Nur diese Änderung speichern, ohne anderen ungespeicherten Entwurf mitzunehmen
    pub fn einzeln_pub(&mut self, f: impl Fn(&mut Settings)) { self.einzeln(f) }
    fn einzeln(&mut self, f: impl Fn(&mut Settings)) {
        let mut g = self.gespeichert.clone(); f(&mut g); f(&mut self.entwurf);
        self.meldung = match self.schreiben(g.clone()) { Ok(()) => { self.gespeichert = g; None } Err(e) => Some((false, format!("Speichern fehlgeschlagen: {e}"))) };
    }
    pub fn akt(&mut self, a: SetAkt) {
        match a {
            SetAkt::Seite(s) => { self.seite = s; self.meldung = None }
            SetAkt::Speichern => self.speichern(),
            SetAkt::Verwerfen => { self.entwurf = self.gespeichert.clone(); self.meldung = None }
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
pub(crate) fn abschnitt<'a>(pal: Pal, titel: &str, hinweis: &str) -> Element<'a, Msg> {
    column![text(titel.to_string()).size(15).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(hinweis.to_string()).size(12).color(pal.mute)].spacing(3).into()
}
pub(crate) fn karte<'a>(pal: Pal, inhalt: Vec<Element<'a, Msg>>) -> Element<'a, Msg> {
    container(column(inhalt).spacing(10)).padding(14).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
}
fn zeile<'a>(pal: Pal, titel: &str, inhalt: Element<'a, Msg>) -> Element<'a, Msg> {
    row![text(titel.to_string()).size(13).color(pal.text).width(170), container(inhalt).width(Length::Fill)].spacing(12).align_y(Alignment::Start).into()
}
pub(crate) fn klein<'a>(pal: Pal, s: impl Into<String>) -> Element<'a, Msg> { text(s.into()).size(11.5).color(pal.mute).into() }
pub(crate) fn mono<'a>(pal: Pal, s: impl Into<String>) -> Element<'a, Msg> { text(s.into()).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute).into() }
fn wahl<'a>(pal: Pal, optionen: &[(&str, &str)], aktuell: &str, m: impl Fn(String) -> Msg) -> Element<'a, Msg> {
    row(optionen.iter().map(|(w, l)| { let on = *w == aktuell; button(text(l.to_string()).size(13).color(pal.text)).padding([5, 12]).on_press(m(w.to_string())).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6).into()
}
pub(crate) fn feld_klein<'a>(pal: Pal, hinweis: &str, wert: &str, m: impl Fn(String) -> Msg + 'a) -> Element<'a, Msg> {
    let _ = pal; text_input(hinweis, wert).on_input(m).size(13).padding(6).width(Length::Fill).into()
}

impl App {
    /// Aktion der Einstellungsseite; die Seite „ROMs“ liest ihren Stand beim ersten Öffnen im Hintergrund
    pub(crate) fn set_akt(&mut self, a: SetAkt) -> Task<Msg> {
        match a {
            SetAkt::Rom(r) => self.rom_akt(r),
            SetAkt::Ziel(a) => self.ziel_akt(a),
            SetAkt::Arg(x) => { self.arg_akt(x); Task::none() }
            SetAkt::Seite(Seite::Ziele) => { self.satz.akt(SetAkt::Seite(Seite::Ziele)); self.ziel_oeffnen() }
            SetAkt::Seite(Seite::Roms) => { self.satz.akt(SetAkt::Seite(Seite::Roms)); if self.satz.roms.stand.is_none() && !self.satz.roms.laedt { self.rom_laden() } else { Task::none() } }
            a => { self.satz.akt(a); Task::none() }
        }
    }
    pub(crate) fn einstellungen_ansicht(&self, pal: Pal) -> Element<'_, Msg> {
        let z = &self.satz; let sa = |s: Seite| Msg::Set(SetAkt::Seite(s));
        let reiter = row(SEITEN.iter().map(|(s, l)| { let on = z.seite == *s; button(text(*l).size(13).color(pal.text)).padding([6, 12]).on_press(sa(*s)).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6);
        let inhalt: Element<Msg> = match z.seite { Seite::Allgemein => self.seite_allgemein(pal), Seite::Toolchain => self.seite_toolchain(pal), Seite::Editor => self.seite_editor(pal), Seite::Kuerzel => self.seite_kuerzel(pal), Seite::Roms => self.seite_roms(pal), Seite::Ziele => self.seite_ziele(pal) };
        let fuss: Element<Msg> = if z.geaendert() || z.meldung.is_some() {
            let msg = z.meldung.clone().map(|(ok, t)| text(t).size(12.5).color(if ok { pal.ok } else { pal.err }));
            let mut r = row![].spacing(10).align_y(Alignment::Center);
            if z.geaendert() { r = r.push(text("● Ungespeicherte Änderungen").size(12.5).color(pal.warn)).push(space::horizontal()).push(knopf(pal, "Verwerfen", Some(Msg::Set(SetAkt::Verwerfen)), pal.text, false)).push(knopf(pal, "Speichern", Some(Msg::Set(SetAkt::Speichern)), pal.text, true)) }
            if let Some(m) = msg { r = r.push(m) }
            container(r.wrap().vertical_spacing(6)).padding([8, 14]).width(Length::Fill).style(move |_| flaeche(pal.panel, pal.line)).into()
        } else { space::vertical().height(0).into() };
        column![container(column![text(vc_gui_ui::texte::EINSTELLUNGEN).size(20).color(pal.text), reiter].spacing(10)).padding([14, 20]).width(Length::Fill), scrollable(container(inhalt).padding([6, 20]).max_width(1000)).height(Length::Fill), fuss].into()
    }
    fn seite_allgemein(&self, pal: Pal) -> Element<'_, Msg> {
        let (e, g) = (&self.satz.entwurf, &self.einst);
        let sc = g.schema;
        let darst = karte(pal, vec![
            zeile(pal, vc_gui_ui::texte::FARBSCHEMA, row![
                self.seg(pal, vc_gui_ui::texte::DUNKEL, sc == crate::einstellungen::Schema::Dunkel, Msg::SetSchema(crate::einstellungen::Schema::Dunkel)), self.seg(pal, vc_gui_ui::texte::HELL, sc == crate::einstellungen::Schema::Hell, Msg::SetSchema(crate::einstellungen::Schema::Hell)),
                self.seg(pal, vc_gui_ui::texte::SYSTEM, sc == crate::einstellungen::Schema::System, Msg::SetSchema(crate::einstellungen::Schema::System)), self.seg(pal, vc_gui_ui::texte::KONTRAST, sc == crate::einstellungen::Schema::Kontrast, Msg::SetSchema(crate::einstellungen::Schema::Kontrast))].spacing(6).wrap().vertical_spacing(6).into()),
            zeile(pal, vc_gui_ui::texte::OBERFLAECHE, row![knopf(pal, "−", Some(Msg::SetZoom(g.zoom - 0.1)), pal.text, false), text(format!("{:.0} %", g.zoom * 100.0)).size(13).color(pal.text), knopf(pal, "+", Some(Msg::SetZoom(g.zoom + 0.1)), pal.text, false)].spacing(10).align_y(Alignment::Center).into()),
            klein(pal, &befehle::plattform_text("Größe der ganzen Oberfläche; auch mit Cmd + und −, zurück mit Cmd 0.")),
        ]);
        let letzter = vc_ide::setup::Paths::current().map(|p| vc_ide::setup::WizardState::load(&p.wizard).last_run).filter(|d| !d.is_empty());
        let konfig = karte(pal, vec![zeile(pal, "Konfiguration", row![knopf(pal, "Exportieren …", Some(Msg::KonfigWahl(crate::konfig::KArt::Export)), pal.text, false), knopf(pal, "Importieren …", Some(Msg::KonfigWahl(crate::konfig::KArt::Import)), pal.text, false), knopf(pal, "Zurücksetzen …", Some(Msg::KonfigResetFrage), pal.text, false)].spacing(8).wrap().vertical_spacing(8).into()),
            klein(pal, "Exportieren schreibt die Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen, Parameter) in eine JSON-Datei, ohne Passwörter. Importieren zeigt vorher, was drinsteht, und ersetzt dann die jetzigen Einstellungen; Zurücksetzen stellt die Vorgabe her. Farbschema und Zoom gehören nicht dazu.")]);
        let einr = karte(pal, vec![zeile(pal, "Assistent", row![knopf(pal, "Assistent starten", Some(Msg::Befehl(vc_gui_ui::befehle::Cmd::Assistent)), pal.text, false), klein(pal, match letzter { Some(d) => format!("zuletzt ausgeführt: {d}"), None => "noch nicht ausgeführt".into() })].spacing(10).align_y(Alignment::Center).wrap().into())]);
        let env_log = std::env::var("VC_DEBUG_LOG").ok().filter(|v| !v.trim().is_empty());
        let an = if env_log.is_some() { vc_ide::debuglog::enabled() } else { e.debug_log };
        let mut f = vec![zeile(pal, "Debug-Protokoll", wahl(pal, &[("an", "Protokolldatei schreiben"), ("aus", "Aus")], if an { "an" } else { "aus" }, |w| Msg::Set(SetAkt::DebugLog(w == "an"))))];
        if let Some(v) = &env_log { f.push(text(format!("Umgebungsvariable VC_DEBUG_LOG überstimmt die Einstellung: {v}")).size(11.5).color(pal.warn).into()) }
        if let Some(p) = vc_ide::debuglog::path() { f.push(mono(pal, format!("Datei: {}", p.display()))) }
        f.push(klein(pal, "Pfade im persönlichen Ordner stehen als ~, Passwörter und Tokens werden geschwärzt. Dateinamen und Fehlermeldungen deines Projekts können vorkommen: vor dem Weitergeben ansehen. Ab 5 MB beginnt eine neue Datei."));
        let fehler = karte(pal, f);
        let bilder = if e.help.images.is_empty() { "ask" } else { e.help.images.as_str() };
        let mut h = vec![zeile(pal, "Bilder laden", wahl(pal, &[("ask", "Fragen"), ("always", "Immer"), ("never", "Nie")], bilder, |w| Msg::Set(SetAkt::HilfeBilder(w)))), zeile(pal, "Adresse", feld_klein(pal, vc_ide::help_images::DEFAULT_BASE, &e.help.images_url, |s| Msg::Set(SetAkt::HilfeUrl(s))))];
        if let Ok(v) = std::env::var("VC_HELP_IMAGES") { h.push(text(format!("Umgebungsvariable VC_HELP_IMAGES überstimmt die Adresse: {v}")).size(11.5).color(pal.warn).into()) }
        h.push(klein(pal, "Ein Ordner statt einer Webadresse wird gelesen statt geladen. Beim Laden wird nur die Adresse des Bildes abgerufen; geladene Bilder bleiben im Zwischenspeicher."));
        if let Some(d) = vc_ide::help_images::cache_dir() { h.push(mono(pal, format!("Zwischenspeicher: {}", d.display()))) }
        let hilfe = karte(pal, h);
        let mut spalte = column![abschnitt(pal, "Darstellung", "Gilt für die ganze Oberfläche und wirkt sofort."), darst, abschnitt(pal, "Konfiguration sichern", "Einstellungen in eine Datei schreiben, von dort übernehmen oder auf die Vorgabe zurücksetzen."), konfig, abschnitt(pal, "Einrichtung", "Der Konfigurationsassistent prüft die Einrichtung und ändert nur, was du bestätigst."), einr,
            abschnitt(pal, "Fehlersuche", "Das Debug-Protokoll schreibt Befehle, Meldungen, Fehler und Abstürze in eine Datei auf deinem Rechner; nichts wird gesendet."), fehler,
            abschnitt(pal, "Hilfe", "Die Bilder des Handbuchs liegen im Internet und werden erst nach deiner Zustimmung geladen; die Wahl wirkt sofort und wird gespeichert."), hilfe].spacing(8);
        if vc_ide::features::features().update() {
            let u = &e.updates; let v = if u.check.is_empty() { "" } else { u.check.as_str() };
            let mut k = vec![zeile(pal, "Nach Updates suchen", wahl(pal, &[("never", "Nur auf Wunsch"), ("daily", "Täglich beim Start"), ("weekly", "Wöchentlich beim Start")], v, |w| Msg::Set(SetAkt::UpdPruefen(w))))];
            if u.check.is_empty() { k.push(text("Noch nicht entschieden: es wird nichts gesendet.").size(11.5).color(pal.warn).into()) }
            k.push(zeile(pal, "Kanal", text("Stabil (nur fertige Versionen)").size(13).color(pal.text).into()));
            k.push(zeile(pal, "Jetzt prüfen", row![knopf(pal, if self.upd.laeuft { "Prüfe …" } else { "Jetzt prüfen" }, (!self.upd.laeuft).then(|| Msg::Befehl(vc_gui_ui::befehle::Cmd::UpdatePruefen)), pal.text, false), klein(pal, if self.upd.stand.last_result.is_empty() { "noch nicht geprüft".to_string() } else { self.upd.stand.last_result.clone() })].spacing(10).align_y(Alignment::Center).wrap().into()));
            if !u.skipped.is_empty() { k.push(zeile(pal, "Übersprungen", row![text(format!("Version {}", u.skipped)).size(13).color(pal.text), knopf(pal, "Zurücknehmen", Some(Msg::Set(SetAkt::UpdSkipZurueck)), pal.text, false)].spacing(10).align_y(Alignment::Center).into())) }
            k.push(zeile(pal, "Adresse", feld_klein(pal, vc_ide::update::FEED_URL, &u.manifest_url, |s| Msg::Set(SetAkt::UpdUrl(s)))));
            if let Ok(v) = std::env::var("VC_UPDATE_URL") { k.push(text(format!("Umgebungsvariable VC_UPDATE_URL überstimmt die Adresse: {v}")).size(11.5).color(pal.warn).into()) }
            k.push(klein(pal, "Die Adresse betrifft nur die Versionsliste (https oder ein Ordner); der Link zur Releaseseite ist im Programm fest. Übertragen wird eine Anfrage nach der Versionsliste (update.json) an github.com: GitHub sieht deine IP-Adresse und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Keine Kennung, kein Konto, keine Angaben zu Rechner, Projekten oder Einstellungen."));
            spalte = spalte.push(abschnitt(pal, "Updates", "VintageCoder kann nachsehen, ob es eine neue Version gibt, und zeigt die Änderungen mit einem Link zur Releaseseite. Es lädt und installiert nichts selbst. Ohne deine Zustimmung geht nie etwas ins Netz.")).push(karte(pal, k));
        }
        let dir = vc_core::userdirs::config_dir().map(|d| d.display().to_string()).unwrap_or_else(|| "–".into());
        spalte = spalte.push(abschnitt(pal, "Dateien", "Wo VintageCoder Einstellungen ablegt.")).push(karte(pal, vec![zeile(pal, "Ordner", mono(pal, dir)), zeile(pal, "Einstellungen", mono(pal, vc_ide::settings::file().map(|f| f.display().to_string()).unwrap_or_else(|| "–".into()))),
            klein(pal, "Die Einstellungsdatei enthält die Passwörter der Hardware und ist nur für dich lesbar."), row![knopf(pal, "Im Ordner zeigen", Some(Msg::Set(SetAkt::OrdnerZeigen)), pal.text, false)].into()]));
        spalte.padding([0, 0]).into()
    }
    fn seg(&self, pal: Pal, name: &'static str, on: bool, m: Msg) -> Element<'_, Msg> {
        button(text(name).size(13).color(pal.text)).padding([5, 12]).on_press(m).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into()
    }
    fn seite_toolchain(&self, pal: Pal) -> Element<'_, Msg> {
        let e = &self.satz.entwurf;
        let asm = karte(pal, vec![
            zeile(pal, "Assembler", row![text("eingebaut (vc-asm)").size(13).color(pal.text)].into()),
            zeile(pal, "Include-Suche", column![klein(pal, "1. Ordner des Projekts"), klein(pal, "2. Zusätzliche Ordner (unten: weitere Parameter, -I)"), mono(pal, format!("3. Systemdateien: {}", self.data.join("sys").display()))].spacing(2).into()),
            self.args_editor(pal, "asm", "Assembler-Optionen beim Bauen", vc_ide::args::catalog_for("vcc-asm"), vc_ide::args::ArgCtx::example("c64"), "vcc asm", "z. B. -I /pfad/zu/includes -D DEBUG=1", &e.assembler.opts, &e.assembler.args, false),
            klein(pal, "Gilt für das Bauen in der IDE (nicht für vcc auf der Kommandozeile). Konstanten aus der Projektdatei kommen zuerst, diese danach."),
        ]);
        let mut d = vec![];
        for s in vc_ide::targets::installed_systems(Some(&self.data)) {
            let dia = vc_ide_model::recent::target_dialect(&self.data, s.id).map(|x| vc_ide_model::recent::dialect_label(&x).to_string()).unwrap_or_else(|| "unbekannt".into());
            d.push(zeile(pal, s.name, text(if s.id == "atari8" { format!("{dia} · auch Turbo BASIC XL") } else { dia }).size(13).color(pal.text).into()));
        }
        d.push(klein(pal, "BASIC (.bas, Zeilennummern optional) gibt es für alle Dialekte."));
        let ex = karte(pal, vec![zeile(pal, "Projekt", text(if self.proj.name.is_empty() { "kein Projekt geöffnet".to_string() } else { format!("{} (Auto-Export in den Projekt-Einstellungen)", self.proj.name) }).size(13).color(pal.text).into()), klein(pal, "Gilt je Projekt und steht in dessen Einstellungen.")]);
        column![abschnitt(pal, "Assembler", "VintageCoder hat einen eigenen Assembler; es wird kein externer Assembler benötigt."), asm,
            abschnitt(pal, "BASIC-Dialekt je Zielsystem", "Das BASIC eines Projekts folgt dem Dialekt seines Zielsystems; je Quelle lässt er sich in der Projektdatei ändern."), karte(pal, d),
            abschnitt(pal, "Export beim Bauen", "Sprites, Zeichensätze und Songs werden vor dem Assemblieren nach build/export/ geschrieben, wenn sie veraltet sind."), ex].spacing(8).into()
    }
    pub(crate) fn seite_editor(&self, pal: Pal) -> Element<'_, Msg> {
        let g = &self.einst;
        let k = karte(pal, vec![
            zeile(pal, vc_gui_ui::texte::EDITOR_SCHRIFT, row![knopf(pal, "−", Some(Msg::SetSchrift(g.editor_schrift - 1.0)), pal.text, false), text(format!("{:.0} pt", g.editor_schrift)).size(13).color(pal.text), knopf(pal, "+", Some(Msg::SetSchrift(g.editor_schrift + 1.0)), pal.text, false)].spacing(10).align_y(Alignment::Center).into()),
            zeile(pal, vc_gui_ui::texte::UMBRUCH, wahl(pal, &[("aus", "Aus"), ("an", "An")], if g.umbruch { "an" } else { "aus" }, |w| Msg::SetUmbruch(w == "an"))),
            zeile(pal, "Menüleiste anzeigen", if cfg!(target_os = "macos") { klein(pal, "Unter macOS steht die Menüleiste des Systems oben am Bildschirm.") } else { wahl(pal, &[("an", "An"), ("aus", "Aus")], if g.menueleiste { "an" } else { "aus" }, |w| Msg::SetMenueleiste(w == "an")) }),
            zeile(pal, "Zeile beim Verlassen formatieren", wahl(pal, &[("an", "An"), ("aus", "Aus")], if g.pretty_zeile { "an" } else { "aus" }, |w| Msg::SetPretty(w == "an"))),
            klein(pal, "Assembler und BASIC: Leerräume und Kommentarspalte werden aufgeräumt, sobald du die Zeile verlässt (nie bei Auswahl). Auf Befehl: Bearbeiten ▸ Zeile / Datei formatieren."),
            zeile(pal, "Einzeltasten der Studios", wahl(pal, &[("an", "An"), ("aus", "Aus")], if g.studio_einzeltasten { "an" } else { "aus" }, |w| Msg::SetStudioTasten(w == "an"))),
            zeile(pal, "Kürzel-Diagnose", wahl(pal, &[("aus", "Aus"), ("an", "An")], if g.kuerzel_diagnose { "an" } else { "aus" }, |w| Msg::SetDiagnose(w == "an"))),
            klein(pal, "Schreibt zu jedem Kürzel (Taste, Umschalttasten, gefundener Befehl, Menü-Auslösung) eine Zeile ins Protokoll. Nur lokal, nichts wird übertragen."),
            zeile(pal, "Tastaturlayout", wahl(pal, &[("auto", "Automatisch"), ("qwertz", "QWERTZ"), ("qwerty", "QWERTY"), ("azerty", "AZERTY")], match g.tastaturlayout { crate::einstellungen::Tastaturlayout::Auto => "auto", crate::einstellungen::Tastaturlayout::Qwertz => "qwertz", crate::einstellungen::Tastaturlayout::Qwerty => "qwerty", crate::einstellungen::Tastaturlayout::Azerty => "azerty" }, |w| Msg::SetLayout(match w.as_str() { "qwertz" => crate::einstellungen::Tastaturlayout::Qwertz, "qwerty" => crate::einstellungen::Tastaturlayout::Qwerty, "azerty" => crate::einstellungen::Tastaturlayout::Azerty, _ => crate::einstellungen::Tastaturlayout::Auto }))),
            klein(pal, &format!("Die Klaviertasten des Trackers gelten nach der Position der Taste (wie bei US-QWERTY); das Layout bestimmt nur, welche Buchstaben die Hinweise nennen. Jetzt erkannt: {}.", crate::tastatur::name(self.layout()))),
            zeile(pal, "Tasten auf der Klaviatur", wahl(pal, &[("an", "An"), ("aus", "Aus")], if g.tasten_auf_klaviatur { "an" } else { "aus" }, |w| Msg::SetKlavierTasten(w == "an"))),
            klein(pal, "Buchstaben- und Zifferntasten in den Studios (D, E, F … für Werkzeuge, Z S X … für Noten, 1–4 für Zeichenfarben). Aus: Sie tippen nichts mehr versehentlich; Pfeile, Leertaste, Entf, Esc und alle Kürzel mit Cmd oder Alt bleiben, die Werkzeuge wählen Sie dann mit der Maus oder über die Befehlspalette."),
            zeile(pal, "Startbild", wahl(pal, &[("an", "An"), ("aus", "Aus")], if g.splash { "an" } else { "aus" }, |w| Msg::SetSplash(w == "an"))),
            zeile(pal, "Takt in der Statusleiste", wahl(pal, &[("aus", "Aus"), ("an", "An")], if g.takt_anzeige { "an" } else { "aus" }, |w| Msg::SetTakt(w == "an"))),
            klein(pal, "Zeigt unten Takt, Norm (PAL/NTSC), Zeilen je Bild und Zyklen je Zeile des Zielsystems, beim Debuggen auch den Programmzähler. Nur eine Anzeige aus den Systemdaten: nichts wird erhoben oder gesendet."),
            zeile(pal, "Sicherung", wahl(pal, &[("an", "Unbenannte Reiter nach einem Absturz wiederherstellen"), ("aus", "Aus")], if self.satz.scratch_an { "an" } else { "aus" }, |w| Msg::Set(SetAkt::ScratchSicherung(w == "an")))),
            klein(pal, "Der Inhalt unbenannter Reiter wird alle paar Sekunden in scratch.json im Konfigurationsordner gesichert: nur auf diesem Rechner, nie im Projekt, unverschlüsselt, nach 14 Tagen gelöscht. Beim Abschalten verschwindet die Sicherung sofort."),
            klein(pal, "Lange Zeilen werden an der sichtbaren Breite umbrochen (nur Darstellung). Der Text bleibt unverändert; Zeilennummern und Haltepunkte gehören zur ersten Zeile."),
        ]);
        column![abschnitt(pal, "Editor", "Gilt für alle Quelltexte und wirkt sofort."), k].spacing(8).into()
    }
    fn seite_kuerzel(&self, pal: Pal) -> Element<'_, Msg> {
        let q = self.satz.kuerzel_suche.to_lowercase();
        let mut gruppen: Vec<(&str, Vec<(&str, String)>)> = vec![];
        for i in befehle::BEFEHLE.iter().filter(|i| !i.tasten.is_empty()) {
            let t = befehle::fuer_anzeige(i.tasten, cfg!(target_os = "macos"));
            if !q.is_empty() && !i.name.to_lowercase().contains(&q) && !t.to_lowercase().contains(&q) && !i.menue.to_lowercase().contains(&q) { continue }
            match gruppen.iter_mut().find(|(m, _)| *m == i.menue) { Some((_, v)) => v.push((i.name, t)), None => gruppen.push((i.menue, vec![(i.name, t)])) }
        }
        let mut c = column![abschnitt(pal, "Tastenkürzel", "Die Kürzel sind fest; Anpassen ist noch nicht möglich. Die Liste kommt aus der Befehlstabelle und gilt auch für Menü und Palette."),
            text_input("Befehl oder Kürzel suchen", &self.satz.kuerzel_suche).on_input(|s| Msg::Set(SetAkt::KuerzelSuche(s))).size(13).padding(6).width(320)].spacing(8);
        if gruppen.is_empty() { c = c.push(klein(pal, "Nichts gefunden.")) }
        for (m, v) in gruppen {
            c = c.push(text(m.to_string()).size(12).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute));
            c = c.push(karte(pal, v.into_iter().map(|(n, t)| row![text(n.to_string()).size(13).color(pal.text).width(Length::Fill), text(t).size(12.5).font(vc_gui_ui::fonts::MONO).color(pal.text)].spacing(12).into()).collect()));
        }
        c.into()
    }
}
