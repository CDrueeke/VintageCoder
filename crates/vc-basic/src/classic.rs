// SPDX-License-Identifier: GPL-3.0-or-later
//! Klassischer Tokenizer: nummerierter BASIC-Text → Tokenzeilen (Algorithmus wie im C64-ROM / `petcat`).
use crate::dialect::Dialect;
use crate::petscii;
use vc_core::{msg, Diagnostic, Msg};

/// Schlüsselwort-Suche an Position `i`: erster Treffer in Token-Reihenfolge (Groß-/Kleinschreibung egal).
pub(crate) fn match_keyword(chars: &[char], i: usize, d: &Dialect) -> Option<(Vec<u8>, usize)> {
    let hit = |kw: &String| { let k: Vec<char> = kw.chars().collect(); i + k.len() <= chars.len() && chars[i..i + k.len()].iter().zip(&k).all(|(a, b)| a.to_ascii_uppercase() == *b) };
    let mut it = d.tokens.iter().filter(|(kw, _)| hit(kw));
    let best = if d.match_longest { it.fold(None::<&(String, Vec<u8>)>, |b, t| match b { Some(x) if x.0.chars().count() >= t.0.chars().count() => Some(x), _ => Some(t) }) } else { it.next() };
    best.map(|(kw, t)| (t.clone(), kw.chars().count()))
}

/// Bereiche (Byte-Versätze in `rest`, Teil nach der Zeilennummer), an denen der Tokenizer Schlüsselwörter findet: greedy wie im ROM, also auch ohne Leerzeichen
/// (`fori=1to10` → `for`, `to`). Zeichenketten, `{…}`, `REM`-Rest und `DATA`-Werte enthalten keine. Nur Lesen, ändert nichts am Übersetzen.
pub fn keyword_spans(rest: &str, d: &Dialect) -> Vec<(usize, usize)> {
    let cb: Vec<(usize, char)> = rest.char_indices().collect(); let ch: Vec<char> = cb.iter().map(|x| x.1).collect();
    let byte = |i: usize| cb.get(i).map_or(rest.len(), |x| x.0);
    let (rem, data) = (d.tok("REM"), d.tok("DATA"));
    let (mut out, mut i, mut quote, mut in_data) = (vec![], 0usize, false, false);
    while i < ch.len() {
        let c = ch[i];
        if c == '{' { match ch[i..].iter().position(|x| *x == '}') { Some(p) => { i += p + 1; continue; } None => break } }
        if c == '"' { quote = !quote; i += 1; continue; }
        if quote { i += 1; continue; }
        if in_data { if c == ':' { in_data = false; } i += 1; continue; }
        if c == '?' { out.push((byte(i), byte(i + 1))); i += 1; continue; }
        if let Some((t, n)) = match_keyword(&ch, i, d) {
            out.push((byte(i), byte(i + n))); i += n;
            if t == [rem] { break; }
            if t == [data] { in_data = true; }
            continue;
        }
        i += 1;
    }
    out
}

