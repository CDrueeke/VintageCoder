// SPDX-License-Identifier: GPL-3.0-or-later
//! Hex-Vorschläge für importierte BASIC-Programme: Dezimaladressen in `POKE`, `PEEK`, `SYS`, `WAIT`, `USR`, die ein Chipregister des
//! Zielsystems sind, könnten als `$D020` lesbarer geschrieben werden. Der Vorschlag ändert nichts; der Anwender wählt jede Zeile einzeln aus
//! (Dialog im Editor, `vcc basic --suggest-hex`). Rechenausdrücke, `DATA` und Zeichenketten kommen nie vor (siehe `vc_basic::numtools::suggest_hex`).
use crate::chipmap::ChipMap;
use std::path::Path;
use vc_basic::numbers::{Lexicon, NumberOptions};

pub use vc_basic::numtools::{apply_suggestions, Suggestion};

/// Vorschläge für `text` im BASIC-Dialekt `dialect` (z. B. `cbm_v2`) und für die Register des Zielsystems `system` (z. B. `c64`)
pub fn suggestions(data: &Path, dialect: &str, system: &str, text: &str) -> Result<Vec<Suggestion>, String> {
    let map = ChipMap::load(data, system);
    let known = |a: u32| map.known(a);
    let o = NumberOptions::default();
    Ok(if vc_basic::dialect_family(data, dialect)? == "atari" {
        let d = vc_basic::atari::AtariDialect::load(data, dialect)?;
        vc_basic::numtools::suggest_hex(text, &Lexicon::Atari(&d), &o, &known)
    } else {
        let d = vc_basic::Dialect::load(data, dialect)?;
        vc_basic::numtools::suggest_hex(text, &Lexicon::Cbm(&d), &o, &known)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn suggestions_use_the_registers_of_the_target() {
        let src = "10 POKE 53280,0\n20 POKE 1024,1\n30 X=PEEK(56320)\n40 POKE 53280+1,1\n";
        let s = suggestions(&data(), "cbm_v2", "c64", src).unwrap();
        let got: Vec<(usize, &str)> = s.iter().map(|x| (x.line + 1, x.to.as_str())).collect();
        assert_eq!(got, vec![(1, "$D020"), (3, "$DC00")]);
        // Atari: andere Register
        let a = suggestions(&data(), "atari_basic", "atari8", "10 POKE 53248,1\n20 POKE 710,2\n").unwrap();
        assert!(a.iter().all(|x| x.value >= 256));
        // nichts angewendet, solange nichts gewählt ist
        assert_eq!(apply_suggestions(src, &s, &vec![false; s.len()]), src);
    }

    #[test]
    fn import_only_hints_and_never_rewrites() {
        let d = vc_basic::Dialect::load(&data(), "cbm_v2").unwrap();
        let prg = vc_basic::compile_plain_cbm("POKE 53280,0\nPOKE 1024,1\n", &d, &vc_basic::PlainOptions::new("t.bas", 63999), 0x0801).unwrap().prg;
        let root = std::env::temp_dir().join(format!("vc_hexhint_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
        let out = crate::diskimage::import_data(&root, "spiel.prg", &prg, true, false, Some(&data()), &Default::default()).unwrap();
        assert!(out.notes.iter().any(|n| n.contains("1 Adresse") && n.contains("Hex")), "{:?}", out.notes);
        let text = std::fs::read_to_string(root.join(&out.files[0])).unwrap();
        assert!(text.contains("53280") && !text.contains('$'), "{text}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
