// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungsseite „ROMs“: Status je System und Rolle, ROM-Ordner öffnen, neu prüfen, Dateien und ZIP-Archive ablegen (Drag-and-drop oder „Auswählen …“),
//! „ROMs anderer Programme mitbenutzen“ und „ROMs aus VICE importieren“. Es wird nichts kopiert, bevor der Nutzer die Vorschau bestätigt hat.
//! Der Kern (Suche, Erkennung, Import) steht in `vc_roms`; diese Datei enthält nur die Ansicht.
use crate::theme::{bad, dim, ok, warn};
use crate::ui::widgets::{card, chip, section};
use eframe::egui::{self, RichText};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use vc_ide::settings::RomPrefs;
use vc_roms::import::{self, Action, ImportItem};
use vc_roms::{systems, RomStore, Stage, Status};

/// Auf das Fenster gezogene Dateien, solange die Seite „ROMs“ sichtbar ist (`app.rs` reicht sie durch, statt sie ins Projekt zu kopieren)
static DROPS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static LAST_FRAME: AtomicU64 = AtomicU64::new(0);

/// Ist die Seite „ROMs“ gerade sichtbar (in diesem oder dem vorigen Bild gezeichnet)?
pub fn wants_drops(ctx: &egui::Context) -> bool { ctx.cumulative_frame_nr().saturating_sub(LAST_FRAME.load(Ordering::Relaxed)) <= 2 }

/// Assistent „ROMs einrichten“ meldet sich ebenfalls als Empfänger abgelegter Dateien
pub fn push_drops_flag(ctx: &egui::Context) { LAST_FRAME.store(ctx.cumulative_frame_nr(), Ordering::Relaxed); }

/// Abgelegte Dateien abholen
pub fn take_drops() -> Vec<PathBuf> { DROPS.lock().map(|mut d| std::mem::take(&mut *d)).unwrap_or_default() }

pub fn push_drops(files: Vec<PathBuf>) { if let Ok(mut d) = DROPS.lock() { d.extend(files); } }

/// Vorschau eines Imports: Einträge mit Häkchen
pub(crate) struct Preview { pub title: String, pub items: Vec<(ImportItem, bool)> }

impl Preview {
    pub fn new(title: String, items: Vec<ImportItem>) -> Preview { Preview { title, items: items.into_iter().map(|i| { let c = i.action == Action::Copy; (i, c) }).collect() } }
}

#[derive(Default)]
pub struct RomsUi {
    store: Option<RomStore>,
    /// laufender Hintergrundlauf, der einen neuen Stand einliest (Orte können langsam sein)
    loading: Option<std::sync::mpsc::Receiver<RomStore>>,
    /// laufende Suche in ROM-Ordnern anderer Programme
    search: Option<crate::rom_ui::SearchJob>,
    foreign_state: bool,
    system: usize,
    preview: Option<Preview>,
    message: Option<(bool, String)>,
    help: bool,
}

/// ROM-Orte im Hintergrund einlesen (Dateisystem, ggf. Netzlaufwerke): die Oberfläche bleibt bedienbar
fn start_store_job(st: &mut RomsUi, use_foreign: bool) {
    let mut env = vc_ide::roms::env(None, None);
    env.use_foreign = use_foreign;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(RomStore::new(env)); });
    st.loading = Some(rx);
    st.foreign_state = use_foreign;
}

fn open_folder(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    if std::env::var("VC_SHOT").is_ok() { return; }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(dir).spawn(); }
    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(dir).spawn(); }
    #[cfg(all(unix, not(target_os = "macos")))] { let _ = std::process::Command::new("xdg-open").arg(dir).spawn(); }
}

fn status_color(s: Status) -> egui::Color32 { match s { Status::Recognized => ok(), Status::UnknownRevision | Status::Replacement => warn(), Status::Missing => bad(), Status::Unusable => bad() } }

/// Gesamtampel eines Systems: grün = alles Erforderliche als Original erkannt, gelb = benutzbar mit Ersatz/unbekannter Revision, rot = Erforderliches fehlt
pub fn system_color(store: &RomStore, system: &str) -> egui::Color32 {
    if !store.missing_required(system).is_empty() { return bad(); }
    let sd = systems::system(system);
    let all_original = sd.is_some_and(|sd| sd.roles.iter().filter(|r| r.required).all(|r| store.resolve(system, r.id).status == Status::Recognized))
        && (system != "atari8" || store.resolve(system, "os").status == Status::Recognized);
    if all_original { ok() } else { warn() }
}


