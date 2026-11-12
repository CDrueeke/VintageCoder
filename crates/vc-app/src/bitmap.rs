// SPDX-License-Identifier: GPL-3.0-or-later
//! Pixel-Editor für Sprites, Player-Missile-Grafik und Zeichensätze (1 und 2 Bit je Pixel).
use eframe::egui::{self, pos2, vec2, Color32, Rect, RichText, Sense, Stroke};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use crate::ui::studio;
use vc_media::sprite::{self, Anim, AnimMode};

/// Einzeltasten im Grafik-Editor (Einstellung im Menü „Ansicht“)
pub static HOTKEYS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Modus eines Sprite-Frames aus dem Profil (`modes`)
#[derive(Clone)]
struct ModeInfo { label: String, hint: String, width: usize, bpp: usize, aspect: f32, values: Vec<String> }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool { Pen, Erase, Fill, Line, Rect, Select, Pick }

/// Farbtabellen der Zielsysteme (aus `vc_media::palette`, PAL-Variante)
pub struct Palettes { by_target: std::collections::HashMap<String, vc_media::palette::TargetPalette>, default: Option<vc_media::palette::TargetPalette>, atari_ntsc: Option<vc_media::palette::TargetPalette> }

/// Atari-Farben in der NTSC- statt der PAL-Tabelle zeigen (Schalter im Studio, gilt für alle Atari-Editoren)
pub static ATARI_NTSC: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
fn parse_hex(s: &str) -> Color32 {
    let t = s.trim_start_matches('#');
    if t.len() == 6 { if let Ok(v) = u32::from_str_radix(t, 16) { return Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8); } }
    Color32::GRAY
}

impl Palettes {
    pub fn load(data: &Path) -> Palettes {
        let by_target = vc_media::palette::known_targets(data).into_iter().filter_map(|t| vc_media::palette::palette_for(data, &t, "pal").ok().map(|p| (t, p))).collect();
        Palettes { by_target, default: vc_media::palette::palette_for(data, "", "pal").ok(), atari_ntsc: vc_media::palette::palette_for(data, "atari8", "ntsc").ok() }
    }
    pub fn color(&self, target: &str, n: usize) -> Color32 {
        let ntsc = target == "atari8" && ATARI_NTSC.load(std::sync::atomic::Ordering::Relaxed);
        let rgb = if ntsc { self.atari_ntsc.as_ref() } else { None }.or(self.by_target.get(target)).or(self.default.as_ref()).and_then(|p| p.rgb(n));
        rgb.map_or(Color32::GRAY, |c| Color32::from_rgb(c[0], c[1], c[2]))
    }
}

/// Datei nach `build/export/` exportieren (speichert vorher nicht; der Aufrufer speichert zuerst)
pub fn export_media(env: &Env, rel: &str, kind: &str) {
    let fmt = if kind == "play" { "bas" } else { "asm" };
    match vc_media::project::export(env.root, env.data, rel, fmt, None, None, 10, 10) {
        Ok(v) => { let files: Vec<String> = v["files"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| v["path"].as_str().map(|p| vec![p.to_string()]).unwrap_or_default()); env.reqs.info(format!("Exportiert: {}", if files.is_empty() { "build/export/".to_string() } else { files.join(", ") })); }
        Err(e) => env.reqs.error(format!("Export {rel}: {e}")),
    }
}

/// Gemeinsame Schnittstelle aller nativen Media-Editoren
/// Anfragen der Editoren an die Anwendung (Dateien öffnen, Meldungen)
#[derive(Default)]
pub struct Requests { pub run: std::cell::Cell<bool>, pub open: std::cell::RefCell<Vec<String>>, pub insert: std::cell::RefCell<Vec<String>>, pub log: std::cell::RefCell<Vec<(u8, String)>> }
impl Requests {
    pub fn open(&self, rel: &str) { self.open.borrow_mut().push(rel.to_string()); }
    /// Asset (Pfad im Projekt) als `.include` in den zuletzt benutzten Assembler-Quelltext einfügen
    /// Projekt bauen und im Emulator starten (aus „Im Emulator testen“ im Export-Dialog)
    pub fn run_project(&self) { self.run.set(true); }
    pub fn insert_asset(&self, rel: &str) { self.insert.borrow_mut().push(rel.to_string()); }
    pub fn info(&self, m: impl Into<String>) { self.log.borrow_mut().push((0, m.into())); }
    pub fn warn(&self, m: impl Into<String>) { self.log.borrow_mut().push((1, m.into())); }
    pub fn error(&self, m: impl Into<String>) { self.log.borrow_mut().push((2, m.into())); }
}

pub struct Env<'a> { pub pal: &'a Palettes, pub audio: &'a crate::audio::Audio, pub reqs: &'a Requests, pub root: &'a Path, pub data: &'a Path }

pub trait MediaEd {
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env);
    fn dirty(&self) -> bool;
    fn save(&mut self) -> Result<(), String>;
}

impl MediaEd for Bitmap {
    fn ui(&mut self, ui: &mut egui::Ui, env: &Env) { Bitmap::ui(self, ui, env) }
    fn dirty(&self) -> bool { Bitmap::dirty(self) }
    fn save(&mut self) -> Result<(), String> { Bitmap::save(self) }
}

/// Dialog „Konvertieren von …“ (Sprites aus dem SPD-Format, Rohdaten oder einer Sprite-Datei eines anderen Systems)
struct ImportDlg { file: PathBuf, name: String, kind: ImportKind, info: String, offset: String, count: u32, mode: u8, frame: u32, available: usize }
#[derive(Clone, Copy, PartialEq)]
enum ImportKind { Spd, Raw, Media }

pub struct Bitmap {
    pub rel: String,
    profile: String,
    abs: PathBuf,
    media: Value,
    kind: String,
    target: String,
    values: Vec<String>,
    frame_names: Vec<String>,
    width: usize,
    height: usize,
    bpp: usize,
    aspect: f32,
    fixed_frames: bool,
    frames: Vec<Vec<Vec<u8>>>,
    fi: usize,
    pv: u8,
    tool: Tool,
    zoom: f32,
    /// Zoom folgt dem freien Platz in der Mitte (ganzzahlig), bis der Regler oder Strg+Mausrad ihn festlegt
    fit: bool,
    avail: egui::Vec2,
    undo: Vec<String>,
    redo: Vec<String>,
    saved: String,
    drag_from: Option<(usize, usize)>,
    sel: Option<(usize, usize, usize, usize)>,
    clip: Option<Vec<Vec<u8>>>,
    onion: bool,
    anim: bool,
    /// Sprites: Multicolor-Flag je Frame (Hires und Multicolor dürfen gemischt vorkommen) und Modus des Profils
    frame_mc: Vec<bool>,
    profile_mc: bool,
    /// Betriebsarten je Frame (Profilfeld `modes`; leer, wenn das Profil keinen Modus je Frame kennt): [0] = ein Bit je Pixel, [1] = Multicolor
    frame_modes: Vec<ModeInfo>,
    /// Animationen: gewählte Animation, Startzeit der Vorschau, Name im Eingabefeld, Zusammenfassen von Undo-Schritten
    sel_anim: usize,
    anim_t0: f64,
    anim_name: String,
    coalesce: Option<String>,
    imp: Option<ImportDlg>,
    exp: Option<crate::exportdlg::ExportDlg>,
    pub status: String,
}

