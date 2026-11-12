// SPDX-License-Identifier: GPL-3.0-or-later
//! Bank-Presets: Registerwerte der Speicherumschaltung (C64 `$01`, C128 MMU, Atari PORTB) und die
//! daraus sichtbaren Speicherregionen. Daraus entsteht der Seitenplan (256 Seiten zu je 256 Byte)
//! für die Memory-Map-Ansicht.
use crate::parse_num;
use serde::{Deserialize, Deserializer};

/// Art des Speichers, der in einer Seite sichtbar ist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageKind {
    Ram,
    /// sonstiges ROM (z. B. Monitor, Selbsttest)
    Rom,
    BasicRom,
    KernalRom,
    CharRom,
    Io,
    /// Steckmodul / Funktions-ROM
    Module,
}

/// Seitenfarbe der Memory Map (Mockup 7). Aus dem Preset ergeben sich Zeropage, Stack, E/A, ROM, BASIC und frei;
/// Bildschirm, Sprites, Zeichensatz und Musik legt erst das Projekt fest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageCategory { Zeropage, Stack, Screen, Sprites, Charset, Music, Io, Rom, Basic, Free }

impl PageCategory {
    /// Vorgabe aus Art und Seitennummer (Seite 0 = Zeropage, Seite 1 = Stack, sonst frei bzw. nach Art)
    pub fn default_for(kind: PageKind, page: u8) -> PageCategory {
        match kind {
            PageKind::Ram => match page { 0 => PageCategory::Zeropage, 1 => PageCategory::Stack, _ => PageCategory::Free },
            PageKind::BasicRom => PageCategory::Basic,
            PageKind::Io => PageCategory::Io,
            PageKind::Rom | PageKind::KernalRom | PageKind::CharRom | PageKind::Module => PageCategory::Rom,
        }
    }
}

fn de_u16<'de, D: Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum N { S(String), I(u64) }
    let n = match N::deserialize(d)? {
        N::S(s) => parse_num(&s).ok_or_else(|| serde::de::Error::custom(format!("ungültige Zahl »{s}«")))?,
        N::I(i) => i as i64,
    };
    u16::try_from(n).map_err(|_| serde::de::Error::custom(format!("Wert {n} außerhalb 0..65535")))
}

fn de_u8<'de, D: Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    let n = de_u16(d)?;
    u8::try_from(n).map_err(|_| serde::de::Error::custom(format!("Wert {n} außerhalb 0..255")))
}

/// Ein Bit des Banking-Registers
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BankBit {
    pub bit: u8,
    pub name: String,
    /// Wirkung in Worten (auch für Tooltips)
    #[serde(default)]
    pub hint: String,
}

/// Das Register, das die Umschaltung steuert
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BankRegister {
    pub name: String,
    #[serde(deserialize_with = "de_u16")]
    pub addr: u16,
    #[serde(default)]
    pub bits: Vec<BankBit>,
}

/// Ein sichtbarer Bereich (Seitengrenzen: `start` und `end` einschließlich; jede berührte Seite zählt)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BankRegion {
    #[serde(deserialize_with = "de_u16")]
    pub start: u16,
    #[serde(deserialize_with = "de_u16")]
    pub end: u16,
    pub kind: PageKind,
    /// Überschreibt die Vorgabe aus [`PageCategory::default_for`]
    #[serde(default)]
    pub category: Option<PageCategory>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BankPreset {
    /// Kennung, z. B. `37` oder `os-an-basic-aus`
    pub id: String,
    pub name: String,
    /// Wert für das Register
    #[serde(deserialize_with = "de_u8")]
    pub value: u8,
    #[serde(default)]
    pub note: String,
    /// Nur für diese Modelle (leer = alle), z. B. `130xe`
    #[serde(default)]
    pub models: Vec<String>,
    /// Regionen, die über [`Banking::base`] gelegt werden (spätere gewinnen)
    #[serde(default)]
    pub regions: Vec<BankRegion>,
}

/// Bank-Umschaltung eines Targets
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Banking {
    /// Herkunft der Werte (Handbuch, Speicherplan)
    #[serde(default)]
    pub source: String,
    pub register: BankRegister,
    /// Kennung des Standard-Presets
    #[serde(default)]
    pub default: Option<String>,
    /// Immer gültige Grundbelegung (zuerst angewendet); Seiten ohne Eintrag gelten als RAM
    #[serde(default)]
    pub base: Vec<BankRegion>,
    pub presets: Vec<BankPreset>,
}

/// Art und Farbe einer Seite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub kind: PageKind,
    pub category: PageCategory,
}

impl Banking {
    pub fn preset(&self, id: &str) -> Option<&BankPreset> { self.presets.iter().find(|p| p.id == id) }
    /// Das Standard-Preset (`default`, sonst das erste)
    pub fn default_preset(&self) -> Option<&BankPreset> {
        self.default.as_deref().and_then(|d| self.preset(d)).or(self.presets.first())
    }
    /// Preset, dessen Wert gleich `value` ist
    pub fn preset_for_value(&self, value: u8) -> Option<&BankPreset> { self.presets.iter().find(|p| p.value == value) }
    /// Presets, die für das Modell gelten (Presets ohne `models` gelten für alle); ohne Modellangabe alle
    pub fn presets_for_model<'a>(&'a self, model: Option<&'a str>) -> impl Iterator<Item = &'a BankPreset> + 'a {
        self.presets.iter().filter(move |p| match model { Some(m) => p.models.is_empty() || p.models.iter().any(|x| x == m), None => true })
    }
    /// Seitenplan: für jede der 256 Seiten Art und Farbe
    pub fn page_plan(&self, preset: &BankPreset) -> [Page; 256] {
        let mut plan = [Page { kind: PageKind::Ram, category: PageCategory::Free }; 256];
        for (i, p) in plan.iter_mut().enumerate() { p.category = PageCategory::default_for(PageKind::Ram, i as u8); }
        for r in self.base.iter().chain(preset.regions.iter()) {
            for page in (r.start >> 8)..=(r.end >> 8) {
                let p = page as u8;
                plan[page as usize] = Page { kind: r.kind, category: r.category.unwrap_or_else(|| PageCategory::default_for(r.kind, p)) };
            }
        }
        plan
    }
    /// Prüft Bereiche und Kennungen; Fehlertext nennt die Stelle
    pub fn validate(&self) -> Result<(), String> {
        let check = |what: &str, r: &BankRegion| if r.start > r.end { Err(format!("{what}: Anfang ${:04X} hinter Ende ${:04X}", r.start, r.end)) } else { Ok(()) };
        for r in &self.base { check("base", r)?; }
        for (i, p) in self.presets.iter().enumerate() {
            if self.presets[..i].iter().any(|q| q.id == p.id) { return Err(format!("Preset-Kennung »{}« doppelt", p.id)); }
            for r in &p.regions { check(&format!("Preset {}", p.id), r)?; }
        }
        if let Some(d) = &self.default { if self.preset(d).is_none() { return Err(format!("Standard-Preset »{d}« fehlt")); } }
        for b in &self.register.bits { if b.bit > 7 { return Err(format!("Bit {} außerhalb 0..7", b.bit)); } }
        Ok(())
    }
}
