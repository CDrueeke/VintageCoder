// SPDX-License-Identifier: GPL-3.0-or-later
//! Parser: nummerierter CBM-BASIC-Text → Syntaxbaum.
//!
//! Der Text wird mit dem vorhandenen Tokenisierer (`tokenize_line`) in die Tokenform des ROMs gebracht; so gelten dieselben
//! Regeln für Schlüsselwörter ohne Leerzeichen (`FORI=1TO10`, `TO` in `TOTAL`) wie im Interpreter. Der Parser liest dann die Tokenbytes.
use super::ast::*;
use crate::dialect::Dialect;
use vc_core::Diagnostic;

#[derive(Debug, Clone, PartialEq)]
enum T { Kw(String), Id(String), Num(String), Str(Vec<u8>), Sym(char), Data(Vec<u8>), Rem(Vec<u8>) }

/// Atari-BASIC arbeitet mit ASCII, der Tokenisierer mit PETSCII (manche Zeichen wie `|` oder `_` fehlen dort): Texte in Anführungszeichen, `DATA` und `REM`
/// werden vor dem Tokenisieren durch Platzhalter `#n` ersetzt und beim Lesen (lex) als rohe ASCII-Bytes wieder eingesetzt.
fn protect_atari(text: &str) -> (String, Vec<Vec<u8>>) {
    let b: Vec<char> = text.chars().collect();
    let (mut out, mut tab, mut i) = (String::new(), Vec::<Vec<u8>>::new(), 0usize);
    let raw = |c: &[char]| -> Vec<u8> { c.iter().map(|ch| if (*ch as u32) < 256 { *ch as u32 as u8 } else { b'?' }).collect() };
    while i < b.len() {
        let c = b[i];
        if c == '"' {
            let end = b[i + 1..].iter().position(|x| *x == '"').map(|p| p + i + 1);
            let stop = end.unwrap_or(b.len());
            tab.push(raw(&b[i + 1..stop]));
            out.push_str(&format!("\"#{}\"", tab.len() - 1));
            i = end.map_or(b.len(), |e| e + 1);
            continue;
        }
        if c.is_ascii_alphabetic() {
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') { i += 1; }
            let w: String = b[st..i].iter().collect();
            let up = w.to_ascii_uppercase();
            let prev_at = st > 0 && b[st - 1] == '@';
            if !prev_at && up == "REM" { out.push_str("REM"); return (out, tab); }   // Kommentartext entfällt
            if !prev_at && up == "DATA" { while i < b.len() && b[i] == ' ' { i += 1; } tab.push(raw(&b[i..])); out.push_str(&format!("DATA #{}", tab.len() - 1)); return (out, tab); }
            out.push_str(&w);
            continue;
        }
        out.push(c);
        i += 1;
    }
    (out, tab)
}

