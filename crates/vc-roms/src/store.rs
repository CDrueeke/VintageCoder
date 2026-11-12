// SPDX-License-Identifier: GPL-3.0-or-later
//! `RomStore`: Suchorte scannen, Dateien erkennen (Größe + CRC-32 + SHA-1 gegen die Tabelle), Rollen auflösen.
use crate::env::{Layout, Place, RomEnv, Stage};
use crate::hash;
use crate::known::{KnownRom, KnownTable};
use crate::systems::{self, OS_ALTERNATIVES};
use std::path::{Path, PathBuf};

/// Dateien über dieser Größe werden nicht erkannt (ROM-Abbilder sind höchstens 128 KiB)
const MAX_READ: u64 = 1 << 20;

#[derive(Debug, Clone, PartialEq)]
pub enum Ident {
    /// Prüfsumme in der Tabelle (Index in [`RomStore::known`])
    Known(usize),
    /// Größe und Name passen, Prüfsumme unbekannt: benutzbar, mit Warnung
    UnknownRevision { system: String, role: String },
    /// Größe passt zu einer Rolle, die Rolle ist aber nicht ableitbar (z. B. zwei Rollen mit 8192 Byte)
    Ambiguous { system: String, candidates: Vec<String> },
    /// Name legt eine Rolle nahe, die Größe stimmt nicht (abgeschnitten, falsche Datei)
    WrongSize { system: String, role: String, expected: Vec<usize> },
    /// nicht zuzuordnen
    Unrecognized(String),
}

/// Eine gefundene Datei
#[derive(Debug, Clone)]
pub struct Found {
    pub path: PathBuf,
    pub stage: Stage,
    pub place: usize,
    pub size: u64,
    pub crc32: Option<u32>,
    pub sha1: Option<String>,
    /// System aus dem Ordner (`roms/c64/…`)
    pub hint: Option<String>,
    pub ident: Ident,
    /// im `roms.json` des Ortes festgenagelt
    pub pinned: bool,
}

impl Found {
    pub fn file_name(&self) -> String { self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Prüfsumme bekannt, Original
    Recognized,
    UnknownRevision,
    /// freie Ersatz-ROM (Open-ROMs, Altirra …)
    Replacement,
    Missing,
    /// Datei da, aber unbrauchbar (falsche Größe)
    Unusable,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self { Status::Recognized => "Erkannt", Status::UnknownRevision => "Unbekannte Revision", Status::Replacement => "Ersatz (frei)", Status::Missing => "Fehlt", Status::Unusable => "Nicht benutzbar" }
    }
    /// Benutzbar (Datei vorhanden und passend)
    pub fn usable(self) -> bool { matches!(self, Status::Recognized | Status::UnknownRevision | Status::Replacement) }
}

/// Ergebnis für eine Rolle
#[derive(Debug, Clone)]
pub struct Resolved {
    pub system: String,
    pub role: String,
    pub status: Status,
    pub path: Option<PathBuf>,
    pub stage: Option<Stage>,
    pub place_label: Option<String>,
    pub size: Option<u64>,
    pub crc32: Option<u32>,
    pub sha1: Option<String>,
    /// Bezeichnung der erkannten Revision
    pub label: Option<String>,
    pub known_id: Option<String>,
    /// erwartete Größen
    pub expected: Vec<usize>,
    /// alle weiteren Treffer derselben Rolle (der benutzte steht nicht darin)
    pub others: Vec<Found>,
}

impl Resolved {
    pub fn is_usable(&self) -> bool { self.status.usable() }

    /// Kurze Zeile für Konsole/Protokoll: „c64/kernal: Erkannt – C64 KERNAL 901227-03 (Benutzerordner: /pfad)“
    pub fn describe(&self) -> String {
        let mut s = format!("{}/{}: {}", self.system, self.role, self.status.label());
        if let Some(l) = &self.label { s.push_str(&format!(" – {l}")); }
        if let (Some(p), Some(st)) = (&self.path, self.stage) { s.push_str(&format!(" ({}: {})", st.label(), p.display())); }
        if self.status == Status::Missing { s.push_str(&format!(" – erwartet {} Byte", self.expected.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" oder "))); }
        s
    }
}

/// Ein durchsuchter Ort für die Diagnose („Gesuchte Orte“)
#[derive(Debug, Clone)]
pub struct Searched { pub stage: Stage, pub label: String, pub path: PathBuf, pub exists: bool, pub files: usize }

