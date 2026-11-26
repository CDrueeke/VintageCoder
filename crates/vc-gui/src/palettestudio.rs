// SPDX-License-Identifier: GPL-3.0-or-later
//! Paletten-Studio (MEGA65: 256 Farben, 8 Bit je Kanal): Farbraster, Regler für Rot, Grün und Blau, Hexwert, Standardpalette.
//! Das Modell ist `vc_ide_model::palettedoc::PaletteDoc`.
use super::*;
use iced::widget::{slider, text};
use vc_gui_ui::muster;
use vc_ide_model::palettedoc::{hex_zu_rgb, PaletteDoc};

#[derive(Clone, Debug, PartialEq)]
pub enum PAkt { Wahl(usize), Kanal(usize, u8), Fertig, Hex(String), HexOk, Undo, Redo, Standard, Speichern, Exportieren }

/// Art der Mediendatei (`kind`), `None` wenn die Datei kein Medium ist
pub fn medienart(pfad: &std::path::Path) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(pfad).ok()?).ok()?; v["kind"].as_str().map(String::from)
}

impl App {
    pub(crate) fn palette_oeffnen(&mut self, rel: &str) {
        if !self.paletten.contains_key(rel) {
            match PaletteDoc::open(&self.root, rel) { Ok(d) => { self.pal_hex = d.eintrag(0); self.paletten.insert(rel.to_string(), d); } Err(e) => { self.log.push(e); return } }
        }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }
    pub(crate) fn palette_aktion(&mut self, rel: &str, a: PAkt) {
        let Some(d) = self.paletten.get_mut(rel) else { return };
        match a {
            PAkt::Wahl(i) => { d.fertig(); d.sel = i.min(d.anzahl().saturating_sub(1)); self.pal_hex = d.eintrag(d.sel) }
            PAkt::Kanal(k, v) => { d.kanal(k, v); self.pal_hex = d.eintrag(d.sel) }
            PAkt::Fertig => d.fertig(),
            PAkt::Hex(h) => self.pal_hex = h,
            PAkt::HexOk => { if let Some(c) = hex_zu_rgb(&self.pal_hex) { let i = d.sel; d.setze(i, c); d.fertig() } self.pal_hex = d.eintrag(d.sel) }
            PAkt::Undo => { d.undo(); self.pal_hex = d.eintrag(d.sel) } PAkt::Redo => { d.redo(); self.pal_hex = d.eintrag(d.sel) }
            PAkt::Standard => { d.standard(); self.pal_hex = d.eintrag(d.sel) }
            PAkt::Speichern => match d.save() { Ok(()) => self.log.push(vc_i18n::t!("palettestudio-gespeichert-rel", rel = &rel)), Err(e) => self.log.push(e) },
            PAkt::Exportieren => self.export_oeffnen(),
        }
    }
    pub(crate) fn farbpalette_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.farbpalette_ansicht_k(pal, &rel, muster::klasse(size.width))).into()
    }
    fn farbpalette_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse) -> Element<'_, Msg> {
        let Some(d) = self.paletten.get(rel) else { return text("").into() }; let r = rel.to_string();
        let knopf_p = |name: &str, tipp: &str, an: bool, a: PAkt| -> Element<'static, Msg> { muster::knopf(pal, name, tipp, an.then(|| Msg::PaletteAkt(r.clone(), a)), false, false) };
        // Zone 2: genau eine Leiste (Bearbeiten · Standardpalette); Datei-Aktionen stehen nur in der Kopfzeile
        let leiste = muster::leiste(pal, iced::widget::row![knopf_p("↶", &vc_i18n::t!("palettestudio-rueckgaengig-cmd-z"), d.kann_undo(), PAkt::Undo), knopf_p("↷", &vc_i18n::t!("palettestudio-wiederholen-cmd-umschalt-z"), d.kann_redo(), PAkt::Redo), muster::trenner(pal), knopf_p(&vc_i18n::t!("palettestudio-standardpalette"), &vc_i18n::t!("palettestudio-auf-die-standardpalette-zuruecksetzen-alle-aktionen"), true, PAkt::Standard)].spacing(8).align_y(Alignment::Center).into());
        // Zone 4: das Farbraster füllt den freien Platz (quadratische Felder, ganzzahlig)
        let n = d.anzahl(); let zeilen_n = n.div_ceil(16).max(1); let sel = d.sel; let rel_c = r.clone();
        let flaeche_el: Element<Msg> = iced::widget::responsive(move |size| {
            let k = (((size.width - 24.0 - 28.0 - 2.0 * 15.0) / 16.0).min((size.height - 24.0 - 2.0 * (zeilen_n as f32 - 1.0)) / zeilen_n as f32)).floor().clamp(4.0, 56.0);
            let mut raster = iced::widget::column![].spacing(2);
            for zeile in 0..zeilen_n {
                let mut z = iced::widget::row![].spacing(2).align_y(Alignment::Center); if k >= 13.0 { z = z.push(text(format!("{:02X}", zeile * 16)).size(10).font(vc_gui_ui::fonts::MONO).color(pal.mute).width(24)) }
                for i in zeile * 16..((zeile + 1) * 16).min(n) {
                    let c = self.paletten.get(&rel_c).map_or([0, 0, 0], |d| d.rgb(i)); let farbe = Color::from_rgb8(c[0], c[1], c[2]); let wahl = i == sel;
                    z = z.push(button(space::vertical()).width(k).height(k).padding(0).on_press(Msg::PaletteAkt(rel_c.clone(), PAkt::Wahl(i))).style(move |_, _| button::Style { background: Some(farbe.into()), border: Border { color: if wahl { pal.accent } else { pal.line }, width: if wahl { 3.0 } else { 1.0 }, radius: 2.0.into() }, ..Default::default() }));
                }
                raster = raster.push(z);
            }
            container(raster).center(Length::Fill).into()
        }).into();
        let flaeche_el: Element<Msg> = container(flaeche_el).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        let c = d.rgb(d.sel); let farbe = Color::from_rgb8(c[0], c[1], c[2]);
        let regler = |name: &str, k: usize, v: u8| -> Element<'static, Msg> { let r2 = r.clone(); let r3 = r.clone(); iced::widget::row![text(name.to_string()).size(12).color(pal.mute).width(14), slider(0..=255u8, v, move |x| Msg::PaletteAkt(r2.clone(), PAkt::Kanal(k, x))).on_release(Msg::PaletteAkt(r3, PAkt::Fertig)).width(Length::Fill), text(format!("{v}")).size(12).font(vc_gui_ui::fonts::MONO).color(pal.text).width(30).align_x(iced::alignment::Horizontal::Right)].spacing(8).align_y(Alignment::Center).into() };
        // Karte Dokument: der gewählte Eintrag (Farbfeld, Regler, Hexwert); wird mit der Datei gespeichert
        let eintrag = iced::widget::column![
            container(space::vertical()).width(Length::Fill).height(60).style(move |_| container::Style { background: Some(farbe.into()), border: Border { color: pal.line, width: 1.0, radius: 8.0.into() }, ..Default::default() }),
            regler("R", 0, c[0]), regler("G", 1, c[1]), regler("B", 2, c[2]),
            iced::widget::row![text(vc_i18n::t!("palettestudio-hex")).size(12).color(pal.mute), text_input("#rrggbb", &self.pal_hex).on_input({ let r = r.clone(); move |h| Msg::PaletteAkt(r.clone(), PAkt::Hex(h)) }).on_submit(Msg::PaletteAkt(r.clone(), PAkt::HexOk)).size(12).padding(4).width(110).font(vc_gui_ui::fonts::MONO)].spacing(8).align_y(Alignment::Center),
            text(t::palette_hinweis()).size(11).color(pal.mute).width(Length::Fill)].spacing(muster::abstand::GRUPPE);
        let karten: Vec<Element<Msg>> = vec![muster::karte(pal, &vc_i18n::t!("palettestudio-eintrag-p1-p2", p1 = &d.sel, p2 = format!("{:02X}", d.sel)), "▤", &vc_i18n::t!("palettestudio-farbe-dieses-eintrags-wird-gespeichert"), None, eintrag.into())];
        let seite = muster::seitenspalte(klasse, karten);
        let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        let kopf_z = muster::kopfzeile(pal, klasse, "PAL", &name, d.dirty(), &vc_i18n::t!("palettestudio-n-farben", n = &n), vec![knopf_p(&vc_i18n::t!("palettestudio-exportieren"), &vc_i18n::t!("palettestudio-export-dialog-cmd-umschalt-e"), true, PAkt::Exportieren), knopf_p(&vc_i18n::t!("palettestudio-speichern"), &vc_i18n::t!("palettestudio-speichern-cmd-s"), d.dirty(), PAkt::Speichern)]);
        let fuss = muster::fusszeile(pal, &vc_i18n::t!("palettestudio-eintrag-p1-p2-p3", p1 = &d.sel, p2 = format!("{:02X}", d.sel), p3 = d.eintrag(d.sel)), text("").into());
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: muster::LISTE_BREITE, kopf: kopf_z, leiste, liste: None, flaeche: { let r2 = r.clone(); crate::kontext::rechtsklick(flaeche_el, move |pt| Msg::KontextAuf(crate::kontext::Art::FlaecheStudio, pt, Some(r2.clone()))) }, seite, fuss })
    }
}
