// SPDX-License-Identifier: GPL-3.0-or-later
//! Reine Textfunktionen des Editors: Zeilenänderungen, Wortgrenzen, Kommentar umschalten, Include-Zeilen, Hover-Ziele – ohne GUI.
use crate::editor_names::include_span;
use crate::lang::Lang;

/// Fasst die Anzeigezeilen (Reihen) einer Textansicht zu logischen Zeilen zusammen: je Zeile (erste, letzte Reihe).
/// `ends_with_newline` je Reihe; ohne Umbruch ist jede Reihe genau eine Zeile.
pub fn line_row_spans(ends_with_newline: impl IntoIterator<Item = bool>) -> Vec<(usize, usize)> {
    let mut out = Vec::new(); let mut first = None; let mut last = 0;
    for (i, nl) in ends_with_newline.into_iter().enumerate() {
        if first.is_none() { first = Some(i); }
        last = i;
        if nl { out.push((first.take().unwrap_or(i), i)); }
    }
    if let Some(f) = first { out.push((f, last)); }
    out
}

/// Änderung an ganzen Zeilen (0-basierte Zeilenindizes)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineEdit {
    /// `text` als neue Zeile vor Zeile `at` (hinter dem Ende: anhängen)
    Insert { at: usize, text: String },
    /// Zeile `at` von `old` auf `new` setzen, falls sie noch `old` lautet
    Replace { at: usize, old: String, new: String },
    /// Zeile `at` löschen, falls sie noch `old` lautet
    Remove { at: usize, old: String },
}

/// Wendet `e` auf den Text an; liefert den neuen Text, den Zeilenindex der Änderung und den Zeichenindex hinter der Zeile. `None`: nichts zu tun.
pub fn apply_line_edit(text: &str, e: &LineEdit) -> Option<(String, usize, usize)> {
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let at = match e {
        LineEdit::Insert { at, text: t } => {
            let at = if *at >= lines.len() { if lines.last().map_or(true, |l| l.is_empty()) { lines.len().saturating_sub(1) } else { lines.len() } } else { *at };
            lines.insert(at, t.clone()); at
        }
        LineEdit::Replace { at, old, new } => { if lines.get(*at) != Some(old) { return None; } lines[*at] = new.clone(); *at }
        LineEdit::Remove { at, old } => { if lines.get(*at) != Some(old) { return None; } lines.remove(*at); (*at).min(lines.len().saturating_sub(1)) }
    };
    let end: usize = lines[..at].iter().map(|l| l.chars().count() + 1).sum::<usize>() + if matches!(e, LineEdit::Remove { .. }) { 0 } else { lines[at].chars().count() };
    Some((lines.join("\n"), at, end))
}

pub fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' || c == '$' }

/// Anfang (Zeichenindex) des Wortes vor dem Cursor
pub fn word_start(text: &str, cursor: usize) -> usize {
    let chars: Vec<char> = text.chars().collect(); let mut i = cursor.min(chars.len());
    while i > 0 && is_word(chars[i - 1]) { i -= 1; } i
}

/// Zeichenindex hinter einem offenen `{` vor dem Cursor, wenn der Cursor in einer Zeichenkette steht
pub fn esc_start(text: &str, cur: usize) -> Option<usize> {
    let before: String = text.chars().take(cur).collect();
    let line = before.rsplit('\n').next()?;
    let p = crate::charesc::open_brace(line)?;
    Some(cur - line.chars().count() + p)
}

/// `{…}`-Angabe in einer Zeichenkette, die das Zeichenfeld `col` (Zeichenindex) enthält: (Anfang, Ende) als Zeichenindizes
pub fn esc_target(line: &str, lang: Lang, col: usize) -> Option<(usize, usize)> {
    let byte = line.char_indices().nth(col).map_or(line.len(), |x| x.0);
    let sp = crate::charesc::spans(line, lang).into_iter().find(|s| s.a <= byte && byte < s.z)?;
    Some((line[..sp.a].chars().count(), line[..sp.z].chars().count()))
}

/// Inhalt der `{…}`-Angabe, die genau die Zeichen `s..e` umfasst
pub fn esc_inner(line: &str, lang: Lang, s: usize, e: usize) -> Option<String> {
    let t: String = line.chars().skip(s).take(e - s).collect();
    if !(t.starts_with('{') && t.ends_with('}') && t.len() >= 2) { return None; }
    crate::charesc::spans(line, lang).iter().any(|sp| line[..sp.a].chars().count() == s && line[..sp.z].chars().count() == e).then(|| t[1..t.len() - 1].to_string())
}