/// Vorschau eines Imports als Karte. Knöpfe stehen **oben** (immer sichtbar), die Liste ist nach Systemen gruppiert und scrollt in begrenzter Höhe;
/// alle Zeilen haben dieselben Spalten (Häkchen, Status, ROM, Datei, Revision). Liefert die zu kopierenden Einträge bzw. den Abbruch.
pub(crate) fn preview_card(ui: &mut egui::Ui, p: &mut Preview) -> (Option<Vec<ImportItem>>, bool) {
    let (mut run, mut cancel) = (None, false);
    card(ui, true, |ui| {
        ui.label(RichText::new(&p.title).strong());
        let n = p.items.iter().filter(|(i, on)| *on && i.action == Action::Copy).count();
        let copyable = p.items.iter().filter(|(i, _)| i.action == Action::Copy).count();
        ui.horizontal(|ui| {
            if ui.add_enabled(n > 0, egui::Button::new(format!("{n} Datei(en) kopieren"))).clicked() { run = Some(p.items.iter().filter(|(i, on)| *on && i.action == Action::Copy).map(|(i, _)| i.clone()).collect()); }
            if ui.button("Abbrechen").clicked() { cancel = true; }
            if copyable > 0 {
                if ui.small_button("Alle abwählen").clicked() { for (i, on) in p.items.iter_mut() { if i.action == Action::Copy { *on = false; } } }
                if ui.small_button("Alle wählen").clicked() { for (i, on) in p.items.iter_mut() { if i.action == Action::Copy { *on = true; } } }
            }
        });
        ui.label(RichText::new("Ziel: ROM-Ordner; vorhandene Dateien werden nie überschrieben.").small().weak());
        if p.items.is_empty() { ui.label(RichText::new("Nichts zu übernehmen.").weak()); }
        let height = (ui.ctx().content_rect().height() * 0.45).clamp(160.0, 420.0);
        egui::ScrollArea::vertical().max_height(height).auto_shrink([false, true]).show(ui, |ui| {
            let mut last: Option<String> = None;
            // nach System gruppieren (Reihenfolge der Systeme), innerhalb stabil
            let mut order: Vec<usize> = (0..p.items.len()).collect();
            let rank = |s: &Option<String>| systems::SYSTEMS.iter().position(|d| Some(d.id) == s.as_deref()).unwrap_or(usize::MAX);
            order.sort_by_key(|&i| rank(&p.items[i].0.system));
            for i in order {
                let (it, on) = &mut p.items[i];
                let sys = it.system.clone();
                if last != sys {
                    ui.add_space(4.0);
                    let label = sys.as_deref().and_then(systems::system).map(|d| d.label).unwrap_or("Sonstige");
                    ui.label(RichText::new(label).strong().small());
                    last = sys;
                }
                let role = it.role.as_deref().and_then(|r| systems::role(it.system.as_deref().unwrap_or(""), r)).map(|r| r.label).unwrap_or("?");
                let name = it.data.as_ref().and_then(|_| it.dest.as_ref().and_then(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))).unwrap_or_else(|| it.src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    fixed(ui, 26.0, |ui| { if it.action == Action::Copy { ui.checkbox(on, ""); } });
                    fixed(ui, 150.0, |ui| match &it.action {
                        Action::Copy => { chip(ui, if it.label.is_some() { "Erkannt" } else { "Unbekannte Revision" }, if it.label.is_some() { ok() } else { warn() }); }
                        Action::AlreadyPresent(_) => { chip(ui, "Schon vorhanden", dim()); }
                        Action::Skip(_) => { chip(ui, "Übersprungen", dim()); }
                    });
                    fixed(ui, 150.0, |ui| { ui.label(role); });
                    ui.label(RichText::new(name).monospace().small()).on_hover_text(it.src.display().to_string());
                    let note = match &it.action { Action::Skip(w) => w.clone(), _ => format!("{}{}", it.label.clone().unwrap_or_default(), it.source_label.as_ref().map(|s| format!("  [{s}]")).unwrap_or_default()) };
                    if !note.trim().is_empty() { ui.label(RichText::new(note).small().weak()); }
                });
            }
        });
    });
    (run, cancel)
}