fn lex(bytes: &[u8], d: &Dialect, atari: Option<&[Vec<u8>]>) -> Result<Vec<T>, String> {
    let ph = |b: &[u8]| -> Vec<u8> {   // Platzhalter "#n" -> roher Text
        match atari { Some(t) => std::str::from_utf8(b).ok().and_then(|s| s.trim().strip_prefix('#')?.parse::<usize>().ok()).and_then(|n| t.get(n).cloned()).unwrap_or_else(|| b.to_vec()), None => b.to_vec() }
    };
    let atari = atari.is_some();
    let (mut out, mut i) = (vec![], 0usize);
    while i < bytes.len() {
        let b = bytes[i];
        if b == b' ' { i += 1; continue; }
        if b == 0x22 {
            let end = bytes[i + 1..].iter().position(|x| *x == 0x22).map(|p| p + i + 1);
            let stop = end.unwrap_or(bytes.len());
            out.push(T::Str(ph(&bytes[i + 1..stop])));
            i = end.map_or(bytes.len(), |e| e + 1);
            continue;
        }
        if b >= 0x80 || matches!(b, b'+' | b'-' | b'*' | b'/' | b'^' | b'<' | b'>' | b'=') {
            if let Some((kw, n)) = d.keyword_at(bytes, i) {
                i += n;
                match kw {
                    "REM" => { out.push(T::Rem(bytes[i..].to_vec())); return Ok(out); }
                    "DATA" => {
                        out.push(T::Kw("DATA".into()));
                        let (mut quote, mut j) = (false, i);
                        while j < bytes.len() { if bytes[j] == 0x22 { quote = !quote; } if bytes[j] == b':' && !quote && !atari { break; } j += 1; }   // Atari: DATA reicht bis zum Zeilenende
                        out.push(T::Data(ph(&bytes[i..j])));
                        i = j;
                    }
                    k => out.push(T::Kw(k.to_string())),
                }
                continue;
            }
            return Err(format!("unbekanntes Token ${b:02X}"));
        }
        if b.is_ascii_uppercase() {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_uppercase() || bytes[j].is_ascii_digit()) { j += 1; }
            let mut name = String::from_utf8_lossy(&bytes[i..j]).into_owned();
            if j < bytes.len() && (bytes[j] == b'$' || bytes[j] == b'%') { name.push(bytes[j] as char); j += 1; }
            out.push(T::Id(name));
            i = j;
            continue;
        }
        if b.is_ascii_digit() || b == b'.' {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') { j += 1; }
            if j < bytes.len() && bytes[j] == b'E' {
                let mut k = j + 1;
                if k < bytes.len() && (bytes[k] == 0xAA || bytes[k] == 0xAB) { k += 1; }
                if k < bytes.len() && bytes[k].is_ascii_digit() {
                    while k < bytes.len() && bytes[k].is_ascii_digit() { k += 1; }
                    let s: String = bytes[i..k].iter().map(|c| match c { 0xAA => '+', 0xAB => '-', c => *c as char }).collect();
                    out.push(T::Num(s));
                    i = k;
                    continue;
                }
            }
            out.push(T::Num(String::from_utf8_lossy(&bytes[i..j]).into_owned()));
            i = j;
            continue;
        }
        if matches!(b, b'(' | b')' | b',' | b';' | b':' | b'#') { out.push(T::Sym(b as char)); i += 1; continue; }
        return Err(format!("unerwartetes Zeichen ${b:02X}"));
    }
    Ok(out)
}

const FUNCS: &[&str] = &["SGN", "INT", "ABS", "USR", "FRE", "POS", "SQR", "RND", "LOG", "EXP", "COS", "SIN", "TAN", "ATN", "PEEK", "LEN", "STR$", "VAL", "ASC", "CHR$", "LEFT$", "RIGHT$", "MID$", "STICK", "STRIG", "DPEEK", "HEX$", "DEC", "INSTR"];
/// Anweisungen, die der Parser als solche erkennt und als `Unsupported` weitergibt
const OTHER_STMTS: &[&str] = &["LOAD", "SAVE", "VERIFY", "OPEN", "CLOSE", "CMD", "PRINT#", "INPUT#", "LIST", "NEW", "CONT"];

type R<X> = Result<X, String>;

struct P { t: Vec<T>, i: usize, atari: bool, turbo: bool }

fn var_of(name: &str) -> Var {
    let (n, ty) = match name.chars().last() { Some('$') => (&name[..name.len() - 1], Ty::Str), Some('%') => (&name[..name.len() - 1], Ty::Int), _ => (name, Ty::Float) };
    Var { name: n.to_string(), ty }
}

impl P {
    fn peek(&self) -> Option<&T> { self.t.get(self.i) }
    fn next(&mut self) -> Option<T> { let t = self.t.get(self.i).cloned(); if t.is_some() { self.i += 1; } t }
    fn at_end(&self) -> bool { self.i >= self.t.len() }
    fn is_kw(&self, k: &str) -> bool { matches!(self.peek(), Some(T::Kw(x)) if x == k) }
    fn is_sym(&self, c: char) -> bool { matches!(self.peek(), Some(T::Sym(x)) if *x == c) }
    fn eat_kw(&mut self, k: &str) -> bool { if self.is_kw(k) { self.i += 1; true } else { false } }
    fn eat_sym(&mut self, c: char) -> bool { if self.is_sym(c) { self.i += 1; true } else { false } }
    fn want_sym(&mut self, c: char) -> R<()> { if self.eat_sym(c) { Ok(()) } else { Err(format!("'{c}' erwartet")) } }
    fn stmt_end(&self) -> bool { self.at_end() || self.is_sym(':') }

