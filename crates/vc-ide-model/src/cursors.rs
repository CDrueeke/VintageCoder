// SPDX-License-Identifier: GPL-3.0-or-later
//! Cursorstellung je Datei und Projekt (`cursors.json` neben `session.json`): Beim Öffnen einer Datei steht der Cursor dort, wo er
//! zuletzt war. Die Datei gilt nur auf diesem Rechner und steht nicht im Projekt. Sie wird unabhängig davon gelesen, ob das Projekt
//! „Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen“ eingeschaltet hat. Gespeichert werden Zeile und Spalte (1-basiert,
//! wie `Doc::cursor`), nicht der Zeichenindex: Ändert sich die Datei außerhalb, bleibt die Stelle nahe an der alten.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Obergrenze der gemerkten Dateien je Projekt
pub const MAX_PER_PROJECT: usize = 500;

pub type Marks = BTreeMap<String, (usize, usize)>;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CursorFile { pub projects: BTreeMap<String, Marks> }

impl CursorFile {
    pub fn parse(t: &str) -> CursorFile { serde_json::from_str(t).unwrap_or_default() }
    pub fn to_json(&self) -> String { serde_json::to_string_pretty(self).unwrap_or_default() }
}

fn key(root: &Path) -> String { root.to_string_lossy().into_owned() }

pub fn file() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("cursors.json")) }

pub fn load_from(path: &Path, root: &Path) -> Marks {
    std::fs::read_to_string(path).ok().and_then(|t| CursorFile::parse(&t).projects.remove(&key(root))).unwrap_or_default()
}

/// Stellen eines Projekts speichern; andere Projekte bleiben erhalten. Fehlende Dateien entfallen, die Zahl ist begrenzt.
pub fn save_to(path: &Path, root: &Path, marks: &Marks) -> bool {
    let mut f = std::fs::read_to_string(path).map(|t| CursorFile::parse(&t)).unwrap_or_default();
    let mut m: Marks = marks.iter().filter(|(rel, _)| root.join(rel).is_file()).map(|(k, v)| (k.clone(), *v)).collect();
    while m.len() > MAX_PER_PROJECT { let Some(k) = m.keys().next_back().cloned() else { break }; m.remove(&k); }
    f.projects.insert(key(root), m);
    if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
    vc_core::atomar::schreiben(path, format!("{}\n", f.to_json())).is_ok()
}

/// Zeichenindex einer gemerkten Stelle (Zeile und Spalte 1-basiert) im aktuellen Text; zu große Werte rücken ans Zeilen- bzw. Textende.
pub fn char_index(text: &str, (line, col): (usize, usize)) -> usize {
    let (line, col) = (line.max(1) - 1, col.max(1) - 1);
    let mut idx = 0usize;
    let mut lines = text.split('\n');
    for _ in 0..line {
        match lines.next() { Some(l) => idx += l.chars().count() + 1, None => return text.chars().count() }
    }
    match lines.next() { Some(l) => idx + col.min(l.chars().count()), None => text.chars().count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_aus_zeile_und_spalte_klemmt_an_zeilen_und_textende() {
        let t = "abc\nde\n\nfghij";
        assert_eq!(char_index(t, (1, 1)), 0);
        assert_eq!(char_index(t, (1, 3)), 2);
        assert_eq!(char_index(t, (2, 2)), 5, "zweite Zeile, zweites Zeichen");
        assert_eq!(char_index(t, (2, 99)), 6, "Spalte hinter dem Zeilenende: Zeilenende");
        assert_eq!(char_index(t, (3, 1)), 7, "leere Zeile");
        assert_eq!(char_index(t, (4, 3)), 10);
        assert_eq!(char_index(t, (99, 1)), t.chars().count(), "Zeile gibt es nicht mehr: Textende");
        assert_eq!(char_index("", (1, 1)), 0);
        assert_eq!(char_index("ä\nö", (2, 2)), 3, "Zeichen statt Bytes");
        assert_eq!(char_index(t, (0, 0)), 0, "ungültige Null bleibt am Anfang");
    }

    #[test]
    fn speichern_laden_je_projekt_fehlendes_entfaellt_und_obergrenze() {
        let dir = std::env::temp_dir().join(format!("vc_cursors_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (pa, pb) = (dir.join("a"), dir.join("b"));
        std::fs::create_dir_all(&pa).unwrap(); std::fs::create_dir_all(&pb).unwrap();
        std::fs::write(pa.join("x.asm"), "x").unwrap(); std::fs::write(pa.join("y.asm"), "y").unwrap(); std::fs::write(pb.join("z.bas"), "z").unwrap();
        let f = dir.join("cursors.json");
        let mut a = Marks::new(); a.insert("x.asm".into(), (3, 7)); a.insert("y.asm".into(), (1, 1)); a.insert("weg.asm".into(), (9, 9));
        let mut b = Marks::new(); b.insert("z.bas".into(), (2, 2));
        assert!(load_from(&f, &pa).is_empty());
        assert!(save_to(&f, &pa, &a)); assert!(save_to(&f, &pb, &b));
        let la = load_from(&f, &pa);
        assert_eq!(la.get("x.asm"), Some(&(3, 7)));
        assert!(!la.contains_key("weg.asm"), "gelöschte Datei wird vergessen");
        assert_eq!(load_from(&f, &pb).get("z.bas"), Some(&(2, 2)), "anderes Projekt bleibt");
        std::fs::write(&f, "kaputt").unwrap();
        assert!(load_from(&f, &pa).is_empty(), "kaputte Datei: leer statt Absturz");
        let mut many = Marks::new();
        for i in 0..(MAX_PER_PROJECT + 20) { let n = format!("f{i:04}.asm"); std::fs::write(pa.join(&n), "x").unwrap(); many.insert(n, (1, 1)); }
        assert!(save_to(&f, &pa, &many));
        assert_eq!(load_from(&f, &pa).len(), MAX_PER_PROJECT);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
