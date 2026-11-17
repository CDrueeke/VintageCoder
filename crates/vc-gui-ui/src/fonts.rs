// SPDX-License-Identifier: GPL-3.0-or-later
//! Eingebettete Schriften (Entscheidung 18): Inter für die Oberfläche, JetBrains Mono für Code, Adressen, Hex und Register.
//! Beide unter der SIL Open Font License 1.1 (Lizenztexte neben den Dateien, Weitergabe mit dem Paket).
use iced::font::{Family, Weight};
use iced::Font;

pub const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.otf");
pub const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.otf");
pub const JETBRAINS_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
pub const JETBRAINS_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");

/// Alle Schriftdateien zum Laden beim Start
pub fn alle() -> [&'static [u8]; 4] { [INTER_REGULAR, INTER_SEMIBOLD, JETBRAINS_REGULAR, JETBRAINS_BOLD] }

/// Oberfläche
pub const UI: Font = Font { family: Family::Name("Inter"), ..Font::DEFAULT };
pub const UI_FETT: Font = Font { family: Family::Name("Inter"), weight: Weight::Semibold, ..Font::DEFAULT };
/// Code, Adressen, Hex, Register
pub const MONO: Font = Font { family: Family::Name("JetBrains Mono"), ..Font::DEFAULT };
pub const MONO_FETT: Font = Font { family: Family::Name("JetBrains Mono"), weight: Weight::Bold, ..Font::DEFAULT };

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn schriften_sind_eingebettet_und_haben_die_erwarteten_namen() {
        for (b, name) in [(INTER_REGULAR, "Inter"), (INTER_SEMIBOLD, "Inter"), (JETBRAINS_REGULAR, "JetBrains Mono"), (JETBRAINS_BOLD, "JetBrains Mono")] {
            assert!(b.len() > 50_000, "{name}: Datei zu klein");
            let utf16: Vec<u8> = name.encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
            assert!(b.windows(utf16.len()).any(|w| w == utf16.as_slice()), "{name} steht nicht in der Namenstabelle");
            assert!(b.starts_with(b"OTTO") || b.starts_with(&[0, 1, 0, 0]), "{name}: keine OpenType-Datei");
        }
    }
}
