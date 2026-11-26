// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE (Stufe 0) der neuen Oberfläche mit Iced 0.14. KEIN Produktcode: belegt oder widerlegt die Tragfähigkeit (Konzept Abschnitt 14–16).
//! Aufruf: `vc-spike-iced [Projektordner] [--big] [--bench idle|tippen|scrollen|canvas|alles] [--first-frame]`
mod core;
mod highlight;
mod theme;
mod widget;
mod studio;
mod bench;
#[cfg(target_os = "macos")]
mod native;

use crate::core::{Editor, Pos};
use crate::theme::Pal;
use crate::widget::{editor_view, EdMsg};
use iced::widget::{button, column, container, image, row, scrollable, space, stack, text, text_input};
use iced::{keyboard, Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

const LOGO_ICON: &[u8] = include_bytes!("../../../branding/vintagecoder-icon.png");
const WORDMARK_FOR_DARK: &[u8] = include_bytes!("../../../branding/vintagecoder-wortmarke-transparent.png");
const WORDMARK_FOR_LIGHT: &[u8] = include_bytes!("../../../branding/vintagecoder-wortmarke-hell.png");

pub static UPDATE_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static VIEW_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq)]
pub enum Tab { File(String), Studio }

#[derive(Clone, Debug)]
pub struct Problem { pub file: String, pub line: usize, pub msg: String, pub err: bool }

