// SPDX-License-Identifier: GPL-3.0-or-later
//! Modell des Paletten-Studios (MEGA65: 256 Farben, 8 Bit je Kanal), ohne Oberfläche.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PaletteDoc { pub rel: String, abs: PathBuf, pub media: Value, pub sel: usize, undo: Vec<String>, redo: Vec<String>, saved: String, coalesce: Option<usize>, pub status: String }

/// `#rrggbb` (mit oder ohne `#`) als Farbwert
pub fn hex_zu_rgb(s: &str) -> Option<[u8; 3]> {
    let t = s.trim().trim_start_matches('#');
    (t.len() == 6).then(|| u32::from_str_radix(t, 16).ok()).flatten().map(|v| [(v >> 16) as u8, (v >> 8) as u8, v as u8])
}
pub fn rgb_zu_hex(c: [u8; 3]) -> String { format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]) }

impl PaletteDoc {
    pub fn open(root: &Path, rel: &str) -> Result<PaletteDoc, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| vc_i18n::t!("palettedoc-rel-e", rel = &rel, e = e.to_string()))?).map_err(|e| vc_i18n::t!("palettedoc-rel-e", rel = &rel, e = e.to_string()))?;
        if media["kind"] != "palette" || media["entries"].as_array().map_or(true, |a| a.is_empty()) { return Err(vc_i18n::t!("palettedoc-rel-keine-palettenwerte", rel = &rel)); }
        Ok(PaletteDoc { rel: rel.into(), abs, saved: media.to_string(), media, sel: 0, undo: vec![], redo: vec![], coalesce: None, status: String::new() })
    }
    pub fn anzahl(&self) -> usize { self.media["entries"].as_array().map_or(0, |a| a.len()) }
    pub fn eintrag(&self, i: usize) -> String { self.media["entries"][i].as_str().unwrap_or("#000000").to_string() }
    pub fn rgb(&self, i: usize) -> [u8; 3] { hex_zu_rgb(&self.eintrag(i)).unwrap_or([0; 3]) }
    pub fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    pub fn push_undo(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear(); self.coalesce = None; }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    pub fn undo(&mut self) { if let Some(u) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&u) { self.media = v } self.coalesce = None } }
    pub fn redo(&mut self) { if let Some(u) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&u) { self.media = v } self.coalesce = None } }
    /// Eintrag `i` setzen; aufeinanderfolgende Änderungen am selben Eintrag (Regler ziehen) zählen als ein Undo-Schritt
    pub fn setze(&mut self, i: usize, c: [u8; 3]) {
        if i >= self.anzahl() { return }
        if self.coalesce != Some(i) { self.push_undo(); self.coalesce = Some(i) }
        self.media["entries"][i] = json!(rgb_zu_hex(c));
    }
    /// Ein Farbkanal (0 = R, 1 = G, 2 = B) des gewählten Eintrags
    pub fn kanal(&mut self, k: usize, v: u8) { let mut c = self.rgb(self.sel); if k < 3 { c[k] = v } self.setze(self.sel, c) }
    /// Beendet das Zusammenfassen (Regler losgelassen)
    pub fn fertig(&mut self) { self.coalesce = None }
    pub fn standard(&mut self) { self.push_undo(); self.media["entries"] = json!(vc_media::default_palette()); }
    pub fn media_text(&self) -> String { vc_media::project::format_media(&self.media) }
    pub fn save(&mut self) -> Result<(), String> {
        vc_core::atomar::schreiben(&self.abs, self.media_text()).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = vc_i18n::t!("palettedoc-gespeichert"); Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn farben_regler_undo_und_speichern() {
        let dir = std::env::temp_dir().join(format!("vc-pal-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = vc_media::load_profiles(&root.join("data")).unwrap(); let p = profiles.iter().find(|p| p.kind == "palette").expect("Palettenprofil");
        std::fs::write(dir.join("p.vcpal"), vc_media::project::format_media(&vc_media::new_media(p, "p"))).unwrap();
        let mut d = PaletteDoc::open(&dir, "p.vcpal").unwrap(); assert!(d.anzahl() >= 16 && !d.dirty());
        let alt = d.rgb(3); d.sel = 3; d.kanal(0, 200); d.kanal(0, 201); d.kanal(1, 7); d.fertig();
        assert_eq!((d.rgb(3)[0], d.rgb(3)[1]), (201, 7)); assert!(d.dirty());
        d.undo(); assert_eq!(d.rgb(3), alt, "die Reglerbewegungen am selben Eintrag sind ein einziger Undo-Schritt"); d.redo(); assert_eq!(d.rgb(3)[0], 201);
        d.save().unwrap(); let d2 = PaletteDoc::open(&dir, "p.vcpal").unwrap(); assert_eq!(d2.rgb(3)[0], 201);
        assert_eq!(hex_zu_rgb("#0a0b0c"), Some([10, 11, 12])); assert_eq!(hex_zu_rgb("xyz"), None); assert_eq!(rgb_zu_hex([1, 2, 255]), "#0102ff");
        d.standard(); assert!(d.dirty()); std::fs::remove_dir_all(&dir).ok();
    }
}
