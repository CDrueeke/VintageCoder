// SPDX-License-Identifier: GPL-3.0-or-later
//! Bereich „Projekt“ (P2): Projektdatei lesen und die Felder der Seiten Übersicht, Allgemein und Ziele ändern. Alle Regeln (Prüfung, Speichern) kommen aus `vc_ide::projfile`;
//! hier liegt nur das Anzeige-DTO und die gebündelte Änderung. Gespeichert wird sofort (kein Entwurfszustand), eine ungültige Änderung bleibt ungespeichert und wird gemeldet.
use serde_json::{json, Value};
use std::path::Path;
use vc_ide::projfile::{find, ProjectFile};

fn laden(root: &Path) -> Result<(std::path::PathBuf, ProjectFile), String> {
    let f = find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let pf = ProjectFile::load(&f)?; Ok((f, pf))
}
/// Normen eines Zielsystems (`pal`, `ntsc` …) aus den Systemdaten; leer, wenn das System keine Varianten kennt
fn varianten(data: &Path, system: &str) -> Vec<String> {
    let f = data.join("targets").join(format!("{system}.target.json"));
    let Ok(t) = std::fs::read_to_string(f) else { return vec![] };
    let Ok(v) = serde_json::from_str::<Value>(&t) else { return vec![] };
    v["variants"].as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default()
}

/// Zustand des Projekts für die Seiten: Angaben, Ziele mit wählbaren Varianten, Quellen- und Diskettenstand, Probleme
pub fn info(root: &Path, data: &Path) -> Result<Value, String> {
    let (file, pf) = laden(root)?;
    let (fehler, warn) = pf.validate_getrennt(root);
    let quellen = pf.source_files();
    let fehlende: Vec<&String> = quellen.iter().map(|q| &q.0).filter(|f| !root.join(f).is_file()).collect();
    let main = pf.main_file();
    let ziele: Vec<Value> = pf.targets.iter().map(|t| { let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() }; json!({ "id": t.id, "system": sys, "variant": t.variant, "variants": varianten(data, &sys), "emulator": t.emulator, "startOn": t.start_on, "illegals": t.illegals, "defaultEmulator": t.default_emulator }) }).collect();
    let disk = pf.disk.as_ref().map(|d| json!({ "format": d.format, "name": d.name, "file": d.file, "files": d.files }));
    Ok(json!({
        "file": file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        "name": pf.project.name, "description": pf.project.description, "main": pf.main, "mainEffective": main,
        "sources": quellen.iter().map(|q| json!({ "file": q.0, "kind": q.1 })).collect::<Vec<_>>(),
        "groups": pf.sources.len(), "sourceGroups": pf.sources.iter().map(|g| json!({ "kind": g.kind, "targets": g.targets, "dialect": g.dialect,
            "files": g.files.iter().map(|f| json!({ "file": f, "missing": !root.join(f).is_file() })).collect::<Vec<_>>() })).collect::<Vec<_>>(), "missing": fehlende, "targets": ziele, "disk": disk, "turbo": pf.turbo.is_some(),
        "restoreTabs": pf.restore_tabs, "errors": fehler, "warnings": warn,
    }))
}

/// Felder ändern: `name`, `description`, `main` (null/leer = erste Quelle), `restoreTabs`, `variant` (`{ id, variant }`). Gespeichert wird nur eine Datei ohne Fehler; sonst bleibt die Datei unverändert.
/// „Projekt für anderes System kopieren“ (Entwurf): macht `system` zum Normalziel (erster Eintrag) der Projektdatei im Ordner `root`.
/// Ist es schon ein Ziel, rückt es nur nach vorn; sonst wird es vorn eingefügt, die übrigen Ziele bleiben. Quellen werden nicht umgesetzt.
pub fn normalziel_setzen(root: &Path, data: &Path, system: &str) -> Result<(), String> {
    if !vc_ide::targets::installed_systems(Some(data)).iter().any(|i| i.id == system) { return Err(vc_i18n::t!("projekt-ziel-unbekannt", id = system).to_string()) }
    let (file, mut pf) = laden(root)?;
    match pf.targets.iter().position(|t| t.id == system || t.system == system) {
        Some(i) => { let t = pf.targets.remove(i); pf.targets.insert(0, t); }
        None => pf.targets.insert(0, vc_ide::projfile::TargetDef { id: system.into(), system: system.into(), ..Default::default() }),
    }
    pf.save(&file)
}

