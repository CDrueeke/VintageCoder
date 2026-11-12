// SPDX-License-Identifier: GPL-3.0-or-later
//! Oberfläche zum ROM-Verzeichnis außerhalb der Einstellungsseite: Assistent „ROMs einrichten“ (3 Schritte), Fehlerdialog „ROM fehlt“,
//! Karte für die Willkommensseite und Hinweisband für den Projekt-Wizard. Der Kern steht in `vc_roms`; die Seite „ROMs“ der Einstellungen
//! (`settings_ui::roms`) liefert Vorschau und Kopieren, die hier mitbenutzt werden.
use crate::settings_ui::roms::{preview_card, push_drops_flag, run_import, take_drops, Preview};
use crate::theme::{bad, ok, warn};
use crate::ui::widgets::{card, chip};
use eframe::egui::{self, RichText};
use vc_roms::{import, systems, RomStore, Status};

/// Zeitgrenze der Suche in ROM-Ordnern anderer Programme
const SEARCH_LIMIT: std::time::Duration = std::time::Duration::from_secs(45);

/// Suche in den ROM-Ordnern anderer Programme im Hintergrund: abbrechbar, mit Zeitgrenze, liefert (Funde, vollständig?)
pub struct SearchJob {
    rx: std::sync::mpsc::Receiver<(Vec<import::ImportItem>, bool)>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    started: std::time::Instant,
}

impl SearchJob {
    pub fn start(env: vc_roms::RomEnv) -> SearchJob {
        let (tx, rx) = std::sync::mpsc::channel();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = stop.clone();
        std::thread::spawn(move || {
            let t0 = std::time::Instant::now();
            // der lokale Stand wird nur für „schon vorhanden“ und das Ziel gebraucht, fremde Orte hier nicht doppelt einlesen
            let store = RomStore::new(vc_roms::RomEnv { use_foreign: false, ..env });
            let halt = || flag.load(std::sync::atomic::Ordering::Relaxed) || t0.elapsed() > SEARCH_LIMIT;
            let items = import::foreign_candidates_until(&store, &halt);
            let complete = !halt();
            let _ = tx.send((items, complete));
        });
        SearchJob { rx, stop, started: std::time::Instant::now() }
    }
    /// Ergebnis, sobald es da ist
    pub fn poll(&self) -> Option<(Vec<import::ImportItem>, bool)> { self.rx.try_recv().ok() }
    pub fn cancel(&self) { self.stop.store(true, std::sync::atomic::Ordering::Relaxed); }
}

/// Fortschrittszeile der Suche mit „Abbrechen“; liefert `true`, wenn abgebrochen werden soll
pub(crate) fn search_progress(ui: &mut egui::Ui, job: &SearchJob) -> bool {
    let mut cancel = false;
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(format!("Suche in anderen Programmen läuft … {} s", job.started.elapsed().as_secs()));
        if ui.button("Abbrechen").clicked() { cancel = true; }
    });
    cancel
}

/// Rahmen für Dialoge: Innenabstand, kräftiger Schatten und Linie, damit sie sich auch im hellen Farbschema vom Hintergrund abheben
fn dialog_frame(ctx: &egui::Context) -> egui::Frame {
    let k = crate::ui::tokens::t();
    egui::Frame::window(&ctx.global_style()).inner_margin(egui::Margin::same(18)).stroke(egui::Stroke::new(1.0, k.cy.gamma_multiply(0.45)))
        .shadow(egui::Shadow { offset: [0, 8], blur: 30, spread: 4, color: egui::Color32::from_black_alpha(120) })
}

/// Hintergrund abdunkeln (modal), wie beim Projekt-Wizard
fn dim_background(ctx: &egui::Context, id: &str) {
    let screen = ctx.content_rect().size();
    egui::Area::new(egui::Id::new(id)).order(egui::Order::Tooltip).fixed_pos(egui::Pos2::ZERO).show(ctx, |ui| {
        let (r, _) = ui.allocate_exact_size(screen, egui::Sense::click_and_drag());
        ui.painter().rect_filled(r, 0.0, egui::Color32::from_black_alpha(110));
    });
}

