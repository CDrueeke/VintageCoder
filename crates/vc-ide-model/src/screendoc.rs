// SPDX-License-Identifier: GPL-3.0-or-later
//! Textbildschirm (`.vcscreen` mit `chars` und `attrs`, Profil `*.screen.text`) ohne Oberfläche: Zeichen- und Farbraster, Zeichensatz,
//! Werkzeuge (Zeichen, Text, Füllen, Rahmen, Pipette, Löschen), Rückgängig, Speichern. Die Oberfläche zeigt nur an und leitet Mausaktionen weiter.
//! Kachel-/Bank-Bildschirme (`screens`) und Kachelsätze haben eigene Modelle.
use crate::glyphs::{bildschirmcode, Bild, Glyphs, McCols};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Werkzeug { Zeichen, Text, Fuellen, Rahmen, Pipette, Loeschen }
impl Werkzeug {
    /// Bezeichnung in der gewählten Sprache
    pub fn label(self) -> String {
        match self {
            Werkzeug::Zeichen => vc_i18n::t!("screendoc-werkzeug-zeichen"), Werkzeug::Text => vc_i18n::t!("screendoc-werkzeug-text"), Werkzeug::Fuellen => vc_i18n::t!("screendoc-werkzeug-fuellen"),
            Werkzeug::Rahmen => vc_i18n::t!("screendoc-werkzeug-rahmen"), Werkzeug::Pipette => vc_i18n::t!("screendoc-werkzeug-pipette"), Werkzeug::Loeschen => vc_i18n::t!("screendoc-werkzeug-loeschen"),
        }
    }
}

pub struct ScreenDoc {
    pub rel: String, abs: PathBuf, root: PathBuf,
    pub media: Value, pub target: String, pub w: usize, pub h: usize, pub farb_ram: bool,
    pub glyphs: Option<Glyphs>,
    /// Zeichensätze des Projekts für dieses System (Auswahlliste)
    pub zeichensaetze: Vec<String>,
    undo: Vec<String>, redo: Vec<String>, saved: String,
    pub status: String,
    pub werkzeug: Werkzeug, pub zeichen: u8, pub farbe: usize,
    /// Textcursor / zuletzt bearbeitete Zelle
    pub cursor: (usize, usize),
    /// Startzelle des Rahmens (zwischen Drücken und Loslassen)
    pub von: Option<(usize, usize)>,
    /// zuletzt gezeichnete Zelle beim Ziehen (schnelle Mausbewegung lässt keine Lücken)
    letzte: Option<(usize, usize)>,
}

/// Zeichensätze des Projekts, die zum Zielsystem `ziel` passen
pub fn zeichensaetze_im_projekt(root: &Path, ziel: &str, profiles: &[vc_media::Profile]) -> Vec<String> {
    fn lauf(root: &Path, dir: &Path, tiefe: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().into_owned(); if n.starts_with('.') || n == "build" || n == "target" { continue }
            let p = e.path();
            if p.is_dir() { if tiefe < 4 { lauf(root, &p, tiefe + 1, out) } } else if n.ends_with(".vcchar") || n.ends_with(".asset.json") { if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")) } }
        }
    }
    let mut v = vec![]; lauf(root, root, 0, &mut v);
    v.retain(|rel| std::fs::read_to_string(root.join(rel)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).is_some_and(|w| w["kind"] == "charset" && profiles.iter().any(|p| Some(p.id.as_str()) == w["profile"].as_str() && p.target == ziel)));
    v.sort(); v
}

/// Ist die Datei ein einfacher Textbildschirm (kein Bank-Bildschirm)?
pub fn ist_textbildschirm(v: &Value) -> bool { v["kind"] == "screen" && !v["screens"].is_array() && v["chars"].is_array() }

