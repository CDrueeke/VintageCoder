// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungsseite „ROMs“ (Stufe 4a): Status je System und Rolle, ROM-Ordner öffnen, neu prüfen, Dateien und ZIP-Archive ablegen oder wählen,
//! „ROMs anderer Programme mitbenutzen“, „ROMs aus VICE importieren“ und Festnageln einer Revision. Kopiert wird erst nach der Vorschau.
//! Der Kern (Suche, Erkennung, Import) steht in `vc_roms`; hier stehen Zustand, Aktionen und Ansicht. Im Testlauf zeigt der ROM-Dienst auf einen
//! Temp-Ordner (`VC_ROMS`-Ersatz), nie auf echte ROM-Orte; Dateidialoge laufen nur als `Task` und werden im Test nicht ausgeführt.
use crate::setui::{abschnitt, karte, klein, mono, SetAkt};
use crate::{flaeche, knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use vc_roms::import::{self, Action, ImportItem};
use vc_roms::{systems, RomEnv, RomStore, Status};

/// Gelesener ROM-Stand als teilbarer Wert (der Dienst ist nicht `Clone`)
#[derive(Clone)]
pub struct StandPtr(pub Arc<RomStore>);
impl std::fmt::Debug for StandPtr { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("RomStand") } }

#[derive(Clone, Debug)]
pub enum RomAkt {
    Geladen(StandPtr), Neu, System(usize), OrdnerOeffnen, Fremde(bool), Suchen, SucheAbbrechen, Gefunden(Vec<ImportItem>, bool),
    Waehlen, Gewaehlt(Option<PathBuf>), Abgelegt(PathBuf), Haken(usize, bool), Alle(bool), Kopieren, VorschauZu, Fest(String, String, Option<String>), Hilfe,
}

pub struct Vorschau { pub titel: String, pub items: Vec<(ImportItem, bool)> }

pub struct RomZ {
    pub stand: Option<Arc<RomStore>>, pub laedt: bool, pub system: usize, pub vorschau: Option<Vorschau>, pub meldung: Option<(bool, String)>, pub hilfe: bool,
    pub sucht: Option<Arc<AtomicBool>>, pub fremd_gelesen: bool,
    /// Test: eigener ROM-Ordner statt der echten Orte
    pub test_ordner: Option<PathBuf>,
}
impl Default for RomZ {
    fn default() -> Self { RomZ { stand: None, laedt: false, system: 0, vorschau: None, meldung: None, hilfe: false, sucht: None, fremd_gelesen: false, test_ordner: None } }
}

fn farbe(pal: Pal, s: Status) -> Color { match s { Status::Recognized => pal.ok, Status::UnknownRevision | Status::Replacement => pal.warn, Status::Missing | Status::Unusable => pal.err } }
/// Gesamtampel eines Systems: ok = alles Erforderliche als Original erkannt, warn = benutzbar mit Ersatz, err = Erforderliches fehlt
fn ampel(pal: Pal, store: &RomStore, system: &str) -> (Color, &'static str) {
    if !store.missing_required(system).is_empty() { return (pal.err, "erforderliche ROM fehlt") }
    let alle_original = systems::system(system).is_some_and(|sd| sd.roles.iter().filter(|r| r.required).all(|r| store.resolve(system, r.id).status == Status::Recognized)) && (system != "atari8" || store.resolve(system, "os").status == Status::Recognized);
    if alle_original { (pal.ok, "alles Erforderliche als Original erkannt") } else { (pal.warn, "benutzbar, aber Ersatz oder unbekannte Revision") }
}
fn kurz(sha: &str) -> String { format!("{}…", &sha[..12.min(sha.len())]) }
fn groessen(s: &[usize]) -> String { s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" / ") }
fn dateiname(p: &std::path::Path) -> String { p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }

