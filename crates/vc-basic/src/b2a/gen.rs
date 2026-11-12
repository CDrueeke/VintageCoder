// SPDX-License-Identifier: GPL-3.0-or-later
//! Codegenerator: Syntaxbaum → Assembler-Quelltext (Stufe 1: Ganzzahl, Ziel C64).
//!
//! Regeln: jede BASIC-Zeile wird ein Block mit Kommentar des Originals und eigenem Label,
//! Variablen behalten ihren Namen, Ausdrücke werden so übersetzt, wie sie dastehen.
use super::ast::*;
use super::{B2aOptions, Converted, MapEntry};
use crate::petscii;
use crate::Dialect;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use vc_core::Diagnostic;

/// DATA-Eintrag im Zeichenketten-Modus: Text und (falls es eine Zahl ist) der Zahlenwert
struct SItem { text: Vec<u8>, label: Option<String>, num: Option<f64> }

/// `DEF FN`: Unterprogramm `fn_<Name>`, Merker `fd_<Name>` (gesetzt, sobald die DEF-Anweisung gelaufen ist)
/// Offener Turbo-BASIC-Block: Labels für Anfang/Ende (IF: Else- und Ende-Label)
enum Ctl { If { els: String, end: String, has_else: bool }, While { start: String, end: String }, Repeat { start: String, end: String }, Do { start: String, end: String }, Proc { end: String } }

struct FnInfo { label: String, param: Var, body: Expr, line: usize, number: u32, ret_int: bool }

struct ArrInfo { label: String, name: String, count: i64, dimmed: bool, size: i64 }

struct G<'a> {
    o: &'a B2aOptions,
    d: &'a Dialect,
    errs: Vec<Diagnostic>,
    warns: Vec<Diagnostic>,
    src_line: usize,
    line_no: u32,
    lines: BTreeSet<u32>,
    scalars: BTreeMap<String, (String, String, i64)>,   // Schlüssel → (Label, Anzeigename, Bytes)
    float: bool,
    atari: bool,
    fsz: i64,                // Bytes je Fließkommazahl: 5 (MBF des CBM-BASIC) oder 6 (BCD des Atari-OS)
    str: bool,
    fconsts: Vec<Vec<u8>>,
    fdata: Vec<Vec<Vec<u8>>>,
    sdata: Vec<Vec<SItem>>,
    adata: Vec<Vec<Vec<u8>>>,           // Atari: DATA-Einträge als roher Text je DATA-Zeile
    scap: BTreeMap<String, i64>,       // Atari: Kapazität je Zeichenkettenvariable (Label → größtes DIM)
    fns: BTreeMap<String, FnInfo>,
    subs: String,
    arrays: BTreeMap<String, ArrInfo>,
    strings: Vec<Vec<u8>>,
    data: Vec<(u32, Vec<i64>)>,
    uniq: u32,
    need_stage: u8,
    hint_float: bool,
    hint_big: bool,
    uses_flow: bool,
    uses_io: bool,
    uses_data: bool,
    uses_clr: bool,
    uses_gfx: bool,
    uses_open: bool,
    uses_turbo: bool,
    ctl: Vec<Ctl>,                   // Turbo BASIC XL: offene Blöcke (IF, WHILE, REPEAT, DO, PROC)
    procs: BTreeSet<String>,         // Turbo BASIC XL: Namen der Prozeduren
    uses_run: bool,
    body: String,
}

fn lv_is_str(lv: &LValue) -> bool { matches!(lv, LValue::Var(v) | LValue::Elem(v, _) if v.ty == Ty::Str) }

/// Name nach FN → Schlüssel (erste 2 Zeichen + Typkennzeichen, wie bei Variablen)
fn fn_key(name: &str) -> String {
    let (base, suffix) = match name.chars().last() { Some(c @ ('$' | '%')) => (&name[..name.len() - 1], c.to_string()), _ => (name, String::new()) };
    format!("{}{}", base.chars().take(2).collect::<String>(), suffix)
}

fn is_reserved_scalar(v: &Var) -> bool { v.ty != Ty::Str && matches!(v.name.as_str(), "TI" | "ST") }

fn key_of(v: &Var) -> String {
    let n: String = v.name.chars().take(2).collect();
    format!("{n}{}", match v.ty { Ty::Int => "%", Ty::Str => "$", Ty::Float => "" })
}
fn label_of(prefix: &str, v: &Var) -> String {
    format!("{prefix}_{}{}", v.name, match v.ty { Ty::Int => "_P", Ty::Str => "_S", Ty::Float => "" })
}
fn display_of(v: &Var) -> String { format!("{}{}", v.name, match v.ty { Ty::Int => "%", Ty::Str => "$", Ty::Float => "" }) }

fn hex16(v: i64) -> String { format!("${:04X}", v & 0xFFFF) }

