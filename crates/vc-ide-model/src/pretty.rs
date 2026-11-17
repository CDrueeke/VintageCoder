// SPDX-License-Identifier: GPL-3.0-or-later
//! Prettyprint einzelner Zeilen für den Assembler- und BASIC-Editor (angewendet beim Verlassen einer bearbeiteten Zeile).
//!
//! Die Funktionen sind rein (Zeile → Zeile) und idempotent: eine bereits formatierte Zeile bleibt unverändert.
use std::collections::HashSet;

/// Spalte (0-basiert), ab der ein Zeilenkommentar nach Code steht; ist der Code länger, folgt der Kommentar nach einem Leerzeichen
pub const COMMENT_COL: usize = 40;
/// Spalte der Mnemonik hinter einem Label
const MNEMONIC_COL: usize = 8;

/// Trennt Code und Kommentar; Strings (`"…"`) und im Assembler Zeichenliterale (`'x'`) zählen nicht. Kommentar: Assembler `;` oder `//`, BASIC `'`.
fn split_comment(line: &str, asm: bool) -> (&str, Option<&str>) {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let (mut i, mut in_str) = (0, false);
    while i < chars.len() {
        let (pos, c) = chars[i];
        if in_str { if c == '"' { in_str = false; } }
        else if c == '"' { in_str = true; }
        else if asm && c == '\'' && chars.get(i + 2).map_or(false, |x| x.1 == '\'') { i += 2; }
        else if (asm && (c == ';' || (c == '/' && chars.get(i + 1).map_or(false, |x| x.1 == '/')))) || (!asm && c == '\'') { return (&line[..pos], Some(&line[pos..])); }
        i += 1;
    }
    (line, None)
}

/// Mehrfachleerzeichen außerhalb von Strings (und Zeichenliteralen) zu einem zusammenfassen, Anfang und Ende entfernen
fn collapse(s: &str, asm: bool) -> String {
    let chars: Vec<char> = s.chars().collect();
    let (mut out, mut i, mut in_str, mut space) = (String::new(), 0, false, false);
    while i < chars.len() {
        let c = chars[i];
        if in_str { out.push(c); if c == '"' { in_str = false; } }
        else if c.is_whitespace() { space = true; i += 1; continue; }
        else {
            if space && !out.is_empty() { out.push(' '); }
            space = false;
            if c == '"' { in_str = true; out.push(c); }
            else if asm && c == '\'' && chars.get(i + 2) == Some(&'\'') { out.push(c); out.push(chars[i + 1]); out.push('\''); i += 2; }
            else { out.push(c); }
        }
        i += 1;
    }
    out
}

