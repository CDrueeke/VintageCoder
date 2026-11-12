// SPDX-License-Identifier: GPL-3.0-or-later
//! Code-Editor-Widget: Farben, Zeilennummern, Haltepunkte, Falten, Fehlerwellen aus Sprachdienst/Build und Code-Vervollständigung.
use crate::fold;
use crate::highlight::{self, Lang, pal_err};
use crate::lsp::{Cmd, Item, Lsp};
use eframe::egui::{self, pos2, text::{CCursor, CCursorRange}, vec2, Align2, Rect, Sense};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// BASIC-Dialekt, nach dem der Hover lange Namen auf Kurznamen abbildet
enum NameDialect { Cbm(vc_basic::Dialect), Atari(vc_basic::atari::AtariDialect) }
static NAME_DIALECT: std::sync::Mutex<Option<(String, std::sync::Arc<NameDialect>)>> = std::sync::Mutex::new(None);

/// Dialekt (`cbm_v2`, `atari_basic` …) für den Hover über langen Namen wählen; lädt nur bei Wechsel neu
pub fn select_name_dialect(data: &std::path::Path, id: &str) {
    let Ok(mut g) = NAME_DIALECT.lock() else { return };
    if g.as_ref().map_or(false, |(i, _)| i == id) { return; }
    let d = match vc_basic::dialect_family(data, id).as_deref() {
        Ok("atari") => vc_basic::atari::AtariDialect::load(data, id).ok().map(NameDialect::Atari),
        _ => vc_basic::Dialect::load(data, id).ok().map(NameDialect::Cbm),
    };
    *g = d.map(|d| (id.to_string(), std::sync::Arc::new(d)));
}

/// Hover über einem langen Namen (`#longnames`): der Kurzname im gebauten Programm
fn long_name_hover(text: &str, word: &str) -> Option<String> {
    if word.starts_with('@') || !vc_basic::longnames::has_directive(text) { return None; }
    let d = NAME_DIALECT.lock().ok()?.as_ref()?.1.clone();
    let entries = match &*d { NameDialect::Cbm(d) => vc_basic::longnames::lookup(text, vc_basic::longnames::Target::Cbm(d), word), NameDialect::Atari(d) => vc_basic::longnames::lookup(text, vc_basic::longnames::Target::Atari(d), word) };
    if entries.is_empty() { return None; }
    let mut md = format!("**{}** – langer Name, beim Bauen umgesetzt\n\n", word.trim_end_matches(['$', '%']));
    for e in &entries { md.push_str(&format!("- `{}` → `{}`  ({}{})\n", e.long, e.short, e.kind, if e.array { if e.scalar { ", auch als Feld" } else { ", Feld" } } else { "" })); }
    Some(md)
}

/// Zeile beim Verlassen formatieren (Einstellung, Standard an)
pub static PRETTY: AtomicBool = AtomicBool::new(true);

/// Hover über einem Zahlenliteral oder einer Konstante: Ziel (Zeile, Anfang, Ende), Anfrage an den Sprachdienst, Text (Markdown) und Zeit des Beginns
pub struct Hover { pub key: (usize, usize, usize), pub req: u64, pub text: Option<String>, since: f64 }

#[derive(Default)]
pub struct Completion { pub all: Vec<Item>, pub req: u64, pub open: bool, pub sel: usize, pub accept: Option<usize> }

/// Änderung an ganzen Zeilen (0-basierte Zeilenindizes)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineEdit {
    /// `text` als neue Zeile vor Zeile `at` (hinter dem Ende: anhängen)
    Insert { at: usize, text: String },
    /// Zeile `at` von `old` auf `new` setzen, falls sie noch `old` lautet
    Replace { at: usize, old: String, new: String },
    /// Zeile `at` löschen, falls sie noch `old` lautet
    Remove { at: usize, old: String },
}

/// Wendet `e` auf den Text an; liefert den neuen Text, den Zeilenindex der Änderung und den Zeichenindex hinter der Zeile. `None`: nichts zu tun.
pub fn apply_line_edit(text: &str, e: &LineEdit) -> Option<(String, usize, usize)> {
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let at = match e {
        LineEdit::Insert { at, text: t } => {
            let at = if *at >= lines.len() { if lines.last().map_or(true, |l| l.is_empty()) { lines.len().saturating_sub(1) } else { lines.len() } } else { *at };
            lines.insert(at, t.clone()); at
        }
        LineEdit::Replace { at, old, new } => { if lines.get(*at) != Some(old) { return None; } lines[*at] = new.clone(); *at }
        LineEdit::Remove { at, old } => { if lines.get(*at) != Some(old) { return None; } lines.remove(*at); (*at).min(lines.len().saturating_sub(1)) }
    };
    let end: usize = lines[..at].iter().map(|l| l.chars().count() + 1).sum::<usize>() + if matches!(e, LineEdit::Remove { .. }) { 0 } else { lines[at].chars().count() };
    Some((lines.join("\n"), at, end))
}

pub struct Doc {
    pub rel: String,
    pub abs: PathBuf,
    pub text: String,
    pub saved: String,
    pub lang: Lang,
    pub breakpoints: BTreeSet<usize>,
    pub errors: HashMap<usize, String>,
    pub diag: HashMap<usize, String>,
    pub folded: BTreeSet<usize>,
    pub cursor: (usize, usize),
    /// Text, der beim nächsten Zeichnen an der Cursorposition eingefügt wird (ersetzt die Auswahl)
    pub insert: Option<String>,
    /// Zeilenweise Änderung (Include per Drag & Drop): wird beim nächsten Zeichnen als eigener Undo-Schritt angewendet
    pub line_edit: Option<LineEdit>,
    /// Einfügemarke beim Ziehen aus dem Projektbaum: Zeilenindex (0-basiert), vor dem eingefügt würde
    pub drop_at: Option<usize>,
    pub version: i64,
    pub pc_line: Option<usize>,
    pub reveal: Option<usize>,
    /// Zeile, die nach einem Sprung kurz aufleuchtet, und der Zeitpunkt des ersten Zeichnens (0 = noch nicht gezeichnet)
    pub flash: Option<(usize, f64)>,
    pub comp: Completion,
    /// Adressen je Quellzeile (Zeile 0-basiert → Adresse, Länge) vom letzten erfolgreichen Assemblieren, samt dem Text, zu dem sie gehören
    pub addrs: HashMap<usize, (u16, usize)>,
    pub addr_src: Vec<String>,
    /// Vorschau der automatischen BASIC-Zeilennummern (nur natives BASIC ohne Zeilennummern), je Textstand neu berechnet
    pseudo: Vec<Option<u32>>,
    pseudo_hash: u64,
    next_req: u64,
    /// Bearbeitete, noch nicht formatierte Zeile (0-basiert) und die Zeilenzahl vor der Bearbeitung
    edit_line: Option<(usize, usize)>,
    /// Vom Kontextmenü angefordert; die Anwendung führt es aus (Hilfe, Datei öffnen, Sprachdienst, Debugger)
    pub action: Option<DocAction>,
    pub hover: Option<Hover>,
    /// Ergebnisse der Tests mit Quelltextstelle in dieser Datei (Randmarke und Zeilenhinweis), von der Anwendung gesetzt
    pub test_marks: Vec<vc_ide::tests::suite::LineMark>,
    /// Suchleiste, „Gehe zu Zeile“ und eine angeforderte Auswahl (Zeichenbereich), die beim nächsten Zeichnen gesetzt wird
    pub find: crate::editor_find::Find,
    pub goto: crate::editor_find::Goto,
    pub select: Option<(usize, usize)>,
}

