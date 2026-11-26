// SPDX-License-Identifier: GPL-3.0-or-later
//! Syntaxprüfung für Commodore-BASIC (Konzept „BASIC-Editor und Dialekte“, Abschnitt 6.1, Entscheidung B12).
//!
//! Alle Meldungen sind **Warnungen** mit stabilem Code (Katalog `diag-basic-b15x-…`) und jeder Code lässt sich abschalten
//! ([`CheckOptions::disabled`], [`crate::codes::can_disable`]). Geprüft wird, was der Interpreter erst zur Laufzeit meldet; wo ein Befund nur
//! ein Verdacht ist, steht das im Text. Es gibt hier keinen Fehler: Programme, die der Interpreter sicher ablehnt, scheitern schon am Tokenizer
//! (`B100`–`B114`).
//!
//! * **Stufe 1** (alle Commodore-Dialekte): `B150` Schlüsselwort im Variablennamen (`SCORE`, `TOTAL`), `B151` `FOR` ohne `NEXT` und umgekehrt,
//!   `B152` `GOSUB` ohne `RETURN`, `B153` Sprungziel nicht vorhanden (nur Nummernmodus), `B154` Variablen, die der Interpreter nicht unterscheidet,
//!   `B155` Zeichenkette am Zeilenende offen (Hinweis), `B167` `'` ist kein Kommentar.
//! * **Stufe 2** (Dialekte mit `syntax` in der Dialektdatei: BASIC V2 und BASIC 65): `B156` Argumentzahl, `B157` Anweisung eines anderen
//!   Dialekts, `B159` Funktion ohne Klammer.
use crate::codes;
use crate::dialect::Dialect;
use crate::labels::{parse_line, Unit, K};
use std::collections::{HashMap, HashSet};
use vc_core::{msg, Diagnostic, Range};

/// Einstellungen der Prüfung
#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    /// abgeschaltete Codes (nur Warnungen und Hinweise lassen sich abschalten)
    pub disabled: Vec<String>,
    /// die ganze Prüfung auslassen
    pub off: bool,
}

impl CheckOptions {
    fn on(&self, code: &str) -> bool { !self.off && !(codes::can_disable(code) && self.disabled.iter().any(|d| d.eq_ignore_ascii_case(code))) }
}

// ───────────────────────── Wörter zerlegen ─────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum T { Kw(String), Name(String), Num(String), Str { closed: bool }, Sym(char) }

#[derive(Debug, Clone)]
struct Tok { t: T, start: usize, end: usize }

