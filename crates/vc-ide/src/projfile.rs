// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektdatei (`*.vcprj`): Modell, Laden/Speichern, Prüfung und Gesamt-Build über alle Quellen.
//!
//! ```json
//! { "format": 1, "project": {"name": …, "description": …},
//!   "targets": [{"id": "c64", "system": "c64", …}],
//!   "sources": [{"targets": ["c64"], "kind": "asm", "files": ["a.asm", "b.asm"]}],
//!   "main": "a.asm",                                   // Startprogramm für „Starten“ (F5)
//!   "disk": {"format": "d64", "name": "MEINEDISK", "file": "build/demo.d64", "files": ["a.asm"]},   // optional
//!   "layout": {"binary": "res/bin", "source": "src/lib"},   // optional: Zielordner je Kategorie
//!   "assets": [{"file": "data/font.bin", "category": "charset", "description": "Zeichensatz-Rohdaten"}] }   // optional: weitere Dateien
//! ```
use crate::build::Ctx;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default)] pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub description: String,
    #[serde(flatten)] pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetDef {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub system: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub cpu: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub variant: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub illegals: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub emulator: String,
    /// Nur für dieses Projekt: Kennung des Standard-Emulators dieses Systems aus den globalen Einstellungen (leer = globaler Standard)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub default_emulator: String,
    /// Nur für dieses Projekt: Startziel (F5) dieses Systems – `@emulator` oder die Kennung einer Maschine aus den globalen Einstellungen (leer = globaler Wert)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub start_on: String,
    #[serde(flatten)] pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Source {
    #[serde(default)] pub targets: Vec<String>,
    #[serde(default)] pub kind: String,
    #[serde(default)] pub files: Vec<String>,
    /// BASIC-Dialekt (nur Art `basic`): z. B. `turbo_basic_xl`; leer = Standarddialekt des Targets
    #[serde(default, skip_serializing_if = "String::is_empty")] pub dialect: String,
    /// Nur Art `basic`: Die Quellen dieser Gruppe werden beim Bauen erst nach Assembler konvertiert (basic2asm), gebaut wird dann das Ergebnis
    #[serde(default, skip_serializing_if = "Option::is_none")] pub convert: Option<ConvertSpec>,
}

/// Einstellungen der Konvertierung BASIC → Assembler für eine Quellgruppe
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ConvertSpec {
    /// Zielsystem der Laufzeit (`c64`, `c128`, `plus4`); leer: Target der Gruppe
    #[serde(default, skip_serializing_if = "String::is_empty")] pub target: String,
    /// Ausgabedatei relativ zum Projekt; leer: `<name>_asm.asm` neben der Quelle
    #[serde(default, skip_serializing_if = "String::is_empty")] pub out: String,
    #[serde(default, skip_serializing_if = "is_false")] pub int_div: bool,
    #[serde(default, skip_serializing_if = "is_false")] pub float: bool,
    /// Zeilenverfolgung (Vorgabe: an)
    #[serde(default = "yes", skip_serializing_if = "is_true")] pub track_line: bool,
    #[serde(default, skip_serializing_if = "is_false")] pub no_rem: bool,
    /// Stil `inline` (einfache Ganzzahlausdrücke als direkte Befehle)
    #[serde(default, skip_serializing_if = "is_false")] pub inline: bool,
    /// Laufzeit in die erzeugte Datei kopieren (statt `#include`)
    #[serde(default, skip_serializing_if = "is_false")] pub embed: bool,
    /// Quelle ist Turbo BASIC XL (nur atari8)
    #[serde(default, skip_serializing_if = "is_false")] pub turbo: bool,
}
fn yes() -> bool { true }
fn is_true(b: &bool) -> bool { *b }
fn is_false(b: &bool) -> bool { !*b }

/// Name des Projektbilds im Projektordner (sichtbar, wird mit eingecheckt)
pub const PREVIEW_FILE: &str = "projektbild.png";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DiskSpec {
    /// d64, d71, d81 oder atr
    pub format: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub name: String,
    /// Ausgabedatei relativ zum Projekt (Standard: `build/<Projektordner>.<format>`)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub file: String,
    /// Quelldateien, deren Programme auf die Diskette kommen (leer: alle)
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub files: Vec<String>,
}

