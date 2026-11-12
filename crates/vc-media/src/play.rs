// SPDX-License-Identifier: GPL-3.0-or-later
//! PLAY-Tracker (`kind: "play"`): Noten-Raster für den BASIC-65-Befehl `PLAY`; Export als lauffähiges BASIC-Programm
//! (`TEMPO`, `ENVELOPE`, je Stimme eine oder mehrere `PLAY`-Anweisungen).
//!
//! Ein Schritt = Sechzehntel. Eine Note klingt bis zur nächsten Note oder `===` (Pause); leere Zellen (`--- --`) halten sie.
//! Längen werden in Ganze/Halbe/Viertel/Achtel/Sechzehntel (mit Punkt) zerlegt; was sich nicht in einer Note ausdrücken lässt, wird
//! als wiederholter Anschlag derselben Tonhöhe ausgegeben.
use crate::song::{parse_cell, INS_KEEP};
use crate::Profile;
use serde_json::{json, Value};

const LETTERS: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
/// Länge in Schritten → (Dauerbuchstabe, Punkt)
const LENS: [(usize, char, bool); 8] = [(16, 'W', false), (12, 'H', true), (8, 'H', false), (6, 'Q', true), (4, 'Q', false), (3, 'I', true), (2, 'I', false), (1, 'S', false)];

