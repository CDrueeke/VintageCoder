// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeilenparser: Label, Zuweisung, Direktive oder Instruktion.
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Body {
    Empty,
    Assign(String, String),
    Dir(String, String),
    Ins(String, String),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub file: Rc<str>,
    pub line: usize,
    pub label: Option<String>,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    None,
    Acc,
    Imm(String),
    Direct(String, Option<char>),
    IndX(String),
    IndY(String),
    Ind(String),
    /// 45GS02: `($nn),z`, `($nn,sp),y`, `[$nn]` / `[$nn],z` (32-Bit-Zeiger), `a,b` (BBR/BBS: Zeropage, Sprungziel)
    IndZ(String),
    IndSpY(String),
    Ind32(String),
    Pair(String, String),
}

pub fn strip_comment(s: &str) -> &str {
    let mut q: Option<char> = None;
    for (i, c) in s.char_indices() {
        match q {
            Some(qc) if c == qc => q = None,
            Some(_) => {}
            None if c == '"' => q = Some('"'),
            None if c == '\'' => q = Some('\''),
            None if c == ';' => return &s[..i],
            None if c == '/' && s[i + 1..].starts_with('/') => return &s[..i],
            None => {}
        }
    }
    s
}

fn is_ident_char(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' }

pub fn parse_line(line_no: usize, raw: &str) -> Result<Stmt, String> {
    let mut s = strip_comment(raw).trim();
    let mut label = None;
    // anonymes Label: einzelnes `+` oder `-` am Zeilenanfang (danach Leerraum oder Zeilenende)
    if let Some(c) = s.chars().next().filter(|c| *c == '+' || *c == '-') {
        let rest = &s[1..];
        if rest.is_empty() || rest.starts_with(char::is_whitespace) { label = Some(c.to_string()); s = rest.trim(); }
    }
    let end = if label.is_some() { 0 } else { s.find(|c: char| !is_ident_char(c)).unwrap_or(s.len()) };
    if end > 0 && s[end..].starts_with(':') && !s.chars().next().unwrap().is_ascii_digit() {
        label = Some(s[..end].to_string());
        s = s[end + 1..].trim();
    }
    if s.is_empty() { return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Empty }); }
    // Zuweisung  NAME = expr
    if label.is_none() && end > 0 {
        let rest = s[end..].trim_start();
        if rest.starts_with('=') && !rest.starts_with("==") {
            return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Assign(s[..end].to_string(), rest[1..].trim().to_string()) });
        }
        // x += e  usw.  →  x = x + (e)   (nur für Variablen sinnvoll)
        for op in ["<<=", ">>=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^="] {
            if let Some(r) = rest.strip_prefix(op) {
                let name = &s[..end];
                return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Assign(name.to_string(), format!("{name} {} ({})", &op[..op.len() - 1], r.trim())) });
            }
        }
    }
    // `*= adresse` (auch `* = adresse`) ist Kurzform von `.org adresse`
    if let Some(r) = s.strip_prefix('*') { let r = r.trim_start(); if r.starts_with('=') && !r.starts_with("==") { return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Dir(".org".into(), r[1..].trim().to_string()) }); } }
    if let Some(r) = s.strip_prefix('{') { return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Dir(".scope".into(), r.trim().to_string()) }); }
    if let Some(r) = s.strip_prefix('}') { return Ok(Stmt { file: Rc::from(""), line: line_no, label, body: Body::Dir(".endscope".into(), r.trim().to_string()) }); }
    let wend = s.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(s.len());
    let (word, args) = (&s[..wend], s[wend..].trim());
    let body = if word.starts_with('.') { Body::Dir(word.to_ascii_lowercase(), args.to_string()) }
               else { Body::Ins(word.to_ascii_uppercase(), args.to_string()) };
    Ok(Stmt { file: Rc::from(""), line: line_no, label, body })
}

/// Teilt an Kommas außerhalb von Klammern und Anführungszeichen.
pub fn split_top(s: &str, sep: char) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut q) = (vec![], String::new(), 0i32, None::<char>);
    for c in s.chars() {
        match q {
            Some(qc) => { cur.push(c); if c == qc { q = None; } }
            None => match c {
                '"' | '\'' => { q = Some(c); cur.push(c); }
                '(' | '[' => { depth += 1; cur.push(c); }
                ')' | ']' => { depth -= 1; cur.push(c); }
                c if c == sep && depth == 0 => out.push(std::mem::take(&mut cur)),
                _ => cur.push(c),
            },
        }
    }
    if !cur.trim().is_empty() || !out.is_empty() { out.push(cur); }
    out.into_iter().map(|x| x.trim().to_string()).collect()
}

fn match_paren(s: &str) -> Option<usize> {
    let mut d = 0;
    for (i, c) in s.char_indices() {
        match c { '(' => d += 1, ')' => { d -= 1; if d == 0 { return Some(i); } } _ => {} }
    }
    None
}

pub fn parse_operand(s: &str) -> Result<Operand, String> {
    let s = s.trim();
    if s.is_empty() { return Ok(Operand::None); }
    if s.eq_ignore_ascii_case("a") || s.eq_ignore_ascii_case("q") { return Ok(Operand::Acc); }
    if s.starts_with('[') {
        if let Some(close) = s.find(']') {
            let rest: String = s[close + 1..].chars().filter(|c| !c.is_whitespace()).collect();
            if rest.is_empty() || rest.eq_ignore_ascii_case(",z") { return Ok(Operand::Ind32(s[1..close].trim().to_string())); }
        }
        return Err("ungültiger 32-Bit-Zeiger".into());
    }
    if let Some(r) = s.strip_prefix('#') { return Ok(Operand::Imm(r.trim().to_string())); }
    if s.starts_with('(') {
        if let Some(close) = match_paren(s) {
            let inner = s[1..close].trim();
            let rest = s[close + 1..].trim();
            if rest.is_empty() {
                let parts = split_top(inner, ',');
                if parts.len() == 2 && parts[1].eq_ignore_ascii_case("x") { return Ok(Operand::IndX(parts[0].clone())); }
                if parts.len() == 2 && parts[1].eq_ignore_ascii_case("sp") { return Err("($nn,sp) braucht ,y dahinter".into()); }
                if parts.len() == 1 { return Ok(Operand::Ind(inner.to_string())); }
                return Err("ungültiger indirekter Operand".into());
            }
            let r2: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
            if r2.eq_ignore_ascii_case(",y") {
                let parts = split_top(inner, ',');
                if parts.len() == 2 && parts[1].eq_ignore_ascii_case("sp") { return Ok(Operand::IndSpY(parts[0].clone())); }
                return Ok(Operand::IndY(inner.to_string()));
            }
            if r2.eq_ignore_ascii_case(",z") { return Ok(Operand::IndZ(inner.to_string())); }
        }
    }
    let parts = split_top(s, ',');
    match parts.len() {
        1 => Ok(Operand::Direct(parts[0].clone(), None)),
        2 if parts[1].eq_ignore_ascii_case("x") => Ok(Operand::Direct(parts[0].clone(), Some('x'))),
        2 if parts[1].eq_ignore_ascii_case("y") => Ok(Operand::Direct(parts[0].clone(), Some('y'))),
        2 => Ok(Operand::Pair(parts[0].clone(), parts[1].clone())),
        _ => Err("ungültiger Operand".into()),
    }
}
