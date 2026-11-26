// SPDX-License-Identifier: GPL-3.0-or-later
//! Zahlenwerte für den Hover: Konstanten und Zahlenliterale in mehreren Schreibweisen (dezimal, hexadezimal, binär, mit Vorzeichen, Zeichen, Low-/High-Byte).
use vc_core::{msg, Msg};
use vc_i18n::Lang;

fn hex(v: i64) -> String {
    if v < 0 { return if v >= -32768 { format!("${:04X}", v as u16) } else { format!("${:08X}", v as u32) }; }
    if v <= 0xFF { format!("${v:02X}") } else if v <= 0xFFFF { format!("${v:04X}") } else if v <= 0xFF_FFFF { format!("${v:06X}") } else { format!("${v:08X}") }
}

fn bin(v: i64) -> String {
    let (bits, u) = if v < 0 { if v >= -32768 { (16, v as u16 as u64) } else { (32, v as u32 as u64) } } else { (if v <= 0xFF { 8 } else if v <= 0xFFFF { 16 } else if v <= 0xFF_FFFF { 24 } else { 32 }, v as u64) };
    let s = format!("{u:0bits$b}");
    let groups: Vec<&str> = (0..s.len()).step_by(4).map(|i| &s[i..(i + 4).min(s.len())]).collect();
    format!("%{}", groups.join(" "))
}

/// Markdown für den Hover über einem Wert in der aktuellen Sprache: Überschrift und Tabelle der Schreibweisen
pub fn value_hover(label: &str, v: i64) -> String { value_hover_in(vc_i18n::language(), label, v) }

/// Wie [`value_hover`], aber in der Sprache `lang`
pub fn value_hover_in(lang: Lang, label: &str, v: i64) -> String {
    let t = |m: Msg| m.text_in(lang);
    let mut s = format!("**{label}** = `{}`\n\n{}\n{}\n{}\n{}\n", hex(v), t(msg!("lsp-value-table-head")),
        t(msg!("lsp-value-row-dec", v = v.to_string())), t(msg!("lsp-value-row-hex", v = hex(v))), t(msg!("lsp-value-row-bin", v = bin(v))));
    if (128..=255).contains(&v) { s.push_str(&t(msg!("lsp-value-row-signed8", v = (v - 256).to_string()))); s.push('\n'); }
    else if (32768..=65535).contains(&v) { s.push_str(&t(msg!("lsp-value-row-signed16", v = (v - 65536).to_string()))); s.push('\n'); }
    else if (-128..0).contains(&v) { s.push_str(&t(msg!("lsp-value-row-unsigned8", v = (v + 256).to_string(), hex = hex(v + 256)))); s.push('\n'); }
    else if (-32768..-128).contains(&v) { s.push_str(&t(msg!("lsp-value-row-unsigned16", v = (v + 65536).to_string()))); s.push('\n'); }
    if (32..=126).contains(&v) { s.push_str(&t(msg!("lsp-value-row-ascii", v = (v as u8 as char).to_string()))); s.push('\n'); }
    if (256..=65535).contains(&v) {
        s.push_str(&t(msg!("lsp-value-row-lowbyte", hex = hex(v & 0xFF), dec = (v & 0xFF).to_string()))); s.push('\n');
        s.push_str(&t(msg!("lsp-value-row-highbyte", hex = hex(v >> 8), dec = (v >> 8).to_string()))); s.push('\n');
    }
    s
}

