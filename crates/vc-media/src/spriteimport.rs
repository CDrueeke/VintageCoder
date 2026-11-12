// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprites aus anderen Quellen übernehmen: SPD-Dateien (`.spd`), Rohdaten in 64-Byte-Blöcken und Sprite-Dateien anderer Systeme
//! (C64-Sprites ↔ Atari-Player/Missile-Grafik).
use crate::sprite::{frame_is_mc, hires_to_mc, mc_to_hires};
use crate::{new_media, Profile};
use serde_json::{json, Value};

/// Ein C64-Sprite: Modus und Pixel (Hires 24×21 mit Werten 0/1, Multicolor 12×21 mit Werten 0..3)
#[derive(Clone, Debug, PartialEq)]
pub struct C64Spr { pub mc: bool, pub rows: Vec<Vec<u8>> }

#[derive(Clone, Debug, Default)]
pub struct SprOpts {
    /// Rohdaten: Byte-Offset des ersten Sprites
    pub offset: usize,
    /// Rohdaten: Zahl der Sprites (0 = alle vollständigen Blöcke)
    pub count: usize,
    /// Rohdaten: Modus (None = Modus des Zielprofils)
    pub mc: Option<bool>,
    /// beim Wechsel zu Atari-PMG: welches Sprite der Quelle (0-basiert)
    pub frame: usize,
}

#[derive(Clone, Debug)]
pub struct SpdInfo { pub version: u8, pub sprites: usize, pub colors: [u8; 3], pub data_start: usize }

/// Kopf einer SPD-Datei (Versionen 1, 3 und 5)
pub fn spd_info(b: &[u8]) -> Result<SpdInfo, String> {
    if b.len() < 9 || &b[0..3] != b"SPD" { return Err("keine SPD-Datei (SPD)".into()); }
    let version = b[3];
    let u16at = |o: usize| b.get(o..o + 2).map(|x| x[0] as usize | (x[1] as usize) << 8);
    let (sprites, colors, start) = match version {
        1 => (b[4] as usize + 1, [b[6], b[7], b[8]], 9),
        3 | 5 => {
            if b.len() < 16 { return Err("SPD-Kopf unvollständig".into()); }
            (u16at(5).unwrap_or(0), [b[13], b[14], b[15]], if version == 5 { 20 } else { 16 })
        }
        v => return Err(format!("SPD-Version {v} wird nicht unterstützt (1, 3 und 5)")),
    };
    if sprites == 0 { return Err("die SPD-Datei enthält keine Sprites".into()); }
    if b.len() < start + 64 * sprites { return Err(format!("SPD-Datei zu kurz: {sprites} Sprites brauchen {} Bytes ab Offset {start}, Datei hat {}", 64 * sprites, b.len())); }
    Ok(SpdInfo { version, sprites, colors, data_start: start })
}

/// 63 Datenbytes eines Sprites in Pixel umrechnen
pub fn decode_block(b: &[u8], mc: bool) -> Vec<Vec<u8>> {
    (0..21).map(|y| {
        let row = &b[y * 3..y * 3 + 3];
        if mc { (0..12).map(|x| (row[x / 4] >> (6 - 2 * (x % 4))) & 3).collect() } else { (0..24).map(|x| (row[x / 8] >> (7 - x % 8)) & 1).collect() }
    }).collect()
}

/// SPD-Datei: Sprites mit Modus (Bit 7 der Attributbyte); liefert die Sprites, die Farben (Sprite, Multicolor 1, 2) und Hinweise
pub fn read_spd(b: &[u8]) -> Result<(Vec<C64Spr>, [u8; 3], Vec<String>), String> {
    let i = spd_info(b)?; let mut warn = vec![];
    let mut out = vec![]; let mut first_color = None; let mut colors_differ = false; let mut overlays = 0;
    for n in 0..i.sprites {
        let blk = &b[i.data_start + n * 64..i.data_start + (n + 1) * 64];
        let attr = blk[63];
        let c = attr & 0x0F; match first_color { None => first_color = Some(c), Some(f) if f != c => colors_differ = true, _ => {} }
        if attr & 0x10 != 0 { overlays += 1; }
        out.push(C64Spr { mc: attr & 0x80 != 0, rows: decode_block(&blk[..63], attr & 0x80 != 0) });
    }
    if colors_differ { warn.push("Die Sprites haben im SPD-Format verschiedene Farben; übernommen wird die Farbe des ersten (ein Sprite-Asset hat eine Sprite-Farbe)".into()); }
    if overlays > 0 { warn.push(format!("{overlays} Sprite(s) sind im SPD-Format als Overlay markiert; die Overlay-Zuordnung wird nicht übernommen")); }
    Ok((out, [first_color.unwrap_or(1), i.colors[1], i.colors[2]], warn))
}

/// Rohdaten: Blöcke zu 64 Bytes (63 Daten + 1 Füll-/Attributbyte)
pub fn read_raw(b: &[u8], o: &SprOpts, default_mc: bool) -> Result<Vec<C64Spr>, String> {
    let tail = b.get(o.offset..).ok_or_else(|| format!("Offset {} liegt hinter dem Dateiende ({} Bytes)", o.offset, b.len()))?;
    let avail = tail.len() / 64;
    if avail == 0 { return Err(format!("ab Offset {} sind weniger als 64 Bytes übrig", o.offset)); }
    let n = if o.count == 0 { avail } else { o.count.min(avail) };
    if o.count > avail { return Err(format!("{} Sprites verlangt, ab Offset {} gibt es nur {avail} vollständige Blöcke", o.count, o.offset)); }
    let mc = o.mc.unwrap_or(default_mc);
    Ok((0..n.min(256)).map(|i| C64Spr { mc, rows: decode_block(&tail[i * 64..i * 64 + 63], mc) }).collect())
}

