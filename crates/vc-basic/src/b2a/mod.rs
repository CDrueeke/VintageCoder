// SPDX-License-Identifier: GPL-3.0-or-later
//! basic2asm: Konverter von nativem BASIC nach lesbarem Assembler-Quelltext.
//!
//! Stand: Phase 0 (Parser, Syntaxbaum) und Phase 1, 3 und 4 (Stufe 1: Ganzzahl, Stufe 2: Fließkomma über das BASIC-ROM, Stufe 3: Zeichenketten; Ziel C64, Dialekt `cbm_v2`).
pub mod ast;
mod gen;
mod parse;
pub mod print;

use crate::{number_lines, Dialect, PlainOptions};
use vc_core::Diagnostic;

pub use gen::{bcd, mbf, target_info, TargetInfo, TARGETS};
pub use parse::{parse_numbered, parse_numbered_for, parse_numbered_with};

#[derive(Debug, Clone)]
pub struct B2aOptions {
    pub file: String,
    /// Zielsystem: `c64`, `c128`, `plus4` oder `atari8` (legt Ladeadresse, CPU und BASIC-Dialekt fest)
    pub target: String,
    pub start: u32,
    pub step: u32,
    /// höchste zugelassene Stufe (1 = Ganzzahl, 2 = Fließkomma, 3 = Zeichenketten); gewählt wird die kleinste, die das Programm braucht
    pub stage: u8,
    /// Fließkomma-Laufzeit auch für Programme erzwingen, die sie nicht brauchen
    pub force_float: bool,
    /// `/` als ganzzahlige Division zulassen (sonst Stufe 2)
    pub int_div: bool,
    /// Zeilennummer zur Laufzeit mitführen (für Fehlermeldungen)
    pub track_line: bool,
    /// Zeilen, die nur aus `REM` bestehen, nicht übernehmen
    pub no_rem: bool,
    /// Quelle ist Turbo BASIC XL (nur Ziel atari8): IF-Block, ELSE/ENDIF, WHILE/WEND, REPEAT/UNTIL, DO/LOOP/EXIT, PROC/ENDPROC/EXEC, DIV, MOD, DPOKE, DPEEK, PAUSE, MOVE, -MOVE, BPUT, BGET, CLS, HEX$, DEC, INSTR, RAND, INKEY$, TIME$
    pub turbo: bool,
    /// Stil `inline`: einfache Ganzzahlausdrücke als direkte 6502-Befehle statt Laufzeitaufrufen (sonst `compact`)
    pub inline: bool,
    /// Laufzeit in die Ergebnisdatei kopieren (`embed`) statt per `#include` einzubinden; braucht `data_dir`
    pub embed: bool,
    /// Ordner `data` (für `embed`: die Quellen der Laufzeit liegen unter `data/sys/`)
    pub data_dir: Option<std::path::PathBuf>,
}

impl B2aOptions {
    pub fn new(file: impl Into<String>) -> Self { B2aOptions { file: file.into(), target: "c64".into(), start: 10, step: 10, stage: 3, force_float: false, int_div: false, track_line: true, no_rem: false, inline: false, embed: false, turbo: false, data_dir: None } }
}

/// Eintrag der Zeilenkarte: BASIC-Zeile ↔ Quelltextzeile ↔ Label im Assembler
#[derive(Debug, Clone)]
pub struct MapEntry { pub basic_line: u32, pub src_line: usize, pub label: String }

#[derive(Debug, Clone)]
pub struct Converted {
    pub asm: String,
    pub map: Vec<MapEntry>,
    /// eingebundene Laufzeitmodule (Dateien unter `data/sys/common/b2a/`)
    pub modules: Vec<String>,
    pub stage: u8,
    pub warnings: Vec<Diagnostic>,
    /// Zuordnung langer Variablennamen → Kurznamen (`#longnames`), leer sonst
    pub names: Vec<crate::longnames::NameEntry>,
}

