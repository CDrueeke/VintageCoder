// SPDX-License-Identifier: GPL-3.0-or-later
//! Hex-Ansicht: Binärdateien im Projektbaum nur ansehen (Adresse, 16 Bytes hexadezimal, ASCII). Nur Lesen: nichts davon wird je geschrieben.
//! Die Datei wird seitenweise gezeigt (4 KiB je Seite); gelesen werden höchstens die ersten 4 MiB.
use super::*;
use iced::widget::{scrollable, text};
use vc_gui_ui::muster;

pub const SEITE: usize = 4096;
pub const MAX: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub enum HAkt { Seite(i64), Anfang, Ende, Extern }

pub struct HexDoc { pub bytes: Vec<u8>, pub gesamt: u64, pub seite: usize, pub fehler: Option<String> }
impl HexDoc {
    pub fn laden(p: &std::path::Path) -> HexDoc {
        use std::io::Read;
        match std::fs::File::open(p).and_then(|f| { let gesamt = f.metadata().map(|m| m.len()).unwrap_or(0); let mut b = Vec::new(); f.take(MAX as u64).read_to_end(&mut b).map(|_| (b, gesamt)) }) {
            Ok((bytes, gesamt)) => HexDoc { bytes, gesamt, seite: 0, fehler: None },
            Err(e) => HexDoc { bytes: vec![], gesamt: 0, seite: 0, fehler: Some(format!("{}: {e}", p.display())) },
        }
    }
    pub fn seiten(&self) -> usize { self.bytes.len().div_ceil(SEITE).max(1) }
    pub fn gehe(&mut self, a: &HAkt) { let n = self.seiten(); self.seite = match a { HAkt::Seite(d) => (self.seite as i64 + d).clamp(0, n as i64 - 1) as usize, HAkt::Anfang => 0, HAkt::Ende => n - 1, HAkt::Extern => self.seite } }
    pub fn zeilen(&self) -> Vec<String> { let von = self.seite * SEITE; let bis = (von + SEITE).min(self.bytes.len()); self.bytes[von.min(bis)..bis].chunks(16).enumerate().map(|(i, c)| zeile(von + i * 16, c)).collect() }
}

/// Eine Zeile: Adresse, 16 Bytes (nach 8 eine Lücke), ASCII (Nichtdruckbares als Punkt)
pub fn zeile(adr: usize, c: &[u8]) -> String {
    let mut s = format!("{}  ", vc_i18n::t!("hexansicht-adr", adr = format!("{:08X}", adr)));
    for i in 0..16 { match c.get(i) { Some(b) => s.push_str(&format!("{} ", vc_i18n::t!("hexansicht-b", b = format!("{:02X}", b)))), None => s.push_str("   ") } if i == 7 { s.push(' ') } }
    s.push_str(" |"); for b in c { s.push(if (0x20..0x7F).contains(b) { *b as char } else { '.' }) } s.push('|'); s
}

