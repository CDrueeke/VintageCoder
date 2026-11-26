// SPDX-License-Identifier: GPL-3.0-or-later
//! Werkzeuge rund um Zahlenschreibweisen im nativen BASIC (Konzept „BASIC-Zahlenformate“, Stufen 3 bis 6):
//! Umwandeln (Dezimal, Hex, Binär, nach Verwendung), Suchen nach dem Zahlenwert, Bit-Rechnung für den Bit-Editor,
//! Hex-Vorschläge für importierte Programme und ein kleiner Rechner.
//!
//! Alles arbeitet auf der Erkennung von `numbers::scan_line`: Zeichenketten, Kommentare (`REM`), Zeilennummern, Marken
//! und Zeilenziele bleiben unberührt. Positionen sind Zeichenindizes (nicht Bytes).
use crate::numbers::{scan_line, Lexicon, Num, NumberOptions, Role};

// ------------------------------------------------------------------ Umwandeln

/// Zielschreibweise
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target { Dec, Hex, Bin, ByUse }

/// Welche Zahlen umgewandelt werden (Rollen; Zeilenziele nie)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subsets {
    /// Adressen von `POKE`, `PEEK`, `SYS`, `WAIT`, `USR`
    pub addresses: bool,
    /// Werte von `POKE` (zweites Argument) und `WAIT`-Masken
    pub values: bool,
    /// Konstanten in sonstigen Ausdrücken
    pub exprs: bool,
    /// vollständige `DATA`-Elemente
    pub data: bool,
}
impl Subsets {
    pub const ALL: Subsets = Subsets { addresses: true, values: true, exprs: true, data: true };
    pub const ADDRESSES: Subsets = Subsets { addresses: true, values: false, exprs: false, data: false };
    fn takes(&self, r: Role) -> bool { match r { Role::Address => self.addresses, Role::Value => self.values, Role::Other => self.exprs, Role::Data => self.data, Role::Target => false } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvertOptions {
    pub target: Target,
    pub subsets: Subsets,
    /// Hexziffern groß (`$D020`), sonst klein
    pub upper: bool,
    /// Binärzahlen in Vierergruppen mit `_` (`%0000_1110`)
    pub groups: bool,
    /// bei Hex ↔ Binär die Breite der alten Schreibweise behalten, soweit sie bekannt ist
    pub keep_zeros: bool,
}
impl Default for ConvertOptions {
    fn default() -> Self { ConvertOptions { target: Target::Hex, subsets: Subsets::ALL, upper: true, groups: false, keep_zeros: true } }
}

/// Schreibt `v` in der Schreibweise `radix` (10, 16, 2). Hex: 2 Ziffern bis 255, 4 bis 65535, sonst 6 oder 8 (Adressen mindestens 4);
/// Binär: 8 Ziffern bis 255, sonst 16, 24 oder 32 (Adressen mindestens 16). `min_digits` erzwingt eine Mindestbreite (führende Nullen).
pub fn format_num(v: u32, radix: u32, role: Role, c: &ConvertOptions, min_digits: usize) -> String {
    match radix {
        16 => {
            let base = if role == Role::Address || v > 0xFF { if v > 0xFFFF { if v > 0xFF_FFFF { 8 } else { 6 } } else { 4 } } else { 2 };
            let w = base.max(min_digits);
            if c.upper { format!("${v:0w$X}") } else { format!("${v:0w$x}") }
        }
        2 => {
            let base = if role == Role::Address || v > 0xFF { if v > 0xFFFF { if v > 0xFF_FFFF { 32 } else { 24 } } else { 16 } } else { 8 };
            let w = base.max(min_digits);
            let s = format!("{v:0w$b}");
            if c.groups && s.len() > 4 {
                let first = s.len() % 4;
                let mut parts: Vec<&str> = vec![];
                if first > 0 { parts.push(&s[..first]); }
                parts.extend((first..s.len()).step_by(4).map(|i| &s[i..i + 4]));
                format!("%{}", parts.join("_"))
            } else { format!("%{s}") }
        }
        _ => v.to_string(),
    }
}

fn target_radix(t: Target, role: Role, v: u32) -> u32 {
    match t {
        Target::Dec => 10, Target::Hex => 16, Target::Bin => 2,
        Target::ByUse => match role { Role::Address => 16, Role::Value if v <= 0xFF => 2, Role::Value => 16, _ => 10 },
    }
}

/// Neue Schreibweise einer Zahl, `None` wenn sie bleibt (Zeilenziel, gleiche Schreibweise, nicht gewählt)
fn new_text(n: &Num, c: &ConvertOptions) -> Option<String> {
    if !c.subsets.takes(n.role) { return None; }
    let r = target_radix(c.target, n.role, n.value);
    if r == n.radix { return None; }
    let min = if c.keep_zeros && n.radix != 10 {
        let bits = if n.radix == 16 { n.digits * 4 } else { n.digits };
        if r == 16 { bits.div_ceil(4) } else if r == 2 { bits } else { 0 }
    } else { 0 };
    Some(format_num(n.value, r, n.role, c, min))
}

/// Eine Änderung für die Vorschau
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Zeile (0-basiert) und Bereich in Zeichen der alten Zeile
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Default)]
pub struct Converted {
    pub text: String,
    pub changes: Vec<Change>,
}

