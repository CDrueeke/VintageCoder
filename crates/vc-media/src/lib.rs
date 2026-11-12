// SPDX-License-Identifier: GPL-3.0-or-later
//! Media-Modell und Exporte: Sprites, Player-Missile-Grafik, Zeichensätze, Bildschirme und Kachelkarten.
//!
//! Natives Format (`<name>.vc<art>`, z. B. `font.vcchar`): Pixel als Zeilen-Strings aus Hex-Ziffern (Palettenindex je Pixel), damit Git-Diffs lesbar bleiben.
pub mod naming;
pub mod project;
pub mod png;
pub mod import;
pub mod song;
pub mod goat;
pub mod sprite;
pub mod spriteimport;
pub mod cpu6502;
pub mod sid;
pub mod play;
pub mod wizard;
pub mod palette;
pub mod romref;
pub mod tiles;

pub use vc_roms::Status;

use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub target: String,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub block: usize,
    pub count: usize,
    pub raw: Value,
}

/// Löst `extends` auf: ein Profil erbt alle Felder des Elternprofils und überschreibt nur die eigenen (höchstens drei Ebenen, Zyklen sind Fehler).
/// `id` und `extends` selbst werden nicht vererbt; im Ergebnis fehlt `extends`.
pub fn resolve_profile_json(all: &[Value], p: &Value, depth: usize) -> Result<Value, String> {
    let id = p["id"].as_str().unwrap_or("?");
    let Some(parent_id) = p["extends"].as_str() else { return Ok(p.clone()) };
    if depth >= 3 { return Err(format!("Profil '{id}': extends ist zu tief verschachtelt (höchstens 3 Ebenen) oder zyklisch")); }
    let parent = all.iter().find(|x| x["id"] == parent_id).ok_or_else(|| format!("Profil '{id}': extends verweist auf unbekanntes Profil '{parent_id}'"))?;
    let mut merged = resolve_profile_json(all, parent, depth + 1)?;
    let (m, c) = (merged.as_object_mut().ok_or("Profil ist kein Objekt")?, p.as_object().ok_or("Profil ist kein Objekt")?);
    for (k, v) in c { if k != "extends" { m.insert(k.clone(), v.clone()); } }
    m.remove("extends");
    Ok(merged)
}

pub fn load_profiles(data_dir: &Path) -> Result<Vec<Profile>, String> {
    let path = data_dir.join("profile/profiles.json");
    let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let all = v["profiles"].as_array().ok_or("profiles fehlt")?;
    all.iter().map(|p| {
        let p = resolve_profile_json(all, p, 0).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Profile {
            id: p["id"].as_str().unwrap_or("").into(), kind: p["kind"].as_str().unwrap_or("").into(), name: p["name"].as_str().unwrap_or("").into(),
            target: p["target"].as_str().unwrap_or("").into(), width: p["width"].as_u64().unwrap_or(0) as usize, height: p["height"].as_u64().unwrap_or(0) as usize,
            bpp: p["bpp"].as_u64().unwrap_or(1) as usize, block: p["block"].as_u64().unwrap_or(0) as usize, count: p["count"].as_u64().unwrap_or(0) as usize, raw: p,
        })
    }).collect()
}

pub fn profile<'a>(profiles: &'a [Profile], id: &str) -> Result<&'a Profile, String> { profiles.iter().find(|p| p.id == id).ok_or_else(|| format!("unbekanntes Profil '{id}'")) }

/// Ein Media im natürlichen Format (nur gelesen; geschrieben wird von der Oberfläche).
#[derive(Debug, Clone)]
pub struct Media { pub kind: String, pub profile: String, pub name: String, pub root: Value }

