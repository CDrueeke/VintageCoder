// SPDX-License-Identifier: GPL-3.0-or-later
//! Maschinencode im nativen BASIC (Konzept „BASIC-Editor und Dialekte“, 6.3, Entscheidung B14): Direktive
//! `#asm "datei.asm" [with "song.asm"]` und Marken des Codes im Programmtext als `asm.name` (wird beim Bauen durch die Dezimaladresse ersetzt).
//!
//! Der Kern hier ist **assemblerfrei**: Der Aufrufer reicht eine Funktion herein, die den Code übersetzt (in der IDE und in `vcc` ist das
//! `vc_ide::blob::assemble_blobs`, die Technik des strukturierten BASIC). Die Platzierung:
//! * [`Placement::Behind`] (Commodore): der Code liegt **hinter dem BASIC-Text** im PRG. Seine Adresse hängt von der Länge des Programms ab,
//!   und diese von den eingesetzten Zahlen; darum wird wiederholt, bis die Adresse stabil ist (höchstens acht Durchgänge).
//! * [`Placement::Data`] (MEGA65, Atari): der Code kommt als **DATA-Zeilen** ab dem Label `@asm_data` (Blöcke `adresse,anzahl,werte…`, Ende `0`) und wird
//!   vom Programm selbst an die feste Adresse geladen. Atari: die Adresse liegt unter `$A000` an einer Seitengrenze.
//!
//! Dem Programm stehen außer den Marken des Codes `asm.base_addr` (Anfang des Codes) und `asm.base_page` (Seite, für `POKE 106`) zur Verfügung.
use crate::structured::{parse_blob, BlobReq};
use crate::{compile_plain_atari, compile_plain_cbm, PlainAtari, PlainCbm, PlainOptions};
use vc_core::{msg, Diagnostic};

/// Ergebnis des Assemblierens
#[derive(Debug, Clone, Default)]
pub struct Assembled {
    /// Bytes ab der Basisadresse (bei `Behind` der Anhang)
    pub bytes: Vec<u8>,
    /// Marken des Codes (Name klein, Wert)
    pub symbols: Vec<(String, i64)>,
    /// Blöcke (Adresse, Bytes) für DATA-Zeilen
    pub chunks: Vec<(u16, Vec<u8>)>,
}

/// Übersetzt die Blobs ab `addr`; `strict`: der Code muss genau bei `addr` beginnen
pub type AsmFn<'a> = dyn FnMut(&[BlobReq], usize, bool) -> Result<Assembled, Vec<Diagnostic>> + 'a;

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    /// hinter dem BASIC-Text (Commodore)
    Behind,
    /// als DATA-Zeilen; `Some(adresse)` fest (MEGA65), `None`: unter `$A000` an einer Seitengrenze (Atari)
    Data(Option<usize>),
}

/// Ergebnis mit Maschinencode
pub struct Blobbed<T> {
    pub built: T,
    /// Bytes hinter dem BASIC-Text (nur `Behind`; sie sind in `built.prg` schon enthalten)
    pub behind: Vec<u8>,
    pub base: usize,
    pub size: usize,
    pub symbols: Vec<(String, i64)>,
}

/// Zeile `#asm …`?
pub fn is_blob_directive(line: &str) -> bool {
    let t = line.trim();
    t.get(..4).map_or(false, |h| h.eq_ignore_ascii_case("#asm")) && t[4..].starts_with(|c: char| c.is_whitespace() || c == '"')
}

/// Zeile mit dem früheren Namen `#blob …`? (gibt es nicht mehr: Fehler `B177` mit dem Hinweis auf `#asm`)
pub fn is_old_blob_directive(line: &str) -> bool {
    let t = line.trim();
    t.get(..5).map_or(false, |h| h.eq_ignore_ascii_case("#blob")) && t[5..].starts_with(|c: char| c.is_whitespace() || c == '"')
}

