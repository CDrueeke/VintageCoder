// SPDX-License-Identifier: GPL-3.0-or-later
//! Mehrsprachigkeit für VintageCoder (Konzept: assets/docs/Konzept-Mehrsprachigkeit-i18n.md).
//!
//! * Kataloge im Format Fluent (`catalog/<sprache>/<bereich>.ftl`), in die Programmdatei eingebettet.
//! * **Deutsch ist Quelle und Rückfall**: fehlt ein Schlüssel im Englischen, gilt der deutsche Text; fehlt er auch dort, erscheint ein sichtbarer
//!   Hinweis statt eines Absturzes (`⟦schlüssel⟧` in Tests und Debug-Bau, sonst `{schlüssel}`) und ein einmaliger Eintrag über [`set_missing_hook`].
//! * Pseudo-Sprache `xx` (zur Laufzeit aus dem Deutschen erzeugt): verfremdet und verlängert jeden Text, damit feste Texte im Code auffallen.
//! * Aufruf: `t!("status-files", n = 3)`; Mehrzahl und Parameter regelt Fluent.
//! * Maschinenlesbares (Codes, Dateiformate, JSON-Schlüssel, Direktiven) wird nie übersetzt.
mod detect;
mod overlay;
mod pseudo;

pub use overlay::{localize_json, localize_json_in};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use unic_langid::LanguageIdentifier;

pub use detect::{detect_from, detect_system, parse_locale};

/// Sprachen der Oberfläche. `Xx` ist die Pseudo-Sprache (nur zum Prüfen).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang { De, En, Xx }

impl Lang {
    pub fn code(self) -> &'static str { match self { Lang::De => "de", Lang::En => "en", Lang::Xx => "xx" } }
    pub fn from_code(s: &str) -> Option<Lang> {
        match s.trim().to_ascii_lowercase().as_str() { "de" => Some(Lang::De), "en" => Some(Lang::En), "xx" => Some(Lang::Xx), _ => None }
    }
    /// Eigenname der Sprache in ihrer eigenen Sprache (für die Auswahl)
    pub fn native_name(self) -> &'static str { match self { Lang::De => "Deutsch", Lang::En => "English", Lang::Xx => "Pseudo (xx)" } }
}