pub fn parse_media(text: &str) -> Result<Media, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("Datei (JSON): {e}"))?;
    // Format 2 (Zeichensatz mit `base` auf ein ROM): Verweis auflösen, `overrides` darüberlegen; Speichern geht über `romref::for_saving`
    if v["format"] == 2 && v["kind"] == "charset" && v["base"].is_string() {
        let r = romref::resolve_media(&v)?;
        return Ok(Media { kind: "charset".into(), profile: r["profile"].as_str().ok_or("profile fehlt")?.into(), name: r["name"].as_str().unwrap_or("media").into(), root: r });
    }
    if v["format"] != 1 { return Err("Datei: format 1 erwartet".into()); }
    Ok(Media { kind: v["kind"].as_str().ok_or("kind fehlt")?.into(), profile: v["profile"].as_str().ok_or("profile fehlt")?.into(), name: v["name"].as_str().unwrap_or("media").into(), root: v })
}

/// Leeres Media passend zum Profil (als JSON-Wert).
pub fn new_media(p: &Profile, name: &str) -> Value {
    let blank_row = |w: usize| "0".repeat(w * p.raw["digits"].as_u64().unwrap_or(1) as usize);
    let frame = |h: usize| json!({ "rows": (0..h).map(|_| blank_row(p.width)).collect::<Vec<_>>() });
    let colors = p.raw["defaultColors"].clone();
    match p.kind.as_str() {
        "sprite" => json!({ "format": 1, "kind": "sprite", "profile": p.id, "name": name, "colors": colors, "frames": [frame(p.height)] }),
        "pmg" => { let n = p.raw["frames"].as_u64().unwrap_or(4) as usize; json!({ "format": 1, "kind": "pmg", "profile": p.id, "name": name, "height": p.height, "colors": colors, "frames": (0..n).map(|_| frame(p.height)).collect::<Vec<_>>() }) }
        "charset" => json!({ "format": 1, "kind": "charset", "profile": p.id, "name": name, "colors": colors, "frames": (0..p.count).map(|_| frame(p.height)).collect::<Vec<_>>() }),
        "song" => song::new_song(p, name),
        "play" => play::new_play(p, name),
        "palette" => json!({ "format": 1, "kind": "palette", "profile": p.id, "name": name, "entries": default_palette() }),
        "screen" => json!({ "format": 1, "kind": "screen", "profile": p.id, "name": name, "colors": colors, "charset": null,
                            "chars": (0..p.height).map(|_| vec![32; p.width]).collect::<Vec<_>>(), "attrs": (0..p.height).map(|_| vec![p.raw["defaultColors"]["fg"].as_u64().unwrap_or(1); p.width]).collect::<Vec<_>>() }),
        "tileset" => json!({ "format": 1, "kind": "tileset", "profile": p.id, "name": name, "size": { "w": p.raw["tileWidth"].as_u64().unwrap_or(2), "h": p.raw["tileHeight"].as_u64().unwrap_or(2) },
                             "colorMode": if p.raw["colorRam"].as_bool().unwrap_or(false) { "tile" } else { "none" }, "charsets": [], "propDefs": [{ "name": "solid", "bit": 0 }],
                             "tiles": [{ "name": "leer", "cells": vec![0; (p.raw["tileWidth"].as_u64().unwrap_or(2) * p.raw["tileHeight"].as_u64().unwrap_or(2)) as usize], "col": 1 }] }),
        _ => json!({ "format": 1, "kind": "tilemap", "profile": p.id, "name": name, "colors": colors, "charset": null,
                     "tiles": [{ "name": "leer", "chars": [32, 32, 32, 32], "color": 1, "solid": false }],
                     "map": (0..p.height).map(|_| vec![0; p.width]).collect::<Vec<_>>() }),
    }
}

fn parse_rows(frame: &Value, w: usize, h: usize, maxv: u32, digits: usize) -> Result<Vec<Vec<u32>>, String> {
    let rows = frame["rows"].as_array().ok_or("frame.rows fehlt")?;
    if rows.len() != h { return Err(format!("Frame hat {} statt {h} Zeilen", rows.len())); }
    rows.iter().map(|r| {
        let s = r.as_str().ok_or("Zeile ist kein String")?;
        if s.chars().count() != w * digits { return Err(format!("Zeile '{s}' hat {} statt {w} Pixel ({digits} Hex-Ziffer(n) je Pixel)", s.chars().count() / digits)); }
        let cs: Vec<char> = s.chars().collect();
        cs.chunks(digits).map(|c| { let t: String = c.iter().collect(); u32::from_str_radix(&t, 16).ok().filter(|v| *v <= maxv).ok_or(format!("Pixelwert '{t}' ungültig (max. {maxv})")) }).collect()
    }).collect()
}

