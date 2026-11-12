// SPDX-License-Identifier: GPL-3.0-or-later
//! Eingebettete Schriften (SIL Open Font License 1.1, siehe `assets/fonts/README.md`): Inter für die Oberfläche,
//! JetBrains Mono für Code, Adressen, Hex und Register. Die in egui eingebauten Schriften bleiben als Rückfall
//! für Zeichen, die diese Schriften nicht enthalten (Symbole, Emoji).
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

/// Familie für halbfette Oberfläche (Überschriften, Hervorhebungen)
pub fn ui_strong() -> FontFamily { FontFamily::Name("ui-strong".into()) }
/// Familie für halbfette Festbreitenschrift (Hervorhebungen in Hex- und Registeransichten)
pub fn mono_strong() -> FontFamily { FontFamily::Name("mono-strong".into()) }

pub fn install(ctx: &egui::Context) {
    let mut fd = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 4] = [
        ("Inter", include_bytes!("../assets/fonts/Inter-Regular.otf")),
        ("Inter-SemiBold", include_bytes!("../assets/fonts/Inter-SemiBold.otf")),
        ("JetBrainsMono", include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")),
        ("JetBrainsMono-Bold", include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf")),
    ];
    for (name, bytes) in faces { fd.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes))); }
    // Rückfall: die bisherigen Familien (egui-Standardschriften) folgen nach der neuen Schrift
    let prop = fd.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mono = fd.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let with = |first: &str, rest: &[String]| { let mut v = vec![first.to_string()]; v.extend(rest.iter().cloned()); v };
    fd.families.insert(FontFamily::Proportional, with("Inter", &prop));
    fd.families.insert(FontFamily::Monospace, with("JetBrainsMono", &mono));
    fd.families.insert(ui_strong(), with("Inter-SemiBold", &prop));
    fd.families.insert(mono_strong(), with("JetBrainsMono-Bold", &mono));
    ctx.set_fonts(fd);
}
