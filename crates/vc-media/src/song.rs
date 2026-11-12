// SPDX-License-Identifier: GPL-3.0-or-later
//! Tracker-Songs (`kind: "song"`): Noten parsen und Pattern, Reihenfolge und Instrumente in Tabellen für die Spielroutinen kodieren.
//!
//! Zelle: `"C-4 01"` = Note + Instrument (hex, `--` = unverändert); Note `---` = nichts, `===` = Aus; `C#4` = Cis in Oktave 4. C-0 = 1 … B-7 = 96.
use serde_json::{json, Value};
use crate::Profile;

pub const NOTE_OFF: u8 = 0xFF;
pub const INS_KEEP: u8 = 0xFF;
const NAMES: [&str; 12] = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"];

pub fn note_name(n: u8) -> String { if n == 0 { "---".into() } else if n == NOTE_OFF { "===".into() } else { format!("{}{}", NAMES[((n - 1) % 12) as usize], (n - 1) / 12) } }

pub fn parse_cell(cell: &str) -> Result<(u8, u8), String> {
    let t: Vec<&str> = cell.split_whitespace().collect();
    let note = match t.first() {
        None | Some(&"---") | Some(&"...") => 0,
        Some(&"===") => NOTE_OFF,
        Some(n) if n.len() == 3 => {
            let (name, oct) = n.split_at(2);
            let k = NAMES.iter().position(|x| x.eq_ignore_ascii_case(name)).ok_or_else(|| format!("Note '{n}' unbekannt"))?;
            let o: u8 = oct.parse().map_err(|_| format!("Oktave in '{n}' ungültig"))?;
            if o > 7 { return Err(format!("Oktave in '{n}' außerhalb 0..7")); }
            (o * 12 + k as u8 + 1) as u8
        }
        Some(n) => return Err(format!("Note '{n}' unbekannt")),
    };
    let ins = match t.get(1) { None | Some(&"--") | Some(&"..") => INS_KEEP, Some(h) => u8::from_str_radix(h, 16).map_err(|_| format!("Instrument '{h}' ungültig (Hex)"))? };
    Ok((note, ins))
}

pub fn format_cell(note: u8, ins: u8) -> String { format!("{} {}", note_name(note), if ins == INS_KEEP { "--".to_string() } else { format!("{ins:02X}") }) }

