// SPDX-License-Identifier: GPL-3.0-or-later
//! Erkennung der Systemsprache für „Automatisch“: macOS bevorzugte Sprachen, Windows Benutzer-Oberflächensprache, sonst `LC_ALL`, `LC_MESSAGES`, `LANGUAGE`, `LANG`.
//! Deutsch ergibt Deutsch; alles andere (auch unbekannt oder `C`) ergibt Englisch.
use crate::Lang;

/// Sprachteil einer Gebietsangabe: `de_DE.UTF-8` → `de`, `en-US` → `en`, `C`/`POSIX`/leer → `None`
pub fn parse_locale(s: &str) -> Option<String> {
    let t = s.trim();
    let base = t.split(['.', '@', '_', '-']).next().unwrap_or("").trim().to_ascii_lowercase();
    if base.is_empty() || base == "c" || base == "posix" { None } else { Some(base) }
}

fn lang_of(code: &str) -> Lang { if code == "de" { Lang::De } else { Lang::En } }

/// Wählt aus den Systemangaben die Sprache. `system` = bevorzugte Sprachen des Betriebssystems (macOS, Windows; zuerst die wichtigste),
/// `env` liefert Umgebungsvariablen. Die erste verwertbare Angabe entscheidet; ohne Angabe gilt Englisch.
pub fn detect_from(env: &dyn Fn(&str) -> Option<String>, system: &[String]) -> Lang {
    if let Some(c) = system.iter().find_map(|s| parse_locale(s)) { return lang_of(&c); }
    for var in ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"] {
        if let Some(v) = env(var) {
            if let Some(c) = v.split(':').find_map(parse_locale) { return lang_of(&c); }
        }
    }
    Lang::En
}

/// Erste Sprache aus der Ausgabe von `defaults read -g AppleLanguages` (`( "de-DE", "en-DE" )`)
pub(crate) fn first_quoted(text: &str) -> Option<String> {
    let a = text.find('"')? + 1;
    let b = text[a..].find('"')? + a;
    Some(text[a..b].to_string())
}

#[cfg(target_os = "macos")]
fn system_languages() -> Vec<String> {
    std::process::Command::new("defaults").args(["read", "-g", "AppleLanguages"]).output().ok()
        .and_then(|o| first_quoted(&String::from_utf8_lossy(&o.stdout))).into_iter().collect()
}

#[cfg(windows)]
fn system_languages() -> Vec<String> {
    extern "system" { fn GetUserDefaultUILanguage() -> u16; }
    // SAFETY: einfache Abfrage ohne Zeiger
    let id = unsafe { GetUserDefaultUILanguage() };
    vec![match id & 0x3ff { 0x07 => "de", 0x09 => "en", _ => "other" }.to_string()]
}

#[cfg(not(any(target_os = "macos", windows)))]
fn system_languages() -> Vec<String> { vec![] }

/// Systemsprache dieses Rechners
pub fn detect_system() -> Lang { detect_from(&|k| std::env::var(k).ok(), &system_languages()) }
