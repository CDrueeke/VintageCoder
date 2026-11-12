// SPDX-License-Identifier: GPL-3.0-or-later
//! Zuletzt geöffnete Projekte (`recent.json` im Konfigurationsordner neben `view.json`) samt Kurzinfo für die Willkommensseite.
//! Die Info (Zielsystem, Sprache bzw. BASIC-Dialekt, Standard-Emulator) wird aus der Projektdatei gelesen, nie erfunden.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const MAX_UNPINNED: usize = 12;

/// Kurzname eines Zielsystems für Marken
pub fn short_name(system: &str) -> &'static str {
    match system { "c64" => "C64", "c128" => "C128", "plus4" => "Plus/4", "c16" => "C16", "vic20" => "VC-20", "atari8" => "Atari 8 Bit", "mega65" => "MEGA65", _ => "?" }
}

pub fn dialect_label(id: &str) -> &'static str {
    match id { "cbm_v2" => "BASIC V2", "basic7" => "BASIC 7.0", "basic35" => "BASIC 3.5", "basic65" => "BASIC 65", "atari_basic" => "Atari BASIC", "turbo_basic_xl" => "Turbo BASIC XL", _ => "BASIC" }
}

/// BASIC-Dialekt, den ein Target von sich aus nutzt (`basic.dialect` in `data/targets/<id>.target.json`)
pub fn target_dialect(data: &Path, system: &str) -> Option<String> {
    let t = std::fs::read_to_string(data.join("targets").join(format!("{system}.target.json"))).ok()?;
    serde_json::from_str::<Value>(&t).ok()?["basic"]["dialect"].as_str().map(String::from)
}

#[derive(Clone, Default)]
pub struct Info {
    /// (Zielsystem, Videonorm)
    pub systems: Vec<(String, String)>,
    /// Sprachmarken: „vcasm“ bzw. BASIC-Dialekt
    pub langs: Vec<String>,
    /// Standard-Emulator des ersten Zielsystems laut Einstellungen
    pub emulator: Option<String>,
    /// Das Projekt will kein Projektbild (Einstellung in der Projektdatei)
    pub no_preview: bool,
}

#[derive(Clone)]
pub struct Recent { pub path: PathBuf, pub name: String, pub pinned: bool, pub last: u64, pub missing: bool, pub info: Info }

pub fn file() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VC_RECENT") { return Some(PathBuf::from(p)); }
    Some(vc_core::userdirs::config_dir()?.join("recent.json"))
}

pub fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

/// Kurzinfo aus der Projektdatei im Ordner `root`
pub fn read_info(root: &Path, data: Option<&Path>) -> Info {
    let Some(f) = vc_ide::projfile::find(root) else { return Info::default() };
    let Ok(pf) = vc_ide::projfile::ProjectFile::load(&f) else { return Info::default() };
    let mut info = Info { no_preview: pf.no_preview, ..Default::default() };
    for t in &pf.targets {
        let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
        if !info.systems.iter().any(|(s, _)| *s == sys) { info.systems.push((sys, t.variant.clone())); }
    }
    for s in &pf.sources {
        let label = if s.kind == "basic" {
            let sys = s.targets.first().cloned().or_else(|| info.systems.first().map(|x| x.0.clone())).unwrap_or_default();
            let id = if s.dialect.is_empty() { data.and_then(|d| target_dialect(d, &sys)).unwrap_or_default() } else { s.dialect.clone() };
            dialect_label(&id).to_string()
        } else { "vcasm".to_string() };
        if !info.langs.contains(&label) { info.langs.push(label); }
    }
    if let Some((sys, _)) = info.systems.first() {
        let e = vc_ide::settings::effective().emulator_for(sys);
        info.emulator = Some(vc_ide::targets::kind_label(&e.kind).to_string());
    }
    info
}

