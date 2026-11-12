// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder-Assembler (Phase 1): Mehrpass-Assembler für die 6502-Familie.
mod cart;
mod disasm;
mod expr;
mod parse;
pub mod tokens;
mod tree;

pub use cart::{to_car, to_crt, to_rom, to_vic20_cart};
pub use disasm::{disassemble, DisLine};
use expr::{eval, lookup_scoped, qualified, Ctx, Func, Val};
use parse::{parse_operand, split_top, Body, Operand, Stmt};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::rc::Rc;
use tree::{build, load, quoted, resolve, substitute, Cond, ForSpec, Macros, Node, StructNode};
use vc_core::Diagnostic;
use vc_target::{Cpu, Mode};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy { Deny, Warn, Allow }

impl Policy {
    pub fn parse(s: &str) -> Option<Policy> {
        match s { "deny" => Some(Policy::Deny), "warn" => Some(Policy::Warn), "allow" => Some(Policy::Allow), _ => None }
    }
}

#[derive(Clone)]
pub struct Options {
    pub illegals: Policy,
    pub file: String,
    pub include_dirs: Vec<PathBuf>,
    pub defines: Vec<(String, i64)>,
    /// Vordefinierte Symbole (z. B. Chip-Register des Targets): nutzbar, aber nicht im Symbol-Export.
    pub predefined: Vec<(String, i64)>,
    /// `.screen` und `.encoding "screen"` wandeln in den Atari-Bildschirmcode (ANTIC) statt in den C64-Bildschirmcode.
    pub atari_text: bool,
    /// zu weite bedingte Sprünge automatisch verlängern (Voreinstellung; im Quelltext mit `.branchlong auto|off`)
    pub branch_long: bool,
    /// BASIC-Startadresse des Zielsystems (aus den Zieldaten, `basic.loadAddr`, ggf. mit Speicherausbau); `.basicstub` ohne `.org` beginnt dort. `None`: $0801
    pub basic_start: Option<u16>,
}

impl Options {
    pub fn new(file: impl Into<String>, illegals: Policy) -> Self {
        Options { illegals, file: file.into(), include_dirs: vec![], defines: vec![], predefined: vec![], atari_text: false, branch_long: false, basic_start: None }
    }
}

#[derive(Debug, Clone)]
pub struct Segment { pub start: u16, pub bytes: Vec<u8>, pub bank: u32, pub name: String }

#[derive(Debug, Clone)]
pub struct LineInfo { pub file: String, pub line: usize, pub addr: u16, pub bytes: Vec<u8>, pub cycles: Option<(u8, u8)> }

#[derive(Debug, Clone)]
pub struct CycleReport { pub name: String, pub file: String, pub line: usize, /// Zeile von `.endcycles` (0 = unbekannt)
    pub end_line: usize, pub min: u32, pub max: u32, pub budget: Option<u32> }

#[derive(Debug, Clone)]
pub struct Output {
    pub defs: BTreeMap<String, (String, usize)>,
    pub run: Option<u16>,
    pub cycles: Vec<CycleReport>,
    pub segments: Vec<Segment>,
    /// `.cartridge "typ", "NAME"`: Modultyp und Name für die Modulformate
    pub cart: Option<(String, String)>,
    pub symbols: BTreeMap<String, i64>,
    /// Makros (auch aus Includes): Name und Parameternamen
    pub macros: Vec<(String, Vec<String>)>,
    pub lines: Vec<LineInfo>,
    pub warnings: Vec<Diagnostic>,
    pub sources: Vec<(String, Vec<String>)>,
    /// Anzahl der Durchläufe (Symbolauflösung + abschließender Durchlauf)
    pub passes: usize,
    /// Quelltext nach dem Einsetzen der Includes: (Datei, Zeile, Text)
    pub expanded: Vec<(String, usize, String)>,
}

const ALIASES: &[(&str, &str)] = &[
    ("ASO", "SLO"), ("LSE", "SRE"), ("ISB", "ISC"), ("INS", "ISC"), ("DCM", "DCP"), ("SBX", "AXS"), ("ASR", "ALR"),
    ("ANE", "XAA"), ("AXA", "SHA"), ("SXA", "SHX"), ("SYA", "SHY"), ("XAS", "TAS"), ("KIL", "JAM"), ("HLT", "JAM"),
    ("BLT", "BCC"), ("BGE", "BCS"),
];

fn petscii(c: u8) -> u8 {
    match c { b'a'..=b'z' => c - 0x20, b'A'..=b'Z' => c + 0x80, _ => c }
}
fn valid_name(n: &str) -> bool { !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_') && !n.starts_with(|c: char| c.is_ascii_digit()) }
fn atari_screen(c: u8) -> u8 { match c { 0x20..=0x5F => c - 0x20, 0x00..=0x1F => c + 0x40, _ => c } }
fn atascii(c: u8) -> u8 { if c == b'\n' { 0x9B } else { c } }
fn screencode(c: u8) -> u8 {
    match petscii(c) { p @ 0x40..=0x5F => p - 0x40, p @ 0x60..=0x7F => p - 0x20, p @ 0xC0..=0xDF => p - 0x80, p => p }
}

/// Definierte Struktur: Größe und alle Feldnamen (auch verschachtelt, z. B. `pos.x`) mit Offset
#[derive(Clone)]
struct StructDef { size: i64, members: Vec<(String, i64)>, is_enum: bool }

#[derive(Clone, Copy, PartialEq)]
enum LoopCtl { Break, Continue }

/// Zeichenkodierung für `.text`, `.null`, `.ptext`, `.shift`
#[derive(Clone, PartialEq)]
enum Enc { Ascii, Petscii, Screen, Atascii, Internal, Table(Rc<Vec<u8>>) }
impl Enc {
    fn map(&self, c: u8) -> u8 {
        match self {
            Enc::Ascii => c,
            Enc::Petscii => petscii(c),
            Enc::Screen => screencode(c),
            Enc::Atascii => atascii(c),
            Enc::Internal => atari_screen(c),
            Enc::Table(t) => t.get(c as usize).copied().unwrap_or(c),
        }
    }
}

struct CycleBlock { name: String, file: String, line: usize, min: u32, max: u32, budget: Option<u32> }

struct Pass<'a> {
    real: Option<i64>,
    run: Option<i64>,
    open_cycles: Vec<CycleBlock>,
    cycle_reports: Vec<CycleReport>,
    cpu: &'a Cpu,
    opts: &'a Options,
    macros: &'a Macros,
    strict: bool,
    syms: &'a HashMap<String, i64>,
    new_syms: HashMap<String, i64>,
    defs: HashMap<String, (String, usize)>,
    locals: HashMap<String, i64>,
    segments: Vec<Segment>,
    bank: u32,
    bank_pos: HashMap<u32, (Option<i64>, Option<i64>)>,
    seg_name: String,
    seg_range: Option<(i64, i64, String)>,
    cart: Option<(String, String)>,
    lines: Vec<LineInfo>,
    diags: Vec<Diagnostic>,
    pc: Option<i64>,
    scope: String,
    /// geöffnete `{ }`-Scopes (Namen) und die dazu gemerkten Zustände der lokalen Labels
    path: Vec<String>, path_str: String, scope_stack: Vec<String>, anon: usize, enc: Enc, blocks: Vec<Option<Enc>>,
    policy: Policy,
    line: usize,
    file: Rc<str>,
    trace: Vec<u8>,
    last_cycles: Option<(u8, u8)>,
    macro_counter: usize,
    structs: HashMap<String, StructDef>,
    branch_auto: bool,
    /// `.branchlong jmp`: lange Sprünge auch auf der 45GS02 als umgekehrter Sprung plus JMP (6502-kompatibel)
    branch_jmp: bool,
    branch_idx: usize,
    long: std::collections::HashSet<usize>,
    vars: HashMap<String, Val>,
    funcs: HashMap<String, Func>,
    loop_ctl: Option<LoopCtl>,
    loop_depth: usize,
    exit_macro: bool,
    depth: usize,
    /// bisher in diesem Durchlauf definierte anonyme Labels `-` bzw. `+` (Schlüssel `-#n` / `+#n` in den Symbolen)
    anon_minus: usize,
    anon_plus: usize,
}

