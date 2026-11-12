// SPDX-License-Identifier: GPL-3.0-or-later
//! Testmodell: Lesen der `*.vctest` und Ergebnisse. Ein Ergebnis kennt drei Zustände (bestanden, fehlgeschlagen,
//! Zeitüberschreitung, übersprungen); „kann ich nicht messen“ ist nie „bestanden“.
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State { Passed, Failed, TimedOut, Skipped }

/// Gewicht eines Fehlschlags: `Error` bricht `vcc test` ab (Exit-Code), `Hint` wird nur gemeldet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level { Error, Hint }

/// Register der CPU bei der Rückkehr aus dem Unterprogramm (Trap-Modus)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegSnap { pub pc: u16, pub a: u8, pub x: u8, pub y: u8, pub sp: u8, pub flags: u8 }

#[derive(Debug, Clone)]
pub struct Outcome {
    pub name: String,
    pub state: State,
    pub level: Level,
    /// Begründung: bei Fehlschlag Erwartung und Ist-Wert, bei „übersprungen“ der Grund
    pub detail: String,
    /// Messwert zur Anzeige (z. B. `$1F3C` oder `48 bis 52`)
    pub measured: Option<String>,
    /// Quelltextstelle (Datei, Zeile), falls bekannt
    pub location: Option<(String, usize)>,
    /// Registerstand bei der Rückkehr (nur Tests im Emulator)
    pub regs: Option<RegSnap>,
}

impl Outcome {
    pub fn passed(name: impl Into<String>, measured: Option<String>) -> Outcome { Outcome { name: name.into(), state: State::Passed, level: Level::Error, detail: String::new(), measured, location: None, regs: None } }
    pub fn failed(name: impl Into<String>, level: Level, detail: impl Into<String>) -> Outcome { Outcome { name: name.into(), state: State::Failed, level, detail: detail.into(), measured: None, location: None, regs: None } }
    pub fn skipped(name: impl Into<String>, reason: impl Into<String>) -> Outcome { Outcome { name: name.into(), state: State::Skipped, level: Level::Error, detail: reason.into(), measured: None, location: None, regs: None } }
    pub fn with_regs(mut self, r: RegSnap) -> Outcome { self.regs = Some(r); self }
    pub fn at(mut self, file: &str, line: usize) -> Outcome { self.location = Some((file.to_string(), line)); self }
    pub fn with_measured(mut self, m: impl Into<String>) -> Outcome { self.measured = Some(m.into()); self }
    /// Zählt für den Exit-Code von `vcc test`
    pub fn is_error(&self) -> bool { matches!(self.state, State::Failed | State::TimedOut) && self.level == Level::Error }
    pub fn timed_out(name: impl Into<String>, detail: impl Into<String>) -> Outcome { Outcome { name: name.into(), state: State::TimedOut, level: Level::Error, detail: detail.into(), measured: None, location: None, regs: None } }
}

/// Eine statische Prüfung aus dem Teil `static` der Testdatei. Zahlen und Ausdrücke bleiben als Text stehen und werden
/// erst bei der Prüfung ausgewertet (Symbole der `.vcdbg`, `size("segment")`).
#[derive(Debug, Clone, PartialEq)]
pub enum StaticCheck {
    /// Größe (Bytes) eines Segments oder aller Segmente; `max_end`: Adresse hinter dem letzten Byte
    Size { segment: Option<String>, min: Option<String>, max: Option<String>, max_end: Option<String> },
    /// Verbotene Adressbereiche (`$02-$1f`): Segmente dürfen sie nicht belegen, Befehle sie nicht ansprechen
    Zeropage { forbid: Vec<String> },
    Overlap,
    /// Zyklenspanne ab einer Marke (`label`) oder aus einem Zyklenblock (`block`, `.cycles name` im Quelltext)
    Cycles { label: Option<String>, block: Option<String>, max: Option<String>, level: Option<Level> },
    /// Beliebiger Ausdruck (Sprache der bedingten Haltepunkte), z. B. `size("Main") <= $2000`
    Expr { expr: String, name: Option<String> },
}

/// Art der Tests einer Datei: Trap-Modus (der Läufer ruft Unterprogramme auf) oder Postfach-Modus (das Programm prüft sich selbst)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestMode { #[default] Trap, Mailbox, Measure }

