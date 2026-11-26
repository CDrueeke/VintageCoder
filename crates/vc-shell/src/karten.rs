// SPDX-License-Identifier: GPL-3.0-or-later
//! Karten-Editor (E10): `BankDoc` (vc-ide-model) als Dokument im `DocStore`. Eine Bank-Datei (`.vcscreen` mit `tilesets` und `screens`) enthält mehrere Karten („Screens“, im Spiel Levels),
//! gebaut aus Kacheln des Kachelsatzes und freien Zeichen-Zellen darüber. Die Ansicht (`view`) liefert das Bild der gewählten Karte (PNG als Datenadresse, Rand und Zeichen in den Systemfarben),
//! Vorschaubilder aller Karten, das Kachelraster und die Zeichentafel; Absichten (`apply`) sind die Operationen des Modells. Ein Zug mit der Maus (`pointer` mit mehreren Punkten) ist ein Verlaufsschritt.
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use crate::kacheln::png_adresse;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use vc_ide_model::bankdoc::{grid_dims, BankDoc, Modus, Werkzeug};

pub struct KartenDok { pub doc: BankDoc, data: PathBuf, /** Wahl der Ansicht: project, pal, ntsc */ norm: String, projekt_norm: &'static str, /** Ebenen sichtbar: Kacheln, Zellen (Zeichen darüber) */ kachel_ebene: bool, zell_ebene: bool, /** Vorschaubilder aller Karten, neu berechnet nur wenn sich der Inhalt ändert (`stand`) */ vorschau: RefCell<(u64, usize, Vec<String>)> }

fn werkzeug_id(w: Werkzeug) -> &'static str {
    match w { Werkzeug::Stift => "pen", Werkzeug::Linie => "line", Werkzeug::Rechteck => "rect", Werkzeug::Fuellen => "fill", Werkzeug::Pipette => "pick", Werkzeug::Auswahl => "select", Werkzeug::Stempel => "stamp", Werkzeug::Text => "text", Werkzeug::Loeschen => "erase" }
}
fn werkzeug_aus(s: &str) -> Option<Werkzeug> {
    Some(match s { "pen" => Werkzeug::Stift, "line" => Werkzeug::Linie, "rect" => Werkzeug::Rechteck, "fill" => Werkzeug::Fuellen, "pick" => Werkzeug::Pipette, "select" => Werkzeug::Auswahl, "stamp" => Werkzeug::Stempel, "text" => Werkzeug::Text, "erase" => Werkzeug::Loeschen, _ => return None })
}

