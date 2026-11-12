// SPDX-License-Identifier: GPL-3.0-or-later
//! Lange Variablennamen im nativen BASIC (`.bas`).
//!
//! Diese Funktion ist Birko vom N3rdroom gewidmet.
//!
//! Steht in der Quelle eine Zeile `#longnames`, sind Namen aus Buchstaben, Ziffern und `_` beliebig lang. Beim Bauen werden sie
//! je Dialekt auf Kurznamen abgebildet, wenn der Dialekt nur die ersten Zeichen unterscheidet (CBM: 2 Zeichen, `varSigChars`):
//!
//! - Ein Wort ist immer ein ganzer Name: entweder genau ein Schlüsselwort des Dialekts (`TOTAL`, `FORM`, `SCORE` sind Namen,
//!   `FOR`, `TO` Schlüsselwörter) oder eine Variable. Schlüsselwörter dürfen deshalb nicht mehr an Namen oder Zahlen kleben (`FORI=1TO10`).
//! - Kurze Namen (höchstens `varSigChars` Zeichen, ohne `_`) bleiben unverändert und sind belegt.
//! - Ein langer Name bekommt zuerst seine ersten Zeichen (ohne `_`) – so entsteht dasselbe Programm wie im klassischen BASIC –,
//!   dann andere Namen mit gleichem Anfangsbuchstaben, dann beliebige. Namen, die ein Schlüsselwort ergäben oder enthielten
//!   (`TO`, `OR`, `IF`, `GO`, `FN` …) oder reserviert sind (`ST`, `TI`, `DS`, `ER`, `EL`), fallen weg.
//! - Zahl, Ganzzahl (`%`) und Text (`$`) sowie Felder `name(` teilen den Kurznamen der Grundform, bleiben aber getrennte Variablen,
//!   wie im BASIC üblich. Funktionsnamen (`DEF FN name(x)`) haben einen eigenen Namensraum.
//! - Strings, `REM`, `DATA`, `{…}`-Zeichen, Zahlen (auch `$ff`, `%101`) und `@label` bleiben unberührt.
//!
//! Atari BASIC / Turbo-BASIC XL kennen lange Namen (bis 120 Zeichen): sie bleiben unverändert, nur `_` fällt weg.
use crate::atari::AtariDialect;
use crate::classic::match_keyword;
use crate::dialect::Dialect;
use std::collections::{BTreeMap, HashMap, HashSet};
use vc_core::Diagnostic;

/// Zeile, die die langen Namen einschaltet
pub const DIRECTIVE: &str = "#longnames";

/// Ist `line` die Zeile `#longnames`?
pub fn is_directive(line: &str) -> bool { line.trim().eq_ignore_ascii_case(DIRECTIVE) }

/// Schaltet die Quelle lange Namen ein?
pub fn has_directive(src: &str) -> bool { src.lines().any(is_directive) }

/// Für welchen Dialekt wird umgesetzt
#[derive(Clone, Copy)]
pub enum Target<'a> { Cbm(&'a Dialect), Atari(&'a AtariDialect) }

impl Target<'_> {
    fn sig(&self) -> usize { match self { Target::Cbm(d) => d.var_sig_chars, Target::Atari(_) => 120 } }
    fn has_fn(&self) -> bool { matches!(self, Target::Cbm(d) if d.token_bytes("FN").is_some()) }
    /// Ist das (großgeschriebene) Wort samt folgendem Zeichen ein Schlüsselwort?
    fn keyword(&self, up: &str, next: Option<char>) -> bool {
        match self {
            Target::Cbm(d) => d.token_bytes(up).is_some()
                || next.map_or(false, |n| matches!(n, '$' | '(' | '#') && d.token_bytes(&format!("{up}{n}")).is_some()),
            Target::Atari(_) => false,
        }
    }
    /// Darf `name` (groß) als Kurzname dienen? Keine Schlüsselwörter, keine reservierten Variablen.
    fn usable(&self, name: &str) -> bool {
        let c: Vec<char> = name.chars().collect();
        if c.is_empty() || c.len() > self.sig() || !c[0].is_ascii_alphabetic() || !c.iter().all(char::is_ascii_alphanumeric) { return false; }
        match self {
            Target::Cbm(d) => {
                if d.is_reserved_var(name) || ["IF", "ON", "GO", "TO", "OR", "FN"].contains(&name) { return false; }
                // kein Schlüsselwort darf im Namen beginnen (auch nicht mit Typzeichen oder Klammer dahinter): so tokenisiert das ROM
                ["", "$", "%"].iter().all(|suf| {
                    let t: Vec<char> = format!("{name}{suf}(").chars().collect();
                    (0..c.len()).all(|i| match_keyword(&t, i, d).is_none())
                })
            }
            Target::Atari(d) => !d.statements.iter().chain(&d.functions).any(|(k, _)| k.eq_ignore_ascii_case(name)),
        }
    }
    fn data_to_eol(&self) -> bool { matches!(self, Target::Atari(_)) }
}

