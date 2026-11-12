// SPDX-License-Identifier: GPL-3.0-or-later
//! Konfigurationsassistent, Logik ohne Oberfläche (Konzept „Konfigurationsassistent“): Entwurf, Änderungsliste, Übernehmen mit Sicherung,
//! Wiederaufruf (Statusmarken, Neufunde), Teil-Assistenten, Export/Import, Zurücksetzen, Erststart-Erkennung, Beispiele kopieren.
//!
//! Regeln: Der Assistent arbeitet auf einem [`Draft`] im Speicher und schreibt erst in [`apply_to`] (Beispiele, `settings.json` atomar mit
//! `settings.json.bak`, `view.json`). Passwörter erscheinen nie in einer [`Change`], nie im Export (Voreinstellung) und nie in `wizard.json`.
//! Es gibt keinen Netzwerkzugriff und keine Installation; Installationshinweise sind Text aus `data/setup/hints.json`.
use crate::settings::{expand_home, write_atomic, Machine, Settings};
use crate::targets::{self, EmulatorCfg, TargetCfg, SYSTEMS};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Version des Formats von `settings.json` mit `targets`, `machines`, `projects_dir` (davor: fehlend)
pub const SCHEMA_VERSION: u32 = 2;
/// Version von `wizard.json`
pub const WIZARD_VERSION: u32 = 1;

// ---------------------------------------------------------------------------------------------------------------- Schritte und Wege

/// Ein Schritt des Assistenten (Kennung englisch und stabil, siehe `wizard.json`, Liste `later`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Topic { Welcome, Design, Systems, Toolchain, Emulators, Roms, Hardware, StartTarget, Editor, Projects, Summary, Done }

impl Topic {
    pub const ALL: [Topic; 12] = [Topic::Welcome, Topic::Design, Topic::Systems, Topic::Toolchain, Topic::Emulators, Topic::Roms, Topic::Hardware, Topic::StartTarget, Topic::Editor, Topic::Projects, Topic::Summary, Topic::Done];
    pub fn id(self) -> &'static str {
        match self { Topic::Welcome => "welcome", Topic::Design => "design", Topic::Systems => "systems", Topic::Toolchain => "toolchain", Topic::Emulators => "emulators", Topic::Roms => "roms",
            Topic::Hardware => "hardware", Topic::StartTarget => "start_target", Topic::Editor => "editor", Topic::Projects => "projects", Topic::Summary => "summary", Topic::Done => "done" }
    }
    pub fn from_id(s: &str) -> Option<Topic> { Topic::ALL.into_iter().find(|t| t.id() == s) }
    pub fn title(self) -> &'static str {
        match self { Topic::Welcome => "Willkommen", Topic::Design => "Design", Topic::Systems => "Zielsysteme", Topic::Toolchain => "Toolchain", Topic::Emulators => "Emulatoren", Topic::Roms => "ROMs",
            Topic::Hardware => "Echte Hardware", Topic::StartTarget => "Startziel F5", Topic::Editor => "Editor", Topic::Projects => "Projekte", Topic::Summary => "Zusammenfassung", Topic::Done => "Fertig" }
    }
    pub fn group(self) -> &'static str {
        match self { Topic::Welcome | Topic::Design => "Start", Topic::Systems | Topic::Toolchain | Topic::Emulators | Topic::Roms => "Systeme", Topic::Hardware | Topic::StartTarget => "Maschinen",
            Topic::Editor | Topic::Projects => "Arbeitsplatz", Topic::Summary | Topic::Done => "Abschluss" }
    }
}

/// Teil-Assistenten des Wiederaufrufs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part { Roms, Emulators, Hardware, DesignEditor }

/// Weg durch den Assistenten
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode { Quick, Full, Part(Part) }

impl Mode {
    pub fn id(self) -> &'static str { match self { Mode::Quick => "quick", Mode::Full => "full", Mode::Part(_) => "part" } }
    /// Schrittliste des Weges. Schnelleinrichtung: 7 Schritte, Vollweg: 12; die Schritte 5 bis 9 fehlen in der Schnelleinrichtung (automatisch erledigt)
    pub fn steps(self) -> Vec<Topic> {
        use Topic::*;
        match self {
            Mode::Quick => vec![Welcome, Design, Systems, Toolchain, Projects, Summary, Done],
            Mode::Full => Topic::ALL.to_vec(),
            Mode::Part(Part::Roms) => vec![Systems, Roms, Summary],
            Mode::Part(Part::Emulators) => vec![Systems, Toolchain, Emulators, Summary],
            Mode::Part(Part::Hardware) => vec![Hardware, StartTarget, Summary],
            Mode::Part(Part::DesignEditor) => vec![Design, Editor, Summary],
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------- Dateien

/// Orte der drei Dateien des Assistenten
#[derive(Clone, Debug)]
pub struct Paths { pub settings: PathBuf, pub view: PathBuf, pub wizard: PathBuf }

impl Paths {
    /// `settings.json` (oder `VC_SETTINGS`), `view.json` im Konfigurationsordner, `wizard.json` neben der Einstellungsdatei
    pub fn current() -> Option<Paths> {
        let settings = crate::settings::file()?;
        let dir = settings.parent().map(Path::to_path_buf).unwrap_or_default();
        let view = vc_core::userdirs::config_dir().map(|d| d.join("view.json")).unwrap_or_else(|| dir.join("view.json"));
        Some(Paths { settings, view, wizard: dir.join("wizard.json") })
    }
    /// Alle drei Dateien in einem Ordner (Tests)
    pub fn in_dir(dir: &Path) -> Paths { Paths { settings: dir.join("settings.json"), view: dir.join("view.json"), wizard: dir.join("wizard.json") } }
    pub fn backup(&self) -> PathBuf { let mut n = self.settings.clone().into_os_string(); n.push(".bak"); PathBuf::from(n) }
}

/// Darstellung (Inhalt von `view.json`)
#[derive(Clone, Debug, PartialEq)]
pub struct ViewPrefs { pub scheme: String, pub zoom: f32, pub font: f32 }
impl Default for ViewPrefs { fn default() -> Self { ViewPrefs { scheme: "dark".into(), zoom: 1.0, font: 13.0 } } }

impl ViewPrefs {
    pub fn load(path: &Path) -> ViewPrefs {
        let mut v = ViewPrefs::default();
        if let Some(j) = std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            if let Some(s) = j["scheme"].as_str() { v.scheme = norm_scheme(s); }
            if let Some(z) = j["zoom"].as_f64() { v.zoom = (z as f32).clamp(0.7, 2.5); }
            if let Some(f) = j["font"].as_f64() { v.font = (f as f32).clamp(9.0, 36.0); }
        }
        v
    }
    pub fn to_json(&self) -> Value { serde_json::json!({ "scheme": self.scheme, "zoom": self.zoom, "font": self.font }) }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
        write_atomic(path, format!("{}\n", self.to_json()).as_bytes())
    }
}
fn norm_scheme(s: &str) -> String { match s { "light" | "system" => s.into(), _ => "dark".into() } }

/// Zustand des Assistenten (`wizard.json`): bewusst ohne Zugangsdaten und ohne Pfade
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WizardState {
    #[serde(default)] pub version: u32,
    #[serde(default)] pub first_run_done: bool,
    /// Datum des letzten Laufs (`JJJJ-MM-TT`)
    #[serde(default)] pub last_run: String,
    /// `quick`, `full` oder `part`
    #[serde(default)] pub mode: String,
    /// Kopierte Beispiele (`c64/jumprun`)
    #[serde(default)] pub examples_copied: Vec<String>,
    /// Offene Themen (Schrittkennungen), wenn der Nutzer „Später fortsetzen“ gewählt hat
    #[serde(default)] pub later: Vec<String>,
    /// Ignorierte Funde (`emu:xemu`, `port:/dev/ttyUSB0`) und ausgeblendete Hinweise (`banner`)
    #[serde(default)] pub dismissed: Vec<String>,
    /// Schritt, bei dem „Später fortsetzen“ gewählt wurde
    #[serde(default, skip_serializing_if = "String::is_empty")] pub resume_step: String,
    /// Emulator-Sorten, die beim letzten Lauf gefunden wurden (für den Hinweis „Neues erkannt“ beim Programmstart)
    #[serde(default)] pub known_programs: Vec<String>,
    /// Hinweis „Neues erkannt“ abgeschaltet
    #[serde(default)] pub no_news_hint: bool,
}

impl WizardState {
    pub fn load(path: &Path) -> WizardState { std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default() }
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        self.version = WIZARD_VERSION;
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
        let mut t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?; t.push('\n');
        write_atomic(path, t.as_bytes())
    }
    /// Die Einrichtung ist angefangen und nicht fertig („Einrichtung unvollständig“ auf der Willkommensseite)
    pub fn incomplete(&self) -> bool { !self.later.is_empty() && !self.dismissed.iter().any(|d| d == "banner") }
    pub fn mark_later(&mut self, topic: Topic) { let id = topic.id().to_string(); if !self.later.contains(&id) { self.later.push(id); } }
    pub fn clear_later(&mut self, topic: Topic) { self.later.retain(|t| t != topic.id()); }
    pub fn dismiss(&mut self, what: &str) { if !self.dismissed.iter().any(|d| d == what) { self.dismissed.push(what.into()); } }
}

