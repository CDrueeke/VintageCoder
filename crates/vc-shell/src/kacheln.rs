// SPDX-License-Identifier: GPL-3.0-or-later
//! Kachel-Editor (E8): `TilesetDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht (`view`) liefert Kachelliste mit Vorschaubildern (PNG als Datenadresse), die große Kachel
//! mit Zellen, die Zeichentafel, Farben, Eigenschaften, Füllstand und Rückfragen; Absichten (`apply`) sind die Operationen des Modells. Ein Pinselzug (`paint` mit mehreren Zellen) ist ein Verlaufsschritt.
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::Path;
use vc_ide_model::tilesetdoc::{Frage, PinselModus, TilesetDoc};
use vc_media::tiles::ColorMode;

pub struct KachelDok { pub doc: TilesetDoc, data: std::path::PathBuf, norm: &'static str }

/// Bild als PNG in einer Datenadresse (`data:image/png;base64,…`)
pub fn png_adresse(w: usize, h: usize, rgba: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = vc_media::png::encode_rgba(w, h, rgba); let mut s = String::from("data:image/png;base64,");
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char); s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' }); s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}
fn farbmodus_name(m: ColorMode) -> &'static str { match m { ColorMode::Tile => "tile", ColorMode::Cell => "cell", ColorMode::None => "none" } }
fn pinselmodus_name(m: PinselModus) -> &'static str { match m { PinselModus::Zeichen => "char", PinselModus::Farbe => "color", PinselModus::Beides => "both" } }

impl KachelDok {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<KachelDok, Fehler> {
        let doc = TilesetDoc::oeffnen(root, rel, data).map_err(|e| Fehler::mit("kacheln-unlesbar", json!({ "detail": e, "rel": rel })))?;
        let norm = crate::pixel::projektnorm(Some(root), &doc.target);
        Ok(KachelDok { doc, data: data.to_path_buf(), norm })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
    fn palette(&self) -> Option<vc_media::palette::TargetPalette> { vc_media::palette::palette_for(&self.data, &self.doc.target, self.norm).ok() }
}

impl Dokument for KachelDok {
    fn art(&self) -> &'static str { "tileset" }
    fn view(&self) -> Value {
        let d = &self.doc; let ts = &d.ts; let pal = self.palette();
        let rgb = |i: usize| -> [u8; 3] { pal.as_ref().and_then(|p| p.rgb(i)).unwrap_or([128, 128, 128]) };
        let palette: Vec<Value> = pal.as_ref().map(|p| p.entries().into_iter().map(|(i, c)| json!({ "i": i, "color": format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]) })).collect()).unwrap_or_default();
        let thumbs: Vec<Value> = (0..ts.tiles.len()).map(|i| { let b = d.kachel_bild(i, 2, &rgb); let t = &ts.tiles[i];
            json!({ "i": i, "name": t.name, "col": t.col, "props": t.props, "uses": d.verwendung(i), "double": d.doppelt.get(i).copied().flatten(), "img": png_adresse(b.w, b.h, &b.rgba) }) }).collect();
        let (tile, cells) = if let Some(t) = ts.tiles.get(d.sel) {
            let b = d.kachel_bild(d.sel, 6, &rgb);
            (json!({ "img": png_adresse(b.w, b.h, &b.rgba), "w": b.w, "h": b.h, "name": t.name, "col": t.col, "props": t.props }), t.cells.iter().map(|c| json!({ "code": c.code, "cs": c.cs, "col": c.col, "sel": c.sel })).collect::<Vec<_>>())
        } else { (Value::Null, vec![]) };
        let tafel = d.tafel_bild(2, &rgb); let n = if d.atari() { 128 } else { 256 };
        let fill = d.fill.as_ref().map(|f| json!({ "used": f.used, "max": f.max, "doubles": f.doubles, "err": f.err }));
        let frage = match &d.frage { Some(Frage::Loeschen(i)) => json!({ "kind": "delete", "i": i, "uses": d.verwendung(*i) }), Some(Frage::Groesse(w, h)) => json!({ "kind": "size", "w": w, "h": h }), None => Value::Null };
        json!({
            "rel": d.rel, "kind": "tileset", "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |x| x.to_string_lossy().into_owned()), "name": ts.name, "target": d.target, "atari": d.atari(),
            "tw": ts.tw, "th": ts.th, "count": ts.tiles.len(), "sel": d.sel, "cur": d.cur, "tiles": thumbs, "tile": tile, "cells": cells,
            "tafel": { "img": png_adresse(tafel.w, tafel.h, &tafel.rgba), "n": n, "w": tafel.w, "h": tafel.h }, "reiter": d.reiter, "charsets": ts.charsets, "angebot": d.zeichensatz_angebot(),
            "usedCharset": (0..ts.charsets.len()).map(|i| d.zeichensatz_benutzt(i)).collect::<Vec<_>>(),
            "colorMode": farbmodus_name(ts.color_mode), "brush": { "code": d.pinsel.code, "cs": d.pinsel.cs, "sel": d.pinsel.sel, "mode": pinselmodus_name(d.wirksamer_modus()), "color": d.farbe },
            "bg": d.hintergrundfarbe(), "palette": palette, "propDefs": ts.prop_defs.iter().map(|(n, b)| json!({ "name": n, "bit": b })).collect::<Vec<_>>(),
            "fill": fill, "banks": d.banken.iter().map(|b| json!({ "rel": b.rel, "screens": b.screens })).collect::<Vec<_>>(), "ask": frage,
            "hasClip": d.zwischen.is_some(), "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "dirty": d.dirty(), "status": d.status,
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.status.clear();
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let zahl = |k: &str| op[k].as_i64().map(|x| x as i32).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let cells = self.doc.ts.tw * self.doc.ts.th;
        match op["kind"].as_str().unwrap_or("") {
            "tile" => { let i = n("i")?; if i >= self.doc.ts.tiles.len() { return Err(Fehler::neu("op-ungueltig")) } self.doc.kachel_waehlen(i) }
            "cell" => { let k = n("k")?; if k >= cells { return Err(Fehler::neu("op-ungueltig")) } self.doc.zelle_waehlen(k) }
            "cursor" => self.doc.cursor_bewegen(zahl("dx")?, zahl("dy")?),
            "paint" => {
                let ks: Vec<usize> = op["cells"].as_array().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "cells" })))?.iter().filter_map(|x| x.as_u64().map(|k| k as usize)).filter(|k| *k < cells).collect();
                if ks.is_empty() { return Err(Fehler::neu("op-ungueltig")) } for (j, k) in ks.iter().enumerate() { self.doc.malen(*k, j == 0) }
            }
            "pick" => { let k = n("k")?; if k >= cells { return Err(Fehler::neu("op-ungueltig")) } self.doc.aufnehmen(k) }
            "char" => { let c = n("code")?; if c > 255 { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeichen_waehlen(c as u8) }
            "brush_color" => self.doc.pinsel_farbe(n("i")?),
            "bit7" => self.doc.farbwahl_bit(op["on"].as_bool().unwrap_or(false)),
            "brush_mode" => self.doc.modus_waehlen(match op["mode"].as_str() { Some("char") => PinselModus::Zeichen, Some("color") => PinselModus::Farbe, Some("both") => PinselModus::Beides, _ => return Err(Fehler::neu("op-ungueltig")) }),
            "flip" => self.doc.spiegeln(match op["dir"].as_str() { Some("h") => true, Some("v") => false, _ => return Err(Fehler::neu("op-ungueltig")) }),
            "rotate" => self.doc.drehen(),
            "roll" => self.doc.rollen(zahl("dx")?, zahl("dy")?),
            "clear" => self.doc.leeren(),
            "copy" => self.doc.kopieren(),
            "paste" => { if !self.doc.zwischen.as_ref().is_some_and(|c| c.len() == cells) { return Err(Fehler::neu("kacheln-zwischenablage")) } self.doc.einfuegen() }
            "size" => { let (w, h) = (n("w")?, n("h")?); self.doc.groesse_anfragen(w, h) }
            "ask_yes" => self.doc.frage_bestaetigen().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e })))?,
            "ask_no" => self.doc.frage_abbrechen(),
            "tile_new" => self.doc.kachel_neu(),
            "tile_dup" => self.doc.kachel_duplizieren(),
            "tile_delete" => self.doc.kachel_loeschen(),
            "tile_merge" => self.doc.zusammenfuehren().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e })))?,
            "rename" => { let nm = op["name"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "name" })))?; self.doc.kachel_umbenennen(nm.trim(), op["first"].as_bool().unwrap_or(true)) }
            "color_mode" => self.doc.farbmodus(match op["mode"].as_str() { Some("tile") => ColorMode::Tile, Some("cell") => ColorMode::Cell, Some("none") => ColorMode::None, _ => return Err(Fehler::neu("op-ungueltig")) }),
            "tile_color" => self.doc.kachelfarbe(n("i")?),
            "bg" => self.doc.hintergrund(n("i")?),
            "prop" => { let b = n("bit")?; if b > 7 { return Err(Fehler::neu("op-ungueltig")) } self.doc.eigenschaft_setzen(b as u8, op["on"].as_bool().unwrap_or(false)) }
            "prop_new" => { let nm = op["name"].as_str().unwrap_or(""); if !self.doc.eigenschaft_neu(nm) { return Err(Fehler::mit("kacheln-eigenschaft", json!({ "name": nm }))) } }
            "charset_add" => { let rel = op["rel"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "rel" })))?; if !self.doc.zeichensatz_angebot().iter().any(|x| x == rel) { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeichensatz_hinzu(rel) }
            "charset_remove" => { let i = n("i")?; if i >= self.doc.ts.charsets.len() || self.doc.zeichensatz_benutzt(i) { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeichensatz_entfernen(i) }
            "tab" => self.doc.reiter_waehlen(n("i")?),
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
        let rel = &self.doc.rel; let name = Path::new(rel).file_stem().map_or_else(|| "tiles".into(), |s| s.to_string_lossy().into_owned());
        Some(ExportQuelle { rel: rel.clone(), kind: "tileset".into(), name, media_text: self.doc.media_text() })
    }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str) -> (std::path::PathBuf, KachelDok) {
        let w = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let d = std::env::temp_dir().join(format!("vc_kach_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("graphics")).unwrap();
        for f in ["font.vcchar", "level_tiles.vctiles", "level.vcscreen", "title.vcscreen"] { std::fs::copy(w.join("examples/c64/gfx_demo/graphics").join(f), d.join("graphics").join(f)).unwrap(); }
        let k = KachelDok::oeffnen(&d, "graphics/level_tiles.vctiles", &w.join("data")).unwrap(); (d, k)
    }
    #[test] fn ansicht_hat_kacheln_bilder_zellen_tafel_und_fuellstand() {
        let (d, k) = kopie("ansicht"); let v = k.view();
        assert_eq!(v["count"], 4); assert_eq!((v["tw"].clone(), v["th"].clone()), (json!(2), json!(2))); assert_eq!(v["cells"].as_array().unwrap().len(), 4);
        assert!(v["tiles"][0]["img"].as_str().unwrap().starts_with("data:image/png;base64,")); assert!(v["tile"]["img"].is_string()); assert_eq!(v["tafel"]["n"], 256); assert!(v["fill"]["used"].as_u64().unwrap() > 0);
        assert_eq!(v["colorMode"], "tile"); assert!(v["tiles"][1]["uses"].as_u64().unwrap() > 0, "Stein wird in der Bank benutzt"); assert_eq!(v["dirty"], false); assert!(!v["palette"].as_array().unwrap().is_empty());
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn pinselzug_ist_ein_schritt_und_kachel_operationen_gehen() {
        let (d, mut k) = kopie("ops");
        k.apply(&json!({ "kind": "tile", "i": 2 })).unwrap(); k.apply(&json!({ "kind": "char", "code": 7 })).unwrap();
        let v = k.apply(&json!({ "kind": "paint", "cells": [0, 1, 3] })).unwrap(); assert_eq!((v["cells"][0]["code"].clone(), v["cells"][1]["code"].clone(), v["cells"][3]["code"].clone()), (json!(7), json!(7), json!(7))); assert_eq!(v["dirty"], true);
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_ne!(v["cells"][1]["code"], 7, "Zug = ein Schritt (Zeichenwahl davor ist ein eigener)");
        k.apply(&json!({ "kind": "redo" })).unwrap(); k.apply(&json!({ "kind": "flip", "dir": "h" })).unwrap(); k.apply(&json!({ "kind": "rotate" })).unwrap(); k.apply(&json!({ "kind": "roll", "dx": 1, "dy": 0 })).unwrap();
        k.apply(&json!({ "kind": "copy" })).unwrap(); k.apply(&json!({ "kind": "clear" })).unwrap(); let v = k.apply(&json!({ "kind": "paste" })).unwrap(); assert!(v["hasClip"] == true && v["cells"].as_array().unwrap().iter().any(|c| c["code"] == 7));
        let v = k.apply(&json!({ "kind": "rename", "name": "wiese" })).unwrap(); assert_eq!(v["tile"]["name"], "wiese");
        k.apply(&json!({ "kind": "tile_color", "i": 5 })).unwrap(); k.apply(&json!({ "kind": "prop", "bit": 0, "on": true })).unwrap(); let v = k.apply(&json!({ "kind": "prop_new", "name": "tod" })).unwrap(); assert_eq!(v["propDefs"].as_array().unwrap().len(), 2);
        assert_eq!(k.apply(&json!({ "kind": "prop_new", "name": "tod" })).unwrap_err().code, "kacheln-eigenschaft");
        let v = k.apply(&json!({ "kind": "color_mode", "mode": "cell" })).unwrap(); assert_eq!(v["colorMode"], "cell"); assert_eq!(k.apply(&json!({ "kind": "color_mode", "mode": "x" })).unwrap_err().code, "op-ungueltig");
        let v = k.apply(&json!({ "kind": "tile_new" })).unwrap(); assert_eq!(v["count"], 5); let v = k.apply(&json!({ "kind": "tile_dup" })).unwrap(); assert_eq!(v["tiles"][5]["double"], 4); let v = k.apply(&json!({ "kind": "tile_merge" })).unwrap(); assert_eq!(v["count"], 5);
        let v = k.apply(&json!({ "kind": "size", "w": 1, "h": 1 })).unwrap(); assert_eq!(v["ask"]["kind"], "size"); let v = k.apply(&json!({ "kind": "ask_no" })).unwrap(); assert!(v["ask"].is_null());
        k.apply(&json!({ "kind": "tile", "i": 1 })).unwrap(); let v = k.apply(&json!({ "kind": "tile_delete" })).unwrap(); assert_eq!(v["ask"]["kind"], "delete"); assert!(v["ask"]["uses"].as_u64().unwrap() > 0, "die Bank nutzt Stein");
        k.apply(&json!({ "kind": "ask_no" })).unwrap();
        assert_eq!(k.apply(&json!({ "kind": "paint", "cells": [99] })).unwrap_err().code, "op-ungueltig"); assert_eq!(k.apply(&json!({ "kind": "charset_remove", "i": 0 })).unwrap_err().code, "op-ungueltig");
        let v = k.apply(&json!({ "kind": "save" })).unwrap(); assert_eq!(v["dirty"], false); assert!(k.export_quelle().is_some());
        std::fs::remove_dir_all(d).ok();
    }
}
