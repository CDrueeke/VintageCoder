// SPDX-License-Identifier: GPL-3.0-or-later
//! Screen-Editor für Screen-Banken (`.vcscreen` mit `screens`; Modell in `vc_media::tiles`). Alte Textbildschirme (`chars`/`attrs`) bleiben im
//! alten Editor (`screen.rs`), der sie mit „In Kacheln zerlegen“ (`decompose`) in Kachelsatz und Bank umwandeln kann.
//!
//! Oben die Reiter der Screens (Neu, Duplizieren, Umbenennen, Löschen, Reihenfolge), links im **Kachelmodus** die Palette des Kachelsatzes
//! (Klick = Pinsel, Umschalt-Klick = Block als Stempel), im **Zeichenmodus** die Zellebene (Patches, bis 255 je Screen). Rechts Größe, Ursprung
//! des Kachelrasters, Farben, Zeichenwähler, Eigenschaften-Ebene und eine Vorschau mit echtem Zeichensatz. Konzept: Abschnitt 4.2 des Konzepts
//! `assets/docs/Konzept-Kachel-und-Screen-Editor.md`.
use crate::bitmap::{Env, MediaEd, Palettes};
use crate::screen::{load_charset, screen_code, Common, Glyphs, McCols};
use crate::ui::studio;
use eframe::egui::{self, pos2, vec2, Color32, Rect, RichText, Sense, Stroke};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_media::tiles::{self, Bank, Cell, ColorMode, Patch, Screen, Tile, TileSet};

/// Höchstzahl der Zellen-Patches je Screen (Zähler im Export ein Byte)
pub const MAX_PATCHES: usize = 255;
/// Platzhalter in einem Stempel: keine Kachel an dieser Stelle
const HOLE: usize = usize::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode { Tile, Cell }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tool { Pen, Line, Rect, Fill, Pick, Select, Stamp, Erase, Text }

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

fn default_col(ts: &TileSet) -> u8 { ts.tiles.first().map_or(1, |t| t.col) }

/// Kachel und Zelle, die das Kachelraster an Zelle (x, y) liefert
fn tile_cell<'a>(s: &Screen, ts: &'a TileSet, x: usize, y: usize) -> Option<(&'a Tile, &'a Cell)> {
    let (ox, oy) = s.origin;
    if x < ox || y < oy { return None; }
    let (gx, gy) = ((x - ox) / ts.tw, (y - oy) / ts.th);
    let t = ts.tiles.get(*s.grid.get(gy)?.get(gx)?)?;
    t.cells.get(((y - oy) % ts.th) * ts.tw + (x - ox) % ts.tw).map(|c| (t, c))
}

/// Farbnummer, mit der eine Zelle erscheint
fn color_index(ts: &TileSet, s: &Screen, t: Option<&Tile>, c: &Cell) -> usize {
    match ts.color_mode {
        ColorMode::None => s.colors["fg"].as_u64().unwrap_or(14) as usize,
        ColorMode::Tile => t.map_or_else(|| c.col.unwrap_or(default_col(ts)), |t| t.col) as usize,
        ColorMode::Cell => c.col.or(t.map(|t| t.col)).unwrap_or(default_col(ts)) as usize,
    }
}

fn patch_at(s: &Screen, x: usize, y: usize) -> Option<usize> { s.patches.iter().rposition(|p| p.x == x && p.y == y) }

/// Was an Zelle (x, y) zu sehen ist: Patch, sonst Kachelzelle
fn effective(s: &Screen, ts: &TileSet, x: usize, y: usize) -> Option<(Cell, usize)> {
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
    let blank = screen_code(&p.target, ' ');
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

/// Dialog „In Kacheln zerlegen“ im alten Screen-Editor
pub struct DecompDlg { pub tw: usize, pub th: usize, pub origin: (usize, usize), pub result: String }

impl DecompDlg {
    pub fn new() -> DecompDlg { DecompDlg { tw: 2, th: 2, origin: (0, 0), result: String::new() } }
    /// Zeichnet das Fenster; `false`, wenn es geschlossen wurde
    pub fn show(&mut self, ctx: &egui::Context, media: &Value, p: &vc_media::Profile, profiles: &[vc_media::Profile], root: &Path, rel: &str) -> bool {
        let mut open = true;
        let tp = profiles.iter().find(|x| x.kind == "tileset" && x.target == p.target).map(|x| x.id.clone());
        egui::Window::new(RichText::new("In Kacheln zerlegen").strong()).id(egui::Id::new(("decomp", rel))).open(&mut open).collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label("Gleiche Blöcke des Bildschirms werden zu einer Kachel; Zellen außerhalb des Rasters kommen in die Zellebene.");
            ui.horizontal(|ui| {
                ui.label("Kachel"); ui.add(egui::DragValue::new(&mut self.tw).range(1..=tiles::MAX_SIDE)); ui.label("×"); ui.add(egui::DragValue::new(&mut self.th).range(1..=tiles::MAX_SIDE));
                ui.label("Ursprung"); ui.add(egui::DragValue::new(&mut self.origin.0).range(0..=40)); ui.label(","); ui.add(egui::DragValue::new(&mut self.origin.1).range(0..=40));
            });
            let name = vc_media::naming::stem(rel).to_string();
            let Some(tp) = tp else { ui.label(RichText::new("Für dieses System gibt es kein Kachelsatz-Profil.").color(crate::theme::bad())); return };
            match decompose(media, p, &tp, self.tw, self.th, self.origin, &name) {
                Err(e) => { ui.label(RichText::new(e).color(crate::theme::bad())); }
                Ok(mut d) => {
                    let pct = 100.0 - d.bytes_after as f32 * 100.0 / d.bytes_before.max(1) as f32;
                    let farbe = match d.color_mode { ColorMode::Tile => "je Kachel", ColorMode::Cell => "je Zelle", ColorMode::None => "ohne Farbe" };
                    ui.label(format!("{} Kacheln, {} Zellen in der Zellebene · {} Byte statt {} Byte ({:+.0} %) · Farbe: {}", d.tiles, d.patches, d.bytes_after, d.bytes_before, -pct, farbe));
                    if d.bytes_after >= d.bytes_before { ui.label(RichText::new("Das spart hier keinen Platz; Kachelgröße oder Ursprung ändern.").color(crate::theme::warn()).small()); }
                    if ui.button("Umwandeln").on_hover_text("Legt Kachelsatz und Bank neben dem alten Screen an; der alte Screen bleibt").clicked() {
                        self.result = match write_decomp(root, rel, &mut d) { Ok((s, b)) => format!("Geschrieben: {s} und {b}. Die neuen Dateien bitte im Projektbaum öffnen."), Err(e) => e };
                    }
                }
            }
            if !self.result.is_empty() { ui.label(RichText::new(&self.result).small()); }
        });
        open
    }
}

// ---------------------------------------------------------------------------------------------- Editor

/// Rückfrage „Auswahl → Kachel anlegen“
#[derive(Clone)]
struct NewTile { at: (usize, usize), cells: Vec<Cell>, col: u8, dup: Option<usize>, aligned: bool, replace: bool, name: String }

pub struct BankEd {
    c: Common, data: PathBuf, profiles: Vec<vc_media::Profile>, dim: (usize, usize),
    bank: Bank, synced: String, touched: bool, frame: u32,
    ts: Option<TileSet>, ts_base: Value, ts_err: Option<String>, ts_dirty: bool, ts_mtime: Option<std::time::SystemTime>, sets: Vec<String>,
    glyphs: Vec<Option<Glyphs>>,
    cur: usize, mode: Mode, tool: Tool, brush: usize, stamp: Option<Vec<Vec<usize>>>, recent: Vec<usize>, pal_anchor: Option<usize>, pal_cols: usize,
    cell_brush: Cell, tab: usize, col: usize, zoom: f32, fit: bool, avail: egui::Vec2, tile_lines: bool, cell_grid: bool, filled: bool, props_bit: Option<u8>, show_preview: bool,
    sel: Option<((usize, usize), (usize, usize))>, drag: Option<(usize, usize)>, last: Option<(usize, usize)>, textpos: (usize, usize), newtile: Option<NewTile>,
}

impl BankEd {
    pub fn open(ctx: &egui::Context, root: &Path, rel: &str, data: &Path) -> Result<BankEd, String> {
        let c = Common::new(ctx, root, rel, data)?;
        let profiles = vc_media::load_profiles(data)?;
        let p = vc_media::profile(&profiles, c.media["profile"].as_str().unwrap_or(""))?;
        let dim = (p.width, p.height);
        let bank = tiles::parse_bank(&c.media).map_err(|e| format!("{rel}: {e}"))?;
        let synced = c.media.to_string();
        let mut e = BankEd { data: data.into(), profiles, dim, bank, synced, touched: false, frame: 0, ts: None, ts_base: Value::Null, ts_err: None, ts_dirty: false, ts_mtime: None, sets: vec![], glyphs: vec![],
            cur: 0, mode: Mode::Tile, tool: Tool::Pen, brush: 0, stamp: None, recent: vec![], pal_anchor: None, pal_cols: 4,
            cell_brush: Cell { code: 1, cs: 0, col: None, sel: false }, tab: 0, col: 1, zoom: 16.0, fit: true, avail: egui::Vec2::ZERO, tile_lines: true, cell_grid: false, filled: false, props_bit: None, show_preview: true,
            sel: None, drag: None, last: None, textpos: (0, 0), newtile: None, c };
        e.load_ts();
        e.sets = e.scan_sets();
        Ok(e)
    }

    fn cur_screen(&self) -> Option<&Screen> { self.bank.screens.get(self.cur) }
    fn scr_mut(&mut self) -> Option<&mut Screen> { self.bank.screens.get_mut(self.cur) }

