// SPDX-License-Identifier: GPL-3.0-or-later
//! Pseudo-Sprache `xx`: verfremdet den deutschen Text (Buchstaben → ähnliche Zeichen mit Akzent), hängt etwa 40 % Füllung an und klammert ihn mit `[…]`.
//! Eingesetzte Parameterwerte (Dateinamen, Zahlen …) bleiben unverändert (sie werden vorher mit Schutzzeichen umgeben).
//! So fällt sofort auf, welche Texte nicht aus dem Katalog kommen (sie erscheinen normal) und wo Felder zu kurz wären.

const OPEN: char = '\u{E000}';
const CLOSE: char = '\u{E001}';

/// Parameterwert vor der Verfremdung schützen
pub(crate) fn guard(v: &str) -> String { format!("{OPEN}{v}{CLOSE}") }

fn map(c: char) -> char {
    match c {
        'a' => 'á', 'b' => 'ƀ', 'c' => 'ç', 'd' => 'ð', 'e' => 'é', 'f' => 'ƒ', 'g' => 'ĝ', 'h' => 'ĥ', 'i' => 'í', 'j' => 'ĵ', 'k' => 'ķ', 'l' => 'ļ', 'm' => 'ɱ',
        'n' => 'ñ', 'o' => 'ó', 'p' => 'þ', 'q' => 'ǫ', 'r' => 'ŕ', 's' => 'š', 't' => 'ţ', 'u' => 'ú', 'v' => 'ṽ', 'w' => 'ŵ', 'x' => 'ẋ', 'y' => 'ý', 'z' => 'ž',
        'A' => 'Á', 'B' => 'Ɓ', 'C' => 'Ç', 'D' => 'Ð', 'E' => 'É', 'F' => 'Ƒ', 'G' => 'Ĝ', 'H' => 'Ĥ', 'I' => 'Í', 'J' => 'Ĵ', 'K' => 'Ķ', 'L' => 'Ļ', 'M' => 'Ṁ',
        'N' => 'Ñ', 'O' => 'Ó', 'P' => 'Þ', 'Q' => 'Ǫ', 'R' => 'Ŕ', 'S' => 'Š', 'T' => 'Ţ', 'U' => 'Ú', 'V' => 'Ṽ', 'W' => 'Ŵ', 'X' => 'Ẋ', 'Y' => 'Ý', 'Z' => 'Ž',
        'ä' => 'æ', 'ö' => 'œ', 'ü' => 'ů', 'ß' => 'ƞ', 'Ä' => 'Æ', 'Ö' => 'Œ', 'Ü' => 'Ů',
        other => other,
    }
}

/// Verfremdet `s`; Abschnitte zwischen den Schutzzeichen bleiben wie sie sind
pub(crate) fn transform(s: &str) -> String {
    let mut out = String::from("[");
    let (mut guarded, mut letters) = (false, 0usize);
    for c in s.chars() {
        match c {
            OPEN => guarded = true,
            CLOSE => guarded = false,
            _ if guarded => out.push(c),
            _ => { if c.is_alphabetic() { letters += 1 } out.push(map(c)) }
        }
    }
    out.extend(std::iter::repeat_n('~', (letters * 2).div_ceil(5)));
    out.push(']');
    out
}
