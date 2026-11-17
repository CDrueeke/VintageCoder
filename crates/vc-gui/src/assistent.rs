// SPDX-License-Identifier: GPL-3.0-or-later
//! Konfigurationsassistent (Stufe 4e): Ersteinrichtung und Wiederaufruf. Ein Assistent, zwei Wege (Schnelleinrichtung 7 Schritte, „Alles selbst“ 12),
//! dazu Teil-Assistenten. Er arbeitet auf einem Entwurf im Speicher (`vc_ide::setup::Draft`), zeigt am Ende die Änderungsliste und schreibt erst bei
//! „Übernehmen“ (Sicherung `settings.json.bak`, danach „Rückgängig“). Die Seiten für Zielsysteme, Emulatoren, Hardware, ROMs und Editor sind die der
//! Einstellungen (keine zweite Fassung); sie bearbeiten denselben Entwurf (`Satz::entwurf`). Der Assistent installiert nichts von selbst und lädt nichts herunter.
//! Im Testlauf zeigen die Dateien auf Temp-Pfade, nie auf die echte Konfiguration.
use crate::setui::{abschnitt, karte, klein};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::collections::BTreeSet;
use vc_ide::setup::{self, Applied, Change, Draft, Kind, Mode, Part, Paths, Snapshot, Topic, WizardState};

