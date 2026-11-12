// SPDX-License-Identifier: GPL-3.0-or-later
//! Verweise auf Zeichensätze in ROM-Abbildern des Anwenders (`rom:c64/chargen#upper`) und Zeichensatzdateien im Format 2 (`base` + `overrides`).
//! Die Original-Glyphen stehen nie in der Projektdatei: gespeichert werden nur die eigenen Änderungen.
//!
//! Verweis: `rom:<system>/<rolle>[@<revision>]#<satz>`. Ohne Revision gilt die vom ROM-Verzeichnis gewählte Datei der Rolle, mit Revision genau dieses
//! Abbild (sonst Fehler „Abbild fehlt“). Atari: die Rolle `os` meint das erste vorhandene Betriebssystem, das den Satz enthält.
use serde_json::{json, Map, Value};
use std::sync::{OnceLock, RwLock};
use vc_roms::systems::{self, SetDef};
use vc_roms::{Found, RomEnv, RomStore, Status};

fn cell() -> &'static RwLock<RomStore> { static S: OnceLock<RwLock<RomStore>> = OnceLock::new(); S.get_or_init(|| RwLock::new(RomStore::new(RomEnv::from_process()))) }

/// ROM-Dienst des Prozesses neu einstellen (Programmstart, Projektwechsel, geänderte Einstellung „ROMs anderer Programme mitbenutzen“)
pub fn configure(env: RomEnv) { let fresh = RomStore::new(env); if let Ok(mut g) = cell().write() { *g = fresh; } }

/// Wie [`configure`], im Hintergrund (die Oberfläche wartet nie auf Dateizugriffe; bis zur Fertigstellung gilt der alte Stand)
pub fn configure_async(env: RomEnv) { std::thread::spawn(move || configure(env)); }

/// Alle Orte neu einlesen („Neu prüfen“)
pub fn rescan() { if let Ok(mut g) = cell().write() { g.rescan(); } }

/// Wie [`rescan`], aber im Hintergrund: der neue Stand wird in einem eigenen Thread aufgebaut und erst dann eingetauscht, damit die Oberfläche nie auf Dateizugriffe wartet
pub fn rescan_async() {
    let env = with_store(|s| s.env.clone());
    std::thread::spawn(move || { let fresh = RomStore::new(env); if let Ok(mut g) = cell().write() { *g = fresh; } });
}

pub fn with_store<R>(f: impl FnOnce(&RomStore) -> R) -> R { let g = cell().read().unwrap_or_else(|e| e.into_inner()); f(&g) }

#[derive(Debug, Clone, PartialEq)]
pub struct RomRef { pub system: String, pub role: String, pub rev: Option<String>, pub set: String }

impl RomRef {
    pub fn parse(text: &str) -> Result<RomRef, String> {
        let rest = text.strip_prefix("rom:").ok_or_else(|| format!("'{text}' ist kein ROM-Verweis (rom:system/rolle#satz)"))?;
        let (path, set) = rest.split_once('#').ok_or_else(|| format!("'{text}': Satz fehlt (rom:system/rolle#satz)"))?;
        let (sr, rev) = match path.split_once('@') { Some((a, b)) => (a, Some(b.to_string())), None => (path, None) };
        let (system, role) = sr.split_once('/').ok_or_else(|| format!("'{text}': system/rolle erwartet"))?;
        if system.is_empty() || role.is_empty() || set.is_empty() { return Err(format!("'{text}': unvollständiger ROM-Verweis")); }
        Ok(RomRef { system: system.into(), role: role.into(), rev, set: set.into() })
    }
    pub fn to_text(&self) -> String { format!("rom:{}/{}{}#{}", self.system, self.role, self.rev.as_ref().map(|r| format!("@{r}")).unwrap_or_default(), self.set) }
}

/// Revisionskürzel eines bekannten Abbilds: `c64.chargen.901225-01` → `901225-01`
pub fn revision_of(known_id: &str) -> &str { known_id.splitn(3, '.').nth(2).unwrap_or(known_id) }

