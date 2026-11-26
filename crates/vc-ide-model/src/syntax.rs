// SPDX-License-Identifier: GPL-3.0-or-later
//! Zerlegung einer Quelltextzeile in Token-Arten (Assembler, strukturiertes BASIC) – die Farben wählt die Oberfläche.
use crate::lang::Lang;
use std::collections::HashSet;

/// Art eines Tokens
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Plain, Comment, Str, Num, Hex, Bin, Bad, Directive, Mnemonic, Label, Symbol, Keyword, Register }

pub const BASIC_KEYWORDS: &[&str] = &["if", "then", "else", "elseif", "endif", "for", "to", "step", "next", "while", "wend", "do", "loop", "until", "repeat", "const", "include", "gosub", "goto", "return", "print", "poke", "peek", "dim", "data", "read", "restore", "input", "end", "rem", "let", "and", "or", "not", "sub", "endsub", "proc", "endproc", "call", "blob"];

/// Eingebaute Funktionen des Assemblers (nur mit folgender Klammer)
pub const ASM_FUNCTIONS: &[&str] = &["sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sqrt", "pow", "exp", "log", "log2", "log10", "int", "floor", "ceil", "round", "abs", "sign", "min", "max", "len", "range", "next", "align"];

/// Schlüsselwort-Bereiche (Bytes in `line`) einer nummerierten CBM-BASIC-Zeile; leer, wenn die Zeile keine Zeilennummer hat oder kein CBM-Dialekt geladen ist
fn native_keywords(line: &str) -> Vec<(usize, usize)> {
    let t = line.trim_start(); if !t.starts_with(|c: char| c.is_ascii_digit()) { return vec![]; }
    let Some(d) = crate::editor_names::name_dialect() else { return vec![] };
    let crate::editor_names::NameDialect::Cbm(d) = &*d else { return vec![] };
    let lead = line.len() - t.len(); let digits = t.chars().take_while(char::is_ascii_digit).count();
    let off = lead + digits + (t[digits..].len() - t[digits..].trim_start().len());
    vc_basic::keyword_spans(&line[off..], d).into_iter().map(|(a, b)| (a + off, b + off)).collect()
}

/// Ein Wort (Bytebereich `ws..we`), in dem der Tokenizer Schlüsselwörter findet, in Teile zerlegen: Schlüsselwörter, Zahlen, Rest; `None`, wenn keines darin liegt
fn split_run(kw: &[(usize, usize)], ws: usize, we: usize, line: &str) -> Option<Vec<(usize, usize, Kind)>> {
    let inside: Vec<&(usize, usize)> = kw.iter().filter(|(a, e)| *a >= ws && *e <= we).collect();
    if inside.is_empty() { return None; }
    let mut parts = vec![]; let mut p = ws;
    let rest = |a: usize, e: usize, parts: &mut Vec<(usize, usize, Kind)>| { if e > a { parts.push((a, e, if line[a..e].chars().all(|c| c.is_ascii_digit()) { Kind::Num } else { Kind::Plain })); } };
    for (a, e) in inside { rest(p, *a, &mut parts); parts.push((*a, *e, Kind::Keyword)); p = *e; }
    rest(p, we, &mut parts);
    Some(parts)
}

/// Zerlegt eine Zeile in (Bereich, Art)
pub fn tokens(line: &str, lang: Lang, mn: &HashSet<String>, nums: Option<&vc_basic::numbers::Scan>) -> Vec<(usize, usize, Kind)> { tokens_q(line, lang, mn, nums, true) }

