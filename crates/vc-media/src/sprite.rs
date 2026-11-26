// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprites: Hires-/Multicolor-Modus je Frame und Animationsfolgen.
//!
//! Ein Sprite-Media hat ein Profil (hires 24×21 oder multicolor 12×21); einzelne Frames dürfen mit `"mode": "hires" | "multicolor"` davon abweichen.
//! Animationen stehen unter `"animations"`: `{ "name": "lauf", "mode": "loop" | "pingpong" | "once", "steps": [ { "frame": 0, "delay": 4 }, … ] }`
//! (`delay` in Bildern, 1/50 s). Beim Export entstehen Tabellen mit den Sprite-Zeigern und den Dauern.
use crate::Profile;
use serde_json::{json, Value};

/// Ist das Frame ein Multicolor-Frame? (eigener `mode`, sonst der Modus des Profils)
pub fn frame_is_mc(p: &Profile, frame: &Value) -> bool {
    match frame["mode"].as_str() { Some("multicolor") => true, Some("hires") => false, _ => p.bpp == 2 }
}

/// Hat das Sprite-Profil einen Modus je Frame (`perFrameMode` mit der Liste `modes` im Profil, z. B. VIC-II-Sprites: Hires und Multicolor)?
pub fn has_frame_modes(p: &Profile) -> bool { p.kind == "sprite" && p.raw["perFrameMode"].as_bool().unwrap_or(false) && p.raw["modes"].as_array().map_or(false, |m| !m.is_empty()) }

/// Breite (in Pixeln) und Bits je Pixel eines Frames; bei Profilen mit Modus je Frame aus der Liste `modes`
pub fn frame_geometry(p: &Profile, frame: &Value) -> (usize, usize) {
    if !has_frame_modes(p) { return (p.width, p.bpp); }
    let id = if frame_is_mc(p, frame) { "multicolor" } else { "hires" };
    let m = p.raw["modes"].as_array().and_then(|a| a.iter().find(|m| m["id"] == id));
    match m { Some(m) => (m["width"].as_u64().unwrap_or(p.width as u64) as usize, m["bpp"].as_u64().unwrap_or(p.bpp as u64) as usize), None => (p.width, p.bpp) }
}

/// Hires → Multicolor: je zwei Pixel werden einer (gesetzt, wenn eines gesetzt ist; Wert 2 = Sprite-Farbe, Bitpaar 10 wie in der Hardware)
pub fn hires_to_mc(rows: &[Vec<u8>]) -> Vec<Vec<u8>> {
    rows.iter().map(|r| r.chunks(2).map(|c| if c.iter().any(|&v| v != 0) { 2 } else { 0 }).collect()).collect()
}