/// Ablagezone: gestrichelter Rand, Pfeil nach unten, Text; bei überfliegender Datei hervorgehoben
pub(crate) fn drop_zone(ui: &mut egui::Ui, hovering: bool, title: &str, sub: &str) {
    let k = crate::ui::tokens::t();
    let color = if hovering { k.cy } else { k.mute };
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 86.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 6.0, if hovering { k.cy.gamma_multiply(0.14) } else { k.bg2 });
    let r = rect.shrink(3.0);
    let stroke = egui::Stroke::new(1.5, color);
    for seg in [[r.left_top(), r.right_top()], [r.right_top(), r.right_bottom()], [r.right_bottom(), r.left_bottom()], [r.left_bottom(), r.left_top()]] {
        p.extend(egui::Shape::dashed_line(&seg, stroke, 7.0, 5.0));
    }
    // Pfeil nach unten auf eine Ablage
    let c = egui::pos2(rect.center().x, rect.top() + 24.0);
    p.line_segment([c + egui::vec2(0.0, -9.0), c + egui::vec2(0.0, 7.0)], egui::Stroke::new(2.0, color));
    p.line_segment([c + egui::vec2(-6.0, 1.0), c + egui::vec2(0.0, 8.0)], egui::Stroke::new(2.0, color));
    p.line_segment([c + egui::vec2(6.0, 1.0), c + egui::vec2(0.0, 8.0)], egui::Stroke::new(2.0, color));
    p.line_segment([c + egui::vec2(-9.0, 12.0), c + egui::vec2(9.0, 12.0)], egui::Stroke::new(2.0, color));
    p.text(egui::pos2(rect.center().x, rect.top() + 50.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(14.0), ui.visuals().strong_text_color());
    p.text(egui::pos2(rect.center().x, rect.top() + 69.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(11.5), ui.visuals().weak_text_color());
}

/// Eine Zeile mit Systempunkten: Punkt in der Ampelfarbe vor dem Namen, bricht nur zwischen den Einträgen um
fn dots_line(ui: &mut egui::Ui, items: &[(egui::Color32, &str)]) {
    let mut job = egui::text::LayoutJob::default();
    let font = egui::FontId::proportional(13.0);
    let text = ui.visuals().text_color();
    for (i, (c, name)) in items.iter().enumerate() {
        if i > 0 { job.append("    ", 0.0, egui::text::TextFormat { font_id: font.clone(), color: text, ..Default::default() }); }
        job.append("●", 0.0, egui::text::TextFormat { font_id: font.clone(), color: *c, ..Default::default() });
        job.append(&format!(" {name}"), 0.0, egui::text::TextFormat { font_id: font.clone(), color: text, ..Default::default() });
    }
    ui.add(egui::Label::new(job).wrap());
}

/// Zusammenfassung eines Systems für Karte und Band
#[derive(Clone, Debug, PartialEq)]
pub struct SystemSummary {
    pub id: &'static str,
    pub label: &'static str,
    /// erforderliche Rollen insgesamt / davon benutzbar
    pub required: usize,
    pub usable: usize,
    /// Namen der fehlenden erforderlichen Rollen
    pub missing: Vec<&'static str>,
    /// mindestens eine benutzbare ROM ist Ersatz oder unbekannte Revision
    pub degraded: bool,
}

impl SystemSummary {
    pub fn color(&self) -> egui::Color32 { if self.usable < self.required { bad() } else if self.degraded { warn() } else { ok() } }
    pub fn text(&self) -> String {
        if self.usable == self.required { format!("{} von {} ROM-Dateien", self.usable, self.required) } else { format!("{} von {} ROM-Dateien erkannt", self.usable, self.required) }
    }
}

/// Erforderliche Rollen eines Systems (Atari: ein Betriebssystem zählt als eine Rolle `os`)
pub fn summary_of(store: &RomStore, system: &str) -> Option<SystemSummary> {
    let sd = systems::system(system)?;
    let mut roles: Vec<(&'static str, &'static str)> = sd.roles.iter().filter(|r| r.required).map(|r| (r.id, r.label)).collect();
    if system == "atari8" { roles.insert(0, ("os", "Betriebssystem")); }
    let mut s = SystemSummary { id: sd.id, label: sd.label, required: roles.len(), usable: 0, missing: vec![], degraded: false };
    for (id, label) in roles {
        let r = store.resolve(system, id);
        if r.is_usable() { s.usable += 1; if r.status != Status::Recognized { s.degraded = true; } } else { s.missing.push(label); }
    }
    Some(s)
}

/// Zusammenfassungen aller Systeme aus dem ROM-Dienst des Prozesses (`vc_media::romref`)
fn summaries() -> Vec<SystemSummary> { vc_media::romref::with_store(|s| systems::SYSTEMS.iter().filter_map(|d| summary_of(s, d.id)).collect()) }

/// Karte für die Willkommensseite: Ampel je System, Knopf in der Kopfzeile (immer sichtbar). Liefert `true`, wenn „ROMs einrichten …“ geklickt wurde.
pub fn rom_card(ui: &mut egui::Ui) -> bool {
    let mut open = false;
    let all = summaries();
    card(ui, false, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("ROMs").font(egui::FontId::new(14.0, crate::fonts::ui_strong())));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("ROMs einrichten …").clicked() { open = true; } });
        });
        ui.add_space(2.0);
        let items: Vec<(egui::Color32, &str)> = all.iter().map(|s| (s.color(), s.label)).collect();
        dots_line(ui, &items);
        ui.add_space(2.0);
        ui.label(RichText::new("Deine eigenen ROM-Abbilder für Emulator und Zeichensätze. Fehlende ROMs sind ein Hinweis, kein Fehler.").small().weak());
    });
    open
}

