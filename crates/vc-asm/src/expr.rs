// SPDX-License-Identifier: GPL-3.0-or-later
//! Ausdrucksauswertung (Pratt-Parser) für Operanden und Direktiven.
use std::collections::HashMap;
use std::rc::Rc;

pub struct Ctx<'a> {
    pub syms: &'a HashMap<String, i64>,
    pub locals: &'a HashMap<String, i64>,
    /// Variablen (`.var`): Wert in dieser Reihenfolge des Quelltexts, in jedem Durchlauf neu
    pub vars: &'a HashMap<String, Val>,
    /// Benutzerfunktionen (`.function`)
    pub funcs: &'a HashMap<String, Func>,
    /// Gerade ausgewertete Benutzerfunktionen (Schutz gegen Rekursion)
    pub stack: &'a [String],
    pub pc: i64,
    pub scope: &'a str,
    /// Pfad der geöffneten Scopes, z. B. `init.schleife` (leer = global)
    pub path: &'a str,
    pub strict: bool,
}

/// Einzeilige Benutzerfunktion: `.function name(a, b) = ausdruck`
#[derive(Debug, Clone)]
pub struct Func { pub params: Vec<String>, pub body: String }

/// Ergebnis eines Ausdrucks. `v` ist immer der ganzzahlige Wert (bei Kommazahlen zur Null hin gekürzt),
/// `f` der genaue Wert, wenn der Ausdruck eine Kommazahl ist.
#[derive(Debug, Clone)]
pub struct Val {
    pub v: i64,
    pub known: bool,
    pub f: Option<f64>,
    /// Liste (dann ist `v` ohne Bedeutung); `known` gilt, wenn alle Elemente bekannt sind
    pub list: Option<Rc<Vec<Val>>>,
}

