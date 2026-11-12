// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachel-Editor für Kachelsätze (Art `tileset`, Datei `.vctiles`; Modell in `vc_media::tiles`).
//! Links die Kachelliste mit Verwendungszahlen, in der Mitte die große Kachel (Zelle anklicken, Zeichen setzen), rechts Zeichenwähler mit
//! einem Reiter je Zeichensatz, Farbe, Eigenschaften und der Füllstand des Zielzeichensatzes. Alte Kachelkarten (`tilemap`) werden erkannt
//! und lassen sich mit `vc_media::project::migrate_tiles` umwandeln. Konzept: `assets/docs/Konzept-Kachel-und-Screen-Editor.md`, Abschnitt 4.1.
use crate::bitmap::{Env, MediaEd, Palettes};
use crate::screen::{load_charset, Common, Glyphs, McCols};
use crate::ui::studio;
use eframe::egui::{self, pos2, vec2, Color32, Rect, RichText, Sense, Stroke};
use serde_json::json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use vc_media::tiles::{self, Cell, ColorMode, Tile, TileSet, MAX_SIDE, MAX_TILES};

/// Eine Änderung am Kachelsatz; `true` heißt: vorher einen Rückgängig-Schritt anlegen
type Edit = (bool, Box<dyn FnOnce(&mut TileSet)>);

/// Verwendung des Satzes in einer Screen-Bank des Projekts
struct BankUse { rel: String, screens: usize, counts: Vec<usize> }

/// Füllstand des Zielzeichensatzes nach `merge_glyphs`
#[derive(Clone, Default, PartialEq, Debug)]
struct Fill { used: usize, max: usize, doubles: usize, err: Option<String> }

/// Rückfrage, die in der Mitte als Leiste erscheint
#[derive(Clone)]
enum Ask { Delete(usize), Resize(usize, usize) }

pub struct TileEd {
    c: Common, data: PathBuf,
    /// `None` bei einer alten Kachelkarte (`tilemap`)
    ts: Option<TileSet>, legacy: bool, legacy_report: Vec<String>,
    synced: String, glyphs: Vec<Option<Glyphs>>, glyph_paths: Vec<String>,
    sel: usize, cur: usize, tab: usize, brush: Cell, col: usize, zoom: f32, auto_zoom: bool, size_in: (usize, usize),
    banks: Vec<BankUse>, fill: Option<Fill>, fill_key: String, used: HashSet<(usize, u8)>, dups: Vec<Option<usize>>,
    ask: Option<Ask>, clip: Option<Vec<Cell>>, new_prop: String, profiles: Vec<vc_media::Profile>,
    /// Profil-Kennung je Zeichensatz des Projekts (nur Zeichensätze mit gleichem Profil lassen sich in einem Satz mischen)
    cs_profile: std::collections::HashMap<String, String>,
    /// Gelöschte Kachel, deren Verweise in den Screen-Bänken am Ende des Bildes umgeschrieben werden
    renumber: Option<usize>, renumber_to: usize, filter: String,
}

fn blank() -> Cell { Cell { code: 0, cs: 0, col: None, sel: false } }

// ---------------------------------------------------------------------------------------------- Änderungen an Zellen (frei, testbar)

fn flip_h(c: &mut [Cell], w: usize, h: usize) { for y in 0..h { c[y * w..(y + 1) * w].reverse(); } }
fn flip_v(c: &mut [Cell], w: usize, h: usize) { for y in 0..h / 2 { for x in 0..w { c.swap(y * w + x, (h - 1 - y) * w + x); } } }
/// Um 90° im Uhrzeigersinn; nur bei quadratischer Kachel
fn rotate(c: &mut Vec<Cell>, n: usize) { let old = c.clone(); for y in 0..n { for x in 0..n { c[y * n + x] = old[(n - 1 - x) * n + y].clone(); } } }
/// Zellen rollen (am Rand wieder hinein)
fn shift(c: &mut Vec<Cell>, w: usize, h: usize, dx: i32, dy: i32) {
    let old = c.clone();
    for y in 0..h { for x in 0..w { let (sx, sy) = ((x as i32 - dx).rem_euclid(w as i32) as usize, (y as i32 - dy).rem_euclid(h as i32) as usize); c[y * w + x] = old[sy * w + sx].clone(); } }
}
/// Größe ändern: beschneiden oder mit leeren Zellen auffüllen
fn resized(ts: &mut TileSet, w: usize, h: usize) {
    let (ow, oh) = (ts.tw, ts.th);
    for t in &mut ts.tiles { t.cells = (0..h * w).map(|i| { let (x, y) = (i % w, i / w); if x < ow && y < oh { t.cells[y * ow + x].clone() } else { blank() } }).collect(); }
    ts.tw = w; ts.th = h;
}
fn loses_cells(ts: &TileSet, w: usize, h: usize) -> bool {
    ts.tiles.iter().any(|t| t.cells.iter().enumerate().any(|(i, c)| { let (x, y) = (i % ts.tw, i / ts.tw); (x >= w || y >= h) && *c != blank() }))
}

/// Alle Verweise auf Kachelsätze in Screen-Bänken des Projekts, die `set_rel` nennen
fn scan_banks(root: &Path, set_rel: &str, tile_count: usize) -> Vec<BankUse> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with('.') || n == "build" || n == "target" { continue; }
            if p.is_dir() { if depth < 5 { walk(&p, depth + 1, out); } } else if n.ends_with(".vcscreen") { out.push(p); } }
    }
    let mut files = vec![]; walk(root, 0, &mut files); files.sort();
    let mut out = vec![];
    for f in files {
        let Ok(v) = std::fs::read_to_string(&f).map_err(|_| ()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|_| ())) else { continue };
        if !tiles::is_bank(&v) { continue; }
        let Some(idx) = v["tilesets"].as_array().and_then(|a| a.iter().position(|x| x.as_str() == Some(set_rel))) else { continue };
        let (mut counts, mut screens) = (vec![0usize; tile_count], 0);
        for s in v["screens"].as_array().into_iter().flatten() {
            if s["tiles"]["set"].as_u64().unwrap_or(0) as usize != idx { continue; }
            let mut any = false;
            for r in s["tiles"]["grid"].as_array().into_iter().flatten() { for t in r.as_array().into_iter().flatten() { if let Some(n) = t.as_u64() { any = true; if let Some(c) = counts.get_mut(n as usize) { *c += 1; } } } }
            if any { screens += 1; }
        }
        out.push(BankUse { rel: f.strip_prefix(root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default(), screens, counts });
    }
    out
}

