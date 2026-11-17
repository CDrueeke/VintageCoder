// SPDX-License-Identifier: GPL-3.0-or-later
//! Studio-Muster (Entscheidung des Projektinhabers vom 2026-11-16, Konzept „UX-Überarbeitung“): die Bausteine, aus denen jedes Studio und jede
//! Seite gebaut wird. **Pflicht für neue Studios:** Wer ein Studio baut, benutzt [`studio_rahmen`] und [`karte`], keine eigenen Leisten.
//!
//! Sechs Zonen (Bild M1): 1 Kopfzeile (Art-Chip, Name, ● ungespeichert, Maße, rechts alle Datei-Aktionen), 2 Werkzeugleiste (genau eine Reihe),
//! 3 Liste (Frames, Kacheln, Zeichen, Muster), 4 Arbeitsfläche (eingepasst, ohne Scrollbalken), 5 Seitenspalte aus Karten
//! (Werkzeug · Dokument · Info), 6 Fußzeile. Maße im 4-px-Raster, Schrift nach der Leiter in [`schrift`], Farben nur aus [`Pal`].
//! Die Breitenklasse folgt der Breite des Dokumentenfelds ([`klasse`]), nicht der des Fensters.
use crate::theme::Pal;
use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Alignment, Border, Element, Length};

/// Zen-Modus: ist er an, zeigen die Studios nur Kopfzeile, Leiste, Fläche und Fußzeile (keine Liste, keine Seitenspalte). Der Schalter steht je Aufbau der Ansicht fest
/// (`zen_setzen` am Anfang von `view`), damit kein Studio ihn einzeln durchreichen muss.
thread_local! { static ZEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub fn zen_setzen(an: bool) { ZEN.with(|z| z.set(an)) }
pub fn zen() -> bool { ZEN.with(|z| z.get()) }

/// Abstände (4-px-Raster)
pub mod abstand { pub const ENG: f32 = 4.0; pub const GRUPPE: f32 = 8.0; pub const KARTE: f32 = 12.0; pub const ABSCHNITT: f32 = 16.0; pub const BEREICH: f32 = 24.0; pub const SEITENRAND: f32 = 32.0; }
/// Schriftleiter (Punkt)
pub mod schrift { pub const TITEL: f32 = 20.0; pub const ABSCHNITT: f32 = 15.0; pub const KARTE: f32 = 13.0; pub const TEXT: f32 = 13.0; pub const HINWEIS: f32 = 11.0; pub const WERT: f32 = 12.0; }
/// Maße der Zonen und Elemente
pub const KOPF_HOEHE: f32 = 36.0;
pub const LEISTE_HOEHE: f32 = 38.0;
pub const FUSS_HOEHE: f32 = 24.0;
pub const LISTE_BREITE: f32 = 130.0;
pub const SEITE_BREITE: f32 = 290.0;
pub const KNOPF_HOEHE: f32 = 28.0;
pub const WERKZEUG_KNOPF: f32 = 30.0;
/// Kleinste Arbeitsfläche; kleiner wechselt das Studio in die nächste Breitenklasse
pub const MIN_FLAECHE: (f32, f32) = (320.0, 240.0);
/// Radius von Knopf und Karte
pub const RADIUS_KNOPF: f32 = 6.0;
pub const RADIUS_KARTE: f32 = 8.0;

/// Breitenklasse des Studios (nach der Breite des Dokumentenfelds)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Klasse {
    /// ≥ 900 px: Liste · Fläche · Seitenspalte; Leiste mit sichtbaren Gruppen
    Breit,
    /// 640–899 px: Fläche · Seitenspalte; Liste als Streifen unter der Fläche; „Bearbeiten“ und „Ansicht“ in ⋯
    Mittel,
    /// < 640 px: eine Spalte: Fläche; Seitenspalte als Unterreiter darunter; Kopfzeile nur Name und Speichern
    Schmal,
}
/// Klasse zu einer Studiobreite (Pixel)
pub fn klasse(studio_breite: f32) -> Klasse { if studio_breite >= 900.0 { Klasse::Breit } else if studio_breite >= 640.0 { Klasse::Mittel } else { Klasse::Schmal } }
/// Studiobreite aus der Fensterbreite: Spalte links, Panel rechts und Ränder gehen ab (Näherung für die Wahl der Klasse; Zahlen prüfen)
pub fn studiobreite(fenster: f32, spalte: f32, panel: f32) -> f32 { (fenster - spalte - panel).max(0.0) }