/// Aktionen des Kontextmenüs, die über den Editor hinausgehen
#[derive(Clone, Debug, PartialEq)]
pub enum DocAction {
    /// Hilfe/Referenz zu einem Wort
    Help(String),
    /// Datei aus einer `.include`-Zeile öffnen (Pfad wie im Quelltext)
    OpenFile(String),
    /// Zur Definition des Wortes in (Zeile, Spalte) springen (0-basiert)
    Definition { line: usize, col: usize },
    /// Im Debugger bis zu dieser Zeile (0-basiert) ausführen
    RunTo(usize),
    /// Dialog „Zeichen einfügen“ öffnen
    CharPicker,
}

/// Läuft eine Debug-Sitzung? (für „Hierher ausführen“ im Kontextmenü; setzt die Anwendung)
pub static DEBUGGING: AtomicBool = AtomicBool::new(false);

impl Doc {
    pub fn open(root: &std::path::Path, rel: &str) -> Result<Doc, String> {
        let abs = root.join(rel);
        let bytes = std::fs::read(&abs).map_err(|e| format!("{rel}: {e}"))?;
        let text = String::from_utf8(bytes).map_err(|_| format!("{rel}: Binärdatei, wird nicht angezeigt"))?;
        Ok(Doc { rel: rel.into(), abs, saved: text.clone(), text, lang: Lang::of(rel), breakpoints: BTreeSet::new(), errors: HashMap::new(), diag: HashMap::new(), folded: BTreeSet::new(), insert: None, line_edit: None, drop_at: None, cursor: (1, 1), version: 1, pc_line: None, reveal: None, comp: Completion::default(), addrs: HashMap::new(), addr_src: vec![], pseudo: vec![], pseudo_hash: 0, next_req: 1, edit_line: None, action: None, hover: None, test_marks: vec![], flash: None, find: Default::default(), goto: Default::default(), select: None })
    }
    pub fn dirty(&self) -> bool { self.text != self.saved }
    /// Steht der Cursor in einer Zeichenkette einer Textdirektive (`.petscii "…|"`, `.text`, …)? Dann passen `{name}`-Sonderzeichen.
    pub fn in_text_string(&self) -> bool {
        let line = self.text.split('\n').nth(self.cursor.0.saturating_sub(1)).unwrap_or("");
        let before: Vec<char> = line.chars().take(self.cursor.1.saturating_sub(1)).collect();
        let mut in_str = false;
        for (i, c) in before.iter().enumerate() {
            if *c == '"' { in_str = !in_str; }
            else if !in_str && (*c == ';' || (*c == '/' && before.get(i + 1) == Some(&'/'))) { return false; }
        }
        if !in_str { return false; }
        let t = line.trim_start();
        let t = match t.find(':') { Some(i) if t[..i].chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') => t[i + 1..].trim_start(), _ => t };
        let word = t.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
        matches!(word.as_str(), ".petscii" | ".atascii" | ".screen" | ".text" | ".null" | ".ptext" | ".shift")
    }
    /// Ziel aus der ersten `.target`-Zeile des Textes (z. B. `c64`, `atari8`)
    pub fn target_hint(&self) -> Option<String> {
        self.text.lines().find_map(|l| {
            let l = l.split(';').next().unwrap_or("").trim();
            l.strip_prefix(".target").map(|r| r.trim().trim_matches('"').to_ascii_lowercase()).filter(|t| !t.is_empty())
        })
    }
    /// Wort unter dem Cursor (Zeichen, Ziffern, `_`, `.`, `$`)
    pub fn word_at_cursor(&self) -> String {
        let line = self.text.split('\n').nth(self.cursor.0.saturating_sub(1)).unwrap_or(""); let chars: Vec<char> = line.chars().collect();
        let col = (self.cursor.1.saturating_sub(1)).min(chars.len()); let mut a = col; let mut b = col;
        while a > 0 && is_word(chars[a - 1]) { a -= 1; } while b < chars.len() && is_word(chars[b]) { b += 1; }
        chars[a..b].iter().collect()
    }
    pub fn save(&mut self) -> Result<(), String> {
        std::fs::write(&self.abs, &self.text).map_err(|e| format!("{}: {e}", self.rel))?;
        self.saved = self.text.clone(); Ok(())
    }
}

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' || c == '$' }
fn char_to_byte(s: &str, idx: usize) -> usize { s.char_indices().nth(idx).map_or(s.len(), |x| x.0) }

/// Anfang (Zeichenindex) des Wortes vor dem Cursor
fn word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<char> = text.chars().collect(); let mut i = cursor.min(chars.len());
    while i > 0 && is_word(chars[i - 1]) { i -= 1; } i
}

fn filtered<'a>(c: &'a Completion, prefix: &str) -> Vec<&'a Item> {
    let p = prefix.to_ascii_lowercase();
    let mut v: Vec<&Item> = c.all.iter().filter(|i| { let l = i.label.to_ascii_lowercase(); l.starts_with(&p) && l != p }).collect();
    v.dedup_by(|a, b| a.label == b.label); v.truncate(12); v
}

/// Ziel des Hovers in einer Zeile: Zahlenliteral oder Bezeichner (Zeichenindizes Anfang/Ende)
fn hover_target(line: &str, col: usize) -> Option<(usize, usize)> {
    if let Some((s, e, _)) = vc_lsp::literal_at(line, col) { return Some((s, e)); }
    let chars: Vec<char> = line.chars().collect();
    let mut c = col.min(chars.len());
    let id = |ch: char| ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '@';
    if (c >= chars.len() || !id(chars[c])) && c > 0 && id(chars[c - 1]) { c -= 1; }
    if c >= chars.len() || !id(chars[c]) { return None; }
    let (mut s, mut e) = (c, c + 1);
    while s > 0 && id(chars[s - 1]) { s -= 1; }
    while e < chars.len() && id(chars[e]) { e += 1; }
    Some((s, e))
}

/// `const NAME = zahl` aus strukturiertem BASIC (nur Zahlenwerte)
fn basic_consts(text: &str) -> Vec<(String, i64)> {
    text.lines().filter_map(|l| {
        let code = l.split('\'').next().unwrap_or("").trim();
        let rest = code.strip_prefix("const ").or_else(|| code.strip_prefix("CONST "))?;
        let (name, expr) = rest.split_once('=')?;
        let expr = expr.trim();
        let (s, e, v) = vc_lsp::literal_at(expr, 0)?;
        (s == 0 && e == expr.chars().count()).then(|| (name.trim().to_string(), v))
    }).collect()
}

/// Hover ohne Sprachdienst: Zahlenliterale (beide Sprachen) und `const`-Zahlen in BASIC; sonst `None` (der Assembler fragt den Sprachdienst)
fn local_hover(doc: &Doc, k: (usize, usize, usize)) -> Option<String> {
    let line = doc.text.split('\n').nth(k.0)?;
    if let Some((s, e, v)) = vc_lsp::literal_at(line, k.1) { if s == k.1 && e == k.2 { return Some(vc_lsp::value_hover(&line.chars().skip(s).take(e - s).collect::<String>(), v)); } }
    if doc.lang == Lang::Basic {
        let word: String = line.chars().skip(k.1).take(k.2 - k.1).collect();
        if let Some(h) = basic_consts(&doc.text).into_iter().find(|(n, _)| n.eq_ignore_ascii_case(&word)).map(|(n, v)| vc_lsp::value_hover(&n, v)) { return Some(h); }
        return long_name_hover(&doc.text, &word);
    }
    None
}