impl TestMode { pub fn label(self) -> &'static str { match self { TestMode::Trap => "Trap", TestMode::Mailbox => "Postfach", TestMode::Measure => "Zyklen" } } }

#[derive(Debug, Clone)]
pub struct TestFile {
    pub path: PathBuf,
    pub target: String,
    /// Programmdatei wie in der Testdatei geschrieben (relativ zur Testdatei oder zu einem übergeordneten Ordner)
    pub program: String,
    /// Tests für den Emulator (Trap-Modus): werden gelesen, aber erst mit T2 ausgeführt
    pub tests: Vec<Value>,
    /// Jeder Test startet den Emulator neu (langsam, aber sauber); sonst ein Start je Datei
    pub isolate: bool,
    /// `"mode"`: `trap` (Standard) oder `mailbox`
    pub mode: TestMode,
    /// Adresse der Falle im freien RAM (Ausdruck); sonst Standard je Ziel
    pub trap: Option<String>,
    pub statics: Vec<StaticCheck>,
}

#[derive(Debug, Clone)]
pub struct Report { pub file: PathBuf, pub target: String, pub program: String, pub outcomes: Vec<Outcome> }

impl Report {
    pub fn count(&self, s: State) -> usize { self.outcomes.iter().filter(|o| o.state == s).count() }
    pub fn errors(&self) -> usize { self.outcomes.iter().filter(|o| o.is_error()).count() }
}

/// Zahl oder Text aus der Testdatei als Ausdruckstext (`63` und `"$3f"` sind erlaubt)
fn expr_text(v: &Value, what: &str) -> Result<String, String> {
    match v { Value::Number(n) => Ok(n.to_string()), Value::String(s) if !s.trim().is_empty() => Ok(s.clone()), _ => Err(format!("{what}: Zahl oder Ausdruck als Text erwartet")) }
}
fn opt_expr(o: &Value, key: &str) -> Result<Option<String>, String> { o.get(key).map(|v| expr_text(v, key)).transpose() }
fn opt_str(o: &Value, key: &str) -> Result<Option<String>, String> {
    match o.get(key) { None | Some(Value::Null) => Ok(None), Some(Value::String(s)) => Ok(Some(s.clone())), _ => Err(format!("{key}: Text erwartet")) }
}

fn parse_check(i: usize, o: &Value) -> Result<StaticCheck, String> {
    let ctx = |e: String| format!("static[{i}]: {e}");
    let kind = o.get("check").and_then(Value::as_str).ok_or_else(|| ctx("\"check\" fehlt (size, zeropage, overlap, cycles, expr)".into()))?;
    Ok(match kind {
        "size" => StaticCheck::Size { segment: opt_str(o, "segment").map_err(ctx)?, min: opt_expr(o, "min").map_err(ctx)?, max: opt_expr(o, "max").map_err(ctx)?, max_end: opt_expr(o, "maxEnd").map_err(ctx)? },
        "zeropage" => {
            let forbid = match o.get("forbid") {
                Some(Value::String(s)) => s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
                Some(Value::Array(a)) => a.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(|| ctx("forbid: Texte erwartet".into()))).collect::<Result<Vec<_>, _>>()?,
                _ => return Err(ctx("zeropage braucht \"forbid\": \"$02-$1f\" (Bereich oder Liste)".into())),
            };
            if forbid.is_empty() { return Err(ctx("forbid ist leer".into())); }
            StaticCheck::Zeropage { forbid }
        }
        "overlap" => StaticCheck::Overlap,
        "cycles" => {
            let (label, block) = (opt_str(o, "label").map_err(ctx)?, opt_str(o, "block").map_err(ctx)?);
            if label.is_none() == block.is_none() { return Err(ctx("cycles braucht genau eines von \"label\" und \"block\"".into())); }
            let level = match opt_str(o, "level").map_err(ctx)?.as_deref() { None => None, Some("hint") => Some(Level::Hint), Some("error") => Some(Level::Error), Some(x) => return Err(ctx(format!("level: \"hint\" oder \"error\", nicht \"{x}\""))) };
            StaticCheck::Cycles { label, block, max: opt_expr(o, "max").map_err(ctx)?, level }
        }
        "expr" => StaticCheck::Expr { expr: o.get("expr").and_then(Value::as_str).ok_or_else(|| ctx("expr: Ausdruck als Text fehlt".into()))?.to_string(), name: opt_str(o, "name").map_err(ctx)? },
        x => return Err(ctx(format!("unbekannte Prüfung \"{x}\" (size, zeropage, overlap, cycles, expr)"))),
    })
}

