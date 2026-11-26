// SPDX-License-Identifier: GPL-3.0-or-later
//! Faltbereiche (Start-Zeile → letzte Zeile des Bereichs), Portierung der Regeln der Web-Oberfläche:
//! Blöcke (`.macro`/`.endmacro`, `.if`/`.endif`, BASIC `while`/`wend` …), Label-Abschnitte und Kommentarblöcke ab drei Zeilen.
use crate::lang::Lang;
use std::collections::BTreeMap;

fn code(l: &str, lang: Lang, quote: bool) -> String {
    let cut = if lang == Lang::Basic { if quote { l.find('\'') } else { None } } else { [l.find(';'), l.find("//")].into_iter().flatten().min() };
    cut.map_or(l, |i| &l[..i]).trim().to_ascii_lowercase()
}
fn word(s: &str) -> &str { s.split(|c: char| !(c.is_alphanumeric() || c == '.' || c == '_')).next().unwrap_or("") }

pub fn ranges(text: &str, lang: Lang) -> BTreeMap<usize, usize> { ranges_q(text, lang, true) }

/// Wie [`ranges`]; `quote_comment`: `'` beginnt im BASIC einen Kommentar (nicht in den Commodore-Dialekten)
pub fn ranges_q(text: &str, lang: Lang, quote: bool) -> BTreeMap<usize, usize> {
    let mut out = BTreeMap::new();
    if lang.is_text() { return out; }
    let lines: Vec<&str> = text.split('\n').collect();
    // Assembler: Blockanfänge und -enden stehen in der zentralen Namenstabelle (`data/asm/directives.json`), Alias eingeschlossen
    let (opens, closes): (Vec<&str>, Vec<&str>) = if lang == Lang::Basic { (vec!["while", "repeat", "for", "do", "proc"], vec!["wend", "until", "next", "endif", "loop", "endproc"]) } else { vc_asm::directives::block_words() };
    let (opens, closes) = (&opens[..], &closes[..]);
    let (mut stack, mut secs, mut cm): (Vec<usize>, Vec<usize>, Option<usize>) = (vec![], vec![], None);
    for (i, raw) in lines.iter().enumerate() {
        let c = code(raw, lang, quote); let w = word(&c);
        let is_open = opens.contains(&w) || (lang == Lang::Asm && c.ends_with('{')) || (lang == Lang::Basic && w == "if" && c.ends_with("then")) || (lang == Lang::Basic && c.starts_with("#if"));
        if is_open { stack.push(i); }
        else if closes.contains(&w) || (lang == Lang::Basic && c.starts_with("#endif")) || (lang == Lang::Asm && c.starts_with('}')) { if let Some(st) = stack.pop() { if i > st { out.insert(st, i); } } }
        else if stack.is_empty() && is_label(raw, lang) { secs.push(i); }
        let is_cm = { let t = raw.trim_start(); if lang == Lang::Basic { (quote && t.starts_with('\'')) || t.to_ascii_lowercase().starts_with("rem") } else { t.starts_with(';') || t.starts_with("//") } };
        if is_cm { if cm.is_none() { cm = Some(i); } } else { if let Some(s) = cm { if i - s >= 3 && !out.contains_key(&s) { out.insert(s, i - 1); } } cm = None; }
    }
    for (k, &st) in secs.iter().enumerate() {
        let mut e = secs.get(k + 1).copied().unwrap_or(lines.len()) - 1;
        while e > st && code(lines[e], lang, quote).is_empty() { e -= 1; }
        if e > st && !out.contains_key(&st) { out.insert(st, e); }
    }
    out
}

fn is_label(raw: &str, lang: Lang) -> bool {
    let mut it = raw.chars();
    match it.next() { Some('@') if lang == Lang::Basic => {}, Some(c) if lang == Lang::Asm && (c.is_alphabetic() || c == '_') => {}, _ => return false }
    raw.trim_end().split(|c: char| c.is_whitespace()).next().map_or(false, |w| w.ends_with(':'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folds(src: &str) -> Vec<(usize, usize)> { ranges(src, Lang::Asm).into_iter().filter(|(s, _)| !src.split('\n').nth(*s).map_or(false, |l| l.trim_start().starts_with(';'))).collect() }

    #[test]
    fn blocks_of_the_name_table_fold_with_old_and_new_names() {
        // neue Namen, alte Namen und gemischte Paare falten gleich
        for (open, close) in [(".relocate $0300", ".endrelocate"), (".pseudopc $0300", ".endpseudopc"), (".pseudopc $0300", ".realpc"), (".relocate $0300", ".realpc"),
                              (".for i = 0, i < 4, i += 1", ".endfor"), (".for i = 0, i < 4, i += 1", ".next"), (".for i in range(4)", ".NEXT")] {
            let src = format!("{open}\n nop\n nop\n{close}\n rts\n");
            assert_eq!(folds(&src), vec![(0, 3)], "{open} … {close}");
        }
        // verschachtelt
        let src = ".for i = 0, i < 2, i += 1\n.relocate $0300\n nop\n.endrelocate\n.endfor\n";
        assert_eq!(folds(src), vec![(0, 4), (1, 3)]);
        // alle übrigen Blöcke wie bisher
        for (open, close) in [(".macro m", ".endmacro"), (".if 1", ".endif"), (".repeat 2", ".endrepeat"), (".struct s", ".endstruct"), (".enum e", ".endenum"), (".cycles x", ".endcycles")] {
            assert_eq!(folds(&format!("{open}\n nop\n{close}\n")), vec![(0, 2)], "{open}");
        }
        // BASIC bleibt unberührt: `next` schließt dort `for`
        assert_eq!(ranges("for i = 1 to 3\nprint i\nnext\n", Lang::Basic).get(&0), Some(&2));
        // `'` ist in den Commodore-Dialekten kein Kommentar: drei Zeilen mit `'` bilden keinen Kommentarblock, und ein `'` verdeckt kein `next`
        let rem = "' a\n' b\n' c\n' d\nprint 1\n";
        assert!(ranges_q(rem, Lang::Basic, true).contains_key(&0));
        assert!(!ranges_q(rem, Lang::Basic, false).contains_key(&0));
        assert_eq!(ranges_q("for i=1 to 2 'x\nnext i\n", Lang::Basic, false).get(&0), Some(&1));
    }
}
