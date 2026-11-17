// SPDX-License-Identifier: GPL-3.0-or-later
//! Tabelle der Studios (Editoren für Grafik, Klang, Disketten, Speicher). **Eine** Stelle statt verstreuter `match`-Arme:
//! Kennung (wie in `release/registry.json`), Dateiart, Titel, Hilfethemen. Ob ein Studio im Release enthalten ist, entscheidet
//! `vc_ide::features` (Datei `data/release.json`; fehlt sie, ist alles an). Abgewählte Studios haben keine Menüpunkte, keine
//! Befehle der Befehlspalette, keine Reiter und kein Öffnen (die Datei öffnet dann als Text); Dateiformate und `vcc media`/`vcc import`
//! bleiben immer erhalten.
use crate::project::Group;
use vc_ide::features::{features, Features};

pub struct StudioDef {
    /// Kennung in der Registry (`STU-SPRITE` ↔ `sprite`)
    pub id: &'static str,
    pub title: &'static str,
    /// Gruppe der Dateien im Projektbaum; `None` bei Studios ohne eigene Dateiart (Disk-Studio, Memory Map)
    pub group: Option<Group>,
    /// Dateiarten (`kind` in den Media-Dateien), die das Studio bearbeitet
    pub kinds: &'static [&'static str],
    /// Hilfethemen, die an diesem Studio hängen (siehe `vc_ide::help::needs`)
    #[allow(dead_code)]   // gelesen vom Test `studios_registry_stimmt` (Abgleich mit der Registry)
    pub help: &'static [&'static str],
}

pub const STUDIOS: &[StudioDef] = &[
    StudioDef { id: "sprite", title: "Sprites / PMG", group: Some(Group::Sprite), kinds: &["sprite", "pmg"], help: &["sprite", "sprites"] },
    StudioDef { id: "char", title: "Zeichensätze", group: Some(Group::Charset), kinds: &["charset"], help: &["charset", "zeichensatz"] },
    StudioDef { id: "screen", title: "Bildschirme", group: Some(Group::Screen), kinds: &["screen"], help: &["screen", "bildschirm-kacheln"] },
    StudioDef { id: "tiles", title: "Kacheln", group: Some(Group::Tiles), kinds: &["tilemap", "tileset"], help: &["tiles", "bildschirm-kacheln"] },
    StudioDef { id: "palette", title: "Paletten", group: Some(Group::Palette), kinds: &["palette"], help: &["paletten"] },
    StudioDef { id: "tracker", title: "Tracker", group: Some(Group::Song), kinds: &["song"], help: &["tracker", "envelope"] },
    StudioDef { id: "play", title: "PLAY-Tracker", group: Some(Group::Play), kinds: &["play"], help: &["play", "play-tracker"] },
    StudioDef { id: "disk", title: "Disk-Studio", group: None, kinds: &[], help: &["disks", "disk-studio"] },
    StudioDef { id: "memmap", title: "Memory Map", group: None, kinds: &[], help: &["memmap"] },
    StudioDef { id: "extract", title: "Extraktor", group: None, kinds: &[], help: &[] },
];

#[cfg(test)]
pub fn def(id: &str) -> Option<&'static StudioDef> { STUDIOS.iter().find(|s| s.id == id) }
pub fn def_for_group(g: Group) -> Option<&'static StudioDef> { STUDIOS.iter().find(|s| s.group == Some(g)) }
pub fn def_for_kind(kind: &str) -> Option<&'static StudioDef> { STUDIOS.iter().find(|s| s.kinds.contains(&kind)) }

/// Studios mit eigener Dateiart („Medien-Studios“): alle, die ein eigenes Fenster bekommen
pub fn media() -> impl Iterator<Item = &'static StudioDef> { STUDIOS.iter().filter(|s| s.group.is_some()) }

pub fn enabled(id: &str) -> bool { features().studio(id) }
pub fn group_enabled_with(g: Group, f: &Features) -> bool { def_for_group(g).map_or(true, |d| f.studio(d.id)) }
pub fn group_enabled(g: Group) -> bool { group_enabled_with(g, &features()) }
/// Ist mindestens ein Medien-Studio (Sprite, Zeichensatz, …) im Release?
pub fn any_media_with(f: &Features) -> bool { media().any(|s| f.studio(s.id)) }
pub fn any_media() -> bool { any_media_with(&features()) }

/// Hinweis beim Öffnen einer Datei, deren Studio fehlt
pub fn missing_note(title: &str) -> String { format!("Dieses Release enthält das Studio „{title}“ nicht. Datei wird als Text geöffnet.") }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn studios_registry_stimmt() {
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../release/registry.json")).expect("registry.json");
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let mut reg: Vec<String> = v["features"].as_object().unwrap().values().filter(|f| f["kind"] == "studio").map(|f| f["id"].as_str().unwrap().to_string()).collect();
        let mut tab: Vec<String> = STUDIOS.iter().map(|s| s.id.to_string()).collect();
        reg.sort(); tab.sort();
        assert_eq!(tab, reg, "STUDIOS und release/registry.json führen verschiedene Studios");
        // Hilfethemen: Tabelle, Registry (`help`) und `vc_ide::help::needs` gleich
        for f in v["features"].as_object().unwrap().values().filter(|f| f["kind"] == "studio") {
            let id = f["id"].as_str().unwrap();
            let mut reg_help: Vec<&str> = f["help"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
            let mut tab_help: Vec<&str> = def(id).unwrap().help.to_vec();
            reg_help.sort(); tab_help.sort();
            assert_eq!(tab_help, reg_help, "Hilfethemen von Studio {id}");
        }
    }

    #[test]
    fn dateiarten_gehoeren_genau_einem_studio() {
        for k in ["sprite", "pmg", "charset", "screen", "tilemap", "tileset", "palette", "song", "play"] {
            let d = def_for_kind(k).unwrap_or_else(|| panic!("Art {k} ohne Studio"));
            assert_eq!(Group::of_kind(k), d.group.unwrap(), "{k}");
        }
        assert!(def_for_kind("unbekannt").is_none());
    }

    #[test]
    fn abgewaehlt_wirkt() {
        let f = Features::from_json(r#"{"format":1,"studios":["sprite"]}"#);
        assert!(group_enabled_with(Group::Sprite, &f) && !group_enabled_with(Group::Song, &f) && group_enabled_with(Group::Asm, &f));
        assert!(any_media_with(&f) && !any_media_with(&Features::from_json(r#"{"format":1,"studios":["disk"]}"#)));
    }
}
