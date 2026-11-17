// SPDX-License-Identifier: GPL-3.0-or-later
//! Screen-Bank (`.vcscreen` mit `screens`) ohne Oberfläche: Kachelebene (Raster von Kachelnummern) und Zellebene (freie Zellen, „Patches“), mehrere
//! Screens je Bank, Werkzeuge (Stift, Linie, Rechteck, Füllen, Pipette, Auswahl, Stempel, Text, Löschen), Kachel aus Auswahl anlegen, Kachelsatz wählen,
//! Rückgängig, Speichern, Bild. Das Dateimodell ist `vc_media::tiles`.
//! Alte Textbildschirme (`chars`/`attrs`) lassen sich mit [`decompose`] in Kachelsatz und Bank umwandeln.
use crate::glyphs::{bildschirmcode, Bild, Glyphs, McCols};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_media::tiles::{self, Bank, Cell, ColorMode, Patch, Screen, Tile, TileSet};

/// Höchstzahl der Zellen-Patches je Screen (Zähler im Export ein Byte)
pub const MAX_PATCHES: usize = 255;
/// Platzhalter in einem Stempel: keine Kachel an dieser Stelle
pub const HOLE: usize = usize::MAX;

// ---------------------------------------------------------------------------------------------- Dateischreibweise

fn screen_json(s: &Screen) -> Value {
    let mut o = json!({ "name": s.name, "w": s.w, "h": s.h, "tiles": { "set": s.set, "origin": [s.origin.0, s.origin.1], "grid": s.grid } });
    if !s.colors.is_null() { o["colors"] = s.colors.clone(); }
    if !s.patches.is_empty() { o["cells"] = json!({ "patches": s.patches.iter().map(|p| json!([p.x, p.y, tiles::cell_json(&p.cell)])).collect::<Vec<_>>() }); }
    o
}

/// Schreibt die Bank zurück in die Dateischreibweise; alle anderen Schlüssel von `base` (format, profile, name …) bleiben erhalten.
pub fn bank_json(b: &Bank, base: &Value) -> Value {
    let mut v = base.clone();
    v["tilesets"] = json!(b.tilesets);
    v["screens"] = Value::Array(b.screens.iter().map(screen_json).collect());
    v
}

// ---------------------------------------------------------------------------------------------- Rechnen auf Screens (frei, testbar)

/// Kachelraster, das in den Screen passt (Spalten, Zeilen); der Rest sind Restzellen der Zellebene
pub fn grid_dims(s: &Screen, ts: &TileSet) -> (usize, usize) { (s.w.saturating_sub(s.origin.0) / ts.tw, s.h.saturating_sub(s.origin.1) / ts.th) }

/// Kachelraster auf die passende Größe bringen (neue Felder bekommen Kachel 0, überstehende fallen weg)
pub fn fit_grid(s: &mut Screen, ts: &TileSet) {
    let (c, r) = grid_dims(s, ts);
    s.grid.resize(r, vec![]);
    for row in s.grid.iter_mut() { row.resize(c, 0); }
}

pub fn default_col(ts: &TileSet) -> u8 { ts.tiles.first().map_or(1, |t| t.col) }

/// Kachel und Zelle, die das Kachelraster an Zelle (x, y) liefert
pub fn tile_cell<'a>(s: &Screen, ts: &'a TileSet, x: usize, y: usize) -> Option<(&'a Tile, &'a Cell)> {
    let (ox, oy) = s.origin;
    if x < ox || y < oy { return None; }
    let (gx, gy) = ((x - ox) / ts.tw, (y - oy) / ts.th);
    let t = ts.tiles.get(*s.grid.get(gy)?.get(gx)?)?;
    t.cells.get(((y - oy) % ts.th) * ts.tw + (x - ox) % ts.tw).map(|c| (t, c))
}

/// Farbnummer, mit der eine Zelle erscheint
pub fn color_index(ts: &TileSet, s: &Screen, t: Option<&Tile>, c: &Cell) -> usize {
    match ts.color_mode {
        ColorMode::None => s.colors["fg"].as_u64().unwrap_or(14) as usize,
        ColorMode::Tile => t.map_or_else(|| c.col.unwrap_or(default_col(ts)), |t| t.col) as usize,
        ColorMode::Cell => c.col.or(t.map(|t| t.col)).unwrap_or(default_col(ts)) as usize,
    }
}

pub fn patch_at(s: &Screen, x: usize, y: usize) -> Option<usize> { s.patches.iter().rposition(|p| p.x == x && p.y == y) }

/// Was an Zelle (x, y) zu sehen ist: Patch, sonst Kachelzelle
pub fn effective(s: &Screen, ts: &TileSet, x: usize, y: usize) -> Option<(Cell, usize)> {
    if let Some(i) = patch_at(s, x, y) { let c = s.patches[i].cell.clone(); let col = color_index(ts, s, None, &c); return Some((c, col)); }
    tile_cell(s, ts, x, y).map(|(t, c)| (c.clone(), color_index(ts, s, Some(t), c)))
}

/// Patch setzen; `false`, wenn das Höchstmaß erreicht ist
pub fn set_patch(s: &mut Screen, x: usize, y: usize, cell: Cell) -> bool {
    if x >= s.w || y >= s.h { return false; }
    if let Some(i) = patch_at(s, x, y) { s.patches[i].cell = cell; return true; }
    if s.patches.len() >= MAX_PATCHES { return false; }
    s.patches.push(Patch { x, y, cell }); true
}

pub fn remove_patch(s: &mut Screen, x: usize, y: usize) -> bool { let n = s.patches.len(); s.patches.retain(|p| !(p.x == x && p.y == y)); s.patches.len() != n }

/// Zellen der Linie von a nach b (Bresenham)
pub fn line_points(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (a.0 as i32, a.1 as i32);
    let (x1, y1) = (b.0 as i32, b.1 as i32);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let (mut err, mut out) = (dx + dy, vec![]);
    loop {
        out.push((x as usize, y as usize));
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
    out
}

/// Zellen eines Rechtecks (nur Rahmen oder gefüllt)
pub fn rect_points(a: (usize, usize), b: (usize, usize), filled: bool) -> Vec<(usize, usize)> {
    let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1));
    let mut v = vec![];
    for y in y0..=y1 { for x in x0..=x1 { if filled || x == x0 || x == x1 || y == y0 || y == y1 { v.push((x, y)); } } }
    v
}

/// Zusammenhängende gleiche Kacheln durch `to` ersetzen
pub fn flood_tiles(grid: &mut [Vec<usize>], x: usize, y: usize, to: usize) {
    let Some(from) = grid.get(y).and_then(|r| r.get(x)).copied() else { return };
    if from == to { return; }
    let mut st = vec![(x, y)];
    while let Some((px, py)) = st.pop() {
        if grid.get(py).and_then(|r| r.get(px)) != Some(&from) { continue; }
        grid[py][px] = to;
        if px > 0 { st.push((px - 1, py)); } if py > 0 { st.push((px, py - 1)); }
        st.push((px + 1, py)); st.push((px, py + 1));
    }
}

/// Zellen und Kachelfarbe des Blocks ab (x, y) in Kachelgröße, so wie sie im Screen zu sehen sind
pub fn block_cells(s: &Screen, ts: &TileSet, x: usize, y: usize) -> Option<(Vec<Cell>, u8)> {
    if x + ts.tw > s.w || y + ts.th > s.h { return None; }
    let (mut cells, mut first) = (vec![], None);
    for dy in 0..ts.th { for dx in 0..ts.tw {
        let (c, col) = effective(s, ts, x + dx, y + dy).unwrap_or((Cell { code: 0, cs: 0, col: None, sel: false }, default_col(ts) as usize));
        first.get_or_insert(col);
        cells.push(Cell { col: (ts.color_mode == ColorMode::Cell).then_some(col as u8), ..c });
    } }
    Some((cells, first.unwrap_or(1) as u8))
}

/// Gibt es schon eine Kachel mit diesen Zellen (und, bei Kachelfarbe, dieser Farbe)?
pub fn find_dup(ts: &TileSet, cells: &[Cell], col: u8) -> Option<usize> {
    ts.tiles.iter().position(|t| {
        let norm: Vec<Cell> = t.cells.iter().map(|c| Cell { col: (ts.color_mode == ColorMode::Cell).then(|| c.col.unwrap_or(t.col)), ..c.clone() }).collect();
        norm == cells && (ts.color_mode != ColorMode::Tile || t.col == col)
    })
}

/// Kachel `idx` anstelle der Zellen ab (x, y) einsetzen: nur bei Ausrichtung auf das Kachelraster; entfernt die Patches im Block
pub fn replace_block(s: &mut Screen, ts: &TileSet, x: usize, y: usize, idx: usize) -> bool {
    let (ox, oy) = s.origin;
    if x < ox || y < oy || (x - ox) % ts.tw != 0 || (y - oy) % ts.th != 0 { return false; }
    let (gx, gy) = ((x - ox) / ts.tw, (y - oy) / ts.th);
    let Some(slot) = s.grid.get_mut(gy).and_then(|r| r.get_mut(gx)) else { return false };
    *slot = idx;
    s.patches.retain(|p| !(p.x >= x && p.x < x + ts.tw && p.y >= y && p.y < y + ts.th));
    true
}

