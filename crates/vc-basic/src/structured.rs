// SPDX-License-Identifier: GPL-3.0-or-later
//! Strukturiertes BASIC (ohne Zeilennummern) → tokenisierte Zeilen.
//! Labels (`@name:`), `const`, `include`, mehrzeilige `if/else/endif`, `while/wend`, `repeat/until`,
//! lange Variablennamen (automatisch auf 2 Zeichen abgebildet), optionales Zusammenfassen von Zeilen.
use crate::detok::build_prg;
use crate::dialect::Dialect;
use crate::petscii;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use vc_core::Diagnostic;

#[derive(Clone)]
pub struct Options {
    pub file: String,
    pub start: u32,
    pub step: u32,
    pub pack: bool,
    pub load_addr: Option<u16>,
    /// Vordefinierte Konstanten, z. B. Symbole aus dem Assembler (`asm.start` → 2061).
    pub consts: Vec<(String, i64)>,
    pub include_dirs: Vec<PathBuf>,
    /// Zielsystem (`c64`, `c128`, `plus4`, `vic20`, `mega65`, `atari8`): setzt `SYSTEM_*` für `#if` und ersetzt `{system}` in `include`-Pfaden.
    pub system: Option<String>,
    /// Maschinencode (Adresse, Bytes) als DATA-Zeilen ab `@blob_data` (Format je Block: `adresse, anzahl, werte…`, Ende: `0`), für Systeme, die Code hinter dem BASIC-Text nicht schützen
    pub blob_chunks: Vec<(u16, Vec<u8>)>,
}

impl Options {
    pub fn new(file: impl Into<String>) -> Self {
        Options { file: file.into(), start: 10, step: 10, pack: false, load_addr: None, consts: vec![], include_dirs: vec![], system: None, blob_chunks: vec![] }
    }
}

/// `blob "player.asm" with "song.asm"`: Maschinencode, den der Bau hinter dem BASIC-Text ins PRG legt.
#[derive(Clone, Debug, PartialEq)]
pub struct BlobReq { pub file: String, pub with: Vec<String>, pub src: (String, usize) }

pub struct Compiled {
    pub blobs: Vec<BlobReq>,
    pub prg: Vec<u8>,
    pub lines: Vec<(u16, Vec<u8>)>,
    pub labels: BTreeMap<String, u16>,
    /// Quellname → generierter 2-Zeichen-Name.
    pub vars: Vec<(String, String)>,
    pub warnings: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
enum Piece { B(Vec<u8>), Var(String), Label(String) }

struct PLine { src: (String, usize), labels: Vec<String>, pieces: Vec<Piece> }

enum Block {
    If { els: String, end: String, has_else: bool, at: (String, usize) },
    While { top: String, end: String, at: (String, usize) },
    Repeat { top: String, end: String, at: (String, usize) },
    /// `do … loop`: `native` = das Zielsystem kennt DO/LOOP/EXIT (BASIC 3.5/7/65), sonst werden Sprünge erzeugt
    Do { top: String, end: String, native: bool, at: (String, usize) },
    Proc { skip: String, at: (String, usize) },
}

/// Teilt `a, f(b, c), "x,y"` an den Kommas außerhalb von Klammern und Zeichenketten.
pub(crate) fn split_args(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth, mut q) = (vec![], String::new(), 0i32, false);
    for c in s.chars() {
        match c {
            '"' => { q = !q; cur.push(c); }
            '(' if !q => { depth += 1; cur.push(c); }
            ')' if !q => { depth -= 1; cur.push(c); }
            ',' if !q && depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() || !out.is_empty() { out.push(cur); }
    out.into_iter().map(|a| a.trim().to_string()).collect()
}

/// `name` oder `name(a, b$)` → Name und Parameter (kleingeschrieben)
pub(crate) fn proc_head(rest: &str) -> Result<(String, Vec<String>), String> {
    let rest = rest.trim();
    let name: String = rest.chars().take_while(|c| is_word(*c)).collect();
    if name.is_empty() || !name.chars().next().map_or(false, char::is_alphabetic) { return Err("Prozedurname fehlt".into()); }
    let tail = rest[name.len()..].trim();
    let params = if tail.is_empty() { vec![] } else {
        let inner = tail.strip_prefix('(').and_then(|t| t.strip_suffix(')')).ok_or("erwartet name(parameter, …)")?;
        split_args(inner).into_iter().map(|p| p.to_ascii_lowercase()).collect()
    };
    for p in &params {
        let base = p.trim_end_matches(|c| c == '$' || c == '%');
        if base.is_empty() || !base.chars().next().map_or(false, char::is_alphabetic) || !base.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("ungültiger Parametername '{p}'"));
        }
    }
    Ok((name.to_ascii_lowercase(), params))
}

pub(crate) struct Src { pub file: String, pub line: usize, pub text: String }

/// Umgebung der Vorverarbeitung: Zielsystem und Namen für `#if`.
pub(crate) struct Env { pub system: Option<String>, pub defs: HashMap<String, i64> }

impl Env {
    pub(crate) fn new(system: Option<&str>, consts: &[(String, i64)]) -> Env {
        let mut defs: HashMap<String, i64> = consts.iter().map(|(n, v)| (n.to_ascii_lowercase(), *v)).collect();
        let sys = system.map(|s| s.to_ascii_lowercase());
        for (name, id) in [("c64", "c64"), ("c128", "c128"), ("plus4", "plus4"), ("vic20", "vic20"), ("mega65", "mega65"), ("atari", "atari8")] {
            defs.insert(format!("system_{name}"), (sys.as_deref() == Some(id)) as i64);
        }
        Env { system: sys, defs }
    }
}