/// Bestätigte Einträge kopieren: (Anzahl kopiert, Fehlertexte); danach den ROM-Dienst neu einlesen
pub(crate) fn run_import(items: &[ImportItem]) -> (usize, Vec<String>) {
    let (mut done, mut errs) = (0, vec![]);
    for it in items { match import::execute(it) { Ok(_) => done += 1, Err(e) => errs.push(e) } }
    vc_media::romref::rescan_async();
    (done, errs)
}

/// Feste Breite für eine Spalte, Inhalt senkrecht zentriert (Chip und Text stehen so auf einer Linie)
fn fixed(ui: &mut egui::Ui, width: f32, body: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(egui::vec2(width, 24.0), egui::Layout::left_to_right(egui::Align::Center), body);
}

/// Eine Rolle als zwei Zeilen: oben Status, Name, Datei und Knöpfe, darunter (klein) Herkunft, Revision, Größe und Prüfsummen. Der Platz richtet sich nach der Fensterbreite.
fn role_row(ui: &mut egui::Ui, system: &str, rd: &systems::RoleDef, r: &vc_roms::Resolved, needed: bool, pick: &mut Option<&'static str>, pin: &mut Option<(String, String, Option<String>)>) {
    let (text, color) = match r.status {
        Status::Missing => (if needed { "Fehlt" } else { "Fehlt (optional)" }, if needed { bad() } else { dim() }),
        s => (s.label(), status_color(s)),
    };
    let file_name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        fixed(ui, 150.0, |ui| { chip(ui, text, color); });
        fixed(ui, 190.0, |ui| { ui.label(RichText::new(rd.label).strong()).on_hover_text(format!("Rolle {}  ·  {} Byte{}", rd.id, size_text(rd.sizes), if rd.required { "" } else { "  ·  optional" })); });
        match &r.path {
            Some(p) => { ui.label(RichText::new(file_name(p)).monospace().small()).on_hover_text(p.display().to_string()); }
            None => { ui.label(RichText::new(format!("z. B. {}", rd.file)).small().weak()); }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Auswählen …").on_hover_text("ROM-Datei wählen; sie wird nach Bestätigung in den ROM-Ordner kopiert").clicked() { *pick = Some(rd.id); }
            if !r.others.is_empty() {
                ui.menu_button("Festnageln", |ui| {
                    let used = r.path.as_ref().map(|p| file_name(p));
                    for f in std::iter::once(r.path.clone().map(|p| (p, true))).flatten().chain(r.others.iter().map(|f| (f.path.clone(), false))) {
                        let name = file_name(&f.0);
                        if ui.selectable_label(Some(&name) == used.as_ref() && f.1, &name).clicked() { *pin = Some((system.to_string(), rd.id.to_string(), Some(name))); ui.close(); }
                    }
                    if ui.button("Automatisch (Standardrevision)").clicked() { *pin = Some((system.to_string(), rd.id.to_string(), None)); ui.close(); }
                });
            }
        });
    });
    // zweite Zeile: eigene Beschriftung, bricht bei Platzmangel um
    let detail = match (&r.path, &r.crc32, &r.sha1) {
        (Some(_), Some(c), Some(sha)) => {
            let mut parts = vec![r.place_label.clone().unwrap_or_default()];
            if let Some(l) = &r.label { parts.push(l.clone()); }
            parts.push(format!("{} Byte", r.size.unwrap_or(0)));
            parts.push(format!("CRC-32 {c:08X}"));
            parts.push(format!("SHA-1 {}", short(sha)));
            if !r.others.is_empty() { parts.push(format!("+{} weitere Datei(en) derselben Rolle", r.others.len())); }
            parts.join("  ·  ")
        }
        _ => format!("Erwartet {} Byte", size_text(rd.sizes)),
    };
    ui.horizontal(|ui| {
        ui.add_space(160.0);
        let l = ui.add(egui::Label::new(RichText::new(detail).small().weak()).wrap());
        if let Some(sha) = &r.sha1 { l.on_hover_text(format!("SHA-1 {sha}")); }
    });
    ui.add_space(2.0);
}

