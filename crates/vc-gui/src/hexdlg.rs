// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialog „BASIC: Hex-Vorschläge für Adressen“: listet Dezimaladressen in POKE/PEEK/SYS/WAIT/USR, die ein Chipregister des Zielsystems sind, als `$D020` zur Auswahl. **Jede Zeile
//! einzeln, nichts ungefragt**: ohne Haken ändert sich nichts. Die Erkennung steht in `vc_ide::hexhint`, das Ersetzen in `apply_suggestions`.
use crate::setui::{abschnitt, klein};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{checkbox, column, container, row, scrollable, text};
use iced::{Border, Color, Element, Length, Task};
use vc_ide::hexhint::Suggestion;

#[derive(Clone, Debug)]
pub enum HAkt { Zeile(usize, bool), Alle, Keine, Ok, Abbruch }

pub struct HexDlg { pub rel: String, pub vorschlaege: Vec<Suggestion>, pub gewaehlt: Vec<bool>, pub zeilen: Vec<String> }

impl App {
    pub(crate) fn hexv_oeffnen(&mut self) {
        let Some(rel) = self.name().filter(|n| self.editoren.contains_key(n) && (n.to_lowercase().ends_with(".bas") || n.to_lowercase().ends_with(".sbas"))) else { self.log.push("Hex-Vorschläge: öffne zuerst eine BASIC-Datei.".into()); return };
        let sys = self.system_id(); let dialekt = vc_target::load_target(&self.data, &sys).ok().and_then(|t| t.basic_dialect).unwrap_or_else(|| "cbm_v2".into());
        let text = self.editoren[&rel].text();
        match vc_ide::hexhint::suggestions(&self.data, &dialekt, &sys, &text) {
            Ok(v) if v.is_empty() => self.log.push("Hex-Vorschläge: keine Dezimaladresse eines bekannten Registers gefunden.".into()),
            Ok(v) => { let zeilen: Vec<String> = text.lines().map(String::from).collect(); self.hex = Some(HexDlg { rel, gewaehlt: vec![false; v.len()], vorschlaege: v, zeilen }) }
            Err(e) => self.log.push(format!("Hex-Vorschläge: {e}")),
        }
    }
    pub(crate) fn hexv_akt(&mut self, a: HAkt) -> Task<Msg> {
        let Some(d) = self.hex.as_mut() else { return Task::none() };
        match a {
            HAkt::Zeile(i, b) => { if let Some(x) = d.gewaehlt.get_mut(i) { *x = b } }
            HAkt::Alle => d.gewaehlt.iter_mut().for_each(|x| *x = true), HAkt::Keine => d.gewaehlt.iter_mut().for_each(|x| *x = false),
            HAkt::Abbruch => self.hex = None,
            HAkt::Ok => {
                let (rel, sel, vs) = (d.rel.clone(), d.gewaehlt.clone(), d.vorschlaege.clone()); self.hex = None;
                if !sel.iter().any(|b| *b) { return Task::none() }
                if let Some(e) = self.editoren.get_mut(&rel) {
                    let alt = e.text(); let neu = vc_ide::hexhint::apply_suggestions(&alt, &vs, &sel);
                    if neu != alt { let ende = vc_gui_editor::core::Pos { line: e.lines.len() - 1, col: e.line_len(e.lines.len() - 1) }; e.replace(vc_gui_editor::core::Pos { line: 0, col: 0 }, ende, &neu); self.log.push(format!("{} Adresse(n) als Hex geschrieben ({rel}); Rückgängig macht es in einem Schritt rückgängig.", sel.iter().filter(|b| **b).count())) }
                }
            }
        }
        Task::none()
    }
    pub(crate) fn hexv_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.hex.as_ref()?; let h = |a: HAkt| Msg::HexVorschlag(a); let mono = vc_gui_ui::fonts::MONO;
        let mut l = column![].spacing(4);
        for (i, v) in d.vorschlaege.iter().enumerate() {
            let zeile = d.zeilen.get(v.line).cloned().unwrap_or_default();
            l = l.push(row![checkbox(d.gewaehlt[i]).on_toggle(move |b| Msg::HexVorschlag(HAkt::Zeile(i, b))).size(16), text(format!("Zeile {}", v.line + 1)).size(11.5).color(pal.mute).width(70), text(format!("{} → {}", v.from, v.to)).size(12.5).font(mono).color(pal.accent).width(150), text(zeile).size(11.5).font(mono).color(pal.text)].spacing(8).align_y(iced::Alignment::Center));
        }
        let n = d.gewaehlt.iter().filter(|b| **b).count();
        let c = column![abschnitt(pal, "BASIC: Hex-Vorschläge für Adressen", "Dezimaladressen bekannter Register als Hex schreiben. Es wird nur geändert, was du ankreuzt."), container(scrollable(l).height(Length::Fixed(260.0))).padding(6),
            row![knopf(pal, "Alle", Some(h(HAkt::Alle)), pal.text, false), knopf(pal, "Keine", Some(h(HAkt::Keine)), pal.text, false), iced::widget::space::horizontal(), knopf(pal, "Abbrechen", Some(h(HAkt::Abbruch)), pal.text, false), knopf(pal, &format!("{n} übernehmen"), (n > 0).then(|| h(HAkt::Ok)), pal.text, true)].spacing(8),
            klein(pal, "Rechenausdrücke, DATA-Zeilen und Zeichenketten kommen nie vor.")].spacing(10);
        let karte = container(c).width(760).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hg = iced::widget::mouse_area(container(iced::widget::space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.55, ..Color::BLACK }.into()), ..Default::default() }));
        Some(iced::widget::stack![hg, container(karte).center(Length::Fill)].into())
    }
}