// ---------------------------------------------------------------------------------------------- In Kacheln zerlegen (alter Zeichen-Screen)

/// Ergebnis von „In Kacheln zerlegen“: Kachelsatz und Bank als Dateiinhalt, dazu Zahlen für die Vorschau
pub struct Decomp { pub set: Value, pub bank: Value, pub tiles: usize, pub patches: usize, pub bytes_before: usize, pub bytes_after: usize, pub color_mode: ColorMode }

/// Zerlegt einen alten Textbildschirm (`chars`, `attrs`) in Kacheln `tw`×`th` ab `origin`; gleiche Blöcke werden eine Kachel.
/// Zellen im Rest (rechts, unten, vor dem Ursprung) und Zellen, die ein Patch brauchen, landen in der Zellebene.
pub fn decompose(old: &Value, p: &vc_media::Profile, tileset_profile: &str, tw: usize, th: usize, origin: (usize, usize), name: &str) -> Result<Decomp, String> {
    if old["kind"] != "screen" || old["screens"].is_array() { return Err("alter Zeichen-Screen erwartet".into()); }
    if !(1..=tiles::MAX_SIDE).contains(&tw) || !(1..=tiles::MAX_SIDE).contains(&th) { return Err("Kachelgröße 1..16".into()); }
    let rows = old["chars"].as_array().ok_or("chars fehlt")?;
    let (h, w) = (rows.len(), rows.first().and_then(|r| r.as_array()).map_or(0, |r| r.len()));
    if w == 0 || h == 0 { return Err("leerer Bildschirm".into()); }
    let color_ram = p.raw["colorRam"].as_bool().unwrap_or(false);
    let blank = bildschirmcode(&p.target, ' ');
    let code = |x: usize, y: usize| old["chars"][y][x].as_u64().unwrap_or(blank as u64) as u8;
    let attr = |x: usize, y: usize| old["attrs"][y][x].as_u64().unwrap_or(1) as u8;
    let (cols, nrows) = (w.saturating_sub(origin.0) / tw, h.saturating_sub(origin.1) / th);
    // Farbmodus: je Kachel, wenn jeder Block (ohne Leerzeichen) einfarbig ist, sonst je Zelle
    let mixed = color_ram && (0..nrows).any(|gy| (0..cols).any(|gx| {
        let mut cs = (0..tw * th).filter_map(|k| { let (x, y) = (origin.0 + gx * tw + k % tw, origin.1 + gy * th + k / tw); (code(x, y) != blank).then(|| attr(x, y)) });
        let f = cs.next(); cs.any(|c| Some(c) != f)
    }));
    let mode = if !color_ram { ColorMode::None } else if mixed { ColorMode::Cell } else { ColorMode::Tile };
    let mut tiles_v: Vec<Tile> = vec![];
    let mut grid = vec![vec![0usize; cols]; nrows];
    for gy in 0..nrows { for gx in 0..cols {
        let pos = |k: usize| (origin.0 + gx * tw + k % tw, origin.1 + gy * th + k / tw);
        let first_col = (0..tw * th).map(|k| pos(k)).find(|&(x, y)| code(x, y) != blank).map_or(1, |(x, y)| attr(x, y));
        let cells: Vec<Cell> = (0..tw * th).map(|k| { let (x, y) = pos(k); Cell { code: code(x, y), cs: 0, col: (mode == ColorMode::Cell).then(|| if code(x, y) == blank { first_col } else { attr(x, y) }), sel: false } }).collect();
        let col = if mode == ColorMode::None { 1 } else { first_col };
        let idx = match tiles_v.iter().position(|t| t.cells == cells && (mode != ColorMode::Tile || t.col == col)) {
            Some(i) => i,
            None => { tiles_v.push(Tile { name: String::new(), cells, col, props: 0 }); tiles_v.len() - 1 }
        };
        grid[gy][gx] = idx;
    } }
    if tiles_v.len() > tiles::MAX_TILES { return Err(format!("{} verschiedene Kacheln: mit 8-Bit-Index sind höchstens {} möglich; größere Kacheln wählen", tiles_v.len(), tiles::MAX_TILES)); }
    let mut patches = vec![];
    for y in 0..h { for x in 0..w {
        let covered = x >= origin.0 && y >= origin.1 && (x - origin.0) / tw < cols && (y - origin.1) / th < nrows;
        if !covered && code(x, y) != blank { patches.push(Patch { x, y, cell: Cell { code: code(x, y), cs: 0, col: color_ram.then(|| attr(x, y)), sel: false } }); }
    } }
    if patches.len() > MAX_PATCHES { return Err(format!("{} Zellen außerhalb des Kachelrasters: mehr als {MAX_PATCHES} Patches; anderen Ursprung oder andere Größe wählen", patches.len())); }
    let charsets: Vec<String> = old["charset"].as_str().map(|c| vec![c.to_string()]).unwrap_or_default();
    let ts = TileSet { name: name.into(), tw, th, color_mode: mode, charsets, tiles: tiles_v, prop_defs: vec![("solid".into(), 0)], pins: vec![] };
    let set = tiles::tileset_json(&ts, &json!({ "format": 1, "kind": "tileset", "profile": tileset_profile, "name": name }));
    let screen = Screen { name: old["name"].as_str().unwrap_or(name).into(), w, h, colors: old["colors"].clone(), set: 0, origin, grid, patches };
    let (np, cell_bytes) = (screen.patches.len(), ts.tw * ts.th);
    let color_tables = match mode { ColorMode::Tile => ts.tiles.len(), ColorMode::Cell => ts.tiles.len() * cell_bytes, ColorMode::None => 0 };
    let after = ts.tiles.len() * cell_bytes + color_tables + cols * nrows + if np > 0 { 1 + np * (3 + color_ram as usize) } else { 0 };
    let bank = Bank { name: name.into(), tilesets: vec![], screens: vec![screen] };
    Ok(Decomp { set, bank: bank_json(&bank, &json!({ "format": 1, "kind": "screen", "profile": old["profile"], "name": name })), tiles: ts.tiles.len(), patches: np, bytes_before: w * h * (1 + color_ram as usize), bytes_after: after, color_mode: mode })
}

/// Schreibt Kachelsatz und Bank neben den alten Screen (`<name>_tiles.vctiles`, `<name>_bank.vcscreen`); vorhandene Dateien bleiben unberührt
pub fn write_decomp(root: &Path, rel_old: &str, d: &mut Decomp) -> Result<(String, String), String> {
    let (dir, stem) = (rel_old.rsplit_once('/').map_or("", |x| x.0), vc_media::naming::stem(rel_old).to_string());
    let join = |f: String| if dir.is_empty() { f } else { format!("{dir}/{f}") };
    let (set_rel, bank_rel) = (join(format!("{stem}_tiles.vctiles")), join(format!("{stem}_bank.vcscreen")));
    for r in [&set_rel, &bank_rel] { if root.join(r).exists() { return Err(format!("{r} gibt es schon; bitte erst umbenennen oder entfernen")); } }
    d.bank["tilesets"] = json!([set_rel]);
    std::fs::write(root.join(&set_rel), vc_media::project::format_media(&d.set)).map_err(|e| format!("{set_rel}: {e}"))?;
    std::fs::write(root.join(&bank_rel), vc_media::project::format_media(&d.bank)).map_err(|e| format!("{bank_rel}: {e}"))?;
    Ok((set_rel, bank_rel))
}

// ---------------------------------------------------------------------------------------------- Bank-Dokument

#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Modus { Kachel, Zelle }
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Werkzeug { Stift, Linie, Rechteck, Fuellen, Pipette, Auswahl, Stempel, Text, Loeschen }

/// Rückfrage „Auswahl → Kachel anlegen“
#[derive(Clone, Debug, PartialEq)]
pub struct NeueKachel { pub at: (usize, usize), pub cells: Vec<Cell>, pub col: u8, pub dup: Option<usize>, pub ausgerichtet: bool, pub ersetzen: bool, pub name: String }

type Auswahl = ((usize, usize), (usize, usize));

pub struct BankDoc {
    pub rel: String, abs: PathBuf, root: PathBuf, profiles: Vec<vc_media::Profile>,
    pub media: Value, pub bank: Bank, pub target: String, dim: (usize, usize),
    pub ts: Option<TileSet>, ts_base: Value, pub ts_fehler: Option<String>, pub ts_dirty: bool,
    /// Kachelsätze des Projekts für dieses System
    pub saetze: Vec<String>,
    pub glyphs: Vec<Option<Glyphs>>,
    undo: Vec<String>, redo: Vec<String>, saved: String, pub status: String,
    pub cur: usize, pub modus: Modus, pub werkzeug: Werkzeug,
    pub pinsel: usize, pub stempel: Option<Vec<Vec<usize>>>, pub zuletzt: Vec<usize>,
    pub zell_pinsel: Cell, pub reiter: usize, pub farbe: usize, pub gefuellt: bool, pub eigenschaft: Option<u8>,
    pub auswahl: Option<Auswahl>, /** Beginn eines Verschiebens der Auswahl (Werkzeug Auswahl, Zug innerhalb der Auswahl) */ pub verschiebe_von: Option<(usize, usize)>, pub von: Option<(usize, usize)>, letzte: Option<(usize, usize)>, pub text_pos: (usize, usize),
    pub neue_kachel: Option<NeueKachel>,
}