/// Zerlegt eine Anweisungszeile wie das ROM: Schlüsselwörter werden an jeder Stelle erkannt, auch mitten in Namen (`SCORE` = `SC`, `OR`, `E`).
/// Nach `REM` und in `DATA` gibt es keine weiteren Wörter.
fn lex(body: &str, d: &Dialect, whole: bool) -> Vec<Tok> {
    let ch: Vec<char> = body.chars().collect();
    let n = ch.len();
    let lx = crate::numbers::Lexicon::Cbm(d);
    let (mut out, mut i) = (vec![], 0usize);
    while i < n {
        let c = ch[i];
        if c == ' ' { i += 1; continue; }
        let st = i;
        if c == '"' {
            i += 1;
            while i < n && ch[i] != '"' { i += 1; }
            let closed = i < n;
            if closed { i += 1; }
            out.push(Tok { t: T::Str { closed }, start: st, end: i });
        } else if c == '{' {
            while i < n && ch[i] != '}' { i += 1; }
            i = (i + 1).min(n);
        } else if c.is_ascii_digit() || (c == '.' && ch.get(i + 1).map_or(false, char::is_ascii_digit)) {
            while i < n && (ch[i].is_ascii_digit() || ch[i] == '.') { i += 1; }
            if matches!(ch.get(i), Some('e' | 'E')) {
                let sign = matches!(ch.get(i + 1), Some('+' | '-'));
                if ch.get(i + 1 + sign as usize).map_or(false, char::is_ascii_digit) { i += 1 + sign as usize; while i < n && ch[i].is_ascii_digit() { i += 1; } }
            }
            out.push(Tok { t: T::Num(ch[st..i].iter().collect()), start: st, end: i });
        } else if (c == '$' || c == '%') && !(i > 0 && ch[i - 1].is_ascii_alphanumeric()) {
            i += 1;
            while i < n && (ch[i].is_ascii_hexdigit() || ch[i] == '_') { i += 1; }
            out.push(Tok { t: T::Num(ch[st..i].iter().collect()), start: st, end: i });
        } else if c == '@' && ch.get(i + 1).map_or(false, |x| x.is_ascii_alphabetic() || *x == '_') {
            i += 1;                                                     // Label: Name wird zur Zeilennummer, keine Variable
            while i < n && (ch[i].is_ascii_alphanumeric() || ch[i] == '_') { i += 1; }
        } else if c.is_ascii_alphabetic() && !(i > 0 && ch[i - 1].is_ascii_alphanumeric()) && ident_end(&ch, i).map_or(false, |e| e - i > 5 && ch[i..e].iter().collect::<String>().to_ascii_lowercase().ends_with("_line")) {
            let e = ident_end(&ch, i).unwrap();                         // `<marke>_line`: Zeilennummer einer Marke (Medien), wirkt wie eine Zahl
            out.push(Tok { t: T::Num("0".into()), start: i, end: e });
            i = e;
        } else if c == '\'' {
            out.push(Tok { t: T::Sym('\''), start: i, end: i + 1 });
            break;                                                      // der Rest ist als Kommentar gemeint: nicht weiter prüfen
        } else if c == '?' {
            out.push(Tok { t: T::Kw("PRINT".into()), start: i, end: i + 1 });
            i += 1;
        } else if c.is_ascii_alphabetic() {
            if let Some(len) = lx.keyword_at(&ch, i, whole) {
                let w: String = ch[i..i + len].iter().collect::<String>().to_ascii_uppercase();
                i += len;
                out.push(Tok { t: T::Kw(w.clone()), start: st, end: i });
                if w == "REM" { break; }
                if w == "DATA" { while i < n && ch[i] != ':' { i += 1; } }
                continue;
            }
            let mut j = i;
            while j < n && ch[j].is_ascii_alphanumeric() {
                if j > i && ch[j].is_ascii_alphabetic() && !whole && lx.keyword_at(&ch, j, whole).is_some() { break; }
                j += 1;
            }
            if matches!(ch.get(j), Some('$' | '%')) && j < n { j += 1; }
            out.push(Tok { t: T::Name(ch[i..j].iter().collect::<String>().to_ascii_uppercase()), start: i, end: j });
            i = j;
        } else {
            out.push(Tok { t: T::Sym(c), start: i, end: i + 1 });
            i += 1;
        }
    }
    out
}

fn is_word(w: &str) -> bool { w.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) }

// ───────────────────────── Prüfung ─────────────────────────

struct Stmt { line: usize, base: usize, toks: Vec<Tok> }

/// Prüft eine Quelle (eine Datei; Einbindungen sind schon aufgelöst).
pub fn check_cbm(src: &str, file: &str, d: &Dialect, o: &CheckOptions) -> Vec<Diagnostic> {
    check_units(&[Unit { file: file.to_string(), text: src.to_string() }], d, o)
}