impl<'a> Pass<'a> {
    fn err(&mut self, code: &'static str, msg: impl Into<String>) {
        if self.strict { self.diags.push(Diagnostic::error(code, &self.file, self.line, msg)); }
    }
    fn warn(&mut self, code: &'static str, msg: impl Into<String>) {
        if self.strict { self.diags.push(Diagnostic::warning(code, &self.file, self.line, msg)); }
    }
    /// Anonymes Label als Operand: `-` / `--` … = das (n-te) vorige `-`-Label, `+` / `++` … = das (n-te) nächste `+`-Label.
    fn anon_ref(&mut self, t: &str) -> Val {
        let level = t.chars().count();
        if t.starts_with('-') {
            if level > self.anon_minus { self.err("E214", format!("kein passendes '{t}'-Label davor")); return Val::int(0, false); }
            let v = self.new_syms.get(&format!("-#{}", self.anon_minus - level)).copied();
            Val::int(v.unwrap_or(0), v.is_some())
        } else {
            let v = self.syms.get(&format!("+#{}", self.anon_plus + level - 1)).copied();
            if v.is_none() && self.strict { self.err("E214", format!("kein passendes '{t}'-Label dahinter")); }
            Val::int(v.unwrap_or(0), v.is_some())
        }
    }
    /// Wie `ev`, aber ohne Warnung bei Nachkommastellen (für Variablen, die Kommazahlen halten dürfen)
    fn ev_raw(&mut self, s: &str) -> Val {
        let t = s.trim();
        if !t.is_empty() && (t.chars().all(|c| c == '+') || t.chars().all(|c| c == '-')) { return self.anon_ref(t); }
        let ctx = Ctx { syms: self.syms, locals: &self.locals, vars: &self.vars, funcs: &self.funcs, stack: &[], pc: self.pc.unwrap_or(0), scope: &self.scope, path: &self.path_str, strict: self.strict };
        match eval(s, &ctx) {
            Ok(v) => v,
            Err(e) => { self.err("E202", format!("{e} in '{s}'")); Val::int(0, false) }
        }
    }
    /// Wert als ganze Zahl; Nachkommastellen werden abgeschnitten und gemeldet (W202)
    fn ev(&mut self, s: &str) -> Val {
        let v = self.ev_raw(s);
        if v.list.is_some() {
            self.err("E202", format!("Liste statt Zahl in '{}' (Elemente mit [i] wählen oder .byte/.word für die ganze Liste)", s.trim()));
            return Val::int(0, false);
        }
        if v.known && v.has_fraction() {
            self.warn("W202", format!("Kommazahl {} auf {} abgeschnitten (int(), round() oder floor() verwenden) in '{}'", v.f.unwrap(), v.v, s.trim()));
        }
        Val::int(v.v, v.known)
    }
    /// Wert(e) eines Ausdrucks für Datendirektiven: eine Liste wird in ihre Zahlen aufgelöst (verschachtelte Listen flach)
    fn ev_items(&mut self, s: &str) -> Vec<Val> {
        let v = self.ev_raw(s);
        v.flatten().into_iter().map(|x| {
            if x.known && x.has_fraction() {
                self.warn("W202", format!("Kommazahl {} auf {} abgeschnitten (int(), round() oder floor() verwenden) in '{}'", x.f.unwrap(), x.v, s.trim()));
            }
            Val::int(x.v, x.known)
        }).collect()
    }
    /// Wertet `expr` für `var` = 0..n aus (die Laufvariable ist nur dabei sichtbar)
    fn indexed_values(&mut self, n: i64, expr: &str, var: &str) -> Vec<Val> {
        let saved = self.locals.get(var).copied();
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            self.locals.insert(var.to_string(), i);
            out.push(self.ev(expr));
        }
        match saved { Some(v) => { self.locals.insert(var.to_string(), v); } None => { self.locals.remove(var); } }
        out
    }
    /// Schlüssel der sichtbaren Variablen `name` (innerster Scope zuerst)
    fn var_key(&self, name: &str) -> Option<String> {
        let mut f = |k: &str| self.vars.contains_key(k).then(|| k.to_string());
        lookup_scoped(name, &self.scope, &self.path_str, &mut f)
    }
    fn declare_var(&mut self, dir: &str, args: &str) {
        let Some((n, e)) = args.split_once('=').filter(|(_, e)| !e.starts_with('=')) else { self.err("E208", format!("{dir} erwartet NAME = Ausdruck")); return };
        let name = n.trim();
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.starts_with(|c: char| c.is_ascii_digit()) {
            self.err("E208", format!("{dir}: ungültiger Variablenname '{name}'")); return;
        }
        let v = self.ev_raw(e.trim());
        match (dir, self.var_key(name)) {
            (".set", None) => self.err("E216", format!("Variable '{name}' ist nicht mit .var angelegt")),
            (".set", Some(k)) => { self.vars.insert(k, v); }
            _ => { let k = qualified(name, &self.scope, &self.path_str); self.vars.insert(k, v); }
        }
    }
    /// `.ifdef`/`.ifndef`: nur Symbole, die in diesem Durchlauf schon definiert sind (sonst schwankt die Abfrage zwischen den Durchläufen)
    fn is_defined(&self, name: &str) -> bool {
        let mut f = |k: &str| (self.new_syms.contains_key(k) || self.locals.contains_key(k)).then_some(());
        lookup_scoped(name, &self.scope, &self.path_str, &mut f).is_some()
    }
    fn define(&mut self, name: &str, v: i64) {
        let key = qualified(name, &self.scope, &self.path_str);
        if self.strict && self.new_syms.get(&key).map_or(false, |old| *old != v) { self.err("E204", format!("Symbol '{name}' doppelt definiert")); }
        self.defs.insert(key.clone(), (self.file.to_string(), self.line));
        self.new_syms.insert(key, v);
    }
    /// Öffnet einen Scope (`{`); `name` = Label davor, sonst anonym. Innerhalb gilt `name.symbol`-Zugriff von außen.
    fn open_scope(&mut self, name: Option<&str>) {
        let n = match name { Some(n) if !n.starts_with('.') && !n.contains('.') => n.to_string(), Some(n) => { self.err("E213", format!("Scope-Name '{n}' darf keinen Punkt enthalten")); self.anon += 1; format!("_s{}", self.anon) }
                             None => { self.anon += 1; format!("_s{}", self.anon) } };
        self.blocks.push(None);
        self.scope_stack.push(std::mem::take(&mut self.scope));
        self.path.push(n);
        self.path_str = self.path.join(".");
        self.scope = self.path_str.clone();
    }
    fn close_scope(&mut self) {
        if let Some(Some(prev)) = self.blocks.last().cloned() { self.blocks.pop(); self.enc = prev; return; }
        self.blocks.pop();
        if self.path.pop().is_none() { self.err("E213", "'}' ohne öffnendes '{'"); return; }
        self.path_str = self.path.join(".");
        self.scope = self.scope_stack.pop().unwrap_or_default();
    }
    fn emit(&mut self, bytes: &[u8]) {
        let Some(pc) = self.pc else { self.err("E300", "Code vor .org"); return };
        let real = self.real.unwrap_or(pc);
        if let Some((lo, hi, name)) = self.seg_range.clone() {
            if !bytes.is_empty() && (real < lo || real + bytes.len() as i64 - 1 > hi) { self.err("E301", format!("Segment '{name}' verlässt seinen Bereich ${lo:04X}-${hi:04X} (${real:04X}..${:04X})", real + bytes.len() as i64 - 1)); }
        }
        if self.segments.is_empty() { self.segments.push(Segment { start: real as u16, bytes: vec![], bank: self.bank, name: self.seg_name.clone() }); }
        self.segments.last_mut().unwrap().bytes.extend_from_slice(bytes);
        self.trace.extend_from_slice(bytes);
        self.pc = Some(pc + bytes.len() as i64);
        self.real = Some(real + bytes.len() as i64);
    }
    /// Beginnt ein neues Segment (Bank und Name der aktuellen Einstellung); ein noch leeres letztes Segment wird wiederverwendet.
    fn begin_segment(&mut self, start: u16) {
        let (bank, name) = (self.bank, self.seg_name.clone());
        match self.segments.last_mut() {
            Some(s) if s.bytes.is_empty() => { s.start = start; s.bank = bank; s.name = name; }
            _ => self.segments.push(Segment { start, bytes: vec![], bank, name }),
        }
    }
    fn byte(&mut self, v: Val, lo: i64, hi: i64) -> u8 {
        if self.strict && v.known && (v.v < lo || v.v > hi) { self.err("E203", format!("Wert {} außerhalb {}..{}", v.v, lo, hi)); }
        (v.v & 0xFF) as u8
    }

    /// Zeichenkette mit `{name}`-Sonderzeichen in Bytes; `conv` wandelt die gewöhnlichen Zeichen
    fn text_bytes(&mut self, s: &str, names: tokens::Names, conv: &dyn Fn(u8) -> u8) -> Option<Vec<u8>> {
        match tokens::expand(s, names) {
            Ok(p) => Some(p.into_iter().map(|x| match x { tokens::Piece::Char(c) => conv(c), tokens::Piece::Raw(b) => b }).collect()),
            Err(e) => { self.err("E208", e); None }
        }
    }
    fn string_bytes(&mut self, args: &str, conv: fn(u8) -> u8, names: tokens::Names) {
        match quoted(args) {
            Some(s) => { if let Some(b) = self.text_bytes(s, names, &conv) { self.emit(&b); } }
            None => self.err("E208", "Zeichenkette in Anführungszeichen erwartet"),
        }
    }
    /// Zeichenketten (kommagetrennt) in der aktuellen Kodierung; `f` formt die Bytes einer Kette um
    fn enc_strings(&mut self, dir: &str, args: &str, f: fn(Vec<u8>) -> Result<Vec<u8>, &'static str>) {
        for part in split_top(args, ',') {
            let Some(s) = quoted(&part) else { self.err("E208", format!("{dir} erwartet Zeichenketten in Anführungszeichen")); return };
            let names = match self.enc { Enc::Ascii => tokens::Names::Off, Enc::Petscii => tokens::Names::Petscii, Enc::Atascii => tokens::Names::Atascii, _ => tokens::Names::Numeric };
            let enc = self.enc.clone();
            let Some(b) = self.text_bytes(s, names, &|c| enc.map(c)) else { return };
            match f(b) { Ok(b) => self.emit(&b), Err(m) => { self.err("E203", format!("{dir}: {m}")); } }
        }
    }
    fn set_encoding(&mut self, args: &str) {
        let a = args.trim();
        let block = a.ends_with('{');
        let a = a.trim_end_matches('{').trim();
        let Some(name) = quoted(a) else { self.err("E208", ".encoding erwartet einen Namen in Anführungszeichen"); return };
        let enc = match name.to_ascii_lowercase().as_str() {
            "ascii" | "text" => Enc::Ascii,
            "petscii" => Enc::Petscii,
            "screen" | "screencode" => if self.opts.atari_text { Enc::Internal } else { Enc::Screen },
            "atascii" => Enc::Atascii,
            _ => {
                let Some(path) = resolve(&self.file, name, &self.opts.include_dirs) else { self.err("E208", format!(".encoding: unbekannte Kodierung oder Datei '{name}'")); return };
                match std::fs::read(&path) {
                    Ok(d) if d.len() == 256 => Enc::Table(Rc::new(d)),
                    Ok(_) => { self.err("E208", format!(".encoding: '{name}' muss genau 256 Byte enthalten")); return }
                    Err(e) => { self.err("E212", format!("{}: {e}", path.display())); return }
                }
            }
        };
        if block { self.blocks.push(Some(std::mem::replace(&mut self.enc, enc))); } else { self.enc = enc; }
    }

    fn directive(&mut self, name: &str, args: &str) {
        match name {
            ".org" => {
                let v = self.ev(args);
                if self.strict && !(0..=0xFFFF).contains(&v.v) { self.err("E203", ".org außerhalb 0..65535"); }
                self.pc = Some(v.v);
                self.real = Some(v.v);
                self.begin_segment(v.v as u16);
            }
            ".bank" => {
                let n = self.ev(args);
                if self.strict && !(0..=1023).contains(&n.v) { self.err("E203", ".bank außerhalb 0..1023"); }
                self.bank_pos.insert(self.bank, (self.pc, self.real));
                self.bank = n.v.clamp(0, 1023) as u32;
                let (pc, real) = self.bank_pos.get(&self.bank).copied().unwrap_or((None, None));
                self.pc = pc; self.real = real;
                self.begin_segment(real.unwrap_or(0) as u16);
            }
            ".segment" => {
                let p = split_top(args, ',');
                match p.first().and_then(|s| quoted(s)) {
                    Some(name) => { self.seg_name = name.to_string(); }
                    None if args.trim().is_empty() => { self.seg_name.clear(); }
                    None => { self.err("E208", ".segment erwartet einen Namen in Anführungszeichen"); return; }
                }
                self.seg_range = if p.len() >= 3 { let (lo, hi) = (self.ev(&p[1]).v, self.ev(&p[2]).v); if self.strict && lo > hi { self.err("E203", ".segment: Anfang liegt hinter dem Ende"); } Some((lo, hi, self.seg_name.clone())) } else { None };
                self.begin_segment(self.real.unwrap_or(0) as u16);
            }
            ".cartridge" => {
                let p = split_top(args, ',');
                let kind = p.first().and_then(|s| quoted(s)).map(str::to_string);
                let Some(kind) = kind else { self.err("E208", ".cartridge erwartet den Typ in Anführungszeichen (z. B. \"ocean\")"); return };
                self.cart = Some((kind, p.get(1).and_then(|s| quoted(s)).unwrap_or("").to_string()));
            }
            ".byte" | ".db" => {
                for part in split_top(args, ',') {
                    if let Some(s) = quoted(&part) { let b: Vec<u8> = s.bytes().collect(); self.emit(&b); }
                    else { for v in self.ev_items(&part) { let b = self.byte(v, -128, 255); self.emit(&[b]); } }
                }
            }
            ".word" | ".dw" => {
                for part in split_top(args, ',') { for v in self.ev_items(&part) {
                    if self.strict && v.known && !(-32768..=65535).contains(&v.v) { self.err("E203", format!("Wert {} außerhalb 16 Bit", v.v)); }
                    self.emit(&[(v.v & 0xFF) as u8, ((v.v >> 8) & 0xFF) as u8]);
                } }
            }
            ".var" | ".set" => self.declare_var(name, args),
            ".function" => self.declare_function(args),
            ".break" | ".continue" => {
                if self.loop_depth == 0 { self.err("E210", format!("{name} außerhalb einer Schleife")); }
                else { self.loop_ctl = Some(if name == ".break" { LoopCtl::Break } else { LoopCtl::Continue }); }
            }
            ".encoding" => self.set_encoding(args),
            ".text" => self.enc_strings(".text", args, Ok),
            ".null" => self.enc_strings(".null", args, |mut b| { b.push(0); Ok(b) }),
            ".ptext" => self.enc_strings(".ptext", args, |b| {
                if b.len() > 255 { return Err("Text länger als 255 Zeichen"); }
                let mut o = vec![b.len() as u8]; o.extend(b); Ok(o)
            }),
            ".shift" => self.enc_strings(".shift", args, |mut b| {
                match b.last_mut() { Some(l) => { *l |= 0x80; Ok(b) } None => Err("leerer Text") }
            }),
            ".rta" => {
                for part in split_top(args, ',') { for v in self.ev_items(&part) {
                    let w = v.v - 1;
                    if self.strict && v.known && !(0..=65536).contains(&v.v) { self.err("E203", format!("Wert {} außerhalb 16 Bit", v.v)); }
                    self.emit(&[(w & 0xFF) as u8, ((w >> 8) & 0xFF) as u8]);
                } }
            }
            ".be16" | ".dbyt" => {
                for part in split_top(args, ',') { for v in self.ev_items(&part) {
                    if self.strict && v.known && !(-32768..=65535).contains(&v.v) { self.err("E203", format!("Wert {} außerhalb 16 Bit", v.v)); }
                    self.emit(&[((v.v >> 8) & 0xFF) as u8, (v.v & 0xFF) as u8]);
                } }
            }
            ".dword" | ".long" => {
                for part in split_top(args, ',') { for v in self.ev_items(&part) {
                    if self.strict && v.known && !(-2147483648..=4294967295).contains(&v.v) { self.err("E203", format!("Wert {} außerhalb 32 Bit", v.v)); }
                    self.emit(&(v.v as u32).to_le_bytes());
                } }
            }
            ".hex" => {
                for part in split_top(args, ',') {
                    let Some(s) = quoted(&part) else { self.err("E208", ".hex erwartet Hexziffern in Anführungszeichen"); return };
                    let d: Vec<u8> = s.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
                    if d.len() % 2 != 0 || !d.iter().all(u8::is_ascii_hexdigit) { self.err("E208", ".hex: gerade Anzahl Hexziffern erwartet"); return; }
                    let b: Vec<u8> = d.chunks(2).map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap()).collect();
                    self.emit(&b);
                }
            }
            ".petscii" => self.string_bytes(args, petscii, tokens::Names::Petscii),
            ".screen" => self.string_bytes(args, if self.opts.atari_text { atari_screen } else { screencode }, tokens::Names::Numeric),
            ".atascii" => self.string_bytes(args, atascii, tokens::Names::Atascii),
            ".fill" => {
                // .fill anzahl [, ausdruck [, laufvariable]]: der Ausdruck darf die Laufvariable (Standard `i`, 0-basiert) benutzen
                let p = split_top(args, ',');
                let n = self.ev(p.first().map(String::as_str).unwrap_or("0"));
                if n.v < 0 || n.v > 65536 { self.err("E203", ".fill: ungültige Länge"); return; }
                let var = p.get(2).map(String::as_str).unwrap_or("i");
                let vals = self.indexed_values(n.v, p.get(1).map(String::as_str).unwrap_or("0"), var);
                let bytes: Vec<u8> = vals.into_iter().map(|v| self.byte(v, -128, 255)).collect();
                self.emit(&bytes);
            }
            ".lohifill" => {
                // .lohifill anzahl, ausdruck [, laufvariable]: erst alle Low-Bytes, dann alle High-Bytes (Adresstabellen)
                let p = split_top(args, ',');
                if p.len() < 2 { self.err("E208", ".lohifill erwartet: .lohifill anzahl, ausdruck(i)"); return; }
                let n = self.ev(&p[0]);
                if n.v < 0 || n.v > 32768 { self.err("E203", ".lohifill: ungültige Länge"); return; }
                let var = p.get(2).map(String::as_str).unwrap_or("i");
                let vals = self.indexed_values(n.v, &p[1], var);
                if self.strict { if let Some(v) = vals.iter().find(|v| v.known && !(-32768..=65535).contains(&v.v)) { self.err("E203", format!("Wert {} außerhalb 16 Bit", v.v)); } }
                let mut bytes: Vec<u8> = vals.iter().map(|v| (v.v & 0xFF) as u8).collect();
                bytes.extend(vals.iter().map(|v| ((v.v >> 8) & 0xFF) as u8));
                self.emit(&bytes);
            }
            ".page" => {
                // nächste Seite (256 Byte) bzw. mit Argument die nächste durch n teilbare Adresse; ohne Füllbytes, neues Segment
                let n = if args.trim().is_empty() { 256 } else { self.ev(args).v };
                if n < 1 { self.err("E203", ".page: Größe muss größer als 0 sein"); return; }
                let pc = self.pc.unwrap_or(0); let v = (pc + n - 1) / n * n;
                if self.strict && !(0..=0xFFFF).contains(&v) { self.err("E203", ".page: Adresse außerhalb 0..65535"); }
                self.pc = Some(v); self.real = Some(v); self.begin_segment(v as u16);
            }
            ".align" => {
                let n = self.ev(args).v.max(1);
                if let Some(pc) = self.pc { let pad = (n - pc % n) % n; self.emit(&vec![0; pad as usize]); }
            }
            ".incbin" => {
                let p = split_top(args, ',');
                let Some(name) = p.first().and_then(|s| quoted(s)).map(str::to_string) else { self.err("E208", ".incbin erwartet einen Dateinamen"); return };
                let Some(path) = resolve(&self.file, &name, &self.opts.include_dirs) else { self.err("E212", format!("Datei '{name}' nicht gefunden")); return };
                match std::fs::read(&path) {
                    Ok(data) => {
                        let off = if p.len() > 1 { self.ev(&p[1]).v.max(0) as usize } else { 0 };
                        let len = if p.len() > 2 { self.ev(&p[2]).v.max(0) as usize } else { data.len().saturating_sub(off) };
                        if off + len > data.len() { self.err("E203", ".incbin: Bereich außerhalb der Datei"); } else { self.emit(&data[off..off + len]); }
                    }
                    Err(e) => self.err("E212", format!("{}: {e}", path.display())),
                }
            }
            ".pseudopc" => {
                let v = self.ev(args);
                match self.pc {
                    Some(pc) => { self.real = Some(self.real.unwrap_or(pc)); self.pc = Some(v.v); }
                    None => self.err("E300", ".pseudopc vor .org"),
                }
            }
            ".realpc" | ".endpseudopc" => { if self.real.is_some() { self.pc = self.real; } }
            ".run" => { let v = self.ev(args); self.run = Some(v.v); }
            ".cycles" => {
                let mut name = String::new();
                let mut budget = None;
                for w in args.split_whitespace() {
                    if let Some(e) = w.strip_prefix("max=") { let v = self.ev(e); budget = Some(v.v.max(0) as u32); } else { name = w.to_string(); }
                }
                self.open_cycles.push(CycleBlock { name, file: self.file.to_string(), line: self.line, min: 0, max: 0, budget });
            }
            ".endcycles" => match self.open_cycles.pop() {
                Some(b) => {
                    if let Some(limit) = b.budget {
                        if b.max > limit {
                            self.warn("W201", format!("Zyklenbudget '{}' überschritten: worst-case {} > {} Zyklen", b.name, b.max, limit));
                        }
                    }
                    self.cycle_reports.push(CycleReport { name: b.name, file: b.file, line: b.line, end_line: self.line, min: b.min, max: b.max, budget: b.budget });
                }
                None => self.err("E210", ".endcycles ohne .cycles"),
            },
            ".const" | ".equ" => {
                if let Some((n, e)) = args.split_once('=') { let v = self.ev(e.trim()); self.define(n.trim(), v.v); }
                else { self.err("E208", ".const erwartet NAME = Ausdruck"); }
            }
            ".branchlong" => match args.trim().to_ascii_lowercase().as_str() {
                "auto" | "on" => { self.branch_auto = true; self.branch_jmp = false; }
                "jmp" => { self.branch_auto = true; self.branch_jmp = true; }
                "off" => self.branch_auto = false,
                _ => self.err("E208", ".branchlong erwartet auto, jmp oder off"),
            },
            ".exitmacro" => { if self.depth == 0 { self.err("E210", ".exitmacro außerhalb eines Makros"); } else { self.exit_macro = true; } }
            ".target" => {}
            ".cpu" => {
                if let Some(i) = args.find("illegals") {
                    let word = args[i + 8..].trim_start().trim_start_matches('=').trim_start()
                        .split(|c: char| !c.is_alphabetic()).next().unwrap_or("");
                    match Policy::parse(word) { Some(p) => self.policy = p, None => self.err("E208", format!("illegals: unbekannter Wert '{word}'")) }
                }
            }
            ".basicstub" => {
                // .basicstub ziel [, zeilennummer] [, "kommentar"]  →  <zeile> SYS <ziel> : REM <KOMMENTAR>
                // Ohne .org und ohne bisherige Ausgabe beginnt der Stub an der BASIC-Startadresse des Zielsystems
                let start = self.opts.basic_start.map(i64::from);
                if self.pc.is_none() {
                    let a = start.unwrap_or(0x0801);
                    self.pc = Some(a); self.real = Some(a); self.begin_segment(a as u16);
                } else if self.segments.iter().any(|s| !s.bytes.is_empty()) {
                    self.err("E218", ".basicstub muss am Anfang des Programms stehen (davor wurden schon Bytes ausgegeben)");
                    return;
                } else if let (Some(b), Some(pc)) = (start, self.pc) {
                    // Ausnahmen bleiben möglich: Module (.cartridge) und .pseudopc-Abschnitte
                    if pc != b && self.cart.is_none() && self.real.map_or(true, |r| r == pc) {
                        self.warn("W204", format!(".basicstub steht bei ${pc:04X}, die BASIC-Startadresse des Zielsystems ist ${b:04X} (BASIC findet die Zeile dort nicht)"));
                    }
                }
                let parts = split_top(args, ',');
                let v = self.ev(&parts[0]);
                let target = if v.known { v.v } else { 2061 };
                let (mut line, mut rem) = (10i64, None::<String>);
                for p in &parts[1..] {
                    let p = p.trim();
                    if p.is_empty() { continue; }
                    if let Some(t) = quoted(p) { rem = Some(t.to_string()); }
                    else {
                        let n = self.ev(p);
                        if n.known { line = n.v; }
                    }
                }
                if !(0..=63999).contains(&line) { self.err("E203", format!(".basicstub: Zeilennummer {line} außerhalb 0–63999")); return; }
                let mut body = vec![0x9E];
                body.extend(target.to_string().bytes());
                if let Some(t) = rem {
                    let Some(txt) = self.text_bytes(&t, tokens::Names::Petscii, &petscii) else { return };
                    body.extend([b':', 0x8F, b' ']);
                    body.extend(txt);
                }
                if body.len() > 200 { self.err("E203", format!(".basicstub: BASIC-Zeile zu lang ({} Bytes, höchstens 200)", body.len())); return; }
                let pc = self.pc.unwrap_or(0x0801);
                let next = pc + 2 + 2 + body.len() as i64 + 1;
                let mut b = vec![(next & 0xFF) as u8, (next >> 8) as u8, (line & 0xFF) as u8, (line >> 8) as u8];
                b.extend(body);
                b.extend([0, 0, 0]);
                self.emit(&b);
            }
            ".assert" => {
                let p = split_top(args, ',');
                let v = self.ev(&p[0]);
                if self.strict && v.v == 0 { self.err("E207", format!("Assertion fehlgeschlagen{}", p.get(1).map(|m| format!(": {m}")).unwrap_or_default())); }
            }
            ".error" => self.err("E207", args.trim_matches('"').to_string()),
            ".warning" => self.warn("W200", args.trim_matches('"').to_string()),
            _ => self.err("E208", format!("unbekannte Direktive '{name}'")),
        }
    }

    fn instruction(&mut self, mn: &str, args: &str) {
        let mn = if self.cpu.has_mnemonic(mn) { mn } else { ALIASES.iter().find(|(a, _)| *a == mn).map(|(_, r)| *r).unwrap_or(mn) };
        let mn = if mn == "CMPQ" { "CPQ" } else { mn };
        // 45GS02: Q-Befehle (LDQ, STQ, ADCQ …) = Präfix NEG NEG + Grundbefehl
        let mn = if mn == "NOP" && !self.cpu.has_mnemonic("NOP") && self.cpu.has_mnemonic("EOM") { "EOM" } else { mn };   // 45GS02: NOP = EOM ($EA)
        let (mn, quad) = match self.cpu.quad_base(mn) { Some(b) => (b.to_string(), true), None => (mn.to_string(), false) };
        let mn = mn.as_str();
        // Adressmodus erzwingen: `lda.z` / `lda.zp` (Zeropage), `lda.a` / `lda.abs` (absolut)
        let (mn, force) = match mn.rsplit_once('.') {
            Some((b, suf)) if !self.cpu.has_mnemonic(mn) => match suf.to_ascii_lowercase().as_str() {
                "z" | "zp" => (b, Some('z')),
                "a" | "abs" => (b, Some('a')),
                _ => (mn, None),
            },
            _ => (mn, None),
        };
        if !self.cpu.has_mnemonic(mn) { self.err("E205", format!("unbekannter Befehl '{mn}'")); return; }
        let operand = match parse_operand(args) { Ok(o) => o, Err(e) => { self.err("E206", e); return } };
        if force.is_some() && !matches!(operand, Operand::Direct(..)) {
            self.err("E215", format!("Suffix .{} gilt nur für Zeropage- und absolute Adressierung", if force == Some('z') { "z" } else { "a" })); return;
        }
        let pc = self.pc.unwrap_or(0);
        let has = |m: Mode| self.cpu.lookup(mn, m).is_some();
        let (mut prefix32, mut second): (bool, Option<Val>) = (false, None);
        let (mode, val): (Mode, Val) = match &operand {
            Operand::None => if has(Mode::Imp) { (Mode::Imp, Val::int(0, true)) } else if has(Mode::Acc) { (Mode::Acc, Val::int(0, true)) }
                             else { self.err("E206", format!("{mn} braucht einen Operanden")); return },
            Operand::Acc => (Mode::Acc, Val::int(0, true)),
            Operand::Imm(e) => (if !has(Mode::Imm) && has(Mode::Imw) { Mode::Imw } else { Mode::Imm }, self.ev(e)),
            Operand::IndX(e) => (if !has(Mode::Izx) && has(Mode::Iax) { Mode::Iax } else { Mode::Izx }, self.ev(e)),
            Operand::IndY(e) => (Mode::Izy, self.ev(e)),
            Operand::Ind(e) => (if !has(Mode::Ind) && quad && has(Mode::Izz) { Mode::Izz } else { Mode::Ind }, self.ev(e)),
            Operand::IndZ(e) => (Mode::Izz, self.ev(e)),
            Operand::IndSpY(e) => (Mode::Isy, self.ev(e)),
            Operand::Ind32(e) => { prefix32 = true; (Mode::Izz, self.ev(e)) }
            Operand::Pair(a, b) => { second = Some(self.ev(b)); (Mode::Zpr, self.ev(a)) }
            Operand::Direct(e, idx) if force.is_some() => {
                let v = self.ev(e);
                let z = force == Some('z');
                let m = match (z, idx) {
                    (true, None) => Mode::Zp, (true, Some('x')) => Mode::Zpx, (true, _) => Mode::Zpy,
                    (false, None) => Mode::Abs, (false, Some('x')) => Mode::Abx, (false, _) => Mode::Aby,
                };
                if !has(m) {
                    let what = if has(Mode::Rel) || has(Mode::Rl) { "Sprungbefehle haben nur eine Form" } else if z { "keine Zeropage-Form" } else { "keine absolute Form" };
                    self.err("E215", format!("{mn}.{}: {what}", if z { "z" } else { "a" })); return;
                }
                (m, v)
            }
            Operand::Direct(e, idx) => {
                let v = self.ev(e);
                let small = v.known && (0..256).contains(&v.v);
                let m = match idx {
                    None => if has(Mode::Rel) { Mode::Rel } else if has(Mode::Rl) && !has(Mode::Abs) && !has(Mode::Zp) { Mode::Rl } else if small && has(Mode::Zp) { Mode::Zp } else if has(Mode::Abs) { Mode::Abs } else { Mode::Zp },
                    Some('x') => if small && has(Mode::Zpx) { Mode::Zpx } else if has(Mode::Abx) { Mode::Abx } else { Mode::Zpx },
                    _ => if small && has(Mode::Zpy) { Mode::Zpy } else if has(Mode::Aby) { Mode::Aby } else { Mode::Zpy },
                };
                (m, v)
            }
        };
        let Some(op) = self.cpu.lookup(mn, mode).cloned() else {
            self.err("E206", format!("{mn} unterstützt die Adressierungsart {mode:?} nicht")); return
        };
        if mode == Mode::Rel && self.branch_auto && !quad && !prefix32 {
            let idx = self.branch_idx;
            self.branch_idx += 1;
            let off = val.v - (pc + 2);
            if val.known && !(-128..=127).contains(&off) { self.long.insert(idx); }
            if self.long.contains(&idx) && self.long_branch(mn, &val, pc) { return; }
        }
        if op.group != "official" {
            let unstable = op.group == "illegal.unstable";
            if !self.cpu.has_group(&op.group) { self.err("E201", format!("{mn} existiert auf CPU {} nicht", self.cpu.id)); }
            else if self.policy == Policy::Deny { self.err("E201", format!("undokumentierter Opcode {mn} (illegals=deny)")); }
            else if unstable { self.warn("W105", format!("{mn} ist ein instabiler undokumentierter Opcode")); }
            else if self.policy == Policy::Warn { self.warn("W104", format!("{mn} ist ein undokumentierter Opcode (illegals=warn)")); }
        }
        if quad && !matches!(mode, Mode::Imp | Mode::Acc | Mode::Zp | Mode::Zpx | Mode::Abs | Mode::Abx | Mode::Izz) { self.err("E206", format!("{mn}Q unterstützt die Adressierungsart {mode:?} nicht")); return; }
        if quad && !self.cpu.has_group("mega65.ce02") { self.err("E201", format!("{mn}Q existiert auf CPU {} nicht", self.cpu.id)); return; }
        if prefix32 && self.cpu.nop32.is_none() { self.err("E206", "32-Bit-Zeiger [..] gibt es auf dieser CPU nicht"); return; }
        let mut bytes: Vec<u8> = vec![];
        if quad { bytes.extend(&self.cpu.quad_prefix); }
        if prefix32 { bytes.push(self.cpu.nop32.unwrap()); }
        let plen = bytes.len() as i64;
        bytes.push(op.opcode);
        match mode {
            Mode::Imp | Mode::Acc => {}
            Mode::Imm => bytes.push(self.byte(val, -128, 255)),
            Mode::Zp | Mode::Zpx | Mode::Zpy | Mode::Izx | Mode::Izy | Mode::Izz | Mode::Isy => bytes.push(self.byte(val, 0, 255)),
            Mode::Imw => { if self.strict && val.known && !(-32768..=65535).contains(&val.v) { self.err("E203", format!("Wert {} außerhalb 16 Bit", val.v)); } bytes.extend([(val.v & 0xFF) as u8, ((val.v >> 8) & 0xFF) as u8]); }
            Mode::Zpr => {
                bytes.push(self.byte(val, 0, 255));
                let off = second.as_ref().map_or(0, |t| t.v) - (pc + plen + 3);
                if self.strict && second.as_ref().map_or(false, |t| t.known) && !(-128..=127).contains(&off) { self.err("E203", format!("Sprung zu weit ({off} Bytes)")); }
                bytes.push((off & 0xFF) as u8);
            }
            Mode::Rl => {
                let off = val.v - (pc + plen + 2);
                if self.strict && val.known && !(-32768..=32767).contains(&off) { self.err("E203", format!("Sprung zu weit ({off} Bytes)")); }
                bytes.extend([(off & 0xFF) as u8, ((off >> 8) & 0xFF) as u8]);
            }
            Mode::Abs | Mode::Abx | Mode::Aby | Mode::Ind | Mode::Iax => {
                if self.strict && val.known && !(0..=0xFFFF).contains(&val.v) { self.err("E203", format!("Adresse {} außerhalb 0..65535", val.v)); }
                bytes.extend([(val.v & 0xFF) as u8, ((val.v >> 8) & 0xFF) as u8]);
            }
            Mode::Rel => {
                let off = val.v - (pc + plen + 2);
                if self.strict && val.known && !(-128..=127).contains(&off) { self.err("E203", format!("Sprung zu weit ({off} Bytes)")); }
                bytes.push((off & 0xFF) as u8);
            }
        }
        let worst = op.cycles + op.page_cross as u8 + 2 * op.branch as u8;
        self.last_cycles = Some((op.cycles, worst));
        for b in &mut self.open_cycles { b.min += op.cycles as u32; b.max += worst as u32; }
        self.emit(&bytes);
    }

    /// Zu weiter Sprung: 45GS02 hat lange Branches (Mode::Rl); sonst invertierter Branch über ein `JMP`. `false`, wenn es keine lange Form gibt.
    fn long_branch(&mut self, mn: &str, val: &Val, pc: i64) -> bool {
        let (bytes, cycles): (Vec<u8>, (u8, u8)) = if let Some(rl) = self.cpu.lookup(&format!("L{mn}"), Mode::Rl).cloned().filter(|_| !self.branch_jmp) {   // 45GS02: LBEQ, LBRA …
            let off = val.v - (pc + 2);
            if self.strict && val.known && !(-32768..=32767).contains(&off) { self.err("E203", format!("Sprung zu weit ({off} Bytes)")); }
            (vec![rl.opcode, (off & 0xFF) as u8, ((off >> 8) & 0xFF) as u8], (rl.cycles, rl.cycles + rl.page_cross as u8 + 2 * rl.branch as u8))
        } else {
            let inverse = match mn { "BEQ" => "BNE", "BNE" => "BEQ", "BCC" => "BCS", "BCS" => "BCC", "BMI" => "BPL", "BPL" => "BMI", "BVC" => "BVS", "BVS" => "BVC", _ => "" };
            let Some(jmp) = self.cpu.lookup("JMP", Mode::Abs).cloned() else { return false };
            if mn == "BRA" {
                if self.strict && val.known && !(0..=0xFFFF).contains(&val.v) { self.err("E203", format!("Adresse {} außerhalb 0..65535", val.v)); }
                (vec![jmp.opcode, (val.v & 0xFF) as u8, ((val.v >> 8) & 0xFF) as u8], (jmp.cycles, jmp.cycles))
            } else {
                let Some(inv) = self.cpu.lookup(inverse, Mode::Rel).cloned() else { return false };
                if self.strict && val.known && !(0..=0xFFFF).contains(&val.v) { self.err("E203", format!("Adresse {} außerhalb 0..65535", val.v)); }
                (vec![inv.opcode, 3, jmp.opcode, (val.v & 0xFF) as u8, ((val.v >> 8) & 0xFF) as u8], (inv.cycles + 1, inv.cycles + jmp.cycles))
            }
        };
        self.warn("W203", format!("Sprung zu weit für {mn}: lange Form ({} Bytes) verwendet", bytes.len()));
        self.last_cycles = Some(cycles);
        for b in &mut self.open_cycles { b.min += cycles.0 as u32; b.max += cycles.1 as u32; }
        self.emit(&bytes);
        true
    }

    fn macro_call(&mut self, name: &str, args: &str) -> bool {
        let Some(m) = self.macros.get(name) else { return false };
        if self.depth >= 32 { self.err("E211", format!("Makro '{name}' zu tief verschachtelt")); return true; }
        let call_args: Vec<String> = split_top(args.trim().trim_start_matches('(').trim_end_matches(')'), ',')
            .into_iter().filter(|a| !a.is_empty()).collect();
        let required = m.defaults.iter().filter(|d| d.is_none()).count();
        if call_args.len() < required || call_args.len() > m.params.len() {
            let soll = if required == m.params.len() { required.to_string() } else { format!("{required} bis {}", m.params.len()) };
            self.err("E209", format!("Makro '{name}' erwartet {soll} Argument(e), erhalten {}", call_args.len()));
            return true;
        }
        let mut call_args = call_args;
        for d in &m.defaults[call_args.len()..] { call_args.push(d.clone().unwrap_or_default()); }
        self.macro_counter += 1;
        let counter = self.macro_counter;
        let lines: Vec<tree::RawLine> = m.lines.iter()
            .map(|l| tree::RawLine { file: l.file.clone(), line: l.line, text: substitute(&l.text, &m.params, &call_args, counter) }).collect();
        let mut errs = vec![];
        let mut dummy = Macros::new();
        let nodes = build(&lines, &mut dummy, &mut errs);
        if self.strict { self.diags.extend(errs); }
        let (f, l) = (self.file.clone(), self.line);
        self.depth += 1;
        let outer_loops = std::mem::take(&mut self.loop_depth);
        self.run_nodes(&nodes);
        self.loop_depth = outer_loops;
        self.exit_macro = false;
        self.depth -= 1;
        self.file = f;
        self.line = l;
        true
    }

    fn stmt(&mut self, s: &Stmt) {
        self.line = s.line;
        self.file = s.file.clone();
        self.trace.clear();
        self.last_cycles = None;
        let start_pc = self.pc;
        if let Some(l) = s.label.as_deref().filter(|l| *l == "+" || *l == "-") {
            match self.pc {
                Some(pc) => {
                    let n = if l == "-" { &mut self.anon_minus } else { &mut self.anon_plus };
                    let key = format!("{l}#{n}");
                    *n += 1;
                    self.new_syms.insert(key, pc);
                }
                None => self.err("E300", "Label vor .org"),
            }
        } else if let Some(l) = &s.label {
            match self.pc {
                Some(pc) => {
                    if !l.starts_with('.') { self.scope = qualified(l, "", &self.path_str); }
                    self.define(l, pc);
                }
                None => self.err("E300", "Label vor .org"),
            }
        }
        match &s.body {
            Body::Empty => {}
            Body::Assign(n, e) => {
                if let Some(k) = self.var_key(n) { let v = self.ev_raw(e); self.vars.insert(k, v); }
                else { let v = self.ev(e); self.define(n, v.v); }
            }
            Body::Dir(n, a) if n == ".scope" => { let name = s.label.as_deref().filter(|l| !l.starts_with('.')); self.open_scope(name); if !a.is_empty() { self.err("E213", "nach '{' darf nichts in derselben Zeile stehen"); } }
            Body::Dir(n, _) if n == ".endscope" => self.close_scope(),
            Body::Dir(n, a) if n == ".namespace" => {
                let nm = a.trim().trim_end_matches('{').trim();
                if nm.is_empty() || !a.trim_end().ends_with('{') { self.err("E213", ".namespace erwartet: .namespace NAME {"); } else { self.open_scope(Some(nm)); }
            }
            Body::Dir(n, a) => self.directive(n, a),
            Body::Ins(m, a) => {
                if self.macros.contains_key(m) {
                    self.macro_call(m, a);
                    if s.label.is_some() {
                        self.lines.push(LineInfo { file: s.file.to_string(), line: s.line, addr: start_pc.unwrap_or(0) as u16, bytes: vec![], cycles: None });
                    }
                    return;
                }
                self.instruction(m, a);
            }
        }
        if s.label.is_some() || !matches!(s.body, Body::Empty) {
            self.lines.push(LineInfo { file: self.file.to_string(), line: s.line, addr: start_pc.unwrap_or(0) as u16,
                                       bytes: self.trace.clone(), cycles: self.last_cycles });
        }
    }

    /// Wert einer Zuweisung `name = ausdruck` / `name += ausdruck` aus Text lesen
    fn parse_assign(&mut self, text: &str) -> Option<(String, String)> {
        match parse::parse_line(self.line, text) {
            Ok(Stmt { body: Body::Assign(n, e), .. }) => Some((n, e)),
            _ => { self.err("E208", format!("Zuweisung erwartet statt '{text}'")); None }
        }
    }
    fn run_for(&mut self, spec: &ForSpec, body: &[Node]) {
        const LIMIT: usize = 65536;
        match spec {
            ForSpec::C { init, cond, step } => {
                let Some((var, e)) = self.parse_assign(init) else { return };
                let key = qualified(&var, &self.scope, &self.path_str);
                let saved = self.vars.get(&key).cloned();
                let v = self.ev_raw(&e);
                self.vars.insert(key.clone(), v);
                let mut n = 0;
                while self.ev(cond).v != 0 {
                    n += 1;
                    if n > LIMIT { self.err("E217", format!(".for: mehr als {LIMIT} Durchläufe")); break; }
                    self.run_nodes(body);
                    if self.loop_ctl.take() == Some(LoopCtl::Break) || self.exit_macro { break; }
                    let Some((sv, se)) = self.parse_assign(step) else { break };
                    match self.var_key(&sv) {
                        Some(k) => { let v = self.ev_raw(&se); self.vars.insert(k, v); }
                        None => { self.err("E216", format!("Variable '{sv}' ist nicht mit .var angelegt")); break; }
                    }
                }
                match saved { Some(v) => { self.vars.insert(key, v); } None => { self.vars.remove(&key); } }
            }
            ForSpec::In { var, expr } => {
                let list = self.ev_raw(expr);
                let Some(items) = list.list.clone() else { self.err("E202", format!(".for … in erwartet eine Liste oder range(…), nicht '{}'", expr.trim())); return };
                let key = qualified(var, &self.scope, &self.path_str);
                let saved = self.vars.get(&key).cloned();
                for (n, item) in items.iter().enumerate() {
                    if n >= LIMIT { self.err("E217", format!(".for: mehr als {LIMIT} Durchläufe")); break; }
                    self.vars.insert(key.clone(), item.clone());
                    self.run_nodes(body);
                    if self.loop_ctl.take() == Some(LoopCtl::Break) || self.exit_macro { break; }
                }
                match saved { Some(v) => { self.vars.insert(key, v); } None => { self.vars.remove(&key); } }
            }
        }
    }
    /// `.struct NAME` / `.enum NAME`: definiert `NAME.feld` (Offset bzw. Wert) und bei Strukturen `NAME.size`
    fn run_struct(&mut self, s: &StructNode) {
        self.line = s.line;
        self.file = s.file.clone();
        let name = s.name.trim();
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.starts_with(|c: char| c.is_ascii_digit()) {
            self.err("E208", format!("{}: ungültiger Name '{name}'", if s.is_enum { ".enum" } else { ".struct" })); return;
        }
        if self.structs.contains_key(name) { self.err("E204", format!("'{name}' doppelt definiert")); return; }
        let mut members: Vec<(String, i64)> = vec![];
        let (mut offset, mut next) = (0i64, 0i64);
        for rl in &s.body {
            self.line = rl.line;
            self.file = rl.file.clone();
            let text = parse::strip_comment(&rl.text).trim().to_string();
            if text.is_empty() { continue; }
            if s.is_enum {
                for item in split_top(&text, ',') {
                    if item.is_empty() { continue; }
                    let (n, v) = match item.split_once('=') {
                        Some((n, e)) if !e.starts_with('=') => (n.trim().to_string(), Some(e.trim().to_string())),
                        _ => (item.trim().to_string(), None),
                    };
                    if !valid_name(&n) { self.err("E208", format!(".enum: ungültiger Name '{n}'")); continue; }
                    if let Some(e) = v { next = self.ev(&e).v; }
                    members.push((n.clone(), next));
                    self.define(&format!("{name}.{n}"), next);
                    next += 1;
                }
                continue;
            }
            // Feld: name typ [anzahl]
            let (field, after) = text.split_once(char::is_whitespace).map_or((text.as_str(), ""), |(f, r)| (f, r.trim_start()));
            let (ty, count_text) = after.split_once(char::is_whitespace).map_or((after, ""), |(t, r)| (t, r.trim()));
            if ty.is_empty() { self.err("E208", format!(".struct: Feld erwartet 'name typ [anzahl]', gefunden '{text}'")); continue; }
            let count = if count_text.is_empty() { 1 } else { self.ev(count_text).v };
            if !valid_name(field) { self.err("E208", format!(".struct: ungültiger Feldname '{field}'")); continue; }
            if count < 0 || count > 65536 { self.err("E203", format!(".struct: ungültige Anzahl bei '{field}'")); continue; }
            let (size, sub): (i64, Option<StructDef>) = match ty.to_ascii_lowercase().as_str() {
                ".byte" | ".db" => (1, None),
                ".word" | ".dw" | ".addr" => (2, None),
                ".dword" | ".long" => (4, None),
                _ => match self.structs.get(ty) {
                    Some(d) if d.is_enum => { self.err("E208", format!(".struct: '{ty}' ist eine Aufzählung; für das Feld .byte (oder .word) verwenden")); continue }
                    Some(d) => (d.size, Some(d.clone())),
                    _ => { self.err("E208", format!(".struct: unbekannter Feldtyp '{ty}' (.byte, .word, .dword oder eine früher definierte Struktur)")); continue }
                },
            };
            members.push((field.to_string(), offset));
            self.define(&format!("{name}.{field}"), offset);
            if let Some(d) = sub {
                for (m, o) in &d.members {
                    members.push((format!("{field}.{m}"), offset + o));
                    self.define(&format!("{name}.{field}.{m}"), offset + o);
                }
            }
            offset += size * count;
        }
        if !s.is_enum { self.define(&format!("{name}.size"), offset); }
        self.structs.insert(name.to_string(), StructDef { size: if s.is_enum { 0 } else { offset }, members, is_enum: s.is_enum });
    }
    fn declare_function(&mut self, args: &str) {
        let (Some(open), Some(close)) = (args.find('('), args.find(')')) else { self.err("E208", ".function erwartet: .function name(param, …) = ausdruck"); return };
        let name = args[..open].trim();
        let Some(body) = args[close + 1..].trim().strip_prefix('=').filter(|b| !b.starts_with('=')) else { self.err("E208", ".function erwartet: .function name(param, …) = ausdruck"); return };
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.starts_with(|c: char| c.is_ascii_digit()) {
            self.err("E208", format!(".function: ungültiger Name '{name}'")); return;
        }
        let params: Vec<String> = split_top(&args[open + 1..close], ',').into_iter().filter(|p| !p.is_empty()).collect();
        if expr::is_builtin(&name.to_ascii_lowercase()) { self.err("E204", format!("'{name}' ist eine eingebaute Funktion")); return; }
        if self.funcs.contains_key(name) { self.err("E204", format!("Funktion '{name}' doppelt definiert")); return; }
        self.funcs.insert(name.to_string(), Func { params, body: body.trim().to_string() });
    }
    fn run_nodes(&mut self, nodes: &[Node]) {
        for n in nodes {
            if self.exit_macro || self.loop_ctl.is_some() { return; }
            match n {
                Node::Stmt(s) => self.stmt(s),
                Node::If(f) => {
                    self.line = f.line;
                    self.file = f.file.clone();
                    for (cond, body) in &f.arms {
                        let take = match cond {
                            Cond::Expr(e) => self.ev(e).v != 0,
                            Cond::Defined(n) => self.is_defined(n),
                            Cond::NotDefined(n) => !self.is_defined(n),
                            Cond::Else => true,
                        };
                        if take { self.run_nodes(body); break; }
                    }
                }
                Node::Repeat(r) => {
                    self.line = r.line;
                    self.file = r.file.clone();
                    let n = self.ev(&r.count);
                    if n.v < 0 || n.v > 65536 { self.err("E203", ".repeat: ungültige Anzahl"); continue; }
                    self.loop_depth += 1;
                    for i in 0..n.v {
                        if self.exit_macro { break; }
                        if let Some(v) = &r.var { self.locals.insert(v.clone(), i); }
                        self.run_nodes(&r.body);
                        if self.loop_ctl.take() == Some(LoopCtl::Break) { break; }
                    }
                    self.loop_depth -= 1;
                    if let Some(v) = &r.var { self.locals.remove(v); }
                }
                Node::Struct(s) => self.run_struct(s),
                Node::For(f) => {
                    self.line = f.line;
                    self.file = f.file.clone();
                    self.loop_depth += 1;
                    self.run_for(&f.spec, &f.body);
                    self.loop_depth -= 1;
                }
            }
        }
    }
}