/// Nach dem Löschen von Kachel `del`: Verweise auf `del` werden zu `to` (beim Zusammenführen die gleiche Kachel, sonst 0), höhere Nummern rücken nach. Liefert die Zahl geänderter Bänke.
fn remap_banks(root: &Path, set_rel: &str, del: usize, to: usize) -> Result<usize, String> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with('.') || n == "build" || n == "target" { continue; }
            if p.is_dir() { if depth < 5 { walk(&p, depth + 1, out); } } else if n.ends_with(".vcscreen") { out.push(p); } }
    }
    let mut files = vec![]; walk(root, 0, &mut files);
    let mut changed = 0;
    for f in files {
        let Ok(mut v) = std::fs::read_to_string(&f).map_err(|_| ()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(|_| ())) else { continue };
        if !tiles::is_bank(&v) { continue; }
        let Some(idx) = v["tilesets"].as_array().and_then(|a| a.iter().position(|x| x.as_str() == Some(set_rel))) else { continue };
        let mut any = false;
        for s in v["screens"].as_array_mut().into_iter().flatten() {
            if s["tiles"]["set"].as_u64().unwrap_or(0) as usize != idx { continue; }
            for r in s["tiles"]["grid"].as_array_mut().into_iter().flatten() { for t in r.as_array_mut().into_iter().flatten() {
                if let Some(n) = t.as_u64() { let n = n as usize; let m = if n == del { to } else if n > del { n - 1 } else { n }; if m != n { *t = json!(m); any = true; } }
            } }
        }
        if any { std::fs::write(&f, vc_media::project::format_media(&v)).map_err(|e| format!("{}: {e}", f.display()))?; changed += 1; }
    }
    Ok(changed)
}