/// Wahl in den Einstellungen
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Choice { #[default] Auto, De, En }

impl Choice {
    pub fn code(self) -> &'static str { match self { Choice::Auto => "auto", Choice::De => "de", Choice::En => "en" } }
    pub fn from_code(s: &str) -> Choice { match s.trim().to_ascii_lowercase().as_str() { "de" => Choice::De, "en" => Choice::En, _ => Choice::Auto } }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

fn to_u8(l: Lang) -> u8 { match l { Lang::De => 0, Lang::En => 1, Lang::Xx => 2 } }
fn from_u8(v: u8) -> Lang { match v { 1 => Lang::En, 2 => Lang::Xx, _ => Lang::De } }

thread_local! { static LOCAL: std::cell::Cell<Option<Lang>> = const { std::cell::Cell::new(None) }; }

/// Sprache nur für den aufrufenden Thread (für Tests: parallele Tests stören einander so nicht); `None` hebt sie auf
pub fn set_thread_language(l: Option<Lang>) { LOCAL.with(|c| c.set(l)); }

/// Aktuelle Sprache: die des Threads (nur Tests), sonst die globale (threadsicher)
pub fn language() -> Lang { LOCAL.with(|c| c.get()).unwrap_or_else(|| from_u8(CURRENT.load(Ordering::Relaxed))) }
pub fn set_language(l: Lang) { CURRENT.store(to_u8(l), Ordering::Relaxed); }

/// Erkannte Systemsprache (nur Deutsch oder Englisch; alles Fremde ergibt Englisch)
pub fn system_language() -> Lang { detect_system() }

/// Aus der Wahl der Einstellungen die tatsächliche Sprache. `VC_LANG` (de, en, xx) überstimmt alles.
pub fn resolve(choice: Choice) -> Lang {
    if let Some(l) = std::env::var("VC_LANG").ok().and_then(|v| Lang::from_code(&v)) { return l; }
    match choice { Choice::De => Lang::De, Choice::En => Lang::En, Choice::Auto => system_language() }
}

/// Sprache nach der Wahl setzen und zurückgeben
pub fn apply(choice: Choice) -> Lang { let l = resolve(choice); set_language(l); l }

// ------------------------------------------------------------------------------------------------------------------ Kataloge

// (Bereich, deutscher Katalog, englischer Katalog) – eingebettet; erzeugt von build.rs aus `catalog/de/*.ftl` (zu jeder Datei muss es die englische geben)
include!(concat!(env!("OUT_DIR"), "/catalogs.rs"));

type Bundle = FluentBundle<FluentResource>;

fn build(lang: Lang) -> Bundle {
    let id: LanguageIdentifier = match lang { Lang::En => "en", _ => "de" }.parse().unwrap_or_default();
    let mut b = FluentBundle::new_concurrent(vec![id]);
    b.set_use_isolating(false);
    for (area, de, en) in CATALOGS {
        let src = if lang == Lang::En { en } else { de };
        match FluentResource::try_new((*src).to_string()) {
            Ok(r) => { if let Err(e) = b.add_resource(r) { debug_assert!(false, "Katalog {area}/{}: doppelte Schlüssel: {e:?}", lang.code()); } }
            Err((r, e)) => { debug_assert!(false, "Katalog {area}/{}: Syntaxfehler: {e:?}", lang.code()); let _ = b.add_resource(r); }
        }
    }
    b
}

fn bundles() -> &'static (Bundle, Bundle) {
    static B: OnceLock<(Bundle, Bundle)> = OnceLock::new();
    B.get_or_init(|| (build(Lang::De), build(Lang::En)))
}

// ------------------------------------------------------------------------------------------------------------------ Argumente

/// Wert eines benannten Parameters
#[derive(Clone, Debug, PartialEq)]
pub enum Arg { Num(f64), Text(String) }

macro_rules! arg_from_num { ($($t:ty),*) => { $(impl From<$t> for Arg { fn from(v: $t) -> Arg { Arg::Num(v as f64) } })* } }
arg_from_num!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);
impl From<&str> for Arg { fn from(v: &str) -> Arg { Arg::Text(v.to_string()) } }
impl From<String> for Arg { fn from(v: String) -> Arg { Arg::Text(v) } }
/// Verweise auf übergebbare Werte (`&String`, `&&str`, `&i32` …): so verschiebt ein Aufruf `t!("k", n = &x.name)` nichts aus dem Besitzer
impl<T: Clone + Into<Arg>> From<&T> for Arg { fn from(v: &T) -> Arg { v.clone().into() } }
impl From<&std::io::Error> for Arg { fn from(v: &std::io::Error) -> Arg { Arg::Text(v.to_string()) } }
impl From<char> for Arg { fn from(v: char) -> Arg { Arg::Text(v.to_string()) } }
impl From<std::borrow::Cow<'_, str>> for Arg { fn from(v: std::borrow::Cow<'_, str>) -> Arg { Arg::Text(v.into_owned()) } }
impl From<std::path::Display<'_>> for Arg { fn from(v: std::path::Display<'_>) -> Arg { Arg::Text(v.to_string()) } }

// ------------------------------------------------------------------------------------------------------------------ Übersetzen

type MissingHook = fn(&str);
static MISSING_HOOK: Mutex<Option<MissingHook>> = Mutex::new(None);
static MISSING_SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Wird einmal je fehlendem Schlüssel aufgerufen (die Oberfläche trägt ihn ins Protokoll ein); ohne Hook geht die Meldung auf die Fehlerausgabe
pub fn set_missing_hook(h: MissingHook) { if let Ok(mut g) = MISSING_HOOK.lock() { *g = Some(h); } }

