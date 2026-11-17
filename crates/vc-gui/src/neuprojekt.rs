// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „Neues Projekt“ (Stufe 4b): Zielsystem, Vorlage und Details (Name, Ort, BASIC-Dialekt, Videonorm, VIC-20-Speicher, Emulator und Startziel,
//! Diskette, Projektstruktur, README, Git) mit Vorschau der Dateistruktur. Vorlagen kommen aus `data/templates`; angelegt wird mit
//! `vc_media::wizard::create_with`, danach ergänzt `anlegen` die Projektdatei. Weicht Emulator oder Startziel von den Einstellungen ab, steht die
//! Abweichung als Überschreibung in der Projektdatei. Es gibt nur unseren Assembler, daher keine Wahl eines Cross-Assemblers.
//! Das Anlegen schreibt nur in den gewählten Ort; `git init` läuft nur über `aussen` (im Test aufgezeichnet).
use crate::setui::{abschnitt, klein};
use crate::{aussen, flaeche, knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Task};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::targets;
use vc_ide_model::recent::{dialect_label, short_name, target_dialect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sprache { Asm, Basic }

#[derive(Clone, Debug)]
pub struct Karte { pub vorlage: String, pub variante: Option<String>, pub name: String, pub beschreibung: String, pub sprache: Sprache, pub systeme: Vec<String>, pub dateien: Vec<String> }

#[derive(Clone, Debug)]
pub enum NpAkt {
    System(String), Sprache(Sprache), Karte(usize), Name(String), Ort, OrtGewaehlt(Option<PathBuf>), Dialekt(String), Norm(String), Speicher(String), Emu(String), Start(String),
    Disk(String), Zeichensatz(String), Struktur(usize), Readme(bool), Git(bool), Extra(String, bool), Ok, Abbruch,
}

pub struct NeuProj {
    pub data: PathBuf, pub karten: Vec<Karte>, pub name: String, pub ort: PathBuf, pub system: String, pub extra: Vec<String>, pub sprache: Sprache, pub karte: Option<usize>,
    pub norm: String, pub speicher: String, pub dialekt: String, pub disk: String, pub emulator: String, pub start_on: String, pub git: bool, pub readme: bool,
    pub strukturen: Vec<vc_ide::structure::Structure>, pub struktur: usize, pub fehler: String, pub zeichensatz: vc_media::wizard::Zeichensatz,
}

fn strs(v: &Value) -> Vec<String> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }
fn json_lesen(p: &Path) -> Option<Value> { serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok() }
/// Diskettenformate, die zu einem System passen
pub fn disk_formate(system: &str) -> &'static [&'static str] { match system { "c128" => &["d64", "d71", "d81"], "atari8" => &["atr"], "mega65" => &["d81"], _ => &["d64"] } }