fn rand(pal: Pal, breit: f32) -> Border { Border { color: pal.line, width: breit, radius: RADIUS_KARTE.into() } }

/// Zone 1: Kopfzeile. `art`: Kürzel des Medientyps (SPR, TRK, MAP …), `masse`: „Frame 0 von 2 · 12×21 · Multicolor“; `aktionen`: Datei-Aktionen (Exportieren …, Speichern, ⋯)
/// stehen **nur hier**. In der schmalen Klasse zeigt die Kopfzeile nur Name und die letzte Aktion (Speichern).
pub fn kopfzeile<'a, M: Clone + 'a>(pal: Pal, klasse: Klasse, art: &str, name: &str, ungespeichert: bool, masse: &str, aktionen: Vec<Element<'a, M>>) -> Element<'a, M> {
    let chip = container(text(art.to_string()).size(schrift::HINWEIS).color(pal.accent)).padding([2, 8]).style(move |_| container::Style { border: Border { color: pal.accent, width: 1.0, radius: RADIUS_KNOPF.into() }, ..Default::default() });
    let punkt: Element<'a, M> = if ungespeichert { text("●").size(schrift::KARTE).color(pal.warn).into() } else { space::horizontal().width(0).into() };
    let mut r = row![chip, text(name.to_string()).size(14.0).font(crate::fonts::UI_FETT).color(pal.text), punkt].spacing(abstand::GRUPPE).align_y(Alignment::Center);
    if klasse != Klasse::Schmal && !masse.is_empty() { r = r.push(text(masse.to_string()).size(schrift::HINWEIS).color(pal.mute)) }
    r = r.push(space::horizontal());
    // schmal: nur die letzte Aktion (Speichern); Exportieren und der Rest sind über Befehlspalette und Kürzel erreichbar
    let skip = if klasse == Klasse::Schmal { aktionen.len().saturating_sub(1) } else { 0 };
    for a in aktionen.into_iter().skip(skip) { r = r.push(a) }
    container(r.spacing(abstand::GRUPPE)).height(KOPF_HOEHE).padding([0, 12]).center_y(Length::Fixed(KOPF_HOEHE)).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() }).into()
}

/// Zone 2: Werkzeugleiste in genau einer Reihe (`inhalt`: Gruppen mit Trennern; was nicht passt, gehört in ⋯)
pub fn leiste<'a, M: 'a>(pal: Pal, inhalt: Element<'a, M>) -> Element<'a, M> {
    container(inhalt).height(LEISTE_HOEHE).padding([0, 8]).center_y(Length::Fixed(LEISTE_HOEHE)).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() }).into()
}
/// Senkrechter Trenner zwischen Gruppen der Leiste
pub fn trenner<'a, M: 'a>(pal: Pal) -> Element<'a, M> { container(space::horizontal()).width(1).height(20).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() }).into() }

/// Zone 6: Fußzeile (links Zelle/Zoom/Position, rechts „? Tasten“)
pub fn fusszeile<'a, M: 'a>(pal: Pal, links: &str, rechts: Element<'a, M>) -> Element<'a, M> {
    container(row![text(links.to_string()).size(schrift::HINWEIS).color(pal.mute), space::horizontal(), rechts].align_y(Alignment::Center)).height(FUSS_HOEHE).padding([0, 12]).center_y(Length::Fixed(FUSS_HOEHE)).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() }).into()
}

