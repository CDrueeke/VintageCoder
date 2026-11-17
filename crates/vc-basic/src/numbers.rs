// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenpass: `$hex` und `%binär` im nativen BASIC (Konzept „BASIC-Zahlenformate“, Stufe 0).
//!
//! Der Pass läuft nach `number_lines` und vor `apply_long_names` und dem Tokenisieren. Er schreibt Hex- und Binärzahlen je Dialekt um:
//! Dialekte ohne Literale bekommen Dezimalzahlen, Turbo-BASIC XL behält `$` (Token `hexConst`), BASIC 65 kann beides behalten
//! (Einstellung `native`). Zeichenketten, `REM` und Text in `DATA` bleiben unverändert; vollständige `DATA`-Elemente (`DATA $FF,%1010`) werden umgewandelt.
//!
//! `scan_line` ist die **eine** Erkennung: Der Übersetzer und (später) der Editor-Hover benutzen dieselbe Funktion.
//! Alle Positionen sind **Zeichenindizes** (nicht Bytes) in der übergebenen Zeile.
use crate::atari::AtariDialect;
use crate::dialect::Dialect;
use vc_core::Diagnostic;

/// Fähigkeit eines Dialekts (`numberLiterals` in `data/basic/*.dialect.json`); fehlt das Feld, kennt der Dialekt keine Literale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberRules {
    /// der Interpreter liest `$hex` selbst
    pub hex: bool,
    /// der Interpreter liest `%binär` selbst
    pub bin: bool,
    /// größte Bitbreite eines Literals
    pub bits: u8,
    /// Vorgabe: Literale, die der Dialekt kennt, bleiben im erzeugten Programm stehen (Turbo-BASIC XL: ja, BASIC 65: nein)
    pub keep_default: bool,
    /// `%0`–`%3` allein sind eigene Konstanten (Turbo-BASIC XL, Token `per0`–`per3`)
    pub single_digit_constants: bool,
}

impl Default for NumberRules {
    fn default() -> Self { NumberRules { hex: false, bin: false, bits: 32, keep_default: false, single_digit_constants: false } }
}

impl NumberRules {
    pub(crate) fn from_json(v: &serde_json::Value) -> NumberRules {
        let n = &v["numberLiterals"];
        if !n.is_object() { return NumberRules::default(); }
        NumberRules {
            hex: n["hex"].as_bool().unwrap_or(false),
            bin: n["bin"].as_bool().unwrap_or(false),
            bits: n["bits"].as_u64().unwrap_or(32).clamp(1, 32) as u8,
            keep_default: n["keepByDefault"].as_bool().unwrap_or(false),
            single_digit_constants: n["singleDigitConstants"].as_bool().unwrap_or(false),
        }
    }
}

/// Prüfung von Konstanten gegen den Wertebereich der Verwendung (B134, B135)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeCheck { Off, Warn, Error }

/// Einstellungen des Zahlenpasses
#[derive(Debug, Clone)]
pub struct NumberOptions {
    /// Literale, die der Dialekt selbst kennt, stehen lassen: `None` = Vorgabe des Dialekts (`keepByDefault`)
    pub native: Option<bool>,
    /// `_` als Trenner zulassen (`$D0_20`, `%0000_1110`)
    pub underscore: bool,
    /// vollständige `DATA`-Elemente umwandeln
    pub data: bool,
    /// Der Quelltext hat `#longnames`: Namen sind ganze Wörter (kein Schlüsselwort mitten im Namen wie `ON` in `position`)
    pub whole_words: bool,
    /// Adresse über 65535 und `POKE`-Wert über 255 (B134, B135): Warnung, Fehler (›streng‹) oder aus. B137 ist immer nur ein Hinweis
    pub ranges: RangeCheck,
}

impl Default for NumberOptions {
    fn default() -> Self { NumberOptions { native: None, underscore: true, data: true, whole_words: false, ranges: RangeCheck::Warn } }
}

static DEFAULTS: std::sync::RwLock<NumberOptions> = std::sync::RwLock::new(NumberOptions { native: None, underscore: true, data: true, whole_words: false, ranges: RangeCheck::Warn });

