// SPDX-License-Identifier: GPL-3.0-or-later
//! Vorverarbeitung des nativen BASIC (Konzept „BASIC-Editor und Dialekte“, 6.2, Entscheidung B13):
//! `#define NAME wert` und `#if …` / `#else` / `#endif`. Sie läuft **vor** der Nummerierung (Reihenfolge der Pässe: `#include` → hier →
//! `number_lines` → Zahlenpass → `#longnames` → Tokenisieren).
//!
//! * `#define NAME wert`: Von da an wird jedes **ganze Wort** `NAME` (Groß-/Kleinschreibung zählt) durch `wert` ersetzt, nicht in Zeichenketten,
//!   nicht nach `REM`, nicht in `DATA`, nicht hinter `@` (Labels) und nicht in Zahlen (`$FF`). Der Wert darf andere Namen enthalten (höchstens
//!   acht Stufen, zyklisch: `B176`). Konstanten kosten dadurch weder eine Variable noch Laufzeit. **Keine Makros mit Parametern.**
//! * `#if BEDINGUNG`: `SYSTEM_C64` (Kennung des Zielsystems in Großbuchstaben, `SYSTEM_ATARI8`, …), ein definierter Name (wahr, wenn sein Wert
//!   nicht `0` ist) oder `!BEDINGUNG`. `#else` und `#endif` schließen ab; Verschachtelung ist erlaubt.
//! * Der Text behält **genau so viele Zeilen** wie die Quelle (Direktiven und ausgeschlossene Zeilen werden leer), damit Zeilenangaben der
//!   Meldungen stimmen.
//! * Fehler: `B172` (Name ist ein Schlüsselwort des Dialekts), `B174` (Syntax, doppelte Definition), `B175` (Aufbau von `#if`), `B176` (zyklisch).
use std::collections::{HashMap, HashSet};
use vc_core::{msg, Diagnostic};

/// Eine Definition (für Hover und Gliederung der Oberfläche)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Define { pub name: String, pub value: String, pub line: usize }

/// Einstellungen der Vorverarbeitung
#[derive(Debug, Clone, Default)]
pub struct PreOptions {
    pub file: String,
    /// Kennung des Zielsystems (`c64`, `atari8` …) für `SYSTEM_*`
    pub system: Option<String>,
    /// vordefinierte Namen (Kommandozeile `-D NAME=wert`, Assembler-Marken)
    pub defines: Vec<(String, String)>,
    /// Schlüsselwörter des Dialekts in Großbuchstaben (ein `#define` darf keines davon heißen)
    pub keywords: HashSet<String>,
}

impl PreOptions { pub fn new(file: impl Into<String>) -> Self { PreOptions { file: file.into(), ..Default::default() } } }

/// Ergebnis: Text (gleiche Zeilenzahl wie die Quelle), alle Definitionen, ausgeschlossene Zeilen (1-basiert)
#[derive(Debug, Clone)]
pub struct Preprocessed { pub text: String, pub defines: Vec<Define>, pub excluded: Vec<usize> }

