// SPDX-License-Identifier: GPL-3.0-or-later
//! Zeichentext der Zeichentabelle (PETSCII/ATASCII): Namen, Literale, Zusammensetzen von Strings, Zeichensatzliste (ohne GUI).
use std::path::Path;
use vc_asm::tokens::{self, Names};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode { InString, Value, ByteDir, Decimal, ScreenCode }

pub struct SetEntry { pub label: String, pub file: String }

pub fn abbr(atari: bool, code: u8) -> Option<String> {
    let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
    let (n, _, _) = names.iter().find(|(_, c, _)| *c == code)?;
    if matches!(*n, "space" | "shift space") { return None; }
    Some(match *n {
        "rvs on" => "RVS".into(), "rvs off" => "OFF".into(), "shift ret" => "S-RET".into(), "orange" => "ORG".into(), "lower" => "a/A".into(), "upper" => "A/a".into(),
        "del line" => "DEL".into(), "ins line" => "INS".into(), "clr tab" => "CT".into(), "set tab" => "ST".into(), "del char" => "DCH".into(), "ins char" => "ICH".into(),
        "up" => "UP".into(), "down" => "DN".into(), "left" => "LT".into(), "right" => "RT".into(),
        other => other.to_uppercase(),
    })
}

/// Glyph-Nummer im Zeichensatz-Bild (und ob invers) für einen Code der Tabelle; `None` = Steuerzeichen/unbelegt
pub fn glyph_of(atari: bool, tab: usize, code: u8) -> Option<(usize, bool)> {
    if atari {
        if tab == 1 { return Some(((code & 0x7F) as usize, code >= 0x80)); }
        if abbr(true, code).is_some() { return None; }
        let b = code & 0x7F;
        let i = if b < 0x20 { b + 0x40 } else if b < 0x60 { b - 0x20 } else { b };
        return Some((i as usize, code >= 0x80));
    }
    if tab == 1 { return Some((code as usize, false)); }
    let i = match code {
        0x00..=0x1F | 0x80..=0x9F => return None,
        0x20..=0x3F => code,
        0x40..=0x5F => code - 0x40,
        0x60..=0x7F => code - 0x20,
        0xA0..=0xBF => code - 0x40,
        0xC0..=0xFE => code - 0x80,
        0xFF => 0x5E,
    };
    Some((i as usize, false))
}

/// Bildschirmcode-Wert zu einem Code der ersten Tabelle (für „Einfügen als Bildschirmcode“)
pub fn screen_value(atari: bool, tab: usize, code: u8) -> Option<u8> {
    glyph_of(atari, tab, code).map(|(i, inv)| if atari { i as u8 | if inv { 0x80 } else { 0 } } else { i as u8 })
}

/// Zeichen, das man im Quelltext einer Zeichenkette direkt schreiben kann (wird vom Assembler wieder in diesen Code umgewandelt)
pub fn literal_of(atari: bool, tab: usize, code: u8) -> Option<char> {
    if tab == 1 { return None; }
    let c = if atari {
        match code { 0x20..=0x7A => code as char, _ => return None }
    } else {
        match code { 0x20..=0x3F => code as char, 0x41..=0x5A => (code + 0x20) as char, 0xC1..=0xDA => (code - 0x80) as char, _ => return None }
    };
    if c == '"' || c == '{' || c == '}' { None } else { Some(c) }
}

fn token_of(atari: bool, tab: usize, code: u8, basic: bool) -> String {
    if basic {
        // BASIC: Hauptname des BASIC aus der gemeinsamen Namensquelle (sonst `{$xx}`)
        let f = if atari { vc_core::chars::Family::Atascii } else { vc_core::chars::Family::Petscii };
        if tab == 0 { if let Some(c) = vc_core::chars::by_code(f, code) { return format!("{{{}}}", c.main(vc_core::chars::Lang::Basic)); } }
        return format!("{{${code:02x}}}");
    }
    if tab == 0 {
        let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
        if let Some((n, _, _)) = names.iter().find(|(_, c, _)| *c == code) { return format!("{{{n}}}"); }
    }
    format!("{{${code:02x}}}")
}

/// Text für einen Satz Codes, je nach Einfügeart
#[cfg(test)]
pub fn compose(atari: bool, tab: usize, mode: Mode, codes: &[u8]) -> String { compose_ex(atari, tab, mode, codes, false) }

/// Wie [`compose`]; mit `basic` die Hauptnamen des BASIC (`orng`) statt der des Assemblers (`orange`)
pub fn compose_ex(atari: bool, tab: usize, mode: Mode, codes: &[u8], basic: bool) -> String {
    match mode {
        Mode::InString => {
            let mut out = String::new();
            let mut i = 0;
            while i < codes.len() {
                let c = codes[i];
                let mut n = 1; while i + n < codes.len() && codes[i + n] == c { n += 1; }
                match literal_of(atari, tab, c) {
                    Some(ch) => out.extend(std::iter::repeat(ch).take(n)),
                    None => { let t = token_of(atari, tab, c, basic); if n > 1 { out.push_str(&format!("{}*{n}}}", &t[..t.len() - 1])); } else { out.push_str(&t); } }
                }
                i += n;
            }
            out
        }
        Mode::Value => codes.iter().map(|c| format!("${c:02x}")).collect::<Vec<_>>().join(", "),
        Mode::ByteDir => format!(".byte {}", codes.iter().map(|c| format!("${c:02x}")).collect::<Vec<_>>().join(", ")),
        Mode::Decimal => codes.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "),
        Mode::ScreenCode => codes.iter().filter_map(|c| screen_value(atari, tab, *c)).map(|v| format!("${v:02x}")).collect::<Vec<_>>().join(", "),
    }
}