impl Val {
    pub fn int(v: i64, known: bool) -> Val { Val { v, known, f: None, list: None } }
    pub fn list(items: Vec<Val>) -> Val { Val { v: 0, known: items.iter().all(|x| x.known), f: None, list: Some(Rc::new(items)) } }
    /// Fehler, wenn der Wert eine Liste ist (für Rechenoperationen mit Zahlen)
    pub fn scalar(self, op: &str) -> Result<Val, String> {
        if self.list.is_some() { Err(format!("'{op}' ist für Listen nicht erlaubt")) } else { Ok(self) }
    }
    /// Alle Zahlen einer (verschachtelten) Liste der Reihe nach; ein einzelner Wert ergibt sich selbst
    pub fn flatten(&self) -> Vec<Val> {
        match &self.list {
            Some(l) => l.iter().flat_map(Val::flatten).collect(),
            None => vec![self.clone()],
        }
    }
    fn real(x: f64, known: bool) -> Result<Val, String> {
        if !x.is_finite() { return if known { Err("Ergebnis ist keine endliche Zahl".into()) } else { Ok(Val::int(0, false)) }; }
        Ok(Val { v: x as i64, known, f: Some(x), list: None })
    }
    fn as_f64(&self) -> f64 { self.f.unwrap_or(self.v as f64) }
    /// Nachkommaanteil vorhanden?
    pub fn has_fraction(&self) -> bool { self.f.map_or(false, |x| x.fract() != 0.0) }
    fn as_int(&self, op: &str) -> Result<i64, String> {
        if self.list.is_some() { return Err(format!("'{op}' ist für Listen nicht erlaubt")); }
        if self.has_fraction() { Err(format!("'{op}' ist nur für ganze Zahlen erlaubt")) } else { Ok(self.v) }
    }
    fn truthy(&self) -> bool { self.as_f64() != 0.0 }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(i64),
    Float(f64),
    Ident(String),
    Op(&'static str),
}

pub fn scoped(name: &str, scope: &str) -> String {
    if name.starts_with('.') { format!("{scope}{name}") } else { name.to_string() }
}

/// Voller Name eines in `path` definierten Symbols.
pub fn qualified(name: &str, scope: &str, path: &str) -> String {
    if name.starts_with('.') || path.is_empty() { scoped(name, scope) } else { format!("{path}.{name}") }
}

/// Sucht `name` vom innersten Scope nach außen (zuletzt global); `.lokal` gilt nur beim letzten globalen Label.
pub fn lookup_scoped<T>(name: &str, scope: &str, path: &str, find: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
    if name.starts_with('.') { return find(&format!("{scope}{name}")); }
    let mut p = path;
    loop {
        let key = if p.is_empty() { name.to_string() } else { format!("{p}.{name}") };
        if let Some(v) = find(&key) { return Some(v); }
        if p.is_empty() { return None; }
        p = p.rfind('.').map_or("", |i| &p[..i]);
    }
}

fn lex(s: &str) -> Result<Vec<Tok>, String> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = vec![];
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() { i += 1; continue; }
        if ch == '$' || (ch == '0' && c.get(i + 1).map_or(false, |x| *x == 'x' || *x == 'X')) {
            i += if ch == '$' { 1 } else { 2 };
            let st = i;
            while i < c.len() && c[i].is_ascii_hexdigit() { i += 1; }
            if st == i { return Err("ungültige Hex-Zahl".into()); }
            out.push(Tok::Num(i64::from_str_radix(&c[st..i].iter().collect::<String>(), 16).map_err(|e| e.to_string())?));
        } else if ch == '%' && c.get(i + 1).map_or(false, |x| *x == '0' || *x == '1') {
            i += 1;
            let st = i;
            while i < c.len() && (c[i] == '0' || c[i] == '1') { i += 1; }
            out.push(Tok::Num(i64::from_str_radix(&c[st..i].iter().collect::<String>(), 2).map_err(|e| e.to_string())?));
        } else if ch.is_ascii_digit() {
            let st = i;
            while i < c.len() && c[i].is_ascii_digit() { i += 1; }
            let mut real = false;
            if i + 1 < c.len() && c[i] == '.' && c[i + 1].is_ascii_digit() {
                real = true;
                i += 1;
                while i < c.len() && c[i].is_ascii_digit() { i += 1; }
            }
            if i < c.len() && (c[i] == 'e' || c[i] == 'E') {
                let mut j = i + 1;
                if j < c.len() && (c[j] == '+' || c[j] == '-') { j += 1; }
                if j < c.len() && c[j].is_ascii_digit() {
                    while j < c.len() && c[j].is_ascii_digit() { j += 1; }
                    real = true;
                    i = j;
                }
            }
            let text: String = c[st..i].iter().collect();
            if real { out.push(Tok::Float(text.parse().map_err(|_| "ungültige Kommazahl".to_string())?)); }
            else { out.push(Tok::Num(text.parse().map_err(|_| "Zahl zu groß".to_string())?)); }
        } else if ch == '\'' {
            if c.get(i + 2) != Some(&'\'') { return Err("ungültiges Zeichenliteral".into()); }
            out.push(Tok::Num(c[i + 1] as i64));
            i += 3;
        } else if ch.is_alphabetic() || ch == '_' || ch == '.' {
            let st = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_' || c[i] == '.') { i += 1; }
            out.push(Tok::Ident(c[st..i].iter().collect()));
        } else {
            let two: String = c[i..(i + 2).min(c.len())].iter().collect();
            let op: &'static str = match two.as_str() {
                "<<" => "<<", ">>" => ">>", "==" => "==", "!=" => "!=", "<=" => "<=", ">=" => ">=", "&&" => "&&", "||" => "||",
                _ => match ch {
                    '!' => "!",
                    '+' => "+", '-' => "-", '*' => "*", '/' => "/", '%' => "%", '&' => "&", '|' => "|", '^' => "^",
                    '~' => "~", '<' => "<", '>' => ">", '(' => "(", ')' => ")", ',' => ",", '[' => "[", ']' => "]", ':' => ":",
                    _ => return Err(format!("unerwartetes Zeichen '{ch}'")),
                },
            };
            i += op.len();
            out.push(Tok::Op(op));
        }
    }
    Ok(out)
}

