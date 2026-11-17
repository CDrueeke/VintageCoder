// SPDX-License-Identifier: GPL-3.0-or-later
//! Zwischenkopie unbenannter Reiter: Damit ein Absturz oder ein Stromausfall keine ungespeicherten Notizen kostet, schreibt die IDE
//! den Inhalt unbenannter Reiter in `scratch.json` im Konfigurationsordner. Die Datei liegt **nur auf diesem Rechner**, nie im Projekt, ist
//! **nicht verschlüsselt** (der Hinweis steht im Dialog und in den Einstellungen), verfällt nach 14 Tagen und lässt sich abschalten.
//! Dieses Modul kennt keine Oberfläche: Lesen, Schreiben, Verfall und die Einstellung sind getestet; `app.rs` ruft es auf.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

/// Wie lange eine Zwischenkopie aufgehoben wird
pub const MAX_AGE_DAYS: u64 = 14;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Kennung des Reiters („Unbenannt 1“, „Notiz 2“)
    pub name: String,
    pub text: String,
    /// Reiter war eine Notiz (wird als `.md` gespeichert)
    #[serde(default)]
    pub note: bool,
    /// Zeitpunkt der letzten Änderung (Sekunden seit 1970)
    pub saved_at: u64,
}

#[derive(Default, Serialize, Deserialize)]
struct File { entries: Vec<Entry> }

pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("scratch.json")) }
fn pref_file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("scratch_pref.json")) }

pub fn now_secs() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

/// Einträge lesen; ältere als `MAX_AGE_DAYS` und leere entfallen
pub fn load_from(path: &Path, now: u64) -> Vec<Entry> {
    let Ok(t) = std::fs::read_to_string(path) else { return vec![] };
    let f: File = serde_json::from_str(&t).unwrap_or_default();
    f.entries.into_iter().filter(|e| !e.text.trim().is_empty() && now.saturating_sub(e.saved_at) <= MAX_AGE_DAYS * 86_400).collect()
}

/// Einträge schreiben (über eine Zwischendatei, damit ein Absturz beim Schreiben die alte Fassung nicht zerstört); keine Einträge: Datei entfernen
pub fn save_to(path: &Path, entries: &[Entry]) -> bool {
    if entries.is_empty() { let _ = std::fs::remove_file(path); return true; }
    if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
    let tmp = path.with_extension("json.tmp");
    let Ok(json) = serde_json::to_string_pretty(&File { entries: entries.to_vec() }) else { return false };
    std::fs::write(&tmp, json).is_ok() && std::fs::rename(&tmp, path).is_ok()
}

/// Einstellung „Zwischenkopie unbenannter Reiter“: 0 = noch nicht gelesen, 1 = an, 2 = aus
static ENABLED: AtomicU8 = AtomicU8::new(0);

pub fn enabled() -> bool {
    if ENABLED.load(Ordering::Relaxed) == 0 {
        let on = pref_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).and_then(|v| v["enabled"].as_bool()).unwrap_or(true);
        ENABLED.store(if on { 1 } else { 2 }, Ordering::Relaxed);
    }
    ENABLED.load(Ordering::Relaxed) == 1
}

/// Einstellung ändern und speichern; beim Abschalten verschwindet die vorhandene Zwischenkopie sofort
pub fn set_enabled(on: bool) {
    ENABLED.store(if on { 1 } else { 2 }, Ordering::Relaxed);
    if let Some(f) = pref_file() { if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); } let _ = std::fs::write(f, format!("{}\n", serde_json::json!({ "enabled": on }))); }
    if !on { if let Some(f) = file() { let _ = std::fs::remove_file(f); } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_scratchsave_{}_{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); d }
    fn e(name: &str, text: &str, at: u64) -> Entry { Entry { name: name.into(), text: text.into(), note: false, saved_at: at } }

    #[test]
    fn schreiben_lesen_und_leer_loescht_die_datei() {
        let d = tmp("rt"); let f = d.join("scratch.json");
        let v = vec![e("Unbenannt 1", "hallo", 1_000), Entry { note: true, ..e("Notiz 1", "# Idee\ntext", 1_005) }];
        assert!(save_to(&f, &v));
        assert_eq!(load_from(&f, 2_000), v);
        assert!(save_to(&f, &[]));
        assert!(!f.exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn alte_und_leere_eintraege_verfallen() {
        let d = tmp("age"); let f = d.join("scratch.json");
        let now = 100 * 86_400;
        assert!(save_to(&f, &[e("a", "neu", now - 3 * 86_400), e("b", "alt", now - 15 * 86_400), e("c", "  \n", now)]));
        let got = load_from(&f, now);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name, "a");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn kaputte_datei_ergibt_keine_eintraege() {
        let d = tmp("bad"); std::fs::create_dir_all(&d).unwrap(); let f = d.join("scratch.json");
        std::fs::write(&f, "{ nicht json").unwrap();
        assert!(load_from(&f, 1).is_empty());
        assert!(load_from(&d.join("fehlt.json"), 1).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }
}
