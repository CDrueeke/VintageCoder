// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachelsatz-Studio (`.vctiles`): links die Kacheln des Satzes mit Verwendung, in der Mitte die gewählte Kachel zum Bearbeiten (Zellen aus dem Zeichensatz),
//! rechts Name, Farbe, Eigenschaften, Zeichensätze, Zeichentafel und Füllstand des Zielzeichensatzes. Modell: `vc_ide_model::tilesetdoc::TilesetDoc`.
use super::*;
use crate::studio::{Ansicht, Farben};
use vc_gui_ui::muster;
use iced::widget::{canvas, text, tooltip};
use vc_ide_model::glyphs::Bild;
use vc_ide_model::tilesetdoc::{Frage, TilesetDoc};
use vc_media::tiles::ColorMode;

pub struct TileStudio { pub doc: TilesetDoc, pub zoomansicht: vc_gui_ui::flaeche::Ansicht, pub skala: std::cell::Cell<usize>, pub ansicht: Ansicht, pub neue_prop: String, pub unten: bool, pub ueber: Option<usize>, pub mehr: bool, pub karten_zu: std::collections::BTreeSet<&'static str> }

#[derive(Clone, Debug, PartialEq)]
pub enum TAkt {
    Undo, Redo, Speichern, Exportieren, Modus(vc_ide_model::tilesetdoc::PinselModus), Kachel(usize), Neu, Dupl, Loeschen, Zusammen, Name(String), NameFertig, Farbmodus(ColorMode), Kachelfarbe(usize), Pinselfarbe(usize), Bg(usize), Prop(u8, bool), PropText(String), PropNeu,
    SpiegelH, SpiegelV, Drehen, Rollen(i32, i32), Leeren, Kopieren, Einfuegen, Groesse(usize, usize), Zeichen(u8), Reiter(usize), ZeichensatzPlus(String), ZeichensatzMinus, Bit7(bool), FrageJa, FrageNein, Zoom(i32), Einpassen, Schieben(f32, f32), Mehr, Karte(&'static str), Ansicht(Ansicht), Cursor(i32, i32),
}
#[derive(Clone, Debug, PartialEq)]
pub enum TMaus { Unten { k: usize, rechts: bool }, Zug { k: usize }, Los, Ueber(Option<usize>) }

fn handle(b: &Bild) -> iced::widget::image::Handle { crate::bildcache::handle(b.w as u32, b.h as u32, &b.rgba) }

/// Größtes Zoom der Kachel (Bild je Zeichenpixel)
const KANTE_MAX: usize = 24;
/// Einheiten der Fläche bei Zoom 1: Breite und Höhe der Kachel in Bildpunkten (8 je Zelle)
fn einheiten(ts: &vc_media::tiles::TileSet) -> (f32, f32) { ((ts.tw * 8) as f32, (ts.th * 8) as f32) }
fn lage(b: iced::Size, bw: f32, bh: f32, a: &vc_gui_ui::flaeche::Ansicht) -> (f32, f32) {
    a.ecke(b, bw, bh)
}

/// Bildebene: die gewählte Kachel groß (eigene Ebene unter Raster und Cursor)
struct BildEbene<'a> { s: &'a TileStudio, bild: iced::widget::image::Handle, bw: f32, bh: f32 }
impl<'a> canvas::Program<Msg> for BildEbene<'a> {
    type State = ();
    fn update(&self, _st: &mut (), _e: &iced::Event, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> { None }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let (ox, oy) = lage(b.size(), self.bw, self.bh, &self.s.zoomansicht);
        f.draw_image(iced::Rectangle::new(iced::Point::new(ox, oy), iced::Size::new(self.bw, self.bh)), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        vec![f.into_geometry()]
    }
}

/// Zeichenfläche (obere Ebene): Zellen anklicken/ziehen setzt den Pinsel, rechte Taste nimmt die Zelle auf; Raster, gewählte Zelle, aktive Zelle als Doppelrahmen
struct Flaeche<'a> { s: &'a TileStudio, tw: usize, th: usize, kante: f32, bw: f32, bh: f32, cur: usize, pal: Pal, rel: String }
#[derive(Default)] struct FlaechenZustand { unten: bool, ueber: Option<usize>, gesten: vc_gui_ui::flaeche::Gesten }
impl<'a> Flaeche<'a> {
    fn ecke(&self, b: iced::Size) -> (f32, f32) { lage(b, self.bw, self.bh, &self.s.zoomansicht) }
    fn zelle(&self, b: iced::Rectangle, p: iced::Point) -> Option<usize> { let (ox, oy) = self.ecke(b.size()); let (fx, fy) = ((p.x - ox) / self.kante, (p.y - oy) / self.kante); (fx >= 0.0 && fy >= 0.0 && (fx as usize) < self.tw && (fy as usize) < self.th).then(|| fy as usize * self.tw + fx as usize) }
}
impl<'a> canvas::Program<Msg> for Flaeche<'a> {
    type State = FlaechenZustand;
    fn update(&self, st: &mut FlaechenZustand, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        use iced::mouse::{Button, Event as M};
        // Leertaste + Ziehen verschiebt, Cmd/Strg + Mausrad zoomt (Tasten bleiben bei der Befehlstabelle)
        if let Some(g) = st.gesten.ereignis(e, b, c) {
            use vc_gui_ui::flaeche::Geste;
            return match g { Geste::Erfasst => Some(canvas::Action::capture()), Geste::Schieben(dx, dy) => Some(canvas::Action::publish(Msg::TileAkt(self.rel.clone(), TAkt::Schieben(dx, dy))).and_capture()), Geste::Zoom(plus) => Some(canvas::Action::publish(Msg::TileAkt(self.rel.clone(), TAkt::Zoom(if plus { 1 } else { -1 }))).and_capture()) }
        }
        let rel = self.rel.clone(); let send = |m: TMaus| Some(canvas::Action::publish(Msg::TileMaus(rel.clone(), m)).and_capture());
        match e {
            iced::Event::Mouse(M::ButtonPressed(k @ (Button::Left | Button::Right))) => { let k2 = self.zelle(b, c.position_in(b)?)?; st.unten = true; send(TMaus::Unten { k: k2, rechts: *k == Button::Right }) }
            iced::Event::Mouse(M::CursorMoved { .. }) if st.unten => { let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let k = self.zelle(b, p)?; st.ueber = Some(k); send(TMaus::Zug { k }) }
            iced::Event::Mouse(M::CursorMoved { .. }) => { let z = c.position_in(b).and_then(|p| self.zelle(b, p)); if z != st.ueber { st.ueber = z; Some(canvas::Action::publish(Msg::TileMaus(rel, TMaus::Ueber(z)))) } else { None } }
            iced::Event::Mouse(M::ButtonReleased(Button::Left | Button::Right)) if st.unten => { st.unten = false; send(TMaus::Los) }
            _ => None,
        }
    }
    fn draw(&self, st: &FlaechenZustand, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        self.s.zoomansicht.platz.set((b.width, b.height));
        let mut f = canvas::Frame::new(r, b.size()); let (ox, oy) = self.ecke(b.size()); let pal = self.pal; let k = self.kante;
        let linie = canvas::Stroke::default().with_color(Color { a: 0.35, ..pal.text }).with_width(1.0);
        for x in 0..=self.tw { let px = ox + x as f32 * k; f.stroke(&canvas::Path::line(iced::Point::new(px, oy), iced::Point::new(px, oy + self.th as f32 * k)), linie) }
        for y in 0..=self.th { let py = oy + y as f32 * k; f.stroke(&canvas::Path::line(iced::Point::new(ox, py), iced::Point::new(ox + self.tw as f32 * k, py)), linie) }
        // gewählte Zelle (kräftiger Rahmen) und aktive Zelle unter dem Zeiger (Doppelrahmen mit Achsenmarken)
        let doppel = |f: &mut canvas::Frame, i: usize, innen: Color| { let (cx, cy) = ((i % self.tw) as f32, (i / self.tw) as f32); let p = iced::Point::new(ox + cx * k, oy + cy * k);
            f.stroke_rectangle(iced::Point::new(p.x - 1.0, p.y - 1.0), iced::Size::new(k + 2.0, k + 2.0), canvas::Stroke::default().with_color(innen).with_width(2.0));
            f.stroke_rectangle(iced::Point::new(p.x - 3.5, p.y - 3.5), iced::Size::new(k + 7.0, k + 7.0), canvas::Stroke { line_dash: canvas::LineDash { segments: &[4.0, 3.0], offset: 0 }, ..canvas::Stroke::default().with_color(pal.text).with_width(1.5) });
            f.fill_rectangle(iced::Point::new(p.x, oy - 8.0), iced::Size::new(k, 4.0), innen); f.fill_rectangle(iced::Point::new(ox - 8.0, p.y), iced::Size::new(4.0, k), innen) };
        if let Some(i) = st.ueber.filter(|i| *i < self.tw * self.th) { doppel(&mut f, i, pal.accent) }
        doppel(&mut f, self.cur.min(self.tw * self.th - 1), pal.warn);
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &FlaechenZustand, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Crosshair }
}

/// Zeichentafel des gewählten Zeichensatzes
struct Tafel { bild: iced::widget::image::Handle, kante: f32, zeilen: usize, gewaehlt: Option<u8>, benutzt: Vec<bool>, pal: Pal, rel: String }
impl canvas::Program<Msg> for Tafel {
    type State = ();
    fn update(&self, _st: &mut (), e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = e {
            let p = c.position_in(b)?; let (x, y) = ((p.x / self.kante) as usize, (p.y / self.kante) as usize);
            if x < 16 && y < self.zeilen { return Some(canvas::Action::publish(Msg::TileAkt(self.rel.clone(), TAkt::Zeichen((y * 16 + x) as u8))).and_capture()) }
        }
        None
    }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size());
        f.draw_image(iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(16.0 * self.kante, self.zeilen as f32 * self.kante)), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        for (i, u) in self.benutzt.iter().enumerate() { if *u { f.fill_rectangle(iced::Point::new((i % 16) as f32 * self.kante + self.kante - 4.0, (i / 16) as f32 * self.kante + 1.0), iced::Size::new(3.0, 3.0), self.pal.accent) } }
        if let Some(g) = self.gewaehlt { f.stroke_rectangle(iced::Point::new((g % 16) as f32 * self.kante, (g / 16) as f32 * self.kante), iced::Size::new(self.kante, self.kante), canvas::Stroke::default().with_color(self.pal.accent).with_width(2.0)) }
        vec![f.into_geometry()]
    }
}

