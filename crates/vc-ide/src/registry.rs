// SPDX-License-Identifier: GPL-3.0-or-later
//! Registry: ein gemeinsames Verzeichnis der Quellen, aus denen das Programm Daten liest (Vorlagen, Systeme, Beispiele, Paletten, Hilfe, Include-Dateien …).
//! Jede Quelle hat eine **Herkunft** und einen **Vorrang**: Projekt > Benutzer > System-weit > Programm. Die Registry liefert Suchpfade, die erste passende Datei
//! und zusammengeführte JSON-Listen (Einträge nach Kennung, mit Herkunft); **Konflikte** (dieselbe Kennung aus mehreren Quellen) werden gemeldet und nie still aufgelöst,
//! kaputte Dateien werden übersprungen und gemeldet.
//!
//! Aufbau einer Quelle (wie ein Paket, Konzept „Erweiterungen“ Abschnitt 6): `<wurzel>/data/<art>/…`, `<wurzel>/help`, `<wurzel>/examples`, `<wurzel>/lib`.
//! Im Programm gibt es nur die Quelle „Programm“ (Wurzel = Ordner über `data/`); Benutzer-, Projekt- und System-weite Quellen sind gebaut und getestet,
//! werden im Programm aber erst mit der Installation von Erweiterungen aktiv (Etappe E1). Für die Entwicklung liest [`Registry::mit_entwicklungsordner`] einen Ordner mit
//! entpackten Paketen (Umgebungsvariable `VC_EXTENSIONS_DEV`, nur mit dem Cargo-Feature `dev-extensions`, nie im Auslieferungsbau).
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Art der Quelle; die Reihenfolge ist der Vorrang (später = höher)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quelle { Programm, SystemWeit, Benutzer, Projekt }
impl Quelle {
    pub fn kennung(self) -> &'static str { match self { Quelle::Programm => "program", Quelle::SystemWeit => "system", Quelle::Benutzer => "user", Quelle::Projekt => "project" } }
}

/// Eine Wurzel einer Quelle
#[derive(Debug, Clone, PartialEq)]
pub struct Wurzel {
    pub quelle: Quelle,
    pub pfad: PathBuf,
    /// Kennung der Erweiterung; `None` für das Programm
    pub paket: Option<String>,
}
impl Wurzel {
    /// Name für die Anzeige der Herkunft (`Programm` oder die Kennung des Pakets)
    pub fn herkunft(&self) -> String { self.paket.clone().unwrap_or_else(|| "Programm".to_string()) }
}

/// Ein Eintrag einer zusammengeführten Liste
#[derive(Debug, Clone, PartialEq)]
pub struct Eintrag { pub kennung: String, pub quelle: Quelle, pub herkunft: String, pub datei: PathBuf, pub wert: Value }

/// Dieselbe Kennung aus mehreren Quellen: `gewinner` ist der Index in `anbieter` (höchster Vorrang), `ungeloest` bei Gleichstand der Vorrangstufe (zwei Pakete gleicher Art)
#[derive(Debug, Clone, PartialEq)]
pub struct Konflikt { pub kennung: String, pub anbieter: Vec<(Quelle, String)>, pub gewinner: usize, pub ungeloest: bool }

/// Eine übersprungene Datei oder ein übersprungener Eintrag, mit Grund
#[derive(Debug, Clone, PartialEq)]
pub struct Problem { pub quelle: Quelle, pub herkunft: String, pub datei: PathBuf, pub text: String }

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Liste { pub eintraege: Vec<Eintrag>, pub konflikte: Vec<Konflikt>, pub probleme: Vec<Problem> }

/// Ein Paket mit gelesenem Manifest `extension.json` (Konzept Anhang A; für die Registry zählen `id`, `version`, `name` und `commands`)
#[derive(Debug, Clone, PartialEq)]
pub struct Paket { pub wurzel: Wurzel, pub kennung: String, pub version: String, pub name: String, pub manifest: Value }

#[derive(Debug, Clone, Default)]
pub struct Registry { wurzeln: Vec<Wurzel> }

