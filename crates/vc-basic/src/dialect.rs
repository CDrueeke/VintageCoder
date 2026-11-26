// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC-Dialekt aus JSON: Tokentabelle, Steuercodes, Grenzen.
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use vc_target::parse_num;

#[derive(Debug, Clone)]
pub struct Dialect {
    pub id: String,
    pub load_addr: u16,
    pub max_line_len: usize,
    /// BASIC 65: längster Treffer gewinnt (FOREGROUND vor FOR); sonst der erste in Tabellenreihenfolge (C64-ROM)
    pub match_longest: bool,
    pub reserved_vars: Vec<String>,
    /// Zahl der Zeichen, die das BASIC an einem Variablennamen unterscheidet (`varSigChars`; CBM: 2)
    pub var_sig_chars: usize,
    /// `RESTORE zeile` kennt der Dialekt (nicht das BASIC V2 des C64)
    pub restore_line: bool,
    /// Keywords in Token-Reihenfolge (die ROM-Suche nimmt den ersten Treffer).
    /// Ein Token besteht aus einem Byte oder (BASIC 65: `$CE xx`, `$FE xx`) aus Präfix und Folgebyte.
    pub tokens: Vec<(String, Vec<u8>)>,
    pub control: Vec<(String, u8)>,
    /// `$hex`/`%binär` als Literal im Interpreter (`numberLiterals`); fehlt das Feld: keine
    pub number_literals: crate::numbers::NumberRules,
    /// Wörter, hinter denen Zeilennummern als Sprungziele stehen (`jumps` der Dialektdatei)
    pub jumps: Vec<String>,
    /// Wörter, die zur Laufzeit die Zeilennummer liefern (`runtimeLine`)
    pub runtime_line: Vec<String>,
    /// `'` leitet einen Kommentar ein (`quoteComment`; Commodore: nein)
    pub quote_comment: bool,
    /// Argumentzahlen der Anweisungen und Funktionen (`syntax`) für die Prüfung
    pub syntax: Syntax,
    /// Schlüsselwörter anderer Commodore-Dialekte, die dieser nicht kennt (für `B157`)
    pub foreign: std::collections::HashSet<String>,
    by_name: HashMap<String, Vec<u8>>,
    by_token: HashMap<Vec<u8>, String>,
}

/// Zulässige Argumentzahl `(kleinste, größte)` je Schlüsselwort
#[derive(Debug, Clone, Default)]
pub struct Syntax { pub statements: HashMap<String, (usize, usize)>, pub functions: HashMap<String, (usize, usize)> }

fn arity(v: &Value) -> HashMap<String, (usize, usize)> {
    v.as_object().map(|o| o.iter().filter_map(|(k, x)| {
        let s = x.as_str()?;
        let (a, b) = s.split_once('-').unwrap_or((s, s));
        Some((k.to_ascii_uppercase(), (a.trim().parse().ok()?, b.trim().parse().ok()?)))
    }).collect()).unwrap_or_default()
}