/// Karte der Seitenspalte: Überschrift mit Symbol, Unterzeile „gilt für …“, optional einklappbar (`einklappen`: (ist offen, Nachricht zum Umschalten)).
/// Der Zustand „offen“ gehört ins Modell des Studios, nie in den Inhalt.
pub fn karte<'a, M: Clone + 'a>(pal: Pal, titel: &str, symbol: &str, unterzeile: &str, einklappen: Option<(bool, M)>, inhalt: Element<'a, M>) -> Element<'a, M> {
    let offen = einklappen.as_ref().map_or(true, |e| e.0);
    let kopf = row![text(symbol.to_string()).size(schrift::KARTE).color(pal.accent), text(titel.to_string()).size(schrift::KARTE).font(crate::fonts::UI_FETT).color(pal.text), space::horizontal(),
        text(if einklappen.is_none() { "" } else if offen { "▾" } else { "▸" }).size(schrift::HINWEIS).color(pal.mute)].spacing(abstand::GRUPPE).align_y(Alignment::Center);
    let kopf: Element<'a, M> = match einklappen { Some((_, m)) => button(kopf).padding(0).width(Length::Fill).on_press(m).style(|_, _| button::Style::default()).into(), None => kopf.into() };
    let mut c = column![kopf].spacing(abstand::ENG);
    if !unterzeile.is_empty() { c = c.push(text(unterzeile.to_string()).size(schrift::HINWEIS).color(pal.mute)) }
    if offen { c = c.push(container(inhalt).padding(iced::Padding { top: abstand::GRUPPE, ..Default::default() })) }
    container(c).padding(abstand::KARTE).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: rand(pal, if pal.hoch { 2.0 } else { 1.0 }), ..Default::default() }).into()
}

/// Der Knopf der Studios (Entscheidung „Qualitätsmaßstab“): Höhe 28, Radius 6, Text 13; `aktiv` = gewählt (Füllung und Rahmen in Akzent), `primaer` = Hauptknopf (grün, Text auf Grün);
/// `wirkt: None` = nicht verfügbar (gedämpft). Der Tooltip nennt die Wirkung und das Kürzel.
pub fn knopf<'a, M: Clone + 'a>(pal: Pal, beschriftung: &str, tipp: &str, wirkt: Option<M>, aktiv: bool, primaer: bool) -> Element<'a, M> {
    let farbe = if primaer && wirkt.is_some() { pal.accent_text() } else if wirkt.is_some() { pal.text } else { pal.mute };
    knopf_el(pal, text(beschriftung.to_string()).size(schrift::TEXT).color(farbe).wrapping(iced::widget::text::Wrapping::None).into(), tipp, wirkt, aktiv, primaer, false)
}
/// Wie [`knopf`], aber mit beliebigem Inhalt (Symbol, Farbfläche); `quadrat`: Breite = Höhe (Werkzeugknöpfe mit Symbol)
pub fn knopf_el<'a, M: Clone + 'a>(pal: Pal, inhalt: Element<'a, M>, tipp: &str, wirkt: Option<M>, aktiv: bool, primaer: bool, quadrat: bool) -> Element<'a, M> {
    let an = wirkt.is_some(); let farbe = if primaer && an { pal.accent_text() } else if an { pal.text } else { pal.mute };
    let inner: Element<'a, M> = if quadrat { container(inhalt).center(Length::Fill).into() } else { container(inhalt).center_y(Length::Fill).into() };
    let mut b = button(inner).height(KNOPF_HOEHE).padding(if quadrat { [0, 0] } else { [0, 10] })
        .style(move |_, st| { let hover = matches!(st, button::Status::Hovered);
            button::Style { background: Some(if primaer && an { pal.ok.into() } else if aktiv { pal.sel.into() } else if hover && an { pal.line.into() } else { pal.panel2.into() }),
                border: Border { color: if primaer && an { pal.ok } else if aktiv { pal.accent } else { pal.line }, width: if pal.hoch { 2.0 } else { 1.0 }, radius: RADIUS_KNOPF.into() }, text_color: farbe, ..Default::default() } });
    if quadrat { b = b.width(KNOPF_HOEHE + 4.0) }
    if let Some(m) = wirkt { b = b.on_press(m) }
    if tipp.is_empty() { return b.into() }
    iced::widget::tooltip(b, container(text(tipp.to_string()).size(schrift::HINWEIS).color(pal.text)).padding([3, 7]).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 4.0.into() }, ..Default::default() }), iced::widget::tooltip::Position::Bottom).gap(4).into()
}

