// SPDX-License-Identifier: GPL-3.0-or-later
//! Syntaxfarben für Assembler und strukturiertes BASIC als egui-`LayoutJob`.
use eframe::egui::{text::{LayoutJob, TextFormat}, Color32, FontId, Stroke};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang { Asm, Basic, Plain }

impl Lang {
    pub fn of(path: &str) -> Lang {
        let p = path.to_ascii_lowercase();
        if p.ends_with(".asm") || p.ends_with(".inc") || p.ends_with(".s") { Lang::Asm } else if p.ends_with(".sbas") || p.ends_with(".bas") || p.ends_with(".lst") || p.ends_with(".tur") { Lang::Basic } else { Lang::Plain }
    }
    pub fn name(self) -> &'static str { match self { Lang::Asm => "Assembler", Lang::Basic => "BASIC", Lang::Plain => "Text" } }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Plain, Comment, Str, Num, Directive, Mnemonic, Label, Symbol, Keyword, Register }

/// Fehlerrot für ältere Aufrufer außerhalb des Editors (feste Farbe, auf beiden Gründen lesbar)
pub const PAL_ERR: Color32 = Color32::from_rgb(0xdc, 0x4e, 0x44);
/// Fehlerrot des Editors (Token `rd` des aktiven Schemas)
pub fn pal_err() -> Color32 { crate::ui::tokens::syntax().error }

fn colour(k: Kind) -> Color32 {
    let s = crate::ui::tokens::syntax();
    match k { Kind::Plain => s.text, Kind::Comment => s.comment, Kind::Str => s.string, Kind::Num => s.number, Kind::Directive | Kind::Keyword => s.directive, Kind::Mnemonic => s.opcode, Kind::Label | Kind::Symbol | Kind::Register => s.label }
}

const BASIC_KEYWORDS: &[&str] = &["if", "then", "else", "elseif", "endif", "for", "to", "step", "next", "while", "wend", "do", "loop", "until", "repeat", "const", "include", "gosub", "goto", "return", "print", "poke", "peek", "dim", "data", "read", "restore", "input", "end", "rem", "let", "and", "or", "not", "sub", "endsub", "proc", "endproc", "call", "blob"];

/// Eingebaute Funktionen des Assemblers (nur mit folgender Klammer)
const ASM_FUNCTIONS: &[&str] = &["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sqrt", "pow", "exp", "log", "log2", "log10", "int", "floor", "ceil", "round", "abs", "sign", "min", "max", "len", "range", "next", "align"];

/// Zerlegt eine Zeile in (Bereich, Art)
fn tokens(line: &str, lang: Lang, mn: &HashSet<String>) -> Vec<(usize, usize, Kind)> {
    if lang == Lang::Basic && vc_basic::longnames::is_directive(line) { let a = line.len() - line.trim_start().len(); return vec![(a, line.trim_end().len(), Kind::Directive)]; }   // `#longnames`
    let b: Vec<(usize, char)> = line.char_indices().collect();
    let mut out = vec![]; let mut i = 0; let mut stmt_seen = false;
    let at = |k: usize| b.get(k).map(|x| x.1);
    let pos = |k: usize| b.get(k).map(|x| x.0).unwrap_or(line.len());
    while i < b.len() {
        let c = b[i].1; let start = i;
        if c.is_whitespace() { i += 1; continue; }
        if lang == Lang::Asm && (c == ';' || (c == '/' && at(i + 1) == Some('/'))) || lang == Lang::Basic && c == '\'' { out.push((pos(i), line.len(), Kind::Comment)); break; }
        // anonyme Labels: `-`/`+` (auch mehrfach) allein am Zeilenanfang oder als gesamter Operand
        if lang == Lang::Asm && (c == '+' || c == '-') {
            let mut j = i; while at(j) == Some(c) { j += 1; }
            let rest_empty = b[j..].iter().all(|x| x.1.is_whitespace()) || { let r = line[pos(j)..].trim_start(); r.starts_with(';') || r.starts_with("//") };
            let alone = at(j).map_or(true, |d| d.is_whitespace() || d == ';');
            if alone && ((!stmt_seen && out.is_empty()) || (stmt_seen && rest_empty)) {
                out.push((pos(i), pos(j), Kind::Label)); i = j; stmt_seen = true; continue;
            }
        }
        if c == '"' { i += 1; while i < b.len() && b[i].1 != '"' { i += 1; } i = (i + 1).min(b.len()); out.push((pos(start), pos(i), Kind::Str)); continue; }
        if c == '$' || c == '%' && at(i + 1).map_or(false, |d| d == '0' || d == '1') || c.is_ascii_digit() {
            i += 1; while i < b.len() && (b[i].1.is_ascii_alphanumeric() || b[i].1 == '_') { i += 1; }
            // Kommazahl: 2.5, 1e3, 1.5e-2
            if lang == Lang::Asm && c.is_ascii_digit() {
                if at(i) == Some('.') && at(i + 1).map_or(false, |d| d.is_ascii_digit()) { i += 1; while i < b.len() && (b[i].1.is_ascii_alphanumeric() || b[i].1 == '_') { i += 1; } }
                if matches!(at(i.wrapping_sub(1)), Some('e' | 'E')) && matches!(at(i), Some('+' | '-')) && at(i + 1).map_or(false, |d| d.is_ascii_digit()) { i += 1; while i < b.len() && b[i].1.is_ascii_digit() { i += 1; } }
            }
            out.push((pos(start), pos(i), Kind::Num)); stmt_seen = true; continue;
        }
        if c.is_alphabetic() || c == '_' || c == '.' || c == '@' {
            i += 1; while i < b.len() && (b[i].1.is_alphanumeric() || b[i].1 == '_' || b[i].1 == '.' ) { i += 1; }
            let w = &line[pos(start)..pos(i)]; let lw = w.to_ascii_lowercase();
            let colon = at(i) == Some(':');
            let kind = match lang {
                Lang::Asm => {
                    if colon && !stmt_seen { i += 1; Kind::Label }
                    else if !stmt_seen && w.starts_with('.') { stmt_seen = true; Kind::Directive }
                    else if !stmt_seen && mn.contains(&lw) { stmt_seen = true; Kind::Mnemonic }
                    else if matches!(lw.as_str(), "a" | "x" | "y" | "z" | "s" | "sp") { Kind::Register }
                    else if ASM_FUNCTIONS.contains(&lw.as_str()) && at(i) == Some('(') { stmt_seen = true; Kind::Keyword }
                    else if w.contains('.') || (stmt_seen && !w.is_empty() && !w.starts_with('.') && false) { Kind::Symbol }
                    else { stmt_seen = true; Kind::Plain }
                }
                Lang::Basic => {
                    if w.starts_with('@') { Kind::Label } else if BASIC_KEYWORDS.contains(&lw.as_str()) { Kind::Keyword } else { Kind::Plain }
                }
                Lang::Plain => Kind::Plain,
            };
            out.push((pos(start), pos(i), kind)); continue;
        }
        i += 1; out.push((pos(start), pos(i), Kind::Plain));
    }
    out
}

/// Farbiger Text; `errors`: Zeilen (ab 0) mit Fehlern werden rot unterstrichen
pub fn layout(text: &str, lang: Lang, mn: &HashSet<String>, errors: &HashMap<usize, String>, font: FontId, hidden: &HashSet<usize>, marks: &HashMap<usize, usize>, pc_line: Option<usize>) -> LayoutJob {
    let mut job = LayoutJob::default(); job.wrap.max_width = f32::INFINITY;
    let mut first = true;
    for (n, line) in text.split('\n').enumerate() {
        if hidden.contains(&n) {   // gefaltete Zeile: winzig und durchsichtig, bleibt aber im Text (Bearbeiten und Positionen bleiben 1:1)
            let f = TextFormat::simple(FontId::monospace(0.5), Color32::TRANSPARENT);
            if !first { job.append("\n", 0.0, f.clone()); } first = false;
            if !line.is_empty() { job.append(line, 0.0, f); }
            continue;
        }
        if !first { job.append("\n", 0.0, TextFormat::simple(font.clone(), crate::ui::tokens::syntax().text)); }
        first = false;
        let under = if errors.contains_key(&n) { Stroke::new(1.5, pal_err()) } else { Stroke::NONE };
        let mut at = 0;
        let bg = if pc_line == Some(n) { { let w = crate::theme::accent(); Color32::from_rgba_unmultiplied(w.r(), w.g(), w.b(), 28) } } else { Color32::TRANSPARENT };
        let put = |job: &mut LayoutJob, s: &str, k: Kind| { if s.is_empty() { return; } let mut f = TextFormat::simple(font.clone(), colour(k)); f.underline = under; f.background = bg; job.append(s, 0.0, f); };
        for (a, z, k) in tokens(line, lang, mn) { if a > at { put(&mut job, &line[at..a], Kind::Plain); } put(&mut job, &line[a..z], k); at = z; }
        if at < line.len() { put(&mut job, &line[at..], Kind::Plain); }
        if let Some(k) = marks.get(&n) { let mut f = TextFormat::simple(font.clone(), crate::ui::tokens::syntax().comment); f.italics = true; f.background = { let a = crate::theme::accent(); Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), 30) }; job.append(&format!("  … {k} Zeilen "), 0.0, f); }
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str) -> Vec<(String, &'static str)> {
        let mn: HashSet<String> = ["lda", "bne", "jmp", "sta"].iter().map(|s| s.to_string()).collect();
        tokens(line, Lang::Asm, &mn).into_iter().map(|(a, b, k)| (line[a..b].to_string(), match k {
            Kind::Plain => "plain", Kind::Comment => "comment", Kind::Str => "str", Kind::Num => "num", Kind::Directive => "dir",
            Kind::Mnemonic => "mn", Kind::Label => "label", Kind::Symbol => "sym", Kind::Keyword => "kw", Kind::Register => "reg",
        })).collect()
    }
    fn has(line: &str, text: &str, kind: &str) -> bool { kinds(line).iter().any(|(t, k)| t == text && *k == kind) }

    #[test]
    fn slash_comments_and_float_numbers() {
        assert!(has("lda #1 // Kommentar", "// Kommentar", "comment"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "2.5", "num"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "1e3", "num"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "1.5e-2", "num"));
        assert!(!kinds(".byte 2.5").iter().any(|(_, k)| *k == "sym"));
    }

    #[test]
    fn anonymous_labels() {
        assert!(has("- dex", "-", "label"));
        assert!(has("+ nop", "+", "label"));
        assert!(has("  bne -", "-", "label"));
        assert!(has("  jmp ++ ; weiter", "++", "label"));
        // Rechenzeichen im Ausdruck bleiben normal
        assert!(!has("  lda #1 - 1", "-", "label"));
        assert!(!has(".byte 1 + 2", "+", "label"));
    }

    #[test]
    fn math_functions_need_a_parenthesis() {
        assert!(has(".byte round(sin(1))", "round", "kw"));
        assert!(has(".byte round(sin(1))", "sin", "kw"));
        assert!(!has(".byte max", "max", "kw"));
        assert!(has(".byte max(1, 2)", "max", "kw"));
    }
}
