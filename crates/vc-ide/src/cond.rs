// SPDX-License-Identifier: GPL-3.0-or-later
//! Bedingungen für Haltepunkte: kleine Ausdruckssprache über Register, Speicher und Symbole.
//!
//! `a == $10 && [$d020] != 3`, `x >= 5`, `peekw($fffe) == $ea31`, `fc && !fz`, `lin > 100`
//! Zahlen: `$ff`, `0xff`, `%1010`, Dezimal. Register: `a x y sp pc p` (Statusbyte), Flags `fn fv fb fd fi fz fc`, Zusatzwerte des
//! Emulators (`lin`, `cyc`; bei Tests `cycles` = Zyklen des Unterprogramms ohne den `jsr`), Symbole der letzten Übersetzung. Speicher: `[adr]` (Byte), `peek(adr)`, `peekw(adr)` (Wort).
//! Statische Prüfungen (`vcc test`): `size("Segment")` = Bytes des Segments (Name in Anführungszeichen, `""` = unbenannt).
//! Operatoren wie in C: `|| && | ^ & == != < <= > >= << >> + - * / % ! ~` und Klammern.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Node { Num(i64), Reg(String), Size(String), Peek(Box<Node>, bool), Un(char, Box<Node>), Bin(&'static str, Box<Node>, Box<Node>) }

#[derive(Debug, Clone, PartialEq)]
pub struct Expr { node: Node, pub source: String }

pub trait Ctx {
    fn reg(&self, name: &str) -> Option<i64>;
    fn peek(&mut self, addr: u16) -> Result<u8, String>;
    /// Größe eines Segments in Bytes (nur bei statischen Prüfungen verfügbar)
    fn size(&self, _segment: &str) -> Result<i64, String> { Err("size() steht hier nicht zur Verfügung".into()) }
}

struct P<'a> { t: Vec<String>, i: usize, syms: &'a HashMap<String, i64> }

fn tokenize(s: &str) -> Result<Vec<String>, String> {
    let c: Vec<char> = s.chars().collect(); let (mut i, mut out) = (0, vec![]);
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() { i += 1; continue; }
        if ch == '"' {
            let st = i; i += 1;
            while i < c.len() && c[i] != '"' { i += 1; }
            if i >= c.len() { return Err("Anführungszeichen nicht geschlossen".into()); }
            i += 1; out.push(c[st..i].iter().collect()); continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch == '%' && c.get(i + 1).map_or(false, |d| *d == '0' || *d == '1') && out.last().map_or(true, |t: &String| !t.chars().last().map_or(false, |l| l.is_ascii_alphanumeric() || l == ')' || l == ']')) || ch == '.' {
            let st = i; i += 1;
            while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '_' || c[i] == '.') { i += 1; }
            out.push(c[st..i].iter().collect()); continue;
        }
        let two: String = c[i..(i + 2).min(c.len())].iter().collect();
        if ["||", "&&", "==", "!=", "<=", ">=", "<<", ">>"].contains(&two.as_str()) { out.push(two); i += 2; continue; }
        if "+-*/%&|^!~<>()[],".contains(ch) { out.push(ch.to_string()); i += 1; continue; }
        return Err(format!("Zeichen '{ch}' nicht erlaubt"));
    }
    Ok(out)
}

fn number(t: &str) -> Option<i64> {
    if let Some(h) = t.strip_prefix('$').or_else(|| t.strip_prefix("0x")).or_else(|| t.strip_prefix("0X")) { return i64::from_str_radix(h, 16).ok(); }
    if let Some(b) = t.strip_prefix('%') { return i64::from_str_radix(b, 2).ok(); }
    if t.chars().all(|c| c.is_ascii_digit()) { return t.parse().ok(); }
    None
}

const LEVELS: [&[&str]; 9] = [&["||"], &["&&"], &["|"], &["^"], &["&"], &["==", "!="], &["<", "<=", ">", ">="], &["<<", ">>"], &["+", "-"]];

