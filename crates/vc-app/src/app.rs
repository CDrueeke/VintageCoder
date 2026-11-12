// SPDX-License-Identifier: GPL-3.0-or-later
//! Anwendungszustand, Dock-Layout und Werkzeugleiste.
use crate::debug::{Cmd as DbgCmd, Debugger};
use crate::ui::dialog::{self, Act, Dlg};
use crate::ui::widgets;
use crate::editor::{self, Doc};
use crate::highlight::PAL_ERR;
use crate::lsp::{self, Lsp, Reply};
use crate::project::{Group, Project};
use crate::bitmap::{Palettes, Requests};
use crate::windows::{self, MediaDoc, MediaWindow};
use eframe::egui::{self, Color32, RichText};
use egui_dock::widgets::tab_viewer::OnCloseResponse;
use crate::layout::{self, LayoutFile, Open, Workplace, ZoneTabs};
use egui_dock::{DockArea, DockState, NodePath, Style, TabViewer};

mod commandbar;
mod ctx;
mod zones;
use ctx::Ctx;
mod sidebar;
mod narrow;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum Tab { Project, Overview, Doc(String), Output, BuildLog, Problems, Tests, Welcome, Regs, Disasm, Memory, Snapshot, Breaks, MemMap, Ref, Help, Watch, ProjectEdit, Settings, Live, Emulator, Studio(u8), Outline, StudioInfo }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Code, Debug }

/// Feste Zonen, in denen Panels zu Hause sind (Seitenleiste, Inspektor, Fußpanel). Dokumente, Studios, Willkommen, Übersicht und
/// Einstellungen gehören keiner Zone, sie liegen als Reiter im Editorbereich.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Zone { Side, Inspector, Footer }

impl Tab {
    fn zone(&self) -> Option<Zone> {
        match self {
            Tab::Project => Some(Zone::Side),
            Tab::MemMap | Tab::Ref | Tab::Help | Tab::Regs | Tab::Watch | Tab::Breaks | Tab::Outline | Tab::StudioInfo => Some(Zone::Inspector),
            Tab::Output | Tab::BuildLog | Tab::Problems | Tab::Tests | Tab::Disasm | Tab::Memory | Tab::Snapshot | Tab::Live | Tab::Emulator => Some(Zone::Footer),
            Tab::Overview | Tab::Doc(_) | Tab::Welcome | Tab::ProjectEdit | Tab::Settings | Tab::Studio(_) => None,
        }
    }
}

/// Reiter der drei Zonen eines Arbeitsplatzes (nicht gespeichert: die Anwendung bestimmt, was wo steht).
/// Der Inspektor des Quelltext-Arbeitsplatzes hat zwei Reiterlisten, eine für Quelltexte und eine für Studios; `studio_on` wählt sie.
/// Jede merkt sich ihren aktiven Reiter. `view` ist der Inhalt der Seitenleiste (je Arbeitsplatz gemerkt).
struct Zones { side: ZoneTabs<Tab>, inspector: ZoneTabs<Tab>, studio: ZoneTabs<Tab>, studio_on: bool, footer: ZoneTabs<Tab>, view: sidebar::SideView }

impl Zones {
    fn defaults(mode: Mode) -> Zones {
        let debug = mode == Mode::Debug;
        let inspector = ctx::inspector_tabs(if debug { Ctx::Debug } else { Ctx::Code });
        Zones { side: ZoneTabs::new(vec![Tab::Project]), inspector: ZoneTabs::new(inspector), studio: ZoneTabs::new(ctx::inspector_tabs(Ctx::Studio)), studio_on: false, footer: ZoneTabs::new(ctx::footer_tabs(debug)), view: sidebar::SideView::Project }
    }
    /// Reiter des Inspektors im aktuellen Kontext
    fn insp(&mut self) -> &mut ZoneTabs<Tab> { if self.studio_on { &mut self.studio } else { &mut self.inspector } }
    fn zone_mut(&mut self, z: Zone) -> &mut ZoneTabs<Tab> { match z { Zone::Side => &mut self.side, Zone::Inspector => self.insp(), Zone::Footer => &mut self.footer } }
}

/// Persönliche Einstellungen (`prefs.json` im Konfigurationsordner, Windows `%APPDATA%\vintagecoder`, sonst `~/.config/vintagecoder`): Adressspalte im Editor (Standard: an)
fn prefs_file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("prefs.json")) }
fn read_prefs() -> Option<Value> { prefs_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()) }
fn load_prefs() -> (bool, String) {
    let v = read_prefs();
    if let Some(h) = v.as_ref().and_then(|v| v["hotkeys"].as_bool()) { crate::bitmap::HOTKEYS.store(h, std::sync::atomic::Ordering::Relaxed); }
    (v.as_ref().and_then(|v| v["show_addr"].as_bool()).unwrap_or(true), v.as_ref().and_then(|v| v["run_on"].as_str().map(String::from)).unwrap_or_default())
}
fn save_prefs(show_addr: bool, run_on: &str) {
    // vorhandene Einträge (z. B. hotkeys) erhalten
    let mut v = read_prefs().filter(|v| v.is_object()).unwrap_or_else(|| serde_json::json!({}));
    v["show_addr"] = serde_json::json!(show_addr); v["run_on"] = serde_json::json!(run_on); v["hotkeys"] = serde_json::json!(crate::bitmap::HOTKEYS.load(std::sync::atomic::Ordering::Relaxed));
    if let Some(f) = prefs_file() { if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); } let _ = std::fs::write(f, format!("{v}\n")); }
}

enum Dialog {
    NewMedia { name: String, profiles: Vec<(String, String, String)>, sel: usize },
    ImportSheet { file: PathBuf, profiles: Vec<(String, String)>, sel: usize, name: String, invert: bool },
    ImportRom { file: PathBuf, cands: Vec<(String, String, usize)>, sel: usize, name: String, recog: String },
    /// `items`: (Profil|Verweis, Anzeige, gewählt); `copy`: Glyphen einbetten statt nur auf das ROM verweisen
    StdCharsets { items: Vec<(String, String, bool)>, copy: bool },
}

#[derive(Default)]
struct HelpState { doc: Option<(String, String)>, hist: Vec<(String, String)>, search: String, results: Vec<(String, String, String)>, ref_word: String, ref_doc: Option<(String, String)>, topics: Vec<(String, String, String)>, want_tab: bool, images: crate::help_images::HelpImages }

#[derive(Clone, Copy, PartialEq)]
enum Level { Info, Ok, Warn, Err }
struct Problem { file: String, line: usize, error: bool, code: String, msg: String }

#[derive(Clone, Copy, PartialEq, Eq)]
enum After { Nothing, Run, Debug }
struct BuildResult { file: String, result: Result<Value, String>, after: After }

/// Was gebaut wird: eine Datei, das ganze Projekt oder nur ein Programm des Projekts (nach Projekteinstellungen, mit deren Target)
#[derive(Clone)]
enum BuildKind { File(String), Project(Option<String>) }

/// Ergebnis des letzten erfolgreichen Baus: Artefakt, Zeilen-Zuordnung und Symbole
/// Protokoll eines Builds (Stufen, Meldungen, Dateien) für das Panel „Bauprotokoll“
struct BuildLogEntry { file: String, ok: bool, artifact: String, log: Value }

struct BuildInfo { target: String, artifact: PathBuf, lines: Vec<(String, usize, u16, usize)>, symbols: HashMap<String, i64>, labels: HashMap<u16, String>, run: Option<u16>, segments: Vec<(u32, usize)>, regions: Vec<(String, u32, u32, String)> }
impl BuildInfo {
    fn line_at(&self, pc: u16) -> Option<(&str, usize)> { self.lines.iter().find(|(_, _, a, l)| pc >= *a && (pc as usize) < *a as usize + (*l).max(1)).map(|(f, n, _, _)| (f.as_str(), *n)) }
    fn addr_of(&self, file: &str, line: usize) -> Option<u16> { self.lines.iter().find(|(f, n, _, _)| f == file && *n == line).map(|(_, _, a, _)| *a) }
}

#[derive(Default)]
struct SnapForm { name: String, screen: String, charset: String, sprites: bool, tiles: bool, use_screen: bool, use_charset: bool, for_det: Option<(u16, Option<u16>)> }

struct State {
    ctx: egui::Context,
    lsp: Option<Lsp>,
    data: Option<PathBuf>,
    project: Option<Project>,
    docs: Vec<Doc>,
    mnemonics: HashSet<String>,
    output: Vec<(Level, String)>,
    problems: Vec<Problem>,
    target: String,
    building: Option<Receiver<BuildResult>>,
    open_request: Option<String>,
    /// Einfachklick im Baum (Vorschau-Reiter) und Cmd/Strg+Klick (zweiter Editorbereich); `preview` ist der Vorschau-Reiter
    preview_request: Option<String>,
    second_request: Option<String>,
    preview: Option<String>,
    /// Wunsch aus einem Reiter (Doppelklick): Seitenleiste, Inspektor und Fußpanel zeitweise aus- bzw. einblenden
    want_zen: bool,
    /// Rückgängig (false) / Wiederholen (true), im nächsten Bild als Tastendruck einzuspeisen
    pending_undo: Option<bool>,
    status: String,
    dbg: Option<Debugger>,
    /// Reiter „Tests“: Ergebnisse, Lauf im Hintergrund; `tests_act` ist die Aktion der Ansicht, `tests_marks` der Stand der Editor-Marken (Version, Anzahl Dokumente)
    tests: crate::tests_panel::TestsState,
    tests_act: Option<crate::tests_panel::Act>,
    tests_marks: (u64, usize),
    tree_mode: bool,
    /// Wo Starten/Debuggen läuft: leer = Emulator, sonst die Id einer echten Maschine aus den Einstellungen
    run_on: String,
    /// Einstellungsseite (Entwurf) und deren Aktion
    se: Option<crate::settings_ui::SettingsEd>,
    se_act: crate::settings_ui::Act,
    /// Willkommensseite und deren Aktion
    welcome: crate::welcome::WelcomeState,
    welcome_act: Option<crate::welcome::Act>,
    /// Projekt-Übersicht (Tab „Übersicht“)
    ov: crate::overview::OverviewState,
    /// Emulator des Ziels für die Statusleiste: (Zeitpunkt, Ziel, Name, gefunden)
    emu_cache: Option<(std::time::Instant, String, String, bool)>,
    /// Kopie der Darstellung für die Einstellungsseite (die Anwendung gleicht sie jedes Bild ab)
    view: crate::theme::View,
    /// Meldungen aus Hintergrund-Threads (Start auf echter Hardware): (ok, Text)
    bg_msgs: std::sync::Arc<std::sync::Mutex<Vec<(bool, String)>>>,
    /// Aktion aus dem Reiter-Kontextmenü (führt die Anwendung aus, weil sie das Dock besitzt)
    tab_act: Option<TabAct>,
    /// „Bis hierher ausführen“: Adresse des einmaligen Haltepunkts und ob der Lauf schon beobachtet wurde
    run_to: Option<(u16, bool)>,
    /// Reiter, der nach vorn geholt werden soll (aus Kontextmenüs)
    show_tab: Option<Tab>,
    /// Aktion aus dem Kontextmenü des Projektbaums (führt die Anwendung aus)
    tree_act: Option<TreeAct>,
    /// Bereich des Projektbaums im letzten Bild (zum Zuordnen von Drag & Drop)
    tree_rect: Option<egui::Rect>,
    /// Ordner des Projektbaums mit ihren Flächen (Ablageziel beim Ziehen von Dateien); jedes Bild neu gefüllt
    tree_dirs: Vec<(String, egui::Rect)>,
    /// Ordner, der beim Ziehen von Dateien darüber aufklappen soll (einmalig)
    tree_open_req: Option<String>,
    /// Ordner unter dem Zeiger beim Ziehen von Dateien und seit wann
    drop_hover: Option<(String, std::time::Instant)>,
    /// Quelltext unter dem Mauszeiger (vom letzten Bild) und der zuletzt benutzte Assembler-Quelltext – Ziele fürs Einfügen von Includes
    editor_hover: Option<String>,
    /** Rückmeldung zum Einbinden per Drag & Drop / Menü (mit Umschalten und Rückgängig) */
    drop_note: Option<DropNote>,
    last_asm: Option<String>,
    /// Sprung zur Definition: laufende Anfrage und danach zu zeigende Zeile
    def_req: u64,
    pending_reveal: Option<(String, usize)>,
    /// Zusatzspalte im Editor: Adressen (Assembler) bzw. Pseudo-Zeilennummern (natives BASIC); wird in `prefs.json` gemerkt
    show_addr: bool,
    pe: Option<crate::projed::ProjEdit>,
    open_projed: bool,
    pick_image_request: bool,
    /// Menü „Dateien neu ordnen …"
    reorg_request: bool,
    main_request: Option<String>,
    known_targets: Vec<(String, String)>,
    known_structures: Vec<vc_ide::structure::Structure>,
    known_dialects: Vec<String>,
    pe_act: crate::projed::Act,
    info: Option<BuildInfo>,
    applied: HashSet<u16>,
    mem_input: String,
    last_pc: Option<(String, usize)>,
    disasm_toggle: Option<u16>,
    dump_form: (String, usize, String),
    snap_form: SnapForm, legacy: bool, snap_req: Option<u16>, snap_tex: Option<(usize, egui::TextureHandle)>,
    notes_seen: usize, err_seen: u32, active_tabs: Vec<Tab>,
    watch_exprs: Vec<String>,
    watch_input: String,
    bp_form: (usize, String, String, String, String),
    help: HelpState,
    buildlog: Vec<BuildLogEntry>,
    hex_view: Option<(String, Vec<u8>)>,
    active_word: String,
    active_lang: crate::highlight::Lang,
    /// Reiter „Live-Bild“ (Bildstrom der Ultimate)
    live: crate::live::LiveState,
    /// Reiter „Emulator“ (Bild des VICE-Emulators über den Monitor)
    emu: crate::emulator::EmuState,
    /// Selbsttätiges Projektbild: (Sitzung, Bildnummer bei der Anforderung); gesetzt = für diese Sitzung erledigt oder angefordert
    pic_auto: Option<(usize, u64)>,
    /// Aktiver Quelltext bzw. aktives Studio (Gliederung, Studio-Panel) und das zuletzt berechnete Studio-Panel (Dokument, Zeit, Inhalt)
    ctx_doc: Option<String>, ctx_studio: Option<u8>, studio_cache: Option<(String, f64, ctx::StudioInfo)>,
}