/// Wie [`tokens`]; `quote_comment`: im BASIC leitet `'` einen Kommentar ein (Atari, strukturiertes BASIC). In den Commodore-Dialekten ist es kein Kommentar (Warnung `B167`).
pub fn tokens_q(line: &str, lang: Lang, mn: &HashSet<String>, nums: Option<&vc_basic::numbers::Scan>, quote_comment: bool) -> Vec<(usize, usize, Kind)> {
    if lang == Lang::Basic && vc_basic::longnames::is_directive(line) { let a = line.len() - line.trim_start().len(); return vec![(a, line.trim_end().len(), Kind::Directive)]; }   // `#longnames`
    let b: Vec<(usize, char)> = line.char_indices().collect();
    let mut out = vec![]; let mut i = 0; let mut stmt_seen = false;
    let at = |k: usize| b.get(k).map(|x| x.1);
    let pos = |k: usize| b.get(k).map(|x| x.0).unwrap_or(line.len());
    // Nummeriertes BASIC (CBM): Schlüsselwörter stehen auch ohne Leerzeichen (`fori=1to10`); der Tokenizer des Übersetzers sagt, wo (nur mit geladenem Dialekt)
    let kw = if lang == Lang::Basic { native_keywords(line) } else { vec![] };
    while i < b.len() {
        let c = b[i].1; let start = i;
        if c.is_whitespace() { i += 1; continue; }
        if lang == Lang::Asm && (c == ';' || (c == '/' && at(i + 1) == Some('/'))) || lang == Lang::Basic && quote_comment && c == '\'' { out.push((pos(i), line.len(), Kind::Comment)); break; }
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
        // BASIC mit geladenem Dialekt: Hex- und Binärzahlen kommen aus der Erkennung des Übersetzers (Typkennzeichen `A$` sind keine Zahlen)
        if let (Lang::Basic, Some(sc)) = (lang, nums) {
            if c == '$' || c == '%' {
                if let Some(l) = sc.lits.iter().find(|l| l.start == i) {
                    i = l.end.max(i + 1);
                    out.push((pos(start), pos(i), if l.value.is_none() { Kind::Bad } else if l.radix == 16 { Kind::Hex } else { Kind::Bin }));
                } else { i += 1; out.push((pos(start), pos(i), Kind::Plain)); }
                stmt_seen = true; continue;
            }
        }
        if c == '$' || c == '%' && at(i + 1).map_or(false, |d| d == '0' || d == '1') || c.is_ascii_digit() {
            i += 1; while i < b.len() && (b[i].1.is_ascii_alphanumeric() || b[i].1 == '_') { i += 1; }
            if c.is_ascii_digit() { if let Some(parts) = split_run(&kw, pos(start), pos(i), line) { out.extend(parts); continue; } }   // `1to10`: Zahl, Schlüsselwort, Zahl
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
                    if w.starts_with('@') { Kind::Label }
                    else if let Some(parts) = split_run(&kw, pos(start), pos(i), line) { out.extend(parts); continue; }
                    else if BASIC_KEYWORDS.contains(&lw.as_str()) { Kind::Keyword } else { Kind::Plain }
                }
                Lang::Plain | Lang::Markdown => Kind::Plain,
            };
            out.push((pos(start), pos(i), kind)); continue;
        }
        i += 1; out.push((pos(start), pos(i), Kind::Plain));
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str) -> Vec<(String, &'static str)> {
        let mn: HashSet<String> = ["lda", "bne", "jmp", "sta"].iter().map(|s| s.to_string()).collect();
        tokens(line, Lang::Asm, &mn, None).into_iter().map(|(a, b, k)| (line[a..b].to_string(), match k {
            Kind::Plain => "plain", Kind::Comment => "comment", Kind::Str => "str", Kind::Num => "num", Kind::Hex => "hex", Kind::Bin => "bin", Kind::Bad => "bad", Kind::Directive => "dir",
            Kind::Mnemonic => "mn", Kind::Label => "label", Kind::Symbol => "sym", Kind::Keyword => "kw", Kind::Register => "reg",
        })).collect()
    }
    fn has(line: &str, text: &str, kind: &str) -> bool { kinds(line).iter().any(|(t, k)| t == text && *k == kind) }

    #[test]
    fn apostroph_ist_in_commodore_kein_kommentar() {
        let mn = HashSet::new();
        let k = |q: bool| tokens_q("10 print 1 ' hallo", Lang::Basic, &mn, None, q).into_iter().filter(|t| t.2 == Kind::Comment).count();
        assert_eq!((k(true), k(false)), (1, 0));
    }

    #[test]
    fn nummeriertes_basic_ohne_leerzeichen_zeigt_schluesselwoerter() {
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        if !data.join("basic/cbm_v2.dialect.json").exists() { return; }
        crate::editor_names::select_name_dialect(&data, "cbm_v2");
        let mn: HashSet<String> = HashSet::new();
        let kinds = |l: &str| tokens(l, Lang::Basic, &mn, None).into_iter().map(|(a, b, k)| (l[a..b].to_string(), k)).collect::<Vec<_>>();
        let k = kinds("10 fori=1to10");
        assert!(k.contains(&("for".into(), Kind::Keyword)) && k.contains(&("to".into(), Kind::Keyword)), "{k:?}");
        assert!(k.contains(&("i".into(), Kind::Plain)) && k.contains(&("10".into(), Kind::Num)), "{k:?}");
        // mit Leerzeichen wie bisher
        let k = kinds("10 for i = 1 to 10"); assert_eq!(k.iter().filter(|x| x.1 == Kind::Keyword).count(), 2, "{k:?}");
        // ohne Zeilennummer (strukturiertes BASIC): unverändert, ganzes Wort zählt
        let k = kinds("fori = 1"); assert!(!k.iter().any(|x| x.1 == Kind::Keyword), "{k:?}");
    }

    #[test]
    fn slash_comments_and_float_numbers() {
        assert!(has("lda #1 // Kommentar", "// Kommentar", "comment"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "2.5", "num"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "1e3", "num"));
        assert!(has(".byte 2.5, 1e3, 1.5e-2", "1.5e-2", "num"));
        assert!(!kinds(".byte 2.5").iter().any(|(_, k)| *k == "sym"));
    }

    #[test]
    fn basic_numbers_get_their_own_colours_and_suffixes_stay_plain() {
        let d = vc_basic::Dialect::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"), "cbm_v2").unwrap();
        let line = "10 A$=\"$FF\":POKE $D020,%00001110:X=$G";
        let sc = vc_basic::numbers::scan_line(line, &vc_basic::numbers::Lexicon::Cbm(&d), &Default::default());
        let mn = HashSet::new();
        let k: Vec<(String, &str)> = tokens(line, Lang::Basic, &mn, Some(&sc)).into_iter().map(|(a, b, k)| (line[a..b].to_string(), match k { Kind::Hex => "hex", Kind::Bin => "bin", Kind::Bad => "bad", Kind::Str => "str", Kind::Plain => "plain", _ => "other" })).collect();
        let has = |t: &str, kind: &str| k.iter().any(|(x, y)| x == t && *y == kind);
        assert!(has("$D020", "hex") && has("%00001110", "bin") && has("$G", "bad") == false && has("$", "bad"), "{k:?}");
        assert!(has("\"$FF\"", "str"), "Zeichenkette bleibt Text: {k:?}");
        assert!(has("$", "plain") || k.iter().any(|(x, y)| x == "$" && *y == "plain"), "Typkennzeichen A$ ist keine Zahl: {k:?}");
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
