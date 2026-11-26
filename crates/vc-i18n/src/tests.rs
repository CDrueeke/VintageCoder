// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;

fn env_of(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> { move |k| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| v.to_string()) }

#[test]
fn german_is_the_source_and_english_translates() {
    assert_eq!(translate_in(Lang::De, "menu-file", &[]), "Datei");
    assert_eq!(translate_in(Lang::En, "menu-file", &[]), "File");
}

#[test]
fn parameters_and_plurals() {
    let n = |n: i32, d: i32| [("n", Arg::from(n)), ("dirs", Arg::from(d))];
    assert_eq!(translate_in(Lang::De, "status-tree-summary", &n(1, 0)), "1 Datei(en) · 0 Ordner");
    assert_eq!(translate_in(Lang::En, "status-tree-summary", &n(1, 1)), "1 file · 1 folder");
    assert_eq!(translate_in(Lang::En, "status-tree-summary", &n(3, 0)), "3 files · 0 folders");
    assert_eq!(translate_in(Lang::En, "build-failed", &[("n", Arg::from(1))]), "Build failed: 1 problem");
    assert_eq!(translate_in(Lang::En, "build-failed", &[("n", Arg::from(5))]), "Build failed: 5 problems");
    assert_eq!(translate_in(Lang::De, "settings-language-auto", &[("lang", Arg::from("Deutsch"))]), "Automatisch (erkannt: Deutsch)");
}

#[test]
fn macro_takes_named_parameters_and_follows_the_thread_language() {
    set_thread_language(Some(Lang::En));
    assert_eq!(t!("build-failed", n = 2), "Build failed: 2 problems");
    assert_eq!(t!("menu-help"), "Help");
    set_thread_language(Some(Lang::De));
    assert_eq!(t!("menu-help"), "Hilfe");
    set_thread_language(None);
}

#[test]
fn unknown_key_never_crashes_and_is_visible() {
    for l in [Lang::De, Lang::En, Lang::Xx] {
        let s = translate_in(l, "gibt-es-nicht", &[]);
        assert!(s.contains("gibt-es-nicht"), "{s}");
    }
    assert!(translate_in(Lang::En, "gibt-es-nicht", &[]).starts_with(if cfg!(debug_assertions) { "⟦" } else { "{" }));
}

#[test]
fn missing_key_is_reported_once() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    fn hook(k: &str) { if k == "nur-einmal-melden" { N.fetch_add(1, Ordering::SeqCst); } }
    set_missing_hook(hook);
    for _ in 0..3 { let _ = translate_in(Lang::En, "nur-einmal-melden", &[]); }
    assert_eq!(N.load(Ordering::SeqCst), 1);
}

#[test]
fn pseudo_language_alters_and_lengthens_text_but_keeps_parameters() {
    let s = translate_in(Lang::Xx, "menu-file", &[]);
    assert!(s.starts_with('[') && s.ends_with(']') && s != "Datei" && s.contains('~'), "{s}");
    assert!(s.chars().count() > "Datei".chars().count() * 140 / 100, "{s}");
    let t = translate_in(Lang::Xx, "settings-language-auto", &[("lang", Arg::from("Deutsch"))]);
    assert!(t.contains("Deutsch"), "Parameterwert bleibt lesbar: {t}");
    assert!(!t.contains("Automatisch"), "{t}");
    // Zahlen bleiben Ziffern
    assert!(translate_in(Lang::Xx, "status-tree-summary", &[("n", Arg::from(3)), ("dirs", Arg::from(2))]).contains('3'));
}

#[test]
fn locale_parsing() {
    assert_eq!(parse_locale("de_DE.UTF-8").as_deref(), Some("de"));
    assert_eq!(parse_locale("en-US").as_deref(), Some("en"));
    assert_eq!(parse_locale("fr_FR@euro").as_deref(), Some("fr"));
    for s in ["", "C", "POSIX", "C.UTF-8"] { assert_eq!(parse_locale(s), None, "{s}"); }
}