/// Was beim Programmstart zu tun ist
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// weder `wizard.json` noch `settings.json`: Assistent öffnet sich von selbst
    FirstRun,
    /// bestehende Einstellungen, neue Programmversion: einmal „Neu: Konfigurationsassistent“ anbieten, kein Zwang
    OfferOnce,
    Normal,
}

pub fn start_state(paths: &Paths) -> Start {
    match (paths.wizard.exists(), paths.settings.exists()) { (false, false) => Start::FirstRun, (false, true) => Start::OfferOnce, _ => Start::Normal }
}

/// Wann `VC_NO_WIZARD` jeden automatischen Start unterdrückt
pub fn auto_start_suppressed() -> bool { std::env::var("VC_NO_WIZARD").map_or(false, |v| !v.is_empty() && v != "0") }

// ---------------------------------------------------------------------------------------------------------------- Entwurf

/// Der Stand der Konfiguration, wie er auf der Platte liegt (oder im Entwurf stehen soll)
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub settings: Settings,
    pub view: ViewPrefs,
    /// Prüfsumme des Dateiinhalts beim Laden (0 = Datei fehlte); erkennt Änderungen von außen
    pub disk_hash: u64,
}

/// Prüfsumme eines Dateiinhalts, wie sie [`Snapshot::disk_hash`] verwendet
pub fn hash_of(b: &[u8]) -> u64 { hash_bytes(b) }

fn hash_bytes(b: &[u8]) -> u64 { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); b.hash(&mut h); h.finish() }

/// Lädt den Stand von der Platte. Fehlt `settings.json`, ist es der Standard; eine defekte Datei ist ein Fehler (sie wird nie überschrieben)
pub fn load_snapshot(paths: &Paths) -> Result<Snapshot, String> {
    let (mut settings, disk_hash) = match std::fs::read(&paths.settings) {
        Ok(b) => (serde_json::from_slice::<Settings>(&b).map_err(|e| format!("{}: {e}", paths.settings.display()))?, hash_bytes(&b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Settings::default(), 0),
        Err(e) => return Err(format!("{}: {e}", paths.settings.display())),
    };
    // nur eine vorhandene Datei wird migriert; eine neue Installation bekommt nicht alle Systeme auf einmal
    if disk_hash != 0 { migrate(&mut settings); }
    Ok(Snapshot { settings, view: ViewPrefs::load(&paths.view), disk_hash })
}

/// Hebt ein älteres Dateiformat an. Neuere Versionen bleiben erhalten (kein Herabstufen).
pub fn migrate(s: &mut Settings) {
    if s.schema_version < 2 { migrate_v1_to_v2(s); }
}
/// Vor Version 2: nach Programmen gegliederte Felder in Zielsysteme überführen
fn migrate_v1_to_v2(s: &mut Settings) { s.migrate_legacy(); s.schema_version = 2; }

/// Zustand der Beispiel-Kopie im Entwurf
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExamplesCfg {
    pub copy: bool,
    /// abgewählte Beispiele (`c64/jumprun`)
    pub excluded: BTreeSet<String>,
    /// Quelle (`<Installation>/examples`), `None` = nicht gefunden
    pub source: Option<PathBuf>,
    /// Eigener Zielordner der Beispiele (leer = `<Basis>/Beispiele`)
    pub dest: String,
}

/// Der Entwurf, auf dem der Assistent arbeitet
#[derive(Clone, Debug, PartialEq)]
pub struct Draft { pub settings: Settings, pub view: ViewPrefs, pub examples: ExamplesCfg }

impl Draft {
    pub fn from_snapshot(s: &Snapshot) -> Draft { Draft { settings: s.settings.clone(), view: s.view.clone(), examples: ExamplesCfg::default() } }
    /// Entwurf der Erst-Einrichtung: gespeicherte Werte, dazu mindestens ein System (C64), wenn noch keines eingerichtet ist
    pub fn for_first_run(s: &Snapshot) -> Draft {
        let mut d = Draft::from_snapshot(s);
        if d.settings.targets.is_empty() { set_system(&mut d.settings, "c64", true); }
        d
    }
}

/// Wird die Emulator-Sorte auf diesem Betriebssystem angeboten? (Altirra nur unter Windows, Entscheidung 13)
pub fn kind_offered(kind: &str) -> bool { kind_offered_on(kind, cfg!(windows)) }
pub fn kind_offered_on(kind: &str, windows: bool) -> bool { kind != "altirra" || windows }

/// Die Sorten eines Systems, die hier angeboten werden
pub fn offered_kinds(system: &str) -> Vec<&'static targets::EmuKind> { targets::system(system).map(|i| i.kinds().iter().filter(|k| kind_offered(k.id)).collect()).unwrap_or_default() }

/// Blendet ein System ein oder aus. Ausblenden verschiebt den Eintrag nach `hidden_targets` (nichts geht verloren); Einblenden holt ihn zurück.
/// Ein neues System bekommt den ersten hier angebotenen Emulator als Standard.
pub fn set_system(s: &mut Settings, id: &str, on: bool) {
    if on {
        if s.target(id).is_some() { return; }
        if let Some(pos) = s.hidden_targets.iter().position(|t| t.system == id) {
            let t = s.hidden_targets.remove(pos);
            s.targets.push(t);
            s.targets.sort_by_key(|t| SYSTEMS.iter().position(|x| x.id == t.system).unwrap_or(usize::MAX));
        } else {
            s.add_target(id);
            let offered = offered_kinds(id);
            if let (Some(t), Some(first)) = (s.target_mut(id), offered.first()) {
                // das erste System-Emulator-Angebot kann hier fehlen (Altirra): dann den ersten angebotenen nehmen
                if t.emulators.first().map_or(false, |e| !kind_offered(&e.kind)) { t.emulators.clear(); t.add_emulator(first.id); }
            }
        }
    } else if let Some(pos) = s.targets.iter().position(|t| t.system == id) {
        let t = s.targets.remove(pos);
        s.hidden_targets.retain(|h| h.system != id);
        s.hidden_targets.push(t);
    }
}

// ---------------------------------------------------------------------------------------------------------------- Änderungsliste

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { New, Changed, Removed, Same }

/// Eine einzelne, einzeln abwählbare Änderung
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    /// Schlüssel (`target:c64`, `emu:c64:c64-x64sc`, `start:c64`, `machine:m1`, `projects_dir`, `view:scheme`, `examples` …)
    pub key: String,
    /// Bereich (SYSTEM, EMULATOR, F5, HARDWARE, DESIGN, PROJEKTE, ROMS, TOOLCHAIN)
    pub area: &'static str,
    pub kind: Kind,
    pub label: String,
    pub old: String,
    pub new: String,
    pub selected: bool,
    /// Betrifft Zugangsdaten: `old`/`new` sind schon maskiert
    pub sensitive: bool,
    /// Hinweis (z. B. „Einträge bleiben erhalten“)
    pub note: String,
    pub topic: Topic,
}

