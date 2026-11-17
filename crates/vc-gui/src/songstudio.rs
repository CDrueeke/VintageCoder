// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker-Studio (`.vcsong`): Muster-Raster mit Zellen „NOT II“, Noteneingabe per Klaviertasten, Auswahl und Block, Oktave und Instrument,
//! Muster und Reihenfolge, Tempo und Zeilenzahl. Das Modell ist `vc_ide_model::songdoc::SongDoc`; hier wird nur angezeigt und weitergeleitet.
use super::*;
use iced::widget::{canvas, text};
use serde_json::json;
use vc_ide_model::songdoc::{klaviertaste, Eingabe, SongDoc};
use vc_gui_ui::muster;

/// Dialog „Song aus anderem System übernehmen“ bzw. „RMT-Modul importieren“
pub struct SongImport { pub rel: String, pub rmt: bool, pub datei: PathBuf, pub name: String, pub quelle: vc_ide_model::songimport::Quelle, pub ziel: String, pub subtune: u16, pub sekunden: u32, pub fehler: String, pub laeuft: bool }

pub struct SongStudio { pub doc: SongDoc, pub unten: Option<(usize, usize)>, /** Wiedergabe: (Eintrag der Reihenfolge, Zeile) */ pub spiel: Option<(usize, usize)>, /** Pegel je Kanal und gesamt (0..1) */ pub pegel: (Vec<f32>, f32), /** Regler/Namensfeld in Bearbeitung: ein Rückgängig-Schritt je Zug */ pub zieht: bool, /** sichtbarer Ausschnitt des Rasters: (Versatz x, Versatz y, Breite, Höhe) */ pub ansicht: (f32, f32, f32, f32), /** „Nur Muster“ spielt: dieses Muster (die Position der Wiedergabe zählt dann im Ein-Eintrag-Ablauf, nicht in der Reihenfolge des Songs) */ pub spiel_muster: Option<usize>, /** Block- und Musterleisten eingeklappt (None: automatisch nach der Fensterhöhe) */ pub leisten_zu: Option<bool>, /** Modus des Studios (Entscheidung 25, Variante C) */ pub modus: SongModus, pub karten_zu: std::collections::BTreeSet<&'static str>, pub mehr: bool, /** Raster scrollt beim Abspielen mit (W2) */ pub mitlaufen: bool, /** zuletzt programmatisch angesteuerte Scrollhöhe (zur Unterscheidung vom Handscrollen) */ pub scroll_ziel: Option<f32>, /** Prüfsumme des Songs bei der letzten Übergabe an die Wiedergabe (W7) */ pub live_stand: u64, /** Breite einer Kanalspalte im Raster (elastisch, wird beim Zeichnen der Ansicht gesetzt) */ pub kb: std::cell::Cell<f32>, /** gerade klingende Noten (Wiedergabe je Kanal und Vorhörton): nur Anzeige auf der Klaviatur */ pub klingt: Vec<u8>, /** im Raster ausgeblendete Kanäle (nur Ansicht: sie spielen weiter, die Datei bleibt unverändert) */ pub aus: std::collections::BTreeSet<usize> }
impl SongStudio {
    /// Kanäle, die im Raster stehen, in Reihenfolge
    pub fn sichtbar(&self) -> Vec<usize> { (0..self.doc.kanaele).filter(|c| !self.aus.contains(c)).collect() }
    /// Nächster sichtbarer Kanal ab `c` in Richtung `d` (+1/−1, rundet um); `c` selbst, wenn es keinen anderen gibt
    pub fn sichtbar_naechster(&self, c: usize, d: i32) -> usize { let n = self.doc.kanaele.max(1) as i32; (1..=n).map(|k| (c as i32 + d * k).rem_euclid(n) as usize).find(|x| !self.aus.contains(x)).unwrap_or(c) }
    /// Kanal im Raster zeigen oder ausblenden; der letzte sichtbare Kanal bleibt; ein Cursor im ausgeblendeten Kanal springt zum nächsten sichtbaren
    pub fn sichtbar_umschalten(&mut self, c: usize) {
        if self.aus.remove(&c) { return }
        if self.sichtbar().len() <= 1 || c >= self.doc.kanaele { return }
        self.aus.insert(c); let r = self.doc.cursor.0; self.doc.springe(r); /* Auswahl und Anker verwerfen */
        if self.doc.cursor.1 == c { self.doc.cursor.1 = self.sichtbar_naechster(c, 1) }
    }
    /// Cursor um einen sichtbaren Kanal bewegen (ausgeblendete werden übersprungen)
    pub fn bewege_sichtbar(&mut self, dr: i32, dc: i32, erweitern: bool) {
        let dc = if dc == 0 { 0 } else { let c = self.doc.cursor.1; self.sichtbar_naechster(c, dc.signum()) as i32 - c as i32 };
        self.doc.bewege(dr, dc, erweitern)
    }
}

/// Die drei Modi des Tracker-Studios: Noten eingeben, Instrumente bearbeiten, Ablauf (Reihenfolge und Muster) gestalten
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SongModus { #[default] Noten, Instrumente, Ablauf }

#[derive(Clone, Debug, PartialEq)]
pub enum GAkt { Sichtbar(usize), AlleEinblenden, Eintragen(u8), Mitlaufen(bool), Vorhoeren(usize), Probe(u8), Modus(SongModus), Karte(&'static str), Mehr, Leisten, Undo, Redo, Speichern, Exportieren, Oktave(i32), Instrument(i32), Muster(usize), Tempo(i32), Zeilen(i32), Kopieren, Ausschneiden, Einfuegen, Transponieren(i32), Loeschen, InstrumentAnwenden, Schritt(i32),
    Ord(usize), OrdPlus, OrdMinus, OrdSetzen, OrdVerschieben(i32), Schleife(i32), MusterNeu, MusterDupl, MusterLoeschen, InsWahl(usize), InsNeu, InsDupl, InsLoeschen, InsName(String), InsWelle(String), InsDist(String), InsZahl(&'static str, i64), Huell(usize, u8), ReglerLos, Stumm(usize), Solo(usize), SpielSong, SpielMuster, Stopp, Lautstaerke(i32), Konvertieren, RmtImport }
#[derive(Clone, Debug, PartialEq)]
pub enum GMaus { Unten { r: usize, c: usize }, Zug { r: usize, c: usize }, Los }

/// Alle Tasten des Tracker-Fokus als Kürzel im Schreibweise der Befehlstabelle (ohne Cmd/Strg/Alt gelten sie nur, wenn der Tracker den Fokus hat);
/// ein Test sorgt dafür, dass keine davon in der Befehlstabelle vorkommt
pub const TRACKER_TASTEN: &[&str] = &["Z", "S", "X", "D", "C", "V", "G", "B", "H", "N", "J", "M", "Q", "2", "W", "3", "E", "R", "5", "T", "6", "Y", "7", "U", "I", "1", ",", ".", "[", "]", "Space", "Umschalt+Space", "Entf", "Rückschritt", "Links", "Rechts", "Oben", "Unten", "Umschalt+Links", "Umschalt+Rechts", "Umschalt+Oben", "Umschalt+Unten", "Bild auf", "Bild ab", "Pos1", "Ende"];
/// Mit Cmd gehören diese dem Tracker (Block-Operationen); alle anderen Cmd-Kürzel bleiben global
pub const TRACKER_CMD_TASTEN: &[&str] = &["Cmd+C", "Cmd+X", "Cmd+V", "Cmd+A"];

const ZH: f32 = 20.0; const NB: f32 = 34.0; const KB: f32 = 70.0; const KOPF: f32 = 24.0;

/// Das Raster: Kopfzeile mit Kanälen, Zeilennummern, Zellen, Cursor, Auswahl, Abspielzeile
struct Raster<'a> { d: &'a SongDoc, pal: Pal, rel: String, spiel: Option<usize>, versatz: (f32, f32), /** Breite einer Kanalspalte */ kb: f32, /** Breite der ganzen Zeichenfläche (mindestens die sichtbare Breite: Zeilenband läuft durch) */ breite: f32, /** Kanäle im Raster (ausgeblendete fehlen) */ sichtbar: Vec<usize> }
impl<'a> Raster<'a> {
    /// „Kanal 1 · Puls“: der Name des Instruments der ersten Note im Kanal (leer, wenn keine); in schmalen Spalten nur die Nummer
    fn kanal_kopf(&self, c: usize) -> String {
        if self.kb < 100.0 { return format!("{}", c + 1) }
        let ins = (0..self.d.zeilen()).find_map(|r| { let z = self.d.zelle(self.d.muster, r, c); if z.starts_with("---") || z.starts_with("===") { None } else { z.rsplit(' ').next().and_then(|h| usize::from_str_radix(h, 16).ok()) } });
        match ins { Some(i) => format!("Kanal {} · {}", c + 1, self.d.instrument_name(i)), None => format!("Kanal {}", c + 1) }
    }
    fn zelle_bei(&self, p: iced::Point) -> Option<(usize, usize)> {
        let (x, y) = (p.x - NB, p.y - KOPF); if x < 0.0 || y < 0.0 { return None }
        let (pos, r) = ((x / self.kb) as usize, (y / ZH) as usize); let c = *self.sichtbar.get(pos)?; (r < self.d.zeilen()).then_some((r, c))
    }
}
impl<'a> canvas::Program<Msg> for Raster<'a> {
    type State = bool;
    fn update(&self, st: &mut bool, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        use iced::mouse::{Button, Event as M};
        let rel = self.rel.clone(); let send = |m: GMaus| Some(canvas::Action::publish(Msg::SongMaus(rel.clone(), m)).and_capture());
        match e {
            iced::Event::Mouse(M::ButtonPressed(Button::Right)) => { let p = c.position()?; c.position_in(b)?; Some(canvas::Action::publish(Msg::KontextAuf(crate::kontext::Art::Flaeche, p, Some(self.rel.clone()))).and_capture()) }
            iced::Event::Mouse(M::ButtonPressed(Button::Left)) => { let (r, k) = self.zelle_bei(c.position_in(b)?)?; *st = true; send(GMaus::Unten { r, c: k }) }
            iced::Event::Mouse(M::CursorMoved { .. }) if *st => { let p = c.position().map(|p| iced::Point::new(p.x - b.x, p.y - b.y))?; let (r, k) = self.zelle_bei(p)?; send(GMaus::Zug { r, c: k }) }
            iced::Event::Mouse(M::ButtonReleased(Button::Left)) if *st => { *st = false; send(GMaus::Los) }
            _ => None,
        }
    }
    fn draw(&self, _st: &bool, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let (d, pal, kb) = (self.d, self.pal, self.kb); let mono = vc_gui_ui::fonts::MONO;
        let breite = self.breite.max(NB + kb * self.sichtbar.len() as f32);
        let txt = |f: &mut canvas::Frame, s: String, x: f32, y: f32, farbe: Color, ausrichtung: iced::alignment::Horizontal, font: iced::Font| f.fill_text(canvas::Text { content: s, position: iced::Point::new(x, y), color: farbe, size: iced::Pixels(13.0), font, align_x: ausrichtung.into(), align_y: iced::alignment::Vertical::Center, ..Default::default() });
        let zeilennr = |rr: usize| format!("{rr:02}");
        for rr in 0..d.zeilen() {
            let y = KOPF + rr as f32 * ZH;
            // Zeilenband über die ganze Breite: Taktband alle vier Zeilen, Cursorzeile leicht in Akzent, gespielte Zeile kräftiger
            if rr % 4 == 0 { f.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(breite, ZH), pal.panel2) }
            if d.cursor.0 == rr { f.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(breite, ZH), Color { a: 0.14, ..pal.accent }) }
            if self.spiel == Some(rr) { f.fill_rectangle(iced::Point::new(0.0, y), iced::Size::new(breite, ZH), Color { a: 0.30, ..pal.accent }); txt(&mut f, "▶".into(), 4.0, y + ZH / 2.0, pal.accent, iced::alignment::Horizontal::Left, mono) }
            let aktiv = d.cursor.0 == rr || self.spiel == Some(rr);
            txt(&mut f, zeilennr(rr), NB - 6.0, y + ZH / 2.0, if aktiv { pal.accent } else if rr % 4 == 0 { pal.mute } else { Color { a: 0.6, ..pal.mute } }, iced::alignment::Horizontal::Right, if aktiv { vc_gui_ui::fonts::MONO_FETT } else { mono });
            for (pos, &c) in self.sichtbar.iter().enumerate() {
                let x = NB + pos as f32 * kb; let t = d.zelle(d.muster, rr, c);
                let in_auswahl = d.auswahl.is_some_and(|((r0, c0), (r1, c1))| rr >= r0 && rr <= r1 && c >= c0 && c <= c1);
                if in_auswahl { f.fill_rectangle(iced::Point::new(x + 1.0, y), iced::Size::new(kb - 2.0, ZH), Color { a: 0.35, ..pal.warn }) }
                let hier = d.cursor == (rr, c);
                if hier { f.fill_rectangle(iced::Point::new(x + 4.0, y + 1.0), iced::Size::new(94.0f32.min(kb - 8.0), ZH - 2.0), pal.accent) }
                let (leer, ende) = (t.starts_with("---"), t.starts_with("==="));
                let farbe = if hier { pal.accent_text() } else if ende { pal.warn } else if leer { Color { a: 0.45, ..pal.mute } } else { pal.text };
                txt(&mut f, t, x + 4.0 + 94.0f32.min(kb - 8.0) / 2.0, y + ZH / 2.0, farbe, iced::alignment::Horizontal::Center, if hier { vc_gui_ui::fonts::MONO_FETT } else { mono });
                if pos > 0 { f.stroke(&canvas::Path::line(iced::Point::new(x, y), iced::Point::new(x, y + ZH)), canvas::Stroke::default().with_color(pal.line).with_width(1.0)) }
            }
        }
        // Kanalköpfe und Zeilennummern bleiben beim Scrollen am Rand sichtbar (kleben an der oberen und linken Kante des sichtbaren Ausschnitts)
        let (vx, vy) = self.versatz; let hintergrund = Color { a: 1.0, ..pal.editor };
        f.fill_rectangle(iced::Point::new(vx, vy), iced::Size::new(b.width.max(breite), KOPF), hintergrund);
        for (pos, &c) in self.sichtbar.iter().enumerate() { txt(&mut f, self.kanal_kopf(c), NB + pos as f32 * kb + kb / 2.0, vy + KOPF / 2.0, pal.mute, iced::alignment::Horizontal::Center, vc_gui_ui::fonts::UI) }
        f.stroke(&canvas::Path::line(iced::Point::new(vx, vy + KOPF), iced::Point::new(vx + b.width.max(breite), vy + KOPF)), canvas::Stroke::default().with_color(pal.line).with_width(1.0));
        if vx > 0.5 {
            f.fill_rectangle(iced::Point::new(vx, vy + KOPF), iced::Size::new(NB, b.height.max(KOPF + ZH * d.zeilen() as f32)), hintergrund);
            for rr in 0..d.zeilen() { let y = KOPF + rr as f32 * ZH; txt(&mut f, zeilennr(rr), vx + NB - 6.0, y + ZH / 2.0, if rr % 4 == 0 { pal.mute } else { Color { a: 0.6, ..pal.mute } }, iced::alignment::Horizontal::Right, mono) }
            f.fill_rectangle(iced::Point::new(vx, vy), iced::Size::new(NB, KOPF), hintergrund);
            f.stroke(&canvas::Path::line(iced::Point::new(vx + NB, vy), iced::Point::new(vx + NB, vy + b.height.max(breite))), canvas::Stroke::default().with_color(pal.line).with_width(1.0));
        }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &bool, _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Crosshair }
}

pub const RASTER_ID: &str = "song-raster";

impl App {
    /// Sichtbaren Ausschnitt merken (Meldung des Rasters beim Scrollen)
    pub(crate) fn song_scroll(&mut self, rel: &str, x: f32, y: f32, w: f32, h: f32) {
        let spielt = self.audio_song.as_deref() == Some(rel);
        if let Some(s) = self.songs.get_mut(rel) {
            // Handscrollen während der Wiedergabe beendet das Mitlaufen (W2); die vom Mitlaufen selbst ausgelöste Bewegung ist das Ziel `scroll_ziel`
            if spielt && s.mitlaufen { match s.scroll_ziel { Some(t) if (y - t).abs() > 4.0 => { s.mitlaufen = false; s.scroll_ziel = None } Some(_) => s.scroll_ziel = None, None => { if (y - s.ansicht.1).abs() > 4.0 && s.ansicht.3 > 0.0 { s.mitlaufen = false } } } }
            s.ansicht = (x, y, w, h)
        }
    }
    /// Cursor sichtbar halten: liegt die Cursorzelle außerhalb des Ausschnitts, wird minimal nachgescrollt (Pfeiltasten, Tab, Eingabe, Wiederholung)
    pub(crate) fn song_nachfuehren(&mut self) -> Task<Msg> {
        let Some(rel) = self.name().filter(|n| self.songs.contains_key(n)) else { return Task::none() };
        let Some(s) = self.songs.get_mut(&rel) else { return Task::none() }; let (ox, oy, w, h) = s.ansicht; if w < 1.0 || h < 1.0 { return Task::none() }
        let (r, c) = s.doc.cursor; let c = s.sichtbar().iter().position(|x| *x == c).unwrap_or(0); let kb = s.kb.get(); let (x0, x1) = (NB + c as f32 * kb, NB + (c + 1) as f32 * kb); let (y0, y1) = (KOPF + r as f32 * ZH, KOPF + (r + 1) as f32 * ZH);
        // die klebenden Ränder (Zeilennummern links, Kanalköpfe oben) verdecken Zellen: sichtbar ist der Bereich dahinter
        let nx = if x0 < ox + NB { Some((x0 - NB).max(0.0)) } else if x1 > ox + w { Some(x1 - w) } else { None };
        let ny = if y0 < oy + KOPF { Some((y0 - KOPF).max(0.0)) } else if y1 > oy + h { Some(y1 - h) } else { None };
        if nx.is_none() && ny.is_none() { return Task::none() }
        if let Some(x) = nx { s.ansicht.0 = x } if let Some(y) = ny { s.ansicht.1 = y }
        iced::widget::operation::scroll_to(iced::widget::Id::new(RASTER_ID), iced::widget::scrollable::AbsoluteOffset { x: nx, y: ny })
    }
}

/// Klaviatur zum Probespielen des gewählten Instruments (zwei Oktaven ab der Oktave der Eingabe; Klick spielt die Note, trägt nichts ein)
struct Klaviatur { oktave: usize, pal: Pal, rel: String, /** klingende Noten (absolut, wie im Raster) */ klingt: Vec<u8>, /** Computertaste je Halbton (0–24) in der Beschriftung des Layouts; leer = nicht anzeigen */ tasten: Vec<String> }
impl Klaviatur {
    /// Weiße Tasten der zwei Oktaven: (Halbton über C, Position als Zahl der weißen Tasten davor)
    const WEISS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
    /// Schwarze Tasten: (Halbton, Position zwischen den weißen in Taste-Einheiten)
    const SCHWARZ: [(u8, f32); 5] = [(1, 1.0), (3, 2.0), (6, 4.0), (8, 5.0), (10, 6.0)];
    fn geo(&self, b: iced::Size) -> (f32, f32) { let w = ((b.width - 8.0) / 14.0).floor().clamp(14.0, 44.0); (w, b.height - 8.0) }
    fn note_bei(&self, b: iced::Size, p: iced::Point) -> Option<u8> {
        let (kw, kh) = self.geo(b); let (x, y) = (p.x - 4.0, p.y - 4.0); if x < 0.0 || y < 0.0 || y > kh || x >= 14.0 * kw { return None }
        // schwarze Tasten zuerst (obere 60 % der Höhe)
        if y < kh * 0.6 { for o in 0..2 { for (h, pos) in Self::SCHWARZ { let cx = (o as f32 * 7.0 + pos) * kw; if (x - cx).abs() <= kw * 0.3 { return Some((self.oktave as u32 * 12 + (o as u32 * 12 + h as u32) + 1).min(254) as u8) } } } }
        let (o, w) = ((x / kw) as usize / 7, (x / kw) as usize % 7); Some((self.oktave as u32 * 12 + (o as u32 * 12 + Self::WEISS[w] as u32) + 1).min(254) as u8)
    }
}
impl canvas::Program<Msg> for Klaviatur {
    type State = ();
    fn update(&self, _st: &mut (), e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = e { let n = self.note_bei(b.size(), c.position_in(b)?)?; return Some(canvas::Action::publish(Msg::SongAkt(self.rel.clone(), GAkt::Eintragen(n))).and_capture()) }
        None
    }
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let (kw, kh) = self.geo(b.size()); let pal = self.pal;
        let von = self.oktave as i32 * 12 + 1; // Note der ersten Taste (C der Eingabeoktave)
        let klingt = |h: i32| self.klingt.iter().any(|n| *n as i32 == von + h);
        let akzent = pal.accent; let weiss = if pal.dark { Color::from_rgb8(0xe6, 0xe8, 0xec) } else { Color::WHITE };
        for i in 0..14 { let x = 4.0 + i as f32 * kw; let h = (i / 7) as i32 * 12 + Self::WEISS[i % 7] as i32; let an = klingt(h);
            f.fill_rectangle(iced::Point::new(x, 4.0), iced::Size::new(kw - 1.0, kh), if an { Color { a: 0.9, ..akzent } } else { weiss });
            // klingend: kräftiger Rahmen (nicht nur Farbe) und Punkt unten
            f.stroke_rectangle(iced::Point::new(x, 4.0), iced::Size::new(kw - 1.0, kh), canvas::Stroke::default().with_color(if an { akzent } else { Color { a: 0.7, ..pal.mute } }).with_width(if an { 3.0 } else { 1.0 }));
            if an { f.fill(&canvas::Path::circle(iced::Point::new(x + (kw - 1.0) / 2.0, kh - 4.0), 3.5), pal.accent_text()) }
            if let Some(t) = self.tasten.get(h as usize) { f.fill_text(canvas::Text { content: t.clone(), position: iced::Point::new(x + (kw - 1.0) / 2.0, kh - 22.0), color: if an { pal.accent_text() } else { Color::from_rgb8(0x5d, 0x66, 0x75) }, size: iced::Pixels(9.5), align_x: iced::alignment::Horizontal::Center.into(), ..Default::default() }) }
            if i % 7 == 0 { f.fill_text(canvas::Text { content: format!("C{}", self.oktave + i / 7), position: iced::Point::new(x + 3.0, kh - 12.0), color: if an { pal.accent_text() } else { Color::from_rgb8(0x5d, 0x66, 0x75) }, size: iced::Pixels(10.0), ..Default::default() }) } }
        for o in 0..2 { for (hh, pos) in Self::SCHWARZ { let cx = 4.0 + (o as f32 * 7.0 + pos) * kw; let h = o as i32 * 12 + hh as i32; let an = klingt(h);
            f.fill_rectangle(iced::Point::new(cx - kw * 0.3, 4.0), iced::Size::new(kw * 0.6, kh * 0.6), if an { akzent } else { Color::from_rgb8(0x14, 0x16, 0x1a) });
            if let Some(t) = self.tasten.get(h as usize) { f.fill_text(canvas::Text { content: t.clone(), position: iced::Point::new(cx, 4.0 + kh * 0.6 - 12.0), color: Color::from_rgb8(0xe6, 0xe8, 0xec), size: iced::Pixels(9.0), align_x: iced::alignment::Horizontal::Center.into(), ..Default::default() }) }
            if an { f.stroke_rectangle(iced::Point::new(cx - kw * 0.3, 4.0), iced::Size::new(kw * 0.6, kh * 0.6), canvas::Stroke::default().with_color(pal.text).with_width(2.0)) } } }
        // klingende Noten außerhalb der gezeigten zwei Oktaven: Pfeil am Rand
        let (tiefer, hoeher) = (self.klingt.iter().any(|n| (*n as i32) < von), self.klingt.iter().any(|n| (*n as i32) >= von + 24));
        let pfeil = |f: &mut canvas::Frame, t: &str, x: f32, a: iced::alignment::Horizontal| f.fill_text(canvas::Text { content: t.into(), position: iced::Point::new(x, 14.0), color: akzent, size: iced::Pixels(14.0), align_x: a.into(), ..Default::default() });
        if tiefer { pfeil(&mut f, "◂", 6.0, iced::alignment::Horizontal::Left) } if hoeher { pfeil(&mut f, "▸", b.width - 6.0, iced::alignment::Horizontal::Right) }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(&self, _st: &(), _b: iced::Rectangle, _c: iced::mouse::Cursor) -> iced::mouse::Interaction { iced::mouse::Interaction::Pointer }
}

/// Hüllkurve des gewählten Instruments (SID: ADSR-Verlauf, POKEY: 16 Lautstärkebalken mit Wiederholungsstelle)
struct Huellkurve { ins: vc_ide_model::synth::Instr, pal: Pal }
impl canvas::Program<Msg> for Huellkurve {
    type State = ();
    fn draw(&self, _st: &(), r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        use vc_ide_model::synth::{Instr, SID_A, SID_D};
        let mut f = canvas::Frame::new(r, b.size()); let (w, h) = (b.width, b.height);
        f.fill_rectangle(iced::Point::ORIGIN, iced::Size::new(w, h), self.pal.editor); f.stroke_rectangle(iced::Point::ORIGIN, iced::Size::new(w, h), canvas::Stroke::default().with_color(self.pal.line).with_width(1.0));
        let linie = canvas::Stroke::default().with_color(self.pal.accent).with_width(2.0);
        match &self.ins {
            Instr::Sid { a, d, s, r: rel, .. } => {
                let (ta, td, tr) = (SID_A[*a].max(2.0), SID_D[*d].max(6.0), SID_D[*rel].max(6.0)); let total = ta + td * 1.2 + 120.0 + tr;
                let x = |t: f32| 6.0 + t / total * (w - 12.0); let y = |l: f32| h - 6.0 - l * (h - 12.0); let sus = *s as f32 / 15.0;
                let pts = [(0.0, 0.0), (ta, 1.0), (ta + td, sus), (ta + td * 1.2 + 120.0, sus), (total, 0.0)];
                let pfad = canvas::Path::new(|p| { p.move_to(iced::Point::new(x(pts[0].0), y(pts[0].1))); for q in &pts[1..] { p.line_to(iced::Point::new(x(q.0), y(q.1))) } });
                f.fill(&canvas::Path::new(|p| { p.move_to(iced::Point::new(x(0.0), y(0.0))); for q in &pts { p.line_to(iced::Point::new(x(q.0), y(q.1))) } p.line_to(iced::Point::new(x(total), y(0.0))); p.close() }), Color { a: 0.2, ..self.pal.accent });
                f.stroke(&pfad, linie);
                for (tx, n) in [(x(ta / 2.0), "A"), (x(ta + td / 2.0), "D"), (x(ta + td * 1.2 + 60.0), "S"), (x(total - tr / 2.0), "R")] { f.fill_text(canvas::Text { content: n.into(), position: iced::Point::new(tx, 9.0), color: self.pal.mute, size: iced::Pixels(10.0), align_x: iced::alignment::Horizontal::Center.into(), align_y: iced::alignment::Vertical::Center, ..Default::default() }) }
            }
            Instr::Pokey { env, loop_at, .. } => {
                let bw = (w - 12.0) / 16.0;
                for (i, v) in env.iter().take(16).enumerate() { let hh = *v as f32 / 15.0 * (h - 22.0); f.fill_rectangle(iced::Point::new(6.0 + i as f32 * bw, h - 6.0 - hh), iced::Size::new(bw - 1.5, hh), if i >= *loop_at { self.pal.accent } else { Color { a: 0.55, ..self.pal.accent } }) }
                if *loop_at < 16 { let x = 6.0 + *loop_at as f32 * bw - 0.5; f.stroke(&canvas::Path::line(iced::Point::new(x, 4.0), iced::Point::new(x, h - 4.0)), canvas::Stroke::default().with_color(self.pal.warn).with_width(1.0)); f.fill_text(canvas::Text { content: "↺ ab hier".into(), position: iced::Point::new(x + 3.0, 8.0), color: self.pal.warn, size: iced::Pixels(10.0), ..Default::default() }) }
            }
        }
        vec![f.into_geometry()]
    }
}

impl App {
    /// Dialog öffnen: Quelle prüfen, Name vorschlagen; Fehler (unlesbare SID-Datei) gehen ins Protokoll
    pub(crate) fn song_import_oeffnen(&mut self, rel: &str, rmt: bool, datei: PathBuf) {
        let Some(s) = self.songs.get(rel) else { return };
        let quelle = if rmt { Ok(vc_ide_model::songimport::Quelle { art: "RMT-Modul (Raster Music Tracker)".into(), name: vc_media::naming::stem(&datei.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()).replace(' ', "_"), sid: None }) } else { vc_ide_model::songimport::quelle_pruefen(&datei) };
        match quelle {
            Ok(q) => { let ziel = if rmt || s.doc.pokey { "POKEY (Atari)".to_string() } else { "SID (C64)".to_string() }; let sub = q.sid.as_ref().map_or(0, |(_, _, start)| (*start).max(1)); self.song_import = Some(SongImport { rel: rel.into(), rmt, datei, name: q.name.clone(), quelle: q, ziel, subtune: sub, sekunden: 30, fehler: String::new(), laeuft: false }); self.ed_aktiv = false }
            Err(e) => { self.log.push(e); self.schublade_zeigen() }
        }
    }
    /// Auftrag für den Hintergrund: die Umwandlung (SID-Aufnahme kann dauern) ohne Zugriff auf `self`
    pub(crate) fn song_import_job(&self) -> Option<impl FnOnce() -> Result<(String, Vec<String>), String> + Send + 'static> {
        let d = self.song_import.as_ref()?; let s = self.songs.get(&d.rel)?;
        let (root, data, datei, name, rmt) = (self.root.clone(), self.data.clone(), d.datei.clone(), d.name.trim().to_string(), d.rmt);
        let ordner = d.rel.rsplit_once('/').map(|x| x.0.to_string()).filter(|o| !o.is_empty()).unwrap_or_else(|| "music".into());
        let (profil, sub, sek) = (s.doc.media["profile"].as_str().unwrap_or("").to_string(), d.subtune, d.sekunden);
        Some(move || if rmt { vc_ide_model::songimport::rmt_importieren(&root, &data, &datei, &name, &ordner) } else { vc_ide_model::songimport::konvertieren(&root, &data, &datei, &profil, &name, &ordner, sub, sek) })
    }
    pub(crate) fn song_import_fertig(&mut self, r: Result<(String, Vec<String>), String>) {
        match r {
            Ok((rel, warn)) => { self.song_import = None; for w in warn { self.log.push(format!("Hinweis: {w}")) } self.log.push(format!("Angelegt: {rel}")); self.projekt_neu_laden(); self.oeffne(&rel) }
            Err(e) => { if let Some(d) = &mut self.song_import { d.fehler = e.clone(); d.laeuft = false } self.log.push(e) }
        }
    }
    pub(crate) fn song_import_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.song_import.as_ref()?; let mono = vc_gui_ui::fonts::MONO;
        let klein = |s: &str, m: Msg| -> Element<Msg> { button(text(s.to_string()).size(12).color(pal.text)).padding([2, 8]).on_press(m).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }).into() };
        let mut c = iced::widget::column![text(if d.rmt { "RMT-Modul importieren" } else { "Song aus anderem System übernehmen" }).size(16).color(pal.text), iced::widget::row![text("Neuer Name").size(12).color(pal.mute), text_input("name", &d.name).on_input(Msg::SongImpName).on_submit(Msg::SongImpOk).size(12).padding(4).width(220).font(mono)].spacing(8).align_y(Alignment::Center),
            text(format!("Quelle: {}", d.datei.display())).size(11.5).color(pal.mute), text(format!("{}  →  {}", d.quelle.art, d.ziel)).size(12).color(pal.text)].spacing(8);
        if let Some((txt, songs, _)) = &d.quelle.sid {
            c = c.push(text(txt.clone()).size(12).font(vc_gui_ui::fonts::UI_FETT).color(pal.text)).push(iced::widget::row![text(format!("Subtune {} von {songs}", d.subtune)).size(12).color(pal.text), klein("−", Msg::SongImpSub(-1)), klein("+", Msg::SongImpSub(1)), text(format!("Länge {} s", d.sekunden)).size(12).color(pal.text), klein("−5", Msg::SongImpSek(-5)), klein("+5", Msg::SongImpSek(5)), klein("+30", Msg::SongImpSek(30))].spacing(8).align_y(Alignment::Center))
                .push(text("Die Spielroutine läuft in einem 6502-Kern; aufgezeichnet werden Gate, Tonhöhe, Wellenform, ADSR und Pulsbreite. Vibrato, Filter und Effekte entfallen.").size(11).color(pal.mute));
        }
        c = c.push(text(if d.rmt { "RMT4, Mono, mit Ladeadresse. Effekte, Tabellen und Lautstärken werden nicht übernommen." } else { "Der Song wird als neue Datei im Projekt angelegt. Noten und Struktur bleiben; Instrumente werden angenähert." }).size(11).color(pal.mute));
        if !d.fehler.is_empty() { c = c.push(text(d.fehler.clone()).size(12).color(pal.err)) }
        if d.laeuft { c = c.push(text("läuft …").size(12).color(pal.ok)) }
        c = c.push(iced::widget::row![knopf(pal, "OK", (!d.laeuft && !d.name.trim().is_empty()).then_some(Msg::SongImpOk), pal.text, true), knopf(pal, "Abbrechen", Some(Msg::SongImpAbbruch), pal.text, false)].spacing(8));
        let karte = container(c).width(620).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() }));
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }

    pub(crate) fn song_oeffnen(&mut self, rel: &str) {
        if !self.songs.contains_key(rel) {
            match SongDoc::oeffnen(&self.root, rel, &self.data) { Ok(doc) => { self.songs.insert(rel.to_string(), SongStudio { doc, unten: None, spiel: None, pegel: (vec![], 0.0), zieht: false, ansicht: (0.0, 0.0, 0.0, 0.0), spiel_muster: None, leisten_zu: None, modus: SongModus::Noten, karten_zu: ["block", "dokument"].into_iter().collect(), mehr: false, mitlaufen: true, scroll_ziel: None, live_stand: 0, kb: std::cell::Cell::new(KB), klingt: vec![], aus: Default::default() }); } Err(e) => { self.log.push(e); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }

    pub(crate) fn song_maus(&mut self, rel: &str, m: GMaus) {
        let Some(s) = self.songs.get_mut(rel) else { return };
        match m {
            GMaus::Unten { r, c } => { s.unten = Some((r, c)); s.doc.setze_cursor(r, c) }
            GMaus::Zug { r, c } => { if let Some(a) = s.unten { if a != (r, c) { s.doc.waehle(a, (r, c)) } } }
            GMaus::Los => s.unten = None,
        }
    }

    pub(crate) fn song_aktion(&mut self, rel: &str, a: GAkt) {
        if a == GAkt::Exportieren { self.export_oeffnen(); return }
        if matches!(a, GAkt::Konvertieren | GAkt::RmtImport) { return }
        let Some(s) = self.songs.get_mut(rel) else { return }; let d = &mut s.doc;
        match a {
            GAkt::Undo => d.rueckgaengig(), GAkt::Redo => d.wiederholen(),
            GAkt::Speichern => match d.speichern() { Ok(()) => self.log.push(format!("Gespeichert: {rel}")), Err(e) => self.log.push(e) },
            GAkt::Oktave(x) => d.oktave_aendern(x), GAkt::Instrument(x) => d.instrument_aendern(x), GAkt::Muster(p) => d.muster_waehlen(p),
            GAkt::Tempo(x) => { let t = (d.tempo() as i64 + x as i64).clamp(1, if d.ist_play() { 255 } else { 31 }) as u64; d.tempo_setzen(t) } GAkt::Lautstaerke(x) => { let v = (d.lautstaerke() as i64 + x as i64).clamp(0, 15) as u64; d.lautstaerke_setzen(v) } GAkt::Zeilen(x) => { let n = (d.zeilen() as i32 + x).clamp(1, 128) as usize; d.zeilen_setzen(n) }
            GAkt::Kopieren => d.kopieren(), GAkt::Ausschneiden => d.ausschneiden(), GAkt::Einfuegen => d.einfuegen(), GAkt::Transponieren(x) => d.transponieren(x), GAkt::Loeschen => d.loeschen(), GAkt::InstrumentAnwenden => d.instrument_anwenden(),
            GAkt::Schritt(x) => d.schritt = (d.schritt as i32 + x).clamp(0, 16) as usize,
            GAkt::Ord(i) => d.ord_waehlen(i), GAkt::OrdPlus => d.ord_anhaengen(), GAkt::OrdMinus => d.ord_entfernen(), GAkt::OrdSetzen => d.ord_setzen(), GAkt::OrdVerschieben(x) => d.ord_verschieben(x),
            GAkt::Schleife(x) => { let l = (d.media["loop"].as_u64().unwrap_or(0) as i64 + x as i64).max(0) as usize; d.schleife_setzen(l) }
            GAkt::MusterNeu => d.muster_neu(), GAkt::MusterDupl => d.muster_duplizieren(), GAkt::MusterLoeschen => d.muster_loeschen(),
            GAkt::InsWahl(i) => d.instrument_waehlen(i), GAkt::InsNeu => d.instrument_neu(), GAkt::InsDupl => d.instrument_duplizieren(), GAkt::InsLoeschen => { if !d.instrument_loeschen() { d.status = "Löschen nur für das letzte, unbenutzte Instrument".into() } }
            GAkt::InsName(n) => { let e = !s.zieht; s.zieht = true; d.instrument_wert("name", json!(n), e) }
            GAkt::InsWelle(w) => d.instrument_wert("wave", json!(w), true), GAkt::InsDist(w) => d.instrument_wert("dist", json!(w), true),
            GAkt::InsZahl(k, v) => { let e = !s.zieht; s.zieht = true; d.instrument_wert(k, json!(v), e) }
            GAkt::Huell(k, v) => { let e = !s.zieht; s.zieht = true; d.huellkurve_setzen(k, v, e) }
            GAkt::Modus(m) => s.modus = m,
            GAkt::Eintragen(n) => { let h = (n as i32 - 1 - d.oktave as i32 * 12).clamp(0, 24) as u8; if let Eingabe::Note(nn, i) = d.note_eingeben(h) { self.song_vorhoeren(rel, nn, i) } return }
            GAkt::Mitlaufen(b) => { s.mitlaufen = b; s.scroll_ziel = None }
            GAkt::Vorhoeren(i) => { let n = (d.oktave as u32 * 12 + 1).min(254) as u8; d.instrument_waehlen(i); self.song_vorhoeren(rel, n, i); return }
            GAkt::Probe(n) => { let i = d.ins_wahl; self.song_vorhoeren(rel, n, i); return } GAkt::Karte(k) => { if !s.karten_zu.remove(k) { s.karten_zu.insert(k); } } GAkt::Mehr => s.mehr = !s.mehr,
            GAkt::Leisten => { let zu = s.leisten_zu.unwrap_or(self.fenster.height / self.einst.zoom < 780.0); s.leisten_zu = Some(!zu) } GAkt::ReglerLos => s.zieht = false, GAkt::Sichtbar(c) => s.sichtbar_umschalten(c), GAkt::AlleEinblenden => s.aus.clear(), GAkt::Stumm(c) => { d.stumm_umschalten(c); let an = d.stumm[c]; self.audio.stumm(c, an) } GAkt::Solo(c) => { d.solo_umschalten(c); let an = d.solo[c]; self.audio.solo(c, an) }
            GAkt::SpielSong | GAkt::SpielMuster => { let nur = a == GAkt::SpielMuster; let (ord, zeile) = if nur { (0, 0) } else { (d.ord_wahl, 0) }; self.audio.abspielen(d.song_daten(nur), ord, zeile, &d.stumm, &d.solo); self.audio_song = Some(rel.to_string()); s.spiel = Some((ord, zeile)); s.spiel_muster = nur.then_some(d.muster) }
            GAkt::Stopp => { self.audio.stoppen(); self.audio_song = None; s.spiel = None; s.pegel = (vec![], 0.0) }
            GAkt::Exportieren | GAkt::Konvertieren | GAkt::RmtImport => {}
        }
    }

    /// Tracker-Tasten (nur ohne Cmd/Strg/Alt und nur bei Tracker-Fokus): Pfeile, Seiten, Klaviertasten, Entf, 1 = Notenende, `,` `.` Oktave, `[` `]` Instrument
    pub(crate) fn song_taste(&mut self, rel: &str, k: &keyboard::Key, m: keyboard::Modifiers) -> bool {
        if m.command() && !m.alt() && !m.shift() { if let keyboard::Key::Character(c) = k { let a = match c.to_lowercase().as_str() { "c" => Some(GAkt::Kopieren), "x" => Some(GAkt::Ausschneiden), "v" => Some(GAkt::Einfuegen), "a" => { if let Some(s) = self.songs.get_mut(rel) { s.doc.alles_waehlen() } return true } _ => None }; if let Some(a) = a { self.song_aktion(rel, a); return true } } }
        // Alt+↑/↓: Instrument der Zelle (oder der Auswahl) ändern, auch im Modus Noten; spielt die Note mit dem neuen Instrument vor
        if m.alt() && !m.command() && !m.control() { if let keyboard::Key::Named(n @ (keyboard::key::Named::ArrowUp | keyboard::key::Named::ArrowDown)) = k { if self.songs.get(rel).is_some_and(|s| s.modus == SongModus::Noten) {
            let dx = if *n == keyboard::key::Named::ArrowUp { 1 } else { -1 }; let r = self.songs.get_mut(rel).and_then(|s| s.doc.zell_instrument_aendern(dx)); if let Some((note, i)) = r { self.song_vorhoeren(rel, note, i) } return true } } }
        if m.command() || m.alt() || m.control() { return false }
        if self.songs.get(rel).is_some_and(|s| s.modus == SongModus::Instrumente) {
            // Modus Instrumente: die Klaviertasten spielen das gewählte Instrument vor (nichts wird eingetragen); ↑/↓ wählen das Instrument, Leertaste spielt vor
            use keyboard::key::Named; let (i, n) = { let d = &self.songs[rel].doc; (d.ins_wahl, d.anzahl_instrumente()) };
            match k {
                keyboard::Key::Named(Named::ArrowUp) => { if i > 0 { self.song_aktion(rel, GAkt::InsWahl(i - 1)) } return true }
                keyboard::Key::Named(Named::ArrowDown) => { if i + 1 < n { self.song_aktion(rel, GAkt::InsWahl(i + 1)) } return true }
                keyboard::Key::Named(Named::Space) => { self.song_aktion(rel, GAkt::Vorhoeren(i)); return true }
                keyboard::Key::Character(c) => { let ch = c.chars().next().unwrap_or(' '); let ch = self.phys.unwrap_or(ch); match ch { ',' => { self.song_aktion(rel, GAkt::Oktave(-1)); return true } '.' => { self.song_aktion(rel, GAkt::Oktave(1)); return true } _ => {} }
                    if let Some(h) = klaviertaste(ch) { let note = (self.songs[rel].doc.oktave as u32 * 12 + h as u32 + 1).min(254) as u8; self.song_aktion(rel, GAkt::Probe(note)); return true } return false }
                _ => return false,
            }
        }
        let Some(s) = self.songs.get_mut(rel) else { return false }; let d = &mut s.doc; use keyboard::key::Named;
        match k {
            keyboard::Key::Named(Named::ArrowDown) => d.bewege(1, 0, m.shift()), keyboard::Key::Named(Named::ArrowUp) => d.bewege(-1, 0, m.shift()),
            keyboard::Key::Named(Named::ArrowRight) => s.bewege_sichtbar(0, 1, m.shift()), keyboard::Key::Named(Named::ArrowLeft) => s.bewege_sichtbar(0, -1, m.shift()),
            keyboard::Key::Named(Named::PageDown) => d.bewege(8, 0, m.shift()), keyboard::Key::Named(Named::PageUp) => d.bewege(-8, 0, m.shift()),
            keyboard::Key::Named(Named::Home) => d.springe(0), keyboard::Key::Named(Named::End) => { let n = d.zeilen() - 1; d.springe(n) }
            keyboard::Key::Named(Named::Delete | Named::Backspace) => d.loeschen(),
            keyboard::Key::Named(Named::Space) => { let a = if self.audio_song.as_deref() == Some(rel) { GAkt::Stopp } else if m.shift() { GAkt::SpielMuster } else { GAkt::SpielSong }; self.song_aktion(rel, a); return true }
            keyboard::Key::Character(c) => {
                let ch = c.chars().next().unwrap_or(' '); let ch = self.phys.unwrap_or(ch); // Klaviertasten und 1 nach der Position der Taste
                match ch { ',' => d.oktave_aendern(-1), '.' => d.oktave_aendern(1), '[' => d.instrument_aendern(-1), ']' => d.instrument_aendern(1), '1' => d.notenende(),
                    _ => match klaviertaste(ch) { Some(h) => { if let Eingabe::Note(n, i) = d.note_eingeben(h) { self.song_vorhoeren(rel, n, i) } } None => return false } }
            }
            _ => return false,
        }
        true
    }
    /// Hörprobe der eingegebenen Note (7c: Audio über `aussen`)
    fn song_vorhoeren(&mut self, rel: &str, note: u8, instrument: usize) { if let Some(s) = self.songs.get(rel) { if let Some(i) = s.doc.instrumente().get(instrument) { self.audio.vorhoeren(i, note); self.probe_aktiv = true } } }

    /// Takt der Wiedergabe: Position, Mitlaufen (Muster folgt der Reihenfolge) und Pegel aktualisieren; am Ende Zustand zurücksetzen
    pub(crate) fn song_tick(&mut self) -> Task<Msg> {
        // Anzeige der Klaviatur: klingende Noten je Kanal und der Vorhörton (nur Anzeige); auch ohne Wiedergabe, solange ein Vorhörton klingt
        let snap = self.audio.pegel();
        let klingt: Vec<u8> = snap.notes.iter().flatten().copied().chain(snap.audition).collect();
        let anzeige = self.audio_song.clone().or_else(|| self.name().filter(|n| self.songs.contains_key(n)));
        if let Some(a) = &anzeige { if let Some(s) = self.songs.get_mut(a) { s.klingt = klingt.clone() } }
        if self.audio_song.is_none() { if klingt.is_empty() { self.probe_aktiv = false } return Task::none() }
        let Some(rel) = self.audio_song.clone() else { return Task::none() };
        let spielt = self.audio.spielt(); let (o, z) = self.audio.position(); let mut task = Task::none();
        if let Some(s) = self.songs.get_mut(&rel) {
            if spielt { if s.spiel_muster.is_none() { let p = s.doc.reihenfolge().get(o).copied(); if let Some(p) = p { s.doc.muster = p.min(s.doc.anzahl_muster().saturating_sub(1)); s.doc.ord_wahl = o } } s.spiel = Some((o, z)); s.pegel = (snap.peaks.clone(), snap.master_peak) } else { s.spiel = None; s.pegel = (vec![], 0.0) }
            if spielt {
                // W7: Änderungen im Raster gelten sofort: ändert sich der Song, geht eine neue Momentaufnahme an die Wiedergabe (Position bleibt)
                let stand = { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); s.doc.media.to_string().hash(&mut h); h.finish() };
                if s.live_stand == 0 { s.live_stand = stand } else if stand != s.live_stand { s.live_stand = stand; self.audio.aktualisieren(s.doc.song_daten(s.spiel_muster.is_some())) }
                // W2: das Raster scrollt senkrecht mit, die gespielte Zeile möglichst mittig, an den Mustergrenzen nie über den Rand
                if s.mitlaufen && s.modus == SongModus::Noten && s.ansicht.3 > 1.0 && s.spiel.is_some_and(|(oo, _)| s.spiel_muster.is_some() || s.doc.reihenfolge().get(oo) == Some(&s.doc.muster)) {
                    let (h, gesamt) = (s.ansicht.3, KOPF + ZH * s.doc.zeilen() as f32); let ziel = (KOPF + z as f32 * ZH + ZH / 2.0 - h / 2.0).clamp(0.0, (gesamt - h).max(0.0));
                    if (ziel - s.ansicht.1).abs() > 1.0 { s.scroll_ziel = Some(ziel); s.ansicht.1 = ziel; task = iced::widget::operation::scroll_to(iced::widget::Id::new(RASTER_ID), iced::widget::scrollable::AbsoluteOffset { x: None, y: Some(ziel) }) }
                }
            } else { s.live_stand = 0 }
        }
        if !spielt { self.audio_song = None }
        task
    }

    pub(crate) fn song_a11y(&self) -> Option<(String, String)> {
        let n = self.name()?; let s = self.songs.get(&n)?; let d = &s.doc; let (r, c) = d.cursor;
        Some((format!("Tracker {n}"), format!("Muster {} von {}, {} Zeilen, {} Kanäle, Cursor Zeile {r:02X} Kanal {}, Zelle {}, Oktave {}, Instrument {} {}", d.muster, d.anzahl_muster(), d.zeilen(), d.kanaele, c + 1, d.zelle(d.muster, r, c), d.oktave, d.instrument, d.instrument_name(d.instrument))))
    }

    pub(crate) fn song_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.song_ansicht_k(pal, &rel, muster::klasse(size.width), size.width)).into()
    }
    fn song_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse, breite: f32) -> Element<'_, Msg> {
        let Some(s) = self.songs.get(rel) else { return text("").into() }; let d = &s.doc; let r = rel.to_string();
        use muster::{abstand, schrift};
        // Der Knopf der Studios (muster::knopf): Höhe 28, Radius 6; `an` = verfügbar, `aktiv` = gewählt
        let tb = |name: &str, tipp: &str, an: bool, a: GAkt, aktiv: bool| -> Element<Msg> { muster::knopf(pal, name, tipp, an.then(|| Msg::SongAkt(r.clone(), a)), aktiv, false) };
        let play = d.ist_play();
        let chip_text = if play { format!("MEGA65 PLAY · {} Stimmen", d.kanaele) } else { match d.chip.as_str() { "sid" => format!("SID · {} Stimmen", d.kanaele), "pokey" => format!("POKEY · {} Stimmen", d.kanaele), c => format!("{} · {} Stimmen", c.to_uppercase(), d.kanaele) } };
        let spielt = self.audio_song.as_deref() == Some(rel);
        let kopf = |t: &str| text(t.to_string()).size(schrift::HINWEIS).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute);
        let hinweis = |t: String| text(t).size(schrift::HINWEIS).color(pal.mute);
        let wert = |t: String| text(t).size(schrift::TEXT).color(pal.text);
        let breit = klasse == muster::Klasse::Breit; let layout = self.layout();
        // Zeile mit Zähler: Beschriftung · − Wert +
        let zaehler = |name: &str, v: String, tipp_m: &str, tipp_p: &str, m_an: bool, p_an: bool, am: GAkt, ap: GAkt| -> Element<Msg> { iced::widget::row![text(name.to_string()).size(schrift::TEXT).color(pal.mute), tb("−", tipp_m, m_an, am, false), text(v).size(schrift::TEXT).font(vc_gui_ui::fonts::MONO).color(pal.text).width(20).align_x(iced::alignment::Horizontal::Center), tb("+", tipp_p, p_an, ap, false)].spacing(abstand::ENG).align_y(Alignment::Center).into() };
        // ---- Zone 2: genau eine Leiste: Modi · Transport (grüner Hauptknopf) · Oktave · Schritt · Mitscrollen
        let modus_wahl = iced::widget::row![
            tb("Noten", "Noten eingeben und bearbeiten (Cmd+1)", true, GAkt::Modus(SongModus::Noten), s.modus == SongModus::Noten), tb("Instrumente", "Instrumente, Klang und Probespielen (Cmd+2)", true, GAkt::Modus(SongModus::Instrumente), s.modus == SongModus::Instrumente), tb("Ablauf", "Reihenfolge und Muster (Cmd+3)", true, GAkt::Modus(SongModus::Ablauf), s.modus == SongModus::Ablauf)].spacing(abstand::ENG);
        let song_knopf = muster::knopf(pal, if spielt { "■ Stopp" } else { "▶ Song" }, "Song ab dem gewählten Eintrag der Reihenfolge abspielen / anhalten (Leertaste)", Some(Msg::SongAkt(r.clone(), if spielt { GAkt::Stopp } else { GAkt::SpielSong })), false, !spielt);
        let transport = iced::widget::row![song_knopf, tb("▶ Muster", "Nur das gewählte Muster abspielen (Umschalt+Leertaste)", !spielt, GAkt::SpielMuster, false)].spacing(abstand::ENG);
        let oktave = zaehler("Oktave", format!("{}", d.oktave), "Oktave tiefer (,)", "Oktave höher (.)", d.oktave > 0, d.oktave < 7, GAkt::Oktave(-1), GAkt::Oktave(1));
        let schritt = zaehler("Schritt", format!("{}", d.schritt), "Schrittweite kleiner", "Schrittweite größer", d.schritt > 0, d.schritt < 16, GAkt::Schritt(-1), GAkt::Schritt(1));
        let spiel_text = if spielt { match s.spiel { Some((_, z)) if s.spiel_muster.is_some() => format!("spielt: Muster {:02X} · Zeile {z:02}", d.muster), Some((o, z)) => format!("spielt: Eintrag {o} · Zeile {z:02}", ), None => "spielt".into() } } else { String::new() };
        let status_rechts: Element<Msg> = if let Some(e) = self.audio.fehler() { text(format!("Audio: {e}")).size(schrift::HINWEIS).color(pal.err).into() } else if spielt { text(spiel_text.clone()).size(schrift::HINWEIS).color(pal.ok).into() } else { space::horizontal().width(0).into() };
        let mitscrollen = tb(if s.mitlaufen { "Mitscrollen ✓" } else { "Mitscrollen" }, "Das Raster scrollt beim Abspielen mit; Handscrollen beendet das Mitlaufen, ein Klick schaltet es wieder ein", true, GAkt::Mitlaufen(!s.mitlaufen), s.mitlaufen);
        let leiste_inhalt: Element<Msg> = match klasse {
            muster::Klasse::Breit => { let mut z = iced::widget::row![modus_wahl, muster::trenner(pal), transport].spacing(abstand::GRUPPE).align_y(Alignment::Center);
                // nur, was im Modus gilt: Oktave (Noten, Instrumente), Schritt und Mitscrollen (nur Noten)
                if s.modus != SongModus::Ablauf { z = z.push(muster::trenner(pal)).push(oktave) } if s.modus == SongModus::Noten { z = z.push(schritt) }
                z = z.push(space::horizontal()).push(status_rechts); if s.modus == SongModus::Noten { z = z.push(mitscrollen) } z.into() }
            muster::Klasse::Mittel => { let mut z = iced::widget::row![modus_wahl, muster::trenner(pal), transport, space::horizontal()].spacing(abstand::GRUPPE).align_y(Alignment::Center); if s.modus == SongModus::Noten { z = z.push(mitscrollen) } z.into() }
            muster::Klasse::Schmal => iced::widget::row![modus_wahl, muster::trenner(pal), transport].spacing(abstand::GRUPPE).align_y(Alignment::Center).into(),
        };
        let leiste = muster::leiste(pal, leiste_inhalt);
        let ord = d.reihenfolge();
        // ---- Raster: Spalten füllen die Fläche (elastisch 112–200 px, viele Kanäle enger), Zeilenband läuft durch
        let verfuegbar = (breite - match klasse { muster::Klasse::Breit => 150.0 + 2.0 * abstand::GRUPPE + muster::SEITE_BREITE, muster::Klasse::Mittel => muster::SEITE_BREITE, muster::Klasse::Schmal => 0.0 } - 2.0 * abstand::GRUPPE - 14.0).max(NB + 60.0);
        let sichtbar = s.sichtbar(); let nk = sichtbar.len().max(1);
        let kb = ((verfuegbar - NB) / nk as f32).floor().clamp(if nk > 6 { 84.0 } else { 112.0 }, 200.0); s.kb.set(kb);
        let bw = (NB + kb * nk as f32).max(verfuegbar); let bh = KOPF + ZH * d.zeilen() as f32;
        let raster = scrollable(canvas::Canvas::new(Raster { d, pal, rel: rel.to_string(), versatz: (s.ansicht.0, s.ansicht.1), kb, breite: bw, sichtbar: sichtbar.clone(), spiel: s.spiel.filter(|(o, _)| match s.spiel_muster { Some(m) => m == d.muster, None => ord.get(*o) == Some(&d.muster) }).map(|(_, z)| z) }).width(bw).height(bh)).id(iced::widget::Id::new(RASTER_ID)).on_scroll({ let rel = rel.to_string(); move |v| { let (o, b) = (v.absolute_offset(), v.bounds()); Msg::SongScroll(rel.clone(), o.x, o.y, b.width, b.height) } }).direction(iced::widget::scrollable::Direction::Both { vertical: Default::default(), horizontal: Default::default() }).width(Length::Fill).height(Length::Fill);
        // ---- Zone 3 (Noten, Ablauf): Muster als Karten, „+“ im Kopf
        let muster_liste: Element<Msg> = { let mut l = iced::widget::column![iced::widget::row![kopf(&format!("Muster · {}", d.anzahl_muster())), space::horizontal(), muster::knopf(pal, "+", "Leeres Muster anlegen", Some(Msg::SongAkt(r.clone(), GAkt::MusterNeu)), false, false)].align_y(Alignment::Center)].spacing(abstand::GRUPPE);
            for p in 0..d.anzahl_muster() { let an = d.muster == p; let n = ord.iter().filter(|&&q| q == p).count();
                l = l.push(button(iced::widget::column![text(format!("Muster {p:02}")).size(schrift::TEXT).color(pal.text), text(if n == 0 { "unbenutzt".to_string() } else { format!("{n}× im Ablauf") }).size(schrift::HINWEIS).color(pal.mute)].spacing(2)).width(Length::Fill).padding([6, 10]).on_press(Msg::SongAkt(r.clone(), GAkt::Muster(p)))
                    .style(move |_, st| button::Style { background: Some(if an { pal.sel.into() } else if matches!(st, button::Status::Hovered) { pal.line.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() })) }
            l = l.push(iced::widget::row![tb("Dupl.", "Gewähltes Muster kopieren", true, GAkt::MusterDupl, false), tb("Löschen", "Gewähltes Muster entfernen (Einträge der Reihenfolge verschwinden)", d.anzahl_muster() > 1, GAkt::MusterLoeschen, false)].spacing(abstand::ENG).wrap().vertical_spacing(abstand::ENG));
            if breit { scrollable(l).into() } else { l.into() } };
        let loop_at = d.media["loop"].as_u64().unwrap_or(0) as usize;
        // ---- Ablauf: die Reihenfolge als Kartenkette (große Zellen, ↺ = Wiederholung ab, ▶ = wird gespielt)
        let ablauf: Element<Msg> = if play { container(hinweis("PLAY-Dateien kennen keine Reihenfolge: ein Muster wird einmal gespielt.".into())).padding(abstand::KARTE).into() } else {
            let mut kette = iced::widget::row![].spacing(abstand::GRUPPE);
            for (i, p) in ord.iter().enumerate() { let an = d.ord_wahl == i; let spielend = s.spiel.is_some_and(|(o, _)| o == i);
                kette = kette.push(button(iced::widget::column![text(format!("{}{i:02}", if spielend { "▶ " } else { "" })).size(schrift::HINWEIS).color(if spielend { pal.ok } else { pal.mute }), text(format!("Muster {p:02}")).size(schrift::TEXT).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(if i == loop_at { "↺ Wiederholung" } else { " " }).size(schrift::HINWEIS).color(pal.accent)].spacing(2)).padding([8, 12]).width(112).on_press(Msg::SongAkt(r.clone(), GAkt::Ord(i)))
                    .style(move |_, st| button::Style { background: Some(if an { pal.sel.into() } else if matches!(st, button::Status::Hovered) { pal.line.into() } else { pal.panel.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KARTE.into() }, text_color: pal.text, ..Default::default() })) }
            let aktionen = iced::widget::row![tb("+", "Gewähltes Muster hinten anhängen", true, GAkt::OrdPlus, false), tb("−", "Gewählten Eintrag entfernen", ord.len() > 1, GAkt::OrdMinus, false), tb("Setzen", "Eintrag auf das gewählte Muster setzen", true, GAkt::OrdSetzen, false), tb("←", "Eintrag nach vorn", d.ord_wahl > 0, GAkt::OrdVerschieben(-1), false), tb("→", "Eintrag nach hinten", d.ord_wahl + 1 < ord.len(), GAkt::OrdVerschieben(1), false), muster::trenner(pal),
                text(format!("Wiederholung ab {loop_at}")).size(schrift::TEXT).color(pal.mute), tb("−", "Wiederholung früher beginnen", loop_at > 0, GAkt::Schleife(-1), false), tb("+", "Wiederholung später beginnen", loop_at + 1 < ord.len(), GAkt::Schleife(1), false)].spacing(abstand::ENG).align_y(Alignment::Center).wrap().vertical_spacing(abstand::ENG);
            iced::widget::column![text("Reihenfolge").size(schrift::ABSCHNITT).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), hinweis("Die Einträge werden nacheinander gespielt; ↺ = ab hier wiederholt sich der Song.".into()), kette.wrap().vertical_spacing(abstand::GRUPPE), aktionen].spacing(abstand::GRUPPE).into() };
        // ---- Instrumente-Modus: Klang-Editor eingepasst (keine Scrollleiste, keine Klaviatur quer)
        let mut instr = iced::widget::column![].spacing(abstand::GRUPPE);
        instr = instr.push(kopf(if play { "Hüllkurven · ENVELOPE 0–9 im PLAY-Befehl" } else if d.pokey { "Verzerrung und Lautstärke je Bild" } else { "Wellenform, Pulsbreite und ADSR" }));
        let iv = &d.media["instruments"][d.ins_wahl];
        if !iv.is_null() {
            let r2 = r.clone(); instr = instr.push(text_input("Name", iv["name"].as_str().unwrap_or("")).on_input(move |n| Msg::SongAkt(r2.clone(), GAkt::InsName(n))).on_submit(Msg::SongAkt(r.clone(), GAkt::ReglerLos)).size(schrift::TEXT).padding(6));
            instr = instr.push(canvas::Canvas::new(Huellkurve { ins: vc_ide_model::synth::Instr::from_json(iv, d.pokey), pal }).width(Length::Fill).height(Length::Fill));
            let zahl = |name: &'static str, k: &'static str, max: i32, d0: i32| -> Element<Msg> { let v = (iv[k].as_i64().unwrap_or(d0 as i64) as i32).clamp(0, max); let (r2, r3) = (r.clone(), r.clone()); iced::widget::row![text(name).size(schrift::TEXT).color(pal.mute).width(96), iced::widget::slider(0..=max, v, move |x| Msg::SongAkt(r2.clone(), GAkt::InsZahl(k, x as i64))).on_release(Msg::SongAkt(r3, GAkt::ReglerLos)).width(Length::Fill), text(format!("{v}")).size(schrift::TEXT).font(vc_gui_ui::fonts::MONO).color(pal.text).width(40).align_x(iced::alignment::Horizontal::Right)].spacing(abstand::GRUPPE).align_y(Alignment::Center).into() };
            if d.pokey {
                let cur = iv["dist"].as_str().unwrap_or("pure"); let mut dz = iced::widget::row![].spacing(abstand::ENG);
                for (k, l) in [("pure", "Reiner Ton"), ("noise", "Rauschen"), ("buzz", "Buzz (4-Bit)"), ("gritty", "Gritty (5-Bit)"), ("bass", "Bass (5+4-Bit)")] { dz = dz.push(tb(l, "Verzerrung", true, GAkt::InsDist(k.into()), cur == k)) } instr = instr.push(dz.wrap().vertical_spacing(abstand::ENG));
                instr = instr.push(hinweis("Lautstärke je Bild (0–15)".into()));
                let env: Vec<u8> = iv["env"].as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0).min(15) as u8).collect()).unwrap_or_else(|| vec![15; 16]); let mut er = iced::widget::row![].spacing(2);
                for k in 0..16 { let (r2, r3) = (r.clone(), r.clone()); er = er.push(iced::widget::vertical_slider(0..=15u8, env.get(k).copied().unwrap_or(0), move |v| Msg::SongAkt(r2.clone(), GAkt::Huell(k, v))).on_release(Msg::SongAkt(r3, GAkt::ReglerLos)).height(56).width(14)) } instr = instr.push(er);
                instr = instr.push(zahl("Wiederholung ab", "loop", 15, 15));
            } else {
                let cur = iv["wave"].as_str().unwrap_or("pulse"); let mut wz = iced::widget::row![].spacing(abstand::ENG);
                for (k, l) in [("triangle", "Dreieck"), ("saw", "Sägezahn"), ("pulse", "Rechteck/Puls"), ("noise", "Rauschen")] { wz = wz.push(tb(l, "Wellenform", true, GAkt::InsWelle(k.into()), cur == k)) } instr = instr.push(wz.wrap().vertical_spacing(abstand::ENG));
                instr = instr.push(zahl("Pulsbreite", "pw", 4095, 2048)).push(zahl("Attack", "a", 15, 0)).push(zahl("Decay", "d", 15, 9)).push(zahl("Sustain", "s", 15, 8)).push(zahl("Release", "r", 15, 4));
            }
        }
        // ---- Karten der Seitenspalte
        let karte_zu = |k: &'static str| -> Option<(bool, Msg)> { Some((!s.karten_zu.contains(k), Msg::SongAkt(r.clone(), GAkt::Karte(k)))) };
        // Werkzeug (Noten): Instrumentenliste mit ▶ je Zeile; Oktave und Schritt stehen in der breiten Klasse in der Leiste
        let mut inst_zeilen = iced::widget::column![].spacing(abstand::ENG);
        for i in 0..d.anzahl_instrumente() { let an = d.instrument == i;
            inst_zeilen = inst_zeilen.push(iced::widget::row![button(container(text(format!("{i:02}  {}", d.instrument_name(i))).size(schrift::TEXT).color(pal.text)).center_y(Length::Fill)).height(muster::KNOPF_HOEHE).padding([0, 10]).width(Length::Fill).on_press(Msg::SongAkt(r.clone(), GAkt::InsWahl(i)))
                .style(move |_, st| button::Style { background: Some(if an { pal.sel.into() } else if matches!(st, button::Status::Hovered) { pal.line.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() }),
                tb("▶", "Vorhören (Note C der gewählten Oktave)", true, GAkt::Vorhoeren(i), false)].spacing(abstand::ENG).align_y(Alignment::Center)) }
        let mut werkzeug = iced::widget::column![kopf("Instrument"), inst_zeilen].spacing(abstand::GRUPPE);
        if !breit { werkzeug = werkzeug.push(oktave_stueck(&tb, d)).push(schritt_stueck(&tb, d)) }
        werkzeug = werkzeug.push(hinweis("Vorhandene Note ändern: Zahl in der Zelle tippen oder Alt+↑/↓.".into()));
        let klaviatur: Element<Msg> = iced::widget::column![container(canvas::Canvas::new(Klaviatur { oktave: d.oktave as usize, pal, rel: rel.to_string(), klingt: s.klingt.clone(), tasten: if self.einst.tasten_auf_klaviatur { (0..=24).filter_map(|h| crate::tastatur::fuer_halbton(layout, h)).collect() } else { vec![] } }).width(Length::Fill).height(Length::Fixed(88.0))).style(move |_| flaeche(pal.editor, pal.line)), hinweis(format!("Klick trägt die Note mit Instrument {:02} an der Cursorzelle ein und rückt vor (Oktave {}–{}). Computertasten {} … tun dasselbe.", d.instrument, d.oktave, d.oktave + 1, crate::tastatur::hinweis_unten(layout).chars().take(5).collect::<String>()))].spacing(abstand::GRUPPE).into();
        // Kanäle: Solo/Stumm je Kanal; Pegel: Balken je Kanal und Summe
        let mut kanaele = iced::widget::column![].spacing(abstand::GRUPPE);
        for c in 0..d.kanaele { kanaele = kanaele.push(iced::widget::row![text(format!("Kanal {}", c + 1)).size(schrift::TEXT).color(pal.text).width(Length::Fill).wrapping(iced::widget::text::Wrapping::None), tb(if d.solo[c] { "Solo ✓" } else { "Solo" }, &format!("Kanal {} solo", c + 1), true, GAkt::Solo(c), d.solo[c]), tb(if d.stumm[c] { "Stumm ✓" } else { "Stumm" }, &format!("Kanal {} stumm", c + 1), true, GAkt::Stumm(c), d.stumm[c]), tb(if s.aus.contains(&c) { "Zeigen" } else { "Zeigen ✓" }, &format!("Kanal {} im Raster zeigen oder ausblenden (spielt weiter; nur Ansicht)", c + 1), s.aus.contains(&c) || s.sichtbar().len() > 1, GAkt::Sichtbar(c), !s.aus.contains(&c))].spacing(abstand::ENG).align_y(Alignment::Center)) }
        let mut pegel = iced::widget::column![].spacing(abstand::GRUPPE);
        for c in 0..d.kanaele { let v = s.pegel.0.get(c).copied().unwrap_or(0.0).clamp(0.0, 1.0); pegel = pegel.push(iced::widget::progress_bar(0.0..=1.0, v).girth(8).length(Length::Fill)) }
        pegel = pegel.push(iced::widget::row![hinweis("Summe".into()), iced::widget::progress_bar(0.0..=1.0, (s.pegel.1 * 1.8).clamp(0.0, 1.0)).girth(8).length(Length::Fill)].spacing(abstand::GRUPPE).align_y(Alignment::Center));
        let block = iced::widget::row![tb("Kopieren", "Zellen merken (Auswahl oder Cursorzelle) (Cmd+C)", true, GAkt::Kopieren, false), tb("Ausschneiden", "Zellen merken und leeren (Cmd+X)", true, GAkt::Ausschneiden, false), tb("Einfügen", "Gemerkte Zellen ab dem Cursor einfügen (Cmd+V)", d.zwischen.is_some(), GAkt::Einfuegen, false),
            tb("Leeren", "Auswahl leeren (Entf)", true, GAkt::Loeschen, false), tb("♯", "Noten im Block einen Halbton höher", true, GAkt::Transponieren(1), false), tb("♭", "Noten im Block einen Halbton tiefer", true, GAkt::Transponieren(-1), false), tb("+12", "Noten im Block eine Oktave höher", true, GAkt::Transponieren(12), false), tb("−12", "Noten im Block eine Oktave tiefer", true, GAkt::Transponieren(-12), false),
            tb("Instrument setzen", "Gewähltes Instrument in alle Noten des Blocks eintragen", true, GAkt::InstrumentAnwenden, false)].spacing(abstand::ENG).wrap().vertical_spacing(abstand::ENG);
        let tempo = || zaehler(if play { "Tempo" } else { "Takt" }, format!("{}", d.tempo()), if play { "Tempo langsamer" } else { "Takt schneller (weniger Bilder je Zeile)" }, if play { "Tempo schneller" } else { "Takt langsamer" }, d.tempo() > 1, d.tempo() < if play { 255 } else { 31 }, GAkt::Tempo(-1), GAkt::Tempo(1));
        let dokument_bauen = || { let mut dokument = iced::widget::column![wert(chip_text.clone()), zaehler(if play { "Schritte" } else { "Zeilen" }, format!("{}", d.zeilen()), "8 Zeilen weniger", "8 Zeilen mehr", d.zeilen() > 8, d.zeilen() < 128, GAkt::Zeilen(-8), GAkt::Zeilen(8)), tempo()].spacing(abstand::GRUPPE);
            if play { dokument = dokument.push(zaehler("Lautstärke", format!("{}", d.lautstaerke()), "Leiser (VOL im PLAY-Befehl)", "Lauter", d.lautstaerke() > 0, d.lautstaerke() < 15, GAkt::Lautstaerke(-1), GAkt::Lautstaerke(1))) }
            dokument = dokument.push(iced::widget::row![tb("Konvertieren von …", "Song aus einem anderen System holen und für diesen Chip umwandeln: Song (SID ↔ POKEY), GoatTracker (.sng) oder SID-Datei (.sid, Aufnahme)", !play, GAkt::Konvertieren, false), tb("RMT importieren …", "RMT-Modul (Raster Music Tracker) als POKEY-Song laden", d.pokey && !play, GAkt::RmtImport, false)].spacing(abstand::ENG).wrap().vertical_spacing(abstand::ENG));
            dokument };
        let k_werkzeug = muster::karte(pal, "Werkzeug", "✎", "gilt für neue Noten · nicht in der Datei", karte_zu("werkzeug"), werkzeug.into());
        let k_klav = muster::karte(pal, "Klaviatur", "♫", "Eingabe per Klick · nicht in der Datei", karte_zu("klaviatur"), klaviatur);
        let k_kanaele = muster::karte(pal, "Kanäle", "▤", "Wiedergabe", karte_zu("kanaele"), kanaele.into());
        let k_pegel = muster::karte(pal, "Pegel", "≈", if self.audio.fehler().is_some() { "Audio nicht verfügbar" } else { "" }, karte_zu("pegel"), pegel.into());
        let k_block = muster::karte(pal, "Block", "▭", "Auswahl und Zwischenablage", karte_zu("block"), block.into());
        let k_dok = muster::karte(pal, "Dokument", "▥", "Chip, Länge, Tempo · wird gespeichert", karte_zu("dokument"), dokument_bauen().into());
        let k_vorhoeren = muster::karte(pal, "Vorhören", "▶", "nicht in der Datei", karte_zu("vorhoeren"), iced::widget::column![oktave_stueck(&tb, d), hinweis(format!("Computertasten {} … spielen das gewählte Instrument vor, ohne etwas einzutragen; ↑/↓ wählen das Instrument, Leertaste spielt vor.", crate::tastatur::hinweis_unten(layout).chars().take(11).collect::<String>()))].spacing(abstand::GRUPPE).into());
        let (cr, cc) = d.cursor;
        let status = format!("Muster {:02} · Zeile {cr:02} · Kanal {} · {}{}", d.muster, cc + 1, d.zelle(d.muster, cr, cc), if d.status.is_empty() { String::new() } else { format!(" · {}", d.status) });
        let fuss = muster::fusszeile(pal, &status, iced::widget::tooltip(text("? Tasten").size(schrift::HINWEIS).color(pal.mute), container(text(tasten_tipp(layout)).size(11.5).color(pal.text)).padding([4, 8]).max_width(440).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Top).into());
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let masse = format!("{} · Muster {:02} von {}", chip_text, d.muster, d.anzahl_muster());
        // schmal: nur Name und Speichern (Rückgängig und Export bleiben über Cmd+Z und die Befehlspalette erreichbar)
        let kopf_aktionen = if klasse == muster::Klasse::Schmal { vec![tb("Speichern", "Speichern (Cmd+S)", d.dirty(), GAkt::Speichern, false)] } else { let mut v = vec![]; if breit { v.push(tb("↶", "Rückgängig (Cmd+Z)", d.kann_undo(), GAkt::Undo, false)); v.push(tb("↷", "Wiederholen (Cmd+Umschalt+Z)", d.kann_redo(), GAkt::Redo, false)) }
            v.push(tb("Exportieren …", "Export-Dialog (Cmd+Umschalt+E)", true, GAkt::Exportieren, false)); v.push(tb("Speichern", "Speichern (Cmd+S)", d.dirty(), GAkt::Speichern, false)); v };
        let kopf_z = muster::kopfzeile(pal, klasse, if play { "PLY" } else { "TRK" }, &name, d.dirty(), &masse, kopf_aktionen);
        // Instrumentenliste im Modus Instrumente: Name + ▶ je Zeile
        let mut ins_liste = iced::widget::column![kopf(&format!("Instrumente · {}", d.anzahl_instrumente()))].spacing(abstand::GRUPPE);
        for i in 0..d.anzahl_instrumente() { let an = d.ins_wahl == i; ins_liste = ins_liste.push(iced::widget::row![button(container(text(format!("{i:02} {}", d.instrument_name(i))).size(schrift::TEXT).color(pal.text)).center_y(Length::Fill)).height(muster::KNOPF_HOEHE).padding([0, 10]).width(Length::Fill).on_press(Msg::SongAkt(r.clone(), GAkt::InsWahl(i)))
            .style(move |_, st| button::Style { background: Some(if an { pal.sel.into() } else if matches!(st, button::Status::Hovered) { pal.line.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if an { 2.0 } else { 1.0 }, radius: muster::RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() }), tb("▶", "Vorhören (Note C der gewählten Oktave)", true, GAkt::Vorhoeren(i), false)].spacing(abstand::ENG).align_y(Alignment::Center)) }
        ins_liste = ins_liste.push(iced::widget::row![tb("Neu", "Neues Instrument (PLAY: höchstens 10, sonst 32)", d.anzahl_instrumente() < d.max_instrumente(), GAkt::InsNeu, false), tb("Dupl.", "Instrument kopieren", d.anzahl_instrumente() > 0 && d.anzahl_instrumente() < d.max_instrumente(), GAkt::InsDupl, false), tb("Löschen", "Nur das letzte, unbenutzte Instrument lässt sich löschen", d.anzahl_instrumente() > 1, GAkt::InsLoeschen, false)].spacing(abstand::ENG).wrap().vertical_spacing(abstand::ENG));
        let flaechen_stil = move |_: &Theme| flaeche(pal.editor, pal.line);
        let (fl, liste, seite): (Element<Msg>, Option<Element<Msg>>, Element<Msg>) = match s.modus {
            SongModus::Noten => (container(raster).padding(abstand::GRUPPE).width(Length::Fill).height(Length::Fill).style(flaechen_stil).into(), Some(muster_liste), muster::seitenspalte(klasse, vec![k_werkzeug, k_klav, k_kanaele, k_pegel, k_block, k_dok])),
            SongModus::Instrumente => (container(instr).padding(abstand::KARTE).width(Length::Fill).height(Length::Fill).style(flaechen_stil).into(), Some(if breit { scrollable(ins_liste).into() } else { Element::from(ins_liste) }), muster::seitenspalte(klasse, vec![k_vorhoeren, k_pegel, k_dok])),
            SongModus::Ablauf => (container(ablauf).padding(abstand::KARTE).width(Length::Fill).height(Length::Fill).style(flaechen_stil).into(), Some(muster_liste), muster::seitenspalte(klasse, vec![k_kanaele, k_pegel, k_dok])),
        };
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: if s.modus == SongModus::Instrumente { 220.0 } else { 150.0 }, kopf: kopf_z, leiste, liste, flaeche: fl, seite, fuss })
    }
}

