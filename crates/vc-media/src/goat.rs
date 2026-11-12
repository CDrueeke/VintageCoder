// SPDX-License-Identifier: GPL-3.0-or-later
//! GoatTracker-2-Import (`*.sng`, Kennung GTS3/GTS4/GTS5) → SID-Song.
//!
//! Übernommen werden Songaufbau (Orderlisten mit Wiederholung und Transponierung), Noten, Pausen, Note-aus, Instrumentnummern,
//! ADSR, erste Wellenform und Pulsbreite der Instrumente sowie das Tempo. Nicht übernommen werden Wellen-/Puls-/Filtertabellen,
//! Vibrato und die Patternbefehle (Portamento, Vibrato, Filter …); dazu gibt es Hinweise.
//!
//! GoatTracker spielt die drei Stimmen mit eigenen Orderlisten ab. Hier werden sie zu einer gemeinsamen Zeitachse ausgerollt und in
//! gleich lange Blöcke geschnitten; gleiche Blöcke werden zu einem Pattern zusammengefasst.
use crate::song::{format_cell, INS_KEEP, NOTE_OFF};
use crate::Profile;
use serde_json::{json, Value};
use std::collections::HashMap;

const FIRST_NOTE: u8 = 0x60;
const LAST_NOTE: u8 = 0xBC;
#[allow(dead_code)]
const REST: u8 = 0xBD;
const KEY_OFF: u8 = 0xBE;
const PAT_END: u8 = 0xFF;
const MAX_ROWS: usize = 1 << 16;

struct Ins { ad: u8, sr: u8, wave: u8, pulse: u8, filter: u8, vib: u8, name: String }
struct Gt { orders: Vec<Vec<u8>>, ins: Vec<Ins>, tables: [(Vec<u8>, Vec<u8>); 4], pats: Vec<Vec<[u8; 4]>>, subtunes: usize, channels: usize }

struct Rd<'a> { d: &'a [u8], p: usize }
impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.p + n > self.d.len() { return Err("Datei zu kurz".into()); }
        let s = &self.d[self.p..self.p + n]; self.p += n; Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> { Ok(self.take(1)?[0]) }
}

/// Liest die Datei mit `chn` Orderlisten je Subtune (3; Stereo-Dateien haben 6). Ist danach nicht genau das Dateiende erreicht, gilt die Annahme als falsch.
fn parse(data: &[u8], chn: usize) -> Result<Gt, String> {
    let mut r = Rd { d: data, p: 0 };
    let id = r.take(4)?;
    if !matches!(id, b"GTS3" | b"GTS4" | b"GTS5") {
        return Err(if matches!(id, b"GTS2" | b"GTS!") { "alte GoatTracker-Version (GTS2/GTS!): bitte in GoatTracker 2.x laden und neu speichern".into() } else { "keine GoatTracker-Datei (erwartet GTS5)".into() });
    }
    r.take(96)?;                                     // Titel, Autor, Copyright
    let subtunes = r.u8()? as usize;
    if subtunes == 0 || subtunes > 32 { return Err(format!("Unplausible Subtune-Zahl {subtunes}")); }
    let mut orders = vec![];
    for s in 0..subtunes { for c in 0..chn { let n = r.u8()? as usize + 1; let o = r.take(n)?.to_vec(); if s == 0 { orders.push(o); } let _ = c; } }
    let n_ins = r.u8()? as usize;
    if n_ins > 63 { return Err(format!("Unplausible Instrumentzahl {n_ins}")); }
    let mut ins = vec![];
    for _ in 0..n_ins {
        let b = r.take(25)?;
        let name: String = b[9..25].iter().take_while(|&&c| c != 0).map(|&c| if (32..127).contains(&c) { c as char } else { '?' }).collect();
        ins.push(Ins { ad: b[0], sr: b[1], wave: b[2], pulse: b[3], filter: b[4], vib: b[5], name: name.trim().to_string() });
    }
    let mut tables: [(Vec<u8>, Vec<u8>); 4] = Default::default();
    for t in tables.iter_mut() { let n = r.u8()? as usize; let l = r.take(n)?.to_vec(); let rr = r.take(n)?.to_vec(); *t = (l, rr); }
    let n_pat = r.u8()? as usize;
    let mut pats = vec![];
    for _ in 0..n_pat {
        let m = r.u8()? as usize; let b = r.take(m * 4)?;
        pats.push(b.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect());
    }
    if r.p != data.len() { return Err("Restdaten nach den Patterns".into()); }
    Ok(Gt { orders, ins, tables, pats, subtunes, channels: chn })
}

struct Row { note: u8, ins: u8, cmd: u8, data: u8 }

fn gcd(a: usize, b: usize) -> usize { if b == 0 { a } else { gcd(b, a % b) } }

const CMD_NAMES: [&str; 15] = ["", "Portamento aufwärts", "Portamento abwärts", "Tonportamento", "Vibrato", "AD setzen", "SR setzen", "Wellenform setzen", "Wellentabellen-Zeiger",
    "Pulstabellen-Zeiger", "Filtertabellen-Zeiger", "Filtersteuerung", "Filter-Grenzfrequenz", "Hauptlautstärke", "Funktempo"];

