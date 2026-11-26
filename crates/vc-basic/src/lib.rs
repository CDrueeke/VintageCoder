// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder BASIC-Werkzeuge: Dialekte, klassischer Tokenizer, Detokenizer (Phase 2).
pub mod atari;
pub mod check;
pub mod codes;
pub mod dialects;
pub mod include;
pub mod labels;
pub mod numbering;
pub mod b2a;
pub mod blob;
mod classic;
pub mod compact;
mod detok;
mod dialect;
pub mod longnames;
pub mod numbers;
pub mod numtools;
pub mod mediasize;
mod plain;
pub mod preprocess;
mod structured;
mod structured_atari;
pub mod petscii;

/// Meldung der Zeichentabelle (`vc_core::chars`, noch als fertiger Text) als Meldung des BASIC-Tokenizers
pub(crate) fn char_ref(text: String) -> vc_core::Msg { vc_core::msg!("diag-basic-b100-char-ref", text = text) }

pub use classic::{keyword_spans, tokenize_line, tokenize_text};
pub use detok::{build_prg, line_text, line_text_spaced, parse_prg, to_classic_text, to_structured_text, Parsed};
pub use dialect::Dialect;
pub use plain::{auto_line_numbers, expand_includes, expand_includes_with, include_directive, number_lines, remap_diagnostics, LineMap, Numbered, PlainOptions};
pub use structured::{compile, expanded_source, find_blobs, BlobReq, Compiled, Options};
pub use structured_atari::{compile_atari, AtariCompiled, AtariOptions};

/// Liest die Familie (`cbm` oder `atari`) einer Dialektdatei.
pub fn dialect_family(data_dir: &std::path::Path, id: &str) -> Result<String, String> {
    let v = dialects::load_json(data_dir, id)?;
    Ok(v["family"].as_str().unwrap_or("cbm").to_string())
}

/// Ergebnis des Übersetzens von natives BASIC (CBM): `.prg` samt Label-Tabelle und Zeilenanzahl
/// `names`: Zuordnung langer Namen → Kurznamen (leer ohne `#longnames`), `warnings`: Hinweise der Umsetzung
pub struct PlainCbm { pub prg: Vec<u8>, pub labels: Vec<(String, u32)>, pub listing: String, pub names: Vec<longnames::NameEntry>, pub warnings: Vec<vc_core::Diagnostic>, pub numbers_converted: usize }
pub struct PlainAtari { pub file: Vec<u8>, pub labels: Vec<(String, u32)>, pub listing: String, pub names: Vec<longnames::NameEntry>, pub warnings: Vec<vc_core::Diagnostic>, pub numbers_converted: usize }

/// Setzt bei `#longnames` die langen Namen um (nach Nummerierung und Label-Auflösung); sonst bleibt der Text unverändert.
pub fn apply_long_names(n: Numbered, t: longnames::Target, file: &str) -> Result<longnames::Renamed, Vec<vc_core::Diagnostic>> {
    if n.long_names { longnames::rename(&n.text, t, file) } else { Ok(longnames::Renamed { text: n.text, names: vec![], warnings: vec![] }) }
}

/// Natives CBM-BASIC (Zeilennummern und/oder `@label`) → PRG
pub fn compile_plain_cbm(src: &str, d: &Dialect, o: &PlainOptions, load: u16) -> Result<PlainCbm, Vec<vc_core::Diagnostic>> {
    let keywords = d.tokens.iter().map(|(k, _)| k.trim_end_matches(|c| matches!(c, '(' | '#' | '$')).to_ascii_uppercase()).collect();
    let pre = preprocess::run(src, &o.preprocess(keywords))?;
    let src = pre.text.as_str();
    let o = &o.with_numbering(src)?;
    let mut n = number_lines(src, o)?;
    let conv = numbers::convert_numbers(&n.text, &numbers::Lexicon::Cbm(d), &numbers::NumberOptions { whole_words: n.long_names, ..o.numbers.clone() }, &o.file)?;
    n.text = conv.text;
    let labels = n.labels.clone();
    let mut r = apply_long_names(n, longnames::Target::Cbm(d), &o.file)?;
    r.warnings.extend(conv.warnings);
    r.warnings.extend(check::check_cbm(src, &o.file, d, &o.check));
    if o.compact { r.text = compact::compact_text(&r.text, d); }
    let lines = tokenize_text(&r.text, d, &o.file)?;
    let prg = build_prg(&lines, load);
    r.warnings.extend(mediasize::check_size(prg.len().saturating_sub(2), o.system.as_deref(), &o.file));
    r.warnings.extend(mediasize::check_order(src, o.media, &d.jumps, &d.id, &o.file));
    let listing = to_classic_text(&Parsed { load_addr: load, lines }, d);
    Ok(PlainCbm { prg, labels, listing, names: r.names, warnings: r.warnings, numbers_converted: conv.count })
}

/// Natives Atari-BASIC (Zeilennummern und/oder `@label`) → `.BAS`-Datei im SAVE-Format
pub fn compile_plain_atari(src: &str, d: &atari::AtariDialect, o: &PlainOptions) -> Result<PlainAtari, Vec<vc_core::Diagnostic>> {
    let keywords = d.statements.iter().chain(d.functions.iter()).map(|(k, _)| k.trim_end_matches(|c| matches!(c, '(' | '#' | '$')).to_ascii_uppercase()).collect();
    let pre = preprocess::run(src, &o.preprocess(keywords))?;
    let src = pre.text.as_str();
    let o = &o.with_numbering(src)?;
    let mut n = number_lines(src, o)?;
    let conv = numbers::convert_numbers(&n.text, &numbers::Lexicon::Atari(d), &numbers::NumberOptions { whole_words: n.long_names, ..o.numbers.clone() }, &o.file)?;
    n.text = conv.text;
    let labels = n.labels.clone();
    let mut r = apply_long_names(n, longnames::Target::Atari(d), &o.file)?;
    r.warnings.extend(conv.warnings);
    let p = atari::tokenize_text(&r.text, d, &o.file)?;
    Ok(PlainAtari { file: p.save_file(d), labels, listing: p.listing(d), names: r.names, warnings: r.warnings, numbers_converted: conv.count })
}