/// Prüft die Dateien einer Einbindung zusammen (Wurzel zuerst, Einbindungen an ihrer Stelle wie beim Bauen)
pub fn check_units(units: &[Unit], d: &Dialect, o: &CheckOptions) -> Vec<Diagnostic> {
    if o.off || units.is_empty() { return vec![]; }
    let whole = units.iter().any(|u| crate::longnames::has_directive(&u.text));
    // Zeilen in Programmreihenfolge (Einbindungen an ihrer Stelle)
    let mut lines: Vec<(usize, usize, String)> = vec![];                     // (Datei, Zeile, Text)
    fn go(u: usize, units: &[Unit], lines: &mut Vec<(usize, usize, String)>, seen: &mut Vec<usize>) {
        seen.push(u);
        for (i, raw) in units[u].text.lines().enumerate() {
            lines.push((u, i, raw.to_string()));
            if let K::Include(p) = parse_line(raw) {
                let key = std::path::Path::new(&units[u].file).parent().map(|x| x.to_path_buf()).unwrap_or_default().join(p).to_string_lossy().into_owned();
                if let Some(v) = units.iter().position(|x| x.file == key) { if !seen.contains(&v) { go(v, units, lines, seen); } }
            }
        }
        seen.pop();
    }
    go(0, units, &mut lines, &mut vec![]);
    let mut ds: Vec<Diagnostic> = vec![];
    let mut push = |ds: &mut Vec<Diagnostic>, code: &'static str, u: usize, line: usize, off: usize, tok: Option<(usize, usize)>, m: vc_core::Msg| {
        if !o.on(code) { return; }
        let mut x = Diagnostic::warning(code, &units[u].file, line + 1, m);
        if let Some((s, e)) = tok { x = x.with_range(Range { line: line + 1, col: off + s + 1, end_line: line + 1, end_col: off + e + 1 }); }
        ds.push(x);
    };
    let mut stmts: Vec<(usize, Stmt)> = vec![];                               // (Datei, Anweisungszeile)
    let mut numbered: HashSet<u32> = HashSet::new();
    let (mut any_unnumbered, mut any_number) = (false, false);
    for (u, i, raw) in &lines {
        let K::Prog(p) = parse_line(raw) else { continue };
        if p.body.is_empty() { continue; }
        match p.num { Some(n) => { numbered.insert(n); any_number = true; } None => any_unnumbered = true }
        let off = raw.trim_end().chars().count() - p.body.chars().count();
        stmts.push((*u, Stmt { line: *i, base: off, toks: lex(&p.body, d, whole) }));
    }
    let numbers_only = any_number && !any_unnumbered;
    // Zeilenprüfungen
    for (u, s) in &stmts {
        let t = &s.toks;
        for (k, tok) in t.iter().enumerate() {
            let rng = Some((tok.start, tok.end));
            match &tok.t {
                T::Sym('\'') if !d.quote_comment => push(&mut ds, "B167", *u, s.line, s.base, rng, msg!("diag-basic-b167-quote-not-comment")),
                T::Str { closed: false } => push(&mut ds, "B155", *u, s.line, s.base, rng, msg!("diag-basic-b155-string-open")),
                T::Kw(w) if is_word(w) && !whole => {
                    // Name, Schlüsselwort, Name ohne Leerzeichen: das Schlüsselwort steckt mitten in einem Namen
                    let prev = k.checked_sub(1).map(|j| &t[j]);
                    let next = t.get(k + 1);
                    let joined_prev = prev.filter(|p| p.end == tok.start && matches!(p.t, T::Name(_)));
                    let joined_next = next.filter(|x| x.start == tok.end && matches!(&x.t, T::Name(n) if is_word(n)));
                    if let (Some(a), Some(b)) = (joined_prev, joined_next) {
                        if let (T::Name(x), T::Name(y)) = (&a.t, &b.t) {
                            if x.chars().last().map_or(false, |c| c.is_ascii_alphabetic()) {
                                push(&mut ds, "B150", *u, s.line, s.base, Some((a.start, b.end)), msg!("diag-basic-b150-keyword-in-name", name = format!("{x}{w}{y}"), keyword = w.as_str()));
                            }
                        }
                    } else if let Some(b) = joined_next {
                        // Name beginnt mit einem Wort, das keine Anweisung einleitet (`TOTAL` = `TO` + `TAL`)
                        let starts_run = prev.map_or(true, |p| p.end != tok.start);
                        if starts_run && matches!(w.as_str(), "TO" | "THEN" | "STEP" | "AND" | "OR") {
                            if let T::Name(y) = &b.t { push(&mut ds, "B150", *u, s.line, s.base, Some((tok.start, b.end)), msg!("diag-basic-b150-keyword-in-name", name = format!("{w}{y}"), keyword = w.as_str())); }
                        }
                    }
                }
                _ => {}
            }
        }
        // Stufe 2: Argumentzahl, Funktionen, Anweisungen anderer Dialekte
        let has_syntax = !d.syntax.statements.is_empty() || !d.syntax.functions.is_empty();
        let mut at_start = true;
        for (k, tok) in t.iter().enumerate() {
            let rng = Some((tok.start, tok.end));
            if at_start && !d.foreign.is_empty() && has_syntax {
                if let T::Name(_) = &tok.t {
                    // Wort am Anweisungsanfang: ganzes Wort aus den Namen- und Schlüsselwort-Stücken bis zum nächsten Trenner
                    let mut e = k;
                    while t.get(e + 1).map_or(false, |x| x.start == t[e].end && matches!(x.t, T::Name(_) | T::Kw(_) | T::Num(_))) { e += 1; }
                    let word: String = t[k..=e].iter().map(|x| match &x.t { T::Name(n) | T::Kw(n) | T::Num(n) => n.clone(), _ => String::new() }).collect();
                    let assign = t.get(e + 1).map_or(false, |x| x.t == T::Sym('=') || x.t == T::Kw("=".into()) || x.t == T::Sym('('));
                    if d.foreign.contains(&word) && !assign {
                        push(&mut ds, "B157", *u, s.line, s.base, Some((tok.start, t[e].end)), msg!("diag-basic-b157-keyword-unknown", word = word.as_str(), dialect = d.id.as_str()));
                    }
                }
            }
            at_start = matches!(&tok.t, T::Sym(':')) || matches!(&tok.t, T::Kw(w) if w == "THEN" || w == "ELSE");
            let T::Kw(w) = &tok.t else { continue };
            if let Some(&(lo, hi)) = d.syntax.statements.get(w.as_str()) {
                let n = count_args(t, k + 1, false);
                if n < lo || n > hi { push(&mut ds, "B156", *u, s.line, s.base, rng, msg!("diag-basic-b156-arg-count", word = w.as_str(), count = n, expected = range_text(lo, hi))); }
            }
            let key = w.trim_end_matches('(');
            if let Some(&(lo, hi)) = d.syntax.functions.get(w.as_str()).or_else(|| d.syntax.functions.get(key)) {
                let paren_in_token = w.ends_with('(');
                if !paren_in_token && !matches!(t.get(k + 1).map(|x| &x.t), Some(T::Sym('('))) {
                    push(&mut ds, "B159", *u, s.line, s.base, rng, msg!("diag-basic-b159-function-paren", word = w.as_str()));
                } else {
                    let n = count_args(t, if paren_in_token { k + 1 } else { k + 2 }, true);
                    if n < lo || n > hi { push(&mut ds, "B156", *u, s.line, s.base, rng, msg!("diag-basic-b156-arg-count", word = w.as_str(), count = n, expected = range_text(lo, hi))); }
                }
            }
        }
    }
    // Sprungziele (Nummernmodus)
    if numbers_only {
        for (u, s) in &stmts {
            let t = &s.toks;
            for (k, tok) in t.iter().enumerate() {
                let T::Kw(w) = &tok.t else { continue };
                if !d.jumps.iter().any(|j| j == w) { continue; }
                let mut j = k + 1;
                loop {
                    let Some(Tok { t: T::Num(x), start, end }) = t.get(j) else { break };
                    let ints = x.chars().all(|c| c.is_ascii_digit());
                    let after = t.get(j + 1).map(|y| &y.t);
                    let calc = match after { Some(T::Kw(k)) => !is_word(k), Some(T::Sym(c)) => *c == '(', _ => false };
                    if ints && !calc { if let Ok(v) = x.parse::<u32>() { if !numbered.contains(&v) { push(&mut ds, "B153", *u, s.line, s.base, Some((*start, *end)), msg!("diag-basic-b153-target-missing", num = v)); } } }
                    if matches!(w.as_str(), "GOTO" | "GOSUB") && matches!(t.get(j + 1).map(|y| &y.t), Some(T::Sym(','))) { j += 2; } else { break; }
                }
            }
        }
    }
    // FOR / NEXT
    let mut stack: Vec<(String, usize, usize, usize, usize)> = vec![];       // (Variable, Datei, Zeile, Beginn, Ende)
    let (mut first_gosub, mut first_return, mut gosubs, mut returns): (Option<(usize, usize, usize, usize)>, Option<(usize, usize, usize, usize)>, usize, usize) = (None, None, 0, 0);
    for (u, s) in &stmts {
        let t = &s.toks;
        for (k, tok) in t.iter().enumerate() {
            let T::Kw(w) = &tok.t else { continue };
            match w.as_str() {
                "FOR" => { if let Some(Tok { t: T::Name(v), .. }) = t.get(k + 1) { stack.push((v.clone(), *u, s.line, s.base + tok.start, s.base + tok.end)); } }
                "NEXT" => {
                    let mut names = vec![];
                    let mut j = k + 1;
                    while let Some(Tok { t: T::Name(v), .. }) = t.get(j) { names.push(v.clone()); if matches!(t.get(j + 1).map(|y| &y.t), Some(T::Sym(','))) { j += 2; } else { break; } }
                    if names.is_empty() {
                        if stack.pop().is_none() { push(&mut ds, "B151", *u, s.line, s.base, Some((tok.start, tok.end)), msg!("diag-basic-b151-next-without-for", var = "")); }
                    }
                    for v in names {
                        match stack.iter().rposition(|(x, ..)| *x == v) {
                            Some(p) => stack.truncate(p),
                            None => push(&mut ds, "B151", *u, s.line, s.base, Some((tok.start, tok.end)), msg!("diag-basic-b151-next-without-for", var = v.as_str())),
                        }
                    }
                }
                "GOSUB" => { gosubs += 1; first_gosub.get_or_insert((*u, s.line, s.base + tok.start, s.base + tok.end)); }
                "RETURN" => { returns += 1; first_return.get_or_insert((*u, s.line, s.base + tok.start, s.base + tok.end)); }
                _ => {}
            }
        }
    }
    for (v, u, l, a, b) in stack { push(&mut ds, "B151", u, l, 0, Some((a, b)), msg!("diag-basic-b151-for-without-next", var = v.as_str())); }
    if gosubs > 0 && returns == 0 { let (u, l, a, b) = first_gosub.unwrap(); push(&mut ds, "B152", u, l, 0, Some((a, b)), msg!("diag-basic-b152-gosub-without-return")); }
    if returns > 0 && gosubs == 0 { let (u, l, a, b) = first_return.unwrap(); push(&mut ds, "B152", u, l, 0, Some((a, b)), msg!("diag-basic-b152-return-without-gosub")); }
    // Variablen, die der Interpreter nicht unterscheidet
    if !whole {
        let sig = d.var_sig_chars;
        let mut seen: HashMap<String, String> = HashMap::new();
        let mut warned: HashSet<String> = HashSet::new();
        for (u, s) in &stmts {
            let t = &s.toks;
            let mut after_fn = false;
            for (k, tok) in t.iter().enumerate() {
                if let T::Kw(w) = &tok.t { after_fn = w == "FN" || w == "DEF"; continue; }
                let T::Name(n) = &tok.t else { after_fn = false; continue };
                if after_fn { after_fn = false; continue; }
                let (base, suffix) = match n.chars().last() { Some(c @ ('$' | '%')) => (&n[..n.len() - 1], c.to_string()), _ => (n.as_str(), String::new()) };
                if base.chars().count() <= sig || d.is_reserved_var(base) { continue; }
                let array = matches!(t.get(k + 1).map(|x| &x.t), Some(T::Sym('(')));
                let key = format!("{}{}{}", base.chars().take(sig).collect::<String>(), suffix, if array { "(" } else { "" });
                match seen.get(&key) {
                    None => { seen.insert(key, n.clone()); }
                    Some(first) if first != n && warned.insert(format!("{first}/{n}")) => push(&mut ds, "B154", *u, s.line, s.base, Some((tok.start, tok.end)), msg!("diag-basic-b154-variable-clash", name = n.as_str(), other = first.as_str(), chars = sig)),
                    _ => {}
                }
            }
        }
    }
    merge_b150(ds)
}

