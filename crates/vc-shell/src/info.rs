// SPDX-License-Identifier: GPL-3.0-or-later
//! Info, Feedback und Update (E12.4): Version und Lizenz, Hinweise zu Drittanbietern, Feedback-Mail vorbereiten (gesendet wird erst im Mailprogramm), Versionsliste auf Wunsch abfragen
//! (oder nach der gewählten Häufigkeit), nichts laden oder installieren. Das Netz wird nur auf Wunsch oder mit Zustimmung angefasst; im Testlauf nur über `VC_UPDATE_URL` (lokale Datei).
use crate::docstore::Fehler;
use serde_json::{json, Value};
use vc_ide::update::{self, Evaluation, Version};

pub const WEBSITE: &str = "https://vintagecoder.drueeke.de";
pub const REPOSITORY: &str = "https://github.com/CDrueeke/VintageCoder";
pub const LIZENZ: &str = "GPL-3.0-or-later";
const HINWEISE: &str = include_str!("../../../THIRD-PARTY-NOTICES.md");

fn bewertung(ev: &Evaluation) -> Value {
    json!({ "latest": ev.latest.to_string(), "yanked": ev.installed_yanked, "update": ev.update.as_ref().map(|u| json!({ "version": u.version.to_string(), "url": u.url, "notes": u.notes, "formatWarning": u.project_format_warning, "published": u.published })) })
}
fn zustand_json(ev: Option<&Evaluation>) -> Value {
    let st = update::State::load(); let prefs = vc_ide::settings::get().updates;
    let badge = ev.filter(|e| update::badge_visible(e, &prefs, &st, update::now_secs())).and_then(|e| e.update.as_ref()).map(|u| u.version.to_string());
    json!({ "lastCheck": if st.last_check > 0 { Some(update::format_time(st.last_check)) } else { None }, "lastResult": st.last_result, "badge": badge, "eval": ev.map(bewertung), "enabled": vc_ide::features::features().update(), "source": update::source() })
}

pub fn ueber() -> Value {
    json!({ "name": "VintageCoder", "version": env!("CARGO_PKG_VERSION"), "license": LIZENZ, "website": WEBSITE, "repository": REPOSITORY, "notices": HINWEISE, "credits": danksagungen_anzeige(), "update": zustand_json(update::cached(Version::installed()).as_ref()) })
}

/// Danksagungen (`data/danksagungen.json`): Eintrag mit Kennung, Name, Lizenz und Links (nur https; die Texte je Kennung stehen im Katalog)
fn danksagungen_lesen() -> Vec<Value> {
    let Some(d) = crate::lsp::daten_ordner() else { return vec![] };
    let Ok(t) = std::fs::read_to_string(d.join("danksagungen.json")) else { return vec![] };
    serde_json::from_str::<Vec<Value>>(&t).unwrap_or_default()
}
fn nur_https(u: &str) -> bool { u.starts_with("https://") && !u.contains(char::is_whitespace) }
fn danksagungen_anzeige() -> Vec<Value> {
    danksagungen_lesen().iter().filter_map(|e| {
        let id = e["id"].as_str()?; let name = e["name"].as_str()?;
        let links: Vec<Value> = e["links"].as_array().map(|a| a.iter().filter_map(|l| l.as_str().filter(|u| nur_https(u)).map(|u| json!({ "url": u, "label": u.trim_start_matches("https://").trim_end_matches('/') }))).collect()).unwrap_or_default();
        Some(json!({ "id": id, "name": name, "license": e["lizenz"], "links": links }))
    }).collect()
}
/// Link einer Danksagung im Standardbrowser öffnen (nur auf Klick; nur Adressen aus der Liste)
pub fn danksagung_oeffnen(id: &str, index: usize) -> Result<(), Fehler> {
    let url = danksagungen_lesen().iter().find(|e| e["id"] == id).and_then(|e| e["links"].get(index)).and_then(|l| l.as_str()).filter(|u| nur_https(u)).map(String::from).ok_or_else(|| Fehler::neu("op-ungueltig"))?;
    if crate::bau::gesperrt() { return Ok(()) }
    let (prog, args) = vc_ide::feedback::open_command(std::env::consts::OS, &url);
    std::process::Command::new(prog).args(args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ()).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))
}
/// Stand der Update-Anzeige (Statusleiste): gemerkte Liste, nicht übersprungen und nicht in der Ruhezeit
pub fn update_stand() -> Value { zustand_json(update::cached(Version::installed()).as_ref()) }

/// Versionsliste abfragen und das Ergebnis merken; `auto`: nur wenn die Häufigkeit es erlaubt (sonst nichts tun)
pub fn update_pruefen(auto: bool) -> Value {
    let st = update::State::load();
    if auto && !update::auto_check_allowed(&vc_ide::features::features(), &vc_ide::settings::get().updates, &st, update::now_secs()) { return update_stand() }
    let src = update::source(); let r = update::check(&src, Version::installed(), update::now_secs());
    let mut st = st; update::record(&r, &mut st);
    let mut v = zustand_json(r.result.as_ref().ok()); v["error"] = r.result.as_ref().err().map(|e| json!(e)).unwrap_or(Value::Null); v
}
pub fn update_spaeter() -> Value { let mut st = update::State::load(); st.snooze_until = update::now_secs() + update::SNOOZE_SECS; let _ = st.save(); update_stand() }
pub fn update_ueberspringen(version: &str) -> Result<Value, Fehler> {
    let mut s = vc_ide::settings::get(); s.updates.skipped = version.to_string(); vc_ide::settings::set(s).map_err(|e| Fehler::mit("einst-schreiben", json!({ "detail": e })))?; Ok(update_stand())
}

pub fn feedback(system: Option<&str>) -> Value {
    let m = vc_ide::feedback::current(system);
    json!({ "subject": m.subject, "body": m.body, "bodyFull": m.body_full, "url": m.url, "dropped": m.dropped, "tooLong": m.too_long, "clipboard": m.clipboard() })
}
pub fn mail_oeffnen(url: &str) -> Result<(), Fehler> {
    if crate::bau::gesperrt() { return Ok(()) }
    vc_ide::feedback::open_mailto(url).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e })))
}
/// Feste Adressen im Browser öffnen (Webseite, Quelltext, Releaseseite); nie eine Adresse aus Eingaben
pub fn adresse_oeffnen(welche: &str, version: Option<&str>) -> Result<(), Fehler> {
    let url = match welche { "web" => WEBSITE.to_string(), "repo" => REPOSITORY.to_string(), "release" => update::release_url(version.and_then(Version::parse)), _ => return Err(Fehler::neu("op-ungueltig")) };
    if crate::bau::gesperrt() { return Ok(()) }
    let (prog, args) = vc_ide::feedback::open_command(std::env::consts::OS, &url);
    std::process::Command::new(prog).args(args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ()).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ueber_nennt_version_lizenz_und_hinweise() { let v = ueber(); assert_eq!(v["license"], LIZENZ); assert!(v["version"].as_str().unwrap().contains('.')); assert!(v["notices"].as_str().unwrap().len() > 100); }
    #[test] fn danksagungen_nur_https_und_aus_der_liste() {
        assert!(nur_https("https://github.com/x") && !nur_https("http://raster.atari.org") && !nur_https("javascript:alert(1)") && !nur_https("https://a b"));
        assert!(danksagung_oeffnen("gibt-es-nicht", 0).is_err() && danksagung_oeffnen("rmt", 99).is_err());
    }
    #[test] fn feste_adressen_nur() { assert!(adresse_oeffnen("anderes", None).is_err()); assert!(adresse_oeffnen("web", None).is_ok()); }
}