pub fn new_song(p: &Profile, name: &str) -> Value {
    let (ch, rows) = (p.raw["channels"].as_u64().unwrap_or(3) as usize, p.raw["rows"].as_u64().unwrap_or(32) as usize);
    let empty = || json!({ "rows": (0..rows).map(|_| (0..ch).map(|_| "--- --").collect::<Vec<_>>()).collect::<Vec<_>>() });
    let sid = p.raw["chip"] == "sid";
    let ins = if sid { json!([{ "name": "Puls", "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 }, { "name": "Bass", "wave": "saw", "pw": 2048, "a": 0, "d": 6, "s": 10, "r": 2 }]) }
              else { json!([{ "name": "Ton", "dist": "pure", "env": [15, 14, 12, 10, 8, 6, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0], "loop": 15 }, { "name": "Rauschen", "dist": "noise", "env": [15, 10, 6, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "loop": 15 }]) };
    json!({ "format": 1, "kind": "song", "profile": p.id, "name": name, "speed": 6, "rows": rows, "instruments": ins, "patterns": [empty()], "order": [0], "loop": 0 })
}

fn wave_ctl(w: &str) -> Result<u8, String> {
    Ok(match w { "triangle" | "tri" => 0x10, "saw" | "sawtooth" => 0x20, "pulse" | "square" => 0x40, "noise" => 0x80, o => return Err(format!("Wellenform '{o}' unbekannt (triangle, saw, pulse, noise)")) })
}
fn dist_bits(d: &str) -> Result<u8, String> {
    Ok(match d { "pure" => 0xA0, "noise" => 0x80, "buzz" => 0xC0, "gritty" => 0x20, "bass" => 0x40, o => return Err(format!("Verzerrung '{o}' unbekannt (pure, noise, buzz, gritty, bass)")) })
}

/// Kodierte Teile: (`order`: Pattern-Nummern, `$ff`, Wiederholungsposition), (`patterns`: je Pattern Zeilen × Stimmen × (Note, Instrument)), (`ins`: je Instrument ein Datensatz).
/// SID-Datensatz (8 Bytes): Steuerbyte (Wellenform), AD, SR, PW low, PW high, 0, 0, 0.  POKEY-Datensatz (32 Bytes): Verzerrung (AUDC oben), Wiederholungsstelle, 16 Lautstärken, 0.
pub fn encode(root: &Value, p: &Profile) -> Result<Vec<(String, Vec<u8>)>, String> {
    let (ch, rows) = (p.raw["channels"].as_u64().unwrap_or(3) as usize, root["rows"].as_u64().unwrap_or(32) as usize);
    let sid = p.raw["chip"] == "sid";
    let pats = root["patterns"].as_array().ok_or("patterns fehlt")?;
    let ins = root["instruments"].as_array().ok_or("instruments fehlt")?;
    let max_ins = if sid { 32 } else { 8 };   // die Spielroutinen indizieren die Instrumente mit einem Byte (SID: 8 Bytes je Satz, POKEY: 32)
    if ins.is_empty() || ins.len() > max_ins { return Err(format!("1..{max_ins} Instrumente erwartet (Spielroutine)")); }
    let mut pat_bytes = vec![];
    for (pi, pat) in pats.iter().enumerate() {
        let r = pat["rows"].as_array().ok_or(format!("Pattern {pi}: rows fehlt"))?;
        if r.len() != rows { return Err(format!("Pattern {pi}: {} statt {rows} Zeilen", r.len())); }
        for (ri, row) in r.iter().enumerate() {
            let cells = row.as_array().ok_or(format!("Pattern {pi} Zeile {ri}: Liste erwartet"))?;
            if cells.len() != ch { return Err(format!("Pattern {pi} Zeile {ri}: {} statt {ch} Stimmen", cells.len())); }
            for (ci, c) in cells.iter().enumerate() {
                let (n, i) = parse_cell(c.as_str().unwrap_or("")).map_err(|e| format!("Pattern {pi} Zeile {ri} Stimme {ci}: {e}"))?;
                if i != INS_KEEP && i as usize >= ins.len() { return Err(format!("Pattern {pi} Zeile {ri} Stimme {ci}: Instrument {i:02X} existiert nicht")); }
                pat_bytes.push(n); pat_bytes.push(i);
            }
        }
    }
    let mut order = vec![];
    for o in root["order"].as_array().ok_or("order fehlt")? { let v = o.as_u64().ok_or("order: Zahl erwartet")? as usize; if v >= pats.len() { return Err(format!("order verweist auf Pattern {v}, es gibt nur {}", pats.len())); } order.push(v as u8); }
    if order.is_empty() { return Err("order ist leer".into()); }
    let lp = root["loop"].as_u64().unwrap_or(0) as usize; if lp >= order.len() { return Err("loop außerhalb der Reihenfolge".into()); }
    order.push(0xFF); order.push(lp as u8);
    let mut ins_bytes = vec![];
    for (i, x) in ins.iter().enumerate() {
        let g = |k: &str, max: u64| -> Result<u64, String> { let v = x[k].as_u64().unwrap_or(0); if v > max { Err(format!("Instrument {i}: {k} = {v} außerhalb 0..{max}")) } else { Ok(v) } };
        if sid {
            let ctl = wave_ctl(x["wave"].as_str().unwrap_or("pulse")).map_err(|e| format!("Instrument {i}: {e}"))?;
            let pw = g("pw", 4095)?;
            ins_bytes.extend([ctl, (g("a", 15)? << 4 | g("d", 15)?) as u8, (g("s", 15)? << 4 | g("r", 15)?) as u8, (pw & 255) as u8, (pw >> 8) as u8, 0, 0, 0]);
        } else {
            let d = dist_bits(x["dist"].as_str().unwrap_or("pure")).map_err(|e| format!("Instrument {i}: {e}"))?;
            let env = x["env"].as_array().ok_or(format!("Instrument {i}: env fehlt"))?;
            if env.is_empty() || env.len() > 16 { return Err(format!("Instrument {i}: env braucht 1..16 Werte")); }
            let mut e: Vec<u8> = env.iter().map(|v| v.as_u64().unwrap_or(0).min(15) as u8).collect();
            let last = *e.last().unwrap(); e.resize(16, last);
            ins_bytes.push(d); ins_bytes.push(g("loop", 15)? as u8); ins_bytes.extend(e); ins_bytes.extend([0u8; 14]);
        }
    }
    Ok(vec![("order".into(), order), ("patterns".into(), pat_bytes), ("ins".into(), ins_bytes)])
}

/// Konstanten für den Export: Geschwindigkeit, Zeilen, Stimmen, Patterngröße, Anzahl
pub fn consts(label: &str, root: &Value, p: &Profile, sbas: bool) -> String {
    let c = if sbas { "const" } else { ".const" };
    let (ch, rows) = (p.raw["channels"].as_u64().unwrap_or(3), root["rows"].as_u64().unwrap_or(32));
    format!("{c} {label}_speed = {}\n{c} {label}_rows = {rows}\n{c} {label}_channels = {ch}\n{c} {label}_patsize = {}\n{c} {label}_patterns_count = {}\n{c} {label}_ins_count = {}\n",
            root["speed"].as_u64().unwrap_or(6), rows * ch * 2, root["patterns"].as_array().map_or(0, |a| a.len()), root["instruments"].as_array().map_or(0, |a| a.len()))
}

// ---------------------------------------------------------------------------------------------------------------------
// RMT-Module (Raster Music Tracker, Format RMT4): POKEY-Songs werden als Modul für den Original-RMT-Player ausgegeben.
// Aufbau: Kopf (RMT4, Tracklänge, Takt, Instrumentgeschwindigkeit, Version, vier Zeiger), Instrumententabelle, Track-Zeiger (lo/hi getrennt),
// Instrumente, Tracks, Songzeilen (4 Tracknummern je Zeile, Ende = Sprungzeile $FE).
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RelocKind { Word, Lo, Hi }

#[derive(Debug)]
pub struct Rmt { pub bytes: Vec<u8>, pub relocs: Vec<(usize, RelocKind, usize)> }

/// Erste Note (C-3, 130,8 Hz) entspricht RMT-Note 0; die RMT-Frequenztabellen reichen bis Note 60.
pub const RMT_FIRST_NOTE: u8 = 37;
const RMT_DIST: [(&str, u8); 5] = [("pure", 5), ("noise", 4), ("buzz", 3), ("gritty", 1), ("bass", 2)];

fn rmt_instrument(x: &Value, i: usize) -> Result<Vec<u8>, String> {
    let d = x["dist"].as_str().unwrap_or("pure");
    let dist = RMT_DIST.iter().find(|(n, _)| *n == d).ok_or(format!("Instrument {i}: Verzerrung '{d}' unbekannt"))?.1;
    let env = x["env"].as_array().ok_or(format!("Instrument {i}: env fehlt"))?;
    if env.is_empty() || env.len() > 16 { return Err(format!("Instrument {i}: env braucht 1..16 Werte")); }
    let mut e: Vec<u8> = env.iter().map(|v| v.as_u64().unwrap_or(0).min(15) as u8).collect();
    let last = *e.last().unwrap(); e.resize(16, last);
    let lp = x["loop"].as_u64().unwrap_or(15).min(15) as u8;
    // Kopf (12 Bytes): Tabellenende, Tabellenschleife, Länge der Hüllkurve, Schleifenstart, Tabellentyp/-tempo, AUDCTL, Lautstärke-Slide, Mindestlautstärke, Effektverzögerung, Vibrato, Frequenzverschiebung, 0
    let mut v = vec![12, 12, 13 + 3 * 15, 13 + 3 * lp, 0, 0, 0, 0, 0, 0, 0, 0, 0 /* Tabelle: Note +0 */];
    for vol in e { v.extend([vol << 4 | vol, dist << 1, 0]); }
    Ok(v)
}

/// Ein Track (eine Stimme eines Patterns) als RMT-Ereignisfolge; `cur_ins` = zuletzt gesetztes Instrument der Stimme
fn rmt_track(cells: &[(u8, u8)], cur_ins: &mut u8, who: &str) -> Result<Vec<u8>, String> {
    let (mut out, mut pause) = (vec![], 0usize);
    let flush = |out: &mut Vec<u8>, pause: &mut usize| { while *pause > 0 { let n = (*pause).min(255); if n <= 3 { out.push(62 | (n as u8) << 6); } else { out.extend([62, n as u8]); } *pause -= n; } };
    for (r, &(n, i)) in cells.iter().enumerate() {
        if i != INS_KEEP { *cur_ins = i; }
        if n == 0 { pause += 1; continue; }
        flush(&mut out, &mut pause);
        if n == NOTE_OFF { out.extend([61, 0]); continue; }       // nur Lautstärke: 0
        if n < RMT_FIRST_NOTE { return Err(format!("{who} Zeile {r}: Note {} liegt unter dem POKEY-Bereich (ab C-3)", note_name(n))); }
        let rn = n - RMT_FIRST_NOTE; if rn > 60 { return Err(format!("{who} Zeile {r}: Note {} zu hoch für RMT", note_name(n))); }
        out.extend([rn | 3 << 6, *cur_ins << 2 | 3]);             // Lautstärke 15 (Bits 0-1 im ersten, 2-3 im zweiten Byte)
    }
    flush(&mut out, &mut pause);
    Ok(out)
}

pub fn rmt_build(root: &Value, p: &Profile) -> Result<Rmt, String> {
    if p.raw["channels"].as_u64() != Some(4) { return Err("RMT-Export nur für 4 Stimmen (POKEY)".into()); }
    let rows = root["rows"].as_u64().unwrap_or(32) as usize; if rows == 0 || rows > 255 { return Err("RMT: 1..255 Zeilen je Pattern".into()); }
    let pats = root["patterns"].as_array().ok_or("patterns fehlt")?; let ins = root["instruments"].as_array().ok_or("instruments fehlt")?;
    if ins.is_empty() || ins.len() > 64 { return Err("RMT: 1..64 Instrumente".into()); }
    let instr: Vec<Vec<u8>> = ins.iter().enumerate().map(|(i, x)| rmt_instrument(x, i)).collect::<Result<_, _>>()?;
    let order: Vec<usize> = root["order"].as_array().ok_or("order fehlt")?.iter().map(|o| o.as_u64().unwrap_or(u64::MAX) as usize).collect();
    if order.is_empty() || order.iter().any(|&o| o >= pats.len()) { return Err("order ungültig".into()); }
    let lp = root["loop"].as_u64().unwrap_or(0) as usize; if lp >= order.len() { return Err("loop außerhalb der Reihenfolge".into()); }
    // Tracks je Songposition und Stimme (Instrument-Übernahme folgt der Spielreihenfolge), gleiche Tracks teilen sich eine Nummer
    let mut tracks: Vec<Vec<u8>> = vec![]; let mut lines: Vec<[u8; 4]> = vec![]; let mut cur = [0u8; 4];
    for (pos, &pi) in order.iter().enumerate() {
        let r = pats[pi]["rows"].as_array().ok_or(format!("Pattern {pi}: rows fehlt"))?; if r.len() != rows { return Err(format!("Pattern {pi}: {} statt {rows} Zeilen", r.len())); }
        let mut line = [0u8; 4];
        for ch in 0..4 {
            let cells: Vec<(u8, u8)> = r.iter().enumerate().map(|(ri, row)| {
                let c = row.as_array().and_then(|a| a.get(ch)).and_then(|c| c.as_str()).ok_or(format!("Pattern {pi} Zeile {ri}: Stimme {ch} fehlt"))?;
                let (n, i) = parse_cell(c).map_err(|e| format!("Pattern {pi} Zeile {ri} Stimme {ch}: {e}"))?;
                if i != INS_KEEP && i as usize >= ins.len() { return Err(format!("Pattern {pi} Zeile {ri} Stimme {ch}: Instrument {i:02X} existiert nicht")); }
                Ok((n, i))
            }).collect::<Result<_, String>>()?;
            let t = rmt_track(&cells, &mut cur[ch], &format!("Pattern {pi} Stimme {ch}"))?;
            let idx = tracks.iter().position(|x| *x == t).unwrap_or_else(|| { tracks.push(t); tracks.len() - 1 });
            if idx > 253 { return Err("RMT: höchstens 254 verschiedene Tracks".into()); }
            line[ch] = idx as u8;
        }
        let _ = pos; lines.push(line);
    }
    // Layout
    let mut b: Vec<u8> = b"RMT4".to_vec(); let mut rel: Vec<(usize, RelocKind, usize)> = vec![];
    b.extend([rows as u8, root["speed"].as_u64().unwrap_or(6).clamp(1, 255) as u8, 1, 0]);
    let hdr_ptr = b.len(); b.extend([0u8; 8]);
    let tis = b.len(); b.extend(vec![0u8; ins.len() * 2]);
    let tlo = b.len(); b.extend(vec![0u8; tracks.len()]); let thi = b.len(); b.extend(vec![0u8; tracks.len()]);
    for (i, x) in instr.iter().enumerate() { rel.push((tis + 2 * i, RelocKind::Word, b.len())); b.extend(x); }
    for (i, t) in tracks.iter().enumerate() { rel.push((tlo + i, RelocKind::Lo, b.len())); rel.push((thi + i, RelocKind::Hi, b.len())); b.extend(t); }
    let song = b.len(); for l in &lines { b.extend(l); }
    let goto = b.len(); b.extend([0xFE, lp as u8, 0, 0]); rel.push((goto + 2, RelocKind::Word, song + lp * 4));
    for (k, target) in [tis, tlo, thi, song].into_iter().enumerate() { rel.push((hdr_ptr + 2 * k, RelocKind::Word, target)); }
    Ok(Rmt { bytes: b, relocs: rel })
}

impl Rmt {
    /// Binärdatei im Atari-Format (Kopf $FFFF, Start, Ende) für die feste Ladeadresse `addr`
    pub fn to_file(&self, addr: u16) -> Vec<u8> {
        let mut b = self.bytes.clone();
        for &(pos, k, target) in &self.relocs {
            let v = addr as usize + target;
            match k { RelocKind::Word => { b[pos] = v as u8; b[pos + 1] = (v >> 8) as u8; } RelocKind::Lo => b[pos] = v as u8, RelocKind::Hi => b[pos] = (v >> 8) as u8 }
        }
        let end = addr as usize + b.len() - 1;
        let mut f = vec![0xFF, 0xFF, addr as u8, (addr >> 8) as u8, end as u8, (end >> 8) as u8]; f.extend(b); f
    }
    /// Assemblerquelle mit Label `label`; Zeiger werden als Ausdrücke `label+offset` ausgegeben (frei verschiebbar)
    pub fn to_asm(&self, label: &str) -> String {
        let mut s = format!("; erzeugt von VintageCoder – RMT-Modul (Raster Music Tracker, RMT4); Start: {label}\n{label}:\n");
        let mut rel = self.relocs.clone(); rel.sort_by_key(|r| r.0);
        let (mut i, mut ri) = (0usize, 0usize); let mut run: Vec<u8> = vec![];
        let flush = |s: &mut String, run: &mut Vec<u8>| { for c in run.chunks(12) { s.push_str(&format!("        .byte {}\n", c.iter().map(|b| format!("${b:02x}")).collect::<Vec<_>>().join(","))); } run.clear(); };
        while i < self.bytes.len() {
            if ri < rel.len() && rel[ri].0 == i {
                flush(&mut s, &mut run); let (_, k, t) = rel[ri]; ri += 1;
                match k { RelocKind::Word => { s.push_str(&format!("        .word {label}+{t}\n")); i += 2; } RelocKind::Lo => { s.push_str(&format!("        .byte <({label}+{t})\n")); i += 1; } RelocKind::Hi => { s.push_str(&format!("        .byte >({label}+{t})\n")); i += 1; } }
            } else { run.push(self.bytes[i]); i += 1; }
        }
        flush(&mut s, &mut run);
        s.push_str(&format!(".const {label}_len = {}\n", self.bytes.len())); s
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Konvertierung zwischen SID- und POKEY-Songs (Noten und Struktur bleiben, Instrumente werden angenähert)
// ---------------------------------------------------------------------------------------------------------------------

const SID_ATTACK_MS: [f64; 16] = [2.0, 8.0, 16.0, 24.0, 38.0, 56.0, 68.0, 80.0, 100.0, 250.0, 500.0, 800.0, 1000.0, 3000.0, 5000.0, 8000.0];
const SID_DECAY_MS: [f64; 16] = [6.0, 24.0, 48.0, 72.0, 114.0, 168.0, 204.0, 240.0, 300.0, 750.0, 1500.0, 2400.0, 3000.0, 9000.0, 15000.0, 24000.0];
const FRAME_MS: f64 = 20.0;

fn nearest_idx(table: &[f64; 16], ms: f64) -> u64 { (0..16).min_by(|&a, &b| (table[a] - ms).abs().partial_cmp(&(table[b] - ms).abs()).unwrap()).unwrap() as u64 }

/// Lautstärkeverlauf eines SID-Instruments (0..1) zur Zeit `t` ms nach dem Einschalten
fn sid_level(a: f64, d: f64, s: f64, t: f64) -> f64 { if t < a { t / a } else { s + (1.0 - s) * (-(t - a) / (d / 3.0)).exp() } }

fn sid_to_pokey_instrument(x: &Value, warn: &mut Vec<String>, i: usize) -> Value {
    let wave = x["wave"].as_str().unwrap_or("pulse");
    let dist = match wave { "noise" => "noise", "saw" | "sawtooth" => "buzz", _ => "pure" };
    let (a, d, s) = (SID_ATTACK_MS[x["a"].as_u64().unwrap_or(0).min(15) as usize], SID_DECAY_MS[x["d"].as_u64().unwrap_or(0).min(15) as usize], x["s"].as_u64().unwrap_or(15).min(15) as f64 / 15.0);
    let mut env: Vec<u64> = (0..15).map(|k| (sid_level(a, d, s, (k as f64 + 0.5) * FRAME_MS) * 15.0).round() as u64).collect();
    env.push((s * 15.0).round() as u64);                           // Schritt 16 = Haltepegel (Sustain)
    if a + 3.0 * d > 16.0 * FRAME_MS && s < 1.0 { warn.push(format!("Instrument {i:02X}: Hüllkurve auf 16 Bilder gekürzt (Attack/Decay länger als {} ms)", 16.0 * FRAME_MS)); }
    json!({ "name": x["name"], "dist": dist, "env": env, "loop": 15 })
}

fn pokey_to_sid_instrument(x: &Value) -> Value {
    let env: Vec<u64> = x["env"].as_array().map(|a| a.iter().map(|v| v.as_u64().unwrap_or(0).min(15)).collect()).unwrap_or_default();
    let wave = match x["dist"].as_str().unwrap_or("pure") { "noise" | "gritty" => "noise", "buzz" => "saw", "bass" => "triangle", _ => "pulse" };
    let peak = env.iter().copied().max().unwrap_or(15).max(1); let pi = env.iter().position(|&v| v == peak).unwrap_or(0);
    let sustain = env.last().copied().unwrap_or(0);
    let s = ((sustain as f64 / peak as f64) * 15.0).round() as u64;
    let a = nearest_idx(&SID_ATTACK_MS, (pi as f64 + 1.0) * FRAME_MS);
    // Decay: Dauer bis der Pegel in der Nähe des Haltepegels ist
    let settle = env.iter().enumerate().skip(pi).find(|(_, &v)| (v as i64 - sustain as i64).abs() <= 1).map(|(k, _)| k).unwrap_or(env.len());
    let d = nearest_idx(&SID_DECAY_MS, ((settle.saturating_sub(pi)) as f64 + 1.0) * FRAME_MS * 3.0 / 3.0);
    json!({ "name": x["name"], "wave": wave, "pw": 2048, "a": a, "d": d, "s": s, "r": 3 })
}

/// Konvertiert einen Song in das Profil `to` (anderer Chip). Gibt das neue Media und Hinweise zurück.
pub fn convert(root: &Value, from: &Profile, to: &Profile) -> Result<(Value, Vec<String>), String> {
    if from.kind != "song" || to.kind != "song" { return Err("Konvertierung nur für Songs".into()); }
    if from.raw["chip"] == to.raw["chip"] { return Err("Quell- und Zielprofil haben denselben Chip".into()); }
    let (fc, tc) = (from.raw["channels"].as_u64().unwrap_or(3) as usize, to.raw["channels"].as_u64().unwrap_or(3) as usize);
    let (to_pokey, mut warn) = (to.raw["chip"] == "pokey", vec![]);
    let ins = root["instruments"].as_array().ok_or("instruments fehlt")?;
    let new_ins: Vec<Value> = ins.iter().enumerate().map(|(i, x)| if to_pokey { sid_to_pokey_instrument(x, &mut warn, i) } else { pokey_to_sid_instrument(x) }).collect();
    // Stimmenzuordnung: SID→POKEY: 0..2 → 0..2, vierte leer; POKEY→SID: die am wenigsten benutzte Stimme entfällt
    let pats = root["patterns"].as_array().ok_or("patterns fehlt")?;
    let mut used = vec![0usize; fc];
    for p in pats { for row in p["rows"].as_array().into_iter().flatten() { for (c, cell) in row.as_array().into_iter().flatten().enumerate() { if c < fc && parse_cell(cell.as_str().unwrap_or("")).map(|(n, _)| n != 0).unwrap_or(false) { used[c] += 1; } } } }
    let mut map: Vec<Option<usize>> = (0..tc).map(|c| if c < fc { Some(c) } else { None }).collect();
    if fc > tc {
        let drop = (0..fc).rev().min_by_key(|&c| used[c]).unwrap();
        let keep: Vec<usize> = (0..fc).filter(|&c| c != drop).collect(); map = keep.into_iter().map(Some).collect();
        warn.push(format!("Stimme {} entfällt ({} Noten/Ereignisse)", drop + 1, used[drop]));
    }
    let mut shifted = 0usize;
    let new_pats: Vec<Value> = pats.iter().map(|p| {
        let rows: Vec<Value> = p["rows"].as_array().into_iter().flatten().map(|row| {
            let cells = row.as_array().cloned().unwrap_or_default();
            Value::Array(map.iter().map(|m| match m {
                None => Value::String("--- --".into()),
                Some(c) => {
                    let (mut n, i) = parse_cell(cells.get(*c).and_then(|v| v.as_str()).unwrap_or("")).unwrap_or((0, INS_KEEP));
                    if to_pokey && n != 0 && n != NOTE_OFF && n < RMT_FIRST_NOTE { while n < RMT_FIRST_NOTE { n += 12; } shifted += 1; }
                    Value::String(format_cell(n, i))
                }
            }).collect())
        }).collect();
        json!({ "rows": rows })
    }).collect();
    if shifted > 0 { warn.push(format!("{shifted} Noten lagen unter C-3 (POKEY-Bereich) und wurden oktaviert")); }
    if !to_pokey { warn.push("Klang: Wellenform und ADSR sind aus der Verzerrung und Lautstärkehüllkurve geschätzt – bitte nachhören".into()); }
    let mut out = root.clone();
    out["profile"] = json!(to.id); out["instruments"] = Value::Array(new_ins); out["patterns"] = Value::Array(new_pats);
    Ok((out, warn))
}

// ---------------------------------------------------------------------------------------------------------------------
// RMT-Import: Raster-Music-Tracker-Modul (RMT4, Mono) → POKEY-Song. Übernommen werden Noten, Pausen, Note-aus, Instrument-Nummern,
// Hüllkurven (erste 16 Schritte: Lautstärke, Verzerrung, Wiederholungsstelle) und Songaufbau; Effekte/Tabellen/Lautstärken nicht.
// ---------------------------------------------------------------------------------------------------------------------

fn atari_image(data: &[u8]) -> Result<(Vec<u8>, usize), String> {
    if data.len() < 6 || data[0] != 0xFF || data[1] != 0xFF { return Err("keine Atari-Binärdatei (Kopf $FFFF fehlt) – das Modul braucht seine Ladeadresse".into()); }
    let (mut img, mut i, mut base) = (vec![0u8; 0x10000], 0usize, None);
    while i + 4 <= data.len() {
        if data[i] == 0xFF && data[i + 1] == 0xFF { i += 2; continue; }
        let (s, e) = (data[i] as usize | (data[i + 1] as usize) << 8, data[i + 2] as usize | (data[i + 3] as usize) << 8); i += 4;
        if e < s || i + (e - s + 1) > data.len() { return Err("Binärdatei beschädigt".into()); }
        img[s..=e].copy_from_slice(&data[i..i + e - s + 1]); i += e - s + 1;
        if base.is_none() && &img[s..(s + 3).min(0x10000)] == b"RMT" { base = Some(s); }
    }
    Ok((img, base.ok_or("kein RMT-Modul (Kennung 'RMT4' am Blockanfang nicht gefunden)")?))
}

pub fn rmt_import(data: &[u8], p: &Profile, name: &str) -> Result<(Value, Vec<String>), String> {
    let (img, base) = atari_image(data)?;
    match &img[base..base + 4] { b"RMT4" => {} b"RMT8" => return Err("RMT8 (Stereo, 8 Spuren) wird nicht unterstützt".into()), _ => return Err("unbekannte Modulkennung (erwartet RMT4)".into()) }
    let w = |a: usize| img[a] as usize | (img[a + 1] as usize) << 8;
    let (maxlen, speed, instrspeed) = (img[base + 4] as usize, img[base + 5].max(1), img[base + 6]);
    let (tis, tlo, thi, song) = (w(base + 8), w(base + 10), w(base + 12), w(base + 14));
    if maxlen == 0 || maxlen > 128 { return Err(format!("Tracklänge {maxlen} wird nicht unterstützt (1..128)")); }
    if !(tis <= tlo && tlo <= thi) { return Err("Zeigertabellen im Kopf ungültig".into()); }
    let mut warn = vec![];
    if instrspeed != 1 { warn.push(format!("Instrumentgeschwindigkeit {instrspeed} (Hüllkurven laufen {instrspeed}× pro Bild) wird als 1 behandelt")); }
    // Instrumente
    let n_ins = ((tlo - tis) / 2).min(64);
    let mut instruments = vec![]; let mut effects = false;
    for i in 0..n_ins {
        let a = w(tis + 2 * i);
        if a == 0 || a < tis || a + 13 > 0x10000 { instruments.push(json!({ "name": format!("Leer {i:02X}"), "dist": "pure", "env": [0], "loop": 0 })); continue; }
        let (tableend, instrlen, instrlop) = (img[a] as usize, img[a + 2] as usize, img[a + 3] as usize);
        let env0 = tableend + 1;
        if instrlen < env0 { instruments.push(json!({ "name": format!("Defekt {i:02X}"), "dist": "pure", "env": [0], "loop": 0 })); continue; }
        let steps = (instrlen - env0) / 3 + 1;
        if tableend > 12 || img[a + 5..a + 11].iter().any(|&b| b != 0) || img[a + 11] != 0 { effects = true; }
        let mut env = vec![]; let mut dists = vec![];
        for k in 0..steps {
            let (r1, r2, r3) = (img[a + env0 + 3 * k], img[a + env0 + 3 * k + 1], img[a + env0 + 3 * k + 2]);
            env.push((r1 & 0x0f) as u64); dists.push((r2 >> 1) & 7);
            if r2 & 0xF1 != 0 || r3 != 0 { effects = true; }
        }
        if steps > 16 { warn.push(format!("Instrument {i:02X}: {steps} Hüllkurvenschritte, nur die ersten 16 übernommen")); }
        let lp = (instrlop.saturating_sub(env0) / 3).min(15);
        env.truncate(16); if env.len() < 16 { let last = *env.last().unwrap_or(&0); env.resize(16, last); }
        let lp = if lp >= steps.min(16) { 15 } else { lp };
        if dists.iter().any(|&d| d != dists[0]) { warn.push(format!("Instrument {i:02X}: wechselnde Verzerrung, nur die des ersten Schritts übernommen")); }
        let dist = match dists[0] { 5 => "pure", 4 | 0 => "noise", 3 => "buzz", 1 => "gritty", _ => "bass" };
        if dists[0] >= 6 { warn.push(format!("Instrument {i:02X}: Verzerrung {} (16-Bit-Bass) als „bass“ angenähert", dists[0])); }
        instruments.push(json!({ "name": format!("Instrument {i:02X}"), "dist": dist, "env": env, "loop": lp }));
    }
    if effects { warn.push("Effekte, Tabellen, Portamento, Filter oder AUDCTL in Instrumenten werden nicht übernommen".into()); }
    if instruments.is_empty() { return Err("Modul enthält keine Instrumente".into()); }
    // Tracks → Zellen
    let n_tracks = thi - tlo; let mut volumes_ignored = false; let mut below = 0;
    let mut tracks: Vec<Vec<String>> = vec![];
    for t in 0..n_tracks {
        let mut p0 = img[tlo + t] as usize | (img[thi + t] as usize) << 8; let mut cells = vec!["--- --".to_string(); maxlen]; let mut row = 0;
        'track: while row < maxlen && p0 < 0xFFFF {
            let b0 = img[p0]; let n = b0 & 0x3f; p0 += 1;
            match n {
                0..=60 => { let b1 = img[p0]; p0 += 1; let (ins, vol) = ((b1 >> 2) as usize, (b1 & 3) << 2 | b0 >> 6);
                            if ins >= instruments.len() { return Err(format!("Track {t}: Instrument {ins} existiert nicht")); }
                            cells[row] = if vol == 0 { "=== --".into() } else { if vol != 15 { volumes_ignored = true; } format_cell(n + RMT_FIRST_NOTE, ins as u8) }; row += 1; }
                61 => { let b1 = img[p0]; p0 += 1; if (b1 & 3) << 2 | b0 >> 6 == 0 { cells[row] = "=== --".into(); } row += 1; }
                62 => { let c = if b0 >> 6 == 0 { let c = img[p0] as usize; p0 += 1; c } else { (b0 >> 6) as usize }; row += c; }
                _ => { if b0 & 0x80 == 0 { warn.push(format!("Track {t}: Tempowechsel im Track wird ignoriert")); p0 += 1; } else { break 'track; } }   // $FF = Trackende, sonst Sprung im Track
            }
        }
        let _ = &mut below; tracks.push(cells);
    }
    if volumes_ignored { warn.push("Lautstärken unter 15 in den Tracks werden nicht übernommen (der Tracker kennt keine Lautstärkespalte)".into()); }
    // Songzeilen → Pattern/Reihenfolge
    let (mut order, mut patterns, mut keys, mut lp, mut a, mut guard) = (vec![], vec![], vec![], 0usize, song, 0);
    loop {
        guard += 1; if guard > 255 || a + 4 > 0x10000 { return Err("Songende (Sprungzeile) nicht gefunden".into()); }
        if img[a] == 0xFE { let target = w(a + 2); if target >= song && (target - song) % 4 == 0 { lp = ((target - song) / 4).min(order.len().saturating_sub(1)); } break; }
        let line: Vec<u8> = img[a..a + 4].to_vec(); a += 4;
        if line.iter().any(|&t| t != 0xFF && t as usize >= tracks.len()) { return Err("Songzeile verweist auf einen fehlenden Track".into()); }
        let idx = keys.iter().position(|k| *k == line).unwrap_or_else(|| {
            keys.push(line.clone());
            patterns.push(json!({ "rows": (0..maxlen).map(|r| (0..4).map(|c| if line[c] == 0xFF { "--- --".to_string() } else { tracks[line[c] as usize][r].clone() }).collect::<Vec<_>>()).collect::<Vec<_>>() }));
            patterns.len() - 1
        });
        order.push(idx);
    }
    if order.is_empty() { return Err("Song hat keine Zeilen".into()); }
    let ins_json: Vec<Value> = instruments;
    Ok((json!({ "format": 1, "kind": "song", "profile": p.id, "name": name, "speed": speed, "rows": maxlen, "instruments": ins_json, "patterns": patterns, "order": order, "loop": lp }), warn))
}
