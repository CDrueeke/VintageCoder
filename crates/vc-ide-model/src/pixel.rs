// SPDX-License-Identifier: GPL-3.0-or-later
//! Modell des Pixel-Editors für Sprites, Player/Missile-Grafik und Zeichensätze (1, 2 und 8 Bit je Pixel), **ohne Oberfläche**
//! (Stufe 3, Studios): Rahmen mit Pixelwerten, Modus je Frame (Hires/Multicolor), Undo/Redo als JSON-Stände, Zeichenwerkzeuge
//! (Füllen, Linie, Rechteck, Auswahl, Kopieren/Einfügen, Spiegeln/Verschieben), Animationen, Hardware-Flags, Laden und Speichern.
//! Die Tests liegen am Ende der Datei.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_media::sprite::{self, Anim, AnimMode};

/// Modus eines Sprite-Frames aus dem Profil (`modes`)
#[derive(Clone, Debug)]
pub struct ModeInfo { pub label: String, pub hint: String, pub width: usize, pub bpp: usize, pub aspect: f32, pub values: Vec<String> }

/// Zeichenwerkzeuge der Zeichenfläche
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool { Pen, Erase, Fill, Line, Rect, Select, Pick }

/// Änderungen an den Animationen eines Sprites
#[derive(Clone, Debug, PartialEq)]
pub enum AnimOp { New, Delete, Rename(String), Mode(AnimMode), AddStep(usize), DelStep(usize), Move(usize, isize), Delay(usize, u32) }

#[derive(Debug)]
pub struct PixelDoc {
    pub rel: String, pub profile: String, abs: PathBuf, pub media: Value, pub kind: String, pub target: String,
    pub values: Vec<String>, pub frame_names: Vec<String>, pub width: usize, pub height: usize, pub bpp: usize, pub aspect: f32,
    pub fixed_frames: bool, pub frames: Vec<Vec<Vec<u8>>>, pub fi: usize, pub pv: u8, pub tool: Tool,
    undo: Vec<String>, redo: Vec<String>, saved: String,
    pub sel: Option<(usize, usize, usize, usize)>, pub clip: Option<Vec<Vec<u8>>>,
    /// Sprites: Multicolor-Flag je Frame und Modus des Profils
    pub frame_mc: Vec<bool>, pub profile_mc: bool,
    /// Betriebsarten je Frame (leer, wenn das Profil keinen Modus je Frame kennt)
    pub frame_modes: Vec<ModeInfo>,
    /// Gewählte Animation
    pub sel_anim: usize,
    /// Zusammenfassen von Undo-Schritten (z. B. Dauer eines Schritts)
    coalesce: Option<String>,
    pub status: String,
}