pub fn describe(atari: bool, tab: usize, code: u8) -> (String, String) {
    // (Name, Tastatur)
    if tab == 0 {
        let names = tokens::names_of(if atari { Names::Atascii } else { Names::Petscii });
        if let Some((n, _, d)) = names.iter().find(|(_, c, _)| *c == code) { return (format!("{d}  ·  {{{n}}}"), String::new()); }
    }
    let name = match literal_of(atari, tab, code) { Some(c) => format!("im Quelltext tippbar als »{c}«"), None => if glyph_of(atari, tab, code).is_some() { "Grafikzeichen".into() } else { "unbelegt".into() } };
    let key = if tab == 1 { String::new() } else if atari {
        match code { 0x01..=0x1A => format!("Strg+{}", (b'A' + code - 1) as char), 0x00 => "Strg+,".into(), _ => String::new() }
    } else {
        match code { 0xC1..=0xDA => format!("Umschalt+{}", (code - 0x80) as char), 0x41..=0x5A => format!("{}", (code) as char), _ => String::new() }
    };
    (name, key)
}

/// Zeichensätze (aus den ROMs des Anwenders, Verweise `rom:system/rolle#satz`) für das System; fehlt jedes ROM, ist die Liste leer
pub fn charset_list(sys: &str, data: &Path) -> Vec<SetEntry> {
    let want = if sys == "mega65" { "c64" } else { sys };
    let mut sets: Vec<SetEntry> = vec![];
    for c in vc_media::project::std_charset_list(data) {
        let hires8 = c.profile.ends_with("char.hires") || c.profile.ends_with("char.antic2");
        if c.system == want && hires8 && c.status != vc_media::Status::Missing && !sets.iter().any(|x| x.file == c.reference) {
            sets.push(SetEntry { label: c.label, file: c.reference });
        }
    }
    sets
}

/// Vorgabe: Groß/Grafik (C64) bzw. der Standardsatz (Atari)
pub fn default_set(sets: &[SetEntry]) -> usize { sets.iter().position(|s| s.file.ends_with("#upper") || s.file.ends_with("#standard")).unwrap_or(0) }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_in_strings_uses_names_literals_and_repeats() {
        // PETSCII: CLR, Buchstaben (direkt: a = $41, A = $C1), ein Grafikzeichen als Hexwert, drei mal Cursor runter
        assert_eq!(compose(false, 0, Mode::InString, &[0x93, 0x41, 0xC1, 0xA1, 0x11, 0x11, 0x11]), "{clr}aA{$a1}{down*3}");
        assert_eq!(compose(false, 0, Mode::InString, &[0xC1 + 1, 0x31, 0x22]), "B1{$22}");
        assert_eq!(compose(true, 0, Mode::InString, &[0x7D, 0x41, 0x14, 0x9B]), "{clr}A{$14}{eol}");
        // Bildschirmcode-Reiter: nur Zahlen
        assert_eq!(compose(false, 1, Mode::InString, &[0x01, 0x41]), "{$01}{$41}");
    }

    #[test]
    fn compose_for_basic_uses_the_dialect_names() {
        assert_eq!(compose_ex(false, 0, Mode::InString, &[0x93, 0x41, 0x11, 0x11], true), "{clr}a{down*2}");
        // Hauptnamen des BASIC statt der des Assemblers (`orng` statt `orange`, `return` statt `ret`)
        assert_eq!(compose_ex(false, 0, Mode::InString, &[0x81, 0x0D], true), "{orng}{return}");
        assert_eq!(compose_ex(false, 0, Mode::InString, &[0x81, 0x0D], false), "{orange}{ret}");
        assert_eq!(compose_ex(false, 0, Mode::Decimal, &[0x93, 0x05], true), "147, 5");
    }

    #[test]
    fn compose_other_modes() {
        assert_eq!(compose(false, 0, Mode::Value, &[0xC1, 0x93]), "$c1, $93");
        assert_eq!(compose(false, 0, Mode::ByteDir, &[0xC1]), ".byte $c1");
        assert_eq!(compose(false, 0, Mode::Decimal, &[0xC1, 0x93]), "193, 147");
        assert_eq!(compose(false, 0, Mode::ScreenCode, &[0xC1, 0x93]), "$41");     // Steuerzeichen haben keinen Bildschirmcode
        assert_eq!(compose(true, 0, Mode::ScreenCode, &[0x14, 0x94]), "$54, $d4");
    }

    #[test]
    fn petscii_screen_mapping_matches_the_assembler() {
        // jedes PETSCII-Zeichen, das der Assembler aus einem ASCII-Buchstaben erzeugt, zeigt das Bildschirmcode-Zeichen von `.screen`
        assert_eq!(glyph_of(false, 0, 0x41).unwrap().0, 0x01);
        assert_eq!(glyph_of(false, 0, 0xC1).unwrap().0, 0x41);
        assert_eq!(glyph_of(false, 0, 0x93), None);
        assert_eq!(glyph_of(true, 0, 0x14).unwrap(), (0x54, false));
        assert_eq!(glyph_of(true, 0, 0x94).unwrap(), (0x54, true));
        assert_eq!(glyph_of(true, 0, 0x7D), None);
    }

    #[test]
    fn every_composed_string_is_understood_by_the_assembler() {
        use vc_asm::tokens::{expand, Names};
        for code in 0..=255u8 {
            for (atari, names) in [(false, Names::Petscii), (true, Names::Atascii)] {
                let s = compose(atari, 0, Mode::InString, &[code, code, code]);
                let p = expand(&s, names).unwrap_or_else(|e| panic!("{s}: {e}"));
                assert_eq!(p.len(), 3, "{s}");
            }
        }
    }
}

