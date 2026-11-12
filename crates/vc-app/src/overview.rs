// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt-Übersicht (Tab „Übersicht“): Kennzahlen, Quellen, Studios, Ziel und Verbindung, Ereignisse.
//! Zeigt nur echte Werte aus Projektdatei, Bauergebnis, Dateiliste und Einstellungen; gibt es noch keinen Build,
//! steht dort „noch nicht gebaut“. Die Datenaufbereitung (oberer Teil) kennt keine Oberfläche und ist getestet.
use crate::project::{Entry, Group, Project};
use crate::ui::tokens::t;
use crate::ui::widgets::{card, chip, section, system_color};
use crate::shell::group_color;
use eframe::egui::{self, Color32, RichText};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use vc_ide::platform::{self, Presence};
use vc_ide::projfile::ProjectFile;
use vc_ide::targets::{kind_label, EmulatorCfg};

// ---------------------------------------------------------------------------------------------
// Datenaufbereitung (ohne Oberfläche)
// ---------------------------------------------------------------------------------------------

/// Deutsche Zahl mit einer Nachkommastelle: `14,8`
fn dec1(v: f64) -> String { format!("{v:.1}").replace('.', ",") }

/// Größe in Bytes als „812 B“ oder „14,8 KB“
pub fn fmt_size(bytes: usize) -> String {
    if bytes < 1024 { format!("{bytes} B") } else { format!("{} KB", dec1(bytes as f64 / 1024.0)) }
}

/// „1 Sprite“ / „2 Sprites“
pub fn count_label(n: usize, one: &str, many: &str) -> String { format!("{n} {}", if n == 1 { one } else { many }) }

/// „gerade eben“, „vor 42 s“, „vor 3 Min.“, „vor 2 Std.“
pub fn ago(secs: u64) -> String {
    match secs { 0..=1 => "gerade eben".into(), 2..=59 => format!("vor {secs} s"), 60..=3599 => format!("vor {} Min.", secs / 60), _ => format!("vor {} Std.", secs / 3600) }
}

/// Belegter Speicher aus den Segmenten der letzten Übersetzung (Start, Länge)
#[derive(Debug, PartialEq)]
pub struct MemFacts { pub used: usize, pub first: u32, pub last: u32, pub segments: usize }

pub fn memory_facts(segments: &[(u32, usize)]) -> Option<MemFacts> {
    let seg: Vec<&(u32, usize)> = segments.iter().filter(|s| s.1 > 0).collect();
    if seg.is_empty() { return None; }
    Some(MemFacts { used: seg.iter().map(|s| s.1).sum(), first: seg.iter().map(|s| s.0).min()?, last: seg.iter().map(|s| s.0 + s.1 as u32 - 1).max()?, segments: seg.len() })
}

/// Anzahl der Grafik- und Klangdateien je Art
#[derive(Debug, Default, PartialEq)]
pub struct MediaCounts { pub sprites: usize, pub charsets: usize, pub screens: usize, pub tiles: usize, pub songs: usize, pub palettes: usize }

impl MediaCounts {
    pub fn of(groups: impl Iterator<Item = Group>) -> MediaCounts {
        let mut c = MediaCounts::default();
        for g in groups { match g { Group::Sprite => c.sprites += 1, Group::Charset => c.charsets += 1, Group::Screen => c.screens += 1, Group::Tiles => c.tiles += 1, Group::Song | Group::Play => c.songs += 1, Group::Palette => c.palettes += 1, _ => {} } }
        c
    }
    pub fn total(&self) -> usize { self.sprites + self.charsets + self.screens + self.tiles + self.songs + self.palettes }
    /// Erste Zeile (Grafik) und zweite Zeile (Rest) der Kachel
    pub fn lines(&self) -> (String, String) {
        let mut a = vec![]; let mut b = vec![];
        if self.sprites > 0 { a.push(count_label(self.sprites, "Sprite", "Sprites")); }
        if self.charsets > 0 { a.push(count_label(self.charsets, "Zeichensatz", "Zeichensätze")); }
        if self.screens > 0 { b.push(count_label(self.screens, "Bildschirm", "Bildschirme")); }
        if self.tiles > 0 { b.push(count_label(self.tiles, "Kachelkarte", "Kachelkarten")); }
        if self.songs > 0 { b.push(count_label(self.songs, "Lied", "Lieder")); }
        if self.palettes > 0 { b.push(count_label(self.palettes, "Palette", "Paletten")); }
        (a.join(" · "), b.join(" · "))
    }
}

/// Zusammenfassung des letzten Baus
#[derive(Debug, PartialEq)]
pub struct BuildFacts { pub ok: bool, pub programs: usize, pub errors: usize, pub warnings: usize }

/// `log`: (Datei, gelungen) je Programm; `problems`: (Datei, ist Fehler). `None`, solange es kein Bauprotokoll gibt.
pub fn build_facts(log: &[(String, bool)], problems: &[(String, bool)]) -> Option<BuildFacts> {
    if log.is_empty() { return None; }
    let errors = problems.iter().filter(|p| p.1).count();
    Some(BuildFacts { ok: log.iter().all(|l| l.1) && errors == 0, programs: log.len(), errors, warnings: problems.len() - errors })
}

/// Zustand einer Quelldatei in der Quellenliste
#[derive(Clone, Debug, PartialEq)]
pub enum SrcStatus { Missing, NotBuilt, Built, Warnings(usize), Errors(usize) }

pub struct SrcRow { pub file: String, pub kind: String, pub dialect: String, pub converted: bool, pub target: String, pub main: bool, pub status: SrcStatus }