#[test]
fn system_language_detection_from_environment() {
    let none: Vec<String> = vec![];
    assert_eq!(detect_from(&env_of(&[("LANG", "de_DE.UTF-8")]), &none), Lang::De);
    assert_eq!(detect_from(&env_of(&[("LANG", "en_GB.UTF-8")]), &none), Lang::En);
    assert_eq!(detect_from(&env_of(&[("LANG", "fr_FR.UTF-8")]), &none), Lang::En, "fremde Sprache ergibt Englisch");
    assert_eq!(detect_from(&env_of(&[]), &none), Lang::En, "ohne Angabe Englisch");
    assert_eq!(detect_from(&env_of(&[("LANG", "C")]), &none), Lang::En);
    // Vorrang: LC_ALL vor LC_MESSAGES vor LANGUAGE vor LANG
    assert_eq!(detect_from(&env_of(&[("LC_ALL", "de_DE"), ("LANG", "en_US")]), &none), Lang::De);
    assert_eq!(detect_from(&env_of(&[("LC_ALL", "C"), ("LANG", "de_DE")]), &none), Lang::De, "C zählt nicht");
    assert_eq!(detect_from(&env_of(&[("LANGUAGE", "de:en")]), &none), Lang::De);
    // Systemliste (macOS/Windows) geht vor der Umgebung
    assert_eq!(detect_from(&env_of(&[("LANG", "en_US")]), &["de-DE".to_string()]), Lang::De);
    assert_eq!(detect_from(&env_of(&[("LANG", "de_DE")]), &["en-DE".to_string()]), Lang::En);
}

#[test]
fn first_quoted_reads_the_macos_list() {
    assert_eq!(detect::first_quoted("(\n    \"de-DE\",\n    \"en-DE\"\n)\n").as_deref(), Some("de-DE"));
    assert_eq!(detect::first_quoted("()"), None);
}

#[test]
fn vc_lang_overrides_the_choice() {
    std::env::set_var("VC_LANG", "xx");
    assert_eq!(resolve(Choice::De), Lang::Xx);
    std::env::set_var("VC_LANG", "en");
    assert_eq!(resolve(Choice::De), Lang::En);
    assert_eq!(resolve(Choice::Auto), Lang::En);
    std::env::set_var("VC_LANG", "kauderwelsch");
    assert_eq!(resolve(Choice::De), Lang::De, "unbekannter Wert wird ignoriert");
    std::env::remove_var("VC_LANG");
    assert_eq!(resolve(Choice::En), Lang::En);
}

#[test]
fn number_format_per_language() {
    assert_eq!(num_in(Lang::De, 1.23456, 2), "1,23");
    assert_eq!(num_in(Lang::En, 1.23456, 2), "1.23");
    assert_eq!(num_in(Lang::De, 42.0, 0), "42");
    assert_eq!(num_in(Lang::Xx, 0.5, 1), "0,5");
}

#[test]
fn choice_and_lang_codes_round_trip() {
    for c in [Choice::Auto, Choice::De, Choice::En] { assert_eq!(Choice::from_code(c.code()), c); }
    assert_eq!(Choice::from_code("klingonisch"), Choice::Auto);
    for l in [Lang::De, Lang::En, Lang::Xx] { assert_eq!(Lang::from_code(l.code()), Some(l)); }
    assert_eq!(Lang::from_code("fr"), None);
}

#[test]
fn every_catalog_parses_without_errors() {
    // gleiche Schlüssel und Parameter in de und en prüft tools/check_i18n.py
    for (area, de, en) in CATALOGS {
        for (l, src) in [("de", de), ("en", en)] {
            assert!(FluentResource::try_new((*src).to_string()).is_ok(), "Bereich {area}/{l}: Syntaxfehler im Katalog");
        }
    }
}
