// SPDX-License-Identifier: GPL-3.0-or-later
//! Export-Dialog der Studios (Mockup 12): Format, Name, Adresse, Umfang, Vorschau des erzeugten Textes (Quelltext, Hex, BASIC-DATA) und die
//! Aktionen „Im Emulator testen“, „In Projekt speichern“ (nach `build/export/`) und „In aktiven Code einfügen“ (`.include` an der Cursorstelle).
//! Das Modell (Vorschau, Formate) liegt in `vc_ide_model::exportfacts`; hier steht der Dialog und das Ausführen.
use super::*;
pub use vc_ide_model::exportfacts::{default_addr, parse_addr, Preview, FORMATS};

pub struct ExportDialog { /** Kachel-Layout (nur Kachelsätze und Screen-Banken) und Speicherbedarf mit 8- und 16-Bit-Index */ pub kachel: Option<vc_media::tiles::TilesLayout>, pub speicher: Option<(vc_ide_model::exportfacts::KachelSpeicher, vc_ide_model::exportfacts::KachelSpeicher)>, pub rel: String, pub kind: String, pub format: String, pub name: String, pub adresse: String, pub tab: String, pub cache: Preview, pub data_tab: Option<Result<String, String>>, pub meldung: String }

/// Wahl im Block „Kachel-Layout“
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KachelAkt { Tiles(bool), Karte(bool), Index(bool) }

#[derive(Clone, Debug, PartialEq)]
pub enum ExportAus { Schliessen, Test, Speichern, Einfuegen }

impl ExportDialog {
    pub fn neu(rel: &str, kind: &str, name: &str) -> ExportDialog { ExportDialog { kachel: None, speicher: None, rel: rel.into(), kind: kind.into(), format: if kind == "play" { "bas" } else { "asm" }.into(), name: name.replace('-', "_"), adresse: default_addr(kind).into(), tab: "text".into(), cache: Preview::default(), data_tab: None, meldung: String::new() } }
    pub fn braucht_adresse(&self) -> bool { matches!(self.format.as_str(), "inc" | "prg") }
    pub fn adr(&self) -> Option<u16> { parse_addr(&self.adresse) }
    pub fn adresse_ok(&self) -> bool { !self.braucht_adresse() || self.adr().is_some() }
}