pub struct RomStore {
    pub env: RomEnv,
    pub known: KnownTable,
    places: Vec<Place>,
    found: Vec<Found>,
}

impl RomStore {
    /// Mit der eingebauten Tabelle bzw. `<data>/roms/known.json`
    pub fn new(env: RomEnv) -> RomStore {
        let known = KnownTable::load(env.data_dir.as_deref());
        Self::with_known(env, known)
    }

    /// Mit einer eigenen Tabelle (Tests mit synthetischen Daten)
    pub fn with_known(env: RomEnv, known: KnownTable) -> RomStore {
        let mut s = RomStore { env, known, places: vec![], found: vec![] };
        s.rescan();
        s
    }

    /// Alle Orte neu einlesen („Neu prüfen“)
    pub fn rescan(&mut self) {
        self.places = self.env.places();
        let mut found = vec![];
        for (i, p) in self.places.iter().enumerate() { scan_place(&self.known, i, p, &mut found, &|| false); }
        self.found = found;
    }

    pub fn places(&self) -> &[Place] { &self.places }
    pub fn found(&self) -> &[Found] { &self.found }

    pub fn known_entry(&self, f: &Found) -> Option<&KnownRom> { if let Ident::Known(i) = f.ident { self.known.roms.get(i) } else { None } }

    /// System und Rolle, für die diese Datei taugt
    pub fn role_of(&self, f: &Found) -> Option<(String, String)> {
        match &f.ident {
            // Liegt die Datei in einem Systemordner (`roms/mega65/…`, VICE `C128/…`), zählt sie nur für dieses System: ein C64-Zeichensatz im Ordner von Xemu
            // ist keine ROM für VICE. Ohne Ordner (Einzeldatei, lose Datei) gilt die erste Verwendung der Tabelle.
            Ident::Known(i) => {
                let k = &self.known.roms[*i];
                let u = match f.hint.as_deref() { Some(h) => k.uses.iter().find(|u| u.system == h)?, None => &k.uses[0] };
                Some((u.system.clone(), u.role.clone()))
            }
            Ident::UnknownRevision { system, role } => Some((system.clone(), role.clone())),
            _ => None,
        }
    }

    fn matches(&self, f: &Found, system: &str, role: &str) -> bool {
        match &f.ident {
            Ident::Known(i) => f.hint.as_deref().map_or(true, |h| h == system) && self.known.roms[*i].uses.iter().any(|u| u.system == system && u.role == role),
            Ident::UnknownRevision { system: s, role: r } => s == system && r == role,
            _ => false,
        }
    }

    /// Sortierschlüssel: Stufe vor Festnageln vor Güte (Original-Standard, Original, Unbekannt, Ersatz)
    fn rank(&self, f: &Found) -> (Stage, u8, u8, String) {
        let quality = match &f.ident {
            Ident::Known(i) => { let k = &self.known.roms[*i]; if k.free || f.stage == Stage::Bundled { 3 } else if k.default { 0 } else { 1 } }
            _ => if f.stage == Stage::Bundled { 3 } else { 2 },
        };
        (f.stage, if f.pinned { 0 } else { 1 }, quality, f.file_name())
    }

    /// Rolle auflösen. Atari: Rolle `os` = erste vorhandene von `os-xl`, `os-b`, `os-a`.
    pub fn resolve(&self, system: &str, role: &str) -> Resolved {
        if system == "atari8" && role == "os" {
            let all: Vec<Resolved> = OS_ALTERNATIVES.iter().map(|r| self.resolve_one(system, r)).collect();
            if let Some(r) = all.iter().find(|r| r.is_usable()) { return r.clone(); }
            return all.into_iter().find(|r| r.status == Status::Unusable).unwrap_or_else(|| Resolved { role: "os".into(), ..self.resolve_one(system, "os-xl") });
        }
        self.resolve_one(system, role)
    }

