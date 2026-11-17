// SPDX-License-Identifier: GPL-3.0-or-later
//! Markdown-Ansicht: ein Dokument (`vc_md::Doc`) als Widgets (Überschriften, Absätze mit Auszeichnung, Listen, Aufgaben, Zitate, Code, Tabellen, Linien). Nur Anzeige, ohne Netz:
//! Bilder erscheinen als Platzhaltertext, Links sind farbig und unterstrichen und öffnen nichts von allein. Große Dokumente (zum Beispiel die Lizenzhinweise) werden in
//! Blöcken gezeigt („Weitere Blöcke anzeigen“), damit der Aufbau schnell bleibt; das geparste Dokument liegt im Zwischenspeicher der App.
use super::*;
use iced::widget::{column, rich_text, span};
use vc_md::{Align, Block, Doc, Inline, Node};

/// Wie viele Blöcke zuerst gezeigt werden und um wie viele „Weitere Blöcke anzeigen“ erweitert
pub const BLOECKE: usize = 20;

thread_local! { static SCHMAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) } }
/// Wie [`ansicht`], aber für schmale Spalten (Hilfe-Panel): Tabellen werden zu gestapelten Karten (erste Spalte als Titel, danach „Spalte: Wert“), damit nichts überlappt
pub fn ansicht_schmal<'a>(pal: Pal, doc: &Doc, grenze: usize, mehr: Option<Msg>) -> Element<'a, Msg> { SCHMAL.with(|f| f.set(true)); let e = ansicht(pal, doc, grenze, mehr); SCHMAL.with(|f| f.set(false)); e }

type Sp = iced::widget::text::Span<'static, String, iced::Font>;

fn spannen(pal: Pal, k: &[Inline], groesse: f32, farbe: Color, fett: bool, out: &mut Vec<Sp>) {
    let font = |f: bool| if f { vc_gui_ui::fonts::UI_FETT } else { vc_gui_ui::fonts::UI };
    for i in k {
        match i {
            Inline::Text(t) => out.push(span(t.clone()).size(groesse).color(farbe).font(font(fett))),
            Inline::Code(t) => out.push(span(format!(" {t} ")).font(vc_gui_ui::fonts::MONO).size(groesse - 0.5).color(pal.lab).background(pal.editor)),
            Inline::Strong(x) => spannen(pal, x, groesse, farbe, true, out),
            Inline::Emph(x) => spannen(pal, x, groesse, pal.text, fett, out),
            Inline::Strike(x) => { let mut v = vec![]; spannen(pal, x, groesse, pal.mute, fett, &mut v); out.extend(v.into_iter().map(|s| s.strikethrough(true))) }
            Inline::Link { kids, url, .. } => { let mut v = vec![]; spannen(pal, kids, groesse, pal.accent, fett, &mut v); let u = url.clone(); out.extend(v.into_iter().map(move |s| s.underline(true).link(u.clone()))) }
            Inline::Image { alt, .. } => out.push(span(format!("[Bild: {}]", if alt.is_empty() { "ohne Beschreibung" } else { alt })).size(groesse).color(pal.mute)),
            Inline::SoftBreak => out.push(span(" ").size(groesse)),
            Inline::HardBreak => out.push(span("\n").size(groesse)),
        }
    }
}
fn absatz<'a>(pal: Pal, k: &[Inline], groesse: f32, farbe: Color, fett: bool) -> Element<'a, Msg> { let mut v = vec![]; spannen(pal, k, groesse, farbe, fett, &mut v); rich_text(v).on_link_click(Msg::MdLink).into() }

