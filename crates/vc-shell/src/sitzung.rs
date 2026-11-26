// SPDX-License-Identifier: GPL-3.0-or-later
//! Sitzung (zuletzt offene Dateien je Projekt), Schalter „Sitzung wiederherstellen“ im Projekt und Zwischenkopie unbenannter Reiter.
//! Die Formate sind die der Modellschicht (`session.json`, `scratch.json`), also dieselben Dateien wie in der alten Oberfläche.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide_model::{scratchsave, session};

fn pfade(v: &Value) -> Vec<String> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }

/// Schalter „Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen“ aus der `.vcprj` (Standard aus)
pub fn wiederherstellen_an(root: &Path) -> bool { vc_ide::projfile::find(root).and_then(|f| vc_ide::projfile::ProjectFile::load(&f).ok()).map_or(false, |p| p.restore_tabs) }

pub fn wiederherstellen_setzen(root: &Path, an: bool) -> Result<(), String> {
    let f = vc_ide::projfile::find(root).ok_or_else(|| root.display().to_string())?;
    let mut p = vc_ide::projfile::ProjectFile::load(&f)?; if p.restore_tabs == an { return Ok(()); }
    p.restore_tabs = an; p.save(&f)
}

/// Sitzung zum Wiederöffnen: die Reiter in ihrer alten Reihenfolge (nur vorhandene, ohne Doppelte); die aktive Datei steht gesondert in `active`
/// und wird von der Oberfläche zuletzt aktiviert. `enabled` sagt, ob das Projekt es wünscht.
pub fn lesen(root: &Path, sitzungsdatei: &Path) -> Value {
    let an = wiederherstellen_an(root);
    let s = session::load_from(sitzungsdatei, root).unwrap_or_default();
    let mut offen: Vec<String> = vec![]; for r in &s.open { if root.join(r).is_file() && !offen.contains(r) { offen.push(r.clone()); } }
    let aktiv = s.active.clone().filter(|a| offen.contains(a));
    let marks: serde_json::Map<String, Value> = s.marks.iter().filter(|(r, m)| root.join(r).is_file() && (!m.breakpoints.is_empty() || !m.bookmarks.is_empty()))
        .map(|(r, m)| (r.clone(), serde_json::to_value(m).unwrap_or(Value::Null))).collect();
    json!({ "enabled": an, "open": offen, "active": aktiv, "marks": marks })
}

pub fn speichern(root: &Path, sitzungsdatei: &Path, offen: &Value, aktiv: &Value, marken: &Value) -> bool {
    let mut s = session::load_from(sitzungsdatei, root).unwrap_or_default();
    s.open = pfade(offen); s.active = aktiv.as_str().map(String::from);
    if let Some(o) = marken.as_object() {   // nur wenn mitgeschickt: Haltepunkte und Lesezeichen je Datei (leere entfallen)
        s.marks = o.iter().filter_map(|(r, v)| serde_json::from_value::<session::FileMarks>(v.clone()).ok().map(|m| (r.clone(), m)))
            .filter(|(_, m)| !m.breakpoints.is_empty() || !m.bookmarks.is_empty()).collect();
    }
    session::save_to(sitzungsdatei, root, &s)
}

pub fn zwischenkopie_lesen(datei: &Path, jetzt: u64) -> Value {
    json!(scratchsave::load_from(datei, jetzt).into_iter().map(|e| json!({ "name": e.name, "text": e.text, "note": e.note, "savedAt": e.saved_at })).collect::<Vec<_>>())
}
pub fn zwischenkopie_schreiben(datei: &Path, eintraege: &Value, jetzt: u64) -> bool {
    let v: Vec<scratchsave::Entry> = eintraege.as_array().map(|a| a.iter().filter_map(|e| Some(scratchsave::Entry {
        name: e["name"].as_str()?.to_string(), text: e["text"].as_str()?.to_string(), note: e["note"].as_bool().unwrap_or(false), saved_at: jetzt })).filter(|e| !e.text.trim().is_empty()).collect()).unwrap_or_default();
    scratchsave::save_to(datei, &v)
}

pub fn sitzungsdatei() -> Option<PathBuf> { session::file() }
pub fn zwischenkopiedatei() -> Option<PathBuf> { scratchsave::file() }

#[cfg(test)]
mod tests {
    use super::*;
    fn dir(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_sitzung_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test] fn sitzung_wird_je_projekt_gespeichert_fehlende_dateien_entfallen_reihenfolge_bleibt() {
        let d = dir("s"); let root = d.join("p"); std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("a.asm"), "").unwrap(); std::fs::write(root.join("b.asm"), "").unwrap();
        let f = d.join("session.json");
        assert!(speichern(&root, &f, &json!(["a.asm", "weg.asm", "b.asm"]), &json!("a.asm"), &json!({ "a.asm": { "breakpoints": [{ "line": 3, "text": "lda #1" }], "bookmarks": [] }, "weg.asm": { "breakpoints": [{ "line": 1, "text": "x" }], "bookmarks": [] } })));
        let v = lesen(&root, &f);
        assert_eq!(v["open"], json!(["a.asm", "b.asm"]), "alte Reihenfolge, Fehlendes entfällt"); assert_eq!(v["active"], json!("a.asm"), "aktive Datei gesondert");
        assert_eq!(v["active"], json!("a.asm")); assert_eq!(v["enabled"], json!(false), "ohne .vcprj aus");
        assert_eq!(v["marks"]["a.asm"]["breakpoints"][0]["line"], json!(3), "Haltepunkte bleiben je Datei erhalten"); assert_eq!(v["marks"]["a.asm"]["breakpoints"][0]["text"], json!("lda #1"));
        assert!(v["marks"].get("weg.asm").is_none(), "Marken fehlender Dateien entfallen");
        assert!(speichern(&root, &f, &json!(["a.asm"]), &json!("a.asm"), &Value::Null)); assert_eq!(lesen(&root, &f)["marks"]["a.asm"]["breakpoints"][0]["line"], json!(3), "ohne mitgeschickte Marken bleiben die alten");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn schalter_im_projekt_wird_in_der_vcprj_gespeichert_und_gelesen() {
        let d = dir("w"); std::fs::write(d.join("t.vcprj"), r#"{"project":{"name":"T"}}"#).unwrap();
        assert!(!wiederherstellen_an(&d));
        wiederherstellen_setzen(&d, true).unwrap(); assert!(wiederherstellen_an(&d));
        wiederherstellen_setzen(&d, false).unwrap(); assert!(!wiederherstellen_an(&d));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn zwischenkopie_wird_geschrieben_gelesen_und_leer_entfernt() {
        let d = dir("z"); let f = d.join("scratch.json"); let jetzt = scratchsave::now_secs();
        assert!(zwischenkopie_schreiben(&f, &json!([{ "name": "Unbenannt 1", "text": "lda #1", "note": false }, { "name": "Notiz 1", "text": "  ", "note": true }]), jetzt));
        let v = zwischenkopie_lesen(&f, jetzt); assert_eq!(v.as_array().unwrap().len(), 1, "leere Reiter werden nicht gesichert");
        assert_eq!(v[0]["text"], json!("lda #1"));
        assert!(zwischenkopie_schreiben(&f, &json!([]), jetzt)); assert!(!f.exists(), "ohne Einträge verschwindet die Datei");
        assert_eq!(zwischenkopie_lesen(&f, jetzt), json!([]));
        let _ = std::fs::remove_dir_all(&d);
    }
}