/// Ein umgesetzter Name
#[derive(Debug, Clone, PartialEq)]
pub struct NameEntry {
    /// wie im Quelltext (kleingeschrieben, mit `$`/`%`)
    pub long: String,
    /// Kurzname im gebauten Programm (großgeschrieben, mit `$`/`%`)
    pub short: String,
    /// `Zahl`, `Ganzzahl`, `Text` oder `Funktion`
    pub kind: &'static str,
    pub scalar: bool,
    pub array: bool,
}

impl NameEntry {
    /// Zeile der Zuordnungstabelle, z. B. `feld(…)  →  FE(…)  (Zahl, Feld)`
    pub fn line(&self) -> String {
        let form = |n: &str| match (self.scalar, self.array) { (_, true) if !self.scalar => format!("{n}(…)"), (true, true) => format!("{n}, {n}(…)"), _ => n.to_string() };
        let what = if self.array && !self.scalar { format!("{}, Feld", self.kind) } else { self.kind.to_string() };
        format!("{:<26} →  {:<10} ({what})", form(&self.long), form(&self.short))
    }
}

/// Zuordnungstabelle als Text (eine Zeile je Name)
pub fn table_text(names: &[NameEntry]) -> String {
    names.iter().map(|n| n.line() + "\n").collect()
}

#[derive(Debug, Clone)]
pub struct Renamed {
    pub text: String,
    pub names: Vec<NameEntry>,
    pub warnings: Vec<Diagnostic>,
}

enum Seg { Lit(String), Name { func: bool, key: String, suffix: String } }