pub fn assemble(src: &str, cpu: &Cpu, opts: &Options) -> Result<Output, Vec<Diagnostic>> {
    let mut raw = vec![];
    let mut files = vec![];
    load(src, &opts.file, &opts.include_dirs, 0, &mut raw, &mut files).map_err(|d| vec![d])?;
    let mut macros = Macros::new();
    let mut errs = vec![];
    let nodes = build(&raw, &mut macros, &mut errs);
    for name in macros.keys() {
        if cpu.has_mnemonic(name) { errs.push(Diagnostic::error("E209", &opts.file, 0, format!("Makroname '{name}' kollidiert mit einem Befehl"))); }
    }
    if !errs.is_empty() { return Err(errs); }
    let base: HashMap<String, i64> = opts.defines.iter().chain(opts.predefined.iter()).cloned().collect();
    let mut syms = base.clone();
    let mut converged = false; let mut passes = 0usize;
    let mut long: std::collections::HashSet<usize> = Default::default();
    for _ in 0..16 {
        passes += 1;
        let mut p = new_pass(cpu, opts, &macros, &syms, &base, false);
        p.long = long.clone();
        p.run_nodes(&nodes);
        if p.new_syms == syms && p.long.len() == long.len() { converged = true; break; }
        let (ns, nl) = (p.new_syms, p.long);
        syms = ns;
        long = nl;
    }
    if !converged { return Err(vec![Diagnostic::error("E300", &opts.file, 0, "Symbole konvergieren nicht (zu viele Durchläufe)")]); }
    let mut p = new_pass(cpu, opts, &macros, &syms, &base, true);
    p.long = long;
    p.run_nodes(&nodes);
    if !p.path.is_empty() { p.diags.push(Diagnostic::error("E213", &opts.file, p.line, format!("Scope '{}' wird nicht mit '}}' geschlossen", p.path_str))); }
    for b in std::mem::take(&mut p.open_cycles) {
        p.diags.push(Diagnostic::error("E210", &b.file, b.line, format!(".cycles '{}' ohne .endcycles", b.name)));
    }
    let mut macro_list: Vec<(String, Vec<String>)> = macros.iter().map(|(n, m)| (n.clone(), m.params.clone())).collect(); macro_list.sort();
    let (diags, segments, lines, mut new_syms, cycles, run, defs, cart) = (p.diags, p.segments, p.lines, p.new_syms, p.cycle_reports, p.run, p.defs, p.cart);
    for (n, _) in &opts.predefined { new_syms.remove(n); }
    new_syms.retain(|k, _| !k.starts_with("+#") && !k.starts_with("-#"));
    if diags.iter().any(|d| d.is_error()) { return Err(diags); }
    let mut segs: Vec<Segment> = segments.into_iter().filter(|s| !s.bytes.is_empty()).collect();
    segs.sort_by_key(|s| (s.bank, s.start));
    for w in segs.windows(2) {
        if w[0].bank == w[1].bank && w[0].start as usize + w[0].bytes.len() > w[1].start as usize {
            return Err(vec![Diagnostic::error("E301", &opts.file, 0, format!("Segmente überlappen{}: ${:04X} und ${:04X}", if w[0].bank != 0 { format!(" (Bank {})", w[0].bank) } else { String::new() }, w[0].start, w[1].start))]);
        }
    }
    if let Some(s) = segs.iter().find(|s| s.start as usize + s.bytes.len() > 0x10000) { return Err(vec![Diagnostic::error("E301", &opts.file, 0, format!("Segment bei ${:04X} überschreitet $FFFF", s.start))]); }
    let sources = files.into_iter().map(|(f, t)| (f, t.lines().map(str::to_string).collect())).collect();
    let expanded = raw.iter().map(|r| (r.file.to_string(), r.line, r.text.clone())).collect();
    Ok(Output { defs: defs.into_iter().collect(), run: run.map(|r| r as u16), cycles, segments: segs, cart, symbols: new_syms.into_iter().collect(), macros: macro_list, lines, warnings: diags, sources, passes: passes + 1, expanded })
}