#[derive(Debug, Clone, PartialEq)]
pub enum RomError {
    /// kein passendes Abbild gefunden
    Missing(String),
    /// Abbild da, aber der Satz liegt nicht darin (Größe, unbekannter Satz)
    NoSet(String),
}

impl std::fmt::Display for RomError { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { match self { RomError::Missing(m) | RomError::NoSet(m) => f.write_str(m) } } }

/// Gelesener Zeichensatz aus einem ROM-Abbild
#[derive(Debug, Clone)]
pub struct SetData {
    pub set: SetDef,
    /// Bytes des Satzes (`set.size` Stück)
    pub bytes: Vec<u8>,
    /// ganzes Abbild (für den Import mit einem anderen Profil)
    pub rom: Vec<u8>,
    pub status: Status,
    /// Bezeichnung der Revision, falls erkannt
    pub label: Option<String>,
    pub path: std::path::PathBuf,
}

impl SetData {
    /// Hinweistext zu Ersatz-ROM bzw. unbekannter Revision (für Banner und Bau-Warnung)
    pub fn note(&self) -> Option<String> {
        match self.status {
            Status::Replacement if self.path.as_os_str() == BUILTIN_PATH => Some("Eingebaute Ersatzschrift (Eigenbau), nicht die Originalschrift des Systems – ROMs einrichten für das Original".into()),
            Status::Replacement => Some("Ersatzschrift (freie ROM), nicht die Originalschrift des Systems".into()),
            Status::UnknownRevision => Some("unbekannte Revision des Zeichensatz-ROMs".into()),
            _ => None,
        }
    }
}

fn candidates<'a>(store: &'a RomStore, system: &str, role: &str, rev: Option<&str>) -> Vec<&'a Found> {
    store.found().iter().filter(|f| {
        let Some((s, r)) = store.role_of(f).filter(|(s, r)| s == system && r == role) else { return false };
        let _ = (s, r);
        match rev {
            None => true,
            Some(rev) => store.known_entry(f).is_some_and(|k| revision_of(&k.id).eq_ignore_ascii_case(rev)) || f.file_name().to_lowercase().contains(&rev.to_lowercase()),
        }
    }).collect()
}

fn not_found(store: &RomStore, r: &RomRef) -> RomError {
    let rd = systems::role(&r.system, &r.role);
    let size = rd.map(|d| d.sizes.iter().map(|s| format!("{s} Byte")).collect::<Vec<_>>().join(" oder ")).unwrap_or_default();
    let dir = store.env.install_dir().map(|d| d.join(&r.system).display().to_string()).unwrap_or_else(|| format!("roms/{}", r.system));
    let what = match &r.rev { Some(v) => format!(" in der Revision {v}"), None => String::new() };
    RomError::Missing(format!("Zeichensatz-ROM {}/{}{what} fehlt (erwartet {size}); Datei nach {dir} legen oder ROMs einrichten", r.system, r.role))
}