pub fn new_play(p: &Profile, name: &str) -> Value {
    let (v, rows) = (p.raw["voices"].as_u64().unwrap_or(6) as usize, p.raw["rows"].as_u64().unwrap_or(32) as usize);
    json!({ "format": 1, "kind": "play", "profile": p.id, "name": name, "tempo": 8, "volume": 8, "rows": rows, "voices": v,
        "instruments": [
            { "name": "Dreieck", "wave": "triangle", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4 },
            { "name": "Puls", "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 6, "r": 4 },
            { "name": "Saege", "wave": "saw", "pw": 2048, "a": 0, "d": 6, "s": 10, "r": 2 },
            { "name": "Rauschen", "wave": "noise", "pw": 2048, "a": 0, "d": 4, "s": 0, "r": 3 } ],
        "steps": (0..rows).map(|_| (0..v).map(|_| "--- --").collect::<Vec<_>>()).collect::<Vec<_>>() })
}

fn wave_no(w: &str) -> Result<u8, String> {
    Ok(match w { "triangle" => 0, "saw" => 1, "pulse" => 2, "noise" => 3, o => return Err(format!("Wellenform '{o}' unbekannt (triangle, saw, pulse, noise)")) })
}

enum Ev { Note(u8, u8, usize), Rest(usize) }

/// Ereignisse einer Stimme (Note, Instrument, Länge in Schritten)
fn events(steps: &[Vec<(u8, u8)>], v: usize) -> Vec<Ev> {
    let mut out: Vec<Ev> = vec![]; let mut ins = 0u8;
    for row in steps {
        let (n, i) = row[v];
        if n != 0 && n != crate::song::NOTE_OFF {
            if i != INS_KEEP { ins = i; }
            out.push(Ev::Note(n, ins, 1));
        } else if n == crate::song::NOTE_OFF { out.push(Ev::Rest(1)); }
        else if let Some(last) = out.last_mut() { match last { Ev::Note(_, _, l) | Ev::Rest(l) => *l += 1 } }
        else { out.push(Ev::Rest(1)); }
    }
    // Pausen am Ende entfallen, damit die Stimme nicht unnötig lange "wartet"
    while matches!(out.last(), Some(Ev::Rest(_))) { out.pop(); }
    out
}

/// BASIC-Programm (nummeriert, klassische Schreibweise) zum Media
pub fn to_basic(root: &Value, name: &str) -> Result<String, String> {
    let voices = root["voices"].as_u64().unwrap_or(6) as usize;
    if voices == 0 || voices > 6 { return Err("PLAY kennt 1–6 Stimmen".into()); }
    let ins = root["instruments"].as_array().ok_or("instruments fehlt")?;
    if ins.len() > 10 { return Err(format!("PLAY kennt Hüllkurven 0–9, der Song hat {} Instrumente", ins.len())); }
    let steps: Vec<Vec<(u8, u8)>> = root["steps"].as_array().ok_or("steps fehlt")?.iter().enumerate().map(|(r, row)| {
        let cells = row.as_array().ok_or(format!("Zeile {r}: Array erwartet"))?;
        if cells.len() != voices { return Err(format!("Zeile {r}: {} statt {voices} Zellen", cells.len())); }
        cells.iter().enumerate().map(|(c, x)| parse_cell(x.as_str().unwrap_or("")).map_err(|e| format!("Zeile {r}, Stimme {}: {e}", c + 1))).collect::<Result<Vec<_>, _>>()
    }).collect::<Result<_, _>>()?;
    let mut ln = 10u32; let mut s = String::new();
    let mut push = |t: String| { s.push_str(&format!("{ln} {t}\n")); ln += 10; };
    push(format!("REM {} - erzeugt von VintageCoder", name.to_uppercase()));
    push(format!("VOL {}", root["volume"].as_u64().unwrap_or(8).min(15)));
    push(format!("TEMPO {}", root["tempo"].as_u64().unwrap_or(8).clamp(1, 255)));
    for (i, e) in ins.iter().enumerate() {
        let g = |k: &str, d: u64| e[k].as_u64().unwrap_or(d).min(15);
        push(format!("ENVELOPE {i},{},{},{},{},{},{}", g("a", 0), g("d", 9), g("s", 8), g("r", 4), wave_no(e["wave"].as_str().unwrap_or("triangle"))?, e["pw"].as_u64().unwrap_or(2048).min(4095)));
    }
    for v in 0..voices {
        let evs = events(&steps, v);
        if evs.is_empty() { continue; }
        let head = |ins: i32| format!("V{}{}", v + 1, if ins >= 0 { format!("T{ins}") } else { String::new() });
        let (mut cur, mut o_cur, mut t_cur, mut d_cur) = (String::new(), -1i32, -1i32, ' ');
        let mut lines: Vec<String> = vec![];
        let flush = |cur: &mut String, lines: &mut Vec<String>, o: &mut i32, t: &mut i32, d: &mut char| { if !cur.is_empty() { lines.push(std::mem::take(cur)); } *o = -1; *t = -1; *d = ' '; };
        for ev in &evs {
            let (len, note) = match ev { Ev::Note(n, i, l) => (*l, Some((*n, *i))), Ev::Rest(l) => (*l, None) };
            let mut left = len;
            while left > 0 {
                let &(sz, ch, dot) = LENS.iter().find(|x| x.0 <= left).unwrap();
                left -= sz;
                if cur.len() > 150 { flush(&mut cur, &mut lines, &mut o_cur, &mut t_cur, &mut d_cur); }
                if cur.is_empty() { cur.push_str(&head(-1)); }
                let tok = match note {
                    Some((n, i)) => {
                        let k = (n - 1) as usize; let (oct, pc) = ((k / 12) as i32, LETTERS[k % 12]);
                        if oct > 6 { return Err(format!("Stimme {}: Oktave {oct} - PLAY kennt nur 0–6", v + 1)); }
                        if i as usize >= ins.len() { return Err(format!("Stimme {}: Instrument {i} existiert nicht", v + 1)); }
                        let mut t = String::new();
                        if t_cur != i as i32 { t.push_str(&format!("T{i}")); t_cur = i as i32; }
                        if o_cur != oct { t.push_str(&format!("O{oct}")); o_cur = oct; }
                        if d_cur != ch { t.push(ch); d_cur = ch; }
                        t.push_str(pc); if dot { t.push('.'); } t
                    }
                    None => { let mut t = String::new(); if d_cur != ch { t.push(ch); d_cur = ch; } t.push('R'); if dot { t.push('.'); } t }
                };
                cur.push_str(&tok);
            }
        }
        flush(&mut cur, &mut lines, &mut o_cur, &mut t_cur, &mut d_cur);
        for l in lines { push(format!("PLAY \"{l}\"")); }
    }
    Ok(s)
}
