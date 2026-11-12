// SPDX-License-Identifier: GPL-3.0-or-later
//! Faltbereiche (Start-Zeile → letzte Zeile des Bereichs), Portierung der Regeln der Web-Oberfläche:
//! Blöcke (`.macro`/`.endmacro`, `.if`/`.endif`, BASIC `while`/`wend` …), Label-Abschnitte und Kommentarblöcke ab drei Zeilen.
use crate::highlight::Lang;
use std::collections::BTreeMap;

fn code(l: &str, lang: Lang) -> String {
    let cut = if lang == Lang::Basic { l.find('\'') } else { [l.find(';'), l.find("//")].into_iter().flatten().min() };
    cut.map_or(l, |i| &l[..i]).trim().to_ascii_lowercase()
}
fn word(s: &str) -> &str { s.split(|c: char| !(c.is_alphanumeric() || c == '.' || c == '_')).next().unwrap_or("") }

pub fn ranges(text: &str, lang: Lang) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    if lang == Lang::Plain { return out; }
    let lines: Vec<&str> = text.split('\n').collect();
    let (opens, closes): (&[&str], &[&str]) = if lang == Lang::Basic { (&["while", "repeat", "for", "do", "proc"], &["wend", "until", "next", "endif", "loop", "endproc"]) } else { (&[".macro", ".if", ".ifdef", ".ifndef", ".repeat", ".for", ".struct", ".enum", ".pseudopc", ".cycles"], &[".endmacro", ".endif", ".endrepeat", ".next", ".endstruct", ".endenum", ".endpseudopc", ".endcycles"]) };
    let (mut stack, mut secs, mut cm): (Vec<usize>, Vec<usize>, Option<usize>) = (vec![], vec![], None);
    for (i, raw) in lines.iter().enumerate() {
        let c = code(raw, lang); let w = word(&c);
        let is_open = opens.contains(&w) || (lang == Lang::Asm && c.ends_with('{')) || (lang == Lang::Basic && w == "if" && c.ends_with("then")) || (lang == Lang::Basic && c.starts_with("#if"));
        if is_open { stack.push(i); }
        else if closes.contains(&w) || (lang == Lang::Basic && c.starts_with("#endif")) || (lang == Lang::Asm && c.starts_with('}')) { if let Some(st) = stack.pop() { if i > st { out.insert(st, i); } } }
        else if stack.is_empty() && is_label(raw, lang) { secs.push(i); }
        let is_cm = { let t = raw.trim_start(); if lang == Lang::Basic { t.starts_with('\'') || t.to_ascii_lowercase().starts_with("rem") } else { t.starts_with(';') || t.starts_with("//") } };
        if is_cm { if cm.is_none() { cm = Some(i); } } else { if let Some(s) = cm { if i - s >= 3 && !out.contains_key(&s) { out.insert(s, i - 1); } } cm = None; }
    }
    for (k, &st) in secs.iter().enumerate() {
        let mut e = secs.get(k + 1).copied().unwrap_or(lines.len()) - 1;
        while e > st && code(lines[e], lang).is_empty() { e -= 1; }
        if e > st && !out.contains_key(&st) { out.insert(st, e); }
    }
    out
}

fn is_label(raw: &str, lang: Lang) -> bool {
    let mut it = raw.chars();
    match it.next() { Some('@') if lang == Lang::Basic => {}, Some(c) if lang == Lang::Asm && (c.is_alphabetic() || c == '_') => {}, _ => return false }
    raw.trim_end().split(|c: char| c.is_whitespace()).next().map_or(false, |w| w.ends_with(':'))
}