/// Zahlenliteral an Zeichenposition `ch` (Zeichenindex in `line`): `$FF`, `%1010`, `0xFF`, `255`, `'A'`; liefert Anfang, Ende (Zeichenindizes) und Wert
pub fn literal_at(line: &str, ch: usize) -> Option<(usize, usize, i64)> {
    let chars: Vec<char> = line.chars().collect();
    // Zeichenliteral 'x'
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i] == '\'' && chars[i + 2] == '\'' && (i..=i + 2).contains(&ch) && chars[i + 1] != '\'' { return Some((i, i + 3, chars[i + 1] as i64)); }
    }
    let tok = |c: char| c.is_ascii_alphanumeric() || c == '$' || c == '%' || c == '_';
    let mut c = ch.min(chars.len());
    if (c >= chars.len() || !tok(chars[c])) && c > 0 && tok(chars[c - 1]) { c -= 1; }
    if c >= chars.len() || !tok(chars[c]) { return None; }
    let (mut s, mut e) = (c, c + 1);
    while s > 0 && tok(chars[s - 1]) { s -= 1; }
    while e < chars.len() && tok(chars[e]) { e += 1; }
    let t: String = chars[s..e].iter().filter(|c| **c != '_').collect();
    let lower = t.to_ascii_lowercase();
    let v = if let Some(h) = lower.strip_prefix('$').or_else(|| lower.strip_prefix("0x")) { i64::from_str_radix(h, 16).ok() }
        else if let Some(b) = lower.strip_prefix('%') { i64::from_str_radix(b, 2).ok() }
        else if lower.chars().all(|c| c.is_ascii_digit()) && !lower.is_empty() { lower.parse::<i64>().ok() }
        else { None }?;
    // Vorzeichen: `-5` ist negativ, `a-5` (Subtraktion) nicht
    let neg = s > 0 && chars[s - 1] == '-' && (s == 1 || !(chars[s - 2].is_alphanumeric() || matches!(chars[s - 2], '_' | ')' | ']' | '$')) );
    Some(if neg { (s - 1, e, -v) } else { (s, e, v) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_formats() {
        let h = value_hover_in(Lang::De, "VIC", 0xD020);
        assert!(h.starts_with("**VIC** = `$D020`") && h.contains("| Dezimal | 53280 |") && h.contains("`%1101 0000 0010 0000`") && h.contains("Low-Byte (`<`) | `$20` = 32") && h.contains("High-Byte (`>`) | `$D0` = 208"), "{h}");
        let h = value_hover_in(Lang::De, "X", 200);
        assert!(h.contains("`$C8`") && h.contains("`%1100 1000`") && h.contains("Mit Vorzeichen (8 Bit) | -56"), "{h}");
        let h = value_hover_in(Lang::De, "A", 65);
        assert!(h.contains("Zeichen (ASCII) | `'A'`") && h.contains("`%0100 0001`"), "{h}");
        let h = value_hover_in(Lang::De, "N", -1);
        assert!(h.contains("`$FFFF`") && h.contains("255 (`$FF`)"), "{h}");
        let h = value_hover_in(Lang::De, "P", 70000);
        assert!(h.contains("`$011170`") && h.contains("`%0000 0001 0001 0001 0111 0000`"), "{h}");
        assert!(!value_hover_in(Lang::De, "Z", 0).contains("Zeichen"));
    }

    #[test]
    fn value_formats_english() {
        let h = value_hover_in(Lang::En, "A", 65);
        assert!(h.contains("| Decimal | 65 |") && h.contains("| Character (ASCII) | `'A'` |"), "{h}");
        assert!(!h.contains("Dezimal"));
    }

    #[test]
    fn finds_literals() {
        assert_eq!(literal_at("lda #$d020", 6), Some((5, 10, 0xD020)));
        assert_eq!(literal_at("lda #$d020", 10), Some((5, 10, 0xD020)), "Cursor am Ende des Tokens");
        assert_eq!(literal_at("sta $d020,x", 4), Some((4, 9, 0xD020)));
        assert_eq!(literal_at("ldx #%1010_0101", 7), Some((5, 15, 0xA5)));
        assert_eq!(literal_at("lda #255", 6), Some((5, 8, 255)));
        assert_eq!(literal_at("lda #0xFF", 6), Some((5, 9, 255)));
        assert_eq!(literal_at("lda #'A'", 6), Some((5, 8, 65)));
        assert_eq!(literal_at("lda #-5", 6), Some((5, 7, -5)));
        assert_eq!(literal_at("lda #LIVES", 7), None, "Bezeichner ist kein Literal");
        assert_eq!(literal_at("x = a-5", 6).map(|r| r.2), Some(5), "Subtraktion, kein Minuszeichen");
        assert_eq!(literal_at("lda label2", 8), None);
        assert_eq!(literal_at("lda #$", 5), None);
        assert_eq!(literal_at("   ", 1), None);
    }
}