/// Zeile `l` formatieren; liefert das Zeichenende der alten Zeile und die Längenänderung (für die Cursorkorrektur), falls sich etwas geändert hat
fn format_line(doc: &mut Doc, l: usize, mn: &HashSet<String>) -> Option<(usize, isize)> {
    let lines: Vec<&str> = doc.text.split('\n').collect();
    let old = *lines.get(l)?;
    let new = match doc.lang {
        Lang::Asm => crate::pretty::asm_line(old, mn),
        Lang::Basic if doc.rel.ends_with(".bas") || doc.rel.ends_with(".lst") || doc.rel.ends_with(".tur") => crate::pretty::native_basic_line(old),
        Lang::Basic => crate::pretty::basic_line(&lines, l),
        _ => return None,
    };
    if new == old { return None; }
    let start_b: usize = lines[..l].iter().map(|x| x.len() + 1).sum();
    let start_c = doc.text[..start_b].chars().count();
    let (old_c, new_c) = (old.chars().count(), new.chars().count());
    let old_len = old.len();
    doc.text.replace_range(start_b..start_b + old_len, &new);
    Some((start_c + old_c, new_c as isize - old_c as isize))
}

/// Wortgrenzen um das Zeichen `idx` (Zeichenindizes)
fn word_bounds(chars: &[char], idx: usize) -> (usize, usize) {
    let mut a = idx.min(chars.len()); let mut b = a;
    while a > 0 && is_word(chars[a - 1]) { a -= 1; }
    while b < chars.len() && is_word(chars[b]) { b += 1; }
    (a, b)
}

/// Pfad aus einer `.include "…"`-, `#include "…"`-, `.incbin "…"`- bzw. BASIC-`include "…"`-Zeile samt Zeichenbereich (Zeichenindizes, ohne Anführungszeichen).
/// Systembibliotheken (`#include <…>`) liefern nichts.
pub fn include_span(line: &str) -> Option<(String, usize, usize)> {
    let t = line.trim_start(); let l = t.to_ascii_lowercase();
    let kw = [".include", "#include", "include", ".incbin", "incbin"].into_iter().find(|k| l.starts_with(k))?;
    if !t[kw.len()..].starts_with(|c: char| c.is_whitespace() || c == '"') { return None; }
    let a = t.find('"')?; let b = t[a + 1..].find('"')?;
    let lead = line.chars().count() - t.chars().count();
    let ca = lead + t[..a].chars().count() + 1;
    let path = t[a + 1..a + 1 + b].to_string();
    let n = path.chars().count();
    Some((path, ca, ca + n))
}
fn include_path(line: &str) -> Option<String> { include_span(line).map(|x| x.0) }

fn comment_prefix(rel: &str, lang: Lang) -> &'static str { if lang == Lang::Basic && rel.ends_with(".sbas") { "'" } else { ";" } }

/// Zeilen `a..=b` auskommentieren oder – wenn schon alle (nicht leeren) kommentiert sind – die Kommentarzeichen entfernen
fn toggle_comment(text: &str, a: usize, b: usize, prefix: &str) -> String {
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let b = b.min(lines.len() - 1);
    let all = (a..=b).filter(|i| !lines[*i].trim().is_empty()).all(|i| lines[i].trim_start().starts_with(prefix));
    for i in a..=b {
        let l = lines[i].clone();
        if l.trim().is_empty() { continue; }
        let indent = l.len() - l.trim_start().len();
        let (head, rest) = l.split_at(indent);
        lines[i] = if all { let r = rest.strip_prefix(prefix).unwrap_or(rest); format!("{head}{}", r.strip_prefix(' ').unwrap_or(r)) } else { format!("{head}{prefix} {rest}") };
    }
    lines.join("\n")
}

/// Ordnet Zeilen des aktuellen Textes den Zeilen des Textes zu, zu dem die Adressen gehören (gleicher Anfang und gleiches Ende; dazwischen: unbekannt)
fn map_to_old(old: &[String], cur: &[&str]) -> impl Fn(usize) -> Option<usize> {
    let p = old.iter().zip(cur).take_while(|(a, b)| a.as_str() == **b).count();
    let room = old.len().min(cur.len()) - p;
    let s = old.iter().rev().zip(cur.iter().rev()).take_while(|(a, b)| a.as_str() == **b).count().min(room);
    let (olen, clen) = (old.len() as isize, cur.len() as isize);
    move |i| if i < p { Some(i) } else if i as isize >= clen - s as isize { let j = i as isize + olen - clen; if j >= 0 { Some(j as usize) } else { None } } else { None }
}

/// Texte der Zusatzspalte im Seitenstreifen: Adresse (Assembler) oder automatische Zeilennummer (natives BASIC ohne Zeilennummern)
fn annotations(doc: &mut Doc, n_lines: usize) -> Vec<Option<String>> {
    let mut v: Vec<Option<String>> = vec![None; n_lines];
    match doc.lang {
        Lang::Asm if !doc.addrs.is_empty() => {
            let cur: Vec<&str> = doc.text.split('\n').collect();
            let map = map_to_old(&doc.addr_src, &cur);
            for (i, slot) in v.iter_mut().enumerate() { if let Some((a, _)) = map(i).and_then(|o| doc.addrs.get(&o)) { *slot = Some(format!("${a:04X}")); } }
        }
        Lang::Basic if doc.rel.ends_with(".bas") || doc.rel.ends_with(".lst") || doc.rel.ends_with(".tur") => {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new(); doc.text.hash(&mut h);
            if h.finish() != doc.pseudo_hash || doc.pseudo.len() != n_lines { doc.pseudo = vc_basic::auto_line_numbers(&doc.text, 10, 10); doc.pseudo_hash = h.finish(); }
            for (i, slot) in v.iter_mut().enumerate() { if let Some(Some(n)) = doc.pseudo.get(i) { *slot = Some(n.to_string()); } }
        }
        _ => {}
    }
    v
}

/// Farbe der Testmarke: grün bestanden, rot fehlgeschlagen, bernstein Hinweis/Zeitüberschreitung, grau übersprungen
fn test_mark_color(m: &vc_ide::tests::suite::LineMark) -> egui::Color32 {
    use vc_ide::tests::{Level, State};
    match (m.state, m.level) { (State::Passed, _) => crate::theme::ok(), (State::Failed, Level::Error) => crate::theme::bad(), (State::Failed, Level::Hint) | (State::TimedOut, _) => crate::theme::warn(), (State::Skipped, _) => crate::theme::dim() }
}