struct P<'a, 'b> {
    t: Vec<Tok>,
    i: usize,
    ctx: &'a Ctx<'b>,
}

fn prec(op: &str) -> Option<u8> {
    Some(match op {
        "||" => 1, "&&" => 2, "==" | "!=" => 3, "<" | ">" | "<=" | ">=" => 4, "|" => 5, "^" => 6, "&" => 7,
        "<<" | ">>" => 8, "+" | "-" => 9, "*" | "/" | "%" => 10,
        _ => return None,
    })
}

pub fn is_builtin(n: &str) -> bool { func_arity(n).is_some() }

/// Anzahl der Argumente (min, max) der eingebauten Funktionen
fn func_arity(n: &str) -> Option<(usize, usize)> {
    Some(match n {
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sqrt" | "exp" | "log" | "log2" | "log10"
        | "int" | "floor" | "ceil" | "round" | "abs" | "sign" => (1, 1),
        "atan2" | "pow" => (2, 2),
        "min" | "max" => (1, usize::MAX),
        "len" => (1, 1),
        "range" => (1, 3),
        _ => return None,
    })
}

impl P<'_, '_> {
    /// Funktionsaufruf `name(arg, …)`; der Name steht schon gelesen, die Klammer kommt als Nächstes.
    fn args(&mut self, name: &str) -> Result<Vec<Val>, String> {
        self.i += 1;
        let mut args: Vec<Val> = vec![];
        if self.t.get(self.i) == Some(&Tok::Op(")")) { self.i += 1; } else {
            loop {
                args.push(self.expr(1)?);
                match self.t.get(self.i) {
                    Some(Tok::Op(",")) => self.i += 1,
                    Some(Tok::Op(")")) => { self.i += 1; break; }
                    _ => return Err(format!("')' fehlt bei {name}(…)")),
                }
            }
        }
        Ok(args)
    }
    fn call_user(&mut self, name: &str) -> Result<Val, String> {
        let args = self.args(name)?;
        let f = &self.ctx.funcs[name];
        if args.len() != f.params.len() { return Err(format!("{name}() erwartet {} Argument(e), erhalten {}", f.params.len(), args.len())); }
        if self.ctx.stack.iter().any(|s| s == name) { return Err(format!("Funktion '{name}' ruft sich selbst auf")); }
        if self.ctx.stack.len() >= 32 { return Err("Funktionen zu tief verschachtelt".into()); }
        let mut vars = self.ctx.vars.clone();
        for (p, a) in f.params.iter().zip(&args) { vars.insert(qualified(p, "", self.ctx.path), a.clone()); }
        let mut stack = self.ctx.stack.to_vec();
        stack.push(name.to_string());
        let ctx = Ctx { syms: self.ctx.syms, locals: self.ctx.locals, vars: &vars, funcs: self.ctx.funcs, stack: &stack, pc: self.ctx.pc, scope: self.ctx.scope, path: self.ctx.path, strict: self.ctx.strict };
        let mut v = eval(&f.body, &ctx).map_err(|e| format!("{e} (in Funktion {name})"))?;
        v.known &= args.iter().all(|a| a.known);
        Ok(v)
    }
    fn call(&mut self, name: &str) -> Result<Val, String> {
        let args = self.args(name)?;
        let (lo, hi) = func_arity(name).unwrap();
        if args.len() < lo || args.len() > hi {
            return Err(if lo == hi { format!("{name}() erwartet {lo} Argument(e), erhalten {}", args.len()) } else { format!("{name}() erwartet mindestens {lo} Argument(e), erhalten {}", args.len()) });
        }
        let known = args.iter().all(|a| a.known);
        if name == "len" {
            return match &args[0].list { Some(l) => Ok(Val::int(l.len() as i64, args[0].known)), None => Err("len() erwartet eine Liste".into()) };
        }
        for a in &args { if a.list.is_some() { return Err(format!("{name}() erwartet Zahlen, keine Liste")); } }
        if name == "range" {
            let (lo, hi, st) = match args.as_slice() { [n] => (0, n.v, 1), [l, h] => (l.v, h.v, 1), [l, h, s] => (l.v, h.v, s.v), _ => unreachable!() };
            if st == 0 { return Err("range: Schrittweite darf nicht 0 sein".into()); }
            let count = if st > 0 { (hi - lo).max(0) + st - 1 } else { (lo - hi).max(0) - st - 1 } / st.abs();
            if count > 65536 { return Err("range: mehr als 65536 Elemente".into()); }
            return Ok(Val::list((0..count).map(|k| Val::int(lo + k * st, known)).collect()));
        }
        let x = args[0].as_f64();
        let r = |f: f64| Val::real(f, known);
        match name {
            "sin" => r(x.sin()), "cos" => r(x.cos()), "tan" => r(x.tan()),
            "asin" => r(x.asin()), "acos" => r(x.acos()), "atan" => r(x.atan()),
            "atan2" => r(x.atan2(args[1].as_f64())),
            "sqrt" => r(x.sqrt()), "exp" => r(x.exp()), "log" => r(x.ln()), "log2" => r(x.log2()), "log10" => r(x.log10()),
            "pow" => r(x.powf(args[1].as_f64())),
            "int" => Ok(Val::int(x.trunc() as i64, known)),
            "floor" => Ok(Val::int(x.floor() as i64, known)),
            "ceil" => Ok(Val::int(x.ceil() as i64, known)),
            "round" => Ok(Val::int(x.round() as i64, known)),
            "sign" => Ok(Val::int(if x > 0.0 { 1 } else if x < 0.0 { -1 } else { 0 }, known)),
            "abs" => match args[0].f { Some(f) => Val::real(f.abs(), known), None => Ok(Val::int(args[0].v.wrapping_abs(), known)) },
            "min" | "max" => {
                let mut best = args[0].clone();
                for a in &args[1..] {
                    let less = a.as_f64() < best.as_f64();
                    if (name == "min") == less && a.as_f64() != best.as_f64() { best = a.clone(); }
                }
                best.known = known;
                Ok(best)
            }
            _ => unreachable!(),
        }
    }

