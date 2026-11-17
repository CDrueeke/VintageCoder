// SPDX-License-Identifier: GPL-3.0-or-later
//! Arbeitsfläche der Studios (Muster, Konzept „UX-Überarbeitung“ 3.4): Einpassen statt Scrollen, ganzzahliges Zoom, Verschieben mit Leertaste und Ziehen,
//! Cmd/Strg + Mausrad zoomt. Gemeinsam für alle Studios; die Zeichenflächen rechnen nur mit [`Ansicht::geometrie`] und reichen Ereignisse an [`Gesten`].
//! Tasten gehören der Befehlstabelle: [`Gesten`] merkt sich Leertaste und Cmd nur als Zustand und verbraucht keine Taste.
use iced::{keyboard, mouse, Event, Point, Rectangle, Size};
use std::cell::Cell;

/// Rand um die eingepasste Fläche (Pixel, insgesamt)
pub const RAND: f32 = 24.0;
pub const ZOOM_MIN: f32 = 1.0;
pub const ZOOM_MAX: f32 = 48.0;

/// Zoom und Verschiebung einer Fläche; `platz` ist die zuletzt gemeldete Größe der Zeichenfläche (zum Rechnen außerhalb von `draw`)
#[derive(Clone)]
pub struct Ansicht { pub zoom: Option<f32>, pub pan: (f32, f32), pub platz: Cell<(f32, f32)>, /// höchstes Zoom dieser Fläche (Bilder, die bei jeder Stufe neu erzeugt werden, begrenzen es)
    pub max: f32 }
impl Default for Ansicht { fn default() -> Self { Ansicht { zoom: None, pan: (0.0, 0.0), platz: Cell::new((0.0, 0.0)), max: ZOOM_MAX } } }

/// Lage der Fläche im Zeichenbereich: Zoom, linke obere Ecke
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geo { pub z: f32, pub ox: f32, pub oy: f32 }

impl Ansicht {
    /// Größtes ganzzahliges Zoom, bei dem eine Fläche aus `w`×`h` Einheiten samt Rand ganz in `b` passt
    pub fn einpass(b: Size, w: f32, h: f32) -> f32 { ((b.width - RAND) / w.max(1.0)).min((b.height - RAND) / h.max(1.0)).floor().clamp(ZOOM_MIN, ZOOM_MAX) }
    pub fn platz_groesse(&self) -> Size { Size::new(self.platz.get().0, self.platz.get().1) }
    /// Zoom, das gerade gilt (von Hand gesetzt oder eingepasst)
    pub fn zoom_wert(&self, w: f32, h: f32) -> f32 { self.zoom.unwrap_or_else(|| Self::einpass(self.platz_groesse(), w, h).min(self.max)) }
    /// Fläche mit höchstem Zoom `max`
    pub fn mit_max(max: f32) -> Ansicht { Ansicht { max: max.clamp(ZOOM_MIN, ZOOM_MAX), ..Default::default() } }
    /// Verschiebung begrenzen: passt die Fläche hinein, bleibt sie zentriert; sonst wandert sie nie ganz heraus
    pub fn begrenzen(b: Size, z: f32, w: f32, h: f32, pan: (f32, f32)) -> (f32, f32) {
        let ein = |avail: f32, inhalt: f32, p: f32| if inhalt + RAND <= avail { 0.0 } else { let m = (inhalt + RAND - avail) / 2.0; p.clamp(-m, m) };
        (ein(b.width, w * z, pan.0), ein(b.height, h * z, pan.1))
    }
    /// Linke obere Ecke eines Bildes (`w`×`h` Punkte, Zoom 1) im Zeichenbereich `b`: zentriert, mit begrenzter Verschiebung, auf ganze Punkte
    pub fn ecke(&self, b: Size, w: f32, h: f32) -> (f32, f32) {
        let (px, py) = Self::begrenzen(b, 1.0, w, h, self.pan); (((b.width - w) / 2.0 + px).floor(), ((b.height - h) / 2.0 + py).floor())
    }
    /// Lage der Fläche (`w`×`h` Einheiten) im Zeichenbereich `b`; merkt dessen Größe
    pub fn geometrie(&self, b: Size, w: f32, h: f32) -> Geo {
        self.platz.set((b.width, b.height));
        let z = self.zoom.unwrap_or_else(|| Self::einpass(b, w, h).min(self.max)); let (px, py) = Self::begrenzen(b, z, w, h, self.pan);
        Geo { z, ox: ((b.width - w * z) / 2.0 + px).floor(), oy: ((b.height - h * z) / 2.0 + py).floor() }
    }
    /// Eine Stufe größer bzw. kleiner (ganzzahlig, von der sichtbaren Stufe aus)
    pub fn stufe(&mut self, groesser: bool, w: f32, h: f32) {
        let z = self.zoom_wert(w, h).round(); self.zoom = Some(if groesser { (z + 1.0).min(self.max) } else { (z - 1.0).max(ZOOM_MIN) });
        self.pan = Self::begrenzen(self.platz_groesse(), self.zoom.unwrap_or(z), w, h, self.pan);
    }
    pub fn setze(&mut self, z: f32) { self.zoom = Some(z.round().clamp(ZOOM_MIN, self.max)) }
    pub fn einpassen(&mut self) { self.zoom = None; self.pan = (0.0, 0.0) }
    pub fn schieben(&mut self, dx: f32, dy: f32, w: f32, h: f32) { let z = self.zoom_wert(w, h); self.pan = Self::begrenzen(self.platz_groesse(), z, w, h, (self.pan.0 + dx, self.pan.1 + dy)) }
    /// Beschriftung für die Fußzeile: „Einpassen · 16×“ bzw. „16×“
    pub fn zoom_text(&self, w: f32, h: f32) -> String {
        if self.platz.get().0 < 1.0 { return if self.zoom.is_none() { "Einpassen".into() } else { format!("{}×", self.zoom.unwrap_or(1.0) as i32) } }
        format!("{}{}×", if self.zoom.is_none() { "Einpassen · " } else { "" }, self.zoom_wert(w, h) as i32)
    }
}

