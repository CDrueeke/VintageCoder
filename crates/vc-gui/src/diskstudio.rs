// SPDX-License-Identifier: GPL-3.0-or-later
//! Disk-Studio: D64, D71, D81 und ATR ansehen, prüfen, Dateien ablegen und ins Projekt übernehmen (Konzept 16.1, Mockup 12).
//! Kein Zeichenwerk außer der Belegungskarte: Verzeichnis, Sektor-Inspektor und Leisten sind normale Widgets.
//! Das Modell (Öffnen, Prüfen, Ablegen, Übernehmen) ist `vc_ide_model::diskdoc::DiskDoc`.
use super::*;
use iced::widget::{canvas, checkbox, text};
use vc_disk::{Addr, Cell};
use vc_ide_model::diskdoc::{self, DiskDoc};
use vc_gui_ui::muster;

#[derive(Clone, Debug, PartialEq)]
pub enum DAkt {
    Datei(usize), Sektor(Addr), Vormerken(usize, bool), Alle, Keine, Programme, Basic, Zielordner(String), OptBasic(bool), OptRaw(bool), OptAsm(bool),
    KetteAn(bool), Folgen, Zurueck, Validieren, ReportZu, Speichern, SpeichernUnter, Hinzufuegen, Uebernehmen, Einlegen, Mega65,
}
/// Wofür der Dateidialog gewählt wurde
#[derive(Clone, Debug, PartialEq)]
pub enum DPfad { SpeichernUnter, Hinzufuegen }

/// Aufbau der Belegungskarte: Zellkante, Ränder für die Achsenbeschriftung und Zahl der Spurblöcke nebeneinander (große Disketten stehen in zwei Spalten)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KartenGeo { pub cs: f32, pub bloecke: usize, pub pro: usize, pub spalten: usize }
pub const RAND_L: f32 = 26.0; pub const RAND_O: f32 = 14.0; const LUECKE: f32 = 16.0;
/// Zellgröße aus dem verfügbaren Platz: ganzzahlig, alle Spuren sichtbar ohne Scrollen; ab 41 Spuren (D71, D81) kann die Karte in zwei Spurblöcke nebeneinander umbrechen
pub fn karten_geo(spuren: usize, spalten: usize, w: f32, h: f32) -> KartenGeo {
    let (spuren, spalten) = (spuren.max(1), spalten.max(1));
    let eins = (((w - RAND_L) / spalten as f32).min((h - RAND_O) / spuren as f32)).floor().clamp(3.0, 56.0);
    if spuren > 8 { let pro = spuren.div_ceil(2); let zwei = (((w - 2.0 * RAND_L - LUECKE) / (2 * spalten) as f32).min((h - RAND_O) / pro as f32)).floor().clamp(3.0, 56.0); if zwei > eins { return KartenGeo { cs: zwei, bloecke: 2, pro, spalten } } }
    KartenGeo { cs: eins, bloecke: 1, pro: spuren, spalten }
}
impl KartenGeo {
    /// Linke obere Ecke der Zelle (Spur `tr` ab 1, Sektor `s`)
    pub fn ecke(&self, tr: usize, s: usize) -> (f32, f32) { let b = (tr - 1) / self.pro; let z = (tr - 1) % self.pro; (RAND_L + b as f32 * (RAND_L + self.spalten as f32 * self.cs + LUECKE) + s as f32 * self.cs, RAND_O + z as f32 * self.cs) }
    /// Zelle unter einem Punkt: (Spur, Sektor)
    pub fn zelle_bei(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let y = y - RAND_O; if y < 0.0 { return None }
        let blockbreite = RAND_L + self.spalten as f32 * self.cs + LUECKE; let b = ((x) / blockbreite) as usize; if b >= self.bloecke { return None }
        let x = x - b as f32 * blockbreite - RAND_L; if x < 0.0 { return None }
        let (z, s) = ((y / self.cs) as usize, (x / self.cs) as usize); if z >= self.pro || s >= self.spalten { return None }
        Some((b * self.pro + z + 1, s))
    }
}

/// Farbe einer Zelle der Belegungskarte
fn zellfarbe(pal: Pal, c: Cell, files: &[vc_disk::FileInfo]) -> Color {
    match c {
        Cell::Free => pal.panel2, Cell::System => pal.mute, Cell::Dir => pal.warn, Cell::Orphan | Cell::Conflict => pal.err,
        Cell::File(i) => match files.get(i).map(|f| f.kind) { Some("PRG" | "XEX") => pal.accent, Some("SEQ" | "BAS") => pal.ok, _ => Color::from_rgb(0.66, 0.52, 0.92) },
    }
}