/// Such- und Ersetzen-Leiste sowie „Gehe zu Zeile“ über dem Text. Liefert, ob der Text geändert wurde (Ersetzen).
fn find_bar(ui: &mut egui::Ui, doc: &mut Doc, id: egui::Id) -> bool {
    use crate::editor_find as f;
    let ctx = ui.ctx().clone();
    let k = crate::ui::tokens::t();
    let mut changed = false;
    // Auswahl des Textfelds (Zeichenindizes)
    let sel = |ctx: &egui::Context| egui::TextEdit::load_state(ctx, id).and_then(|s| s.cursor.char_range()).map_or((0, 0), |r| (r.primary.index.0.min(r.secondary.index.0), r.primary.index.0.max(r.secondary.index.0)));
    let go = |doc: &mut Doc, r: (usize, usize)| { doc.select = Some(r); let l = f::line_of(&doc.text, r.0); doc.reveal = Some(l - 1); doc.flash = Some((l - 1, 0.0)); };
    if doc.find.open {
        let (mut next, mut prev, mut rep, mut all, mut close, mut typed) = (false, false, false, false, false, false);
        egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.line)).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Suchen");
                let r = ui.add(egui::TextEdit::singleline(&mut doc.find.query).id(id.with("find_q")).desired_width(200.0));
                if doc.find.focus { r.request_focus(); doc.find.focus = false; }
                typed = r.changed();
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { if ui.input(|i| i.modifiers.shift) { prev = true } else { next = true } r.request_focus(); }
                if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) { close = true; }
                if ui.toggle_value(&mut doc.find.case, "Aa").on_hover_text("Groß-/Kleinschreibung beachten").changed() { typed = true; }
                if ui.button("Zurück").clicked() { prev = true; }
                if ui.button("Weiter").clicked() { next = true; }
                if doc.find.replace_mode {
                    ui.separator();
                    ui.label("Ersetzen");
                    ui.add(egui::TextEdit::singleline(&mut doc.find.replace).id(id.with("find_r")).desired_width(160.0));
                    if ui.button("Ersetzen").clicked() { rep = true; }
                    if ui.button("Alle").on_hover_text("Alle Treffer ersetzen").clicked() { all = true; }
                } else if ui.small_button("Ersetzen …").clicked() { doc.find.replace_mode = true; }
                if !doc.find.status.is_empty() { ui.label(egui::RichText::new(&doc.find.status).small().color(k.mute)); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("×").on_hover_text("Schließen (Esc)").clicked() { close = true; } });
            });
        });
        let m = f::find_all(&doc.text, &doc.find.query, doc.find.case);
        let (mut a, mut b) = sel(&ctx);
        if rep { if m.iter().any(|r| *r == (a, b)) { let w = doc.find.replace.clone(); f::replace_range(&mut doc.text, (a, b), &w); changed = true; a += w.chars().count(); b = a; doc.find.status = "1 ersetzt".into(); } next = true; }
        let m = if changed { f::find_all(&doc.text, &doc.find.query, doc.find.case) } else { m };
        if all { let n = f::replace_all(&mut doc.text, &doc.find.query.clone(), &doc.find.replace.clone(), doc.find.case); changed |= n > 0; doc.find.status = format!("{n} ersetzt"); }
        else if typed || next || prev {
            let from = if prev { a } else if typed { a } else { b.max(a) };
            match f::next_match(&m, from, !prev) { Some(r) => { go(doc, r); doc.find.status = format!("{} Treffer", m.len()); } None => doc.find.status = if doc.find.query.is_empty() { String::new() } else { "kein Treffer".into() } }
        }
        if close { doc.find.open = false; ctx.memory_mut(|mm| mm.request_focus(id)); }
    }
    if doc.goto.open {
        let (mut go_line, mut close) = (None, false);
        egui::Frame::new().fill(k.bg1).stroke(egui::Stroke::new(1.0, k.line)).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Gehe zu Zeile");
                let r = ui.add(egui::TextEdit::singleline(&mut doc.goto.input).id(id.with("goto")).desired_width(80.0));
                if doc.goto.focus { r.request_focus(); doc.goto.focus = false; }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { go_line = doc.goto.input.trim().parse::<usize>().ok(); close = true; }
                if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) { close = true; }
                ui.label(egui::RichText::new(format!("von {}", doc.text.split('\n').count())).small().color(k.mute));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.small_button("×").clicked() { close = true; } });
            });
        });
        if let Some(l) = go_line { let (pos, _) = f::line_start(&doc.text, l); go(doc, (pos, pos)); }
        if close { doc.goto.open = false; ctx.memory_mut(|mm| mm.request_focus(id)); }
    }
    changed
}

