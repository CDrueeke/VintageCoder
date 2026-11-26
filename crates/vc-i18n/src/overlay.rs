// SPDX-License-Identifier: GPL-3.0-or-later
//! Englische Überlagerungen der Datendateien (`data/i18n/en/<bereich>/<datei>.json`, Etappe I3).
//!
//! Die Quelldateien unter `data/` bleiben deutsch und unverändert. Zu jeder Datei mit Anzeigetexten gibt es eine Überlagerung mit Einträgen
//! `"/zeiger": {"de": Originaltext, "en": Übersetzung, "status": …}` (JSON-Zeiger in die Quelldatei). [`localize_json`] setzt beim Laden
//! den englischen Text an diese Stelle, aber nur, wenn dort noch der Originaltext steht (eine geänderte oder selbst bearbeitete Datei bleibt
//! also wie sie ist) und die Übersetzung nicht leer ist. In jeder anderen Sprache als Englisch bleibt alles deutsch.
use crate::{language, Lang};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

include!(concat!(env!("OUT_DIR"), "/data_overlays.rs"));

type Entries = std::sync::Arc<Vec<(String, String, String)>>; // (Zeiger, deutscher Text, englischer Text)

fn entries(rel: &str) -> Option<Entries> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<Entries>>>> = OnceLock::new();
    let mut g = CACHE.get_or_init(Default::default).lock().ok()?;
    g.entry(rel.to_string()).or_insert_with(|| {
        let (_, text) = OVERLAYS.iter().find(|(r, _)| *r == rel)?;
        let v: Value = serde_json::from_str(text).ok()?;
        let list = v["entries"].as_object()?.iter().filter_map(|(z, e)| {
            let en = e["en"].as_str().unwrap_or("");
            if en.is_empty() { return None; }
            Some((z.clone(), e["de"].as_str().unwrap_or("").to_string(), en.to_string()))
        }).collect();
        Some(std::sync::Arc::new(list))
    }).clone()
}

/// Übersetzt die Textfelder einer geladenen Datendatei in die aktuelle Sprache. `rel` ist der Pfad der Quelldatei unter `data/`
/// (z. B. `"targets/c64.target.json"`).
pub fn localize_json(v: &mut Value, rel: &str) { localize_json_in(language(), v, rel) }

pub fn localize_json_in(lang: Lang, v: &mut Value, rel: &str) {
    if lang != Lang::En { return; }
    let Some(list) = entries(rel) else { return };
    for (z, de, en) in list.iter() {
        if let Some(slot) = v.pointer_mut(z) {
            if slot.as_str() == Some(de.as_str()) { *slot = Value::String(en.clone()); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_wird_nur_ersetzt_wenn_das_original_noch_dasselbe_ist() {
        let list: Entries = std::sync::Arc::new(vec![("/a".into(), "eins".into(), "one".into()), ("/b/0".into(), "zwei".into(), "two".into())]);
        let mut v: Value = serde_json::json!({"a": "eins", "b": ["geaendert"]});
        for (z, de, en) in list.iter() { if let Some(s) = v.pointer_mut(z) { if s.as_str() == Some(de) { *s = Value::String(en.clone()); } } }
        assert_eq!(v["a"], "one");
        assert_eq!(v["b"][0], "geaendert");
    }

    #[test]
    fn deutsch_bleibt_unveraendert() {
        let mut v = serde_json::json!({"a": "eins"});
        localize_json_in(Lang::De, &mut v, "gibt-es-nicht.json");
        localize_json_in(Lang::En, &mut v, "gibt-es-nicht.json");
        assert_eq!(v["a"], "eins");
    }

    /// Jede eingebettete Überlagerung passt zur Quelldatei: Zeiger vorhanden, deutscher Text gleich; nach dem Laden steht dort der englische Text.
    #[test]
    fn alle_ueberlagerungen_passen_zu_ihren_quelldateien() {
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        assert!(!OVERLAYS.is_empty());
        let mut n = 0;
        for (rel, text) in OVERLAYS {
            if rel.starts_with("sys/") { continue; } // Zeilenüberlagerungen der Systembibliothek: geprüft in vc-ide (sysmap) und tools/check_i18n_data.py
            let ov: Value = serde_json::from_str(text).unwrap_or_else(|e| panic!("{rel}: {e}"));
            let src = std::fs::read_to_string(data.join(rel)).unwrap_or_else(|e| panic!("{rel}: Quelle fehlt: {e}"));
            let orig: Value = serde_json::from_str(&src).unwrap();
            let mut loaded = orig.clone();
            localize_json_in(Lang::En, &mut loaded, rel);
            let mut de = orig.clone();
            localize_json_in(Lang::De, &mut de, rel);
            assert_eq!(de, orig, "{rel}: Deutsch bleibt unverändert");
            for (z, e) in ov["entries"].as_object().unwrap() {
                assert_eq!(orig.pointer(z).and_then(Value::as_str), e["de"].as_str(), "{rel}{z}: deutscher Text weicht von der Quelle ab");
                let en = e["en"].as_str().unwrap();
                if !en.is_empty() { assert_eq!(loaded.pointer(z).and_then(Value::as_str), Some(en), "{rel}{z}"); n += 1; }
            }
        }
        assert!(n > 1000, "{n} übersetzte Felder");
    }
}
