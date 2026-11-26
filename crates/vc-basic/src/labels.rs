// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeilennummern ↔ Labels (Konzept „BASIC-Editor und Dialekte“, Abschnitt 4; Entscheidungen B1–B7 des Inhabers).
//!
//! Reine Funktionen auf Texten, kein Dateizugriff im Umwandlungskern (nur [`load_set`] liest eingebundene Dateien).
//! * **Modus** einer Datei ergibt sich aus dem Inhalt ([`detect_mode`]); Mischung meldet [`check_modes`] als `B160`.
//! * [`to_labels`] (Nummern → Labels): Label nur für angesprungene Zeilen (B5), vorhandene Namen bleiben unverändert (F4),
//!   Namen automatischer Labels aus dem vorangehenden Kommentar, sonst `z<alte Nummer>` (B4), berechnete Ziele brechen mit `B162` ab (B3).
//! * [`to_numbers`] (Labels → Nummern, auch Neu-Nummerieren): immer neu nummeriert nach `start`/`step` (B6); die Namen bleiben als
//!   **Markerzeile** `;@name` unmittelbar vor der Zeile erhalten (B1) und werden bei der Rückumwandlung wieder gefunden.
//! * Umfang: eine Datei samt der Dateien, die sie einbindet (B7); Labels und Nummern gelten dateiübergreifend.
//!
//! Markerzeile: `;@name` (manuell) oder `;@name auto` (automatisch erzeugt). Im Programm kostet sie nichts (`;`-Zeilen werden beim
//! Übersetzen übersprungen). Das Kennzeichen `auto` wird gelesen und im Nummernmodus erhalten; im Labelmodus gibt es dafür keinen Platz,
//! dort gilt jedes Label als manuell (Beispiel Anhang B des Konzepts).
use crate::dialects::{AnyDialect, DialectInfo};
use crate::plain::{include_directive, is_ident, is_ident_start, label_def};
use std::collections::{HashMap, HashSet};
use vc_core::{msg, Diagnostic};

/// Eine Quelldatei (Pfad wie für `#include` und Meldungen, Inhalt)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit { pub file: String, pub text: String }

/// Dialekt und Nummerierung für die Umwandlung
pub struct Cx<'a> {
    pub dialect: &'a AnyDialect,
    pub info: &'a DialectInfo,
    /// erste Nummer und Schritt beim Nummerieren (Projektvorgabe, 10/10)
    pub start: u32,
    pub step: u32,
}

impl<'a> Cx<'a> {
    pub fn new(dialect: &'a AnyDialect, info: &'a DialectInfo) -> Cx<'a> { Cx { dialect, info, start: 10, step: 10 } }
    pub fn numbering(mut self, start: u32, step: u32) -> Self { self.start = start; self.step = step; self }
}

/// Modus einer Quelle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode { Numbers, Labels, Mixed, Undetermined }

/// Herkunft eines Labels (Kennzeichen „manuell“ aus F3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin { Manual, Auto }

/// Ein Label im Ergebnis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelInfo {
    pub name: String,
    pub origin: Origin,
    pub file: String,
    /// Zeile (1-basiert) in der **Quelle**
    pub line: usize,
    /// Zahl der Verweise
    pub refs: usize,
    /// Nummer vor der Umwandlung (falls die Zeile eine hatte) und danach (Nummernmodus)
    pub old_number: Option<u32>,
    pub new_number: Option<u32>,
}

/// Eine geänderte Zeile für die Vorschau (Zeile alt → neu)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChange { pub file: String, pub line: usize, pub old: String, pub new: String }

/// Ergebnis einer Umwandlung: neue Texte aller Dateien (gleiche Reihenfolge wie die Eingabe), Vorschau, Warnungen
#[derive(Debug, Clone)]
pub struct Conversion {
    pub units: Vec<Unit>,
    /// neu erzeugte Namen (Nummern → Labels: automatisch vergeben)
    pub new_labels: Vec<LabelInfo>,
    /// unverändert gebliebene (vorhandene oder gemerkte) Namen
    pub kept_labels: Vec<LabelInfo>,
    pub changes: Vec<LineChange>,
    pub warnings: Vec<Diagnostic>,
}

// ───────────────────────── Zeilen lesen ─────────────────────────

#[derive(Debug, Clone)]
pub(crate) enum K {
    Blank,
    Comment(String),
    Marker { name: String, auto: bool },
    Directive,
    Include(String),
    Prog(Prog),
}

#[derive(Debug, Clone)]
pub(crate) struct Prog {
    /// Leerzeichen am Zeilenanfang (vor der Nummer)
    pub lead: String,
    pub num: Option<u32>,
    /// Leerzeichen zwischen Nummer und Anweisung
    pub gap: String,
    /// Labeldefinitionen `@name:` am Zeilenanfang (kleingeschrieben)
    pub labels: Vec<String>,
    pub body: String,
}