/// Zusätzliche Datei des Projekts (Quelltext-Bibliothek, Text, Binärdaten, Grafikrohdaten …), z. B. per Drag and Drop abgelegt.
/// Der Assembler bindet Binärdaten mit `.incbin "pfad"` ein; die Kategorie bestimmt die Art-Marke im Projektbaum.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Pfad relativ zum Projekt
    pub file: String,
    /// `source`, `text`, `binary`, `graphics` oder `charset`
    #[serde(default)] pub category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub description: String,
}

/// Rolle eines Ordners (Anzeige im Projektbaum, Vorschläge); der Build ignoriert sie
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FolderDef {
    pub dir: String,
    #[serde(default)] pub role: String,
    /// Nur für diese Targets (Mehrsystem-Projekte); leer = für alle
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub targets: Vec<String>,
}

/// Bekannte Ordnerrollen: (Kennung, Beschriftung)
pub const ROLES: [(&str, &str); 6] = [("source", "Quelltext"), ("resources", "Ressourcen"), ("data", "Daten"), ("docs", "Doku"), ("tests", "Tests"), ("tools", "Werkzeuge")];
pub fn role_label(id: &str) -> Option<&'static str> { ROLES.iter().find(|r| r.0 == id).map(|r| r.1) }

/// Regel für abgelegte Dateien: passt der Dateiname auf `match` (Glob mit `*` und `?`, ohne Groß-/Kleinschreibung),
/// gelten `category` und `dir` statt des Vorschlags aus Art und `layout`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "match", default)] pub pattern: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub dir: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    #[serde(default = "one")] pub format: u32,
    #[serde(default)] pub project: Meta,
    #[serde(default)] pub targets: Vec<TargetDef>,
    #[serde(default)] pub sources: Vec<Source>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub main: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub disk: Option<DiskSpec>,
    /// Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen (Standard: nein)
    #[serde(default, skip_serializing_if = "is_false")] pub restore_tabs: bool,
    /// Kein Projektbild (`projektbild.png`) anlegen und anzeigen (Standard: es gibt eines)
    #[serde(default, skip_serializing_if = "is_false")] pub no_preview: bool,
    /// Weitere Dateien mit Kategorie und Beschreibung (Abschnitt `assets`)
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub assets: Vec<Asset>,
    /// Projektstruktur: Zielordner je Kategorie (`source`, `text`, `binary`, `graphics`, `charset`) für abgelegte Dateien; fehlt eine, gilt der Vorschlag der IDE
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")] pub layout: BTreeMap<String, String>,
    /// Ordner mit Rolle (Abschnitt `folders`)
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub folders: Vec<FolderDef>,
    /// Regeln nach Dateimuster, vor `layout` (Abschnitt `rules`)
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub rules: Vec<Rule>,
    #[serde(flatten)] pub extra: Map<String, Value>,
}
fn one() -> u32 { 1 }

/// Die `.vcprj` im Ordner (die erste in alphabetischer Reihenfolge)
pub fn find(root: &Path) -> Option<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root).ok()?.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "vcprj")).collect();
    v.sort(); v.into_iter().next()
}