pub fn show(ui: &mut egui::Ui, doc: &mut Doc, mn: &HashSet<String>, lsp: Option<&Lsp>, uri: &str, show_addr: bool) {
    let ctx = ui.ctx().clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let dim = crate::theme::dim();
    let id = egui::Id::new(("editor", &doc.rel));
    let bar_changed = if doc.find.open || doc.goto.open { find_bar(ui, doc, id) } else { false };
    let fr = fold::ranges(&doc.text, doc.lang);
    doc.folded.retain(|f| fr.contains_key(f));
    let hidden: HashSet<usize> = doc.folded.iter().flat_map(|f| f + 1..=fr[f]).collect();
    let marks: HashMap<usize, usize> = doc.folded.iter().map(|f| (*f, fr[f] - f)).collect();
    let mut errs = doc.diag.clone(); errs.extend(doc.errors.clone());
    let lang = doc.lang;
    // Zusatzspalte (Adressen / Pseudo-Zeilennummern), gedämpft und mit eigener Breite
    let ann = if show_addr { annotations(doc, doc.text.split('\n').count()) } else { vec![] };
    let want_col = show_addr && (ann.iter().any(Option::is_some) || (lang == Lang::Asm));
    let ann_w = if want_col { ui.fonts_mut(|f| f.glyph_width(&font, '0')) * 6.0 + 8.0 } else { 0.0 };
    let gw = ui.fonts_mut(|f| f.glyph_width(&font, '0')) * 3.7 + 34.0 + ann_w;   // Platz für Zeilennummer, Haltepunkt und Faltmarke; wächst mit der Schrift
    let ann_col = dim.gamma_multiply(0.55);

    // Zustand des Textfelds vor dem Zeichnen: Cursor, Vervollständigung bedienen
    let mut state = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
    let cur_idx = state.cursor.char_range().map_or(0, |r| r.primary.index.0);
    let start = word_start(&doc.text, cur_idx);
    let prefix: String = doc.text.chars().skip(start).take(cur_idx.saturating_sub(start)).collect();
    let items: Vec<Item> = if doc.comp.open { filtered(&doc.comp, &prefix).into_iter().cloned().collect() } else { vec![] };
    let popup = doc.comp.open && !items.is_empty() && !prefix.is_empty();
    if popup {
        let (down, up, accept, esc) = ui.input_mut(|i| (i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown), i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab), i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)));
        if down { doc.comp.sel = (doc.comp.sel + 1) % items.len(); }
        if up { doc.comp.sel = (doc.comp.sel + items.len() - 1) % items.len(); }
        if esc { doc.comp.open = false; }
        if accept { doc.comp.accept = Some(doc.comp.sel.min(items.len() - 1)); }
    }
    if let Some(k) = doc.comp.accept.take() {
        if let Some(it) = items.get(k) {
            let (a, z) = (char_to_byte(&doc.text, start), char_to_byte(&doc.text, cur_idx));
            doc.text.replace_range(a..z, &it.label);
            let ni = start + it.label.chars().count();
            state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(ni)))); state.clone().store(&ctx, id);
            doc.version += 1; if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
        }
        doc.comp.open = false;
    }
    if let Some(le) = doc.line_edit.take() {
        if let Some((nt, at, end)) = apply_line_edit(&doc.text, &le) {
            // eigener Undo-Punkt vor der Änderung: Cmd/Strg+Z nimmt genau diese Einfügung zurück
            let before = (state.cursor.char_range().unwrap_or(CCursorRange::one(CCursor::new(cur_idx))), doc.text.clone());
            let mut u = state.undoer(); u.add_undo(&before); state.set_undoer(u);
            doc.text = nt;
            state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(end)))); state.clone().store(&ctx, id);
            doc.reveal = Some(at); if !matches!(le, LineEdit::Remove { .. }) { doc.flash = Some((at, 0.0)); }
            doc.version += 1; if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
        }
    }
    if let Some(ins) = doc.insert.take() {
        let (a, z) = state.cursor.char_range().map(|r| (r.primary.index.0.min(r.secondary.index.0), r.primary.index.0.max(r.secondary.index.0))).unwrap_or((cur_idx, cur_idx));
        let (ba, bz) = (char_to_byte(&doc.text, a), char_to_byte(&doc.text, z));
        doc.text.replace_range(ba..bz, &ins);
        let ni = a + ins.chars().count();
        state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(ni)))); state.clone().store(&ctx, id);
        doc.version += 1; if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
    }
    if bar_changed { doc.version += 1; if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); } }
    if let Some((a, b)) = doc.select.take() { state.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b)))); state.clone().store(&ctx, id); }
    let force = ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));

    let (line_before, count_before) = (doc.cursor.0.saturating_sub(1), doc.text.split('\n').count());
    let mut toggle_fold: Option<usize> = None;
    let mut cursor_pos: Option<egui::Pos2> = None;
    let mut changed = false; let mut new_cursor = cur_idx;
    let mut hover_at: Option<(usize, usize, usize)> = None; let mut menu_edit = false;
    ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, crate::ui::tokens::t().bg0);   // Editorgrund nach Studie: tiefster Ton
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add_space(gw);
            let errors = errs.clone(); let (hid, mk) = (hidden.clone(), marks.clone()); let pcl = doc.pc_line;
            let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _w: f32| {
                let job = highlight::layout(text.as_str(), lang, mn, &errors, font.clone(), &hid, &mk, pcl);
                ui.fonts_mut(|f| f.layout_job(job))
            };
            let out = egui::TextEdit::multiline(&mut doc.text).id(id).font(font.clone()).frame(egui::Frame::NONE).desired_width(f32::INFINITY).desired_rows(1).lock_focus(true).layouter(&mut layouter).show(ui);
            changed = out.response.changed();
            // Ziehen aus dem Projektbaum: Einfügemarke zwischen den Zeilen unter dem Mauszeiger
            doc.drop_at = None;
            if egui::DragAndDrop::has_payload_of_type::<String>(ui.ctx()) {
                if let Some(pp) = ui.ctx().pointer_hover_pos().filter(|p| out.response.rect.expand2(egui::vec2(gw, 0.0)).contains(*p)) {
                    let rows = &out.galley.rows;
                    let y = pp.y - out.galley_pos.y;
                    let i = rows.iter().position(|r| y < r.max_y()).unwrap_or(rows.len().saturating_sub(1));
                    let mut ins = if rows.get(i).map_or(false, |r| y > (r.min_y() + r.max_y()) / 2.0) { i + 1 } else { i };
                    while hidden.contains(&ins) { ins += 1; }
                    let ly = match rows.get(ins) { Some(r) => out.galley_pos.y + r.min_y(), None => out.galley_pos.y + rows.last().map_or(0.0, |r| r.max_y()) };
                    let x0 = out.response.rect.left() - gw + 4.0;
                    ui.painter().line_segment([pos2(x0, ly), pos2(out.response.rect.right().max(x0 + 40.0), ly)], egui::Stroke::new(2.0, crate::theme::warn()));
                    ui.painter().circle_filled(pos2(x0, ly), 3.5, crate::theme::warn());
                    doc.drop_at = Some(ins);
                }
            }
            if let Some(r) = out.cursor_range {
                new_cursor = r.primary.index.0;
                let before: String = doc.text.chars().take(new_cursor).collect();
                doc.cursor = (before.matches('\n').count() + 1, before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1);
                cursor_pos = Some(out.galley_pos + out.galley.pos_from_cursor(r.primary).left_bottom().to_vec2());
            }
            // Doppelklick auf den Dateinamen einer Include-Zeile: Datei öffnen (oder anlegen, falls es sie nicht gibt)
            if out.response.double_clicked() && (lang == Lang::Asm || lang == Lang::Basic) {
                if let Some(pp) = out.response.interact_pointer_pos() {
                    let idx = out.galley.cursor_from_pos(pp - out.galley_pos).index.0;
                    let before: String = doc.text.chars().take(idx).collect();
                    let (ln, col) = (before.matches('\n').count(), before.rsplit('\n').next().map_or(0, |l| l.chars().count()));
                    if let Some((path, a, b)) = doc.text.split('\n').nth(ln).and_then(include_span) {
                        if col >= a && col <= b {
                            let mut st = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
                            let start = idx - (col - a);
                            st.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(start), CCursor::new(start + b - a)))); st.store(&ctx, id);
                            doc.action = Some(DocAction::OpenFile(path));
                        }
                    }
                }
            }
            // --- Kontextmenü (rechte Maustaste): Cursor an die Klickstelle (außer innerhalb einer Auswahl), Menü nur bei geöffnetem Popup berechnen ---
            let pos_id = id.with("ctx_idx");
            if out.response.secondary_clicked() {
                if let Some(pp) = out.response.interact_pointer_pos() {
                    let idx = out.galley.cursor_from_pos(pp - out.galley_pos).index.0;
                    let sel = out.cursor_range.map(|r| (r.primary.index.0.min(r.secondary.index.0), r.primary.index.0.max(r.secondary.index.0)));
                    if !sel.map_or(false, |(a, b)| a != b && idx >= a && idx <= b) {
                        let mut st = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
                        st.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(idx)))); st.store(&ctx, id);
                    }
                    ctx.data_mut(|d| d.insert_temp(pos_id, idx));
                }
            }
            out.response.context_menu(|ui| {
                let chars: Vec<char> = doc.text.chars().collect();
                let click_idx = ctx.data(|d| d.get_temp::<usize>(pos_id)).unwrap_or(0).min(chars.len());
                let st = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
                let (sa, sb) = st.cursor.char_range().map_or((click_idx, click_idx), |r| (r.primary.index.0.min(r.secondary.index.0), r.primary.index.0.max(r.secondary.index.0)));
                let (sa, sb) = (sa.min(chars.len()), sb.min(chars.len()));
                let has_sel = sb > sa;
                let set_cursor = |a: usize, b: usize| { let mut s = egui::TextEdit::load_state(&ctx, id).unwrap_or_default(); s.cursor.set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b)))); s.store(&ctx, id); };
                let line_of = |idx: usize| chars[..idx.min(chars.len())].iter().filter(|c| **c == '\n').count();
                let line_start = |idx: usize| chars[..idx.min(chars.len())].iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
                let (line, col) = (line_of(click_idx), click_idx - line_start(click_idx));
                let (wa, wb) = word_bounds(&chars, click_idx);
                let word: String = chars[wa..wb].iter().collect();
                let line_text: String = doc.text.split('\n').nth(line).unwrap_or("").to_string();
                let asm = lang == Lang::Asm;
                let selected: String = chars[sa..sb].iter().collect();

                if ui.add_enabled(has_sel, egui::Button::new("Ausschneiden")).clicked() {
                    ctx.copy_text(selected.clone());
                    let (ba, bb) = (char_to_byte(&doc.text, sa), char_to_byte(&doc.text, sb));
                    doc.text.replace_range(ba..bb, ""); set_cursor(sa, sa); menu_edit = true; ui.close();
                }
                if ui.add_enabled(has_sel, egui::Button::new("Kopieren")).clicked() { ctx.copy_text(selected.clone()); ui.close(); }
                if ui.button("Einfügen").clicked() { ctx.memory_mut(|m| m.request_focus(id)); ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste); ui.close(); }
                if ui.button("Alles auswählen").clicked() { set_cursor(0, chars.len()); ctx.memory_mut(|m| m.request_focus(id)); ui.close(); }
                if asm || lang == Lang::Basic { ui.separator(); }
                if asm {
                    let has_bp = doc.breakpoints.contains(&line);
                    if ui.button(format!("{} (Zeile {})", if has_bp { "Haltepunkt entfernen" } else { "Haltepunkt setzen" }, line + 1)).clicked() { if !doc.breakpoints.remove(&line) { doc.breakpoints.insert(line); } ui.close(); }
                    if DEBUGGING.load(Ordering::Relaxed) && ui.button("Bis hierher ausführen").clicked() { doc.action = Some(DocAction::RunTo(line)); ui.close(); }
                    if ui.add_enabled(!word.is_empty(), egui::Button::new(if word.is_empty() { "Zur Definition springen".to_string() } else { format!("Zur Definition von »{word}«") })).clicked() { doc.action = Some(DocAction::Definition { line, col }); ui.close(); }
                }
                if asm || lang == Lang::Basic {
                    if ui.add_enabled(!word.is_empty(), egui::Button::new(if word.is_empty() { "Hilfe zum Wort (F1)".to_string() } else { format!("Hilfe zu »{word}« (F1)") })).clicked() { doc.action = Some(DocAction::Help(word.clone())); ui.close(); }
                    if let Some(path) = include_path(&line_text) { if ui.button(format!("Datei öffnen: {path}")).clicked() { doc.action = Some(DocAction::OpenFile(path)); ui.close(); } }
                    ui.separator();
                    if asm && ui.button("Zeichen einfügen …  (Strg+Umschalt+Y)").clicked() { doc.action = Some(DocAction::CharPicker); ui.close(); }
                    let (la, lb) = if has_sel { (line_of(sa), line_of(sb).saturating_sub(if chars.get(sb.wrapping_sub(1)) == Some(&'\n') { 1 } else { 0 })) } else { (line, line) };
                    let multi = lb > la;
                    if ui.button(if multi { "Zeilen auskommentieren / Kommentar entfernen" } else { "Zeile auskommentieren / Kommentar entfernen" }).clicked() {
                        doc.text = toggle_comment(&doc.text, la, lb, comment_prefix(&doc.rel, lang)); menu_edit = true; ui.close();
                    }
                    ui.add_enabled_ui(!fr.is_empty(), |ui| {
                        if ui.button("Alles falten").clicked() { doc.folded = fr.keys().copied().collect(); ui.close(); }
                        if ui.add_enabled(!doc.folded.is_empty(), egui::Button::new("Alles entfalten")).clicked() { doc.folded.clear(); ui.close(); }
                    });
                }
            });
            // Zeiger über dem Text: Zahlenliteral oder Bezeichner darunter bestimmen (für den Wert-Hover)
            if out.response.hovered() {
                if let Some(pt) = ui.input(|i| i.pointer.hover_pos()) {
                    let c = out.galley.cursor_from_pos(pt - out.galley_pos);
                    let idx = c.index.0;
                    let at_end = doc.text.chars().nth(idx).map_or(true, |ch| ch == '\n');
                    if !(at_end && pt.x - out.galley_pos.x > out.galley.pos_from_cursor(c).left() + 10.0) {
                        let before: String = doc.text.chars().take(idx).collect();
                        let (line, col) = (before.matches('\n').count(), before.rsplit('\n').next().map_or(0, |l| l.chars().count()));
                        hover_at = doc.text.split('\n').nth(line).and_then(|l| hover_target(l, col)).map(|(s, e)| (line, s, e));
                    }
                }
            }
            // Seitenstreifen nach dem Text zeichnen (Zeilenhöhen kommen aus dem Galley, gefaltete Zeilen sind unsichtbar)
            let tr = out.response.rect;
            let gut = Rect::from_min_max(pos2(tr.left() - gw, tr.top()), pos2(tr.left(), tr.bottom()));
            let resp = ui.interact(gut, id.with("gutter"), Sense::click());
            let p = ui.painter_at(gut);
            let k = crate::ui::tokens::t();
            // Rinnengrund bis zum unteren Rand des sichtbaren Bereichs, auch bei kurzem Text
            let gut_bg = Rect::from_min_max(gut.min, pos2(gut.right(), gut.bottom().max(ui.clip_rect().bottom())));
            ui.painter_at(gut_bg).rect_filled(gut_bg, 0.0, k.bg2);
            ui.painter_at(gut_bg).line_segment([gut_bg.right_top(), gut_bg.right_bottom()], egui::Stroke::new(1.0, k.line));
            let cur_line = doc.cursor.0.saturating_sub(1);
            let mut y_of: Vec<(f32, f32)> = vec![];
            for row in out.galley.rows.iter() { let (t, b) = (out.galley_pos.y + row.min_y(), out.galley_pos.y + row.max_y()); y_of.push((t, b)); }
            for (i, (t, b)) in y_of.iter().enumerate() {
                if hidden.contains(&i) || b - t < 2.0 { continue; }
                let err = errs.contains_key(&i);
                p.text(pos2(gut.right() - 22.0, *t), Align2::RIGHT_TOP, format!("{}", i + 1), font.clone(), if err { pal_err() } else if i == cur_line { k.mute } else { dim });
                if let Some(Some(s)) = ann.get(i) { p.text(pos2(gut.left() + 20.0 + ann_w - 6.0, *t), Align2::RIGHT_TOP, s, font.clone(), ann_col); }
                if let Some(m) = doc.test_marks.iter().find(|m| m.line == i) {
                    // Randmarke: farbiger Streifen am Textrand; Zeilenhinweis: Kurztext hinter dem Zeilenende (gedämpft)
                    let c = test_mark_color(m);
                    p.rect_filled(Rect::from_min_max(pos2(gut.right() - 3.0, *t), pos2(gut.right() - 0.5, *b)), 1.0, c);
                    if let Some(row) = out.galley.rows.get(i) { ui.painter_at(tr.expand2(egui::vec2(400.0, 0.0))).text(pos2(out.galley_pos.x + row.rect().max.x + 18.0, *t), Align2::LEFT_TOP, &m.text, font.clone(), c.gamma_multiply(0.85)); }
                }
                if doc.breakpoints.contains(&i) { p.circle_filled(pos2(gut.left() + 10.0, (t + b) / 2.0), 4.5, pal_err()); }
                if doc.pc_line == Some(i) {   // aktuelle Debug-Zeile: Cyan-Streifen links, leichter Cyan-Grund über Rinne und Text
                    ui.painter().rect_filled(Rect::from_min_max(pos2(gut.left(), *t), pos2(tr.right(), *b)), 0.0, k.cy.gamma_multiply(0.10));
                    p.rect_filled(Rect::from_min_max(pos2(gut.left(), *t), pos2(gut.left() + 3.0, *b)), 0.0, k.cy);
                }
                if fr.contains_key(&i) { p.text(pos2(gut.right() - 10.0, (t + b) / 2.0), Align2::CENTER_CENTER, if doc.folded.contains(&i) { "▸" } else { "▾" }, font.clone(), dim.gamma_multiply(0.7)); }
            }
            // kurzes Aufleuchten der Zielzeile nach einem Sprung (Gehe zu Definition, Zur Zeile springen): 1,2 s, dann aus
            if let Some((fl, t0)) = doc.flash {
                let now = ui.input(|i| i.time);
                let t0 = if t0 == 0.0 { doc.flash = Some((fl, now)); now } else { t0 };
                let age = (now - t0) as f32;
                match y_of.get(fl) {
                    Some((t, b)) if age < 1.2 => {
                        let a = 0.35 * (1.0 - age / 1.2);
                        ui.painter().rect_filled(Rect::from_min_max(pos2(gut.left(), *t), pos2(tr.right(), *b)), 0.0, crate::theme::accent().gamma_multiply(a));
                        ui.ctx().request_repaint();
                    }
                    _ => doc.flash = None,
                }
            }
            if let Some(l) = doc.reveal.take() { if let Some((t, b)) = y_of.get(l) { ui.scroll_to_rect(Rect::from_min_max(pos2(gut.left(), *t - 40.0), pos2(gut.left() + 10.0, *b + 40.0)), None); } }
            let line_at = |y: f32| y_of.iter().position(|(t, b)| y >= *t && y < *b && b - t >= 2.0);
            if let Some(pt) = resp.interact_pointer_pos().filter(|_| resp.clicked()) {
                if let Some(line) = line_at(pt.y) {
                    if pt.x > gut.right() - 20.0 && fr.contains_key(&line) { toggle_fold = Some(line); }
                    else if !doc.breakpoints.remove(&line) { doc.breakpoints.insert(line); }
                }
            }
            if let Some(pt) = resp.hover_pos() { if let Some(msg) = line_at(pt.y).and_then(|l| errs.get(&l)) {
                egui::Tooltip::always_open(ui.ctx().clone(), ui.layer_id(), egui::Id::new("errtip"), egui::PopupAnchor::Pointer).show(|ui| { ui.colored_label(pal_err(), msg); });
            } }
        });
    });
    if let Some(l) = toggle_fold { if !doc.folded.remove(&l) { doc.folded.insert(l); } }
    if menu_edit {   // Text über das Kontextmenü geändert: wie eine Eingabe behandeln
        doc.folded.clear(); doc.version += 1;
        if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
    }
    if changed {
        if !doc.folded.is_empty() { doc.folded.clear(); }          // Bearbeiten nur im entfalteten Text
        doc.version += 1;
        if let Some(l) = lsp.filter(|_| lang == Lang::Asm) { l.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
        // Vervollständigung bei Wortzeichen anbieten
        let s2 = word_start(&doc.text, new_cursor);
        let typed = new_cursor > s2;
        if typed {
            if !doc.comp.open || new_cursor - s2 == 1 {
                if let Some(l) = lsp { doc.next_req += 1; doc.comp.req = doc.next_req; doc.comp.sel = 0; doc.comp.open = false;
                    l.send(Cmd::Complete { uri: uri.into(), line: doc.cursor.0 - 1, col: doc.cursor.1 - 1, req: doc.comp.req }); }
            }
        } else { doc.comp.open = false; }
    }
    // Wert-Hover: Konstanten und Zahlenliterale in mehreren Schreibweisen
    if changed || popup { doc.hover = None; }
    else if let Some(k) = hover_at {
        let now = ctx.input(|i| i.time);
        if doc.hover.as_ref().map_or(true, |h| h.key != k) {
            let text = local_hover(doc, k);
            let mut h = Hover { key: k, req: 0, text, since: now };
            if h.text.is_none() && lang == Lang::Asm { if let Some(l) = lsp { doc.next_req += 1; h.req = doc.next_req; l.send(Cmd::Hover { uri: uri.into(), line: k.0, col: k.1, req: h.req }); } }
            doc.hover = Some(h);
        }
        if let Some(h) = &doc.hover { if let Some(t) = &h.text {
            if now - h.since >= 0.35 {
                egui::Tooltip::always_open(ctx.clone(), ui.layer_id(), egui::Id::new(("valuehover", &doc.rel)), egui::PopupAnchor::Pointer).show(|ui| { ui.set_max_width(420.0); crate::markdown::render(ui, t); });
            } else { ctx.request_repaint_after(std::time::Duration::from_millis(100)); }
        } }
    } else { doc.hover = None; }
    // Prettyprint: beim Verlassen einer bearbeiteten Zeile (Cursor wechselt die Zeile) wird sie formatiert
    if changed && doc.edit_line.is_none() { doc.edit_line = Some((line_before, count_before)); }
    if let Some((l, cnt)) = doc.edit_line {
        if doc.cursor.0 - 1 != l && !popup {
            doc.edit_line = None;
            let n = doc.text.split('\n').count();
            if PRETTY.load(Ordering::Relaxed) && matches!(lang, Lang::Asm | Lang::Basic) && n >= cnt && n <= cnt + 1 {
                if let Some((end, delta)) = format_line(doc, l, mn) {
                    let mut st = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
                    if let Some(r) = st.cursor.char_range() {
                        let sh = |c: CCursor| if c.index.0 > end { CCursor::new((c.index.0 as isize + delta).max(0) as usize) } else { c };
                        st.cursor.set_char_range(Some(CCursorRange::two(sh(r.secondary), sh(r.primary)))); st.store(&ctx, id);
                    }
                    doc.folded.clear(); doc.version += 1;
                    if let Some(lc) = lsp.filter(|_| lang == Lang::Asm) { lc.send(Cmd::Change { uri: uri.into(), text: doc.text.clone(), version: doc.version }); }
                    ctx.request_repaint();
                }
            }
        }
    }
    if force { if let Some(l) = lsp { doc.next_req += 1; doc.comp.req = doc.next_req; l.send(Cmd::Complete { uri: uri.into(), line: doc.cursor.0 - 1, col: doc.cursor.1 - 1, req: doc.comp.req }); } }
    // Popup
    if popup {
        if let Some(pos) = cursor_pos {
            let sel = doc.comp.sel.min(items.len() - 1); let mut clicked = None;
            egui::Area::new(id.with("complete")).order(egui::Order::Foreground).fixed_pos(pos + vec2(0.0, 2.0)).show(&ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(220.0);
                    for (i, it) in items.iter().enumerate() {
                        let r = ui.horizontal(|ui| { let r = ui.selectable_label(i == sel, egui::RichText::new(&it.label).monospace()); if !it.detail.is_empty() { ui.weak(&it.detail); } r });
                        if r.inner.clicked() { clicked = Some(i); }
                    }
                });
            });
            if clicked.is_some() { doc.comp.accept = clicked; ctx.request_repaint(); }
        }
    }
}

