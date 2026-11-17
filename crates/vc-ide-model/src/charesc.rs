// SPDX-License-Identifier: GPL-3.0-or-later
//! `{name}`-Angaben in Zeichenketten des Quelltexts (`{clr}`, `{rvs on}`, `{$93}`, `{down*3}`): finden, prüfen, beschreiben.
//! Grundlage für Färbung, Hover, Vervollständigung und Diagnose im Editor. Die Namen kommen aus `vc_core::chars` (`data/chars/`).
use crate::lang::Lang;
use std::sync::OnceLock;
use vc_core::chars::{self, Family, Lang as CLang};

/// Eine `{…}`-Angabe: Byte-Bereich in der Zeile (mit Klammern) und Ergebnis der Prüfung
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub a: usize,
    pub z: usize,
    /// `Ok(Farbnummer 0..15 bei Farbcodes)` oder die Fehlermeldung
    pub res: Result<Option<u8>, String>,
}

/// Familie der Namen für `{…}` in dieser Zeile
pub fn family(lang: Lang, line: &str) -> Family {
    match lang {
        Lang::Basic => match crate::editor_names::name_dialect().as_deref() { Some(crate::editor_names::NameDialect::Atari(_)) => Family::Atascii, _ => Family::Petscii },
        _ => if directive_word(line).as_deref() == Some(".atascii") { Family::Atascii } else { Family::Petscii },
    }
}

