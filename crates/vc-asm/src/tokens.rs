// SPDX-License-Identifier: GPL-3.0-or-later
//! Sonderzeichen in Zeichenketten: `{down}`, `{clr}`, `{rvs on}`, `{$c1}`, `{147}`, `{down*3}`.
//! Gilt für `.petscii`, `.atascii`, `.screen` und für `.text`/`.null`/`.ptext`/`.shift` bei gesetzter Kodierung (nicht `ascii`).
//! Der Wert in den Klammern ist das fertige Byte (nicht noch einmal umgewandelt); `{{` ergibt ein einzelnes `{`.

/// Welche Schreibweisen erlaubt sind
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Names {
    /// Klammern sind gewöhnliche Zeichen
    Off,
    /// nur Zahlen: `{$41}`, `{65}`
    Numeric,
    /// zusätzlich die PETSCII-Steuerzeichen
    Petscii,
    /// zusätzlich die ATASCII-Steuerzeichen
    Atascii,
}

/// (Name, Code, Beschreibung); Namen klein, mit Leerzeichen statt `_`/`-`
pub const PETSCII_NAMES: &[(&str, u8, &str)] = &[
    ("wht", 0x05, "Weiß"), ("ret", 0x0D, "Wagenrücklauf (RETURN)"), ("lower", 0x0E, "Klein-/Großschrift einschalten"), ("down", 0x11, "Cursor runter"),
    ("rvs on", 0x12, "Invers an"), ("home", 0x13, "Cursor links oben"), ("del", 0x14, "Zeichen links löschen"), ("red", 0x1C, "Rot"),
    ("right", 0x1D, "Cursor rechts"), ("grn", 0x1E, "Grün"), ("blu", 0x1F, "Blau"),
    ("orange", 0x81, "Orange"), ("f1", 0x85, "F1"), ("f3", 0x86, "F3"), ("f5", 0x87, "F5"), ("f7", 0x88, "F7"), ("f2", 0x89, "F2"), ("f4", 0x8A, "F4"),
    ("f6", 0x8B, "F6"), ("f8", 0x8C, "F8"), ("shift ret", 0x8D, "Zeilenvorschub (Umschalt+RETURN)"), ("upper", 0x8E, "Groß-/Grafikschrift einschalten"),
    ("blk", 0x90, "Schwarz"), ("up", 0x91, "Cursor hoch"), ("rvs off", 0x92, "Invers aus"), ("clr", 0x93, "Bildschirm löschen"), ("inst", 0x94, "Zeichen einfügen"),
    ("brn", 0x95, "Braun"), ("lred", 0x96, "Hellrot"), ("gry1", 0x97, "Dunkelgrau"), ("gry2", 0x98, "Mittelgrau"), ("lgrn", 0x99, "Hellgrün"),
    ("lblu", 0x9A, "Hellblau"), ("gry3", 0x9B, "Hellgrau"), ("pur", 0x9C, "Violett"), ("left", 0x9D, "Cursor links"), ("yel", 0x9E, "Gelb"), ("cyn", 0x9F, "Türkis"),
    ("space", 0x20, "Leerzeichen"), ("shift space", 0xA0, "Umschalt+Leerzeichen"),
];

pub const ATASCII_NAMES: &[(&str, u8, &str)] = &[
    ("esc", 0x1B, "Escape (nächstes Zeichen wird nur dargestellt)"), ("up", 0x1C, "Cursor hoch"), ("down", 0x1D, "Cursor runter"), ("left", 0x1E, "Cursor links"),
    ("right", 0x1F, "Cursor rechts"), ("clr", 0x7D, "Bildschirm löschen"), ("bs", 0x7E, "Rückschritt"), ("tab", 0x7F, "Tabulator"), ("eol", 0x9B, "Zeilenende"),
    ("del line", 0x9C, "Zeile löschen"), ("ins line", 0x9D, "Zeile einfügen"), ("clr tab", 0x9E, "Tabulator löschen"), ("set tab", 0x9F, "Tabulator setzen"),
    ("bell", 0xFD, "Klingel"), ("del char", 0xFE, "Zeichen löschen"), ("ins char", 0xFF, "Zeichen einfügen"),
];