/// Kodiert Bitmap-Frames (Sprite, PMG, Zeichensatz) zeilenweise, MSB links, Blöcke auf `block` Bytes aufgefüllt.
pub fn encode(media: &Media, p: &Profile) -> Result<Vec<u8>, String> {
    match p.kind.as_str() {
        "sprite" | "pmg" | "charset" => {}
        _ => return Err(format!("'{}' hat keine Bitmap-Kodierung (nutze encode_screen/encode_tilemap)", p.kind)),
    }
    let h = if p.kind == "pmg" { media.root["height"].as_u64().unwrap_or(p.height as u64) as usize } else { p.height };
    let frames = media.root["frames"].as_array().ok_or("frames fehlt")?;
    if p.kind == "charset" && frames.len() != p.count { return Err(format!("Zeichensatz braucht {} Zeichen, Datei hat {}", p.count, frames.len())); }
    if (p.width * p.bpp) % 8 != 0 { return Err("Breite × Bits pro Pixel muss durch 8 teilbar sein".into()); }
    let mut out = vec![];
    for (i, f) in frames.iter().enumerate() {
        // Sprites: jedes Frame darf hires oder multicolor sein (`mode` im Frame)
        let (fw, fbpp) = sprite::frame_geometry(p, f);
        let maxv = (1u32 << fbpp) - 1;
        let rows = parse_rows(f, fw, h, maxv, p.raw["digits"].as_u64().unwrap_or(1) as usize).map_err(|e| format!("Frame {i}: {e}"))?;
        let mut block = vec![];
        for r in rows {
            let mut acc = 0u32;
            let mut bits = 0;
            for px in r { acc = (acc << fbpp) | px; bits += fbpp; if bits == 8 { block.push(acc as u8); acc = 0; bits = 0; } }
        }
        if p.block > 0 { if block.len() > p.block { return Err(format!("Frame {i}: {} Bytes passen nicht in Block von {}", block.len(), p.block)); } block.resize(p.block, 0); }
        out.extend(block);
    }
    Ok(out)
}

/// Zeichensatz aus einem ROM-Abbild: `offset` = Beginn des Zeichensatzes in der Datei. Profile mit 8 Bit je Pixel (MEGA65-Vollfarbe)
/// bekommen die 1-Bit-Vorlage mit Palettenfarbe 1 (weiß) als Vordergrund.
pub fn import_rom(p: &Profile, bytes: &[u8], offset: usize, name: &str) -> Result<Value, String> {
    if p.kind != "charset" { return Err(format!("ROM-Import: nur für Zeichensätze, '{}' ist {}", p.id, p.kind)); }
    let tail = bytes.get(offset..).ok_or_else(|| format!("Offset {offset} liegt hinter dem Dateiende ({} Bytes)", bytes.len()))?;
    if p.bpp == 8 {
        let mut one = p.clone(); one.bpp = 1; one.block = one.height * one.width / 8;
        let v = decode(&one, tail, name)?;
        let mut a = new_media(p, name);
        a["frames"] = Value::Array(v["frames"].as_array().ok_or("frames fehlen")?.iter().map(|f| json!({ "rows": f["rows"].as_array().map(|r| r.iter().map(|row| row.as_str().unwrap_or("").chars().map(|c| if c == '0' { "00" } else { "01" }).collect::<String>()).collect::<Vec<_>>()).unwrap_or_default() })).collect());
        return Ok(a);
    }
    decode(p, tail, name)
}