/// Ziel des Hovers in einer Zeile: Zahlenliteral oder Bezeichner (Zeichenindizes Anfang/Ende)
pub fn hover_target(line: &str, col: usize) -> Option<(usize, usize)> {
    if let Some((s, e, _)) = vc_lsp::literal_at(line, col) { return Some((s, e)); }
    let chars: Vec<char> = line.chars().collect();
    let mut c = col.min(chars.len());
    let id = |ch: char| ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '@';
    if (c >= chars.len() || !id(chars[c])) && c > 0 && id(chars[c - 1]) { c -= 1; }
    if c >= chars.len() || !id(chars[c]) { return None; }
    let (mut s, mut e) = (c, c + 1);
    while s > 0 && id(chars[s - 1]) { s -= 1; }
    while e < chars.len() && id(chars[e]) { e += 1; }
    Some((s, e))
}

/// `const NAME = zahl` aus strukturiertem BASIC (nur Zahlenwerte)
pub fn basic_consts(text: &str) -> Vec<(String, i64)> {
    text.lines().filter_map(|l| {
        let code = l.split('\'').next().unwrap_or("").trim();
        let rest = code.strip_prefix("const ").or_else(|| code.strip_prefix("CONST "))?;
        let (name, expr) = rest.split_once('=')?;
        let expr = expr.trim();
        let (s, e, v) = vc_lsp::literal_at(expr, 0)?;
        (s == 0 && e == expr.chars().count()).then(|| (name.trim().to_string(), v))
    }).collect()
}

/// Wortgrenzen um das Zeichen `idx` (Zeichenindizes)
pub fn word_bounds(chars: &[char], idx: usize) -> (usize, usize) {
    let mut a = idx.min(chars.len()); let mut b = a;
    while a > 0 && is_word(chars[a - 1]) { a -= 1; }
    while b < chars.len() && is_word(chars[b]) { b += 1; }
    (a, b)
}

pub fn include_path(line: &str) -> Option<String> { include_span(line).map(|x| x.0) }

pub fn comment_prefix(rel: &str, lang: Lang) -> &'static str { if lang == Lang::Basic && rel.ends_with(".sbas") { "'" } else { ";" } }

/// Zeilen `a..=b` auskommentieren oder – wenn schon alle (nicht leeren) kommentiert sind – die Kommentarzeichen entfernen
pub fn toggle_comment(text: &str, a: usize, b: usize, prefix: &str) -> String {
    let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
    let b = b.min(lines.len() - 1);
    let all = (a..=b).filter(|i| !lines[*i].trim().is_empty()).all(|i| lines[i].trim_start().starts_with(prefix));
    for i in a..=b {
        let l = lines[i].clone();
        if l.trim().is_empty() { continue; }
        let indent = l.len() - l.trim_start().len();
        let (head, rest) = l.split_at(indent);
        lines[i] = if all { let r = rest.strip_prefix(prefix).unwrap_or(rest); format!("{head}{}", r.strip_prefix(' ').unwrap_or(r)) } else { format!("{head}{prefix} {rest}") };
    }
    lines.join("\n")
}

