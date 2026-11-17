// SPDX-License-Identifier: GPL-3.0-or-later
//! SPIKE: kleiner Assembler-Färber (GUI-frei). Der Altbestand (`vc-app/highlight.rs`, in der Historie) trennt Zerlegung (`tokens`) und
//! egui-Ausgabe; die Zerlegung ließe sich nach `vc-ide-model` heben. Hier nur das Nötigste.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Plain, Comment, Str, Num, Directive, Mnemonic, Label }

pub const MNEMONICS: &[&str] = &["adc", "and", "asl", "bcc", "bcs", "beq", "bit", "bmi", "bne", "bpl", "brk", "bvc", "bvs", "clc", "cld", "cli", "clv", "cmp", "cpx", "cpy", "dec", "dex", "dey", "eor", "inc", "inx", "iny", "jmp", "jsr", "lda", "ldx", "ldy", "lsr", "nop", "ora", "pha", "php", "pla", "plp", "rol", "ror", "rti", "rts", "sbc", "sec", "sed", "sei", "sta", "stx", "sty", "tax", "tay", "tsx", "txa", "txs", "tya"];
pub const DIRECTIVES: &[&str] = &[".byte", ".word", ".text", ".org", ".include", ".fill", ".if", ".endif", ".macro", ".endmacro", ".align"];

/// Zerlegt eine Zeile in (Startspalte, Endspalte, Art), Spalten in Zeichen.
pub fn tokens(line: &str) -> Vec<(usize, usize, Kind)> {
    let b: Vec<char> = line.chars().collect(); let mut out = vec![]; let mut i = 0; let mut first_word = true;
    if !b.is_empty() && !b[0].is_whitespace() && b[0] != ';' { // Marke am Zeilenanfang
        let mut j = 0; while j < b.len() && (b[j].is_alphanumeric() || b[j] == '_' || b[j] == '.') { j += 1 }
        if j > 0 && (j == b.len() || b[j] == ':' || b[j].is_whitespace()) && !(b[0] == '.' ) && !MNEMONICS.contains(&b[..j].iter().collect::<String>().to_lowercase().as_str()) { out.push((0, if j < b.len() && b[j] == ':' { j + 1 } else { j }, Kind::Label)); i = if j < b.len() && b[j] == ':' { j + 1 } else { j }; }
    }
    while i < b.len() {
        let c = b[i]; let s = i;
        if c.is_whitespace() { i += 1; continue }
        if c == ';' { out.push((s, b.len(), Kind::Comment)); break }
        if c == '"' { i += 1; while i < b.len() && b[i] != '"' { i += 1 } i = (i + 1).min(b.len()); out.push((s, i, Kind::Str)); continue }
        if c == '$' || c == '%' || c.is_ascii_digit() || (c == '#' && i + 1 < b.len() && (b[i + 1] == '$' || b[i + 1] == '%' || b[i + 1].is_ascii_digit())) {
            i += 1; while i < b.len() && (b[i].is_ascii_hexdigit() || b[i] == '$' || b[i] == '%') { i += 1 } out.push((s, i, Kind::Num)); continue }
        if c.is_alphabetic() || c == '_' || c == '.' {
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_' || b[i] == '.') { i += 1 }
            let w: String = b[s..i].iter().collect(); let lw = w.to_lowercase();
            let k = if first_word && MNEMONICS.contains(&lw.as_str()) { Kind::Mnemonic } else if first_word && lw.starts_with('.') && DIRECTIVES.contains(&lw.as_str()) { Kind::Directive } else { Kind::Plain };
            if k != Kind::Plain { out.push((s, i, k)) } first_word = false; continue
        }
        first_word = false; i += 1;
    }
    out
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn zerlegt_zeile() {
        let t = tokens("start:  lda #$00 ; Rahmen"); assert_eq!(t[0].2, Kind::Label); assert_eq!(t[1].2, Kind::Mnemonic); assert_eq!(t[2].2, Kind::Num); assert_eq!(t[3].2, Kind::Comment); }
    #[test] fn direktive_und_umlaut() { let t = tokens("  .text \"hällo\""); assert_eq!(t[0].2, Kind::Directive); assert_eq!(t[1], (8, 15, Kind::Str)); }
}
