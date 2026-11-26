// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungsseite „Zielsysteme und Hardware“ (Stufe 4a): links die Zielsysteme nach Hersteller (Filter, hinzufügen, deaktivieren), rechts Kopf mit
//! Status und Startziel, die Emulatorkarten (Pfad, automatische Suche, ROM, Standard, Test-Start), echte Hardware (Ultimate, MEGA65 mit
//! Verbindungstest und Port-Scan) und die Fujisan-Karte. Bearbeitet wird der Entwurf (`Satz::entwurf`); Speichern macht der Fuß der Einstellungen.
//! Prozessstarts, Verbindungstests, Port-Suche und Downloads laufen nur über `aussen` (im Test aufgezeichnet, nie ausgeführt); geladen wird nur nach Bestätigung.
use crate::setui::{abschnitt, karte, klein, SetAkt};
use crate::{aussen, flaeche, knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use vc_ide::platform::{self, Presence};
use vc_ide::targets::{self, EmulatorCfg};

#[derive(Clone, Debug, PartialEq)]
pub enum Probe { Laeuft, Ok(String), Fehler(String) }

#[derive(Clone, Debug)]
pub enum Feld { Name, Host, Passwort, Geraet, Baud, EthSchnittstelle, Etherload, FtpTool, SdOrdner }
#[derive(Clone, Copy, Debug)]
pub enum Haken { RunNach, ResetVor }

#[derive(Clone, Debug)]
pub enum ZielAkt {
    Wahl(String), Filter(String), Hinzu(String), Weg,
    StartZiel(String),
    EmuPlus(String), EmuName(String, String), EmuArt(String, String), EmuPfad(String, String), EmuRom(String, String), EmuArgs(String, String), EmuStandard(String), EmuWeg(String), EmuSuche(String), EmuTest(String),
    MaschPlus(String), MaschWeg(String), MaschFeld(String, Feld, String), MaschVerb(String, String), MaschVerbFuer(String, &'static str, String), MaschHaken(String, Haken, bool), MaschTest(String, Option<vc_ide::mega65net::Link>), PwZeigen(String),
    Ports, PortWahl(String, String), BaudWahl(String, String),
    ProbeErg(String, Result<String, String>), PortsErg(Vec<String>),
    FujisanPfad(String), FujisanDebug(String),
    InstallFrage(Option<String>), Installieren(String),
}

pub struct ZielZ {
    pub system: String, pub filter: String, pub proben: HashMap<String, Probe>, pub ports: Vec<String>, pub scannt: bool, pub pw_zeigen: HashSet<String>, pub install_frage: Option<String>,
    anwesend: RefCell<HashMap<String, (Instant, String, Presence)>>,
}
impl Default for ZielZ {
    fn default() -> Self { ZielZ { system: String::new(), filter: String::new(), proben: HashMap::new(), ports: vec![], scannt: false, pw_zeigen: HashSet::new(), install_frage: None, anwesend: RefCell::new(HashMap::new()) } }
}
impl ZielZ {
    /// Programm gefunden? Das Ergebnis gilt zwei Sekunden, damit nicht in jedem Bild das Dateisystem durchsucht wird
    fn anwesend(&self, e: &EmulatorCfg) -> Presence {
        let sig = format!("{}|{}", e.kind, e.path); let mut m = self.anwesend.borrow_mut();
        if let Some((t, s, p)) = m.get(&e.id) { if *s == sig && t.elapsed() < Duration::from_secs(2) { return p.clone() } }
        let p = platform::presence(e); m.insert(e.id.clone(), (Instant::now(), sig, p.clone())); p
    }
    fn vergessen(&self) { self.anwesend.borrow_mut().clear() }
}

pub fn emu_key(id: &str) -> String { format!("emu:{id}") }
pub fn hw_key(id: &str) -> String { format!("hw:{id}") }
fn install_key(id: &str) -> String { format!("inst:{id}") }
fn mb(b: u64) -> f64 { b as f64 / 1_048_576.0 }

fn z(a: ZielAkt) -> Msg { Msg::Set(SetAkt::Ziel(a)) }
fn eingabe<'a>(hint: &str, wert: &str, breite: Length, m: impl Fn(String) -> ZielAkt + 'a) -> Element<'a, Msg> { text_input(hint, wert).on_input(move |s| z(m(s))).size(13).padding(5).width(breite).into() }
fn zeile2<'a>(pal: Pal, schmal: bool, titel: impl AsRef<str>, inhalt: Element<'a, Msg>) -> Element<'a, Msg> { if schmal { return column![text(titel.as_ref().to_string()).size(12).color(pal.mute), inhalt].spacing(3).into() } row![text(titel.as_ref().to_string()).size(12.5).color(pal.mute).width(175), container(inhalt).width(Length::Fill)].spacing(10).align_y(Alignment::Center).into() }
fn chip<'a>(pal: Pal, s: impl AsRef<str>, c: Color) -> Element<'a, Msg> { container(text(s.as_ref().to_string()).size(11.5).color(c)).padding([2, 8]).style(move |_| container::Style { border: Border { color: c, width: 1.0, radius: 10.0.into() }, background: Some(Color { a: 0.12, ..c }.into()), ..Default::default() }).into() }
fn wahl<'a>(pal: Pal, opt: &[(&str, &str)], aktuell: &str, m: impl Fn(String) -> ZielAkt + 'a) -> Element<'a, Msg> {
    row(opt.iter().map(|(w, l)| { let on = *w == aktuell; button(text(l.to_string()).size(12.5).color(pal.text)).padding([4, 10]).on_press(z(m(w.to_string()))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6).into()
}

impl App {
    fn ziel_gueltig(&mut self) {
        let e = &self.satz.entwurf;
        if e.target(&self.satz.ziel.system).is_none() { self.satz.ziel.system = e.targets.first().map(|t| t.system.clone()).unwrap_or_default() }
    }
    /// Beim ersten Öffnen: Auswahl und Port-Liste
    pub(crate) fn ziel_oeffnen(&mut self) -> Task<Msg> {
        self.ziel_gueltig();
        if self.satz.ziel.ports.is_empty() && !self.satz.ziel.scannt { return self.ziel_ports() }
        Task::none()
    }
    fn ziel_ports(&mut self) -> Task<Msg> {
        self.satz.ziel.scannt = true;
        Task::perform(async { tokio::task::spawn_blocking(aussen::ports_suchen).await.unwrap_or_default() }, |p| z(ZielAkt::PortsErg(p)))
    }
    fn ziel_probe(&mut self, key: String, arbeit: impl FnOnce() -> Result<String, String> + Send + 'static) -> Task<Msg> {
        self.satz.ziel.proben.insert(key.clone(), Probe::Laeuft);
        Task::perform(async move { tokio::task::spawn_blocking(arbeit).await.unwrap_or_else(|e| Err(e.to_string())) }, move |r| z(ZielAkt::ProbeErg(key.clone(), r)))
    }

    pub(crate) fn ziel_akt(&mut self, a: ZielAkt) -> Task<Msg> {
        let sys = self.satz.ziel.system.clone();
        match a {
            ZielAkt::Wahl(s) => self.satz.ziel.system = s,
            ZielAkt::Filter(f) => self.satz.ziel.filter = f,
            ZielAkt::Hinzu(id) => { vc_ide::setup::set_system(&mut self.satz.entwurf, &id, true); self.satz.ziel.system = id }
            ZielAkt::Weg => { vc_ide::setup::set_system(&mut self.satz.entwurf, &sys, false); self.ziel_gueltig() }
            ZielAkt::StartZiel(id) => { if let Some(t) = self.satz.entwurf.target_mut(&sys) { t.start_on = id } }
            ZielAkt::EmuPlus(k) => { if let Some(t) = self.satz.entwurf.target_mut(&sys) { t.add_emulator(&k); } }
            ZielAkt::EmuName(id, v) => self.emu_mut(&sys, &id, |e| e.name = v),
            ZielAkt::EmuArt(id, v) => self.emu_mut(&sys, &id, |e| e.kind = v),
            ZielAkt::EmuPfad(id, v) => { self.emu_mut(&sys, &id, |e| e.path = v); self.satz.ziel.vergessen() }
            ZielAkt::EmuRom(id, v) => self.emu_mut(&sys, &id, |e| e.rom = v),
            ZielAkt::EmuArgs(id, v) => self.emu_mut(&sys, &id, |e| e.args = v),
            ZielAkt::EmuStandard(id) => { if let Some(t) = self.satz.entwurf.target_mut(&sys) { t.set_standard(&id) } }
            ZielAkt::EmuWeg(id) => { if let Some(t) = self.satz.entwurf.target_mut(&sys) { t.remove_emulator(&id) } self.satz.ziel.proben.remove(&emu_key(&id)); }
            ZielAkt::EmuSuche(id) => { let kind = self.satz.entwurf.target(&sys).and_then(|t| t.emulators.iter().find(|e| e.id == id)).map(|e| e.kind.clone()); if let Some(p) = kind.and_then(|k| platform::discover(&k)) { self.emu_mut(&sys, &id, |e| e.path = p); self.satz.ziel.vergessen() } }
            ZielAkt::EmuTest(id) => { if let Some(e) = self.satz.entwurf.target(&sys).and_then(|t| t.emulators.iter().find(|e| e.id == id)).cloned() { return self.ziel_probe(emu_key(&id), move || aussen::emulator_test(&e)) } }
            ZielAkt::MaschPlus(k) => { self.satz.entwurf.add_machine(&sys, &k); }
            ZielAkt::MaschWeg(id) => { self.satz.entwurf.remove_machine(&id); self.satz.ziel.proben.remove(&hw_key(&id)); }
            ZielAkt::MaschFeld(id, f, v) => self.masch_mut(&id, |m| match f { Feld::Name => m.name = v, Feld::Host => m.host = v, Feld::Passwort => m.password = v, Feld::Geraet => m.device = v, Feld::Baud => m.baud = v, Feld::EthSchnittstelle => m.eth_iface = v, Feld::Etherload => m.etherload = v, Feld::FtpTool => m.ftp_tool = v, Feld::SdOrdner => m.sd_dir = v }),
            ZielAkt::MaschVerb(id, v) => self.masch_mut(&id, |m| m.link = v),
            ZielAkt::MaschVerbFuer(id, was, v) => self.masch_mut(&id, |m| match was { "start" => m.link_start = v, "copy" => m.link_copy = v, _ => m.link_debug = v }),
            ZielAkt::MaschHaken(id, h, an) => self.masch_mut(&id, |m| match h { Haken::RunNach => m.run_after = an, Haken::ResetVor => m.reset_before = an }),
            ZielAkt::MaschTest(id, link) => { if let Some(m) = self.satz.entwurf.machine(&id).cloned() { return self.ziel_probe(hw_key(&id), move || aussen::hardware_test(&m, link)) } }
            ZielAkt::PwZeigen(id) => { if !self.satz.ziel.pw_zeigen.remove(&id) { self.satz.ziel.pw_zeigen.insert(id); } }
            ZielAkt::Ports => return self.ziel_ports(),
            ZielAkt::PortWahl(id, p) => self.masch_mut(&id, |m| m.device = p),
            ZielAkt::BaudWahl(id, b) => self.masch_mut(&id, |m| m.baud = b),
            ZielAkt::ProbeErg(k, r) => { self.satz.ziel.proben.insert(k, match r { Ok(t) => Probe::Ok(t), Err(e) => Probe::Fehler(e) }); }
            ZielAkt::PortsErg(p) => { self.satz.ziel.ports = p; self.satz.ziel.scannt = false }
            ZielAkt::FujisanPfad(p) => { self.satz.entwurf.emulators.fujisan.path = p; self.satz.ziel.vergessen() }
            ZielAkt::FujisanDebug(d) => self.satz.entwurf.emulators.fujisan.debug_for_atari = d,
            ZielAkt::InstallFrage(s) => self.satz.ziel.install_frage = s,
            ZielAkt::Installieren(spec) => { self.satz.ziel.install_frage = None; let key = install_key(spec.split('@').next().unwrap_or(&spec)); return self.ziel_probe(key, move || aussen::emulator_installieren(&spec)) }
        }
        Task::none()
    }
    fn emu_mut(&mut self, sys: &str, id: &str, f: impl FnOnce(&mut EmulatorCfg)) { if let Some(e) = self.satz.entwurf.target_mut(sys).and_then(|t| t.emulators.iter_mut().find(|e| e.id == id)) { f(e) } }
    fn masch_mut(&mut self, id: &str, f: impl FnOnce(&mut vc_ide::settings::Machine)) { if let Some(m) = self.satz.entwurf.machines.iter_mut().find(|m| m.id == id) { f(m) } }

    pub(crate) fn seite_ziele(&self, pal: Pal) -> Element<'_, Msg> { iced::widget::responsive(move |size| self.ziele_inhalt(pal, size.width)).into() }
    fn ziele_inhalt(&self, pal: Pal, breite: f32) -> Element<'_, Msg> {
        let schmal = breite < 760.0; let zs = &self.satz.ziel; let s = &self.satz.entwurf;
        let aktiv = if s.target(&zs.system).is_some() { zs.system.clone() } else { s.targets.first().map(|t| t.system.clone()).unwrap_or_default() };
        // ---- linke Liste
        let q = zs.filter.trim().to_lowercase();
        let passt = |i: &targets::SystemInfo| -> bool {
            if q.is_empty() { return true }
            let mut hay = vec![i.name().to_lowercase(), i.id.to_string(), i.group().to_lowercase()];
            if let Some(t) = s.target(i.id) { for e in &t.emulators { hay.push(e.display_name().to_lowercase()); hay.push(e.kind.clone()) } }
            for m in s.machines_for(i.id) { hay.push(m.display_name().to_lowercase()); hay.push(m.host.to_lowercase()); hay.push(m.device.to_lowercase()) }
            hay.iter().any(|h| h.contains(&q))
        };
        let mut liste = column![].spacing(4); let mut letzte = String::new(); let mut gezeigt = 0;
        for i in targets::SYSTEMS {
            let Some(t) = s.target(i.id) else { continue }; if !passt(i) { continue } gezeigt += 1;
            if i.group() != letzte { letzte = i.group(); liste = liste.push(text(i.group().to_uppercase()).size(11).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute)) }
            let emu = t.standard_emulator(); let hw = s.machines_for(i.id).count();
            let mut sub = emu.map_or(vc_i18n::t!("zielui-kein-emulator").to_string(), |e| e.display_name().to_string()); if hw > 0 { sub.push_str(&format!(" {}", vc_i18n::t!("zielui-hw-hw", hw = &hw))) }
            let punkt = match emu { Some(e) if zs.anwesend(e).found => pal.ok, Some(_) => pal.err, None => pal.mute };
            let on = i.id == aktiv;
            liste = liste.push(button(row![text("●").size(12).color(punkt), column![text(i.name()).size(13).color(pal.text), text(sub).size(11).color(pal.mute)].spacing(1)].spacing(8).align_y(Alignment::Center)).width(Length::Fill).padding([6, 8]).on_press(z(ZielAkt::Wahl(i.id.to_string()))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }))
        }
        if gezeigt == 0 { liste = liste.push(klein(pal, if s.targets.is_empty() { vc_i18n::t!("zielui-noch-kein-zielsystem-unten-eines-hinzufuegen") } else { vc_i18n::t!("zielui-nichts-gefunden") })) }
        let gewaehlt = vc_ide::features::features(); let frei: Vec<&targets::SystemInfo> = targets::SYSTEMS.iter().filter(|i| gewaehlt.target(i.id) && s.target(i.id).is_none()).collect();
        let hinzu: Element<Msg> = if frei.is_empty() { klein(pal, vc_i18n::t!("zielui-alle-systeme-sind-bereits-eingetragen")) } else {
            column![klein(pal, vc_i18n::t!("zielui-zielsystem-hinzufuegen")), row(frei.iter().map(|i| { let hid = s.hidden_targets.iter().any(|h| h.system == i.id); button(text(if hid { vc_i18n::t!("zielui-p1-deaktiviert", p1 = &i.name()) } else { i.name().to_string() }).size(12).color(pal.text)).padding([3, 8]).on_press(z(ZielAkt::Hinzu(i.id.to_string()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(5).wrap().vertical_spacing(5)].spacing(5).into() };
        let links = column![text_input(&vc_i18n::t!("zielui-filter-z-b-vice-c64-ip"), &zs.filter).on_input(|f| z(ZielAkt::Filter(f))).size(13).padding(6), liste, hinzu].spacing(8).width(if schmal { Length::Fill } else { Length::Fixed(230.0) });
        // ---- rechts
        let rechts: Element<Msg> = match targets::system(&aktiv) { Some(info) if s.target(&aktiv).is_some() => self.ziel_detail(pal, info, schmal), _ => column![klein(pal, vc_i18n::t!("zielui-kein-zielsystem-ausgewaehlt-links-unten-laesst"))].into() };
        if schmal { column![links, rechts].spacing(16).into() } else { row![links, container(rechts).width(Length::Fill)].spacing(16).into() }
    }

    fn ziel_detail(&self, pal: Pal, info: &'static targets::SystemInfo, schmal: bool) -> Element<'_, Msg> {
        let zs = &self.satz.ziel; let s = &self.satz.entwurf;
        let t = s.target(info.id).expect(&vc_i18n::t!("zielui-zielsystem"));
        let std_emu = t.standard_emulator().cloned();
        // Kopf
        let status = match &std_emu { Some(e) => if zs.anwesend(e).found { chip(pal, vc_i18n::t!("zielui-emulator-bereit"), pal.ok) } else { chip(pal, vc_i18n::t!("zielui-emulator-fehlt"), pal.err) }, None => chip(pal, vc_i18n::t!("zielui-kein-emulator-2"), pal.warn) };
        let kopf = row![text(info.name()).size(18).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), status, klein(pal, info.cpu()), iced::widget::space::horizontal(), knopf(pal, vc_i18n::t!("zielui-zielsystem-deaktivieren"), Some(z(ZielAkt::Weg)), pal.text, false)].spacing(10).align_y(Alignment::Center).wrap().vertical_spacing(6);
        // Startziel
        let maschinen: Vec<(String, String)> = s.machines_for(info.id).filter(|m| vc_ide::emulators::device_enabled(&m.kind)).map(|m| (m.id.clone(), if m.kind == "mega65" { format!("{} · {}", m.display_name(), m.link_for(vc_ide::mega65net::Action::Start).label()) } else { m.display_name().to_string() })).collect();
        let mut start_on = t.start_on.clone(); if !start_on.is_empty() && !maschinen.iter().any(|(id, _)| *id == start_on) { start_on.clear() }
        let emu_name = std_emu.as_ref().map_or(vc_i18n::t!("zielui-kein-emulator").to_string(), |e| e.display_name().to_string());
        let mut opt: Vec<(String, String)> = vec![(String::new(), vc_i18n::t!("zielui-emulator-emu-name", emu_name = &emu_name))]; for (id, n) in &maschinen { opt.push((id.clone(), vc_i18n::t!("zielui-n-hardware", n = &n))) }
        let optr: Vec<(&str, &str)> = opt.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let mut startziel = vec![zeile2(pal, schmal, vc_i18n::t!("zielui-startziel-f5"), wahl(pal, &optr, &start_on, ZielAkt::StartZiel))];
        if maschinen.is_empty() { startziel.push(klein(pal, vc_i18n::t!("zielui-keine-hardware-eingetragen"))) }
        // Überschreibung durch das Projekt: Das geöffnete Projekt kann das Ziel in der Projektdatei überschreiben (Anzeige folgt mit den Projekt-Einstellungen, 4b)
        let mut spalte = column![kopf, karte(pal, startziel)].spacing(10);
        // Emulatoren
        let arten: Vec<&'static targets::EmuKind> = info.kinds().iter().filter(|k| vc_ide::emulators::enabled(k.id)).collect();
        let mut plus = row![].spacing(6); for k in &arten { plus = plus.push(button(text(format!("+ {}", k.label)).size(12).color(pal.text)).padding([3, 8]).on_press(z(ZielAkt::EmuPlus(k.id.to_string()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() })) }
        spalte = spalte.push(abschnitt(pal, vc_i18n::t!("zielui-konfigurierte-emulatoren"), vc_i18n::t!("zielui-programm-startargumente-und-standard-emulator-dieses"))).push(plus.wrap().vertical_spacing(6));
        let n_emu = t.emulators.iter().filter(|e| vc_ide::emulators::enabled(&e.kind)).count();
        if arten.is_empty() { spalte = spalte.push(klein(pal, vc_i18n::t!("zielui-in-diesem-release-ist-keine-emulator"))) } else if n_emu == 0 { spalte = spalte.push(klein(pal, vc_i18n::t!("zielui-kein-emulator-eingetragen-ohne-emulator-laesst"))) }
        for e in t.emulators.iter().filter(|e| vc_ide::emulators::enabled(&e.kind)) { spalte = spalte.push(self.emu_karte(pal, schmal, info, e, &arten, n_emu)) }
        if info.id == "atari8" && vc_ide::emulators::enabled("fujisan") { spalte = spalte.push(self.fujisan_karte(pal, schmal)) }
        // Hardware
        let hw_arten: Vec<&'static str> = info.hardware().iter().copied().filter(|k| vc_ide::emulators::device_enabled(k)).collect();
        let mut hplus = row![].spacing(6); for k in &hw_arten { hplus = hplus.push(button(text(format!("+ {}", targets::hardware_label(k))).size(12).color(pal.text)).padding([3, 8]).on_press(z(ZielAkt::MaschPlus(k.to_string()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() })) }
        spalte = spalte.push(abschnitt(pal, vc_i18n::t!("zielui-physische-zielhardware-netzwerkgeraete"), vc_i18n::t!("zielui-kompilierte-programme-direkt-ueber-ethernet-oder"))).push(hplus.wrap().vertical_spacing(6));
        let ms: Vec<&vc_ide::settings::Machine> = s.machines_for(info.id).filter(|m| vc_ide::emulators::device_enabled(&m.kind)).collect();
        if ms.is_empty() { spalte = spalte.push(klein(pal, if hw_arten.is_empty() { if info.hardware().is_empty() { vc_i18n::t!("zielui-fuer-dieses-system-ist-keine-echte") } else { vc_i18n::t!("zielui-in-diesem-release-ist-keine-hardware") } } else { vc_i18n::t!("zielui-noch-keine-hardware-eingetragen") })) }
        for m in ms { spalte = spalte.push(self.masch_karte(pal, schmal, m)) }
        spalte.into()
    }

    fn probe_zeile<'a>(&self, pal: Pal, key: &str) -> Element<'a, Msg> {
        match self.satz.ziel.proben.get(key) { Some(Probe::Ok(t)) => text(vc_i18n::t!("zielui-ok-t", t = &t)).size(11.5).color(pal.ok).into(), Some(Probe::Fehler(t)) => text(t.clone()).size(11.5).color(pal.err).into(), Some(Probe::Laeuft) => klein(pal, vc_i18n::t!("zielui-laeuft")), None => text("").into() }
    }

    fn emu_karte<'a>(&'a self, pal: Pal, schmal: bool, info: &'static targets::SystemInfo, e: &'a EmulatorCfg, arten: &[&'static targets::EmuKind], n_emu: usize) -> Element<'a, Msg> {
        let zs = &self.satz.ziel; let id = e.id.clone(); let p = zs.anwesend(e);
        let mut kopf = row![eingabe(&vc_i18n::t!("zielui-name"), &e.name, Length::Fixed(200.0), { let id = id.clone(); move |v| ZielAkt::EmuName(id.clone(), v) })].spacing(8).align_y(Alignment::Center);
        if arten.len() > 1 { kopf = kopf.push(wahl(pal, &arten.iter().map(|k| (k.id, k.label)).collect::<Vec<_>>(), &e.kind, { let id = id.clone(); move |v| ZielAkt::EmuArt(id.clone(), v) })) } else { kopf = kopf.push(klein(pal, targets::kind_label(&e.kind))) }
        if e.standard { kopf = kopf.push(chip(pal, "STANDARD", pal.accent)) }
        kopf = kopf.push(if p.found { chip(pal, if p.auto { vc_i18n::t!("zielui-automatisch-erkannt") } else { vc_i18n::t!("zielui-gefunden") }, pal.ok) } else { chip(pal, vc_i18n::t!("zielui-fehlt"), pal.err) });
        let mut k: Vec<Element<Msg>> = vec![kopf.wrap().vertical_spacing(6).into()];
        if let Some(en) = vc_ide::emulators::for_kind(&e.kind) {
            k.push(klein(pal, format!("{} · {} · {}", en.homepage, en.license, en.role)));
            k.push(self.install_zeile(pal, en, p.found));
        }
        let hint = if p.auto && p.found { vc_i18n::t!("zielui-automatisch-p1", p1 = &p.text) } else { vc_i18n::t!("zielui-leer-automatisch-suchen").to_string() };
        k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-pfad"), row![crate::pfadfeld::pfadfeld(pal, &hint, &e.path, crate::pfadfeld::PfadArt::Datei, Length::Fill, { let id = id.clone(); move |v| z(ZielAkt::EmuPfad(id.clone(), v)) }), knopf(pal, vc_i18n::t!("zielui-automatisch-suchen"), Some(z(ZielAkt::EmuSuche(id.clone()))), pal.text, false)].spacing(8).into()));
        if let Some(r) = vc_ide::emulators::for_kind(&e.kind).and_then(|x| x.rom.as_ref()) {
            let v = e.rom.trim(); let da = if v.is_empty() { None } else { let pth = std::path::PathBuf::from(vc_ide::settings::expand_home(v)); Some(if r.folder { pth.is_dir() } else { pth.is_file() }) };
            k.push(zeile2(pal, schmal, &r.label, row![crate::pfadfeld::pfadfeld(pal, &r.hint, &e.rom, crate::pfadfeld::PfadArt::Datei, Length::Fill, { let id = id.clone(); move |v| z(ZielAkt::EmuRom(id.clone(), v)) }), match da { Some(true) => text("vorhanden").size(11.5).color(pal.ok), Some(false) => text(vc_i18n::t!("zielui-nicht-gefunden")).size(11.5).color(pal.err), None => text("").size(11.5) }].spacing(8).align_y(Alignment::Center).into()))
        }
        if !p.found { k.push(text(p.text.clone()).size(11.5).color(pal.err).into()) }
        let programm = if e.path.trim().is_empty() { e.kind.clone() } else { e.path.trim().to_string() };
        let bsp = match e.kind.as_str() { "altirra" => vc_i18n::t!("zielui-z-b-f"), "atari800" => vc_i18n::t!("zielui-z-b-nosound"), "xemu" => vc_i18n::t!("zielui-z-b-fullscreen"), _ => vc_i18n::t!("zielui-z-b-viciifilter-0-sidenginemodel-1") };
        k.push(self.args_editor(pal, &e.id, &vc_i18n::t!("zielui-startparameter"), vc_ide::args::catalog_of_kind(&e.kind), vc_ide::args::ArgCtx::example(info.id), &programm, &bsp, &e.opts, &e.args, true));
        let key = emu_key(&e.id); let busy = zs.proben.get(&key) == Some(&Probe::Laeuft);
        let mut aktion = row![knopf(pal, if busy { vc_i18n::t!("zielui-startet") } else { vc_i18n::t!("zielui-test-start") }, (!busy && p.found).then(|| z(ZielAkt::EmuTest(id.clone()))), pal.text, false), knopf(pal, vc_i18n::t!("zielui-als-standard-setzen"), (!e.standard).then(|| z(ZielAkt::EmuStandard(id.clone()))), pal.text, false), knopf(pal, vc_i18n::t!("zielui-loeschen"), Some(z(ZielAkt::EmuWeg(id.clone()))), pal.text, false)].spacing(8);
        aktion = aktion.push(self.probe_zeile(pal, &key));
        k.push(aktion.wrap().vertical_spacing(6).into());
        let _ = n_emu;
        karte(pal, k)
    }

    /// Weitere Version portabel installieren (Download nur nach Bestätigung)
    fn install_zeile<'a>(&'a self, pal: Pal, en: &'static vc_ide::emulators::Entry, gefunden: bool) -> Element<'a, Msg> {
        let zs = &self.satz.ziel; let key = install_key(&en.id);
        if zs.proben.get(&key) == Some(&Probe::Laeuft) { return text(vc_i18n::t!("zielui-laedt-und-installiert")).size(12).color(pal.accent).into() }
        let have: Vec<String> = vc_emu::install::versions(&en.id).into_iter().map(|(v, _)| v).collect();
        let todo: Vec<&vc_ide::emulators::Pkg> = en.packages_here().into_iter().filter(|p| !have.contains(&p.version)).collect();
        let mut c = column![].spacing(4);
        if let Some(pkg) = todo.first() {
            let spec = format!("{}@{}", en.id, pkg.version);
            if zs.install_frage.as_deref() == Some(spec.as_str()) {
                let nach = vc_ide::emulators::install_dir(&en.id).map(|d| d.join(&pkg.version).display().to_string()).unwrap_or_default();
                c = c.push(klein(pal, vc_i18n::t!("zielui-laedt-p1-p2-p3-mb-von", p1 = &en.name, p2 = &pkg.version, p3 = vc_i18n::num((mb(pkg.size)) as f64, 1), p4 = pkg.url.split('/').nth(2).unwrap_or(""), nach = &nach)));
                c = c.push(row![knopf(pal, vc_i18n::t!("zielui-jetzt-laden"), Some(z(ZielAkt::Installieren(spec))), pal.text, true), knopf(pal, vc_i18n::t!("zielui-abbrechen"), Some(z(ZielAkt::InstallFrage(None))), pal.text, false)].spacing(8));
            } else {
                let t = if have.is_empty() && !gefunden { vc_i18n::t!("zielui-portabel-installieren-p1-p2-mb", p1 = &pkg.version, p2 = vc_i18n::num((mb(pkg.size)) as f64, 1)) } else { vc_i18n::t!("zielui-version-p1-portabel-dazuinstallieren", p1 = &pkg.version) };
                c = c.push(knopf(pal, &t, Some(z(ZielAkt::InstallFrage(Some(spec)))), pal.text, false));
            }
        } else if en.packages_here().is_empty() && !gefunden { c = c.push(klein(pal, format!("{} {}", en.manual_hint().map(|s| s.to_string()).unwrap_or_else(|| vc_i18n::t!("emulators-fuer-dieses-system-gibt-es-kein-2")), en.download_page))) }
        let erg: Element<Msg> = match zs.proben.get(&key) { Some(Probe::Ok(t)) => text(vc_i18n::t!("zielui-installiert-t", t = &t)).size(11.5).color(pal.ok).into(), Some(Probe::Fehler(t)) => text(t.clone()).size(11.5).color(pal.err).into(), _ => text("").into() };
        c = c.push(erg);
        c.into()
    }

    fn fujisan_karte(&self, pal: Pal, schmal: bool) -> Element<'_, Msg> {
        let fj = &self.satz.entwurf.emulators.fujisan; let typ = fj.path.trim().to_string();
        let gef = if typ.is_empty() { vc_ide::launch::detect("fujisan") } else { Some(vc_ide::settings::expand_home(&typ)).filter(|p| std::path::Path::new(p).is_file()) };
        let hint = match &gef { Some(p) if typ.is_empty() => vc_i18n::t!("zielui-automatisch-p", p = &p), _ => vc_i18n::t!("zielui-leer-automatisch-suchen").to_string() };
        let mut k = vec![row![text(vc_i18n::t!("zielui-fujisan")).size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), if gef.is_some() { chip(pal, if typ.is_empty() { vc_i18n::t!("zielui-automatisch-erkannt") } else { vc_i18n::t!("zielui-gefunden") }, pal.ok) } else { chip(pal, vc_i18n::t!("zielui-fehlt"), pal.err) }].spacing(10).into(),
            zeile2(pal, schmal, vc_i18n::t!("zielui-pfad"), crate::pfadfeld::pfadfeld(pal, &hint, &fj.path, crate::pfadfeld::PfadArt::Datei, Length::Fill, |v| z(ZielAkt::FujisanPfad(v)))),
            zeile2(pal, schmal, vc_i18n::t!("zielui-debugger"), wahl(pal, &[("", &vc_i18n::t!("zielui-automatisch-fujisan-wenn-gefunden")), ("fujisan", &vc_i18n::t!("zielui-immer-fujisan")), ("atari800", &vc_i18n::t!("zielui-immer-atari800"))], fj.debug_for_atari.trim(), ZielAkt::FujisanDebug))];
        if let Some(en) = vc_ide::emulators::find("fujisan") { k.push(klein(pal, format!("{} · {} · {}", en.homepage, en.license, en.role))); k.push(self.install_zeile(pal, en, gef.is_some())) }
        column![abschnitt(pal, vc_i18n::t!("zielui-fujisan-debugger-fuer-atari-programme"), vc_i18n::t!("zielui-atari-emulator-mit-tcp-fernsteuerung-der")), karte(pal, k)].spacing(8).into()
    }

    fn masch_karte<'a>(&'a self, pal: Pal, schmal: bool, m: &'a vc_ide::settings::Machine) -> Element<'a, Msg> {
        let zs = &self.satz.ziel; let id = m.id.clone(); let key = hw_key(&m.id);
        let status = match zs.proben.get(&key) { Some(Probe::Ok(_)) => chip(pal, vc_i18n::t!("zielui-online"), pal.ok), Some(Probe::Fehler(_)) => chip(pal, vc_i18n::t!("zielui-nicht-erreichbar"), pal.err), Some(Probe::Laeuft) => chip(pal, vc_i18n::t!("zielui-pruefe"), pal.warn), None => chip(pal, vc_i18n::t!("zielui-ungeprueft"), pal.mute) };
        let f = |feld: Feld, hint: &str, wert: &str, br: Length| -> Element<'a, Msg> { let id = id.clone(); eingabe(hint, wert, br, move |v| ZielAkt::MaschFeld(id.clone(), feld.clone(), v)) };
        let mut k: Vec<Element<Msg>> = vec![row![f(Feld::Name, &vc_i18n::t!("zielui-name"), &m.name, Length::Fixed(200.0)), klein(pal, targets::hardware_label(&m.kind)), status, iced::widget::space::horizontal(), knopf(pal, vc_i18n::t!("zielui-entfernen"), Some(z(ZielAkt::MaschWeg(id.clone()))), pal.text, false)].spacing(10).align_y(Alignment::Center).wrap().vertical_spacing(6).into()];
        if m.kind == "ultimate64" {
            k.push(zeile2(pal, schmal, "Adresse", f(Feld::Host, &vc_i18n::t!("zielui-ip-oder-host-port"), &m.host, Length::Fill)));
            let zeigen = zs.pw_zeigen.contains(&m.id);
            k.push(zeile2(pal, schmal, "Passwort", row![text_input(&vc_i18n::t!("zielui-leer-kein-netzwerkpasswort"), &m.password).secure(!zeigen).on_input({ let id = id.clone(); move |v| z(ZielAkt::MaschFeld(id.clone(), Feld::Passwort, v)) }).size(13).padding(5).width(Length::Fill), knopf(pal, if zeigen { vc_i18n::t!("zielui-verbergen") } else { vc_i18n::t!("zielui-zeigen") }, Some(z(ZielAkt::PwZeigen(id.clone()))), pal.text, false)].spacing(8).into()));
        } else {
            let cur = if m.link == "eth" { "eth" } else { "usb" };
            k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-verbindung"), wahl(pal, &[("usb", &vc_i18n::t!("zielui-usb-standard")), ("eth", &vc_i18n::t!("zielui-ethernet"))], cur, { let id = id.clone(); move |v| ZielAkt::MaschVerb(id.clone(), v) })));
            for (n, was, wert) in [(vc_i18n::t!("zielui-starten"), "start", &m.link_start), (vc_i18n::t!("zielui-kopieren"), "copy", &m.link_copy), (vc_i18n::t!("zielui-debuggen"), "debug", &m.link_debug)] {
                k.push(zeile2(pal, schmal, &n, wahl(pal, &[("", &vc_i18n::t!("zielui-wie-standard")), ("usb", "USB"), ("eth", &vc_i18n::t!("zielui-ethernet"))], wert, { let id = id.clone(); move |v| ZielAkt::MaschVerbFuer(id.clone(), was, v) })));
            }
            let ports: Vec<Element<Msg>> = zs.ports.iter().map(|p| { let on = m.device == *p; let (id2, p2) = (id.clone(), p.clone()); button(text(p.clone()).size(12).color(pal.text)).padding([3, 8]).on_press(z(ZielAkt::PortWahl(id2, p2))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect();
            k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-schnittstelle"), column![row![f(Feld::Geraet, if cfg!(windows) { "COM3" } else if cfg!(target_os = "linux") { "/dev/ttyUSB0" } else { "/dev/cu.usbserial-…" }, &m.device, Length::Fill), knopf(pal, if zs.scannt { vc_i18n::t!("zielui-sucht") } else { vc_i18n::t!("zielui-ports-scannen") }, (!zs.scannt).then(|| z(ZielAkt::Ports)), pal.text, false)].spacing(8), if ports.is_empty() { klein(pal, vc_i18n::t!("zielui-keine-usb-seriell-geraete-gefunden")) } else { row(ports).spacing(5).wrap().vertical_spacing(5).into() }].spacing(6).into()));
            let bauds: Vec<Element<Msg>> = platform::BAUD_RATES.iter().map(|b| { let on = m.baud == *b; let (id2, b2) = (id.clone(), b.to_string()); button(text(*b).size(12).color(pal.text)).padding([3, 8]).on_press(z(ZielAkt::BaudWahl(id2, b2))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect();
            k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-baudrate"), column![f(Feld::Baud, "2000000", &m.baud, Length::Fixed(140.0)), row(bauds).spacing(5).wrap().vertical_spacing(5)].spacing(6).into()));
            k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-ethernet"), column![text(vc_i18n::t!("zielui-p1-nur-gegen-attrappen-getestet-voraussetzung", p1 = vc_ide::mega65net::unchecked())).size(11.5).color(pal.warn),
                zeile2(pal, schmal, vc_i18n::t!("zielui-netzwerkschnittstelle"), f(Feld::EthSchnittstelle, &if cfg!(windows) { vc_i18n::t!("zielui-index-z-b-5") } else if cfg!(target_os = "macos") { "en0".to_string() } else { "eth0".to_string() }, &m.eth_iface, Length::Fixed(200.0))),
                zeile2(pal, schmal, "etherload", f(Feld::Etherload, &vc_i18n::t!("zielui-leer-im-path-suchen"), &m.etherload, Length::Fill)), zeile2(pal, schmal, "mega65_ftp", f(Feld::FtpTool, &vc_i18n::t!("zielui-leer-im-path-suchen"), &m.ftp_tool, Length::Fill)),
                zeile2(pal, schmal, vc_i18n::t!("zielui-sd-ordner"), f(Feld::SdOrdner, &vc_i18n::t!("zielui-leer-oberste-ebene-z-b-vintagecoder"), &m.sd_dir, Length::Fixed(260.0)))].spacing(6).into()));
        }
        let mut start = row![checkbox(m.run_after).label(vc_i18n::t!("zielui-nach-uebertragung-run-ausfuehren")).on_toggle({ let id = id.clone(); move |b| z(ZielAkt::MaschHaken(id.clone(), Haken::RunNach, b)) }).size(16).text_size(13)].spacing(16);
        if m.kind == "ultimate64" { start = start.push(checkbox(m.reset_before).label(vc_i18n::t!("zielui-maschine-vorher-zuruecksetzen")).on_toggle({ let id = id.clone(); move |b| z(ZielAkt::MaschHaken(id.clone(), Haken::ResetVor, b)) }).size(16).text_size(13)) }
        k.push(zeile2(pal, schmal, vc_i18n::t!("zielui-start"), start.wrap().vertical_spacing(6).into()));
        let busy = zs.proben.get(&key) == Some(&Probe::Laeuft);
        let eth = m.kind == "mega65" && m.link_default() == vc_ide::mega65net::Link::Eth;
        let bereit = match m.kind.as_str() { "ultimate64" => !m.host.trim().is_empty(), _ => eth || !m.device.trim().is_empty() };
        let mut aktion = row![knopf(pal, if busy { vc_i18n::t!("zielui-pruefe") } else if m.kind == "mega65" { if eth { "Ethernet testen".to_string() } else { "USB testen".to_string() } } else { vc_i18n::t!("zielui-verbindung-testen") }, (!busy && bereit).then(|| z(ZielAkt::MaschTest(id.clone(), None))), pal.text, false)].spacing(8);
        if m.kind == "mega65" { let andere = if eth { vc_ide::mega65net::Link::Usb } else { vc_ide::mega65net::Link::Eth }; let bereit2 = if eth { !m.device.trim().is_empty() } else { true }; aktion = aktion.push(knopf(pal, vc_i18n::t!("zielui-p1-testen", p1 = andere.label()), (!busy && bereit2).then(|| z(ZielAkt::MaschTest(id.clone(), Some(andere)))), pal.text, false)) }
        aktion = aktion.push(self.probe_zeile(pal, &key));
        k.push(aktion.wrap().vertical_spacing(6).into());
        karte(pal, k)
    }
}
