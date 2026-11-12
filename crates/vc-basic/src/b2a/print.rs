// SPDX-License-Identifier: GPL-3.0-or-later
//! Syntaxbaum → kanonischer BASIC-Text (für die Rundreise Text → Baum → Text → Baum) und Syntaxbaum → JSON.
use super::ast::*;
use crate::{petscii, Dialect};
use serde_json::{json, Value};

/// Vorrangstufen (niedrig → hoch). CBM: OR < AND < NOT < Vergleich < +- < */ < unäres - < ^; Atari: NOT und unäre Vorzeichen stehen über ^.
const OR: u8 = 1;
const AND: u8 = 2;
const NOT: u8 = 3;
const CMP: u8 = 4;
const ADD: u8 = 5;
const MUL: u8 = 6;
const NEG: u8 = 7;
const POW: u8 = 8;
const ATOM: u8 = 10;

struct W<'a> { d: &'a Dialect, atari: bool }

impl<'a> W<'a> {
    fn text(&self, b: &[u8]) -> Result<String, String> {
        if self.atari { return Ok(b.iter().map(|c| *c as char).collect()); }
        Ok(b.iter().map(|c| petscii::to_text(*c, self.d)).collect())
    }
    fn var(v: &Var) -> String { format!("{}{}", v.name, match v.ty { Ty::Int => "%", Ty::Str => "$", Ty::Float => "" }) }
    fn level(&self, e: &Expr) -> u8 {
        match e {
            Expr::Bin(op, _, _) => match op { BinOp::Or => OR, BinOp::And => AND, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => CMP, BinOp::Add | BinOp::Sub => ADD, BinOp::Mul | BinOp::Div => MUL, BinOp::Pow => POW },
            Expr::Call(n, a) if n == "DIV" && a.len() == 2 => MUL,
            Expr::Un(UnOp::Not, _) => if self.atari { POW + 1 } else { NOT },
            Expr::Un(UnOp::Neg, _) => if self.atari { POW + 1 } else { NEG },
            _ => ATOM,
        }
    }
    /// Ausdruck; `need` = niedrigste Stufe, die ohne Klammern stehen darf
    fn expr(&self, e: &Expr, need: u8) -> Result<String, String> {
        let s = match e {
            Expr::Num(n) => n.to_string(),
            Expr::Float(f) => f.clone(),
            Expr::Str(b) => format!("\"{}\"", self.text(b)?),
            Expr::Var(v) => Self::var(v),
            Expr::Elem(v, a) => format!("{}({})", Self::var(v), self.list(a)?),
            Expr::Call(n, a) if n == "DIV" && a.len() == 2 => format!("{} DIV {}", self.expr(&a[0], MUL)?, self.expr(&a[1], MUL + 1)?),
            Expr::Call(n, a) if a.is_empty() => n.clone(),
            Expr::Call(n, a) => format!("{n}({})", self.list(a)?),
            Expr::Fn(n, a) => format!("FN{n}({})", self.expr(a, 0)?),
            Expr::Un(UnOp::Neg, x) => format!("-{}", self.expr(x, if self.atari { POW + 1 } else { NEG })?),
            Expr::Un(UnOp::Not, x) => format!("NOT {}", self.expr(x, if self.atari { POW + 1 } else { NOT })?),
            Expr::Bin(op, a, b) => {
                let (sym, lvl) = match op {
                    BinOp::Or => (" OR ", OR), BinOp::And => (" AND ", AND), BinOp::Eq => ("=", CMP), BinOp::Ne => ("<>", CMP), BinOp::Lt => ("<", CMP), BinOp::Gt => (">", CMP), BinOp::Le => ("<=", CMP), BinOp::Ge => (">=", CMP),
                    BinOp::Add => ("+", ADD), BinOp::Sub => ("-", ADD), BinOp::Mul => ("*", MUL), BinOp::Div => ("/", MUL), BinOp::Pow => ("^", POW),
                };
                format!("{}{sym}{}", self.expr(a, lvl)?, self.expr(b, lvl + 1)?)
            }
        };
        Ok(if self.level(e) < need { format!("({s})") } else { s })
    }
    fn list(&self, a: &[Expr]) -> Result<String, String> { Ok(a.iter().map(|x| self.expr(x, 0)).collect::<Result<Vec<_>, _>>()?.join(",")) }
    fn lv(&self, l: &LValue) -> Result<String, String> {
        Ok(match l { LValue::Var(v) => Self::var(v), LValue::Elem(v, a) => format!("{}({})", Self::var(v), self.list(a)?) })
    }
    fn items(&self, items: &[PrintItem]) -> Result<String, String> {
        let mut s = String::new();
        for i in items {
            match i {
                PrintItem::Expr(e) => s.push_str(&self.expr(e, 0)?),
                PrintItem::Tab(e) => s.push_str(&format!("TAB({})", self.expr(e, 0)?)),
                PrintItem::Spc(e) => s.push_str(&format!("SPC({})", self.expr(e, 0)?)),
                PrintItem::Semi => s.push(';'),
                PrintItem::Comma => s.push(','),
            }
        }
        Ok(s)
    }
    fn stmts(&self, v: &[Stmt]) -> Result<String, String> { Ok(v.iter().map(|s| self.stmt(s)).collect::<Result<Vec<_>, _>>()?.join(":")) }
    fn stmt(&self, s: &Stmt) -> Result<String, String> {
        Ok(match s {
            Stmt::Let { target, expr } => format!("{}={}", self.lv(target)?, self.expr(expr, 0)?),
            Stmt::Print { items } => format!("PRINT {}", self.items(items)?).trim_end().to_string(),
            Stmt::PrintTo { chan, items } => format!("PRINT #{};{}", self.expr(chan, 0)?, self.items(items)?),
            Stmt::Input { prompt, vars } => {
                let p = match prompt { Some(b) => format!("\"{}\";", self.text(b)?), None => String::new() };
                format!("INPUT {p}{}", vars.iter().map(|v| self.lv(v)).collect::<Result<Vec<_>, _>>()?.join(","))
            }
            Stmt::Get(l) => format!("GET {}", self.lv(l)?),
            Stmt::Goto(n) => format!("GOTO {n}"),
            Stmt::Gosub(n) => format!("GOSUB {n}"),
            Stmt::Return => "RETURN".into(),
            Stmt::OnGoto { sel, targets } => format!("ON {} GOTO {}", self.expr(sel, 0)?, targets.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")),
            Stmt::OnGosub { sel, targets } => format!("ON {} GOSUB {}", self.expr(sel, 0)?, targets.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(",")),
            Stmt::If { cond, then } => format!("IF {} THEN {}", self.expr(cond, 0)?, self.stmts(then)?),
            Stmt::For { var, from, to, step } => format!("FOR {}={} TO {}{}", Self::var(var), self.expr(from, 0)?, self.expr(to, 0)?, match step { Some(e) => format!(" STEP {}", self.expr(e, 0)?), None => String::new() }),
            Stmt::Next(v) => format!("NEXT {}", v.iter().map(Self::var).collect::<Vec<_>>().join(",")).trim_end().to_string(),
            Stmt::Dim(l) => format!("DIM {}", l.iter().map(|(v, d)| Ok(format!("{}({})", Self::var(v), self.list(d)?))).collect::<Result<Vec<_>, String>>()?.join(",")),
            Stmt::Read(v) => format!("READ {}", v.iter().map(|x| self.lv(x)).collect::<Result<Vec<_>, _>>()?.join(",")),
            Stmt::Data(items) => format!("DATA {}", items.iter().map(|i| Ok(match i {
                DataItem::Num(n) => n.to_string(),
                DataItem::Float(f) => f.clone(),
                DataItem::Str(b) => format!("\"{}\"", self.text(b)?),
                DataItem::Raw(b) => self.text(b)?,
            })).collect::<Result<Vec<_>, String>>()?.join(",")),
            Stmt::Restore(t) => match t { Some(n) => format!("RESTORE {n}"), None => "RESTORE".into() },
            Stmt::Poke(a, b) => format!("POKE {},{}", self.expr(a, 0)?, self.expr(b, 0)?),
            Stmt::Sys(e) => format!("SYS {}", self.expr(e, 0)?),
            Stmt::Wait(a, b, c) => format!("WAIT {},{}{}", self.expr(a, 0)?, self.expr(b, 0)?, match c { Some(e) => format!(",{}", self.expr(e, 0)?), None => String::new() }),
            Stmt::Rem(b) => format!("REM{}", self.text(b)?),
            Stmt::End => "END".into(),
            Stmt::Stop => "STOP".into(),
            Stmt::Clr => "CLR".into(),
            Stmt::Run(t) => match t { Some(n) => format!("RUN {n}"), None => "RUN".into() },
            Stmt::DefFn { name, param, body } => format!("DEF FN{name}({})={}", Self::var(param), self.expr(body, 0)?),
            Stmt::Atari { op, args } => format!("{op} {}", self.list(args)?),
            Stmt::Turbo(t) => match t {
                Struct::If(c) => format!("IF {}", self.expr(c, 0)?),
                Struct::Else => "ELSE".into(), Struct::Endif => "ENDIF".into(),
                Struct::While(c) => format!("WHILE {}", self.expr(c, 0)?), Struct::Wend => "WEND".into(),
                Struct::Repeat => "REPEAT".into(), Struct::Until(c) => format!("UNTIL {}", self.expr(c, 0)?),
                Struct::Do => "DO".into(), Struct::Loop => "LOOP".into(), Struct::Exit => "EXIT".into(),
                Struct::Proc(n) => format!("PROC {n}"), Struct::Endproc => "ENDPROC".into(), Struct::Exec(n) => format!("EXEC {n}"), Struct::OnExec(e, n) => format!("ON {} EXEC {}", self.expr(e, 0)?, n.join(",")),
            },
            Stmt::GetCh(c, l) => format!("GET #{},{}", self.expr(c, 0)?, self.lv(l)?),
            Stmt::Locate(x, y, l) => format!("LOCATE {},{},{}", self.expr(x, 0)?, self.expr(y, 0)?, self.lv(l)?),
            Stmt::Unsupported(k) => return Err(format!("{k} lässt sich nicht zurückschreiben (nicht übersetzte Anweisung)")),
        })
    }
}

/// Kanonischer Text des Programms (eine Zeile je BASIC-Zeile, ohne Leerzeichen außer vor Schlüsselwörtern). `atari`: Atari-Regeln (Rang, ASCII-Texte)
pub fn to_text(prog: &Program, d: &Dialect, atari: bool) -> Result<String, String> {
    let w = W { d, atari };
    let mut out = String::new();
    for l in &prog.lines { out.push_str(&format!("{} {}\n", l.number, w.stmts(&l.stmts).map_err(|e| format!("Zeile {}: {e}", l.number))?)); }
    Ok(out)
}

fn expr_json(e: &Expr) -> Value {
    match e {
        Expr::Num(n) => json!({"num": n}),
        Expr::Float(f) => json!({"float": f}),
        Expr::Str(b) => json!({"str": b}),
        Expr::Var(v) => json!({"var": var_json(v)}),
        Expr::Elem(v, a) => json!({"elem": var_json(v), "index": a.iter().map(expr_json).collect::<Vec<_>>()}),
        Expr::Un(op, x) => json!({"un": format!("{op:?}"), "arg": expr_json(x)}),
        Expr::Bin(op, a, b) => json!({"bin": format!("{op:?}"), "left": expr_json(a), "right": expr_json(b)}),
        Expr::Call(n, a) => json!({"call": n, "args": a.iter().map(expr_json).collect::<Vec<_>>()}),
        Expr::Fn(n, a) => json!({"fn": n, "arg": expr_json(a)}),
    }
}
fn var_json(v: &Var) -> Value { json!({"name": v.name, "type": format!("{:?}", v.ty)}) }
fn lv_json(l: &LValue) -> Value { match l { LValue::Var(v) => json!({"var": var_json(v)}), LValue::Elem(v, a) => json!({"elem": var_json(v), "index": a.iter().map(expr_json).collect::<Vec<_>>()}) } }
fn item_json(i: &PrintItem) -> Value {
    match i { PrintItem::Expr(e) => json!({"expr": expr_json(e)}), PrintItem::Tab(e) => json!({"tab": expr_json(e)}), PrintItem::Spc(e) => json!({"spc": expr_json(e)}), PrintItem::Semi => json!("semi"), PrintItem::Comma => json!("comma") }
}
fn stmt_json(s: &Stmt) -> Value {
    let es = |v: &[Expr]| v.iter().map(expr_json).collect::<Vec<_>>();
    match s {
        Stmt::Let { target, expr } => json!({"stmt": "let", "target": lv_json(target), "expr": expr_json(expr)}),
        Stmt::Print { items } => json!({"stmt": "print", "items": items.iter().map(item_json).collect::<Vec<_>>()}),
        Stmt::PrintTo { chan, items } => json!({"stmt": "print_to", "channel": expr_json(chan), "items": items.iter().map(item_json).collect::<Vec<_>>()}),
        Stmt::Input { prompt, vars } => json!({"stmt": "input", "prompt": prompt, "vars": vars.iter().map(lv_json).collect::<Vec<_>>()}),
        Stmt::Get(l) => json!({"stmt": "get", "target": lv_json(l)}),
        Stmt::Goto(n) => json!({"stmt": "goto", "line": n}),
        Stmt::Gosub(n) => json!({"stmt": "gosub", "line": n}),
        Stmt::Return => json!({"stmt": "return"}),
        Stmt::OnGoto { sel, targets } => json!({"stmt": "on_goto", "selector": expr_json(sel), "lines": targets}),
        Stmt::OnGosub { sel, targets } => json!({"stmt": "on_gosub", "selector": expr_json(sel), "lines": targets}),
        Stmt::If { cond, then } => json!({"stmt": "if", "cond": expr_json(cond), "then": then.iter().map(stmt_json).collect::<Vec<_>>()}),
        Stmt::For { var, from, to, step } => json!({"stmt": "for", "var": var_json(var), "from": expr_json(from), "to": expr_json(to), "step": step.as_ref().map(expr_json)}),
        Stmt::Next(v) => json!({"stmt": "next", "vars": v.iter().map(var_json).collect::<Vec<_>>()}),
        Stmt::Dim(l) => json!({"stmt": "dim", "arrays": l.iter().map(|(v, d)| json!({"var": var_json(v), "dims": es(d)})).collect::<Vec<_>>()}),
        Stmt::Read(v) => json!({"stmt": "read", "targets": v.iter().map(lv_json).collect::<Vec<_>>()}),
        Stmt::Data(items) => json!({"stmt": "data", "items": items.iter().map(|i| match i { DataItem::Num(n) => json!({"num": n}), DataItem::Float(f) => json!({"float": f}), DataItem::Str(b) => json!({"str": b}), DataItem::Raw(b) => json!({"raw": b}) }).collect::<Vec<_>>()}),
        Stmt::Restore(t) => json!({"stmt": "restore", "line": t}),
        Stmt::Poke(a, b) => json!({"stmt": "poke", "args": es(&[a.clone(), b.clone()])}),
        Stmt::Sys(e) => json!({"stmt": "sys", "arg": expr_json(e)}),
        Stmt::Wait(a, b, c) => json!({"stmt": "wait", "args": es(&[a.clone(), b.clone()]), "xor": c.as_ref().map(expr_json)}),
        Stmt::Rem(b) => json!({"stmt": "rem", "text": b}),
        Stmt::End => json!({"stmt": "end"}),
        Stmt::Stop => json!({"stmt": "stop"}),
        Stmt::Clr => json!({"stmt": "clr"}),
        Stmt::Run(t) => json!({"stmt": "run", "line": t}),
        Stmt::DefFn { name, param, body } => json!({"stmt": "def_fn", "name": name, "param": var_json(param), "body": expr_json(body)}),
        Stmt::Atari { op, args } => json!({"stmt": "atari", "op": op, "args": es(args)}),
        Stmt::Turbo(t) => json!({"stmt": "turbo", "block": match t {
            Struct::If(c) => json!({"if": expr_json(c)}), Struct::Else => json!("else"), Struct::Endif => json!("endif"),
            Struct::While(c) => json!({"while": expr_json(c)}), Struct::Wend => json!("wend"),
            Struct::Repeat => json!("repeat"), Struct::Until(c) => json!({"until": expr_json(c)}),
            Struct::Do => json!("do"), Struct::Loop => json!("loop"), Struct::Exit => json!("exit"),
            Struct::Proc(n) => json!({"proc": n}), Struct::Endproc => json!("endproc"), Struct::Exec(n) => json!({"exec": n}), Struct::OnExec(e, n) => json!({"on_exec": expr_json(e), "procs": n}),
        }}),
        Stmt::GetCh(c, l) => json!({"stmt": "get_ch", "channel": expr_json(c), "target": lv_json(l)}),
        Stmt::Locate(x, y, l) => json!({"stmt": "locate", "x": expr_json(x), "y": expr_json(y), "target": lv_json(l)}),
        Stmt::Unsupported(k) => json!({"stmt": "unsupported", "word": k}),
    }
}

/// Syntaxbaum als JSON: `{"lines": [{"number", "line", "text", "stmts": [...]}]}`; Zeichenketten als Byteliste des Zielzeichensatzes
pub fn to_json(prog: &Program) -> Value {
    json!({"lines": prog.lines.iter().map(|l| json!({"number": l.number, "line": l.src_line, "text": l.text, "stmts": l.stmts.iter().map(stmt_json).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}