    fn scan_sets(&self) -> Vec<String> {
        fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
                if n.starts_with('.') || n == "build" || n == "target" { continue; }
                if p.is_dir() { if depth < 5 { walk(&p, depth + 1, out); } } else if n.ends_with(".vctiles") { out.push(p); } }
        }
        let mut files = vec![]; walk(&self.c.root, 0, &mut files); files.sort();
        files.into_iter().filter_map(|f| {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(&f).ok()?).ok()?;
            let target = vc_media::profile(&self.profiles, v["profile"].as_str()?).ok()?.target.clone();
            (v["kind"] == "tileset" && target == self.c.target).then(|| f.strip_prefix(&self.c.root).map(|p| p.to_string_lossy().replace('\\', "/")).ok()).flatten()
        }).collect()
    }

    /// Kachelsatz der Bank (Eintrag 0 in `tilesets`) von der Platte lesen
    fn load_ts(&mut self) {
        self.ts = None; self.ts_err = None; self.glyphs.clear();
        let Some(rel) = self.bank.tilesets.first().cloned() else { self.ts_err = Some("Die Bank nennt noch keinen Kachelsatz: rechts einen wählen.".into()); return };
        let path = self.c.root.join(&rel);
        let r = std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string())).and_then(|v| { let ts = tiles::parse_tileset(&v)?; Ok((v, ts)) });
        match r {
            Ok((v, ts)) => {
                self.ts_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                self.glyphs = ts.charsets.iter().map(|p| load_charset(&self.c.root, p, &self.c.glyph_ctx)).collect();
                self.tab = self.tab.min(ts.charsets.len().saturating_sub(1)); self.brush = self.brush.min(ts.tiles.len().saturating_sub(1));
                self.ts_base = v; self.ts = Some(ts); self.ts_dirty = false;
            }
            Err(e) => self.ts_err = Some(format!("{rel}: {e}")),
        }
    }

    /// Alle 20 Bilder prüfen, ob der Kachelsatz auf der Platte neu ist (z. B. im Kachel-Editor gespeichert)
    fn watch_ts(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.frame % 20 != 0 || self.ts_dirty { return; }
        let Some(rel) = self.bank.tilesets.first() else { return };
        let m = std::fs::metadata(self.c.root.join(rel)).and_then(|m| m.modified()).ok();
        if m != self.ts_mtime { self.load_ts(); }
    }

    fn commit(&mut self) {
        self.c.media = bank_json(&self.bank, &self.c.media);
        self.synced = self.c.media.to_string(); self.touched = false;
    }

    /// Nach Rückgängig/Wiederholen die Bank neu lesen
    fn sync(&mut self) {
        let now = self.c.media.to_string();
        if now == self.synced { return; }
        if let Ok(b) = tiles::parse_bank(&self.c.media) {
            let set_changed = b.tilesets != self.bank.tilesets;
            self.bank = b; if set_changed { self.load_ts(); }
        }
        self.synced = now; self.cur = self.cur.min(self.bank.screens.len().saturating_sub(1)); self.sel = None; self.newtile = None;
    }

    fn undo_point(&mut self) { self.c.push_undo(); }

    // ------------------------------------------------------------------------ Screens (Reiter)

    fn unique_name(&self, base: &str) -> String {
        if !self.bank.screens.iter().any(|s| s.name == base) { return base.into(); }
        (2..).map(|n| format!("{base}{n}")).find(|n| !self.bank.screens.iter().any(|s| &s.name == n)).unwrap_or_default()
    }
    fn new_screen(&mut self) {
        self.undo_point();
        let (w, h) = self.cur_screen().map_or(self.dim, |s| (s.w, s.h));
        let colors = self.cur_screen().map_or(json!({}), |s| s.colors.clone());
        let mut s = Screen { name: self.unique_name("screen"), w, h, colors, set: 0, origin: self.cur_screen().map_or((0, 0), |s| s.origin), grid: vec![], patches: vec![] };
        if let Some(ts) = &self.ts { fit_grid(&mut s, ts); }
        self.bank.screens.push(s); self.cur = self.bank.screens.len() - 1; self.sel = None; self.touched = true;
    }
    fn dup_screen(&mut self) {
        let Some(mut s) = self.cur_screen().cloned() else { return };
        self.undo_point(); s.name = self.unique_name(&format!("{}_kopie", s.name));
        self.bank.screens.insert(self.cur + 1, s); self.cur += 1; self.touched = true;
    }
    fn delete_screen(&mut self) {
        if self.bank.screens.len() < 2 { return; }
        self.undo_point(); self.bank.screens.remove(self.cur); self.cur = self.cur.min(self.bank.screens.len() - 1); self.sel = None; self.touched = true;
    }
    fn move_screen(&mut self, d: i32) {
        let to = self.cur as i32 + d;
        if to < 0 || to as usize >= self.bank.screens.len() { return; }
        self.undo_point(); self.bank.screens.swap(self.cur, to as usize); self.cur = to as usize; self.touched = true;
    }

    // ------------------------------------------------------------------------ Zeichnen

    /// Zeichen aus Zeichensatz `cs` zeichnen; `fgi` = Farbindex der Zelle (bei C64-Mehrfarbzeichen mit Bit 3 = mehrfarbig), `mc` = Mehrfarbwerte 1/2 des Screens, `alpha` = Deckkraft
    fn glyph_at(&self, p: &egui::Painter, pal: &Palettes, cs: usize, code: u8, r: Rect, fgi: usize, mc: McCols, hi: bool, alpha: f32) {
        let drawn = self.glyphs.get(cs).and_then(|g| g.as_ref()).map_or(false, |g| g.paint(p, pal, &self.c.target, code, r, fgi, mc, hi, alpha));
        if !drawn { p.text(r.center(), egui::Align2::CENTER_CENTER, format!("{code}"), egui::FontId::monospace((r.height() * 0.4).clamp(5.0, 14.0)), pal.color(&self.c.target, fgi).gamma_multiply(0.7 * alpha)); }
    }
    fn screen_mc(s: &Screen) -> McCols { McCols::from_colors(&s.colors) }
    fn bg(&self, pal: &Palettes, s: &Screen) -> Color32 { pal.color(&self.c.target, s.colors["bg"].as_u64().unwrap_or(6) as usize) }

    /// Kachel- und Zellebene eines Screens ab `org` mit Zellgröße `cz`; `ta`/`ca` = Deckkraft der Ebenen (abgeblendet < 1)
    fn paint_screen(&self, p: &egui::Painter, pal: &Palettes, s: &Screen, ts: &TileSet, org: egui::Pos2, cz: f32, ta: f32, ca: f32) {
        let cell_rect = |x: usize, y: usize| Rect::from_min_size(pos2(org.x + x as f32 * cz, org.y + y as f32 * cz), vec2(cz, cz));
        if ta > 0.0 {
            for (gy, row) in s.grid.iter().enumerate() { for (gx, &ti) in row.iter().enumerate() {
                let Some(t) = ts.tiles.get(ti) else { continue };
                for (k, c) in t.cells.iter().enumerate() {
                    let (x, y) = (s.origin.0 + gx * ts.tw + k % ts.tw, s.origin.1 + gy * ts.th + k / ts.tw);
                    if x >= s.w || y >= s.h { continue; }
                    self.glyph_at(p, pal, c.cs, c.code, cell_rect(x, y), color_index(ts, s, Some(t), c), Self::screen_mc(s), c.sel, ta);
                }
            } }
        }
        if ca > 0.0 {
            for pa in &s.patches {
                let r = cell_rect(pa.x, pa.y);
                p.rect_filled(r, 0.0, self.bg(pal, s).gamma_multiply(ca));
                self.glyph_at(p, pal, pa.cell.cs, pa.cell.code, r, color_index(ts, s, None, &pa.cell), Self::screen_mc(s), pa.cell.sel, ca);
            }
        }
    }

    /// Rechteck einer Einheit des Modus: im Kachelmodus eine Kachel (Raster-Koordinate), im Zeichenmodus eine Zelle
    fn unit_rect(&self, s: &Screen, ts: &TileSet, inner: Rect, cz: f32, u: (usize, usize)) -> Rect {
        match self.mode {
            Mode::Tile => Rect::from_min_size(pos2(inner.left() + (s.origin.0 + u.0 * ts.tw) as f32 * cz, inner.top() + (s.origin.1 + u.1 * ts.th) as f32 * cz), vec2(ts.tw as f32 * cz, ts.th as f32 * cz)),
            Mode::Cell => Rect::from_min_size(pos2(inner.left() + u.0 as f32 * cz, inner.top() + u.1 as f32 * cz), vec2(cz, cz)),
        }
    }

    // ------------------------------------------------------------------------ Bedienung

    fn modes_tools(&self) -> &'static [(Tool, &'static str)] {
        match self.mode {
            Mode::Tile => &[(Tool::Pen, "Stift"), (Tool::Line, "Linie"), (Tool::Rect, "Rechteck"), (Tool::Fill, "Füllen"), (Tool::Pick, "Pipette"), (Tool::Select, "Auswahl"), (Tool::Stamp, "Stempel"), (Tool::Erase, "Löschen")],
            Mode::Cell => &[(Tool::Pen, "Zeichen"), (Tool::Line, "Linie"), (Tool::Rect, "Rechteck"), (Tool::Pick, "Pipette"), (Tool::Select, "Auswahl"), (Tool::Text, "Text"), (Tool::Erase, "Löschen")],
        }
    }
    fn set_mode(&mut self, m: Mode) {
        if self.mode == m { return; }
        self.mode = m; self.sel = None; self.drag = None; self.last = None;
        if !self.modes_tools().iter().any(|(t, _)| *t == self.tool) { self.tool = Tool::Pen; }
    }

    fn tools_ui(&mut self, ui: &mut egui::Ui) {
        self.c.file_tools(ui);
        let mut m = if self.mode == Mode::Tile { "tile" } else { "cell" }.to_string();
        if crate::ui::widgets::segmented(ui, &mut m, &[("tile", "Kacheln"), ("cell", "Zeichen")]) { self.set_mode(if m == "tile" { Mode::Tile } else { Mode::Cell }); }
        ui.separator();
        for (t, n) in self.modes_tools() { ui.selectable_value(&mut self.tool, *t, *n); }
        if matches!(self.tool, Tool::Rect) { ui.checkbox(&mut self.filled, "gefüllt"); }
    }

    /// Ansichtsleiste über der Fläche: Gitter und Zoom (Passend rechnet aus dem freien Platz)
    fn view_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.tile_lines, "Kachelgrenzen"); ui.checkbox(&mut self.cell_grid, "Zellgitter");
            ui.label("Zoom");
            ui.spacing_mut().slider_width = 110.0;
            if ui.add(egui::Slider::new(&mut self.zoom, 4.0..=48.0).show_value(false)).changed() { self.fit = false; }
            ui.label(RichText::new(format!("{}×", self.zoom.round())).weak().small());
            if ui.selectable_label(self.fit, "Passend").on_hover_text("Zoom automatisch an die freie Fläche anpassen").clicked() { self.fit = !self.fit; }
        });
    }

    fn remember(&mut self, t: usize) { self.recent.retain(|&x| x != t); self.recent.insert(0, t); self.recent.truncate(8); }

    fn palette_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet) {
        studio::caption(ui, "Kacheln");
        let sz = 34.0;
        let draw = |me: &Self, ui: &mut egui::Ui, i: usize, size: f32, scr: &Screen| -> (Rect, egui::Response) {
            let (r, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click());
            let p = ui.painter_at(r);
            p.rect_filled(r, 0.0, me.bg(pal, scr));
            if let Some(t) = ts.tiles.get(i) {
                let cz = size / ts.tw.max(ts.th) as f32; // Kacheln gleich groß, längere Seite füllt
                for (k, c) in t.cells.iter().enumerate() {
                    let cr = Rect::from_min_size(r.min + vec2((k % ts.tw) as f32 * cz, (k / ts.tw) as f32 * cz), vec2(cz, cz));
                    me.glyph_at(&p, pal, c.cs, c.code, cr, color_index(ts, scr, Some(t), c), Self::screen_mc(scr), c.sel, 1.0);
                }
            }
            if size >= 30.0 { // Nummer in der Ecke, damit Kacheln auch ohne Hover zu unterscheiden sind
                let g = p.layout_no_wrap(i.to_string(), egui::FontId::monospace(9.0), Color32::WHITE);
                let bgr = Rect::from_min_size(r.left_bottom() - vec2(0.0, g.size().y + 1.0), g.size() + vec2(3.0, 1.0));
                p.rect_filled(bgr, 0.0, Color32::from_black_alpha(170)); p.galley(bgr.min + vec2(1.5, 0.5), g, Color32::WHITE);
            }
            (r, resp)
        };
        let scr = self.cur_screen().cloned().unwrap_or(Screen { name: String::new(), w: 1, h: 1, colors: Value::Null, set: 0, origin: (0, 0), grid: vec![], patches: vec![] });
        if !self.recent.is_empty() {
            ui.label(RichText::new("zuletzt").small().color(crate::theme::dim()));
            ui.horizontal_wrapped(|ui| { ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                for i in self.recent.clone() { if i >= ts.tiles.len() { continue; } let (r, resp) = draw(self, ui, i, 24.0, &scr);
                    if i == self.brush { ui.painter().rect_stroke(r, 0.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside); }
                    if resp.on_hover_text(format!("Kachel {i}")).clicked() { self.brush = i; self.stamp = None; if self.tool == Tool::Stamp { self.tool = Tool::Pen; } } }
            });
        }
        ui.label(RichText::new(match ts.tiles.get(self.brush) { Some(t) if !t.name.is_empty() => format!("Kachel {} · {}", self.brush, t.name), Some(_) => format!("Kachel {}", self.brush), None => String::new() }).small());
        let cols = (((ui.available_width() - 14.0 + 2.0) / (sz + 2.0)).floor() as usize).max(1);
        self.pal_cols = cols;
        let shift = ui.input(|i| i.modifiers.shift);
        let mut clicked: Option<usize> = None;
        let block = self.stamp.as_ref().filter(|_| self.tool == Tool::Stamp).and(self.pal_anchor).map(|a| (a, self.brush));
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for row in 0..(ts.tiles.len() + cols - 1) / cols { ui.horizontal(|ui| { for i in row * cols..((row + 1) * cols).min(ts.tiles.len()) {
                let (r, resp) = draw(self, ui, i, sz, &scr);
                let in_block = block.map_or(false, |(a, b)| { let (x0, x1, y0, y1) = ((a % cols).min(b % cols), (a % cols).max(b % cols), (a / cols).min(b / cols), (a / cols).max(b / cols)); (x0..=x1).contains(&(i % cols)) && (y0..=y1).contains(&(i / cols)) });
                if i == self.brush || in_block { ui.painter().rect_stroke(r, 0.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside); }
                let name = ts.tiles[i].name.clone();
                if resp.on_hover_text(format!("Kachel {i}{}\nUmschalt-Klick: Block als Stempel", if name.is_empty() { String::new() } else { format!(" {name}") })).clicked() { clicked = Some(i); }
            } }); }
        });
        if let Some(i) = clicked {
            if shift && self.pal_anchor.is_some() && self.pal_anchor != Some(i) {
                let a = self.pal_anchor.unwrap_or(i);
                let (x0, x1, y0, y1) = ((a % cols).min(i % cols), (a % cols).max(i % cols), (a / cols).min(i / cols), (a / cols).max(i / cols));
                self.stamp = Some((y0..=y1).map(|y| (x0..=x1).map(|x| { let k = y * cols + x; if k < ts.tiles.len() { k } else { HOLE } }).collect()).collect());
                self.brush = i; self.tool = Tool::Stamp;
            } else {
                self.brush = i; self.pal_anchor = Some(i); self.stamp = None; self.remember(i);
                if matches!(self.tool, Tool::Stamp | Tool::Pick | Tool::Select | Tool::Erase) { self.tool = Tool::Pen; }
            }
        }
    }

    fn cell_left_ui(&mut self, ui: &mut egui::Ui, ts: &TileSet) {
        studio::caption(ui, "Zellebene");
        let n = self.cur_screen().map_or(0, |s| s.patches.len());
        ui.label(RichText::new(format!("{n} von {MAX_PATCHES} Patches")).color(if n >= MAX_PATCHES { crate::theme::bad() } else { crate::theme::label_gray() }));
        ui.label(RichText::new("Zellen über den Kacheln. „Löschen“ lässt die Kachel wieder durchscheinen.").weak().small());
        if ui.add_enabled(n > 0, egui::Button::new("Alle Patches entfernen")).clicked() { self.undo_point(); if let Some(s) = self.scr_mut() { s.patches.clear(); } self.touched = true; }
        studio::caption(ui, "Auswahl");
        let Some(((x0, y0), (x1, y1))) = self.sel else { ui.label(RichText::new("Mit dem Werkzeug „Auswahl“ ein Rechteck ziehen.").weak().small()); return };
        ui.label(format!("{}×{} ab {x0},{y0}", x1 - x0 + 1, y1 - y0 + 1));
        if ui.button("Zellen entfernen").clicked() { self.undo_point(); if let Some(s) = self.scr_mut() { s.patches.retain(|p| !(p.x >= x0 && p.x <= x1 && p.y >= y0 && p.y <= y1)); } self.touched = true; }
        if ui.add_enabled(ts.tiles.len() < tiles::MAX_TILES, egui::Button::new("Als Kachel anlegen …")).on_hover_text(format!("Die {}×{} Zellen ab der linken oberen Ecke der Auswahl werden eine neue Kachel", ts.tw, ts.th)).clicked() { self.start_new_tile(ts, (x0, y0)); }
    }

    fn start_new_tile(&mut self, ts: &TileSet, at: (usize, usize)) {
        let Some(s) = self.cur_screen() else { return };
        let Some((cells, col)) = block_cells(s, ts, at.0, at.1) else { self.c.status = format!("Ab {},{} passt keine {}×{}-Kachel mehr in den Screen", at.0, at.1, ts.tw, ts.th); return };
        let (ox, oy) = s.origin;
        let (gx, gy) = (at.0.wrapping_sub(ox) / ts.tw, at.1.wrapping_sub(oy) / ts.th);
        let aligned = at.0 >= ox && at.1 >= oy && (at.0 - ox) % ts.tw == 0 && (at.1 - oy) % ts.th == 0 && s.grid.get(gy).map_or(false, |r| gx < r.len());
        let dup = find_dup(ts, &cells, col);
        self.newtile = Some(NewTile { at, cells, col, dup, aligned, replace: aligned, name: format!("neu{}", ts.tiles.len()) });
    }

    /// Rückfrage als schwebendes Fenster: schiebt die Fläche nicht, hat eigenen Hintergrund und Rahmen
    fn newtile_ui(&mut self, ctx: &egui::Context) {
        let Some(mut nt) = self.newtile.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        egui::Window::new("Neue Kachel").id(egui::Id::new(("bank-newtile", &self.c.rel))).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_TOP, vec2(0.0, 90.0)).show(ctx, |ui| {
            ui.set_max_width(340.0);
            ui.label(RichText::new(format!("Neue Kachel aus den Zellen ab {},{}", nt.at.0, nt.at.1)).strong());
            if let Some(d) = nt.dup { ui.label(RichText::new(format!("Es gibt schon Kachel {d} mit gleichem Inhalt: sie wird benutzt, es entsteht keine neue.")).color(crate::theme::warn())); }
            else { ui.horizontal(|ui| { ui.label("Name"); ui.add(egui::TextEdit::singleline(&mut nt.name).desired_width(120.0)); }); }
            ui.add_enabled(nt.aligned, egui::Checkbox::new(&mut nt.replace, "Zellen im Screen durch den Kachelverweis ersetzen")).on_hover_text("Ersetzt die Zellen im Screen durch einen Verweis auf die Kachel");
            if !nt.aligned { ui.label(RichText::new("Ausgegraut: Die Zellen liegen nicht auf dem Kachelraster (Ursprung + Vielfache der Kachelgröße). Die Kachel entsteht trotzdem, der Screen bleibt unverändert.").weak().small()); }
            ui.horizontal(|ui| { if ui.button(if nt.dup.is_some() { "Benutzen" } else { "Anlegen" }).clicked() { ok = true; } if ui.button("Abbrechen").clicked() { cancel = true; } });
            ui.label(RichText::new("Der Kachelsatz wird beim Speichern mitgeschrieben; „Rückgängig“ nimmt nur die Änderung am Screen zurück.").weak().small());
        });
        if cancel { self.newtile = None; return; }
        if ok { self.finish_new_tile(nt); } else { self.newtile = Some(nt); }
    }

    fn finish_new_tile(&mut self, nt: NewTile) {
        self.newtile = None;
        let Some(mut ts) = self.ts.take() else { return };
        let idx = match nt.dup {
            Some(d) => d,
            None => {
                ts.tiles.push(Tile { name: nt.name.clone(), cells: nt.cells.clone(), col: nt.col, props: 0 });
                self.ts_dirty = true; ts.tiles.len() - 1
            }
        };
        self.undo_point();
        if nt.replace { let at = nt.at; if let Some(s) = self.scr_mut() { replace_block(s, &ts, at.0, at.1, idx); } }
        self.c.status = format!("Kachel {idx} {}", if nt.dup.is_some() { "benutzt" } else { "angelegt" });
        self.brush = idx; self.remember(idx);
        self.ts = Some(ts); self.touched = true; self.sel = None;
    }

    fn right_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: Option<&TileSet>) {
        self.c.video_switch(ui);
        // Kachelsatz
        studio::caption(ui, "Kachelsatz");
        let cur = self.bank.tilesets.first().cloned().unwrap_or_default();
        let mut pick = None;
        egui::ComboBox::from_id_salt(("bank-set", &self.c.rel)).width(200.0).selected_text(if cur.is_empty() { "(keiner)".to_string() } else { cur.clone() }).show_ui(ui, |ui| {
            for s in &self.sets { if ui.selectable_label(&cur == s, s).clicked() { pick = Some(s.clone()); } }
            if self.sets.is_empty() { ui.label(RichText::new("Keine Kachelsätze für dieses System im Projekt").weak()); }
        });
        if let Some(s) = pick { self.undo_point(); if self.bank.tilesets.is_empty() { self.bank.tilesets.push(s); } else { self.bank.tilesets[0] = s; } self.load_ts();
            if let Some(ts) = self.ts.clone() { let n = ts.tiles.len(); for sc in self.bank.screens.iter_mut() { fit_grid(sc, &ts); for r in sc.grid.iter_mut() { for t in r.iter_mut() { if *t >= n { *t = 0; } } } } } self.touched = true; }
        if let Some(e) = &self.ts_err { ui.label(RichText::new(e).color(crate::theme::bad()).small()); }
        let Some(ts) = ts else { return };
        let Some(scr) = self.cur_screen().cloned() else { return };
        // Größe und Ursprung
        studio::caption(ui, "Screen");
        let (mut w, mut h, mut ox, mut oy) = (scr.w, scr.h, scr.origin.0, scr.origin.1);
        let mut chg = false; let mut start = false;
        ui.horizontal(|ui| { ui.label("Größe");
            let r = ui.add(egui::DragValue::new(&mut w).range(1..=255usize)); chg |= r.changed(); start |= r.drag_started() || r.gained_focus();
            ui.label("×"); let r = ui.add(egui::DragValue::new(&mut h).range(1..=255usize)); chg |= r.changed(); start |= r.drag_started() || r.gained_focus(); });
        ui.horizontal(|ui| { ui.label("Ursprung");
            let (mx, my) = (w.saturating_sub(1), h.saturating_sub(1));
            let r = ui.add(egui::DragValue::new(&mut ox).range(0..=mx)); chg |= r.changed(); start |= r.drag_started() || r.gained_focus();
            ui.label(","); let r = ui.add(egui::DragValue::new(&mut oy).range(0..=my)); chg |= r.changed(); start |= r.drag_started() || r.gained_focus(); });
        if chg {
            if start { self.undo_point(); }
            if let Some(s) = self.scr_mut() {
                s.w = w; s.h = h; s.origin = (ox.min(w - 1), oy.min(h - 1)); s.patches.retain(|p| p.x < w && p.y < h);
                fit_grid(s, ts);
            }
            self.sel = None; self.touched = true;
        }
        let (cols, rows) = grid_dims(&scr, ts);
        let (rest_r, rest_b) = ((scr.w - scr.origin.0) % ts.tw, (scr.h - scr.origin.1) % ts.th);
        let mut rest = format!("Kachelraster {cols}×{rows}");
        if scr.origin.0 > 0 || scr.origin.1 > 0 { rest.push_str(&format!(" ab {},{}", scr.origin.0, scr.origin.1)); }
        if rest_r > 0 || rest_b > 0 || scr.origin.0 > 0 || scr.origin.1 > 0 { rest.push_str(&format!(" · Restzellen: {} (schraffiert, Zellebene)", [(scr.origin.1, "oben"), (scr.origin.0, "links"), (rest_b, "unten"), (rest_r, "rechts")].iter().filter(|(n, _)| *n > 0).map(|(n, w)| format!("{n} {w}")).collect::<Vec<_>>().join(", "))); }
        ui.label(RichText::new(rest).weak().small());
        // Farben
        studio::caption(ui, "Hintergrund");
        let (bg, border) = (scr.colors["bg"].as_u64().unwrap_or(6) as usize, scr.colors["border"].as_u64().unwrap_or(14) as usize);
        if let Some(i) = self.c.swatches(ui, pal, bg) { self.undo_point(); if let Some(s) = self.scr_mut() { s.colors["bg"] = json!(i); } self.touched = true; }
        if matches!(self.c.target.as_str(), "c64" | "vic20") { studio::caption(ui, "Rahmen"); if let Some(i) = self.c.swatches(ui, pal, border) { self.undo_point(); if let Some(s) = self.scr_mut() { s.colors["border"] = json!(i); } self.touched = true; } }
        else if ts.color_mode == ColorMode::None { ui.label(RichText::new("Vordergrund (Helligkeit)").strong()); if let Some(i) = self.c.swatches(ui, pal, scr.colors["fg"].as_u64().unwrap_or(14) as usize) { self.undo_point(); if let Some(s) = self.scr_mut() { s.colors["fg"] = json!(i); } self.touched = true; } }
        // Zeichenmodus: Zeichenwähler
        if self.mode == Mode::Cell { self.char_picker_ui(ui, pal, ts); }
        // Eigenschaften-Ebene
        if !ts.prop_defs.is_empty() {
            studio::caption(ui, "Eigenschaften-Ebene");
            let cur = self.props_bit.and_then(|b| ts.prop_defs.iter().find(|(_, x)| *x == b).map(|(n, _)| n.clone())).unwrap_or_else(|| "aus".into());
            egui::ComboBox::from_id_salt(("bank-props", &self.c.rel)).selected_text(cur).show_ui(ui, |ui| {
                if ui.selectable_label(self.props_bit.is_none(), "aus").clicked() { self.props_bit = None; }
                for (n, b) in &ts.prop_defs { if ui.selectable_label(self.props_bit == Some(*b), n).clicked() { self.props_bit = Some(*b); } }
            });
        }
        // Vorschau
        studio::caption(ui, "Vorschau");
        ui.checkbox(&mut self.show_preview, "mit Rahmen, echte Farben");
        if self.show_preview {
            let pad_cells = 2.0; let avail = ui.available_width() - 14.0;
            let cz = (avail / (scr.w as f32 + pad_cells * 2.0)).min(8.0).max(2.0);
            let (outer, _) = ui.allocate_exact_size(vec2((scr.w as f32 + pad_cells * 2.0) * cz, (scr.h as f32 + pad_cells * 2.0) * cz), Sense::hover());
            let p = ui.painter_at(outer);
            p.rect_filled(outer, 0.0, pal.color(&self.c.target, border));
            let inner = outer.shrink(pad_cells * cz);
            p.rect_filled(inner, 0.0, self.bg(pal, &scr));
            self.paint_screen(&p, pal, &scr, ts, inner.min, cz, 1.0, 1.0);
            ui.label(RichText::new(if cz >= 8.0 { "1:1 (8 Pixel je Zelle)" } else { "verkleinert, damit es in die Spalte passt" }).weak().small());
        }
    }

    fn char_picker_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet) {
        studio::caption(ui, "Zeichen (Zellebene)");
        if ts.charsets.is_empty() { ui.label(RichText::new("Der Kachelsatz hat keinen Zeichensatz.").weak().small()); return; }
        ui.horizontal_wrapped(|ui| { for (i, p) in ts.charsets.iter().enumerate() {
            if ui.selectable_label(self.tab == i, format!("{} {}", (b'A' + (i % 26) as u8) as char, vc_media::naming::stem(p))).on_hover_text(p).clicked() { self.tab = i; self.cell_brush.cs = i; }
        } });
        let atari = self.c.target == "atari8";
        let n = if atari { 128 } else { 256 };
        let fgi = if ts.color_mode == ColorMode::None { self.cur_screen().map_or(14, |s| s.colors["fg"].as_u64().unwrap_or(14) as usize) } else { self.col };
        let bg = self.cur_screen().map_or(6, |s| s.colors["bg"].as_u64().unwrap_or(6) as usize);
        let s = 18.0;
        ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
        for row in 0..n / 16 { ui.horizontal(|ui| { for i in row * 16..(row + 1) * 16 {
            let (r, resp) = ui.allocate_exact_size(vec2(s, s), Sense::click());
            let p = ui.painter_at(r);
            p.rect_filled(r.shrink(1.0), 0.0, pal.color(&self.c.target, bg));
            let fgi = if matches!(self.c.target.as_str(), "c64" | "mega65" | "vic20") && self.glyphs.get(self.tab).and_then(|g| g.as_ref()).map_or(false, |g| g.mc) { fgi | 8 } else { fgi };
            self.glyph_at(&p, pal, self.tab, i as u8, r.shrink(1.0), fgi, McCols::default(), false, 1.0);
            if self.cell_brush.cs == self.tab && self.cell_brush.code == i as u8 { ui.painter().rect_stroke(r, 1.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside); }
            if resp.on_hover_text(format!("Zeichen {i} ($ {i:02X})")).clicked() { self.cell_brush.code = i as u8; self.cell_brush.cs = self.tab; if matches!(self.tool, Tool::Pick | Tool::Select | Tool::Erase) { self.tool = Tool::Pen; } }
        } }); }
        ui.horizontal(|ui| { ui.label("Zeichen"); let mut code = self.cell_brush.code; if ui.add(egui::DragValue::new(&mut code).range(0..=255)).changed() { self.cell_brush.code = code; self.cell_brush.cs = self.tab; } ui.label(RichText::new(format!("$ {:02X}", self.cell_brush.code)).weak()); });
        if ts.color_mode != ColorMode::None {
            ui.label(RichText::new("Zeichenfarbe").strong());
            if let Some(i) = self.c.swatches(ui, pal, self.col) { self.col = i; }
        }
        if atari { ui.checkbox(&mut self.cell_brush.sel, "Farbwahl (Zeichenbit 7)").on_hover_text("Atari: wählt die zweite Farbgruppe des Zeichens (Bit 7)"); }
    }

    /// Die Zelle, die ein Zeichen-Strich setzt
    fn painted(&self, ts: &TileSet) -> Cell { Cell { col: (ts.color_mode != ColorMode::None).then_some(self.col as u8), ..self.cell_brush.clone() } }

    /// Einheit (Kachelraster-Koordinate oder Zelle) unter dem Zeigerpunkt
    fn unit_at(&self, s: &Screen, ts: &TileSet, inner: Rect, cz: f32, pt: egui::Pos2, clamp: bool) -> Option<(usize, usize)> {
        let (fx, fy) = (((pt.x - inner.left()) / cz).floor() as i32, ((pt.y - inner.top()) / cz).floor() as i32);
        match self.mode {
            Mode::Cell => { let ok = fx >= 0 && fy >= 0 && (fx as usize) < s.w && (fy as usize) < s.h; if ok { Some((fx as usize, fy as usize)) } else if clamp { Some((fx.clamp(0, s.w as i32 - 1) as usize, fy.clamp(0, s.h as i32 - 1) as usize)) } else { None } }
            Mode::Tile => {
                let (cols, rows) = grid_dims(s, ts);
                if cols == 0 || rows == 0 { return None; }
                let (gx, gy) = ((fx - s.origin.0 as i32).div_euclid(ts.tw as i32), (fy - s.origin.1 as i32).div_euclid(ts.th as i32));
                let ok = fx >= s.origin.0 as i32 && fy >= s.origin.1 as i32 && (gx as usize) < cols && (gy as usize) < rows;
                if ok { Some((gx as usize, gy as usize)) } else if clamp { Some((gx.clamp(0, cols as i32 - 1) as usize, gy.clamp(0, rows as i32 - 1) as usize)) } else { None }
            }
        }
    }

    fn put(&mut self, ts: &TileSet, u: (usize, usize), erase: bool) {
        match self.mode {
            Mode::Tile => { let t = if erase { 0 } else { self.brush }; if let Some(c) = self.scr_mut().and_then(|s| s.grid.get_mut(u.1)).and_then(|r| r.get_mut(u.0)) { *c = t; self.touched = true; } }
            Mode::Cell => {
                let cell = self.painted(ts);
                let full = match self.scr_mut() { Some(s) => if erase { remove_patch(s, u.0, u.1); false } else { !set_patch(s, u.0, u.1, cell) }, None => false };
                if full { self.c.status = format!("Höchstens {MAX_PATCHES} Zellen-Patches je Screen"); }
                self.touched = true;
            }
        }
    }

    fn shape(&mut self, ts: &TileSet, a: (usize, usize), b: (usize, usize)) {
        match self.tool {
            Tool::Select => { self.sel = Some(((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)))); }
            Tool::Line | Tool::Rect => {
                let pts = if self.tool == Tool::Line { line_points(a, b) } else { rect_points(a, b, self.filled) };
                self.undo_point();
                for u in pts { self.put(ts, u, false); }
            }
            _ => {}
        }
    }

    fn pick(&mut self, ts: &TileSet, u: (usize, usize)) {
        match self.mode {
            Mode::Tile => { if let Some(&t) = self.cur_screen().and_then(|s| s.grid.get(u.1)).and_then(|r| r.get(u.0)) { self.brush = t; self.stamp = None; self.remember(t); self.tool = Tool::Pen; } }
            Mode::Cell => {
                let Some(s) = self.cur_screen() else { return };
                if let Some((c, col)) = effective(s, ts, u.0, u.1) { self.cell_brush = Cell { col: None, ..c }; self.tab = self.cell_brush.cs.min(ts.charsets.len().saturating_sub(1)); if ts.color_mode != ColorMode::None { self.col = col; } self.tool = Tool::Pen; }
            }
        }
    }

    fn stamp_at(&mut self, u: (usize, usize)) {
        let Some(clip) = self.stamp.clone() else { return };
        self.undo_point();
        if let Some(s) = self.scr_mut() {
            for (dy, row) in clip.iter().enumerate() { for (dx, &t) in row.iter().enumerate() {
                if t == HOLE { continue; }
                if let Some(c) = s.grid.get_mut(u.1 + dy).and_then(|r| r.get_mut(u.0 + dx)) { *c = t; }
            } }
        }
        self.touched = true;
    }

    fn sel_ui(&mut self, ui: &mut egui::Ui, ts: &TileSet) {
        let Some(((x0, y0), (x1, y1))) = self.sel else { return };
        if self.mode != Mode::Tile { return; }
        ui.label(RichText::new(format!("Auswahl {}×{} Kacheln", x1 - x0 + 1, y1 - y0 + 1)).small());
        if ui.button("Kopieren (Stempel)").clicked() {
            let clip: Vec<Vec<usize>> = self.cur_screen().map(|s| (y0..=y1).map(|y| (x0..=x1).map(|x| s.grid.get(y).and_then(|r| r.get(x)).copied().unwrap_or(HOLE)).collect()).collect()).unwrap_or_default();
            self.stamp = Some(clip); self.tool = Tool::Stamp; self.pal_anchor = None;
        }
        if ui.button("Leeren (Kachel 0)").clicked() { self.undo_point(); if let Some(s) = self.scr_mut() { for y in y0..=y1 { for x in x0..=x1 { if let Some(c) = s.grid.get_mut(y).and_then(|r| r.get_mut(x)) { *c = 0; } } } } self.touched = true; }
        let _ = ts;
    }

    fn tab_strip(&mut self, ui: &mut egui::Ui, ts: Option<&TileSet>) {
        ui.horizontal_wrapped(|ui| {
            for i in 0..self.bank.screens.len() {
                let name = if self.bank.screens[i].name.is_empty() { format!("{i}") } else { self.bank.screens[i].name.clone() };
                if ui.selectable_label(self.cur == i, format!("{i} {name}")).on_hover_text("Nummer = Auswahlnummer für screen_select").clicked() { self.cur = i; self.sel = None; self.newtile = None; }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Neu").on_hover_text("Leerer Screen in Größe und Farben des aktuellen").clicked() { self.new_screen(); }
            if ui.button("Duplizieren").clicked() { self.dup_screen(); }
            if ui.add_enabled(self.bank.screens.len() > 1, egui::Button::new("Löschen")).on_hover_text("Screen entfernen (Rückgängig stellt ihn wieder her)").clicked() { self.delete_screen(); }
            if ui.add_enabled(self.cur > 0, egui::Button::new("←")).on_hover_text("Nach vorn").clicked() { self.move_screen(-1); }
            if ui.add_enabled(self.cur + 1 < self.bank.screens.len(), egui::Button::new("→")).on_hover_text("Nach hinten").clicked() { self.move_screen(1); }
            if let Some(s) = self.cur_screen() {
                let mut name = s.name.clone();
                let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(120.0).hint_text("Name"));
                if r.gained_focus() { self.undo_point(); }
                if r.changed() { if let Some(s) = self.scr_mut() { s.name = name; } self.touched = true; }
            }
            if let Some(ts) = ts { self.sel_ui(ui, ts); }
        });
    }

    fn center_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: Option<&TileSet>) {
        self.tab_strip(ui, ts);
        let Some(ts) = ts else {
            ui.add_space(8.0);
            ui.label(self.ts_err.clone().unwrap_or_else(|| "Kein Kachelsatz.".into()));
            return;
        };
        self.newtile_ui(ui.ctx());
        self.view_ui(ui);
        let Some(scr) = self.cur_screen().cloned() else { return };
        if self.fit && self.avail.x > 0.0 {
            // freier Platz = Mitte minus bereits belegte Höhe, Fußzeile und Rolleisten
            let pad = if matches!(self.c.target.as_str(), "c64" | "vic20") { 10.0 } else { 6.0 };
            let w = self.avail.x - 16.0 - 2.0 * pad - 14.0;
            let h = self.avail.y - 16.0 - ui.min_rect().height() - 26.0 - 2.0 * pad - 14.0;
            self.zoom = (w / scr.w as f32).min(h / scr.h as f32).floor().clamp(4.0, 48.0);
        }
        let cz = self.zoom;
        let pad = if matches!(self.c.target.as_str(), "c64" | "vic20") { cz.max(10.0) } else { 6.0 };
        let (outer, resp) = ui.allocate_exact_size(vec2(scr.w as f32 * cz + pad * 2.0, scr.h as f32 * cz + pad * 2.0), Sense::click_and_drag());
        let p = ui.painter_at(outer);
        let inner = outer.shrink(pad);
        p.rect_filled(outer, 0.0, pal.color(&self.c.target, scr.colors["border"].as_u64().unwrap_or(scr.colors["bg"].as_u64().unwrap_or(6)) as usize));
        p.rect_filled(inner, 0.0, self.bg(pal, &scr));
        let (ta, ca) = if self.mode == Mode::Tile { (1.0, 0.45) } else { (0.4, 1.0) };
        self.paint_screen(&p, pal, &scr, ts, inner.min, cz, ta, ca);
        // Restzellen schraffieren
        let (cols, rows) = grid_dims(&scr, ts);
        let covered = |x: usize, y: usize| x >= scr.origin.0 && y >= scr.origin.1 && (x - scr.origin.0) / ts.tw < cols && (y - scr.origin.1) / ts.th < rows;
        let hatch = Stroke::new(1.0, Color32::from_white_alpha(if self.mode == Mode::Tile { 50 } else { 25 }));
        if cz >= 8.0 { for y in 0..scr.h { for x in 0..scr.w { if !covered(x, y) {
            let r = Rect::from_min_size(pos2(inner.left() + x as f32 * cz, inner.top() + y as f32 * cz), vec2(cz, cz));
            p.line_segment([r.left_bottom(), r.right_top()], hatch);
        } } } }
        // Eigenschaften-Ebene
        if let Some(bit) = self.props_bit {
            let colors = [Color32::from_rgb(220, 60, 60), Color32::from_rgb(70, 110, 230), Color32::from_rgb(70, 190, 90), Color32::from_rgb(230, 190, 50), Color32::from_rgb(170, 90, 220), Color32::from_rgb(240, 140, 50), Color32::from_rgb(60, 200, 210), Color32::from_rgb(230, 110, 180)];
            for (gy, row) in scr.grid.iter().enumerate() { for (gx, &t) in row.iter().enumerate() {
                if ts.tiles.get(t).map_or(false, |t| t.props >> bit & 1 == 1) {
                    let r = Rect::from_min_size(pos2(inner.left() + (scr.origin.0 + gx * ts.tw) as f32 * cz, inner.top() + (scr.origin.1 + gy * ts.th) as f32 * cz), vec2(ts.tw as f32 * cz, ts.th as f32 * cz));
                    p.rect_filled(r, 0.0, colors[bit as usize % 8].gamma_multiply(0.45));
                }
            } }
        }
        // Gitter
        if self.cell_grid { let g = Stroke::new(1.0, Color32::from_white_alpha(22));
            for x in 0..=scr.w { let px = inner.left() + x as f32 * cz; p.line_segment([pos2(px, inner.top()), pos2(px, inner.bottom())], g); }
            for y in 0..=scr.h { let py = inner.top() + y as f32 * cz; p.line_segment([pos2(inner.left(), py), pos2(inner.right(), py)], g); } }
        if self.tile_lines && cols > 0 && rows > 0 { let g = Stroke::new(1.0, Color32::from_white_alpha(70));
            for gx in 0..=cols { let px = inner.left() + (scr.origin.0 + gx * ts.tw) as f32 * cz; p.line_segment([pos2(px, inner.top() + scr.origin.1 as f32 * cz), pos2(px, inner.top() + (scr.origin.1 + rows * ts.th) as f32 * cz)], g); }
            for gy in 0..=rows { let py = inner.top() + (scr.origin.1 + gy * ts.th) as f32 * cz; p.line_segment([pos2(inner.left() + scr.origin.0 as f32 * cz, py), pos2(inner.left() + (scr.origin.0 + cols * ts.tw) as f32 * cz, py)], g); } }
        // Zoom mit Strg/Cmd + Rad
        if resp.hovered() { let z = ui.input(|i| i.zoom_delta()); if z != 1.0 { self.zoom = (self.zoom * z).clamp(6.0, 48.0); } }
        // Zeiger
        let pt = resp.interact_pointer_pos().or(resp.hover_pos());
        let hover = pt.and_then(|pt| self.unit_at(&scr, ts, inner, cz, pt, false));
        let near = pt.and_then(|pt| self.unit_at(&scr, ts, inner, cz, pt, true));
        let tool = self.tool;
        if let Some(u) = near {
            if resp.secondary_clicked() { if let Some(h) = hover { self.pick(ts, h); } }
            else if resp.drag_started_by(egui::PointerButton::Primary) {
                resp.request_focus();
                self.drag = Some(u); self.last = Some(u); self.textpos = u;
                if matches!(tool, Tool::Pen | Tool::Erase) { self.undo_point(); self.put(ts, u, tool == Tool::Erase); }
            } else if resp.clicked_by(egui::PointerButton::Primary) {
                resp.request_focus();
                if let Some(u) = hover {
                    self.textpos = u;
                    match tool {
                        Tool::Pen | Tool::Erase => { self.undo_point(); self.put(ts, u, tool == Tool::Erase); }
                        Tool::Line | Tool::Rect | Tool::Select => self.shape(ts, u, u),
                        Tool::Fill => { if self.mode == Mode::Tile { self.undo_point(); let b = self.brush; if let Some(s) = self.scr_mut() { flood_tiles(&mut s.grid, u.0, u.1, b); } self.touched = true; } }
                        Tool::Pick => self.pick(ts, u),
                        Tool::Stamp => self.stamp_at(u),
                        Tool::Text => {}
                    }
                }
            } else if resp.dragged_by(egui::PointerButton::Primary) && matches!(tool, Tool::Pen | Tool::Erase) {
                let from = self.last.unwrap_or(u);
                for q in line_points(from, u) { self.put(ts, q, tool == Tool::Erase); }
                self.last = Some(u);
            }
            if resp.drag_stopped() { if let Some(a) = self.drag.take() { if matches!(tool, Tool::Line | Tool::Rect | Tool::Select) { self.shape(ts, a, u); } } self.last = None; }
        }
        // Vorschau von Linie, Rechteck, Auswahl und Stempel
        let ghost = Color32::from_white_alpha(60);
        if let (Some(a), Some(b)) = (self.drag, near) {
            let pts = match tool { Tool::Line => line_points(a, b), Tool::Rect => rect_points(a, b, self.filled), Tool::Select => rect_points(a, b, true), _ => vec![] };
            for u in pts { p.rect_filled(self.unit_rect(&scr, ts, inner, cz, u), 0.0, ghost); }
        }
        if tool == Tool::Stamp && self.mode == Mode::Tile { if let (Some(u), Some(clip)) = (hover, &self.stamp) { for (dy, row) in clip.iter().enumerate() { for (dx, &t) in row.iter().enumerate() { if t != HOLE && u.1 + dy < rows && u.0 + dx < cols { p.rect_stroke(self.unit_rect(&scr, ts, inner, cz, (u.0 + dx, u.1 + dy)), 0.0, Stroke::new(2.0, crate::theme::accent()), egui::StrokeKind::Inside); } } } } }
        else if let Some(u) = hover { p.rect_stroke(self.unit_rect(&scr, ts, inner, cz, u), 0.0, Stroke::new(1.5, crate::theme::select()), egui::StrokeKind::Inside); }
        if let Some(((x0, y0), (x1, y1))) = self.sel {
            let r = self.unit_rect(&scr, ts, inner, cz, (x0, y0)).union(self.unit_rect(&scr, ts, inner, cz, (x1, y1)));
            p.rect_stroke(r, 0.0, Stroke::new(2.0, crate::theme::warn()), egui::StrokeKind::Inside);
        }
        // Text tippen
        if tool == Tool::Text && self.mode == Mode::Cell && resp.has_focus() {
            let (typed, back, enter) = ui.input(|i| (i.events.iter().filter_map(|e| if let egui::Event::Text(t) = e { Some(t.clone()) } else { None }).collect::<String>(), i.key_pressed(egui::Key::Backspace), i.key_pressed(egui::Key::Enter)));
            if !typed.is_empty() || back || enter { self.undo_point(); }
            for ch in typed.chars() {
                let (x, y) = self.textpos;
                let cell = Cell { code: screen_code(&self.c.target, ch), ..self.painted(ts) };
                if let Some(s) = self.scr_mut() { if !set_patch(s, x, y, cell) { self.c.status = format!("Höchstens {MAX_PATCHES} Zellen-Patches je Screen"); break; } }
                self.textpos = if x + 1 < scr.w { (x + 1, y) } else { (0, (y + 1).min(scr.h - 1)) }; self.touched = true;
            }
            if back { let (x, y) = self.textpos; let np = if x > 0 { (x - 1, y) } else if y > 0 { (scr.w - 1, y - 1) } else { (0, 0) }; if let Some(s) = self.scr_mut() { remove_patch(s, np.0, np.1); } self.textpos = np; self.touched = true; }
            if enter { self.textpos = (0, (self.textpos.1 + 1).min(scr.h - 1)); }
            p.rect_stroke(Rect::from_min_size(pos2(inner.left() + self.textpos.0 as f32 * cz, inner.top() + self.textpos.1 as f32 * cz), vec2(cz, cz)), 0.0, Stroke::new(2.0, crate::theme::warn()), egui::StrokeKind::Inside);
        }
        if resp.has_focus() && self.sel.is_some() && ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) && tool != Tool::Text {
            if let Some(((x0, y0), (x1, y1))) = self.sel { self.undo_point(); let cellm = self.mode == Mode::Cell; if let Some(s) = self.scr_mut() {
                if cellm { s.patches.retain(|q| !(q.x >= x0 && q.x <= x1 && q.y >= y0 && q.y <= y1)); } else { for y in y0..=y1 { for x in x0..=x1 { if let Some(c) = s.grid.get_mut(y).and_then(|r| r.get_mut(x)) { *c = 0; } } } }
            } self.touched = true; }
        }
        // Fußzeile unter dem Bild
        let at = hover.map(|u| match self.mode { Mode::Tile => format!("Kachel {},{} → {}", u.0, u.1, scr.grid.get(u.1).and_then(|r| r.get(u.0)).map_or("-".into(), |t| format!("{t}"))), Mode::Cell => format!("Zelle {},{}{}", u.0, u.1, patch_at(&scr, u.0, u.1).map_or(String::new(), |i| format!(" · Patch Zeichen {}", scr.patches[i].cell.code))) }).unwrap_or_default();
        ui.label(RichText::new(format!("{}×{} Zellen · {} · {at}", scr.w, scr.h, if self.mode == Mode::Tile { "Kachelmodus (Tab wechselt)" } else { "Zeichenmodus (Tab wechselt)" })).weak().small());
    }
}