    fn resolve_one(&self, system: &str, role: &str) -> Resolved {
        let expected = systems::role(system, role).map(|r| r.sizes.to_vec()).unwrap_or_default();
        let mut hits: Vec<&Found> = self.found.iter().filter(|f| self.matches(f, system, role)).collect();
        hits.sort_by_key(|f| self.rank(f));
        let mut r = Resolved { system: system.into(), role: role.into(), status: Status::Missing, path: None, stage: None, place_label: None, size: None, crc32: None, sha1: None, label: None, known_id: None, expected, others: vec![] };
        if let Some(best) = hits.first() {
            let k = self.known_entry(best);
            r.status = match k { Some(k) if k.free || best.stage == Stage::Bundled => Status::Replacement, Some(_) => Status::Recognized, None if best.stage == Stage::Bundled => Status::Replacement, None => Status::UnknownRevision };
            r.path = Some(best.path.clone());
            r.stage = Some(best.stage);
            r.place_label = Some(self.places[best.place].label.clone());
            r.size = Some(best.size);
            r.crc32 = best.crc32;
            r.sha1 = best.sha1.clone();
            r.label = k.map(|k| k.label.clone());
            r.known_id = k.map(|k| k.id.clone());
            r.others = hits[1..].iter().map(|f| (*f).clone()).collect();
        } else if let Some(w) = self.found.iter().find(|f| matches!(&f.ident, Ident::WrongSize { system: s, role: ro, .. } if s == system && ro == role)) {
            r.status = Status::Unusable;
            r.path = Some(w.path.clone());
            r.stage = Some(w.stage);
            r.size = Some(w.size);
        }
        r
    }

    /// Alle Rollen aller Systeme (für Statusanzeige und `vcc roms list`)
    pub fn status_all(&self) -> Vec<Resolved> {
        systems::SYSTEMS.iter().flat_map(|s| s.roles.iter().map(move |r| (s.id, r.id))).map(|(s, r)| self.resolve(s, r)).collect()
    }

    /// Fehlende **erforderliche** Rollen eines Systems (Atari: fehlt jedes Betriebssystem, steht `os` darin)
    pub fn missing_required(&self, system: &str) -> Vec<String> {
        let Some(sd) = systems::system(system) else { return vec![] };
        let mut v: Vec<String> = sd.roles.iter().filter(|r| r.required && !self.resolve(system, r.id).is_usable()).map(|r| r.id.to_string()).collect();
        if system == "atari8" && !self.resolve(system, "os").is_usable() { v.push("os".into()); }
        v
    }

    /// Gesuchte Orte mit Treffern
    pub fn searched(&self) -> Vec<Searched> {
        self.places.iter().enumerate().map(|(i, p)| Searched { stage: p.stage, label: p.label.clone(), path: p.path.clone(), exists: p.path.exists(), files: self.found.iter().filter(|f| f.place == i).count() }).collect()
    }

    /// Wie [`RomStore::identify_file`] für Bytes im Speicher (Dateien aus einem ZIP-Archiv); `name` ist nur der Dateiname
    pub fn identify_bytes(&self, name: &str, data: &[u8], hint: Option<&str>) -> Option<Found> {
        if data.len() as u64 > MAX_READ { return None; }
        let lower = name.to_lowercase();
        let named = hint.is_some_and(|h| !role_by_name(h, &lower).is_empty());
        if !systems::all_sizes().contains(&data.len()) && !named { return None; }
        let mut f = Found { path: PathBuf::from(name), stage: Stage::Explicit, place: 0, size: data.len() as u64, crc32: Some(hash::crc32(data)), sha1: Some(hash::sha1_hex(data)), hint: hint.map(str::to_string), ident: Ident::Unrecognized(String::new()), pinned: false };
        f.ident = identify(&self.known, &f, data, hint, &lower);
        Some(f)
    }

    /// Schnelle Prüfung einer einzelnen Datei (Import, Drag-and-drop): ohne Ort, ohne Systemordner
    pub fn identify_file(&self, path: &Path, hint: Option<&str>) -> Option<Found> {
        let mut out = vec![];
        scan_file(&self.known, 0, Stage::Explicit, path, hint, false, &mut out);
        out.pop()
    }
}

/// Datei für eine Rolle festnageln: schreibt `{"system": {"rolle": "datei"}}` in `roms.json` im ROM-Ordner `root` (`file` = Name oder Pfad relativ zu `root`)
pub fn pin(root: &Path, system: &str, role: &str, file: Option<&str>) -> Result<(), String> {
    let path = root.join("roms.json");
    let mut v: serde_json::Value = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| serde_json::json!({}));
    if !v.is_object() { v = serde_json::json!({}); }
    let obj = v.as_object_mut().unwrap();
    let sys = obj.entry(system.to_string()).or_insert_with(|| serde_json::json!({}));
    if !sys.is_object() { *sys = serde_json::json!({}); }
    match file { Some(f) => { sys[role] = serde_json::json!(f); } None => { sys.as_object_mut().unwrap().remove(role); } }
    std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))
}

