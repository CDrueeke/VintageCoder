// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachelsatz (`.vctiles`, Art `tileset`) ohne Oberfläche: Kacheln aus Zeichenzellen bearbeiten (Zellen setzen, spiegeln, drehen, rollen, kopieren),
//! Größe ändern, Kacheln anlegen/duplizieren/löschen/zusammenführen (mit Anpassung der Screen-Bänke des Projekts), Zeichensätze, Farbmodus,
//! Eigenschaften, Füllstand des Zielzeichensatzes. Das Dateimodell (`TileSet`) ist `vc_media::tiles`.
use crate::glyphs::{Bild, Glyphs, McCols};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use vc_media::tiles::{self, Cell, ColorMode, Tile, TileSet, MAX_SIDE, MAX_TILES};

/// Verwendung des Satzes in einer Screen-Bank des Projekts
#[derive(Clone, Debug, PartialEq)]
pub struct BankUse { pub rel: String, pub screens: usize, pub counts: Vec<usize> }

/// Füllstand des Zielzeichensatzes nach `merge_glyphs`
#[derive(Clone, Default, PartialEq, Debug)]
pub struct Fill { pub used: usize, pub max: usize, pub doubles: usize, pub err: Option<String> }

/// Rückfrage, die vor einer folgenreichen Änderung gestellt wird
#[derive(Clone, Debug, PartialEq)]
pub enum Frage { Loeschen(usize), Groesse(usize, usize) }

