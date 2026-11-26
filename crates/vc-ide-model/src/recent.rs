// SPDX-License-Identifier: GPL-3.0-or-later
//! Zuletzt geöffnete Projekte (`recent.json` im Konfigurationsordner neben `view.json`) samt Kurzinfo für die Willkommensseite.
//! Die Info (Zielsystem, Sprache bzw. BASIC-Dialekt, Standard-Emulator) wird aus der Projektdatei gelesen, nie erfunden.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const MAX_UNPINNED: usize = 12;

/// Kurzname eines Zielsystems für Marken
pub fn short_name(system: &str) -> String {
    match system { "c64" => "C64".into(), "c128" => "C128".into(), "plus4" => vc_i18n::t!("recent-plus-4"), "c16" => "C16".into(), "vic20" => "VC-20".into(), "atari8" => vc_i18n::t!("recent-atari-8-bit"), "mega65" => "MEGA65".into(), _ => "?".into() }
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

thread_local! { static UMLEITUNG: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) }; }
/// Schreibt/liest in diesem Thread eine andere Datei (Tests der Oberfläche: nie die echte Konfiguration anfassen)
pub fn umleiten(p: Option<PathBuf>) { UMLEITUNG.with(|u| *u.borrow_mut() = p) }
pub fn umgeleitet() -> bool { UMLEITUNG.with(|u| u.borrow().is_some()) }

pub fn file() -> Option<PathBuf> {
    if let Some(p) = UMLEITUNG.with(|u| u.borrow().clone()) { return Some(p); }
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
    // dasselbe Projekt nur einmal (Schreibweise des Pfads kann abweichen); der erste Eintrag der Sortierung bleibt
    let mut gesehen = std::collections::HashSet::new(); v.retain(|r| gesehen.insert(schluessel(&r.path)));
    v
}

fn sort(v: &mut [Recent]) { v.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.last.cmp(&a.last))); }

pub fn save(v: &[Recent]) {
    if std::env::var("VC_SHOT").is_ok() { return; }   // Selbsttests ändern keine Einstellungen
    let Some(f) = file() else { return };
    if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
    let a: Vec<Value> = v.iter().map(|r| json!({ "path": r.path.to_string_lossy(), "name": r.name, "pinned": r.pinned, "last": r.last })).collect();
    let _ = vc_core::atomar::schreiben(f, serde_json::to_string_pretty(&a).unwrap_or_default() + "\n");
}

/// Projekt als zuletzt geöffnet eintragen (vorn), ältere nicht angeheftete Einträge fallen heraus
/// Vergleichsschlüssel eines Pfads: `/` statt `\`, ohne Schlusstrenner, unter Windows ohne Beachtung der Groß-/Kleinschreibung (C:/tmp/x und c:\\TMP\\X sind dasselbe Projekt)
pub fn schluessel(p: &Path) -> String {
    // Gibt es den Pfad, löst canonicalize Symlinks, `//` und `.`/`..` auf (dasselbe Projekt über andere Wege = ein Eintrag); sonst nur die Schreibweise vereinheitlichen
    let echt = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let mut s = echt.to_string_lossy().replace('\\', "/");
    if let Some(r) = s.strip_prefix("//?/") { s = r.to_string(); }   // Windows: \\?\-Vorsatz von canonicalize
    let mut o = String::with_capacity(s.len()); let mut vor = ' ';
    for (i, c) in s.chars().enumerate() { if c == '/' && vor == '/' && !(cfg!(windows) && i == 1) { continue } o.push(c); vor = c; }   // `//` zusammenfassen (UNC-Anfang unter Windows bleibt)
    let o = o.trim_end_matches('/').to_string();
    if cfg!(windows) { o.to_lowercase() } else { o }
}
pub fn touch(v: &mut Vec<Recent>, root: &Path, name: &str, data: Option<&Path>) {
    let k = schluessel(root);
    let pinned = v.iter().find(|r| schluessel(&r.path) == k).map_or(false, |r| r.pinned);
    v.retain(|r| schluessel(&r.path) != k);
    // vorn einfügen: bei gleicher Sekunde (stabile Sortierung) bleibt das zuletzt geöffnete Projekt vorn
    v.insert(0, Recent { path: root.to_path_buf(), name: name.to_string(), pinned, last: now(), missing: false, info: read_info(root, data) });
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
    match d { 0..=59 => vc_i18n::t!("recent-gerade-eben").into(), 60..=3599 => vc_i18n::t!("recent-vor-p1-min", p1 = d / 60), 3600..=86399 => vc_i18n::t!("recent-vor-p1-std", p1 = d / 3600), 86400..=172799 => vc_i18n::t!("recent-gestern"), _ => vc_i18n::t!("recent-vor-p1-tagen", p1 = d / 86400) }
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

#[cfg(test)]
mod schluessel_tests {
    use super::*;
    #[test] fn pfadschluessel_vereinheitlicht_trenner_und_schluss() {
        assert_eq!(schluessel(Path::new("a/b/")), schluessel(Path::new("a/b")));
        assert_eq!(schluessel(Path::new("a\\b")), schluessel(Path::new("a/b")));
        assert_ne!(schluessel(Path::new("a/b")), schluessel(Path::new("a/c")));
    }
    #[test] fn doppelte_trenner_und_symlinks_ergeben_denselben_schluessel() {
        let d = std::env::temp_dir().join(format!("vc_recent_schl_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("jumprun")).unwrap();
        let a = d.join("jumprun"); let b = PathBuf::from(format!("{}//jumprun/", d.display())); let c = d.join("jumprun").join("..").join("jumprun");
        assert_eq!(schluessel(&a), schluessel(&b), "// und Schluss-/ gelten nicht"); assert_eq!(schluessel(&a), schluessel(&c), ".. wird aufgelöst");
        #[cfg(unix)] { let l = d.join("link"); std::os::unix::fs::symlink(&a, &l).unwrap(); assert_eq!(schluessel(&a), schluessel(&l), "Symlink = dasselbe Projekt"); }
        assert_eq!(schluessel(Path::new("/gibt//es/nicht/")), schluessel(Path::new("/gibt/es/nicht")), "auch bei nicht vorhandenem Pfad");
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn zuletzt_geoeffnetes_steht_auch_bei_gleicher_sekunde_vorn() {
        std::env::set_var("VC_SHOT", "1");   // `touch` speichert sonst in die echte Liste des Benutzers
        let d = std::env::temp_dir().join(format!("vc-recent-sek-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); let (a, b) = (d.join("a"), d.join("b")); std::fs::create_dir_all(&a).unwrap(); std::fs::create_dir_all(&b).unwrap();
        let mut v: Vec<Recent> = vec![]; touch(&mut v, &a, "a", None); touch(&mut v, &b, "b", None);
        assert_eq!(v[0].name, "b", "gleiche Sekunde: das zweite Projekt steht vorn"); touch(&mut v, &a, "a", None); assert_eq!(v[0].name, "a");
        let _ = std::fs::remove_dir_all(&d);
    }
}
