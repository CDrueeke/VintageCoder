// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungen ›BASIC · Zahlen‹: Farben, Schreibweise im erzeugten Programm, `_`-Trenner, `DATA`, Bereichsprüfung.
//! Sie liegen in `prefs.json` (Schlüssel `numbers`); die Werte des Zahlenpasses setzt `vc_basic::numbers::set_defaults` für Editor und Bauen.
use vc_basic::numbers::{defaults, set_defaults, RangeCheck};

/// Ausgabe im erzeugten Programm für Dialekte, die Literale selbst lesen (BASIC 65, Turbo-BASIC XL)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Output { Default, AsTyped, Decimal }

pub fn output() -> Output { match defaults().native { None => Output::Default, Some(true) => Output::AsTyped, Some(false) => Output::Decimal } }
pub fn set_output(o: Output) { let mut d = defaults(); d.native = match o { Output::Default => None, Output::AsTyped => Some(true), Output::Decimal => Some(false) }; set_defaults(d); }
/// BASIC-Zahlen in eigenen Farben (Hex, Binär, Dezimal); Einstellung ›BASIC · Zahlen‹, Vorgabe an
pub static NUM_COLORS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn colors() -> bool { NUM_COLORS.load(std::sync::atomic::Ordering::Relaxed) }
pub fn set_colors(on: bool) { NUM_COLORS.store(on, std::sync::atomic::Ordering::Relaxed); }
/// Eingabehilfe nach `$` und `%`: 0 aus, 1 Hinweis, 2 Hinweis mit Liste
static ASSIST: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(1);
pub fn assist() -> u8 { ASSIST.load(std::sync::atomic::Ordering::Relaxed) }
pub fn set_assist(v: u8) { ASSIST.store(v.min(2), std::sync::atomic::Ordering::Relaxed); }
/// Hexziffern beim Tippen großschreiben (`$d0` → `$D0`)
static UPPER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub fn upper_typing() -> bool { UPPER.load(std::sync::atomic::Ordering::Relaxed) }
pub fn set_upper_typing(on: bool) { UPPER.store(on, std::sync::atomic::Ordering::Relaxed); }
pub fn update(f: impl FnOnce(&mut vc_basic::numbers::NumberOptions)) { let mut d = defaults(); f(&mut d); set_defaults(d); }

pub fn to_json() -> serde_json::Value {
    let d = defaults();
    serde_json::json!({
        "colors": colors(),
        "assist": assist(),
        "upper": upper_typing(),
        "underscore": d.underscore,
        "data": d.data,
        "output": match output() { Output::Default => "default", Output::AsTyped => "typed", Output::Decimal => "decimal" },
        "ranges": match d.ranges { RangeCheck::Off => "off", RangeCheck::Warn => "warn", RangeCheck::Error => "error" },
    })
}

pub fn apply_json(v: &serde_json::Value) {
    if let Some(b) = v["colors"].as_bool() { set_colors(b); }
    if let Some(a) = v["assist"].as_u64() { set_assist(a as u8); }
    if let Some(b) = v["upper"].as_bool() { set_upper_typing(b); }
    let mut d = defaults();
    if let Some(b) = v["underscore"].as_bool() { d.underscore = b; }
    if let Some(b) = v["data"].as_bool() { d.data = b; }
    d.native = match v["output"].as_str() { Some("typed") => Some(true), Some("decimal") => Some(false), _ => None };
    d.ranges = match v["ranges"].as_str() { Some("off") => RangeCheck::Off, Some("error") => RangeCheck::Error, _ => RangeCheck::Warn };
    set_defaults(d);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_round_trip() {
        let before = to_json();
        apply_json(&serde_json::json!({"colors": false, "underscore": false, "data": false, "output": "default", "ranges": "off"}));
        let j = to_json();
        assert_eq!(j["colors"], false); assert_eq!(j["underscore"], false); assert_eq!(j["data"], false); assert_eq!(j["output"], "default"); assert_eq!(j["ranges"], "off");
        apply_json(&before);
        assert_eq!(to_json(), before);
    }
}