pub struct App { about_open: bool, feedback: Option<crate::feedback::Dlg>, splash: crate::splash::Splash, rom_wizard: Option<crate::rom_ui::RomWizard>, rom_error: Option<String>, repaint: crate::shell::RepaintGuard, palette: crate::cmdpalette::Palette, diskimg: Option<crate::disk_studio::DiskStudio>, charpick: Option<crate::charpick::CharPick>, charpick_doc: Option<String>, view: crate::theme::View, reqs: Requests, wizard: Option<crate::wizard::Wizard>, setup: Option<crate::setup_wizard::SetupWizard>, dialog: Option<Dialog>, file_dlg: Option<FileDlg>, import_dlg: Option<crate::importdlg::ImportDlg>, reorg_dlg: Option<crate::reorgdlg::ReorgDlg>, reorg_job: Option<std::sync::mpsc::Receiver<crate::reorgdlg::Msg>>, /** Quelltexte, die beim Neu ordnen offen waren und nur angepasst wurden: danach wieder öffnen */ reorg_reopen: Vec<String>, mode: Mode, layout: LayoutFile, layout_saved: String, layout_t: std::time::Instant, session_last: Option<(PathBuf, crate::session::ProjSession)>, layout_gen: u32, ctx: Ctx, side: sidebar::SideUi, zen: bool, stop_dbg_request: bool,
    /// Editorbereich (nur hier gibt es ein `egui_dock`): Dokumente, Studios, Willkommen, Übersicht, Einstellungen
    dock: DockState<Tab>,
    /// Reiter der festen Zonen je Arbeitsplatz (Code, Debug)
    zones: [Zones; 2],
    st: State, frames: u32, pal: Palettes, audio: crate::audio::Audio, wins: Vec<MediaWindow>, native: crate::nativemenu::Native }

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, arg: Option<PathBuf>) -> App {
        let view0 = crate::theme::View::load();
        crate::fonts::install(&cc.egui_ctx);
        crate::winctl::load();
        crate::ui::style::install(&cc.egui_ctx);
        view0.apply(&cc.egui_ctx);
        let layout = if std::env::var("VC_SHOT").is_ok() { LayoutFile::default() } else { layout::file().map(|f| layout::load_from(&f)).unwrap_or_default() };   // Selbsttests nutzen und überschreiben keine gespeicherte Anordnung
        windows::set_detached_prefs(&layout.detached_studios); windows::set_rects(&layout.studio_rects);
        let data = std::env::var("VC_DATA").ok().map(PathBuf::from)
            .or_else(|| std::env::current_exe().ok().and_then(|e| vc_ide::find_data_dir(&e)))      // zuerst neben dem Programm (installierte Version)
            .or_else(|| std::env::current_dir().ok().and_then(|d| vc_ide::find_data_dir(&d)))
            .or_else(|| vc_ide::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))));      // DevBuild: Quellbaum, aus dem gebaut wurde
        if let Some(w) = vc_ide::features::init(data.as_deref()) { eprintln!("Warnung: {w}"); }
        let welcome = crate::welcome::WelcomeState::new(data.as_deref());
        let mut st = State { drop_note: None, tests: Default::default(), tests_act: None, tests_marks: (0, 0), welcome, welcome_act: None, ov: Default::default(), emu_cache: None, view: crate::theme::View::default(), ctx: cc.egui_ctx.clone(), lsp: None, data, project: None, docs: vec![], mnemonics: HashSet::new(), output: vec![], problems: vec![], target: "auto".into(), building: None, open_request: None, preview_request: None, second_request: None, preview: None, want_zen: false, pending_undo: None, status: "bereit".into(), dbg: None, tree_mode: true, tab_act: None, run_to: None, show_tab: None, tree_act: None, tree_rect: None, tree_dirs: vec![], tree_open_req: None, drop_hover: None, editor_hover: None, last_asm: None, def_req: 0, pending_reveal: None, show_addr: load_prefs().0, run_on: load_prefs().1, se: None, se_act: crate::settings_ui::Act::None, bg_msgs: Default::default(), open_projed: false, pick_image_request: false, reorg_request: false, main_request: None, pe: None, known_targets: vec![], known_dialects: vec![], known_structures: vec![], pe_act: crate::projed::Act::None, info: None, applied: HashSet::new(), mem_input: "0400".into(), last_pc: None, disasm_toggle: None, dump_form: (String::new(), 0, "dump_zeichensatz".into()), snap_req: None, snap_tex: None, legacy: false, snap_form: SnapForm { name: "snap".into(), sprites: true, use_screen: true, use_charset: true, ..Default::default() }, notes_seen: 0, err_seen: 0, active_tabs: vec![], watch_exprs: vec![], watch_input: String::new(), bp_form: (0, String::new(), String::new(), String::new(), String::new()), help: HelpState::default(), buildlog: vec![], hex_view: None, active_word: String::new(), active_lang: crate::highlight::Lang::Plain, live: Default::default(), emu: Default::default(), pic_auto: None, ctx_doc: None, ctx_studio: None, studio_cache: None };
        if let Some(d) = &st.data { for cpu in ["6510", "sally", "45gs02"] { if let Ok(c) = vc_target::load_cpu(d, cpu) { st.mnemonics.extend(c.ops.iter().map(|o| o.mnemonic.to_ascii_lowercase())); } } }
        match &st.data { Some(d) => st.log(Level::Info, format!("Datenbank: {}", d.display())), None => st.log(Level::Err, "Hardware-Datenbank (data/) nicht gefunden – VC_DATA setzen") }
        let mut app = App { about_open: false, feedback: None, splash: { let mut sp = crate::splash::Splash::new(); if std::env::var("VC_SHOT").is_ok() { sp.open = std::env::var("VC_DEMO").map_or(false, |d| d.contains("splash")); } sp }, rom_wizard: None, setup: None, rom_error: None, repaint: Default::default(), palette: Default::default(), diskimg: None, charpick: None, charpick_doc: None, view: crate::theme::View::load(), reqs: Requests::default(), wizard: None, dialog: None, file_dlg: None, import_dlg: None, reorg_dlg: None, reorg_job: None, reorg_reopen: vec![], mode: Mode::Code, layout_saved: layout.to_json(), layout, layout_t: std::time::Instant::now(), session_last: None, layout_gen: 0, ctx: Ctx::Code, side: Default::default(), zen: false, stop_dbg_request: false, dock: DockState::new(vec![Tab::Welcome]), zones: [Zones::defaults(Mode::Code), Zones::defaults(Mode::Debug)], pal: Palettes::load(st.data.as_deref().unwrap_or(std::path::Path::new(""))), st, frames: 0, audio: crate::audio::Audio::new(), wins: vec![], native: crate::nativemenu::Native::new(&cc.egui_ctx) };
        app.start_setup();
        if let Some(p) = arg { app.open_project_path(p); if let Ok(f) = std::env::var("VC_OPEN") { app.open_doc(&f); } }
        app
    }

    /// Zahlen und Schalter der Zonen speichern (nie Reiter); nur bei Änderung
    fn save_layout(&mut self) {
        if std::env::var("VC_SHOT").is_ok() { return; }
        self.layout.detached_studios = windows::detached_prefs(); self.layout.studio_rects = windows::rects();
        let j = self.layout.to_json();
        if j == self.layout_saved { return; }
        if let Some(f) = layout::file() { if layout::save_to(&f, &self.layout) { self.layout_saved = j; } }
    }

    /// Offene Dateien des Projekts (Reiter und Studio-Dokumente) und die aktive Datei, nur Pfade
    fn current_session(&mut self) -> Option<crate::session::ProjSession> {
        self.st.project.as_ref()?;
        let mut open: Vec<String> = self.dock.iter_all_tabs().filter_map(|(_, t)| if let Tab::Doc(r) = t { Some(r.clone()) } else { None }).collect();
        for w in self.wins.iter().filter(|w| w.embedded) { for d in &w.docs { if !open.contains(&d.rel) { open.push(d.rel.clone()); } } }
        let active = match self.dock.find_active_focused() {
            Some((_, Tab::Doc(r))) => Some(r.clone()),
            Some((_, Tab::Studio(g))) => self.wins.iter_mut().find(|w| w.group as u8 == *g).and_then(|w| w.dock.find_active_focused().map(|(_, t)| t.clone())).filter(|r| open.contains(r)),
            _ => None,
        };
        Some(crate::session::ProjSession { open, active })
    }

    /// Offene Dateien in `session.json` merken (nur bei Änderung, nur wenn das Projekt es verlangt)
    fn save_session(&mut self) {
        if std::env::var("VC_SHOT").is_ok() { return; }
        let Some(root) = self.st.project.as_ref().filter(|p| p.pf.as_ref().map_or(false, |pf| pf.restore_tabs)).map(|p| p.root.clone()) else { return };
        let Some(s) = self.current_session() else { return };
        if self.session_last.as_ref() == Some(&(root.clone(), s.clone())) { return; }
        if let Some(f) = crate::session::file() { if crate::session::save_to(&f, &root, &s) { self.session_last = Some((root, s)); } }
    }

    fn layout_wp(&self) -> Workplace { *self.layout.workplace(self.mode == Mode::Debug) }
    fn mode_ix(&self) -> usize { match self.mode { Mode::Code => 0, Mode::Debug => 1 } }
    /// Größen und Schalter des aktuellen Arbeitsplatzes
    fn wp(&mut self) -> &mut Workplace { self.layout.workplace_mut(self.mode == Mode::Debug) }
    fn zones_mut(&mut self) -> &mut Zones { let i = self.mode_ix(); &mut self.zones[i] }

    /// Arbeitsplatz wechseln: nur die Panels der Zonen ändern sich, der Editorbereich bleibt wie er ist
    fn switch_mode(&mut self, mode: Mode) { self.mode = mode; }

    /// „Ansicht ▸ Layout zurücksetzen“: Standardgrößen, alle Zonen sichtbar, Standardreiter, Editorbereich ungeteilt
    fn reset_layout(&mut self) {
        let (detached, rects) = (self.layout.detached_studios.clone(), self.layout.studio_rects.clone());
        self.layout = LayoutFile { detached_studios: detached, studio_rects: rects, ..Default::default() };
        self.zones = [Zones::defaults(Mode::Code), Zones::defaults(Mode::Debug)];
        self.layout_gen += 1;   // neue Panel-Kennungen: egui vergisst die gemerkten Größen
        self.zen = false;
        layout::unsplit(&mut self.dock);
    }

    /// Ein Panel einer festen Zone nach vorn holen und die Zone einblenden; ein Reiter des Editorbereichs wird dort aktiv
    fn reveal(&mut self, t: Tab) {
        match t.zone() {
            Some(z) => {
                self.zen = false;
                { let w = self.wp(); match z { Zone::Side => w.side = true, Zone::Inspector => w.inspector = true, Zone::Footer => w.footer = true } }
                self.zones_mut().zone_mut(z).show(&t);
                self.open_drawer_if_narrow(z);
            }
            None => { if let Some(l) = self.dock.find_tab(&t) { let _ = self.dock.set_active_tab(l); self.dock.set_focused_node_and_surface(NodePath { surface: l.surface, node: l.node }); } }
        }
    }

    /// Reiter im aktiven Editorbereich öffnen oder nach vorn holen; das Willkommen-Platzhalter verschwindet
    fn show_in_editor(&mut self, tab: Tab) {
        layout::show_unique(&mut self.dock, tab.clone());
        if tab != Tab::Welcome { if let Some(l) = self.dock.find_tab(&Tab::Welcome) { self.dock.remove_tab(l); } }
    }

    /// Startordner der Dateidialoge „Projekt öffnen“: Ordner des geöffneten Projekts, sonst das Basisverzeichnis der Projekte (falls vorhanden)
    pub(super) fn open_start_dir(&self) -> Option<std::path::PathBuf> {
        self.st.project.as_ref().and_then(|p| p.root.parent().map(|x| x.to_path_buf())).or_else(projects_base_dir)
    }

    fn new_project_dialog(&mut self) {
        let Some(data) = &self.st.data else { return };
        let base = projects_base_dir();
        let parent = self.st.project.as_ref().and_then(|p| p.root.parent().map(|x| x.to_path_buf())).or(base).or_else(vc_core::userdirs::home_dir).unwrap_or_default();
        self.wizard = crate::wizard::Wizard::new(data, parent);
        if self.wizard.is_none() { self.st.log(Level::Err, "Vorlagen (data/templates) nicht lesbar"); }
    }

    /// Beim Programmstart: Erststart öffnet den Assistenten, sonst gegebenenfalls ein Hinweis auf der Willkommensseite.
    /// `VC_NO_WIZARD=1` unterdrückt alles; Selbsttests (`VC_SHOT`) öffnen ihn nur mit `VC_WIZ_STEP` oder `VC_WIZ_MODE`.
    fn start_setup(&mut self) {
        use crate::setup_wizard::Launch;
        if std::env::var("VC_SHOT").is_ok() {
            if let Ok(m) = std::env::var("VC_WIZ_MODE") { self.open_setup(if matches!(m.as_str(), "recall" | "roms") { Launch::Recall } else { Launch::FirstRun }); }
            else if std::env::var("VC_WIZ_STEP").is_ok() { self.open_setup(Launch::FirstRun); }
            return;
        }
        if vc_ide::setup::auto_start_suppressed() { return; }
        let Some(paths) = vc_ide::setup::Paths::current() else { return };
        let w = vc_ide::setup::WizardState::load(&paths.wizard);
        match vc_ide::setup::start_state(&paths) {
            vc_ide::setup::Start::FirstRun => self.open_setup(Launch::FirstRun),
            vc_ide::setup::Start::OfferOnce => self.st.welcome.banner = Some(crate::welcome::Banner::OfferOnce),
            vc_ide::setup::Start::Normal => {
                if w.incomplete() { self.st.welcome.banner = Some(crate::welcome::Banner::Incomplete); }
                else if !vc_ide::setup::news_since_last_run(&w, &|k| vc_ide::platform::discover(k)).is_empty() { self.st.welcome.banner = Some(crate::welcome::Banner::News); }
            }
        }
    }

    fn open_setup(&mut self, launch: crate::setup_wizard::Launch) {
        let mut env = crate::setup_wizard::Env { view: &mut self.view, show_addr: &mut self.st.show_addr, data: self.st.data.as_deref() };
        match crate::setup_wizard::SetupWizard::new(launch, &mut env) {
            Some(w) => { self.st.welcome.banner = None; self.setup = Some(w); }
            None => self.st.log(Level::Err, "Kein Ordner für die Einstellungen (HOME bzw. unter Windows APPDATA fehlt)"),
        }
    }

    fn show_setup(&mut self, ctx: &egui::Context) {
        use crate::setup_wizard::Out;
        if ctx.data_mut(|d| d.remove_temp::<bool>(egui::Id::new("setup_request"))).unwrap_or(false) { self.open_setup(crate::setup_wizard::Launch::Recall); }
        let Some(w) = &mut self.setup else { return };
        let before = self.view;
        let mut env = crate::setup_wizard::Env { view: &mut self.view, show_addr: &mut self.st.show_addr, data: self.st.data.as_deref() };
        let out = w.show(ctx, &mut env);
        if self.view != before { self.view.apply(ctx); }   // Darstellung wirkt sofort; gespeichert wird erst bei „Übernehmen“
        match out {
            Out::None => {}
            Out::Close => self.setup = None,
            Out::Applied(v) | Out::Undone(v) => {
                self.view = v; self.view.apply(ctx);
                self.st.se = None;   // Einstellungsseite liest den neuen Stand
                let ids: Vec<String> = vc_ide::settings::get().machines.iter().map(|m| m.id.clone()).collect();
                if !self.st.run_on.is_empty() && self.st.run_on != "@emulator" && !ids.contains(&self.st.run_on) { self.st.run_on.clear(); save_prefs(self.st.show_addr, ""); }
                vc_ide::roms::reconfigure_media();
                self.st.log(Level::Ok, "Konfiguration übernommen (Sicherung: settings.json.bak)");
            }
            Out::OpenProject(p) => { self.setup = None; self.open_project_path(p); }
            Out::NewProject => { self.setup = None; self.new_project_dialog(); }
            Out::Help(id) => { self.setup = None; self.st.help_open(id); self.st.show_tab = Some(Tab::Help); }
        }
    }

    fn show_wizard(&mut self, ctx: &egui::Context) {
        let Some(w) = &mut self.wizard else { return };
        match w.show(ctx) {
            crate::wizard::Out::None => {}
            crate::wizard::Out::Cancel => self.wizard = None,
            crate::wizard::Out::Created(dir, notes) => { self.wizard = None; for n in notes { self.st.log(Level::Warn, n); } self.open_project_path(dir); }
        }
    }

    fn new_media_dialog(&mut self) { self.new_media_dialog_for(None); }

    /// „Neu anlegen“ für Mediendateien; mit `only` nur die Profile dieser Art (Sprite, Zeichensatz, Bildschirm, Kacheln, Palette, Song, PLAY)
    fn new_media_dialog_for(&mut self, only: Option<Group>) {
        let (Some(data), Some(pr)) = (&self.st.data, &self.st.project) else { return };
        let sys = vc_media::project::project_systems(&pr.root, data);
        let Ok(list) = vc_media::load_profiles(data) else { return };
        let profiles: Vec<(String, String, String)> = list.iter()    .filter(|p| sys.is_empty() || sys.contains(&p.target)).filter(|p| only.map_or(true, |g| Group::of_kind(&p.kind) == g)).map(|p| (p.id.clone(), p.name.clone(), p.kind.clone())).collect();
        if profiles.is_empty() { self.st.log(Level::Warn, match only { Some(g) => format!("Für die Zielsysteme dieses Projekts gibt es keine Profile für „{}“", g.title()), None => "Für die Zielsysteme dieses Projekts gibt es keine Profile für Sprites, Zeichensätze oder Songs".to_string() }); return; }
        self.dialog = Some(Dialog::NewMedia { name: "neu".into(), profiles, sel: 0 });
    }

    fn import_sheet_dialog(&mut self) {
        let (Some(pr), Some(data)) = (&self.st.project, self.st.data.clone()) else { return };
        // Selbsttest (Screenshots): VC_SHEET = PNG-Datei statt Dateiwahl
        let picked = std::env::var("VC_SHOT").ok().and_then(|_| std::env::var("VC_SHEET").ok()).map(PathBuf::from);
        let Some(file) = picked.or_else(|| rfd::FileDialog::new().set_title("PNG-Raster mit Zeichensatz wählen (16 Zeichen je Zeile)").add_filter("PNG", &["png"]).pick_file()) else { return };
        let sys = vc_media::project::project_systems(&pr.root, &data);
        let profiles: Vec<(String, String)> = vc_media::load_profiles(&data).unwrap_or_default().iter().filter(|p| p.kind == "charset" && (p.bpp == 1 || p.bpp == 8) && (sys.is_empty() || sys.contains(&p.target))).map(|p| (p.id.clone(), p.name.clone())).collect();
        if profiles.is_empty() { self.st.log(Level::Warn, "Für die Zielsysteme dieses Projekts gibt es kein passendes Zeichensatz-Profil"); return; }
        let name = file.file_stem().map(|n| n.to_string_lossy().to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()).unwrap_or_else(|| "zeichensatz".into());
        self.dialog = Some(Dialog::ImportSheet { file, profiles, sel: 0, name, invert: false });
    }

    /// Standardzeichensätze der Zielsysteme des Projekts zur Auswahl anbieten
    fn std_charset_dialog(&mut self) {
        let Some(pr) = &self.st.project else { return };
        let data = self.st.data.clone().unwrap_or_default();
        let sys = vc_media::project::project_systems(&pr.root, &data);
        let items: Vec<(String, String, bool)> = vc_media::project::std_charset_list(&data).into_iter().filter(|c| sys.is_empty() || sys.contains(&c.system))
            .map(|c| (format!("{}|{}", c.profile, c.reference), if c.status == vc_media::Status::Missing { format!("{}  (ROM fehlt)", c.label) } else { c.label }, false)).collect();
        if items.is_empty() { self.st.log(Level::Warn, "Keine Standardzeichensätze für die Zielsysteme dieses Projekts".to_string()); return; }
        self.dialog = Some(Dialog::StdCharsets { items, copy: false });
    }

    fn import_rom_dialog(&mut self) {
        let Some(pr) = &self.st.project else { return };
        let Some(file) = rfd::FileDialog::new().set_title("ROM-Abbild mit Zeichensatz wählen").add_filter("ROM / Binärdatei", &["rom", "bin", "ROM", "BIN"]).pick_file() else { return };
        let len = std::fs::metadata(&file).map(|m| m.len() as usize).unwrap_or(0);
        let rec = vc_media::project::recognize_rom(&file);
        let recog = match &rec.label { Some(l) => format!("Erkannt: {l} ({})", rec.status.label()), None => "Nicht erkannt – Vorschläge nach der Dateigröße".to_string() };
        let data = self.st.data.clone().unwrap_or_default();
        let sys = vc_media::project::project_systems(&pr.root, &data);
        let profiles = vc_media::load_profiles(&data).unwrap_or_default();
        let cands: Vec<(String, String, usize)> = rec.sets.into_iter()
            .filter(|(_, p, _)| profiles.iter().any(|x| x.id == *p && (sys.is_empty() || sys.contains(&x.target)))).collect();
        if cands.is_empty() { self.st.log(Level::Warn, format!("{}: {len} Bytes – kein bekanntes ROM-Format für die Zielsysteme dieses Projekts (erwartet: C64/MEGA65 2048 oder 4096, Atari 1024, 8192, 10240 oder 16384 Bytes)", file.display())); return; }
        let name = file.file_stem().map(|n| format!("zs_{}", n.to_string_lossy().to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>())).unwrap_or_else(|| "rom_zeichensatz".into());
        self.dialog = Some(Dialog::ImportRom { file, cands, sel: 0, name, recog });
    }

    /// Assistent „ROMs einrichten“ und Fehlerdialog „ROM fehlt“
    fn show_rom_ui(&mut self, ctx: &egui::Context) {
        if ctx.data_mut(|d| d.remove_temp::<bool>(egui::Id::new("rom_setup_request"))).unwrap_or(false) { self.open_rom_wizard(); }
        if let Some(msg) = self.rom_error.clone() {
            match crate::rom_ui::error_dialog(ctx, &msg) {
                crate::rom_ui::ErrAct::Setup => { self.rom_error = None; self.open_rom_wizard(); }
                crate::rom_ui::ErrAct::Close => self.rom_error = None,
                crate::rom_ui::ErrAct::None => {}
            }
        }
        if let Some(w) = &mut self.rom_wizard { if !w.show(ctx) { self.rom_wizard = None; vc_ide::roms::reconfigure_media(); } }
    }

    /// Assistent öffnen; die Zielsysteme des geöffneten Projekts sind vorgewählt
    fn open_rom_wizard(&mut self) {
        let ids: Vec<String> = self.st.project.as_ref().map(|p| vc_media::project::project_systems(&p.root, &self.st.data.clone().unwrap_or_default())).unwrap_or_default();
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        self.rom_wizard = Some(crate::rom_ui::RomWizard::new(&refs));
    }

    /// Fehlermeldung des Starts: fehlende ROM als Dialog mit Weg zur Abhilfe, sonst nur ins Protokoll
    fn log_start_error(&mut self, e: String) {
        if crate::rom_ui::is_rom_error(&e) { self.rom_error = Some(e.clone()); }
        self.st.log(Level::Err, e);
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
        self.show_rom_ui(ctx);
        let Some(mut d) = self.dialog.take() else { return };
        let mut keep = true; let mut create_media: Option<(String, String)> = None; let mut import_rom: Option<(PathBuf, String, usize, String)> = None; let mut std_charsets: Option<(Vec<String>, bool)> = None; let mut import_sheet: Option<(PathBuf, String, bool, String)> = None;
        let valid = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        let act;
        match &mut d {
            Dialog::ImportSheet { file, profiles, sel, name, invert } => {
                let shown = file.display().to_string();
                act = dialog::show(ctx, &Dlg::new("Zeichensatz aus PNG-Raster", "Importieren").sub(&shown), |ui| {
                    widgets::card(ui, false, |ui| { ui.label("Profil"); for (i, (_, n)) in profiles.iter().enumerate() { ui.radio_value(sel, i, n.as_str()); } });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| { widgets::label(ui, "Name"); ui.text_edit_singleline(name); });
                    ui.checkbox(invert, "Dunkle Pixel sind gesetzt (statt heller)");
                    ui.label(RichText::new("Erwartet 16 Zeichen je Zeile in Leserichtung; Zeichen dürfen ganzzahlig vergrößert sein (z. B. 128×128 für 256 Zeichen zu 8×8).").weak().small());
                    valid(name)
                });
                if act == Act::Primary { import_sheet = Some((file.clone(), profiles[*sel].0.clone(), *invert, name.clone())); }
            }
            Dialog::StdCharsets { items, copy } => {
                let n = items.iter().filter(|i| i.2).count();
                let label = format!("Hinzufügen ({n})");
                act = dialog::show(ctx, &Dlg::new("Standardzeichensatz hinzufügen", &label).sub("Verweis auf deine ROMs, Ziel: graphics/"), |ui| {
                    ui.label("Welche Zeichensätze sollen ins Projekt?");
                    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| { for (_, label, on) in items.iter_mut() { ui.checkbox(on, label.as_str()); } });
                    ui.label(RichText::new("Es wird nur ein Verweis auf das ROM deines Systems angelegt; die Original-Zeichen stehen nicht im Projekt. Fehlt das ROM, zeigt das Studio ein leeres Raster mit Hinweis.").weak().small());
                    ui.checkbox(copy, "Zeichen stattdessen einbetten (Kopie in graphics/standard/)");
                    if *copy { ui.label(RichText::new("Die Zeichensätze stammen aus den ROMs der Systeme und sind urheberrechtlich geschützt – vor einer Veröffentlichung des Projekts prüfen.").weak().small()); }
                    n > 0
                });
                if act == Act::Primary { std_charsets = Some((items.iter().filter(|i| i.2).map(|i| i.0.clone()).collect(), *copy)); }
            }
            Dialog::ImportRom { file, cands, sel, name, recog } => {
                let shown = file.display().to_string();
                act = dialog::show(ctx, &Dlg::new("Zeichensatz aus ROM importieren", "Importieren").sub(&shown), |ui| {
                    ui.label(RichText::new(recog.as_str()).weak());
                    widgets::card(ui, false, |ui| { ui.label("Welcher Zeichensatz?"); for (i, (d, _, o)) in cands.iter().enumerate() { ui.radio_value(sel, i, format!("{d}  (ab ${o:X})")); } });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| { widgets::label(ui, "Name"); ui.text_edit_singleline(name); });
                    ui.label(RichText::new("Wird als Zeichensatz in graphics/standard/ angelegt. ROM-Abbilder können urheberrechtlich geschützt sein – vor einer Veröffentlichung des Projekts prüfen.").weak().small());
                    valid(name)
                });
                if act == Act::Primary { import_rom = Some((file.clone(), cands[*sel].1.clone(), cands[*sel].2, name.clone())); }
            }
            Dialog::NewMedia { name, profiles, sel } => {
                act = dialog::show(ctx, &Dlg::new("Neu anlegen", "Anlegen"), |ui| {
                    ui.horizontal(|ui| { widgets::label(ui, "Name"); ui.text_edit_singleline(name); });
                    ui.add_space(4.0);
                    widgets::card(ui, false, |ui| { ui.label("Profil"); for (i, (_, n, _)) in profiles.iter().enumerate() { ui.radio_value(sel, i, n.as_str()); } });
                    valid(name)
                });
                if act == Act::Primary { create_media = Some((name.clone(), profiles[*sel].0.clone())); }
            }
        }
        if act != Act::None { keep = false; }
        if keep { self.dialog = Some(d); }
        if let (Some((file, profile, invert, name)), Some(data), Some(root)) = (import_sheet, self.st.data.clone(), self.st.project.as_ref().map(|p| p.root.clone())) {
            match vc_media::project::import_sheet(&root, &data, &file, &profile, invert, &name, "") {
                Ok(v) => { let rel = v["path"].as_str().unwrap_or("").to_string(); if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Zeichensatz importiert: {rel}")); self.open_media(&rel); }
                Err(e) => self.st.log(Level::Err, e),
            }
        }
        if let (Some((ids, copy)), Some(_data), Some(root)) = (std_charsets, self.st.data.clone(), self.st.project.as_ref().map(|p| p.root.clone())) {
            let mut first: Option<String> = None;
            for id in ids {
                let (profile, reference) = id.split_once('|').unwrap_or(("", &id));
                match vc_media::project::add_std_charset(&root, reference, profile, if copy { "graphics/standard" } else { "graphics" }, copy) {
                    Ok(v) => {
                        let rel = v["path"].as_str().unwrap_or("").to_string(); self.st.log(Level::Ok, format!("Standardzeichensatz hinzugefügt: {rel}"));
                        if let Some(w) = v["warning"].as_str() { self.st.log(Level::Warn, w.to_string()); }
                        first.get_or_insert(rel);
                    }
                    Err(e) => self.st.log(Level::Warn, e),
                }
            }
            if let Some(pr) = &mut self.st.project { pr.rescan(); }
            if let Some(rel) = first { self.open_media(&rel); }
        }
        if let (Some((file, profile, offset, name)), Some(data), Some(root)) = (import_rom, self.st.data.clone(), self.st.project.as_ref().map(|p| p.root.clone())) {
            match vc_media::project::import_rom(&root, &data, &file, &profile, offset, &name, "graphics/standard") {
                Ok(v) => { let rel = v["path"].as_str().unwrap_or("").to_string(); if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Zeichensatz importiert: {rel}")); self.open_media(&rel); }
                Err(e) => self.st.log(Level::Err, e),
            }
        }
        if let (Some((name, profile)), Some(data), Some(root)) = (create_media, self.st.data.clone(), self.st.project.as_ref().map(|p| p.root.clone())) {
            match vc_media::project::create(&root, &data, &name, &profile, "") {
                Ok(v) => { let rel = v["path"].as_str().unwrap_or("").to_string(); if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Angelegt: {rel}")); self.open_media(&rel); }
                Err(e) => self.st.log(Level::Err, e),
            }
        }
    }

    /// Altes Dateischema (`*.asset.json`, `assets/`) auf die neuen Namen umstellen und das Projekt neu laden
    fn migrate_project(&mut self) {
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        self.save_active();
        match vc_media::naming::migrate(&root) {
            Ok(m) => { self.st.log(Level::Ok, format!("{} Datei(en) umbenannt (z. B. {})", m.len(), m.first().map(|x| format!("{} → {}", x.0, x.1)).unwrap_or_default())); self.open_project_path(root); }
            Err(e) => self.st.log(Level::Err, format!("Umstellen: {e}")),
        }
    }

    fn open_project_path(&mut self, p: PathBuf) {
        let file = if p.is_dir() { std::fs::read_dir(&p).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|f| f.extension().map_or(false, |x| x == "vcprj"))) } else { Some(p.clone()) };
        let res = match file { Some(f) if f.is_file() => Project::open(&f), _ => Project::load(p.clone(), None) };
        match res {
            Ok(pr) => {
                self.save_session();
                self.st.docs.clear(); self.wins.clear(); self.st.pe = None;
                let docs: Vec<Tab> = self.dock.iter_all_tabs().filter_map(|(_, t)| if let Tab::Doc(_) | Tab::ProjectEdit | Tab::Settings | Tab::Overview | Tab::Studio(_) = t { Some(t.clone()) } else { None }).collect();
                for t in docs { if let Some(loc) = self.dock.find_tab(&t) { self.dock.remove_tab(loc); } }
                self.st.log(Level::Ok, format!("Projekt geöffnet: {} ({})", pr.name, pr.root.display()));
                self.st.legacy = vc_media::naming::has_legacy(&pr.root);
                if self.st.legacy { self.st.log(Level::Warn, "Dieses Projekt nutzt noch das alte Dateischema (*.asset.json, assets/). Es funktioniert weiter; „Datei ▸ Projekt auf neue Dateinamen umstellen“ benennt alles um (Verweise werden angepasst)."); }
                self.st.target = "auto".into(); self.st.problems.clear();
                self.st.lsp = self.st.data.as_ref().map(|d| Lsp::start(&pr.root, d, self.st.ctx.clone()));
                crate::recent::touch(&mut self.st.welcome.recent, &pr.root, &pr.name, self.st.data.as_deref());
                vc_ide::roms::configure_media(Some(&pr.root));
                self.st.project = Some(pr);
                let restore = self.st.project.as_ref().filter(|p| p.pf.as_ref().map_or(false, |pf| pf.restore_tabs)).and_then(|p| {
                    let s = crate::session::file().and_then(|f| crate::session::load_from(&f, &p.root))?;
                    Some(crate::session::plan_restore(&s, |r| p.root.join(r).is_file()))
                }).unwrap_or_default();
                if restore.is_empty() {
                    let first = self.st.project.as_ref().and_then(|p| p.entries.iter().find(|e| e.group == Group::Asm || e.group == Group::Basic).map(|e| e.rel.clone()));
                    if let Some(f) = first { self.open_doc(&f); }
                } else { for r in &restore { self.open_doc(r); } }
            }
            Err(e) => self.st.log(Level::Err, e),
        }
    }

    /// Projekt-Übersicht: Ereignisse mitschreiben, Tab beim Öffnen eines Projekts anlegen, Wünsche der Übersicht ausführen
    /// Reiter „Tests“: Lauf nachführen, Marken der Editoren aktualisieren, Aktionen der Ansicht ausführen
    fn run_tests(&mut self, ctx: &egui::Context) {
        use crate::tests_panel::Act;
        self.st.tests.poll();
        let root = self.st.project.as_ref().map(|p| p.root.clone());
        if let Some(root) = &root {
            let key = (self.st.tests.version, self.st.docs.len());
            if key != self.st.tests_marks {
                self.st.tests_marks = key;
                for d in &mut self.st.docs { d.test_marks = self.st.tests.suite.marks_for(root, &d.rel); }
            }
        }
        let Some(act) = self.st.tests_act.take() else { return };
        match act {
            Act::Stop => { self.st.tests.stop_blocking(); self.st.log(Level::Info, "Testlauf abgebrochen"); }
            Act::Source(f, line) => {
                let rel = root.as_deref().map(|r| vc_ide::tests::suite::rel_to(r, &f)).unwrap_or(f);
                self.st.open_request = Some(rel.clone());
                if line > 0 { self.st.pending_reveal = Some((rel, line - 1)); }
            }
            Act::Run(only) => {
                let (Some(root), Some(data)) = (root, self.st.data.clone()) else { return };
                if self.st.tests.backend.is_some() { self.stop_debug(); vc_ide::launch::stop_running(); }        // nur ein Emulator gleichzeitig
                self.st.tests.start(root, data, only, false, ctx);
                self.st.status = "Tests laufen …".into();
            }
            Act::Debug(file, test) => {
                let Some(data) = self.st.data.clone() else { return };
                self.st.tests.stop_blocking(); self.stop_debug(); vc_ide::launch::stop_running();
                self.st.notes_seen = 0; self.st.err_seen = 0; self.st.applied.clear();
                self.st.dbg = Some(Debugger::start_test(file, data, test.clone(), ctx.clone()));
                self.st.log(Level::Info, format!("Test „{test}“ wird im Debugger vorbereitet …")); self.st.status = "Emulator startet …".into();
                self.switch_mode(Mode::Debug);
            }
        }
    }

    fn run_overview(&mut self, ctx: &egui::Context) {
        use crate::overview::{Act, Kind};
        self.st.ov.observe(self.st.output.iter().map(|(l, s)| (match l { Level::Info => Kind::Info, Level::Ok => Kind::Ok, Level::Warn => Kind::Warn, Level::Err => Kind::Err }, s.as_str())));
        let root = self.st.project.as_ref().map(|p| p.root.clone());
        if root != self.st.ov.shown_for {
            self.st.ov.reset();
            if root.is_some() && self.dock.find_tab(&Tab::Overview).is_none() { self.show_in_editor(Tab::Overview); }
            // Selbsttest (Screenshots): VC_OPEN soll vorne bleiben, die Übersicht öffnet sich sonst darüber (zeigen: VC_DEMO=showov)
            if std::env::var("VC_SHOT").is_ok() && !std::env::var("VC_DEMO").map_or(false, |d| d.contains("showov")) { if let Ok(f) = std::env::var("VC_OPEN") { self.open_doc(&f); } }
            self.st.ov.shown_for = root;
        }
        if let Some(a) = self.st.ov.act.take() {
            match a {
                Act::Build => self.build(ctx, After::Nothing),
                Act::Run => self.build(ctx, After::Run),
                Act::NewMedia => self.new_media_dialog(),
                Act::PickImage => self.st.pick_image_request = true,
                Act::OpenDoc(r) => self.st.open_request = Some(r),
                Act::ProjectEdit => self.st.open_projed = true,
                Act::Settings => self.open_settings(),
            }
        }
    }

    /// Fester Reiter im aktiven Editorbereich (oder Sprung zum vorhandenen)
    fn open_doc(&mut self, rel: &str) { self.open_doc_as(rel, Open::Fixed); }

    /// Projektdatei (`.vcprj`) als Quelltext öffnen (Kontextmenü „Als Text öffnen“)
    fn open_doc_text(&mut self, rel: &str) { self.open_doc_as_text(rel, Open::Fixed); }

    /// Öffnen nach den Regeln des Layoutkonzepts. Studios und Disketten-Images folgen ihren eigenen Wegen, Textdateien den Regeln `how`.
    fn open_doc_as(&mut self, rel: &str, how: Open) { self.open_doc_impl(rel, how, false); }

    fn open_doc_as_text(&mut self, rel: &str, how: Open) { self.open_doc_impl(rel, how, true); }

    fn open_doc_impl(&mut self, rel: &str, how: Open, as_text: bool) {
        if vc_media::naming::is_data_file(rel) {
            // Studio nicht im Release: die Datei öffnet als Text (Konzept Release-Auswahl 5.3), der Bau bleibt unberührt
            match self.missing_studio(rel) { None => { self.open_media(rel); return; } Some(title) => { let note = crate::studios::missing_note(title); self.st.status = note.clone(); self.st.log(Level::Warn, format!("{rel}: {note}")); } }
        }
        // Die Projektdatei gehört in die Projekt-Einstellungen; das JSON zeigt nur „Als Text öffnen“ (`open_doc_text`)
        if !as_text && is_project_file(rel) { self.open_project_editor(); return; }
        // Diskettenimages (D64, D71, D81, ATR) öffnen das Disk-Studio statt „Binärdatei, wird nicht angezeigt“
        if vc_ide::diskimage::is_image_name(rel) && crate::studios::enabled("disk") { if let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) { self.open_image(root.join(rel)); } return; }
        // Binärdateien (keine Textdatei): schreibgeschützter Hexdump statt Editor
        if !self.dock.find_tab(&Tab::Doc(rel.to_string())).is_some() {
            if let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) {
                if let Ok(b) = std::fs::read(root.join(rel)) { if vc_ide::importfiles::looks_binary(&b) { self.st.hex_view = Some((rel.to_string(), b)); return; } }
            }
        }
        let tab = Tab::Doc(rel.to_string());
        let is_open = self.dock.find_tab(&tab).is_some();
        let cur_dirty = self.st.preview.as_ref().map_or(false, |p| self.st.docs.iter().any(|d| &d.rel == p && d.dirty()));
        let plan = layout::plan_open(self.st.preview.as_deref(), cur_dirty, is_open, rel, how);
        if let Some(old) = &plan.close { self.close_doc(old); }
        self.st.preview = plan.preview.clone();
        if !is_open {
            let Some(pr) = &self.st.project else { return };
            match Doc::open(&pr.root, rel) { Ok(d) => { if let Some(l) = self.st.lsp.as_ref().filter(|_| d.lang == crate::highlight::Lang::Asm) { l.send(lsp::Cmd::Open { uri: lsp::uri(&pr.root, rel), text: d.text.clone() }); } self.st.docs.push(d) } Err(e) => { self.st.log(Level::Err, e); self.st.preview = None; return; } }
            if plan.second { layout::open_in_second(&mut self.dock, tab.clone()); } else { layout::show_unique(&mut self.dock, tab.clone()); }
            if let Some(loc) = self.dock.find_tab(&Tab::Welcome) { self.dock.remove_tab(loc); }
        } else {
            if plan.second { layout::open_in_second(&mut self.dock, tab.clone()); }
            self.reveal(tab);
        }
    }

    /// Vorschau-Reiter, die bearbeitet wurden oder nicht mehr offen sind, werden zu festen Reitern
    fn settle_preview(&mut self) {
        let Some(p) = self.st.preview.clone() else { return };
        if !self.st.docs.iter().any(|d| d.rel == p) || self.st.docs.iter().any(|d| d.rel == p && d.dirty()) { self.st.preview = None; }
    }

    /// Media öffnen in einem eigenen Fenster je Art (Tabs für mehrere Dateien gleicher Art)
    /// Titel des Studios, das die Datei bearbeiten würde, wenn es in diesem Release fehlt
    fn missing_studio(&self, rel: &str) -> Option<&'static str> {
        vc_media::naming::kind_from_name(rel).and_then(|k| crate::studios::def_for_kind(&k)).filter(|d| !crate::studios::enabled(d.id)).map(|d| d.title)
    }

    fn open_media(&mut self, rel: &str) {
        if self.missing_studio(rel).is_some() { self.open_doc_text(rel); return; }
        let Some(pr) = &self.st.project else { return };
        let Some(data) = self.st.data.clone() else { return };
        let doc = match MediaDoc::load(&self.st.ctx, &pr.root, rel, &data) { Ok(d) => d, Err(e) => { self.st.log(Level::Err, e); return } };
        let group = Group::of_kind(&doc.kind);
        if !windows::is_media_group(group) || !crate::studios::group_enabled(group) { self.st.log(Level::Warn, format!("{rel}: unbekannter Dateityp '{}'", doc.kind)); return; }
        match self.wins.iter_mut().find(|w| w.group == group) {
            Some(w) => { w.open(doc); if w.embedded { self.st.show_tab = Some(Tab::Studio(group as u8)); } }
            None => {
                let mut w = MediaWindow::new(group); w.open(doc); self.wins.push(w);
                if windows::prefers_embed(group) { self.embed_studio(group); }
            }
        }
    }

    /// Studio als Reiter ins Hauptfenster holen
    fn embed_studio(&mut self, g: Group) {
        let Some(w) = self.wins.iter_mut().find(|w| w.group == g) else { return };
        w.embedded = true; w.want = None;
        windows::set_prefers_embed(g, true);
        self.show_in_editor(Tab::Studio(g as u8));
    }

    /// Eingebettetes Studio wieder als eigenes Betriebssystem-Fenster zeigen
    fn detach_studio(&mut self, g: Group) {
        if let Some(l) = self.dock.find_tab(&Tab::Studio(g as u8)) { self.dock.remove_tab(l); }
        let Some(w) = self.wins.iter_mut().find(|w| w.group == g) else { return };
        w.embedded = false; w.want = None;
        windows::set_prefers_embed(g, false);
    }

    /// Wünsche der Studios (Knöpfe, Menüs), gezogene Reiter und verwaiste Studio-Reiter abarbeiten
    fn apply_studio_moves(&mut self) {
        let wants: Vec<(Group, windows::Move)> = self.wins.iter().filter_map(|w| w.want.map(|m| (w.group, m))).collect();
        for (g, m) in wants { match m { windows::Move::Embed => self.embed_studio(g), windows::Move::Detach => self.detach_studio(g) } }
        let orphans: Vec<u8> = self.dock.iter_all_tabs().filter_map(|(_, t)| match t { Tab::Studio(g) if !self.wins.iter().any(|w| w.group as u8 == *g && w.embedded) => Some(*g), _ => None }).collect();
        for g in orphans { if let Some(l) = self.dock.find_tab(&Tab::Studio(g)) { self.dock.remove_tab(l); } }
    }

    /// Diskettenimage (aus dem Projekt oder von irgendwo) im Dialog öffnen
    fn open_image(&mut self, path: std::path::PathBuf) {
        if !crate::studios::enabled("disk") { self.st.status = crate::studios::missing_note("Disk-Studio"); return; }
        if self.st.project.is_none() { self.st.status = "Diskettenimage: erst ein Projekt öffnen".into(); return; }
        self.diskimg = Some(crate::disk_studio::DiskStudio::new(path, self.st.data.clone()).with_dir(self.st.project.as_ref().map(|p| p.root.clone())));
    }

    /// Dateiauswahl für ein Image außerhalb des Projekts
    fn pick_image(&mut self) {
        let mut dlg = rfd::FileDialog::new().set_title("Diskettenimage öffnen").add_filter("Diskettenimage", &["d64", "d71", "d81", "atr", "D64", "D71", "D81", "ATR"]);
        if let Some(pr) = &self.st.project { dlg = dlg.set_directory(&pr.root); }
        let Some(f) = dlg.pick_file() else { return };
        self.open_image(f);
    }

    fn show_diskimg(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.diskimg.take() else { return };
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        match d.show(ctx, &root) {
            crate::disk_studio::Out::None => self.diskimg = Some(d),
            crate::disk_studio::Out::Close => {}
            crate::disk_studio::Out::Done(msgs) => {
                let n = msgs.iter().filter(|m| m.0 && !m.1.starts_with("  ")).count();
                for (ok, m) in msgs { self.st.log(if ok { Level::Ok } else { Level::Err }, m); }
                self.st.log(Level::Ok, format!("{n} Datei(en) aus dem Image übernommen"));
                if let Some(pr) = &mut self.st.project { pr.rescan(); }
            }
        }
    }

    /// Dialog „Zeichen einfügen“ für das Ziel der aktiven Assembler-Datei öffnen
    fn open_charpick(&mut self) {
        let rel = self.charpick_doc.clone().or_else(|| self.active_doc());
        let Some(doc) = rel.as_ref().and_then(|r| self.st.docs.iter().find(|d| &d.rel == r)) else { self.st.status = "Zeichen einfügen: erst eine Quelltext-Datei öffnen".into(); return };
        if doc.lang != crate::highlight::Lang::Asm { self.st.status = "Zeichen einfügen: nur im Assembler-Editor".into(); return; }
        let known = ["c64", "c128", "plus4", "c16", "vic20", "mega65", "atari8"];
        let sys = doc.target_hint().filter(|t| known.contains(&t.as_str()))
            .or_else(|| (self.st.target != "auto" && known.contains(&self.st.target.as_str())).then(|| self.st.target.clone()))
            .or_else(|| self.st.project.as_ref().and_then(|p| p.systems.iter().find(|s| known.contains(&s.as_str())).cloned()))
            .unwrap_or_else(|| vc_target::default_target_or_c64(self.st.data.as_deref()));
        self.charpick_doc = Some(doc.rel.clone());
        let Some(data) = self.st.data.clone() else { self.st.status = "Zeichen einfügen: Hardware-Datenbank (data/) nicht gefunden".into(); return };
        if self.charpick.as_ref().map_or(true, |c| c.system() != sys) { self.charpick = Some(crate::charpick::CharPick::new(&sys, &data)); }
    }

    /// Dialog zeichnen und Einfügungen an den Editor geben
    fn show_charpick(&mut self, ctx: &egui::Context) {
        let Some(mut cp) = self.charpick.take() else { return };
        let rel = self.charpick_doc.clone();
        let in_string = rel.as_ref().and_then(|r| self.st.docs.iter().find(|d| &d.rel == r)).map_or(false, |d| d.in_text_string());
        let mut keep = true;
        match cp.show(ctx, in_string) {
            crate::charpick::Out::Insert(t) => {
                if let Some(d) = rel.as_ref().and_then(|r| self.st.docs.iter_mut().find(|d| &d.rel == r)) { d.insert = Some(t); ctx.request_repaint(); }
                if !cp.keep_open() { keep = false; }
            }
            crate::charpick::Out::Close => keep = false,
            crate::charpick::Out::None => {}
        }
        if keep { self.charpick = Some(cp); }
    }

    /// Dialog „Feedback senden“ öffnen; vom offenen Projekt geht nur das Zielsystem in die Mail
    fn open_feedback(&mut self) {
        let sys = self.st.project.as_ref().and_then(|p| if self.st.target == "auto" { p.systems.first().cloned() } else { Some(self.st.target.clone()) });
        self.feedback = Some(crate::feedback::Dlg::new(sys.as_deref()));
    }

    /// Befehl aus der Menüleiste am Bildschirmrand (macOS); dieselben Aktionen wie die Menüs im Fenster
    fn menu_cmd(&mut self, ctx: &egui::Context, c: crate::nativemenu::Cmd, do_open: &mut bool, want_convert: &mut bool) {
        use crate::nativemenu::Cmd::*;
        vc_ide::debuglog::log("befehl", &format!("{c:?}"));
        let mut view_changed = false;
        match c {
            NewProject => self.new_project_dialog(),
            OpenProject => *do_open = true,
            Settings => self.open_settings(),
            About => self.about_open = true,
            Feedback => self.open_feedback(),
            NewFile => self.file_dlg = Some(FileDlg::NewFile { dir: String::new(), name: String::new() }),
            NewMedia => self.new_media_dialog(),
            ConvertBasic => *want_convert = true,
            StdCharset => self.std_charset_dialog(),
            RomSetup => self.open_rom_wizard(),
            SetupAll => self.open_setup(crate::setup_wizard::Launch::Recall),
            SetupRoms => self.open_setup(crate::setup_wizard::Launch::Part(vc_ide::setup::Part::Roms)),
            SetupEmulators => self.open_setup(crate::setup_wizard::Launch::Part(vc_ide::setup::Part::Emulators)),
            SetupHardware => self.open_setup(crate::setup_wizard::Launch::Part(vc_ide::setup::Part::Hardware)),
            SetupDesign => self.open_setup(crate::setup_wizard::Launch::Part(vc_ide::setup::Part::DesignEditor)),
            SetupExport => self.open_setup(crate::setup_wizard::Launch::Export),
            SetupImport => self.open_setup(crate::setup_wizard::Launch::Import),
            SetupReset => self.open_setup(crate::setup_wizard::Launch::Reset),
            RomFolder => { if let Some(d) = vc_ide::roms::env(None, None).install_dir() { let _ = std::fs::create_dir_all(&d); reveal_in_file_manager(&d.join("x").to_string_lossy()); } }
            RomCheck => {
                let lines = vc_media::romref::with_store(|s| s.status_all().iter().map(|r| r.describe()).collect::<Vec<_>>());
                for l in lines { self.st.log(Level::Info, l); }
            }
            ImportRom => self.import_rom_dialog(),
            ImportSheet => self.import_sheet_dialog(),
            Migrate => self.migrate_project(),
            Save => self.save_active(),
            Build => self.build(ctx, After::Nothing),
            Run => self.build(ctx, After::Run),
            Debug => self.build(ctx, After::Debug),
            BuildActive => self.build_active(ctx, After::Nothing),
            RunActive => self.build_active(ctx, After::Run),
            DebugActive => self.build_active(ctx, After::Debug),
            ProjectSettings => self.open_project_editor(),
            InsertChar => { self.charpick_doc = None; self.open_charpick(); }
            Palette => self.palette.toggle(),   // Cmd/Strg+K
            LiveImage => self.open_live(),
            DbgEmulator => { self.switch_mode(Mode::Debug); self.reveal(Tab::Emulator); }
            StudioDetach => if let Some((_, Tab::Studio(g))) = self.dock.find_active_focused() { let g = *g; if let Some(w) = self.wins.iter_mut().find(|w| w.group as u8 == g) { w.want = Some(windows::Move::Detach); } },
            StudioEmbed => for w in self.wins.iter_mut().filter(|w| !w.embedded) { w.want = Some(windows::Move::Embed); },
            HotKeys => { let hk = !crate::bitmap::HOTKEYS.load(std::sync::atomic::Ordering::Relaxed); crate::bitmap::HOTKEYS.store(hk, std::sync::atomic::Ordering::Relaxed); save_prefs(self.st.show_addr, &self.st.run_on); }
            ShowAddr => { self.st.show_addr = !self.st.show_addr; save_prefs(self.st.show_addr, &self.st.run_on); }
            SchemeDark => { self.view.scheme = crate::theme::Scheme::Dark; view_changed = true; }
            SchemeLight => { self.view.scheme = crate::theme::Scheme::Light; view_changed = true; }
            SchemeSystem => { self.view.scheme = crate::theme::Scheme::System; view_changed = true; }
            ZoomIn => { self.view.zoom = (self.view.zoom * 1.1).clamp(crate::theme::ZOOM_RANGE.0, crate::theme::ZOOM_RANGE.1); view_changed = true; }
            ZoomOut => { self.view.zoom = (self.view.zoom / 1.1).clamp(crate::theme::ZOOM_RANGE.0, crate::theme::ZOOM_RANGE.1); view_changed = true; }
            ZoomReset => { self.view.zoom = 1.0; view_changed = true; }
            FontBigger => { self.view.font = (self.view.font + 1.0).clamp(crate::theme::FONT_RANGE.0, crate::theme::FONT_RANGE.1); view_changed = true; }
            FontSmaller => { self.view.font = (self.view.font - 1.0).clamp(crate::theme::FONT_RANGE.0, crate::theme::FONT_RANGE.1); view_changed = true; }
            ResetView => { self.view = crate::theme::View::default(); view_changed = true; }
            SaveAll => { self.save_all_docs(); self.st.status = "alle gespeichert".into(); }
            CloseFile => { if let Some(r) = self.active_doc() { self.close_doc(&r); } }
            SplitEditor => { if layout::is_split(&self.dock) { layout::unsplit(&mut self.dock); } else if let Some(r) = self.active_doc() { self.st.tab_act = Some(TabAct::Second(r)); } }
            ToggleSide => self.toggle_zone(ctx, Zone::Side),
            ToggleInspector => self.toggle_zone(ctx, Zone::Inspector),
            FindInProject => self.show_project_search(),
            ToggleFooter => { self.zen = false; let w = self.wp(); w.footer = !w.footer; }
            WorkCode => self.switch_mode(Mode::Code),
            WorkDebug => self.switch_mode(Mode::Debug),
            Maximize => self.zen = !self.zen,
            ResetLayout => self.reset_layout(),
            DbgCont => if let Some(d) = &self.st.dbg { d.send(DbgCmd::Cont); },
            DbgPause => if let Some(d) = &self.st.dbg { d.send(DbgCmd::Pause); },
            DbgStepInto => if let Some(d) = &self.st.dbg { d.send(DbgCmd::Step("into")); },
            DbgStepOver => if let Some(d) = &self.st.dbg { d.send(DbgCmd::Step("over")); },
            DbgStepOut => if let Some(d) = &self.st.dbg { d.send(DbgCmd::Step("out")); },
            DbgStop => self.stop_dbg_request = true,
            DbgMemory => { self.switch_mode(Mode::Debug); self.reveal(Tab::Memory); }
            Help => self.reveal(Tab::Help),
            Shortcuts => { self.st.help_open("shortcuts"); self.reveal(Tab::Help); }
            Undo | Redo => {
                // an das fokussierte Feld des Editors bzw. Studios: Tastendruck im nächsten Bild einspeisen (egui führt Rückgängig dort selbst aus)
                if let Some(r) = self.active_doc() { ctx.memory_mut(|m| m.request_focus(egui::Id::new(("editor", &r)))); }
                self.st.pending_undo = Some(c == Redo); ctx.request_repaint();
            }
            Find | Replace => match self.active_doc().and_then(|r| self.st.docs.iter_mut().find(|d| d.rel == r)) {
                Some(d) => { d.goto.open = false; d.find.open = true; d.find.replace_mode = c == Replace; d.find.focus = true; }
                None => self.st.status = "Suchen: erst eine Quelltext-Datei öffnen".into(),
            },
            GotoLine => match self.active_doc().and_then(|r| self.st.docs.iter_mut().find(|d| d.rel == r)) {
                Some(d) => { d.find.open = false; d.goto.open = true; d.goto.focus = true; d.goto.input.clear(); }
                None => self.st.status = "Gehe zu Zeile: erst eine Quelltext-Datei öffnen".into(),
            },
            Export => {
                if matches!(self.dock.find_active_focused(), Some((_, Tab::Studio(_)))) { crate::exportdlg::WANT.store(true, std::sync::atomic::Ordering::Relaxed); ctx.request_repaint(); }
                else { self.st.status = "Exportieren: erst ein Studio (Sprites, Zeichensatz, Bildschirm …) öffnen".into(); }
            }
            Recent(i) => { let p = self.st.welcome.recent.iter().filter(|r| !r.missing).nth(i as usize).map(|r| r.path.clone()); if let Some(p) = p { self.open_project_path(p); } }
        }
        if view_changed { self.view.apply(ctx); self.view.save(); }
    }

    /// Befehl „Live-Bild öffnen“: Reiter anlegen bzw. nach vorn holen und den Bildstrom starten (ausdrückliche Handlung des Nutzers)
    fn open_live(&mut self) {
        self.reveal(Tab::Live);
        self.st.live.request_start();
    }

    fn active_doc(&mut self) -> Option<String> {
        match self.dock.find_active_focused() { Some((_, Tab::Doc(r))) => Some(r.clone()), _ => None }
    }

    /// Reiter „Projekt-Einstellungen“ öffnen bzw. nach vorn holen
    /// Reiter „Einstellungen“ öffnen bzw. nach vorn holen
    fn open_settings(&mut self) { self.show_in_editor(Tab::Settings); }

    fn open_project_editor(&mut self) {
        if self.st.project.is_none() { return; }
        self.show_in_editor(Tab::ProjectEdit);
    }

    /// Startprogramm im Projektbaum gewählt: sofort in der Projektdatei speichern
    fn set_main(&mut self, rel: String) {
        let Some(pr) = &self.st.project else { return };
        let (Some(file), Some(mut pf)) = (pr.file.clone(), pr.pf.clone()) else { self.st.log(Level::Warn, "Projekt hat keine .vcprj – zuerst unter Projekt-Einstellungen speichern"); return };
        if !pf.source_files().iter().any(|s| s.0 == rel) {   // Datei ist noch keine Quelle: in die passende Gruppe aufnehmen
            let kind = if rel.ends_with(".sbas") || rel.ends_with(".bas") || rel.ends_with(".lst") || rel.ends_with(".tur") { "basic" } else { "asm" };
            match pf.sources.iter_mut().find(|s| s.kind == kind) { Some(g) => g.files.push(rel.clone()), None => pf.sources.push(vc_ide::projfile::Source { targets: pf.targets.first().map(|t| vec![t.id.clone()]).unwrap_or_default(), kind: kind.into(), files: vec![rel.clone()] , ..Default::default() }) }
        }
        pf.main = Some(rel.clone());
        match pf.save(&pr.root.join(&file)) {
            Ok(()) => { self.st.log(Level::Ok, format!("Startprogramm: {rel}")); if let Some(pr) = &mut self.st.project { pr.reload(); } self.st.pe = None; }
            Err(e) => self.st.log(Level::Err, e),
        }
    }

    /// Speichern im Projekteditor (legt die Projektdatei bei Bedarf an)
    fn save_project_file(&mut self) -> bool {
        let (Some(pr), Some(ed)) = (&mut self.st.project, &mut self.st.pe) else { return false };
        let file = pr.file.clone().unwrap_or_else(|| format!("{}.vcprj", pr.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "projekt".into())));
        match ed.draft.save(&pr.root.join(&file)) {
            Ok(()) => { ed.saved = ed.draft.clone(); pr.file = Some(file); pr.reload(); self.st.status = "Projektdatei gespeichert".into(); self.st.log(Level::Ok, "Projektdatei gespeichert"); true }
            Err(e) => { self.st.log(Level::Err, e); false }
        }
    }

    // ---- Dateiaktionen des Projektbaums -----------------------------------------------------------------

    fn close_doc(&mut self, rel: &str) {
        if let Some(loc) = self.dock.find_tab(&Tab::Doc(rel.to_string())) { self.dock.remove_tab(loc); }
        if let (Some(l), Some(pr)) = (&self.st.lsp, &self.st.project) { l.send(lsp::Cmd::Close { uri: lsp::uri(&pr.root, rel) }); }
        self.st.docs.retain(|d| d.rel != rel);
    }

    /// Pfad in der Projektdatei nachführen: `new = None` entfernt die Datei aus Quellen, Startprogramm und Diskette
    fn pf_paths(&mut self, old: &str, new: Option<&str>) {
        let Some(pr) = &self.st.project else { return };
        let (Some(file), Some(mut pf)) = (pr.file.clone(), pr.pf.clone()) else { return };
        let before = pf.clone();
        for s in &mut pf.sources { match new { Some(n) => for f in &mut s.files { if f == old { *f = n.to_string(); } }, None => s.files.retain(|f| f != old) } }
        pf.sources.retain(|s| !s.files.is_empty() || new.is_some());
        pf.rename_asset(old, new);
        if pf.main.as_deref() == Some(old) { pf.main = new.map(String::from); }
        if let Some(d) = &mut pf.disk { match new { Some(n) => for f in &mut d.files { if f == old { *f = n.to_string(); } }, None => d.files.retain(|f| f != old) } }
        if pf == before { return; }
        let root = pr.root.clone();
        match pf.save(&root.join(&file)) { Ok(()) => { if let Some(pr) = &mut self.st.project { pr.reload(); } self.st.pe = None; } Err(e) => self.st.log(Level::Err, e) }
    }

    /// Projektdatei ändern und speichern (Rolle, Zielordner aus dem Projektbaum); `f` liefert, ob etwas geändert wurde
    fn edit_pf(&mut self, f: impl FnOnce(&mut vc_ide::projfile::ProjectFile) -> bool) -> bool {
        let Some((root, file, pf)) = self.st.project.as_ref().map(|p| (p.root.clone(), p.file.clone(), p.pf.clone())) else { return false };
        let (Some(file), Some(mut pf)) = (file, pf) else { self.st.log(Level::Warn, "Das Projekt hat keine .vcprj"); return false };
        if !f(&mut pf) { return false; }
        match pf.save(&root.join(&file)) { Ok(()) => { if let Some(pr) = &mut self.st.project { pr.reload(); } self.st.pe = None; true } Err(e) => { self.st.log(Level::Err, e); false } }
    }

    fn run_tree_action(&mut self, ctx: &egui::Context, a: TreeAct) {
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        match a {
            TreeAct::Rename(rel) => { let name = std::path::Path::new(&rel).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(); self.file_dlg = Some(FileDlg::Rename { rel, name, ask: RenameAsk { rewrite: true, ..Default::default() } }); }
            TreeAct::Delete(rel) => self.file_dlg = Some(FileDlg::Delete { rel }),
            TreeAct::OpenImage(rel) => self.open_image(root.join(&rel)),
            TreeAct::OpenText(rel) => self.open_doc_text(&rel),
            TreeAct::Include(rel) => self.insert_include_menu(&rel),
            TreeAct::SetRole(dir, role) => { if self.edit_pf(|pf| pf.set_role(&dir, role.as_deref())) { self.st.log(Level::Ok, match &role { Some(r) => format!("Ordner {dir}/: Rolle {}", vc_ide::projfile::role_label(r).unwrap_or(r)), None => format!("Ordner {dir}/: Rolle entfernt") }); } }
            TreeAct::SetFolderTarget(dir, t) => { if self.edit_pf(|pf| pf.set_folder_target(&dir, t.as_deref())) { self.st.log(Level::Ok, match &t { Some(t) => format!("Ordner {dir}/: nur für Target {t}"), None => format!("Ordner {dir}/: für alle Targets") }); } }
            TreeAct::SetTarget(dir, cat) => { if self.edit_pf(|pf| pf.set_layout_dir(&cat, &dir)) { let l = vc_ide::importfiles::Category::from_id(&cat).map_or(cat.as_str(), |c| c.label()); self.st.log(Level::Ok, format!("Abgelegte Dateien der Kategorie „{l}“ kommen künftig nach {dir}/")); } }
            TreeAct::NewFolder(dir) => self.file_dlg = Some(FileDlg::NewFolder { dir, name: String::new() }),
            TreeAct::NewFile(dir) => self.file_dlg = Some(FileDlg::NewFile { dir, name: String::new() }),
            TreeAct::NewMedia(g) => self.new_media_dialog_for(Some(g)),
            TreeAct::NewTest => self.file_dlg = Some(FileDlg::NewFile { dir: String::new(), name: "tests.vctest".into() }),
            TreeAct::CopyPath(rel) => ctx.copy_text(root.join(&rel).to_string_lossy().into_owned()),
            TreeAct::Reveal(rel) => {
                reveal_in_file_manager(&root.join(&rel).to_string_lossy());
            }
            TreeAct::Duplicate(rel) => {
                let p = std::path::Path::new(&rel);
                let (stem, ext) = (p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(), p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default());
                let dir = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
                let mk = |n: usize| { let nm = if n == 0 { format!("{stem}_copy{ext}") } else { format!("{stem}_copy{}{ext}", n + 1) }; if dir.is_empty() { nm } else { format!("{dir}/{nm}") } };
                let new = (0..50).map(mk).find(|c| !root.join(c).exists());
                match new { Some(n) => match std::fs::copy(root.join(&rel), root.join(&n)) { Ok(_) => { self.st.log(Level::Ok, format!("Dupliziert: {n}")); if let Some(pr) = &mut self.st.project { pr.rescan(); } } Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")) }, None => self.st.log(Level::Err, "Kein freier Name für die Kopie") }
            }
            TreeAct::ConvertBasic(rel) => {
                let mut req = vc_ide::b2a::Req::new(rel.clone());
                req.out = vc_ide::b2a::default_out(&root, &rel);
                if let Some(t) = self.st.project.as_ref().and_then(|p| p.pf.as_ref()).and_then(|pf| pf.target_of(&rel)) { if vc_ide::b2a::targets().contains(&t.as_str()) { req.target = t; } }
                self.file_dlg = Some(FileDlg::ConvertBasic { req });
            }
            TreeAct::BuildFile(rel) => { self.save_all_docs(); self.start_build(ctx, BuildKind::File(rel), After::Nothing); }
        }
    }

    /// Dateiname ohne Endung bekommt die Endung der Hauptsprache des Projekts (.bas, wenn das Startprogramm BASIC ist, sonst .asm)
    fn default_ext(&self, name: &str) -> String {
        if name.contains('.') || name.is_empty() { return name.to_string(); }
        let basic = self.st.project.as_ref().and_then(|p| p.pf.as_ref().and_then(|pf| pf.main_file())).map_or(false, |m| { let m = m.to_ascii_lowercase(); m.ends_with(".bas") || m.ends_with(".lst") || m.ends_with(".tur") });
        format!("{name}{}", if basic { ".bas" } else { ".asm" })
    }

    /// Einzubindende Zeile für die Projektdatei `rel` im Quelltext `doc` (Regeln und Ablehnungen: siehe `dropinc`)
    fn include_plan(&self, doc: &str, rel: &str) -> Result<crate::dropinc::Plan, crate::dropinc::Reject> {
        let text = self.st.docs.iter().find(|d| d.rel == doc).map(|d| d.text.as_str()).unwrap_or("");
        crate::dropinc::plan(doc, text, rel, self.st.project.as_ref().map(|p| p.root.as_path()))
    }
    fn include_line(&self, doc: &str, rel: &str) -> Option<String> { self.include_plan(doc, rel).ok().map(|p| p.line) }

    /// Zeilen an der Cursorposition des Quelltexts `doc` einfügen
    fn insert_lines(&mut self, doc: &str, lines: &[String]) {
        if lines.is_empty() { return; }
        if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == doc) {
            let text = lines.join("\n");
            d.insert = Some(if d.cursor.1 <= 1 { format!("{text}\n") } else { format!("\n{text}") });
            self.st.log(Level::Ok, format!("In {doc} eingefügt: {}", lines.join(" · ")));
        }
    }

    /// Include für `src` in `doc` einfügen: vor Zeile `at` (0-basiert), sonst unter dem Cursor. Ablehnungen und „schon eingebunden“ melden, nichts erfinden.
    fn insert_include(&mut self, doc: &str, src: &str, at: Option<usize>) {
        match self.include_plan(doc, src) {
            Err(e) => {
                if let crate::dropinc::Reject::Already(l) = &e { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == doc) { d.reveal = Some(*l); d.flash = Some((*l, 0.0)); } }
                self.st.log(Level::Warn, e.to_string());
                self.st.drop_note = Some(DropNote { text: e.to_string(), ok: false, doc: doc.into(), at: 0, line: String::new(), alt: None, t0: None });
            }
            Ok(plan) => {
                let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == doc) else { return };
                let at = at.unwrap_or(if d.cursor.1 <= 1 { d.cursor.0.saturating_sub(1) } else { d.cursor.0 });
                d.line_edit = Some(crate::editor::LineEdit::Insert { at, text: plan.line.clone() });
                self.st.log(Level::Ok, format!("In {doc} eingefügt: {}", plan.line));
                self.st.drop_note = Some(DropNote { text: format!("Eingefügt in {doc}"), ok: true, doc: doc.into(), at, line: plan.line, alt: plan.alt, t0: None });
            }
        }
    }

    /// „In Code einfügen“ aus einem Asset-Editor: in den zuletzt benutzten Quelltext
    fn insert_asset_include(&mut self, rel: &str) {
        let target = self.st.last_asm.clone().filter(|r| self.st.docs.iter().any(|d| &d.rel == r));
        let Some(doc) = target else { self.st.log(Level::Warn, "Kein Quelltext offen – zuerst im Editor die Einfügestelle wählen"); return };
        self.insert_include(&doc, rel, None);
        self.st.open_request = Some(doc);
    }

    /// Zieldokument für „Als Include einfügen“ im Projektbaum: der aktive Editorreiter, sonst der zuletzt benutzte Quelltext
    fn include_target(&mut self) -> Option<String> {
        let cand = self.active_doc().or_else(|| self.st.last_asm.clone());
        cand.filter(|r| self.st.docs.iter().any(|d| &d.rel == r))
    }

    /// Kontextmenü „Als Include einfügen“
    fn insert_include_menu(&mut self, rel: &str) {
        let Some(doc) = self.include_target() else { self.st.log(Level::Warn, "Kein Quelltext offen – zuerst im Editor die Einfügestelle wählen"); return };
        self.insert_include(&doc, rel, None);
        self.st.open_request = Some(doc);
    }

    /// Hinweisleiste nach dem Einbinden (Umschalten auf die Alternative, Rückgängig) bzw. nach einer Ablehnung
    fn show_drop_note(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let Some(n) = &mut self.st.drop_note else { return };
        let t0 = *n.t0.get_or_insert(now);
        if now - t0 > if n.ok { 12.0 } else { 6.0 } { self.st.drop_note = None; return; }
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        let mut act: Option<(crate::editor::LineEdit, Option<String>)> = None; let mut close = false;
        let (ok, text, line, alt, doc, at) = (n.ok, n.text.clone(), n.line.clone(), n.alt.clone(), n.doc.clone(), n.at);
        egui::Area::new(egui::Id::new("drop_note")).order(egui::Order::Foreground).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0]).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ok {
                        ui.label(RichText::new("✓").color(crate::theme::ok()).strong());
                        ui.label(RichText::new(&line).monospace());
                        if let Some((lab, new)) = &alt { if ui.small_button(format!("als {lab}")).on_hover_text(new.as_str()).clicked() { act = Some((crate::editor::LineEdit::Replace { at, old: line.clone(), new: new.clone() }, Some(new.clone()))); } }
                        if ui.small_button("Rückgängig").on_hover_text("Nimmt die Zeile wieder heraus (Cmd/Strg+Z im Editor tut dasselbe)").clicked() { act = Some((crate::editor::LineEdit::Remove { at, old: line.clone() }, None)); close = true; }
                    } else {
                        ui.label(RichText::new("✗").color(crate::theme::bad()).strong());
                        ui.label(&text);
                    }
                    if ui.small_button("×").clicked() { close = true; }
                });
            });
        });
        if let Some((mut le, new)) = act {
            // Zeile suchen, falls sie sich verschoben hat
            if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == doc) {
                let old = match &le { crate::editor::LineEdit::Replace { old, .. } | crate::editor::LineEdit::Remove { old, .. } => old.clone(), _ => String::new() };
                let lines: Vec<&str> = d.text.split('\n').collect();
                let cur = match &le { crate::editor::LineEdit::Replace { at, .. } | crate::editor::LineEdit::Remove { at, .. } => *at, _ => 0 };
                let found = if lines.get(cur) == Some(&old.as_str()) { Some(cur) } else { lines.iter().position(|l| *l == old) };
                match found {
                    Some(f) => { match &mut le { crate::editor::LineEdit::Replace { at, .. } | crate::editor::LineEdit::Remove { at, .. } => *at = f, _ => {} } d.line_edit = Some(le); ctx.request_repaint(); }
                    None => { self.st.log(Level::Warn, "Die eingefügte Zeile wurde inzwischen geändert – nichts zurückgenommen"); close = true; }
                }
            }
            if let (Some(n), Some(new)) = (&mut self.st.drop_note, new) { n.line = new; n.alt = None; n.t0 = Some(now); }
        }
        if close { self.st.drop_note = None; }
    }

    /// Beim Umbenennen eines Assets: `build/export/<alt>.asm` in allen Quelltexten des Projekts auf den neuen Namen umstellen
    fn rename_export_refs(&mut self, old_rel: &str, new_rel: &str) {
        if vc_media::naming::stem(old_rel) == vc_media::naming::stem(new_rel) { return; }
        for ext in ["asm", "data.bas"] {
            let (old, new) = (format!("build/export/{}.{ext}", vc_media::naming::stem(old_rel)), format!("build/export/{}.{ext}", vc_media::naming::stem(new_rel)));
            self.rename_export_ref(&old, &new);
        }
    }

    fn rename_export_ref(&mut self, old: &str, new: &str) {
        let (old, new) = (old.to_string(), new.to_string());
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        let srcs: Vec<String> = self.st.project.as_ref().map(|p| p.entries.iter().filter(|e| e.group == Group::Asm || e.group == Group::Basic).map(|e| e.rel.clone()).collect()).unwrap_or_default();
        let mut changed = vec![];
        for rel in srcs {
            if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == rel) {
                if d.text.contains(&old) { d.text = d.text.replace(&old, &new); changed.push(rel.clone()); }
                continue;
            }
            // der Pfad im Quelltext ist relativ zur Datei (`../build/export/x.asm`): nur das Ende ersetzen
            if let Ok(t) = std::fs::read_to_string(root.join(&rel)) { if t.contains(&old) && std::fs::write(root.join(&rel), t.replace(&old, &new)).is_ok() { changed.push(rel); } }
        }
        let _ = std::fs::remove_file(root.join(&old));
        if !changed.is_empty() { self.st.log(Level::Ok, format!("Include-Pfade angepasst ({} → {}): {}", old, new, changed.join(", "))); }
    }

    /// Dateien, die aus dem Finder/Explorer auf das Fenster gezogen werden: über dem Projektbaum ins Projekt kopieren,
    /// über dem Editor (Assembler) zusätzlich als `.include` bzw. `.incbin` an der Cursorposition einfügen
    fn handle_drops(&mut self, ctx: &egui::Context) {
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
        let pos = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.latest_pos()));
        let over_tree = pos.zip(self.st.tree_rect).map_or(false, |(p, r)| r.contains(p));
        let roms_page = crate::settings_ui::roms::wants_drops(ctx);
        // Beim Ziehen über einen zugeklappten Ordner: nach 600 ms aufklappen, damit tiefere Ziele erreichbar sind
        if hovering && over_tree {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
            match pos.and_then(|p| self.st.tree_dir_at(p)) {
                Some(d) => match &self.st.drop_hover {
                    Some((cur, since)) if *cur == d => { if since.elapsed() > std::time::Duration::from_millis(600) { self.st.tree_open_req = Some(d.clone()); self.st.drop_hover = Some((d, std::time::Instant::now())); } }
                    _ => self.st.drop_hover = Some((d, std::time::Instant::now())),
                },
                None => self.st.drop_hover = None,
            }
        } else { self.st.drop_hover = None; }
        if hovering && self.st.project.is_some() && !roms_page {
            let msg = if over_tree { "Loslassen: Dateien in diesen Ordner übernehmen (mit Rückfrage zu Art und Kategorie)" } else { "Loslassen: Dateien ins Projekt übernehmen und im Quelltext einbinden (.include / .incbin)" };
            egui::Area::new(egui::Id::new("drop_hint")).order(egui::Order::Foreground).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0]).interactable(false).show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| { ui.label(RichText::new(msg).strong()); });
            });
        }
        // Ziehen aus dem Projektbaum in einen Quelltext: Include-Zeile an der Einfügemarke (Zeile unter dem Mauszeiger) einfügen
        if let Some(rel) = egui::DragAndDrop::payload::<String>(ctx) {
            let hover_doc = self.st.editor_hover.clone().filter(|r| self.st.docs.iter().any(|d| &d.rel == r));
            if let Some(p) = pos {
                // Vorschau am Zeiger: Name der Datei, darunter die entstehende Zeile bzw. der Grund der Ablehnung
                let preview = hover_doc.as_ref().map(|d| self.include_plan(d, &rel));
                egui::Area::new(egui::Id::new("drag_ghost")).order(egui::Order::Tooltip).fixed_pos(p + egui::vec2(12.0, 12.0)).interactable(false).show(ctx, |ui| { egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_max_width(340.0);
                    ui.label(RichText::new(vc_media::naming::stem(&rel)).strong());
                    match &preview {
                        Some(Ok(pl)) => { ui.add(egui::Label::new(RichText::new(&pl.line).monospace()).extend()); }
                        Some(Err(e)) => { ui.label(RichText::new(format!("✗ {e}")).color(crate::theme::bad()).small()); }
                        None => {}
                    }
                }); });
            }
            if ctx.input(|i| i.pointer.any_released()) {
                if let Some(doc) = hover_doc {
                    let at = self.st.docs.iter().find(|d| d.rel == doc).and_then(|d| d.drop_at);
                    self.insert_include(&doc, &rel, at);
                }
            }
        }
        self.st.editor_hover = None;
        let dropped: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        if dropped.is_empty() { return; }
        // Seite „ROMs“ der Einstellungen sichtbar: abgelegte ROM-Dateien und ZIP-Archive gehören dorthin (Vorschau, dann Kopieren), nicht ins Projekt
        if roms_page { crate::settings_ui::roms::push_drops(dropped); return; }
        if self.st.project.is_none() { self.st.log(Level::Warn, "Kein Projekt geöffnet – Dateien können nicht übernommen werden"); return; }
        // Zielordner: Ordner unter dem Mauszeiger im Projektbaum (sonst dessen Wurzel); außerhalb des Baums schlägt der Dialog je Kategorie einen vor
        let fixed_dir = if over_tree { Some(pos.and_then(|p| self.st.tree_dir_at(p)).unwrap_or_default()) } else { None };
        let doc_rel = if over_tree { None } else { self.active_doc().filter(|r| self.st.docs.iter().any(|d| &d.rel == r && d.lang != crate::highlight::Lang::Plain)) };
        match &mut self.import_dlg {
            Some(d) => d.add(dropped),
            None => { let (layout, rules) = self.st.project.as_ref().and_then(|p| p.pf.as_ref()).map(|pf| (pf.layout.clone(), pf.rules.clone())).unwrap_or_default(); self.import_dlg = crate::importdlg::ImportDlg::new(dropped, fixed_dir, doc_rel, layout, rules); }
        }
    }

    /// Dialog „Dateien neu ordnen“ öffnen (speichert vorher offene Dokumente)
    fn open_reorg(&mut self) {
        let Some((root, pf)) = self.st.project.as_ref().map(|p| (p.root.clone(), p.pf.clone())) else { return };
        let Some(pf) = pf else { self.st.log(Level::Warn, "Das Projekt hat keine .vcprj – Neu ordnen braucht Projektdatei und Projektstruktur"); return };
        self.reorg_dlg = Some(crate::reorgdlg::ReorgDlg::new(&root, &pf));
    }

    fn show_reorg_dlg(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.reorg_dlg.take() else { return };
        let Some((root, file, pf)) = self.st.project.as_ref().map(|p| (p.root.clone(), p.file.clone(), p.pf.clone())) else { return };
        let (Some(file), Some(pf)) = (file, pf) else { return };
        // Meldungen des Hintergrundthreads
        let mut finished: Option<Result<vc_ide::reorganize::Report, String>> = None;
        if let Some(rx) = &self.reorg_job {
            while let Ok(m) = rx.try_recv() {
                match m { crate::reorgdlg::Msg::Progress(t) => d.stage = crate::reorgdlg::Stage::Running(t), crate::reorgdlg::Msg::Done(r) => finished = Some(r) }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
        if let Some(r) = finished {
            self.reorg_job = None;
            match r {
                Ok(rep) => {
                    self.st.log(Level::Ok, format!("Neu geordnet: {} Datei(en) verschoben, {} Quelltext(e) angepasst, Sicherung {}", rep.moved.len(), rep.rewritten.len(), rep.backup.display()));
                    if let vc_ide::reorganize::Check::Different(v) = &rep.check { self.st.log(Level::Warn, format!("Prüfbau: Ergebnis weicht ab ({})", v.join(", "))); }
                    if let Some(pr) = &mut self.st.project { pr.reload(); pr.rescan(); }
                    self.st.pe = None;
                    for r in std::mem::take(&mut self.reorg_reopen) { if !rep.moved.iter().any(|m| m.0 == r) { self.open_doc(&r); } }
                    d.stage = crate::reorgdlg::Stage::Done(rep);
                }
                Err(e) => { self.st.log(Level::Err, format!("Neu ordnen fehlgeschlagen: {e} (nichts verändert)")); d.stage = crate::reorgdlg::Stage::Plan; }
            }
        }
        let act = d.show(ctx, &root, &pf);
        match (&d.stage, act) {
            (crate::reorgdlg::Stage::Plan, dialog::Act::Primary) => {
                let moves = d.selected();
                self.save_all_docs();
                // betroffene Dokumente schließen (Pfade ändern sich); sie lassen sich danach neu öffnen
                let mut affected: Vec<String> = moves.iter().map(|m| m.from.clone()).collect();
                if d.rewrite { affected.extend(vc_ide::reorganize::rewrites(&root, &moves).into_iter().map(|r| r.0)); }
                // offene Quelltexte, die nur angepasst (nicht verschoben) werden, danach wieder öffnen
                self.reorg_reopen = affected.iter().filter(|r| !moves.iter().any(|m| &m.from == *r) && self.st.docs.iter().any(|x| &x.rel == *r)).cloned().collect();
                for r in affected { self.close_doc(&r); }
                let data = if d.check { self.st.data.clone() } else { None };
                let (tx, rx) = std::sync::mpsc::channel();
                let (ctx2, root2, file2, mut pf2, rewrite) = (ctx.clone(), root.clone(), root.join(&file), pf.clone(), d.rewrite);
                std::thread::spawn(move || {
                    let (txp, cp) = (tx.clone(), ctx2.clone());
                    let r = vc_ide::reorganize::apply(&root2, &file2, &mut pf2, &moves, data.as_deref(), rewrite, &|s| { let _ = txp.send(crate::reorgdlg::Msg::Progress(s.to_string())); cp.request_repaint(); });
                    let _ = tx.send(crate::reorgdlg::Msg::Done(r));
                    ctx2.request_repaint();
                });
                self.reorg_job = Some(rx);
                d.stage = crate::reorgdlg::Stage::Running("Starte …".into());
                self.reorg_dlg = Some(d);
            }
            (crate::reorgdlg::Stage::Plan, dialog::Act::Secondary) => {}
            (crate::reorgdlg::Stage::Done(rep), dialog::Act::Primary) => {
                for (_, to) in &rep.moved { self.close_doc(to); }
                match vc_ide::reorganize::undo(&root, &rep.backup) {
                    Ok(n) => { self.st.log(Level::Ok, format!("Rückgängig: {n} Datei(en) zurückverschoben")); if let Some(pr) = &mut self.st.project { pr.reload(); pr.rescan(); } self.st.pe = None; }
                    Err(e) => self.st.log(Level::Err, format!("Rückgängig fehlgeschlagen: {e}")),
                }
            }
            (crate::reorgdlg::Stage::Done(_), dialog::Act::Secondary) => {}
            _ => self.reorg_dlg = Some(d),
        }
    }

    /// Rückfrage zu abgelegten Dateien; „Übernehmen“ kopiert, trägt in die Projektdatei ein und fügt auf Wunsch Include-Zeilen ein
    fn show_import_dlg(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.import_dlg.take() else { return };
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        match d.show(ctx, &root) {
            dialog::Act::Primary => self.run_import(&d),
            dialog::Act::Secondary => {}
            dialog::Act::None => self.import_dlg = Some(d),
        }
    }

    fn run_import(&mut self, d: &crate::importdlg::ImportDlg) {
        use vc_ide::importfiles::{self, Outcome};
        let Some((root, file, mut pf)) = self.st.project.as_ref().map(|p| (p.root.clone(), p.file.clone(), p.pf.clone())) else { return };
        let before = pf.clone();
        let mut lines: Vec<String> = vec![]; let mut to_open: Option<String> = None;
        for plan in d.plans() {
            match importfiles::import(&root, pf.as_mut(), &plan) {
                Ok(o) => {
                    match &o {
                        Outcome::Copied(r) => self.st.log(Level::Ok, format!("Ins Projekt kopiert: {r} ({})", plan.category.label())),
                        Outcome::Replaced(r) => self.st.log(Level::Ok, format!("Ersetzt: {r}")),
                        Outcome::Existing(r) => self.st.log(Level::Info, format!("Schon im Projekt: {r} ({})", plan.category.label())),
                        Outcome::Skipped(r) => self.st.log(Level::Info, format!("Übersprungen: {r}")),
                    }
                    let Some(rel) = o.rel().map(String::from) else { continue };
                    if let Some(doc) = &d.insert_into { if let Some(line) = self.include_line(doc, &rel) { lines.push(line); } }
                    let l = rel.to_ascii_lowercase();
                    if to_open.is_none() && (vc_media::naming::is_data_file(&l) || [".asm", ".inc", ".s", ".bas", ".lst", ".tur", ".txt", ".md"].iter().any(|e| l.ends_with(e))) { to_open = Some(rel); }
                }
                Err(e) => self.st.log(Level::Err, e),
            }
        }
        if pf != before {
            match (&file, &pf) {
                (Some(f), Some(pf)) => if let Err(e) = pf.save(&root.join(f)) { self.st.log(Level::Err, e); },
                _ => {}
            }
            self.st.pe = None;
        } else if file.is_none() { self.st.log(Level::Warn, "Das Projekt hat keine .vcprj – die Kategorie konnte nicht vermerkt werden"); }
        if let Some(pr) = &mut self.st.project { pr.reload(); pr.rescan(); }
        if let Some(doc) = &d.insert_into { self.insert_lines(doc, &lines); }
        if let Some(r) = to_open { self.st.open_request = Some(r); }
    }

    /// BASIC → Assembler (Dialog bestätigt): konvertieren, Ergebnis melden und öffnen
    fn run_convert(&mut self, root: &std::path::Path, req: &vc_ide::b2a::Req) {
        self.save_all_docs();
        let Some(data) = self.st.data.clone() else { self.st.log(Level::Err, "Hardware-Datenbank (data/) nicht gefunden"); return };
        match vc_ide::b2a::convert_file(root, &data, req) {
            Ok(rep) => {
                self.st.log(Level::Ok, format!("Konvertiert: {} nach {} (Stufe {}, {} Zeilen, Laufzeit: {})", req.rel, rep.out, rep.stage, rep.lines, rep.modules.join(", ")));
                for w in &rep.warnings { self.st.log(Level::Warn, format!("{}:{}: {} {}", req.rel, w.line, w.code, w.message)); }
                if rep.registered { if let Some(pr) = &mut self.st.project { pr.reload(); } self.st.pe = None; }
                if let Some(pr) = &mut self.st.project { pr.rescan(); }
                self.close_doc(&rep.out);
                self.open_doc(&rep.out);
            }
            Err(es) => {
                self.st.problems.clear();
                for e in &es {
                    self.st.log(Level::Err, format!("{}:{}: {} {}", req.rel, e.line, e.code, e.message));
                    self.st.problems.push(Problem { file: req.rel.clone(), line: e.line, error: e.is_error(), code: e.code.to_string(), msg: e.message.clone() });
                }
                self.st.log(Level::Err, format!("Konvertierung fehlgeschlagen: {} Meldung(en), siehe Probleme", es.len()));
            }
        }
    }

    fn show_file_dlg(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.file_dlg.take() else { return };
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        let mut keep = true; let mut do_it = false; let act;
        let valid_name = |n: &str| !n.is_empty() && !n.contains('/') && !n.contains('\\') && !n.starts_with('.') && n != "build";
        match &mut d {
            FileDlg::ConvertBasic { req } => {
                let clash = !req.out.trim().is_empty() && root.join(req.out.trim()).exists() && !vc_ide::b2a::is_generated(&root.join(req.out.trim()));
                let chips = [(req.target.to_uppercase(), crate::theme::accent())];
                act = dialog::show(ctx, &Dlg::new("Nach Assembler konvertieren", "Konvertieren").sub(req.rel.as_str()).chips(&chips), |ui| {
                    ui.horizontal(|ui| { widgets::label(ui, "Ausgabedatei"); ui.add(egui::TextEdit::singleline(&mut req.out).desired_width(280.0)); });
                    if clash { ui.colored_label(crate::theme::bad(), "Diese Datei gibt es schon und sie stammt nicht aus basic2asm – sie wird nicht überschrieben."); }
                    else if !req.out.trim().is_empty() && root.join(req.out.trim()).exists() { ui.label(RichText::new("Die erzeugte Datei wird ersetzt.").weak()); }
                    ui.horizontal(|ui| {
                        widgets::label(ui, "Zielsystem");
                        egui::ComboBox::from_id_salt("b2a_target").selected_text(req.target.to_uppercase()).show_ui(ui, |ui| { for t in vc_ide::b2a::targets() { ui.selectable_value(&mut req.target, t.to_string(), t.to_uppercase()); } });
                    });
                    ui.add_space(4.0);
                    widgets::card(ui, false, |ui| {
                        ui.label(RichText::new("Die Stufe – Ganzzahl, Fließkomma oder Zeichenketten – wählt der Konverter selbst. Fließkomma gibt es für C64, C128, Plus/4 und Atari.").weak().small());
                        ui.checkbox(&mut req.int_div, "„/“ als ganzzahlige Division übersetzen");
                        ui.checkbox(&mut req.force_float, "Fließkomma-Laufzeit auch für reine Ganzzahl-Programme");
                        ui.checkbox(&mut req.track_line, "Zeilennummer zur Laufzeit mitführen (Fehlermeldungen mit Zeile)");
                        ui.checkbox(&mut req.no_rem, "REM-Zeilen nicht übernehmen");
                        ui.checkbox(&mut req.turbo, "Quelle ist Turbo BASIC XL (nur Atari: WHILE, REPEAT, DO, PROC/EXEC, IF-Block, DIV, MOD)");
                        ui.checkbox(&mut req.inline, "Stil inline: einfache Ganzzahlausdrücke als direkte Befehle");
                        ui.checkbox(&mut req.embed, "Laufzeit in die Datei kopieren (statt #include)");
                        ui.checkbox(&mut req.register, "Ergebnis als Assembler-Quelle in die Projektdatei eintragen");
                    });
                    !clash
                });
            }
            FileDlg::Rename { rel, name, ask } => {
                // Einbindungen suchen (bei jeder Namensänderung neu): wer die Datei per .include/.incbin nennt, wird gefragt
                if ask.for_name != *name {
                    ask.for_name = name.clone();
                    ask.texts = if valid_name(name) && name != std::path::Path::new(rel.as_str()).file_name().and_then(|n| n.to_str()).unwrap_or("") && !vc_media::naming::is_data_file(rel) {
                        let to = std::path::Path::new(rel.as_str()).with_file_name(name.as_str()).to_string_lossy().replace('\\', "/");
                        vc_ide::reorganize::rewrites(&root, &[vc_ide::reorganize::Move { from: rel.clone(), to, category: vc_ide::importfiles::Category::Text, label: "", conflict: false }])
                    } else { vec![] };
                }
                let target = std::path::Path::new(rel.as_str()).with_file_name(name.as_str());
                let exists = !name.is_empty() && root.join(&target).exists() && root.join(rel.as_str()) != root.join(&target);
                let ok = valid_name(name) && !exists;
                act = dialog::show(ctx, &Dlg::new("Datei umbenennen", "Umbenennen").sub(rel.as_str()), |ui| {
                    ui.horizontal(|ui| { widgets::label(ui, "Neuer Name"); let r = ui.add(egui::TextEdit::singleline(name).desired_width(280.0)); r.request_focus(); });
                    if exists { ui.colored_label(crate::theme::bad(), "Diese Datei gibt es schon."); }
                    if !ask.texts.is_empty() {
                        let n: usize = ask.texts.iter().map(|t| t.2).sum();
                        ui.add_space(4.0);
                        ui.checkbox(&mut ask.rewrite, format!("Code anpassen: {n} Pfad(e) in {} Quelltext(en) auf den neuen Namen ändern", ask.texts.len()));
                        for t in &ask.texts { ui.label(RichText::new(format!("{}  ({})", t.0, t.2)).monospace().small().weak()); }
                        if !ask.rewrite { ui.colored_label(crate::theme::warn(), "Ohne Anpassung finden diese Quelltexte die Datei nicht mehr."); }
                    }
                    ok
                });
            }
            FileDlg::Delete { rel } => {
                let chips = [("nicht umkehrbar".to_string(), crate::theme::bad())];
                let hard = !can_trash();
                act = dialog::show(ctx, &Dlg::new("Datei löschen", "Löschen").sub(rel.as_str()).chips(if hard { &chips } else { &[] }), |ui| {
                    ui.label(format!("»{rel}« in den Papierkorb legen?"));
                    if hard { ui.colored_label(crate::theme::warn(), "Auf diesem System wird die Datei endgültig gelöscht."); }
                    true
                });
            }
            FileDlg::CreateInclude { rel } => {
                act = dialog::show(ctx, &Dlg::new("Datei anlegen?", "Anlegen").sub(rel.as_str()), |ui| {
                    ui.label(format!("»{rel}« gibt es noch nicht."));
                    ui.label("Leere Datei anlegen und öffnen?");
                    true
                });
            }
            FileDlg::NewFolder { dir, name } => {
                // Unterordner mit `/` im Namen sind erlaubt (`res/bin/fonts`), aber nur relativ und ohne `..`
                let rel = vc_ide::importfiles::join_rel(dir, name.trim_matches('/'));
                let exists = !name.trim().is_empty() && root.join(&rel).exists();
                let ok = !name.trim().is_empty() && vc_ide::importfiles::valid_dir(&rel) && !exists && !name.trim_start_matches('/').starts_with('.');
                let sub = if dir.is_empty() { "Projektordner".to_string() } else { format!("{dir}/") };
                act = dialog::show(ctx, &Dlg::new("Neuer Ordner", "Anlegen").sub(&sub), |ui| {
                    ui.horizontal(|ui| { widgets::label(ui, "Name"); let r = ui.add(egui::TextEdit::singleline(name).hint_text("name  oder  pfad/zum/ordner").desired_width(280.0)); r.request_focus(); });
                    if exists { ui.colored_label(crate::theme::bad(), "Diesen Ordner gibt es schon."); }
                    ok
                });
            }
            FileDlg::NewFile { dir, name } => {
                // geprüft wird der Name mit Endung, wie er angelegt würde; ein leeres Feld ist kein Fehler (nur „Anlegen“ bleibt gesperrt)
                let exists = file_exists_for(&root, dir, name, &self.default_ext(name));
                let ok = valid_name(name) && !exists;
                let sub = if dir.is_empty() { "Projektordner".to_string() } else { format!("{dir}/") };
                act = dialog::show(ctx, &Dlg::new("Neue Datei", "Anlegen").sub(&sub), |ui| {
                    ui.horizontal(|ui| { widgets::label(ui, "Name"); let r = ui.add(egui::TextEdit::singleline(name).hint_text("name  (ohne Endung: .asm bzw. .bas)").desired_width(280.0)); r.request_focus(); });
                    if exists { ui.colored_label(crate::theme::bad(), "Diese Datei gibt es schon."); }
                    ok
                });
            }
        }
        match act { Act::Primary => do_it = true, Act::Secondary => keep = false, Act::None => {} }
        if do_it {
            keep = false;
            match &d {
                FileDlg::ConvertBasic { req } => self.run_convert(&root, req),
                FileDlg::Rename { rel, name, ask } => {
                    let new = std::path::Path::new(rel).with_file_name(name).to_string_lossy().into_owned();
                    let was_open = self.st.docs.iter().any(|x| &x.rel == rel);
                    if was_open { self.save_all_docs(); self.close_doc(rel); }
                    // Code anpassen (auf Wunsch): erst die Texte am alten Ort, dann umbenennen
                    let mut reopen: Vec<String> = vec![];
                    if ask.rewrite && !ask.texts.is_empty() {
                        self.save_all_docs();
                        // nach dem Speichern neu berechnen: die Texte auf der Platte sind jetzt maßgeblich
                        let rw = vc_ide::reorganize::rewrites(&root, &[vc_ide::reorganize::Move { from: rel.clone(), to: new.replace('\\', "/"), category: vc_ide::importfiles::Category::Text, label: "", conflict: false }]);
                        for t in &rw { if self.st.docs.iter().any(|x| x.rel == t.0) { reopen.push(t.0.clone()); } self.close_doc(&t.0); }
                        match vc_ide::reorganize::write_rewrites(&root, &rw) { Ok(()) => self.st.log(Level::Ok, format!("Pfade angepasst in {} Quelltext(en)", rw.len())), Err(e) => self.st.log(Level::Err, e) }
                    }
                    reopen.retain(|r| r != rel);
                    for r in reopen { self.open_doc(&r); }
                    match std::fs::rename(root.join(rel), root.join(&new)) {
                        Ok(()) => { if vc_media::naming::is_data_file(rel) { self.rename_export_refs(rel, &new); } self.pf_paths(rel, Some(&new)); if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Umbenannt: {rel} → {new}")); if was_open { self.open_doc(&new); } }
                        Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")),
                    }
                }
                FileDlg::Delete { rel } => {
                    self.close_doc(rel);
                    let abs = root.join(rel);
                    let moved = move_to_trash(&abs);
                    let res = if moved { Ok(()) } else { std::fs::remove_file(&abs) };
                    match res { Ok(()) => { self.pf_paths(rel, None); if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Gelöscht: {rel}")); } Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")) }
                }
                FileDlg::CreateInclude { rel } => {
                    let abs = root.join(rel);
                    if let Some(parent) = abs.parent() { let _ = std::fs::create_dir_all(parent); }
                    match std::fs::write(&abs, "") {
                        Ok(()) => { if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Angelegt: {rel}")); self.open_doc(rel); }
                        Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")),
                    }
                }
                FileDlg::NewFolder { dir, name } => {
                    let rel = vc_ide::importfiles::join_rel(dir, name.trim_matches('/'));
                    match std::fs::create_dir_all(root.join(&rel)) {
                        Ok(()) => { if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Ordner angelegt: {rel}/")); }
                        Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")),
                    }
                }
                FileDlg::NewFile { dir, name } => {
                    let name = &self.default_ext(name);
                    let rel = if dir.is_empty() { name.clone() } else { format!("{dir}/{name}") };
                    let content = if rel.ends_with(".vctest") { self.test_template() } else { String::new() };
                    match std::fs::write(root.join(&rel), content) {
                        Ok(()) => { if let Some(pr) = &mut self.st.project { pr.rescan(); } self.st.log(Level::Ok, format!("Angelegt: {rel}")); self.open_doc(&rel); }
                        Err(e) => self.st.log(Level::Err, format!("{rel}: {e}")),
                    }
                }
            }
        }
        if keep { self.file_dlg = Some(d); }
    }

    /// Aktionen aus dem Reiter-Kontextmenü
    fn run_tab_action(&mut self, ctx: &egui::Context, a: TabAct) {
        match a {
            TabAct::Second(r) => { self.st.preview = self.st.preview.take().filter(|p| *p != r); layout::open_in_second(&mut self.dock, Tab::Doc(r)); return; }
            TabAct::Unsplit => { layout::unsplit(&mut self.dock); return; }
            _ => {}
        }
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        match a {
            TabAct::CloseOthers(keep) => self.close_docs_except(Some(&keep)),
            TabAct::CloseAll => self.close_docs_except(None),
            TabAct::Save(r) => { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == r) { match d.save() { Ok(()) => self.st.status = format!("gespeichert: {r}"), Err(e) => self.st.log(Level::Err, e) } } }
            TabAct::Discard(r) => { self.close_doc(&r); self.open_doc(&r); self.st.status = format!("neu geladen: {r}"); }
            TabAct::CopyPath(r) => ctx.copy_text(root.join(&r).to_string_lossy().into_owned()),
            TabAct::Reveal(r) => reveal_in_file_manager(&root.join(&r).to_string_lossy()),
            TabAct::Second(_) | TabAct::Unsplit => {}
        }
    }

    /// Alle Quelltext-Reiter außer `keep` schließen; Dateien mit ungespeicherten Änderungen bleiben offen
    fn close_docs_except(&mut self, keep: Option<&str>) {
        let all: Vec<(String, bool)> = self.st.docs.iter().filter(|d| Some(d.rel.as_str()) != keep).map(|d| (d.rel.clone(), d.dirty())).collect();
        let skipped = all.iter().filter(|(_, dirty)| *dirty).count();
        for (r, dirty) in all { if !dirty { self.close_doc(&r); } }
        if skipped > 0 { self.st.log(Level::Warn, format!("{skipped} Datei(en) mit ungespeicherten Änderungen bleiben offen")); }
    }

    /// Aktionen aus dem Kontextmenü des Editors ausführen (Hilfe, Datei öffnen, Definition, bis hierher ausführen)
    fn run_doc_actions(&mut self) {
        let acts: Vec<(String, editor::DocAction)> = self.st.docs.iter_mut().filter_map(|d| d.action.take().map(|a| (d.rel.clone(), a))).collect();
        if acts.is_empty() { return; }
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        // Pfad ohne `.`/`..`-Anteile (Include-Pfade wie `../lib/x.inc`)
        let norm = |p: &std::path::Path| -> String {
            let mut parts: Vec<String> = vec![];
            for c in p.components() { match c { std::path::Component::ParentDir => { parts.pop(); } std::path::Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()), _ => {} } }
            parts.join("/")
        };
        for (rel, a) in acts {
            match a {
                editor::DocAction::CharPicker => { self.charpick_doc = Some(rel.clone()); self.open_charpick(); }
                editor::DocAction::Help(w) => { self.st.help_open(&w); self.st.help.want_tab = true; }
                editor::DocAction::OpenFile(p) => {
                    let dir = std::path::Path::new(&rel).parent().map(|d| d.to_path_buf()).unwrap_or_default();
                    let found = [norm(&dir.join(&p)), norm(std::path::Path::new(&p))].into_iter().find(|c| !c.is_empty() && root.join(c).is_file());
                    match found {
                        Some(c) => self.st.open_request = Some(c),
                        None => {
                            // fehlt die Datei und sieht sie nach Quelltext aus: anbieten, sie anzulegen (Pfad wie im Assembler: relativ zur einbindenden Datei)
                            let target = norm(&dir.join(&p));
                            // exportierte Mediendatei (build/export/name.asm): die Quelle (Sprite, Zeichensatz, Song …) öffnen
                            if let Some(stem) = target.strip_prefix("build/export/").and_then(|t| t.strip_suffix(".data.bas").or(t.strip_suffix(".asm"))) {
                                let src = self.st.project.as_ref().and_then(|pr| pr.entries.iter().find(|e| vc_media::naming::is_data_file(&e.rel) && vc_media::naming::stem(&e.rel) == stem).map(|e| e.rel.clone()));
                                if let Some(r) = src { self.st.open_request = Some(r); continue; }
                            }
                            let ext = std::path::Path::new(&target).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
                            if !target.is_empty() && [ "asm", "inc", "s", "bas", "lst", "tur", "txt" ].contains(&ext.as_str()) { self.file_dlg = Some(FileDlg::CreateInclude { rel: target }); }
                            else { self.st.log(Level::Warn, format!("{p}: Datei nicht gefunden")); }
                        }
                    }
                }
                editor::DocAction::Definition { line, col } => {
                    self.st.def_req += 1;
                    if let Some(l) = &self.st.lsp { l.send(lsp::Cmd::Definition { uri: lsp::uri(&root, &rel), line, col, req: self.st.def_req }); }
                }
                editor::DocAction::RunTo(line) => {
                    let addr = self.st.info.as_ref().and_then(|i| i.addr_of(&rel, line + 1));
                    match (addr, self.st.dbg.is_some()) {
                        (Some(a), true) => {
                            // einmaliger Haltepunkt: wird nach dem nächsten Halt wieder entfernt
                            if let Some(d) = &self.st.dbg { d.send(DbgCmd::AddPoint { kind: "exec".into(), start: a, end: a, cond: None, ignore: 0 }); d.send(DbgCmd::Cont); }
                            self.st.run_to = Some((a, false));
                            self.st.status = format!("läuft bis ${a:04X}");
                        }
                        (None, _) => self.st.log(Level::Warn, "Diese Zeile erzeugt keinen Code (oder das Programm wurde seit dem Bauen geändert)"),
                        (_, false) => self.st.log(Level::Warn, "Keine Debug-Sitzung"),
                    }
                }
            }
        }
    }

    fn save_active(&mut self) {
        if let Some(r) = self.active_doc() { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == r) { match d.save() { Ok(()) => self.st.status = format!("gespeichert: {r}"), Err(e) => self.st.log(Level::Err, e) } } }
    }

    /// Das Projekt hat Quellen laut Projektdatei
    fn has_sources(&self) -> bool { self.st.project.as_ref().and_then(|p| p.pf.as_ref()).map_or(false, |pf| !pf.source_files().is_empty()) }

    /// Primäre Aktion: „Bauen“ baut das ganze Projekt; „Starten“/„Debuggen“ bauen das Startprogramm (ohne Projektdatei-Quellen: die aktive Datei).
    fn build(&mut self, ctx: &egui::Context, after: After) {
        if self.has_sources() {
            self.save_all_docs();
            let main = self.st.project.as_ref().and_then(|p| p.pf.as_ref()).and_then(|pf| pf.main_file());
            let with_disk = after == After::Run && self.st.project.as_ref().and_then(|p| p.pf.as_ref().and_then(|pf| pf.run_disk(&p.root))).is_some();   // Start von der Diskette: alles bauen
            let kind = if after == After::Nothing || with_disk { BuildKind::Project(None) } else { match main { Some(m) => BuildKind::Project(Some(m)), None => { self.st.log(Level::Warn, "Kein Startprogramm festgelegt"); return } } };
            self.start_build(ctx, kind, after);
        } else { self.build_active(ctx, after); }
    }

    /// Nur die im Editor aktive Datei bauen (und ggf. starten)
    fn build_active(&mut self, ctx: &egui::Context, after: After) {
        let Some(rel) = self.active_doc() else { self.st.log(Level::Warn, "Keine Quelldatei aktiv"); return };
        self.save_active();
        self.start_build(ctx, BuildKind::File(rel), after);
    }

    fn save_all_docs(&mut self) {
        let mut errs = vec![];
        for d in self.st.docs.iter_mut().filter(|d| d.dirty()) { if let Err(e) = d.save() { errs.push(e); } }
        for e in errs { self.st.log(Level::Err, e); }
    }

    fn start_build(&mut self, ctx: &egui::Context, kind: BuildKind, after: After) {
        let (Some(pr), Some(data)) = (&self.st.project, self.st.data.clone()) else { return };
        let root = pr.root.clone(); let pf = pr.pf.clone();
        let label = match &kind { BuildKind::File(f) => f.clone(), BuildKind::Project(None) => "Projekt".into(), BuildKind::Project(Some(m)) => m.clone() };
        // Ohne ausdrückliche Wahl gilt für einzelne Quellen das Target der Projektdatei
        let target = if self.st.target == "auto" { match (&kind, &pf) { (BuildKind::File(f), Some(pf)) if pf.source_files().iter().any(|s| &s.0 == f) => pf.target_of(f), _ => None } } else { Some(self.st.target.clone()) };
        let (tx, rx) = channel(); let ctx = ctx.clone(); let l2 = label.clone();
        std::thread::spawn(move || {
            let c = vc_ide::build::Ctx { root: &root, data: &data };
            let result = match (&kind, &pf) {
                (BuildKind::File(f), _) => vc_ide::build::build(&c, f, target.as_deref()),
                (BuildKind::Project(only), Some(pf)) => Ok(vc_ide::projfile::build_project(&c, pf, only.as_deref())),
                (BuildKind::Project(_), None) => Err("Keine Projektdatei (.vcprj)".to_string()),
            };
            let _ = tx.send(BuildResult { file: l2, result, after }); ctx.request_repaint();
        });
        self.st.building = Some(rx); self.st.status = format!("baut {label} …");
    }

    fn start_run(&mut self) {
        self.st.tests.stop_blocking();                          // nur ein Emulator gleichzeitig
        if self.st.dbg.is_some() { self.stop_debug(); }        // nur ein Emulator gleichzeitig
        let Some((info_art, info_target)) = self.st.info.as_ref().map(|i| (i.artifact.clone(), i.target.clone())) else { return };
        let root = self.st.project.as_ref().map(|p| p.root.clone()).unwrap_or_default();
        // Echte Maschine aus den Einstellungen: im Hintergrund (Netzwerk/Seriell kann dauern)
        if let Some(m) = vc_ide::settings::effective().start_machine(&self.st.run_on, &info_target).cloned() {
            let (art, target, msgs, ctx) = (info_art.clone(), info_target.clone(), self.st.bg_msgs.clone(), self.st.ctx.clone());
            // MEGA65 über Ethernet: hat das Projekt eine Diskette mit dem Startprogramm, wird sie kopiert, eingebunden und das Programm gestartet (ungeprüft)
            let disk = if vc_ide::launch::machine_runs_disks(&m) { self.st.project.as_ref().and_then(|p| p.pf.as_ref().and_then(|pf| pf.run_disk_for(&p.root, true))).filter(|d| d.is_file()) } else { None };
            self.st.status = format!("startet auf {} …", if m.name.is_empty() { &m.id } else { &m.name });
            std::thread::spawn(move || { let r = match &disk { Some(d) => vc_ide::launch::run_disk_on_machine(&m, d, &art, &target), None => vc_ide::launch::run_on_machine(&m, &art, &target) }; if let Ok(mut g) = msgs.lock() { g.push(match r { Ok(t) => (true, t), Err(e) => (false, e) }); } ctx.request_repaint(); });
            return;
        }
        // Hat das Projekt eine Diskette mit dem Startprogramm vorn, startet der Emulator die Diskette (Nachladen von Programmen)
        let disk = self.st.project.as_ref().and_then(|p| p.pf.as_ref().and_then(|pf| pf.run_disk(&p.root))).filter(|d| d.is_file());
        let art = disk.clone().unwrap_or(info_art);
        if let Some(d) = &disk { let n = d.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(); self.st.log(Level::Info, format!("Start von der Diskette {n}")); }
        match vc_ide::launch::spawn(&art, &info_target, &vc_ide::launch::atari_variant(&root), &self.st.data.as_deref().map(|d| vc_ide::launch::project_args(d, &root, &info_target)).unwrap_or_default()) {
            Ok(cmd) => { self.st.log(Level::Ok, format!("gestartet: {cmd}")); self.st.status = "gestartet".into(); }
            Err(e) => self.log_start_error(e),
        }
    }

    fn start_debug(&mut self, ctx: &egui::Context) {
        self.st.tests.stop_blocking();                          // nur ein Emulator gleichzeitig
        self.stop_debug();
        vc_ide::launch::stop_running();                        // nur ein Emulator gleichzeitig
        let (Some(info), Some(data)) = (&self.st.info, self.st.data.clone()) else { return };
        let (target, artifact, symbols, run) = (info.target.clone(), info.artifact.clone(), info.symbols.clone(), info.run);
        let cpu = match vc_target::load_target(&data, &target).and_then(|t| vc_target::load_cpu(&data, &t.cpu)) { Ok(c) => c, Err(e) => { self.st.log(Level::Err, e); return } };
        // echte Maschine aus den Einstellungen oder Emulator
        let machine = vc_ide::settings::effective().start_machine(&self.st.run_on, &target).cloned();
        if let Some(m) = machine.as_ref().filter(|m| m.kind == "mega65" && m.link_for(vc_ide::mega65net::Action::Debug) == vc_ide::mega65net::Link::Eth) {
            self.st.log(Level::Err, format!("{}: Debuggen geht nur über USB (Ethernet bietet keinen Monitor). Unter Einstellungen → Hardware die Verbindung „Debuggen“ auf USB stellen.", m.display_name()));
            return;
        }
        let (adapter, host, password): (String, Option<String>, Option<String>) = match machine {
            Some(m) if m.kind == "ultimate64" => ("ultimate64".into(), Some(m.host.clone()), Some(m.password.clone())),
            Some(m) if m.kind == "mega65" => ("mega65".into(), vc_ide::settings::mega65_spec(&m).ok(), None),
            _ => (if target == "mega65" { "mega65".into() } else { "emulator".into() }, if target == "mega65" { Some("xemu".into()) } else { None }, None),
        };
        let spec = vc_ide::debug::LaunchSpec { artifact: artifact.to_string_lossy().into_owned(), target: target.clone(), adapter, host, password, entry: if target == "mega65" { run } else { None }, extra_args: self.st.data.as_deref().zip(self.st.project.as_ref()).map(|(d, p)| vc_ide::launch::project_args(d, &p.root, &target)).unwrap_or_default() };
        let bps = self.desired_breakpoints();
        self.st.applied = bps.iter().copied().collect();
        self.st.notes_seen = 0; self.st.err_seen = 0;
        self.forget_b2a_maps();
        self.st.dbg = Some(Debugger::start(spec, cpu, symbols, bps, ctx.clone()));
        self.st.log(Level::Info, "Debug-Sitzung startet …".to_string()); self.st.status = "Emulator startet …".into();
        self.switch_mode(Mode::Debug);
    }

    fn stop_debug(&mut self) {
        self.st.dbg = None;                // Drop beendet den Emulator
        self.forget_b2a_maps();
        self.switch_mode(Mode::Code);
        for d in &mut self.st.docs { d.pc_line = None; } self.st.last_pc = None; self.st.applied.clear();
    }

    /// Zeilenkarten konvertierter BASIC-Programme (basic2asm), je Debug-Sitzung einmal gelesen
    fn b2a_maps(&self) -> std::sync::Arc<Vec<vc_ide::b2a::LineMap>> {
        let id = egui::Id::new("b2a_line_maps");
        if let Some(m) = self.st.ctx.data(|d| d.get_temp::<std::sync::Arc<Vec<vc_ide::b2a::LineMap>>>(id)) { return m; }
        let maps = match self.st.project.as_ref().and_then(|pr| pr.pf.as_ref().map(|pf| vc_ide::b2a::load_maps(&pr.root, pf))) { Some(m) => m, None => vec![] };
        let a = std::sync::Arc::new(maps);
        self.st.ctx.data_mut(|d| d.insert_temp(id, a.clone()));
        a
    }

    fn forget_b2a_maps(&self) { self.st.ctx.data_mut(|d| d.remove_temp::<std::sync::Arc<Vec<vc_ide::b2a::LineMap>>>(egui::Id::new("b2a_line_maps"))); }

    /// Haltepunkte der Editoren als Adressen (nur mit Zeilen-Zuordnung des letzten Baus); Haltepunkte in einer `.bas`, die mit
    /// basic2asm konvertiert wurde, gelten für den Code ihrer BASIC-Zeile im Assembler-Ergebnis
    fn desired_breakpoints(&self) -> Vec<u16> {
        let Some(info) = &self.st.info else { return vec![] };
        let maps = self.b2a_maps();
        let sym = |n: &str| info.symbols.get(n).copied();
        let mut v: Vec<u16> = self.st.docs.iter().flat_map(|d| {
            let mut a: Vec<u16> = d.breakpoints.iter().filter_map(|l| info.addr_of(&d.rel, l + 1)).collect();
            for m in maps.iter().filter(|m| m.source == d.rel) { a.extend(d.breakpoints.iter().filter_map(|l| m.addr_of(&sym, l + 1))); }
            a
        }).collect();
        v.sort(); v.dedup(); v
    }

    /// Pro Bild: Haltepunkte abgleichen, Ausführungszeile markieren, Sitzungsende erkennen
    fn sync_debug(&mut self) {
        let Some(snap) = self.st.dbg.as_ref().map(|d| d.snapshot()) else { return };
        // Jeder Fehler eines Debugger-Befehls erscheint in Ausgabe und Statusleiste (auch derselbe Fehler bei jedem Versuch); das Panel zeigt den letzten
        if snap.error_seq != self.st.err_seen {
            self.st.err_seen = snap.error_seq;
            if let Some(e) = &snap.error { self.st.log(Level::Err, e.clone()); self.st.status = e.clone(); }
        } else if let (Some(e), 0) = (&snap.error, snap.error_seq) { if self.st.output.last().map_or(true, |(_, m)| m != e) { self.st.log(Level::Err, e.clone()); } }
        if snap.ended { self.st.log(Level::Info, "Debug-Sitzung beendet".to_string()); self.stop_debug(); self.st.status = "Debugger beendet".into(); return; }
        while self.st.notes_seen < snap.notes.len() {
            let (ok, m) = snap.notes[self.st.notes_seen].clone(); self.st.notes_seen += 1;
            self.st.log(if ok { Level::Ok } else { Level::Err }, m.clone());
            if ok { if let Some(rel) = m.rsplit(": ").next().map(String::from) { if let Some(pr) = &mut self.st.project { pr.rescan(); } self.open_media(&rel); } }
        }
        // „Bis hierher ausführen“: der einmalige Haltepunkt verschwindet, sobald der Lauf wieder angehalten hat
        if let Some((a, seen_running)) = self.st.run_to {
            if snap.running { self.st.run_to = Some((a, true)); }
            else if seen_running && !snap.starting {
                if let Some(dbg) = &self.st.dbg { for p in snap.points.iter().filter(|p| p.kind == "exec" && p.start == a && p.end == a) { dbg.send(DbgCmd::DelPoint(p.id)); } }
                self.st.run_to = None;
            }
        }
        let want: HashSet<u16> = self.desired_breakpoints().into_iter().collect();
        if !snap.starting {
            let (add, del): (Vec<u16>, Vec<u16>) = (want.difference(&self.st.applied).copied().collect(), self.st.applied.difference(&want).copied().collect());
            if let Some(dbg) = &self.st.dbg { for a in add { dbg.send(DbgCmd::Bp(a, true)); } for a in del { dbg.send(DbgCmd::Bp(a, false)); } }
            self.st.applied = want;
        }
        let pcl = if !snap.running && snap.have_regs { self.st.info.as_ref().and_then(|i| i.line_at(snap.pc)).map(|(f, n)| (f.to_string(), n)) } else { None };
        for d in &mut self.st.docs { d.pc_line = pcl.as_ref().filter(|(f, _)| f == &d.rel).map(|(_, n)| n - 1); }
        // BASIC-Zeile im konvertierten Programm: die Quelle (.bas) zeigt dieselbe Ausführungsmarke wie der Assembler
        let maps = self.b2a_maps();
        let bas_line = if !snap.running && snap.have_regs && !maps.is_empty() {
            self.st.info.as_ref().and_then(|i| { let sym = |n: &str| i.symbols.get(n).copied(); maps.iter().find_map(|m| m.line_at(&sym, snap.pc).map(|(b, s)| (m.source.clone(), b, s))) })
        } else { None };
        if let Some((src, _, s)) = &bas_line { for d in &mut self.st.docs { if &d.rel == src { d.pc_line = Some(s - 1); if pcl != self.st.last_pc { d.reveal = Some(s - 1); } } } }
        if pcl != self.st.last_pc { if let Some((f, n)) = &pcl { self.st.open_request = Some(f.clone()); if let Some(d) = self.st.docs.iter_mut().find(|d| &d.rel == f) { d.reveal = Some(n - 1); } } self.st.last_pc = pcl; }
        self.st.status = if snap.starting { "Emulator startet …".into() } else if snap.running { "läuft".into() } else { match &bas_line { Some((_, b, _)) => format!("angehalten bei ${:04X} – BASIC-Zeile {b}", snap.pc), None => format!("angehalten bei ${:04X}", snap.pc) } };
    }

    fn poll_lsp(&mut self) {
        let Some(l) = &self.st.lsp else { return };
        let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) else { return };
        while let Ok(r) = l.rx.try_recv() {
            match r {
                Reply::Diagnostics { uri, items } => if let Some(d) = self.st.docs.iter_mut().find(|d| lsp::uri(&root, &d.rel) == uri) {
                    d.diag.clear(); for i in items { if i.error { d.diag.entry(i.line).or_insert(format!("{} {}", i.code, i.msg)); } }
                },
                Reply::Addresses { uri, lines, source } => if let Some(d) = self.st.docs.iter_mut().find(|d| lsp::uri(&root, &d.rel) == uri) {
                    d.addrs = lines.into_iter().map(|(l, a, n)| (l, (a, n))).collect(); d.addr_src = source;
                },
                Reply::Definition { uri, line } => { if let Some(rel) = vc_core::paths::uri_to_rel(&root, &uri) { self.st.open_request = Some(rel.clone()); self.st.pending_reveal = Some((rel, line)); } else { self.st.output.push((Level::Warn, "Definition liegt außerhalb des Projekts".into())); } }
                Reply::NoDefinition => { self.st.output.push((Level::Warn, "Definition nicht gefunden".into())); self.st.status = "Definition nicht gefunden".into(); }
                Reply::Hover { req, text } => if let Some(h) = self.st.docs.iter_mut().filter_map(|d| d.hover.as_mut()).find(|h| h.req == req) { h.text = Some(text); },
                Reply::Completion { req, items } => if let Some(d) = self.st.docs.iter_mut().find(|d| d.comp.req == req) { d.comp.all = items; d.comp.open = true; },
            }
        }
    }

    fn poll_build(&mut self) {
        let Some(rx) = &self.st.building else { return };
        let Ok(r) = rx.try_recv() else { return };
        self.st.building = None; self.st.problems.clear();
        for d in &mut self.st.docs { d.errors.clear(); }
        match r.result {
            Err(e) => { self.st.log(Level::Err, e); self.st.status = "Build fehlgeschlagen".into(); self.reveal(Tab::Output); }   // kein Ergebnis, nur eine Meldung: die Ausgabe zeigt sie
            Ok(v) => {
                let is_project = v.get("results").is_some();
                let results: Vec<Value> = if is_project { v["results"].as_array().cloned().unwrap_or_default() } else { vec![v.clone()] };
                self.st.ov.built_now();
                self.st.buildlog = results.iter().filter_map(|r| Some(BuildLogEntry { file: r["file"].as_str()?.to_string(), ok: r["ok"] == true, artifact: r["artifact"].as_str().unwrap_or("").to_string(), log: r.get("log")?.clone() })).collect();
                for res in &results {
                    for d in res["diagnostics"].as_array().into_iter().flatten() {
                        let (file, line) = (d["file"].as_str().unwrap_or("").to_string(), d["line"].as_u64().unwrap_or(0) as usize);
                        let error = d["severity"] == "error";
                        self.st.problems.push(Problem { file: file.clone(), line, error, code: d["code"].as_str().unwrap_or("").into(), msg: d["message"].as_str().unwrap_or("").into() });
                        if error && line > 0 { if let Some(doc) = self.st.docs.iter_mut().find(|x| x.rel == file) { doc.errors.entry(line - 1).or_insert_with(|| d["message"].as_str().unwrap_or("").to_string()); } }
                    }
                    if res["ok"] == true { let f = res["file"].as_str().unwrap_or(&r.file).to_string(); self.st.log(Level::Ok, format!("Build OK · {} -> {} ({} Bytes)", f, res["artifact"].as_str().unwrap_or("?"), res["size"])); }
                    else if is_project { let f = res["file"].as_str().unwrap_or("?").to_string(); self.st.log(Level::Err, format!("Build fehlgeschlagen · {f}")); }
                }
                // Fußpanel: Fehler öffnen es (Probleme), ein fehlerfreier Bau zeigt das Protokoll nur, wo das Panel ohnehin offen ist
                match ctx::build_footer_action(self.st.problems.iter().filter(|p| p.error).count(), self.layout_wp().footer && !self.zen) {
                    ctx::FooterAct::ShowProblems => self.reveal(Tab::Problems),
                    ctx::FooterAct::SelectBuildLog if self.mode == Mode::Code && !self.st.buildlog.is_empty() => self.zones_mut().zone_mut(Zone::Footer).show(&Tab::BuildLog),
                    _ => {}
                }
                if let Some(d) = v["disk"].as_object() { self.st.log(Level::Ok, format!("Diskette · {} ({} Bytes): {}", d["file"].as_str().unwrap_or(""), d["size"], d["programs"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default())); }
                if let Some(e) = v["disk_error"].as_str() { self.st.log(Level::Err, format!("Diskette: {e}")); }
                // Für Starten und Debuggen zählt das Startprogramm (bei Projektbau) bzw. die gebaute Datei
                let main = v["main"].as_str().map(String::from);
                let src = if is_project { results.iter().find(|x| x["ok"] == true && x["file"].as_str() == main.as_deref()) } else { results.first().filter(|x| x["ok"] == true) };
                let mut have_info = false;
                if let Some(res) = src {
                    if let (Some(pr), Some(art)) = (&self.st.project, res["artifact"].as_str()) {
                        let lines = res["lines"].as_array().map(|a| a.iter().filter_map(|l| Some((l["file"].as_str()?.to_string(), l["line"].as_u64()? as usize, l["addr"].as_u64()? as u16, l["len"].as_u64().unwrap_or(1) as usize))).collect()).unwrap_or_default();
                        let symbols: HashMap<String, i64> = res["symbols"].as_array().map(|a| a.iter().filter_map(|x| Some((x["name"].as_str()?.to_string(), x["value"].as_i64()?))).collect()).unwrap_or_default();
                        let labels = symbols.iter().filter(|(n, val)| **val >= 0 && **val < 0x10000 && !n.contains(".__m") && !n.contains('.') ).map(|(n, val)| (*val as u16, n.clone())).collect();
                        let tid = res["target"].as_str().map(String::from).unwrap_or_else(|| vc_target::default_target_or_c64(self.st.data.as_deref()));
                        let segments = res["segments"].as_array().map(|a| a.iter().filter_map(|x| Some((x["start"].as_u64()? as u32, x["length"].as_u64()? as usize))).collect()).unwrap_or_default();
                        let regions = self.st.data.as_ref().and_then(|d| vc_core::read_static(&d.join("targets").join(format!("{tid}.target.json"))).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok())
                            .and_then(|j| j["regions"].as_array().map(|a| a.iter().filter_map(|r| { let p = |k: &str| r[k].as_str().and_then(|s| u32::from_str_radix(s.trim_start_matches('$'), 16).ok()); Some((r["name"].as_str()?.to_string(), p("start")?, p("end")?, r["kind"].as_str().unwrap_or("ram").to_string())) }).collect())).unwrap_or_default();
                        self.st.info = Some(BuildInfo { target: tid, artifact: pr.root.join(art), lines, symbols, labels, run: res["run"].as_u64().map(|r| r as u16), segments, regions });
                    }
                    have_info = self.st.info.as_ref().map_or(false, |i| res["artifact"].as_str().map_or(false, |a| self.st.project.as_ref().map_or(false, |p| p.root.join(a) == i.artifact)));
                }
                if v["ok"] == true {
                    self.st.status = if is_project { format!("Projekt gebaut ({} Programme)", results.len()) } else { "Build OK".into() };
                    if have_info { match r.after { After::Run => self.start_run(), After::Debug => { let ctx = self.st.ctx.clone(); self.start_debug(&ctx); } After::Nothing => {} } }
                    else if r.after != After::Nothing { self.st.log(Level::Warn, "Kein Startprogramm zum Starten gebaut"); }
                } else { let n = self.st.problems.iter().filter(|p| p.error).count(); self.st.log(Level::Err, format!("Build fehlgeschlagen: {n} Fehler")); self.st.status = "Build fehlgeschlagen".into(); }
            }
        }
        if let Some(pr) = &mut self.st.project { pr.rescan(); }
    }
}