/// Vorgaben für `PlainOptions::new` (die IDE setzt sie aus ihren Einstellungen; Kommandozeile: Standardwerte)
pub fn defaults() -> NumberOptions { DEFAULTS.read().map(|o| o.clone()).unwrap_or_default() }
pub fn set_defaults(o: NumberOptions) { if let Ok(mut g) = DEFAULTS.write() { *g = o; } }

/// Wörterbuch des Dialekts: wo steht ein Schlüsselwort?
#[derive(Clone, Copy)]
pub enum Lexicon<'a> { Cbm(&'a Dialect), Atari(&'a AtariDialect) }

/// Wörter, die der Atari-Ausdrucksparser außer Anweisungen und Funktionen kennt
const ATARI_WORDS: [&str; 9] = ["AND", "OR", "NOT", "TO", "STEP", "THEN", "DIV", "MOD", "EXOR"];

impl<'a> Lexicon<'a> {
    pub fn rules(&self) -> NumberRules { match self { Lexicon::Cbm(d) => d.number_literals, Lexicon::Atari(d) => d.number_literals } }
    /// `DATA` läuft bis zum Zeilenende (Atari) statt bis `:` (CBM)
    fn data_to_eol(&self) -> bool { matches!(self, Lexicon::Atari(_)) }
    /// Länge des Schlüsselworts an Position `i` (in Zeichen)
    fn keyword_at(&self, ch: &[char], i: usize, whole_words: bool) -> Option<usize> {
        match self {
            // das ROM sucht an jeder Stelle des Textes (auch mitten in einem Namen)
            Lexicon::Cbm(d) if !whole_words => crate::classic::match_keyword(ch, i, d).map(|(_, n)| n),
            // `#longnames`: ganze Wörter wie im Namenspass (`longnames::Target::keyword`)
            Lexicon::Cbm(d) => {
                let w = |c: char| c.is_ascii_alphanumeric() || c == '_';
                if i > 0 && w(ch[i - 1]) { return None; }
                let mut j = i;
                while j < ch.len() && w(ch[j]) { j += 1; }
                let word: String = ch[i..j].iter().collect::<String>().to_ascii_uppercase();
                if let Some(n) = ch.get(j).filter(|n| matches!(n, '$' | '(' | '#')) { if d.token_bytes(&format!("{word}{n}")).is_some() { return Some(j + 1 - i); } }
                if d.token_bytes(&word).is_some() { Some(j - i) } else { None }
            }
            // der Atari liest ein ganzes Wort; nur an Wortgrenzen
            Lexicon::Atari(d) => {
                if i > 0 && ch[i - 1].is_ascii_alphanumeric() { return None; }
                let mut j = i;
                while j < ch.len() && ch[j].is_ascii_alphanumeric() { j += 1; }
                let word: String = ch[i..j].iter().collect::<String>().to_ascii_uppercase();
                let known = |w: &str| d.statements.iter().any(|(s, _)| s == w) || d.functions.iter().any(|(s, _)| s == w) || ATARI_WORDS.contains(&w);
                if ch.get(j) == Some(&'$') && known(&format!("{word}$")) { return Some(j + 1 - i); }
                if known(&word) { Some(j - i) } else { None }
            }
        }
    }
}

/// Ein gefundenes Literal (`$FF`, `%00001110`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lit {
    /// Zeichenindex von `$`/`%` und hinter der letzten Ziffer
    pub start: usize,
    pub end: usize,
    /// 16 oder 2
    pub radix: u32,
    /// Ziffernzahl ohne `_` (bestimmt die Anzeigebreite: 8 Binärziffern = 8 Bit)
    pub digits: usize,
    /// Wert; `None` bei ungültigem oder zu großem Literal
    pub value: Option<u32>,
    /// das Literal ist ein vollständiges `DATA`-Element
    pub in_data: bool,
    /// das Literal steht als Zeilenziel (`GOTO $0100`)
    pub jump_target: bool,
    /// normalisierte Schreibweise (`$D020`: Großbuchstaben, ohne `_`)
    pub text: String,
}

/// Meldung zu einer Stelle der Zeile
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub start: usize,
    pub end: usize,
    pub code: &'static str,
    pub error: bool,
    pub message: String,
}