pub fn setzen(root: &Path, data: &Path, a: &Value) -> Result<Value, String> {
    let (file, mut pf) = laden(root)?;
    if let Some(n) = a["name"].as_str() { pf.project.name = n.trim().to_string(); }
    if let Some(d) = a["description"].as_str() { pf.project.description = d.trim().to_string(); }
    if a.get("main").is_some() { pf.main = a["main"].as_str().map(str::trim).filter(|m| !m.is_empty()).map(String::from); }
    if let Some(b) = a["restoreTabs"].as_bool() { pf.restore_tabs = b; }
    if let Some(m) = a.get("move").filter(|m| m.is_object()) {
        let (f, dir) = (m["file"].as_str().unwrap_or(""), m["dir"].as_i64().unwrap_or(0).signum());
        let g = pf.sources.iter_mut().find(|g| g.files.iter().any(|x| x == f)).ok_or_else(|| vc_i18n::t!("projekt-quelle-unbekannt", f = f))?;
        let i = g.files.iter().position(|x| x == f).unwrap_or(0); let j = i as i64 + dir;
        if j >= 0 && (j as usize) < g.files.len() { g.files.swap(i, j as usize); }
    }
    if let Some(v) = a.get("variant").filter(|v| v.is_object()) {
        let id = v["id"].as_str().unwrap_or(""); let var = v["variant"].as_str().unwrap_or("");
        let t = pf.targets.iter_mut().find(|t| t.id == id).ok_or_else(|| vc_i18n::t!("projekt-ziel-unbekannt", id = id))?;
        let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
        let ok = varianten(data, &sys); if !var.is_empty() && !ok.iter().any(|x| x == var) { return Err(vc_i18n::t!("projekt-variante-unbekannt", v = var)) }
        t.variant = var.to_string();
    }
    if let Some(sys) = a["addTarget"].as_str().map(str::trim).filter(|x| !x.is_empty()) {
        if !vc_ide::targets::installed_systems(Some(data)).iter().any(|i| i.id == sys) { return Err(vc_i18n::t!("projekt-ziel-unbekannt", id = sys).to_string()) }
        if pf.targets.iter().any(|t| t.id == sys || t.system == sys) { return Err(vc_i18n::t!("projekt-ziel-schon-da", id = sys).to_string()) }
        pf.targets.push(vc_ide::projfile::TargetDef { id: sys.into(), system: sys.into(), ..Default::default() });
    }
    if let Some(id) = a["removeTarget"].as_str() {
        if pf.targets.len() <= 1 { return Err(vc_i18n::t!("projekt-letztes-ziel").to_string()) }
        if !pf.targets.iter().any(|t| t.id == id) { return Err(vc_i18n::t!("projekt-ziel-unbekannt", id = id).to_string()) }
        pf.targets.retain(|t| t.id != id); for g in &mut pf.sources { g.targets.retain(|x| x != id); }
    }
    if let Some(g) = a.get("groupAdd").and_then(|g| g.as_object()) {
        let kind = g.get("kind").and_then(|k| k.as_str()).unwrap_or("asm");
        if !matches!(kind, "asm" | "basic") { return Err(vc_i18n::t!("projekt-quellart-unbekannt", kind = kind).to_string()) }
        pf.sources.push(vc_ide::projfile::Source { kind: kind.into(), ..Default::default() });
    }
    if let Some(i) = a["groupRemove"].as_u64() {
        let i = i as usize; if i >= pf.sources.len() { return Err(vc_i18n::t!("projekt-gruppe-unbekannt").to_string()) }
        if pf.sources[i].files.iter().any(|f| pf.main.as_deref() == Some(f.as_str())) { pf.main = None; }
        pf.sources.remove(i);
    }
    if let Some(g) = a.get("groupSet").and_then(|g| g.as_object()) {
        let i = g.get("i").and_then(|x| x.as_u64()).unwrap_or(u64::MAX) as usize; let src = pf.sources.get_mut(i).ok_or_else(|| vc_i18n::t!("projekt-gruppe-unbekannt").to_string())?;
        if let Some(k) = g.get("kind").and_then(|x| x.as_str()) { if !matches!(k, "asm" | "basic") { return Err(vc_i18n::t!("projekt-quellart-unbekannt", kind = k).to_string()) } src.kind = k.into(); if k != "basic" { src.dialect.clear(); src.convert = None; } }
        if let Some(d) = g.get("dialect").and_then(|x| x.as_str()) { src.dialect = d.trim().to_string(); }
        if let Some(t) = g.get("targets").and_then(|x| x.as_array()) { src.targets = t.iter().filter_map(|x| x.as_str().map(String::from)).collect(); }
    }
    if let Some(f) = a.get("fileAdd").and_then(|f| f.as_object()) {
        let i = f.get("group").and_then(|x| x.as_u64()).unwrap_or(u64::MAX) as usize; let name = f.get("file").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
        let rel = vc_ide_model::fileops::safe_rel(&name).map_err(|e| e.to_string())?.replace('\\', "/");
        if !root.join(&rel).is_file() { return Err(vc_i18n::t!("projfile-f-datei-fehlt", f = &rel).to_string()) }
        if pf.source_files().iter().any(|q| q.0 == rel) { return Err(vc_i18n::t!("projekt-quelle-schon-da", f = &rel).to_string()) }
        pf.sources.get_mut(i).ok_or_else(|| vc_i18n::t!("projekt-gruppe-unbekannt").to_string())?.files.push(rel);
    }
    if let Some(f) = a["fileRemove"].as_str() {
        for g in &mut pf.sources { g.files.retain(|x| x != f); } if pf.main.as_deref() == Some(f) { pf.main = None; }
    }
    if let Some(t) = a.get("targetSet").and_then(|t| t.as_object()) {
        let id = t.get("id").and_then(|x| x.as_str()).unwrap_or(""); let td = pf.targets.iter_mut().find(|x| x.id == id).ok_or_else(|| vc_i18n::t!("projekt-ziel-unbekannt", id = id).to_string())?;
        if let Some(v) = t.get("illegals").and_then(|x| x.as_str()) { td.illegals = v.trim().to_string(); }
        if let Some(v) = t.get("startOn").and_then(|x| x.as_str()) { td.start_on = v.trim().to_string(); }
        if let Some(v) = t.get("defaultEmulator").and_then(|x| x.as_str()) { td.default_emulator = v.trim().to_string(); }
    }
    if let Some(d) = a.get("disk") {
        if d.is_null() { pf.disk = None; }
        else if let Some(o) = d.as_object() {
            let cur = pf.disk.take().unwrap_or(vc_ide::projfile::DiskSpec { format: "d64".into(), name: String::new(), file: String::new(), files: vec![] });
            let fmt = o.get("format").and_then(|x| x.as_str()).unwrap_or(&cur.format).to_string();
            if !matches!(fmt.as_str(), "d64" | "d71" | "d81" | "atr") { return Err(vc_i18n::t!("projekt-diskformat-unbekannt", f = &fmt).to_string()) }
            pf.disk = Some(vc_ide::projfile::DiskSpec { format: fmt, name: o.get("name").and_then(|x| x.as_str()).map(|x| x.trim().to_string()).unwrap_or(cur.name), file: cur.file,
                files: o.get("files").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or(cur.files) });
        }
    }
    let (fehler, _) = pf.validate_getrennt(root);
    if let Some(e) = fehler.first() { return Err(vc_i18n::t!("projekt-nicht-gespeichert", grund = e)) }
    pf.save(&file)?;
    info(root, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str) -> std::path::PathBuf {
        let w = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let d = std::env::temp_dir().join(format!("vc_projekt_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d);
        fn kopieren(a: &Path, b: &Path) { std::fs::create_dir_all(b).unwrap(); for e in std::fs::read_dir(a).unwrap().flatten() { let (p, q) = (e.path(), b.join(e.file_name())); if p.is_dir() { kopieren(&p, &q) } else { std::fs::copy(&p, &q).unwrap(); } } }
        kopieren(&w.join("examples/c64/jumprun"), &d); d
    }
    #[test] fn normalziel_wird_erstes_ziel() {
        let (d, data) = (kopie("nz"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        normalziel_setzen(&d, &data, "atari8").unwrap();
        let v = info(&d, &data).unwrap(); assert_eq!(v["targets"][0]["id"], "atari8"); assert_eq!(v["targets"].as_array().unwrap().len(), 2);
        normalziel_setzen(&d, &data, "c64").unwrap(); assert_eq!(info(&d, &data).unwrap()["targets"][0]["id"], "c64");
        assert!(normalziel_setzen(&d, &data, "gibtsnicht").is_err()); std::fs::remove_dir_all(d).ok();
    }
    #[test] fn ziele_hinzufuegen_und_entfernen() {
        let (d, data) = (kopie("ziele"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let v = setzen(&d, &data, &json!({ "addTarget": "atari8" })).unwrap(); assert_eq!(v["targets"].as_array().unwrap().len(), 2);
        assert!(setzen(&d, &data, &json!({ "addTarget": "atari8" })).is_err(), "doppelt");
        assert!(setzen(&d, &data, &json!({ "addTarget": "gibtsnicht" })).is_err());
        let v = setzen(&d, &data, &json!({ "removeTarget": "atari8" })).unwrap(); assert_eq!(v["targets"].as_array().unwrap().len(), 1);
        assert!(setzen(&d, &data, &json!({ "removeTarget": "c64" })).is_err(), "das letzte Ziel bleibt"); std::fs::remove_dir_all(d).ok();
    }
    #[test] fn quellgruppen_ziele_und_diskette_aendern() {
        let (d, data) = (kopie("qg"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let v = info(&d, &data).unwrap(); let n0 = v["sourceGroups"].as_array().unwrap().len();
        let datei = v["sources"][0]["file"].as_str().unwrap().to_string();
        assert!(setzen(&d, &data, &json!({ "groupAdd": { "kind": "xyz" } })).is_err());
        let v = setzen(&d, &data, &json!({ "groupAdd": { "kind": "asm" } })).unwrap(); assert_eq!(v["sourceGroups"].as_array().unwrap().len(), n0 + 1);
        std::fs::write(d.join("neu.asm"), "rts\n").unwrap();
        assert!(setzen(&d, &data, &json!({ "fileAdd": { "group": n0, "file": "fehlt.asm" } })).is_err(), "Datei fehlt");
        assert!(setzen(&d, &data, &json!({ "fileAdd": { "group": n0, "file": datei } })).is_err(), "schon Quelle");
        assert!(setzen(&d, &data, &json!({ "fileAdd": { "group": n0, "file": "../x.asm" } })).is_err(), "Pfad außerhalb");
        let v = setzen(&d, &data, &json!({ "fileAdd": { "group": n0, "file": "neu.asm" } })).unwrap(); assert_eq!(v["sourceGroups"][n0]["files"][0]["file"], "neu.asm");
        let v = setzen(&d, &data, &json!({ "groupSet": { "i": n0, "kind": "basic", "dialect": "turbo_basic_xl" } })).unwrap(); assert_eq!(v["sourceGroups"][n0]["kind"], "basic");
        let v = setzen(&d, &data, &json!({ "fileRemove": "neu.asm" })).unwrap(); assert_eq!(v["sourceGroups"][n0]["files"].as_array().unwrap().len(), 0);
        let v = setzen(&d, &data, &json!({ "groupRemove": n0 })).unwrap(); assert_eq!(v["sourceGroups"].as_array().unwrap().len(), n0);
        let v = setzen(&d, &data, &json!({ "targetSet": { "id": "c64", "illegals": "warn" } })).unwrap(); assert!(v["targets"][0].is_object());
        assert!(setzen(&d, &data, &json!({ "disk": { "format": "xx" } })).is_err());
        let v = setzen(&d, &data, &json!({ "disk": { "format": "d81", "name": "TEST" } })).unwrap(); assert_eq!(v["disk"]["format"], "d81");
        let v = setzen(&d, &data, &json!({ "disk": null })).unwrap(); assert!(v["disk"].is_null()); std::fs::remove_dir_all(d).ok();
    }
    #[test] fn info_nennt_name_ziele_varianten_und_quellen() {
        let (d, data) = (kopie("info"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let v = info(&d, &data).unwrap();
        assert!(!v["name"].as_str().unwrap().is_empty()); assert_eq!(v["targets"][0]["system"], "c64"); let vs = v["targets"][0]["variants"].as_array().unwrap(); assert!(vs.contains(&json!("pal")) && vs.contains(&json!("ntsc")));
        assert!(!v["sources"].as_array().unwrap().is_empty()); assert_eq!(v["errors"], json!([])); std::fs::remove_dir_all(d).ok();
    }
    #[test] fn quellen_lassen_sich_in_der_gruppe_verschieben() {
        let (d, data) = (kopie("move"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let v = info(&d, &data).unwrap(); let gruppen = v["sourceGroups"].as_array().unwrap(); assert!(!gruppen.is_empty());
        let f = gruppen[0]["files"][0]["file"].as_str().unwrap().to_string(); let n = gruppen[0]["files"].as_array().unwrap().len();
        let v = setzen(&d, &data, &json!({ "move": { "file": f, "dir": 1 } })).unwrap(); let g = &v["sourceGroups"][0]["files"];
        if n > 1 { assert_eq!(g[1]["file"], json!(f)); } else { assert_eq!(g[0]["file"], json!(f), "eine einzelne Datei bleibt stehen"); }
        assert!(setzen(&d, &data, &json!({ "move": { "file": "gibts-nicht.asm", "dir": 1 } })).is_err()); std::fs::remove_dir_all(d).ok();
    }
    #[test] fn aenderungen_werden_gespeichert_und_ungueltige_abgelehnt() {
        let (d, data) = (kopie("setzen"), std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let v = setzen(&d, &data, &json!({ "name": "  Neuer Name ", "description": "Text" })).unwrap(); assert_eq!(v["name"], "Neuer Name"); assert_eq!(v["description"], "Text");
        assert_eq!(info(&d, &data).unwrap()["name"], "Neuer Name", "steht in der Datei");
        let e = setzen(&d, &data, &json!({ "name": "  " })).unwrap_err(); assert!(e.contains(""), "{e}");
        assert_eq!(info(&d, &data).unwrap()["name"], "Neuer Name", "ungültig: Datei unverändert");
        assert!(setzen(&d, &data, &json!({ "main": "gibts-nicht.asm" })).is_err());
        let v = setzen(&d, &data, &json!({ "variant": { "id": v_id(&info(&d, &data).unwrap()), "variant": "ntsc" } })).unwrap(); assert_eq!(v["targets"][0]["variant"], "ntsc");
        assert!(setzen(&d, &data, &json!({ "variant": { "id": v_id(&v), "variant": "secam" } })).is_err());
        std::fs::remove_dir_all(d).ok();
    }
    fn v_id(v: &Value) -> String { v["targets"][0]["id"].as_str().unwrap().to_string() }
}
