// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: GUI-freies Editormodell (Zeilenpuffer, Cursor, Auswahl, Rückgängig, Falten, Umbruch, Vervollständigung).
//! Bewusst klein gehalten; Zeilen sind `String`s, Positionen zählen Zeichen (nicht Bytes).
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Pos { pub line: usize, pub col: usize }

#[derive(Clone, Debug)]
struct Edit { at: Pos, removed: String, inserted: String, cursor_before: Pos, cursor_after: Pos }

pub const TAB: usize = 4;

/// Anzeigespalte eines Zeichenindex (Tabulatoren springen zur nächsten Vielfachen von `TAB`)
pub fn vis_col(line: &str, col: usize) -> usize {
    let mut v = 0;
    for ch in line.chars().take(col) { if ch == '\t' { v = (v / TAB + 1) * TAB } else { v += 1 } }
    v
}
/// Zeichenindex, der zu einer Anzeigespalte gehört (rundet auf das Zeichen, in dem die Spalte liegt)
pub fn col_at_vis(line: &str, target: usize) -> usize {
    let mut v = 0;
    for (i, ch) in line.chars().enumerate() {
        let w = if ch == '\t' { (v / TAB + 1) * TAB - v } else { 1 };
        if target < v + w { return i }
        v += w;
    }
    line.chars().count()
}
pub fn vis_len(line: &str) -> usize { vis_col(line, usize::MAX) }

/// Hinweis des Sprachdienstes zu einer Stelle: gilt für den Bereich von bis (ausschließlich, eine Zeile), nicht für den Textcursor
#[derive(Clone, Debug, PartialEq)]
pub struct HoverInfo { pub von: Pos, pub bis: Pos, pub text: String }
impl HoverInfo { pub fn enthaelt(&self, p: Pos) -> bool { p.line == self.von.line && p.line == self.bis.line && p.col >= self.von.col && p.col < self.bis.col } }