/// Wozu eine Zahl im Programm dient
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Adresse als Argument von `POKE`, `PEEK`, `SYS`, `WAIT` (und `USR` beim Atari)
    Address,
    /// Byte-Wert: zweites Argument von `POKE`, Maske von `WAIT`
    Value,
    /// Konstante in einem sonstigen Ausdruck
    Other,
    /// vollständiges `DATA`-Element
    Data,
    /// Zeilenziel (`GOTO 100`): nie umschreiben
    Target,
}

/// Eine ganze Zahl im Quelltext, gleich in welcher Schreibweise
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Num {
    pub start: usize,
    pub end: usize,
    /// 10, 16 oder 2
    pub radix: u32,
    /// Ziffernzahl ohne `_` und Vorzeichen
    pub digits: usize,
    pub value: u32,
    pub role: Role,
}

#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub lits: Vec<Lit>,
    pub notes: Vec<Note>,
    /// alle ganzen Zahlen mit gültigem Wert (dezimal, hex, binär), nach Position; ohne negative Dezimalzahlen und ohne Zeilennummer
    pub nums: Vec<Num>,
}

struct Parsed { lit: Lit, notes: Vec<Note> }

fn is_digit_of(radix: u32, c: char) -> bool { if radix == 16 { c.is_ascii_hexdigit() } else { c == '0' || c == '1' } }

/// Liest das Literal ab `i` (`ch[i]` ist `$` oder `%`)
fn parse_lit(ch: &[char], i: usize, o: &NumberOptions, lx: &Lexicon, in_data: bool) -> Parsed {
    let radix = if ch[i] == '$' { 16 } else { 2 };
    let kind = if radix == 16 { "Hex" } else { "Binär" };
    let (code_digit, name) = if radix == 16 { ("B130", "Hexziffer") } else { ("B131", "Binärziffer") };
    let mut notes = vec![];
    let mut j = i + 1;
    while j < ch.len() && (is_digit_of(radix, ch[j]) || (o.underscore && ch[j] == '_')) { j += 1; }
    let run: String = ch[i + 1..j].iter().collect();
    let digits: String = run.chars().filter(|c| *c != '_').collect();
    let err = |start, end, code, message: String| Note { start, end, code, error: true, message };
    if run.starts_with('_') || run.ends_with('_') || run.contains("__") {
        notes.push(err(i, j, code_digit, "Trenner _ nur einzeln zwischen zwei Ziffern".into()));
    }
    let mut value = None;
    if digits.is_empty() {
        // `$` ohne Ziffern: steht dahinter ein Zeichen, das keine Ziffer ist, wird es genannt
        match ch.get(j) {
            Some(c) if radix == 2 && c.is_ascii_digit() => notes.push(err(i, j + 1, "B131", format!("'{c}' ist keine Binärziffer (nur 0 und 1)"))),
            Some(c) if radix == 16 && c.is_ascii_alphabetic() => notes.push(err(i, j + 1, "B130", format!("'{c}' ist keine Hexziffer (0–9, A–F)"))),
            _ => notes.push(err(i, j.max(i + 1), "B132", format!("{} ohne Ziffern", if radix == 16 { "$" } else { "%" }))),
        }
    } else {
        // Zeichen direkt hinter der Zahl
        if let Some(&c) = ch.get(j) {
            if radix == 2 && c.is_ascii_digit() {
                notes.push(err(i, j + 1, "B131", format!("'{c}' ist keine Binärziffer (nur 0 und 1)")));
            } else if c.is_ascii_alphabetic() || (radix == 16 && c == '_') {
                match lx.keyword_at(ch, j, o.whole_words) {
                    Some(n) if radix == 16 => {
                        let kw: String = ch[j..j + n].iter().collect();
                        notes.push(Note { start: j, end: j + n, code: "B136", error: false, message: format!("Hexzahl grenzt an das Schlüsselwort {kw}: Leerzeichen setzen (Ziffern A–F könnten zum Wort gehören)") });
                    }
                    Some(_) => {}
                    None => notes.push(err(i, j + 1, code_digit, format!("'{c}' ist keine {name}{}", if radix == 16 { " (vor einem Schlüsselwort ein Leerzeichen setzen)" } else { "" }))),
                }
            }
        }
        // Wert
        let mut v: u64 = 0;
        let mut over = false;
        for c in digits.chars() { v = v * radix as u64 + c.to_digit(radix).unwrap() as u64; if v > u32::MAX as u64 { over = true; v = u32::MAX as u64 + 1; } }
        let bits = lx.rules().bits as u32;
        let max = if bits >= 32 { u32::MAX as u64 } else { (1u64 << bits) - 1 };
        if over || v > max {
            notes.push(err(i, j, "B133", format!("{kind}zahl über {bits} Bit (größter Wert {max})")));
        } else if !notes.iter().any(|n| n.error) { value = Some(v as u32); }
    }
    let text = format!("{}{}", ch[i], digits.to_ascii_uppercase());
    let _ = in_data;
    Parsed { lit: Lit { start: i, end: j, radix, digits: digits.len(), value, in_data, jump_target: false, text }, notes }
}

