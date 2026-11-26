// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien als Include (Konzept „Include von Medien“, Teil A): `.include "graphics/hero.vcsprite"` bzw. `#include "…"` im BASIC.
//!
//! [`resolve_include`] wandelt die Mediendatei beim Bauen selbst um und liefert den Text zum Einbinden. Der Quelltext nennt den Build-Ordner nicht mehr.
//! * Format [`IncFormat::Asm`]: Assembler-Text (Daten, Marken, Konstanten; bei Sprites mit Animationen folgen die Tabellen im selben Text).
//! * Format [`IncFormat::Data`]: BASIC-DATA-Zeilen mit `@marke:` (PLAY-Songs: ein BASIC-Programmstück).
//!
//! Aktualität: Inhaltsschlüssel statt Zeitstempel (Mediendatei, Profilstand, Exporter-Stand, Format, Ziel; bei Kacheln und Screen-Bänken auch die übrigen Medien
//! des Projekts, weil sie ihre Kachelsätze lesen). Das Ergebnis liegt im Zwischenspeicher `build/.media-cache/`; er ist reine Beschleunigung und
//! jederzeit löschbar. Lesbare Kopie: `build/.media-cache/<name>.asm` bzw. `<name>.data.bas`.
use std::path::{Path, PathBuf};

/// Zwischenspeicher im Build-Ordner des Projekts
pub const CACHE_DIR: &str = "build/.media-cache";
/// Stand des Exporters: bei Änderungen an der erzeugten Ausgabe erhöhen, damit alte Zwischenspeicher verfallen
pub const EXPORTER_REV: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncFormat { Asm, Data }

/// Wohin das Ergebnis gemerkt wird: `Disk` = Zwischenspeicher im Projekt (Bau), `Memory` = nur im Arbeitsspeicher des Prozesses (Sprachdienste,
/// Vorschau: beim bloßen Öffnen oder Tippen darf nichts im Projektordner entstehen)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheMode { Disk, Memory }

fn memory_cache() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    static C: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();
    C.get_or_init(Default::default)
}

impl IncFormat {
    fn export_name(self) -> &'static str { match self { IncFormat::Asm => "asm", IncFormat::Data => "data" } }
    fn tag(self) -> &'static str { match self { IncFormat::Asm => "asm", IncFormat::Data => "data" } }
    /// Endung der lesbaren Kopie
    fn ext(self) -> &'static str { match self { IncFormat::Asm => "asm", IncFormat::Data => "data.bas" } }
}

/// Ergebnis: Text zum Einbinden und die Dateien, von denen er abhängt
#[derive(Clone, Debug)]
pub struct Resolved {
    pub text: String,
    /// Mediendatei (absolut)
    pub source: PathBuf,
    /// Alle Dateien, deren Änderung den Text ändert (Mediendatei, bei Kacheln weitere Medien); für Neubau und Sprachdienste
    pub deps: Vec<PathBuf>,
    /// Treffer im Zwischenspeicher (keine Umwandlung nötig)
    pub cached: bool,
    /// Lesbare Kopie im Zwischenspeicher
    pub readable: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncludeError {
    /// Die Mediendatei gibt es nicht
    NotFound(String),
    /// Die Datei ist kein Medium, passt nicht zum Ziel oder der Exporter meldet einen Fehler (Text vom Exporter)
    Failed(String),
}

impl std::fmt::Display for IncludeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { IncludeError::NotFound(s) | IncludeError::Failed(s) => write!(f, "{s}") } }
}

/// Ist der Pfad ein Medium (Endung `.vc<art>`, alt `.asset.json`)?
pub fn is_media_path(path: &str) -> bool { crate::naming::is_data_file(path) }

/// FNV-1a, 64 Bit: stabil über Rust-Versionen hinweg (der Schlüssel steht in Dateinamen)
fn fnv(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for p in parts { for b in *p { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); } h ^= 0xff; h = h.wrapping_mul(0x100000001b3); }
    h
}

fn all_media(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path(); let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() { if depth < 6 && !name.starts_with('.') && name != "build" && name != "target" && name != "node_modules" { walk(&p, depth + 1, out); } }
            else if crate::naming::is_data_file(&name) { out.push(p); }
        }
    }
    let mut v = vec![]; walk(root, 0, &mut v); v.sort(); v
}

/// Wandelt das Medium `media` (Pfad im Projekt, relativ zu `root` oder absolut) in Include-Text um.
/// `target`: Zielsystem des Baus (`None`: die Ziele der Projektdatei; ohne Projektdatei keine Einschränkung). Passt das Profil nicht dazu, ist das ein Fehler.
pub fn resolve_include(root: &Path, data: &Path, media: &Path, format: IncFormat, target: Option<&str>) -> Result<Resolved, IncludeError> {
    resolve_include_with(root, data, media, format, target, CacheMode::Disk)
}

