// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachelsätze (`.vctiles`, Art `tileset`) und Screen-Bänke (`.vcscreen` mit `screens`): Modell, Zeichensatz-Zusammenführung, Export.
//!
//! Ein **Kachelsatz** besteht aus n Kacheln gleicher Größe B×H. Jede Zelle nennt Zeichencode und Zeichensatz (Index in `charsets`);
//! beim Export entsteht aus allen benutzten Glyphen **ein** Zielzeichensatz (gleiche Muster zählen einmal, Umnummerierung).
//! Eine **Screen-Bank** enthält einen oder mehrere Screens aus Kachelebene (Raster von Kachelnummern) und Zellebene (freie Zellen).
//! Zellschreibweise in der Datei: Zahl (Code, Zeichensatz 0), `[code, zeichensatz]` oder `{ "c": code, "s": zeichensatz, "col": farbe, "sel": true }`.
//! Konzept: `assets/docs/Konzept-Kachel-und-Screen-Editor.md`.
use crate::{encode, Media, Profile};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

pub const MAX_TILES: usize = 256;
pub const MAX_SIDE: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell { pub code: u8, pub cs: usize, pub col: Option<u8>, pub sel: bool }

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ColorMode { Tile, Cell, None }

#[derive(Debug, Clone)]
pub struct Tile { pub name: String, pub cells: Vec<Cell>, pub col: u8, pub props: u8 }

#[derive(Debug, Clone)]
pub struct TileSet {
    pub name: String, pub tw: usize, pub th: usize, pub color_mode: ColorMode, pub charsets: Vec<String>,
    pub tiles: Vec<Tile>, pub prop_defs: Vec<(String, u8)>, pub pins: Vec<(usize, u8, u8)>,
}

fn parse_cell(v: &Value) -> Result<Cell, String> {
    let code = |x: &Value| x.as_u64().filter(|n| *n <= 255).map(|n| n as u8).ok_or_else(|| "Zeichencode außerhalb 0..255".to_string());
    match v {
        Value::Number(_) => Ok(Cell { code: code(v)?, cs: 0, col: None, sel: false }),
        Value::Array(a) => Ok(Cell { code: code(a.first().ok_or("leere Zelle")?)?, cs: a.get(1).and_then(|x| x.as_u64()).unwrap_or(0) as usize, col: a.get(2).and_then(|x| x.as_u64()).map(|n| n as u8), sel: false }),
        Value::Object(_) => Ok(Cell { code: code(&v["c"])?, cs: v["s"].as_u64().unwrap_or(0) as usize, col: v["col"].as_u64().map(|n| n as u8), sel: v["sel"].as_bool().unwrap_or(false) }),
        _ => Err("Zelle: Zahl, Liste oder Objekt erwartet".into()),
    }
}

/// Zelle in die kürzeste Dateischreibweise
pub fn cell_json(c: &Cell) -> Value {
    if c.col.is_none() && !c.sel { return if c.cs == 0 { json!(c.code) } else { json!([c.code, c.cs]) }; }
    let mut o = json!({ "c": c.code });
    if c.cs != 0 { o["s"] = json!(c.cs); }
    if let Some(col) = c.col { o["col"] = json!(col); }
    if c.sel { o["sel"] = json!(true); }
    o
}