impl App {
    pub(crate) fn rom_env(&self) -> RomEnv {
        let fremd = self.satz.gespeichert.roms.use_foreign;
        match &self.satz.roms.test_ordner {
            Some(d) => RomEnv { vc_roms: Some(d.clone()), use_foreign: false, data_dir: Some(self.data.clone()), ..Default::default() },
            None => { let mut e = vc_ide::roms::env(None, None); e.use_foreign = fremd; e }
        }
    }
    /// ROM-Dienst im Hintergrund einlesen (Orte können langsam sein, etwa Netzlaufwerke)
    pub(crate) fn rom_laden(&mut self) -> Task<Msg> {
        self.satz.roms.laedt = true; self.satz.roms.fremd_gelesen = self.satz.gespeichert.roms.use_foreign;
        let env = self.rom_env();
        Task::perform(async move { tokio::task::spawn_blocking(move || RomStore::new(env)).await.ok() }, |s| match s { Some(s) => Msg::Set(SetAkt::Rom(RomAkt::Geladen(StandPtr(Arc::new(s))))), None => Msg::Set(SetAkt::Rom(RomAkt::Gefunden(vec![], false))) })
    }
    fn rom_plan(&mut self, quellen: Vec<PathBuf>, titel: String) {
        let Some(st) = &self.satz.roms.stand else { return };
        let sys = systems::SYSTEMS[self.satz.roms.system].id;
        let items = import::plan(st, &quellen, Some(sys));
        self.satz.roms.vorschau = Some(Vorschau { titel, items: items.into_iter().map(|i| { let c = i.action == Action::Copy; (i, c) }).collect() });
    }
    /// Datei auf das Fenster gezogen, solange die Seite „ROMs“ offen ist: Vorschau (kopiert wird erst nach Bestätigung)
    pub(crate) fn rom_abgelegt(&mut self, p: PathBuf) { let _ = self.rom_akt(RomAkt::Abgelegt(p)); }
    pub(crate) fn rom_seite_offen(&self) -> bool { self.tabs.get(self.aktiv) == Some(&crate::Tab::Einstellungen) && self.satz.seite == crate::setui::Seite::Roms }