impl NeuProj {
    pub fn neu(data: &Path, ort: PathBuf, einst: &vc_ide::settings::Settings) -> Option<NeuProj> {
        let idx = json_lesen(&data.join("templates/index.json"))?;
        let mut karten = vec![];
        for t in idx["templates"].as_array()? {
            let mut add = |name: String, beschr: String, id: Option<String>, node: &Value| {
                let sprache = if node["sources"].as_array().and_then(|s| s.first()).map_or("asm", |s| s["kind"].as_str().unwrap_or("asm")) == "basic" { Sprache::Basic } else { Sprache::Asm };
                let dateien = node["files"].as_array().map(|f| f.iter().filter_map(|x| x["to"].as_str().map(String::from)).collect()).unwrap_or_default();
                karten.push(Karte { vorlage: t["id"].as_str().unwrap_or("").into(), variante: id, name, beschreibung: beschr, sprache, systeme: strs(&node["systems"]), dateien });
            };
            add(t["name"].as_str().unwrap_or("").into(), t["description"].as_str().unwrap_or("").into(), None, t);
            for v in t["variants"].as_array().into_iter().flatten() { add(v["name"].as_str().unwrap_or("").into(), v["description"].as_str().unwrap_or("").into(), v["id"].as_str().map(String::from), v); }
        }
        let mut w = NeuProj { data: data.to_path_buf(), karten, name: "meinprojekt".into(), ort, system: vc_target::default_target_or_c64(Some(data)), extra: vec![], sprache: Sprache::Asm, karte: None, zeichensatz: vc_media::wizard::Zeichensatz::Auto, norm: "pal".into(), speicher: String::new(), dialekt: String::new(), disk: String::new(), emulator: String::new(), start_on: String::new(), git: false, readme: true,
            strukturen: vc_ide::structure::list_all(data), struktur: 0, // Vorgabe „einfach“ (flach); die Struktur legt der Anwender selbst fest
             fehler: String::new() };
        w.abgleichen(einst); Some(w)
    }
    pub fn gefiltert(&self) -> Vec<usize> { self.karten.iter().enumerate().filter(|(_, c)| c.sprache == self.sprache && c.systeme.contains(&self.system)).map(|(i, _)| i).collect() }
    pub fn dialekte(&self) -> Vec<String> { let mut v: Vec<String> = target_dialect(&self.data, &self.system).into_iter().collect(); if self.system == "atari8" { v.push("turbo_basic_xl".into()) } v }
    pub fn vic20_speicher(&self) -> Vec<(String, String)> {
        let Some(t) = json_lesen(&self.data.join("targets/vic20.target.json")) else { return vec![] };
        let vorgabe = t["memory"]["default"].as_str().unwrap_or("").to_string();
        let mut v: Vec<(String, String)> = t["memory"]["configs"].as_object().map(|o| o.iter().map(|(k, c)| (k.clone(), c["label"].as_str().unwrap_or(k).to_string())).collect()).unwrap_or_default();
        if let Some(i) = v.iter().position(|x| x.0 == vorgabe) { let d = v.remove(i); v.insert(0, d) } v
    }
    /// Auswahl nach einer Änderung gültig halten
    pub fn abgleichen(&mut self, einst: &vc_ide::settings::Settings) {
        let f = self.gefiltert(); if !self.karte.is_some_and(|c| f.contains(&c)) { self.karte = f.first().copied() }
        let (system, karte) = (self.system.clone(), self.karte);
        self.extra.retain(|e| *e != system && karte.is_some_and(|c| self.karten[c].systeme.contains(e)));
        if self.dialekt.is_empty() || !self.dialekte().contains(&self.dialekt) { self.dialekt = self.dialekte().first().cloned().unwrap_or_default() }
        if !disk_formate(&self.system).contains(&self.disk.as_str()) { self.disk.clear() }
        let cfg = einst.target(&self.system);
        if !cfg.is_some_and(|t| t.emulators.iter().any(|e| e.id == self.emulator)) { self.emulator = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.clone()).unwrap_or_default() }
        if !self.start_on.is_empty() && einst.machines_for(&self.system).all(|m| m.id != self.start_on) { self.start_on.clear() }
        if self.system == "vic20" && self.speicher.is_empty() { self.speicher = self.vic20_speicher().first().map(|m| m.0.clone()).unwrap_or_default() }
    }
    pub fn name_ok(&self) -> bool { !self.name.is_empty() && self.name.len() <= 48 && self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') }
    pub fn systeme(&self) -> Vec<String> { let mut v = vec![self.system.clone()]; v.extend(self.extra.iter().cloned()); v }
    /// Dateistruktur-Vorschau
    pub fn vorschau(&self) -> Vec<String> {
        let mut f = vec![format!("{}.vcprj", self.name)];
        if let Some(c) = self.karte { for s in self.systeme() { for d in &self.karten[c].dateien { f.push(d.replace("{system}", &s)) } } }
        if let Some(st) = self.strukturen.get(self.struktur) { for d in st.dirs_for(&self.systeme()) { if !f.iter().any(|x| x.starts_with(&format!("{d}/"))) { f.push(format!("{d}/")) } } }
        for z in vc_media::wizard::zeichensatz_vorschau(&self.data, &self.systeme(), self.zeichensatz) { f.push(z) }
        if self.readme { f.push("README.md".into()) } if self.git { f.push(".gitignore".into()) } f.push("build/".into()); f
    }
    /// Projekt anlegen und die Projektdatei ergänzen; liefert den Projektordner und Hinweise (z. B. Git fehlt)
    pub fn anlegen(&self, einst: &vc_ide::settings::Settings) -> Result<(PathBuf, Vec<String>), String> {
        let c = &self.karten[self.karte.ok_or("keine Vorlage gewählt")?];
        let dir = self.ort.join(&self.name); let systeme = self.systeme();
        vc_media::wizard::create_with(&self.data, &dir, &self.name, &systeme, &c.vorlage, c.variante.as_deref())?;
        let mut notizen = vec![]; let datei = dir.join(format!("{}.vcprj", self.name));
        let mut pf = vc_ide::projfile::ProjectFile::load(&datei)?;
        for t in &mut pf.targets {
            let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
            if sys != "mega65" { t.variant = self.norm.clone() }
            if sys == "vic20" && !self.speicher.is_empty() { t.extra.insert("memory".into(), json!(self.speicher)); }
        }
        if self.sprache == Sprache::Basic { for s in &mut pf.sources { if s.kind != "basic" { continue } let sys = s.targets.first().cloned().unwrap_or_else(|| self.system.clone()); if target_dialect(&self.data, &sys).as_deref() != Some(self.dialekt.as_str()) && sys == self.system { s.dialect = self.dialekt.clone() } } }
        pf.set_overrides_if_differs(&self.system, einst, &self.emulator, &self.start_on);
        if !self.disk.is_empty() { pf.disk = Some(vc_ide::projfile::DiskSpec { format: self.disk.clone(), ..Default::default() }) }
        if let Some(st) = self.strukturen.get(self.struktur) { st.apply(&dir, &mut pf, &systeme, self.git)?; }
        pf.save(&datei)?;
        if let Err(e) = vc_media::wizard::zeichensatz_anlegen(&self.data, &dir, &systeme, self.zeichensatz) { notizen.push(format!("Zeichensatz nicht angelegt: {e}")) }
        if self.readme {
            let sys = targets::system(&self.system).map_or(self.system.as_str(), |s| s.name);
            std::fs::write(dir.join("README.md"), format!("# {}\n\nProjekt für {} ({}), angelegt mit VintageCoder.\n\n- Bauen: F7 (oder `vcc build .`)\n- Starten: F5\n- Debuggen: F6\n", self.name, sys, c.name)).map_err(|e| e.to_string())?;
        }
        if self.git {
            std::fs::write(dir.join(".gitignore"), "build/\n.vcbackup/\n").map_err(|e| e.to_string())?;
            if !aussen::git_init(&dir) { notizen.push("git init ist fehlgeschlagen (ist Git installiert?). .gitignore wurde trotzdem angelegt.".to_string()) }
        }
        Ok((dir, notizen))
    }
}

