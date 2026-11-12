// SPDX-License-Identifier: GPL-3.0-or-later
//! Atari BASIC (Speicherabbild-Backend): Tokenizer, SAVE-Datei (`.BAS`) schreiben/lesen, Listing.
//! Das Format wurde gegen das echte BASIC-ROM (atari800) abgeglichen.
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use vc_core::Diagnostic;
use vc_target::parse_num;

#[derive(Debug, Clone)]
pub struct AtariDialect {
    pub id: String,
    pub statements: Vec<(String, u8)>,
    pub functions: Vec<(String, u8)>,
    ops: HashMap<String, u8>,
    pub lomem_offset: u16,
    /// Turbo BASIC XL: erweiterte Token, Labels (PROC/EXEC), bis 256 Variablen
    pub turbo: bool,
}

impl AtariDialect {
    pub fn load(data_dir: &Path, id: &str) -> Result<AtariDialect, String> {
        let path = data_dir.join("basic").join(format!("{id}.dialect.json"));
        let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        if v["family"] != "atari" { return Err(format!("{}: kein Atari-Dialekt", path.display())); }
        let rows = |key: &str| -> Result<Vec<(String, u8)>, String> {
            v[key].as_array().ok_or(format!("{key} fehlt"))?.iter()
                .map(|r| Ok((r[0].as_str().ok_or("Name fehlt")?.to_string(), r[1].as_str().and_then(parse_num).ok_or("Code ungültig")? as u8))).collect()
        };
        let ops = v["ops"].as_object().ok_or("ops fehlt")?.iter().map(|(k, c)| (k.clone(), c.as_str().and_then(parse_num).unwrap_or(0) as u8)).collect();
        Ok(AtariDialect { id: id.into(), statements: rows("statements")?, functions: rows("functions")?, ops,
                          lomem_offset: v["lomemOffset"].as_str().and_then(parse_num).unwrap_or(0x100) as u16, turbo: v["turbo"].as_bool().unwrap_or(false) })
    }
    fn op(&self, n: &str) -> u8 { self.ops[n] }
    #[allow(dead_code)]
    fn has_op(&self, n: &str) -> bool { self.ops.contains_key(n) }
    fn statement(&self, n: &str) -> Option<u8> { self.statements.iter().find(|(s, _)| s == n).map(|(_, c)| *c) }
    fn function(&self, n: &str) -> Option<u8> { self.functions.iter().find(|(s, _)| s == n).map(|(_, c)| *c) }
}

// ------------------------------------------------------------------ BCD-Zahlen (6 Byte)

/// Wandelt ein Dezimalliteral (`12`, `3.5`, `.5`, `1E3`) in das 6-Byte-BCD-Format des Atari.
pub fn bcd_encode(lit: &str) -> Result<[u8; 6], String> {
    let s = lit.trim().to_ascii_uppercase();
    let (mant, exp) = match s.split_once('E') { Some((m, e)) => (m.to_string(), e.parse::<i32>().map_err(|_| format!("ungültiger Exponent in '{lit}'"))?), None => (s.clone(), 0) };
    let (ip, fp) = mant.split_once('.').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((mant.clone(), String::new()));
    if !ip.chars().chain(fp.chars()).all(|c| c.is_ascii_digit()) || (ip.is_empty() && fp.is_empty()) { return Err(format!("ungültige Zahl '{lit}'")); }
    let mut digits: String = format!("{ip}{fp}");
    let mut e10 = ip.len() as i32 + exp;
    let lead = digits.chars().take_while(|c| *c == '0').count();
    digits = digits[lead..].to_string();
    e10 -= lead as i32;
    if digits.is_empty() { return Ok([0; 6]); }
    if !(-98..=98).contains(&e10) { return Err(format!("Zahl '{lit}' außerhalb des Bereichs (max. 9,99…E97)")); }
    let (pairs, x) = if e10 % 2 == 0 { (digits.clone(), e10 / 2 - 1) } else { (format!("0{digits}"), (e10 - 1) / 2) };
    let mut p: Vec<u8> = pairs.as_bytes().chunks(2).map(|c| { let d = |b: u8| b - b'0'; d(c[0]) * 10 + c.get(1).map_or(0, |b| d(*b)) }).collect();
    if p.len() > 5 {                                     // auf 10 Stellen runden
        let round_up = p[5] >= 50;
        p.truncate(5);
        if round_up { let mut i = 4; loop { p[i] += 1; if p[i] == 100 { p[i] = 0; if i == 0 { p.insert(0, 1); p.truncate(5); return Ok(bcd_from(x + 1, &p)); } i -= 1; } else { break; } } }
    }
    while p.len() < 5 { p.push(0); }
    if !(-64..=62).contains(&x) { return Err(format!("Zahl '{lit}' außerhalb des Bereichs")); }
    Ok(bcd_from(x, &p))
}

fn bcd_from(x: i32, p: &[u8]) -> [u8; 6] {
    let mut b = [0u8; 6];
    b[0] = (0x40 + x) as u8;
    for i in 0..5 { b[i + 1] = ((p[i] / 10) << 4) | (p[i] % 10); }
    b
}