pub fn parse_tileset(v: &Value) -> Result<TileSet, String> {
    if v["kind"] != "tileset" { return Err(format!("Kachelsatz erwartet (kind 'tileset'), gefunden '{}'", v["kind"].as_str().unwrap_or("?"))); }
    let (tw, th) = (v["size"]["w"].as_u64().unwrap_or(0) as usize, v["size"]["h"].as_u64().unwrap_or(0) as usize);
    if !(1..=MAX_SIDE).contains(&tw) || !(1..=MAX_SIDE).contains(&th) { return Err(format!("Kachelgröße {tw}×{th}: Breite und Höhe müssen 1..{MAX_SIDE} sein")); }
    let color_mode = match v["colorMode"].as_str().unwrap_or("tile") { "tile" => ColorMode::Tile, "cell" => ColorMode::Cell, "none" => ColorMode::None, m => return Err(format!("colorMode '{m}' unbekannt (tile, cell, none)")) };
    let charsets: Vec<String> = v["charsets"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let prop_defs: Vec<(String, u8)> = v["propDefs"].as_array().map(|a| a.iter().filter_map(|d| Some((d["name"].as_str()?.to_string(), d["bit"].as_u64()? as u8))).collect()).unwrap_or_default();
    if let Some((n, _)) = prop_defs.iter().find(|(_, b)| *b > 7) { return Err(format!("Eigenschaft '{n}': Bit muss 0..7 sein")); }
    let pins = v["pins"].as_array().map(|a| a.iter().filter_map(|p| Some((p["cs"].as_u64().unwrap_or(0) as usize, p["from"].as_u64()? as u8, p["to"].as_u64()? as u8))).collect()).unwrap_or_default();
    let arr = v["tiles"].as_array().ok_or("tiles fehlt")?;
    if arr.len() > MAX_TILES { return Err(format!("{} Kacheln: mit 8-Bit-Index sind höchstens {MAX_TILES} möglich", arr.len())); }
    let mut tiles = vec![];
    for (i, t) in arr.iter().enumerate() {
        let cells: Vec<Cell> = t["cells"].as_array().ok_or(format!("Kachel {i}: cells fehlt"))?.iter().map(parse_cell).collect::<Result<_, _>>().map_err(|e| format!("Kachel {i}: {e}"))?;
        if cells.len() != tw * th { return Err(format!("Kachel {i}: {} statt {} Zellen", cells.len(), tw * th)); }
        if let Some(c) = cells.iter().find(|c| !charsets.is_empty() && c.cs >= charsets.len()) { return Err(format!("Kachel {i}: Zeichensatz {} gibt es nicht (charsets hat {})", c.cs, charsets.len())); }
        let mut props = 0u8;
        for p in t["props"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            let n = p.as_str().ok_or(format!("Kachel {i}: props enthält keinen Namen"))?;
            let (_, bit) = prop_defs.iter().find(|(d, _)| d == n).ok_or(format!("Kachel {i}: Eigenschaft '{n}' ist in propDefs nicht definiert"))?;
            props |= 1 << bit;
        }
        tiles.push(Tile { name: t["name"].as_str().unwrap_or("").into(), cells, col: t["col"].as_u64().unwrap_or(1) as u8, props });
    }
    Ok(TileSet { name: v["name"].as_str().unwrap_or("tiles").into(), tw, th, color_mode, charsets, tiles, prop_defs, pins })
}

impl ColorMode {
    pub fn as_str(self) -> &'static str { match self { ColorMode::Tile => "tile", ColorMode::Cell => "cell", ColorMode::None => "none" } }
}

/// Schreibt den Kachelsatz zurück in die Dateischreibweise; alle anderen Schlüssel von `base` (format, profile, name, pins …) bleiben erhalten.
pub fn tileset_json(ts: &TileSet, base: &Value) -> Value {
    let mut v = base.clone();
    v["size"] = json!({ "w": ts.tw, "h": ts.th });
    v["colorMode"] = json!(ts.color_mode.as_str());
    v["charsets"] = json!(ts.charsets);
    v["propDefs"] = Value::Array(ts.prop_defs.iter().map(|(n, b)| json!({ "name": n, "bit": b })).collect());
    v["tiles"] = Value::Array(ts.tiles.iter().map(|t| {
        let mut o = json!({ "name": t.name, "cells": t.cells.iter().map(cell_json).collect::<Vec<_>>(), "col": t.col });
        let props: Vec<&str> = ts.prop_defs.iter().filter(|(_, b)| t.props >> b & 1 == 1).map(|(n, _)| n.as_str()).collect();
        if !props.is_empty() { o["props"] = json!(props); }
        o
    }).collect());
    v
}

// ---------------------------------------------------------------------------------------------- Zeichensatz-Zusammenführung

/// Ergebnis der Zusammenführung: Umnummerierung `(zeichensatz, code) → Zielcode` und der Zielzeichensatz (Zeilen je Glyph, `count` Einträge).
#[derive(Debug, Clone)]
pub struct Merge { pub remap: HashMap<(usize, u8), u8>, pub frames: Vec<Vec<String>>, pub used: usize, pub max: usize, pub doubles: usize, pub charset_profile: String }

pub type Frames = Vec<Vec<String>>;

/// Lädt die Zeichensätze eines Kachelsatzes (Pfade relativ zum Projektordner); alle müssen dasselbe Profil haben.
pub fn load_charsets(root: &Path, paths: &[String]) -> Result<(Vec<Frames>, String), String> {
    if paths.is_empty() { return Err("Kachelsatz hat keinen Zeichensatz (charsets ist leer)".into()); }
    let (mut all, mut prof) = (vec![], String::new());
    for p in paths {
        let text = std::fs::read_to_string(root.join(p)).map_err(|e| format!("Zeichensatz {p}: {e}"))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("Zeichensatz {p}: {e}"))?;
        if v["kind"] != "charset" { return Err(format!("{p} ist kein Zeichensatz")); }
        let id = v["profile"].as_str().unwrap_or("").to_string();
        if prof.is_empty() { prof = id; } else if prof != id { return Err(format!("Zeichensätze mit verschiedenen Profilen ('{prof}' und '{id}') lassen sich nicht zusammenführen")); }
        let frames = v["frames"].as_array().ok_or(format!("{p}: frames fehlt"))?;
        all.push(frames.iter().map(|f| f["rows"].as_array().map(|r| r.iter().filter_map(|s| s.as_str().map(String::from)).collect()).unwrap_or_default()).collect());
    }
    Ok((all, prof))
}

