// SPDX-License-Identifier: GPL-3.0-or-later
//! Quelltext → Knotenbaum: Includes, `.if/.else/.endif`, `.repeat`, Makrodefinitionen.
use crate::parse::{parse_line, split_top, strip_comment, Body, Stmt};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use vc_core::Diagnostic;

#[derive(Clone, Debug)]
pub struct RawLine { pub file: Rc<str>, pub line: usize, pub text: String }

pub enum Cond { Expr(String), Defined(String), NotDefined(String), Else }

pub struct IfNode { pub line: usize, pub file: Rc<str>, pub arms: Vec<(Cond, Vec<Node>)> }
pub struct RepeatNode { pub line: usize, pub file: Rc<str>, pub count: String, pub var: Option<String>, pub body: Vec<Node> }
pub enum ForSpec { C { init: String, cond: String, step: String }, In { var: String, expr: String } }
pub struct ForNode { pub line: usize, pub file: Rc<str>, pub spec: ForSpec, pub body: Vec<Node> }
/// `.struct`/`.enum`: Rohzeilen des Rumpfs, ausgewertet erst beim Durchlauf (Anzahlen und Werte dürfen Ausdrücke sein)
pub struct StructNode { pub line: usize, pub file: Rc<str>, pub name: String, pub is_enum: bool, pub body: Vec<RawLine> }
pub enum Node { Stmt(Stmt), If(IfNode), Repeat(RepeatNode), For(ForNode), Struct(StructNode) }

#[derive(Clone, Debug)]
pub struct Macro { pub params: Vec<String>, /// Standardwerte (gleiche Länge wie `params`); `None` = Pflichtparameter
    pub defaults: Vec<Option<String>>, pub lines: Vec<RawLine> }
pub type Macros = HashMap<String, Macro>;

/// Sucht `name` relativ zur aufrufenden Datei, dann in den Include-Verzeichnissen.
pub fn resolve(from_file: &str, name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let base = Path::new(from_file).parent().map(Path::to_path_buf).unwrap_or_default();
    std::iter::once(base).chain(dirs.iter().cloned()).map(|d| d.join(name)).find(|p| p.is_file())
}

pub fn angled(args: &str) -> Option<&str> {
    let a = args.trim();
    if a.len() >= 3 && a.starts_with('<') && a.ends_with('>') { Some(&a[1..a.len() - 1]) } else { None }
}

pub fn quoted(args: &str) -> Option<&str> {
    let a = args.trim();
    if a.len() >= 2 && a.starts_with('"') && a.ends_with('"') { Some(&a[1..a.len() - 1]) } else { None }
}

pub fn load(text: &str, file: &str, dirs: &[PathBuf], depth: usize, out: &mut Vec<RawLine>,
            files: &mut Vec<(String, String)>) -> Result<(), Diagnostic> {
    load_in(text, file, dirs, depth, out, files, "", &mut std::collections::HashSet::new())
}

/// `ns` ist der Namensraumpfad der aktuellen Datei, `once` die bereits per `#include` geladenen (Datei, Namensraum)-Paare.
fn load_in(text: &str, file: &str, dirs: &[PathBuf], depth: usize, out: &mut Vec<RawLine>,
           files: &mut Vec<(String, String)>, ns: &str, once: &mut std::collections::HashSet<(PathBuf, String)>) -> Result<(), Diagnostic> {
    files.push((file.to_string(), text.to_string()));
    let rc: Rc<str> = Rc::from(file);
    for (i, l) in text.lines().enumerate() {
        let st = strip_comment(l).trim();
        let mut words = st.splitn(2, char::is_whitespace);
        let first = words.next().unwrap_or("");
        let is_dot = first.eq_ignore_ascii_case(".include");
        let is_hash = first.eq_ignore_ascii_case("#include");
        if is_dot || is_hash {
            let arg = words.next().unwrap_or("");
            // `.include "datei" as name`: Inhalt in einem eigenen Namensraum `name` (Zugriff von außen: name.symbol)
            let (arg, alias) = match arg.rfind(" as ").or_else(|| arg.to_ascii_lowercase().rfind(" as ")) {
                Some(k) if arg[..k].trim_end().ends_with(['"', '>']) => (arg[..k].trim(), Some(arg[k + 4..].trim())),
                _ => (arg, None),
            };
            if let Some(a) = alias {
                if a.is_empty() || !a.chars().all(|c| c.is_alphanumeric() || c == '_') || a.starts_with(|c: char| c.is_ascii_digit()) {
                    return Err(Diagnostic::error("E208", file, i + 1, format!("{first} ... as: ungültiger Name '{a}' (nur Buchstaben, Ziffern und _)")));
                }
            }
            // `#include <pfad>` sucht nur in den Include-Verzeichnissen (Systembibliothek), `"pfad"` zuerst relativ zur Datei
            let name = quoted(arg).or_else(|| if is_hash { angled(arg) } else { None })
                .ok_or_else(|| Diagnostic::error("E208", file, i + 1, format!("{first} erwartet einen Dateinamen in Anführungszeichen oder <spitzen Klammern>")))?;
            if depth >= 16 { return Err(Diagnostic::error("E211", file, i + 1, "Include zu tief verschachtelt (Zyklus?)")); }
            let path = resolve(file, name, dirs).ok_or_else(|| Diagnostic::error("E212", file, i + 1, format!("Include-Datei '{name}' nicht gefunden")))?;
            // `#include` bindet jede Datei nur einmal ein (wie `#pragma once`): so dürfen Bibliotheken einander einbinden; mit `as name` immer (eigener Namensraum)
            let sub_ns = match alias { Some(a) if ns.is_empty() => a.to_string(), Some(a) => format!("{ns}.{a}"), None => ns.to_string() };
            if is_hash && !once.insert((path.clone(), sub_ns.clone())) { continue; }
            let sub = std::fs::read_to_string(&path).map_err(|e| Diagnostic::error("E212", file, i + 1, format!("{}: {e}", path.display())))?;
            if let Some(a) = alias { out.push(RawLine { file: rc.clone(), line: i + 1, text: format!(".namespace {a} {{") }); }
            load_in(&sub, &path.to_string_lossy(), dirs, depth + 1, out, files, &sub_ns, once)?;
            if alias.is_some() { out.push(RawLine { file: rc.clone(), line: i + 1, text: "}".into() }); }
        } else {
            out.push(RawLine { file: rc.clone(), line: i + 1, text: l.to_string() });
        }
    }
    Ok(())
}

