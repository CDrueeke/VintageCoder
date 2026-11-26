// SPDX-License-Identifier: GPL-3.0-or-later
//! Kurzbeschreibungen für Hover und Completion. Die Texte stehen im Katalog (`lsp-mnemonic-<name>`, `lsp-directive-<name>`); Namen bleiben unübersetzt.
use std::collections::HashMap;
use std::sync::Mutex;
use vc_core::{msg, Msg};
use vc_i18n::Lang;

/// Mnemonics mit Beschreibung (in der Reihenfolge der Dokumentation)
pub const MNEMONIC_NAMES: &[&str] = &["ADC", "AND", "ASL", "BCC", "BCS", "BEQ", "BIT", "BMI", "BNE", "BPL", "BRK", "BVC", "BVS", "CLC", "CLD", "CLI", "CLV", "CMP", "CPX", "CPY", "DEC", "DEX", "DEY", "EOR", "INC", "INX", "INY", "JMP", "JSR", "LDA", "LDX", "LDY", "LSR", "NOP", "ORA", "PHA", "PHP", "PLA", "PLP", "ROL", "ROR", "RTI", "RTS", "SBC", "SEC", "SED", "SEI", "STA", "STX", "STY", "TAX", "TAY", "TSX", "TXA", "TXS", "TYA", "ASR", "ASW", "ROW", "INW", "DEW", "PHW", "BBR0", "BBR1", "BBR2", "BBR3", "BBR4", "BBR5", "BBR6", "BBR7", "BBS0", "BBS1", "BBS2", "BBS3", "BBS4", "BBS5", "BBS6", "BBS7", "RMB0", "RMB1", "RMB2", "RMB3", "RMB4", "RMB5", "RMB6", "RMB7", "SMB0", "SMB1", "SMB2", "SMB3", "SMB4", "SMB5", "SMB6", "SMB7", "BRA", "BSR", "LBCC", "LBCS", "LBEQ", "LBNE", "LBMI", "LBPL", "LBVC", "LBVS", "LBRA", "CLE", "SEE", "CPZ", "LDZ", "STZ", "INZ", "DEZ", "PHX", "PHY", "PHZ", "PLX", "PLY", "PLZ", "TAZ", "TZA", "TAB", "TBA", "TSY", "TYS", "TRB", "TSB", "NEG", "EOM", "MAP", "LDQ", "STQ", "ADCQ", "SBCQ", "ANDQ", "ORQ", "EORQ", "CPQ", "BITQ", "ASLQ", "LSRQ", "ROLQ", "RORQ", "ASRQ", "INQ", "DEQ", "NEGQ", "LAX", "SAX", "DCP", "ISC", "SLO", "RLA", "SRE", "RRA", "ANC", "ALR", "ARR", "AXS", "LAS", "XAA", "LXA", "SHA", "SHX", "SHY", "TAS", "JAM"];

/// Kanonische Direktivennamen samt Blockenden (mit Punkt), aus der zentralen Tabelle `data/asm/directives.json` (vc-asm)
pub fn directive_names() -> Vec<&'static str> { vc_asm::directives::canonical_names() }

