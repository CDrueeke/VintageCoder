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

// ---------------------------------------------------------------------------------------------- Zeichensatz des Projekts

/// Zeichensatz eines neuen Projekts: `Auto` = dein ROM hat Vorrang (nur ein `rom:`-Verweis in der Datei), sonst der eigene Standardsatz; `Eigen` = immer der eigene Satz (weitergebbar); `Keine`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zeichensatz { Auto, Eigen, Keine }
impl Zeichensatz {
    pub fn parse(s: &str) -> Option<Zeichensatz> { match s { "auto" => Some(Zeichensatz::Auto), "own" | "eigen" => Some(Zeichensatz::Eigen), "none" | "keine" => Some(Zeichensatz::Keine), _ => None } }
}

/// Legt den Zeichensatz des Projekts an (nach `create`): je System das erste Zeichensatzprofil mit eigenem Satz. Mit ROM (`Auto`) nur der Verweis in `graphics/`, sonst die **Kopie**
/// des eigenen Satzes in `graphics/vc-standard/` samt `HERKUNFT.txt`. Liefert die angelegten Dateien (relativ). Vorhandene Dateien bleiben unberührt.
pub fn zeichensatz_anlegen(data: &Path, dir: &Path, systems: &[String], wahl: Zeichensatz) -> Result<Vec<String>, String> { zeichensatz_plan(data, dir, systems, wahl, false) }

/// Welche Dateien `zeichensatz_anlegen` anlegen würde (für die Vorschau im Dialog); schreibt nichts
pub fn zeichensatz_vorschau(data: &Path, systems: &[String], wahl: Zeichensatz) -> Vec<String> { zeichensatz_plan(data, Path::new("."), systems, wahl, true).unwrap_or_default() }

fn zeichensatz_plan(data: &Path, dir: &Path, systems: &[String], wahl: Zeichensatz, trocken: bool) -> Result<Vec<String>, String> {
    if wahl == Zeichensatz::Keine { return Ok(vec![]) }
    let profiles = crate::load_profiles(data)?; let (mut files, mut sets) = (vec![], vec![]);
    for sys in systems {
        let Some(p) = profiles.iter().find(|p| p.kind == "charset" && p.target == *sys && !crate::project::eigene_saetze(data, &p.id).is_empty()) else { continue };
        if wahl == Zeichensatz::Auto && !crate::project::std_charsets_for_profile(data, &p.id, true).is_empty() {
            if let Some(r) = crate::project::default_std_charset(data, &p.id) {
                if trocken { if let Ok(rr) = crate::romref::RomRef::parse(&r) { files.push(format!("{}/{}_{}.vcchar", crate::naming::out_dir("graphics", "charset"), rr.system, rr.set)) } continue }
                match crate::project::add_std_charset(dir, &r, &p.id, "graphics", false) { Ok(v) => { if let Some(x) = v["path"].as_str() { files.push(x.to_string()) } } Err(e) if e.contains("existiert") => {} Err(e) => return Err(e) } continue }
        }
        let alle = crate::project::eigene_saetze(data, &p.id);
        let e = alle.iter().find(|e| e.satz == "upper" || e.satz == "standard").unwrap_or(&alle[0]).clone();
        let rel = format!("graphics/vc-standard/{}.vcchar", e.id); let ziel = dir.join(&rel);
        if trocken { files.push(rel); sets.push(e); continue }
        if ziel.exists() { continue }
        std::fs::create_dir_all(ziel.parent().unwrap()).map_err(|x| x.to_string())?;
        std::fs::copy(&e.datei, &ziel).map_err(|x| format!("{}: {x}", e.datei.display()))?; files.push(rel); sets.push(e);
    }
    if !sets.is_empty() {
        let t = herkunft_text(&sets);
        let h = dir.join("graphics/vc-standard/HERKUNFT.txt"); if trocken { files.push("graphics/vc-standard/HERKUNFT.txt".into()) } else if !h.exists() { std::fs::write(&h, t).map_err(|x| x.to_string())?; files.push("graphics/vc-standard/HERKUNFT.txt".into()) }
    }
    Ok(files)
}