impl ProjectFile {
    pub fn load(file: &Path) -> Result<ProjectFile, String> {
        let t = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        serde_json::from_str(&t).map_err(|e| format!("{}: {e}", file.display()))
    }
    pub fn save(&self, file: &Path) -> Result<(), String> {
        let mut t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?; t.push('\n');
        std::fs::write(file, t).map_err(|e| format!("{}: {e}", file.display()))
    }
    /// Überschreibungen der globalen Einstellungen, die dieses Projekt je Target festlegt (nur Targets mit mindestens einer Angabe)
    pub fn overrides(&self) -> Vec<crate::targets::ProjectOverride> {
        self.targets.iter().filter(|t| !t.default_emulator.is_empty() || !t.start_on.is_empty()).map(|t| crate::targets::ProjectOverride {
            system: if t.system.is_empty() { t.id.clone() } else { t.system.clone() }, default_emulator: t.default_emulator.clone(), start_on: t.start_on.clone(),
        }).collect()
    }
    /// Schreibt Standard-Emulator und Startziel für `system` als Überschreibung ins Projekt – aber nur, wenn sie von den globalen
    /// Einstellungen abweichen (`emulator`: Kennung eines Emulators des Systems, leer = Standard; `start_on`: leer = Emulator, sonst
    /// Kennung einer Maschine). Ohne Abweichung bleibt die Projektdatei unberührt (eine vorhandene Überschreibung entfällt).
    /// Unbekannte Kennungen zählen nicht als Abweichung. Liefert, ob etwas eingetragen wurde.
    pub fn set_overrides_if_differs(&mut self, system: &str, global: &crate::settings::Settings, emulator: &str, start_on: &str) -> bool {
        let cfg = global.target(system);
        let std_id = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.as_str()).unwrap_or("");
        let default_emulator = if !emulator.is_empty() && emulator != std_id && cfg.map_or(false, |t| t.emulators.iter().any(|e| e.id == emulator)) { emulator.to_string() } else { String::new() };
        let global_start = cfg.map(|t| t.start_on.as_str()).filter(|id| global.machine(id).is_some()).unwrap_or("");
        let start = if start_on != global_start && (start_on.is_empty() || global.machine(start_on).is_some()) { if start_on.is_empty() { "@emulator".to_string() } else { start_on.to_string() } } else { String::new() };
        let mut set = false;
        for t in self.targets.iter_mut().filter(|t| t.system == system || (t.system.is_empty() && t.id == system)) {
            t.default_emulator = default_emulator.clone();
            t.start_on = start.clone();
            set |= !default_emulator.is_empty() || !start.is_empty();
        }
        set
    }
    /// Entfernt die Überschreibungen für `system`; liefert, ob etwas geändert wurde
    pub fn clear_override(&mut self, system: &str) -> bool {
        let mut changed = false;
        for t in self.targets.iter_mut().filter(|t| t.system == system || (t.system.is_empty() && t.id == system)) {
            if !t.default_emulator.is_empty() || !t.start_on.is_empty() { t.default_emulator.clear(); t.start_on.clear(); changed = true; }
        }
        changed
    }
    /// Eine Datei wurde verschoben oder umbenannt: Quellen, `main`, Diskette und `assets` nachführen
    pub fn rename_path(&mut self, old: &str, new: &str) {
        for s in &mut self.sources { for f in &mut s.files { if f == old { *f = new.to_string(); } } }
        if self.main.as_deref() == Some(old) { self.main = Some(new.to_string()); }
        if let Some(d) = &mut self.disk { for f in &mut d.files { if f == old { *f = new.to_string(); } } }
        self.rename_asset(old, Some(new));
    }
    /// Rolle des Ordners (Pfad relativ, ohne abschließendes `/`)
    pub fn role_of(&self, dir: &str) -> Option<&str> { self.folders.iter().find(|f| f.dir == dir).map(|f| f.role.as_str()).filter(|r| !r.is_empty()) }
    /// Setzt oder entfernt (`None`) die Rolle eines Ordners; liefert, ob sich etwas geändert hat
    pub fn set_role(&mut self, dir: &str, role: Option<&str>) -> bool {
        let before = self.folders.clone();
        match role {
            Some(r) => match self.folders.iter_mut().find(|f| f.dir == dir) { Some(f) => f.role = r.to_string(), None => self.folders.push(FolderDef { dir: dir.to_string(), role: r.to_string(), targets: vec![] }) },
            None => { for f in &mut self.folders { if f.dir == dir { f.role.clear(); } } self.folders.retain(|f| f.dir != dir || !f.targets.is_empty()); }
        }
        self.folders != before
    }
    /// Beschränkt einen Ordner auf ein Target (`Some`) oder hebt die Beschränkung auf (`None`); liefert, ob sich etwas geändert hat
    pub fn set_folder_target(&mut self, dir: &str, target: Option<&str>) -> bool {
        let before = self.folders.clone();
        match target {
            Some(t) => match self.folders.iter_mut().find(|f| f.dir == dir) { Some(f) => f.targets = vec![t.to_string()], None => self.folders.push(FolderDef { dir: dir.to_string(), role: String::new(), targets: vec![t.to_string()] }) },
            None => { for f in &mut self.folders { if f.dir == dir { f.targets.clear(); } } self.folders.retain(|f| f.dir != dir || !f.role.is_empty()); }
        }
        self.folders != before
    }
    /// Targets, auf die ein Ordner beschränkt ist (leer = alle)
    pub fn targets_of(&self, dir: &str) -> &[String] { self.folders.iter().find(|f| f.dir == dir).map_or(&[], |f| f.targets.as_slice()) }
    /// Liegt `rel` in einem (oder unter einem) auf bestimmte Targets beschränkten Ordner?
    pub fn in_target_folder(&self, rel: &str) -> bool {
        self.folders.iter().any(|f| !f.targets.is_empty() && !f.dir.is_empty() && rel.starts_with(&format!("{}/", f.dir)))
    }
    /// Macht `dir` zum Zielordner der Kategorie (`layout`); liefert, ob sich etwas geändert hat
    pub fn set_layout_dir(&mut self, category: &str, dir: &str) -> bool { self.layout.insert(category.to_string(), dir.to_string()).as_deref() != Some(dir) }
    /// Eintrag der Datei in `assets`
    pub fn asset(&self, file: &str) -> Option<&Asset> { self.assets.iter().find(|a| a.file == file) }
    /// Trägt eine Datei in `assets` ein oder ersetzt den vorhandenen Eintrag
    pub fn set_asset(&mut self, a: Asset) {
        match self.assets.iter_mut().find(|x| x.file == a.file) { Some(x) => *x = a, None => self.assets.push(a) }
    }
    /// Benennt den Eintrag um (`new` = `Some`) oder entfernt ihn (`None`); liefert, ob etwas geändert wurde
    pub fn rename_asset(&mut self, old: &str, new: Option<&str>) -> bool {
        let n = self.assets.len();
        match new { Some(nw) => { for a in &mut self.assets { if a.file == old { a.file = nw.to_string(); } } }, None => self.assets.retain(|a| a.file != old) }
        self.assets.len() != n || new.map_or(false, |nw| self.assets.iter().any(|a| a.file == nw))
    }
    /// Alle Quelldateien in der Reihenfolge der Projektdatei, mit Art und Target
    pub fn source_files(&self) -> Vec<(String, String, Option<String>)> {
        self.sources.iter().flat_map(|s| s.files.iter().map(move |f| (f.clone(), s.kind.clone(), s.targets.first().cloned()))).collect()
    }
    /// Startprogramm: `main`, sonst die erste Quelldatei
    pub fn main_file(&self) -> Option<String> {
        self.main.clone().filter(|m| !m.is_empty()).or_else(|| self.source_files().into_iter().next().map(|s| s.0))
    }
    /// Ausdrücklich gewählter BASIC-Dialekt der Quellgruppe von `file`
    pub fn dialect_of(&self, file: &str) -> Option<String> {
        self.sources.iter().find(|s| s.files.iter().any(|f| f == file)).map(|s| s.dialect.clone()).filter(|d| !d.is_empty())
    }
    /// Konvertierungseinstellung der Quellgruppe von `file` (nur BASIC-Quellen)
    pub fn convert_of(&self, file: &str) -> Option<ConvertSpec> {
        self.sources.iter().find(|s| s.kind == "basic" && s.files.iter().any(|f| f == file)).and_then(|s| s.convert.clone())
    }
    /// Target-ID, mit der `file` gebaut wird (erstes Target der Quellgruppe, sonst das erste Projekt-Target)
    pub fn target_of(&self, file: &str) -> Option<String> {
        self.source_files().into_iter().find(|s| s.0 == file).and_then(|s| s.2).or_else(|| self.targets.first().map(|t| t.id.clone()))
    }
    /// Diskettenabbild, das beim Starten statt des einzelnen Programms in den Emulator gelegt wird: nur bei VICE-Systemen,
    /// wenn das Startprogramm das erste Programm der Diskette ist (der Autostart lädt immer den ersten Eintrag) – damit kann
    /// ein Programm weitere Dateien der Diskette nachladen (`LOAD "NAME",8,1`).
    pub fn run_disk(&self, root: &Path) -> Option<PathBuf> { self.run_disk_for(root, false) }

    /// Wie [`ProjectFile::run_disk`]; mit `mega65` zählt statt der Commodore-Systeme der MEGA65 mit einer `.d81`-Diskette
    /// (für den Start auf echter Hardware über Ethernet, siehe `mega65net`; Emulator und USB starten das Programm selbst)
    pub fn run_disk_for(&self, root: &Path, mega65: bool) -> Option<PathBuf> {
        let d = self.disk.as_ref()?;
        let main = self.main_file()?;
        let ok = if mega65 { self.target_of(&main)? == "mega65" && d.format == "d81" } else { matches!(self.target_of(&main)?.as_str(), "c64" | "c128" | "plus4" | "c16" | "vic20") && matches!(d.format.as_str(), "d64" | "d71" | "d81") };
        if !ok { return None; }
        let first = self.source_files().into_iter().map(|s| s.0).find(|f| d.files.is_empty() || d.files.contains(f))?;
        if first != main { return None; }
        Some(root.join(disk_file(root, d)))
    }

    /// Probleme der Projektdatei (fehlende Dateien, unbekannte Targets, ungültige Diskette)
    pub fn validate(&self, root: &Path) -> Vec<String> {
        let mut v = vec![];
        let ids: Vec<&str> = self.targets.iter().map(|t| t.id.as_str()).collect();
        let mut seen = std::collections::HashSet::new();
        for (f, kind, _) in self.source_files() {
            if !root.join(&f).is_file() { v.push(format!("{f}: Datei fehlt")); }
            if !seen.insert(f.clone()) { v.push(format!("{f}: mehrfach als Quelle eingetragen")); }
            if !matches!(kind.as_str(), "asm" | "basic") { v.push(format!("{f}: unbekannte Art '{kind}' (asm oder basic)")); }
        }
        for s in &self.sources { for t in &s.targets { if !ids.contains(&t.as_str()) { v.push(format!("Quellgruppe {:?}: Target '{t}' ist nicht definiert", s.files.first().map(String::as_str).unwrap_or(""))); } } }
        for a in &self.assets { if !root.join(&a.file).is_file() { v.push(format!("{}: Datei fehlt (Abschnitt assets)", a.file)); } }
        for (k, d) in &self.layout {
            if crate::importfiles::Category::from_id(k).is_none() { v.push(format!("layout: unbekannte Kategorie '{k}'")); }
            if !d.is_empty() && !crate::importfiles::valid_dir(d) { v.push(format!("layout: '{d}' ist kein gültiger Ordner (relativ, ohne ..)")); }
        }
        for f in &self.folders {
            if !crate::importfiles::valid_dir(&f.dir) || f.dir.is_empty() { v.push(format!("folders: '{}' ist kein gültiger Ordner", f.dir)); }
            if !f.role.is_empty() && role_label(&f.role).is_none() { v.push(format!("folders: unbekannte Rolle '{}' bei '{}'", f.role, f.dir)); }
            for t in &f.targets { if !ids.contains(&t.as_str()) && !self.targets.iter().any(|x| x.system == *t) { v.push(format!("folders: Target '{t}' bei '{}' ist nicht definiert", f.dir)); } }
        }
        for r in &self.rules {
            if r.pattern.trim().is_empty() { v.push("rules: Muster fehlt".into()); }
            if !r.category.is_empty() && crate::importfiles::Category::from_id(&r.category).is_none() { v.push(format!("rules: unbekannte Kategorie '{}' bei '{}'", r.category, r.pattern)); }
            if !r.dir.is_empty() && !crate::importfiles::valid_dir(&r.dir) { v.push(format!("rules: '{}' ist kein gültiger Ordner (bei '{}')", r.dir, r.pattern)); }
        }
        let srcs: Vec<String> = self.source_files().into_iter().map(|s| s.0).collect();
        if let Some(m) = &self.main { if !m.is_empty() && !srcs.contains(m) { v.push(format!("main: '{m}' ist keine Quelldatei des Projekts")); } }
        let mut stems = std::collections::HashMap::new();
        for f in &srcs { if let Some(s) = Path::new(f).file_stem() { if let Some(o) = stems.insert(s.to_string_lossy().into_owned(), f.clone()) { v.push(format!("{o} und {f} erzeugen dieselbe Ausgabedatei (gleicher Dateiname)")); } } }
        if let Some(d) = &self.disk {
            if !matches!(d.format.as_str(), "d64" | "d71" | "d81" | "atr") { v.push(format!("disk: unbekanntes Format '{}' (d64, d71, d81, atr)", d.format)); }
            for f in &d.files { if !srcs.contains(f) { v.push(format!("disk: '{f}' ist keine Quelldatei des Projekts")); } }
        }
        v
    }
}

