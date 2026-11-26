// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenwerkzeuge (Kern): eine Zahl oder ein Ausdruck (`$D020`, `0xFF`, `%1010`, `0b1010`, `255`, `$D000+$20`, `1<<5`, `-1`) wird gelesen und in allen Schreibweisen
//! ausgegeben (dezimal, hexadezimal mit `$`, binär mit `%`, mit Vorzeichen, Lo-/Hi-Byte, Zeichen). Nur ganze Zahlen (64 Bit), keine Fließkommazahlen.
//! Die Oberfläche (Dialog „Zahlenwerkzeuge …“) fragt über `zahlen_werte` und setzt die Schreibweisen in den Editor ein; Fehlertexte kommen aus dem Katalog.
//! Die Auswertung ist dieselbe wie im Dialog der alten Oberfläche (`crates/vc-gui/src/zahlen.rs`, eingefroren).
use serde_json::{json, Value};

struct P<'a> { s: &'a [u8], i: usize }
impl<'a> P<'a> {
    fn blank(&mut self) { while self.s.get(self.i).is_some_and(|c| c.is_ascii_whitespace()) { self.i += 1 } }
    fn peek(&mut self) -> Option<u8> { self.blank(); self.s.get(self.i).copied() }
    fn zahl(&mut self) -> Result<i64, String> {
        self.blank(); let start = self.i;
        let (basis, ziffern): (u32, &dyn Fn(u8) -> bool) = if self.s.get(self.i) == Some(&b'$') { self.i += 1; (16, &|c: u8| c.is_ascii_hexdigit() || c == b'_') }
            else if self.s[self.i..].starts_with(b"0x") || self.s[self.i..].starts_with(b"0X") { self.i += 2; (16, &|c: u8| c.is_ascii_hexdigit() || c == b'_') }
            else if self.s.get(self.i) == Some(&b'%') { self.i += 1; (2, &|c: u8| c == b'0' || c == b'1' || c == b'_') }
            else if self.s[self.i..].starts_with(b"0b") || self.s[self.i..].starts_with(b"0B") { self.i += 2; (2, &|c: u8| c == b'0' || c == b'1' || c == b'_') }
            else { (10, &|c: u8| c.is_ascii_digit() || c == b'_') };
        let von = self.i; while self.s.get(self.i).is_some_and(|c| ziffern(*c)) { self.i += 1 }
        let t: String = self.s[von..self.i].iter().filter(|c| **c != b'_').map(|c| *c as char).collect();
        if t.is_empty() { return Err(vc_i18n::t!("zahlen-zahl-erwartet-bei-zeichen-p1", p1 = start + 1)) }
        i64::from_str_radix(&t, basis).map_err(|_| vc_i18n::t!("zahlen-zahl-zu-gross").to_string())
    }
    fn atom(&mut self) -> Result<i64, String> {
        match self.peek() {
            Some(b'(') => { self.i += 1; let v = self.ausdruck(0)?; if self.peek() == Some(b')') { self.i += 1; Ok(v) } else { Err(vc_i18n::t!("zahlen-fehlt").into()) } }
            Some(b'-') => { self.i += 1; Ok(self.atom()?.wrapping_neg()) } Some(b'+') => { self.i += 1; self.atom() } Some(b'~') => { self.i += 1; Ok(!self.atom()?) }
            Some(_) => self.zahl(), None => Err(vc_i18n::t!("zahlen-zahl-erwartet").into()),
        }
    }
    /// Operatoren: `|` 1, `^` 2, `&` 3, `<<` `>>` 4, `+` `-` 5, `*` `/` `%` 6 (kleiner Rang bindet schwächer); `%` ist hier der Rest nur nach einem Wert mit Leerzeichen, sonst Binärpräfix
    fn ausdruck(&mut self, min: u8) -> Result<i64, String> {
        let mut l = self.atom()?;
        loop {
            let Some(c) = self.peek() else { break };
            let (op, rang, laenge): (&str, u8, usize) = match c { b'|' => ("|", 1, 1), b'^' => ("^", 2, 1), b'&' => ("&", 3, 1), b'+' => ("+", 5, 1), b'-' => ("-", 5, 1), b'*' => ("*", 6, 1), b'/' => ("/", 6, 1),
                b'<' if self.s.get(self.i + 1) == Some(&b'<') => ("<<", 4, 2), b'>' if self.s.get(self.i + 1) == Some(&b'>') => (">>", 4, 2), _ => break };
            if rang < min { break }
            self.i += laenge; let r = self.ausdruck(rang + 1)?;
            l = match op { "|" => l | r, "^" => l ^ r, "&" => l & r, "+" => l.wrapping_add(r), "-" => l.wrapping_sub(r), "*" => l.wrapping_mul(r),
                "/" => { if r == 0 { return Err(vc_i18n::t!("zahlen-teilung-durch-0").into()) } l / r } "<<" => l.checked_shl(r as u32).unwrap_or(0), ">>" => l.checked_shr(r as u32).unwrap_or(0), _ => l };
        }
        Ok(l)
    }
}
/// Wert eines Ausdrucks (ganze Zahlen)
pub fn werte(text: &str) -> Result<i64, String> {
    let mut p = P { s: text.trim().as_bytes(), i: 0 }; if p.s.is_empty() { return Err(vc_i18n::t!("zahlen-zahl-oder-ausdruck-eingeben").into()) }
    let v = p.ausdruck(0)?; if p.peek().is_some() { return Err(vc_i18n::t!("zahlen-unerwartetes-zeichen-bei-p1", p1 = p.i + 1)) } Ok(v)
}

