// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeilennummerierung: Start und Schrittweite (Standard 10/10). Quellen in dieser Rangfolge: Direktive `#numbering start=N step=N` in der Datei
//! → Projekteinstellung (`.vcprj`, Abschnitt `basic`: `start`, `step`; die Kommandozeile `--start`/`--step` überschreibt die Projektvorgabe)
//! → Standard. Wie bei `#dialect` schlägt die Datei das Projekt.
//!
//! Schnittstelle für die Oberfläche: [`Numbering`] (Lesen/Schreiben der Werte: [`Numbering::from_json`], [`Numbering::to_json`] auf dem
//! Objekt `basic` der Projektdatei), [`resolve`] (Wert für eine Datei), [`Numbering::validate`] (Wertebereich), [`directive_line`] (Zeile erzeugen).
use serde_json::{json, Value};
use vc_core::{msg, Diagnostic};

/// Start und Schrittweite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numbering { pub start: u32, pub step: u32 }

impl Default for Numbering { fn default() -> Self { Numbering { start: 10, step: 10 } } }

impl Numbering {
    pub fn new(start: u32, step: u32) -> Self { Numbering { start, step } }
    /// Liest `start` und `step` aus dem Objekt `basic` der Projektdatei; fehlende oder unlesbare Werte bleiben beim Standard
    pub fn from_json(basic: &Value) -> Numbering {
        let d = Numbering::default();
        Numbering { start: basic["start"].as_u64().map_or(d.start, |v| v.min(u32::MAX as u64) as u32), step: basic["step"].as_u64().map_or(d.step, |v| v.min(u32::MAX as u64) as u32) }
    }
    /// Werte für das Objekt `basic` der Projektdatei
    pub fn to_json(&self) -> Value { json!({ "start": self.start, "step": self.step }) }
    /// Wertebereich: Schrittweite ≥ 1, Start ≤ `max_line` (größte Nummer des Dialekts: 63999 bzw. 32767)
    pub fn validate(&self, max_line: u32, file: &str, line: usize) -> Result<(), Diagnostic> {
        if self.step < 1 || self.start > max_line { return Err(Diagnostic::error("B191", file, line, msg!("diag-basic-b191-numbering-range", start = self.start, step = self.step, max = max_line))); }
        Ok(())
    }
    /// Reichen die Nummern für `count` Zeilen? Sonst `B192` mit einem Vorschlag für eine kleinere Schrittweite
    pub fn check_fit(&self, count: usize, max_line: u32, file: &str, line: usize) -> Result<(), Diagnostic> {
        if count == 0 { return Ok(()); }
        let last = self.start as u64 + (count as u64 - 1) * self.step as u64;
        if last <= max_line as u64 { return Ok(()); }
        let suggestion = if count > 1 { ((max_line - self.start.min(max_line)) as u64 / (count as u64 - 1)).max(1) } else { 1 };
        Err(Diagnostic::error("B192", file, line, msg!("diag-basic-b192-numbering-overflow", count = count, last = last, max = max_line, suggestion = suggestion)))
    }
}

/// Zeile `#numbering [start=N] [step=N]`: Teilwerte und Zeile; `Err(())` bei ungültiger Form
fn parse(line: &str) -> Option<Result<(Option<u32>, Option<u32>), ()>> {
    let t = line.trim();
    if !t.get(..10).map_or(false, |h| h.eq_ignore_ascii_case("#numbering")) { return None; }
    let rest = &t[10..];
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) { return None; }
    let (mut start, mut step) = (None, None);
    for w in rest.split_whitespace() {
        let Some((k, v)) = w.split_once('=') else { return Some(Err(())) };
        let Ok(n) = v.parse::<u32>() else { return Some(Err(())) };
        match k.to_ascii_lowercase().as_str() { "start" => start = Some(n), "step" => step = Some(n), _ => return Some(Err(())) }
    }
    if start.is_none() && step.is_none() { return Some(Err(())); }
    Some(Ok((start, step)))
}

/// Ist die Zeile die Richtlinie `#numbering …`?
pub fn is_directive(line: &str) -> bool { parse(line).is_some() }

/// Die Zeile für die Direktive (z. B. für „in Datei festlegen“ der Oberfläche)
pub fn directive_line(n: Numbering) -> String { format!("#numbering start={} step={}", n.start, n.step) }

/// Start und Schrittweite für diese Quelle: `#numbering` der Datei, sonst `project` (Teilangaben ergänzen den Projektwert). Prüft den Wertebereich;
/// ungültige Direktive → `B190`.
pub fn resolve(src: &str, file: &str, project: Numbering, max_line: u32) -> Result<Numbering, Vec<Diagnostic>> {
    let mut n = project;
    let mut at = 1;
    for (i, l) in src.lines().enumerate() {
        match parse(l) {
            None => {}
            Some(Err(())) => return Err(vec![Diagnostic::error("B190", file, i + 1, msg!("diag-basic-b190-numbering-syntax", text = l.trim()))]),
            Some(Ok((s, p))) => { if let Some(s) = s { n.start = s; } if let Some(p) = p { n.step = p; } at = i + 1; break; }
        }
    }
    n.validate(max_line, file, at).map_err(|d| vec![d])?;
    Ok(n)
}