/// Ausgabedatei der Diskette relativ zum Projekt (Standard: `build/<Ordnername>.<format>`)
pub fn disk_file(root: &Path, spec: &DiskSpec) -> String {
    if spec.file.is_empty() { format!("build/{}.{}", root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "disk".into()), spec.format) } else { spec.file.clone() }
}

/// Dateiname eines Programms auf der Diskette
fn disk_name(format: &str, stem: &str, ext: &str) -> String {
    if format == "atr" {
        let b: String = stem.to_ascii_uppercase().chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).take(8).collect();
        let b = if b.chars().next().map_or(true, |c| !c.is_ascii_alphabetic()) { format!("P{b}").chars().take(8).collect() } else { b };
        format!("{b}.{}", ext.to_ascii_uppercase().chars().take(3).collect::<String>())
    } else { stem.to_ascii_uppercase().chars().take(16).collect() }
}

/// Packt die gebauten Programme auf einen Datenträger. `artifacts`: (Quelldatei, Artefakt relativ zum Projekt)
pub fn make_disk(root: &Path, pf: &ProjectFile, spec: &DiskSpec, artifacts: &[(String, String)]) -> Result<Value, String> {
    let chosen: Vec<&(String, String)> = artifacts.iter().filter(|(f, _)| spec.files.is_empty() || spec.files.contains(f)).collect();
    if chosen.is_empty() { return Err("Diskette: keine Programme ausgewählt".into()); }
    let out_rel = disk_file(root, spec);
    let title = if spec.name.is_empty() { pf.project.name.to_ascii_uppercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ').take(16).collect::<String>() } else { spec.name.clone() };
    let title = if title.is_empty() { "VINTAGECODER".to_string() } else { title };
    let mut names = vec![];
    let bytes = match spec.format.as_str() {
        "atr" => {
            let mut a = vc_disk::Atr::new_dos2();
            for (_, art) in &chosen {
                let p = Path::new(art); let n = disk_name("atr", &p.file_stem().unwrap_or_default().to_string_lossy(), &p.extension().unwrap_or_default().to_string_lossy());
                a.dos2_add(&n, &std::fs::read(root.join(art)).map_err(|e| format!("{art}: {e}"))?)?; names.push(n);
            }
            a.to_bytes()
        }
        "d64" => {
            let mut d = vc_disk::D64::blank(&title, "VC")?;
            for (_, art) in &chosen { let n = disk_name("d64", &Path::new(art).file_stem().unwrap_or_default().to_string_lossy(), ""); d.add_prg(&n, &std::fs::read(root.join(art)).map_err(|e| format!("{art}: {e}"))?)?; names.push(n); }
            d.to_bytes()
        }
        "d71" | "d81" => {
            let mut d = vc_disk::Disk::blank(if spec.format == "d71" { vc_disk::Kind::D71 } else { vc_disk::Kind::D81 }, &title, "VC")?;
            for (_, art) in &chosen { let n = disk_name(&spec.format, &Path::new(art).file_stem().unwrap_or_default().to_string_lossy(), ""); d.add_prg(&n, &std::fs::read(root.join(art)).map_err(|e| format!("{art}: {e}"))?)?; names.push(n); }
            d.to_bytes()
        }
        f => return Err(format!("Diskettenformat '{f}' unbekannt (d64, d71, d81, atr)")),
    };
    let out = root.join(&out_rel);
    if let Some(p) = out.parent() { std::fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    std::fs::write(&out, &bytes).map_err(|e| format!("{out_rel}: {e}"))?;
    Ok(json!({ "file": out_rel, "size": bytes.len(), "programs": names }))
}

/// Baut alle Quellen des Projekts (ein fehlerhaftes Programm hält die übrigen nicht auf) und danach die Diskette.
/// Ergebnis: `ok`, `results` (je Quelle das Ergebnis von [`build`] samt `file`/`target`), `main`, `main_artifact`, `main_target`, `disk`.
/// `only`: nur diese Quelle bauen (zum Starten des Hauptprogramms).
pub fn build_project(c: &Ctx, pf: &ProjectFile, only: Option<&str>) -> Value {
    let mut results = vec![]; let mut arts: Vec<(String, String)> = vec![]; let mut all_ok = true;
    for (f, _kind, _) in pf.source_files() {
        if only.map_or(false, |o| o != f) { continue; }
        let tgt = pf.target_of(&f);
        let built = if let Some(cv) = pf.convert_of(&f) {
            // BASIC → Assembler → Programm: erst konvertieren, dann das Ergebnis bauen
            let mut req = crate::b2a::Req::new(f.clone());
            req.out = cv.out.clone(); req.register = false;
            req.target = if cv.target.is_empty() { tgt.clone().unwrap_or_else(|| vc_target::default_target_or_c64(Some(c.data))) } else { cv.target.clone() };
            req.int_div = cv.int_div; req.force_float = cv.float; req.track_line = cv.track_line; req.no_rem = cv.no_rem; req.inline = cv.inline; req.embed = cv.embed; req.turbo = cv.turbo || f.to_ascii_lowercase().ends_with(".tur");
            match crate::b2a::convert_file(c.root, c.data, &req) {
                Ok(rep) => crate::build::build_with(c, &rep.out, tgt.as_deref(), None).map(|mut v| {
                    v["file"] = json!(f); v["converted"] = json!(rep.out);
                    if let Some(w) = v["diagnostics"].as_array_mut() { for d in &rep.warnings { w.push(json!({ "file": f, "line": d.line, "severity": "warning", "code": d.code, "message": d.message })); } }
                    v
                }),
                Err(ds) => Ok(json!({ "ok": false, "file": f, "target": tgt, "diagnostics": ds.iter().map(|d| json!({ "file": d.file, "line": d.line, "severity": if d.is_error() { "error" } else { "warning" }, "code": d.code, "message": d.message })).collect::<Vec<_>>() })),
            }
        } else { crate::build::build_with(c, &f, tgt.as_deref(), pf.dialect_of(&f).as_deref()) };
        let r = match built {
            Ok(v) => v,
            Err(e) => json!({ "ok": false, "file": f, "target": tgt, "diagnostics": [{ "file": f, "line": 0, "severity": "error", "code": "BUILD", "message": e }] }),
        };
        if r["ok"] == true { if let Some(a) = r["artifact"].as_str() { arts.push((f.clone(), a.to_string())); } } else { all_ok = false; }
        results.push(r);
    }
    if only.is_some() && results.is_empty() { all_ok = false; }
    let main = pf.main_file();
    let main_art = main.as_ref().and_then(|m| arts.iter().find(|(f, _)| f == m).map(|(_, a)| a.clone()));
    let mut out = json!({ "ok": all_ok, "results": results, "main": main, "main_artifact": main_art, "main_target": main.as_ref().and_then(|m| pf.target_of(m)) });
    if only.is_none() && all_ok {
        if let Some(d) = &pf.disk {
            match make_disk(c.root, pf, d, &arts) { Ok(v) => out["disk"] = v, Err(e) => { out["ok"] = json!(false); out["disk_error"] = json!(e); } }
        }
    }
    out
}

/// Einstellung `key` (Text) des Projekt-Targets `tid` (nach `id` oder `system`) aus der `.vcprj` im Ordner `root`, z. B. `memory` beim VC-20
pub fn target_setting(root: &Path, tid: &str, key: &str) -> Option<String> {
    let file = std::fs::read_dir(root).ok()?.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |e| e == "vcprj"))?;
    let v: Value = serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()?;
    v["targets"].as_array()?.iter().find(|t| t["id"] == tid || t["system"] == tid).and_then(|t| t[key].as_str().map(String::from))
}

