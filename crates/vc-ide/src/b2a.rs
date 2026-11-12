// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC → Assembler in der IDE: `.bas` eines Projekts nach lesbarem Assembler-Quelltext konvertieren
//!
//!
//! Die erzeugte Datei trägt eine Kopfzeile ([`MARK`]) und gilt als *erzeugt*: Beim erneuten Konvertieren wird sie ersetzt.
//! Eine vorhandene Datei ohne diese Kopfzeile (handgepflegt) wird nie überschrieben.
use crate::projfile::{ProjectFile, Source};
use std::path::{Path, PathBuf};
use vc_basic::b2a::{convert, B2aOptions};
use vc_basic::Dialect;
use vc_core::Diagnostic;

/// Erste Zeile jeder erzeugten Datei
pub const MARK: &str = "; Erzeugt von VintageCoder (basic2asm)";

#[derive(Clone, Debug)]
pub struct Req {
    /// BASIC-Quelle relativ zum Projekt
    pub rel: String,
    /// Ausgabedatei relativ zum Projekt (leer: [`default_out`])
    pub out: String,
    /// Zielsystem: `c64`, `c128`, `plus4` oder `atari8`
    pub target: String,
    pub int_div: bool,
    pub force_float: bool,
    pub track_line: bool,
    pub no_rem: bool,
    /// Stil `inline`: einfache Ganzzahlausdrücke als direkte Befehle
    pub inline: bool,
    /// Laufzeit in die Ergebnisdatei kopieren (statt `#include`)
    pub embed: bool,
    /// Quelle ist Turbo BASIC XL (nur atari8)
    pub turbo: bool,
    /// Ergebnis als Quelle in die Projektdatei eintragen (die `.bas` bleibt Quelle)
    pub register: bool,
}

impl Req {
    /// `.tur`-Dateien sind Turbo BASIC XL: der Schalter `turbo` ist dann vorgewählt
    pub fn new(rel: impl Into<String>) -> Req {
        let rel: String = rel.into();
        let turbo = rel.to_ascii_lowercase().ends_with(".tur");
        Req { rel, out: String::new(), target: if turbo { "atari8".into() } else { "c64".into() }, int_div: false, force_float: false, track_line: true, no_rem: false, inline: false, embed: false, turbo, register: true }
    }
}

#[derive(Debug)]
pub struct Report {
    pub out: String,
    pub map: String,
    pub lines: usize,
    pub stage: u8,
    pub modules: Vec<String>,
    pub warnings: Vec<Diagnostic>,
    pub registered: bool,
}

/// Wurde die Datei von basic2asm erzeugt?
pub fn is_generated(path: &Path) -> bool {
    std::fs::read_to_string(path).map_or(false, |t| t.starts_with(MARK))
}

/// Vorschlag für die Ausgabedatei: `<name>_asm.asm` im Ordner der Quelle (nicht `<name>.asm`, damit die Ausgabedateien
/// `build/<name>.prg` von BASIC-Quelle und Ergebnis nicht zusammenfallen). Ist der Name durch eine handgepflegte Datei belegt,
/// wird durchnummeriert.
pub fn default_out(root: &Path, rel: &str) -> String {
    let p = Path::new(rel);
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "programm".into());
    let dir = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
    let name = |n: usize| { let f = if n == 0 { format!("{stem}_asm.asm") } else { format!("{stem}_asm{}.asm", n + 1) }; if dir.is_empty() { f } else { format!("{dir}/{f}") } };
    (0..100).map(name).find(|c| { let a = root.join(c); !a.exists() || is_generated(&a) }).unwrap_or_else(|| name(0))
}

/// Zeilenkarte neben der Ausgabe: `<name>.b2a.json`
fn map_name(out: &str) -> String { format!("{}.b2a.json", out.strip_suffix(".asm").unwrap_or(out)) }

