// SPDX-License-Identifier: GPL-3.0-or-later
//! Chipregister eines Zielsystems (aus `data/chips/*.chip.json`) für Hover, Bit-Editor und Hex-Vorschläge im BASIC.
//! Die Daten sind dieselben wie im Chip-Panel des Debuggers; hier nur zum Nachschlagen einer Adresse.
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitInfo { pub name: String, pub lo: u8, pub width: u8, pub doc: String }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegInfo { pub chip: String, pub name: String, pub addr: u16, pub doc: String, /// `r`, `w` oder `rw` (leer, wenn die Daten nichts nennen)
    pub access: String,
    /// Atari: Schattenregister des Betriebssystems (`os.color4`), das dieses Hardwareregister jedes Bild überschreibt
    pub shadow: Option<String>, pub bits: Vec<BitInfo> }

impl RegInfo {
    /// `chip.name`, z. B. `vic2.border`
    pub fn label(&self) -> String { format!("{}.{}", self.chip, self.name) }
}

#[derive(Debug, Clone, Default)]
pub struct ChipMap {
    regs: Vec<RegInfo>,
    /// (Anfang, Ende, Spiegelgröße) je Chip; Register wiederholen sich im Fenster alle `mirror` Adressen
    windows: Vec<(u32, u32, u32)>,
}

fn addr(s: &str) -> Option<u32> { let s = s.trim(); if let Some(h) = s.strip_prefix('$') { u32::from_str_radix(h, 16).ok() } else { s.parse().ok() } }

impl ChipMap {
    /// Register der Chips des Zielsystems `target` (`c64`, `c128` …); ohne Zielsystem oder Datei: leer
    pub fn load(data: &Path, target: &str) -> ChipMap {
        let ids: Vec<String> = std::fs::read_to_string(data.join("targets").join(format!("{target}.target.json"))).ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| v["target"]["chips"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()))
            .unwrap_or_default();
        ChipMap::from_chips(data, &ids)
    }

    pub fn from_chips(data: &Path, ids: &[String]) -> ChipMap {
        let mut m = ChipMap::default();
        for id in ids {
            let Some(v) = std::fs::read_to_string(data.join("chips").join(format!("{id}.chip.json"))).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) else { continue };
            let (Some(start), Some(end)) = (v["window"]["start"].as_str().and_then(addr), v["window"]["end"].as_str().and_then(addr)) else { continue };
            let mirror = v["mirror"].as_str().and_then(addr).unwrap_or(0);
            m.windows.push((start, end, mirror));
            let name = id.as_str();
            for r in v["regs"].as_array().into_iter().flatten() {
                let Some(a) = r["addr"].as_str().and_then(addr).filter(|a| *a <= 0xFFFF) else { continue };
                let Some(rn) = r["name"].as_str() else { continue };
                let bits = r["bits"].as_array().map(|b| b.iter().filter_map(|b| Some(BitInfo { name: b["name"].as_str()?.into(), lo: b["bit"].as_u64()? as u8, width: b["width"].as_u64().unwrap_or(1).max(1) as u8, doc: b["doc"].as_str().unwrap_or("").into() })).collect()).unwrap_or_default();
                m.regs.push(RegInfo { chip: name.into(), name: rn.into(), addr: a as u16, doc: r["doc"].as_str().unwrap_or("").into(), access: r["access"].as_str().unwrap_or("").into(), shadow: r["shadow"].as_str().map(String::from), bits });
            }
        }
        m
    }

    /// Register nach `chip.name` (z. B. `vic2.ctrl1`)
    pub fn by_label(&self, label: &str) -> Option<&RegInfo> { self.regs.iter().find(|r| r.label() == label) }
    /// Fenster des Chips (Anfang) – Basisadresse für Variablen
    pub fn chip_base(&self, chip: &str) -> Option<u16> { self.regs.iter().filter(|r| r.chip == chip).map(|r| r.addr).min() }

    pub fn is_empty(&self) -> bool { self.regs.is_empty() }
    pub fn regs(&self) -> &[RegInfo] { &self.regs }

    /// Register an der Adresse (gespiegelte Adressen eingeschlossen)
    pub fn lookup(&self, a: u32) -> Option<&RegInfo> {
        if let Some(r) = self.regs.iter().find(|r| r.addr as u32 == a) { return Some(r); }
        let (start, end, mirror) = self.windows.iter().find(|w| a >= w.0 && a <= w.1 && w.2 > 0).map(|w| (w.0, w.1, w.2))?;
        let canon = start + (a - start) % mirror;
        self.regs.iter().find(|r| r.addr as u32 == canon && canon >= start && canon <= end)
    }

    /// Ist die Adresse ein bekanntes Register?
    pub fn known(&self, a: u32) -> bool { self.lookup(a).is_some() }

    /// Register, deren Adresse (Hex, ohne `$`) mit `prefix` beginnt (für die Eingabehilfe nach `$`)
    pub fn with_prefix(&self, prefix: &str, limit: usize) -> Vec<&RegInfo> {
        let p = prefix.to_ascii_uppercase();
        let mut v: Vec<&RegInfo> = self.regs.iter().filter(|r| format!("{:04X}", r.addr).starts_with(&p) || (p.len() <= 2 && format!("{:X}", r.addr).starts_with(&p) && r.addr < 0x100)).collect();
        v.sort_by_key(|r| r.addr);
        v.truncate(limit);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn c64_registers_with_mirrors_and_bits() {
        let m = ChipMap::load(&data(), "c64");
        assert!(!m.is_empty());
        let r = m.lookup(0xD020).unwrap();
        assert_eq!(r.label(), "vic2.border");
        assert!(m.known(0xD021) && m.known(0xD418) && m.known(0xDC00));
        assert!(!m.known(0x0400) && !m.known(0xC000));
        // VIC-II spiegelt alle 64 Adressen
        assert_eq!(m.lookup(0xD060).map(|r| r.addr), Some(0xD020));
        let ctrl = m.lookup(0xD011).unwrap();
        assert!(ctrl.bits.iter().any(|b| b.name == "YSCROLL" && b.lo == 0 && b.width == 3));
        assert!(m.with_prefix("D02", 20).iter().all(|r| r.addr >> 4 == 0xD02));
    }

    #[test]
    fn unknown_target_is_empty() { assert!(ChipMap::load(&data(), "gibtsnicht").is_empty()); }
}