/// `;@name` oder `;@name auto`; `Err(())` für `;@…` mit ungültigem Namen
fn parse_marker(t: &str) -> Option<Result<(String, bool), ()>> {
    let r = t.strip_prefix(";@")?;
    let end = r.find(|c: char| !is_ident(c)).unwrap_or(r.len());
    let name = &r[..end];
    if name.is_empty() || !is_ident_start(name.chars().next().unwrap()) { return Some(Err(())); }
    let rest = r[end..].trim();
    match rest { "" => Some(Ok((name.to_ascii_lowercase(), false))), x if x.eq_ignore_ascii_case("auto") => Some(Ok((name.to_ascii_lowercase(), true))), _ => Some(Err(())) }
}

pub(crate) fn parse_line(raw: &str) -> K {
    let t = raw.trim();
    if t.is_empty() { return K::Blank; }
    if t.starts_with(';') { return match parse_marker(t) { Some(Ok((name, auto))) => K::Marker { name, auto }, _ => K::Comment(t[1..].to_string()) }; }
    if let Some(p) = include_directive(t) { return K::Include(p.to_string()); }
    if t.starts_with('#') { return K::Directive; }
    let lead: String = raw.chars().take_while(|c| c.is_whitespace()).collect();
    let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
    let num = if digits.is_empty() { None } else { Some(digits.parse::<u32>().unwrap_or(u32::MAX)) };
    let after = &t[digits.len()..];
    let mut body = after.trim_start();
    let gap = after[..after.len() - body.len()].to_string();
    let mut labels = vec![];
    while let Some((name, rest)) = label_def(body) { labels.push(name); body = rest; }
    K::Prog(Prog { lead, num, gap, labels, body: body.to_string() })
}

// ───────────────────────── Sprungziele erkennen ─────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum RefKind { Num(u32), Label(String), Computed }

/// Verweis in einer Anweisungszeile: Zeichenbereich `[start, end)` und Art; `word` ist das Sprungwort davor (`GOTO` …)
#[derive(Debug, Clone, PartialEq, Eq)]
struct Ref { start: usize, end: usize, kind: RefKind, word: String }

fn skip_ws(ch: &[char], mut j: usize) -> usize { while j < ch.len() && ch[j] == ' ' { j += 1; } j }

/// Alle Sprungziele einer Zeile (ohne Zeilennummer): Zahlen hinter Sprungwörtern, `@name` überall, berechnete Ziele
fn scan_refs(body: &str, cx: &Cx, whole: bool) -> Vec<Ref> {
    let ch: Vec<char> = body.chars().collect();
    let n = ch.len();
    let lx = cx.dialect.lexicon();
    let atari = cx.dialect.is_atari();
    let is_jump = |w: &str| cx.info.jumps.iter().any(|j| j == w);
    let (mut out, mut i, mut quote) = (vec![], 0usize, false);
    while i < n {
        let c = ch[i];
        if c == '"' { quote = !quote; i += 1; continue; }
        if quote { i += 1; continue; }
        if c == '{' { while i < n && ch[i] != '}' { i += 1; } i = (i + 1).min(n); continue; }
        if c == '@' && ch.get(i + 1).map_or(false, |c| is_ident_start(*c)) {
            let mut j = i + 1;
            while j < n && is_ident(ch[j]) { j += 1; }
            out.push(Ref { start: i, end: j, kind: RefKind::Label(ch[i + 1..j].iter().collect::<String>().to_ascii_lowercase()), word: String::new() });
            i = j; continue;
        }
        if (c == '$' || c == '%') && !(i > 0 && ch[i - 1].is_ascii_alphanumeric()) {
            let mut j = i + 1;
            while j < n && (ch[j].is_ascii_hexdigit() || ch[j] == '_') { j += 1; }
            i = j.max(i + 1); continue;
        }
        if !c.is_ascii_alphabetic() { i += 1; continue; }
        let Some(len) = lx.keyword_at(&ch, i, whole) else { i += 1; continue };
        let mut word: String = ch[i..i + len].iter().collect::<String>().to_ascii_uppercase();
        i += len;
        if word == "REM" { break; }
        if word == "DATA" {
            if atari { break; }
            while i < n && ch[i] != ':' { i += 1; }
            continue;
        }
        if word == "GO" {                                      // `GO TO`
            let j = skip_ws(&ch, i);
            if let Some(l2) = lx.keyword_at(&ch, j, whole) { if ch[j..j + l2].iter().collect::<String>().eq_ignore_ascii_case("TO") { i = j + l2; word = "GOTO".into(); } }
        }
        if !is_jump(&word) { continue; }
        let required = matches!(word.as_str(), "GOTO" | "GOSUB");
        let list = required;
        let mut j = i;
        loop {
            j = skip_ws(&ch, j);
            if j >= n || ch[j] == ':' { break; }
            let c = ch[j];
            let st = j;
            if c.is_ascii_digit() {
                while j < n && ch[j].is_ascii_digit() { j += 1; }
                let value = ch[st..j].iter().collect::<String>().parse::<u32>().unwrap_or(u32::MAX);
                let k = skip_ws(&ch, j);
                let operator = ch.get(k).map_or(false, |c| matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | '.' | '<' | '>' | '='));
                if operator { out.push(Ref { start: st, end: j, kind: RefKind::Computed, word: word.clone() }); j = k; break; }
                out.push(Ref { start: st, end: j, kind: RefKind::Num(value), word: word.clone() });
            } else if c == '@' && ch.get(j + 1).map_or(false, |c| is_ident_start(*c)) {
                break;                                          // wird oben als Label-Verweis erfasst
            } else {
                // Ausdruck statt Zahl
                let alpha = c.is_ascii_alphabetic();
                let kw = alpha && lx.keyword_at(&ch, j, whole).is_some();
                let is_expr = if required { true } else { match word.as_str() { "THEN" | "ELSE" | "RUN" => false, _ => (alpha && !kw) || matches!(c, '(' | '-' | '+') } };
                if !is_expr { break; }
                out.push(Ref { start: st, end: st + 1, kind: RefKind::Computed, word: word.clone() });
                break;
            }
            let k = skip_ws(&ch, j);
            if list && ch.get(k) == Some(&',') { j = k + 1; continue; }
            break;
        }
        i = j.max(i);
    }
    out
}