    fn line_no(&mut self) -> R<u32> {
        match self.next() {
            Some(T::Num(s)) if s.chars().all(|c| c.is_ascii_digit()) => s.parse::<u32>().ok().filter(|n| *n <= 63999).ok_or_else(|| format!("Zeilennummer {s} ungültig")),
            _ => Err("Zeilennummer erwartet".into()),
        }
    }

    // ---- Ausdrücke (Vorrang von niedrig nach hoch: OR, AND, NOT, Vergleich, + -, * /, unäres -, ^)
    fn expr(&mut self) -> R<Expr> {
        let mut l = self.and()?;
        while self.eat_kw("OR") { let r = self.and()?; l = Expr::Bin(BinOp::Or, Box::new(l), Box::new(r)); }
        Ok(l)
    }
    fn and(&mut self) -> R<Expr> {
        let mut l = self.not()?;
        while self.eat_kw("AND") { let r = self.not()?; l = Expr::Bin(BinOp::And, Box::new(l), Box::new(r)); }
        Ok(l)
    }
    fn not(&mut self) -> R<Expr> {
        if self.atari { return self.cmp(); }   // Atari: NOT, unäres + und - binden stärker als ^ (siehe aunary)
        if self.eat_kw("NOT") { let e = self.not()?; return Ok(Expr::Un(UnOp::Not, Box::new(e))); }
        self.cmp()
    }
    fn cmp(&mut self) -> R<Expr> {
        let mut l = self.add()?;
        loop {
            let op = match self.peek() {
                Some(T::Kw(k)) if k == "=" || k == "<" || k == ">" => k.clone(),
                _ => return Ok(l),
            };
            self.i += 1;
            let op = match (op.as_str(), self.peek()) {
                ("<", Some(T::Kw(k))) if k == "=" => { self.i += 1; BinOp::Le }
                ("<", Some(T::Kw(k))) if k == ">" => { self.i += 1; BinOp::Ne }
                (">", Some(T::Kw(k))) if k == "=" => { self.i += 1; BinOp::Ge }
                (">", Some(T::Kw(k))) if k == "<" => { self.i += 1; BinOp::Ne }
                ("=", Some(T::Kw(k))) if k == "<" => { self.i += 1; BinOp::Le }
                ("=", Some(T::Kw(k))) if k == ">" => { self.i += 1; BinOp::Ge }
                ("<", _) => BinOp::Lt,
                (">", _) => BinOp::Gt,
                _ => BinOp::Eq,
            };
            let r = self.add()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
    }
    fn add(&mut self) -> R<Expr> {
        let mut l = self.mul()?;
        loop {
            let op = if self.is_kw("+") { BinOp::Add } else if self.is_kw("-") { BinOp::Sub } else { return Ok(l) };
            self.i += 1;
            let r = self.mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
    }
    fn mul(&mut self) -> R<Expr> {
        let mut l = self.unary()?;
        loop {
            if self.is_kw("DIV") || self.is_kw("MOD") {   // Turbo BASIC XL: ganzzahlige Division und Rest (a MOD b = a - b*(a DIV b))
                let m = self.is_kw("MOD");
                self.i += 1;
                let r = self.unary()?;
                let div = Expr::Call("DIV".into(), vec![l.clone(), r.clone()]);
                l = if m { Expr::Bin(BinOp::Sub, Box::new(l), Box::new(Expr::Bin(BinOp::Mul, Box::new(r), Box::new(div)))) } else { div };
                continue;
            }
            let op = if self.is_kw("*") { BinOp::Mul } else if self.is_kw("/") { BinOp::Div } else { return Ok(l) };
            self.i += 1;
            let r = self.unary()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
    }
    fn unary(&mut self) -> R<Expr> {
        if self.atari { return self.pow(); }
        if self.eat_kw("-") { let e = self.unary()?; return Ok(Expr::Un(UnOp::Neg, Box::new(e))); }
        if self.eat_kw("+") { return self.unary(); }
        self.pow()
    }
    /// Atari-BASIC: unäres -, + und NOT stehen über dem Potenzieren (-2^2 = 4, NOT 0=1 ist (NOT 0)=1)
    fn aunary(&mut self) -> R<Expr> {
        // Messung am Original: ein Vorzeichen unmittelbar vor einem weiteren Vorzeichen entfällt (`- -2` ist -2, `- +X` ist X); `NOT -1` ist Fehler 11
        if self.eat_kw("-") { if self.is_kw("-") || self.is_kw("+") { return self.aunary(); } let e = self.aunary()?; return Ok(Expr::Un(UnOp::Neg, Box::new(e))); }
        if self.eat_kw("+") { return self.aunary(); }
        if self.eat_kw("NOT") {
            if self.is_kw("-") || self.is_kw("+") { return Err("NOT mit einem Vorzeichen davor (NOT -1) ist in Atari-BASIC Fehler 11; Klammern setzen: NOT (-1)".into()); }
            let e = self.aunary()?; return Ok(Expr::Un(UnOp::Not, Box::new(e)));
        }
        self.primary()
    }
    fn pow(&mut self) -> R<Expr> {
        let mut l = if self.atari { self.aunary()? } else { self.primary()? };
        while self.eat_kw("^") {
            let r = if self.atari { self.aunary()? } else if self.eat_kw("-") { let e = self.primary()?; Expr::Un(UnOp::Neg, Box::new(e)) } else { self.primary()? };
            l = Expr::Bin(BinOp::Pow, Box::new(l), Box::new(r));
        }
        Ok(l)
    }
    fn args(&mut self) -> R<Vec<Expr>> {
        self.want_sym('(')?;
        let mut v = vec![self.expr()?];
        while self.eat_sym(',') { v.push(self.expr()?); }
        self.want_sym(')')?;
        Ok(v)
    }
    fn primary(&mut self) -> R<Expr> {
        match self.next() {
            Some(T::Num(s)) => Ok(if s.chars().all(|c| c.is_ascii_digit()) { s.parse::<i64>().map(Expr::Num).unwrap_or(Expr::Float(s)) } else { Expr::Float(s) }),
            Some(T::Str(b)) => Ok(Expr::Str(b)),
            Some(T::Sym('(')) => { let e = self.expr()?; self.want_sym(')')?; Ok(e) }
            Some(T::Id(n)) => {
                let v = var_of(&n);
                if self.is_sym('(') { Ok(Expr::Elem(v, self.args()?)) } else { Ok(Expr::Var(v)) }
            }
            Some(T::Kw(k)) if k == "FN" => {
                let name = match self.next() { Some(T::Id(n)) => n, _ => return Err("Name nach FN erwartet".into()) };
                let mut a = self.args()?;
                if a.len() != 1 { return Err("FN erwartet genau ein Argument".into()); }
                Ok(Expr::Fn(name, Box::new(a.remove(0))))
            }
            Some(T::Kw(k)) if self.turbo && (k == "INKEY$" || k == "TIME$") => Ok(Expr::Call(k, vec![])),
            // RAND(n) = INT(RND(0)*n)
            Some(T::Kw(k)) if self.turbo && k == "RAND" => {
                let mut a = self.args()?;
                if a.len() != 1 { return Err("RAND erwartet genau ein Argument".into()); }
                Ok(Expr::Call("INT".into(), vec![Expr::Bin(BinOp::Mul, Box::new(Expr::Call("RND".into(), vec![Expr::Num(0)])), Box::new(a.remove(0)))]))
            }
            Some(T::Kw(k)) if FUNCS.contains(&k.as_str()) => Ok(Expr::Call(k, self.args()?)),
            Some(T::Kw(k)) => Err(format!("{k} ist hier nicht erlaubt")),
            Some(t) => Err(format!("Ausdruck erwartet, gefunden {t:?}")),
            None => Err("Ausdruck erwartet".into()),
        }
    }
    fn lvalue(&mut self) -> R<LValue> {
        match self.next() {
            Some(T::Id(n)) => { let v = var_of(&n); if self.is_sym('(') { Ok(LValue::Elem(v, self.args()?)) } else { Ok(LValue::Var(v)) } }
            _ => Err("Variable erwartet".into()),
        }
    }

    // ---- Anweisungen
    fn stmts(&mut self) -> R<Vec<Stmt>> {
        let mut v = vec![];
        loop {
            while self.eat_sym(':') {}
            if self.at_end() { return Ok(v); }
            v.push(self.stmt()?);
            if !self.at_end() && !self.is_sym(':') { return Err(format!("unerwartet: {:?}", self.peek().unwrap())); }
        }
    }
    /// Anweisungen bis zum Zeilenende (für `THEN`)
    fn rest(&mut self) -> R<Vec<Stmt>> { self.stmts() }

    fn target_list(&mut self) -> R<Vec<u32>> {
        let mut v = vec![self.line_no()?];
        while self.eat_sym(',') { v.push(self.line_no()?); }
        Ok(v)
    }

    fn stmt(&mut self) -> R<Stmt> {
        match self.next() {
            Some(T::Id(n)) => {
                self.i -= 1;
                let _ = n;
                let target = self.lvalue()?;
                if !self.eat_kw("=") { return Err("'=' erwartet".into()); }
                Ok(Stmt::Let { target, expr: self.expr()? })
            }
            Some(T::Rem(b)) => Ok(Stmt::Rem(b)),
            Some(T::Kw(k)) => self.keyword(&k),
            Some(t) => Err(format!("Anweisung erwartet, gefunden {t:?}")),
            None => Err("Anweisung erwartet".into()),
        }
    }

    fn keyword(&mut self, k: &str) -> R<Stmt> {
        Ok(match k {
            "LET" => { let target = self.lvalue()?; if !self.eat_kw("=") { return Err("'=' erwartet".into()); } Stmt::Let { target, expr: self.expr()? } }
            "WHILE" => Stmt::Turbo(Struct::While(self.expr()?)),
            "UNTIL" => Stmt::Turbo(Struct::Until(self.expr()?)),
            "WEND" => Stmt::Turbo(Struct::Wend),
            "REPEAT" => Stmt::Turbo(Struct::Repeat),
            "DO" => Stmt::Turbo(Struct::Do),
            "LOOP" => Stmt::Turbo(Struct::Loop),
            "EXIT" => Stmt::Turbo(Struct::Exit),
            "ELSE" => Stmt::Turbo(Struct::Else),
            "ENDIF" => Stmt::Turbo(Struct::Endif),
            "ENDPROC" => Stmt::Turbo(Struct::Endproc),
            "PROC" | "EXEC" => {
                let n = match self.next() { Some(T::Id(n)) => n, _ => return Err(format!("Name nach {k} erwartet")) };
                Stmt::Turbo(if k == "PROC" { Struct::Proc(n) } else { Struct::Exec(n) })
            }
            // Kanäle (CIO): OPEN #n,aux1,aux2,"name"; CLOSE #n; PUT #n,b; Turbo BASIC XL: BPUT/BGET #n,adresse,länge
            "OPEN" if self.atari => {
                self.eat_sym('#');
                let mut args = vec![self.expr()?];
                for _ in 0..3 { self.want_sym(',')?; args.push(self.expr()?); }
                Stmt::Atari { op: "OPEN".into(), args }
            }
            "CLOSE" if self.atari => { self.eat_sym('#'); Stmt::Atari { op: "CLOSE".into(), args: vec![self.expr()?] } }
            "PUT" if self.atari => {
                self.eat_sym('#');
                let c = self.expr()?; self.want_sym(',')?;
                Stmt::Atari { op: "PUT".into(), args: vec![c, self.expr()?] }
            }
            "BPUT" | "BGET" if self.turbo => {
                self.eat_sym('#');
                let mut args = vec![self.expr()?];
                for _ in 0..2 { self.want_sym(',')?; args.push(self.expr()?); }
                Stmt::Atari { op: k.to_string(), args }
            }
            "CLS" => Stmt::Atari { op: "CLS".into(), args: vec![] },
            "-" if self.turbo && self.is_kw("MOVE") => {
                self.i += 1;
                let mut args = vec![self.expr()?];
                while self.eat_sym(',') { args.push(self.expr()?); }
                if args.len() != 3 { return Err("-MOVE erwartet 3 Zahlen".into()); }
                Stmt::Atari { op: "-MOVE".into(), args }
            }
            "DPOKE" | "PAUSE" | "MOVE" => {
                let mut args = vec![self.expr()?];
                while self.eat_sym(',') { args.push(self.expr()?); }
                let want = match k { "PAUSE" => 1, "DPOKE" => 2, _ => 3 };
                if args.len() != want { return Err(format!("{k} erwartet {want} Zahlen")); }
                Stmt::Atari { op: k.to_string(), args }
            }
            "FCOLOR" | "FILLTO" if self.turbo => {
                let mut args = vec![self.expr()?];
                while self.eat_sym(',') { args.push(self.expr()?); }
                let want = if k == "FCOLOR" { 1 } else { 2 };
                if args.len() != want { return Err(format!("{k} erwartet {want} Zahlen")); }
                Stmt::Atari { op: k.to_string(), args }
            }
            "GRAPHICS" | "COLOR" | "PLOT" | "DRAWTO" | "POSITION" | "SETCOLOR" | "SOUND" => {
                let mut args = vec![self.expr()?];
                while self.eat_sym(',') { args.push(self.expr()?); }
                let want = match k { "GRAPHICS" | "COLOR" => 1, "PLOT" | "DRAWTO" | "POSITION" => 2, "SETCOLOR" => 3, _ => 4 };
                if args.len() != want { return Err(format!("{k} erwartet {want} Zahlen")); }
                Stmt::Atari { op: k.to_string(), args }
            }
            "LOCATE" => {
                let x = self.expr()?; self.want_sym(',')?; let y = self.expr()?; self.want_sym(',')?;
                Stmt::Locate(x, y, self.lvalue()?)
            }
            "PRINT" | "PRINT#" if self.is_sym('#') || k == "PRINT#" => {
                self.eat_sym('#');
                let chan = self.expr()?;
                if !self.eat_sym(';') && !self.eat_sym(',') { return Err("';' nach der Kanalnummer erwartet".into()); }
                let mut items = vec![];
                while !self.stmt_end() {
                    if self.eat_sym(';') { items.push(PrintItem::Semi); }
                    else if self.eat_sym(',') { items.push(PrintItem::Comma); }
                    else { items.push(PrintItem::Expr(self.expr()?)); }
                }
                Stmt::PrintTo { chan, items }
            }
            "PRINT" => {
                let mut items = vec![];
                while !self.stmt_end() {
                    if self.eat_sym(';') { items.push(PrintItem::Semi); }
                    else if self.eat_sym(',') { items.push(PrintItem::Comma); }
                    else if self.eat_kw("TAB(") { let e = self.expr()?; self.want_sym(')')?; items.push(PrintItem::Tab(e)); }
                    else if self.eat_kw("SPC(") { let e = self.expr()?; self.want_sym(')')?; items.push(PrintItem::Spc(e)); }
                    else { items.push(PrintItem::Expr(self.expr()?)); }
                }
                Stmt::Print { items }
            }
            "INPUT" => {
                let prompt = if let Some(T::Str(b)) = self.peek().cloned() { self.i += 1; if !self.eat_sym(';') && !self.eat_sym(',') { return Err("';' nach dem Eingabetext erwartet".into()); } Some(b) } else { None };
                let mut vars = vec![self.lvalue()?];
                while self.eat_sym(',') { vars.push(self.lvalue()?); }
                Stmt::Input { prompt, vars }
            }
            "GET" if self.atari && self.is_sym('#') => {
                self.eat_sym('#');
                let chan = self.expr()?;
                self.want_sym(',')?;
                Stmt::GetCh(chan, self.lvalue()?)
            }
            "GET" => Stmt::Get(self.lvalue()?),
            "GOTO" => Stmt::Goto(self.line_no()?),
            "GO" => { if !self.eat_kw("TO") { return Err("TO nach GO erwartet".into()); } Stmt::Goto(self.line_no()?) }
            "GOSUB" => Stmt::Gosub(self.line_no()?),
            "RETURN" => Stmt::Return,
            "ON" => {
                let sel = self.expr()?;
                if self.turbo && self.eat_kw("EXEC") {
                    let mut names = vec![];
                    loop { match self.next() { Some(T::Id(n)) => names.push(n), _ => return Err("Prozedurname nach ON … EXEC erwartet".into()) } if !self.eat_sym(',') { break; } }
                    return Ok(Stmt::Turbo(Struct::OnExec(sel, names)));
                }
                let gosub = if self.eat_kw("GOSUB") { true } else if self.eat_kw("GOTO") { false } else if self.eat_kw("GO") { if !self.eat_kw("TO") { return Err("TO erwartet".into()); } false } else { return Err("GOTO oder GOSUB nach ON erwartet".into()); };
                let targets = self.target_list()?;
                if gosub { Stmt::OnGosub { sel, targets } } else { Stmt::OnGoto { sel, targets } }
            }
            "IF" => {
                let cond = self.expr()?;
                let then = if self.eat_kw("THEN") {
                    if matches!(self.peek(), Some(T::Num(_))) { let n = self.line_no()?; let mut v = vec![Stmt::Goto(n)]; v.extend(self.rest()?); v } else { self.rest()? }
                } else if self.eat_kw("GOTO") { let n = self.line_no()?; let mut v = vec![Stmt::Goto(n)]; v.extend(self.rest()?); v }
                else if self.turbo && self.stmt_end() { return Ok(Stmt::Turbo(Struct::If(cond))); }   // Turbo BASIC XL: IF ohne THEN beginnt einen Block
                else { return Err("THEN oder GOTO nach IF erwartet".into()) };
                Stmt::If { cond, then }
            }
            "FOR" => {
                let var = match self.next() { Some(T::Id(n)) => var_of(&n), _ => return Err("Schleifenvariable erwartet".into()) };
                if !self.eat_kw("=") { return Err("'=' erwartet".into()); }
                let from = self.expr()?;
                if !self.eat_kw("TO") { return Err("TO erwartet".into()); }
                let to = self.expr()?;
                let step = if self.eat_kw("STEP") { Some(self.expr()?) } else { None };
                Stmt::For { var, from, to, step }
            }
            "NEXT" => {
                let mut v = vec![];
                while let Some(T::Id(n)) = self.peek().cloned() { self.i += 1; v.push(var_of(&n)); if !self.eat_sym(',') { break; } }
                Stmt::Next(v)
            }
            "DIM" => {
                let mut v = vec![];
                loop {
                    let n = match self.next() { Some(T::Id(n)) => var_of(&n), _ => return Err("Feldname erwartet".into()) };
                    v.push((n, self.args()?));
                    if !self.eat_sym(',') { break; }
                }
                Stmt::Dim(v)
            }
            "READ" => { let mut v = vec![self.lvalue()?]; while self.eat_sym(',') { v.push(self.lvalue()?); } Stmt::Read(v) }
            "DATA" => {
                let raw = match self.next() { Some(T::Data(b)) => b, _ => vec![] };
                Stmt::Data(if self.atari { raw.split(|b| *b == b',').map(|x| DataItem::Raw(x.to_vec())).collect() } else { data_items(&raw)? })
            }
            "RESTORE" => Stmt::Restore(if matches!(self.peek(), Some(T::Num(_))) { Some(self.line_no()?) } else { None }),
            "POKE" => { let a = self.expr()?; self.want_sym(',')?; Stmt::Poke(a, self.expr()?) }
            "SYS" => Stmt::Sys(self.expr()?),
            "WAIT" => { let a = self.expr()?; self.want_sym(',')?; let m = self.expr()?; let x = if self.eat_sym(',') { Some(self.expr()?) } else { None }; Stmt::Wait(a, m, x) }
            "DEF" => {
                if !self.eat_kw("FN") { return Err("FN nach DEF erwartet".into()); }
                let name = match self.next() { Some(T::Id(n)) => n, _ => return Err("Funktionsname nach FN erwartet".into()) };
                self.want_sym('(')?;
                let param = match self.next() { Some(T::Id(n)) => var_of(&n), _ => return Err("Parametername erwartet".into()) };
                self.want_sym(')')?;
                if !self.eat_kw("=") { return Err("'=' erwartet".into()); }
                Stmt::DefFn { name, param, body: self.expr()? }
            }
            "END" => Stmt::End,
            "STOP" => Stmt::Stop,
            "CLR" => Stmt::Clr,
            "RUN" => Stmt::Run(if matches!(self.peek(), Some(T::Num(_))) { Some(self.line_no()?) } else { None }),
            k if OTHER_STMTS.contains(&k) => { self.i = self.t.len().min(self.stmt_rest_end()); Stmt::Unsupported(k.to_string()) }
            k => return Err(format!("{k} ist hier keine Anweisung")),
        })
    }

    /// Position des nächsten `:` außerhalb von Zeichenketten (Zeichenketten sind schon eigene Token)
    fn stmt_rest_end(&self) -> usize {
        let mut j = self.i;
        while j < self.t.len() && self.t[j] != T::Sym(':') { j += 1; }
        j
    }
}

fn data_items(raw: &[u8]) -> R<Vec<DataItem>> {
    // Einträge durch Komma getrennt; Zeichenketten in Anführungszeichen dürfen Kommas enthalten
    let (mut items, mut cur, mut quote, mut was_quoted) = (vec![], Vec::<u8>::new(), false, false);
    let flush = |cur: &mut Vec<u8>, was_quoted: &mut bool, items: &mut Vec<DataItem>| -> R<()> {
        let s: Vec<u8> = if *was_quoted { cur.clone() } else { let t = String::from_utf8_lossy(cur).trim().to_string(); t.into_bytes() };
        if *was_quoted { items.push(DataItem::Str(s)); }
        else {
            let t = String::from_utf8_lossy(&s).to_string();
            let t = t.replace('\u{aa}', "+").replace('\u{ab}', "-");
            if t.is_empty() { items.push(DataItem::Num(0)); }
            else if let Ok(n) = t.parse::<i64>() { items.push(DataItem::Num(n)); }
            else if t.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'E')) && t.chars().any(|c| c.is_ascii_digit()) { items.push(DataItem::Float(t)); }
            else { items.push(DataItem::Str(s)); }
        }
        cur.clear(); *was_quoted = false; Ok(())
    };
    for &b in raw {
        if b == 0x22 { if !quote && cur.iter().all(|c| *c == b' ') { cur.clear(); } quote = !quote; was_quoted = true; continue; }
        if b == b',' && !quote { flush(&mut cur, &mut was_quoted, &mut items)?; continue; }
        // im Tokenisierer werden Buchstaben in DATA klein gelassen und '-' bleibt ASCII
        cur.push(b);
    }
    flush(&mut cur, &mut was_quoted, &mut items)?;
    Ok(items)
}

/// Syntaxbaum aus dem nummerierten Text (`number_lines` hat Nummern und Labels schon aufgelöst).
pub fn parse_numbered(numbered: &str, d: &Dialect, file: &str) -> Result<Program, Vec<Diagnostic>> { parse_numbered_for(numbered, d, file, false) }

/// Wie `parse_numbered`; `atari` schaltet die Atari-Regeln ein (bisher: `DATA` bis zum Zeilenende, Einträge als roher Text)
pub fn parse_numbered_for(numbered: &str, d: &Dialect, file: &str, atari: bool) -> Result<Program, Vec<Diagnostic>> { parse_numbered_with(numbered, d, file, atari, false) }

/// Wie `parse_numbered_for`; `turbo` schaltet zusätzlich die Turbo-BASIC-XL-Konstrukte ein (IF-Block, WHILE, REPEAT, DO, PROC)
pub fn parse_numbered_with(numbered: &str, d: &Dialect, file: &str, atari: bool, turbo: bool) -> Result<Program, Vec<Diagnostic>> {
    let (mut lines, mut errs): (Vec<Line>, Vec<Diagnostic>) = (vec![], vec![]);
    for (n, l) in numbered.lines().enumerate() {
        let t = l.trim_start();
        if t.is_empty() || t.starts_with(';') { continue; }
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        let Ok(num) = digits.parse::<u32>() else { errs.push(Diagnostic::error("E100", file, n + 1, "Zeile beginnt nicht mit einer Zeilennummer")); continue };
        let text = t[digits.len()..].trim_start().to_string();
        let (text_t, table) = if atari { protect_atari(&text) } else { (text.clone(), vec![]) };
        let bytes = match crate::tokenize_line(&text_t, d) { Ok(b) => b, Err(e) => { errs.push(Diagnostic::error("E100", file, n + 1, e)); continue } };
        let toks = match lex(&bytes, d, if atari { Some(&table[..]) } else { None }) { Ok(t) => t, Err(e) => { errs.push(Diagnostic::error("E101", file, n + 1, e)); continue } };
        let mut p = P { t: toks, i: 0, atari, turbo };
        match p.stmts() {
            Ok(stmts) => {
                if lines.iter().any(|x| x.number == num) { errs.push(Diagnostic::error("E102", file, n + 1, format!("Zeilennummer {num} doppelt"))); continue; }
                lines.push(Line { number: num, src_line: n + 1, text, stmts });
            }
            Err(e) => errs.push(Diagnostic::error("E103", file, n + 1, format!("Zeile {num}: {e}"))),
        }
    }
    lines.sort_by_key(|l| l.number);
    if errs.is_empty() { Ok(Program { lines }) } else { Err(errs) }
}