pub fn names_of(n: Names) -> &'static [(&'static str, u8, &'static str)] {
    match n { Names::Petscii => PETSCII_NAMES, Names::Atascii => ATASCII_NAMES, _ => &[] }
}

/// Name vereinheitlichen: klein, `_` und `-` wie Leerzeichen, mehrfache Leerzeichen zusammenfassen
pub fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['_', '-'], " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn lookup(n: Names, name: &str) -> Option<u8> {
    let name = normalize(name);
    names_of(n).iter().find(|(k, _, _)| *k == name).map(|(_, c, _)| *c)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    /// gewöhnliches Zeichen (wird nach der Kodierung umgewandelt)
    Char(u8),
    /// fertiges Byte
    Raw(u8),
}

/// Zerlegt eine Zeichenkette; Fehlertext, wenn eine Klammer nicht verstanden wird
pub fn expand(s: &str, names: Names) -> Result<Vec<Piece>, String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if names == Names::Off || b[i] != b'{' { out.push(Piece::Char(b[i])); i += 1; continue; }
        if b.get(i + 1) == Some(&b'{') { out.push(Piece::Char(b'{')); i += 2; continue; }
        let Some(end) = b[i + 1..].iter().position(|c| *c == b'}') else { return Err("'{' ohne schließendes '}' (für ein einzelnes '{' schreibe '{{')".into()) };
        let inner = &s[i + 1..i + 1 + end];
        let (base, count) = match inner.split_once('*') {
            Some((a, n)) => (a, n.trim().parse::<usize>().ok().filter(|n| (1..=255).contains(n)).ok_or(format!("{{{inner}}}: Wiederholung 1 bis 255 erwartet"))?),
            None => (inner, 1),
        };
        let base = base.trim();
        let code = if let Some(h) = base.strip_prefix('$') {
            u8::from_str_radix(h, 16).map_err(|_| format!("{{{inner}}}: ungültiger Hexwert (00 bis ff)"))?
        } else if !base.is_empty() && base.bytes().all(|c| c.is_ascii_digit()) {
            base.parse::<u8>().map_err(|_| format!("{{{inner}}}: Zahl 0 bis 255 erwartet"))?
        } else if names == Names::Numeric {
            return Err(format!("{{{inner}}}: Namen gibt es nur mit .petscii/.atascii oder .encoding \"petscii\"/\"atascii\" (hier nur {{$hh}} und {{nnn}})"));
        } else {
            lookup(names, base).ok_or_else(|| format!("unbekanntes Sonderzeichen {{{inner}}}"))?
        };
        out.extend(std::iter::repeat(Piece::Raw(code)).take(count));
        i += end + 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_numbers_repeats_and_escapes() {
        use Piece::*;
        assert_eq!(expand("A{down}B", Names::Petscii).unwrap(), vec![Char(b'A'), Raw(0x11), Char(b'B')]);
        assert_eq!(expand("{CLR}{rvs_on}{Rvs-Off}", Names::Petscii).unwrap(), vec![Raw(0x93), Raw(0x12), Raw(0x92)]);
        assert_eq!(expand("{$c1}{147}{down*3}", Names::Petscii).unwrap(), vec![Raw(0xC1), Raw(147), Raw(0x11), Raw(0x11), Raw(0x11)]);
        assert_eq!(expand("a{{b", Names::Petscii).unwrap(), vec![Char(b'a'), Char(b'{'), Char(b'b')]);
        assert_eq!(expand("{eol}", Names::Atascii).unwrap(), vec![Raw(0x9B)]);
        assert_eq!(expand("{$41}", Names::Numeric).unwrap(), vec![Raw(0x41)]);
        assert_eq!(expand("x{y}", Names::Off).unwrap().len(), 4);
        assert!(expand("{nix}", Names::Petscii).is_err());
        assert!(expand("{down", Names::Petscii).is_err());
        assert!(expand("{down}", Names::Numeric).is_err());
        assert!(expand("{300}", Names::Petscii).is_err());
        assert!(expand("{down*0}", Names::Petscii).is_err());
        assert!(expand("{clr}", Names::Atascii).is_ok() && expand("{wht}", Names::Atascii).is_err());
    }
}
