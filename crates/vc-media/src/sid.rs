// SPDX-License-Identifier: GPL-3.0-or-later
//! SID-Datei (PSID/RSID, `*.sid`) → SID-Song durch Aufnahme.
//!
//! Eine `.sid`-Datei enthält ein 6502-Programm (Spielroutine) mit den Musikdaten, aber keine Patterns. Deshalb wird das Programm in einem kleinen
//! 6502-Kern abgespielt und für jedes Bild der Stand der SID-Register aufgezeichnet. Aus Gate, Tonhöhe, Wellenform, Hüllkurve und Pulsbreite werden
//! Noten und Instrumente abgeleitet. Das ist eine Näherung des Klangs, nicht die Quelle des Stücks: Vibrato, Portamento, Filter, Ringmodulation und
//! Synchronisation werden nicht übernommen, Arpeggios werden als Einzelnoten im Rasterschritt der Zeilen erfasst.
use crate::cpu6502::{Bus, Cpu};
use crate::song::{format_cell, INS_KEEP, NOTE_OFF};
use crate::Profile;
use serde_json::{json, Value};
use std::collections::HashMap;

const SENTINEL: u16 = 0xFFF0;
const PAL_CLOCK: f64 = 985_248.0;
const NTSC_CLOCK: f64 = 1_022_727.0;

#[derive(Clone, Debug)]
pub struct SidInfo { pub title: String, pub author: String, pub released: String, pub songs: u16, pub start_song: u16, pub rsid: bool, pub ntsc: bool, pub load: u16, pub init: u16, pub play: u16, pub speed: u32, pub sids: usize }

#[derive(Clone, Debug)]
pub struct SidOpts { /// 1-basierter Subtune; 0 = Startsong der Datei
    pub song: u16, /// aufzunehmende Sekunden
    pub seconds: u32 }
impl Default for SidOpts { fn default() -> Self { SidOpts { song: 0, seconds: 30 } } }

fn text(b: &[u8]) -> String { b.iter().take_while(|&&c| c != 0).map(|&c| if (32..127).contains(&c) { c as char } else { '?' }).collect::<String>().trim().to_string() }
fn be16(b: &[u8], o: usize) -> u16 { (b[o] as u16) << 8 | b[o + 1] as u16 }

/// Kopf einer PSID/RSID-Datei lesen
pub fn info(data: &[u8]) -> Result<(SidInfo, usize), String> {
    if data.len() < 0x76 || !(data.starts_with(b"PSID") || data.starts_with(b"RSID")) { return Err("keine SID-Datei (PSID/RSID erwartet)".into()); }
    let version = be16(data, 4); let off = be16(data, 6) as usize;
    if !(1..=4).contains(&version) || off < 0x76 || off > data.len() { return Err(format!("SID-Kopf ungültig (Version {version}, Datenbeginn ${off:X})")); }
    let (flags, sid2, sid3) = if version >= 2 && data.len() >= 0x7C { (be16(data, 0x76), data[0x7A], if version >= 3 && data.len() > 0x7B { data[0x7B] } else { 0 }) } else { (0, 0, 0) };
    let info = SidInfo {
        title: text(&data[0x16..0x36]), author: text(&data[0x36..0x56]), released: text(&data[0x56..0x76]),
        songs: be16(data, 0x0E).max(1), start_song: be16(data, 0x10).max(1), rsid: data.starts_with(b"RSID"), ntsc: (flags >> 2) & 3 == 2,
        load: be16(data, 8), init: be16(data, 0x0A), play: be16(data, 0x0C), speed: u32::from_be_bytes([data[0x12], data[0x13], data[0x14], data[0x15]]),
        sids: 1 + (sid2 != 0) as usize + (sid3 != 0) as usize,
    };
    Ok((info, off))
}

struct Machine { ram: Vec<u8>, cycles: u64, sid: [u8; 32], timer_latch: u16 }
impl Bus for Machine {
    fn read(&mut self, a: u16) -> u8 {
        let line = (self.cycles / 63) % 312;
        match a {
            0xD012 => line as u8,
            0xD011 => (self.ram[0xD011] & 0x7F) | if line >= 256 { 0x80 } else { 0 },
            0xD019 => 0x81,
            0xD41B => (self.cycles.wrapping_mul(2654435761) >> 8) as u8,
            0xD41C => 0,
            0xDC00 | 0xDC01 | 0xDD00 => 0xFF,
            0xDC0D | 0xDD0D => 0x81,
            _ => self.ram[a as usize],
        }
    }
    fn write(&mut self, a: u16, v: u8) {
        if (0xD400..0xD800).contains(&a) { self.sid[(a & 0x1F) as usize] = v; }
        else if a == 0xDC04 { self.timer_latch = self.timer_latch & 0xFF00 | v as u16; }
        else if a == 0xDC05 { self.timer_latch = self.timer_latch & 0x00FF | (v as u16) << 8; }
        self.ram[a as usize] = v;
    }
}

