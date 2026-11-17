// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeichensatz-Glyphen zum Zeichnen von Bildschirmen und Kacheln, ohne Oberfläche: aus einer Zeichensatz-Datei (`.vcchar`) werden die 8×8-Muster
//! gelesen und mit den Farben des Systems in einen RGBA-Puffer gezeichnet. Mehrfarb-Zeichensätze (4 Pixel je Zeile, Werte 0–3) bekommen
//! ihre Farben aus Farb-RAM, Bildschirm/Kachelsatz und Zeichensatz (Reihenfolge wie in der Hardware).
use serde_json::Value;
use std::path::Path;

/// Farben der Mehrfarbwerte, falls ein Bildschirm/Kachelsatz eigene hat (`None`: die des Zeichensatzes):
/// `mc1`, `mc2` = Pixelwerte 1 und 2, `pf3` = Farbe der Zeichen mit Bit 7 am Atari (COLPF3 statt COLPF2)
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct McCols { pub mc1: Option<usize>, pub mc2: Option<usize>, pub pf3: Option<usize> }
impl McCols {
    pub fn aus_farben(c: &Value) -> McCols { let g = |k: &str| c[k].as_u64().map(|v| v as usize); McCols { mc1: g("mc1"), mc2: g("mc2"), pf3: g("pf3") } }
}

/// Die Glyphen eines Zeichensatzes
#[derive(Clone, Debug)]
pub struct Glyphs {
    /// je Zeichen 8 Zeilen aus Ziffern (`0`–`3`), bei Mehrfarbe 4 Ziffern je Zeile, sonst 8
    pub zeilen: Vec<Vec<String>>,
    pub mc: bool, mc1: usize, mc2: usize, pf3: Option<usize>,
}

/// RGBA-Puffer mit Breite und Höhe in Pixeln
#[derive(Clone, Debug, PartialEq)]
pub struct Bild { pub w: usize, pub h: usize, pub rgba: Vec<u8> }
impl Bild {
    pub fn neu(w: usize, h: usize, farbe: [u8; 3]) -> Bild { let mut rgba = Vec::with_capacity(w * h * 4); for _ in 0..w * h { rgba.extend_from_slice(&[farbe[0], farbe[1], farbe[2], 255]) } Bild { w, h, rgba } }
    pub fn setze(&mut self, x: usize, y: usize, f: [u8; 3]) { if x < self.w && y < self.h { let o = (y * self.w + x) * 4; self.rgba[o..o + 3].copy_from_slice(&f) } }
    pub fn flaeche(&mut self, x0: usize, y0: usize, w: usize, h: usize, f: [u8; 3]) { for y in y0..y0 + h { for x in x0..x0 + w { self.setze(x, y, f) } } }
}