impl<'a> P<'a> {
    fn peek_tok(&self) -> Option<&str> { self.t.get(self.i).map(String::as_str) }
    fn eat(&mut self, s: &str) -> bool { if self.peek_tok() == Some(s) { self.i += 1; true } else { false } }
    fn level(&mut self, l: usize) -> Result<Node, String> {
        if l == LEVELS.len() { return self.mul(); }
        let mut left = self.level(l + 1)?;
        while let Some(op) = self.peek_tok().and_then(|t| LEVELS[l].iter().find(|o| **o == t).copied()) { self.i += 1; let r = self.level(l + 1)?; left = Node::Bin(op, Box::new(left), Box::new(r)); }
        Ok(left)
    }
    fn mul(&mut self) -> Result<Node, String> {
        let mut left = self.unary()?;
        while let Some(op) = self.peek_tok().and_then(|t| ["*", "/", "%"].iter().find(|o| **o == t).copied()) { self.i += 1; let r = self.unary()?; left = Node::Bin(op, Box::new(left), Box::new(r)); }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Node, String> {
        for (t, c) in [("!", '!'), ("~", '~'), ("-", '-')] { if self.eat(t) { return Ok(Node::Un(c, Box::new(self.unary()?))); } }
        self.primary()
    }
    fn primary(&mut self) -> Result<Node, String> {
        let t = self.t.get(self.i).cloned().ok_or("Ausdruck endet unerwartet")?; self.i += 1;
        if t == "(" { let e = self.level(0)?; if !self.eat(")") { return Err("')' fehlt".into()); } return Ok(e); }
        if t == "[" { let e = self.level(0)?; if !self.eat("]") { return Err("']' fehlt".into()); } return Ok(Node::Peek(Box::new(e), false)); }
        if let Some(n) = number(&t) { return Ok(Node::Num(n)); }
        if t.starts_with('"') { return Err("Text ist nur als Argument von size(\"…\") erlaubt".into()); }
        let low = t.to_ascii_lowercase();
        if (low == "peek" || low == "peekw") && self.peek_tok() == Some("(") { self.i += 1; let e = self.level(0)?; if !self.eat(")") { return Err("')' fehlt".into()); } return Ok(Node::Peek(Box::new(e), low == "peekw")); }
        if low == "size" && self.peek_tok() == Some("(") {
            self.i += 1;
            let name = match self.t.get(self.i) { Some(n) if n.len() >= 2 && n.starts_with('"') => n[1..n.len() - 1].to_string(), _ => return Err("size() erwartet den Segmentnamen in Anführungszeichen".into()) };
            self.i += 1;
            if !self.eat(")") { return Err("')' fehlt".into()); }
            return Ok(Node::Size(name));
        }
        if let Some(v) = self.syms.get(&t) { return Ok(Node::Num(*v)); }
        if t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') && t.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_' || c == '.') { return Ok(Node::Reg(low)); }
        Err(format!("'{t}' nicht verständlich"))
    }
}

impl Expr {
    pub fn parse(src: &str, syms: &HashMap<String, i64>) -> Result<Expr, String> {
        let mut p = P { t: tokenize(src)?, i: 0, syms };
        if p.t.is_empty() { return Err("leere Bedingung".into()); }
        let node = p.level(0)?;
        if p.i != p.t.len() { return Err(format!("unerwartet '{}'", p.t[p.i])); }
        Ok(Expr { node, source: src.trim().to_string() })
    }
    pub fn eval(&self, ctx: &mut dyn Ctx) -> Result<i64, String> { ev(&self.node, ctx) }
    /// Ist der Ausdruck ein Vergleich (`==`, `!=`, `<`, `<=`, `>`, `>=`), liefert dies beide Seiten, damit eine Fehlermeldung den Ist-Wert nennen kann
    pub fn comparison(&self) -> Option<(Expr, &'static str, Expr)> {
        match &self.node {
            Node::Bin(op, l, r) if ["==", "!=", "<", "<=", ">", ">="].contains(op) => Some((Expr { node: (**l).clone(), source: String::new() }, *op, Expr { node: (**r).clone(), source: String::new() })),
            _ => None,
        }
    }
    pub fn holds(&self, ctx: &mut dyn Ctx) -> Result<bool, String> { Ok(self.eval(ctx)? != 0) }
}