/// Hinweis „Bilder aus dem Internet laden?“: 1 = Laden (diesmal), 2 = Nicht laden, 3 = Immer erlauben, 0 = keine Antwort
fn help_consent_box(ui: &mut egui::Ui, host: &str) -> u8 {
    let k = crate::ui::tokens::t();
    let mut ans = 0;
    egui::Frame::new().fill(k.cy.gamma_multiply(0.13)).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.55))).corner_radius(6.0).inner_margin(egui::Margin::symmetric(10, 8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("Bilder aus dem Internet laden?").strong().color(k.fg));
        ui.label(format!("Die Bilder dieses Kapitels liegen nicht in der Programmdatei, sondern auf {host}. Beim Laden wird nur die Adresse jedes Bildes abgerufen, es werden keine Daten von dir gesendet. Geladene Bilder bleiben auf der Platte (Zwischenspeicher). Bis du zustimmst, zeigt die Hilfe Platzhalter. Die Wahl lässt sich unter Einstellungen > Allgemein > Hilfe ändern."));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Laden").clicked() { ans = 1; }
            if ui.button("Nicht laden").clicked() { ans = 2; }
            if ui.button("Immer erlauben").clicked() { ans = 3; }
        });
    });
    ui.add_space(6.0);
    ans
}

impl State {
    fn help_target(&self) -> String {
        self.info.as_ref().map(|i| i.target.clone()).or_else(|| self.project.as_ref().and_then(|p| p.systems.first().cloned())).unwrap_or_else(|| vc_target::default_target_or_c64(self.data.as_deref()))
    }
    /// Slash-Hilfe: Thema (`id`) oder Wort (Befehl, Direktive, Register)
    fn help_lookup(&self, key: &str) -> Option<(String, String)> {
        if let Some(t) = vc_ide::help::topic(key) { return Some((t["title"].as_str()?.into(), t["markdown"].as_str()?.into())); }
        let data = self.data.as_ref()?;
        let lang = if self.active_lang == crate::highlight::Lang::Basic { "basic" } else { "asm" };
        let v = vc_ide::help::lookup(data, &self.help_target(), key, lang).ok()?;
        if v["kind"] == "none" && v["markdown"].as_str().map_or(true, |m| m.is_empty()) { return None; }
        Some((v["title"].as_str().unwrap_or(key).into(), v["markdown"].as_str()?.into()))
    }
    fn help_open(&mut self, key: &str) {
        if let Some(d) = self.help_lookup(key) { if let Some(cur) = self.help.doc.take() { self.help.hist.push(cur); } self.help.doc = Some(d); }
    }