fn new_pass<'a>(cpu: &'a Cpu, opts: &'a Options, macros: &'a Macros, syms: &'a HashMap<String, i64>,
                base: &HashMap<String, i64>, strict: bool) -> Pass<'a> {
    Pass { real: None, run: None, open_cycles: vec![], cycle_reports: vec![], cpu, opts, macros, strict, syms, new_syms: base.clone(), defs: HashMap::new(), locals: HashMap::new(), segments: vec![], bank: 0, bank_pos: HashMap::new(), seg_name: String::new(), seg_range: None, cart: None, lines: vec![], diags: vec![],
           pc: None, scope: String::new(), path: vec![], path_str: String::new(), scope_stack: vec![], anon: 0, enc: Enc::Ascii, blocks: vec![], policy: opts.illegals, line: 0, file: Rc::from(opts.file.as_str()), trace: vec![],
           last_cycles: None, macro_counter: 0, structs: HashMap::new(), branch_auto: opts.branch_long, branch_jmp: false, branch_idx: 0, long: Default::default(), vars: HashMap::new(), funcs: HashMap::new(), loop_ctl: None, loop_depth: 0, exit_macro: false, depth: 0, anon_minus: 0, anon_plus: 0 }
}

/// PRG: 2 Byte Ladeadresse + zusammenhängende Daten (Lücken mit 0 gefüllt).
fn single_bank(out: &Output) -> Result<(), String> {
    if out.segments.iter().any(|s| s.bank != 0) { Err("mehrere Bänke (.bank) gibt es nur in den Modulformaten crt/car/rom".into()) } else { Ok(()) }
}