impl Registry {
    /// Nur das Programm: `data_dir` ist der Datenordner (`…/data`), die Wurzel der Ordner darüber
    pub fn programm(data_dir: Option<&Path>) -> Registry {
        let mut r = Registry::default();
        if let Some(root) = data_dir.and_then(|d| d.parent()) { r.wurzeln.push(Wurzel { quelle: Quelle::Programm, pfad: root.to_path_buf(), paket: None }); }
        r
    }
    /// Weitere Quelle hinzufügen
    pub fn mit_quelle(mut self, quelle: Quelle, pfad: PathBuf, paket: Option<String>) -> Registry { self.wurzeln.push(Wurzel { quelle, pfad, paket }); self }
    /// Alle Pakete in den Unterordnern von `ordner` (je Unterordner ein entpacktes Paket, Kennung = Ordnername) als Quelle `quelle`
    pub fn mit_paketordner(mut self, quelle: Quelle, ordner: &Path) -> Registry {
        let mut namen: Vec<PathBuf> = std::fs::read_dir(ordner).map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
        namen.sort();
        for p in namen { let id = p.file_name().map(|n| n.to_string_lossy().into_owned()); self.wurzeln.push(Wurzel { quelle, pfad: p, paket: id }); }
        self
    }
    /// Entwicklerschalter: Ordner aus `VC_EXTENSIONS_DEV` als Benutzerquelle. Ohne das Feature `dev-extensions` wirkungslos (der Auslieferungsbau liest nichts Fremdes).
    pub fn mit_entwicklungsordner(self) -> Registry {
        #[cfg(feature = "dev-extensions")]
        if let Some(d) = std::env::var_os("VC_EXTENSIONS_DEV").map(PathBuf::from) { return self.mit_paketordner(Quelle::Benutzer, &d); }
        self
    }

    /// Wurzeln nach Vorrang absteigend (Projekt zuerst); bei gleicher Stufe nach Paketkennung aufsteigend (die alphabetisch erste gewinnt, stabil)
    pub fn wurzeln(&self) -> Vec<&Wurzel> {
        let mut v: Vec<&Wurzel> = self.wurzeln.iter().collect();
        v.sort_by(|a, b| b.quelle.cmp(&a.quelle).then_with(|| a.paket.cmp(&b.paket)));
        v
    }
    /// Gibt es außer dem Programm weitere Quellen?
    pub fn hat_erweiterungen(&self) -> bool { self.wurzeln.iter().any(|w| w.quelle != Quelle::Programm) }

    /// Vorhandene Ordner `<wurzel>/<art>` (z. B. `data/templates`, `examples`, `help`) mit ihrer Wurzel, nach Vorrang
    pub fn suchpfade(&self, art: &str) -> Vec<(&Wurzel, PathBuf)> {
        self.wurzeln().into_iter().filter_map(|w| { let p = w.pfad.join(art); p.is_dir().then_some((w, p)) }).collect()
    }
    /// Erste vorhandene Datei `<wurzel>/<art>/<rel>` nach Vorrang. `rel` darf nicht aus dem Ordner herausführen.
    pub fn datei(&self, art: &str, rel: &str) -> Option<(&Wurzel, PathBuf)> {
        if rel.is_empty() || rel.split(['/', '\\']).any(|s| s == ".." || s.is_empty()) || Path::new(rel).is_absolute() { return None; }
        self.wurzeln().into_iter().find_map(|w| { let p = w.pfad.join(art).join(rel); p.is_file().then_some((w, p)) })
    }

