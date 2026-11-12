// SPDX-License-Identifier: GPL-3.0-or-later
//! Strukturiertes BASIC für Atari BASIC: Textlowering (Labels, const, Blöcke) → nummeriertes Listing → Tokenizer.
//! Atari BASIC kennt lange Variablennamen; diese bleiben unverändert (nur Großbuchstaben und Ziffern erlaubt).
use crate::atari::{tokenize_text, AtariDialect, AtariProgram};
use crate::structured::{expand, BlobReq, Env, first_word, is_word, proc_head, split_args};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use vc_core::Diagnostic;

#[derive(Clone)]
pub struct AtariOptions {
    pub file: String,
    pub start: u32,
    pub step: u32,
    pub consts: Vec<(String, i64)>,
    pub include_dirs: Vec<PathBuf>,
    pub system: Option<String>,
    /// Maschinencode (Adresse, Bytes), den das Programm als DATA-Zeilen ab `@blob_data` mitbringt; Format je Block: `adresse, anzahl, werte…`, Ende: `0`
    pub blob_chunks: Vec<(u16, Vec<u8>)>,
}

impl AtariOptions {
    pub fn new(file: impl Into<String>) -> Self { AtariOptions { file: file.into(), start: 10, step: 10, consts: vec![], include_dirs: vec![], system: None, blob_chunks: vec![] } }
}

pub struct AtariCompiled {
    pub blobs: Vec<BlobReq>,
    pub program: AtariProgram,
    /// Inhalt der `.BAS`-Datei (wie von SAVE geschrieben).
    pub file: Vec<u8>,
    pub listing: String,
    pub labels: BTreeMap<String, u32>,
}

enum Block {
    If { els: String, end: String, has_else: bool, at: (String, usize) },
    While { top: String, end: String, at: (String, usize) },
    Repeat { top: String, end: String, at: (String, usize) },
    Do { top: String, end: String, at: (String, usize) },
    Proc { skip: String, at: (String, usize) },
}

struct GLine { src: (String, usize), labels: Vec<String>, text: String }

const MARK: char = '\u{1}';

/// Wandelt eine Anweisungszeile um: Kommentare entfernen, Konstanten ersetzen, `$hex`/`%bin` → dezimal, `@label` → Platzhalter.
fn transform(s: &str, consts: &HashMap<String, String>) -> Result<String, String> {
    let ch: Vec<char> = s.chars().collect();
    let (mut out, mut i) = (String::new(), 0);
    while i < ch.len() {
        let c = ch[i];
        if c == '"' {
            let mut j = i + 1;
            while j < ch.len() && ch[j] != '"' { j += 1; }
            out.extend(&ch[i..(j + 1).min(ch.len())]);
            i = j + 1;
        } else if c == '\'' { break; }
        else if c == '@' {
            let st = i + 1;
            i = st;
            while i < ch.len() && is_word(ch[i]) { i += 1; }
            if st == i { return Err("Label-Name nach '@' fehlt".into()); }
            out.push(MARK);
            out.push_str(&ch[st..i].iter().collect::<String>().to_ascii_lowercase());
            out.push(MARK);
        } else if c == '$' && ch.get(i + 1).map_or(false, char::is_ascii_hexdigit) {
            let st = i + 1;
            i = st;
            while i < ch.len() && ch[i].is_ascii_hexdigit() { i += 1; }
            out.push_str(&i64::from_str_radix(&ch[st..i].iter().collect::<String>(), 16).map_err(|e| e.to_string())?.to_string());
        } else if c == '%' && ch.get(i + 1).map_or(false, |x| *x == '0' || *x == '1') {
            let st = i + 1;
            i = st;
            while i < ch.len() && (ch[i] == '0' || ch[i] == '1') { i += 1; }
            out.push_str(&i64::from_str_radix(&ch[st..i].iter().collect::<String>(), 2).map_err(|e| e.to_string())?.to_string());
        } else if c.is_ascii_alphabetic() {
            let st = i;
            while i < ch.len() && is_word(ch[i]) { i += 1; }
            if ch.get(i) == Some(&'$') { i += 1; }
            let word: String = ch[st..i].iter().collect();
            let low = word.to_ascii_lowercase();
            if low == "rem" || low == "data" { out.push_str(&word); out.extend(&ch[i..]); return Ok(out); }
            match consts.get(&low) { Some(v) => out.push_str(v), None => out.push_str(&word) }
        } else { out.push(c); i += 1; }
    }
    Ok(out.trim().to_string())
}

