// SPDX-License-Identifier: GPL-3.0-or-later
//! Disk-Studio: D64, D71, D81 und ATR ansehen, prüfen, bearbeiten und ins Projekt übernehmen (Phase 9 des Designkonzepts, Mockup 8).
//! Verzeichnis und Belegung sind für alle vier Formate derselbe Kern; die Besonderheiten (Spur/Sektor bzw. Sektornummer, PETSCII bzw. ATASCII,
//! Kettenzeiger) kommen aus `vc_disk::AnyDisk`.
use crate::ui::tokens::{t, tint};
use crate::ui::widgets;
use eframe::egui::{self, text::LayoutJob, Color32, RichText, TextFormat};
use std::path::{Path, PathBuf};
use vc_disk::{Addr, AnyDisk, Analysis, Cell, FileInfo, Format};
use vc_ide::diskimage::{detect_basic, import_data, BasicInfo, Image, ImportOpts};

/// Ergebnis eines Durchlaufs
pub enum Out {
    None,
    Close,
    /// Meldungen der Übernahme: (Erfolg, Text); das Studio schließt danach
    Done(Vec<(bool, String)>),
}

/// Formular „Neue Diskette“
struct NewForm { format: String, name: String, id: String, error: Option<String> }

pub struct DiskStudio {
    /// Ort der Imagedatei; `None` = neu, noch nicht gespeichert
    path: Option<PathBuf>,
    disk: Result<AnyDisk, String>,
    /// Lesesicht für Dateiinhalte und Übernahme ins Projekt
    img: Option<Image>,
    files: Vec<FileInfo>,
    an: Analysis,
    dirty: bool,
    /// gewählte Datei im Verzeichnis
    sel_file: Option<usize>,
    /// Sektor im Inspektor und besuchte Sektoren („Zurück“)
    cur: Addr,
    history: Vec<Addr>,
    checked: Vec<bool>,
    basic: Vec<Option<BasicInfo>>,
    opts: ImportOpts,
    data: Option<PathBuf>,
    /// Ergebnis von „Validieren“ (leer = in Ordnung)
    report: Option<Vec<String>>,
    note: Option<(bool, String)>,
    /// laufende Übertragung auf einen MEGA65 (Ergebnis: ok, Meldung)
    xfer: Option<std::sync::mpsc::Receiver<(bool, String)>>,
    new_form: Option<NewForm>,
    frames: u32,
    /// Projektordner: Startordner der Dateidialoge, solange das Image nicht selbst einen Ordner vorgibt
    dir: Option<PathBuf>,
    /// Die Belegungskarte soll beim ersten Zeichnen zu den belegten Spuren scrollen
    scroll_pending: bool,
}

// ---------------------------------------------------------------------------------------------------------------------
// Darstellung von Namen und Zahlen

/// PETSCII (Großschrift/Grafik-Zeichensatz) in lesbaren Text; Grafikzeichen erscheinen als »▒«
pub fn petscii_text(b: &[u8]) -> String {
    b.iter().map(|&c| match c {
        0x20..=0x40 | 0x41..=0x5A | 0x5B | 0x5D => c as char,
        0x5C => '£', 0x5E => '↑', 0x5F => '←',
        0xA0 => ' ',
        _ => '▒',
    }).collect()
}

fn atascii_text(b: &[u8]) -> String { b.iter().map(|&c| if (0x20..0x7F).contains(&c) { c as char } else { '·' }).collect() }

fn name_text(atari: bool, b: &[u8]) -> String { if atari { atascii_text(b) } else { petscii_text(b) } }

/// „3,5 KB“ wie im Mockup
fn size_text(n: usize) -> String { if n < 1024 { format!("{n} B") } else { format!("{:.1} KB", n as f64 / 1024.0).replace('.', ",") } }

fn addr_text(f: Format, a: Addr) -> String { if f.is_atari() { format!("{}", AnyDisk::atr_number(a)) } else { format!("{}/{:02}", a.track, a.sector) } }

fn geometry_text(d: &AnyDisk) -> String {
    match d.format() {
        Format::D64 => "1541 · 35 Spuren · 683 Sektoren".into(),
        Format::D71 => "1571 · 70 Spuren (2 Seiten) · 1366 Sektoren".into(),
        Format::D81 => "1581 · 80 Spuren zu 40 Sektoren · 3200 Sektoren".into(),
        Format::Atr => "Atari DOS 2 · 720 Sektoren zu 128 Byte".into(),
    }
}

fn kind_color(kind: &str) -> Color32 {
    let k = t();
    match kind { "PRG" | "XEX" => k.cy, "SEQ" | "BAS" => k.gr, _ => k.pu }
}

fn cell_color(c: Cell, files: &[FileInfo]) -> Color32 {
    let k = t();
    match c {
        Cell::Free => k.bg0,
        Cell::System => k.dim,
        Cell::Dir => k.am,
        Cell::File(i) => files.get(i).map_or(k.cy, |f| kind_color(f.kind)),
        Cell::Orphan | Cell::Conflict => k.rd,
    }
}

fn target_for(f: Format) -> &'static str { if f.is_atari() { "atari8" } else { "c64" } }

// ---------------------------------------------------------------------------------------------------------------------

impl DiskStudio {
    pub fn new(path: PathBuf, data: Option<PathBuf>) -> Self {
        let mut s = Self::empty(data);
        s.load(Some(path));
        s
    }

    fn empty(data: Option<PathBuf>) -> Self {
        DiskStudio { path: None, disk: Err(String::new()), img: None, files: vec![], an: Analysis { cells: vec![], free: 0, used: 0, problems: vec![], interleave: None },
            dirty: false, sel_file: None, cur: Addr { track: 1, sector: 0 }, history: vec![], checked: vec![], basic: vec![], opts: ImportOpts::default(), data,
            report: None, note: None, xfer: None, new_form: None, frames: 0, dir: None, scroll_pending: true }
    }

