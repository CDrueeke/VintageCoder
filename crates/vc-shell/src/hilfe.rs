// SPDX-License-Identifier: GPL-3.0-or-later
//! Hilfe (E13): Themenliste, Suche und Nachschlagen aus `vc_ide::help` und `vc_ide_model::helpstate` (Handbuchkapitel eingebettet), Handbuchbilder aus dem Zwischenspeicher oder -
//! nur mit Zustimmung (Einstellung `help.images`) - aus dem Netz.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::path::Path;

pub fn themen() -> Value { vc_ide::help::topic_list() }
pub fn suchen(data: &Path, ziel: &str, q: &str) -> Value {
    json!(vc_ide_model::helpstate::search(data, ziel, q).into_iter().map(|r| json!({ "id": r.id, "title": r.title, "snippet": r.snippet })).collect::<Vec<_>>())
}
pub fn oeffnen(data: &Path, ziel: &str, lang: &str, key: &str) -> Value {
    match vc_ide_model::helpstate::open(data, ziel, lang, key) { Some(d) => json!({ "title": d.title, "markdown": d.markdown, "id": d.id }), None => Value::Null }
}

pub fn base64(b: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(b.len() * 4 / 3 + 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char); s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' }); s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}

/// Bild zu einer Kennung: im Zwischenspeicher/lokal vorhanden → `img`; sonst, wenn es aus dem Netz käme: `ask` (Zustimmung fehlt), `never` (abgelehnt) oder - mit `hole` und Zustimmung - geholt
pub fn bild(id: &str, hole: bool) -> Result<Value, Fehler> {
    use vc_ide::help_images as hi;
    if !hi::valid_id(id) { return Ok(json!({ "state": "invalid" })) }
    if let Some(d) = hi::cached(id) { return Ok(json!({ "state": "ok", "img": format!("data:image/png;base64,{}", base64(&d)) })) }
    let base = hi::base();
    if !hi::is_remote(&base) { return Ok(json!({ "state": "missing" })) }
    match hi::Consent::from_setting(&vc_ide::settings::get().help.images) {
        hi::Consent::Never => Ok(json!({ "state": "never" })),
        hi::Consent::Ask if !hole => Ok(json!({ "state": "ask" })),
        _ => { if !hole && !matches!(hi::Consent::from_setting(&vc_ide::settings::get().help.images), hi::Consent::Always) { return Ok(json!({ "state": "ask" })) }
            match hi::fetch(id) { Ok(d) => Ok(json!({ "state": "ok", "img": format!("data:image/png;base64,{}", base64(&d)) })), Err(e) => Ok(json!({ "state": "error", "error": e })) } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn themen_enthalten_handbuch() { let t = themen(); let a = t["topics"].as_array().unwrap(); assert!(a.len() > 10 && a.iter().any(|x| x["group"] == "Handbuch")); }
    #[test] fn base64_stimmt() { assert_eq!(base64(b"Man"), "TWFu"); assert_eq!(base64(b"Ma"), "TWE="); assert_eq!(base64(b"M"), "TQ=="); }
    #[test] fn ungueltige_bildkennung() { assert_eq!(bild("../x", false).unwrap()["state"], "invalid"); }
}