/// Satz aus dem ROM-Verzeichnis lesen
pub fn read_set(store: &RomStore, r: &RomRef) -> Result<SetData, RomError> {
    let roles: Vec<String> = if r.system == "atari8" && r.role == "os" { systems::OS_ALTERNATIVES.iter().map(|s| s.to_string()).collect() } else { vec![r.role.clone()] };
    let mut last: Option<RomError> = None;
    for role in roles {
        // gewähltes Abbild: mit Revision genau dieses, sonst das, was das ROM-Verzeichnis für die Rolle wählt
        let (found, label, status): (Option<Found>, Option<String>, Status) = match &r.rev {
            Some(rev) => match candidates(store, &r.system, &role, Some(rev)).first() {
                Some(f) => { let k = store.known_entry(f); ((*f).clone().into(), k.map(|k| k.label.clone()), status_of(store, f)) }
                None => { last = Some(not_found(store, r)); continue; }
            },
            None => {
                let res = store.resolve(&r.system, &role);
                if !res.is_usable() { last = Some(not_found(store, r)); continue; }
                let path = res.path.clone().unwrap();
                match store.found().iter().find(|f| f.path == path) { Some(f) => (Some(f.clone()), res.label.clone(), res.status), None => { last = Some(not_found(store, r)); continue; } }
            }
        };
        let found = found.unwrap();
        let sets = match store.known_entry(&found) { Some(k) => k.char_sets(&r.system, &role), None => systems::default_sets(&r.system, &role) };
        let Some(set) = sets.into_iter().find(|s| s.name == r.set) else { last = Some(RomError::NoSet(format!("{}/{} enthält keinen Zeichensatz '{}'", r.system, role, r.set))); continue; };
        let rom = std::fs::read(&found.path).map_err(|e| RomError::Missing(format!("{}: {e}", found.path.display())))?;
        let Some(bytes) = rom.get(set.offset..set.offset + set.size) else { last = Some(RomError::NoSet(format!("{}: Zeichensatz '{}' liegt hinter dem Dateiende", found.path.display(), set.name))); continue; };
        return Ok(SetData { bytes: bytes.to_vec(), set, rom, status, label, path: found.path.clone() });
    }
    // Kein ROM des Anwenders: eingebaute Ersatzschrift (Eigenbau), außer es wurde ausdrücklich eine Revision verlangt
    if r.rev.is_none() && matches!(last, None | Some(RomError::Missing(_))) {
        if let Some(d) = builtin_set(r) { return Ok(d); }
    }
    Err(last.unwrap_or_else(|| not_found(store, r)))
}

/// Pfad, den `SetData::path` für die eingebaute Ersatzschrift trägt
pub const BUILTIN_PATH: &str = "(eingebaut)";

/// Eingebaute Ersatzschrift (`vc_roms::fallback`) für einen Verweis, falls es für das Profil des Satzes eine gibt
fn builtin_set(r: &RomRef) -> Option<SetData> {
    let role = if r.system == "atari8" && r.role == "os" { "os-xl" } else { r.role.as_str() };
    let set = systems::default_sets(&r.system, role).into_iter().find(|s| s.name == r.set)?;
    let bytes = vc_roms::fallback::bytes_for(&set)?;
    let set = SetDef { offset: 0, ..set };
    Some(SetData { set, rom: bytes.clone(), bytes, status: Status::Replacement, label: Some("VintageCoder-Ersatzschrift".into()), path: std::path::PathBuf::from(BUILTIN_PATH) })
}

fn status_of(store: &RomStore, f: &Found) -> Status {
    match store.known_entry(f) { Some(k) if k.free || f.stage == vc_roms::Stage::Bundled => Status::Replacement, Some(_) => Status::Recognized, None if f.stage == vc_roms::Stage::Bundled => Status::Replacement, None => Status::UnknownRevision }
}

/// Zeilenzahl je Zeichen und Anzahl der Zeichen eines Satzes
fn geometry(set: &SetDef) -> (usize, usize) { let h = if set.profile.ends_with(".vdc") { 16 } else { 8 }; (h, set.size / h) }

fn frames_from_bytes(bytes: &[u8], h: usize, fcm: bool) -> Vec<Value> {
    bytes.chunks(h).filter(|c| c.len() == h).map(|c| json!({ "rows": c.iter().map(|b| { let s = format!("{b:08b}"); if fcm { s.chars().map(|c| if c == '1' { "01" } else { "00" }).collect::<String>() } else { s } }).collect::<Vec<_>>() })).collect()
}

fn blank_frames(h: usize, count: usize, fcm: bool) -> Vec<Value> { frames_from_bytes(&vec![0u8; h * count], h, fcm) }