impl Glyphs {
    /// Zeichensatz aus einer Datei laden; `None`, wenn es keine Zeichensatz-Datei ist
    pub fn laden(root: &Path, rel: &str) -> Option<Glyphs> {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(root.join(rel)).ok()?).ok()?;
        Self::aus_wert(&v)
    }
    pub fn aus_wert(v: &Value) -> Option<Glyphs> {
        let frames = v["frames"].as_array()?;
        let zeilen: Vec<Vec<String>> = frames.iter().map(|f| f["rows"].as_array().map(|r| r.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()).collect();
        let prof = v["profile"].as_str().unwrap_or("");
        let mc = prof.contains("multicolor") || prof.contains("antic4") || zeilen.iter().any(|r| r.iter().any(|s| { let n = s.chars().count(); n > 0 && n <= 4 }));
        let col = |k: &str, d: usize| v["colors"][k].as_u64().map_or(d, |x| x as usize);
        Some(Glyphs { zeilen, mc, mc1: col("mc1", 2), mc2: col("mc2", 5), pf3: v["colors"]["pf3"].as_u64().map(|x| x as usize) })
    }
    pub fn anzahl(&self) -> usize { self.zeilen.len() }

    /// Zeichen `code` an (`ox`,`oy`) in `bild` zeichnen (Hintergrund bleibt, nur gesetzte Pixel); `zelle` = Farbe der Zelle (Farb-RAM bzw. Vordergrund),
    /// `mc` = Farben der Mehrfarbwerte, `hi` = Zeichenbit 7 (Atari ANTIC 4), `rgb` = Palette des Systems. `false`, wenn es das Zeichen nicht gibt.
    /// `skala` = Pixelgröße (ganze Zahl ≥ 1): jedes Glyph-Pixel wird `skala`×`skala` Bildpunkte groß.
    #[allow(clippy::too_many_arguments)]
    pub fn zeichne(&self, bild: &mut Bild, ox: usize, oy: usize, skala: usize, ziel: &str, code: u8, zelle: usize, mc: McCols, hi: bool, rgb: &dyn Fn(usize) -> [u8; 3]) -> bool {
        let Some(zeilen) = self.zeilen.get(code as usize) else { return false };
        let vic2 = matches!(ziel, "c64" | "mega65" | "vic20");
        // VIC-II/VIC: Mehrfarbe nur, wenn Bit 3 der Farbe gesetzt ist (dann zählen die unteren 3 Bit); TED/ANTIC 4: immer
        let echt_mc = self.mc && (!vic2 || zelle >= 8);
        let c3 = if vic2 { zelle & 7 } else if hi { mc.pf3.or(self.pf3).unwrap_or(zelle) } else { zelle };
        let s = skala.max(1);
        for (y, r) in zeilen.iter().take(8).enumerate() {
            if self.mc {
                for (x, c) in r.chars().take(4).enumerate() {
                    let val = c.to_digit(4).unwrap_or(0) as usize; if val == 0 { continue }
                    if echt_mc {
                        let f = rgb(match val { 1 => mc.mc1.unwrap_or(self.mc1), 2 => mc.mc2.unwrap_or(self.mc2), _ => c3 });
                        bild.flaeche(ox + x * 2 * s, oy + y * s, 2 * s, s, f);
                    } else { // Hires-Deutung derselben Bytes
                        let f = rgb(zelle);
                        if val & 2 != 0 { bild.flaeche(ox + x * 2 * s, oy + y * s, s, s, f) }
                        if val & 1 != 0 { bild.flaeche(ox + (x * 2 + 1) * s, oy + y * s, s, s, f) }
                    }
                }
            } else {
                let f = rgb(zelle);
                for (x, c) in r.chars().take(8).enumerate() { if c != '0' { bild.flaeche(ox + x * s, oy + y * s, s, s, f) } }
            }
        }
        true
    }
}

/// ASCII → Bildschirmcode des Zielsystems
pub fn bildschirmcode(ziel: &str, ch: char) -> u8 {
    let a = ch as u32;
    if ziel == "atari8" { return if (32..96).contains(&a) { (a - 32) as u8 } else if a < 32 { (a + 64) as u8 } else { (a & 127) as u8 }; }
    if ch.is_ascii_lowercase() { return (a - 96) as u8; }
    if ch.is_ascii_uppercase() { return (a - 64) as u8; }
    (a & 127) as u8
}