fn ch(key: String, area: &'static str, topic: Topic, kind: Kind, label: String, old: String, new: String) -> Change {
    Change { key, area, kind, label, old, new, selected: kind != Kind::Same, sensitive: false, note: String::new(), topic }
}

fn pw_state(m: &Machine) -> &'static str { if m.password.is_empty() { "nicht gesetzt" } else { "gesetzt" } }

/// Kurzbeschreibung eines Emulators für die Änderungsliste (Pfad oder „automatisch“)
fn emu_text(e: &EmulatorCfg) -> String {
    let mut p = vec![if e.path.trim().is_empty() { "Pfad: automatisch".to_string() } else { format!("Pfad: {}", e.path.trim()) }];
    if e.standard { p.push("Standard".into()); }
    if !e.args.trim().is_empty() { p.push(format!("Argumente: {}", e.args.trim())); }
    if !e.opts.is_empty() { p.push(format!("{} Parameter", e.opts.len())); }
    if !e.rom.trim().is_empty() { p.push(format!("ROM: {}", e.rom.trim())); }
    p.join(", ")
}

/// Kurzbeschreibung einer Maschine; das Passwort erscheint nie, nur „gesetzt“ / „nicht gesetzt“
fn machine_text(m: &Machine) -> String {
    let mut p = vec![m.display_name().to_string()];
    match m.kind.as_str() { "mega65" => { p.push(format!("Schnittstelle: {}", if m.device.is_empty() { "–" } else { &m.device })); if !m.baud.is_empty() { p.push(format!("Baudrate {}", m.baud)); } }
        _ => { p.push(format!("Adresse: {}", if m.host.is_empty() { "–" } else { &m.host })); p.push(format!("Passwort: {}", pw_state(m))); } }
    p.push(if m.run_after { "RUN danach".into() } else { "nur laden".into() });
    p.join(", ")
}

fn start_text(s: &Settings, sys: &str) -> String {
    match s.target(sys).map(|t| t.start_on.as_str()).unwrap_or("") { "" => "Emulator".into(), id => s.machine(id).map_or_else(|| format!("{id} (fehlt)"), |m| m.display_name().to_string()) }
}

/// Verzeichnis der Beispiele zu einem System (`atari8` heißt in `examples/` `atari`)
pub fn example_dir_name(system: &str) -> &str { if system == "atari8" { "atari" } else { system } }

/// Ein mitgeliefertes Beispielprojekt
#[derive(Clone, Debug, PartialEq)]
pub struct ExampleItem { pub system: String, pub name: String, pub src: PathBuf }
impl ExampleItem { pub fn id(&self) -> String { format!("{}/{}", self.system, self.name) } }

/// Beispielprojekte (Ordner mit `*.vcprj`) der gewählten Systeme in `<Installation>/examples`
pub fn list_examples(root: &Path, systems: &[String]) -> Vec<ExampleItem> {
    let mut v = vec![];
    for sys in systems {
        let Ok(rd) = std::fs::read_dir(root.join(example_dir_name(sys))) else { continue };
        let mut names: Vec<(String, PathBuf)> = rd.flatten().filter(|e| e.path().is_dir()).filter(|e| std::fs::read_dir(e.path()).map_or(false, |d| d.flatten().any(|f| f.path().extension().map_or(false, |x| x == "vcprj")))).map(|e| (e.file_name().to_string_lossy().into_owned(), e.path())).collect();
        names.sort();
        v.extend(names.into_iter().map(|(name, src)| ExampleItem { system: sys.clone(), name, src }));
    }
    v
}

/// Ergebnis des Kopierens der Beispiele
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CopyReport { pub copied: Vec<String>, pub files_created: Vec<PathBuf>, pub files_skipped: usize }

/// Kopiert Beispiele nach `<dest>/<system>/<name>`. Vorhandene Dateien werden nie überschrieben (sie zählen als übersprungen).
pub fn copy_examples(items: &[ExampleItem], dest: &Path) -> Result<CopyReport, String> {
    let mut rep = CopyReport::default();
    // Gemeinsame Dateien (`examples/common`, von den Beispielen als `../../common/…` eingebunden) gehören mit: sonst bauen die Kopien nicht
    if let Some(common) = items.first().and_then(|it| it.src.parent()).and_then(|p| p.parent()).map(|r| r.join("common")).filter(|c| c.is_dir()) {
        copy_tree(&common, &dest.join("common"), &mut rep).map_err(|e| format!("{}: {e}", dest.join("common").display()))?;
    }
    for it in items {
        let to = dest.join(example_dir_name(&it.system)).join(&it.name);
        let before = rep.files_created.len();
        copy_tree(&it.src, &to, &mut rep).map_err(|e| format!("{}: {e}", to.display()))?;
        if rep.files_created.len() > before { rep.copied.push(it.id()); }
    }
    Ok(rep)
}

fn copy_tree(from: &Path, to: &Path, rep: &mut CopyReport) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)?.flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if src.is_dir() { copy_tree(&src, &dst, rep)?; }
        else if dst.exists() { rep.files_skipped += 1; }
        else { std::fs::copy(&src, &dst)?; rep.files_created.push(dst); }
    }
    Ok(())
}

/// Die Beispiele, die der Entwurf kopieren will (nur die gewählten Systeme, ohne abgewählte)
pub fn planned_examples(d: &Draft) -> Vec<ExampleItem> {
    let Some(root) = &d.examples.source else { return vec![] };
    if !d.examples.copy { return vec![]; }
    let systems: Vec<String> = d.settings.targets.iter().map(|t| t.system.clone()).collect();
    list_examples(root, &systems).into_iter().filter(|i| !d.examples.excluded.contains(&i.id())).collect()
}

/// Zielordner der Beispiele: `<Basis>/Beispiele`
pub fn examples_dest(projects_dir: &str) -> PathBuf { PathBuf::from(expand_home(projects_dir.trim())).join("Beispiele") }

/// Zielordner der Beispiele mit eigener Wahl: `custom` (nicht leer) gilt unverändert, sonst `<Basis>/Beispiele`
pub fn examples_dest_in(custom: &str, projects_dir: &str) -> PathBuf {
    if custom.trim().is_empty() { examples_dest(projects_dir) } else { PathBuf::from(expand_home(custom.trim())) }
}

/// Vorschlag für das Basisverzeichnis der Projekte: portabel unterhalb des Installationsordners, sonst `~/VintageCoder`
/// (Windows: `Dokumente\VintageCoder`, im Dateisystem `Documents`)
pub fn default_projects_dir(portable_root: Option<&Path>, home: Option<&Path>, windows: bool) -> String {
    if let Some(p) = portable_root { return p.join("Projekte").to_string_lossy().into_owned(); }
    match home { Some(h) if windows => h.join("Documents").join("VintageCoder").to_string_lossy().into_owned(), Some(h) => h.join("VintageCoder").to_string_lossy().into_owned(), None => "~/VintageCoder".into() }
}

/// Ordner des portablen Betriebs: der Ordner der Programmdatei, wenn dort die Markierungsdatei `portable` liegt
pub fn portable_root() -> Option<PathBuf> { std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)).filter(|d| d.join("portable").is_file()) }

