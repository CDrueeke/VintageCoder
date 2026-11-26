// SPDX-License-Identifier: GPL-3.0-or-later
//! Sonderzeichen in Zeichenketten: `{down}`, `{clr}`, `{rvs on}`, `{$c1}`, `{147}`, `{down*3}`.
//! Gilt für `.petscii`, `.atascii`, `.screen` und für `.text`/`.null`/`.ptext`/`.shift` bei gesetzter Kodierung (nicht `ascii`).
//! Der Wert in den Klammern ist das fertige Byte (nicht noch einmal umgewandelt); `{{` ergibt ein einzelnes `{`.
use vc_core::{msg, Msg};

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

/// (Hauptname im Assembler, Code, Beschreibung) aus der gemeinsamen Namensquelle `data/chars/` (`vc_core::chars`)
pub fn names_of(n: Names) -> &'static [(&'static str, u8, &'static str)] {
    use std::sync::OnceLock;
    type Row = (&'static str, u8, &'static str);
    static PET: OnceLock<Vec<Row>> = OnceLock::new();
    static ATA: OnceLock<Vec<Row>> = OnceLock::new();
    let rows = |f| vc_core::chars::table(f).iter().map(|c| (c.name.as_str(), c.code, c.desc.as_str())).collect::<Vec<Row>>();
    match n {
        Names::Petscii => PET.get_or_init(|| rows(vc_core::chars::Family::Petscii)),
        Names::Atascii => ATA.get_or_init(|| rows(vc_core::chars::Family::Atascii)),
        _ => &[],
    }
}

/// Name vereinheitlichen: klein, `_` und `-` wie Leerzeichen, mehrfache Leerzeichen zusammenfassen
pub fn normalize(name: &str) -> String { vc_core::chars::normalize(name) }

/// Code zu einem Namen; jeder Name der Tabelle gilt (auch die BASIC-Schreibweisen, `orng` wie `orange`)
pub fn lookup(n: Names, name: &str) -> Option<u8> {
    match n {
        Names::Petscii => vc_core::chars::lookup(vc_core::chars::Family::Petscii, name),
        Names::Atascii => vc_core::chars::lookup(vc_core::chars::Family::Atascii, name),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    /// gewöhnliches Zeichen (wird nach der Kodierung umgewandelt)
    Char(u8),
    /// fertiges Byte
    Raw(u8),
}

/// Zerlegt eine Zeichenkette; Fehlertext, wenn eine Klammer nicht verstanden wird
pub fn expand(s: &str, names: Names) -> Result<Vec<Piece>, Msg> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if names == Names::Off || b[i] != b'{' { out.push(Piece::Char(b[i])); i += 1; continue; }
        if b.get(i + 1) == Some(&b'{') { out.push(Piece::Char(b'{')); i += 2; continue; }
        let Some(end) = b[i + 1..].iter().position(|c| *c == b'}') else { return Err(msg!("diag-asm-e208-token-unclosed")) };
        let inner = &s[i + 1..i + 1 + end];
        let (base, count) = match inner.split_once('*') {
            Some((a, n)) => (a, n.trim().parse::<usize>().ok().filter(|n| (1..=255).contains(n)).ok_or_else(|| msg!("diag-asm-e208-token-repeat", token = format!("{{{inner}}}")))?),
            None => (inner, 1),
        };
        let base = base.trim();
        let code = if let Some(h) = base.strip_prefix('$') {
            u8::from_str_radix(h, 16).map_err(|_| msg!("diag-asm-e208-token-hex", token = format!("{{{inner}}}")))?
        } else if !base.is_empty() && base.bytes().all(|c| c.is_ascii_digit()) {
            base.parse::<u8>().map_err(|_| msg!("diag-asm-e208-token-number", token = format!("{{{inner}}}")))?
        } else if names == Names::Numeric {
            return Err(msg!("diag-asm-e208-token-names", token = format!("{{{inner}}}")));
        } else {
            lookup(names, base).ok_or_else(|| msg!("diag-asm-e208-token-unknown", token = format!("{{{inner}}}")))?
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