impl TestFile {
    pub fn load(path: &Path) -> Result<TestFile, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(path, &text).map_err(|e| format!("{}: {e}", path.display()))
    }
    pub fn parse(path: &Path, text: &str) -> Result<TestFile, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("kein gültiges JSON ({e})"))?;
        if let Some(k) = v.get("kind").and_then(Value::as_str) { if k != "test" { return Err(format!("\"kind\" ist \"{k}\", erwartet \"test\"")); } }
        let target = v.get("target").and_then(Value::as_str).ok_or("\"target\" fehlt (z. B. \"c64\")")?.to_string();
        let program = v.get("program").and_then(Value::as_str).ok_or("\"program\" fehlt (z. B. \"build/math.prg\")")?.to_string();
        let tests = match v.get("tests") { None | Some(Value::Null) => vec![], Some(Value::Array(a)) => a.clone(), _ => return Err("\"tests\" muss eine Liste sein".into()) };
        let statics = match v.get("static") {
            None | Some(Value::Null) => vec![],
            Some(Value::Array(a)) => a.iter().enumerate().map(|(i, o)| parse_check(i, o)).collect::<Result<Vec<_>, _>>()?,
            _ => return Err("\"static\" muss eine Liste sein".into()),
        };
        let isolate = v.get("isolate").and_then(Value::as_bool).unwrap_or(false);
        let trap = opt_expr(&v, "trap")?;
        let mode = match v.get("mode").and_then(Value::as_str) { None | Some("trap") => TestMode::Trap, Some("mailbox") => TestMode::Mailbox, Some("measure") => TestMode::Measure, Some(x) => return Err(format!("\"mode\": \"trap\", \"mailbox\" oder \"measure\", nicht \"{x}\"")) };
        Ok(TestFile { path: path.to_path_buf(), target, program, tests, isolate, mode, trap, statics })
    }
}

/// Alle `*.vctest` unter `dir` (sortiert). Übersprungen werden `build`, `target`, `.git`, `node_modules` und versteckte Ordner.
pub fn find_tests(dir: &Path) -> Vec<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if p.is_dir() { if !name.starts_with('.') && !["build", "target", "node_modules"].contains(&name.as_str()) { walk(&p, out); } }
            else if name.ends_with(".vctest") { out.push(p); }
        }
    }
    let mut v = vec![];
    if dir.is_file() { v.push(dir.to_path_buf()); } else { walk(dir, &mut v); }
    v.sort();
    v
}

/// Ein Test für den Emulator (Trap-Modus): Setup, Aufruf, Erwartungen, Zyklenbudget
#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub name: String,
    /// Speicher vor dem Aufruf: Adresse → Bytes (Ausdrücke)
    pub mem: Vec<(String, Vec<String>)>,
    /// Register vor dem Aufruf: `a x y sp p` → Ausdruck
    pub regs: Vec<(String, String)>,
    /// Einzelne Flags vor dem Aufruf (`fn fv fd fi fz fc`)
    pub flags: Vec<(String, bool)>,
    /// Marke oder Adresse des Unterprogramms
    pub call: String,
    /// Das Unterprogramm endet mit `rti` statt `rts` (Interruptroutinen): die Falle legt einen Rücksprungrahmen auf den Stapel
    pub rti: bool,
    pub expect: Vec<String>,
    /// Höchstzahl Zyklen des Unterprogramms (`"cycles": { "max": 220 }`)
    pub cycles_max: Option<String>,
    pub level: Option<Level>,
    pub timeout_ms: u64,
}

impl TestCase {
    pub fn parse(i: usize, t: &Value) -> Result<TestCase, String> {
        let name = t.get("name").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("Test {}", i + 1));
        let err = |e: String| format!("{name}: {e}");
        let call = t.get("call").and_then(Value::as_str).ok_or_else(|| err("\"call\" fehlt (Marke oder Adresse)".into()))?.to_string();
        let (mut mem, mut regs, mut flags) = (vec![], vec![], vec![]);
        if let Some(setup) = t.get("setup") {
            let o = setup.as_object().ok_or_else(|| err("\"setup\" muss ein Objekt sein".into()))?;
            for (k, v) in o {
                match k.as_str() {
                    "mem" => for (addr, val) in v.as_object().ok_or_else(|| err("setup.mem muss ein Objekt sein".into()))? {
                        let bytes = match val { Value::Array(a) => a.iter().map(|x| expr_text(x, "setup.mem")).collect::<Result<Vec<_>, _>>(), x => expr_text(x, "setup.mem").map(|s| vec![s]) }.map_err(err)?;
                        mem.push((addr.clone(), bytes));
                    },
                    "a" | "x" | "y" | "sp" | "p" => regs.push((k.clone(), expr_text(v, k).map_err(err)?)),
                    "fn" | "fv" | "fd" | "fi" | "fz" | "fc" => flags.push((k.clone(), match v { Value::Bool(b) => *b, Value::Number(n) => n.as_i64() != Some(0), _ => return Err(err(format!("{k}: 0/1 oder true/false erwartet"))) })),
                    other => return Err(err(format!("setup.{other} unbekannt (mem, a, x, y, sp, p, fn fv fd fi fz fc)"))),
                }
            }
        }
        let rti = match t.get("return").and_then(Value::as_str) { None | Some("rts") => false, Some("rti") => true, Some(x) => return Err(err(format!("return: \"rts\" oder \"rti\", nicht \"{x}\""))) };
        let expect = match t.get("expect") { None | Some(Value::Null) => vec![], Some(Value::Array(a)) => a.iter().map(|x| x.as_str().map(str::to_string).ok_or_else(|| err("expect: Texte erwartet".into()))).collect::<Result<Vec<_>, _>>()?, _ => return Err(err("expect muss eine Liste von Ausdrücken sein".into())) };
        let (mut cycles_max, mut level) = (None, None);
        if let Some(c) = t.get("cycles") {
            cycles_max = opt_expr(c, "max").map_err(err)?;
            level = match opt_str(c, "level").map_err(err)?.as_deref() { None => None, Some("hint") => Some(Level::Hint), Some("error") => Some(Level::Error), Some(x) => return Err(err(format!("cycles.level: \"hint\" oder \"error\", nicht \"{x}\""))) };
        }
        if expect.is_empty() && cycles_max.is_none() { return Err(err("weder \"expect\" noch \"cycles\": der Test prüft nichts".into())); }
        let timeout_ms = match t.get("timeoutMs") { None => 2000, Some(v) => v.as_u64().filter(|x| *x > 0).ok_or_else(|| err("timeoutMs: positive Zahl erwartet".into()))? };
        Ok(TestCase { name, mem, regs, flags, call, rti, expect, cycles_max, level, timeout_ms })
    }
}