/// Mehrere Orte scannen (ohne Store), z. B. fremde Fundorte für den Import
pub fn scan_places(known: &KnownTable, places: &[Place]) -> Vec<Found> { scan_places_until(known, places, &|| false) }

/// Wie [`scan_places`], bricht ab, sobald `stop()` wahr wird (Abbrechen-Knopf, Zeitgrenze); bis dahin Gefundenes bleibt erhalten
pub fn scan_places_until(known: &KnownTable, places: &[Place], stop: &dyn Fn() -> bool) -> Vec<Found> {
    let mut found = vec![];
    for (i, p) in places.iter().enumerate() { if stop() { break; } scan_place(known, i, p, &mut found, stop); }
    found
}

/// Unterordner eines Ortes (eine Ebene tief), sortiert
fn list_dir(p: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (mut files, mut dirs) = (vec![], vec![]);
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') { continue; }
            if path.is_dir() { dirs.push(path) } else if path.is_file() { files.push(path) }
        }
    }
    files.sort();
    dirs.sort();
    (files, dirs)
}

fn read_pins(root: &Path) -> Vec<(String, String, String)> {
    let Ok(text) = std::fs::read_to_string(root.join("roms.json")) else { return vec![] };
    let Ok(serde_json::Value::Object(m)) = serde_json::from_str::<serde_json::Value>(&text) else { return vec![] };
    let mut v = vec![];
    for (sys, roles) in m { if let serde_json::Value::Object(rm) = roles { for (role, f) in rm { if let Some(f) = f.as_str() { v.push((sys.clone(), role, f.to_string())); } } } }
    v
}

fn scan_place(known: &KnownTable, idx: usize, place: &Place, out: &mut Vec<Found>, stop: &dyn Fn() -> bool) {
    let start = out.len();
    match &place.layout {
        Layout::File { system, .. } => scan_file(known, idx, place.stage, &place.path, Some(system), false, out),
        Layout::Dir(system) => scan_dir(known, idx, place.stage, &place.path, Some(system), out, stop),
        Layout::Tree | Layout::Mapped(_) => {
            let (files, dirs) = list_dir(&place.path);
            for f in files { if stop() { return; } if !is_ignored(&f) { scan_file(known, idx, place.stage, &f, None, false, out); } }
            for d in dirs {
                if stop() { return; }
                let name = d.file_name().unwrap().to_string_lossy().into_owned();
                let sys = match &place.layout {
                    Layout::Mapped(m) => m.iter().find(|(a, _)| a.eq_ignore_ascii_case(&name)).map(|(_, b)| b.clone()),
                    _ => systems::system(&name.to_lowercase()).map(|s| s.id.to_string()),
                };
                match sys { Some(s) => scan_dir(known, idx, place.stage, &d, Some(&s), out, stop), None => {} }
            }
        }
    }
    // festgenagelte Dateien
    if matches!(place.layout, Layout::Tree) {
        let pins = read_pins(&place.path);
        for f in &mut out[start..] {
            let name = f.file_name();
            let rel = f.path.strip_prefix(&place.path).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            if pins.iter().any(|(_, _, p)| *p == name || *p == rel) { f.pinned = true; }
        }
    }
}

fn is_ignored(p: &Path) -> bool {
    let n = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    n == "roms.json" || n.ends_with(".txt") || n.ends_with(".md") || n.ends_with(".json")
}

/// Ordner eines Systems: Dateien und eine Ebene Unterordner
fn scan_dir(known: &KnownTable, idx: usize, stage: Stage, dir: &Path, hint: Option<&str>, out: &mut Vec<Found>, stop: &dyn Fn() -> bool) {
    let (files, dirs) = list_dir(dir);
    for f in files { if stop() { return; } if !is_ignored(&f) { scan_file(known, idx, stage, &f, hint, false, out); } }
    for d in dirs { let (files, _) = list_dir(&d); for f in files { if stop() { return; } if !is_ignored(&f) { scan_file(known, idx, stage, &f, hint, false, out); } } }
}