/// Hinweisband für den Projekt-Wizard (unterer Rand): „Für C64: 2 von 3 ROM-Dateien erkannt. Zeichensatz fehlt.“ Nur wenn etwas fehlt.
/// Liefert `true`, wenn „ROMs einrichten …“ geklickt wurde.
pub fn rom_band(ui: &mut egui::Ui, systems_chosen: &[&str]) -> bool {
    let mut open = false;
    let all: Vec<SystemSummary> = vc_media::romref::with_store(|s| systems_chosen.iter().filter_map(|id| systems::system_for_target(id).and_then(|d| summary_of(s, d.id))).collect());
    let lines: Vec<(egui::Color32, String)> = all.iter().filter(|s| s.usable < s.required || s.degraded).map(|s| (warn(), if s.usable < s.required { format!("Für {}: {}. Es fehlt: {}.", s.label, s.text(), s.missing.join(", ")) } else { format!("Für {}: ROMs sind Ersatz oder unbekannte Revisionen.", s.label) })).collect();
    if lines.is_empty() { return false; }
    egui::Frame::new().fill(warn().gamma_multiply(0.12)).stroke(egui::Stroke::new(1.0, warn().gamma_multiply(0.5))).corner_radius(4.0).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            for (c, l) in &lines { ui.label(RichText::new(l).small().color(*c)); }   // eine Warnfarbe, auch wenn etwas ganz fehlt
            if ui.small_button("ROMs einrichten …").clicked() { open = true; }
        });
    });
    open
}

/// Meldung aus dem Start, die auf eine fehlende ROM zurückgeht?
pub fn is_rom_error(msg: &str) -> bool { msg.starts_with("ROM ") && msg.contains("nicht gefunden") || msg.starts_with("MEGA65-ROM ") }

#[derive(PartialEq, Eq)]
pub enum ErrAct { None, Setup, Close }