impl Converted {
    /// Zahl der betroffenen Zeilen
    pub fn lines_changed(&self) -> usize { let mut l: Vec<usize> = self.changes.iter().map(|c| c.line).collect(); l.dedup(); l.len() }
}

/// Wandelt die Zahlen der Zeilen `first..=last` (0-basiert) um; `chars`: nur Zahlen, die ganz in diesem Zeichenbereich der Gesamttexts liegen
/// (Auswahl). Zeichenketten, Kommentare, Zeilennummern, Marken und Zeilenziele bleiben unberührt.
pub fn convert_lines(text: &str, first: usize, last: usize, chars: Option<(usize, usize)>, c: &ConvertOptions, lx: &Lexicon, o: &NumberOptions) -> Converted {
    let o = NumberOptions { data: true, ..o.clone() };
    let mut out = Converted::default();
    let mut res = String::with_capacity(text.len());
    let mut offset = 0usize;   // Zeichenindex des Zeilenanfangs im Gesamttext
    let lines: Vec<&str> = text.split('\n').collect();
    for (ln, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        if ln >= first && ln <= last {
            let ch: Vec<char> = line.chars().collect();
            let sc = scan_line(line, lx, &o);
            let mut at = 0;
            let mut s = String::new();
            for n in &sc.nums {
                if let Some((a, b)) = chars { if offset + n.start < a || offset + n.end > b { continue; } }
                let Some(new) = new_text(n, c) else { continue };
                s.extend(&ch[at..n.start]);
                s.push_str(&new);
                out.changes.push(Change { line: ln, start: n.start, end: n.end, from: ch[n.start..n.end].iter().collect(), to: new });
                at = n.end;
            }
            s.extend(&ch[at..]);
            res.push_str(&s);
        } else { res.push_str(line); }
        if ln + 1 < lines.len() { res.push('\n'); }
        offset += len + 1;
    }
    out.text = res;
    out
}

// ------------------------------------------------------------------ Zahlen lesen, suchen

/// Liest eine Zahl wie im Quelltext: `255`, `$FF`, `%1111_1111` (Vorzeichen `-` erlaubt, Ergebnis dann negativ)
pub fn parse_number(s: &str) -> Option<i64> {
    let t = s.trim();
    let (neg, t) = match t.strip_prefix('-') { Some(r) => (true, r.trim_start()), None => (false, t) };
    let clean: String = t.chars().filter(|c| *c != '_').collect();
    let body = t.strip_prefix(['$', '%']).unwrap_or(t);
    if clean.is_empty() || body.is_empty() || body.starts_with('_') || body.ends_with('_') || body.contains("__") { return None; }
    let v = if let Some(h) = clean.strip_prefix('$') { u32::from_str_radix(h, 16).ok()? as i64 }
        else if let Some(b) = clean.strip_prefix('%') { u32::from_str_radix(b, 2).ok()? as i64 }
        else if clean.chars().all(|c| c.is_ascii_digit()) { clean.parse::<u32>().ok()? as i64 }
        else { return None };
    Some(if neg { -v } else { v })
}