/// Zeichensatzdatei im Format 2 auflösen: `base` einlesen, `overrides` darüberlegen. Das Ergebnis trägt die vollständigen `frames` (für Studios und Export)
/// und behält `base`. Fehlt das ROM, kommt ein leeres Raster passender Größe und `rom_missing` (Text für das Banner im Studio); sonst `rom_status`.
pub fn resolve_media(v: &Value) -> Result<Value, String> {
    let base = v["base"].as_str().ok_or("Format 2: base fehlt")?;
    let r = RomRef::parse(base)?;
    let profile = v["profile"].as_str().ok_or("profile fehlt")?;
    let fcm = profile.ends_with(".fcm");
    let (frames, extra) = match with_store(|s| read_set(s, &r)) {
        Ok(d) => {
            if !fcm && d.set.profile != profile && !(profile.starts_with("c128.char.hires") && d.set.profile.ends_with(".hires")) {
                return Err(format!("Zeichensatz-ROM '{base}' (Profil {}) passt nicht zum Profil {profile}", d.set.profile));
            }
            let (h, _) = geometry(&d.set);
            let mut x = Map::new();
            x.insert("rom_status".into(), json!(match d.status { Status::Recognized => "original", Status::UnknownRevision => "unknown_revision", Status::Replacement => "replacement", _ => "other" }));
            if let Some(n) = d.note() { x.insert("rom_note".into(), json!(n)); }
            (frames_from_bytes(&d.bytes, h, fcm), x)
        }
        Err(e) => {
            let sets = systems::default_sets(&r.system, if r.role == "os" { "os-xl" } else { &r.role });
            let set = sets.iter().find(|s| s.name == r.set).or(sets.first()).cloned().ok_or_else(|| e.to_string())?;
            let (h, n) = geometry(&set);
            let mut x = Map::new();
            x.insert("rom_missing".into(), json!(e.to_string()));
            (blank_frames(h, n, fcm), x)
        }
    };
    let mut frames = frames;
    if let Some(o) = v["overrides"].as_object() {
        for (k, rows) in o {
            let i: usize = k.parse().map_err(|_| format!("overrides: '{k}' ist keine Zeichennummer"))?;
            let n = frames.len();
            let f = frames.get_mut(i).ok_or_else(|| format!("overrides: Zeichen {i} gibt es nicht (Satz hat {n})"))?;
            *f = json!({ "rows": rows });
        }
    }
    let mut out = v.as_object().cloned().ok_or("Datei ist kein Objekt")?;
    out.remove("overrides");
    out.insert("frames".into(), Value::Array(frames));
    out.extend(extra);
    Ok(Value::Object(out))
}

/// Form zum Speichern: hat das Media ein `base`, bleiben nur die geänderten Zeichen als `overrides` (Format 2); sonst unverändert.
/// Die ROM-Glyphen selbst werden nie geschrieben.
pub fn for_saving(root: &Value) -> Value {
    let Some(_) = root["base"].as_str().filter(|b| b.starts_with("rom:")) else { return root.clone() };
    let mut plain = root.as_object().cloned().unwrap_or_default();
    plain.remove("frames"); plain.remove("rom_status"); plain.remove("rom_note"); plain.remove("rom_missing"); plain.remove("overrides");
    let Ok(base) = resolve_media(&Value::Object(plain.clone())) else { return root.clone() };
    let (b, f) = (base["frames"].as_array(), root["frames"].as_array());
    let (Some(b), Some(f)) = (b, f) else { return root.clone() };
    let mut over = Map::new();
    for (i, fr) in f.iter().enumerate() { if b.get(i) != Some(fr) { over.insert(i.to_string(), fr["rows"].clone()); } }
    plain.insert("format".into(), json!(2));
    if !over.is_empty() { plain.insert("overrides".into(), Value::Object(over)); }
    Value::Object(plain)
}

/// Neue Zeichensatzdatei (Format 2), die nur auf das ROM verweist
pub fn new_reference(profile_id: &str, name: &str, base: &str, colors: &Value) -> Value {
    json!({ "format": 2, "kind": "charset", "profile": profile_id, "name": name, "colors": colors, "base": base })
}

/// Eine auswählbare Zeichensatzquelle („Aus ROM“)
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub reference: String,
    pub label: String,
    pub system: String,
    pub status: Status,
    /// kurzer Dateiname bzw. „fehlt“
    pub source: String,
}