impl PixelDoc {
    pub fn open(root: &Path, rel: &str, data: &Path) -> Result<PixelDoc, String> {
        let abs = root.join(rel);
        let mut media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| format!("{rel}: {e}"))?).map_err(|e| format!("{rel}: {e}"))?;
        // Format 2: Verweis auf ein ROM des Anwenders (`base`) auflösen, die eigenen Änderungen (`overrides`) darüberlegen
        if media["format"] == 2 && media["base"].is_string() { media = vc_media::romref::resolve_media(&media).map_err(|e| format!("{rel}: {e}"))?; }
        let pid = media["profile"].as_str().ok_or("Profil fehlt")?;
        let profiles = vc_media::load_profiles(data)?;
        let p = vc_media::profile(&profiles, pid)?;
        if p.bpp > 8 { return Err(format!("{rel}: {} Bit je Pixel werden nicht unterstützt", p.bpp)); }
        let kind = media["kind"].as_str().unwrap_or("").to_string();
        let height = if kind == "pmg" { media["height"].as_u64().map(|h| h as usize).unwrap_or(p.height) } else { p.height };
        let digits = p.raw["digits"].as_u64().unwrap_or(1) as usize;
        let frames: Vec<Vec<Vec<u8>>> = media["frames"].as_array().ok_or("frames fehlt")?.iter().map(|f| {
            f["rows"].as_array().map(|rows| rows.iter().map(|r| r.as_str().unwrap_or("").chars().collect::<Vec<_>>().chunks(digits).map(|c| c.iter().collect::<String>()).map(|t| u8::from_str_radix(&t, 16).unwrap_or(0)).collect()).collect()).unwrap_or_default()
        }).collect();
        let strs = |k: &str| p.raw[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut b = PixelDoc { rel: rel.into(), profile: pid.to_string(), abs, kind: kind.clone(), target: p.target.clone(), values: strs("values"), frame_names: strs("frameNames"), width: p.width, height, bpp: p.bpp,
            aspect: p.raw["aspect"].as_f64().unwrap_or(1.0) as f32, fixed_frames: kind != "sprite", frames, fi: 0, pv: 1, tool: Tool::Pen,
            undo: vec![], redo: vec![], saved: String::new(), sel: None, clip: None, frame_mc: vec![], profile_mc: p.bpp == 2, frame_modes: if vc_media::sprite::has_frame_modes(p) { p.raw["modes"].as_array().map(|a| a.iter().map(|m| ModeInfo { label: m["label"].as_str().unwrap_or("").into(), hint: m["hint"].as_str().unwrap_or("").into(), width: m["width"].as_u64().unwrap_or(0) as usize, bpp: m["bpp"].as_u64().unwrap_or(1) as usize, aspect: m["aspect"].as_f64().unwrap_or(1.0) as f32, values: m["values"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }).collect()).unwrap_or_default() } else { vec![] }, sel_anim: 0, coalesce: None, status: String::new(), media };
        b.read_modes(); b.sync_mode();
        b.saved = b.snapshot_json();
        if let Some(m) = b.media["rom_missing"].as_str() { b.status = format!("Zeichensatz-ROM fehlt – leeres Raster. {m}"); }
        else if let Some(n) = b.media["rom_note"].as_str() { b.status = n.to_string(); }
        Ok(b)
    }

    /// Zeichensatz-Vorlagen aus den ROMs des Anwenders zur Auswahl (rückgängig mit Strg+Z); der Standard des Systems steht vorn.
    fn snapshot_json(&self) -> String { self.to_value().to_string() }
    fn to_value(&self) -> Value {
        let mut a = self.media.clone();
        let digits = if self.bpp > 4 { 2 } else { 1 };
        a["frames"] = Value::Array(self.frames.iter().enumerate().map(|(i, f)| {
            let mut fr = json!({ "rows": f.iter().map(|r| r.iter().map(|v| format!("{:0w$x}", v, w = digits)).collect::<String>()).collect::<Vec<_>>() });
            // Nur abweichende Frames bekommen einen eigenen Modus (alte Dateien bleiben unverändert)
            if self.kind == "sprite" { if let Some(&mc) = self.frame_mc.get(i) { if mc != self.profile_mc { fr = json!({ "mode": if mc { "multicolor" } else { "hires" }, "rows": fr["rows"].clone() }); } } }
            fr
        }).collect());
        a
    }
    /// Multicolor-Flags der Frames aus dem Media lesen
    fn read_modes(&mut self) {
        if self.kind != "sprite" { self.frame_mc.clear(); return; }
        self.frame_mc = self.media["frames"].as_array().map(|fs| fs.iter().map(|f| match f["mode"].as_str() { Some("multicolor") => true, Some("hires") => false, _ => self.profile_mc }).collect()).unwrap_or_default();
        self.frame_mc.resize(self.frames.len(), self.profile_mc);
    }
    /// Breite, Bits je Pixel, Seitenverhältnis und Wertnamen richten sich bei Sprites nach dem Modus des gewählten Frames
    fn sync_mode(&mut self) {
        if self.kind != "sprite" || self.frame_modes.len() < 2 { return; }
        let mc = self.frame_mc.get(self.fi).copied().unwrap_or(self.profile_mc);
        let m = &self.frame_modes[if mc { 1 } else { 0 }];
        self.width = m.width; self.bpp = m.bpp; self.aspect = m.aspect; self.values = m.values.clone();
        if self.pv as usize >= (1 << self.bpp) { self.pv = 1; }
    }
    fn is_mc(&self, i: usize) -> bool { self.frame_mc.get(i).copied().unwrap_or(self.profile_mc) }
    /// Modus des aktuellen Frames wechseln und die Pixel umrechnen (rückgängig mit Strg+Z)
    fn set_frame_mode(&mut self, to_mc: bool) {
        if self.kind != "sprite" || self.is_mc(self.fi) == to_mc { return; }
        self.push_undo();
        let f = std::mem::take(&mut self.frames[self.fi]);
        self.frames[self.fi] = if to_mc { sprite::hires_to_mc(&f) } else { sprite::mc_to_hires(&f) };
        if self.frame_mc.len() <= self.fi { self.frame_mc.resize(self.fi + 1, self.profile_mc); }
        self.frame_mc[self.fi] = to_mc;
        if to_mc { for (k, d) in [("sprite", 7u64), ("mc1", 2), ("mc2", 6)] { if self.media["colors"][k].is_null() { self.media["colors"][k] = json!(d); } } }
        self.sync_mode();
    }
    fn restore(&mut self, s: &str) {
        if let Ok(v) = serde_json::from_str::<Value>(s) {
            let digits = if self.bpp > 4 { 2 } else { 1 };
            self.frames = v["frames"].as_array().map(|fs| fs.iter().map(|f| f["rows"].as_array().map(|rows| rows.iter().map(|r| r.as_str().unwrap_or("").chars().collect::<Vec<_>>().chunks(digits).map(|c| u8::from_str_radix(&c.iter().collect::<String>(), 16).unwrap_or(0)).collect()).collect()).unwrap_or_default()).collect()).unwrap_or_default();
            self.media = v; self.fi = self.fi.min(self.frames.len().saturating_sub(1));
            self.read_modes(); self.sync_mode();
        }
    }
    #[allow(dead_code)] fn do_undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.snapshot_json()); self.restore(&s); } }
    #[allow(dead_code)] fn do_redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.snapshot_json()); self.restore(&s); } }

    pub fn save(&mut self) -> Result<(), String> {
        let v = self.to_value();
        // mit `base` auf ein ROM werden nur die eigenen Änderungen geschrieben (Format 2), nie die Original-Zeichen
        std::fs::write(&self.abs, vc_media::project::format_media(&vc_media::romref::for_saving(&v))).map_err(|e| format!("{}: {e}", self.rel))?;
        self.saved = v.to_string(); self.status = "gespeichert".into(); Ok(())
    }

    fn colors(&self) -> &Value { &self.media["colors"] }
    fn cnum(&self, key: &str, default: usize) -> usize { self.colors()[key].as_u64().map(|v| v as usize).unwrap_or(default) }
    fn color_keys(&self) -> Vec<(String, String)> {
        if self.bpp == 8 { return vec![]; }
        match self.kind.as_str() {
            "sprite" => if self.bpp == 1 && !self.frame_mc.iter().any(|&m| m) { vec![("sprite".into(), "Sprite-Farbe".into())] } else { vec![("sprite".into(), "Sprite-Farbe".into()), ("mc1".into(), "Multicolor 1".into()), ("mc2".into(), "Multicolor 2".into())] },
            "pmg" => vec![(format!("P{}", self.fi), format!("Farbe {}", self.frame_names.get(self.fi).cloned().unwrap_or_default()))],
            _ => if self.bpp == 1 { vec![("fg".into(), "Vordergrund".into()), ("bg".into(), "Hintergrund".into())] } else {
                // Pixelwerte 0..3 gehören zu bg, mc1, mc2, fg; die Beschriftung kommt aus den Werten des Profils (VC-20: Hintergrund, Rahmen, Zeichenfarbe, Hilfsfarbe)
                let l = |i: usize, d: &str| self.values.get(i).cloned().unwrap_or_else(|| d.to_string());
                vec![("fg".into(), l(3, "Zeichenfarbe")), ("bg".into(), l(0, "Hintergrund")), ("mc1".into(), l(1, "Multicolor 1")), ("mc2".into(), l(2, "Multicolor 2"))]
            },
        }
    }
    fn set(&mut self, x: usize, y: usize, v: u8) { if let Some(r) = self.frames.get_mut(self.fi).and_then(|f| f.get_mut(y)) { if let Some(c) = r.get_mut(x) { *c = v; } } }
    fn get(&self, x: usize, y: usize) -> u8 { self.frames.get(self.fi).and_then(|f| f.get(y)).and_then(|r| r.get(x)).copied().unwrap_or(0) }

    fn flood(&mut self, x: usize, y: usize, v: u8) {
        let from = self.get(x, y); if from == v { return; }
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if self.get(cx, cy) != from { continue; } self.set(cx, cy, v);
            if cx > 0 { stack.push((cx - 1, cy)); } if cy > 0 { stack.push((cx, cy - 1)); }
            if cx + 1 < self.width { stack.push((cx + 1, cy)); } if cy + 1 < self.height { stack.push((cx, cy + 1)); }
        }
    }
    fn line(&mut self, a: (usize, usize), b: (usize, usize), v: u8) {
        let (mut x0, mut y0, x1, y1) = (a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs()); let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut err = dx + dy;
        loop { self.set(x0 as usize, y0 as usize, v); if x0 == x1 && y0 == y1 { break; } let e2 = 2 * err; if e2 >= dy { err += dy; x0 += sx; } if e2 <= dx { err += dx; y0 += sy; } }
    }
    fn rect_outline(&mut self, a: (usize, usize), b: (usize, usize), v: u8) {
        let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1));
        for x in x0..=x1 { self.set(x, y0, v); self.set(x, y1, v); } for y in y0..=y1 { self.set(x0, y, v); self.set(x1, y, v); }
    }
    fn sel_copy(&mut self, cut: bool) {
        let Some((x0, y0, x1, y1)) = self.sel else { return };
        let buf: Vec<Vec<u8>> = (y0..=y1).map(|y| (x0..=x1).map(|x| self.get(x, y)).collect()).collect(); self.clip = Some(buf);
        if cut { self.push_undo(); for y in y0..=y1 { for x in x0..=x1 { self.set(x, y, 0); } } }
    }
    fn sel_paste(&mut self) {
        let Some(c) = self.clip.clone() else { return }; self.push_undo();
        let (ox, oy) = self.sel.map_or((0, 0), |s| (s.0, s.1));
        for (dy, row) in c.iter().enumerate() { for (dx, v) in row.iter().enumerate() { let (x, y) = (ox + dx, oy + dy); if x < self.width && y < self.height { self.set(x, y, *v); } } }
        self.sel = Some((ox, oy, (ox + c[0].len() - 1).min(self.width - 1), (oy + c.len() - 1).min(self.height - 1)));
    }
    fn sel_clear(&mut self) { if let Some((x0, y0, x1, y1)) = self.sel { self.push_undo(); for y in y0..=y1 { for x in x0..=x1 { self.set(x, y, 0); } } } }
    fn op(&mut self, which: &str) {
        self.push_undo();
        let (w, h) = (self.width, self.height); let f = &mut self.frames[self.fi];
        match which {
            "mh" => f.iter_mut().for_each(|r| r.reverse()),
            "mv" => f.reverse(),
            "sl" => f.iter_mut().for_each(|r| r.rotate_left(1)),
            "sr" => f.iter_mut().for_each(|r| r.rotate_right(1)),
            "su" => f.rotate_left(1),
            "sd" => f.rotate_right(1),
            "clear" => { for r in f.iter_mut() { for c in r.iter_mut() { *c = 0; } } }
            _ => {}
        }
        let _ = (w, h);
    }


    /// Palettenindex (im Farbwerk des Ziels) eines Pixelwerts; `None` = transparent
    pub fn farb_index(&self, v: u8) -> Option<usize> { self.farb_index_frame(self.fi, v) }
    /// Wie `farb_index`, aber für Frame `fi` (Vorschau, Onion Skin): Spielerfarbe bei Player/Missile und Modus (Hires/Multicolor) des Sprites gehören dem jeweiligen Frame, nicht dem gewählten
    pub fn farb_index_frame(&self, fi: usize, v: u8) -> Option<usize> {
        let vi = v as usize;
        if self.bpp == 8 { return Some(vi) }
        match self.kind.as_str() {
            // Hires: Pixel = Sprite-Farbe. Mehrfarb: Bitpaare wie in der Hardware (01 = Multicolor 1 $D025, 10 = Sprite-Farbe, 11 = Multicolor 2 $D026)
            "sprite" => if v == 0 { None } else { Some(self.cnum(if !self.is_mc(fi) || v == 2 { "sprite" } else if v == 1 { "mc1" } else { "mc2" }, 1)) },
            "pmg" => if v == 0 { None } else { Some(self.cnum(&format!("P{fi}"), 14)) },
            _ => Some(if v == 0 { self.cnum("bg", 0) } else if v == 1 { if self.bpp == 2 { self.cnum("mc1", 2) } else { self.cnum("fg", 1) } } else if v == 2 { self.cnum("mc2", 6) } else { self.cnum("fg", 1) }),
        }
    }
    /// Aktueller (auch ungespeicherter) Stand als Text der Mediendatei, wie er gespeichert würde (für die Export-Vorschau)
    pub fn media_text(&self) -> String { vc_media::project::format_media(&vc_media::romref::for_saving(&self.to_value())) }
    /// Sind ungespeicherte Änderungen da?
    pub fn dirty(&self) -> bool { self.snapshot_json() != self.saved }
    /// Vor einer Änderung aufrufen: merkt den Stand für „Rückgängig“
    pub fn push_undo(&mut self) { self.undo.push(self.snapshot_json()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear(); self.coalesce = None; }
    pub fn undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.snapshot_json()); self.restore(&s); } }
    pub fn redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.snapshot_json()); self.restore(&s); } }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    /// Ein Pixel des gewählten Frames setzen (ohne Undo: der Aufrufer ruft vor dem Strich `push_undo`)
    pub fn setze(&mut self, x: usize, y: usize, v: u8) { self.set(x, y, v) }
    pub fn pixel(&self, x: usize, y: usize) -> u8 { self.get(x, y) }
    pub fn fuelle(&mut self, x: usize, y: usize, v: u8) { self.flood(x, y, v) }
    pub fn linie(&mut self, a: (usize, usize), b: (usize, usize), v: u8) { self.line(a, b, v) }
    pub fn rahmen(&mut self, a: (usize, usize), b: (usize, usize), v: u8) { self.rect_outline(a, b, v) }
    pub fn auswahl_kopieren(&mut self, ausschneiden: bool) { self.sel_copy(ausschneiden) }
    pub fn auswahl_einfuegen(&mut self) { self.sel_paste() }
    pub fn auswahl_loeschen(&mut self) { self.sel_clear() }
    /// Bildoperation auf dem gewählten Frame: „mh“ horizontal spiegeln, „mv“ vertikal, „sl/sr/su/sd“ verschieben (Links, Rechts, Hoch, Runter), „clear“ leeren
    pub fn operation(&mut self, was: &str) { self.op(was) }
    pub fn modus_wechseln(&mut self, multicolor: bool) { self.set_frame_mode(multicolor) }
    pub fn ist_multicolor(&self, i: usize) -> bool { self.is_mc(i) }
    pub fn frame_waehlen(&mut self, i: usize) { if i < self.frames.len() { self.fi = i; self.sync_mode(); if self.sel.is_some_and(|s| s.2 >= self.width || s.3 >= self.height) { self.sel = None } } }
    pub fn farben(&self) -> Vec<(String, String)> { self.color_keys() }
    pub fn farbe(&self, key: &str, default: usize) -> usize { self.cnum(key, default) }
    /// Farbe (Palettenindex) einer Farbrolle setzen (`sprite`, `mc1`, `mc2`, `fg`, `bg`, `P0` …)
    pub fn farbe_setzen(&mut self, key: &str, index: usize) { self.push_undo(); self.media["colors"][key] = json!(index); }

    /// Leeres Frame anhängen (im Modus des gewählten Frames)
    pub fn frame_anhaengen(&mut self) {
        if self.fixed_frames { return }
        self.push_undo(); let mc = self.is_mc(self.fi);
        self.frames.push(vec![vec![0; self.width]; self.height]); self.frame_mc.resize(self.frames.len() - 1, self.profile_mc); self.frame_mc.push(mc);
        self.fi = self.frames.len() - 1;
    }
    /// Gewähltes Frame duplizieren (Schritte der Animationen verschieben sich)
    pub fn frame_duplizieren(&mut self) {
        if self.fixed_frames { return }
        self.push_undo(); let f = self.frames[self.fi].clone(); let mc = self.is_mc(self.fi);
        self.frames.insert(self.fi + 1, f); self.frame_mc.resize(self.frames.len() - 1, self.profile_mc); self.frame_mc.insert(self.fi + 1, mc);
        sprite::animations_after_insert(&mut self.media, self.fi + 1); self.fi += 1;
    }
    /// Gewähltes Frame löschen (Schritte der Animationen mit diesem Frame entfallen); das letzte Frame bleibt
    pub fn frame_loeschen(&mut self) {
        if self.fixed_frames || self.frames.len() < 2 { return }
        self.push_undo(); self.frames.remove(self.fi); if self.fi < self.frame_mc.len() { self.frame_mc.remove(self.fi); }
        sprite::animations_after_remove(&mut self.media, self.fi); self.fi = self.fi.min(self.frames.len() - 1); self.sync_mode();
    }

    fn anim_view(&self) -> Value { json!({ "frames": vec![Value::Null; self.frames.len()], "animations": self.media["animations"].clone() }) }
    /// Animationen des Sprites
    pub fn animationen(&self) -> Result<Vec<Anim>, String> { sprite::parse_animations(&self.anim_view()) }
    /// Eine Änderung an den Animationen anwenden (Dauer-Änderungen am selben Schritt zählen als ein Undo-Schritt)
    pub fn anim_aendern(&mut self, op: AnimOp) -> Result<(), String> {
        let anims = self.animationen()?; let n = self.sel_anim.min(anims.len().saturating_sub(1));
        let key = if let AnimOp::Delay(i, _) = &op { Some(format!("delay:{n}:{i}")) } else { None };
        if key.is_none() || self.coalesce != key { self.push_undo(); } self.coalesce = key;
        let mut list = anims;
        match op {
            AnimOp::New => { let mut nm = "anim".to_string(); let mut k = 1; while list.iter().any(|x| x.name == nm) { k += 1; nm = format!("anim{k}"); } list.push(Anim { name: nm, mode: AnimMode::Loop, steps: vec![(self.fi, 4)] }); self.sel_anim = list.len() - 1; }
            AnimOp::Delete => { if n < list.len() { list.remove(n); self.sel_anim = n.saturating_sub(1); } }
            AnimOp::Rename(nm) => { if !sprite::valid_name(&nm) || list.iter().enumerate().any(|(i, x)| i != n && x.name == nm) { return Err(format!("„{nm}“ ist kein gültiger oder freier Name")) } if let Some(a) = list.get_mut(n) { a.name = nm; } }
            AnimOp::Mode(m) => { if let Some(a) = list.get_mut(n) { a.mode = m; } }
            AnimOp::AddStep(f) => { if let Some(a) = list.get_mut(n) { let d = a.steps.last().map_or(4, |s| s.1); a.steps.push((f, d)); } }
            AnimOp::DelStep(i) => { if let Some(a) = list.get_mut(n) { if i < a.steps.len() { a.steps.remove(i); } } }
            AnimOp::Move(i, dir) => { if let Some(a) = list.get_mut(n) { let j = i as isize + dir; if j >= 0 && (j as usize) < a.steps.len() { a.steps.swap(i, j as usize); } } }
            AnimOp::Delay(i, d) => { if let Some(a) = list.get_mut(n) { if let Some(s) = a.steps.get_mut(i) { s.1 = d.clamp(1, 255); } } }
        }
        self.media["animations"] = sprite::animations_to_json(&list); Ok(())
    }
    /// Zeichensatz-Vorlagen zum Profil: (Verweis, Beschriftung, nicht verfügbar). Reihenfolge: ROM des Anwenders (wenn vorhanden), **Standard (eigen)** (`std:<id>`), ROM fehlt (ausgegraut).
    /// Eigene Sätze werden beim Laden als Kopie eingefügt (kein Verweis in der Datei), ROM-Sätze als Verweis.
    pub fn vorlagen(&self, data: &Path) -> Vec<(String, String, bool)> {
        let rom: Vec<(String, String, bool)> = vc_media::project::std_charsets_for_profile(data, &self.profile, false).into_iter().enumerate().map(|(i, c)| (c.reference.clone(), format!("{}{}", c.label, if i == 0 { "  (Standard)" } else { "" }), c.status == vc_media::Status::Missing)).collect();
        let eigen: Vec<(String, String, bool)> = vc_media::project::eigene_saetze(data, &self.profile).into_iter().map(|e| (format!("std:{}", e.id), format!("Standard (eigen): {}", e.name), false)).collect();
        let (mut da, mut fehlt): (Vec<_>, Vec<_>) = rom.into_iter().partition(|r| !r.2);
        da.extend(eigen); da.append(&mut fehlt); da
    }
    /// Aktuelle Vorlage: `rom:…` (`base`), `std:<id>` für einen eigenen Satz, leer wenn der Zeichensatz selbst gestaltet ist
    pub fn vorlage_aktuell(&self) -> String {
        if let Some(b) = self.media["base"].as_str().filter(|b| !b.is_empty()) { return b.to_string() }
        match (self.media["origin"]["source"].as_str(), self.media["origin"]["id"].as_str()) { (Some("vintagecoder-standard"), Some(id)) => format!("std:{id}"), _ => String::new() }
    }
    /// Zeichensatz-Vorlage laden (rückgängig mit Undo); ROM-Sätze als Verweis (die Original-Zeichen stehen nicht in der Datei), eigene Sätze (`std:<id>`) als Kopie mit `origin`
    pub fn vorlage_anwenden_mit(&mut self, data: &Path, reference: &str) -> Result<(), String> {
        if let Some(id) = reference.strip_prefix("std:") {
            let std = vc_media::project::eigener_satz_media(data, id)?;
            if std["frames"].as_array().map_or(0, |f| f.len()) != self.frames.len() { return Err("Der eigene Satz passt nicht zur Zeichenzahl dieses Profils".into()) }
            self.push_undo();
            let mut v = self.to_value(); v["frames"] = std["frames"].clone(); v["origin"] = std["origin"].clone();
            if let Some(o) = v.as_object_mut() { for k in ["base", "rom_missing", "rom_note", "rom_status"] { o.remove(k); } }
            self.restore(&v.to_string()); self.status = "Eigenen Standardsatz geladen – mit Rückgängig zurück".into(); return Ok(())
        }
        self.vorlage_anwenden(reference)
    }
    pub fn vorlage_anwenden(&mut self, reference: &str) -> Result<(), String> {
        let std = vc_media::project::std_charset_media(&self.profile, reference)?;
        if std["frames"].as_array().map_or(0, |f| f.len()) != self.frames.len() { return Err("Vorlage passt nicht zur Zeichenzahl dieses Profils".into()) }
        self.push_undo();
        let mut v = self.to_value(); v["frames"] = std["frames"].clone(); v["base"] = json!(reference);
        if let Some(o) = v.as_object_mut() { o.remove("origin"); }
        for k in ["rom_missing", "rom_note", "rom_status"] { if let Some(x) = std.get(k) { v[k] = x.clone(); } else if let Some(o) = v.as_object_mut() { o.remove(k); } }
        self.restore(&v.to_string()); self.status = "Vorlage geladen – mit Rückgängig zurück".into(); Ok(())
    }
    /// Herkunft des Zeichensatzes für die Herkunftskarte: (Etikett, Zeilen). Etiketten immer als Wort: EIGEN, EIGEN, GEÄNDERT, AUS DEINEM ROM, ROM FEHLT; `None`, wenn der Satz selbst gestaltet ist
    pub fn herkunft(&self, data: &Path) -> Option<(&'static str, Vec<String>)> {
        if let Some(b) = self.media["base"].as_str().filter(|b| b.starts_with("rom:")) {
            return Some(if self.media["rom_missing"].is_string() { ("ROM FEHLT", vec![format!("Verweis {b}: das ROM liegt auf diesem Rechner nicht vor; der Bau nimmt ersatzweise den eigenen Standardsatz.")]) } else { ("AUS DEINEM ROM", vec![format!("Verweis {b}: die Zeichen gehören dem Hersteller, in der Datei steht nur der Verweis.")]) })
        }
        let o = &self.media["origin"]; if o["source"].as_str() != Some("vintagecoder-standard") { return None }
        let id = o["id"].as_str().unwrap_or("");
        let jetzt = self.to_value(); let unveraendert = vc_media::project::eigener_satz_media(data, id).ok().is_some_and(|m| m["frames"] == jetzt["frames"]);
        let mut z = vec![format!("Satz {id}, Revision {}", o["rev"].as_u64().unwrap_or(0)), format!("Urheber: {}", o["author"].as_str().unwrap_or("")), format!("Entstanden: {} · Werkzeug: {}", o["created"].as_str().unwrap_or(""), o["tool"].as_str().unwrap_or("")), format!("Lizenz: {}", o["license"].as_str().unwrap_or(""))];
        let pr = o["check"]["result"].as_str().unwrap_or("offen"); let fr = o["freigabe"].as_str().unwrap_or("offen"); z.push(format!("Prüfung der Abstände: {pr} · Freigabe: {fr}{}", if pr != "bestanden" && fr != "offen" { " (Abstandsprüfung nicht bestanden, vom Inhaber akzeptiert)" } else { "" }));
        Some((if unveraendert { "EIGEN" } else { "EIGEN, GEÄNDERT" }, z))
    }
    /// PNG-Bild in das gewählte Frame übernehmen (Farben werden zugeordnet); speichert vorher. Meldung zum Ergebnis.
    pub fn png_importieren(&mut self, root: &Path, data: &Path, png: &[u8]) -> Result<String, String> {
        if self.kind == "sprite" && self.is_mc(self.fi) != self.profile_mc { return Err("PNG laden gilt für Frames im Modus des Profils; dieses Frame hat einen anderen Modus".into()) }
        self.save()?;
        let v = vc_media::project::import_png(root, data, &self.rel, png, self.fi, true, "pal")?;
        self.undo.push(self.snapshot_json()); self.redo.clear(); self.coalesce = None;
        let (media, rep) = (v["media"].to_string(), &v["report"]);
        self.restore(&media); self.saved = self.snapshot_json();
        let msg = format!("PNG importiert in Frame {} ({} Bildfarben, {} erlaubt{})", self.fi, rep["imageColors"], rep["allowed"], if rep["conflict"] == true { " – Farben reduziert" } else { "" });
        self.status = msg.clone(); Ok(msg)
    }
    pub fn titel(&self) -> &str { match self.kind.as_str() { "sprite" => "Sprite", "pmg" => "Player/Missile", _ => "Zeichensatz" } }
}