fn mnemonic_msg(m: &str) -> Option<Msg> {
    Some(match m {
        "ADC" => msg!("lsp-mnemonic-adc"),
        "AND" => msg!("lsp-mnemonic-and"),
        "ASL" => msg!("lsp-mnemonic-asl"),
        "BCC" => msg!("lsp-mnemonic-bcc"),
        "BCS" => msg!("lsp-mnemonic-bcs"),
        "BEQ" => msg!("lsp-mnemonic-beq"),
        "BIT" => msg!("lsp-mnemonic-bit"),
        "BMI" => msg!("lsp-mnemonic-bmi"),
        "BNE" => msg!("lsp-mnemonic-bne"),
        "BPL" => msg!("lsp-mnemonic-bpl"),
        "BRK" => msg!("lsp-mnemonic-brk"),
        "BVC" => msg!("lsp-mnemonic-bvc"),
        "BVS" => msg!("lsp-mnemonic-bvs"),
        "CLC" => msg!("lsp-mnemonic-clc"),
        "CLD" => msg!("lsp-mnemonic-cld"),
        "CLI" => msg!("lsp-mnemonic-cli"),
        "CLV" => msg!("lsp-mnemonic-clv"),
        "CMP" => msg!("lsp-mnemonic-cmp"),
        "CPX" => msg!("lsp-mnemonic-cpx"),
        "CPY" => msg!("lsp-mnemonic-cpy"),
        "DEC" => msg!("lsp-mnemonic-dec"),
        "DEX" => msg!("lsp-mnemonic-dex"),
        "DEY" => msg!("lsp-mnemonic-dey"),
        "EOR" => msg!("lsp-mnemonic-eor"),
        "INC" => msg!("lsp-mnemonic-inc"),
        "INX" => msg!("lsp-mnemonic-inx"),
        "INY" => msg!("lsp-mnemonic-iny"),
        "JMP" => msg!("lsp-mnemonic-jmp"),
        "JSR" => msg!("lsp-mnemonic-jsr"),
        "LDA" => msg!("lsp-mnemonic-lda"),
        "LDX" => msg!("lsp-mnemonic-ldx"),
        "LDY" => msg!("lsp-mnemonic-ldy"),
        "LSR" => msg!("lsp-mnemonic-lsr"),
        "NOP" => msg!("lsp-mnemonic-nop"),
        "ORA" => msg!("lsp-mnemonic-ora"),
        "PHA" => msg!("lsp-mnemonic-pha"),
        "PHP" => msg!("lsp-mnemonic-php"),
        "PLA" => msg!("lsp-mnemonic-pla"),
        "PLP" => msg!("lsp-mnemonic-plp"),
        "ROL" => msg!("lsp-mnemonic-rol"),
        "ROR" => msg!("lsp-mnemonic-ror"),
        "RTI" => msg!("lsp-mnemonic-rti"),
        "RTS" => msg!("lsp-mnemonic-rts"),
        "SBC" => msg!("lsp-mnemonic-sbc"),
        "SEC" => msg!("lsp-mnemonic-sec"),
        "SED" => msg!("lsp-mnemonic-sed"),
        "SEI" => msg!("lsp-mnemonic-sei"),
        "STA" => msg!("lsp-mnemonic-sta"),
        "STX" => msg!("lsp-mnemonic-stx"),
        "STY" => msg!("lsp-mnemonic-sty"),
        "TAX" => msg!("lsp-mnemonic-tax"),
        "TAY" => msg!("lsp-mnemonic-tay"),
        "TSX" => msg!("lsp-mnemonic-tsx"),
        "TXA" => msg!("lsp-mnemonic-txa"),
        "TXS" => msg!("lsp-mnemonic-txs"),
        "TYA" => msg!("lsp-mnemonic-tya"),
        "ASR" => msg!("lsp-mnemonic-asr"),
        "ASW" => msg!("lsp-mnemonic-asw"),
        "ROW" => msg!("lsp-mnemonic-row"),
        "INW" => msg!("lsp-mnemonic-inw"),
        "DEW" => msg!("lsp-mnemonic-dew"),
        "PHW" => msg!("lsp-mnemonic-phw"),
        "BBR0" => msg!("lsp-mnemonic-bbr0"),
        "BBR1" => msg!("lsp-mnemonic-bbr1"),
        "BBR2" => msg!("lsp-mnemonic-bbr2"),
        "BBR3" => msg!("lsp-mnemonic-bbr3"),
        "BBR4" => msg!("lsp-mnemonic-bbr4"),
        "BBR5" => msg!("lsp-mnemonic-bbr5"),
        "BBR6" => msg!("lsp-mnemonic-bbr6"),
        "BBR7" => msg!("lsp-mnemonic-bbr7"),
        "BBS0" => msg!("lsp-mnemonic-bbs0"),
        "BBS1" => msg!("lsp-mnemonic-bbs1"),
        "BBS2" => msg!("lsp-mnemonic-bbs2"),
        "BBS3" => msg!("lsp-mnemonic-bbs3"),
        "BBS4" => msg!("lsp-mnemonic-bbs4"),
        "BBS5" => msg!("lsp-mnemonic-bbs5"),
        "BBS6" => msg!("lsp-mnemonic-bbs6"),
        "BBS7" => msg!("lsp-mnemonic-bbs7"),
        "RMB0" => msg!("lsp-mnemonic-rmb0"),
        "RMB1" => msg!("lsp-mnemonic-rmb1"),
        "RMB2" => msg!("lsp-mnemonic-rmb2"),
        "RMB3" => msg!("lsp-mnemonic-rmb3"),
        "RMB4" => msg!("lsp-mnemonic-rmb4"),
        "RMB5" => msg!("lsp-mnemonic-rmb5"),
        "RMB6" => msg!("lsp-mnemonic-rmb6"),
        "RMB7" => msg!("lsp-mnemonic-rmb7"),
        "SMB0" => msg!("lsp-mnemonic-smb0"),
        "SMB1" => msg!("lsp-mnemonic-smb1"),
        "SMB2" => msg!("lsp-mnemonic-smb2"),
        "SMB3" => msg!("lsp-mnemonic-smb3"),
        "SMB4" => msg!("lsp-mnemonic-smb4"),
        "SMB5" => msg!("lsp-mnemonic-smb5"),
        "SMB6" => msg!("lsp-mnemonic-smb6"),
        "SMB7" => msg!("lsp-mnemonic-smb7"),
        "BRA" => msg!("lsp-mnemonic-bra"),
        "BSR" => msg!("lsp-mnemonic-bsr"),
        "LBCC" => msg!("lsp-mnemonic-lbcc"),
        "LBCS" => msg!("lsp-mnemonic-lbcs"),
        "LBEQ" => msg!("lsp-mnemonic-lbeq"),
        "LBNE" => msg!("lsp-mnemonic-lbne"),
        "LBMI" => msg!("lsp-mnemonic-lbmi"),
        "LBPL" => msg!("lsp-mnemonic-lbpl"),
        "LBVC" => msg!("lsp-mnemonic-lbvc"),
        "LBVS" => msg!("lsp-mnemonic-lbvs"),
        "LBRA" => msg!("lsp-mnemonic-lbra"),
        "CLE" => msg!("lsp-mnemonic-cle"),
        "SEE" => msg!("lsp-mnemonic-see"),
        "CPZ" => msg!("lsp-mnemonic-cpz"),
        "LDZ" => msg!("lsp-mnemonic-ldz"),
        "STZ" => msg!("lsp-mnemonic-stz"),
        "INZ" => msg!("lsp-mnemonic-inz"),
        "DEZ" => msg!("lsp-mnemonic-dez"),
        "PHX" => msg!("lsp-mnemonic-phx"),
        "PHY" => msg!("lsp-mnemonic-phy"),
        "PHZ" => msg!("lsp-mnemonic-phz"),
        "PLX" => msg!("lsp-mnemonic-plx"),
        "PLY" => msg!("lsp-mnemonic-ply"),
        "PLZ" => msg!("lsp-mnemonic-plz"),
        "TAZ" => msg!("lsp-mnemonic-taz"),
        "TZA" => msg!("lsp-mnemonic-tza"),
        "TAB" => msg!("lsp-mnemonic-tab"),
        "TBA" => msg!("lsp-mnemonic-tba"),
        "TSY" => msg!("lsp-mnemonic-tsy"),
        "TYS" => msg!("lsp-mnemonic-tys"),
        "TRB" => msg!("lsp-mnemonic-trb"),
        "TSB" => msg!("lsp-mnemonic-tsb"),
        "NEG" => msg!("lsp-mnemonic-neg"),
        "EOM" => msg!("lsp-mnemonic-eom"),
        "MAP" => msg!("lsp-mnemonic-map"),
        "LDQ" => msg!("lsp-mnemonic-ldq"),
        "STQ" => msg!("lsp-mnemonic-stq"),
        "ADCQ" => msg!("lsp-mnemonic-adcq"),
        "SBCQ" => msg!("lsp-mnemonic-sbcq"),
        "ANDQ" => msg!("lsp-mnemonic-andq"),
        "ORQ" => msg!("lsp-mnemonic-orq"),
        "EORQ" => msg!("lsp-mnemonic-eorq"),
        "CPQ" => msg!("lsp-mnemonic-cpq"),
        "BITQ" => msg!("lsp-mnemonic-bitq"),
        "ASLQ" => msg!("lsp-mnemonic-aslq"),
        "LSRQ" => msg!("lsp-mnemonic-lsrq"),
        "ROLQ" => msg!("lsp-mnemonic-rolq"),
        "RORQ" => msg!("lsp-mnemonic-rorq"),
        "ASRQ" => msg!("lsp-mnemonic-asrq"),
        "INQ" => msg!("lsp-mnemonic-inq"),
        "DEQ" => msg!("lsp-mnemonic-deq"),
        "NEGQ" => msg!("lsp-mnemonic-negq"),
        "LAX" => msg!("lsp-mnemonic-lax"),
        "SAX" => msg!("lsp-mnemonic-sax"),
        "DCP" => msg!("lsp-mnemonic-dcp"),
        "ISC" => msg!("lsp-mnemonic-isc"),
        "SLO" => msg!("lsp-mnemonic-slo"),
        "RLA" => msg!("lsp-mnemonic-rla"),
        "SRE" => msg!("lsp-mnemonic-sre"),
        "RRA" => msg!("lsp-mnemonic-rra"),
        "ANC" => msg!("lsp-mnemonic-anc"),
        "ALR" => msg!("lsp-mnemonic-alr"),
        "ARR" => msg!("lsp-mnemonic-arr"),
        "AXS" => msg!("lsp-mnemonic-axs"),
        "LAS" => msg!("lsp-mnemonic-las"),
        "XAA" => msg!("lsp-mnemonic-xaa"),
        "LXA" => msg!("lsp-mnemonic-lxa"),
        "SHA" => msg!("lsp-mnemonic-sha"),
        "SHX" => msg!("lsp-mnemonic-shx"),
        "SHY" => msg!("lsp-mnemonic-shy"),
        "TAS" => msg!("lsp-mnemonic-tas"),
        "JAM" => msg!("lsp-mnemonic-jam"),
        _ => return None,
    })
}