#[cfg(test)]
mod override_tests {
    use super::*;

    #[test]
    fn ueberschreibungen_gehen_ohne_formatwechsel_hin_und_zurueck() {
        let alt = r#"{ "format": 1, "project": {"name": "x"}, "targets": [{"id": "c64", "system": "c64", "emulator": "vice"}] }"#;
        let pf: ProjectFile = serde_json::from_str(alt).unwrap();
        assert!(pf.overrides().is_empty());
        let mut pf = pf;
        assert_eq!(pf.format, 1);
        pf.targets[0].default_emulator = "c64-x64sc".into();
        pf.targets[0].start_on = "m1".into();
        assert_eq!(pf.overrides(), vec![crate::targets::ProjectOverride { system: "c64".into(), default_emulator: "c64-x64sc".into(), start_on: "m1".into() }]);
        let t = serde_json::to_string(&pf).unwrap();
        assert!(t.contains("\"default_emulator\":\"c64-x64sc\"") && t.contains("\"start_on\":\"m1\"") && t.contains("\"format\":1"));
        assert_eq!(serde_json::from_str::<ProjectFile>(&t).unwrap(), pf);
        assert!(pf.clear_override("c64"));
        assert!(!pf.clear_override("c64"));
        let t = serde_json::to_string(&pf).unwrap();
        assert!(!t.contains("default_emulator") && !t.contains("start_on"));
    }
}

