// SPDX-License-Identifier: GPL-3.0-or-later
//! Prettyprint im Editor: eine Zeile beim Verlassen (Cursor wechselt die Zeile) und die ganze Datei auf Befehl.
//! Die Formatierung selbst steht rein in `vc_ide_model::pretty`; hier nur das Anwenden auf den Editor (Cursor bleibt, Datei als ein Undo-Schritt).
use std::collections::HashSet;
use vc_gui_editor::core::{Editor, Pos};
use vc_ide_model::{lang::Lang, pretty};

/// Bekannte Mnemonik (klein) für die Assembler-Formatierung
pub fn mnemonik() -> HashSet<String> { vc_gui_editor::highlight::MNEMONICS.iter().map(|s| s.to_string()).collect() }

/// Neue Fassung der Zeile `l`, oder `None`, wenn nichts zu tun ist (andere Sprache, Zeile außerhalb, schon formatiert)
fn neue_zeile(lang: Lang, rel: &str, lines: &[String], l: usize, mn: &HashSet<String>) -> Option<String> {
    let alt = lines.get(l)?;
    let neu = match lang {
        Lang::Asm => pretty::asm_line(alt, mn),
        Lang::Basic => {
            let l_ = rel.to_ascii_lowercase();
            if l_.ends_with(".bas") || l_.ends_with(".lst") || l_.ends_with(".tur") { pretty::native_basic_line(alt) } else { let v: Vec<&str> = lines.iter().map(|s| s.as_str()).collect(); pretty::basic_line(&v, l) }
        }
        _ => return None,
    };
    (neu != *alt).then_some(neu)
}

/// Eine Zeile formatieren; Cursor und Auswahl bleiben, wo sie waren. `true`, wenn sich etwas geändert hat
pub fn zeile_formatieren(ed: &mut Editor, rel: &str, l: usize) -> bool {
    let Some(neu) = neue_zeile(ed.lang, rel, &ed.lines, l, &mnemonik()) else { return false };
    let (cursor, anker) = (ed.cursor, ed.anchor);
    let len = ed.line_len(l);
    ed.replace(Pos { line: l, col: 0 }, Pos { line: l, col: len }, &neu);
    ed.cursor = cursor; ed.anchor = anker; ed.completion = None; true
}

/// Die ganze Datei formatieren, als ein Undo-Schritt; der Cursor bleibt in seiner Zeile (Spalte wird begrenzt)
pub fn datei_formatieren(ed: &mut Editor, rel: &str) -> bool {
    let mn = mnemonik(); let mut zeilen = ed.lines.clone(); let mut geaendert = false;
    for l in 0..zeilen.len() { if let Some(n) = neue_zeile(ed.lang, rel, &zeilen, l, &mn) { zeilen[l] = n; geaendert = true } }
    if !geaendert { return false }
    let cursor = ed.cursor; let letzte = ed.lines.len() - 1; let ende = Pos { line: letzte, col: ed.line_len(letzte) };
    ed.replace(Pos { line: 0, col: 0 }, ende, &zeilen.join("\n"));
    let l = cursor.line.min(ed.lines.len() - 1); ed.cursor = Pos { line: l, col: cursor.col.min(ed.line_len(l)) }; ed.anchor = None; ed.completion = None; true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn editor(rel_lang: Lang, text: &str) -> Editor { let mut e = Editor::new(text); e.lang = rel_lang; e }
    #[test]
    fn zeile_wird_formatiert_cursor_bleibt_und_zweites_mal_nichts() {
        let mut e = editor(Lang::Asm, "start:   lda   #1    ; wert\nrts");
        e.cursor = Pos { line: 1, col: 2 };
        assert!(zeile_formatieren(&mut e, "a.asm", 0));
        assert_eq!(e.cursor, Pos { line: 1, col: 2 }, "Cursor in anderer Zeile bleibt");
        assert!(e.lines[0].starts_with("start:  lda #1") && e.lines[0].ends_with("; wert"), "{:?}", e.lines[0]);
        assert!(!zeile_formatieren(&mut e, "a.asm", 0), "idempotent");
        e.undo(); assert_eq!(e.lines[0], "start:   lda   #1    ; wert", "ein Undo nimmt es zurück");
    }
    #[test]
    fn andere_sprachen_bleiben_unberuehrt() {
        let mut e = editor(Lang::Plain, "a    b"); assert!(!zeile_formatieren(&mut e, "a.txt", 0)); assert!(!datei_formatieren(&mut e, "a.txt"));
        let mut m = editor(Lang::Markdown, "a    b"); assert!(!datei_formatieren(&mut m, "a.md"));
    }
    #[test]
    fn datei_in_einem_undo_schritt() {
        let mut e = editor(Lang::Asm, "a:   lda  #1\nb:    sta   $d020\n"); e.cursor = Pos { line: 1, col: 3 };
        let vorher = e.text();
        assert!(datei_formatieren(&mut e, "a.asm")); assert_ne!(e.text(), vorher); assert_eq!(e.cursor.line, 1);
        e.undo(); assert_eq!(e.text(), vorher, "ganze Datei mit einem Undo zurück");
        assert!(datei_formatieren(&mut e, "a.asm")); assert!(!datei_formatieren(&mut e, "a.asm"), "idempotent");
    }
    #[test]
    fn basic_dateien_nach_endung() {
        let mut e = editor(Lang::Basic, "10   print   \"a  b\""); assert!(zeile_formatieren(&mut e, "x.bas", 0)); assert_eq!(e.lines[0], "10 print \"a  b\"", "{:?}", e.lines[0]);
    }
}
