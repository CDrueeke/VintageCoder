// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektstruktur-Vorlagen (`data/templates/structures.json`): Ordnerliste und `layout` (Kategorie → Zielordner), die beim Anlegen
//! eines Projekts (Wizard, `vcc new --structure`) angewendet werden. Eine Struktur ergänzt die Dateivorlage; sie ist nur ein Vorschlag.
//! In `dirs` steht `{system}` für jedes gewählte Zielsystem.
use crate::importfiles::valid_dir;
use crate::projfile::ProjectFile;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Structure {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dirs: Vec<String>,
    pub layout: BTreeMap<String, String>,
    /// Rolle je Ordner (`dir` → Rolle, siehe `projfile::ROLES`); `{system}` wie in `dirs`
    pub roles: BTreeMap<String, String>,
    /// Eigene Vorlage des Benutzers (aus der Konfiguration), nicht mitgeliefert
    pub user: bool,
}

/// Alle mitgelieferten Strukturen aus `<data>/templates/structures.json` (Vorgabe zuerst: `einfach`)
pub fn list(data: &Path) -> Vec<Structure> { load_file(&data.join("templates/structures.json"), false) }

/// Strukturen aus einer Datei im Format von `structures.json`
pub fn load_file(path: &Path, user: bool) -> Vec<Structure> {
    let Some(v) = std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else { return vec![] };
    v["structures"].as_array().into_iter().flatten().filter_map(parse).map(|mut s| { s.user = user; s }).collect()
}

/// Datei mit den eigenen Vorlagen: `VC_STRUCTURES`, sonst `structures.json` im Konfigurationsordner (neben `settings.json`)
pub fn user_file() -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("VC_STRUCTURES") { return Some(p.into()); }
    Some(vc_core::userdirs::config_dir()?.join("structures.json"))
}

/// Mitgelieferte und eigene Vorlagen (eigene mit gleicher Kennung ersetzen die mitgelieferte nicht, sie stehen dahinter)
pub fn list_all(data: &Path) -> Vec<Structure> {
    let mut v = list(data);
    if let Some(f) = user_file() { v.extend(load_file(&f, true)); }
    v
}

fn to_json(s: &Structure) -> Value {
    serde_json::json!({ "id": s.id, "name": s.name, "description": s.description, "dirs": s.dirs, "layout": s.layout, "roles": s.roles })
}

/// Trägt eine eigene Vorlage in `file` ein (gleiche Kennung wird ersetzt)
pub fn save_user(file: &Path, s: &Structure) -> Result<(), String> {
    let mut all = load_file(file, true);
    match all.iter_mut().find(|x| x.id == s.id) { Some(x) => *x = s.clone(), None => all.push(s.clone()) }
    write_all(file, &all)
}

/// Entfernt eine eigene Vorlage; liefert, ob es sie gab
pub fn delete_user(file: &Path, id: &str) -> Result<bool, String> {
    let mut all = load_file(file, true);
    let n = all.len();
    all.retain(|x| x.id != id);
    if all.len() == n { return Ok(false); }
    write_all(file, &all)?;
    Ok(true)
}

fn write_all(file: &Path, all: &[Structure]) -> Result<(), String> {
    if let Some(p) = file.parent() { std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?; }
    let v = serde_json::json!({ "structures": all.iter().map(to_json).collect::<Vec<_>>() });
    std::fs::write(file, serde_json::to_string_pretty(&v).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{}: {e}", file.display()))
}

/// Kennung aus einem Namen: Kleinbuchstaben, Ziffern, `-`
pub fn slug(name: &str) -> String {
    let s: String = name.trim().to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if s.is_empty() { "vorlage".into() } else { format!("eigene-{s}") }
}

/// Alle Ordner eines Projekts (relativ, mit `/`; ohne versteckte, `build/` im Projektordner, `target`, `node_modules`)
pub fn list_dirs(root: &Path) -> Vec<String> {
    fn go(root: &Path, dir: &Path, depth: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" || name == "node_modules" || !e.file_type().map_or(false, |t| t.is_dir()) { continue; }
            if depth == 0 && name == "build" { continue; }
            if let Ok(r) = e.path().strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")); }
            if depth < 16 { go(root, &e.path(), depth + 1, out); }
        }
    }
    let mut v = vec![]; go(root, root, 0, &mut v); v.sort(); v
}