/// Quellen in Baureihenfolge aus der `.vcprj`. `exists`: gibt es die Datei im Projekt?
pub fn source_rows(pf: &ProjectFile, exists: impl Fn(&str) -> bool, log: &[(String, bool)], problems: &[(String, bool)]) -> Vec<SrcRow> {
    let main = pf.main_file();
    let mut rows = vec![];
    for s in &pf.sources { for f in &s.files {
        let errs = problems.iter().filter(|p| &p.0 == f && p.1).count();
        let warns = problems.iter().filter(|p| &p.0 == f && !p.1).count();
        let built = log.iter().find(|l| &l.0 == f).map(|l| l.1);
        let status = if !exists(f) { SrcStatus::Missing } else if errs > 0 || built == Some(false) { SrcStatus::Errors(errs) }
            else if warns > 0 { SrcStatus::Warnings(warns) } else if built == Some(true) { SrcStatus::Built } else { SrcStatus::NotBuilt };
        rows.push(SrcRow { file: f.clone(), kind: s.kind.clone(), dialect: s.dialect.clone(), converted: s.convert.is_some(), target: s.targets.first().cloned().unwrap_or_default(), main: main.as_deref() == Some(f.as_str()), status });
    } }
    rows
}

/// Zeile der Dateitabelle: jede am Projekt beteiligte Datei
#[derive(Debug, PartialEq)]
pub struct FileRow { pub rel: String, pub group: Group, pub tag: String, pub size: Option<u64>, pub main: bool, pub code: bool, pub status: Option<SrcStatus> }

/// Marke der Art wie in der Seitenleiste; sonstige Dateien zeigen ihre Endung
pub fn file_tag(group: Group, rel: &str) -> String {
    let t = group.tag();
    if !t.is_empty() { return t.to_string(); }
    match rel.rsplit_once('.') { Some((_, e)) if !e.is_empty() && e.len() <= 4 => e.to_uppercase(), _ => "DAT".into() }
}

/// Alle Dateien des Projekts: zuerst der Code (Startprogramm ganz oben, danach alphabetisch), dann der Rest nach Art gruppiert,
/// je Art alphabetisch. Quellen, die die `.vcprj` nennt und die fehlen, stehen mit Status „Datei fehlt“ dabei.
pub fn file_rows(root: &std::path::Path, entries: &[Entry], pf: Option<&ProjectFile>, log: &[(String, bool)], problems: &[(String, bool)]) -> Vec<FileRow> {
    let src: Vec<SrcRow> = pf.map(|pf| source_rows(pf, |f| root.join(f).is_file(), log, problems)).unwrap_or_default();
    let main = pf.and_then(|pf| pf.main_file());
    let mut rows: Vec<FileRow> = vec![];
    let mut add = |rel: &str, group: Group| {
        if rows.iter().any(|r| r.rel == rel) { return; }
        let status = src.iter().find(|s| s.file == rel).map(|s| s.status.clone());
        let in_src = status.is_some();
        rows.push(FileRow { rel: rel.to_string(), group, tag: file_tag(group, rel), size: std::fs::metadata(root.join(rel)).ok().filter(|m| m.is_file()).map(|m| m.len()), main: main.as_deref() == Some(rel), code: matches!(group, Group::Asm | Group::Basic) || in_src, status });
    };
    for e in entries { add(&e.rel, e.group); }
    for s in &src { add(&s.file, if s.kind == "basic" { Group::Basic } else { Group::Asm }); }
    rows.sort_by_key(|r| (!r.code, !r.main, if r.code { 0 } else { r.group as u8 }, r.rel.to_lowercase()));
    rows
}

/// Bezeichnung der Art: `vcasm` bzw. BASIC mit ausdrücklich gewähltem Dialekt
pub fn kind_chip(r: &SrcRow) -> String {
    match r.kind.as_str() { "asm" => "vcasm".into(), "basic" if r.dialect.is_empty() => "BASIC".into(), "basic" => format!("BASIC · {}", r.dialect), k => k.to_string() }
}

/// Zustand der Diskette des Projekts
#[derive(Debug, PartialEq)]
pub enum DiskFacts {
    /// Die Projektdatei beschreibt keine Diskette
    None,
    /// Beschrieben, aber noch nicht geschrieben
    NotBuilt { file: String, format: String },
    Image { file: String, format: String, free: String, files: usize },
    Broken { file: String, error: String },
}

pub fn disk_facts(root: &std::path::Path, pf: &ProjectFile) -> DiskFacts {
    let Some(spec) = &pf.disk else { return DiskFacts::None };
    let rel = vc_ide::projfile::disk_file(root, spec);
    let path = root.join(&rel);
    let file = rel.rsplit('/').next().unwrap_or(&rel).to_string();
    if !path.is_file() { return DiskFacts::NotBuilt { file, format: spec.format.to_uppercase() }; }
    match vc_ide::diskimage::Image::open(&path) {
        Ok(img) => DiskFacts::Image { file, format: spec.format.to_uppercase(), free: img.free, files: img.entries.len() },
        Err(error) => DiskFacts::Broken { file, error },
    }
}

/// Ampelzustand einer Verbindung
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dot { Ready, Idle, Bad }

/// Ergebnis des Verbindungstests einer Maschine
#[derive(Clone, Debug, PartialEq)]
pub enum Probe { Running, Ok(String), Err(String) }

/// Zeile des Verbindungsdiagramms
#[derive(Debug, PartialEq)]
pub struct Backend { pub name: &'static str, pub protocol: &'static str, pub dot: Dot }

/// Welches Protokoll spricht diese Emulator-Sorte? `None`: wird nur gestartet, ohne Monitor
pub fn protocol_of(kind: &str) -> Option<&'static str> {
    match kind { "x64sc" | "x64" | "x128" | "xplus4" | "xvic" => Some("vice"), "atari800" => Some("atari800"), _ => None }
}