/// Multicolor → Hires: jedes Pixel wird doppelt breit; alle Farben außer „transparent“ werden zur Sprite-Farbe
pub fn mc_to_hires(rows: &[Vec<u8>]) -> Vec<Vec<u8>> {
    rows.iter().map(|r| r.iter().flat_map(|&v| { let b = if v != 0 { 1 } else { 0 }; [b, b] }).collect()).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimMode { Loop, PingPong, Once }
impl AnimMode {
    pub fn parse(s: &str) -> AnimMode { match s { "pingpong" => AnimMode::PingPong, "once" => AnimMode::Once, _ => AnimMode::Loop } }
    pub fn key(self) -> &'static str { match self { AnimMode::Loop => "loop", AnimMode::PingPong => "pingpong", AnimMode::Once => "once" } }
    pub fn label(self) -> String { match self { AnimMode::Loop => vc_i18n::t!("media-schleife"), AnimMode::PingPong => vc_i18n::t!("media-hin-und-zurueck"), AnimMode::Once => vc_i18n::t!("media-einmal") } }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Anim { pub name: String, pub mode: AnimMode, pub steps: Vec<(usize, u32)> }

impl Anim {
    /// Ablauf, wie er abgespielt wird: „Hin und zurück“ wird ausgeschrieben (ohne doppelte Endbilder), danach läuft alles als Schleife bzw. einmal
    pub fn expanded(&self) -> Vec<(usize, u32)> {
        let mut v = self.steps.clone();
        if self.mode == AnimMode::PingPong && v.len() > 2 { for i in (1..v.len() - 1).rev() { v.push(self.steps[i]); } }
        v
    }
    pub fn total_ticks(&self) -> u32 { self.expanded().iter().map(|s| s.1).sum() }
    /// Welcher Schritt (Index in `expanded()`) läuft zum Zeitpunkt `t` Sekunden (50 Bilder/s)?
    pub fn step_at(&self, t: f64) -> usize {
        let ex = self.expanded(); let total: u32 = ex.iter().map(|s| s.1).sum();
        if ex.is_empty() || total == 0 { return 0; }
        let tick = (t * 50.0).max(0.0) as u64;
        let tick = if self.mode == AnimMode::Once { tick.min(total as u64 - 1) } else { tick % total as u64 } as u32;
        let mut acc = 0; for (i, s) in ex.iter().enumerate() { acc += s.1; if tick < acc { return i; } }
        ex.len() - 1
    }
}

pub fn valid_name(n: &str) -> bool { !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !n.starts_with(|c: char| c.is_ascii_digit()) }

/// Animationen aus dem Media lesen und prüfen (Frames vorhanden, Namen eindeutig, Dauer 1..255)
pub fn parse_animations(root: &Value) -> Result<Vec<Anim>, String> {
    let n = root["frames"].as_array().map_or(0, |f| f.len());
    let mut out: Vec<Anim> = vec![];
    for (i, a) in root["animations"].as_array().into_iter().flatten().enumerate() {
        let name = a["name"].as_str().unwrap_or("").to_string();
        if !valid_name(&name) { return Err(vc_i18n::t!("media-animation-p1-name-name-ungueltig-buchstaben", p1 = i + 1, name = &name)); }
        if out.iter().any(|o| o.name == name) { return Err(format!("Animation '{name}' kommt doppelt vor")); }
        let mut steps = vec![];
        for (k, s) in a["steps"].as_array().into_iter().flatten().enumerate() {
            let f = s["frame"].as_u64().ok_or(vc_i18n::t!("media-animation-name-schritt-p2-frame-fehlt", name = &name, p2 = k + 1))? as usize;
            if f >= n { return Err(vc_i18n::t!("media-animation-name-schritt-p2-frame-f", name = &name, p2 = k + 1, f = &f, n = &n)); }
            let d = s["delay"].as_u64().unwrap_or(4);
            if !(1..=255).contains(&d) { return Err(vc_i18n::t!("media-animation-name-schritt-p2-dauer-d", name = &name, p2 = k + 1, d = &d)); }
            steps.push((f, d as u32));
        }
        out.push(Anim { name, mode: AnimMode::parse(a["mode"].as_str().unwrap_or("loop")), steps });
    }
    Ok(out)
}

pub fn animations_to_json(anims: &[Anim]) -> Value {
    Value::Array(anims.iter().map(|a| json!({ "name": a.name, "mode": a.mode.key(), "steps": a.steps.iter().map(|(f, d)| json!({ "frame": f, "delay": d })).collect::<Vec<_>>() })).collect())
}

/// Nach dem Einfügen eines Frames an Position `at` verschieben sich die Frame-Nummern ab `at`
pub fn animations_after_insert(root: &mut Value, at: usize) {
    for a in root["animations"].as_array_mut().into_iter().flatten() { for s in a["steps"].as_array_mut().into_iter().flatten() { if let Some(f) = s["frame"].as_u64() { if f as usize >= at { s["frame"] = json!(f + 1); } } } }
}

/// Nach dem Löschen von Frame `at`: Schritte mit diesem Frame entfallen, spätere Nummern rücken nach
pub fn animations_after_remove(root: &mut Value, at: usize) {
    for a in root["animations"].as_array_mut().into_iter().flatten() {
        if let Some(steps) = a["steps"].as_array_mut() {
            steps.retain(|s| s["frame"].as_u64().map_or(true, |f| f as usize != at));
            for s in steps.iter_mut() { if let Some(f) = s["frame"].as_u64() { if f as usize > at { s["frame"] = json!(f - 1); } } }
        }
    }
}

/// Konstanten (ohne Daten): Frame-Zahl und Multicolor-Maske. Sie belegen keinen Speicher und stehen in der Haupt-Exportdatei.
pub fn export_consts(label: &str, p: &Profile, root: &Value, sbas: bool) -> Result<String, String> {
    let c = if sbas { "const" } else { ".const" };
    if p.kind == "pmg" {
        // Player/Missile (Atari): Breite je Spieler (SIZEP0–3 $D008–$D00B: 0 normal, 1 doppelt, 3 vierfach) und Priorität (PRIOR $D01B).
        // Alle fünf Konstanten stehen immer im Export, auch bei Vorgabewerten und alten Dateien ohne `hw` (Breite 0, Priorität 1 = Spieler vor Spielfeld): das Programm kann sich darauf verlassen
        let mut out = String::new();
        let hw = &root["hw"];
        for i in 0..4 { let n = hw["sizep"][i].as_u64().filter(|v| [0, 1, 3].contains(v)).unwrap_or(0); out.push_str(&format!("{c} {label}_sizep{i} = {n}\n")); }
        let n = hw["prior"].as_u64().filter(|v| [1, 2, 4, 8].contains(v)).unwrap_or(1);
        out.push_str(&format!("{c} {label}_prior = {n}\n"));
        return Ok(out);
    }
    if p.kind != "sprite" { return Ok(String::new()); }
    let frames = root["frames"].as_array().ok_or(vc_i18n::t!("media-frames-fehlt"))?;
    let mc: Vec<bool> = frames.iter().map(|f| frame_is_mc(p, f)).collect();
    let mask = mc.iter().take(8).enumerate().fold(0u32, |m, (i, &b)| m | ((b as u32) << i));
    let note = if sbas { vc_i18n::t!("media-bit-i-frame-i-ist-multicolor") } else { vc_i18n::t!("media-bit-i-frame-i-ist-multicolor-2") };
    let mut out = format!("{c} {label}_count = {}\n{c} {label}_multicolor_mask = ${mask:02x}   {note}\n", frames.len());
    // Optionale Hardware-Flags aus den Metadaten `hw` ($D01D Breit, $D017 Hoch, $D01B hinter dem Hintergrund); ohne `hw` bleibt die Ausgabe wie bisher
    if let Some(hw) = root["hw"].as_object() {
        for (key, name) in [("expandX", "expand_x"), ("expandY", "expand_y"), ("behind", "behind")] { if let Some(b) = hw.get(key).and_then(|v| v.as_bool()) { out.push_str(&format!("{c} {label}_{name} = {}\n", b as u8)); } }
    }
    Ok(out)
}

/// Assembler-Tabellen (Daten) für Modi und Animationen; `None`, wenn es weder gemischte Frames noch Animationen gibt.
/// Sie kommen in eine eigene Datei, damit die Sprite-Daten selbst unverändert bleiben (die Tabellen belegen Speicher hinter ihnen).
/// `label` ist die Marke des ersten Sprite-Blocks (64 Byte je Frame).
pub fn export_anim_asm(label: &str, p: &Profile, root: &Value) -> Result<Option<String>, String> {
    if p.kind != "sprite" { return Ok(None); }
    let frames = root["frames"].as_array().ok_or(vc_i18n::t!("media-frames-fehlt"))?;
    let anims = parse_animations(root)?;
    let mc: Vec<bool> = frames.iter().map(|f| frame_is_mc(p, f)).collect();
    let mixed = mc.iter().any(|&m| m != (p.bpp == 2));
    if anims.is_empty() && !mixed { return Ok(None); }
    let mut s = vc_i18n::t!("media-kopf-sprite-tabellen") + "\n";
    if mixed { s.push_str(&format!("{label}_modes:\n        .byte {}\n", mc.iter().map(|&b| (b as u8).to_string()).collect::<Vec<_>>().join(","))); }
    for a in &anims {
        let ex = a.expanded();
        if ex.is_empty() { return Err(vc_i18n::t!("media-animation-p1-hat-keine-schritte", p1 = &a.name)); }
        s.push_str(&format!("; Animation \"{}\" ({}): Sprite-Zeiger = Blocknummer im VIC-Bank, Dauer in Bildern (1/50 s)\n", a.name, a.mode.label()));
        s.push_str(&format!("{label}_anim_{}_ptr:\n        .byte {}\n", a.name, ex.iter().map(|(f, _)| format!("(({label} & $3fff) / 64) + {f}")).collect::<Vec<_>>().join(",")));
        s.push_str(&format!("{label}_anim_{}_delay:\n        .byte {}\n", a.name, ex.iter().map(|(_, d)| d.to_string()).collect::<Vec<_>>().join(",")));
        s.push_str(&format!(".const {label}_anim_{}_len = {}\n.const {label}_anim_{}_mode = {}   ; 0 = Schleife, 1 = einmal\n", a.name, ex.len(), a.name, (a.mode == AnimMode::Once) as u8));
    }
    Ok(Some(s))
}