/// Sucht die `#asm`-Zeilen, ohne etwas zu übersetzen
pub fn find_blobs(src: &str, file: &str) -> Result<Vec<BlobReq>, Vec<Diagnostic>> {
    let (mut out, mut errs) = (vec![], vec![]);
    for (i, l) in src.lines().enumerate() {
        if !is_blob_directive(l) { continue; }
        match parse_blob(&format!("blob{}", &l.trim()[4..]), (file.to_string(), i + 1)) {
            Some(b) => out.push(b),
            None => errs.push(Diagnostic::error("B177", file, i + 1, msg!("diag-basic-b177-blob-syntax"))),
        }
    }
    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

/// Namen hinter `asm.` im Programmtext (klein, ohne Wiederholung), außer in Zeichenketten, nach `REM` und in `DATA`
pub fn asm_refs(src: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for l in src.lines() {
        let t = l.trim_start();
        if t.starts_with(';') || t.starts_with('#') { continue; }
        let ch: Vec<char> = l.chars().collect();
        let (mut i, mut quote) = (0, false);
        while i < ch.len() {
            let c = ch[i];
            if c == '"' { quote = !quote; i += 1; continue; }
            if quote { i += 1; continue; }
            if c.is_ascii_alphabetic() || c == '_' {
                let mut j = i;
                while j < ch.len() && (ch[j].is_ascii_alphanumeric() || ch[j] == '_') { j += 1; }
                let w: String = ch[i..j].iter().collect::<String>().to_ascii_uppercase();
                if w == "REM" { break; }
                if w == "DATA" { while j < ch.len() && ch[j] != ':' { j += 1; } i = j; continue; }
                if w == "ASM" && ch.get(j) == Some(&'.') && i.checked_sub(1).map_or(true, |p| !ch[p].is_ascii_alphanumeric()) {
                    let mut k = j + 1;
                    while k < ch.len() && (ch[k].is_ascii_alphanumeric() || ch[k] == '_') { k += 1; }
                    let name: String = ch[j + 1..k].iter().collect::<String>().to_ascii_lowercase();
                    if !name.is_empty() && !out.contains(&name) { out.push(name); }
                    i = k; continue;
                }
                i = j; continue;
            }
            i += 1;
        }
    }
    out
}

/// Quelle ohne `#asm`-Zeilen (gleiche Zeilenzahl: die Zeilen werden leer)
fn strip(src: &str) -> String {
    let mut t = src.lines().map(|l| if is_blob_directive(l) { "" } else { l }).collect::<Vec<_>>().join("\n");
    if src.ends_with('\n') { t.push('\n'); }
    t
}

fn data_block(chunks: &[(u16, Vec<u8>)]) -> String {
    let mut s = String::new();
    let mut first = true;
    let mut put = |t: String, s: &mut String| { if first { s.push_str("@asm_data: "); first = false; } s.push_str("DATA "); s.push_str(&t); s.push('\n'); };
    for (addr, bytes) in chunks {
        put(format!("{addr},{}", bytes.len()), &mut s);
        for row in bytes.chunks(16) { put(row.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","), &mut s); }
    }
    put("0".into(), &mut s);
    s
}

/// Setzt die Marken als Namen für `asm.…`; fehlt eine Marke, die der Text braucht, gibt es `B177`
fn with_symbols(o: &PlainOptions, refs: &[String], syms: &[(String, i64)], base: usize, first: &BlobReq, strict: bool) -> Result<PlainOptions, Vec<Diagnostic>> {
    let mut o2 = o.clone();
    let mut all: Vec<(String, i64)> = syms.to_vec();
    all.push(("base_addr".into(), base as i64));
    all.push(("base_page".into(), (base >> 8) as i64));
    for r in refs {
        match all.iter().find(|(n, _)| n == r) {
            Some((_, v)) => o2.defines.push((format!("asm.{r}"), v.to_string())),
            None if strict => return Err(vec![Diagnostic::error("B177", &first.src.0, first.src.1, msg!("diag-basic-b177-symbol-missing", name = r.as_str()))]),
            None => o2.defines.push((format!("asm.{r}"), "0".into())),
        }
    }
    Ok(o2)
}

/// Prüft, dass die Datenzeilen im Nummernmodus noch Platz haben (`B173`)
fn check_range(text: &str, lines: usize, o: &PlainOptions, at: &BlobReq) -> Result<(), Vec<Diagnostic>> {
    let last = text.lines().filter_map(|l| { let t = l.trim_start(); let d: String = t.chars().take_while(char::is_ascii_digit).collect(); d.parse::<u32>().ok() }).max();
    if let Some(l) = last { if l as u64 + o.step as u64 * lines as u64 > o.max_line as u64 { return Err(vec![Diagnostic::error("B173", &at.src.0, at.src.1, msg!("diag-basic-b173-blob-range", last = l, lines = lines, max = o.max_line))]); } }
    Ok(())
}