/// Die vier Wege von vc-ide zum Ziel. `found`: Sorten der Standard-Emulatoren der Projekt-Systeme, die gefunden wurden;
/// `hw`: Maschinen der Projekt-Systeme (Sorte, Testergebnis).
pub fn backends(found: &[String], hw: &[(String, Option<Probe>)]) -> Vec<Backend> {
    let emu = |proto: &str| if found.iter().any(|k| protocol_of(k) == Some(proto)) { Dot::Ready } else { Dot::Idle };
    let machine = |kind: &str| {
        let mine: Vec<&Option<Probe>> = hw.iter().filter(|h| h.0 == kind).map(|h| &h.1).collect();
        if mine.iter().any(|p| matches!(p, Some(Probe::Ok(_)))) { Dot::Ready } else if mine.iter().any(|p| matches!(p, Some(Probe::Err(_)))) { Dot::Bad } else { Dot::Idle }
    };
    vec![
        Backend { name: "VICE", protocol: "Binärmonitor", dot: emu("vice") },
        Backend { name: "atari800", protocol: "Textmonitor", dot: emu("atari800") },
        Backend { name: "Ultimate", protocol: "REST", dot: machine("ultimate64") },
        Backend { name: "MEGA65", protocol: "UART", dot: machine("mega65") },
    ]
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind { Info, Ok, Warn, Err }

pub struct Event { pub at: Instant, pub kind: Kind, pub text: String }

// ---------------------------------------------------------------------------------------------
// Zustand und Oberfläche
// ---------------------------------------------------------------------------------------------

/// Was der Nutzer in der Übersicht angestoßen hat; die Anwendung führt es aus
pub enum Act { Build, Run, NewMedia, PickImage, OpenDoc(String), ProjectEdit, Settings }

/// Alles, was die Übersicht von der Anwendung braucht (nur Lesen)
pub struct Input<'a> {
    pub project: &'a Project,
    /// Segmente der letzten Übersetzung: (Start, Länge)
    pub segments: Option<&'a [(u32, usize)]>,
    /// Bauprotokoll: (Datei, gelungen); Probleme: (Datei, ist Fehler)
    pub log: &'a [(String, bool)],
    pub problems: &'a [(String, bool)],
    pub building: bool,
    /// Startziel (`run_on` der Anwendung): leer = Standard des Systems
    pub run_on: &'a str,
}

#[derive(Default)]
pub struct OverviewState {
    /// Projektordner, für den der Tab zuletzt angelegt wurde
    pub shown_for: Option<std::path::PathBuf>,
    pub act: Option<Act>,
    built_at: Option<Instant>,
    events: Vec<Event>,
    seen: usize,
    files: Vec<FileRow>,
    disk: Option<DiskFacts>,
    disk_at: Option<Instant>,
    found: Vec<(String, Presence)>,
    found_at: Option<Instant>,
    probes: HashMap<String, Probe>,
    chan: Option<(Sender<(String, Result<String, String>)>, Receiver<(String, Result<String, String>)>)>,
}

impl OverviewState {
    /// Ein Bau ist gerade fertig geworden
    pub fn built_now(&mut self) { self.built_at = Some(Instant::now()); self.disk_at = None; }

    /// Neue Zeilen des Ausgabeprotokolls als Ereignisse übernehmen (mit der Zeit, zu der sie auffielen)
    pub fn observe<'a>(&mut self, output: impl ExactSizeIterator<Item = (Kind, &'a str)>) {
        if output.len() < self.seen { self.seen = 0; self.events.clear(); }
        let now = Instant::now();
        for (kind, text) in output.skip(self.seen) { self.events.push(Event { at: now, kind, text: text.to_string() }); self.seen += 1; }
        if self.events.len() > 60 { self.events.drain(..self.events.len() - 60); }
    }

    /// Projekt gewechselt: alles Gemerkte verwerfen
    pub fn reset(&mut self) { self.built_at = None; self.files.clear(); self.disk = None; self.disk_at = None; self.probes.clear(); }

    fn refresh(&mut self, inp: &Input) {
        if self.disk_at.map_or(true, |t| t.elapsed() > Duration::from_secs(2)) {
            self.disk = inp.project.pf.as_ref().map(|pf| disk_facts(&inp.project.root, pf)); self.disk_at = Some(Instant::now());
            self.files = file_rows(&inp.project.root, &inp.project.entries, inp.project.pf.as_ref(), inp.log, inp.problems);
        }
        if self.found_at.map_or(true, |t| t.elapsed() > Duration::from_secs(5)) {
            let s = vc_ide::settings::effective();
            self.found = inp.project.systems.iter().map(|sys| { let e = s.emulator_for(sys); (e.kind.clone(), platform::presence(&EmulatorCfg { id: e.kind.clone(), kind: e.kind.clone(), path: e.path.unwrap_or_default(), ..Default::default() })) }).collect();
            self.found_at = Some(Instant::now());
        }
        let ch = self.chan.get_or_insert_with(channel);
        while let Ok((id, r)) = ch.1.try_recv() { self.probes.insert(id, match r { Ok(m) => Probe::Ok(m), Err(e) => Probe::Err(e) }); }
    }
}

fn dot_color(d: Dot) -> Color32 { let k = t(); match d { Dot::Ready => k.gr, Dot::Idle => k.dim, Dot::Bad => k.rd } }

/// Ab dieser Breite stehen Quellen/Studios und Ziel/Ereignisse nebeneinander, darunter untereinander
pub const TWO_COLUMNS_W: f32 = 900.0;

/// Kacheln je Zeile bei der Breite `w`: 4, 2 oder 1
pub fn tiles_per_row(w: f32) -> usize { if w >= TWO_COLUMNS_W { 4 } else if w >= 520.0 { 2 } else { 1 } }

fn status_chip(ui: &mut egui::Ui, s: &SrcStatus) {
    let k = t();
    match s {
        SrcStatus::Missing => chip(ui, "Datei fehlt", k.rd),
        SrcStatus::NotBuilt => chip(ui, "nicht gebaut", k.dim),
        SrcStatus::Built => chip(ui, "gebaut", k.gr),
        SrcStatus::Warnings(n) => chip(ui, &count_label(*n, "Warnung", "Warnungen"), k.am),
        SrcStatus::Errors(n) => chip(ui, &if *n == 0 { "Fehler".to_string() } else { count_label(*n, "Fehler", "Fehler") }, k.rd),
    }
}