fn block<'a>(pal: Pal, n: &Node, tiefe: usize) -> Element<'a, Msg> {
    match &n.block {
        Block::Heading { level, text: t, .. } => { let (g, abstand) = match level { 1 => (21.0, 6.0), 2 => (17.0, 4.0), 3 => (15.0, 2.0), 4 => (13.5, 0.0), _ => (13.0, 0.0) }; container(absatz(pal, t, g, pal.text, true)).padding(iced::Padding { top: abstand * 2.0, ..Default::default() }).into() }
        Block::Para(k) => absatz(pal, k, 13.0, pal.text, false),
        Block::Quote(ns) => { let inhalt = column(ns.iter().map(|x| block(pal, x, tiefe + 1)).collect::<Vec<_>>()).spacing(6); row![container(space::horizontal()).width(3).height(Length::Fill).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() }), container(inhalt).padding([0, 10])].height(Length::Shrink).into() }
        Block::List { start, items } => {
            let mut c = column![].spacing(4);
            for (i, it) in items.iter().enumerate() {
                let marke = match (it.task, start) { (Some(true), _) => "☑".to_string(), (Some(false), _) => "☐".to_string(), (None, Some(s)) => format!("{}.", s + i as u64), (None, None) => "•".to_string() };
                let inhalt = column(it.nodes.iter().map(|x| block(pal, x, tiefe + 1)).collect::<Vec<_>>()).spacing(3);
                c = c.push(row![container(text(marke).size(13).color(if it.task.is_some() { pal.accent } else { pal.mute })).width(22), inhalt].spacing(4));
            }
            container(c).padding(iced::Padding { left: 6.0 + 14.0 * tiefe as f32, ..Default::default() }).into()
        }
        Block::Code { text: t, .. } => container(text(t.trim_end_matches('\n').to_string()).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text)).padding([8, 10]).width(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, ..Default::default() }).into(),
        Block::Rule => container(space::horizontal()).width(Length::Fill).height(1).style(move |_| container::Style { background: Some(pal.line.into()), ..Default::default() }).into(),
        Block::Html(t) => text(t.clone()).size(11.5).font(vc_gui_ui::fonts::MONO).color(pal.mute).into(),
        Block::Table { head, rows, .. } if SCHMAL.with(|f| f.get()) => {
            let mut t = column![].spacing(6);
            for r in rows {
                let mut k = column![].spacing(2);
                for (i, c) in r.iter().enumerate() {
                    if i == 0 { k = k.push(absatz(pal, c, 13.0, pal.text, true)) }
                    else { let mut v: Vec<Sp> = vec![]; if let Some(h) = head.get(i) { spannen(pal, h, 11.5, pal.mute, false, &mut v); v.push(span(": ").size(11.5).color(pal.mute)) } spannen(pal, c, 12.5, pal.text, false, &mut v); k = k.push(rich_text(v).on_link_click(Msg::MdLink)) }
                }
                t = t.push(container(k).padding([6, 8]).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 0.5, radius: 6.0.into() }, ..Default::default() }));
            }
            t.into()
        }
        Block::Table { head, rows, align } => {
            let spalten = head.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
            let zelle = |k: &Vec<Inline>, kopf: bool, i: usize| -> Element<'a, Msg> { let a = align.get(i).copied().unwrap_or(Align::None); container(absatz(pal, k, if kopf { 12.0 } else { 12.5 }, if kopf { pal.mute } else { pal.text }, kopf)).padding([4, 8]).width(Length::FillPortion(1)).align_x(match a { Align::Right => iced::alignment::Horizontal::Right, Align::Center => iced::alignment::Horizontal::Center, _ => iced::alignment::Horizontal::Left }).into() };
            let mut t = column![].spacing(0);
            t = t.push(container(row((0..spalten).map(|i| zelle(&head.get(i).cloned().unwrap_or_default(), true, i)).collect::<Vec<_>>())).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() }));
            for (ri, r) in rows.iter().enumerate() { t = t.push(container(row((0..spalten).map(|i| zelle(&r.get(i).cloned().unwrap_or_default(), false, i)).collect::<Vec<_>>())).style(move |_| container::Style { background: (ri % 2 == 1).then(|| Color { a: 0.5, ..pal.editor }.into()), border: Border { color: pal.line, width: 0.5, radius: 0.0.into() }, ..Default::default() })) }
            container(t).width(Length::Fill).style(move |_| container::Style { border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, ..Default::default() }).into()
        }
    }
}

/// Die ersten `grenze` Blöcke; darunter, wenn mehr da sind, „Weitere Blöcke anzeigen“ (`mehr` liefert die Nachricht dafür)
pub fn ansicht<'a>(pal: Pal, doc: &Doc, grenze: usize, mehr: Option<Msg>) -> Element<'a, Msg> {
    let n = doc.nodes.len(); let mut c = column![].spacing(10).width(Length::Fill);
    for x in doc.nodes.iter().take(grenze) { c = c.push(block(pal, x, 0)) }
    if n > grenze { if let Some(m) = mehr { c = c.push(vc_gui_ui::muster::knopf(pal, &format!("Weitere Blöcke anzeigen ({} von {n} sichtbar)", grenze.min(n)), "", Some(m), false, false)) } }
    c.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ein_dokument_wird_zu_bloecken_und_die_grenze_begrenzt() {
        let md = "# Titel\n\nAbsatz mit **fett**, `code` und [Link](http://x).\n\n- eins\n- [x] fertig\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n```\nlda #1\n```\n\n---\n";
        let d = vc_md::parse(md); assert!(d.nodes.len() >= 6, "{:?}", d.nodes.len());
        assert!(d.nodes.iter().any(|n| matches!(n.block, Block::Table { .. })) && d.nodes.iter().any(|n| matches!(n.block, Block::Code { .. })) && d.nodes.iter().any(|n| matches!(&n.block, Block::List { items, .. } if items.iter().any(|i| i.task == Some(true)))));
    }
}