/// Oktave- und Schrittzeilen für die Karten der mittleren und schmalen Klasse (in der breiten stehen sie in der Leiste)
fn oktave_stueck<'a>(tb: &dyn Fn(&str, &str, bool, GAkt, bool) -> Element<'a, Msg>, d: &SongDoc) -> Element<'a, Msg> {
    iced::widget::row![text(format!("Oktave {}", d.oktave)).size(muster::schrift::TEXT).width(90), tb("−", "Oktave tiefer (,)", d.oktave > 0, GAkt::Oktave(-1), false), tb("+", "Oktave höher (.)", d.oktave < 7, GAkt::Oktave(1), false)].spacing(muster::abstand::ENG).align_y(Alignment::Center).into()
}
fn schritt_stueck<'a>(tb: &dyn Fn(&str, &str, bool, GAkt, bool) -> Element<'a, Msg>, d: &SongDoc) -> Element<'a, Msg> {
    iced::widget::row![text(format!("Schritt {}", d.schritt)).size(muster::schrift::TEXT).width(90), tb("−", "Schrittweite kleiner", d.schritt > 0, GAkt::Schritt(-1), false), tb("+", "Schrittweite größer", d.schritt < 16, GAkt::Schritt(1), false)].spacing(muster::abstand::ENG).align_y(Alignment::Center).into()
}

/// Text des Tooltips „? Tasten“ in der Fußzeile; die Noten-Tasten gelten nach der Position und stehen in der Beschriftung des Layouts
pub fn tasten_tipp(layout: crate::einstellungen::Tastaturlayout) -> String {
    format!("Leertaste spielt/stoppt (Umschalt: nur Muster) · {} / {} = Noten (nach Tastenposition, Layout {}) · 1 Notenende · Entf löscht · , . Oktave · [ ] Instrument · Pfeile, Umschalt+Pfeile wählt aus", crate::tastatur::hinweis_unten(layout), crate::tastatur::hinweis_oben(layout), crate::tastatur::name(layout))
}
