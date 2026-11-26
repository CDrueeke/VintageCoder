// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildschirm-Studio für Textbildschirme (`.vcscreen`): Zeichenfläche als Bild (Zeichen aus dem Zeichensatz, Farben des Systems), Zeichenwahl,
//! Farben, Zeichensatz, Werkzeuge. Das Modell ist `vc_ide_model::screendoc::ScreenDoc`; hier wird nur angezeigt und weitergeleitet.
use super::*;
use crate::studio::{Ansicht, Farben};
use vc_gui_ui::muster;
use iced::widget::{canvas, text};
use vc_ide_model::glyphs::Bild;
use vc_ide_model::screendoc::{ScreenDoc, Werkzeug};

pub struct ScreenStudio { pub doc: ScreenDoc, pub zoomansicht: vc_gui_ui::flaeche::Ansicht, pub skala: std::cell::Cell<usize>, pub ansicht: Ansicht, pub ueber: Option<(usize, usize)>, pub rechts: bool, pub mehr: bool, pub zerlegen: Option<(usize, usize)>, pub karten_zu: std::collections::BTreeSet<&'static str> }

#[derive(Clone, Debug, PartialEq)]
pub enum SAkt { Werkzeug(Werkzeug), Zeichen(u8), Farbe(usize), Rolle(&'static str, usize), Zeichensatz(Option<String>), Undo, Redo, Speichern, Exportieren, Zoom(i32), Einpassen, Schieben(f32, f32), Mehr, Karte(&'static str), ZerlegenDialog(Option<(usize, usize)>), Ansicht(Ansicht), Cursor(i32, i32), Zerlegen(usize, usize) }
#[derive(Clone, Debug, PartialEq)]
pub enum SMaus { Unten { x: usize, y: usize, rechts: bool }, Zug { x: usize, y: usize }, Los { x: usize, y: usize }, Ueber(Option<(usize, usize)>) }

fn bild_handle(b: &Bild) -> iced::widget::image::Handle { crate::bildcache::handle(b.w as u32, b.h as u32, &b.rgba) }

/// Größe des Bildschirmbilds bei Zoom 1 in Bildpunkten (Rand eingerechnet): die „Einheiten“ für Einpassen und Zoom
fn einheiten(d: &ScreenDoc) -> (f32, f32) { (((d.w + 2 * d.rand()) * 8) as f32, ((d.h + 2 * d.rand()) * 8) as f32) }
/// Höchste Bildschirm-Vergrößerung (das Bild wird in dieser Stufe neu erzeugt)
const SKALA_MAX: usize = 8;
/// Linke obere Ecke des Bildes (Breite `bw`, Höhe `bh` in Bildpunkten) im Zeichenbereich `b`: zentriert plus begrenzte Verschiebung
fn lage(b: iced::Size, bw: f32, bh: f32, a: &vc_gui_ui::flaeche::Ansicht) -> (f32, f32) {
    a.ecke(b, bw, bh)
}

/// Bildebene: das fertige Bild an der Stelle der Zeichenfläche (eigene Ebene unter den Linien, damit nichts flackert)
struct BildEbene<'a> { s: &'a ScreenStudio, bild: iced::widget::image::Handle, bw: f32, bh: f32 }
impl<'a> canvas::Program<Msg> for BildEbene<'a> {
    type State = ();
    fn update(&self, _st: &mut (), _e: &iced::Event, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> { None }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let (ox, oy) = lage(b.size(), self.bw, self.bh, &self.s.zoomansicht);
        f.draw_image(iced::Rectangle::new(iced::Point::new(ox, oy), iced::Size::new(self.bw, self.bh)), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        vec![f.into_geometry()]
    }
}

/// Zeichenfläche (obere Ebene): Klicks, Cursor, aktive Zelle, Ziehvorschau
struct Flaeche<'a> { s: &'a ScreenStudio, skala: usize, bw: f32, bh: f32, pal: Pal, rel: String }
#[derive(Default)] struct FlaechenZustand { unten: Option<bool>, ueber: Option<(usize, usize)>, gesten: vc_gui_ui::flaeche::Gesten }
impl<'a> Flaeche<'a> {
    /// Ecke des Zellrasters (hinter dem Rand) und Zellkante
    fn raster(&self, b: iced::Size) -> (f32, f32, f32) { let (ox, oy) = lage(b, self.bw, self.bh, &self.s.zoomansicht); let rd = (self.s.doc.rand() * 8 * self.skala) as f32; (ox + rd, oy + rd, self.skala as f32 * 8.0) }
    fn zelle(&self, b: iced::Rectangle, p: iced::Point) -> Option<(usize, usize)> {
        let (x0, y0, sk) = self.raster(b.size()); let (x, y) = ((p.x - x0) / sk, (p.y - y0) / sk);
        (x >= 0.0 && y >= 0.0 && (x as usize) < self.s.doc.w && (y as usize) < self.s.doc.h).then(|| (x as usize, y as usize))
    }
}
impl<'a> canvas::Program<Msg> for Flaeche<'a> {
    type State = FlaechenZustand;
    fn update(&self, st: &mut FlaechenZustand, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        use iced::mouse::{Button, Event as M};
        // Leertaste + Ziehen verschiebt, Cmd/Strg + Mausrad zoomt (Tasten bleiben bei der Befehlstabelle)
        if let Some(g) = st.gesten.ereignis(e, b, c) {
            use vc_gui_ui::flaeche::Geste;
            return match g { Geste::Erfasst => Some(canvas::Action::capture()), Geste::Schieben(dx, dy) => Some(canvas::Action::publish(Msg::ScreenAkt(self.rel.clone(), SAkt::Schieben(dx, dy))).and_capture()), Geste::Zoom(plus) => Some(canvas::Action::publish(Msg::ScreenAkt(self.rel.clone(), SAkt::Zoom(if plus { 1 } else { -1 }))).and_capture()) }
        }
        let rel = self.rel.clone(); let send = |m: SMaus| Some(canvas::Action::publish(Msg::ScreenMaus(rel.clone(), m)).and_capture());
        match e {
            iced::Event::Mouse(M::ButtonPressed(k @ (Button::Left | Button::Right))) => { let p = c.position_in(b)?; let (x, y) = self.zelle(b, p)?; st.unten = Some(*k == Button::Right); send(SMaus::Unten { x, y, rechts: *k == Button::Right }) }
            iced::Event::Mouse(M::CursorMoved { .. }) if st.unten.is_some() => { let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let (x, y) = self.zelle(b, p)?; st.ueber = Some((x, y)); send(SMaus::Zug { x, y }) }
            iced::Event::Mouse(M::CursorMoved { .. }) => { let z = c.position_in(b).and_then(|p| self.zelle(b, p)); if z != st.ueber { st.ueber = z; Some(canvas::Action::publish(Msg::ScreenMaus(rel, SMaus::Ueber(z)))) } else { None } }
            iced::Event::Mouse(M::ButtonReleased(Button::Left | Button::Right)) if st.unten.is_some() => { st.unten = None; let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let (x, y) = self.zelle(b, p).unwrap_or((0, 0)); send(SMaus::Los { x, y }) }
            _ => None,
        }
    }
    fn draw(&self, st: &FlaechenZustand, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        self.s.zoomansicht.platz.set((b.width, b.height));
        let mut f = canvas::Frame::new(r, b.size()); let (x0, y0, sk) = self.raster(b.size()); let pal = self.pal; let d = &self.s.doc;
        let zelle = |x: usize, y: usize| iced::Point::new(x0 + x as f32 * sk, y0 + y as f32 * sk);
        // Ziehvorschau (Rahmen): halbtransparent von der Startzelle bis unter den Zeiger; Esc bricht ab
        if let (Some(a), Some(h)) = (d.von.filter(|_| d.werkzeug == Werkzeug::Rahmen), st.ueber) { for (px, py) in ScreenDoc::rahmen_zellen(a, h) { f.fill_rectangle(zelle(px, py), iced::Size::new(sk, sk), Color { a: 0.35, ..pal.accent }) } }
        // Textcursor und aktive Zelle als Doppelrahmen (innen Akzent 2 px, außen Text gestrichelt) mit Marken an beiden Achsen
        let doppel = |f: &mut canvas::Frame, x: usize, y: usize, voll: bool| {
            let p = zelle(x, y);
            f.stroke_rectangle(iced::Point::new(p.x - 1.0, p.y - 1.0), iced::Size::new(sk + 2.0, sk + 2.0), canvas::Stroke::default().with_color(if voll { pal.accent } else { pal.text }).with_width(2.0));
            f.stroke_rectangle(iced::Point::new(p.x - 3.5, p.y - 3.5), iced::Size::new(sk + 7.0, sk + 7.0), canvas::Stroke { line_dash: canvas::LineDash { segments: &[4.0, 3.0], offset: 0 }, ..canvas::Stroke::default().with_color(pal.text).with_width(1.5) });
            f.fill_rectangle(iced::Point::new(p.x, y0 - 8.0), iced::Size::new(sk, 4.0), pal.accent); f.fill_rectangle(iced::Point::new(x0 - 8.0, p.y), iced::Size::new(4.0, sk), pal.accent);
        };
        if let Some((x, y)) = st.ueber.filter(|(x, y)| *x < d.w && *y < d.h) { doppel(&mut f, x, y, true) }
        let (cx, cy) = d.cursor; doppel(&mut f, cx, cy, false);
        if let Some(a) = d.von { f.stroke_rectangle(zelle(a.0, a.1), iced::Size::new(sk, sk), canvas::Stroke::default().with_color(pal.accent).with_width(2.0)) }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &FlaechenZustand, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Crosshair }
}