/// Tokenisiert den Teil einer Zeile nach der Zeilennummer.
pub fn tokenize_line(rest: &str, d: &Dialect) -> Result<Vec<u8>, Msg> {
    let ch: Vec<char> = rest.chars().collect();
    let (rem, data) = (d.tok("REM"), d.tok("DATA"));
    let (mut out, mut i, mut quote, mut in_data) = (vec![], 0, false, false);
    let raw = |c: char, out: &mut Vec<u8>| -> Result<(), Msg> {
        out.push(petscii::from_ascii(c).ok_or_else(|| msg!("diag-basic-b100-petscii-char", c = c))?);
        Ok(())
    };
    while i < ch.len() {
        let c = ch[i];
        if c == '{' {
            let end = ch[i..].iter().position(|x| *x == '}').ok_or_else(|| msg!("diag-basic-b100-brace-missing"))? + i;
            out.extend(petscii::from_text(&ch[i..=end].iter().collect::<String>(), d)?);
            i = end + 1;
            continue;
        }
        if c == '"' { quote = !quote; out.push(0x22); i += 1; continue; }
        if quote { raw(c, &mut out)?; i += 1; continue; }
        if in_data {
            if c == ':' { in_data = false; out.push(b':'); } else if c == ' ' { out.push(b' '); } else { raw(if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c }, &mut out)?; }
            i += 1;
            continue;
        }
        if c == '?' { out.push(d.tok("PRINT")); i += 1; continue; }
        if let Some((t, n)) = match_keyword(&ch, i, d) {
            out.extend(&t);
            i += n;
            if t == [rem] {
                for &x in &ch[i..] { if x == '{' { break; } raw(x, &mut out)?; }
                if let Some(p) = ch[i..].iter().position(|x| *x == '{') { out.extend(petscii::from_text(&ch[i + p..].iter().collect::<String>(), d)?); }
                return Ok(out);
            }
            if t == [data] { in_data = true; }
            continue;
        }
        // Buchstaben außerhalb von Strings: immer als (unumgeschaltete) PETSCII-Großbuchstaben
        let c = if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c };
        raw(c, &mut out)?;
        i += 1;
    }
    Ok(out)
}

/// Zeilen des Formats `NNN text`; leere Zeilen und Kommentarzeilen (`;`) werden übersprungen.
pub fn tokenize_text(src: &str, d: &Dialect, file: &str) -> Result<Vec<(u16, Vec<u8>)>, Vec<Diagnostic>> {
    let (mut lines, mut errs): (Vec<(u16, Vec<u8>)>, Vec<Diagnostic>) = (vec![], vec![]);
    for (n, l) in src.lines().enumerate() {
        let t = l.trim_start();
        if t.is_empty() || t.starts_with(';') { continue; }
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        let Ok(num) = digits.parse::<u32>() else { errs.push(Diagnostic::error("B100", file, n + 1, msg!("diag-basic-b100-no-line-number"))); continue };
        if num > 63999 { errs.push(Diagnostic::error("B100", file, n + 1, msg!("diag-basic-b100-line-number-max-cbm", num = num))); continue; }
        match tokenize_line(t[digits.len()..].trim_start(), d) {
            Ok(b) if b.len() > 250 => errs.push(Diagnostic::error("B102", file, n + 1, msg!("diag-basic-b102-line-too-long"))),
            Ok(b) => {
                match lines.iter().position(|(x, _)| *x == num as u16) { Some(p) => lines[p].1 = b, None => lines.push((num as u16, b)) }
            }
            Err(e) => errs.push(Diagnostic::error("B100", file, n + 1, e)),
        }
    }
    lines.sort_by_key(|(n, _)| *n);
    if errs.is_empty() { Ok(lines) } else { Err(errs) }
}

#[cfg(test)]
mod keyword_span_tests {
    use super::*;
    fn d() -> Option<Dialect> { let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"); Dialect::load(&data, "cbm_v2").ok() }
    fn words(s: &str, d: &Dialect) -> Vec<String> { keyword_spans(s, d).into_iter().map(|(a, b)| s[a..b].to_ascii_lowercase()).filter(|w| w.starts_with(|c: char| c.is_ascii_alphabetic() || c == '?')).collect() }   // Operatoren wie `=` sind auch Token, hier nur Wörter
    #[test] fn findet_schluesselwoerter_ohne_leerzeichen() {
        let Some(d) = d() else { return };
        assert_eq!(words("fori=1to10", &d), ["for", "to"]);
        assert_eq!(words("for i = 1 to 10 step 2", &d), ["for", "to", "step"]);
        assert_eq!(words("printa$:gotoloop", &d), ["print", "goto"]);
        assert_eq!(words("print\"for\":next", &d), ["print", "next"], "in Zeichenketten nichts");
        assert_eq!(words("rem for i", &d), ["rem"], "nach REM nichts");
        assert_eq!(words("data for,to:next", &d), ["data", "next"], "DATA-Werte sind keine Schlüsselwörter");
        assert_eq!(words("?a", &d), ["?"]);
        assert_eq!(words("pokeA,1{clr}for", &d), ["poke", "for"]);
    }
}