// ---------------------------------------------------------------------------------------------------- Schreibweisen
pub fn hex(v: i64) -> String { if (0..=0xFF).contains(&v) { format!("${v:02X}") } else if (0..=0xFFFF).contains(&v) { format!("${v:04X}") } else if v >= 0 { format!("${v:X}") } else { format!("-${:X}", v.unsigned_abs()) } }
pub fn binaer(v: i64) -> String { let b = if (0..=0xFF).contains(&v) { 8 } else if (0..=0xFFFF).contains(&v) { 16 } else { 0 }; if v < 0 { format!("-%{:b}", v.unsigned_abs()) } else if b > 0 { format!("%{v:0b$b}") } else { format!("%{v:b}") } }
/// Wert als Zweierkomplement in `bits` Bit (8 oder 16)
pub fn vorzeichen(v: i64, bits: u8) -> i64 { let m = (1i64 << bits) - 1; let u = v & m; if u >> (bits - 1) == 1 { u - (1 << bits) } else { u } }
pub fn zeichen(v: i64) -> Option<char> { (0x20..0x7F).contains(&v).then(|| v as u8 as char) }
/// Bit `n` umschalten (nur innerhalb der gewählten Breite)
pub fn bit_umschalten(v: i64, n: u8, breite: u8) -> i64 { if n >= breite { return v } let m = (1i64 << breite) - 1; (v & !m) | ((v & m) ^ (1 << n)) }

/// Alle Schreibweisen eines Textes für die Oberfläche: `{ ok, value, dec, hex, bin, signed, lo, hi, char, bits, error }`; `bits` = 8 oder 16 (Breite des Bit-Editors)
pub fn alle(text: &str, bits: u8) -> Value {
    let bits = if bits == 16 { 16 } else { 8 };
    match werte(text) {
        Err(e) => json!({ "ok": false, "error": e }),
        Ok(v) => json!({ "ok": true, "value": v, "dec": v.to_string(), "hex": hex(v), "bin": binaer(v), "signed": vorzeichen(v, bits), "bits": bits,
            "lo": v & 0xFF, "hi": (v >> 8) & 0xFF, "char": zeichen(v).map(|c| c.to_string()), "bitList": (0..bits).rev().map(|n| (v >> n) & 1).collect::<Vec<_>>() }),
    }
}
/// Text nach dem Umschalten von Bit `n` (Breite `breite`), als Hexadezimalzahl; bei einem Fehler im Text bleibt er unverändert
pub fn bit_text(text: &str, n: u8, breite: u8) -> String { match werte(text) { Ok(v) => hex(bit_umschalten(v, n, if breite == 16 { 16 } else { 8 })), Err(_) => text.to_string() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rechnet_und_zeigt_schreibweisen() {
        assert_eq!(werte("$D000+$20").unwrap(), 0xD020); assert_eq!(werte("1<<5").unwrap(), 32); assert_eq!(werte("%1010").unwrap(), 10); assert_eq!(werte("0xFF").unwrap(), 255); assert_eq!(werte("-1").unwrap(), -1);
        assert!(werte("").is_err() && werte("1/0").is_err() && werte("$").is_err() && werte("5 5").is_err());
        let v = alle("$D020", 8); assert_eq!(v["dec"], "53280"); assert_eq!(v["hex"], "$D020"); assert_eq!(v["lo"], 0x20); assert_eq!(v["hi"], 0xD0); assert_eq!(v["signed"], 32);
        assert_eq!(alle("65", 8)["char"], "A"); assert_eq!(alle("255", 8)["signed"], -1); assert_eq!(alle("255", 16)["signed"], 255);
        assert_eq!(alle("5", 8)["bitList"], json!([0, 0, 0, 0, 0, 1, 0, 1]));
        assert_eq!(bit_text("$00", 3, 8), "$08"); assert_eq!(bit_text("$08", 3, 8), "$00"); assert_eq!(bit_text("xx", 1, 8), "xx");
    }
}