/// Tabelle aller Projektdateien; bei schmalem Platz seitlich verschiebbar, bei vielen Dateien begrenzt hoch
fn files_table(ui: &mut egui::Ui, rows: &[FileRow], act: &mut Option<Act>) {
    let k = t();
    if rows.is_empty() { ui.label(RichText::new("Das Projekt enthält noch keine Dateien.").color(k.mute)); return; }
    egui::ScrollArea::both().id_salt("ov_files_scroll").max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
        egui::Grid::new("ov_files").num_columns(4).striped(true).spacing([14.0, 4.0]).min_col_width(40.0).show(ui, |ui| {
            for h in ["ART", "DATEI", "GRÖSSE", "STATUS"] { ui.label(RichText::new(h).small().color(k.dim)); }
            ui.end_row();
            for r in rows {
                chip(ui, &r.tag, group_color(r.group));
                ui.horizontal(|ui| {
                    let resp = ui.add(egui::Label::new(RichText::new(&r.rel).monospace().color(if r.status == Some(SrcStatus::Missing) { k.rd } else { k.fg })).sense(egui::Sense::click())).on_hover_text("Doppelklick öffnet die Datei");
                    if resp.double_clicked() { *act = Some(Act::OpenDoc(r.rel.clone())); }
                    if r.main { ui.label(RichText::new("★").color(k.am)).on_hover_text("Startprogramm (main)"); }
                });
                ui.label(RichText::new(r.size.map(|b| fmt_size(b as usize)).unwrap_or_else(|| "–".into())).monospace().small().color(k.mute));
                match &r.status { Some(s) => status_chip(ui, s), None => { ui.label(""); } }
                ui.end_row();
            }
        });
    });
}