pub fn to_prg(out: &Output) -> Result<Vec<u8>, String> {
    single_bank(out)?;
    let first = out.segments.first().ok_or("keine Ausgabe")?;
    let mut v = vec![(first.start & 0xFF) as u8, (first.start >> 8) as u8];
    v.extend(to_bin(out)?);
    Ok(v)
}

pub fn to_bin(out: &Output) -> Result<Vec<u8>, String> {
    single_bank(out)?;
    let first = out.segments.first().ok_or("keine Ausgabe")?;
    let mut data: Vec<u8> = vec![];
    for s in &out.segments {
        let off = s.start as usize - first.start as usize;
        if data.len() < off { data.resize(off, 0); }
        data.extend_from_slice(&s.bytes);
    }
    Ok(data)
}

/// Atari-XEX: Segmente mit `$FFFF`-Kopf; `.run` erzeugt ein RUNAD-Segment (`$02E0`).
pub fn to_xex(out: &Output) -> Result<Vec<u8>, String> {
    single_bank(out)?;
    if out.segments.is_empty() { return Err("keine Ausgabe".into()); }
    let mut v = vec![0xFF, 0xFF];
    let seg = |start: u16, data: &[u8], v: &mut Vec<u8>| {
        let end = start as usize + data.len() - 1;
        v.extend([(start & 0xFF) as u8, (start >> 8) as u8, (end & 0xFF) as u8, (end >> 8) as u8]);
        v.extend_from_slice(data);
    };
    for s in &out.segments { seg(s.start, &s.bytes, &mut v); }
    if let Some(r) = out.run { seg(0x02E0, &[(r & 0xFF) as u8, (r >> 8) as u8], &mut v); }
    Ok(v)
}