/// Datei als Assembler-Quelle in die Projektdatei eintragen; true, wenn sich etwas geändert hat
pub fn register(pf: &mut ProjectFile, bas_rel: &str, out_rel: &str, target: &str) -> bool {
    if pf.source_files().iter().any(|s| s.0 == out_rel) { return false; }
    let tg = pf.target_of(bas_rel).filter(|t| !t.is_empty()).unwrap_or_else(|| target.to_string());
    match pf.sources.iter_mut().find(|s| s.kind == "asm" && s.targets.first().map_or(true, |t| *t == tg) && s.dialect.is_empty()) {
        Some(g) => g.files.push(out_rel.to_string()),
        None => pf.sources.push(Source { targets: vec![tg], kind: "asm".into(), files: vec![out_rel.to_string()], ..Default::default() }),
    }
    true
}

/// Konvertiert `req.rel` und schreibt Assembler und Zeilenkarte; trägt die Ausgabe auf Wunsch in die Projektdatei ein.
pub fn convert_file(root: &Path, data: &Path, req: &Req) -> Result<Report, Vec<Diagnostic>> {
    let err = |m: String| vec![Diagnostic::error("E330", &req.rel, 0, m)];
    let Some(ti) = vc_basic::b2a::target_info(&req.target) else { return Err(err(format!("Ziel '{}' ist noch nicht umgesetzt (verfügbar: {})", req.target, vc_basic::b2a::TARGETS.iter().map(|t| t.id).collect::<Vec<_>>().join(", ")))) };
    let lower = req.rel.to_ascii_lowercase();
    if !(lower.ends_with(".bas") || lower.ends_with(".lst") || lower.ends_with(".tur")) { return Err(err("Nur .bas-, .lst- oder .tur-Dateien (natives BASIC) lassen sich konvertieren".into())); }
    let src_path = root.join(&req.rel);
    let src = std::fs::read_to_string(&src_path).map_err(|e| err(format!("{}: {e}", src_path.display())))?;
    let out_rel = if req.out.trim().is_empty() { default_out(root, &req.rel) } else { req.out.trim().to_string() };
    if !out_rel.to_ascii_lowercase().ends_with(".asm") { return Err(err(format!("Ausgabedatei '{out_rel}' muss auf .asm enden"))); }
    let out_path = root.join(&out_rel);
    if out_path.exists() && !is_generated(&out_path) {
        return Err(err(format!("'{out_rel}' existiert und wurde nicht von basic2asm erzeugt – wird nicht überschrieben (anderen Namen wählen)")));
    }
    let dialect = Dialect::load(data, ti.dialect).map_err(|e| err(e))?;
    let mut o = B2aOptions::new(req.rel.clone());
    o.target = req.target.clone();
    o.int_div = req.int_div;
    o.force_float = req.force_float;
    o.track_line = req.track_line;
    o.no_rem = req.no_rem;
    o.inline = req.inline;
    o.embed = req.embed;
    o.turbo = req.turbo;
    o.data_dir = Some(data.to_path_buf());
    let c = convert(&src, &dialect, &o)?;
    if let Some(d) = out_path.parent() { std::fs::create_dir_all(d).map_err(|e| err(e.to_string()))?; }
    std::fs::write(&out_path, &c.asm).map_err(|e| err(format!("{out_rel}: {e}")))?;
    let map = map_name(&out_rel);
    std::fs::write(root.join(&map), c.map_json(&req.rel)).map_err(|e| err(format!("{map}: {e}")))?;
    let mut registered = false;
    if req.register {
        if let Some(file) = crate::projfile::find(root) {
            if let Ok(mut pf) = ProjectFile::load(&file) {
                if register(&mut pf, &req.rel, &out_rel, &req.target) { pf.save(&file).map_err(|e| err(e))?; registered = true; }
            }
        }
    }
    Ok(Report { out: out_rel, map, lines: c.map.len(), stage: c.stage, modules: c.modules, warnings: c.warnings, registered })
}

/// Pfad für Meldungen
pub fn show(root: &Path, rel: &str) -> PathBuf { root.join(rel) }

