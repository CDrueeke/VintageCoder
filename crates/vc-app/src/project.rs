// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt = Ordner mit `.vcprj`: Dateien nach Art gruppiert.
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group { Asm, Basic, Song, Play, Sprite, Charset, Screen, Tiles, Palette, Text, Binary, Other }

impl Group {
    pub const ALL: [Group; 12] = [Group::Asm, Group::Basic, Group::Song, Group::Play, Group::Sprite, Group::Charset, Group::Screen, Group::Tiles, Group::Palette, Group::Text, Group::Binary, Group::Other];
    pub fn title(self) -> &'static str {
        match self { Group::Asm => "Assembler", Group::Basic => "BASIC", Group::Song => "Tracker", Group::Play => "PLAY-Tracker", Group::Sprite => "Sprites / PMG", Group::Charset => "Zeichensätze", Group::Screen => "Bildschirme", Group::Tiles => "Kacheln", Group::Palette => "Paletten", Group::Text => "Text / Doku", Group::Binary => "Binärdaten", Group::Other => "Weitere Dateien" }
    }
    /// Kurzbezeichnung für den Ordnerbaum
    pub fn tag(self) -> &'static str {
        match self { Group::Asm => "ASM", Group::Basic => "BAS", Group::Song => "TRK", Group::Play => "PLAY", Group::Sprite => "SPR", Group::Charset => "CHR", Group::Screen => "SCR", Group::Tiles => "MAP", Group::Palette => "PAL", Group::Text => "TXT", Group::Binary => "BIN", Group::Other => "" }
    }
    pub fn of_kind(k: &str) -> Group {
        match k { "song" => Group::Song, "play" => Group::Play, "sprite" | "pmg" => Group::Sprite, "charset" => Group::Charset, "screen" => Group::Screen, "tilemap" | "tileset" => Group::Tiles, "palette" => Group::Palette, _ => Group::Other }
    }
}

#[derive(Clone)]
pub struct Entry { pub rel: String, pub group: Group }

pub struct Project {
    pub root: PathBuf,
    pub file: Option<String>,
    pub name: String,
    pub systems: Vec<String>,
    pub entries: Vec<Entry>,
    /// Alle Ordner des Projekts (auch leere), relativ und mit `/`
    pub dirs: Vec<String>,
    /// Geladenes Modell der Projektdatei (Quellen, Startprogramm, Diskette)
    pub pf: Option<vc_ide::projfile::ProjectFile>,
}

impl Project {
    pub fn open(file: &Path) -> Result<Project, String> {
        let root = file.parent().ok_or("Projektordner fehlt")?.to_path_buf();
        Self::load(root, file.file_name().map(|n| n.to_string_lossy().into_owned()))
    }
    pub fn load(root: PathBuf, file: Option<String>) -> Result<Project, String> {
        let meta = file.as_ref().and_then(|f| std::fs::read_to_string(root.join(f)).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok());
        let name = meta.as_ref().and_then(|m| m["project"]["name"].as_str().map(String::from)).or_else(|| root.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_else(|| "Projekt".into());
        let systems = meta.as_ref().and_then(|m| m["targets"].as_array().map(|t| t.iter().filter_map(|x| x["system"].as_str().or(x["id"].as_str()).map(String::from)).collect())).unwrap_or_default();
        let (mut entries, mut dirs) = (vec![], vec![]); scan(&root, &root, 0, &mut entries, &mut dirs); dirs.sort();
        entries.sort_by(|a, b| (a.group as u8, a.rel.to_lowercase()).cmp(&(b.group as u8, b.rel.to_lowercase())));
        let pf = file.as_ref().and_then(|f| vc_ide::projfile::ProjectFile::load(&root.join(f)).ok());
        apply_assets(&mut entries, pf.as_ref());
        Ok(Project { root, file, name, systems, entries, dirs, pf })
    }
    /// Projektdatei neu einlesen (nach dem Speichern im Projekteditor)
    pub fn reload(&mut self) { if let Some(f) = &self.file { if let Ok(pf) = vc_ide::projfile::ProjectFile::load(&self.root.join(f)) { self.name = if pf.project.name.is_empty() { self.name.clone() } else { pf.project.name.clone() }; self.systems = pf.targets.iter().map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).collect(); self.pf = Some(pf); } } self.rescan(); }
    pub fn rescan(&mut self) { let (mut e, mut d) = (vec![], vec![]); scan(&self.root, &self.root, 0, &mut e, &mut d); d.sort(); self.dirs = d; apply_assets(&mut e, self.pf.as_ref()); e.sort_by(|a, b| (a.group as u8, a.rel.to_lowercase()).cmp(&(b.group as u8, b.rel.to_lowercase()))); self.entries = e; }
}

