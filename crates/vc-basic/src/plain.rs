// SPDX-License-Identifier: GPL-3.0-or-later
//! Natives BASIC mit optionalen Labels: Quelltext im Dialekt des Systems (CBM BASIC V2/3.5/7/65, Atari BASIC, Turbo BASIC XL).
//!
//! Zeilen dürfen eine Nummer haben (`10 PRINT "HI"`) oder keine; Zeilen ohne Nummer werden fortlaufend nummeriert.
//! `@name:` am Zeilenanfang definiert ein Label (die Zeile kann leer sein, das Label gilt dann für die nächste Anweisung),
//! `@name` im Ausdruck (`GOTO @name`, `THEN @name`, `ON X GOSUB @a,@b`, `RESTORE @daten`) wird durch die Zeilennummer ersetzt.
//! In Zeichenketten sowie nach `REM` und in `DATA` bleibt der Text unverändert. Der Rest der Zeile ist unverändertes BASIC.
use std::collections::HashMap;
use vc_core::{msg, Diagnostic};

#[derive(Debug, Clone)]
pub struct PlainOptions {
    pub file: String,
    pub start: u32,
    pub step: u32,
    /// größte erlaubte Zeilennummer (CBM 63999, Atari 32767)
    pub max_line: u32,
    /// Zahlenpass (`$hex`, `%binär`), siehe `numbers`
    pub numbers: crate::numbers::NumberOptions,
    /// Syntaxprüfung (Warnungen `B150`–`B159`, `B167`; Commodore-Dialekte): je Code abschaltbar, siehe `check`
    pub check: crate::check::CheckOptions,
    /// Bauoption „kompakt“: Leerzeichen innerhalb der Anweisungen entfernen, wo es nichts ändert (nur Commodore; Standard aus), siehe `compact`
    pub compact: bool,
    /// Zielsystem (`c64`, `atari8` …) für `#if SYSTEM_…` (`preprocess`)
    pub system: Option<String>,
    /// vordefinierte Namen für `#define`-Ersetzung (Kommandozeile `-D NAME=wert`)
    pub defines: Vec<(String, String)>,
    /// Zahl der per `#include` eingebundenen Medien (setzt der Bau nach `expand_includes_with`; für die Lesereihenfolge-Warnung `B196`)
    pub media: usize,
}

impl PlainOptions {
    /// Kopie mit Start und Schrittweite dieser Quelle (`#numbering` der Datei vor den Werten hier), Wertebereich geprüft (`B190`–`B192`)
    pub fn with_numbering(&self, src: &str) -> Result<PlainOptions, Vec<Diagnostic>> {
        let n = crate::numbering::resolve(src, &self.file, crate::numbering::Numbering::new(self.start, self.step), self.max_line)?;
        let mut o = self.clone();
        (o.start, o.step) = (n.start, n.step);
        // Reichen die Nummern für alle Zeilen ohne eigene Nummer?
        let free = src.lines().filter(|l| { let t = l.trim(); !(t.is_empty() || t.starts_with(';') || t.starts_with('#') || t.chars().next().map_or(false, |c| c.is_ascii_digit())) }).count();
        n.check_fit(free, self.max_line, &self.file, 1).map_err(|d| vec![d])?;
        Ok(o)
    }
    /// Einstellungen der Vorverarbeitung (`#define`, `#if`) mit den Schlüsselwörtern des Dialekts
    pub fn preprocess(&self, keywords: std::collections::HashSet<String>) -> crate::preprocess::PreOptions {
        crate::preprocess::PreOptions { file: self.file.clone(), system: self.system.clone(), defines: self.defines.clone(), keywords }
    }
    pub fn new(file: impl Into<String>, max_line: u32) -> Self { PlainOptions { file: file.into(), start: 10, step: 10, max_line, numbers: crate::numbers::defaults(), check: Default::default(), compact: false, system: None, defines: vec![], media: 0 } }
}

#[derive(Debug, Clone)]
pub struct Numbered {
    /// Nummerierter Text; je Quellzeile genau eine Zeile (leer für Kommentar-, Leer- und Labelzeilen), damit Zeilenangaben stimmen
    pub text: String,
    pub labels: Vec<(String, u32)>,
    /// Die Quelle enthält `#longnames`: lange Variablennamen sind eingeschaltet (siehe `longnames`)
    pub long_names: bool,
}

pub(crate) fn is_ident_start(c: char) -> bool { c.is_ascii_alphabetic() || c == '_' }
pub(crate) fn is_ident(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }

/// Label am Zeilenanfang (`@name:`): Name und Rest der Zeile
pub(crate) fn label_def(t: &str) -> Option<(String, &str)> {
    let r = t.strip_prefix('@')?;
    let end = r.find(|c: char| !is_ident(c)).unwrap_or(r.len());
    let name = &r[..end];
    if name.is_empty() || !is_ident_start(name.chars().next()?) { return None; }
    let rest = r[end..].trim_start().strip_prefix(':')?;
    Some((name.to_ascii_lowercase(), rest.trim_start()))
}

