// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprite-Editor (E3): `PixelDoc` (vc-ide-model) als Dokument im `DocStore`. Die Ansicht (`view`) ist das Anzeige-DTO für Zeichenfläche, Farben und
//! Statuszeile; Absichten (`apply`) sind gebündelte Vorgänge: ein Strich der Oberfläche (viele Punkte) ist **ein** Verlaufsschritt.
//! Pixelwerte, Farben und Verlauf kommen aus dem Kern; die Oberfläche zeichnet nur lokal vor und ersetzt ihren Stand durch die Antwort.
use crate::docstore::{Dokument, ExportQuelle, Fehler};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide_model::pixel::{AnimOp, PixelDoc, Tool};
use vc_media::sprite::AnimMode;

pub struct PixelDok { pub doc: PixelDoc, data: PathBuf, root: PathBuf, /** gewählte Palettenansicht: `project`, `pal` oder `ntsc` */ norm: String, projekt_norm: &'static str, pub meldung: Option<(bool, String)>, /** Zeichen der geladenen Vorlage (Verweis, Zeichen) für den Vergleich */ vorlage_cache: std::sync::Mutex<Option<(String, Value)>> }

/// Videonorm der Projektdatei für das Ziel (`variant` im Eintrag des Ziels; sonst PAL)
pub fn projektnorm(root: Option<&Path>, ziel: &str) -> &'static str {
    let Some(pf) = root.and_then(vc_ide::projfile::find).and_then(|f| vc_ide::projfile::ProjectFile::load(&f).ok()) else { return "pal" };
    let v = pf.targets.iter().find(|t| t.system == ziel || (t.system.is_empty() && t.id == ziel)).map(|t| t.variant.to_ascii_lowercase());
    if v.as_deref() == Some("ntsc") { "ntsc" } else { "pal" }
}

/// Größe eines Bildes im Speicher (Block des Profils, sonst aus Breite, Höhe und Bits)
fn block_bytes(d: &PixelDoc) -> usize {
    let bytes = |w: usize| (w * d.bpp).div_ceil(8) * d.height;
    if d.kind == "sprite" { 64 } else { bytes(d.width) }
}
fn hex(c: [u8; 3]) -> String { format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2]) }
fn werkzeug_name(t: Tool) -> &'static str { match t { Tool::Pen => "pen", Tool::Erase => "erase", Tool::Fill => "fill", Tool::Line => "line", Tool::Rect => "rect", Tool::Select => "select", Tool::Pick => "pick" } }
fn werkzeug_aus(s: &str) -> Option<Tool> { Some(match s { "pen" => Tool::Pen, "erase" => Tool::Erase, "fill" => Tool::Fill, "line" => Tool::Line, "rect" => Tool::Rect, "select" => Tool::Select, "pick" => Tool::Pick, _ => return None }) }

/// Anzeigename eines Pixelwerts/einer Farbrolle: die Namen im Profil sind Bezeichner (Programmlogik vergleicht sie), die Oberfläche zeigt den Katalogtext der Sprache
fn wert_anzeige(name: &str) -> String {
    match name {
        "transparent" => vc_i18n::t!("profilwert-transparent").to_string(),
        "Sprite-Farbe" => vc_i18n::t!("profilwert-sprite-farbe").to_string(),
        "Multicolor 1" => vc_i18n::t!("profilwert-multicolor-1").to_string(),
        "Multicolor 1 ($D025)" => vc_i18n::t!("profilwert-multicolor-1-d025").to_string(),
        "Multicolor 2" => vc_i18n::t!("profilwert-multicolor-2").to_string(),
        "Multicolor 2 ($D026)" => vc_i18n::t!("profilwert-multicolor-2-d026").to_string(),
        "Hilfsfarbe" => vc_i18n::t!("profilwert-hilfsfarbe").to_string(),
        "Hintergrund" => vc_i18n::t!("profilwert-hintergrund").to_string(),
        "Hintergrund (Farbe 0, $FF15)" => vc_i18n::t!("profilwert-hintergrund-farbe-0-ff15").to_string(),
        "Rahmen" => vc_i18n::t!("profilwert-rahmen").to_string(),
        "Spielerfarbe" => vc_i18n::t!("profilwert-spielerfarbe").to_string(),
        "Vordergrund" => vc_i18n::t!("profilwert-vordergrund").to_string(),
        "Zeichenfarbe" => vc_i18n::t!("profilwert-zeichenfarbe").to_string(),
        "Zeichenfarbe (Farb-RAM)" => vc_i18n::t!("profilwert-zeichenfarbe-farb-ram").to_string(),
        "Palettenfarbe 0–255" => vc_i18n::t!("profilwert-palettenfarbe-0-255").to_string(),
        "Farbe 0 (COLPF0)" => vc_i18n::t!("profilwert-farbe-0-colpf0").to_string(),
        "Farbe 1 ($FF16)" => vc_i18n::t!("profilwert-farbe-1-ff16").to_string(),
        "Farbe 1 (COLPF1)" => vc_i18n::t!("profilwert-farbe-1-colpf1").to_string(),
        "Farbe 2 ($FF17)" => vc_i18n::t!("profilwert-farbe-2-ff17").to_string(),
        "Farbe 2 (COLPF2; Zeichenbit 7: COLPF3)" => vc_i18n::t!("profilwert-farbe-2-colpf2-zeichenbit-7-colpf3").to_string(),
        _ => name.to_string(),
    }
}