impl Bitmap {
    pub fn open(root: &Path, rel: &str, data: &Path) -> Result<Bitmap, String> {
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
        let mut b = Bitmap { rel: rel.into(), profile: pid.to_string(), abs, kind: kind.clone(), target: p.target.clone(), values: strs("values"), frame_names: strs("frameNames"), width: p.width, height, bpp: p.bpp,
            aspect: p.raw["aspect"].as_f64().unwrap_or(1.0) as f32, fixed_frames: kind != "sprite", frames, fi: 0, pv: 1, tool: Tool::Pen,
            zoom: if kind == "charset" { 36.0 } else if p.width as f32 * p.raw["aspect"].as_f64().unwrap_or(1.0) as f32 > 20.0 { 14.0 } else { 22.0 },
            fit: true, avail: egui::Vec2::ZERO,
            undo: vec![], redo: vec![], saved: String::new(), drag_from: None, sel: None, clip: None, onion: false, anim: false, frame_mc: vec![], profile_mc: p.bpp == 2, frame_modes: if vc_media::sprite::has_frame_modes(p) { p.raw["modes"].as_array().map(|a| a.iter().map(|m| ModeInfo { label: m["label"].as_str().unwrap_or("").into(), hint: m["hint"].as_str().unwrap_or("").into(), width: m["width"].as_u64().unwrap_or(0) as usize, bpp: m["bpp"].as_u64().unwrap_or(1) as usize, aspect: m["aspect"].as_f64().unwrap_or(1.0) as f32, values: m["values"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }).collect()).unwrap_or_default() } else { vec![] }, sel_anim: 0, anim_t0: 0.0, anim_name: String::new(), coalesce: None, imp: None, exp: None, status: String::new(), media };
        b.read_modes(); b.sync_mode();
        b.saved = b.snapshot_json();
        if let Some(m) = b.media["rom_missing"].as_str() { b.status = format!("Zeichensatz-ROM fehlt – leeres Raster. {m}"); }
        else if let Some(n) = b.media["rom_note"].as_str() { b.status = n.to_string(); }
        Ok(b)
    }

    /// Zeichensatz-Vorlagen aus den ROMs des Anwenders zur Auswahl (rückgängig mit Strg+Z); der Standard des Systems steht vorn.
    /// Gewählt wird nur ein Verweis (`base`); die Original-Zeichen werden nicht in die Datei geschrieben.
    fn template_combo(&mut self, ui: &mut egui::Ui, env: &Env) {
        let sets = vc_media::project::std_charsets_for_profile(env.data, &self.profile, false);
        if sets.is_empty() { return; }
        let base = self.media["base"].as_str().unwrap_or("").to_string();
        let cur = sets.iter().find(|c| c.reference == base).map(|c| c.label.clone()).unwrap_or_else(|| "eigener Zeichensatz".into());
        let mut pick: Option<String> = None;
        ui.label("Vorlage");
        egui::ComboBox::from_id_salt(("bm-template", &self.rel)).width(280.0).selected_text(cur).show_ui(ui, |ui| {
            for (i, c) in sets.iter().enumerate() {
                let missing = c.status == vc_media::Status::Missing;
                let text = format!("{}{}{}", c.label, if i == 0 { "  (Standard)" } else { "" }, if missing { "  – ROM fehlt" } else { "" });
                if ui.add_enabled(!missing, egui::Button::selectable(c.reference == base, text)).clicked() { pick = Some(c.reference.clone()); }
            }
        });
        if sets.len() > 1 { ui.label(RichText::new(format!("{} Zeichensatz-Quellen", sets.len())).weak().small()); }
        if let Some(id) = pick {
            match vc_media::project::std_charset_media(&self.profile, &id) {
                Ok(std) if std["frames"].as_array().map_or(0, |f| f.len()) == self.frames.len() => {
                    self.push_undo();
                    let mut v = self.to_value(); v["frames"] = std["frames"].clone(); v["base"] = serde_json::json!(id);
                    for k in ["rom_missing", "rom_note", "rom_status"] { if let Some(x) = std.get(k) { v[k] = x.clone(); } else if let Some(o) = v.as_object_mut() { o.remove(k); } }
                    self.restore(&v.to_string());
                    self.status = "Vorlage geladen – mit Strg+Z rückgängig".into();
                }
                Ok(_) => self.status = "Vorlage passt nicht zur Zeichenzahl dieses Profils".into(),
                Err(e) => self.status = e,
            }
        }
    }

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
    pub fn dirty(&self) -> bool { self.snapshot_json() != self.saved }
    fn push_undo(&mut self) { self.undo.push(self.snapshot_json()); if self.undo.len() > 100 { self.undo.remove(0); } self.redo.clear(); }
    fn do_undo(&mut self) { if let Some(s) = self.undo.pop() { self.redo.push(self.snapshot_json()); self.restore(&s); } }
    fn do_redo(&mut self) { if let Some(s) = self.redo.pop() { self.undo.push(self.snapshot_json()); self.restore(&s); } }

    pub fn save(&mut self) -> Result<(), String> {
        let v = self.to_value();
        // mit `base` auf ein ROM werden nur die eigenen Änderungen geschrieben (Format 2), nie die Original-Zeichen
        std::fs::write(&self.abs, vc_media::project::format_media(&vc_media::romref::for_saving(&v))).map_err(|e| format!("{}: {e}", self.rel))?;
        self.saved = v.to_string(); self.status = "gespeichert".into(); Ok(())
    }

    fn colors(&self) -> &Value { &self.media["colors"] }
    fn cnum(&self, key: &str, default: usize) -> usize { self.colors()[key].as_u64().map(|v| v as usize).unwrap_or(default) }
    /// Anzeigefarbe eines Pixelwerts; `None` = transparent
    fn value_color(&self, pal: &Palettes, v: u8) -> Option<Color32> {
        let t = &self.target; let v = v as usize;
        if self.bpp == 8 { return Some(pal.color(t, v)); }
        match self.kind.as_str() {
            // Hires: Pixel = Sprite-Farbe. Mehrfarb: Bitpaare wie in der Hardware (01 = Multicolor 1 $D025, 10 = Sprite-Farbe, 11 = Multicolor 2 $D026)
            "sprite" => if v == 0 { None } else { Some(pal.color(t, self.cnum(if self.bpp == 1 || v == 2 { "sprite" } else if v == 1 { "mc1" } else { "mc2" }, 1))) },
            "pmg" => if v == 0 { None } else { Some(pal.color(t, self.cnum(&format!("P{}", self.fi), 14))) },
            _ => Some(pal.color(t, if v == 0 { self.cnum("bg", 0) } else if v == 1 { if self.bpp == 2 { self.cnum("mc1", 2) } else { self.cnum("fg", 1) } } else if v == 2 { self.cnum("mc2", 6) } else { self.cnum("fg", 1) })),
        }
    }
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
    fn import_png(&mut self, env: &Env) {
        if self.kind == "sprite" && self.is_mc(self.fi) != self.profile_mc { self.status = "PNG laden gilt für Frames im Modus des Profils; dieses Frame hat einen anderen Modus".into(); return; }
        let Some(f) = rfd::FileDialog::new().add_filter("Bild", &["png"]).pick_file() else { return };
        if let Err(e) = self.save() { self.status = e; return; }
        let bytes = match std::fs::read(&f) { Ok(b) => b, Err(e) => { self.status = format!("{}: {e}", f.display()); return } };
        match vc_media::project::import_png(env.root, env.data, &self.rel, &bytes, self.fi, true, "pal") {
            Ok(v) => {
                self.undo.push(self.snapshot_json()); self.redo.clear();
                let (media, rep) = (v["media"].to_string(), &v["report"]);
                self.restore(&media); self.saved = self.snapshot_json();
                let msg = format!("PNG importiert in Frame {} ({} Bildfarben, {} erlaubt{})", self.fi, rep["imageColors"], rep["allowed"], if rep["conflict"] == true { " – Farben reduziert" } else { "" });
                if rep["conflict"] == true { env.reqs.warn(msg.clone()); } else { env.reqs.info(msg.clone()); } self.status = msg;
            }
            Err(e) => { self.status = e.clone(); env.reqs.error(e); }
        }
    }
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

    /// Zeichnet ein Frame verkleinert (Vorschau/Liste)
    fn paint_frame(&self, ui: &egui::Ui, pal: &Palettes, fi: usize, rect: Rect) {
        let Some(f) = self.frames.get(fi) else { return };
        let fw = f.first().map_or(self.width, |r| r.len()).max(1);
        let (cw, ch) = (rect.width() / fw as f32, rect.height() / self.height as f32);
        let p = ui.painter_at(rect); p.rect_filled(rect, 0.0, Color32::from_rgb(0x12, 0x13, 0x17));
        let me = self;
        for (y, row) in f.iter().enumerate() { for (x, v) in row.iter().enumerate() { if let Some(c) = me.value_color_for(pal, *v, fi) { p.rect_filled(Rect::from_min_size(pos2(rect.left() + x as f32 * cw, rect.top() + y as f32 * ch), vec2(cw + 0.5, ch + 0.5)), 0.0, c); } } }
    }
    fn value_color_for(&self, pal: &Palettes, v: u8, fi: usize) -> Option<Color32> {
        if self.kind == "pmg" { if v == 0 { None } else { Some(pal.color(&self.target, self.cnum(&format!("P{fi}"), 14))) } } else { self.value_color(pal, v) }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, env: &Env) {
        let pal = env.pal;
        let (cmd_s, cmd_z, cmd_shift_z) = ui.input(|i| (i.modifiers.command && i.key_pressed(egui::Key::S), i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z), i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)));
        if cmd_s { if let Err(e) = self.save() { self.status = e; } }
        if cmd_z { self.do_undo(); } if cmd_shift_z { self.do_redo(); }
        self.hotkeys(ui);
        let id = self.rel.clone();
        let file = id.rsplit('/').next().unwrap_or(&id).to_string();
        let chips = self.head_chips();
        studio::head(ui, &id, &studio::Head { file: &file, dirty: self.dirty(), chips: &chips });
        studio::tools(ui, &id, |ui| self.tool_row(ui, env));
        let out_rel = format!("build/export/{}{}", self.media["name"].as_str().unwrap_or("export"), ".asm");
        let fresh = std::fs::metadata(env.root.join(&out_rel)).and_then(|m| m.modified()).ok().zip(std::fs::metadata(&self.abs).and_then(|m| m.modified()).ok()).map(|(o, i)| o >= i);
        let info = format!("Ziel: {}{} · Zoom {:.0} %", self.target_label(), if self.target == "atari8" { if ATARI_NTSC.load(std::sync::atomic::Ordering::Relaxed) { " NTSC" } else { " PAL" } } else { "" }, self.zoom / 22.0 * 100.0);
        let act = studio::foot(ui, &id, &studio::Foot { formats: &["ASM", "INC", "BIN", "PRG", "DATA"], out: fresh.map(|f| (out_rel.as_str(), f && !self.dirty())), info: &info, status: &self.status });
        if act == studio::FootAct::OpenExport { self.open_export(); }
        self.export_dialog(ui, env);
        self.import_dialog(ui, env);
        studio::left(ui, &id, if self.kind == "charset" { 280.0 } else { 120.0 }, |ui| self.frame_list(ui, pal));
        self.sync_mode();
        if self.kind == "sprite" { egui::Panel::bottom(egui::Id::new(("bm-anim", &self.rel))).default_size(if ui.available_height() < 520.0 { 96.0 } else { 170.0 }).show_inside(ui, |ui| self.anim_panel(ui, pal)); }
        studio::right(ui, &id, 260.0, |ui| self.side(ui, pal));
        { let a = ui.available_size(); if a != self.avail { self.avail = a; ui.ctx().request_repaint(); } }
        studio::center(ui, |ui| self.canvas(ui, pal));
    }

    /// Kurzname des Zielsystems für Chips und Fuß
    fn target_label(&self) -> String { crate::recent::short_name(&self.target).to_string() }

    fn head_chips(&self) -> Vec<(String, Color32)> {
        let mut v = vec![(self.target_label(), crate::ui::widgets::system_color(&self.target).gamma_multiply(1.4)), (self.profile.clone(), crate::ui::tokens::t().cy)];
        if self.kind == "sprite" { v.push((if self.is_mc(self.fi) { "Multicolor".into() } else { "Hires".into() }, crate::ui::tokens::t().pu)); }
        v
    }

    fn open_export(&mut self) { crate::exportdlg::open(&mut self.exp, &self.rel, &self.kind, self.media["name"].as_str().unwrap_or("export")); }

    /// Export-Dialog zeigen und seine Entscheidung ausführen
    fn export_dialog(&mut self, ui: &mut egui::Ui, env: &Env) {
        let text = vc_media::project::format_media(&self.to_value());
        let target = format!("Ziel: {}", self.target_label());
        let mut exp = self.exp.take();
        let rel = self.rel.clone();
        if let Some(st) = crate::exportdlg::drive(&mut exp, ui, env, &rel, &text, &target, || self.save()) { self.status = st; }
        self.exp = exp;
    }

    /// Werkzeugleiste: Datei, Rückgängig, Werkzeuge, Bearbeitung, Import, Zoom
    fn tool_row(&mut self, ui: &mut egui::Ui, env: &Env) {
        if ui.button("Speichern").on_hover_text("Strg/Cmd+S").clicked() { if let Err(e) = self.save() { self.status = e; } }
        if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo")).on_hover_text("Rückgängig (Strg/Cmd+Z)").clicked() { self.do_undo(); }
        if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo")).on_hover_text("Wiederholen").clicked() { self.do_redo(); }
        ui.separator();
        for (t, name) in [(Tool::Pen, "Stift"), (Tool::Erase, "Radierer"), (Tool::Fill, "Füllen"), (Tool::Line, "Linie"), (Tool::Rect, "Rahmen"), (Tool::Select, "Auswahl"), (Tool::Pick, "Pipette")] { ui.selectable_value(&mut self.tool, t, name); }
        ui.separator();
        for (k, name) in [("mh", "Spiegeln H"), ("mv", "Spiegeln V"), ("sl", "Links"), ("sr", "Rechts"), ("su", "Hoch"), ("sd", "Runter"), ("clear", "Leeren")] { if ui.button(name).clicked() { self.op(k); } }
        if self.tool == Tool::Select { for (n, k) in [("Kopieren", 0), ("Ausschneiden", 1), ("Einfügen", 2), ("Auswahl löschen", 3)] { if ui.button(n).clicked() { match k { 0 => self.sel_copy(false), 1 => self.sel_copy(true), 2 => self.sel_paste(), _ => self.sel_clear() } } } }
        ui.separator();
        if self.kind == "charset" && (self.bpp == 1 || self.bpp == 8) && ui.button("Als PNG-Raster …").on_hover_text("Alle Zeichen als Bild speichern (16 je Zeile)").clicked() {
            if let Some(out) = rfd::FileDialog::new().add_filter("PNG", &["png"]).set_file_name(format!("{}.png", self.rel.rsplit('/').next().unwrap_or("zeichensatz").trim_end_matches(".vcchar").trim_end_matches(".vcsprite").trim_end_matches(".vcpmg").trim_end_matches(".vcscreen").trim_end_matches(".vctiles").trim_end_matches(".vcpal").trim_end_matches(".asset.json"))).save_file() {
                match self.save().and_then(|_| vc_media::project::export_sheet(env.root, env.data, &self.rel, &out)) { Ok(()) => env.reqs.info(format!("PNG-Raster gespeichert: {}", out.display())), Err(e) => env.reqs.error(e) }
            }
        }
        if self.bpp <= 4 && ui.button("PNG laden …").on_hover_text("Bild auf Frame-Größe skalieren und auf die Palette abbilden").clicked() { self.import_png(env); }
        if self.kind == "sprite" || self.kind == "pmg" {
            ui.checkbox(&mut self.onion, "Onion-Skin");
            if ui.button("Konvertieren von …").on_hover_text("Sprites aus dem SPD-Format (.spd), Rohdaten oder einer Sprite-Datei eines anderen Systems (C64 ↔ Atari) ins Projekt holen").clicked() { self.pick_import_source(); }
        }
        if self.kind == "charset" { self.template_combo(ui, env); }
        ui.label("Zoom"); if ui.add(egui::Slider::new(&mut self.zoom, 6.0..=48.0).show_value(false)).changed() { self.fit = false; }
        ui.checkbox(&mut self.fit, "Anpassen").on_hover_text("Zoom automatisch an den freien Platz anpassen (Strg+Mausrad oder Regler stellt ihn fest)");
    }

    fn frame_list(&mut self, ui: &mut egui::Ui, pal: &Palettes) {
        if self.kind == "charset" {
            ui.label(RichText::new(format!("Zeichen {} ($ {:02X})", self.fi, self.fi)).strong());
            egui::ScrollArea::vertical().show(ui, |ui| {
                let per = 16; let cell = 16.0;
                for row in 0..(self.frames.len() + per - 1) / per {
                    ui.horizontal(|ui| { ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                        for i in row * per..((row + 1) * per).min(self.frames.len()) {
                            let (r, resp) = ui.allocate_exact_size(vec2(cell, cell), Sense::click());
                            self.paint_frame(ui, pal, i, r.shrink(1.0));
                            if i == self.fi { ui.painter().rect_stroke(r, 1.0, Stroke::new(2.0, crate::theme::warn()), egui::StrokeKind::Inside); }
                            if resp.clicked() { self.fi = i; }
                        }
                    });
                }
            });
            return;
        }
        ui.label(RichText::new("Frames").strong());
        if !self.fixed_frames {
            ui.horizontal(|ui| {
                if ui.button("+").on_hover_text("Leeres Frame anhängen (im Modus des gewählten Frames)").clicked() { self.push_undo(); let mc = self.is_mc(self.fi); self.frames.push(vec![vec![0; self.width]; self.height]); self.frame_mc.resize(self.frames.len() - 1, self.profile_mc); self.frame_mc.push(mc); self.fi = self.frames.len() - 1; }
                if ui.button("Dupl.").clicked() { self.push_undo(); let f = self.frames[self.fi].clone(); let mc = self.is_mc(self.fi); self.frames.insert(self.fi + 1, f); self.frame_mc.resize(self.frames.len() - 1, self.profile_mc); self.frame_mc.insert(self.fi + 1, mc); sprite::animations_after_insert(&mut self.media, self.fi + 1); self.fi += 1; }
                if ui.add_enabled(self.frames.len() > 1, egui::Button::new("−")).on_hover_text("Frame löschen (Schritte der Animationen mit diesem Frame entfallen)").clicked() { self.push_undo(); self.frames.remove(self.fi); if self.fi < self.frame_mc.len() { self.frame_mc.remove(self.fi); } sprite::animations_after_remove(&mut self.media, self.fi); self.fi = self.fi.min(self.frames.len() - 1); }
            });
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..self.frames.len() {
                let label = self.frame_names.get(i).cloned().unwrap_or_else(|| if self.kind == "sprite" { format!("Frame {i}  {}", if self.is_mc(i) { "Multicolor" } else { "Hires" }) } else { format!("Frame {i}") });
                let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(100.0), 62.0), Sense::click());
                let th = Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2((self.width as f32 * self.aspect * 40.0 / self.height as f32).min(64.0).max(24.0), 40.0));
                self.paint_frame(ui, pal, i, th);
                ui.painter().text(r.left_bottom() + vec2(3.0, -2.0), egui::Align2::LEFT_BOTTOM, label, egui::FontId::proportional(12.0), Color32::GRAY);
                if i == self.fi { ui.painter().rect_stroke(r, 3.0, Stroke::new(2.0, crate::theme::accent()), egui::StrokeKind::Inside); }
                if resp.clicked() { self.fi = i; }
            }
        });
    }

    /// Datei wählen und den Import-Dialog vorbereiten
    fn pick_import_source(&mut self) {
        let Some(f) = rfd::FileDialog::new().set_title("Sprites aus Datei wählen").add_filter("Sprites (.spd, .bin, .spr, .prg, .vcsprite, .vcpmg)", &["spd", "bin", "spr", "prg", "dat", "raw", "vcsprite", "vcpmg"]).pick_file() else { return };
        let fname = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let stem = vc_media::naming::stem(&fname); let stem = stem.rsplit_once('.').map_or(stem.to_string(), |x| x.0.to_string());
        let name: String = stem.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let bytes = match std::fs::read(&f) { Ok(b) => b, Err(e) => { self.status = format!("{fname}: {e}"); return } };
        let (kind, info, available) = match ext.as_str() {
            "spd" => match vc_media::spriteimport::spd_info(&bytes) { Ok(i) => (ImportKind::Spd, format!("SPD-Version {}: {} Sprites", i.version, i.sprites), i.sprites), Err(e) => { self.status = e; return } },
            "vcsprite" | "vcpmg" | "json" => match vc_media::parse_media(&String::from_utf8_lossy(&bytes)) { Ok(m) => { let n = m.root["frames"].as_array().map_or(0, |a| a.len()); (ImportKind::Media, format!("Sprite-Datei ({}), {n} Frames", m.profile), n) } Err(e) => { self.status = e; return } },
            _ => (ImportKind::Raw, format!("Rohdaten, {} Bytes", bytes.len()), 0),
        };
        self.imp = Some(ImportDlg { file: f, name, kind, info, offset: "0".into(), count: 0, mode: 0, frame: 0, available });
    }

    fn import_dialog(&mut self, ui: &mut egui::Ui, env: &Env) {
        let Some(mut d) = self.imp.take() else { return };
        let (mut keep, mut go) = (true, false);
        let target_pmg = self.kind == "pmg";
        let to_name = if target_pmg { "Atari Player/Missile" } else if self.profile_mc { "C64 Sprite (Multicolor)" } else { "C64 Sprite (Hires)" };
        egui::Window::new("Sprites aus anderer Quelle übernehmen").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ui.ctx(), |ui| {
            ui.horizontal(|ui| { ui.label("Neuer Name"); ui.text_edit_singleline(&mut d.name); });
            ui.label(RichText::new(d.file.display().to_string()).weak().small());
            ui.label(&d.info);
            ui.label(format!("Ziel: {to_name} (Profil dieses Assets)"));
            if d.kind == ImportKind::Raw {
                ui.horizontal(|ui| { ui.label("Offset"); ui.add(egui::TextEdit::singleline(&mut d.offset).desired_width(80.0)).on_hover_text("Dezimal, $hex oder 0xhex; bei .prg werden die 2 Bytes Ladeadresse zusätzlich übersprungen"); ui.label("Anzahl (0 = alle)"); ui.add(egui::DragValue::new(&mut d.count).range(0..=256)); });
                if !target_pmg { ui.horizontal(|ui| { ui.label("Modus"); ui.radio_value(&mut d.mode, 0, "wie Profil"); ui.radio_value(&mut d.mode, 1, "Hires"); ui.radio_value(&mut d.mode, 2, "Multicolor"); }); }
            }
            if target_pmg && (d.available > 1 || d.kind == ImportKind::Raw) { ui.horizontal(|ui| { ui.label("Sprite Nr."); ui.add(egui::DragValue::new(&mut d.frame).range(0..=if d.available > 0 { d.available as u32 - 1 } else { 255 })); ui.label(RichText::new("(ein PMG-Asset hat einen Satz aus 4 Playern)").weak().small()); }); }
            ui.label(RichText::new("Wird als neue Datei im Projekt angelegt. Farben und Größen werden angenähert; Hinweise stehen danach in der Ausgabe.").weak().small());
            ui.horizontal(|ui| { if ui.button("OK").clicked() { go = true; keep = false; } if ui.button("Abbrechen").clicked() { keep = false; } });
        });
        if go {
            let dir = self.rel.rsplit_once('/').map(|x| x.0.to_string()).filter(|x| !x.is_empty()).unwrap_or_else(|| "graphics".into());
            let parse = |t: &str| -> usize { let t = t.trim(); t.strip_prefix('$').or_else(|| t.strip_prefix("0x")).and_then(|h| usize::from_str_radix(h, 16).ok()).or_else(|| t.parse().ok()).unwrap_or(0) };
            let opts = vc_media::spriteimport::SprOpts { offset: parse(&d.offset), count: d.count as usize, mc: match d.mode { 1 => Some(false), 2 => Some(true), _ => None }, frame: d.frame as usize };
            match self.save().and_then(|_| vc_media::project::import_sprite(env.root, env.data, &d.file, &self.profile, &d.name, &dir, &opts)) {
                Ok(r) => {
                    let rel = r["path"].as_str().unwrap_or("").to_string();
                    for w in r["warnings"].as_array().into_iter().flatten().filter_map(|w| w.as_str()) { env.reqs.warn(format!("Hinweis: {w}")); }
                    env.reqs.info(format!("Angelegt: {rel} ({} Frames)", r["count"])); env.reqs.open(&rel);
                }
                Err(e) => { env.reqs.error(e.clone()); self.status = e; }
            }
        }
        if keep { self.imp = Some(d); }
    }

    /// Nur Frame-Zahl und Animationen, so wie sie `sprite::parse_animations` prüft (der Wert `media` enthält die Frames noch im Stand beim Laden)
    fn anim_view(&self) -> Value { json!({ "frames": vec![Value::Null; self.frames.len()], "animations": self.media["animations"].clone() }) }

    /// Animationen: Folgen von Frames mit Dauer (Bilder zu 1/50 s) und Ablauf; Vorschau in Echtzeit, Export als Tabellen
    fn anim_panel(&mut self, ui: &mut egui::Ui, pal: &Palettes) {
        #[derive(Clone)]
        enum Op { New, Delete, Rename(String), Mode(AnimMode), AddStep(usize), DelStep(usize), Move(usize, isize), Delay(usize, u32) }
        let anims = match sprite::parse_animations(&self.anim_view()) { Ok(a) => a, Err(e) => { ui.colored_label(crate::highlight::PAL_ERR, format!("Animationen fehlerhaft: {e}")); return; } };
        let n = self.sel_anim.min(anims.len().saturating_sub(1)); if n != self.sel_anim { self.sel_anim = n; }
        let cur = anims.get(n).cloned();
        if let Some(a) = &cur { if self.anim_name != a.name && !self.coalesce.as_deref().map_or(false, |c| c == "rename") { self.anim_name = a.name.clone(); } }
        let mut ops: Vec<Op> = vec![]; let mut select: Option<usize> = None; let restart = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Animationen").strong());
            if ui.button("+ Neu").on_hover_text("Neue Animation mit dem gewählten Frame").clicked() { ops.push(Op::New); }
            if let Some(a) = &cur {
                egui::ComboBox::from_id_salt(("bm-anim-sel", &self.rel)).selected_text(&a.name).show_ui(ui, |ui| { for (i, x) in anims.iter().enumerate() { if ui.selectable_label(i == n, &x.name).clicked() { select = Some(i); } } });
                let r = ui.add(egui::TextEdit::singleline(&mut self.anim_name).desired_width(110.0).hint_text("Name"));
                if r.changed() { self.coalesce = Some("rename".into()); }
                if r.lost_focus() { self.coalesce = None; let nm = self.anim_name.clone(); if nm != a.name { if sprite::valid_name(&nm) && !anims.iter().any(|x| x.name == nm) { ops.push(Op::Rename(nm)); } else { self.status = "Name: Buchstaben, Ziffern, _; nicht doppelt".into(); self.anim_name = a.name.clone(); } } }
                let mut m = a.mode;
                egui::ComboBox::from_id_salt(("bm-anim-mode", &self.rel)).selected_text(m.label()).show_ui(ui, |ui| { for k in [AnimMode::Loop, AnimMode::PingPong, AnimMode::Once] { ui.selectable_value(&mut m, k, k.label()); } });
                if m != a.mode { ops.push(Op::Mode(m)); }
                if ui.button("Frame anhängen").on_hover_text("Das gewählte Frame als nächsten Schritt anfügen").clicked() { ops.push(Op::AddStep(self.fi)); }
                if ui.button("Animation löschen").clicked() { ops.push(Op::Delete); }
                ui.label(RichText::new(format!("{} Bilder = {:.2} s", a.total_ticks(), a.total_ticks() as f32 / 50.0)).weak().small());
            } else { ui.label(RichText::new("Noch keine Animation. „+ Neu“ legt eine Folge aus Frames an, die im Loop läuft.").weak()); }
        });
        if let Some(a) = cur.clone() {
            ui.horizontal_top(|ui| {
                // Vorschau in Echtzeit
                let ex = a.expanded(); let now = ui.input(|i| i.time);
                if restart || self.anim_t0 == 0.0 { self.anim_t0 = now; }
                let step = a.step_at(now - self.anim_t0);
                ui.vertical(|ui| {
                    let (r, resp) = ui.allocate_exact_size(vec2(24.0 * 2.5, 21.0 * 2.5), Sense::click());
                    if let Some((f, _)) = ex.get(step) { self.paint_frame(ui, pal, *f, r); }
                    if resp.on_hover_text("Klick: von vorn abspielen").clicked() { self.anim_t0 = now; }
                    ui.label(RichText::new(format!("Schritt {}/{}", step + 1, ex.len().max(1))).weak().small());
                });
                ui.separator();
                egui::ScrollArea::horizontal().show(ui, |ui| { ui.horizontal(|ui| {
                    for (i, (f, d)) in a.steps.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.set_width(58.0);
                            let (r, resp) = ui.allocate_exact_size(vec2(48.0, 42.0), Sense::click());
                            self.paint_frame(ui, pal, *f, r);
                            if resp.on_hover_text("Klick wählt dieses Frame zum Bearbeiten").clicked() { self.fi = *f; }
                            ui.label(RichText::new(format!("Frame {f}")).small().weak());
                            let mut dv = *d; if ui.add(egui::DragValue::new(&mut dv).range(1..=255).suffix(" B")).on_hover_text("Dauer in Bildern (1/50 s)").changed() { ops.push(Op::Delay(i, dv)); }
                            ui.horizontal(|ui| {
                                if ui.add_enabled(i > 0, egui::Button::new("◀").small()).clicked() { ops.push(Op::Move(i, -1)); }
                                if ui.add_enabled(i + 1 < a.steps.len(), egui::Button::new("▶").small()).clicked() { ops.push(Op::Move(i, 1)); }
                                if ui.small_button("×").on_hover_text("Schritt entfernen").clicked() { ops.push(Op::DelStep(i)); }
                            });
                        });
                    }
                }); });
            });
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(20));
        }
        if let Some(i) = select { self.sel_anim = i; self.anim_t0 = 0.0; self.coalesce = None; }
        if ops.is_empty() { return; }
        // Änderungen anwenden (Delay-Änderungen am selben Schritt zählen als ein Undo-Schritt)
        let key = match ops.first() { Some(Op::Delay(i, _)) => Some(format!("delay:{n}:{i}")), _ => None };
        if key.is_none() || self.coalesce != key { self.push_undo(); }
        self.coalesce = key;
        let mut list = anims;
        for op in ops { match op {
            Op::New => { let mut nm = "anim".to_string(); let mut k = 1; while list.iter().any(|x| x.name == nm) { k += 1; nm = format!("anim{k}"); } list.push(Anim { name: nm, mode: AnimMode::Loop, steps: vec![(self.fi, 4)] }); self.sel_anim = list.len() - 1; self.anim_t0 = 0.0; }
            Op::Delete => { if n < list.len() { list.remove(n); self.sel_anim = n.saturating_sub(1); self.anim_t0 = 0.0; } }
            Op::Rename(nm) => { if let Some(a) = list.get_mut(n) { a.name = nm; } }
            Op::Mode(m) => { if let Some(a) = list.get_mut(n) { a.mode = m; self.anim_t0 = 0.0; } }
            Op::AddStep(f) => { if let Some(a) = list.get_mut(n) { let d = a.steps.last().map_or(4, |s| s.1); a.steps.push((f, d)); } }
            Op::DelStep(i) => { if let Some(a) = list.get_mut(n) { if i < a.steps.len() { a.steps.remove(i); } } }
            Op::Move(i, dir) => { if let Some(a) = list.get_mut(n) { let j = i as isize + dir; if j >= 0 && (j as usize) < a.steps.len() { a.steps.swap(i, j as usize); } } }
            Op::Delay(i, d) => { if let Some(a) = list.get_mut(n) { if let Some(s) = a.steps.get_mut(i) { s.1 = d; } } }
        } }
        self.media["animations"] = sprite::animations_to_json(&list);
    }

    /// Einzeltasten (abschaltbar): D Stift, E Radierer, F Füllen, L Linie, R Rahmen, S Auswahl, I Pipette, X/Y Spiegeln, C Hires/Multicolor,
    /// 1–4 Pixelwert, ←/→ Frame. Nur wenn der Zeiger über diesem Editor steht und kein Textfeld Fokus hat.
    fn hotkeys(&mut self, ui: &egui::Ui) {
        use egui::Key;
        if !HOTKEYS.load(std::sync::atomic::Ordering::Relaxed) || ui.ctx().egui_wants_keyboard_input() || !ui.rect_contains_pointer(ui.max_rect()) { return; }
        let pressed: Vec<Key> = ui.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl || i.modifiers.alt { return vec![]; }
            [Key::D, Key::E, Key::F, Key::L, Key::R, Key::S, Key::I, Key::X, Key::Y, Key::C, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::ArrowLeft, Key::ArrowRight].into_iter().filter(|k| i.key_pressed(*k)).collect()
        });
        for k in pressed {
            match k {
                Key::D => self.tool = Tool::Pen, Key::E => self.tool = Tool::Erase, Key::F => self.tool = Tool::Fill, Key::L => self.tool = Tool::Line,
                Key::R => self.tool = Tool::Rect, Key::S => self.tool = Tool::Select, Key::I => self.tool = Tool::Pick,
                Key::X => self.op("mh"), Key::Y => self.op("mv"),
                Key::C => { if self.frame_modes.len() >= 2 { let mc = self.is_mc(self.fi); self.set_frame_mode(!mc); } }
                Key::Num1 | Key::Num2 | Key::Num3 | Key::Num4 => {
                    let n = match k { Key::Num1 => 0, Key::Num2 => 1, Key::Num3 => 2, _ => 3 };
                    if self.bpp < 8 && n < self.values.len() { self.pv = n as u8; }
                }
                Key::ArrowLeft => { if self.fi > 0 { self.fi -= 1; } }
                Key::ArrowRight => { if self.fi + 1 < self.frames.len() { self.fi += 1; } }
                _ => {}
            }
        }
    }

    /// Hardware-Flags: Bitfeld-Schalter (Sprites der VIC-II-Systeme) bzw. Größe und Priorität (Atari Player/Missile)
    fn hw_block(&mut self, ui: &mut egui::Ui) {
        if has_sprite_flags(&self.target, &self.kind) {
            studio::caption(ui, "VIC-II Sprite-Flags");
            let mut bits = flags_from_hw(&self.media["hw"]) | (self.is_mc(self.fi) as u8);
            let before = bits;
            crate::ui::widgets::bitfield(ui, &mut bits, SPRITE_FLAGS);
            if bits != before {
                if (bits ^ before) & 0b1110 != 0 { self.push_undo(); flags_to_hw(&mut self.media, bits); }
                if (bits ^ before) & 1 != 0 { self.set_frame_mode(bits & 1 != 0); }
            }
            ui.label(RichText::new("MC folgt dem Modus des gewählten Frames. X2, Y2 und BG gelten für die ganze Datei und stehen als Konstanten im Export.").small().color(crate::ui::tokens::t().dim));
        } else if self.kind == "pmg" {
            studio::caption(ui, "Player/Missile");
            let mut size = self.media["hw"]["size"].as_u64().unwrap_or(0).to_string();
            ui.label(RichText::new(format!("Breite (SIZEP{} ${:04X})", self.fi, 0xD008 + self.fi)).small());
            if crate::ui::widgets::segmented(ui, &mut size, &[("0", "einfach"), ("1", "doppelt"), ("3", "vierfach")]) { self.push_undo(); self.media["hw"]["size"] = json!(size.parse::<u64>().unwrap_or(0)); }
            let mut prior = self.media["hw"]["prior"].as_str().unwrap_or("player").to_string();
            ui.label(RichText::new("Priorität (PRIOR $D01B)").small());
            if crate::ui::widgets::segmented(ui, &mut prior, &[("player", "Player vor Feld"), ("field", "Feld vor Player")]) { self.push_undo(); self.media["hw"]["prior"] = json!(prior); }
        }
    }

    fn side(&mut self, ui: &mut egui::Ui, pal: &Palettes) {
        if self.bpp == 8 {
            ui.label(RichText::new(format!("Palettenfarbe {} ($ {:02X})", self.pv, self.pv)).strong());
            ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
            for row in 0..16 { ui.horizontal(|ui| { for k in 0..16 { let i = row * 16 + k; let (r, resp) = ui.allocate_exact_size(vec2(11.0, 11.0), Sense::click()); ui.painter().rect_filled(r, 1.0, pal.color(&self.target, i)); if i == self.pv as usize { ui.painter().rect_stroke(r, 1.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Outside); } if resp.clicked() { self.pv = i as u8; } } }); }
            ui.add_space(8.0); ui.label(RichText::new("Vorschau").strong());
            let sz = vec2(self.width as f32 * self.aspect * 4.0, self.height as f32 * 4.0); let (r, _) = ui.allocate_exact_size(sz, Sense::hover()); self.paint_frame(ui, pal, self.fi, r);
            return;
        }
        if self.frame_modes.len() >= 2 {
            ui.label(RichText::new("Modus dieses Frames").strong());
            let mc = self.is_mc(self.fi); let mut to: Option<bool> = None;
            ui.horizontal(|ui| {
                if ui.selectable_label(!mc, self.frame_modes[0].label.as_str()).on_hover_text(self.frame_modes[0].hint.as_str()).clicked() { to = Some(false); }
                if ui.selectable_label(mc, self.frame_modes[1].label.as_str()).on_hover_text(self.frame_modes[1].hint.as_str()).clicked() { to = Some(true); }
            });
            ui.label(RichText::new("Frames dürfen gemischt sein; beim Wechsel werden die Pixel umgerechnet (Strg+Z macht es rückgängig).").weak().small());
            if let Some(m) = to { self.set_frame_mode(m); }
            ui.add_space(6.0);
        }
        studio::caption(ui, "Pixelwerte");
        for (i, name) in self.values.clone().iter().enumerate() {
            let reg = value_reg(&self.target, &self.kind, self.bpp, i, self.fi, name);
            if studio::value_row(ui, self.pv as usize == i, self.value_color(pal, i as u8), &studio::value_bits(i, self.bpp), name, reg.as_deref()) { self.pv = i as u8; }
        }
        self.hw_block(ui);
        ui.add_space(8.0);
        if self.target == "atari8" { studio::caption(ui, "Videonorm"); let mut v = if ATARI_NTSC.load(std::sync::atomic::Ordering::Relaxed) { "ntsc".to_string() } else { "pal".to_string() }; if crate::ui::widgets::segmented(ui, &mut v, &[("pal", "PAL"), ("ntsc", "NTSC")]) { ATARI_NTSC.store(v == "ntsc", std::sync::atomic::Ordering::Relaxed); } }
        for (key, label) in self.color_keys() {
            studio::caption(ui, &label);
            let cur = self.cnum(&key, 1);
            let n = if self.target == "atari8" { 128 } else { 16 }; let per = if self.target == "atari8" { 16 } else { 8 };
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
            for row in 0..n / per {
                ui.horizontal(|ui| { for k in 0..per {
                    let idx = if self.target == "atari8" { (row * per + k) * 2 } else { row * per + k };
                    let (r, resp) = ui.allocate_exact_size(vec2(if self.target == "atari8" { 11.0 } else { 20.0 }, if self.target == "atari8" { 11.0 } else { 20.0 }), Sense::click());
                    ui.painter().rect_filled(r, 2.0, pal.color(&self.target, idx));
                    if idx == cur { ui.painter().rect_stroke(r, 2.0, Stroke::new(2.0, crate::theme::select()), egui::StrokeKind::Outside); }
                    if resp.clicked() { self.push_undo(); self.media["colors"][&key] = json!(idx); }
                } });
            }
            ui.add_space(6.0);
        }
        studio::caption(ui, "Vorschau");
        let sz = vec2(self.width as f32 * self.aspect * 3.0, self.height as f32 * 3.0);
        let show = if self.anim && self.frames.len() > 1 { ((ui.input(|i| i.time) * 6.0) as usize) % self.frames.len() } else { self.fi };
        let (r, _) = ui.allocate_exact_size(sz, Sense::hover()); self.paint_frame(ui, pal, show, r);
        if self.kind == "sprite" && self.frames.len() > 1 { if ui.button(if self.anim { "■ Animation" } else { "▶ Animation" }).clicked() { self.anim = !self.anim; } if self.anim { ui.ctx().request_repaint_after(std::time::Duration::from_millis(80)); } }
    }

    fn canvas(&mut self, ui: &mut egui::Ui, pal: &Palettes) {
        if self.fit { self.zoom = studio::fit_zoom(self.avail, vec2(self.aspect * self.width as f32, self.height as f32), vec2(24.0, 24.0), 6.0, 48.0); }
        let (cw, ch) = (self.zoom * self.aspect, self.zoom);
        let size = vec2(cw * self.width as f32, ch * self.height as f32);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
        if resp.hovered() { let z = ui.input(|i| i.zoom_delta()); if z != 1.0 { self.zoom = (self.zoom * z).clamp(6.0, 48.0); self.fit = false; } }
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 0.0, Color32::from_rgb(0x12, 0x13, 0x17));
        for y in 0..self.height { for x in 0..self.width {
            let r = Rect::from_min_size(pos2(rect.left() + x as f32 * cw, rect.top() + y as f32 * ch), vec2(cw, ch));
            match self.value_color(pal, self.get(x, y)) {
                Some(c) => { p.rect_filled(r, 0.0, c); }
                None => { if (x + y) % 2 == 0 { p.rect_filled(r, 0.0, Color32::from_rgb(0x1c, 0x1e, 0x25)); } }
            }
        } }
        if self.onion && self.fi > 0 && self.is_mc(self.fi - 1) == self.is_mc(self.fi) { for y in 0..self.height { for x in 0..self.width {
            if self.get(x, y) != 0 { continue; }
            let pv = self.frames[self.fi - 1].get(y).and_then(|r| r.get(x)).copied().unwrap_or(0);
            if let Some(c) = self.value_color_for(pal, pv, self.fi - 1) { let r = Rect::from_min_size(pos2(rect.left() + x as f32 * cw, rect.top() + y as f32 * ch), vec2(cw, ch)); p.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 70)); }
        } } }
        let grid = Stroke::new(1.0, Color32::from_white_alpha(18));
        for x in 0..=self.width { let px = rect.left() + x as f32 * cw; p.line_segment([pos2(px, rect.top()), pos2(px, rect.bottom())], grid); }
        for y in 0..=self.height { let py = rect.top() + y as f32 * ch; p.line_segment([pos2(rect.left(), py), pos2(rect.right(), py)], grid); }
        let (gw, gh) = (self.width, self.height);
        let cell_at = |pt: egui::Pos2| -> Option<(usize, usize)> {
            if !rect.contains(pt) { return None; }
            Some((((pt.x - rect.left()) / cw) as usize, ((pt.y - rect.top()) / ch) as usize)).filter(|(x, y)| *x < gw && *y < gh)
        };
        let secondary = resp.dragged_by(egui::PointerButton::Secondary) || resp.secondary_clicked();
        let value = if secondary || self.tool == Tool::Erase { 0 } else { self.pv };
        if resp.drag_started() || resp.clicked() || resp.secondary_clicked() {
            if let Some(c) = resp.interact_pointer_pos().and_then(&cell_at) {
                match self.tool {
                    Tool::Pick => { self.pv = self.get(c.0, c.1); }
                    Tool::Fill => { self.push_undo(); self.flood(c.0, c.1, value); }
                    Tool::Line | Tool::Rect | Tool::Select => { self.drag_from = Some(c); if self.tool == Tool::Select { self.sel = Some((c.0, c.1, c.0, c.1)); } }
                    _ => { self.push_undo(); self.set(c.0, c.1, value); }
                }
            }
        } else if resp.dragged() && matches!(self.tool, Tool::Pen | Tool::Erase) {
            if let Some(c) = resp.interact_pointer_pos().and_then(&cell_at) { if self.get(c.0, c.1) != value { self.set(c.0, c.1, value); } }
        }
        if resp.dragged() && self.tool == Tool::Select { if let (Some(a), Some(c)) = (self.drag_from, resp.interact_pointer_pos().and_then(&cell_at)) { self.sel = Some((a.0.min(c.0), a.1.min(c.1), a.0.max(c.0), a.1.max(c.1))); } }
        if let Some((x0, y0, x1, y1)) = self.sel { let r = Rect::from_min_max(pos2(rect.left() + x0 as f32 * cw, rect.top() + y0 as f32 * ch), pos2(rect.left() + (x1 + 1) as f32 * cw, rect.top() + (y1 + 1) as f32 * ch)); p.rect_stroke(r, 0.0, Stroke::new(2.0, crate::theme::warn()), egui::StrokeKind::Inside); }
        if resp.drag_stopped() && self.tool == Tool::Rect { if let (Some(a), Some(c)) = (self.drag_from.take(), resp.interact_pointer_pos().and_then(&cell_at)) { self.push_undo(); self.rect_outline(a, c, value); } }
        if resp.drag_stopped() && self.tool == Tool::Line {
            if let (Some(a), Some(c)) = (self.drag_from.take(), resp.interact_pointer_pos().and_then(&cell_at)) { self.push_undo(); self.line(a, c, value); }
        }
        ui.add_space(6.0);
        ui.label(RichText::new(format!("{} · Frame {}{} · {}×{} · linke Taste zeichnet, rechte löscht", self.profile_title(), self.fi, if self.kind == "sprite" { if self.is_mc(self.fi) { " (Multicolor)" } else { " (Hires)" } } else { "" }, self.width, self.height)).weak().small());
    }
    fn profile_title(&self) -> &str { match self.kind.as_str() { "sprite" => "Sprite", "pmg" => "Player/Missile", _ => "Zeichensatz" } }
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
        let mut b = Bitmap::open(&dir, "hero.asset.json", &data).unwrap();
        assert_eq!(b.to_value()["frames"], orig["frames"], "Laden und Zurückschreiben ändert nichts");
        assert!(!b.dirty());
        b.push_undo(); b.set(0, 0, 3); b.flood(5, 5, 2); b.line((0, 20), (11, 20), 1);
        assert!(b.dirty());
        b.save().unwrap();
        let b2 = Bitmap::open(&dir, "hero.asset.json", &data).unwrap();
        assert_eq!(b2.get(0, 0), 3); assert_eq!(b2.get(11, 20), 1);
        b.do_undo(); assert_eq!(b.get(0, 0), orig["frames"][0]["rows"][0].as_str().unwrap().chars().next().unwrap().to_digit(16).unwrap() as u8);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn pmg_and_charset_open() {
        let data = root().join("data");
        for (rel, kind) in [("examples/atari/pmg_ship/graphics/ship.vcpmg", "pmg"), ("examples/atari/gfx_demo/graphics/afont.vcchar", "charset")] {
            let p = root().join(rel); let b = Bitmap::open(p.parent().unwrap(), p.file_name().unwrap().to_str().unwrap(), &data).unwrap();
            assert_eq!(b.kind, kind); assert!(!b.frames.is_empty());
        }
    }

    #[test]
    fn rect_selection_paste_and_fcm_roundtrip() {
        let data = root().join("data"); let dir = std::env::temp_dir().join(format!("vc-bm2-{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(root().join("examples/c64/sprite_demo/graphics/hero.vcsprite"), dir.join("hero.asset.json")).unwrap();
        let mut b = Bitmap::open(&dir, "hero.asset.json", &data).unwrap();
        b.rect_outline((1, 1), (4, 3), 2); assert_eq!((b.get(1, 1), b.get(4, 3), b.get(2, 2)), (2, 2, b.get(2, 2)));
        b.sel = Some((1, 1, 4, 3)); b.sel_copy(true); assert_eq!(b.get(1, 1), 0); assert!(b.clip.is_some());
        b.sel = Some((6, 6, 6, 6)); b.sel_paste(); assert_eq!(b.get(6, 6), 2, "Ecke eingefügt");
        // MEGA65-Vollfarbzeichensatz (8 Bit je Pixel, zwei Hex-Ziffern)
        let profiles = vc_media::load_profiles(&data).unwrap(); let p = vc_media::profile(&profiles, "mega65.char.fcm").unwrap();
        std::fs::write(dir.join("fcm.asset.json"), vc_media::project::format_media(&vc_media::new_media(p, "fcm"))).unwrap();
        let mut f = Bitmap::open(&dir, "fcm.asset.json", &data).unwrap(); assert_eq!(f.bpp, 8);
        f.fi = 3; f.set(2, 2, 200); f.save().unwrap();
        let f2 = Bitmap::open(&dir, "fcm.asset.json", &data).unwrap(); assert_eq!(f2.frames.len(), 256.min(f2.frames.len())); assert_eq!({ let mut g = f2; g.fi = 3; g.get(2, 2) }, 200);
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
        let mut b = Bitmap::open(&dir, "held.vcsprite", &data).unwrap();
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
        b.do_undo(); assert!(!b.is_mc(0)); assert_eq!(b.frames[0][0].len(), 24);
        // Frame duplizieren verschiebt die Frame-Nummern der Animationen
        b.push_undo(); let f = b.frames[1].clone(); b.frames.insert(2, f); b.frame_mc.insert(2, true); sprite::animations_after_insert(&mut b.media, 2);
        assert_eq!(sprite::parse_animations(&b.anim_view()).unwrap()[0].steps, vec![(0, 4), (1, 3), (3, 5)]);
        b.save().unwrap();
        let b2 = Bitmap::open(&dir, "held.vcsprite", &data).unwrap();
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

    #[test]
    fn export_nennt_flags_als_konstanten() {
        let data = root().join("data");
        let tmp = std::env::temp_dir().join(format!("vc_hwflags_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let rows: Vec<String> = (0..21).map(|_| "0".repeat(24)).collect();
        let mut m = json!({ "format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": "hero", "colors": { "sprite": 1 }, "frames": [{ "rows": rows }] });
        std::fs::write(tmp.join("hero.vcsprite"), vc_media::project::format_media(&m)).unwrap();
        let t = vc_media::project::export(&tmp, &data, "hero.vcsprite", "asm", None, None, 10, 10).unwrap();
        assert!(!t["preview"].as_str().unwrap().contains("expand_x"));   // ohne hw bleibt die Ausgabe unverändert
        flags_to_hw(&mut m, 0b0110);
        std::fs::write(tmp.join("hero.vcsprite"), vc_media::project::format_media(&m)).unwrap();
        let t = vc_media::project::export(&tmp, &data, "hero.vcsprite", "asm", None, None, 10, 10).unwrap();
        let p = t["preview"].as_str().unwrap();
        assert!(p.contains(".const hero_expand_x = 1") && p.contains(".const hero_expand_y = 1") && p.contains(".const hero_behind = 0"), "{p}");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn studio_zeichnet_mit_exportdialog_hell_und_dunkel() {
        let data = root().join("data");
        let pal = Palettes::load(&data);
        let (audio, reqs) = (crate::audio::Audio::new(), Requests::default());
        let env = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &root().join("examples/c64/sprite_demo"), data: &data };
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        for (rel, flags) in [("graphics/hero.vcsprite", true), ("graphics/hero.vcsprite", false)] {
            let mut b = Bitmap::open(env.root, rel, &data).unwrap();
            if flags { b.exp = Some(crate::exportdlg::ExportDlg::new(rel, "sprite", "hero")); }
            for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, 640.0))), ..Default::default() }, |ui| b.ui(ui, &env)); o.textures_delta.clear(); }
            if flags { assert!(b.exp.is_some(), "Export-Dialog des Sprites bleibt offen"); }
        }
        let c = Bitmap::open(&root().join("examples/c64/jumprun"), "graphics/hero_base.vcsprite", &data);
        assert!(c.is_ok());
        // Zeichensatz und PMG
        let tmp = std::env::temp_dir().join(format!("vc_studio_ui_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let rel = vc_media::project::create_with(&tmp, &data, "font", "c64.char.hires", "", false).unwrap()["path"].as_str().unwrap().to_string();
        let env2 = Env { pal: &pal, audio: &audio, reqs: &reqs, root: &tmp, data: &data };
        let mut b = Bitmap::open(&tmp, &rel, &data).unwrap();
        b.exp = Some(crate::exportdlg::ExportDlg::new(&rel, "charset", "font"));
        for _ in 0..3 { let mut o = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, 640.0))), ..Default::default() }, |ui| b.ui(ui, &env2)); o.textures_delta.clear(); }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