/// Verständlicher Name der fehlenden ROM aus der technischen Meldung („ROM mega65/rom nicht gefunden (erwartet 131072 Byte) …“ → „MEGA65 System-ROM (128 KB)“)
fn friendly_rom_name(msg: &str) -> Option<String> {
    let rest = msg.strip_prefix("ROM ")?;
    let (key, tail) = rest.split_once(' ')?;
    let (sys, role) = key.split_once('/')?;
    let (sd, rd) = (systems::system(sys)?, systems::role(sys, role)?);
    let kb = tail.split("erwartet ").nth(1).and_then(|t| t.split_whitespace().next()).and_then(|n| n.parse::<usize>().ok()).map(|n| format!(" ({} KB)", (n + 1023) / 1024)).unwrap_or_default();
    Some(format!("{} {}{kb}", sd.label, rd.label))
}

/// Fehlerdialog „ROM fehlt“: verständlicher Text, Folge, Weg; Zielordner und gesuchte Orte einklappbar
pub fn error_dialog(ctx: &egui::Context, msg: &str) -> ErrAct {
    let mut act = ErrAct::None;
    let (head, places) = msg.split_once(" Gesuchte Orte: ").map(|(a, b)| (a.to_string(), Some(b.to_string()))).unwrap_or((msg.to_string(), None));
    let target = head.split_once("Datei nach ").and_then(|(_, t)| t.split(" legen").next()).map(str::to_string);
    let name = friendly_rom_name(&head);
    dim_background(ctx, "rom_err_dim");
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { return ErrAct::Close; }
    egui::Window::new("ROM fehlt").order(egui::Order::Tooltip).frame(dialog_frame(ctx)).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        ui.set_width(ctx.content_rect().width().min(520.0) - 60.0);
        match &name {
            Some(n) => { ui.label(RichText::new(format!("{n} fehlt")).strong().size(15.0)); ui.add_space(4.0); ui.label("Der Emulator konnte deshalb nicht gestartet werden."); }
            None => { ui.label(RichText::new("Eine ROM-Datei fehlt").strong().size(15.0)); ui.add_space(4.0); ui.label(&head); }
        }
        ui.add_space(4.0);
        ui.label(RichText::new("Lege deine eigene ROM-Datei in den ROM-Ordner. VintageCoder erkennt sie am Inhalt, der Dateiname ist egal. Der Assistent führt dich dorthin.").small().weak());
        if target.is_some() || places.is_some() {
            egui::CollapsingHeader::new(RichText::new("Einzelheiten").small()).show(ui, |ui| {
                if let Some(t) = &target { ui.label(RichText::new("Zielordner / Dateiname").small().weak()); ui.label(RichText::new(t).monospace().small()); }
                if let Some(d) = &places { ui.label(RichText::new("Gesuchte Orte").small().weak()); for p in d.split("; ") { ui.label(RichText::new(p).monospace().small()); } }
            });
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("ROMs einrichten …").clicked() { act = ErrAct::Setup; }
            if ui.button("Abbrechen").clicked() { act = ErrAct::Close; }
        });
    });
    act
}

/// Assistent „ROMs einrichten“
pub struct RomWizard {
    step: usize,
    chosen: Vec<bool>,
    /// nur die lokalen Orte (ROM-Ordner); fremde Orte nur über die ausdrückliche Suche
    store: RomStore,
    preview: Option<Preview>,
    note: Option<String>,
    search: Option<SearchJob>,
    /// Fläche des Fensters im letzten Bild (Tests)
    last_rect: Option<egui::Rect>,
}

impl RomWizard {
    /// `initial`: Systeme, die vorgewählt sind (z. B. die Zielsysteme des Projekts)
    /// Selbsttest (Screenshots, `VC_DEMO=romwiz2`): direkt zu Schritt 2 „Dateien ablegen“
    pub fn demo_step(&mut self, step: usize) { self.step = step; }

    pub fn new(initial: &[&str]) -> RomWizard {
        let ids: Vec<&str> = initial.iter().filter_map(|t| systems::system_for_target(t).map(|d| d.id)).collect();
        let chosen = systems::SYSTEMS.iter().map(|d| ids.contains(&d.id)).collect();
        RomWizard { step: 0, chosen, store: Self::local_store(), preview: None, note: None, search: None, last_rect: None }
    }

    /// Nur die eigenen Orte (Einstellung, Benutzer-/Programmordner): wenige Dateien, schnell
    fn local_store() -> RomStore { RomStore::new(vc_roms::RomEnv { use_foreign: false, ..vc_ide::roms::env(None, None) }) }