impl KartenDok {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<KartenDok, Fehler> {
        let doc = BankDoc::oeffnen(root, rel, data).map_err(|e| Fehler::mit("karten-unlesbar", json!({ "detail": e, "rel": rel })))?;
        let norm = crate::pixel::projektnorm(Some(root), &doc.target);
        Ok(KartenDok { doc, data: data.to_path_buf(), norm: "project".into(), projekt_norm: norm, kachel_ebene: true, zell_ebene: true, vorschau: RefCell::new((0, usize::MAX, vec![])) })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
    fn norm_aktiv(&self) -> &str { match self.norm.as_str() { "pal" => "pal", "ntsc" => "ntsc", _ => self.projekt_norm } }
    fn palette(&self) -> Option<vc_media::palette::TargetPalette> { vc_media::palette::palette_for(&self.data, &self.doc.target, self.norm_aktiv()).ok() }
}

impl Dokument for KartenDok {
    fn art(&self) -> &'static str { "bank" }
    fn view(&self) -> Value {
        let d = &self.doc; let pal = self.palette();
        let rgb = |i: usize| -> [u8; 3] { pal.as_ref().and_then(|p| p.rgb(i)).unwrap_or([128, 128, 128]) };
        let palette: Vec<Value> = pal.as_ref().map(|p| p.entries().into_iter().map(|(i, c)| json!({ "i": i, "color": format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]) })).collect()).unwrap_or_default();
        // Vorschaubilder der Karten: nur neu, wenn sich der Inhalt (oder die Zahl der Karten) geändert hat
        let stand = d.stand();
        { let mut c = self.vorschau.borrow_mut(); if c.0 != stand || c.1 != d.bank.screens.len() { c.2 = (0..d.bank.screens.len()).map(|i| { let b = d.bild_screen(i, 1, &rgb, 1.0, 1.0); png_adresse(b.w, b.h, &b.rgba) }).collect(); c.0 = stand; c.1 = d.bank.screens.len(); } }
        let vorschau = self.vorschau.borrow();
        let screens: Vec<Value> = d.bank.screens.iter().enumerate().map(|(i, s)| json!({ "i": i, "name": s.name, "w": s.w, "h": s.h, "img": vorschau.2.get(i).cloned().unwrap_or_default() })).collect();
        let (main, grid, cols_rows) = match (d.screen(), d.ts.as_ref()) {
            (Some(s), ts) => {
                let b = d.bild(2, &rgb, if self.kachel_ebene { 1.0 } else { 0.0 }, if self.zell_ebene { 1.0 } else { 0.0 });
                let (cols, rows) = ts.map_or((0, 0), |t| grid_dims(s, t));
                (json!({ "img": png_adresse(b.w, b.h, &b.rgba), "w": b.w, "h": b.h, "name": s.name, "cellsW": s.w, "cellsH": s.h, "origin": [s.origin.0, s.origin.1], "colors": s.colors, "patches": s.patches.len(), "grid": s.grid }), Some(s.set), (cols, rows))
            }
            _ => (Value::Null, None, (0, 0)),
        };
        let kacheln: Vec<Value> = d.ts.as_ref().map(|ts| (0..ts.tiles.len()).map(|i| { let b = d.kachel_bild(i, 2, &rgb); let t = &ts.tiles[i]; json!({ "i": i, "name": t.name, "col": t.col, "props": t.props, "img": png_adresse(b.w, b.h, &b.rgba) }) }).collect()).unwrap_or_default();
        let tafel = d.tafel_bild(2, &rgb); let n = if d.target == "atari8" { 128 } else { 256 };
        let (tw, th) = d.ts.as_ref().map_or((1, 1), |t| (t.tw, t.th));
        let ausgabe_bytes = d.media_text().len();
        json!({
            "rel": d.rel, "kind": "bank", "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |x| x.to_string_lossy().into_owned()), "target": d.target,
            "norm": self.norm, "projectNorm": self.projekt_norm, "layers": { "tiles": self.kachel_ebene, "cells": self.zell_ebene }, "cur": d.cur, "screens": screens, "map": main, "set": grid, "cols": cols_rows.0, "rows": cols_rows.1, "tw": tw, "th": th, "rand": d.rand(),
            "mode": if d.modus == Modus::Kachel { "tile" } else { "cell" }, "tool": werkzeug_id(d.werkzeug), "filled": d.gefuellt,
            "tools": d.werkzeuge().into_iter().map(|(w, l)| json!({ "id": werkzeug_id(w), "label": l })).collect::<Vec<_>>(),
            "tiles": kacheln, "brush": { "tile": d.pinsel, "recent": d.zuletzt, "cell": { "code": d.zell_pinsel.code, "cs": d.zell_pinsel.cs }, "color": d.farbe, "prop": d.eigenschaft }, "stamp": d.stempel.as_ref().map(|s| json!([s.first().map_or(0, |r| r.len()), s.len()])),
            "tafel": { "img": png_adresse(tafel.w, tafel.h, &tafel.rgba), "n": n, "w": tafel.w, "h": tafel.h }, "reiter": d.reiter,
            "charsets": d.ts.as_ref().map(|t| t.charsets.clone()).unwrap_or_default(), "colorMode": d.ts.as_ref().map_or("none", |t| match t.color_mode { vc_media::tiles::ColorMode::Tile => "tile", vc_media::tiles::ColorMode::Cell => "cell", vc_media::tiles::ColorMode::None => "none" }),
            "propDefs": d.ts.as_ref().map(|t| t.prop_defs.iter().map(|(n, b)| json!({ "name": n, "bit": b })).collect::<Vec<_>>()).unwrap_or_default(),
            "sets": d.saetze, "setRel": d.bank.tilesets.first(), "setError": d.ts_fehler, "palette": palette,
            "selection": d.auswahl.map(|((x0, y0), (x1, y1))| json!([x0, y0, x1, y1])), "textPos": [d.text_pos.0, d.text_pos.1],
            "ask": d.neue_kachel.as_ref().map(|n| json!({ "at": [n.at.0, n.at.1], "dup": n.dup, "aligned": n.ausgerichtet, "replace": n.ersetzen, "name": n.name })),
            "bankFiles": d.bank.tilesets.clone(), "outputBytes": ausgabe_bytes,
            "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "dirty": d.dirty(), "status": d.status,
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.status.clear();
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let zahl = |k: &str| op[k].as_i64().map(|x| x as i32).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let screens = self.doc.bank.screens.len();
        match op["kind"].as_str().unwrap_or("") {
            "screen" => { let i = n("i")?; if i >= screens { return Err(Fehler::neu("op-ungueltig")) } self.doc.screen_waehlen(i) }
            "screen_new" => self.doc.screen_neu(),
            "screen_dup" => self.doc.screen_duplizieren(),
            "screen_delete" => { if screens < 2 { return Err(Fehler::neu("op-ungueltig")) } self.doc.screen_loeschen() }
            "screen_move" => self.doc.screen_verschieben(zahl("d")?),
            "screen_order" => { let (von, nach) = (n("from")?, n("to")?); if von >= screens || nach >= screens { return Err(Fehler::neu("op-ungueltig")) } self.doc.screen_ordnen(von, nach) }
            "screen_rename" => { let nm = op["name"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "name" })))?; self.doc.screen_umbenennen(nm.trim(), op["first"].as_bool().unwrap_or(true)) }
            "screen_size" => self.doc.screen_groesse(n("w")?, n("h")?, n("ox")?, n("oy")?, op["first"].as_bool().unwrap_or(true)),
            "screen_color" => { let rolle = op["role"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "role" })))?; if !["bg", "border", "fg", "mc1", "mc2"].contains(&rolle) { return Err(Fehler::neu("op-ungueltig")) } self.doc.screen_farbe(rolle, n("i")?) }
            "set" => { let rel = op["rel"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "rel" })))?; if !self.doc.saetze.iter().any(|x| x == rel) { return Err(Fehler::neu("op-ungueltig")) } self.doc.satz_waehlen(rel) }
            "mode" => self.doc.modus_waehlen(match op["mode"].as_str() { Some("tile") => Modus::Kachel, Some("cell") => Modus::Zelle, _ => return Err(Fehler::neu("op-ungueltig")) }),
            "tool" => { let w = werkzeug_aus(op["tool"].as_str().unwrap_or("")).ok_or_else(|| Fehler::neu("op-ungueltig"))?; self.doc.werkzeug_waehlen(w) }
            "layers" => { if let Some(b) = op["tiles"].as_bool() { self.kachel_ebene = b } if let Some(b) = op["cells"].as_bool() { self.zell_ebene = b } }
            "norm" => { let v = op["norm"].as_str().unwrap_or(""); if !["project", "pal", "ntsc"].contains(&v) { return Err(Fehler::neu("op-ungueltig")) } self.norm = v.to_string() }
            "filled" => self.doc.gefuellt = op["on"].as_bool().unwrap_or(false),
            "tile" => { let i = n("i")?; if i >= self.doc.ts.as_ref().map_or(0, |t| t.tiles.len()) { return Err(Fehler::neu("op-ungueltig")) } self.doc.kachel_pinsel(i) }
            "char" => { let c = n("code")?; if c > 255 { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeichen_pinsel(c as u8) }
            "tab" => self.doc.reiter_waehlen(n("i")?),
            "brush_color" => self.doc.farbe = n("i")?,
            "bit7" => { let an = op["on"].as_bool().unwrap_or(false); self.doc.zell_pinsel.sel = an }
            "prop" => { self.doc.eigenschaft = match op["bit"].as_i64() { Some(b) if (0..=7).contains(&b) => Some(b as u8), _ => None } }
            // Zug mit der Maus: Punkte in Zellkoordinaten; der erste ist „gedrückt“, die weiteren „gezogen“, der letzte „losgelassen“ (ein Verlaufsschritt)
            "pointer" => {
                let pts: Vec<(i32, i32)> = op["pts"].as_array().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "pts" })))?.iter().filter_map(|p| Some((p[0].as_i64()? as i32, p[1].as_i64()? as i32))).collect();
                if pts.is_empty() { return Err(Fehler::neu("op-ungueltig")) }
                let rechts = op["right"].as_bool().unwrap_or(false); let (f0, l0) = (pts[0], pts[pts.len() - 1]);
                let (h, na) = (self.doc.einheit_bei(f0.0, f0.1, false), self.doc.einheit_bei(f0.0, f0.1, true));
                // Rechte Maustaste mit Stift oder Löschen: löscht (Zug über mehrere Felder wie mit dem Löschen-Werkzeug); sonst bleibt sie die Pipette
                if rechts && matches!(self.doc.werkzeug, Werkzeug::Stift | Werkzeug::Loeschen) {
                    let alt = self.doc.werkzeug; self.doc.werkzeug = Werkzeug::Loeschen;
                    self.doc.maus_unten(h, na, false);
                    for p in &pts[1..] { let nn = self.doc.einheit_bei(p.0, p.1, true); self.doc.maus_zug(nn) }
                    let nl = self.doc.einheit_bei(l0.0, l0.1, true); self.doc.maus_los(nl); self.doc.werkzeug = alt;
                } else {
                self.doc.maus_unten(h, na, rechts);
                if !rechts { for p in &pts[1..] { let nn = self.doc.einheit_bei(p.0, p.1, true); self.doc.maus_zug(nn) } let nl = self.doc.einheit_bei(l0.0, l0.1, true); self.doc.maus_los(nl) }
                }
            }
            "text" => { let t = op["text"].as_str().unwrap_or(""); self.doc.tippe(t) }
            "text_back" => self.doc.text_zurueck(),
            "text_newline" => self.doc.text_zeilenwechsel(),
            "text_at" => { let (x, y) = (n("x")?, n("y")?); self.doc.text_pos = (x, y) }
            "selection_copy" => self.doc.auswahl_kopieren(),
            "selection_clear" => self.doc.auswahl_leeren(),
            "selection_move" => { self.doc.auswahl_verschieben(zahl("dx")?, zahl("dy")?); }
            "selection_none" => self.doc.auswahl = None,
            "tile_from_selection" => { let (x, y) = (n("x")?, n("y")?); self.doc.neue_kachel_anfragen((x, y)) }
            "ask_name" => { if let Some(nk) = self.doc.neue_kachel.as_mut() { nk.name = op["name"].as_str().unwrap_or("").trim().to_string() } }
            "ask_replace" => { if let Some(nk) = self.doc.neue_kachel.as_mut() { nk.ersetzen = op["on"].as_bool().unwrap_or(false) && nk.ausgerichtet } }
            "ask_yes" => self.doc.neue_kachel_ok(),
            "ask_no" => self.doc.neue_kachel_abbrechen(),
            "undo" => { if !self.doc.kann_undo() { return Err(Fehler::neu("undo-leer")) } self.doc.rueckgaengig() }
            "redo" => { if !self.doc.kann_redo() { return Err(Fehler::neu("redo-leer")) } self.doc.wiederholen() }
            "save" => self.save()?,
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        Ok(self.view())
    }
    fn undo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "undo" })) }
    fn redo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "redo" })) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.speichern().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e }))) }
    fn undo_tiefe(&self) -> usize { if self.doc.kann_undo() { 1 } else { 0 } }
    fn export_quelle(&self) -> Option<ExportQuelle> {
        let rel = &self.doc.rel; let name = Path::new(rel).file_stem().map_or_else(|| "bank".into(), |s| s.to_string_lossy().into_owned());
        Some(ExportQuelle { rel: rel.clone(), kind: "screen".into(), name, media_text: self.doc.media_text() })
    }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str) -> (std::path::PathBuf, KartenDok) {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/c64/gfx_demo"); let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let d = std::env::temp_dir().join(format!("vc-karten-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d);
        fn kopiere(a: &Path, b: &Path) { std::fs::create_dir_all(b).unwrap(); for e in std::fs::read_dir(a).unwrap().flatten() { let p = e.path(); let z = b.join(e.file_name()); if p.is_dir() { kopiere(&p, &z) } else { std::fs::copy(&p, &z).unwrap(); } } }
        kopiere(&src, &d);
        let dok = KartenDok::oeffnen(&d, "graphics/level.vcscreen", &data).unwrap(); (d, dok)
    }
    #[test] fn oeffnet_die_bank_und_zeigt_karten_kacheln_und_bild() {
        let (d, k) = kopie("a"); let v = k.doc_view();
        assert_eq!(v["kind"], "bank"); assert!(v["screens"].as_array().unwrap().len() >= 1, "mindestens eine Karte"); assert!(v["tiles"].as_array().unwrap().len() >= 2, "Kacheln des Satzes");
        assert!(v["map"]["img"].as_str().unwrap().starts_with("data:image/png;base64,") && v["cols"].as_u64().unwrap() > 0 && v["tools"].as_array().unwrap().len() >= 5);
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn zug_ist_ein_verlaufsschritt_und_wird_gespeichert() {
        let (d, mut k) = kopie("b"); let v0 = k.doc_view(); let (cols, tw) = (v0["cols"].as_u64().unwrap() as i32, v0["tw"].as_i64().unwrap() as i32); assert!(cols >= 3 && tw >= 1);
        let o = (v0["map"]["origin"][0].as_i64().unwrap() as i32, v0["map"]["origin"][1].as_i64().unwrap() as i32);
        let vor = v0["map"]["grid"].to_string();
        k.apply(&json!({ "kind": "tile", "i": 1 })).unwrap();
        let v = k.apply(&json!({ "kind": "pointer", "pts": [[o.0, o.1], [o.0 + tw, o.1], [o.0 + 2 * tw, o.1]] })).unwrap();
        assert_ne!(v["map"]["grid"].to_string(), vor, "der Strich hat Kacheln gesetzt"); assert!(v["dirty"].as_bool().unwrap());
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["map"]["grid"].to_string(), vor, "ein Zug ist ein Rückgängig-Schritt");
        k.apply(&json!({ "kind": "redo" })).unwrap(); k.apply(&json!({ "kind": "save" })).unwrap(); assert!(!k.doc_view()["dirty"].as_bool().unwrap());
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn rechte_maustaste_loescht_mit_dem_stift() {
        let (d, mut k) = kopie("r"); let v0 = k.doc_view(); let o = (v0["map"]["origin"][0].as_i64().unwrap(), v0["map"]["origin"][1].as_i64().unwrap()); let tw = v0["tw"].as_i64().unwrap();
        k.apply(&json!({ "kind": "tile", "i": 1 })).unwrap();
        let v = k.apply(&json!({ "kind": "pointer", "pts": [[o.0, o.1], [o.0 + 2 * tw, o.1]] })).unwrap(); assert_eq!(v["map"]["grid"][0][0], 1); assert_eq!(v["map"]["grid"][0][2], 1);
        let v = k.apply(&json!({ "kind": "pointer", "right": true, "pts": [[o.0, o.1], [o.0 + 2 * tw, o.1]] })).unwrap();
        assert_eq!(v["map"]["grid"][0][0], 0, "rechter Zug löscht"); assert_eq!(v["map"]["grid"][0][2], 0); assert_eq!(v["tool"], "pen", "das Werkzeug bleibt der Stift");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn karten_neu_kopieren_ordnen_loeschen() {
        let (d, mut k) = kopie("c"); let n0 = k.doc_view()["screens"].as_array().unwrap().len();
        let v = k.apply(&json!({ "kind": "screen_new" })).unwrap(); assert_eq!(v["screens"].as_array().unwrap().len(), n0 + 1); assert_eq!(v["cur"].as_u64().unwrap() as usize, n0);
        let v = k.apply(&json!({ "kind": "screen_dup" })).unwrap(); assert_eq!(v["screens"].as_array().unwrap().len(), n0 + 2);
        let v = k.apply(&json!({ "kind": "screen_move", "d": -1 })).unwrap(); assert_eq!(v["cur"].as_u64().unwrap() as usize, n0);
        let namen = |v: &Value| v["screens"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap().to_string()).collect::<Vec<_>>(); let vorher = namen(&k.doc_view());
        let v = k.apply(&json!({ "kind": "screen_order", "from": n0, "to": 0 })).unwrap(); assert_eq!(v["cur"].as_u64().unwrap(), 0, "per Ziehen nach vorn: die Karte steht vorn und ist gewählt"); assert!(k.apply(&json!({ "kind": "screen_order", "from": 99, "to": 0 })).is_err());
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(namen(&v), vorher, "Rückgängig stellt die Reihenfolge wieder her"); k.apply(&json!({ "kind": "screen", "i": n0 })).unwrap();
        let v = k.apply(&json!({ "kind": "screen_delete" })).unwrap(); assert_eq!(v["screens"].as_array().unwrap().len(), n0 + 1);
        assert!(k.apply(&json!({ "kind": "screen", "i": 99 })).is_err() && k.apply(&json!({ "kind": "unbekannt" })).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