/// Name enthält ein Rollenwort?
fn role_by_name(system: &str, file_lower: &str) -> Vec<&'static str> {
    let Some(sd) = systems::system(system) else { return vec![] };
    // genauere (längere) Wörter zuerst: „kernal64“ vor „kernal“, „basiclo“ vor „basic“
    let mut best: Vec<(usize, &'static str)> = vec![];
    for r in sd.roles {
        let stem_conv = r.file.rsplit_once('.').map(|(s, _)| s).unwrap_or(r.file);
        for kw in r.keywords.iter().copied().chain(std::iter::once(stem_conv)) {
            if file_lower.contains(kw) { best.push((kw.len(), r.id)); }
        }
    }
    let max = best.iter().map(|b| b.0).max().unwrap_or(0);
    let mut v: Vec<&'static str> = best.into_iter().filter(|b| b.0 == max).map(|b| b.1).collect();
    v.dedup();
    v
}

fn scan_file(known: &KnownTable, idx: usize, stage: Stage, path: &Path, hint: Option<&str>, pinned: bool, out: &mut Vec<Found>) {
    let Ok(meta) = std::fs::metadata(path) else { return };
    if !meta.is_file() { return; }
    let size = meta.len();
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let mut f = Found { path: path.to_path_buf(), stage, place: idx, size, crc32: None, sha1: None, hint: hint.map(str::to_string), ident: Ident::Unrecognized("keine ROM-Größe".into()), pinned };
    if size > MAX_READ { f.ident = Ident::Unrecognized("zu groß für ein ROM-Abbild".into()); out.push(f); return; }
    let size_known = systems::all_sizes().contains(&(size as usize));
    let named = hint.is_some_and(|h| !role_by_name(h, &name).is_empty());
    if !size_known && !named { out.push(f); return; }
    let Ok(data) = std::fs::read(path) else { f.ident = Ident::Unrecognized("nicht lesbar".into()); out.push(f); return; };
    f.size = data.len() as u64;
    f.crc32 = Some(hash::crc32(&data));
    f.sha1 = Some(hash::sha1_hex(&data));
    f.ident = identify(known, &f, &data, hint, &name);
    out.push(f);
}