    /// Projektordner als Startordner für „Öffnen …“ und „Speichern unter …“
    pub fn with_dir(mut self, dir: Option<PathBuf>) -> Self { self.dir = dir; self }

    /// Startordner der Dateidialoge: Ordner des geöffneten Images, sonst der Projektordner
    fn start_dir(&self) -> Option<PathBuf> {
        self.path.as_ref().and_then(|p| p.parent()).filter(|d| d.is_dir()).map(Path::to_path_buf).or_else(|| self.dir.clone().filter(|d| d.is_dir()))
    }

    fn load(&mut self, path: Option<PathBuf>) {
        let Some(p) = path else { return };
        let r = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display())).and_then(|b| AnyDisk::from_file_bytes(&p.to_string_lossy(), &b));
        self.path = Some(p);
        self.dirty = false;
        self.set_disk(r);
    }

    fn set_disk(&mut self, d: Result<AnyDisk, String>) {
        self.disk = d;
        self.scroll_pending = true;
        self.report = None;
        self.sel_file = None;
        self.history.clear();
        self.refresh();
        if let Ok(d) = &self.disk { self.cur = self.default_sector(d); }
    }

    /// Erster interessanter Sektor: Kopf bzw. BAM/VTOC
    fn default_sector(&self, d: &AnyDisk) -> Addr {
        match d.format() { Format::D64 | Format::D71 => Addr { track: 18, sector: 0 }, Format::D81 => Addr { track: 40, sector: 0 }, Format::Atr => AnyDisk::atr_addr(360) }
    }

    /// Nach jeder Änderung: Belegung, Verzeichnis und Lesesicht neu berechnen
    fn refresh(&mut self) {
        let Ok(d) = &self.disk else { self.img = None; self.files.clear(); self.checked.clear(); self.basic.clear(); return };
        self.files = d.files();
        self.an = d.analyze();
        let name = self.path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| format!("neu.{}", d.format().label().to_lowercase()));
        self.img = Image::from_bytes(&name, &d.to_bytes()).ok();
        let atari = d.format().is_atari();
        self.basic = match &self.img {
            Some(i) => i.entries.iter().map(|e| match (&e.data, &self.data) { (Ok(b), Some(dd)) if e.kind == "PRG" || atari => detect_basic(dd, b, atari), _ => None }).collect(),
            None => vec![],
        };
        self.checked.resize(self.files.len(), false);
    }

    /// Alle Dateien vormerken (Selbsttest)
    pub fn select_all(&mut self) { self.checked.iter_mut().for_each(|s| *s = true); }

    /// Gewählte Dateien übernehmen; Meldungen (Erfolg, Text)
    pub fn import_selected(&self, root: &Path) -> Vec<(bool, String)> {
        let mut msgs = vec![];
        let Some(img) = &self.img else { return msgs };
        for (i, e) in img.entries.iter().enumerate().filter(|(i, _)| self.checked.get(*i).copied().unwrap_or(false)) {
            let _ = i;
            let r = match &e.data {
                Ok(d) => import_data(root, &img.suggested_name(e), d, is_program(img, &e.kind), img.atari, self.data.as_deref(), &self.opts),
                Err(m) => Err(m.clone()),
            };
            match r {
                Ok(o) => { msgs.push((true, format!("{} → {}", e.name, o.files.join(", ")))); for n in o.notes { msgs.push((true, format!("  {n}"))); } }
                Err(m) => msgs.push((false, format!("{}: {m}", e.name))),
            }
        }
        msgs
    }

    fn goto(&mut self, a: Addr) {
        if a != self.cur { self.history.push(self.cur); if self.history.len() > 200 { self.history.remove(0); } }
        self.cur = a;
    }

    fn save_to(&mut self, p: PathBuf) {
        let Ok(d) = &self.disk else { return };
        match std::fs::write(&p, d.to_bytes()) {
            Ok(()) => { self.note = Some((true, format!("gespeichert: {}", p.display()))); self.path = Some(p); self.dirty = false; self.refresh(); }
            Err(e) => self.note = Some((false, format!("{}: {e}", p.display()))),
        }
    }

    fn save_as(&mut self) {
        let Ok(d) = &self.disk else { return };
        let f = d.format().label().to_lowercase();
        let start = self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| format!("neu.{f}"));
        let mut dlg = rfd::FileDialog::new().set_title("Image speichern unter").set_file_name(start).add_filter("Diskettenimage", &[f.as_str()]);
        if let Some(dir) = self.start_dir() { dlg = dlg.set_directory(dir); }
        if let Some(p) = dlg.save_file() { self.save_to(p); }
    }

    /// Ablegen von Dateien aus dem Finder: jede wird als Datei auf die Diskette geschrieben
    fn add_dropped(&mut self, paths: Vec<PathBuf>) {
        let Ok(d) = &mut self.disk else { return };
        if !d.has_filesystem() { self.note = Some((false, "Boot-Diskette ohne Dateisystem: Dateien lassen sich hier nicht ablegen.".into())); return; }
        let mut msgs = vec![];
        let mut added = 0;
        for p in paths {
            let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let name = if d.format().is_atari() { p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default() } else { stem.chars().take(16).collect() };
            match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| d.add_file(&name, &b)) {
                Ok(n) => { added += 1; msgs.push(format!("»{n}« hinzugefügt")); }
                Err(e) => msgs.push(format!("{}: {e}", p.display())),
            }
        }
        self.note = Some((added > 0 && msgs.len() == added, msgs.join(" · ")));
        if added > 0 { self.dirty = true; self.refresh(); }
    }

    // -----------------------------------------------------------------------------------------------------------------
    // Oberfläche

    pub fn show(&mut self, ctx: &egui::Context, root: &Path) -> Out {
        self.frames += 1;
        if let Some(rx) = &self.xfer { if let Ok(r) = rx.try_recv() { self.note = Some(r); self.xfer = None; } else { ctx.request_repaint_after(std::time::Duration::from_millis(200)); } }
        self.demo();
        let mut out = Out::None;
        let mut open = true;
        let screen = ctx.content_rect().size();
        let size = egui::vec2((screen.x * 0.94).min(1320.0), (screen.y * 0.92).min(840.0));
        let title = match &self.path { Some(p) => format!("Disk-Studio – {}", p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()), None => "Disk-Studio – neue Diskette".into() };
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        if !dropped.is_empty() { self.add_dropped(dropped); }
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
        let mut do_import = false;
        let (esc, enter) = ctx.input(|i| (i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Enter)));
        let typing = ctx.egui_wants_keyboard_input();   // vor den Fenstern: ein Eingabefeld gibt bei Esc den Fokus noch im selben Bild ab
        let esc = esc_action(esc, typing, self.new_form.is_some());
        egui::Window::new(title).id(egui::Id::new("diskstudio")).open(&mut open).collapsible(false).default_size(size).resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            self.header(ui);
            ui.add_space(6.0);
            if self.disk.is_ok() {
                self.status_row(ui);
                ui.add_space(6.0);
                self.report_row(ui);
                let h = (ui.available_height() - 112.0).max(240.0);
                ui.horizontal_top(|ui| {
                    let full = ui.available_width();
                    let left = (full * 0.50).clamp(380.0, 620.0);
                    ui.allocate_ui_with_layout(egui::vec2(left, h), egui::Layout::top_down(egui::Align::Min), |ui| { ui.set_min_height(h); self.directory(ui, h); });
                    ui.add_space(8.0);
                    ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), h), egui::Layout::top_down(egui::Align::Min), |ui| { ui.set_min_height(h); self.matrix(ui, (h * 0.5).max(190.0)); ui.add_space(8.0); self.inspector(ui); });
                });
                ui.add_space(6.0);
                self.import_bar(ui, root, &mut do_import, &mut out);
            } else if let Err(e) = &self.disk {
                if e.is_empty() { ui.label(RichText::new("Keine Diskette geladen: »Neu« oder »Öffnen« wählen oder ein Image hierher ziehen.").weak()); }
                else { ui.colored_label(t().rd, e.as_str()); }
                if ui.button("Schließen").clicked() { out = Out::Close; }
            }
        });
        self.new_window(ctx, enter, esc == EscAction::CloseDialog);
        if esc == EscAction::CloseStudio { out = Out::Close; }
        if hovering {
            let r = ctx.content_rect();
            let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("diskdrop")));
            p.rect_filled(r, 0.0, t().cy.gamma_multiply(0.12));
            p.text(r.center(), egui::Align2::CENTER_CENTER, "Dateien hier ablegen – sie werden auf die Diskette geschrieben", egui::FontId::proportional(20.0), t().cy);
        }
        if !open { out = Out::Close; }
        if do_import { out = Out::Done(self.import_selected(root)); }
        out
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let (fmt, geom) = match &self.disk { Ok(d) => (Some(d.format()), geometry_text(d)), Err(_) => (None, String::new()) };
        let mut act = None;
        ui.horizontal_wrapped(|ui| {
            if let Some(f) = fmt {
                widgets::chip(ui, f.label(), widgets::system_color(target_for(f)).gamma_multiply(1.15));
                let name = self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "(neu, nicht gespeichert)".into());
                ui.label(RichText::new(name).font(egui::FontId::new(15.0, crate::fonts::ui_strong())));
                if self.dirty { widgets::chip(ui, "geändert", k.am); }
                ui.label(RichText::new(geom).weak());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(f) = fmt {
                    let label = if f.is_atari() { "Im Emulator einlegen (D1:)" } else { "Im Emulator einlegen (Laufwerk 8)" };
                    if ui.add_enabled(self.path.is_some() && !self.dirty, egui::Button::new(label)).on_hover_text(if self.path.is_none() || self.dirty { "Erst speichern, dann einlegen" } else { "Startet den Standard-Emulator des Systems mit dieser Diskette; ein früher gestarteter Emulator wird beendet" }).clicked() { act = Some("mount"); }
                    let is_d81 = self.path.as_ref().and_then(|p| p.extension()).map_or(false, |e| e.eq_ignore_ascii_case("d81"));
                    if is_d81 {
                        let busy = self.xfer.is_some();
                        if ui.add_enabled(self.path.is_some() && !self.dirty && !busy, egui::Button::new(if busy { "überträgt …" } else { "Auf MEGA65 kopieren" }))
                            .on_hover_text("Kopiert die Diskette per Ethernet (mega65_ftp) auf die SD-Karte des MEGA65 und bindet sie ein (etherload). UNGEPRÜFT: noch nicht an einem Gerät getestet. Maschine und Ordner unter Einstellungen → Hardware.").clicked() { act = Some("m65"); }
                    }
                    if ui.button("Validieren").on_hover_text("BAM bzw. VTOC, Verzeichnis und Dateiketten prüfen").clicked() { act = Some("validate"); }
                    if ui.button("Exportieren …").on_hover_text("Image als Kopie unter neuem Namen speichern").clicked() { act = Some("export"); }
                    if ui.add_enabled(self.dirty && self.path.is_some(), egui::Button::new("Speichern")).clicked() { act = Some("save"); }
                }
                if ui.button("Öffnen …").clicked() { act = Some("open"); }
                if ui.button("Neu …").clicked() { act = Some("new"); }
            });
        });
        if let Some((ok, m)) = &self.note { ui.label(RichText::new(m.as_str()).small().color(if *ok { k.gr } else { k.rd })); }
        match act {
            Some("new") => self.new_form = Some(NewForm { format: "d64".into(), name: "DISK".into(), id: "01".into(), error: None }),
            Some("open") => {
                let mut dlg = rfd::FileDialog::new().set_title("Diskettenimage öffnen").add_filter("Diskettenimage", &["d64", "d71", "d81", "atr", "D64", "D71", "D81", "ATR"]);
                if let Some(dir) = self.start_dir() { dlg = dlg.set_directory(dir); }
                if let Some(p) = dlg.pick_file() { self.load(Some(p)); }
            }
            Some("m65") => {
                if let Some(p) = self.path.clone() {
                    let st = vc_ide::settings::effective();
                    match st.machines.iter().find(|m| m.kind == "mega65" && m.link_for(vc_ide::mega65net::Action::Copy) == vc_ide::mega65net::Link::Eth).cloned() {
                        None => self.note = Some((false, "Keine MEGA65 mit Ethernet eingetragen (Einstellungen → Hardware → Verbindung „Kopieren“)".into())),
                        Some(m) => {
                            let (tx, rx) = std::sync::mpsc::channel(); let c = ui.ctx().clone();
                            self.xfer = Some(rx);
                            std::thread::spawn(move || { let r = vc_ide::mega65net::copy_and_mount(&m, &p); let _ = tx.send(match r { Ok(t) => (true, t), Err(e) => (false, e) }); c.request_repaint(); });
                        }
                    }
                }
            }
            Some("save") => { if let Some(p) = self.path.clone() { self.save_to(p); } }
            Some("export") => self.save_as(),
            Some("validate") => { if let Ok(d) = &self.disk { self.report = Some(d.check()); } }
            Some("mount") => {
                if let (Some(p), Some(f)) = (self.path.clone(), fmt) {
                    let variant = p.parent().map(vc_ide::launch::atari_variant).unwrap_or_else(|| "pal".into());
                    self.note = Some(match vc_ide::launch::insert_disk(&p, target_for(f), &variant) { Ok(cmd) => (true, format!("eingelegt: {cmd}")), Err(e) => (false, e) });
                }
            }
            _ => {}
        }
    }

    fn status_row(&mut self, ui: &mut egui::Ui) {
        let (Ok(d), k) = (&self.disk, t()) else { return };
        let atari = d.format().is_atari();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            widgets::chip(ui, &format!("{} {} frei", d.free_blocks(), if atari { "Sektoren" } else { "Blöcke" }), k.gr);
            widgets::chip(ui, &format!("{} belegt", self.an.used), k.cy);
            let errs = self.an.problems.len();
            widgets::chip(ui, &format!("{errs} Fehler"), if errs == 0 { k.mute } else { k.rd });
            widgets::chip(ui, &match self.an.interleave { Some(i) => format!("Interleave {i}"), None => "Interleave –".into() }, k.am);
            let title = d.title();
            if !atari && !title.is_empty() { ui.label(RichText::new(format!("0 \"{}\"", petscii_text(&title))).monospace().weak()); }
            if atari { ui.label(RichText::new("Boot-Sektoren 1–3").small().weak()); }
        });
    }

    fn report_row(&mut self, ui: &mut egui::Ui) {
        let Some(r) = &self.report else { return };
        let k = t();
        let mut close = false;
        egui::Frame::new().fill(tint(if r.is_empty() { k.gr } else { k.rd })).corner_radius(4.0).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if r.is_empty() { ui.label(RichText::new("Validierung: keine Fehler – BAM, Verzeichnis und Ketten stimmen überein.").color(k.gr)); }
                else { ui.label(RichText::new(format!("Validierung: {} Problem(e)", r.len())).strong().color(k.rd)); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("✕").clicked() { close = true; } });
            });
            if !r.is_empty() {
                egui::ScrollArea::vertical().max_height(90.0).id_salt("disk_report").show(ui, |ui| { for m in r { ui.label(RichText::new(m).small().monospace()); } });
            }
        });
        if close { self.report = None; }
        ui.add_space(6.0);
    }

    fn directory(&mut self, ui: &mut egui::Ui, h: f32) {
        let k = t();
        let Ok(d) = &self.disk else { return };
        let (atari, fmt) = (d.format().is_atari(), d.format());
        widgets::section(ui, "Verzeichnis", if atari { "Namen in ATASCII · Datei auf die Fläche ziehen zum Hinzufügen" } else { "Namen in PETSCII · Datei auf die Fläche ziehen zum Hinzufügen" }, |_| {});
        if !d.has_filesystem() {
            egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                widgets::chip(ui, "Boot-Diskette ohne Dateisystem", k.am);
                ui.add_space(4.0);
                ui.label("Die Diskette enthält ein Boot-Programm, aber kein DOS-2-Verzeichnis. Es gibt deshalb keine Dateien, die sich auflisten, lesen oder übernehmen lassen. Die Belegungskarte zeigt die Sektoren mit Inhalt.");
            });
            return;
        }
        let mut pick = None;
        egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(6)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::both().id_salt("disk_dir").max_height(h - 74.0).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("disk_dir_grid").num_columns(8).spacing([12.0, 4.0]).striped(true).show(ui, |ui| {
                    for h in ["", "Blk", "Name", "Typ", if atari { "Sektor" } else { "Start T/S" }, "Größe", "Lade", ""] { ui.label(RichText::new(h).small().color(k.dim)); }
                    ui.end_row();
                    for (i, f) in self.files.iter().enumerate() {
                        let usable = self.img.as_ref().and_then(|im| im.entries.get(i)).map_or(false, |e| e.data.is_ok());
                        if let Some(c) = self.checked.get_mut(i) { ui.add_enabled(usable, egui::Checkbox::without_text(c)).on_hover_text("Für die Übernahme ins Projekt vormerken"); } else { ui.label(""); }
                        ui.label(RichText::new(f.blocks.to_string()).monospace().color(k.mute));
                        let sel = self.sel_file == Some(i);
                        let nm = name_text(atari, &f.name);
                        let r = ui.add(egui::Label::new(RichText::new(if atari { nm } else { format!("\"{nm}\"") }).monospace().color(if sel { k.cy } else { k.fg })).selectable(false).sense(egui::Sense::click()));
                        if sel { ui.painter().rect_stroke(r.rect.expand(2.0), 3.0, egui::Stroke::new(1.0, k.cy.gamma_multiply(0.6)), egui::StrokeKind::Outside); }
                        if r.clicked() { pick = Some(i); }
                        r.on_hover_text(format!("Rohbytes: {}", f.name.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")));
                        ui.horizontal(|ui| { ui.spacing_mut().item_spacing.x = 4.0; widgets::mono_chip(ui, f.kind, kind_color(f.kind)); if f.locked { ui.label(RichText::new("🔒").small()); } });
                        ui.label(RichText::new(addr_text(fmt, f.start)).monospace().color(k.mute));
                        let e = self.img.as_ref().and_then(|im| im.entries.get(i));
                        ui.label(RichText::new(match e { Some(e) if e.data.is_ok() => size_text(e.size()), Some(_) => "Fehler".into(), None => "–".into() }).small().color(if usable { k.mute } else { k.rd }));
                        ui.label(RichText::new(e.and_then(|e| e.load_address()).map(|a| format!("${a:04X}")).unwrap_or_default()).monospace().small().color(k.dim));
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            if f.autostart { widgets::chip(ui, "Autostart", k.gr); }
                            if self.basic.get(i).map_or(false, |b| b.is_some()) { widgets::chip(ui, "BASIC", k.am); }
                            if let Some(Err(m)) = e.map(|e| &e.data) { ui.label(RichText::new(m.as_str()).small().color(k.rd)); }
                        });
                        ui.end_row();
                    }
                });
                if self.files.is_empty() { ui.add_space(6.0); ui.label(RichText::new("Die Diskette enthält keine Dateien.").weak()); }
            });
        });
        if let Some(i) = pick { self.sel_file = Some(i); let s = self.files[i].start; self.history.clear(); self.cur = s; }
    }

    fn matrix(&mut self, ui: &mut egui::Ui, h: f32) {
        let k = t();
        let Ok(d) = &self.disk else { return };
        let atari = d.format().is_atari();
        widgets::section(ui, "Belegung", if atari { "Sektoren in 40 Spuren zu 18 · Klick wählt den Sektor" } else { "Spuren × Sektoren · Klick wählt den Sektor" }, |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for (c, n) in [(k.bg0, "frei"), (k.cy, "PRG/XEX"), (k.gr, "SEQ/BAS"), (k.pu, "andere"), (k.am, "Verzeichnis"), (k.dim, "System"), (k.rd, "Fehler")] {
                ui.label(RichText::new(n).small().color(k.mute));
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 2.0, c);
                if c == k.bg0 { ui.painter().rect_stroke(r, 2.0, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside); }
            }
        });
        let tracks = d.tracks();
        let cols = (1..=tracks).map(|t| d.sectors_in_track(t)).max().unwrap_or(1);
        let sel_cells: Option<usize> = self.sel_file;
        let mut clicked = None;
        let mut hover = None;
        egui::Frame::new().fill(k.bg2).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(6)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical().id_salt("disk_matrix").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
                let label_w = 26.0;
                let cs = ((ui.available_width() - label_w - 4.0) / cols as f32).clamp(7.0, 16.0);
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(label_w + cols as f32 * cs, tracks as f32 * cs), egui::Sense::click());
                // Beim ersten Zeichnen zu den belegten Spuren scrollen (bei D64 liegt das Verzeichnis auf Spur 18, bei kleiner Karte außerhalb des Sichtbereichs)
                if self.scroll_pending {
                    self.scroll_pending = false;
                    if let Some(tr) = (1..=tracks).find(|&tr| self.an.cells[tr - 1].iter().any(|c| *c != Cell::Free)) {
                        let y = rect.top() + (tr - 1) as f32 * cs;
                        ui.scroll_to_rect(egui::Rect::from_min_size(egui::pos2(rect.left(), y), egui::vec2(rect.width(), cs * 3.0)), Some(egui::Align::Center));
                    }
                }
                let p = ui.painter_at(rect);
                for tr in 1..=tracks {
                    let y = rect.top() + (tr - 1) as f32 * cs;
                    if cs >= 9.0 || tr % 5 == 0 || tr == 1 { p.text(egui::pos2(rect.left() + label_w - 4.0, y + cs / 2.0), egui::Align2::RIGHT_CENTER, tr.to_string(), egui::FontId::monospace((cs * 0.75).min(10.0)), k.dim); }
                    for s in 0..d.sectors_in_track(tr) {
                        let c = self.an.cells[tr - 1][s];
                        let r = egui::Rect::from_min_size(egui::pos2(rect.left() + label_w + s as f32 * cs, y), egui::vec2(cs, cs)).shrink(1.0);
                        let dim = sel_cells.map_or(false, |sf| !matches!(c, Cell::File(i) if i == sf));
                        let mut col = cell_color(c, &self.files);
                        if c != Cell::Free && dim { col = col.gamma_multiply(0.55); }
                        p.rect_filled(r, 1.5, col);
                        if c == Cell::Free { p.rect_stroke(r, 1.5, egui::Stroke::new(1.0, k.line), egui::StrokeKind::Inside); }
                        if (Addr { track: tr, sector: s }) == self.cur { p.rect_stroke(r.expand(1.0), 2.0, egui::Stroke::new(2.0, k.fg), egui::StrokeKind::Outside); }
                    }
                }
                if let Some(pos) = resp.interact_pointer_pos().or_else(|| resp.hover_pos()) {
                    let (x, y) = (pos.x - rect.left() - label_w, pos.y - rect.top());
                    if x >= 0.0 && y >= 0.0 {
                        let (tr, s) = ((y / cs) as usize + 1, (x / cs) as usize);
                        if tr <= tracks && s < d.sectors_in_track(tr) { let a = Addr { track: tr, sector: s }; hover = Some(a); if resp.clicked() { clicked = Some(a); } }
                    }
                }
            });
        });
        if let Some(a) = hover {
            if let Some(c) = self.an.cell(a) {
                let what = match c {
                    Cell::Free => "frei".to_string(),
                    Cell::System => if atari { "Boot / VTOC".into() } else { "Kopf / BAM".into() },
                    Cell::Dir => "Verzeichnis".into(),
                    Cell::File(i) => self.files.get(i).map(|f| format!("{} ({})", name_text(atari, &f.name), f.kind)).unwrap_or_default(),
                    Cell::Orphan => "belegt, aber ohne Datei".into(),
                    Cell::Conflict => "von mehreren Ketten erreicht".into(),
                };
                ui.label(RichText::new(format!("{} · {what}", if atari { format!("Sektor {}", AnyDisk::atr_number(a)) } else { format!("Spur {} Sektor {}", a.track, a.sector) })).small().color(k.mute));
            }
        } else { ui.label(RichText::new(" ").small()); }
        if let Some(a) = clicked { if let Some(Cell::File(i)) = self.an.cell(a) { self.sel_file = Some(i); } self.goto(a); }
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        let k = t();
        let Ok(d) = &self.disk else { return };
        let atari = d.format().is_atari();
        let cur = self.cur;
        let next = d.next(cur, &self.an);
        let mut go = None;
        let mut back = false;
        let title = if atari { format!("Sektor {}", AnyDisk::atr_number(cur)) } else { format!("Sektor {}/{}", cur.track, cur.sector) };
        widgets::section(ui, "Sektor-Inspektor", &title, |ui| {
            if ui.add_enabled(next.is_some(), egui::Button::new(match next { Some(n) => format!("Folgen ▶ {}", addr_text(d.format(), n)), None => "Folgen ▶ (Kettenende)".into() })).on_hover_text(if atari { "Dem Folgesektor in den Bytes 125/126 nachgehen" } else { "Dem Zeiger Spur/Sektor in den Bytes 0/1 nachgehen" }).clicked() { go = next; }
            if ui.add_enabled(!self.history.is_empty(), egui::Button::new("◀ Zurück")).clicked() { back = true; }
        });
        let Some(data) = d.sector(cur) else { ui.label(RichText::new("Sektor außerhalb der Diskette").color(k.rd)); return };
        let owner = match self.an.cell(cur) {
            Some(Cell::File(i)) => self.files.get(i).map(|f| format!("gehört zu {}", name_text(atari, &f.name))),
            Some(Cell::Dir) => Some("Verzeichnis".into()), Some(Cell::System) => Some(if atari { "Boot bzw. VTOC".into() } else { "Kopf / BAM".into() }),
            Some(Cell::Free) => Some("frei laut BAM".into()), Some(Cell::Orphan) => Some("belegt ohne Datei".into()), Some(Cell::Conflict) => Some("Konflikt: mehrfach belegt".into()), None => None,
        };
        let ptr: std::ops::Range<usize> = if atari { 125..128 } else { 0..2 };
        let ptr_note = if atari {
            let n = ((data[125] & 3) as usize) << 8 | data[126] as usize;
            format!("Datei-Nr. {} · Folgesektor {} · {} Datenbytes", data[125] >> 2, if n == 0 { "–".into() } else { n.to_string() }, data[127])
        } else if data[0] == 0 { format!("Kettenende · letztes Byte {}", data[1]) } else { format!("Zeiger {}/{}", data[0], data[1]) };
        ui.horizontal_wrapped(|ui| {
            if let Some(o) = &owner { widgets::chip(ui, o, k.cy); }
            ui.label(RichText::new(ptr_note).small().color(k.mute));
        });
        egui::Frame::new().fill(k.bg0).stroke(egui::Stroke::new(1.0, k.line)).corner_radius(4.0).inner_margin(egui::Margin::same(6)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical().id_salt("disk_hex").auto_shrink([false, false]).show(ui, |ui| {
                let mono = egui::FontId::monospace(12.5);
                for (row, chunk) in data.chunks(16).enumerate() {
                    let mut job = LayoutJob::default();
                    let f = |c: Color32| TextFormat { font_id: mono.clone(), color: c, ..Default::default() };
                    job.append(&format!("{:02X}  ", row * 16), 0.0, f(k.dim));
                    for (i, b) in chunk.iter().enumerate() {
                        let idx = row * 16 + i;
                        let col = if ptr.contains(&idx) { k.am } else if *b == 0 { k.dim } else { k.fg };
                        job.append(&format!("{b:02X} "), 0.0, f(col));
                        if i == 7 { job.append(" ", 0.0, f(k.dim)); }
                    }
                    job.append(" ", 0.0, f(k.dim));
                    let txt: String = chunk.iter().map(|&b| if (0x20..0x7F).contains(&b) { b as char } else { '·' }).collect();
                    job.append(&txt, 0.0, f(k.mute));
                    ui.label(job);
                }
            });
        });
        if back { if let Some(a) = self.history.pop() { self.cur = a; } }
        if let Some(a) = go { self.goto(a); }
    }

    fn import_bar(&mut self, ui: &mut egui::Ui, root: &Path, do_import: &mut bool, out: &mut Out) {
        let k = t();
        let Some(img) = &self.img else { if ui.button("Schließen").clicked() { *out = Out::Close; } return };
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("In das Projekt übernehmen").strong());
            if ui.small_button("Alle").clicked() { self.checked.iter_mut().for_each(|s| *s = true); }
            if ui.small_button("Keine").clicked() { self.checked.iter_mut().for_each(|s| *s = false); }
            if ui.small_button("Nur Programme").clicked() { for (i, e) in img.entries.iter().enumerate() { self.checked[i] = is_program(img, &e.kind) && e.data.is_ok(); } }
            if ui.small_button("Nur BASIC").clicked() { for i in 0..self.checked.len() { self.checked[i] = self.basic.get(i).map_or(false, |b| b.is_some()); } }
            ui.label("Zielordner");
            ui.add(egui::TextEdit::singleline(&mut self.opts.dir).desired_width(110.0).hint_text("import"));
            let any_basic = self.checked.iter().zip(&self.basic).any(|(s, b)| *s && b.is_some());
            let any_prog = img.entries.iter().enumerate().any(|(i, e)| is_program(img, &e.kind) && self.checked.get(i).copied().unwrap_or(false));
            ui.add_enabled(any_basic || self.basic.iter().any(|b| b.is_some()), egui::Checkbox::new(&mut self.opts.basic, "BASIC als Quelltext")).on_hover_text("Das tokenisierte Programm wird in lesbaren, nummerierten Text zurückübersetzt");
            ui.add_enabled(self.opts.basic && any_basic, egui::Checkbox::new(&mut self.opts.raw, "Original behalten"));
            ui.add_enabled(any_prog || img.entries.iter().any(|e| is_program(img, &e.kind)), egui::Checkbox::new(&mut self.opts.asm, "zusätzlich als Assembler")).on_hover_text("Rein mechanisch: Daten erscheinen als Befehle und müssen von Hand geprüft werden");
        });
        let n = self.checked.iter().filter(|s| **s).count();
        let bad = !self.opts.dir.is_empty() && (self.opts.dir.starts_with('/') || self.opts.dir.contains("..") || self.opts.dir.contains('\\'));
        if bad { ui.colored_label(k.rd, "Der Zielordner muss innerhalb des Projekts liegen."); }
        let _ = root;
        ui.horizontal(|ui| {
            if ui.add_enabled(n > 0 && !bad, egui::Button::new(format!("{n} Datei(en) übernehmen"))).clicked() { *do_import = true; }
            if ui.button("Schließen").clicked() { *out = Out::Close; }
            if self.dirty { ui.label(RichText::new("Ungespeicherte Änderungen am Image gehen beim Schließen verloren.").small().color(k.am)); }
        });
    }

    /// Kleines Fenster „Neue Diskette“
    fn new_window(&mut self, ctx: &egui::Context, enter: bool, esc: bool) {
        let Some(mut f) = self.new_form.take() else { return };
        let (mut create, mut cancel) = (enter, esc);
        egui::Window::new("Neue Diskette").id(egui::Id::new("disknew")).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            widgets::segmented(ui, &mut f.format, &[("d64", "D64 · 1541"), ("d71", "D71 · 1571"), ("d81", "D81 · 1581"), ("atr", "ATR · Atari DOS 2")]);
            ui.add_space(6.0);
            if f.format != "atr" {
                ui.horizontal(|ui| { widgets::label(ui, "Name"); ui.add(egui::TextEdit::singleline(&mut f.name).desired_width(170.0).char_limit(16)); });
                ui.horizontal(|ui| { widgets::label(ui, "ID"); ui.add(egui::TextEdit::singleline(&mut f.id).desired_width(40.0).char_limit(2)); });
            } else { ui.label(RichText::new("DOS 2.0S, 720 Sektoren zu 128 Byte (90K), leer formatiert.").small().weak()); }
            if let Some(e) = &f.error { ui.colored_label(t().rd, e.as_str()); }
            ui.add_space(6.0);
            ui.horizontal(|ui| { if ui.button("Anlegen").clicked() { create = true; } if ui.button("Abbrechen").clicked() { cancel = true; } });
        });
        if create {
            let fmt = match f.format.as_str() { "d71" => Format::D71, "d81" => Format::D81, "atr" => Format::Atr, _ => Format::D64 };
            match AnyDisk::blank(fmt, &f.name, &f.id) {
                Ok(d) => { self.path = None; self.dirty = true; self.note = None; self.set_disk(Ok(d)); return; }
                Err(e) => f.error = Some(e),
            }
        }
        if !cancel { self.new_form = Some(f); }
    }

    /// Selbsttests (`VC_DEMO`): `diskfollow` folgt der Kette der ersten Datei, `disksel` wählt sie, `diskval` validiert, `disknew` zeigt „Neu“
    fn demo(&mut self) {
        let Ok(demo) = std::env::var("VC_DEMO") else { return };
        if std::env::var("VC_SHOT").is_err() || !demo.contains("disk") { return; }
        if self.frames == 4 && demo.contains("disksel") { if let Some(f) = self.files.first() { self.sel_file = Some(0); self.cur = f.start; } }
        if self.frames == 4 && demo.contains("diskfile") { if let Some(i) = std::env::var("VC_DISK_FILE").ok().and_then(|v| v.parse::<usize>().ok()) { if let Some(f) = self.files.get(i) { self.sel_file = Some(i); self.cur = f.start; } } }
        if self.frames == 5 && demo.contains("diskfollow") { for _ in 0..2 { if let Ok(d) = &self.disk { if let Some(n) = d.next(self.cur, &self.an) { self.goto(n); } } } }
        if self.frames == 5 && demo.contains("diskval") { if let Ok(d) = &self.disk { self.report = Some(d.check()); } }
        if self.frames == 5 && demo.contains("disknew") { self.new_form = Some(NewForm { format: "d81".into(), name: "DISK".into(), id: "01".into(), error: None }); }
    }
}