/// Unterprogramm bei `addr` ausführen, bis es zurückkehrt (RTS zur Marke, RTI oder Sprung in die Standard-IRQ-Routine des KERNALs)
fn call(cpu: &mut Cpu, m: &mut Machine, addr: u16, a: u8, as_irq: bool, cap: u64) -> Result<(), String> {
    cpu.a = a; cpu.x = 0; cpu.y = 0; cpu.sp = 0xFD; cpu.i = false;
    if as_irq { cpu.pc = SENTINEL; cpu.irq(m, addr); }
    else { let r = SENTINEL.wrapping_sub(1); cpu.push(m, (r >> 8) as u8); cpu.push(m, r as u8); cpu.pc = addr; }
    let start = m.cycles;
    while cpu.pc != SENTINEL {
        if matches!(cpu.pc, 0xEA31 | 0xEA7E | 0xEA81 | 0xFF48 | 0xFFD2) { break; }
        let c = cpu.step(m)?; m.cycles += c as u64;
        if m.cycles - start > cap { return Err(format!("Zeitüberschreitung bei ${:04X} (Routine kehrt nicht zurück)", addr)); }
    }
    Ok(())
}

/// Spielt `seconds` Sekunden des Subtunes und liefert je Bild den Registerstand von $D400–$D418
fn record(data: &[u8], i: &SidInfo, off: usize, song: u16, seconds: u32, warn: &mut Vec<String>) -> Result<(Vec<[u8; 25]>, f64, f64), String> {
    let mut m = Machine { ram: vec![0; 65536], cycles: 0, sid: [0; 32], timer_latch: 0 };
    let mut payload = &data[off..];
    let load = if i.load == 0 { if payload.len() < 2 { return Err("SID-Datei ohne Daten".into()); } let l = payload[0] as u16 | (payload[1] as u16) << 8; payload = &payload[2..]; l } else { i.load };
    if load as usize + payload.len() > 0x10000 { return Err(format!("Daten ab ${load:04X} passen nicht in den Speicher")); }
    m.ram[load as usize..load as usize + payload.len()].copy_from_slice(payload);
    m.ram[0] = 0x2F; m.ram[1] = 0x37; m.ram[0x314] = 0x31; m.ram[0x315] = 0xEA; m.ram[0x316] = 0x66; m.ram[0x317] = 0xFE; m.ram[0xFFFE] = 0x48; m.ram[0xFFFF] = 0xFF;
    let (clock, hz) = if i.ntsc { (NTSC_CLOCK, 60.0) } else { (PAL_CLOCK, 50.0) };
    let mut cpu = Cpu::new();
    let init = if i.init == 0 { load } else { i.init };
    call(&mut cpu, &mut m, init, (song - 1) as u8, false, 20_000_000).map_err(|e| format!("Init: {e}"))?;
    // Aufrufe der Spielroutine pro Bild: Takt aus CIA 1 (Speed-Bit des Subtunes), sonst einmal pro Bild
    let cia = (i.speed >> ((song - 1).min(31))) & 1 == 1;
    let calls = if cia && m.timer_latch > 0 { ((clock / (m.timer_latch as f64 + 1.0)) / hz).round().clamp(1.0, 8.0) as usize } else { 1 };
    if calls > 1 { warn.push(format!("Mehrfachtempo: die Spielroutine läuft {calls}× pro Bild; aufgenommen wird der Stand am Bildende")); }
    let (play, as_irq) = if i.play != 0 { (i.play, false) } else {
        let v = m.ram[0x314] as u16 | (m.ram[0x315] as u16) << 8;
        if v != 0xEA31 && v != 0 { (v, true) } else { (m.ram[0xFFFE] as u16 | (m.ram[0xFFFF] as u16) << 8, true) }
    };
    if i.sids > 1 { warn.push(format!("{} SID-Chips: nur der erste wird aufgenommen", i.sids)); }
    let frames = (seconds as f64 * hz) as usize;
    let per_frame = (clock / hz) as u64; let mut out = Vec::with_capacity(frames);
    for _ in 0..frames {
        let t0 = m.cycles;
        for _ in 0..calls { call(&mut cpu, &mut m, play, 0, as_irq, 400_000).map_err(|e| format!("Spielroutine: {e}"))?; }
        let mut regs = [0u8; 25]; regs.copy_from_slice(&m.sid[..25]); out.push(regs);
        if m.cycles - t0 < per_frame { m.cycles = t0 + per_frame; }
    }
    Ok((out, clock, hz))
}