/// Was ein Klick in die große Kachel setzt (nur bei Farbmodus „je Zelle“): das Zeichen, nur die Farbe der Zelle oder beides
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PinselModus { Zeichen, Farbe, #[default] Beides }

pub struct TilesetDoc {
    pub rel: String, abs: PathBuf, root: PathBuf,
    pub media: Value, pub ts: TileSet, pub target: String,
    pub glyphs: Vec<Option<Glyphs>>, glyph_paths: Vec<String>,
    pub banken: Vec<BankUse>,
    pub fill: Option<Fill>, fill_key: String,
    /// benutzte (Zeichensatz, Code)-Paare
    pub benutzt: HashSet<(usize, u8)>,
    /// je Kachel: Nummer einer früheren gleichen Kachel
    pub doppelt: Vec<Option<usize>>,
    undo: Vec<String>, redo: Vec<String>, saved: String,
    pub status: String,
    /// gewählte Kachel, gewählte Zelle, Pinsel, Pinselfarbe (bei Farbe je Zelle), gewählter Zeichensatz-Reiter
    pub sel: usize, pub cur: usize, pub pinsel: Cell, pub pinsel_modus: PinselModus, pub farbe: usize, pub reiter: usize,
    pub zwischen: Option<Vec<Cell>>,
    pub frage: Option<Frage>,
    profiles: Vec<vc_media::Profile>,
    /// Zeichensätze des Projekts für dieses System, mit ihrem Profil (nur gleiche Profile lassen sich mischen)
    pub zeichensaetze: Vec<(String, String)>,
}

fn leer() -> Cell { Cell { code: 0, cs: 0, col: None, sel: false } }

// -- Änderungen an Zellen (frei, testbar) --------------------------------------------------------------------------------
pub fn spiegeln_h(c: &mut [Cell], w: usize, h: usize) { for y in 0..h { c[y * w..(y + 1) * w].reverse() } }
pub fn spiegeln_v(c: &mut [Cell], w: usize, h: usize) { for y in 0..h / 2 { for x in 0..w { c.swap(y * w + x, (h - 1 - y) * w + x) } } }
/// Um 90° im Uhrzeigersinn; nur bei quadratischer Kachel
pub fn drehen(c: &mut [Cell], n: usize) { let alt = c.to_vec(); for y in 0..n { for x in 0..n { c[y * n + x] = alt[(n - 1 - x) * n + y].clone() } } }
/// Zellen rollen (am Rand wieder hinein)
pub fn rollen(c: &mut [Cell], w: usize, h: usize, dx: i32, dy: i32) {
    let alt = c.to_vec();
    for y in 0..h { for x in 0..w { let (sx, sy) = ((x as i32 - dx).rem_euclid(w as i32) as usize, (y as i32 - dy).rem_euclid(h as i32) as usize); c[y * w + x] = alt[sy * w + sx].clone() } }
}
/// Größe ändern: beschneiden oder mit leeren Zellen auffüllen
pub fn neue_groesse(ts: &mut TileSet, w: usize, h: usize) {
    let (ow, oh) = (ts.tw, ts.th);
    for t in &mut ts.tiles { t.cells = (0..h * w).map(|i| { let (x, y) = (i % w, i / w); if x < ow && y < oh { t.cells[y * ow + x].clone() } else { leer() } }).collect() }
    ts.tw = w; ts.th = h;
}
pub fn verliert_zellen(ts: &TileSet, w: usize, h: usize) -> bool {
    ts.tiles.iter().any(|t| t.cells.iter().enumerate().any(|(i, c)| { let (x, y) = (i % ts.tw, i / ts.tw); (x >= w || y >= h) && *c != leer() }))
}

fn bank_dateien(root: &Path) -> Vec<PathBuf> {
    fn lauf(dir: &Path, tiefe: usize, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() { let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
            if n.starts_with('.') || n == "build" || n == "target" { continue }
            if p.is_dir() { if tiefe < 5 { lauf(&p, tiefe + 1, out) } } else if n.ends_with(".vcscreen") { out.push(p) } }
    }
    let mut v = vec![]; lauf(root, 0, &mut v); v.sort(); v
}

/// Alle Verweise auf den Kachelsatz `set_rel` in Screen-Bänken des Projekts
pub fn banken_suchen(root: &Path, set_rel: &str, tile_count: usize) -> Vec<BankUse> {
    let mut out = vec![];
    for f in bank_dateien(root) {
        let Ok(v) = std::fs::read_to_string(&f).map_err(|_| ()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|_| ())) else { continue };
        if !tiles::is_bank(&v) { continue }
        let Some(idx) = v["tilesets"].as_array().and_then(|a| a.iter().position(|x| x.as_str() == Some(set_rel))) else { continue };
        let (mut counts, mut screens) = (vec![0usize; tile_count], 0);
        for s in v["screens"].as_array().into_iter().flatten() {
            if s["tiles"]["set"].as_u64().unwrap_or(0) as usize != idx { continue }
            let mut any = false;
            for r in s["tiles"]["grid"].as_array().into_iter().flatten() { for t in r.as_array().into_iter().flatten() { if let Some(n) = t.as_u64() { any = true; if let Some(c) = counts.get_mut(n as usize) { *c += 1 } } } }
            if any { screens += 1 }
        }
        out.push(BankUse { rel: f.strip_prefix(root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default(), screens, counts });
    }
    out
}

/// Nach dem Löschen von Kachel `del`: Verweise auf `del` werden zu `nach` (beim Zusammenführen die gleiche Kachel, sonst 0), höhere Nummern rücken nach.
/// Liefert die Zahl geänderter Bänke.
pub fn banken_umnummerieren(root: &Path, set_rel: &str, del: usize, nach: usize) -> Result<usize, String> {
    let mut geaendert = 0;
    for f in bank_dateien(root) {
        let Ok(mut v) = std::fs::read_to_string(&f).map_err(|_| ()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|_| ())) else { continue };
        if !tiles::is_bank(&v) { continue }
        let Some(idx) = v["tilesets"].as_array().and_then(|a| a.iter().position(|x| x.as_str() == Some(set_rel))) else { continue };
        let mut any = false;
        for s in v["screens"].as_array_mut().into_iter().flatten() {
            if s["tiles"]["set"].as_u64().unwrap_or(0) as usize != idx { continue }
            for r in s["tiles"]["grid"].as_array_mut().into_iter().flatten() { for t in r.as_array_mut().into_iter().flatten() {
                if let Some(n) = t.as_u64() { let n = n as usize; let m = if n == del { nach } else if n > del { n - 1 } else { n }; if m != n { *t = json!(m); any = true } }
            } }
        }
        if any { vc_core::atomar::schreiben(&f, vc_media::project::format_media(&v)).map_err(|e| format!("{}: {e}", f.display()))?; geaendert += 1 }
    }
    Ok(geaendert)
}