    /// Wahl zum Nachladen der Handbuchbilder dauerhaft speichern (Einstellungsdatei und offener Einstellungsentwurf)
    fn set_help_images(&mut self, c: vc_ide::help_images::Consent) {
        let mut s = vc_ide::settings::get(); s.help.images = c.setting().into();
        if let Err(e) = vc_ide::settings::set(s) { self.output.push((Level::Warn, format!("Einstellung „Hilfe-Bilder“ nicht gespeichert: {e}"))); }
        if let Some(e) = &mut self.se { e.draft.help.images = c.setting().into(); e.saved.help.images = c.setting().into(); }
        self.help.images.sync();
    }

    fn help_ui(&mut self, ui: &mut egui::Ui) {
        self.help.images.sync();
        self.help.images.poll(ui.ctx());
        let mut consent: Option<Option<vc_ide::help_images::Consent>> = None;   // Some(None) = nur diesmal laden, Some(Some(c)) = dauerhaft
        let mut decline = false;
        if self.help.topics.is_empty() { if let Some(l) = vc_ide::help::topic_list()["topics"].as_array() { self.help.topics = l.iter().map(|t| (t["id"].as_str().unwrap_or("").into(), t["group"].as_str().unwrap_or("").into(), t["title"].as_str().unwrap_or("").into())).collect(); } }
        let mut open: Option<String> = None;
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.help.hist.is_empty(), egui::Button::new("Zurück")).clicked() { self.help.doc = self.help.hist.pop(); }
            if ui.button("Inhalt").clicked() { if let Some(cur) = self.help.doc.take() { self.help.hist.push(cur); } }
            let r = ui.add(egui::TextEdit::singleline(&mut self.help.search).hint_text("Suchen: Thema, Befehl, Register …").desired_width(ui.available_width()));
            if r.changed() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                self.help.results = match (&self.data, self.help.search.trim().is_empty()) { (Some(d), false) => vc_ide::help::search(d, &self.help_target(), &self.help.search).ok().and_then(|v| v["results"].as_array().map(|a| a.iter().map(|x| (x["id"].as_str().unwrap_or("").into(), x["title"].as_str().unwrap_or("").into(), x["snippet"].as_str().unwrap_or("").into())).collect())).unwrap_or_default(), _ => vec![] };
            }
        });
        ui.separator();
        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        // Selbsttest (Screenshots): VC_HELP_SCROLL=<Pixel> blättert die Hilfeseite
        if let Some(y) = std::env::var("VC_HELP_SCROLL").ok().filter(|_| std::env::var("VC_SHOT").is_ok()).and_then(|v| v.parse::<f32>().ok()) { area = area.vertical_scroll_offset(y); }
        area.show(ui, |ui| {
            if !self.help.search.trim().is_empty() && !self.help.results.is_empty() {
                for (id, title, snip) in &self.help.results { if ui.link(RichText::new(title).strong()).clicked() { open = Some(id.clone()); } ui.label(RichText::new(snip).weak().small()); ui.add_space(3.0); }
            } else if let Some((_, md)) = &self.help.doc {
                // Bilder liegen online: vor dem ersten Laden fragen (nichts geht ohne Zustimmung ins Netz)
                if self.help.images.needs_prompt(md) { let host = self.help.images.base_host(); match help_consent_box(ui, &host) { 1 => consent = Some(None), 2 => decline = true, 3 => consent = Some(Some(vc_ide::help_images::Consent::Always)), _ => {} } }
                let imgs: &mut dyn crate::markdown::Images = &mut self.help.images;
                if let Some(id) = crate::markdown::render_with(ui, md, Some(imgs)) { open = Some(id); }
            }
            else {
                ui.label(RichText::new("Hilfe").strong().size(18.0));
                let mut group = String::new();
                for (id, g, title) in &self.help.topics { if *g != group { group = g.clone(); ui.add_space(6.0); ui.label(RichText::new(g).strong()); } if ui.link(title).clicked() { open = Some(id.clone()); } }
            }
        });
        self.help.images.zoom_ui(ui.ctx());
        match consent { Some(None) => self.help.images.answer_session(true), Some(Some(c)) => self.set_help_images(c), None => {} }
        if decline { self.help.images.answer_session(false); }
        if let Some(k) = open { self.help.search.clear(); self.help.results.clear(); self.help_open(&k); }
    }

    fn ref_ui(&mut self, ui: &mut egui::Ui) {
        if self.help.ref_word != self.active_word { self.help.ref_word = self.active_word.clone(); self.help.ref_doc = if self.active_word.is_empty() { None } else { self.help_lookup(&self.active_word) }; }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            match self.help.ref_doc.clone() { Some((_, md)) => { if let Some(id) = crate::markdown::render(ui, &md) { self.help_open(&id); self.help.want_tab = true; } } None => { ui.weak("Cursor auf einen Befehl, eine Direktive, ein Register oder ein BASIC-Wort setzen."); } }
        });
    }

    /// Reiter „Live-Bild“: die Ultimate 64 wählen (die unter „Läuft auf“ gewählte, sonst die erste eingetragene) und zeichnen
    fn live_ui(&mut self, ui: &mut egui::Ui) {
        let set = vc_ide::settings::effective();
        let is_u = |m: &vc_ide::settings::Machine| m.kind == "ultimate64" && !m.host.trim().is_empty();
        let m = set.machines.iter().find(|m| is_u(m) && m.id == self.run_on).or_else(|| set.machines.iter().find(|m| is_u(m)));
        let env = crate::live::Env {
            machine: m.map(|m| crate::live::MachineInfo { name: if m.name.is_empty() { "Ultimate 64".into() } else { m.name.clone() }, host: m.host.trim().to_string(), password: m.password.clone() }),
            opts: vc_ide::streams::StreamOptions::from_settings(&set),
            root: self.project.as_ref().map(|p| p.root.clone()),
        };
        self.live.ui(ui, &env);
    }

    fn memmap_ui(&mut self, ui: &mut egui::Ui) {
        if crate::memmap::tabs(ui) {
            let (tid, i) = (self.help_target(), self.info.as_ref());
            let input = crate::memmap::Input { data: self.data.as_deref(), target: &tid, artifact: i.map(|i| i.artifact.as_path()), segments: i.map_or(&[][..], |i| &i.segments), symbols: i.map(|i| &i.symbols), lines: i.map_or(&[][..], |i| &i.lines) };
            crate::memmap::show(ui, &input);
            return;
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            self.memmap_program(ui);
            ui.add_space(10.0); ui.separator();
            self.memmap_symbols(ui);
        });
    }

    /// Symbole der Systembibliothek (data/sys) des Ziels: nach Abschnitten gruppiert, mit Filter; Klick öffnet die Referenz
    fn memmap_symbols(&mut self, ui: &mut egui::Ui) {
        let Some(data) = self.data.clone() else { return };
        let target = self.help_target();
        let id = egui::Id::new("memmap_syms");
        let cached: Option<std::sync::Arc<(String, Vec<vc_ide::sysmap::Sym>)>> = ui.data(|d| d.get_temp(id));
        let cache = match cached { Some(c) if c.0 == target => c, _ => { let c = std::sync::Arc::new((target.clone(), vc_ide::sysmap::symbols(&data, &target))); ui.data_mut(|d| d.insert_temp(id, c.clone())); c } };
        let fid = egui::Id::new("memmap_filter");
        let mut filter: String = ui.data(|d| d.get_temp(fid)).unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Symbole ({})", cache.1.len())).strong());
            ui.add(egui::TextEdit::singleline(&mut filter).hint_text("Name, Adresse oder Text filtern").desired_width(ui.available_width()));
        });
        ui.data_mut(|d| d.insert_temp(fid, filter.clone()));
        if cache.1.is_empty() { ui.weak("Keine Systembibliothek für dieses Ziel (data/sys)."); return; }
        let q = filter.trim().to_lowercase();
        let q_addr = q.trim_start_matches('$').to_string();
        let matches = |s: &vc_ide::sysmap::Sym| q.is_empty() || s.name.to_lowercase().contains(&q) || s.doc.to_lowercase().contains(&q) || format!("{:04x}", s.addr).contains(&q_addr);
        let mut open: Option<String> = None;
        let mut sections: Vec<String> = vec![];
        for s in cache.1.iter() { if !sections.contains(&s.section) { sections.push(s.section.clone()); } }
        for sec in sections {
            let rows: Vec<&vc_ide::sysmap::Sym> = cache.1.iter().filter(|s| s.section == sec && matches(s)).collect();
            if rows.is_empty() { continue; }
            let title = if sec.is_empty() { "Allgemein".to_string() } else { sec.clone() };
            egui::CollapsingHeader::new(RichText::new(format!("{title} ({})", rows.len())).strong()).id_salt(("memmap_sec", &sec)).open(if q.is_empty() { None } else { Some(true) }).show(ui, |ui| {
                for s in rows {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("${:04X}", s.addr)).monospace().weak());
                        if ui.link(RichText::new(&s.name).monospace()).on_hover_text(&s.doc).clicked() { open = Some(s.name.clone()); }
                        if !s.doc.is_empty() { ui.label(RichText::new(&s.doc).small().weak()); }
                    });
                }
            });
        }
        if let Some(k) = open { self.help_open(&k); self.help.want_tab = true; }
    }

    fn memmap_program(&mut self, ui: &mut egui::Ui) {
        let Some(info) = &self.info else { ui.weak("Bauen, um die Belegung des Programms zu sehen."); return };
        ui.label(RichText::new(format!("{} ({} Segmente)", info.target, info.segments.len())).strong());
        let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
        let p = ui.painter_at(r); let x = |a: u32| r.left() + a as f32 / 65536.0 * r.width();
        for (name, a, b, kind) in &info.regions { if *a == 0 && *b == 0xFFFF && info.regions.len() > 1 { continue; } let c = match kind.as_str() { "rom" => Color32::from_rgb(0x6b, 0x5a, 0x3a), "io" => Color32::from_rgb(0x8d, 0x57, 0xa8), _ => Color32::from_rgb(0x3b, 0x6a, 0xa8) }; p.rect_filled(egui::Rect::from_min_max(egui::pos2(x(*a), r.top()), egui::pos2(x(*b + 1).max(x(*a) + 1.0), r.bottom())), 0.0, c); let _ = name; }
        for (a, l) in &info.segments { p.rect_filled(egui::Rect::from_min_max(egui::pos2(x(*a), r.top() + 6.0), egui::pos2(x(*a + *l as u32).max(x(*a) + 2.0), r.bottom() - 6.0)), 0.0, crate::theme::warn()); }
        ui.horizontal(|ui| { for (c, n) in [(Color32::from_rgb(0x3b, 0x6a, 0xa8), "RAM"), (Color32::from_rgb(0x6b, 0x5a, 0x3a), "ROM"), (Color32::from_rgb(0x8d, 0x57, 0xa8), "E/A"), (crate::theme::warn(), "Programm")] { let (b, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover()); ui.painter().rect_filled(b, 2.0, c); ui.label(RichText::new(n).small()); } });
        if !info.segments.is_empty() { ui.add_space(6.0); egui::Grid::new("segs").striped(true).show(ui, |ui| { ui.label("Start"); ui.label("Ende"); ui.label("Bytes"); ui.end_row(); for (a, l) in &info.segments { ui.label(format!("${a:04X}")); ui.label(format!("${:04X}", *a as usize + l - 1)); ui.label(format!("{l}")); ui.end_row(); } }); }
        ui.add_space(6.0); ui.label(RichText::new("Bereiche").strong());
        for (name, a, b, kind) in &info.regions { ui.label(RichText::new(format!("${a:04X}–${b:04X}  {name} ({kind})")).small().weak()); }
    }

    fn debug_ui(&mut self, ui: &mut egui::Ui, tab: &Tab) {
        let Some(d) = &self.dbg else { ui.weak("Keine Debug-Sitzung. „Debuggen“ (F6) baut und startet den Emulator."); return };
        let sn = d.snapshot(); let mono = egui::TextStyle::Monospace;
        if *tab == Tab::Regs {
            let src = if sn.have_regs && !sn.running { self.info.as_ref().and_then(|i| i.line_at(sn.pc)).map(|(f, n)| (f.to_string(), n)) } else { None };
            crate::debugger::status_chip(ui, &sn, src);
            crate::debugger::caps_row(ui, &sn);
            ui.add_space(6.0);
        }
        if sn.starting { if *tab != Tab::Regs { ui.label("Emulator startet …"); } return; }
        if sn.running && *tab != Tab::Breaks && *tab != Tab::Watch && *tab != Tab::Snapshot { ui.weak("Der Emulator läuft. Anhalten mit „Pause“ oder durch einen Haltepunkt."); return; }
        match tab {
            Tab::Regs => {
                let tiles = crate::debugger::reg_tiles(ui, &sn);
                for (n, v, r) in tiles {
                    let w = if n == "PC" { 4 } else { 2 };
                    r.context_menu(|ui| {
                        if ui.button("Wert kopieren (hex)").clicked() { ui.ctx().copy_text(format!("${:0w$X}", v, w = w)); ui.close(); }
                        if ui.button("Wert kopieren (dezimal)").clicked() { ui.ctx().copy_text(v.to_string()); ui.close(); }
                        if ui.button(format!("Beobachten: {}", n.to_lowercase())).clicked() { self.watch_exprs.push(n.to_lowercase()); d.send(DbgCmd::Watches(self.watch_exprs.clone())); self.show_tab = Some(Tab::Watch); ui.close(); }
                        if n == "PC" || n == "SP" {
                            let a = if n == "PC" { sn.pc } else { 0x0100 + sn.sp as u16 };
                            if ui.button(format!("Speicher ab ${a:04X} anzeigen")).clicked() { d.send(DbgCmd::Mem(a)); self.mem_input = format!("{a:04X}"); self.show_tab = Some(Tab::Memory); ui.close(); }
                        }
                    });
                }
                ui.add_space(4.0);
                crate::debugger::flag_pills(ui, &sn);
                if let Some(e) = &sn.cond_error { ui.colored_label(PAL_ERR, e); }
                if let Some(w) = &sn.step_block { ui.add(egui::Label::new(RichText::new(format!("Einzelschritt gesperrt: {w}")).color(crate::theme::warn())).wrap()); }
                if let Some(e) = &sn.error { ui.colored_label(PAL_ERR, format!("Letzter Fehler: {e}")); }
                ui.add_space(8.0);
                let ids = self.data.as_ref().and_then(|dd| vc_target::load_target(dd, &self.help_target()).ok()).map(|t| t.chips).unwrap_or_default();
                crate::debugger::chip_panel(ui, &sn, d, self.data.as_deref(), &ids);
            }
            Tab::Disasm => {
                crate::debugger::disasm_header(ui);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for l in &sn.disasm {
                        let bytes = l.bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
                        let lab = self.info.as_ref().and_then(|i| i.labels.get(&l.addr)).map(|n| format!("{n}:")).unwrap_or_default();
                        let text = format!("{:04X}  {:<9} {:<10}{}{}", l.addr, bytes, lab, l.text, if l.illegal { "  (undokumentiert)" } else { "" });
                        let cur = l.addr == sn.pc; let bp = sn.breakpoints.contains(&l.addr);
                        let r = crate::debugger::disasm_row(ui, l, lab.trim_end_matches(':'), cur, bp);
                        if r.clicked() { self.disasm_toggle = Some(l.addr); }
                        r.context_menu(|ui| {
                            let a = l.addr;
                            let src = self.info.as_ref().and_then(|i| i.line_at(a)).map(|(f, n)| (f.to_string(), n));
                            let exec_pt = sn.points.iter().find(|p| p.kind == "exec" && p.start == a && p.end == a).map(|p| p.id);
                            if ui.button(if bp || exec_pt.is_some() { "Haltepunkt entfernen" } else { "Haltepunkt setzen" }).clicked() {
                                if bp { self.disasm_toggle = Some(a); }
                                else if let Some(id) = exec_pt { d.send(DbgCmd::DelPoint(id)); }
                                else if src.is_some() { self.disasm_toggle = Some(a); }
                                else { d.send(DbgCmd::AddPoint { kind: "exec".into(), start: a, end: a, cond: None, ignore: 0 }); }
                                ui.close();
                            }
                            if ui.button("Bis hierher ausführen").clicked() {
                                d.send(DbgCmd::AddPoint { kind: "exec".into(), start: a, end: a, cond: None, ignore: 0 }); d.send(DbgCmd::Cont);
                                self.run_to = Some((a, false)); ui.close();
                            }
                            ui.separator();
                            if ui.button(format!("Speicher ab ${a:04X} anzeigen")).clicked() { d.send(DbgCmd::Mem(a)); self.mem_input = format!("{a:04X}"); self.show_tab = Some(Tab::Memory); ui.close(); }
                            if ui.button(format!("Beobachten: [${a:04X}]")).clicked() { self.watch_exprs.push(format!("[${a:04X}]")); d.send(DbgCmd::Watches(self.watch_exprs.clone())); self.show_tab = Some(Tab::Watch); ui.close(); }
                            if let Some((f, n)) = &src { if ui.button(format!("Zur Quellzeile ({f}:{n})")).clicked() { self.open_request = Some(f.clone()); self.pending_reveal = Some((f.clone(), n - 1)); ui.close(); } }
                            ui.separator();
                            if ui.button("Adresse kopieren").clicked() { ui.ctx().copy_text(format!("${a:04X}")); ui.close(); }
                            if ui.button("Zeile kopieren").clicked() { ui.ctx().copy_text(text.clone()); ui.close(); }
                        });
                    }
                });
            }
            Tab::Snapshot => {
                let sys = if self.help_target() == "atari8" { "atari8" } else { "c64" }.to_string();
                ui.label(RichText::new("Momentaufnahme: Bildschirm, Zeichensatz und Sprites aus dem laufenden Programm sichern").strong());
                if sn.running {
                    self.snap_req = None;
                    ui.add_space(4.0);
                    if sn.can_pause { ui.label("Der Rechner läuft. Halte ihn an der Stelle an, die du festhalten willst (z. B. wenn das Spiel das gewünschte Bild zeigt)."); } else { ui.label("Der Rechner läuft."); }
                    if sn.can_pause { if ui.button("⏸ Jetzt anhalten").clicked() { d.send(DbgCmd::Pause); } }
                    else { ui.label(RichText::new("Dieser Emulator lässt sich nicht per Knopf anhalten. Setze im Quelltext einen Haltepunkt (Klick neben die Zeilennummer) an der Stelle, die du festhalten willst, und starte mit „Debuggen“ (F6) neu – oder drücke im Emulatorfenster F8.").strong()); }
                    return;
                }
                if self.snap_req != Some(sn.pc) { self.snap_req = Some(sn.pc); d.send(DbgCmd::SnapDetect(sys.clone(), self.data.clone().unwrap_or_default())); }
                ui.horizontal(|ui| {
                    ui.weak("Angehalten bei"); ui.label(RichText::new(format!("${:04X}", sn.pc)).text_style(mono.clone()));
                    if ui.button("↻ Neu erkennen").on_hover_text("liest die Hardware-Register erneut aus (verwirft deine Änderungen an den Adressen)").clicked() { self.snap_form.for_det = None; d.send(DbgCmd::SnapDetect(sys.clone(), self.data.clone().unwrap_or_default())); }
                });
                match &sn.detected {
                    None => { ui.weak("Erkenne …"); }
                    Some(Err(e)) => { ui.colored_label(PAL_ERR, e); }
                    Some(Ok(det)) => {
                      ui.horizontal_top(|ui| {
                        if let Some(pv) = &det.preview {
                            let key = std::sync::Arc::as_ptr(&pv.rgba) as usize;
                            if self.snap_tex.as_ref().map_or(true, |t| t.0 != key) {
                                let img = egui::ColorImage::from_rgba_unmultiplied([pv.w, pv.h], &pv.rgba);
                                self.snap_tex = Some((key, ui.ctx().load_texture("snapshot-preview", img, egui::TextureOptions::NEAREST)));
                            }
                            if let Some((_, t)) = &self.snap_tex { ui.vertical(|ui| { ui.image((t.id(), egui::vec2(pv.w as f32, pv.h as f32))); ui.label(RichText::new("Vorschau des Bildschirms (ohne Rahmen und Sprites)").weak().small()); }); }
                        } else { ui.vertical(|ui| { ui.weak("Keine Vorschau\n(Zeichensatz nicht lesbar\noder kein Textmodus)"); }); }
                        ui.vertical(|ui| {
                        let f = &mut self.snap_form;
                        if f.for_det != Some((det.screen, det.charset)) { f.for_det = Some((det.screen, det.charset)); f.screen = format!("{:04X}", det.screen); f.charset = det.charset.map(|c| format!("{c:04X}")).unwrap_or_default(); f.use_screen = det.supported; f.use_charset = det.charset.is_some() || det.rom_off.is_some(); }
                        ui.label(RichText::new(format!("Erkannt: {}", det.mode)));
                        for n in &det.notes { ui.label(RichText::new(n).weak().small()); }
                        ui.label(RichText::new(if det.system == "c64" { format!("Farben werden mitgespeichert: Rahmen {}, Hintergrund {}, Farb-RAM jeder Zelle, Sprite- und Multicolor-Farben.", det.border, det.bg[0]) } else { format!("Farben werden mitgespeichert: Rahmen ${:02X}, Hintergrund ${:02X}, Schrift ${:02X}.", det.border, det.bg[0], det.fg) }).weak().small());
                        ui.add_space(4.0);
                        let hex = |t: &str| u16::from_str_radix(t.trim().trim_start_matches('$'), 16).ok();
                        egui::Grid::new("snapgrid").num_columns(3).spacing([10.0, 6.0]).show(ui, |ui| {
                            ui.add_enabled(det.supported, egui::Checkbox::new(&mut f.use_screen, "Bildschirm")); ui.horizontal(|ui| { ui.label("ab $"); ui.add_enabled_ui(det.supported, |ui| ui.add_sized([70.0, 20.0], egui::TextEdit::singleline(&mut f.screen).font(mono.clone()))); });
                            ui.weak(if det.supported { "Zeichen und Farben des Textbildschirms" } else { "in diesem Grafikmodus nicht möglich" }); ui.end_row();
                            ui.checkbox(&mut f.use_charset, "Zeichensatz");
                            if det.rom_off.is_some() { ui.horizontal(|ui| { ui.label("Zeichen-ROM"); ui.weak("oder ab $"); ui.add_sized([70.0, 20.0], egui::TextEdit::singleline(&mut f.charset).font(mono.clone()).hint_text("—")); }); }
                            else { ui.horizontal(|ui| { ui.label("ab $"); ui.add_sized([70.0, 20.0], egui::TextEdit::singleline(&mut f.charset).font(mono.clone()).hint_text("—")); }); }
                            ui.weak(if det.rom_off.is_some() { "Standardschrift, wird aus dem Zeichen-ROM gelesen. Eine Adresse ist nur nötig, wenn das Programm den Zeichensatz stattdessen im RAM hat." } else { "die Zeichenformen, die gerade angezeigt werden" }); ui.end_row();
                            ui.add_enabled(det.supported && f.use_screen, egui::Checkbox::new(&mut f.tiles, "Kacheln 2×2")); ui.label(""); ui.weak("zusätzlich eine Kachelkarte aus dem Bildschirm (gleiche Zeichen und Farben = eine Kachel)"); ui.end_row();
                            if det.sprites.is_empty() { ui.weak(if det.system == "atari8" { "Player/Missiles" } else { "Sprites" }); ui.label(""); ui.weak(if det.system == "atari8" { "werden noch nicht ausgelesen" } else { "keine aktiv" }); ui.end_row(); }
                            if !det.sprites.is_empty() {
                                ui.checkbox(&mut f.sprites, format!("{} Sprite(s)", det.sprites.len())); ui.label(""); ui.weak(det.sprites.iter().map(|s| format!("#{} ${:04X}", s.index, s.addr)).collect::<Vec<_>>().join("   ")); ui.end_row();
                            }
                        });
                        let (bad_s, bad_c) = (f.use_screen && det.supported && hex(&f.screen).is_none(), f.use_charset && (!f.charset.trim().is_empty() || det.rom_off.is_none()) && hex(&f.charset).is_none());
                        if bad_s || bad_c { ui.colored_label(PAL_ERR, "Adresse als Hex-Zahl eintragen, z. B. 0400"); }
                        ui.add_space(4.0);
                        ui.horizontal(|ui| { ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(120.0)); ui.weak(format!("Dateien in graphics/snapshot/: {n}_screen, {n}_charset, {n}_sprite0 …", n = f.name)); });
                        let name_ok = !f.name.is_empty() && f.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                        if !name_ok { ui.colored_label(PAL_ERR, "Name: nur Buchstaben, Ziffern, _ und -"); }
                        let (scr, chs) = (if f.use_screen && det.supported { hex(&f.screen) } else { None }, if f.use_charset { hex(&f.charset) } else { None });
                        let rom = f.use_charset && chs.is_none() && det.rom_off.is_some();
                        let ok = name_ok && !bad_s && !bad_c && (scr.is_some() || chs.is_some() || rom || (f.sprites && !det.sprites.is_empty()));
                        if ui.add_enabled(ok, egui::Button::new(RichText::new("Speichern").strong())).on_disabled_hover_text("Wähle mindestens etwas aus und prüfe die Adressen").clicked() {
                            if let (Some(root), Some(data)) = (self.project.as_ref().map(|p| p.root.clone()), self.data.clone()) {
                                d.send(DbgCmd::SnapSave { system: sys.clone(), det: det.clone(), opts: crate::snapshot::Opts { screen: scr, charset: chs, rom, sprites: f.sprites && !det.sprites.is_empty(), tiles: f.tiles && scr.is_some(), name: f.name.clone() }, root, data });
                            }
                        }
                        ui.label(RichText::new("Alles wird gespeichert und sofort in den Editoren geöffnet. Bereits vorhandene mit gleichem Namen werden ersetzt.").weak().small());
                        });
                      });
                    }
                }
            }
            Tab::Memory => {
                ui.horizontal(|ui| {
                    ui.label("Adresse $"); let r = ui.add(egui::TextEdit::singleline(&mut self.mem_input).desired_width(60.0).font(mono.clone()));
                    if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Lesen").clicked() { if let Ok(a) = u16::from_str_radix(self.mem_input.trim().trim_start_matches('$'), 16) { d.send(DbgCmd::Mem(a)); } }
                    if ui.button("−256").clicked() { d.send(DbgCmd::Mem(sn.mem_addr.wrapping_sub(256))); self.mem_input = format!("{:04X}", sn.mem_addr.wrapping_sub(256)); }
                    if ui.button("+256").clicked() { d.send(DbgCmd::Mem(sn.mem_addr.wrapping_add(256))); self.mem_input = format!("{:04X}", sn.mem_addr.wrapping_add(256)); }
                });
                egui::CollapsingHeader::new("Bereich als Zeichensatz speichern").id_salt("dumpcs").show(ui, |ui| {
                    let sys = self.project.as_ref().map(|p| vc_media::project::project_systems(&p.root, self.data.as_deref().unwrap_or(std::path::Path::new("")))).unwrap_or_default();
                    let profs: Vec<(String, String)> = self.data.as_ref().and_then(|d| vc_media::load_profiles(d).ok()).unwrap_or_default().iter().filter(|p| p.kind == "charset" && p.bpp <= 2 && (sys.is_empty() || sys.contains(&p.target))).map(|p| (p.id.clone(), p.name.clone())).collect();
                    if profs.is_empty() { ui.weak("Kein passendes Zeichensatz-Profil in diesem Projekt."); return; }
                    let f = &mut self.dump_form; if f.0.is_empty() { f.0 = self.mem_input.clone(); } f.1 = f.1.min(profs.len() - 1);
                    ui.horizontal(|ui| { ui.label("ab $"); ui.add(egui::TextEdit::singleline(&mut f.0).desired_width(60.0).font(mono.clone())); ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut f.2).desired_width(150.0)); });
                    for (i, (_, n)) in profs.iter().enumerate() { ui.radio_value(&mut f.1, i, n.as_str()); }
                    let ok = !f.2.is_empty() && f.2.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') && u16::from_str_radix(f.0.trim().trim_start_matches('$'), 16).is_ok();
                    if ui.add_enabled(ok, egui::Button::new("Speichern")).on_hover_text("liest 2048 (C64) bzw. 1024 (Atari) Bytes aus dem Speicher des Rechners und legt graphics/dump/NAME an").clicked() {
                        if let (Some(root), Some(data)) = (self.project.as_ref().map(|p| p.root.clone()), self.data.clone()) { d.send(DbgCmd::SaveCharset { addr: u16::from_str_radix(f.0.trim().trim_start_matches('$'), 16).unwrap_or(0), profile: profs[f.1].0.clone(), name: f.2.clone(), root, data }); }
                    }
                    ui.label(RichText::new("Hinweis: Gelesen wird die Sicht der CPU. Das Zeichen-ROM des C64 liegt nur bei abgeschaltetem E/A-Bereich an $D000 und ist über die Speicherumschaltung zu erreichen; Zeichensätze im RAM (z. B. eines laufenden Spiels) liest du direkt.").weak().small());
                });
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for (row, chunk) in sn.mem.chunks(16).enumerate() {
                        let asc: String = chunk.iter().map(|b| if (0x20..0x7f).contains(b) { *b as char } else { '.' }).collect();
                        let addr = sn.mem_addr.wrapping_add((row * 16) as u16);
                        let line = format!("{:04X}  {}  {}", addr, chunk.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "), asc);
                        let r = ui.add(egui::Label::new(crate::debugger::mem_job(addr, chunk, &sn.mem_prev, row)).sense(egui::Sense::click()));
                        r.context_menu(|ui| {
                            if ui.button(format!("Beobachten: [${addr:04X}]")).clicked() { self.watch_exprs.push(format!("[${addr:04X}]")); d.send(DbgCmd::Watches(self.watch_exprs.clone())); self.show_tab = Some(Tab::Watch); ui.close(); }
                            if ui.button(format!("Haltepunkt: Schreiben auf ${addr:04X}")).clicked() { d.send(DbgCmd::AddPoint { kind: "write".into(), start: addr, end: addr, cond: None, ignore: 0 }); ui.close(); }
                            if ui.button(format!("Haltepunkt: Lesen von ${addr:04X}")).clicked() { d.send(DbgCmd::AddPoint { kind: "read".into(), start: addr, end: addr, cond: None, ignore: 0 }); ui.close(); }
                            ui.separator();
                            if ui.button("Adresse kopieren").clicked() { ui.ctx().copy_text(format!("${addr:04X}")); ui.close(); }
                            if ui.button("Zeile kopieren").clicked() { ui.ctx().copy_text(line.clone()); ui.close(); }
                        });
                    }
                });
            }
            Tab::Breaks => {
                ui.label(RichText::new("Haltepunkte").strong());
                if sn.points.is_empty() { ui.weak("Keine. Im Editor-Seitenstreifen klicken oder unten einen anlegen."); }
                for p in &sn.points { ui.horizontal(|ui| {
                    let k = match p.kind.as_str() { "exec" => "Ausführung", "read" => "Lesen", "write" => "Schreiben", _ => "Zugriff" };
                    let range = if p.start == p.end { format!("${:04X}", p.start) } else { format!("${:04X}–${:04X}", p.start, p.end) };
                    let rp = ui.add(egui::Label::new(RichText::new(format!("{k} {range}")).text_style(mono.clone())).sense(egui::Sense::click()));
                    rp.context_menu(|ui| {
                        if ui.button("Entfernen").clicked() { d.send(DbgCmd::DelPoint(p.id)); ui.close(); }
                        if ui.button("Bearbeiten (ins Formular übernehmen)").clicked() {
                            self.bp_form = (["exec", "read", "write", "access"].iter().position(|x| *x == p.kind).unwrap_or(0), format!("{:04X}", p.start), format!("{:04X}", p.end), p.cond.clone().unwrap_or_default(), if p.ignore > 0 { p.ignore.to_string() } else { String::new() });
                            d.send(DbgCmd::DelPoint(p.id)); ui.close();
                        }
                        ui.separator();
                        if ui.button(format!("Speicher ab ${:04X} anzeigen", p.start)).clicked() { d.send(DbgCmd::Mem(p.start)); self.mem_input = format!("{:04X}", p.start); self.show_tab = Some(Tab::Memory); ui.close(); }
                        if p.kind == "exec" { if let Some((f, n)) = self.info.as_ref().and_then(|i| i.line_at(p.start)).map(|(f, n)| (f.to_string(), n)) { if ui.button(format!("Zur Quellzeile ({f}:{n})")).clicked() { self.open_request = Some(f.clone()); self.pending_reveal = Some((f, n - 1)); ui.close(); } } }
                        if ui.button("Adresse kopieren").clicked() { ui.ctx().copy_text(format!("${:04X}", p.start)); ui.close(); }
                    });
                    if let Some(n) = self.info.as_ref().and_then(|i| i.labels.get(&p.start)) { ui.label(n); }
                    if let Some(c) = &p.cond { ui.label(RichText::new(format!("wenn {c}")).weak()); }
                    if p.ignore > 0 { ui.label(RichText::new(format!("ignorieren {}", p.ignore)).weak()); }
                    ui.label(RichText::new(format!("{}×", p.hits)).weak());
                    if sn.stopped_at == Some(p.id) { ui.label(RichText::new("getroffen").color(PAL_ERR)); }
                    if ui.small_button("x").clicked() { d.send(DbgCmd::DelPoint(p.id)); }
                }); }
                ui.separator(); ui.label(RichText::new("Neu").strong());
                let f = &mut self.bp_form;
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("bpkind").selected_text(["Ausführung", "Lesen", "Schreiben", "Zugriff"][f.0]).show_ui(ui, |ui| { for (i, n) in ["Ausführung", "Lesen", "Schreiben", "Zugriff"].iter().enumerate() { let ok = i == 0 || sn.caps.watch; ui.add_enabled_ui(ok, |ui| { ui.selectable_value(&mut f.0, i, *n); }).response.on_disabled_hover_text(format!("{} unterstützt keine Datenhaltepunkte", sn.backend)); } });
                    if !sn.caps.watch { crate::debugger::unavailable(ui, "Datenhaltepunkte (Lesen/Schreiben) sind bei diesem Emulator nicht verfügbar."); }
                    ui.label("von $"); ui.add(egui::TextEdit::singleline(&mut f.1).desired_width(50.0).font(mono.clone()));
                    if f.0 != 0 { ui.label("bis $"); ui.add(egui::TextEdit::singleline(&mut f.2).desired_width(50.0).font(mono.clone())); }
                    ui.label("Bedingung"); ui.add(egui::TextEdit::singleline(&mut f.3).desired_width(150.0).hint_text("z. B. a == $80 oder [$d020] & 15 == 9").font(mono.clone()));
                    ui.label("Ignorieren"); ui.add(egui::TextEdit::singleline(&mut f.4).desired_width(36.0));
                    if ui.button("Anlegen").clicked() {
                        let hex = |s: &str| u16::from_str_radix(s.trim().trim_start_matches('$'), 16).ok();
                        let label_or_hex = |s: &str| hex(s).or_else(|| self.info.as_ref().and_then(|i| i.symbols.get(s.trim()).map(|v| *v as u16)));
                        if let Some(a) = label_or_hex(&f.1) {
                            let end = if f.0 == 0 { a } else { label_or_hex(&f.2).unwrap_or(a) };
                            d.send(DbgCmd::AddPoint { kind: ["exec", "read", "write", "access"][f.0].into(), start: a, end, cond: Some(f.3.trim().to_string()).filter(|c| !c.is_empty()), ignore: f.4.trim().parse().unwrap_or(0) });
                        }
                    }
                });
            }
            Tab::Watch => {
                ui.label(RichText::new("Beobachten").strong());
                ui.horizontal(|ui| {
                    let r = ui.add(egui::TextEdit::singleline(&mut self.watch_input).desired_width(ui.available_width() - 70.0).hint_text("Ausdruck, z. B. a, x + 1, [$d020], [sprite_x]").font(mono.clone()));
                    if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || ui.button("Hinzufügen").clicked()) && !self.watch_input.trim().is_empty() { self.watch_exprs.push(self.watch_input.trim().to_string()); self.watch_input.clear(); d.send(DbgCmd::Watches(self.watch_exprs.clone())); }
                });
                if self.watch_exprs.is_empty() { ui.weak("Ausdrücke werden bei jedem Halt neu ausgewertet."); }
                let mut del = None; let mut del_all = false;
                egui::Grid::new("watchg").striped(true).show(ui, |ui| {
                    for (i, e) in self.watch_exprs.iter().enumerate() {
                        let value = sn.watch.iter().find(|(w, _)| w == e).map(|(_, r)| r.clone());
                        let r1 = ui.add(egui::Label::new(RichText::new(e).text_style(mono.clone())).sense(egui::Sense::click()));
                        let r2 = match &value { Some(Ok(v)) => ui.add(egui::Label::new(RichText::new(format!("${:X}  ({})", v, v)).text_style(mono.clone())).sense(egui::Sense::click())), Some(Err(m)) => ui.add(egui::Label::new(RichText::new(m).color(PAL_ERR)).sense(egui::Sense::click())), None => ui.add(egui::Label::new(RichText::new("…").weak()).sense(egui::Sense::click())) };
                        for r in [&r1, &r2] { r.context_menu(|ui| {
                            if let Some(Ok(v)) = &value {
                                if ui.button("Wert kopieren (hex)").clicked() { ui.ctx().copy_text(format!("${v:X}")); ui.close(); }
                                if ui.button("Wert kopieren (dezimal)").clicked() { ui.ctx().copy_text(v.to_string()); ui.close(); }
                            }
                            if ui.button("Ausdruck kopieren").clicked() { ui.ctx().copy_text(e.clone()); ui.close(); }
                            ui.separator();
                            if ui.button("Ausdruck entfernen").clicked() { del = Some(i); ui.close(); }
                            if ui.button("Alle entfernen").clicked() { del_all = true; ui.close(); }
                        }); }
                        if ui.small_button("x").clicked() { del = Some(i); } ui.end_row();
                    }
                });
                if let Some(i) = del { self.watch_exprs.remove(i); d.send(DbgCmd::Watches(self.watch_exprs.clone())); }
                if del_all { self.watch_exprs.clear(); d.send(DbgCmd::Watches(vec![])); }
            }
            _ => {}
        }
    }

    fn log(&mut self, l: Level, s: impl Into<String>) { let s = s.into(); vc_ide::debuglog::log(match l { Level::Err => "fehler", Level::Warn => "warnung", _ => "ausgabe" }, &s); self.output.push((l, s)); }

    /// Name des Emulators für `system` und ob sein Programm gefunden wurde (alle 5 s neu geprüft)
    fn emulator_status(&mut self, system: &str) -> Option<(String, bool)> {
        if let Some((t, sy, name, found)) = &self.emu_cache { if sy == system && t.elapsed().as_secs() < 5 { return Some((name.clone(), *found)); } }
        let e = vc_ide::settings::effective().emulator_for(system);
        let found = vc_ide::platform::presence(&vc_ide::targets::EmulatorCfg { id: "status".into(), kind: e.kind.clone(), path: e.path.clone().unwrap_or_default(), ..Default::default() }).found;
        let name = vc_ide::targets::kind_label(&e.kind).to_string();
        self.emu_cache = Some((std::time::Instant::now(), system.to_string(), name.clone(), found));
        Some((name, found))
    }

    fn welcome_ui(&mut self, ui: &mut egui::Ui) {
        let a = crate::welcome::show(ui, &mut self.welcome, self.data.as_deref(), self.project.is_some());
        if a.is_some() { self.welcome_act = a; }
    }

    fn overview_ui(&mut self, ui: &mut egui::Ui) {
        let Some(pr) = &self.project else { ui.label("Kein Projekt geöffnet."); return };
        let log: Vec<(String, bool)> = self.buildlog.iter().map(|e| (e.file.clone(), e.ok)).collect();
        let problems: Vec<(String, bool)> = self.problems.iter().map(|p| (p.file.clone(), p.error)).collect();
        let segs: Option<Vec<(u32, usize)>> = self.info.as_ref().map(|i| i.segments.clone());
        let inp = crate::overview::Input { project: pr, segments: segs.as_deref(), log: &log, problems: &problems, building: self.building.is_some(), run_on: &self.run_on };
        crate::overview::show(ui, &mut self.ov, &inp);
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        if self.se.is_none() { self.se = Some(crate::settings_ui::SettingsEd::new(vc_ide::settings::get())); }
        let Some(ed) = &mut self.se else { return };
        ed.set_project(self.project.as_ref().map(|p| p.pf.as_ref().map(|pf| pf.overrides()).unwrap_or_default()));
        let project = self.project.as_ref().map(|p| (p.name.clone(), p.pf.as_ref().and_then(|pf| pf.extra.get("export")).and_then(|e| e["autoExport"].as_bool()).unwrap_or(true)));
        let mut env = crate::settings_ui::Env { view: &mut self.view, show_addr: &mut self.show_addr, data: self.data.as_deref(), project };
        let a = crate::settings_ui::show(ui, ed, &mut env);
        if a != crate::settings_ui::Act::None { self.se_act = a; }
        if let Some(sys) = ed.take_clear_override() { self.clear_project_override(&sys); }
    }

    /// Einträge der Befehlspalette: alle Menübefehle (Tabelle `nativemenu::COMMANDS`), Studios und die Dateien des Projekts
    fn palette_entries(&self, ms: crate::nativemenu::MenuState) -> Vec<crate::cmdpalette::Entry> {
        use crate::cmdpalette::{Entry, Item};
        let mac = cfg!(target_os = "macos");
        let mut v: Vec<Entry> = crate::cmdpalette::command_entries(&vc_ide::features::features(), &ms, mac);
        if let Some(pr) = &self.project {
            for e in &pr.entries { v.push(Entry { label: e.rel.clone(), group: format!("Datei · {}", e.group.title()), keys: String::new(), enabled: true, only_when_searching: true, item: Item::File(e.rel.clone()) }); }
        }
        v
    }

    /// „Überschreibung entfernen“ in den Einstellungen: löscht Standard-Emulator und Startziel des Systems aus der Projektdatei
    fn clear_project_override(&mut self, system: &str) {
        let Some(pr) = &mut self.project else { return };
        let (Some(file), Some(pf)) = (pr.file.clone(), pr.pf.as_mut()) else { return };
        if !pf.clear_override(system) { return; }
        let res = pf.save(&pr.root.join(&file));
        pr.reload();
        match res { Ok(()) => self.status = format!("Überschreibung für {system} entfernt"), Err(e) => self.log(Level::Err, e) }
        if let Some(pe) = &mut self.pe { pe.draft.clear_override(system); pe.saved.clear_override(system); }
    }

    fn projed_ui(&mut self, ui: &mut egui::Ui) {
        let Some(pr) = &self.project else { ui.label("Kein Projekt geöffnet."); return };
        if self.pe.is_none() { self.pe = Some(crate::projed::ProjEdit::new(pr.pf.clone().unwrap_or_else(|| { let mut pf = vc_ide::projfile::ProjectFile::default(); pf.project.name = pr.name.clone(); pf }))); }
        if self.known_targets.is_empty() {
            if let Some(d) = &self.data {
                let mut v: Vec<(String, String)> = std::fs::read_dir(d.join("targets")).into_iter().flatten().flatten().filter_map(|e| { let n = e.file_name().to_string_lossy().into_owned(); let id = n.strip_suffix(".target.json")?.to_string(); let cpu = vc_target::load_target(d, &id).ok()?.cpu; Some((id, cpu)) }).collect();
                v.sort(); self.known_targets = v;
            }
        }
        if self.known_dialects.is_empty() {
            if let Some(d) = &self.data { let mut v: Vec<String> = std::fs::read_dir(d.join("basic")).into_iter().flatten().flatten().filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".dialect.json").map(String::from)).collect(); v.sort(); self.known_dialects = v; }
            if let Some(d) = &self.data { self.known_structures = vc_ide::structure::list_all(d); }
        }
        let files: Vec<String> = pr.entries.iter().map(|e| e.rel.clone()).collect();
        let Some(ed) = &mut self.pe else { return };
        let a = crate::projed::show(ui, ed, &crate::projed::Env { targets: &self.known_targets, files: &files, dialects: &self.known_dialects, structures: &self.known_structures }, &pr.root);
        if a != crate::projed::Act::None { self.pe_act = a; }
    }

    /// Ordner des Projektbaums unter dem Zeiger: der innerste (kleinste Fläche), sonst keiner
    fn tree_dir_at(&self, p: egui::Pos2) -> Option<String> {
        self.tree_dirs.iter().filter(|(_, r)| r.contains(p)).min_by(|a, b| (a.1.width() * a.1.height()).total_cmp(&(b.1.width() * b.1.height()))).map(|(d, _)| d.clone())
    }

    fn project_ui(&mut self, ui: &mut egui::Ui) {
        let Some(pr) = &self.project else { ui.label("Kein Projekt geöffnet."); return };
        self.tree_rect = Some(ui.max_rect());
        self.tree_dirs.clear();
        ui.label(RichText::new(&pr.name).strong());
        if !pr.systems.is_empty() { ui.label(RichText::new(pr.systems.join(" · ").to_uppercase()).small().weak()); }
        let main = pr.pf.as_ref().and_then(|pf| pf.main_file());
        match &main { Some(m) => { ui.horizontal(|ui| { ui.label(RichText::new("★").color(crate::theme::warn())); ui.label(RichText::new(m).small()); }); } None => { ui.label(RichText::new("kein Startprogramm").small().weak()); } }
        ui.horizontal(|ui| { ui.selectable_value(&mut self.tree_mode, true, "Ordner"); ui.selectable_value(&mut self.tree_mode, false, "Nach Art"); });
        ui.add_space(2.0);
        let open_docs: HashSet<String> = self.docs.iter().map(|d| d.rel.clone()).collect();
        let dirty_docs: HashSet<String> = self.docs.iter().filter(|d| d.dirty()).map(|d| d.rel.clone()).collect();
        let mut clicked: Option<(String, Open)> = None; let mut set_main: Option<String> = None; let mut act: Option<TreeAct> = None; let mut dirs: Vec<(String, egui::Rect)> = vec![];
        let open_req = self.tree_open_req.take();
        let roles: std::collections::HashMap<String, (String, Vec<String>)> = pr.pf.as_ref().map(|pf| pf.folders.iter().filter(|f| !f.role.is_empty() || !f.targets.is_empty()).map(|f| (f.dir.clone(), (f.role.clone(), f.targets.clone()))).collect()).unwrap_or_default();
        let target_ids: Vec<String> = pr.pf.as_ref().map(|pf| pf.targets.iter().map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).collect()).unwrap_or_default();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if self.tree_mode {
                // Verzeichnisbaum: Ordner einklappbar, Dateien mit Kürzel der Art
                #[derive(Default)] struct Dir { dirs: std::collections::BTreeMap<String, Dir>, files: Vec<(String, String, Group)> }
                impl Dir { fn count(&self) -> usize { self.files.len() + self.dirs.values().map(Dir::count).sum::<usize>() } }
                let mut root = Dir::default();
                for dpath in &pr.dirs { let mut d = &mut root; for p in dpath.split('/') { d = d.dirs.entry(p.to_string()).or_default(); } }
                for e in &pr.entries {
                    let parts: Vec<&str> = e.rel.split('/').collect(); let mut d = &mut root;
                    for p in &parts[..parts.len() - 1] { d = d.dirs.entry(p.to_string()).or_default(); }
                    d.files.push((parts[parts.len() - 1].to_string(), e.rel.clone(), e.group));
                }
                fn draw(ui: &mut egui::Ui, d: &Dir, path: &str, open_docs: &HashSet<String>, dirty_docs: &HashSet<String>, clicked: &mut Option<(String, Open)>, main: &Option<String>, set_main: &mut Option<String>, act: &mut Option<TreeAct>, dirs: &mut Vec<(String, egui::Rect)>, roles: &std::collections::HashMap<String, (String, Vec<String>)>, target_ids: &[String], open_req: &Option<String>) {
                    for (name, sub) in &d.dirs {
                        let p = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
                        let title = {
                            let k = crate::ui::tokens::t();
                            let mut job = egui::text::LayoutJob::default();
                            job.append(&format!("{name}/"), 0.0, egui::text::TextFormat { font_id: egui::FontId::new(13.0, crate::fonts::ui_strong()), color: k.fg, ..Default::default() });
                            let (role, tg) = roles.get(&p).map_or((None, String::new()), |(r, t)| (vc_ide::projfile::role_label(r), t.iter().map(|x| x.to_uppercase()).collect::<Vec<_>>().join("+")));
                            let mut parts: Vec<String> = vec![];
                            if let Some(r) = role { parts.push(r.to_string()); }
                            if !tg.is_empty() { parts.push(format!("nur {tg}")); }
                            // Dateizahl nur an zugeklappten Ordnern (aufgeklappt sieht man die Dateien)
                            let is_open = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(("dir", &p)), path.is_empty() && name != "build").is_open();
                            if !is_open { parts.push(sub.count().to_string()); }
                            let tail = if parts.is_empty() { String::new() } else { format!("   {}", parts.join(" · ")) };
                            job.append(&tail, 0.0, egui::text::TextFormat { font_id: egui::FontId::proportional(10.5), color: k.dim, ..Default::default() });
                            job
                        };
                        let r = egui::CollapsingHeader::new(title).open(if open_req.as_deref() == Some(p.as_str()) { Some(true) } else { None }).id_salt(("dir", &p)).default_open(path.is_empty() && name != "build").show(ui, |ui| draw(ui, sub, &p, open_docs, dirty_docs, clicked, main, set_main, act, dirs, roles, target_ids, open_req));
                        dirs.push((p.clone(), r.header_response.rect));
                        if let Some(b) = &r.body_response { dirs.push((p.clone(), b.rect)); }
                        r.header_response.context_menu(|ui| {
                            if ui.button("Neue Datei hier …").clicked() { *act = Some(TreeAct::NewFile(p.clone())); ui.close(); }
                            if ui.button("Neuer Ordner hier …").clicked() { *act = Some(TreeAct::NewFolder(p.clone())); ui.close(); }
                            ui.separator();
                            ui.menu_button("Ordner-Rolle", |ui| {
                                let cur = roles.get(&p).map(|x| x.0.clone()).filter(|r| !r.is_empty());
                                if ui.selectable_label(cur.is_none(), "keine").clicked() { *act = Some(TreeAct::SetRole(p.clone(), None)); ui.close(); }
                                for (id, label) in vc_ide::projfile::ROLES { if ui.selectable_label(cur.as_deref() == Some(id), label).clicked() { *act = Some(TreeAct::SetRole(p.clone(), Some(id.to_string()))); ui.close(); } }
                            });
                            if !target_ids.is_empty() {
                                ui.menu_button("Nur für Target …", |ui| {
                                    let cur = roles.get(&p).and_then(|x| x.1.first().cloned());
                                    if ui.selectable_label(cur.is_none(), "alle Targets").clicked() { *act = Some(TreeAct::SetFolderTarget(p.clone(), None)); ui.close(); }
                                    for t in target_ids { if ui.selectable_label(cur.as_deref() == Some(t.as_str()), t.to_uppercase()).clicked() { *act = Some(TreeAct::SetFolderTarget(p.clone(), Some(t.clone()))); ui.close(); } }
                                });
                            }
                            ui.menu_button("Als Ziel festlegen für …", |ui| {
                                for c in vc_ide::importfiles::Category::CHOICES { if ui.button(c.label()).on_hover_text("Abgelegte Dateien dieser Kategorie kommen künftig in diesen Ordner (layout)").clicked() { *act = Some(TreeAct::SetTarget(p.clone(), c.id().to_string())); ui.close(); } }
                            });
                            ui.separator();
                            if ui.button("Im Finder zeigen").clicked() { *act = Some(TreeAct::Reveal(p.clone())); ui.close(); }
                            if ui.button("Pfad kopieren").clicked() { *act = Some(TreeAct::CopyPath(p.clone())); ui.close(); }
                        });
                    }
                    let mut files = d.files.clone(); files.sort_by_key(|f| f.0.to_lowercase());
                    for (name, rel, g) in files {
                        let is_main = main.as_deref() == Some(rel.as_str());
                        let resp = crate::shell::tree_row(ui, g, vc_media::naming::stem(&name), open_docs.contains(&rel), is_main);
                        resp.dnd_set_drag_payload(rel.clone());
                        dirs.push((path.to_string(), resp.rect));
                        crate::shell::dirty_dot(ui, &resp, dirty_docs.contains(&rel));
                        if let Some(h) = tree_click(ui, &resp) { *clicked = Some((rel.clone(), h)); }
                        file_menu(&resp, &rel, g, clicked, set_main, act);
                    }
                }
                draw(ui, &root, "", &open_docs, &dirty_docs, &mut clicked, &main, &mut set_main, &mut act, &mut dirs, &roles, &target_ids, &open_req);
            } else {
                for g in Group::ALL {
                    let items: Vec<&crate::project::Entry> = pr.entries.iter().filter(|e| e.group == g).collect();
                    if items.is_empty() { continue; }
                    egui::CollapsingHeader::new(RichText::new(format!("{}  ({})", g.title(), items.len())).strong()).default_open(true).show(ui, |ui| {
                        for e in items { let resp = crate::shell::tree_row(ui, e.group, vc_media::naming::stem(&e.rel), open_docs.contains(&e.rel), main.as_deref() == Some(e.rel.as_str())); resp.dnd_set_drag_payload(e.rel.clone()); crate::shell::dirty_dot(ui, &resp, dirty_docs.contains(&e.rel)); if let Some(h) = tree_click(ui, &resp) { clicked = Some((e.rel.clone(), h)); } file_menu(&resp, &e.rel, e.group, &mut clicked, &mut set_main, &mut act); }
                    });
                }
            }
        });
        self.tree_dirs = dirs;
        match clicked {
            Some((r, Open::Fixed)) => self.open_request = Some(r),
            Some((r, Open::Preview)) => self.preview_request = Some(r),
            Some((r, Open::Second)) => self.second_request = Some(r),
            None => {}
        }
        if let Some(m) = set_main { self.main_request = Some(m); }
        if act.is_some() { self.tree_act = act; }
    }
}

