// SPDX-License-Identifier: GPL-3.0-or-later
//! Gemeinsame Grundtypen: Diagnostics.
use std::fmt;

pub mod paths;
pub mod userdirs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub file: String,
    pub line: usize,
}

impl Diagnostic {
    pub fn error(code: &'static str, file: &str, line: usize, message: impl Into<String>) -> Self {
        Self { severity: Severity::Error, code, message: message.into(), file: file.into(), line }
    }
    pub fn warning(code: &'static str, file: &str, line: usize, message: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, code, message: message.into(), file: file.into(), line }
    }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
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