/// Kategorie aus `assets` der Projektdatei bestimmt die Art-Marke (Text, Binärdaten); Quelltext behält die Marke seiner Endung
fn apply_assets(entries: &mut [Entry], pf: Option<&vc_ide::projfile::ProjectFile>) {
    let Some(pf) = pf else { return };
    for e in entries.iter_mut() {
        let Some(a) = pf.asset(&e.rel) else { continue };
        e.group = match (a.category.as_str(), e.group) {
            ("text", Group::Other | Group::Text | Group::Binary) => Group::Text,
            ("binary" | "graphics" | "charset", Group::Other | Group::Text | Group::Binary) => Group::Binary,
            (_, g) => g,
        };
    }
}

/// Tiefste Ordnerebene, die der Projektbaum durchsucht
const MAX_DEPTH: usize = 16;

fn scan(root: &Path, dir: &Path, depth: usize, out: &mut Vec<Entry>, dirs: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
        let p = e.path();
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_dir { if !(depth == 0 && name == "build") && depth < MAX_DEPTH { dirs.push(vc_core::paths::rel_path(root, &p)); scan(root, &p, depth + 1, out, dirs); } continue; }
        let rel = if p.starts_with(root) { vc_core::paths::rel_path(root, &p) } else { name.clone() };
        let l = name.to_ascii_lowercase();
        let group = if vc_media::naming::is_data_file(&l) {
            let k = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|a| a["kind"].as_str().map(String::from)).unwrap_or_default();
            Group::of_kind(&k)
        } else if l.ends_with(".asm") || l.ends_with(".inc") || l.ends_with(".s") { Group::Asm } else if l.ends_with(".sbas") || l.ends_with(".bas") || l.ends_with(".lst") || l.ends_with(".tur") { Group::Basic } else { match vc_ide::importfiles::tree_hint(&l) { Some("binary") => Group::Binary, Some(_) => Group::Text, None => Group::Other } };
        out.push(Entry { rel, group });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kategorie_aus_assets_bestimmt_die_marke() {
        let root = std::env::temp_dir().join(format!("vc_proj_assets_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(root.join("data/font.xyz"), [0u8, 1]).unwrap();
        std::fs::write(root.join("data/tab.bin"), [0u8, 1]).unwrap();
        std::fs::write(root.join("notiz.txt"), "hi").unwrap();
        std::fs::write(root.join("x.vcprj"), r#"{"format":1,"project":{"name":"x"},"assets":[{"file":"data/font.xyz","category":"charset"},{"file":"notiz.txt","category":"binary"}]}"#).unwrap();
        let p = Project::load(root.clone(), Some("x.vcprj".into())).unwrap();
        let g = |r: &str| p.entries.iter().find(|e| e.rel == r).map(|e| e.group);
        assert_eq!(g("data/font.xyz"), Some(Group::Binary), "Eintrag in assets, Endung unbekannt");
        assert_eq!(g("data/tab.bin"), Some(Group::Binary), "Endung genügt");
        assert_eq!(g("notiz.txt"), Some(Group::Binary), "Kategorie der Projektdatei geht vor");
        assert_eq!((Group::Binary.tag(), Group::Text.tag()), ("BIN", "TXT"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn leere_und_tiefe_ordner_gehoeren_zum_baum() {
        let root = std::env::temp_dir().join(format!("vc_proj_dirs_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let deep = "a/b/c/d/e/f/g/h"; // 8 Ebenen: früher abgeschnitten (Grenze 6)
        std::fs::create_dir_all(root.join(deep)).unwrap(); std::fs::create_dir_all(root.join("leer")).unwrap(); std::fs::create_dir_all(root.join("build")).unwrap();
        std::fs::write(root.join(deep).join("x.asm"), "").unwrap();
        let p = Project::load(root.clone(), None).unwrap();
        assert!(p.dirs.contains(&"leer".to_string()) && p.dirs.contains(&deep.to_string()));
        assert!(p.entries.iter().any(|e| e.rel.ends_with("h/x.asm")));
        assert!(!p.dirs.contains(&"build".to_string()), "build/ bleibt draußen");
        let _ = std::fs::remove_dir_all(&root);
    }
}
