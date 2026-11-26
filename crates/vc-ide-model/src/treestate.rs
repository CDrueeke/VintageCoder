// SPDX-License-Identifier: GPL-3.0-or-later
//! Zugeklappte Ordner des Dateibaums je Projekt (`tree.json` neben `cursors.json`). Gilt nur auf diesem Rechner, steht nicht im Projekt.
//! Gemerkt werden die zugeklappten Ordner (alles andere ist aufgeklappt); Ordner, die es nicht mehr gibt, entfallen beim Speichern.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Obergrenze der gemerkten Ordner je Projekt
pub const MAX_PER_PROJECT: usize = 500;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TreeFile { pub projects: BTreeMap<String, BTreeSet<String>> }

impl TreeFile {
    pub fn parse(t: &str) -> TreeFile { serde_json::from_str(t).unwrap_or_default() }
    pub fn to_json(&self) -> String { serde_json::to_string_pretty(self).unwrap_or_default() }
}

fn key(root: &Path) -> String { root.to_string_lossy().into_owned() }

pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("tree.json")) }

pub fn load_from(path: &Path, root: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(path).ok().and_then(|t| TreeFile::parse(&t).projects.remove(&key(root))).unwrap_or_default()
}

/// Zugeklappte Ordner eines Projekts speichern; andere Projekte bleiben erhalten
pub fn save_to(path: &Path, root: &Path, zu: &BTreeSet<String>) -> bool {
    let mut f = std::fs::read_to_string(path).map(|t| TreeFile::parse(&t)).unwrap_or_default();
    let mut m: BTreeSet<String> = zu.iter().filter(|d| root.join(d).is_dir()).cloned().collect();
    while m.len() > MAX_PER_PROJECT { let Some(k) = m.iter().next_back().cloned() else { break }; m.remove(&k); }
    f.projects.insert(key(root), m);
    if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
    vc_core::atomar::schreiben(path, format!("{}\n", f.to_json())).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zugeklappte_ordner_je_projekt_merken_und_andere_projekte_behalten() {
        let t = std::env::temp_dir().join(format!("vc_treestate_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        let (a, b) = (t.join("a"), t.join("b")); for d in [a.join("lib"), a.join("gfx"), b.join("x")] { std::fs::create_dir_all(d).unwrap() }
        let datei = t.join("conf/tree.json");
        let za: BTreeSet<String> = ["lib", "weg"].iter().map(|s| s.to_string()).collect();
        assert!(save_to(&datei, &a, &za)); assert!(save_to(&datei, &b, &["x".to_string()].into_iter().collect()));
        assert_eq!(load_from(&datei, &a), ["lib".to_string()].into_iter().collect(), "nicht vorhandener Ordner entfällt");
        assert_eq!(load_from(&datei, &b).len(), 1); assert!(load_from(&datei, &t.join("c")).is_empty());
        assert!(load_from(&t.join("fehlt.json"), &a).is_empty());
        let _ = std::fs::remove_dir_all(&t);
    }
}