pub fn show(ui: &mut egui::Ui, st: &mut OverviewState, inp: &Input) {
    st.refresh(inp);
    if st.probes.values().any(|p| matches!(p, Probe::Running)) || inp.building { ui.ctx().request_repaint_after(Duration::from_millis(250)); }
    else if st.built_at.is_some() { ui.ctx().request_repaint_after(Duration::from_secs(5)); }   // „vor 42 s“ aktuell halten
    let k = t();
    let pr = inp.project;
    let media = MediaCounts::of(pr.entries.iter().map(|e| e.group));
    let build = build_facts(inp.log, inp.problems);
    let mut act: Option<Act> = None;

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(12.0);
        // Kopf: Name, Systeme, Sprachen, Knöpfe
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&pr.name).font(egui::FontId::new(22.0, crate::fonts::ui_strong())));
            for tg in pr.pf.iter().flat_map(|pf| pf.targets.iter()) {
                let sys = if tg.system.is_empty() { tg.id.as_str() } else { tg.system.as_str() };
                chip(ui, &if tg.variant.is_empty() { crate::recent::short_name(sys).to_string() } else { format!("{} {}", crate::recent::short_name(sys), tg.variant.to_uppercase()) }, system_color(sys).gamma_multiply(1.4));
            }
            let mut kinds: Vec<String> = pr.pf.iter().flat_map(|pf| pf.sources.iter()).map(|s| match s.kind.as_str() { "asm" => "vcasm".to_string(), "basic" => "BASIC".into(), o => o.to_string() }).collect();
            kinds.dedup();
            for l in kinds { chip(ui, &l, k.pu); }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let idle = !inp.building;
                if ui.add_enabled(idle, egui::Button::new(RichText::new("▶ Starten  F5").strong().color(k.on_cy)).fill(k.cy).stroke(egui::Stroke::NONE).min_size(egui::vec2(0.0, 28.0))).clicked() { act = Some(Act::Run); }
                if ui.add_enabled(idle, egui::Button::new("Bauen  F7").min_size(egui::vec2(0.0, 28.0))).clicked() { act = Some(Act::Build); }
            });
        });
        ui.add_space(10.0);

        // Kennzahlkacheln: 4 nebeneinander, bei weniger Platz 2 mal 2, ganz schmal untereinander
        let w = ui.available_width();
        let per = tiles_per_row(w);
        let ids: Vec<usize> = (0..4).collect();
        for chunk in ids.chunks(per) {
            ui.columns(per, |cols| for (c, &i) in chunk.iter().enumerate() {
                let ui = &mut cols[c];
                match i {
                    0 => tile(ui, "SPEICHER", |ui| match inp.segments.and_then(memory_facts) {
                        Some(m) => {
                            big(ui, &format!("{} / 64 KB", fmt_size(m.used)));
                            ui.label(RichText::new(format!("${:04X}–${:04X} · {}", m.first, m.last, count_label(m.segments, "Segment", "Segmente"))).small().color(k.mute));
                            let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 5.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 2.0, k.line);
                            ui.painter().rect_filled(egui::Rect::from_min_size(r.min, egui::vec2((r.width() * m.used as f32 / 65536.0).max(2.0), r.height())), 2.0, k.cy);
                        }
                        None if inp.segments.is_some() => { big(ui, "–"); ui.label(RichText::new("Die Übersetzung meldet keine Segmente").small().color(k.mute)); }
                        None => { big(ui, "noch nicht gebaut"); ui.label(RichText::new("Bauen zeigt die Belegung").small().color(k.mute)); }
                    }),
                    1 => tile(ui, "DISKETTE", |ui| match &st.disk {
                        Some(DiskFacts::Image { file, format, free, files }) => { big(ui, file); ui.label(RichText::new(free).small().color(k.mute)); ui.label(RichText::new(format!("{} · {}", count_label(*files, "Datei", "Dateien"), format)).small().color(k.dim)); }
                        Some(DiskFacts::NotBuilt { file, format }) => { big(ui, "noch nicht gebaut"); ui.label(RichText::new(format!("{file} · {format}")).small().color(k.mute)); }
                        Some(DiskFacts::Broken { file, error }) => { big(ui, file); ui.label(RichText::new(error).small().color(k.rd)); }
                        Some(DiskFacts::None) | None => { big(ui, "keine"); if ui.add(egui::Label::new(RichText::new("Diskette in den Projekt-Einstellungen anlegen").color(k.cy).underline()).wrap().sense(egui::Sense::click())).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { act = Some(Act::ProjectEdit); } }
                    }),
                    2 => tile(ui, "GRAFIK UND KLANG", |ui| {
                        if media.total() == 0 { big(ui, "keine"); ui.label(RichText::new("Noch keine Dateien aus den Studios").small().color(k.mute)); }
                        else { let (a, b) = media.lines(); big(ui, if a.is_empty() { &b } else { &a }); if !a.is_empty() && !b.is_empty() { ui.label(RichText::new(b).small().color(k.mute)); } }
                    }),
                    _ => tile(ui, "LETZTER BUILD", |ui| match &build {
                        Some(b) => {
                            ui.horizontal(|ui| { ui.label(RichText::new(if b.ok { "Sauber" } else { "Fehlgeschlagen" }).font(egui::FontId::new(18.0, crate::fonts::mono_strong())).color(if b.ok { k.gr } else { k.rd })); });
                            if let Some(at) = st.built_at { ui.label(RichText::new(ago(at.elapsed().as_secs())).small().color(k.mute)); }
                            ui.add(egui::Label::new(RichText::new(format!("{} · {} · {}", count_label(b.programs, "Programm", "Programme"), count_label(b.errors, "Fehler", "Fehler"), count_label(b.warnings, "Warnung", "Warnungen"))).small().color(if b.errors > 0 { k.rd } else if b.warnings > 0 { k.am } else { k.dim })).wrap());
                        }
                        None => { big(ui, "noch nicht gebaut"); ui.label(RichText::new(if inp.building { "Bau läuft …" } else { "F7 baut das Projekt" }).small().color(k.mute)); }
                    }),
                }
            });
            ui.add_space(8.0);
        }
        ui.add_space(4.0);

        // Quellen und Studios links, Ziel und Ereignisse rechts; unter `TWO_COLUMNS_W` untereinander
        let left = |ui: &mut egui::Ui, act: &mut Option<Act>| {
            section(ui, "Quellen", "aus der .vcprj · Reihenfolge = Baureihenfolge", |ui| { if ui.small_button("⚙ Projekt-Einstellungen").clicked() { *act = Some(Act::ProjectEdit); } });
            card(ui, false, |ui| match &pr.pf {
                None => { ui.label(RichText::new("Keine Projektdatei (.vcprj) geladen.").color(k.mute)); }
                Some(pf) => {
                    let rows = source_rows(pf, |f| pr.root.join(f).is_file(), inp.log, inp.problems);
                    if rows.is_empty() { ui.label(RichText::new("Die Projektdatei nennt keine Quellen. Unter „Projekt-Einstellungen“ hinzufügen.").color(k.mute)); }
                    egui::ScrollArea::horizontal().id_salt("ov_sources_scroll").auto_shrink([false, true]).show(ui, |ui| {
                    egui::Grid::new("ov_sources").num_columns(4).spacing([14.0, 6.0]).show(ui, |ui| {
                        for h in ["DATEI", "ART", "TARGET", "STATUS"] { ui.label(RichText::new(h).small().color(k.dim)); }
                        ui.end_row();
                        for r in &rows {
                            ui.horizontal(|ui| {
                                if ui.add(egui::Label::new(RichText::new(&r.file).monospace().color(k.fg)).sense(egui::Sense::click())).on_hover_text("Im Editor öffnen").clicked() { *act = Some(Act::OpenDoc(r.file.clone())); }
                                if r.main { ui.label(RichText::new("★").color(k.am)).on_hover_text("Startprogramm (main)"); }
                            });
                            ui.horizontal(|ui| { chip(ui, &kind_chip(r), k.pu); if r.converted { chip(ui, "→ vcasm", k.dim); } });
                            ui.label(RichText::new(&r.target).monospace().small().color(k.mute));
                            ui.horizontal(|ui| {
                                status_chip(ui, &r.status);
                                if r.main { chip(ui, "Startprogramm", k.cy); }
                            });
                            ui.end_row();
                        }
                    });
                    });
                }
            });
            let studios: Vec<(&str, &str)> = [("sprite", "Sprites", "Sprite/PMG-Editor"), ("char", "Zeichensatz", "8×8-Zeichen, ROM-Import"), ("screen", "Bildschirm", "Textbildschirme mit Farbe"), ("tiles", "Kacheln", "Kachelkarten"), ("tracker", "Tracker", "SID, POKEY, PLAY"), ("palette", "Palette", "Farbpaletten")]
                .into_iter().filter(|(id, _, _)| crate::studios::enabled(id)).map(|(_, n, s)| (n, s)).collect();
            let disk = crate::studios::enabled("disk");
            if !studios.is_empty() || disk {
            ui.add_space(12.0);
            section(ui, "Studios", "legt eine neue Datei im Projekt an", |_| {});
            ui.horizontal_wrapped(|ui| {
                for (name, sub) in studios {
                    if ui.add(egui::Button::new(RichText::new(name).strong()).min_size(egui::vec2(104.0, 28.0))).on_hover_text(sub).clicked() { *act = Some(Act::NewMedia); }
                }
                if disk && ui.add(egui::Button::new(RichText::new("Disk-Image").strong()).min_size(egui::vec2(104.0, 28.0))).on_hover_text("Diskettenimage öffnen und Dateien übernehmen").clicked() { *act = Some(Act::PickImage); }
            });
            }
        };
        if w >= TWO_COLUMNS_W {
            ui.columns(2, |cols| { left(&mut cols[0], &mut act); connection_and_events(&mut cols[1], st, inp, &mut act); });
        } else {
            left(ui, &mut act);
            ui.add_space(12.0);
            connection_and_events(ui, st, inp, &mut act);
        }
        ui.add_space(12.0);

        // Alle Dateien in voller Breite
        section(ui, "Dateien", &format!("alles, was zum Projekt gehört · {} · Doppelklick öffnet", count_label(st.files.len(), "Datei", "Dateien")), |_| {});
        card(ui, false, |ui| files_table(ui, &st.files, &mut act));
        ui.add_space(12.0);
    });
    st.act = act.or(st.act.take());
}

