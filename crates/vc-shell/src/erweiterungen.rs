// SPDX-License-Identifier: GPL-3.0-or-later
//! Anbindung der Registry (`vc_ide::registry`) an die Oberfläche: Befehle aus Paketen, Release-Auswahl für Menü, Palette und Navigation, Auskunft für die Einstellungen.
//! Im Programm gibt es nur die Quelle „Programm“; weitere Quellen entstehen erst mit der Installation von Erweiterungen (E1). Für Entwicklung und Handprüfung
//! liest der Schalter `VC_EXTENSIONS_DEV=<Ordner mit entpackten Paketen>` (nur mit dem Cargo-Feature `dev-extensions`) Pakete als Benutzerquelle.
use serde_json::{json, Value};
use vc_commands::{Befehl, Braucht};
use vc_ide::registry::{Problem, Registry};

/// Die Registry dieses Programmlaufs
pub fn registry() -> Registry { Registry::programm(crate::lsp::daten_ordner().as_deref()).mit_entwicklungsordner() }

fn braucht(s: &str) -> Braucht { match s { "project" => Braucht::Projekt, "editor" => Braucht::Editor, _ => Braucht::Nichts } }

/// Befehle aus den Manifesten der Pakete. Fehlerhafte Befehle werden übersprungen und gemeldet.
pub fn befehle_aus(r: &Registry) -> (Vec<Befehl>, Vec<Problem>) {
    let (pakete, mut probleme) = r.pakete();
    let mut out = vec![];
    for p in pakete {
        for (i, c) in p.manifest["commands"].as_array().into_iter().flatten().enumerate() {
            let id = c["id"].as_str().unwrap_or("");
            let meld = |text: String| Problem { quelle: p.wurzel.quelle, herkunft: p.kennung.clone(), datei: p.wurzel.pfad.join("extension.json"), text };
            if id.is_empty() { probleme.push(meld(format!("Befehl {} ohne „id“", i + 1))); continue; }
            let titel: Vec<(String, String)> = c["title"].as_object().map(|o| o.iter().filter_map(|(k, v)| v.as_str().map(|t| (k.clone(), t.to_string()))).collect()).unwrap_or_default();
            if titel.is_empty() { probleme.push(meld(format!("Befehl „{id}“ ohne Titel"))); continue; }
            out.push(Befehl { id: format!("ext:{}:{id}", p.kennung), titel, menue: c["menu"].as_str().unwrap_or("Projekt").to_string(), untermenue: c["submenu"].as_str().unwrap_or("").to_string(),
                tasten: c["keys"].as_str().unwrap_or("").to_string(), braucht: braucht(c["needs"].as_str().unwrap_or("none")), herkunft: p.kennung.clone(), feature: c["feature"].as_str().map(String::from) });
        }
    }
    (out, probleme)
}

/// Dynamische Befehle aus der Registry neu setzen (beim Start); Rückgabe: Meldungen (fehlerhafte Manifeste, abgelehnte Befehle, belegte Kürzel)
pub fn befehle_laden() -> Vec<String> {
    let (befehle, probleme) = befehle_aus(&registry());
    let mut m: Vec<String> = probleme.iter().map(|p| format!("{}: {}", p.herkunft, p.text)).collect();
    m.extend(vc_commands::dynamisch::setze(befehle));
    m
}

/// Zeilen der Befehlstabelle nach der Release-Auswahl: Befehle eines abgewählten Editors entfallen (Menü, Palette)
pub fn zeilen_sichtbar(lang: &str) -> Vec<vc_commands::Zeile> {
    let f = vc_ide::features::features();
    vc_commands::dynamisch::zeilen(lang).into_iter().filter(|z| z.feature.as_deref().map_or(true, |s| f.studio(s)) && !vc_commands::NOCH_NICHT_GEBAUT.contains(&z.id.as_str())).collect()
}

