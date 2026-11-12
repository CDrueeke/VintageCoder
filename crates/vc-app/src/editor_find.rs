// SPDX-License-Identifier: GPL-3.0-or-later
//! Suchen, Ersetzen und „Gehe zu Zeile“ im Quelltext-Editor: reine Textfunktionen (Zeichenpositionen, nicht Bytes) und der Zustand der Leiste.

/// Zustand der Suchleiste eines Dokuments
#[derive(Default, Clone)]
pub struct Find {
    pub open: bool,
    /// Mit Ersetzen-Feld
    pub replace_mode: bool,
    pub query: String,
    pub replace: String,
    pub case: bool,
    /// Eingabefeld beim nächsten Zeichnen fokussieren
    pub focus: bool,
    /// Ergebnis der letzten Aktion („3 Treffer“, „kein Treffer“, „2 ersetzt“)
    pub status: String,
}

/// Zustand von „Gehe zu Zeile“
#[derive(Default, Clone)]
pub struct Goto { pub open: bool, pub input: String, pub focus: bool }

fn norm(s: &str, case: bool) -> Vec<char> { if case { s.chars().collect() } else { s.chars().flat_map(|c| c.to_lowercase()).collect() } }

/// Alle Treffer als Zeichenbereiche `(von, bis)`, ohne Überlappung. Groß-/Kleinschreibung nur bei `case`.
pub fn find_all(text: &str, q: &str, case: bool) -> Vec<(usize, usize)> {
    if q.is_empty() { return vec![]; }
    let (t, p) = (norm(text, case), norm(q, case));
    if t.len() < p.len() || t.len() != text.chars().count() { return find_exact_len(text, q, case); }
    let mut out = vec![]; let mut i = 0;
    while i + p.len() <= t.len() { if t[i..i + p.len()] == p[..] { out.push((i, i + p.len())); i += p.len(); } else { i += 1; } }
    out
}

/// Rückfall, wenn das Kleinschreiben die Länge ändert (seltene Sonderzeichen): zeichenweise vergleichen
fn find_exact_len(text: &str, q: &str, case: bool) -> Vec<(usize, usize)> {
    let one = |c: char| if case { c } else { c.to_lowercase().next().unwrap_or(c) };
    let t: Vec<char> = text.chars().map(one).collect(); let p: Vec<char> = q.chars().map(one).collect();
    let mut out = vec![]; let mut i = 0;
    while !p.is_empty() && i + p.len() <= t.len() { if t[i..i + p.len()] == p[..] { out.push((i, i + p.len())); i += p.len(); } else { i += 1; } }
    out
}

/// Nächster Treffer ab `from` (vorwärts: erster Treffer mit Beginn ≥ `from`, sonst von vorn; rückwärts: letzter Treffer mit Ende ≤ `from`, sonst von hinten)
pub fn next_match(matches: &[(usize, usize)], from: usize, forward: bool) -> Option<(usize, usize)> {
    if matches.is_empty() { return None; }
    if forward { matches.iter().find(|m| m.0 >= from).or(matches.first()).copied() } else { matches.iter().rev().find(|m| m.1 <= from).or(matches.last()).copied() }
}

fn byte_at(text: &str, ch: usize) -> usize { text.char_indices().nth(ch).map_or(text.len(), |(b, _)| b) }

/// Einen Zeichenbereich ersetzen
pub fn replace_range(text: &mut String, r: (usize, usize), with: &str) { let (a, b) = (byte_at(text, r.0), byte_at(text, r.1)); text.replace_range(a..b, with); }

/// Alle Treffer ersetzen; liefert die Anzahl
pub fn replace_all(text: &mut String, q: &str, with: &str, case: bool) -> usize {
    let m = find_all(text, q, case);
    for r in m.iter().rev() { replace_range(text, *r, with); }
    m.len()
}

/// Zeichenposition des Anfangs von Zeile `line` (1-basiert, wird auf die vorhandenen Zeilen begrenzt) und die tatsächliche Zeile
pub fn line_start(text: &str, line: usize) -> (usize, usize) {
    let total = text.split('\n').count();
    let line = line.clamp(1, total);
    let mut pos = 0;
    for l in text.split('\n').take(line - 1) { pos += l.chars().count() + 1; }
    (pos, line)
}

/// Zeile (1-basiert) eines Zeichenindex
pub fn line_of(text: &str, idx: usize) -> usize { text.chars().take(idx).filter(|c| *c == '\n').count() + 1 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treffer_mit_und_ohne_grossschreibung() {
        assert_eq!(find_all("lda #1\nLDA #2", "lda", false), vec![(0, 3), (7, 10)]);
        assert_eq!(find_all("lda #1\nLDA #2", "lda", true), vec![(0, 3)]);
        assert!(find_all("abc", "", false).is_empty());
        assert_eq!(find_all("aaaa", "aa", true), vec![(0, 2), (2, 4)], "keine Überlappung");
    }

    #[test]
    fn positionen_zaehlen_zeichen_nicht_bytes() {
        let t = "äö lda\nüx lda";
        assert_eq!(find_all(t, "lda", false), vec![(3, 6), (10, 13)]);
        let mut s = t.to_string();
        replace_range(&mut s, (3, 6), "sta");
        assert_eq!(s, "äö sta\nüx lda");
    }

    #[test]
    fn weiter_und_zurueck_laufen_im_kreis() {
        let m = vec![(2, 4), (10, 12), (20, 22)];
        assert_eq!(next_match(&m, 0, true), Some((2, 4)));
        assert_eq!(next_match(&m, 5, true), Some((10, 12)));
        assert_eq!(next_match(&m, 23, true), Some((2, 4)), "am Ende von vorn");
        assert_eq!(next_match(&m, 12, false), Some((10, 12)));
        assert_eq!(next_match(&m, 1, false), Some((20, 22)), "am Anfang von hinten");
        assert_eq!(next_match(&[], 0, true), None);
    }

    #[test]
    fn alles_ersetzen_zaehlt() {
        let mut s = "lda #1\nLda #2\nsta $d020".to_string();
        assert_eq!(replace_all(&mut s, "lda", "ldx", false), 2);
        assert_eq!(s, "ldx #1\nldx #2\nsta $d020");
        assert_eq!(replace_all(&mut s, "nichts", "x", false), 0);
    }

    #[test]
    fn gehe_zu_zeile() {
        let t = "ab\ncde\nf";
        assert_eq!(line_start(t, 1), (0, 1));
        assert_eq!(line_start(t, 2), (3, 2));
        assert_eq!(line_start(t, 3), (7, 3));
        assert_eq!(line_start(t, 99), (7, 3), "zu große Zeile: letzte");
        assert_eq!(line_start(t, 0), (0, 1));
        assert_eq!(line_of(t, 4), 2);
    }
}