/// Ersetzt `@name` durch die Zeilennummer (außer in Zeichenketten, nach REM und in DATA).
fn substitute(rest: &str, labels: &HashMap<String, u32>, line_consts: bool, file: &str, line: usize, errs: &mut Vec<Diagnostic>) -> String {
    let ch: Vec<char> = rest.chars().collect();
    let (mut out, mut i, mut quote, mut data) = (String::new(), 0, false, false);
    while i < ch.len() {
        let c = ch[i];
        if c == '"' { quote = !quote; out.push(c); i += 1; continue; }
        if quote { out.push(c); i += 1; continue; }
        if data { if c == ':' { data = false; } out.push(c); i += 1; continue; }
        // Schlüsselwörter REM / DATA an Wortgrenze
        if i == 0 || !is_ident(ch[i - 1]) {
            let word: String = ch[i..].iter().take_while(|c| c.is_ascii_alphabetic()).collect::<String>().to_ascii_uppercase();
            if word == "REM" { out.extend(&ch[i..]); return out; }
            if word == "DATA" { data = true; out.extend(&ch[i..i + 4]); i += 4; continue; }
        }
        if c == '@' && ch.get(i + 1).map_or(false, |c| is_ident_start(*c)) {
            let mut j = i + 1;
            while j < ch.len() && is_ident(ch[j]) { j += 1; }
            let name: String = ch[i + 1..j].iter().collect::<String>().to_ascii_lowercase();
            match labels.get(&name) {
                Some(n) => out.push_str(&n.to_string()),
                None => { errs.push(Diagnostic::error("B110", file, line, msg!("diag-basic-b110-label-undefined", name = name))); out.push_str("0"); }
            }
            i = j;
            continue;
        }
        // `<label>_line`: Zeilennummer der Marke als Konstante (Medien, siehe `include`)
        if line_consts && is_ident_start(c) && (i == 0 || !(is_ident(ch[i - 1]) || matches!(ch[i - 1], '@' | '$' | '%'))) {
            let mut j = i;
            while j < ch.len() && is_ident(ch[j]) { j += 1; }
            let w: String = ch[i..j].iter().collect::<String>().to_ascii_lowercase();
            if let Some(n) = w.strip_suffix("_line").and_then(|stem| labels.get(stem)).filter(|_| !matches!(ch.get(j), Some('$' | '%'))) { out.push_str(&n.to_string()); i = j; continue; }
            out.extend(&ch[i..j]); i = j; continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Pfad aus einer Zeile `#include "datei"` (Groß-/Kleinschreibung egal)
pub fn include_directive(line: &str) -> Option<&str> {
    let t = line.trim();
    if !t.get(..8)?.eq_ignore_ascii_case("#include") { return None; }
    let r = t[8..].trim();
    let r = r.strip_prefix('"')?;
    r.find('"').map(|e| &r[..e])
}

/// Zeile im Gesamttext → (Datei, Zeile) in der Quelle, aus der sie stammt
pub type LineMap = Vec<(String, usize)>;

/// Ersetzt `#include "datei"`-Zeilen durch den Inhalt der Datei (Pfad relativ zur einbindenden Datei, auch verschachtelt).
/// Liefert den Gesamttext und je Zeile die Herkunft, damit Fehlermeldungen auf die richtige Datei und Zeile zeigen.
pub fn expand_includes(src: &str, file: &str) -> Result<(String, LineMap), Vec<Diagnostic>> {
    expand_includes_with(src, file, &crate::include::IncludeHooks::default()).map(|e| (e.text, e.map))
}

/// Wie [`expand_includes`], aber mit Haken für Medien (Provider) und Altpfad-Hinweise; liefert zusätzlich Warnungen und die eingebundenen Medien
pub fn expand_includes_with(src: &str, file: &str, hooks: &crate::include::IncludeHooks) -> Result<crate::include::Expanded, Vec<Diagnostic>> {
    crate::include::expand(src, file, hooks)
}

/// Diagnosen des Gesamttextes auf Datei und Zeile der Quelle zurückführen
pub fn remap_diagnostics(ds: &[Diagnostic], file: &str, map: &LineMap) -> Vec<Diagnostic> {
    ds.iter().map(|d| {
        let mut d = d.clone();
        if d.file == file { if let Some((f, l)) = d.line.checked_sub(1).and_then(|i| map.get(i)) { d.file = f.clone(); d.line = *l; } }
        d
    }).collect()
}

/// Nummeriert Zeilen ohne Nummer, sammelt Labels und löst `@name` auf.
pub fn number_lines(src: &str, o: &PlainOptions) -> Result<Numbered, Vec<Diagnostic>> {
    struct L { rest: String, num: u32, file_line: usize }
    let (mut errs, mut labels, mut order): (Vec<Diagnostic>, HashMap<String, u32>, Vec<(String, u32)>) = (vec![], HashMap::new(), vec![]);
    let (mut next, mut last, mut pending): (u32, Option<u32>, Vec<(String, usize)>) = (o.start, None, vec![]);
    let (mut lines, n_src): (Vec<L>, usize) = (vec![], src.lines().count());
    for (n, raw) in src.lines().enumerate() {
        let t = raw.trim();
        if t.is_empty() || t.starts_with(';') || include_directive(t).is_some() || crate::longnames::is_directive(t) || crate::dialects::is_directive(t) || crate::numbering::is_directive(t) { continue; }
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        let explicit: Option<u32> = if digits.is_empty() { None } else { digits.parse().ok().or(Some(u32::MAX)) };
        let mut body = t[digits.len()..].trim_start();
        let mut mine: Vec<String> = vec![];
        while let Some((name, rest)) = label_def(body) { mine.push(name); body = rest; }
        if let Some(e) = explicit {
            if e > o.max_line { errs.push(Diagnostic::error("B100", &o.file, n + 1, msg!("diag-basic-b100-line-number-max", digits = digits.as_str(), max = o.max_line))); continue; }
        }
        if body.is_empty() && !mine.is_empty() {
            // nur Label(s): gelten für die nächste Anweisung
            if let Some(e) = explicit { next = e; }
            pending.extend(mine.into_iter().map(|m| (m, n + 1)));
            continue;
        }
        let num = explicit.unwrap_or(next);
        if num > o.max_line { errs.push(Diagnostic::error("B100", &o.file, n + 1, msg!("diag-basic-b100-line-number-auto-max", num = num, max = o.max_line))); continue; }
        if explicit.is_some() { if let Some(l) = last { if num <= l { errs.push(Diagnostic::error("B100", &o.file, n + 1, msg!("diag-basic-b100-line-number-order", num = num, last = l))); } } }
        for (m, at) in pending.drain(..).map(|(m, at)| (m, at)).chain(mine.into_iter().map(|m| (m, n + 1))) {
            if labels.insert(m.clone(), num).is_some() { errs.push(Diagnostic::error("B111", &o.file, at, msg!("diag-basic-b111-label-duplicate", name = m.as_str()))); }
            order.push((m, num));
        }
        last = Some(num); next = num + o.step;
        lines.push(L { rest: body.to_string(), num, file_line: n + 1 });
    }
    for (m, at) in &pending { errs.push(Diagnostic::error("B112", &o.file, *at, msg!("diag-basic-b112-label-at-end", name = m.as_str()))); }
    let mut out = vec![String::new(); n_src];
    for l in &lines {
        let r = substitute(&l.rest, &labels, o.media > 0, &o.file, l.file_line, &mut errs);
        out[l.file_line - 1] = format!("{} {}", l.num, r);
    }
    if !errs.is_empty() { return Err(errs); }
    let mut text = out.join("\n"); text.push('\n');
    Ok(Numbered { text, labels: order, long_names: crate::longnames::has_directive(src) })
}

/// Vorschau für den Editor: die Zeilennummer, die jede Quellzeile beim Bauen automatisch bekommt.
/// `None` für Zeilen mit eigener Nummer, Kommentar-, Leer- und Labelzeilen. Fehler (z. B. unbekannte Labels) spielen hier keine Rolle.
pub fn auto_line_numbers(src: &str, start: u32, step: u32) -> Vec<Option<u32>> {
    let mut next = start;
    src.lines().map(|raw| {
        let t = raw.trim();
        if t.is_empty() || t.starts_with(';') || crate::longnames::is_directive(t) || crate::dialects::is_directive(t) || crate::numbering::is_directive(t) { return None; }
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        let explicit: Option<u32> = if digits.is_empty() { None } else { digits.parse().ok() };
        let mut body = t[digits.len()..].trim_start();
        while let Some((_, rest)) = label_def(body) { body = rest; }
        if body.is_empty() { if let Some(e) = explicit { next = e; } return None; }   // nur Label(s): die Nummer gilt für die nächste Anweisung
        match explicit {
            Some(e) => { next = e.saturating_add(step); None }
            None => { let n = next; next = next.saturating_add(step); Some(n) }
        }
    }).collect()
}

#[cfg(test)]
mod include_tests {
    use super::*;
    #[test]
    fn directive_and_expansion() {
        assert_eq!(include_directive("  #INCLUDE \"a/b.bas\" "), Some("a/b.bas"));
        assert_eq!(include_directive("10 PRINT"), None);
        let dir = std::env::temp_dir().join(format!("vc_inc_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::write(dir.join("lib/d.bas"), "@d:\nDATA 1,2\n").unwrap();
        let main = dir.join("m.bas").to_string_lossy().into_owned();
        let (t, map) = expand_includes("PRINT 1\n#include \"lib/d.bas\"\nREAD A\n", &main).unwrap();
        assert_eq!(t, "PRINT 1\n@d:\nDATA 1,2\nREAD A\n");
        assert_eq!(map[2].1, 2); assert!(map[1].0.ends_with("d.bas")); assert_eq!(map[3], (main.clone(), 3));
        let e = expand_includes("#include \"fehlt.bas\"\n", &main).unwrap_err();
        assert_eq!(e[0].code, "B120");
        let n = number_lines("PRINT 1\n#include \"x\"\nPRINT 2\n", &PlainOptions::new("m.bas", 63999)).unwrap();
        assert!(n.text.contains("10 PRINT 1") && n.text.contains("20 PRINT 2"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
