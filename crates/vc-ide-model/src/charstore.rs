// SPDX-License-Identifier: GPL-3.0-or-later
//! „Meine“ im Panel „Zeichen“: zuletzt benutzte Zeichen, Favoriten und eigene Sequenzen. Gespeichert wird lokal beim Anwender
//! (`chars.json` im Einstellungsordner), nie im Projekt.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const RECENT_MAX: usize = 12;

/// Ein Zeichen der Namenstabelle: Familie und Code
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key { pub atari: bool, pub code: u8 }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seq { pub name: String, pub atari: bool, pub codes: Vec<u8> }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)] pub recent: Vec<Key>,
    #[serde(default)] pub favorites: Vec<Key>,
    #[serde(default)] pub sequences: Vec<Seq>,
}

pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("chars.json")) }

impl Store {
    /// Aus der Datei lesen; fehlt sie oder ist sie kaputt, ist der Speicher leer
    pub fn load(path: &Path) -> Store { std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default() }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
        std::fs::write(path, serde_json::to_string_pretty(self).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }

    /// Als zuletzt benutzt vermerken (vorn, ohne Doppelte, höchstens zwölf)
    pub fn used(&mut self, k: Key) {
        self.recent.retain(|x| *x != k);
        self.recent.insert(0, k);
        self.recent.truncate(RECENT_MAX);
    }

    pub fn toggle_favorite(&mut self, k: Key) {
        if let Some(i) = self.favorites.iter().position(|x| *x == k) { self.favorites.remove(i); } else { self.favorites.push(k); }
    }

    /// Sequenz speichern; gleicher Name (je Familie) ersetzt die alte. Leere Namen oder Folgen werden abgelehnt.
    pub fn add_sequence(&mut self, name: &str, atari: bool, codes: Vec<u8>) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() { return Err("Name fehlt".into()); }
        if codes.is_empty() { return Err("Die Sequenz ist leer".into()); }
        self.sequences.retain(|s| !(s.atari == atari && s.name == name));
        self.sequences.push(Seq { name: name.to_string(), atari, codes });
        Ok(())
    }

    pub fn remove_sequence(&mut self, i: usize) { if i < self.sequences.len() { self.sequences.remove(i); } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: u8) -> Key { Key { atari: false, code } }

    #[test]
    fn recent_keeps_order_without_duplicates_and_a_limit() {
        let mut s = Store::default();
        for c in 0..20u8 { s.used(k(c)); }
        assert_eq!(s.recent.len(), RECENT_MAX);
        assert_eq!(s.recent[0], k(19));
        s.used(k(10));
        assert_eq!(s.recent[0], k(10));
        assert_eq!(s.recent.iter().filter(|x| **x == k(10)).count(), 1);
        s.used(Key { atari: true, code: 10 });
        assert_eq!(s.recent.len(), RECENT_MAX, "andere Familie ist ein anderes Zeichen");
    }

    #[test]
    fn favorites_toggle_and_sequences_replace_by_name() {
        let mut s = Store::default();
        s.toggle_favorite(k(0x93)); s.toggle_favorite(k(0x05));
        assert!(s.favorites.contains(&k(0x93)));
        s.toggle_favorite(k(0x93));
        assert_eq!(s.favorites, vec![k(0x05)]);
        s.add_sequence(" weiß auf schwarz ", false, vec![0x93, 0x05]).unwrap();
        s.add_sequence("weiß auf schwarz", false, vec![0x05]).unwrap();
        assert_eq!(s.sequences.len(), 1);
        assert_eq!(s.sequences[0].codes, vec![0x05]);
        s.add_sequence("weiß auf schwarz", true, vec![0x7D]).unwrap();
        assert_eq!(s.sequences.len(), 2, "gleicher Name in anderer Familie");
        assert!(s.add_sequence("  ", false, vec![1]).is_err());
        assert!(s.add_sequence("x", false, vec![]).is_err());
        s.remove_sequence(0);
        assert_eq!(s.sequences.len(), 1);
        s.remove_sequence(9);
    }

    #[test]
    fn saves_and_loads_and_survives_garbage() {
        let dir = std::env::temp_dir().join(format!("vc_chars_{}", std::process::id()));
        let f = dir.join("sub").join("chars.json");
        let mut s = Store::default();
        s.used(k(0x93)); s.toggle_favorite(k(0x1C)); s.add_sequence("neu", false, vec![0x93]).unwrap();
        s.save(&f).unwrap();
        assert_eq!(Store::load(&f), s);
        std::fs::write(&f, "kaputt {").unwrap();
        assert_eq!(Store::load(&f), Store::default());
        assert_eq!(Store::load(&dir.join("fehlt.json")), Store::default());
        // alte Dateien ohne einzelne Felder bleiben lesbar
        std::fs::write(&f, r#"{"favorites":[{"atari":false,"code":5}]}"#).unwrap();
        assert_eq!(Store::load(&f).favorites, vec![k(5)]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