/// Sprites eines C64-Sprite-Assets
pub fn read_c64_media(src: &Value, from: &Profile) -> Result<Vec<C64Spr>, String> {
    src["frames"].as_array().ok_or("frames fehlen")?.iter().enumerate().map(|(i, f)| {
        let mc = frame_is_mc(from, f);
        let rows = f["rows"].as_array().ok_or(format!("Frame {i}: rows fehlt"))?.iter().map(|r| r.as_str().unwrap_or("").chars().map(|c| c.to_digit(16).unwrap_or(0) as u8).collect::<Vec<u8>>()).collect::<Vec<_>>();
        let w = if mc { 12 } else { 24 };
        if rows.len() != 21 || rows.iter().any(|r| r.len() != w) { return Err(format!("Frame {i}: {}×{} statt {w}×21 Pixel", rows.first().map_or(0, |r| r.len()), rows.len())); }
        Ok(C64Spr { mc, rows })
    }).collect()
}

/// Atari-PMG-Asset → C64-Sprite: die Player 0–2 nebeneinander (24 Pixel breit, Hires), auf 21 Zeilen gekürzt
pub fn pmg_to_c64(src: &Value, warn: &mut Vec<String>) -> Result<C64Spr, String> {
    let frames = src["frames"].as_array().ok_or("frames fehlen")?;
    if frames.len() < 3 { return Err("PMG-Datei hat weniger als 3 Player".into()); }
    let rows_of = |f: &Value| f["rows"].as_array().map(|r| r.iter().map(|x| x.as_str().unwrap_or("").chars().map(|c| (c == '1') as u8).collect::<Vec<u8>>()).collect::<Vec<_>>()).unwrap_or_default();
    let players: Vec<Vec<Vec<u8>>> = frames.iter().map(rows_of).collect();
    let h = players[0].len();
    let mut rows = vec![vec![0u8; 24]; 21];
    for y in 0..21 { for p in 0..3 { for x in 0..8 { rows[y][p * 8 + x] = players[p].get(y).and_then(|r| r.get(x)).copied().unwrap_or(0); } } }
    if h > 21 && players.iter().take(3).any(|p| p.iter().skip(21).any(|r| r.iter().any(|&v| v != 0))) { warn.push(format!("Die PMG-Grafik hat {h} Zeilen; die Zeilen ab 22 fehlen im C64-Sprite")); }
    if players.get(3).map_or(false, |p| p.iter().any(|r| r.iter().any(|&v| v != 0))) { warn.push("Player 3 hat Pixel; ein C64-Sprite ist nur 24 Pixel breit, Player 3 wird nicht übernommen".into()); }
    warn.push("Die Farben der Player werden nicht übernommen (C64-Farben stehen im Sprite-Asset)".into());
    Ok(C64Spr { mc: false, rows })
}

/// C64-Sprite → Atari-PMG-Asset: Hires-Pixel auf drei Player (je 8 Pixel), Player 3 bleibt leer. Multicolor wird zu Hires verdoppelt.
pub fn c64_to_pmg(s: &C64Spr, to: &Profile, name: &str, warn: &mut Vec<String>) -> Value {
    let hires = if s.mc { warn.push("Multicolor-Sprite: die Pixel werden doppelt breit als Hires übernommen, die Mehrfarbigkeit geht verloren (der Atari hat je Player eine Farbe)".into()); mc_to_hires(&s.rows) } else { s.rows.clone() };
    let mut m = new_media(to, name);
    let h = m["height"].as_u64().unwrap_or(to.height as u64) as usize;
    if h < 21 { warn.push(format!("Das PMG-Profil hat nur {h} Zeilen; das Sprite wird unten abgeschnitten")); }
    let frames: Vec<Value> = (0..4).map(|p| json!({ "rows": (0..h).map(|y| (0..8).map(|x| if p < 3 { hires.get(y).and_then(|r| r.get(p * 8 + x)).map_or('0', |&v| if v != 0 { '1' } else { '0' }) } else { '0' }).collect::<String>()).collect::<Vec<_>>() })).collect();
    m["frames"] = Value::Array(frames);
    warn.push("Die Farben werden nicht übernommen (Standardfarben der Player); die drei Player müssen im Programm nebeneinander positioniert werden".into());
    m
}

/// Sprites in ein C64-Sprite-Asset des Profils `to` schreiben; Frames, deren Modus vom Profil abweicht, bekommen `mode`
pub fn c64_media(sprites: &[C64Spr], colors: [u8; 3], to: &Profile, name: &str) -> Value {
    let mut m = new_media(to, name);
    let profile_mc = to.bpp == 2;
    m["frames"] = Value::Array(sprites.iter().map(|s| {
        let rows: Vec<String> = s.rows.iter().map(|r| r.iter().map(|v| std::char::from_digit(*v as u32, 16).unwrap_or('0')).collect()).collect();
        if s.mc != profile_mc { json!({ "mode": if s.mc { "multicolor" } else { "hires" }, "rows": rows }) } else { json!({ "rows": rows }) }
    }).collect());
    m["colors"] = json!({ "sprite": colors[0], "mc1": colors[1], "mc2": colors[2] });
    m
}

/// Hires ↔ Multicolor einzeln umrechnen (für Importe, die den Modus des Zielprofils erzwingen sollen)
pub fn force_mode(s: &C64Spr, mc: bool) -> C64Spr {
    if s.mc == mc { return s.clone(); }
    C64Spr { mc, rows: if mc { hires_to_mc(&s.rows) } else { mc_to_hires(&s.rows) } }
}