#[cfg(test)]
mod pretty_tests {
    use super::*;

    fn doc(rel: &str, text: &str) -> Doc {
        let dir = std::env::temp_dir().join(format!("vc_ed_{}_{}", std::process::id(), rel.replace('.', "_")));
        std::fs::create_dir_all(&dir).unwrap(); std::fs::write(dir.join(rel), text).unwrap();
        Doc::open(&dir, rel).unwrap()
    }

    #[test]
    fn formats_one_line_and_reports_cursor_shift() {
        let mn: HashSet<String> = ["lda", "sta"].iter().map(|s| s.to_string()).collect();
        let mut d = doc("t.asm", "start:\nLDA   #1\nsta $d020\n");
        let (end, delta) = format_line(&mut d, 1, &mn).unwrap();
        assert_eq!(d.text, "start:\n        lda #1\nsta $d020\n");
        assert_eq!((end, delta), (7 + 8, 14 - 8));      // altes Zeilenende (Zeichenindex) und Längenänderung
        assert!(format_line(&mut d, 1, &mn).is_none(), "zweites Mal keine Änderung");
        assert!(format_line(&mut d, 9, &mn).is_none(), "Zeile außerhalb");
        // Mehrbyte-Zeichen vor der Zeile (Umlaute im Kommentar): Zeichen- statt Byteindex
        let mut d = doc("u.asm", "; schön\nlda #1\n");
        let (end, _) = format_line(&mut d, 1, &mn).unwrap();
        assert_eq!(end, 8 + 6);
        assert_eq!(d.text, "; schön\n        lda #1\n");
    }

