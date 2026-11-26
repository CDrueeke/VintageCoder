// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialektmechanismus (Konzept „BASIC-Editor und Dialekte“, B8): Dialekte sind Daten (`data/basic/*.dialect.json`),
//! Vererbung über `base`, gemeinsame Schnittstelle [`AnyDialect`] vor den beiden Familien (Commodore: verkettete Zeilen, Atari: Speicherabbild),
//! Auswahl je Projekt und je Datei (`#dialect <id>`).
//!
//! Neue, **optionale** Felder der Dialektdatei: `name`, `kind` (`builtin`/`module`), `base`, `systems`, `jumps`, `runtimeLine`,
//! `quoteComment`, `provenance {source, checked, license}`, `syntax`, `help`. Alte Dateien bleiben gültig.
use crate::atari::AtariDialect;
use crate::dialect::Dialect;
use serde_json::Value;
use std::path::Path;
use vc_core::{msg, Diagnostic};

/// Tiefe der `base`-Kette (Schutz vor Schleifen)
const MAX_DEPTH: usize = 4;

/// Liest die Dialektdatei samt Vererbung: eigene `tokens` stehen vor denen der Basis (längere Wörter zuerst), `capabilities` werden
/// feldweise zusammengeführt, alle übrigen Felder der Datei überschreiben die der Basis.
pub fn load_json(data_dir: &Path, id: &str) -> Result<Value, String> { load_json_depth(data_dir, id, 0) }

fn load_json_depth(data_dir: &Path, id: &str, depth: usize) -> Result<Value, String> {
    let path = data_dir.join("basic").join(format!("{id}.dialect.json"));
    let text = vc_core::read_static(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let Some(base) = v["base"].as_str().map(String::from) else { return Ok(v) };
    if depth >= MAX_DEPTH || base == id { return Err(vc_i18n::t!("basic-dialect-base-cycle", id = id)); }
    let mut b = load_json_depth(data_dir, &base, depth + 1)?;
    let (own, bm) = (v.as_object_mut().unwrap(), b.as_object_mut().unwrap());
    let mut tokens: Vec<Value> = own.get("tokens").and_then(Value::as_array).cloned().unwrap_or_default();
    tokens.extend(bm.get("tokens").and_then(Value::as_array).cloned().unwrap_or_default());
    let mut caps = bm.get("capabilities").and_then(Value::as_object).cloned().unwrap_or_default();
    if let Some(c) = own.get("capabilities").and_then(Value::as_object) { for (k, x) in c { caps.insert(k.clone(), x.clone()); } }
    for (k, x) in own.iter() { if k != "tokens" && k != "capabilities" { bm.insert(k.clone(), x.clone()); } }
    bm.insert("tokens".into(), Value::Array(tokens));
    bm.insert("capabilities".into(), Value::Object(caps));
    bm.insert("id".into(), Value::String(id.to_string()));
    Ok(b)
}

/// Metadaten eines Dialekts (für Auswahl, Hilfe, Prüfung und Umwandlung), unabhängig von der Familie
#[derive(Debug, Clone, PartialEq)]
pub struct DialectInfo {
    pub id: String,
    /// Anzeigename (Eigenname, nicht übersetzt)
    pub name: String,
    /// `cbm` oder `atari`
    pub family: String,
    /// `builtin` oder `module`
    pub kind: String,
    pub base: Option<String>,
    /// zulässige Zielsysteme (Target-Kennungen); leer = alle
    pub systems: Vec<String>,
    /// Wörter, hinter denen Zeilennummern als Sprungziele stehen (`GOTO`, `THEN` …)
    pub jumps: Vec<String>,
    /// Wörter, die zur Laufzeit die Zeilennummer liefern (`EL`)
    pub runtime_line: Vec<String>,
    /// `'` leitet einen Kommentar ein (Atari, Turbo-BASIC XL)
    pub quote_comment: bool,
    pub max_line_len: usize,
    pub max_line_number: u32,
    pub load_addr: u16,
    pub provenance_source: String,
    pub provenance_checked: String,
}

fn strings(v: &Value) -> Vec<String> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }

/// Metadaten aus der (vererbten) Dialektdatei
pub fn info(data_dir: &Path, id: &str) -> Result<DialectInfo, String> {
    let v = load_json(data_dir, id)?;
    let family = v["family"].as_str().unwrap_or("cbm").to_string();
    let atari = family == "atari";
    let default_jumps: &[&str] = if atari { &["GOTO", "GOSUB", "THEN", "TRAP"] } else { &["GOTO", "GOSUB", "THEN", "RUN"] };
    Ok(DialectInfo {
        id: id.to_string(),
        name: v["name"].as_str().unwrap_or(id).to_string(),
        kind: v["kind"].as_str().unwrap_or("builtin").to_string(),
        base: v["base"].as_str().map(String::from),
        systems: strings(&v["systems"]),
        jumps: if v["jumps"].is_array() { strings(&v["jumps"]) } else { default_jumps.iter().map(|s| s.to_string()).collect() },
        runtime_line: strings(&v["runtimeLine"]),
        quote_comment: v["quoteComment"].as_bool().unwrap_or(atari),
        max_line_len: v["maxLineLen"].as_u64().unwrap_or(if atari { 120 } else { 80 }) as usize,
        max_line_number: v["maxLineNumber"].as_u64().unwrap_or(if atari { 32767 } else { 63999 }) as u32,
        load_addr: v["loadAddr"].as_str().and_then(vc_target::parse_num).unwrap_or(0x0801) as u16,
        provenance_source: v["provenance"]["source"].as_str().unwrap_or("").to_string(),
        provenance_checked: v["provenance"]["checked"].as_str().unwrap_or("").to_string(),
        family,
    })
}