/// Das wirksame Basisverzeichnis: eingestellt, sonst Vorschlag
pub fn effective_projects_dir(s: &Settings) -> String {
    if !s.projects_dir.trim().is_empty() { return s.projects_dir.trim().to_string(); }
    default_projects_dir(portable_root().as_deref(), vc_core::userdirs::home_dir().as_deref(), cfg!(windows))
}

/// Prüft das Basisverzeichnis: nicht leer, kein Systemordner, nicht in der Installation, anlegbar/schreibbar. Fehler = Text für die Oberfläche.
pub fn check_projects_dir(dir: &str, install_dir: Option<&Path>) -> Result<ProjectsDirState, String> {
    let d = dir.trim();
    if d.is_empty() { return Err("Der Ordner darf nicht leer sein".into()); }
    let p = PathBuf::from(expand_home(d));
    if !p.is_absolute() { return Err("Bitte einen vollständigen Pfad angeben (beginnend mit / oder ~/)".into()); }
    let comps = p.components().count();
    if comps <= 2 || ["/usr", "/bin", "/etc", "/System", "/Library", "/var/lib", "/var/log", "/opt", "/sbin", "/dev", "/proc"].iter().any(|s| p.starts_with(s)) || (cfg!(windows) && p.to_string_lossy().to_lowercase().contains("\\windows")) { return Err("Das ist ein Systemordner; bitte einen Ordner in deinem Benutzerordner wählen".into()); }
    if let Some(i) = install_dir { if p.starts_with(i) { return Err("Der Ordner liegt innerhalb der Installation von VintageCoder; sie wird bei Updates ersetzt".into()); } }
    if p.is_dir() {
        let probe = p.join(format!(".vc-write-test-{}", std::process::id()));
        return match std::fs::write(&probe, b"") { Ok(()) => { let _ = std::fs::remove_file(&probe); Ok(ProjectsDirState::Writable) } Err(_) => Err("Der Ordner ist nicht schreibbar".into()) };
    }
    // anlegbar? der nächste vorhandene Elternordner muss schreibbar sein
    let mut anc = p.parent();
    while let Some(a) = anc { if a.is_dir() { break; } anc = a.parent(); }
    match anc { Some(a) if std::fs::metadata(a).map_or(false, |m| !m.permissions().readonly()) => Ok(ProjectsDirState::WillCreate), _ => Err("Der Ordner lässt sich hier nicht anlegen".into()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectsDirState { Writable, WillCreate }

/// Die Änderungsliste: alles, was sich zwischen dem gespeicherten Stand und dem Entwurf unterscheidet, dazu „Unverändert“ für Orientierung
pub fn diff(cur: &Snapshot, d: &Draft) -> Vec<Change> {
    let (a, b) = (&cur.settings, &d.settings);
    let mut out: Vec<Change> = vec![];
    // Zielsysteme
    for info in SYSTEMS {
        let (ta, tb) = (a.target(info.id), b.target(info.id));
        let key = format!("target:{}", info.id);
        match (ta, tb) {
            (None, Some(t)) => {
                let restored = a.hidden_targets.iter().any(|h| h.system == info.id);
                let mut c = ch(key, "SYSTEM", Topic::Systems, Kind::New, format!("Zielsystem {}", info.name), String::new(), t.standard_emulator().map_or("kein Emulator".into(), |e| e.display_name().to_string()));
                if restored { c.note = "ausgeblendete Einträge werden wiederhergestellt".into(); }
                out.push(c);
            }
            (Some(_), None) => { let mut c = ch(key, "SYSTEM", Topic::Systems, Kind::Removed, format!("Zielsystem {}", info.name), "eingerichtet".into(), "ausgeblendet".into()); c.note = "Emulatoren und Startziel bleiben erhalten; Wiedereinblenden stellt sie her (Sicherung: settings.json.bak)".into(); out.push(c); }
            (Some(x), Some(y)) => {
                out.push(ch(key, "SYSTEM", Topic::Systems, Kind::Same, format!("Zielsystem {}", info.name), String::new(), String::new()));
                let ids: BTreeSet<&str> = x.emulators.iter().chain(y.emulators.iter()).map(|e| e.id.as_str()).collect();
                for id in ids {
                    let (ea, eb) = (x.emulators.iter().find(|e| e.id == id), y.emulators.iter().find(|e| e.id == id));
                    let k = format!("emu:{}:{id}", info.id);
                    out.push(match (ea, eb) {
                        (None, Some(e)) => ch(k, "EMULATOR", Topic::Emulators, Kind::New, format!("{}: {}", info.name, e.display_name()), String::new(), emu_text(e)),
                        (Some(e), None) => ch(k, "EMULATOR", Topic::Emulators, Kind::Removed, format!("{}: {}", info.name, e.display_name()), emu_text(e), String::new()),
                        (Some(e), Some(f)) if e != f => ch(k, "EMULATOR", Topic::Emulators, Kind::Changed, format!("{}: {}", info.name, f.display_name()), emu_text(e), emu_text(f)),
                        (Some(e), Some(_)) => ch(k, "EMULATOR", Topic::Emulators, Kind::Same, format!("{}: {}", info.name, e.display_name()), emu_text(e), String::new()),
                        (None, None) => unreachable!(),
                    });
                }
                let (sa, sb) = (start_text(a, info.id), start_text(b, info.id));
                let kind = if x.start_on != y.start_on { Kind::Changed } else { Kind::Same };
                out.push(ch(format!("start:{}", info.id), "F5", Topic::StartTarget, kind, format!("{}: F5 startet auf", info.name), sa, sb));
            }
            (None, None) => {}
        }
    }
    // Maschinen
    let ids: BTreeSet<&str> = a.machines.iter().chain(b.machines.iter()).map(|m| m.id.as_str()).collect();
    for id in ids {
        let (ma, mb) = (a.machine(id), b.machine(id));
        let k = format!("machine:{id}");
        let mut c = match (ma, mb) {
            (None, Some(m)) => ch(k, "HARDWARE", Topic::Hardware, Kind::New, format!("Maschine {}", m.display_name()), String::new(), machine_text(m)),
            (Some(m), None) => ch(k, "HARDWARE", Topic::Hardware, Kind::Removed, format!("Maschine {}", m.display_name()), machine_text(m), String::new()),
            (Some(m), Some(n)) if m != n => ch(k, "HARDWARE", Topic::Hardware, Kind::Changed, format!("Maschine {}", n.display_name()), machine_text(m), machine_text(n)),
            (Some(m), Some(_)) => ch(k, "HARDWARE", Topic::Hardware, Kind::Same, format!("Maschine {}", m.display_name()), machine_text(m), String::new()),
            (None, None) => unreachable!(),
        };
        // ein geändertes Passwort ist allein schon eine Änderung (die Texte zeigen es nur als „gesetzt“)
        if ma.zip(mb).map_or(false, |(m, n)| m.password != n.password) { c.sensitive = true; if c.old == c.new { c.new.push_str(" (Passwort geändert)"); } }
        if ma.is_some_and(|m| !m.password.is_empty()) || mb.is_some_and(|m| !m.password.is_empty()) { c.sensitive = true; }
        out.push(c);
    }
    // Projektordner und Beispiele
    let (pa, pb) = (effective_projects_dir(a), if b.projects_dir.trim().is_empty() { effective_projects_dir(a) } else { b.projects_dir.trim().to_string() });
    let kind = if a.projects_dir.trim() == b.projects_dir.trim() { Kind::Same } else if a.projects_dir.trim().is_empty() { Kind::New } else { Kind::Changed };
    let mut c = ch("projects_dir".into(), "PROJEKTE", Topic::Projects, kind, "Projektordner".into(), if a.projects_dir.trim().is_empty() { String::new() } else { pa }, pb);
    if kind != Kind::Same { c.note = "verschiebt nichts; nur die Voreinstellung für neue Projekte ändert sich".into(); }
    out.push(c);
    let planned = planned_examples(d);
    if !planned.is_empty() {
        let dest = examples_dest_in(&d.examples.dest, &effective_projects_dir(b));
        let missing = planned.iter().filter(|i| !dest.join(example_dir_name(&i.system)).join(&i.name).exists()).count();
        if missing > 0 { out.push(ch("examples".into(), "PROJEKTE", Topic::Projects, Kind::New, format!("{} Beispiele kopieren", planned.len()), String::new(), format!("nach {} ({} fehlen noch, vorhandene Dateien bleiben unberührt)", dest.display(), missing))); }
    }
    // Darstellung
    let (va, vb) = (&cur.view, &d.view);
    out.push(ch("view:scheme".into(), "DESIGN", Topic::Design, if va.scheme != vb.scheme { Kind::Changed } else { Kind::Same }, "Farbschema".into(), va.scheme_label().into(), vb.scheme_label().into()));
    out.push(ch("view:zoom".into(), "DESIGN", Topic::Design, if (va.zoom - vb.zoom).abs() > 0.001 { Kind::Changed } else { Kind::Same }, "Oberfläche".into(), format!("{:.0} %", va.zoom * 100.0), format!("{:.0} %", vb.zoom * 100.0)));
    out.push(ch("view:font".into(), "DESIGN", Topic::Design, if (va.font - vb.font).abs() > 0.001 { Kind::Changed } else { Kind::Same }, "Editorschrift".into(), format!("{:.0} pt", va.font), format!("{:.0} pt", vb.font)));
    // übrige Einstellungen
    out.push(ch("roms:use_foreign".into(), "ROMS", Topic::Roms, if a.roms != b.roms { Kind::Changed } else { Kind::Same }, "ROMs anderer Programme mitbenutzen".into(), yes_no(a.roms.use_foreign).into(), yes_no(b.roms.use_foreign).into()));
    out.push(ch("assembler".into(), "TOOLCHAIN", Topic::Toolchain, if a.assembler != b.assembler { Kind::Changed } else { Kind::Same }, "Assembler-Optionen".into(), format!("{} Optionen", a.assembler.opts.len()), format!("{} Optionen", b.assembler.opts.len())));
    out.push(ch("streams".into(), "HARDWARE", Topic::Hardware, if a.streams != b.streams { Kind::Changed } else { Kind::Same }, "Datenströme der Ultimate".into(), String::new(), String::new()));
    out
}

fn yes_no(b: bool) -> &'static str { if b { "ja" } else { "nein" } }

impl ViewPrefs { pub fn scheme_label(&self) -> &'static str { match self.scheme.as_str() { "light" => "Hell", "system" => "System", _ => "Dunkel" } } }

/// Wählt die Änderungen eines Themas ab (Teil-Assistenten ändern nur ihren Bereich) oder an
pub fn only_topics(changes: &mut [Change], topics: &[Topic]) { for c in changes { if c.kind != Kind::Same && !topics.contains(&c.topic) { c.selected = false; } } }

/// Fügt die gewählten Änderungen des Entwurfs in den gespeicherten Stand ein. Rückgabe: neue Einstellungen, neue Darstellung, Hinweise
/// (z. B. Startziel auf eine Maschine, die nicht übernommen wird → Startziel wird zurückgesetzt).
pub fn merge(cur: &Snapshot, d: &Draft, selected: &dyn Fn(&str) -> bool) -> (Settings, ViewPrefs, Vec<String>) {
    let (mut s, b) = (cur.settings.clone(), &d.settings);
    let mut notes = vec![];
    for info in SYSTEMS {
        let (ta, tb) = (cur.settings.target(info.id).cloned(), b.target(info.id).cloned());
        match (ta, tb) {
            (None, Some(t)) if selected(&format!("target:{}", info.id)) => {
                s.hidden_targets.retain(|h| h.system != info.id);
                s.targets.push(t);
            }
            (Some(_), None) if selected(&format!("target:{}", info.id)) => set_system(&mut s, info.id, false),
            (Some(_), Some(y)) => {
                // Emulatoren einzeln
                let ids: BTreeSet<String> = cur.settings.target(info.id).into_iter().flat_map(|t| t.emulators.iter()).chain(y.emulators.iter()).map(|e| e.id.clone()).collect();
                if let Some(t) = s.target_mut(info.id) {
                    for id in ids {
                        if !selected(&format!("emu:{}:{id}", info.id)) { continue; }
                        let src = y.emulators.iter().find(|e| e.id == id).cloned();
                        let pos = t.emulators.iter().position(|e| e.id == id);
                        match (pos, src) { (Some(p), Some(e)) => t.emulators[p] = e, (None, Some(e)) => t.emulators.push(e), (Some(p), None) => { t.emulators.remove(p); } (None, None) => {} }
                    }
                    if selected(&format!("start:{}", info.id)) { t.start_on = y.start_on.clone(); }
                    t.normalize();
                }
            }
            _ => {}
        }
    }
    // Maschinen
    let ids: BTreeSet<String> = cur.settings.machines.iter().chain(b.machines.iter()).map(|m| m.id.clone()).collect();
    for id in ids {
        if !selected(&format!("machine:{id}")) { continue; }
        let src = b.machine(&id).cloned();
        let pos = s.machines.iter().position(|m| m.id == id);
        match (pos, src) { (Some(p), Some(m)) => s.machines[p] = m, (None, Some(m)) => s.machines.push(m), (Some(p), None) => { s.machines.remove(p); } (None, None) => {} }
    }
    if selected("projects_dir") { s.projects_dir = b.projects_dir.clone(); }
    if selected("roms:use_foreign") { s.roms = b.roms.clone(); }
    if selected("assembler") { s.assembler = b.assembler.clone(); }
    if selected("streams") { s.streams = b.streams.clone(); }
    // Abhängigkeiten: Startziel auf eine Maschine, die es nicht gibt
    let machine_ids: Vec<String> = s.machines.iter().map(|m| m.id.clone()).collect();
    for t in &mut s.targets { if !t.start_on.is_empty() && !machine_ids.contains(&t.start_on) { notes.push(format!("{}: Startziel zeigte auf eine nicht übernommene Maschine; F5 startet wieder im Emulator", targets::system(&t.system).map_or(t.system.as_str(), |i| i.name))); t.start_on.clear(); } }
    s.schema_version = s.schema_version.max(SCHEMA_VERSION);
    let mut v = cur.view.clone();
    if selected("view:scheme") { v.scheme = d.view.scheme.clone(); }
    if selected("view:zoom") { v.zoom = d.view.zoom; }
    if selected("view:font") { v.font = d.view.font; }
    (s, v, notes)
}

// ---------------------------------------------------------------------------------------------------------------- Übernehmen

/// Was [`apply_to`] getan hat; hält den Vorzustand für „Rückgängig“ bis zum Neustart
#[derive(Clone, Debug)]
pub struct Applied {
    pub settings: Settings,
    pub view: ViewPrefs,
    pub notes: Vec<String>,
    pub backup: Option<PathBuf>,
    pub copy: CopyReport,
    before_settings: Option<Vec<u8>>,
    before_view: Option<Vec<u8>>,
}

fn read_opt(p: &Path) -> Option<Vec<u8>> { std::fs::read(p).ok() }

fn restore(p: &Path, bytes: &Option<Vec<u8>>) -> Result<(), String> {
    match bytes { Some(b) => write_atomic(p, b), None => match std::fs::remove_file(p) { Ok(()) => Ok(()), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), Err(e) => Err(format!("{}: {e}", p.display())) } }
}

fn remove_created(rep: &CopyReport) { for f in rep.files_created.iter().rev() { let _ = std::fs::remove_file(f); let mut d = f.parent(); for _ in 0..3 { if let Some(x) = d { if std::fs::remove_dir(x).is_err() { break; } d = x.parent(); } } } }

/// Schreibt die gewählten Änderungen. Reihenfolge: (1) Beispiele kopieren (rückholbar), (2) `settings.json` atomar mit Sicherung `.bak`, (3) `view.json`.
/// Schlägt ein Schritt fehl, ist danach alles wie vorher (aus den im Speicher gehaltenen Dateiinhalten) und der Fehler nennt Pfad und Ursache.
/// Ist die Einstellungsdatei seit dem Laden von außen geändert worden, wird nichts geschrieben.
pub fn apply_to(paths: &Paths, cur: &Snapshot, d: &Draft, selected: &dyn Fn(&str) -> bool) -> Result<Applied, String> {
    let before_settings = read_opt(&paths.settings);
    if before_settings.as_deref().map_or(0, hash_bytes) != cur.disk_hash { return Err("Die Konfiguration hat sich geändert, seit der Assistent sie geladen hat. Bitte den Assistenten neu starten.".into()); }
    let before_view = read_opt(&paths.view);
    let (settings, view, notes) = merge(cur, d, selected);
    // (1) Beispiele
    let mut copy = CopyReport::default();
    if selected("examples") {
        let dest = examples_dest_in(&d.examples.dest, &if settings.projects_dir.trim().is_empty() { effective_projects_dir(&settings) } else { settings.projects_dir.clone() });
        match copy_examples(&planned_examples(d), &dest) { Ok(r) => copy = r, Err(e) => { return Err(e); } }
    }
    let fail = |e: String, copy: &CopyReport| -> Result<Applied, String> { remove_created(copy); Err(e) };
    // (2) settings.json mit Sicherung
    let backup = match &before_settings {
        Some(b) => { let bak = paths.backup(); if let Err(e) = write_atomic(&bak, b) { return fail(format!("Sicherung {}", e), &copy); } Some(bak) }
        None => None,
    };
    if let Err(e) = settings.save_atomic(&paths.settings) { let _ = restore(&paths.settings, &before_settings); return fail(e, &copy); }
    // (3) view.json (nur wenn sich etwas ändert)
    if view != cur.view {
        if let Err(e) = view.save(&paths.view) {
            let r1 = restore(&paths.settings, &before_settings);
            let r2 = restore(&paths.view, &before_view);
            remove_created(&copy);
            return Err(format!("{e}{}", if r1.is_err() || r2.is_err() { " (Rückbau unvollständig, Sicherung: settings.json.bak)" } else { " (Änderungen zurückgenommen)" }));
        }
    }
    Ok(Applied { settings, view, notes, backup, copy, before_settings, before_view })
}

/// Übernimmt und macht die Einstellungen im Prozess wirksam (für die Oberfläche)
pub fn apply(cur: &Snapshot, d: &Draft, selected: &dyn Fn(&str) -> bool) -> Result<Applied, String> {
    let paths = Paths::current().ok_or("Kein Ordner für die Einstellungen (HOME bzw. unter Windows APPDATA fehlt)")?;
    let a = apply_to(&paths, cur, d, selected)?;
    crate::settings::replace_in_memory(a.settings.clone());
    Ok(a)
}

/// Macht die übernommenen Einstellungen im Prozess wirksam (nach [`apply_to`])
pub fn apply_replace_memory(a: &Applied) { crate::settings::replace_in_memory(a.settings.clone()); }
/// Macht Einstellungen im Prozess wirksam (nach dem Rückgängigmachen)
pub fn apply_replace_memory_settings(s: Settings) { crate::settings::replace_in_memory(s); }

/// „Rückgängig“: stellt Einstellungen, Darstellung und kopierte Beispiele wieder her
pub fn undo_to(paths: &Paths, a: &Applied) -> Result<(), String> {
    restore(&paths.settings, &a.before_settings)?;
    restore(&paths.view, &a.before_view)?;
    remove_created(&a.copy);
    Ok(())
}

/// Eine defekte Einstellungsdatei sichern (`.bak`), damit „Neu beginnen“ sie nie ohne Kopie überschreibt
pub fn backup_broken(paths: &Paths) -> Result<PathBuf, String> {
    let b = std::fs::read(&paths.settings).map_err(|e| format!("{}: {e}", paths.settings.display()))?;
    let mut n = paths.settings.clone().into_os_string(); n.push(".defekt.bak");
    let n = PathBuf::from(n);
    write_atomic(&n, &b)?;
    Ok(n)
}

// ---------------------------------------------------------------------------------------------------------------- Wiederaufruf

/// Statusmarke eines Schrittes im Wiederaufruf
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Badge { Unchanged, New, Changed, Problem, Skipped }

/// Ein Problem im gespeicherten Stand (Pfad existiert nicht mehr …)
#[derive(Clone, Debug, PartialEq)]
pub struct Problem { pub topic: Topic, pub text: String }

/// Probleme des gespeicherten Stands: eingetragene Programmpfade und ROM-Orte, die es nicht gibt, Maschinen ohne Adresse/Schnittstelle, fehlender Projektordner
pub fn problems(s: &Settings) -> Vec<Problem> {
    let mut v = vec![];
    for t in &s.targets {
        let sys = targets::system(&t.system).map_or(t.system.as_str(), |i| i.name);
        for e in &t.emulators {
            if !e.path.trim().is_empty() && crate::settings::check_program(&e.path).is_err() { v.push(Problem { topic: Topic::Emulators, text: format!("{sys}: Pfad von {} existiert nicht mehr", e.display_name()) }); }
            if !e.rom.trim().is_empty() && !Path::new(&expand_home(e.rom.trim())).exists() { v.push(Problem { topic: Topic::Roms, text: format!("{sys}: ROM-Ort von {} existiert nicht mehr", e.display_name()) }); }
        }
    }
    for m in &s.machines {
        let missing = match m.kind.as_str() { "mega65" => m.device.trim().is_empty(), _ => m.host.trim().is_empty() };
        if missing { v.push(Problem { topic: Topic::Hardware, text: format!("{}: Adresse bzw. Schnittstelle fehlt", m.display_name()) }); }
    }
    if !s.projects_dir.trim().is_empty() && !Path::new(&expand_home(s.projects_dir.trim())).is_dir() { v.push(Problem { topic: Topic::Projects, text: "Projektordner existiert nicht (wird beim ersten Projekt angelegt)".into() }); }
    v
}

/// Neufund: wird nur vorgeschlagen (Übernehmen / Ignorieren), nie still übernommen
#[derive(Clone, Debug, PartialEq)]
pub struct Finding { pub id: String, pub topic: Topic, pub text: String, pub system: String, pub kind: String }

/// Neufunde: Emulatoren, die die Suche findet und die noch nicht eingetragen sind; ignorierte kommen nicht wieder.
/// `discover` liefert den Fundort einer Emulator-Sorte (Dateisystem und PATH, kein Netzwerk, nichts wird gestartet).
pub fn findings(s: &Settings, w: &WizardState, discover: &dyn Fn(&str) -> Option<String>) -> Vec<Finding> {
    let mut v = vec![];
    for t in &s.targets {
        let Some(info) = targets::system(&t.system) else { continue };
        for k in info.kinds() {
            if !kind_offered(k.id) || t.emulators.iter().any(|e| e.kind == k.id) { continue; }
            // die schnelle Nebenform (x64) ist kein eigener Vorschlag
            if k.id == "x64" { continue; }
            let id = format!("emu:{}:{}", t.system, k.id);
            if w.dismissed.contains(&id) { continue; }
            if discover(k.id).is_some() { v.push(Finding { id, topic: Topic::Emulators, text: format!("{}: {} gefunden, noch nicht eingetragen", info.name, k.label), system: t.system.clone(), kind: k.id.into() }); }
        }
    }
    v
}

/// Übernimmt einen Neufund in den Entwurf
pub fn accept_finding(d: &mut Draft, f: &Finding) { if let Some(t) = d.settings.target_mut(&f.system) { if !t.emulators.iter().any(|e| e.kind == f.kind) { t.add_emulator(&f.kind); } } }

/// Hinweis beim Programmstart: Gibt es Emulator-Sorten, die jetzt gefunden werden und beim letzten Lauf nicht? (Nur Dateisystem.)
pub fn news_since_last_run(w: &WizardState, discover: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    if w.no_news_hint || !w.first_run_done { return vec![]; }
    let mut v = vec![];
    for info in SYSTEMS { for k in info.kinds() { if kind_offered(k.id) && k.id != "x64" && !w.known_programs.iter().any(|p| p == k.id) && !v.iter().any(|x: &String| x == k.id) && discover(k.id).is_some() { v.push(k.id.to_string()); } } }
    v
}

/// Emulator-Sorten, die gerade gefunden werden (für `known_programs`)
pub fn found_kinds(discover: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for info in SYSTEMS { for k in info.kinds() { if kind_offered(k.id) && discover(k.id).is_some() && !v.iter().any(|x| x == k.id) { v.push(k.id.to_string()); } } }
    v
}

/// Statusmarke je Schritt: Problem vor Geändert vor Neu; Schritte ohne Handlungsbedarf sind „unverändert“
pub fn badges(changes: &[Change], problems: &[Problem], findings: &[Finding], skipped: &BTreeSet<Topic>) -> BTreeMap<Topic, Badge> {
    let mut m: BTreeMap<Topic, Badge> = Topic::ALL.into_iter().map(|t| (t, Badge::Unchanged)).collect();
    let mut put = |t: Topic, b: Badge| { let cur = m.get(&t).copied().unwrap_or(Badge::Unchanged); let rank = |b: Badge| match b { Badge::Unchanged => 0, Badge::Skipped => 1, Badge::New => 2, Badge::Changed => 3, Badge::Problem => 4 }; if rank(b) > rank(cur) { m.insert(t, b); } };
    for t in skipped { put(*t, Badge::Skipped); }
    for c in changes { match c.kind { Kind::New => put(c.topic, Badge::New), Kind::Changed | Kind::Removed => put(c.topic, Badge::Changed), Kind::Same => {} } }
    for f in findings { put(f.topic, Badge::New); }
    for p in problems { put(p.topic, Badge::Problem); }
    m
}

// ---------------------------------------------------------------------------------------------------------------- Zurücksetzen

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetArea { Design, Roms, Emulators, Hardware, All }

/// Setzt einen Bereich des Entwurfs auf den Standard. `All` lässt Hardware und Passwörter in Ruhe (dafür ist `Hardware` ausdrücklich zu wählen).
pub fn reset_area(d: &mut Draft, area: ResetArea) {
    match area {
        ResetArea::Design => d.view = ViewPrefs::default(),
        ResetArea::Roms => d.settings.roms = Default::default(),
        ResetArea::Hardware => { d.settings.machines.clear(); for t in &mut d.settings.targets { t.start_on.clear(); } d.settings.streams = Default::default(); }
        ResetArea::Emulators => {
            let systems: Vec<String> = d.settings.targets.iter().map(|t| t.system.clone()).collect();
            for sys in systems {
                let start_on = d.settings.target(&sys).map(|t| t.start_on.clone()).unwrap_or_default();
                d.settings.targets.retain(|t| t.system != sys); d.settings.add_target(&sys);
                set_system_default_kind(&mut d.settings, &sys);
                if let Some(t) = d.settings.target_mut(&sys) { t.start_on = start_on; }
            }
            d.settings.assembler = Default::default();
        }
        ResetArea::All => {
            let machines = std::mem::take(&mut d.settings.machines);
            let streams = std::mem::take(&mut d.settings.streams);
            let hidden = d.settings.hidden_targets.clone();
            let start: BTreeMap<String, String> = d.settings.targets.iter().map(|t| (t.system.clone(), t.start_on.clone())).collect();
            reset_area(d, ResetArea::Design); reset_area(d, ResetArea::Roms); reset_area(d, ResetArea::Emulators);
            d.settings.projects_dir.clear(); d.settings.hidden_targets = hidden;
            d.settings.machines = machines; d.settings.streams = streams;
            for t in &mut d.settings.targets { if let Some(s) = start.get(&t.system) { t.start_on = s.clone(); } }
        }
    }
}

fn set_system_default_kind(s: &mut Settings, sys: &str) {
    let offered = offered_kinds(sys);
    if let (Some(t), Some(first)) = (s.target_mut(sys), offered.first()) { if t.emulators.first().map_or(false, |e| !kind_offered(&e.kind)) { t.emulators.clear(); t.add_emulator(first.id); } }
}

// ---------------------------------------------------------------------------------------------------------------- Export und Import

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExportOpts {
    /// Netzwerkpasswörter der Maschinen einschließen (Voreinstellung: nein)
    pub passwords: bool,
    /// Pfade außerhalb des Projektordners (Emulatoren, ROMs) einschließen; sonst leer (= auf dem anderen Rechner automatisch suchen)
    pub outside_paths: bool,
}

pub const EXPORT_KIND: &str = "vintagecoder-config";

/// Pfade, die beim Export umgeschrieben werden: (Bezeichnung, Wert). Für die Rückfrage „Pfade außerhalb des Projektordners“.
pub fn outside_paths(s: &Settings) -> Vec<String> {
    let base = PathBuf::from(expand_home(&effective_projects_dir(s)));
    path_fields(s).into_iter().filter(|p| !p.trim().is_empty() && !Path::new(&expand_home(p.trim())).starts_with(&base)).collect()
}

fn path_fields(s: &Settings) -> Vec<String> {
    let mut v = vec![];
    for t in &s.targets { for e in &t.emulators { v.push(e.path.clone()); v.push(e.rom.clone()); } }
    for m in &s.machines { v.push(m.etherload.clone()); v.push(m.ftp_tool.clone()); }
    v
}

fn map_paths(s: &mut Settings, f: &dyn Fn(&str) -> String) {
    for t in s.targets.iter_mut().chain(s.hidden_targets.iter_mut()) { for e in &mut t.emulators { e.path = f(&e.path); e.rom = f(&e.rom); } }
    for m in &mut s.machines { m.etherload = f(&m.etherload); m.ftp_tool = f(&m.ftp_tool); }
}

/// Exportdatei als JSON: Schemaversion, Einstellungen (Pfade im Projektordner als `{projects_dir}/…`, Passwörter nur auf Wunsch), Darstellung
pub fn export_json(s: &Settings, view: &ViewPrefs, o: ExportOpts) -> Value {
    let mut s = s.clone();
    let base = PathBuf::from(expand_home(&effective_projects_dir(&s)));
    map_paths(&mut s, &|p| {
        let t = p.trim(); if t.is_empty() { return String::new(); }
        let full = PathBuf::from(expand_home(t));
        match full.strip_prefix(&base) { Ok(rel) => format!("{{projects_dir}}/{}", rel.to_string_lossy().replace('\\', "/")), Err(_) => if o.outside_paths { t.to_string() } else { String::new() } }
    });
    if !o.passwords { for m in &mut s.machines { m.password.clear(); } }
    let mut j = serde_json::to_value(&s).unwrap_or(Value::Null);
    if let Value::Object(m) = &mut j { m.remove("projects_dir"); }
    serde_json::json!({ "kind": EXPORT_KIND, "schema_version": SCHEMA_VERSION, "passwords_included": o.passwords, "projects_dir": if s.projects_dir.trim().is_empty() { Value::Null } else { Value::String(s.projects_dir.clone()) }, "settings": j, "view": view.to_json() })
}

/// Ergebnis eines Imports: Entwurf (gegen den gespeicherten Stand zu vergleichen) und Pfadhinweise
#[derive(Clone, Debug)]
pub struct Imported { pub draft: Draft, pub path_notes: Vec<String> }

/// Liest eine Exportdatei in einen Entwurf. Ein leeres Passwort in der Datei löscht kein vorhandenes. Pfade, die es hier nicht gibt, werden gemeldet.
pub fn import_json(text: &str, cur: &Snapshot) -> Result<Imported, String> {
    let j: Value = serde_json::from_str(text).map_err(|e| format!("Keine gültige Exportdatei: {e}"))?;
    if j["kind"] != EXPORT_KIND { return Err("Das ist keine Exportdatei von VintageCoder".into()); }
    let ver = j["schema_version"].as_u64().unwrap_or(0) as u32;
    if ver > SCHEMA_VERSION { return Err(format!("Die Datei stammt aus einer neueren Version (Format {ver}); bitte VintageCoder aktualisieren")); }
    let mut s: Settings = serde_json::from_value(j["settings"].clone()).map_err(|e| format!("Einstellungen in der Datei unlesbar: {e}"))?;
    migrate(&mut s);
    // Basisverzeichnis: aus der Datei, sonst der hiesige
    s.projects_dir = j["projects_dir"].as_str().map(String::from).unwrap_or_else(|| cur.settings.projects_dir.clone());
    let base = effective_projects_dir(&s);
    map_paths(&mut s, &|p| p.replace("{projects_dir}", &base));
    for m in &mut s.machines { if m.password.is_empty() { if let Some(old) = cur.settings.machine(&m.id) { m.password = old.password.clone(); } } }
    let mut notes = vec![];
    for t in &s.targets { for e in &t.emulators {
        if !e.path.trim().is_empty() && crate::settings::check_program(&e.path).is_err() { notes.push(format!("{}: Pfad existiert hier nicht, bitte neu suchen: {}", e.display_name(), e.path.trim())); }
        if !e.rom.trim().is_empty() && !Path::new(&expand_home(e.rom.trim())).exists() { notes.push(format!("{}: ROM-Ort existiert hier nicht: {}", e.display_name(), e.rom.trim())); }
    } }
    s.hidden_targets = cur.settings.hidden_targets.clone();
    let mut view = cur.view.clone();
    if let Some(v) = j["view"].as_object() {
        if let Some(x) = v.get("scheme").and_then(|x| x.as_str()) { view.scheme = norm_scheme(x); }
        if let Some(x) = v.get("zoom").and_then(|x| x.as_f64()) { view.zoom = (x as f32).clamp(0.7, 2.5); }
        if let Some(x) = v.get("font").and_then(|x| x.as_f64()) { view.font = (x as f32).clamp(9.0, 36.0); }
    }
    Ok(Imported { draft: Draft { settings: s, view, examples: ExamplesCfg::default() }, path_notes: notes })
}

// ---------------------------------------------------------------------------------------------------------------- Toolchain und Hinweise

/// Eine Zeile der Toolchain-Tabelle
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRow { pub kind: String, pub name: String, pub used_for: Vec<String>, pub builtin: bool }

/// Programme, die die gewählten Systeme brauchen: eingebauter Assembler, Emulator-Sorten (nur hier angebotene), Java ist nicht nötig
pub fn toolchain_rows(systems: &[String]) -> Vec<ToolRow> {
    let mut rows: Vec<ToolRow> = vec![ToolRow { kind: "vcc-asm".into(), name: "Assembler (eingebaut)".into(), used_for: systems.iter().filter_map(|s| targets::system(s).map(|i| i.name.to_string())).collect(), builtin: true }];
    for sys in systems {
        let Some(info) = targets::system(sys) else { continue };
        for k in info.kinds().iter().filter(|k| kind_offered(k.id) && k.id != "x64") {
            match rows.iter_mut().find(|r| r.kind == k.id) { Some(r) => r.used_for.push(info.name.into()), None => rows.push(ToolRow { kind: k.id.into(), name: k.label.into(), used_for: vec![info.name.into()], builtin: false }) }
        }
    }
    rows
}

/// Programmname in `hints.json` zu einer Sorte
pub fn hint_program(kind: &str) -> &str { match kind { "x64sc" | "x64" | "x128" | "xplus4" | "xvic" => "vice", k => k } }

/// Ein Installationshinweis aus `data/setup/hints.json`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hint {
    pub program: String,
    /// `macos`, `windows`, `linux`
    pub os: String,
    /// `brew`, `winget`, `apt`, `dnf`, `zypper`, `flatpak` oder leer (nur Webseite)
    #[serde(default)] pub package_manager: String,
    /// Befehl zum Kopieren (wird nie ausgeführt)
    #[serde(default)] pub command: String,
    #[serde(default)] pub url: String,
    #[serde(default)] pub note: String,
}

pub fn load_hints(data_dir: &Path) -> Vec<Hint> { std::fs::read_to_string(data_dir.join("setup/hints.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default() }

/// Betriebssystem als Kennung der Hinweise
pub fn os_id() -> &'static str { if cfg!(target_os = "macos") { "macos" } else if cfg!(windows) { "windows" } else { "linux" } }

pub fn hints_for<'a>(h: &'a [Hint], kind: &str, os: &str) -> Vec<&'a Hint> { h.iter().filter(|x| x.program == hint_program(kind) && x.os == os).collect() }

/// Paketmanager der Linux-Verteilung aus dem Inhalt von `/etc/os-release` (`ID` und `ID_LIKE`)
pub fn package_manager_from_os_release(text: &str) -> Option<&'static str> {
    let get = |k: &str| text.lines().find_map(|l| l.strip_prefix(&format!("{k}=")).map(|v| v.trim().trim_matches('"').to_lowercase())).unwrap_or_default();
    let all = format!("{} {}", get("ID"), get("ID_LIKE"));
    let has = |n: &str| all.split_whitespace().any(|w| w == n);
    if has("debian") || has("ubuntu") { Some("apt") } else if has("fedora") || has("rhel") || has("centos") { Some("dnf") } else if has("opensuse") || has("suse") || all.contains("opensuse") { Some("zypper") } else { None }
}

/// Orte, an denen nach einem Programm gesucht wird (für „Gesucht in“): `PATH` und die festen Ordner der Suche
pub fn search_places(kind: &str) -> Vec<String> {
    let mut v: Vec<String> = vec!["PATH".into()];
    let fixed: &[&str] = match kind {
        "x64sc" | "x64" | "x128" | "xplus4" | "xvic" => &["/Applications/vice-arm64-gtk3-3.10/bin", "/Applications/vice-arm64-gtk3-3.9/bin", "/opt/homebrew/bin", "/usr/local/bin"],
        "atari800" => &["/opt/homebrew/bin", "/usr/local/bin"],
        "xemu" => &["/opt/homebrew/bin", "/usr/local/bin", "/Applications/xmega65.app/Contents/MacOS", "/Applications/Xemu"],
        _ => &[],
    };
    v.extend(fixed.iter().map(|s| s.to_string()));
    v.push("von VintageCoder portabel installierte Programme".into());
    v
}

/// Bürgerliches Datum (UTC) `JJJJ-MM-TT` aus Sekunden seit 1970
pub fn date_text(secs: u64) -> String {
    let z = (secs / 86400) as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}", if m <= 2 { y + 1 } else { y }, m, d)
}
pub fn today() -> String { date_text(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())) }

/// Das Schreiben der Konfiguration kann mehrere Systeme berühren: Zielsysteme, die der Assistent anbietet (ohne ausgeblendete)
pub fn visible_systems(s: &Settings) -> Vec<String> { s.targets.iter().map(|t| t.system.clone()).collect() }

/// Hat `TargetCfg` für diese Eingabe einen Standard? (Prüfung der Schrittregel „genau ein Standard je System“)
pub fn one_standard_each(s: &Settings) -> bool { s.targets.iter().all(|t: &TargetCfg| t.emulators.is_empty() || t.emulators.iter().filter(|e| e.standard).count() == 1) }

#[cfg(test)]
mod tests;