/// Führt die benutzten Glyphen zusammen. `cells` in Reihenfolge des ersten Auftretens; Glyph 'leer' (nur Nullen) bekommt Code 0, wenn er vorkommt
/// und 0 nicht angepinnt ist; angepinnte Codes behalten ihre Nummer.
pub fn merge_glyphs(cells: &[(usize, u8)], sets: &[Frames], pins: &[(usize, u8, u8)], cp: &Profile) -> Result<Merge, String> {
    let max = cp.count;
    let blank: Vec<String> = (0..cp.height).map(|_| "0".repeat(cp.width * cp.raw["digits"].as_u64().unwrap_or(1) as usize)).collect();
    let rows_of = |cs: usize, code: u8| -> Result<Vec<String>, String> {
        sets.get(cs).ok_or(format!("Zeichensatz {cs} gibt es nicht"))?.get(code as usize).cloned().ok_or(format!("Zeichensatz {cs} hat kein Zeichen {code}"))
    };
    let mut by_rows: HashMap<Vec<String>, u8> = HashMap::new();
    let mut slots: Vec<Option<Vec<String>>> = vec![None; max];
    let mut remap: HashMap<(usize, u8), u8> = HashMap::new();
    // 1. Pins
    for &(cs, from, to) in pins {
        for code in from..=to {
            if code as usize >= max { return Err(format!("Pin {code}: Ziel hat nur {max} Zeichen")); }
            let r = rows_of(cs, code)?;
            match &slots[code as usize] { Some(x) if *x != r => return Err(format!("Pin-Konflikt bei Code {code}: zwei verschiedene Glyphen")), _ => {} }
            slots[code as usize] = Some(r.clone()); by_rows.entry(r).or_insert(code); remap.insert((cs, code), code);
        }
    }
    // 2. leeres Glyph auf 0
    let uses_blank = cells.iter().any(|&(cs, code)| rows_of(cs, code).map_or(false, |r| r == blank));
    if uses_blank && slots[0].is_none() && !by_rows.contains_key(&blank) { slots[0] = Some(blank.clone()); by_rows.insert(blank.clone(), 0); }
    // 3. übrige in Reihenfolge des Auftretens
    let mut next = 0usize;
    let mut distinct = std::collections::HashSet::new();
    for &(cs, code) in cells {
        distinct.insert((cs, code));
        if remap.contains_key(&(cs, code)) { continue; }
        let r = rows_of(cs, code)?;
        if let Some(&t) = by_rows.get(&r) { remap.insert((cs, code), t); continue; }
        while next < max && slots[next].is_some() { next += 1; }
        if next >= max { return Err(format!("Zeichensatz-Überlauf: mehr als {max} verschiedene Glyphen (Ziel fasst {max})")); }
        slots[next] = Some(r.clone()); by_rows.insert(r, next as u8); remap.insert((cs, code), next as u8);
    }
    let used = slots.iter().filter(|s| s.is_some()).count();
    let frames = slots.into_iter().map(|s| s.unwrap_or_else(|| blank.clone())).collect();
    let doubles = distinct.len().saturating_sub(by_rows.len().min(distinct.len()));
    Ok(Merge { remap, frames, used, max, doubles, charset_profile: cp.id.clone() })
}