impl Converted {
    /// Zeilenkarte als JSON
    pub fn map_json(&self, source: &str) -> String {
        let rows: Vec<String> = self.map.iter().map(|m| format!("    {{\"basic\": {}, \"source\": {}, \"label\": \"{}\"}}", m.basic_line, m.src_line, m.label)).collect();
        let names: Vec<String> = self.names.iter().map(|n| format!("    {{\"long\": {:?}, \"short\": {:?}, \"kind\": {:?}}}", n.long, n.short, n.kind)).collect();
        format!("{{\n  \"source\": {:?},\n  \"lines\": [\n{}\n  ],\n  \"names\": [\n{}\n  ]\n}}\n", source, rows.join(",\n"), names.join(",\n"))
    }
}

type Prepared = (ast::Program, Vec<(String, u32)>, Vec<crate::longnames::NameEntry>, Vec<Diagnostic>);

/// Anweisungen und Funktionen von Atari-BASIC, die das CBM-V2-Vorbild des Parsers nicht kennt und die der Konverter nicht übersetzt (Grafik, Ton, Kanal-Ein-/Ausgabe, Dateien, Joystick)
const ATARI_ONLY: &[&str] = &[
    "LIST", "ENTER", "TRAP", "BYE", "COM", "DOS", "NOTE", "POINT", "POP",
    "STATUS", "XIO", "LPRINT", "CSAVE", "CLOAD", "ADR", "PADDLE", "PTRIG", "CLOG",
];

/// Atari-Wörter, die der Konverter übersetzt (Grafik, Ton, Joystick): sie werden dem CBM-Tokenisierer als zusätzliche Schlüsselwörter bekanntgemacht,
/// damit Umbenennung und Parser sie als Wörter sehen und nicht als Variablen mit CBM-Schlüsselwörtern darin (`COLOR` enthält `OR`)
const ATARI_WORDS: &[&str] = &["GRAPHICS", "COLOR", "PLOT", "DRAWTO", "POSITION", "SETCOLOR", "SOUND", "LOCATE", "STICK", "STRIG", "PUT"];

/// Zusätzliche Wörter von Turbo BASIC XL (nur mit `B2aOptions::turbo`); in Atari-BASIC sind es gewöhnliche Namen
const TURBO_WORDS: &[&str] = &["WHILE", "WEND", "REPEAT", "UNTIL", "DO", "LOOP", "EXIT", "ELSE", "ENDIF", "PROC", "ENDPROC", "EXEC", "DPOKE", "DPEEK", "PAUSE", "MOVE", "MOD", "DIV", "HEX$", "DEC", "INSTR", "RAND", "CLS", "INKEY$", "TIME$", "FCOLOR", "FILLTO", "BPUT", "BGET"];

/// Dialekt des Atari-Parsers: CBM-V2-Tokenisierer mit den Atari-Wörtern, bei Turbo BASIC XL auch mit den Turbo-Wörtern
fn atari_dialect(d: &Dialect, turbo: bool) -> Dialect {
    let a = d.with_extra(ATARI_WORDS, 0xCC);
    if turbo { a.with_extra(TURBO_WORDS, 0xD8) } else { a }
}

/// Atari-BASIC liest der Parser als Teilmenge von CBM V2; Wörter, die nur Atari kennt, werden hier mit einer klaren Meldung abgewiesen
fn atari_check(src: &str, file: &str) -> Vec<Diagnostic> {
    let mut errs = vec![];
    for (n, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with(';') { continue; }
        let b: Vec<char> = t.chars().collect();
        let (mut i, mut in_str) = (0, false);
        // Zeilennummer oder @label: am Anfang überspringen ist nicht nötig: Ziffern und @name sind keine Schlüsselwörter
        while i < b.len() {
            let c = b[i];
            if c == '"' { in_str = !in_str; i += 1; continue; }
            if in_str || !c.is_ascii_alphabetic() { i += 1; continue; }
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') { i += 1; }
            let w: String = b[st..i].iter().collect::<String>().to_ascii_uppercase();
            if w == "REM" || w == "DATA" { break; }
            if st > 0 && b[st - 1] == '@' { continue; }
            if ATARI_ONLY.contains(&w.as_str()) {
                errs.push(Diagnostic::error("E311", file, n + 1, format!("'{w}' ist Atari-BASIC und wird nicht übersetzt (Kanal- und Dateibefehle, XIO, TRAP, NOTE/POINT, PADDLE u. a.)")));
            }
        }
    }
    errs
}