/// Zeichen für die Anzeige eines Bildschirmcodes (Hinweise, Bildschirmleser)
pub fn code_zeichen(ziel: &str, code: u8) -> char {
    if ziel == "atari8" { return char::from_u32(code as u32 + 32).filter(|c| c.is_ascii_graphic()).unwrap_or('·') }
    match code { 1..=26 => (b'A' + code - 1) as char, 32..=63 => code as char, 0 => '@', _ => '·' }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn font(mc: bool) -> Glyphs {
        let rows8: Vec<String> = (0..8).map(|y| if y == 0 { "11111111".into() } else { "10000001".into() }).collect();
        let rows4: Vec<String> = (0..8).map(|y| if y == 0 { "1230".into() } else { "0000".into() }).collect();
        Glyphs::aus_wert(&json!({ "frames": [{ "rows": if mc { rows4 } else { rows8 } }], "profile": "x", "colors": { "mc1": 2, "mc2": 5 } })).unwrap()
    }
    fn rgb(i: usize) -> [u8; 3] { [i as u8 * 10, 0, 0] }

    #[test]
    fn hires_zeichnet_gesetzte_pixel_in_der_zellenfarbe_und_skaliert() {
        let g = font(false); let mut b = Bild::neu(16, 16, [1, 2, 3]);
        assert!(g.zeichne(&mut b, 0, 0, 2, "c64", 0, 7, McCols::default(), false, &rgb));
        assert_eq!(&b.rgba[0..4], &[70, 0, 0, 255], "Pixel (0,0) gesetzt, 2×2 groß"); assert_eq!(&b.rgba[(1 * 16 + 1) * 4..][..3], &[70, 0, 0]);
        assert_eq!(&b.rgba[(4 * 16 + 4) * 4..][..3], &[1, 2, 3], "Mitte leer: Hintergrund bleibt"); assert!(!g.zeichne(&mut b, 0, 0, 1, "c64", 5, 7, McCols::default(), false, &rgb), "Zeichen gibt es nicht");
    }

    #[test]
    fn mehrfarbe_nur_mit_gesetztem_farbbit3_am_c64_atari_immer() {
        let g = font(true); assert!(g.mc);
        // C64: Farb-RAM 3 (< 8) → Hires-Deutung der Bytes in Farbe 3; Farb-RAM 9 → Mehrfarbe mit Farben mc1=2, mc2=5, Zelle&7=1
        let mut hi = Bild::neu(8, 8, [0, 0, 0]); g.zeichne(&mut hi, 0, 0, 1, "c64", 0, 3, McCols::default(), false, &rgb);
        assert_eq!(&hi.rgba[0..3], &[0, 0, 0], "Wert 1 = 01 in Hires-Deutung: nur das rechte Pixel (Bit 0), links leer"); assert_eq!(&hi.rgba[4..7], &[30, 0, 0]);
        let mut mc = Bild::neu(8, 8, [0, 0, 0]); g.zeichne(&mut mc, 0, 0, 1, "c64", 0, 9, McCols::default(), false, &rgb);
        assert_eq!(&mc.rgba[0..3], &[20, 0, 0], "Wert 1 → mc1 = Farbe 2, doppelt breit"); assert_eq!(&mc.rgba[4..7], &[20, 0, 0]);
        assert_eq!(&mc.rgba[8..11], &[50, 0, 0], "Wert 2 → mc2 = Farbe 5"); assert_eq!(&mc.rgba[16..19], &[10, 0, 0], "Wert 3 → Zelle & 7 = 1");
        // Atari: immer Mehrfarbe; eigene Farben des Bildschirms gehen vor
        let mut a = Bild::neu(8, 8, [0, 0, 0]); g.zeichne(&mut a, 0, 0, 1, "atari8", 0, 3, McCols { mc1: Some(7), ..Default::default() }, false, &rgb);
        assert_eq!(&a.rgba[0..3], &[70, 0, 0], "mc1 des Bildschirms vor dem des Zeichensatzes"); assert_eq!(&a.rgba[16..19], &[30, 0, 0], "Wert 3 = Zellenfarbe");
    }

    #[test]
    fn bildschirmcodes() {
        assert_eq!(bildschirmcode("c64", 'a'), 1); assert_eq!(bildschirmcode("c64", 'A'), 1); assert_eq!(bildschirmcode("atari8", 'A'), 33); assert_eq!(bildschirmcode("c64", ' '), 32);
        assert_eq!(code_zeichen("c64", 1), 'A'); assert_eq!(code_zeichen("c64", 0), '@'); assert_eq!(code_zeichen("atari8", 33), 'A'); assert_eq!(code_zeichen("c64", 200), '·');
    }
}