/// Fundstellen (Zeichenbereiche) aller Zahlen der Zeile mit dem Wert `value`, gleich in welcher Schreibweise.
/// Zeichenketten, Kommentare und Text in `DATA` zählen nicht.
pub fn find_value(line: &str, value: u32, lx: &Lexicon, o: &NumberOptions) -> Vec<Num> {
    let o = NumberOptions { data: true, ..o.clone() };
    scan_line(line, lx, &o).nums.into_iter().filter(|n| n.value == value).collect()
}

/// Schreibweise einer gefundenen Zahl für einen neuen Wert übernehmen (Ersetzen behält die Schreibweise der Fundstelle)
pub fn reformat_like(old: &Num, value: u32, upper: bool) -> String {
    let c = ConvertOptions { upper, groups: false, ..Default::default() };
    format_num(value, old.radix, old.role, &c, if old.radix == 10 { 0 } else { old.digits })
}

// ------------------------------------------------------------------ Bit-Rechnung

/// Zahl mit fester Breite für den Bit-Editor (8, 16 oder 32 Bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bits { pub width: u8, pub value: u32 }

impl Bits {
    /// kleinste Breite (8, 16, 32), in die `v` passt
    pub fn fit(v: u32) -> u8 { if v <= 0xFF { 8 } else if v <= 0xFFFF { 16 } else { 32 } }
    pub fn new(width: u8, value: u32) -> Bits { let b = Bits { width, value: 0 }; Bits { value: value & b.mask(), ..b } }
    fn mask(&self) -> u32 { if self.width >= 32 { u32::MAX } else { (1u32 << self.width) - 1 } }
    pub fn get(&self, bit: u8) -> bool { bit < self.width && self.value >> bit & 1 == 1 }
    pub fn toggle(&mut self, bit: u8) { if bit < self.width { self.value ^= 1 << bit; } }
    pub fn set(&mut self, bit: u8, on: bool) { if bit < self.width { if on { self.value |= 1 << bit } else { self.value &= !(1 << bit) } } }
    /// Bits `lo..=hi` auf einen Wert setzen (Feld aus mehreren Bits, z. B. `YSCROLL` Bits 2–0)
    pub fn set_field(&mut self, lo: u8, hi: u8, field: u32) {
        let (lo, hi) = (lo.min(hi), hi.max(lo).min(self.width.saturating_sub(1)));
        let w = hi - lo + 1;
        let m = if w >= 32 { u32::MAX } else { (1u32 << w) - 1 };
        self.value = (self.value & !(m << lo)) | ((field & m) << lo);
    }
    pub fn field(&self, lo: u8, hi: u8) -> u32 {
        let (lo, hi) = (lo.min(hi), hi.max(lo).min(self.width.saturating_sub(1)));
        let w = hi - lo + 1;
        (self.value >> lo) & if w >= 32 { u32::MAX } else { (1u32 << w) - 1 }
    }
    pub fn clear(&mut self) { self.value = 0; }
    pub fn set_all(&mut self) { self.value = self.mask(); }
    pub fn invert(&mut self) { self.value = !self.value & self.mask(); }
    pub fn shift_left(&mut self) { self.value = (self.value << 1) & self.mask(); }
    pub fn shift_right(&mut self) { self.value >>= 1; }
    /// Low- und High-Byte tauschen (nur bei 16 Bit)
    pub fn swap_bytes(&mut self) { if self.width == 16 { self.value = (self.value as u16).swap_bytes() as u32; } }
    /// Breite ändern; der Wert bleibt, soweit er passt
    pub fn resize(&mut self, width: u8) { self.width = width; self.value &= self.mask(); }
    /// Binärziffern mit führenden Nullen, in Vierergruppen, wenn `groups`
    pub fn bin_text(&self, groups: bool) -> String {
        let c = ConvertOptions { groups, ..Default::default() };
        format_num(self.value, 2, Role::Other, &c, self.width as usize)
    }
    pub fn hex_text(&self, upper: bool) -> String {
        let c = ConvertOptions { upper, ..Default::default() };
        format_num(self.value, 16, Role::Other, &c, self.width as usize / 4)
    }
    /// Schreibweise für den Quelltext
    pub fn source_text(&self, radix: u32, upper: bool, groups: bool) -> String {
        match radix { 16 => self.hex_text(upper), 2 => self.bin_text(groups), _ => self.value.to_string() }
    }
}