impl ScreenDoc {
    pub fn oeffnen(root: &Path, rel: &str, data: &Path) -> Result<ScreenDoc, String> {
        let abs = root.join(rel);
        let media: Value = serde_json::from_str(&std::fs::read_to_string(&abs).map_err(|e| vc_i18n::t!("screendoc-rel-e", rel = &rel, e = e.to_string()))?).map_err(|e| vc_i18n::t!("screendoc-rel-e", rel = &rel, e = e.to_string()))?;
        if !ist_textbildschirm(&media) { return Err(vc_i18n::t!("screendoc-rel-kein-textbildschirm", rel = &rel)) }
        let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, media["profile"].as_str().unwrap_or(""))?;
        let glyphs = media["charset"].as_str().and_then(|c| Glyphs::laden(root, c));
        let farb_ram = p.raw["colorRam"].as_bool().unwrap_or(false);
        let farbe = media["colors"]["fg"].as_u64().map_or(1, |v| v as usize);
        Ok(ScreenDoc { rel: rel.into(), root: root.into(), saved: media.to_string(), zeichensaetze: zeichensaetze_im_projekt(root, &p.target, &profiles), target: p.target.clone(), w: p.width, h: p.height, farb_ram, glyphs, media, abs,
            undo: vec![], redo: vec![], status: String::new(), werkzeug: Werkzeug::Zeichen, zeichen: 1, farbe, cursor: (0, 0), von: None, letzte: None })
    }

    // -- Zustand --------------------------------------------------------------------------------------------------------
    pub fn dirty(&self) -> bool { self.media.to_string() != self.saved }
    pub fn push_undo(&mut self) { self.undo.push(self.media.to_string()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear() }
    pub fn undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.glyphs_neu() } } }
    pub fn redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.media.to_string()); if let Ok(v) = serde_json::from_str(&s) { self.media = v; self.glyphs_neu() } } }
    pub fn kann_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn kann_redo(&self) -> bool { !self.redo.is_empty() }
    fn glyphs_neu(&mut self) { self.glyphs = self.media["charset"].as_str().and_then(|c| Glyphs::laden(&self.root, c)) }
    pub fn save(&mut self) -> Result<(), String> {
        vc_core::atomar::schreiben(&self.abs, vc_media::project::format_media(&self.media)).map_err(|e| format!("{}: {e}", self.rel))?;
        self.saved = self.media.to_string(); self.status = vc_i18n::t!("screendoc-gespeichert"); Ok(())
    }
    /// Text der Mediendatei, wie er gespeichert würde (Export-Vorschau)
    pub fn media_text(&self) -> String { vc_media::project::format_media(&self.media) }

    // -- Lesen ----------------------------------------------------------------------------------------------------------
    pub fn code(&self, x: usize, y: usize) -> u8 { self.media["chars"][y][x].as_u64().unwrap_or(32) as u8 }
    pub fn attr(&self, x: usize, y: usize) -> usize { self.media["attrs"][y][x].as_u64().map_or(1, |v| v as usize) }
    pub fn farbe_von(&self, schluessel: &str, d: usize) -> usize { self.media["colors"][schluessel].as_u64().map_or(d, |v| v as usize) }
    pub fn zeichensatz(&self) -> Option<&str> { self.media["charset"].as_str() }
    pub fn mc(&self) -> McCols { McCols::aus_farben(&self.media["colors"]) }

    // -- Ändern ---------------------------------------------------------------------------------------------------------
    pub fn setze(&mut self, x: usize, y: usize, code: u8, farbe: usize) {
        if x >= self.w || y >= self.h { return }
        self.media["chars"][y][x] = json!(code); if self.farb_ram { self.media["attrs"][y][x] = json!(farbe) }
    }
    pub fn fuelle(&mut self, x: usize, y: usize, code: u8) {
        if x >= self.w || y >= self.h { return }
        let von = self.code(x, y); if von == code { return }
        let mut st = vec![(x, y)];
        while let Some((px, py)) = st.pop() {
            if self.code(px, py) != von { continue }
            let f = self.farbe; self.setze(px, py, code, f);
            if px > 0 { st.push((px - 1, py)) } if py > 0 { st.push((px, py - 1)) } if px + 1 < self.w { st.push((px + 1, py)) } if py + 1 < self.h { st.push((px, py + 1)) }
        }
    }
    /// Zellen des Rahmens zwischen zwei Ecken
    pub fn rahmen_zellen(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
        let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1)); let mut v = vec![];
        for x in x0..=x1 { v.push((x, y0)); v.push((x, y1)) } for y in y0 + 1..y1 { v.push((x0, y)); v.push((x1, y)) } v.dedup(); v
    }
    /// Zellen auf der Geraden zwischen zwei Zellen (beide eingeschlossen)
    pub fn linie(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
        let (mut x, mut y) = (a.0 as i32, a.1 as i32); let (x1, y1) = (b.0 as i32, b.1 as i32); let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs()); let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
        let mut fehler = dx + dy; let mut v = vec![];
        loop { v.push((x as usize, y as usize)); if x == x1 && y == y1 { break } let e2 = 2 * fehler; if e2 >= dy { fehler += dy; x += sx } if e2 <= dx { fehler += dx; y += sy } }
        v
    }
    pub fn werkzeug_waehlen(&mut self, w: Werkzeug) { self.werkzeug = w; self.von = None }
    pub fn zeichen_waehlen(&mut self, c: u8) { self.zeichen = c }
    pub fn farbe_waehlen(&mut self, i: usize) { self.farbe = i }
    /// Farbrolle des Bildschirms setzen (`bg`, `border`, `fg`)
    pub fn rollenfarbe(&mut self, rolle: &str, i: usize) { self.push_undo(); self.media["colors"][rolle] = json!(i) }
    pub fn zeichensatz_waehlen(&mut self, rel: Option<String>) { self.push_undo(); self.media["charset"] = rel.map_or(Value::Null, Value::from); self.glyphs_neu() }

    /// Maus gedrückt in Zelle (x,y); `rechts` löscht (Leerzeichen) bei den Zeichenwerkzeugen
    pub fn maus_unten(&mut self, x: usize, y: usize, rechts: bool) {
        if x >= self.w || y >= self.h { return }
        self.cursor = (x, y);
        match self.werkzeug {
            Werkzeug::Pipette => { self.zeichen = self.code(x, y); if self.farb_ram { self.farbe = self.attr(x, y) } self.werkzeug = Werkzeug::Zeichen }
            Werkzeug::Text => {}
            Werkzeug::Fuellen => { self.push_undo(); let c = if rechts { 32 } else { self.zeichen }; self.fuelle(x, y, c) }
            Werkzeug::Rahmen => self.von = Some((x, y)),
            Werkzeug::Zeichen | Werkzeug::Loeschen => { self.push_undo(); let c = if self.werkzeug == Werkzeug::Loeschen || rechts { 32 } else { self.zeichen }; let f = self.farbe; self.setze(x, y, c, f); self.letzte = Some((x, y)) }
        }
    }
    /// Maus gezogen über Zelle (x,y)
    pub fn maus_zug(&mut self, x: usize, y: usize, rechts: bool) {
        if x >= self.w || y >= self.h { return }
        if matches!(self.werkzeug, Werkzeug::Zeichen | Werkzeug::Loeschen) {
            self.cursor = (x, y); let c = if self.werkzeug == Werkzeug::Loeschen || rechts { 32 } else { self.zeichen }; let f = self.farbe;
            for (px, py) in Self::linie(self.letzte.unwrap_or((x, y)), (x, y)) { self.setze(px, py, c, f) }
            self.letzte = Some((x, y));
        }
    }
    /// Maus losgelassen in Zelle (x,y)
    pub fn maus_los(&mut self, x: usize, y: usize, rechts: bool) {
        self.letzte = None;
        if self.werkzeug == Werkzeug::Rahmen { if let Some(a) = self.von.take() { self.push_undo(); let (c, f) = (if rechts { 32 } else { self.zeichen }, self.farbe); for (px, py) in Self::rahmen_zellen(a, (x.min(self.w - 1), y.min(self.h - 1))) { self.setze(px, py, c, f) } } }
    }
    /// Zeichen tippen (Werkzeug Text): setzt das Zeichen an den Cursor und rückt vor
    pub fn tippe(&mut self, ch: char) {
        if self.werkzeug != Werkzeug::Text { return }
        let (x, y) = self.cursor; self.push_undo(); let (c, f) = (bildschirmcode(&self.target, ch), self.farbe); self.setze(x, y, c, f);
        self.cursor = if x + 1 < self.w { (x + 1, y) } else if y + 1 < self.h { (0, y + 1) } else { (x, y) };
    }
    pub fn cursor_bewegen(&mut self, dx: i32, dy: i32) { self.cursor = ((self.cursor.0 as i32 + dx).clamp(0, self.w as i32 - 1) as usize, (self.cursor.1 as i32 + dy).clamp(0, self.h as i32 - 1) as usize) }

    // -- Bild -----------------------------------------------------------------------------------------------------------
    /// Randbreite in Pixeln des Bildes (C64/VIC-20 haben einen sichtbaren Rahmen)
    pub fn rand(&self) -> usize { if matches!(self.target.as_str(), "c64" | "vic20") { 2 } else { 1 } }
    /// Bildschirm als RGBA-Bild: Rand (`border`), Hintergrund (`bg`), Zeichen; `skala` Bildpunkte je Zeichenpixel; `rgb` = Palette des Systems
    pub fn bild(&self, skala: usize, rgb: &dyn Fn(usize) -> [u8; 3]) -> Bild {
        let (s, rand) = (skala.max(1), self.rand() * 8 * skala.max(1));
        let (bw, bh) = (self.w * 8 * s, self.h * 8 * s);
        let mut b = Bild::neu(bw + 2 * rand, bh + 2 * rand, rgb(self.farbe_von("border", self.farbe_von("bg", 6))));
        b.flaeche(rand, rand, bw, bh, rgb(self.farbe_von("bg", 6)));
        let (mc, fg) = (self.mc(), self.farbe_von("fg", 14));
        if let Some(g) = &self.glyphs { for y in 0..self.h { for x in 0..self.w {
            let zelle = if self.farb_ram { self.attr(x, y) } else { fg };
            g.zeichne(&mut b, rand + x * 8 * s, rand + y * 8 * s, s, &self.target, self.code(x, y), zelle, mc, false, rgb);
        } } }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn kopie(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vc-screendoc-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        for (f, t) in [("font", "vcchar"), ("title", "vcscreen")] { std::fs::copy(repo().join(format!("examples/c64/gfx_demo/graphics/{f}.{t}")), dir.join(format!("graphics/{f}.{t}"))).unwrap(); }
        dir
    }

    #[test]
    fn oeffnen_bearbeiten_speichern_rundreise() {
        let (data, dir) = (repo().join("data"), kopie("a"));
        let mut s = ScreenDoc::oeffnen(&dir, "graphics/title.vcscreen", &data).unwrap();
        assert_eq!((s.w, s.h), (40, 25)); assert!(s.glyphs.is_some(), "Zeichensatz aus dem Projekt geladen"); assert_eq!(s.zeichensaetze, vec!["graphics/font.vcchar".to_string()]); assert!(!s.dirty());
        s.push_undo(); s.setze(3, 4, 7, 5); s.fuelle(10, 10, 9); assert!(s.dirty()); s.save().unwrap(); assert!(!s.dirty());
        let s2 = ScreenDoc::oeffnen(&dir, "graphics/title.vcscreen", &data).unwrap(); assert_eq!(s2.code(3, 4), 7); assert_eq!(s2.attr(3, 4), 5);
        let mut s3 = ScreenDoc::oeffnen(&dir, "graphics/title.vcscreen", &data).unwrap(); s3.push_undo(); s3.setze(0, 0, 99, 1); s3.undo(); assert_eq!(s3.code(0, 0), s2.code(0, 0), "Rückgängig"); s3.redo(); assert_eq!(s3.code(0, 0), 99);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn werkzeuge_zeichen_text_fuellen_rahmen_pipette_loeschen() {
        let (data, dir) = (repo().join("data"), kopie("b")); let mut s = ScreenDoc::oeffnen(&dir, "graphics/title.vcscreen", &data).unwrap();
        s.zeichen_waehlen(66); s.farbe_waehlen(5);
        s.maus_unten(2, 2, false); s.maus_zug(3, 2, false); s.maus_los(3, 2, false); assert_eq!((s.code(2, 2), s.code(3, 2), s.attr(3, 2)), (66, 66, 5), "Zeichen setzt beim Klick und Ziehen");
        s.maus_unten(2, 2, true); assert_eq!(s.code(2, 2), 32, "rechte Taste löscht");
        s.werkzeug_waehlen(Werkzeug::Rahmen); s.maus_unten(10, 10, false); s.maus_los(13, 12, false);
        let rahmen = ScreenDoc::rahmen_zellen((10, 10), (13, 12)); assert_eq!(rahmen.len(), 10, "Rahmen 4×3: 10 Randzellen"); assert!(rahmen.iter().all(|&(x, y)| s.code(x, y) == 66)); assert_eq!(s.code(11, 11), 32, "Innen unberührt");
        s.werkzeug_waehlen(Werkzeug::Pipette); s.maus_unten(10, 10, false); assert_eq!((s.zeichen, s.farbe, s.werkzeug), (66, 5, Werkzeug::Zeichen));
        s.werkzeug_waehlen(Werkzeug::Text); s.maus_unten(20, 20, false); for c in "ab".chars() { s.tippe(c) } assert_eq!((s.code(20, 20), s.code(21, 20), s.cursor), (1, 2, (22, 20)), "Text: Bildschirmcodes, Cursor rückt vor");
        s.werkzeug_waehlen(Werkzeug::Fuellen); s.zeichen_waehlen(81); let n0 = s.undo.len(); s.maus_unten(30, 20, false); assert_eq!(s.undo.len(), n0 + 1, "ein Rückgängig-Schritt je Füllen"); assert_eq!(s.code(39, 24), 81);
        s.werkzeug_waehlen(Werkzeug::Loeschen); s.maus_unten(30, 20, false); assert_eq!(s.code(30, 20), 32);
        assert_eq!(ScreenDoc::linie((0, 0), (3, 0)), vec![(0, 0), (1, 0), (2, 0), (3, 0)]); assert_eq!(ScreenDoc::linie((2, 2), (0, 0)).len(), 3, "schräg, rückwärts");
        s.werkzeug_waehlen(Werkzeug::Zeichen); s.zeichen_waehlen(70); s.maus_unten(0, 24, false); s.maus_zug(5, 24, false); s.maus_los(5, 24, false); assert!((0..=5).all(|x| s.code(x, 24) == 70), "schnelles Ziehen lässt keine Lücken");
        s.maus_unten(99, 99, false); s.setze(99, 99, 1, 1); // außerhalb: nichts, kein Absturz
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bild_hat_rand_hintergrund_und_zeichen_und_zeichensatzwechsel() {
        let (data, dir) = (repo().join("data"), kopie("c")); let mut s = ScreenDoc::oeffnen(&dir, "graphics/title.vcscreen", &data).unwrap();
        let rgb = |i: usize| -> [u8; 3] { [i as u8 * 10 + 1, 0, 0] };
        let b = s.bild(1, &rgb); assert_eq!((b.w, b.h), (40 * 8 + 32, 25 * 8 + 32), "C64: 2 Zeichen Rand");
        assert_eq!(&b.rgba[0..3], &rgb(s.farbe_von("border", 14))[..], "Rand"); let innen = ((16 * b.w) + 16 + 0) * 4; assert_ne!(&b.rgba[innen..innen + 3], &rgb(s.farbe_von("border", 14))[..], "Bildinneres hat nicht die Randfarbe");
        let n = b.rgba.chunks(4).filter(|p| p[..3] == rgb(1)[..] || p[..3] == rgb(s.farbe_von("fg", 14))[..]).count(); assert!(n > 0, "Zeichen sind gezeichnet");
        s.zeichensatz_waehlen(None); assert!(s.glyphs.is_none() && s.dirty()); let leer = s.bild(1, &rgb); assert_ne!(leer, b, "ohne Zeichensatz keine Zeichen"); s.undo(); assert!(s.glyphs.is_some(), "Rückgängig stellt den Zeichensatz wieder her");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