fn directive_msg_exact(d: &str) -> Option<Msg> {
    Some(match d {
        ".bank" => msg!("lsp-directive-bank"),
        ".segment" => msg!("lsp-directive-segment"),
        ".cartridge" => msg!("lsp-directive-cartridge"),
        ".org" => msg!("lsp-directive-org"),
        ".byte" => msg!("lsp-directive-byte"),
        ".word" => msg!("lsp-directive-word"),
        ".text" => msg!("lsp-directive-text"),
        ".petscii" => msg!("lsp-directive-petscii"),
        ".screencode" => msg!("lsp-directive-screencode"),
        ".atascii" => msg!("lsp-directive-atascii"),
        ".encoding" => msg!("lsp-directive-encoding"),
        ".textnull" => msg!("lsp-directive-textnull"),
        ".textlen" => msg!("lsp-directive-textlen"),
        ".textshift" => msg!("lsp-directive-textshift"),
        ".retaddr" => msg!("lsp-directive-retaddr"),
        ".be16" => msg!("lsp-directive-be16"),
        ".be32" => msg!("lsp-directive-be32"),
        ".dword" => msg!("lsp-directive-dword"),
        ".hex" => msg!("lsp-directive-hex"),
        ".var" => msg!("lsp-directive-var"),
        ".set" => msg!("lsp-directive-set"),
        ".for" => msg!("lsp-directive-for"),
        ".endfor" => msg!("lsp-directive-endfor"),
        ".break" => msg!("lsp-directive-break"),
        ".continue" => msg!("lsp-directive-continue"),
        ".function" => msg!("lsp-directive-function"),
        ".exitmacro" => msg!("lsp-directive-exitmacro"),
        ".fill" => msg!("lsp-directive-fill"),
        ".lohitable" => msg!("lsp-directive-lohitable"),
        ".branchlong" => msg!("lsp-directive-branchlong"),
        ".struct" => msg!("lsp-directive-struct"),
        ".endstruct" => msg!("lsp-directive-endstruct"),
        ".enum" => msg!("lsp-directive-enum"),
        ".endenum" => msg!("lsp-directive-endenum"),
        ".align" => msg!("lsp-directive-align"),
        ".const" => msg!("lsp-directive-const"),
        ".include" => msg!("lsp-directive-include"),
        ".incbin" => msg!("lsp-directive-incbin"),
        ".if" => msg!("lsp-directive-if"),
        ".ifdef" => msg!("lsp-directive-ifdef"),
        ".ifndef" => msg!("lsp-directive-ifndef"),
        ".else" => msg!("lsp-directive-else"),
        ".elseif" => msg!("lsp-directive-elseif"),
        ".endif" => msg!("lsp-directive-endif"),
        ".macro" => msg!("lsp-directive-macro"),
        ".endmacro" => msg!("lsp-directive-endmacro"),
        ".repeat" => msg!("lsp-directive-repeat"),
        ".endrepeat" => msg!("lsp-directive-endrepeat"),
        ".relocate" => msg!("lsp-directive-relocate"),
        ".endrelocate" => msg!("lsp-directive-endrelocate"),
        ".cycles" => msg!("lsp-directive-cycles"),
        ".endcycles" => msg!("lsp-directive-endcycles"),
        ".basicstub" => msg!("lsp-directive-basicstub"),
        ".run" => msg!("lsp-directive-run"),
        ".assert" => msg!("lsp-directive-assert"),
        ".error" => msg!("lsp-directive-error"),
        ".warning" => msg!("lsp-directive-warning"),
        ".target" => msg!("lsp-directive-target"),
        ".cpu" => msg!("lsp-directive-cpu"),
        ".scope" => msg!("lsp-directive-scope"),
        ".endscope" => msg!("lsp-directive-endscope"),
        ".namespace" => msg!("lsp-directive-namespace"),
        _ => return None,
    })
}