/// Erstes Wort der Anweisung (nach einem Label), klein
fn directive_word(line: &str) -> Option<String> {
    let t = line.trim_start();
    let t = match t.find(':') { Some(i) if t[..i].chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') => t[i + 1..].trim_start(), _ => t };
    t.split_whitespace().next().map(|w| w.to_ascii_lowercase())
}

/// Ist das eine Zeile mit Textdirektive, in der `{name}` gilt (`.petscii`, `.atascii`, `.screen`, `.text`, `.null`, `.ptext`, `.shift`)?
pub fn asm_text_line(line: &str) -> bool {
    matches!(directive_word(line).as_deref(), Some(".petscii" | ".atascii" | ".screen" | ".text" | ".null" | ".ptext" | ".shift"))
}

/// Zeichenketten der Zeile als Byte-Bereiche (mit Anführungszeichen). Kommentare (`;`, `//`, BASIC: `'`) zählen nicht.
pub fn strings(line: &str, lang: Lang) -> Vec<(usize, usize)> {
    let mut out = vec![];
    let b: Vec<(usize, char)> = line.char_indices().collect();
    let mut i = 0;
    while i < b.len() {
        let c = b[i].1;
        if lang == Lang::Asm && (c == ';' || (c == '/' && b.get(i + 1).map(|x| x.1) == Some('/'))) || lang == Lang::Basic && c == '\'' { break; }
        if c == '"' {
            let start = i; i += 1;
            while i < b.len() && b[i].1 != '"' { i += 1; }
            let end = if i < b.len() { b[i].0 + 1 } else { line.len() };
            out.push((b[start].0, end));
            i += 1; continue;
        }
        i += 1;
    }
    out
}

/// Alle `{…}`-Angaben in den Zeichenketten der Zeile. Im Assembler nur in Textdirektiven.
pub fn spans(line: &str, lang: Lang) -> Vec<Span> {
    if !line.contains('{') || lang.is_text() || (lang == Lang::Asm && !asm_text_line(line)) { return vec![]; }
    let fam = family(lang, line);
    let mut out = vec![];
    for (sa, sz) in strings(line, lang) {
        let s = &line[sa..sz];
        let mut i = 0;
        while let Some(p) = s[i..].find('{') {
            let a = i + p;
            // `{{` ergibt ein einzelnes `{` (Assembler, Atari BASIC); im CBM-BASIC gibt es kein `{` im PETSCII
            if s[a + 1..].starts_with('{') {
                if lang == Lang::Basic && fam == Family::Petscii { out.push(Span { a: sa + a, z: sa + a + 2, res: Err("{{ gibt es im PETSCII nicht".into()) }); }
                i = a + 2; continue;
            }
            match s[a + 1..].find('}') {
                None => { out.push(Span { a: sa + a, z: sa + s.trim_end_matches('"').len().max(a + 1), res: Err("'}' fehlt".into()) }); break; }
                Some(e) => {
                    let inner = &s[a + 1..a + 1 + e];
                    let res = chars::parse_ref(fam, inner, &|_| None).map(|(code, _)| chars::by_code(fam, code).and_then(|c| c.color));
                    out.push(Span { a: sa + a, z: sa + a + e + 2, res });
                    i = a + e + 2;
                }
            }
        }
    }
    out
}

static PALETTE: OnceLock<Vec<[u8; 3]>> = OnceLock::new();

/// Farben der Farbcodes (C64-Palette); einmal setzen, sobald der Datenordner bekannt ist
pub fn set_palette(colors: Vec<[u8; 3]>) { let _ = PALETTE.set(colors); }
pub fn have_palette() -> bool { PALETTE.get().is_some() }
pub fn palette_color(n: u8) -> Option<[u8; 3]> { PALETTE.get().and_then(|p| p.get(n as usize).copied()) }

/// Hover-Text (Markdown) zur Angabe `{inner}` in der Zeile
pub fn hover(lang: Lang, line: &str, inner: &str) -> Option<String> {
    let fam = family(lang, line);
    let l = if lang == Lang::Basic { CLang::Basic } else { CLang::Asm };
    match chars::parse_ref(fam, inner, &|_| None) {
        Ok((code, count)) => {
            let e = chars::by_code(fam, code);
            let title = match e { Some(c) => format!("**{{{}}}**  –  {}", c.main(l), c.desc), None => format!("**{{${code:02x}}}**  –  Code ohne Namen") };
            let mut md = format!("{title}\n\n| | |\n|---|---|\n| Code | `${code:02X}` = {code} |\n");
            if count > 1 { md.push_str(&format!("| Wiederholung | {count}× |\n")); }
            if let Some(c) = e {
                let all = c.names(l);
                if all.len() > 1 { md.push_str(&format!("| Auch | {} |\n", all[1..].iter().map(|n| format!("`{{{n}}}`")).collect::<Vec<_>>().join(" "))); }
                if let Some(n) = c.color { md.push_str(&format!("| Farbe | Nr. {n} |\n")); }
            }
            Some(md)
        }
        Err(m) => Some(format!("**Fehler:** {m}")),
    }
}

/// Offenes `{` vor dem Cursor: Zeichenindex hinter der Klammer, wenn danach nur Namenszeichen stehen
/// (`before` = Text der Zeile vor dem Cursor)
pub fn open_brace(before: &str) -> Option<usize> {
    let ch: Vec<char> = before.chars().collect();
    let p = ch.iter().rposition(|c| *c == '{')?;
    if p > 0 && ch[p - 1] == '{' { return None; }
    if !ch[p + 1..].iter().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '$' | '*')) { return None; }
    Some(p + 1)
}

