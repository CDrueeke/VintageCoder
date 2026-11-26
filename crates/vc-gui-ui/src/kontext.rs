// SPDX-License-Identifier: GPL-3.0-or-later
//! Kontextmenü (Rechtsklick): ein Menü-Widget für alle Stellen; die Einträge kommen aus der Befehlstabelle (gleiche Namen, Kürzel
//! und Ausgrauen wie Palette und Menü). Dazu ein kleiner Umschlag, der den Rechtsklick samt Mausposition meldet.
use crate::befehle::{self, Cmd};
use crate::theme::Pal;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{mouse, overlay, renderer, Clipboard, Shell, Widget};
use iced::widget::{button, column, container, row, space, text};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector};

/// Wo das Menü aufgeht
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Art { Baum, BaumFlaeche, Reiter, Editor, Flaeche, /** Aktionen des Studios im Reiter (Bildschirm, Kachelsatz, Bank, Palette, Disk): Einträge kommen aus `studioaktionen` */ FlaecheStudio, /** Rechtsklick auf eine Meldung unter „Probleme“ (Ziel: Nummer) */ Problem, /** Rechtsklick im „Protokoll“ */ Protokoll, /** Menüleiste im Fenster (Windows, Linux): die Einträge kommen aus der Befehlstabelle nach Menü */ Menue }

/// Einträge eines Menüs: `None` ist ein Trenner. Die Reihenfolge ist die der Anzeige.
pub fn eintraege(art: Art) -> &'static [Option<Cmd>] {
    use Cmd::*;
    match art {
        Art::BaumFlaeche => &[Some(NeueDatei), Some(NeuerOrdner), Some(NeueNotizDialog)],
        Art::Baum => &[Some(ZielOeffnen), None, Some(DateiUmbenennen), Some(DateiLoeschen), None, Some(AlsInclude), Some(PfadKopieren), Some(ImOrdnerZeigen), Some(HexAnsicht), None, Some(NeueDatei), Some(NeuerOrdner), Some(NeueNotizDialog)],
        Art::Reiter => &[Some(DateiSchliessen), None, Some(ReiterLinks), Some(ReiterRechts), Some(ReiterAndereHaelfte), None, Some(Teilen)],
        Art::Flaeche => &[Some(Rueckgaengig), Some(Wiederholen), None, Some(Ausschneiden), Some(Kopieren), Some(Einfuegen), None, Some(Exportieren)],
        Art::FlaecheStudio | Art::Menue => &[],
        Art::Problem => &[Some(ProblemSpringen), Some(ProblemKopieren), None, Some(ProblemeAlleKopieren)],
        Art::Protokoll => &[Some(ProtokollKopieren), Some(ProtokollLeeren)],
        Art::Editor => &[Some(Ausschneiden), Some(Kopieren), Some(Einfuegen), Some(AlleAuswaehlen), None, Some(Hilfe), Some(GeheZuDefinition), None, Some(AllesFalten), Some(AllesEntfalten), None, Some(HaltepunktUmschalten), Some(DebugHierher)],
    }
}
/// Menüname für Bildschirmleser
pub fn menuename(art: Art) -> String { match art { Art::Menue => vc_i18n::t!("kontext-menue"), Art::Problem => vc_i18n::t!("kontext-kontextmenue-problem"), Art::Protokoll => vc_i18n::t!("kontext-kontextmenue-protokoll"), Art::Baum => vc_i18n::t!("kontext-kontextmenue-datei"), Art::BaumFlaeche => vc_i18n::t!("kontext-kontextmenue-dateibaum"), Art::Reiter => vc_i18n::t!("kontext-kontextmenue-reiter"), Art::Editor => vc_i18n::t!("kontext-kontextmenue-editor"), Art::Flaeche => vc_i18n::t!("kontext-kontextmenue-zeichenflaeche"), Art::FlaecheStudio => vc_i18n::t!("kontext-kontextmenue-studio") } }
/// Kürzel, das im Menü steht: aus der Tabelle, bei den Textbearbeitungen die des Editors (sie sind in der Tabelle bewusst frei)
pub fn kuerzel(c: Cmd, mac: bool) -> String {
    let t = match c { Cmd::Ausschneiden => "Cmd+X", Cmd::Kopieren => "Cmd+C", Cmd::Einfuegen => "Cmd+V", Cmd::AlleAuswaehlen => "Cmd+A", _ => befehle::info(c).tasten };
    if t.is_empty() { String::new() } else { befehle::fuer_anzeige(t, mac) }
}
/// Name im Menü; „Hilfe“ heißt im Editor „Hilfe zum Wort“
pub fn eintragsname(c: Cmd) -> String { if c == Cmd::Hilfe { vc_i18n::t!("kontext-hilfe-zum-wort") } else { befehle::info(c).name() } }