#[cfg(test)]
mod wizard_override_tests {
    use super::*;
    use crate::settings::Settings;

    fn global() -> Settings {
        let mut s = Settings::default();
        s.add_target("atari8");
        s.target_mut("atari8").unwrap().add_emulator("atari800");
        s.add_machine("atari8", "ultimate64");
        s
    }
    fn project() -> ProjectFile {
        serde_json::from_str(r#"{ "format": 1, "project": {"name": "x"}, "targets": [{"id": "atari8", "system": "atari8"}, {"id": "c64", "system": "c64"}] }"#).unwrap()
    }

    #[test]
    fn ohne_abweichung_wird_nichts_geschrieben() {
        let (g, mut pf) = (global(), project());
        let std_id = g.target("atari8").unwrap().standard_emulator().unwrap().id.clone();
        assert!(!pf.set_overrides_if_differs("atari8", &g, &std_id, ""));
        assert!(!pf.set_overrides_if_differs("atari8", &g, "", ""));
        let t = serde_json::to_string(&pf).unwrap();
        assert!(!t.contains("default_emulator") && !t.contains("start_on"), "{t}");
    }

    #[test]
    fn abweichender_emulator_und_startziel() {
        let (g, mut pf) = (global(), project());
        let other = g.target("atari8").unwrap().emulators.iter().find(|e| !e.standard).unwrap().id.clone();
        assert!(pf.set_overrides_if_differs("atari8", &g, &other, "m1"));
        assert_eq!(pf.targets[0].default_emulator, other);
        assert_eq!(pf.targets[0].start_on, "m1");
        assert!(pf.targets[1].default_emulator.is_empty() && pf.targets[1].start_on.is_empty());   // anderes System unberührt
        // Hin und zurück über die Datei
        let back: ProjectFile = serde_json::from_str(&serde_json::to_string_pretty(&pf).unwrap()).unwrap();
        assert_eq!(back.overrides(), vec![crate::targets::ProjectOverride { system: "atari8".into(), default_emulator: other, start_on: "m1".into() }]);
    }

    #[test]
    fn global_maschine_projekt_emulator_und_unbekannte_kennungen() {
        let mut g = global();
        g.target_mut("atari8").unwrap().start_on = "m1".into();
        let mut pf = project();
        // Global startet auf der Maschine, das Projekt will den Emulator: ausdrücklich eintragen
        assert!(pf.set_overrides_if_differs("atari8", &g, "", ""));
        assert_eq!(pf.targets[0].start_on, "@emulator");
        // gleiche Wahl wie global: kein Eintrag
        assert!(!pf.set_overrides_if_differs("atari8", &g, "", "m1"));
        assert!(pf.targets[0].start_on.is_empty());
        // unbekannte Kennungen zählen nicht
        assert!(!pf.set_overrides_if_differs("atari8", &g, "gibt-es-nicht", "m9"));
        assert!(pf.overrides().is_empty());
    }
}

#[cfg(test)]
mod restore_tabs_tests {
    use super::*;

    #[test]
    fn dateien_wiederherstellen_ist_standardmaessig_aus_und_wird_gespeichert() {
        let p: ProjectFile = serde_json::from_str(r#"{"targets":[{"id":"c64"}]}"#).unwrap();
        assert!(!p.restore_tabs);
        assert!(!serde_json::to_string(&p).unwrap().contains("restore_tabs"));
        let mut p = p; p.restore_tabs = true;
        let back: ProjectFile = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert!(back.restore_tabs);
    }
}
