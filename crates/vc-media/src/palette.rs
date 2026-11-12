// SPDX-License-Identifier: GPL-3.0-or-later
//! Paletten der Zielsysteme an einer Stelle: `data/palettes/index.json` ordnet jedem System eine Palette zu,
//! `palette_for` liefert sie mit Maske und Schrittweite der Farbwerte. Editoren, Import und Export fragen nur noch hier.
use serde_json::Value;
use std::path::Path;

pub type Rgb = [u8; 3];

/// Palette eines Zielsystems samt Abbildung der Farbwerte des Media auf Palettenzeilen
#[derive(Debug, Clone, PartialEq)]
pub struct TargetPalette {
    pub colors: Vec<Rgb>,
    /// gültige Bits eines Farbwerts (C64: 15, Atari/MEGA65: 255)
    pub mask: usize,
    /// Farbwerte je Zeile (Atari: 2, sonst 1)
    pub step: usize,
}

impl TargetPalette {
    /// RGB-Wert für den Farbwert `n` des Media
    pub fn rgb(&self, n: usize) -> Option<Rgb> { self.colors.get((n & self.mask) / self.step.max(1)).copied() }
    /// (Farbwert, RGB) je Zeile, z. B. für die Zuordnung beim Bildimport
    pub fn entries(&self) -> Vec<(u32, Rgb)> { self.colors.iter().enumerate().map(|(i, c)| ((i * self.step.max(1)) as u32, *c)).collect() }
}

pub fn parse_hex(s: &str) -> Option<Rgb> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 { return None; }
    Some([u8::from_str_radix(&s[0..2], 16).ok()?, u8::from_str_radix(&s[2..4], 16).ok()?, u8::from_str_radix(&s[4..6], 16).ok()?])
}

fn read_json(data: &Path, rel: &str) -> Result<Value, String> {
    let text = vc_core::read_static(&data.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{rel}: {e}"))
}

/// Inhalt einer Palettendatei (`palettes/<name>.json`) unverändert (für die Weitergabe an die Oberfläche)
pub fn file_json(data: &Path, name: &str) -> Value { read_json(data, &format!("palettes/{name}.json")).unwrap_or(Value::Null) }

/// Systeme, für die `index.json` eine Palette kennt
pub fn known_targets(data: &Path) -> Vec<String> {
    read_json(data, "palettes/index.json").ok().and_then(|v| v["targets"].as_object().map(|o| o.keys().cloned().collect())).unwrap_or_default()
}

/// Palette des Systems `target`; `variant` ist `pal` oder `ntsc` (nur für Systeme mit Videonorm-abhängiger Palette).
/// Unbekannte Systeme bekommen die Palette aus `default` (C64).
pub fn palette_for(data: &Path, target: &str, variant: &str) -> Result<TargetPalette, String> {
    let idx = read_json(data, "palettes/index.json")?;
    let entry = idx["targets"].get(target).or_else(|| idx["targets"].get(idx["default"].as_str().unwrap_or("c64"))).ok_or_else(|| format!("palettes/index.json: kein Eintrag für '{target}'"))?;
    let (mask, step) = (entry["mask"].as_u64().unwrap_or(15) as usize, entry["step"].as_u64().unwrap_or(1) as usize);
    if entry["builtin"].as_str() == Some("mega65") {
        return Ok(TargetPalette { colors: crate::default_palette().iter().filter_map(|h| parse_hex(h)).collect(), mask, step });
    }
    let name = entry["palette"].as_str().ok_or_else(|| format!("palettes/index.json: '{target}' ohne palette/builtin"))?;
    let file = read_json(data, &format!("palettes/{name}.json"))?;
    let key = entry["variants"][variant].as_str().or_else(|| entry["variants"]["pal"].as_str()).unwrap_or("colors");
    let list = file[key].as_array().ok_or_else(|| format!("palettes/{name}.json: '{key}' fehlt"))?;
    let colors = list.iter().map(|c| parse_hex(c.as_str().unwrap_or("")).ok_or_else(|| format!("palettes/{name}.json: ungültige Farbe")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TargetPalette { colors, mask, step })
}