fn short(sha1: &str) -> String { format!("{}…", &sha1[..12.min(sha1.len())]) }

fn size_text(sizes: &[usize]) -> String { sizes.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" / ") }

pub fn show(ui: &mut egui::Ui, st: &mut RomsUi, draft: &mut RomPrefs, saved: &mut RomPrefs) {
    LAST_FRAME.store(ui.ctx().cumulative_frame_nr(), Ordering::Relaxed);
    if let Some(rx) = &st.loading { if let Ok(s) = rx.try_recv() { st.store = Some(s); st.loading = None; } }
    if st.loading.is_none() && (st.store.is_none() || st.foreign_state != draft.use_foreign) { start_store_job(st, draft.use_foreign); }
    if let Some(job) = &st.search {
        if let Some((items, complete)) = job.poll() {
            let title = if items.is_empty() { if complete { "In anderen Programmen wurden keine bekannten ROMs gefunden".to_string() } else { "Suche abgebrochen, nichts gefunden".to_string() } } else { format!("{} ROM-Datei(en) in anderen Programmen gefunden{}", items.len(), if complete { "" } else { " (Suche vorzeitig beendet)" }) };
            st.preview = Some(Preview::new(title, items));
            st.search = None;
        }
    }
    if st.loading.is_some() || st.search.is_some() { ui.ctx().request_repaint_after(std::time::Duration::from_millis(100)); }
    if st.store.is_none() { section(ui, "ROMs", "Prüfe ROM-Orte …", |_| {}); return; }
    // abgelegte Dateien: Vorschau (kopiert wird erst nach Bestätigung)
    let dropped: Vec<PathBuf> = DROPS.lock().map(|mut d| std::mem::take(&mut *d)).unwrap_or_default();
    if !dropped.is_empty() {
        let sys = systems::SYSTEMS[st.system].id;
        let items = import::plan(st.store.as_ref().unwrap(), &dropped, Some(sys));
        st.preview = Some(Preview::new(format!("{} Datei(en) abgelegt", dropped.len()), items));
    }
    let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    let store = st.store.as_ref().unwrap();

    section(ui, "ROMs", "Deine eigenen ROM-Abbilder. VintageCoder liefert keine Original-ROMs mit; erkannt wird am Inhalt (Prüfsumme), der Dateiname ist egal.", |_| {});
    // Kopf: Ordner, portabler Modus, Aktionen
    let mut rescan = false;
    card(ui, false, |ui| {
        let dir = store.env.install_dir();
        ui.horizontal(|ui| {
            crate::ui::widgets::label(ui, "ROM-Ordner");
            ui.label(RichText::new(dir.as_ref().map(|d| d.display().to_string()).unwrap_or_else(|| "–".into())).monospace().small());
            if let Some(d) = &dir { if ui.button("Ordner öffnen").on_hover_text("Öffnet den Ordner im Dateimanager (wird bei Bedarf angelegt)").clicked() { open_folder(d); } }
            if ui.button("Neu prüfen").clicked() { rescan = true; }
        });
        ui.horizontal(|ui| {
            crate::ui::widgets::label(ui, "Portabler Modus");
            match &store.env.portable {
                Some(vc_roms::Portable::Active { dir, by_marker }) => { chip(ui, "aktiv", ok()); ui.label(RichText::new(format!("{}{}", dir.display(), if *by_marker { "  (Markierungsdatei „portable“)" } else { "" })).small().weak()); }
                Some(vc_roms::Portable::Unavailable(why)) => { chip(ui, "nicht verfügbar", warn()); ui.label(RichText::new(why).small().weak()); }
                Some(vc_roms::Portable::Off { base: Some(b) }) => { chip(ui, "aus", dim()); ui.label(RichText::new(format!("Aktivieren: Ordner „roms“ oder Datei „portable“ in {} anlegen", b.display())).small().weak()); }
                _ => { chip(ui, "aus", dim()); }
            }
        });
        if store.env.vc_roms.is_some() { ui.label(RichText::new("Die Umgebungsvariable VC_ROMS ersetzt Benutzer- und Programmordner.").small().color(warn())); }
        egui::CollapsingHeader::new(RichText::new("Gesuchte Orte").small()).id_salt("rom_places").show(ui, |ui| {
            for s in store.searched() {
                ui.horizontal(|ui| {
                    chip(ui, s.stage.label(), if s.files > 0 { ok() } else { dim() });
                    ui.label(RichText::new(s.path.display().to_string()).monospace().small());
                    ui.label(RichText::new(if s.exists { format!("{} Datei(en)", s.files) } else { "nicht vorhanden".into() }).small().weak());
                });
            }
            if !store.env.use_foreign { ui.label(RichText::new("ROM-Ordner anderer Programme werden nicht durchsucht (Schalter unten).").small().weak()); }
        });
    });
    ui.add_space(8.0);

    // Reiter je System
    ui.horizontal_wrapped(|ui| {
        for (i, sd) in systems::SYSTEMS.iter().enumerate() {
            let c = system_color(store, sd.id);
            let text = RichText::new(format!("● {}", sd.label)).color(if st.system == i { ui.visuals().strong_text_color() } else { c });
            let r = ui.selectable_label(st.system == i, text);
            if r.clicked() { st.system = i; }
            r.on_hover_text(match c { x if x == ok() => "alles Erforderliche als Original erkannt", x if x == warn() => "benutzbar, aber Ersatz oder unbekannte Revision", _ => "erforderliche ROM fehlt" });
        }
    });
    let sd = &systems::SYSTEMS[st.system];
    let mut pick: Option<&'static str> = None;
    let mut pin: Option<(String, String, Option<String>)> = None;
    card(ui, false, |ui| {
        for (n, rd) in sd.roles.iter().enumerate() {
            let r = store.resolve(sd.id, rd.id);
            let atari_os = sd.id == "atari8" && systems::OS_ALTERNATIVES.contains(&rd.id);
            let needed = rd.required || (atari_os && !store.resolve("atari8", "os").is_usable());
            if n > 0 { ui.separator(); }
            role_row(ui, sd.id, rd, &r, needed, &mut pick, &mut pin);
        }
        if sd.id == "atari8" { ui.label(RichText::new("Von den Betriebssystemen genügt eines; das erste vorhandene (XL/XE, dann 400/800 B, dann A) wird benutzt. Ohne ROM läuft atari800 mit seinem eingebauten Ersatz.").small().weak()); }
        if sd.id == "mega65" { ui.label(RichText::new("Xemu braucht eine ROM-Datei von genau 131072 Byte; eine unbekannte Fassung wird mit Warnung benutzt.").small().weak()); }
    });

    // Ablagezone
    ui.add_space(8.0);
    crate::rom_ui::drop_zone(ui, hovering, "ROM-Dateien oder ZIP-Archive hier ablegen", &format!("Erkannte Dateien kommen nach Bestätigung in den ROM-Ordner (für {} bevorzugt)", sd.label));

    // Fremde Fundorte
    ui.add_space(8.0);
    card(ui, false, |ui| {
        // wirkt sofort und wird sofort gespeichert (unabhängig von ungespeicherten Änderungen an den Zielsystemen)
        if ui.checkbox(&mut draft.use_foreign, "ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800 …)").on_hover_text("Nur lesend. Nichts wird kopiert, außer du importierst ausdrücklich.").changed() {
            saved.use_foreign = draft.use_foreign;
            let mut cur = vc_ide::settings::get(); cur.roms.use_foreign = draft.use_foreign;
            match vc_ide::settings::set(cur) { Ok(()) => vc_ide::roms::reconfigure_media(), Err(e) => st.message = Some((false, e)) }
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(st.search.is_none(), egui::Button::new("ROMs aus VICE importieren …")).on_hover_text("Sucht die ROMs der installierten Programme (nur in deren bekannten Ordnern, im Hintergrund), zeigt sie an und kopiert nach Bestätigung in den ROM-Ordner").clicked() {
                st.message = None;
                st.search = Some(crate::rom_ui::SearchJob::start(vc_ide::roms::env(None, None)));
            }
            if ui.link("Woher bekomme ich ROMs?").clicked() { st.help = !st.help; }
        });
        if st.search.is_some() { if crate::rom_ui::search_progress(ui, st.search.as_ref().unwrap()) { if let Some(j) = &st.search { j.cancel(); } } }
        if st.help {
            ui.label(RichText::new("• Aus dem eigenen Gerät auslesen (EPROM-Programmer, Kartuschen- und Dump-Werkzeuge).\n• Käuflich mit Lizenz, z. B. als Sammlung eines Anbieters, der die Systemdateien lizenziert.\n• Freie Ersatz-ROMs: Open-ROMs (C64, MEGA65), Altirra (Atari).\nVintageCoder lädt nichts herunter. Ob du ein Abbild benutzen darfst, klärst du selbst; das ist keine Rechtsberatung.").small());
        }
    });

    // Vorschau des Imports
    let mut run: Option<Vec<ImportItem>> = None;
    let mut cancel = false;
    if let Some(p) = &mut st.preview { ui.add_space(8.0); let (r, c) = preview_card(ui, p); run = r; cancel = c; }
    if let Some((good, m)) = &st.message { ui.add_space(6.0); ui.label(RichText::new(m).color(if *good { ok() } else { bad() })); }

    // Aktionen ausführen (nach dem Zeichnen, damit `store` nicht geliehen ist)
    if cancel { st.preview = None; }
    if let Some(items) = run {
        let (done, errs) = run_import(&items);
        st.message = Some((errs.is_empty(), if errs.is_empty() { format!("{done} ROM-Datei(en) kopiert.") } else { format!("{done} kopiert, Fehler: {}", errs.join("; ")) }));
        st.preview = None;
        rescan = true;
    }
    if let Some(role) = pick {
        if let Some(f) = rfd::FileDialog::new().set_title("ROM-Datei wählen").pick_file() {
            let items = import::plan(st.store.as_ref().unwrap(), &[f], Some(sd.id));
            let _ = role;
            st.preview = Some(Preview::new("Gewählte Datei".into(), items));
        }
    }
    if let Some((s, r, f)) = pin {
        // das Festnageln gilt im Ordner, in dem die Datei liegt: Benutzer- bzw. portabler ROM-Ordner
        let root = st.store.as_ref().and_then(|s| s.env.install_dir());
        match root { Some(root) => match vc_roms::store::pin(&root, &s, &r, f.as_deref()) { Ok(()) => { st.message = Some((true, "Auswahl gespeichert (roms.json).".into())); rescan = true; } Err(e) => st.message = Some((false, e)) }, None => st.message = Some((false, "Kein ROM-Ordner bekannt".into())) }
    }
    if rescan { start_store_job(st, draft.use_foreign); vc_media::romref::rescan_async(); }
    let _ = Stage::User;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Seite zeichnet ohne Absturz: leerer ROM-Ordner, dann mit synthetischen ROMs, Vorschau und alle Systemreiter
    #[test]
    fn seite_zeichnet_mit_und_ohne_roms() {
        let dir = std::env::temp_dir().join(format!("vc-app-romspage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("mega65")).unwrap();
        std::env::set_var("VC_ROMS", &dir);
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        let mut st = RomsUi::default();
        let (mut draft, mut saved) = (RomPrefs::default(), RomPrefs::default());
        let mut frame = |st: &mut RomsUi| { let mut out = ctx.run_ui(egui::RawInput::default(), |ui| { show(ui, st, &mut draft, &mut saved); }); out.textures_delta.clear(); };
        frame(&mut st);
        std::fs::write(dir.join("mega65/mega65.rom"), vec![7u8; 131072]).unwrap();
        start_store_job(&mut st, false);
        for _ in 0..200 { frame(&mut st); if st.store.is_some() && st.loading.is_none() { break; } std::thread::sleep(std::time::Duration::from_millis(10)); }
        for i in 0..systems::SYSTEMS.len() { st.system = i; frame(&mut st); }
        // abgelegte Datei: Vorschau entsteht, ohne zu kopieren
        let f = dir.join("neu.rom"); std::fs::write(&f, vec![9u8; 131072]).unwrap();
        st.system = systems::SYSTEMS.len() - 1;
        push_drops(vec![f]);
        frame(&mut st);
        assert!(st.preview.is_some());
        assert!(!dir.join("mega65/neu.rom").exists(), "vor der Bestätigung wird nichts kopiert");
        std::env::remove_var("VC_ROMS");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
