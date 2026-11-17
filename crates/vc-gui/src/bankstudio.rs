// SPDX-License-Identifier: GPL-3.0-or-later
//! Screen-Bank-Studio (`.vcscreen` mit `screens`): oben die Screens der Bank, links im Kachelmodus die Kachelpalette, in der Mitte der Screen als Bild
//! (Kachelebene und Zellebene), rechts Kachelsatz, Größe und Ursprung, Farben, Zeichenwähler und Eigenschaften-Ebene. Modell: `vc_ide_model::bankdoc::BankDoc`.
use super::*;
use crate::studio::{Ansicht, Farben};
use vc_gui_ui::muster;
use iced::widget::{canvas, text};
use vc_ide_model::bankdoc::{grid_dims, BankDoc, Modus, Werkzeug, HOLE};
use vc_ide_model::glyphs::Bild;
use vc_media::tiles::ColorMode;

pub struct BankStudio { pub doc: BankDoc, pub zoomansicht: vc_gui_ui::flaeche::Ansicht, pub skala: std::cell::Cell<usize>, pub mehr: bool, pub karten_zu: std::collections::BTreeSet<&'static str>, pub ansicht: Ansicht, pub ueber: Option<(i32, i32)>, pub unten: bool, pub kachelgrenzen: bool, pub zellgitter: bool, /** Miniaturen der Screens: (Stand des Inhalts, Bilder) */ pub minis: std::cell::RefCell<(u64, Vec<iced::widget::image::Handle>)> }

#[derive(Clone, Debug, PartialEq)]
pub enum BAkt {
    Undo, Redo, Speichern, Exportieren, Modus(Modus), Werkzeug(Werkzeug), Gefuellt(bool), Screen(usize), ScreenNeu, ScreenDupl, ScreenLoeschen, ScreenVor(i32), ScreenName(String), Groesse(usize, usize, usize, usize, bool),
    Farbe(&'static str, usize), Satz(String), Kachel(usize), Zeichen(u8), Reiter(usize), Pinselfarbe(usize), Bit7(bool), Zoom(i32), Cursor(i32, i32), Setzen, Einpassen, Schieben(f32, f32), Mehr, Karte(&'static str), Gitter(bool), Kachelgrenzen(bool), Eigenschaft(Option<u8>),
    AuswahlKopieren, AuswahlLeeren, AlsKachel, NKName(String), NKErsetzen(bool), NKOk, NKAbbruch, Ansicht(Ansicht),
}
#[derive(Clone, Debug, PartialEq)]
pub enum BMaus { Unten { fx: i32, fy: i32, rechts: bool }, Zug { fx: i32, fy: i32 }, Los { fx: i32, fy: i32 }, Ueber(Option<(i32, i32)>) }

fn handle(b: &Bild) -> iced::widget::image::Handle { crate::bildcache::handle(b.w as u32, b.h as u32, &b.rgba) }

/// Höchste Vergrößerung des Screens (das Bild wird in jeder Stufe neu erzeugt)
const SKALA_MAX: usize = 8;
/// Größe des Screen-Bilds bei Zoom 1 in Bildpunkten (Rand eingerechnet): die „Einheiten“ für Einpassen und Zoom
fn einheiten(d: &BankDoc) -> (f32, f32) { let (w, h) = d.screen().map_or((1, 1), |s| (s.w, s.h)); (((w + 2 * d.rand()) * 8) as f32, ((h + 2 * d.rand()) * 8) as f32) }
fn lage(b: iced::Size, bw: f32, bh: f32, a: &vc_gui_ui::flaeche::Ansicht) -> (f32, f32) {
    a.ecke(b, bw, bh)
}

/// Bildebene: der fertige Screen (eigene Ebene unter Gittern und Auswahl, damit nichts flackert)
struct BildEbene<'a> { s: &'a BankStudio, bild: iced::widget::image::Handle, bw: f32, bh: f32 }
impl<'a> canvas::Program<Msg> for BildEbene<'a> {
    type State = ();
    fn update(&self, _st: &mut (), _e: &iced::Event, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> { None }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let (ox, oy) = lage(b.size(), self.bw, self.bh, &self.s.zoomansicht);
        f.draw_image(iced::Rectangle::new(iced::Point::new(ox, oy), iced::Size::new(self.bw, self.bh)), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        vec![f.into_geometry()]
    }
}

/// Die Zeichenfläche des Screens (obere Ebene): Klicks, Gitter, Auswahl, Ziehvorschau, aktive Einheit
struct Flaeche<'a> { s: &'a BankStudio, skala: usize, bw: f32, bh: f32, pal: Pal, rel: String }
#[derive(Default)] struct FlaechenZustand { unten: bool, gesten: vc_gui_ui::flaeche::Gesten }
impl<'a> Flaeche<'a> {
    fn ursprung(&self, b: iced::Rectangle) -> (f32, f32) { lage(b.size(), self.bw, self.bh, &self.s.zoomansicht) }
    fn zelle(&self, b: iced::Rectangle, p: iced::Point) -> (i32, i32) {
        let (ox, oy) = self.ursprung(b); let (k, rd) = (self.skala as f32 * 8.0, (self.s.doc.rand() * 8 * self.skala) as f32);
        (((p.x - ox - rd) / k).floor() as i32, ((p.y - oy - rd) / k).floor() as i32)
    }
}
impl<'a> canvas::Program<Msg> for Flaeche<'a> {
    type State = FlaechenZustand;
    fn update(&self, st: &mut FlaechenZustand, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        use iced::mouse::{Button, Event as M};
        // Leertaste + Ziehen verschiebt, Cmd/Strg + Mausrad zoomt (Tasten bleiben bei der Befehlstabelle)
        if let Some(g) = st.gesten.ereignis(e, b, c) {
            use vc_gui_ui::flaeche::Geste;
            return match g { Geste::Erfasst => Some(canvas::Action::capture()), Geste::Schieben(dx, dy) => Some(canvas::Action::publish(Msg::BankAkt(self.rel.clone(), BAkt::Schieben(dx, dy))).and_capture()), Geste::Zoom(plus) => Some(canvas::Action::publish(Msg::BankAkt(self.rel.clone(), BAkt::Zoom(if plus { 1 } else { -1 }))).and_capture()) }
        }
        let rel = self.rel.clone(); let send = |m: BMaus| Some(canvas::Action::publish(Msg::BankMaus(rel.clone(), m)).and_capture());
        match e {
            iced::Event::Mouse(M::ButtonPressed(k @ (Button::Left | Button::Right))) => { let p = c.position_in(b)?; st.unten = true; let (fx, fy) = self.zelle(b, p); send(BMaus::Unten { fx, fy, rechts: *k == Button::Right }) }
            iced::Event::Mouse(M::CursorMoved { .. }) => {
                let rel_pos = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?;
                let (fx, fy) = self.zelle(b, rel_pos);
                if st.unten { send(BMaus::Zug { fx, fy }) } else { let z = c.position_in(b).map(|_| (fx, fy)); if z != self.s.ueber { Some(canvas::Action::publish(Msg::BankMaus(rel, BMaus::Ueber(z)))) } else { None } }
            }
            iced::Event::Mouse(M::ButtonReleased(Button::Left | Button::Right)) if st.unten => { st.unten = false; let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let (fx, fy) = self.zelle(b, p); send(BMaus::Los { fx, fy }) }
            _ => None,
        }
    }
    fn draw(&self, _st: &FlaechenZustand, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        self.s.zoomansicht.platz.set((b.width, b.height));
        let mut f = canvas::Frame::new(r, b.size()); let (ox, oy) = self.ursprung(b); let d = &self.s.doc;
        let (Some(s), Some(ts)) = (d.screen(), d.ts.as_ref()) else { return vec![f.into_geometry()] };
        let (k, rd) = (self.skala as f32 * 8.0, (d.rand() * 8 * self.skala) as f32); let (x0, y0) = (ox + rd, oy + rd);
        if self.s.zellgitter { let g = canvas::Stroke::default().with_color(Color { a: 0.12, ..Color::WHITE }).with_width(1.0); for x in 0..=s.w { f.stroke(&canvas::Path::line(iced::Point::new(x0 + x as f32 * k, y0), iced::Point::new(x0 + x as f32 * k, y0 + s.h as f32 * k)), g) } for y in 0..=s.h { f.stroke(&canvas::Path::line(iced::Point::new(x0, y0 + y as f32 * k), iced::Point::new(x0 + s.w as f32 * k, y0 + y as f32 * k)), g) } }
        let (cols, rows) = grid_dims(s, ts);
        if self.s.kachelgrenzen && cols > 0 && rows > 0 { let g = canvas::Stroke::default().with_color(Color { a: 0.3, ..Color::WHITE }).with_width(1.0);
            for gx in 0..=cols { let px = x0 + (s.origin.0 + gx * ts.tw) as f32 * k; f.stroke(&canvas::Path::line(iced::Point::new(px, y0 + s.origin.1 as f32 * k), iced::Point::new(px, y0 + (s.origin.1 + rows * ts.th) as f32 * k)), g) }
            for gy in 0..=rows { let py = y0 + (s.origin.1 + gy * ts.th) as f32 * k; f.stroke(&canvas::Path::line(iced::Point::new(x0 + s.origin.0 as f32 * k, py), iced::Point::new(x0 + (s.origin.0 + cols * ts.tw) as f32 * k, py)), g) } }
        // Eigenschaften-Ebene
        if let Some(bit) = d.eigenschaft { let farben = [Color::from_rgb8(220, 60, 60), Color::from_rgb8(70, 110, 230), Color::from_rgb8(70, 190, 90), Color::from_rgb8(230, 190, 50), Color::from_rgb8(170, 90, 220), Color::from_rgb8(240, 140, 50), Color::from_rgb8(60, 200, 210), Color::from_rgb8(230, 110, 170)];
            for (gy, row) in s.grid.iter().enumerate() { for (gx, &t) in row.iter().enumerate() { if ts.tiles.get(t).is_some_and(|t| t.props >> bit & 1 == 1) { f.fill_rectangle(iced::Point::new(x0 + (s.origin.0 + gx * ts.tw) as f32 * k, y0 + (s.origin.1 + gy * ts.th) as f32 * k), iced::Size::new(ts.tw as f32 * k, ts.th as f32 * k), Color { a: 0.45, ..farben[bit as usize % 8] }) } } } }
        // Einheit im Modus als Rechteck
        let einheit = |u: (usize, usize)| -> (iced::Point, iced::Size) { match d.modus { Modus::Kachel => (iced::Point::new(x0 + (s.origin.0 + u.0 * ts.tw) as f32 * k, y0 + (s.origin.1 + u.1 * ts.th) as f32 * k), iced::Size::new(ts.tw as f32 * k, ts.th as f32 * k)), Modus::Zelle => (iced::Point::new(x0 + u.0 as f32 * k, y0 + u.1 as f32 * k), iced::Size::new(k, k)) } };
        let hover = self.s.ueber.and_then(|(fx, fy)| d.einheit_bei(fx, fy, false));
        if let (Some(a), Some(h)) = (d.von, self.s.ueber.and_then(|(fx, fy)| d.einheit_bei(fx, fy, true))) {
            let pts = match d.werkzeug { Werkzeug::Linie => vc_ide_model::bankdoc::line_points(a, h), Werkzeug::Rechteck => vc_ide_model::bankdoc::rect_points(a, h, d.gefuellt), Werkzeug::Auswahl => vc_ide_model::bankdoc::rect_points(a, h, true), _ => vec![] };
            for u in pts { let (p, sz) = einheit(u); f.fill_rectangle(p, sz, Color { a: 0.25, ..Color::WHITE }) } }
        if d.werkzeug == Werkzeug::Stempel && d.modus == Modus::Kachel { if let (Some(h), Some(clip)) = (hover, &d.stempel) { for (dy, row) in clip.iter().enumerate() { for (dx, &t) in row.iter().enumerate() { if t != HOLE && h.1 + dy < rows && h.0 + dx < cols { let (p, sz) = einheit((h.0 + dx, h.1 + dy)); f.stroke_rectangle(p, sz, canvas::Stroke::default().with_color(self.pal.accent).with_width(1.5)) } } } } }
        else if let Some(h) = hover { let (p, sz) = einheit(h);
            // aktive Einheit unter dem Zeiger: Doppelrahmen (innen Akzent 2 px, außen Text gestrichelt) mit Achsenmarken
            f.stroke_rectangle(iced::Point::new(p.x - 1.0, p.y - 1.0), iced::Size::new(sz.width + 2.0, sz.height + 2.0), canvas::Stroke::default().with_color(self.pal.accent).with_width(2.0));
            f.stroke_rectangle(iced::Point::new(p.x - 3.5, p.y - 3.5), iced::Size::new(sz.width + 7.0, sz.height + 7.0), canvas::Stroke { line_dash: canvas::LineDash { segments: &[4.0, 3.0], offset: 0 }, ..canvas::Stroke::default().with_color(self.pal.text).with_width(1.5) });
            f.fill_rectangle(iced::Point::new(p.x, y0 - 8.0), iced::Size::new(sz.width, 4.0), self.pal.accent); f.fill_rectangle(iced::Point::new(x0 - 8.0, p.y), iced::Size::new(4.0, sz.height), self.pal.accent); }
        if let Some((a, bb)) = d.auswahl { let (p0, s0) = einheit(a); let (p1, s1) = einheit(bb); f.stroke_rectangle(p0, iced::Size::new(p1.x + s1.width - p0.x, p1.y + s1.height - p0.y), canvas::Stroke::default().with_color(self.pal.warn).with_width(2.0)) }
        if d.werkzeug == Werkzeug::Text && d.modus == Modus::Zelle { f.stroke_rectangle(iced::Point::new(x0 + d.text_pos.0 as f32 * k, y0 + d.text_pos.1 as f32 * k), iced::Size::new(k, k), canvas::Stroke::default().with_color(self.pal.warn).with_width(2.0)) }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &FlaechenZustand, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Crosshair }
}