impl BankDoc {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<BankDoc, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| format!("{rel}: {e}"))?).map_err(|e| format!("{rel}: {e}"))?;
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, media["profile"].as_str().unwrap_or(""))?;
        let bank = tiles::parse_bank(&media).map_err(|e| format!("{rel}: {e}"))?;
        let saetze = Self::saetze_suchen(root, &p.target, &profiles);
        let mut d = BankDoc { rel: rel.into(), abs, root: root.into(), dim: (p.width, p.height), target: p.target.clone(), profiles, saved: media.to_string(), media, bank, ts: None, ts_base: Value::Null, ts_fehler: None, ts_dirty: false, saetze,
            glyphs: vec![], undo: vec![], redo: vec![], status: String::new(), cur: 0, modus: Modus::Kachel, werkzeug: Werkzeug::Stift, pinsel: 0, stempel: None, zuletzt: vec![], zell_pinsel: Cell { code: 1, cs: 0, col: None, sel: false }, reiter: 0, farbe: 1,
            gefuellt: false, eigenschaft: None, auswahl: None, verschiebe_von: None, von: None, letzte: None, text_pos: (0, 0), neue_kachel: None };
        d.satz_laden(); Ok(d)
    }

    fn saetze_suchen(root: &Path, ziel: &str, profiles: &[vc_media::Profile]) -> Vec<String> {
        fn lauf(root: &Path, dir: &Path, tiefe: usize, out: &mut Vec<String>) {
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            for e in rd.flatten() { let n = e.file_name().to_string_lossy().into_owned(); if n.starts_with('.') || n == "build" || n == "target" { continue } let p = e.path();
                if p.is_dir() { if tiefe < 4 { lauf(root, &p, tiefe + 1, out) } } else if n.ends_with(".vctiles") { if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")) } } }
        }
        let mut v = vec![]; lauf(root, root, 0, &mut v);
        v.retain(|rel| std::fs::read_to_string(root.join(rel)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).is_some_and(|w| w["kind"] == "tileset" && profiles.iter().any(|p| Some(p.id.as_str()) == w["profile"].as_str() && p.target == ziel)));
        v.sort(); v
    }

    /// Kachelsatz der Bank (erster Eintrag von `tilesets`) von der Platte lesen
    pub fn satz_laden(&mut self) {
        self.ts = None; self.ts_fehler = None; self.glyphs.clear();
        let Some(rel) = self.bank.tilesets.first().cloned() else { self.ts_fehler = Some("Die Bank nennt noch keinen Kachelsatz: rechts einen wählen.".into()); return };
        let r = std::fs::read_to_string(self.root.join(&rel)).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string())).and_then(|v| { let ts = tiles::parse_tileset(&v)?; Ok((v, ts)) });
        match r {
            Ok((v, ts)) => { self.glyphs = ts.charsets.iter().map(|p| Glyphs::laden(&self.root, p)).collect(); self.reiter = self.reiter.min(ts.charsets.len().saturating_sub(1)); self.pinsel = self.pinsel.min(ts.tiles.len().saturating_sub(1)); self.ts_base = v; self.ts = Some(ts); self.ts_dirty = false }
            Err(e) => self.ts_fehler = Some(format!("{rel}: {e}")),
        }
    }

    pub fn screen(&self) -> Option<&Screen> { self.bank.screens.get(self.cur) }
    fn screen_mut(&mut self) -> Option<&mut Screen> { self.bank.screens.get_mut(self.cur) }

    // -- Zustand --------------------------------------------------------------------------------------------------------
    pub fn dirty(&self) -> bool { self.media.to_string() != self.saved || self.ts_dirty }
    fn undo_punkt(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear() }
    fn uebernehmen(&mut self) { self.media = bank_json(&self.bank, &self.media) }
    fn wiederherstellen(&mut self, s: &str) {
        let Ok(v) = serde_json::from_str::<Value>(s) else { return }; let Ok(b) = tiles::parse_bank(&v) else { return };
        let satz_neu = b.tilesets != self.bank.tilesets; self.media = v; self.bank = b; if satz_neu { self.satz_laden() }
        self.cur = self.cur.min(self.bank.screens.len().saturating_sub(1)); self.auswahl = None; self.neue_kachel = None;
    }
    pub fn rueckgaengig(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); self.wiederherstellen(&s) } }
    pub fn wiederholen(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); self.wiederherstellen(&s) } }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    pub fn media_text(&self) -> String { vc_media::project::format_media(&self.media) }
    pub fn speichern(&mut self) -> Result<(), String> {
        if self.ts_dirty {
            if let (Some(ts), Some(rel)) = (&self.ts, self.bank.tilesets.first()) {
                let v = tiles::tileset_json(ts, &self.ts_base);
                std::fs::write(self.root.join(rel), vc_media::project::format_media(&v)).map_err(|e| format!("{rel}: {e}"))?; self.ts_base = v; self.ts_dirty = false;
            }
        }
        self.uebernehmen();
        std::fs::write(&self.abs, self.media_text()).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = "gespeichert".into(); Ok(())
    }

    // -- Screens --------------------------------------------------------------------------------------------------------
    fn eindeutiger_name(&self, base: &str) -> String {
        if !self.bank.screens.iter().any(|s| s.name == base) { return base.into() }
        (2..).map(|n| format!("{base}{n}")).find(|n| !self.bank.screens.iter().any(|s| &s.name == n)).unwrap_or_default()
    }
    pub fn screen_neu(&mut self) {
        self.undo_punkt();
        let (w, h) = self.screen().map_or(self.dim, |s| (s.w, s.h)); let colors = self.screen().map_or(json!({}), |s| s.colors.clone());
        let mut s = Screen { name: self.eindeutiger_name("screen"), w, h, colors, set: 0, origin: self.screen().map_or((0, 0), |s| s.origin), grid: vec![], patches: vec![] };
        if let Some(ts) = &self.ts { fit_grid(&mut s, ts) }
        self.bank.screens.push(s); self.cur = self.bank.screens.len() - 1; self.auswahl = None; self.uebernehmen()
    }
    pub fn screen_duplizieren(&mut self) {
        let Some(mut s) = self.screen().cloned() else { return };
        self.undo_punkt(); s.name = self.eindeutiger_name(&format!("{}_kopie", s.name)); self.bank.screens.insert(self.cur + 1, s); self.cur += 1; self.uebernehmen()
    }
    pub fn screen_loeschen(&mut self) {
        if self.bank.screens.len() < 2 { return }
        self.undo_punkt(); self.bank.screens.remove(self.cur); self.cur = self.cur.min(self.bank.screens.len() - 1); self.auswahl = None; self.uebernehmen()
    }
    pub fn screen_verschieben(&mut self, d: i32) {
        let nach = self.cur as i32 + d; if nach < 0 || nach as usize >= self.bank.screens.len() { return }
        self.undo_punkt(); self.bank.screens.swap(self.cur, nach as usize); self.cur = nach as usize; self.uebernehmen()
    }
    pub fn screen_waehlen(&mut self, i: usize) { if i < self.bank.screens.len() { self.cur = i; self.auswahl = None; self.neue_kachel = None } }
    pub fn screen_umbenennen(&mut self, name: &str, erster: bool) { if erster { self.undo_punkt() } if let Some(s) = self.screen_mut() { s.name = name.to_string() } self.uebernehmen() }
    /// Größe und Ursprung des Kachelrasters; `erster` = Rückgängig-Punkt
    pub fn screen_groesse(&mut self, w: usize, h: usize, ox: usize, oy: usize, erster: bool) {
        if erster { self.undo_punkt() }
        let ts = self.ts.clone();
        if let Some(s) = self.screen_mut() { let (w, h) = (w.clamp(1, 255), h.clamp(1, 255)); s.w = w; s.h = h; s.origin = (ox.min(w - 1), oy.min(h - 1)); s.patches.retain(|p| p.x < w && p.y < h); if let Some(ts) = &ts { fit_grid(s, ts) } }
        self.auswahl = None; self.uebernehmen()
    }
    pub fn screen_farbe(&mut self, rolle: &str, i: usize) { self.undo_punkt(); if let Some(s) = self.screen_mut() { s.colors[rolle] = json!(i) } self.uebernehmen() }
    /// Kachelsatz wählen: Raster aller Screens passt sich an, unbekannte Kacheln werden zu 0
    pub fn satz_waehlen(&mut self, rel: &str) {
        self.undo_punkt(); if self.bank.tilesets.is_empty() { self.bank.tilesets.push(rel.into()) } else { self.bank.tilesets[0] = rel.into() } self.satz_laden();
        if let Some(ts) = self.ts.clone() { let n = ts.tiles.len(); for sc in self.bank.screens.iter_mut() { fit_grid(sc, &ts); for r in sc.grid.iter_mut() { for t in r.iter_mut() { if *t >= n { *t = 0 } } } } }
        self.uebernehmen()
    }

    // -- Werkzeuge ------------------------------------------------------------------------------------------------------
    pub fn werkzeuge(&self) -> &'static [(Werkzeug, &'static str)] {
        match self.modus {
            Modus::Kachel => &[(Werkzeug::Stift, "Stift"), (Werkzeug::Linie, "Linie"), (Werkzeug::Rechteck, "Rechteck"), (Werkzeug::Fuellen, "Füllen"), (Werkzeug::Pipette, "Pipette"), (Werkzeug::Auswahl, "Auswahl"), (Werkzeug::Stempel, "Stempel"), (Werkzeug::Loeschen, "Löschen")],
            Modus::Zelle => &[(Werkzeug::Stift, "Zeichen"), (Werkzeug::Linie, "Linie"), (Werkzeug::Rechteck, "Rechteck"), (Werkzeug::Pipette, "Pipette"), (Werkzeug::Auswahl, "Auswahl"), (Werkzeug::Text, "Text"), (Werkzeug::Loeschen, "Löschen")],
        }
    }
    pub fn modus_waehlen(&mut self, m: Modus) {
        if self.modus == m { return }
        self.modus = m; self.auswahl = None; self.von = None; self.letzte = None; if !self.werkzeuge().iter().any(|(t, _)| *t == self.werkzeug) { self.werkzeug = Werkzeug::Stift }
    }
    pub fn werkzeug_waehlen(&mut self, w: Werkzeug) { if self.werkzeuge().iter().any(|(t, _)| *t == w) { self.werkzeug = w; self.von = None } }
    pub fn kachel_pinsel(&mut self, t: usize) { self.pinsel = t; self.stempel = None; self.zuletzt.retain(|&x| x != t); self.zuletzt.insert(0, t); self.zuletzt.truncate(8); if matches!(self.werkzeug, Werkzeug::Stempel | Werkzeug::Pipette | Werkzeug::Auswahl | Werkzeug::Loeschen) { self.werkzeug = Werkzeug::Stift } }
    pub fn zeichen_pinsel(&mut self, code: u8) { self.zell_pinsel.code = code; self.zell_pinsel.cs = self.reiter; if matches!(self.werkzeug, Werkzeug::Pipette | Werkzeug::Auswahl | Werkzeug::Loeschen) { self.werkzeug = Werkzeug::Stift } }
    pub fn reiter_waehlen(&mut self, i: usize) { self.reiter = i; self.zell_pinsel.cs = i }
    fn gemalt(&self) -> Cell { let m = self.ts.as_ref().map(|t| t.color_mode); Cell { col: (m != Some(ColorMode::None)).then_some(self.farbe as u8), ..self.zell_pinsel.clone() } }

    /// Einheit (Kachelraster-Koordinate oder Zelle) zu den Zellkoordinaten `(fx, fy)`; `klemmen` zieht Punkte außerhalb an den Rand
    pub fn einheit_bei(&self, fx: i32, fy: i32, klemmen: bool) -> Option<(usize, usize)> {
        let (s, ts) = (self.screen()?, self.ts.as_ref()?);
        match self.modus {
            Modus::Zelle => { let ok = fx >= 0 && fy >= 0 && (fx as usize) < s.w && (fy as usize) < s.h; if ok { Some((fx as usize, fy as usize)) } else if klemmen { Some((fx.clamp(0, s.w as i32 - 1) as usize, fy.clamp(0, s.h as i32 - 1) as usize)) } else { None } }
            Modus::Kachel => {
                let (cols, rows) = grid_dims(s, ts); if cols == 0 || rows == 0 { return None }
                let (gx, gy) = ((fx - s.origin.0 as i32).div_euclid(ts.tw as i32), (fy - s.origin.1 as i32).div_euclid(ts.th as i32));
                let ok = fx >= s.origin.0 as i32 && fy >= s.origin.1 as i32 && (gx as usize) < cols && (gy as usize) < rows;
                if ok { Some((gx as usize, gy as usize)) } else if klemmen { Some((gx.clamp(0, cols as i32 - 1) as usize, gy.clamp(0, rows as i32 - 1) as usize)) } else { None }
            }
        }
    }

    fn setzen(&mut self, u: (usize, usize), loeschen: bool) {
        match self.modus {
            Modus::Kachel => { let t = if loeschen { 0 } else { self.pinsel }; if let Some(c) = self.screen_mut().and_then(|s| s.grid.get_mut(u.1)).and_then(|r| r.get_mut(u.0)) { *c = t } }
            Modus::Zelle => {
                let zelle = self.gemalt();
                let voll = match self.screen_mut() { Some(s) => if loeschen { remove_patch(s, u.0, u.1); false } else { !set_patch(s, u.0, u.1, zelle) }, None => false };
                if voll { self.status = format!("Höchstens {MAX_PATCHES} Zellen-Patches je Screen") }
            }
        }
        self.uebernehmen()
    }
    fn form(&mut self, a: (usize, usize), b: (usize, usize)) {
        match self.werkzeug {
            Werkzeug::Auswahl => self.auswahl = Some(((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)))),
            Werkzeug::Linie | Werkzeug::Rechteck => { let pts = if self.werkzeug == Werkzeug::Linie { line_points(a, b) } else { rect_points(a, b, self.gefuellt) }; self.undo_punkt(); for u in pts { self.setzen(u, false) } }
            _ => {}
        }
    }
    pub fn aufnehmen(&mut self, u: (usize, usize)) {
        match self.modus {
            Modus::Kachel => { if let Some(&t) = self.screen().and_then(|s| s.grid.get(u.1)).and_then(|r| r.get(u.0)) { self.kachel_pinsel(t); self.werkzeug = Werkzeug::Stift } }
            Modus::Zelle => {
                let (Some(s), Some(ts)) = (self.screen(), self.ts.as_ref()) else { return };
                if let Some((c, col)) = effective(s, ts, u.0, u.1) { let m = ts.color_mode; let n = ts.charsets.len(); self.zell_pinsel = Cell { col: None, ..c }; self.reiter = self.zell_pinsel.cs.min(n.saturating_sub(1)); if m != ColorMode::None { self.farbe = col } self.werkzeug = Werkzeug::Stift }
            }
        }
    }
    fn stempeln(&mut self, u: (usize, usize)) {
        let Some(clip) = self.stempel.clone() else { return }; self.undo_punkt();
        if let Some(s) = self.screen_mut() { for (dy, row) in clip.iter().enumerate() { for (dx, &t) in row.iter().enumerate() { if t == HOLE { continue } if let Some(c) = s.grid.get_mut(u.1 + dy).and_then(|r| r.get_mut(u.0 + dx)) { *c = t } } } }
        self.uebernehmen()
    }

    /// Maus gedrückt: `hier` = Einheit unter dem Zeiger (`None` außerhalb), `nah` = nächste Einheit (geklemmt)
    pub fn maus_unten(&mut self, hier: Option<(usize, usize)>, nah: Option<(usize, usize)>, rechts: bool) {
        if rechts { if let Some(h) = hier { self.aufnehmen(h) } return }
        let Some(u) = nah else { return }; self.von = Some(u); self.letzte = Some(u); self.text_pos = u;
        match self.werkzeug {
            Werkzeug::Auswahl if self.auswahl.is_some_and(|((x0, y0), (x1, y1))| u.0 >= x0 && u.0 <= x1 && u.1 >= y0 && u.1 <= y1) => { self.verschiebe_von = Some(u); self.von = None }
            Werkzeug::Stift if self.modus == Modus::Kachel && self.eigenschaft.is_some() => { self.eigenschaft_malen(u) }
            Werkzeug::Stift | Werkzeug::Loeschen => { self.undo_punkt(); let l = self.werkzeug == Werkzeug::Loeschen; self.setzen(u, l) }
            Werkzeug::Fuellen => { if self.modus == Modus::Kachel { self.undo_punkt(); let b = self.pinsel; if let Some(s) = self.screen_mut() { flood_tiles(&mut s.grid, u.0, u.1, b) } self.uebernehmen() } }
            Werkzeug::Pipette => self.aufnehmen(u), Werkzeug::Stempel => self.stempeln(u), _ => {}
        }
    }
    pub fn maus_zug(&mut self, nah: Option<(usize, usize)>) {
        let (Some(u), Some(from)) = (nah, self.letzte) else { return };
        if matches!(self.werkzeug, Werkzeug::Stift | Werkzeug::Loeschen) { let l = self.werkzeug == Werkzeug::Loeschen; for q in line_points(from, u) { self.setzen(q, l) } self.letzte = Some(u) }
    }
    pub fn maus_los(&mut self, nah: Option<(usize, usize)>) {
        if let (Some(a), Some(u)) = (self.verschiebe_von.take(), nah) { self.auswahl_verschieben(u.0 as i32 - a.0 as i32, u.1 as i32 - a.1 as i32); self.letzte = None; return }
        if let (Some(a), Some(u)) = (self.von.take(), nah) { if matches!(self.werkzeug, Werkzeug::Linie | Werkzeug::Rechteck | Werkzeug::Auswahl) { self.form(a, u) } }
        self.letzte = None
    }

    /// Text tippen (Zeichenmodus, Werkzeug Text): setzt Patches ab dem Textcursor
    pub fn tippe(&mut self, text: &str) {
        if self.werkzeug != Werkzeug::Text || self.modus != Modus::Zelle { return }
        self.undo_punkt();
        let (w, h) = self.screen().map_or((1, 1), |s| (s.w, s.h));
        for ch in text.chars() {
            let (x, y) = self.text_pos; let zelle = Cell { code: bildschirmcode(&self.target, ch), ..self.gemalt() };
            if let Some(s) = self.screen_mut() { if !set_patch(s, x, y, zelle) { self.status = format!("Höchstens {MAX_PATCHES} Zellen-Patches je Screen"); break } }
            self.text_pos = if x + 1 < w { (x + 1, y) } else { (0, (y + 1).min(h - 1)) };
        }
        self.uebernehmen()
    }
    pub fn text_zurueck(&mut self) {
        if self.werkzeug != Werkzeug::Text { return } self.undo_punkt();
        let (x, y) = self.text_pos; let w = self.screen().map_or(1, |s| s.w);
        let np = if x > 0 { (x - 1, y) } else if y > 0 { (w - 1, y - 1) } else { (0, 0) }; if let Some(s) = self.screen_mut() { remove_patch(s, np.0, np.1); } self.text_pos = np; self.uebernehmen()
    }
    pub fn text_zeilenwechsel(&mut self) { let h = self.screen().map_or(1, |s| s.h); self.text_pos = (0, (self.text_pos.1 + 1).min(h - 1)) }

    // -- Auswahl ------------------------------------------------------------------------------------------------------
    /// Auswahl als Stempel merken (Kachelmodus)
    pub fn auswahl_kopieren(&mut self) {
        let (Some(((x0, y0), (x1, y1))), Some(s)) = (self.auswahl, self.screen()) else { return }; if self.modus != Modus::Kachel { return }
        let clip: Vec<Vec<usize>> = (y0..=y1).map(|y| (x0..=x1).map(|x| s.grid.get(y).and_then(|r| r.get(x)).copied().unwrap_or(HOLE)).collect()).collect();
        self.stempel = Some(clip); self.werkzeug = Werkzeug::Stempel
    }
    /// Auswahl leeren: Kacheln → 0 (Kachelmodus), Patches entfernen (Zeichenmodus)
    pub fn auswahl_leeren(&mut self) {
        let Some(((x0, y0), (x1, y1))) = self.auswahl else { return }; self.undo_punkt(); let zelle = self.modus == Modus::Zelle;
        if let Some(s) = self.screen_mut() { if zelle { s.patches.retain(|q| !(q.x >= x0 && q.x <= x1 && q.y >= y0 && q.y <= y1)) } else { for y in y0..=y1 { for x in x0..=x1 { if let Some(c) = s.grid.get_mut(y).and_then(|r| r.get_mut(x)) { *c = 0 } } } } }
        self.uebernehmen()
    }

    /// Auswahl um (dx, dy) Einheiten verschieben (Kachelmodus: Kacheln der Karte, Zeichenmodus: Zellen-Patches). Was herausragt, wird abgeschnitten; die Quelle wird geleert
    /// (Kachel 0 bzw. keine Patches), das Ziel überschrieben. Die Auswahl wandert mit. Gibt zurück, ob sich etwas bewegt hat.
    pub fn auswahl_verschieben(&mut self, dx: i32, dy: i32) -> bool {
        let Some(((x0, y0), (x1, y1))) = self.auswahl else { return false }; if dx == 0 && dy == 0 { return false }
        let zelle = self.modus == Modus::Zelle; let (breite, hoehe) = match self.screen() { Some(s) if zelle => (s.w, s.h), Some(s) => (s.grid.first().map_or(0, |r| r.len()), s.grid.len()), None => return false };
        if breite == 0 || hoehe == 0 || x1 < x0 || y1 < y0 || x1 >= breite || y1 >= hoehe { return false } // veraltete Auswahl außerhalb des Screens
        let (nx0, ny0) = ((x0 as i32 + dx).clamp(0, breite as i32 - 1 - (x1 - x0) as i32) as usize, (y0 as i32 + dy).clamp(0, hoehe as i32 - 1 - (y1 - y0) as i32) as usize);
        if (nx0, ny0) == (x0, y0) { return false } self.undo_punkt();
        if let Some(s) = self.screen_mut() {
            if zelle {
                let bewegt: Vec<Patch> = s.patches.iter().filter(|q| q.x >= x0 && q.x <= x1 && q.y >= y0 && q.y <= y1).cloned().collect();
                s.patches.retain(|q| !(q.x >= x0 && q.x <= x1 && q.y >= y0 && q.y <= y1) && !(q.x >= nx0 && q.x <= nx0 + (x1 - x0) && q.y >= ny0 && q.y <= ny0 + (y1 - y0)));
                for q in bewegt { s.patches.push(Patch { x: q.x - x0 + nx0, y: q.y - y0 + ny0, cell: q.cell }) }
            } else {
                let block: Vec<Vec<usize>> = (y0..=y1).map(|y| (x0..=x1).map(|x| s.grid[y][x]).collect()).collect();
                for y in y0..=y1 { for x in x0..=x1 { s.grid[y][x] = 0 } }
                for (j, r) in block.iter().enumerate() { for (i, &t) in r.iter().enumerate() { s.grid[ny0 + j][nx0 + i] = t } }
            }
        }
        self.auswahl = Some(((nx0, ny0), (nx0 + (x1 - x0), ny0 + (y1 - y0)))); self.uebernehmen(); true
    }
    /// „Eigenschaft malen“: bei eingeschalteter Eigenschaften-Ebene schaltet der Stift die Eigenschaft der Kachel unter dem Zeiger um (gilt für die Kachel im Satz, also für alle ihre Stellen)
    pub fn eigenschaft_malen(&mut self, u: (usize, usize)) {
        let (Some(bit), Some(t)) = (self.eigenschaft, self.screen().and_then(|s| s.grid.get(u.1)).and_then(|r| r.get(u.0)).copied()) else { return };
        self.undo_punkt(); let mut ok = false;
        if let Some(ts) = &mut self.ts { if let Some(k) = ts.tiles.get_mut(t) { k.props ^= 1u8 << bit; ok = true } }
        if ok { self.ts_dirty = true; self.status = "Eigenschaft der Kachel umgeschaltet (gilt für alle ihre Stellen)".into() }
    }

    // -- Kachel aus Auswahl ---------------------------------------------------------------------------------------------
    /// „Als Kachel anlegen“ ab der linken oberen Ecke `at` (Zellkoordinaten)
    pub fn neue_kachel_anfragen(&mut self, at: (usize, usize)) {
        let (Some(s), Some(ts)) = (self.screen(), self.ts.clone()) else { return };
        let Some((cells, col)) = block_cells(s, &ts, at.0, at.1) else { self.status = format!("Ab {},{} passt keine {}×{}-Kachel mehr in den Screen", at.0, at.1, ts.tw, ts.th); return };
        let (ox, oy) = s.origin; let (gx, gy) = (at.0.wrapping_sub(ox) / ts.tw, at.1.wrapping_sub(oy) / ts.th);
        let ausgerichtet = at.0 >= ox && at.1 >= oy && (at.0 - ox) % ts.tw == 0 && (at.1 - oy) % ts.th == 0 && s.grid.get(gy).is_some_and(|r| gx < r.len());
        let dup = find_dup(&ts, &cells, col);
        self.neue_kachel = Some(NeueKachel { at, cells, col, dup, ausgerichtet, ersetzen: ausgerichtet, name: format!("neu{}", ts.tiles.len()) })
    }
    pub fn neue_kachel_abbrechen(&mut self) { self.neue_kachel = None }
    pub fn neue_kachel_ok(&mut self) {
        let Some(nt) = self.neue_kachel.take() else { return }; let Some(mut ts) = self.ts.take() else { return };
        let idx = match nt.dup { Some(d) => d, None => { ts.tiles.push(Tile { name: nt.name.clone(), cells: nt.cells.clone(), col: nt.col, props: 0 }); self.ts_dirty = true; ts.tiles.len() - 1 } };
        self.undo_punkt();
        if nt.ersetzen { let at = nt.at; if let Some(s) = self.screen_mut() { replace_block(s, &ts, at.0, at.1, idx); } }
        self.status = format!("Kachel {idx} {}", if nt.dup.is_some() { "benutzt" } else { "angelegt" }); self.kachel_pinsel(idx); self.ts = Some(ts); self.auswahl = None; self.uebernehmen()
    }

    // -- Bild -----------------------------------------------------------------------------------------------------------
    fn mc(s: &Screen) -> McCols { McCols::aus_farben(&s.colors) }
    pub fn rand(&self) -> usize { if matches!(self.target.as_str(), "c64" | "vic20") { 2 } else { 1 } }
    /// Screen als RGBA-Bild mit Rand; `ta`/`ca` = Deckkraft von Kachel- und Zellebene (abgeblendet < 1), `skala` Bildpunkte je Zeichenpixel
    pub fn bild(&self, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3], ta: f32, ca: f32) -> Bild { self.bild_screen(self.cur, skala, rgb, ta, ca) }
    /// Wie [`BankDoc::bild`] für den Screen `idx` (Miniaturen der Liste)
    pub fn bild_screen(&self, idx: usize, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3], ta: f32, ca: f32) -> Bild {
        let Some(s) = self.bank.screens.get(idx) else { return Bild::neu(8, 8, [0, 0, 0]) };
        let (k, rand) = (skala.max(1), self.rand() * 8 * skala.max(1)); let (bw, bh) = (s.w * 8 * k, s.h * 8 * k);
        let bg = rgb(s.colors["bg"].as_u64().map_or(6, |v| v as usize));
        let mut b = Bild::neu(bw + 2 * rand, bh + 2 * rand, rgb(s.colors["border"].as_u64().map_or(s.colors["bg"].as_u64().map_or(6, |v| v as usize), |v| v as usize)));
        b.flaeche(rand, rand, bw, bh, bg);
        let Some(ts) = self.ts.as_ref() else { return b };
        let mischen = |f: [u8; 3], a: f32| -> [u8; 3] { [0, 1, 2].map(|i| (f[i] as f32 * a + bg[i] as f32 * (1.0 - a)) as u8) };
        let mc = Self::mc(s); let zeichne = |b: &mut Bild, x: usize, y: usize, cs: usize, code: u8, farbe: usize, hi: bool, a: f32| {
            let f = |i: usize| if a >= 1.0 { rgb(i) } else { mischen(rgb(i), a) };
            if let Some(Some(g)) = self.glyphs.get(cs) { g.zeichne(b, rand + x * 8 * k, rand + y * 8 * k, k, &self.target, code, farbe, mc, hi, &f); }
        };
        if ta > 0.0 { for (gy, row) in s.grid.iter().enumerate() { for (gx, &ti) in row.iter().enumerate() { let Some(t) = ts.tiles.get(ti) else { continue };
            for (kk, c) in t.cells.iter().enumerate() { let (x, y) = (s.origin.0 + gx * ts.tw + kk % ts.tw, s.origin.1 + gy * ts.th + kk / ts.tw); if x >= s.w || y >= s.h { continue } zeichne(&mut b, x, y, c.cs, c.code, color_index(ts, s, Some(t), c), c.sel, ta) } } } }
        if ca > 0.0 { for pa in &s.patches { b.flaeche(rand + pa.x * 8 * k, rand + pa.y * 8 * k, 8 * k, 8 * k, mischen(bg, 1.0)); zeichne(&mut b, pa.x, pa.y, pa.cell.cs, pa.cell.code, color_index(ts, s, None, &pa.cell), pa.cell.sel, ca) } }
        b
    }
    /// Prüfsumme der Inhalte aller Screens und des Kachelsatzes (ändert sich bei jeder Bearbeitung; für Zwischenspeicher der Miniaturen)
    pub fn stand(&self) -> u64 {
        use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new();
        for sc in &self.bank.screens { (sc.w, sc.h, sc.origin, &sc.grid, sc.patches.len(), sc.colors.to_string()).hash(&mut h); for p in &sc.patches { (p.x, p.y, p.cell.code, p.cell.cs, p.cell.col).hash(&mut h) } }
        if let Some(ts) = &self.ts { for t in &ts.tiles { for c in &t.cells { (c.code, c.cs, c.col).hash(&mut h) } t.col.hash(&mut h) } }
        h.finish()
    }
    /// Zeichentafel des gewählten Zeichensatzes (Zellebene)
    pub fn tafel_bild(&self, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3]) -> Bild {
        let k = skala.max(1); let n = if self.target == "atari8" { 128 } else { 256 }; let s = self.screen();
        let bg = s.map_or(6, |s| s.colors["bg"].as_u64().map_or(6, |v| v as usize));
        let mut b = Bild::neu(16 * 8 * k, (n / 16) * 8 * k, rgb(bg));
        let mut f = if self.ts.as_ref().is_some_and(|t| t.color_mode == ColorMode::None) { s.map_or(14, |s| s.colors["fg"].as_u64().map_or(14, |v| v as usize)) } else { self.farbe };
        if matches!(self.target.as_str(), "c64" | "mega65" | "vic20") && self.glyphs.get(self.reiter).and_then(|g| g.as_ref()).is_some_and(|g| g.mc) { f |= 8 }
        if let Some(Some(g)) = self.glyphs.get(self.reiter) { for c in 0..n { g.zeichne(&mut b, (c % 16) * 8 * k, (c / 16) * 8 * k, k, &self.target, c as u8, f, McCols::default(), false, rgb); } }
        b
    }
    /// Kachelbild für die Palette (ohne Rand, Satzfarben)
    pub fn kachel_bild(&self, i: usize, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3]) -> Bild {
        let (Some(ts), Some(s)) = (self.ts.as_ref(), self.screen()) else { return Bild::neu(8, 8, [0, 0, 0]) }; let k = skala.max(1);
        let mut b = Bild::neu(ts.tw * 8 * k, ts.th * 8 * k, rgb(s.colors["bg"].as_u64().map_or(6, |v| v as usize)));
        if let Some(t) = ts.tiles.get(i) { let mc = Self::mc(s); for (kk, c) in t.cells.iter().enumerate() { if let Some(Some(g)) = self.glyphs.get(c.cs) { g.zeichne(&mut b, (kk % ts.tw) * 8 * k, (kk / ts.tw) * 8 * k, k, &self.target, c.code, color_index(ts, s, Some(t), c), mc, c.sel, rgb); } } }
        b
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn projekt() -> (PathBuf, PathBuf) {
        let data = repo().join("data");
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!("vc-bankdoc-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        std::fs::copy(repo().join("examples/c64/gfx_demo/graphics/font.vcchar"), dir.join("graphics/font.vcchar")).unwrap();
        (dir, data)
    }
    /// Projekt mit Kachelsatz (3 Kacheln 2×2) und Bank mit zwei Screens
    fn mit_bank() -> (PathBuf, PathBuf, String) {
        let (dir, data) = projekt();
        let t = |name: &str, c: [u8; 4], col: u8| json!({ "name": name, "cells": c, "col": col });
        let satz = json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "hg", "size": { "w": 2, "h": 2 }, "colorMode": "tile", "charsets": ["graphics/font.vcchar"],
            "propDefs": [{ "name": "solid", "bit": 0 }], "tiles": [t("leer", [32, 32, 32, 32], 1), t("a", [1, 2, 3, 4], 5), { "name": "b", "cells": [5, 6, 7, 8], "col": 7, "props": ["solid"] }] });
        std::fs::write(dir.join("graphics/hg.vctiles"), vc_media::project::format_media(&satz)).unwrap();
        let bank = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "lv", "tilesets": ["graphics/hg.vctiles"], "screens": [
            { "name": "eins", "w": 8, "h": 5, "colors": { "border": 0, "bg": 3 }, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 1, 2, 0], [1, 1, 0, 2]] }, "cells": { "patches": [[0, 4, 72], [1, 4, [73, 0, 5]]] } },
            { "name": "zwei", "w": 8, "h": 5, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 0, 0, 0], [0, 0, 0, 0]] } }] });
        let rel = "graphics/lv.vcscreen".to_string(); std::fs::write(dir.join(&rel), vc_media::project::format_media(&bank)).unwrap();
        (dir, data, rel)
    }
    fn leer() -> Cell { Cell { code: 0, cs: 0, col: None, sel: false } }

    #[test]
    fn rechnen_linie_rechteck_fuellen() {
        assert_eq!(line_points((0, 0), (3, 0)), vec![(0, 0), (1, 0), (2, 0), (3, 0)]); assert_eq!(line_points((2, 2), (0, 0)).len(), 3);
        assert_eq!(rect_points((0, 0), (2, 2), false).len(), 8); assert_eq!(rect_points((0, 0), (2, 2), true).len(), 9);
        let mut g = vec![vec![0, 0, 1], vec![0, 1, 1], vec![0, 0, 0]]; flood_tiles(&mut g, 0, 0, 7); assert_eq!(g, vec![vec![7, 7, 1], vec![7, 1, 1], vec![7, 7, 7]]);
    }

    #[test]
    fn bank_rundreise_und_patch_grenze() {
        let (dir, _data, rel) = mit_bank();
        let v: Value = serde_json::from_str(&std::fs::read_to_string(dir.join(&rel)).unwrap()).unwrap(); let b = tiles::parse_bank(&v).unwrap(); let back = bank_json(&b, &v); let b2 = tiles::parse_bank(&back).unwrap();
        assert_eq!((b2.screens.len(), b2.screens[0].patches.len(), b2.screens[0].grid.clone()), (2, 2, b.screens[0].grid.clone())); assert_eq!(b2.screens[0].patches[1].cell, Cell { code: 73, cs: 0, col: Some(5), sel: false }); assert_eq!(back["name"], "lv");
        let mut s2 = b.screens[0].clone(); s2.w = 255; s2.h = 255; s2.patches.clear();
        let mut n = 0; for y in 0..255 { for x in 0..255 { if set_patch(&mut s2, x, y, leer()) { n += 1 } else { break } } if n >= MAX_PATCHES { break } }
        assert_eq!(s2.patches.len(), MAX_PATCHES); assert!(!set_patch(&mut s2, 200, 200, leer()), "256. Patch wird abgelehnt"); assert!(set_patch(&mut s2, 0, 0, Cell { code: 9, ..leer() }), "vorhandener Patch lässt sich überschreiben"); assert_eq!(s2.patches.len(), MAX_PATCHES);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn auswahl_verschieben_und_eigenschaft_malen() {
        let (dir, data, rel) = mit_bank(); let mut e = BankDoc::oeffnen(&dir, &rel, &data).unwrap();
        let vorher = e.bank.screens[0].grid.clone(); let (h, w) = (vorher.len(), vorher[0].len());
        // Kachelmodus: Auswahl 2×1 in der linken oberen Ecke um 2 nach rechts schieben: Quelle leer (0), Ziel überschrieben, Auswahl wandert mit
        e.modus = Modus::Kachel; e.auswahl = Some(((0, 0), (1, 0))); let block = (vorher[0][0], vorher[0][1]);
        assert!(e.auswahl_verschieben(2, 0)); let g = &e.bank.screens[0].grid; assert_eq!((g[0][0], g[0][1]), (0, 0)); assert_eq!((g[0][2], g[0][3]), block); assert_eq!(e.auswahl, Some(((2, 0), (3, 0))));
        e.rueckgaengig(); assert_eq!(e.bank.screens[0].grid, vorher, "Rückgängig stellt die Karte her");
        // am Rand abgeschnitten (geklemmt): weit nach rechts schieben endet am rechten Rand
        e.auswahl = Some(((0, 0), (1, 0))); assert!(e.auswahl_verschieben(1000, 0)); assert_eq!(e.auswahl, Some(((w - 2, 0), (w - 1, 0)))); e.rueckgaengig();
        assert!(!e.auswahl_verschieben(0, 0), "keine Bewegung, kein Undo-Punkt");
        // Zeichenmodus: Patches in der Auswahl wandern, Patches am Ziel werden überschrieben
        e.modus = Modus::Zelle; let c = |code: u8| Cell { code, cs: 0, col: None, sel: false };
        e.bank.screens[0].patches = vec![Patch { x: 0, y: 0, cell: c(40) }, Patch { x: 3, y: 1, cell: c(41) }]; e.auswahl = Some(((0, 0), (0, 0)));
        assert!(e.auswahl_verschieben(3, 1)); let p = &e.bank.screens[0].patches; assert_eq!(p.len(), 1); assert_eq!((p[0].x, p[0].y, p[0].cell.code), (3, 1, 40), "verschoben, altes Ziel überschrieben");
        let _ = h;
        // Eigenschaft malen: Ebene „solid“ an, Stift im Kachelmodus schaltet die Eigenschaft der Kachel um (gilt für den Satz)
        e.modus = Modus::Kachel; e.werkzeug = Werkzeug::Stift; let bit = e.ts.as_ref().unwrap().prop_defs.first().map(|d| d.1); 
        if let Some(bit) = bit { e.eigenschaft = Some(bit); let t = e.bank.screens[0].grid[0][0]; let vor = e.ts.as_ref().unwrap().tiles[t].props; e.maus_unten(Some((0, 0)), Some((0, 0)), false);
            assert_eq!(e.ts.as_ref().unwrap().tiles[t].props, vor ^ (1 << bit)); assert!(e.ts_dirty); assert_eq!(e.bank.screens[0].grid, vorher, "die Karte bleibt unberührt"); e.maus_unten(Some((0, 0)), Some((0, 0)), false); assert_eq!(e.ts.as_ref().unwrap().tiles[t].props, vor, "zweimal = wieder wie vorher"); }
    }

    #[test]
    fn auswahl_zu_kachel_duplikat_und_ersetzen() {
        let (dir, data, rel) = mit_bank(); let mut e = BankDoc::oeffnen(&dir, &rel, &data).unwrap(); let ts = e.ts.clone().unwrap();
        // Block (4,0): im Kachelraster Kachel 2 (cells 5,6,7,8, Farbe 7), also Duplikat von Kachel 2
        let s = e.bank.screens[0].clone(); let (cells, col) = block_cells(&s, &ts, 4, 0).unwrap();
        assert_eq!((cells.iter().map(|c| c.code).collect::<Vec<_>>(), col), (vec![5, 6, 7, 8], 7)); assert_eq!(find_dup(&ts, &cells, col), Some(2)); assert!(block_cells(&s, &ts, 0, 4).is_none(), "Block reicht über den unteren Rand");
        // Neue Kachel aus Block (0,0) mit Patch darüber
        e.bank.screens[0].patches.push(Patch { x: 0, y: 0, cell: Cell { code: 40, cs: 0, col: Some(9), sel: false } });
        e.neue_kachel_anfragen((0, 0)); let nt = e.neue_kachel.clone().unwrap(); assert!(nt.ausgerichtet && nt.dup.is_none() && nt.cells[0].code == 40);
        e.neue_kachel_ok(); assert_eq!(e.ts.as_ref().unwrap().tiles.len(), 4); assert!(e.ts_dirty && e.dirty()); assert_eq!(e.bank.screens[0].grid[0][0], 3, "Zellen im Screen durch den Kachelverweis ersetzt");
        assert!(!e.bank.screens[0].patches.iter().any(|p| p.x == 0 && p.y == 0), "Patch im Block entfernt");
        // Duplikat wird wiederverwendet
        e.neue_kachel_anfragen((4, 0)); assert_eq!(e.neue_kachel.as_ref().unwrap().dup, Some(2)); e.neue_kachel_ok(); assert_eq!(e.ts.as_ref().unwrap().tiles.len(), 4, "keine neue Kachel bei Duplikat");
        // unausgerichtet: kein Ersetzen
        e.neue_kachel_anfragen((1, 0)); let nt = e.neue_kachel.clone().unwrap(); assert!(!nt.ausgerichtet && !nt.ersetzen); e.neue_kachel_abbrechen();
        // speichern: Satz und Bank
        e.speichern().unwrap(); let t2: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/hg.vctiles")).unwrap()).unwrap(); assert_eq!(t2["tiles"].as_array().unwrap().len(), 4);
        let e2 = BankDoc::oeffnen(&dir, &rel, &data).unwrap(); assert_eq!(e2.bank.screens[0].grid[0][0], 3); assert!(!e.dirty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bank_oeffnen_bearbeiten_speichern_rundreise() {
        let (dir, data, rel) = mit_bank(); let mut e = BankDoc::oeffnen(&dir, &rel, &data).unwrap();
        assert_eq!((e.bank.screens.len(), e.ts.as_ref().unwrap().tiles.len()), (2, 3)); assert!(e.glyphs.first().is_some_and(|g| g.is_some()), "Zeichensatz des Kachelsatzes geladen"); assert_eq!(e.saetze, vec!["graphics/hg.vctiles".to_string()]);
        // Kachel setzen (Stift) per Maus-Schnittstelle, Rückgängig
        e.kachel_pinsel(2); e.maus_unten(Some((1, 1)), Some((1, 1)), false); e.maus_los(Some((1, 1))); assert_eq!(e.bank.screens[0].grid[1][1], 2); assert!(e.dirty());
        e.rueckgaengig(); assert_eq!(e.bank.screens[0].grid[1][1], 1, "Rückgängig stellt die Kachel wieder her"); e.wiederholen(); assert_eq!(e.bank.screens[0].grid[1][1], 2);
        // Screen neu, duplizieren, Reihenfolge, löschen
        e.screen_neu(); assert_eq!((e.bank.screens.len(), e.bank.screens[2].grid.len()), (3, 2)); e.screen_duplizieren(); assert_eq!((e.bank.screens.len(), e.cur), (4, 3)); e.screen_verschieben(-1); assert_eq!(e.cur, 2);
        let namen: Vec<String> = e.bank.screens.iter().map(|s| s.name.clone()).collect(); assert_eq!(namen.len(), namen.iter().collect::<std::collections::HashSet<_>>().len(), "Namen eindeutig: {namen:?}");
        e.screen_loeschen(); assert_eq!(e.bank.screens.len(), 3);
        // Zellebene: Zeichen setzen und löschen
        e.screen_waehlen(0); let zelle = Cell { code: 12, col: Some(2), ..leer() };
        e.modus_waehlen(Modus::Zelle); e.zeichen_pinsel(12); e.farbe = 2; e.maus_unten(Some((7, 4)), Some((7, 4)), false); e.maus_los(Some((7, 4))); e.werkzeug_waehlen(Werkzeug::Loeschen); e.maus_unten(Some((0, 4)), Some((0, 4)), false); e.maus_los(Some((0, 4)));
        assert!(e.bank.screens[0].patches.iter().any(|p| p.x == 7 && p.y == 4 && p.cell == zelle)); assert!(!e.bank.screens[0].patches.iter().any(|p| p.x == 0 && p.y == 4), "Löschen entfernt den Patch");
        // Ursprung und Restzellen
        let ts = e.ts.clone().unwrap(); let mut s = e.bank.screens[0].clone(); s.origin = (1, 1); fit_grid(&mut s, &ts); assert_eq!(grid_dims(&s, &ts), (3, 2)); assert_eq!(s.grid[0].len(), 3);
        e.speichern().unwrap(); let e2 = BankDoc::oeffnen(&dir, &rel, &data).unwrap(); assert_eq!(e2.bank.screens.len(), 3); assert_eq!(e2.bank.screens[0].grid[1][1], 2); assert!(e2.bank.screens[0].patches.iter().any(|p| p.x == 7 && p.y == 4 && p.cell == zelle));
        let out = vc_media::project::export(&dir, &data, &rel, "asm", Some("lv"), None, 10, 10).unwrap(); assert!(out["files"].as_array().is_some_and(|f| !f.is_empty()), "{out}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn werkzeuge_linie_rechteck_fuellen_auswahl_stempel_text_und_bild() {
        let (dir, data, rel) = mit_bank(); let mut e = BankDoc::oeffnen(&dir, &rel, &data).unwrap(); e.screen_waehlen(1); e.kachel_pinsel(1);
        let g = |e: &BankDoc| e.bank.screens[1].grid.clone();
        // Linie, Rechteck, Füllen (Kachelmodus)
        e.werkzeug_waehlen(Werkzeug::Linie); e.maus_unten(Some((0, 0)), Some((0, 0)), false); e.maus_los(Some((3, 0))); assert_eq!(g(&e)[0], vec![1, 1, 1, 1]);
        e.werkzeug_waehlen(Werkzeug::Fuellen); e.kachel_pinsel(2); e.werkzeug_waehlen(Werkzeug::Fuellen); e.maus_unten(Some((0, 1)), Some((0, 1)), false); assert_eq!(g(&e)[1], vec![2, 2, 2, 2]); assert_eq!(g(&e)[0], vec![1, 1, 1, 1], "Füllen bleibt in der Fläche");
        // Auswahl → Stempel → stempeln
        e.werkzeug_waehlen(Werkzeug::Auswahl); e.maus_unten(Some((0, 0)), Some((0, 0)), false); e.maus_los(Some((1, 1))); assert_eq!(e.auswahl, Some(((0, 0), (1, 1)))); e.auswahl_kopieren(); assert_eq!(e.werkzeug, Werkzeug::Stempel);
        e.maus_unten(Some((2, 0)), Some((2, 0)), false); assert_eq!((g(&e)[0][2], g(&e)[0][3], g(&e)[1][2], g(&e)[1][3]), (1, 1, 2, 2), "Stempel setzt den kopierten Block");
        e.werkzeug_waehlen(Werkzeug::Auswahl); e.auswahl = Some(((0, 0), (3, 1))); e.auswahl_leeren(); assert!(g(&e).iter().flatten().all(|&t| t == 0), "Auswahl leeren: Kachel 0");
        // Zellebene: Text tippen
        e.modus_waehlen(Modus::Zelle); e.werkzeug_waehlen(Werkzeug::Text); e.farbe = 3; e.maus_unten(Some((1, 2)), Some((1, 2)), false); e.maus_los(Some((1, 2))); e.tippe("ab"); assert_eq!(e.bank.screens[1].patches.len(), 2); assert_eq!((e.bank.screens[1].patches[0].cell.code, e.bank.screens[1].patches[1].cell.code, e.text_pos), (1, 2, (3, 2)));
        e.text_zurueck(); assert_eq!((e.bank.screens[1].patches.len(), e.text_pos), (1, (2, 2)));
        // Pipette nimmt Zeichen und Farbe aus der Zelle auf
        e.screen_waehlen(0); e.werkzeug_waehlen(Werkzeug::Stift); e.maus_unten(None, None, true); e.maus_unten(Some((1, 4)), Some((1, 4)), true); assert_eq!(e.zell_pinsel.code, 73, "rechte Taste nimmt die Zelle auf: Patch (1,4)");
        // Bild: Rand, Hintergrund, Kachelzeichen sichtbar; Palette und Kachelbild
        let rgb = |i: usize| -> [u8; 3] { [i as u8 * 10 + 1, 2, 3] }; let b = e.bild(1, &rgb, 1.0, 1.0); assert_eq!((b.w, b.h), (8 * 8 + 32, 5 * 8 + 32), "C64: 2 Zeichen Rand"); assert_eq!(&b.rgba[0..3], &rgb(0)[..], "Rand 0"); assert!(b.rgba.chunks(4).any(|p| p[..3] == rgb(5)[..]), "Kachel a (Farbe 5) ist zu sehen");
        assert_eq!(e.kachel_bild(1, 1, &rgb).w, 16); assert_eq!(e.tafel_bild(1, &rgb).w, 128);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn alte_zeichen_screens_werden_zerlegt() {
        let (dir, data) = projekt(); let profiles = vc_media::load_profiles(&data).unwrap(); let p = vc_media::profile(&profiles, "c64.screen.text").unwrap().clone(); let tp = "c64.tileset";
        // 40×25, 2×2-Raster: Muster aus vier Kacheln, mehrfach wiederholt; Zeile 24 mit Text
        let (w, h) = (p.width, p.height); let mut chars = vec![vec![32u8; w]; h]; let mut attrs = vec![vec![1u8; w]; h];
        for y in 0..24 { for x in 0..w { chars[y][x] = [65u8, 66, 67, 68][(x % 2) + (y % 2) * 2]; attrs[y][x] = if (x / 2 + y / 2) % 3 == 0 { 5 } else { 7 }; } } for (i, c) in "SCORE".bytes().enumerate() { chars[24][i] = c & 63; }
        let old = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "title", "charset": "graphics/font.vcchar", "colors": { "border": 0, "bg": 6 }, "chars": chars, "attrs": attrs });
        let d = decompose(&old, &p, tp, 2, 2, (0, 0), "title").unwrap();
        assert_eq!(d.color_mode, ColorMode::Tile); assert_eq!(d.tiles, 2, "gleiche Blöcke zählen einmal, zwei Farben"); assert_eq!(d.patches, 5, "SCORE in der 25. Zeile bleibt Zellebene"); assert_eq!(d.bytes_before, 2000); assert!(d.bytes_after < d.bytes_before / 4, "{} von {}", d.bytes_after, d.bytes_before);
        let mut wild = chars.clone(); for y in 0..24 { for x in 0..w { wild[y][x] = ((x * 7 + y * 13 + x * y) % 200) as u8 + 33; } }
        assert!(decompose(&json!({ "kind": "screen", "profile": "c64.screen.text", "charset": null, "chars": wild, "attrs": attrs }), &p, tp, 1, 1, (0, 0), "w").is_err(), "mehr als 256 Kacheln");
        let mut d = d; std::fs::write(dir.join("graphics/title.vcscreen"), "{}").unwrap();
        let (satz, bank) = write_decomp(&dir, "graphics/title.vcscreen", &mut d).unwrap(); assert_eq!((satz.as_str(), bank.as_str()), ("graphics/title_tiles.vctiles", "graphics/title_bank.vcscreen"));
        assert!(write_decomp(&dir, "graphics/title.vcscreen", &mut d).is_err(), "bestehende Dateien werden nicht überschrieben");
        let e = BankDoc::oeffnen(&dir, &bank, &data).unwrap(); let ts = e.ts.as_ref().unwrap();
        assert_eq!((ts.tiles.len(), e.bank.screens[0].grid.len(), e.bank.screens[0].grid[0].len(), e.bank.screens[0].patches.len()), (2, 12, 20, 5));
        let (c, col) = effective(&e.bank.screens[0], ts, 3, 5).unwrap(); assert_eq!((c.code, col), (chars[5][3], attrs[5][3] as usize), "Anzeige entspricht dem alten Bild");
        let out = vc_media::project::export(&dir, &data, &bank, "asm", Some("title"), None, 10, 10).unwrap(); assert!(out["files"].as_array().is_some_and(|f| !f.is_empty()), "{out}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