/// VICE-Monitor-Labeldatei (`al C:addr .name`).
pub fn vice_labels(out: &Output) -> String {
    let mut s = String::new();
    for (name, v) in &out.symbols {
        if (0..=0xFFFF).contains(v) { s.push_str(&format!("al C:{:04x} .{}\n", v, name.replace('.', "_"))); }
    }
    s
}

/// Kanonisches Debug-Format (.vcdbg, JSON): Symbole, Segmente, Zeilen-Mapping mit Zyklen.
pub fn vcdbg(out: &Output) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let syms: Vec<String> = out.symbols.iter().map(|(n, v)| format!("    {{ \"name\": \"{}\", \"value\": {} }}", esc(n), v)).collect();
    let segs: Vec<String> = out.segments.iter().map(|s| format!("    {{ \"start\": {}, \"length\": {}, \"bank\": {}, \"name\": \"{}\" }}", s.start, s.bytes.len(), s.bank, s.name.replace('"', ""))).collect();
    let lines: Vec<String> = out.lines.iter().filter(|l| !l.bytes.is_empty()).map(|l| {
        let cy = l.cycles.map(|(a, b)| format!(", \"cycles\": [{a}, {b}]")).unwrap_or_default();
        format!("    {{ \"file\": \"{}\", \"line\": {}, \"addr\": {}, \"len\": {}{} }}", esc(&l.file), l.line, l.addr, l.bytes.len(), cy)
    }).collect();
    let cyc: Vec<String> = out.cycles.iter().map(|c| format!("    {{ \"name\": \"{}\", \"file\": \"{}\", \"line\": {}, \"endLine\": {}, \"min\": {}, \"max\": {}, \"budget\": {} }}",
        esc(&c.name), esc(&c.file), c.line, c.end_line, c.min, c.max, c.budget.map_or("null".to_string(), |b| b.to_string()))).collect();
    format!("{{\n  \"format\": 1,\n  \"symbols\": [\n{}\n  ],\n  \"segments\": [\n{}\n  ],\n  \"cycleBlocks\": [\n{}\n  ],\n  \"lines\": [\n{}\n  ]\n}}\n",
            syms.join(",\n"), segs.join(",\n"), cyc.join(",\n"), lines.join(",\n"))
}