/// `B150`: alle Vorkommen desselben Namens in einer Datei werden zu EINER Meldung (an der ersten Stelle, mit den weiteren Zeilen)
fn merge_b150(ds: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let name_of = |d: &Diagnostic| d.args.iter().find(|(k, _)| *k == "name").and_then(|(_, v)| if let vc_core::MsgArg::Text(t) = v { Some(t.clone()) } else { None });
    let mut firsts: std::collections::HashMap<(String, String), usize> = std::collections::HashMap::new();
    let mut more: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    let mut drop = vec![false; ds.len()];
    for (i, d) in ds.iter().enumerate() {
        if d.code != "B150" { continue; }
        let Some(n) = name_of(d) else { continue };
        match firsts.get(&(d.file.clone(), n.clone())) {
            None => { firsts.insert((d.file.clone(), n), i); }
            Some(&f) => { drop[i] = true; let l = d.line; let v = more.entry(f).or_default(); if !v.contains(&l) && l != ds[f].line { v.push(l); } }
        }
    }
    let mut out = vec![];
    for (i, d) in ds.into_iter().enumerate() {
        if drop[i] { continue; }
        match more.get(&i).filter(|v| !v.is_empty()) {
            Some(v) => {
                let (name, kw) = (name_of(&d).unwrap_or_default(), d.args.iter().find(|(k, _)| *k == "keyword").and_then(|(_, v)| if let vc_core::MsgArg::Text(t) = v { Some(t.clone()) } else { None }).unwrap_or_default());
                let lines = v.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ");
                let mut x = Diagnostic::warning("B150", &d.file, d.line, msg!("diag-basic-b150-keyword-in-name-more", name = name, keyword = kw, lines = lines));
                x.range = d.range.clone();
                out.push(x);
            }
            None => out.push(d),
        }
    }
    out
}