/// Ersetzt Zeichenbereiche einer Zeile
fn rewrite(body: &str, edits: &mut Vec<(usize, usize, String)>) -> String {
    edits.sort_by_key(|e| e.0);
    let ch: Vec<char> = body.chars().collect();
    let (mut out, mut at) = (String::new(), 0);
    for (s, e, t) in edits.iter() { if *s < at { continue; } out.extend(&ch[at..*s]); out.push_str(t); at = *e; }
    out.extend(&ch[at..]);
    out
}

/// Kommt ein Wort, das zur Laufzeit die Zeilennummer liefert (`EL`), als Name vor?
fn has_runtime_line(body: &str, words: &[String]) -> Option<String> {
    if words.is_empty() { return None; }
    let ch: Vec<char> = body.chars().collect();
    let (mut i, mut quote) = (0, false);
    while i < ch.len() {
        if ch[i] == '"' { quote = !quote; i += 1; continue; }
        if quote || !ch[i].is_ascii_alphabetic() { i += 1; continue; }
        let mut j = i;
        while j < ch.len() && ch[j].is_ascii_alphanumeric() { j += 1; }
        let w: String = ch[i..j].iter().collect::<String>().to_ascii_uppercase();
        if w == "REM" { break; }
        if words.iter().any(|x| *x == w) { return Some(w); }
        i = j;
    }
    None
}

// ───────────────────────── Quellen sammeln ─────────────────────────

/// Liest eine Datei samt der Dateien, die sie einbindet (Pfade wie bei `#include`: relativ zur einbindenden Datei). Die Wurzel steht vorn.
pub fn load_set(path: &str, text: &str) -> Result<Vec<Unit>, Vec<Diagnostic>> { load_set_with(path, text, &crate::include::IncludeHooks::default()) }

/// Wie [`load_set`], aber Medien (`#include "graphics/hero.vcsprite"`) werden über den Provider als eigene Einheit (mit dem Medienpfad) geladen
pub fn load_set_with(path: &str, text: &str, hooks: &crate::include::IncludeHooks) -> Result<Vec<Unit>, Vec<Diagnostic>> { crate::include::load_set(path, text, hooks) }

/// Geordnete Zeilenliste (Einbindungen an ihrer Stelle) und die gelesenen Zeilen je Datei
struct Flat { lines: Vec<Vec<K>>, raws: Vec<Vec<String>>, order: Vec<(usize, usize)> }

fn flatten(units: &[Unit]) -> Flat {
    let raws: Vec<Vec<String>> = units.iter().map(|u| u.text.lines().map(String::from).collect()).collect();
    let lines: Vec<Vec<K>> = raws.iter().map(|r| r.iter().map(|l| parse_line(l)).collect()).collect();
    let mut order = vec![];
    fn go(u: usize, units: &[Unit], lines: &[Vec<K>], order: &mut Vec<(usize, usize)>, seen: &mut Vec<usize>) {
        seen.push(u);
        for (i, k) in lines[u].iter().enumerate() {
            order.push((u, i));
            if let K::Include(p) = k {
                let key = std::path::Path::new(&units[u].file).parent().map(|d| d.to_path_buf()).unwrap_or_default().join(p).to_string_lossy().into_owned();
                if let Some(v) = units.iter().position(|x| x.file == key) { if !seen.contains(&v) { go(v, units, lines, order, seen); } }
            }
        }
        seen.pop();
    }
    if !units.is_empty() { go(0, units, &lines, &mut order, &mut vec![]); }
    Flat { lines, raws, order }
}