/// Rechte Spalte: Ziel und Verbindung (mit Verbindungsdiagramm) und die Ereignisliste
fn connection_and_events(ui: &mut egui::Ui, st: &mut OverviewState, inp: &Input, act: &mut Option<Act>) {
    let k = t();
    let settings = vc_ide::settings::effective();
    let systems = &inp.project.systems;
    section(ui, "Ziel und Verbindung", "womit dieses Projekt läuft und worüber", |ui| { if ui.small_button("Einstellungen").clicked() { *act = Some(Act::Settings); } });
    card(ui, false, |ui| {
        if systems.is_empty() { ui.label(RichText::new("Die Projektdatei nennt kein Zielsystem.").color(k.mute)); }
        // Emulator je Zielsystem des Projekts
        for (sys, (kind, p)) in systems.iter().zip(st.found.iter()) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Emulator").small().color(k.dim));
                ui.add(egui::Label::new(RichText::new(format!("{} · {}", crate::recent::short_name(sys), kind_label(kind))).monospace().small()).truncate()).on_hover_text(&p.text);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if p.found { chip(ui, "bereit", k.gr); } else { chip(ui, "nicht gefunden", k.dim); } });
            });
        }
        // echte Maschinen dieser Systeme: ungeprüft, bis der Nutzer testet
        let machines: Vec<_> = settings.machines.iter().filter(|m| systems.iter().any(|s| s == m.system())).cloned().collect();
        for m in &machines {
            let addr = if m.kind == "ultimate64" { m.host.clone() } else { m.device.clone() };
            ui.horizontal(|ui| {
                ui.label(RichText::new("Hardware").small().color(k.dim));
                ui.add(egui::Label::new(RichText::new(format!("{} · {}", m.display_name(), if addr.is_empty() { "–".to_string() } else { addr })).monospace().small()).truncate());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match st.probes.get(&m.id) {
                        Some(Probe::Ok(_)) => { chip(ui, "online", k.gr); }
                        Some(Probe::Err(_)) => { chip(ui, "nicht erreichbar", k.rd); }
                        Some(Probe::Running) => { chip(ui, "prüfe …", k.am); }
                        None => { chip(ui, "ungeprüft", k.dim); }
                    }
                    if !matches!(st.probes.get(&m.id), Some(Probe::Running)) && ui.small_button("Verbindung testen").on_hover_text("Prüft jetzt; beim Start findet kein Netzwerkzugriff statt").clicked() {
                        st.probes.insert(m.id.clone(), Probe::Running);
                        let (tx, ctx, mm) = (st.chan.get_or_insert_with(channel).0.clone(), ui.ctx().clone(), m.clone());
                        std::thread::spawn(move || { let r = platform::test_hardware(&mm, Duration::from_secs(12)).map(|(t, d)| format!("{t} ({} ms)", d.as_millis().max(1))); let _ = tx.send((mm.id.clone(), r)); ctx.request_repaint(); });
                    }
                });
            });
            if let Some(Probe::Err(e)) = st.probes.get(&m.id) { ui.label(RichText::new(e).small().color(k.rd)); }
            if let Some(Probe::Ok(e)) = st.probes.get(&m.id) { ui.label(RichText::new(e).small().color(k.mute)); }
        }
        if machines.is_empty() { ui.label(RichText::new("Keine Hardware für dieses Projekt eingetragen").small().weak()); }
        ui.add_space(8.0);
        // Verbindungsdiagramm
        let found: Vec<String> = st.found.iter().filter(|f| f.1.found).map(|f| f.0.clone()).collect();
        let hw: Vec<(String, Option<Probe>)> = machines.iter().map(|m| (m.kind.clone(), st.probes.get(&m.id).cloned())).collect();
        ui.label(RichText::new("vc-app → vc-ide → vc-emu").monospace().small().color(k.mute));
        let rows = backends(&found, &hw);
        for (i, b) in rows.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.label(RichText::new(if i + 1 == rows.len() { "└" } else { "├" }).monospace().small().color(k.dim));
                ui.label(RichText::new(format!("{} · {}", b.name, b.protocol)).monospace().small().color(if b.dot == Dot::Idle { k.mute } else { k.fg }));
                ui.label(RichText::new("●").small().color(dot_color(b.dot)));
            });
        }
        ui.add_space(6.0);
        let first = systems.first().map(|s| s.as_str()).unwrap_or("");
        let target = match settings.start_machine(inp.run_on, first) { Some(m) if m.kind == "mega65" => format!("{} (echte Hardware, {})", m.display_name(), m.link_for(vc_ide::mega65net::Action::Start).label()), Some(m) => format!("{} (echte Hardware)", m.display_name()), None if inp.run_on == "@emulator" => "Emulator".to_string(), None => "Emulator (Standard des Systems)".to_string() };
        ui.label(RichText::new(format!("F5 startet auf: {target}")).small().color(k.mute));
    });
    ui.add_space(12.0);
    section(ui, "Ereignisse", "aus dem Ausgabeprotokoll dieser Sitzung", |_| {});
    card(ui, false, |ui| {
        if st.events.is_empty() { ui.label(RichText::new("Noch nichts geschehen").color(k.mute)); }
        for e in st.events.iter().rev().take(8).collect::<Vec<_>>().into_iter().rev() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(ago(e.at.elapsed().as_secs())).monospace().small().color(k.dim));
                let c = match e.kind { Kind::Ok => k.gr, Kind::Warn => k.am, Kind::Err => k.rd, Kind::Info => k.mute };
                ui.add(egui::Label::new(RichText::new(&e.text).small().color(c)).truncate());
            });
        }
    });
}