/// Ausdruck hinter `#if`: Namen, Zahlen, `and` `or` `not`, `=` `<>`, Klammern.
fn eval_cond(text: &str, env: &Env) -> Result<bool, String> {
    let ch: Vec<char> = text.chars().collect();
    let (mut toks, mut i) = (vec![], 0);
    while i < ch.len() {
        let c = ch[i];
        if c.is_whitespace() { i += 1; }
        else if c.is_alphanumeric() || c == '_' { let st = i; while i < ch.len() && (ch[i].is_alphanumeric() || ch[i] == '_') { i += 1; } toks.push(ch[st..i].iter().collect::<String>().to_ascii_lowercase()); }
        else if c == '<' && ch.get(i + 1) == Some(&'>') { toks.push("<>".into()); i += 2; }
        else if "()=".contains(c) { toks.push(c.to_string()); i += 1; }
        else { return Err(format!("Zeichen '{c}' in der Bedingung nicht erlaubt")); }
    }
    struct P<'a> { t: Vec<String>, i: usize, env: &'a Env }
    impl P<'_> {
        fn peek(&self) -> Option<&str> { self.t.get(self.i).map(String::as_str) }
        fn or(&mut self) -> Result<i64, String> { let mut v = self.and()?; while self.peek() == Some("or") { self.i += 1; let r = self.and()?; v = (v != 0 || r != 0) as i64; } Ok(v) }
        fn and(&mut self) -> Result<i64, String> { let mut v = self.not()?; while self.peek() == Some("and") { self.i += 1; let r = self.not()?; v = (v != 0 && r != 0) as i64; } Ok(v) }
        fn not(&mut self) -> Result<i64, String> { if self.peek() == Some("not") { self.i += 1; Ok((self.not()? == 0) as i64) } else { self.cmp() } }
        fn cmp(&mut self) -> Result<i64, String> {
            let a = self.atom()?;
            match self.peek() { Some("=") => { self.i += 1; Ok((a == self.atom()?) as i64) } Some("<>") => { self.i += 1; Ok((a != self.atom()?) as i64) } _ => Ok(a) }
        }
        fn atom(&mut self) -> Result<i64, String> {
            let Some(t) = self.t.get(self.i).cloned() else { return Err("Ausdruck unvollständig".into()) };
            self.i += 1;
            if t == "(" { let v = self.or()?; if self.peek() != Some(")") { return Err("')' fehlt".into()); } self.i += 1; return Ok(v); }
            if let Ok(n) = t.parse::<i64>() { return Ok(n); }
            if matches!(t.as_str(), ")" | "=" | "<>" | "and" | "or") { return Err(format!("unerwartet '{t}'")); }
            self.env.defs.get(&t).copied().ok_or(format!("unbekannter Name '{t}' (bekannt: SYSTEM_C64, SYSTEM_C128, SYSTEM_PLUS4, SYSTEM_VIC20, SYSTEM_MEGA65, SYSTEM_ATARI und -D-Konstanten)"))
        }
    }
    let mut p = P { t: toks, i: 0, env };
    let v = p.or()?;
    if p.i < p.t.len() { return Err(format!("unerwartet '{}'", p.t[p.i])); }
    Ok(v != 0)
}

struct Cond { parent: bool, taken: bool, active: bool, seen_else: bool, line: usize }