/// Eine eigene Vorlage aus einem bestehenden Projekt: Ordner, Zielordner je Kategorie und Rollen (ohne Targets)
pub fn from_project(root: &Path, pf: &ProjectFile, name: &str) -> Structure {
    Structure {
        id: slug(name), name: name.trim().to_string(), description: "Eigene Vorlage".into(),
        dirs: list_dirs(root), layout: pf.layout.clone(),
        roles: pf.folders.iter().filter(|f| !f.role.is_empty() && f.targets.is_empty()).map(|f| (f.dir.clone(), f.role.clone())).collect(),
        user: true,
    }
}

/// Zeile einer Baumvorschau
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewLine { pub depth: usize, pub name: String, /** Ordner? */ pub dir: bool, pub note: String, /** gibt es noch nicht */ pub new: bool }

/// Baum aus Pfaden (Ordner enden auf `/`) mit Anmerkungen je Ordner (`dir` → Text), sortiert, Ordner vor Dateien
pub fn preview_tree(paths: &[String], notes: &BTreeMap<String, String>, existing: &dyn Fn(&str) -> bool) -> Vec<PreviewLine> {
    #[derive(Default)] struct N { dirs: BTreeMap<String, N>, files: Vec<String> }
    let mut root = N::default();
    for p in paths {
        let is_dir = p.ends_with('/');
        let parts: Vec<&str> = p.trim_end_matches('/').split('/').filter(|x| !x.is_empty()).collect();
        let Some((last, dirs)) = parts.split_last() else { continue };
        let mut n = &mut root;
        for d in dirs { n = n.dirs.entry(d.to_string()).or_default(); }
        if is_dir { n.dirs.entry(last.to_string()).or_default(); } else { n.files.push(last.to_string()); }
    }
    fn walk(n: &N, path: &str, depth: usize, notes: &BTreeMap<String, String>, ex: &dyn Fn(&str) -> bool, out: &mut Vec<PreviewLine>) {
        for (name, sub) in &n.dirs {
            let p = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
            out.push(PreviewLine { depth, name: name.clone(), dir: true, note: notes.get(&p).cloned().unwrap_or_default(), new: !ex(&p) });
            walk(sub, &p, depth + 1, notes, ex, out);
        }
        for f in &n.files { out.push(PreviewLine { depth, name: f.clone(), dir: false, note: String::new(), new: false }); }
    }
    let mut out = vec![]; walk(&root, "", 0, notes, existing, &mut out); out
}

/// Vorschau der Projektstruktur: vorhandene Ordner, Zielordner aus `layout`/`rules` und Ordner mit Rolle; Rolle und Kategorien als Anmerkung
pub fn preview_project(root: &Path, pf: &ProjectFile) -> Vec<PreviewLine> {
    let mut paths: Vec<String> = list_dirs(root).into_iter().map(|d| format!("{d}/")).collect();
    let mut notes: BTreeMap<String, String> = BTreeMap::new();
    let mut add_note = |d: &str, t: String| { let e = notes.entry(d.to_string()).or_default(); if !e.is_empty() { e.push_str(", "); } e.push_str(&t); };
    for f in &pf.folders { if !f.dir.is_empty() { paths.push(format!("{}/", f.dir)); if let Some(l) = crate::projfile::role_label(&f.role) { add_note(&f.dir, l.to_string()); } } }
    for (k, d) in &pf.layout {
        if d.is_empty() || !valid_dir(d) { continue; }
        paths.push(format!("{d}/"));
        if let Some(c) = crate::importfiles::Category::from_id(k) { add_note(d.trim_matches('/'), format!("← {}", c.label())); }
    }
    for r in &pf.rules { if !r.dir.is_empty() && valid_dir(&r.dir) { paths.push(format!("{}/", r.dir)); add_note(r.dir.trim_matches('/'), format!("← {}", r.pattern)); } }
    paths.sort(); paths.dedup();
    preview_tree(&paths, &notes, &|d| root.join(d).is_dir())
}