// ---- Zeilenkarte für den Debugger BASIC-Zeile ↔ Adresse im konvertierten Programm

/// Zeilenkarte einer konvertierten Datei: Quelle (`.bas`), Assembler-Datei und je BASIC-Zeile (Zeilennummer, Zeile in der Quelle, Label)
#[derive(Clone, Debug, PartialEq)]
pub struct LineMap {
    pub source: String,
    pub asm: String,
    pub lines: Vec<(u32, usize, String)>,
}

/// Liest die Zeilenkarten aller Assembler-Quellen des Projekts (Datei `<name>.b2a.json` neben der `.asm`)
pub fn load_maps(root: &Path, pf: &ProjectFile) -> Vec<LineMap> {
    let mut v = vec![];
    for (file, kind, _) in pf.source_files() {
        if kind != "asm" { continue; }
        let Ok(text) = std::fs::read_to_string(root.join(map_name(&file))) else { continue };
        let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        let Some(source) = j["source"].as_str() else { continue };
        let lines = j["lines"].as_array().map(|a| a.iter().filter_map(|e| Some((e["basic"].as_u64()? as u32, e["source"].as_u64()? as usize, e["label"].as_str()?.to_string()))).collect()).unwrap_or_default();
        v.push(LineMap { source: source.to_string(), asm: file, lines });
    }
    v
}

impl LineMap {
    /// Adresse der BASIC-Zeile, die in der Quelldatei in Zeile `src_line` (1-basiert) steht; `sym` löst Labels des Assemblers auf
    pub fn addr_of(&self, sym: &dyn Fn(&str) -> Option<i64>, src_line: usize) -> Option<u16> {
        let (_, _, label) = self.lines.iter().find(|l| l.1 == src_line)?;
        sym(label).map(|a| a as u16)
    }

    /// BASIC-Zeile (Nummer, Zeile in der Quelldatei), in der der Programmzähler `pc` steht. Nur im Programmteil (bis `b2a_fall`),
    /// nicht in der Laufzeit-Bibliothek dahinter.
    pub fn line_at(&self, sym: &dyn Fn(&str) -> Option<i64>, pc: u16) -> Option<(u32, usize)> {
        let end = sym("b2a_fall")? as u32;
        let mut v: Vec<(u32, u32, usize)> = self.lines.iter().filter_map(|(b, s, l)| sym(l).map(|a| (a as u32, *b, *s))).collect();
        v.sort();
        let pc = pc as u32;
        if pc >= end { return None; }
        v.iter().rev().find(|(a, _, _)| *a <= pc).map(|(_, b, s)| (*b, *s))
    }

    /// Anfangsadresse der nächsten BASIC-Zeile hinter der Zeile, in der `pc` steht (Ziel für „nächste BASIC-Zeile“); in der letzten Zeile
    /// das Ende des Programmteils (`b2a_fall`), außerhalb des Programmteils `None`
    pub fn next_line_addr(&self, sym: &dyn Fn(&str) -> Option<i64>, pc: u16) -> Option<u16> {
        let end = sym("b2a_fall")? as u32;
        if pc as u32 >= end { return None; }
        let mut starts: Vec<u32> = self.lines.iter().filter_map(|(_, _, l)| sym(l).map(|a| a as u32)).filter(|a| *a < end).collect();
        starts.sort(); starts.dedup();
        Some(starts.into_iter().find(|a| *a > pc as u32).unwrap_or(end) as u16)
    }
}

/// Zielsysteme, die der Konverter kennt
pub fn targets() -> Vec<&'static str> { vc_basic::b2a::TARGETS.iter().map(|t| t.id).collect() }

#[cfg(test)]
mod tur_tests {
    use super::Req;
    #[test]
    fn tur_files_default_to_turbo_on_atari() {
        let r = Req::new("spiel/balken.tur");
        assert!(r.turbo && r.target == "atari8");
        let r = Req::new("a.bas");
        assert!(!r.turbo && r.target == "c64");
    }
}