impl<'a> G<'a> {
    fn err(&mut self, code: &'static str, msg: impl Into<String>) { let l = self.src_line; self.errs.push(Diagnostic::error(code, &self.o.file, l, msg)); }
    fn warn(&mut self, code: &'static str, msg: impl Into<String>) { let l = self.src_line; self.warns.push(Diagnostic::warning(code, &self.o.file, l, msg)); }
    fn stage(&mut self, n: u8, what: &str) {
        self.need_stage = self.need_stage.max(n);
        let (code, name) = if n == 2 { ("E301", "Fließkomma") } else { ("E302", "Zeichenketten") };
        self.err(code, format!("{what} braucht Stufe {n} ({name}); erlaubt ist höchstens Stufe {}", self.o.stage));
    }
    fn ins(&mut self, s: impl AsRef<str>) { let _ = writeln!(self.body, "        {}", s.as_ref()); }
    fn label(&mut self, l: &str) { let _ = writeln!(self.body, "{l}:"); }
    fn comment(&mut self, s: &str) { let _ = writeln!(self.body, ";{s}"); }
    fn fresh(&mut self) -> String { self.uniq += 1; format!("L{}_{}", self.line_no, self.uniq) }

    fn scalar(&mut self, v: &Var) -> String {
        let k = key_of(v);
        if let Some((l, shown, _)) = self.scalars.get(&k).cloned() {
            if shown != display_of(v) { self.warn("W312", format!("{} und {} sind im BASIC dieselbe Variable (nur die ersten 2 Zeichen zählen)", shown, display_of(v))); }
            return l;
        }
        if v.ty == Ty::Float && !self.hint_float && !self.float {
            self.hint_float = true;
            self.warn("I110", format!("Variable {} ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen", display_of(v)));
        }
        let l = label_of("v", v);
        let size = if v.ty == Ty::Str { if self.atari { 4 } else { 3 } } else if self.float && v.ty == Ty::Float { self.fsz } else { 2 };
        if self.atari && v.ty == Ty::Str && !self.scap.contains_key(&l) { self.err("E321", format!("Zeichenkettenvariable {} braucht ein DIM mit fester Größe (Atari-BASIC: DIM {}(n), sonst Fehler 9)", display_of(v), display_of(v))); }
        self.scalars.insert(k, (l.clone(), display_of(v), size));
        l
    }
    fn array(&mut self, v: &Var) -> String {
        let k = key_of(v);
        if let Some(a) = self.arrays.get(&k) { return a.label.clone(); }
        let l = label_of("a", v);
        let float = self.float && v.ty == Ty::Float;
        self.arrays.insert(k, ArrInfo { label: l.clone(), name: display_of(v), count: 11, dimmed: false, size: if v.ty == Ty::Str { 3 } else if float { self.fsz } else { 2 } });
        self.uses_data = true;
        l
    }

    // ---- Konstantenfaltung (nur Literale)
    fn konst(&self, e: &Expr) -> Option<i64> {
        let fl = self.float;
        // Stufe 1: 16-Bit-Zahl mit Vorzeichen (läuft um); Stufe 2: echte Zahl, gefaltet wird nur, was als kleine Ganzzahl herauskommt
        let w = |v: i64| if fl { v } else { ((v & 0xFFFF) as u16) as i16 as i64 };
        let r = match e {
            Expr::Num(n) if fl => Some(*n),
            Expr::Num(n) if *n <= 65535 => Some(w(*n)),
            Expr::Un(UnOp::Neg, x) => self.konst(x).map(|v| w(-v)),
            Expr::Un(UnOp::Not, x) => self.konst(x).map(|v| if self.atari { (v == 0) as i64 } else { w(!v) }),
            Expr::Bin(op, a, b) => {
                let (a, b) = (self.konst(a)?, self.konst(b)?);
                let t: i64 = if self.atari { 1 } else { -1 };   // wahr: Atari-BASIC 1, CBM -1
                Some(match op {
                    BinOp::Add => w(a + b), BinOp::Sub => w(a - b), BinOp::Mul => w(a.wrapping_mul(b)),
                    BinOp::Div => { if b == 0 { return None; } if a % b != 0 && (fl || !self.o.int_div) { return None; } w(a / b) }
                    BinOp::Pow => return None,
                    BinOp::Eq => if a == b { t } else { 0 }, BinOp::Ne => if a != b { t } else { 0 },
                    BinOp::Lt => if a < b { t } else { 0 }, BinOp::Gt => if a > b { t } else { 0 },
                    BinOp::Le => if a <= b { t } else { 0 }, BinOp::Ge => if a >= b { t } else { 0 },
                    BinOp::And => if self.atari { (a != 0 && b != 0) as i64 } else { w(a & b) }, BinOp::Or => if self.atari { (a != 0 || b != 0) as i64 } else { w(a | b) },
                })
            }
            _ => None,
        };
        if fl && !matches!(e, Expr::Num(_)) { return r.filter(|v| (-32768..=32767).contains(v)); }
        r
    }

    fn load_imm(&mut self, v: i64) {
        let v = v & 0xFFFF;
        self.ins(format!("lda #{}", if v & 0xFF > 9 { format!("${:02X}", v & 0xFF) } else { (v & 0xFF).to_string() }));
        self.ins(format!("ldy #{}", if v >> 8 > 9 { format!("${:02X}", v >> 8) } else { (v >> 8).to_string() }));
    }
    fn set_ptr(&mut self, zp: &str, label: &str) {
        self.ins(format!("lda #<{label}"));
        self.ins(format!("sta {zp}"));
        self.ins(format!("lda #>{label}"));
        self.ins(format!("sta {zp}+1"));
    }

    // ---- Ausdrücke: Ergebnis in A (niedrig) / Y (hoch)
    fn expr(&mut self, e: &Expr) {
        if self.str && self.str_hook(e) { return; }
        if self.float { return self.fexpr(e); }
        if !matches!(e, Expr::Num(_)) {
            if let Some(v) = self.konst(e) { self.load_imm(v); return; }
        }
        match e {
            Expr::Num(n) => {
                if *n > 65535 { self.err("E303", format!("Zahl {n} ist zu groß für 16 Bit")); return; }
                if *n > 32767 && !self.hint_big {
                    self.hint_big = true;
                    self.warn("I111", "Zahlen über 32767 gelten als 16-Bit-Muster (Adressen für PEEK/POKE/SYS); im Rechnen und im PRINT erscheinen sie negativ");
                }
                self.load_imm(*n);
            }
            Expr::Float(s) => self.stage(2, &format!("Zahl {s}")),
            Expr::Str(_) => self.stage(3, "Zeichenkette in einem Ausdruck"),
            Expr::Var(v) => {
                if v.ty == Ty::Str { self.stage(3, &format!("Zeichenkettenvariable {}$", v.name)); return; }
                if v.ty != Ty::Int && v.name == "TI" { self.ins("jsr rt_ti"); self.warn("I112", "TI liefert nur die unteren 16 Bit der Systemuhr"); return; }
                if v.ty != Ty::Int && v.name == "ST" { self.ins("lda $90"); self.ins("ldy #0"); return; }
                let l = self.scalar(v);
                self.ins(format!("lda {l}"));
                self.ins(format!("ldy {l}+1"));
            }
            Expr::Elem(v, idx) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenfeld"); return; }
                if idx.len() != 1 { self.err("E304", "Stufe 1 kennt nur eindimensionale Felder"); return; }
                self.expr(&idx[0]);
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_ldelem");
            }
            Expr::Un(op, x) => {
                self.expr(x);
                self.ins(match op { UnOp::Neg => "jsr rt_neg", UnOp::Not => "jsr rt_not" });
            }
            Expr::Bin(op, a, b) if self.o.inline && self.inline_bin(op, a, b) => {}
            Expr::Bin(op, a, b) => {
                let rt = match op {
                    BinOp::Add => "rt_add", BinOp::Sub => "rt_sub", BinOp::Mul => "rt_mul",
                    BinOp::Div => { if !self.o.int_div { self.stage(2, "Division '/' (liefert Bruchzahlen; mit --int-div ganzzahlig)"); return; } "rt_div" }
                    BinOp::Pow => { self.stage(2, "Potenz '^'"); return; }
                    BinOp::Eq => "rt_eq", BinOp::Ne => "rt_ne", BinOp::Lt => "rt_lt", BinOp::Gt => "rt_gt", BinOp::Le => "rt_le", BinOp::Ge => "rt_ge",
                    BinOp::And => "rt_and", BinOp::Or => "rt_or",
                };
                self.expr(a);
                self.ins("jsr rt_push");
                self.expr(b);
                self.ins(format!("jsr {rt}"));
            }
            Expr::Call(name, args) => self.call(name, args),
            Expr::Fn(n, a) => self.fn_call(n, a),
        }
    }

    /// Operand eines Inline-Ausdrucks: Ganzzahlvariable (Label) oder Konstante
    fn inline_opd(&mut self, e: &Expr) -> Option<Result<String, i64>> {
        if let Expr::Var(v) = e {
            if v.ty == Ty::Str || is_reserved_scalar(v) { return None; }
            return Some(Ok(self.scalar(v)));
        }
        self.konst(e).map(Err)
    }

    /// Stil `inline` (nur Ganzzahlen): Variable ± Konstante, Variable ± Variable, Konstante - Variable und Gleichheit als direkte Befehlsfolge, Ergebnis in A/Y.
    /// false = kein einfacher Fall (die Laufzeitroutine übernimmt)
    fn inline_bin(&mut self, op: &BinOp, a: &Expr, b: &Expr) -> bool {
        if self.float { return false; }
        if !matches!(op, BinOp::Add | BinOp::Sub | BinOp::Eq | BinOp::Ne) { return false; }
        let simple = |s: &mut Self, e: &Expr| matches!(e, Expr::Var(v) if v.ty != Ty::Str && !is_reserved_scalar(v)) || (!matches!(e, Expr::Var(_)) && s.konst(e).is_some());
        if !simple(self, a) || !simple(self, b) { return false; }
        let (Some(oa), Some(ob)) = (self.inline_opd(a), self.inline_opd(b)) else { return false };
        let (mut oa, mut ob) = (oa, ob);
        let lohi = |v: i64| (format!("#{}", v & 0xFF), format!("#{}", (v >> 8) & 0xFF));
        match op {
            BinOp::Add | BinOp::Sub => {
                let add = matches!(op, BinOp::Add);
                if add && oa.is_err() && ob.is_ok() { std::mem::swap(&mut oa, &mut ob); }
                let (clc, adc) = if add { ("clc", "adc") } else { ("sec", "sbc") };
                match (&oa, &ob) {
                    (Ok(x), Ok(y)) => { self.ins(format!("lda {x}")); self.ins(clc); self.ins(format!("{adc} {y}")); self.ins("pha"); self.ins(format!("lda {x}+1")); self.ins(format!("{adc} {y}+1")); }
                    (Ok(x), Err(c)) => { let (lo, hi) = lohi(*c); self.ins(format!("lda {x}")); self.ins(clc); self.ins(format!("{adc} {lo}")); self.ins("pha"); self.ins(format!("lda {x}+1")); self.ins(format!("{adc} {hi}")); }
                    (Err(c), Ok(y)) => { let (lo, hi) = lohi(*c); self.ins(format!("lda {lo}")); self.ins("sec"); self.ins(format!("sbc {y}")); self.ins("pha"); self.ins(format!("lda {hi}")); self.ins(format!("sbc {y}+1")); }
                    (Err(_), Err(_)) => return false,
                }
                self.ins("tay"); self.ins("pla");
            }
            _ => {
                let (f, e) = (self.fresh(), self.fresh());
                let eq = matches!(op, BinOp::Eq);
                if oa.is_err() && ob.is_ok() { std::mem::swap(&mut oa, &mut ob); }
                let part = |o: &Result<String, i64>, hi: bool| -> String { match o { Ok(l) => if hi { format!("{l}+1") } else { l.clone() }, Err(c) => { let (lo, h) = lohi(*c); if hi { h } else { lo } } } };
                self.ins(format!("lda {}", part(&oa, false))); self.ins(format!("cmp {}", part(&ob, false))); self.ins(format!("bne {f}"));
                self.ins(format!("lda {}", part(&oa, true))); self.ins(format!("cmp {}", part(&ob, true))); self.ins(format!("bne {f}"));
                let t = if self.atari { ["lda #1", "ldy #0"] } else { ["lda #$FF", "tay"] };
                let (tv, fv) = if eq { (t, ["lda #0", "tay"]) } else { (["lda #0", "tay"], t) };
                for i in tv { self.ins(i); }
                self.ins(format!("jmp {e}"));
                self.label(&f);
                for i in fv { self.ins(i); }
                self.label(&e);
            }
        }
        true
    }

    /// `X = X + c` / `X = X - c` (Ganzzahl, Stil `inline`): direkt in der Variablen rechnen
    fn inline_step(&mut self, target: &LValue, expr: &Expr) -> bool {
        if !self.o.inline || self.float { return false; }
        let (LValue::Var(v), Expr::Bin(op @ (BinOp::Add | BinOp::Sub), a, b)) = (target, expr) else { return false };
        if v.ty == Ty::Str || is_reserved_scalar(v) { return false; }
        let (Expr::Var(w), Some(c)) = (&**a, self.konst(b)) else { return false };
        if w != v || matches!(**b, Expr::Var(_)) { return false; }
        let l = self.scalar(v);
        let (clc, adc) = if matches!(op, BinOp::Add) { ("clc", "adc") } else { ("sec", "sbc") };
        self.ins(format!("lda {l}")); self.ins(clc); self.ins(format!("{adc} #{}", c & 0xFF)); self.ins(format!("sta {l}"));
        self.ins(format!("lda {l}+1")); self.ins(format!("{adc} #{}", (c >> 8) & 0xFF)); self.ins(format!("sta {l}+1"));
        true
    }

    /// rt_ap setzen, ohne A/Y (Ausdruckswert) zu verändern: über X
    fn set_ap_keep(&mut self, label: &str) {
        self.ins(format!("ldx #<{label}"));
        self.ins("stx rt_ap");
        self.ins(format!("ldx #>{label}"));
        self.ins("stx rt_ap+1");
    }

    fn call(&mut self, name: &str, args: &[Expr]) {
        let want = |n: usize| args.len() == n;
        match name {
            "DIV" if want(2) => { self.expr(&args[0]); self.ins("jsr rt_push"); self.expr(&args[1]); self.ins("jsr rt_div"); }
            "DPEEK" if want(1) => { self.uses_turbo = true; self.num_ay(&args[0]); self.ins("jsr rt_dpeek"); }
            "STICK" | "STRIG" if want(1) => { self.uses_gfx = true; self.num_ay(&args[0]); self.ins(if name == "STICK" { "jsr rt_stick" } else { "jsr rt_strig" }); }
            "PEEK" if want(1) => { self.expr(&args[0]); self.ins("jsr rt_peek"); }
            "ABS" if want(1) => { self.expr(&args[0]); self.ins("jsr rt_abs"); }
            "SGN" if want(1) => { self.expr(&args[0]); self.ins("jsr rt_sgn"); }
            "INT" if want(1) => { self.expr(&args[0]); self.warn("I113", "INT ist in Stufe 1 ohne Wirkung (nur Ganzzahlen)"); }
            "PEEK" | "ABS" | "SGN" | "INT" => self.err("E305", format!("{name} erwartet genau ein Argument")),
            "CHR$" | "STR$" | "LEFT$" | "RIGHT$" | "MID$" | "HEX$" | "INKEY$" | "TIME$" => self.stage(3, &format!("{name}")),
            "LEN" | "ASC" | "VAL" | "DEC" | "INSTR" => self.stage(3, name),
            "SQR" | "RND" | "LOG" | "EXP" | "COS" | "SIN" | "TAN" | "ATN" => self.stage(2, name),
            _ => self.err("E306", format!("Funktion {name} wird nicht übersetzt")),
        }
    }

    fn store(&mut self, lv: &LValue) {
        if self.float { return self.fstore(lv); }
        match lv {
            LValue::Var(v) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenvariable"); return; }
                if is_reserved_scalar(v) { self.err("E307", format!("{} kann nicht zugewiesen werden", v.name)); return; }
                let l = self.scalar(v);
                self.ins(format!("sta {l}"));
                self.ins(format!("sty {l}+1"));
            }
            LValue::Elem(v, idx) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenfeld"); return; }
                if idx.len() != 1 { self.err("E304", "Stufe 1 kennt nur eindimensionale Felder"); return; }
                self.ins("jsr rt_push");
                self.expr(&idx[0]);
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_stelem");
            }
        }
    }

    fn intern(&mut self, s: &[u8]) -> String {
        let i = self.strings.iter().position(|x| x == s).unwrap_or_else(|| { self.strings.push(s.to_vec()); self.strings.len() - 1 });
        format!("str_{}", i + 1)
    }

    fn target(&mut self, n: u32) -> String {
        if !self.lines.contains(&n) { self.err("E201", format!("Sprungziel: Zeile {n} existiert nicht")); }
        format!("L{n}")
    }

    fn numeric_lvalue_ok(&mut self, lv: &LValue) -> bool {
        let ty = match lv { LValue::Var(v) | LValue::Elem(v, _) => v.ty };
        if ty == Ty::Str && !self.str { self.stage(3, "Zeichenketten-Variable"); false } else { true }
    }

    fn stmts(&mut self, list: &[Stmt], next: &str) {
        for s in list { self.stmt(s, next); }
    }

    fn stmt(&mut self, s: &Stmt, next: &str) {
        match s {
            Stmt::Rem(_) => {}
            Stmt::Let { target, expr } => {
                if self.inline_step(target, expr) { return; }
                if self.str {
                    let (ts, es) = (lv_is_str(target), self.is_str(expr));
                    if ts != es { self.err("E320", "Typen passen nicht (TYPE MISMATCH): Zeichenkette und Zahl"); return; }
                    if ts { self.sexpr(expr); self.sstore(target); return; }
                }
                if let LValue::Var(v) = target { if v.ty == Ty::Str { self.stage(3, "Zeichenkettenzuweisung"); return; } }
                self.expr(expr);
                self.store(target);
            }
            Stmt::Print { items } => self.print(items),
            Stmt::Turbo(t) => self.turbo_stmt(t),
            Stmt::Atari { op, args } if op == "OPEN" => {
                self.uses_gfx = true; self.uses_open = true;
                if args.len() != 4 { self.err("E305", "OPEN erwartet #Kanal, Zahl, Zahl, Name"); return; }
                for a in &args[..3] { self.num_ay(a); self.ins("jsr rt_push"); }
                if !self.is_str(&args[3]) { self.err("E320", "OPEN: der Dateiname muss eine Zeichenkette sein (TYPE MISMATCH)"); return; }
                self.sexpr(&args[3]);
                self.ins("jsr rt_open");
            }
            Stmt::Atari { op, args } if matches!(op.as_str(), "CLOSE" | "PUT" | "BPUT" | "BGET") => {
                self.uses_gfx = true;
                let n = args.len();
                for (i, a) in args.iter().enumerate() { self.num_ay(a); if i + 1 < n { self.ins("jsr rt_push"); } }
                self.ins(format!("jsr rt_{}", op.to_ascii_lowercase()));
            }
            Stmt::Atari { op, args } if op == "CLS" => { self.ins("lda #125"); self.ins("jsr CHROUT"); let _ = args; }
            Stmt::Atari { op, args } => {
                let n = args.len();
                for (i, a) in args.iter().enumerate() {
                    self.num_ay(a);
                    if i + 1 < n { self.ins("jsr rt_push"); }
                }
                let rt = match op.as_str() { "GRAPHICS" => "rt_gr", "COLOR" => "rt_color", "PLOT" => "rt_plot", "DRAWTO" => "rt_drawto", "POSITION" => "rt_pos", "SETCOLOR" => "rt_setcolor", "DPOKE" => "rt_dpoke", "PAUSE" => "rt_pause", "MOVE" => "rt_move", "-MOVE" => "rt_movd", "FCOLOR" => "rt_fcolor", "FILLTO" => "rt_fillto", _ => "rt_sound" };
                if matches!(op.as_str(), "DPOKE" | "PAUSE" | "MOVE" | "-MOVE") { self.uses_turbo = true; } else { self.uses_gfx = true; }
                self.ins(format!("jsr {rt}"));
            }
            Stmt::Locate(x, y, lv) => {
                self.uses_gfx = true;
                if !self.numeric_lvalue_ok(lv) { return; }
                self.num_ay(x); self.ins("jsr rt_push"); self.num_ay(y);
                self.ins("jsr rt_locate");
                self.ins("ldy #0");
                if self.float { self.ins("jsr rt_i2f"); }
                self.store(lv);
            }
            Stmt::GetCh(chan, lv) => {
                self.uses_gfx = true;
                if !self.numeric_lvalue_ok(lv) { return; }
                self.num_ay(chan);
                self.ins("jsr rt_getc");
                self.ins("ldy #0");
                if self.float { self.ins("jsr rt_i2f"); }
                self.store(lv);
            }
            Stmt::PrintTo { chan, items } => {
                self.uses_gfx = true;
                self.num_ay(chan);
                for _ in 0..4 { self.ins("asl"); }
                self.ins("sta rt_chan");
                self.print(items);
                self.ins("lda #0"); self.ins("sta rt_chan");
            }
            Stmt::Input { prompt, vars } => {
                self.uses_io = true;
                if vars.len() != 1 { self.err("E308", "INPUT mit mehreren Variablen wird in Stufe 1 nicht übersetzt (je Variable ein INPUT)"); return; }
                if self.str && lv_is_str(&vars[0]) {
                    if let Some(p) = prompt { let l = self.intern(p); self.ins(format!("lda #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr rt_print_str"); }
                    self.ins("jsr rt_input_s");
                    self.sstore(&vars[0]);
                    return;
                }
                if !self.numeric_lvalue_ok(&vars[0]) { return; }
                if let Some(p) = prompt { let l = self.intern(p); self.ins(format!("lda #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr rt_print_str"); }
                self.ins(if self.float { "jsr rt_input_f" } else { "jsr rt_input_int" });
                let skip = self.fresh();
                self.ins(format!("bcs {skip}"));
                self.store(&vars[0]);
                self.label(&skip);
            }
            Stmt::Get(lv) => {
                self.uses_io = true;
                if self.str && lv_is_str(lv) { if self.atari { self.err("E311", "GET in eine Zeichenkette gibt es in Atari-BASIC nicht (GET #n,A liest ein Byte in eine Zahl)"); return; } self.ins("jsr rt_get_s"); self.sstore(lv); return; }
                if !self.numeric_lvalue_ok(lv) { return; }
                self.ins("jsr rt_get_int");
                if self.float { self.ins("jsr rt_i2f"); }
                self.store(lv);
            }
            Stmt::Goto(n) => { let t = self.target(*n); self.ins(format!("jmp {t}")); }
            Stmt::Gosub(n) => {
                self.uses_flow = true;
                let t = self.target(*n);
                let ret = self.fresh();
                self.set_ptr("rt_bp", &ret);
                self.ins("jsr rt_gosub");
                self.ins(format!("jmp {t}"));
                self.label(&ret);
            }
            Stmt::Return => { self.uses_flow = true; self.ins("jmp rt_return"); }
            Stmt::OnGoto { sel, targets } | Stmt::OnGosub { sel, targets } => {
                let gosub = matches!(s, Stmt::OnGosub { .. });
                let end = self.fresh();
                if gosub { self.uses_flow = true; self.set_ptr("rt_bp", &end); }
                self.expr(sel);
                if self.float { self.ins("jsr rt_f2addr"); }
                self.ins("jsr rt_byte");
                for (i, t) in targets.iter().enumerate() {
                    let tl = self.target(*t);
                    let nxt = self.fresh();
                    self.ins(format!("cmp #{}", i + 1));
                    self.ins(format!("bne {nxt}"));
                    if gosub { self.ins("jsr rt_gosub"); }
                    self.ins(format!("jmp {tl}"));
                    self.label(&nxt);
                }
                self.label(&end);
            }
            Stmt::If { cond, then } => {
                self.expr(cond);
                if self.float {
                    self.ins("lda F_FACEXP         ; Exponent des FAC: 0 = Zahl ist 0");
                } else {
                    self.ins("sty rt_tmp");
                    self.ins("ora rt_tmp");
                }
                self.ins(format!("beq {next}"));
                self.stmts(then, next);
            }
            Stmt::For { var, from, to, step } => {
                self.uses_flow = true;
                if var.ty == Ty::Str { self.stage(3, "FOR mit Zeichenkettenvariable"); return; }
                if var.ty == Ty::Int { self.err("E314", format!("FOR-Variable {}% darf nicht ganzzahlig sein (im BASIC V2 ein SYNTAX ERROR)", var.name)); return; }
                let v = self.scalar(var);
                self.expr(from);
                if self.float {
                    self.ins(format!("ldx #<{v}"));
                    self.ins(format!("ldy #>{v}"));
                    self.ins("jsr F_MOVMF");
                } else {
                    self.ins(format!("sta {v}"));
                    self.ins(format!("sty {v}+1"));
                }
                self.expr(to);
                self.ins(if self.float { "jsr rt_fpush" } else { "jsr rt_push" });
                match step {
                    Some(e) => self.expr(e),
                    None if self.float => { self.ins("lda #<rt_f1"); self.ins("ldy #>rt_f1"); self.ins("jsr rt_movfm"); }
                    None => { self.ins("lda #1"); self.ins("ldy #0"); }
                }
                self.ins(if self.float { "jsr rt_fpush" } else { "jsr rt_push" });
                self.set_ptr("rt_fv", &v);
                let body = self.fresh();
                self.set_ptr("rt_bp", &body);
                self.ins("jsr rt_for");
                self.label(&body);
            }
            Stmt::Next(vars) => {
                self.uses_flow = true;
                if vars.is_empty() {
                    self.ins("lda #0"); self.ins("sta rt_fv"); self.ins("sta rt_fv+1"); self.ins("jsr rt_next");
                }
                for var in vars {
                    let v = self.scalar(var);
                    self.set_ptr("rt_fv", &v);
                    self.ins("jsr rt_next");
                }
            }
            Stmt::Dim(list) => {   // Größen werden vor der Übersetzung gesammelt, das Feld liegt statisch im Datenbereich
                if self.atari {   // Zeichenketten: Kapazität im Kopf setzen (zweites DIM: Fehler 9)
                    for (v, dims) in list {
                        if v.ty != Ty::Str { continue; }
                        let Some(n) = dims.first().and_then(|d| self.konst(d)) else { continue };
                        let l = self.scalar(v);
                        self.ins(format!("lda #<{n}")); self.ins("sta rt_t0"); self.ins(format!("lda #>{n}")); self.ins("sta rt_t0+1");
                        self.ins(format!("lda #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr rt_adim");
                    }
                }
            }
            Stmt::Read(vars) => {
                self.uses_data = true;
                for lv in vars {
                    if self.str && lv_is_str(lv) { self.ins("jsr rt_reads"); self.sstore(lv); continue; }
                    if !self.numeric_lvalue_ok(lv) { continue; }
                    self.ins(if self.str { "jsr rt_readn" } else { "jsr rt_read" });
                    self.store(lv);
                }
            }
            Stmt::Data(_) => {}
            Stmt::Restore(t) => {
                self.uses_data = true;
                let l = match t {
                    None => "b2a_data".to_string(),
                    Some(n) => match self.data.iter().find(|(ln, _)| ln >= n) { Some((ln, _)) => format!("D{ln}"), None => "b2a_data_end".into() },
                };
                self.set_ptr("rt_dp", &l);
            }
            Stmt::Poke(a, v) => {
                match self.konst_addr(a) {
                    Some(addr) => {
                        match self.konst(v) { Some(c) if (0..=255).contains(&c) => self.ins(format!("lda #{c}")), Some(_) => { self.err("E309", "POKE-Wert muss 0..255 sein"); return; } None => { self.expr(v); if self.float { self.ins("jsr rt_f2addr"); } self.ins("jsr rt_byte"); } }
                        self.ins(format!("sta {}", if addr < 256 { format!("${addr:02X}") } else { format!("${addr:04X}") }));
                    }
                    None => {
                        self.expr(a);
                        if self.float {
                            self.ins("jsr rt_fpush");
                            self.expr(v);
                            self.ins("jsr rt_fpoke");
                        } else {
                            self.ins("jsr rt_push");
                            self.expr(v);
                            self.ins("jsr rt_poke");
                        }
                    }
                }
            }
            Stmt::Sys(e) => {
                match self.konst_addr(e) {
                    Some(addr) => self.ins(format!("jsr ${addr:04X}")),
                    None => { self.expr(e); if self.float { self.ins("jsr rt_f2addr"); } self.ins("jsr rt_sys"); }
                }
            }
            Stmt::Wait(a, m, x) => {
                let push = if self.float { "jsr rt_fpush" } else { "jsr rt_push" };
                self.expr(a); self.ins(push);
                self.expr(m); self.ins(push);
                match x {
                    Some(e) => self.expr(e),
                    None if self.float => { self.ins("lda #<rt_f0"); self.ins("ldy #>rt_f0"); self.ins("jsr rt_movfm"); }
                    None => { self.ins("lda #0"); self.ins("ldy #0"); }
                }
                self.ins(if self.float { "jsr rt_fwait" } else { "jsr rt_wait" });
            }
            Stmt::End => self.ins("jmp rt_end"),
            Stmt::Stop => { self.load_imm(self.line_no as i64); self.ins("jmp rt_stop"); }
            Stmt::Clr => { self.uses_clr = true; self.ins("jsr b2a_clr"); }
            Stmt::Run(None) => { self.uses_clr = true; self.uses_run = true; self.ins("jsr b2a_clr"); self.ins("jmp b2a_run"); }
            Stmt::Run(Some(_)) => self.err("E310", "RUN mit Zeilennummer wird nicht übersetzt"),
            Stmt::DefFn { name, .. } => { let l = format!("fd_{}", self.fn_label(name)); self.ins("lda #1"); self.ins(format!("sta {l}")); }
            Stmt::Unsupported(k) => self.err("E311", format!("{k} wird nicht übersetzt (Dateien, LIST/NEW/CONT und DEF FN sind nicht Teil der Umsetzung)")),
        }
    }

    /// Literaladresse 0..65535 für POKE/SYS
    fn konst_addr(&self, e: &Expr) -> Option<u32> {
        match e { Expr::Num(n) if (0..=65535).contains(n) => Some(*n as u32), _ => None }
    }

    fn print(&mut self, items: &[PrintItem]) {
        for it in items {
            match it {
                PrintItem::Semi => {}
                PrintItem::Comma => self.ins("jsr rt_comma"),
                PrintItem::Tab(e) => { self.expr(e); if self.float { self.ins("jsr rt_f2addr"); } self.ins("jsr rt_tab"); }
                PrintItem::Spc(e) => { self.expr(e); if self.float { self.ins("jsr rt_f2addr"); } self.ins("jsr rt_spc"); }
                PrintItem::Expr(Expr::Str(b)) => {
                    if b.is_empty() { continue; }
                    let l = self.intern(b);
                    self.ins(format!("lda #<{l}"));
                    self.ins(format!("ldy #>{l}"));
                    self.ins("jsr rt_print_str");
                }
                PrintItem::Expr(Expr::Call(n, a)) if n == "CHR$" && a.len() == 1 => { self.expr(&a[0]); if self.float { self.ins("jsr rt_f2addr"); } self.ins("jsr rt_print_chr"); }
                PrintItem::Expr(e) if self.str && self.is_str(e) => { self.sexpr(e); self.ins("jsr rt_sprint"); }
                PrintItem::Expr(e) => {
                    self.uses_io = true;
                    self.expr(e);
                    self.ins(if self.float { "jsr rt_print_f" } else { "jsr rt_print_int" });
                }
            }
        }
        if !matches!(items.last(), Some(PrintItem::Semi) | Some(PrintItem::Comma)) { self.ins("jsr rt_print_nl"); }
    }
}

/// Zeichenkette als `.petscii "…"`; Zeichen ohne Textform erscheinen als `{$xx}`
fn petscii_text(bytes: &[u8], d: &Dialect) -> String {
    bytes.iter().map(|b| petscii::to_text(*b, d)).collect()
}

/// Dezimalzahl → 5-Byte-Gleitkommaformat des BASIC (Exponent + 128, 4 Byte Mantisse, Vorzeichen im obersten Mantissenbit)
pub fn mbf(v: f64) -> Result<[u8; 5], String> {
    if v == 0.0 { return Ok([0; 5]); }
    let neg = v < 0.0;
    let a = v.abs();
    let mut e = a.log2().floor() as i32 + 1;
    let mut m = a / 2f64.powi(e);
    while m >= 1.0 { m /= 2.0; e += 1; }
    while m < 0.5 { m *= 2.0; e -= 1; }
    let mut mant = (m * 4294967296.0).round() as u64;
    if mant >= 1 << 32 { mant >>= 1; e += 1; }
    let exp = e + 128;
    if exp > 255 { return Err(format!("Zahl {v} ist zu groß für das BASIC-Zahlenformat")); }
    if exp < 1 { return Ok([0; 5]); }
    Ok([exp as u8, ((mant >> 24) & 0x7F) as u8 | if neg { 0x80 } else { 0 }, (mant >> 16) as u8, (mant >> 8) as u8, mant as u8])
}

/// Zahl als 6-Byte-BCD des Atari-OS: Byte 0 = Vorzeichen (Bit 7) und Exponent zur Basis 100 (Excess $40), Byte 1-5 = Ziffernpaare; 1.0 = 40 01 00 00 00 00
pub fn bcd(v: f64) -> Result<[u8; 6], String> {
    if v == 0.0 { return Ok([0; 6]); }
    let neg = v < 0.0;
    let a = v.abs();
    let digits_of = |s: &str| -> (Vec<u8>, i32) {
        let (m, x) = s.split_once('e').unwrap();
        (m.bytes().filter(|c| *c != b'.').map(|c| c - b'0').collect(), x.parse().unwrap())
    };
    let first = format!("{:.9e}", a);
    let x0: i32 = digits_of(&first).1;
    let (mut digits, mut x) = digits_of(&format!("{:.*e}", if x0.rem_euclid(2) == 0 { 8 } else { 9 }, a));
    if x.rem_euclid(2) != x0.rem_euclid(2) {   // Aufrunden auf eine Zehnerpotenz (z. B. 9,99999999995)
        let n = if x.rem_euclid(2) == 1 { 10 } else { 9 };
        digits = std::iter::once(1).chain(std::iter::repeat(0).take(n - 1)).collect();
    }
    if x.rem_euclid(2) == 0 { digits.insert(0, 0); } else { x -= 1; }
    let e = x.div_euclid(2);
    let exp = e + 0x40;
    if exp > 0x70 { return Err(format!("Zahl {v} ist zu groß für das BASIC-Zahlenformat (höchstens 9,99999999E+97)")); }   // Atari: 1E-98 bis 9,99999999E+97
    if exp < 0x0F { return Ok([0; 6]); }
    let mut out = [0u8; 6];
    out[0] = exp as u8 | if neg { 0x80 } else { 0 };
    for i in 0..5 { out[1 + i] = digits[2 * i] << 4 | digits[2 * i + 1]; }
    Ok(out)
}

/// Roher DATA-Text als Zahl (Leerzeichen am Rand erlaubt)
fn raw_num(b: &[u8]) -> Option<f64> {
    let t = String::from_utf8_lossy(b).trim().to_string();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'E' | 'e')) || !t.chars().any(|c| c.is_ascii_digit()) { return None; }
    t.parse::<f64>().ok()
}

