// SPDX-License-Identifier: GPL-3.0-or-later
//! Bildschirm-Editor (E9): `ScreenDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht liefert das Bild (PNG als Datenadresse, Rand und Zeichen in den Systemfarben), die Zeichen- und Farbcodes
//! je Zelle (für Ansagen und Anzeige), Zeichentafel, Farben, Zeichensätze und die Vorschau von „In Kacheln zerlegen“; Absichten (`apply`) sind die Werkzeuge des Modells. Ein Zug ist ein Verlaufsschritt.
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide_model::glyphs::{bildschirmcode, Bild};
use vc_ide_model::screendoc::{ScreenDoc, Werkzeug};

pub struct BildschirmDok { pub doc: ScreenDoc, data: PathBuf, root: PathBuf, norm: String, projekt_norm: &'static str, zerlegen_text: Option<Result<String, String>>, zerlegt: Option<Value> }

fn png_adresse(w: usize, h: usize, rgba: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = vc_media::png::encode_rgba(w, h, rgba); let mut s = String::from("data:image/png;base64,");
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char); s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' }); s.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    s
}
fn werkzeug_name(w: Werkzeug) -> &'static str { match w { Werkzeug::Zeichen => "char", Werkzeug::Text => "text", Werkzeug::Fuellen => "fill", Werkzeug::Rahmen => "rect", Werkzeug::Pipette => "pick", Werkzeug::Loeschen => "erase" } }
fn werkzeug_aus(s: &str) -> Option<Werkzeug> { Some(match s { "char" => Werkzeug::Zeichen, "text" => Werkzeug::Text, "fill" => Werkzeug::Fuellen, "rect" => Werkzeug::Rahmen, "pick" => Werkzeug::Pipette, "erase" => Werkzeug::Loeschen, _ => return None }) }