impl App {
    pub(crate) fn tile_oeffnen(&mut self, rel: &str) {
        if !self.tilesets.contains_key(rel) {
            match TilesetDoc::oeffnen(&self.root, rel, &self.data) { Ok(doc) => { self.tilesets.insert(rel.to_string(), TileStudio { doc, zoomansicht: vc_gui_ui::flaeche::Ansicht::mit_max(KANTE_MAX as f32), skala: std::cell::Cell::new(8), ansicht: Ansicht::Projekt, neue_prop: String::new(), unten: false, ueber: None, mehr: false, karten_zu: ["dokument", "info"].into_iter().collect() }); } Err(e) => { self.log.push(e); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }

    pub(crate) fn tile_maus(&mut self, rel: &str, m: TMaus) {
        let Some(s) = self.tilesets.get_mut(rel) else { return };
        match m {
            TMaus::Unten { k, rechts } => { s.unten = true; if rechts { s.doc.aufnehmen(k) } else { s.doc.malen(k, true) } }
            TMaus::Zug { k } => { if s.unten { s.doc.malen(k, false) } }
            TMaus::Los => s.unten = false,
            TMaus::Ueber(z) => s.ueber = z,
        }
    }

    pub(crate) fn tile_aktion(&mut self, rel: &str, a: TAkt) {
        if a == TAkt::Exportieren { self.export_oeffnen(); return }
        let Some(s) = self.tilesets.get_mut(rel) else { return }; let d = &mut s.doc;
        match a {
            TAkt::Undo => d.rueckgaengig(), TAkt::Redo => d.wiederholen(),
            TAkt::Speichern => match d.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {rel}")), Err(e) => self.log.push(e) },
            TAkt::Kachel(i) => d.kachel_waehlen(i), TAkt::Neu => d.kachel_neu(), TAkt::Dupl => d.kachel_duplizieren(), TAkt::Loeschen => d.kachel_loeschen(),
            TAkt::Zusammen => { if let Err(e) = d.zusammenfuehren() { self.log.push(e) } else if !d.status.is_empty() { self.log.push(d.status.clone()) } if !self.tilesets.is_empty() { self.projekt_neu_laden() } }
            TAkt::Name(n) => d.kachel_umbenennen(&n, false), TAkt::NameFertig => {}
            TAkt::Modus(m) => d.modus_waehlen(m), TAkt::Farbmodus(m) => d.farbmodus(m), TAkt::Kachelfarbe(i) => d.kachelfarbe(i), TAkt::Pinselfarbe(i) => d.pinsel_farbe(i), TAkt::Bg(i) => d.hintergrund(i),
            TAkt::Prop(b, an) => d.eigenschaft_setzen(b, an), TAkt::PropText(t) => s.neue_prop = t, TAkt::PropNeu => { let n = s.neue_prop.clone(); if d.eigenschaft_neu(&n) { s.neue_prop.clear() } }
            TAkt::SpiegelH => d.spiegeln(true), TAkt::SpiegelV => d.spiegeln(false), TAkt::Drehen => d.drehen(), TAkt::Rollen(x, y) => d.rollen(x, y), TAkt::Leeren => d.leeren(), TAkt::Kopieren => d.kopieren(), TAkt::Einfuegen => d.einfuegen(),
            TAkt::Groesse(w, h) => d.groesse_anfragen(w, h), TAkt::Zeichen(c) => d.zeichen_waehlen(c), TAkt::Reiter(i) => d.reiter_waehlen(i), TAkt::ZeichensatzPlus(p) => d.zeichensatz_hinzu(&p), TAkt::ZeichensatzMinus => { let i = d.reiter; d.zeichensatz_entfernen(i) }
            TAkt::Bit7(b) => d.farbwahl_bit(b),
            TAkt::FrageJa => { if let Err(e) = d.frage_bestaetigen() { self.log.push(e) } else if !d.status.is_empty() { self.log.push(d.status.clone()) } self.projekt_neu_laden() }
            TAkt::FrageNein => d.frage_abbrechen(),
            TAkt::Zoom(z) => { let (w, h) = einheiten(&d.ts); s.zoomansicht.stufe(z > 0, w, h) } TAkt::Einpassen => s.zoomansicht.einpassen(), TAkt::Schieben(dx, dy) => { let (w, h) = einheiten(&d.ts); s.zoomansicht.schieben(dx, dy, w, h) }
            TAkt::Mehr => s.mehr = !s.mehr, TAkt::Karte(k) => { if !s.karten_zu.remove(k) { s.karten_zu.insert(k); } }
            TAkt::Ansicht(x) => s.ansicht = x, TAkt::Cursor(dx, dy) => d.cursor_bewegen(dx, dy),
            TAkt::Exportieren => {}
        }
    }

    /// Einzeltasten: Pfeile wandern in der Kachel
    pub(crate) fn tile_taste(&mut self, rel: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
        if m.command() || m.alt() || m.control() || !self.tilesets.contains_key(rel) { return false }
        use keyboard::key::Named;
        let a = match k { keyboard::Key::Named(Named::ArrowLeft) => TAkt::Cursor(-1, 0), keyboard::Key::Named(Named::ArrowRight) => TAkt::Cursor(1, 0), keyboard::Key::Named(Named::ArrowUp) => TAkt::Cursor(0, -1), keyboard::Key::Named(Named::ArrowDown) => TAkt::Cursor(0, 1),
            keyboard::Key::Character(c) => match c.as_str() { "+" | "=" => TAkt::Zoom(1), "-" => TAkt::Zoom(-1), "0" => TAkt::Einpassen, _ => return false }, _ => return false };
        self.tile_aktion(rel, a); true
    }

    pub(crate) fn tile_a11y(&self) -> Option<(String, String)> {
        let n = self.name()?; let s = self.tilesets.get(&n)?; let d = &s.doc;
        Some((format!("Kachelsatz {n}"), format!("{} Kacheln, {}×{} Zellen, gewählt Kachel {} {}, Zelle {}", d.ts.tiles.len(), d.ts.tw, d.ts.th, d.sel, d.ts.tiles.get(d.sel).map(|t| t.name.clone()).unwrap_or_default(), d.cur)))
    }

    pub(crate) fn tile_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.tile_ansicht_k(pal, &rel, muster::klasse(size.width), size.width)).into()
    }
    fn tile_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse, breite: f32) -> Element<'_, Msg> {
        let Some(s) = self.tilesets.get(rel) else { return text("").into() }; let d = &s.doc; let r = rel.to_string(); let ts = &d.ts;
        let sehr_schmal = klasse == muster::Klasse::Schmal && breite < 480.0;
        let norm = self.norm_fuer(&d.target, s.ansicht); let farben: &Farben = &self.farben; let rgb = |i: usize| farben.rgb(&d.target, norm, i);
        let tb = |name: &str, tipp: &str, an: bool, a: TAkt, aktiv: bool| -> Element<Msg> { muster::knopf(pal, name, tipp, an.then(|| Msg::TileAkt(r.clone(), a)), aktiv, false) };
        let (w, h, hat) = (ts.tw, ts.th, !ts.tiles.is_empty());
        // Zone 2: genau eine Leistenreihe: Bearbeiten · Transformieren (Spiegeln, Drehen, Rollen, Leeren, Kopieren, Einfügen) · Ansicht; bei wenig Breite im ⋯-Menü
        let bearbeiten = iced::widget::row![tb("↶", "Rückgängig (Cmd+Z)", d.kann_undo(), TAkt::Undo, false), tb("↷", "Wiederholen (Cmd+Umschalt+Z)", d.kann_redo(), TAkt::Redo, false)].spacing(4);
        let transformieren = || iced::widget::row![tb("⇋", "Zellreihenfolge waagerecht spiegeln", hat && w > 1, TAkt::SpiegelH, false), tb("⇅", "Zellreihenfolge senkrecht spiegeln", hat && h > 1, TAkt::SpiegelV, false), tb("Drehen", "Um 90° drehen (nur quadratische Kacheln)", hat && w == h && w > 1, TAkt::Drehen, false),
            tb("←", "Zellen nach links rollen", hat && w > 1, TAkt::Rollen(-1, 0), false), tb("→", "Zellen nach rechts rollen", hat && w > 1, TAkt::Rollen(1, 0), false), tb("↑", "Zellen nach oben rollen", hat && h > 1, TAkt::Rollen(0, -1), false), tb("↓", "Zellen nach unten rollen", hat && h > 1, TAkt::Rollen(0, 1), false),
            tb("Leeren", "Alle Zellen auf Zeichen 0 setzen", hat, TAkt::Leeren, false), tb("Kopieren", "Zellen der Kachel merken", hat, TAkt::Kopieren, false), tb("Einfügen", "Gemerkte Zellen in diese Kachel einfügen", d.zwischen.as_ref().is_some_and(|c| c.len() == w * h), TAkt::Einfuegen, false)].spacing(4).align_y(Alignment::Center);
        let zoomgruppe = || iced::widget::row![tb("−", "Verkleinern (−, Cmd+Mausrad)", true, TAkt::Zoom(-1), false), tb("+", "Vergrößern (+, Cmd+Mausrad)", true, TAkt::Zoom(1), false), tb("Einpassen", "Fläche einpassen (0)", true, TAkt::Einpassen, s.zoomansicht.zoom.is_none())].spacing(4).align_y(Alignment::Center);
        let mehr_knopf = tb("⋯", "Transformieren und Ansicht · alle Aktionen: Menü-Taste oder Umschalt+F10", true, TAkt::Mehr, s.mehr);
        let leiste_inhalt: Element<Msg> = match klasse {
            muster::Klasse::Breit => iced::widget::row![bearbeiten, muster::trenner(pal), tb("Transformieren ▾", "Spiegeln, Drehen, Rollen, Leeren, Kopieren, Einfügen · alle Aktionen: Menü-Taste oder Umschalt+F10", true, TAkt::Mehr, s.mehr), space::horizontal(), zoomgruppe()].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Mittel => iced::widget::row![bearbeiten, space::horizontal(), zoomgruppe(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Schmal => iced::widget::row![bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
        };
        let leiste = muster::leiste(pal, leiste_inhalt);
        // Kachelliste links
        let mut liste = iced::widget::column![].spacing(3);
        let hat_banken = !d.banken.is_empty();
        for (i, t) in ts.tiles.iter().enumerate() {
            let b = d.kachel_bild(i, 2, &rgb); let n = d.verwendung(i); let gew = i == d.sel;
            let unter = match (hat_banken, d.doppelt[i]) { (_, Some(j)) => format!("gleich wie {j}"), (true, _) => if n == 0 { "unbenutzt".into() } else { format!("{n}× benutzt") }, _ => String::new() };
            let zeile = iced::widget::row![iced::widget::image(handle(&b)).filter_method(iced::widget::image::FilterMethod::Nearest).width(Length::Fixed((w * 16) as f32)).height(Length::Fixed((h * 16) as f32)),
                iced::widget::column![text(if t.name.is_empty() { format!("{i}") } else { format!("{i} {}", t.name) }).size(12.5).color(if hat_banken && n == 0 { pal.mute } else { pal.text }), text(unter).size(10.5).color(if d.doppelt[i].is_some() { pal.warn } else { pal.mute })].spacing(1)].spacing(8).align_y(Alignment::Center);
            liste = liste.push(button(zeile).width(Length::Fill).padding(3).on_press(Msg::TileAkt(r.clone(), TAkt::Kachel(i))).style(move |_, st| button::Style { background: (gew || matches!(st, button::Status::Hovered)).then(|| pal.sel.into()), border: Border { color: if gew { pal.accent } else { Color::TRANSPARENT }, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }));
        }
        let kopf_liste = iced::widget::row![text(format!("Kacheln · {}", ts.tiles.len())).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute), space::horizontal(), tb("+", "Neue leere Kachel am Ende", ts.tiles.len() < vc_media::tiles::MAX_TILES, TAkt::Neu, false)].align_y(Alignment::Center);
        let verw = iced::widget::row![tb("Dupl.", "Gewählte Kachel kopieren", hat && ts.tiles.len() < vc_media::tiles::MAX_TILES, TAkt::Dupl, false), tb("Löschen", "Gewählte Kachel entfernen (benutzte Kacheln fragen nach)", hat, TAkt::Loeschen, false)].spacing(4).wrap().vertical_spacing(4);
        let liste_el: Element<Msg> = iced::widget::column![kopf_liste, verw, if klasse == muster::Klasse::Breit { scrollable(liste).height(Length::Fill).into() } else { Element::from(liste) }].spacing(8).into();
        // Zone 4: die gewählte Kachel groß, eingepasst in ganzen Stufen; Bild und Zeichenebene getrennt
        let (w1, h1) = einheiten(ts); let skala_zuletzt = &s.skala; let rel_c = rel.to_string(); let cur = d.cur; let sel = d.sel;
        let flaeche_roh: Element<Msg> = if hat { iced::widget::responsive(move |size| {
            let z = s.zoomansicht.zoom.map(|z| z.round()).unwrap_or_else(|| vc_gui_ui::flaeche::Ansicht::einpass(size, w1, h1)).clamp(1.0, KANTE_MAX as f32) as usize;
            skala_zuletzt.set(z); s.zoomansicht.platz.set((size.width, size.height));
            let bild = d.kachel_bild(sel, z, &rgb); let (bw, bh) = (bild.w as f32, bild.h as f32);
            iced::widget::stack![canvas::Canvas::new(BildEbene { s, bild: handle(&bild), bw, bh }).width(Length::Fill).height(Length::Fill), canvas::Canvas::new(Flaeche { s, tw: w, th: h, kante: (z * 8) as f32, bw, bh, cur, pal, rel: rel_c.clone() }).width(Length::Fill).height(Length::Fill)].into()
        }).into() } else { container(text("Der Satz hat keine Kachel: „Neu“ legt eine an.").size(13).color(pal.mute)).center(Length::Fill).into() };
        let kopf = |t: &str| text(t.to_string()).size(11).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute);
        let farbfeld = |wahl: usize, mach: Box<dyn Fn(usize) -> TAkt>| -> Element<Msg> {
            let eintr = farben.eintraege(&d.target, norm); let mut raster = iced::widget::column![].spacing(2); let mut zeile = iced::widget::row![].spacing(2);
            for (k, (idx, c)) in eintr.iter().enumerate() { let (idx, c, gew) = (*idx as usize, *c, *idx as usize == wahl); let a = mach(idx);
                zeile = zeile.push(button(space::vertical()).width(16).height(16).padding(0).on_press(Msg::TileAkt(r.clone(), a)).style(move |_, _| button::Style { background: Some(c.into()), border: Border { color: if gew { pal.accent } else { pal.line }, width: if gew { 2.0 } else { 1.0 }, radius: 3.0.into() }, ..Default::default() }));
                if (k + 1) % 8 == 0 { raster = raster.push(zeile); zeile = iced::widget::row![].spacing(2) } }
            raster.push(zeile).into()
        };
        let mut dokument = iced::widget::column![].spacing(6); let mut werkzeug = iced::widget::column![].spacing(6); let mut fuell = iced::widget::column![].spacing(6); let mut eigen = iced::widget::column![].spacing(6);
        // Füllstand des Zielzeichensatzes: ganz oben in der Seitenspalte, damit man ihn sieht
        { let mut f_el = iced::widget::column![kopf("Füllstand des Zeichensatzes")].spacing(4);
          match &d.fill { None => f_el = f_el.push(text("Ohne Zeichensatz gibt es keinen Füllstand.").size(11).color(pal.mute)),
            Some(f) if f.err.is_some() => f_el = f_el.push(text(f.err.clone().unwrap_or_default()).size(11).color(pal.err)),
            Some(f) => { let frac = f.used as f32 / f.max.max(1) as f32; f_el = f_el.push(iced::widget::progress_bar(0.0..=1.0, frac).girth(10)).push(text(format!("{} von {} Zeichen belegt{}", f.used, f.max, if f.doubles > 0 { format!(" · {} gleiche werden beim Export zusammengelegt", f.doubles) } else { String::new() })).size(11.5).color(if frac > 0.9 { pal.warn } else { pal.text })) } }
          fuell = fuell.push(tooltip(f_el, container(text("So viele der 256 Zeichen des Zielzeichensatzes sind belegt").size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom)); }
        if let Some(t) = ts.tiles.get(d.sel) {
            dokument = dokument.push(text(format!("Kachel {}", d.sel)).size(14).font(vc_gui_ui::fonts::UI_FETT).color(pal.text));
            { let r2 = r.clone(); dokument = dokument.push(text_input("Name", &t.name).on_input(move |n| Msg::TileAkt(r2.clone(), TAkt::Name(n))).size(12).padding(4).font(vc_gui_ui::fonts::MONO)) }
            if let Some(j) = d.doppelt.get(d.sel).copied().flatten() { dokument = dokument.push(text(format!("Gleich wie Kachel {j}.")).size(11.5).color(pal.warn)).push(tb(&format!("Mit {j} zusammenführen"), "Diese Kachel entfernen; Verweise in den Screen-Bänken zeigen danach auf die gleiche Kachel", true, TAkt::Zusammen, false)) }
            dokument = dokument.push(kopf("Farbe")).push(iced::widget::row![tb("je Kachel", "Eine Farbe für die ganze Kachel", true, TAkt::Farbmodus(ColorMode::Tile), ts.color_mode == ColorMode::Tile), tb("je Zelle", "Farbe je Zelle", true, TAkt::Farbmodus(ColorMode::Cell), ts.color_mode == ColorMode::Cell), tb("keine", "Keine Farbe im Satz", true, TAkt::Farbmodus(ColorMode::None), ts.color_mode == ColorMode::None)].spacing(4));
            match ts.color_mode {
                ColorMode::Tile => { dokument = dokument.push(text("Kachelfarbe").size(11.5).color(pal.mute)).push(farbfeld(t.col as usize, Box::new(TAkt::Kachelfarbe))) }
                ColorMode::Cell => { use vc_ide_model::tilesetdoc::PinselModus as PM; werkzeug = werkzeug.push(text("Ein Klick in die Kachel setzt:").size(11.5).color(pal.mute)).push(iced::widget::row![tb("Zeichen", "Nur das Zeichen der Zelle; ihre Farbe bleibt", true, TAkt::Modus(PM::Zeichen), d.pinsel_modus == PM::Zeichen), tb("Farbe", "Nur die Farbe der Zelle (Pinselfarbe); das Zeichen bleibt", true, TAkt::Modus(PM::Farbe), d.pinsel_modus == PM::Farbe), tb("beides", "Zeichen und Pinselfarbe", true, TAkt::Modus(PM::Beides), d.pinsel_modus == PM::Beides)].spacing(4).wrap()).push(text("Pinselfarbe (für Zellen)").size(11.5).color(pal.mute)).push(farbfeld(d.farbe, Box::new(TAkt::Pinselfarbe))).push(text("Kachelfarbe (für Zellen ohne eigene Farbe)").size(11.5).color(pal.mute)).push(farbfeld(t.col as usize, Box::new(TAkt::Kachelfarbe))) }
                ColorMode::None => { dokument = dokument.push(text("Keine Farbe im Satz: es gilt die Vordergrundfarbe des Systems.").size(11).color(pal.mute)) }
            }
            if d.atari() { let r2 = r.clone(); dokument = dokument.push(iced::widget::checkbox(d.pinsel.sel).label("Farbwahl (Zeichenbit 7)").on_toggle(move |b| Msg::TileAkt(r2.clone(), TAkt::Bit7(b))).size(14).text_size(12)) }
            eigen = eigen.push(tooltip(kopf("Eigenschaften"), container(text("Eigenschaften: Merkmale wie „solid“, die dein Programm abfragt; Export als _props-Tabelle").size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Bottom)).push(text("Merkmale wie „solid“, die dein Programm abfragt (Export: _props-Tabelle). Haken = gilt für diese Kachel; neuer Name + „+“ legt sie für den ganzen Satz an.").size(10.5).color(pal.mute));
            for (n, bit) in &ts.prop_defs { let (r2, bit) = (r.clone(), *bit); eigen = eigen.push(iced::widget::checkbox(t.props >> bit & 1 == 1).label(n.clone()).on_toggle(move |b| Msg::TileAkt(r2.clone(), TAkt::Prop(bit, b))).size(14).text_size(12)) }
            { let (r2, r3) = (r.clone(), r.clone()); eigen = eigen.push(iced::widget::row![text_input("neue Eigenschaft", &s.neue_prop).on_input(move |t| Msg::TileAkt(r2.clone(), TAkt::PropText(t))).on_submit(Msg::TileAkt(r3.clone(), TAkt::PropNeu)).size(12).padding(3).width(120), tb("+", "Eigenschaft für den ganzen Satz anlegen (höchstens 8)", !s.neue_prop.trim().is_empty() && ts.prop_defs.len() < 8, TAkt::PropNeu, false)].spacing(4).wrap().vertical_spacing(4)) }
        }
        let kachel_teil = std::mem::replace(&mut dokument, iced::widget::column![].spacing(6));
        dokument = dokument.push(kopf("Zeichensatz"));
        let mut reiter = iced::widget::row![].spacing(4);
        for (i, p) in ts.charsets.iter().enumerate() { reiter = reiter.push(tb(&format!("{} {}", (b'A' + (i % 26) as u8) as char, vc_media::naming::stem(p)), p, true, TAkt::Reiter(i), d.reiter == i)) }
        dokument = dokument.push(reiter.wrap());
        for c in d.zeichensatz_angebot() { dokument = dokument.push(tb(&format!("+ {}", c.rsplit('/').next().unwrap_or(&c)), "Zeichensatz zum Satz hinzufügen", true, TAkt::ZeichensatzPlus(c), false)) }
        dokument = dokument.push(tb("Zeichensatz entfernen", if d.zeichensatz_benutzt(d.reiter) { "Der Zeichensatz wird noch von Zellen benutzt" } else { "Reiter entfernen" }, d.reiter < ts.charsets.len() && !d.zeichensatz_benutzt(d.reiter), TAkt::ZeichensatzMinus, false));
        if ts.charsets.is_empty() { dokument = dokument.push(text("Dieser Satz hat noch keinen Zeichensatz: oben hinzufügen.").size(11).color(pal.mute)) }
        else {
            let n = if d.atari() { 128 } else { 256 }; let kante = 16.0;
            let benutzt: Vec<bool> = (0..n).map(|c| d.benutzt.contains(&(d.reiter, c as u8))).collect();
            let tafel = canvas::Canvas::new(Tafel { bild: handle(&d.tafel_bild(2, &rgb)), kante, zeilen: n / 16, gewaehlt: (d.pinsel.cs == d.reiter).then_some(d.pinsel.code), benutzt, pal, rel: rel.to_string() }).width(16.0 * kante).height((n / 16) as f32 * kante);
            werkzeug = werkzeug.push(text(format!("Zeichen {} (${:02X})", d.pinsel.code, d.pinsel.code)).size(11.5).color(pal.mute)).push(tafel);
        }
        dokument = dokument.push(kopf("Hintergrund")).push(farbfeld(d.hintergrundfarbe(), Box::new(TAkt::Bg)));
        dokument = dokument.push(text(self.palettenzeile(&d.target, s.ansicht)).size(10.5).color(pal.mute));
        if self.farben.hat_varianten(&d.target) { let r3 = r.clone(); dokument = dokument.push(self.ansicht_wahl_mit(pal, s.ansicht, move |a| Msg::TileAkt(r3.clone(), TAkt::Ansicht(a)))) }
        // Karten: Füllstand (immer sichtbar) · Werkzeug · Dokument · Info (Verwendung)
        let karte_zu = |k: &'static str| -> Option<(bool, Msg)> { Some((!s.karten_zu.contains(k), Msg::TileAkt(r.clone(), TAkt::Karte(k)))) };
        let mut verw_info = iced::widget::column![].spacing(2);
        if let Some(t) = ts.tiles.get(d.sel) {
            let c = &t.cells[d.cur.min(t.cells.len().saturating_sub(1))];
            verw_info = verw_info.push(text(format!("Zelle {},{} · Zeichen {} (${:02X}) · Zeichensatz {}{}", d.cur % w, d.cur / w, c.code, c.code, (b'A' + (c.cs % 26) as u8) as char, if c.sel { " (Farbwahl)" } else { "" })).size(muster::schrift::HINWEIS).color(pal.mute));
        }
        if !d.banken.is_empty() { for b in &d.banken { verw_info = verw_info.push(text(format!("{}: {} Zelle(n) in {} Screen(s) der Bank", b.rel, b.counts.get(d.sel).copied().unwrap_or(0), b.screens)).size(muster::schrift::HINWEIS).color(pal.mute)) } }
        let kachel_name = ts.tiles.get(d.sel).map(|t| if t.name.is_empty() { format!("Kachel {}", d.sel) } else { format!("Kachel {} · {}", d.sel, t.name) });
        let karten: Vec<Element<Msg>> = vec![
            muster::karte(pal, "Kachel", "▣", "diese Kachel · wird gespeichert", karte_zu("kachel"), kachel_teil.into()),
            muster::karte(pal, "Werkzeug", "✎", "fürs Zeichnen · nicht in der Datei", karte_zu("werkzeug"), werkzeug.into()),
            muster::karte(pal, "Eigenschaften", "⚑", "eigene Marken, die dein Programm abfragen kann", karte_zu("eigenschaften"), eigen.into()),
            muster::karte(pal, "Füllstand", "▮", "Zeichen des Zielzeichensatzes und Kacheln", karte_zu("fuellstand"), fuell.push(text(format!("{} von {} Kacheln", ts.tiles.len(), vc_media::tiles::MAX_TILES)).size(muster::schrift::HINWEIS).color(pal.mute)).into()),
            muster::karte(pal, "Zeichensatz und Hintergrund", "▤", "Satz · wird gespeichert", karte_zu("dokument"), dokument.into()),
            muster::karte(pal, "Info", "ⓘ", "", karte_zu("info"), verw_info.into())];
        let seite = muster::seitenspalte(klasse, karten);
        // Zone 1: Kopfzeile
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let kopf_z = muster::kopfzeile(pal, klasse, "MAP", &name, d.dirty(), &format!("{} Kacheln · {}×{} Zellen", ts.tiles.len(), w, h), vec![
            tb("Exportieren …", "Export-Dialog (Cmd+Umschalt+E)", true, TAkt::Exportieren, false), tb("Speichern", "Speichern (Cmd+S)", d.dirty(), TAkt::Speichern, false)]);
        let status = if d.status.is_empty() { String::new() } else { format!(" · {}", d.status) };
        let fuss = muster::fusszeile(pal, &format!("Kachel {} · Zelle {},{} · Zoom {}{status}", d.sel, d.cur % w, d.cur / w, if s.zoomansicht.platz.get().0 < 1.0 { "Einpassen".to_string() } else { format!("{}{}×", if s.zoomansicht.zoom.is_none() { "Einpassen · " } else { "" }, s.skala.get()) }),
            tooltip(text("? Tasten").size(muster::schrift::HINWEIS).color(pal.mute), container(text("linke Taste setzt den Pinsel, rechte Taste nimmt die Zelle auf · Pfeile wandern in der Kachel · + − Zoom · 0 Einpassen · Leertaste+Ziehen verschiebt · Cmd+Mausrad zoomt").size(11.5).color(pal.text)).padding([4, 8]).max_width(420).style(move |_| flaeche(pal.panel2, pal.line)), tooltip::Position::Top).into());
        // Überlagerungen: Rückfrage (Löschen/Größe) und ⋯-Karte
        let mut flaeche_el: Element<Msg> = container(flaeche_roh).padding(iced::Padding { top: 28.0, ..Default::default() }).into();
        if let Some(fr) = &d.frage {
            let msg = match fr {
                Frage::Loeschen(i) => { let n: usize = d.banken.iter().filter(|b| b.counts.get(*i).copied().unwrap_or(0) > 0).map(|b| b.screens).sum(); format!("Kachel {i} wird {}× in {n} Screen(s) benutzt. Beim Löschen werden die Screen-Bänke angepasst (Verweise werden zu Kachel 0, höhere Nummern rücken nach) und der Satz wird gespeichert.", d.verwendung(*i)) }
                Frage::Groesse(w2, h2) => format!("Größe {w2}×{h2}: belegte Zellen außerhalb gehen verloren."),
            };
            let dlg = container(iced::widget::column![text(msg).size(12).color(pal.warn), iced::widget::row![knopf(pal, if matches!(fr, Frage::Loeschen(_)) { "Löschen" } else { "Beschneiden" }, Some(Msg::TileAkt(r.clone(), TAkt::FrageJa)), pal.text, true), knopf(pal, "Abbrechen", Some(Msg::TileAkt(r.clone(), TAkt::FrageNein)), pal.text, false)].spacing(8)].spacing(8)).padding(10).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.warn, width: 1.0, radius: 8.0.into() }, ..Default::default() });
            flaeche_el = iced::widget::stack![flaeche_el, container(container(dlg).max_width(420)).padding(muster::abstand::BEREICH).center(Length::Fill)].into();
        }
        if let Some(n) = kachel_name { flaeche_el = iced::widget::stack![flaeche_el, container(text(n).size(muster::schrift::TEXT).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)).padding(muster::abstand::GRUPPE)].into(); }
        if s.mehr {
            let karte_mehr = container(iced::widget::column![text("Transformieren").size(muster::schrift::HINWEIS).color(pal.mute), transformieren().wrap().vertical_spacing(4), text("Ansicht").size(muster::schrift::HINWEIS).color(pal.mute), zoomgruppe()].spacing(muster::abstand::GRUPPE)).padding(muster::abstand::KARTE).max_width(360).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() });
            flaeche_el = iced::widget::stack![flaeche_el, container(karte_mehr).padding(muster::abstand::GRUPPE).align_right(Length::Fill)].into();
        }
        let flaeche_el: Element<Msg> = container(flaeche_el).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        let _ = sehr_schmal;
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: 200.0, kopf: kopf_z, leiste, liste: Some(liste_el), flaeche: flaeche_el, seite, fuss })
    }
}
