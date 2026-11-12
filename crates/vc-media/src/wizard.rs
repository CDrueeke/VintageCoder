// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt-Wizard: legt aus einer Vorlage (`data/templates/`) ein Projekt an – `.vcprj`, Quelldateien und Ordner.
//!
//! Vorlagen sind Daten: `index.json` beschreibt Vorlagen (Systeme, Dateien, Quellen), `systems.json` die Ziel-Einträge der
//! Projektdatei. In Dateinamen steht `{system}` für das Zielsystem, in Dateiinhalten `{{name}}` für den Projektnamen.
//! Eine Vorlage kann `variants` haben (Steckmodul statt Programm): gleiche Projektdatei, nur andere Startquelltexte.
use serde_json::{json, Value};
use std::path::Path;

fn read_json(p: &Path) -> Result<Value, String> {
    let t = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_str(&t).map_err(|e| format!("{}: {e}", p.display()))
}

/// Verfügbare Vorlagen und Systeme (für Auswahldialoge)
pub fn templates(data: &Path) -> Result<Value, String> {
    let idx = read_json(&data.join("templates/index.json"))?;
    let sys = read_json(&data.join("templates/systems.json"))?;
    let systems: Vec<Value> = sys["systems"].as_object().ok_or("systems fehlt")?.iter().filter(|(k, _)| vc_target::target_available(data, k)).map(|(k, v)| json!({ "id": k, "label": v["label"] })).collect();
    Ok(json!({ "templates": idx["templates"], "systems": systems }))
}

fn valid_name(n: &str) -> bool { !n.is_empty() && n.len() <= 48 && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') }

/// Projekt in `dir` anlegen (der Ordner wird bei Bedarf erzeugt, darf aber noch keine `.vcprj` enthalten).
/// `cartridge` gilt weiter als Kurzform für die Vorlage `asm-hello` in der Variante `cartridge` (Steckmodul).
pub fn create(data: &Path, dir: &Path, name: &str, systems: &[String], template: &str) -> Result<Value, String> {
    if template == "cartridge" { create_with(data, dir, name, systems, "asm-hello", Some("cartridge")) } else { create_with(data, dir, name, systems, template, None) }
}

/// Wie `create`, mit einer Variante der Vorlage (`variants` in `index.json`, z. B. `cartridge` bei `asm-hello`): Die Variante ersetzt
/// Dateien und Quellen der Vorlage und hat eigene Systeme.
pub fn create_with(data: &Path, dir: &Path, name: &str, systems: &[String], template: &str, variant: Option<&str>) -> Result<Value, String> {
    if !valid_name(name) { return Err("Projektname: nur Buchstaben, Ziffern, _ und -".into()); }
    if systems.is_empty() { return Err("mindestens ein Zielsystem wählen".into()); }
    let idx = read_json(&data.join("templates/index.json"))?;
    let sys = read_json(&data.join("templates/systems.json"))?;
    let t = idx["templates"].as_array().and_then(|a| a.iter().find(|x| x["id"] == template)).ok_or_else(|| format!("Vorlage '{template}' unbekannt"))?;
    let t = match variant {
        None => t,
        Some(v) => t["variants"].as_array().and_then(|a| a.iter().find(|x| x["id"] == v)).ok_or_else(|| format!("Vorlage '{template}' hat keine Variante '{v}'"))?,
    };
    let label = match variant { None => template.to_string(), Some(v) => format!("{template}/{v}") };
    let allowed: Vec<&str> = t["systems"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
    for s in systems {
        if sys["systems"].get(s.as_str()).is_none() { return Err(format!("Zielsystem '{s}' unbekannt")); }
        if !vc_target::target_available(data, s) { return Err(vc_target::missing_message(s)); }
        if !allowed.contains(&s.as_str()) { return Err(format!("Vorlage '{label}' gibt es nicht für '{s}' (möglich: {})", allowed.join(", "))); }
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        if rd.flatten().any(|e| e.path().extension().map_or(false, |x| x == "vcprj")) { return Err(format!("{} enthält schon ein Projekt", dir.display())); }
    }
    std::fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
    let subst = |s: &str, sy: &str| s.replace("{system}", sy).replace("{{name}}", name);
    let (mut written, mut sources) = (vec![], vec![]);
    for s in systems {
        let mut files = vec![];
        for f in t["files"].as_array().into_iter().flatten() {
            let from = data.join("templates").join(subst(f["from"].as_str().ok_or("from fehlt")?, s));
            let to = subst(f["to"].as_str().ok_or("to fehlt")?, s);
            if to.starts_with('/') || to.contains("..") { return Err(format!("Vorlage: unzulässiger Zielpfad '{to}'")); }
            let text = std::fs::read_to_string(&from).map_err(|e| format!("{}: {e}", from.display()))?;
            let dest = dir.join(&to);
            if dest.exists() { return Err(format!("{to} existiert bereits")); }
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&dest, subst(&text, s)).map_err(|e| e.to_string())?;
            written.push(to);
        }
        for so in t["sources"].as_array().into_iter().flatten() { files.push((so["kind"].as_str().unwrap_or("asm").to_string(), subst(so["file"].as_str().unwrap_or(""), s))); }
        for kind in ["asm", "basic"] {
            let fs: Vec<&String> = files.iter().filter(|(k, _)| k == kind).map(|(_, f)| f).collect();
            if !fs.is_empty() { sources.push(json!({ "targets": [s], "kind": kind, "files": fs })); }
        }
    }
    let proj = json!({ "format": 1, "project": { "name": name, "description": format!("Vorlage '{}' ({})", t["name"].as_str().unwrap_or(&label), systems.join(", ")) },
        "targets": systems.iter().map(|s| sys["systems"][s.as_str()]["target"].clone()).collect::<Vec<_>>(), "sources": sources });
    let pf = format!("{name}.vcprj");
    std::fs::write(dir.join(&pf), serde_json::to_string_pretty(&proj).unwrap() + "\n").map_err(|e| e.to_string())?;
    written.insert(0, pf.clone());
    Ok(json!({ "ok": true, "dir": dir.to_string_lossy(), "project": pf, "files": written }))
}