fn has_longnames(units: &[Unit]) -> bool { units.iter().any(|u| crate::longnames::has_directive(&u.text)) }

// ───────────────────────── Modus ─────────────────────────

/// Zählt Zeilen mit Nummer und Hinweise auf Labels
fn evidence(units: &[Unit], cx: &Cx) -> (usize, usize) {
    let f = flatten(units);
    let whole = has_longnames(units);
    let (mut nums, mut labs) = (0, 0);
    for &(u, i) in &f.order {
        if let K::Prog(p) = &f.lines[u][i] {
            if p.num.is_some() { nums += 1; }
            if !p.labels.is_empty() || scan_refs(&p.body, cx, whole).iter().any(|r| matches!(r.kind, RefKind::Label(_))) { labs += 1; }
        }
    }
    (nums, labs)
}

/// Modus der Quelle (alle Dateien zusammen): aus dem Inhalt erkannt (Entscheidung B2)
pub fn detect_mode(units: &[Unit], cx: &Cx) -> Mode {
    match evidence(units, cx) { (0, 0) => Mode::Undetermined, (_, 0) => Mode::Numbers, (0, _) => Mode::Labels, _ => Mode::Mixed }
}

/// Modus für Texte ohne Einbindungen
pub fn detect_mode_text(src: &str, cx: &Cx) -> Mode { detect_mode(&[Unit { file: String::new(), text: src.to_string() }], cx) }

/// Prüfung für den Editor: `B160` (gemischt, je Datei), `B166` (Dateien einer Einbindung mit verschiedenem Modus),
/// `B165` (Marker ohne folgende Anweisung), `B168` (ungültiger Marker, doppelter Name)
pub fn check_modes(units: &[Unit], cx: &Cx) -> Vec<Diagnostic> {
    let mut ds = vec![];
    let whole = has_longnames(units);
    let mut modes: Vec<(String, Mode)> = vec![];
    for u in units {
        let one = std::slice::from_ref(u);
        let (nums, labs) = evidence(one, cx);
        let mode = match (nums, labs) { (0, 0) => Mode::Undetermined, (_, 0) => Mode::Numbers, (0, _) => Mode::Labels, _ => Mode::Mixed };
        modes.push((u.file.clone(), mode));
        if mode == Mode::Mixed {
            // erste Zeile, die der zuerst gesehenen Art widerspricht
            let f = flatten(one);
            let mut first: Option<bool> = None;               // true = Nummer, false = Label
            for &(uu, i) in &f.order {
                if let K::Prog(p) = &f.lines[uu][i] {
                    let lab = !p.labels.is_empty() || scan_refs(&p.body, cx, whole).iter().any(|r| matches!(r.kind, RefKind::Label(_)));
                    let num = p.num.is_some();
                    if first.is_none() { first = if num { Some(true) } else if lab { Some(false) } else { None }; }
                    let contradicts = match first { Some(true) => lab, Some(false) => num, None => false };
                    if contradicts || (first == Some(true) && num && lab) { ds.push(Diagnostic::warning("B160", &u.file, i + 1, msg!("diag-basic-b160-mixed-mode", numbers = nums, labels = labs))); break; }
                }
            }
        }
        // Marker
        let f = flatten(one);
        let mut pending: Vec<usize> = vec![];
        let mut seen: HashSet<String> = HashSet::new();
        for &(uu, i) in &f.order {
            match &f.lines[uu][i] {
                K::Marker { name, .. } => {
                    if !seen.insert(name.clone()) { ds.push(Diagnostic::warning("B168", &u.file, i + 1, msg!("diag-basic-b168-marker-invalid", text = f.raws[uu][i].trim()))); }
                    pending.push(i);
                }
                K::Comment(c) if c.starts_with('@') => ds.push(Diagnostic::warning("B168", &u.file, i + 1, msg!("diag-basic-b168-marker-invalid", text = f.raws[uu][i].trim()))),
                K::Prog(p) => { if !p.body.is_empty() { pending.clear(); } if !p.labels.is_empty() { for l in &p.labels { seen.insert(l.clone()); } } }
                _ => {}
            }
        }
        for i in pending { if let K::Marker { name, .. } = &f.lines[0][i] { ds.push(Diagnostic::warning("B165", &u.file, i + 1, msg!("diag-basic-b165-marker-orphan", name = name.as_str()))); } }
    }
    let distinct: Vec<&(String, Mode)> = modes.iter().filter(|(_, m)| matches!(m, Mode::Numbers | Mode::Labels)).collect();
    if distinct.len() > 1 && distinct.iter().any(|(_, m)| *m != distinct[0].1) {
        for (f, m) in &distinct { ds.push(Diagnostic::error("B166", f, 1, msg!("diag-basic-b166-include-mode", file = f.as_str(), mode = if *m == Mode::Numbers { "numbers" } else { "labels" }))); }
    }
    ds
}