// ------------------------------------------------------------------ Hex-Vorschläge für importierte Programme

/// Vorschlag, eine Dezimaladresse als Hexzahl zu schreiben
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub value: u32,
    pub from: String,
    pub to: String,
}

/// Dezimale Adressen vollständiger Konstanten als Argument von `POKE`, `PEEK`, `SYS`, `WAIT`, `USR` (Wert ≥ 256), die `known` kennt
/// (Chipregister oder Speicherkarte des Zielsystems). Rechenausdrücke, `DATA` und Zeichenketten nie. Der Vorschlag ändert nichts:
/// erst `apply_suggestions` schreibt die ausgewählten.
pub fn suggest_hex(text: &str, lx: &Lexicon, o: &NumberOptions, known: &dyn Fn(u32) -> bool) -> Vec<Suggestion> {
    let c = ConvertOptions::default();
    let mut out = vec![];
    for (ln, line) in text.split('\n').enumerate() {
        let ch: Vec<char> = line.chars().collect();
        for n in scan_line(line, lx, o).nums {
            if n.role != Role::Address || n.radix != 10 || n.value < 256 || !known(n.value) { continue; }
            out.push(Suggestion { line: ln, start: n.start, end: n.end, value: n.value, from: ch[n.start..n.end].iter().collect(), to: format_num(n.value, 16, Role::Address, &c, 0) });
        }
    }
    out
}

/// Schreibt die Vorschläge mit `chosen[i] == true` in den Text
pub fn apply_suggestions(text: &str, sug: &[Suggestion], chosen: &[bool]) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut res: Vec<String> = vec![];
    for (ln, line) in lines.iter().enumerate() {
        let mut picks: Vec<&Suggestion> = sug.iter().enumerate().filter(|(i, s)| s.line == ln && chosen.get(*i).copied().unwrap_or(false)).map(|(_, s)| s).collect();
        picks.sort_by_key(|s| s.start);
        let ch: Vec<char> = line.chars().collect();
        let (mut at, mut s) = (0, String::new());
        for p in picks { s.extend(&ch[at..p.start]); s.push_str(&p.to); at = p.end; }
        s.extend(&ch[at..]);
        res.push(s);
    }
    res.join("\n")
}

// ------------------------------------------------------------------ Rechner