pub(crate) fn expand(text: &str, file: &str, dirs: &[PathBuf], depth: usize, out: &mut Vec<Src>, errs: &mut Vec<Diagnostic>, env: &Env) {
    let mut stack: Vec<Cond> = vec![];
    for (i, l) in text.lines().enumerate() {
        let t = l.trim();
        let low = t.to_ascii_lowercase();
        let is_dir = |d: &str| low.starts_with('#') && low[1..].starts_with(d) && !low[1 + d.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_');
        let active = stack.last().map_or(true, |c| c.active);
        if is_dir("if") {
            let v = active && eval_cond(&t[3..], env).map_err(|e| errs.push(Diagnostic::error("B111", file, i + 1, e))).unwrap_or(false);
            stack.push(Cond { parent: active, taken: v, active: v, seen_else: false, line: i + 1 });
            continue;
        }
        if is_dir("elseif") || is_dir("else") || is_dir("endif") {
            let Some(top) = stack.last_mut() else { errs.push(Diagnostic::error("B111", file, i + 1, format!("{} ohne #if", &t[..t.find(char::is_whitespace).unwrap_or(t.len())]))); continue };
            if is_dir("endif") { stack.pop(); }
            else if top.seen_else { errs.push(Diagnostic::error("B111", file, i + 1, "#elseif/#else nach #else")); }
            else if is_dir("elseif") {
                let v = top.parent && !top.taken && eval_cond(&t[7..], env).map_err(|e| errs.push(Diagnostic::error("B111", file, i + 1, e))).unwrap_or(false);
                top.active = v;
                top.taken |= v;
            } else { top.active = top.parent && !top.taken; top.taken = true; top.seen_else = true; }
            continue;
        }
        if !active { continue; }
        if low.starts_with("include ") || low.starts_with("include\t") {
            let raw = t[7..].trim().trim_matches('"');
            let name = if raw.contains("{system}") {
                match &env.system { Some(sys) => raw.replace("{system}", sys), None => { errs.push(Diagnostic::error("B111", file, i + 1, "{system} im include-Pfad, aber das Zielsystem ist unbekannt")); continue } }
            } else { raw.to_string() };
            let name = name.as_str();
            let base = Path::new(file).parent().map(Path::to_path_buf).unwrap_or_default();
            let found = std::iter::once(base).chain(dirs.iter().cloned()).map(|d| d.join(name)).find(|p| p.is_file());
            match (found, depth < 16) {
                (Some(p), true) => match std::fs::read_to_string(&p) {
                    Ok(sub) => expand(&sub, &p.to_string_lossy(), dirs, depth + 1, out, errs, env),
                    Err(e) => errs.push(Diagnostic::error("B110", file, i + 1, format!("{}: {e}", p.display()))),
                },
                (_, false) => errs.push(Diagnostic::error("B110", file, i + 1, "include zu tief verschachtelt")),
                _ => errs.push(Diagnostic::error("B110", file, i + 1, format!("Include-Datei '{name}' nicht gefunden"))),
            }
        } else { out.push(Src { file: file.to_string(), line: i + 1, text: l.to_string() }); }
    }
    for c in stack { errs.push(Diagnostic::error("B111", file, c.line, "#if ohne #endif")); }
}

/// `blob "datei" [with "…"]` → Anforderung (`None`, wenn die Zeile nicht passt)
pub(crate) fn parse_blob(t: &str, at: (String, usize)) -> Option<BlobReq> {
    let strs: Vec<&str> = t[4..].split('"').enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, x)| x).collect();
    let ok = t[4..].trim_start().starts_with('"') && !strs.is_empty() && (strs.len() == 1 || t[4..].to_ascii_lowercase().contains(" with "));
    ok.then(|| BlobReq { file: strs[0].to_string(), with: strs[1..].iter().map(|x| x.to_string()).collect(), src: at })
}

