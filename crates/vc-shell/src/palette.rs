// SPDX-License-Identifier: GPL-3.0-or-later
//! Palette-Editor (E7): `PaletteDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht (`view`) ist das Anzeige-DTO für Farbraster, Regler, Hexwert und „Im Einsatz“;
//! Absichten (`apply`): `sel`, `kanal`, `hex`, `fertig`, `standard`, `undo`, `redo`, `save`. Regler ziehen zählt als ein Verlaufsschritt (`fertig` beendet ihn).
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide_model::palettedoc::{hex_zu_rgb, rgb_zu_hex, PaletteDoc};

pub struct PaletteDok { pub doc: PaletteDoc, root: PathBuf }

/// Mediendateien des Projekts, die den Farbindex `i` in einem `colors`-Abschnitt verwenden: (Datei, Art)
fn im_einsatz(root: &Path, i: usize) -> Vec<(String, String)> {
    fn zahlen(v: &Value, i: u64, in_colors: bool) -> bool {
        match v {
            Value::Number(n) => in_colors && n.as_u64() == Some(i),
            Value::Array(a) => a.iter().any(|x| zahlen(x, i, in_colors)),
            Value::Object(o) => o.iter().any(|(k, x)| zahlen(x, i, in_colors || k == "colors")),
            _ => false,
        }
    }
    fn lauf(root: &Path, dir: &Path, i: usize, out: &mut Vec<(String, String)>, tiefe: usize) {
        if tiefe > 6 || out.len() >= 40 { return }
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut e: Vec<_> = rd.flatten().collect(); e.sort_by_key(|x| x.file_name());
        for x in e {
            let p = x.path(); let n = x.file_name().to_string_lossy().into_owned();
            if p.is_dir() { if !matches!(n.as_str(), "build" | ".git" | "node_modules" | "target") && !n.starts_with('.') { lauf(root, &p, i, out, tiefe + 1) } continue }
            let art = match p.extension().and_then(|e| e.to_str()) { Some("vcsprite") => "sprite", Some("vcpmg") => "pmg", Some("vcchar") => "charset", Some("vctiles") => "tiles", Some("vcscreen") => "screen", _ => continue };
            let Ok(t) = std::fs::read_to_string(&p) else { continue }; let Ok(v) = serde_json::from_str::<Value>(&t) else { continue };
            if zahlen(&v, i as u64, false) { out.push((p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/"), art.to_string())); }
        }
    }
    let mut out = vec![]; lauf(root, root, i, &mut out, 0); out
}

impl PaletteDok {
    pub fn oeffnen(root: &Path, rel: &str) -> Result<PaletteDok, Fehler> {
        let doc = PaletteDoc::open(root, rel).map_err(|e| Fehler::mit("palette-unlesbar", json!({ "detail": e, "rel": rel })))?;
        Ok(PaletteDok { doc, root: root.to_path_buf() })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
}

impl Dokument for PaletteDok {
    fn art(&self) -> &'static str { "palette" }
    fn view(&self) -> Value {
        let d = &self.doc; let n = d.anzahl(); let c = d.rgb(d.sel);
        let entries: Vec<String> = (0..n).map(|i| d.eintrag(i)).collect();
        let uses: Vec<Value> = im_einsatz(&self.root, d.sel).into_iter().map(|(f, a)| json!({ "file": f, "kind": a })).collect();
        json!({
            "rel": d.rel, "kind": "palette", "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |x| x.to_string_lossy().into_owned()),
            "name": d.media["name"].as_str().unwrap_or(""), "profile": d.media["profile"].as_str().unwrap_or(""),
            "count": n, "entries": entries, "sel": d.sel, "rgb": c, "hex": d.eintrag(d.sel), "uses": uses, "bytes": n * 3,
            "dirty": d.dirty(), "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "status": d.status,
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.status.clear();
        match op["kind"].as_str().unwrap_or("") {
            "sel" => { let i = op["i"].as_u64().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "i" })))? as usize; if i >= self.doc.anzahl() { return Err(Fehler::neu("op-ungueltig")) } self.doc.fertig(); self.doc.sel = i; }
            "kanal" => {
                let k = op["k"].as_u64().unwrap_or(9) as usize; let v = op["v"].as_u64().unwrap_or(256);
                if k > 2 || v > 255 { return Err(Fehler::neu("op-ungueltig")) } self.doc.kanal(k, v as u8);
            }
            "hex" => {
                let h = op["hex"].as_str().unwrap_or(""); let c = hex_zu_rgb(h).ok_or_else(|| Fehler::mit("palette-hex", json!({ "hex": h })))?;
                let i = self.doc.sel; if self.doc.rgb(i) != c { self.doc.fertig(); self.doc.setze(i, c); self.doc.fertig(); }
            }
            "fertig" => self.doc.fertig(),
            "standard" => self.doc.standard(),
            "undo" => { if !self.doc.kann_undo() { return Err(Fehler::neu("undo-leer")) } self.doc.undo() }
            "redo" => { if !self.doc.kann_redo() { return Err(Fehler::neu("redo-leer")) } self.doc.redo() }
            "save" => self.save()?,
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        let _ = rgb_zu_hex; Ok(self.view())
    }
    fn undo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "undo" })) }
    fn redo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "redo" })) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.save().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e }))) }
    fn undo_tiefe(&self) -> usize { if self.doc.kann_undo() { 1 } else { 0 } }
    fn export_quelle(&self) -> Option<ExportQuelle> {
        let rel = &self.doc.rel; let name = Path::new(rel).file_stem().map_or_else(|| "palette".into(), |s| s.to_string_lossy().into_owned());
        Some(ExportQuelle { rel: rel.clone(), kind: "palette".into(), name, media_text: self.doc.media_text() })
    }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str) -> (PathBuf, PaletteDok) {
        let w = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let d = std::env::temp_dir().join(format!("vc_pal_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap();
        std::fs::copy(w.join("examples/mega65/pal_demo/graphics/pal65.vcpal"), d.join("pal65.vcpal")).unwrap();
        let p = PaletteDok::oeffnen(&d, "pal65.vcpal").unwrap(); (d, p)
    }
    #[test] fn ansicht_hat_eintraege_auswahl_und_bytes() {
        let (d, p) = kopie("ansicht"); let v = p.view();
        assert_eq!(v["count"], 256); assert_eq!(v["entries"].as_array().unwrap().len(), 256); assert_eq!(v["sel"], 0); assert_eq!(v["bytes"], 768); assert_eq!(v["dirty"], false); assert_eq!(v["hex"], "#000000");
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn regler_ziehen_ist_ein_verlaufsschritt_hex_und_standard_gehen_und_ungueltiges_wird_abgelehnt() {
        let (d, mut p) = kopie("regler");
        p.apply(&json!({ "kind": "sel", "i": 5 })).unwrap();
        for v in [10, 20, 30] { p.apply(&json!({ "kind": "kanal", "k": 0, "v": v })).unwrap(); }
        let v = p.apply(&json!({ "kind": "fertig" })).unwrap(); assert_eq!(v["rgb"][0], 30); assert_eq!(v["dirty"], true);
        let v = p.apply(&json!({ "kind": "undo" })).unwrap(); assert_ne!(v["rgb"][0], 30, "drei Reglerstände = ein Undo-Schritt"); assert_eq!(v["canUndo"], false);
        let v = p.apply(&json!({ "kind": "redo" })).unwrap(); assert_eq!(v["rgb"][0], 30);
        let v = p.apply(&json!({ "kind": "hex", "hex": "#123456" })).unwrap(); assert_eq!(v["hex"], "#123456");
        assert_eq!(p.apply(&json!({ "kind": "hex", "hex": "zz" })).unwrap_err().code, "palette-hex");
        assert_eq!(p.apply(&json!({ "kind": "kanal", "k": 3, "v": 1 })).unwrap_err().code, "op-ungueltig"); assert_eq!(p.apply(&json!({ "kind": "sel", "i": 999 })).unwrap_err().code, "op-ungueltig");
        let v = p.apply(&json!({ "kind": "standard" })).unwrap(); assert_eq!(v["count"], 256);
        let v = p.apply(&json!({ "kind": "save" })).unwrap(); assert_eq!(v["dirty"], false); assert!(p.export_quelle().is_some());
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn im_einsatz_findet_dateien_mit_dem_farbindex() {
        let (d, p) = kopie("einsatz"); std::fs::write(d.join("a.vcsprite"), r#"{"colors":{"sprite":7,"mc1":2}}"#).unwrap(); std::fs::write(d.join("b.vcchar"), r#"{"colors":{"fg":1},"frames":[7]}"#).unwrap();
        let mut p = p; p.apply(&json!({ "kind": "sel", "i": 7 })).unwrap(); let v = p.view(); let u = v["uses"].as_array().unwrap();
        assert!(u.iter().any(|x| x["file"] == "a.vcsprite" && x["kind"] == "sprite")); assert!(!u.iter().any(|x| x["file"] == "b.vcchar"), "nur Werte unter colors zählen");
        std::fs::remove_dir_all(d).ok();
    }
}