fn report_missing(key: &str) {
    let first = MISSING_SEEN.lock().map(|mut g| g.get_or_insert_with(HashSet::new).insert(key.to_string())).unwrap_or(false);
    if !first { return; }
    let hook = MISSING_HOOK.lock().ok().and_then(|g| *g);
    match hook { Some(h) => h(key), None => eprintln!("i18n: Schlüssel fehlt: {key}") }
}

/// Sichtbarer Ersatz für einen unbekannten Schlüssel
fn missing_text(key: &str) -> String { if cfg!(debug_assertions) { format!("⟦{key}⟧") } else { format!("{{{key}}}") } }

fn format_in(b: &Bundle, key: &str, args: &FluentArgs) -> Option<String> {
    let msg = b.get_message(key)?;
    let pat = msg.value()?;
    let mut errs = vec![];
    let s = b.format_pattern(pat, Some(args), &mut errs).into_owned();
    debug_assert!(errs.is_empty(), "Schlüssel {key}: Fluent-Fehler {errs:?}");
    Some(s)
}

/// Text zu `key` in der aktuellen Sprache; siehe [`translate_in`]
pub fn translate(key: &str, args: &[(&str, Arg)]) -> String { translate_in(language(), key, args) }

/// Text zu `key` in `lang`: gewählte Sprache → Deutsch → sichtbarer Hinweis (nie ein Absturz). Die Pseudo-Sprache verfremdet den deutschen Text.
pub fn translate_in(lang: Lang, key: &str, args: &[(&str, Arg)]) -> String {
    let (de, en) = bundles();
    let pseudo = lang == Lang::Xx;
    let mut fa = FluentArgs::new();
    for (k, v) in args {
        match v {
            Arg::Num(n) => fa.set(*k, FluentValue::from(*n)),
            Arg::Text(t) => fa.set(*k, FluentValue::from(if pseudo { pseudo::guard(t) } else { t.clone() })),
        }
    }
    let found = if lang == Lang::En { format_in(en, key, &fa).or_else(|| format_in(de, key, &fa)) } else { format_in(de, key, &fa) };
    match found {
        Some(s) => if pseudo { pseudo::transform(&s) } else { s },
        None => { report_missing(key); missing_text(key) }
    }
}

/// `t!("schlüssel")` oder `t!("schlüssel", name = wert, n = 3)`: Text in der aktuellen Sprache
#[macro_export]
macro_rules! t {
    ($key:expr) => { $crate::translate($key, &[]) };
    ($key:expr, $($name:ident = $val:expr),+ $(,)?) => { $crate::translate($key, &[$((stringify!($name), $crate::Arg::from($val))),+]) };
}

/// Gibt es den Schlüssel im deutschen Katalog? (für Prüfungen und Tests)
pub fn has_key(key: &str) -> bool { bundles().0.has_message(key) }

/// Gibt es den Schlüssel im Katalog der Sprache `lang`? (`Xx` zählt wie Deutsch)
pub fn has_key_in(lang: Lang, key: &str) -> bool { let (de, en) = bundles(); if lang == Lang::En { en.has_message(key) } else { de.has_message(key) } }

// ------------------------------------------------------------------------------------------------------------------ Zahlen

/// Zahl mit `stellen` Nachkommastellen im Format der aktuellen Sprache (Deutsch: Komma, sonst Punkt; keine Tausendertrennung).
/// Hex und Adressen bleiben unverändert und gehören nicht hierher.
pub fn num(x: f64, stellen: usize) -> String { num_in(language(), x, stellen) }

pub fn num_in(lang: Lang, x: f64, stellen: usize) -> String {
    let s = format!("{x:.stellen$}");
    if lang == Lang::En { s } else { s.replace('.', ",") }
}

#[cfg(test)]
mod tests;