impl TileEd {
    pub fn open(ctx: &egui::Context, root: &Path, rel: &str, data: &Path) -> Result<TileEd, String> {
        let c = Common::new(ctx, root, rel, data)?;
        let profiles = vc_media::load_profiles(data)?;
        let legacy = c.media["kind"] == "tilemap";
        let ts = if legacy { None } else { Some(tiles::parse_tileset(&c.media).map_err(|e| format!("{rel}: {e}"))?) };
        let size_in = ts.as_ref().map_or((2, 2), |t| (t.tw, t.th));
        let synced = c.media.to_string();
        let mut e = TileEd { c, data: data.into(), ts, legacy, legacy_report: vec![], synced, glyphs: vec![], glyph_paths: vec![], sel: 0, cur: 0, tab: 0,
            brush: Cell { code: 1, cs: 0, col: None, sel: false }, col: 1, zoom: 48.0, auto_zoom: true, size_in, banks: vec![], fill: None, fill_key: String::new(), used: HashSet::new(), dups: vec![],
            ask: None, clip: None, new_prop: String::new(), profiles, cs_profile: Default::default(), renumber: None, renumber_to: 0, filter: String::new() };
        e.cs_profile = e.c.charsets.iter().filter_map(|p| Some((p.clone(), serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(e.c.root.join(p)).ok()?).ok()?["profile"].as_str()?.to_string()))).collect();
        e.scan();
        e.after_change();
        Ok(e)
    }

    fn scan(&mut self) { if let Some(ts) = &self.ts { self.banks = scan_banks(&self.c.root, &self.c.rel, ts.tiles.len()); } }
    fn usage(&self, i: usize) -> usize { self.banks.iter().map(|b| b.counts.get(i).copied().unwrap_or(0)).sum() }
    fn target_is_atari(&self) -> bool { self.c.target == "atari8" }

    /// Nach `undo`/`redo` (Medium hat sich geändert) den Satz neu lesen
    fn sync(&mut self) {
        let now = self.c.media.to_string();
        if now == self.synced { return; }
        if !self.legacy { if let Ok(t) = tiles::parse_tileset(&self.c.media) { self.ts = Some(t); } }
        self.synced = now; self.after_change();
    }

    /// Abgeleitete Daten neu berechnen: Auswahl, Zeichensatz-Texturen, Duplikate, Füllstand
    fn after_change(&mut self) {
        let Some(ts) = self.ts.clone() else { return };
        self.sel = self.sel.min(ts.tiles.len().saturating_sub(1)); self.cur = self.cur.min(ts.tw * ts.th - 1);
        self.tab = self.tab.min(ts.charsets.len().saturating_sub(1)); self.size_in = (ts.tw, ts.th);
        if self.glyph_paths != ts.charsets { self.glyphs = ts.charsets.iter().map(|p| load_charset(&self.c.root, p, &self.c.glyph_ctx)).collect(); self.glyph_paths = ts.charsets.clone(); }
        self.used = ts.tiles.iter().flat_map(|t| t.cells.iter().map(|c| (c.cs, c.code))).collect();
        self.dups = (0..ts.tiles.len()).map(|i| (0..i).find(|&j| ts.tiles[j].cells == ts.tiles[i].cells && (ts.color_mode == ColorMode::None || ts.tiles[j].col == ts.tiles[i].col))).collect();
        let mut u: Vec<_> = self.used.iter().copied().collect(); u.sort();
        let key = format!("{:?}|{:?}|{:?}", ts.charsets, u, ts.pins);
        if key != self.fill_key { self.fill_key = key; self.fill = self.compute_fill(&ts); }
    }

    fn compute_fill(&self, ts: &TileSet) -> Option<Fill> {
        if ts.charsets.is_empty() { return None; }
        let r = tiles::load_charsets(&self.c.root, &ts.charsets).and_then(|(sets, cpid)| {
            let cp = vc_media::profile(&self.profiles, &cpid)?;
            let (mut seen, mut cells) = (HashSet::new(), vec![]);
            for c in ts.tiles.iter().flat_map(|t| t.cells.iter()) { if seen.insert((c.cs, c.code)) { cells.push((c.cs, c.code)); } }
            tiles::merge_glyphs(&cells, &sets, &ts.pins, cp)
        });
        Some(match r { Ok(m) => Fill { used: m.used, max: m.max, doubles: m.doubles, err: None }, Err(e) => Fill { err: Some(e), ..Default::default() } })
    }

    /// Änderung sofort anwenden (die Oberfläche sammelt sie und ruft das erst am Ende des Bildes auf)
    fn edit(&mut self, undo: bool, f: impl FnOnce(&mut TileSet)) {
        if undo { self.c.push_undo(); }
        if let Some(ts) = self.ts.as_mut() { f(ts); }
        if let Some(ts) = &self.ts { self.c.media = tiles::tileset_json(ts, &self.c.media); }
        self.synced = self.c.media.to_string();
        self.after_change();
    }

    // ------------------------------------------------------------------------ Zeichnen

    /// Zeichen aus Zeichensatz `cs` zeichnen; `fgi` = Farbindex der Zelle (bei C64-Mehrfarbzeichen mit Bit 3 = mehrfarbig)
    fn glyph_at(&self, p: &egui::Painter, pal: &Palettes, cs: usize, code: u8, r: Rect, fgi: usize, hi: bool, bg: Option<Color32>) {
        if let Some(b) = bg { p.rect_filled(r, 0.0, b); }
        let drawn = self.glyphs.get(cs).and_then(|g| g.as_ref()).map_or(false, |g| g.paint(p, pal, &self.c.target, code, r, fgi, McCols::from_colors(&self.c.media["colors"]), hi, 1.0));
        if !drawn { p.text(r.center(), egui::Align2::CENTER_CENTER, format!("{code}"), egui::FontId::monospace((r.height() * 0.4).clamp(6.0, 14.0)), pal.color(&self.c.target, fgi).gamma_multiply(0.7)); }
    }
    fn cell_color(&self, ts: &TileSet, t: &Tile, c: &Cell) -> usize {
        match ts.color_mode { ColorMode::Tile => t.col as usize, ColorMode::Cell => c.col.unwrap_or(t.col) as usize, ColorMode::None => self.c.cnum("fg", 14) }
    }
    fn bg(&self, pal: &Palettes) -> Color32 { pal.color(&self.c.target, self.c.cnum("bg", 6)) }
    fn draw_tile(&self, p: &egui::Painter, pal: &Palettes, ts: &TileSet, t: &Tile, r: Rect) {
        let (cw, ch) = (r.width() / ts.tw as f32, r.height() / ts.th as f32);
        for (k, c) in t.cells.iter().enumerate() {
            let cr = Rect::from_min_size(pos2(r.left() + (k % ts.tw) as f32 * cw, r.top() + (k / ts.tw) as f32 * ch), vec2(cw, ch));
            self.glyph_at(p, pal, c.cs, c.code, cr, self.cell_color(ts, t, c), c.sel, None);
        }
    }
    /// Zelle, die ein Pinselstrich setzen würde
    fn painted(&self, ts: &TileSet) -> Cell { Cell { col: (ts.color_mode == ColorMode::Cell).then_some(self.col as u8), ..self.brush.clone() } }

    // ------------------------------------------------------------------------ Bereiche der Oberfläche

    fn request_resize(&mut self, ts: &TileSet, w: usize, h: usize, edits: &mut Vec<Edit>) {
        if (w, h) == (ts.tw, ts.th) || !(1..=MAX_SIDE).contains(&w) || !(1..=MAX_SIDE).contains(&h) { return; }
        if loses_cells(ts, w, h) { self.ask = Some(Ask::Resize(w, h)); } else { edits.push((true, Box::new(move |t| resized(t, w, h)))); }
    }

    fn tools_ui(&mut self, ui: &mut egui::Ui, ts: &TileSet, edits: &mut Vec<Edit>) {
        ui.label("Größe");
        for (w, h) in [(2, 2), (3, 3), (4, 4), (3, 2)] { if ui.selectable_label((ts.tw, ts.th) == (w, h), format!("{w}×{h}")).on_hover_text("Größe für den ganzen Satz").clicked() { self.request_resize(ts, w, h, edits); } }
        ui.add(egui::DragValue::new(&mut self.size_in.0).range(1..=MAX_SIDE)); ui.label("×"); ui.add(egui::DragValue::new(&mut self.size_in.1).range(1..=MAX_SIDE));
        if ui.add_enabled(self.size_in != (ts.tw, ts.th), egui::Button::new("Anwenden")).on_hover_text("Breite × Höhe aller Kacheln ändern (Zellen beschneiden oder auffüllen)").clicked() { let (w, h) = self.size_in; self.request_resize(ts, w, h, edits); }
        ui.separator();
        if ts.tiles.is_empty() { return; }
        let (s, w, h) = (self.sel, ts.tw, ts.th);
        let mut op = |ui: &mut egui::Ui, label: &str, tip: &str, enabled: bool, f: Box<dyn FnOnce(&mut Vec<Cell>)>| {
            if ui.add_enabled(enabled, egui::Button::new(label)).on_hover_text(tip).clicked() { edits.push((true, Box::new(move |t: &mut TileSet| { if let Some(tile) = t.tiles.get_mut(s) { f(&mut tile.cells); } }))); }
        };
        op(ui, "Spiegeln H", "Zellreihenfolge waagerecht spiegeln (die Zeichenbilder bleiben)", w > 1, Box::new(move |c| flip_h(c, w, h)));
        op(ui, "Spiegeln V", "Zellreihenfolge senkrecht spiegeln", h > 1, Box::new(move |c| flip_v(c, w, h)));
        op(ui, "Drehen", "Um 90° drehen (nur quadratische Kacheln)", w == h && w > 1, Box::new(move |c| rotate(c, w)));
        ui.label("Rollen");
        op(ui, "←", "Zellen nach links verschieben", w > 1, Box::new(move |c| shift(c, w, h, -1, 0)));
        op(ui, "→", "Zellen nach rechts verschieben", w > 1, Box::new(move |c| shift(c, w, h, 1, 0)));
        op(ui, "↑", "Zellen nach oben verschieben", h > 1, Box::new(move |c| shift(c, w, h, 0, -1)));
        op(ui, "↓", "Zellen nach unten verschieben", h > 1, Box::new(move |c| shift(c, w, h, 0, 1)));
        op(ui, "Leeren", "Alle Zellen auf Zeichen 0 setzen", true, Box::new(|c| for x in c.iter_mut() { *x = blank(); }));
        if ui.button("Kopieren").on_hover_text("Zellen der Kachel merken").clicked() { self.clip = Some(ts.tiles[s].cells.clone()); }
        let clip = self.clip.clone().filter(|c| c.len() == w * h);
        if ui.add_enabled(clip.is_some(), egui::Button::new("Einfügen")).on_hover_text("Gemerkte Zellen in diese Kachel einfügen").clicked() { if let Some(cl) = clip { edits.push((true, Box::new(move |t| if let Some(tile) = t.tiles.get_mut(s) { tile.cells = cl; }))); } }
        ui.separator();
        ui.label("Zoom"); ui.spacing_mut().slider_width = 90.0;
        if ui.add_enabled(!self.auto_zoom, egui::Slider::new(&mut self.zoom, 24.0..=160.0).show_value(false)).changed() { self.auto_zoom = false; }
        if ui.checkbox(&mut self.auto_zoom, "Anpassen").on_hover_text("Kachel automatisch auf die Fläche der Mitte einpassen").changed() && !self.auto_zoom { self.zoom = self.zoom.round(); }
    }

    fn left_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet, edits: &mut Vec<Edit>) {
        studio::caption(ui, "Kacheln");
        ui.horizontal(|ui| {
            if ui.add_enabled(ts.tiles.len() < MAX_TILES, egui::Button::new("Neu")).on_hover_text("Neue leere Kachel am Ende").clicked() {
                let n = ts.tiles.len(); self.sel = n;
                edits.push((true, Box::new(move |t| { let cells = vec![blank(); t.tw * t.th]; t.tiles.push(Tile { name: format!("neu{n}"), cells, col: 1, props: 0 }); })));
            }
            if ui.add_enabled(!ts.tiles.is_empty() && ts.tiles.len() < MAX_TILES, egui::Button::new("Dupl.")).on_hover_text("Gewählte Kachel kopieren").clicked() {
                let (s, n) = (self.sel, ts.tiles.len()); self.sel = n;
                edits.push((true, Box::new(move |t| { if let Some(x) = t.tiles.get(s).cloned() { t.tiles.push(x); } })));
            }
            if ui.add_enabled(ts.tiles.len() > 1, egui::Button::new("Löschen")).on_hover_text("Gewählte Kachel entfernen (bei Verwendung mit Rückfrage)").clicked() {
                let s = self.sel;
                if self.banks.is_empty() || (self.usage(s) == 0 && s + 1 == ts.tiles.len()) { edits.push((true, Box::new(move |t| { if s < t.tiles.len() && t.tiles.len() > 1 { t.tiles.remove(s); } }))); } else { self.ask = Some(Ask::Delete(s)); }
            }
        });
        let have_banks = !self.banks.is_empty();
        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Suchen (Name oder Nummer)").desired_width(f32::INFINITY));
        let filter = self.filter.trim().to_lowercase();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { for (i, t) in ts.tiles.iter().enumerate() {
            if !filter.is_empty() && !t.name.to_lowercase().contains(&filter) && i.to_string() != filter { continue; }
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
            let th = Rect::from_min_size(r.min + vec2(3.0, 3.0), vec2(38.0, 38.0));
            ui.painter().rect_filled(th, 0.0, self.bg(pal)); self.draw_tile(&ui.painter_at(th), pal, ts, t, th);
            ui.painter().rect_stroke(th, 0.0, Stroke::new(1.0, Color32::from_gray(128).gamma_multiply(0.6)), egui::StrokeKind::Outside);
            let n = self.usage(i);
            let dim = have_banks && n == 0;
            let name = if t.name.is_empty() { format!("{i}") } else { format!("{i} {}", t.name) };
            let col = if dim { crate::theme::dim() } else { crate::theme::label_gray() };
            ui.painter().text(r.min + vec2(48.0, 14.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(13.0), col);
            let sub = match (have_banks, self.dups[i]) { (_, Some(j)) => format!("gleich wie {j}"), (true, _) => if n == 0 { "unbenutzt".into() } else { format!("{n}× benutzt") }, _ => String::new() };
            ui.painter().text(r.min + vec2(48.0, 31.0), egui::Align2::LEFT_CENTER, sub, egui::FontId::proportional(11.0), if self.dups[i].is_some() { crate::theme::warn() } else { crate::theme::dim() });
            if i == self.sel { ui.painter().rect_stroke(r, 3.0, Stroke::new(2.0, crate::theme::accent()), egui::StrokeKind::Inside); }
            let resp = resp.on_hover_text(format!("Kachel {i}{}{}", if t.name.is_empty() { String::new() } else { format!(" „{}“", t.name) }, if have_banks { format!(" · {n}× benutzt") } else { String::new() }));
            if resp.clicked() { self.sel = i; self.cur = self.cur.min(ts.tw * ts.th - 1); }
        } });
    }

    fn picker(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet, edits: &mut Vec<Edit>) {
        if ts.charsets.is_empty() { ui.label(RichText::new("Dieser Satz hat noch keinen Zeichensatz: unten hinzufügen.").weak()); return; }
        let fg = ts.tiles.get(self.sel).map_or(1, |t| t.col) as usize;
        let n = if self.target_is_atari() { 128 } else { 256 };
        let s = ((ui.available_width() - 17.0) / 16.0).floor().clamp(11.0, 26.0);
        ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
        let mut picked = None;
        for row in 0..n / 16 { ui.horizontal(|ui| { for i in row * 16..(row + 1) * 16 {
            let (r, resp) = ui.allocate_exact_size(vec2(s, s), Sense::click());
            let fgc = if ts.color_mode == ColorMode::None { self.c.cnum("fg", 14) } else if ts.color_mode == ColorMode::Cell { self.col } else { fg };
            let fgc = if matches!(self.c.target.as_str(), "c64" | "mega65" | "vic20") && self.glyphs.get(self.tab).and_then(|g| g.as_ref()).map_or(false, |g| g.mc) { fgc | 8 } else { fgc };
            self.glyph_at(&ui.painter_at(r), pal, self.tab, i as u8, r.shrink(1.0), fgc, false, Some(self.bg(pal)));
            ui.painter().rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_gray(128).gamma_multiply(0.35)), egui::StrokeKind::Inside);
            if self.used.contains(&(self.tab, i as u8)) { ui.painter().circle_filled(r.right_top() + vec2(-3.0, 3.0), 1.5, crate::theme::accent()); }
            if self.brush.cs == self.tab && self.brush.code == i as u8 { ui.painter().rect_stroke(r, 1.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside); }
            if resp.on_hover_text(format!("Zeichen {i} ($ {i:02X})")).clicked() { picked = Some(i as u8); }
        } }); }
        if let Some(code) = picked { self.pick(ts, code, edits); }
    }

    /// Zeichen aus dem Wähler oder der Eingabe: Pinsel setzen und die gewählte Zelle belegen
    fn pick(&mut self, ts: &TileSet, code: u8, edits: &mut Vec<Edit>) {
        self.brush.code = code; self.brush.cs = self.tab;
        let (s, k, cell) = (self.sel, self.cur, self.painted(ts));
        if ts.tiles.get(s).map_or(false, |t| t.cells.get(k) != Some(&cell)) { edits.push((true, Box::new(move |t| { if let Some(c) = t.tiles.get_mut(s).and_then(|x| x.cells.get_mut(k)) { *c = cell; } }))); }
    }

    fn right_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet, edits: &mut Vec<Edit>) {
        self.c.video_switch(ui);
        let s = self.sel;
        if let Some(t) = ts.tiles.get(s) {
            ui.label(RichText::new(format!("Kachel {s}")).strong());
            let mut name = t.name.clone();
            let r = ui.text_edit_singleline(&mut name);
            if r.changed() { let undo = r.gained_focus() || !ui.memory(|m| m.has_focus(r.id)); edits.push((undo, Box::new(move |ts| if let Some(t) = ts.tiles.get_mut(s) { t.name = name; }))); }
            if r.gained_focus() { self.c.push_undo(); }
            if let Some(j) = self.dups.get(s).copied().flatten() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("Gleich wie Kachel {j}.")).color(crate::theme::warn()));
                    if ui.button(format!("Mit {j} zusammenführen")).on_hover_text("Diese Kachel entfernen; Verweise in den Screen-Bänken zeigen danach auf die gleiche Kachel").clicked() {
                        if self.banks.is_empty() { edits.push((true, Box::new(move |t| { if s < t.tiles.len() && t.tiles.len() > 1 { t.tiles.remove(s); } }))); }
                        else { edits.push((true, Box::new(move |t| { if s < t.tiles.len() && t.tiles.len() > 1 { t.tiles.remove(s); } }))); self.renumber = Some(s); self.renumber_to = j; }
                    }
                });
            }
            // Farbe
            studio::caption(ui, "Farbe");
            let mut mode = ts.color_mode.as_str().to_string();
            if crate::ui::widgets::segmented(ui, &mut mode, &[("tile", "je Kachel"), ("cell", "je Zelle"), ("none", "keine")]) {
                let m = match mode.as_str() { "cell" => ColorMode::Cell, "none" => ColorMode::None, _ => ColorMode::Tile };
                edits.push((true, Box::new(move |t| t.color_mode = m)));
            }
            match ts.color_mode {
                ColorMode::Tile => { ui.label("Kachelfarbe"); if let Some(i) = self.c.swatches(ui, pal, t.col as usize) { edits.push((true, Box::new(move |ts| if let Some(t) = ts.tiles.get_mut(s) { t.col = i as u8; }))); } }
                ColorMode::Cell => { ui.label("Pinselfarbe (für Zellen); Kachelfarbe gilt für Zellen ohne eigene Farbe"); if let Some(i) = self.c.swatches(ui, pal, self.col) { self.col = i; }
                    ui.label("Kachelfarbe"); if let Some(i) = self.c.swatches(ui, pal, t.col as usize) { edits.push((true, Box::new(move |ts| if let Some(t) = ts.tiles.get_mut(s) { t.col = i as u8; }))); } }
                ColorMode::None => { ui.label(RichText::new("Keine Farbe im Satz: es gilt die Vordergrundfarbe des Systems.").weak().small()); }
            }
            if self.target_is_atari() {
                let mut sel = self.brush.sel;
                if ui.checkbox(&mut sel, "Farbwahl (Zeichenbit 7)").on_hover_text("Atari: wählt die zweite Farbgruppe des Zeichens (Bit 7)").changed() { self.brush.sel = sel; let (k, cell) = (self.cur, self.painted(ts)); if t.cells.get(k).map_or(false, |c| c.code == cell.code) { edits.push((true, Box::new(move |ts| if let Some(c) = ts.tiles.get_mut(s).and_then(|x| x.cells.get_mut(k)) { c.sel = sel; }))); } }
            }
            // Eigenschaften
            studio::caption(ui, "Eigenschaften");
            for (n, bit) in &ts.prop_defs {
                let mut on = t.props >> bit & 1 == 1; let bit = *bit;
                if ui.checkbox(&mut on, n).changed() { edits.push((true, Box::new(move |ts| if let Some(t) = ts.tiles.get_mut(s) { if on { t.props |= 1 << bit } else { t.props &= !(1 << bit) } }))); }
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_prop).hint_text("neue Eigenschaft").desired_width(110.0));
                let name = self.new_prop.trim().to_string();
                let free = (0u8..8).find(|b| !ts.prop_defs.iter().any(|(_, x)| x == b));
                if ui.add_enabled(!name.is_empty() && free.is_some() && !ts.prop_defs.iter().any(|(n, _)| *n == name), egui::Button::new("+")).on_hover_text("Eigenschaft für den ganzen Satz anlegen (höchstens 8)").clicked() {
                    if let Some(b) = free { edits.push((true, Box::new(move |ts| ts.prop_defs.push((name, b))))); self.new_prop.clear(); }
                }
            });
        }
        // Zeichensatz
        studio::caption(ui, "Zeichensatz");
        ui.horizontal_wrapped(|ui| {
            for (i, p) in ts.charsets.iter().enumerate() {
                let stem = vc_media::naming::stem(p);
                if ui.selectable_label(self.tab == i, format!("{} {}", (b'A' + (i % 26) as u8) as char, stem)).on_hover_text(p).clicked() { self.tab = i; self.brush.cs = i; }
            }
        });
        ui.horizontal(|ui| {
            let first = ts.charsets.first().and_then(|f| self.cs_profile.get(f));
            let avail: Vec<String> = self.c.charsets.iter().filter(|c| !ts.charsets.contains(c) && first.map_or(true, |f| self.cs_profile.get(*c) == Some(f))).cloned().collect();
            egui::ComboBox::from_id_salt(("addcs", &self.c.rel)).width(150.0).selected_text("+ Zeichensatz").show_ui(ui, |ui| {
                if avail.is_empty() { ui.label(RichText::new("Keine weiteren Zeichensätze mit gleichem Profil").weak()); }
                for c in avail { if ui.selectable_label(false, &c).clicked() { self.tab = ts.charsets.len(); let p = c.clone(); edits.push((true, Box::new(move |t| t.charsets.push(p)))); } }
            });
            let idx = self.tab;
            let in_use = ts.tiles.iter().any(|t| t.cells.iter().any(|c| c.cs == idx));
            if ui.add_enabled(idx < ts.charsets.len() && !in_use, egui::Button::new("Entfernen")).on_hover_text(if in_use { "Der Zeichensatz wird noch von Zellen benutzt" } else { "Reiter entfernen" }).clicked() {
                edits.push((true, Box::new(move |t| { if idx < t.charsets.len() { t.charsets.remove(idx); for tile in &mut t.tiles { for c in &mut tile.cells { if c.cs > idx { c.cs -= 1; } } } } })));
            }
        });
        let mut code = self.brush.code;
        ui.horizontal(|ui| {
            ui.label("Zeichen"); if ui.add(egui::DragValue::new(&mut code).range(0..=255)).changed() { self.pick(ts, code, edits); }
            ui.label(RichText::new(format!("$ {:02X}", self.brush.code)).weak());
        });
        self.picker(ui, pal, ts, edits);
        // Füllstand
        studio::caption(ui, "Zielzeichensatz");
        match &self.fill {
            None => { ui.label(RichText::new("Ohne Zeichensatz gibt es keinen Füllstand.").weak().small()); }
            Some(f) if f.err.is_some() => { ui.label(RichText::new(f.err.clone().unwrap_or_default()).color(crate::theme::bad()).small()); }
            Some(f) => {
                let frac = f.used as f32 / f.max.max(1) as f32;
                ui.add(egui::ProgressBar::new(frac).text(format!("{} von {} Glyphen", f.used, f.max)).fill(if frac > 0.9 { crate::theme::warn() } else { crate::theme::accent() }));
                if f.doubles > 0 { ui.label(RichText::new(format!("{} gleiche Glyphen werden beim Export zusammengelegt", f.doubles)).weak().small()); }
            }
        }
        studio::caption(ui, "Hintergrund");
        if let Some(i) = self.c.swatches(ui, pal, self.c.cnum("bg", 6)) { self.c.push_undo(); self.c.media["colors"]["bg"] = json!(i); self.synced = self.c.media.to_string(); }
    }

    fn center_ui(&mut self, ui: &mut egui::Ui, pal: &Palettes, ts: &TileSet, edits: &mut Vec<Edit>) {
        if let Some(a) = self.ask.clone() {
            let msg = match &a {
                Ask::Delete(i) => { let screens: usize = self.banks.iter().filter(|b| b.counts.get(*i).copied().unwrap_or(0) > 0).map(|b| b.screens).sum();
                    format!("Kachel {i} wird {}× in {screens} Screen(s) benutzt. Beim Löschen werden die Screen-Bänke angepasst (Verweise auf sie werden zu Kachel 0, höhere Nummern rücken nach), der Satz wird gespeichert und Rückgängig geleert.", self.usage(*i)) }
                Ask::Resize(w, h) => format!("Größe {w}×{h}: belegte Zellen außerhalb gehen verloren."),
            };
            egui::Frame::group(ui.style()).show(ui, |ui| { ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(msg).color(crate::theme::warn()));
                if ui.button(if matches!(a, Ask::Delete(_)) { "Löschen" } else { "Beschneiden" }).clicked() {
                    match a { Ask::Delete(i) => { edits.push((true, Box::new(move |t| { if i < t.tiles.len() && t.tiles.len() > 1 { t.tiles.remove(i); } }))); self.renumber = Some(i); } Ask::Resize(w, h) => edits.push((true, Box::new(move |t| resized(t, w, h)))) }
                    self.ask = None;
                }
                if ui.button("Abbrechen").clicked() { self.ask = None; }
            }); });
        }
        let Some(t) = ts.tiles.get(self.sel) else { ui.label("Der Satz hat keine Kachel: „Neu“ legt eine an."); return; };
        let avail = ui.available_size();
        if self.auto_zoom { self.zoom = ((avail.x - 16.0) / ts.tw as f32).min((avail.y - 130.0) / ts.th as f32).floor().clamp(24.0, 160.0); }
        let cz = self.zoom;
        let side = (avail.x - ts.tw as f32 * cz).max(0.0) / 2.0;
        let (outer, resp) = ui.allocate_exact_size(vec2((ts.tw as f32 * cz).max(avail.x - 16.0), ts.th as f32 * cz), Sense::click_and_drag());
        let rect = Rect::from_min_size(outer.min + vec2(side, 0.0), vec2(ts.tw as f32 * cz, ts.th as f32 * cz));
        let p = ui.painter_at(rect.expand(3.0));
        p.rect_filled(rect, 0.0, self.bg(pal));
        for (k, c) in t.cells.iter().enumerate() {
            let cr = Rect::from_min_size(pos2(rect.left() + (k % ts.tw) as f32 * cz, rect.top() + (k / ts.tw) as f32 * cz), vec2(cz, cz));
            self.glyph_at(&p, pal, c.cs, c.code, cr, self.cell_color(ts, t, c), c.sel, None);
            if c.cs > 0 { p.text(cr.left_top() + vec2(3.0, 2.0), egui::Align2::LEFT_TOP, ((b'A' + (c.cs % 26) as u8) as char).to_string(), egui::FontId::proportional(10.0), crate::theme::accent()); }
            if c.sel { p.text(cr.right_bottom() - vec2(3.0, 2.0), egui::Align2::RIGHT_BOTTOM, "S", egui::FontId::proportional(10.0), crate::theme::warn()); }
        }
        let grid = Stroke::new(1.0, Color32::from_gray(128).gamma_multiply(0.5));
        for x in 0..=ts.tw { let px = rect.left() + x as f32 * cz; p.line_segment([pos2(px, rect.top()), pos2(px, rect.bottom())], grid); }
        for y in 0..=ts.th { let py = rect.top() + y as f32 * cz; p.line_segment([pos2(rect.left(), py), pos2(rect.right(), py)], grid); }
        let (tw, th) = (ts.tw, ts.th);
        let hit = |pt: egui::Pos2| -> Option<usize> { if !rect.contains(pt) { return None; } let (x, y) = (((pt.x - rect.left()) / cz) as usize, ((pt.y - rect.top()) / cz) as usize); (x < tw && y < th).then_some(y * tw + x) };
        let pos = resp.interact_pointer_pos().and_then(hit);
        if let Some(k) = pos {
            if resp.secondary_clicked() {
                // Pipette: Zelle als Pinsel übernehmen
                self.cur = k; self.brush = Cell { col: None, ..t.cells[k].clone() }; self.tab = self.brush.cs.min(ts.charsets.len().saturating_sub(1));
                if let Some(c) = t.cells[k].col { self.col = c as usize; }
            } else {
                let first = resp.clicked() || resp.drag_started_by(egui::PointerButton::Primary);
                if first || resp.dragged_by(egui::PointerButton::Primary) {
                    self.cur = k; resp.request_focus();
                    let (s, cell) = (self.sel, self.painted(ts));
                    if t.cells[k] != cell { edits.push((first, Box::new(move |ts| if let Some(c) = ts.tiles.get_mut(s).and_then(|x| x.cells.get_mut(k)) { *c = cell; }))); }
                }
            }
        }
        if resp.has_focus() { ui.input(|i| {
            let (x, y) = (self.cur % tw, self.cur / tw);
            if i.key_pressed(egui::Key::ArrowLeft) && x > 0 { self.cur -= 1; }
            if i.key_pressed(egui::Key::ArrowRight) && x + 1 < tw { self.cur += 1; }
            if i.key_pressed(egui::Key::ArrowUp) && y > 0 { self.cur -= tw; }
            if i.key_pressed(egui::Key::ArrowDown) && y + 1 < th { self.cur += tw; }
        }); }
        let (cx, cy) = (self.cur % tw, self.cur / tw);
        ui.painter().rect_stroke(Rect::from_min_size(pos2(rect.left() + cx as f32 * cz, rect.top() + cy as f32 * cz), vec2(cz, cz)), 0.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Inside);
        let c = &t.cells[self.cur.min(t.cells.len() - 1)];
        ui.label(RichText::new(format!("Zelle {cx},{cy} · Zeichen {} ($ {:02X}) · Zeichensatz {}{}", c.code, c.code, (b'A' + (c.cs % 26) as u8) as char, if c.sel { " · Farbwahl" } else { "" })).weak().small());
        ui.label(RichText::new("Linke Taste setzt das gewählte Zeichen, rechte Taste nimmt das Zeichen der Zelle auf; Pfeiltasten wandern.").weak().small());
        if !self.banks.is_empty() {
            studio::caption(ui, "Verwendung");
            for b in &self.banks { let n = b.counts.get(self.sel).copied().unwrap_or(0); ui.label(RichText::new(format!("{}: {} Zelle(n) in {} Screen(s) der Bank", b.rel, n, b.screens)).small()); }
        }
    }

    fn legacy_ui(&mut self, ui: &mut egui::Ui, _env: &Env) {
        self.c.head(ui);
        let mut go: Option<bool> = None;
        studio::center(ui, |ui| {
            ui.add_space(8.0);
            ui.heading("Alte Kachelkarte");
            ui.label("Diese Datei ist noch eine Kachelkarte (Art „tilemap“): Kacheln und Karte stecken in einer Datei. Der Kachel-Editor arbeitet jetzt mit Kachelsätzen (.vctiles) und Screen-Bänken (.vcscreen).");
            ui.label("Das Umwandeln betrifft alle alten Kachelkarten des Projekts: Karten mit gleichem Kachelsatz ergeben einen Kachelsatz und eine Screen-Bank mit einem Screen je Karte.");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Umwandeln (alte Dateien entfernen)").clicked() { go = Some(true); }
                if ui.button("Umwandeln (alte Dateien behalten)").clicked() { go = Some(false); }
            });
            for l in &self.legacy_report { ui.label(RichText::new(l).monospace().small()); }
        });
        if let Some(remove) = go {
            self.legacy_report = match vc_media::project::migrate_tiles(&self.c.root, &self.data, remove) { Ok(mut r) => { r.push("Fertig. Die neuen Dateien bitte im Projektbaum öffnen.".into()); r } Err(e) => vec![format!("Fehler: {e}")] };
        }
    }
}