/// Sucht die `blob`-Zeilen des Programms (nach `include` und `#if`), ohne es zu übersetzen
pub fn find_blobs(src: &str, file: &str, dirs: &[PathBuf], system: Option<&str>, consts: &[(String, i64)]) -> Result<Vec<BlobReq>, Vec<Diagnostic>> {
    let (mut lines, mut errs) = (vec![], vec![]);
    expand(src, file, dirs, 0, &mut lines, &mut errs, &Env::new(system, consts));
    let mut out = vec![];
    for l in &lines {
        let t = l.text.trim();
        if first_word(t) == "blob" { match parse_blob(t, (l.file.clone(), l.line)) { Some(b) => out.push(b), None => errs.push(Diagnostic::error("B113", &l.file, l.line, "blob erwartet: blob \"datei.asm\" [with \"song.asm\"]")) } }
    }
    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

/// Quelltext nach dem Einsetzen der `include`-Anweisungen (für das Bauprotokoll): der Text und je Datei die Zahl der beitragenden Zeilen
pub fn expanded_source(text: &str, file: &str, dirs: &[PathBuf], system: Option<&str>) -> Result<(String, Vec<(String, usize)>), Vec<Diagnostic>> {
    let (mut out, mut errs) = (vec![], vec![]);
    expand(text, file, dirs, 0, &mut out, &mut errs, &Env::new(system, &[]));
    if !errs.is_empty() { return Err(errs); }
    let mut files: Vec<(String, usize)> = vec![];
    for l in &out { match files.iter_mut().find(|f| f.0 == l.file) { Some(f) => f.1 += 1, None => files.push((l.file.clone(), 1)) } }
    Ok((out.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n") + "\n", files))
}

pub(crate) fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' }

const RESERVED: &[&str] = &["const", "include", "else", "elseif", "endif", "while", "wend", "repeat", "until", "do", "loop", "exit", "proc", "endproc", "call", "blob"];

struct Lexer<'a> { d: &'a Dialect, consts: &'a HashMap<String, Vec<u8>> }

impl Lexer<'_> {
    fn number(ch: &[char], i: &mut usize) -> String {
        let st = *i;
        while *i < ch.len() && ch[*i].is_ascii_digit() { *i += 1; }
        if *i < ch.len() && ch[*i] == '.' { *i += 1; while *i < ch.len() && ch[*i].is_ascii_digit() { *i += 1; } }
        if *i < ch.len() && (ch[*i] == 'e' || ch[*i] == 'E') {
            let mut j = *i + 1;
            if j < ch.len() && (ch[j] == '+' || ch[j] == '-') { j += 1; }
            if j < ch.len() && ch[j].is_ascii_digit() { while j < ch.len() && ch[j].is_ascii_digit() { j += 1; } *i = j; }
        }
        ch[st..*i].iter().collect::<String>().to_ascii_uppercase()
    }

    /// Zerlegt eine Anweisungszeile in Teile; `Err` enthält eine Meldung.
    fn lex(&self, s: &str) -> Result<Vec<Piece>, String> {
        let d = self.d;
        let ch: Vec<char> = s.chars().collect();
        let (mut i, mut out) = (0usize, vec![]);
        let kw = |n: &str| Piece::B(d.token_bytes(n).unwrap_or_else(|| vec![d.tok(n)]));
        while i < ch.len() {
            let c = ch[i];
            if c.is_whitespace() { i += 1; continue; }
            if c == '\'' { break; }
            if c == '"' {
                let mut j = i + 1;
                while j < ch.len() && ch[j] != '"' { j += 1; }
                let inner: String = ch[i + 1..j].iter().collect();
                let mut b = vec![0x22];
                b.extend(petscii::from_text(&inner, d)?);
                b.push(0x22);
                out.push(Piece::B(b));
                i = (j + 1).min(ch.len() + 1);
                continue;
            }
            if c == '@' {
                let st = i + 1;
                i = st;
                while i < ch.len() && is_word(ch[i]) { i += 1; }
                if st == i { return Err("Label-Name nach '@' fehlt".into()); }
                out.push(Piece::Label(ch[st..i].iter().collect::<String>().to_ascii_lowercase()));
                continue;
            }
            if c.is_ascii_digit() || (c == '.' && ch.get(i + 1).map_or(false, char::is_ascii_digit)) {
                out.push(Piece::B(Self::number(&ch, &mut i).into_bytes()));
                continue;
            }
            if c == '$' && ch.get(i + 1).map_or(false, char::is_ascii_hexdigit) {
                let st = i + 1;
                i = st;
                while i < ch.len() && ch[i].is_ascii_hexdigit() { i += 1; }
                let v = i64::from_str_radix(&ch[st..i].iter().collect::<String>(), 16).map_err(|e| e.to_string())?;
                out.push(Piece::B(v.to_string().into_bytes()));
                continue;
            }
            if c == '%' && ch.get(i + 1).map_or(false, |x| *x == '0' || *x == '1') {
                let st = i + 1;
                i = st;
                while i < ch.len() && (ch[i] == '0' || ch[i] == '1') { i += 1; }
                let v = i64::from_str_radix(&ch[st..i].iter().collect::<String>(), 2).map_err(|e| e.to_string())?;
                out.push(Piece::B(v.to_string().into_bytes()));
                continue;
            }
            if c.is_alphabetic() {
                let st = i;
                while i < ch.len() && is_word(ch[i]) { i += 1; }
                let word: String = ch[st..i].iter().collect();
                let low = word.to_ascii_lowercase();
                let upper = low.to_ascii_uppercase();
                let next = ch.get(i).copied();
                if low == "rem" {
                    out.push(kw("REM"));
                    let rest: String = ch[i..].iter().collect();
                    let rest = rest.trim_start().to_ascii_lowercase(); // Kommentare: Kleinbuchstaben (keine umgeschalteten Zeichen, die als Tokens fehlgedeutet werden)
                    if !rest.is_empty() { let mut b = vec![b' ']; b.extend(petscii::from_text(&rest, d)?); out.push(Piece::B(b)); }
                    return Ok(out);
                }
                if low == "data" {
                    out.push(kw("DATA"));
                    let (mut q, mut j) = (false, i);
                    let mut raw = String::new();
                    while j < ch.len() {
                        if ch[j] == '"' { q = !q; }
                        if !q && ch[j] == ':' { break; }
                        raw.push(if q { ch[j] } else { ch[j].to_ascii_lowercase() });
                        j += 1;
                    }
                    out.push(Piece::B(petscii::from_text(raw.trim_start(), d)?));
                    i = j;
                    continue;
                }
                if (low == "print" || low == "input") && ch[i..].iter().find(|x| !x.is_whitespace()) == Some(&'#') {
                    while ch[i] != '#' { i += 1; }
                    i += 1;
                    out.push(kw(&format!("{upper}#")));
                    continue;
                }
                if (low == "tab" || low == "spc") && next == Some('(') { i += 1; out.push(kw(&format!("{upper}("))); continue; }
                if next == Some('$') && d.token_bytes(&format!("{upper}$")).is_some() { i += 1; out.push(kw(&format!("{upper}$"))); continue; }
                if let Some(v) = self.consts.get(&low) { out.push(Piece::B(v.clone())); continue; }
                if let Some(t) = d.token_bytes(&upper) { out.push(Piece::B(t)); continue; }
                if low.contains('.') { return Err(format!("ungültiger Name '{word}' (Punkt nur in Konstanten wie asm.label)")); }
                let suffix = match next { Some(x @ ('$' | '%')) => { i += 1; x.to_string() } _ => String::new() };
                out.push(Piece::Var(format!("{low}{suffix}")));
                continue;
            }
            // Operatoren und Satzzeichen
            let op = match c { '+' => Some("+"), '-' => Some("-"), '*' => Some("*"), '/' => Some("/"), '^' => Some("^"), '>' => Some(">"), '=' => Some("="), '<' => Some("<"), '?' => Some("PRINT"), _ => None };
            if let Some(o) = op { out.push(kw(o)); i += 1; continue; }
            out.push(Piece::B(vec![petscii::from_ascii(c).ok_or(format!("unerwartetes Zeichen '{c}'"))?]));
            i += 1;
        }
        Ok(out)
    }
}

pub(crate) fn first_word(t: &str) -> String { t.chars().take_while(|c| c.is_alphabetic()).collect::<String>().to_ascii_lowercase() }

fn valid_short(name: &str, d: &Dialect) -> bool {
    let c: Vec<char> = name.chars().collect();
    !c.is_empty() && c.len() <= 2 && c[0].is_ascii_alphabetic() && c.iter().all(char::is_ascii_alphanumeric)
        && !d.is_reserved_var(name) && d.token_bytes(&name.to_ascii_uppercase()).is_none()
        && !["IF", "ON", "GO", "TO", "OR", "FN"].contains(&name.to_ascii_uppercase().as_str())
}

fn map_vars(keys: &[String], d: &Dialect) -> Vec<(String, String)> {
    let split = |k: &str| -> (String, String) {
        match k.chars().last() { Some(c @ ('$' | '%')) => (k[..k.len() - 1].to_string(), c.to_string()), _ => (k.to_string(), String::new()) }
    };
    let mut used: HashSet<String> = HashSet::new();
    let mut map: HashMap<String, String> = HashMap::new();
    for k in keys {
        let (base, suf) = split(k);
        if valid_short(&base, d) { let short = format!("{}{}", base.to_ascii_uppercase(), suf); used.insert(short.clone()); map.insert(k.clone(), short); }
    }
    let pool: Vec<char> = ('A'..='Z').chain('0'..='9').collect();
    for k in keys {
        if map.contains_key(k) { continue; }
        let (base, suf) = split(k);
        let letters: Vec<char> = base.chars().filter(char::is_ascii_alphabetic).map(|c| c.to_ascii_uppercase()).collect();
        let first = letters.first().copied().unwrap_or('V');
        let mut cands: Vec<String> = vec![];
        for c in letters.iter().skip(1) { cands.push(format!("{first}{c}")); }
        for c in &pool { cands.push(format!("{first}{c}")); }
        for a in 'A'..='Z' { for c in &pool { cands.push(format!("{a}{c}")); } }
        let short = cands.into_iter().find(|c| valid_short(c, d) && !used.contains(&format!("{c}{suf}"))).unwrap_or_else(|| "ZZ".into());
        let short = format!("{short}{suf}");
        used.insert(short.clone());
        map.insert(k.clone(), short);
    }
    keys.iter().map(|k| (k.clone(), map[k].clone())).collect()
}

fn estimate_len(pieces: &[Piece], d: &Dialect) -> usize {
    pieces.iter().map(|p| match p {
        Piece::B(b) => { let (mut i, mut n) = (0, 0); while i < b.len() { match d.keyword_at(b, i) { Some((k, l)) => { n += k.len(); i += l; } None => { n += 1; i += 1; } } } n }
        Piece::Var(_) => 3,
        Piece::Label(_) => 5,
    }).sum()
}

pub fn compile(src: &str, d: &Dialect, opts: &Options) -> Result<Compiled, Vec<Diagnostic>> {
    let mut errs = vec![];
    let mut warnings = vec![];
    let mut lines_src = vec![];
    expand(src, &opts.file, &opts.include_dirs, 0, &mut lines_src, &mut errs, &Env::new(opts.system.as_deref(), &opts.consts));
    let mut consts: HashMap<String, Vec<u8>> = opts.consts.iter().map(|(n, v)| (n.to_ascii_lowercase(), v.to_string().into_bytes())).collect();
    let (mut plines, mut pending, mut blocks): (Vec<PLine>, Vec<String>, Vec<Block>) = (vec![], vec![], vec![]);
    let mut counter = 0;
    let (t_if, t_then, t_goto, t_rem) = (d.tok("IF"), d.tok("THEN"), d.tok("GOTO"), d.tok("REM"));
    // „wenn Bedingung falsch“: `(c)=0` statt `not(c)`, denn NOT ist bitweise (NOT 1 = -2 wäre wahr) und `if x then` mit x = 1 muss als wahr gelten
    let t_eq = d.token_bytes("=").unwrap();
    let (t_gosub, t_return) = (d.tok("GOSUB"), d.tok("RETURN"));
    // DO/LOOP/EXIT gibt es nativ im BASIC 3.5/7/65; sonst entstehen daraus Sprünge
    let native_do = d.token("DO").is_some() && d.token("LOOP").is_some() && d.token("EXIT").is_some();
    let t_exit = d.token("EXIT");
    // Prozeduren vorab sammeln, damit `call` vor `proc` stehen darf
    let mut used_ends: HashSet<String> = HashSet::new();
    let mut blobs: Vec<BlobReq> = vec![];
    let mut procs: HashMap<String, Vec<String>> = HashMap::new();
    for s in &lines_src {
        let t = s.text.trim();
        if first_word(t) == "proc" {
            match proc_head(&t[4..]) {
                Ok((n, ps)) => { procs.insert(n, ps); }
                Err(e) => errs.push(Diagnostic::error("B109", &s.file, s.line, e)),
            }
        }
    }
    macro_rules! err { ($s:expr, $code:expr, $msg:expr) => { errs.push(Diagnostic::error($code, &$s.file, $s.line, $msg)) } }
    for s in &lines_src {
        let mut t = s.text.trim();
        let at = (s.file.clone(), s.line);
        if t.is_empty() || t.starts_with('\'') { continue; }
        while t.starts_with('@') {
            let name: String = t[1..].chars().take_while(|c| is_word(*c)).collect();
            let rest = t[1 + name.len()..].trim_start();
            if name.is_empty() || !rest.starts_with(':') { break; }
            pending.push(name.to_ascii_lowercase());
            t = rest[1..].trim_start();
        }
        if t.is_empty() || t.starts_with('\'') { continue; }
        let word = first_word(t);
        let lexer = Lexer { d, consts: &consts };
        let cond_if = |pieces: &[Piece], target: String| -> Vec<Piece> {
            let mut v = vec![Piece::B(vec![t_if, b'('])];
            v.extend_from_slice(pieces);
            let mut close = vec![b')'];
            close.extend_from_slice(&t_eq);
            close.extend_from_slice(b"0");
            close.push(t_then);
            v.push(Piece::B(close));
            v.push(Piece::Label(target));
            v
        };
        let cond_goto = |pieces: &[Piece], target: String| -> Vec<Piece> {
            let mut v = vec![Piece::B(vec![t_if])];
            v.extend_from_slice(pieces);
            v.push(Piece::B(vec![t_then]));
            v.push(Piece::Label(target));
            v
        };
        let mut emit = |pieces: Vec<Piece>, pending: &mut Vec<String>| plines.push(PLine { src: at.clone(), labels: std::mem::take(pending), pieces });
        match word.as_str() {
            "const" => {
                let rest = t[5..].trim();
                let Some((n, v)) = rest.split_once('=') else { err!(s, "B108", "const erwartet NAME = Wert"); continue };
                let name = n.trim().to_ascii_lowercase();
                match lexer.lex(v.trim()) {
                    Ok(p) if p.len() == 1 && matches!(p[0], Piece::B(_)) => { if let Piece::B(b) = &p[0] { consts.insert(name, b.clone()); } }
                    Ok(_) => err!(s, "B108", "const erwartet eine einzelne Zahl oder Zeichenkette"),
                    Err(e) => err!(s, "B108", e),
                }
            }
            "else" if t.eq_ignore_ascii_case("else") => match blocks.last_mut() {
                Some(Block::If { els, end, has_else, .. }) if !*has_else => {
                    *has_else = true;
                    emit(vec![Piece::B(vec![t_goto]), Piece::Label(end.clone())], &mut pending);
                    pending.push(els.clone());
                }
                _ => err!(s, "B106", "else ohne passendes mehrzeiliges if"),
            },
            "endif" if t.eq_ignore_ascii_case("endif") => match blocks.pop() {
                Some(Block::If { els, end, has_else, .. }) => { if !has_else { pending.push(els); } pending.push(end); }
                _ => err!(s, "B106", "endif ohne if"),
            },
            "while" => match lexer.lex(t[5..].trim()) {
                Ok(c) => { counter += 1; let (top, end) = (format!("__w{counter}_top"), format!("__w{counter}_end")); pending.push(top.clone()); emit(cond_if(&c, end.clone()), &mut pending); blocks.push(Block::While { top, end, at: at.clone() }); }
                Err(e) => err!(s, "B100", e),
            },
            "wend" if t.eq_ignore_ascii_case("wend") => match blocks.pop() {
                Some(Block::While { top, end, .. }) => { emit(vec![Piece::B(vec![t_goto]), Piece::Label(top)], &mut pending); pending.push(end); }
                _ => err!(s, "B106", "wend ohne while"),
            },
            "repeat" if t.eq_ignore_ascii_case("repeat") => { counter += 1; let (top, end) = (format!("__r{counter}_top"), format!("__r{counter}_end")); pending.push(top.clone()); blocks.push(Block::Repeat { top, end, at: at.clone() }); }
            "until" => match (blocks.pop(), lexer.lex(t[5..].trim())) {
                (Some(Block::Repeat { top, end, .. }), Ok(c)) => { emit(cond_if(&c, top), &mut pending); if used_ends.contains(&end) { pending.push(end); } }
                (Some(_), _) | (None, _) => err!(s, "B106", "until ohne repeat"),
            },
            "blob" => {
                // blob "datei.asm" [with "a.asm", "b.asm"]
                let strs: Vec<&str> = t[4..].split('"').enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, x)| x).collect();
                let ok = t[4..].trim_start().starts_with('"') && !strs.is_empty() && (strs.len() == 1 || t[4..].to_ascii_lowercase().contains(" with "));
                if ok { blobs.push(BlobReq { file: strs[0].to_string(), with: strs[1..].iter().map(|x| x.to_string()).collect(), src: at.clone() }); }
                else { err!(s, "B113", "blob erwartet: blob \"datei.asm\" [with \"song.asm\"]"); }
            }
            "elseif" => match blocks.last_mut() {
                Some(Block::If { els, end, has_else, .. }) if !*has_else => match lexer.lex(t[6..].trim()) {
                    Ok(mut c) => {
                        if c.last() == Some(&Piece::B(vec![t_then])) { c.pop(); }
                        counter += 1;
                        let next_els = format!("__i{counter}_else");
                        emit(vec![Piece::B(vec![t_goto]), Piece::Label(end.clone())], &mut pending);
                        pending.push(std::mem::replace(els, next_els.clone()));
                        emit(cond_if(&c, next_els), &mut pending);
                    }
                    Err(e) => err!(s, "B100", e),
                },
                _ => err!(s, "B106", "elseif ohne passendes mehrzeiliges if (oder nach else)"),
            },
            "do" if native_do => match lexer.lex(t) {
                Ok(p) => { counter += 1; blocks.push(Block::Do { top: String::new(), end: String::new(), native: true, at: at.clone() }); emit(p, &mut pending); }
                Err(e) => err!(s, "B100", e),
            },
            "do" => {
                let rest = t[2..].trim();
                let (kind, cond) = (first_word(rest), rest[first_word(rest).len()..].trim());
                counter += 1;
                let (top, end) = (format!("__d{counter}_top"), format!("__d{counter}_end"));
                pending.push(top.clone());
                match (kind.as_str(), lexer.lex(cond)) {
                    ("", _) => {}
                    ("while", Ok(c)) => { used_ends.insert(end.clone()); emit(cond_if(&c, end.clone()), &mut pending) }
                    ("until", Ok(c)) => { used_ends.insert(end.clone()); emit(cond_goto(&c, end.clone()), &mut pending) }
                    (_, Err(e)) => err!(s, "B100", e),
                    _ => err!(s, "B106", "do erwartet nichts, 'while Bedingung' oder 'until Bedingung'"),
                }
                blocks.push(Block::Do { top, end, native: false, at: at.clone() });
            }
            "loop" => match blocks.pop() {
                Some(Block::Do { native: true, .. }) => match lexer.lex(t) { Ok(p) => emit(p, &mut pending), Err(e) => err!(s, "B100", e) },
                Some(Block::Do { top, end, .. }) => {
                    let rest = t[4..].trim();
                    let (kind, cond) = (first_word(rest), rest[first_word(rest).len()..].trim());
                    match (kind.as_str(), lexer.lex(cond)) {
                        ("", _) => emit(vec![Piece::B(vec![t_goto]), Piece::Label(top)], &mut pending),
                        ("while", Ok(c)) => emit(cond_goto(&c, top), &mut pending),
                        ("until", Ok(c)) => emit(cond_if(&c, top), &mut pending),
                        (_, Err(e)) => err!(s, "B100", e),
                        _ => err!(s, "B106", "loop erwartet nichts, 'while Bedingung' oder 'until Bedingung'"),
                    }
                    if used_ends.contains(&end) { pending.push(end); }
                }
                other => { if let Some(b) = other { blocks.push(b); } err!(s, "B106", "loop ohne do"); }
            },
            "exit" if t.eq_ignore_ascii_case("exit") => {
                let target = blocks.iter().rev().take_while(|b| !matches!(b, Block::Proc { .. }))
                    .find_map(|b| match b { Block::While { end, .. } | Block::Repeat { end, .. } => Some(Ok(end.clone())), Block::Do { native: true, .. } => Some(Err(())), Block::Do { end, .. } => Some(Ok(end.clone())), _ => None });
                match (target, t_exit) {
                    (Some(Ok(end)), _) => { used_ends.insert(end.clone()); emit(vec![Piece::B(vec![t_goto]), Piece::Label(end)], &mut pending) }
                    (Some(Err(())), Some(x)) => emit(vec![Piece::B(vec![x])], &mut pending),
                    _ => err!(s, "B106", "exit außerhalb von while, repeat oder do"),
                }
            }
            "proc" => match proc_head(&t[4..]) {
                Ok((name, _)) => {
                    if blocks.iter().any(|b| matches!(b, Block::Proc { .. })) { err!(s, "B109", "proc darf nicht in einer anderen proc stehen"); continue; }
                    counter += 1;
                    let skip = format!("__p{counter}_skip");
                    emit(vec![Piece::B(vec![t_goto]), Piece::Label(skip.clone())], &mut pending);
                    pending.push(name);
                    blocks.push(Block::Proc { skip, at: at.clone() });
                }
                Err(e) => err!(s, "B109", e),
            },
            "endproc" if t.eq_ignore_ascii_case("endproc") => match blocks.pop() {
                Some(Block::Proc { skip, .. }) => { emit(vec![Piece::B(vec![t_return])], &mut pending); pending.push(skip); }
                other => { if let Some(b) = other { blocks.push(b); } err!(s, "B106", "endproc ohne proc"); }
            },
            "call" => match t[4..].trim().chars().take_while(|c| is_word(*c)).collect::<String>().to_ascii_lowercase() {
                name if name.is_empty() => err!(s, "B109", "Prozedurname fehlt"),
                name if !procs.contains_key(&name) => err!(s, "B109", format!("Prozedur '{name}' ist nicht definiert")),
                name => {
                    let rest = t[4..].trim();
                    let args = match rest[name.len()..].trim() { "" => vec![], a => split_args(a.strip_prefix('(').and_then(|x| x.strip_suffix(')')).unwrap_or(a)) };
                    let params = &procs[&name];
                    if args.len() != params.len() { err!(s, "B109", format!("Prozedur '{name}' erwartet {} Argument(e), übergeben: {}", params.len(), args.len())); continue; }
                    let mut pieces = vec![];
                    let mut bad = false;
                    for (p, a) in params.iter().zip(&args) {
                        match (lexer.lex(p), lexer.lex(a)) {
                            (Ok(mut pv), Ok(av)) => { pieces.append(&mut pv); pieces.push(Piece::B(d.token_bytes("=").unwrap())); pieces.extend(av); pieces.push(Piece::B(vec![b':'])); }
                            (Err(e), _) | (_, Err(e)) => { err!(s, "B100", e); bad = true; break; }
                        }
                    }
                    if bad { continue; }
                    pieces.push(Piece::B(vec![t_gosub]));
                    pieces.push(Piece::Label(name));
                    emit(pieces, &mut pending);
                }
            },
            _ => match lexer.lex(t) {
                Err(e) => err!(s, "B100", e),
                Ok(mut pieces) => {
                    if pieces.first() == Some(&Piece::B(vec![t_if])) && pieces.last() == Some(&Piece::B(vec![t_then])) && pieces.len() > 2 {
                        counter += 1;
                        let (els, end) = (format!("__i{counter}_else"), format!("__i{counter}_end"));
                        let cond: Vec<Piece> = pieces[1..pieces.len() - 1].to_vec();
                        pieces = cond_if(&cond, els.clone());
                        blocks.push(Block::If { els, end, has_else: false, at: at.clone() });
                    }
                    if !d.restore_line {
                        let rs = d.token_bytes("RESTORE").unwrap_or_default();
                        if pieces.windows(2).any(|w| matches!(&w[0], Piece::B(b) if *b == rs) && matches!(w[1], Piece::Label(_))) {
                            err!(s, "B112", "restore mit Marke gibt es im BASIC V2 nicht (erst ab BASIC 3.5): ohne Marke zurücksetzen und der Reihe nach lesen");
                        }
                    }
                    for p in &pieces { if let Piece::Var(n) = p { if RESERVED.contains(&n.as_str()) { err!(s, "B103", format!("'{n}' ist ein reserviertes Wort der Struktursprache")); } } }
                    emit(pieces, &mut pending);
                }
            },
        }
    }
    for b in &blocks {
        let at = match b { Block::If { at, .. } | Block::While { at, .. } | Block::Repeat { at, .. } | Block::Do { at, .. } | Block::Proc { at, .. } => at };
        errs.push(Diagnostic::error("B106", &at.0, at.1, "Block nicht geschlossen"));
    }
    if !opts.blob_chunks.is_empty() {
        let at = plines.last().map(|l| l.src.clone()).unwrap_or((opts.file.clone(), 1));
        pending.push("blob_data".into());
        let data = d.tok("DATA");
        let mut put = |text: String, pending: &mut Vec<String>| plines.push(PLine { src: at.clone(), labels: std::mem::take(pending), pieces: vec![Piece::B(vec![data]), Piece::B(text.into_bytes())] });
        for (addr, bytes) in &opts.blob_chunks {
            put(format!("{addr},{}", bytes.len()), &mut pending);
            for row in bytes.chunks(16) { put(row.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","), &mut pending); }
        }
        put("0".into(), &mut pending);
    }
    if !pending.is_empty() {
        let at = plines.last().map(|l| l.src.clone()).unwrap_or((opts.file.clone(), 1));
        plines.push(PLine { src: at, labels: std::mem::take(&mut pending), pieces: vec![Piece::B(vec![t_rem])] });
    }
    // Zeilen zusammenfassen
    if opts.pack {
        let mut packed: Vec<PLine> = vec![];
        for l in plines {
            let can = l.labels.is_empty() && packed.last().map_or(false, |p| {
                let has = |tok: u8| p.pieces.iter().any(|x| matches!(x, Piece::B(b) if b.contains(&tok)));
                !has(t_if) && !has(t_rem) && !has(d.tok("DATA"))
                    && estimate_len(&p.pieces, d) + 1 + estimate_len(&l.pieces, d) <= d.max_line_len
            });
            if can { let p = packed.last_mut().unwrap(); p.pieces.push(Piece::B(vec![b':'])); p.pieces.extend(l.pieces); } else { packed.push(l); }
        }
        plines = packed;
    }
    // Nummern vergeben und Labels auflösen
    let mut labels: BTreeMap<String, u16> = BTreeMap::new();
    for (i, l) in plines.iter().enumerate() {
        let num = opts.start + i as u32 * opts.step;
        if num > 63999 { errs.push(Diagnostic::error("B105", &l.src.0, l.src.1, "Zeilennummern über 63999 – step verkleinern oder pack nutzen")); break; }
        for name in &l.labels {
            if labels.insert(name.clone(), num as u16).is_some() { errs.push(Diagnostic::error("B104", &l.src.0, l.src.1, format!("Label '{name}' doppelt definiert"))); }
        }
    }
    let mut keys: Vec<String> = vec![];
    for l in &plines { for p in &l.pieces { if let Piece::Var(k) = p { if !keys.contains(k) { keys.push(k.clone()); } } } }
    let vars = map_vars(&keys, d);
    let vmap: HashMap<&str, &str> = vars.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let mut out_lines = vec![];
    for (i, l) in plines.iter().enumerate() {
        let mut bytes = vec![];
        for p in &l.pieces {
            match p {
                Piece::B(b) => bytes.extend_from_slice(b),
                Piece::Var(k) => bytes.extend_from_slice(vmap[k.as_str()].as_bytes()),
                Piece::Label(n) => match labels.get(n) {
                    Some(v) => bytes.extend_from_slice(v.to_string().as_bytes()),
                    None if n.starts_with("__") => {}
                    None => errs.push(Diagnostic::error("B107", &l.src.0, l.src.1, format!("Label '@{n}' nicht definiert"))),
                },
            }
        }
        if bytes.len() > 250 { errs.push(Diagnostic::error("B102", &l.src.0, l.src.1, "Zeile zu lang (mehr als 250 Bytes) – Zeile teilen")); }
        else if crate::detok::line_text(&bytes, d).chars().count() > d.max_line_len {
            warnings.push(Diagnostic::warning("B101", &l.src.0, l.src.1, format!("Zeile länger als {} Zeichen: im Editor nicht mehr bearbeitbar", d.max_line_len)));
        }
        out_lines.push(((opts.start + i as u32 * opts.step) as u16, bytes));
    }
    if !errs.is_empty() { return Err(errs); }
    let load = opts.load_addr.unwrap_or(d.load_addr);
    Ok(Compiled { blobs, prg: build_prg(&out_lines, load), lines: out_lines, labels, vars, warnings })
}