    pub(crate) fn rom_akt(&mut self, a: RomAkt) -> Task<Msg> {
        match a {
            RomAkt::Geladen(s) => { self.satz.roms.stand = Some(s.0); self.satz.roms.laedt = false }
            RomAkt::Neu => { if !self.satz.roms.laedt { return self.rom_laden() } }
            RomAkt::System(i) => self.satz.roms.system = i.min(systems::SYSTEMS.len() - 1),
            RomAkt::OrdnerOeffnen => { if let Some(d) = self.satz.roms.stand.as_ref().and_then(|s| s.env.install_dir()) { let _ = std::fs::create_dir_all(&d); crate::aussen::ausfuehren(crate::aussen::Aufruf::ImOrdnerZeigen(d.join("."))) } }
            RomAkt::Fremde(an) => {
                self.satz.einzeln_pub(|s| s.roms.use_foreign = an);
                if !cfg!(test) { vc_ide::roms::reconfigure_media() }
                return self.rom_laden()
            }
            RomAkt::Hilfe => self.satz.roms.hilfe = !self.satz.roms.hilfe,
            RomAkt::Suchen => {
                self.satz.roms.meldung = None; let stop = Arc::new(AtomicBool::new(false)); self.satz.roms.sucht = Some(stop.clone());
                let env = self.rom_env(); let flag = stop.clone();
                return Task::perform(async move { tokio::task::spawn_blocking(move || {
                    let t0 = std::time::Instant::now();
                    // der lokale Stand wird nur für „schon vorhanden“ und das Ziel gebraucht, fremde Orte hier nicht doppelt einlesen
                    let store = RomStore::new(RomEnv { use_foreign: false, ..env });
                    let halt = || flag.load(Ordering::Relaxed) || t0.elapsed() > std::time::Duration::from_secs(45);
                    let items = import::foreign_candidates_until(&store, &halt); (items, !halt())
                }).await.unwrap_or_default() }, |(i, v)| Msg::Set(SetAkt::Rom(RomAkt::Gefunden(i, v))))
            }
            RomAkt::SucheAbbrechen => { if let Some(s) = &self.satz.roms.sucht { s.store(true, Ordering::Relaxed) } }
            RomAkt::Gefunden(items, vollstaendig) => {
                if self.satz.roms.sucht.take().is_some() {
                    let titel = if items.is_empty() { if vollstaendig { "In anderen Programmen wurden keine bekannten ROMs gefunden".to_string() } else { "Suche abgebrochen, nichts gefunden".to_string() } } else { format!("{} ROM-Datei(en) in anderen Programmen gefunden{}", items.len(), if vollstaendig { "" } else { " (Suche vorzeitig beendet)" }) };
                    self.satz.roms.vorschau = Some(Vorschau { titel, items: items.into_iter().map(|i| { let c = i.action == Action::Copy; (i, c) }).collect() });
                }
            }
            RomAkt::Waehlen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("ROM-Datei wählen").pick_file().await.map(|f| f.path().to_path_buf()) }, |p| Msg::Set(SetAkt::Rom(RomAkt::Gewaehlt(p)))),
            RomAkt::Gewaehlt(p) => { if let Some(p) = p { self.rom_plan(vec![p], "Gewählte Datei".into()) } }
            RomAkt::Abgelegt(p) => { let n = 1; self.rom_plan(vec![p], format!("{n} Datei(en) abgelegt")) }
            RomAkt::Haken(i, an) => { if let Some((it, h)) = self.satz.roms.vorschau.as_mut().and_then(|v| v.items.get_mut(i)) { if it.action == Action::Copy { *h = an } } }
            RomAkt::Alle(an) => { if let Some(v) = &mut self.satz.roms.vorschau { for (it, h) in v.items.iter_mut() { if it.action == Action::Copy { *h = an } } } }
            RomAkt::VorschauZu => self.satz.roms.vorschau = None,
            RomAkt::Kopieren => {
                let Some(v) = self.satz.roms.vorschau.take() else { return Task::none() };
                let (mut ok, mut fehler) = (0, vec![]);
                for (it, h) in v.items.iter().filter(|(i, h)| *h && i.action == Action::Copy) { let _ = h; match import::execute(it) { Ok(_) => ok += 1, Err(e) => fehler.push(e) } }
                self.satz.roms.meldung = Some((fehler.is_empty(), if fehler.is_empty() { format!("{ok} ROM-Datei(en) kopiert.") } else { format!("{ok} kopiert, Fehler: {}", fehler.join("; ")) }));
                if !cfg!(test) { vc_media::romref::rescan_async() }
                return self.rom_laden()
            }
            RomAkt::Fest(s, r, f) => {
                let wurzel = self.satz.roms.stand.as_ref().and_then(|s| s.env.install_dir());
                match wurzel { Some(w) => match vc_roms::store::pin(&w, &s, &r, f.as_deref()) { Ok(()) => { self.satz.roms.meldung = Some((true, "Auswahl gespeichert (roms.json).".into())); return self.rom_laden() } Err(e) => self.satz.roms.meldung = Some((false, e)) }, None => self.satz.roms.meldung = Some((false, "Kein ROM-Ordner verfügbar.".into())) }
            }
        }
        Task::none()
    }

    pub(crate) fn seite_roms(&self, pal: Pal) -> Element<'_, Msg> {
        let z = &self.satz.roms;
        let Some(store) = &z.stand else { return column![abschnitt(pal, "ROMs", "Prüfe ROM-Orte …")].into() };
        let r = |a: RomAkt| Msg::Set(SetAkt::Rom(a));
        let dir = store.env.install_dir();
        let mut kopf = vec![row![text("ROM-Ordner").size(13).color(pal.text).width(170), mono(pal, dir.as_ref().map(|d| d.display().to_string()).unwrap_or_else(|| "–".into()))].spacing(12).into(),
            row![knopf(pal, "Ordner öffnen", dir.as_ref().map(|_| r(RomAkt::OrdnerOeffnen)), pal.text, false), knopf(pal, if z.laedt { "Prüfe …" } else { "Neu prüfen" }, (!z.laedt).then(|| r(RomAkt::Neu)), pal.text, false)].spacing(8).into()];
        let port: String = match &store.env.portable {
            Some(vc_roms::Portable::Active { dir, by_marker }) => format!("aktiv: {}{}", dir.display(), if *by_marker { " (Markierungsdatei „portable“)" } else { "" }),
            Some(vc_roms::Portable::Unavailable(w)) => format!("nicht verfügbar: {w}"),
            Some(vc_roms::Portable::Off { base: Some(b) }) => format!("aus · aktivieren: Ordner „roms“ oder Datei „portable“ in {} anlegen", b.display()),
            _ => "aus".into() };
        kopf.push(row![text("Portabler Modus").size(13).color(pal.text).width(170), klein(pal, port)].spacing(12).into());
        if store.env.vc_roms.is_some() && self.satz.roms.test_ordner.is_none() { kopf.push(text("Die Umgebungsvariable VC_ROMS ersetzt Benutzer- und Programmordner.").size(11.5).color(pal.warn).into()) }
        let mut orte = vec![klein(pal, "Gesuchte Orte")];
        for s in store.searched() { orte.push(row![text(s.stage.label()).size(11.5).color(if s.files > 0 { pal.ok } else { pal.mute }).width(120), mono(pal, s.path.display().to_string()), klein(pal, if s.exists { format!("{} Datei(en)", s.files) } else { "nicht vorhanden".into() })].spacing(8).wrap().into()) }
        if !store.env.use_foreign { orte.push(klein(pal, "ROM-Ordner anderer Programme werden nicht durchsucht (Schalter unten).")) }
        kopf.push(column(orte).spacing(2).into());
        let reiter = iced::widget::row(systems::SYSTEMS.iter().enumerate().map(|(i, sd)| {
            let (c, tipp) = ampel(pal, store, sd.id); let on = z.system == i;
            iced::widget::tooltip(button(text(format!("● {}", sd.label)).size(13).color(if on { pal.text } else { c })).padding([5, 10]).on_press(r(RomAkt::System(i))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }),
                container(text(tipp).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Bottom).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6);
        let sd = &systems::SYSTEMS[z.system.min(systems::SYSTEMS.len() - 1)];
        let mut rollen: Vec<Element<Msg>> = vec![];
        for rd in sd.roles.iter() {
            let rs = store.resolve(sd.id, rd.id);
            let atari_os = sd.id == "atari8" && systems::OS_ALTERNATIVES.contains(&rd.id);
            let noetig = rd.required || (atari_os && !store.resolve("atari8", "os").is_usable());
            let (st_text, st_farbe) = match rs.status { Status::Missing => (if noetig { "Fehlt" } else { "Fehlt (optional)" }, if noetig { pal.err } else { pal.mute }), s => (s.label(), farbe(pal, s)) };
            let datei: Element<Msg> = match &rs.path { Some(p) => mono(pal, dateiname(p)), None => klein(pal, format!("z. B. {}", rd.file)) };
            let mut knoepfe = row![knopf(pal, "Auswählen …", Some(r(RomAkt::Waehlen)), pal.text, false)].spacing(6);
            if !rs.others.is_empty() {
                // Festnageln: je Datei ein Knopf, dazu „Automatisch“
                let benutzt = rs.path.as_ref().map(|p| dateiname(p));
                for f in rs.path.iter().map(|p| (p.clone(), true)).chain(rs.others.iter().map(|f| (f.path.clone(), false))) {
                    let n = dateiname(&f.0); let aktiv = Some(&n) == benutzt.as_ref() && f.1;
                    knoepfe = knoepfe.push(button(text(format!("{}{n}", if aktiv { "✓ " } else { "" })).size(12).color(pal.text)).padding([4, 8]).on_press(r(RomAkt::Fest(sd.id.to_string(), rd.id.to_string(), Some(n)))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: if aktiv { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }));
                }
                knoepfe = knoepfe.push(knopf(pal, "Automatisch", Some(r(RomAkt::Fest(sd.id.to_string(), rd.id.to_string(), None))), pal.text, false));
            }
            let detail = match (&rs.path, &rs.crc32, &rs.sha1) {
                (Some(_), Some(c), Some(sha)) => { let mut t = vec![rs.place_label.clone().unwrap_or_default()]; if let Some(l) = &rs.label { t.push(l.clone()) } t.push(format!("{} Byte", rs.size.unwrap_or(0))); t.push(format!("CRC-32 {c:08X}")); t.push(format!("SHA-1 {}", kurz(sha))); if !rs.others.is_empty() { t.push(format!("+{} weitere Datei(en) derselben Rolle", rs.others.len())) } t.join("  ·  ") }
                _ => format!("Erwartet {} Byte", groessen(rd.sizes)) };
            rollen.push(column![row![text(st_text).size(12.5).color(st_farbe).width(140), text(rd.label).size(13).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).width(170), datei].spacing(10).align_y(Alignment::Center).wrap().vertical_spacing(4), knoepfe.wrap().vertical_spacing(4), klein(pal, detail)].spacing(3).into());
        }
        if sd.id == "atari8" { rollen.push(klein(pal, "Von den Betriebssystemen genügt eines; das erste vorhandene (XL/XE, dann 400/800 B, dann A) wird benutzt. Ohne ROM läuft atari800 mit seinem eingebauten Ersatz.")) }
        if sd.id == "mega65" { rollen.push(klein(pal, "Xemu braucht eine ROM-Datei von genau 131072 Byte; eine unbekannte Fassung wird mit Warnung benutzt.")) }
        let fremd_an = self.satz.gespeichert.roms.use_foreign;
        let mut fremd = vec![row![checkbox(fremd_an).label("ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800 …)").on_toggle(|an| Msg::Set(SetAkt::Rom(RomAkt::Fremde(an)))).size(16).text_size(13)].into(), klein(pal, "Nur lesend. Nichts wird kopiert, außer du importierst ausdrücklich.")];
        fremd.push(if z.sucht.is_some() { row![text("Suche in anderen Programmen läuft …").size(13).color(pal.text), knopf(pal, "Abbrechen", Some(r(RomAkt::SucheAbbrechen)), pal.text, false)].spacing(10).align_y(Alignment::Center).into() }
            else { row![knopf(pal, "ROMs aus VICE importieren …", Some(r(RomAkt::Suchen)), pal.text, false), knopf(pal, "Woher bekomme ich ROMs?", Some(r(RomAkt::Hilfe)), pal.text, false)].spacing(8).wrap().vertical_spacing(6).into() });
        if z.hilfe { fremd.push(klein(pal, "• Aus dem eigenen Gerät auslesen (EPROM-Programmer, Kartuschen- und Dump-Werkzeuge).\n• Käuflich mit Lizenz, z. B. als Sammlung eines Anbieters, der die Systemdateien lizenziert.\n• Freie Ersatz-ROMs: Open-ROMs (C64, MEGA65), Altirra (Atari). VintageCoder liefert keine Original-ROMs mit.")) }
        let mut spalte = column![abschnitt(pal, "ROMs", "Deine eigenen ROM-Abbilder. VintageCoder liefert keine Original-ROMs mit; erkannt wird am Inhalt (Prüfsumme), der Dateiname ist egal."), karte(pal, kopf), reiter, karte(pal, rollen),
            klein(pal, format!("ROM-Dateien oder ZIP-Archive lassen sich auf das Fenster ziehen (für {} bevorzugt) oder mit „Auswählen …“ wählen; sie kommen erst nach Bestätigung in den ROM-Ordner.", sd.label)), karte(pal, fremd)].spacing(8);
        if let Some(v) = &z.vorschau { spalte = spalte.push(self.rom_vorschau(pal, v)) }
        if let Some((gut, m)) = &z.meldung { spalte = spalte.push(text(m.clone()).size(12.5).color(if *gut { pal.ok } else { pal.err })) }
        spalte.into()
    }

    fn rom_vorschau<'a>(&'a self, pal: Pal, v: &'a Vorschau) -> Element<'a, Msg> {
        let r = |a: RomAkt| Msg::Set(SetAkt::Rom(a));
        let n = v.items.iter().filter(|(i, h)| *h && i.action == Action::Copy).count(); let kopierbar = v.items.iter().filter(|(i, _)| i.action == Action::Copy).count();
        let mut z = row![knopf(pal, &format!("{n} Datei(en) kopieren"), (n > 0).then(|| r(RomAkt::Kopieren)), pal.text, true), knopf(pal, "Abbrechen", Some(r(RomAkt::VorschauZu)), pal.text, false)].spacing(8);
        if kopierbar > 0 { z = z.push(knopf(pal, "Alle abwählen", Some(r(RomAkt::Alle(false))), pal.text, false)).push(knopf(pal, "Alle wählen", Some(r(RomAkt::Alle(true))), pal.text, false)) }
        let mut liste = column![].spacing(6);
        if v.items.is_empty() { liste = liste.push(klein(pal, "Nichts zu übernehmen.")) }
        // nach System gruppiert (Reihenfolge der Systeme), innerhalb stabil
        let mut ord: Vec<usize> = (0..v.items.len()).collect();
        let rang = |s: &Option<String>| systems::SYSTEMS.iter().position(|d| Some(d.id) == s.as_deref()).unwrap_or(usize::MAX);
        ord.sort_by_key(|&i| rang(&v.items[i].0.system));
        let mut letzte: Option<Option<String>> = None;
        for i in ord {
            let (it, an) = &v.items[i];
            if letzte.as_ref() != Some(&it.system) { liste = liste.push(text(it.system.as_deref().and_then(systems::system).map(|d| d.label).unwrap_or("Sonstige").to_string()).size(12).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute)); letzte = Some(it.system.clone()) }
            let rolle = it.role.as_deref().and_then(|x| systems::role(it.system.as_deref().unwrap_or(""), x)).map(|x| x.label).unwrap_or("?");
            let name = it.data.as_ref().and_then(|_| it.dest.as_ref().map(|d| dateiname(d))).unwrap_or_else(|| dateiname(&it.src));
            let (status, sf) = match &it.action { Action::Copy => if it.label.is_some() { ("Erkannt", pal.ok) } else { ("Unbekannte Revision", pal.warn) }, Action::AlreadyPresent(_) => ("Schon vorhanden", pal.mute), Action::Skip(_) => ("Übersprungen", pal.mute) };
            let notiz = match &it.action { Action::Skip(w) => w.clone(), _ => format!("{}{}", it.label.clone().unwrap_or_default(), it.source_label.as_ref().map(|s| format!("  [{s}]")).unwrap_or_default()) };
            let haken: Element<Msg> = if it.action == Action::Copy { checkbox(*an).on_toggle(move |b| r(RomAkt::Haken(i, b))).size(16).into() } else { iced::widget::space::horizontal().width(16).into() };
            liste = liste.push(row![haken, text(status).size(12).color(sf).width(130), text(rolle).size(12.5).color(pal.text).width(130), mono(pal, name), klein(pal, notiz)].spacing(8).align_y(Alignment::Center).wrap().vertical_spacing(2));
        }
        karte(pal, vec![text(v.titel.clone()).size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), z.wrap().vertical_spacing(6).into(), klein(pal, "Ziel: ROM-Ordner; vorhandene Dateien werden nie überschrieben."), scrollable(liste).height(Length::Fixed(260.0)).into()])
    }
}