/// Zeichensatz aus einem PNG-Raster (16 Zeichen je Zeile, Zeichen in Leserichtung, beliebig vergrößert): helle Pixel = gesetzt (`invert` dreht das um).
pub fn import_sheet(p: &Profile, png: &[u8], name: &str, invert: bool) -> Result<Value, String> {
    if p.kind != "charset" { return Err(format!("PNG-Raster: nur für Zeichensätze, '{}' ist {}", p.id, p.kind)); }
    if p.bpp != 1 && p.bpp != 8 { return Err("PNG-Raster: nur für Zeichensätze mit 1 Bit je Pixel (und die MEGA65-Vollfarbe)".into()); }
    let img = png::decode(png)?; let (cols, rows) = (16usize, (p.count + 15) / 16);
    if img.w % cols != 0 || img.h % rows != 0 { return Err(format!("Bild {}×{} passt nicht zu {cols}×{rows} Zeichen", img.w, img.h)); }
    let (cw, ch) = (img.w / cols, img.h / rows);
    if cw % p.width != 0 || ch % p.height != 0 || cw / p.width != ch / p.height || cw < p.width { return Err(format!("Zeichen im Bild sind {cw}×{ch} Pixel – erwartet ein ganzzahliges Vielfaches von {}×{}", p.width, p.height)); }
    let k = cw / p.width; let digits = if p.bpp == 8 { 2 } else { 1 };
    let on = |x: usize, y: usize| -> bool { let i = (y * img.w + x) * 4; let (r, g, b, a) = (img.rgba[i] as u32, img.rgba[i + 1] as u32, img.rgba[i + 2] as u32, img.rgba[i + 3] as u32); (a >= 128 && (r * 30 + g * 59 + b * 11) / 100 >= 128) != invert };
    let frames: Vec<Value> = (0..p.count).map(|n| {
        let (ox, oy) = ((n % cols) * cw, (n / cols) * ch);
        json!({ "rows": (0..p.height).map(|y| (0..p.width).map(|x| { let v = on(ox + x * k + k / 2, oy + y * k + k / 2); if digits == 2 { if v { "01" } else { "00" } } else if v { "1" } else { "0" } }).collect::<String>()).collect::<Vec<_>>() })
    }).collect();
    let mut a = new_media(p, name); a["frames"] = Value::Array(frames); Ok(a)
}

/// Zeichensatz als PNG-Raster (weiß auf schwarz, 16 Zeichen je Zeile, 1:1)
pub fn export_sheet(p: &Profile, media: &Value) -> Result<Vec<u8>, String> {
    if p.kind != "charset" || (p.bpp != 1 && p.bpp != 8) { return Err("PNG-Raster: nur Zeichensätze mit 1 Bit (oder MEGA65-Vollfarbe)".into()); }
    let frames = media["frames"].as_array().ok_or("frames fehlen")?; let (cols, rows) = (16usize, (frames.len() + 15) / 16);
    let (w, h) = (cols * p.width, rows * p.height); let mut rgba = vec![0u8; w * h * 4];
    for px in rgba.chunks_mut(4) { px[3] = 255; }
    let d = if p.bpp == 8 { 2 } else { 1 };
    for (n, f) in frames.iter().enumerate() {
        for (y, row) in f["rows"].as_array().ok_or("rows fehlen")?.iter().enumerate() {
            let cs: Vec<char> = row.as_str().unwrap_or("").chars().collect();
            for x in 0..p.width { if cs[x * d..x * d + d].iter().any(|c| *c != '0') { let (px, py) = ((n % cols) * p.width + x, (n / cols) * p.height + y); let i = (py * w + px) * 4; rgba[i..i + 3].copy_from_slice(&[255, 255, 255]); } }
        }
    }
    Ok(png::encode_rgba(w, h, &rgba))
}