/// Release-Auswahl der Editoren für die Oberfläche
pub fn features_json() -> Value {
    let f = vc_ide::features::features();
    let ids = ["sprite", "char", "screen", "tiles", "palette", "tracker", "play", "disk", "memmap", "extract"];
    json!({ "all": f.is_all(), "studios": ids.iter().map(|i| json!({ "id": i, "on": f.studio(i) })).collect::<Vec<_>>() })
}

/// Auskunft für die Einstellungsseite: Quellen, Pakete, Befehle und Meldungen
pub fn info_json() -> Value {
    let r = registry(); let (pakete, probleme) = r.pakete();
    json!({
        "sources": r.wurzeln().iter().map(|w| json!({ "kind": w.quelle.kennung(), "origin": w.herkunft(), "path": w.pfad.to_string_lossy() })).collect::<Vec<_>>(),
        "packages": pakete.iter().map(|p| json!({ "id": p.kennung, "version": p.version, "name": p.name, "kind": p.wurzel.quelle.kennung() })).collect::<Vec<_>>(),
        "commands": vc_commands::dynamisch::dynamische().iter().map(|b| json!({ "id": b.id, "origin": b.herkunft, "feature": b.feature })).collect::<Vec<_>>(),
        "problems": probleme.iter().map(|p| json!({ "origin": p.herkunft, "file": p.datei.to_string_lossy(), "text": p.text })).collect::<Vec<_>>(),
        "hasExtensions": r.hat_erweiterungen(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_ide::registry::Quelle;

    #[test]
    fn befehle_aus_manifesten() {
        let t = std::env::temp_dir().join(format!("vc_erw_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        let p = t.join("ext/autor.paket"); std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("extension.json"), r#"{"id":"autor.paket","version":"1.0.0","commands":[
            {"id":"hallo","title":{"de":"Hallo sagen","en":"Say hello"},"keys":"Cmd+Alt+Umschalt+K","needs":"project","feature":"sprite"},
            {"id":"ohne"},{"title":{"en":"x"}},{"id":"zwei","title":{"en":"Two"},"menu":"Datei","submenu":"Mehr"}]}"#).unwrap();
        let (b, pr) = befehle_aus(&Registry::programm(None).mit_paketordner(Quelle::Benutzer, &t.join("ext")));
        assert_eq!(b.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["ext:autor.paket:hallo", "ext:autor.paket:zwei"]);
        assert_eq!(pr.len(), 2, "{pr:?}");
        assert_eq!((b[0].braucht, b[0].feature.as_deref(), b[0].titel_in("de").as_str()), (Braucht::Projekt, Some("sprite"), "Hallo sagen"));
        assert_eq!((b[1].menue.as_str(), b[1].untermenue.as_str()), ("Datei", "Mehr"));
        std::fs::remove_dir_all(t).ok();
    }
    #[test]
    fn release_auswahl_blendet_befehle_eines_abgewaehlten_editors_aus() {
        let _g = vc_commands::test_sperre_pub();
        vc_commands::dynamisch::setze(vec![]);
        let alle = zeilen_sichtbar("de");
        assert!(alle.iter().any(|z| z.id == "png_laden"), "ohne release.json ist alles an");
        vc_ide::features::init(None);   // ohne Datenordner: alles an
        let tmp = std::env::temp_dir().join(format!("vc_rel_{}.json", std::process::id())); std::fs::write(&tmp, r#"{"format":1,"studios":["tracker"]}"#).unwrap();
        std::env::set_var("VC_RELEASE", &tmp); vc_ide::features::init(None);
        let z = zeilen_sichtbar("de");
        assert!(!z.iter().any(|x| x.id == "png_laden") && z.iter().any(|x| x.id == "tracker_noten") && z.iter().any(|x| x.id == "speichern"), "Sprite-Befehl weg, Tracker und allgemeine Befehle bleiben");
        std::env::remove_var("VC_RELEASE"); std::fs::remove_file(&tmp).ok(); vc_ide::features::init(None);
    }
}