/// Ordnet Zeilen des aktuellen Textes den Zeilen des Textes zu, zu dem die Adressen gehören (gleicher Anfang und gleiches Ende; dazwischen: unbekannt)
pub fn map_to_old(old: &[String], cur: &[&str]) -> impl Fn(usize) -> Option<usize> {
    let p = old.iter().zip(cur).take_while(|(a, b)| a.as_str() == **b).count();
    let room = old.len().min(cur.len()) - p;
    let s = old.iter().rev().zip(cur.iter().rev()).take_while(|(a, b)| a.as_str() == **b).count().min(room);
    let (olen, clen) = (old.len() as isize, cur.len() as isize);
    move |i| if i < p { Some(i) } else if i as isize >= clen - s as isize { let j = i as isize + olen - clen; if j >= 0 { Some(j as usize) } else { None } } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_toggle_adds_and_removes_markers_keeping_indent() {
        let t = "start:  sei\n        lda #1\n\n        sta $d020\n";
        let c = toggle_comment(t, 1, 3, ";");
        assert_eq!(c, "start:  sei\n        ; lda #1\n\n        ; sta $d020\n");
        assert_eq!(toggle_comment(&c, 1, 3, ";"), t, "zweimal umschalten ergibt den Ausgangstext");
        // gemischt (nur eine Zeile kommentiert): alle werden kommentiert
        assert_eq!(toggle_comment("a\n; b\n", 0, 1, ";"), "; a\n; ; b\n");
        assert_eq!(toggle_comment("print 1\n", 0, 5, "'"), "' print 1\n", "Bereich über das Textende hinaus wird begrenzt");
    }

    #[test]
    fn include_path_is_taken_from_include_lines() {
        assert_eq!(include_path("  .include \"lib/music.inc\"  ; Musik").as_deref(), Some("lib/music.inc"));
        assert_eq!(include_path("include \"../x.sbas\"").as_deref(), Some("../x.sbas"));
        assert_eq!(include_path("lda #1"), None);
        assert_eq!(include_path("#include \"a.inc\""), Some("a.inc".into()));
        assert_eq!(include_path("#include <c64/kernal.asm>"), None);
        assert_eq!(include_span("  .incbin \"x.bin\",2"), Some(("x.bin".into(), 11, 16)));
        assert_eq!(include_path(".include broken"), None);
    }

    #[test]
    fn line_edits_insert_replace_remove() {
        use LineEdit::*;
        let t = "a\nb\n";
        let ins = |at: usize| apply_line_edit(t, &Insert { at, text: "X".into() }).unwrap();
        assert_eq!(ins(0), ("X\na\nb\n".into(), 0, 1));
        assert_eq!(ins(1), ("a\nX\nb\n".into(), 1, 3));
        assert_eq!(ins(2).0, "a\nb\nX\n", "vor dem leeren Rest nach dem letzten Zeilenumbruch");
        assert_eq!(ins(99).0, "a\nb\nX\n");
        assert_eq!(apply_line_edit("a\nb", &Insert { at: 5, text: "X".into() }).unwrap().0, "a\nb\nX", "ohne Zeilenumbruch am Ende anhängen");
        assert_eq!(apply_line_edit("", &Insert { at: 0, text: "X".into() }).unwrap().0, "X\n");
        assert_eq!(apply_line_edit(t, &Replace { at: 1, old: "b".into(), new: "Y".into() }).unwrap().0, "a\nY\n");
        assert!(apply_line_edit(t, &Replace { at: 1, old: "q".into(), new: "Y".into() }).is_none(), "geänderte Zeile bleibt unberührt");
        assert_eq!(apply_line_edit(t, &Remove { at: 0, old: "a".into() }).unwrap().0, "b\n");
        assert!(apply_line_edit(t, &Remove { at: 0, old: "b".into() }).is_none());
    }

    #[test]
    fn word_bounds_cover_labels_dots_and_dollars() {
        let c: Vec<char> = "  jsr .loop_1 ; $d020".chars().collect();
        let w = |i: usize| { let (a, b) = word_bounds(&c, i); c[a..b].iter().collect::<String>() };
        assert_eq!(w(7), ".loop_1"); assert_eq!(w(2), "jsr"); assert_eq!(w(16), "$d020"); assert_eq!(w(0), "");
    }

    #[test]
    fn comment_prefix_depends_on_dialect() {
        assert_eq!(comment_prefix("a.asm", Lang::Asm), ";"); assert_eq!(comment_prefix("a.sbas", Lang::Basic), "'"); assert_eq!(comment_prefix("a.bas", Lang::Basic), ";");
    }

    #[test]
    fn hover_targets_are_literals_or_identifiers() {
        assert_eq!(hover_target("lda #$d020", 7), Some((5, 10)));
        assert_eq!(hover_target("        sta vic2.border", 14), Some((12, 23)), "Bezeichner mit Punkt");
        assert_eq!(hover_target("lda #LIVES", 6), Some((5, 10)));
        assert_eq!(hover_target("lda #LIVES", 4), None, "auf # und Leerzeichen kein Ziel");
        assert_eq!(hover_target("lda #255", 6), Some((5, 8)));
        assert_eq!(hover_target("", 0), None);
    }

    #[test]
    fn basic_constants_are_found_with_their_values() {
        let text = "' Kommentar\nconst PLAYER = 8192\nconst COLORRAM = $d800   ' Farb-RAM\nconst MIX = PLAYER + 1\nconst BITS = %1010\nprint PLAYER";
        let c = basic_consts(text);
        assert_eq!(c, vec![("PLAYER".to_string(), 8192), ("COLORRAM".to_string(), 0xD800), ("BITS".to_string(), 10)], "Ausdrücke bleiben unberücksichtigt");
    }

    #[test]
    fn row_spans_group_rows_into_lines() {
        assert_eq!(line_row_spans([true, true, true]), vec![(0, 0), (1, 1), (2, 2)], "ohne Umbruch");
        assert_eq!(line_row_spans([false, false, true, true, false]), vec![(0, 2), (3, 3), (4, 4)], "Umbruch: erste Zeile hat drei Reihen");
        assert_eq!(line_row_spans([false]), vec![(0, 0)]);
        assert!(line_row_spans(std::iter::empty()).is_empty());
    }
}
