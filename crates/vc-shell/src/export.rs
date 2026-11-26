// SPDX-License-Identifier: GPL-3.0-or-later
//! Export-Dialog der Medien-Editoren (E3.4): Vorschau, Speichern ins Projekt (`build/export/`), Einbinden per `.include` bzw. `#include` und
//! „direkt in den Quelltext kopieren“ (ohne Include). Für BASIC auf Wunsch DATA-Zeilen samt Einleseroutine. Die Formate kommen aus `vc_media::project`,
//! die Vorschau aus `vc_ide_model::exportfacts`, die Zeile fürs Einbinden aus `vc_ide_model::dropinc`.
use crate::docstore::{ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::Path;
use vc_ide_model::exportfacts as ef;

pub const FORMATE: &[&str] = &["asm", "inc", "bin", "prg", "data"];

pub fn adresse(s: &str) -> Option<u16> { ef::parse_addr(s) }
fn braucht_adresse(fmt: &str) -> bool { matches!(fmt, "inc" | "prg") }

/// Grunddaten für den Dialog
pub fn info(q: &ExportQuelle) -> Value { json!({ "rel": q.rel, "kind": q.kind, "name": q.name, "addr": ef::default_addr(&q.kind), "formats": FORMATE }) }

/// Zeilen zum Einlesen der DATA-Zeilen in den Speicher (BASIC V2 kennt kein `RESTORE zeile`: die Routine steht deshalb vor den DATA-Zeilen und liest ab dem aktuellen DATA-Zeiger)
pub fn mit_routine(label: &str, addr: u16, bytes: usize, data: &str) -> String {
    let lab = label.trim(); let lab = if lab.is_empty() { "daten" } else { lab };
    format!("REM {}\n@{lab}_load:\nFOR I = 0 TO {} : READ A : POKE {} + I, A : NEXT : RETURN\n{data}", vc_i18n::t!("export-routine-rem", label = lab, addr = addr), bytes.saturating_sub(1), addr)
}

/// Vorschau: Dateien mit Größen, Text (bei BASIC DATA auf Wunsch mit Routine), Hex-Auszug oder Fehler
pub fn vorschau(root: &Path, data: &Path, q: &ExportQuelle, fmt: &str, name: &str, addr: &str, routine: bool) -> Value {
    if !FORMATE.contains(&fmt) { return json!({ "error": vc_i18n::t!("export-format-unbekannt").to_string() }); }
    let a = adresse(addr);
    if braucht_adresse(fmt) && a.is_none() { return json!({ "error": vc_i18n::t!("exportdlg-prg-braucht-eine-gueltige-ladeadresse-z").to_string(), "files": [] }); }
    let p = ef::preview_im_projekt(Some(root), data, &q.rel, &q.media_text, fmt, name, if braucht_adresse(fmt) { a } else { None });
    let mut text = p.text.clone();
    if fmt == "data" && routine {
        match (a, ef::preview_im_projekt(Some(root), data, &q.rel, &q.media_text, "bin", name, None).files.first().map(|f| f.1)) {
            (Some(ad), Some(n)) => text = mit_routine(name, ad, n, &p.text),
            _ => return json!({ "error": vc_i18n::t!("export-routine-braucht-adresse").to_string(), "files": p.files }),
        }
    }
    json!({ "files": p.files.iter().map(|f| json!([f.0, f.1])).collect::<Vec<_>>(), "text": text, "hex": p.hex, "error": p.error })
}

/// Text zum direkten Einfügen in den Quelltext (Assembler: der erzeugte Quelltext; BASIC: DATA-Zeilen, auf Wunsch mit Routine)
pub fn kopiertext(root: &Path, data: &Path, q: &ExportQuelle, ziel_basic: bool, name: &str, addr: &str, routine: bool) -> Result<String, Fehler> {
    let fehler = |e: String| Fehler::mit("export-fehler", json!({ "detail": e }));
    if !ziel_basic { return ef::volltext_im_projekt(Some(root), data, &q.rel, &q.media_text, "asm", name, None).map_err(fehler); }
    let t = ef::volltext_im_projekt(Some(root), data, &q.rel, &q.media_text, "data", name, None).map_err(fehler)?;
    if !routine { return Ok(t); }
    let ad = adresse(addr).ok_or_else(|| Fehler::neu("export-routine-braucht-adresse"))?;
    let n = ef::preview_im_projekt(Some(root), data, &q.rel, &q.media_text, "bin", name, None).files.first().map(|f| f.1).unwrap_or(0);
    Ok(mit_routine(name, ad, n, &t))
}

/// In das Projekt schreiben (nach `build/export/`); liefert die Dateipfade
pub fn speichern(root: &Path, data: &Path, rel: &str, fmt: &str, name: &str, addr: &str) -> Result<Vec<String>, Fehler> {
    if !FORMATE.contains(&fmt) { return Err(Fehler::neu("op-ungueltig")); }
    let a = adresse(addr);
    if braucht_adresse(fmt) && a.is_none() { return Err(Fehler::neu("export-adresse")); }
    let v = vc_media::project::export_mit(root, data, rel, fmt, Some(name).filter(|l| !l.is_empty()), if braucht_adresse(fmt) { a } else { None }, 10, 10, None).map_err(|e| Fehler::mit("export-fehler", json!({ "detail": e })))?;
    Ok(v["files"].as_array().map(|a| a.iter().filter_map(|f| f["path"].as_str().map(String::from)).collect()).unwrap_or_default())
}

/// Zeile zum Einbinden der Mediendatei in den Quelltext `ziel` (mit dessen aktuellem `text`); der Export wird vorher geschrieben
pub fn einbinden(root: &Path, data: &Path, rel: &str, ziel: &str, text: &str, name: &str) -> Result<(String, Vec<String>), Fehler> {
    let basic = !(ziel.to_ascii_lowercase().ends_with(".asm") || ziel.to_ascii_lowercase().ends_with(".inc") || ziel.to_ascii_lowercase().ends_with(".s"));
    // erst schreiben, dann die tatsächlich erzeugte Datei einbinden (ihr Name kommt aus dem Namen im Medium, nicht aus dem Dateinamen)
    let dateien = speichern(root, data, rel, if basic { "data" } else { "asm" }, name, "")?;
    let erzeugt = dateien.first().cloned().ok_or_else(|| Fehler::mit("export-fehler", json!({ "detail": vc_i18n::t!("exportdlg-der-export-hat-keine-datei-erzeugt").to_string() })))?;
    let plan = vc_ide_model::dropinc::plan(ziel, text, &erzeugt, Some(root)).map_err(|e| Fehler::mit("export-abgelehnt", json!({ "detail": e.to_string() })))?;
    Ok((plan.line, dateien))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quelle() -> (std::path::PathBuf, std::path::PathBuf, ExportQuelle) {
        let wurzel = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = std::env::temp_dir().join(format!("vc_export_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        std::fs::copy(wurzel.join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("graphics/hero.vcsprite")).unwrap();
        let text = std::fs::read_to_string(dir.join("graphics/hero.vcsprite")).unwrap();
        (dir, wurzel.join("data"), ExportQuelle { rel: "graphics/hero.vcsprite".into(), kind: "sprite".into(), name: "hero".into(), media_text: text })
    }
    #[test] fn vorschau_speichern_kopieren_und_routine() {
        let (dir, data, q) = quelle();
        let v = vorschau(&dir, &data, &q, "asm", "hero", "", false); assert!(v["error"].is_null(), "{v}"); assert!(v["text"].as_str().unwrap().contains("hero"));
        let v = vorschau(&dir, &data, &q, "prg", "hero", "", false); assert!(v["error"].is_string(), "PRG braucht eine Ladeadresse");
        let v = vorschau(&dir, &data, &q, "prg", "hero", "$2000", false); assert!(v["error"].is_null()); assert!(v["hex"].as_str().unwrap().starts_with("2000"));
        let d = vorschau(&dir, &data, &q, "data", "hero", "$2000", true); assert!(d["text"].as_str().unwrap().contains("GOSUB") || d["text"].as_str().unwrap().contains("READ A"), "{d}");
        assert!(d["text"].as_str().unwrap().contains("POKE 8192 + I"), "{d}");
        let ohne = vorschau(&dir, &data, &q, "data", "hero", "", true); assert!(ohne["error"].is_string(), "Routine braucht eine Adresse");
        let asm = kopiertext(&dir, &data, &q, false, "hero", "", false).unwrap(); assert!(asm.contains("hero") && !asm.contains(".include"), "ohne Include: {asm}");
        let bas = kopiertext(&dir, &data, &q, true, "hero", "$2000", true).unwrap(); assert!(bas.contains("DATA") && bas.contains("READ A"), "{bas}");
        let f = speichern(&dir, &data, &q.rel, "asm", "hero", "").unwrap(); assert!(f.iter().any(|p| p.ends_with(".asm")) && dir.join(&f[0]).is_file());
        let (zeile, _) = einbinden(&dir, &data, &q.rel, "main.asm", "", "hero").unwrap(); assert!(zeile.starts_with(".include") && zeile.contains("build/export"), "{zeile}");
        let (zeile, _) = einbinden(&dir, &data, &q.rel, "main.bas", "", "hero").unwrap(); assert!(zeile.starts_with("#include"), "{zeile}");
        assert_eq!(einbinden(&dir, &data, &q.rel, "daten.txt", "", "hero").unwrap_err().code, "export-abgelehnt");
        std::fs::remove_dir_all(dir).ok();
    }
}