#[derive(Clone)]
pub struct Completion { pub items: Vec<String>, pub details: Vec<String>, pub sel: usize, pub start: usize }

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: Pos,
    pub anchor: Option<Pos>,
    pub want_col: usize,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    pub folds: BTreeMap<usize, usize>,
    pub breakpoints: HashSet<usize>,
    pub errors: HashMap<usize, String>,
    pub warns: HashMap<usize, String>,
    /// Hinweise (Schweregrad 3 des Sprachdienstes, z. B. Folgefehler einer Include-Datei ohne Wurzel): dezente Markierung, Text im Hover
    pub hints: HashMap<usize, String>,
    pub center_pending: Option<usize>,
    pub matches: Vec<(Pos, Pos)>,
    pub match_lines: HashMap<usize, Vec<(usize, usize, bool)>>,
    pub current_match: Option<usize>,
    pub debug_line: Option<usize>,
    /// Einfügemarke beim Ziehen einer Datei aus dem Projektbaum: Zeilengrenze (0..=Zeilenzahl), vor der eingefügt würde
    pub einfuegemarke: Option<usize>,
    pub wrap: bool,
    pub wrap_cols: usize,
    pub scroll_row: f32,
    pub hscroll: usize,
    pub view_rows: usize,
    pub completion: Option<Completion>,
    /// Vorschläge des Sprachdienstes (Text, Beschreibung) für die aktuelle Datei; ergänzen den eingebauten Wortschatz
    pub lsp_items: Vec<(String, String)>,
    /// Hinweistext des Sprachdienstes (Markdown) zur Stelle unter der Maus
    pub hover: Option<HoverInfo>,
    pub preedit: Option<String>,
    pub version: u64,
    rows_cache: Vec<usize>,   // erste Anzeigezeile je logische Zeile (Länge = Zeilen + 1)
    rows_version: u64,
    rows_key: (usize, bool, u64),
    fold_version: u64,
    maxvis_cache: std::cell::Cell<(u64, usize)>,
    /// Sprache der Datei (bestimmt die Faltregeln)
    pub lang: vc_ide_model::lang::Lang,
    fold_cache: std::cell::RefCell<(u64, vc_ide_model::lang::Lang, BTreeMap<usize, usize>)>,
    pub dirty: bool,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        let mut lines: Vec<String> = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect();
        if lines.is_empty() { lines.push(String::new()) }
        Editor { lines, cursor: Pos::default(), anchor: None, want_col: 0, undo: vec![], redo: vec![], folds: BTreeMap::new(), breakpoints: HashSet::new(), errors: HashMap::new(), warns: HashMap::new(), hints: HashMap::new(), center_pending: None, matches: vec![], match_lines: HashMap::new(), current_match: None, debug_line: None, einfuegemarke: None, wrap: false, wrap_cols: 80, scroll_row: 0.0, hscroll: 0, view_rows: 30, completion: None, lsp_items: vec![], hover: None, preedit: None, version: 1, rows_cache: vec![], rows_version: 0, rows_key: (0, false, 0), fold_version: 0, maxvis_cache: std::cell::Cell::new((u64::MAX, 0)), lang: vc_ide_model::lang::Lang::Asm, fold_cache: std::cell::RefCell::new((u64::MAX, vc_ide_model::lang::Lang::Asm, BTreeMap::new())), dirty: false }
    }
    pub fn text(&self) -> String { self.lines.join("\n") }
    pub fn line_len(&self, l: usize) -> usize { self.lines[l].chars().count() }
    fn byte_of(&self, p: Pos) -> usize { self.lines[p.line].char_indices().nth(p.col).map(|x| x.0).unwrap_or(self.lines[p.line].len()) }
    fn clamp(&self, p: Pos) -> Pos { let line = p.line.min(self.lines.len() - 1); Pos { line, col: p.col.min(self.line_len(line)) } }

    // ---- Auswahl ----
    pub fn selection(&self) -> Option<(Pos, Pos)> { self.anchor.filter(|a| *a != self.cursor).map(|a| if a < self.cursor { (a, self.cursor) } else { (self.cursor, a) }) }
    pub fn selected_text(&self) -> Option<String> { self.selection().map(|(a, b)| self.text_range(a, b)) }
    pub fn text_range(&self, a: Pos, b: Pos) -> String {
        if a.line == b.line { return self.lines[a.line].chars().skip(a.col).take(b.col - a.col).collect() }
        let mut s: String = self.lines[a.line].chars().skip(a.col).collect();
        for l in a.line + 1..b.line { s.push('\n'); s.push_str(&self.lines[l]) }
        s.push('\n'); s.extend(self.lines[b.line].chars().take(b.col)); s
    }
    pub fn select_all(&mut self) { self.anchor = Some(Pos::default()); let l = self.lines.len() - 1; self.cursor = Pos { line: l, col: self.line_len(l) }; self.want_col = vis_col(&self.lines[l], self.cursor.col) }

    // ---- Kernoperation: Bereich ersetzen ----
    fn replace_raw(&mut self, a: Pos, b: Pos, text: &str) -> Pos {
        let a = self.clamp(a); let b = self.clamp(b);
        let (ba, bb) = (self.byte_of(a), self.byte_of(b));
        let tail = self.lines[b.line][bb..].to_string();
        let head = self.lines[a.line][..ba].to_string();
        let mut new: Vec<String> = text.split('\n').map(String::from).collect();
        let last_len = new.last().map(|s| s.chars().count()).unwrap_or(0);
        let first_len = head.chars().count();
        new[0] = format!("{head}{}", new[0]);
        let n = new.len(); new[n - 1].push_str(&tail);
        let end = if n == 1 { Pos { line: a.line, col: first_len + last_len } } else { Pos { line: a.line + n - 1, col: last_len } };
        self.lines.splice(a.line..=b.line, new);
        self.version += 1; self.dirty = true;
        end
    }
    pub fn replace(&mut self, a: Pos, b: Pos, text: &str) {
        let removed = self.text_range(a, b);
        let before = self.cursor;
        let end = self.replace_raw(a, b, text);
        self.cursor = end; self.anchor = None; self.want_col = vis_col(&self.lines[end.line], end.col);
        // Zusammenfassen: aufeinanderfolgendes Tippen einzelner Zeichen
        if removed.is_empty() && text.chars().count() == 1 && text != "\n" {
            if let Some(last) = self.undo.last_mut() { if last.removed.is_empty() && !last.inserted.contains('\n') && last.cursor_after == a && last.inserted.chars().count() < 40 { last.inserted.push_str(text); last.cursor_after = end; self.redo.clear(); self.after_edit(); return } }
        }
        self.undo.push(Edit { at: a, removed, inserted: text.to_string(), cursor_before: before, cursor_after: end });
        self.redo.clear(); self.after_edit();
    }
    fn after_edit(&mut self) { self.fix_folds(); self.ensure_visible() }
    pub fn insert(&mut self, text: &str) { let (a, b) = self.selection().unwrap_or((self.cursor, self.cursor)); self.replace(a, b, text) }
    pub fn undo(&mut self) {
        let Some(e) = self.undo.pop() else { return };
        let end = Pos { line: e.at.line + e.inserted.matches('\n').count(), col: if e.inserted.contains('\n') { e.inserted.rsplit('\n').next().unwrap().chars().count() } else { e.at.col + e.inserted.chars().count() } };
        self.replace_raw(e.at, end, &e.removed); self.cursor = e.cursor_before; self.anchor = None; self.redo.push(e); self.after_edit();
    }
    pub fn redo(&mut self) {
        let Some(e) = self.redo.pop() else { return };
        let end = Pos { line: e.at.line + e.removed.matches('\n').count(), col: if e.removed.contains('\n') { e.removed.rsplit('\n').next().unwrap().chars().count() } else { e.at.col + e.removed.chars().count() } };
        self.replace_raw(e.at, end, &e.inserted); self.cursor = e.cursor_after; self.anchor = None; self.undo.push(e); self.after_edit();
    }
    pub fn backspace(&mut self) {
        if self.selection().is_some() { return self.insert("") }
        let c = self.cursor;
        if c.col > 0 { self.replace(Pos { line: c.line, col: c.col - 1 }, c, "") } else if c.line > 0 { let l = c.line - 1; self.replace(Pos { line: l, col: self.line_len(l) }, c, "") }
    }
    pub fn delete(&mut self) {
        if self.selection().is_some() { return self.insert("") }
        let c = self.cursor;
        if c.col < self.line_len(c.line) { self.replace(c, Pos { line: c.line, col: c.col + 1 }, "") } else if c.line + 1 < self.lines.len() { self.replace(c, Pos { line: c.line + 1, col: 0 }, "") }
    }
    pub fn enter(&mut self) {
        let indent: String = self.lines[self.cursor.line].chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        self.insert(&format!("\n{indent}"))
    }

    // ---- Cursorbewegung ----
    pub fn set_cursor(&mut self, p: Pos, extend: bool) {
        let p = self.clamp(p);
        if extend { if self.anchor.is_none() { self.anchor = Some(self.cursor) } } else { self.anchor = None }
        self.cursor = p; self.want_col = vis_col(&self.lines[p.line], p.col); self.completion = None; self.ensure_visible();
    }
    pub fn move_h(&mut self, dx: i32, extend: bool) {
        if !extend { if let Some((a, b)) = self.selection() { self.anchor = None; self.cursor = if dx < 0 { a } else { b }; self.want_col = vis_col(&self.lines[self.cursor.line], self.cursor.col); self.ensure_visible(); return } }
        let c = self.cursor;
        let p = if dx < 0 { if c.col > 0 { Pos { line: c.line, col: c.col - 1 } } else if c.line > 0 { Pos { line: c.line - 1, col: self.line_len(c.line - 1) } } else { c } }
                else if c.col < self.line_len(c.line) { Pos { line: c.line, col: c.col + 1 } } else if c.line + 1 < self.lines.len() { Pos { line: c.line + 1, col: 0 } } else { c };
        self.set_cursor(p, extend)
    }
    /// Cursor um `n` sichtbare Zeilen verschieben (Falten übersprungen); die Wunschspalte bleibt erhalten
    pub fn move_v(&mut self, n: i32, extend: bool) {
        let mut l = self.cursor.line as i64; let step = n.signum() as i64;
        for _ in 0..n.abs() { let mut nl = l + step; while nl >= 0 && (nl as usize) < self.lines.len() && self.is_hidden(nl as usize) { nl += step } if nl < 0 || nl as usize >= self.lines.len() { break } l = nl }
        let line = l as usize;
        let want = self.want_col;
        let p = if l as usize == self.cursor.line && n != 0 { if n < 0 { Pos { line, col: 0 } } else { Pos { line, col: self.line_len(line) } } } else { Pos { line, col: col_at_vis(&self.lines[line], want) } };
        if extend && self.anchor.is_none() { self.anchor = Some(self.cursor) } else if !extend { self.anchor = None }
        self.cursor = p; self.completion = None; self.ensure_visible(); // want_col bleibt
    }
    pub fn home(&mut self, extend: bool) { let l = self.cursor.line; let first = self.lines[l].chars().take_while(|c| c.is_whitespace()).count(); let col = if self.cursor.col == first { 0 } else { first }; self.set_cursor(Pos { line: l, col }, extend) }
    pub fn end(&mut self, extend: bool) { let l = self.cursor.line; self.set_cursor(Pos { line: l, col: self.line_len(l) }, extend) }

    // ---- Falten ----
    pub fn is_hidden(&self, line: usize) -> bool { self.folds.range(..line).next_back().map_or(false, |(_, e)| line <= *e) }
    /// Faltbereich ab einer Zeile (letzte Zeile des Bereichs) nach den Regeln aus `vc-ide-model::fold` (Blöcke, Marken-Abschnitte, Kommentarblöcke)
    pub fn fold_range(&self, line: usize) -> Option<usize> {
        let mut c = self.fold_cache.borrow_mut();
        if c.0 != self.version || c.1 != self.lang { *c = (self.version, self.lang, vc_ide_model::fold::ranges(&self.text(), self.lang)) }
        c.2.get(&line).copied()
    }
    pub fn toggle_fold(&mut self, line: usize) {
        if self.folds.remove(&line).is_none() { if let Some(e) = self.fold_range(line) { self.folds.insert(line, e); if self.cursor.line > line && self.cursor.line <= e { self.cursor = Pos { line, col: 0 } } } }
        self.fold_version += 1;
    }
    fn fix_folds(&mut self) { if !self.folds.is_empty() { self.folds.clear(); self.fold_version += 1 } } // wie im Altbestand: Bearbeiten löst alle Falten

    // ---- Anzeigezeilen (Umbruch, Falten) ----
    fn row_count(&self, l: usize) -> usize { if self.is_hidden(l) { 0 } else if self.wrap { (vis_len(&self.lines[l]).max(1) + self.wrap_cols - 1) / self.wrap_cols.max(1) } else { 1 } }
    pub fn rebuild_rows(&mut self) {
        let key = (self.wrap_cols, self.wrap, self.fold_version);
        if self.rows_version == self.version && self.rows_key == key && !self.rows_cache.is_empty() { return }
        let n = self.lines.len(); let mut v = Vec::with_capacity(n + 1); let mut acc = 0;
        if !self.wrap && self.folds.is_empty() { v.extend(0..=n) } else { for l in 0..n { v.push(acc); acc += self.row_count(l) } v.push(acc) }
        self.rows_cache = v; self.rows_version = self.version; self.rows_key = key;
    }
    pub fn total_rows(&mut self) -> usize { self.rebuild_rows(); *self.rows_cache.last().unwrap() }
    /// (logische Zeile, Teilzeile) zu einer Anzeigezeile
    pub fn line_of_row(&mut self, row: usize) -> (usize, usize) {
        self.rebuild_rows();
        let i = self.rows_cache.partition_point(|s| *s <= row).saturating_sub(1).min(self.lines.len() - 1);
        // bei versteckten Zeilen (Länge 0) auf die letzte Zeile mit demselben Start gehen
        let mut l = i; while l > 0 && self.rows_cache[l] == self.rows_cache[l + 1] { l -= 1 }
        (l, row.saturating_sub(self.rows_cache[l]))
    }
    pub fn row_of(&mut self, p: Pos) -> usize { self.rebuild_rows(); let l = p.line.min(self.lines.len() - 1); self.rows_cache[l] + if self.wrap { vis_col(&self.lines[l], p.col) / self.wrap_cols.max(1) } else { 0 } }
    pub fn ensure_visible(&mut self) {
        let r = self.row_of(self.cursor) as f32;
        if r < self.scroll_row { self.scroll_row = r } else if r >= self.scroll_row + self.view_rows as f32 - 1.0 { self.scroll_row = r - self.view_rows as f32 + 2.0 }
        self.scroll_row = self.scroll_row.max(0.0);
        if self.wrap { self.hscroll = 0 } else { let v = vis_col(&self.lines[self.cursor.line], self.cursor.col); let cols = self.wrap_cols; if v < self.hscroll { self.hscroll = v } else if v + 3 >= self.hscroll + cols { self.hscroll = v + 4 - cols.min(v + 4) } }
    }
    /// Anzahl der Anzeigezeilen laut Zwischenspeicher (ohne Neuberechnung; für die senkrechte Scrollleiste)
    pub fn rows_total_ro(&self) -> usize { self.rows_cache.last().copied().unwrap_or(self.lines.len()).max(self.lines.len().min(1)) }
    /// Breite der längsten Zeile in Anzeigespalten (für die waagerechte Scrollleiste)
    pub fn max_vis(&self) -> usize {
        let (v, m) = self.maxvis_cache.get(); if v == self.version { return m }
        let m = self.lines.iter().map(|l| vis_len(l)).max().unwrap_or(0); self.maxvis_cache.set((self.version, m)); m
    }
    /// Größte sinnvolle waagerechte Verschiebung (0, wenn alles passt oder umbrochen wird)
    pub fn hscroll_max(&self) -> usize { if self.wrap { 0 } else { (self.max_vis() + 4).saturating_sub(self.wrap_cols) } }
    pub fn set_hscroll(&mut self, v: usize) { self.hscroll = v.min(self.hscroll_max()) }
    /// Zeile in die Mitte des Sichtbereichs rücken (Sprung von einem Problem); gilt erneut, sobald die Fenstergröße bekannt ist
    pub fn center_on(&mut self, line: usize) { let r = self.row_of(Pos { line, col: 0 }) as f32; self.scroll_row = (r - self.view_rows as f32 / 2.0).max(0.0); self.center_pending = Some(line) }
    pub fn scroll_by(&mut self, rows: f32) { let max = (self.total_rows() as f32 - 3.0).max(0.0); self.scroll_row = (self.scroll_row + rows).clamp(0.0, max) }

    // ---- Suchen und Ersetzen ----
    /// Alle Treffer von `q` (nicht überlappend, je Zeile; Spalten in Zeichen). Leere Suche: keine Treffer.
    pub fn finde_alle(lines: &[String], q: &str, gross_klein: bool) -> Vec<(Pos, Pos)> {
        if q.is_empty() { return vec![] }
        let qc: Vec<char> = if gross_klein { q.chars().collect() } else { q.to_lowercase().chars().collect() };
        let mut out = vec![];
        for (l, line) in lines.iter().enumerate() {
            let lc: Vec<char> = if gross_klein { line.chars().collect() } else { line.to_lowercase().chars().collect() };
            if lc.len() != line.chars().count() { continue } // Zeichen mit anderer Länge in Kleinschreibung: überspringen
            let mut i = 0; while i + qc.len() <= lc.len() { if lc[i..i + qc.len()] == qc[..] { out.push((Pos { line: l, col: i }, Pos { line: l, col: i + qc.len() })); i += qc.len() } else { i += 1 } }
        }
        out
    }
    /// Suche setzen (leer = aus); wählt den ersten Treffer ab dem Cursor
    pub fn set_search(&mut self, q: &str, gross_klein: bool) {
        self.matches = Self::finde_alle(&self.lines, q, gross_klein); self.match_lines.clear();
        let cur = self.cursor;
        self.current_match = self.matches.iter().position(|m| m.0 >= cur).or(if self.matches.is_empty() { None } else { Some(0) });
        self.rebuild_match_lines();
    }
    fn rebuild_match_lines(&mut self) { self.match_lines.clear(); for (i, m) in self.matches.iter().enumerate() { self.match_lines.entry(m.0.line).or_default().push((m.0.col, m.1.col, Some(i) == self.current_match)) } }
    /// Zum nächsten (+1) oder vorigen (-1) Treffer, mit Umlauf; liefert die Nummer (ab 1)
    pub fn step_match(&mut self, dir: i32) -> Option<usize> {
        if self.matches.is_empty() { return None }
        let n = self.matches.len() as i32; let cur = self.current_match.map(|i| i as i32).unwrap_or(if dir > 0 { -1 } else { 0 });
        let i = (cur + dir).rem_euclid(n) as usize; self.current_match = Some(i);
        let m = self.matches[i]; self.anchor = Some(m.0); self.cursor = m.1; self.want_col = vis_col(&self.lines[m.1.line], m.1.col); self.rebuild_match_lines(); self.center_on(m.1.line); Some(i + 1)
    }
    pub fn replace_current(&mut self, with: &str, q: &str, gross_klein: bool) {
        if let Some(i) = self.current_match { let m = self.matches[i]; self.replace(m.0, m.1, with); self.set_search(q, gross_klein); if !self.matches.is_empty() { let k = self.matches.iter().position(|x| x.0 >= self.cursor).unwrap_or(0); self.current_match = Some(k); self.step_match(0); } }
    }
    /// Alle Treffer ersetzen (ein Rückgängig-Schritt); liefert die Anzahl
    pub fn replace_all(&mut self, with: &str, q: &str, gross_klein: bool) -> usize {
        let ms = Self::finde_alle(&self.lines, q, gross_klein); if ms.is_empty() { return 0 }
        let mut lines = self.lines.clone();
        for (a, b) in ms.iter().rev() { let ch: Vec<char> = lines[a.line].chars().collect(); let mut n: String = ch[..a.col].iter().collect(); n.push_str(with); n.extend(ch[b.col..].iter()); lines[a.line] = n }
        let last = self.lines.len() - 1; let end = Pos { line: last, col: self.line_len(last) };
        let (cur, anc) = (self.cursor, self.anchor); self.replace(Pos::default(), end, &lines.join("\n")); self.cursor = Pos { line: cur.line.min(self.lines.len() - 1), col: 0 }; self.anchor = anc.map(|_| self.cursor);
        self.set_search(q, gross_klein); ms.len()
    }
    pub fn clear_search(&mut self) { self.matches.clear(); self.match_lines.clear(); self.current_match = None }

    // ---- Vervollständigung ----
    pub fn update_completion(&mut self, vocab: &[&str]) { self.update_completion_ab(vocab, 2) }
    /// Wie `update_completion`, aber schon ab `min` getippten Zeichen (0: auch ohne Anfang; für den Befehl „Vervollständigung öffnen“)
    pub fn update_completion_ab(&mut self, vocab: &[&str], min: usize) {
        let l = &self.lines[self.cursor.line]; let chars: Vec<char> = l.chars().collect();
        let mut s = self.cursor.col; while s > 0 && (chars[s - 1].is_alphanumeric() || chars[s - 1] == '_' || chars[s - 1] == '.') { s -= 1 }
        let prefix: String = chars[s..self.cursor.col].iter().collect();
        if prefix.chars().count() < min { self.completion = None; return }
        let low = prefix.to_lowercase();
        let mut items: Vec<String> = vec![]; let mut details: Vec<String> = vec![];
        for (t, d) in &self.lsp_items { if t.to_lowercase().starts_with(&low) && t.to_lowercase() != low && !items.iter().any(|i| i.eq_ignore_ascii_case(t)) { items.push(t.clone()); details.push(d.clone()) } }
        for w in vocab.iter().filter(|w| w.starts_with(&low) && **w != low) { if !items.iter().any(|i| i.eq_ignore_ascii_case(w)) { items.push(w.to_string()); details.push(String::new()) } }
        // Marken der Datei
        for ln in self.lines.iter().take(20000) { if let Some(name) = ln.strip_suffix(':') { if !name.starts_with(char::is_whitespace) && name.to_lowercase().starts_with(&low) && name != prefix && !items.contains(&name.to_string()) { items.push(name.to_string()); details.push(String::new()) } } }
        items.truncate(8); details.truncate(8);
        self.completion = (!items.is_empty()).then_some(Completion { items, details, sel: 0, start: s });
    }
    pub fn accept_completion(&mut self) -> bool {
        let Some(c) = self.completion.take() else { return false };
        let word = c.items[c.sel].clone(); let cur = self.cursor;
        self.replace(Pos { line: cur.line, col: c.start }, cur, &word); true
    }
    pub fn resize(&mut self, cols: usize, rows: usize) { if cols != self.wrap_cols || rows != self.view_rows { self.wrap_cols = cols.max(10); self.view_rows = rows.max(2); if let Some(l) = self.center_pending { self.center_on(l) } } }
}

