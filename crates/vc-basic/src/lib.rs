// SPDX-License-Identifier: GPL-3.0-or-later
//! VintageCoder BASIC-Werkzeuge: Dialekte, klassischer Tokenizer, Detokenizer (Phase 2).
pub mod atari;
pub mod b2a;
mod classic;
mod detok;
mod dialect;
pub mod longnames;
mod plain;
mod structured;
mod structured_atari;
pub mod petscii;

pub use classic::{tokenize_line, tokenize_text};
pub use detok::{build_prg, line_text, line_text_spaced, parse_prg, to_classic_text, to_structured_text, Parsed};
pub use dialect::Dialect;
pub use plain::{auto_line_numbers, expand_includes, include_directive, number_lines, remap_diagnostics, LineMap, Numbered, PlainOptions};
pub use structured::{compile, expanded_source, find_blobs, BlobReq, Compiled, Options};
pub use structured_atari::{compile_atari, AtariCompiled, AtariOptions};

/// Liest die Familie (`cbm` oder `atari`) einer Dialektdatei.
pub fn dialect_family(data_dir: &std::path::Path, id: &str) -> Result<String, String> {
    let path = data_dir.join("basic").join(format!("{id}.dialect.json"));
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(v["family"].as_str().unwrap_or("cbm").to_string())
}

/// Ergebnis des Übersetzens von natives BASIC (CBM): `.prg` samt Label-Tabelle und Zeilenanzahl
/// `names`: Zuordnung langer Namen → Kurznamen (leer ohne `#longnames`), `warnings`: Hinweise der Umsetzung
pub struct PlainCbm { pub prg: Vec<u8>, pub labels: Vec<(String, u32)>, pub listing: String, pub names: Vec<longnames::NameEntry>, pub warnings: Vec<vc_core::Diagnostic> }
pub struct PlainAtari { pub file: Vec<u8>, pub labels: Vec<(String, u32)>, pub listing: String, pub names: Vec<longnames::NameEntry>, pub warnings: Vec<vc_core::Diagnostic> }

/// Setzt bei `#longnames` die langen Namen um (nach Nummerierung und Label-Auflösung); sonst bleibt der Text unverändert.
pub fn apply_long_names(n: Numbered, t: longnames::Target, file: &str) -> Result<longnames::Renamed, Vec<vc_core::Diagnostic>> {
    if n.long_names { longnames::rename(&n.text, t, file) } else { Ok(longnames::Renamed { text: n.text, names: vec![], warnings: vec![] }) }
}

/// Natives CBM-BASIC (Zeilennummern und/oder `@label`) → PRG
pub fn compile_plain_cbm(src: &str, d: &Dialect, o: &PlainOptions, load: u16) -> Result<PlainCbm, Vec<vc_core::Diagnostic>> {
    let n = number_lines(src, o)?;
    let labels = n.labels.clone();
    let r = apply_long_names(n, longnames::Target::Cbm(d), &o.file)?;
    let lines = tokenize_text(&r.text, d, &o.file)?;
    let prg = build_prg(&lines, load);
    let listing = to_classic_text(&Parsed { load_addr: load, lines }, d);
    Ok(PlainCbm { prg, labels, listing, names: r.names, warnings: r.warnings })
}

/// Natives Atari-BASIC (Zeilennummern und/oder `@label`) → `.BAS`-Datei im SAVE-Format
pub fn compile_plain_atari(src: &str, d: &atari::AtariDialect, o: &PlainOptions) -> Result<PlainAtari, Vec<vc_core::Diagnostic>> {
    let n = number_lines(src, o)?;
    let labels = n.labels.clone();
    let r = apply_long_names(n, longnames::Target::Atari(d), &o.file)?;
    let p = atari::tokenize_text(&r.text, d, &o.file)?;
    Ok(PlainAtari { file: p.save_file(d), labels, listing: p.listing(d), names: r.names, warnings: r.warnings })
}