/// Wertet einen Ausdruck aus Zahlen (`255`, `$FF`, `%1010`) und `+ - * / ( ) AND OR XOR << >>` aus (64-Bit-Ganzzahlen, `/` ganzzahlig, Rangfolge wie in C).
pub fn calc(expr: &str) -> Result<i64, String> {
    #[derive(Debug, Clone, PartialEq)]
    enum T { N(i64), Op(&'static str), L, R }
    let ch: Vec<char> = expr.chars().collect();
    let (mut t, mut i) = (vec![], 0);
    while i < ch.len() {
        let c = ch[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == '(' { t.push(T::L); i += 1; continue; }
        if c == ')' { t.push(T::R); i += 1; continue; }
        if "+-*/".contains(c) { t.push(T::Op(match c { '+' => "+", '-' => "-", '*' => "*", _ => "/" })); i += 1; continue; }
        if c == '<' && ch.get(i + 1) == Some(&'<') { t.push(T::Op("<<")); i += 2; continue; }
        if c == '>' && ch.get(i + 1) == Some(&'>') { t.push(T::Op(">>")); i += 2; continue; }
        if c.is_ascii_alphabetic() {
            let w: String = ch[i..].iter().take_while(|x| x.is_ascii_alphabetic()).collect::<String>().to_ascii_uppercase();
            match w.as_str() { "AND" => t.push(T::Op("&")), "OR" => t.push(T::Op("|")), "XOR" => t.push(T::Op("^")), _ => return Err(vc_i18n::t!("basic-calc-unknown-word", w = w.as_str())) }
            i += w.len(); continue;
        }
        if c == '$' || c == '%' || c.is_ascii_digit() {
            let st = i; i += 1;
            while i < ch.len() && (ch[i].is_ascii_alphanumeric() || ch[i] == '_') { i += 1; }
            let s: String = ch[st..i].iter().collect();
            t.push(T::N(parse_number(&s).ok_or_else(|| vc_i18n::t!("basic-calc-bad-number", s = s.as_str()))?));
            continue;
        }
        return Err(vc_i18n::t!("basic-calc-unexpected-char", c = c));
    }
    struct P { t: Vec<T>, i: usize }
    impl P {
        fn prec(op: &str) -> u8 { match op { "|" => 1, "^" => 2, "&" => 3, "<<" | ">>" => 4, "+" | "-" => 5, _ => 6 } }
        fn unary(&mut self) -> Result<i64, String> {
            match self.t.get(self.i).cloned() {
                Some(T::Op("-")) => { self.i += 1; Ok(-self.unary()?) }
                Some(T::Op("+")) => { self.i += 1; self.unary() }
                Some(T::N(v)) => { self.i += 1; Ok(v) }
                Some(T::L) => { self.i += 1; let v = self.expr(0)?; if self.t.get(self.i) != Some(&T::R) { return Err(vc_i18n::t!("basic-calc-rparen-missing")); } self.i += 1; Ok(v) }
                _ => Err(vc_i18n::t!("basic-calc-number-expected")),
            }
        }
        fn expr(&mut self, min: u8) -> Result<i64, String> {
            let mut l = self.unary()?;
            while let Some(T::Op(op)) = self.t.get(self.i).cloned() {
                let p = P::prec(op);
                if p < min || p == 0 { break; }
                self.i += 1;
                let r = self.expr(p + 1)?;
                l = match op {
                    "+" => l.checked_add(r).ok_or_else(|| vc_i18n::t!("basic-calc-overflow"))?, "-" => l.checked_sub(r).ok_or_else(|| vc_i18n::t!("basic-calc-overflow"))?, "*" => l.checked_mul(r).ok_or_else(|| vc_i18n::t!("basic-calc-overflow"))?,
                    "/" => { if r == 0 { return Err(vc_i18n::t!("basic-calc-div-zero")); } l / r }
                    "<<" => { if !(0..64).contains(&r) { return Err(vc_i18n::t!("basic-calc-shift-range")); } l.checked_shl(r as u32).ok_or_else(|| vc_i18n::t!("basic-calc-overflow"))? }
                    ">>" => { if !(0..64).contains(&r) { return Err(vc_i18n::t!("basic-calc-shift-range")); } l >> r }
                    "&" => l & r, "|" => l | r, _ => l ^ r,
                };
            }
            Ok(l)
        }
    }
    if t.is_empty() { return Err(vc_i18n::t!("basic-calc-empty")); }
    let mut p = P { t, i: 0 };
    let v = p.expr(0)?;
    if p.i != p.t.len() { return Err(vc_i18n::t!("basic-calc-trailing")); }
    Ok(v)
}