fn tile(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    // `ui.columns` legt linksbündig-„justified“ aus; das streckt Texte in schmalen Karten (Blocksatz): hier normales Top-down
    card(ui, false, |ui| ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| { ui.set_min_height(74.0); ui.label(RichText::new(title).small().color(t().dim)); body(ui); }));
}

fn big(ui: &mut egui::Ui, text: &str) { ui.add(egui::Label::new(RichText::new(text).font(egui::FontId::new(18.0, crate::fonts::mono_strong())).color(t().fg)).wrap()); }

#[cfg(test)]
mod tests {
    use super::*;

    fn pf(json: &str) -> ProjectFile { serde_json::from_str(json).unwrap() }

    #[test]
    fn groessen_und_zahlwoerter() {
        assert_eq!(fmt_size(812), "812 B");
        assert_eq!(fmt_size(15155), "14,8 KB");
        assert_eq!(count_label(1, "Sprite", "Sprites"), "1 Sprite");
        assert_eq!(count_label(2, "Sprite", "Sprites"), "2 Sprites");
        assert_eq!(ago(0), "gerade eben"); assert_eq!(ago(42), "vor 42 s"); assert_eq!(ago(180), "vor 3 Min."); assert_eq!(ago(7300), "vor 2 Std.");
    }

    #[test]
    fn speicher_aus_segmenten() {
        assert_eq!(memory_facts(&[]), None);
        assert_eq!(memory_facts(&[(0x0801, 0)]), None);
        assert_eq!(memory_facts(&[(0x0801, 0x100), (0xC000, 0x20)]), Some(MemFacts { used: 0x120, first: 0x0801, last: 0xC01F, segments: 2 }));
    }

    #[test]
    fn medien_zaehlen() {
        let c = MediaCounts::of([Group::Sprite, Group::Sprite, Group::Charset, Group::Song, Group::Asm, Group::Other].into_iter());
        assert_eq!((c.sprites, c.charsets, c.songs, c.total()), (2, 1, 1, 4));
        assert_eq!(c.lines(), ("2 Sprites · 1 Zeichensatz".to_string(), "1 Lied".to_string()));
        assert_eq!(MediaCounts::default().total(), 0);
    }

    #[test]
    fn build_ohne_protokoll_ist_none() {
        assert_eq!(build_facts(&[], &[]), None);
        let b = build_facts(&[("a.asm".into(), true)], &[("a.asm".into(), false)]).unwrap();
        assert_eq!(b, BuildFacts { ok: true, programs: 1, errors: 0, warnings: 1 });
        let b = build_facts(&[("a.asm".into(), false)], &[("a.asm".into(), true)]).unwrap();
        assert!(!b.ok && b.errors == 1);
    }