fn identify(known: &KnownTable, f: &Found, data: &[u8], hint: Option<&str>, name_lower: &str) -> Ident {
    if let Some(k) = known.find(data.len(), f.crc32.unwrap_or(0), f.sha1.as_deref().unwrap_or("")) {
        return Ident::Known(known.roms.iter().position(|x| std::ptr::eq(x, k)).unwrap());
    }
    let size = data.len();
    // 1. Ordner + Name: Rolle ableiten, Größe prüfen
    if let Some(h) = hint {
        let sd = systems::system(h);
        let by_name = role_by_name(h, name_lower);
        if let (Some(sd), [only]) = (sd, by_name.as_slice()) {
            let r = sd.roles.iter().find(|r| r.id == *only).unwrap();
            return if r.sizes.contains(&size) { Ident::UnknownRevision { system: h.into(), role: r.id.into() } }
                   else { Ident::WrongSize { system: h.into(), role: r.id.into(), expected: r.sizes.to_vec() } };
        }
        // 2. Ordner + Größe: eindeutig?
        if let Some(sd) = sd {
            let by_size: Vec<&str> = sd.roles.iter().filter(|r| r.sizes.contains(&size)).map(|r| r.id).collect();
            let both: Vec<&str> = if by_name.is_empty() { by_size.clone() } else { by_size.iter().copied().filter(|r| by_name.contains(r)).collect() };
            match both.as_slice() {
                [one] => return Ident::UnknownRevision { system: h.into(), role: (*one).into() },
                [] => {}
                many => return Ident::Ambiguous { system: h.into(), candidates: many.iter().map(|s| s.to_string()).collect() },
            }
        }
        return Ident::Unrecognized(format!("passt zu keiner Rolle von {h} (Größe {size} Byte)"));
    }
    // ohne Ordner: nur eindeutige Größe + Name über alle Systeme ist zu unsicher
    let cands: Vec<String> = systems::SYSTEMS.iter().filter(|s| s.roles.iter().any(|r| r.sizes.contains(&size))).map(|s| s.id.to_string()).collect();
    if cands.is_empty() { Ident::Unrecognized("keine ROM-Größe".into()) } else { Ident::Ambiguous { system: String::new(), candidates: cands } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::known::Use;

    /// Synthetische „ROMs“: richtige Größe, erkennbares Muster, keine echten Daten
    fn fake(size: usize, seed: u8) -> Vec<u8> { (0..size).map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed)).collect() }

    fn entry(id: &str, data: &[u8], sys: &str, role: &str, default: bool, free: bool) -> KnownRom {
        KnownRom { id: id.into(), label: id.into(), size: data.len(), crc32: hash::crc32_hex(data), sha1: hash::sha1_hex(data), uses: vec![Use { system: sys.into(), role: role.into() }], default, free, note: String::new(), sets: Default::default() }
    }

    struct Tmp(PathBuf);
    impl Tmp {
        fn new(n: &str) -> Tmp { let p = std::env::temp_dir().join(format!("vc-roms-{n}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&p); std::fs::create_dir_all(&p).unwrap(); Tmp(p) }
        fn put(&self, rel: &str, data: &[u8]) { let p = self.0.join(rel); std::fs::create_dir_all(p.parent().unwrap()).unwrap(); std::fs::write(p, data).unwrap(); }
    }
    impl Drop for Tmp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    fn env_with(user: &Path) -> RomEnv { RomEnv { vc_roms: Some(user.to_path_buf()), ..Default::default() } }

    #[test]
    fn erkennung_nach_pruefsumme_auch_mit_beliebigem_namen() {
        let t = Tmp::new("erk");
        let (k1, k2) = (fake(8192, 1), fake(8192, 2));
        t.put("c64/irgendwas.bin", &k1);
        t.put("c64/anders.rom", &k2);
        let known = KnownTable { format: 1, roms: vec![entry("c64.kernal.a", &k1, "c64", "kernal", true, false), entry("c64.basic.b", &k2, "c64", "basic", true, false)] };
        let s = RomStore::with_known(env_with(&t.0), known);
        let k = s.resolve("c64", "kernal");
        assert_eq!(k.status, Status::Recognized);
        assert!(k.path.unwrap().ends_with("irgendwas.bin"));
        assert_eq!(s.resolve("c64", "basic").status, Status::Recognized);
        assert_eq!(s.resolve("c64", "chargen").status, Status::Missing);
        assert_eq!(s.missing_required("c64"), vec!["chargen".to_string()]);
    }

    #[test]
    fn unbekannte_revision_nach_name_und_groesse() {
        let t = Tmp::new("unb");
        t.put("c64/kernal.bin", &fake(8192, 9));
        t.put("c64/basic.bin", &fake(8192, 8));
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![] });
        assert_eq!(s.resolve("c64", "kernal").status, Status::UnknownRevision);
        assert_eq!(s.resolve("c64", "basic").status, Status::UnknownRevision);
        // gleiche Größe, aber beliebiger Name: Rolle mehrdeutig, nicht benutzt
        t.put("c64/xyz.bin", &fake(8192, 7));
        let mut s = s; s.rescan();
        let f = s.found().iter().find(|f| f.file_name() == "xyz.bin").unwrap();
        assert!(matches!(f.ident, Ident::Ambiguous { .. }), "{:?}", f.ident);
    }

    #[test]
    fn falsche_groesse_ist_unbrauchbar() {
        let t = Tmp::new("gr");
        t.put("c64/kernal.bin", &fake(4000, 1));
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![] });
        let r = s.resolve("c64", "kernal");
        assert_eq!(r.status, Status::Unusable);
        assert_eq!(r.expected, vec![8192]);
    }

    #[test]
    fn mega65_rom_131072() {
        let t = Tmp::new("m65");
        t.put("mega65/mega65.rom", &fake(131072, 3));
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![] });
        let r = s.resolve("mega65", "rom");
        assert_eq!(r.status, Status::UnknownRevision);
        assert_eq!(r.size, Some(131072));
        assert_eq!(s.missing_required("mega65"), Vec::<String>::new());
    }

    #[test]
    fn atari_rolle_os_nimmt_die_erste_vorhandene() {
        let t = Tmp::new("atari");
        t.put("atari8/os-b.rom", &fake(10240, 5));
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![] });
        assert_eq!(s.resolve("atari8", "os").role, "os-b");
        assert!(s.missing_required("atari8").is_empty());
        t.put("atari8/os-xl.rom", &fake(16384, 6));
        let mut s = s; s.rescan();
        assert_eq!(s.resolve("atari8", "os").role, "os-xl");
        let leer = RomStore::with_known(env_with(&t.0.join("nix")), KnownTable { format: 1, roms: vec![] });
        assert_eq!(leer.missing_required("atari8"), vec!["os".to_string()]);
    }

    #[test]
    fn suchreihenfolge_projekt_vor_benutzer_vor_ersatz() {
        let (p, u, d) = (Tmp::new("prj"), Tmp::new("usr"), Tmp::new("dat"));
        let (a, b, c) = (fake(8192, 11), fake(8192, 12), fake(8192, 13));
        p.put("roms/c64/kernal.bin", &a);
        u.put("c64/kernal.bin", &b);
        d.put("roms-open/c64/kernal.bin", &c);
        let known = KnownTable { format: 1, roms: vec![entry("p", &a, "c64", "kernal", false, false), entry("u", &b, "c64", "kernal", true, false), entry("o", &c, "c64", "kernal", false, true)] };
        let env = RomEnv { project_dir: Some(p.0.clone()), vc_roms: Some(u.0.clone()), data_dir: Some(d.0.clone()), ..Default::default() };
        let s = RomStore::with_known(env.clone(), known.clone());
        let r = s.resolve("c64", "kernal");
        assert_eq!(r.stage, Some(Stage::Project));
        assert_eq!(r.others.len(), 2);
        // ohne Projekt: Benutzerordner, dann (nur noch) Ersatz
        let s = RomStore::with_known(RomEnv { project_dir: None, ..env.clone() }, known.clone());
        assert_eq!(s.resolve("c64", "kernal").stage, Some(Stage::User));
        std::fs::remove_file(u.0.join("c64/kernal.bin")).unwrap();
        let s = RomStore::with_known(RomEnv { project_dir: None, ..env }, known);
        let r = s.resolve("c64", "kernal");
        assert_eq!((r.stage, r.status), (Some(Stage::Bundled), Status::Replacement));
    }

    #[test]
    fn mehrere_treffer_standard_gewinnt_und_festnageln_ueberstimmt() {
        let t = Tmp::new("multi");
        let (a, b) = (fake(8192, 21), fake(8192, 22));
        t.put("c64/kernal-a.bin", &a);
        t.put("c64/kernal-b.bin", &b);
        let known = KnownTable { format: 1, roms: vec![entry("a", &a, "c64", "kernal", false, false), entry("b", &b, "c64", "kernal", true, false)] };
        let s = RomStore::with_known(env_with(&t.0), known.clone());
        assert_eq!(s.resolve("c64", "kernal").known_id.as_deref(), Some("b"));
        t.put("roms.json", br#"{"c64": {"kernal": "kernal-a.bin"}}"#);
        let s = RomStore::with_known(env_with(&t.0), known);
        let r = s.resolve("c64", "kernal");
        assert_eq!(r.known_id.as_deref(), Some("a"));
        assert_eq!(r.others.len(), 1);
    }

    #[test]
    fn mehrfachnutzung_im_c128_ordner_zaehlt_nur_fuer_den_c128() {
        let t = Tmp::new("c128");
        let k = fake(8192, 31);
        t.put("c128/kernal64.bin", &k);
        let mut e = entry("c64.kernal.x", &k, "c64", "kernal", true, false);
        e.uses.push(Use { system: "c128".into(), role: "kernal64".into() });
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![e] });
        assert_eq!(s.resolve("c128", "kernal64").status, Status::Recognized);
        // der Ordner entscheidet: im C128-Ordner gefunden zählt der Inhalt nur für den C128, nicht für den C64
        assert_eq!(s.resolve("c64", "kernal").status, Status::Missing);
        // lose Datei ohne Systemordner: erste Verwendung der Tabelle
        t.put("lose.bin", &k);
        let s2 = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![entry("c64.kernal.x", &k, "c64", "kernal", true, false)] });
        assert_eq!(s2.resolve("c64", "kernal").status, Status::Recognized);
    }

    #[test]
    fn diagnose_zaehlt_orte_und_dateien() {
        let t = Tmp::new("diag");
        t.put("c64/kernal.bin", &fake(8192, 1));
        let s = RomStore::with_known(env_with(&t.0), KnownTable { format: 1, roms: vec![] });
        let d = s.searched();
        assert_eq!(d.len(), 1);
        assert!(d[0].exists);
        assert_eq!(d[0].files, 1);
        assert_eq!(d[0].label, "VC_ROMS");
    }

    #[test]
    fn festnageln_schreibt_roms_json_und_gilt_beim_naechsten_scan() {
        let t = Tmp::new("pin");
        let (a, b) = (fake(8192, 41), fake(8192, 42));
        t.put("c64/kernal-a.bin", &a); t.put("c64/kernal-b.bin", &b);
        let known = KnownTable { format: 1, roms: vec![entry("a", &a, "c64", "kernal", true, false), entry("b", &b, "c64", "kernal", false, false)] };
        assert_eq!(RomStore::with_known(env_with(&t.0), known.clone()).resolve("c64", "kernal").known_id.as_deref(), Some("a"));
        pin(&t.0, "c64", "kernal", Some("kernal-b.bin")).unwrap();
        assert_eq!(RomStore::with_known(env_with(&t.0), known.clone()).resolve("c64", "kernal").known_id.as_deref(), Some("b"));
        pin(&t.0, "c64", "kernal", None).unwrap();
        assert_eq!(RomStore::with_known(env_with(&t.0), known).resolve("c64", "kernal").known_id.as_deref(), Some("a"));
    }

    #[test]
    fn bytes_im_speicher_werden_wie_dateien_erkannt() {
        let d = fake(8192, 51);
        let known = KnownTable { format: 1, roms: vec![entry("k", &d, "c64", "kernal", true, false)] };
        let s = RomStore::with_known(RomEnv::default(), known);
        let f = s.identify_bytes("irgendwas.bin", &d, None).unwrap();
        assert_eq!(s.role_of(&f), Some(("c64".to_string(), "kernal".to_string())));
        assert!(s.identify_bytes("x.bin", &[1, 2, 3], None).is_none());
    }

    #[test]
    fn scan_laesst_sich_abbrechen() {
        let t = Tmp::new("stop");
        for i in 0..20u8 { t.put(&format!("c64/kernal-{i}.bin"), &fake(8192, i)); }
        let known = KnownTable { format: 1, roms: vec![] };
        let places = RomEnv { vc_roms: Some(t.0.clone()), ..Default::default() }.places();
        assert_eq!(scan_places_until(&known, &places, &|| false).len(), 20);
        assert!(scan_places_until(&known, &places, &|| true).is_empty(), "sofort abgebrochen: nichts gelesen");
        let n = std::cell::Cell::new(0);
        let some = scan_places_until(&known, &places, &|| { n.set(n.get() + 1); n.get() > 5 });
        assert!(!some.is_empty() && some.len() < 20, "nach einigen Dateien abgebrochen: {}", some.len());
    }

    /// Fehler aus dem Windows-Test: `CHARROM.M65` (= C64-Zeichensatz) im Xemu-Ordner wurde VICE als `-chargen` übergeben
    #[test]
    fn c64_zeichensatz_im_mega65_ordner_ist_keine_c64_rom() {
        let t = Tmp::new("m65dir");
        let chars = fake(4096, 61);
        t.put("CHARROM.M65", &chars);
        let known = KnownTable { format: 1, roms: vec![entry("c64.chargen.x", &chars, "c64", "chargen", true, false)] };
        let env = RomEnv { config_dir: Some(t.0.clone()), ..Default::default() };   // Altablage `<config>/mega65` gibt es hier nicht: als eigene Stelle nachbilden
        let _ = env;
        let place = crate::env::Place { stage: Stage::Legacy, label: "Xemu".into(), path: t.0.clone(), layout: crate::env::Layout::Dir("mega65".into()) };
        let found = scan_places_until(&known, &[place], &|| false);
        assert_eq!(found.len(), 1);
        let s = RomStore::with_known(RomEnv::default(), known);
        assert_eq!(s.role_of(&found[0]), None, "im Ordner mega65 gefunden: keine Rolle des C64");
        assert!(matches!(found[0].ident, Ident::Known(_)), "der Inhalt selbst ist trotzdem bekannt");
    }

    #[test]
    fn eingebaute_tabelle_ist_gueltig() { KnownTable::builtin(); }
}