/// Braucht das Programm Zeichenketten? (Zeichenkettenvariablen, Zeichenkettenfunktionen, Texte außerhalb von PRINT)
fn needs_str(prog: &Program) -> bool {
    const FN: &[&str] = &["LEN", "ASC", "VAL", "STR$", "LEFT$", "RIGHT$", "MID$", "CHR$", "HEX$", "DEC", "INSTR", "INKEY$", "TIME$"];
    fn ex(e: &Expr) -> bool {
        match e {
            Expr::Str(_) => true,
            Expr::Var(v) => v.ty == Ty::Str,
            Expr::Elem(v, a) => v.ty == Ty::Str || a.iter().any(ex),
            Expr::Un(_, x) => ex(x),
            Expr::Bin(_, a, b) => ex(a) || ex(b),
            Expr::Call(n, a) => FN.contains(&n.as_str()) || a.iter().any(ex),
            Expr::Fn(_, x) => ex(x),
            Expr::Num(_) | Expr::Float(_) => false,
        }
    }
    fn lv(l: &LValue) -> bool { match l { LValue::Var(v) => v.ty == Ty::Str, LValue::Elem(v, a) => v.ty == Ty::Str || a.iter().any(ex) } }
    fn st(s: &Stmt) -> bool {
        match s {
            Stmt::Let { target, expr } => lv(target) || ex(expr),
            // PRINT "text" und PRINT CHR$(n) brauchen keine Zeichenketten-Laufzeit
            Stmt::Print { items } => items.iter().any(|i| match i {
                PrintItem::Expr(Expr::Str(_)) => false,
                PrintItem::Expr(Expr::Call(n, a)) if n == "CHR$" && a.len() == 1 => ex(&a[0]),
                PrintItem::Expr(e) | PrintItem::Tab(e) | PrintItem::Spc(e) => ex(e),
                _ => false,
            }),
            Stmt::Turbo(Struct::If(c) | Struct::While(c) | Struct::Until(c) | Struct::OnExec(c, _)) => ex(c),
            Stmt::PrintTo { chan, items } => ex(chan) || items.iter().any(|i| match i {
                PrintItem::Expr(Expr::Str(_)) => false,
                PrintItem::Expr(e) | PrintItem::Tab(e) | PrintItem::Spc(e) => ex(e),
                _ => false,
            }),
            Stmt::Atari { args, .. } => args.iter().any(ex),
            Stmt::Locate(x, y, l) => ex(x) || ex(y) || lv(l),
            Stmt::GetCh(c, l) => ex(c) || lv(l),
            Stmt::Input { vars, .. } | Stmt::Read(vars) => vars.iter().any(lv),
            Stmt::Get(v) => lv(v),
            Stmt::OnGoto { sel, .. } | Stmt::OnGosub { sel, .. } => ex(sel),
            Stmt::If { cond, then } => ex(cond) || then.iter().any(st),
            Stmt::For { var, from, to, step } => var.ty == Ty::Str || ex(from) || ex(to) || step.as_ref().map_or(false, ex),
            Stmt::Dim(l) => l.iter().any(|(v, d)| v.ty == Ty::Str || d.iter().any(ex)),
            Stmt::Data(items) => items.iter().any(|i| matches!(i, DataItem::Str(_)) || matches!(i, DataItem::Raw(b) if raw_num(b).is_none())),   // Atari: Text im DATA (auch bei READ in Zahlen: Fehler 8 zur Laufzeit)
            Stmt::Poke(a, b) | Stmt::Wait(a, b, None) => ex(a) || ex(b),
            Stmt::Wait(a, b, Some(c)) => ex(a) || ex(b) || ex(c),
            Stmt::Sys(e) => ex(e),
            Stmt::DefFn { body, param, .. } => param.ty == Ty::Str || ex(body),
            _ => false,
        }
    }
    prog.lines.iter().any(|l| l.stmts.iter().any(st))
}

