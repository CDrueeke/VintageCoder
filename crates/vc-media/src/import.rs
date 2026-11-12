// SPDX-License-Identifier: GPL-3.0-or-later
//! PNG-Import: skaliert ein Bild auf Frame-Größe und quantisiert es auf die Zielpalette des Profils.
//! Meldet Farbkonflikte (mehr Farben im Bild als das Profil gleichzeitig darstellen kann).
use serde_json::{json, Value};
use std::path::Path;
use crate::{png, Profile};

pub use crate::palette::Rgb;


/// Palette des Ziels als (Farbwert, RGB), siehe `crate::palette` (`sys` = pal|ntsc)
pub fn palette(data: &Path, target: &str, sys: &str) -> Result<Vec<(u32, Rgb)>, String> { Ok(crate::palette::palette_for(data, target, sys)?.entries()) }

fn dist(a: Rgb, b: Rgb) -> i32 { // gewichteter RGB-Abstand (wahrnehmungsnäher als reines Euklid)
    let rm = (a[0] as i32 + b[0] as i32) / 2; let (dr, dg, db) = (a[0] as i32 - b[0] as i32, a[1] as i32 - b[1] as i32, a[2] as i32 - b[2] as i32);
    (((512 + rm) * dr * dr) >> 8) + 4 * dg * dg + (((767 - rm) * db * db) >> 8)
}
fn nearest(c: Rgb, cands: &[(u32, Rgb)]) -> usize { (0..cands.len()).min_by_key(|&i| dist(c, cands[i].1)).unwrap() }

/// Schlüssel in `colors`, deren Farben die Pixelwerte 1.. (bzw. 0.. bei Zeichensätzen) belegen
fn value_keys(p: &Profile, frame: usize) -> Vec<Option<String>> {
    match p.kind.as_str() {
        "sprite" => if p.bpp == 1 { vec![None, Some("sprite".into())] } else { vec![None, Some("mc1".into()), Some("sprite".into()), Some("mc2".into())] },
        "pmg" => vec![None, Some(format!("P{frame}"))],
        _ => if p.bpp == 1 { vec![Some("bg".into()), Some("fg".into())] } else { vec![Some("bg".into()), Some("mc1".into()), Some("mc2".into()), Some("fg".into())] },
    }
}

/// Importiert `png_bytes` in Frame `frame` von `media`. `auto`: Farben des Media aus dem Bild bestimmen.
pub fn import_png(png_bytes: &[u8], media: &mut Value, p: &Profile, data: &Path, frame: usize, auto: bool, sys: &str) -> Result<Value, String> {
    if !matches!(p.kind.as_str(), "sprite" | "pmg" | "charset") { return Err(format!("PNG-Import: für '{}' nicht verfügbar", p.kind)); }
    if p.bpp > 4 { return Err("PNG-Import: nur für Profile mit höchstens 16 Farben (4 Bit je Pixel)".into()); }
    let img = png::decode(png_bytes)?;
    let frames = media["frames"].as_array().ok_or("frames fehlt")?; if frame >= frames.len() { return Err(format!("Frame {frame} existiert nicht")); }
    let (w, h) = (p.width, frames[frame]["rows"].as_array().map(|r| r.len()).unwrap_or(p.height));
    let pal = palette(data, &p.target, sys)?;
    // Skalieren (Flächenmittel), Transparenz < 50 % Deckkraft
    let mut cells: Vec<Option<Rgb>> = Vec::with_capacity(w * h);
    for y in 0..h { for x in 0..w {
        let (x0, x1) = (x * img.w / w, ((x + 1) * img.w / w).max(x * img.w / w + 1)); let (y0, y1) = (y * img.h / h, ((y + 1) * img.h / h).max(y * img.h / h + 1));
        let (mut n, mut op, mut s) = (0u32, 0u32, [0u32; 3]);
        for yy in y0..y1.min(img.h) { for xx in x0..x1.min(img.w) { let i = (yy * img.w + xx) * 4; n += 1; if img.rgba[i + 3] >= 128 { op += 1; for k in 0..3 { s[k] += img.rgba[i + k] as u32; } } } }
        cells.push(if n > 0 && op * 2 >= n { Some([(s[0] / op) as u8, (s[1] / op) as u8, (s[2] / op) as u8]) } else { None });
    } }
    let keys = value_keys(p, frame); let charset = p.kind == "charset";
    // Zielfarben je Pixelwert
    let first = if charset { 0 } else { 1 };
    let want: Vec<usize> = (first..keys.len()).collect();
    // Häufigkeit der Palettenfarben im Bild
    let mut hist = vec![0usize; pal.len()];
    for c in cells.iter().flatten() { hist[nearest(*c, &pal)] += 1; }
    let distinct = hist.iter().filter(|&&n| n > 0).count();
    let colors = media["colors"].as_object_mut().ok_or("colors fehlt")?;
    if auto {
        let mut order: Vec<usize> = (0..pal.len()).filter(|&i| hist[i] > 0).collect(); order.sort_by_key(|&i| std::cmp::Reverse(hist[i]));
        // Zeichensatz: Hintergrund = häufigste Farbe, Rest nach Häufigkeit; Sprites: Reihenfolge der Schlüssel
        let mut it = order.into_iter();
        for &v in &want { if let (Some(k), Some(i)) = (&keys[v], it.next()) { colors.insert(k.clone(), json!(pal[i].0)); } }
    }
    let cand: Vec<(usize, (u32, Rgb))> = want.iter().filter_map(|&v| keys[v].as_ref().and_then(|k| colors.get(k)).and_then(|c| c.as_u64()).and_then(|c| pal.iter().find(|e| e.0 == c as u32).map(|e| (v, *e)))).collect();
    if cand.is_empty() { return Err("keine Zielfarben in dieser Datei".into()); }
    let cl: Vec<(u32, Rgb)> = cand.iter().map(|c| c.1).collect();
    let mut used_pixels = vec![0usize; cand.len()]; let mut dropped: Vec<usize> = vec![0; pal.len()];
    let mut rows = Vec::new();
    for y in 0..h {
        let mut row = String::new();
        for x in 0..w {
            match cells[y * w + x] {
                None => row.push('0'),
                Some(c) => { let k = nearest(c, &cl); used_pixels[k] += 1; row.push(char::from_digit(cand[k].0 as u32, 16).unwrap()); }
            }
        }
        rows.push(Value::String(row));
    }
    let chosen: Vec<u32> = cl.iter().map(|c| c.0).collect();
    for (i, n) in hist.iter().enumerate() { if *n > 0 && !chosen.contains(&pal[i].0) { dropped[i] = *n; } }
    let lost: Vec<Value> = (0..pal.len()).filter(|&i| dropped[i] > 0).map(|i| json!({ "color": pal[i].0, "pixels": dropped[i] })).collect();
    let colors = Value::Object(colors.clone());
    media["frames"][frame]["rows"] = Value::Array(rows);
    Ok(json!({ "frame": frame, "size": [w, h], "imageColors": distinct, "allowed": cand.len(), "conflict": distinct > cand.len(), "reducedColors": lost, "colors": colors }))
}