/// Eine Menüzeile (Name, Kürzel, ob wählbar, was sie auslöst)
#[derive(Clone, Debug)]
pub struct Zeile<M> { pub name: String, pub kuerzel: String, pub aktiv: bool, pub msg: Option<M> }

/// Index der nächsten wählbaren Zeile ab `von` in Richtung `d` (Trenner haben `msg == None` und `name` leer); mit Umlauf
pub fn naechste<M>(zeilen: &[Zeile<M>], von: usize, d: i32) -> usize {
    let n = zeilen.len() as i32; if n == 0 { return 0 }
    let mut i = von as i32;
    for _ in 0..n { i = (i + d).rem_euclid(n); let z = &zeilen[i as usize]; if z.aktiv && !z.name.is_empty() { return i as usize } }
    von
}

/// Das Menü als Karte; `gewaehlt` ist die per Tastatur markierte Zeile
pub fn menue<'a, M: Clone + 'a>(pal: Pal, zeilen: Vec<Zeile<M>>, gewaehlt: usize) -> Element<'a, M> { menue_breit(pal, zeilen, gewaehlt, 260.0) }
/// Wie [`menue`] mit gewählter Breite (die Menüs der Menüleiste haben längere Namen mit Grund)
pub fn menue_breit<'a, M: Clone + 'a>(pal: Pal, zeilen: Vec<Zeile<M>>, gewaehlt: usize, breite: f32) -> Element<'a, M> {
    let mut spalte = column![].spacing(1);
    for (i, z) in zeilen.into_iter().enumerate() {
        if z.name.is_empty() { spalte = spalte.push(container(space::horizontal()).width(Length::Fill).height(1).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() })); continue }
        let (aktiv, farbe) = (z.aktiv, if z.aktiv { pal.text } else { pal.mute });
        let inhalt = row![text(z.name).size(13).color(farbe), space::horizontal(), text(z.kuerzel).size(11.5).color(pal.mute)].spacing(24);
        let mut b = button(inhalt).width(Length::Fill).padding([5, 10]).style(move |_, st| button::Style { background: (aktiv && (matches!(st, button::Status::Hovered | button::Status::Pressed) || i == gewaehlt)).then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: farbe, ..Default::default() });
        if let (true, Some(m)) = (aktiv, z.msg) { b = b.on_press(m) }
        spalte = spalte.push(b);
    }
    container(spalte).width(breite).padding(4).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 8.0.into() }, shadow: iced::Shadow { color: Color { a: 0.35, ..Color::BLACK }, offset: Vector::new(0.0, 4.0), blur_radius: 12.0 }, ..Default::default() }).into()
}

/// Meldet einen Rechtsklick auf den Inhalt samt Mausposition (Fensterkoordinaten); alles andere geht unverändert durch
pub struct RechtsKlick<'a, M> { inhalt: Element<'a, M>, f: Box<dyn Fn(Point) -> M + 'a> }
pub fn rechtsklick<'a, M: 'a>(inhalt: impl Into<Element<'a, M>>, f: impl Fn(Point) -> M + 'a) -> Element<'a, M> { Element::new(RechtsKlick { inhalt: inhalt.into(), f: Box::new(f) }) }