    fn expr(&mut self, min: u8) -> Result<Val, String> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.t.get(self.i) { Some(Tok::Op(o)) => *o, _ => break };
            let Some(p) = prec(op) else { break };
            if p < min { break; }
            self.i += 1;
            let rhs = self.expr(p + 1)?;
            let known = lhs.known && rhs.known;
            if lhs.list.is_some() || rhs.list.is_some() {
                lhs = match (op, &lhs.list, &rhs.list) {
                    ("+", Some(a), Some(b)) => Val::list(a.iter().chain(b.iter()).cloned().collect()),
                    ("*", Some(a), None) | ("*", None, Some(a)) => {
                        let n = if lhs.list.is_some() { rhs.as_int("*")? } else { lhs.as_int("*")? };
                        if n < 0 || n.saturating_mul(a.len() as i64) > 65536 { return Err("Liste zu lang (höchstens 65536 Elemente)".into()); }
                        Val::list((0..n).flat_map(|_| a.iter().cloned()).collect())
                    }
                    _ => return Err(format!("'{op}' ist für Listen nicht erlaubt (nur Liste + Liste und Liste * Zahl)")),
                };
                continue;
            }
            if matches!(op, "%" | "&" | "|" | "^" | "<<" | ">>") { lhs.as_int(op)?; rhs.as_int(op)?; }
            else if lhs.f.is_some() || rhs.f.is_some() {
                let (x, y) = (lhs.as_f64(), rhs.as_f64());
                lhs = match op {
                    "+" => Val::real(x + y, known)?, "-" => Val::real(x - y, known)?, "*" => Val::real(x * y, known)?,
                    "/" => { if y == 0.0 { if known { return Err("Division durch 0".into()) } else { Val::int(0, false) } } else { Val::real(x / y, known)? } }
                    "==" => Val::int((x == y) as i64, known), "!=" => Val::int((x != y) as i64, known),
                    "<" => Val::int((x < y) as i64, known), ">" => Val::int((x > y) as i64, known),
                    "<=" => Val::int((x <= y) as i64, known), ">=" => Val::int((x >= y) as i64, known),
                    "&&" => Val::int((lhs.truthy() && rhs.truthy()) as i64, known), "||" => Val::int((lhs.truthy() || rhs.truthy()) as i64, known),
                    _ => unreachable!(),
                };
                continue;
            }
            let (a, b) = (lhs.v, rhs.v);
            let v = match op {
                "+" => a.wrapping_add(b), "-" => a.wrapping_sub(b), "*" => a.wrapping_mul(b),
                "/" => if b == 0 { if known { return Err("Division durch 0".into()) } else { 0 } } else { a / b },
                "%" => if b == 0 { if known { return Err("Division durch 0".into()) } else { 0 } } else { a % b },
                "&" => a & b, "|" => a | b, "^" => a ^ b,
                "==" => (a == b) as i64, "!=" => (a != b) as i64, "<" => (a < b) as i64, ">" => (a > b) as i64,
                "<=" => (a <= b) as i64, ">=" => (a >= b) as i64, "&&" => (a != 0 && b != 0) as i64, "||" => (a != 0 || b != 0) as i64,
                "<<" => a.wrapping_shl(b as u32), ">>" => a.wrapping_shr(b as u32),
                _ => unreachable!(),
            };
            lhs = Val::int(v, known);
        }
        Ok(lhs)
    }

    /// Primärausdruck mit anschließenden Index- und Ausschnittzugriffen `t[i]`, `t[a:b]`
    fn unary(&mut self) -> Result<Val, String> {
        let mut v = self.primary()?;
        while self.t.get(self.i) == Some(&Tok::Op("[")) {
            self.i += 1;
            let Some(items) = v.list.clone() else { return Err("Index nur auf Listen erlaubt".into()) };
            let n = items.len() as i64;
            let fix = |k: i64| if k < 0 { k + n } else { k };
            let mut lo: Option<Val> = None;
            if self.t.get(self.i) != Some(&Tok::Op(":")) { lo = Some(self.expr(1)?); }
            if self.t.get(self.i) == Some(&Tok::Op(":")) {
                self.i += 1;
                let hi = if self.t.get(self.i) == Some(&Tok::Op("]")) { None } else { Some(self.expr(1)?) };
                if self.t.get(self.i) != Some(&Tok::Op("]")) { return Err("']' fehlt".into()); }
                self.i += 1;
                let a = match &lo { Some(x) => fix(x.clone().scalar(":")?.v), None => 0 }.clamp(0, n);
                let b = match &hi { Some(x) => fix(x.clone().scalar(":")?.v), None => n }.clamp(a, n);
                v = Val::list(items[a as usize..b as usize].to_vec());
                v.known &= lo.map_or(true, |x| x.known) && hi.map_or(true, |x| x.known);
            } else {
                let idx = lo.ok_or("Index fehlt")?;
                if self.t.get(self.i) != Some(&Tok::Op("]")) { return Err("']' fehlt".into()); }
                self.i += 1;
                let k = fix(idx.clone().scalar("[]")?.v);
                if k < 0 || k >= n {
                    if idx.known && v.known { return Err(format!("Index {} außerhalb der Liste (Länge {n})", idx.v)); }
                    v = Val::int(0, false);
                } else {
                    let mut e = items[k as usize].clone();
                    e.known &= idx.known;
                    v = e;
                }
            }
        }
        Ok(v)
    }

    fn primary(&mut self) -> Result<Val, String> {
        let t = self.t.get(self.i).cloned().ok_or("Ausdruck unvollständig")?;
        self.i += 1;
        match t {
            Tok::Num(n) => Ok(Val::int(n, true)),
            Tok::Float(x) => Val::real(x, true),
            Tok::Ident(name) if self.t.get(self.i) == Some(&Tok::Op("(")) && self.ctx.funcs.contains_key(&name) => self.call_user(&name),
            Tok::Ident(name) if self.t.get(self.i) == Some(&Tok::Op("(")) && func_arity(&name.to_ascii_lowercase()).is_some() => self.call(&name.to_ascii_lowercase()),
            Tok::Ident(name) if matches!(name.to_ascii_lowercase().as_str(), "next" | "align") && self.t.get(self.i) == Some(&Tok::Op("(")) => {
                // next(n): nächste durch n teilbare Adresse ab dem Programmzähler (bleibt stehen, wenn er schon passt)
                self.i += 1;
                let n = self.expr(1)?;
                if self.t.get(self.i) != Some(&Tok::Op(")")) { return Err("')' fehlt".into()); }
                self.i += 1;
                if n.known && n.v < 1 { return Err("next(n): n muss größer als 0 sein".into()); }
                let n1 = n.v.max(1);
                Ok(Val::int((self.ctx.pc + n1 - 1) / n1 * n1, n.known))
            }
            Tok::Ident(name) => {
                let found = lookup_scoped(&name, self.ctx.scope, self.ctx.path, &mut |k| {
                    self.ctx.locals.get(k).map(|v| Val::int(*v, true)).or_else(|| self.ctx.vars.get(k).cloned()).or_else(|| self.ctx.syms.get(k).map(|v| Val::int(*v, true)))
                });
                match found {
                    Some(v) => Ok(v),
                    None if name.eq_ignore_ascii_case("pi") => Val::real(std::f64::consts::PI, true),
                    None if self.ctx.strict => Err(format!("unbekanntes Symbol '{name}'")),
                    None => Ok(Val::int(0, false)),
                }
            }
            Tok::Op("[") => {
                let mut items = vec![];
                if self.t.get(self.i) == Some(&Tok::Op("]")) { self.i += 1; } else {
                    loop {
                        items.push(self.expr(1)?);
                        match self.t.get(self.i) {
                            Some(Tok::Op(",")) => self.i += 1,
                            Some(Tok::Op("]")) => { self.i += 1; break; }
                            _ => return Err("']' fehlt".into()),
                        }
                    }
                }
                Ok(Val::list(items))
            }
            Tok::Op("*") => Ok(Val::int(self.ctx.pc, true)),
            Tok::Op("-") => { let v = self.unary()?.scalar("-")?; match v.f { Some(x) => Val::real(-x, v.known), None => Ok(Val::int(v.v.wrapping_neg(), v.known)) } }
            Tok::Op("+") => self.unary(),
            Tok::Op("!") => { let v = self.unary()?.scalar("!")?; Ok(Val::int((!v.truthy()) as i64, v.known)) }
            Tok::Op("~") => { let v = self.unary()?; Ok(Val::int(!v.as_int("~")?, v.known)) }
            Tok::Op("<") => { let v = self.unary()?; Ok(Val::int(v.as_int("<")? & 0xFF, v.known)) }
            Tok::Op(">") => { let v = self.unary()?; Ok(Val::int((v.as_int(">")? >> 8) & 0xFF, v.known)) }
            Tok::Op("(") => {
                let v = self.expr(1)?;
                if self.t.get(self.i) != Some(&Tok::Op(")")) { return Err("')' fehlt".into()); }
                self.i += 1;
                Ok(v)
            }
            Tok::Op(o) => Err(format!("unerwarteter Operator '{o}'")),
        }
    }
}

pub fn eval(s: &str, ctx: &Ctx) -> Result<Val, String> {
    let t = lex(s)?;
    if t.is_empty() { return Err("Ausdruck fehlt".into()); }
    let mut p = P { t, i: 0, ctx };
    let v = p.expr(1)?;
    if p.i != p.t.len() { return Err("unerwartete Zeichen am Ausdrucksende".into()); }
    Ok(v)
}