/// Natives CBM-BASIC mit `#asm` → PRG (siehe Moduldoku)
pub fn compile_plain_cbm_blobs(src: &str, d: &crate::Dialect, o: &PlainOptions, load: u16, place: Placement, asm: &mut AsmFn) -> Result<Blobbed<PlainCbm>, Vec<Diagnostic>> {
    let reqs = find_blobs(src, &o.file)?;
    let text = strip(src);
    if reqs.is_empty() { return compile_plain_cbm(&text, d, o, load).map(|built| Blobbed { built, behind: vec![], base: 0, size: 0, symbols: vec![] }); }
    let refs = asm_refs(&text);
    match place {
        Placement::Behind => {
            let first = compile_plain_cbm(&text, d, &with_symbols(o, &refs, &[], 0, &reqs[0], false)?, load)?;
            let mut addr = load as usize + first.prg.len() - 2;
            for _ in 0..8 {
                let a = asm(&reqs, addr, true)?;
                let mut c = compile_plain_cbm(&text, d, &with_symbols(o, &refs, &a.symbols, addr, &reqs[0], true)?, load)?;
                let end = load as usize + c.prg.len() - 2;
                if end == addr {
                    if end + a.bytes.len() > 0xFFFF { return Err(vec![Diagnostic::error("B177", &reqs[0].src.0, reqs[0].src.1, msg!("diag-basic-b177-too-large", end = format!("{:05X}", end + a.bytes.len())))]); }
                    c.prg.extend_from_slice(&a.bytes);
                    return Ok(Blobbed { size: a.bytes.len(), behind: a.bytes, base: addr, symbols: a.symbols, built: c });
                }
                addr = end;
            }
            Err(vec![Diagnostic::error("B177", &reqs[0].src.0, reqs[0].src.1, msg!("diag-basic-b177-address-unstable"))])
        }
        Placement::Data(base) => {
            let base = base.unwrap_or(0xC000);
            let a = asm(&reqs, base, false)?;
            let block = data_block(&a.chunks);
            check_range(&text, block.lines().count(), o, &reqs[0])?;
            let full = format!("{}{}{}", text, if text.ends_with('\n') || text.is_empty() { "" } else { "\n" }, block);
            let built = compile_plain_cbm(&full, d, &with_symbols(o, &refs, &a.symbols, base, &reqs[0], true)?, load)?;
            Ok(Blobbed { size: a.chunks.iter().map(|c| c.1.len()).sum(), behind: vec![], base, symbols: a.symbols, built })
        }
    }
}

/// Natives Atari-BASIC mit `#asm`: der Code kommt als DATA-Zeilen (`Placement::Data(None)`: unter `$A000`, an einer Seitengrenze)
pub fn compile_plain_atari_blobs(src: &str, d: &crate::atari::AtariDialect, o: &PlainOptions, asm: &mut AsmFn) -> Result<Blobbed<PlainAtari>, Vec<Diagnostic>> {
    let reqs = find_blobs(src, &o.file)?;
    let text = strip(src);
    if reqs.is_empty() { return compile_plain_atari(&text, d, o).map(|built| Blobbed { built, behind: vec![], base: 0, size: 0, symbols: vec![] }); }
    let refs = asm_refs(&text);
    let probe = asm(&reqs, 0x6000, false)?;
    let base = (0xA000usize.saturating_sub(probe.bytes.len())) & 0xFF00;
    if base < 0x4000 { return Err(vec![Diagnostic::error("B177", &reqs[0].src.0, reqs[0].src.1, msg!("diag-basic-b177-code-too-large", bytes = probe.bytes.len()))]); }
    let a = asm(&reqs, base, false)?;
    let block = data_block(&a.chunks);
    check_range(&text, block.lines().count(), o, &reqs[0])?;
    let full = format!("{}{}{}", text, if text.ends_with('\n') || text.is_empty() { "" } else { "\n" }, block);
    let built = compile_plain_atari(&full, d, &with_symbols(o, &refs, &a.symbols, base, &reqs[0], true)?)?;
    Ok(Blobbed { size: a.chunks.iter().map(|c| c.1.len()).sum(), behind: vec![], base, symbols: a.symbols, built })
}