/// Braucht das Programm Fließkomma? (Fließkommazahlen, `/` ohne glatte Teilung, `^`, SQR/RND/LOG/EXP/SIN/COS/TAN/ATN, große Zahlen)
fn needs_float(prog: &Program, o: &B2aOptions) -> bool {
    fn ex(e: &Expr, o: &B2aOptions) -> bool {
        match e {
            Expr::Float(_) => true,
            Expr::Num(n) => *n > 65535,
            Expr::Str(_) | Expr::Var(_) => false,
            Expr::Elem(_, a) => a.iter().any(|x| ex(x, o)),
            Expr::Un(_, x) => ex(x, o),
            Expr::Bin(BinOp::Pow, _, _) => true,
            Expr::Bin(BinOp::Div, a, b) => {
                if o.int_div { return ex(a, o) || ex(b, o); }
                match (&**a, &**b) { (Expr::Num(x), Expr::Num(y)) if *y != 0 && x % y == 0 => false, _ => true }
            }
            Expr::Bin(_, a, b) => ex(a, o) || ex(b, o),
            Expr::Call(n, a) => matches!(n.as_str(), "SQR" | "RND" | "LOG" | "EXP" | "SIN" | "COS" | "TAN" | "ATN" | "DPEEK") || a.iter().any(|x| ex(x, o)),
            Expr::Fn(_, x) => ex(x, o),
        }
    }
    fn lv(l: &LValue, o: &B2aOptions) -> bool { match l { LValue::Var(_) => false, LValue::Elem(_, a) => a.iter().any(|x| ex(x, o)) } }
    fn st(s: &Stmt, o: &B2aOptions) -> bool {
        match s {
            Stmt::Let { target, expr } => lv(target, o) || ex(expr, o),
            Stmt::Print { items } => items.iter().any(|i| match i { PrintItem::Expr(e) | PrintItem::Tab(e) | PrintItem::Spc(e) => ex(e, o), _ => false }),
            Stmt::Turbo(Struct::If(c) | Struct::While(c) | Struct::Until(c) | Struct::OnExec(c, _)) => ex(c, o),
            Stmt::PrintTo { chan, items } => ex(chan, o) || items.iter().any(|i| match i { PrintItem::Expr(e) | PrintItem::Tab(e) | PrintItem::Spc(e) => ex(e, o), _ => false }),
            Stmt::Atari { args, .. } => args.iter().any(|a| ex(a, o)),
            Stmt::Locate(x, y, l) => ex(x, o) || ex(y, o) || lv(l, o),
            Stmt::GetCh(c, l) => ex(c, o) || lv(l, o),
            Stmt::Input { vars, .. } | Stmt::Read(vars) => vars.iter().any(|v| lv(v, o)),
            Stmt::Get(v) => lv(v, o),
            Stmt::OnGoto { sel, .. } | Stmt::OnGosub { sel, .. } => ex(sel, o),
            Stmt::If { cond, then } => ex(cond, o) || then.iter().any(|s| st(s, o)),
            Stmt::For { from, to, step, .. } => ex(from, o) || ex(to, o) || step.as_ref().map_or(false, |e| ex(e, o)),
            Stmt::Dim(l) => l.iter().any(|(_, d)| d.iter().any(|x| ex(x, o))),
            Stmt::Data(items) => items.iter().any(|i| matches!(i, DataItem::Float(_)) || matches!(i, DataItem::Num(n) if *n > 32767 || *n < -32768) || matches!(i, DataItem::Raw(b) if raw_num(b).map_or(false, |x| x.fract() != 0.0 || x.abs() > 32767.0 || b.iter().any(|c| matches!(c, b'.' | b'E' | b'e'))))),
            Stmt::Poke(a, b) | Stmt::Wait(a, b, None) => ex(a, o) || ex(b, o),
            Stmt::Wait(a, b, Some(c)) => ex(a, o) || ex(b, o) || ex(c, o),
            Stmt::Sys(e) => ex(e, o),
            Stmt::DefFn { body, .. } => ex(body, o),
            _ => false,
        }
    }
    prog.lines.iter().any(|l| l.stmts.iter().any(|s| st(s, o)))
}

/// Zielsysteme von basic2asm: Ladeadresse, CPU und BASIC-Dialekt
/// `dialect` ist der Dialekt, nach dem der Parser die Quelle liest (Atari-Programme liest er als CBM-V2-Teilmenge, siehe `atari_check`); `sys` der Ordner unter `data/sys/`
pub struct TargetInfo { pub id: &'static str, pub org: u16, pub cpu: &'static str, pub dialect: &'static str, pub float: bool, pub atari: bool, pub sys: &'static str }

pub const TARGETS: &[TargetInfo] = &[
    TargetInfo { id: "c64", org: 0x0801, cpu: "6510", dialect: "cbm_v2", float: true, atari: false, sys: "c64" },
    TargetInfo { id: "c128", org: 0x1C01, cpu: "8502", dialect: "basic7", float: true, atari: false, sys: "c128" },
    TargetInfo { id: "plus4", org: 0x1001, cpu: "7501", dialect: "basic35", float: true, atari: false, sys: "plus4" },
    TargetInfo { id: "mega65", org: 0x2001, cpu: "45gs02", dialect: "basic65", float: false, atari: false, sys: "mega65" },
    TargetInfo { id: "atari8", org: 0x2000, cpu: "sally", dialect: "cbm_v2", float: true, atari: true, sys: "atari8bit" },
];

pub fn target_info(id: &str) -> Option<&'static TargetInfo> { TARGETS.iter().find(|t| t.id == id) }