#[derive(Debug, Clone)]
pub enum Msg {
    Boot, Ed(EdMsg), OpenFile(String), SelectTab(usize), CloseTab(usize), Build, BuildDone(Result<serde_json::Value, String>), GotoProblem(usize),
    Palette, PaletteQuery(String), Command(Cmd), Frame(Instant), Menu(String), Dropped(PathBuf), OpenDialog, Opened(Option<PathBuf>), NewTab, Drawer(usize),
    Studio(studio::StudioMsg), Key(keyboard::Key, keyboard::Modifiers), A11yReady,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd { Build, Theme, Save, New, Open, Wrap, Studio, BigFile }
const COMMANDS: &[(&str, Cmd, &str)] = &[("Projekt bauen", Cmd::Build, "F7"), ("Hell/Dunkel umschalten", Cmd::Theme, ""), ("Speichern", Cmd::Save, "⌘S"), ("Neuer Reiter (unbenannt)", Cmd::New, "⌘N"), ("Datei öffnen …", Cmd::Open, "⌘O"), ("Zeilenumbruch umschalten", Cmd::Wrap, ""), ("Sprite-Studio öffnen", Cmd::Studio, ""), ("Große Datei laden (50 000 Zeilen)", Cmd::BigFile, "")];

pub struct App {
    pub pal: Pal, pub project: PathBuf, pub data: PathBuf, pub files: Vec<String>, pub tabs: Vec<Tab>, pub active: usize,
    pub editors: HashMap<String, Editor>, pub drawer: usize, pub log: Vec<String>, pub problems: Vec<Problem>, pub building: bool,
    pub palette_open: bool, pub palette_query: String, pub studio: studio::Studio, pub bench: Option<bench::Bench>, pub status: String,
    pub logos: [image::Handle; 3], pub started: Instant, pub untitled: usize, pub first_frame_exit: bool, pub frames_wanted: bool,
}

fn vocab() -> Vec<&'static str> { highlight::MNEMONICS.iter().chain(highlight::DIRECTIVES.iter()).copied().collect() }

pub fn big_text() -> String {
    let mut s = String::with_capacity(1_200_000);
    for i in 0..5000 { s.push_str(&format!("; Abschnitt {i}\nrout{i}:\n\tlda #${:02x}\n\tsta $d020 ; Rahmen {i}\n\tldx #{}\nloop{i}:\n\tdex\n\tbne loop{i}\n\t.text \"hällo wörld {i}\"\n\trts\n", i % 256, i % 100)) }
    s.push_str("; Ende\n"); s
}

impl App {
    fn new() -> (App, Task<Msg>) {
        let args: Vec<String> = std::env::args().collect();
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap_or_default();
        let project = args.iter().skip(1).find(|a| !a.starts_with("--") && PathBuf::from(a).is_dir()).map(PathBuf::from).unwrap_or_else(|| repo.join("examples/c64/prallball"));
        let mut files: Vec<String> = std::fs::read_dir(&project).map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".asm") || n.ends_with(".inc") || n.ends_with(".vcprj")).collect()).unwrap_or_default();
        files.sort(); files.push("ball.sprite".into());
        let bench_kind = args.iter().position(|a| a == "--bench").and_then(|i| args.get(i + 1)).cloned();
        let mut app = App { pal: Pal::dark(), project, data: repo.join("data"), files, tabs: vec![], active: 0, editors: HashMap::new(), drawer: 0, log: vec!["Bereit. Cmd+K öffnet die Befehlspalette.".into()], problems: vec![], building: false, palette_open: false, palette_query: String::new(), studio: studio::Studio::new(), bench: None, status: String::new(), logos: [image::Handle::from_bytes(WORDMARK_FOR_DARK), image::Handle::from_bytes(WORDMARK_FOR_LIGHT), image::Handle::from_bytes(LOGO_ICON)], started: Instant::now(), untitled: 0, first_frame_exit: args.iter().any(|a| a == "--first-frame"), frames_wanted: false };
        if let Some(f) = app.files.iter().find(|f| f.ends_with(".asm")).cloned() { app.open_file(&f) }
        if args.iter().any(|a| a == "--big") { app.load_big() }
        if args.iter().any(|a| a == "--light") { app.pal = Pal::light() }
        if args.iter().any(|a| a == "--demo") { // nur zum Ansehen: Marken, Auswahl, Vervollständigung, Umbruch
            if let Some(ed) = app.cur() { ed.lines.insert(30, "\tld".into()); ed.lines.insert(31, "start:".into()); ed.lines.insert(32, "\tlda #$00 ; Rahmen".into()); ed.lines.insert(33, "\tsta $d020".into()); ed.version += 1;
                ed.breakpoints.insert(32); ed.errors.insert(33, "x".into()); ed.debug_line = Some(32); ed.set_cursor(Pos { line: 24, col: 4 }, false); ed.set_cursor(Pos { line: 26, col: 30 }, true);
                ed.set_cursor(Pos { line: 30, col: 3 }, false); ed.update_completion(&vocab()); ed.scroll_row = 18.0; ed.wrap = false; ed.folds.insert(34, 36); ed.scroll_row = 22.0; ed.set_cursor(Pos { line: 30, col: 3 }, false); ed.update_completion(&vocab()); } }
        if args.iter().any(|a| a == "--studio") { app.open_tab(Tab::Studio) }
        if let Some(k) = bench_kind { let b = bench::Bench::new(&k); if b.needs_big() { app.load_big() } app.bench = Some(b); app.frames_wanted = true; }
        if app.first_frame_exit { app.frames_wanted = true }
        (app, Task::done(Msg::Boot))
    }
    pub fn load_big(&mut self) {
        let name = "gross.asm".to_string(); let mut ed = Editor::new(&big_text()); ed.dirty = false; self.editors.insert(name.clone(), ed);
        if !self.files.contains(&name) { self.files.insert(0, name.clone()) } self.open_tab(Tab::File(name));
    }
    pub fn open_tab(&mut self, t: Tab) { if let Some(i) = self.tabs.iter().position(|x| *x == t) { self.active = i } else { self.tabs.push(t); self.active = self.tabs.len() - 1 } }
    pub fn open_file(&mut self, name: &str) {
        if name == "ball.sprite" { self.open_tab(Tab::Studio); return }
        if !self.editors.contains_key(name) {
            let text = std::fs::read_to_string(self.project.join(name)).or_else(|_| std::fs::read_to_string(name)).unwrap_or_else(|e| format!("; {name}: {e}"));
            self.editors.insert(name.into(), Editor::new(&text));
        }
        self.open_tab(Tab::File(name.into()));
    }
    pub fn cur_name(&self) -> Option<String> { match self.tabs.get(self.active) { Some(Tab::File(n)) => Some(n.clone()), _ => None } }
    pub fn cur(&mut self) -> Option<&mut Editor> { let n = self.cur_name()?; self.editors.get_mut(&n) }
    fn sync_markers(&mut self) {
        let Some(n) = self.cur_name() else { return };
        let errs: HashMap<usize, String> = self.problems.iter().filter(|p| p.file == n && p.err).map(|p| (p.line.saturating_sub(1), p.msg.clone())).collect();
        let warns: HashMap<usize, String> = self.problems.iter().filter(|p| p.file == n && !p.err).map(|p| (p.line.saturating_sub(1), p.msg.clone())).collect();
        if let Some(e) = self.editors.get_mut(&n) { e.errors = errs; e.warns = warns }
    }

    fn update(&mut self, m: Msg) -> Task<Msg> {
        let t0 = Instant::now(); let r = self.update_inner(m); UPDATE_NS.fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed); r
    }
    fn a11y(&self) { #[cfg(target_os = "macos")] native::a11y_update(self) }
    fn update_inner(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Boot => { if std::env::var("VC_SPIKE_AUTOBUILD").is_ok() { return Task::done(Msg::Build) } #[cfg(target_os = "macos")] { native::init(); return native::a11y_task() } }
            Msg::A11yReady => { self.a11y(); #[cfg(target_os = "macos")] if std::env::var("VC_SPIKE_A11Y_CHECK").is_ok() { let _ = std::fs::write("spike-a11y-baum.txt", native::self_check()); return iced::exit() } }
            Msg::Ed(e) => { self.apply_ed(e); }
            Msg::OpenFile(n) => { self.open_file(&n); self.sync_markers(); self.a11y() }
            Msg::SelectTab(i) => { self.active = i; self.sync_markers(); self.a11y() }
            Msg::CloseTab(i) => { if i < self.tabs.len() { self.tabs.remove(i); self.active = self.active.min(self.tabs.len().saturating_sub(1)); self.a11y() } }
            Msg::Drawer(i) => self.drawer = i,
            Msg::Build => return self.start_build(),
            Msg::BuildDone(r) => { self.building = false; self.finish_build(r); self.sync_markers(); if std::env::var("VC_SPIKE_AUTOBUILD").is_ok() { let mut o = self.log.join("\n"); for p in &self.problems { o.push_str(&format!("\nPROBLEM {}:{} {}", p.file, p.line, p.msg)) } for i in [0usize, 1, 40, 300, self.problems.len().saturating_sub(1)] { if i < self.problems.len() { let _ = self.update_inner(Msg::GotoProblem(i)); let p = self.problems[i].clone(); if let Some(e) = self.cur() { let r = e.row_of(e.cursor); o.push_str(&format!("\nGOTO {} -> {}:{} Cursor Zeile {} (erwartet {}), Anzeigezeile {}, scroll_row {:.1}, view_rows {}", i, p.file, p.line, e.cursor.line + 1, p.line, r, e.scroll_row, e.view_rows)) } } }
                let _ = std::fs::write("spike-build.txt", o); return iced::exit() } }
            Msg::GotoProblem(i) => { if let Some(p) = self.problems.get(i).cloned() { self.open_file(&p.file); self.sync_markers(); if let Some(e) = self.cur() { let l = p.line.saturating_sub(1).min(e.lines.len() - 1); e.set_cursor(Pos { line: l, col: 0 }, false); e.center_on(l) } } }
            Msg::Palette => { self.palette_open = !self.palette_open; self.palette_query.clear(); if self.palette_open { return iced::widget::operation::focus("palette") } }
            Msg::PaletteQuery(q) => self.palette_query = q,
            Msg::Command(c) => { self.palette_open = false; return self.run_cmd(c) }
            Msg::Menu(id) => { return match id.as_str() { "build" => self.start_build(), "save" => self.run_cmd(Cmd::Save), "new" => self.run_cmd(Cmd::New), "open" => self.run_cmd(Cmd::Open), "palette" => self.update(Msg::Palette), "theme" => self.run_cmd(Cmd::Theme), "wrap" => self.run_cmd(Cmd::Wrap), _ => Task::none() } }
            Msg::Dropped(p) => { self.log.push(format!("Datei abgelegt: {}", p.display())); self.open_path(p) }
            Msg::OpenDialog => return Task::perform(async { rfd::AsyncFileDialog::new().set_title("Datei öffnen").pick_file().await.map(|f| f.path().to_path_buf()) }, Msg::Opened),
            Msg::Opened(p) => { if let Some(p) = p { self.open_path(p) } }
            Msg::NewTab => return self.run_cmd(Cmd::New),
            Msg::Studio(s) => { let t = self.studio.update(s); return t.map(Msg::Studio) }
            Msg::Key(k, mods) => {
                if let keyboard::Key::Named(keyboard::key::Named::F7) = k { return self.start_build() }
                if let keyboard::Key::Character(c) = &k { if mods.command() { match c.as_str() { "k" => return self.update(Msg::Palette), "s" => return self.run_cmd(Cmd::Save), "n" => return self.run_cmd(Cmd::New), "o" => return self.run_cmd(Cmd::Open), _ => {} } } }
                if matches!(k, keyboard::Key::Named(keyboard::key::Named::Escape)) { self.palette_open = false }
            }
            Msg::Frame(now) => return bench::on_frame(self, now),
        }
        Task::none()
    }
    fn open_path(&mut self, p: PathBuf) {
        match std::fs::read_to_string(&p) { Ok(t) => { let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or("datei".into()); self.editors.insert(name.clone(), Editor::new(&t)); if !self.files.contains(&name) { self.files.push(name.clone()) } self.open_tab(Tab::File(name)); self.a11y() } Err(e) => self.log.push(format!("{}: {e}", p.display())) }
    }
    fn run_cmd(&mut self, c: Cmd) -> Task<Msg> {
        match c {
            Cmd::Build => return self.start_build(),
            Cmd::Theme => self.pal = if self.pal.dark { Pal::light() } else { Pal::dark() },
            Cmd::Save => { if let Some(n) = self.cur_name() { let path = self.project.join(&n); let txt = self.editors[&n].text(); match std::fs::write(&path, txt) { Ok(_) => { self.log.push(format!("Gespeichert: {n}")); self.editors.get_mut(&n).unwrap().dirty = false } Err(e) => self.log.push(format!("Speichern fehlgeschlagen: {e}")) } } }
            Cmd::New => { self.untitled += 1; let name = format!("Unbenannt {}", self.untitled); self.editors.insert(name.clone(), Editor::new("")); self.open_tab(Tab::File(name)); self.a11y() }
            Cmd::Open => return self.update(Msg::OpenDialog),
            Cmd::Wrap => { if let Some(e) = self.cur() { e.wrap = !e.wrap; e.version += 1; e.ensure_visible() } }
            Cmd::Studio => self.open_tab(Tab::Studio),
            Cmd::BigFile => self.load_big(),
        }
        Task::none()
    }
    pub fn apply_ed(&mut self, e: EdMsg) {
        let Some(ed) = self.cur() else { return };
        let vocab = vocab();
        match e {
            EdMsg::Insert(s) => { ed.insert(&s); if s.chars().count() == 1 { ed.update_completion(&vocab) } else { ed.completion = None } }
            EdMsg::Backspace => { ed.backspace(); ed.completion = None } EdMsg::Delete => ed.delete(), EdMsg::Enter => { ed.completion = None; ed.enter() } EdMsg::Tab => ed.insert("\t"),
            EdMsg::Move { dx, dy, extend } => { if dy != 0 { ed.move_v(dy, extend) } else { ed.move_h(dx, extend) } }
            EdMsg::Home(x) => ed.home(x), EdMsg::End(x) => ed.end(x),
            EdMsg::Page(d, x) => { let n = (ed.view_rows as i32 - 2).max(1) * d; ed.move_v(n, x); ed.scroll_by(n as f32) }
            EdMsg::SelectAll => ed.select_all(), EdMsg::Undo => ed.undo(), EdMsg::Redo => ed.redo(),
            EdMsg::Click(p, shift, dbl) => { ed.center_pending = None; if dbl { let ch: Vec<char> = ed.lines[p.line].chars().collect(); let (mut a, mut b) = (p.col.min(ch.len()), p.col.min(ch.len())); while a > 0 && (ch[a - 1].is_alphanumeric() || ch[a - 1] == '_') { a -= 1 } while b < ch.len() && (ch[b].is_alphanumeric() || ch[b] == '_') { b += 1 } ed.set_cursor(Pos { line: p.line, col: a }, false); ed.set_cursor(Pos { line: p.line, col: b }, true) } else { ed.set_cursor(p, shift) } }
            EdMsg::Drag(p) => { let anchor = ed.anchor.unwrap_or(ed.cursor); ed.anchor = Some(anchor); ed.cursor = p; ed.want_col = crate::core::vis_col(&ed.lines[p.line], p.col); }
            EdMsg::Breakpoint(l) => { if !ed.breakpoints.remove(&l) { ed.breakpoints.insert(l); } }
            EdMsg::Fold(l) => ed.toggle_fold(l), EdMsg::Scroll(r) => { ed.center_pending = None; ed.scroll_by(r) }
            EdMsg::Resize(c, r) => { ed.resize(c, r); ed.version += 1 }
            EdMsg::CompletionMove(d) => { if let Some(c) = &mut ed.completion { c.sel = (c.sel as i32 + d).rem_euclid(c.items.len() as i32) as usize } }
            EdMsg::CompletionAccept => { ed.accept_completion(); }
            EdMsg::CompletionClose => ed.completion = None,
            EdMsg::Preedit(p) => ed.preedit = p, EdMsg::ToggleWrap => { ed.wrap = !ed.wrap; ed.version += 1 }
        }
    }

    fn start_build(&mut self) -> Task<Msg> {
        if self.building { return Task::none() }
        let Some(rel) = self.files.iter().find(|f| f.ends_with(".asm") && *f != "gross.asm").cloned() else { return Task::none() };
        for (n, ed) in &self.editors { if ed.dirty && self.files.contains(n) && n != "gross.asm" { let _ = std::fs::write(self.project.join(n), ed.text()); } }
        self.building = true; self.drawer = 0; self.log.push(format!("Bauen: {rel} …"));
        let (root, data) = (self.project.clone(), self.data.clone());
        Task::perform(async move { tokio::task::spawn_blocking(move || vc_ide::build::build(&vc_ide::build::Ctx { root: &root, data: &data }, &rel, None)).await.unwrap_or_else(|e| Err(e.to_string())) }, Msg::BuildDone)
    }
    fn finish_build(&mut self, r: Result<serde_json::Value, String>) {
        self.problems.clear();
        match r {
            Err(e) => self.log.push(format!("Fehler: {e}")),
            Ok(v) => {
                if let Some(d) = v["diagnostics"].as_array() { for x in d { self.problems.push(Problem { file: x["file"].as_str().unwrap_or("").into(), line: x["line"].as_u64().unwrap_or(0) as usize, msg: x["message"].as_str().unwrap_or("").into(), err: x["severity"] == "error" }) } }
                match &v["log"] { serde_json::Value::Array(l) => for x in l { self.log.push(x.as_str().map(String::from).unwrap_or_else(|| x.to_string())) }, serde_json::Value::Null => {}, o => self.log.push(o.to_string().chars().take(600).collect()) }
                if v["ok"] == true { self.log.push(format!("Fertig: {} Bytes → {}", v["size"], v["artifact"])) } else { self.log.push(format!("Bauen fehlgeschlagen: {} Problem(e)", self.problems.len())); self.drawer = 1 }
            }
        }
    }

    // ------------------------------------------------------------ Ansicht
    fn view(&self) -> Element<'_, Msg> {
        let t0 = Instant::now(); let pal = self.pal;
        let header = container(row![
            image(self.logos[if pal.dark { 0 } else { 1 }].clone()).height(24),
            column![text(self.project.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()).size(14).color(pal.text), text("Projekt ▾").size(11).color(pal.mute)].spacing(1),
            chip(pal, "C64 PAL · VICE"),
            btn(pal, "▶ Starten", None, pal.ok, true), btn(pal, "● Debuggen", None, pal.text, false),
            btn(pal, if self.building { "… Baut" } else { "▣ Bauen" }, (!self.building).then_some(Msg::Build), pal.text, false),
            space::horizontal(),
            button(text("Befehl oder Datei suchen        ⌘K").size(12).color(pal.mute)).on_press(Msg::Palette).style(move |_, _| button::Style { background: Some(pal.editor.into()), border: Border { color: pal.line, width: 1.0, radius: 15.0.into() }, text_color: pal.mute, ..Default::default() }),
            btn(pal, if pal.dark { "☀" } else { "☾" }, Some(Msg::Command(Cmd::Theme)), pal.text, false),
        ].spacing(10).align_y(Alignment::Center)).padding([8, 14]).width(Length::Fill).style(move |_| panel(pal.panel, pal.line));

        let files = container(column![
            text("DATEIEN").size(10).color(pal.mute),
            column(self.files.iter().map(|f| { let sel = self.cur_name().as_deref() == Some(f.as_str()) || (f == "ball.sprite" && self.tabs.get(self.active) == Some(&Tab::Studio)); let f2 = f.clone();
                button(text(f.clone()).size(13).color(pal.text)).width(Length::Fill).padding([5, 8]).on_press(Msg::OpenFile(f2)).style(move |_, _| button::Style { background: sel.then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }).collect::<Vec<_>>()).spacing(2),
            space::vertical(), btn(pal, "+ Neu", Some(Msg::NewTab), pal.text, false),
        ].spacing(8)).width(236).height(Length::Fill).padding(10).style(move |_| panel(pal.panel, pal.line));

        let tabs = container(row(self.tabs.iter().enumerate().map(|(i, t)| {
            let (name, dirty) = match t { Tab::File(n) => (n.clone(), self.editors.get(n).map_or(false, |e| e.dirty)), Tab::Studio => ("ball.sprite".into(), false) };
            let sel = i == self.active; let label = format!("{name}{}", if dirty { " •" } else { "" });
            row![button(text(label).size(12.5).color(if sel { pal.text } else { pal.mute })).padding([6, 10]).on_press(Msg::SelectTab(i)).style(move |_, _| button::Style { background: sel.then(|| pal.editor.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }),
                 button(text("×").size(13).color(pal.mute)).padding([4, 6]).on_press(Msg::CloseTab(i)).style(move |_, _| button::Style { background: None, text_color: pal.mute, ..Default::default() })].align_y(Alignment::Center).into() }).collect::<Vec<_>>()).spacing(4)).padding([4, 8]).width(Length::Fill).style(move |_| panel(pal.panel, pal.line));

        let content: Element<Msg> = match self.tabs.get(self.active) {
            Some(Tab::File(n)) => match self.editors.get(n) { Some(ed) => container(editor_view(ed, pal, 13.0, Msg::Ed)).width(Length::Fill).height(Length::Fill).into(), None => text("").into() },
            Some(Tab::Studio) => self.studio.view(pal).map(Msg::Studio),
            None => container(column![image(self.logos[2].clone()).height(96), text("Keine Datei geöffnet. Cmd+K, Cmd+O oder links wählen.").color(pal.mute)].spacing(16).align_x(Alignment::Center)).center(Length::Fill).into(),
        };

        let drawer_body: Element<Msg> = if self.drawer == 0 {
            scrollable(column(self.log.iter().rev().take(200).rev().map(|l| text(l.clone()).size(12).font(iced::Font::MONOSPACE).color(pal.text).into()).collect::<Vec<_>>()).spacing(2).padding(8)).anchor_bottom().width(Length::Fill).into()
        } else {
            scrollable(column(self.problems.iter().enumerate().map(|(i, p)| button(text(format!("{}:{}  {}", p.file, p.line, p.msg)).size(12).font(iced::Font::MONOSPACE).color(if p.err { pal.err } else { pal.warn })).on_press(Msg::GotoProblem(i)).style(move |_, _| button::Style { background: None, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>()).spacing(2).padding(8)).width(Length::Fill).into()
        };
        let drawer = container(column![
            row![tab_btn(pal, "Protokoll", self.drawer == 0, Msg::Drawer(0)), tab_btn(pal, &format!("Probleme ({})", self.problems.len()), self.drawer == 1, Msg::Drawer(1))].spacing(6).padding([6, 10]),
            drawer_body]).height(170).width(Length::Fill).style(move |_| panel(pal.panel, pal.line));

        let (z, s) = self.tabs.get(self.active).and_then(|t| if let Tab::File(n) = t { self.editors.get(n) } else { None }).map(|e| (e.cursor.line + 1, e.cursor.col + 1)).unwrap_or((0, 0));
        let status = container(row![text(if self.building { "baut …" } else { "Bereit" }).size(11.5).color(pal.mute), space::horizontal(), text(format!("{}   Z {z} · S {s}   Assembler   UTF-8 · LF", self.status)).size(11.5).color(pal.mute)]).padding([4, 12]).width(Length::Fill).style(move |_| panel(pal.panel, pal.line));

        let main = column![header, row![files, column![tabs, content, drawer].width(Length::Fill)].height(Length::Fill), status];
        let base: Element<Msg> = container(main).style(move |_| container::Style { background: Some(pal.bg.into()), ..Default::default() }).into();
        let out = if self.palette_open {
            let q = self.palette_query.to_lowercase();
            let items = COMMANDS.iter().filter(|c| q.is_empty() || c.0.to_lowercase().contains(&q));
            let list = column(items.map(|(n, c, k)| button(row![text(*n).size(14).color(pal.text), space::horizontal(), text(*k).size(12).color(pal.mute)]).width(Length::Fill).padding([8, 12]).on_press(Msg::Command(*c)).style(move |_, st| button::Style { background: matches!(st, button::Status::Hovered).then(|| pal.sel.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into()).collect::<Vec<_>>());
            let card = container(column![text_input("Befehl oder Datei suchen …", &self.palette_query).id("palette").on_input(Msg::PaletteQuery).padding(10), list].spacing(8).width(560)).padding(14).style(move |_| container::Style { background: Some(pal.panel.into()), border: Border { color: pal.line, width: 1.0, radius: 14.0.into() }, ..Default::default() });
            let dim = container(space::vertical()).width(Length::Fill).height(Length::Fill).style(|_| container::Style { background: Some(Background::Color(Color { a: 0.5, ..Color::BLACK })), ..Default::default() });
            stack![base, dim, container(card).center_x(Length::Fill).padding(90)].into()
        } else { base };
        VIEW_NS.fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        out
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut v = vec![
            iced::event::listen_with(|e, status, _| match e {
                iced::Event::Window(iced::window::Event::FileDropped(p)) => Some(Msg::Dropped(p)),
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) if status == iced::event::Status::Ignored => Some(Msg::Key(key, modifiers)),
                _ => None }),
        ];
        #[cfg(target_os = "macos")] v.push(native::menu_subscription());
        if self.frames_wanted { v.push(iced::window::frames().map(Msg::Frame)) }
        Subscription::batch(v)
    }
}

fn panel(bg: Color, line: Color) -> container::Style { container::Style { background: Some(bg.into()), border: Border { color: line, width: 0.5, radius: 0.0.into() }, ..Default::default() } }
fn chip<'a>(pal: Pal, s: &str) -> Element<'a, Msg> { container(text(s.to_string()).size(11.5).color(pal.accent)).padding([3, 10]).style(move |_| container::Style { border: Border { color: pal.accent, width: 1.0, radius: 12.0.into() }, ..Default::default() }).into() }
fn btn<'a>(pal: Pal, s: &str, on: Option<Msg>, c: Color, primary: bool) -> Element<'a, Msg> {
    let mut b = button(text(s.to_string()).size(13).color(if primary { if pal.dark { Color::BLACK } else { Color::WHITE } } else { c })).padding([6, 12]).style(move |_, _| button::Style { background: Some(if primary { pal.ok.into() } else { pal.panel.into() }), border: Border { color: pal.line, width: 1.0, radius: 7.0.into() }, text_color: c, ..Default::default() });
    if let Some(m) = on { b = b.on_press(m) } b.into()
}
fn tab_btn<'a>(pal: Pal, s: &str, sel: bool, m: Msg) -> Element<'a, Msg> { button(text(s.to_string()).size(12.5).color(pal.text)).padding([4, 10]).on_press(m).style(move |_, _| button::Style { background: sel.then(|| pal.panel2.into()), border: Border { radius: 6.0.into(), ..Border::default() }, text_color: pal.text, ..Default::default() }).into() }

pub static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
fn main() -> iced::Result {
    let _ = START.set(Instant::now());
    iced::application(App::new, App::update, App::view)
        .title("VintageCoder – Iced-Spike")
        .subscription(App::subscription)
        .theme(|a: &App| if a.pal.dark { Theme::Dark } else { Theme::Light })
        .window(iced::window::Settings { size: iced::Size::new(1280.0, 800.0), icon: iced::window::icon::from_file_data(LOGO_ICON, None).ok(), ..Default::default() })
        .antialiasing(false)
        .run()
}