/// Alle Literale und Hinweise einer Zeile.
/// Die Zeile darf mit einer Zeilennummer beginnen (wird übersprungen). Kommentar- (`;`) und Richtlinienzeilen (`#…`) enthalten nichts.
pub fn scan_line(line: &str, lx: &Lexicon, o: &NumberOptions) -> Scan {
    let ch: Vec<char> = line.chars().collect();
    let n = ch.len();
    let mut out = Scan::default();
    let mut i = 0;
    while i < n && ch[i].is_whitespace() { i += 1; }
    if i < n && (ch[i] == ';' || ch[i] == '#') { return out; }
    while i < n && ch[i].is_ascii_digit() { i += 1; }
    let rules = lx.rules();
    let (mut quote, mut in_name, mut target, mut prev_go) = (false, false, false, false);
    let mut roles: Vec<(usize, Role)> = vec![];
    let mut decimals: Vec<Num> = vec![];
    while i < n {
        let c = ch[i];
        if c == '"' { quote = !quote; in_name = false; target = false; i += 1; continue; }
        if quote { i += 1; continue; }
        if c == '{' { while i < n && ch[i] != '}' { i += 1; } i = (i + 1).min(n); in_name = false; continue; }
        if c == '@' { i += 1; while i < n && (ch[i].is_ascii_alphanumeric() || ch[i] == '_') { i += 1; } in_name = false; continue; }
        // Zeilenziele: nur Leerzeichen, Ziffern und Komma halten den Zustand
        if !(c == ' ' || c == ',' || c.is_ascii_digit() || c == '$' || c == '%') { target = false; }
        if c == '$' || c == '%' {
            if in_name {                                                    // Typkennzeichen: `A$`, `A%`
                if c == '%' && o.ranges != RangeCheck::Off && matches!(lx, Lexicon::Cbm(_)) { check_int_assign(&ch, i + 1, o, lx, &mut out); }
                in_name = false; i += 1; continue;
            }
            if c == '%' && rules.single_digit_constants && matches!(ch.get(i + 1), Some('0'..='3')) && !matches!(ch.get(i + 2), Some('0' | '1')) { i += 2; continue; }   // Turbo: %0..%3
            let mut p = parse_lit(&ch, i, o, lx, false);
            p.lit.jump_target = target;
            if target {
                p.notes.push(Note { start: i, end: p.lit.end, code: "B138", error: true, message: "Hex/Binär als Sprungziel: Zeilennummern entstehen beim Bauen, Marke verwenden (@name)".into() });
                p.lit.value = None;
            }
            i = p.lit.end.max(i + 1);
            out.notes.extend(p.notes);
            out.lits.push(p.lit);
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && ch.get(i + 1).map_or(false, char::is_ascii_digit)) {
            if in_name { i += 1; continue; }
            // `0xFF` ist kein BASIC
            if c == '0' && matches!(ch.get(i + 1), Some('x' | 'X')) && ch.get(i + 2).map_or(false, char::is_ascii_hexdigit) {
                let mut j = i + 2;
                while j < n && ch[j].is_ascii_hexdigit() { j += 1; }
                out.notes.push(Note { start: i, end: j, code: "B139", error: false, message: "0x… ist kein BASIC: Hexzahlen schreibt man $FF".into() });
                i = j; continue;
            }
            let st = i;
            while i < n && (ch[i].is_ascii_digit() || ch[i] == '.') { i += 1; }
            let mut plain_int = ch[st..i].iter().all(char::is_ascii_digit);
            if matches!(ch.get(i), Some('e' | 'E')) {
                let sign = matches!(ch.get(i + 1), Some('+' | '-'));
                if ch.get(i + 1 + sign as usize).map_or(false, char::is_ascii_digit) {
                    plain_int = false;
                    i += 1 + sign as usize;
                    while i < n && ch[i].is_ascii_digit() { i += 1; }
                }
            }
            if plain_int && !unary_minus(&ch, st) {
                if let Ok(v) = ch[st..i].iter().collect::<String>().parse::<u32>() {
                    let role = if target { Role::Target } else { roles.iter().find(|r| r.0 == st).map_or(Role::Other, |r| r.1) };
                    decimals.push(Num { start: st, end: i, radix: 10, digits: i - st, value: v, role });
                }
            }
            continue;
        }
        if c.is_ascii_alphabetic() {
            if let Some(len) = lx.keyword_at(&ch, i, o.whole_words) {
                let name: String = ch[i..i + len].iter().collect::<String>().to_ascii_uppercase();
                in_name = false;
                i += len;
                if name == "REM" { break; }
                if name == "DATA" {
                    if o.data { i = scan_data(&ch, i, lx, o, &mut out); } else { i = skip_data(&ch, i, lx); }
                    target = false;
                    continue;
                }
                check_args(&ch, i, &name, lx, o, &mut out, &mut roles);
                target = matches!(name.as_str(), "GOTO" | "GOSUB" | "RESTORE" | "RUN" | "TRAP" | "THEN") || (name == "TO" && prev_go);
                prev_go = name == "GO";
                continue;
            }
            prev_go = false;
            in_name = true; i += 1; continue;
        }
        if c == '_' { in_name = true; i += 1; continue; }
        in_name = false;
        i += 1;
    }
    // alle Zahlen nach Position: Dezimalzahlen, DATA-Elemente und gültige Hex-/Binärliterale
    let mut nums = std::mem::take(&mut out.nums);
    nums.extend(decimals);
    for l in &out.lits {
        let Some(v) = l.value else { continue };
        let role = if l.jump_target { Role::Target } else if l.in_data { Role::Data } else { roles.iter().find(|r| r.0 == l.start).map_or(Role::Other, |r| r.1) };
        nums.push(Num { start: l.start, end: l.end, radix: l.radix, digits: l.digits, value: v, role });
    }
    nums.sort_by_key(|x| x.start);
    out.nums = nums;
    out
}