pub fn generate(prog: &Program, d: &Dialect, labels: &[(String, u32)], comments: &[(usize, String)], o: &B2aOptions) -> Result<Converted, Vec<Diagnostic>> {
    let Some(ti) = target_info(&o.target) else { return Err(vec![Diagnostic::error("E330", &o.file, 0, format!("Ziel '{}' ist nicht umgesetzt (verfügbar: {})", o.target, TARGETS.iter().map(|t| t.id).collect::<Vec<_>>().join(", ")))]) };
    let t = ti.id;
    let wants_float = o.force_float || needs_float(prog, o) || (ti.atari && o.stage >= 3 && needs_str(prog));   // Atari: Zeichenketten rechnen immer mit dem Fließkommapaket (VAL, STR$, DATA)
    if wants_float && !ti.float && o.stage >= 2 {
        return Err(vec![Diagnostic::error("E331", &o.file, 0, format!("Fließkomma (Stufe 2) ist für das Ziel {t} noch nicht umgesetzt (verfügbar: {}); das Programm braucht es", TARGETS.iter().filter(|x| x.float).map(|x| x.id).collect::<Vec<_>>().join(", ")))]);
    }
    let mut g = G {
        o, d, errs: vec![], warns: vec![], src_line: 0, line_no: 0, lines: prog.lines.iter().map(|l| l.number).collect(),
        scalars: BTreeMap::new(), float: o.stage >= 2 && ti.float && wants_float, atari: ti.atari, fsz: if ti.atari { 6 } else { 5 }, str: o.stage >= 3 && needs_str(prog), fconsts: vec![], fdata: vec![], sdata: vec![], adata: vec![], scap: BTreeMap::new(), fns: BTreeMap::new(), subs: String::new(),
        arrays: BTreeMap::new(), strings: vec![], data: vec![], uniq: 0, need_stage: 1,
        hint_float: false, hint_big: false, uses_flow: false, uses_io: false, uses_data: false, uses_clr: false, uses_gfx: false, uses_open: false, uses_turbo: false, ctl: vec![], procs: BTreeSet::new(), uses_run: false, body: String::new(),
    };
    if o.stage == 0 || o.stage > 3 { g.errs.push(Diagnostic::error("E300", &o.file, 0, format!("Stufe {} ist nicht umgesetzt; verfügbar sind Stufe 1 (Ganzzahl), 2 (Fließkomma) und 3 (Zeichenketten)", o.stage))); }
    if g.float { g.need_stage = 2; }
    if g.str { g.need_stage = 3; }

    // Vorlauf: DIM-Größen und DATA-Tabelle
    for l in &prog.lines {
        g.src_line = l.src_line;
        g.line_no = l.number;
        let mut all: Vec<&Stmt> = vec![];
        fn flat<'x>(v: &'x [Stmt], out: &mut Vec<&'x Stmt>) { for s in v { out.push(s); if let Stmt::If { then, .. } = s { flat(then, out); } } }
        flat(&l.stmts, &mut all);
        for s in all {
            match s {
                Stmt::Dim(list) => for (v, dims) in list {
                    if v.ty == Ty::Str && !g.str { g.stage(3, "Zeichenkettenfeld"); continue; }
                    if g.atari && v.ty == Ty::Str {   // Atari: DIM A$(n) = Kapazität einer Zeichenkette (kein Feld)
                        if dims.len() != 1 { g.err("E304", "DIM einer Zeichenkette braucht genau eine Größe: DIM A$(n)"); continue; }
                        match g.konst(&dims[0]) {
                            Some(n) if (1..=32767).contains(&n) => { let l = label_of("v", v); let e = g.scap.entry(l).or_insert(0); *e = (*e).max(n); }
                            Some(_) => g.err("E312", "DIM A$(n): n muss 1 bis 32767 sein (Atari-BASIC: sonst Fehler 9)"),
                            None => g.err("E312", "DIM A$(n) braucht in basic2asm eine Zahl als Größe"),
                        }
                        continue;
                    }
                    if dims.len() != 1 { g.err("E304", "Es sind nur eindimensionale Felder umgesetzt"); continue; }
                    let n = match g.konst(&dims[0]) { Some(n) if n >= 0 => n, _ => { g.err("E312", "DIM braucht in Stufe 1 eine Zahl als Größe"); continue; } };
                    let label = g.array(v);
                    let a = g.arrays.get_mut(&key_of(v)).unwrap();
                    if a.dimmed { let name = a.name.clone(); g.err("E313", format!("Feld {name} wird zweimal dimensioniert (REDIM'D ARRAY)")); continue; }
                    a.dimmed = true;
                    a.count = n + 1;
                    let _ = label;
                }
                Stmt::DefFn { name, param, body } => {
                    let key = fn_key(name);
                    if name.ends_with('$') || param.ty == Ty::Str { g.err("E315", format!("FN{name}: Zeichenkettenfunktionen und -parameter werden nicht übersetzt")); continue; }
                    if g.fns.contains_key(&key) { g.err("E316", format!("FN{name} wird zweimal definiert")); continue; }
                    let label = format!("{}{}", name.trim_end_matches(['%', '$']), if name.ends_with('%') { "_P" } else { "" });
                    g.fns.insert(key, FnInfo { label, param: param.clone(), body: body.clone(), line: l.src_line, number: l.number, ret_int: name.ends_with('%') });
                }
                Stmt::Data(items) => {
                    let mut nums = vec![];
                    let mut fl: Vec<Vec<u8>> = vec![];
                    let conv: Vec<DataItem>;
                    let items = if g.atari && g.str {   // Atari mit Zeichenketten: alle Einträge als Text, jeder wird beim Lesen zur Zahl oder Zeichenkette
                        g.adata.push(items.iter().map(|i| if let DataItem::Raw(b) = i { b.clone() } else { vec![] }).collect());
                        g.data.push((l.number, nums));
                        g.uses_data = true;
                        continue;
                    } else if g.atari {   // Atari ohne Zeichenketten: Einträge müssen Zahlen sein
                        let mut v = vec![];
                        for i in items {
                            let DataItem::Raw(b) = i else { continue };
                            match raw_num(b) {
                                Some(x) if x.fract() == 0.0 && x.abs() < 1e15 && !b.iter().any(|c| matches!(c, b'.' | b'E' | b'e')) => v.push(DataItem::Num(x as i64)),
                                Some(_) => v.push(DataItem::Float(String::from_utf8_lossy(b).trim().to_string())),
                                None => g.err("E322", format!("DATA-Eintrag '{}' ist keine Zahl (Atari-BASIC: Fehler 8 beim Lesen)", String::from_utf8_lossy(b))),
                            }
                        }
                        conv = v;
                        &conv
                    } else { items };
                    if g.str {
                        let mut v = vec![];
                        for it in items {
                            let (text, num) = match it {
                                DataItem::Num(n) => (n.to_string().into_bytes(), Some(*n as f64)),
                                DataItem::Float(t) => (t.clone().into_bytes(), t.parse::<f64>().ok()),
                                DataItem::Str(b) | DataItem::Raw(b) => (b.clone(), None),
                            };
                            let label = if text.is_empty() { None } else { Some(g.intern(&text)) };
                            let item = SItem { text, label, num };
                            if let Some(x) = item.num { if g.float { if let Err(e) = g.fbytes(x) { g.err("E303", e); } } else if !(-32768.0..=65535.0).contains(&x) { g.err("E303", format!("DATA-Zahl {x} ist zu groß für 16 Bit")); } }
                            v.push(item);
                        }
                        g.data.push((l.number, nums));
                        g.sdata.push(v);
                        g.uses_data = true;
                        continue;
                    }
                    if g.float {
                        for it in items {
                            let v = match it { DataItem::Num(n) => Some(*n as f64), DataItem::Float(s) => s.parse::<f64>().ok(), DataItem::Str(_) | DataItem::Raw(_) => { g.stage(3, "DATA mit Text"); None } };
                            if let Some(v) = v { match g.fbytes(v) { Ok(b) => fl.push(b), Err(e) => g.err("E303", e) } }
                        }
                        g.data.push((l.number, nums));
                        g.fdata.push(fl);
                        g.uses_data = true;
                        continue;
                    }
                    for it in items { match it { DataItem::Num(n) if *n <= 65535 && *n >= -32768 => nums.push(*n), DataItem::Num(n) => g.err("E303", format!("DATA-Zahl {n} ist zu groß für 16 Bit")), DataItem::Float(s) => g.stage(2, &format!("DATA-Zahl {s}")), DataItem::Str(_) | DataItem::Raw(_) => g.stage(3, "DATA mit Text") } }
                    g.data.push((l.number, nums));
                    g.uses_data = true;
                }
                _ => {}
            }
        }
    }

    // Turbo BASIC XL: Prozedurnamen sammeln
    for l in &prog.lines {
        for s in &l.stmts {
            if let Stmt::Turbo(Struct::Proc(n)) = s {
                g.src_line = l.src_line;
                if !g.procs.insert(n.clone()) { g.err("E340", format!("Prozedur {n} wird zweimal definiert")); }
            }
        }
    }

    // Programmteil
    let mut map = vec![];
    let line_nums: Vec<u32> = prog.lines.iter().map(|l| l.number).collect();
    let mut ci = 0usize;
    let mut blank_pending = false;
    for (i, l) in prog.lines.iter().enumerate() {
        g.src_line = l.src_line;
        g.line_no = l.number;
        g.uniq = 0;
        // Kommentar- und Leerzeilen des Originals vor dieser Zeile übernehmen (eine Leerzeile höchstens, keine am Anfang)
        while ci < comments.len() && comments[ci].0 < l.src_line {
            let t = &comments[ci].1;
            if !o.no_rem {
                if t.is_empty() { blank_pending = !g.body.is_empty(); }
                else { if blank_pending { g.body.push('\n'); blank_pending = false; } let _ = writeln!(g.body, ";{}", t); }
            }
            ci += 1;
        }
        let next = match line_nums.get(i + 1) { Some(n) => format!("L{n}"), None => "b2a_fall".to_string() };
        let names: Vec<&str> = labels.iter().filter(|(_, n)| *n == l.number).map(|(s, _)| s.as_str()).collect();
        let shown = if names.is_empty() { String::new() } else { format!(" ({})", names.iter().map(|n| format!("@{n}")).collect::<Vec<_>>().join(", ")) };
        if l.stmts.iter().all(|s| matches!(s, Stmt::Rem(_))) && o.no_rem { continue; }
        g.comment(&format!("---- Zeile {}{}: {}", l.number, shown, l.text));
        g.label(&format!("L{}", l.number));
        for n in &names { g.label(&format!("bas_{n}")); }
        map.push(MapEntry { basic_line: l.number, src_line: l.src_line, label: format!("L{}", l.number) });
        if o.track_line {
            g.ins(format!("lda #<{}", l.number)); g.ins("sta rt_line"); g.ins(format!("lda #>{}", l.number)); g.ins("sta rt_line+1");
        }
        g.stmts(&l.stmts, &next);
    }
    // Kommentarzeilen nach der letzten BASIC-Zeile
    if !o.no_rem { for (_, t) in &comments[ci..] { if !t.is_empty() { let _ = writeln!(g.body, ";{}", t); } } }
    // DEF FN: ein Unterprogramm je Funktion (Parameter wird für die Dauer des Aufrufs mit dem Argument belegt und danach zurückgesetzt)
    let main = std::mem::take(&mut g.body);
    let fns: Vec<(String, FnInfo)> = std::mem::take(&mut g.fns).into_iter().collect();
    for (_, f) in &fns {
        g.src_line = f.line;
        g.line_no = f.number;
        g.gen_fn(f);
    }
    g.fns = fns.into_iter().collect();
    g.subs = std::mem::replace(&mut g.body, main);
    // Zeilen, die ein `no_rem` übersprungen hat, dürfen kein Sprungziel sein: Label trotzdem anlegen
    if o.no_rem { for l in &prog.lines { if l.stmts.iter().all(|s| matches!(s, Stmt::Rem(_))) { let _ = writeln!(g.body, "L{}:", l.number); } } }
    if !g.ctl.is_empty() { g.src_line = 0; g.err("E341", format!("{} Block(s) von Turbo BASIC XL am Programmende nicht abgeschlossen (ENDIF, WEND, UNTIL, LOOP oder ENDPROC fehlt)", g.ctl.len())); }
    if !g.errs.is_empty() { return Err(g.errs); }

    // Ausgabe zusammensetzen
    let mut out = String::new();
    let _ = writeln!(out, "; Erzeugt von VintageCoder (basic2asm) aus {}", o.file);
    let _ = writeln!(out, "; Ziel: {t}, Stufe {} ({}), Stil compact. Jede BASIC-Zeile steht als Kommentar vor ihrem Block (Label L<Zeile>).", if g.str { 3 } else if g.float { 2 } else { 1 }, match (g.str, g.float) { (true, true) => "Zeichenketten und Fließkomma", (true, false) => "Zeichenketten und Ganzzahl", (false, true) => if ti.atari { "Fließkomma über das OS-Paket" } else { "Fließkomma über das BASIC-ROM" }, _ => "Ganzzahl" });
    if ti.atari { let _ = writeln!(out, ".target {t}\n.cpu {}\n.branchlong auto\n#include <atari8bit/os.asm>", ti.cpu); }
    else { let _ = writeln!(out, ".target {t}\n.cpu {}\n.branchlong {}\n#include <{t}/kernal.asm>", ti.cpu, if ti.cpu == "45gs02" { "jmp" } else { "auto" }); }   // 45GS02: Ergebnis bleibt 6502-kompatibel
    if g.float { let _ = writeln!(out, ".const RT_FLOAT = 1"); }
    let _ = writeln!(out, "#include <{}/b2a/rt_zp.asm>\n", ti.sys);
    if ti.atari { let _ = writeln!(out, ".org ${:04X}\n.run b2a_start\n", ti.org); }
    else { let _ = writeln!(out, ".org ${:04X}\n.basicstub b2a_start\n", ti.org); }
    let _ = writeln!(out, "b2a_start:");
    let _ = writeln!(out, "        jsr rt_init");
    if ti.atari && g.float { let _ = writeln!(out, "        lda #0\n        sta $FB              ; DEGFLG: Bogenmaß (DEG/RAD setzen ihn über POKE 251)"); }
    if g.str && !g.atari { let _ = writeln!(out, "        jsr rt_sinit"); }
    if g.uses_data || !g.data.is_empty() { let _ = writeln!(out, "        lda #<b2a_data\n        sta rt_dp\n        lda #>b2a_data\n        sta rt_dp+1"); }
    if g.uses_run { let _ = writeln!(out, "b2a_run:"); }
    out.push_str(&g.body);
    let _ = writeln!(out, "b2a_fall:");
    let _ = writeln!(out, "        jmp rt_end              ; Programmende\n");
    if !g.subs.is_empty() { let _ = writeln!(out, "; ---------------- DEF FN ----------------"); out.push_str(&g.subs); let _ = writeln!(out); }
    if g.uses_clr {
        let _ = writeln!(out, "; CLR: alle Variablen auf 0, Stapel leeren\nb2a_clr:");
        if g.str && !g.atari { let _ = writeln!(out, "        jsr rt_sinit"); }
        let _ = writeln!(out, "        lda #0\n        sta rt_ssp\n        sta rt_esp");
        for (l, _, size) in g.scalars.values() { for k in 0..*size { let _ = writeln!(out, "        sta {l}{}", if k == 0 { String::new() } else { format!("+{k}") }); } }
        for a in g.arrays.values() {
            let call = if a.size == 3 { "rt_sclrarr" } else { "rt_clrarr" };
            let _ = writeln!(out, "        lda #<{0}\n        sta rt_ap\n        lda #>{0}\n        sta rt_ap+1\n{1}        jsr {2}", a.label, if g.float && a.size != 3 { format!("        ldx #{}\n", a.size) } else { String::new() }, call);
        }
        let _ = writeln!(out, "        rts\n");
    }
    let mut modules = vec!["rt_core".to_string(), if g.float { "rt_fp" } else { "rt_arith" }.to_string()];
    if g.uses_flow { modules.push(if g.float { "rt_flowf" } else { "rt_flow" }.into()); }
    if g.uses_data || !g.arrays.is_empty() { modules.push(if g.float { "rt_dataf" } else { "rt_data" }.into()); }
    if g.uses_io || g.float { modules.push(if g.float { "rt_iof" } else { "rt_io" }.into()); }
    if g.str { modules.push(if g.atari { "rt_astr" } else { "rt_str" }.into()); }
    let _ = writeln!(out, "; ---------------- Laufzeit ----------------");
    if g.str && !g.atari && !g.sdata.is_empty() { let _ = writeln!(out, ".const RT_SDATA = 1"); }
    if g.atari && o.turbo { let _ = writeln!(out, ".const RT_P10 = 1"); }   // Turbo BASIC XL druckt alle 10 Stellen
    if g.str && g.atari && (g.uses_data || !g.data.is_empty()) { let _ = writeln!(out, ".const RT_ADATA = 1"); }
    for m in &modules { let _ = writeln!(out, "#include <{}/b2a/{m}.asm>", if m == "rt_astr" { "atari8bit" } else { "common" }); }
    if ti.atari { let _ = writeln!(out, "#include <atari8bit/b2a/rt_os.asm>"); }
    if ti.atari && g.float { let _ = writeln!(out, "#include <atari8bit/b2a/rt_osfp.asm>"); }
    if ti.atari && g.uses_turbo { let _ = writeln!(out, "#include <atari8bit/b2a/rt_turbo.asm>"); }
    if ti.atari && g.uses_turbo && g.str { let _ = writeln!(out, "#include <atari8bit/b2a/rt_tstr.asm>"); }
    if ti.atari && g.uses_gfx { let _ = writeln!(out, "#include <atari8bit/b2a/rt_gfx.asm>"); }
    if ti.atari && g.uses_open { let _ = writeln!(out, "#include <atari8bit/b2a/rt_copen.asm>"); }
    let _ = writeln!(out, "\n; ---------------- Daten ----------------");
    for (l, shown, size) in g.scalars.values() {
        match *size {
            n if n == g.fsz && g.float => { let _ = writeln!(out, "{l}: .byte {}       ; {shown}", vec!["0"; n as usize].join(", ")); }
            4 if g.atari => { let cap = g.scap.get(l).copied().unwrap_or(0); let _ = writeln!(out, "{l}: .word 0, 0               ; {shown} (Länge, Kapazität {cap})\n        .fill {cap}, 0"); }
            3 => { let _ = writeln!(out, "{l}: .byte 0, 0, 0             ; {shown} (Länge, Adresse)"); }
            _ => { let _ = writeln!(out, "{l}: .word 0                 ; {shown}"); }
        }
    }
    for a in g.arrays.values() { let _ = writeln!(out, "{}: .word {}               ; {}({})\n        .fill {}, 0", a.label, a.count, a.name, a.count - 1, a.count * a.size); }
    for (i, b) in g.fconsts.iter().enumerate() { let _ = writeln!(out, "fc_{}: .byte {}", i + 1, b.iter().map(|x| format!("${x:02X}")).collect::<Vec<_>>().join(", ")); }
    for f in g.fns.values() { let _ = writeln!(out, "fd_{}: .byte 0                 ; FN{} definiert?", f.label, f.label.trim_end_matches("_P")); }
    for (i, s) in g.strings.iter().enumerate() {
        if ti.atari {   // ASCII-Bytes unverändert (Atari-Quelle wird ohne PETSCII gelesen)
            let shown: String = s.iter().map(|b| if (0x20..0x7F).contains(b) && *b != b'"' { *b as char } else { '.' }).collect();
            let _ = writeln!(out, "str_{}: .byte {}{}   ; \"{}\"", i + 1, s.iter().map(|b| format!("${b:02X}")).collect::<Vec<_>>().join(", "), if s.is_empty() { "0" } else { ", 0" }, shown);
        } else { let _ = writeln!(out, "str_{}: .petscii \"{}\"\n        .byte 0", i + 1, petscii_text(s, d)); }
    }
    if g.uses_data || !g.data.is_empty() {
        let _ = writeln!(out, "b2a_data:");
        for (di, (ln, nums)) in g.data.iter().enumerate() {
            let _ = write!(out, "D{ln}:");
            if g.atari && g.str {
                let items = &g.adata[di];
                if items.is_empty() { let _ = writeln!(out); }
                for (k, it) in items.iter().enumerate() { let _ = writeln!(out, "{}.byte {}{}", if k == 0 { " " } else { "    " }, it.len() & 0xFF, it.iter().map(|b| format!(", ${b:02X}")).collect::<String>()); }
                continue;
            }
            if g.str {
                let items = std::mem::take(&mut g.sdata[di]);
                if items.is_empty() { let _ = writeln!(out); }
                for (k, it) in items.iter().enumerate() {
                    let (ptr, len) = match &it.label { None => ("0, 0".to_string(), 0), Some(l) => (format!("<{l}, >{l}"), it.text.len()) };
                    let payload = match (it.num, g.float) {
                        (Some(x), true) => g.fbytes(x).unwrap_or(vec![0; g.fsz as usize]).iter().map(|b| format!("${b:02X}")).collect::<Vec<_>>().join(", "),
                        (Some(x), false) => { let w = (x as i64 & 0xFFFF) as u16; format!("${:02X}, ${:02X}", w & 0xFF, w >> 8) }
                        (None, true) => vec!["0"; g.fsz as usize].join(", "),
                        (None, false) => "0, 0".to_string(),
                    };
                    let _ = writeln!(out, "{}.byte {}, {}, {}, {}", if k == 0 { " " } else { "    " }, len, ptr, if it.num.is_some() { 1 } else { 0 }, payload);
                }
                continue;
            }
            if g.float {
                let items = &g.fdata[di];
                if items.is_empty() { let _ = writeln!(out); }
                for (k, b) in items.iter().enumerate() { let _ = writeln!(out, "{}.byte {}", if k == 0 { " " } else { "    " }, b.iter().map(|x| format!("${x:02X}")).collect::<Vec<_>>().join(", ")); }
                continue;
            }
            if nums.is_empty() { let _ = writeln!(out); }
            for (k, chunk) in nums.chunks(8).enumerate() {
                let s: Vec<String> = chunk.iter().map(|n| hex16(*n)).collect();
                let _ = writeln!(out, "{}.word {}", if k == 0 { " " } else { "    " }, s.join(", "));
            }
        }
        let _ = writeln!(out, "b2a_data_end:");
    }
    if g.str && !g.atari {
        let roots: Vec<&str> = g.scalars.values().filter(|(_, _, sz)| *sz == 3).map(|(l, _, _)| l.as_str()).collect();
        let _ = writeln!(out, "b2a_sroots: .word {}0", roots.iter().map(|r| format!("{r}, ")).collect::<String>());
        let aroots: Vec<&str> = g.arrays.values().filter(|a| a.size == 3).map(|a| a.label.as_str()).collect();
        let _ = writeln!(out, "b2a_aroots: .word {}0", aroots.iter().map(|r| format!("{r}, ")).collect::<String>());
    }
    let _ = writeln!(out, "b2a_end:                    ; hier endet das Programm{}", if g.str { "; darüber liegt der Zeichenketten-Heap (bis RT_HEAPTOP)" } else { "" });
    let _ = writeln!(out, ".assert b2a_end <= RT_HEAPTOP, \"Programm zu groß für {t}: es muss unterhalb von RT_HEAPTOP enden\"");
    let _ = (&g.d, &g.o);
    Ok(Converted { asm: out, map, modules, stage: g.need_stage.max(1), warnings: g.warns, names: vec![] })
}