/// Namen für die Vervollständigung nach `{`: (Name, Beschreibung mit Code); Hauptname der Sprache zuerst, dann Aliase
pub fn completions(lang: Lang, fam: Family) -> Vec<(String, String)> {
    let l = if lang == Lang::Basic { CLang::Basic } else { CLang::Asm };
    let mut mains = vec![]; let mut others = vec![];
    for c in chars::table(fam) {
        let names = c.names(l);
        mains.push((names[0].to_string(), format!("${:02X}  {}", c.code, c.desc)));
        for n in &names[1..] { others.push((n.to_string(), format!("= {{{}}}", names[0]))); }
    }
    mains.extend(others);
    mains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_skip_comments() {
        assert_eq!(strings("10 PRINT \"A\";\"B\"", Lang::Basic), vec![(9, 12), (13, 16)]);
        assert_eq!(strings("10 PRINT \"A\" ' \"B\"", Lang::Basic), vec![(9, 12)]);
        assert_eq!(strings("  .petscii \"hi\" ; \"x\"", Lang::Asm), vec![(11, 15)]);
        assert_eq!(strings("print \"offen", Lang::Basic), vec![(6, 12)]);
    }

    #[test]
    fn spans_in_basic_and_assembler() {
        let l = "10 PRINT \"{clr}A{orange}{nix}{down*3}\"";
        let v = spans(l, Lang::Basic);
        let t: Vec<(&str, bool, Option<Option<u8>>)> = v.iter().map(|s| (&l[s.a..s.z], s.res.is_ok(), s.res.clone().ok())).collect();
        assert_eq!(t, vec![("{clr}", true, Some(None)), ("{orange}", true, Some(Some(8))), ("{nix}", false, None), ("{down*3}", true, Some(None))]);
        assert!(v[2].res.clone().unwrap_err().contains("unbekannter Steuercode"));
        // Assembler: nur in Textdirektiven
        assert_eq!(spans("  .petscii \"{clr}{red}\"", Lang::Asm).len(), 2);
        assert!(spans("  lda #\"{clr}\"", Lang::Asm).is_empty());
        assert!(spans("  .byte \"{clr}\"", Lang::Asm).is_empty());
        assert_eq!(spans("msg: .text \"a{$c1}\"", Lang::Asm).len(), 1);
        // doppelte Klammer ist keine Angabe (Assembler); offene Klammer ist ein Fehler
        assert!(spans("  .petscii \"a{{b\"", Lang::Asm).is_empty());
        let v = spans("  .petscii \"a{clr\"", Lang::Asm);
        assert_eq!(v.len(), 1);
        assert!(v[0].res.is_err());
        // außerhalb von Anführungszeichen keine Angabe
        assert!(spans("10 PRINT {clr}", Lang::Basic).is_empty());
    }

    #[test]
    fn open_braces_before_the_cursor() {
        assert_eq!(open_brace("10 PRINT \"{"), Some(11));
        assert_eq!(open_brace("10 PRINT \"{rvs o"), Some(11));
        assert_eq!(open_brace("10 PRINT \"{clr}"), None, "geschlossen");
        assert_eq!(open_brace("10 PRINT \"{clr}{do"), Some(16));
        assert_eq!(open_brace("a{{"), None, "doppelte Klammer");
        assert_eq!(open_brace("10 PRINT \"{a\"b"), None, "Anführungszeichen beendet den Namen");
        assert_eq!(open_brace("nichts"), None);
    }

    #[test]
    fn hover_text_names_code_and_spellings() {
        let h = hover(Lang::Basic, "10 PRINT \"{orng}\"", "orng").unwrap();
        assert!(h.starts_with("**{orng}**") && h.contains("| Code | `$81` = 129 |") && h.contains("`{orange}`") && h.contains("Farbe | Nr. 8"), "{h}");
        let h = hover(Lang::Asm, "  .petscii \"{down*3}\"", "down*3").unwrap();
        assert!(h.contains("| Wiederholung | 3× |") && h.contains("Cursor runter"), "{h}");
        assert!(hover(Lang::Basic, "", "nix").unwrap().starts_with("**Fehler:**"));
        assert!(hover(Lang::Asm, "  .petscii \"\"", "$ff").unwrap().contains("`$FF`"));
    }

    #[test]
    fn completions_list_main_names_first_then_spellings() {
        let b = completions(Lang::Basic, Family::Petscii);
        assert!(b.iter().any(|(n, d)| n == "orng" && d.starts_with("$81")));
        assert!(b.iter().any(|(n, d)| n == "orange" && d == "= {orng}"));
        let a = completions(Lang::Asm, Family::Petscii);
        assert!(a.iter().any(|(n, _)| n == "orange") && a.iter().any(|(n, d)| n == "orng" && d == "= {orange}"));
        assert!(completions(Lang::Asm, Family::Atascii).iter().any(|(n, _)| n == "eol"));
    }
}