/// Steht vor der Zahl ab `st` ein Vorzeichen-Minus (kein Rechenzeichen zwischen zwei Operanden)?
fn unary_minus(ch: &[char], st: usize) -> bool {
    let mut k = st;
    while k > 0 && ch[k - 1] == ' ' { k -= 1; }
    if k == 0 || ch[k - 1] != '-' { return false; }
    let mut p = k - 1;
    while p > 0 && ch[p - 1] == ' ' { p -= 1; }
    !(p > 0 && (ch[p - 1].is_ascii_alphanumeric() || matches!(ch[p - 1], ')' | '$' | '%' | '_' | '"')))
}

/// Eine ganze Konstante als Argument (`-5`, `53280`, `$D020`, `%101`) ab `i`: (Anfang, Ende, Wert, Index hinter dem Argument).
/// `None`, wenn das Argument ein Ausdruck, eine Kommazahl oder ungültig ist.
fn const_arg(ch: &[char], mut i: usize, o: &NumberOptions, lx: &Lexicon) -> Option<(usize, usize, i64, usize)> {
    let n = ch.len();
    while i < n && ch[i] == ' ' { i += 1; }
    let start = i;
    let neg = ch.get(i) == Some(&'-');
    if neg { i += 1; while i < n && ch[i] == ' ' { i += 1; } }
    let (value, end) = match ch.get(i)? {
        c if c.is_ascii_digit() => {
            let mut j = i;
            while j < n && ch[j].is_ascii_digit() { j += 1; }
            if matches!(ch.get(j), Some('.' | 'e' | 'E')) { return None; }
            let v: i64 = ch[i..j].iter().collect::<String>().parse().unwrap_or(i64::MAX / 2);
            (v, j)
        }
        '$' | '%' => { let p = parse_lit(ch, i, o, lx, false); if p.notes.iter().any(|x| x.error) { return None; } (p.lit.value? as i64, p.lit.end) }
        _ => return None,
    };
    let mut k = end;
    while k < n && ch[k] == ' ' { k += 1; }
    if k < n && !matches!(ch[k], ',' | ')' | ':') { return None; }
    Some((start, end, if neg { -value } else { value }, k))
}