// ---- Stufe 2: Ausdrücke im FAC (Gleitkomma-Akkumulator des BASIC-ROMs)
impl<'a> G<'a> {
    /// Zahl im Format des Ziels: MBF (5 Byte, CBM) oder BCD (6 Byte, Atari-OS)
    fn fbytes(&self, v: f64) -> Result<Vec<u8>, String> {
        if self.atari { bcd(v).map(|b| b.to_vec()) } else { mbf(v).map(|b| b.to_vec()) }
    }
    fn fconst(&mut self, b: Vec<u8>) -> String {
        let i = self.fconsts.iter().position(|x| *x == b).unwrap_or_else(|| { self.fconsts.push(b); self.fconsts.len() - 1 });
        format!("fc_{}", i + 1)
    }
    fn fload(&mut self, label: &str) {
        self.ins(format!("lda #<{label}"));
        self.ins(format!("ldy #>{label}"));
        self.ins("jsr rt_movfm");
    }
    fn fnum(&mut self, n: i64) {
        if (-32768..=32767).contains(&n) { self.load_imm(n); self.ins("jsr rt_i2f"); }
        else { match self.fbytes(n as f64) { Ok(b) => { let c = self.fconst(b); self.fload(&c); } Err(e) => self.err("E303", e) } }
    }

