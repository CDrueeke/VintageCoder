// SPDX-License-Identifier: GPL-3.0-or-later
//! ASCII ↔ PETSCII (C64, Großschrift-/Grafikmodus) und `{...}`-Steuercodes.
use crate::dialect::Dialect;

/// ASCII-Zeichen → PETSCII-Byte (Kleinbuchstaben werden zu PETSCII-Großbuchstaben, Großbuchstaben zu den umgeschalteten Zeichen).
pub fn from_ascii(c: char) -> Option<u8> {
    Some(match c {
        'a'..='z' => c as u8 - 0x20,
        'A'..='Z' => c as u8 + 0x80,
        ' '..='@' => c as u8,
        '[' => 0x5B, ']' => 0x5D, '^' => 0x5E, '_' => 0x5F,
        _ => return None,
    })
}

/// PETSCII-Byte → Text; Steuer- und Sonderzeichen als `{name}` bzw. `{$xx}`.
pub fn to_text(b: u8, d: &Dialect) -> String {
    match b {
        0x41..=0x5A => ((b + 0x20) as char).to_string(),
        0xC1..=0xDA => ((b - 0x80) as char).to_string(),
        0x20..=0x40 | 0x5B | 0x5D | 0x5E | 0x5F => (b as char).to_string(),
        _ => match d.control_name(b) { Some(n) => format!("{{{n}}}"), None => format!("{{${b:02x}}}") },
    }
}

/// Wandelt den Inhalt einer Zeichenkette (mit `{name}`, `{name*n}`, `{$xx}`) in PETSCII-Bytes.
pub fn from_text(s: &str, d: &Dialect) -> Result<Vec<u8>, String> {
    let c: Vec<char> = s.chars().collect();
    let (mut out, mut i) = (vec![], 0);
    while i < c.len() {
        if c[i] == '{' {
            let end = c[i..].iter().position(|x| *x == '}').ok_or("'}' fehlt")? + i;
            let body: String = c[i + 1..end].iter().collect();
            // Name des Dialekts zuerst, dann die gemeinsame Tabelle (`data/chars/`: alle Schreibweisen, `{147}`, `{$93}`, `{down*3}`)
            let (code, count) = vc_core::chars::parse_ref(vc_core::chars::Family::Petscii, &body, &|n| d.control_code(n))?;
            out.extend(std::iter::repeat(code).take(count));
            i = end + 1;
        } else {
            out.push(from_ascii(c[i]).ok_or(format!("Zeichen '{}' hat keine PETSCII-Entsprechung (nutze {{$xx}})", c[i]))?);
            i += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialect() -> Dialect { Dialect::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"), "cbm_v2").unwrap() }

    #[test]
    fn names_follow_the_shared_table_in_every_spelling() {
        let d = dialect();
        // Namen des Dialekts wie bisher
        assert_eq!(from_text("{clr}{orng}{rvon}{down*3}", &d).unwrap(), vec![0x93, 0x81, 0x12, 0x11, 0x11, 0x11]);
        // Namen des Assemblers gelten ebenfalls, mit `_`, `-` und mehreren Leerzeichen
        assert_eq!(from_text("{orange}{rvs_on}{Rvs-Off}{shift  ret}{ret}", &d).unwrap(), vec![0x81, 0x12, 0x92, 0x8D, 0x0D]);
        // Zahlen: dezimal und hexadezimal
        assert_eq!(from_text("{147}{$93}{$c1*2}", &d).unwrap(), vec![147, 0x93, 0xC1, 0xC1]);
        assert!(from_text("{300}", &d).is_err());
        assert!(from_text("{nix}", &d).unwrap_err().contains("unbekannter Steuercode"));
        assert!(from_text("{down*0}", &d).is_err());
    }

    #[test]
    fn listing_keeps_the_dialect_names() {
        let d = dialect();
        assert_eq!(to_text(0x81, &d), "{orng}");
        assert_eq!(to_text(0x1D, &d), "{rght}");
        assert_eq!(to_text(0x0D, &d), "{return}");
    }
}