/// Umkehrung von `encode` (Import von Rohdaten).
pub fn decode(p: &Profile, bytes: &[u8], name: &str) -> Result<Value, String> {
    let rowbytes = p.width * p.bpp / 8;
    let used = rowbytes * p.height;
    let block = if p.block > 0 { p.block } else { used };
    if block == 0 || bytes.len() < block { return Err("zu wenige Daten".into()); }
    let n = if p.kind == "charset" { p.count } else { bytes.len() / block };
    if bytes.len() < n * block { return Err(format!("{} Bytes für {n} Blöcke zu je {block} nötig", n * block)); }
    let mut a = new_media(p, name);
    let mut frames = vec![];
    for i in 0..n {
        let blk = &bytes[i * block..i * block + used];
        let rows: Vec<String> = (0..p.height).map(|y| {
            let mut s = String::new();
            for x in 0..p.width { let bit = x * p.bpp; let byte = blk[y * rowbytes + bit / 8]; if p.bpp == 8 { s.push_str(&format!("{byte:02x}")); continue; } let sh = 8 - p.bpp - (bit % 8); s.push(std::char::from_digit(((byte >> sh) as u32) & ((1 << p.bpp) - 1), 16).unwrap()); }
            s
        }).collect();
        frames.push(json!({ "rows": rows }));
    }
    a["frames"] = Value::Array(frames);
    Ok(a)
}

/// Bildschirm: (Zeichencodes, Attribute/Farben) zeilenweise.
pub fn encode_screen(media: &Media, p: &Profile) -> Result<(Vec<u8>, Vec<u8>), String> {
    let grid = |key: &str| -> Result<Vec<u8>, String> {
        let rows = media.root[key].as_array().ok_or(format!("{key} fehlt"))?;
        if rows.len() != p.height { return Err(format!("{key}: {} statt {} Zeilen", rows.len(), p.height)); }
        let mut v = vec![];
        for r in rows { let r = r.as_array().ok_or("Zeile ist kein Array")?; if r.len() != p.width { return Err(format!("{key}: Zeile hat {} statt {} Zellen", r.len(), p.width)); }
            for c in r { v.push(c.as_u64().filter(|x| *x <= 255).ok_or(format!("{key}: Wert außerhalb 0..255"))? as u8); } }
        Ok(v)
    };
    Ok((grid("chars")?, if p.raw["colorRam"].as_bool().unwrap_or(false) { grid("attrs")? } else { vec![] }))
}

/// Kachelkarte: (Kachelzeichen je Kachel (w×h Codes), Karte als Kachelindizes, Kachelfarben).
pub fn encode_tilemap(media: &Media, p: &Profile) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
    let (tw, th) = (p.raw["tileWidth"].as_u64().unwrap_or(2) as usize, p.raw["tileHeight"].as_u64().unwrap_or(2) as usize);
    let tiles = media.root["tiles"].as_array().ok_or("tiles fehlt")?;
    let (mut chars, mut colors) = (vec![], vec![]);
    for (i, t) in tiles.iter().enumerate() {
        let c = t["chars"].as_array().ok_or(format!("Kachel {i}: chars fehlt"))?;
        if c.len() != tw * th { return Err(format!("Kachel {i}: {} statt {} Zeichen", c.len(), tw * th)); }
        for x in c { chars.push(x.as_u64().filter(|v| *v <= 255).ok_or(format!("Kachel {i}: Zeichencode außerhalb 0..255"))? as u8); }
        colors.push(t["color"].as_u64().unwrap_or(1) as u8);
    }
    let map_rows = media.root["map"].as_array().ok_or("map fehlt")?;
    let mut map = vec![];
    for r in map_rows { for c in r.as_array().ok_or("map-Zeile ist kein Array")? { let v = c.as_u64().ok_or("map: Zahl erwartet")? as usize; if v >= tiles.len() { return Err(format!("map verweist auf Kachel {v}, es gibt nur {}", tiles.len())); } map.push(v as u8); } }
    Ok((chars, map, colors))
}

// ------------------------------------------------------------------ Exporte