/// Aktionen des Projektbaum-Kontextmenüs (Pfade relativ zum Projekt)
#[derive(Clone, Debug)]
enum TreeAct { /** Ordner, Rolle (None = entfernen) */ SetRole(String, Option<String>), /** Ordner wird Zielordner dieser Kategorie */ SetTarget(String, String), /** Ordner nur für dieses Target (None = alle) */ SetFolderTarget(String, Option<String>), NewFolder(String), OpenImage(String), Rename(String), Duplicate(String), Delete(String), Reveal(String), CopyPath(String), NewFile(String), /** neue Mediendatei dieser Art (Seitenleiste ▸ +) */ NewMedia(Group), NewTest, BuildFile(String), ConvertBasic(String), OpenText(String), /** Include-Zeile (je nach Dateiart `.include`, `.incbin`, `#include`) in den aktiven Quelltext */ Include(String) }

/// Hinweisleiste zum Einbinden (`ok`: eingefügt, sonst abgelehnt)
struct DropNote { text: String, ok: bool, doc: String, at: usize, line: String, alt: Option<(String, String)>, t0: Option<f64> }

/// Aktionen des Reiter-Kontextmenüs
#[derive(Clone, Debug)]
enum TabAct { CloseOthers(String), CloseAll, Save(String), Discard(String), CopyPath(String), Reveal(String), Second(String), Unsplit }