/// Was Esc im Studio bewirkt
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum EscAction { Nothing, CloseDialog, CloseStudio }

/// Esc schließt zuerst den offenen Dialog („Neue Diskette“), sonst das Studio – aber nie, solange in einem Eingabefeld getippt wird
fn esc_action(pressed: bool, typing: bool, dialog_open: bool) -> EscAction {
    if !pressed { EscAction::Nothing } else if dialog_open { EscAction::CloseDialog } else if typing { EscAction::Nothing } else { EscAction::CloseStudio }
}

fn is_program(img: &Image, kind: &str) -> bool { kind == "PRG" || (img.atari && kind == "DOS") }

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let tmp = std::env::temp_dir().join(format!("vc_diskstudio_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        tmp
    }
    fn frames(s: &mut DiskStudio, root: &Path) {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput::default(), |ui| { assert!(matches!(s.show(ui.ctx(), root), Out::None)); }); o.textures_delta.clear(); }
    }

    #[test]
    fn studio_zeichnet_und_uebernimmt() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let tmp = tmpdir("c64");
        // D64 mit BASIC-Programm (10 SYS2061 + Maschinencode) und reinem Maschinenprogramm
        let stub = [0x01u8, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x9E, b'2', b'0', b'6', b'1', 0x00, 0x00, 0x00, 0xA9, 0x00, 0x60];
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap();
        d.add_prg("STUB", &stub).unwrap(); d.add_prg("MC", &[0x00, 0xC0, 0xEE, 0x20, 0xD0, 0x60]).unwrap();
        std::fs::write(tmp.join("t.d64"), d.to_bytes()).unwrap();
        let mut dlg = DiskStudio::new(tmp.join("t.d64"), Some(data));
        assert_eq!(dlg.basic.iter().map(|b| b.is_some()).collect::<Vec<_>>(), vec![true, false]);
        assert_eq!(dlg.files.iter().map(|f| f.autostart).collect::<Vec<_>>(), vec![true, false]);
        dlg.select_all();
        frames(&mut dlg, &tmp);
        let msgs = dlg.import_selected(&tmp);
        assert!(msgs.iter().all(|m| m.0), "{msgs:?}");
        for f in ["import/stub.bas", "import/stub.prg", "import/mc.prg"] { assert!(tmp.join(f).exists(), "{f} fehlt: {msgs:?}"); }
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn esc_schliesst_zuerst_den_dialog_dann_das_studio_aber_nicht_beim_tippen() {
        assert_eq!(esc_action(false, false, false), EscAction::Nothing);
        assert_eq!(esc_action(true, false, false), EscAction::CloseStudio);
        assert_eq!(esc_action(true, true, false), EscAction::Nothing);
        assert_eq!(esc_action(true, true, true), EscAction::CloseDialog);
        assert_eq!(esc_action(true, false, true), EscAction::CloseDialog);
    }

    #[test]
    fn dateidialoge_starten_im_projektordner_und_boot_disk_hat_kein_dateisystem() {
        let tmp = tmpdir("dir");
        let proj = tmp.join("projekt"); std::fs::create_dir_all(&proj).unwrap();
        // neue, noch nicht gespeicherte Diskette: Projektordner
        let n = DiskStudio::empty(None).with_dir(Some(proj.clone()));
        assert_eq!(n.start_dir(), Some(proj.clone()));
        assert_eq!(DiskStudio::empty(None).start_dir(), None, "ohne Projekt kein Vorgabeordner");
        // geöffnetes Image: dessen Ordner
        let other = tmp.join("anders"); std::fs::create_dir_all(&other).unwrap();
        let mut a = vc_disk::Atr::blank(); a.sector_mut(1)[0] = 0x55;   // Boot-Diskette ohne DOS-2-VTOC
        std::fs::write(other.join("b.atr"), a.to_bytes()).unwrap();
        let s = DiskStudio::new(other.join("b.atr"), None).with_dir(Some(proj));
        assert_eq!(s.start_dir(), Some(other));
        assert!(s.disk.as_ref().map_or(false, |d| !d.has_filesystem()));
        assert!(s.files.is_empty() && s.an.problems.is_empty());
        assert!(s.scroll_pending, "Belegungskarte scrollt beim ersten Zeichnen zu den belegten Spuren");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn atari_und_neu_ablegen_speichern() {
        let tmp = tmpdir("atr");
        let mut a = vc_disk::Atr::new_dos2();
        a.dos2_add("AUTORUN.SYS", &[0xFF, 0xFF, 0, 0x20, 0x02, 0x20, 0xA9, 0x00, 0x60]).unwrap();
        std::fs::write(tmp.join("t.atr"), a.to_bytes()).unwrap();
        let mut s = DiskStudio::new(tmp.join("t.atr"), None);
        assert_eq!(s.files.len(), 1);
        frames(&mut s, &tmp);
        // Datei ablegen: Name wird auf DOS-2-Regeln gebracht, Image ist danach gültig
        std::fs::write(tmp.join("level one.dat"), vec![7u8; 400]).unwrap();
        s.add_dropped(vec![tmp.join("level one.dat")]);
        assert!(s.dirty && s.files.len() == 2 && s.an.problems.is_empty(), "{:?} {:?}", s.note, s.an.problems);
        s.save_to(tmp.join("t.atr"));
        assert!(!s.dirty);
        assert_eq!(vc_disk::Atr::from_bytes(&std::fs::read(tmp.join("t.atr")).unwrap()).unwrap().dos2_entries().len(), 2);
        // neue D81 anlegen, Datei ablegen, Kette folgen
        let mut n = DiskStudio::empty(None);
        n.set_disk(AnyDisk::blank(Format::D81, "NEU", "01"));
        std::fs::write(tmp.join("prog.prg"), vec![1u8; 1000]).unwrap();
        n.add_dropped(vec![tmp.join("prog.prg")]);
        assert_eq!(n.files.len(), 1);
        frames(&mut n, &tmp);
        n.cur = n.files[0].start;
        let d = n.disk.clone().unwrap();
        let mut hops = 0;
        while let Some(x) = d.next(n.cur, &n.an) { n.goto(x); hops += 1; }
        assert_eq!(hops, 3);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn namen_petscii_und_atascii() {
        assert_eq!(petscii_text(b"HALLO 1"), "HALLO 1");
        assert_eq!(petscii_text(&[0x41, 0xC1, 0xA0]), "A▒ ");
        assert_eq!(atascii_text(&[b'A', 0x9B]), "A·");
        assert_eq!(size_text(3584), "3,5 KB");
    }
}