/// `DEG` und `RAD` setzen im Atari-BASIC das Byte `DEGFLG` ($FB): 6 = Grad, 0 = Bogenmaß. Der Konverter schreibt sie in `POKE 251,6` bzw. `POKE 251,0` um
/// (Wörter außerhalb von Zeichenketten, `REM` und `DATA`), die Laufzeit von SIN, COS und ATN liest `$FB` wie das BASIC-Modul.
fn atari_deg_rad(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        let b: Vec<char> = line.chars().collect();
        let (mut i, mut in_str, mut cut) = (0, false, false);
        while i < b.len() {
            let c = b[i];
            if c == '"' { in_str = !in_str; out.push(c); i += 1; continue; }
            if in_str || cut || !c.is_ascii_alphabetic() { out.push(c); i += 1; continue; }
            let st = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') { i += 1; }
            let w: String = b[st..i].iter().collect();
            let up = w.to_ascii_uppercase();
            let after_at = st > 0 && b[st - 1] == '@';
            let prev_alnum = st > 0 && (b[st - 1].is_ascii_alphanumeric() || b[st - 1] == '$');
            if !after_at && !prev_alnum && (up == "REM" || up == "DATA") { cut = true; }
            if !after_at && !cut && up == "DEG" { out.push_str("POKE 251,6"); } else if !after_at && !cut && up == "RAD" { out.push_str("POKE 251,0"); } else { out.push_str(&w); }
        }
        out.push('\n');
    }
    out
}

fn prepare(src: &str, d: &Dialect, o: &B2aOptions) -> Result<Prepared, Vec<Diagnostic>> {
    let atari_src;
    let atari_d;
    let is_atari = target_info(&o.target).map_or(false, |t| t.atari);
    if o.turbo && !is_atari { return Err(vec![Diagnostic::error("E344", &o.file, 0, "Turbo BASIC XL gibt es nur für das Ziel atari8")]); }
    let d: &Dialect = if is_atari { atari_d = atari_dialect(d, o.turbo); &atari_d } else { d };
    let src = if is_atari {
        let e = atari_check(src, &o.file);
        if !e.is_empty() { return Err(e); }
        atari_src = atari_deg_rad(src);
        atari_src.as_str()
    } else { src };
    let mut po = PlainOptions::new(o.file.clone(), 63999);
    po.start = o.start;
    po.step = o.step;
    let n = number_lines(src, &po)?;
    let labels = n.labels.clone();
    // Atari-BASIC kennt lange Namen immer: sie werden stets auf Kurznamen abgebildet (der Parser liest nach CBM-Regeln: 2 Zeichen zählen)
    let r = if target_info(&o.target).map_or(false, |t| t.atari) { crate::longnames::rename(&n.text, crate::longnames::Target::Cbm(d), &o.file)? } else { crate::apply_long_names(n, crate::longnames::Target::Cbm(d), &o.file)? };
    let prog = parse_numbered_with(&r.text, d, &o.file, is_atari, o.turbo)?;
    Ok((prog, labels, r.names, r.warnings))
}

/// Ersetzt jede Zeile `#include <pfad>` durch den Inhalt der Datei unter `sys` (rekursiv): die Ergebnisdatei ist dann ohne Include-Ordner assemblierbar
fn embed_includes(asm: &str, sys: &std::path::Path, depth: usize, file: &str) -> Result<String, Vec<Diagnostic>> {
    let mut out = String::new();
    for line in asm.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include <") {
            let name = rest.trim_end_matches('>');
            let path = sys.join(name);
            let text = vc_core::read_static(&path).map_err(|e| vec![Diagnostic::error("E192", file, 0, format!("embed: {}: {e}", path.display()))])?;
            if depth > 8 { return Err(vec![Diagnostic::error("E192", file, 0, format!("embed: Include-Schleife bei {name}"))]); }
            out.push_str(&format!("; ---------------- {name} ----------------\n"));
            out.push_str(&embed_includes(&text, sys, depth + 1, file)?);
            if !out.ends_with('\n') { out.push('\n'); }
            out.push_str(&format!("; ---------------- Ende {name} ----------------\n"));
        } else { out.push_str(line); out.push('\n'); }
    }
    Ok(out)
}