impl Merge {
    /// Zielcode einer Zelle, bei `sel` mit gesetztem Bit 7 (nur zulässig, wenn der Zielzeichensatz höchstens 128 Zeichen hat)
    pub fn code(&self, c: &Cell) -> Result<u8, String> {
        let t = *self.remap.get(&(c.cs, c.code)).ok_or(format!("Zeichen {}:{} fehlt in der Zusammenführung", c.cs, c.code))?;
        if c.sel { if self.max > 128 { return Err("Farbwahl-Bit (sel) braucht einen Zielzeichensatz mit höchstens 128 Zeichen (Atari)".into()); } Ok(t | 0x80) } else { Ok(t) }
    }
    fn charset_bytes(&self, cp: &Profile) -> Result<Vec<u8>, String> {
        let frames: Vec<Value> = self.frames.iter().map(|r| json!({ "rows": r })).collect();
        encode(&Media { kind: "charset".into(), profile: cp.id.clone(), name: String::new(), root: json!({ "frames": frames }) }, cp)
    }
}

fn ordered_cells(ts: &TileSet, extra: &[Cell]) -> Vec<(usize, u8)> {
    let mut seen = std::collections::HashSet::new(); let mut out = vec![];
    for c in ts.tiles.iter().flat_map(|t| t.cells.iter()).chain(extra.iter()) { if seen.insert((c.cs, c.code)) { out.push((c.cs, c.code)); } }
    out
}

// ---------------------------------------------------------------------------------------------- Screen-Bank

#[derive(Debug, Clone)]
pub struct Patch { pub x: usize, pub y: usize, pub cell: Cell }
#[derive(Debug, Clone)]
pub struct Screen { pub name: String, pub w: usize, pub h: usize, pub colors: Value, pub set: usize, pub origin: (usize, usize), pub grid: Vec<Vec<usize>>, pub patches: Vec<Patch> }
#[derive(Debug, Clone)]
pub struct Bank { pub name: String, pub tilesets: Vec<String>, pub screens: Vec<Screen> }

pub fn is_bank(v: &Value) -> bool { v["kind"] == "screen" && v["screens"].is_array() }