    fn rescan(&mut self) { self.store.rescan(); vc_media::romref::rescan_async(); }

    /// Zeichnen; `false` = schließen
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        push_drops_flag(ctx);
        let dropped = take_drops();
        if !dropped.is_empty() && self.step == 1 {
            let items = import::plan(&self.store, &dropped, None);
            self.preview = Some(Preview::new(format!("{} Datei(en) abgelegt", dropped.len()), items));
        }
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if let Some(job) = &self.search {
            if let Some((items, complete)) = job.poll() {
                let title = if items.is_empty() { if complete { "In anderen Programmen wurden keine bekannten ROMs gefunden".to_string() } else { "Suche abgebrochen, nichts gefunden".to_string() } } else { format!("{} ROM-Datei(en) gefunden{}", items.len(), if complete { "" } else { " (Suche vorzeitig beendet)" }) };
                self.preview = Some(Preview::new(title, items));
                self.search = None;
            } else { ctx.request_repaint_after(std::time::Duration::from_millis(100)); }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { if let Some(j) = &self.search { j.cancel(); } return false; }
        // Fenster höchstens so groß wie der Bildschirm: Inhalt scrollt, die Knopfleiste bleibt unten stehen
        let screen = ctx.content_rect().size();
        let max_h = (screen.y - 90.0).max(260.0);
        let max_w = (screen.x - 60.0).clamp(360.0, 760.0);
        dim_background(ctx, "rom_wiz_dim");
        let shown = egui::Window::new("ROMs einrichten").order(egui::Order::Tooltip).frame(dialog_frame(ctx)).collapsible(false).resizable(false).max_width(max_w).max_height(max_h).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, t) in ["1  Systeme wählen", "2  Dateien ablegen", "3  Ergebnis"].iter().enumerate() {
                    ui.label(if i == self.step { RichText::new(*t).strong() } else { RichText::new(*t).weak() });
                    if i < 2 { ui.label(RichText::new("›").weak()); }
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().max_height((max_h - 150.0).max(120.0)).auto_shrink([false, true]).show(ui, |ui| {
            match self.step {
                0 => {
                    ui.label("Für welche Systeme brauchst du ROMs? (Du kannst das später jederzeit in den Einstellungen unter „ROMs“ ändern.)");
                    for (i, d) in systems::SYSTEMS.iter().enumerate() {
                        let s = summary_of(&self.store, d.id).unwrap();
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.chosen[i], d.label);
                            // abgewählte Systeme neutral: kein rotes „0 von N“ für etwas, das man gar nicht braucht
                            let (txt, col) = if self.chosen[i] { (format!("● {}", s.text()), s.color()) } else { (format!("● {} – nicht gewählt", s.text()), crate::theme::dim()) };
                            ui.label(RichText::new(txt).small().color(col));
                        });
                    }
                }
                1 => {
                    let dir = self.store.env.install_dir();
                    ui.label("Lege deine ROM-Dateien oder ZIP-Archive in den ROM-Ordner oder ziehe sie hierher. Der Dateiname ist egal.");
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(dir.as_ref().map(|d| d.display().to_string()).unwrap_or_default()).monospace().small());
                        if let Some(d) = &dir { if ui.button("Ordner öffnen").clicked() { let _ = std::fs::create_dir_all(d); open_dir(d); } }
                        if ui.button("Neu prüfen").clicked() { self.rescan(); }
                    });
                    drop_zone(ui, hovering, "Hier ablegen", "ROM-Dateien oder ZIP-Archive aus dem Finder bzw. Explorer hierher ziehen");
                    ui.add_space(4.0);
                    ui.label(RichText::new("ROM-Ordner anderer Programme (z. B. VICE) werden nur durchsucht, wenn du das hier ausdrücklich erlaubst. Es wird nichts kopiert, bevor du bestätigst.").small().weak());
                    if ui.add_enabled(self.search.is_none(), egui::Button::new("In anderen Programmen suchen (VICE …)")).on_hover_text("Nur in den bekannten Ordnern dieser Programme, im Hintergrund, abbrechbar").clicked() {
                        self.search = Some(SearchJob::start(vc_ide::roms::env(None, None)));
                    }
                    if let Some(j) = &self.search { if search_progress(ui, j) { j.cancel(); } }
                    let mut run = None; let mut cancel = false;
                    if let Some(p) = &mut self.preview { let (r, c) = preview_card(ui, p); run = r; cancel = c; }
                    if cancel { self.preview = None; }
                    if let Some(items) = run { let (n, errs) = run_import(&items); self.note = Some(if errs.is_empty() { format!("{n} Datei(en) kopiert.") } else { format!("{n} kopiert, Fehler: {}", errs.join("; ")) }); self.preview = None; self.store.rescan(); }
                    if let Some(n) = &self.note { ui.label(RichText::new(n).small()); }
                }
                _ => {
                    for (i, d) in systems::SYSTEMS.iter().enumerate().filter(|(i, _)| self.chosen[*i]) {
                        let _ = i;
                        let s = summary_of(&self.store, d.id).unwrap();
                        card(ui, false, |ui| {
                            ui.horizontal(|ui| { ui.label(RichText::new(d.label).strong()); chip(ui, &s.text(), s.color()); });
                            for rd in d.roles.iter().filter(|r| r.required || d.id == "atari8") {
                                let r = self.store.resolve(d.id, rd.id);
                                if d.id == "atari8" && !r.is_usable() && rd.id != "os-xl" { continue; }
                                ui.horizontal(|ui| {
                                    chip(ui, if r.status == Status::Missing { "Fehlt" } else { r.status.label() }, match r.status { Status::Recognized => ok(), Status::Missing | Status::Unusable => bad(), _ => warn() });
                                    ui.label(rd.label);
                                    if let Some(l) = &r.label { ui.label(RichText::new(l).small().weak()); }
                                });
                            }
                            if !s.missing.is_empty() { ui.label(RichText::new("Ohne diese ROM nimmt VintageCoder – wo es sie gibt – die freie Ersatz-ROM bzw. den eingebauten Ersatz des Emulators; Zeichensätze erscheinen dann als leeres Raster mit Hinweis.").small().weak()); }
                            else if s.degraded { ui.label(RichText::new("Es sind Ersatz-ROMs oder unbekannte Revisionen dabei: Zeichen und interne Adressen können vom Original abweichen.").small().weak()); }
                        });
                    }
                    if self.chosen.iter().all(|c| !c) { ui.label(RichText::new("Kein System gewählt.").weak()); }
                }
            }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if self.step > 0 && ui.button("Zurück").clicked() { self.step -= 1; self.preview = None; }
                if self.step < 2 { if ui.button("Weiter").clicked() { self.step += 1; self.preview = None; if self.step == 2 { self.store.rescan(); } } } else if ui.button("Fertig").clicked() { open = false; }
                if ui.button("Später").clicked() { open = false; }
            });
        });
        self.last_rect = shown.map(|r| r.response.rect);
        open
    }
}

