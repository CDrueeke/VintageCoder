// SPDX-License-Identifier: GPL-3.0-or-later
//! Bereichs-Widget: Kopfzeile eines Bereichs mit Minimieren, Maximieren und Wiederherstellen (Konzept Abschnitt 5.1).
//! Alle Bereiche (linke Spalte, Dokumentenfeld, Schublade, Panel, Studios) benutzen dieses eine Stück; die Knöpfe sind aus Rahmen
//! gezeichnet (kein Symbolschrift nötig) und tragen einen Hinweis mit dem Namen und dem Kürzel.
use crate::texte;
use crate::theme::Pal;
use iced::widget::{button, container, mouse_area, row, space, stack, text, tooltip};
use iced::{Alignment, Border, Color, Element, Length};
use vc_ide_model::layout::Anzeige;

/// Nachrichten der drei Knöpfe und des Doppelklicks
#[derive(Clone, Debug)]
pub struct Aktionen<M> { pub minimieren: M, pub maximieren: M, pub wiederherstellen: M }

/// Namen der Knöpfe für Hinweis und Bildschirmleser: (Minimieren, Maximieren oder Wiederherstellen)
pub fn knopfnamen(bereich: &str, anzeige: Anzeige) -> (String, String) {
    (texte::knopfname(bereich, "minimieren"), texte::knopfname(bereich, if anzeige == Anzeige::Maximiert { "wiederherstellen" } else { "maximieren" }))
}
/// Zustandstext für Bildschirmleser
pub fn zustandstext(anzeige: Anzeige) -> String { match anzeige { Anzeige::Normal => texte::normal(), Anzeige::Minimiert => texte::minimiert(), Anzeige::Maximiert => texte::maximiert() } }

fn rahmen(pal: Pal, breite: f32, hoehe: f32, gefuellt: bool) -> Element<'static, ()> {
    container(space::horizontal()).width(breite).height(hoehe).style(move |_| container::Style { background: gefuellt.then(|| pal.text.into()), border: Border { color: pal.text, width: if gefuellt { 0.0 } else { 1.4 }, radius: 1.0.into() }, ..Default::default() }).into()
}
fn symbol<'a, M: 'a>(pal: Pal, art: u8) -> Element<'a, M> {
    // 0 Minimieren (Strich), 1 Maximieren (Quadrat), 2 Wiederherstellen (zwei Quadrate)
    let e: Element<'a, ()> = match art {
        0 => container(rahmen(pal, 10.0, 2.0, true)).padding(iced::Padding { top: 9.0, ..Default::default() }).into(),
        1 => rahmen(pal, 10.0, 10.0, false),
        _ => stack![container(rahmen(pal, 9.0, 9.0, false)).padding(iced::Padding { left: 3.0, ..Default::default() }), container(rahmen(pal, 9.0, 9.0, false)).padding(iced::Padding { top: 3.0, ..Default::default() })].width(13).height(13).into(),
    };
    e.map(|_| unreachable!())
}
fn knopf<'a, M: Clone + 'a>(pal: Pal, art: u8, name: String, kuerzel: &str, auf: M) -> Element<'a, M> {
    let inhalt = container(symbol::<M>(pal, art)).center_x(Length::Fixed(24.0)).center_y(Length::Fixed(24.0));
    let b = button(inhalt).padding(0).on_press(auf).style(move |_, st| button::Style { background: Some(if matches!(st, button::Status::Hovered | button::Status::Pressed) { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() });
    let hinweis: Element<'a, M> = container(iced::widget::column![text(name).size(12).color(pal.text), text(crate::befehle::plattform_text(kuerzel)).size(11).color(pal.mute)].spacing(1)).padding([5, 8]).style(move |_| container::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, ..Default::default() }).into();
    tooltip(b, hinweis, tooltip::Position::Bottom).into()
}

/// Kopfzeile eines Bereichs: `links` (Titel oder Reiter) und rechts die Knöpfe. `kann_min`: das Dokumentenfeld hat kein Minimieren.
/// Doppelklick auf die Kopfzeile schaltet Maximieren um. `kuerzel_max` steht im Hinweis (z. B. „Cmd+Umschalt+Enter“).
pub fn kopfzeile<'a, M: Clone + 'a>(pal: Pal, bereich: &str, anzeige: Anzeige, links: Element<'a, M>, kann_min: bool, kuerzel_min: &str, kuerzel_max: &str, a: Aktionen<M>) -> Element<'a, M> {
    let (n_min, n_max) = knopfnamen(bereich, anzeige);
    let mut knoepfe = row![].spacing(4).align_y(Alignment::Center);
    if kann_min && anzeige != Anzeige::Minimiert { knoepfe = knoepfe.push(knopf(pal, 0, n_min, kuerzel_min, a.minimieren.clone())) }
    knoepfe = knoepfe.push(match anzeige {
        Anzeige::Maximiert => knopf(pal, 2, n_max, kuerzel_max, a.wiederherstellen.clone()),
        _ => knopf(pal, 1, n_max, kuerzel_max, a.maximieren.clone()),
    });
    let zeile = row![container(links).width(Length::Fill), knoepfe].spacing(8).align_y(Alignment::Center);
    let umschalten = if anzeige == Anzeige::Maximiert { a.wiederherstellen } else { a.maximieren };
    mouse_area(container(zeile).padding([4, 8]).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: Color::TRANSPARENT, width: 0.0, radius: 0.0.into() }, ..Default::default() })).on_double_click(umschalten).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn namen_nennen_bereich_und_aktion() {
        assert_eq!(knopfnamen("Schublade", Anzeige::Normal), ("Schublade minimieren".into(), "Schublade maximieren".into()));
        assert_eq!(knopfnamen("Panel", Anzeige::Maximiert).1, "Panel wiederherstellen");
        assert_eq!((zustandstext(Anzeige::Minimiert), zustandstext(Anzeige::Maximiert)), ("minimiert".to_string(), "maximiert".to_string()));
    }
    #[test] fn kopfzeile_laesst_sich_bauen() {
        let _e: Element<'_, u8> = kopfzeile(Pal::dark(), "Schublade", Anzeige::Normal, text("x").into(), true, "Cmd+J", "Cmd+Umschalt+Enter", Aktionen { minimieren: 1, maximieren: 2, wiederherstellen: 3 });
    }
}