impl PixelDok {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path, projekt: Option<&Path>) -> Result<PixelDok, Fehler> {
        let doc = PixelDoc::open(root, rel, data).map_err(|e| Fehler::mit("pixel-unlesbar", json!({ "detail": e, "rel": rel })))?;
        let projekt_norm = projektnorm(projekt, &doc.target);
        Ok(PixelDok { doc, data: data.to_path_buf(), root: root.to_path_buf(), norm: "project".into(), projekt_norm, meldung: None, vorlage_cache: Default::default() })
    }
    pub fn doc_view(&self) -> Value { Dokument::view(self) }
    /// Videonorm der Palette, die gezeigt wird
    fn norm_aktiv(&self) -> &str { match self.norm.as_str() { "pal" => "pal", "ntsc" => "ntsc", _ => self.projekt_norm } }
    fn palette(&self) -> Option<vc_media::palette::TargetPalette> { vc_media::palette::palette_for(&self.data, &self.doc.target, self.norm_aktiv()).ok() }
    /// Zahl der wählbaren Pixelwerte im gewählten Frame (Hires-Sprites 2, sonst bis 4)
    fn werte_anzahl(&self) -> usize {
        let d = &self.doc;
        if d.kind == "sprite" && !d.frame_modes.is_empty() && !d.ist_multicolor(d.fi) { 2 } else { (1usize << d.bpp.min(2)).clamp(2, 4) }
    }
    /// Farbrolle (Schlüssel aus `farben()`), die ein Pixelwert im gewählten Bild bekommt; `None` = durchsichtig. Spiegelt `PixelDoc::farb_index_frame`
    fn rolle(&self, v: usize) -> Option<String> {
        let d = &self.doc;
        match d.kind.as_str() {
            "sprite" => if v == 0 { None } else { Some(if !d.ist_multicolor(d.fi) || v == 2 { "sprite" } else if v == 1 { "mc1" } else { "mc2" }.into()) },
            "pmg" => if v == 0 { None } else { Some(format!("P{}", d.fi)) },
            _ => Some(if v == 0 { "bg" } else if v == 1 { if d.bpp == 2 { "mc1" } else { "fg" } } else if v == 2 { "mc2" } else { "fg" }.into()),
        }
    }
    /// Pixel des gewählten Bildes ändern; nur wenn sich wirklich etwas ändert, entsteht ein Verlaufsschritt (davor liegt der Stand vor der Änderung)
    fn aendern(&mut self, f: impl FnOnce(&mut PixelDoc)) {
        let fi = self.doc.fi; let alt = self.doc.frames[fi].clone();
        f(&mut self.doc);
        if self.doc.frames[fi] != alt { let neu = std::mem::replace(&mut self.doc.frames[fi], alt); self.doc.push_undo(); self.doc.frames[fi] = neu; }
    }
    /// Zeichensatz-Karten: Vorlagen zum Profil, aktuelle Vorlage, Herkunft und je Zeichen, ob es von der Vorlage abweicht
    fn zeichensatz_ansicht(&self) -> Value {
        let d = &self.doc;
        let vorlagen: Vec<Value> = d.vorlagen(&self.data).into_iter().map(|(r, l, fehlt)| json!({ "ref": r, "label": l, "missing": fehlt })).collect();
        let herkunft = d.herkunft(&self.data).map(|(l, z)| json!({ "label": l, "lines": z })).unwrap_or(Value::Null);
        let aktuell = d.vorlage_aktuell();
        let vorlage = if aktuell.is_empty() { None } else {
            let mut c = self.vorlage_cache.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if c.as_ref().map(|x| x.0.as_str()) != Some(aktuell.as_str()) { *c = d.vorlage_frames(&self.data).map(|f| (aktuell.clone(), f)); }
            c.as_ref().map(|x| x.1.clone()) };
        let geaendert: Value = match (vorlage, d.frames_json()) {
            (Some(Value::Array(f)), Value::Array(jetzt)) => json!((0..jetzt.len()).map(|i| f.get(i).is_some_and(|x| *x != jetzt[i])).collect::<Vec<_>>()),
            _ => Value::Null };
        json!({ "vorlagen": vorlagen, "vorlage": d.vorlage_aktuell(), "herkunft": herkunft, "geaendert": geaendert })
    }
    fn pv(&self) -> u8 { self.doc.pv.min(self.werte_anzahl() as u8 - 1) }
}