/// Ob eine Datei statt im Standardprogramm in der Hex-Ansicht öffnet: Rohdaten, Programme, Module, unbekannte Binärformate; Bilder, PDF, Archive und Klang bleiben beim System
pub fn hex_statt_extern(rel: &str) -> bool {
    let l = rel.to_lowercase();
    ![".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".pdf", ".zip", ".gz", ".tar", ".7z", ".wav", ".mp3", ".ogg"].iter().any(|e| l.ends_with(e))
}

impl App {
    pub(crate) fn hex_oeffnen(&mut self, rel: &str) {
        if !self.hexe.contains_key(rel) { let d = HexDoc::laden(&project::pfad(&self.root, rel)); self.hexe.insert(rel.to_string(), d); }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }
    pub(crate) fn hex_aktion(&mut self, rel: &str, a: HAkt) {
        if a == HAkt::Extern { aussen::ausfuehren(aussen::Aufruf::Standardprogramm(project::pfad(&self.root, rel))); return }
        if let Some(d) = self.hexe.get_mut(rel) { d.gehe(&a) }
    }
    pub(crate) fn hex_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.hex_ansicht_k(pal, &rel, muster::klasse(size.width))).into()
    }
    fn hex_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse) -> Element<'_, Msg> {
        let Some(d) = self.hexe.get(rel) else { return text("").into() }; let r = rel.to_string();
        if let Some(f) = &d.fehler { return container(text(f.clone()).size(13).color(pal.err)).padding(20).into() }
        let tb = |name: &str, tipp: &str, an: Option<HAkt>| -> Element<Msg> {
            let mut b = button(text(name.to_string()).size(12).color(if an.is_some() { pal.text } else { pal.mute })).padding([4, 8]).style(move |_, st| button::Style { background: Some(if matches!(st, button::Status::Hovered) { pal.panel2.into() } else { pal.panel.into() }), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() });
            if let Some(a) = an { b = b.on_press(Msg::HexAkt(r.clone(), a)) }
            iced::widget::tooltip(b, container(text(tipp.to_string()).size(11.5).color(pal.text)).padding([3, 7]).style(move |_| flaeche(pal.panel2, pal.line)), iced::widget::tooltip::Position::Bottom).gap(4).into()
        };
        let n = d.seiten();
        let kopf = muster::kopfzeile(pal, klasse, &vc_i18n::t!("hexansicht-hex"), rel.rsplit('/').next().unwrap_or(rel), false, &vc_i18n::t!("hexansicht-p1-bytes-nur-lesen", p1 = &d.gesamt), vec![tb(&vc_i18n::t!("hexansicht-im-standardprogramm-oeffnen"), &vc_i18n::t!("hexansicht-oeffnet-die-datei-mit-dem-programm"), Some(HAkt::Extern))]);
        let von = d.seite * SEITE; let bis = (von + SEITE).min(d.bytes.len());
        let leiste = muster::leiste(pal, iced::widget::row![tb("⏮", &vc_i18n::t!("hexansicht-zum-anfang"), (d.seite > 0).then_some(HAkt::Anfang)), tb("◀", &vc_i18n::t!("hexansicht-vorige-seite-4-kib"), (d.seite > 0).then_some(HAkt::Seite(-1))), text(vc_i18n::t!("hexansicht-seite-p1-von-n-von-p4", p1 = d.seite + 1, n = &n, von = format!("{:08X}", von), p4 = format!("{:08X}", bis.saturating_sub(1).max(von)))).size(12).color(pal.text), tb("▶", &vc_i18n::t!("hexansicht-naechste-seite-4-kib"), (d.seite + 1 < n).then_some(HAkt::Seite(1))), tb("⏭", &vc_i18n::t!("hexansicht-zum-ende-der-gelesenen-daten"), (d.seite + 1 < n).then_some(HAkt::Ende))].spacing(8).align_y(Alignment::Center).into());
        let mut zeilen = iced::widget::column![].spacing(0);
        for z in d.zeilen() { zeilen = zeilen.push(text(z).size(12.5).font(vc_gui_ui::fonts::MONO).color(pal.text)) }
        let flaeche_el = scrollable(container(zeilen).padding(12)).width(Length::Fill).height(Length::Fill);
        let hinweis = if d.gesamt as usize > d.bytes.len() { vc_i18n::t!("hexansicht-gezeigt-werden-die-ersten-p1-mib", p1 = MAX / 1024 / 1024, p2 = &d.gesamt) } else { vc_i18n::t!("hexansicht-die-ganze-datei-ist-geladen").to_string() };
        let seite = muster::seitenspalte(klasse, vec![muster::karte(pal, &vc_i18n::t!("hexansicht-nur-lesen"), "ℹ", &vc_i18n::t!("hexansicht-binaerdatei"), None, text(vc_i18n::t!("hexansicht-diese-ansicht-veraendert-die-datei-nie")).size(12).color(pal.mute).into()), muster::karte(pal, &vc_i18n::t!("hexansicht-umfang"), "ℹ", "", None, text(hinweis).size(12).color(pal.mute).into())]);
        let fuss = muster::fusszeile(pal, &vc_i18n::t!("hexansicht-p1-bytes", p1 = &d.gesamt), text("").into());
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: 0.0, kopf, leiste, liste: None, flaeche: container(flaeche_el).style(move |_| flaeche(pal.editor, pal.line)).width(Length::Fill).height(Length::Fill).into(), seite, fuss })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn zeile_zeigt_adresse_hex_und_ascii() {
        let z = zeile(0x10, b"AB\x00\x7fz"); assert!(z.starts_with("00000010  41 42 00 7F 7A "), "{z}"); assert!(z.ends_with("|AB..z|"), "{z}");
        assert_eq!(zeile(0, &[0u8; 16]).len(), 8 + 2 + 16 * 3 + 1 + 2 + 16 + 1);
    }
    #[test] fn seiten_und_grenzen() {
        let mut d = HexDoc { bytes: vec![1; SEITE * 2 + 5], gesamt: (SEITE * 2 + 5) as u64, seite: 0, fehler: None };
        assert_eq!(d.seiten(), 3); d.gehe(&HAkt::Seite(-1)); assert_eq!(d.seite, 0); d.gehe(&HAkt::Ende); assert_eq!((d.seite, d.zeilen().len()), (2, 1)); d.gehe(&HAkt::Seite(5)); assert_eq!(d.seite, 2);
        assert_eq!(HexDoc { bytes: vec![], gesamt: 0, seite: 0, fehler: None }.seiten(), 1);
    }
    #[test] fn bilder_bleiben_beim_system() { assert!(hex_statt_extern("a.bin") && hex_statt_extern("x.prg") && !hex_statt_extern("b.PNG") && !hex_statt_extern("c.zip")); }
}