fn parse(v: &Value) -> Option<Structure> {
    let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
    let id = s("id");
    if id.is_empty() { return None; }
    let dirs = v["dirs"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let layout = v["layout"].as_object().map(|o| o.iter().filter_map(|(k, x)| x.as_str().map(|d| (k.clone(), d.to_string()))).collect()).unwrap_or_default();
    let roles = v["roles"].as_object().map(|o| o.iter().filter_map(|(k, x)| x.as_str().map(|d| (k.clone(), d.to_string()))).collect()).unwrap_or_default();
    Some(Structure { id, name: s("name"), description: s("description"), dirs, layout, roles, user: false })
}

impl Structure {
    /// Ordner mit eingesetzten Zielsystemen, ohne Doppelte, nur gültige relative Pfade
    pub fn dirs_for(&self, systems: &[String]) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        for d in &self.dirs {
            let expanded: Vec<String> = if d.contains("{system}") { systems.iter().map(|s| d.replace("{system}", s)).collect() } else { vec![d.clone()] };
            for e in expanded { if !valid_dir(&e) { continue; } let e = e.trim_end_matches('/').to_string(); if !e.is_empty() && !out.contains(&e) { out.push(e); } }
        }
        out
    }

    /// Ordner anlegen (auf Wunsch mit `.gitkeep`, damit Git leere Ordner behält) und `layout` in die Projektdatei eintragen.
    /// Vorhandene Ordner und Dateien bleiben unberührt. Liefert die neu angelegten Ordner.
    pub fn apply(&self, root: &Path, pf: &mut ProjectFile, systems: &[String], gitkeep: bool) -> Result<Vec<String>, String> {
        let mut made = vec![];
        for d in self.dirs_for(systems) {
            let p = root.join(&d);
            let new = !p.exists();
            std::fs::create_dir_all(&p).map_err(|e| format!("{d}: {e}"))?;
            if new {
                if gitkeep { std::fs::write(p.join(".gitkeep"), "").map_err(|e| format!("{d}: {e}"))?; }
                made.push(d);
            }
        }
        pf.folders = self.roles.iter().flat_map(|(d, r)| {
            // `{system}`-Ordner gehören zu genau diesem Target
            let dirs: Vec<(String, Vec<String>)> = if d.contains("{system}") { systems.iter().map(|s| (d.replace("{system}", s), vec![s.clone()])).collect() } else { vec![(d.clone(), vec![])] };
            dirs.into_iter().filter(|(d, _)| valid_dir(d) && !d.is_empty()).map(|(d, t)| crate::projfile::FolderDef { dir: d, role: r.clone(), targets: t }).collect::<Vec<_>>()
        }).collect();
        pf.layout = self.layout.iter().filter(|(_, d)| valid_dir(d)).map(|(k, d)| (k.clone(), d.trim_matches('/').to_string())).collect();
        Ok(made)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> std::path::PathBuf { crate::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data/") }

    #[test]
    fn mitgelieferte_strukturen_sind_gueltig() {
        let l = list(&data());
        let ids: Vec<&str> = l.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["einfach", "standard", "gross", "mehrsystem", "bibliothek", "spiel"]);
        assert!(l[0].dirs.is_empty() && l[0].layout.is_empty(), "Vorgabe ändert nichts");
        for s in &l {
            assert!(!s.name.is_empty() && !s.description.is_empty(), "{}", s.id);
            for d in s.dirs.iter().chain(s.layout.values()) { assert!(valid_dir(&d.replace("{system}", "c64")), "{}: {d}", s.id); }
            for r in s.roles.values() { assert!(crate::projfile::role_label(r).is_some(), "{}: Rolle {r}", s.id); }
            for k in s.layout.keys() { assert!(crate::importfiles::Category::from_id(k).is_some(), "{}: Kategorie {k}", s.id); }
        }
    }

    #[test]
    fn anwenden_legt_ordner_an_und_traegt_layout_ein() {
        let root = std::env::temp_dir().join(format!("vc_struct_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(root.join("doc")).unwrap();
        let s = list(&data()).into_iter().find(|s| s.id == "mehrsystem").unwrap();
        let sys = vec!["c64".to_string(), "atari8".to_string()];
        assert_eq!(s.dirs_for(&sys), ["src/common", "src/c64", "src/atari8", "res/common", "res/c64", "res/atari8", "doc", "tests"]);
        let mut pf = ProjectFile::default();
        let made = s.apply(&root, &mut pf, &sys, true).unwrap();
        assert!(made.contains(&"res/atari8".to_string()) && !made.contains(&"doc".to_string()), "vorhandene Ordner zählen nicht");
        assert!(root.join("src/c64/.gitkeep").is_file() && !root.join("doc/.gitkeep").exists());
        assert_eq!(pf.layout.get("binary").map(String::as_str), Some("res/common"));
        assert_eq!(pf.role_of("res/common"), Some("resources"));
        assert_eq!(pf.targets_of("src/c64"), ["c64"]);
        assert_eq!(pf.targets_of("src/atari8"), ["atari8"]);
        assert!(pf.targets_of("src/common").is_empty());
        assert!(pf.in_target_folder("src/c64/main.asm") && !pf.in_target_folder("src/common/x.asm"));
        assert_eq!(pf.role_of("doc"), Some("docs"));
        assert!(pf.validate(&root).iter().any(|m| m.contains("Target 'c64'")), "Targets noch nicht in der Projektdatei");
        pf.targets = sys.iter().map(|s| crate::projfile::TargetDef { id: s.clone(), system: s.clone(), ..Default::default() }).collect();
        assert!(pf.validate(&root).iter().all(|m| !m.contains("folders")));
        // zweites Anwenden ändert nichts an den Ordnern
        assert!(s.apply(&root, &mut pf, &sys, false).unwrap().is_empty());
        // Round-Trip der Projektdatei
        let back: ProjectFile = serde_json::from_str(&serde_json::to_string(&pf).unwrap()).unwrap();
        assert_eq!(back.layout, pf.layout);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ungueltige_ordner_werden_ignoriert() {
        let s = Structure { dirs: vec!["../raus".into(), "/abs".into(), "ok/{system}".into(), "".into()], ..Default::default() };
        assert_eq!(s.dirs_for(&["c64".to_string()]), ["ok/c64"]);
    }

    #[test]
    fn eigene_vorlagen_speichern_laden_loeschen_und_aus_projekt_bauen() {
        let root = std::env::temp_dir().join(format!("vc_struct_user_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("res/bin")).unwrap(); std::fs::create_dir_all(root.join("src/c64")).unwrap(); std::fs::create_dir_all(root.join("build")).unwrap(); std::fs::create_dir_all(root.join(".git")).unwrap();
        let mut pf = ProjectFile::default();
        pf.layout.insert("binary".into(), "res/bin".into());
        pf.set_role("res", Some("resources")); pf.set_folder_target("src/c64", Some("c64"));
        let s = from_project(&root, &pf, "Mein Spiel!");
        assert_eq!(s.id, "eigene-mein-spiel");
        assert_eq!(s.dirs, ["res", "res/bin", "src", "src/c64"], "ohne build und versteckte");
        assert_eq!(s.roles.len(), 1, "Ordner mit Target werden nicht übernommen");
        let file = root.join("cfg/structures.json");
        save_user(&file, &s).unwrap();
        save_user(&file, &Structure { description: "neu".into(), ..s.clone() }).unwrap();
        let l = load_file(&file, true);
        assert_eq!((l.len(), l[0].description.as_str(), l[0].user), (1, "neu", true));
        assert_eq!(l[0].layout.get("binary").map(String::as_str), Some("res/bin"));
        assert!(delete_user(&file, "eigene-mein-spiel").unwrap() && !delete_user(&file, "eigene-mein-spiel").unwrap() && load_file(&file, true).is_empty());
        assert_eq!(slug("   "), "vorlage");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn baumvorschau_sortiert_und_markiert_neue_ordner() {
        let paths: Vec<String> = ["res/bin/", "src/", "res/", "res/fonts/", "a.txt"].iter().map(|s| s.to_string()).collect();
        let mut notes = BTreeMap::new(); notes.insert("res".to_string(), "Ressourcen".to_string());
        let v = preview_tree(&paths, &notes, &|d| d == "src");
        let got: Vec<(usize, &str, bool, bool)> = v.iter().map(|l| (l.depth, l.name.as_str(), l.dir, l.new)).collect();
        assert_eq!(got, vec![(0, "res", true, true), (1, "bin", true, true), (1, "fonts", true, true), (0, "src", true, false), (0, "a.txt", false, false)]);
        assert_eq!(v[0].note, "Ressourcen");
        let root = std::env::temp_dir().join(format!("vc_prev_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(root.join("src")).unwrap();
        let mut pf = ProjectFile::default(); pf.layout.insert("binary".into(), "res/bin".into());
        pf.rules.push(crate::projfile::Rule { pattern: "*.sid".into(), category: "binary".into(), dir: "res/music".into() });
        let v = preview_project(&root, &pf);
        assert!(v.iter().any(|l| l.name == "bin" && l.note.contains("Binärdaten") && l.new) && v.iter().any(|l| l.name == "music" && l.note.contains("*.sid")) && v.iter().any(|l| l.name == "src" && !l.new));
        let _ = std::fs::remove_dir_all(&root);
    }
}