/// Registeradresse (oder Registername) zu einem Pixelwert; `None`, wenn der Name sie schon nennt oder nichts zu zeigen ist
fn value_reg(target: &str, kind: &str, bpp: usize, i: usize, fi: usize, name: &str) -> Option<String> {
    if name.contains('$') { return None; }
    let r = |s: &str| Some(s.to_string());
    match (target, kind, bpp, i) {
        ("c64" | "c128" | "mega65", "sprite", 1, 1) | ("c64" | "c128" | "mega65", "sprite", 2, 2) => r("$D027+n"),
        ("atari8", "pmg", _, 1) => Some(format!("COLPM{fi} ${:04X}", 0xD012 + fi)),
        ("c64" | "c128", "charset", _, 0) => r("$D021"),
        ("c64" | "c128", "charset", 1, 1) => r("Farb-RAM $D800"),
        ("c64" | "c128", "charset", 2, 1) => r("$D022"),
        ("c64" | "c128", "charset", 2, 2) => r("$D023"),
        ("c64" | "c128", "charset", 2, 3) => r("Farb-RAM $D800"),
        ("atari8", "charset", 1, 0) => r("COLPF2 $D018"),
        ("atari8", "charset", 1, 1) => r("COLPF1 $D017"),
        _ => None,
    }
}