fn range_text(lo: usize, hi: usize) -> String { if lo == hi { lo.to_string() } else { format!("{lo}-{hi}") } }

/// Zahl der Argumente ab `from`: bei Funktionen bis zur schließenden Klammer, bei Anweisungen bis `:`, `THEN` oder `ELSE`; Kommas in Klammern zählen nicht
fn count_args(t: &[Tok], from: usize, in_paren: bool) -> usize {
    let (mut depth, mut commas, mut any) = (0i32, 0usize, false);
    for tok in t.iter().skip(from) {
        match &tok.t {
            T::Sym('(') => { depth += 1; any = true; }
            T::Kw(w) if w.ends_with('(') => { depth += 1; any = true; }
            T::Sym(')') => { if depth == 0 && in_paren { break; } depth -= 1; any = true; }
            T::Sym(',') if depth == 0 => commas += 1,
            T::Sym(':') if depth == 0 && !in_paren => break,
            T::Kw(w) if depth == 0 && !in_paren && (w == "THEN" || w == "ELSE") => break,
            _ => any = true,
        }
    }
    if any || commas > 0 { commas + 1 } else { 0 }
}

/// Ende eines Namens aus Buchstaben, Ziffern und `_` ab `i` (`Some`, wenn er einen `_` enthält)
fn ident_end(ch: &[char], i: usize) -> Option<usize> {
    let mut e = i;
    while e < ch.len() && (ch[e].is_ascii_alphanumeric() || ch[e] == '_') { e += 1; }
    ch[i..e].contains(&'_').then_some(e)
}
