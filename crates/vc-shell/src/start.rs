// SPDX-License-Identifier: GPL-3.0-or-later
//! Startseite (E1.4): zuletzt geöffnete Projekte anheften und entfernen, Beispielprojekte, erkannte Emulatoren und Projekt kopieren.
//! Die Logik steht in `vc-ide-model` und `vc-ide`; hier nur die Verbindung zur Oberfläche.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::platform;
use vc_ide::targets::{kind_label, EmulatorCfg};
use vc_ide_model::recent;

/// Zuletzt geöffnete Projekte für die Oberfläche
/// Projektbild `projektbild.png` eines Projekts (höchstens 600 KB, nur PNG) als Datenadresse; fehlt es, `null` (die Oberfläche zeigt eine Ersatzkachel).
/// Beispielbilder liegen nur im Programmpaket, nicht im Quelltext.
pub fn projektbild(dir: &Path) -> Value {
    let p = dir.join("projektbild.png");
    let Ok(m) = std::fs::metadata(&p) else { return Value::Null }; if !m.is_file() || m.len() > 600_000 { return Value::Null }
    match std::fs::read(&p) { Ok(b) if b.starts_with(&[0x89, b'P', b'N', b'G']) => json!(format!("data:image/png;base64,{}", crate::hilfe::base64(&b))), _ => Value::Null }
}

pub fn recent_json() -> Value {
    let now = recent::now();
    Value::Array(recent::load(None).iter().map(|r| json!({
        "path": r.path.to_string_lossy(), "pretty": recent::pretty(&r.path), "name": r.name, "pinned": r.pinned, "missing": r.missing,
        "systems": r.info.systems.iter().map(|(s, v)| json!([recent::short_name(s), v])).collect::<Vec<_>>(),
        "langs": r.info.langs, "emulator": r.info.emulator, "ago": recent::ago(r.last, now), "image": if r.missing { Value::Null } else { projektbild(&r.path) },
    })).collect())
}
fn index_of(v: &[recent::Recent], path: &str) -> Option<usize> { let k = recent::schluessel(std::path::Path::new(path)); v.iter().position(|r| recent::schluessel(&r.path) == k) }
pub fn pin(path: &str) -> Value { let mut v = recent::load(None); if let Some(i) = index_of(&v, path) { recent::toggle_pin(&mut v, i); } recent_json() }
pub fn remove(path: &str) -> Value { let mut v = recent::load(None); if let Some(i) = index_of(&v, path) { recent::remove(&mut v, i); } recent_json() }

/// Anzeigename eines Systemordners unter `examples/`
fn system_name(dir: &str) -> String {
    match dir { "c64" => "C64", "c128" => "C128", "c16" => "C16", "plus4" => "Plus/4", "vic20" => return vc_i18n::t!("system-name-vic20").to_string(), "pet" => "PET", "mega65" => "MEGA65", "atari" => "Atari 8-Bit", x => x }.to_string()
}
/// Empfohlene Einsteigerbeispiele (stehen auf der Startseite; der Rest im Dialog „Alle Beispiele“)
const EMPFOHLEN: [&str; 4] = ["c64/jumprun", "c64/basic_native", "mega65/hello", "atari/jumprun"];

/// Beispielprojekte aus allen Quellen der Registry (`examples/<System>/<Projekt>` mit einer `.vcprj`), mit Titel und Beschreibung aus der Projektdatei.
/// Die Beispiele des Programms stehen neben dem Datenordner; jede weitere Quelle (Erweiterung) bringt ihre eigenen mit, gekennzeichnet in `source`.
pub fn beispiele(r: &vc_ide::registry::Registry) -> Value {
    let mut alle: Vec<Value> = vec![];
    // Programm zuerst, dann Erweiterungen nach Vorrang aufsteigend (die Startseite zeigt empfohlene zuerst, der Dialog sortiert nach System)
    for (w, dir) in r.suchpfade("examples").into_iter().rev() {
        let quelle = w.paket.clone();
        let mut v: Vec<(String, String, PathBuf, String, String)> = vec![];
        let englisch = vc_i18n::language() == vc_i18n::Lang::En;
        for sys in std::fs::read_dir(&dir).into_iter().flatten().flatten().filter(|e| e.path().is_dir() && e.file_name() != "en") {   // `en` ist der englische Spiegel, kein System
            for p in std::fs::read_dir(sys.path()).into_iter().flatten().flatten().filter(|e| e.path().is_dir()) {
                let Some(f) = std::fs::read_dir(p.path()).into_iter().flatten().flatten().map(|f| f.path()).find(|f| f.extension().is_some_and(|x| x == "vcprj")) else { continue };
                let meta = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
                // Englisch: Titel und Beschreibung aus dem Spiegel `examples/en/<system>/<projekt>/<projekt>.vcprj`, falls vorhanden
                let spiegel = if englisch { spiegel_projekt(&dir, &sys.file_name().to_string_lossy(), &p.file_name().to_string_lossy()) } else { None };
                let text = |k: &str| spiegel.as_ref().and_then(|m| m["project"][k].as_str()).filter(|x| !x.is_empty()).or_else(|| meta.as_ref().and_then(|m| m["project"][k].as_str())).unwrap_or("").to_string();
                v.push((sys.file_name().to_string_lossy().into_owned(), p.file_name().to_string_lossy().into_owned(), p.path(), text("name"), text("description")));
            }
        }
        v.sort();
        // Im Programmpaket (macOS: `….app/Contents/Resources`) sind die Beispiele schreibgeschützt: dort nur „Kopieren“ (Bauen würde ins Paket schreiben und es beschädigen); Beispiele aus Erweiterungen ebenso
        let readonly = quelle.is_some() || dir.to_string_lossy().contains(".app/Contents/");
        alle.extend(v.into_iter().map(|(sys, name, p, titel, beschr)| json!({
            "name": format!("{} / {}", sys, name), "system": system_name(&sys), "title": if titel.is_empty() { name.clone() } else { titel }, "description": beschr,
            "path": p.to_string_lossy(), "image": projektbild(&p), "readonly": readonly, "featured": quelle.is_none() && EMPFOHLEN.contains(&format!("{sys}/{name}").as_str()), "source": quelle,
        })));
    }
    json!(alle)
}