struct Karte<'a> { d: &'a DiskDoc, pal: Pal, rel: String, geo: KartenGeo }
impl<'a> canvas::Program<Msg> for Karte<'a> {
    /// Zelle unter der Maus (für „Block n von m“ in der Kette)
    type State = Option<(usize, usize)>;
    fn update(&self, st: &mut Option<(usize, usize)>, e: &iced::Event, b: iced::Rectangle, c: iced::mouse::Cursor) -> Option<canvas::Action<Msg>> {
        let Ok(disk) = &self.d.disk else { return None };
        if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = e { let neu = c.position_in(b).and_then(|p| self.geo.zelle_bei(p.x, p.y)); if neu != *st { *st = neu; return Some(canvas::Action::request_redraw()) } }
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) = e { let p = c.position()?; c.position_in(b)?; return Some(canvas::Action::publish(Msg::KontextAuf(crate::kontext::Art::FlaecheStudio, p, Some(self.rel.clone()))).and_capture()) }
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = e {
            let p = c.position_in(b)?; let (tr, s) = self.geo.zelle_bei(p.x, p.y)?;
            if tr <= disk.tracks() && s < disk.sectors_in_track(tr) { return Some(canvas::Action::publish(Msg::DiskAkt(self.rel.clone(), DAkt::Sektor(Addr { track: tr, sector: s }))).and_capture()) }
        }
        None
    }
    fn draw(&self, st: &Option<(usize, usize)>, r: &iced::Renderer, _t: &Theme, b: iced::Rectangle, _c: iced::mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = canvas::Frame::new(r, b.size()); let Ok(disk) = &self.d.disk else { return vec![f.into_geometry()] };
        let (pal, g) = (self.pal, self.geo); let cs = g.cs;
        let kette = if self.d.kette_an { self.d.kette() } else { vec![] };
        let beschr = |f: &mut canvas::Frame, t: String, x: f32, y: f32, al: iced::alignment::Horizontal| f.fill_text(canvas::Text { content: t, position: iced::Point::new(x, y), color: pal.mute, size: iced::Pixels(10.0), align_x: al.into(), ..Default::default() });
        // Achsen: Spuren links (alle 5), Sektoren oben (alle 5), je Spurblock
        for blk in 0..g.bloecke { let x0 = RAND_L + blk as f32 * (RAND_L + g.spalten as f32 * cs + LUECKE);
            for s in (0..g.spalten).filter(|s| s % 5 == 0) { beschr(&mut f, s.to_string(), x0 + s as f32 * cs + cs / 2.0, 1.0, iced::alignment::Horizontal::Center) } }
        for tr in 1..=disk.tracks() {
            let (lx, ly) = g.ecke(tr, 0);
            if tr % 5 == 0 || tr == 1 { beschr(&mut f, tr.to_string(), lx - 4.0, ly + (cs - 10.0).max(0.0) / 2.0, iced::alignment::Horizontal::Right) }
            for s in 0..disk.sectors_in_track(tr) {
                let c = self.d.an.cells.get(tr - 1).and_then(|r| r.get(s)).copied().unwrap_or(Cell::Free);
                let mut col = zellfarbe(pal, c, &self.d.files);
                if let Some(sf) = self.d.sel_file { if c != Cell::Free && !matches!(c, Cell::File(i) if i == sf) { col = Color { a: 0.35, ..col } } }
                let (x0, y0) = g.ecke(tr, s);
                f.fill_rectangle(iced::Point::new(x0 + 0.5, y0 + 0.5), iced::Size::new(cs - 1.0, cs - 1.0), col);
            }
        }
        // Kette der gewählten Datei in Lesereihenfolge: Linie mit Hof (hebt sich von Zellen und Nummern ab), Pfeilspitze je Schritt,
        // Start (Kreis, grün) und Ende (Quadrat, rot) markiert; bei großen Zellen die Nummern auf Kreisen über der Linie
        let mitte = |a: Addr| { let (x, y) = g.ecke(a.track, a.sector); iced::Point::new(x + cs / 2.0, y + cs / 2.0) };
        let (kern, hof) = (pal.text, Color { a: 0.95, ..pal.panel });
        if kette.len() > 1 {
            let (bk, bh) = ((cs * 0.22).clamp(1.5, 3.0), (cs * 0.22).clamp(1.5, 3.0) + 2.5);
            for w in kette.windows(2) { let (p, q) = (mitte(w[0]), mitte(w[1])); let l = canvas::Path::line(p, q);
                f.stroke(&l, canvas::Stroke::default().with_color(hof).with_width(bh).with_line_cap(canvas::LineCap::Round)); }
            for w in kette.windows(2) { let (p, q) = (mitte(w[0]), mitte(w[1])); let l = canvas::Path::line(p, q);
                f.stroke(&l, canvas::Stroke::default().with_color(kern).with_width(bk).with_line_cap(canvas::LineCap::Round));
                // Pfeilspitze bei 60 % der Strecke, wenn die Zellen weit genug auseinander liegen
                let (dx, dy) = (q.x - p.x, q.y - p.y); let len = (dx * dx + dy * dy).sqrt();
                if len > cs * 0.9 { let (ux, uy) = (dx / len, dy / len); let a = (cs * 0.42).clamp(3.5, 7.0); let m = iced::Point::new(p.x + dx * 0.6, p.y + dy * 0.6);
                    let spitze = canvas::Path::new(|pb| { pb.move_to(iced::Point::new(m.x + ux * a, m.y + uy * a)); pb.line_to(iced::Point::new(m.x - ux * a * 0.6 - uy * a * 0.7, m.y - uy * a * 0.6 + ux * a * 0.7)); pb.line_to(iced::Point::new(m.x - ux * a * 0.6 + uy * a * 0.7, m.y - uy * a * 0.6 - ux * a * 0.7)); pb.close() });
                    f.stroke(&spitze, canvas::Stroke::default().with_color(hof).with_width(2.0)); f.fill(&spitze, kern); } }
        }
        let n = kette.len();
        for (i, a) in kette.iter().enumerate() {
            let (x, y) = g.ecke(a.track, a.sector); let m = mitte(*a); let (start, ende) = (i == 0, i + 1 == n && n > 1);
            f.stroke_rectangle(iced::Point::new(x + 0.5, y + 0.5), iced::Size::new(cs - 1.0, cs - 1.0), canvas::Stroke::default().with_color(pal.text).with_width(1.0));
            // Start: Kreis, Ende: Quadrat – Form und Farbe (nicht nur Farbe)
            let r = (cs * 0.34).max(2.5);
            if start { let k = canvas::Path::circle(m, r); f.fill(&k, pal.ok); f.stroke(&k, canvas::Stroke::default().with_color(pal.text).with_width(1.5)); }
            else if ende { let q = canvas::Path::rectangle(iced::Point::new(m.x - r, m.y - r), iced::Size::new(2.0 * r, 2.0 * r)); f.fill(&q, pal.err); f.stroke(&q, canvas::Stroke::default().with_color(pal.text).with_width(1.5)); }
            if cs >= 14.0 {
                if !start && !ende { f.fill(&canvas::Path::circle(m, cs * 0.34), pal.accent); }
                let t = if start { "S".to_string() } else if ende { "E".to_string() } else { (i + 1).to_string() };
                f.fill_text(canvas::Text { content: t, position: m, color: if start || ende { pal.accent_text() } else { pal.accent_text() }, size: iced::Pixels((cs * 0.46).min(12.0)), align_x: iced::alignment::Horizontal::Center.into(), align_y: iced::alignment::Vertical::Center, ..Default::default() })
            }
        }
        // Hover auf einer Zelle der Kette: „Block n von m“ (und Start/Ende)
        if let Some((tr, se)) = *st { if let Some(i) = kette.iter().position(|a| a.track == tr && a.sector == se) {
            let (x, y) = g.ecke(tr, se); let m = kette.len();
            let t = vc_i18n::t!("diskstudio-block-p1-von-m-p3", p1 = i + 1, m = &m, p3 = if i == 0 { format!(" {}", vc_i18n::t!("diskstudio-start")) } else if i + 1 == m { format!(" {}", vc_i18n::t!("diskstudio-ende")) } else { String::new() }); let breite = 7.2 * t.chars().count() as f32 + 12.0;
            let (bx, by) = ((x + cs + 6.0).min(b.width - breite - 2.0).max(2.0), (y - 4.0).clamp(2.0, (b.height - 22.0).max(2.0)));
            f.fill_rectangle(iced::Point::new(bx, by), iced::Size::new(breite, 20.0), pal.panel); f.stroke_rectangle(iced::Point::new(bx, by), iced::Size::new(breite, 20.0), canvas::Stroke::default().with_color(pal.accent).with_width(1.5));
            f.fill_text(canvas::Text { content: t, position: iced::Point::new(bx + 6.0, by + 4.0), color: pal.text, size: iced::Pixels(11.5), ..Default::default() }) } }
        // gewählter Sektor: Doppelrahmen
        let cur = self.d.cur; if cur.track >= 1 && cur.track <= disk.tracks() && cur.sector < disk.sectors_in_track(cur.track) { let (x, y) = g.ecke(cur.track, cur.sector);
            f.stroke_rectangle(iced::Point::new(x - 1.0, y - 1.0), iced::Size::new(cs + 2.0, cs + 2.0), canvas::Stroke::default().with_color(pal.accent).with_width(2.0));
            f.stroke_rectangle(iced::Point::new(x - 3.0, y - 3.0), iced::Size::new(cs + 6.0, cs + 6.0), canvas::Stroke { line_dash: canvas::LineDash { segments: &[3.0, 3.0], offset: 0 }, ..canvas::Stroke::default().with_color(pal.text).with_width(1.5) }); }
        vec![f.into_geometry()]
    }
}