fn ev(n: &Node, c: &mut dyn Ctx) -> Result<i64, String> {
    Ok(match n {
        Node::Num(v) => *v,
        Node::Reg(r) => c.reg(r).ok_or_else(|| format!("'{r}' ist weder Register noch Symbol"))?,
        Node::Size(n) => c.size(n)?,
        Node::Peek(e, word) => { let a = ev(e, c)?; if !(0..=0xFFFF).contains(&a) { return Err(format!("Adresse {a} außerhalb 0..65535")); } let lo = c.peek(a as u16)? as i64; if *word { lo | (c.peek((a as u16).wrapping_add(1))? as i64) << 8 } else { lo } }
        Node::Un(op, e) => { let v = ev(e, c)?; match op { '!' => (v == 0) as i64, '~' => !v, _ => -v } }
        Node::Bin(op, l, r) => {
            if *op == "&&" { return Ok((ev(l, c)? != 0 && ev(r, c)? != 0) as i64); }
            if *op == "||" { return Ok((ev(l, c)? != 0 || ev(r, c)? != 0) as i64); }
            let (a, b) = (ev(l, c)?, ev(r, c)?);
            match *op { "|" => a | b, "^" => a ^ b, "&" => a & b, "==" => (a == b) as i64, "!=" => (a != b) as i64, "<" => (a < b) as i64, "<=" => (a <= b) as i64, ">" => (a > b) as i64, ">=" => (a >= b) as i64,
                "<<" => a.checked_shl(b as u32).unwrap_or(0), ">>" => a.checked_shr(b as u32).unwrap_or(0), "+" => a.wrapping_add(b), "-" => a.wrapping_sub(b), "*" => a.wrapping_mul(b),
                "/" => { if b == 0 { return Err("Division durch 0".into()); } a / b } "%" => { if b == 0 { return Err("Division durch 0".into()); } a % b } _ => unreachable!() }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct T { mem: Vec<u8> }
    impl Ctx for T {
        fn reg(&self, n: &str) -> Option<i64> { match n { "a" => Some(0x80), "x" => Some(7), "fz" => Some(1), "fc" => Some(0), "lin" => Some(120), _ => None } }
        fn peek(&mut self, a: u16) -> Result<u8, String> { self.mem.get(a as usize).copied().ok_or("außerhalb".into()) }
    }
    fn run(s: &str) -> Result<i64, String> { let mut syms = HashMap::new(); syms.insert("irq".to_string(), 0x10); let mut t = T { mem: (0..=255).collect() }; Expr::parse(s, &syms)?.eval(&mut t) }
    #[test] fn arithmetic_and_precedence() {
        assert_eq!(run("1 + 2 * 3").unwrap(), 7); assert_eq!(run("(1 + 2) * 3").unwrap(), 9); assert_eq!(run("$10 | %0101").unwrap(), 0x15); assert_eq!(run("0x20 >> 2").unwrap(), 8);
        assert_eq!(run("10 % 4 == 2 && 3 < 4").unwrap(), 1); assert_eq!(run("-3 + 5").unwrap(), 2); assert_eq!(run("!0 + ~0").unwrap(), 0);
    }
    #[test] fn registers_memory_symbols() {
        assert_eq!(run("a == $80 && x > 5").unwrap(), 1); assert_eq!(run("[$20] == $20").unwrap(), 1); assert_eq!(run("peekw($10) == $1110").unwrap(), 1);
        assert_eq!(run("peek(irq) + 1").unwrap(), 0x11); assert_eq!(run("fz && !fc").unwrap(), 1); assert_eq!(run("lin >= 100").unwrap(), 1);
    }
    #[test] fn segment_size() {
        struct S;
        impl Ctx for S {
            fn reg(&self, _: &str) -> Option<i64> { None }
            fn peek(&mut self, _: u16) -> Result<u8, String> { Err("kein Speicher".into()) }
            fn size(&self, n: &str) -> Result<i64, String> { match n { "Main" => Ok(0x1F3C), "" => Ok(10), _ => Err(format!("Segment '{n}' nicht gefunden")) } }
        }
        let ev = |s: &str| Expr::parse(s, &HashMap::new()).and_then(|e| e.eval(&mut S));
        assert_eq!(ev("size(\"Main\") <= $2000").unwrap(), 1); assert_eq!(ev("size(\"\") + 1").unwrap(), 11);
        assert!(ev("size(\"Gibt es nicht\")").unwrap_err().contains("nicht gefunden")); assert!(ev("size(Main)").unwrap_err().contains("Anführungszeichen"));
        assert!(ev("size(\"Main").unwrap_err().contains("nicht geschlossen")); assert!(ev("\"x\" + 1").unwrap_err().contains("size"));
        assert!(run("size(\"Main\")").unwrap_err().contains("nicht zur Verfügung"));
    }
    #[test] fn errors() {
        assert!(run("1 +").unwrap_err().contains("endet")); assert!(run("(1").unwrap_err().contains("')'")); assert!(run("1 2").unwrap_err().contains("unerwartet"));
        assert!(run("foo == 1").unwrap_err().contains("weder Register")); assert!(run("1 / 0").unwrap_err().contains("Division")); assert!(run("1 # 2").unwrap_err().contains("nicht erlaubt"));
        assert!(run("[$ffff0]").unwrap_err().contains("außerhalb")); assert!(run("").is_err());
    }
}