/// Projektdatei des englischen Spiegels (`<examples>/en/<system>/<projekt>/*.vcprj`) als JSON
fn spiegel_projekt(examples: &Path, sys: &str, proj: &str) -> Option<Value> {
    let d = examples.join("en").join(sys).join(proj);
    let f = std::fs::read_dir(&d).ok()?.flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|x| x == "vcprj"))?;
    serde_json::from_str(&std::fs::read_to_string(f).ok()?).ok()
}

/// Englischer Spiegel eines kopierten Beispiels: die Dateien aus `<examples>/en/<system>/<projekt>/` ersetzen (gleicher Unterpfad) die der Kopie `ziel`.
/// `quelle` ist das kopierte Beispielprojekt `<examples>/<system>/<projekt>`; ohne Spiegel bleibt alles Deutsch. Rückgabe: Zahl der überlagerten Dateien.
pub fn spiegel_ueberlagern(quelle: &Path, ziel: &Path) -> usize {
    let (Some(proj), Some(sys)) = (quelle.file_name(), quelle.parent().and_then(|p| p.file_name())) else { return 0 };
    let Some(examples) = quelle.parent().and_then(|p| p.parent()) else { return 0 };
    let spiegel = examples.join("en").join(sys).join(proj);
    if !spiegel.is_dir() { return 0; }
    fn kopieren(von: &Path, nach: &Path) -> usize {
        let mut n = 0;
        for e in std::fs::read_dir(von).into_iter().flatten().flatten() {
            let (p, z) = (e.path(), nach.join(e.file_name()));
            if p.is_dir() { let _ = std::fs::create_dir_all(&z); n += kopieren(&p, &z); } else if std::fs::copy(&p, &z).is_ok() { n += 1; }
        }
        n
    }
    kopieren(&spiegel, ziel)
}

/// Erkannte Emulatoren (Suche nach Programmen, kein Netz)
pub fn werkzeuge(data: &Path) -> Value {
    let set = vc_ide::settings::get(); let mut gesehen: Vec<&str> = vec![]; let mut out = vec![];
    for sys in vc_ide::targets::installed_systems(Some(data)) { for k in sys.kinds() {
        if gesehen.contains(&k.id) || !vc_ide::emulators::enabled(k.id) { continue; }
        gesehen.push(k.id);
        let pfad = set.target(sys.id).and_then(|t| t.emulators.iter().find(|e| e.kind == k.id)).map(|e| e.path.clone()).unwrap_or_default();
        let p = platform::presence(&EmulatorCfg { id: k.id.into(), kind: k.id.into(), path: pfad, ..Default::default() });
        out.push(json!({ "id": k.id, "label": kind_label(k.id), "found": p.found, "text": p.text }));
    } }
    Value::Array(out)
}