fn is_word(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

/// Setzt alle langen Namen im nummerierten (oder rohen) Text um.
pub fn rename(text: &str, t: Target, file: &str) -> Result<Renamed, Vec<Diagnostic>> {
    let sig = t.sig();
    let mut segs: Vec<Seg> = vec![];
    // Reihenfolge des ersten Auftretens: (Funktion?, Name klein) → erste Zeile
    let mut order: Vec<((bool, String), usize)> = vec![];
    let mut seen: HashSet<(bool, String)> = HashSet::new();
    let mut kinds: BTreeMap<(bool, String, String), (bool, bool)> = BTreeMap::new();   // (func, key, suffix) → (skalar, Feld)
    let mut lower_style: HashSet<(bool, String)> = HashSet::new();   // Atari: Name im Quelltext kleingeschrieben
    let (mut lit_var, mut lit_fn): (HashSet<String>, HashSet<String>) = (HashSet::new(), HashSet::new());
    for (ln, line) in text.split('\n').enumerate() {
        if ln > 0 { segs.push(Seg::Lit("\n".into())); }
        let tr = line.trim_start();
        if tr.starts_with(';') || tr.starts_with('#') { segs.push(Seg::Lit(line.to_string())); continue; }
        let ch: Vec<char> = line.chars().collect();
        let (mut i, mut quote, mut in_data, mut prev_fn) = (0usize, false, false, false);
        let mut lit = String::new();
        macro_rules! flush { () => { if !lit.is_empty() { segs.push(Seg::Lit(std::mem::take(&mut lit))); } } }
        // Zeilennummer
        while i < ch.len() && ch[i].is_whitespace() { lit.push(ch[i]); i += 1; }
        while i < ch.len() && ch[i].is_ascii_digit() { lit.push(ch[i]); i += 1; }
        while i < ch.len() {
            let c = ch[i];
            if c == '"' { quote = !quote; lit.push(c); i += 1; continue; }
            if quote { lit.push(c); i += 1; continue; }
            if c == '{' { while i < ch.len() && ch[i] != '}' { lit.push(ch[i]); i += 1; } if i < ch.len() { lit.push('}'); i += 1; } continue; }
            if in_data { if c == ':' { in_data = false; } lit.push(c); i += 1; continue; }
            if c == '@' { lit.push(c); i += 1; while i < ch.len() && is_word(ch[i]) { lit.push(ch[i]); i += 1; } continue; }
            // $ff und %101 (Hex-/Binärzahlen)
            if (c == '$' && ch.get(i + 1).map_or(false, char::is_ascii_hexdigit)) || (c == '%' && ch.get(i + 1).map_or(false, |x| *x == '0' || *x == '1')) {
                lit.push(c); i += 1;
                while i < ch.len() && ch[i].is_ascii_hexdigit() { lit.push(ch[i]); i += 1; }
                prev_fn = false; continue;
            }
            // Zahlen
            if c.is_ascii_digit() || (c == '.' && ch.get(i + 1).map_or(false, char::is_ascii_digit)) {
                while i < ch.len() && (ch[i].is_ascii_digit() || ch[i] == '.') { lit.push(ch[i]); i += 1; }
                if matches!(ch.get(i), Some('e' | 'E')) {
                    let sign = matches!(ch.get(i + 1), Some('+' | '-'));
                    if ch.get(i + 1 + sign as usize).map_or(false, char::is_ascii_digit) {
                        lit.push(ch[i]); i += 1; if sign { lit.push(ch[i]); i += 1; }
                        while i < ch.len() && ch[i].is_ascii_digit() { lit.push(ch[i]); i += 1; }
                    }
                }
                prev_fn = false; continue;
            }
            if !c.is_ascii_alphabetic() { lit.push(c); i += 1; if !c.is_whitespace() { prev_fn = false; } continue; }
            // Wort
            let st = i;
            while i < ch.len() && is_word(ch[i]) { i += 1; }
            let word: String = ch[st..i].iter().collect();
            let up = word.to_ascii_uppercase();
            let next = ch.get(i).copied();
            if up == "REM" || (up == "DATA" && t.data_to_eol()) {
                lit.push_str(&word); lit.extend(&ch[i..]); i = ch.len(); continue;
            }
            if up == "DATA" { in_data = true; lit.push_str(&word); continue; }
            if t.keyword(&up, next) {
                lit.push_str(&word);
                if next == Some('$') && matches!(t, Target::Cbm(d) if d.token_bytes(&format!("{up}$")).is_some()) { lit.push('$'); i += 1; }
                prev_fn = up == "FN";
                continue;
            }
            // Funktionsname: nach `FN` oder verklebt (`FNname`)
            let mut name = word.clone();
            let mut func = false;
            if t.has_fn() {
                if prev_fn { func = true; }
                else if up.starts_with("FN") && word.len() > 2 && word.as_bytes()[2].is_ascii_alphabetic() { lit.push_str(&word[..2]); name = word[2..].to_string(); func = true; }
            }
            prev_fn = false;
            let mut suffix = String::new();
            if let Some(&s) = ch.get(i) { if s == '$' || s == '%' { suffix.push(s); i += 1; } }
            let mut k = i; while k < ch.len() && ch[k] == ' ' { k += 1; }
            let array = !func && ch.get(k) == Some(&'(');
            if name.len() <= sig && !name.contains('_') {
                (if func { &mut lit_fn } else { &mut lit_var }).insert(name.to_ascii_uppercase());
                lit.push_str(&name); lit.push_str(&suffix);
                continue;
            }
            let key = name.to_ascii_lowercase();
            if seen.insert((func, key.clone())) { order.push(((func, key.clone()), ln + 1)); if name.chars().any(|c| c.is_ascii_lowercase()) { lower_style.insert((func, key.clone())); } }
            let e = kinds.entry((func, key.clone(), suffix.clone())).or_default();
            if array { e.1 = true; } else { e.0 = true; }
            flush!();
            segs.push(Seg::Name { func, key, suffix });
        }
        flush!();
    }
    // Kurznamen vergeben
    let mut errs = vec![];
    let mut warns = vec![];
    let mut map: HashMap<(bool, String), String> = HashMap::new();
    let (mut used_var, mut used_fn) = (lit_var, lit_fn);
    let mut natural_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for ((func, key), line) in &order {
        let used = if *func { &mut used_fn } else { &mut used_var };
        let nat = natural(key, sig);
        match candidates(key, sig).into_iter().find(|c| t.usable(c) && !used.contains(c)) {
            Some(c) => { used.insert(c.clone());
                let shown = if matches!(t, Target::Atari(_)) && lower_style.contains(&(*func, key.clone())) { c.to_ascii_lowercase() } else { c.clone() };
                map.insert((*func, key.clone()), shown);
                if !*func { natural_groups.entry(nat).or_default().push(key.clone()); } }
            None => errs.push(Diagnostic::error("B131", file, *line, format!("Für den Namen '{key}' ist kein freier Kurzname mehr übrig (zu viele Variablen)"))),
        }
    }
    if !errs.is_empty() { return Err(errs); }
    for (nat, keys) in &natural_groups {
        if keys.len() < 2 { continue; }
        let first_line = keys.iter().filter_map(|k| order.iter().find(|((f, n), _)| !*f && n == k).map(|(_, l)| *l)).max().unwrap_or(1);
        let shorts: Vec<String> = keys.iter().map(|k| map[&(false, k.clone())].clone()).collect();
        if sig <= 8 {
            warns.push(Diagnostic::warning("W313", file, first_line, format!(
                "{} unterscheiden sich erst nach den ersten {sig} Zeichen ('{nat}'); sie werden hier getrennt ({}). Ein BASIC ohne diese Umsetzung sähe nur eine Variable",
                keys.iter().map(|k| format!("'{k}'")).collect::<Vec<_>>().join(", "), shorts.join(", "))));
        }
    }
    let mut out = String::new();
    for s in &segs {
        match s {
            Seg::Lit(l) => out.push_str(l),
            Seg::Name { func, key, suffix } => { out.push_str(&map[&(*func, key.clone())]); out.push_str(suffix); }
        }
    }
    let mut names = vec![];
    for ((func, key), _) in &order {
        for ((f, k, suf), (scalar, array)) in &kinds {
            if f != func || k != key { continue; }
            let kind = if *func { "Funktion" } else { match suf.as_str() { "$" => "Text", "%" => "Ganzzahl", _ => "Zahl" } };
            names.push(NameEntry { long: format!("{k}{suf}"), short: format!("{}{suf}", map[&(*func, key.clone())]), kind, scalar: *scalar, array: *array });
        }
    }
    Ok(Renamed { text: out, names, warnings: warns })
}

/// Erste `sig` Zeichen ohne `_`, groß
fn natural(key: &str, sig: usize) -> String { key.chars().filter(|c| *c != '_').take(sig).collect::<String>().to_ascii_uppercase() }

/// Kandidaten für den Kurznamen in der Reihenfolge der Bevorzugung
fn candidates(key: &str, sig: usize) -> Vec<String> {
    let nat = natural(key, sig);
    let mut v = vec![nat.clone()];
    if sig <= 2 {
        let letters: Vec<char> = key.chars().filter(char::is_ascii_alphabetic).map(|c| c.to_ascii_uppercase()).collect();
        let first = letters.first().copied().unwrap_or('V');
        let pool: Vec<char> = ('A'..='Z').chain('0'..='9').collect();
        for c in letters.iter().skip(1) { v.push(format!("{first}{c}")); }
        for c in &pool { v.push(format!("{first}{c}")); }
        for a in 'A'..='Z' { for c in &pool { v.push(format!("{a}{c}")); } }
        v.push(first.to_string());
    } else {
        for n in 2..1000 { let d = n.to_string(); let keep = nat.len().min(sig - d.len()); v.push(format!("{}{d}", &nat[..keep])); }
    }
    v
}

/// Zuordnungen zu einem Wort (für den Hover im Editor): Einträge zum Namen `word` (mit oder ohne `$`/`%`)
pub fn lookup(text: &str, t: Target, word: &str) -> Vec<NameEntry> {
    if !has_directive(text) { return vec![]; }
    let base = word.trim_end_matches(['$', '%']).to_ascii_lowercase();
    match rename(text, t, "") { Ok(r) => r.names.into_iter().filter(|n| n.long.trim_end_matches(['$', '%']) == base).collect(), Err(_) => vec![] }
}