pub fn import(data: &[u8], p: &Profile, name: &str) -> Result<(Value, Vec<String>), String> {
    let mut warn = vec![];
    let gt = match parse(data, 3) {
        Ok(g) => g,
        Err(e3) => match parse(data, 6) { Ok(g) => g, Err(_) => return Err(e3) },
    };
    if gt.channels == 6 { warn.push("Stereo-Song (2 SIDs): nur die ersten drei Stimmen werden übernommen".into()); }
    if gt.subtunes > 1 { warn.push(format!("{} Subtunes: nur der erste wird übernommen", gt.subtunes)); }
    let eff = |pat: &Vec<[u8; 4]>| pat.iter().position(|r| r[0] == PAT_END).unwrap_or(pat.len());
    // --- Zeitachse je Stimme
    let (mut tracks, mut loops, mut clamped, mut missing) = (vec![], vec![], 0usize, false);
    let mut lens: Vec<usize> = vec![];
    for ch in 0..3 {
        let ol = gt.orders.get(ch).ok_or("Orderliste fehlt")?;
        let (mut tl, mut starts, mut tr, mut rep, mut i) = (Vec::<Row>::new(), vec![0usize; ol.len()], 0i32, 1usize, 0usize);
        let mut restart = 0usize;
        while i < ol.len() {
            starts[i] = tl.len(); let b = ol[i];
            if b == 0xFF { restart = ol.get(i + 1).copied().unwrap_or(0) as usize; break; }
            if b >= 0xE0 { tr = b as i32 - 0xF0; }
            else if b >= 0xD0 { rep = (b & 0x0F) as usize + 1; }
            else {
                match gt.pats.get(b as usize) {
                    None => { missing = true; }
                    Some(pat) => {
                        let n = eff(pat);
                        if n > 0 { lens.push(n); }
                        for _ in 0..rep {
                            for r in &pat[..n] {
                                let note = match r[0] { x @ FIRST_NOTE..=LAST_NOTE => { let v = (x - FIRST_NOTE) as i32 + 1 + tr; if !(1..=96).contains(&v) { clamped += 1; } v.clamp(1, 96) as u8 } KEY_OFF => NOTE_OFF, _ => 0 };
                                tl.push(Row { note, ins: if r[1] == 0 { INS_KEEP } else { r[1] - 1 }, cmd: r[2], data: r[3] });
                            }
                            if tl.len() > MAX_ROWS { return Err("Song zu lang (über 65536 Zeilen)".into()); }
                        }
                    }
                }
                rep = 1;
            }
            i += 1;
        }
        loops.push(starts.get(restart).copied().unwrap_or(0));
        tracks.push(tl);
    }
    if missing { warn.push("Orderliste verweist auf ein nicht vorhandenes Pattern (übersprungen)".into()); }
    if clamped > 0 { warn.push(format!("{clamped} Noten lagen nach der Transponierung außerhalb des Bereichs und wurden begrenzt")); }
    let total = tracks.iter().map(|t| t.len()).max().unwrap_or(0);
    if total == 0 { return Err("Song enthält keine Zeilen".into()); }
    if tracks.iter().any(|t| t.len() != total) { warn.push("Die Stimmen sind unterschiedlich lang: kürzere werden mit Pausen aufgefüllt (GoatTracker lässt sie unabhängig wiederholen)".into()); }
    if loops.iter().any(|&l| l != loops[0]) { warn.push("Die Stimmen haben verschiedene Wiederholungspunkte: es gilt der der ersten Stimme".into()); }
    let loop_row = loops[0];
    // --- Tempo und Befehle
    let mut speed: Option<(usize, u8)> = None; let mut cmds: HashMap<u8, usize> = HashMap::new(); let mut chan_tempo = false;
    for t in &tracks { for (i, r) in t.iter().enumerate() {
        if r.cmd == 0xF { if (3..=0x7F).contains(&r.data) { if speed.map_or(true, |(k, _)| i < k) { speed = Some((i, r.data)); } } else { chan_tempo = true; } }
        else if r.cmd != 0 { *cmds.entry(r.cmd).or_default() += 1; }
    } }
    let mut speed = speed.map_or(6, |(_, d)| d as u64);
    if speed > 31 { warn.push(format!("Tempo {speed} liegt über dem Tracker-Maximum 31 und wurde auf 31 gesetzt")); speed = 31; }
    if chan_tempo { warn.push("Kanalspezifische Tempi und Funktempo-Rückruf (FXY mit $00/$01/$80+) werden nicht übernommen".into()); }
    let mut keys: Vec<_> = cmds.into_iter().collect(); keys.sort();
    for (c, n) in keys { warn.push(format!("Befehl {c:X}XY ({}) in {n} Zeilen nicht übernommen", CMD_NAMES.get(c as usize).copied().unwrap_or("?"))); }
    // --- Instrumente
    let (wt, pt) = (&gt.tables[0], &gt.tables[1]);
    let (mut multi, mut pmod, mut filt, mut vib, mut combined) = (vec![], vec![], vec![], vec![], vec![]);
    let mut instruments = vec![];
    for (i, x) in gt.ins.iter().enumerate() {
        let (mut wave, mut pw) = ("pulse", 2048u64);
        if x.wave > 0 && (x.wave as usize) <= wt.0.len() {
            let start = x.wave as usize - 1;
            let steps = wt.0[start..].iter().take_while(|&&l| l != 0xFF).count();
            let first = wt.0[start..].iter().take_while(|&&l| l != 0xFF).find(|&&l| (0x10..0xE0).contains(&l)).copied();
            if let Some(w) = first {
                let bits = w & 0xF0;
                if bits.count_ones() > 1 { combined.push(i + 1); }
                wave = if bits & 0x80 != 0 { "noise" } else if bits & 0x40 != 0 { "pulse" } else if bits & 0x20 != 0 { "saw" } else { "triangle" };
            }
            if steps > 1 { multi.push(i + 1); }
        }
        if x.pulse > 0 && (x.pulse as usize) <= pt.0.len() {
            let k = x.pulse as usize - 1; let (l, rr) = (pt.0[k], pt.1[k]);
            if l >= 0x80 && l != 0xFF { pw = ((l as u64 & 0x0F) << 8) | rr as u64; }
            if (1..0x80).contains(&l) || pt.0[k + 1..].first().map_or(false, |&n| n != 0xFF && n != 0) { pmod.push(i + 1); }
        }
        if x.filter != 0 { filt.push(i + 1); }
        if x.vib != 0 { vib.push(i + 1); }
        let nm = if x.name.is_empty() { format!("Instrument {:02X}", i + 1) } else { x.name.clone() };
        instruments.push(json!({ "name": nm, "wave": wave, "pw": pw.min(4095), "a": x.ad >> 4, "d": x.ad & 15, "s": x.sr >> 4, "r": x.sr & 15 }));
    }
    let list = |v: &Vec<usize>| v.iter().map(|n| format!("{n:02X}")).collect::<Vec<_>>().join(", ");
    if !multi.is_empty() { warn.push(format!("Wellentabelle (Arpeggio, Wellenformwechsel): nur der erste Eintrag zählt – Instrument {}", list(&multi))); }
    if !combined.is_empty() { warn.push(format!("Kombinierte Wellenform auf eine einzelne Wellenform abgebildet – Instrument {}", list(&combined))); }
    if !pmod.is_empty() { warn.push(format!("Pulsbreiten-Modulation wird nicht übernommen – Instrument {}", list(&pmod))); }
    if !filt.is_empty() { warn.push(format!("Filtertabelle wird nicht übernommen – Instrument {}", list(&filt))); }
    if !vib.is_empty() { warn.push(format!("Vibrato wird nicht übernommen – Instrument {}", list(&vib))); }
    if instruments.is_empty() { instruments.push(json!({ "name": "Puls", "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 })); }
    let max_ins = instruments.len() as u8;
    let mut bad_ins = false;
    for t in tracks.iter_mut() { for r in t.iter_mut() { if r.ins != INS_KEEP && r.ins >= max_ins { r.ins = INS_KEEP; bad_ins = true; } } }
    if bad_ins { warn.push("Zeilen mit nicht vorhandenem Instrument behalten das vorherige Instrument".into()); }
    // --- Blocklänge: größter gemeinsamer Teiler aller Patternlängen (und des Wiederholungspunkts)
    let mut n = lens.iter().copied().fold(0, gcd);
    if loop_row > 0 { n = gcd(n, loop_row); }
    if n == 0 { n = 16; }
    while n > 128 { n = if n % 2 == 0 { n / 2 } else { 128 }; }
    if n < 4 { warn.push(format!("Patternlängen haben nur den gemeinsamen Teiler {n}: es wird mit 16 Zeilen je Pattern gearbeitet")); n = 16; }
    let blocks = (total + n - 1) / n;
    let loop_block = loop_row / n;
    if loop_row % n != 0 { warn.push("Der Wiederholungspunkt liegt nicht auf einer Patterngrenze und wurde nach vorn gerückt".into()); }
    let mut patterns: Vec<Value> = vec![]; let mut seen: HashMap<Vec<String>, usize> = HashMap::new(); let mut order = vec![];
    for b in 0..blocks {
        let mut rows: Vec<Vec<String>> = vec![];
        for k in 0..n {
            let idx = b * n + k;
            rows.push(tracks.iter().map(|t| t.get(idx).map_or("--- --".to_string(), |r| format_cell(r.note, r.ins))).collect());
        }
        let key: Vec<String> = rows.iter().map(|r| r.join("|")).collect();
        let id = *seen.entry(key).or_insert_with(|| { patterns.push(json!({ "rows": rows })); patterns.len() - 1 });
        order.push(id);
    }
    let root = json!({ "format": 1, "kind": "song", "profile": p.id, "name": name, "speed": speed, "rows": n, "instruments": instruments, "patterns": patterns, "order": order, "loop": loop_block.min(blocks - 1) });
    Ok((root, warn))
}