/// Frage beim Umbenennen: Quelltexte, die die Datei per `.include`/`.incbin` einbinden, sollen den neuen Namen bekommen?
#[derive(Default)]
struct RenameAsk { for_name: String, texts: Vec<(String, String, usize)>, rewrite: bool }

/// Kleine Dialoge für Dateiaktionen
enum FileDlg { ConvertBasic { req: vc_ide::b2a::Req }, Rename { rel: String, name: String, ask: RenameAsk }, Delete { rel: String }, NewFile { dir: String, name: String }, NewFolder { dir: String, name: String }, CreateInclude { rel: String } }

/// Geste auf einer Zeile des Projektbaums: Einfachklick = Vorschau, Doppelklick = fester Reiter, Cmd/Strg+Klick = zweiter Editorbereich
fn tree_click(ui: &egui::Ui, resp: &egui::Response) -> Option<Open> {
    if resp.double_clicked() { Some(Open::Fixed) }
    else if resp.clicked() { Some(if ui.input(|i| i.modifiers.command) { Open::Second } else { Open::Preview }) }
    else { None }
}

/// Ist das die Projektdatei (`.vcprj`)? Sie öffnet in den Projekt-Einstellungen, nicht als Quelltext.
fn is_project_file(rel: &str) -> bool { rel.to_ascii_lowercase().ends_with(".vcprj") }

/// Kontextmenü einer Datei im Projektbaum
fn file_menu(resp: &egui::Response, rel: &str, g: Group, clicked: &mut Option<(String, Open)>, set_main: &mut Option<String>, act: &mut Option<TreeAct>) {
    let source = (g == Group::Asm || g == Group::Basic) && [".asm", ".sbas", ".bas", ".lst", ".tur"].iter().any(|e| rel.ends_with(e));
    resp.context_menu(|ui| {
        if vc_ide::diskimage::is_image_name(rel) && crate::studios::enabled("disk") {
            if ui.button("Diskettenimage öffnen …").clicked() { *act = Some(TreeAct::OpenImage(rel.to_string())); ui.close(); }
        } else {
            if ui.button("Öffnen").clicked() { *clicked = Some((rel.to_string(), Open::Fixed)); ui.close(); }
            if is_project_file(rel) { if ui.button("Als Text öffnen").on_hover_text("Die .vcprj als JSON-Quelltext ansehen").clicked() { *act = Some(TreeAct::OpenText(rel.to_string())); ui.close(); } }
            else if ui.button("Im zweiten Editorbereich öffnen").clicked() { *clicked = Some((rel.to_string(), Open::Second)); ui.close(); }
        }
        if !is_project_file(rel) && !vc_ide::diskimage::is_image_name(rel) {
            if ui.button("Als Include einfügen").on_hover_text("Fügt die passende Zeile (.include, .incbin oder #include) im aktiven Quelltext ein: unter dem Cursor. Per Maus: Datei in den Quelltext ziehen.").clicked() { *act = Some(TreeAct::Include(rel.to_string())); ui.close(); }
        }
        if source {
            if ui.button("★ Als Startprogramm festlegen").clicked() { *set_main = Some(rel.to_string()); ui.close(); }
            if ui.button("Nur diese Datei bauen").clicked() { *act = Some(TreeAct::BuildFile(rel.to_string())); ui.close(); }
            if rel.ends_with(".bas") || rel.ends_with(".lst") || rel.ends_with(".tur") {
                if ui.button("Nach Assembler konvertieren …").on_hover_text("BASIC in lesbaren Assembler-Quelltext übersetzen (basic2asm)").clicked() { *act = Some(TreeAct::ConvertBasic(rel.to_string())); ui.close(); }
            }
        }
        ui.separator();
        if ui.button("Umbenennen …").clicked() { *act = Some(TreeAct::Rename(rel.to_string())); ui.close(); }
        if ui.button("Duplizieren").clicked() { *act = Some(TreeAct::Duplicate(rel.to_string())); ui.close(); }
        if ui.button("Löschen …").clicked() { *act = Some(TreeAct::Delete(rel.to_string())); ui.close(); }
        ui.separator();
        if ui.button("Im Finder zeigen").clicked() { *act = Some(TreeAct::Reveal(rel.to_string())); ui.close(); }
        if ui.button("Pfad kopieren").clicked() { *act = Some(TreeAct::CopyPath(rel.to_string())); ui.close(); }
    });
}

struct Viewer<'a> { split: bool, st: &'a mut State, studios: &'a mut Vec<MediaWindow>, pal: &'a Palettes, audio: &'a crate::audio::Audio, reqs: &'a Requests, root: PathBuf, data: PathBuf }