pub fn parse_bank(v: &Value) -> Result<Bank, String> {
    if !is_bank(v) { return Err("Screen-Bank erwartet (kind 'screen' mit 'screens')".into()); }
    let tilesets: Vec<String> = v["tilesets"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let mut screens = vec![];
    for (i, s) in v["screens"].as_array().unwrap().iter().enumerate() {
        let name = s["name"].as_str().unwrap_or("").to_string();
        let (w, h) = (s["w"].as_u64().unwrap_or(0) as usize, s["h"].as_u64().unwrap_or(0) as usize);
        if w == 0 || h == 0 || w > 255 || h > 255 { return Err(format!("Screen {i} '{name}': Größe {w}×{h} ungültig (1..255)")); }
        let t = &s["tiles"];
        let set = t["set"].as_u64().unwrap_or(0) as usize;
        let origin = (t["origin"][0].as_u64().unwrap_or(0) as usize, t["origin"][1].as_u64().unwrap_or(0) as usize);
        let grid: Vec<Vec<usize>> = t["grid"].as_array().map(|rows| rows.iter().map(|r| r.as_array().map(|c| c.iter().map(|x| x.as_u64().unwrap_or(0) as usize).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        if let Some(r) = grid.iter().find(|r| r.len() != grid[0].len()) { return Err(format!("Screen {i} '{name}': Kartenzeilen verschieden lang ({} und {})", grid[0].len(), r.len())); }
        let mut patches = vec![];
        for p in s["cells"]["patches"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            let a = p.as_array().ok_or(format!("Screen {i}: Patch ist keine Liste [x, y, zelle]"))?;
            if a.len() < 3 { return Err(format!("Screen {i}: Patch braucht [x, y, zelle]")); }
            let (x, y) = (a[0].as_u64().unwrap_or(u64::MAX) as usize, a[1].as_u64().unwrap_or(u64::MAX) as usize);
            if x >= w || y >= h { return Err(format!("Screen {i} '{name}': Patch ({x}, {y}) liegt außerhalb {w}×{h}")); }
            patches.push(Patch { x, y, cell: parse_cell(&a[2]).map_err(|e| format!("Screen {i}: {e}"))? });
        }
        screens.push(Screen { name, w, h, colors: s["colors"].clone(), set, origin, grid, patches });
    }
    Ok(Bank { name: v["name"].as_str().unwrap_or("screens").into(), tilesets, screens })
}

fn validate_screen(i: usize, s: &Screen, ts: &TileSet) -> Result<(), String> {
    let (rows, cols) = (s.grid.len(), s.grid.first().map_or(0, |r| r.len()));
    if s.origin.0 + cols * ts.tw > s.w || s.origin.1 + rows * ts.th > s.h {
        return Err(format!("Screen {i} '{}': Kachelraster {cols}×{rows} ab ({}, {}) mit {}×{}-Kacheln passt nicht in {}×{} Zellen", s.name, s.origin.0, s.origin.1, ts.tw, ts.th, s.w, s.h));
    }
    for (y, r) in s.grid.iter().enumerate() { for (x, &t) in r.iter().enumerate() { if t >= ts.tiles.len() { return Err(format!("Screen {i} '{}': Karte ({x}, {y}) verweist auf Kachel {t}, der Satz hat nur {}", s.name, ts.tiles.len())); } } }
    Ok(())
}

// ---------------------------------------------------------------------------------------------- Export

/// Alles, was `project::export` für Kachelsätze und Screen-Bänke braucht
pub struct TilesOut { pub parts: Vec<(String, Vec<u8>)>, pub consts: String, pub consts_sbas: String, pub tail_asm: String, pub report: String }

fn ident(s: &str) -> String { s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect() }

fn charset_profile<'a>(profiles: &'a [Profile], id: &str) -> Result<&'a Profile, String> { crate::profile(profiles, id) }

fn tileset_tables(ts: &TileSet, m: &Merge) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut parts = vec![];
    for k in 0..ts.tw * ts.th {
        parts.push((format!("c{k}"), ts.tiles.iter().map(|t| m.code(&t.cells[k])).collect::<Result<Vec<u8>, _>>()?));
    }
    match ts.color_mode {
        ColorMode::Tile => parts.push(("col".into(), ts.tiles.iter().map(|t| t.col).collect())),
        ColorMode::Cell => for k in 0..ts.tw * ts.th { parts.push((format!("col{k}"), ts.tiles.iter().map(|t| t.cells[k].col.unwrap_or(t.col)).collect())); },
        ColorMode::None => {}
    }
    if !ts.prop_defs.is_empty() { parts.push(("props".into(), ts.tiles.iter().map(|t| t.props).collect())); }
    Ok(parts)
}

fn tileset_consts(label: &str, ts: &TileSet, m: &Merge, sbas: bool) -> String {
    let c = if sbas { "const" } else { ".const" };
    let mut s = format!("{c} {label}_tw = {}\n{c} {label}_th = {}\n{c} {label}_count = {}\n{c} {label}_glyphs = {}\n", ts.tw, ts.th, ts.tiles.len(), m.used);
    for (n, bit) in &ts.prop_defs { s.push_str(&format!("{c} {label}_p_{} = {}\n", ident(n), 1u32 << bit)); }
    s
}

/// Exportteile für einen Kachelsatz (Art `tileset`) oder eine Screen-Bank (`kind: screen` mit `screens`).
pub fn export_parts(root: &Path, media: &Value, profiles: &[Profile], label: &str) -> Result<TilesOut, String> {
    if media["kind"] == "tileset" {
        let ts = parse_tileset(media)?;
        let (sets, cpid) = load_charsets(root, &ts.charsets)?; let cp = charset_profile(profiles, &cpid)?;
        let m = merge_glyphs(&ordered_cells(&ts, &[]), &sets, &ts.pins, cp)?;
        let mut parts = tileset_tables(&ts, &m)?;
        parts.push(("chars".into(), m.charset_bytes(cp)?));
        let report = format!("{} Kacheln {}×{}, Zeichensatz-Füllstand {} von {} Glyphen, {} Doppelte zusammengelegt", ts.tiles.len(), ts.tw, ts.th, m.used, m.max, m.doubles);
        return Ok(TilesOut { parts, consts: tileset_consts(label, &ts, &m, false), consts_sbas: tileset_consts(label, &ts, &m, true), tail_asm: String::new(), report });
    }
    let bank = parse_bank(media)?;
    if bank.tilesets.len() != 1 { return Err(format!("Screen-Bank braucht genau einen Kachelsatz in 'tilesets' (gefunden {}); mehrere Sätze je Bank sind noch nicht unterstützt", bank.tilesets.len())); }
    if let Some(s) = bank.screens.iter().find(|s| s.set != 0) { return Err(format!("Screen '{}': nur Kachelsatz 0 ist unterstützt", s.name)); }
    let ts_text = std::fs::read_to_string(root.join(&bank.tilesets[0])).map_err(|e| format!("Kachelsatz {}: {e}", bank.tilesets[0]))?;
    let ts = parse_tileset(&serde_json::from_str(&ts_text).map_err(|e| format!("Kachelsatz {}: {e}", bank.tilesets[0]))?)?;
    let (sets, cpid) = load_charsets(root, &ts.charsets)?; let cp = charset_profile(profiles, &cpid)?;
    for (i, s) in bank.screens.iter().enumerate() { validate_screen(i, s, &ts)?; }
    let extra: Vec<Cell> = bank.screens.iter().flat_map(|s| s.patches.iter().map(|p| p.cell.clone())).collect();
    let m = merge_glyphs(&ordered_cells(&ts, &extra), &sets, &ts.pins, cp)?;
    let (mut parts, mut consts, mut sb) = (vec![], String::new(), String::new());
    let (mut map_labels, mut cell_labels) = (vec![], vec![]);
    for (i, s) in bank.screens.iter().enumerate() {
        parts.push((format!("map{i}"), s.grid.iter().flat_map(|r| r.iter().map(|&t| t as u8)).collect()));
        map_labels.push(format!("{label}_map{i}"));
        if s.patches.is_empty() { cell_labels.push(None); } else {
            if s.patches.len() > 255 { return Err(format!("Screen '{}': mehr als 255 Zellen-Patches; volle Zeilen sind noch nicht unterstützt", s.name)); }
            let mut b = vec![s.patches.len() as u8];
            for p in &s.patches { b.push(p.x as u8); b.push(p.y as u8); b.push(m.code(&p.cell)?); if ts.color_mode != ColorMode::None { b.push(p.cell.col.unwrap_or(ts.tiles.first().map_or(1, |t| t.col))); } }
            parts.push((format!("cells{i}"), b)); cell_labels.push(Some(format!("{label}_cells{i}")));
        }
        let n = ident(&s.name); let n = if n.is_empty() { format!("s{i}") } else { n };
        for (c, dst) in [(".const", &mut consts), ("const", &mut sb)] {
            dst.push_str(&format!("{c} {label}_{n}_id = {i}\n{c} {label}_{n}_w = {}\n{c} {label}_{n}_h = {}\n{c} {label}_{n}_tiles_w = {}\n{c} {label}_{n}_tiles_h = {}\n{c} {label}_{n}_origin_x = {}\n{c} {label}_{n}_origin_y = {}\n", s.w, s.h, s.grid.first().map_or(0, |r| r.len()), s.grid.len(), s.origin.0, s.origin.1));
            for k in ["border", "bg", "fg"] { if let Some(v) = s.colors[k].as_u64() { dst.push_str(&format!("{c} {label}_{n}_{k} = {v}\n")); } }
        }
    }
    parts.push(("chars".into(), m.charset_bytes(cp)?));
    consts.push_str(&format!(".const {label}_count = {}\n.const {label}_glyphs = {}\n", bank.screens.len(), m.used));
    sb.push_str(&format!("const {label}_count = {}\nconst {label}_glyphs = {}\n", bank.screens.len(), m.used));
    let join = |ls: Vec<String>, op: &str| ls.into_iter().map(|l| format!("{op}{l}")).collect::<Vec<_>>().join(",");
    let mut tail = String::new();
    if !map_labels.is_empty() {
        tail.push_str(&format!("{label}_screens_lo:\n        .byte {}\n{label}_screens_hi:\n        .byte {}\n", join(map_labels.clone(), "<"), join(map_labels, ">")));
        let (lo, hi): (Vec<String>, Vec<String>) = cell_labels.iter().map(|l| match l { Some(l) => (format!("<{l}"), format!(">{l}")), None => ("0".into(), "0".into()) }).unzip();
        tail.push_str(&format!("{label}_cells_lo:\n        .byte {}\n{label}_cells_hi:\n        .byte {}\n", lo.join(","), hi.join(",")));
    }
    let report = format!("{} Screens, Zeichensatz-Füllstand {} von {} Glyphen", bank.screens.len(), m.used, m.max);
    Ok(TilesOut { parts, consts, consts_sbas: sb, tail_asm: tail, report })
}

// ---------------------------------------------------------------------------------------------- Umsetzer alte Kachelkarte → Satz + Bank

/// Wandelt eine alte Kachelkarte (`kind: tilemap`, Kacheln mit `chars`/`color`/`solid`, `map`) in einen Kachelsatz und einen Screen um.
/// `tw`/`th` stammen aus dem alten Profil (`tileWidth`/`tileHeight`), `color_ram` aus `colorRam`.
pub fn migrate_tilemap(old: &Value, tw: usize, th: usize, color_ram: bool, set_name: &str) -> Result<(Value, Value), String> {
    if old["kind"] != "tilemap" { return Err("alte Kachelkarte (kind 'tilemap') erwartet".into()); }
    let tiles: Vec<Value> = old["tiles"].as_array().ok_or("tiles fehlt")?.iter().map(|t| {
        let mut o = json!({ "name": t["name"], "cells": t["chars"].as_array().map(|a| a.iter().map(|c| json!(c)).collect::<Vec<_>>()).unwrap_or_default() });
        if color_ram { o["col"] = t["color"].clone(); }
        if t["solid"].as_bool().unwrap_or(false) { o["props"] = json!(["solid"]); }
        o
    }).collect();
    let charsets: Vec<Value> = old["charset"].as_str().map(|c| vec![json!(c)]).unwrap_or_default();
    let set = json!({ "format": 1, "kind": "tileset", "profile": old["profile"].as_str().map(|p| p.replace("tilemap.2x2", "tileset")).unwrap_or_default(), "name": set_name,
        "size": { "w": tw, "h": th }, "colorMode": if color_ram { "tile" } else { "none" }, "charsets": charsets,
        "propDefs": [{ "name": "solid", "bit": 0 }], "tiles": tiles });
    let grid = old["map"].clone();
    let (rows, cols) = (grid.as_array().map_or(0, |r| r.len()), grid[0].as_array().map_or(0, |r| r.len()));
    let screen = json!({ "name": old["name"], "w": cols * tw, "h": rows * th, "colors": old["colors"], "tiles": { "set": 0, "origin": [0, 0], "grid": grid } });
    Ok((set, screen))
}