// ───────────────────────── gemeinsame Auswertung ─────────────────────────

/// Eine Programmzeile mit Anweisung (Träger-Zeilen, die nur Labels enthalten, sind in `carriers` und `names` aufgenommen)
struct Item {
    pos: (usize, usize),
    prog: Prog,
    /// Namen vor dieser Anweisung: Marker, Träger-Labels, eigene Labels (Reihenfolge der Quelle)
    names: Vec<(String, Origin)>,
    /// Quellzeilen, die entfallen (Marker, Träger)
    consumed: Vec<(usize, usize)>,
    /// Nummern von Trägerzeilen (`10 @a:`), die auf diese Anweisung zeigen
    carrier_nums: Vec<u32>,
}

struct Model { flat: Flat, items: Vec<Item>, orphans: Vec<(usize, usize, String)>, errors: Vec<Diagnostic>, all_names: HashSet<String> }

fn build(units: &[Unit]) -> Model {
    let flat = flatten(units);
    let (mut items, mut errors): (Vec<Item>, Vec<Diagnostic>) = (vec![], vec![]);
    let mut pend_names: Vec<(String, Origin)> = vec![];
    let mut pend_consumed: Vec<(usize, usize)> = vec![];
    let mut pend_nums: Vec<u32> = vec![];
    let mut pend_marker_lines: Vec<(usize, usize, String)> = vec![];
    let mut pend_carrier_lines: Vec<(usize, usize, String)> = vec![];
    let mut all: HashSet<String> = HashSet::new();
    for &(u, i) in &flat.order {
        match &flat.lines[u][i] {
            K::Marker { name, auto } => {
                all.insert(name.clone());
                pend_names.push((name.clone(), if *auto { Origin::Auto } else { Origin::Manual }));
                pend_consumed.push((u, i));
                pend_marker_lines.push((u, i, name.clone()));
            }
            K::Prog(p) if p.body.is_empty() && !p.labels.is_empty() => {
                for l in &p.labels { all.insert(l.clone()); pend_names.push((l.clone(), Origin::Manual)); pend_carrier_lines.push((u, i, l.clone())); }
                pend_consumed.push((u, i));
                if let Some(n) = p.num { pend_nums.push(n); }
            }
            K::Prog(p) if !p.body.is_empty() || p.num.is_some() => {
                for l in &p.labels { all.insert(l.clone()); }
                let mut names = std::mem::take(&mut pend_names);
                // eigene Labels: stehen sie schon als Marker da, bleibt der Marker maßgeblich (Kennzeichen `auto`)
                for l in &p.labels { if let Some(e) = names.iter().position(|(n, _)| n == l) { let _ = e; } else { names.push((l.clone(), Origin::Manual)); } }
                items.push(Item { pos: (u, i), prog: p.clone(), names, consumed: std::mem::take(&mut pend_consumed), carrier_nums: std::mem::take(&mut pend_nums) });
                pend_marker_lines.clear(); pend_carrier_lines.clear();
            }
            _ => {}
        }
    }
    // übrig: Marker ohne Anweisung (verwaist, bleiben stehen), Labels ohne Anweisung (B112)
    let orphans: Vec<(usize, usize, String)> = pend_marker_lines;
    for (u, i, l) in pend_carrier_lines { errors.push(Diagnostic::error("B112", &units[u].file, i + 1, msg!("diag-basic-b112-label-at-end", name = l.as_str()))); }
    Model { flat, items, orphans, errors, all_names: all }
}

fn join_lines(old: &str, lines: Vec<String>) -> String {
    let nl = if old.contains("\r\n") { "\r\n" } else { "\n" };
    let mut t = lines.join(nl);
    if old.ends_with('\n') || old.is_empty() { t.push_str(nl); }
    t
}

/// Schreibt die Ersetzungen (Quellzeile → null oder mehr neue Zeilen) in die Dateien zurück und bildet die Änderungsliste
fn assemble(units: &[Unit], flat: &Flat, repl: &HashMap<(usize, usize), Vec<String>>) -> (Vec<Unit>, Vec<LineChange>) {
    let (mut out, mut changes) = (vec![], vec![]);
    for (u, unit) in units.iter().enumerate() {
        let mut lines = vec![];
        for (i, raw) in flat.raws[u].iter().enumerate() {
            match repl.get(&(u, i)) {
                None => lines.push(raw.clone()),
                Some(new) => {
                    if new.len() != 1 || new[0] != *raw { changes.push(LineChange { file: unit.file.clone(), line: i + 1, old: raw.clone(), new: new.join("\n") }); }
                    lines.extend(new.iter().cloned());
                }
            }
        }
        out.push(Unit { file: unit.file.clone(), text: join_lines(&unit.text, lines) });
    }
    (out, changes)
}

