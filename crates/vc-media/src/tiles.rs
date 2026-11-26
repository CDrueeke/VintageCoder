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
    let code = |x: &Value| x.as_u64().filter(|n| *n <= 255).map(|n| n as u8).ok_or_else(|| vc_i18n::t!("media-zeichencode-ausserhalb-0-255").to_string());
    match v {
        Value::Number(_) => Ok(Cell { code: code(v)?, cs: 0, col: None, sel: false }),
        Value::Array(a) => Ok(Cell { code: code(a.first().ok_or("leere Zelle")?)?, cs: a.get(1).and_then(|x| x.as_u64()).unwrap_or(0) as usize, col: a.get(2).and_then(|x| x.as_u64()).map(|n| n as u8), sel: false }),
        Value::Object(_) => Ok(Cell { code: code(&v["c"])?, cs: v["s"].as_u64().unwrap_or(0) as usize, col: v["col"].as_u64().map(|n| n as u8), sel: v["sel"].as_bool().unwrap_or(false) }),
        _ => Err(vc_i18n::t!("media-zelle-zahl-liste-oder-objekt-erwartet").into()),
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

pub fn parse_tileset(v: &Value) -> Result<TileSet, String> { parse_tileset_max(v, if TilesLayout::aus_medium(v).index16 { 65535 } else { MAX_TILES }) }

/// Wie [`parse_tileset`] mit ausdrücklicher Obergrenze der Kachelzahl (16-Bit-Index: 65535)
pub fn parse_tileset_max(v: &Value, max: usize) -> Result<TileSet, String> {
    if v["kind"] != "tileset" { return Err(vc_i18n::t!("media-kachelsatz-erwartet-kind-tileset-gefunden-p1", p1 = v["kind"].as_str().unwrap_or("?"))); }
    let (tw, th) = (v["size"]["w"].as_u64().unwrap_or(0) as usize, v["size"]["h"].as_u64().unwrap_or(0) as usize);
    if !(1..=MAX_SIDE).contains(&tw) || !(1..=MAX_SIDE).contains(&th) { return Err(vc_i18n::t!("media-kachelgroesse-tw-th-breite-und-hoehe", tw = &tw, th = &th, max_side = MAX_SIDE)); }
    let color_mode = match v["colorMode"].as_str().unwrap_or("tile") { "tile" => ColorMode::Tile, "cell" => ColorMode::Cell, "none" => ColorMode::None, m => return Err(vc_i18n::t!("media-colormode-unbekannt", m = &m)) };
    let charsets: Vec<String> = v["charsets"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let prop_defs: Vec<(String, u8)> = v["propDefs"].as_array().map(|a| a.iter().filter_map(|d| Some((d["name"].as_str()?.to_string(), d["bit"].as_u64()? as u8))).collect()).unwrap_or_default();
    if let Some((n, _)) = prop_defs.iter().find(|(_, b)| *b > 7) { return Err(vc_i18n::t!("media-eigenschaft-n-bit-muss-0-7", n = &n)); }
    let pins = v["pins"].as_array().map(|a| a.iter().filter_map(|p| Some((p["cs"].as_u64().unwrap_or(0) as usize, p["from"].as_u64()? as u8, p["to"].as_u64()? as u8))).collect()).unwrap_or_default();
    let arr = v["tiles"].as_array().ok_or(vc_i18n::t!("media-tiles-fehlt"))?;
    if arr.len() > max { return Err(vc_i18n::t!("media-p1-kacheln-mit-p2-bit-index", p1 = arr.len(), p2 = if max > MAX_TILES { 16 } else { 8 }, max = &max)); }
    let mut tiles = vec![];
    for (i, t) in arr.iter().enumerate() {
        let cells: Vec<Cell> = t["cells"].as_array().ok_or(vc_i18n::t!("media-kachel-i-cells-fehlt", i = &i))?.iter().map(parse_cell).collect::<Result<_, _>>().map_err(|e| vc_i18n::t!("media-kachel-i-e", i = &i, e = &e))?;
        if cells.len() != tw * th { return Err(vc_i18n::t!("media-kachel-i-zellen-anzahl", i = &i, p1 = cells.len(), p2 = tw * th)); }
        if let Some(c) = cells.iter().find(|c| !charsets.is_empty() && c.cs >= charsets.len()) { return Err(vc_i18n::t!("media-kachel-i-zeichensatz-p2-gibt-es", i = &i, p2 = &c.cs, p3 = charsets.len())); }
        let mut props = 0u8;
        for p in t["props"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            let n = p.as_str().ok_or(vc_i18n::t!("media-kachel-i-props-enthaelt-keinen-namen", i = &i))?;
            let (_, bit) = prop_defs.iter().find(|(d, _)| d == n).ok_or(vc_i18n::t!("media-kachel-i-eigenschaft-n-ist-in", i = &i, n = &n))?;
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
    if paths.is_empty() { return Err(vc_i18n::t!("media-kachelsatz-hat-keinen-zeichensatz-charsets-ist").into()); }
    let (mut all, mut prof) = (vec![], String::new());
    for p in paths {
        let text = std::fs::read_to_string(root.join(p)).map_err(|e| vc_i18n::t!("media-zeichensatz-p-e", p = &p, e = e.to_string()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| vc_i18n::t!("media-zeichensatz-p-e", p = &p, e = e.to_string()))?;
        if v["kind"] != "charset" { return Err(vc_i18n::t!("media-p-ist-kein-zeichensatz", p = &p)); }
        let id = v["profile"].as_str().unwrap_or("").to_string();
        if prof.is_empty() { prof = id; } else if prof != id { return Err(vc_i18n::t!("media-zeichensaetze-mit-verschiedenen-profilen-prof-und", prof = &prof, id = &id)); }
        let frames = v["frames"].as_array().ok_or(vc_i18n::t!("media-p-frames-fehlt", p = &p))?;
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
        sets.get(cs).ok_or(vc_i18n::t!("media-zeichensatz-cs-gibt-es-nicht", cs = &cs))?.get(code as usize).cloned().ok_or(vc_i18n::t!("media-zeichensatz-cs-hat-kein-zeichen-code", cs = &cs, code = &code))
    };
    let mut by_rows: HashMap<Vec<String>, u8> = HashMap::new();
    let mut slots: Vec<Option<Vec<String>>> = vec![None; max];
    let mut remap: HashMap<(usize, u8), u8> = HashMap::new();
    // 1. Pins
    for &(cs, from, to) in pins {
        for code in from..=to {
            if code as usize >= max { return Err(vc_i18n::t!("media-pin-code-ziel-hat-nur-max", code = &code, max = &max)); }
            let r = rows_of(cs, code)?;
            match &slots[code as usize] { Some(x) if *x != r => return Err(vc_i18n::t!("media-pin-konflikt-bei-code-code-zwei", code = &code)), _ => {} }
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
        if next >= max { return Err(vc_i18n::t!("media-zeichensatz-ueberlauf-mehr-als-max-verschiedene", max = &max)); }
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
        let t = *self.remap.get(&(c.cs, c.code)).ok_or(vc_i18n::t!("media-zeichen-p1-p2-fehlt-in-der", p1 = &c.cs, p2 = &c.code))?;
        if c.sel { if self.max > 128 { return Err(vc_i18n::t!("media-farbwahl-bit-sel-braucht-einen-zielzeichensatz").into()); } Ok(t | 0x80) } else { Ok(t) }
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
    if !is_bank(v) { return Err(vc_i18n::t!("media-screen-bank-erwartet-kind-screen-mit").into()); }
    let tilesets: Vec<String> = v["tilesets"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let mut screens = vec![];
    for (i, s) in v["screens"].as_array().unwrap().iter().enumerate() {
        let name = s["name"].as_str().unwrap_or("").to_string();
        let (w, h) = (s["w"].as_u64().unwrap_or(0) as usize, s["h"].as_u64().unwrap_or(0) as usize);
        if w == 0 || h == 0 || w > 255 || h > 255 { return Err(vc_i18n::t!("media-screen-i-name-groesse-w-h", i = &i, name = &name, w = &w, h = &h)); }
        let t = &s["tiles"];
        let set = t["set"].as_u64().unwrap_or(0) as usize;
        let origin = (t["origin"][0].as_u64().unwrap_or(0) as usize, t["origin"][1].as_u64().unwrap_or(0) as usize);
        let grid: Vec<Vec<usize>> = t["grid"].as_array().map(|rows| rows.iter().map(|r| r.as_array().map(|c| c.iter().map(|x| x.as_u64().unwrap_or(0) as usize).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        if let Some(r) = grid.iter().find(|r| r.len() != grid[0].len()) { return Err(vc_i18n::t!("media-screen-i-name-kartenzeilen-verschieden-lang", i = &i, name = &name, p3 = grid[0].len(), p4 = r.len())); }
        let mut patches = vec![];
        for p in s["cells"]["patches"].as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
            let a = p.as_array().ok_or(vc_i18n::t!("media-screen-i-patch-ist-keine-liste", i = &i))?;
            if a.len() < 3 { return Err(format!("Screen {i}: Patch braucht [x, y, zelle]")); }
            let (x, y) = (a[0].as_u64().unwrap_or(u64::MAX) as usize, a[1].as_u64().unwrap_or(u64::MAX) as usize);
            if x >= w || y >= h { return Err(vc_i18n::t!("media-screen-i-name-patch-x-y", i = &i, name = &name, x = &x, y = &y, w = &w, h = &h)); }
            patches.push(Patch { x, y, cell: parse_cell(&a[2]).map_err(|e| format!("Screen {i}: {e}"))? });
        }
        screens.push(Screen { name, w, h, colors: s["colors"].clone(), set, origin, grid, patches });
    }
    Ok(Bank { name: v["name"].as_str().unwrap_or("screens").into(), tilesets, screens })
}

fn validate_screen(i: usize, s: &Screen, ts: &TileSet) -> Result<(), String> {
    let (rows, cols) = (s.grid.len(), s.grid.first().map_or(0, |r| r.len()));
    if s.origin.0 + cols * ts.tw > s.w || s.origin.1 + rows * ts.th > s.h {
        return Err(vc_i18n::t!("media-screen-i-p2-kachelraster-cols-rows", i = &i, p2 = &s.name, cols = &cols, rows = &rows, p5 = s.origin.0, p6 = s.origin.1, p7 = &ts.tw, p8 = &ts.th, p9 = &s.w, p10 = &s.h));
    }
    for (y, r) in s.grid.iter().enumerate() { for (x, &t) in r.iter().enumerate() { if t >= ts.tiles.len() { return Err(vc_i18n::t!("media-screen-i-p2-karte-x-y", i = &i, p2 = &s.name, x = &x, y = &y, t = &t, p6 = ts.tiles.len())); } } }
    Ok(())
}

// ---------------------------------------------------------------------------------------------- Export

/// Layout der exportierten Tabellen (Konzept Kachel/Screen, Abschnitt 5.2). Vorgabe: planar, Karte zeilenweise, 8-Bit-Index (so exportierte bisher alles).
/// Steht in der Mediendatei unter `"layout": { "tiles": "planar" | "interleaved", "map": "rows" | "columns", "index": 8 | 16 }` oder kommt vom Export-Dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilesLayout { /** Kachel-Tabellen: eine Tabelle je Zelle (false) oder B×H Byte je Kachel hintereinander (true) */ pub interleaved: bool, /** Karte spaltenweise statt zeilenweise */ pub map_columns: bool, /** 16-Bit-Kachelindex: Karte als zwei Ebenen (niederes und oberes Byte) */ pub index16: bool }
impl Default for TilesLayout { fn default() -> Self { TilesLayout { interleaved: false, map_columns: false, index16: false } } }
impl TilesLayout {
    pub fn aus_medium(v: &Value) -> TilesLayout {
        let l = &v["layout"]; TilesLayout { interleaved: l["tiles"] == "interleaved", map_columns: l["map"] == "columns", index16: l["index"].as_u64() == Some(16) }
    }
    pub fn ist_vorgabe(&self) -> bool { *self == TilesLayout::default() }
    /// Als JSON für die Mediendatei (leer, wenn alles Vorgabe ist)
    pub fn json(&self) -> Option<Value> {
        if self.ist_vorgabe() { return None }
        Some(json!({ "tiles": if self.interleaved { "interleaved" } else { "planar" }, "map": if self.map_columns { "columns" } else { "rows" }, "index": if self.index16 { 16 } else { 8 } }))
    }
    /// Kurzer Name für Dialog und Bericht
    pub fn text(&self) -> String { format!("{} · Karte {} · {}-Bit-Index", if self.interleaved { "interleaved" } else { "planar" }, if self.map_columns { "spaltenweise" } else { "zeilenweise" }, if self.index16 { 16 } else { 8 }) }
}

/// Alles, was `project::export` für Kachelsätze und Screen-Bänke braucht
pub struct TilesOut { pub parts: Vec<(String, Vec<u8>)>, pub consts: String, pub consts_sbas: String, pub tail_asm: String, pub report: String, /** erzeugte `tiles_use`-Zeile (nur Screen-Bänke; leer bei Kachelsätzen) */ pub tiles_use: String }

/// Lauflängenkodierung einer Karte (Export-Option `"rle": true` der Screen-Bank): Paare *Kachelnummer, Anzahl* (Anzahl 1 bis 255) über die ganze Karte
/// (zeilenübergreifend, ohne Endemarke; der Entpacker der Bibliothek `screen_load_rle` kennt Breite mal Höhe). Gemessen an Jumprun: 320 statt 960 Byte.
pub fn rle_encode(map: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < map.len() {
        let mut j = i;
        while j < map.len() && map[j] == map[i] && j - i < 255 { j += 1; }
        out.push(map[i]);
        out.push((j - i) as u8);
        i = j;
    }
    out
}

/// Umkehrung von [`rle_encode`] (für Tests und Studios); `len` = Breite mal Höhe.
pub fn rle_decode(data: &[u8], len: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(len);
    for p in data.chunks(2) {
        if p.len() != 2 || p[1] == 0 { return Err(vc_i18n::t!("media-rle-daten-paar-unvollstaendig-oder-anzahl").into()); }
        out.extend(std::iter::repeat(p[0]).take(p[1] as usize));
    }
    if out.len() != len { return Err(vc_i18n::t!("media-rle-daten-ergeben-anzahl", p1 = out.len(), len = &len)); }
    Ok(out)
}

fn ident(s: &str) -> String { s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect() }

fn charset_profile<'a>(profiles: &'a [Profile], id: &str) -> Result<&'a Profile, String> { crate::profile(profiles, id) }

fn tileset_tables(ts: &TileSet, m: &Merge, lay: TilesLayout) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut parts = vec![]; let n = ts.tw * ts.th;
    if lay.interleaved {
        let mut b = Vec::with_capacity(ts.tiles.len() * n);
        for t in &ts.tiles { for k in 0..n { b.push(m.code(&t.cells[k])?) } }
        parts.push(("t".to_string(), b));
    } else {
        for k in 0..n { parts.push((format!("c{k}"), ts.tiles.iter().map(|t| m.code(&t.cells[k])).collect::<Result<Vec<u8>, _>>()?)); }
    }
    match ts.color_mode {
        ColorMode::Tile => parts.push(("col".into(), ts.tiles.iter().map(|t| t.col).collect())),
        ColorMode::Cell if lay.interleaved => parts.push(("colt".into(), ts.tiles.iter().flat_map(|t| (0..n).map(move |k| t.cells[k].col.unwrap_or(t.col))).collect())),
        ColorMode::Cell => for k in 0..n { parts.push((format!("col{k}"), ts.tiles.iter().map(|t| t.cells[k].col.unwrap_or(t.col)).collect())); },
        ColorMode::None => {}
    }
    if !ts.prop_defs.is_empty() { parts.push(("props".into(), ts.tiles.iter().map(|t| t.props).collect())); }
    Ok(parts)
}

fn tileset_consts(label: &str, ts: &TileSet, m: &Merge, sbas: bool, lay: TilesLayout) -> String {
    let c = if sbas { "const" } else { ".const" };
    let mut s = format!("{c} {label}_tw = {}\n{c} {label}_th = {}\n{c} {label}_count = {}\n{c} {label}_glyphs = {}\n", ts.tw, ts.th, ts.tiles.len(), m.used);
    for (n, bit) in &ts.prop_defs { s.push_str(&format!("{c} {label}_p_{} = {}\n", ident(n), 1u32 << bit)); }
    if lay.interleaved { s.push_str(&format!("{c} {label}_interleaved = 1\n")) }
    s
}

/// Exportteile für einen Kachelsatz (Art `tileset`) oder eine Screen-Bank (`kind: screen` mit `screens`).
pub fn export_parts(root: &Path, media: &Value, profiles: &[Profile], label: &str) -> Result<TilesOut, String> { export_parts_mit(root, media, profiles, label, TilesLayout::aus_medium(media)) }

/// Wie [`export_parts`], aber mit ausdrücklich gewähltem Layout (Export-Dialog)
pub fn export_parts_mit(root: &Path, media: &Value, profiles: &[Profile], label: &str, lay: TilesLayout) -> Result<TilesOut, String> {
    if media["kind"] == "tileset" {
        let ts = parse_tileset_max(media, if lay.index16 { 65535 } else { MAX_TILES })?;
        let (sets, cpid) = load_charsets(root, &ts.charsets)?; let cp = charset_profile(profiles, &cpid)?;
        let m = merge_glyphs(&ordered_cells(&ts, &[]), &sets, &ts.pins, cp)?;
        let mut parts = tileset_tables(&ts, &m, lay)?;
        parts.push(("chars".into(), m.charset_bytes(cp)?));
        let report = vc_i18n::t!("media-p1-kacheln-p2-p3-zeichensatz-fuellstand", p1 = ts.tiles.len(), p2 = &ts.tw, p3 = &ts.th, p4 = &m.used, p5 = &m.max, p6 = &m.doubles);
        return Ok(TilesOut { parts, consts: tileset_consts(label, &ts, &m, false, lay), consts_sbas: tileset_consts(label, &ts, &m, true, lay), tail_asm: String::new(), report, tiles_use: String::new() });
    }
    let bank = parse_bank(media)?;
    if bank.tilesets.is_empty() { return Err(vc_i18n::t!("media-screen-bank-braucht-mindestens-einen-kachelsatz").into()); }
    // Alle Kachelsätze der Bank: Zeichensatzlisten aneinanderhängen (cs-Nummern je Satz verschieben), damit EIN Zielzeichensatz entsteht (Konzept 3.2)
    let (mut tss, mut alle_zs, mut offs, mut cpid): (Vec<TileSet>, Vec<Frames>, Vec<usize>, String) = (vec![], vec![], vec![], String::new());
    for rel in &bank.tilesets {
        let ts_text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("Kachelsatz {rel} (gesucht in {}): {e}", root.display()))?;
        let mut ts = parse_tileset_max(&serde_json::from_str(&ts_text).map_err(|e| format!("Kachelsatz {rel}: {e}"))?, if lay.index16 { 65535 } else { MAX_TILES })?;
        let (zs, id) = load_charsets(root, &ts.charsets)?;
        if cpid.is_empty() { cpid = id } else if cpid != id { return Err(vc_i18n::t!("media-kachelsatz-rel-zeichensatzprofil-id-passt-nicht", rel = &rel, id = &id, cpid = &cpid)) }
        let off = alle_zs.len(); offs.push(off); alle_zs.extend(zs);
        for t in &mut ts.tiles { for c in &mut t.cells { c.cs += off } } for p in &mut ts.pins { p.0 += off }
        tss.push(ts);
    }
    if let Some(sc) = bank.screens.iter().find(|s| s.set >= tss.len()) { return Err(vc_i18n::t!("media-screen-p1-kachelsatz-p2-gibt-es", p1 = &sc.name, p2 = &sc.set, p3 = tss.len())) }
    let cp = charset_profile(profiles, &cpid)?; let sets = alle_zs;
    for (i, s) in bank.screens.iter().enumerate() { validate_screen(i, s, &tss[s.set])?; }
    let sh = |c: &Cell, k: usize| Cell { cs: c.cs + offs[k], ..c.clone() };
    let mut gesehen = std::collections::HashSet::new(); let mut zellen: Vec<(usize, u8)> = vec![];
    for c in tss.iter().flat_map(|t| t.tiles.iter().flat_map(|t| t.cells.iter().cloned())).chain(bank.screens.iter().flat_map(|s| s.patches.iter().map(|p| sh(&p.cell, s.set)))) { if gesehen.insert((c.cs, c.code)) { zellen.push((c.cs, c.code)) } }
    let pins: Vec<(usize, u8, u8)> = tss.iter().flat_map(|t| t.pins.iter().copied()).collect();
    let m = merge_glyphs(&zellen, &sets, &pins, cp)?;
    let rle = media["rle"].as_bool().unwrap_or(false);
    let (mut parts, mut consts, mut sb) = (vec![], String::new(), String::new());
    let (mut map_labels, mut cell_labels, mut hi_labels, mut row_labels): (Vec<String>, Vec<Option<String>>, Vec<String>, Vec<(usize, String)>) = (vec![], vec![], vec![], vec![]);
    for (i, s) in bank.screens.iter().enumerate() {
        let ts = &tss[s.set];
        if !lay.index16 { if let Some(t) = s.grid.iter().flatten().find(|&&t| t > 255) { return Err(vc_i18n::t!("media-screen-p1-kachel-t-passt-nicht", p1 = &s.name, t = &t)) } }
        let reihenfolge: Vec<usize> = if lay.map_columns { let (rows, cols) = (s.grid.len(), s.grid.first().map_or(0, |r| r.len())); (0..cols).flat_map(|x| (0..rows).map(move |y| s.grid[y][x])).collect() } else { s.grid.iter().flatten().copied().collect() };
        let raw: Vec<u8> = reihenfolge.iter().map(|&t| (t & 0xFF) as u8).collect();
        parts.push((format!("map{i}"), if rle { rle_encode(&raw) } else { raw }));
        if lay.index16 { parts.push((format!("map{i}_hi"), reihenfolge.iter().map(|&t| (t >> 8) as u8).collect())); hi_labels.push(format!("{label}_map{i}_hi")); }
        map_labels.push(format!("{label}_map{i}"));
        if s.patches.is_empty() { cell_labels.push(None); } else {
            if s.patches.len() > 255 {
                // volle Zeilen statt Patches: jede betroffene Zeile als Ganzes (Kachelebene eingerechnet, Patches darüber); Zeile = y, w Codes, bei Farbe w Farben
                let mut ys: Vec<usize> = s.patches.iter().map(|p| p.y).collect(); ys.sort_unstable(); ys.dedup();
                let farbig = ts.color_mode != ColorMode::None; let mut b = vec![ys.len() as u8];
                if ys.len() > 255 { return Err(vc_i18n::t!("media-screen-p1-mehr-als-255-zeilen", p1 = &s.name)) }
                for &y in &ys {
                    b.push(y as u8); let (mut codes, mut cols) = (Vec::with_capacity(s.w), Vec::with_capacity(s.w));
                    for x in 0..s.w {
                        if let Some(p) = s.patches.iter().rev().find(|p| p.x == x && p.y == y) { codes.push(m.code(&sh(&p.cell, s.set))?); cols.push(p.cell.col.unwrap_or(ts.tiles.first().map_or(1, |t| t.col))); continue }
                        let (gx, gy) = (x.checked_sub(s.origin.0), y.checked_sub(s.origin.1));
                        let zelle = gx.zip(gy).and_then(|(gx, gy)| { let t = *s.grid.get(gy / ts.th)?.get(gx / ts.tw)?; let t = ts.tiles.get(t)?; Some((m.code(&t.cells[(gy % ts.th) * ts.tw + gx % ts.tw]).ok()?, t.cells[(gy % ts.th) * ts.tw + gx % ts.tw].col.unwrap_or(t.col))) });
                        let (c, col) = zelle.unwrap_or((0, s.colors["fg"].as_u64().unwrap_or(1) as u8)); codes.push(c); cols.push(col);
                    }
                    b.extend(codes); if farbig { b.extend(cols) }
                }
                parts.push((format!("rows{i}"), b)); row_labels.push((i, format!("{label}_rows{i}"))); cell_labels.push(None); continue;
            }
            let mut b = vec![s.patches.len() as u8];
            for p in &s.patches { b.push(p.x as u8); b.push(p.y as u8); b.push(m.code(&sh(&p.cell, s.set))?); if ts.color_mode != ColorMode::None { b.push(p.cell.col.unwrap_or(ts.tiles.first().map_or(1, |t| t.col))); } }
            parts.push((format!("cells{i}"), b)); cell_labels.push(Some(format!("{label}_cells{i}")));
        }
        let n = ident(&s.name); let n = if n.is_empty() { format!("s{i}") } else { n };
        for (c, dst) in [(".const", &mut consts), ("const", &mut sb)] {
            dst.push_str(&format!("{c} {label}_{n}_id = {i}\n{c} {label}_{n}_w = {}\n{c} {label}_{n}_h = {}\n{c} {label}_{n}_tiles_w = {}\n{c} {label}_{n}_tiles_h = {}\n{c} {label}_{n}_origin_x = {}\n{c} {label}_{n}_origin_y = {}\n", s.w, s.h, s.grid.first().map_or(0, |r| r.len()), s.grid.len(), s.origin.0, s.origin.1));
            for k in ["border", "bg", "fg"] { if let Some(v) = s.colors[k].as_u64() { dst.push_str(&format!("{c} {label}_{n}_{k} = {v}\n")); } }
        }
    }
    let mut tail = String::new();
    if tss.len() > 1 {
        // mehrere Sätze: ihre Tabellen mit den zusammengeführten Zeichencodes gehören in den Bank-Export (ein Satz-Export allein hätte eigene Codes)
        for (k, t) in tss.iter().enumerate() {
            for (suf, b) in tileset_tables(t, &m, lay)? { parts.push((format!("s{k}_{suf}"), b)) }
            consts.push_str(&tileset_consts(&format!("{label}_s{k}"), t, &m, false, lay)); sb.push_str(&tileset_consts(&format!("{label}_s{k}"), t, &m, true, lay));
        }
        let zuordnung: Vec<String> = bank.screens.iter().map(|s| s.set.to_string()).collect();
        consts.push_str(&format!(".const {label}_sets = {}\n", tss.len())); sb.push_str(&format!("const {label}_sets = {}\n", tss.len()));
        tail.push_str(&format!("{label}_screen_set:\n        .byte {}\n", zuordnung.join(",")));
    }
    parts.push(("chars".into(), m.charset_bytes(cp)?));
    consts.push_str(&format!(".const {label}_count = {}\n.const {label}_glyphs = {}\n", bank.screens.len(), m.used));
    sb.push_str(&format!("const {label}_count = {}\nconst {label}_glyphs = {}\n", bank.screens.len(), m.used));
    if rle { consts.push_str(&format!(".const {label}_rle = 1\n")); sb.push_str(&format!("const {label}_rle = 1\n")); }
    let join = |ls: Vec<String>, op: &str| ls.into_iter().map(|l| format!("{op}{l}")).collect::<Vec<_>>().join(",");
    if !map_labels.is_empty() {
        tail.push_str(&format!("{label}_screens_lo:\n        .byte {}\n{label}_screens_hi:\n        .byte {}\n", join(map_labels.clone(), "<"), join(map_labels.clone(), ">")));
        if lay.index16 { tail.push_str(&format!("{label}_screens_hp_lo:\n        .byte {}\n{label}_screens_hp_hi:\n        .byte {}\n", join(hi_labels.clone(), "<"), join(hi_labels, ">"))); }
        let (lo, hi): (Vec<String>, Vec<String>) = cell_labels.iter().map(|l| match l { Some(l) => (format!("<{l}"), format!(">{l}")), None => ("0".into(), "0".into()) }).unzip();
        tail.push_str(&format!("{label}_cells_lo:\n        .byte {}\n{label}_cells_hi:\n        .byte {}\n", lo.join(","), hi.join(",")));
        if !row_labels.is_empty() {
            let (rl, rh): (Vec<String>, Vec<String>) = (0..bank.screens.len()).map(|i| match row_labels.iter().find(|(j, _)| *j == i) { Some((_, l)) => (format!("<{l}"), format!(">{l}")), None => ("0".into(), "0".into()) }).unzip();
            tail.push_str(&format!("{label}_rows_lo:\n        .byte {}\n{label}_rows_hi:\n        .byte {}\n", rl.join(","), rh.join(",")));
        }
    }
    if lay.map_columns { consts.push_str(&format!(".const {label}_map_columns = 1\n")); sb.push_str(&format!("const {label}_map_columns = 1\n")); }
    if lay.index16 { consts.push_str(&format!(".const {label}_index16 = 1\n")); sb.push_str(&format!("const {label}_index16 = 1\n")); }
    let report = vc_i18n::t!("media-p1-screens-zeichensatz-fuellstand-p2-von", p1 = bank.screens.len(), p2 = &m.used, p3 = &m.max, p4 = if lay.ist_vorgabe() { String::new() } else { format!(", Layout {}", lay.text()) });
    let ts = &tss[0]; let satz = if tss.len() > 1 { format!("{label}_s0") } else { ident(ts.name.as_str()) }; let erste = bank.screens.first().map(|s| { let n = ident(&s.name); if n.is_empty() { "s0".to_string() } else { n } }).unwrap_or_default();
    let (c0, col, props) = (if lay.interleaved { format!("{satz}_t") } else { format!("{satz}_c0") }, if ts.color_mode == ColorMode::None { "0".to_string() } else if ts.color_mode == ColorMode::Cell { if lay.interleaved { format!("{satz}_colt") } else { format!("{satz}_col0") } } else { format!("{satz}_col") }, if ts.prop_defs.is_empty() { "0".to_string() } else { format!("{satz}_props") });
    let tiles_use = format!("tiles_use {c0}, {col}, {props}, {satz}_tw, {satz}_th, {satz}_count, {label}_screens_lo, {label}_screens_hi, {label}_{erste}_tiles_w, {label}_{erste}_tiles_h");
    Ok(TilesOut { parts, consts, consts_sbas: sb, tail_asm: tail, report, tiles_use })
}

// ---------------------------------------------------------------------------------------------- Umsetzer alte Kachelkarte → Satz + Bank

/// Wandelt eine alte Kachelkarte (`kind: tilemap`, Kacheln mit `chars`/`color`/`solid`, `map`) in einen Kachelsatz und einen Screen um.
/// `tw`/`th` stammen aus dem alten Profil (`tileWidth`/`tileHeight`), `color_ram` aus `colorRam`.
pub fn migrate_tilemap(old: &Value, tw: usize, th: usize, color_ram: bool, set_name: &str) -> Result<(Value, Value), String> {
    if old["kind"] != "tilemap" { return Err(vc_i18n::t!("media-alte-kachelkarte-erwartet").into()); }
    let tiles: Vec<Value> = old["tiles"].as_array().ok_or(vc_i18n::t!("media-tiles-fehlt"))?.iter().map(|t| {
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