fn is_ident(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' || c == '@' }

/// Code und Kommentar wieder zusammensetzen: Kommentar ab `COMMENT_COL`
fn join_comment(code: String, comment: Option<&str>) -> String {
    match comment {
        None => code,
        Some(c) => { let n = code.chars().count(); let pad = if n < COMMENT_COL { COMMENT_COL - n } else { 1 }; format!("{code}{}{}", " ".repeat(pad), c.trim_end()) }
    }
}

/// Wort am Anfang (Buchstaben, Ziffern, `_`, `.`) und der Rest
fn first_word(s: &str) -> (&str, &str) {
    let end = s.char_indices().find(|(_, c)| !(c.is_alphanumeric() || *c == '_' || *c == '.')).map_or(s.len(), |x| x.0);
    (&s[..end], &s[end..])
}

/// Label am Zeilenanfang (`name:`), auch ohne Leerzeichen dahinter (`name:lda`); liefert Label ohne Doppelpunkt und den Rest
fn split_label(code: &str) -> (Option<&str>, &str) {
    let end = code.char_indices().find(|(_, c)| !is_ident(*c)).map_or(code.len(), |x| x.0);
    if end > 0 && code[end..].starts_with(':') && !code.starts_with(|c: char| c.is_ascii_digit()) { (Some(&code[..end]), code[end + 1..].trim_start()) } else { (None, code) }
}

/// Zuweisung `name = ausdruck` (nicht `==`)
fn is_assignment(s: &str) -> bool {
    let (w, rest) = first_word(s);
    !w.is_empty() && { let r = rest.trim_start(); r.starts_with('=') && !r.starts_with("==") }
}

/// Assembler-Zeile formatieren. `mn`: bekannte Mnemonik (klein geschrieben); bekannte Direktiven kommen aus dem Sprachdienst.
pub fn asm_line(line: &str, mn: &HashSet<String>) -> String {
    let line = line.trim_end();
    let (code, comment) = split_comment(line, true);
    let t = code.trim();
    if t.is_empty() || t.starts_with('{') || t.starts_with('}') { return line.to_string(); }   // Leer-, Kommentar- und Scope-Zeilen: nur rechts getrimmt
    let indent = code.len() - code.trim_start().len();
    let (mut label, mut rest) = split_label(t);
    // anonymes Label: `-` oder `+` (auch mehrfach) allein am Zeilenanfang, ohne Doppelpunkt
    let mut anon = false;
    if label.is_none() {
        if let Some(c) = t.chars().next().filter(|c| *c == '+' || *c == '-') {
            let run = t.chars().take_while(|x| *x == c).count();
            if t[run..].is_empty() || t[run..].starts_with(char::is_whitespace) { label = Some(&t[..run]); rest = t[run..].trim_start(); anon = true; }
        }
    }
    let rest = collapse(rest, true);
    let stmt = if rest.is_empty() { String::new() } else if is_assignment(&rest) { rest.clone() } else {
        let (w, ops) = first_word(&rest);
        let lw = w.to_ascii_lowercase();
        let w = if w.starts_with('.') { if vc_lsp::directive_doc(&lw).is_some() { lw.clone() } else { w.to_string() } } else if mn.contains(&lw) { lw.clone() } else { w.to_string() };
        let ops = ops.trim();
        if ops.is_empty() { w } else { format!("{w} {ops}") }
    };
    let directive_like = rest.starts_with('.') || is_assignment(&rest);
    let out = match (label, stmt.is_empty()) {
        (Some(l), true) => if anon { l.to_string() } else { format!("{l}:") },
        (Some(l), false) => { let head = if anon { l.to_string() } else { format!("{l}:") }; let n = head.chars().count(); let pad = if n < MNEMONIC_COL { MNEMONIC_COL - n } else { 1 }; format!("{head}{}{stmt}", " ".repeat(pad)) }
        (None, _) if directive_like => format!("{}{stmt}", " ".repeat(indent)),
        (None, _) => format!("{}{stmt}", " ".repeat(MNEMONIC_COL)),
    };
    join_comment(out, comment)
}

/// Erstes Wort der Codezeile in Kleinbuchstaben (hinter einem `@label:` bzw. einer Zeilennummer)
fn basic_word(code: &str) -> String {
    let t = code.trim();
    let t = match t.split_whitespace().next() { Some(f) if f.starts_with('@') && f.ends_with(':') => t[f.len()..].trim_start(), _ => t };
    first_word(t).0.to_ascii_lowercase()
}

/// Blocktiefe vor einer Zeile: offene `for`, `while`, `repeat` und mehrzeilige `if … then`
fn basic_depth(lines: &[&str]) -> usize {
    let mut d: usize = 0;
    for l in lines {
        let (code, _) = split_comment(l, false);
        let c = code.trim().to_ascii_lowercase(); if c.is_empty() { continue; }
        let w = basic_word(&c);
        match w.as_str() {
            "for" | "while" | "repeat" => d += 1,
            "if" if c.ends_with("then") => d += 1,
            "next" | "wend" | "until" | "endif" => d = d.saturating_sub(1),
            _ => {}
        }
    }
    d
}

/// Zeile `idx` von strukturiertem BASIC formatieren (Einrückung nach Blocktiefe aus den Zeilen davor)
pub fn basic_line(lines: &[&str], idx: usize) -> String {
    let line = lines[idx].trim_end();
    let (code, comment) = split_comment(line, false);
    let t = code.trim();
    if t.is_empty() || t.to_ascii_lowercase().split_whitespace().next() == Some("rem") { return line.to_string(); }
    let w = basic_word(t);
    let is_label = t.split_whitespace().next().map_or(false, |f| f.starts_with('@') && f.ends_with(':'));
    let depth = basic_depth(&lines[..idx]);
    let levels = if is_label { 0 } else if matches!(w.as_str(), "next" | "wend" | "until" | "endif" | "else") { depth.saturating_sub(1) } else { depth };
    join_comment(format!("{}{}", "  ".repeat(levels), collapse(t, false)), comment)
}

/// Zeile von natives BASIC (`.bas`, `.lst`) formatieren: Mehrfachleerzeichen zusammenfassen, ein Leerzeichen hinter der Zeilennummer; `REM`-Text bleibt unverändert
pub fn native_basic_line(line: &str) -> String {
    let line = line.trim_end();
    let lead = &line[..line.len() - line.trim_start().len()];
    let t = line.trim_start();
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    let (num, rest, lead) = if digits > 0 { (&t[..digits], t[digits..].trim_start(), "") } else { ("", t, lead) };
    let body = if rest.to_ascii_lowercase().starts_with("rem") { rest.to_string() } else { collapse(rest, false) };
    match (num.is_empty(), body.is_empty()) { (true, _) => format!("{lead}{body}"), (false, true) => num.to_string(), (false, false) => format!("{num} {body}") }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mn() -> HashSet<String> { ["lda", "sta", "jmp", "jsr", "ldx", "rts", "nop", "bne"].iter().map(|s| s.to_string()).collect() }
    fn a(l: &str) -> String { asm_line(l, &mn()) }
    fn idem_asm(l: &str) { let once = a(l); assert_eq!(a(&once), once, "nicht idempotent: {l:?}"); }

    #[test]
    fn asm_instructions_labels_and_directives() {
        assert_eq!(a("lda #1"), "        lda #1");
        assert_eq!(a("      LDA   #$01"), "        lda #$01");
        assert_eq!(a("start: lda #0"), "start:  lda #0");
        assert_eq!(a("   start:lda   #0 "), "start:  lda #0");
        assert_eq!(a("long_label_x: sta $d020"), "long_label_x: sta $d020");
        assert_eq!(a(".loop:   dex"), ".loop:  dex");
        assert_eq!(a("@w: bne @w"), "@w:     bne @w");
        assert_eq!(a("start:"), "start:");
        assert_eq!(a(".ORG   $0801"), ".org $0801");
        assert_eq!(a("  .byte 1,  2"), "  .byte 1, 2");
        assert_eq!(a("data:  .byte 1"), "data:   .byte 1");
        assert_eq!(a(".unbekannt  x"), ".unbekannt x");
        // Makroaufrufe: Name unverändert, eingerückt
        assert_eq!(a("MusicInit  a,  b"), "        MusicInit a, b");
        // Zuweisungen
        assert_eq!(a("  VIC   =  $d000"), "  VIC = $d000");
        assert_eq!(a(".const X = 5"), ".const X = 5");
    }

    #[test]
    fn asm_comments_strings_and_scopes() {
        assert_eq!(a("; nur Kommentar   "), "; nur Kommentar");
        assert_eq!(a("   ; eingerückt"), "   ; eingerückt");
        assert_eq!(a(""), "");
        assert_eq!(a("   "), "");
        assert_eq!(a("{  "), "{");
        assert_eq!(a("  }"), "  }");
        // Kommentar ab Spalte 40
        let l = a("lda #1 ; Farbe");
        assert_eq!(l.find(';'), Some(COMMENT_COL), "{l:?}");
        assert!(l.starts_with("        lda #1 "));
        // zu langer Code: ein Leerzeichen
        let long = a("lda $1234567890123456789012345678901234,x ; c");
        assert!(long.contains("x ; c"), "{long}");
        // Semikolon in Strings und Zeichenliteralen ist kein Kommentar
        assert_eq!(a(".text \"a;b\"   ; k").find("; k"), Some(COMMENT_COL));
        assert_eq!(a("lda #';'"), "        lda #';'");
        assert_eq!(a("lda #' '  ; leerzeichen"), format!("        lda #' '{}; leerzeichen", " ".repeat(COMMENT_COL - 16)));
        assert_eq!(a("lda #1 // doppelt").find("//"), Some(COMMENT_COL));
        // Mehrfachleerzeichen in Strings bleiben
        assert_eq!(a(".text \"a    b\""), ".text \"a    b\"");
    }

    #[test]
    fn asm_new_syntax_survives_formatting() {
        // Zwischenstand ausgeben, falls eine Zeile verändert wird
        for l in ["        .var x = 0", "        x += 2", "        .for i = 0, i < 8, i += 1", "        .byte t[1:3], len(t)", "        .byte [1, 2, 3]",
                  "        .function lerp(a, b, t) = a + (b - a) * t / 255", "        .byte round(sin(i * 2 * pi / 256) * 127)", "        bne -", "        jmp ++"] {
            assert_eq!(a(l), l, "Zeile verändert: {l:?}");
        }
        assert_eq!(a("- dex"), "-       dex");
    }

    #[test]
    fn asm_is_idempotent() {
        for l in ["lda #1 ; x", "start:lda  #0", "  .byte 1,2", "MusicInit a,b", ".loop: dex ; c", "VIC = $d000 ; c", "{", "; k", "", "lda #' ' ; k", ".text \"x;y\" ; z"] { idem_asm(l); }
    }

    fn b(src: &str, i: usize) -> String { let lines: Vec<&str> = src.split('\n').collect(); basic_line(&lines, i) }

    #[test]
    fn basic_block_indentation() {
        let src = "for i = 1 to 3\nprint i\nif i = 2 then\nprint \"zwei\"\nelse\nprint \"andere\"\nendif\nnext i\nprint \"fertig\"";
        let out: Vec<String> = (0..src.split('\n').count()).map(|i| b(src, i)).collect();
        assert_eq!(out, ["for i = 1 to 3", "  print i", "  if i = 2 then", "    print \"zwei\"", "  else", "    print \"andere\"", "  endif", "next i", "print \"fertig\""]);
        // while / repeat
        let src = "while x < 3\nrepeat\nx = x + 1\nuntil x > 2\nwend";
        let out: Vec<String> = (0..5).map(|i| b(src, i)).collect();
        assert_eq!(out, ["while x < 3", "  repeat", "    x = x + 1", "  until x > 2", "wend"]);
        // einzeiliges If öffnet keinen Block
        let src = "if a then print 1\nprint 2";
        assert_eq!((b(src, 0), b(src, 1)), ("if a then print 1".to_string(), "print 2".to_string()));
    }

    #[test]
    fn basic_labels_spaces_strings_and_comments() {
        assert_eq!(b("for i=1 to 3\n@loop:   print   i", 1), "@loop: print i");
        assert_eq!(b("@top:", 0), "@top:");
        assert_eq!(b("print   \"a    b{clr}\"  ;  x", 0), "print \"a    b{clr}\" ; x");
        assert_eq!(b("print \"x'y\"   ' kommentar", 0).find("' kommentar"), Some(COMMENT_COL));
        assert_eq!(b("  ' kommentar  ", 0), "  ' kommentar");
        assert_eq!(b("rem   alles   bleibt", 0), "rem   alles   bleibt");
        assert_eq!(b("", 0), "");
        assert_eq!(b("const  ROT  =  2", 0), "const ROT = 2");
    }

    #[test]
    fn basic_is_idempotent() {
        let src = "for i=1   to 3\n  print i ' x\n if i then\nnext i\n@l:  print   1\nendif";
        let lines: Vec<&str> = src.split('\n').collect();
        let once: Vec<String> = (0..lines.len()).map(|i| basic_line(&lines, i)).collect();
        let l2: Vec<&str> = once.iter().map(|s| s.as_str()).collect();
        let twice: Vec<String> = (0..l2.len()).map(|i| basic_line(&l2, i)).collect();
        assert_eq!(once, twice);
    }

    #[test]
    fn native_basic() {
        assert_eq!(native_basic_line("10   print   \"a  b\"  "), "10 print \"a  b\"");
        assert_eq!(native_basic_line("  20 goto   10"), "20 goto 10");
        assert_eq!(native_basic_line("30"), "30");
        assert_eq!(native_basic_line("40 rem   viel   platz  "), "40 rem   viel   platz");
        assert_eq!(native_basic_line("  print   x"), "  print x");
        assert_eq!(native_basic_line("print x"), "print x");
        let once = native_basic_line("10   for  i=1   to 3"); assert_eq!(native_basic_line(&once), once);
    }
}