/// Beschreibung zu einem Namen der Tabelle; Alias liefern die Beschreibung des kanonischen Namens
fn directive_msg(d: &str) -> Option<Msg> { directive_msg_exact(vc_asm::directives::canonical(d)?) }

/// Beschreibung eines Mnemonics (großgeschrieben) in `lang`
pub fn mnemonic_doc_in(lang: Lang, m: &str) -> Option<String> { mnemonic_msg(m).map(|x| x.text_in(lang)) }
/// Beschreibung einer Direktive (mit Punkt, kleingeschrieben) in `lang`
pub fn directive_doc_in(lang: Lang, d: &str) -> Option<String> { directive_msg(d).map(|x| x.text_in(lang)) }

/// Text in `'static` für Aufrufer, die ihn so brauchen (je Sprache und Schlüssel einmal angelegt, höchstens einige hundert kurze Texte)
fn cached(lang: Lang, kind: u8, name: &str, make: impl FnOnce() -> Option<String>) -> Option<&'static str> {
    static CACHE: Mutex<Option<HashMap<(Lang, u8, String), &'static str>>> = Mutex::new(None);
    let mut g = CACHE.lock().ok()?;
    let map = g.get_or_insert_with(HashMap::new);
    let k = (lang, kind, name.to_string());
    if let Some(s) = map.get(&k) { return Some(*s); }
    let s: &'static str = Box::leak(make()?.into_boxed_str());
    map.insert(k, s);
    Some(s)
}

/// Beschreibung eines Mnemonics in der aktuellen Sprache
pub fn mnemonic_doc(m: &str) -> Option<&'static str> { let l = vc_i18n::language(); cached(l, 0, m, || mnemonic_doc_in(l, m)) }
/// Beschreibung einer Direktive in der aktuellen Sprache
pub fn directive_doc(d: &str) -> Option<&'static str> { let l = vc_i18n::language(); cached(l, 1, d, || directive_doc_in(l, d)) }

/// Tabelle (Name, Beschreibung) aller Direktiven in der aktuellen Sprache; `for (d, doc) in DIRECTIVES` funktioniert wie früher mit der festen Liste
pub struct DirectiveTable;
pub const DIRECTIVES: DirectiveTable = DirectiveTable;

impl IntoIterator for DirectiveTable {
    type Item = (&'static str, &'static str);
    type IntoIter = std::vec::IntoIter<(&'static str, &'static str)>;
    fn into_iter(self) -> Self::IntoIter { directive_names().into_iter().filter_map(|n| directive_doc(n).map(|d| (n, d))).collect::<Vec<_>>().into_iter() }
}