    #[test]
    fn quellenliste_mit_status() {
        let p = pf(r#"{"targets":[{"id":"c64","system":"c64"}],"sources":[
            {"targets":["c64"],"kind":"basic","dialect":"turbo","files":["menu.bas"]},
            {"targets":["c64"],"kind":"asm","files":["player.asm","sfx.asm","weg.asm"]}],"main":"menu.bas"}"#);
        let log = vec![("menu.bas".to_string(), true), ("player.asm".to_string(), true), ("sfx.asm".to_string(), true)];
        let problems = vec![("sfx.asm".to_string(), false)];
        let rows = source_rows(&p, |f| f != "weg.asm", &log, &problems);
        assert_eq!(rows.iter().map(|r| &r.status).collect::<Vec<_>>(), vec![&SrcStatus::Built, &SrcStatus::Built, &SrcStatus::Warnings(1), &SrcStatus::Missing]);
        assert!(rows[0].main && !rows[1].main);
        assert_eq!(kind_chip(&rows[0]), "BASIC · turbo"); assert_eq!(kind_chip(&rows[1]), "vcasm");
        // ohne Bau: nichts als gebaut ausgeben
        assert!(source_rows(&p, |_| true, &[], &[]).iter().all(|r| r.status == SrcStatus::NotBuilt));
        // Fehler schlagen Warnungen und Erfolg
        let rows = source_rows(&p, |_| true, &log, &[("player.asm".into(), true)]);
        assert_eq!(rows[1].status, SrcStatus::Errors(1));
    }

    #[test]
    fn startprogramm_ohne_main_ist_die_erste_quelle() {
        let p = pf(r#"{"targets":[{"id":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm","b.asm"]}]}"#);
        let rows = source_rows(&p, |_| true, &[], &[]);
        assert!(rows[0].main && !rows[1].main);
    }

    #[test]
    fn diskette_nicht_beschrieben_nicht_gebaut_und_fertig() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("build")).unwrap();
        assert_eq!(disk_facts(&tmp, &pf(r#"{"targets":[{"id":"c64"}]}"#)), DiskFacts::None);
        let p = pf(r#"{"targets":[{"id":"c64"}],"disk":{"format":"d64","file":"build/x.d64"}}"#);
        assert_eq!(disk_facts(&tmp, &p), DiskFacts::NotBuilt { file: "x.d64".into(), format: "D64".into() });
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap();
        d.add_prg("A", &[0x01, 0x08, 0x60]).unwrap();
        std::fs::write(tmp.join("build/x.d64"), d.to_bytes()).unwrap();
        match disk_facts(&tmp, &p) { DiskFacts::Image { file, format, free, files } => { assert_eq!((file.as_str(), format.as_str(), files), ("x.d64", "D64", 1)); assert!(free.ends_with("Blöcke frei"), "{free}"); } o => panic!("{o:?}") }
        std::fs::write(tmp.join("build/x.d64"), b"kaputt").unwrap();
        assert!(matches!(disk_facts(&tmp, &p), DiskFacts::Broken { .. }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn verbindungen_gruen_grau_rot() {
        let b = backends(&[], &[]);
        assert!(b.iter().all(|x| x.dot == Dot::Idle));
        let b = backends(&["x64sc".into(), "altirra".into()], &[("ultimate64".into(), None), ("mega65".into(), Some(Probe::Err("x".into())))]);
        assert_eq!(b.iter().map(|x| x.dot).collect::<Vec<_>>(), vec![Dot::Ready, Dot::Idle, Dot::Idle, Dot::Bad]);
        let b = backends(&["atari800".into()], &[("ultimate64".into(), Some(Probe::Ok("ok".into())))]);
        assert_eq!(b.iter().map(|x| x.dot).collect::<Vec<_>>(), vec![Dot::Idle, Dot::Ready, Dot::Ready, Dot::Idle]);
        assert_eq!(protocol_of("xemu"), None);
    }

    #[test]
    fn ereignisse_nur_neue_zeilen() {
        let mut s = OverviewState::default();
        let v = vec![(Kind::Ok, "a"), (Kind::Err, "b")];
        s.observe(v.iter().copied());
        s.observe(v.iter().copied());
        assert_eq!(s.events.len(), 2);
        let v2 = vec![(Kind::Ok, "a"), (Kind::Err, "b"), (Kind::Info, "c")];
        s.observe(v2.iter().copied());
        assert_eq!(s.events.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        s.observe(std::iter::empty());   // Protokoll geleert
        assert!(s.events.is_empty());
    }

    #[test]
    fn dateitabelle_code_zuerst_main_oben_dann_nach_art() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_files_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["b.asm", "a.asm", "main.asm", "z.chr.json", "a.spr.json", "song.trk.json", "x.d64"] { std::fs::write(tmp.join(f), "12345").unwrap(); }
        let e = |rel: &str, group| Entry { rel: rel.into(), group };
        let entries = vec![e("a.spr.json", Group::Sprite), e("b.asm", Group::Asm), e("a.asm", Group::Asm), e("main.asm", Group::Asm), e("song.trk.json", Group::Song), e("x.d64", Group::Other), e("z.chr.json", Group::Charset)];
        let p = pf(r#"{"targets":[{"id":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm","b.asm","main.asm","fehlt.asm"]}],"main":"main.asm"}"#);
        let rows = file_rows(&tmp, &entries, Some(&p), &[], &[]);
        assert_eq!(rows.iter().map(|r| r.rel.as_str()).collect::<Vec<_>>(), vec!["main.asm", "a.asm", "b.asm", "fehlt.asm", "song.trk.json", "a.spr.json", "z.chr.json", "x.d64"]);
        assert_eq!(rows.iter().map(|r| r.tag.as_str()).collect::<Vec<_>>(), vec!["ASM", "ASM", "ASM", "ASM", "TRK", "SPR", "CHR", "D64"]);
        assert!(rows[0].main && rows[0].code && rows[0].size == Some(5));
        assert_eq!(rows[3].status, Some(SrcStatus::Missing)); assert_eq!(rows[3].size, None);
        assert!(rows[5].status.is_none() && !rows[5].code);
        // keine Projektdatei: trotzdem alle Dateien, ohne Startprogramm
        assert!(file_rows(&tmp, &entries, None, &[], &[]).iter().all(|r| !r.main));
        assert_eq!(file_tag(Group::Other, "notes.txt"), "TXT"); assert_eq!(file_tag(Group::Other, "LIESMICH"), "DAT");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn kacheln_stapeln_sich_nach_breite() {
        assert_eq!((tiles_per_row(1200.0), tiles_per_row(900.0), tiles_per_row(800.0), tiles_per_row(520.0), tiles_per_row(400.0)), (4, 4, 2, 2, 1));
    }

    #[test]
    fn oberflaeche_bricht_bei_jeder_breite_um() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_w_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), "nop\n").unwrap();
        std::fs::write(tmp.join("p.vcprj"), r#"{"project":{"name":"T"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm"]}],"main":"a.asm"}"#).unwrap();
        let pr = Project::load(tmp.clone(), Some("p.vcprj".into())).unwrap();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        for w in [320.0, 600.0, 800.0, 899.0, 900.0, 1400.0] {
            let mut st = OverviewState::default();
            let inp = Input { project: &pr, segments: None, log: &[], problems: &[], building: false, run_on: "" };
            let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, 900.0))), ..Default::default() };
            for _ in 0..2 { let mut o = ctx.run_ui(raw.clone(), |ui| show(ui, &mut st, &inp)); o.textures_delta.clear(); }
            assert_eq!(st.files.len(), 2, "p.vcprj und a.asm");
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn oberflaeche_zeichnet_ohne_bau_und_mit_bau() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_ui_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.asm"), "nop\n").unwrap();
        std::fs::write(tmp.join("p.vcprj"), r#"{"project":{"name":"T"},"targets":[{"id":"c64","system":"c64","variant":"pal"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm"]}],"main":"a.asm"}"#).unwrap();
        let pr = Project::load(tmp.clone(), Some("p.vcprj".into())).unwrap();
        let mut st = OverviewState::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let log = vec![("a.asm".to_string(), true)];
        for built in [false, true] {
            let inp = Input { project: &pr, segments: if built { Some(&[(0x0801, 3)][..]) } else { None }, log: if built { &log } else { &[] }, problems: &[], building: false, run_on: "" };
            for _ in 0..2 { let mut o = ctx.run_ui(egui::RawInput::default(), |ui| show(ui, &mut st, &inp)); o.textures_delta.clear(); }
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
