// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt-Einstellungen (Stufe 4b): grafischer Editor der `.vcprj` als Reiter „Projekt“: Allgemein (Name, Beschreibung, Projektbild), Targets (Videonorm,
//! VIC-20-Speicher, Überschreibungen von Emulator und Startziel), Quellen mit Reihenfolge und Startprogramm (BASIC-Dialekt, Konvertierung)
//! (Zielordner, Regeln, Vorlagen), Diskette. Bearbeitet wird ein Entwurf; „Speichern“ schreibt die Projektdatei (Fehler stehen oben, ungültiges wird nicht gespeichert).
use crate::setui::{abschnitt, karte, klein};
use crate::{knopf, App, Msg, Pal};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Task};
use std::path::PathBuf;
use vc_ide::projfile::{ConvertSpec, DiskSpec, ProjectFile, Source, TargetDef};

#[derive(Clone, Debug)]
pub enum PeAkt {
    Name(String), Beschr(String), RestoreTabs(bool), ProjektBild(bool), BildWaehlen, BildGewaehlt(Option<PathBuf>), BildLoeschen,
    TargetPlus(String), TargetWeg(usize), TargetNorm(usize, String), TargetMem(usize, String), TargetEmu(usize, String), TargetStart(usize, String), TargetIll(usize, String), TargetEmuArt(usize, String),
    GruppePlus, GruppeAuf(usize), GruppeAb(usize), GruppeWeg(usize), GruppeArt(usize, String), GruppeTarget(usize, String), GruppeDialekt(usize, String), ConvAn(usize, bool), ConvTarget(usize, String), ConvOut(usize, String), ConvFlag(usize, &'static str, bool),
    DateiAuf(usize, usize), DateiAb(usize, usize), DateiWeg(usize, usize), DateiMain(String), DateiPlus(usize, String),
    DiskAn(bool), DiskFormat(String), DiskName(String), DiskDatei(String), DiskProg(String, bool),
    Speichern(bool), Zurueck, Auf(String), Reiter(PeReiter),
}

/// Unterreiter der Projekt-Einstellungen
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum PeReiter { Allgemein, Targets, Quellen, Diskette }
pub struct PeZ { pub reiter: PeReiter, pub entwurf: ProjectFile, pub gespeichert: ProjectFile, pub note: String, pub vorlage_name: String, pub bild_note: String, pub umgeschaltet: std::collections::HashSet<String> }
impl PeZ { pub fn neu(pf: ProjectFile) -> PeZ { PeZ { reiter: PeReiter::Allgemein, gespeichert: pf.clone(), entwurf: pf, note: String::new(), vorlage_name: String::new(), bild_note: String::new(), umgeschaltet: Default::default() } } pub fn geaendert(&self) -> bool { self.entwurf != self.gespeichert } }

fn emulator_fuer(id: &str) -> &'static str { match id { "atari8" => "atari800", "mega65" => "xemu", _ => "vice" } }
fn art_passt(art: &str, f: &str) -> bool { let l = f.to_ascii_lowercase(); if art == "basic" { l.ends_with(".sbas") || l.ends_with(".bas") || l.ends_with(".lst") || l.ends_with(".tur") } else { l.ends_with(".asm") || l.ends_with(".s") } }
fn startkandidat(f: &str) -> bool { !f.starts_with("build/") && !f.to_ascii_lowercase().ends_with(".inc") && (art_passt("asm", f) || art_passt("basic", f)) }

fn p(a: PeAkt) -> Msg { Msg::ProjEinst(a) }

fn pe_wahl<'a>(pal: Pal, opt: Vec<(String, String)>, aktuell: &str, m: Box<dyn Fn(String) -> PeAkt>) -> Element<'a, Msg> {
    row(opt.into_iter().map(|(w, l)| { let on = w == aktuell; button(text(l).size(12).color(pal.text).wrapping(iced::widget::text::Wrapping::None)).padding([3, 9]).on_press(p(m(w))).style(move |_, _| button::Style { background: Some(if on { pal.sel.into() } else { pal.panel2.into() }), border: Border { color: if on { pal.accent } else { pal.line }, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(5).wrap().vertical_spacing(5).into()
}
fn pe_zeile<'a>(pal: Pal, t: &str, i: Element<'a, Msg>) -> Element<'a, Msg> { row![text(t.to_string()).size(12.5).color(pal.mute).width(170), container(i).width(Length::Fill)].spacing(10).align_y(Alignment::Center).into() }
fn pe_eingabe<'a>(h: &str, w: &str, m: Box<dyn Fn(String) -> PeAkt>) -> Element<'a, Msg> { text_input(h, w).on_input(move |s| p(m(s))).size(12.5).padding(5).width(Length::Fill).into() }


impl App {
    pub(crate) fn pe_oeffnen(&mut self) {
        let Some(pf) = self.proj.pf.clone() else { self.log.push("Projekt-Einstellungen: erst ein Projekt öffnen.".into()); return };
        if self.pe.is_none() || !self.pe.as_ref().is_some_and(|z| z.geaendert()) { self.pe = Some(PeZ::neu(pf)) }
        self.tab_oeffnen(crate::Tab::Projekt)
    }
    pub(crate) fn pe_akt(&mut self, a: PeAkt) -> Task<Msg> {
        let root = self.root.clone(); let Some(z) = self.pe.as_mut() else { return Task::none() };
        let pf = &mut z.entwurf;
        match a {
            PeAkt::Name(n) => pf.project.name = n, PeAkt::Beschr(b) => pf.project.description = b, PeAkt::RestoreTabs(b) => pf.restore_tabs = b, PeAkt::ProjektBild(b) => pf.no_preview = !b,
            PeAkt::BildWaehlen => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("Projektbild wählen").add_filter("Bild", &["png", "jpg", "jpeg", "gif", "bmp"]).pick_file().await.map(|f| f.path().to_path_buf()) }, |f| Msg::ProjEinst(PeAkt::BildGewaehlt(f))),
            PeAkt::BildGewaehlt(f) => { if let Some(f) = f { z.bild_note = match vc_ide::livebild::import_project_preview(&root, &f) { Ok(_) => "Projektbild ersetzt.".into(), Err(e) => format!("Fehlgeschlagen: {e}") } } }
            PeAkt::BildLoeschen => z.bild_note = match vc_ide::livebild::delete_project_preview(&root) { Ok(_) => "Projektbild gelöscht.".into(), Err(e) => format!("Fehlgeschlagen: {e}") },
            PeAkt::TargetPlus(id) => { let cpu = vc_target::load_target(&self.data, &id).map(|t| t.cpu.to_string()).unwrap_or_default(); pf.targets.push(TargetDef { system: id.clone(), emulator: emulator_fuer(&id).into(), cpu, id, ..Default::default() }) }
            PeAkt::TargetWeg(i) => { if i < pf.targets.len() { pf.targets.remove(i); } }
            PeAkt::TargetNorm(i, v) => { if let Some(t) = pf.targets.get_mut(i) { t.variant = v } }
            PeAkt::TargetMem(i, v) => { if let Some(t) = pf.targets.get_mut(i) { if v.is_empty() { t.extra.remove("memory"); } else { t.extra.insert("memory".into(), serde_json::Value::String(v)); } } }
            PeAkt::TargetEmu(i, v) => { if let Some(t) = pf.targets.get_mut(i) { t.default_emulator = v } }
            PeAkt::TargetStart(i, v) => { if let Some(t) = pf.targets.get_mut(i) { t.start_on = v } }
            PeAkt::TargetIll(i, v) => { if let Some(t) = pf.targets.get_mut(i) { t.illegals = v } }
            PeAkt::TargetEmuArt(i, v) => { if let Some(t) = pf.targets.get_mut(i) { t.emulator = v } }
            PeAkt::Reiter(r) => z.reiter = r,
            PeAkt::GruppePlus => pf.sources.push(Source { targets: pf.targets.first().map(|t| vec![t.id.clone()]).unwrap_or_default(), kind: "asm".into(), files: vec![], ..Default::default() }),
            PeAkt::GruppeAuf(g) => { if g > 0 && g < pf.sources.len() { pf.sources.swap(g, g - 1) } }
            PeAkt::GruppeAb(g) => { if g + 1 < pf.sources.len() { pf.sources.swap(g, g + 1) } }
            PeAkt::GruppeWeg(g) => { if g < pf.sources.len() { for n in pf.sources[g].files.clone() { if pf.main.as_deref() == Some(n.as_str()) { pf.main = None } } pf.sources.remove(g); } }
            PeAkt::GruppeArt(g, v) => { if let Some(s) = pf.sources.get_mut(g) { s.kind = v } }
            PeAkt::GruppeTarget(g, v) => { if let Some(s) = pf.sources.get_mut(g) { s.targets = if v.is_empty() { vec![] } else { vec![v] } } }
            PeAkt::GruppeDialekt(g, v) => { if let Some(s) = pf.sources.get_mut(g) { s.dialect = v } }
            PeAkt::ConvAn(g, an) => { if let Some(s) = pf.sources.get_mut(g) { s.convert = if an { Some(ConvertSpec { track_line: true, ..Default::default() }) } else { None } } }
            PeAkt::ConvTarget(g, v) => { if let Some(c) = pf.sources.get_mut(g).and_then(|s| s.convert.as_mut()) { c.target = v } }
            PeAkt::ConvOut(g, v) => { if let Some(c) = pf.sources.get_mut(g).and_then(|s| s.convert.as_mut()) { c.out = v } }
            PeAkt::ConvFlag(g, w, b) => { if let Some(c) = pf.sources.get_mut(g).and_then(|s| s.convert.as_mut()) { match w { "int_div" => c.int_div = b, "float" => c.float = b, "track_line" => c.track_line = b, "no_rem" => c.no_rem = b, "inline" => c.inline = b, "embed" => c.embed = b, _ => c.turbo = b } } }
            PeAkt::DateiAuf(g, f) => { if let Some(s) = pf.sources.get_mut(g) { if f > 0 && f < s.files.len() { s.files.swap(f, f - 1) } } }
            PeAkt::DateiAb(g, f) => { if let Some(s) = pf.sources.get_mut(g) { if f + 1 < s.files.len() { s.files.swap(f, f + 1) } } }
            PeAkt::DateiWeg(g, f) => { if let Some(s) = pf.sources.get_mut(g) { if f < s.files.len() { let n = s.files.remove(f); if pf.main.as_deref() == Some(n.as_str()) { pf.main = None } if let Some(d) = &mut pf.disk { d.files.retain(|x| *x != n) } } } }
            PeAkt::DateiMain(n) => pf.main = Some(n),
            PeAkt::DateiPlus(g, n) => { if let Some(s) = pf.sources.get_mut(g) { s.files.push(n) } }
            PeAkt::DiskAn(an) => { pf.disk = if an { Some(DiskSpec { format: match pf.targets.first().map(|t| t.id.as_str()) { Some("atari8") => "atr", Some("mega65") => "d81", _ => "d64" }.into(), ..Default::default() }) } else { None } }
            PeAkt::DiskFormat(v) => { if let Some(d) = &mut pf.disk { d.format = v } } PeAkt::DiskName(v) => { if let Some(d) = &mut pf.disk { d.name = v } } PeAkt::DiskDatei(v) => { if let Some(d) = &mut pf.disk { d.file = v } }
            PeAkt::DiskProg(n, an) => { let srcs: Vec<String> = pf.source_files().into_iter().map(|s| s.0).collect(); if let Some(d) = &mut pf.disk { if an { if !d.files.contains(&n) { d.files.push(n) } } else { d.files.retain(|x| *x != n) } d.files.sort_by_key(|f| srcs.iter().position(|s| s == f).unwrap_or(usize::MAX)) } }
            PeAkt::Auf(k) => { if !z.umgeschaltet.remove(&k) { z.umgeschaltet.insert(k); } }
            PeAkt::Zurueck => { z.entwurf = z.gespeichert.clone(); z.note.clear() }
            PeAkt::Speichern(bauen) => {
                let probleme = z.entwurf.validate_getrennt(&root).0; if !probleme.is_empty() { z.note = format!("Nicht gespeichert: {}", probleme.join("; ")); return Task::none() }
                let Some(datei) = self.proj.file.as_ref().map(|f| root.join(f)) else { return Task::none() };
                match z.entwurf.save(&datei) { Ok(()) => { z.gespeichert = z.entwurf.clone(); z.note = "Gespeichert.".into(); self.log.push("Projektdatei gespeichert".into()); self.projekt_neu_laden(); if bauen { return self.befehl(vc_gui_ui::befehle::Cmd::Bauen) } } Err(e) => z.note = format!("Speichern fehlgeschlagen: {e}") }
            }
        }
        Task::none()
    }

    pub(crate) fn pe_ansicht(&self, pal: Pal) -> Element<'_, Msg> {
        let Some(z) = self.pe.as_ref() else { return column![klein(pal, "Kein Projekt geöffnet.")].padding(20).into() };
        let pf = &z.entwurf; let (probleme, warnungen) = pf.validate_getrennt(&self.root);
        let kopf = row![text("Projekt-Einstellungen").size(20).color(pal.text), if z.geaendert() { text("● ungespeichert").size(12.5).color(pal.warn) } else { text("").size(12.5) }, iced::widget::space::horizontal(),
            knopf(pal, "Zurücksetzen", z.geaendert().then(|| p(PeAkt::Zurueck)), pal.text, false), knopf(pal, "Speichern", (z.geaendert() && probleme.is_empty()).then(|| p(PeAkt::Speichern(false))), pal.text, false), knopf(pal, "Speichern & Projekt bauen", (z.geaendert() && probleme.is_empty()).then(|| p(PeAkt::Speichern(true))), pal.text, true)].spacing(10).align_y(Alignment::Center).wrap().vertical_spacing(6);
        let mut c = column![kopf].spacing(12);
        if !probleme.is_empty() { c = c.push(karte(pal, probleme.iter().map(|x| text(format!("Achtung: {x}")).size(12.5).color(pal.err).into()).collect())) }
        if !warnungen.is_empty() { c = c.push(karte(pal, warnungen.iter().map(|x| text(format!("Hinweis: {x}")).size(12.5).color(pal.warn).into()).collect())) }
        if !z.note.is_empty() { c = c.push(text(z.note.clone()).size(12.5).color(pal.ok)) }
        let mut sek_allgemein = column![].spacing(12); let mut sek_targets = column![].spacing(12); let mut sek_quellen = column![].spacing(12); let mut sek_diskette = column![].spacing(12);
        // Allgemein
        let bild = self.root.join("projektbild.png").is_file();
        sek_allgemein = sek_allgemein.push(abschnitt(pal, "Allgemein", "")).push(karte(pal, vec![pe_zeile(pal, "Name", pe_eingabe("Projektname", &pf.project.name, Box::new(PeAkt::Name))), pe_zeile(pal, "Beschreibung", pe_eingabe("", &pf.project.description, Box::new(PeAkt::Beschr))),
            checkbox(pf.restore_tabs).label("Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen").on_toggle(|b| p(PeAkt::RestoreTabs(b))).size(16).text_size(13).into(),
            checkbox(!pf.no_preview).label("Projektbild verwenden (projektbild.png)").on_toggle(|b| p(PeAkt::ProjektBild(b))).size(16).text_size(13).into(),
            if !pf.no_preview { row![klein(pal, if bild { "projektbild.png vorhanden" } else { "noch kein Projektbild" }), knopf(pal, "Bild wählen …", Some(p(PeAkt::BildWaehlen)), pal.text, false), knopf(pal, "Löschen", bild.then(|| p(PeAkt::BildLoeschen)), pal.text, false), klein(pal, &z.bild_note)].spacing(10).align_y(Alignment::Center).wrap().into() } else { text("").into() }]));
        // Targets
        let bekannte: Vec<(String, String)> = vc_ide::targets::installed_systems(Some(&self.data)).into_iter().map(|s| (s.id.to_string(), s.name.to_string())).collect();
        let g = self.satz.gespeichert.released();
        let mut tl = vec![];
        for (i, t) in pf.targets.iter().enumerate() {
            let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
            let mut k = vec![row![text(t.id.clone()).size(14).font(vc_gui_ui::fonts::MONO).color(pal.text), klein(pal, format!("CPU {}", t.cpu)), iced::widget::space::horizontal(), knopf(pal, "Entfernen", Some(p(PeAkt::TargetWeg(i))), pal.text, false)].spacing(10).align_y(Alignment::Center).into(),
                pe_zeile(pal, "Videonorm", pe_wahl(pal, vec![(String::new(), "Standard".into()), ("pal".into(), "pal".into()), ("ntsc".into(), "ntsc".into())], &t.variant, Box::new(move |v| PeAkt::TargetNorm(i, v))))];
            if sys == "vic20" { let cur = t.extra.get("memory").and_then(|v| v.as_str()).unwrap_or("").to_string(); k.push(pe_zeile(pal, "Speicherausbau", pe_wahl(pal, vec![("".into(), "Standard".into()), ("unexpanded".into(), "5 KB".into()), ("3k".into(), "+3 KB".into()), ("8k".into(), "+8 KB".into()), ("16k".into(), "+16 KB".into()), ("24k".into(), "+24 KB".into()), ("35k".into(), "35 KB".into())], &cur, Box::new(move |v| PeAkt::TargetMem(i, v))))) }
            let mut emus = vec![(String::new(), "wie in den Einstellungen".to_string())]; emus.extend(g.target(&sys).map(|c| c.emulators.iter().map(|e| (e.id.clone(), e.display_name().to_string())).collect::<Vec<_>>()).unwrap_or_default());
            k.push(pe_zeile(pal, "Standard-Emulator", pe_wahl(pal, emus, &t.default_emulator, Box::new(move |v| PeAkt::TargetEmu(i, v)))));
            let mut st = vec![(String::new(), "wie in den Einstellungen".to_string()), ("@emulator".into(), "Emulator".into())]; st.extend(g.machines_for(&sys).map(|m| (m.id.clone(), format!("{} (Hardware)", m.display_name()))));
            k.push(pe_zeile(pal, "Startziel (F5)", pe_wahl(pal, st, &t.start_on, Box::new(move |v| PeAkt::TargetStart(i, v)))));
            k.push(pe_zeile(pal, "Undokumentierte Befehle", pe_wahl(pal, ["", "deny", "warn", "allow"].iter().map(|x| (x.to_string(), if x.is_empty() { "Standard".into() } else { x.to_string() })).collect(), &t.illegals, Box::new(move |v| PeAkt::TargetIll(i, v)))));
            k.push(pe_zeile(pal, "Emulator", pe_wahl(pal, ["", "vice", "atari800", "xemu"].iter().map(|x| (x.to_string(), if x.is_empty() { "Standard".into() } else { x.to_string() })).collect(), &t.emulator, Box::new(move |v| PeAkt::TargetEmuArt(i, v)))));
            tl.push(karte(pal, k));
        }
        let frei: Vec<&(String, String)> = bekannte.iter().filter(|(id, _)| !pf.targets.iter().any(|t| &t.id == id)).collect();
        sek_targets = sek_targets.push(abschnitt(pal, &format!("Targets ({})", pf.targets.len()), "")).push(column(tl).spacing(8)).push(row![klein(pal, "Target hinzufügen:"), row(frei.iter().map(|(id, n)| button(text(n.clone()).size(12).color(pal.text)).padding([3, 8]).on_press(p(PeAkt::TargetPlus(id.clone()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>()).spacing(5).wrap().vertical_spacing(5)].spacing(8).align_y(Alignment::Center).wrap());
        // Quellen
        let alle: Vec<String> = pf.source_files().into_iter().map(|s| s.0).collect(); let main = pf.main_file(); let dateien: Vec<String> = self.namen.clone();
        let tids: Vec<String> = pf.targets.iter().map(|t| t.id.clone()).collect(); let dialekte: Vec<String> = vec!["cbm_v2".into(), "basic7".into(), "basic35".into(), "basic65".into(), "atari_basic".into(), "turbo_basic_xl".into()];
        let mut qs = vec![];
        let ngr = pf.sources.len();
        for (gi, s) in pf.sources.iter().enumerate() {
            let cur = s.targets.first().cloned().unwrap_or_default();
            let mut k = vec![row![klein(pal, "Art"), pe_wahl(pal, vec![("asm".into(), "Assembler".into()), ("basic".into(), "BASIC".into())], &s.kind, Box::new(move |v| PeAkt::GruppeArt(gi, v))), klein(pal, "Target"), pe_wahl(pal, tids.iter().map(|t| (t.clone(), t.clone())).collect(), &cur, Box::new(move |v| PeAkt::GruppeTarget(gi, v))), iced::widget::space::horizontal(), knopf(pal, "Gruppe auf", (gi > 0).then(|| p(PeAkt::GruppeAuf(gi))), pal.text, false), knopf(pal, "Gruppe ab", (gi + 1 < ngr).then(|| p(PeAkt::GruppeAb(gi))), pal.text, false), knopf(pal, "Gruppe löschen", Some(p(PeAkt::GruppeWeg(gi))), pal.text, false)].spacing(8).align_y(Alignment::Center).wrap().into()];
            if s.kind == "basic" {
                let mut d = vec![(String::new(), "Standard des Targets".to_string())]; d.extend(dialekte.iter().map(|x| (x.clone(), vc_ide_model::recent::dialect_label(x).to_string())));
                k.push(pe_zeile(pal, "BASIC-Dialekt", pe_wahl(pal, d, &s.dialect, Box::new(move |v| PeAkt::GruppeDialekt(gi, v)))));
                k.push(checkbox(s.convert.is_some()).label("Beim Bauen nach Assembler konvertieren").on_toggle(move |b| p(PeAkt::ConvAn(gi, b))).size(16).text_size(13).into());
                if let Some(cv) = &s.convert {
                    let mut ct = vec![(String::new(), "Target der Gruppe".to_string())]; ct.extend(vc_ide::b2a::targets().iter().map(|t| (t.to_string(), t.to_uppercase())));
                    k.push(pe_zeile(pal, "Laufzeit für", pe_wahl(pal, ct, &cv.target, Box::new(move |v| PeAkt::ConvTarget(gi, v))))); k.push(pe_zeile(pal, "Ausgabe", crate::pfadfeld::pfadfeld(pal, "Standard: <name>_asm.asm", &cv.out, crate::pfadfeld::PfadArt::ProjektSpeichern(".asm"), Length::Fill, move |v| p(PeAkt::ConvOut(gi, v)))));
                    let fl = [("int_div", "Ganzzahl-Division", cv.int_div), ("float", "Fließkomma erzwingen", cv.float), ("track_line", "Zeilenverfolgung", cv.track_line), ("no_rem", "REM weglassen", cv.no_rem), ("inline", "Stil inline", cv.inline), ("embed", "Laufzeit einbetten", cv.embed), ("turbo", "Turbo BASIC XL", cv.turbo)];
                    k.push(row(fl.into_iter().map(|(w, l, b)| checkbox(b).label(l).on_toggle(move |x| p(PeAkt::ConvFlag(gi, w, x))).size(15).text_size(12.5).into()).collect::<Vec<_>>()).spacing(14).wrap().vertical_spacing(5).into());
                }
            }
            let n = s.files.len();
            for (fi, f) in s.files.iter().enumerate() {
                let ist = main.as_deref() == Some(f.as_str()); let fehlt = !self.root.join(f).is_file();
                k.push(row![knopf(pal, if ist { "★" } else { "☆" }, Some(p(PeAkt::DateiMain(f.clone()))), pal.text, false), text(f.clone()).size(12.5).color(if fehlt { pal.err } else { pal.text }), iced::widget::space::horizontal(), knopf(pal, "Auf", (fi > 0).then(|| p(PeAkt::DateiAuf(gi, fi))), pal.text, false), knopf(pal, "Ab", (fi + 1 < n).then(|| p(PeAkt::DateiAb(gi, fi))), pal.text, false), knopf(pal, "Entf.", Some(p(PeAkt::DateiWeg(gi, fi))), pal.text, false)].spacing(6).align_y(Alignment::Center).wrap().into());
            }
            if n == 0 { k.push(klein(pal, "Noch keine Dateien: unten eine hinzufügen.")) }
            let kand: Vec<&String> = dateien.iter().filter(|f| startkandidat(f) && art_passt(&s.kind, f) && !alle.contains(*f)).collect();
            if !kand.is_empty() { k.push(row![klein(pal, "Datei hinzufügen:"), row(kand.into_iter().map(|f| button(text(f.clone()).size(12).color(pal.text)).padding([3, 8]).on_press(p(PeAkt::DateiPlus(gi, f.clone()))).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>()).spacing(5).wrap().vertical_spacing(5)].spacing(8).wrap().into()) }
            qs.push(karte(pal, k));
        }
        sek_quellen = sek_quellen.push(abschnitt(pal, &format!("Quellen und Startprogramm ({} Programme)", alle.len()), "„Projekt bauen“ baut die Gruppen von oben nach unten, innerhalb einer Gruppe die Dateien von oben nach unten; in dieser Reihenfolge erscheinen auch die Ergebnisse. Ist kein ★ gesetzt, gilt das erste Programm als Startprogramm. ★ = Startprogramm (F5 / F6).")).push(column(qs).spacing(8)).push(knopf(pal, "Quellgruppe hinzufügen", Some(p(PeAkt::GruppePlus)), pal.text, false));
        // Diskette
        let mut dk = vec![checkbox(pf.disk.is_some()).label("Beim Projektbau ein Diskettenabbild erzeugen").on_toggle(|b| p(PeAkt::DiskAn(b))).size(16).text_size(13).into()];
        if let Some(d) = &pf.disk {
            dk.push(pe_zeile(pal, "Format", pe_wahl(pal, vec![("d64".into(), "D64 (C64/1541)".into()), ("d71".into(), "D71 (1571)".into()), ("d81".into(), "D81 (1581, MEGA65)".into()), ("atr".into(), "ATR (Atari, DOS 2)".into())], &d.format, Box::new(PeAkt::DiskFormat))));
            dk.push(pe_zeile(pal, "Diskettenname", pe_eingabe("Projektname", &d.name, Box::new(PeAkt::DiskName)))); dk.push(pe_zeile(pal, "Ausgabedatei", crate::pfadfeld::pfadfeld(pal, &format!("build/<Ordnername>.{}", d.format), &d.file, crate::pfadfeld::PfadArt::ProjektSpeichern(match d.format.as_str() { "d64" => "d64", "d71" => "d71", "d81" => "d81", "atr" => "atr", _ => "" }), Length::Fill, |v| p(PeAkt::DiskDatei(v)))));
            dk.push(klein(pal, "Programme auf der Diskette (nichts angehakt = alle):"));
            for sfile in &alle { let n = sfile.clone(); dk.push(checkbox(d.files.contains(sfile)).label(sfile.clone()).on_toggle(move |b| p(PeAkt::DiskProg(n.clone(), b))).size(15).text_size(12.5).into()) }
        }
        sek_diskette = sek_diskette.push(abschnitt(pal, "Diskette / Datenträger", "")).push(karte(pal, dk));
        // Unterreiter mit Zählern und Zustand; rechts die Übersicht mit „Ändern ›“ (so wird auch die Diskette nicht übersehen)
        let reiter_knopf = |r: PeReiter, titel: String| -> Element<'_, Msg> { let an = z.reiter == r; button(text(titel).size(13).color(if an { pal.accent_text() } else { pal.text })).padding([5, 12]).on_press(p(PeAkt::Reiter(r))).style(move |_, _| button::Style { background: Some(if an { pal.accent.into() } else { pal.panel2.into() }), border: Border { color: if an { pal.accent } else { pal.line }, width: if pal.hoch { 2.0 } else { 1.0 }, radius: 6.0.into() }, text_color: pal.text, ..Default::default() }).into() };
        let reiter = row![reiter_knopf(PeReiter::Allgemein, "Allgemein".into()), reiter_knopf(PeReiter::Targets, format!("Targets {}", pf.targets.len())), reiter_knopf(PeReiter::Quellen, format!("Quellen {}", alle.len())), reiter_knopf(PeReiter::Diskette, format!("Diskette {}", if pf.disk.is_some() { "an" } else { "aus" }))].spacing(6).wrap().vertical_spacing(6);
        let zeile_u = |r: PeReiter, punkt: iced::Color, titel: &str, inhalt: String| -> Element<'_, Msg> { row![text("●").size(11).color(punkt), column![text(titel.to_string()).size(12.5).color(pal.text), text(inhalt).size(11.5).color(pal.mute)].spacing(2).width(Length::Fill), button(text("Ändern ›").size(11.5).color(pal.accent)).padding([2, 6]).on_press(p(PeAkt::Reiter(r))).style(|_, _| button::Style::default())].spacing(8).align_y(Alignment::Center).into() };
        let uebersicht = vc_gui_ui::muster::karte(pal, "Übersicht", "▤", "Zustand des Projekts", None, column![
            zeile_u(PeReiter::Allgemein, if pf.project.name.trim().is_empty() { pal.err } else { pal.ok }, "Allgemein", if pf.project.name.trim().is_empty() { "Name fehlt".into() } else { pf.project.name.clone() }),
            zeile_u(PeReiter::Targets, if pf.targets.is_empty() { pal.err } else { pal.ok }, "Targets", pf.targets.iter().map(|t| t.id.clone()).collect::<Vec<_>>().join(", ")),
            zeile_u(PeReiter::Quellen, if alle.is_empty() { pal.warn } else { pal.ok }, "Quellen", format!("{} Programm(e) in {} Gruppe(n){}", alle.len(), pf.sources.len(), main.as_ref().map_or(String::new(), |m| format!(" · Start: {m}")))),
            zeile_u(PeReiter::Diskette, if pf.disk.is_some() { pal.ok } else { pal.mute }, "Diskette", pf.disk.as_ref().map_or("aus".to_string(), |d| format!("{} · {}", d.format, if d.files.is_empty() { "alle Programme".to_string() } else { format!("{} Programm(e)", d.files.len()) })))].spacing(10).into());
        let inhalt: Element<Msg> = match z.reiter { PeReiter::Allgemein => sek_allgemein.into(), PeReiter::Targets => sek_targets.into(), PeReiter::Quellen => sek_quellen.into(), PeReiter::Diskette => sek_diskette.into() };
        let c = c.push(reiter);
        let seite = row![scrollable(container(inhalt).padding(iced::Padding { right: 12.0, ..Default::default() })).height(Length::Fill).width(Length::Fill), container(uebersicht).width(300)].spacing(16);
        container(column![container(c).padding(iced::Padding { top: 14.0, right: 20.0, bottom: 8.0, left: 20.0 }), container(seite).padding(iced::Padding { top: 0.0, right: 20.0, bottom: 14.0, left: 20.0 }).height(Length::Fill)]).max_width(1100).height(Length::Fill).into()
    }
}