/// Rundreise des Parsers: Text → Syntaxbaum → kanonischer Text → Syntaxbaum; beide Bäume müssen gleich sein (Zeilennummern und Anweisungen).
/// Gibt den kanonischen Text zurück. Anweisungen, die der Parser nur als `Unsupported` kennt, lassen sich nicht zurückschreiben (Fehler).
pub fn roundtrip(src: &str, d: &Dialect, o: &B2aOptions) -> Result<String, Vec<Diagnostic>> {
    let atari = target_info(&o.target).map_or(false, |t| t.atari);
    let first = prepare(src, d, o)?.0;
    let d2;
    let dd: &Dialect = if atari { d2 = atari_dialect(d, o.turbo); &d2 } else { d };
    let text = print::to_text(&first, dd, atari).map_err(|e| vec![Diagnostic::error("E190", &o.file, 0, e)])?;
    let second = parse_numbered_with(&text, dd, &o.file, atari, o.turbo)?;
    let key = |p: &ast::Program| p.lines.iter().map(|l| (l.number, l.stmts.clone())).collect::<Vec<_>>();
    if key(&first) != key(&second) {
        let at = key(&first).iter().zip(key(&second)).position(|(a, b)| *a != b).map_or(0, |i| first.lines[i].number);
        return Err(vec![Diagnostic::error("E191", &o.file, 0, format!("Rundreise ergibt einen anderen Syntaxbaum (erste Abweichung in Zeile {at})"))]);
    }
    Ok(text)
}

/// Nur parsen (Syntaxprüfung, `vcc basic ast`)
pub fn parse(src: &str, d: &Dialect, o: &B2aOptions) -> Result<ast::Program, Vec<Diagnostic>> { prepare(src, d, o).map(|p| p.0) }

/// BASIC-Quelltext → Assembler-Quelltext
pub fn convert(src: &str, d: &Dialect, o: &B2aOptions) -> Result<Converted, Vec<Diagnostic>> {
    let (prog, labels, names, mut warnings) = prepare(src, d, o)?;
    let comments: Vec<(usize, String)> = src.lines().enumerate().filter_map(|(i, raw)| {
        let t = raw.trim();
        if t.is_empty() { Some((i + 1, String::new())) } else { t.strip_prefix(';').map(|r| (i + 1, r.trim_end().to_string())) }
    }).collect();
    let mut c = gen::generate(&prog, d, &labels, &comments, o)?;
    c.names = names;
    if o.embed {
        let Some(dd) = &o.data_dir else { return Err(vec![Diagnostic::error("E192", &o.file, 0, "embed braucht den Ordner data (B2aOptions::data_dir)")]) };
        c.asm = embed_includes(&c.asm, &dd.join("sys"), 0, &o.file)?;
    }
    warnings.append(&mut c.warnings);
    c.warnings = warnings;
    if target_info(&o.target).map_or(false, |t| t.atari) {
        let num = if c.stage >= 2 { "Zahlen sind 6-Byte-BCD des OS-Fließkommapakets (wie Atari-BASIC); TAN gibt es nicht" } else { "Zahlen sind 16-Bit-Ganzzahlen statt BCD mit 10 Stellen" };
        c.warnings.push(Diagnostic::warning("I150", &o.file, 0, format!("Atari: {num}; PRINT gibt Zahlen wie Atari-BASIC ohne Vorzeichenstelle aus; Fehlermeldungen heißen wie bei CBM (z. B. ILLEGAL QUANTITY)")));
    }
    Ok(c)
}