impl MediaEd for TileEd {
    fn dirty(&self) -> bool { self.c.dirty() }
    fn save(&mut self) -> Result<(), String> { let r = self.c.save(); self.scan(); r }
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        if self.legacy { return self.legacy_ui(ui, env); }
        let (pal, id) = (env.pal, self.c.rel.clone());
        self.sync();
        let Some(ts) = self.ts.clone() else { return };
        let mut edits: Vec<Edit> = vec![];
        self.c.head(ui);
        studio::tools(ui, &id, |ui| { self.c.file_tools(ui); self.tools_ui(ui, &ts, &mut edits); });
        let info = format!("{} Kacheln {}×{}{}", ts.tiles.len(), ts.tw, ts.th, self.fill.as_ref().filter(|f| f.err.is_none()).map_or(String::new(), |f| format!(" · Zeichensatz {} von {}", f.used, f.max)));
        self.c.foot(ui, env, &info);
        studio::left(ui, &id, 190.0, |ui| self.left_ui(ui, pal, &ts, &mut edits));
        studio::right_min(ui, &id, 330.0, 300.0, |ui| self.right_ui(ui, pal, &ts, &mut edits));
        studio::center(ui, |ui| self.center_ui(ui, pal, &ts, &mut edits));
        // Rückgängig/Wiederholen im selben Bild macht die gesammelten Änderungen ungültig
        if self.c.media.to_string() != self.synced { self.sync(); return; }
        for (undo, f) in edits { self.edit(undo, f); }
        if let Some(i) = self.renumber.take() {
            let r = self.c.save().and_then(|_| remap_banks(&self.c.root, &self.c.rel, i, std::mem::take(&mut self.renumber_to)));
            self.c.status = match r { Ok(n) => { self.c.undo.clear(); self.c.redo.clear(); format!("Kachel {i} gelöscht, {n} Screen-Bank(en) angepasst") } Err(e) => e };
            self.scan();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../") }
    fn cells(v: &[u8]) -> Vec<Cell> { v.iter().map(|&code| Cell { code, ..blank() }).collect() }
    fn codes(c: &[Cell]) -> Vec<u8> { c.iter().map(|x| x.code).collect() }

    #[test]
    fn zellen_spiegeln_drehen_rollen() {
        let mut c = cells(&[1, 2, 3, 4, 5, 6]); // 3×2
        flip_h(&mut c, 3, 2); assert_eq!(codes(&c), [3, 2, 1, 6, 5, 4]);
        flip_v(&mut c, 3, 2); assert_eq!(codes(&c), [6, 5, 4, 3, 2, 1]);
        let mut q = cells(&[1, 2, 3, 4]); rotate(&mut q, 2); assert_eq!(codes(&q), [3, 1, 4, 2]);
        for _ in 0..3 { rotate(&mut q, 2); } assert_eq!(codes(&q), [1, 2, 3, 4], "viermal drehen ergibt den Ausgangszustand");
        let mut r = cells(&[1, 2, 3, 4, 5, 6]); shift(&mut r, 3, 2, 1, 0); assert_eq!(codes(&r), [3, 1, 2, 6, 4, 5]);
        shift(&mut r, 3, 2, 0, 1); assert_eq!(codes(&r), [6, 4, 5, 3, 1, 2]);
    }

    #[test]
    fn groesse_beschneidet_und_fuellt() {
        let t = |name: &str, c: &[u8]| Tile { name: name.into(), cells: cells(c), col: 1, props: 0 };
        let mut ts = TileSet { name: "s".into(), tw: 2, th: 2, color_mode: ColorMode::Tile, charsets: vec![], tiles: vec![t("a", &[1, 2, 3, 4])], prop_defs: vec![], pins: vec![] };
        assert!(loses_cells(&ts, 1, 2) && !loses_cells(&ts, 3, 3));
        resized(&mut ts, 3, 2); assert_eq!(codes(&ts.tiles[0].cells), [1, 2, 0, 3, 4, 0]);
        assert!(!loses_cells(&ts, 2, 2), "nur leere Zellen fallen weg");
        resized(&mut ts, 1, 1); assert_eq!(codes(&ts.tiles[0].cells), [1]);
    }

    fn project() -> (PathBuf, PathBuf) {
        let data = root().join("data");
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!("vc-tiled-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        std::fs::copy(root().join("examples/c64/jumprun/graphics/chars.vcchar"), dir.join("graphics/chars.vcchar")).unwrap();
        std::fs::copy(root().join("examples/c64/gfx_demo/graphics/font.vcchar"), dir.join("graphics/font.vcchar")).unwrap();
        std::fs::copy(root().join("examples/c64/gfx_demo/graphics/font.vcchar"), dir.join("graphics/font2.vcchar")).unwrap();
        (dir, data)
    }

    #[test]
    fn kachelsatz_oeffnen_bearbeiten_speichern() {
        let (dir, data) = project(); let ctx = egui::Context::default();
        let rel = vc_media::project::create(&dir, &data, "hg", "c64.tileset", "").unwrap()["path"].as_str().unwrap().to_string();
        assert_eq!(rel, "graphics/hg.vctiles");
        let mut t = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
        assert!(!t.legacy && t.fill.is_none());
        assert_eq!(t.c.charsets, vec!["graphics/chars.vcchar".to_string(), "graphics/font.vcchar".to_string(), "graphics/font2.vcchar".to_string()]);
        // zwei Zeichensätze, Kachel anlegen, Zellen setzen
        t.edit(true, |ts| { ts.charsets.push("graphics/font.vcchar".into()); ts.charsets.push("graphics/font2.vcchar".into()); });
        t.edit(true, |ts| ts.tiles.push(Tile { name: "gras".into(), cells: vec![blank(); 4], col: 5, props: 0 }));
        t.edit(true, |ts| { let c = &mut ts.tiles[1].cells; c[0] = Cell { code: 1, ..blank() }; c[1] = Cell { code: 65, cs: 1, ..blank() }; c[3] = Cell { code: 2, ..blank() }; ts.tiles[1].props = 1; });
        assert!(t.dirty());
        let f = t.fill.clone().expect("Füllstand"); assert_eq!((f.max, f.err.clone()), (256, None)); assert!(f.used >= 3, "{f:?}");
        t.save().unwrap();
        let t2 = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
        let ts = t2.ts.as_ref().unwrap();
        assert_eq!((ts.tw, ts.th, ts.tiles.len(), ts.charsets.len()), (2, 2, 2, 2));
        assert_eq!((ts.tiles[1].cells[1].cs, ts.tiles[1].cells[1].code, ts.tiles[1].props, ts.tiles[1].col), (1, 65, 1, 5));
        // Größe ändern, Rückgängig stellt wieder her
        let mut t = t2; t.edit(true, |ts| resized(ts, 3, 3));
        assert_eq!(t.ts.as_ref().unwrap().tiles[1].cells.len(), 9); assert_eq!(t.c.media["size"]["w"], 3);
        t.c.undo(); t.sync(); assert_eq!((t.ts.as_ref().unwrap().tw, t.ts.as_ref().unwrap().tiles[1].cells.len()), (2, 4));
        // Zeichensatz entfernen, der benutzt wird: Zellen werden nicht angefasst, der Entfernen-Knopf ist dann gesperrt (nur Umnummerierung geprüft)
        t.edit(true, |ts| { ts.charsets.remove(0); for tile in &mut ts.tiles { for c in &mut tile.cells { if c.cs > 0 { c.cs -= 1; } } } });
        assert_eq!(t.ts.as_ref().unwrap().tiles[1].cells[1].cs, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verwendung_ueber_screen_banken() {
        let (dir, data) = project(); let ctx = egui::Context::default();
        let rel = vc_media::project::create(&dir, &data, "hg", "c64.tileset", "").unwrap()["path"].as_str().unwrap().to_string();
        let bank = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "b", "tilesets": ["graphics/hg.vctiles"],
            "screens": [{ "name": "a", "w": 4, "h": 2, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 1]] } }, { "name": "b", "w": 4, "h": 2, "tiles": { "set": 0, "origin": [0, 0], "grid": [[1, 1]] } }] });
        std::fs::write(dir.join("graphics/b.vcscreen"), vc_media::project::format_media(&bank)).unwrap();
        let other = json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "o", "tilesets": ["graphics/x.vctiles"], "screens": [{ "name": "a", "w": 2, "h": 2, "tiles": { "set": 0, "grid": [[0]] } }] });
        std::fs::write(dir.join("graphics/o.vcscreen"), vc_media::project::format_media(&other)).unwrap();
        let mut t = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
        t.edit(true, |ts| ts.tiles.push(Tile { name: "x".into(), cells: vec![blank(); 4], col: 1, props: 0 }));
        t.scan();
        assert_eq!(t.banks.len(), 1, "nur Banken, die den Satz nennen");
        assert_eq!((t.usage(0), t.usage(1), t.banks[0].screens), (1, 3, 2));
        assert_eq!(t.dups, vec![None, Some(0)], "zwei gleiche leere Kacheln");
        // Löschen von Kachel 0 schreibt die Bank um: 0 bleibt 0, 1 wird 0
        assert_eq!(remap_banks(&dir, "graphics/hg.vctiles", 0, 0).unwrap(), 1);
        let b: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/b.vcscreen")).unwrap()).unwrap();
        assert_eq!((b["screens"][0]["tiles"]["grid"].clone(), b["screens"][1]["tiles"]["grid"].clone()), (json!([[0, 0]]), json!([[0, 0]])));
        let o: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/o.vcscreen")).unwrap()).unwrap();
        assert_eq!(o["screens"][0]["tiles"]["grid"], json!([[0]]), "fremde Bank unberührt");
        // Zusammenführen von 1 in 0: Verweise auf 1 zeigen auf 0, 2 rückt auf 1
        std::fs::write(dir.join("graphics/b.vcscreen"), vc_media::project::format_media(&json!({ "format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "b", "tilesets": ["graphics/hg.vctiles"],
            "screens": [{ "name": "a", "w": 6, "h": 2, "tiles": { "set": 0, "origin": [0, 0], "grid": [[0, 1, 2]] } }] }))).unwrap();
        assert_eq!(remap_banks(&dir, "graphics/hg.vctiles", 1, 0).unwrap(), 1);
        let b: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("graphics/b.vcscreen")).unwrap()).unwrap();
        assert_eq!(b["screens"][0]["tiles"]["grid"], json!([[0, 0, 1]]));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn alte_kachelkarte_wird_erkannt_und_umgewandelt() {
        let (dir, data) = project(); let ctx = egui::Context::default();
        let rel = vc_media::project::create(&dir, &data, "level", "c64.tilemap.2x2", "").unwrap()["path"].as_str().unwrap().to_string();
        let mut t = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
        assert!(t.legacy && t.ts.is_none());
        let (pal, audio, reqs) = (Palettes::load(&data), crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        for _ in 0..2 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0))), ..Default::default() }, |ui| MediaEd::ui(&mut t, ui, &env)); o.textures_delta.clear(); }
        let r = vc_media::project::migrate_tiles(&dir, &data, true).unwrap();
        assert!(r.iter().any(|l| l.contains("level_tiles.vctiles")), "{r:?}");
        let neu = TileEd::open(&ctx, &dir, "graphics/level_tiles.vctiles", &data).unwrap();
        assert!(!neu.legacy && neu.ts.unwrap().tiles.len() == 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn kachel_editor_zeichnet_mit_exportdialog_und_in_kleinen_fenstern() {
        let (dir, data) = project(); let ctx = egui::Context::default();
        let pal = Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), crate::bitmap::Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        let rel = vc_media::project::create(&dir, &data, "hg", "c64.tileset", "").unwrap()["path"].as_str().unwrap().to_string();
        let mut t = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
        t.edit(true, |ts| { ts.charsets.push("graphics/font.vcchar".into()); ts.tiles[0].cells[0] = Cell { code: 3, ..blank() }; });
        t.save().unwrap();
        t.c.exp = Some(crate::exportdlg::ExportDlg::new(&rel, "tileset", "hg"));
        t.ask = Some(Ask::Resize(1, 1));
        let run = |t: &mut TileEd, w: f32, h: f32| { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| MediaEd::ui(t, ui, &env)); o.textures_delta.clear(); } };
        run(&mut t, 1000.0, 700.0);
        for (w, h) in [(330.0, 500.0), (640.0, 480.0)] { run(&mut t, w, h); assert!(t.c.exp.is_some(), "Export-Dialog offen bei {w}x{h}"); }
        // Export des Satzes liefert Tabellen und Zeichensatz
        let v = vc_media::project::export(&dir, &data, &rel, "asm", Some("hg"), None, 10, 10).unwrap();
        assert!(v["files"].as_array().map_or(false, |f| !f.is_empty()), "{v}");
        assert_eq!(crate::exportdlg::default_addr("tileset"), "$4000");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn atari_und_mega65_profile_und_mittlere_fenstergroesse() {
        let data = root().join("data"); let ctx = egui::Context::default(); let pal = Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), crate::bitmap::Requests::default());
        crate::fonts::install(&ctx); crate::ui::style::install(&ctx);
        for (sys, profile) in [("atari", "atari.tileset"), ("mega65", "mega65.tileset"), ("c64", "c64.tileset")] {
            let dir = std::env::temp_dir().join(format!("vc-tiled-sys-{sys}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
            std::fs::copy(root().join(format!("examples/{sys}/jumprun/graphics/chars.vcchar")), dir.join("graphics/chars.vcchar")).unwrap();
            let rel = vc_media::project::create(&dir, &data, "hg", profile, "").unwrap()["path"].as_str().unwrap().to_string();
            let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &dir, data: &data };
            let mut t = TileEd::open(&ctx, &dir, &rel, &data).unwrap();
            t.edit(true, |ts| ts.charsets.push("graphics/chars.vcchar".into()));
            // Zeichenwähler: Zeichen wählen setzt Pinsel und Zelle
            let ts = t.ts.clone().unwrap(); let mut edits = vec![]; t.cur = 1; t.pick(&ts, 5, &mut edits);
            for (u, f) in edits { t.edit(u, f); }
            assert_eq!(t.ts.as_ref().unwrap().tiles[0].cells[1].code, 5, "{sys}");
            if sys == "atari" { t.brush.sel = true; let ts = t.ts.clone().unwrap(); let mut e = vec![]; t.pick(&ts, 6, &mut e); for (u, f) in e { t.edit(u, f); }
                assert!(t.ts.as_ref().unwrap().tiles[0].cells[1].sel); }
            // Eigenschaft anlegen und setzen
            t.edit(true, |ts| { ts.prop_defs.push(("leiter".into(), 1)); ts.tiles[0].props |= 2; });
            assert_eq!(t.c.media["tiles"][0]["props"].as_array().map(|p| p.len()), Some(1 + t.ts.as_ref().unwrap().tiles[0].props.count_ones() as usize - 1));
            assert!(t.fill.as_ref().map_or(false, |f| f.err.is_none() && f.max == if sys == "atari" { 128 } else { 256 }), "{sys}: {:?}", t.fill);
            for (w, h) in [(1100.0, 760.0), (1500.0, 900.0), (700.0, 500.0)] { for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() }, |ui| MediaEd::ui(&mut t, ui, &env)); o.textures_delta.clear(); } }
            assert!(t.zoom >= 24.0 && t.zoom <= 160.0);
            t.save().unwrap();
            let v = vc_media::project::export(&dir, &data, &rel, "asm", Some("hg"), None, 10, 10).unwrap();
            assert!(v["files"].as_array().map_or(false, |f| !f.is_empty()), "{sys}: {v}");
            std::fs::remove_dir_all(&dir).ok();
        }
    }
}