impl App {
    /// Stand der Mediendatei des Studios (auch ungespeichert)
    fn export_medientext(&self, rel: &str) -> Option<String> { self.studios.get(rel).map(|s| s.doc.media_text()).or_else(|| self.paletten.get(rel).map(|p| p.media_text())).or_else(|| self.screens.get(rel).map(|s| s.doc.media_text())).or_else(|| self.tilesets.get(rel).map(|s| s.doc.media_text())).or_else(|| self.banken.get(rel).map(|s| s.doc.media_text())).or_else(|| self.songs.get(rel).map(|s| s.doc.media_text())) }
    /// Dialog für das Studio der aktiven Datei öffnen
    pub(crate) fn export_oeffnen(&mut self) {
        let Some(rel) = self.name().filter(|n| self.studios.contains_key(n) || self.paletten.contains_key(n) || self.screens.contains_key(n) || self.tilesets.contains_key(n) || self.banken.contains_key(n) || self.songs.contains_key(n)) else { self.log.push(t::EXPORT_KEIN_STUDIO.into()); return };
        let (name, kind) = match self.studios.get(&rel) { Some(s) => (s.doc.media["name"].as_str().unwrap_or("export").to_string(), s.doc.kind.clone()), None => match (self.screens.get(&rel), self.tilesets.get(&rel)) { (Some(s), _) => (s.doc.media["name"].as_str().unwrap_or("export").to_string(), "screen".to_string()), (_, Some(s)) => (s.doc.media["name"].as_str().unwrap_or("export").to_string(), "tileset".to_string()), _ if self.songs.contains_key(&rel) => (self.songs[&rel].doc.media["name"].as_str().unwrap_or("export").to_string(), self.songs[&rel].doc.media["kind"].as_str().unwrap_or("song").to_string()), _ if self.banken.contains_key(&rel) => (self.banken[&rel].doc.media["name"].as_str().unwrap_or("export").to_string(), "screen".to_string()), _ => { let p = &self.paletten[&rel]; (p.media["name"].as_str().unwrap_or("export").to_string(), "palette".to_string()) } } };
        let mut dlg = ExportDialog::neu(&rel, &kind, &name);
        if let Some(v) = self.export_medientext(&rel).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) { if v["kind"] == "tileset" || vc_media::tiles::is_bank(&v) { dlg.kachel = Some(vc_media::tiles::TilesLayout::aus_medium(&v)) } }
        self.export = Some(dlg); self.export_rechnen(); self.ed_aktiv = false;
    }
    /// Vorschau neu berechnen (nach jeder Änderung von Format, Name oder Adresse)
    pub(crate) fn export_rechnen(&mut self) {
        let Some(d) = &self.export else { return }; let (rel, fmt, name, adr) = (d.rel.clone(), d.format.clone(), d.name.clone(), d.adr());
        let Some(text0) = self.export_medientext(&rel) else { return };
        let kachel = self.export.as_ref().and_then(|d| d.kachel);
        let text = match kachel { Some(l) => vc_ide_model::exportfacts::mit_layout(&text0, l), None => text0.clone() };
        let cache = if fmt == "prg" && adr.is_none() { Preview { error: Some("PRG braucht eine gültige Ladeadresse (z. B. $2000)".into()), ..Default::default() } } else { vc_ide_model::exportfacts::preview_im_projekt(Some(&self.root), &self.data, &rel, &text, &fmt, &name, if matches!(fmt.as_str(), "inc" | "prg") { adr } else { None }) };
        let speicher = kachel.and_then(|l| { let (a, b) = (vc_ide_model::exportfacts::kachel_speicher(Some(&self.root), &self.data, &rel, &text0, vc_media::tiles::TilesLayout { index16: false, ..l }).ok()?, vc_ide_model::exportfacts::kachel_speicher(Some(&self.root), &self.data, &rel, &text0, vc_media::tiles::TilesLayout { index16: true, ..l }).ok()?); Some((a, b)) });
        if let Some(d) = &mut self.export { d.cache = cache; d.data_tab = None; d.speicher = speicher; }
    }
    /// BASIC-DATA-Vorschau erst bei Bedarf rechnen
    pub(crate) fn export_data_tab(&mut self) {
        let Some(d) = &self.export else { return }; let (rel, name) = (d.rel.clone(), d.name.clone());
        let Some(text) = self.export_medientext(&rel) else { return };
        let r = vc_ide_model::exportfacts::preview_data_im_projekt(Some(&self.root), &self.data, &rel, &text, &name);
        if let Some(d) = &mut self.export { d.data_tab = Some(r) }
    }
    /// Entscheidung des Dialogs ausführen: Studio speichern, exportieren, danach testen oder einfügen
    pub(crate) fn export_ausfuehren(&mut self, aus: ExportAus) -> Task<Msg> {
        if aus == ExportAus::Schliessen { self.export = None; self.editor_fokus(); return Task::none() }
        let Some(d) = &self.export else { return Task::none() };
        if d.cache.error.is_some() || !d.adresse_ok() { return Task::none() }
        let (rel, name, adr, fmt0, kind0) = (d.rel.clone(), d.name.clone(), d.adr(), d.format.clone(), d.kind.clone());
        if let Some(s) = self.studios.get_mut(&rel) { if let Err(e) = s.doc.save() { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen(); return Task::none() } }
        if let Some(s) = self.screens.get_mut(&rel) { if let Err(e) = s.doc.save() { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen(); return Task::none() } }
        if let Some(s) = self.songs.get_mut(&rel) { if let Err(e) = s.doc.speichern() { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen(); return Task::none() } }
        if let Some(s) = self.banken.get_mut(&rel) { if let Err(e) = s.doc.speichern() { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen(); return Task::none() } }
        if let Some(s) = self.tilesets.get_mut(&rel) { if let Err(e) = s.doc.speichern() { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen(); return Task::none() } }
        if let Some(p) = self.paletten.get_mut(&rel) { if let Err(e) = p.save() { self.log.push(e); return Task::none() } }
        let fmt = if matches!(aus, ExportAus::Test | ExportAus::Einfuegen) { if kind0 == "play" { "bas".to_string() } else { "asm".to_string() } } else { fmt0 };
        let kachel = self.export.as_ref().and_then(|d| d.kachel);
        let r = vc_media::project::export_mit(&self.root, &self.data, &rel, &fmt, Some(name.as_str()).filter(|l| !l.is_empty()), if matches!(fmt.as_str(), "inc" | "prg") { adr } else { None }, 10, 10, kachel);
        match r {
            Err(e) => { self.log.push(format!("Export {rel}: {e}")); if let Some(d) = &mut self.export { d.meldung = e } Task::none() }
            Ok(v) => {
                let dateien: Vec<String> = v["files"].as_array().map(|a| a.iter().filter_map(|f| f["path"].as_str().map(String::from)).collect()).unwrap_or_default();
                self.log.push(format!("Exportiert: {}", dateien.join(", "))); self.projekt_neu_laden();
                if let Some(d) = &mut self.export { d.meldung = format!("Gespeichert im Projekt: {} – im Baum unter „{}“", dateien.join(", "), t::GRUPPE_EXPORT) }
                match aus {
                    ExportAus::Speichern | ExportAus::Schliessen => Task::none(),
                    ExportAus::Einfuegen => {
                        let erg = match dateien.first().cloned() { Some(f) => self.include_einfuegen(&f), None => Err("Der Export hat keine Datei erzeugt – nichts eingefügt".to_string()) };
                        match erg {
                            Ok(m) => { self.log.push(m); self.export = None; self.editor_fokus() }
                            // Ablehnung: Grund im Dialog und im Protokoll, der Dialog bleibt offen (Format wählen, anderen Quelltext öffnen oder abbrechen)
                            Err(e) => { self.log.push(e.clone()); if let Some(d) = &mut self.export { d.meldung = e } self.schublade_zeigen() }
                        }
                        Task::none()
                    }
                    ExportAus::Test => { self.export = None; self.bauen(Danach::Start) }
                }
            }
        }
    }
    /// `.include` der Datei `src` in den zuletzt benutzten Quelltext an der Cursorstelle einfügen; Ablehnungen melden, nichts erfinden
    pub(crate) fn include_einfuegen(&mut self, src: &str) -> Result<String, String> {
        let doc = self.name().filter(|n| self.editoren.contains_key(n) && (n.ends_with(".asm") || n.ends_with(".inc"))).or_else(|| self.letzte_asm.clone().filter(|n| self.editoren.contains_key(n)));
        let Some(doc) = doc else { return Err("Kein Quelltext zum Einfügen offen – öffne zuerst eine .asm-Datei und setze den Cursor an die Einfügestelle".into()) };
        let text = self.editoren[&doc].text();
        let plan = vc_ide_model::dropinc::plan(&doc, &text, src, Some(&self.root)).map_err(|e| format!("{doc}: {e}"))?;
        self.oeffne(&doc);
        if let Some(e) = self.editoren.get_mut(&doc) {
            let l = if e.cursor.col == 0 { e.cursor.line } else { e.cursor.line + 1 };
            if l >= e.lines.len() { let ende = Pos { line: e.lines.len() - 1, col: e.line_len(e.lines.len() - 1) }; e.set_cursor(ende, false); e.insert(&format!("\n{}", plan.line)) } else { e.set_cursor(Pos { line: l, col: 0 }, false); e.insert(&format!("{}\n", plan.line)); }
        }
        Ok(format!("In {doc} eingefügt: {}", plan.line))
    }

    /// Der Dialog als Überlagerung
    pub(crate) fn export_dialog(&self, pal: Pal) -> Option<Element<'_, Msg>> {
        let d = self.export.as_ref()?; let mono = vc_gui_ui::fonts::MONO;
        let seg = |name: &str, aktiv: bool, m: Msg| -> Element<Msg> { button(text(name.to_string()).size(12).color(pal.text)).padding([4, 10]).on_press(m).style(move |_, _| button::Style { background: Some(if aktiv { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if aktiv { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() };
        let titel = text(t::EXPORT_TITEL).size(16).color(pal.text);
        let kopf = iced::widget::row![text(d.rel.clone()).size(11.5).font(mono).color(pal.mute), chip(pal, &d.kind)].spacing(8).align_y(Alignment::Center);
        let formate_liste: &[(&str, &str)] = if d.kind == "play" { &[("bas", "BASIC-Programm (PLAY)"), ("data", "BASIC DATA")] } else { FORMATS };
        let formate = iced::widget::row(formate_liste.iter().map(|(k, n)| seg(n, d.format == *k, Msg::ExportFormat((*k).to_string()))).collect::<Vec<_>>()).spacing(4);
        let mut felder = iced::widget::row![text(t::EXPORT_NAME).size(11).color(pal.mute), text_input("name", &d.name).on_input(Msg::ExportName).size(12).padding(4).width(150).font(mono)].spacing(8).align_y(Alignment::Center);
        if d.braucht_adresse() { felder = felder.push(text(if d.format == "prg" { t::EXPORT_LADEADRESSE } else { t::EXPORT_ADRESSE }).size(11).color(pal.mute)).push(text_input("$2000", &d.adresse).on_input(Msg::ExportAdresse).size(12).padding(4).width(90).font(mono)) }
        let mut umfang = iced::widget::column![].spacing(2);
        if d.cache.files.is_empty() { umfang = umfang.push(text("–").size(12).color(pal.mute)) }
        for (n, g) in &d.cache.files { umfang = umfang.push(iced::widget::row![text(n.clone()).size(12).font(mono).color(pal.text), space::horizontal(), text(format!("{g} Byte")).size(11.5).font(mono).color(pal.mute)]) }
        let tabs = iced::widget::row![seg(t::EXPORT_TAB_TEXT, d.tab == "text", Msg::ExportTab("text".into())), seg("Hex", d.tab == "hex", Msg::ExportTab("hex".into())), seg("BASIC-DATA", d.tab == "data", Msg::ExportTab("data".into()))].spacing(4);
        let koerper: String = match d.tab.as_str() { "hex" => if d.cache.hex.is_empty() { "(keine Binärdaten)".into() } else { d.cache.hex.clone() }, "data" => match &d.data_tab { Some(Ok(s)) => s.clone(), Some(Err(e)) => e.clone(), None => "…".into() }, _ => d.cache.error.clone().unwrap_or_else(|| d.cache.text.clone()) };
        let vorschau = container(scrollable(text(koerper).size(11.5).font(mono).color(pal.text).wrapping(iced::widget::text::Wrapping::None)).direction(iced::widget::scrollable::Direction::Both { vertical: Default::default(), horizontal: Default::default() }).width(Length::Fill).height(Length::Fixed(200.0))).padding(8).width(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, ..Default::default() });
        let gesamt: usize = d.cache.files.iter().map(|f| f.1).sum();
        let (zustand, zfarbe) = match &d.cache.error { Some(e) => (format!("● {e}"), pal.err), None if !d.adresse_ok() => ("● Adresse ungültig".to_string(), pal.err), None => (format!("● bereit · {gesamt} Byte"), pal.ok) };
        let ok = d.cache.error.is_none() && d.adresse_ok();
        let akt = |name: &str, a: ExportAus, haupt: bool| knopf(pal, name, ok.then(|| Msg::ExportAus(a)), pal.text, haupt);
        let kachel_block: Element<Msg> = match d.kachel { None => space::vertical().height(0).into(), Some(l) => {
            let zeile = |titel: &str, opt: Vec<(&str, bool, Msg)>| -> Element<Msg> { iced::widget::row![text(titel.to_string()).size(11).color(pal.mute).width(120), iced::widget::row(opt.into_iter().map(|(n, a, m)| seg(n, a, m)).collect::<Vec<_>>()).spacing(4)].spacing(8).align_y(Alignment::Center).into() };
            let mut b = iced::widget::column![text("Kachel-Layout").size(10.5).color(pal.mute),
                zeile("Kachel-Tabellen", vec![("planar", !l.interleaved, Msg::ExportKachel(KachelAkt::Tiles(false))), ("interleaved", l.interleaved, Msg::ExportKachel(KachelAkt::Tiles(true)))]),
                zeile("Karte", vec![("zeilenweise", !l.map_columns, Msg::ExportKachel(KachelAkt::Karte(false))), ("spaltenweise", l.map_columns, Msg::ExportKachel(KachelAkt::Karte(true)))]),
                zeile("Kachelindex", vec![("8 Bit", !l.index16, Msg::ExportKachel(KachelAkt::Index(false))), ("16 Bit", l.index16, Msg::ExportKachel(KachelAkt::Index(true)))])].spacing(4);
            if let Some((a, z)) = &d.speicher { let jetzt = if l.index16 { z } else { a }; b = b.push(text(format!("Speicher {} Byte (Kacheln {}, Farben {}, Eigenschaften {}, Zeichensatz {}, Karten {}, Zellen {}) · mit {}-Bit-Index {} Byte", jetzt.summe(), jetzt.kacheln, jetzt.farben, jetzt.eigenschaften, jetzt.zeichensatz, jetzt.karten, jetzt.zellen, if l.index16 { 8 } else { 16 }, if l.index16 { a.summe() } else { z.summe() })).size(11).color(pal.mute)) }
            if !d.cache.tiles_use.is_empty() { b = b.push(text(d.cache.tiles_use.clone()).size(11).font(mono).color(pal.text)); if !l.ist_vorgabe() { b = b.push(text("Hinweis: die Kachel-Bibliothek (tiles_use) kennt nur planar, zeilenweise, 8 Bit; andere Layouts brauchen eigene Routinen.").size(11).color(pal.warn)) } }
            b.into() } };
        let mut c = iced::widget::column![titel, kopf, text(t::EXPORT_FORMAT).size(10.5).color(pal.mute), formate, felder, kachel_block, text(t::EXPORT_UMFANG).size(10.5).color(pal.mute), umfang, tabs, vorschau, text(zustand).size(11.5).color(zfarbe)].spacing(8);
        if !d.meldung.is_empty() { c = c.push(text(d.meldung.clone()).size(11.5).color(pal.mute)) }
        c = c.push(iced::widget::row![akt(t::EXPORT_TESTEN, ExportAus::Test, false), akt(t::EXPORT_SPEICHERN, ExportAus::Speichern, false), akt(t::EXPORT_EINFUEGEN, ExportAus::Einfuegen, true), space::horizontal(), knopf(pal, t::BAUM_ABBRUCH, Some(Msg::ExportAus(ExportAus::Schliessen)), pal.text, false)].spacing(8));
        let karte = container(c).width(680).padding(18).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 12.0.into() }, ..Default::default() });
        let hintergrund = iced::widget::mouse_area(container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Color { a: 0.45, ..Color::BLACK }.into()), ..Default::default() }));
        Some(stack![hintergrund, container(karte).center(Length::Fill)].into())
    }
}
