// SPDX-License-Identifier: GPL-3.0-or-later
//! Tabelle bekannter ROM-Abbilder (`data/roms/known.json`): nur Größe und Prüfsummen, keine ROM-Daten.
//! Ein Inhalt kann in mehreren Systemen vorkommen (`uses`), z. B. der C64-KERNAL im C128-Ordner.
use serde::{Deserialize, Serialize};

/// Verwendung eines Abbilds: System + Rolle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Use { pub system: String, pub role: String }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnownRom {
    pub id: String,
    pub label: String,
    pub size: usize,
    /// CRC-32, 8 Hexstellen in Großbuchstaben
    pub crc32: String,
    /// SHA-1, 40 Hexstellen in Kleinbuchstaben
    pub sha1: String,
    pub uses: Vec<Use>,
    /// Standardrevision für ihre Rolle (gewinnt bei mehreren Treffern)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default: bool,
    /// freie Ersatz-ROM (Open-ROMs, Altirra …), kein Original
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub free: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Zeichensätze im Abbild, die von den Standardorten der Rolle abweichen: Name → `{offset, size, profile, label}`; leer = Standardorte (siehe `systems::default_sets`)
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub sets: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownTable {
    pub format: u32,
    pub roms: Vec<KnownRom>,
}

/// Mitgelieferte Tabelle (zur Übersetzungszeit eingebunden; `data/roms/known.json` zur Laufzeit hat Vorrang, siehe [`KnownTable::load`])
const BUILTIN: &str = include_str!("../../../data/roms/known.json");

impl KnownTable {
    pub fn builtin() -> KnownTable { Self::parse(BUILTIN).expect("data/roms/known.json ist gültig") }

    pub fn parse(text: &str) -> Result<KnownTable, String> {
        let t: KnownTable = serde_json::from_str(text).map_err(|e| format!("known.json: {e}"))?;
        if t.format != 1 { return Err(format!("known.json: Format {} wird nicht unterstützt", t.format)); }
        if let Some(k) = t.roms.iter().find(|k| k.uses.is_empty()) { return Err(format!("known.json: {} ohne \"uses\"", k.id)); }
        Ok(t)
    }

    /// Tabelle aus dem Datenordner (`<data>/roms/known.json`), sonst die eingebaute
    pub fn load(data_dir: Option<&std::path::Path>) -> KnownTable {
        data_dir.and_then(|d| std::fs::read_to_string(d.join("roms/known.json")).ok()).and_then(|s| Self::parse(&s).ok()).unwrap_or_else(Self::builtin)
    }

    pub fn find(&self, size: usize, crc32: u32, sha1: &str) -> Option<&KnownRom> {
        let crc = format!("{crc32:08X}");
        self.roms.iter().find(|k| k.size == size && k.crc32.eq_ignore_ascii_case(&crc) && k.sha1.eq_ignore_ascii_case(sha1))
    }

    /// Eintrag hinzufügen oder (gleiche Prüfsumme) um weitere Verwendungen ergänzen
    pub fn merge(&mut self, new: KnownRom) {
        if let Some(old) = self.roms.iter_mut().find(|k| k.sha1 == new.sha1 && k.size == new.size) {
            for u in new.uses { if !old.uses.contains(&u) { old.uses.push(u); } }
            if old.sets.is_empty() { old.sets = new.sets; }
        } else { self.roms.push(new); }
    }

    /// Ausgabe: ein Eintrag je Zeile (gut lesbar, gut vergleichbar), sortiert nach System, Rolle, Kennung
    pub fn to_json(&self) -> String {
        let mut roms = self.roms.clone();
        roms.sort_by(|a, b| (&a.uses[0].system, &a.uses[0].role, &a.id).cmp(&(&b.uses[0].system, &b.uses[0].role, &b.id)));
        let mut s = String::from("{ \"format\": 1,\n  \"roms\": [\n");
        for (i, r) in roms.iter().enumerate() {
            s.push_str("    ");
            s.push_str(&serde_json::to_string(r).unwrap());
            s.push_str(if i + 1 < roms.len() { ",\n" } else { "\n" });
        }
        s.push_str("  ] }\n");
        s
    }
}

impl KnownRom {
    /// Zeichensätze dieses Abbilds für eine Verwendung: Einträge aus `sets`, sonst die Standardorte der Rolle
    pub fn char_sets(&self, system: &str, role: &str) -> Vec<crate::systems::SetDef> {
        let own: Vec<_> = self.sets.iter().filter_map(|(name, v)| Some(crate::systems::SetDef {
            name: name.clone(), label: v["label"].as_str().unwrap_or(name).into(), offset: v["offset"].as_u64()? as usize, size: v["size"].as_u64()? as usize, profile: v["profile"].as_str()?.into(),
        })).collect();
        if own.is_empty() { crate::systems::default_sets(system, role) } else { own }
    }

    /// Verwendung für ein System (bei Hinweis) oder die erste
    pub fn use_for(&self, system_hint: Option<&str>) -> &Use {
        system_hint.and_then(|h| self.uses.iter().find(|u| u.system == h)).unwrap_or(&self.uses[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_und_zusammenfuehren() {
        let mut t = KnownTable { format: 1, roms: vec![] };
        let k = KnownRom { id: "a".into(), label: "A".into(), size: 8192, crc32: "DBE3E7C7".into(), sha1: "x".repeat(40), uses: vec![Use { system: "c64".into(), role: "kernal".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        t.merge(k.clone());
        t.merge(KnownRom { uses: vec![Use { system: "c128".into(), role: "kernal64".into() }], ..k });
        assert_eq!(t.roms.len(), 1);
        assert_eq!(t.roms[0].uses.len(), 2);
        let back = KnownTable::parse(&t.to_json()).unwrap();
        assert_eq!(back.roms, t.roms);
        assert_eq!(back.roms[0].use_for(Some("c128")).role, "kernal64");
        assert_eq!(back.roms[0].use_for(Some("vic20")).role, "kernal");
    }

    #[test]
    fn fremdes_format_wird_abgelehnt() { assert!(KnownTable::parse("{\"format\": 9, \"roms\": []}").is_err()); }
}