/// Zustand der Gesten einer Zeichenfläche (im `State` des Canvas)
#[derive(Default)]
pub struct Gesten { leertaste: bool, befehl: bool, schiebt: Option<Point> }
/// Ergebnis einer Geste
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Geste { /// Ereignis verbraucht (kein weiterer Ablauf, keine Nachricht)
    Erfasst, /// Fläche verschieben (Pixel)
    Schieben(f32, f32), /// eine Stufe größer (`true`) bzw. kleiner
    Zoom(bool) }
impl Gesten {
    /// Leertaste halten + Ziehen verschiebt; Cmd/Strg + Mausrad zoomt. Alles andere bleibt unberührt (`None`)
    pub fn ereignis(&mut self, e: &Event, b: Rectangle, c: mouse::Cursor) -> Option<Geste> {
        use keyboard::key::Named; use mouse::{Button, Event as M};
        match e {
            Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(Named::Space), .. }) => { self.leertaste = true; None }
            Event::Keyboard(keyboard::Event::KeyReleased { key: keyboard::Key::Named(Named::Space), .. }) => { self.leertaste = false; self.schiebt = None; None }
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => { self.befehl = m.command(); None }
            Event::Mouse(M::ButtonPressed(Button::Left)) if self.leertaste => { self.schiebt = c.position_in(b); self.schiebt.map(|_| Geste::Erfasst) }
            Event::Mouse(M::CursorMoved { .. }) if self.schiebt.is_some() => {
                let p = c.position_in(b).or_else(|| c.position().map(|p| Point::new(p.x - b.x, p.y - b.y)))?; let a = self.schiebt.replace(p)?; Some(Geste::Schieben(p.x - a.x, p.y - a.y))
            }
            Event::Mouse(M::ButtonReleased(Button::Left)) if self.schiebt.is_some() => { self.schiebt = None; Some(Geste::Erfasst) }
            Event::Mouse(M::WheelScrolled { delta }) if self.befehl && c.position_in(b).is_some() => {
                let y = match delta { mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => *y };
                if y == 0.0 { None } else { Some(Geste::Zoom(y > 0.0)) }
            }
            _ => None,
        }
    }
    pub fn schiebt(&self) -> bool { self.schiebt.is_some() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn einpassen_ist_ganzzahlig_zentriert_und_begrenzt() {
        let b = Size::new(500.0, 400.0); assert_eq!(Ansicht::einpass(b, 12.0, 21.0), 17.0); assert_eq!(Ansicht::einpass(Size::new(10.0, 10.0), 12.0, 21.0), 1.0, "nie kleiner als 1");
        assert_eq!(Ansicht::einpass(Size::new(5000.0, 5000.0), 2.0, 2.0), 48.0, "nie größer als 48");
        let a = Ansicht::default(); let g = a.geometrie(b, 12.0, 21.0); assert_eq!(g.z, 17.0); assert!(g.ox >= 0.0 && g.oy >= 0.0 && g.ox + 12.0 * 17.0 <= 500.0);
        let mut a = Ansicht::default(); a.geometrie(b, 12.0, 21.0); a.zoom = Some(40.0); a.schieben(10_000.0, -10_000.0, 12.0, 21.0);
        let g = a.geometrie(b, 12.0, 21.0); assert!(g.ox <= RAND && g.oy + 21.0 * 40.0 >= b.height - RAND, "Verschiebung begrenzt: {g:?}");
        a.einpassen(); assert!(a.zoom.is_none() && a.pan == (0.0, 0.0));
    }
    #[test]
    fn zoomstufen_sind_ganzzahlig_von_der_sichtbaren_stufe_aus() {
        let mut a = Ansicht::default(); a.geometrie(Size::new(500.0, 400.0), 12.0, 21.0);
        a.stufe(true, 12.0, 21.0); assert_eq!(a.zoom, Some(18.0)); a.stufe(false, 12.0, 21.0); a.stufe(false, 12.0, 21.0); assert_eq!(a.zoom, Some(16.0));
        a.setze(0.2); assert_eq!(a.zoom, Some(1.0)); a.stufe(false, 12.0, 21.0); assert_eq!(a.zoom, Some(1.0)); a.setze(99.0); assert_eq!(a.zoom, Some(48.0));
        assert_eq!(Ansicht::default().zoom_text(1.0, 1.0), "Einpassen");
    }
}