fn note_of(freq: u16, clock: f64) -> Option<u8> {
    if freq == 0 { return None; }
    let hz = freq as f64 * clock / 16_777_216.0;
    Some((58.0 + 12.0 * (hz / 440.0).log2()).round().clamp(1.0, 96.0) as u8)
}

pub fn import(data: &[u8], p: &Profile, name: &str, o: &SidOpts) -> Result<(Value, Vec<String>), String> {
    let (i, off) = info(data)?;
    let mut warn = vec![];
    if i.rsid { warn.push("RSID-Datei: sie erwartet einen echten C64 (Zeitgeber, KERNAL); die Aufnahme kann unvollständig sein".into()); }
    let song = if o.song == 0 { i.start_song } else { o.song };
    if song == 0 || song > i.songs { return Err(format!("Subtune {song} gibt es nicht (1..{})", i.songs)); }
    let seconds = o.seconds.clamp(1, 600);
    let (regs, clock, hz) = record(data, &i, off, song, seconds, &mut warn)?;
    let nf = regs.len();
    if i.ntsc { warn.push("NTSC-Stück (60 Bilder/s): im Tracker die Bildrate auf 60 Hz stellen".into()); }
    // --- je Stimme: hörbares Gate, Tonhöhe
    let audible = |f: usize, v: usize| { let c = regs[f][v * 7 + 4]; c & 1 != 0 && c & 0xF0 != 0 && c & 8 == 0 };
    let freq = |f: usize, v: usize| regs[f][v * 7] as u16 | (regs[f][v * 7 + 1] as u16) << 8;
    let mut used_fx = false;
    for f in 0..nf { for v in 0..3 { let c = regs[f][v * 7 + 4]; if audible(f, v) && c & 6 != 0 { used_fx = true; } } if regs[f][0x17] & 0x0F != 0 && regs[f][0x18] & 0x70 != 0 { used_fx = true; } }
    if used_fx { warn.push("Filter, Ringmodulation und Synchronisation werden nicht übernommen".into()); }
    // --- Zeilenraster: größter Takt (Bilder je Zeile), auf dem fast alle Noteneinsätze liegen
    let mut onsets: Vec<usize> = vec![];
    for v in 0..3 { for f in 0..nf { if audible(f, v) && (f == 0 || !audible(f - 1, v)) { onsets.push(f); } } }
    if onsets.is_empty() { return Err("In der Aufnahme sind keine Töne zu hören (Gate nie gesetzt). Bei RSID-Dateien oder besonderen Abspielroutinen kann die Aufnahme scheitern.".into()); }
    let (mut speed, mut shift) = (6usize, 0usize);
    if onsets.len() >= 4 {
        for s in (1..=12usize).rev() {
            let mut cnt = vec![0usize; s]; for &f in &onsets { cnt[f % s] += 1; }
            let (ph, best) = cnt.iter().enumerate().max_by_key(|&(_, c)| *c).map(|(k, c)| (k, *c)).unwrap();
            if best as f64 >= 0.9 * onsets.len() as f64 { speed = s; shift = ph; break; }
        }
    }
    let rows = (nf.saturating_sub(shift) + speed - 1) / speed;
    // --- Noten und Instrumente
    let mut keys: HashMap<(u8, u8, u8, u16), usize> = HashMap::new(); let mut instruments: Vec<Value> = vec![]; let mut combined = false; let mut too_many = false;
    let mut cells = vec![vec![(0u8, INS_KEEP); rows]; 3];
    for v in 0..3 {
        let mut cur: Option<(u8, u8)> = None;                                   // (Note, Instrument) der laufenden Note
        for r in 0..rows {
            let (a, b) = (shift + r * speed, (shift + (r + 1) * speed).min(nf)); if a >= b { break; }
            let rising = (a..b).find(|&f| audible(f, v) && (f == 0 || !audible(f - 1, v)));
            let falling = (a..b).any(|f| !audible(f, v) && f > 0 && audible(f - 1, v));
            if let Some(f) = rising {
                let c = regs[f][v * 7 + 4]; let wave_bits = c & 0xF0;
                if wave_bits.count_ones() > 1 { combined = true; }
                let wave = if wave_bits & 0x80 != 0 { "noise" } else if wave_bits & 0x40 != 0 { "pulse" } else if wave_bits & 0x20 != 0 { "saw" } else { "triangle" };
                let pw = (regs[f][v * 7 + 2] as u16 | ((regs[f][v * 7 + 3] & 0x0F) as u16) << 8) as u16;
                let pwq = if wave == "pulse" { if pw == 0 { 2048 } else { ((pw + 128) / 256 * 256).min(4095) } } else { 2048 };
                let (ad, sr) = (regs[f][v * 7 + 5], regs[f][v * 7 + 6]);
                let key = (wave_bits, ad, sr, pwq);
                let idx = match keys.get(&key) { Some(&k) => k, None => {
                    if instruments.len() >= 255 { too_many = true; 254 } else {
                        let k = instruments.len(); keys.insert(key, k);
                        instruments.push(json!({ "name": format!("SID {k:02X} {wave}"), "wave": wave, "pw": pwq, "a": ad >> 4, "d": ad & 15, "s": sr >> 4, "r": sr & 15 })); k
                    } } };
                let fr = freq(f, v); let n = note_of(if fr == 0 { freq(b - 1, v) } else { fr }, clock);
                if let Some(n) = n { cells[v][r] = (n, idx as u8); cur = Some((n, idx as u8)); }
            } else if falling && !audible(b - 1, v) {
                if cur.is_some() { cells[v][r] = (NOTE_OFF, INS_KEEP); cur = None; }
            } else if audible(b - 1, v) {
                if let (Some((n0, ins)), Some(n)) = (cur, note_of(freq(b - 1, v), clock)) { if (n as i32 - n0 as i32).abs() >= 2 { cells[v][r] = (n, ins); cur = Some((n, ins)); } }
            }
        }
    }
    if combined { warn.push("Kombinierte Wellenformen wurden auf eine einzelne Wellenform abgebildet".into()); }
    if too_many { warn.push("Mehr als 255 verschiedene Instrumente: weitere Klänge nutzen das letzte Instrument".into()); }
    // Läuft die Musik am Ende der Aufnahme noch?
    let tail = nf.saturating_sub((hz as usize) * 2);
    if (tail..nf).any(|f| (0..3).any(|v| audible(f, v))) { warn.push(format!("Das Stück läuft über die aufgenommenen {seconds} s hinaus (Länge im Dialog erhöhen, falls mehr gewünscht)")); }
    // --- Patterns
    let used = (0..rows).rev().find(|&r| (0..3).any(|v| cells[v][r].0 != 0)).map_or(1, |r| r + 1);
    let n = (p.raw["rows"].as_u64().unwrap_or(32) as usize).clamp(1, 128);
    let blocks = (used + n - 1) / n;
    let mut patterns: Vec<Value> = vec![]; let mut seen: HashMap<Vec<String>, usize> = HashMap::new(); let mut order = vec![];
    for blk in 0..blocks {
        let rws: Vec<Vec<String>> = (0..n).map(|k| { let r = blk * n + k; (0..3).map(|v| if r < rows && r < used { let (nt, ins) = cells[v][r]; format_cell(nt, ins) } else { "--- --".to_string() }).collect() }).collect();
        let key: Vec<String> = rws.iter().map(|r| r.join("|")).collect();
        let id = *seen.entry(key).or_insert_with(|| { patterns.push(json!({ "rows": rws })); patterns.len() - 1 });
        order.push(id);
    }
    if instruments.is_empty() { instruments.push(json!({ "name": "Puls", "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 })); }
    warn.insert(0, format!("Aus „{}“ ({}) aufgenommen: Subtune {song}, {seconds} s, {} Noteneinsätze, Takt {speed} Bilder/Zeile. Das ist eine Annäherung an den Klang, nicht der Originalsong.", i.title, i.author, onsets.len()));
    Ok((json!({ "format": 1, "kind": "song", "profile": p.id, "name": name, "speed": speed.min(31), "rows": n, "instruments": instruments, "patterns": patterns, "order": order, "loop": 0 }), warn))
}
