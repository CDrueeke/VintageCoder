// SPDX-License-Identifier: GPL-3.0-or-later
//! Quick-Fix „Kanonischen Namen verwenden“: alte oder kurze Direktivennamen (`.pseudopc`, `.rta`, `.next` …) durch den
//! kanonischen Namen der Tabelle `data/asm/directives.json` ersetzen. Kein Hinweis und keine Diagnose ohne Wunsch:
//! die alten Namen bleiben gleichwertig, der Fix wird nur auf Anfrage angeboten.

/// Eine Ersetzung in einer Zeile (Zeichenspalten, 0-basiert, Ende ausgeschlossen)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasFix { pub line: usize, pub start: usize, pub end: usize, pub old: String, pub new_text: String }

/// Ende des Codes einer Zeile (Spalte in Zeichen) ohne Kommentar `;` oder `//`, Anführungszeichen beachtet
fn code_end(chars: &[char]) -> usize {
    let mut q: Option<char> = None;
    for (i, &c) in chars.iter().enumerate() {
        match q {
            Some(qc) => if c == qc { q = None },
            None => match c {
                '"' | '\'' => q = Some(c),
                ';' => return i,
                '/' if chars.get(i + 1) == Some(&'/') => return i,
                _ => {}
            },
        }
    }
    chars.len()
}

/// Der Direktivenname einer Zeile: (Start, Ende, Wort) in Zeichenspalten; ein Label davor (`name:`) wird übersprungen
pub fn directive_word(line: &str) -> Option<(usize, usize, String)> {
    let chars: Vec<char> = line.chars().collect();
    let end = code_end(&chars);
    let mut i = 0;
    let skip_ws = |i: &mut usize| while *i < end && chars[*i].is_whitespace() { *i += 1 };
    skip_ws(&mut i);
    // Label: Wort mit Doppelpunkt am Ende
    let mut j = i;
    while j < end && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '.') { j += 1; }
    if j > i && j < end && chars[j] == ':' { i = j + 1; skip_ws(&mut i); }
    if i >= end || chars[i] != '.' { return None; }
    let mut k = i;
    while k < end && !chars[k].is_whitespace() && chars[k] != '(' { k += 1; }
    Some((i, k, chars[i..k].iter().collect::<String>()))
}

/// Alle Ersetzungen in `text` (Zeilen `only` = nur diese Zeilen, 0-basiert; `None` = alle)
pub fn alias_fixes(text: &str, only: Option<std::ops::RangeInclusive<usize>>) -> Vec<AliasFix> {
    let mut v = vec![];
    for (n, l) in text.lines().enumerate() {
        if only.as_ref().map_or(false, |r| !r.contains(&n)) { continue; }
        let Some((s, e, w)) = directive_word(l) else { continue };
        let lw = w.to_ascii_lowercase();
        if !vc_asm::directives::is_alias(&lw) { continue; }
        let chars: Vec<char> = l.chars().collect();
        if lw == ".page" {
            // `.page [n]` wird zu `.align n, skip`: Name und Argument zusammen ersetzen
            let end = code_end(&chars);
            let args: String = chars[e..end].iter().collect();
            let (name, a) = vc_asm::directives::normalize(&lw, args.trim());
            v.push(AliasFix { line: n, start: s, end: chars[..end].iter().rposition(|c| !c.is_whitespace()).map_or(e, |p| p + 1).max(e), old: w, new_text: format!("{name} {a}") });
        } else {
            v.push(AliasFix { line: n, start: s, end: e, old: w, new_text: vc_asm::directives::canonical(&lw).unwrap().to_string() });
        }
    }
    v
}