    #[test]
    fn basic_files_use_the_right_formatter() {
        let mn = HashSet::new();
        let mut d = doc("p.sbas", "for i=1 to 2\nprint   i\nnext i\n");
        format_line(&mut d, 1, &mn).unwrap();
        assert_eq!(d.text, "for i=1 to 2\n  print i\nnext i\n");
        let mut d = doc("n.bas", "10   print  1\n");
        format_line(&mut d, 0, &mn).unwrap();
        assert_eq!(d.text, "10 print 1\n");
    }
}

#[cfg(test)]
mod menu_tests {
    use super::*;

    #[test]
    fn comment_toggle_adds_and_removes_markers_keeping_indent() {
        let t = "start:  sei\n        lda #1\n\n        sta $d020\n";
        let c = toggle_comment(t, 1, 3, ";");
        assert_eq!(c, "start:  sei\n        ; lda #1\n\n        ; sta $d020\n");
        assert_eq!(toggle_comment(&c, 1, 3, ";"), t, "zweimal umschalten ergibt den Ausgangstext");
        // gemischt (nur eine Zeile kommentiert): alle werden kommentiert
        assert_eq!(toggle_comment("a\n; b\n", 0, 1, ";"), "; a\n; ; b\n");
        assert_eq!(toggle_comment("print 1\n", 0, 5, "'"), "' print 1\n", "Bereich über das Textende hinaus wird begrenzt");
    }