#[cfg(test)]
mod tests {
    #[test] fn vorschlaege_des_sprachdienstes_ergaenzen_den_wortschatz() {
        let mut e = Editor::new("vic"); e.cursor = Pos { line: 0, col: 3 }; e.lsp_items = vec![("vic2.border".into(), "$D020 · rw".into()), ("vicious".into(), String::new())];
        e.update_completion(&["vice"]); let c = e.completion.clone().unwrap();
        assert_eq!(c.items, ["vic2.border", "vicious", "vice"]); assert_eq!(c.details[0], "$D020 · rw"); assert_eq!(c.start, 0);
    }
    #[test] fn waagerechte_verschiebung_ist_begrenzt() {
        let mut e = Editor::new(&format!("kurz\n{}", "x".repeat(200))); e.resize(80, 20);
        assert_eq!(e.max_vis(), 200); assert_eq!(e.hscroll_max(), 124); e.set_hscroll(9999); assert_eq!(e.hscroll, 124);
        e.wrap = true; e.set_hscroll(50); assert_eq!(e.hscroll, 0, "bei Umbruch gibt es keine waagerechte Verschiebung");
        let k = Editor::new("kurz"); assert_eq!(k.hscroll_max(), 0);
    }
    #[test] fn falten_nach_regeln_des_modells() {
        let mut e = Editor::new("start:\n  lda #1\n  rts\nende:\n  nop\n  .macro m\n  nop\n  .endmacro\n"); e.lang = vc_ide_model::lang::Lang::Asm;
        assert_eq!(e.fold_range(0), Some(2), "Marken-Abschnitt bis vor die nächste Marke"); assert_eq!(e.fold_range(5), Some(7), "Makro-Block"); assert_eq!(e.fold_range(1), None);
        e.toggle_fold(0); assert!(e.is_hidden(1) && e.is_hidden(2) && !e.is_hidden(3));
        e.lang = vc_ide_model::lang::Lang::Plain; assert_eq!(e.fold_range(0), None, "Text faltet nicht");
    }
    use super::*;
    #[test]
    fn tippen_und_rueckgaengig() {
        let mut e = Editor::new("abc\ndef");
        e.set_cursor(Pos { line: 0, col: 3 }, false);
        e.insert("x"); e.insert("y"); assert_eq!(e.lines[0], "abcxy");
        e.undo(); assert_eq!(e.lines[0], "abc", "getipptes wird zusammengefasst");
        e.redo(); assert_eq!(e.lines[0], "abcxy");
        e.insert("\n"); assert_eq!(e.lines.len(), 3); e.undo(); assert_eq!(e.lines.len(), 2);
    }
    #[test]
    fn mehrzeilig_ersetzen_und_zurueck() {
        let mut e = Editor::new("eins\nzwei\ndrei");
        e.replace(Pos { line: 0, col: 2 }, Pos { line: 2, col: 2 }, "X\nY");
        assert_eq!(e.text(), "eiX\nYei"); e.undo(); assert_eq!(e.text(), "eins\nzwei\ndrei");
    }
    #[test]
    fn wunschspalte_bei_kurzen_zeilen_und_umlauten() {
        let mut e = Editor::new("äöü lang genug\nkurz\nwieder lang genug");
        e.set_cursor(Pos { line: 0, col: 10 }, false);
        e.move_v(1, false); assert_eq!(e.cursor, Pos { line: 1, col: 4 });
        e.move_v(1, false); assert_eq!(e.cursor, Pos { line: 2, col: 10 }, "Wunschspalte bleibt erhalten");
    }
    #[test]
    fn tabulatoren_haben_anzeigebreite() {
        assert_eq!(vis_col("\tab", 1), 4); assert_eq!(vis_col("a\tb", 2), 4); assert_eq!(col_at_vis("a\tb", 3), 1);
    }
    #[test]
    fn falten_blendet_zeilen_aus_und_cursor_springt_darueber() {
        let mut e = Editor::new("start:\n  lda #1\n  sta $d020\nende:\n  rts");
        assert_eq!(e.fold_range(0), Some(2)); e.toggle_fold(0);
        assert!(e.is_hidden(1) && e.is_hidden(2) && !e.is_hidden(3));
        assert_eq!(e.total_rows(), 3); assert_eq!(e.line_of_row(1).0, 3);
        e.set_cursor(Pos { line: 0, col: 0 }, false); e.move_v(1, false); assert_eq!(e.cursor.line, 3);
    }
    #[test]
    fn umbruch_zaehlt_anzeigezeilen() {
        let mut e = Editor::new(&format!("{}\nkurz", "a".repeat(25))); e.wrap = true; e.wrap_cols = 10;
        assert_eq!(e.total_rows(), 4); assert_eq!(e.line_of_row(2), (0, 2)); assert_eq!(e.line_of_row(3), (1, 0));
    }
    #[test]
    fn vervollstaendigung_nimmt_vokabular_und_marken() {
        let mut e = Editor::new("loop:\n  lda #0\n  jmp lo"); e.set_cursor(Pos { line: 2, col: 8 }, false);
        e.update_completion(&["lda", "ldx", "loop"]); let c = e.completion.as_ref().unwrap(); assert_eq!(c.items, vec!["loop"]);
        assert!(e.accept_completion()); assert_eq!(e.lines[2], "  jmp loop");
    }
    #[test]
    fn suchen_und_ersetzen() {
        let mut e = Editor::new("Lda #1\nlda #2 ; LDA\nsta $d020");
        e.set_search("lda", false); assert_eq!(e.matches.len(), 3);
        e.set_search("lda", true); assert_eq!(e.matches.len(), 1);
        e.set_search("lda", false); assert_eq!(e.step_match(1), Some(2)); assert_eq!(e.cursor.line, 1);
        assert_eq!(e.replace_all("ldx", "lda", false), 3); assert_eq!(e.text(), "ldx #1\nldx #2 ; ldx\nsta $d020");
        e.undo(); assert_eq!(e.text(), "Lda #1\nlda #2 ; LDA\nsta $d020", "Alle ersetzen ist ein Schritt");
    }
    #[test]
    fn suche_umlaute_und_umlauf() {
        let mut e = Editor::new("äpfel Äpfel\nbirne"); e.set_search("äpfel", false); assert_eq!(e.matches.len(), 2);
        assert_eq!(e.step_match(1), Some(2)); assert_eq!(e.step_match(1), Some(1), "Umlauf"); assert_eq!(e.step_match(-1), Some(2));
    }
}
