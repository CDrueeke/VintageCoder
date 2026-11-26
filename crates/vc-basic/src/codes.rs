// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnosecodes des BASIC-Teils ab `B150` (Konzept „BASIC-Editor und Dialekte“, Anhang A): Vergabe, Art und Abschaltbarkeit.
//! Die Texte stehen im Katalog (`diag-basic-<code>-…`); hier steht nur, welcher Code was ist und ob er eine Warnung (abschaltbar) ist.

/// Art einer Meldung
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Error, Warning, Hint }

/// Ein vergebener Code: Kennung, Art, Kurzname (Bestandteil des Katalogschlüssels)
#[derive(Debug, Clone, Copy)]
pub struct CodeInfo { pub code: &'static str, pub kind: Kind, pub name: &'static str }

use Kind::{Error as E, Hint as H, Warning as W};

/// Alle Codes ab B150 (vergeben oder reserviert); die Prüfungen in `check`, `convert` usw. verwenden nur diese
pub const CODES: &[CodeInfo] = &[
    CodeInfo { code: "B150", kind: W, name: "keyword-in-name" },
    CodeInfo { code: "B151", kind: W, name: "for-next" },
    CodeInfo { code: "B152", kind: W, name: "gosub-return" },
    CodeInfo { code: "B153", kind: W, name: "target-missing" },
    CodeInfo { code: "B154", kind: W, name: "variable-clash" },
    CodeInfo { code: "B155", kind: H, name: "string-open" },
    CodeInfo { code: "B156", kind: W, name: "arg-count" },
    CodeInfo { code: "B157", kind: W, name: "keyword-unknown" },
    CodeInfo { code: "B158", kind: W, name: "then-colon" },
    CodeInfo { code: "B159", kind: W, name: "function-args" },
    CodeInfo { code: "B160", kind: W, name: "mixed-mode" },
    CodeInfo { code: "B161", kind: E, name: "convert-failed" },
    CodeInfo { code: "B162", kind: E, name: "computed-target" },
    CodeInfo { code: "B163", kind: W, name: "runtime-line" },
    CodeInfo { code: "B164", kind: H, name: "number-is-line" },
    CodeInfo { code: "B165", kind: W, name: "marker-orphan" },
    CodeInfo { code: "B166", kind: E, name: "include-mode" },
    CodeInfo { code: "B167", kind: W, name: "quote-not-comment" },
    CodeInfo { code: "B168", kind: W, name: "marker-invalid" },
    CodeInfo { code: "B170", kind: W, name: "dialect-system" },
    CodeInfo { code: "B171", kind: E, name: "module-missing" },
    CodeInfo { code: "B172", kind: E, name: "define-keyword" },
    CodeInfo { code: "B173", kind: E, name: "blob-range" },
    CodeInfo { code: "B174", kind: E, name: "define-syntax" },
    CodeInfo { code: "B175", kind: E, name: "if-structure" },
    CodeInfo { code: "B176", kind: E, name: "define-cycle" },
    CodeInfo { code: "B177", kind: E, name: "blob-error" },
    CodeInfo { code: "B178", kind: E, name: "dialect-unknown" },
    CodeInfo { code: "B180", kind: E, name: "atari-statement" },
    CodeInfo { code: "B181", kind: E, name: "atari-expression" },
    CodeInfo { code: "B182", kind: E, name: "atari-name" },
    CodeInfo { code: "B183", kind: E, name: "atari-number" },
    CodeInfo { code: "B184", kind: E, name: "atari-text" },
    CodeInfo { code: "B185", kind: E, name: "atari-limit" },
    CodeInfo { code: "B190", kind: E, name: "numbering-syntax" },
    CodeInfo { code: "B191", kind: E, name: "numbering-range" },
    CodeInfo { code: "B192", kind: E, name: "numbering-overflow" },
    CodeInfo { code: "B193", kind: E, name: "media-failed" },
    CodeInfo { code: "B194", kind: W, name: "legacy-path" },
    CodeInfo { code: "B195", kind: W, name: "program-too-big" },
    CodeInfo { code: "B196", kind: W, name: "media-restore-order" },
];

pub fn info(code: &str) -> Option<&'static CodeInfo> { CODES.iter().find(|c| c.code == code) }

/// Warnungen und Hinweise lassen sich je Code abschalten (Fehler nicht)
pub fn can_disable(code: &str) -> bool { info(code).map_or(false, |c| c.kind != Kind::Error) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codes_are_unique_and_sorted() {
        let mut last = "";
        for c in CODES { assert!(c.code > last, "{} nach {}", c.code, last); last = c.code; }
    }
    #[test]
    fn only_warnings_can_be_disabled() { assert!(can_disable("B160")); assert!(can_disable("B155")); assert!(!can_disable("B162")); assert!(!can_disable("B999")); }
}