    #[test]
    fn include_path_is_taken_from_include_lines() {
        assert_eq!(include_path("  .include \"lib/music.inc\"  ; Musik").as_deref(), Some("lib/music.inc"));
        assert_eq!(include_path("include \"../x.sbas\"").as_deref(), Some("../x.sbas"));
        assert_eq!(include_path("lda #1"), None);
        assert_eq!(include_path("#include \"a.inc\""), Some("a.inc".into()));
        assert_eq!(include_path("#include <c64/kernal.asm>"), None);
        assert_eq!(include_span("  .incbin \"x.bin\",2"), Some(("x.bin".into(), 11, 16)));
        assert_eq!(include_path(".include broken"), None);
    }

    #[test]
    fn line_edits_insert_replace_remove() {
        use LineEdit::*;
        let t = "a\nb\n";
        let ins = |at: usize| apply_line_edit(t, &Insert { at, text: "X".into() }).unwrap();
        assert_eq!(ins(0), ("X\na\nb\n".into(), 0, 1));
        assert_eq!(ins(1), ("a\nX\nb\n".into(), 1, 3));
        assert_eq!(ins(2).0, "a\nb\nX\n", "vor dem leeren Rest nach dem letzten Zeilenumbruch");
        assert_eq!(ins(99).0, "a\nb\nX\n");
        assert_eq!(apply_line_edit("a\nb", &Insert { at: 5, text: "X".into() }).unwrap().0, "a\nb\nX", "ohne Zeilenumbruch am Ende anhängen");
        assert_eq!(apply_line_edit("", &Insert { at: 0, text: "X".into() }).unwrap().0, "X\n");
        assert_eq!(apply_line_edit(t, &Replace { at: 1, old: "b".into(), new: "Y".into() }).unwrap().0, "a\nY\n");
        assert!(apply_line_edit(t, &Replace { at: 1, old: "q".into(), new: "Y".into() }).is_none(), "geänderte Zeile bleibt unberührt");
        assert_eq!(apply_line_edit(t, &Remove { at: 0, old: "a".into() }).unwrap().0, "b\n");
        assert!(apply_line_edit(t, &Remove { at: 0, old: "b".into() }).is_none());
    }

    #[test]
    fn word_bounds_cover_labels_dots_and_dollars() {
        let c: Vec<char> = "  jsr .loop_1 ; $d020".chars().collect();
        let w = |i: usize| { let (a, b) = word_bounds(&c, i); c[a..b].iter().collect::<String>() };
        assert_eq!(w(7), ".loop_1"); assert_eq!(w(2), "jsr"); assert_eq!(w(16), "$d020"); assert_eq!(w(0), "");
    }

    #[test]
    fn comment_prefix_depends_on_dialect() {
        assert_eq!(comment_prefix("a.asm", Lang::Asm), ";"); assert_eq!(comment_prefix("a.sbas", Lang::Basic), "'"); assert_eq!(comment_prefix("a.bas", Lang::Basic), ";");
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    #[test]
    fn hover_targets_are_literals_or_identifiers() {
        assert_eq!(hover_target("lda #$d020", 7), Some((5, 10)));
        assert_eq!(hover_target("        sta vic2.border", 14), Some((12, 23)), "Bezeichner mit Punkt");
        assert_eq!(hover_target("lda #LIVES", 6), Some((5, 10)));
        assert_eq!(hover_target("lda #LIVES", 4), None, "auf # und Leerzeichen kein Ziel");
        assert_eq!(hover_target("lda #255", 6), Some((5, 8)));
        assert_eq!(hover_target("", 0), None);
    }

    #[test]
    fn basic_constants_are_found_with_their_values() {
        let text = "' Kommentar\nconst PLAYER = 8192\nconst COLORRAM = $d800   ' Farb-RAM\nconst MIX = PLAYER + 1\nconst BITS = %1010\nprint PLAYER";
        let c = basic_consts(text);
        assert_eq!(c, vec![("PLAYER".to_string(), 8192), ("COLORRAM".to_string(), 0xD800), ("BITS".to_string(), 10)], "Ausdrücke bleiben unberücksichtigt");
    }

    fn doc(rel: &str, text: &str) -> Doc {
        let dir = std::env::temp_dir().join(format!("vc_hv_{}_{}", std::process::id(), rel.replace('.', "_")));
        std::fs::create_dir_all(&dir).unwrap(); std::fs::write(dir.join(rel), text).unwrap();
        Doc::open(&dir, rel).unwrap()
    }

    #[test]
    fn local_hover_for_basic_constants_and_literals() {
        let d = doc("h.sbas", "const PLAYER = 8192\npoke PLAYER, $ff\nx = 5");
        let h = local_hover(&d, (1, 5, 11)).unwrap();                      // PLAYER
        assert!(h.starts_with("**PLAYER** = `$2000`") && h.contains("| Dezimal | 8192 |"), "{h}");
        let h = local_hover(&d, (1, 13, 16)).unwrap();                     // $ff
        assert!(h.starts_with("**$ff** = `$FF`") && h.contains("Mit Vorzeichen (8 Bit) | -1"), "{h}");
        assert!(local_hover(&d, (2, 0, 1)).is_none(), "Variable x ist keine Konstante");
        // Assembler: Bezeichner fragt den Sprachdienst (hier kein lokaler Treffer), Literale lokal
        let a = doc("h.asm", "lda #LIVES\nsta $d020");
        assert!(local_hover(&a, (0, 5, 10)).is_none());
        assert!(local_hover(&a, (1, 4, 9)).unwrap().contains("| Dezimal | 53280 |"));
    }
}