/// Zone 5: Seitenspalte aus Karten (feste Breite, Abstand 8); scrollt nur, wenn die Karten nicht passen: der einzige Scrollbereich des Studios
pub fn seitenspalte<'a, M: 'a>(klasse: Klasse, karten: Vec<Element<'a, M>>) -> Element<'a, M> {
    // Platz für den Rollbalken (10 px) bleibt frei, damit nichts abgeschnitten wird
    let breite = if klasse == Klasse::Schmal { Length::Fill } else { Length::Fixed(SEITE_BREITE) };
    let inhalt = container(column(karten).spacing(abstand::GRUPPE).width(Length::Fill)).padding(iced::Padding { top: abstand::GRUPPE, right: abstand::GRUPPE + 10.0, bottom: abstand::GRUPPE, left: abstand::GRUPPE });
    container(scrollable(inhalt).height(Length::Fill)).width(breite).height(Length::Fill).into()
}

/// Abschnitt einer Seite (Projekt-Einstellungen, Einstellungen): Überschrift (15 fett) und eine Zeile Beschreibung **direkt** über dem Inhalt (Abstand 8)
pub fn abschnitt<'a, M: 'a>(pal: Pal, titel: &str, beschreibung: &str, inhalt: Element<'a, M>) -> Element<'a, M> {
    let mut c = column![text(titel.to_string()).size(schrift::ABSCHNITT).font(crate::fonts::UI_FETT).color(pal.text)].spacing(abstand::ENG);
    if !beschreibung.is_empty() { c = c.push(text(beschreibung.to_string()).size(schrift::HINWEIS).color(pal.mute)) }
    c.push(container(inhalt).padding(iced::Padding { top: abstand::GRUPPE - abstand::ENG, ..Default::default() })).into()
}

/// Zeile in der Karte: Beschriftung links (96 px in der 290-px-Seitenspalte; 130 px auf Seiten), Wert rechts
pub fn zeile<'a, M: 'a>(pal: Pal, beschriftung: &str, wert: Element<'a, M>) -> Element<'a, M> {
    row![text(beschriftung.to_string()).size(schrift::TEXT).color(pal.mute).width(96), wert].spacing(abstand::GRUPPE).align_y(Alignment::Center).into()
}

/// Segment-Auswahl (Auswahlkette mit sichtbar gewähltem Eintrag, Zustand nicht nur als Farbe: gewählt = Füllung und Rahmen)
pub fn segment<'a, M: Clone + 'a>(pal: Pal, optionen: &[(&str, &str)], aktiv: &str, wahl: impl Fn(String) -> M) -> Element<'a, M> {
    row(optionen.iter().map(|(id, label)| {
        let an = *id == aktiv;
        button(text(label.to_string()).size(schrift::TEXT).color(if an { pal.accent_text() } else { pal.text })).padding([4, 10]).height(KNOPF_HOEHE - 2.0).on_press(wahl(id.to_string()))
            .style(move |_, _| button::Style { background: Some(if an { pal.accent.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if pal.hoch { 2.0 } else { 1.0 }, radius: RADIUS_KNOPF.into() }, text_color: pal.text, ..Default::default() }).into()
    }).collect::<Vec<Element<'a, M>>>()).spacing(abstand::ENG).wrap().into()
}

/// Die Teile eines Studios, geordnet nach den Zonen
pub struct Studio<'a, M> {
    /// Breite der Liste in der breiten Klasse (Standard `LISTE_BREITE`; Zeichensatz-Raster braucht mehr)
    pub liste_breite: f32,
    pub kopf: Element<'a, M>, pub leiste: Element<'a, M>, pub liste: Option<Element<'a, M>>, pub flaeche: Element<'a, M>, pub seite: Element<'a, M>, pub fuss: Element<'a, M>,
}

/// Setzt ein Studio aus den Zonen zusammen; die Breitenklasse bestimmt die Anordnung (Bild M4). Höchstens ein Scrollbereich: die Seitenspalte (nur wenn die Karten nicht passen)
pub fn studio_rahmen<'a, M: 'a>(pal: Pal, klasse: Klasse, s: Studio<'a, M>) -> Element<'a, M> {
    let flaeche = container(s.flaeche).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.bg.into()), ..Default::default() });
    let liste = s.liste.map(|l| container(l).padding(abstand::GRUPPE));
    let mitte: Element<'a, M> = if zen() { flaeche.into() } else { match klasse {
        Klasse::Breit => {
            let mut r = row![].height(Length::Fill);
            if let Some(l) = liste { r = r.push(container(l).width(s.liste_breite + 2.0 * abstand::GRUPPE).height(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() })) }
            r.push(flaeche).push(s.seite).into()
        }
        Klasse::Mittel => {
            let links: Element<'a, M> = match liste { Some(l) => column![flaeche, container(l).width(Length::Fill)].into(), None => flaeche.into() };
            row![container(links).width(Length::Fill).height(Length::Fill), s.seite].height(Length::Fill).into()
        }
        Klasse::Schmal => column![container(flaeche).height(Length::FillPortion(3)), container(s.seite).height(Length::FillPortion(2))].height(Length::Fill).into(),
    } };
    column![s.kopf, s.leiste, container(mitte).height(Length::Fill), s.fuss].width(Length::Fill).height(Length::Fill).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn breitenklassen_nach_studiobreite() {
        assert_eq!((klasse(1200.0), klasse(900.0), klasse(899.9), klasse(640.0), klasse(639.9), klasse(0.0)), (Klasse::Breit, Klasse::Breit, Klasse::Mittel, Klasse::Mittel, Klasse::Schmal, Klasse::Schmal));
        assert_eq!(studiobreite(1280.0, 260.0, 38.0), 982.0); assert_eq!(klasse(studiobreite(1280.0, 260.0, 38.0)), Klasse::Breit);
        assert_eq!(klasse(studiobreite(900.0, 260.0, 38.0)), Klasse::Schmal, "900 px Fenster ≈ 602 px Studio: schmal");
        assert_eq!(studiobreite(100.0, 260.0, 38.0), 0.0);
    }
    #[test]
    fn maße_liegen_im_vier_pixel_raster() {
        for v in [abstand::ENG, abstand::GRUPPE, abstand::KARTE, abstand::ABSCHNITT, abstand::BEREICH, abstand::SEITENRAND, KOPF_HOEHE, LEISTE_HOEHE, FUSS_HOEHE, KNOPF_HOEHE, SEITE_BREITE] { assert_eq!(v % 2.0, 0.0, "{v}"); }
        for v in [abstand::ENG, abstand::GRUPPE, abstand::KARTE, abstand::ABSCHNITT, abstand::BEREICH, abstand::SEITENRAND, KOPF_HOEHE, FUSS_HOEHE, KNOPF_HOEHE] { assert_eq!(v % 4.0, 0.0, "{v}"); }
        assert!(schrift::TITEL > schrift::ABSCHNITT && schrift::ABSCHNITT > schrift::KARTE && schrift::KARTE <= schrift::TEXT && schrift::TEXT > schrift::HINWEIS);
        assert!(SEITE_BREITE >= 240.0 && SEITE_BREITE <= 320.0 && LISTE_BREITE >= 110.0 && LISTE_BREITE <= 160.0);
    }
}