impl App {
    pub(crate) fn disk_oeffnen(&mut self, rel: &str) {
        if !self.disks.contains_key(rel) { let d = DiskDoc::oeffnen(&project::pfad(&self.root, rel), Some(self.data.clone())); self.disks.insert(rel.to_string(), d); }
        self.marken_sichern(); self.tab_oeffnen(Tab::Datei(rel.into())); self.ed_aktiv = false;
    }

    /// Dateien aus dem Fenster auf die Diskette des aktiven Reiters legen; `false` = der aktive Reiter ist keine Diskette
    pub(crate) fn disk_ablegen(&mut self, p: &std::path::Path) -> bool {
        let Some(n) = self.name().filter(|n| self.disks.contains_key(n)) else { return false };
        let pfade: Vec<PathBuf> = if p.is_dir() { std::fs::read_dir(p).map(|rd| rd.flatten().map(|e| e.path()).filter(|q| q.is_file()).collect()).unwrap_or_default() } else { vec![p.to_path_buf()] };
        if let Some(d) = self.disks.get_mut(&n) { d.dateien_ablegen(&pfade); if let Some((_, m)) = &d.note { self.log.push(vc_i18n::t!("diskstudio-n-m", n = &n, m = &m)) } }
        true
    }

    pub(crate) fn disk_aktion(&mut self, rel: &str, a: DAkt) -> Task<Msg> {
        let root = self.root.clone();
        let Some(d) = self.disks.get_mut(rel) else { return Task::none() };
        match a {
            DAkt::Datei(i) => d.datei_waehlen(i), DAkt::Sektor(s) => d.sektor_waehlen(s),
            DAkt::Vormerken(i, an) => d.vormerken(i, an), DAkt::Alle => d.alle_vormerken(), DAkt::Keine => d.keine_vormerken(), DAkt::Programme => d.nur_programme(), DAkt::Basic => d.nur_basic(),
            DAkt::Zielordner(z) => d.opts.dir = z, DAkt::OptBasic(b) => d.opts.basic = b, DAkt::OptRaw(b) => d.opts.raw = b, DAkt::OptAsm(b) => d.opts.asm = b,
            DAkt::KetteAn(b) => d.kette_an = b,
            DAkt::Folgen => d.folgen(), DAkt::Zurueck => d.zurueck(), DAkt::Validieren => d.validieren(), DAkt::ReportZu => d.report = None,
            DAkt::Speichern => { d.speichern(); if let Some((_, m)) = &d.note { self.log.push(m.clone()) } }
            DAkt::SpeichernUnter => {
                let f = d.format().map(|f| f.label().to_lowercase()).unwrap_or_default(); let start = d.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| format!("neu.{f}"));
                let r = rel.to_string(); return Task::perform(async move { rfd::AsyncFileDialog::new().set_title(vc_i18n::t!("diskstudio-image-speichern-unter")).set_file_name(start).add_filter("Diskettenimage", &[f.as_str()]).save_file().await.map(|x| x.path().to_path_buf()) }, move |p| Msg::DiskPfad(r.clone(), DPfad::SpeichernUnter, p))
            }
            DAkt::Hinzufuegen => { let r = rel.to_string(); return Task::perform(async { rfd::AsyncFileDialog::new().set_title(vc_i18n::t!("diskstudio-dateien-auf-die-diskette-legen")).pick_files().await.map(|v| v.into_iter().map(|x| x.path().to_path_buf()).collect::<Vec<_>>()) }, move |p| Msg::DiskDateien(r.clone(), p.unwrap_or_default())) }
            DAkt::Uebernehmen => {
                if d.zielordner_ungueltig() { return Task::none() }
                let msgs = d.uebernehmen(&root); let (ok, fehler) = (msgs.iter().filter(|m| m.0 && !m.1.starts_with("  ")).count(), msgs.iter().filter(|m| !m.0).count());
                for (_, m) in msgs { self.log.push(m) }
                self.log.push(vc_i18n::t!("diskstudio-ok-datei-en-aus-rel-uebernommen", ok = &ok, rel = &rel, p3 = if fehler > 0 { vc_i18n::t!("diskstudio-fehler-fehler", fehler = &fehler) } else { String::new() }));
                if let Some(d) = self.disks.get_mut(rel) { d.keine_vormerken() } self.projekt_neu_laden(); self.schublade_zeigen();
            }
            DAkt::Einlegen => {
                let (Some(p), Some(f)) = (d.path.clone(), d.format()) else { return Task::none() };
                if d.dirty { d.note = Some((false, vc_i18n::t!("diskstudio-erst-speichern-dann-einlegen").into())); return Task::none() }
                let variant = p.parent().map(vc_ide::launch::atari_variant).unwrap_or_else(|| "pal".into());
                let m = match aussen::disk_einlegen(&p, diskdoc::target_for(f), &variant) { Ok(c) => (true, vc_i18n::t!("diskstudio-eingelegt-c", c = &c)), Err(e) => (false, e) };
                self.log.push(m.1.clone()); d.note = Some(m)
            }
            DAkt::Mega65 => {
                let Some(p) = d.path.clone() else { return Task::none() };
                if d.dirty { d.note = Some((false, vc_i18n::t!("diskstudio-erst-speichern-dann-uebertragen").into())); return Task::none() }
                if d.note.as_ref().is_some_and(|n| n.1 == vc_i18n::t!("diskstudio-uebertraegt")) { return Task::none() }
                d.note = Some((true, vc_i18n::t!("diskstudio-uebertraegt").into())); let r = rel.to_string();
                return Task::perform(async move { tokio::task::spawn_blocking(move || aussen::mega65_kopieren(&p)).await.unwrap_or_else(|e| Err(e.to_string())) }, move |res| Msg::DiskFertig(r.clone(), res))
            }
        }
        Task::none()
    }

    pub(crate) fn disk_pfad(&mut self, rel: &str, art: DPfad, p: Option<PathBuf>) {
        let Some(p) = p else { return };
        let Some(d) = self.disks.get_mut(rel) else { return };
        // Endungs-Automatik: fehlt dem gewählten Namen jede Endung, kommt die des Formats dazu (D64, D71, D81, ATR)
        let p = if art == DPfad::SpeichernUnter { let f = d.format().map(|f| f.label().to_lowercase()).unwrap_or_default(); PathBuf::from(crate::pfadfeld::mit_endung(&p.to_string_lossy(), &f)) } else { p };
        match art { DPfad::SpeichernUnter => { d.speichern_unter(p); if let Some((_, m)) = &d.note { self.log.push(m.clone()) } if d.path.as_ref().is_some_and(|q| q.starts_with(&self.root)) { self.projekt_neu_laden() } } DPfad::Hinzufuegen => {} }
    }

    pub(crate) fn disk_dateien(&mut self, rel: &str, pfade: Vec<PathBuf>) {
        if pfade.is_empty() { return }
        if let Some(d) = self.disks.get_mut(rel) { d.dateien_ablegen(&pfade); if let Some((_, m)) = &d.note { self.log.push(vc_i18n::t!("diskstudio-rel-m", rel = &rel, m = &m)) } }
    }

    pub(crate) fn disk_fertig(&mut self, rel: &str, r: Result<String, String>) {
        let m = match r { Ok(t) => (true, t), Err(e) => (false, e) }; self.log.push(m.1.clone());
        if let Some(d) = self.disks.get_mut(rel) { d.note = Some(m) }
    }

    pub(crate) fn disk_a11y(&self) -> Option<(String, String)> {
        let n = self.name()?; let d = self.disks.get(&n)?;
        let (Some(f), Ok(disk)) = (d.format(), &d.disk) else { return Some((vc_i18n::t!("diskstudio-diskette-n", n = &n), d.disk.as_ref().err().cloned().unwrap_or_default())) };
        Some((vc_i18n::t!("diskstudio-disk-studio-n-p2", n = &n, p2 = f.label()), vc_i18n::t!("diskstudio-p1-dateien-p2-frei-p3-fehler", p1 = d.files.len(), p2 = disk.free_blocks(), p3 = d.an.problems.len(), p4 = d.sel_file.and_then(|i| d.files.get(i)).map(|x| vc_i18n::t!("diskstudio-gewaehlt-p1", p1 = diskdoc::name_text(d.atari(), &x.name))).unwrap_or_default())))
    }

    pub(crate) fn disk_ansicht(&self, pal: Pal, rel: &str) -> Element<'_, Msg> {
        // Die Breitenklasse folgt der tatsächlichen Breite des Studios (auch im geteilten Dokumentenfeld)
        let rel = rel.to_string();
        iced::widget::responsive(move |size| self.disk_ansicht_k(pal, &rel, muster::klasse(size.width))).into()
    }
    fn disk_ansicht_k(&self, pal: Pal, rel: &str, klasse: muster::Klasse) -> Element<'_, Msg> {
        let Some(d) = self.disks.get(rel) else { return text("").into() }; let r = rel.to_string(); let mono = vc_gui_ui::fonts::MONO;
        let akt = |a: DAkt| Msg::DiskAkt(r.clone(), a);
        let disk = match &d.disk {
            Ok(disk) => disk,
            Err(fehler) => return container(text(if fehler.is_empty() { vc_i18n::t!("diskstudio-keine-diskette-geladen").to_string() } else { fehler.clone() }).size(13).color(pal.err)).padding(20).into(),
        };
        let atari = d.atari(); let fmt = disk.format();
        let tb = |name: &str, tipp: &str, an: Option<DAkt>| -> Element<'static, Msg> { muster::knopf(pal, name, tipp, an.map(|a| akt(a)), false, false) };
        let gespeichert = d.path.is_some() && !d.dirty; let ueberträgt = d.note.as_ref().is_some_and(|n| n.1 == vc_i18n::t!("diskstudio-uebertraegt"));
        // Zone 1: Kopfzeile: Format-Chip, Name, ● geändert, Geometrie; Datei-Aktionen nur hier
        let kopf_z = muster::kopfzeile(pal, klasse, fmt.label(), &d.titel(), d.dirty, &diskdoc::geometry_text(disk), vec![
            tb(&vc_i18n::t!("diskstudio-datei-hinzufuegen"), &vc_i18n::t!("diskstudio-dateien-auf-die-diskette-schreiben-oder"), disk.has_filesystem().then_some(DAkt::Hinzufuegen)),
            tb(&vc_i18n::t!("diskstudio-speichern-unter"), &vc_i18n::t!("diskstudio-image-als-kopie-unter-neuem-namen"), Some(DAkt::SpeichernUnter)), tb(&vc_i18n::t!("diskstudio-speichern"), &vc_i18n::t!("diskstudio-speichern-cmd-s"), (d.dirty && d.path.is_some()).then_some(DAkt::Speichern))]);
        // Zone 2: genau eine Leiste: Prüfen · Einlegen · Kette
        let mut l = iced::widget::row![tb(&vc_i18n::t!("diskstudio-validieren"), &vc_i18n::t!("diskstudio-bam-bzw-vtoc-verzeichnis-und-dateiketten"), Some(DAkt::Validieren)), tb(&if atari { vc_i18n::t!("diskstudio-einlegen-d1") } else { vc_i18n::t!("diskstudio-einlegen-laufwerk-8") }, &if gespeichert { vc_i18n::t!("diskstudio-startet-den-standard-emulator-des-systems") } else { vc_i18n::t!("diskstudio-erst-speichern-dann-einlegen-2") }, gespeichert.then_some(DAkt::Einlegen))].spacing(4).align_y(Alignment::Center);
        if d.ist_d81() { l = l.push(tb(&if ueberträgt { vc_i18n::t!("diskstudio-uebertraegt") } else { vc_i18n::t!("diskstudio-auf-mega65-kopieren") }, &vc_i18n::t!("diskstudio-kopiert-die-diskette-per-ethernet-auf"), (gespeichert && !ueberträgt).then_some(DAkt::Mega65))) }
        let folgen = tb(&match d.folge() { Some(n) => vc_i18n::t!("diskstudio-folgen-p1", p1 = diskdoc::addr_text(fmt, n)), None => vc_i18n::t!("diskstudio-folgen-kettenende").into() }, &if atari { vc_i18n::t!("diskstudio-dem-folgesektor-in-der-kette-folgen") } else { vc_i18n::t!("diskstudio-dem-kettenzeiger-zum-naechsten-sektor-folgen") }, d.folge().map(|_| DAkt::Folgen));
        let leiste = muster::leiste(pal, iced::widget::row![l, space::horizontal(), folgen, tb(&vc_i18n::t!("diskstudio-zurueck"), &vc_i18n::t!("diskstudio-zum-zuvor-besuchten-sektor"), (!d.history.is_empty()).then_some(DAkt::Zurueck))].spacing(8).align_y(Alignment::Center).into());
        // Zone 3: Verzeichnis (ohne Querscroll)
        let verz: Element<Msg> = if !disk.has_filesystem() {
            container(iced::widget::column![chip(pal, vc_i18n::t!("diskstudio-boot-diskette-ohne-dateisystem")), text(vc_i18n::t!("diskstudio-die-diskette-enthaelt-ein-boot-programm")).size(12).color(pal.text)].spacing(8)).padding(10).width(Length::Fill).into()
        } else {
            let kompakt = klasse != muster::Klasse::Breit;
            let (kopfzeile, breiten, nsp): ([&str; 8], [f32; 8], usize) = if kompakt { (["", &vc_i18n::t!("diskstudio-blk"), &vc_i18n::t!("diskstudio-name"), &vc_i18n::t!("diskstudio-typ"), &vc_i18n::t!("diskstudio-groesse"), "", "", ""], [22.0, 26.0, 120.0, 36.0, 54.0, 0.0, 0.0, 0.0], 5) } else { (["", &vc_i18n::t!("diskstudio-blk"), &vc_i18n::t!("diskstudio-name"), &vc_i18n::t!("diskstudio-typ"), &vc_i18n::t!("diskstudio-start-2"), &vc_i18n::t!("diskstudio-groesse"), &vc_i18n::t!("diskstudio-lade"), &vc_i18n::t!("diskstudio-marken")], [22.0, 26.0, 120.0, 40.0, 60.0, 54.0, 50.0, 0.0], 8) };
            let mut z = iced::widget::row![].spacing(6); for (h, b) in kopfzeile.iter().zip(breiten).take(nsp) { z = z.push(text(h.to_string()).size(10.5).color(pal.mute).width(if b == 0.0 { Length::Shrink } else { Length::Fixed(b) })) }
            let mut liste = iced::widget::column![z].spacing(3);
            for (i, f) in d.files.iter().enumerate() {
                let e = d.img.as_ref().and_then(|im| im.entries.get(i)); let brauchbar = e.is_some_and(|e| e.data.is_ok()); let gew = d.sel_file == Some(i);
                let nm = diskdoc::name_text(atari, &f.name);
                let name = button(text(if atari { nm } else { vc_i18n::t!("diskstudio-nm", nm = &nm) }).size(12.5).font(mono).color(if gew { pal.accent } else { pal.text })).padding([1, 4]).width(120).on_press(akt(DAkt::Datei(i)))
                    .style(move |_, st| button::Style { background: (gew || matches!(st, button::Status::Hovered)).then(|| pal.sel.into()), border: Border { radius: 4.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() });
                let marken = { let mut m = iced::widget::row![].spacing(4).align_y(Alignment::Center); if f.locked { m = m.push(text("🔒").size(11)) } if f.autostart { m = m.push(chip(pal, vc_i18n::t!("diskstudio-autostart"))) } if d.basic.get(i).is_some_and(|b| b.is_some()) { m = m.push(chip(pal, "BASIC")) } if let Some(Err(msg)) = e.map(|e| &e.data) { m = m.push(text(msg.clone()).size(10.5).color(pal.err)) } m };
                let vorm = checkbox(d.checked.get(i).copied().unwrap_or(false)).on_toggle_maybe(brauchbar.then(|| { let r = r.clone(); move |b| Msg::DiskAkt(r.clone(), DAkt::Vormerken(i, b)) })).size(14);
                if kompakt { liste = liste.push(iced::widget::row![container(vorm).width(22), text(f.blocks.to_string()).size(12).font(mono).color(pal.mute).width(26), name, text(f.kind).size(11).font(mono).color(pal.accent).width(36), text(match e { Some(e) if e.data.is_ok() => diskdoc::size_text(e.size()), Some(_) => vc_i18n::t!("diskstudio-fehler").into(), None => "–".into() }).size(11.5).color(pal.mute).width(54)].spacing(6).align_y(Alignment::Center)); } else {
                liste = liste.push(iced::widget::row![container(vorm).width(22), text(f.blocks.to_string()).size(12).font(mono).color(pal.mute).width(26), name, text(f.kind).size(11).font(mono).color(pal.accent).width(40),
                    text(diskdoc::addr_text(fmt, f.start)).size(12).font(mono).color(pal.mute).width(60), text(match e { Some(e) if e.data.is_ok() => diskdoc::size_text(e.size()), Some(_) => vc_i18n::t!("diskstudio-fehler").into(), None => "–".into() }).size(11.5).color(if brauchbar { pal.mute } else { pal.err }).width(54),
                    text(e.and_then(|e| e.load_address()).map(|a| vc_i18n::t!("diskstudio-a", a = format!("{:04X}", a))).unwrap_or_default()).size(11.5).font(mono).color(pal.mute).width(50), marken.wrap()].spacing(6).align_y(Alignment::Center));
                }
            }
            if d.files.is_empty() { liste = liste.push(text(vc_i18n::t!("diskstudio-die-diskette-enthaelt-keine-dateien")).size(12).color(pal.mute)) }
            container(scrollable(liste.padding(8)).height(Length::Fill)).width(Length::Fill).height(Length::Fill).into()
        };
        let liste_el: Element<Msg> = container(iced::widget::column![text(if atari { vc_i18n::t!("diskstudio-verzeichnis-namen-in-atascii") } else { vc_i18n::t!("diskstudio-verzeichnis-namen-in-petscii") }).size(muster::schrift::KARTE).font(vc_gui_ui::fonts::UI_FETT).color(pal.mute), container(verz).height(Length::Fill)].spacing(muster::abstand::ENG)).into();
        // Zone 4: Belegungskarte (füllt den freien Platz, ganzzahlige Zellen, alle Spuren ohne Scrollen) mit Legende und Sektor-Inspektor darunter
        let spuren = disk.tracks(); let spalten = (1..=spuren).map(|t| disk.sectors_in_track(t)).max().unwrap_or(1); let rel_k = rel.to_string();
        let karte_el: Element<Msg> = iced::widget::responsive(move |size| {
            let geo = karten_geo(spuren, spalten, size.width, size.height);
            container(canvas::Canvas::new(Karte { d, pal, rel: rel_k.clone(), geo }).width(Length::Fill).height(Length::Fill)).padding(4).into()
        }).into();
        let legende = { let mut l = iced::widget::row![].spacing(6).align_y(Alignment::Center); for (c, n) in [(pal.panel2, vc_i18n::t!("diskstudio-frei")), (pal.accent, "PRG/XEX".to_string()), (pal.ok, "SEQ/BAS".to_string()), (Color::from_rgb(0.66, 0.52, 0.92), vc_i18n::t!("diskstudio-andere")), (pal.warn, vc_i18n::t!("diskstudio-verzeichnis")), (pal.mute, vc_i18n::t!("diskstudio-system")), (pal.err, vc_i18n::t!("diskstudio-fehler"))] {
            l = l.push(container(space::vertical()).width(9).height(9).style(move |_| container::Style { background: Some(c.into()), border: Border { color: pal.line, width: 0.5, radius: 2.0.into() }, ..Default::default() })).push(text(n).size(10.5).color(pal.mute)) }
            if d.kette_an && d.kette().len() > 1 { l = l.push(text(vc_i18n::t!("diskstudio-kette-start-ende-richtung")).size(10.5).color(pal.text)) } l.wrap() };
        let (cur_sp, cur_se) = (d.cur.track, d.cur.sector);
        let kette_text = match d.kettenposition() { Some((n, m)) => vc_i18n::t!("diskstudio-block-n-von-m-der-datei", n = &n, m = &m, p3 = d.folge().map(|a| format!(" {}", vc_i18n::t!("diskstudio-naechster-sektor-p1", p1 = diskdoc::addr_text(fmt, a)))).unwrap_or_else(|| format!(" {}", vc_i18n::t!("diskstudio-kettenende")).into())), None => d.sel_file.map(|_| vc_i18n::t!("diskstudio-sektor-liegt-nicht-in-der-kette").to_string()).unwrap_or_else(|| vc_i18n::t!("diskstudio-datei-im-verzeichnis-waehlen-ihre-kette").to_string()) };
        let datei_details = d.sel_file.and_then(|i| d.files.get(i).map(|f| { let e = d.img.as_ref().and_then(|im| im.entries.get(i)); vc_i18n::t!("diskstudio-p1-start-p2-p3-bloecke-p4", p1 = diskdoc::name_text(atari, &f.name), p2 = diskdoc::addr_text(fmt, f.start), p3 = &f.blocks, p4 = e.and_then(|e| e.load_address()).map(|a| format!(" {}", vc_i18n::t!("diskstudio-laedt-ab-a", a = format!("{:04X}", a)))).unwrap_or_default(), p5 = if f.autostart { format!(" {}", vc_i18n::t!("diskstudio-autostart-2")) } else { String::new() }, p6 = if f.locked { format!(" {}", vc_i18n::t!("diskstudio-schreibgeschuetzt")) } else { String::new() }) })).unwrap_or_default();
        let hover_text = text(format!("{}{} · {}", if atari { String::new() } else { format!("{} ", vc_i18n::t!("diskstudio-spur-cur-sp-sektor-cur-se", cur_sp = &cur_sp, cur_se = &cur_se)) }, d.sektor_titel(), d.besitzer().unwrap_or_default())).size(muster::schrift::HINWEIS).color(pal.mute);
        let mut hex = iced::widget::column![].spacing(1);
        match d.sektor_zeilen(6) {
            Some(zeilen) => for z in zeilen {
                let mut sp: Vec<iced::widget::text::Span<'static, (), iced::Font>> = vec![iced::widget::span(format!("{:02X}  ", z.versatz)).font(mono).size(12).color(pal.mute)];
                for (i, (b, zeiger)) in z.bytes.iter().enumerate() { sp.push(iced::widget::span(format!("{} ", vc_i18n::t!("diskstudio-b", b = format!("{:02X}", b)))).font(mono).size(12).color(if *zeiger { pal.warn } else if *b == 0 { pal.mute } else { pal.text })); if i == 3 { sp.push(iced::widget::span(" ").font(mono).size(12)) } }
                sp.push(iced::widget::span(format!(" {}", z.text)).font(mono).size(12).color(pal.mute)); hex = hex.push(iced::widget::rich_text(sp))
            },
            None => hex = hex.push(text(vc_i18n::t!("diskstudio-sektor-ausserhalb-der-diskette")).size(12).color(pal.err)),
        }
        let kette_schalter: Element<Msg> = { let rk = r.clone(); checkbox(d.kette_an).label(vc_i18n::t!("diskstudio-kette-anzeigen")).on_toggle(move |b| Msg::DiskAkt(rk.clone(), DAkt::KetteAn(b))).size(14).text_size(12).into() };
        let inspektor: Element<Msg> = iced::widget::column![text(d.sektor_titel()).size(13).font(vc_gui_ui::fonts::UI_FETT).color(pal.text), text(datei_details).size(muster::schrift::HINWEIS).color(pal.mute), text(kette_text).size(muster::schrift::TEXT).color(pal.accent),
            text(format!("{}{}", d.besitzer().map(|b| format!("{b} · ")).unwrap_or_default(), d.zeiger_hinweis().unwrap_or_default())).size(muster::schrift::HINWEIS).color(pal.mute),
            container(scrollable(hex).height(Length::Fill)).padding(6).height(Length::Fixed(150.0)).width(Length::Fill).style(move |_| flaeche(pal.editor, pal.line)), kette_schalter].spacing(4).into();
        let flaeche_el: Element<Msg> = container(iced::widget::column![container(karte_el).height(Length::Fill).width(Length::Fill), legende, hover_text].spacing(6).padding(8)).width(Length::Fill).height(Length::Fill).style(move |_| container::Style { background: Some(pal.editor.into()), ..Default::default() }).into();
        // Zone 5: Karten Zustand und „In das Projekt übernehmen“
        let fehler = d.an.problems.len();
        let mut status = iced::widget::row![chip(pal, &vc_i18n::t!("diskstudio-p1-p2-frei", p1 = disk.free_blocks(), p2 = if atari { vc_i18n::t!("diskstudio-sektoren") } else { vc_i18n::t!("diskstudio-bloecke") })), chip(pal, &vc_i18n::t!("diskstudio-p1-belegt", p1 = &d.an.used)), chip(pal, &vc_i18n::t!("diskstudio-fehler-fehler-2", fehler = &fehler)),
            chip(pal, &match d.an.interleave { Some(i) => vc_i18n::t!("diskstudio-interleave-i", i = &i), None => vc_i18n::t!("diskstudio-interleave").into() })].spacing(6).align_y(Alignment::Center);
        let titel = disk.title(); if !atari && !titel.is_empty() { status = status.push(text(format!("0 \"{}\"", diskdoc::petscii_text(&titel))).size(12).font(mono).color(pal.mute)) }
        if atari { status = status.push(text(vc_i18n::t!("diskstudio-boot-sektoren-1-3")).size(11).color(pal.mute)) }
        if let Some((ok, m)) = &d.note { status = status.push(text(m.clone()).size(11.5).color(if *ok { pal.ok } else { pal.err })) }
        let mut oben = iced::widget::column![status.wrap().vertical_spacing(4)].spacing(6);
        if let Some(rp) = &d.report {
            let kopfz = if rp.is_empty() { text(vc_i18n::t!("diskstudio-validierung-keine-fehler-bam-verzeichnis-und")).size(12).color(pal.ok) } else { text(vc_i18n::t!("diskstudio-validierung-p1-problem-e", p1 = rp.len())).size(12).font(vc_gui_ui::fonts::UI_FETT).color(pal.err) };
            let mut box_ = iced::widget::column![iced::widget::row![kopfz, space::horizontal(), button(text("✕").size(12).color(pal.mute)).padding([0, 6]).on_press(akt(DAkt::ReportZu)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() })]];
            for m in rp.iter().take(30) { box_ = box_.push(text(m.clone()).size(11).font(mono).color(pal.text)) }
            oben = oben.push(container(scrollable(box_.spacing(2)).height(Length::Shrink)).padding(8).max_height(130).width(Length::Fill).style(move |_| container::Style { background: Some(pal.panel2.into()), border: Border { color: if rp.is_empty() { pal.ok } else { pal.err }, width: 1.0, radius: 6.0.into() }, ..Default::default() }));
        }
        let mut zustand = iced::widget::column![oben].spacing(muster::abstand::GRUPPE);
        if !atari { if let Some(i) = d.an.interleave { zustand = zustand.push(text(vc_i18n::t!("diskstudio-interleave-i-sektoren-einer-datei-liegen", i = &i)).size(muster::schrift::HINWEIS).color(pal.mute)) } }
        let mut uebernehmen_inhalt: Option<Element<Msg>> = None;
        if let Some(img) = &d.img {
            let basic_da = d.basic.iter().any(|b| b.is_some()); let basic_gew = d.checked.iter().zip(&d.basic).any(|(s, b)| *s && b.is_some()); let prog_da = img.entries.iter().any(|e| diskdoc::ist_programm(img, &e.kind));
            let n = d.anzahl_vorgemerkt(); let schlecht = d.zielordner_ungueltig();
            let klein = |s: &str, a: DAkt| -> Element<'static, Msg> { button(text(s.to_string()).size(11.5).color(pal.text)).padding([2, 8]).on_press(akt(a)).style(move |_, _| button::Style { background: Some(pal.panel2.into()), border: Border { color: pal.line, width: 1.0, radius: 5.0.into() }, text_color: pal.text, ..Default::default() }).into() };
            let zo = { let r = r.clone(); crate::pfadfeld::pfadfeld(pal, "import", &d.opts.dir, crate::pfadfeld::PfadArt::ProjektOrdner, Length::Fixed(110.0), move |z| Msg::DiskAkt(r.clone(), DAkt::Zielordner(z))) };
            let (rb, rr, ra) = (r.clone(), r.clone(), r.clone());
            let zeile1 = iced::widget::row![klein(&vc_i18n::t!("diskstudio-alle"), DAkt::Alle), klein(&vc_i18n::t!("diskstudio-keine"), DAkt::Keine), klein(&vc_i18n::t!("diskstudio-nur-programme"), DAkt::Programme), klein(&vc_i18n::t!("diskstudio-nur-basic"), DAkt::Basic), text(vc_i18n::t!("diskstudio-zielordner")).size(11.5).color(pal.mute), zo].spacing(8).align_y(Alignment::Center).wrap().vertical_spacing(4);
            let zeile2 = iced::widget::row![
                checkbox(d.opts.basic).label(vc_i18n::t!("diskstudio-basic-als-quelltext")).on_toggle_maybe(basic_da.then(|| move |b| Msg::DiskAkt(rb.clone(), DAkt::OptBasic(b)))).size(14).text_size(12),
                checkbox(d.opts.raw).label(vc_i18n::t!("diskstudio-original-behalten")).on_toggle_maybe((d.opts.basic && basic_gew).then(|| move |b| Msg::DiskAkt(rr.clone(), DAkt::OptRaw(b)))).size(14).text_size(12),
                checkbox(d.opts.asm).label(vc_i18n::t!("diskstudio-auch-als-assembler")).on_toggle_maybe(prog_da.then(|| move |b| Msg::DiskAkt(ra.clone(), DAkt::OptAsm(b)))).size(14).text_size(12),
                space::horizontal().width(8), if d.dirty { text(vc_i18n::t!("diskstudio-image-ungespeichert")).size(11.5).color(pal.warn).into() } else { Element::from(space::horizontal().width(0)) },
                knopf(pal, &vc_i18n::t!("diskstudio-n-datei-en-uebernehmen", n = &n), (n > 0 && !schlecht).then(|| akt(DAkt::Uebernehmen)), pal.text, true)].spacing(12).align_y(Alignment::Center).wrap().vertical_spacing(4);
            let mut fuss = iced::widget::column![zeile1, zeile2].spacing(6);
            if schlecht { fuss = fuss.push(text(vc_i18n::t!("diskstudio-der-zielordner-muss-innerhalb-des-projekts")).size(11.5).color(pal.err)) }
            uebernehmen_inhalt = Some(fuss.into());
        }
        let mut karten: Vec<Element<Msg>> = vec![muster::karte(pal, &vc_i18n::t!("diskstudio-zustand"), "▤", &vc_i18n::t!("diskstudio-belegung-pruefung-hinweise"), None, zustand.into()), muster::karte(pal, &vc_i18n::t!("diskstudio-sektor"), "▦", &vc_i18n::t!("diskstudio-inhalt-und-kette-des-gewaehlten-sektors"), None, inspektor)];
        if let Some(inhalt) = uebernehmen_inhalt { karten.push(muster::karte(pal, &vc_i18n::t!("diskstudio-in-das-projekt-uebernehmen"), "⇥", &vc_i18n::t!("diskstudio-waehlt-dateien-im-verzeichnis-haken"), None, inhalt)) }
        let seite = muster::seitenspalte(klasse, karten);
        let fuss = muster::fusszeile(pal, &vc_i18n::t!("diskstudio-p1-dateien-p2-frei-p3-belegt", p1 = d.files.len(), p2 = disk.free_blocks(), p3 = &d.an.used), text("").into());
        muster::studio_rahmen(pal, klasse, muster::Studio { liste_breite: 300.0, kopf: kopf_z, leiste, liste: None, flaeche: iced::widget::column![container(liste_el).height(Length::FillPortion(2)).width(Length::Fill).padding([8, 8]), container(flaeche_el).height(Length::FillPortion(3)).width(Length::Fill)].into(), seite, fuss })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn belegungskarte_zellgroesse_ganzzahlig_aus_dem_platz_und_klick_trifft_die_gezeichnete_zelle() {
        // D64: 35 Spuren à höchstens 21 Sektoren: alles ohne Scrollen, quadratische ganzzahlige Zellen
        let g = karten_geo(35, 21, 400.0, 420.0); assert_eq!(g.bloecke, 1); assert_eq!(g.cs, g.cs.floor()); assert!(g.cs >= 3.0 && RAND_O + 35.0 * g.cs <= 420.0 && RAND_L + 21.0 * g.cs <= 400.0, "{g:?}");
        // D81: 80 Spuren à 40 Sektoren passen in der Höhe nur in zwei Spurblöcken nebeneinander mit lesbaren Zellen
        let h = karten_geo(80, 40, 700.0, 400.0); assert_eq!(h.bloecke, 2, "{h:?}"); assert_eq!(h.pro, 40); assert!(h.cs >= 5.0, "{h:?}");
        let schmal = karten_geo(80, 40, 300.0, 400.0); assert!(schmal.cs >= 3.0, "{schmal:?}");
        // Klick und Zeichnung benutzen dieselbe Geometrie: die Mitte jeder Zelle liefert (Spur, Sektor) zurück
        for geo in [g, h] { let spuren = if geo.bloecke == 2 { 80 } else { 35 }; for tr in [1, 2, 18, spuren] { for s in [0, geo.spalten - 1] { let (x, y) = geo.ecke(tr, s); assert_eq!(geo.zelle_bei(x + geo.cs / 2.0, y + geo.cs / 2.0), Some((tr, s)), "{tr}/{s} in {geo:?}") } } }
        assert_eq!(g.zelle_bei(2.0, 2.0), None, "Achsenrand zählt nicht als Zelle");
        // Breite Fläche (D64): Zellen wachsen mit dem Platz, zwei Spurblöcke nebeneinander, Klick trifft weiter die gezeichnete Zelle
        let b = karten_geo(35, 21, 1200.0, 700.0); assert_eq!(b.bloecke, 2, "{b:?}"); assert!(b.cs > g.cs * 2.0 && b.cs <= 56.0, "{b:?}"); assert!(2.0 * RAND_L + 2.0 * 21.0 * b.cs + LUECKE <= 1200.0 && RAND_O + b.pro as f32 * b.cs <= 700.0);
        for tr in [1, 18, 19, 35] { let (x, y) = b.ecke(tr, 20); assert_eq!(b.zelle_bei(x + b.cs / 2.0, y + b.cs / 2.0), Some((tr, 20)), "{tr} in {b:?}") }
    }
}