    /// Die Manifeste aller Quellen außer dem Programm (`<wurzel>/extension.json`), nach Vorrang. Fehlt die Datei oder ist sie unlesbar, wird die Quelle übersprungen und gemeldet;
    /// die Kennung im Manifest muss zum Paketnamen (Ordnername bzw. `paket`) passen, sonst gilt der Paketname und es gibt ein Problem.
    pub fn pakete(&self) -> (Vec<Paket>, Vec<Problem>) {
        let (mut pakete, mut probleme) = (vec![], vec![]);
        for w in self.wurzeln().into_iter().filter(|w| w.quelle != Quelle::Programm) {
            let datei = w.pfad.join("extension.json");
            let problem = |text: String| Problem { quelle: w.quelle, herkunft: w.herkunft(), datei: datei.clone(), text };
            let v: Value = match std::fs::read_to_string(&datei).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) { Ok(v) => v, Err(e) => { probleme.push(problem(e)); continue; } };
            let name_im_ordner = w.paket.clone().unwrap_or_default();
            let id = v["id"].as_str().unwrap_or("").to_string();
            if id.is_empty() { probleme.push(problem("Manifest ohne „id“".into())); continue; }
            if !name_im_ordner.is_empty() && id != name_im_ordner { probleme.push(problem(format!("Kennung „{id}“ passt nicht zum Paketnamen „{name_im_ordner}“"))); }
            pakete.push(Paket { wurzel: w.clone(), kennung: id.clone(), version: v["version"].as_str().unwrap_or("").to_string(), name: v["name"].as_str().unwrap_or(&id).to_string(), manifest: v });
        }
        (pakete, probleme)
    }

    /// Zusammengeführte Liste aus der JSON-Datei `<art>/<datei>` jeder Quelle. Die Datei ist ein Array oder ein Objekt mit dem Array unter `liste`; jedes Element ein Objekt,
    /// dessen `id_feld` die Kennung ist. Je Kennung gewinnt die Quelle mit dem höchsten Vorrang; jede Kennung mit mehreren Anbietern ergibt einen Konflikt.
    /// Die Reihenfolge der Einträge: Programm zuerst in Dateireihenfolge, danach neue Kennungen höherer Quellen; ein überlagerter Eintrag behält die Stelle des ersten Auftretens.
    pub fn json_liste(&self, art: &str, datei: &str, liste: Option<&str>, id_feld: &str) -> Liste {
        let mut out = Liste::default();
        let mut je_id: BTreeMap<String, Vec<Eintrag>> = BTreeMap::new(); let mut reihenfolge: Vec<String> = vec![];
        let mut wurzeln = self.wurzeln(); wurzeln.reverse();   // niedrigster Vorrang zuerst: Reihenfolge der Kennungen aus dem Programm
        for w in wurzeln {
            let p = w.pfad.join(art).join(datei); if !p.is_file() { continue; }
            let problem = |text: String| Problem { quelle: w.quelle, herkunft: w.herkunft(), datei: p.clone(), text };
            let v: Value = match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string())) { Ok(v) => v, Err(e) => { out.probleme.push(problem(e)); continue; } };
            let arr = match liste { Some(k) => v.get(k).and_then(|x| x.as_array()), None => v.as_array() };
            let Some(arr) = arr else { out.probleme.push(problem(format!("keine Liste{}", liste.map(|k| format!(" unter „{k}“")).unwrap_or_default()))); continue; };
            for (i, e) in arr.iter().enumerate() {
                let Some(id) = e.get(id_feld).and_then(|x| x.as_str()).filter(|s| !s.is_empty()) else { out.probleme.push(problem(format!("Eintrag {} ohne „{id_feld}“", i + 1))); continue; };
                if !je_id.contains_key(id) { reihenfolge.push(id.to_string()); }
                je_id.entry(id.to_string()).or_default().push(Eintrag { kennung: id.to_string(), quelle: w.quelle, herkunft: w.herkunft(), datei: p.clone(), wert: e.clone() });
            }
        }
        for id in reihenfolge {
            let mut anbieter = je_id.remove(&id).unwrap_or_default();   // niedrigster Vorrang zuerst
            if anbieter.len() > 1 {
                let hoechste = anbieter.last().map(|e| e.quelle).unwrap();
                let gleiche = anbieter.iter().filter(|e| e.quelle == hoechste).count();
                // Gewinner = letzter Eintrag (höchster Vorrang; bei gleicher Stufe die alphabetisch erste Paketkennung, aber als ungelöst gemeldet)
                out.konflikte.push(Konflikt { kennung: id.clone(), anbieter: anbieter.iter().map(|e| (e.quelle, e.herkunft.clone())).collect(), gewinner: anbieter.len() - 1, ungeloest: gleiche > 1 });
            }
            if let Some(e) = anbieter.pop() { out.eintraege.push(e); }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(name: &str) -> PathBuf { let p = std::env::temp_dir().join(format!("vc_registry_{name}_{}", std::process::id())); let _ = fs::remove_dir_all(&p); fs::create_dir_all(&p).unwrap(); p }
    fn schreibe(p: &Path, t: &str) { fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(p, t).unwrap(); }

    /// Programm mit `templates/index.json` ([a, b]); Paket „p1“ (Benutzer) überlagert b und bringt c; Paket „p2“ (Benutzer) bringt c ebenfalls; Projekt überlagert a
    fn aufbau(name: &str) -> (PathBuf, Registry) {
        let t = tmp(name);
        schreibe(&t.join("prog/data/cpus/x.json"), "{}");
        schreibe(&t.join("prog/data/templates/index.json"), r#"[{"id":"a","t":"Programm A"},{"id":"b","t":"Programm B"}]"#);
        schreibe(&t.join("ext/p1/data/templates/index.json"), r#"{"liste":[{"id":"b","t":"P1 B"},{"id":"c","t":"P1 C"}]}"#);
        schreibe(&t.join("ext/p2/data/templates/index.json"), r#"{"liste":[{"id":"c","t":"P2 C"}]}"#);
        schreibe(&t.join("projekt/data/templates/index.json"), r#"{"liste":[{"id":"a","t":"Projekt A"}]}"#);
        let r = Registry::programm(Some(&t.join("prog/data"))).mit_paketordner(Quelle::Benutzer, &t.join("ext")).mit_quelle(Quelle::Projekt, t.join("projekt"), Some("projekt".into()));
        (t, r)
    }

    #[test]
    fn nur_das_programm_ohne_erweiterungen() {
        let t = tmp("prog"); schreibe(&t.join("data/templates/index.json"), r#"[{"id":"a"}]"#);
        let r = Registry::programm(Some(&t.join("data")));
        assert!(!r.hat_erweiterungen()); assert_eq!(r.wurzeln().len(), 1);
        let l = r.json_liste("data/templates", "index.json", None, "id"); assert_eq!(l.eintraege.len(), 1); assert!(l.konflikte.is_empty() && l.probleme.is_empty());
        assert_eq!(l.eintraege[0].herkunft, "Programm");
        assert!(Registry::programm(None).wurzeln().is_empty());
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn vorrang_projekt_vor_benutzer_vor_programm_und_herkunft() {
        let (t, r) = aufbau("vorrang");
        let reihenfolge: Vec<String> = r.wurzeln().iter().map(|w| format!("{}:{}", w.quelle.kennung(), w.herkunft())).collect();
        assert_eq!(reihenfolge, ["project:projekt", "user:p1", "user:p2", "program:Programm"]);
        let l = r.json_liste("data/templates", "index.json", Some("liste"), "id");
        // Programm hat eine Liste ohne Schlüssel „liste“: wird gemeldet, nicht stillschweigend ignoriert
        assert!(l.probleme.iter().any(|p| p.herkunft == "Programm" && p.text.contains("keine Liste")), "{:?}", l.probleme);
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn zusammenfuehren_konflikte_und_ungeloeste_gleichstaende() {
        let t = tmp("zus");
        schreibe(&t.join("prog/data/x/index.json"), r#"{"liste":[{"id":"a","t":"Programm A"},{"id":"b","t":"Programm B"}]}"#);
        schreibe(&t.join("ext/p1/data/x/index.json"), r#"{"liste":[{"id":"b","t":"P1 B"},{"id":"c","t":"P1 C"}]}"#);
        schreibe(&t.join("ext/p2/data/x/index.json"), r#"{"liste":[{"id":"c","t":"P2 C"}]}"#);
        schreibe(&t.join("projekt/data/x/index.json"), r#"{"liste":[{"id":"a","t":"Projekt A"}]}"#);
        let r = Registry::programm(Some(&t.join("prog/data"))).mit_paketordner(Quelle::Benutzer, &t.join("ext")).mit_quelle(Quelle::Projekt, t.join("projekt"), Some("projekt".into()));
        let l = r.json_liste("data/x", "index.json", Some("liste"), "id");
        let texte: Vec<(String, String)> = l.eintraege.iter().map(|e| (e.kennung.clone(), e.wert["t"].as_str().unwrap().into())).collect();
        assert_eq!(texte, [("a".into(), "Projekt A".into()), ("b".into(), "P1 B".into()), ("c".into(), "P1 C".into())], "höchster Vorrang gewinnt, Reihenfolge des ersten Auftretens");
        let k = |id: &str| l.konflikte.iter().find(|k| k.kennung == id).unwrap();
        assert_eq!(l.konflikte.len(), 3);
        assert_eq!(k("a").anbieter, vec![(Quelle::Programm, "Programm".to_string()), (Quelle::Projekt, "projekt".to_string())]); assert!(!k("a").ungeloest);
        assert!(!k("b").ungeloest && k("b").gewinner == 1);
        assert!(k("c").ungeloest, "zwei Pakete gleicher Stufe bringen dieselbe Kennung: Konflikt ohne eindeutigen Vorrang");
        assert_eq!(l.eintraege.iter().find(|e| e.kennung == "c").unwrap().herkunft, "p1", "bei Gleichstand gewinnt die alphabetisch erste Paketkennung (und es bleibt ein gemeldeter Konflikt)");
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn suchpfade_und_erste_datei_nach_vorrang() {
        let (t, r) = aufbau("suche");
        schreibe(&t.join("prog/data/templates/gemeinsam.txt"), "programm"); schreibe(&t.join("ext/p1/data/templates/gemeinsam.txt"), "p1"); schreibe(&t.join("ext/p2/data/templates/nur_p2.txt"), "p2");
        let pfade: Vec<String> = r.suchpfade("data/templates").iter().map(|(w, _)| w.herkunft()).collect(); assert_eq!(pfade, ["projekt", "p1", "p2", "Programm"]);
        assert!(r.suchpfade("help").is_empty(), "nicht vorhandene Ordner fehlen");
        let (w, p) = r.datei("data/templates", "gemeinsam.txt").unwrap(); assert_eq!((w.herkunft().as_str(), fs::read_to_string(p).unwrap().as_str()), ("p1", "p1"));
        assert_eq!(r.datei("data/templates", "nur_p2.txt").unwrap().0.herkunft(), "p2");
        assert!(r.datei("data/templates", "gibt_es_nicht.txt").is_none());
        for schlecht in ["../x", "a/../b", "/etc/passwd", "", "a//b"] { assert!(r.datei("data/templates", schlecht).is_none(), "{schlecht}"); }
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn kaputte_dateien_und_eintraege_werden_gemeldet_und_uebersprungen() {
        let t = tmp("kaputt");
        schreibe(&t.join("prog/data/x/index.json"), r#"[{"id":"a"},{"name":"ohne id"},{"id":""}]"#);
        schreibe(&t.join("ext/p1/data/x/index.json"), "{ kein json");
        schreibe(&t.join("ext/p2/data/x/index.json"), r#"[{"id":"z"}]"#);
        let r = Registry::programm(Some(&t.join("prog/data"))).mit_paketordner(Quelle::Benutzer, &t.join("ext"));
        let l = r.json_liste("data/x", "index.json", None, "id");
        assert_eq!(l.eintraege.iter().map(|e| e.kennung.as_str()).collect::<Vec<_>>(), ["a", "z"], "gute Einträge bleiben");
        assert_eq!(l.probleme.len(), 3, "{:?}", l.probleme);
        assert!(l.probleme.iter().any(|p| p.herkunft == "p1") && l.probleme.iter().filter(|p| p.herkunft == "Programm").count() == 2);
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn manifeste_der_pakete_und_ihre_probleme() {
        let t = tmp("manif");
        schreibe(&t.join("ext/gut/extension.json"), r#"{"id":"gut","version":"1.0.0","name":"Gutes Paket","commands":[]}"#);
        schreibe(&t.join("ext/falsch/extension.json"), r#"{"id":"anders","version":"0.1.0"}"#);
        schreibe(&t.join("ext/ohneid/extension.json"), r#"{"version":"1"}"#);
        schreibe(&t.join("ext/kaputt/extension.json"), "{ nein");
        fs::create_dir_all(t.join("ext/leer")).unwrap();
        let r = Registry::programm(None).mit_paketordner(Quelle::Benutzer, &t.join("ext"));
        let (p, pr) = r.pakete();
        assert_eq!(p.iter().map(|x| (x.kennung.as_str(), x.version.as_str(), x.name.as_str())).collect::<Vec<_>>(), [("anders", "0.1.0", "anders"), ("gut", "1.0.0", "Gutes Paket")]);
        assert_eq!(pr.len(), 4, "{pr:?}");
        assert!(pr.iter().any(|x| x.text.contains("passt nicht zum Paketnamen")) && pr.iter().any(|x| x.text.contains("ohne „id“")) && pr.iter().filter(|x| x.herkunft == "leer" || x.herkunft == "kaputt").count() == 2);
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn fehlende_datei_ist_kein_problem() {
        let (t, r) = aufbau("fehlt");
        let l = r.json_liste("data/templates", "gibt_es_nicht.json", None, "id"); assert!(l.eintraege.is_empty() && l.probleme.is_empty() && l.konflikte.is_empty());
        fs::remove_dir_all(t).ok();
    }

    #[test]
    fn entwicklungsschalter_wirkt_nur_mit_dem_feature() {
        let t = tmp("dev"); schreibe(&t.join("ext/p1/data/x/index.json"), r#"[{"id":"q"}]"#);
        std::env::set_var("VC_EXTENSIONS_DEV", t.join("ext"));
        let r = Registry::programm(None).mit_entwicklungsordner();
        std::env::remove_var("VC_EXTENSIONS_DEV");
        assert_eq!(r.hat_erweiterungen(), cfg!(feature = "dev-extensions"));
        fs::remove_dir_all(t).ok();
    }
}
