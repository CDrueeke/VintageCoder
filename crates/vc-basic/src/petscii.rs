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
            let (name, count) = match body.rsplit_once('*') { Some((n, k)) => (n.trim().to_string(), k.trim().parse::<usize>().map_err(|_| format!("ungültige Anzahl in {{{body}}}"))?), None => (body.trim().to_string(), 1) };
            let code = if let Some(h) = name.strip_prefix('$') { u8::from_str_radix(h, 16).map_err(|_| format!("ungültiger Code {{{body}}}"))? }
                       else { d.control_code(&name.to_ascii_lowercase()).ok_or(format!("unbekannter Steuercode {{{name}}}"))? };
            out.extend(std::iter::repeat(code).take(count));
            i = end + 1;
        } else {
            out.push(from_ascii(c[i]).ok_or(format!("Zeichen '{}' hat keine PETSCII-Entsprechung (nutze {{$xx}})", c[i]))?);
            i += 1;
        }
    }
    Ok(out)
}