fn open_dir(dir: &std::path::Path) {
    if std::env::var("VC_SHOT").is_ok() { return; }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(dir).spawn(); }
    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(dir).spawn(); }
    #[cfg(all(unix, not(target_os = "macos")))] { let _ = std::process::Command::new("xdg-open").arg(dir).spawn(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_roms::RomEnv;

    #[test]
    fn zusammenfassung_zaehlt_erforderliche_rollen() {
        let dir = std::env::temp_dir().join(format!("vc-app-romui-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("c64")).unwrap();
        std::fs::write(dir.join("c64/kernal.bin"), vec![1u8; 8192]).unwrap();
        std::fs::write(dir.join("c64/basic.bin"), vec![2u8; 8192]).unwrap();
        let store = RomStore::new(RomEnv { vc_roms: Some(dir.clone()), ..Default::default() });
        let s = summary_of(&store, "c64").unwrap();
        assert_eq!((s.required, s.usable), (3, 2));
        assert_eq!(s.missing, vec!["Zeichensatz"]);
        assert!(s.degraded && s.text().contains("2 von 3"));
        let a = summary_of(&store, "atari8").unwrap();
        assert_eq!((a.required, a.usable), (1, 0), "Atari: ein Betriebssystem genügt");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Assistent, Fehlerdialog, Karte und Band zeichnen ohne Absturz (alle drei Schritte)
    #[test]
    fn assistent_und_dialoge_zeichnen() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        let run = |f: &mut dyn FnMut(&egui::Context)| { let mut o = ctx.run_ui(egui::RawInput::default(), |ui| f(ui.ctx())); o.textures_delta.clear(); };
        let mut w = RomWizard::new(&["c64", "atari8", "mega65"]);
        for _ in 0..3 { run(&mut |c| { assert!(w.show(c)); }); w.step += 1; }
        run(&mut |c| { let _ = error_dialog(c, "ROM mega65/rom nicht gefunden (erwartet 131072 Byte). Datei nach /x legen Gesuchte Orte: /a (Benutzerordner); /b (Mitgeliefert)"); });
        let mut o = ctx.run_ui(egui::RawInput::default(), |ui| { rom_card(ui); rom_band(ui, &["c64", "mega65"]); });
        o.textures_delta.clear();
    }

    /// Der Assistent passt in ein kleines Fenster (Inhalt scrollt, nichts ragt über den Rand) und Esc schließt ihn
    #[test]
    fn assistent_bleibt_im_fenster_und_esc_schliesst() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0, 520.0));
        let mut w = RomWizard::new(&["c64", "c128", "vic20", "plus4", "atari8", "mega65"]);
        w.step = 2;
        for _ in 0..3 {
            let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| { assert!(w.show(ui.ctx())); });
            o.textures_delta.clear();
        }
        let rect = w.last_rect.expect("Fenster vorhanden");
        assert!(rect.height() <= screen.height() && rect.top() >= 0.0 && rect.bottom() <= screen.bottom() + 1.0, "Fenster {rect:?} ragt aus {screen:?}");
        let esc = egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() };
        let mut closed = false;
        let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), events: vec![esc], ..Default::default() }, |ui| { closed = !w.show(ui.ctx()); });
        o.textures_delta.clear();
        assert!(closed, "Esc schließt den Assistenten");
    }

    /// Die Suche in anderen Programmen läuft im Hintergrund und lässt sich abbrechen
    #[test]
    fn suche_laeuft_im_hintergrund_und_bricht_ab() {
        let job = SearchJob::start(vc_roms::RomEnv { vc_roms: Some(std::env::temp_dir().join("vc-app-search-nix")), ..Default::default() });
        job.cancel();
        let t0 = std::time::Instant::now();
        let res = loop {
            if let Some(r) = job.poll() { break r; }
            assert!(t0.elapsed() < std::time::Duration::from_secs(20), "Suche reagiert nicht auf Abbrechen");
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert!(!res.1, "abgebrochene Suche meldet sich als unvollständig");
    }

    #[test]
    fn fehlermeldung_wird_verstaendlich() {
        assert_eq!(friendly_rom_name("ROM mega65/rom nicht gefunden (erwartet 131072 Byte). Datei nach /x/mega65/mega65.rom legen").as_deref(), Some("MEGA65 System-ROM (128 KB)"));
        assert_eq!(friendly_rom_name("ROM c64/chargen nicht gefunden (erwartet 4096 Byte). Datei nach /x legen").as_deref(), Some("C64 Zeichensatz (4 KB)"));
        assert_eq!(friendly_rom_name("etwas anderes"), None);
    }

    #[test]
    fn rom_fehler_werden_erkannt() {
        assert!(is_rom_error("ROM mega65/rom nicht gefunden (erwartet 131072 Byte). Datei nach /x legen"));
        assert!(is_rom_error("MEGA65-ROM /x hat 10 Byte, erwartet 131072"));
        assert!(!is_rom_error("xmega65 (Xemu) nicht gefunden"));
    }
}