impl Dialect {
    pub fn load(data_dir: &Path, id: &str) -> Result<Dialect, String> {
        let path = data_dir.join("basic").join(format!("{id}.dialect.json"));
        let v: Value = crate::dialects::load_json(data_dir, id)?;
        let rows = |key: &str| -> Result<Vec<(String, Vec<u8>)>, String> {
            v[key].as_array().ok_or_else(|| vc_i18n::t!("basic-dialect-key-missing", key = key))?.iter().map(|r| {
                let n = r[0].as_str().ok_or_else(|| vc_i18n::t!("basic-dialect-name-missing"))?.to_string();
                let c = r[1].as_str().and_then(parse_num).ok_or_else(|| vc_i18n::t!("basic-dialect-code-invalid"))?;
                Ok((n, if c > 0xFF { vec![(c >> 8) as u8, c as u8] } else { vec![c as u8] }))
            }).collect()
        };
        let tokens = rows("tokens")?;
        let control: Vec<(String, u8)> = rows("control")?.into_iter().map(|(n, c)| (n, c[0])).collect();
        let d = Dialect {
            id: v["id"].as_str().unwrap_or(id).to_string(),
            load_addr: v["loadAddr"].as_str().and_then(parse_num).unwrap_or(0x0801) as u16,
            max_line_len: v["maxLineLen"].as_u64().unwrap_or(80) as usize,
            match_longest: v["matchLongest"].as_bool().unwrap_or(false),
            restore_line: v["capabilities"]["restoreLine"].as_bool().unwrap_or(true),
            var_sig_chars: v["varSigChars"].as_u64().unwrap_or(2).max(1) as usize,
            reserved_vars: v["reservedVars"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            by_name: tokens.iter().map(|(n, c)| (n.clone(), c.clone())).collect(),
            by_token: tokens.iter().map(|(n, c)| (c.clone(), n.clone())).collect(),
            number_literals: crate::numbers::NumberRules::from_json(&v),
            jumps: v["jumps"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| ["GOTO", "GOSUB", "THEN", "RUN"].iter().map(|s| s.to_string()).collect()),
            runtime_line: v["runtimeLine"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            quote_comment: v["quoteComment"].as_bool().unwrap_or(false),
            syntax: Syntax { statements: arity(&v["syntax"]["statements"]), functions: arity(&v["syntax"]["functions"]) },
            foreign: Default::default(),
            tokens, control,
        };
        let mut d = d;
        d.foreign = foreign_words(data_dir, &d);
        for need in ["REM", "DATA", "GOTO", "GOSUB", "THEN", "IF", "NOT", "PRINT", "PRINT#", "INPUT", "INPUT#", "TAB(", "SPC(", "ON", "RESTORE", "RUN"] {
            if !d.by_name.contains_key(need) { return Err(vc_i18n::t!("basic-dialect-keyword-required", path = path.display().to_string(), need = need)); }
        }
        Ok(d)
    }
    /// Kopie mit zusätzlichen Schlüsselwörtern (Codes ab `first`); sie stehen vorn in der Tabelle, damit `POSITION` vor `POS` und `COLOR` vor `OR` gefunden wird
    pub fn with_extra(&self, words: &[&str], first: u8) -> Dialect {
        let mut d = self.clone();
        let mut extra = vec![];
        for (i, w) in words.iter().enumerate() {
            let code = vec![first + i as u8];
            extra.push((w.to_string(), code.clone()));
            d.by_name.insert(w.to_string(), code.clone());
            d.by_token.insert(code, w.to_string());
        }
        d.tokens.splice(0..0, extra);
        d
    }
    /// Token als Byte (nur Einzelbyte-Tokens)
    pub fn token(&self, name: &str) -> Option<u8> { self.by_name.get(name).filter(|b| b.len() == 1).map(|b| b[0]) }
    /// Alle Bytes des Tokens (auch Präfix-Tokens)
    pub fn token_bytes(&self, name: &str) -> Option<Vec<u8>> { self.by_name.get(name).cloned() }
    pub fn tok(&self, name: &str) -> u8 { self.by_name[name][0] }
    pub fn keyword(&self, code: u8) -> Option<&str> { self.by_token.get(&vec![code]).map(String::as_str) }
    /// Schlüsselwort an Position `i` im Zeilenbyte-Strom (mit Präfix-Tokens) und seine Länge in Bytes
    pub fn keyword_at(&self, bytes: &[u8], i: usize) -> Option<(&str, usize)> {
        if let Some(b2) = bytes.get(i + 1) { if let Some(k) = self.by_token.get(&vec![bytes[i], *b2]) { return Some((k.as_str(), 2)); } }
        self.by_token.get(&vec![bytes[i]]).map(|k| (k.as_str(), 1))
    }
    pub fn control_code(&self, name: &str) -> Option<u8> { self.control.iter().find(|(n, _)| n == name).map(|(_, c)| *c) }
    pub fn control_name(&self, code: u8) -> Option<&str> { self.control.iter().find(|(_, c)| *c == code).map(|(n, _)| n.as_str()) }
    pub fn is_reserved_var(&self, name: &str) -> bool { self.reserved_vars.iter().any(|r| r.eq_ignore_ascii_case(name)) }
}

/// Wörter anderer Commodore-Dialekte (Dateien in `data/basic`), die `d` nicht kennt; Fehler beim Lesen einzelner Dateien werden übergangen
fn foreign_words(data_dir: &Path, d: &Dialect) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for other in crate::dialects::list(data_dir) {
        if other.family != "cbm" || other.id == d.id { continue; }
        let Ok(v) = crate::dialects::load_json(data_dir, &other.id) else { continue };
        for r in v["tokens"].as_array().into_iter().flatten() {
            let Some(w) = r[0].as_str() else { continue };
            let w = w.trim_end_matches(|c| matches!(c, '(' | '#' | '$')).to_ascii_uppercase();
            if w.len() >= 3 && w.chars().all(|c| c.is_ascii_alphabetic()) && !d.by_name.keys().any(|k| k.trim_end_matches(|c| matches!(c, '(' | '#' | '$')).eq_ignore_ascii_case(&w)) { out.insert(w); }
        }
    }
    out
}