/// Wie [`resolve_include`]; mit [`CacheMode::Memory`] wird nichts in den Projektordner geschrieben (`readable` zeigt dann auf die Stelle, an der die Kopie stünde)
pub fn resolve_include_with(root: &Path, data: &Path, media: &Path, format: IncFormat, target: Option<&str>, mode: CacheMode) -> Result<Resolved, IncludeError> {
    let abs = if media.is_absolute() { media.to_path_buf() } else { root.join(media) };
    let rel = vc_core::paths::rel_path(root, &abs);
    if !abs.is_file() { return Err(IncludeError::NotFound(vc_i18n::t!("media-include-medium-nicht-gefunden", p1 = &rel))); }
    let bytes = std::fs::read(&abs).map_err(|e| IncludeError::Failed(format!("{rel}: {e}")))?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let a = crate::parse_media(&text).map_err(|e| IncludeError::Failed(vc_i18n::t!("media-include-medium-nicht-lesbar", p1 = &rel, p2 = &e)))?;
    let profiles = crate::load_profiles(data).map_err(IncludeError::Failed)?;
    let p = crate::profile(&profiles, &a.profile).map_err(|e| IncludeError::Failed(vc_i18n::t!("media-include-medium-nicht-lesbar", p1 = &rel, p2 = &e)))?;
    // Ziel prüfen: nie still etwas Falsches erzeugen
    let systems: Vec<String> = match target {
        Some(t) => { let mut v = vec![t.to_string()]; for c in crate::project::compatible_systems(data, t) { if !v.contains(&c) { v.push(c); } } v }
        None => crate::project::project_systems(root, data),
    };
    if !systems.is_empty() && !p.target.is_empty() && !systems.contains(&p.target) {
        return Err(IncludeError::Failed(vc_i18n::t!("media-include-profil-passt-nicht-zum-ziel", p1 = &rel, p2 = &p.target, p3 = &systems.join(", "))));
    }
    // Inhaltsschlüssel
    let profiles_json = std::fs::read(data.join("profile/profiles.json")).unwrap_or_default();
    let mut deps = vec![abs.clone()];
    let mut parts: Vec<Vec<u8>> = vec![bytes.clone(), profiles_json, EXPORTER_REV.to_le_bytes().to_vec(), format.tag().as_bytes().to_vec(), systems.join(",").into_bytes()];
    if p.kind == "tileset" || p.kind == "tilemap" || crate::tiles::is_bank(&a.root) {
        for m in all_media(root) { if m != abs { if let Ok(b) = std::fs::read(&m) { parts.push(b); deps.push(m); } } }
    }
    let refs: Vec<&[u8]> = parts.iter().map(|v| v.as_slice()).collect();
    let key = format!("{:016x}", fnv(&refs));
    let name = a.name.clone();
    let cache = root.join(CACHE_DIR);
    let keyed = cache.join(format!("{name}-{key}.{}", format.ext()));
    let readable = cache.join(format!("{name}.{}", format.ext()));
    let mem_key = format!("{}|{}|{key}", abs.display(), format.tag());
    let hit = match mode {
        CacheMode::Disk => std::fs::read_to_string(&keyed).ok(),
        CacheMode::Memory => memory_cache().lock().ok().and_then(|m| m.get(&mem_key).cloned()),
    };
    if let Some(t) = hit {
        if mode == CacheMode::Disk { refresh_readable(&readable, &t); }
        return Ok(Resolved { text: t, source: abs, deps, cached: true, readable });
    }
    // Umwandeln in einen Arbeitsordner außerhalb des Projekts, dann zusammenfügen
    static WORK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let work = std::env::temp_dir().join(format!("vc-media-{}-{}", std::process::id(), WORK.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&work);
    let res = crate::project::export_in(root, data, &rel, format.export_name(), None, None, None, &work).map_err(|e| IncludeError::Failed(vc_i18n::t!("media-include-medium-nicht-exportierbar", p1 = &rel, p2 = &e)));
    let res = match res { Ok(v) => v, Err(e) => { let _ = std::fs::remove_dir_all(&work); return Err(e); } };
    let mut out = String::new();
    for f in res["files"].as_array().into_iter().flatten() {
        let fname = f["path"].as_str().unwrap_or("").rsplit('/').next().unwrap_or("").to_string();
        match std::fs::read_to_string(work.join(&fname)) {
            Ok(t) => { if !out.is_empty() && !out.ends_with('\n') { out.push('\n'); } out.push_str(&t); }
            Err(e) => { let _ = std::fs::remove_dir_all(&work); return Err(IncludeError::Failed(format!("{rel}: {e}"))); }
        }
    }
    let _ = std::fs::remove_dir_all(&work);
    if out.is_empty() { return Err(IncludeError::Failed(vc_i18n::t!("media-include-medium-nicht-exportierbar", p1 = &rel, p2 = &"leer"))); }
    match mode {
        CacheMode::Memory => { if let Ok(mut m) = memory_cache().lock() { m.insert(mem_key, out.clone()); } }
        CacheMode::Disk => {
            // alte Stände dieses Mediums verwerfen, neuen ablegen
            if let Ok(rd) = std::fs::read_dir(&cache) {
                let pre = format!("{name}-"); let suf = format!(".{}", format.ext());
                for e in rd.flatten() { let n = e.file_name().to_string_lossy().into_owned(); if n.starts_with(&pre) && n.ends_with(&suf) && n.len() == pre.len() + 16 + suf.len() { let _ = std::fs::remove_file(e.path()); } }
            }
            let _ = std::fs::create_dir_all(&cache);
            let _ = vc_core::atomar::schreiben(&keyed, out.as_bytes());
            refresh_readable(&readable, &out);
        }
    }
    Ok(Resolved { text: out, source: abs, deps, cached: false, readable })
}

fn refresh_readable(path: &Path, text: &str) {
    if std::fs::read_to_string(path).map_or(true, |t| t != text) { let _ = vc_core::atomar::schreiben(path, text); }
}

/// Ein aufgelöstes Medium (für das Bauprotokoll)
#[derive(Clone, Debug)]
pub struct MediaUse { pub rel: String, pub cached: bool, pub deps: Vec<PathBuf> }

type Provider = std::sync::Arc<dyn Fn(&Path) -> Option<Result<String, String>> + Send + Sync>;
type Legacy = std::sync::Arc<dyn Fn(&Path) -> Option<String> + Send + Sync>;

/// Die Haken für den Assembler (`vc_asm::Options::include_provider` und `legacy_hint`; die Typen sind dieselben, vc-media kennt vc-asm aber nicht)
/// samt Liste der dabei umgewandelten Medien.
pub struct MediaHooks { pub provider: Provider, pub legacy: Legacy, pub used: std::sync::Arc<std::sync::Mutex<Vec<MediaUse>>> }

/// `target`: Zielsystem des Baus (Profile anderer Systeme sind ein Fehler)
pub fn asm_hooks(root: &Path, data: &Path, target: &str) -> MediaHooks { asm_hooks_with(root, data, target, CacheMode::Disk) }

/// Wie [`asm_hooks`]; [`CacheMode::Memory`] für Sprachdienste (nichts im Projektordner schreiben)
pub fn asm_hooks_with(root: &Path, data: &Path, target: &str, mode: CacheMode) -> MediaHooks {
    let used: std::sync::Arc<std::sync::Mutex<Vec<MediaUse>>> = Default::default();
    let (r, d, t, u) = (root.to_path_buf(), data.to_path_buf(), target.to_string(), used.clone());
    let provider: Provider = std::sync::Arc::new(move |p: &Path| {
        if !is_media_path(&p.to_string_lossy()) { return None; }
        Some(match resolve_include_with(&r, &d, p, IncFormat::Asm, Some(&t), mode) {
            Ok(x) => { u.lock().unwrap().push(MediaUse { rel: vc_core::paths::rel_path(&r, &x.source), cached: x.cached, deps: x.deps.clone() }); Ok(x.text) }
            Err(e) => Err(e.to_string()),
        })
    });
    let r2 = root.to_path_buf();
    let legacy: Legacy = std::sync::Arc::new(move |p: &Path| legacy_media(&r2, p));
    MediaHooks { provider, legacy, used }
}

/// Alter Pfad `build/export/<name>.asm` (auch `.anim.asm`, `.data.bas`, `.bas`): das eindeutige Medium im Projekt mit diesem Namen
pub fn legacy_media(root: &Path, path: &Path) -> Option<String> {
    let norm = crate::naming::EXPORT_DIR;
    let s = path.to_string_lossy().replace('\\', "/");
    if !s.contains(&format!("{norm}/")) { return None; }
    let file = s.rsplit('/').next()?;
    let stem = ["anim.asm", "data.sbas", "data.bas", "asm", "bas", "inc"].iter().find_map(|e| file.strip_suffix(&format!(".{e}"))).unwrap_or(file);
    let hits: Vec<String> = all_media(root).into_iter().filter(|m| {
        let name = std::fs::read_to_string(m).ok().and_then(|t| crate::parse_media(&t).ok()).map(|a| a.name);
        name.as_deref() == Some(stem)
    }).map(|m| vc_core::paths::rel_path(root, &m)).collect();
    if hits.len() == 1 { hits.into_iter().next() } else { None }
}

/// Eine Umstellung eines alten Include-Pfads (`vcc migrate includes`)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncChange {
    /// Quelltext (relativ zum Projekt) und 1-basierte Zeile
    pub file: String,
    pub line: usize,
    /// `Some(neu)`: Zeile wird zu `neu`; `None` mit `removed`: Zeile entfällt (Animationstabellen stecken jetzt im selben Include)
    pub old: String,
    pub new: Option<String>,
    /// Grund, warum nichts umgestellt wird (mehrdeutig, kein Medium gefunden …); dann ist `new == None` und `skipped`
    pub skipped: Option<String>,
}