impl Dokument for PixelDok {
    fn art(&self) -> &'static str { "pixel" }
    fn view(&self) -> Value {
        let d = &self.doc; let pal = self.palette();
        let rgb = |i: Option<usize>| -> Value { match (i, &pal) { (Some(i), Some(p)) => p.rgb(i).map_or(Value::Null, |c| json!(hex(c))), _ => Value::Null } };
        let n = self.werte_anzahl();
        // Farbe je Pixelwert (null = durchsichtig); bei 8 Bit je Pixel die ganze Palette
        let wertefarben: Vec<Value> = (0..if d.bpp == 8 { 256 } else { n }).map(|v| rgb(d.farb_index(v as u8))).collect();
        let werte: Vec<Value> = (0..n).map(|v| json!({ "v": v, "name": wert_anzeige(&d.values.get(v).cloned().unwrap_or_default()), "color": wertefarben[v], "role": self.rolle(v) })).collect();
        let slots: Vec<Value> = d.farben().into_iter().map(|(key, label)| { let i = d.farbe(&key, 1); json!({ "key": key, "label": wert_anzeige(&label), "index": i, "color": rgb(Some(i)) }) }).collect();
        let eintraege: Vec<Value> = pal.as_ref().map(|p| p.entries().into_iter().map(|(i, c)| json!({ "i": i, "color": hex(c) })).collect()).unwrap_or_default();
        let pixels: Vec<u8> = d.frames[d.fi].iter().flat_map(|r| r.iter().copied()).collect();
        let pn = |n: &str| vc_media::palette::palette_for(&self.data, &d.target, n).ok().map(|p| p.entries());
        // alle Bilder für Bilderliste, Onion Skin und Vorschau im Spiel: Pixel, Breite und Farbe je Wert (je nach Modus des Bildes)
        let thumbs: Vec<Value> = (0..d.frames.len()).map(|i| json!({
            "w": d.frames[i].first().map_or(0, |r| r.len()), "aspect": if d.kind == "sprite" && !d.frame_modes.is_empty() { d.frame_modes[usize::from(d.ist_multicolor(i))].aspect } else { d.aspect },
            "mc": d.ist_multicolor(i), "pixels": d.frames[i].iter().flat_map(|r| r.iter().copied()).collect::<Vec<u8>>(),
            "colors": (0..4u8).map(|v| rgb(d.farb_index_frame(i, v))).collect::<Vec<_>>() })).collect();
        let modi: Value = if d.kind == "sprite" && d.frame_modes.len() > 1 { json!(d.frame_modes.iter().map(|m| json!({ "label": m.label, "hint": m.hint })).collect::<Vec<_>>()) } else { Value::Null };
        let bits = vc_ide_model::pixel::flags_from_hw(&d.media["hw"]); let hat_flags = vc_ide_model::pixel::has_sprite_flags(&d.target, &d.kind);
        let (himmel, boden) = match d.target.as_str() { "atari8" => (pal.as_ref().and_then(|p| p.rgb(0x98)), pal.as_ref().and_then(|p| p.rgb(0xc6))), _ => (pal.as_ref().and_then(|p| p.rgb(14)), pal.as_ref().and_then(|p| p.rgb(5))) };
        let (anims, anim_fehler): (Value, Value) = if d.kind == "sprite" { match d.animationen() {
            Ok(l) => (json!(l.iter().map(|a| json!({ "name": a.name, "mode": a.mode.key(), "steps": a.steps.iter().map(|s| json!([s.0, s.1])).collect::<Vec<_>>(), "expanded": a.expanded().iter().map(|s| json!([s.0, s.1])).collect::<Vec<_>>() })).collect::<Vec<_>>()), Value::Null),
            Err(e) => (json!([]), json!(e)) } } else { (Value::Null, Value::Null) };
        let hat_varianten = pn("pal") != pn("ntsc");
        json!({
            "rel": d.rel, "kind": d.kind, "profile": d.profile, "target": d.target, "title": Path::new(&d.rel).file_name().map_or_else(|| d.rel.clone(), |n| n.to_string_lossy().into_owned()), "kindTitle": d.titel(), "width": d.width, "height": d.height, "bpp": d.bpp, "aspect": d.aspect,
            "frames": d.frames.len(), "fi": d.fi, "frameNames": d.frame_names, "fixedFrames": d.fixed_frames, "multicolor": d.ist_multicolor(d.fi),
            "pixels": pixels, "anims": anims, "animError": anim_fehler, "selAnim": d.sel_anim, "thumbs": thumbs, "modes": modi, "pmg": if vc_ide_model::pixel::has_pmg_props(&d.target, &d.kind) { let (sz, pr) = vc_ide_model::pixel::pmg_from_hw(&d.media["hw"]); json!({ "sizes": sz, "prior": pr, "priorChoices": vc_ide_model::pixel::PMG_PRIOR, "sizeChoices": vc_ide_model::pixel::PMG_SIZES }) } else { Value::Null },
            "flags": if hat_flags { json!({ "x2": bits & 2 != 0, "y2": bits & 4 != 0, "bg": bits & 8 != 0 }) } else { Value::Null },
            "scene": { "sky": himmel.map(hex), "ground": boden.map(hex) }, "blockBytes": block_bytes(d), "pv": self.pv(), "tool": werkzeug_name(d.tool), "values": werte, "valueColors": wertefarben, "slots": slots, "palette": eintraege,
            "norm": self.norm, "normActive": self.norm_aktiv(), "projectNorm": self.projekt_norm, "normChoice": hat_varianten,
            "sel": d.sel.map(|s| json!([s.0, s.1, s.2, s.3])), "hasClip": d.clip.is_some(), "canUndo": d.kann_undo(), "canRedo": d.kann_redo(), "dirty": d.dirty(), "status": d.status,
            "charset": if d.kind == "charset" { self.zeichensatz_ansicht() } else { Value::Null },
            "note": self.meldung.as_ref().map(|(ok, t)| json!({ "ok": ok, "text": t })),
        })
    }
    fn apply(&mut self, op: &Value) -> Result<Value, Fehler> {
        self.meldung = None; self.doc.status.clear();
        let n = |k: &str| op[k].as_u64().map(|x| x as usize).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k })));
        let (w, h) = (self.doc.width, self.doc.height);
        match op["kind"].as_str().unwrap_or("") {
            "tool" => { self.doc.tool = werkzeug_aus(op["tool"].as_str().unwrap_or("")).ok_or_else(|| Fehler::neu("op-ungueltig"))?; }
            "value" => { let v = n("v")?; if v >= self.werte_anzahl() { return Err(Fehler::neu("op-ungueltig")); } self.doc.pv = v as u8; }
            "pick" => { let (x, y) = (n("x")?, n("y")?); if x >= w || y >= h { return Err(Fehler::neu("op-ungueltig")); } self.doc.pv = self.doc.pixel(x, y).min(self.werte_anzahl() as u8 - 1); }
            "stroke" => {
                // ein Strich: alle Punkte zusammen sind ein Verlaufsschritt; aufeinanderfolgende Punkte werden verbunden (keine Lücken bei schnellem Ziehen)
                let pts: Vec<(usize, usize)> = op["points"].as_array().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "points" })))?.iter()
                    .filter_map(|p| Some((p[0].as_u64()? as usize, p[1].as_u64()? as usize))).filter(|&(x, y)| x < w && y < h).collect();
                if pts.is_empty() { return Err(Fehler::neu("op-ungueltig")); }
                let wert = if op["erase"].as_bool().unwrap_or(false) { 0 } else { self.pv() };
                self.aendern(|d| { let mut vor: Option<(usize, usize)> = None; for &p in &pts { match vor { Some(q) => d.linie(q, p, wert), None => d.setze(p.0, p.1, wert) } vor = Some(p); } });
            }
            "shape" => {
                // Linie oder Rahmen von a nach b (Absicht der Zeichenfläche beim Loslassen)
                let pt = |k: &str| -> Result<(usize, usize), Fehler> { let a = &op[k]; match (a[0].as_u64(), a[1].as_u64()) { (Some(x), Some(y)) if (x as usize) < w && (y as usize) < h => Ok((x as usize, y as usize)), _ => Err(Fehler::neu("op-ungueltig")) } };
                let (a, b) = (pt("a")?, pt("b")?); let wert = if op["erase"].as_bool().unwrap_or(false) { 0 } else { self.pv() };
                match op["tool"].as_str().unwrap_or("") { "line" => self.aendern(|d| d.linie(a, b, wert)), "rect" => self.aendern(|d| d.rahmen(a, b, wert)), _ => return Err(Fehler::neu("op-ungueltig")) }
            }
            "fill" => { let (x, y) = (n("x")?, n("y")?); if x >= w || y >= h { return Err(Fehler::neu("op-ungueltig")); } let wert = if op["erase"].as_bool().unwrap_or(false) { 0 } else { self.pv() }; self.aendern(|d| d.fuelle(x, y, wert)); }
            "select" => {
                let pt = |k: &str| -> Result<(usize, usize), Fehler> { let a = &op[k]; match (a[0].as_u64(), a[1].as_u64()) { (Some(x), Some(y)) => Ok(((x as usize).min(w - 1), (y as usize).min(h - 1))), _ => Err(Fehler::neu("op-ungueltig")) } };
                let (a, b) = (pt("a")?, pt("b")?); self.doc.sel = Some((a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1)));
            }
            "select_all" => self.doc.sel = Some((0, 0, w - 1, h - 1)),
            "select_none" => self.doc.sel = None,
            "copy" | "cut" => {
                let Some((x0, y0, x1, y1)) = self.doc.sel else { return Err(Fehler::neu("pixel-keine-auswahl")) };
                let schnitt = op["kind"] == "cut"; self.doc.auswahl_kopieren(schnitt);
                self.meldung = Some((true, if schnitt { vc_i18n::t!("sprite-ausgeschnitten", w = x1 - x0 + 1, h = y1 - y0 + 1).to_string() } else { vc_i18n::t!("sprite-kopiert", w = x1 - x0 + 1, h = y1 - y0 + 1).to_string() }));
            }
            "paste" => {
                if self.doc.clip.is_none() { return Err(Fehler::neu("pixel-zwischenablage-leer")); }
                // Einfügestelle: gegebene Zelle, sonst die Auswahl, sonst oben links
                if let (Some(x), Some(y)) = (op["x"].as_u64(), op["y"].as_u64()) { let (x, y) = ((x as usize).min(w - 1), (y as usize).min(h - 1)); self.doc.sel = Some((x, y, x, y)); }
                self.doc.auswahl_einfuegen();
            }
            "clear_selection" => { if self.doc.sel.is_none() { return Err(Fehler::neu("pixel-keine-auswahl")); } self.doc.auswahl_loeschen(); }
            "flip" => { let wie = match op["dir"].as_str() { Some("h") => "mh", Some("v") => "mv", _ => return Err(Fehler::neu("op-ungueltig")) }; self.doc.operation(wie); }
            "shift" => { let wie = match op["dir"].as_str() { Some("l") => "sl", Some("r") => "sr", Some("u") => "su", Some("d") => "sd", _ => return Err(Fehler::neu("op-ungueltig")) }; self.doc.operation(wie); }
            "clear" => self.doc.operation("clear"),
            "color" => {
                let key = op["key"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "key" })))?; let i = n("index")?;
                if !self.doc.farben().iter().any(|(k, _)| k == key) { return Err(Fehler::neu("op-ungueltig")); }
                if self.palette().and_then(|p| p.rgb(i)).is_none() { return Err(Fehler::neu("op-ungueltig")); }
                if self.doc.farbe(key, usize::MAX) != i { self.doc.farbe_setzen(key, i); }
            }
            "norm" => { let v = op["norm"].as_str().unwrap_or(""); if !["project", "pal", "ntsc"].contains(&v) { return Err(Fehler::neu("op-ungueltig")); } self.norm = v.to_string(); }
            "frame" => { let i = n("i")?; if i >= self.doc.frames.len() { return Err(Fehler::neu("op-ungueltig")); } self.doc.frame_waehlen(i); }
            "frame_new" => { if self.doc.fixed_frames { return Err(Fehler::neu("op-ungueltig")); } self.doc.frame_anhaengen(); }
            "frame_move" => { if self.doc.fixed_frames { return Err(Fehler::neu("op-ungueltig")); } let (von, nach) = (n("from")?, n("to")?); if von >= self.doc.frames.len() || nach >= self.doc.frames.len() { return Err(Fehler::neu("op-ungueltig")); } self.doc.frame_verschieben(von, nach); }
            "frame_dup" => { if self.doc.fixed_frames { return Err(Fehler::neu("op-ungueltig")); } self.doc.frame_duplizieren(); }
            "frame_delete" => { if self.doc.fixed_frames || self.doc.frames.len() < 2 { return Err(Fehler::neu("op-ungueltig")); } self.doc.frame_loeschen(); }
            "mode" => { if self.doc.kind != "sprite" || self.doc.frame_modes.len() < 2 { return Err(Fehler::neu("op-ungueltig")); } self.doc.modus_wechseln(op["multicolor"].as_bool().unwrap_or(false)); }
            "flag" => {
                if !vc_ide_model::pixel::has_sprite_flags(&self.doc.target, &self.doc.kind) { return Err(Fehler::neu("op-ungueltig")); }
                let bit = match op["flag"].as_str() { Some("x2") => 2, Some("y2") => 4, Some("bg") => 8, _ => return Err(Fehler::neu("op-ungueltig")) };
                let mut bits = vc_ide_model::pixel::flags_from_hw(&self.doc.media["hw"]); match op["on"].as_bool() { Some(true) => bits |= bit, Some(false) => bits &= !bit, None => bits ^= bit }
                self.doc.push_undo(); vc_ide_model::pixel::flags_to_hw(&mut self.doc.media, bits);
            }
            "anim_new" | "anim_delete" | "anim_rename" | "anim_mode" | "anim_add_step" | "anim_del_step" | "anim_move" | "anim_delay" | "anim_from_frames" | "anim_move_to" | "anim_delay_all" => {
                if self.doc.kind != "sprite" { return Err(Fehler::neu("op-ungueltig")); }
                let o = match op["kind"].as_str().unwrap_or("") {
                    "anim_new" => AnimOp::New, "anim_delete" => AnimOp::Delete,
                    "anim_move_to" => AnimOp::MoveTo(n("from")?, n("to")?),
                    "anim_delay_all" => AnimOp::DelayAll(n("delay")?.min(255) as u32),
                    "anim_from_frames" => AnimOp::Aus(op["frames"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect()).unwrap_or_default(), op["name"].as_str().unwrap_or("").to_string(), AnimMode::parse(op["mode"].as_str().unwrap_or("loop")), op["delay"].as_u64().unwrap_or(6).min(255) as u32),
                    "anim_rename" => AnimOp::Rename(op["name"].as_str().unwrap_or("").to_string()),
                    "anim_mode" => AnimOp::Mode(AnimMode::parse(op["mode"].as_str().unwrap_or(""))),
                    "anim_add_step" => { let f = n("frame")?; if f >= self.doc.frames.len() { return Err(Fehler::neu("op-ungueltig")); } AnimOp::AddStep(f) }
                    "anim_del_step" => AnimOp::DelStep(n("i")?),
                    "anim_move" => AnimOp::Move(n("i")?, op["dir"].as_i64().unwrap_or(0).signum() as isize),
                    _ => AnimOp::Delay(n("i")?, n("delay")?.min(255) as u32),
                };
                self.doc.anim_aendern(o).map_err(|e| Fehler::mit("pixel-anim", json!({ "detail": e })))?;
            }
            "anim_select" => { let i = n("i")?; let len = self.doc.animationen().map(|l| l.len()).unwrap_or(0); if i >= len { return Err(Fehler::neu("op-ungueltig")); } self.doc.sel_anim = i; }
            "png" => {
                let pfad = op["path"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "path" })))?;
                let bytes = std::fs::read(pfad).map_err(|e| Fehler::mit("pixel-png", json!({ "detail": e.to_string() })))?;
                let m = self.doc.png_importieren(&self.root, &self.data, &bytes).map_err(|e| Fehler::mit("pixel-png", json!({ "detail": e })))?; self.meldung = Some((true, m));
            }
            "pmg_size" | "pmg_prior" => {
                if !vc_ide_model::pixel::has_pmg_props(&self.doc.target, &self.doc.kind) { return Err(Fehler::neu("op-ungueltig")); }
                let (mut sizes, mut prior) = vc_ide_model::pixel::pmg_from_hw(&self.doc.media["hw"]);
                if op["kind"] == "pmg_size" {
                    let (i, v) = (n("i")?, n("size")? as u8); if i >= 4 || !vc_ide_model::pixel::PMG_SIZES.contains(&v) { return Err(Fehler::neu("op-ungueltig")); } sizes[i] = v;
                } else {
                    let v = n("prior")? as u8; if v != 0 && !vc_ide_model::pixel::PMG_PRIOR.contains(&v) { return Err(Fehler::neu("op-ungueltig")); } prior = v;
                }
                self.doc.push_undo(); vc_ide_model::pixel::pmg_to_hw(&mut self.doc.media, sizes, prior);
            }
            "vorlage" => {
                if self.doc.kind != "charset" { return Err(Fehler::neu("op-ungueltig")); }
                let r = op["ref"].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "ref" })))?;
                if !self.doc.vorlagen(&self.data).iter().any(|v| v.0 == r && !v.2) { return Err(Fehler::neu("op-ungueltig")); }
                self.doc.vorlage_anwenden_mit(&self.data, r).map_err(|e| Fehler::mit("pixel-vorlage", json!({ "detail": e })))?;
            }
            "zeichen_original" => {
                if self.doc.kind != "charset" { return Err(Fehler::neu("op-ungueltig")); }
                let i = n("i")?; let f = self.doc.vorlage_frames(&self.data).and_then(|f| f.get(i).cloned()).ok_or_else(|| Fehler::neu("op-ungueltig"))?;
                self.doc.zeichen_auf(i, &f).map_err(|e| Fehler::mit("pixel-vorlage", json!({ "detail": e })))?;
            }
            "undo" => { if !self.doc.kann_undo() { return Err(Fehler::neu("undo-leer")); } self.doc.undo(); }
            "redo" => { if !self.doc.kann_redo() { return Err(Fehler::neu("redo-leer")); } self.doc.redo(); }
            "save" => { self.save()?; self.meldung = Some((true, vc_i18n::t!("pixel-gespeichert").to_string())); }
            _ => return Err(Fehler::neu("op-ungueltig")),
        }
        Ok(self.view())
    }
    fn undo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "undo" })) }
    fn redo(&mut self) -> Result<Value, Fehler> { self.apply(&json!({ "kind": "redo" })) }
    fn save(&mut self) -> Result<(), Fehler> { self.doc.save().map_err(|e| Fehler::mit("pixel-speichern", json!({ "detail": e }))) }
    fn undo_tiefe(&self) -> usize { usize::from(self.doc.kann_undo()) }
    fn export_quelle(&self) -> Option<ExportQuelle> { Some(ExportQuelle { rel: self.doc.rel.clone(), kind: self.doc.kind.clone(), name: self.doc.media["name"].as_str().unwrap_or("export").to_string(), media_text: self.doc.media_text() }) }
    fn ungesichert(&self) -> bool { self.doc.dirty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wurzel() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn kopie(name: &str) -> (PathBuf, PixelDok) {
        let dir = std::env::temp_dir().join(format!("vc_pixel_{}_{}", name, std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(wurzel().join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("hero.vcsprite")).unwrap();
        let d = PixelDok::oeffnen(&dir, "hero.vcsprite", &wurzel().join("data"), None).unwrap(); (dir, d)
    }
    #[test] fn ansicht_hat_pixel_farben_und_palette() {
        let (dir, d) = kopie("ansicht"); let v = d.view();
        assert_eq!(v["width"], 12); assert_eq!(v["height"], 21); assert_eq!(v["pixels"].as_array().unwrap().len(), 12 * 21);
        assert_eq!(v["values"].as_array().unwrap().len(), 4, "Multicolor: vier Werte"); assert!(v["values"][0]["color"].is_null(), "Wert 0 ist durchsichtig");
        assert!(v["values"][2]["color"].is_string()); assert_eq!(v["palette"].as_array().unwrap().len(), 16); assert_eq!(v["dirty"], false);
        assert_eq!(v["slots"].as_array().unwrap().len(), 3); assert_eq!(v["normActive"], "pal");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn zeichensatz_vorlage_laden_vergleichen_und_zuruecksetzen() {
        let dir = std::env::temp_dir().join(format!("vc_pixel_vorlage_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(wurzel().join("examples/c64/gfx_demo/graphics/font.vcchar"), dir.join("font.vcchar")).unwrap();
        let mut d = PixelDok::oeffnen(&dir, "font.vcchar", &wurzel().join("data"), None).unwrap();
        let v = d.view(); assert_eq!(v["kind"], "charset"); let vl = v["charset"]["vorlagen"].as_array().unwrap().clone();
        let eigen = vl.iter().find(|x| x["ref"].as_str().is_some_and(|r| r.starts_with("std:")) && x["missing"] == false).expect("ein eigener Satz steht zur Wahl")["ref"].as_str().unwrap().to_string();
        assert_eq!(d.apply(&json!({ "kind": "vorlage", "ref": "rom:gibt-es-nicht" })).unwrap_err().code, "op-ungueltig");
        let v = d.apply(&json!({ "kind": "vorlage", "ref": eigen })).unwrap(); assert_eq!(v["charset"]["vorlage"], json!(eigen)); assert!(v["charset"]["herkunft"].is_object());
        assert!(v["charset"]["geaendert"].as_array().unwrap().iter().all(|g| *g == false), "frisch geladen: nichts weicht ab");
        d.apply(&json!({ "kind": "value", "v": 1 })).unwrap(); d.apply(&json!({ "kind": "frame", "i": 3 })).unwrap(); d.apply(&json!({ "kind": "clear" })).unwrap();
        let v = d.apply(&json!({ "kind": "stroke", "points": [[0, 0], [7, 7]] })).unwrap(); assert_eq!(v["charset"]["geaendert"][3], true);
        let v = d.apply(&json!({ "kind": "zeichen_original", "i": 3 })).unwrap(); assert_eq!(v["charset"]["geaendert"][3], false); assert_eq!(v["fi"], 3);
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["charset"]["geaendert"][3], true, "Zurücksetzen ist ein Verlaufsschritt");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn strich_ist_ein_verlaufsschritt_ohne_luecken() {
        let (dir, mut d) = kopie("strich");
        d.apply(&json!({ "kind": "value", "v": 3 })).unwrap();
        let v = d.apply(&json!({ "kind": "stroke", "points": [[0, 0], [11, 0], [11, 5]] })).unwrap();
        let px = |v: &Value, x: usize, y: usize| v["pixels"][y * 12 + x].as_u64().unwrap();
        for x in 0..12 { assert_eq!(px(&v, x, 0), 3, "Lücke bei x={x}"); } for y in 0..6 { assert_eq!(px(&v, 11, y), 3); }
        assert_eq!(v["dirty"], true);
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["dirty"], false, "ein Rückgängig nimmt den ganzen Strich zurück");
        assert_eq!(d.apply(&json!({ "kind": "undo" })).unwrap_err().code, "undo-leer");
        let v = d.apply(&json!({ "kind": "redo" })).unwrap(); assert_eq!(px(&v, 5, 0), 3);
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn strich_ohne_aenderung_macht_keinen_schritt() {
        let (dir, mut d) = kopie("leer");
        d.apply(&json!({ "kind": "stroke", "points": [[0, 0]], "erase": true })).unwrap(); // (0,0) ist schon durchsichtig
        assert_eq!(d.view()["canUndo"], false);
        assert_eq!(d.apply(&json!({ "kind": "stroke", "points": [[99, 99]] })).unwrap_err().code, "op-ungueltig");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn pipette_farbe_norm_und_speichern_rundlauf() {
        let (dir, mut d) = kopie("rund");
        let v = d.apply(&json!({ "kind": "pick", "x": 4, "y": 4 })).unwrap(); assert!(v["pv"].as_u64().unwrap() <= 3);
        let v = d.apply(&json!({ "kind": "color", "key": "sprite", "index": 5 })).unwrap(); assert_eq!(v["slots"][0]["index"], 5); assert_eq!(v["dirty"], true);
        assert_eq!(d.apply(&json!({ "kind": "color", "key": "gibtsnicht", "index": 5 })).unwrap_err().code, "op-ungueltig");
        let v = d.apply(&json!({ "kind": "norm", "norm": "ntsc" })).unwrap(); assert_eq!(v["normActive"], "ntsc");
        d.apply(&json!({ "kind": "stroke", "points": [[1, 1]] })).unwrap();
        let v = d.apply(&json!({ "kind": "save" })).unwrap(); assert_eq!(v["dirty"], false);
        let d2 = PixelDok::oeffnen(&dir, "hero.vcsprite", &wurzel().join("data"), None).unwrap(); assert_eq!(d2.view()["pixels"], d.view()["pixels"]); assert_eq!(d2.view()["slots"][0]["index"], 5);
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn formen_fuellen_auswahl_und_bildoperationen() {
        let (dir, mut d) = kopie("formen"); let px = |v: &Value, x: usize, y: usize| v["pixels"][y * 12 + x].as_u64().unwrap();
        d.apply(&json!({ "kind": "value", "v": 3 })).unwrap();
        let v = d.apply(&json!({ "kind": "shape", "tool": "line", "a": [0, 0], "b": [11, 20] })).unwrap(); assert_eq!(px(&v, 0, 0), 3); assert_eq!(px(&v, 11, 20), 3);
        let v = d.apply(&json!({ "kind": "shape", "tool": "rect", "a": [1, 1], "b": [4, 3] })).unwrap(); assert_eq!(px(&v, 1, 1), 3); assert_eq!(px(&v, 4, 3), 3);
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_ne!(px(&v, 4, 3), 3, "ein Schritt je Form"); assert_eq!(px(&v, 0, 0), 3);
        let v = d.apply(&json!({ "kind": "fill", "x": 11, "y": 0 })).unwrap(); assert_eq!(px(&v, 10, 0), 3);
        d.apply(&json!({ "kind": "select", "a": [0, 0], "b": [2, 2] })).unwrap();
        let v = d.apply(&json!({ "kind": "copy" })).unwrap(); assert_eq!(v["hasClip"], true); assert_eq!(v["note"]["ok"], true);
        let v = d.apply(&json!({ "kind": "paste", "x": 6, "y": 6 })).unwrap(); assert_eq!(v["sel"], json!([6, 6, 8, 8]));
        let v = d.apply(&json!({ "kind": "clear_selection" })).unwrap(); assert_eq!(px(&v, 6, 6), 0);
        d.apply(&json!({ "kind": "select_none" })).unwrap();
        assert_eq!(d.apply(&json!({ "kind": "copy" })).unwrap_err().code, "pixel-keine-auswahl");
        let v = d.apply(&json!({ "kind": "flip", "dir": "h" })).unwrap(); let vor = v["pixels"].clone();
        let v = d.apply(&json!({ "kind": "flip", "dir": "h" })).unwrap(); assert_ne!(v["pixels"], vor); let v = d.apply(&json!({ "kind": "flip", "dir": "v" })).unwrap(); let _ = v;
        let v = d.apply(&json!({ "kind": "shift", "dir": "r" })).unwrap(); assert!(v["dirty"] == true);
        let v = d.apply(&json!({ "kind": "clear" })).unwrap(); assert!(v["pixels"].as_array().unwrap().iter().all(|p| p == 0));
        assert_eq!(d.apply(&json!({ "kind": "shape", "tool": "kreis", "a": [0, 0], "b": [1, 1] })).unwrap_err().code, "op-ungueltig");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn bilder_modus_und_flags() {
        let (dir, mut d) = kopie("bilder"); let v = d.view(); let n0 = v["frames"].as_u64().unwrap();
        assert_eq!(v["thumbs"].as_array().unwrap().len() as u64, n0); assert!(v["modes"].is_array() && v["flags"].is_object(), "C64-Sprite: Modi und Flags"); assert!(v["scene"]["sky"].is_string());
        let v = d.apply(&json!({ "kind": "frame_new" })).unwrap(); assert_eq!(v["frames"].as_u64().unwrap(), n0 + 1); assert_eq!(v["fi"].as_u64().unwrap(), n0);
        let v = d.apply(&json!({ "kind": "frame_dup" })).unwrap(); assert_eq!(v["frames"].as_u64().unwrap(), n0 + 2);
        let v = d.apply(&json!({ "kind": "frame_delete" })).unwrap(); assert_eq!(v["frames"].as_u64().unwrap(), n0 + 1);
        // umordnen: das bewegte Bild steht danach vorn und ist gewählt; Rückgängig stellt die Reihenfolge wieder her
        let vor = d.view()["thumbs"][0]["pixels"].clone(); let letztes = n0 as usize;
        let v = d.apply(&json!({ "kind": "frame_move", "from": letztes, "to": 0 })).unwrap(); assert_eq!(v["fi"].as_u64().unwrap(), 0); assert_ne!(v["thumbs"][0]["pixels"], vor, "das bewegte Bild steht jetzt vorn");
        assert!(d.apply(&json!({ "kind": "frame_move", "from": 99, "to": 0 })).is_err());
        let v = d.undo().unwrap(); assert_eq!(v["thumbs"][0]["pixels"], vor, "Rückgängig stellt die Reihenfolge wieder her");
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["frames"].as_u64().unwrap(), n0 + 2, "Frame löschen ist ein Verlaufsschritt");
        let v = d.apply(&json!({ "kind": "frame", "i": 0 })).unwrap(); assert_eq!(v["fi"], 0);
        let v = d.apply(&json!({ "kind": "mode", "multicolor": false })).unwrap(); assert_eq!(v["width"], 24); assert_eq!(v["multicolor"], false); assert_eq!(v["values"].as_array().unwrap().len(), 2);
        let v = d.apply(&json!({ "kind": "mode", "multicolor": true })).unwrap(); assert_eq!(v["width"], 12);
        let v = d.apply(&json!({ "kind": "flag", "flag": "x2" })).unwrap(); assert_eq!(v["flags"]["x2"], true); assert_eq!(v["dirty"], true);
        let v = d.apply(&json!({ "kind": "flag", "flag": "x2", "on": false })).unwrap(); assert_eq!(v["flags"]["x2"], false);
        assert_eq!(d.apply(&json!({ "kind": "flag", "flag": "zz" })).unwrap_err().code, "op-ungueltig");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn pmg_hat_feste_bilder_und_keine_flags() {
        let dir = std::env::temp_dir().join(format!("vc_pixel_pmg_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(wurzel().join("examples/atari/pmg_ship/graphics/ship.vcpmg"), dir.join("ship.vcpmg")).unwrap();
        let mut d = PixelDok::oeffnen(&dir, "ship.vcpmg", &wurzel().join("data"), None).unwrap(); let v = d.view();
        assert_eq!(v["fixedFrames"], true); assert!(v["flags"].is_null() && v["modes"].is_null());
        assert_eq!(d.apply(&json!({ "kind": "frame_new" })).unwrap_err().code, "op-ungueltig");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn animation_schritte_ziehen_und_tempo_fuer_alle() {
        let (dir, mut d) = kopie("animzieh");
        d.apply(&json!({ "kind": "anim_from_frames", "frames": [0, 1], "name": "z", "mode": "loop", "delay": 4 })).unwrap();
        d.apply(&json!({ "kind": "anim_add_step", "frame": 0 })).unwrap();   // Schritte: 0, 1, 0
        let i = d.view()["selAnim"].as_u64().unwrap() as usize;
        let v = d.apply(&json!({ "kind": "anim_move_to", "from": 2, "to": 0 })).unwrap(); assert_eq!(v["anims"][i]["steps"], json!([[0, 4], [0, 4], [1, 4]]));
        let v = d.apply(&json!({ "kind": "anim_move_to", "from": 0, "to": 2 })).unwrap(); assert_eq!(v["anims"][i]["steps"], json!([[0, 4], [1, 4], [0, 4]]));
        let v = d.apply(&json!({ "kind": "anim_delay_all", "delay": 10 })).unwrap(); assert_eq!(v["anims"][i]["steps"], json!([[0, 10], [1, 10], [0, 10]]));
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["anims"][i]["steps"], json!([[0, 4], [1, 4], [0, 4]]), "Tempo für alle ist ein Schritt");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn animation_aus_ausgewaehlten_bildern() {
        let (dir, mut d) = kopie("animsel"); let v = d.view(); let n = v["anims"].as_array().unwrap().len(); let frames = v["frames"].as_u64().unwrap();
        assert!(frames >= 2, "Beispiel hat mindestens zwei Bilder");
        let v = d.apply(&json!({ "kind": "anim_from_frames", "frames": [1, 0, 1], "name": "", "mode": "pingpong", "delay": 8 })).unwrap();
        let a = &v["anims"][n]; assert_eq!(v["anims"].as_array().unwrap().len(), n + 1); assert_eq!(v["selAnim"], n);
        assert_eq!(a["mode"], "pingpong"); assert_eq!(a["steps"], json!([[0, 8], [1, 8]]), "Reihenfolge der Bilderliste, ohne Doppelte");
        let v = d.apply(&json!({ "kind": "anim_from_frames", "frames": [0], "name": "zwei", "mode": "once", "delay": 3 })).unwrap(); assert_eq!(v["anims"][n + 1]["name"], "zwei");
        assert_eq!(d.apply(&json!({ "kind": "anim_from_frames", "frames": [0], "name": "zwei" })).unwrap_err().code, "pixel-anim");
        assert_eq!(d.apply(&json!({ "kind": "anim_from_frames", "frames": [], "name": "" })).unwrap_err().code, "pixel-anim");
        assert_eq!(d.apply(&json!({ "kind": "anim_from_frames", "frames": [frames + 5], "name": "" })).unwrap_err().code, "pixel-anim");
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["anims"].as_array().unwrap().len(), n + 1, "Rückgängig nimmt eine ganze Animation zurück");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn animationen_und_png() {
        let (dir, mut d) = kopie("anim"); let v = d.view(); assert!(v["anims"].is_array() && v["animError"].is_null());
        let n = v["anims"].as_array().unwrap().len();
        let v = d.apply(&json!({ "kind": "anim_new" })).unwrap(); assert_eq!(v["anims"].as_array().unwrap().len(), n + 1); let i = n;
        let v = d.apply(&json!({ "kind": "anim_select", "i": i })).unwrap(); assert_eq!(v["selAnim"], i);
        let v = d.apply(&json!({ "kind": "anim_rename", "name": "laufen" })).unwrap(); assert_eq!(v["anims"][i]["name"], "laufen");
        assert_eq!(d.apply(&json!({ "kind": "anim_rename", "name": "1 falsch" })).unwrap_err().code, "pixel-anim");
        d.apply(&json!({ "kind": "frame_dup" })).unwrap();
        let v = d.apply(&json!({ "kind": "anim_add_step", "frame": 1 })).unwrap(); assert_eq!(v["anims"][i]["steps"].as_array().unwrap().len(), 2);
        let v = d.apply(&json!({ "kind": "anim_delay", "i": 1, "delay": 9 })).unwrap(); assert_eq!(v["anims"][i]["steps"][1][1], 9);
        let v = d.apply(&json!({ "kind": "anim_move", "i": 1, "dir": -1 })).unwrap(); assert_eq!(v["anims"][i]["steps"][0][1], 9);
        let v = d.apply(&json!({ "kind": "anim_mode", "mode": "pingpong" })).unwrap(); assert_eq!(v["anims"][i]["mode"], "pingpong");
        let v = d.apply(&json!({ "kind": "anim_del_step", "i": 0 })).unwrap(); assert_eq!(v["anims"][i]["steps"].as_array().unwrap().len(), 1);
        let v = d.apply(&json!({ "kind": "anim_delete" })).unwrap(); assert_eq!(v["anims"].as_array().unwrap().len(), n);
        let v = d.apply(&json!({ "kind": "undo" })).unwrap(); assert_eq!(v["anims"].as_array().unwrap().len(), n + 1, "Löschen einer Animation ist ein Verlaufsschritt");
        let (w, h) = (24usize, 21usize); let rgba: Vec<u8> = (0..w * h).flat_map(|i| if i % w < 8 && i / w < 5 { [0x88, 0x39, 0x32, 255] } else { [0, 0, 0, 0] }).collect();
        std::fs::write(dir.join("a.png"), vc_media::png::encode_rgba(w, h, &rgba)).unwrap();
        let v = d.apply(&json!({ "kind": "frame", "i": 0 })).unwrap(); let vor = v["pixels"].clone();
        let v = d.apply(&json!({ "kind": "png", "path": dir.join("a.png").to_string_lossy() })).unwrap(); assert_ne!(v["pixels"], vor); assert_eq!(v["note"]["ok"], true);
        assert_eq!(d.apply(&json!({ "kind": "png", "path": dir.join("gibtsnicht.png").to_string_lossy() })).unwrap_err().code, "pixel-png");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn atari_breite_prioritaet_und_export() {
        let dir = std::env::temp_dir().join(format!("vc_pixel_pmg2_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(wurzel().join("examples/atari/pmg_ship/graphics/ship.vcpmg"), dir.join("ship.vcpmg")).unwrap();
        let mut d = PixelDok::oeffnen(&dir, "ship.vcpmg", &wurzel().join("data"), None).unwrap();
        let v = d.view(); assert_eq!(v["pmg"]["sizes"], json!([0, 0, 0, 0])); assert_eq!(v["pmg"]["prior"], 0);
        let q0 = d.export_quelle().unwrap(); let vorher = vorschau_asm(&dir, &q0); for k in ["ship_sizep0 = 0", "ship_sizep1 = 0", "ship_sizep2 = 0", "ship_sizep3 = 0", "ship_prior = 1"] { assert!(vorher.contains(k), "auch ohne Werte alle Konstanten: {k} in {vorher}"); }
        let v = d.apply(&json!({ "kind": "pmg_size", "i": 1, "size": 3 })).unwrap(); assert_eq!(v["pmg"]["sizes"], json!([0, 3, 0, 0])); assert_eq!(v["dirty"], true);
        let v = d.apply(&json!({ "kind": "pmg_prior", "prior": 4 })).unwrap(); assert_eq!(v["pmg"]["prior"], 4);
        assert_eq!(d.apply(&json!({ "kind": "pmg_size", "i": 1, "size": 2 })).unwrap_err().code, "op-ungueltig");
        assert_eq!(d.apply(&json!({ "kind": "pmg_prior", "prior": 3 })).unwrap_err().code, "op-ungueltig");
        let asm = vorschau_asm(&dir, &d.export_quelle().unwrap()); assert!(["ship_sizep0 = 0", "ship_sizep1 = 3", "ship_sizep2 = 0", "ship_sizep3 = 0", "ship_prior = 4"].iter().all(|k| asm.contains(k)), "{asm}");
        d.save().unwrap(); let d2 = PixelDok::oeffnen(&dir, "ship.vcpmg", &wurzel().join("data"), None).unwrap(); assert_eq!(d2.view()["pmg"]["sizes"], json!([0, 3, 0, 0])); let asm2 = vorschau_asm(&dir, &d2.export_quelle().unwrap()); assert!(asm2.contains("ship_sizep1 = 3") && asm2.contains("ship_prior = 4"), "nach Speichern und Neuöffnen: {asm2}");
        let v = d.apply(&json!({ "kind": "pmg_size", "i": 1, "size": 0 })).unwrap(); let v = { let mut d3 = d; d3.apply(&json!({ "kind": "pmg_prior", "prior": 0 })).unwrap(); d3.view() }; assert!(v["pmg"]["sizes"] == json!([0, 0, 0, 0]));
        std::fs::remove_dir_all(dir).ok();
    }
    fn vorschau_asm(dir: &std::path::Path, q: &crate::docstore::ExportQuelle) -> String { crate::export::kopiertext(dir, &wurzel().join("data"), q, false, "ship", "", false).unwrap() }
    #[test] fn unveraenderte_datei_schreibt_byte_gleich_zurueck() {
        // Rundlauf: öffnen und speichern ohne Änderung liefert dieselben Bytes wie die formatierte Mediendatei (zweites Speichern ändert nichts mehr)
        let (dir, mut d) = kopie("byte"); d.save().unwrap(); let a = std::fs::read(dir.join("hero.vcsprite")).unwrap();
        let mut d2 = PixelDok::oeffnen(&dir, "hero.vcsprite", &wurzel().join("data"), None).unwrap(); d2.save().unwrap(); assert_eq!(a, std::fs::read(dir.join("hero.vcsprite")).unwrap());
        std::fs::remove_dir_all(dir).ok();
    }
}