impl TabViewer for Viewer<'_> {
    type Tab = Tab;
    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(match tab { Tab::Project => "project".to_string(), Tab::Overview => "overview".into(), Tab::Output => "output".into(), Tab::BuildLog => "buildlog".into(), Tab::Problems => "problems".into(), Tab::Tests => "tests".into(), Tab::Welcome => "welcome".into(), Tab::Regs => "regs".into(), Tab::Disasm => "disasm".into(), Tab::Memory => "memory".into(), Tab::Snapshot => "snapshot".into(), Tab::Breaks => "breaks".into(), Tab::MemMap => "memmap".into(), Tab::Ref => "ref".into(), Tab::Help => "help".into(), Tab::Watch => "watch".into(), Tab::ProjectEdit => "projectedit".into(), Tab::Settings => "settings".into(), Tab::Live => "live".into(), Tab::Emulator => "emulator".into(), Tab::Outline => "outline".into(), Tab::StudioInfo => "studioinfo".into(), Tab::Studio(g) => format!("studio:{g}"), Tab::Doc(r) => format!("doc:{r}") })
    }
    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::Project => "Projekt".into(), Tab::Overview => "Übersicht".into(), Tab::Output => "Ausgabe".into(), Tab::BuildLog => "Bauprotokoll".into(), Tab::Welcome => "Willkommen".into(), Tab::Regs => "Register".into(), Tab::Disasm => "Disassembler".into(), Tab::Memory => "Speicher".into(), Tab::Snapshot => "Momentaufnahme".into(), Tab::Breaks => "Haltepunkte".into(), Tab::MemMap => "Memory Map".into(), Tab::Ref => "Referenz".into(), Tab::Help => "Hilfe".into(), Tab::Watch => "Beobachten".into(), Tab::Settings => "Einstellungen".into(), Tab::Emulator => if self.st.emu.active() { "Emulator ●".into() } else { "Emulator".into() }, Tab::Live => if self.st.live.active() { "Live-Bild ●".into() } else { "Live-Bild".into() }, Tab::ProjectEdit => { let d = self.st.pe.as_ref().map_or(false, |e| e.dirty()); if d { "Projekt-Einstellungen *".into() } else { "Projekt-Einstellungen".into() } }
            Tab::Studio(g) => windows::window_title(windows::group_from_u8(*g)).into(),
            Tab::Outline => "Gliederung".into(), Tab::StudioInfo => "Studio".into(),
            Tab::Tests => { let c = self.st.tests.suite.counts(); ctx::titled("Tests", ctx::tests_badge(c.passed, c.total(), self.st.tests.failed_count())) }
            Tab::Problems => { let e = self.st.problems.iter().filter(|p| p.error).count(); ctx::titled("Probleme", ctx::problems_badge(e, self.st.problems.len() - e)) }
            Tab::Doc(r) => {
                // Dateiart-Marke (dezent), Name, Punkt bei ungespeicherten Änderungen
                let k = crate::ui::tokens::t();
                let dirty = self.st.docs.iter().find(|d| &d.rel == r).map_or(false, |d| d.dirty());
                let active = self.st.active_tabs.iter().any(|a| matches!(a, Tab::Doc(x) if x == r));
                let kind = self.st.project.as_ref().and_then(|p| p.entries.iter().find(|e| &e.rel == r)).map(|e| e.group.tag().to_string()).or_else(|| r.rsplit_once('.').map(|(_, e)| e.to_uppercase())).unwrap_or_default();
                let mut job = egui::text::LayoutJob::default();
                let font = egui::FontId::proportional(13.0);
                if !kind.is_empty() { job.append(&kind, 0.0, egui::TextFormat { font_id: egui::FontId::monospace(9.0), color: if active { k.cy.gamma_multiply(0.7) } else { k.dim }, valign: egui::Align::Center, ..Default::default() }); }
                job.append(r.rsplit('/').next().unwrap_or(r), if kind.is_empty() { 0.0 } else { 6.0 }, egui::TextFormat { font_id: font.clone(), color: if active { k.cy } else { k.mute }, italics: self.st.preview.as_deref() == Some(r.as_str()), valign: egui::Align::Center, ..Default::default() });
                if dirty { job.append("●", 6.0, egui::TextFormat { font_id: egui::FontId::proportional(10.0), color: k.am, valign: egui::Align::Center, ..Default::default() }); }
                job.into()
            }
        }
    }
    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Tab, _path: egui_dock::NodePath) {
        match tab {
            Tab::Doc(r) => {
                let dirty = self.st.docs.iter().find(|d| &d.rel == r).map_or(false, |d| d.dirty());
                let n_docs = self.st.docs.len();
                if ui.add_enabled(dirty, egui::Button::new("Speichern")).clicked() { self.st.tab_act = Some(TabAct::Save(r.clone())); ui.close(); }
                if ui.add_enabled(dirty, egui::Button::new("Änderungen verwerfen")).on_hover_text("lädt die Datei neu von der Platte").clicked() { self.st.tab_act = Some(TabAct::Discard(r.clone())); ui.close(); }
                ui.separator();
                if ui.add_enabled(n_docs > 1, egui::Button::new("Andere schließen")).on_hover_text("Dateien mit ungespeicherten Änderungen bleiben offen").clicked() { self.st.tab_act = Some(TabAct::CloseOthers(r.clone())); ui.close(); }
                if ui.button("Alle Quelltexte schließen").on_hover_text("Dateien mit ungespeicherten Änderungen bleiben offen").clicked() { self.st.tab_act = Some(TabAct::CloseAll); ui.close(); }
                ui.separator();
                if self.split { if ui.button("Teilung aufheben").clicked() { self.st.tab_act = Some(TabAct::Unsplit); ui.close(); } }
                else if ui.button("Editor teilen (Reiter nach rechts)").clicked() { self.st.tab_act = Some(TabAct::Second(r.clone())); ui.close(); }
                ui.separator();
                if ui.button("Pfad kopieren").clicked() { self.st.tab_act = Some(TabAct::CopyPath(r.clone())); ui.close(); }
                if ui.button("Im Finder zeigen").clicked() { self.st.tab_act = Some(TabAct::Reveal(r.clone())); ui.close(); }
            }
            Tab::Studio(g) => {
                if ui.button("In eigenem Fenster lösen").clicked() { if let Some(w) = self.studios.iter_mut().find(|w| w.group as u8 == *g) { w.want = Some(windows::Move::Detach); } ui.close(); }
            }
            _ => {}
        }
    }
    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.double_clicked() { if let Tab::Doc(r) = tab { if self.st.preview.as_deref() == Some(r.as_str()) { self.st.preview = None; } } }   // Doppelklick macht die Vorschau fest
        crate::ui::style::dock_tab_overlay(response, self.st.active_tabs.contains(tab), true);
    }
    fn allowed_in_windows(&self, _tab: &mut Tab) -> bool { false }   // Reiter bleiben im Editorbereich; Lösen in ein Fenster geht nur für Studios, über das Reitermenü
    fn on_close(&mut self, tab: &mut Tab) -> OnCloseResponse {
        if matches!(tab, Tab::Live) { self.st.live.stop(); }
        if matches!(tab, Tab::Emulator) { let d = self.st.dbg.as_ref(); self.st.emu.stop(d); }
        if let Tab::Studio(g) = tab { self.studios.retain(|w| w.group as u8 != *g); }   // Schließen des Reiters schließt das Studio samt Dateien wie das Fenster   // Ströme enden mit dem Reiter
        if let Tab::Doc(r) = tab { if let (Some(l), Some(pr)) = (&self.st.lsp, &self.st.project) { l.send(lsp::Cmd::Close { uri: lsp::uri(&pr.root, r) }); } self.st.docs.retain(|d| &d.rel != r); }
        OnCloseResponse::Close
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab {
            Tab::Project => self.st.project_ui(ui),
            Tab::Studio(g) => { let (pal, audio, reqs, root, data) = (self.pal, self.audio, self.reqs, self.root.clone(), self.data.clone()); if let Some(w) = self.studios.iter_mut().find(|w| w.group as u8 == *g) { w.show_embedded(ui, pal, audio, reqs, &root, &data); } }
            Tab::Overview => self.st.overview_ui(ui),
            Tab::Welcome => self.st.welcome_ui(ui),
            Tab::Doc(r) => { if ui.rect_contains_pointer(ui.max_rect()) { self.st.editor_hover = Some(r.clone()); } let State { docs, lsp, mnemonics, project, show_addr, data, .. } = &mut *self.st; if let (Some(d), Some(pr)) = (docs.iter_mut().find(|d| &d.rel == r), project.as_ref()) { if d.lang == crate::highlight::Lang::Basic { if let (Some(data), Some(pf)) = (data.as_deref(), pr.pf.as_ref()) { editor::select_name_dialect(data, &pf.dialect_of(r).or_else(|| pf.target_of(r).and_then(|t| crate::recent::target_dialect(data, &t))).unwrap_or_else(|| "cbm_v2".into())); } } editor::show(ui, d, mnemonics, lsp.as_ref(), &lsp::uri(&pr.root, r), *show_addr); } }
            Tab::Regs | Tab::Disasm | Tab::Memory | Tab::Snapshot | Tab::Breaks | Tab::Watch => self.st.debug_ui(ui, tab),
            Tab::MemMap => self.st.memmap_ui(ui),
            Tab::Outline => self.st.outline_ui(ui),
            Tab::StudioInfo => { let studios = &mut *self.studios; self.st.studio_ui(ui, studios) }
            Tab::Live => self.st.live_ui(ui),
            Tab::Emulator => { let root = self.st.project.as_ref().map(|p| p.root.clone()); let State { emu, dbg, mem_input, .. } = &mut *self.st; emu.ui(ui, dbg.as_ref(), root.as_deref(), mem_input); }
            Tab::ProjectEdit => self.st.projed_ui(ui),
            Tab::Settings => self.st.settings_ui(ui),
            Tab::Ref => self.st.ref_ui(ui),
            Tab::Help => self.st.help_ui(ui),
            Tab::Output => {
                let (mut copy_all, mut clear) = (false, false);
                egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
                    let k = crate::ui::tokens::t();
                    if self.st.output.is_empty() { ui.label(RichText::new("Noch nichts gebaut. Meldungen vom Bauen, Starten und Debuggen erscheinen hier (Bauen: Cmd/Strg+B).").small().color(k.dim)); }
                    for (l, s) in &self.st.output {
                        let c = match l { Level::Info => k.mute, Level::Ok => k.gr, Level::Warn => k.am, Level::Err => k.rd };
                        let r = ui.add(egui::Label::new(RichText::new(s).monospace().size(12.0).color(c)).sense(egui::Sense::click()));
                        r.context_menu(|ui| {
                            if ui.button("Zeile kopieren").clicked() { ui.ctx().copy_text(s.clone()); ui.close(); }
                            if ui.button("Alles kopieren").clicked() { copy_all = true; ui.close(); }
                            ui.separator();
                            if ui.button("Ausgabe leeren").clicked() { clear = true; ui.close(); }
                        });
                    }
                });
                if copy_all { ui.ctx().copy_text(self.st.output.iter().map(|(_, s)| s.as_str()).collect::<Vec<_>>().join("\n")); }
                if clear { self.st.output.clear(); }
            }
            Tab::BuildLog => self.st.buildlog_ui(ui),
            Tab::Tests => self.st.tests_ui(ui),
            Tab::Problems => {
                let k = crate::ui::tokens::t();
                let (errs, warns) = (self.st.problems.iter().filter(|p| p.error).count(), self.st.problems.iter().filter(|p| !p.error).count());
                if self.st.problems.is_empty() { if self.st.buildlog.is_empty() { ui.label(RichText::new("Noch nichts gebaut. Fehler und Warnungen des Baus erscheinen hier; ein Klick springt zur Zeile.").small().color(k.dim)); } else { ui.label(RichText::new("Keine Probleme.").color(k.gr)); } }
                else { ui.horizontal(|ui| { widgets::chip(ui, &format!("{errs} Fehler"), if errs > 0 { k.rd } else { k.mute }); widgets::chip(ui, &if warns == 1 { "1 Warnung".to_string() } else { format!("{warns} Warnungen") }, if warns > 0 { k.am } else { k.mute }); }); ui.add_space(2.0); }
                let mut copy_all = false;
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for p in &self.st.problems {
                    let c = if p.error { k.rd } else { k.am };
                    let text = format!("{}:{}  {}  {}", p.file, p.line, p.code, p.msg);
                    // dichte Zeile (22 px): Statuspunkt, Ort, Code, Meldung; Klick springt zur Zeile
                    let (rect, r) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
                    if r.hovered() { ui.painter().rect_filled(rect, 0.0, k.bg2); }
                    ui.painter().circle_filled(egui::pos2(rect.left() + 8.0, rect.center().y), 3.5, c);
                    let loc = ui.painter().text(egui::pos2(rect.left() + 20.0, rect.center().y), egui::Align2::LEFT_CENTER, format!("{}:{}", p.file, p.line), egui::FontId::monospace(11.0), k.cy);
                    let code = ui.painter().text(egui::pos2(loc.right() + 10.0, rect.center().y), egui::Align2::LEFT_CENTER, &p.code, egui::FontId::monospace(11.0), c);
                    ui.painter().with_clip_rect(rect).text(egui::pos2(code.right() + 10.0, rect.center().y), egui::Align2::LEFT_CENTER, &p.msg, egui::FontId::proportional(12.0), k.fg);
                    let r = r.on_hover_text(p.msg.as_str()).on_hover_cursor(egui::CursorIcon::PointingHand);
                    if r.clicked() {
                        self.st.open_request = Some(p.file.clone());
                        if p.line > 0 { self.st.pending_reveal = Some((p.file.clone(), p.line - 1)); }
                    }
                    r.context_menu(|ui| {
                        if ui.button("Zur Zeile springen").clicked() { self.st.open_request = Some(p.file.clone()); if p.line > 0 { self.st.pending_reveal = Some((p.file.clone(), p.line - 1)); } ui.close(); }
                        if ui.button("Meldung kopieren").clicked() { ui.ctx().copy_text(text.clone()); ui.close(); }
                        if ui.button("Alle Probleme kopieren").clicked() { copy_all = true; ui.close(); }
                    });
                }
                });
                if copy_all { ui.ctx().copy_text(self.st.problems.iter().map(|p| format!("{}:{}  {}  {}", p.file, p.line, p.code, p.msg)).collect::<Vec<_>>().join("\n")); }
            }
        }
    }
}

/// Ob „In den Papierkorb legen“ möglich ist: macOS (Finder), Linux mit `gio trash` (GLib/GVfs, auf Desktop-Installationen vorhanden) oder `trash-put` (trash-cli)
fn can_trash() -> bool {
    if cfg!(target_os = "macos") { return true; }
    cfg!(target_os = "linux") && (vc_emu::find_program("gio", &[]).is_some() || vc_emu::find_program("trash-put", &[]).is_some())
}

/// Datei in den Papierkorb legen; `false`, wenn das nicht geht (dann wird sie endgültig gelöscht)
fn move_to_trash(abs: &std::path::Path) -> bool {
    use std::process::{Command, Stdio};
    let quiet = |mut c: Command| c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success());
    if cfg!(target_os = "macos") {
        return Command::new("osascript").arg("-e").arg(format!("tell application \"Finder\" to delete POSIX file \"{}\"", abs.to_string_lossy().replace('"', "\\\""))).output().map_or(false, |o| o.status.success());
    }
    if cfg!(target_os = "linux") {
        if vc_emu::find_program("gio", &[]).is_some() { let mut c = Command::new("gio"); c.arg("trash").arg("--").arg(abs); if quiet(c) { return true; } }
        if vc_emu::find_program("trash-put", &[]).is_some() { let mut c = Command::new("trash-put"); c.arg("--").arg(abs); if quiet(c) { return true; } }
    }
    false
}

/// Datei im Dateimanager anzeigen
fn reveal_in_file_manager(path: &str) {
    if std::env::var("VC_SHOT").is_ok() { return; }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").args(["-R", path]).spawn(); }
    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(format!("/select,{path}")).spawn(); }
    #[cfg(all(unix, not(target_os = "macos")))] { if let Some(dir) = std::path::Path::new(path).parent() { let _ = std::process::Command::new("xdg-open").arg(dir).spawn(); } }
}

impl State {
    /// Projektbild selbsttätig anlegen: beim ersten Halt einer Debug-Sitzung mit VICE-Bild, falls das Projekt eines will und noch keines hat.
    /// Das Bild wird einmal je Sitzung angefordert (nur bei stehender Maschine) und beim Eintreffen gespeichert.
    fn auto_project_pic(&mut self) {
        let Some(d) = &self.dbg else { self.pic_auto = None; return };
        let Some(pr) = self.project.as_ref().filter(|p| p.pf.as_ref().map_or(false, |pf| !pf.no_preview)) else { return };
        let sid = std::sync::Arc::as_ptr(&d.snap) as *const () as usize;
        const DONE: u64 = u64::MAX;
        const BLANK: u64 = u64::MAX - 1;     // das Bild war leer (Maschine noch nicht gelaufen): beim nächsten Halt erneut versuchen
        if self.pic_auto == Some((sid, DONE)) { return; }
        let sn = d.snapshot();
        if sn.starting || sn.ended || !sn.can_frame { return; }
        match self.pic_auto {
            Some((s, BLANK)) if s == sid => { if sn.running { self.pic_auto = None; } }
            Some((s, seq)) if s == sid => {
                if sn.frame_seq > seq {
                    let blank = sn.frame.as_ref().map_or(true, |f| f.rgba.chunks(4).all(|p| p[..3] == f.rgba[..3]));
                    if blank { self.pic_auto = Some((sid, BLANK)); return; }
                    if let Some(f) = &sn.frame { if !vc_ide::livebild::preview_path(&pr.root).exists() { self.status = match vc_ide::livebild::save_project_preview(&pr.root, f) { Ok(_) => "Projektbild angelegt (projektbild.png)".into(), Err(e) => format!("Projektbild: {e}") }; } }
                    self.pic_auto = Some((sid, DONE));
                }
            }
            _ => {
                if sn.running { return; }
                if vc_ide::livebild::preview_path(&pr.root).exists() { self.pic_auto = Some((sid, DONE)); return; }
                d.send(crate::debug::Cmd::Frame);
                self.pic_auto = Some((sid, sn.frame_seq));
            }
        }
    }

    /// Reiter „Tests“: Ansicht zeichnen, Aktion für `App::run_tests` merken
    fn tests_ui(&mut self, ui: &mut egui::Ui) {
        let root = self.project.as_ref().map(|p| p.root.clone());
        let can_run = root.is_some() && self.data.is_some();
        if let Some(a) = self.tests.show(ui, can_run) { self.tests_act = Some(a); }
    }

    /// Bauprotokoll: je Programm die Stufen des Builds mit Dauer, Meldungen und allen beteiligten Dateien (Quellen, Zwischendateien, Ergebnis)
    fn buildlog_ui(&mut self, ui: &mut egui::Ui) {
        if self.buildlog.is_empty() { ui.weak("Noch nichts gebaut. Das Protokoll zeigt jede Stufe des Baus und alle Dateien, auch die Zwischendateien."); return; }
        let (ok_c, warn_c, gray) = (crate::theme::ok(), crate::theme::warn(), crate::theme::dim());
        let (mut open, mut hex, mut reveal): (Option<String>, Option<(String, String)>, Option<String>) = (None, None, None);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (bi, e) in self.buildlog.iter().enumerate() {
                ui.horizontal(|ui| {
                    widgets::chip(ui, if e.ok { "ok" } else { "Fehler" }, if e.ok { ok_c } else { crate::theme::bad() });
                    ui.label(RichText::new(&e.file).font(egui::FontId::new(13.0, crate::fonts::mono_strong())));
                    if e.ok { ui.label(RichText::new(format!("-> {}", e.artifact)).weak()); }
                    ui.label(RichText::new(format!("{} ms", e.log["ms"])).weak().small());
                });
                for (si, st) in e.log["stages"].as_array().into_iter().flatten().enumerate() {
                    let (icon, c) = match st["status"].as_str().unwrap_or("ok") { "error" => ("[Fehler]", PAL_ERR), "warn" => ("[Warnung]", warn_c), "skip" => ("[übersprungen]", gray), _ => ("[ok]", ok_c) };
                    let title = format!("{icon} {}   {} ms", st["title"].as_str().unwrap_or(""), st["ms"]);
                    egui::CollapsingHeader::new(RichText::new(title).color(c)).id_salt(("buildlog", bi, si)).default_open(true).show(ui, |ui| {
                        for l in st["lines"].as_array().into_iter().flatten().filter_map(|l| l.as_str()) {
                            let col = if l.starts_with("Fehler") { PAL_ERR } else if l.starts_with("Warnung") { warn_c } else { ui.visuals().text_color() };
                            ui.label(RichText::new(l).monospace().small().color(col));
                        }
                        for f in st["files"].as_array().into_iter().flatten() {
                            let (path, abs) = (f["path"].as_str().unwrap_or("").to_string(), f["abs"].as_str().unwrap_or("").to_string());
                            let (is_text, inside, size) = (f["text"] == true, f["inside"] == true, f["size"].as_u64());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("·").weak());
                                if is_text && inside && size.is_some() { if ui.link(RichText::new(&path).monospace()).on_hover_text("im Editor öffnen").clicked() { open = Some(path.clone()); } }
                                else { ui.label(RichText::new(&path).monospace()); }
                                ui.label(RichText::new(f["role"].as_str().unwrap_or("")).weak().small());
                                match size { Some(n) => { ui.label(RichText::new(format!("{n} Bytes")).weak().small()); if n <= 4 << 20 && ui.small_button("Hex").on_hover_text("Inhalt als Hexdump zeigen").clicked() { hex = Some((path.clone(), abs.clone())); } } None => { ui.label(RichText::new("(nicht vorhanden)").small().color(warn_c)); } }
                                if size.is_some() && ui.small_button("Zeigen").on_hover_text("im Dateimanager anzeigen").clicked() { reveal = Some(abs.clone()); }
                            });
                        }
                    });
                }
                ui.add_space(6.0); ui.separator();
            }
        });
        if let Some(p) = open { self.open_request = Some(p); }
        if let Some((name, abs)) = hex { match std::fs::read(&abs) { Ok(b) => self.hex_view = Some((name, b)), Err(e) => self.log(Level::Err, format!("{abs}: {e}")) } }
        if let Some(a) = reveal { reveal_in_file_manager(&a); }
    }

    /// Hexdump einer Datei aus dem Bauprotokoll (eigenes kleines Fenster)
    fn hex_window(&mut self, ctx: &egui::Context) {
        let Some((name, bytes)) = &self.hex_view else { return };
        let mut open = true;
        egui::Window::new(format!("Hexdump (nur lesen): {name} ({} Bytes)", bytes.len())).open(&mut open).default_size([640.0, 420.0]).show(ctx, |ui| {
            let h = ui.text_style_height(&egui::TextStyle::Monospace);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, h, (bytes.len() + 15) / 16, |ui, range| {
                for r in range {
                    let chunk = &bytes[r * 16..(r * 16 + 16).min(bytes.len())];
                    let hexs = chunk.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
                    let asc: String = chunk.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
                    ui.label(RichText::new(format!("{:06X}  {hexs:<47}  {asc}", r * 16)).monospace());
                }
            });
        });
        if !open { self.hex_view = None; }
    }
}