/// Listing: Adresse, Bytes, Zeilennummer, Quelltext.
pub fn listing(out: &Output) -> String {
    let src: HashMap<&str, &Vec<String>> = out.sources.iter().map(|(f, l)| (f.as_str(), l)).collect();
    let mut s = String::new();
    for l in &out.lines {
        let text = src.get(l.file.as_str()).and_then(|v| v.get(l.line - 1)).map(String::as_str).unwrap_or("");
        let hex: Vec<String> = l.bytes.iter().take(4).map(|b| format!("{b:02x}")).collect();
        let more = if l.bytes.len() > 4 { "+" } else { " " };
        let root = out.sources.first().map_or("", |(f, _)| f.as_str());
        let pos = if l.file == root { l.line.to_string() } else {
            format!("{}:{}", std::path::Path::new(&l.file).file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned()), l.line)
        };
        s.push_str(&format!("{:04x}  {:<11}{} {:>6}  {}\n", l.addr, hex.join(" "), more, pos, text));
    }
    s
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineKind { Label, Const, Macro }

#[derive(Debug, Clone)]
pub struct OutlineItem { pub name: String, pub line: usize, pub kind: OutlineKind }

/// Leichtgewichtige Gliederung (Labels, Konstanten, Makros) ohne vollständige Assemblierung – auch bei Fehlern verfügbar.
pub fn outline(src: &str) -> Vec<OutlineItem> {
    let mut v = vec![];
    for (i, l) in src.lines().enumerate() {
        let Ok(st) = parse::parse_line(i + 1, l) else { continue };
        if let Some(n) = &st.label { v.push(OutlineItem { name: n.clone(), line: i + 1, kind: OutlineKind::Label }); }
        match &st.body {
            Body::Assign(n, _) => v.push(OutlineItem { name: n.clone(), line: i + 1, kind: OutlineKind::Const }),
            Body::Dir(d, a) if d == ".const" || d == ".equ" => {
                if let Some((n, _)) = a.split_once('=') { v.push(OutlineItem { name: n.trim().to_string(), line: i + 1, kind: OutlineKind::Const }); }
            }
            Body::Dir(d, a) if d == ".macro" => {
                let name = a.split(|c: char| c == '(' || c.is_whitespace()).next().unwrap_or("").to_string();
                if !name.is_empty() { v.push(OutlineItem { name, line: i + 1, kind: OutlineKind::Macro }); }
            }
            _ => {}
        }
    }
    v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartKind { Instruction, Directive, Assignment }

#[derive(Debug, Clone)]
pub struct LineParts { pub label: Option<String>, pub kind: PartKind, pub name: String, pub args: String }

/// Die vom Parser erkannte Zeilenstruktur (für Hover). `None` bei leeren oder fehlerhaften Zeilen.
pub fn line_parts(line: &str) -> (Option<String>, Option<LineParts>) {
    match parse::parse_line(1, line) {
        Ok(st) => {
            let label = st.label.clone();
            let part = match st.body {
                Body::Ins(m, a) => Some(LineParts { label: st.label, kind: PartKind::Instruction, name: m, args: a }),
                Body::Dir(m, a) => Some(LineParts { label: st.label, kind: PartKind::Directive, name: m, args: a }),
                Body::Assign(n, e) => Some(LineParts { label: st.label, kind: PartKind::Assignment, name: n, args: e }),
                Body::Empty => None,
            };
            (label, part)
        }
        Err(_) => (None, None),
    }
}