/// Hardware-Flags eines Sprites: Bit, Name, Hinweis mit Register (nur Systeme mit VIC-II-Sprites)
const SPRITE_FLAGS: &[(u8, &str, &str)] = &[(0, "MC", "Multicolor ($D01C)"), (1, "X2", "Breit: doppelte Breite ($D01D)"), (2, "Y2", "Hoch: doppelte Höhe ($D017)"), (3, "BG", "Hinter dem Hintergrund ($D01B)")];

/// Hat das Profil Hardware-Flags? (Sprites der VIC-II-Systeme und Atari-Player/Missile)
pub fn has_sprite_flags(target: &str, kind: &str) -> bool { kind == "sprite" && matches!(target, "c64" | "c128") }

/// Flags aus den Metadaten `hw` der Mediendatei als Bitwert (Bit 0 = Multicolor kommt aus dem Frame-Modus, nicht aus `hw`)
pub fn flags_from_hw(hw: &Value) -> u8 { (hw["expandX"].as_bool().unwrap_or(false) as u8) << 1 | (hw["expandY"].as_bool().unwrap_or(false) as u8) << 2 | (hw["behind"].as_bool().unwrap_or(false) as u8) << 3 }

/// Schreibt die Flags (ohne Multicolor) in `hw`; sind alle aus, entfällt `hw` (alte Dateien bleiben unverändert)
pub fn flags_to_hw(media: &mut Value, bits: u8) {
    let hw = json!({ "expandX": bits & 2 != 0, "expandY": bits & 4 != 0, "behind": bits & 8 != 0 });
    if bits & 0b1110 == 0 { if let Some(o) = media.as_object_mut() { o.remove("hw"); } } else { media["hw"] = hw; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../") }

    #[test]
    fn sprite_roundtrip_edit_and_save() {
        let (data, dir) = (root().join("data"), std::env::temp_dir().join(format!("vc-bm-{}", std::process::id())));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(root().join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("hero.asset.json")).unwrap();
        let orig: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("hero.asset.json")).unwrap()).unwrap();
        let mut b = PixelDoc::open(&dir, "hero.asset.json", &data).unwrap();
        assert_eq!(b.to_value()["frames"], orig["frames"], "Laden und Zurückschreiben ändert nichts");
        assert!(!b.dirty());
        b.push_undo(); b.set(0, 0, 3); b.flood(5, 5, 2); b.line((0, 20), (11, 20), 1);
        assert!(b.dirty());
        b.save().unwrap();
        let b2 = PixelDoc::open(&dir, "hero.asset.json", &data).unwrap();
        assert_eq!(b2.get(0, 0), 3); assert_eq!(b2.get(11, 20), 1);
        b.undo(); assert_eq!(b.get(0, 0), orig["frames"][0]["rows"][0].as_str().unwrap().chars().next().unwrap().to_digit(16).unwrap() as u8);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn png_import_und_vorlage() {
        let (data, dir) = (root().join("data"), std::env::temp_dir().join(format!("vc-bm-png-{}", std::process::id()))); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(root().join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("hero.vcsprite")).unwrap();
        let mut b = PixelDoc::open(&dir, "hero.vcsprite", &data).unwrap(); b.frame_waehlen(0);
        // 12×21 Multicolor-Pixel = 24×21 Bildpunkte: eine rote Fläche oben links, Rest transparent
        let (w, h) = (24usize, 21usize); let rgba: Vec<u8> = (0..w * h).flat_map(|i| if i % w < 8 && i / w < 5 { [0x88, 0x39, 0x32, 255] } else { [0, 0, 0, 0] }).collect();
        let png = vc_media::png::encode_rgba(w, h, &rgba); let vorher = b.frames[0].clone();
        let r = b.png_importieren(&dir, &data, &png); assert!(r.is_ok(), "{r:?}"); assert!(b.frames[0] != vorher, "Frame hat sich geändert"); assert!(b.kann_undo()); b.undo(); assert_eq!(b.frames[0], vorher, "Rückgängig stellt das Frame wieder her");
        std::fs::remove_dir_all(&dir).ok();
    }
    #[test]
    fn pmg_and_charset_open() {
        let data = root().join("data");
        for (rel, kind) in [("examples/atari/pmg_ship/graphics/ship.vcpmg", "pmg"), ("examples/atari/gfx_demo/graphics/afont.vcchar", "charset")] {
            let p = root().join(rel); let b = PixelDoc::open(p.parent().unwrap(), p.file_name().unwrap().to_str().unwrap(), &data).unwrap();
            assert_eq!(b.kind, kind); assert!(!b.frames.is_empty());
        }
    }

    #[test]
    fn rect_selection_paste_and_fcm_roundtrip() {
        let data = root().join("data"); let dir = std::env::temp_dir().join(format!("vc-bm2-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(root().join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("hero.asset.json")).unwrap();
        let mut b = PixelDoc::open(&dir, "hero.asset.json", &data).unwrap();
        b.rect_outline((1, 1), (4, 3), 2); assert_eq!((b.get(1, 1), b.get(4, 3), b.get(2, 2)), (2, 2, b.get(2, 2)));
        b.sel = Some((1, 1, 4, 3)); b.sel_copy(true); assert_eq!(b.get(1, 1), 0); assert!(b.clip.is_some());
        b.sel = Some((6, 6, 6, 6)); b.sel_paste(); assert_eq!(b.get(6, 6), 2, "Ecke eingefügt");
        // MEGA65-Vollfarbzeichensatz (8 Bit je Pixel, zwei Hex-Ziffern)
        let profiles = vc_media::load_profiles(&data).unwrap(); let p = vc_media::profile(&profiles, "mega65.char.fcm").unwrap();
        std::fs::write(dir.join("fcm.asset.json"), vc_media::project::format_media(&vc_media::new_media(p, "fcm"))).unwrap();
        let mut f = PixelDoc::open(&dir, "fcm.asset.json", &data).unwrap(); assert_eq!(f.bpp, 8);
        f.fi = 3; f.set(2, 2, 200); f.save().unwrap();
        let f2 = PixelDoc::open(&dir, "fcm.asset.json", &data).unwrap(); assert_eq!(f2.frames.len(), 256.min(f2.frames.len())); assert_eq!({ let mut g = f2; g.fi = 3; g.get(2, 2) }, 200);
        std::fs::remove_dir_all(&dir).ok();
    }
    /// Hires-Sprite mit eingemischten Multicolor-Frames und zwei Animationen
    fn mixed_sprite_json() -> Value {
        let (h, m): (Vec<String>, Vec<String>) = ((0..21).map(|y| if y == 0 { format!("1111{}", "0".repeat(20)) } else { "0".repeat(24) }).collect(), (0..21).map(|y| if y == 0 { format!("123{}", "0".repeat(9)) } else { "0".repeat(12) }).collect());
        json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "held", "colors": { "sprite": 7 },
            "frames": [ { "rows": h.clone() }, { "mode": "multicolor", "rows": m.clone() }, { "rows": h } ],
            "animations": [ { "name": "lauf", "mode": "pingpong", "steps": [ { "frame": 0, "delay": 4 }, { "frame": 1, "delay": 3 }, { "frame": 2, "delay": 5 } ] } ] })
    }

    #[test]
    fn mixed_hires_and_multicolor_frames_roundtrip_and_switch_mode() {
        let (data, dir) = (root().join("data"), std::env::temp_dir().join(format!("vc-bm-mix-{}", std::process::id())));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("held.vcsprite"), mixed_sprite_json().to_string()).unwrap();
        let mut b = PixelDoc::open(&dir, "held.vcsprite", &data).unwrap();
        assert_eq!(b.frame_mc, vec![false, true, false]);
        // Laden und Zurückschreiben ändert nichts (Modus nur bei abweichenden Frames, Animationen bleiben)
        let orig = mixed_sprite_json();
        assert_eq!(b.to_value()["frames"], orig["frames"]); assert_eq!(b.to_value()["animations"], orig["animations"]);
        assert!(!b.dirty());
        // Frame 1 (Multicolor) hat 12×21 Pixel und vier Werte, Frame 0 24×21 und zwei
        b.fi = 1; b.sync_mode(); assert_eq!((b.width, b.bpp, b.values.len()), (12, 2, 4)); assert_eq!(b.get(2, 0), 3);
        b.fi = 0; b.sync_mode(); assert_eq!((b.width, b.bpp, b.values.len()), (24, 1, 2));
        // Modus wechseln: hires -> multicolor rechnet um, Strg+Z stellt wieder her
        b.set_frame_mode(true);
        assert_eq!((b.width, b.frames[0][0].len(), b.is_mc(0)), (12, 12, true)); assert_eq!(&b.frames[0][0][..3], &[2, 2, 0]);   // gesetzte Hires-Pixel werden zu Wert 2 = Sprite-Farbe
        assert_eq!(b.to_value()["frames"][0]["mode"], "multicolor");
        b.undo(); assert!(!b.is_mc(0)); assert_eq!(b.frames[0][0].len(), 24);
        // Frame duplizieren verschiebt die Frame-Nummern der Animationen
        b.push_undo(); let f = b.frames[1].clone(); b.frames.insert(2, f); b.frame_mc.insert(2, true); sprite::animations_after_insert(&mut b.media, 2);
        assert_eq!(sprite::parse_animations(&b.anim_view()).unwrap()[0].steps, vec![(0, 4), (1, 3), (3, 5)]);
        b.save().unwrap();
        let b2 = PixelDoc::open(&dir, "held.vcsprite", &data).unwrap();
        assert_eq!(b2.frame_mc, vec![false, true, true, false]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hardware_flags_abwaertskompatibel() {
        let mut m = json!({ "kind": "sprite", "frames": [] });
        assert_eq!(flags_from_hw(&m["hw"]), 0);
        flags_to_hw(&mut m, 0b1010);   // X2 und BG
        assert_eq!(m["hw"], json!({ "expandX": true, "expandY": false, "behind": true }));
        assert_eq!(flags_from_hw(&m["hw"]), 0b1010);
        flags_to_hw(&mut m, 0b0001);   // nur Multicolor (kommt aus dem Frame-Modus): kein hw
        assert!(m.get("hw").is_none());
        assert!(has_sprite_flags("c64", "sprite") && !has_sprite_flags("atari8", "sprite") && !has_sprite_flags("c64", "charset"));
    }

    #[test]
    fn registerangaben_zu_pixelwerten() {
        assert_eq!(value_reg("c64", "sprite", 1, 1, 0, "Sprite-Farbe").as_deref(), Some("$D027+n"));
        assert_eq!(value_reg("c64", "sprite", 2, 1, 0, "Multicolor 1 ($D025)"), None);   // steht schon im Namen
        assert_eq!(value_reg("atari8", "pmg", 1, 1, 2, "Spielerfarbe").as_deref(), Some("COLPM2 $D014"));
        assert_eq!(value_reg("c64", "charset", 2, 2, 0, "Multicolor 2").as_deref(), Some("$D023"));
        assert_eq!(value_reg("vic20", "charset", 1, 0, 0, "Hintergrund"), None);
    }

}