/// Alte Include-Pfade `build/export/<name>.asm|.data.bas|.bas` in allen Quelltexten des Projekts auf die Mediendatei umstellen.
/// Mit `apply == false` nur anzeigen. Eine `<name>.anim.asm`-Zeile, die unmittelbar auf den Include desselben Mediums folgt, entfällt.
/// Nicht eindeutige Namen (zwei Medien gleichen Namens, keines) bleiben unverändert und werden gemeldet.
pub fn migrate_includes(root: &Path, apply: bool) -> Result<Vec<IncChange>, String> {
    fn sources(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path(); let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() { if depth < 8 && !name.starts_with('.') && name != "build" && name != "target" && name != "node_modules" { sources(&p, depth + 1, out); } }
            else if matches!(p.extension().and_then(|x| x.to_str()).map(|x| x.to_ascii_lowercase()).as_deref(), Some("asm" | "inc" | "s" | "bas" | "lst" | "tur")) { out.push(p); }
        }
    }
    /// (Anfang der Pfadzeichenkette, Pfad, Ende) in einer Include-Zeile
    fn path_of(line: &str) -> Option<(usize, String, usize)> {
        let t = line.trim_start(); let lead = line.len() - t.len();
        let lower = t.to_ascii_lowercase();
        if !(lower.starts_with(".include") || lower.starts_with("#include")) { return None; }
        let q1 = t.find('"')?; let q2 = q1 + 1 + t[q1 + 1..].find('"')?;
        Some((lead + q1 + 1, t[q1 + 1..q2].to_string(), lead + q2))
    }
    let mut files = vec![]; sources(root, 0, &mut files); files.sort();
    let mut changes = vec![];
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else { continue };
        if !text.contains("build/export/") { continue; }
        let rel = vc_core::paths::rel_path(root, &f);
        let mut out = String::new(); let mut changed = false;
        let mut last_main: Option<(String, String)> = None;   // (Mediumname, Medienpfad) des zuletzt umgestellten Includes
        for (i, line) in text.split_inclusive('\n').enumerate() {
            let body = line.trim_end_matches(['\n', '\r']); let eol = &line[body.len()..];
            let Some((a, p, b)) = path_of(body).filter(|(_, p, _)| p.contains("build/export/")) else { if !body.trim().is_empty() { last_main = None; } out.push_str(line); continue };
            let file = p.rsplit('/').next().unwrap_or(&p).to_string();
            let is_anim = file.ends_with(".anim.asm");
            let stem = file.strip_suffix(".anim.asm").or_else(|| file.strip_suffix(".data.bas")).or_else(|| file.strip_suffix(".asm")).or_else(|| file.strip_suffix(".bas")).unwrap_or(&file).to_string();
            if is_anim {
                if last_main.as_ref().map_or(false, |(s, _)| *s == stem) {
                    changes.push(IncChange { file: rel.clone(), line: i + 1, old: body.trim().to_string(), new: None, skipped: None });
                    changed = true; continue;      // Zeile entfällt
                }
                changes.push(IncChange { file: rel.clone(), line: i + 1, old: body.trim().to_string(), new: None, skipped: Some(vc_i18n::t!("media-include-migrate-anim-allein")) });
                out.push_str(line); continue;
            }
            match legacy_media(root, Path::new(&p)) {
                Some(m) => {
                    let nl = format!("{}{}{}", &body[..a], m, &body[b..]);
                    changes.push(IncChange { file: rel.clone(), line: i + 1, old: body.trim().to_string(), new: Some(nl.trim().to_string()), skipped: None });
                    out.push_str(&nl); out.push_str(eol); changed = true; last_main = Some((stem, m));
                }
                None => {
                    changes.push(IncChange { file: rel.clone(), line: i + 1, old: body.trim().to_string(), new: None, skipped: Some(vc_i18n::t!("media-include-migrate-nicht-eindeutig", p1 = &stem)) });
                    out.push_str(line); last_main = None;
                }
            }
        }
        if changed && apply { vc_core::atomar::schreiben(&f, out).map_err(|e| format!("{rel}: {e}"))?; }
    }
    Ok(changes)
}