pub fn build(lines: &[RawLine], macros: &mut Macros, errs: &mut Vec<Diagnostic>) -> Vec<Node> {
    let mut i = 0;
    let (nodes, stop) = block(lines, &mut i, macros, errs, &[]);
    debug_assert!(stop.is_none());
    nodes
}

type Stop = Option<(String, String, usize)>;

fn block(lines: &[RawLine], i: &mut usize, macros: &mut Macros, errs: &mut Vec<Diagnostic>, stops: &[&str]) -> (Vec<Node>, Stop) {
    let mut nodes = vec![];
    while *i < lines.len() {
        let rl = &lines[*i];
        *i += 1;
        let mut st = match parse_line(rl.line, &rl.text) {
            Ok(s) => s,
            Err(e) => { errs.push(Diagnostic::error("E101", &rl.file, rl.line, e)); continue }
        };
        st.file = rl.file.clone();
        if let Body::Dir(d, a) = &st.body {
            let (d, a) = (d.clone(), a.clone());
            if stops.contains(&d.as_str()) { return (nodes, Some((d, a, rl.line))); }
            if st.label.is_some() && matches!(d.as_str(), ".if" | ".ifdef" | ".ifndef" | ".repeat" | ".for" | ".struct" | ".enum") {
                // Label vor einem Blockbeginn bleibt erhalten (z. B. `tabelle: .repeat 8`).
                nodes.push(Node::Stmt(Stmt { body: Body::Empty, ..st.clone() }));
            }
            match d.as_str() {
                ".macro" => {
                    let (name, raw_params) = match a.find(|c: char| c == '(' || c.is_whitespace()) {
                        Some(p) => (a[..p].trim().to_ascii_uppercase(), { let r = a[p..].trim(); r.strip_prefix('(').unwrap_or(r).strip_suffix(')').unwrap_or(r.strip_prefix('(').unwrap_or(r)).to_string() }),
                        None => (a.trim().to_ascii_uppercase(), String::new()),
                    };
                    let mut params: Vec<String> = vec![];
                    let mut defaults: Vec<Option<String>> = vec![];
                    for p in split_top(&raw_params, ',').into_iter().filter(|p| !p.is_empty()) {
                        match p.split_once('=') {
                            Some((n, d)) => { params.push(n.trim().to_string()); defaults.push(Some(d.trim().to_string())); }
                            None => {
                                if defaults.last().map_or(false, Option::is_some) { errs.push(Diagnostic::error("E210", &rl.file, rl.line, format!("Parameter '{p}' ohne Standardwert steht hinter einem Parameter mit Standardwert"))); }
                                params.push(p.trim().to_string()); defaults.push(None);
                            }
                        }
                    }
                    let mut body = vec![];
                    let mut closed = false;
                    while *i < lines.len() {
                        let l = &lines[*i];
                        *i += 1;
                        if strip_comment(&l.text).trim().eq_ignore_ascii_case(".endmacro") { closed = true; break; }
                        body.push(l.clone());
                    }
                    if !closed { errs.push(Diagnostic::error("E210", &rl.file, rl.line, ".macro ohne .endmacro")); }
                    if name.is_empty() { errs.push(Diagnostic::error("E210", &rl.file, rl.line, "Makroname fehlt")); }
                    else { macros.insert(name, Macro { params, defaults, lines: body }); }
                    continue;
                }
                ".if" | ".ifdef" | ".ifndef" => {
                    let mut cond = match d.as_str() { ".if" => Cond::Expr(a), ".ifdef" => Cond::Defined(a.trim().to_string()), _ => Cond::NotDefined(a.trim().to_string()) };
                    let mut arms = vec![];
                    loop {
                        let (body, stop) = block(lines, i, macros, errs, &[".elseif", ".else", ".endif"]);
                        arms.push((cond, body));
                        match stop {
                            Some((s, a, _)) if s == ".elseif" => cond = Cond::Expr(a),
                            Some((s, _, _)) if s == ".else" => cond = Cond::Else,
                            Some((s, _, _)) if s == ".endif" => break,
                            _ => { errs.push(Diagnostic::error("E210", &rl.file, rl.line, ".if ohne .endif")); break }
                        }
                    }
                    nodes.push(Node::If(IfNode { line: rl.line, file: rl.file.clone(), arms }));
                    continue;
                }
                ".repeat" => {
                    let parts = split_top(&a, ',');
                    let (body, stop) = block(lines, i, macros, errs, &[".endrepeat"]);
                    if stop.is_none() { errs.push(Diagnostic::error("E210", &rl.file, rl.line, ".repeat ohne .endrepeat")); }
                    nodes.push(Node::Repeat(RepeatNode { line: rl.line, file: rl.file.clone(), count: parts.first().cloned().unwrap_or_default(),
                                                         var: parts.get(1).cloned(), body }));
                    continue;
                }
                ".struct" | ".enum" => {
                    let is_enum = d == ".enum";
                    let end = if is_enum { ".endenum" } else { ".endstruct" };
                    let mut body = vec![];
                    let mut closed = false;
                    while *i < lines.len() {
                        let l = &lines[*i];
                        *i += 1;
                        if strip_comment(&l.text).trim().eq_ignore_ascii_case(end) { closed = true; break; }
                        body.push(l.clone());
                    }
                    if !closed { errs.push(Diagnostic::error("E210", &rl.file, rl.line, format!("{d} ohne {end}"))); }
                    nodes.push(Node::Struct(StructNode { line: rl.line, file: rl.file.clone(), name: a.trim().to_string(), is_enum, body }));
                    continue;
                }
                ".for" => {
                    let spec = parse_for(&a);
                    let (body, stop) = block(lines, i, macros, errs, &[".next"]);
                    if stop.is_none() { errs.push(Diagnostic::error("E210", &rl.file, rl.line, ".for ohne .next")); }
                    match spec {
                        Ok(spec) => nodes.push(Node::For(ForNode { line: rl.line, file: rl.file.clone(), spec, body })),
                        Err(e) => errs.push(Diagnostic::error("E208", &rl.file, rl.line, e)),
                    }
                    continue;
                }
                ".elseif" | ".else" | ".endif" | ".endrepeat" | ".endmacro" | ".next" | ".endstruct" | ".endenum" => {
                    errs.push(Diagnostic::error("E210", &rl.file, rl.line, format!("unerwartetes {d}")));
                    continue;
                }
                _ => {}
            }
        }
        nodes.push(Node::Stmt(st));
    }
    (nodes, None)
}

