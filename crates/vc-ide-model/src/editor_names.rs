// SPDX-License-Identifier: GPL-3.0-or-later
//! Namens-Dialekt, Chipregister, Zahlenerkennung und Include-Zeilen des Editors (ohne GUI).

/// BASIC-Dialekt, nach dem der Hover lange Namen auf Kurznamen abbildet
#[derive(Debug)]
pub enum NameDialect { Cbm(vc_basic::Dialect), Atari(vc_basic::atari::AtariDialect) }
static NAME_DIALECT: std::sync::Mutex<Option<(String, std::sync::Arc<NameDialect>)>> = std::sync::Mutex::new(None);

/// Dialekt (`cbm_v2`, `atari_basic` …) für den Hover über langen Namen wählen; lädt nur bei Wechsel neu
pub fn select_name_dialect(data: &std::path::Path, id: &str) {
    let Ok(mut g) = NAME_DIALECT.lock() else { return };
    if g.as_ref().map_or(false, |(i, _)| i == id) { return; }
    let d = match vc_basic::dialect_family(data, id).as_deref() {
        Ok("atari") => vc_basic::atari::AtariDialect::load(data, id).ok().map(NameDialect::Atari),
        _ => vc_basic::Dialect::load(data, id).ok().map(NameDialect::Cbm),
    };
    *g = d.map(|d| (id.to_string(), std::sync::Arc::new(d)));
}

/// Chipregister des Zielsystems der aktiven BASIC-Datei (für Hover, Bit-Editor, Eingabehilfe)
static CHIPS: std::sync::Mutex<Option<(String, std::sync::Arc<vc_ide::chipmap::ChipMap>)>> = std::sync::Mutex::new(None);

/// Zielsystem (`c64`, `atari8` …) wählen; lädt die Chipregister nur bei Wechsel neu
pub fn select_chips(data: &std::path::Path, target: &str) {
    let Ok(mut g) = CHIPS.lock() else { return };
    if g.as_ref().map_or(false, |(t, _)| t == target) { return; }
    *g = Some((target.to_string(), std::sync::Arc::new(vc_ide::chipmap::ChipMap::load(data, target))));
}
pub fn chips() -> Option<std::sync::Arc<vc_ide::chipmap::ChipMap>> { CHIPS.lock().ok()?.as_ref().map(|c| c.1.clone()) }

/// Aktueller Dialekt für Zahlenerkennung und Hover (`None`, solange keiner geladen ist)
pub fn name_dialect() -> Option<std::sync::Arc<NameDialect>> { NAME_DIALECT.lock().ok()?.as_ref().map(|x| x.1.clone()) }

/// Zahlenpass-Einstellungen des Editors (Zahlenerkennung, Hover); die Seite ›BASIC · Zahlen‹ setzt sie
pub fn number_options() -> vc_basic::numbers::NumberOptions { vc_basic::numbers::defaults() }

/// Zahlen einer Zeile (die Erkennung des Übersetzers) mit dem Dialekt `d`; `long_names`: Quelltext mit `#longnames`
pub fn scan_basic(d: &NameDialect, line: &str, long_names: bool) -> vc_basic::numbers::Scan {
    use vc_basic::numbers::{scan_line, Lexicon};
    let o = vc_basic::numbers::NumberOptions { whole_words: long_names, ..number_options() };
    match d { NameDialect::Cbm(d) => scan_line(line, &Lexicon::Cbm(d), &o), NameDialect::Atari(d) => scan_line(line, &Lexicon::Atari(d), &o) }
}

/// Hover über einem langen Namen (`#longnames`): der Kurzname im gebauten Programm
pub fn long_name_hover(text: &str, word: &str) -> Option<String> {
    if word.starts_with('@') || !vc_basic::longnames::has_directive(text) { return None; }
    let d = NAME_DIALECT.lock().ok()?.as_ref()?.1.clone();
    let entries = match &*d { NameDialect::Cbm(d) => vc_basic::longnames::lookup(text, vc_basic::longnames::Target::Cbm(d), word), NameDialect::Atari(d) => vc_basic::longnames::lookup(text, vc_basic::longnames::Target::Atari(d), word) };
    if entries.is_empty() { return None; }
    let mut md = format!("**{}** – {}\n\n", word.trim_end_matches(['$', '%']), vc_i18n::t!("editor-names-langer-name-beim-bauen-umgesetzt"));
    for e in &entries { md.push_str(&format!("- `{}` → `{}`  ({}{})\n", e.long, e.short, e.kind, if e.array { if e.scalar { vc_i18n::t!("editor-names-auch-als-feld") } else { vc_i18n::t!("editor-names-feld") } } else { String::new() })); }
    Some(md)
}

/// Pfad aus einer `.include "…"`-, `#include "…"`-, `.incbin "…"`- bzw. BASIC-`include "…"`-Zeile samt Zeichenbereich (Zeichenindizes, ohne Anführungszeichen).
/// Systembibliotheken (`#include <…>`) liefern nichts.
pub fn include_span(line: &str) -> Option<(String, usize, usize)> {
    let t = line.trim_start(); let l = t.to_ascii_lowercase();
    let kw = [".include", "#include", "include", ".incbin", "incbin"].into_iter().find(|k| l.starts_with(k))?;
    if !t[kw.len()..].starts_with(|c: char| c.is_whitespace() || c == '"') { return None; }
    let a = t.find('"')?; let b = t[a + 1..].find('"')?;
    let lead = line.chars().count() - t.chars().count();
    let ca = lead + t[..a].chars().count() + 1;
    let path = t[a + 1..a + 1 + b].to_string();
    let n = path.chars().count();
    Some((path, ca, ca + n))
}

/// Fundstellen (Bytebereiche) aller Zahlen mit dem Wert `v` in einer Zeile, gleich in welcher Schreibweise
pub fn find_value_bytes(d: &NameDialect, line: &str, v: u32) -> Vec<(usize, usize)> {
    use vc_basic::numbers::Lexicon;
    let o = number_options();
    let hits = match d { NameDialect::Cbm(d) => vc_basic::numtools::find_value(line, v, &Lexicon::Cbm(d), &o), NameDialect::Atari(d) => vc_basic::numtools::find_value(line, v, &Lexicon::Atari(d), &o) };
    hits.iter().map(|n| (char_to_byte(line, n.start), crate::editor_names::char_to_byte(line, n.end))).collect()
}


/// Natives BASIC (`.bas`, `.lst`, `.tur`): hier gelten `$hex` und `%binär`
pub fn is_native_basic(rel: &str) -> bool { let r = rel.to_ascii_lowercase(); r.ends_with(".bas") || r.ends_with(".lst") || r.ends_with(".tur") }

pub fn char_to_byte(s: &str, idx: usize) -> usize { s.char_indices().nth(idx).map_or(s.len(), |x| x.0) }