pub fn export_asm(label: &str, bytes: &[u8], per_line: usize) -> String {
    let mut s = format!("; erzeugt von VintageCoder – nicht von Hand ändern\n{label}:\n");
    for c in bytes.chunks(per_line.max(1)) { s.push_str(&format!("        .byte {}\n", c.iter().map(|b| format!("${b:02x}")).collect::<Vec<_>>().join(","))); }
    s.push_str(&format!(".const {label}_len = {}\n", bytes.len()));
    s
}

pub fn export_prg(address: u16, bytes: &[u8]) -> Vec<u8> { let mut v = vec![(address & 0xFF) as u8, (address >> 8) as u8]; v.extend_from_slice(bytes); v }

/// BASIC-DATA-Zeilen (Dezimalwerte, höchstens 12 Werte pro Zeile).
pub fn export_data(bytes: &[u8], start: u32, step: u32) -> String {
    bytes.chunks(12).enumerate().map(|(i, c)| format!("{} DATA {}\n", start + i as u32 * step, c.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","))).collect()
}

/// DATA-Zeilen ohne Zeilennummern, eingeleitet durch ein BASIC-Label (`@name:`): wird per `#include` eingebunden und
/// dann mit den übrigen Zeilen nummeriert; im Programm `RESTORE @name` und `READ`.
pub fn export_data_labeled(label: &str, bytes: &[u8]) -> String {
    let mut s = format!("REM erzeugt von VintageCoder: {} Bytes\n@{label}:\n", bytes.len());
    for c in bytes.chunks(12) { s.push_str(&format!("DATA {}\n", c.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","))); }
    s
}

/// Include-Datei mit Adress- und Längenkonstanten.
pub fn export_inc(label: &str, len: usize, address: Option<u16>) -> String {
    let mut s = format!("; erzeugt von VintageCoder\n.const {label}_len = {len}\n");
    if let Some(a) = address { s.push_str(&format!(".const {label}_addr = ${a:04x}\n")); }
    s
}

/// Standardpalette des MEGA65-Editors: C64-Farben (0–15), 6×6×6-Farbwürfel (16–231) und 24 Graustufen (232–255)
pub fn default_palette() -> Vec<String> {
    const C64: [&str; 16] = ["000000", "ffffff", "813338", "75cec8", "8e3c97", "56ac4d", "2e2c9b", "edf171", "8e5029", "553800", "c46c71", "4a4a4a", "7b7b7b", "a9ff9f", "706deb", "b2b2b2"];
    let mut v: Vec<String> = C64.iter().map(|c| format!("#{c}")).collect();
    for i in 0..216 { let (r, g, b) = (i / 36, (i / 6) % 6, i % 6); v.push(format!("#{:02x}{:02x}{:02x}", r * 51, g * 51, b * 51)); }
    for i in 0..24 { let g = 8 + i * 10; v.push(format!("#{g:02x}{g:02x}{g:02x}")); }
    v
}

/// Palette → (rot, grün, blau), je 256 Bytes in der Registerform des VIC-IV (Halbbytes vertauscht, `$D100/$D200/$D300`)
pub fn encode_palette(media: &Media) -> Result<Vec<(String, Vec<u8>)>, String> {
    let e = media.root["entries"].as_array().ok_or("entries fehlt")?;
    if e.len() != 256 { return Err(format!("Palette braucht 256 Einträge, Datei hat {}", e.len())); }
    let mut ch = [vec![], vec![], vec![]];
    for (i, c) in e.iter().enumerate() {
        let s = c.as_str().and_then(|s| s.strip_prefix('#')).filter(|s| s.len() == 6).ok_or(format!("Eintrag {i}: '#rrggbb' erwartet"))?;
        for k in 0..3 { let v = u8::from_str_radix(&s[2 * k..2 * k + 2], 16).map_err(|_| format!("Eintrag {i}: ungültige Farbe '#{s}'"))?; ch[k].push(v.rotate_left(4)); }
    }
    let [r, g, b] = ch; Ok(vec![("red".into(), r), ("green".into(), g), ("blue".into(), b)])
}