/// Wort für einen Namen aus Kommentartext (Entscheidung B4); `None`, wenn nichts Brauchbares bleibt
fn name_from_comment(text: &str) -> Option<String> {
    let mut s = String::new();
    for c in text.trim().chars() {
        match c {
            'ä' | 'Ä' => s.push_str("ae"), 'ö' | 'Ö' => s.push_str("oe"), 'ü' | 'Ü' => s.push_str("ue"), 'ß' => s.push_str("ss"),
            c if c.is_ascii_alphanumeric() => s.push(c.to_ascii_lowercase()),
            _ => if !s.ends_with('_') { s.push('_'); },
        }
    }
    let parts: Vec<&str> = s.split('_').filter(|p| !p.is_empty()).take(4).collect();
    let mut name = parts.join("_");
    if name.len() > 24 { name.truncate(24); name = name.trim_end_matches('_').to_string(); }
    if name.is_empty() || !name.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_') { return None; }
    Some(name)
}

fn unique(base: &str, used: &mut HashSet<String>) -> String {
    let mut n = base.to_string();
    let mut k = 2;
    while used.contains(&n) { n = format!("{base}_{k}"); k += 1; }
    used.insert(n.clone());
    n
}

// ───────────────────────── Nummern → Labels ─────────────────────────

/// Wandelt Zeilennummern in Labels um (siehe Moduldoku). Quelle: alle Dateien einer Einbindung (Wurzel zuerst).
/// Fehler: berechnete Sprungziele (`B162` je Stelle, dazu `B161`), undefinierte oder doppelte Labels (`B110`/`B111`).
pub fn to_labels(units: &[Unit], cx: &Cx) -> Result<Conversion, Vec<Diagnostic>> {
    let m = build(units);
    let whole = has_longnames(units);
    let mut errs = m.errors.clone();
    let mut warns: Vec<Diagnostic> = vec![];
    // Verweise sammeln
    let mut targets: HashMap<u32, usize> = HashMap::new();
    let mut refs_of: Vec<Vec<Ref>> = vec![];
    for it in &m.items {
        let refs = scan_refs(&it.prog.body, cx, whole);
        for r in &refs {
            match &r.kind {
                RefKind::Num(v) => { *targets.entry(*v).or_default() += 1; }
                RefKind::Computed => errs.push(Diagnostic::error("B162", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b162-computed-target", word = r.word.as_str(), text = it.prog.body.trim()))),
                RefKind::Label(_) => {}
            }
        }
        if let Some(w) = has_runtime_line(&it.prog.body, &cx.info.runtime_line) { warns.push(Diagnostic::warning("B163", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b163-runtime-line", word = w.as_str()))); }
        refs_of.push(refs);
    }
    if errs.iter().any(|e| e.code == "B162") {
        let n = errs.iter().filter(|e| e.code == "B162").count();
        errs.insert(0, Diagnostic::error("B161", &units[0].file, 1, msg!("diag-basic-b161-convert-failed", count = n)));
    }
    // vorhandene Namen: Doppelte melden
    let mut seen: HashMap<String, (usize, usize)> = HashMap::new();
    for it in &m.items { for (n, _) in &it.names { if seen.insert(n.clone(), it.pos).is_some() { errs.push(Diagnostic::error("B111", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b111-label-duplicate", name = n.as_str()))); } } }
    if !errs.is_empty() { return Err(errs); }
    // Namen vergeben (Reihenfolge der Quelle)
    let mut used = m.all_names.clone();
    let mut primary: HashMap<u32, String> = HashMap::new();           // Nummer → Name
    let mut item_names: Vec<Vec<(String, Origin)>> = vec![];
    let mut new_labels = vec![];
    let mut kept = vec![];
    let order_index: HashMap<(usize, usize), usize> = m.flat.order.iter().enumerate().map(|(k, p)| (*p, k)).collect();
    for it in m.items.iter() {
        let mut names = it.names.clone();
        let num = it.prog.num;
        let mut made_auto = false;
        if names.is_empty() {
            if let Some(nu) = num { if targets.contains_key(&nu) {
                // Namen aus dem unmittelbar vorangehenden Kommentar (`;` oder `REM`), sonst z<Nummer>
                let prev = order_index[&it.pos].checked_sub(1).map(|k| m.flat.order[k]);
                let from = prev.and_then(|(u, i)| match &m.flat.lines[u][i] {
                    K::Comment(c) => name_from_comment(c),
                    K::Prog(p) => { let b = p.body.trim_start(); if b.get(..3).map_or(false, |h| h.eq_ignore_ascii_case("rem")) { name_from_comment(&b[3..]) } else { None } }
                    _ => None,
                });
                let base = from.unwrap_or_else(|| format!("z{nu}"));
                names.push((unique(&base, &mut used), Origin::Auto));
                made_auto = true;
            } }
        }
        if let (Some(nu), Some((n, _))) = (num, names.first()) { primary.insert(nu, n.clone()); }
        for cn in &it.carrier_nums { if let Some((n, _)) = names.first() { primary.insert(*cn, n.clone()); } }
        for (n, o) in &names {
            let info = LabelInfo { name: n.clone(), origin: *o, file: units[it.pos.0].file.clone(), line: it.pos.1 + 1, refs: 0, old_number: num, new_number: None };
            if made_auto { new_labels.push(info); } else { kept.push(info); }
        }
        item_names.push(names);
    }
    // Zeilen schreiben
    let mut repl: HashMap<(usize, usize), Vec<String>> = HashMap::new();
    for (idx, it) in m.items.iter().enumerate() {
        for c in &it.consumed { repl.insert(*c, vec![]); }
        let mut edits: Vec<(usize, usize, String)> = vec![];
        for r in &refs_of[idx] {
            if let RefKind::Num(v) = &r.kind {
                match primary.get(v) { Some(n) => edits.push((r.start, r.end, format!("@{n}"))), None => warns.push(Diagnostic::warning("B153", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b153-target-missing", num = *v))) }
            }
        }
        let body = rewrite(&it.prog.body, &mut edits);
        let indent: String = it.prog.gap.chars().skip(1).collect();
        let labels: String = item_names[idx].iter().map(|(n, _)| format!("@{n}: ")).collect();
        // Nur Zeilen mit Nummer, Label oder geändertem Verweis ändern sich
        let changed = it.prog.num.is_some() || !labels.is_empty() || body != it.prog.body;
        if changed { repl.insert(it.pos, vec![format!("{}{}{}{}", it.prog.lead, indent, labels, body)]); }
    }
    for l in new_labels.iter_mut().chain(kept.iter_mut()) {
        l.refs = l.old_number.and_then(|n| targets.get(&n).copied()).unwrap_or(0);
    }
    for (u, i, n) in &m.orphans { warns.push(Diagnostic::warning("B165", &units[*u].file, i + 1, msg!("diag-basic-b165-marker-orphan", name = n.as_str()))); }
    let (out, changes) = assemble(units, &m.flat, &repl);
    Ok(Conversion { units: out, new_labels, kept_labels: kept, changes, warnings: warns })
}

// ───────────────────────── Labels → Nummern, Neu-Nummerieren ─────────────────────────

/// Bereich für das teilweise Neu-Nummerieren (alte Nummern von bis, einschließlich)
pub type RenumRange = Option<(u32, u32)>;

/// Nummeriert neu (alle Anweisungszeilen nach `cx.start`/`cx.step`, Entscheidung B6), löst `@name` zu Nummern auf, passt alle Sprungziele an
/// und schreibt die Namen als Markerzeilen `;@name` vor die Zeilen. Wirkt auf Labelmodus (Umwandlung) und Nummernmodus (Neu-Nummerieren).
/// Mit `range` werden nur die Zeilen mit alter Nummer in diesem Bereich neu nummeriert (nur Nummernmodus; die Reihenfolge muss erhalten bleiben).
/// Berechnete Sprungziele bleiben stehen und werden als Warnung `B162` gemeldet (sie können danach falsch zeigen).
pub fn to_numbers(units: &[Unit], cx: &Cx, range: RenumRange) -> Result<Conversion, Vec<Diagnostic>> {
    let m = build(units);
    let whole = has_longnames(units);
    let mut errs = m.errors.clone();
    let mut warns: Vec<Diagnostic> = vec![];
    let maxn = cx.info.max_line_number;
    // neue Nummern
    let mut new_num: Vec<u32> = vec![];
    if let Some((from, to)) = range {
        if m.items.iter().any(|it| it.prog.num.is_none()) {
            return Err(vec![Diagnostic::error("B161", &units[0].file, 1, msg!("diag-basic-b161-range-labels"))]);
        }
        let mut k = 0u32;
        for it in &m.items {
            let old = it.prog.num.unwrap();
            if old >= from && old <= to { new_num.push(cx.start.saturating_add(k.saturating_mul(cx.step))); k += 1; } else { new_num.push(old); }
        }
        let mut last: Option<u32> = None;
        for (idx, n) in new_num.iter().enumerate() {
            if let Some(l) = last { if *n <= l { let it = &m.items[idx]; return Err(vec![Diagnostic::error("B161", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b161-range-order", num = *n, last = l))]); } }
            last = Some(*n);
        }
    } else {
        for k in 0..m.items.len() as u32 { new_num.push(cx.start.saturating_add(k.saturating_mul(cx.step))); }
    }
    for (idx, n) in new_num.iter().enumerate() {
        if *n > maxn {
            let it = &m.items[idx];
            let (cnt, st) = if range.is_some() { (idx + 1, cx.start) } else { (m.items.len(), cx.start) };
            let d = crate::numbering::Numbering::new(st, cx.step).check_fit(cnt, maxn, &units[it.pos.0].file, it.pos.1 + 1).err()
                .unwrap_or_else(|| Diagnostic::error("B100", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b100-line-number-auto-max", num = *n, max = maxn)));
            errs.push(d); break;
        }
    }
    // Zuordnungen
    let mut by_label: HashMap<String, u32> = HashMap::new();
    let mut by_old: HashMap<u32, u32> = HashMap::new();
    for (idx, it) in m.items.iter().enumerate() {
        for (n, _) in &it.names { if by_label.insert(n.clone(), new_num[idx]).is_some() { errs.push(Diagnostic::error("B111", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b111-label-duplicate", name = n.as_str()))); } }
        if let Some(o) = it.prog.num { if range.map_or(true, |(a, b)| o >= a && o <= b) { by_old.insert(o, new_num[idx]); } }
        for c in &it.carrier_nums { by_old.insert(*c, new_num[idx]); }
    }
    let mut refs_of: Vec<Vec<Ref>> = vec![];
    for it in m.items.iter() {
        let refs = scan_refs(&it.prog.body, cx, whole);
        for r in &refs {
            match &r.kind {
                RefKind::Label(n) if !by_label.contains_key(n) => errs.push(Diagnostic::error("B110", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b110-label-undefined", name = n.as_str()))),
                RefKind::Computed => warns.push(Diagnostic::warning("B162", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b162-computed-target", word = r.word.as_str(), text = it.prog.body.trim()))),
                _ => {}
            }
        }
        if has_runtime_line(&it.prog.body, &cx.info.runtime_line).is_some() && it.prog.num.is_some() {
            warns.push(Diagnostic::warning("B163", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b163-runtime-line", word = has_runtime_line(&it.prog.body, &cx.info.runtime_line).unwrap().as_str())));
        }
        refs_of.push(refs);
    }
    if !errs.is_empty() { return Err(errs); }
    let mut label_refs: HashMap<String, usize> = HashMap::new();
    let mut repl: HashMap<(usize, usize), Vec<String>> = HashMap::new();
    for (idx, it) in m.items.iter().enumerate() {
        for c in &it.consumed { repl.insert(*c, vec![]); }
        let mut edits: Vec<(usize, usize, String)> = vec![];
        for r in &refs_of[idx] {
            match &r.kind {
                RefKind::Label(n) => { *label_refs.entry(n.clone()).or_default() += 1; edits.push((r.start, r.end, by_label[n].to_string())); }
                RefKind::Num(v) => match by_old.get(v) {
                    Some(nn) => edits.push((r.start, r.end, nn.to_string())),
                    None => if range.is_none() && *v <= maxn { warns.push(Diagnostic::warning("B153", &units[it.pos.0].file, it.pos.1 + 1, msg!("diag-basic-b153-target-missing", num = *v))); },
                },
                RefKind::Computed => {}
            }
        }
        let body = rewrite(&it.prog.body, &mut edits);
        let mut new_lines: Vec<String> = it.names.iter().map(|(n, o)| if *o == Origin::Auto { format!(";@{n} auto") } else { format!(";@{n}") }).collect();
        new_lines.push(format!("{} {}{}", new_num[idx], it.prog.lead, body));
        repl.insert(it.pos, new_lines);
    }
    let (mut kept, new_labels) = (vec![], vec![]);
    for (idx, it) in m.items.iter().enumerate() {
        for (n, o) in &it.names { kept.push(LabelInfo { name: n.clone(), origin: *o, file: units[it.pos.0].file.clone(), line: it.pos.1 + 1, refs: label_refs.get(n).copied().unwrap_or(0), old_number: it.prog.num, new_number: Some(new_num[idx]) }); }
    }
    for (u, i, n) in &m.orphans { warns.push(Diagnostic::warning("B165", &units[*u].file, i + 1, msg!("diag-basic-b165-marker-orphan", name = n.as_str()))); }
    let (out, changes) = assemble(units, &m.flat, &repl);
    Ok(Conversion { units: out, new_labels, kept_labels: kept, changes, warnings: warns })
}

/// Neu-Nummerieren im Nummernmodus (Start und Schritt aus `cx`); Kurzform für [`to_numbers`]
pub fn renumber(units: &[Unit], cx: &Cx, range: RenumRange) -> Result<Conversion, Vec<Diagnostic>> { to_numbers(units, cx, range) }

/// Umwandlung eines einzelnen Textes (Einbindungen bleiben unberührt und zählen nicht mit)
pub fn to_labels_text(src: &str, cx: &Cx) -> Result<String, Vec<Diagnostic>> {
    to_labels(&[Unit { file: String::new(), text: src.to_string() }], cx).map(|mut c| c.units.remove(0).text)
}
pub fn to_numbers_text(src: &str, cx: &Cx) -> Result<String, Vec<Diagnostic>> {
    to_numbers(&[Unit { file: String::new(), text: src.to_string() }], cx, None).map(|mut c| c.units.remove(0).text)
}
