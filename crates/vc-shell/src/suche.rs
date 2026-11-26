// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektsuche im Hintergrund (E1.2): Textsuche in den Textdateien des Projekts über `vc-ide-model::projsearch`, abbrechbar.
//! Die Oberfläche startet mit `search_start` (Antwort sofort: Nummer der Suche), Treffer kommen gebündelt als Ereignis `search:hits`,
//! das Ende als `search:done`; `search_cancel` bricht ab. Ungespeicherte Texte offener Reiter gehen der Datei auf der Platte vor.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vc_ide_model::projsearch::{self, Hit, Params, MAX_HITS};

#[derive(Default)]
pub struct Suche { laufend: Mutex<Option<(u64, Arc<AtomicBool>)>>, zaehler: AtomicU64 }

fn u16_bis(text: &str, byte: usize) -> usize { text[..byte.min(text.len())].encode_utf16().count() }

/// Ein Treffer für die Oberfläche: `line` ab 1, `from`/`to` in UTF-16-Einheiten des gekürzten Textes
pub fn treffer_json(h: &Hit) -> Value { json!({ "rel": h.rel, "line": h.line, "text": h.text, "from": u16_bis(&h.text, h.start), "to": u16_bis(&h.text, h.end) }) }

impl Suche {
    pub fn abbrechen(&self) { if let Some((_, c)) = self.laufend.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() { c.store(true, Ordering::Relaxed); } }

    /// Neue Suche starten (eine laufende wird abgebrochen). `senden(name, nutzlast)` bekommt die Ereignisse; Rückgabe: Nummer der Suche.
    pub fn starten(&self, p: Params, senden: impl Fn(&'static str, Value) + Send + 'static) -> u64 {
        self.abbrechen();
        let id = self.zaehler.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel = Arc::new(AtomicBool::new(false));
        *self.laufend.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((id, cancel.clone()));
        std::thread::spawn(move || {
            let mut stapel: Vec<Value> = vec![]; let mut zuletzt = Instant::now();
            let sum = projsearch::run(&p, &cancel, &mut |h| {
                stapel.push(treffer_json(&h));
                if stapel.len() >= 50 || zuletzt.elapsed() > Duration::from_millis(80) { senden("search:hits", json!({ "id": id, "hits": std::mem::take(&mut stapel) })); zuletzt = Instant::now(); }
            });
            if !stapel.is_empty() { senden("search:hits", json!({ "id": id, "hits": stapel })); }
            senden("search:done", json!({ "id": id, "files": sum.files, "truncated": sum.truncated, "cancelled": sum.cancelled, "limit": MAX_HITS }));
        });
        id
    }
}

/// Suchparameter aus Projektordner, Dateiliste, ungespeicherten Texten und Suchtext
pub fn parameter(root: PathBuf, files: Vec<String>, overrides: HashMap<String, String>, text: String, case: bool) -> Params {
    Params { root, files, overrides, text, case, value: None, limit: MAX_HITS }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    fn tmp(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_suche_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test] fn findet_treffer_mit_utf16_spalten_und_nimmt_ungespeicherten_text() {
        let r = tmp("a"); std::fs::write(r.join("a.asm"), "lda #1\n; Größe lda\n").unwrap(); std::fs::write(r.join("b.asm"), "nichts\n").unwrap();
        let s = Suche::default(); let (tx, rx) = channel();
        let mut ov = HashMap::new(); ov.insert("b.asm".to_string(), "x\n😀 lda y\n".to_string());
        let id = s.starten(parameter(r.clone(), vec!["a.asm".into(), "b.asm".into()], ov, "lda".into(), false), move |n, v| { let _ = tx.send((n, v)); });
        let mut hits = vec![]; let done;
        loop { let (n, v) = rx.recv_timeout(Duration::from_secs(5)).expect("Ereignis"); if n == "search:hits" { hits.extend(v["hits"].as_array().unwrap().clone()); } else { done = v; break; } }
        assert_eq!(done["id"], id); assert_eq!(done["cancelled"], false); assert_eq!(hits.len(), 3, "{hits:?}");
        assert_eq!(hits[1]["from"], 8, "Umlaut zählt eine Einheit"); // „; Größe lda“: lda ab Zeichen 8
        let b = hits.iter().find(|h| h["rel"] == "b.asm").unwrap(); assert_eq!(b["from"], 3, "Emoji zählt zwei Einheiten, Text aus dem Reiter"); assert_eq!(b["line"], 2);
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn abbrechen_beendet_die_suche_mit_dem_kennzeichen() {
        let r = tmp("b"); let mut files = vec![]; for i in 0..400 { let n = format!("f{i}.asm"); std::fs::write(r.join(&n), "x\n".repeat(3000)).unwrap(); files.push(n); }
        let s = Suche::default(); let (tx, rx) = channel();
        s.starten(parameter(r.clone(), files, HashMap::new(), "zzz".into(), true), move |n, v| { let _ = tx.send((n, v)); });
        s.abbrechen();
        let done = loop { let (n, v) = rx.recv_timeout(Duration::from_secs(10)).expect("Ende"); if n == "search:done" { break v; } };
        assert_eq!(done["cancelled"], true);
        let _ = std::fs::remove_dir_all(&r);
    }
}