/// Ein Lauf im Postfach-Modus: das Programm (mit `test_begin` … `test_end` aus `test.inc`) wird gestartet und prüft sich selbst
#[derive(Debug, Clone, PartialEq)]
pub struct MailboxCase {
    pub name: String,
    /// Zusätzliche Erwartungen nach dem Ende (Speicher, Register), Sprache der bedingten Haltepunkte
    pub expect: Vec<String>,
    /// Zeitlimit für Hochfahren, Laden und Lauf zusammen
    pub timeout_ms: u64,
}

impl MailboxCase {
    pub fn parse(i: usize, t: &Value) -> Result<MailboxCase, String> {
        let name = t.get("name").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("Postfach-Test {}", i + 1));
        let err = |e: String| format!("{name}: {e}");
        let expect = match t.get("expect") { None | Some(Value::Null) => vec![], Some(Value::Array(a)) => a.iter().map(|x| x.as_str().map(str::to_string).ok_or_else(|| err("expect: Texte erwartet".into()))).collect::<Result<Vec<_>, _>>()?, _ => return Err(err("expect muss eine Liste von Ausdrücken sein".into())) };
        let timeout_ms = match t.get("timeoutMs") { None => 15000, Some(v) => v.as_u64().filter(|x| *x > 0).ok_or_else(|| err("timeoutMs: positive Zahl erwartet".into()))? };
        Ok(MailboxCase { name, expect, timeout_ms })
    }
}

/// Eine Zyklenmessung auf der Hardware (`"mode": "measure"`): Zyklen vom ersten Zugriff auf `from` bis zum ersten danach auf `to`,
/// gegen ein Budget (`max`) und/oder eine Untergrenze (`min`). Gemessen wird mit dem Debug-Strom der Ultimate 64 (Modus „6510 Only“).
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureCase {
    pub name: String,
    /// Marke oder Adresse des Starts (Ausdruck, Symbole der `.vcdbg`)
    pub from: String,
    pub to: String,
    pub max: Option<String>,
    pub min: Option<String>,
    /// Gewicht einer Überschreitung: Standard `error` (ein Urteil), `hint` meldet nur
    pub level: Level,
    /// Zeitlimit für Reset, Laden, Lauf und Messung zusammen
    pub timeout_ms: u64,
}

impl MeasureCase {
    pub fn parse(i: usize, t: &Value) -> Result<MeasureCase, String> {
        let name = t.get("name").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("Messung {}", i + 1));
        let err = |e: String| format!("{name}: {e}");
        let from = t.get("from").ok_or_else(|| err("\"from\" fehlt (Marke oder Adresse)".into())).and_then(|v| expr_text(v, "from").map_err(err))?;
        let to = t.get("to").ok_or_else(|| err("\"to\" fehlt (Marke oder Adresse)".into())).and_then(|v| expr_text(v, "to").map_err(err))?;
        let (max, min) = (opt_expr(t, "max").map_err(err)?, opt_expr(t, "min").map_err(err)?);
        if max.is_none() && min.is_none() { return Err(err("weder \"max\" noch \"min\": die Messung prüft nichts".into())); }
        let level = match opt_str(t, "level").map_err(err)?.as_deref() { None | Some("error") => Level::Error, Some("hint") => Level::Hint, Some(x) => return Err(err(format!("level: \"error\" oder \"hint\", nicht \"{x}\""))) };
        let timeout_ms = match t.get("timeoutMs") { None => 20000, Some(v) => v.as_u64().filter(|x| *x > 0).ok_or_else(|| err("timeoutMs: positive Zahl erwartet".into()))? };
        Ok(MeasureCase { name, from, to, max, min, level, timeout_ms })
    }
}