/// Zeichenwahl: alle Zeichen des Zeichensatzes als 16×16-Tafel
struct Tafel { bild: iced::widget::image::Handle, kante: f32, gewaehlt: u8, pal: Pal, rel: String }
impl canvas::Program<Msg> for Tafel {
    type State = ();
    fn update(&self, _st: &mut (), e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = e {
            let p = c.position_in(b)?; let (x, y) = ((p.x / self.kante) as usize, (p.y / self.kante) as usize);
            if x < 16 && y < 16 { return Some(canvas::Action::publish(Msg::ScreenAkt(self.rel.clone(), SAkt::Zeichen((y * 16 + x) as u8))).and_capture()) }
        }
        None
    }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size());
        f.draw_image(iced::Rectangle::new(iced::Point::ORIGIN, b.size()), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        let (x, y) = ((self.gewaehlt % 16) as f32, (self.gewaehlt / 16) as f32);
        f.stroke_rectangle(iced::Point::new(x * self.kante, y * self.kante), iced::Size::new(self.kante, self.kante), canvas::Stroke::default().with_color(self.pal.accent).with_width(2.0));
        vec![f.into_geometry()]
    }
}

impl App {
    pub(crate) fn screen_oeffnen(&mut self, rel: &str) {
        if !self.screens.contains_key(rel) {
            match ScreenDoc::oeffnen(&self.root, rel, &self.data) { Ok(doc) => { self.screens.insert(rel.to_string(), ScreenStudio { doc, zoomansicht: vc_gui_ui::flaeche::Ansicht::mit_max(SKALA_MAX as f32), skala: std::cell::Cell::new(2), ansicht: Ansicht::Projekt, ueber: None, rechts: false, mehr: false, zerlegen: None, karten_zu: Default::default() }); } Err(e) => { self.log.push(e); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }

    pub(crate) fn screen_maus(&mut self, rel: &str, m: SMaus) {
        let Some(s) = self.screens.get_mut(rel) else { return };
        match m {
            SMaus::Unten { x, y, rechts } => { s.rechts = rechts; s.doc.maus_unten(x, y, rechts) }
            SMaus::Zug { x, y } => { let r = s.rechts; s.doc.maus_zug(x, y, r) }
            SMaus::Los { x, y } => { let r = s.rechts; s.doc.maus_los(x, y, r) }
            SMaus::Ueber(z) => s.ueber = z,
        }
    }

    pub(crate) fn screen_aktion(&mut self, rel: &str, a: SAkt) {
        if a == SAkt::Exportieren { self.export_oeffnen(); return }
        let Some(s) = self.screens.get_mut(rel) else { return };
        match a {
            SAkt::Werkzeug(w) => s.doc.werkzeug_waehlen(w), SAkt::Zeichen(c) => s.doc.zeichen_waehlen(c), SAkt::Farbe(i) => s.doc.farbe_waehlen(i), SAkt::Rolle(r, i) => s.doc.rollenfarbe(r, i),
            SAkt::Zeichensatz(z) => s.doc.zeichensatz_waehlen(z), SAkt::Undo => s.doc.undo(), SAkt::Redo => s.doc.redo(),
            SAkt::Speichern => match s.doc.save() { Ok(()) => self.log.push(vc_i18n::t!("screenstudio-gespeichert-rel", rel = &rel)), Err(e) => self.log.push(e) },
            SAkt::Zoom(d) => { let (w, h) = einheiten(&s.doc); s.zoomansicht.stufe(d > 0, w, h); }
            SAkt::Einpassen => s.zoomansicht.einpassen(), SAkt::Schieben(dx, dy) => { let (w, h) = einheiten(&s.doc); s.zoomansicht.schieben(dx, dy, w, h) }
            SAkt::Mehr => s.mehr = !s.mehr, SAkt::Karte(k) => { if !s.karten_zu.remove(k) { s.karten_zu.insert(k); } } SAkt::ZerlegenDialog(z) => s.zerlegen = z,
            SAkt::Ansicht(x) => s.ansicht = x, SAkt::Cursor(dx, dy) => s.doc.cursor_bewegen(dx, dy),
            SAkt::Zerlegen(tw, th) => { self.screen_zerlegen(rel, tw, th); }
            SAkt::Exportieren => {}
        }
    }

    /// „In Kacheln zerlegen“: gleiche Blöcke werden Kacheln; Kachelsatz und Bank entstehen neben dem Bildschirm, der Bildschirm bleibt unverändert
    pub(crate) fn screen_zerlegen(&mut self, rel: &str, tw: usize, th: usize) {
        let Some(s) = self.screens.get(rel) else { return };
        let r = (|| -> Result<(String, String, String), String> {
            let profiles = vc_media::load_profiles(&self.data)?; let p = vc_media::profile(&profiles, s.doc.media["profile"].as_str().unwrap_or(""))?.clone();
            let satzprofil = profiles.iter().find(|x| x.kind == "tileset" && x.target == p.target).map(|x| x.id.clone()).ok_or_else(|| vc_i18n::t!("screenstudio-fuer-p1-gibt-es-kein-kachelsatz", p1 = &p.target))?;
            let name = vc_media::naming::stem(rel).to_string();
            let mut d = vc_ide_model::bankdoc::decompose(&s.doc.media, &p, &satzprofil, tw, th, (0, 0), &name)?;
            let (satz, bank) = vc_ide_model::bankdoc::write_decomp(&self.root, rel, &mut d)?;
            Ok((satz, bank, vc_i18n::t!("screenstudio-p1-kacheln-p2-zellen-patches-p3", p1 = &d.tiles, p2 = &d.patches, p3 = &d.bytes_before, p4 = &d.bytes_after)))
        })();
        match r { Ok((satz, bank, info)) => { self.log.push(vc_i18n::t!("screenstudio-in-kacheln-zerlegt-tw-th-info", tw = &tw, th = &th, info = &info, satz = &satz, bank = &bank)); self.projekt_neu_laden(); self.oeffne(&bank) } Err(e) => { self.log.push(vc_i18n::t!("screenstudio-zerlegen-nicht-moeglich-e", e = &e)); self.schublade_zeigen() } }
    }

    /// Tasten im Bildschirm-Studio: Text tippen (Werkzeug Text), sonst Einzeltasten für die Werkzeuge; `true` = verarbeitet
    pub(crate) fn screen_taste(&mut self, rel: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
        if m.command() || m.alt() || m.control() { return false }
        let Some(s) = self.screens.get(rel) else { return false }; let text_modus = s.doc.werkzeug == Werkzeug::Text;
        use keyboard::key::Named;
        let a = match k {
            keyboard::Key::Named(Named::ArrowLeft) => SAkt::Cursor(-1, 0), keyboard::Key::Named(Named::ArrowRight) => SAkt::Cursor(1, 0), keyboard::Key::Named(Named::ArrowUp) => SAkt::Cursor(0, -1), keyboard::Key::Named(Named::ArrowDown) => SAkt::Cursor(0, 1),
            keyboard::Key::Named(Named::Escape) if self.screens.get(rel).is_some_and(|s| s.doc.von.is_some()) => { if let Some(s) = self.screens.get_mut(rel) { s.doc.von = None } return true }
            keyboard::Key::Named(Named::Escape) if text_modus => SAkt::Werkzeug(Werkzeug::Zeichen),
            keyboard::Key::Character(c) if text_modus => { if let Some(ch) = c.chars().next() { if let Some(s) = self.screens.get_mut(rel) { s.doc.tippe(ch) } } return true }
            keyboard::Key::Named(Named::Space) if text_modus => { if let Some(s) = self.screens.get_mut(rel) { s.doc.tippe(' ') } return true }
            keyboard::Key::Character(c) => match c.as_str().to_lowercase().as_str() {
                "d" => SAkt::Werkzeug(Werkzeug::Zeichen), "t" => SAkt::Werkzeug(Werkzeug::Text), "f" => SAkt::Werkzeug(Werkzeug::Fuellen), "r" => SAkt::Werkzeug(Werkzeug::Rahmen), "i" => SAkt::Werkzeug(Werkzeug::Pipette), "e" => SAkt::Werkzeug(Werkzeug::Loeschen),
                "+" | "=" => SAkt::Zoom(1), "-" => SAkt::Zoom(-1), "0" => SAkt::Einpassen, _ => return false },
            _ => return false,
        };
        self.screen_aktion(rel, a); true
    }

    pub(crate) fn screen_a11y(&self) -> Option<(String, String)> {
        let n = self.name()?; let s = self.screens.get(&n)?; let d = &s.doc;
        Some((vc_i18n::t!("screenstudio-bildschirm-n", n = &n), vc_i18n::t!("screenstudio-beschreibung", w = d.w, h = d.h, werkzeug = d.werkzeug.label(), zeichen = vc_ide_model::glyphs::code_zeichen(&d.target, d.zeichen), code = d.zeichen, spalte = d.cursor.0, zeile = d.cursor.1, satz = d.zeichensatz().map(|s| s.to_string()).unwrap_or_else(|| vc_i18n::t!("screenstudio-keiner")))))
    }

    pub(crate) fn screen_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.screen_ansicht_k(pal, &rel, muster::klasse(size.width), size.width)).into()
    }
    fn screen_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse, breite: f32) -> Element<'_, Msg> {
        let Some(s) = self.screens.get(rel) else { return text("").into() }; let d = &s.doc; let r = rel.to_string();
        let sehr_schmal = klasse == muster::Klasse::Schmal && breite < 480.0;
        let norm = self.norm_fuer(&d.target, s.ansicht); let farben: &Farben = &self.farben;
        let rgb = |i: usize| farben.rgb(&d.target, norm, i);
        let tb = |name: &str, tipp: &str, an: bool, a: SAkt, aktiv: bool| -> Element<'static, Msg> { muster::knopf(pal, name, tipp, an.then(|| Msg::ScreenAkt(r.clone(), a)), aktiv, false) };
        let w = |n: &str, t: &str, k: Werkzeug| tb(n, t, true, SAkt::Werkzeug(k), d.werkzeug == k);
        // Zone 2: genau eine Leistenreihe. Werkzeuge · Bearbeiten · Ansicht; bei sehr wenig Breite stehen die letzten Werkzeuge im ⋯-Menü
        let werkzeuge_a = || iced::widget::row![w(&vc_i18n::t!("screenstudio-zeichen"), &vc_i18n::t!("screenstudio-zeichen-setzen-d-rechte-taste-loescht"), Werkzeug::Zeichen), w(&vc_i18n::t!("screenstudio-text"), &vc_i18n::t!("screenstudio-text-tippen-t-pfeile-bewegen-den"), Werkzeug::Text), w(&vc_i18n::t!("screenstudio-fuellen"), &vc_i18n::t!("screenstudio-gleiche-zeichen-fuellen-f"), Werkzeug::Fuellen)].spacing(4).align_y(Alignment::Center);
        let werkzeuge_b = || iced::widget::row![w(&vc_i18n::t!("screenstudio-rahmen"), &vc_i18n::t!("screenstudio-rahmen-aufziehen-r-klick-ziehen-loslassen"), Werkzeug::Rahmen), w(&vc_i18n::t!("screenstudio-pipette"), &vc_i18n::t!("screenstudio-zeichen-und-farbe-aufnehmen-i"), Werkzeug::Pipette), w(&vc_i18n::t!("screenstudio-loeschen"), &vc_i18n::t!("screenstudio-zeichen-loeschen-e"), Werkzeug::Loeschen)].spacing(4).align_y(Alignment::Center);
        let bearbeiten = iced::widget::row![tb("↶", &vc_i18n::t!("screenstudio-rueckgaengig-cmd-z"), d.kann_undo(), SAkt::Undo, false), tb("↷", &vc_i18n::t!("screenstudio-wiederholen-cmd-umschalt-z"), d.kann_redo(), SAkt::Redo, false)].spacing(4);
        let zoomgruppe = iced::widget::row![tb("−", &vc_i18n::t!("screenstudio-verkleinern-cmd-mausrad"), true, SAkt::Zoom(-1), false), tb("+", &vc_i18n::t!("screenstudio-vergroessern-cmd-mausrad"), true, SAkt::Zoom(1), false), tb(&vc_i18n::t!("screenstudio-einpassen"), &vc_i18n::t!("screenstudio-flaeche-einpassen-0"), true, SAkt::Einpassen, s.zoomansicht.zoom.is_none())].spacing(4).align_y(Alignment::Center);
        let mehr_knopf = tb("⋯", &vc_i18n::t!("screenstudio-weitere-werkzeuge-und-ansicht-alle-aktionen"), true, SAkt::Mehr, s.mehr);
        let tools: Element<Msg> = if sehr_schmal { werkzeuge_a().into() } else { iced::widget::row![werkzeuge_a(), werkzeuge_b()].spacing(4).into() };
        let leiste_inhalt: Element<Msg> = if klasse == muster::Klasse::Schmal { iced::widget::row![tools, muster::trenner(pal), bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into() }
            else { iced::widget::row![tools, muster::trenner(pal), bearbeiten, space::horizontal(), zoomgruppe].spacing(8).align_y(Alignment::Center).into() };
        let leiste = muster::leiste(pal, leiste_inhalt);
        // Fläche: das Bild wird in der eingepassten (oder von Hand gewählten) ganzzahligen Stufe neu erzeugt; eigene Bildebene unter der Zeichenebene
        let (w1, h1) = einheiten(d); let skala_zuletzt = &s.skala; let rel_c = rel.to_string();
        let flaeche_roh: Element<Msg> = iced::widget::responsive(move |size| {
            let z = s.zoomansicht.zoom.map(|z| z.round()).unwrap_or_else(|| vc_gui_ui::flaeche::Ansicht::einpass(size, w1, h1)).clamp(1.0, SKALA_MAX as f32) as usize;
            skala_zuletzt.set(z); s.zoomansicht.platz.set((size.width, size.height));
            let bild = d.bild(z, &rgb); let (bw, bh) = (bild.w as f32, bild.h as f32);
            iced::widget::stack![canvas::Canvas::new(BildEbene { s, bild: bild_handle(&bild), bw, bh }).width(Length::Fill).height(Length::Fill), canvas::Canvas::new(Flaeche { s, skala: z, bw, bh, pal, rel: rel_c.clone() }).width(Length::Fill).height(Length::Fill)].into()
        }).into();
        let fg_zelle = if d.farb_ram { d.farbe } else { d.farbe_von("fg", 14) }; let bg = d.farbe_von("bg", 6);
        let tafel_bild = { let mut b = Bild::neu(16 * 16, 16 * 16, rgb(bg)); if let Some(g) = &d.glyphs { for c in 0..=255u8 { g.zeichne(&mut b, (c as usize % 16) * 16, (c as usize / 16) * 16, 2, &d.target, c, fg_zelle, d.mc(), false, &rgb); } } b };
        let tafel = canvas::Canvas::new(Tafel { bild: bild_handle(&tafel_bild), kante: 16.0, gewaehlt: d.zeichen, pal, rel: rel.to_string() }).width(256).height(256);
        let farbfeld = |wahl: usize, mach: Box<dyn Fn(usize) -> SAkt>| -> Element<'static, Msg> {
            let eintr = farben.eintraege(&d.target, norm); let mut raster = iced::widget::column![].spacing(2); let mut zeile = iced::widget::row![].spacing(2);
            for (k, (idx, c)) in eintr.iter().enumerate() {
                let (idx, c, gew) = (*idx as usize, *c, *idx as usize == wahl); let r2 = r.clone(); let a = mach(idx);
                zeile = zeile.push(button(space::vertical()).width(16).height(16).padding(0).on_press(Msg::ScreenAkt(r2, a)).style(move |_, _| button::Style { background: Some(c.into()), border: Border { color: if gew { pal.accent } else { pal.line }, width: if gew { 2.0 } else { 1.0 }, radius: 3.0.into() }, ..Default::default() }));
                if (k + 1) % 8 == 0 { raster = raster.push(zeile); zeile = iced::widget::row![].spacing(2); }
            }
            raster.push(zeile).into()
        };
        let kopf = |t: &str| text(t.to_string()).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute);
        // Karte Werkzeug: Zeichen und Zeichenfarbe für das, was als Nächstes gesetzt wird (nicht in der Datei)
        let mut werkzeug = iced::widget::column![kopf(&vc_i18n::t!("screenstudio-zeichen-p1-p2", p1 = &d.zeichen, p2 = format!("{:02X}", d.zeichen))), tafel].spacing(muster::abstand::GRUPPE);
        if d.farb_ram { werkzeug = werkzeug.push(kopf(&vc_i18n::t!("screenstudio-farbe-fuer-neue-zeichen"))).push(farbfeld(d.farbe, Box::new(SAkt::Farbe))).push(text(vc_i18n::t!("screenstudio-gilt-fuer-zeichen-die-du-ab")).size(muster::schrift::HINWEIS).color(pal.mute)) }
        // Karte Dokument: Zeichensatz und Farben, die zum Bildschirm gehören (werden gespeichert)
        let mut dokument = iced::widget::column![kopf(&vc_i18n::t!("screenstudio-zeichensatz"))].spacing(muster::abstand::GRUPPE);
        for z in std::iter::once(None).chain(d.zeichensaetze.iter().cloned().map(Some)) {
            let on = d.zeichensatz() == z.as_deref(); let name = z.clone().unwrap_or_else(|| format!("({})", vc_i18n::t!("screenstudio-keiner")));
            dokument = dokument.push(button(text(format!("{} {}", if on { "●" } else { "○" }, name.rsplit('/').next().unwrap_or(&name))).size(12).color(pal.text)).padding([2, 6]).on_press(Msg::ScreenAkt(r.clone(), SAkt::Zeichensatz(z))).style(move |_, _| button::Style { background: if on { Some(pal.sel.into()) } else { None }, text_color: pal.text, ..Default::default() }));
        }
        dokument = dokument.push(kopf(&vc_i18n::t!("screenstudio-hintergrund"))).push(farbfeld(bg, Box::new(|i| SAkt::Rolle("bg", i))));
        if matches!(d.target.as_str(), "c64" | "vic20") { dokument = dokument.push(kopf(&vc_i18n::t!("screenstudio-rahmen"))).push(farbfeld(d.farbe_von("border", 14), Box::new(|i| SAkt::Rolle("border", i)))) }
        else if !d.farb_ram { dokument = dokument.push(kopf(&vc_i18n::t!("screenstudio-vordergrund-helligkeit"))).push(farbfeld(d.farbe_von("fg", 14), Box::new(|i| SAkt::Rolle("fg", i)))) }
        dokument = dokument.push(text(self.palettenzeile(&d.target, s.ansicht)).size(10.5).color(pal.mute));
        if self.farben.hat_varianten(&d.target) { let r3 = r.clone(); dokument = dokument.push(self.ansicht_wahl_mit(pal, s.ansicht, move |a| Msg::ScreenAkt(r3.clone(), SAkt::Ansicht(a)))) }
        let (cx, cy) = d.cursor;
        let info = iced::widget::column![text(vc_i18n::t!("screenstudio-p1-p2-zeichen", p1 = &d.w, p2 = &d.h)).size(muster::schrift::TEXT).color(pal.text), text(vc_i18n::t!("screenstudio-cursor-spalte-cx-zeile-cy-code", cx = &cx, cy = &cy, p3 = d.code(cx, cy))).size(muster::schrift::HINWEIS).color(pal.mute)].spacing(muster::abstand::ENG);
        let karte_zu = |k: &'static str| -> Option<(bool, Msg)> { Some((!s.karten_zu.contains(k), Msg::ScreenAkt(r.clone(), SAkt::Karte(k)))) };
        let karten: Vec<Element<Msg>> = vec![
            muster::karte(pal, &vc_i18n::t!("screenstudio-werkzeug"), "✎", &vc_i18n::t!("screenstudio-fuers-zeichnen-nicht-in-der-datei"), karte_zu("werkzeug"), werkzeug.into()),
            muster::karte(pal, &vc_i18n::t!("screenstudio-dokument"), "▤", &vc_i18n::t!("screenstudio-gilt-fuer-den-ganzen-bildschirm-wird"), karte_zu("dokument"), dokument.into()),
            muster::karte(pal, &vc_i18n::t!("screenstudio-info"), "ⓘ", "", karte_zu("info"), info.into())];
        let seite = muster::seitenspalte(klasse, karten);
        // Zone 1: Kopfzeile (alle Datei-Aktionen nur hier; Zerlegen öffnet einen Dialog mit Vorschau)
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let kopf_z = muster::kopfzeile(pal, klasse, "SCR", &name, d.dirty(), &vc_i18n::t!("screenstudio-p1-p2-zeichen", p1 = &d.w, p2 = &d.h), vec![
            tb(&vc_i18n::t!("screenstudio-zerlegen"), &vc_i18n::t!("screenstudio-in-kacheln-zerlegen-vorschau-kachelsatz-und"), true, SAkt::ZerlegenDialog(Some((2, 2))), s.zerlegen.is_some()),
            tb(&vc_i18n::t!("screenstudio-exportieren"), &vc_i18n::t!("screenstudio-export-dialog-cmd-umschalt-e"), true, SAkt::Exportieren, false), tb(&vc_i18n::t!("screenstudio-speichern"), &vc_i18n::t!("screenstudio-speichern-cmd-s"), d.dirty(), SAkt::Speichern, false)]);
        let status = if d.status.is_empty() { String::new() } else { format!(" · {}", d.status) };
        let (zw, zh) = (w1, h1);
        let fuss = muster::fusszeile(pal, &vc_i18n::t!("screenstudio-spalte-cx-zeile-cy-code-p3", cx = &cx, cy = &cy, p3 = d.code(cx, cy), p4 = if s.zoomansicht.platz.get().0 < 1.0 { vc_i18n::t!("screenstudio-einpassen").to_string() } else { format!("{}{}×", if s.zoomansicht.zoom.is_none() { format!("{} ", vc_i18n::t!("screenstudio-einpassen-2")) } else { String::new() }, s.skala.get()) }, status = &status),
            iced::widget::tooltip(text(vc_i18n::t!("screenstudio-tasten")).size(muster::schrift::HINWEIS).color(pal.mute), container(text(vc_i18n::t!("screenstudio-linke-taste-setzt-rechte-loescht-d")).size(11.5).color(pal.text)).padding([4, 8]).max_width(420).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Top).into());
        let _ = (zw, zh);
        // Überlagerungen über der Fläche: ⋯-Karte (schmal) und Zerlegen-Dialog
        let mut flaeche_el = flaeche_roh;
        if s.mehr {
            let karte_mehr = container(iced::widget::column![if sehr_schmal { Element::from(iced::widget::column![text(vc_i18n::t!("screenstudio-werkzeuge")).size(muster::schrift::HINWEIS).color(pal.mute), werkzeuge_b()].spacing(muster::abstand::ENG)) } else { space::vertical().height(0).into() }, text(vc_i18n::t!("screenstudio-ansicht")).size(muster::schrift::HINWEIS).color(pal.mute), zoomgruppe_klein(&tb)].spacing(muster::abstand::GRUPPE)).padding(muster::abstand::KARTE).max_width(340).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() });
            flaeche_el = iced::widget::stack![flaeche_el, container(karte_mehr).padding(muster::abstand::GRUPPE).align_right(Length::Fill)].into();
        }
        if let Some((tw, th)) = s.zerlegen {
            let vorschau = self.zerlegen_vorschau(rel, tw, th);
            let groesse = |a: usize, b: usize| -> Element<'static, Msg> { let an = (a, b) == (tw, th); button(text(vc_i18n::t!("screenstudio-a-b", a = &a, b = &b)).size(12.5).color(if an { pal.accent_text() } else { pal.text })).padding([4, 12]).on_press(Msg::ScreenAkt(r.clone(), SAkt::ZerlegenDialog(Some((a, b))))).style(move |_, _| button::Style { background: Some(if an { pal.accent.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() };
            let dlg = container(iced::widget::column![text(vc_i18n::t!("screenstudio-in-kacheln-zerlegen")).size(muster::schrift::ABSCHNITT).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(vc_i18n::t!("screenstudio-gleiche-bloecke-werden-zu-kacheln-kachelsatz")).size(muster::schrift::HINWEIS).color(pal.mute),
                text(vc_i18n::t!("screenstudio-kachelgroesse")).size(muster::schrift::HINWEIS).color(pal.mute), iced::widget::row![groesse(2, 2), groesse(3, 3), groesse(4, 4)].spacing(4),
                text(vorschau.clone().unwrap_or_else(|e| e)).size(muster::schrift::TEXT).color(if vorschau.is_ok() { pal.ok } else { pal.err }),
                iced::widget::row![tb(&vc_i18n::t!("screenstudio-abbrechen"), &vc_i18n::t!("screenstudio-dialog-schliessen"), true, SAkt::ZerlegenDialog(None), false), tb(&vc_i18n::t!("screenstudio-zerlegen-2"), &vc_i18n::t!("screenstudio-kachelsatz-und-bank-anlegen"), vorschau.is_ok(), SAkt::Zerlegen(tw, th), false)].spacing(8)].spacing(muster::abstand::GRUPPE))
                .padding(muster::abstand::KARTE).max_width(380).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.accent, width: 2.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() });
            flaeche_el = iced::widget::stack![flaeche_el, container(dlg).padding(muster::abstand::BEREICH).center(Length::Fill)].into();
        }
        let flaeche_el: Element<Msg> = container(flaeche_el).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: muster::LISTE_BREITE, kopf: kopf_z, leiste, liste: None, flaeche: flaeche_el, seite, fuss })
    }
    /// Vorschau des Zerlegens ohne zu schreiben: „n Kacheln, m Zellen-Patches, x → y Byte“
    fn zerlegen_vorschau(&self, rel: &str, tw: usize, th: usize) -> Result<String, String> {
        let s = self.screens.get(rel).ok_or(vc_i18n::t!("screenstudio-kein-bildschirm"))?;
        let profiles = vc_media::load_profiles(&self.data)?; let p = vc_media::profile(&profiles, s.doc.media["profile"].as_str().unwrap_or(""))?.clone();
        let satzprofil = profiles.iter().find(|x| x.kind == "tileset" && x.target == p.target).map(|x| x.id.clone()).ok_or_else(|| vc_i18n::t!("screenstudio-fuer-p1-gibt-es-kein-kachelsatz", p1 = &p.target))?;
        let d = vc_ide_model::bankdoc::decompose(&s.doc.media, &p, &satzprofil, tw, th, (0, 0), vc_media::naming::stem(rel))?;
        Ok(vc_i18n::t!("screenstudio-p1-kacheln-p2-zellen-patches-p3", p1 = &d.tiles, p2 = &d.patches, p3 = &d.bytes_before, p4 = &d.bytes_after))
    }
}

/// Ansichtsgruppe der ⋯-Karte (Zoom, Einpassen)
fn zoomgruppe_klein<'a>(tb: &dyn Fn(&str, &str, bool, SAkt, bool) -> Element<'a, Msg>) -> Element<'a, Msg> {
    iced::widget::row![tb("−", &vc_i18n::t!("screenstudio-verkleinern"), true, SAkt::Zoom(-1), false), tb("+", &vc_i18n::t!("screenstudio-vergroessern"), true, SAkt::Zoom(1), false), tb(&vc_i18n::t!("screenstudio-einpassen"), &vc_i18n::t!("screenstudio-flaeche-einpassen-0"), true, SAkt::Einpassen, false)].spacing(4).into()
}