/// BCD → kürzeste Dezimaldarstellung.
pub fn bcd_text(b: &[u8]) -> String {
    if b[0] & 0x7F == 0 && b[1..6].iter().all(|x| *x == 0) { return "0".into(); }
    let neg = b[0] & 0x80 != 0;
    let x = (b[0] & 0x7F) as i32 - 0x40;
    let s: String = b[1..6].iter().map(|v| format!("{}{}", v >> 4, v & 15)).collect();
    let s = s.trim_start_matches('0').to_string();
    let trailing = s.len() - s.trim_end_matches('0').len();
    let sig = s.trim_end_matches('0').to_string();
    let e = 2 * x - 8 + trailing as i32;
    let body = if e >= 0 {
        if sig.len() as i32 + e <= 10 { format!("{sig}{}", "0".repeat(e as usize)) } else { format!("{}.{}E{}", &sig[..1], &sig[1..], e + sig.len() as i32 - 1).replace(".E", "E") }
    } else if (-e as usize) < sig.len() { format!("{}.{}", &sig[..sig.len() - (-e) as usize], &sig[sig.len() - (-e) as usize..]) }
    else if (-e as usize) - sig.len() < 5 { format!(".{}{}", "0".repeat((-e) as usize - sig.len()), sig) }
    else { format!("{}.{}E{}", &sig[..1], &sig[1..], e + sig.len() as i32 - 1).replace(".E", "E") };
    if neg { format!("-{body}") } else { body }
}

// ------------------------------------------------------------------ Tokenizer

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ty { Num, Str }

enum End { Normal, Raw, Then }

struct Parser<'a> { ch: Vec<char>, i: usize, d: &'a AtariDialect, vars: &'a mut Vec<String>, raw: HashMap<String, String> }

const CONTROL: &[(&str, u8)] = &[("clr", 0x7D), ("up", 0x1C), ("down", 0x1D), ("left", 0x1E), ("right", 0x1F), ("esc", 0x1B), ("tab", 0x7F), ("del", 0x7E), ("eol", 0x9B), ("bell", 0xFD)];

fn atascii_string(inner: &str) -> Result<Vec<u8>, String> {
    let c: Vec<char> = inner.chars().collect();
    let (mut out, mut i) = (vec![], 0);
    while i < c.len() {
        if c[i] == '{' {
            let end = c[i..].iter().position(|x| *x == '}').ok_or("'}' fehlt")? + i;
            let name: String = c[i + 1..end].iter().collect::<String>().to_ascii_lowercase();
            out.push(if let Some(h) = name.strip_prefix('$') { u8::from_str_radix(h, 16).map_err(|_| format!("ungültiger Code {{{name}}}"))? }
                     else { CONTROL.iter().find(|(n, _)| *n == name).map(|(_, v)| *v).ok_or(format!("unbekannter Steuercode {{{name}}}"))? });
            i = end + 1;
        } else if (c[i] as u32) < 0x7F && (c[i] as u32) >= 0x20 { out.push(c[i] as u8); i += 1; }
        else { return Err(format!("Zeichen '{}' nicht im ATASCII-Bereich (nutze {{$xx}})", c[i])); }
    }
    Ok(out)
}