/// B134/B135: Konstanten hinter `POKE`, `PEEK`, `SYS`, `WAIT` (und `USR` beim Atari)
fn check_args(ch: &[char], mut i: usize, kw: &str, lx: &Lexicon, o: &NumberOptions, out: &mut Scan, roles_out: &mut Vec<(usize, Role)>) {
    // Rollen der Argumente: A = Adresse (0–65535), B = Byte (0–255)
    let roles: &[char] = match kw { "POKE" => &['A', 'B'], "PEEK" | "SYS" => &['A'], "WAIT" => &['A', 'B', 'B'], "USR" if matches!(lx, Lexicon::Atari(_)) => &['A'], _ => return };
    if matches!(kw, "PEEK" | "USR") { while i < ch.len() && ch[i] == ' ' { i += 1; } if ch.get(i) == Some(&'(') { i += 1; } else { return; } }
    let error = o.ranges == RangeCheck::Error;
    for role in roles {
        let Some((s, e, v, next)) = const_arg(ch, i, o, lx) else { return };
        if ch.get(s) != Some(&'-') { roles_out.push((s, if *role == 'A' { Role::Address } else { Role::Value })); }
        if o.ranges == RangeCheck::Off { if ch.get(next) != Some(&',') { return; } i = next + 1; continue; }
        match role {
            'A' if !(0..=65535).contains(&v) => out.notes.push(Note { start: s, end: e, code: "B134", error, message: format!("Adresse {v} außerhalb 0–65535") }),
            'B' if !(0..=255).contains(&v) => out.notes.push(Note { start: s, end: e, code: "B135", error, message: format!("Wert {v} außerhalb 0–255 ({kw} schreibt ein Byte)") }),
            _ => {}
        }
        if ch.get(next) != Some(&',') { return; }
        i = next + 1;
    }
}

/// B137: `A%=40000` (Ganzzahlvariable hält −32768 … 32767)
fn check_int_assign(ch: &[char], mut i: usize, o: &NumberOptions, lx: &Lexicon, out: &mut Scan) {
    while i < ch.len() && ch[i] == ' ' { i += 1; }
    if ch.get(i) != Some(&'=') { return; }
    if let Some((s, e, v, _)) = const_arg(ch, i + 1, o, lx) {
        if !(-32768..=32767).contains(&v) { out.notes.push(Note { start: s, end: e, code: "B137", error: false, message: format!("{v} passt nicht in eine Ganzzahlvariable (−32768 … 32767)") }); }
    }
}

/// Springt über einen `DATA`-Teil (bis `:` bei CBM, sonst bis Zeilenende); Rückgabe: Index des Endes
fn skip_data(ch: &[char], mut i: usize, lx: &Lexicon) -> usize {
    let mut q = false;
    while i < ch.len() {
        if ch[i] == '"' { q = !q; } else if !q && ch[i] == ':' && !lx.data_to_eol() { return i; }
        i += 1;
    }
    i
}

/// Liest die Elemente eines `DATA`-Teils; vollständige gültige Literale werden gemerkt
fn scan_data(ch: &[char], mut i: usize, lx: &Lexicon, o: &NumberOptions, out: &mut Scan) -> usize {
    let n = ch.len();
    loop {
        while i < n && ch[i] == ' ' { i += 1; }
        if i >= n { return n; }
        if ch[i].is_ascii_digit() {
            let mut j = i;
            while j < n && ch[j].is_ascii_digit() { j += 1; }
            let mut k = j;
            while k < n && ch[k] == ' ' { k += 1; }
            let whole = k >= n || ch[k] == ',' || (ch[k] == ':' && !lx.data_to_eol());
            if whole { if let Ok(v) = ch[i..j].iter().collect::<String>().parse::<u32>() { out.nums.push(Num { start: i, end: j, radix: 10, digits: j - i, value: v, role: Role::Data }); } }
        }
        if ch[i] == '$' || ch[i] == '%' {
            let p = parse_lit(ch, i, o, lx, true);
            let mut k = p.lit.end;
            while k < n && ch[k] == ' ' { k += 1; }
            let whole = k >= n || ch[k] == ',' || (ch[k] == ':' && !lx.data_to_eol());
            if whole && p.lit.value.is_some() && p.notes.iter().all(|x| !x.error) { out.lits.push(p.lit); out.notes.extend(p.notes); i = k; }
        }
        // Rest des Elements überspringen
        let mut q = false;
        while i < n {
            if ch[i] == '"' { q = !q; }
            else if !q && ch[i] == ',' { break; }
            else if !q && ch[i] == ':' && !lx.data_to_eol() { return i; }
            i += 1;
        }
        if i >= n { return n; }
        i += 1;   // das Komma
    }
}