/// Alle Dialekte im Datenordner (nach Kennung sortiert); Dateien, die sich nicht lesen lassen, fehlen
pub fn list(data_dir: &Path) -> Vec<DialectInfo> {
    let mut ids: Vec<String> = std::fs::read_dir(data_dir.join("basic")).into_iter().flatten().flatten()
        .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_suffix(".dialect.json")).map(String::from)).collect();
    ids.sort();
    ids.iter().filter_map(|id| info(data_dir, id).ok()).collect()
}

/// Ein geladener Dialekt, gleich welcher Familie: die gemeinsame Schnittstelle der beiden Rückseiten
pub enum AnyDialect { Cbm(Dialect), Atari(AtariDialect) }

impl AnyDialect {
    pub fn load(data_dir: &Path, id: &str) -> Result<AnyDialect, String> {
        let v = load_json(data_dir, id)?;
        if v["family"].as_str() == Some("atari") { AtariDialect::load(data_dir, id).map(AnyDialect::Atari) } else { Dialect::load(data_dir, id).map(AnyDialect::Cbm) }
    }
    pub fn id(&self) -> &str { match self { AnyDialect::Cbm(d) => &d.id, AnyDialect::Atari(d) => &d.id } }
    pub fn lexicon(&self) -> crate::numbers::Lexicon<'_> { match self { AnyDialect::Cbm(d) => crate::numbers::Lexicon::Cbm(d), AnyDialect::Atari(d) => crate::numbers::Lexicon::Atari(d) } }
    /// Alle Schlüsselwörter in Tabellenreihenfolge (für Färbung, Vervollständigung, Hover)
    pub fn keywords(&self) -> Vec<String> {
        match self {
            AnyDialect::Cbm(d) => d.tokens.iter().map(|(k, _)| k.clone()).collect(),
            AnyDialect::Atari(d) => d.statements.iter().chain(d.functions.iter()).map(|(k, _)| k.clone()).collect(),
        }
    }
    pub fn is_atari(&self) -> bool { matches!(self, AnyDialect::Atari(_)) }
}

/// Kopfzeile `#dialect <id>`: Kennung und Zeilennummer (1-basiert) der ersten solchen Zeile
pub fn dialect_directive(src: &str) -> Option<(String, usize)> {
    src.lines().enumerate().find_map(|(n, l)| directive_id(l).map(|id| (id.to_string(), n + 1)))
}

/// Kennung aus einer Zeile `#dialect <id>` (Groß-/Kleinschreibung der Direktive egal)
pub fn directive_id(line: &str) -> Option<&str> {
    let t = line.trim();
    let head = t.get(..8)?;
    if !head.eq_ignore_ascii_case("#dialect") { return None; }
    let rest = &t[8..];
    if !rest.starts_with(char::is_whitespace) { return None; }
    let id = rest.trim();
    if id.is_empty() { None } else { Some(id) }
}

/// Ist die Zeile die Richtlinie `#dialect …`?
pub fn is_directive(line: &str) -> bool { directive_id(line).is_some() }

/// Ergebnis der Dialektwahl
#[derive(Debug, Clone)]
pub struct Resolved { pub id: String, pub from_file: bool, pub warnings: Vec<Diagnostic> }

/// Wählt den Dialekt: `#dialect` der Datei überschreibt die Einstellung des Projekts. Unbekannte Kennung → `B178` (Fehler);
/// passt der Dialekt nicht zum Zielsystem (`system`, Target-Kennung) → `B170` (Warnung).
pub fn resolve(src: &str, file: &str, project_id: &str, system: Option<&str>, data_dir: &Path) -> Result<Resolved, Vec<Diagnostic>> {
    let (id, from_file, line) = match dialect_directive(src) { Some((id, n)) => (id, true, n), None => (project_id.to_string(), false, 1) };
    let i = match info(data_dir, &id) {
        Ok(i) => i,
        Err(_) => {
            let known = list(data_dir).iter().map(|d| d.id.clone()).collect::<Vec<_>>().join(", ");
            return Err(vec![Diagnostic::error("B178", file, line, msg!("diag-basic-b178-dialect-unknown", id = id.as_str(), known = known.as_str()))]);
        }
    };
    let mut warnings = vec![];
    if let Some(s) = system { if !i.systems.is_empty() && !i.systems.iter().any(|x| x == s) {
        warnings.push(Diagnostic::warning("B170", file, line, msg!("diag-basic-b170-dialect-system", id = id.as_str(), system = s)));
    } }
    Ok(Resolved { id, from_file, warnings })
}