fn is_ident_start(c: char) -> bool { c.is_ascii_alphabetic() || c == '_' }
fn is_ident(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

/// Hat die Quelle Direktiven dieses Passes? (schneller Weg: ohne `#define`/`#if` bleibt alles unverändert)
pub fn has_directives(src: &str) -> bool { src.lines().any(|l| directive_word(l).is_some() || crate::blob::is_blob_directive(l) || crate::blob::is_old_blob_directive(l)) }

/// `define`, `if`, `else`, `endif` aus einer Zeile `#word …`
fn directive_word(line: &str) -> Option<(&'static str, &str)> {
    let t = line.trim();
    let r = t.strip_prefix('#')?;
    let end = r.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(r.len());
    let w = r[..end].to_ascii_lowercase();
    let rest = r[end..].trim();
    let word = match w.as_str() { "define" => "define", "if" => "if", "else" => "else", "endif" => "endif", _ => return None };
    Some((word, rest))
}

/// Führt die Vorverarbeitung aus.
pub fn run(src: &str, o: &PreOptions) -> Result<Preprocessed, Vec<Diagnostic>> {
    if !has_directives(src) && o.defines.is_empty() { return Ok(Preprocessed { text: src.to_string(), defines: vec![], excluded: vec![] }); }
    let mut defs: HashMap<String, String> = o.defines.iter().cloned().collect();
    let mut list: Vec<Define> = vec![];
    let mut errs: Vec<Diagnostic> = vec![];
    let mut excluded = vec![];
    // Stapel der offenen `#if`: (Zeile, Eltern aktiv, Zweig gewählt, jetzt aktiv, `#else` schon gesehen)
    let mut stack: Vec<(usize, bool, bool, bool, bool)> = vec![];
    let mut out: Vec<String> = vec![];
    let system = o.system.as_ref().map(|s| s.to_ascii_uppercase());
    let truth = |cond: &str, defs: &HashMap<String, String>| -> Option<bool> {
        let (neg, c) = match cond.strip_prefix('!') { Some(r) => (true, r.trim()), None => (false, cond) };
        if c.is_empty() || !c.chars().all(is_ident) || !c.chars().next().map_or(false, is_ident_start) { return None; }
        let v = if let Some(sys) = c.strip_prefix("SYSTEM_") { system.as_deref().map_or(false, |s| s == sys) } else { defs.get(c).map_or(false, |v| v.trim() != "0") };
        Some(v != neg)
    };
    for (i, raw) in src.lines().enumerate() {
        let n = i + 1;
        let active = stack.last().map_or(true, |s| s.3);
        if active && crate::blob::is_blob_directive(raw) { errs.push(Diagnostic::error("B177", &o.file, n, msg!("diag-basic-b177-blob-here"))); out.push(String::new()); continue; }
        if active && crate::blob::is_old_blob_directive(raw) { errs.push(Diagnostic::error("B177", &o.file, n, msg!("diag-basic-b177-blob-renamed"))); out.push(String::new()); continue; }
        match directive_word(raw) {
            Some(("if", rest)) => {
                let v = if active { match truth(rest, &defs) { Some(v) => v, None => { errs.push(Diagnostic::error("B175", &o.file, n, msg!("diag-basic-b175-if-condition", text = rest))); false } } } else { false };
                stack.push((n, active, v, active && v, false));
                excluded.push(n); out.push(String::new());
            }
            Some(("else", _)) => {
                match stack.last_mut() {
                    None => errs.push(Diagnostic::error("B175", &o.file, n, msg!("diag-basic-b175-else-without-if"))),
                    Some(s) if s.4 => errs.push(Diagnostic::error("B175", &o.file, n, msg!("diag-basic-b175-else-twice"))),
                    Some(s) => { s.4 = true; s.3 = s.1 && !s.2; }
                }
                excluded.push(n); out.push(String::new());
            }
            Some(("endif", _)) => {
                if stack.pop().is_none() { errs.push(Diagnostic::error("B175", &o.file, n, msg!("diag-basic-b175-endif-without-if"))); }
                excluded.push(n); out.push(String::new());
            }
            Some(("define", rest)) => {
                if active {
                    let (name, value) = match rest.find(char::is_whitespace) { Some(p) => (&rest[..p], rest[p..].trim()), None => (rest, "") };
                    if name.is_empty() || !name.chars().all(is_ident) || !is_ident_start(name.chars().next().unwrap()) {
                        errs.push(Diagnostic::error("B174", &o.file, n, msg!("diag-basic-b174-define-syntax", text = raw.trim())));
                    } else if o.keywords.contains(&name.to_ascii_uppercase()) {
                        errs.push(Diagnostic::error("B172", &o.file, n, msg!("diag-basic-b172-define-keyword", name = name)));
                    } else if list.iter().any(|d| d.name == name) {
                        errs.push(Diagnostic::error("B174", &o.file, n, msg!("diag-basic-b174-define-twice", name = name)));
                    } else {
                        defs.insert(name.to_string(), value.to_string());
                        list.push(Define { name: name.to_string(), value: value.to_string(), line: n });
                    }
                }
                excluded.push(n); out.push(String::new());
            }
            _ => {
                if !active { excluded.push(n); out.push(String::new()); continue; }
                match substitute(raw, &defs, &mut vec![], 0) {
                    Ok(t) => out.push(t),
                    Err(name) => { errs.push(Diagnostic::error("B176", &o.file, n, msg!("diag-basic-b176-define-cycle", name = name.as_str()))); out.push(raw.to_string()); }
                }
            }
        }
    }
    for s in &stack { errs.push(Diagnostic::error("B175", &o.file, s.0, msg!("diag-basic-b175-if-not-closed"))); }
    if !errs.is_empty() { return Err(errs); }
    let mut text = out.join("\n");
    if src.ends_with('\n') { text.push('\n'); }
    Ok(Preprocessed { text, defines: list, excluded })
}

/// Ersetzt definierte Namen in einer Zeile; `Err(name)` bei zyklischer oder zu tiefer Ersetzung
fn substitute(line: &str, defs: &HashMap<String, String>, active: &mut Vec<String>, depth: usize) -> Result<String, String> {
    if defs.is_empty() { return Ok(line.to_string()); }
    let t = line.trim_start();
    if t.starts_with(';') || t.starts_with('#') { return Ok(line.to_string()); }
    let ch: Vec<char> = line.chars().collect();
    let (mut out, mut i, mut quote) = (String::new(), 0usize, false);
    while i < ch.len() {
        let c = ch[i];
        if c == '"' { quote = !quote; out.push(c); i += 1; continue; }
        if quote { out.push(c); i += 1; continue; }
        if c == '{' { while i < ch.len() && ch[i] != '}' { out.push(ch[i]); i += 1; } if i < ch.len() { out.push('}'); i += 1; } continue; }
        if c == '$' || c == '%' || c == '@' {                           // Hexzahl, Binärzahl, Label
            out.push(c); i += 1;
            while i < ch.len() && is_ident(ch[i]) { out.push(ch[i]); i += 1; }
            continue;
        }
        if c.is_ascii_digit() { while i < ch.len() && (ch[i].is_ascii_alphanumeric() || ch[i] == '.') { out.push(ch[i]); i += 1; } continue; }
        if !is_ident_start(c) { out.push(c); i += 1; continue; }
        let mut j = i;
        while j < ch.len() && is_ident(ch[j]) { j += 1; }
        let word: String = ch[i..j].iter().collect();
        let up = word.to_ascii_uppercase();
        if up == "ASM" && ch.get(j) == Some(&'.') && ch.get(j + 1).map_or(false, |c| is_ident_start(*c)) {          // Marke des Maschinencodes (`#asm`)
            let mut k = j + 1;
            while k < ch.len() && is_ident(ch[k]) { k += 1; }
            let key = format!("asm.{}", ch[j + 1..k].iter().collect::<String>().to_ascii_lowercase());
            match defs.get(&key) { Some(v) => out.push_str(v), None => out.extend(&ch[i..k]) }
            i = k; continue;
        }
        if up == "REM" && !defs.contains_key(&word) { out.extend(&ch[i..]); return Ok(out); }
        if up == "DATA" && !defs.contains_key(&word) {                    // bis zum Doppelpunkt unverändert
            out.push_str(&word); i = j;
            while i < ch.len() && ch[i] != ':' { out.push(ch[i]); i += 1; }
            continue;
        }
        if let Some(v) = defs.get(&word).filter(|_| !matches!(ch.get(j), Some('$' | '%'))) {
            if active.contains(&word) || depth >= 8 { return Err(word); }
            active.push(word.clone());
            let r = substitute(v, defs, active, depth + 1);
            active.pop();
            out.push_str(&r?);
        } else { out.push_str(&word); }
        i = j;
    }
    Ok(out)
}