impl eframe::App for App {
    /// Selbsttest (VC_DEMO=ctxmenu): Rechtsklick in den Editor (VC_CTX_X/VC_CTX_Y in Punkten, sonst 40 % / 20 % des Fensters) bzw. VC_CTX_TREE=1 auf den Projektbaum.
    /// Mausereignisse müssen vor der Eingabeverarbeitung eingespeist werden.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        // Selbsttest (VC_DEMO=dropdrag): Ziehen mit der linken Maustaste von VC_DRAG_FROM nach VC_DRAG_TO (je „x,y“ in Punkten), ab Bild VC_DRAG_AT (Standard 60);
        // Loslassen nach VC_DRAG_HOLD weiteren Bildern (Standard 10; sehr groß = Zustand beim Ziehen aufnehmen)
        if std::env::var("VC_SHOT").is_ok() && std::env::var("VC_DEMO").map_or(false, |d| d.contains("dropdrag")) {
            let pt = |k: &str| std::env::var(k).ok().and_then(|v| v.split_once(',').and_then(|(x, y)| Some(egui::pos2(x.trim().parse().ok()?, y.trim().parse().ok()?))));
            if let (Some(a), Some(b)) = (pt("VC_DRAG_FROM"), pt("VC_DRAG_TO")) {
                // VC_DRAG_AT: Startbilder, kommagetrennt (mehrere Züge nacheinander, z. B. „60,140“ für ein Duplikat); VC_UNDO_AT: Bild für Cmd/Strg+Z im Editor
                let ats: Vec<u32> = std::env::var("VC_DRAG_AT").unwrap_or_else(|_| "60".into()).split(',').filter_map(|v| v.trim().parse().ok()).collect();
                let hold: u32 = std::env::var("VC_DRAG_HOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
                let btn = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
                let f = self.frames;
                for at in ats {
                    if f == at { raw.events.push(egui::Event::PointerMoved(a)); }
                    else if f == at + 2 { raw.events.push(btn(a, true)); }
                    else if f > at + 2 && f <= at + 14 { let t = (f - at - 2) as f32 / 12.0; raw.events.push(egui::Event::PointerMoved(a + (b - a) * t)); }
                    else if f == at + 14 + hold { raw.events.push(btn(b, false)); }
                }
                if let Some(u) = std::env::var("VC_UNDO_AT").ok().and_then(|v| v.parse::<u32>().ok()) {
                    if f + 5 == u { if let Some(r) = self.active_doc() { _ctx.memory_mut(|m| m.request_focus(egui::Id::new(("editor", &r)))); } }
                    let key = |pressed| egui::Event::Key { key: egui::Key::Z, physical_key: Some(egui::Key::Z), pressed, repeat: false, modifiers: egui::Modifiers::COMMAND };
                    if f == u { raw.events.push(key(true)); } else if f == u + 1 { raw.events.push(key(false)); }
                }
            }
        }
        if std::env::var("VC_SHOT").is_err() || !std::env::var("VC_DEMO").map_or(false, |d| d.contains("ctxmenu")) { return; }
        let r = raw.screen_rect.unwrap_or(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1400.0, 860.0)));
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
        let pos = if std::env::var("VC_CTX_TREE").is_ok() { egui::pos2(num("VC_CTX_X").unwrap_or(90.0), num("VC_CTX_Y").unwrap_or(138.0)) } else { egui::pos2(num("VC_CTX_X").unwrap_or(r.width() * 0.4), num("VC_CTX_Y").unwrap_or(r.height() * 0.2)) };
        let ev = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() };
        let at: u32 = std::env::var("VC_CTX_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
        match self.frames { f if f == at => raw.events.push(egui::Event::PointerMoved(pos)), f if f == at + 2 => raw.events.push(ev(true)), f if f == at + 4 => raw.events.push(ev(false)), _ => {} }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone(); let ctx = &ctx;
        if self.frames == 0 && std::env::var("VC_EMBED").is_ok() { ctx.set_embed_viewports(true); }
        self.repaint.tick(ctx);   // Windows: Fenster nach Maximieren/Größenänderung sicher neu zeichnen
        // Erst Strg/Cmd +, −, 0 (egui ändert den Zoom am Bildende) übernehmen, dann anwenden; umgekehrt setzt `apply` den Zoom sofort zurück
        self.view.follow_zoom(ctx);
        self.view.apply(ctx);
        self.poll_build(); self.poll_lsp();
        for e in vc_ide::launch::take_errors() { self.st.log(Level::Err, e); }
        if self.st.project.is_some() && self.file_dlg.is_none() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::N)) { self.file_dlg = Some(FileDlg::NewFile { dir: String::new(), name: String::new() }); }
        // Kürzel des Gerüsts (auf dem Mac fängt sie die Menüleiste ab, sonst kommen sie hier an)
        let mut key_cmds: Vec<crate::nativemenu::Cmd> = vec![];
        {
            use crate::nativemenu::Cmd;
            let (c, alt, shift) = (egui::Modifiers::COMMAND, egui::Modifiers::ALT, egui::Modifiers::SHIFT);
            for (m, k, cmd) in [(c | shift, egui::Key::Enter, Cmd::Maximize), (c | shift, egui::Key::F, Cmd::FindInProject), (c | alt, egui::Key::B, Cmd::ToggleInspector), (c, egui::Key::B, Cmd::ToggleSide), (c, egui::Key::J, Cmd::ToggleFooter), (c, egui::Key::K, Cmd::Palette), (c, egui::Key::W, Cmd::CloseFile), (c | alt, egui::Key::S, Cmd::SaveAll), (c, egui::Key::Backslash, Cmd::SplitEditor), (c | alt, egui::Key::F, Cmd::Replace), (c, egui::Key::F, Cmd::Find), (c, egui::Key::L, Cmd::GotoLine)] {
                if ctx.input_mut(|i| i.consume_key(m, k)) { key_cmds.push(cmd); }
            }
        }
        if std::mem::take(&mut self.st.want_zen) { self.zen = !self.zen; }
        if let Some(redo) = self.st.pending_undo.take() {
            let mods = if redo { egui::Modifiers::COMMAND | egui::Modifiers::SHIFT } else { egui::Modifiers::COMMAND };
            ctx.input_mut(|i| i.events.push(egui::Event::Key { key: egui::Key::Z, physical_key: Some(egui::Key::Z), pressed: true, repeat: false, modifiers: mods }));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::Y)) { self.charpick_doc = None; self.open_charpick(); }
        self.show_charpick(ctx);
        self.show_diskimg(ctx);
        // ohne Projekt braucht der Editorbereich den Willkommen-Reiter; mit Projekt zeigt er bei leerem Bereich einen Hinweis
        if self.st.project.is_none() && self.dock.iter_all_tabs().next().is_none() { layout::show_unique(&mut self.dock, Tab::Welcome); }
        self.settle_preview();
        self.st.auto_project_pic();
        { let State { emu, dbg, .. } = &mut self.st; emu.watchdog(dbg.as_ref()); }   // Zeitlupen-Live endet, wenn der Reiter nicht mehr zu sehen ist
        // Selbsttest: VC_SHOT=datei.png speichert nach einigen Bildern einen Screenshot und beendet das Programm
        if let Ok(path) = std::env::var("VC_SHOT") {
            ctx.request_repaint();   // Selbsttest: Bildzähler läuft auch ohne Eingaben weiter (sonst bleibt die Aufnahme nach dem Bauen stehen)
            self.frames += 1;
            // VC_WIN=BreitexHöhe: Fenstergröße für Aufnahmen schmaler Fenster (gespeicherte Fenstergröße überstimmt sonst den Start)
            if self.frames == 2 || self.frames == 6 { if let Some((w, h)) = std::env::var("VC_WIN").ok().and_then(|v| v.split_once('x').and_then(|(w, h)| Some((w.parse::<f32>().ok()?, h.parse::<f32>().ok()?)))) { ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false)); ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false)); ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h))); } }
            if let Ok(demo) = std::env::var("VC_DEMO") {
                if let Some(r) = self.active_doc() { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == r) {
                    if demo.contains("ref") && self.frames == 20 { d.cursor = (12, 11); }
                    if demo.contains("bp") && self.frames == 12 { d.breakpoints.insert(std::env::var("VC_BP_LINE").ok().and_then(|v| v.parse().ok()).unwrap_or(13)); }
                    if demo.contains("pcline") && self.frames == 12 { d.pc_line = Some(std::env::var("VC_PC_LINE").ok().and_then(|v| v.parse().ok()).unwrap_or(15)); }
                    if demo.contains("find") && self.frames == 12 { d.find.open = true; d.find.replace_mode = demo.contains("findrep"); d.find.query = "lda".into(); d.find.status = "3 Treffer".into(); d.select = Some((10, 13)); }
                    if demo.contains("goto") && self.frames == 12 { d.goto.open = true; d.goto.input = "7".into(); }
                    if demo.contains("fold") && self.frames == 10 { if let Some(k) = crate::fold::ranges(&d.text, d.lang).keys().next().copied() { d.folded.insert(k); } }
                    if demo.contains("complete") {
                        let id = egui::Id::new(("editor", &d.rel));
                        if self.frames == 20 { ctx.memory_mut(|m| m.request_focus(id)); }
                        if self.frames == 30 { ctx.input_mut(|i| i.events.push(egui::Event::Text("\n        vic2.bo".into()))); }
                        if self.frames == 32 { d.comp.all = [("vic2.border", "$D020 · Rahmenfarbe"), ("vic2.bg0", "$D021 · Hintergrund 0"), ("vic2.bgcolor", "Beispielwert"), ("vic2.bounce", "Beispielwert")].iter().map(|(l, x)| crate::lsp::Item { label: l.to_string(), kind: 6, detail: x.to_string() }).collect(); d.comp.open = true; }
                    }
                } }
                if demo.contains("palette") && self.frames == 20 { self.palette.toggle(); }
                if demo.contains("about") && self.frames == 15 { self.about_open = true; }
                if demo.contains("feedback") && self.frames == 15 { self.open_feedback(); }
                // Dialoge für das Handbuch: dlgstd (Standardzeichensatz hinzufügen), dlgsheet (PNG-Raster, Datei in VC_SHEET), dlgb2a (BASIC nach Assembler für das erste Dokument)
                // dlgexport: Export-Dialog des eingebetteten Studios (Anforderung wie aus dem Menü, Bild VC_EXPORT_AT, Standard 230)
                if demo.contains("dlgexport") && self.frames == std::env::var("VC_EXPORT_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(230) { crate::exportdlg::WANT.store(true, std::sync::atomic::Ordering::Relaxed); }
                // split: zweiter Editorbereich (VC_SPLIT = Datei, Standard: aktive Datei; Bild VC_SPLIT_AT, Standard 20; eine andere Datei wird vorher geöffnet)
                if demo.contains("split") {
                    let at: u64 = std::env::var("VC_SPLIT_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(20) as u64;
                    let want = std::env::var("VC_SPLIT").ok();
                    if self.frames as u64 + 5 == at { if let Some(f) = &want { self.open_doc(f); } }
                    if self.frames as u64 == at { if let Some(r) = want.or_else(|| self.active_doc()) { self.st.tab_act = Some(TabAct::Second(r)); } }
                    if self.frames as u64 == at + 15 && std::env::var("VC_SPLIT").is_ok() { if let Some(r) = self.st.docs.first().map(|d| d.rel.clone()) { self.reveal(Tab::Doc(r)); } }   // erste Datei links wieder nach vorn
                }
                // previewtab: Datei VC_PREVIEW wie per Einfachklick im Baum als Vorschau-Reiter öffnen
                if demo.contains("previewtab") && self.frames == 20 { if let Ok(f) = std::env::var("VC_PREVIEW") { self.st.preview_request = Some(f); } }
                if demo.contains("dlgstd") && self.frames == 15 { self.std_charset_dialog(); }
                if demo.contains("dlgsheet") && self.frames == 15 { self.import_sheet_dialog(); }
                if demo.contains("dlgb2a") && self.frames == 15 {
                    let req = self.st.docs.first().map(|d| d.rel.clone()).and_then(|rel| self.st.project.as_ref().map(|pr| { let mut req = vc_ide::b2a::Req::new(rel.clone()); req.out = vc_ide::b2a::default_out(&pr.root, &rel); if let Some(t) = pr.pf.as_ref().and_then(|pf| pf.target_of(&rel)) { if vc_ide::b2a::targets().contains(&t.as_str()) { req.target = t; } } req }));
                    if let Some(req) = req { self.file_dlg = Some(FileDlg::ConvertBasic { req }); }
                }
                // ROM-Assistent: romwiz (Schritt 1), romwiz2 (Schritt 2 „Dateien ablegen“)
                if demo.contains("romwiz") && self.frames == 15 { self.open_rom_wizard(); if demo.contains("romwiz2") { if let Some(w) = &mut self.rom_wizard { w.demo_step(1); } } }
                // Ablegen und Neu ordnen: dropfiles (VC_DROP = Pfade mit : getrennt, VC_DROP_DIR = fester Zielordner), reorg, reorgsub (mit Unterordnern)
                if demo.contains("dropfiles") && self.frames == 15 {
                    if let Ok(v) = std::env::var("VC_DROP") {
                        let paths: Vec<std::path::PathBuf> = v.split(':').filter(|x| !x.is_empty()).map(std::path::PathBuf::from).collect();
                        let (layout, rules) = self.st.project.as_ref().and_then(|p| p.pf.as_ref()).map(|pf| (pf.layout.clone(), pf.rules.clone())).unwrap_or_default();
                        self.import_dlg = crate::importdlg::ImportDlg::new(paths, std::env::var("VC_DROP_DIR").ok(), None, layout, rules);
                    }
                }
                // „…run“: nach dem Öffnen Enter drücken (bestätigt den Dialog), VC_SHOT_AT später für die Aufnahme
                if demo.contains("run") && self.frames == 25 { ctx.input_mut(|i| i.events.push(egui::Event::Key { key: egui::Key::Enter, physical_key: Some(egui::Key::Enter), pressed: true, repeat: false, modifiers: Default::default() })); }
                if demo.contains("reorg") && self.frames == 15 { self.open_reorg(); if demo.contains("reorgsub") { if let Some(d) = &mut self.reorg_dlg { d.subdirs = true; } } }
                // Fenstersteuerung: maximize = Ausgabe maximieren; embed = erstes Sprite öffnen und einbetten (VC_STUDIO=Dateipfad im Projekt)
                if demo.contains("maximize") && self.frames == 20 { self.st.want_zen = true; }
                // Seitenleiste: sidesearch (VC_SEARCH = Suchtext), sidetests (mit Lauf: VC_DEMO=sidetests,tests), drawer / drawerinsp = Schublade öffnen
                if demo.contains("sidesearch") && self.frames == 10 { self.show_project_search(); self.side.search.text = std::env::var("VC_SEARCH").unwrap_or_else(|_| "lda".into()); }
                if demo.contains("sidetests") && self.frames == 10 { self.zones_mut().view = sidebar::SideView::Tests; self.wp().side = true; self.open_drawer_if_narrow(Zone::Side); }
                if demo.contains("drawerinsp") && self.frames == 14 { self.side.drawer = Some(Zone::Inspector); }
                else if demo.contains("drawer") && self.frames == 14 { self.side.drawer = Some(Zone::Side); }
                if demo.contains("zones") && self.frames == 14 { let w = self.wp(); w.side = false; w.inspector = false; }
                if demo.contains("embed") && self.frames == 10 { if let Ok(r) = std::env::var("VC_STUDIO") { self.open_media(&r); } }
                if demo.contains("embed") && self.frames == 14 { if let Some(w) = self.wins.first_mut() { w.want = Some(windows::Move::Embed); } }
                if demo.contains("detach") && self.frames == 40 { if let Some(w) = self.wins.first_mut() { w.want = Some(windows::Move::Detach); } }
                // Live-Bild mit Testbild aus aufgezeichneten Paketen; VC_LIVE_ERR=lan|fw|loss zeigt Fehlerkarte bzw. Verluste
                if demo.contains("live") && self.frames == 12 {
                    self.switch_mode(Mode::Debug);
                    let bin = std::env::var("VC_LIVE_BIN").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vc-emu/tests/data/video.bin"));
                    self.st.live.demo(&bin, std::env::var("VC_LIVE_ERR").ok().as_deref());
                    self.open_live();
                }
                if demo.contains("showdoc") && self.frames == 8 { if let Some(r) = std::env::var("VC_DOC").ok().filter(|r| self.st.docs.iter().any(|d| &d.rel == r)).or_else(|| self.st.docs.first().map(|d| d.rel.clone())) { self.reveal(Tab::Doc(r)); } }
                if demo.contains("showov") && self.frames == 25 { self.reveal(Tab::Overview); }
                if demo.contains("showprob") && self.frames == 110 { self.reveal(Tab::Problems); }
                // Reiter „Ausgabe“ nach vorn (VC_DEMO=showout, Bild VC_OUT_AT, Standard 380)
                if demo.contains("showout") && self.frames == std::env::var("VC_OUT_AT").ok().and_then(|v| v.parse().ok()).unwrap_or(380) { self.reveal(Tab::Output); }
                if demo.contains("showref") && self.frames == 25 { self.reveal(Tab::Ref); }
                if demo.contains("showmm") && self.frames == 25 {
                    if let Ok(f) = std::env::var("VC_MMFILTER") { ctx.data_mut(|d| d.insert_temp(egui::Id::new("memmap_filter"), f)); }
                    self.reveal(Tab::MemMap);
                }
                if demo.contains("showhelp") && self.frames == 25 { self.st.help_open("debugger"); self.reveal(Tab::Help); }
                // Handbuchkapitel mit Bild (VC_DEMO=showhandbuch, Kapitel VC_HELP_TOPIC, Standard „erstes-projekt“; Bilder lokal: VC_HELP_IMAGES=docs/handbuch/bilder, Scroll VC_HELP_SCROLL)
                if demo.contains("showhandbuch") && self.frames == 25 { let k = std::env::var("VC_HELP_TOPIC").unwrap_or_else(|_| "kapitel:erstes-projekt".into()); self.st.help_open(&k); self.reveal(Tab::Help); }
                if demo.contains("dlgp") && self.frames == 15 { self.new_project_dialog(); }
                if demo.contains("wizbasic") && self.frames == 20 { if let Some(w) = &mut self.wizard { w.demo("atari8", true); } }
                if demo.contains("wizvic") && self.frames == 20 { if let Some(w) = &mut self.wizard { w.demo("vic20", false); } }
                if demo.contains("disk") && self.frames == 15 { if let Ok(p) = std::env::var("VC_IMAGE") { self.open_image(p.into()); } }
                if demo.contains("diskimg") && self.frames == 20 { if let Some(d) = &mut self.diskimg { d.select_all(); } }
                if demo.contains("dbgdemo") && self.frames == 12 { self.st.dbg = Some(Debugger::demo(crate::debugger::demo_snap(&std::env::var("VC_DBG_BACKEND").unwrap_or_else(|_| "vice".into())))); self.switch_mode(Mode::Debug); }
                // Doc-Reiter nach vorn (für Zeichen einfügen und Datei-Dialoge), Datei-Dialoge: dlgnew, dlgren, dlgdel
                if demo.contains("tabdoc") && self.frames == 10 { if let Some(r) = self.st.docs.first().map(|d| d.rel.clone()) { self.reveal(Tab::Doc(r)); } }
                if demo.contains("dlgnew") && self.frames == 15 { self.file_dlg = Some(FileDlg::NewFile { dir: String::new(), name: "level".into() }); }
                if demo.contains("dlgren") && self.frames == 15 { if let Some(r) = std::env::var("VC_RENAME_FROM").ok().or_else(|| self.st.docs.first().map(|d| d.rel.clone())) { self.file_dlg = Some(FileDlg::Rename { name: std::env::var("VC_RENAME_TO").unwrap_or_else(|_| r.clone()), rel: r, ask: RenameAsk { rewrite: true, ..Default::default() } }); } }
                if demo.contains("dlgdel") && self.frames == 15 { if let Some(r) = self.st.docs.first().map(|d| d.rel.clone()) { self.file_dlg = Some(FileDlg::Delete { rel: r }); } }
                if demo.contains("charpick") && self.frames == 15 { self.open_charpick(); }
                if demo.contains("b2adlg") && self.frames == 15 { if let Ok(r) = std::env::var("VC_B2A_FILE") { self.run_tree_action(ctx, TreeAct::ConvertBasic(r)); } }
                if demo.contains("b2arun") && self.frames == 15 { if let Ok(r) = std::env::var("VC_B2A_FILE") { self.run_tree_action(ctx, TreeAct::ConvertBasic(r)); } }
                if demo.contains("b2arun") && self.frames == 20 { if let Some(FileDlg::ConvertBasic { req }) = self.file_dlg.take() { if let Some(root) = self.st.project.as_ref().map(|p| p.root.clone()) { self.run_convert(&root, &req); } } }
                if demo.contains("chars") && self.frames == 15 {
                    self.charpick_doc = self.st.docs.first().map(|d| d.rel.clone());
                    if let Some(d) = self.st.docs.first_mut() { if let Ok(l) = std::env::var("VC_CHARS_LINE") { d.cursor = (l.parse().unwrap_or(1), std::env::var("VC_CHARS_COL").ok().and_then(|c| c.parse().ok()).unwrap_or(1)); } }
                    self.open_charpick();
                }
                if demo.contains("charsins") && self.frames == 30 { if let Some(d) = self.st.docs.first_mut() { d.insert = Some("{clr}A{$c1}".into()); } }
                if demo.contains("projed") && self.frames == 15 { self.open_project_editor(); }
                if demo.contains("settings") && self.frames == 15 { self.open_settings(); }
                if demo.contains("projbuild") && self.frames == 15 { self.build(ctx, After::Nothing); }
                if demo.contains("dlga") && self.frames == 15 { self.new_media_dialog(); }
                if demo.contains("media") && self.frames == 10 {
                    let list: Vec<String> = self.st.project.as_ref().map(|p| p.entries.iter().filter(|e| vc_media::naming::is_data_file(&e.rel)).map(|e| e.rel.clone()).collect()).unwrap_or_default();
                    for r in list { self.open_media(&r); }
                }
                if demo.contains("build") && self.frames == 15 { self.build(ctx, After::Nothing); }
                if demo.contains("open2") && self.frames == 700 { if let Ok(f) = std::env::var("VC_OPEN2") { self.open_doc(&f); } }
                if demo.contains("runit") && self.frames == 15 { self.build(ctx, After::Run); }
                if demo.contains("debug") && self.frames == 15 { self.build(ctx, After::Debug); }
                // Selbsttest Zeilenkarte: Haltepunkt in der BASIC-Quelle (VC_B2A_FILE, Zeile VC_BP_LINE, 0-basiert), dann Debuggen
                if demo.contains("b2adbg") && self.frames == 10 { if let Ok(r) = std::env::var("VC_B2A_FILE") { self.open_doc(&r); if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == r) { d.breakpoints.insert(std::env::var("VC_BP_LINE").ok().and_then(|v| v.parse().ok()).unwrap_or(4)); } } }
                if demo.contains("b2adbg") && self.frames == 20 { self.build(ctx, After::Debug); }
                if demo.contains("dbgfeat") {
                    if self.frames == 500 { for d in &mut self.st.docs { d.breakpoints.clear(); } self.st.watch_exprs = vec!["a".into(), "[$d020]".into(), "x + 1".into()];
                        if let Some(d) = &self.st.dbg { d.send(DbgCmd::AddPoint { kind: "write".into(), start: 0xD020, end: 0xD020, cond: Some("([$d020] & 15) == 9".into()), ignore: 0 }); d.send(DbgCmd::Watches(self.st.watch_exprs.clone())); } }
                    if self.frames == 530 { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Cont); } }
                    if self.frames == 1100 { self.reveal(Tab::Watch); }
                }
                if demo.contains("dumpcs") && self.frames == 500 { if let (Some(d), Some(pr), Some(data)) = (&self.st.dbg, &self.st.project, &self.st.data) { d.send(DbgCmd::SaveCharset { addr: 0xE000, profile: "c64.char.hires".into(), name: "test_dump_tmp".into(), root: pr.root.clone(), data: data.clone() }); } }
                if demo.contains("snap") {
                    if self.frames == 5 { if let Ok(f) = std::env::var("VC_OPEN2") { self.open_doc(&f); } }
                    if self.frames == 15 { self.build(ctx, After::Debug); }
                    if self.frames == 900 { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Pause); } }
                    if self.frames == 930 { self.reveal(Tab::Snapshot); }
                }
                // Selbsttest Reiter „Emulator“ (zusammen mit „debug“): Pause, Reiter zeigen; „emuslow“ schaltet danach das Zeitlupen-Live ein; „emutext“ speist Text ein
                if demo.contains("emu") {
                    if self.frames == 900 { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Pause); } }
                    if self.frames == 930 { self.reveal(Tab::Emulator); }
                    if self.frames == 1000 && demo.contains("emutext") { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Text("print 123\n".into())); d.send(DbgCmd::Cont); } }
                    if self.frames == 1060 && demo.contains("emutext") { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Pause); } }
                    if self.frames == 1200 && demo.contains("emuslow") { self.st.emu.slow = true; if let Some(d) = &self.st.dbg { d.send(DbgCmd::Cont); } }
                }
                if demo.contains("snapsave") && self.frames == 1000 { if let (Some(d), Some(pr), Some(data)) = (&self.st.dbg, &self.st.project, &self.st.data) { if let Some(Ok(det)) = d.snapshot().detected { d.send(DbgCmd::SnapSave { system: "c64".into(), det: det.clone(), opts: crate::snapshot::Opts { screen: Some(det.screen), charset: Some(0x2000), rom: false, sprites: true, tiles: true, name: "snap".into() }, root: pr.root.clone(), data: data.clone() }); } } }
                if demo.contains("tests") && self.frames == 12 { self.st.tests_act = Some(crate::tests_panel::Act::Run(vc_ide::tests::suite::Only::All)); }
                if demo.contains("tests") && self.frames == 30 { self.reveal(Tab::Tests); }
                if demo.contains("testsdbg") && self.frames == 12 { if let (Some(pr), Ok(t)) = (&self.st.project, std::env::var("VC_TEST_NAME")) { let f = pr.root.join(std::env::var("VC_TEST_FILE").unwrap_or_default()); self.st.tests_act = Some(crate::tests_panel::Act::Debug(f, t)); } }
                // Handbuch-Aufnahmen (zusammen mit „debug“): VC_WATCH = Ausdrücke mit ; getrennt, VC_POINT = art,von,bis[,bedingung] (art: exec|read|write|access, Adressen hex),
                // VC_DBG_TAB = regs|disasm|memory|breaks|watch|snapshot|memmap|emulator|live: Reiter nach vorn (Bild 930, nach Anhalten)
                if demo.contains("dbgtab") {
                    if self.frames == 500 {
                        if let Ok(w) = std::env::var("VC_WATCH") { self.st.watch_exprs = w.split(';').map(|x| x.trim().to_string()).collect(); if let Some(d) = &self.st.dbg { d.send(DbgCmd::Watches(self.st.watch_exprs.clone())); } }
                        if let (Ok(pt), false) = (std::env::var("VC_POINT"), std::env::var("VC_POINT_LATE").is_ok()) { let f: Vec<&str> = pt.split(',').collect(); if f.len() >= 3 { if let (Ok(a), Ok(b)) = (u16::from_str_radix(f[1], 16), u16::from_str_radix(f[2], 16)) { if let Some(d) = &self.st.dbg { d.send(DbgCmd::AddPoint { kind: f[0].into(), start: a, end: b, cond: f.get(3).map(|c| c.to_string()), ignore: 0 }); } } } }
                    }
                    // VC_POINT_LATE=1: Haltepunkt erst nach dem Anhalten setzen und weiterlaufen lassen (Treffer erscheint im Programm, nicht im KERNAL-Start)
                    if self.frames == 905 && std::env::var("VC_POINT_LATE").is_ok() { if let (Ok(pt), Some(d)) = (std::env::var("VC_POINT"), &self.st.dbg) { let f: Vec<&str> = pt.split(',').collect(); if f.len() >= 3 { if let (Ok(a), Ok(b)) = (u16::from_str_radix(f[1], 16), u16::from_str_radix(f[2], 16)) { d.send(DbgCmd::AddPoint { kind: f[0].into(), start: a, end: b, cond: f.get(3).map(|c| c.to_string()), ignore: 0 }); d.send(DbgCmd::Cont); } } } }
                    if self.frames == 930 { if let Ok(t) = std::env::var("VC_DBG_TAB") { let tab = match t.as_str() { "regs" => Some(Tab::Regs), "disasm" => Some(Tab::Disasm), "memory" => Some(Tab::Memory), "breaks" => Some(Tab::Breaks), "watch" => Some(Tab::Watch), "snapshot" => Some(Tab::Snapshot), "memmap" => Some(Tab::MemMap), "emulator" => Some(Tab::Emulator), "live" => Some(Tab::Live), _ => None }; if let Some(tab) = tab { self.reveal(tab); } } }
                }
                if demo.contains("bpstep") && self.frames == 400 { if let Some(d) = &self.st.dbg { d.send(DbgCmd::Step("into")); } }
            }
            let names: Vec<_> = self.wins.iter().map(|w| (w.id(), format!("{:?}", w.group).to_lowercase())).collect();
            windows::save_shot_root(ctx, &path, &names);
            if windows::SHOTS.load(std::sync::atomic::Ordering::SeqCst) > self.wins.len() { std::process::exit(0); }
        }
        let (open, save, build, shift) = ctx.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::O), i.modifiers.command && i.key_pressed(egui::Key::S), i.key_pressed(egui::Key::F7), i.modifiers.shift));
        let (f5, f6, f8, f10, f11, f1) = ctx.input(|i| (i.key_pressed(egui::Key::F5), i.key_pressed(egui::Key::F6), i.key_pressed(egui::Key::F8), i.key_pressed(egui::Key::F10), i.key_pressed(egui::Key::F11), i.key_pressed(egui::Key::F1)));
        let mut do_open = open;
        // Datei ▸ „BASIC nach Assembler konvertieren …“ gilt für die aktive .bas/.lst-Datei
        let conv_rel = self.active_doc().filter(|r| { let l = r.to_ascii_lowercase(); l.ends_with(".bas") || l.ends_with(".lst") || l.ends_with(".tur") });
        let mut want_convert = false;
        vc_ide::settings::set_project_overrides(self.st.project.as_ref().and_then(|p| p.pf.as_ref()).map(|pf| pf.overrides()).unwrap_or_default());
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Comma)) { self.open_settings(); }
        // Menüleiste am Bildschirmrand (macOS): Zustand melden, Klicks abarbeiten
        let menu_state: crate::nativemenu::MenuState;
        {
            let has = self.st.project.is_some();
            let ms = crate::nativemenu::MenuState { project: has, idle: has && self.st.building.is_none(), can_debug: self.st.dbg.is_none(), convert: conv_rel.is_some(), legacy: self.st.legacy,
                detached: self.wins.iter().any(|w| !w.embedded), hotkeys: crate::bitmap::HOTKEYS.load(std::sync::atomic::Ordering::Relaxed), show_addr: self.st.show_addr,
                side: self.layout_wp().side, inspector: self.layout_wp().inspector, footer: self.layout_wp().footer, debug_wp: self.mode == Mode::Debug, zen: self.zen, split: layout::is_split(&self.dock), has_doc: !self.st.docs.is_empty(), studio: self.wins.iter().any(|w| w.embedded),
                debugging: self.st.dbg.is_some(), halted: self.st.dbg.as_ref().map_or(false, |d| { let sn = d.snapshot(); !sn.running && !sn.starting }),
                scheme: match self.view.scheme { crate::theme::Scheme::Dark => 0, crate::theme::Scheme::Light => 1, crate::theme::Scheme::System => 2 } };
            self.native.sync(ms); menu_state = ms;
            { let names = self.recent_names(); self.native.set_recent(&names); }
            for c in self.native.poll().into_iter().chain(key_cmds.drain(..)) { self.menu_cmd(ctx, c, &mut do_open, &mut want_convert); }
        }
        // Befehlspalette (Cmd/Strg+P): dieselben Befehle wie das Menü, dazu Dateien des Projekts und Studios
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) { self.palette.toggle(); }   // alter Platz der Befehlssuche; neu: Cmd/Strg+K
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::N)) { self.new_project_dialog(); }
        if crate::about::show(ctx, &mut self.about_open) { self.about_open = false; self.open_feedback(); }
        crate::feedback::show(ctx, &mut self.feedback);
        crate::splash::show(ctx, &mut self.splash);
        if self.palette.is_open() {
            let entries = self.st.palette_entries(menu_state);
            match self.palette.show(ctx, &entries) {
                Some(crate::cmdpalette::Item::Cmd(c)) => self.menu_cmd(ctx, c, &mut do_open, &mut want_convert),
                Some(crate::cmdpalette::Item::File(r)) => self.st.open_request = Some(r),
                Some(crate::cmdpalette::Item::PickImage) => self.st.pick_image_request = true,
                None => {}
            }
        }
        if crate::ui::titlebar::active() { crate::ui::titlebar::sync_theme(ctx); }
        self.title_row(ctx, ui, menu_state, &mut do_open, &mut want_convert);
        self.command_bar(ctx, ui);
        let status = {
            let cur = self.active_doc().and_then(|r| self.st.docs.iter().find(|d| d.rel == r).map(|d| (d.cursor, d.lang)));
            let sys = self.st.project.as_ref().and_then(|p| if self.st.target == "auto" { p.systems.first().cloned() } else { Some(self.st.target.clone()) });
            let target = sys.as_ref().map(|sy| { let v = self.st.project.as_ref().and_then(|p| p.pf.as_ref()).and_then(|pf| pf.targets.iter().find(|t| t.id == *sy || t.system == *sy)).map(|t| t.variant.to_uppercase()).unwrap_or_default(); if v.is_empty() { crate::recent::short_name(sy).to_string() } else { format!("{} {v}", crate::recent::short_name(sy)) } });
            let step_block = self.st.dbg.as_ref().and_then(|d| d.snapshot().step_block);
            let debug = self.st.dbg.as_ref().map(|d| { let sn = d.snapshot(); if sn.starting { "Emulator startet …".to_string() } else if sn.running { "läuft".into() } else { format!("angehalten ${:04X}", sn.pc) } });
            crate::shell::Status { target, lang: cur.map(|(_, l)| l.name().to_string()), cursor: cur.map(|(c, _)| c), message: self.st.status.clone(), step_block, emulator: sys.and_then(|sy| self.st.emulator_status(&sy)), debug, tests: self.st.tests.status_text().map(|t| (t, self.st.tests.failed_count() > 0)) }
        };
        let tests_clicked = egui::Panel::bottom("status").exact_size(layout::STATUS_H).show(ui, |ui| crate::shell::status_bar(ui, &status)).inner;
        if tests_clicked { self.st.show_tab = Some(Tab::Tests); }
        if save { self.save_active(); }
        let idle = self.st.building.is_none() && self.st.project.is_some();
        if build && idle { if shift { self.build_active(ctx, After::Nothing) } else { self.build(ctx, After::Nothing) } }
        if f5 && idle { if shift { self.build_active(ctx, After::Run) } else { self.build(ctx, After::Run) } }
        if f6 && idle && self.st.dbg.is_none() { if shift { self.build_active(ctx, After::Debug) } else { self.build(ctx, After::Debug) } }
        if let Some(d) = &self.st.dbg { let sn = d.snapshot(); if !sn.running && !sn.starting { if f8 { d.send(DbgCmd::Cont); } if f10 { d.send(DbgCmd::Step("over")); } if f11 { d.send(DbgCmd::Step(if shift { "out" } else { "into" })); } } }
        if self.stop_dbg_request { self.stop_dbg_request = false; self.stop_debug(); self.st.status = "Debugger beendet".into(); }
        if let Some(a) = self.st.disasm_toggle.take() {
            if let Some((f, n)) = self.st.info.as_ref().and_then(|i| i.line_at(a)).map(|(f, n)| (f.to_string(), n)) { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == f) { if !d.breakpoints.remove(&(n - 1)) { d.breakpoints.insert(n - 1); } } }
        }
        self.show_dialogs(ctx);
        self.show_wizard(ctx);
        self.show_setup(ctx);
        self.run_overview(ctx);
        self.run_tests(ctx);
        if let Some(a) = self.st.welcome_act.take() {
            use crate::welcome::Act;
            match a {
                Act::New => self.new_project_dialog(),
                Act::Open => do_open = true,
                Act::OpenExample => {
                    let mut dlg = rfd::FileDialog::new().add_filter("Retro-Projekt (.vcprj)", &["vcprj"]).set_title("Beispiel öffnen");
                    if let Some(ex) = self.st.data.as_ref().and_then(|d| d.parent()).map(|p| p.join("examples")).filter(|p| p.is_dir()) { dlg = dlg.set_directory(ex); }
                    if let Some(f) = dlg.pick_file() { self.open_project_path(f); }
                }
                Act::OpenPath(p) => self.open_project_path(p),
                Act::Settings => self.open_settings(),
                Act::Feedback => self.open_feedback(),
                Act::Help(id) => { self.st.help_open(id); self.st.show_tab = Some(Tab::Help); }
                Act::NewMedia => self.new_media_dialog(),
                Act::Roms => self.open_rom_wizard(),
                Act::Wizard => self.open_setup(crate::setup_wizard::Launch::Recall),
                Act::Banner(dismiss) => {
                    if let Some(b) = self.st.welcome.banner.take() {
                        if !dismiss { self.open_setup(crate::setup_wizard::Launch::Recall); }
                        else if let Some(p) = vc_ide::setup::Paths::current() {
                            let mut w = vc_ide::setup::WizardState::load(&p.wizard);
                            match b { crate::welcome::Banner::Incomplete => w.dismiss("banner"), crate::welcome::Banner::OfferOnce => w.first_run_done = true, crate::welcome::Banner::News => w.known_programs = vc_ide::setup::found_kinds(&|k| vc_ide::platform::discover(k)) }
                            let _ = w.save(&p.wizard);
                        }
                    }
                }
            }
        }
        self.show_file_dlg(ctx);
        self.show_import_dlg(ctx);
        if std::mem::take(&mut self.st.reorg_request) { self.open_reorg(); }
        self.show_reorg_dlg(ctx);
        self.handle_drops(ctx);
        self.show_drop_note(ctx);
        if let Some(a) = self.st.tree_act.take() { self.run_tree_action(ctx, a); }
        self.st.hex_window(ctx);
        self.sync_debug();
        // aktives Dokument: Wort am Cursor (Referenz) und Sprache; F1 öffnet die Hilfe dazu
        if let Some(r) = self.active_doc() { if let Some(d) = self.st.docs.iter().find(|d| d.rel == r) { self.st.active_word = d.word_at_cursor(); self.st.active_lang = d.lang; if d.lang != crate::highlight::Lang::Plain { self.st.last_asm = Some(d.rel.clone()); } } }
        if f1 { let w = self.st.active_word.clone(); if !w.is_empty() { self.st.help_open(&w); } self.st.help.want_tab = true; }
        if self.st.help.want_tab { self.st.help.want_tab = false; self.reveal(Tab::Help); }
        if self.layout_t.elapsed().as_secs() >= 5 { self.layout_t = std::time::Instant::now(); self.save_layout(); self.save_session(); }
        if want_convert { if let Some(r) = conv_rel.clone() { self.run_tree_action(ctx, TreeAct::ConvertBasic(r)); } }
        if do_open {
            let mut dlg = rfd::FileDialog::new().add_filter("Retro-Projekt (.vcprj)", &["vcprj"]).set_title("Projekt öffnen");
            if let Some(dir) = self.open_start_dir() { dlg = dlg.set_directory(dir); }
            if let Some(f) = dlg.pick_file() { self.open_project_path(f); }
        }
        self.st.active_tabs = self.dock.iter_leaves().filter_map(|(_, l)| l.tabs.get(l.active.0).cloned()).collect();
        // deutsche Beschriftungen der eingebauten Reiter-Menüeinträge
        if self.dock.translations.tab_context_menu.close_button != "Schließen" {
            let t = &mut self.dock.translations;
            t.tab_context_menu = egui_dock::TabContextMenuTranslations { close_button: "Schließen".into(), eject_button: "In eigenem Fenster lösen".into(), hide_tab_bar_button: "Reiterleiste ausblenden".into(), show_tab_bar_button: "Reiterleiste einblenden".into() };
            t.leaf.close_all_button = "Fenster schließen".into();
            t.leaf.close_button_disabled_tooltip = "Dieser Bereich enthält Reiter, die sich nicht schließen lassen.".into();
        }
        self.st.view = self.view;
        self.show_zones(ctx, ui);
        if self.st.view != self.view { self.view = self.st.view; self.view.apply(ctx); self.view.save(); }
        if let Some(r) = self.st.second_request.take() { self.open_doc_as(&r, Open::Second); }
        if let Some(r) = self.st.preview_request.take() { self.open_doc_as(&r, Open::Preview); }
        if let Some(r) = self.st.open_request.take() {
            self.open_doc(&r);
            if let Some((f, l)) = self.st.pending_reveal.take() { if f == r { if let Some(d) = self.st.docs.iter_mut().find(|d| d.rel == f) { d.reveal = Some(l); d.flash = Some((l, 0.0)); } } else { self.st.pending_reveal = Some((f, l)); } }
        }
        editor::DEBUGGING.store(self.st.dbg.is_some(), std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = self.st.show_tab.take() { self.reveal(t); }
        self.run_doc_actions();
        if let Some(a) = self.st.tab_act.take() { self.run_tab_action(ctx, a); }
        match std::mem::replace(&mut self.st.se_act, crate::settings_ui::Act::None) {
            crate::settings_ui::Act::Save => if let Some(e) = &mut self.st.se {
                match vc_ide::settings::set(e.draft.clone()) {
                    Ok(()) => { e.saved = e.draft.clone(); let ids: Vec<String> = e.draft.machines.iter().map(|m| m.id.clone()).collect(); if !self.st.run_on.is_empty() && self.st.run_on != "@emulator" && !ids.contains(&self.st.run_on) { self.st.run_on.clear(); save_prefs(self.st.show_addr, ""); } self.st.status = "Einstellungen gespeichert".into(); self.st.output.push((Level::Ok, "Einstellungen gespeichert".into())); }
                    Err(er) => self.st.output.push((Level::Err, er)),
                }
            },
            crate::settings_ui::Act::Revert => if let Some(e) = &mut self.st.se { e.draft = e.saved.clone(); },
            crate::settings_ui::Act::OpenProjectSettings => self.open_project_editor(),
            crate::settings_ui::Act::None => {}
        }
        { let msgs: Vec<(bool, String)> = self.st.bg_msgs.lock().map(|mut g| std::mem::take(&mut *g)).unwrap_or_default(); for (ok, t) in msgs { self.st.status = if ok { "gestartet".into() } else { "Start fehlgeschlagen".into() }; self.st.log(if ok { Level::Ok } else { Level::Err }, t); } }
        if std::mem::take(&mut self.st.open_projed) { self.open_project_editor(); }
        if std::mem::take(&mut self.st.pick_image_request) { self.pick_image(); }
        if let Some(m) = self.st.main_request.take() { self.set_main(m); }
        match std::mem::replace(&mut self.st.pe_act, crate::projed::Act::None) {
            crate::projed::Act::Save => { self.save_project_file(); }
            crate::projed::Act::SaveBuild => { if self.save_project_file() { self.build(ctx, After::Nothing); } }
            crate::projed::Act::Revert => { if let Some(e) = &mut self.st.pe { e.draft = e.saved.clone(); } }
            crate::projed::Act::None => {}
        }
        // Editor-Fenster (eigene Betriebssystem-Fenster)
        let (proj, shot) = (self.st.project.as_ref().map(|p| p.name.clone()).unwrap_or_default(), std::env::var("VC_SHOT").ok());
        let (root, data) = (self.st.project.as_ref().map(|p| p.root.clone()).unwrap_or_default(), self.st.data.clone().unwrap_or_default());
        for w in &mut self.wins { w.show(ctx, &proj, shot.as_deref(), &self.pal, &self.audio, &self.reqs, &root, &data); }
        let inserts: Vec<String> = self.reqs.insert.borrow_mut().drain(..).collect();
        for rel in inserts { self.insert_asset_include(&rel); }
        if self.reqs.run.take() && self.st.building.is_none() && self.st.project.is_some() { self.build(ctx, After::Run); }   // „Im Emulator testen“ im Export-Dialog
        let (opens, logs): (Vec<String>, Vec<(u8, String)>) = (self.reqs.open.borrow_mut().drain(..).collect(), self.reqs.log.borrow_mut().drain(..).collect());
        for (lv, m) in logs { self.st.log(match lv { 0 => Level::Info, 1 => Level::Warn, _ => Level::Err }, m); }
        if !opens.is_empty() { if let Some(pr) = &mut self.st.project { pr.rescan(); } for r in opens { self.open_media(&r); } }
        self.wins.retain(|w| !w.closed);
        self.apply_studio_moves();
        crate::winctl::edges(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(match &self.st.project { Some(p) => format!("VintageCoder – {}", p.name), None => "VintageCoder".into() }));
    }
}

/// Gibt es die Datei schon, die ein Dialog mit diesem Namen anlegen würde? `full` ist der Name mit Endung. Ein leeres Namensfeld
/// ist kein Fehler: dann gibt es nichts zu prüfen (nur die Schaltfläche bleibt gesperrt).
fn file_exists_for(root: &std::path::Path, dir: &str, name: &str, full: &str) -> bool {
    if name.is_empty() { return false; }
    root.join(if dir.is_empty() { full.to_string() } else { format!("{dir}/{full}") }).exists()
}

#[cfg(test)]
mod dialog_tests {
    use super::*;

    #[test]
    fn leeres_namensfeld_meldet_keine_vorhandene_datei() {
        let d = std::env::temp_dir().join(format!("vc_dlg_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src")).unwrap();
        std::fs::write(d.join("a.asm"), "").unwrap();
        std::fs::write(d.join("src/b.asm"), "").unwrap();
        // leer: weder Projektordner noch Unterordner gelten als „vorhandene Datei“
        assert!(!file_exists_for(&d, "", "", ""));
        assert!(!file_exists_for(&d, "src", "", ""));
        // vorhanden / nicht vorhanden, jeweils mit Endung geprüft
        assert!(file_exists_for(&d, "", "a", "a.asm"));
        assert!(file_exists_for(&d, "src", "b", "b.asm"));
        assert!(!file_exists_for(&d, "", "neu", "neu.asm"));
        let _ = std::fs::remove_dir_all(&d);
    }
}

/// Pfad von Verzeichnis `from` (relativ zum Projekt) zur Datei `to` (relativ zum Projekt), mit `..` wo nötig – so löst der Assembler Includes auf
#[cfg(test)]
mod projfile_tests {
    #[test]
    fn projektdatei_wird_erkannt() {
        assert!(super::is_project_file("demo.vcprj") && super::is_project_file("sub/Demo.VCPRJ"));
        assert!(!super::is_project_file("demo.asm") && !super::is_project_file("vcprj.txt"));
    }
}

/// Basisverzeichnis der Projekte aus den Einstellungen, wenn es als Ordner existiert
fn projects_base_dir() -> Option<std::path::PathBuf> {
    std::path::PathBuf::from(vc_ide::settings::expand_home(&vc_ide::setup::effective_projects_dir(&vc_ide::settings::get()))).canonicalize().ok().filter(|p| p.is_dir())
}