impl BildschirmDok {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<BildschirmDok, Fehler> {
        let doc = ScreenDoc::oeffnen(root, rel, data).map_err(|e| Fehler::mit("bildschirm-unlesbar", json!({ "detail": e, "rel": rel })))?;
        let projekt_norm = crate::pixel::projektnorm(Some(root), &doc.target);
        Ok(BildschirmDok { doc, data: data.to_path_buf(), root: root.to_path_buf(), norm: "project".into(), projekt_norm, zerlegen_text: None, zerlegt: None })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
    fn norm_aktiv(&self) -> &str { match self.norm.as_str() { "pal" => "pal", "ntsc" => "ntsc", _ => self.projekt_norm } }
    fn palette(&self) -> Option<vc_media::palette::TargetPalette> { vc_media::palette::palette_for(&self.data, &self.doc.target, self.norm_aktiv()).ok() }
    fn zerlegen_vorschau(&self, tw: usize, th: usize) -> Result<String, String> {
        let d = &self.doc; let profiles = vc_media::load_profiles(&self.data)?; let p = vc_media::profile(&profiles, d.media["profile"].as_str().unwrap_or(""))?.clone();
        let satz = profiles.iter().find(|x| x.kind == "tileset" && x.target == p.target).map(|x| x.id.clone()).ok_or_else(|| vc_i18n::t!("screenstudio-fuer-p1-gibt-es-kein-kachelsatz", p1 = &p.target))?;
        let z = vc_ide_model::bankdoc::decompose(&d.media, &p, &satz, tw, th, (0, 0), vc_media::naming::stem(&d.rel))?;
        Ok(vc_i18n::t!("screenstudio-p1-kacheln-p2-zellen-patches-p3", p1 = &z.tiles, p2 = &z.patches, p3 = &z.bytes_before, p4 = &z.bytes_after))
    }
    /// Zerlegen und schreiben: (Kachelsatz, Bank, Info)
    pub fn zerlegen(&mut self, tw: usize, th: usize) -> Result<(String, String, String), String> {
        let d = &self.doc; let profiles = vc_media::load_profiles(&self.data)?; let p = vc_media::profile(&profiles, d.media["profile"].as_str().unwrap_or(""))?.clone();
        let satz = profiles.iter().find(|x| x.kind == "tileset" && x.target == p.target).map(|x| x.id.clone()).ok_or_else(|| vc_i18n::t!("screenstudio-fuer-p1-gibt-es-kein-kachelsatz", p1 = &p.target))?;
        let mut z = vc_ide_model::bankdoc::decompose(&d.media, &p, &satz, tw, th, (0, 0), vc_media::naming::stem(&d.rel))?;
        let (s, b) = vc_ide_model::bankdoc::write_decomp(&self.root, &d.rel, &mut z)?;
        Ok((s, b, vc_i18n::t!("screenstudio-p1-kacheln-p2-zellen-patches-p3", p1 = &z.tiles, p2 = &z.patches, p3 = &z.bytes_before, p4 = &z.bytes_after)))
    }
}

impl Dokument for BildschirmDok {
    fn art(&self) -> &'static str { "screen" }
    fn view(&self) -> Value {
        let d = &self.doc; let pal = self.palette();
        let rgb = |i: usize| -> [u8; 3] { pal.as_ref().and_then(|p| p.rgb(i)).unwrap_or([128, 128, 128]) };
        let b = d.bild(2, &rgb);
        let palette: Vec<Value> = pal.as_ref().map(|p| p.entries().into_iter().map(|(i, c)| json!({ "i": i, "color": format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]) })).collect()).unwrap_or_default();
        let mut codes: Vec<u8> = Vec::with_capacity(d.w * d.h); let mut farben: Vec<u8> = Vec::new();
        for y in 0..d.h { for x in 0..d.w { codes.push(d.code(x, y)); if d.farb_ram { farben.push(d.attr(x, y) as u8) } } }
        // Zeichentafel: alle Zeichen des Satzes in der gewählten Farbe (16 je Zeile)
        let n = if d.target == "atari8" { 128 } else { 256 };
        let mut t = Bild::neu(16 * 8 * 2, (n / 16) * 8 * 2, rgb(d.farbe_von("bg", 6)));
        if let Some(g) = &d.glyphs { for c in 0..n { g.zeichne(&mut t, (c % 16) * 16, (c / 16) * 16, 2, &d.target, c as u8, if d.farb_ram { d.farbe } else { d.farbe_von("fg", 14) }, d.mc(), false, &rgb); } }
        let zer = match &self.zerlegen_text { Some(Ok(s)) => json!({ "ok": true, "text": s }), Some(Err(e)) => json!({ "ok": false, "text": e }), None => Value::Null };
        json!({
            "rel": d.rel, "kind": "screen", "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |x| x.to_string_lossy().into_owned()), "target": d.target, "w": d.w, "h": d.h, "farbRam": d.farb_ram, "atari": d.target == "atari8",
            "img": png_adresse(b.w, b.h, &b.rgba), "imgW": b.w, "imgH": b.h, "rand": d.rand() * 16, "codes": codes, "farben": farben,
            "tafel": { "img": png_adresse(t.w, t.h, &t.rgba), "n": n }, "tool": werkzeug_name(d.werkzeug), "brush": { "code": d.zeichen, "color": d.farbe }, "cursor": [d.cursor.0, d.cursor.1], "cursorCode": d.code(d.cursor.0.min(d.w - 1), d.cursor.1.min(d.h - 1)),
            "roles": { "bg": d.farbe_von("bg", 6), "border": d.farbe_von("border", d.farbe_von("bg", 6)), "fg": d.farbe_von("fg", 14) }, "palette": palette,
            "charset": d.zeichensatz(), "charsets": d.zeichensaetze, "norm": self.norm, "projectNorm": self.projekt_norm, "zerlegen": zer, "zerlegt": self.zerlegt,
            "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "dirty": d.dirty(), "status": d.status,
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.doc.status.clear(); self.zerlegt = None;
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let (w, h) = (self.doc.w, self.doc.h);
        let zellen = |k: &str| -> Result<Vec<(usize, usize)>, Fehler> {
            let v: Vec<(usize, usize)> = op[k].as_array().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })))?.iter().filter_map(|p| Some((p[0].as_u64()? as usize, p[1].as_u64()? as usize))).filter(|&(x, y)| x < w && y < h).collect();
            if v.is_empty() { Err(Fehler::neu("op-ungueltig")) } else { Ok(v) }
        };
        match op["kind"].as_str().unwrap_or("") {
            "tool" => self.doc.werkzeug_waehlen(werkzeug_aus(op["tool"].as_str().unwrap_or("")).ok_or_else(|| Fehler::neu("op-ungueltig"))?),
            "char" => { let c = n("code")?; if c > 255 { return Err(Fehler::neu("op-ungueltig")) } self.doc.zeichen_waehlen(c as u8) }
            "color" => { let i = n("i")?; if self.palette().and_then(|p| p.rgb(i)).is_none() { return Err(Fehler::neu("op-ungueltig")) } self.doc.farbe_waehlen(i) }
            "role" => { let r = op["role"].as_str().unwrap_or(""); if !["bg", "border", "fg"].contains(&r) { return Err(Fehler::neu("op-ungueltig")) } self.doc.rollenfarbe(r, n("i")?) }
            "charset" => { let rel = op["rel"].as_str().map(String::from); if let Some(r) = &rel { if !self.doc.zeichensaetze.contains(r) { return Err(Fehler::neu("op-ungueltig")) } } self.doc.zeichensatz_waehlen(rel) }
            "stroke" => {
                // ein Zug: Zeichen/Löschen über viele Zellen (ein Verlaufsschritt); Füllen, Pipette und Text nehmen die erste Zelle
                let zs = zellen("cells")?; let rechts = op["right"].as_bool().unwrap_or(false);
                self.doc.maus_unten(zs[0].0, zs[0].1, rechts);
                if matches!(self.doc.werkzeug, Werkzeug::Zeichen | Werkzeug::Loeschen) { for &(x, y) in &zs[1..] { self.doc.maus_zug(x, y, rechts) } }
                let l = *zs.last().unwrap_or(&zs[0]); self.doc.maus_los(l.0, l.1, rechts);
            }
            "rect" => { let (a, b) = (zellen("from")?[0], zellen("to")?[0]); if self.doc.werkzeug != Werkzeug::Rahmen { self.doc.werkzeug_waehlen(Werkzeug::Rahmen) } self.doc.maus_unten(a.0, a.1, op["right"].as_bool().unwrap_or(false)); self.doc.maus_los(b.0, b.1, op["right"].as_bool().unwrap_or(false)) }
            "cursor" => { if let (Some(x), Some(y)) = (op["x"].as_u64(), op["y"].as_u64()) { self.doc.cursor = ((x as usize).min(w - 1), (y as usize).min(h - 1)) } else { self.doc.cursor_bewegen(op["dx"].as_i64().unwrap_or(0) as i32, op["dy"].as_i64().unwrap_or(0) as i32) } }
            "type" => {
                // Text tippen: alle Zeichen sind ein Verlaufsschritt
                let t = op["text"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "text" })))?; if t.is_empty() { return Err(Fehler::neu("op-ungueltig")) }
                self.doc.push_undo(); let f = self.doc.farbe;
                for ch in t.chars() { let (x, y) = self.doc.cursor; let c = bildschirmcode(&self.doc.target, ch); self.doc.setze(x, y, c, f); self.doc.cursor = if x + 1 < w { (x + 1, y) } else if y + 1 < h { (0, y + 1) } else { (x, y) }; }
            }
            "norm" => { let v = op["norm"].as_str().unwrap_or(""); if !["project", "pal", "ntsc"].contains(&v) { return Err(Fehler::neu("op-ungueltig")) } self.norm = v.to_string() }
            "zerlegen_vorschau" => { let (tw, th) = (n("w")?, n("h")?); self.zerlegen_text = Some(self.zerlegen_vorschau(tw, th)) }
            "zerlegen" => { let (tw, th) = (n("w")?, n("h")?); let (s, b, info) = self.zerlegen(tw, th).map_err(|e| Fehler::mit("bildschirm-zerlegen", json!({ "detail": e })))?; self.zerlegt = Some(json!({ "set": s, "bank": b, "info": info })) }
            "zerlegen_ende" => { self.zerlegen_text = None; self.zerlegt = None }
            "undo" => { if !self.doc.kann_undo() { return Err(Fehler::neu("undo-leer")) } self.doc.undo() }
            "redo" => { if !self.doc.kann_redo() { return Err(Fehler::neu("redo-leer")) } self.doc.redo() }
            "save" => self.save()?,
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        Ok(self.view())
    }
    fn undo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "undo" })) }
    fn redo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "redo" })) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.save().map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e }))) }
    fn undo_tiefe(&self) -> usize { if self.doc.kann_undo() { 1 } else { 0 } }
    fn export_quelle(&self) -> Option<ExportQuelle> {
        let rel = &self.doc.rel; let name = Path::new(rel).file_stem().map_or_else(|| "screen".into(), |s| s.to_string_lossy().into_owned());
        Some(ExportQuelle { rel: rel.clone(), kind: "screen".into(), name, media_text: self.doc.media_text() })
    }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn kopie(name: &str) -> (PathBuf, BildschirmDok) {
        let w = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let d = std::env::temp_dir().join(format!("vc_bild_{name}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("graphics")).unwrap();
        for f in ["font.vcchar", "title.vcscreen"] { std::fs::copy(w.join("examples/c64/gfx_demo/graphics").join(f), d.join("graphics").join(f)).unwrap(); }
        let k = BildschirmDok::oeffnen(&d, "graphics/title.vcscreen", &w.join("data")).unwrap(); (d, k)
    }
    #[test] fn ansicht_hat_bild_codes_tafel_und_farben() {
        let (d, k) = kopie("ansicht"); let v = k.view();
        assert_eq!((v["w"].clone(), v["h"].clone()), (json!(40), json!(25))); assert_eq!(v["codes"].as_array().unwrap().len(), 1000); assert_eq!(v["farben"].as_array().unwrap().len(), 1000);
        assert!(v["img"].as_str().unwrap().starts_with("data:image/png;base64,")); assert_eq!(v["tafel"]["n"], 256); assert!(!v["palette"].as_array().unwrap().is_empty()); assert_eq!(v["tool"], "char"); assert_eq!(v["charsets"][0], "graphics/font.vcchar");
        std::fs::remove_dir_all(d).ok();
    }
    #[test] fn zug_text_fuellen_rahmen_rollen_und_zerlegen() {
        let (d, mut k) = kopie("ops");
        k.apply(&json!({ "kind": "char", "code": 66 })).unwrap(); k.apply(&json!({ "kind": "color", "i": 5 })).unwrap();
        let v = k.apply(&json!({ "kind": "stroke", "cells": [[2, 2], [3, 2], [6, 2]] })).unwrap(); let c = |x: usize, y: usize| v["codes"][y * 40 + x].as_u64().unwrap();
        assert_eq!((c(2, 2), c(3, 2), c(4, 2), c(6, 2)), (66, 66, 66, 66), "Zug ohne Lücken"); assert_eq!(v["dirty"], true);
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_ne!(v["codes"][2 * 40 + 6], 66, "ein Rückgängig nimmt den ganzen Zug"); k.apply(&json!({ "kind": "redo" })).unwrap();
        k.apply(&json!({ "kind": "tool", "tool": "text" })).unwrap(); k.apply(&json!({ "kind": "cursor", "x": 10, "y": 10 })).unwrap(); let v = k.apply(&json!({ "kind": "type", "text": "AB" })).unwrap(); assert_eq!((v["codes"][10 * 40 + 10].clone(), v["codes"][10 * 40 + 11].clone(), v["cursor"].clone()), (json!(1), json!(2), json!([12, 10])));
        let v = k.apply(&json!({ "kind": "undo" })).unwrap(); assert_ne!(v["codes"][10 * 40 + 10], 1, "Text = ein Schritt"); k.apply(&json!({ "kind": "redo" })).unwrap();
        k.apply(&json!({ "kind": "rect", "from": [[20, 20]], "to": [[23, 22]] })).unwrap(); let v = k.view(); assert_eq!(v["codes"][20 * 40 + 20], 66);
        k.apply(&json!({ "kind": "tool", "tool": "fill" })).unwrap(); let v = k.apply(&json!({ "kind": "stroke", "cells": [[30, 20]] })).unwrap(); assert_eq!(v["codes"][24 * 40 + 39], 66);
        k.apply(&json!({ "kind": "tool", "tool": "pick" })).unwrap(); let v = k.apply(&json!({ "kind": "stroke", "cells": [[2, 2]] })).unwrap(); assert_eq!((v["brush"]["code"].clone(), v["tool"].clone()), (json!(66), json!("char")), "Pipette nimmt auf und kehrt zum Zeichenwerkzeug zurück");
        let v = k.apply(&json!({ "kind": "role", "role": "border", "i": 2 })).unwrap(); assert_eq!(v["roles"]["border"], 2);
        let v = k.apply(&json!({ "kind": "charset", "rel": null })).unwrap(); assert!(v["charset"].is_null()); k.apply(&json!({ "kind": "undo" })).unwrap();
        assert_eq!(k.apply(&json!({ "kind": "charset", "rel": "nirgends.vcchar" })).unwrap_err().code, "op-ungueltig"); assert_eq!(k.apply(&json!({ "kind": "stroke", "cells": [[99, 99]] })).unwrap_err().code, "op-ungueltig"); assert_eq!(k.apply(&json!({ "kind": "tool", "tool": "x" })).unwrap_err().code, "op-ungueltig");
        let v = k.apply(&json!({ "kind": "zerlegen_vorschau", "w": 2, "h": 2 })).unwrap(); assert_eq!(v["zerlegen"]["ok"], true, "{:?}", v["zerlegen"]); let v = k.apply(&json!({ "kind": "zerlegen", "w": 2, "h": 2 })).unwrap(); let r = &v["zerlegt"]; assert!(r["set"].as_str().unwrap().ends_with("_tiles.vctiles") && d.join(r["set"].as_str().unwrap()).is_file());
        let v = k.apply(&json!({ "kind": "save" })).unwrap(); assert_eq!(v["dirty"], false); assert!(k.export_quelle().is_some());
        std::fs::remove_dir_all(d).ok();
    }
}