pub fn compile_atari(src: &str, d: &AtariDialect, opts: &AtariOptions) -> Result<AtariCompiled, Vec<Diagnostic>> {
    let mut errs = vec![];
    let mut lines_src = vec![];
    expand(src, &opts.file, &opts.include_dirs, 0, &mut lines_src, &mut errs, &Env::new(opts.system.as_deref().or(Some("atari8")), &opts.consts));
    let mut consts: HashMap<String, String> = opts.consts.iter().map(|(n, v)| (n.to_ascii_lowercase(), v.to_string())).collect();
    let (mut glines, mut pending, mut blocks): (Vec<GLine>, Vec<String>, Vec<Block>) = (vec![], vec![], vec![]);
    let mut counter = 0;
    let mut used_ends: HashSet<String> = HashSet::new();
    let mut blobs: Vec<BlobReq> = vec![];
    let mut procs: HashMap<String, Vec<String>> = HashMap::new();
    for s in &lines_src {
        let t = s.text.trim();
        if first_word(t) == "proc" {
            match proc_head(&t[4..]) { Ok((n, ps)) => { procs.insert(n, ps); } Err(e) => errs.push(Diagnostic::error("B109", &s.file, s.line, e)) }
        }
    }
    for s in &lines_src {
        let mut t = s.text.trim();
        let at = (s.file.clone(), s.line);
        if t.is_empty() || t.starts_with('\'') { continue; }
        while t.starts_with('@') {
            let name: String = t[1..].chars().take_while(|c| is_word(*c)).collect();
            let rest = t[1 + name.len()..].trim_start();
            if name.is_empty() || !rest.starts_with(':') { break; }
            pending.push(name.to_ascii_lowercase());
            t = rest[1..].trim_start();
        }
        if t.is_empty() || t.starts_with('\'') { continue; }
        let word = first_word(t);
        let tr = |x: &str, errs: &mut Vec<Diagnostic>| -> Option<String> { match transform(x, &consts) { Ok(v) => Some(v), Err(e) => { errs.push(Diagnostic::error("B100", &at.0, at.1, e)); None } } };
        let mut emit = |text: String, pending: &mut Vec<String>| glines.push(GLine { src: at.clone(), labels: std::mem::take(pending), text });
        let low = t.to_ascii_lowercase();
        match word.as_str() {
            "const" => {
                let Some((n, v)) = t[5..].trim().split_once('=') else { errs.push(Diagnostic::error("B108", &at.0, at.1, "const erwartet NAME = Wert")); continue };
                if let Some(v) = tr(v.trim(), &mut errs) {
                    if v.is_empty() || v.contains(MARK) || (!v.starts_with('"') && v.parse::<f64>().is_err()) { errs.push(Diagnostic::error("B108", &at.0, at.1, "const erwartet eine einzelne Zahl oder Zeichenkette")); }
                    else { consts.insert(n.trim().to_ascii_lowercase(), v); }
                }
            }
            "else" if low == "else" => match blocks.last_mut() {
                Some(Block::If { els, end, has_else, .. }) if !*has_else => { *has_else = true; emit(format!("GOTO {MARK}{end}{MARK}"), &mut pending); pending.push(els.clone()); }
                _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "else ohne passendes mehrzeiliges if")),
            },
            "endif" if low == "endif" => match blocks.pop() {
                Some(Block::If { els, end, has_else, .. }) => { if !has_else { pending.push(els); } pending.push(end); }
                _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "endif ohne if")),
            },
            "while" => if let Some(c) = tr(t[5..].trim(), &mut errs) {
                counter += 1;
                let (top, end) = (format!("__w{counter}_top"), format!("__w{counter}_end"));
                pending.push(top.clone());
                emit(format!("IF NOT ({c}) THEN {MARK}{end}{MARK}"), &mut pending);
                blocks.push(Block::While { top, end, at: at.clone() });
            },
            "wend" if low == "wend" => match blocks.pop() {
                Some(Block::While { top, end, .. }) => { emit(format!("GOTO {MARK}{top}{MARK}"), &mut pending); pending.push(end); }
                _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "wend ohne while")),
            },
            "repeat" if low == "repeat" => { counter += 1; let (top, end) = (format!("__r{counter}_top"), format!("__r{counter}_end")); pending.push(top.clone()); blocks.push(Block::Repeat { top, end, at: at.clone() }); }
            "until" => match (blocks.pop(), tr(t[5..].trim(), &mut errs)) {
                (Some(Block::Repeat { top, end, .. }), Some(c)) => { emit(format!("IF NOT ({c}) THEN {MARK}{top}{MARK}"), &mut pending); if used_ends.contains(&end) { pending.push(end); } }
                (None, _) | (Some(_), _) => errs.push(Diagnostic::error("B106", &at.0, at.1, "until ohne repeat")),
            },
            "blob" => {
                let strs: Vec<&str> = t[4..].split('"').enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, x)| x).collect();
                let ok = t[4..].trim_start().starts_with('"') && !strs.is_empty() && (strs.len() == 1 || t[4..].to_ascii_lowercase().contains(" with "));
                if ok { blobs.push(BlobReq { file: strs[0].to_string(), with: strs[1..].iter().map(|x| x.to_string()).collect(), src: at.clone() }); }
                else { errs.push(Diagnostic::error("B113", &at.0, at.1, "blob erwartet: blob \"datei.asm\" [with \"song.asm\"]")); }
            }
            "elseif" => match blocks.last_mut() {
                Some(Block::If { els, end, has_else, .. }) if !*has_else => {
                    let cond = t[6..].trim();
                    let cond = if cond.to_ascii_lowercase().ends_with(" then") { &cond[..cond.len() - 5] } else { cond };
                    if let Some(c) = tr(cond, &mut errs) {
                        counter += 1;
                        let next_els = format!("__i{counter}_else");
                        emit(format!("GOTO {MARK}{end}{MARK}"), &mut pending);
                        pending.push(std::mem::replace(els, next_els.clone()));
                        emit(format!("IF NOT ({c}) THEN {MARK}{next_els}{MARK}"), &mut pending);
                    }
                }
                _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "elseif ohne passendes mehrzeiliges if (oder nach else)")),
            },
            "do" => {
                let rest = t[2..].trim();
                let (kind, cond) = (first_word(rest), rest[first_word(rest).len()..].trim());
                counter += 1;
                let (top, end) = (format!("__d{counter}_top"), format!("__d{counter}_end"));
                pending.push(top.clone());
                match kind.as_str() {
                    "" => {}
                    "while" => if let Some(c) = tr(cond, &mut errs) { used_ends.insert(end.clone()); emit(format!("IF NOT ({c}) THEN {MARK}{end}{MARK}"), &mut pending); },
                    "until" => if let Some(c) = tr(cond, &mut errs) { used_ends.insert(end.clone()); emit(format!("IF {c} THEN {MARK}{end}{MARK}"), &mut pending); },
                    _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "do erwartet nichts, 'while Bedingung' oder 'until Bedingung'")),
                }
                blocks.push(Block::Do { top, end, at: at.clone() });
            }
            "loop" => match blocks.pop() {
                Some(Block::Do { top, end, .. }) => {
                    let rest = t[4..].trim();
                    let (kind, cond) = (first_word(rest), rest[first_word(rest).len()..].trim());
                    match kind.as_str() {
                        "" => emit(format!("GOTO {MARK}{top}{MARK}"), &mut pending),
                        "while" => if let Some(c) = tr(cond, &mut errs) { emit(format!("IF {c} THEN {MARK}{top}{MARK}"), &mut pending); },
                        "until" => if let Some(c) = tr(cond, &mut errs) { emit(format!("IF NOT ({c}) THEN {MARK}{top}{MARK}"), &mut pending); },
                        _ => errs.push(Diagnostic::error("B106", &at.0, at.1, "loop erwartet nichts, 'while Bedingung' oder 'until Bedingung'")),
                    }
                    if used_ends.contains(&end) { pending.push(end); }
                }
                other => { if let Some(b) = other { blocks.push(b); } errs.push(Diagnostic::error("B106", &at.0, at.1, "loop ohne do")); }
            },
            "exit" if low == "exit" => {
                let target = blocks.iter().rev().take_while(|b| !matches!(b, Block::Proc { .. }))
                    .find_map(|b| match b { Block::While { end, .. } | Block::Repeat { end, .. } | Block::Do { end, .. } => Some(end.clone()), _ => None });
                match target {
                    Some(end) => { used_ends.insert(end.clone()); emit(format!("GOTO {MARK}{end}{MARK}"), &mut pending); }
                    None => errs.push(Diagnostic::error("B106", &at.0, at.1, "exit außerhalb von while, repeat oder do")),
                }
            }
            "proc" => match proc_head(&t[4..]) {
                Ok((name, _)) => {
                    if blocks.iter().any(|b| matches!(b, Block::Proc { .. })) { errs.push(Diagnostic::error("B109", &at.0, at.1, "proc darf nicht in einer anderen proc stehen")); continue; }
                    counter += 1;
                    let skip = format!("__p{counter}_skip");
                    emit(format!("GOTO {MARK}{skip}{MARK}"), &mut pending);
                    pending.push(name);
                    blocks.push(Block::Proc { skip, at: at.clone() });
                }
                Err(e) => errs.push(Diagnostic::error("B109", &at.0, at.1, e)),
            },
            "endproc" if low == "endproc" => match blocks.pop() {
                Some(Block::Proc { skip, .. }) => { emit("RETURN".into(), &mut pending); pending.push(skip); }
                other => { if let Some(b) = other { blocks.push(b); } errs.push(Diagnostic::error("B106", &at.0, at.1, "endproc ohne proc")); }
            },
            "call" => {
                let rest = t[4..].trim();
                let name: String = rest.chars().take_while(|c| is_word(*c)).collect::<String>().to_ascii_lowercase();
                if name.is_empty() { errs.push(Diagnostic::error("B109", &at.0, at.1, "Prozedurname fehlt")); continue; }
                let Some(params) = procs.get(&name) else { errs.push(Diagnostic::error("B109", &at.0, at.1, format!("Prozedur '{name}' ist nicht definiert"))); continue };
                let tail = rest[name.len()..].trim();
                let args = if tail.is_empty() { vec![] } else { split_args(tail.strip_prefix('(').and_then(|x| x.strip_suffix(')')).unwrap_or(tail)) };
                if args.len() != params.len() { errs.push(Diagnostic::error("B109", &at.0, at.1, format!("Prozedur '{name}' erwartet {} Argument(e), übergeben: {}", params.len(), args.len()))); continue; }
                let mut text = String::new();
                for (p, a) in params.iter().zip(&args) { if let Some(v) = tr(a, &mut errs) { text.push_str(&format!("{p}={v}:")); } }
                text.push_str(&format!("GOSUB {MARK}{name}{MARK}"));
                emit(text, &mut pending);
            }
            _ => {
                let Some(text) = tr(t, &mut errs) else { continue };
                if word == "if" && text.to_ascii_lowercase().ends_with(" then") && text.len() > 7 {
                    counter += 1;
                    let (els, end) = (format!("__i{counter}_else"), format!("__i{counter}_end"));
                    let cond = &text[2..text.len() - 5];
                    emit(format!("IF NOT ({}) THEN {MARK}{els}{MARK}", cond.trim()), &mut pending);
                    blocks.push(Block::If { els, end, has_else: false, at: at.clone() });
                } else { emit(text, &mut pending); }
            }
        }
    }
    for b in &blocks {
        let at = match b { Block::If { at, .. } | Block::While { at, .. } | Block::Repeat { at, .. } | Block::Do { at, .. } | Block::Proc { at, .. } => at };
        errs.push(Diagnostic::error("B106", &at.0, at.1, "Block nicht geschlossen"));
    }
    if !opts.blob_chunks.is_empty() {
        // Maschinencode als DATA-Zeilen hinten anhängen (der Ablauf kommt nie hierher; `restore @blob_data` + `read` holt sie)
        let at = glines.last().map(|l| l.src.clone()).unwrap_or((opts.file.clone(), 1));
        pending.push("blob_data".into());
        let mut put = |text: String, pending: &mut Vec<String>| glines.push(GLine { src: at.clone(), labels: std::mem::take(pending), text });
        for (addr, bytes) in &opts.blob_chunks {
            put(format!("DATA {addr},{}", bytes.len()), &mut pending);
            for row in bytes.chunks(16) { put(format!("DATA {}", row.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",")), &mut pending); }
        }
        put("DATA 0".into(), &mut pending);
    }
    if !pending.is_empty() {
        let at = glines.last().map(|l| l.src.clone()).unwrap_or((opts.file.clone(), 1));
        glines.push(GLine { src: at, labels: std::mem::take(&mut pending), text: "REM".into() });
    }
    let mut labels: BTreeMap<String, u32> = BTreeMap::new();
    for (i, l) in glines.iter().enumerate() {
        let num = opts.start + i as u32 * opts.step;
        if num > 32767 { errs.push(Diagnostic::error("B105", &l.src.0, l.src.1, "Zeilennummern über 32767 – step verkleinern")); break; }
        for n in &l.labels { if labels.insert(n.clone(), num).is_some() { errs.push(Diagnostic::error("B104", &l.src.0, l.src.1, format!("Label '{n}' doppelt definiert"))); } }
    }
    let mut text = String::new();
    for (i, l) in glines.iter().enumerate() {
        let mut line = String::new();
        let mut parts = l.text.split(MARK);
        line.push_str(parts.next().unwrap_or(""));
        while let (Some(name), Some(rest)) = (parts.next(), parts.next()) {
            match labels.get(name) { Some(n) => line.push_str(&n.to_string()), None if name.starts_with("__") => {}, None => errs.push(Diagnostic::error("B107", &l.src.0, l.src.1, format!("Label '@{name}' nicht definiert"))) }
            line.push_str(rest);
        }
        text.push_str(&format!("{} {}\n", opts.start + i as u32 * opts.step, line));
    }
    if !errs.is_empty() { return Err(errs); }
    match tokenize_text(&text, d, &opts.file) {
        Ok(program) => {
            let file = program.save_file(d);
            let listing = program.listing(d);
            Ok(AtariCompiled { blobs, program, file, listing, labels })
        }
        Err(es) => Err(es.into_iter().map(|e| { let g = &glines[e.line.saturating_sub(1).min(glines.len().saturating_sub(1))]; Diagnostic::error(e.code, &g.src.0, g.src.1, e.message) }).collect()),
    }
}
