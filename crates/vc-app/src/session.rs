// SPDX-License-Identifier: GPL-3.0-or-later
//! Sitzungsdatei `session.json` neben `layout.json`: je Projekt die zuletzt offenen Dateien (nur Pfade) und die aktive Datei.
//! Sie gilt nur auf diesem Rechner und steht nicht in der `.vcprj`. Gelesen wird sie nur, wenn das Projekt
//! „Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen“ eingeschaltet hat.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjSession { pub open: Vec<String>, pub active: Option<String> }

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionFile { pub projects: BTreeMap<String, ProjSession> }

impl SessionFile {
    pub fn parse(t: &str) -> SessionFile { serde_json::from_str(t).unwrap_or_default() }
    pub fn to_json(&self) -> String { serde_json::to_string_pretty(self).unwrap_or_default() }
}

fn key(root: &Path) -> String { root.to_string_lossy().into_owned() }

pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("session.json")) }

pub fn load_from(path: &Path, root: &Path) -> Option<ProjSession> {
    SessionFile::parse(&std::fs::read_to_string(path).ok()?).projects.remove(&key(root))
}

/// Sitzung eines Projekts speichern; andere Projekte bleiben erhalten
pub fn save_to(path: &Path, root: &Path, s: &ProjSession) -> bool {
    let mut f = std::fs::read_to_string(path).map(|t| SessionFile::parse(&t)).unwrap_or_default();
    f.projects.insert(key(root), s.clone());
    if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
    std::fs::write(path, format!("{}\n", f.to_json())).is_ok()
}

/// Reihenfolge zum Wiederöffnen: fehlende und doppelte Dateien entfallen; die aktive Datei kommt zuletzt, damit sie vorn liegt.
pub fn plan_restore(s: &ProjSession, exists: impl Fn(&str) -> bool) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for r in &s.open { if exists(r) && !out.contains(r) && s.active.as_deref() != Some(r.as_str()) { out.push(r.clone()); } }
    if let Some(a) = &s.active { if exists(a) { out.push(a.clone()); } }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiederherstellen_ueberspringt_fehlendes_und_doppeltes_aktive_zuletzt() {
        let s = ProjSession { open: vec!["a.asm".into(), "weg.asm".into(), "b.asm".into(), "a.asm".into(), "c.sprite.json".into()], active: Some("b.asm".into()) };
        assert_eq!(plan_restore(&s, |f| f != "weg.asm"), vec!["a.asm", "c.sprite.json", "b.asm"]);
        let s = ProjSession { open: vec![], active: Some("weg.asm".into()) };
        assert!(plan_restore(&s, |_| false).is_empty());
    }

    #[test]
    fn speichern_und_laden_je_projekt() {
        let dir = std::env::temp_dir().join(format!("vc_session_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let f = dir.join("session.json");
        let (a, b) = (Path::new("/p/a"), Path::new("/p/b"));
        let sa = ProjSession { open: vec!["x.asm".into()], active: Some("x.asm".into()) };
        assert!(load_from(&f, a).is_none());
        assert!(save_to(&f, a, &sa));
        assert!(save_to(&f, b, &ProjSession::default()));
        assert_eq!(load_from(&f, a), Some(sa));
        assert_eq!(load_from(&f, b), Some(ProjSession::default()));
        std::fs::write(&f, "kaputt").unwrap();
        assert!(load_from(&f, a).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