/// `.for i = 0, i < 10, i += 1`  oder  `.for i in range(a[, b[, schritt]])`
fn parse_for(a: &str) -> Result<ForSpec, String> {
    let t = a.trim();
    if let Some((var, rest)) = t.split_once(char::is_whitespace) {
        let rest = rest.trim_start();
        if let Some(r) = rest.strip_prefix("in").filter(|r| r.starts_with(char::is_whitespace)) {
            if r.trim().is_empty() { return Err(".for … in erwartet eine Liste oder range(…)".into()); }
            return Ok(ForSpec::In { var: var.to_string(), expr: r.trim().to_string() });
        }
    }
    let p = split_top(t, ',');
    if p.len() != 3 || p.iter().any(|x| x.is_empty()) { return Err(".for erwartet: .for anfang, bedingung, schritt (z. B. .for i = 0, i < 8, i += 1)".into()); }
    Ok(ForSpec::C { init: p[0].clone(), cond: p[1].clone(), step: p[2].clone() })
}

fn is_ident(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Ersetzt Makroparameter (ganze Bezeichner) und `@name` (eindeutiges lokales Label je Expansion).
pub fn substitute(text: &str, params: &[String], args: &[String], counter: usize) -> String {
    let c: Vec<char> = text.chars().collect();
    let (mut out, mut i, mut quote) = (String::new(), 0, false);
    while i < c.len() {
        let ch = c[i];
        if ch == '"' { quote = !quote; out.push(ch); i += 1; continue; }
        if quote { out.push(ch); i += 1; continue; }
        if ch == ';' { out.extend(&c[i..]); break; }
        if ch == '@' && c.get(i + 1).map_or(false, |x| is_ident(*x)) {
            let st = i + 1;
            i = st;
            while i < c.len() && is_ident(c[i]) { i += 1; }
            out.push_str(&format!(".__m{counter}_{}", c[st..i].iter().collect::<String>()));
        } else if ch.is_alphabetic() || ch == '_' {
            let st = i;
            while i < c.len() && is_ident(c[i]) { i += 1; }
            let w: String = c[st..i].iter().collect();
            match params.iter().position(|p| *p == w) {
                Some(k) => out.push_str(args.get(k).map(String::as_str).unwrap_or("")),
                None => out.push_str(&w),
            }
        } else { out.push(ch); i += 1; }
    }
    out
}
