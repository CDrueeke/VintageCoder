// SPDX-License-Identifier: GPL-3.0-or-later
//! Färbung einer Zeile: die Zerlegung kommt aus `vc-ide-model` (`syntax::tokens`, dieselbe wie im Altbestand), hier nur die Abbildung
//! auf die Farbarten der neuen Oberfläche und die Umrechnung auf Zeichenspalten.
use std::collections::HashSet;
use std::sync::OnceLock;
use vc_ide_model::lang::Lang;
use vc_ide_model::syntax::{self, Kind as M};
pub use vc_gui_ui::theme::Kind;

pub const MNEMONICS: &[&str] = &["adc", "and", "asl", "bcc", "bcs", "beq", "bit", "bmi", "bne", "bpl", "brk", "bvc", "bvs", "clc", "cld", "cli", "clv", "cmp", "cpx", "cpy", "dec", "dex", "dey", "eor", "inc", "inx", "iny", "jmp", "jsr", "lda", "ldx", "ldy", "lsr", "nop", "ora", "pha", "php", "pla", "plp", "rol", "ror", "rti", "rts", "sbc", "sec", "sed", "sei", "sta", "stx", "sty", "tax", "tay", "tsx", "txa", "txs", "tya"];
pub const DIRECTIVES: &[&str] = &[".byte", ".word", ".text", ".org", ".include", ".fill", ".if", ".endif", ".macro", ".endmacro", ".align"];

fn mnemonics() -> &'static HashSet<String> { static S: OnceLock<HashSet<String>> = OnceLock::new(); S.get_or_init(|| MNEMONICS.iter().map(|m| m.to_string()).collect()) }

fn art(k: M) -> Kind { match k { M::Comment => Kind::Comment, M::Str => Kind::Str, M::Num | M::Hex | M::Bin => Kind::Num, M::Directive | M::Keyword => Kind::Directive, M::Mnemonic => Kind::Mnemonic, M::Label | M::Symbol | M::Register => Kind::Label, M::Plain | M::Bad => Kind::Plain } }

/// Zerlegt eine Zeile in (Startspalte, Endspalte, Art), Spalten in Zeichen; Normalfarbe wird nicht aufgeführt.
pub fn tokens(line: &str) -> Vec<(usize, usize, Kind)> {
    let byte_to_char = |b: usize| line[..b.min(line.len())].chars().count();
    syntax::tokens(line, Lang::Asm, mnemonics(), None).into_iter().filter(|t| t.2 != M::Plain).map(|(a, b, k)| (byte_to_char(a), byte_to_char(b), art(k))).filter(|t| t.2 != Kind::Plain).collect()
}
#[cfg(test)]
mod tests { use super::*;
    #[test] fn zerlegt_zeile() {
        let t = tokens("start:  lda #$00 ; Rahmen"); assert_eq!(t[0].2, Kind::Label); assert_eq!(t[1].2, Kind::Mnemonic); assert_eq!(t[2].2, Kind::Num); assert_eq!(t[3].2, Kind::Comment); }
    #[test] fn direktive_und_umlaut() { let t = tokens("  .text \"hällo\""); assert_eq!(t[0].2, Kind::Directive); assert_eq!(t[1], (8, 15, Kind::Str)); }
}
