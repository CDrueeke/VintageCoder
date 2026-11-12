// SPDX-License-Identifier: GPL-3.0-or-later
//! Farben des Designsystems „Retro Terminal IDE“ (Spezifikation des Designers, Hell-Variante abgeleitet).
//! Alle eigenen Farben der Oberfläche kommen von hier; `theme.rs` reicht sie an ältere Aufrufer weiter.
use eframe::egui::Color32;

#[derive(Clone, Copy)]
#[allow(dead_code)]   // dim, pu, on_cy: für die folgenden Phasen des Designkonzepts
pub struct Tokens {
    /// tiefster Grund (Eingabefelder, Editor-Hintergrund)
    pub bg0: Color32,
    /// Grundfläche und Panels
    pub bg1: Color32,
    /// Karten und Docks
    pub bg2: Color32,
    /// erhöhte Flächen: Popups, Dialoge
    pub bg3: Color32,
    /// Rahmen und Trenner
    pub line: Color32,
    pub fg: Color32,
    pub mute: Color32,
    pub dim: Color32,
    /// Primärfarbe: aktiv, Fokus, Auswahl
    pub cy: Color32,
    /// Erfolg, Lauf
    pub gr: Color32,
    /// Warnung, Parameter
    pub am: Color32,
    /// Fehler, Haltepunkte
    pub rd: Color32,
    /// Sprünge, Makros
    pub pu: Color32,
    /// Schriftfarbe auf Flächen in `cy`
    pub on_cy: Color32,
}

const fn c(r: u8, g: u8, b: u8) -> Color32 { Color32::from_rgb(r, g, b) }

pub const DARK: Tokens = Tokens {
    bg0: c(0x0b, 0x0e, 0x18), bg1: c(0x10, 0x13, 0x1d), bg2: c(0x18, 0x1b, 0x25), bg3: c(0x22, 0x26, 0x34), line: c(0x2a, 0x2f, 0x42),
    fg: c(0xe2, 0xe8, 0xf0), mute: c(0x94, 0xa3, 0xb8), dim: c(0x64, 0x70, 0x88),
    cy: c(0x00, 0xc0, 0xff), gr: c(0x10, 0xb9, 0x81), am: c(0xf5, 0x9e, 0x0b), rd: c(0xef, 0x44, 0x44), pu: c(0xa8, 0x55, 0xf7), on_cy: c(0x06, 0x13, 0x1a),
};

pub const LIGHT: Tokens = Tokens {
    bg0: c(0xff, 0xff, 0xff), bg1: c(0xf3, 0xf5, 0xf9), bg2: c(0xe9, 0xed, 0xf4), bg3: c(0xff, 0xff, 0xff), line: c(0xcf, 0xd6, 0xe3),
    fg: c(0x1a, 0x22, 0x33), mute: c(0x4a, 0x56, 0x6d), dim: c(0x7a, 0x86, 0x9c),
    cy: c(0x0a, 0x67, 0xb0), gr: c(0x0b, 0x7d, 0x57), am: c(0x9c, 0x64, 0x00), rd: c(0xc0, 0x26, 0x3f), pu: c(0x7b, 0x38, 0xc2), on_cy: c(0xff, 0xff, 0xff),
};

/// Tokens des aktiven Schemas
pub fn t() -> &'static Tokens { if crate::theme::is_light() { &LIGHT } else { &DARK } }

/// Hintergrund einer Marke: Farbe mit geringer Deckkraft
pub fn tint(color: Color32) -> Color32 { color.gamma_multiply(0.16) }

/// Syntaxfarben des Quelltext-Editors, aus den Tokens des aktiven Schemas abgeleitet
#[derive(Clone, Copy)]
pub struct Syntax {
    pub text: Color32, pub opcode: Color32, pub number: Color32, pub directive: Color32,
    pub comment: Color32, pub string: Color32, pub label: Color32, pub error: Color32,
}

pub fn syntax_of(k: &Tokens) -> Syntax {
    // Violett ist im Dunkelschema als Fließtext zu dunkel für AA: eigener, aufgehellter Syntaxwert
    let directive = if k.bg0 == DARK.bg0 { c(0xbd, 0x8a, 0xfa) } else { k.pu };
    Syntax { text: k.fg, opcode: k.cy, number: k.gr, directive, comment: k.mute, string: k.am, label: k.fg, error: k.rd }
}

/// Syntaxfarben des aktiven Schemas
pub fn syntax() -> Syntax { syntax_of(t()) }

#[allow(dead_code)]
fn lin(v: u8) -> f64 { let c = v as f64 / 255.0; if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) } }
/// Relative Leuchtdichte nach WCAG 2.x
#[allow(dead_code)]
pub fn luminance(c: Color32) -> f64 { 0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b()) }
/// Kontrastverhältnis zweier Farben (1 bis 21)
#[allow(dead_code)]
pub fn contrast(a: Color32, b: Color32) -> f64 { let (x, y) = (luminance(a), luminance(b)); let (hi, lo) = if x > y { (x, y) } else { (y, x) }; (hi + 0.05) / (lo + 0.05) }

/// Bewertung eines Kontrastpaares nach WCAG: (Verhältnis, Text, 0 = AA, 1 = nur für große Schrift, 2 = zu wenig)
pub fn contrast_rating(fg: Color32, bg: Color32) -> (f64, &'static str, u8) {
    let r = contrast(fg, bg);
    if r >= 4.5 { (r, "AA ✓", 0) } else if r >= 3.0 { (r, "nur groß", 1) } else { (r, "zu wenig", 2) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_reference_values() {
        assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.001);
    }

    #[test]
    fn syntax_colours_meet_aa_against_the_editor_ground() {
        let mut bad: Vec<String> = vec![];
        for (name, k) in [("dunkel", &DARK), ("hell", &LIGHT)] {
            let s = syntax_of(k);
            for (what, col) in [("Text", s.text), ("Opcode", s.opcode), ("Zahl", s.number), ("Direktive", s.directive), ("Kommentar", s.comment), ("String", s.string), ("Label", s.label), ("Fehler", s.error)] {
                for (gname, ground) in [("bg0", k.bg0)] {
                    let r = contrast(col, ground);
                    if r < 4.5 { bad.push(format!("{what} ({name}) auf {gname}: {r:.2}")); }
                }
            }
        }
        assert!(bad.is_empty(), "zu wenig Kontrast: {bad:?}");
    }
}
