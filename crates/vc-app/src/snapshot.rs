// SPDX-License-Identifier: GPL-3.0-or-later
//! Momentaufnahme aus dem angehaltenen Rechner (C64 und Atari): Bildschirm, Zeichensatz und Sprites erkennen und als Media speichern.
//! Die Hardware verrät, wo alles liegt (VIC-Bank, `$D018`, Sprite-Zeiger; ANTIC-Display-Liste, CHBAS). Bildschirm und Zeichensatz dürfen deshalb
//! beliebig im Speicher liegen oder dorthin kopiert worden sein; die Adressen lassen sich vor dem Speichern von Hand überschreiben.
use serde_json::{json, Value};
use vc_media::Profile;

/// Speichersichten des Rechners
pub trait Mem {
    /// Sicht der CPU
    fn cpu(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String>;
    /// RAM unabhängig von ROM- und E/A-Umschaltung (so sieht der VIC den Speicher)
    fn ram(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String>;
    /// E/A-Bereich (VIC, CIA, Farb-RAM) unabhängig von der Umschaltung
    fn io(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String>;
    /// Zeichen-ROM des C64 (nur für die Vorschau; wird nie gespeichert)
    fn chargen(&mut self, _offset: u16, _len: usize) -> Result<Vec<u8>, String> { Err("nicht lesbar".into()) }
}

/// Über `Backend` des Emulators
pub struct BackendMem<'a>(pub &'a mut dyn vc_emu::Backend);
impl Mem for BackendMem<'_> {
    fn cpu(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { self.0.read_mem(a, n) }
    fn ram(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { self.0.read_space("ram", a, n) }
    fn io(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { self.0.read_space("io", a, n) }
    fn chargen(&mut self, o: u16, n: usize) -> Result<Vec<u8>, String> { self.0.read_space("rom", 0xD000 + o, n) }
}

#[derive(Clone, Debug, Default)]
pub struct Sprite { pub index: usize, pub addr: u16, pub multicolor: bool, pub color: u8 }

/// Vorschau des erkannten Bildschirms (RGBA)
#[derive(Clone, Debug, Default)]
pub struct Preview { pub w: usize, pub h: usize, pub rgba: std::sync::Arc<Vec<u8>> }

#[derive(Clone, Debug, Default)]
pub struct Detected {
    pub preview: Option<Preview>,
    /// Lage des angezeigten Zeichensatzes (auch im ROM) bzw. Offset im Zeichen-ROM des C64
    pub view_cs: u16, pub rom_off: Option<u16>,
    pub system: String,
    pub supported: bool,
    pub mode: String,
    pub screen: u16,
    /// `None`: Zeichensatz im ROM (nicht lesbar) – per ROM-Import holen oder die Adresse von Hand eintragen
    pub charset: Option<u16>,
    pub charset_multicolor: bool,
    pub sprites: Vec<Sprite>,
    pub border: u8, pub bg: [u8; 4], pub fg: u8,
    pub notes: Vec<String>,
}

fn byte(m: &mut dyn Mem, f: fn(&mut dyn Mem, u16, usize) -> Result<Vec<u8>, String>, a: u16) -> Result<u8, String> { f(m, a, 1).map(|v| v[0]) }
fn cpu(m: &mut dyn Mem, a: u16, n: usize) -> Result<Vec<u8>, String> { m.cpu(a, n) }
fn io(m: &mut dyn Mem, a: u16, n: usize) -> Result<Vec<u8>, String> { m.io(a, n) }
fn ram(m: &mut dyn Mem, a: u16, n: usize) -> Result<Vec<u8>, String> { m.ram(a, n) }

pub fn detect(system: &str, m: &mut dyn Mem) -> Result<Detected, String> {
    match system { "c64" => detect_c64(m), "atari8" => detect_atari(m), s => Err(format!("Momentaufnahme: System '{s}' wird nicht unterstützt (C64 und Atari 8-Bit)")) }
}

fn detect_c64(m: &mut dyn Mem) -> Result<Detected, String> {
    let regs = m.io(0xD000, 0x30)?; let cia2 = byte(m, io, 0xDD00)?;
    let (d011, d016, d018) = (regs[0x11], regs[0x16], regs[0x18]);
    let bank = (!cia2 & 3) as u32;
    let screen = (bank * 0x4000 + ((d018 as u32 >> 4) & 15) * 0x400) as u16;
    let off = ((d018 as u32 >> 1) & 7) * 0x800; let cs = (bank * 0x4000 + off) as u16;
    let rom_cs = (bank == 0 || bank == 2) && (off == 0x1000 || off == 0x1800);
    let mut d = Detected { rom_off: if rom_cs { Some(off as u16 - 0x1000) } else { None }, view_cs: cs, system: "c64".into(), supported: true, screen, charset: if rom_cs { None } else { Some(cs) }, charset_multicolor: d016 & 0x10 != 0,
        border: regs[0x20] & 15, bg: [regs[0x21] & 15, regs[0x22] & 15, regs[0x23] & 15, regs[0x24] & 15], fg: 1, ..Default::default() };
    let (bmm, ecm) = (d011 & 0x20 != 0, d011 & 0x40 != 0);
    d.mode = format!("{}{} · VIC-Bank {bank} (${:04X}–${:04X})", if bmm { "Bitmap" } else { "Text" }, if ecm { " (ECM)" } else if d.charset_multicolor { " (Multicolor)" } else { "" }, bank * 0x4000, bank * 0x4000 + 0x3FFF);
    if bmm { d.supported = false; d.notes.push("Bitmap-Modus: kein Textbildschirm – Screen und Zeichensatz entfallen (Sprites sind trotzdem möglich).".into()); }
    if ecm { d.notes.push("ECM-Modus: es werden nur 64 Zeichen dargestellt; der Bildschirm wird unverändert gelesen.".into()); }
    if rom_cs { d.notes.push("Der Zeichensatz liegt im Zeichen-ROM (Standardschrift) und wird von dort gelesen.".into()); }
    let enabled = regs[0x15]; let mcmask = regs[0x1C];
    let ptrs = m.ram(screen.wrapping_add(0x3F8), 8)?;
    for i in 0..8 {
        if enabled & (1 << i) == 0 { continue; }
        d.sprites.push(Sprite { index: i, addr: (bank * 0x4000 + ptrs[i] as u32 * 64) as u16, multicolor: mcmask & (1 << i) != 0, color: regs[0x27 + i] & 15 });
    }
    Ok(d)
}

fn detect_atari(m: &mut dyn Mem) -> Result<Detected, String> {
    let sh = m.cpu(0x0230, 2)?; let dl = u16::from_le_bytes([sh[0], sh[1]]);
    let chbas = byte(m, cpu, 0x02F4)?; let col = m.cpu(0x02C4, 5)?;
    let mut d = Detected { view_cs: (chbas as u16) << 8, system: "atari8".into(), supported: true, border: col[4], bg: [col[2], 0, 0, 0], fg: (col[2] & 0xF0) | (col[1] & 0x0F), ..Default::default() };
    d.notes.push("Atari: Die Farben und Zeiger stammen aus den Schattenregistern des Betriebssystems. Programme mit eigener Display-Liste oder eigenem Interrupt müssen Adressen und Farben ggf. von Hand anpassen.".into());
    // Display-Liste: erste Zeile mit LMS suchen
    let list = m.cpu(dl, 128.min(0x10000 - dl as usize))?; let mut i = 0; let mut mode = None; let mut screen = None;
    while i < list.len() {
        let b = list[i]; let op = b & 0x0F;
        if op == 0 { i += 1; continue; }
        if op == 1 { if b & 0x40 != 0 { break; } i += 3; continue; }
        if b & 0x40 != 0 && i + 2 < list.len() { if screen.is_none() { screen = Some(u16::from_le_bytes([list[i + 1], list[i + 2]])); mode = Some(op); } i += 3; } else { if mode.is_none() { mode = Some(op); } i += 1; }
    }
    let Some(screen) = screen else { d.supported = false; d.mode = "keine Display-Liste mit LMS gefunden".into(); d.notes.push(format!("Display-Liste bei ${dl:04X} enthält keinen LMS-Befehl.")); return Ok(d); };
    d.screen = screen; d.mode = format!("ANTIC-Modus {}, Display-Liste ${dl:04X}", mode.unwrap_or(0));
    if mode != Some(2) { d.supported = false; d.notes.push(format!("ANTIC-Modus {} wird nicht unterstützt (nur Modus 2, 40×24 Textzeichen).", mode.unwrap_or(0))); }
    let cs = (chbas as u16) << 8;
    d.charset = Some(cs);
    if chbas == 0xE0 { d.notes.push("Der Zeichensatz liegt im Betriebssystem-ROM ($E000) und wird von dort gelesen.".into()); }
    Ok(d)
}


/// Erkennen und Vorschau berechnen
pub fn detect_with_preview(system: &str, m: &mut dyn Mem, pal: &crate::bitmap::Palettes) -> Result<Detected, String> {
    let mut d = detect(system, m)?;
    if d.supported { d.preview = render_preview(&d, m, pal); }
    Ok(d)
}

fn render_preview(d: &Detected, m: &mut dyn Mem, pal: &crate::bitmap::Palettes) -> Option<Preview> {
    let (cols, rows) = if d.system == "c64" { (40usize, 25usize) } else { (40, 24) };
    let (w, h) = (cols * 8, rows * 8);
    let mut rgba = vec![0u8; w * h * 4];
    let mut put = |x: usize, y: usize, c: eframe::egui::Color32| { let i = (y * w + x) * 4; rgba[i..i + 4].copy_from_slice(&c.to_array()); };
    if d.system == "c64" {
        let cs = match (d.charset, d.rom_off) { (Some(a), _) => ram(m, a, 2048).ok()?, (None, Some(o)) => m.chargen(o, 2048).ok()?, _ => return None };
        let chars = ram(m, d.screen, 1000).ok()?; let color = io(m, 0xD800, 1000).ok()?;
        let col = |n: u8| pal.color("c64", n as usize);
        for i in 0..1000 {
            let (cx, cy) = (i % 40, i / 40); let g = &cs[chars[i] as usize * 8..chars[i] as usize * 8 + 8]; let c = color[i] & 15;
            for (y, &b) in g.iter().enumerate() {
                if d.charset_multicolor && c & 8 != 0 {
                    for px in 0..4 { let v = (b >> (6 - px * 2)) & 3; let cc = col(match v { 0 => d.bg[0], 1 => d.bg[1], 2 => d.bg[2], _ => c & 7 }); put(cx * 8 + px * 2, cy * 8 + y, cc); put(cx * 8 + px * 2 + 1, cy * 8 + y, cc); }
                } else {
                    let fg = col(if d.charset_multicolor { c & 7 } else { c });
                    for x in 0..8 { put(cx * 8 + x, cy * 8 + y, if b & (0x80 >> x) != 0 { fg } else { col(d.bg[0]) }); }
                }
            }
        }
    } else {
        let cs = m.cpu(d.view_cs, 1024.min(0x10000 - d.view_cs as usize)).ok()?; let chars = m.cpu(d.screen, 960).ok()?;
        let (fg, bg) = (pal.color("atari8", d.fg as usize), pal.color("atari8", d.bg[0] as usize));
        for i in 0..960 {
            let (cx, cy) = (i % 40, i / 40); let code = chars[i]; let g = (code & 0x7F) as usize * 8; let inv = code & 0x80 != 0;
            if g + 8 > cs.len() { continue; }
            for y in 0..8 { let b = cs[g + y]; for x in 0..8 { let on = (b & (0x80 >> x) != 0) != inv; put(cx * 8 + x, cy * 8 + y, if on { fg } else { bg }); } }
        }
    }
    Some(Preview { w, h, rgba: std::sync::Arc::new(rgba) })
}

/// Auswahl beim Speichern; Adressen dürfen von der Erkennung abweichen (Kopien an anderer Stelle)
#[derive(Clone, Debug)]
pub struct Opts { pub screen: Option<u16>, pub charset: Option<u16>, /** Zeichensatz aus dem ROM lesen (wenn `charset` fehlt) */ pub rom: bool, pub sprites: bool, pub tiles: bool, pub name: String }


/// Aus dem Textbildschirm 2×2-Kacheln bilden. Jede Kachel hat genau eine Farbe (die der ersten sichtbaren Zelle);
/// Zeichen und Farbe zusammen unterscheiden die Kacheln. Gibt (Kacheln, Karte, Hinweis) zurück.
fn build_tiles(chars: &[u8], colors: Option<&[u8]>, rows: usize, empty: u8, mapw: usize, maph: usize) -> Result<(Vec<Value>, Vec<Vec<usize>>, String), String> {
    let mut best: Option<(usize, usize, usize)> = None;   // (Kachelanzahl, ox, oy)
    for oy in 0..2 { for ox in 0..2 {
        let mut set = std::collections::HashSet::new();
        for ty in 0..(rows - oy) / 2 { for tx in 0..(40 - ox) / 2 { set.insert(tile_key(chars, colors, tx * 2 + ox, ty * 2 + oy, empty).0); } }
        if best.map_or(true, |b| set.len() < b.0) { best = Some((set.len(), ox, oy)); }
    } }
    let (_, ox, oy) = best.unwrap();
    let mut tiles: Vec<Value> = vec![json!({ "name": "leer", "chars": vec![empty; 4], "color": 1, "solid": false })];
    let mut index: std::collections::HashMap<([u8; 4], u8), usize> = std::collections::HashMap::new(); index.insert(([empty; 4], 1), 0);
    let mut map = vec![vec![0usize; mapw]; maph]; let mut mixed = 0;
    for ty in 0..((rows - oy) / 2).min(maph) { for tx in 0..((40 - ox) / 2).min(mapw) {
        let (key, m) = tile_key(chars, colors, tx * 2 + ox, ty * 2 + oy, empty); if m { mixed += 1; }
        let i = *index.entry(key).or_insert_with(|| { tiles.push(json!({ "name": format!("t{}", tiles.len()), "chars": key.0, "color": key.1, "solid": false })); tiles.len() - 1 });
        map[ty][tx] = i;
    } }
    if tiles.len() > 256 { return Err(format!("Zu viele verschiedene Kacheln ({}); der Bildschirm lässt sich nicht als Kachelkarte darstellen", tiles.len())); }
    let mut note = format!("{} Kacheln, Raster ab Spalte {ox}, Zeile {oy}.", tiles.len());
    if mixed > 0 { note += &format!(" {mixed} Kachelstellen hatten gemischte Farben (es gilt die Farbe der ersten sichtbaren Zelle)."); }
    Ok((tiles, map, note))
}

fn tile_key(chars: &[u8], colors: Option<&[u8]>, x: usize, y: usize, empty: u8) -> (([u8; 4], u8), bool) {
    let cells = [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)];
    let cs: [u8; 4] = std::array::from_fn(|i| chars[cells[i].1 * 40 + cells[i].0]);
    let Some(colors) = colors else { return ((cs, 1), false) };
    let vis: Vec<u8> = (0..4).filter(|&i| cs[i] != empty).map(|i| colors[cells[i].1 * 40 + cells[i].0] & 15).collect();
    let c = vis.first().copied().unwrap_or(1);
    ((cs, if vis.is_empty() { 1 } else { c }), vis.iter().any(|v| *v != c))
}

/// Liest die Daten und baut die Media: (relativer Pfad, Inhalt)
pub fn build_media(d: &Detected, o: &Opts, m: &mut dyn Mem, profiles: &[Profile]) -> Result<Vec<(String, Value)>, String> {
    let find = |id: &str| profiles.iter().find(|p| p.id == id).ok_or_else(|| format!("Profil '{id}' fehlt"));
    let mut out = vec![]; let mut cs_rel: Option<String> = None;
    let cs_name = format!("{}_charset", o.name); let sc_name = format!("{}_screen", o.name);
    match d.system.as_str() {
        "c64" => {
            let cs_bytes = match (o.charset, d.rom_off) { (Some(a), _) => Some(ram(m, a, 2048)?), (None, Some(r)) if o.rom => Some(m.chargen(r, 2048)?), _ => None };
            if let Some(bytes) = cs_bytes {
                let p = find(if d.charset_multicolor { "c64.char.multicolor" } else { "c64.char.hires" })?;
                let mut v = vc_media::decode(p, &bytes, &cs_name)?;
                v["colors"]["bg"] = json!(d.bg[0]); v["colors"]["fg"] = json!(d.fg); if d.charset_multicolor { v["colors"]["mc1"] = json!(d.bg[1]); v["colors"]["mc2"] = json!(d.bg[2]); }
                let rel = format!("graphics/snapshot/{cs_name}.vcchar"); cs_rel = Some(rel.clone()); out.push((rel, v));
            }
            if let Some(a) = o.screen {
                if !d.supported { return Err("Der aktuelle Modus ist kein Textbildschirm.".into()); }
                let p = find("c64.screen.text")?; let chars = ram(m, a, 1000)?; let color = io(m, 0xD800, 1000)?;
                let mut v = vc_media::new_media(p, &sc_name);
                v["chars"] = json!((0..25).map(|r| chars[r * 40..r * 40 + 40].to_vec()).collect::<Vec<_>>());
                v["attrs"] = json!((0..25).map(|r| color[r * 40..r * 40 + 40].iter().map(|c| c & 15).collect::<Vec<_>>()).collect::<Vec<_>>());
                v["colors"] = json!({ "border": d.border, "bg": d.bg[0], "fg": d.fg });
                if d.charset_multicolor { v["colors"]["mc1"] = json!(d.bg[1]); v["colors"]["mc2"] = json!(d.bg[2]); }
                if let Some(r) = &cs_rel { v["charset"] = json!(r); }
                out.push((format!("graphics/snapshot/{sc_name}.vcscreen"), v));
                if o.tiles {
                    let (tiles, map, _) = build_tiles(&chars, Some(&color), 25, 32, 20, 12)?;
                    let mut t = vc_media::new_media(find("c64.tilemap.2x2")?, &format!("{}_tiles", o.name));
                    t["tiles"] = json!(tiles); t["map"] = json!(map); t["colors"] = json!({ "border": d.border, "bg": d.bg[0] });
                    if d.charset_multicolor { t["colors"]["mc1"] = json!(d.bg[1]); t["colors"]["mc2"] = json!(d.bg[2]); }
                    if let Some(r) = &cs_rel { t["charset"] = json!(r); }
                    out.push((format!("graphics/snapshot/{}_tiles.vctiles", o.name), t));
                }
            }
            if o.sprites {
                let regs = io(m, 0xD025, 2)?;     // Sprite-Multicolor 1 und 2
                for s in &d.sprites {
                    let p = find(if s.multicolor { "c64.sprite.multicolor" } else { "c64.sprite.hires" })?;
                    let name = format!("{}_sprite{}", o.name, s.index);
                    let mut v = vc_media::decode(p, &ram(m, s.addr, 64)?, &name)?;
                    v["colors"]["sprite"] = json!(s.color); if s.multicolor { v["colors"]["mc1"] = json!(regs[0] & 15); v["colors"]["mc2"] = json!(regs[1] & 15); }
                    out.push((format!("graphics/snapshot/{name}.vcsprite"), v));
                }
            }
        }
        "atari8" => {
            if let Some(a) = o.charset {
                let p = find("atari.char.antic2")?; let v0 = vc_media::decode(p, &m.cpu(a, 1024.min(0x10000 - a as usize))?, &cs_name)?; let mut v = v0;
                v["colors"]["bg"] = json!(d.bg[0]); v["colors"]["fg"] = json!(d.fg);
                let rel = format!("graphics/snapshot/{cs_name}.vcchar"); cs_rel = Some(rel.clone()); out.push((rel, v));
            }
            if let Some(a) = o.screen {
                if !d.supported { return Err("Der aktuelle ANTIC-Modus wird nicht unterstützt.".into()); }
                let p = find("atari.screen.antic2")?; let chars = m.cpu(a, 960)?; let mut v = vc_media::new_media(p, &sc_name);
                v["chars"] = json!((0..24).map(|r| chars[r * 40..r * 40 + 40].to_vec()).collect::<Vec<_>>());
                v["colors"] = json!({ "bg": d.bg[0], "fg": d.fg, "border": d.border });
                if let Some(r) = &cs_rel { v["charset"] = json!(r); }
                out.push((format!("graphics/snapshot/{sc_name}.vcscreen"), v));
                if o.tiles {
                    let (tiles, map, _) = build_tiles(&chars, None, 24, 0, 20, 12)?;
                    let mut t = vc_media::new_media(find("atari.tilemap.2x2")?, &format!("{}_tiles", o.name));
                    t["tiles"] = json!(tiles); t["map"] = json!(map); t["colors"] = json!({ "bg": d.bg[0], "fg": d.fg, "border": d.border });
                    if let Some(r) = &cs_rel { t["charset"] = json!(r); }
                    out.push((format!("graphics/snapshot/{}_tiles.vctiles", o.name), t));
                }
            }
        }
        s => return Err(format!("System '{s}' wird nicht unterstützt")),
    }
    if out.is_empty() { return Err("Nichts zu speichern: Adresse für Screen oder Zeichensatz fehlt (Zeichensatz im ROM?).".into()); }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Synthetischer C64: 64-KB-RAM, E/A-Bereich ($D000–$DFFF) getrennt, CPU sieht E/A oder RAM wie eingestellt
    struct Fake { ram: Vec<u8>, io: Vec<u8>, cpu_io: bool }
    impl Mem for Fake {
        fn cpu(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { Ok((0..n).map(|i| { let x = a as usize + i; if self.cpu_io && (0xD000..0xE000).contains(&x) { self.io[x - 0xD000] } else { self.ram[x] } }).collect()) }
        fn ram(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { Ok(self.ram[a as usize..a as usize + n].to_vec()) }
        fn io(&mut self, a: u16, n: usize) -> Result<Vec<u8>, String> { Ok(self.io[a as usize - 0xD000..a as usize - 0xD000 + n].to_vec()) }
        fn chargen(&mut self, o: u16, n: usize) -> Result<Vec<u8>, String> { Ok(vec![0xAA; n + o as usize * 0]) }
    }
    fn profiles() -> Vec<Profile> { vc_media::load_profiles(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")).unwrap() }

    #[test]
    fn c64_screen_charset_and_sprites_copied_elsewhere() {
        let mut f = Fake { ram: vec![0; 65536], io: vec![0; 4096], cpu_io: false };
        // VIC-Bank 2 ($8000–$BFFF): CIA2 $DD00 = %01 -> Bank 2; Bildschirm bei $8400 (d018 hoch 1), Zeichensatz bei $A000 ... liegt innerhalb der Bank (Offset $2000 -> d018 niedrig 4)
        f.io[0xD00] = 0x01; f.io[0x18] = 0x18; f.io[0x11] = 0x1B; f.io[0x16] = 0xC8; f.io[0x20] = 6; f.io[0x21] = 14; f.io[0x15] = 0b0000_0101; f.io[0x1C] = 0b0000_0100; f.io[0x27] = 7; f.io[0x29] = 2; f.io[0x25] = 3; f.io[0x26] = 4;
        let screen = 0x8400usize; let cs = 0xA000usize;
        f.ram[cs + 8..cs + 16].copy_from_slice(&[0x18; 8]);            // Zeichen 1
        for i in 0..1000 { f.ram[screen + i] = (i % 256) as u8; }
        for i in 0..1000 { f.io[0x800 + i] = (i % 16) as u8 | 0xF0; }   // Farb-RAM ($D800), obere Bits ignorieren
        f.ram[screen + 0x3F8] = 0x80; f.ram[screen + 0x3FA] = 0x81;       // Sprite 0 bei $A000+? : Zeiger*64 = $2000/$2040 in Bank 2 -> $A000/$A040
        f.ram[0xA000 + 4] = 0xFF; f.ram[0xA040 + 3] = 0b1100_0011;
        // Hinweis: Sprite 0 und Zeichensatz überlappen bei $A000 absichtlich (nur Lesetest)
        let d = detect("c64", &mut f).unwrap();
        assert_eq!((d.screen, d.charset), (0x8400, Some(0xA000)), "{d:?}");
        assert!(d.supported); assert_eq!((d.border, d.bg[0]), (6, 14));
        assert_eq!(d.sprites.len(), 2); assert_eq!((d.sprites[1].index, d.sprites[1].addr, d.sprites[1].multicolor), (2, 0xA040, true));
        let o = Opts { screen: Some(d.screen), charset: d.charset, rom: d.rom_off.is_some(), sprites: true, tiles: false, name: "snap".into() };
        let a = build_media(&d, &o, &mut f, &profiles()).unwrap();
        let names: Vec<&str> = a.iter().map(|x| x.0.as_str()).collect();
        assert_eq!(names, vec!["graphics/snapshot/snap_charset.vcchar", "graphics/snapshot/snap_screen.vcscreen", "graphics/snapshot/snap_sprite0.vcsprite", "graphics/snapshot/snap_sprite2.vcsprite"]);
        assert_eq!(a[0].1["frames"][1]["rows"][0], "00011000");
        assert_eq!(a[1].1["chars"][0][5], 5); assert_eq!(a[1].1["attrs"][0][3], 3); assert_eq!(a[1].1["charset"], "graphics/snapshot/snap_charset.vcchar");
        assert_eq!((a[1].1["colors"]["border"].clone(), a[1].1["colors"]["bg"].clone()), (json!(6), json!(14)));
        // abweichende Adresse von Hand (Bildschirm wurde an anderer Stelle gepuffert)
        f.ram[0x6000..0x6008].copy_from_slice(b"HALLO!!!");
        let o2 = Opts { screen: Some(0x6000), charset: None, rom: false, sprites: false, tiles: false, name: "buf".into() };
        let b = build_media(&d, &o2, &mut f, &profiles()).unwrap();
        assert_eq!(b.len(), 1); assert_eq!(b[0].1["chars"][0][0], b'H');
    }

    #[test]
    fn c64_rom_charset_bitmap_and_hidden_ram() {
        let mut f = Fake { ram: vec![0; 65536], io: vec![0; 4096], cpu_io: false };
        f.io[0xD00] = 0x03; f.io[0x18] = 0x15; f.io[0x11] = 0x1B;      // Bank 0, Zeichensatz bei $1000 -> Zeichen-ROM
        let d = detect("c64", &mut f).unwrap(); assert_eq!(d.charset, None); assert!(d.notes.iter().any(|n| n.contains("ROM")));
        let a = build_media(&d, &Opts { screen: Some(d.screen), charset: d.charset, rom: d.rom_off.is_some(), sprites: false, tiles: false, name: "x".into() }, &mut f, &profiles()).unwrap();
        assert_eq!(a.len(), 2, "Zeichensatz aus dem ROM und Bildschirm"); assert_eq!(a[0].1["frames"][0]["rows"][0], "10101010");
        let a = build_media(&d, &Opts { screen: Some(d.screen), charset: None, rom: false, sprites: false, tiles: false, name: "x".into() }, &mut f, &profiles()).unwrap(); assert_eq!(a.len(), 1);
        // Bank 3: Bildschirm bei $D000? -> unter dem E/A-Bereich; die RAM-Sicht liest das RAM darunter, die CPU-Sicht hätte E/A geliefert
        f.io[0xD00] = 0x00; f.io[0x18] = 0x04 | 0x00; f.ram[0xC400] = 0x41; f.io[0x11] = 0x1B;
        let d3 = detect("c64", &mut f).unwrap(); assert_eq!(d3.screen, 0xC000 + 0x0000, "{d3:?}");
        f.io[0x11] = 0x3B; assert!(!detect("c64", &mut f).unwrap().supported, "Bitmap");
    }

    #[test]
    fn c64_tiles_keep_colors() {
        let mut f = Fake { ram: vec![0; 65536], io: vec![0; 4096], cpu_io: false };
        f.io[0xD00] = 0x03; f.io[0x18] = 0x15; f.io[0x11] = 0x1B; f.io[0x20] = 14; f.io[0x21] = 6;
        for i in 0..1000 { f.ram[0x0400 + i] = 32; f.io[0x800 + i] = 1; }
        // Kachel „Stein“ (Zeichen 1,2,2,1, Farbe 12) an zwei Stellen im 2×2-Raster, eine dritte Stelle mit anderer Farbe (5)
        for (tx, ty, c) in [(0usize, 0usize, 12u8), (3, 2, 12), (5, 5, 5)] {
            let (x, y) = (tx * 2, ty * 2);
            for (dx, dy, ch) in [(0, 0, 1u8), (1, 0, 2), (0, 1, 2), (1, 1, 1)] { f.ram[0x0400 + (y + dy) * 40 + x + dx] = ch; f.io[0x800 + (y + dy) * 40 + x + dx] = c; }
        }
        let d = detect("c64", &mut f).unwrap();
        let a = build_media(&d, &Opts { screen: Some(0x0400), charset: None, rom: false, sprites: false, tiles: true, name: "t".into() }, &mut f, &profiles()).unwrap();
        let t = &a.iter().find(|x| x.0.ends_with("t_tiles.vctiles")).unwrap().1;
        let tiles = t["tiles"].as_array().unwrap();
        assert_eq!(tiles.len(), 3, "leer + Stein in 12 + Stein in 5");
        assert_eq!((tiles[1]["color"].clone(), tiles[2]["color"].clone()), (json!(12), json!(5)));
        assert_eq!((t["map"][0][0].clone(), t["map"][2][3].clone(), t["map"][5][5].clone(), t["map"][1][1].clone()), (json!(1), json!(1), json!(2), json!(0)));
        assert_eq!(t["colors"]["border"], 14);
        // Farben auch im Bildschirm: Farb-RAM der Zelle (0,0)
        assert_eq!(a.iter().find(|x| x.0.ends_with("t_screen.vcscreen")).unwrap().1["attrs"][0][0], 12);
    }

    #[test]
    fn atari_display_list_screen_and_charset() {
        let mut f = Fake { ram: vec![0; 65536], io: vec![0; 4096], cpu_io: false };
        f.ram[0x0230] = 0x00; f.ram[0x0231] = 0x30; f.ram[0x02F4] = 0x40; f.ram[0x02C5] = 0x0A; f.ram[0x02C6] = 0x90; f.ram[0x02C8] = 0x24;
        // Display-Liste bei $3000: 3 × 8 leere Zeilen, LMS+Modus 2 (Adresse $5000), 23 × Modus 2, JVB
        let dl = [0x70u8, 0x70, 0x70, 0x42, 0x00, 0x50, 2, 2, 2, 0x41, 0x00, 0x30];
        f.ram[0x3000..0x3000 + dl.len()].copy_from_slice(&dl);
        f.ram[0x5000..0x5005].copy_from_slice(&[0, 33, 34, 35, 36]); f.ram[0x4000 + 8..0x4000 + 16].copy_from_slice(&[0x3C; 8]);
        let d = detect("atari8", &mut f).unwrap();
        assert_eq!((d.screen, d.charset, d.supported), (0x5000, Some(0x4000), true), "{d:?}");
        assert_eq!((d.border, d.bg[0], d.fg), (0x24, 0x90, 0x9A));
        let a = build_media(&d, &Opts { screen: Some(d.screen), charset: d.charset, rom: d.rom_off.is_some(), sprites: false, tiles: false, name: "a".into() }, &mut f, &profiles()).unwrap();
        assert_eq!(a.len(), 2); assert_eq!(a[0].1["frames"].as_array().unwrap().len(), 128); assert_eq!(a[1].1["chars"][0][2], 34);
        f.ram[0x02F4] = 0xE0; assert_eq!(detect("atari8", &mut f).unwrap().charset, Some(0xE000));
        f.ram[0x3003] = 0x44; f.ram[0x3004..0x3006].copy_from_slice(&[0, 0x50]); assert!(detect("atari8", &mut f).unwrap().mode.contains("Modus 4")); assert!(!detect("atari8", &mut f).unwrap().supported);
        assert!(detect("plus4", &mut f).is_err());
    }

    /// Gegen echtes VICE: BASIC-Bildschirm eines gebooteten C64 (nur mit VC_EMU_TESTS=1)
    #[test]
    fn vice_live_snapshot() {
        if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let prg = root.join("build/border.prg"); assert!(prg.exists(), "build/border.prg fehlt");
        use vc_emu::Backend as _; let mut v = vc_emu::Vice::launch(prg.to_str().unwrap(), None).expect("VICE starten");
        std::thread::sleep(std::time::Duration::from_secs(3)); v.pause().ok(); std::thread::sleep(std::time::Duration::from_millis(300));
        let d = detect("c64", &mut BackendMem(&mut v)).unwrap();
        let pal = crate::bitmap::Palettes::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
        let pv = detect_with_preview("c64", &mut BackendMem(&mut v), &pal).unwrap().preview;
        let o = Opts { screen: Some(d.screen), charset: d.charset, rom: d.rom_off.is_some(), sprites: true, tiles: false, name: "live".into() };
        let a = build_media(&d, &o, &mut BackendMem(&mut v), &profiles());
        if let Some(p) = &pv { let _ = image::save_buffer("/tmp/vc_preview.png", &p.rgba, p.w as u32, p.h as u32, image::ColorType::Rgba8); }
        assert!(pv.is_some(), "Vorschau (Zeichen-ROM über VICE)");
        v.stop();
        eprintln!("{d:?}");
        let a = a.unwrap();
        assert_eq!(d.screen, 0x0400); assert!(d.charset.is_none(), "Standardschrift liegt im Zeichen-ROM");
        assert!(a.iter().any(|x| x.0.ends_with("live_screen.vcscreen")));
        let cs = a.iter().find(|x| x.0.ends_with("live_charset.vcchar")).expect("Zeichensatz aus dem ROM");
        assert_eq!(cs.1["frames"][1]["rows"][0], "00011000", "Zeichen A aus dem echten Zeichen-ROM");
    }

    /// Gegen echten atari800: Textbildschirm mit Farben aus den Schattenregistern (nur mit VC_EMU_TESTS=1)
    #[test]
    fn atari800_live_snapshot() {
        if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
        use vc_emu::Backend as _;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let xex = root.join("build/text_demo.xex"); assert!(xex.exists(), "build/text_demo.xex fehlt");
        let mut v = vc_emu::Atari800::launch(xex.to_str().unwrap(), None).expect("atari800 starten");
        // atari800 lässt sich nicht per Befehl anhalten: Haltepunkt auf die Endlosschleife (done: jmp done)
        v.set_breakpoint(0x201B).unwrap(); v.cont().unwrap();
        for _ in 0..60 { std::thread::sleep(std::time::Duration::from_millis(250)); if v.state() != vc_emu::RunState::Running { break; } }
        assert!(v.state() != vc_emu::RunState::Running, "Haltepunkt nicht erreicht");
        let pal = crate::bitmap::Palettes::load(&root.join("data"));
        let d = detect_with_preview("atari8", &mut BackendMem(&mut v), &pal);
        let d = match d { Ok(d) => d, Err(e) => { v.stop(); panic!("{e}") } };
        let o = Opts { screen: Some(d.screen), charset: d.charset, rom: false, sprites: false, tiles: true, name: "al".into() };
        let a = build_media(&d, &o, &mut BackendMem(&mut v), &profiles());
        v.stop();
        if let Some(p) = &d.preview { let _ = image::save_buffer("/tmp/vc_preview_atari.png", &p.rgba, p.w as u32, p.h as u32, image::ColorType::Rgba8); }
        let a = a.unwrap();
        let sc = &a.iter().find(|x| x.0.ends_with("al_screen.vcscreen")).unwrap().1;
        assert_eq!(sc["chars"][0][0], 40, "H = Bildschirmcode 40: {}", sc["chars"][0]);
        assert_eq!((d.bg[0], d.border), (0x94, 0x22));
    }
}