impl Parser<'_> {
    fn ws(&mut self) { while self.i < self.ch.len() && self.ch[self.i] == ' ' { self.i += 1; } }
    fn peek(&mut self) -> Option<char> { self.ws(); self.ch.get(self.i).copied() }
    fn eat(&mut self, c: char) -> bool { if self.peek() == Some(c) { self.i += 1; true } else { false } }
    fn rest(&self) -> String { self.ch[self.i..].iter().collect() }

    fn ident(&mut self) -> Option<String> {
        self.ws();
        let st = self.i;
        if !self.ch.get(st).map_or(false, |c| c.is_ascii_alphabetic()) { return None; }
        while self.i < self.ch.len() && self.ch[self.i].is_ascii_alphanumeric() { self.i += 1; }
        if self.ch.get(self.i) == Some(&'$') { self.i += 1; }
        let orig: String = self.ch[st..self.i].iter().collect();
        let up = orig.to_ascii_uppercase();
        self.raw.entry(up.clone()).or_insert(orig);
        Some(up)
    }

    fn accept_word(&mut self, w: &str) -> bool {
        let save = self.i;
        match self.ident() { Some(id) if id == w => true, _ => { self.i = save; false } }
    }

    /// Variablen-Token: `$80 + Nummer`; Turbo BASIC XL kodiert die Nummern 128..255 als `$00, Nummer - 128`.
    fn var_token(&mut self, name: &str) -> Result<Vec<u8>, String> {
        // Namen vergleicht man ohne Rücksicht auf Groß-/Kleinschreibung; Turbo BASIC XL speichert die Schreibweise des ersten Vorkommens
        let idx = match self.vars.iter().position(|v| v.eq_ignore_ascii_case(name)) {
            Some(p) => p,
            None => {
                let suffix_at = name.find(|c| matches!(c, '$' | '(' | '#')).unwrap_or(name.len());
                let spelled = if !self.d.turbo { name.to_string() }
                    else if let Some(r) = self.raw.get(name) { r.clone() }          // z. B. "B$" → "b$"
                    else { format!("{}{}", self.raw.get(&name[..suffix_at]).map(String::as_str).unwrap_or(&name[..suffix_at]), &name[suffix_at..]) };
                self.vars.push(spelled);
                self.vars.len() - 1
            }
        };
        if idx < 128 { Ok(vec![0x80 + idx as u8]) }
        else if self.d.turbo && idx < 256 { Ok(vec![0x00, (idx - 128) as u8]) }
        else { Err(format!("mehr als {} Variablen", if self.d.turbo { 256 } else { 128 })) }
    }
    /// Label (Turbo BASIC XL): eigene Namensklasse, im Namensverzeichnis mit `#` markiert
    fn label_token(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let id = self.ident().ok_or("Label erwartet")?;
        if id.ends_with('$') { return Err(format!("'{id}' ist kein gültiger Labelname")); }
        out.extend(self.var_token(&format!("{id}#"))?);
        Ok(())
    }

    fn number(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let st = self.i;
        while self.i < self.ch.len() && self.ch[self.i].is_ascii_digit() { self.i += 1; }
        if self.ch.get(self.i) == Some(&'.') { self.i += 1; while self.i < self.ch.len() && self.ch[self.i].is_ascii_digit() { self.i += 1; } }
        if matches!(self.ch.get(self.i), Some('E') | Some('e')) {
            let mut j = self.i + 1;
            if matches!(self.ch.get(j), Some('+') | Some('-')) { j += 1; }
            if self.ch.get(j).map_or(false, char::is_ascii_digit) { while self.ch.get(j).map_or(false, char::is_ascii_digit) { j += 1; } self.i = j; }
        }
        let text: String = self.ch[st..self.i].iter().collect();
        out.push(self.d.op("numConst"));
        out.extend(bcd_encode(&text)?);
        Ok(())
    }

    fn subscript(&mut self, out: &mut Vec<u8>, open: u8) -> Result<(), String> {
        out.push(open);
        self.expr(out)?;
        if self.eat(',') { out.push(self.d.op("subComma")); self.expr(out)?; }
        if !self.eat(')') { return Err("')' fehlt".into()); }
        out.push(self.d.op("rparen"));
        Ok(())
    }

    fn variable(&mut self, out: &mut Vec<u8>, id: &str) -> Result<Ty, String> {
        if id.ends_with('$') {
            let t = self.var_token(id)?;
            out.extend(t);
            if self.peek() == Some('(') { self.i += 1; self.subscript(out, self.d.op("strLparen"))?; }
            Ok(Ty::Str)
        } else if self.peek() == Some('(') {
            self.i += 1;
            let t = self.var_token(&format!("{id}("))?;
            out.extend(t);
            self.subscript(out, self.d.op("arrLparen"))?;
            Ok(Ty::Num)
        } else { let t = self.var_token(id)?; out.extend(t); Ok(Ty::Num) }
    }

    /// Operand mit Zeichenkettenvergleich: `A$="x"` ist ein Operand (wie in der Grammatik von Turbo BASIC XL), egal wo er in einem Ausdruck steht
    fn primary(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let t = self.primary0(out)?;
        if t == Ty::Str {
            self.ws();
            let two: String = self.ch[self.i..(self.i + 2).min(self.ch.len())].iter().collect();
            let found = match two.as_str() { "<=" => Some((2, "sle")), "<>" => Some((2, "sne")), ">=" => Some((2, "sge")), _ => match self.ch.get(self.i) { Some('<') => Some((1, "slt")), Some('>') => Some((1, "sgt")), Some('=') => Some((1, "seq")), _ => None } };
            if let Some((len, name)) = found {
                self.i += len;
                out.push(self.d.op(name));
                if self.primary0(out)? != Ty::Str { return Err("Zeichenkette nach dem Vergleichsoperator erwartet".into()); }
                return Ok(Ty::Num);
            }
        }
        Ok(t)
    }

    fn primary0(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let c = self.peek().ok_or("Ausdruck unvollständig")?;
        if c.is_ascii_digit() || (c == '.' && self.ch.get(self.i + 1).map_or(false, char::is_ascii_digit)) { self.number(out)?; return Ok(Ty::Num); }
        if c == '$' && self.ch.get(self.i + 1).map_or(false, char::is_ascii_hexdigit) {
            let st = self.i + 1;
            let mut j = st;
            while j < self.ch.len() && self.ch[j].is_ascii_hexdigit() { j += 1; }
            let v = u32::from_str_radix(&self.ch[st..j].iter().collect::<String>(), 16).map_err(|_| "Hexzahl zu groß")?;
            self.i = j;
            out.push(self.d.op(if self.d.turbo { "hexConst" } else { "numConst" }));
            out.extend(bcd_encode(&v.to_string())?);
            return Ok(Ty::Num);
        }
        if c == '%' && matches!(self.ch.get(self.i + 1), Some('0'..='3')) {
            let n = self.ch[self.i + 1] as u8 - b'0';
            self.i += 2;
            if self.d.turbo { out.push(self.d.op(&format!("per{n}"))); } else { out.push(self.d.op("numConst")); out.extend(bcd_encode(&n.to_string())?); }
            return Ok(Ty::Num);
        }
        if c == '"' {
            let st = self.i + 1;
            let mut j = st;
            while j < self.ch.len() && self.ch[j] != '"' { j += 1; }
            let b = atascii_string(&self.ch[st..j].iter().collect::<String>())?;
            if b.len() > 255 { return Err("Zeichenkette länger als 255 Zeichen".into()); }
            out.push(self.d.op("strConst"));
            out.push(b.len() as u8);
            out.extend(b);
            self.i = (j + 1).min(self.ch.len());
            return Ok(Ty::Str);
        }
        if c == '(' {
            self.i += 1;
            out.push(self.d.op("lparen"));
            let t = self.expr(out)?;
            if !self.eat(')') { return Err("')' fehlt".into()); }
            out.push(self.d.op("rparen"));
            return Ok(t);
        }
        let id = self.ident().ok_or(format!("unerwartetes Zeichen '{c}'"))?;
        if let Some(code) = self.d.function(&id) {
            if self.peek() == Some('(') {
                self.i += 1;
                out.push(code);
                out.push(self.d.op("fnLparen"));
                self.expr(out)?;
                while self.eat(',') { out.push(self.d.op("subComma")); self.expr(out)?; }
                if !self.eat(')') { return Err("')' fehlt".into()); }
                out.push(self.d.op("rparen"));
                return Ok(if matches!(id.as_str(), "STR$" | "CHR$" | "HEX$") { Ty::Str } else { Ty::Num });
            }
            // Turbo BASIC XL: Funktionen ohne Klammern
            if self.d.turbo {
                match id.as_str() {
                    "INKEY$" | "TIME$" => { out.push(code); return Ok(Ty::Str); }
                    "TIME" | "ERR" | "ERL" => { out.push(code); return Ok(Ty::Num); }
                    "RND" => { out.push(self.d.function("RND0").ok_or("RND0 fehlt")?); return Ok(Ty::Num); }
                    _ => {}
                }
            }
        }
        if self.d.statement(&id).is_some() || ["TO", "STEP", "THEN", "AND", "OR", "NOT"].contains(&id.as_str()) { return Err(format!("reserviertes Wort '{id}' an dieser Stelle")); }
        self.variable(out, &id)
    }

    fn unary(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        // NOT darf auch hinter einem Vergleichsoperator stehen (`0<NOT A`); der Tokenizer gibt die Token in Quelltextreihenfolge aus
        if self.accept_word("NOT") { out.push(self.d.op("not")); self.unary(out)?; return Ok(Ty::Num); }
        match self.peek() {
            Some('-') => { self.i += 1; out.push(self.d.op("uminus")); self.unary(out) }
            Some('+') => { self.i += 1; out.push(self.d.op("uplus")); self.unary(out) }
            _ => self.primary(out),
        }
    }
    fn pow(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.unary(out)?;
        while self.eat('^') { out.push(self.d.op("pow")); self.unary(out)?; t = Ty::Num; }
        Ok(t)
    }
    fn mul(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.pow(out)?;
        loop {
            let name = match self.peek() {
                Some('*') => { self.i += 1; "mul" }
                Some('/') => { self.i += 1; "div" }
                Some('&') if self.d.turbo => { self.i += 1; "andBit" }
                Some('!') if self.d.turbo => { self.i += 1; "orBit" }
                _ if self.d.turbo && self.accept_word("DIV") => "idiv",
                _ if self.d.turbo && self.accept_word("MOD") => "mod",
                _ if self.d.turbo && self.accept_word("EXOR") => "exor",
                _ => break,
            };
            out.push(self.d.op(name));
            self.pow(out)?;
            t = Ty::Num;
        }
        Ok(t)
    }
    fn add(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.mul(out)?;
        loop {
            let name = match self.peek() { Some('+') => "add", Some('-') => "sub", _ => break };
            self.i += 1;
            out.push(self.d.op(name));
            self.mul(out)?;
            t = Ty::Num;
        }
        Ok(t)
    }
    fn rel(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.add(out)?;
        loop {
            self.ws();
            let two: String = self.ch[self.i..(self.i + 2).min(self.ch.len())].iter().collect();
            let (len, name) = match two.as_str() { "<=" => (2, "le"), "<>" => (2, "ne"), ">=" => (2, "ge"), _ => match self.ch.get(self.i) { Some('<') => (1, "lt"), Some('>') => (1, "gt"), Some('=') => (1, "eq"), _ => break } };
            self.i += len;
            let name = if t == Ty::Str { format!("s{name}") } else { name.to_string() };
            out.push(self.d.op(&name));
            self.add(out)?;
            t = Ty::Num;
        }
        Ok(t)
    }
    fn not(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        if self.accept_word("NOT") { out.push(self.d.op("not")); self.not(out)?; Ok(Ty::Num) } else { self.rel(out) }
    }
    fn and(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.not(out)?;
        while self.accept_word("AND") { out.push(self.d.op("and")); self.not(out)?; t = Ty::Num; }
        Ok(t)
    }
    fn expr(&mut self, out: &mut Vec<u8>) -> Result<Ty, String> {
        let mut t = self.and(out)?;
        while self.accept_word("OR") { out.push(self.d.op("or")); self.and(out)?; t = Ty::Num; }
        Ok(t)
    }

    fn list(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        loop {
            match self.peek() {
                None | Some(':') => return Ok(()),
                Some('#') => { self.i += 1; out.push(self.d.op("hash")); }
                Some(',') => { self.i += 1; out.push(self.d.op("comma")); }
                Some(';') => { self.i += 1; out.push(self.d.op("semicolon")); }
                _ => { self.expr(out)?; }
            }
        }
    }

    fn assignment(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let id = self.ident().ok_or("Variable erwartet")?;
        let t = self.variable(out, &id)?;
        if !self.eat('=') { return Err(format!("'=' nach '{id}' erwartet")); }
        out.push(self.d.op(if t == Ty::Str { "assignStr" } else { "assignNum" }));
        self.expr(out)?;
        Ok(())
    }

    /// Turbo BASIC XL: Anweisungen, die nicht mit einem Namen beginnen (`'`, `.`, `--`, `*F`, `-MOVE`, `#label`, `%PUT`, `%GET`)
    fn turbo_special(&mut self) -> Result<Option<(Vec<u8>, End)>, String> {
        let c = self.peek().ok_or("Anweisung erwartet")?;
        let next = self.ch.get(self.i + 1).copied().unwrap_or(' ');
        let word_follows = |p: &Self, off: usize, w: &str| -> bool {
            let s: String = p.ch[(p.i + off).min(p.ch.len())..].iter().take(w.len()).collect();
            s.eq_ignore_ascii_case(w) && !p.ch.get(p.i + off + w.len()).map_or(false, |c| c.is_ascii_alphanumeric())
        };
        let rem = |p: &mut Self, skip: usize| -> (Vec<u8>, End) {
            let text: String = p.ch[p.i + skip..].iter().collect();
            let mut out = vec![0x00];
            out.extend(text.trim_start().chars().map(|c| c as u8));
            out.push(0x9B);
            p.i = p.ch.len();
            (out, End::Raw)
        };
        if c == '\'' { return Ok(Some(rem(self, 1))); }
        if c == '.' && !next.is_ascii_digit() { return Ok(Some(rem(self, 1))); }
        if c == '-' && next == '-' {
            self.i = self.ch.len();
            return Ok(Some((vec![self.d.statement("--").ok_or("-- fehlt")?], End::Raw)));   // `--`-Kommentare werden nicht gespeichert
        }
        if c == '*' && matches!(next.to_ascii_uppercase(), 'F' | 'B' | 'L') && !self.ch.get(self.i + 2).map_or(false, |c| c.is_ascii_alphanumeric()) {
            let name = format!("*{}", next.to_ascii_uppercase());
            self.i += 2;
            let mut out = vec![self.d.statement(&name).ok_or("Flag-Anweisung fehlt")?];
            match self.peek() { Some('+') => { self.i += 1; out.push(self.d.op("add")); } Some('-') => { self.i += 1; out.push(self.d.op("sub")); } _ => {} }
            return Ok(Some((out, End::Normal)));
        }
        if c == '-' && word_follows(self, 1, "MOVE") {
            self.i += 5;
            let mut out = vec![self.d.statement("-MOVE").ok_or("-MOVE fehlt")?];
            self.list(&mut out)?;
            return Ok(Some((out, End::Normal)));
        }
        if c == '#' {
            self.i += 1;
            let mut out = vec![self.d.statement("#").ok_or("# fehlt")?];
            self.label_token(&mut out)?;
            return Ok(Some((out, End::Normal)));
        }
        if c == '%' && (word_follows(self, 1, "PUT") || word_follows(self, 1, "GET")) {
            let name = if word_follows(self, 1, "PUT") { "%PUT" } else { "%GET" };
            self.i += 4;
            let mut out = vec![self.d.statement(name).ok_or("%PUT/%GET fehlt")?];
            self.list(&mut out)?;
            return Ok(Some((out, End::Normal)));
        }
        Ok(None)
    }

    fn statement(&mut self) -> Result<(Vec<u8>, End), String> {
        let mut out = vec![];
        self.ws();
        if self.eat('?') { out.push(self.d.statement("?").unwrap()); self.list(&mut out)?; return Ok((out, End::Normal)); }   // `?` hat ein eigenes Token ($28) und bleibt im Listing ein `?`
        if self.d.turbo { if let Some(r) = self.turbo_special()? { return Ok(r); } }
        let save = self.i;
        let Some(id) = self.ident() else { return Err(format!("unerwartetes Zeichen '{}'", self.ch.get(self.i).copied().unwrap_or(' '))) };
        // Turbo BASIC XL: GO# label, TIME$= text
        if self.d.turbo && id == "GO" && self.ch.get(self.i) == Some(&'#') {
            self.i += 1;
            let mut out = vec![self.d.statement("GO#").ok_or("GO# fehlt")?];
            self.label_token(&mut out)?;
            return Ok((out, End::Normal));
        }
        if self.d.turbo && id == "TIME$" && self.peek() == Some('=') {
            self.i += 1;
            let mut out = vec![self.d.statement("TIME$=").ok_or("TIME$= fehlt")?];
            self.expr(&mut out)?;
            return Ok((out, End::Normal));
        }
        let id = if id == "GO" && self.accept_word("TO") { "GO TO".to_string() } else { id };
        // Abkürzung mit Punkt (`G.`, `PR.`): das erste Schlüsselwort der Tabelle, das so beginnt (wie im ROM)
        let id = if self.ch.get(self.i) == Some(&'.') && self.d.statement(&id).is_none() {
            match self.d.statements.iter().find(|(n, _)| n.starts_with(id.as_str()) && n.chars().all(|c| c.is_ascii_alphabetic() || c == ' ')) {
                Some((n, _)) => { self.i += 1; n.clone() }
                None => id,
            }
        } else { id };
        let Some(code) = self.d.statement(&id) else {
            self.i = save;
            out.push(self.d.op("implicitLet"));
            self.assignment(&mut out)?;
            return Ok((out, End::Normal));
        };
        out.push(code);
        match id.as_str() {
            "REM" | "DATA" => {
                let text = self.rest();
                let text = text.trim_start();
                out.extend(text.chars().map(|c| c as u8));
                out.push(0x9B);
                self.i = self.ch.len();
                return Ok((out, End::Raw));
            }
            "LET" => self.assignment(&mut out)?,
            "IF" => {
                self.expr(&mut out)?;
                if !self.accept_word("THEN") {
                    if self.d.turbo && matches!(self.peek(), None | Some(':')) { return Ok((out, End::Normal)); }   // mehrzeiliges IF … ELSE … ENDIF
                    return Err("THEN erwartet".into());
                }
                out.push(self.d.op("then"));
                if self.peek().map_or(false, |c| c.is_ascii_digit() || c == '.') { self.number(&mut out)?; return Ok((out, End::Normal)); }
                if matches!(self.peek(), None | Some(':')) { return Err("Anweisung oder Zeilennummer nach THEN erwartet".into()); }
                return Ok((out, End::Then));
            }
            "FOR" => {
                let v = self.ident().ok_or("Laufvariable erwartet")?;
                let t = self.var_token(&v)?;
                out.extend(t);
                if !self.eat('=') { return Err("'=' erwartet".into()); }
                out.push(self.d.op("assignNum"));
                self.expr(&mut out)?;
                if !self.accept_word("TO") { return Err("TO erwartet".into()); }
                out.push(self.d.op("to"));
                self.expr(&mut out)?;
                if self.accept_word("STEP") { out.push(self.d.op("step")); self.expr(&mut out)?; }
            }
            "NEXT" => { let v = self.ident().ok_or("Variable erwartet")?; let t = self.var_token(&v)?; out.extend(t); }
            "ON" => {
                self.expr(&mut out)?;
                let turbo_go = self.d.turbo && { let s = self.i; if self.accept_word("GO") && self.ch.get(self.i) == Some(&'#') { self.i += 1; true } else { self.i = s; false } };
                if turbo_go || (self.d.turbo && self.accept_word("EXEC")) {
                    out.push(self.d.op(if turbo_go { "onGoSharp" } else { "onExec" }));
                    self.label_token(&mut out)?;
                    while self.eat(',') { out.push(self.d.op("comma")); self.label_token(&mut out)?; }
                } else {
                    out.push(if self.accept_word("GOTO") { self.d.op("goto") } else if self.accept_word("GOSUB") { self.d.op("gosub") } else { return Err("GOTO oder GOSUB erwartet".into()) });
                    self.expr(&mut out)?;
                    while self.eat(',') { out.push(self.d.op("comma")); self.expr(&mut out)?; }
                }
            }
            "PROC" | "EXEC" if self.d.turbo => {
                self.label_token(&mut out)?;
                if self.peek() == Some(',') || self.peek() == Some(';') { return Err("Parameter bei PROC/EXEC sind in Turbo BASIC XL nicht vorgesehen (Variablen sind global)".into()); }
            }
            "TRAP" | "RESTORE" if self.d.turbo && self.peek() == Some('#') => {
                self.i += 1;
                out.push(self.d.op("hash"));
                self.label_token(&mut out)?;
            }
            "DIM" => loop {
                let v = self.ident().ok_or("Name erwartet")?;
                if v.ends_with('$') {
                    let t = self.var_token(&v)?;
                    out.extend(t);
                    if !self.eat('(') { return Err("'(' erwartet".into()); }
                    self.subscript(&mut out, self.d.op("dimStrLparen"))?;
                } else {
                    let t = self.var_token(&format!("{v}("))?;
                    out.extend(t);
                    if !self.eat('(') { return Err("'(' erwartet".into()); }
                    self.subscript(&mut out, self.d.op("dimArrLparen"))?;
                }
                if !self.eat(',') { break; }
                out.push(self.d.op("comma"));
            },
            _ => self.list(&mut out)?,
        }
        Ok((out, End::Normal))
    }

    fn line(&mut self) -> Result<Vec<Vec<u8>>, String> {
        let mut stmts: Vec<Vec<u8>> = vec![];
        loop {
            match self.peek() {
                None => break,
                Some(':') => { self.i += 1; continue; }
                _ => {}
            }
            let (mut body, end) = self.statement()?;
            if let End::Normal = end {
                match self.peek() {
                    Some(':') => { self.i += 1; body.push(self.d.op("colon")); }
                    None => body.push(self.d.op("eol")),
                    Some(c) => return Err(format!("unerwartetes Zeichen '{c}'")),
                }
            }
            stmts.push(body);
        }
        if stmts.is_empty() { return Err("leere Zeile".into()); }
        Ok(stmts)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtariProgram {
    /// Variablennamen in VNT-Reihenfolge (Arrays mit `(`, Strings mit `$`).
    pub vars: Vec<String>,
    /// Vollständige Zeilen: Nummer, Länge, Anweisungen.
    pub lines: Vec<(u16, Vec<u8>)>,
    /// Turbo BASIC XL (andere Direktzeile am Ende, Labels)
    pub turbo: bool,
}

fn build_line(num: u16, stmts: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let mut v = vec![(num & 0xFF) as u8, (num >> 8) as u8, 0];
    for s in stmts {
        let end = v.len() + 1 + s.len();
        if end > 255 { return Err("Zeile zu lang (mehr als 255 Bytes)".into()); }
        v.push(end as u8);
        v.extend_from_slice(s);
    }
    v[2] = v.len() as u8;
    Ok(v)
}

/// Tokenisiert ein Listing mit Zeilennummern (`10 PRINT "HI"`).
pub fn tokenize_text(src: &str, d: &AtariDialect, file: &str) -> Result<AtariProgram, Vec<Diagnostic>> {
    let (mut vars, mut lines, mut errs): (Vec<String>, Vec<(u16, Vec<u8>)>, Vec<Diagnostic>) = (vec![], vec![], vec![]);
    for (n, l) in src.lines().enumerate() {
        let t = l.trim();
        if t.is_empty() || t.starts_with(';') { continue; }
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        let Ok(num) = digits.parse::<u32>() else { errs.push(Diagnostic::error("B100", file, n + 1, "Zeile beginnt nicht mit einer Zeilennummer")); continue };
        if num > 32767 { errs.push(Diagnostic::error("B100", file, n + 1, format!("Zeilennummer {num} größer als 32767"))); continue; }
        let mut p = Parser { ch: t[digits.len()..].chars().collect(), i: 0, d, vars: &mut vars, raw: HashMap::new() };
        match p.line().and_then(|s| build_line(num as u16, &s)) {
            Ok(b) => match lines.iter().position(|(x, _)| *x == num as u16) { Some(i) => lines[i].1 = b, None => lines.push((num as u16, b)) },
            Err(e) => errs.push(Diagnostic::error("B100", file, n + 1, e)),
        }
    }
    lines.sort_by_key(|(n, _)| *n);
    if errs.is_empty() { Ok(AtariProgram { vars, lines, turbo: d.turbo }) } else { Err(errs) }
}

impl AtariProgram {
    fn vnt(&self) -> Vec<u8> {
        let mut v = vec![];
        for name in &self.vars {
            let mut b: Vec<u8> = name.trim_end_matches('#').bytes().collect();
            *b.last_mut().unwrap() |= 0x80;
            v.extend(b);
        }
        v
    }

    /// Speicherabbild ab VNTP: Variablennamen, Terminator, Wertetabelle, Anweisungen, Direktzeile.
    /// Rückgabe: (Abbild, VNTD-Offset, STMTAB-Offset, Direktzeile-Offset) relativ zu VNTP.
    pub fn image(&self) -> (Vec<u8>, usize, usize, usize) {
        let mut img = self.vnt();
        let vntd = img.len();
        img.push(0);
        for (i, name) in self.vars.iter().enumerate() {
            let ty = if name.ends_with('#') { 0xC0 } else if name.ends_with('(') { 0x40 } else if name.ends_with('$') { 0x80 } else { 0x00 };
            img.extend([ty, i as u8, 0, 0, 0, 0, 0, 0]);
        }
        let stmtab = img.len();
        for (_, l) in &self.lines { img.extend_from_slice(l); }
        let direct = img.len();
        img.extend([0x00, 0x80, 0x06, 0x06, if self.turbo { 0x34 } else { 0x15 }, 0x16]);          // Direktzeile (Atari BASIC: END; Turbo BASIC XL wie der Turbo-Tokenizer von dmsc)
        (img, vntd, stmtab, direct)
    }

    /// Inhalt einer mit SAVE erzeugten `.BAS`-Datei: 14-Byte-Kopf (Zeiger relativ zu LOMEM) + Abbild.
    pub fn save_file(&self, d: &AtariDialect) -> Vec<u8> {
        let (img, vntd, stmtab, direct) = self.image();
        let base = d.lomem_offset;
        let w = |v: usize| (base as usize + v) as u16;
        let mut f = vec![];
        for word in [0u16, base, w(vntd), w(vntd + 1), w(stmtab), w(direct), w(img.len())] { f.extend([(word & 0xFF) as u8, (word >> 8) as u8]); }
        f.extend(img);
        f
    }

    pub fn parse_save(data: &[u8], d: &AtariDialect) -> Result<AtariProgram, String> {
        if data.len() < 14 { return Err("Datei zu kurz".into()); }
        let w = |i: usize| data[i * 2] as usize | (data[i * 2 + 1] as usize) << 8;
        let (vntp, vntd, vvtp, stmtab, _cur, starp) = (w(1), w(2), w(3), w(4), w(5), w(6));
        let base = d.lomem_offset as usize;
        if vntp != base { return Err(format!("unerwarteter VNTP-Offset ${vntp:04X}")); }
        let img = &data[14..];
        if img.len() != starp - vntp { return Err(format!("Länge stimmt nicht (Kopf: {}, Datei: {})", starp - vntp, img.len())); }
        let (vnt_end, vvt, stm) = (vntd - vntp, vvtp - vntp, stmtab - vntp);
        if vnt_end > vvt || vvt > stm || stm > img.len() { return Err("Zeiger im Dateikopf ungültig".into()); }
        let mut vars = vec![];
        let mut cur = String::new();
        for &b in &img[..vnt_end] { cur.push((b & 0x7F) as char); if b & 0x80 != 0 { vars.push(std::mem::take(&mut cur)); } }
        if (stm - vvt) != vars.len() * 8 { return Err("Variablenwertetabelle passt nicht zur Namenstabelle".into()); }
        for (i, v) in vars.iter_mut().enumerate() { if img[vvt + i * 8] & 0xC0 == 0xC0 { v.push('#'); } }
        let (mut lines, mut off) = (vec![], stm);
        while off + 3 <= img.len() {
            let num = img[off] as u16 | (img[off + 1] as u16) << 8;
            let len = img[off + 2] as usize;
            if num == 0x8000 { break; }
            if len < 4 || off + len > img.len() { return Err(format!("ungültige Zeilenlänge bei ${:04X}", off)); }
            lines.push((num, img[off..off + len].to_vec()));
            off += len;
        }
        Ok(AtariProgram { vars, lines, turbo: d.turbo })
    }

    pub fn listing(&self, d: &AtariDialect) -> String {
        self.lines.iter().map(|(n, b)| format!("{n} {}\n", self.line_text(b, d))).collect()
    }

    fn line_text(&self, line: &[u8], d: &AtariDialect) -> String {
        let name_of = |idx: usize| self.vars.get(idx).map(|v| v.trim_end_matches('(').trim_end_matches('#').to_string()).unwrap_or_else(|| format!("V{idx}"));
        let op_text = |c: u8| -> Option<&'static str> {
            Some(match c {
                0x12 => ",", 0x14 => ":", 0x15 => ";", 0x1C => "#", 0x17 => " GOTO ", 0x18 => " GOSUB ", 0x19 => " TO ", 0x1A => " STEP ", 0x1B => " THEN ",
                0x1D | 0x2F => "<=", 0x1E | 0x30 => "<>", 0x1F | 0x31 => ">=", 0x20 | 0x32 => "<", 0x21 | 0x33 => ">", 0x22 | 0x2D | 0x2E | 0x34 => "=",
                0x23 => "^", 0x24 => "*", 0x25 | 0x35 => "+", 0x26 | 0x36 => "-", 0x27 => "/", 0x28 => "NOT ", 0x29 => " OR ", 0x2A => " AND ",
                0x2B | 0x37 | 0x38 | 0x39 | 0x3A | 0x3B => "(", 0x2C => ")", 0x3C => ",",
                0x56 if d.turbo => "&", 0x57 if d.turbo => "!", 0x5A if d.turbo => " EXOR ", 0x5D if d.turbo => " DIV ", 0x61 if d.turbo => " MOD ",
                0x62 if d.turbo => " EXEC ", 0x6A if d.turbo => " GO# ", 0x66 if d.turbo => "%0", 0x67 if d.turbo => "%1", 0x68 if d.turbo => "%2", 0x69 if d.turbo => "%3", _ => return None,
            })
        };
        let (mut s, mut off) = (String::new(), 3usize);
        while off < line.len() {
            let end = line[off] as usize;
            let st = &line[off + 1..end.min(line.len())];
            let mut i = 0;
            let tok = st[0];
            i += 1;
            if tok == 0x36 { /* implizites LET */ }
            else if let Some((n, _)) = d.statements.iter().find(|(_, c)| *c == tok) {
                s.push_str(n);
                if st.len() > 1 && st[1] != 0x16 && st[1] != 0x14 { s.push(' '); }
            }
            if tok == 0x00 || tok == 0x01 {
                let raw: String = st[1..].iter().take_while(|b| **b != 0x9B).map(|b| *b as char).collect();
                s.push_str(&raw);
                off = end;
                continue;
            }
            while i < st.len() {
                let c = st[i];
                i += 1;
                match c {
                    0x0E => { s.push_str(&bcd_text(&st[i..i + 6])); i += 6; }
                    0x0D if d.turbo && i + 6 <= st.len() => { let v: u32 = bcd_text(&st[i..i + 6]).parse().unwrap_or(0); s.push_str(&format!("${v:X}")); i += 6; }
                    0x00 if d.turbo && i < st.len() => { s.push_str(&name_of(128 + st[i] as usize)); i += 1; }
                    0x0F => { let n = st[i] as usize; s.push('"'); for &b in &st[i + 1..i + 1 + n] { match b { 0x20..=0x7E if b != b'"' => s.push(b as char), _ => s.push_str(&match CONTROL.iter().find(|(_, v)| *v == b) { Some((nm, _)) => format!("{{{nm}}}"), None => format!("{{${b:02x}}}") }) } } s.push('"'); i += 1 + n; }
                    0x16 => {}
                    0x80..=0xFF => s.push_str(&name_of((c - 0x80) as usize)),
                    c if d.functions.iter().any(|(_, f)| *f == c) => s.push_str(match d.functions.iter().find(|(_, f)| *f == c).unwrap().0.as_str() { "RND0" => "RND", n => n }),
                    c => if let Some(t) = op_text(c) { s.push_str(t); } else { s.push_str(&format!("{{${c:02x}}}")); },
                }
            }
            off = end;
        }
        s
    }
}