    fn fexpr(&mut self, e: &Expr) {
        if self.str && self.str_hook(e) { return; }
        if !matches!(e, Expr::Num(_)) {
            if let Some(v) = self.konst(e) { self.fnum(v); return; }
        }
        match e {
            Expr::Num(n) => self.fnum(*n),
            Expr::Float(s) => match s.parse::<f64>().map_err(|_| format!("Zahl {s} ungültig")).and_then(|v| self.fbytes(v)) {
                Ok(b) => { let c = self.fconst(b); self.fload(&c); }
                Err(m) => self.err("E303", m),
            },
            Expr::Str(_) => self.stage(3, "Zeichenkette in einem Ausdruck"),
            Expr::Var(v) => {
                if v.ty == Ty::Str { self.stage(3, &format!("Zeichenkettenvariable {}$", v.name)); return; }
                if v.ty != Ty::Int && v.name == "TI" { self.ins("jsr rt_fti"); return; }
                if v.ty != Ty::Int && v.name == "ST" { self.ins("lda $90"); self.ins("ldy #0"); self.ins("jsr rt_i2f"); return; }
                let l = self.scalar(v);
                if v.ty == Ty::Int { self.ins(format!("lda {l}")); self.ins(format!("ldy {l}+1")); self.ins("jsr rt_i2f"); }
                else { self.fload(&l); }
            }
            Expr::Elem(v, idx) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenfeld"); return; }
                if idx.len() != 1 { self.err("E304", "Es sind nur eindimensionale Felder umgesetzt"); return; }
                self.fexpr(&idx[0]);
                self.ins("jsr rt_f2addr");
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins(if v.ty == Ty::Float { "jsr rt_ldelemf" } else { "jsr rt_ldelemi" });
            }
            Expr::Un(op, x) => {
                self.fexpr(x);
                self.ins(match op { UnOp::Neg => "jsr F_NEGOP", UnOp::Not => "jsr rt_fnot" });
            }
            Expr::Bin(op, a, b) => {
                let rt = match op {
                    BinOp::Add => "rt_fadd", BinOp::Sub => "rt_fsub", BinOp::Mul => "rt_fmul", BinOp::Div => "rt_fdiv", BinOp::Pow => "rt_fpow",
                    BinOp::Eq => "rt_feq", BinOp::Ne => "rt_fne", BinOp::Lt => "rt_flt", BinOp::Gt => "rt_fgt", BinOp::Le => "rt_fle", BinOp::Ge => "rt_fge",
                    BinOp::And => "rt_fand", BinOp::Or => "rt_forr",
                };
                self.fexpr(a);
                self.ins("jsr rt_fpush");
                self.fexpr(b);
                self.ins(format!("jsr {rt}"));
            }
            Expr::Call(name, args) => {
                if args.len() != 1 && matches!(name.as_str(), "PEEK" | "ABS" | "SGN" | "INT" | "SQR" | "RND" | "LOG" | "EXP" | "SIN" | "COS" | "TAN" | "ATN" | "STICK" | "STRIG") {
                    self.err("E305", format!("{name} erwartet genau ein Argument")); return;
                }
                if self.atari && name == "TAN" {
                    self.err("E334", "TAN gibt es in Atari-BASIC nicht (SIN(x)/COS(x) benutzen)"); return;
                }
                let rom = match name.as_str() {
                    "ABS" => "F_ABS", "SGN" => "F_SGN", "INT" => "F_INT", "SQR" => "F_SQR", "RND" => "F_RND", "LOG" => "F_LOG", "EXP" => "F_EXP",
                    "SIN" => "F_SIN", "COS" => "F_COS", "TAN" => "F_TAN", "ATN" => "F_ATN",
                    "DIV" if args.len() == 2 => { self.fexpr(&args[0]); self.ins("jsr rt_fpush"); self.fexpr(&args[1]); self.ins("jsr rt_fdiv"); self.ins("jsr rt_ftrunc"); self.uses_turbo = true; return; }
                    "DPEEK" if args.len() == 1 => { self.uses_turbo = true; self.num_ay(&args[0]); self.ins("jsr rt_dpeek"); self.ins("jsr rt_u2f"); return; }
                    "STICK" | "STRIG" => { self.uses_gfx = true; self.num_ay(&args[0]); self.ins(if name == "STICK" { "jsr rt_stick" } else { "jsr rt_strig" }); self.ins("jsr rt_i2f"); return; }
                    "PEEK" => { self.fexpr(&args[0]); self.ins("jsr rt_fpeek"); return; }
                    "CHR$" | "STR$" | "LEFT$" | "RIGHT$" | "MID$" | "LEN" | "ASC" | "VAL" | "HEX$" | "DEC" | "INSTR" | "INKEY$" | "TIME$" => { self.stage(3, name); return; }
                    _ => { self.err("E306", format!("Funktion {name} wird nicht übersetzt")); return; }
                };
                self.fexpr(&args[0]);
                self.ins(format!("jsr {rom}"));
            }
            Expr::Fn(n, a) => self.fn_call(n, a),
        }
    }

    /// FAC → Variable
    fn fstore(&mut self, lv: &LValue) {
        match lv {
            LValue::Var(v) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenvariable"); return; }
                if is_reserved_scalar(v) { self.err("E307", format!("{} kann nicht zugewiesen werden", v.name)); return; }
                let l = self.scalar(v);
                if v.ty == Ty::Int { self.ins("jsr rt_f2i"); self.ins(format!("sta {l}")); self.ins(format!("sty {l}+1")); }
                else { self.ins(format!("ldx #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr F_MOVMF"); }
            }
            LValue::Elem(v, idx) => {
                if v.ty == Ty::Str { self.stage(3, "Zeichenkettenfeld"); return; }
                if idx.len() != 1 { self.err("E304", "Es sind nur eindimensionale Felder umgesetzt"); return; }
                self.ins("jsr rt_fpush");
                self.fexpr(&idx[0]);
                self.ins("jsr rt_f2addr");
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins(if v.ty == Ty::Float { "jsr rt_stelemf" } else { "jsr rt_stelemi" });
            }
        }
    }
}

// ---- Stufe 3: Zeichenketten (Akkumulator rt_sacc, Stringstapel; Laufzeit rt_str)
impl<'a> G<'a> {
    fn is_str(&self, e: &Expr) -> bool {
        match e {
            Expr::Str(_) => true,
            Expr::Var(v) | Expr::Elem(v, _) => v.ty == Ty::Str,
            Expr::Bin(BinOp::Add, a, b) => self.is_str(a) || self.is_str(b),
            Expr::Call(n, _) => matches!(n.as_str(), "CHR$" | "STR$" | "LEFT$" | "RIGHT$" | "MID$" | "HEX$" | "INKEY$" | "TIME$"),
            _ => false,
        }
    }

    /// Zahl nach A/Y (vorzeichenlos 0..65535; im Fließkomma-Modus über das ROM)
    fn num_ay(&mut self, e: &Expr) {
        if self.is_str(e) { self.err("E320", "Zahl erwartet, aber eine Zeichenkette (TYPE MISMATCH)"); return; }
        self.expr(e);
        if self.float { self.ins("jsr rt_f2addr"); }
    }

    /// Behandelt Ausdrücke mit Zeichenketten im Zahlenzusammenhang; true = erledigt
    fn str_hook(&mut self, e: &Expr) -> bool {
        match e {
            Expr::Str(_) => { self.err("E320", "Zeichenkette, aber eine Zahl erwartet (TYPE MISMATCH)"); true }
            Expr::Var(v) | Expr::Elem(v, _) if v.ty == Ty::Str => { self.err("E320", format!("Zeichenkettenvariable {}$, aber eine Zahl erwartet (TYPE MISMATCH)", v.name)); true }
            Expr::Bin(op, a, b) if self.is_str(a) || self.is_str(b) => {
                let rt = match op {
                    BinOp::Eq => "rt_seq", BinOp::Ne => "rt_sne", BinOp::Lt => "rt_slt", BinOp::Gt => "rt_sgt", BinOp::Le => "rt_sle", BinOp::Ge => "rt_sge",
                    _ => { self.err("E320", "Mit Zeichenketten sind nur + und Vergleiche erlaubt (TYPE MISMATCH)"); return true; }
                };
                if !(self.is_str(a) && self.is_str(b)) { self.err("E320", "Zeichenkette und Zahl lassen sich nicht vergleichen (TYPE MISMATCH)"); return true; }
                self.sexpr(a);
                self.ins("jsr rt_spush");
                self.sexpr(b);
                self.ins(format!("jsr {rt}"));
                true
            }
            Expr::Un(_, x) if self.is_str(x) => { self.err("E320", "Mit einer Zeichenkette kann man nicht rechnen (TYPE MISMATCH)"); true }
            Expr::Call(n, args) if matches!(n.as_str(), "LEN" | "ASC" | "VAL") => {
                if args.len() != 1 { self.err("E305", format!("{n} erwartet genau ein Argument")); return true; }
                if !self.is_str(&args[0]) { self.err("E320", format!("{n} erwartet eine Zeichenkette (TYPE MISMATCH)")); return true; }
                self.sexpr(&args[0]);
                match n.as_str() {
                    "LEN" | "ASC" => { self.ins(if n == "LEN" { "jsr rt_len" } else { "jsr rt_asc" }); if self.float { self.ins("jsr rt_i2f"); } }
                    _ => { self.ins("jsr rt_val"); }
                }
                true
            }
            Expr::Call(n, args) if n == "DEC" => {
                if args.len() != 1 || !self.is_str(&args[0]) { self.err("E320", "DEC erwartet eine Zeichenkette (TYPE MISMATCH)"); return true; }
                self.uses_turbo = true;
                self.sexpr(&args[0]);
                self.ins("jsr rt_dec"); self.ins("jsr rt_u2f");
                true
            }
            Expr::Call(n, args) if n == "INSTR" => {
                if !(2..=3).contains(&args.len()) || !self.is_str(&args[0]) || !self.is_str(&args[1]) { self.err("E320", "INSTR(a$,b$) oder INSTR(a$,b$,n) mit Zeichenketten (TYPE MISMATCH)"); return true; }
                self.uses_turbo = true;
                self.sexpr(&args[0]); self.ins("jsr rt_spush");
                self.sexpr(&args[1]); self.ins("jsr rt_spush");
                match args.get(2) { Some(k) => self.num_ay(k), None => { self.ins("lda #0"); self.ins("ldy #0"); } }
                self.ins("jsr rt_instr"); self.ins("jsr rt_i2f");
                true
            }
            Expr::Call(n, _) if matches!(n.as_str(), "CHR$" | "STR$" | "LEFT$" | "RIGHT$" | "MID$" | "HEX$" | "INKEY$" | "TIME$") => { self.err("E320", format!("{n} liefert eine Zeichenkette, hier wird eine Zahl erwartet (TYPE MISMATCH)")); true }
            _ => false,
        }
    }

    /// Zeichenkettenausdruck → Akkumulator
    fn sexpr(&mut self, e: &Expr) {
        if self.atari { return self.a_sexpr(e); }
        match e {
            Expr::Str(b) => {
                if b.is_empty() { self.ins("lda #0"); self.ins("sta rt_sacc"); return; }
                let l = self.intern(b);
                self.ins(format!("lda #{}", b.len()));
                self.ins("sta rt_sacc");
                self.ins(format!("lda #<{l}"));
                self.ins("sta rt_sacc+1");
                self.ins(format!("lda #>{l}"));
                self.ins("sta rt_sacc+2");
            }
            Expr::Var(v) if v.name == "TI" => { self.ins("jsr rt_tis"); }
            Expr::Var(v) => {
                let l = self.scalar(v);
                self.ins(format!("lda #<{l}"));
                self.ins(format!("ldy #>{l}"));
                self.ins("jsr rt_sload");
            }
            Expr::Elem(v, idx) => {
                if idx.len() != 1 { self.err("E304", "Es sind nur eindimensionale Felder umgesetzt"); return; }
                self.num_ay(&idx[0]);
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_sldelem");
            }
            Expr::Bin(BinOp::Add, a, b) => {
                if !(self.is_str(a) && self.is_str(b)) { self.err("E320", "Zeichenkette und Zahl lassen sich nicht verketten (TYPE MISMATCH)"); return; }
                self.sexpr(a);
                self.ins("jsr rt_spush");
                self.sexpr(b);
                self.ins("jsr rt_scat");
            }
            Expr::Call(n, args) => {
                let arity_ok = match n.as_str() { "CHR$" | "STR$" => args.len() == 1, "LEFT$" | "RIGHT$" => args.len() == 2, _ => args.len() == 2 || args.len() == 3 };
                if !arity_ok { self.err("E305", format!("{n}: falsche Argumentzahl")); return; }
                match n.as_str() {
                    "CHR$" => { self.num_ay(&args[0]); self.ins("jsr rt_chr"); }
                    "STR$" => {
                        if self.is_str(&args[0]) { self.err("E320", "STR$ erwartet eine Zahl (TYPE MISMATCH)"); return; }
                        self.expr(&args[0]);
                        self.ins("jsr rt_strs");
                    }
                    _ => {
                        if !self.is_str(&args[0]) { self.err("E320", format!("{n} erwartet eine Zeichenkette (TYPE MISMATCH)")); return; }
                        self.sexpr(&args[0]);
                        self.ins("jsr rt_spush");
                        if n == "MID$" {
                            self.num_ay(&args[1]);
                            self.ins("jsr rt_push");
                            match args.get(2) { Some(l) => self.num_ay(l), None => { self.ins("lda #255"); self.ins("ldy #0"); } }
                            self.ins("jsr rt_mid");
                        } else {
                            self.num_ay(&args[1]);
                            self.ins(if n == "LEFT$" { "jsr rt_left" } else { "jsr rt_right" });
                        }
                    }
                }
            }
            _ => self.err("E320", "Zeichenkettenausdruck erwartet (TYPE MISMATCH)"),
        }
    }