fn s(a: NpAkt) -> Msg { Msg::NeuProj(a) }
fn chip<'a>(c: Color, t: impl Into<String>) -> Element<'a, Msg> { let t = t.into(); container(text(t).size(11.5).color(c)).padding([2, 8]).style(move |_| container::Style { border: Border { color: c, width: 1.0, radius: 10.0.into() }, background: Some(Color { a: 0.12, ..c }.into()), ..Default::default() }).into() }
fn wahl<'a>(pal: Pal, opt: Vec<(String, String)>, aktuell: &str, m: impl Fn(String) -> NpAkt + 'a) -> Element<'a, Msg> {
    row(opt.into_iter().map(|(w, l)| { let on = w == aktuell; button(text(l).size(12.5).color(pal.text)).padding([4, 10]).on_press(s(m(w))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6).into()
}
fn marke<'a>(pal: Pal, t: &str) -> Element<'a, Msg> { text(t.to_string()).size(11.5).color(pal.mute).into() }

impl App {
    pub(crate) fn neuprojekt_oeffnen(&mut self) {
        let ort = if cfg!(test) { std::env::temp_dir().join("vcgui-neuprojekt") } else { let p = self.satz.gespeichert.projects_dir.trim(); if p.is_empty() { vc_core::userdirs::home_dir().map(|h| h.join("VintageCoder")).unwrap_or_else(|| PathBuf::from(".")) } else { PathBuf::from(vc_ide::settings::expand_home(p)) } };
        match NeuProj::neu(&self.data, ort, &self.satz.gespeichert) { Some(w) => self.neu_projekt = Some(w), None => self.log.push("Neues Projekt: data/templates/index.json fehlt.".into()) }
    }
    pub(crate) fn np_akt(&mut self, a: NpAkt) -> Task<Msg> {
        let einst = self.satz.gespeichert.clone();
        let Some(w) = self.neu_projekt.as_mut() else { return Task::none() };
        match a {
            NpAkt::System(id) => { w.system = id; w.karte = None; w.abgleichen(&einst) }
            NpAkt::Sprache(sp) => { w.sprache = sp; w.karte = None; w.abgleichen(&einst) }
            NpAkt::Karte(i) => { w.karte = Some(i); w.abgleichen(&einst) }
            NpAkt::Name(n) => { w.name = n; w.fehler.clear() }
            NpAkt::Ort => { let start = w.ort.clone(); return Task::perform(async move { rfd::AsyncFileDialog::new().set_directory(start).pick_folder().await.map(|f| f.path().to_path_buf()) }, |p| Msg::NeuProj(NpAkt::OrtGewaehlt(p))) }
            NpAkt::OrtGewaehlt(p) => { if let Some(p) = p { w.ort = p } }
            NpAkt::Dialekt(d) => w.dialekt = d, NpAkt::Norm(n) => w.norm = n, NpAkt::Speicher(m) => w.speicher = m, NpAkt::Emu(e) => w.emulator = e, NpAkt::Start(x) => w.start_on = x, NpAkt::Disk(d) => w.disk = d,
            NpAkt::Zeichensatz(z) => w.zeichensatz = vc_media::wizard::Zeichensatz::parse(&z).unwrap_or(w.zeichensatz), NpAkt::Struktur(i) => w.struktur = i, NpAkt::Readme(b) => w.readme = b, NpAkt::Git(b) => w.git = b,
            NpAkt::Extra(id, an) => { w.extra.retain(|x| *x != id); if an { w.extra.push(id) } }
            NpAkt::Abbruch => self.neu_projekt = None,
            NpAkt::Ok => {
                if !w.name_ok() || w.karte.is_none() { return Task::none() }
                match w.anlegen(&einst) {
                    Ok((dir, notizen)) => { self.neu_projekt = None; for n in notizen { self.log.push(n) } self.log.push(format!("Projekt angelegt: {}", dir.display())); return self.update_innen(Msg::Geoeffnet(Some(dir))) }
                    Err(e) => w.fehler = e,
                }
            }
        }
        Task::none()
    }

    pub(crate) fn neuprojekt_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let w = self.neu_projekt.as_ref()?; let einst = &self.satz.gespeichert;
        // Spalte 1: Zielsystem
        let mut sys = column![marke(pal, "ZIELSYSTEM")].spacing(5);
        for i in targets::installed_systems(Some(&w.data)) {
            let on = w.system == i.id;
            sys = sys.push(button(column![text(i.name).size(13).color(pal.text), text(i.cpu).size(10.5).color(pal.mute)].spacing(1)).width(Length::Fill).padding([6, 10]).on_press(s(NpAkt::System(i.id.to_string()))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }));
        }
        let gueltig = vc_target::load_target(&w.data, &w.system).is_ok();
        sys = sys.push(row![marke(pal, "Hardware-Profil"), iced::widget::space::horizontal(), if gueltig { chip(pal.ok, "gültig") } else { chip(pal.err, "fehlt") }].align_y(Alignment::Center));
        if let Ok(t) = vc_target::load_target(&w.data, &w.system) { sys = sys.push(text(format!("CPU {} · {}", t.cpu, t.chips.join(", "))).size(11).font(vc_gui_ui::fonts::MONO).color(pal.mute)) }
        // Spalte 2: Vorlage
        let mut vor = column![row![marke(pal, "VORLAGE"), wahl(pal, vec![("asm".into(), "Assembler".into()), ("basic".into(), "BASIC".into())], if w.sprache == Sprache::Asm { "asm" } else { "basic" }, |x| NpAkt::Sprache(if x == "asm" { Sprache::Asm } else { Sprache::Basic }))].spacing(10).align_y(Alignment::Center).wrap()].spacing(8);
        let f = w.gefiltert();
        if f.is_empty() { vor = vor.push(klein(pal, "Für diese Kombination aus Zielsystem und Sprache gibt es keine Vorlage.")) }
        for i in f {
            let c = &w.karten[i]; let on = w.karte == Some(i);
            let dateien = c.dateien.iter().map(|d| d.replace("{system}", &w.system)).collect::<Vec<_>>().join("  ");
            vor = vor.push(button(column![row![text(c.name.clone()).size(13.5).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), chip(pal.warn, if c.sprache == Sprache::Asm { "vcasm" } else { "BASIC" })].spacing(8).align_y(Alignment::Center).wrap(), text(c.beschreibung.clone()).size(11.5).color(pal.mute), text(dateien).size(11).font(vc_gui_ui::fonts::MONO).color(pal.mute)].spacing(3)).width(Length::Fill).padding(10).on_press(s(NpAkt::Karte(i))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 8.0.into() }, text_color: pal.text, ..Default::default() }));
        }
        if w.sprache == Sprache::Basic { vor = vor.push(klein(pal, "BASIC wird im Dialekt des Zielsystems geschrieben, ohne Zeilennummern und mit Sprungmarken (@name). Beim Bauen entsteht daraus das native Programm.")) }
        // Spalte 3: Details
        let mut d = column![marke(pal, "PROJEKT-DETAILS"), marke(pal, "Projektname"), text_input("meinprojekt", &w.name).on_input(|n| s(NpAkt::Name(n))).font(vc_gui_ui::fonts::MONO).size(13).padding(6)].spacing(6);
        if !w.name_ok() { d = d.push(text("Nur Buchstaben, Ziffern, _ und -").size(11.5).color(pal.err)) }
        d = d.push(marke(pal, "Speicherort")).push(row![text(vc_ide_model::recent::pretty(&w.ort)).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.text).width(Length::Fill), knopf(pal, "Wählen …", Some(s(NpAkt::Ort)), pal.text, false)].spacing(8).align_y(Alignment::Center));
        if w.sprache == Sprache::Basic { let o = w.dialekte(); d = d.push(marke(pal, "BASIC-Dialekt")).push(wahl(pal, o.iter().map(|x| (x.clone(), dialect_label(x).to_string())).collect(), &w.dialekt, NpAkt::Dialekt)) }
        if w.system != "mega65" { d = d.push(marke(pal, "Videonorm")).push(wahl(pal, vec![("pal".into(), "PAL".into()), ("ntsc".into(), "NTSC".into())], &w.norm, NpAkt::Norm)) }
        if w.system == "vic20" { d = d.push(marke(pal, "Speicherausbau")).push(wahl(pal, w.vic20_speicher(), &w.speicher, NpAkt::Speicher)) }
        let cfg = einst.target(&w.system);
        let global_emu = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.clone()).unwrap_or_default();
        let global_start = cfg.map(|t| t.start_on.clone()).filter(|id| einst.machine(id).is_some()).unwrap_or_default();
        d = d.push(marke(pal, "Standard-Emulator"));
        match cfg.filter(|t| !t.emulators.is_empty()) {
            Some(t) => { d = d.push(wahl(pal, t.emulators.iter().map(|e| (e.id.clone(), if e.id == global_emu { format!("{} (Einstellungen)", e.display_name()) } else { e.display_name().to_string() })).collect(), &w.emulator, NpAkt::Emu)); if w.emulator != global_emu { d = d.push(text("weicht von den Einstellungen ab: wird in der Projektdatei gespeichert").size(11.5).color(pal.warn)) } }
            None => d = d.push(row![chip(pal.accent, targets::kind_label(&einst.emulator_for(&w.system).kind)), klein(pal, "aus den Einstellungen")].spacing(8).align_y(Alignment::Center)),
        }
        let mut st: Vec<(String, String)> = vec![(String::new(), "Emulator".into())]; for m in einst.machines_for(&w.system) { st.push((m.id.clone(), format!("{} (Hardware)", m.display_name()))) }
        d = d.push(marke(pal, "Startziel (F5)")).push(wahl(pal, st, &w.start_on, NpAkt::Start));
        if w.start_on != global_start { d = d.push(text("weicht von den Einstellungen ab: wird in der Projektdatei gespeichert").size(11.5).color(pal.warn)) }
        let mut df: Vec<(String, String)> = vec![(String::new(), "keine".into())]; for x in disk_formate(&w.system) { df.push((x.to_string(), x.to_uppercase())) }
        d = d.push(marke(pal, "Diskette")).push(wahl(pal, df, &w.disk, NpAkt::Disk));
        let zs_hinweis = match w.zeichensatz { vc_media::wizard::Zeichensatz::Auto => "Dein ROM hat Vorrang (nur ein Verweis in der Datei, nichts Fremdes im Projekt); sonst der eigene Satz VC-Standard als Kopie.", vc_media::wizard::Zeichensatz::Eigen => "Immer der eigene Satz VC-Standard (CC0) als Kopie mit HERKUNFT.txt: das Projekt lässt sich weitergeben.", vc_media::wizard::Zeichensatz::Keine => "Kein Zeichensatz; du legst später im Zeichensatz-Studio einen an." };
        d = d.push(marke(pal, "Zeichensatz des Projekts")).push(wahl(pal, vec![("auto".into(), "Automatisch".into()), ("own".into(), "Nur Standard (eigen)".into()), ("none".into(), "Keinen".into())], match w.zeichensatz { vc_media::wizard::Zeichensatz::Auto => "auto", vc_media::wizard::Zeichensatz::Eigen => "own", vc_media::wizard::Zeichensatz::Keine => "none" }, NpAkt::Zeichensatz)).push(klein(pal, zs_hinweis));
        d = d.push(checkbox(w.readme).label("README.md mit Bauhinweisen anlegen").on_toggle(|b| s(NpAkt::Readme(b))).size(16).text_size(13)).push(checkbox(w.git).label("Git-Repository anlegen (.gitignore für build/ und .vcbackup/)").on_toggle(|b| s(NpAkt::Git(b))).size(16).text_size(13));
        if let Some(c) = w.karte {
            let andere: Vec<&targets::SystemInfo> = targets::installed_systems(Some(&w.data)).into_iter().filter(|x| x.id != w.system && w.karten[c].systeme.iter().any(|y| y == x.id)).collect();
            if !andere.is_empty() { d = d.push(marke(pal, "Weitere Zielsysteme")); for x in andere { let id = x.id.to_string(); d = d.push(checkbox(w.extra.iter().any(|y| *y == id)).label(x.name).on_toggle(move |b| s(NpAkt::Extra(id.clone(), b))).size(16).text_size(13)) } }
        }
        // Vorschau
        let files = w.vorschau(); let n = files.iter().filter(|f| !f.ends_with('/')).count();
        let mut baum = column![row![marke(pal, "Dateistruktur"), chip(pal.mute, format!("{n} Dateien"))].spacing(8).align_y(Alignment::Center), text(format!("{}/", w.name)).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.accent)].spacing(2);
        let notizen: std::collections::BTreeMap<String, String> = [("build".to_string(), "entsteht beim Bauen".to_string())].into_iter().collect();
        for l in vc_ide::structure::preview_tree(&files, &notizen, &|_| true) { baum = baum.push(row![iced::widget::space::horizontal().width(l.depth as f32 * 14.0), text(if l.dir { format!("{}/", l.name) } else { l.name.clone() }).size(11.5).font(vc_gui_ui::fonts::MONO).color(if l.dir { pal.text } else { pal.mute }), klein(pal, l.note.clone())].spacing(6)) }
        d = d.push(container(baum).padding(8).width(Length::Fill).style(move |_| flaeche(pal.bg, pal.line)));
        // Fuß
        let ok = w.name_ok() && w.karte.is_some();
        let norm = if w.system == "mega65" { String::new() } else { format!(" {}", w.norm.to_uppercase()) };
        let fuss = row![text(format!("Ziel: {}{norm} · {}{}", short_name(&w.system), if w.sprache == Sprache::Asm { "Assembler (vcasm)".to_string() } else { dialect_label(&w.dialekt).to_string() }, w.karte.map(|c| format!(" · {}", w.karten[c].name)).unwrap_or_default())).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute), iced::widget::space::horizontal(),
            if w.fehler.is_empty() { text("").into() } else { text(w.fehler.clone()).size(12).color(pal.err) },
            knopf(pal, "Abbrechen", Some(s(NpAkt::Abbruch)), pal.text, false), knopf(pal, "Projekt erstellen und öffnen", ok.then(|| s(NpAkt::Ok)), pal.text, true)].spacing(10).align_y(Alignment::Center).wrap().vertical_spacing(6);
        let breit = self.fenster.width / self.einst.zoom >= 1000.0;
        let inhalt: Element<Msg> = if breit { row![container(scrollable(sys.padding(iced::Padding { right: 12.0, ..Default::default() }))).width(Length::FillPortion(2)), container(scrollable(vor.padding(iced::Padding { right: 12.0, ..Default::default() }))).width(Length::FillPortion(3)), container(scrollable(d.padding(iced::Padding { right: 12.0, ..Default::default() }))).width(Length::FillPortion(3))].spacing(14).height(Length::Fill).into() }
            else { scrollable(column![sys, vor, d].spacing(16).padding(iced::Padding { right: 12.0, ..Default::default() })).height(Length::Fill).into() };
        let karte = container(column![abschnitt(pal, "Neues Projekt erstellen", "Zielsystem, Vorlage und Details wählen"), inhalt, fuss].spacing(12)).padding(18).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() }));
        Some(iced::widget::stack![hg, container(karte).padding(30).width(Length::Fill).height(Length::Fill)].into())
    }
}