pub fn load(data: Option<&Path>) -> Vec<Recent> {
    let Some(t) = file().and_then(|f| std::fs::read_to_string(f).ok()) else { return vec![] };
    let Ok(Value::Array(a)) = serde_json::from_str::<Value>(&t) else { return vec![] };
    let mut v: Vec<Recent> = a.iter().filter_map(|e| {
        let path = PathBuf::from(e["path"].as_str()?);
        let missing = !path.exists();
        let info = if missing { Info::default() } else { read_info(&path, data) };
        Some(Recent { name: e["name"].as_str().map(String::from).unwrap_or_else(|| path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()), pinned: e["pinned"].as_bool().unwrap_or(false), last: e["last"].as_u64().unwrap_or(0), path, missing, info })
    }).collect();
    sort(&mut v);
    v
}

fn sort(v: &mut [Recent]) { v.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.last.cmp(&a.last))); }

pub fn save(v: &[Recent]) {
    if std::env::var("VC_SHOT").is_ok() { return; }   // Selbsttests ändern keine Einstellungen
    let Some(f) = file() else { return };
    if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
    let a: Vec<Value> = v.iter().map(|r| json!({ "path": r.path.to_string_lossy(), "name": r.name, "pinned": r.pinned, "last": r.last })).collect();
    let _ = std::fs::write(f, serde_json::to_string_pretty(&a).unwrap_or_default() + "\n");
}

/// Projekt als zuletzt geöffnet eintragen (vorn), ältere nicht angeheftete Einträge fallen heraus
pub fn touch(v: &mut Vec<Recent>, root: &Path, name: &str, data: Option<&Path>) {
    let pinned = v.iter().find(|r| r.path == root).map_or(false, |r| r.pinned);
    v.retain(|r| r.path != root);
    v.push(Recent { path: root.to_path_buf(), name: name.to_string(), pinned, last: now(), missing: false, info: read_info(root, data) });
    sort(v);
    let mut n = 0;
    v.retain(|r| { if r.pinned { true } else { n += 1; n <= MAX_UNPINNED } });
    save(v);
}

pub fn remove(v: &mut Vec<Recent>, i: usize) { if i < v.len() { v.remove(i); save(v); } }

pub fn toggle_pin(v: &mut Vec<Recent>, i: usize) { if let Some(r) = v.get_mut(i) { r.pinned = !r.pinned; sort(v); save(v); } }

/// „vor 25 Min.“, „gestern“ …
pub fn ago(last: u64, now: u64) -> String {
    if last == 0 { return String::new(); }
    let d = now.saturating_sub(last);
    match d { 0..=59 => "gerade eben".into(), 60..=3599 => format!("vor {} Min.", d / 60), 3600..=86399 => format!("vor {} Std.", d / 3600), 86400..=172799 => "gestern".into(), _ => format!("vor {} Tagen", d / 86400) }
}

/// Pfad mit `~` für den Home-Ordner
pub fn pretty(p: &Path) -> String {
    let s = p.display().to_string();
    match vc_core::userdirs::home_dir() { Some(h) => { let hs = h.display().to_string(); if !hs.is_empty() && s.starts_with(&hs) { format!("~{}", &s[hs.len()..]) } else { s } } None => s }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ago_texts() {
        assert_eq!(ago(1000, 1030), "gerade eben"); assert_eq!(ago(1000, 1000 + 25 * 60), "vor 25 Min."); assert_eq!(ago(0, 5), "");
        assert_eq!(ago(1000, 1000 + 3 * 3600), "vor 3 Std."); assert_eq!(ago(1000, 1000 + 86400), "gestern"); assert_eq!(ago(1000, 1000 + 3 * 86400), "vor 3 Tagen");
    }
    #[test]
    fn touch_orders_by_pin_then_time_and_caps_unpinned() {
        let mut v: Vec<Recent> = vec![];
        for i in 0..15 { v.push(Recent { path: PathBuf::from(format!("/p/{i}")), name: format!("p{i}"), pinned: i == 0, last: i as u64 + 1, missing: true, info: Info::default() }); }
        sort(&mut v);
        assert_eq!(v[0].name, "p0", "angeheftet steht vorn");
        assert_eq!(v[1].name, "p14");
        let mut n = 0; v.retain(|r| { if r.pinned { true } else { n += 1; n <= MAX_UNPINNED } });
        assert_eq!(v.len(), 1 + MAX_UNPINNED);
    }
}