/// Text der `HERKUNFT.txt` neben den kopierten eigenen Zeichensätzen
pub fn herkunft_text(sets: &[crate::project::EigenSatz]) -> String {
    let mut t = String::from("Herkunft der Zeichensätze in diesem Ordner\n===========================================\n\nDiese Dateien sind die eigenen Standardzeichensätze „VC-Standard“ von VintageCoder.\nUrheber: das VintageCoder-Projekt. Lizenz: CC0-1.0 (gemeinfrei, frei weitergebbar, auch in eigenen Programmen).\nEs wurden keine Original-ROMs und keine Nachbauten von Original-Zeichen benutzt.\nDie Gestaltung entstand aus den Regeln des Stilblatts und ist vom Projektinhaber freigegeben. Bei einfachen Formen in einem 8x8-Raster (Klammern, Punkt, Komma, Strich, Doppelpunkt, I-Familie, Linien- und Rahmenzeichen) sind Übereinstimmungen mit anderen Zeichensätzen funktional bedingt; eine lokale Abstandsprüfung ergab bei wenigen solcher Zeichen kleine Abstände, der Projektinhaber hat die Sätze trotzdem freigegeben. Das ist keine Rechtsaussage.\n\nSätze:\n");
        for e in sets { t.push_str(&format!("  {} (Revision {}, Lizenz {}): {}\n", e.id, e.revision, e.lizenz, e.name)) }
    t
}

/// Einen eigenen Standardzeichensatz als Kopie ins Projekt legen (`graphics/vc-standard/<id>.vcchar`, dazu `HERKUNFT.txt`, falls es sie noch nicht gibt); liefert den relativen Pfad
pub fn eigenen_satz_kopieren(data: &Path, dir: &Path, id: &str) -> Result<String, String> {
    let e = crate::project::eigene_saetze(data, "").into_iter().find(|e| e.id == id).ok_or_else(|| format!("Eigener Zeichensatz „{id}“ ist nicht mitgeliefert"))?;
    let rel = format!("graphics/vc-standard/{}.vcchar", e.id); let ziel = dir.join(&rel);
    if ziel.exists() { return Err(format!("{rel} existiert bereits")) }
    std::fs::create_dir_all(ziel.parent().unwrap()).map_err(|x| x.to_string())?; std::fs::copy(&e.datei, &ziel).map_err(|x| format!("{}: {x}", e.datei.display()))?;
    let h = dir.join("graphics/vc-standard/HERKUNFT.txt"); if !h.exists() { std::fs::write(&h, herkunft_text(&[e])).map_err(|x| x.to_string())?; }
    Ok(rel)
}

#[cfg(test)]
mod zeichensatz_tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
    #[test] fn eigener_satz_wird_als_kopie_mit_herkunft_angelegt_keine_wahl_legt_nichts_an() {
        let t = std::env::temp_dir().join(format!("vc-zs-{}", std::process::id())); let _ = std::fs::remove_dir_all(&t); std::fs::create_dir_all(&t).unwrap();
        assert_eq!(zeichensatz_anlegen(&data(), &t, &["c64".into()], Zeichensatz::Keine).unwrap(), Vec::<String>::new());
        let f = zeichensatz_anlegen(&data(), &t, &["c64".into(), "atari8".into()], Zeichensatz::Eigen).unwrap();
        assert!(f.contains(&"graphics/vc-standard/vc-c64-upper.vcchar".to_string()) && f.contains(&"graphics/vc-standard/vc-atari-standard.vcchar".to_string()) && f.contains(&"graphics/vc-standard/HERKUNFT.txt".to_string()), "{f:?}");
        let v: Value = serde_json::from_str(&std::fs::read_to_string(t.join("graphics/vc-standard/vc-c64-upper.vcchar")).unwrap()).unwrap(); assert_eq!(v["origin"]["license"], "CC0-1.0", "origin bleibt in der Kopie");
        assert!(std::fs::read_to_string(t.join("graphics/vc-standard/HERKUNFT.txt")).unwrap().contains("keine Original-ROMs"));
        assert!(zeichensatz_anlegen(&data(), &t, &["c64".into()], Zeichensatz::Eigen).unwrap().is_empty(), "zweiter Lauf: nichts überschrieben");
        assert_eq!(Zeichensatz::parse("auto"), Some(Zeichensatz::Auto)); assert_eq!(Zeichensatz::parse("x"), None);
        let _ = std::fs::remove_dir_all(&t);
    }
}
