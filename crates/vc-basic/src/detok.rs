// SPDX-License-Identifier: GPL-3.0-or-later
//! Detokenizer: PRG → Zeilen → lesbarer Text (klassisch oder strukturiert mit Labels).
use crate::dialect::Dialect;
use crate::petscii;
use std::collections::BTreeSet;

pub struct Parsed { pub load_addr: u16, pub lines: Vec<(u16, Vec<u8>)> }

/// Liest die verkettete Zeilenliste eines BASIC-PRG.
pub fn parse_prg(data: &[u8]) -> Result<Parsed, String> {
    if data.len() < 4 { return Err("Datei zu kurz".into()); }
    let load = data[0] as u16 | (data[1] as u16) << 8;
    let body = &data[2..];
    let (mut off, mut lines) = (0usize, vec![]);
    loop {
        if off + 2 > body.len() { return Err("Programmende ($0000) fehlt".into()); }
        let next = body[off] as usize | (body[off + 1] as usize) << 8;
        if next == 0 { break; }
        if off + 5 > body.len() { return Err("abgeschnittene Zeile".into()); }
        let num = body[off + 2] as u16 | (body[off + 3] as u16) << 8;
        let end = body[off + 4..].iter().position(|b| *b == 0).ok_or("Zeilenende (0) fehlt")? + off + 4;
        lines.push((num, body[off + 4..end].to_vec()));
        let expect = load as usize + end + 1;
        if next != expect { return Err(format!("Link-Zeiger in Zeile {num} stimmt nicht (${next:04X}, erwartet ${expect:04X})")); }
        off = end + 1;
    }
    Ok(Parsed { load_addr: load, lines })
}

/// Zeilenbytes → Text (Keywords klein, Strings mit `{name}`-Steuercodes).
pub fn line_text(bytes: &[u8], d: &Dialect) -> String { text_of(bytes, d, false) }

/// Wie `line_text`, aber mit Leerzeichen um Schlüsselwörter (für die Struktursprache lesbar und lexbar).
pub fn line_text_spaced(bytes: &[u8], d: &Dialect) -> String { text_of(bytes, d, true) }

fn text_of(bytes: &[u8], d: &Dialect, spaced: bool) -> String {
    let (rem, data) = (d.tok("REM"), d.tok("DATA"));
    let (mut s, mut quote, mut in_data, mut in_rem) = (String::new(), false, false, false);
    let mut idx = 0;
    while idx < bytes.len() {
        let b = bytes[idx]; idx += 1;
        if b == 0x22 { quote = !quote; s.push('"'); continue; }
        if quote || in_rem { s.push_str(&petscii::to_text(b, d)); continue; }
        if in_data { if b == b':' { in_data = false; } s.push_str(&petscii::to_text(b, d)); continue; }
        if let Some((kw, n)) = d.keyword_at(bytes, idx - 1) {
            idx += n - 1;
            let word = kw.chars().next().map_or(false, |c| c.is_ascii_alphabetic());
            if spaced && word && !s.ends_with(' ') && !s.is_empty() { s.push(' '); }
            s.push_str(&kw.to_ascii_lowercase());
            if spaced && word && !kw.ends_with('(') { s.push(' '); }
            if b == rem { in_rem = true; }
            if b == data { in_data = true; }
        } else { s.push_str(&petscii::to_text(b, d)); }
    }
    s
}

pub fn to_classic_text(p: &Parsed, d: &Dialect) -> String {
    p.lines.iter().map(|(n, b)| format!("{n} {}\n", line_text(b, d))).collect()
}

/// Sprungziele (Zeilennummern) aus den Tokens: GOTO/GOSUB/THEN/RESTORE/RUN und ON-Listen.
fn targets(bytes: &[u8], d: &Dialect) -> Vec<(usize, usize, u16)> {
    let jump: Vec<u8> = ["GOTO", "GOSUB", "THEN", "RESTORE", "RUN"].iter().map(|k| d.tok(k)).collect();
    let (rem, data, on) = (d.tok("REM"), d.tok("DATA"), d.tok("ON"));
    let (mut out, mut i, mut quote, mut after_on) = (vec![], 0, false, false);
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0x22 { quote = !quote; i += 1; continue; }
        if quote { i += 1; continue; }
        if b == rem || b == data { break; }
        if b == on { after_on = true; }
        if jump.contains(&b) {
            let mut j = i + 1;
            loop {
                while j < bytes.len() && bytes[j] == b' ' { j += 1; }
                let st = j;
                while j < bytes.len() && bytes[j].is_ascii_digit() { j += 1; }
                if st == j { break; }
                let n: u32 = std::str::from_utf8(&bytes[st..j]).unwrap().parse().unwrap_or(u32::MAX);
                if n <= 63999 { out.push((st, j, n as u16)); }
                while j < bytes.len() && bytes[j] == b' ' { j += 1; }
                if after_on && j < bytes.len() && bytes[j] == b',' { j += 1; } else { break; }
            }
            after_on = false;
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Strukturierter Text: keine Zeilennummern, Sprungziele als `@L<nr>`.
pub fn to_structured_text(p: &Parsed, d: &Dialect) -> String {
    let mut wanted = BTreeSet::new();
    for (_, b) in &p.lines { for (_, _, n) in targets(b, d) { wanted.insert(n); } }
    let mut out = String::from("' Aus PRG zurückgewandelt (vcc basic --detokenize --structured)\n");
    for (num, bytes) in &p.lines {
        if wanted.contains(num) { out.push_str(&format!("@L{num}:\n")); }
        let tg = targets(bytes, d);
        let mut text = String::new();
        let mut last = 0;
        for (s, e, n) in &tg {
            text.push_str(&line_text_spaced(&bytes[last..*s], d));
            text.push_str(&format!("@L{n}"));
            last = *e;
        }
        text.push_str(&line_text_spaced(&bytes[last..], d));
        out.push_str(&format!("{text}\n"));
    }
    out
}

pub fn build_prg(lines: &[(u16, Vec<u8>)], load: u16) -> Vec<u8> {
    let mut v = vec![(load & 0xFF) as u8, (load >> 8) as u8];
    let mut addr = load as usize;
    for (n, b) in lines {
        let next = addr + 2 + 2 + b.len() + 1;
        v.extend([(next & 0xFF) as u8, (next >> 8) as u8, (*n & 0xFF) as u8, (*n >> 8) as u8]);
        v.extend_from_slice(b);
        v.push(0);
        addr = next;
    }
    v.extend([0, 0]);
    v
}