impl<'a, M> Widget<M, iced::Theme, iced::Renderer> for RechtsKlick<'a, M> {
    fn tag(&self) -> tree::Tag { self.inhalt.as_widget().tag() }
    fn state(&self) -> tree::State { self.inhalt.as_widget().state() }
    fn children(&self) -> Vec<Tree> { self.inhalt.as_widget().children() }
    fn diff(&self, t: &mut Tree) { self.inhalt.as_widget().diff(t) }
    fn size(&self) -> Size<Length> { self.inhalt.as_widget().size() }
    fn size_hint(&self) -> Size<Length> { self.inhalt.as_widget().size_hint() }
    fn layout(&mut self, t: &mut Tree, r: &iced::Renderer, l: &layout::Limits) -> layout::Node { self.inhalt.as_widget_mut().layout(t, r, l) }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &iced::Renderer, o: &mut dyn iced::advanced::widget::Operation) { self.inhalt.as_widget_mut().operate(t, l, r, o) }
    fn update(&mut self, t: &mut Tree, e: &Event, l: Layout<'_>, c: mouse::Cursor, r: &iced::Renderer, cb: &mut dyn Clipboard, s: &mut Shell<'_, M>, v: &Rectangle) {
        self.inhalt.as_widget_mut().update(t, e, l, c, r, cb, s, v);
        if s.is_event_captured() { return }
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = e { if let Some(p) = c.position_over(l.bounds()) { s.publish((self.f)(p)); s.capture_event() } }
    }
    fn mouse_interaction(&self, t: &Tree, l: Layout<'_>, c: mouse::Cursor, v: &Rectangle, r: &iced::Renderer) -> mouse::Interaction { self.inhalt.as_widget().mouse_interaction(t, l, c, v, r) }
    fn draw(&self, t: &Tree, r: &mut iced::Renderer, th: &iced::Theme, st: &renderer::Style, l: Layout<'_>, c: mouse::Cursor, v: &Rectangle) { self.inhalt.as_widget().draw(t, r, th, st, l, c, v) }
    fn overlay<'b>(&'b mut self, t: &'b mut Tree, l: Layout<'b>, r: &iced::Renderer, v: &Rectangle, tr: Vector) -> Option<overlay::Element<'b, M, iced::Theme, iced::Renderer>> { self.inhalt.as_widget_mut().overlay(t, l, r, v, tr) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn menues_bestehen_nur_aus_befehlen_der_tabelle() {
        for art in [Art::Baum, Art::BaumFlaeche, Art::Reiter, Art::Editor, Art::Flaeche] { let e = eintraege(art); assert!(e.iter().flatten().count() >= 3); for c in e.iter().flatten() { let _ = befehle::info(*c); } assert!(e.first().unwrap().is_some() && e.last().unwrap().is_some(), "kein Trenner am Rand") }
    }
    #[test] fn kuerzel_und_namen() {
        assert_eq!(kuerzel(Cmd::Kopieren, false), "Strg+C"); assert_eq!(kuerzel(Cmd::HaltepunktUmschalten, false), "F9"); assert_eq!(kuerzel(Cmd::DateiLoeschen, true), "");
        assert_eq!(eintragsname(Cmd::Hilfe), "Hilfe zum Wort"); assert_eq!(eintragsname(Cmd::DateiLoeschen), "Löschen …");
    }
    #[test] fn pfeiltasten_ueberspringen_trenner_und_ausgegrautes() {
        let z = |n: &str, a: bool| Zeile::<u8> { name: n.into(), kuerzel: String::new(), aktiv: a, msg: Some(1) };
        let zs = [z("a", true), z("", false), z("b", false), z("c", true)];
        assert_eq!(naechste(&zs, 0, 1), 3); assert_eq!(naechste(&zs, 3, 1), 0, "mit Umlauf"); assert_eq!(naechste(&zs, 0, -1), 3);
    }
}
