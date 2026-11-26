// SPDX-License-Identifier: GPL-3.0-or-later
//! Gemeinsame Grundtypen: Diagnostics.
use std::fmt;

pub mod atomar;
pub mod chars;
pub mod msg;
pub mod paths;
pub mod textfile;
pub mod userdirs;

pub use msg::{Msg, MsgArg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Bereich einer Meldung in der Quelle (1-basiert; Spalte 0 = unbekannt/ganze Zeile)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Range {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Range {
    /// ganze Zeile
    pub fn line(line: usize) -> Range { Range { line, col: 0, end_line: line, end_col: 0 } }
}

/// Meldung von Assembler, BASIC und Werkzeugen. Maschinenlesbar: `code` (stabil), `key` (Katalogschlüssel), `args` (benannt), `range`.
/// `message` ist der daraus gebildete Text in der Sprache, die beim Entstehen galt (Deutsch, solange nichts anderes gewählt ist);
/// [`Diagnostic::text_in`] bildet ihn in jeder Sprache neu.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub key: &'static str,
    pub args: Vec<(&'static str, MsgArg)>,
    pub range: Range,
}

impl Diagnostic {
    fn make(severity: Severity, code: &'static str, file: &str, line: usize, m: Msg) -> Self {
        let message = m.text();
        Self { severity, code, message, file: file.into(), line, key: m.key, args: m.args, range: Range::line(line) }
    }
    pub fn error(code: &'static str, file: &str, line: usize, m: impl Into<Msg>) -> Self { Self::make(Severity::Error, code, file, line, m.into()) }
    pub fn warning(code: &'static str, file: &str, line: usize, m: impl Into<Msg>) -> Self { Self::make(Severity::Warning, code, file, line, m.into()) }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
    /// Bereich setzen (Spalten 1-basiert)
    pub fn with_range(mut self, range: Range) -> Self { self.range = range; self }
    /// Schlüssel und Parameter als [`Msg`]
    pub fn msg(&self) -> Msg { Msg { key: self.key, args: self.args.clone() } }
    /// Text in der Sprache `lang`
    pub fn text_in(&self, lang: vc_i18n::Lang) -> String { self.msg().text_in(lang) }
    /// Wert eines Parameters als Text (leer, wenn er fehlt) – für Tests und Werkzeuge
    pub fn arg_text(&self, name: &str) -> String { self.msg().arg_text(name) }
    /// Maschinenlesbar für `--json` und Werkzeuge: Schweregrad, `code`, `key`, `args`, Position, Bereich und Text in `lang`
    pub fn to_json(&self, lang: vc_i18n::Lang) -> serde_json::Value {
        let mut a = serde_json::Map::new();
        for (k, v) in &self.args { a.insert((*k).to_string(), v.to_json()); }
        serde_json::json!({
            "severity": if self.is_error() { "error" } else { "warning" }, "code": self.code, "key": self.key, "args": a,
            "message": self.text_in(lang), "file": self.file, "line": self.line,
            "range": { "line": self.range.line, "col": self.range.col, "end_line": self.range.end_line, "end_col": self.range.end_col },
        })
    }
}

impl Diagnostic {
    /// Mehrzeilige Darstellung mit Quellzeile (rustc-ähnlich).
    pub fn render(&self, source_line: Option<&str>) -> String {
        let kind = if self.is_error() { "error" } else { "warning" };
        let mut s = format!("{kind}[{}]: {}\n  --> {}:{}\n", self.code, self.message, self.file, self.line);
        if let Some(l) = source_line {
            let n = self.line.to_string();
            let pad = " ".repeat(n.len() - 1);
            s.push_str(&format!("{pad} |\n{n} | {}\n{pad} |\n", l.trim_end()).replace(&format!("{pad} |"), &format!("{pad} |")));
        }
        s
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_error() { "error" } else { "warning" };
        write!(f, "{}:{}: {}[{}]: {}", self.file, self.line, kind, self.code, self.message)
    }
}

/// Liest eine Datenbankdatei (`data/…`) und merkt sich den Inhalt für die Laufzeit des Prozesses.
/// Die Hardware-Datenbank ändert sich im Betrieb nicht; auf Netzlaufwerken kostet jeder Dateizugriff Millisekunden bis Zehntelsekunden.
pub fn read_static(path: &std::path::Path) -> std::io::Result<String> {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<std::collections::HashMap<std::path::PathBuf, String>>> = OnceLock::new();
    let c = CACHE.get_or_init(Default::default);
    if let Some(t) = c.lock().ok().and_then(|g| g.get(path).cloned()) { return Ok(t); }
    let t = std::fs::read_to_string(path)?;
    if let Ok(mut g) = c.lock() { g.insert(path.to_path_buf(), t.clone()); }
    Ok(t)
}