/// Zeichensatz-Quellen für ein Profil (`c64.char.hires` …): alle gefundenen Abbilder mit ihren Sätzen; was fehlt, steht mit Status `Missing` (Standardsätze der Systeme).
/// MEGA65-Vollfarbe (`.fcm`) bietet die 1-Bit-Sätze des C64 und des MEGA65 an.
pub fn choices(store: &RomStore, profile_id: &str) -> Vec<Choice> {
    let fcm = profile_id.ends_with(".fcm");
    let fits = |set: &SetDef| set.profile == profile_id || (fcm && set.profile == "c64.char.hires") || (profile_id == "c128.char.hires" && set.profile == "c64.char.hires");
    let mut out: Vec<Choice> = vec![];
    let mut seen_missing: Vec<String> = vec![];
    let mut push = |c: Choice| if !out.iter().any(|x| x.reference == c.reference) { out.push(c) };
    // gefundene Abbilder
    for f in store.found() {
        let Some((system, role)) = store.role_of(f) else { continue };
        let (k, status) = (store.known_entry(f), status_of(store, f));
        let sets = match k { Some(k) => k.char_sets(&system, &role), None => systems::default_sets(&system, &role) };
        for set in sets.iter().filter(|s| fits(s)) {
            let best = store.resolve(&system, &role);
            let is_best = best.path.as_ref() == Some(&f.path);
            let rev = k.map(|k| revision_of(&k.id).to_string());
            let reference = RomRef { system: system.clone(), role: role.clone(), rev: if is_best { None } else { rev.clone().or_else(|| Some(f.file_name())) }, set: set.name.clone() }.to_text();
            let label = format!("{}: {}", k.map(|k| k.label.clone()).unwrap_or_else(|| format!("{} {} (unbekannte Revision)", systems::system(&system).map(|s| s.label).unwrap_or(""), systems::role(&system, &role).map(|r| r.label).unwrap_or(""))), set.label);
            push(Choice { reference, label, system: system.clone(), status, source: f.file_name() });
        }
    }
    // fehlende Standardsätze
    for sd in systems::SYSTEMS {
        for rd in sd.roles {
            for set in systems::default_sets(sd.id, rd.id).iter().filter(|s| fits(s)) {
                if store.resolve(sd.id, rd.id).is_usable() { continue; }
                let reference = RomRef { system: sd.id.into(), role: rd.id.into(), rev: None, set: set.name.clone() }.to_text();
                if seen_missing.contains(&reference) { continue; }
                seen_missing.push(reference.clone());
                let builtin = vc_roms::fallback::bytes_for(set).is_some();
                push(Choice { reference, label: format!("{} {}: {}{}", sd.label, rd.label, set.label, if builtin { " (Ersatzschrift)" } else { "" }), system: sd.id.into(), status: if builtin { Status::Replacement } else { Status::Missing }, source: if builtin { "Ersatz (eingebaut)".into() } else { "fehlt".into() } });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verweis_hin_und_zurueck() {
        let r = RomRef::parse("rom:c64/chargen@906143-02#lower").unwrap();
        assert_eq!((r.system.as_str(), r.role.as_str(), r.rev.as_deref(), r.set.as_str()), ("c64", "chargen", Some("906143-02"), "lower"));
        assert_eq!(r.to_text(), "rom:c64/chargen@906143-02#lower");
        assert_eq!(RomRef::parse("rom:c64/chargen#upper").unwrap().rev, None);
        for bad in ["c64/chargen#upper", "rom:c64#upper", "rom:c64/chargen", "rom:/x#y"] { assert!(RomRef::parse(bad).is_err(), "{bad}"); }
    }

    #[test]
    fn revisionskuerzel() { assert_eq!(revision_of("c64.chargen.901225-01"), "901225-01"); assert_eq!(revision_of("mega65.rom.open-roms"), "open-roms"); }
}