impl TilesetDoc {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<TilesetDoc, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| vc_i18n::t!("tilesetdoc-rel-e", rel = &rel, e = e.to_string()))?).map_err(|e| vc_i18n::t!("tilesetdoc-rel-e", rel = &rel, e = e.to_string()))?;
        let ts = tiles::parse_tileset(&media).map_err(|e| vc_i18n::t!("tilesetdoc-rel-e", rel = &rel, e = &e))?;
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, media["profile"].as_str().unwrap_or(""))?; let target = p.target.clone();
        let zs = crate::screendoc::zeichensaetze_im_projekt(root, &target, &profiles);
        let zeichensaetze = zs.into_iter().map(|c| { let pr = std::fs::read_to_string(root.join(&c)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v["profile"].as_str().map(String::from)).unwrap_or_default(); (c, pr) }).collect();
        let mut d = TilesetDoc { rel: rel.into(), abs, root: root.into(), saved: media.to_string(), media, ts, target, glyphs: vec![], glyph_paths: vec![], banken: vec![], fill: None, fill_key: String::new(), benutzt: HashSet::new(), doppelt: vec![],
            undo: vec![], redo: vec![], status: String::new(), sel: 0, cur: 0, pinsel: Cell { code: 1, cs: 0, col: None, sel: false }, pinsel_modus: PinselModus::Beides, farbe: 1, reiter: 0, zwischen: None, frage: None, profiles, zeichensaetze };
        d.banken_lesen(); d.abgeleitet(); Ok(d)
    }

    pub fn banken_lesen(&mut self) { self.banken = banken_suchen(&self.root, &self.rel, self.ts.tiles.len()) }
    pub fn verwendung(&self, i: usize) -> usize { self.banken.iter().map(|b| b.counts.get(i).copied().unwrap_or(0)).sum() }
    pub fn atari(&self) -> bool { self.target == "atari8" }

    /// Abgeleitete Daten neu berechnen: Auswahl, Glyphen, Doppelte, Füllstand
    fn abgeleitet(&mut self) {
        let ts = &self.ts;
        self.sel = self.sel.min(ts.tiles.len().saturating_sub(1)); self.cur = self.cur.min((ts.tw * ts.th).saturating_sub(1)); self.reiter = self.reiter.min(ts.charsets.len().saturating_sub(1));
        if self.glyph_paths != ts.charsets { self.glyphs = ts.charsets.iter().map(|p| Glyphs::laden(&self.root, p)).collect(); self.glyph_paths = ts.charsets.clone() }
        self.benutzt = ts.tiles.iter().flat_map(|t| t.cells.iter().map(|c| (c.cs, c.code))).collect();
        self.doppelt = (0..ts.tiles.len()).map(|i| (0..i).find(|&j| ts.tiles[j].cells == ts.tiles[i].cells && (ts.color_mode == ColorMode::None || ts.tiles[j].col == ts.tiles[i].col))).collect();
        let mut u: Vec<_> = self.benutzt.iter().copied().collect(); u.sort();
        let key = format!("{:?}|{:?}|{:?}", ts.charsets, u, ts.pins);
        if key != self.fill_key { self.fill_key = key; self.fill = self.fuellstand() }
    }
    fn fuellstand(&self) -> Option<Fill> {
        let ts = &self.ts; if ts.charsets.is_empty() { return None }
        let r = tiles::load_charsets(&self.root, &ts.charsets).and_then(|(sets, cpid)| {
            let cp = vc_media::profile(&self.profiles, &cpid)?;
            let (mut gesehen, mut zellen) = (HashSet::new(), vec![]);
            for c in ts.tiles.iter().flat_map(|t| t.cells.iter()) { if gesehen.insert((c.cs, c.code)) { zellen.push((c.cs, c.code)) } }
            tiles::merge_glyphs(&zellen, &sets, &ts.pins, cp)
        });
        Some(match r { Ok(m) => Fill { used: m.used, max: m.max, doubles: m.doubles, err: None }, Err(e) => Fill { err: Some(e), ..Default::default() } })
    }

    // -- Zustand --------------------------------------------------------------------------------------------------------
    pub fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    fn undo_punkt(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear() }
    fn neu_lesen(&mut self) { if let Ok(t) = tiles::parse_tileset(&self.media) { self.ts = t } self.abgeleitet() }
    pub fn rueckgaengig(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.neu_lesen() } } }
    pub fn wiederholen(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.neu_lesen() } } }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    pub fn media_text(&self) -> String { vc_media::project::format_media(&self.media) }
    pub fn speichern(&mut self) -> Result<(), String> {
        vc_core::atomar::schreiben(&self.abs, self.media_text()).map_err(|e| format!("{}: {e}", self.rel))?; self.saved = self.media.to_string(); self.status = vc_i18n::t!("tilesetdoc-gespeichert"); self.banken_lesen(); Ok(())
    }
    /// Änderung am Satz; `undo` = Rückgängig-Punkt vorher setzen
    fn aendern(&mut self, undo: bool, f: impl FnOnce(&mut TileSet)) {
        if undo { self.undo_punkt() }
        f(&mut self.ts); self.media = tiles::tileset_json(&self.ts, &self.media); self.abgeleitet()
    }

    // -- Auswahl --------------------------------------------------------------------------------------------------------
    pub fn kachel_waehlen(&mut self, i: usize) { if i < self.ts.tiles.len() { self.sel = i; self.cur = self.cur.min(self.ts.tw * self.ts.th - 1) } }
    pub fn zelle_waehlen(&mut self, k: usize) { if k < self.ts.tw * self.ts.th { self.cur = k } }
    pub fn cursor_bewegen(&mut self, dx: i32, dy: i32) {
        let (w, h) = (self.ts.tw as i32, self.ts.th as i32); let (x, y) = ((self.cur % self.ts.tw) as i32, (self.cur / self.ts.tw) as i32);
        self.cur = ((y + dy).clamp(0, h - 1) * w + (x + dx).clamp(0, w - 1)) as usize;
    }
    /// Zelle, die ein Pinselstrich setzen würde
    pub fn gemalt(&self) -> Cell { Cell { col: (self.ts.color_mode == ColorMode::Cell).then_some(self.farbe as u8), ..self.pinsel.clone() } }

    // -- Zellen ---------------------------------------------------------------------------------------------------------
    /// Pinselstrich in Zelle `k` der gewählten Kachel; `erster` = Rückgängig-Punkt (erster Punkt eines Zuges)
    pub fn malen(&mut self, k: usize, erster: bool) {
        self.cur = k; let s = self.sel; let Some(alt) = self.ts.tiles.get(s).and_then(|t| t.cells.get(k)).cloned() else { return };
        let z = match self.wirksamer_modus() { PinselModus::Beides => self.gemalt(), PinselModus::Zeichen => Cell { col: alt.col, ..self.pinsel.clone() }, PinselModus::Farbe => Cell { col: Some(self.farbe as u8), ..alt.clone() } };
        if alt != z { self.aendern(erster, move |ts| { if let Some(c) = ts.tiles.get_mut(s).and_then(|x| x.cells.get_mut(k)) { *c = z } }) }
    }
    /// Den Modus gibt es nur bei Farbmodus „je Zelle“; sonst setzt ein Klick wie bisher das Zeichen
    pub fn wirksamer_modus(&self) -> PinselModus { if self.ts.color_mode == ColorMode::Cell { self.pinsel_modus } else { PinselModus::Beides } }
    pub fn modus_waehlen(&mut self, m: PinselModus) { self.pinsel_modus = m }
    /// Pipette: Zelle als Pinsel übernehmen
    pub fn aufnehmen(&mut self, k: usize) {
        let Some(c) = self.ts.tiles.get(self.sel).and_then(|t| t.cells.get(k)).cloned() else { return };
        self.cur = k; self.pinsel = Cell { col: None, ..c.clone() }; self.reiter = c.cs.min(self.ts.charsets.len().saturating_sub(1)); if let Some(f) = c.col { self.farbe = f as usize }
    }
    /// Zeichen aus der Tafel oder Eingabe: Pinsel setzen und die gewählte Zelle belegen
    pub fn zeichen_waehlen(&mut self, code: u8) { self.pinsel.code = code; self.pinsel.cs = self.reiter; let k = self.cur; if self.wirksamer_modus() != PinselModus::Farbe { self.malen(k, true) } }
    pub fn pinsel_farbe(&mut self, i: usize) { self.farbe = i }
    pub fn farbwahl_bit(&mut self, an: bool) { self.pinsel.sel = an; let k = self.cur; if self.ts.tiles.get(self.sel).and_then(|t| t.cells.get(k)).is_some() { self.malen(k, true) } }
    fn zellen_op(&mut self, f: impl FnOnce(&mut Vec<Cell>, usize, usize)) { let (s, w, h) = (self.sel, self.ts.tw, self.ts.th); self.aendern(true, move |ts| { if let Some(t) = ts.tiles.get_mut(s) { f(&mut t.cells, w, h) } }) }
    pub fn spiegeln(&mut self, horizontal: bool) { self.zellen_op(move |c, w, h| if horizontal { spiegeln_h(c, w, h) } else { spiegeln_v(c, w, h) }) }
    pub fn drehen(&mut self) { if self.ts.tw == self.ts.th && self.ts.tw > 1 { self.zellen_op(|c, w, _| drehen(c, w)) } }
    pub fn rollen(&mut self, dx: i32, dy: i32) { self.zellen_op(move |c, w, h| rollen(c, w, h, dx, dy)) }
    pub fn leeren(&mut self) { self.zellen_op(|c, _, _| for x in c.iter_mut() { *x = leer() }) }
    pub fn kopieren(&mut self) { self.zwischen = self.ts.tiles.get(self.sel).map(|t| t.cells.clone()) }
    pub fn einfuegen(&mut self) {
        let Some(z) = self.zwischen.clone().filter(|c| c.len() == self.ts.tw * self.ts.th) else { return };
        let s = self.sel; self.aendern(true, move |ts| { if let Some(t) = ts.tiles.get_mut(s) { t.cells = z } })
    }

    // -- Größe ----------------------------------------------------------------------------------------------------------
    pub fn groesse_anfragen(&mut self, w: usize, h: usize) {
        if (w, h) == (self.ts.tw, self.ts.th) || !(1..=MAX_SIDE).contains(&w) || !(1..=MAX_SIDE).contains(&h) { return }
        if verliert_zellen(&self.ts, w, h) { self.frage = Some(Frage::Groesse(w, h)) } else { self.aendern(true, move |ts| neue_groesse(ts, w, h)) }
    }

    // -- Kacheln --------------------------------------------------------------------------------------------------------
    pub fn kachel_neu(&mut self) {
        if self.ts.tiles.len() >= MAX_TILES { return }
        let n = self.ts.tiles.len(); self.sel = n;
        self.aendern(true, move |t| { let cells = vec![leer(); t.tw * t.th]; t.tiles.push(Tile { name: format!("neu{n}"), cells, col: 1, props: 0 }) })
    }
    pub fn kachel_duplizieren(&mut self) {
        if self.ts.tiles.is_empty() || self.ts.tiles.len() >= MAX_TILES { return }
        let (s, n) = (self.sel, self.ts.tiles.len()); self.sel = n;
        self.aendern(true, move |t| { if let Some(x) = t.tiles.get(s).cloned() { t.tiles.push(x) } })
    }
    /// Löschen: ohne Verwendung in Bänken sofort, sonst mit Rückfrage
    pub fn kachel_loeschen(&mut self) {
        let s = self.sel; if self.ts.tiles.len() < 2 { return }
        if self.banken.is_empty() || (self.verwendung(s) == 0 && s + 1 == self.ts.tiles.len()) { self.entfernen(s) } else { self.frage = Some(Frage::Loeschen(s)) }
    }
    fn entfernen(&mut self, i: usize) { self.aendern(true, move |t| { if i < t.tiles.len() && t.tiles.len() > 1 { t.tiles.remove(i); } }) }
    /// Rückfrage bestätigen; beim Löschen werden die Screen-Bänke angepasst (Verweise → Kachel 0, höhere Nummern rücken nach), der Satz wird gespeichert
    pub fn frage_bestaetigen(&mut self) -> Result<(), String> {
        match self.frage.take() {
            Some(Frage::Groesse(w, h)) => self.aendern(true, move |ts| neue_groesse(ts, w, h)),
            Some(Frage::Loeschen(i)) => { self.entfernen(i); self.speichern()?; let n = banken_umnummerieren(&self.root, &self.rel, i, 0)?; self.status = vc_i18n::t!("tilesetdoc-kachel-i-geloescht-n-bank-en", i = &i, n = &n); self.banken_lesen() }
            None => {}
        }
        Ok(())
    }
    pub fn frage_abbrechen(&mut self) { self.frage = None }
    /// Doppelte Kachel mit der früheren gleichen zusammenführen: entfernt sie, Verweise in den Bänken zeigen danach auf die gleiche Kachel
    pub fn zusammenfuehren(&mut self) -> Result<(), String> {
        let s = self.sel; let Some(j) = self.doppelt.get(s).copied().flatten() else { return Ok(()) };
        self.entfernen(s);
        if !self.banken.is_empty() { self.speichern()?; let n = banken_umnummerieren(&self.root, &self.rel, s, j)?; self.status = vc_i18n::t!("tilesetdoc-kachel-s-mit-j-zusammengefuehrt-n", s = &s, j = &j, n = &n); self.banken_lesen() }
        Ok(())
    }
    pub fn kachel_umbenennen(&mut self, name: &str, erster: bool) { let (s, n) = (self.sel, name.to_string()); self.aendern(erster, move |ts| { if let Some(t) = ts.tiles.get_mut(s) { t.name = n } }) }

    // -- Farbe und Eigenschaften -------------------------------------------------------------------------------------
    pub fn farbmodus(&mut self, m: ColorMode) { self.aendern(true, move |t| t.color_mode = m) }
    pub fn kachelfarbe(&mut self, i: usize) { let s = self.sel; self.aendern(true, move |ts| { if let Some(t) = ts.tiles.get_mut(s) { t.col = i as u8 } }) }
    pub fn hintergrund(&mut self, i: usize) { self.undo_punkt(); self.media["colors"]["bg"] = json!(i); self.abgeleitet() }
    pub fn eigenschaft_setzen(&mut self, bit: u8, an: bool) { let s = self.sel; self.aendern(true, move |ts| { if let Some(t) = ts.tiles.get_mut(s) { if an { t.props |= 1 << bit } else { t.props &= !(1 << bit) } } }) }
    /// Neue Eigenschaft für den ganzen Satz (höchstens 8); `false`, wenn kein Bit frei oder der Name schon da ist
    pub fn eigenschaft_neu(&mut self, name: &str) -> bool {
        let name = name.trim().to_string(); let frei = (0u8..8).find(|b| !self.ts.prop_defs.iter().any(|(_, x)| x == b));
        let Some(b) = frei else { return false }; if name.is_empty() || self.ts.prop_defs.iter().any(|(n, _)| *n == name) { return false }
        self.aendern(true, move |ts| ts.prop_defs.push((name, b))); true
    }

    // -- Zeichensätze ---------------------------------------------------------------------------------------------------
    /// Zeichensätze, die sich noch hinzufügen lassen (gleiches Profil wie der erste)
    pub fn zeichensatz_angebot(&self) -> Vec<String> {
        let erstes = self.ts.charsets.first().and_then(|f| self.zeichensaetze.iter().find(|(c, _)| c == f)).map(|(_, p)| p.clone());
        self.zeichensaetze.iter().filter(|(c, p)| !self.ts.charsets.contains(c) && erstes.as_ref().is_none_or(|e| e == p)).map(|(c, _)| c.clone()).collect()
    }
    pub fn zeichensatz_hinzu(&mut self, rel: &str) { self.reiter = self.ts.charsets.len(); let p = rel.to_string(); self.aendern(true, move |t| t.charsets.push(p)) }
    pub fn zeichensatz_benutzt(&self, i: usize) -> bool { self.ts.tiles.iter().any(|t| t.cells.iter().any(|c| c.cs == i)) }
    pub fn zeichensatz_entfernen(&mut self, idx: usize) {
        if idx >= self.ts.charsets.len() || self.zeichensatz_benutzt(idx) { return }
        self.aendern(true, move |t| { t.charsets.remove(idx); for tile in &mut t.tiles { for c in &mut tile.cells { if c.cs > idx { c.cs -= 1 } } } })
    }
    pub fn reiter_waehlen(&mut self, i: usize) { if i < self.ts.charsets.len() { self.reiter = i; self.pinsel.cs = i } }

    // -- Bilder ---------------------------------------------------------------------------------------------------------
    fn zellfarbe(&self, t: &Tile, c: &Cell) -> usize { match self.ts.color_mode { ColorMode::Tile => t.col as usize, ColorMode::Cell => c.col.unwrap_or(t.col) as usize, ColorMode::None => self.media["colors"]["fg"].as_u64().map_or(14, |v| v as usize) } }
    pub fn hintergrundfarbe(&self) -> usize { self.media["colors"]["bg"].as_u64().map_or(6, |v| v as usize) }
    /// Eine Kachel als Bild (`skala` Bildpunkte je Zeichenpixel)
    pub fn kachel_bild(&self, i: usize, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3]) -> Bild {
        let (tw, th, s) = (self.ts.tw, self.ts.th, skala.max(1));
        let mut b = Bild::neu(tw * 8 * s, th * 8 * s, rgb(self.hintergrundfarbe()));
        let Some(t) = self.ts.tiles.get(i) else { return b };
        let mc = McCols::aus_farben(&self.media["colors"]);
        for (k, c) in t.cells.iter().enumerate() {
            if let Some(Some(g)) = self.glyphs.get(c.cs) { g.zeichne(&mut b, (k % tw) * 8 * s, (k / tw) * 8 * s, s, &self.target, c.code, self.zellfarbe(t, c), mc, c.sel, rgb); }
        }
        b
    }
    /// Zeichentafel des Reiters `reiter` (16 Zeichen je Zeile)
    pub fn tafel_bild(&self, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3]) -> Bild {
        let s = skala.max(1); let n = if self.atari() { 128 } else { 256 };
        let mut b = Bild::neu(16 * 8 * s, (n / 16) * 8 * s, rgb(self.hintergrundfarbe()));
        let fg = self.ts.tiles.get(self.sel).map_or(1, |t| t.col) as usize;
        let mut f = match self.ts.color_mode { ColorMode::None => self.media["colors"]["fg"].as_u64().map_or(14, |v| v as usize), ColorMode::Cell => self.farbe, ColorMode::Tile => fg };
        if matches!(self.target.as_str(), "c64" | "mega65" | "vic20") && self.glyphs.get(self.reiter).and_then(|g| g.as_ref()).is_some_and(|g| g.mc) { f |= 8 }
        if let Some(Some(g)) = self.glyphs.get(self.reiter) { for c in 0..n { g.zeichne(&mut b, (c % 16) * 8 * s, (c / 16) * 8 * s, s, &self.target, c as u8, f, McCols::aus_farben(&self.media["colors"]), false, rgb); } }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn zellen(v: &[u8]) -> Vec<Cell> { v.iter().map(|&code| Cell { code, ..leer() }).collect() }
    fn codes(c: &[Cell]) -> Vec<u8> { c.iter().map(|x| x.code).collect() }
    fn kopie(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vc-tilesetdoc-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        for f in ["font.vcchar", "level_tiles.vctiles", "level.vcscreen", "title.vcscreen"] { std::fs::copy(repo().join("examples/c64/gfx_demo/graphics").join(f), dir.join("graphics").join(f)).unwrap(); }
        dir
    }

    #[test]
    fn zellen_spiegeln_drehen_rollen() {
        let mut c = zellen(&[1, 2, 3, 4]);
        spiegeln_h(&mut c, 2, 2); assert_eq!(codes(&c), [2, 1, 4, 3]); spiegeln_v(&mut c, 2, 2); assert_eq!(codes(&c), [4, 3, 2, 1]);
        let mut d = zellen(&[1, 2, 3, 4]); drehen(&mut d, 2); assert_eq!(codes(&d), [3, 1, 4, 2], "90° im Uhrzeigersinn");
        let mut r = zellen(&[1, 2, 3, 4, 5, 6]); rollen(&mut r, 3, 2, 1, 0); assert_eq!(codes(&r), [3, 1, 2, 6, 4, 5]); rollen(&mut r, 3, 2, 0, 1); assert_eq!(codes(&r), [6, 4, 5, 3, 1, 2]);
    }

    #[test]
    fn groesse_beschneidet_und_fuellt() {
        let (data, dir) = (repo().join("data"), kopie("g")); let mut d = TilesetDoc::oeffnen(&dir, "graphics/level_tiles.vctiles", &data).unwrap();
        assert_eq!((d.ts.tw, d.ts.th), (2, 2)); d.groesse_anfragen(3, 3); assert_eq!((d.ts.tw, d.ts.th, d.ts.tiles[1].cells.len()), (3, 3, 9), "größer: aufgefüllt, keine Rückfrage"); assert_eq!(d.ts.tiles[1].cells[0].code, 1);
        d.groesse_anfragen(2, 2); assert_eq!(d.frage, None, "belegte Zellen gehen nicht verloren, weil die neuen leer sind"); assert_eq!(d.ts.tiles[1].cells.len(), 4);
        d.malen(3, true); let k = 3; assert!(d.ts.tiles[d.sel].cells[k].code != 0 || d.sel == 0);
        d.sel = 1; d.pinsel.code = 9; d.malen(3, true); d.groesse_anfragen(1, 1); assert_eq!(d.frage, Some(Frage::Groesse(1, 1)), "Zellen gehen verloren: Rückfrage"); d.frage_bestaetigen().unwrap(); assert_eq!((d.ts.tw, d.ts.th), (1, 1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oeffnen_bearbeiten_speichern_verwendung_und_fuellstand() {
        let (data, dir) = (repo().join("data"), kopie("a")); let mut d = TilesetDoc::oeffnen(&dir, "graphics/level_tiles.vctiles", &data).unwrap();
        assert_eq!((d.ts.tiles.len(), d.ts.tw, d.ts.th, d.ts.color_mode), (4, 2, 2, ColorMode::Tile)); assert!(!d.dirty()); assert!(d.fill.as_ref().is_some_and(|f| f.err.is_none() && f.used > 0), "{:?}", d.fill);
        assert!(!d.banken.is_empty(), "level.vcscreen nutzt den Satz"); assert!(d.verwendung(1) > 0, "Stein ist benutzt");
        // Zelle malen: ein Rückgängig-Schritt je Zug, Pinsel aus der Tafel
        d.kachel_waehlen(2); d.zelle_waehlen(2); d.zeichen_waehlen(7); assert_eq!(d.ts.tiles[2].cells[2].code, 7); assert!(d.dirty()); let n0 = d.undo.len(); d.malen(3, false); d.malen(1, false); assert_eq!(d.undo.len(), n0, "Folgepunkte eines Zuges ohne neuen Punkt");
        d.rueckgaengig(); assert_ne!(d.ts.tiles[2].cells[2].code, 7, "ein Schritt nimmt den ganzen Zug zurück"); assert!(d.ts.tiles[2].cells[1].code != 7 && d.ts.tiles[2].cells[3].code != 7);
        d.wiederholen(); assert_eq!((d.ts.tiles[2].cells[2].code, d.ts.tiles[2].cells[3].code, d.ts.tiles[2].cells[1].code), (7, 7, 7));
        // Name, Farbe, Eigenschaft, Farbmodus
        d.kachel_umbenennen("wiese", true); d.kachelfarbe(5); d.eigenschaft_setzen(0, true); assert_eq!((d.ts.tiles[2].name.as_str(), d.ts.tiles[2].col, d.ts.tiles[2].props), ("wiese", 5, 1)); assert!(d.eigenschaft_neu("tod") && !d.eigenschaft_neu("tod") && !d.eigenschaft_neu(""));
        d.farbmodus(ColorMode::Cell); assert_eq!(d.ts.color_mode, ColorMode::Cell);
        d.speichern().unwrap(); assert!(!d.dirty()); let d2 = TilesetDoc::oeffnen(&dir, "graphics/level_tiles.vctiles", &data).unwrap(); assert_eq!((d2.ts.tiles[2].name.as_str(), d2.ts.tiles[2].col, d2.ts.color_mode, d2.ts.prop_defs.len()), ("wiese", 5, ColorMode::Cell, 2));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn kacheln_neu_duplizieren_loeschen_zusammenfuehren_mit_banken() {
        let (data, dir) = (repo().join("data"), kopie("k")); let mut d = TilesetDoc::oeffnen(&dir, "graphics/level_tiles.vctiles", &data).unwrap();
        let bank_vorher = std::fs::read_to_string(dir.join("graphics/level.vcscreen")).unwrap();
        d.kachel_neu(); assert_eq!((d.ts.tiles.len(), d.sel), (5, 4)); d.kachel_duplizieren(); assert_eq!((d.ts.tiles.len(), d.sel, d.doppelt[5]), (6, 5, Some(4)), "Duplikat erkannt");
        d.zusammenfuehren().unwrap(); assert_eq!(d.ts.tiles.len(), 5, "Zusammenführen entfernt die Kopie"); assert_eq!(std::fs::read_to_string(dir.join("graphics/level.vcscreen")).unwrap(), bank_vorher, "Bank unverändert: Kachel 5 wurde nirgends benutzt");
        // letzte unbenutzte Kachel löschen: sofort; benutzte Kachel (Stein = 1) mit Rückfrage, danach Banken angepasst
        d.kachel_waehlen(4); d.kachel_loeschen(); assert_eq!((d.ts.tiles.len(), d.frage.clone()), (4, None), "unbenutzt und letzte: ohne Rückfrage");
        d.kachel_waehlen(1); d.kachel_loeschen(); assert_eq!(d.frage, Some(Frage::Loeschen(1)), "benutzt: Rückfrage");
        d.frage_abbrechen(); assert_eq!(d.ts.tiles.len(), 4); d.kachel_loeschen(); d.frage_bestaetigen().unwrap();
        assert_eq!(d.ts.tiles.len(), 3); assert!(!d.dirty(), "Löschen mit Bänken speichert den Satz"); let nach = std::fs::read_to_string(dir.join("graphics/level.vcscreen")).unwrap(); assert_ne!(nach, bank_vorher, "Verweise umgeschrieben");
        let v: Value = serde_json::from_str(&nach).unwrap(); let flach: Vec<u64> = v["screens"][0]["tiles"]["grid"].as_array().unwrap().iter().flat_map(|r| r.as_array().unwrap().iter().filter_map(|x| x.as_u64())).collect(); assert!(flach.iter().all(|&n| n < 3), "keine Nummer ≥ Kachelzahl: {flach:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zellenoperationen_der_kachel_zeichensaetze_und_bilder() {
        let (data, dir) = (repo().join("data"), kopie("z")); let mut d = TilesetDoc::oeffnen(&dir, "graphics/level_tiles.vctiles", &data).unwrap();
        d.kachel_waehlen(1); let vor = codes(&d.ts.tiles[1].cells); d.spiegeln(true); assert_eq!(codes(&d.ts.tiles[1].cells), [vor[1], vor[0], vor[3], vor[2]]); d.spiegeln(true); assert_eq!(codes(&d.ts.tiles[1].cells), vor);
        d.drehen(); d.drehen(); d.drehen(); d.drehen(); assert_eq!(codes(&d.ts.tiles[1].cells), vor, "viermal drehen"); d.kopieren(); d.leeren(); assert!(d.ts.tiles[1].cells.iter().all(|c| c.code == 0)); d.kachel_waehlen(2); d.einfuegen(); assert_eq!(codes(&d.ts.tiles[2].cells), vor, "Einfügen aus der Zwischenablage");
        d.aufnehmen(1); assert_eq!(d.pinsel.code, vor[1]);
        // Zeichensatz: nur ein Satz im Projekt, der schon dabei ist; benutzt → nicht entfernbar
        assert!(d.zeichensatz_angebot().is_empty() && d.zeichensatz_benutzt(0)); d.zeichensatz_entfernen(0); assert_eq!(d.ts.charsets.len(), 1, "benutzter Zeichensatz bleibt");
        let rgb = |i: usize| -> [u8; 3] { [i as u8 * 10, 3, 4] }; let b = d.kachel_bild(2, 2, &rgb); assert_eq!((b.w, b.h), (2 * 8 * 2, 2 * 8 * 2)); assert!(b.rgba.chunks(4).any(|p| p[..3] != rgb(d.hintergrundfarbe())[..]), "Kachel hat sichtbare Zeichen");
        assert_eq!((d.tafel_bild(1, &rgb).w, d.tafel_bild(1, &rgb).h), (128, 128));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