/// Zeichentafel (Zellebene) oder Kachelpalette: Klick wählt
struct Tafel { bild: iced::widget::image::Handle, kw: f32, kh: f32, spalten: usize, anzahl: usize, gewaehlt: Option<usize>, kachel: bool, pal: Pal, rel: String }
impl canvas::Program<Msg> for Tafel {
    type State = ();
    fn update(&self, _st: &mut (), e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = e {
            let p = c.position_in(b)?; let (x, y) = ((p.x / self.kw) as usize, (p.y / self.kh) as usize); let i = y * self.spalten + x;
            if x < self.spalten && i < self.anzahl { let a = if self.kachel { BAkt::Kachel(i) } else { BAkt::Zeichen(i as u8) }; return Some(canvas::Action::publish(Msg::BankAkt(self.rel.clone(), a)).and_capture()) }
        }
        None
    }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size());
        f.draw_image(iced::Rectangle::new(iced::Point::ORIGIN, b.size()), canvas::Image::new(self.bild.clone()).filter_method(iced::widget::image::FilterMethod::Nearest));
        if let Some(g) = self.gewaehlt { f.stroke_rectangle(iced::Point::new((g % self.spalten) as f32 * self.kw, (g / self.spalten) as f32 * self.kh), iced::Size::new(self.kw, self.kh), canvas::Stroke::default().with_color(self.pal.accent).with_width(2.0)) }
        vec![f.into_geometry()]
    }
}

