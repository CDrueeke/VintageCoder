// SPDX-License-Identifier: GPL-3.0-or-later
//! Syntaxbaum für natives CBM-BASIC (Phase 0 von basic2asm).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty { Float, Int, Str }

/// Variable: `name` ist der volle Name in Großbuchstaben ohne Typkennzeichen
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var { pub name: String, pub ty: Ty }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp { Neg, Not }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp { Add, Sub, Mul, Div, Pow, Eq, Ne, Lt, Gt, Le, Ge, And, Or }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Ganzzahlliteral
    Num(i64),
    /// Literal mit Dezimalpunkt oder Exponent (Stufe 2)
    Float(String),
    /// Zeichenkette (PETSCII-Bytes)
    Str(Vec<u8>),
    Var(Var),
    Elem(Var, Vec<Expr>),
    Un(UnOp, Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    /// eingebaute Funktion (`PEEK`, `CHR$`, …) in Großbuchstaben
    Call(String, Vec<Expr>),
    /// `FN name(arg)`
    Fn(String, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LValue { Var(Var), Elem(Var, Vec<Expr>) }

#[derive(Debug, Clone, PartialEq)]
pub enum PrintItem { Expr(Expr), Tab(Expr), Spc(Expr), Semi, Comma }

#[derive(Debug, Clone, PartialEq)]
pub enum DataItem { Num(i64), Float(String), Str(Vec<u8>), /// Atari-BASIC: roher Text bis zum Komma (Anführungszeichen und Leerzeichen gehören dazu), erst beim Lesen Zahl oder Zeichenkette
    Raw(Vec<u8>) }

/// Turbo BASIC XL: Blockanweisungen und Prozeduren
#[derive(Debug, Clone, PartialEq)]
pub enum Struct { If(Expr), Else, Endif, While(Expr), Wend, Repeat, Until(Expr), Do, Loop, Exit, Proc(String), Endproc, Exec(String), OnExec(Expr, Vec<String>) }

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { target: LValue, expr: Expr },
    Print { items: Vec<PrintItem> },
    Input { prompt: Option<Vec<u8>>, vars: Vec<LValue> },
    Get(LValue),
    Goto(u32),
    Gosub(u32),
    Return,
    OnGoto { sel: Expr, targets: Vec<u32> },
    OnGosub { sel: Expr, targets: Vec<u32> },
    /// `then` reicht bis zum Zeilenende; `IF x THEN 100` ist `then = [Goto(100)]`
    If { cond: Expr, then: Vec<Stmt> },
    For { var: Var, from: Expr, to: Expr, step: Option<Expr> },
    /// leer = `NEXT` ohne Variable
    Next(Vec<Var>),
    Dim(Vec<(Var, Vec<Expr>)>),
    Read(Vec<LValue>),
    Data(Vec<DataItem>),
    Restore(Option<u32>),
    Poke(Expr, Expr),
    Sys(Expr),
    Wait(Expr, Expr, Option<Expr>),
    Rem(Vec<u8>),
    /// Atari: `GRAPHICS`, `COLOR`, `PLOT`, `DRAWTO`, `POSITION`, `SETCOLOR`, `SOUND` mit ihren Zahlenargumenten
    Atari { op: String, args: Vec<Expr> },
    /// Turbo BASIC XL: `IF` ohne `THEN`, `ELSE`, `ENDIF`, `WHILE`/`WEND`, `REPEAT`/`UNTIL`, `DO`/`LOOP`, `EXIT`, `PROC`/`ENDPROC`/`EXEC`
    Turbo(Struct),
    /// Atari: `LOCATE x,y,var`
    Locate(Expr, Expr, LValue),
    /// Atari: `GET #n,v` (ein Byte vom Kanal in eine Zahl)
    GetCh(Expr, LValue),
    /// Atari: `PRINT #n; …`
    PrintTo { chan: Expr, items: Vec<PrintItem> },
    End,
    Stop,
    Clr,
    Run(Option<u32>),
    /// `DEF FNa(x) = ausdruck`; `name` ist der Name nach FN (mit Typkennzeichen), `param` die Parametervariable
    DefFn { name: String, param: Var, body: Expr },
    /// Anweisung, die der Parser kennt, der Konverter aber nicht übersetzt (z. B. `OPEN`, `LOAD`)
    Unsupported(String),
}

#[derive(Debug, Clone)]
pub struct Line {
    pub number: u32,
    /// Zeile in der Quelldatei (1-basiert)
    pub src_line: usize,
    /// Text nach der Zeilennummer, wie im Quelltext (mit aufgelösten Labels)
    pub text: String,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program { pub lines: Vec<Line> }