/// Erster freier Name „<Name> Kopie“, „<Name> Kopie 2“ … im Zielordner
pub fn kopie_vorschlag(quelle: &Path, ort: &Path) -> String {
    let basis = quelle.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| vc_i18n::t!("kopierdlg-projekt").into());
    let erst = vc_i18n::t!("kopierdlg-basis-kopie", basis = &basis); if !ort.join(&erst).exists() { return erst; }
    (2..1000).map(|i| vc_i18n::t!("kopierdlg-erst-i", erst = &erst, i = &i)).find(|n| !ort.join(n).exists()).unwrap_or(erst)
}
/// Warum das Kopieren noch nicht geht (`None`: es geht)
pub fn kopie_problem(quelle: &Path, ort: &Path, name: &str) -> Option<String> {
    if let Err(e) = vc_ide::copyproject::validate_name(name) { return Some(e); }
    let ziel = ort.join(name.trim());
    if ziel.exists() && !(ziel.is_dir() && std::fs::read_dir(&ziel).is_ok_and(|mut d| d.next().is_none())) { return Some(vc_i18n::t!("kopierdlg-p1-gibt-es-schon", p1 = recent::pretty(&ziel))); }
    if ziel.starts_with(quelle) { return Some(vc_i18n::t!("kopierdlg-das-ziel-laege-im-quellordner").into()); }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn kopie_vorschlag_zaehlt_hoch_und_prueft_ziel() {
        let tmp = std::env::temp_dir().join(format!("vc_start_{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp);
        let quelle = tmp.join("demo"); std::fs::create_dir_all(&quelle).unwrap();
        let n1 = kopie_vorschlag(&quelle, &tmp); std::fs::create_dir_all(tmp.join(&n1)).unwrap(); std::fs::write(tmp.join(&n1).join("x"), "").unwrap();
        let n2 = kopie_vorschlag(&quelle, &tmp); assert_ne!(n1, n2, "der zweite Vorschlag ist ein freier Name");
        assert!(kopie_problem(&quelle, &tmp, &n1).is_some(), "ein vorhandenes, nicht leeres Ziel wird abgelehnt");
        assert!(kopie_problem(&quelle, &tmp, &n2).is_none());
        assert!(kopie_problem(&quelle, &quelle, "innen").is_some(), "ein Ziel im Quellordner wird abgelehnt");
        assert!(kopie_problem(&quelle, &tmp, "").is_some());
        let _ = std::fs::remove_dir_all(&tmp);
    }
    #[test] fn beispiele_aus_programm_und_erweiterung_mit_herkunft() {
        let t = std::env::temp_dir().join(format!("vc_bsp_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        for (w, sys, name, titel) in [("prog", "c64", "eins", "Eins"), ("ext/autor.demo", "c64", "zwei", "Zwei")] {
            let d = t.join(w).join("examples").join(sys).join(name); std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(format!("{name}.vcprj")), format!(r#"{{"project":{{"name":"{titel}","description":"d"}}}}"#)).unwrap();
        }
        std::fs::create_dir_all(t.join("prog/data")).unwrap();
        let r = vc_ide::registry::Registry::programm(Some(&t.join("prog/data"))).mit_paketordner(vc_ide::registry::Quelle::Benutzer, &t.join("ext"));
        let v = beispiele(&r); let a = v.as_array().unwrap();
        assert_eq!(a.iter().map(|x| (x["title"].as_str().unwrap(), x["source"].as_str(), x["readonly"].as_bool().unwrap())).collect::<Vec<_>>(), vec![("Eins", None, false), ("Zwei", Some("autor.demo"), true)]);
        std::fs::remove_dir_all(t).ok();
    }
    #[test] fn englischer_spiegel_titel_und_kopie_ueberlagern() {
        let t = std::env::temp_dir().join(format!("vc_spiegel_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        let proj = t.join("prog/examples/c64/rand"); std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("rand.vcprj"), r#"{"project":{"name":"C64: Rand","description":"Deutsch"}}"#).unwrap(); std::fs::write(proj.join("rand.asm"), "; deutsch\nnop\n").unwrap(); std::fs::write(proj.join("nur_de.bin"), "x").unwrap();
        let sp = t.join("prog/examples/en/c64/rand"); std::fs::create_dir_all(&sp).unwrap();
        std::fs::write(sp.join("rand.vcprj"), r#"{"project":{"name":"C64: border","description":"English"}}"#).unwrap(); std::fs::write(sp.join("rand.asm"), "; english\nnop\n").unwrap();
        std::fs::create_dir_all(t.join("prog/data")).unwrap();
        let r = vc_ide::registry::Registry::programm(Some(&t.join("prog/data")));
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        let v = beispiele(&r); let a = v.as_array().unwrap();
        assert_eq!(a.len(), 1, "der Ordner en ist kein System: {a:?}"); assert_eq!((a[0]["title"].as_str(), a[0]["description"].as_str()), (Some("C64: border"), Some("English")));
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(beispiele(&r)[0]["title"], "C64: Rand");
        // Kopie: deutsch kopiert, dann die englischen Dateien darüber
        let kopie = t.join("kopie"); std::fs::create_dir_all(&kopie).unwrap(); for f in ["rand.vcprj", "rand.asm", "nur_de.bin"] { std::fs::copy(proj.join(f), kopie.join(f)).unwrap(); }
        assert_eq!(spiegel_ueberlagern(&proj, &kopie), 2);
        assert!(std::fs::read_to_string(kopie.join("rand.asm")).unwrap().contains("english") && kopie.join("nur_de.bin").exists());
        assert_eq!(spiegel_ueberlagern(&t.join("prog/examples/c64/ohne"), &kopie), 0);
        vc_i18n::set_thread_language(None);
        std::fs::remove_dir_all(t).ok();
    }
    #[test] fn beispiele_ohne_ordner_sind_leer() { assert_eq!(beispiele(&vc_ide::registry::Registry::programm(Some(Path::new("/nirgendwo/data")))), json!([])); }
}
