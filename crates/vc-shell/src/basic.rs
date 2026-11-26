// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC-Einstellungen des Projekts für die Oberfläche (Karte „BASIC“ in den Projekt-Einstellungen): Nummerierung, „kompakt“, Syntaxprüfung.
//! Das Fachliche steht in `vc_ide::projfile::BasicSettings` und `vc_basic::numbering`; hier wird nur gelesen, geprüft und geschrieben.
use serde_json::{json, Value};
use std::path::Path;
use vc_ide::projfile::{self, BasicSettings, ProjectFile};

/// Größte Zeilennummer, die der Wertebereich-Prüfung zugrunde liegt (Commodore; für Atari-Dialekte prüft der Bau mit 32767 noch einmal, `B191`)
const MAX_ZEILE: u32 = 63999;

/// Zustand für die Oberfläche: `present` (das Projekt hat BASIC-Quellen), Werte und die Meldung der Wertebereichsprüfung (leer = gültig)
pub fn zustand(root: &Path) -> Value {
    let s = projfile::basic_settings(root);
    let present = projfile::find(root).and_then(|f| ProjectFile::load(&f).ok()).map_or(false, |p| p.sources.iter().any(|x| x.kind == "basic"));
    json!({ "present": present, "start": s.numbering.start, "step": s.numbering.step, "compact": s.compact, "check": !s.check_off, "disabled": s.disabled, "maxLine": MAX_ZEILE,
            "error": s.numbering.validate(MAX_ZEILE, "", 0).err().map(|d| d.message.to_string()) })
}

/// Werte prüfen (`B191`) und in den Abschnitt `basic` der Projektdatei schreiben; übrige Felder der Projektdatei bleiben erhalten
pub fn setzen(root: &Path, a: &Value) -> Result<Value, String> {
    let n = projfile::Numbering::new(a["start"].as_u64().unwrap_or(10).min(u32::MAX as u64) as u32, a["step"].as_u64().unwrap_or(10).min(u32::MAX as u64) as u32);
    n.validate(MAX_ZEILE, "", 0).map_err(|d| d.message.to_string())?;
    let s = BasicSettings {
        numbering: n,
        compact: a["compact"].as_bool().unwrap_or(false),
        check_off: a["check"].as_bool() == Some(false),
        disabled: a["disabled"].as_array().map(|v| v.iter().filter_map(|c| c.as_str()).map(|c| c.trim().to_ascii_uppercase()).filter(|c| !c.is_empty()).collect()).unwrap_or_default(),
    };
    let f = projfile::find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let mut p = ProjectFile::load(&f)?;
    match s.to_json() { Value::Object(m) if m.is_empty() => { p.extra.remove("basic"); } v => { p.extra.insert("basic".into(), v); } }
    p.save(&f)?;
    Ok(zustand(root))
}