impl MediaEd for BankEd {
    fn dirty(&self) -> bool { self.c.dirty() || self.ts_dirty }
    fn save(&mut self) -> Result<(), String> {
        if self.ts_dirty {
            if let (Some(ts), Some(rel)) = (&self.ts, self.bank.tilesets.first()) {
                let v = tiles::tileset_json(ts, &self.ts_base);
                std::fs::write(self.c.root.join(rel), vc_media::project::format_media(&v)).map_err(|e| format!("{rel}: {e}"))?;
                self.ts_mtime = std::fs::metadata(self.c.root.join(rel)).and_then(|m| m.modified()).ok(); self.ts_base = v; self.ts_dirty = false;
            }
        }
        if self.touched { self.commit(); }
        self.c.save()
    }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let (pal, id) = (env.pal, self.c.rel.clone());
        self.sync(); self.watch_ts();
        self.c.head(ui);
        studio::tools(ui, &id, |ui| self.tools_ui(ui));
        let info = match (&self.ts, self.cur_screen()) { (Some(ts), Some(s)) => format!("Screen {} von {} · {} Kacheln {}×{} · {} Patches", self.cur + 1, self.bank.screens.len(), ts.tiles.len(), ts.tw, ts.th, s.patches.len()), _ => String::new() };
        self.c.foot(ui, env, &info);
        let ts = self.ts.clone();
        studio::left(ui, &id, 190.0, |ui| match (&ts, self.mode) {
            (Some(ts), Mode::Tile) => self.palette_ui(ui, pal, ts),
            (Some(ts), Mode::Cell) => self.cell_left_ui(ui, ts),
            _ => { ui.label(RichText::new("Kein Kachelsatz").weak()); }
        });
        studio::right(ui, &id, 320.0, |ui| self.right_ui(ui, pal, ts.as_ref()));
        self.avail = ui.available_size();
        studio::center(ui, |ui| self.center_ui(ui, pal, ts.as_ref()));
        if self.touched { self.commit(); } else { self.sync(); }
        if !ui.ctx().egui_wants_keyboard_input() && ui.input(|i| i.key_pressed(egui::Key::Tab)) { self.set_mode(if self.mode == Mode::Tile { Mode::Cell } else { Mode::Tile }); }
        let _ = &self.data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../") }
    fn project() -> (PathBuf, PathBuf) {
        let data = root().join("data");
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!("vc-bank-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        std::fs::copy(root().join("examples/c64/gfx_demo/graphics/font.vcchar"), dir.join("graphics/font.vcchar")).unwrap();
        (dir, data)
    }
    /// Projekt mit Kachelsatz (3 Kacheln 2×2) und Bank mit zwei Screens
    fn with_bank() -> (PathBuf, PathBuf, String) {
        let (dir, data) = project();
        let t = |name: &str, c: [u8; 4], col: u8| json!({ "name": name, "cells": c, "col": col });
        let set = json!({ "format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "hg", "size": { "w": 2, "h": 2 }, "colorMode": "tile", "charsets": ["graphics/font.vcchar"],
            "propDefs": [{ "name": "solid", "bit": 0 }], "tiles": [t("leer", [32, 32, 32, 32], 1), t("a", [1, 2, 3, 4], 5), { "name": "b", "cells": [5, 6, 7, 8], "col": 7, "props": ["solid"] }] });
        std::fs::write(dir.join("graphics/hg.vctiles"), vc_media::project::format_media(&set)).unwrap();
        let bank = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "lv", "tilesets": ["graphics/hg.vctiles"], "screens": [
            { "name": "eins", "w": 8, "h": 5, "colors": { "border": 0, "bg": 3 }, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 1, 2, 0], [1, 1, 0, 2]] }, "cells": { "patches": [[0, 4, 72], [1, 4, [73, 0, 5]]] } },
            { "name": "zwei", "w": 8, "h": 5, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 0, 0, 0], [0, 0, 0, 0]] } }] });
        let rel = "graphics/lv.vcscreen".to_string();
        std::fs::write(dir.join(&rel), vc_media::project::format_media(&bank)).unwrap();
        (dir, data, rel)
    }
    fn blank() -> Cell { Cell { code: 0, cs: 0, col: None, sel: false } }

    #[test]
    fn rechnen_linie_rechteck_fuellen() {
        assert_eq!(line_points((0, 0), (3, 0)), vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
        assert_eq!(line_points((2, 2), (0, 0)).len(), 3);
        assert_eq!(rect_points((0, 0), (2, 2), false).len(), 8); assert_eq!(rect_points((0, 0), (2, 2), true).len(), 9);
        let mut g = vec![vec![0, 0, 1], vec![0, 1, 1], vec![0, 0, 0]];
        flood_tiles(&mut g, 0, 0, 7); assert_eq!(g, vec![vec![7, 7, 1], vec![7, 1, 1], vec![7, 7, 7]]);
    }

    #[test]
    fn bank_rundreise_und_patch_grenze() {
        let (dir, _data, rel) = with_bank();
        let v: Value = serde_json::from_str(&std::fs::read_to_string(dir.join(&rel)).unwrap()).unwrap();
        let b = tiles::parse_bank(&v).unwrap();
        let back = bank_json(&b, &v);
        let b2 = tiles::parse_bank(&back).unwrap();
        assert_eq!((b2.screens.len(), b2.screens[0].patches.len(), b2.screens[0].grid.clone()), (2, 2, b.screens[0].grid.clone()));
        assert_eq!(b2.screens[0].patches[1].cell, Cell { code: 73, cs: 0, col: Some(5), sel: false });
        assert_eq!(back["name"], "lv");
        let mut s2 = b.screens[0].clone(); s2.w = 255; s2.h = 255;
        s2.patches.clear();
        let mut n = 0; for y in 0..255 { for x in 0..255 { if set_patch(&mut s2, x, y, blank()) { n += 1; } else { break; } } if n >= MAX_PATCHES { break; } }
        assert_eq!(s2.patches.len(), MAX_PATCHES); assert!(!set_patch(&mut s2, 200, 200, blank()), "256. Patch wird abgelehnt");
        assert!(set_patch(&mut s2, 0, 0, Cell { code: 9, ..blank() }), "vorhandener Patch lässt sich überschreiben"); assert_eq!(s2.patches.len(), MAX_PATCHES);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn auswahl_zu_kachel_duplikat_und_ersetzen() {
        let (dir, data, rel) = with_bank();
        let ctx = egui::Context::default();
        let mut e = BankEd::open(&ctx, &dir, &rel, &data).unwrap();
        let ts = e.ts.clone().unwrap();
        // Block (2,0): im Kachelraster Kachel 2 (cells 5,6,7,8, Farbe 7), also Duplikat von Kachel 2
        let s = e.bank.screens[0].clone();
        let (cells, col) = block_cells(&s, &ts, 4, 0).unwrap();
        assert_eq!((cells.iter().map(|c| c.code).collect::<Vec<_>>(), col), (vec![5, 6, 7, 8], 7));
        assert_eq!(find_dup(&ts, &cells, col), Some(2));
        // Block ab Zeile 4 reicht über den unteren Rand (h=5)
        assert!(block_cells(&s, &ts, 0, 4).is_none());
        // Neue Kachel aus Block (0,0) mit Patch darüber
        e.bank.screens[0].patches.push(Patch { x: 0, y: 0, cell: Cell { code: 40, cs: 0, col: Some(9), sel: false } });
        let ts = e.ts.clone().unwrap();
        e.start_new_tile(&ts, (0, 0));
        let nt = e.newtile.clone().unwrap();
        assert!(nt.aligned && nt.dup.is_none() && nt.cells[0].code == 40);
        e.finish_new_tile(nt);
        assert_eq!(e.ts.as_ref().unwrap().tiles.len(), 4); assert!(e.ts_dirty && e.touched);
        assert_eq!(e.bank.screens[0].grid[0][0], 3, "Zellen im Screen durch den Kachelverweis ersetzt");
        assert!(!e.bank.screens[0].patches.iter().any(|p| p.x == 0 && p.y == 0), "Patch im Block entfernt");
        // Duplikat wird wiederverwendet
        e.start_new_tile(&e.ts.clone().unwrap(), (4, 0));
        assert_eq!(e.newtile.as_ref().unwrap().dup, Some(2));
        let nt = e.newtile.clone().unwrap(); e.finish_new_tile(nt);
        assert_eq!(e.ts.as_ref().unwrap().tiles.len(), 4, "keine neue Kachel bei Duplikat");
        // unausgerichtet: kein Ersetzen
        e.start_new_tile(&e.ts.clone().unwrap(), (1, 0));
        let nt = e.newtile.clone().unwrap(); assert!(!nt.aligned && !nt.replace);
        // speichern: Satz und Bank
        e.newtile = None; e.save().unwrap();
        let t2: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/hg.vctiles")).unwrap()).unwrap();
        assert_eq!(t2["tiles"].as_array().unwrap().len(), 4);
        let e2 = BankEd::open(&ctx, &dir, &rel, &data).unwrap();
        assert_eq!(e2.bank.screens[0].grid[0][0], 3);
        assert!(!e.dirty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bank_oeffnen_bearbeiten_speichern_rundreise() {
        let (dir, data, rel) = with_bank();
        let ctx = egui::Context::default();
        let mut e = BankEd::open(&ctx, &dir, &rel, &data).unwrap();
        assert_eq!((e.bank.screens.len(), e.ts.as_ref().unwrap().tiles.len()), (2, 3));
        assert!(e.glyphs.first().map_or(false, |g| g.is_some()), "Zeichensatz des Kachelsatzes geladen");
        let ts = e.ts.clone().unwrap();
        // Kachel setzen (Stift), Rückgängig
        e.brush = 2; e.undo_point(); e.put(&ts, (1, 1), false); e.commit();
        assert_eq!(e.bank.screens[0].grid[1][1], 2); assert!(e.dirty());
        e.c.undo(); e.sync(); assert_eq!(e.bank.screens[0].grid[1][1], 1, "Rückgängig stellt die Kachel wieder her");
        e.c.redo(); e.sync(); assert_eq!(e.bank.screens[0].grid[1][1], 2);
        // Screen neu, duplizieren, Reihenfolge, löschen
        e.new_screen(); e.commit(); assert_eq!(e.bank.screens.len(), 3); assert_eq!(e.bank.screens[2].grid.len(), 2);
        e.dup_screen(); e.commit(); assert_eq!(e.bank.screens.len(), 4); assert_eq!(e.cur, 3);
        e.move_screen(-1); e.commit(); assert_eq!(e.cur, 2);
        let names: Vec<String> = e.bank.screens.iter().map(|s| s.name.clone()).collect(); assert_eq!(names.len(), names.iter().collect::<std::collections::HashSet<_>>().len(), "Namen eindeutig: {names:?}");
        e.delete_screen(); e.commit(); assert_eq!(e.bank.screens.len(), 3);
        // Zellebene
        e.cur = 0; let cell = Cell { code: 12, col: Some(2), ..blank() };
        e.mode = Mode::Cell; e.cell_brush.code = 12; e.col = 2; e.put(&ts, (7, 4), false); e.put(&ts, (0, 4), true); e.commit();
        assert!(e.bank.screens[0].patches.iter().any(|p| p.x == 7 && p.y == 4 && p.cell == cell));
        assert!(!e.bank.screens[0].patches.iter().any(|p| p.x == 0 && p.y == 4), "Löschen entfernt den Patch");
        // Ursprung und Restzellen
        let mut s = e.bank.screens[0].clone(); s.origin = (1, 1); fit_grid(&mut s, &ts);
        assert_eq!(grid_dims(&s, &ts), (3, 2)); assert_eq!(s.grid[0].len(), 3);
        e.save().unwrap();
        let e2 = BankEd::open(&ctx, &dir, &rel, &data).unwrap();
        assert_eq!(e2.bank.screens.len(), 3); assert_eq!(e2.bank.screens[0].grid[1][1], 2);
        assert!(e2.bank.screens[0].patches.iter().any(|p| p.x == 7 && p.y == 4 && p.cell == cell));
        // gespeichert im Format des Modells: lässt sich exportieren
        let out = vc_media::project::export(&dir, &data, &rel, "asm", Some("lv"), None, 10, 10).unwrap();
        assert!(out["files"].as_array().map_or(false, |f| !f.is_empty()), "{out}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn alte_zeichen_screens_werden_zerlegt() {
        let (dir, data) = project();
        let profiles = vc_media::load_profiles(&data).unwrap();
        let p = vc_media::profile(&profiles, "c64.screen.text").unwrap().clone();
        let tp = "c64.tileset";
        // 40×25, 2×2-Raster: Muster aus vier Kacheln, mehrfach wiederholt; Zeile 24 mit Text
        let (w, h) = (p.width, p.height);
        let mut chars = vec![vec![32u8; w]; h]; let mut attrs = vec![vec![1u8; w]; h];
        for y in 0..24 { for x in 0..w { chars[y][x] = [65u8, 66, 67, 68][(x % 2) + (y % 2) * 2]; attrs[y][x] = if (x / 2 + y / 2) % 3 == 0 { 5 } else { 7 }; } }
        for (i, c) in "SCORE".bytes().enumerate() { chars[24][i] = c & 63; }
        let old = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "title", "charset": "graphics/font.vcchar", "colors": { "border": 0, "bg": 6 }, "chars": chars, "attrs": attrs });
        let d = decompose(&old, &p, tp, 2, 2, (0, 0), "title").unwrap();
        assert_eq!(d.color_mode, ColorMode::Tile);
        assert_eq!(d.tiles, 2, "gleiche Blöcke zählen einmal, zwei Farben");
        assert_eq!(d.patches, 5, "SCORE in der 25. Zeile bleibt Zellebene");
        assert_eq!(d.bytes_before, 2000);
        assert!(d.bytes_after < d.bytes_before / 4, "{} von {}", d.bytes_after, d.bytes_before);
        // zu viele verschiedene Blöcke melden einen Fehler
        let mut wild = chars.clone(); for y in 0..24 { for x in 0..w { wild[y][x] = ((x * 7 + y * 13 + x * y) % 200) as u8 + 33; } }
        let r = decompose(&json!({ "kind": "screen", "profile": "c64.screen.text", "charset": null, "chars": wild, "attrs": attrs }), &p, tp, 1, 1, (0, 0), "w");
        assert!(r.is_err(), "mehr als 256 Kacheln");
        // schreiben und mit dem neuen Editor öffnen
        let mut d = d;
        std::fs::write(dir.join("graphics/title.vcscreen"), "{}").unwrap();
        let (set, bank) = write_decomp(&dir, "graphics/title.vcscreen", &mut d).unwrap();
        assert_eq!((set.as_str(), bank.as_str()), ("graphics/title_tiles.vctiles", "graphics/title_bank.vcscreen"));
        assert!(write_decomp(&dir, "graphics/title.vcscreen", &mut d).is_err(), "bestehende Dateien werden nicht überschrieben");
        let ctx = egui::Context::default();
        let e = BankEd::open(&ctx, &dir, &bank, &data).unwrap();
        let ts = e.ts.as_ref().unwrap();
        assert_eq!((ts.tiles.len(), e.bank.screens[0].grid.len(), e.bank.screens[0].grid[0].len(), e.bank.screens[0].patches.len()), (2, 12, 20, 5));
        // Anzeige entspricht dem alten Bild: Zelle (3,5) liefert das Zeichen des alten Screens
        let (c, col) = effective(&e.bank.screens[0], ts, 3, 5).unwrap();
        assert_eq!((c.code, col), (chars[5][3], attrs[5][3] as usize));
        // Export der neuen Dateien klappt
        let out = vc_media::project::export(&dir, &data, &bank, "asm", Some("title"), None, 10, 10).unwrap();
        assert!(out["files"].as_array().map_or(false, |f| !f.is_empty()), "{out}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn screen_bank_editor_zeichnet_mit_exportdialog_und_in_kleinen_fenstern() {
        let (dir, data, rel) = with_bank();
        let ctx = egui::Context::default();
        let pal = Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        let mut e = BankEd::open(&ctx, &dir, &rel, &data).unwrap();
        e.c.exp = Some(crate::exportdlg::ExportDlg::new(&rel, "screen", "lv"));
        e.props_bit = Some(0);
        let run = |e: &mut BankEd, w: f32, h: f32| { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| MediaEd::ui(e, ui, &env)); o.textures_delta.clear(); } };
        run(&mut e, 1100.0, 760.0);
        let (sw, sh) = { let sc = e.cur_screen().unwrap(); (sc.w as f32, sc.h as f32) };
        assert!(e.fit && e.zoom >= 4.0 && e.zoom * sw <= 590.0 && e.zoom * sh <= 600.0, "Zoom passt zur Fläche: {}", e.zoom);
        let z_gross = e.zoom; run(&mut e, 640.0, 480.0); assert!(e.zoom < z_gross, "kleineres Fenster, kleinerer Zoom");
        run(&mut e, 1100.0, 760.0);
        for mode in [Mode::Tile, Mode::Cell] {
            e.set_mode(mode);
            for tool in [Tool::Pen, Tool::Select, Tool::Stamp, Tool::Text] { e.tool = tool; run(&mut e, 1100.0, 760.0); }
            for (w, h) in [(330.0, 500.0), (640.0, 480.0)] { run(&mut e, w, h); assert!(e.c.exp.is_some(), "Export-Dialog offen bei {w}x{h}"); }
        }
        e.sel = Some(((0, 0), (1, 1))); let ts = e.ts.clone().unwrap(); e.start_new_tile(&ts, (0, 0)); run(&mut e, 640.0, 480.0); assert!(e.newtile.is_some());
        // Bank ohne Kachelsatz: Hinweis statt Absturz
        e.bank.tilesets.clear(); e.load_ts(); run(&mut e, 800.0, 600.0); assert!(e.ts.is_none() && e.ts_err.is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn zerlegen_dialog_im_alten_editor_zeichnet() {
        let (dir, data) = project();
        let (pal, audio, reqs) = (Palettes::load(&data), crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
        let ctx = egui::Context::default(); crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        let rel = vc_media::project::create(&dir, &data, "title", "c64.screen.text", "").unwrap()["path"].as_str().unwrap().to_string();
        let mut a = crate::screen::ScreenEd::open(&ctx, &dir, &rel, &data).unwrap();
        a.open_decomp();
        for (w, h) in [(1000.0, 700.0), (330.0, 500.0)] { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| MediaEd::ui(&mut a, ui, &env)); o.textures_delta.clear(); } }
        assert!(a.has_decomp(), "Dialog bleibt offen");
        std::fs::remove_dir_all(&dir).ok();
    }
}