#[derive(Clone, Debug)]
pub enum WizAkt {
    Weg(Mode), Weiter, Zurueck, Springe(usize), Spaeter, Abbruch, Schliessen,
    Schema(String), Zoom(f32), Schrift(f32), System(String, bool),
    ProjekteOrdner(String), BeispieleKopieren(bool), BeispielAus(String, bool), Erstes(Erstes),
    Haken(String, bool), Uebernehmen, Rueckgaengig,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Erstes { Beispiel(String), Neu, Spaeter }

pub struct Wiz {
    pub mode: Mode, pub schritte: Vec<Topic>, pub idx: usize, pub erreicht: usize, pub paths: Paths, pub cur: Snapshot, pub draft: Draft, pub zustand: WizardState,
    pub abgewaehlt: BTreeSet<String>, pub fehler: Option<String>, pub angewandt: Option<Applied>, pub erstes: Erstes, pub einst_vorher: (crate::einstellungen::Schema, f32, f32), pub uebersprungen: BTreeSet<Topic>,
}

impl Wiz {
    pub fn weg(&self) -> Mode { self.mode }
    pub fn thema(&self) -> Topic { self.schritte[self.idx.min(self.schritte.len() - 1)] }
    fn neuer_weg(&mut self, m: Mode) { self.mode = m; self.schritte = m.steps(); self.idx = self.schritte.iter().position(|t| *t == Topic::Welcome).map_or(0, |i| i + 1).min(self.schritte.len() - 1); self.erreicht = self.erreicht.max(self.idx) }
    /// Änderungsliste des Entwurfs gegen den gespeicherten Stand, abgewählte Einträge markiert
    pub fn aenderungen(&self) -> Vec<Change> { setup::diff(&self.cur, &self.draft).into_iter().map(|mut c| { if self.abgewaehlt.contains(&c.key) { c.selected = false } c }).collect() }
}

fn a(x: WizAkt) -> Msg { Msg::Wiz(x) }
fn chip<'a>(c: Color, t: impl Into<String>) -> Element<'a, Msg> { let t = t.into(); container(text(t).size(11.5).color(c)).padding([2, 8]).style(move |_| container::Style { border: Border { color: c, width: 1.0, radius: 10.0.into() }, background: Some(Color { a: 0.12, ..c }.into()), ..Default::default() }).into() }
fn wahl<'a>(pal: Pal, opt: Vec<(String, String)>, aktuell: &str, m: impl Fn(String) -> WizAkt + 'a) -> Element<'a, Msg> {
    row(opt.into_iter().map(|(w, l)| { let on = w == aktuell; button(text(l).size(13).color(pal.text)).padding([5, 12]).on_press(a(m(w))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 7.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(6).wrap().vertical_spacing(6).into()
}

impl App {
    pub(crate) fn wiz_pfade(&self) -> Option<Paths> {
        if cfg!(test) { let d = self.satz.datei.as_ref()?.parent()?.to_path_buf(); let id = std::process::id(); return Some(Paths { settings: d.join(format!("vcgui-wiz-{id}-settings.json")), view: d.join(format!("vcgui-wiz-{id}-view.json")), wizard: d.join(format!("vcgui-wiz-{id}-wizard.json")) }) }
        Paths::current()
    }
    /// Assistent öffnen: `weg` None = Willkommensseite, sonst direkt in den Weg
    pub(crate) fn wiz_oeffnen(&mut self, weg: Option<Mode>) {
        let Some(paths) = self.wiz_pfade() else { self.log.push("Assistent: kein Konfigurationsordner.".into()); return };
        let cur = match setup::load_snapshot(&paths) { Ok(s) => s, Err(e) => { self.log.push(format!("Assistent: {e}")); return } };
        let erst = !paths.wizard.exists() && !paths.settings.exists();
        let mut draft = if erst { Draft::for_first_run(&cur) } else { Draft::from_snapshot(&cur) };
        draft.view = setup::ViewPrefs { scheme: match self.einst.schema { crate::einstellungen::Schema::Hell => "light", crate::einstellungen::Schema::System => "system", _ => "dark" }.into(), zoom: self.einst.zoom, font: self.einst.editor_schrift };
        let zustand = WizardState::load(&paths.wizard);
        self.satz.entwurf = draft.settings.clone(); self.satz.gespeichert = cur.settings.clone();
        let mode = weg.unwrap_or(Mode::Quick); let mut w = Wiz { mode, schritte: mode.steps(), idx: 0, erreicht: 0, paths, cur, draft, zustand, abgewaehlt: BTreeSet::new(), fehler: None, angewandt: None, erstes: Erstes::Spaeter, einst_vorher: (self.einst.schema, self.einst.zoom, self.einst.editor_schrift), uebersprungen: BTreeSet::new() };
        if weg.is_some() { w.neuer_weg(mode); w.idx = 0; }
        self.wiz = Some(w);
    }
    fn wiz_stand_uebernehmen(&mut self) { if let Some(w) = &mut self.wiz { w.draft.settings = self.satz.entwurf.clone() } }
    fn wiz_darstellung(&mut self) { if let Some(w) = &self.wiz { self.einst.schema = match w.draft.view.scheme.as_str() { "light" => crate::einstellungen::Schema::Hell, "system" => crate::einstellungen::Schema::System, _ => crate::einstellungen::Schema::Dunkel }; self.einst.zoom = w.draft.view.zoom; self.einst.editor_schrift = w.draft.view.font } }

    pub(crate) fn wiz_akt(&mut self, x: WizAkt) -> Task<Msg> {
        self.wiz_stand_uebernehmen();
        let Some(w) = self.wiz.as_mut() else { return Task::none() };
        match x {
            WizAkt::Weg(m) => { w.neuer_weg(m); w.idx = w.schritte.iter().position(|t| *t != Topic::Welcome).unwrap_or(0).max(if m == Mode::Quick || m == Mode::Full { 1 } else { 0 }).min(w.schritte.len() - 1); w.erreicht = w.erreicht.max(w.idx) }
            WizAkt::Weiter => { if w.idx + 1 < w.schritte.len() { w.idx += 1; w.erreicht = w.erreicht.max(w.idx) } }
            WizAkt::Zurueck => { w.idx = w.idx.saturating_sub(1) }
            WizAkt::Springe(i) => { if i <= w.erreicht && i < w.schritte.len() { w.idx = i } }
            WizAkt::Spaeter => { let t = w.thema(); w.zustand.mark_later(t); w.zustand.resume_step = t.id().to_string(); w.uebersprungen.insert(t); if let Err(e) = w.zustand.save(&w.paths.wizard) { w.fehler = Some(e) } else { return self.wiz_schliessen(false) } }
            WizAkt::Abbruch => return self.wiz_schliessen(false),
            WizAkt::Schliessen => return self.wiz_schliessen(true),
            WizAkt::Schema(s) => { w.draft.view.scheme = s; self.wiz_darstellung() }
            WizAkt::Zoom(z) => { w.draft.view.zoom = (z * 10.0).round() / 10.0; w.draft.view.zoom = w.draft.view.zoom.clamp(0.7, 2.5); self.wiz_darstellung() }
            WizAkt::Schrift(f) => { w.draft.view.font = f.clamp(9.0, 36.0); self.wiz_darstellung() }
            WizAkt::System(id, an) => { setup::set_system(&mut self.satz.entwurf, &id, an); self.wiz_stand_uebernehmen() }
            WizAkt::ProjekteOrdner(p) => { self.satz.entwurf.projects_dir = p; self.wiz_stand_uebernehmen() }
            WizAkt::BeispieleKopieren(b) => w.draft.examples.copy = b,
            WizAkt::BeispielAus(n, aus) => { if aus { w.draft.examples.excluded.insert(n); } else { w.draft.examples.excluded.remove(&n); } }
            WizAkt::Erstes(e) => w.erstes = e,
            WizAkt::Haken(k, an) => { if an { w.abgewaehlt.remove(&k); } else { w.abgewaehlt.insert(k); } }
            WizAkt::Uebernehmen => {
                let abg = w.abgewaehlt.clone();
                match setup::apply_to(&w.paths, &w.cur, &w.draft, &|k| !abg.contains(k)) {
                    Ok(ap) => {
                        if !cfg!(test) { setup::apply_replace_memory(&ap) }
                        self.satz.gespeichert = ap.settings.clone(); self.satz.entwurf = ap.settings.clone();
                        w.draft.view = ap.view.clone(); w.cur = Snapshot { settings: ap.settings.clone(), view: ap.view.clone(), disk_hash: setup::hash_of(&std::fs::read(&w.paths.settings).unwrap_or_default()) };
                        w.zustand.first_run_done = true; w.zustand.mode = w.mode.id().to_string(); w.zustand.later.clear(); w.zustand.resume_step.clear(); w.zustand.last_run = datum_heute();
                        w.zustand.examples_copied.extend(ap.copy.copied.iter().cloned()); let _ = w.zustand.save(&w.paths.wizard);
                        self.log.push(format!("Einrichtung übernommen{}", ap.backup.as_ref().map(|b| format!(" (Sicherung {})", b.display())).unwrap_or_default()));
                        w.angewandt = Some(ap); w.fehler = None; self.wiz_darstellung(); if let Some(w) = &mut self.wiz { if let Some(i) = w.schritte.iter().position(|t| *t == Topic::Done) { w.idx = i; w.erreicht = w.erreicht.max(i) } }
                    }
                    Err(e) => w.fehler = Some(e),
                }
            }
            WizAkt::Rueckgaengig => {
                if let Some(ap) = w.angewandt.take() { match setup::undo_to(&w.paths, &ap) { Ok(()) => { if let Ok(s) = setup::load_snapshot(&w.paths) { if !cfg!(test) { setup::apply_replace_memory_settings(s.settings.clone()) } self.satz.gespeichert = s.settings.clone(); self.satz.entwurf = s.settings.clone(); w.draft = Draft::from_snapshot(&s); w.cur = s } self.log.push("Einrichtung rückgängig gemacht.".into()); self.wiz_darstellung(); if let Some(w) = &mut self.wiz { w.idx = 0; } } Err(e) => w.fehler = Some(e) } }
            }
        }
        Task::none()
    }
    /// Assistent beenden; ohne Übernehmen wird die Darstellung zurückgesetzt
    fn wiz_schliessen(&mut self, fertig: bool) -> Task<Msg> {
        let Some(w) = self.wiz.take() else { return Task::none() };
        if !fertig || w.angewandt.is_none() { if !fertig { (self.einst.schema, self.einst.zoom, self.einst.editor_schrift) = w.einst_vorher; self.satz.entwurf = self.satz.gespeichert.clone() } }
        if fertig && w.angewandt.is_some() { return match w.erstes { Erstes::Beispiel(p) => self.update_innen(Msg::Geoeffnet(Some(std::path::PathBuf::from(p)))), Erstes::Neu => { self.neuprojekt_oeffnen(); Task::none() } Erstes::Spaeter => Task::none() } }
        Task::none()
    }

    pub(crate) fn wiz_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let w = self.wiz.as_ref()?; let t = w.thema();
        // Schienen: Schritte mit Zustand
        let mut schiene = column![text("Einrichtung").size(15).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), chip(pal.accent, match w.mode { Mode::Quick => "Schnelleinrichtung".to_string(), Mode::Full => "Alles selbst".into(), Mode::Part(p) => format!("Teil: {p:?}") })].spacing(8);
        for (i, s) in w.schritte.iter().enumerate() {
            let on = i == w.idx; let erreichbar = i <= w.erreicht;
            let badge = if w.uebersprungen.contains(s) { "•" } else if i < w.idx { "✓" } else { " " };
            schiene = schiene.push(button(row![text(badge).size(12).color(pal.ok).width(14), text(s.title()).size(13).color(if erreichbar { pal.text } else { pal.mute })].spacing(6)).width(Length::Fill).padding([5, 8]).on_press_maybe(erreichbar.then(|| a(WizAkt::Springe(i)))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel.into() }), border: Border { color: if on { pal.accent } else { pal.panel.into() }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }));
        }
        let inhalt: Element<Msg> = self.wiz_schritt(pal, w, t);
        let letzter = w.idx + 1 >= w.schritte.len();
        let nav = row![knopf(pal, "Abbrechen", Some(a(WizAkt::Abbruch)), pal.text, false), knopf(pal, "Später fortsetzen", (t != Topic::Welcome && t != Topic::Done).then(|| a(WizAkt::Spaeter)), pal.text, false), iced::widget::space::horizontal(),
            if let Some(e) = &w.fehler { text(e.clone()).size(12).color(pal.err).into() } else { Element::from(text("")) },
            knopf(pal, "Zurück", (w.idx > 0 && t != Topic::Done).then(|| a(WizAkt::Zurueck)), pal.text, false),
            match t { Topic::Summary => knopf(pal, "Übernehmen", Some(a(WizAkt::Uebernehmen)), pal.text, true), Topic::Done => knopf(pal, "Fertig", Some(a(WizAkt::Schliessen)), pal.text, true), Topic::Welcome => text("").into(), _ => knopf(pal, "Weiter", (!letzter).then(|| a(WizAkt::Weiter)), pal.text, true) }].spacing(8).align_y(Alignment::Center).wrap().vertical_spacing(6);
        let breit = self.fenster.width / self.einst.zoom >= 900.0;
        let rumpf: Element<Msg> = if breit { row![container(scrollable(schiene)).width(200), container(scrollable(container(inhalt).padding(iced::Padding { right: 12.0, ..Default::default() }))).width(Length::Fill)].spacing(16).height(Length::Fill).into() } else { scrollable(column![schiene, inhalt].spacing(14)).height(Length::Fill).into() };
        let kartei = container(column![rumpf, nav].spacing(12)).padding(18).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.6, ..Color::BLACK }.into()), ..Default::default() }));
        Some(iced::widget::stack![hg, container(kartei).padding(24).width(Length::Fill).height(Length::Fill)].into())
    }

    fn wiz_schritt<'a>(&'a self, pal: Pal, w: &'a Wiz, t: Topic) -> Element<'a, Msg> {
        match t {
            Topic::Welcome => {
                let mut c = column![abschnitt(pal, "Willkommen bei VintageCoder", "Der Assistent richtet Zielsysteme, Emulatoren, ROMs, Hardware und Aussehen ein. Er ändert nur, was du am Ende bestätigst, und sichert vorher deine bisherige Einstellungsdatei.")].spacing(12);
                c = c.push(karte(pal, vec![text("Schnelleinrichtung (7 Schritte)").size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), klein(pal, "Design, Zielsysteme, Toolchain und erstes Projekt; Emulatoren, ROMs und Hardware werden automatisch gefunden und später in den Einstellungen verfeinert."), knopf(pal, "Schnelleinrichtung starten", Some(a(WizAkt::Weg(Mode::Quick))), pal.text, true)]));
                c = c.push(karte(pal, vec![text("Alles selbst einstellen (12 Schritte)").size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), klein(pal, "Zusätzlich Emulatoren, ROMs, echte Hardware, Startziel F5 und Editor."), knopf(pal, "Alles selbst einstellen", Some(a(WizAkt::Weg(Mode::Full))), pal.text, false)]));
                c = c.push(karte(pal, vec![text("Nur einen Teil").size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into(), row![knopf(pal, "ROMs", Some(a(WizAkt::Weg(Mode::Part(Part::Roms)))), pal.text, false), knopf(pal, "Emulatoren", Some(a(WizAkt::Weg(Mode::Part(Part::Emulators)))), pal.text, false), knopf(pal, "Hardware", Some(a(WizAkt::Weg(Mode::Part(Part::Hardware)))), pal.text, false), knopf(pal, "Design und Editor", Some(a(WizAkt::Weg(Mode::Part(Part::DesignEditor)))), pal.text, false)].spacing(8).wrap().vertical_spacing(6).into()]));
                if !w.zustand.later.is_empty() { c = c.push(klein(pal, format!("Offen von der letzten Einrichtung: {}", w.zustand.later.iter().filter_map(|i| Topic::from_id(i)).map(|t| t.title()).collect::<Vec<_>>().join(", ")))) }
                c.into()
            }
            Topic::Design => {
                let v = &w.draft.view;
                column![abschnitt(pal, "Design", "Gilt für die ganze Oberfläche und wirkt sofort; „Abbrechen“ stellt das Alte wieder her."), karte(pal, vec![
                    row![text("Farbschema").size(13).width(170), wahl(pal, vec![("dark".into(), "Dunkel".into()), ("light".into(), "Hell".into()), ("system".into(), "System".into())], &v.scheme, WizAkt::Schema)].spacing(12).align_y(Alignment::Center).into(),
                    row![text("Oberfläche").size(13).width(170), knopf(pal, "−", Some(a(WizAkt::Zoom(v.zoom - 0.1))), pal.text, false), text(format!("{:.0} %", v.zoom * 100.0)).size(13), knopf(pal, "+", Some(a(WizAkt::Zoom(v.zoom + 0.1))), pal.text, false)].spacing(10).align_y(Alignment::Center).into(),
                    row![text("Editor-Schrift").size(13).width(170), knopf(pal, "−", Some(a(WizAkt::Schrift(v.font - 1.0))), pal.text, false), text(format!("{:.0} pt", v.font)).size(13), knopf(pal, "+", Some(a(WizAkt::Schrift(v.font + 1.0))), pal.text, false)].spacing(10).align_y(Alignment::Center).into()])].spacing(10).into()
            }
            Topic::Systems => {
                let mut k = vec![];
                for i in vc_ide::targets::installed_systems(Some(&self.data)) { let an = self.satz.entwurf.target(i.id).is_some(); let id = i.id.to_string(); k.push(checkbox(an).label(format!("{} · {}", i.name, i.cpu)).on_toggle(move |b| a(WizAkt::System(id.clone(), b))).size(16).text_size(13).into()) }
                column![abschnitt(pal, "Zielsysteme", "Für welche Systeme willst du programmieren? Abgewählte Systeme werden nur ausgeblendet, nichts geht verloren."), karte(pal, k)].spacing(10).into()
            }
            Topic::Toolchain => {
                let mut k = vec![klein(pal, "VintageCoder hat einen eigenen Assembler; es wird kein externer Assembler benötigt. Emulatoren werden gesucht (PATH und übliche Ordner), nichts wird gestartet oder installiert.")];
                for (kind, p) in self.werkzeuge() { k.push(row![text("●").size(11).color(if p.found { pal.ok } else { pal.mute }), text(vc_ide::targets::kind_label(kind)).size(13).font(vc_gui_ui::fonts::MONO).color(pal.text), iced::widget::space::horizontal(), if p.found { chip(pal.ok, "bereit") } else { chip(pal.mute, "nicht gefunden") }].spacing(8).align_y(Alignment::Center).into()) }
                k.push(klein(pal, "Fehlende Emulatoren trägst du im Schritt „Emulatoren“ (Vollweg) oder später unter Einstellungen ▸ Zielsysteme ein; dort gibt es auch die portable Installation."));
                column![abschnitt(pal, "Toolchain", "Was ist auf diesem Rechner schon da?"), karte(pal, k)].spacing(10).into()
            }
            Topic::Emulators | Topic::Hardware | Topic::StartTarget => column![klein(pal, match t { Topic::Emulators => "Emulatoren je Zielsystem: Pfad (leer = automatisch suchen), Standard, Startparameter.", Topic::Hardware => "Echte Maschinen (Ultimate, MEGA65): Adresse bzw. Schnittstelle, Verbindungstest nur auf Klick.", _ => "Startziel F5 je System: Emulator oder eine eingetragene Maschine." }), self.seite_ziele(pal)].spacing(10).into(),
            Topic::Roms => self.seite_roms(pal),
            Topic::Editor => self.seite_editor(pal),
            Topic::Projects => {
                let beisp = setup::list_examples(&self.data.parent().map(|p| p.join("examples")).unwrap_or_default(), &self.satz.entwurf.targets.iter().map(|t| t.system.clone()).collect::<Vec<_>>());
                let mut k = vec![row![text("Projektordner").size(13).width(170), crate::pfadfeld::pfadfeld(pal, "~/VintageCoder", &self.satz.entwurf.projects_dir, crate::pfadfeld::PfadArt::Ordner, Length::Fill, |s| a(WizAkt::ProjekteOrdner(s)))].spacing(12).align_y(Alignment::Center).into(), klein(pal, "Leer = Voreinstellung ~/VintageCoder. Das ist der vorgeschlagene Ort für neue Projekte.")];
                k.push(checkbox(w.draft.examples.copy).label("Beispielprojekte in meinen Projektordner kopieren (Ordner „Beispiele“)").on_toggle(|b| a(WizAkt::BeispieleKopieren(b))).size(16).text_size(13).into());
                if w.draft.examples.copy { for it in beisp.iter().take(40) { let key = format!("{}", it.name); let aus = w.draft.examples.excluded.contains(&key); k.push(checkbox(!aus).label(it.name.clone()).on_toggle(move |b| a(WizAkt::BeispielAus(key.clone(), !b))).size(15).text_size(12.5).into()) } }
                let mut wahl_erst = vec![("spaeter".to_string(), "Später entscheiden".to_string()), ("neu".to_string(), "Neues Projekt anlegen".to_string())];
                if let Some(b) = beisp.first() { wahl_erst.push((format!("bsp:{}", b.src.display()), format!("Beispiel „{}“ öffnen", b.name))) }
                let akt = match &w.erstes { Erstes::Spaeter => "spaeter".to_string(), Erstes::Neu => "neu".into(), Erstes::Beispiel(p) => format!("bsp:{p}") };
                k.push(text("Erstes Projekt").size(13).font(vc_gui_ui::fonts::UI_FETT).color(pal.text).into()); k.push(wahl(pal, wahl_erst, &akt, |x| WizAkt::Erstes(if x == "neu" { Erstes::Neu } else if let Some(p) = x.strip_prefix("bsp:") { Erstes::Beispiel(p.to_string()) } else { Erstes::Spaeter })));
                column![abschnitt(pal, "Projekte", "Wo liegen deine Projekte, und womit fängst du an?"), karte(pal, k)].spacing(10).into()
            }
            Topic::Summary => {
                let ch = w.aenderungen(); let n = ch.iter().filter(|c| c.kind != Kind::Same && c.selected).count();
                let mut k = vec![];
                if ch.iter().all(|c| c.kind == Kind::Same) { k.push(klein(pal, "Es gibt nichts zu ändern: alles ist schon so eingerichtet.")) }
                for c in ch.iter().filter(|c| c.kind != Kind::Same) { let key = c.key.clone(); let kt = match c.kind { Kind::New => "neu", Kind::Changed => "geändert", Kind::Removed => "entfernt", Kind::Same => "" };
                    k.push(row![checkbox(c.selected).on_toggle(move |b| a(WizAkt::Haken(key.clone(), b))).size(16), chip(pal.accent, c.area), text(c.label.clone()).size(13).color(pal.text), chip(match c.kind { Kind::New => pal.ok, Kind::Removed => pal.err, _ => pal.warn }, kt), klein(pal, if c.old.is_empty() { c.new.clone() } else { format!("{} → {}", c.old, c.new) })].spacing(8).align_y(Alignment::Center).wrap().into()) }
                column![abschnitt(pal, "Zusammenfassung", &format!("{n} Änderung(en) werden übernommen. Vorher entsteht eine Sicherung (settings.json.bak); danach lässt sich alles rückgängig machen.")), karte(pal, k)].spacing(10).into()
            }
            Topic::Done => {
                let mut c = column![abschnitt(pal, "Fertig", "Die Einrichtung ist übernommen. Alles lässt sich unter Einstellungen jederzeit ändern; der Assistent ist über die Palette („Konfigurationsassistent“) erneut erreichbar.")].spacing(10);
                if let Some(ap) = &w.angewandt { let mut k: Vec<Element<Msg>> = vec![]; if let Some(b) = &ap.backup { k.push(klein(pal, format!("Sicherung: {}", b.display()))) } for n in &ap.notes { k.push(klein(pal, n.clone())) } if !ap.copy.copied.is_empty() { k.push(klein(pal, format!("Beispiele kopiert: {}", ap.copy.copied.join(", ")))) } k.push(knopf(pal, "Rückgängig", Some(a(WizAkt::Rueckgaengig)), pal.text, false)); c = c.push(karte(pal, k)) }
                c.into()
            }
        }
    }
}

fn datum_heute() -> String { let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64; let (z, _) = (s / 86400, 0); let (y, m, d) = civil(z); format!("{y:04}-{m:02}-{d:02}") }
/// Tage seit 1970 → (Jahr, Monat, Tag), nach Howard Hinnant
fn civil(z: i64) -> (i64, u32, u32) { let z = z + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097); let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let y = yoe + era * 400; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153; let d = (doy - (153 * mp + 2) / 5 + 1) as u32; let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; (if m <= 2 { y + 1 } else { y }, m, d) }

#[cfg(test)]
mod tests { use super::*; #[test] fn datum_stimmt() { assert_eq!(civil(0), (1970, 1, 1)); assert_eq!(civil(19_000), (2022, 1, 8)); } }