    /// Akkumulator → Variable
    fn sstore(&mut self, lv: &LValue) {
        if self.atari { return self.a_sstore(lv); }
        match lv {
            LValue::Var(v) if v.name == "TI" => { self.ins("jsr rt_tiset"); }
            LValue::Var(v) => {
                let l = self.scalar(v);
                self.ins(format!("lda #<{l}"));
                self.ins(format!("ldy #>{l}"));
                self.ins("jsr rt_sstor");
            }
            LValue::Elem(v, idx) => {
                if idx.len() != 1 { self.err("E304", "Es sind nur eindimensionale Felder umgesetzt"); return; }
                self.ins("jsr rt_spush");
                self.num_ay(&idx[0]);
                let l = self.array(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_ststelem");
            }
        }
    }
}

// ---- Turbo BASIC XL: Blockanweisungen und Prozeduren (nur Ziel atari8)
impl<'a> G<'a> {
    /// Bedingung auswerten und zu `label` springen, wenn sie 0 ist
    fn jump_if_false(&mut self, cond: &Expr, label: &str) {
        self.expr(cond);
        if self.float { self.ins("lda F_FACEXP         ; Exponent des FAC: 0 = Zahl ist 0"); } else { self.ins("sty rt_tmp"); self.ins("ora rt_tmp"); }
        self.ins(format!("beq {label}"));
    }

    fn turbo_stmt(&mut self, t: &Struct) {
        match t {
            Struct::If(c) => { let (els, end) = (self.fresh(), self.fresh()); self.jump_if_false(c, &els); self.ctl.push(Ctl::If { els, end, has_else: false }); }
            Struct::Else => match self.ctl.pop() {
                Some(Ctl::If { els, end, has_else: false }) => { self.ins(format!("jmp {end}")); self.label(&els); self.ctl.push(Ctl::If { els, end, has_else: true }); }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "ELSE ohne passendes IF (oder zweites ELSE)"); }
            },
            Struct::Endif => match self.ctl.pop() {
                Some(Ctl::If { els, end, has_else }) => { if has_else { self.label(&end); } else { self.label(&els); } }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "ENDIF ohne passendes IF"); }
            },
            Struct::While(c) => { let (start, end) = (self.fresh(), self.fresh()); self.label(&start); self.jump_if_false(c, &end); self.ctl.push(Ctl::While { start, end }); }
            Struct::Wend => match self.ctl.pop() {
                Some(Ctl::While { start, end }) => { self.ins(format!("jmp {start}")); self.label(&end); }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "WEND ohne passendes WHILE"); }
            },
            Struct::Repeat => { let (start, end) = (self.fresh(), self.fresh()); self.label(&start); self.ctl.push(Ctl::Repeat { start, end }); }
            Struct::Until(c) => match self.ctl.pop() {
                Some(Ctl::Repeat { start, end }) => { self.jump_if_false(c, &start); self.label(&end); }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "UNTIL ohne passendes REPEAT"); }
            },
            Struct::Do => { let (start, end) = (self.fresh(), self.fresh()); self.label(&start); self.ctl.push(Ctl::Do { start, end }); }
            Struct::Loop => match self.ctl.pop() {
                Some(Ctl::Do { start, end }) => { self.ins(format!("jmp {start}")); self.label(&end); }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "LOOP ohne passendes DO"); }
            },
            Struct::Exit => {
                let end = self.ctl.iter().rev().find_map(|c| match c { Ctl::While { end, .. } | Ctl::Repeat { end, .. } | Ctl::Do { end, .. } => Some(end.clone()), _ => None });
                match end { Some(e) => self.ins(format!("jmp {e}")), None => self.err("E342", "EXIT außerhalb von WHILE, REPEAT und DO") }
            }
            Struct::Proc(n) => {
                if self.ctl.iter().any(|c| matches!(c, Ctl::Proc { .. })) { self.err("E342", "PROC innerhalb einer Prozedur"); return; }
                let end = self.fresh();
                self.ins(format!("jmp {end}"));   // beim Durchlaufen wird die Prozedur übersprungen
                self.label(&format!("P_{n}"));
                self.ctl.push(Ctl::Proc { end });
            }
            Struct::Endproc => match self.ctl.pop() {
                Some(Ctl::Proc { end }) => { self.uses_flow = true; self.ins("jmp rt_return"); self.label(&end); }
                other => { if let Some(o) = other { self.ctl.push(o); } self.err("E342", "ENDPROC ohne passendes PROC"); }
            },
            Struct::OnExec(sel, names) => {
                if let Some(n) = names.iter().find(|n| !self.procs.contains(*n)) { self.err("E343", format!("ON … EXEC: Prozedur {n} ist nicht definiert")); return; }
                self.uses_flow = true;
                let end = self.fresh();
                self.set_ptr("rt_bp", &end);
                self.expr(sel);
                if self.float { self.ins("jsr rt_f2addr"); }
                self.ins("jsr rt_byte");
                for (i, n) in names.iter().enumerate() {
                    let nxt = self.fresh();
                    self.ins(format!("cmp #{}", i + 1));
                    self.ins(format!("bne {nxt}"));
                    self.ins("jsr rt_gosub");
                    self.ins(format!("jmp P_{n}"));
                    self.label(&nxt);
                }
                self.label(&end);
            }
            Struct::Exec(n) => {
                if !self.procs.contains(n) { self.err("E343", format!("EXEC: Prozedur {n} ist nicht definiert")); return; }
                self.uses_flow = true;
                let ret = self.fresh();
                self.set_ptr("rt_bp", &ret);
                self.ins("jsr rt_gosub");
                self.ins(format!("jmp P_{n}"));
                self.label(&ret);
            }
        }
    }
}

// ---- Atari: Zeichenketten als feste Puffer (Laufzeit rt_astr)
impl<'a> G<'a> {
    /// Zeichenkettenausdruck → Akkumulator (rt_sal = Länge, rt_sap = Zeiger)
    fn a_sexpr(&mut self, e: &Expr) {
        match e {
            Expr::Str(b) => {
                let l = self.intern(b);
                self.ins(format!("lda #{}", b.len() & 0xFF)); self.ins("sta rt_sal"); self.ins("lda #0"); self.ins("sta rt_sal+1");
                self.ins(format!("lda #<{l}")); self.ins("sta rt_sap"); self.ins(format!("lda #>{l}")); self.ins("sta rt_sap+1");
            }
            Expr::Var(v) if v.ty == Ty::Str => {
                let l = self.scalar(v);
                self.ins(format!("lda #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr rt_sload");
            }
            Expr::Elem(v, idx) if v.ty == Ty::Str => {
                if idx.is_empty() || idx.len() > 2 { self.err("E304", "Teilzeichenkette: A$(n) oder A$(n,m)"); return; }
                self.num_ay(&idx[0]);
                self.ins("jsr rt_push");
                match idx.get(1) { Some(m) => self.num_ay(m), None => { self.ins("lda #$FF"); self.ins("ldy #$FF"); } }
                let l = self.scalar(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_ssub");
            }
            Expr::Bin(BinOp::Add, _, _) => self.err("E311", "Atari-BASIC kennt kein + für Zeichenketten (A$(LEN(A$)+1) = B$ hängt an)"),
            Expr::Call(n, args) => match n.as_str() {
                "CHR$" if args.len() == 1 => { self.num_ay(&args[0]); self.ins("jsr rt_chr"); }
                "STR$" if args.len() == 1 => {
                    if self.is_str(&args[0]) { self.err("E320", "STR$ erwartet eine Zahl (TYPE MISMATCH)"); return; }
                    self.expr(&args[0]);
                    self.ins("jsr rt_strs");
                }
                "HEX$" if args.len() == 1 => { self.uses_turbo = true; self.num_ay(&args[0]); self.ins("jsr rt_hexs"); }
                "INKEY$" if args.is_empty() => { self.uses_turbo = true; self.ins("jsr rt_inkeys"); }
                "TIME$" if args.is_empty() => { self.uses_turbo = true; self.ins("jsr rt_times"); }
                "LEFT$" | "RIGHT$" | "MID$" => self.err("E311", format!("{n} gibt es in Atari-BASIC nicht (Teilzeichenkette A$(n,m) benutzen)")),
                _ => self.err("E305", format!("{n}: falsche Argumentzahl")),
            },
            _ => self.err("E320", "Zeichenkettenausdruck erwartet (TYPE MISMATCH)"),
        }
    }

    /// Akkumulator → Variable oder Teilzeichenkette
    fn a_sstore(&mut self, lv: &LValue) {
        match lv {
            LValue::Var(v) => {
                let l = self.scalar(v);
                self.ins(format!("lda #<{l}")); self.ins(format!("ldy #>{l}")); self.ins("jsr rt_sstor");
            }
            LValue::Elem(v, idx) => {
                if idx.is_empty() || idx.len() > 2 { self.err("E304", "Teilzeichenkette: A$(n) oder A$(n,m)"); return; }
                self.ins("jsr rt_spush");
                self.num_ay(&idx[0]);
                self.ins("jsr rt_push");
                match idx.get(1) { Some(m) => self.num_ay(m), None => { self.ins("lda #$FF"); self.ins("ldy #$FF"); } }
                let l = self.scalar(v);
                self.set_ap_keep(&l);
                self.ins("jsr rt_sstsub");
            }
        }
    }
}

// ---- DEF FN
impl<'a> G<'a> {
    fn fn_label(&self, name: &str) -> String {
        format!("{}{}", name.trim_end_matches(['%', '$']), if name.ends_with('%') { "_P" } else { "" })
    }

    /// FNa(arg): Argument auswerten, Unterprogramm aufrufen
    fn fn_call(&mut self, name: &str, arg: &Expr) {
        let key = fn_key(name);
        let Some(info) = self.fns.get(&key) else { self.err("E317", format!("FN{name} ist nirgends mit DEF FN definiert (UNDEF'D FUNCTION)")); return };
        let (label, ret_int) = (info.label.clone(), info.ret_int);
        if name.ends_with('$') { self.err("E315", "Zeichenkettenfunktionen werden nicht übersetzt"); return; }
        if self.str && self.is_str(arg) { self.err("E320", format!("FN{name} erwartet eine Zahl (TYPE MISMATCH)")); return; }
        self.expr(arg);
        self.ins(format!("jsr fn_{label}"));
        if ret_int && self.float { self.ins("jsr rt_f2i"); self.ins("jsr rt_i2f"); }
    }

    fn gen_fn(&mut self, f: &FnInfo) {
        let name = f.label.trim_end_matches("_P").to_string();
        let flag = format!("fd_{}", f.label);
        let v = self.scalar(&f.param);
        let shown = display_of(&f.param);
        self.comment(&format!("---- Zeile {}: DEF FN{name}({shown}) – Parameter {shown} gilt nur während des Aufrufs", f.number));
        self.label(&format!("fn_{}", f.label));
        let ok = format!("fn_{}_ok", f.label);
        if self.float {
            self.ins(format!("lda {flag}"));
            self.ins(format!("bne {ok}"));
            self.ins("jmp rt_e_undef");
            self.label(&ok);
            if f.param.ty == Ty::Int {
                self.ins(format!("lda {v}")); self.ins("pha"); self.ins(format!("lda {v}+1")); self.ins("pha");
                self.ins("jsr rt_f2i"); self.ins(format!("sta {v}")); self.ins(format!("sty {v}+1"));
                self.expr(&f.body);
                self.ins("pla"); self.ins(format!("sta {v}+1")); self.ins("pla"); self.ins(format!("sta {v}"));
            } else {
                let s = format!("fn_{}_s", f.label);
                let r = format!("fn_{}_r", f.label);
                self.ins("ldx #4");
                self.label(&s);
                self.ins(format!("lda {v},x")); self.ins("pha"); self.ins("dex"); self.ins(format!("bpl {s}"));
                self.ins(format!("ldx #<{v}")); self.ins(format!("ldy #>{v}")); self.ins("jsr F_MOVMF");
                self.expr(&f.body);
                self.ins("ldx #0");
                self.label(&r);
                self.ins("pla"); self.ins(format!("sta {v},x")); self.ins("inx"); self.ins("cpx #5"); self.ins(format!("bne {r}"));
            }
            self.ins("rts");
        } else {
            self.ins("sta rt_v");
            self.ins("sty rt_v+1");
            self.ins(format!("lda {flag}"));
            self.ins(format!("bne {ok}"));
            self.ins("jmp rt_e_undef");
            self.label(&ok);
            self.ins(format!("lda {v}")); self.ins("pha"); self.ins(format!("lda {v}+1")); self.ins("pha");
            self.ins("lda rt_v"); self.ins(format!("sta {v}")); self.ins("lda rt_v+1"); self.ins(format!("sta {v}+1"));
            self.expr(&f.body);
            self.ins("sta rt_v"); self.ins("sty rt_v+1");
            self.ins("pla"); self.ins(format!("sta {v}+1")); self.ins("pla"); self.ins(format!("sta {v}"));
            self.ins("lda rt_v"); self.ins("ldy rt_v+1");
            self.ins("rts");
        }
    }
}