impl App {
    pub(crate) fn bank_oeffnen(&mut self, rel: &str) {
        if !self.banken.contains_key(rel) {
            match BankDoc::oeffnen(&self.root, rel, &self.data) { Ok(doc) => { self.banken.insert(rel.to_string(), BankStudio { doc, zoomansicht: vc_gui_ui::flaeche::Ansicht::mit_max(SKALA_MAX as f32), skala: std::cell::Cell::new(1), mehr: false, karten_zu: Default::default(), ansicht: Ansicht::Projekt, ueber: None, unten: false, kachelgrenzen: true, zellgitter: false, minis: Default::default() }); } Err(e) => { self.log.push(e); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }

    pub(crate) fn bank_maus(&mut self, rel: &str, m: BMaus) {
        let Some(s) = self.banken.get_mut(rel) else { return };
        match m {
            BMaus::Unten { fx, fy, rechts } => { s.unten = true; let (h, n) = (s.doc.einheit_bei(fx, fy, false), s.doc.einheit_bei(fx, fy, true)); s.doc.maus_unten(h, n, rechts) }
            BMaus::Zug { fx, fy } => { s.ueber = Some((fx, fy)); if s.unten { let n = s.doc.einheit_bei(fx, fy, true); s.doc.maus_zug(n) } }
            BMaus::Los { fx, fy } => { if s.unten { s.unten = false; let n = s.doc.einheit_bei(fx, fy, true); s.doc.maus_los(n) } }
            BMaus::Ueber(z) => s.ueber = z,
        }
    }

    pub(crate) fn bank_aktion(&mut self, rel: &str, a: BAkt) {
        if a == BAkt::Exportieren { self.export_oeffnen(); return }
        let Some(s) = self.banken.get_mut(rel) else { return }; let d = &mut s.doc;
        match a {
            BAkt::Undo => d.rueckgaengig(), BAkt::Redo => d.wiederholen(),
            BAkt::Speichern => match d.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {rel}")), Err(e) => self.log.push(e) },
            BAkt::Modus(m) => d.modus_waehlen(m), BAkt::Werkzeug(w) => d.werkzeug_waehlen(w), BAkt::Gefuellt(b) => d.gefuellt = b,
            BAkt::Screen(i) => d.screen_waehlen(i), BAkt::ScreenNeu => d.screen_neu(), BAkt::ScreenDupl => d.screen_duplizieren(), BAkt::ScreenLoeschen => d.screen_loeschen(), BAkt::ScreenVor(x) => d.screen_verschieben(x), BAkt::ScreenName(n) => d.screen_umbenennen(&n, false),
            BAkt::Groesse(w, h, ox, oy, e) => d.screen_groesse(w, h, ox, oy, e), BAkt::Farbe(r, i) => d.screen_farbe(r, i), BAkt::Satz(p) => d.satz_waehlen(&p),
            BAkt::Kachel(i) => d.kachel_pinsel(i), BAkt::Zeichen(c) => d.zeichen_pinsel(c), BAkt::Reiter(i) => d.reiter_waehlen(i), BAkt::Pinselfarbe(i) => d.farbe = i, BAkt::Bit7(b) => d.zell_pinsel.sel = b,
            BAkt::Zoom(z) => { let (w, h) = einheiten(&s.doc); s.zoomansicht.stufe(z > 0, w, h) } BAkt::Einpassen => s.zoomansicht.einpassen(), BAkt::Schieben(dx, dy) => { let (w, h) = einheiten(&s.doc); s.zoomansicht.schieben(dx, dy, w, h) }
            // Tastatur (Entscheidung 19): Pfeile bewegen die aktive Einheit (Kachel bzw. Zelle), Leertaste setzt dort wie ein Klick
            BAkt::Cursor(dx, dy) => { if let (Some(sc), Some(ts)) = (d.screen(), d.ts.as_ref()) { let (sx, sy) = if d.modus == Modus::Kachel { (ts.tw as i32, ts.th as i32) } else { (1, 1) };
                let (fx, fy) = s.ueber.unwrap_or((sc.origin.0 as i32, sc.origin.1 as i32)); s.ueber = Some(((fx + dx * sx).clamp(0, sc.w as i32 - 1), (fy + dy * sy).clamp(0, sc.h as i32 - 1))) } }
            BAkt::Setzen => { if let Some((fx, fy)) = s.ueber { let rel2 = rel.to_string(); self.bank_maus(&rel2, BMaus::Unten { fx, fy, rechts: false }); self.bank_maus(&rel2, BMaus::Los { fx, fy }); } return }
            BAkt::Mehr => s.mehr = !s.mehr, BAkt::Karte(k) => { if !s.karten_zu.remove(k) { s.karten_zu.insert(k); } }
            BAkt::Gitter(b) => s.zellgitter = b, BAkt::Kachelgrenzen(b) => s.kachelgrenzen = b, BAkt::Eigenschaft(b) => d.eigenschaft = b,
            BAkt::AuswahlKopieren => d.auswahl_kopieren(), BAkt::AuswahlLeeren => d.auswahl_leeren(),
            BAkt::AlsKachel => { if let Some(((x0, y0), _)) = d.auswahl { let at = match (d.modus, d.screen(), d.ts.as_ref()) { (Modus::Kachel, Some(sc), Some(ts)) => (sc.origin.0 + x0 * ts.tw, sc.origin.1 + y0 * ts.th), _ => (x0, y0) }; d.neue_kachel_anfragen(at) } }
            BAkt::NKName(n) => { if let Some(nt) = &mut d.neue_kachel { nt.name = n } } BAkt::NKErsetzen(b) => { if let Some(nt) = &mut d.neue_kachel { if nt.ausgerichtet { nt.ersetzen = b } } } BAkt::NKOk => d.neue_kachel_ok(), BAkt::NKAbbruch => d.neue_kachel_abbrechen(),
            BAkt::Ansicht(x) => s.ansicht = x, BAkt::Exportieren => {}
        }
        if !self.banken.get(rel).is_some_and(|s| s.doc.ts_dirty) { /* Satz unverändert */ } else if let Some(s) = self.banken.get(rel) { if s.doc.status.starts_with("Kachel") { self.log.push(s.doc.status.clone()) } }
    }

    /// Tasten: Tab wechselt den Modus, Text tippen, Einzeltasten für Werkzeuge, Entf leert die Auswahl
    pub(crate) fn bank_taste(&mut self, rel: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
        if m.command() || m.alt() || m.control() { return false }
        let Some(s) = self.banken.get(rel) else { return false }; let (text_modus, hat_auswahl, modus) = (s.doc.werkzeug == Werkzeug::Text && s.doc.modus == Modus::Zelle, s.doc.auswahl.is_some(), s.doc.modus);
        use keyboard::key::Named;
        let a = match k {
            keyboard::Key::Named(Named::Tab) => BAkt::Modus(if modus == Modus::Kachel { Modus::Zelle } else { Modus::Kachel }),
            keyboard::Key::Named(Named::Escape) if self.banken.get(rel).is_some_and(|s| s.doc.von.is_some()) => { if let Some(s) = self.banken.get_mut(rel) { s.doc.von = None } return true }
            keyboard::Key::Named(Named::Escape) if text_modus => BAkt::Werkzeug(Werkzeug::Stift),
            keyboard::Key::Named(Named::Backspace) if text_modus => { if let Some(s) = self.banken.get_mut(rel) { s.doc.text_zurueck() } return true }
            keyboard::Key::Named(Named::Enter) if text_modus => { if let Some(s) = self.banken.get_mut(rel) { s.doc.text_zeilenwechsel() } return true }
            keyboard::Key::Named(Named::Space) if text_modus => { if let Some(s) = self.banken.get_mut(rel) { s.doc.tippe(" ") } return true }
            keyboard::Key::Character(c) if text_modus => { if let Some(s) = self.banken.get_mut(rel) { s.doc.tippe(c.as_str()) } return true }
            keyboard::Key::Named(Named::Delete | Named::Backspace) if hat_auswahl => BAkt::AuswahlLeeren,
            keyboard::Key::Named(Named::ArrowLeft) => BAkt::Cursor(-1, 0), keyboard::Key::Named(Named::ArrowRight) => BAkt::Cursor(1, 0), keyboard::Key::Named(Named::ArrowUp) => BAkt::Cursor(0, -1), keyboard::Key::Named(Named::ArrowDown) => BAkt::Cursor(0, 1),
            keyboard::Key::Named(Named::Space) => BAkt::Setzen,
            keyboard::Key::Character(c) => match c.as_str().to_lowercase().as_str() {
                "d" => BAkt::Werkzeug(Werkzeug::Stift), "l" => BAkt::Werkzeug(Werkzeug::Linie), "r" => BAkt::Werkzeug(Werkzeug::Rechteck), "f" => BAkt::Werkzeug(Werkzeug::Fuellen), "i" => BAkt::Werkzeug(Werkzeug::Pipette), "a" => BAkt::Werkzeug(Werkzeug::Auswahl), "e" => BAkt::Werkzeug(Werkzeug::Loeschen), "t" => BAkt::Werkzeug(Werkzeug::Text), "s" => BAkt::Werkzeug(Werkzeug::Stempel),
                "+" | "=" => BAkt::Zoom(1), "-" => BAkt::Zoom(-1), "0" => BAkt::Einpassen, _ => return false },
            _ => return false,
        };
        self.bank_aktion(rel, a); true
    }

    pub(crate) fn bank_a11y(&self) -> Option<(String, String)> {
        let n = self.name()?; let s = self.banken.get(&n)?; let d = &s.doc; let sc = d.screen()?;
        Some((format!("Screen-Bank {n}"), format!("Screen {} von {} „{}“, {}×{} Zellen, {} Patches, Modus {:?}, Werkzeug {:?}, Kachelsatz {}", d.cur + 1, d.bank.screens.len(), sc.name, sc.w, sc.h, sc.patches.len(), d.modus, d.werkzeug, d.bank.tilesets.first().cloned().unwrap_or_default())))
    }

    pub(crate) fn bank_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.bank_ansicht_k(pal, &rel, muster::klasse(size.width), size.width)).into()
    }
    fn bank_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse, breite: f32) -> Element<'_, Msg> {
        let Some(s) = self.banken.get(rel) else { return text("").into() }; let d = &s.doc; let r = rel.to_string();
        let norm = self.norm_fuer(&d.target, s.ansicht); let farben: &Farben = &self.farben; let rgb = |i: usize| farben.rgb(&d.target, norm, i);
        let tb = |name: &str, tipp: &str, an: bool, a: BAkt, aktiv: bool| -> Element<Msg> { muster::knopf(pal, name, tipp, an.then(|| Msg::BankAkt(r.clone(), a)), aktiv, false) };
        let kopf = |t: &str| text(t.to_string()).size(muster::schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute);
        // Zone 2: genau eine Leistenreihe: Modus · Werkzeuge · Bearbeiten · Ansicht; bei weniger Breite stehen Werkzeuge und Ansicht im ⋯-Menü
        let modus = iced::widget::row![tb("Kacheln", "Kachelmodus (Tab wechselt)", true, BAkt::Modus(Modus::Kachel), d.modus == Modus::Kachel), tb("Zeichen", "Zeichenmodus: freie Zellen über den Kacheln (Tab wechselt)", true, BAkt::Modus(Modus::Zelle), d.modus == Modus::Zelle)].spacing(4);
        let werkzeuge = || { let mut z = iced::widget::row![].spacing(4).align_y(Alignment::Center); for (w, n) in d.werkzeuge() { z = z.push(tb(n, &format!("{n}{}", werkzeug_taste(*w).map(|k| format!(" ({k})")).unwrap_or_default()), true, BAkt::Werkzeug(*w), d.werkzeug == *w)) } z };
        let bearbeiten = iced::widget::row![tb("↶", "Rückgängig (Cmd+Z)", d.kann_undo(), BAkt::Undo, false), tb("↷", "Wiederholen (Cmd+Umschalt+Z)", d.kann_redo(), BAkt::Redo, false)].spacing(4);
        let ansicht = || iced::widget::row![tb("−", "Verkleinern (−, Cmd+Mausrad)", true, BAkt::Zoom(-1), false), tb("+", "Vergrößern (+, Cmd+Mausrad)", true, BAkt::Zoom(1), false), tb("Einpassen", "Fläche einpassen (0)", true, BAkt::Einpassen, s.zoomansicht.zoom.is_none()),
            tb("▦", "Kachelgrenzen zeigen", true, BAkt::Kachelgrenzen(!s.kachelgrenzen), s.kachelgrenzen), tb("▤", "Zellgitter zeigen", true, BAkt::Gitter(!s.zellgitter), s.zellgitter)].spacing(4).align_y(Alignment::Center);
        let mehr_knopf = tb("⋯", "Werkzeuge und Ansicht · alle Aktionen: Menü-Taste oder Umschalt+F10", true, BAkt::Mehr, s.mehr);
        // zu schmal für acht Werkzeuge in der Leiste (auch „mittel“ unter ~780 px): Werkzeuge und Ansicht im ⋯-Menü
        let kompakt = klasse == muster::Klasse::Schmal || (klasse == muster::Klasse::Mittel && breite < 780.0) || (klasse == muster::Klasse::Breit && breite < 1180.0);
        let leiste_inhalt: Element<Msg> = match klasse {
            muster::Klasse::Breit if kompakt => iced::widget::row![modus, muster::trenner(pal), bearbeiten, space::horizontal(), ansicht(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Breit => iced::widget::row![modus, muster::trenner(pal), werkzeuge(), muster::trenner(pal), bearbeiten, space::horizontal(), ansicht()].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Mittel if !kompakt => iced::widget::row![modus, muster::trenner(pal), werkzeuge(), muster::trenner(pal), bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Mittel => iced::widget::row![modus, muster::trenner(pal), bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
            muster::Klasse::Schmal => iced::widget::row![modus, muster::trenner(pal), bearbeiten, space::horizontal(), mehr_knopf].spacing(8).align_y(Alignment::Center).into(),
        };
        let leiste = muster::leiste(pal, leiste_inhalt);
        // Zone 3: Liste der Screens (Name, Reihenfolge, Neu/Duplizieren/Löschen)
        let mut liste = iced::widget::column![text(format!("Screens · {}", d.bank.screens.len())).size(muster::schrift::KARTE).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute)].spacing(4);
        liste = liste.push(iced::widget::row![tb("Neu", "Leerer Screen in Größe und Farben des aktuellen", true, BAkt::ScreenNeu, false), tb("Dupl.", "Screen kopieren", true, BAkt::ScreenDupl, false), tb("Löschen", "Screen entfernen (Rückgängig stellt ihn wieder her)", d.bank.screens.len() > 1, BAkt::ScreenLoeschen, false)].spacing(4));
        // Miniaturen der Screens (nur neu gerechnet, wenn sich der Inhalt ändert)
        { let stand = d.stand(); let mut m = s.minis.borrow_mut(); if m.0 != stand || m.1.len() != d.bank.screens.len() { m.1 = (0..d.bank.screens.len()).map(|i| handle(&d.bild_screen(i, 1, &rgb, 1.0, 1.0))).collect(); m.0 = stand; } }
        for (i, sc) in d.bank.screens.iter().enumerate() {
            let an = d.cur == i; let mini = s.minis.borrow().1.get(i).cloned();
            let inhalt: Element<Msg> = iced::widget::column![mini.map_or_else(|| Element::from(space::vertical().height(0)), |h| container(iced::widget::image(h).filter_method(iced::widget::image::FilterMethod::Linear).width(Length::Fill)).style(move |_| flaeche(pal.editor, pal.line)).into()), text(format!("{i} {}", sc.name)).size(muster::schrift::TEXT).color(pal.text)].spacing(4).into();
            liste = liste.push(iced::widget::tooltip(button(inhalt).width(Length::Fill).padding(6).on_press(Msg::BankAkt(r.clone(), BAkt::Screen(i))).style(move |_, st| button::Style { background: Some(if an { pal.sel.into() } else if matches!(st, button::Status::Hovered) { pal.line.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() }), container(text("Nummer = Auswahlnummer für screen_select").size(muster::schrift::HINWEIS).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Right).gap(4))
        }
        liste = liste.push(iced::widget::row![tb("←", "Nach vorn", d.cur > 0, BAkt::ScreenVor(-1), false), tb("→", "Nach hinten", d.cur + 1 < d.bank.screens.len(), BAkt::ScreenVor(1), false)].spacing(4));
        if let Some(sc) = d.screen() { let r2 = r.clone(); liste = liste.push(text_input("Name", &sc.name).on_input(move |n| Msg::BankAkt(r2.clone(), BAkt::ScreenName(n))).size(12).padding(3)) }
        let liste_el: Element<Msg> = if klasse == muster::Klasse::Breit { scrollable(liste).into() } else { liste.into() };
        // Karte Werkzeug: Kachelpalette (Kachelmodus) bzw. Zeichen (Zellebene); nicht in der Datei
        let mut werkzeug = iced::widget::column![].spacing(6);
        let ts = d.ts.as_ref();
        match (ts, d.modus) {
            (Some(ts), Modus::Kachel) => {
                werkzeug = werkzeug.push(kopf("Kacheln"));
                let spalten = 4usize; let (kw, kh) = ((ts.tw * 8 * 2) as f32, (ts.th * 8 * 2) as f32); let zeilen = ts.tiles.len().div_ceil(spalten);
                let mut atlas = Bild::neu(spalten * ts.tw * 16, zeilen.max(1) * ts.th * 16, rgb(0)); for i in 0..ts.tiles.len() { let b = d.kachel_bild(i, 2, &rgb); for y in 0..b.h { for x in 0..b.w { let o = (y * b.w + x) * 4; atlas.setze((i % spalten) * ts.tw * 16 + x, (i / spalten) * ts.th * 16 + y, [b.rgba[o], b.rgba[o + 1], b.rgba[o + 2]]) } } }
                werkzeug = werkzeug.push(canvas::Canvas::new(Tafel { bild: handle(&atlas), kw, kh, spalten, anzahl: ts.tiles.len(), gewaehlt: Some(d.pinsel), kachel: true, pal, rel: rel.to_string() }).width(kw * spalten as f32).height(kh * zeilen as f32));
                werkzeug = werkzeug.push(text(format!("Kachel {} {}", d.pinsel, ts.tiles.get(d.pinsel).map(|t| t.name.clone()).unwrap_or_default())).size(11.5).color(pal.text));
                if !d.zuletzt.is_empty() { werkzeug = werkzeug.push(text(format!("zuletzt: {}", d.zuletzt.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" "))).size(11).color(pal.mute)) }
            }
            (Some(_), Modus::Zelle) => { werkzeug = werkzeug.push(kopf("Zellebene")).push(text("Einzelne Zeichen über den Kacheln (Patches, bis 255 je Screen). Zeichen rechts wählen.").size(11.5).color(pal.mute)) }
            _ => { werkzeug = werkzeug.push(text("Kein Kachelsatz").size(12).color(pal.mute)) }
        }
        let mut nk_dialog: Option<Element<Msg>> = None;
        if let Some(nt) = &d.neue_kachel {
            let r2 = r.clone();
            let mut zeilen = iced::widget::column![text(format!("Neue Kachel aus den Zellen ab {},{}", nt.at.0, nt.at.1)).size(13).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)].spacing(6);
            if let Some(dp) = nt.dup { zeilen = zeilen.push(text(format!("Es gibt schon Kachel {dp} mit gleichem Inhalt: sie wird benutzt, es entsteht keine neue.")).size(12).color(pal.warn)) } else { zeilen = zeilen.push(text_input("Name", &nt.name).on_input(move |n| Msg::BankAkt(r2.clone(), BAkt::NKName(n))).size(12).padding(3).width(160)) }
            let r3 = r.clone(); zeilen = zeilen.push(iced::widget::checkbox(nt.ersetzen).label("Zellen im Screen durch den Kachelverweis ersetzen").on_toggle_maybe(nt.ausgerichtet.then(|| move |b| Msg::BankAkt(r3.clone(), BAkt::NKErsetzen(b)))).size(14).text_size(12));
            if !nt.ausgerichtet { zeilen = zeilen.push(text("Ausgegraut: Die Zellen liegen nicht auf dem Kachelraster. Die Kachel entsteht trotzdem, der Screen bleibt unverändert.").size(11).color(pal.mute)) }
            zeilen = zeilen.push(iced::widget::row![knopf(pal, if nt.dup.is_some() { "Benutzen" } else { "Anlegen" }, Some(Msg::BankAkt(r.clone(), BAkt::NKOk)), pal.text, true), knopf(pal, "Abbrechen", Some(Msg::BankAkt(r.clone(), BAkt::NKAbbruch)), pal.text, false)].spacing(8));
            nk_dialog = Some(container(zeilen).padding(muster::abstand::KARTE).max_width(420).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.accent, width: 2.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() }).into());
        }
        // Zone 4: der Screen als Bild, eingepasst in ganzen Stufen; Bild und Zeichenebene getrennt
        let (w1, h1) = einheiten(d); let skala_zuletzt = &s.skala; let rel_c = rel.to_string();
        let flaeche_roh: Element<Msg> = if d.ts.is_some() && d.screen().is_some() { iced::widget::responsive(move |size| {
            let z = s.zoomansicht.zoom.map(|z| z.round()).unwrap_or_else(|| vc_gui_ui::flaeche::Ansicht::einpass(size, w1, h1)).clamp(1.0, SKALA_MAX as f32) as usize;
            skala_zuletzt.set(z); s.zoomansicht.platz.set((size.width, size.height));
            let (ta, ca) = if d.modus == Modus::Kachel { (1.0, 0.45) } else { (0.4, 1.0) }; let bild = d.bild(z, &rgb, ta, ca); let (bw, bh) = (bild.w as f32, bild.h as f32);
            iced::widget::stack![canvas::Canvas::new(BildEbene { s, bild: handle(&bild), bw, bh }).width(Length::Fill).height(Length::Fill), canvas::Canvas::new(Flaeche { s, skala: z, bw, bh, pal, rel: rel_c.clone() }).width(Length::Fill).height(Length::Fill)].into()
        }).into() } else { container(text(d.ts_fehler.clone().unwrap_or_else(|| "Kein Kachelsatz.".into())).size(13).color(pal.err)).center(Length::Fill).into() };
        let farbfeld = |wahl: usize, mach: Box<dyn Fn(usize) -> BAkt>| -> Element<Msg> {
            let eintr = farben.eintraege(&d.target, norm); let mut raster = iced::widget::column![].spacing(2); let mut zeile = iced::widget::row![].spacing(2);
            for (kk, (idx, c)) in eintr.iter().enumerate() { let (idx, c, gew) = (*idx as usize, *c, *idx as usize == wahl); let a = mach(idx);
                zeile = zeile.push(button(space::vertical()).width(16).height(16).padding(0).on_press(Msg::BankAkt(r.clone(), a)).style(move |_, _| button::Style { background: Some(c.into()), border: Border { color: if gew { pal.accent } else { pal.line }, width: if gew { 2.0 } else { 1.0 }, radius: 3.0.into() }, ..Default::default() }));
                if (kk + 1) % 8 == 0 { raster = raster.push(zeile); zeile = iced::widget::row![].spacing(2) } }
            raster.push(zeile).into()
        };
        let mut dokument = iced::widget::column![kopf("Kachelsatz")].spacing(6);
        let cur = d.bank.tilesets.first().cloned().unwrap_or_default();
        for sz in &d.saetze { let on = &cur == sz; dokument = dokument.push(button(text(format!("{} {}", if on { "●" } else { "○" }, sz.rsplit('/').next().unwrap_or(sz))).size(12).color(pal.text)).padding([2, 6]).on_press(Msg::BankAkt(r.clone(), BAkt::Satz(sz.clone()))).style(move |_, _| button::Style { background: None, text_color: pal.text, ..Default::default() })) }
        if d.saetze.is_empty() { dokument = dokument.push(text("Keine Kachelsätze für dieses System im Projekt").size(11).color(pal.mute)) }
        if let (Some(ts), Some(sc)) = (d.ts.as_ref(), d.screen()) {
            dokument = dokument.push(kopf("Screen"));
            let (w, h, ox, oy) = (sc.w, sc.h, sc.origin.0, sc.origin.1);
            dokument = dokument.push(iced::widget::row![text(format!("Breite {w}")).size(11.5).color(pal.mute).width(80), tb("−", "schmaler", w > 1, BAkt::Groesse(w.saturating_sub(1), h, ox, oy, true), false), tb("+", "breiter", w < 255, BAkt::Groesse(w + 1, h, ox, oy, true), false), text(format!("Höhe {h}")).size(11.5).color(pal.mute).width(70), tb("−", "niedriger", h > 1, BAkt::Groesse(w, h.saturating_sub(1), ox, oy, true), false), tb("+", "höher", h < 255, BAkt::Groesse(w, h + 1, ox, oy, true), false)].spacing(4).align_y(Alignment::Center).wrap().vertical_spacing(4));
            dokument = dokument.push(iced::widget::row![text(format!("Ursprung {ox},{oy}")).size(11.5).color(pal.mute).width(110), tb("←", "Ursprung nach links", ox > 0, BAkt::Groesse(w, h, ox.saturating_sub(1), oy, true), false), tb("→", "Ursprung nach rechts", ox + 1 < w, BAkt::Groesse(w, h, ox + 1, oy, true), false), tb("↑", "Ursprung nach oben", oy > 0, BAkt::Groesse(w, h, ox, oy.saturating_sub(1), true), false), tb("↓", "Ursprung nach unten", oy + 1 < h, BAkt::Groesse(w, h, ox, oy + 1, true), false)].spacing(4).align_y(Alignment::Center).wrap().vertical_spacing(4));
            let (cols, rows) = grid_dims(sc, ts); dokument = dokument.push(text(format!("Kachelraster {cols}×{rows}{}", if w.saturating_sub(ox) % ts.tw > 0 || h.saturating_sub(oy) % ts.th > 0 || ox > 0 || oy > 0 { " · Restzellen in der Zellebene" } else { "" })).size(11).color(pal.mute));
            dokument = dokument.push(kopf("Hintergrund")).push(farbfeld(sc.colors["bg"].as_u64().map_or(6, |v| v as usize), Box::new(|i| BAkt::Farbe("bg", i))));
            if matches!(d.target.as_str(), "c64" | "vic20") { dokument = dokument.push(kopf("Rahmen")).push(farbfeld(sc.colors["border"].as_u64().map_or(14, |v| v as usize), Box::new(|i| BAkt::Farbe("border", i)))) }
            else if ts.color_mode == ColorMode::None { dokument = dokument.push(kopf("Vordergrund (Helligkeit)")).push(farbfeld(sc.colors["fg"].as_u64().map_or(14, |v| v as usize), Box::new(|i| BAkt::Farbe("fg", i)))) }
            if d.modus == Modus::Zelle {
                werkzeug = werkzeug.push(kopf("Zeichen (Zellebene)"));
                if ts.charsets.is_empty() { werkzeug = werkzeug.push(text("Der Kachelsatz hat keinen Zeichensatz.").size(11).color(pal.mute)) } else {
                    let mut tabs = iced::widget::row![].spacing(4); for (i, p) in ts.charsets.iter().enumerate() { tabs = tabs.push(tb(&format!("{} {}", (b'A' + (i % 26) as u8) as char, vc_media::naming::stem(p)), p, true, BAkt::Reiter(i), d.reiter == i)) } werkzeug = werkzeug.push(tabs.wrap());
                    let n = if d.target == "atari8" { 128 } else { 256 };
                    werkzeug = werkzeug.push(canvas::Canvas::new(Tafel { bild: handle(&d.tafel_bild(2, &rgb)), kw: 16.0, kh: 16.0, spalten: 16, anzahl: n, gewaehlt: (d.zell_pinsel.cs == d.reiter).then_some(d.zell_pinsel.code as usize), kachel: false, pal, rel: rel.to_string() }).width(256).height((n / 16) as f32 * 16.0));
                    werkzeug = werkzeug.push(text(format!("Zeichen {} (${:02X})", d.zell_pinsel.code, d.zell_pinsel.code)).size(11.5).color(pal.mute));
                    if ts.color_mode != ColorMode::None { werkzeug = werkzeug.push(kopf("Zeichenfarbe")).push(farbfeld(d.farbe, Box::new(BAkt::Pinselfarbe))) }
                    if d.target == "atari8" { let r2 = r.clone(); werkzeug = werkzeug.push(iced::widget::checkbox(d.zell_pinsel.sel).label("Farbwahl (Zeichenbit 7)").on_toggle(move |b| Msg::BankAkt(r2.clone(), BAkt::Bit7(b))).size(14).text_size(12)) }
                }
            }
            if !ts.prop_defs.is_empty() {
                dokument = dokument.push(kopf("Eigenschaften-Ebene")); let mut z = iced::widget::row![tb("aus", "Ebene ausblenden", true, BAkt::Eigenschaft(None), d.eigenschaft.is_none())].spacing(4);
                for (n, bit) in &ts.prop_defs { z = z.push(tb(n, "Kacheln mit dieser Eigenschaft einfärben", true, BAkt::Eigenschaft(Some(*bit)), d.eigenschaft == Some(*bit))) } dokument = dokument.push(z.wrap());
            }
        }
        dokument = dokument.push(text(self.palettenzeile(&d.target, s.ansicht)).size(10.5).color(pal.mute));
        if self.farben.hat_varianten(&d.target) { let r3 = r.clone(); dokument = dokument.push(self.ansicht_wahl_mit(pal, s.ansicht, move |a| Msg::BankAkt(r3.clone(), BAkt::Ansicht(a)))) }
        // Karten: Werkzeug (fürs Zeichnen) · Dokument (Kachelsatz, Screen, Farben, Eigenschaften-Ebene) · Info
        let karte_zu = |k: &'static str| -> Option<(bool, Msg)> { Some((!s.karten_zu.contains(k), Msg::BankAkt(r.clone(), BAkt::Karte(k)))) };
        if d.werkzeug == Werkzeug::Rechteck { werkzeug = werkzeug.push(tb("gefüllt", "Rechteck füllen", true, BAkt::Gefuellt(!d.gefuellt), d.gefuellt)) }
        if d.auswahl.is_some() {
            werkzeug = werkzeug.push(tb("Als Kachel anlegen …", "Die Zellen ab der linken oberen Ecke der Auswahl werden eine Kachel", d.ts.as_ref().is_some_and(|t| t.tiles.len() < vc_media::tiles::MAX_TILES), BAkt::AlsKachel, false));
            if d.modus == Modus::Kachel { werkzeug = werkzeug.push(tb("Kopieren (Stempel)", "Auswahl als Stempel merken", true, BAkt::AuswahlKopieren, false)) }
            werkzeug = werkzeug.push(tb("Leeren", "Auswahl leeren (Entf)", true, BAkt::AuswahlLeeren, false));
        }
        let at = s.ueber.and_then(|(fx, fy)| d.einheit_bei(fx, fy, false)).map(|u| match d.modus { Modus::Kachel => format!("Kachel {},{} → {}", u.0, u.1, d.screen().and_then(|sc| sc.grid.get(u.1)).and_then(|r| r.get(u.0)).map_or("-".into(), |t| t.to_string())), Modus::Zelle => format!("Zelle {},{}", u.0, u.1) }).unwrap_or_default();
        let info = iced::widget::column![text(d.screen().map_or(String::new(), |sc| format!("{}×{} Zellen · {} Patches", sc.w, sc.h, sc.patches.len()))).size(muster::schrift::TEXT).color(pal.text), text(if d.modus == Modus::Kachel { "Kachelmodus (Tab wechselt)" } else { "Zeichenmodus (Tab wechselt)" }).size(muster::schrift::HINWEIS).color(pal.mute)].spacing(muster::abstand::ENG);
        let karten: Vec<Element<Msg>> = vec![
            muster::karte(pal, "Werkzeug", "✎", "fürs Zeichnen · nicht in der Datei", karte_zu("werkzeug"), werkzeug.into()),
            muster::karte(pal, "Dokument", "▤", "Kachelsatz, Screen, Farben · wird gespeichert", karte_zu("dokument"), dokument.into()),
            muster::karte(pal, "Info", "ⓘ", "", karte_zu("info"), info.into())];
        let seite = muster::seitenspalte(klasse, karten);
        // Zone 1: Kopfzeile
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let kopf_z = muster::kopfzeile(pal, klasse, "BNK", &name, d.dirty(), &d.screen().map_or(String::new(), |sc| format!("Screen {} von {} · {}×{} Zellen", d.cur + 1, d.bank.screens.len(), sc.w, sc.h)), vec![
            tb("Exportieren …", "Export-Dialog (Cmd+Umschalt+E)", true, BAkt::Exportieren, false), tb("Speichern", "Speichern (Cmd+S)", d.dirty(), BAkt::Speichern, false)]);
        let status = if d.status.is_empty() { String::new() } else { format!(" · {}", d.status) };
        let fuss = muster::fusszeile(pal, &format!("{at}{}Zoom {}{status}", if at.is_empty() { "" } else { " · " }, if s.zoomansicht.platz.get().0 < 1.0 { "Einpassen".to_string() } else { format!("{}{}×", if s.zoomansicht.zoom.is_none() { "Einpassen · " } else { "" }, s.skala.get()) }),
            iced::widget::tooltip(text("? Tasten").size(muster::schrift::HINWEIS).color(pal.mute), container(text("Tab wechselt den Modus · D L R F I A S Werkzeuge · Text: tippen, Entf/Rücktaste, Enter, Esc · Esc bricht Linie/Rechteck ab · + − Zoom · 0 Einpassen · Leertaste+Ziehen verschiebt · Cmd+Mausrad zoomt").size(11.5).color(pal.text)).padding([4, 8]).max_width(420).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Top).into());
        // Überlagerungen über der Fläche: Dialog „Neue Kachel“ und ⋯-Karte
        let mut flaeche_el = flaeche_roh;
        if let Some(dlg) = nk_dialog { flaeche_el = iced::widget::stack![flaeche_el, container(dlg).padding(muster::abstand::BEREICH).center(Length::Fill)].into(); }
        if s.mehr {
            let karte_mehr = container(iced::widget::column![if kompakt { Element::from(iced::widget::column![text("Werkzeuge").size(muster::schrift::HINWEIS).color(pal.mute), werkzeuge().wrap().vertical_spacing(4)].spacing(muster::abstand::ENG)) } else { space::vertical().height(0).into() }, text("Ansicht").size(muster::schrift::HINWEIS).color(pal.mute), ansicht().wrap().vertical_spacing(4)].spacing(muster::abstand::GRUPPE)).padding(muster::abstand::KARTE).max_width(360).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: muster::RADIUS_KARTE.into() }, ..Default::default() });
            flaeche_el = iced::widget::stack![flaeche_el, container(karte_mehr).padding(muster::abstand::GRUPPE).align_right(Length::Fill)].into();
        }
        let flaeche_el: Element<Msg> = container(flaeche_el).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        let _ = breite;
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: 200.0, kopf: kopf_z, leiste, liste: Some(liste_el), flaeche: flaeche_el, seite, fuss })
    }
}

/// Einzeltaste eines Werkzeugs im Bank-Studio (für Tooltip und Befehlspalette); Quelle: `bank_taste`
pub fn werkzeug_taste(w: Werkzeug) -> Option<&'static str> {
    Some(match w { Werkzeug::Stift => "D", Werkzeug::Linie => "L", Werkzeug::Rechteck => "R", Werkzeug::Fuellen => "F", Werkzeug::Pipette => "I", Werkzeug::Auswahl => "A", Werkzeug::Loeschen => "E", Werkzeug::Text => "T", Werkzeug::Stempel => "S" })
}