/// Neue Schreibweise des Literals im erzeugten Programm; `None` = unverändert lassen
pub fn replacement(l: &Lit, lx: &Lexicon, o: &NumberOptions) -> Option<String> {
    let v = l.value?;
    let r = lx.rules();
    let keep = o.native.unwrap_or(r.keep_default);
    let known = if l.radix == 16 { r.hex } else { r.bin };
    if keep && known && !l.in_data { return Some(l.text.clone()); }
    Some(v.to_string())
}

/// Ergebnis des Pass
#[derive(Debug, Clone, Default)]
pub struct Converted {
    pub text: String,
    /// Warnungen (B136, B139)
    pub warnings: Vec<Diagnostic>,
    /// Zahl der umgeschriebenen Literale
    pub count: usize,
}

/// Wandelt alle Literale des nummerierten Textes um. Fehler (B130–B133, B138) sammelt der Pass über alle Zeilen.
pub fn convert_numbers(text: &str, lx: &Lexicon, o: &NumberOptions, file: &str) -> Result<Converted, Vec<Diagnostic>> {
    let (mut errs, mut warns, mut count) = (vec![], vec![], 0usize);
    let mut out = String::with_capacity(text.len());
    for (n, line) in text.lines().enumerate() {
        let s = scan_line(line, lx, o);
        for note in &s.notes {
            // Fehlercodes sind 'static: die Meldung trägt der Text
            let d = if note.error { Diagnostic::error(note.code, file, n + 1, note.message.clone()) } else { Diagnostic::warning(note.code, file, n + 1, note.message.clone()) };
            if note.error { errs.push(d) } else { warns.push(d) }
        }
        if s.lits.is_empty() { out.push_str(line); out.push('\n'); continue; }
        let ch: Vec<char> = line.chars().collect();
        let mut at = 0;
        for l in &s.lits {
            let Some(new) = replacement(l, lx, o) else { continue };
            out.extend(&ch[at..l.start]);
            // `PRINT$FF` → `PRINT 255`: hinter einem Schlüsselwort braucht die Zahl ein Leerzeichen (sonst würde `PRINT255` ein Name)
            if l.start > 0 && ch[l.start - 1].is_ascii_alphanumeric() { out.push(' '); }
            out.push_str(&new);
            at = l.end;
            count += 1;
        }
        out.extend(&ch[at..]);
        out.push('\n');
    }
    if !errs.is_empty() { return Err(errs); }
    Ok(Converted { text: out, warnings: warns, count })
}

/// B140: Der klassische Weg (`vcc basic --tokenize`) wandelt nichts um. Meldet jedes Literal, das der Dialekt nicht selbst liest
/// (Hex bzw. Binär im Quelltext eines Dialekts ohne Literale); der Anwender soll nativ bauen oder Dezimalzahlen schreiben.
pub fn classic_warnings(text: &str, lx: &Lexicon, file: &str) -> Vec<Diagnostic> {
    let r = lx.rules();
    let o = NumberOptions::default();
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        if !line.contains(['$', '%']) { continue; }
        for l in scan_line(line, lx, &o).lits {
            if l.value.is_none() || (l.radix == 16 && r.hex) || (l.radix == 2 && r.bin) { continue; }
            let t: String = line.chars().skip(l.start).take(l.end - l.start).collect();
            out.push(Diagnostic::warning("B140", file, n + 1, format!("{t} wird im klassischen Weg nicht umgewandelt, der Dialekt kennt keine Hex-/Binärzahlen (als natives BASIC bauen: vcc basic {file}, oder Dezimalzahl schreiben)")));
        }
    }
    out
}
