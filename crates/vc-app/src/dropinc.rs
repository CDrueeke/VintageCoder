// SPDX-License-Identifier: GPL-3.0-or-later
//! Ziehen aus dem Projektbaum in den Quelltext: aus Datei und Zieldokument die passende Include-Zeile bestimmen
//! (reine Logik ohne Oberfläche, damit testbar).
//!
//! Regeln (die der eigene Assembler und BASIC wirklich kennen):
//! - Assembler: `.include "pfad"` für Quelltexte (`.asm`, `.inc`, `.s`), `.incbin "pfad"` für sonstige Dateien. Der Pfad wird
//!   relativ zur Datei des Zieldokuments aufgelöst, die Zeichenkette kennt keine Escapes (ein `"` im Namen geht daher nicht).
//! - Medien (Sprite, Zeichensatz, Bildschirm, Song …): über ihren Export `build/export/<name>.asm` bzw. `<name>.data.bas`;
//!   der Build erzeugt ihn bei Bedarf neu.
//! - BASIC: nur `#include "x.bas"` (gleiche Endung) und exportierte DATA-Zeilen; kein `.incbin`.

use std::path::{Component, Path};

/// Ergebnis: Zeile zum Einfügen, eine sinnvolle Alternative und der aufgelöste Pfad der eingebundenen Datei
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan { pub line: String, pub alt: Option<(String, String)>, pub target: String }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reject {
    /// Ordner oder nicht vorhandene Datei
    NotAFile,
    SelfDrop,
    Outside,
    /// Zieldokument ist kein Assembler- oder BASIC-Quelltext
    TargetNotSource,
    Unsupported(String),
    BadName(String),
    /// Schon eingebunden, in dieser (0-basierten) Zeile
    Already(usize),
}

impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reject::NotAFile => write!(f, "Ordner und nicht vorhandene Dateien lassen sich nicht einbinden"),
            Reject::SelfDrop => write!(f, "Eine Datei lässt sich nicht in sich selbst einbinden"),
            Reject::Outside => write!(f, "Die Datei liegt außerhalb des Projekts"),
            Reject::TargetNotSource => write!(f, "Hier gibt es kein Einbinden: nur in Assembler- und BASIC-Quelltexte"),
            Reject::Unsupported(s) | Reject::BadName(s) => write!(f, "{s}"),
            Reject::Already(l) => write!(f, "Diese Datei ist schon eingebunden (Zeile {}) – nichts eingefügt", l + 1),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind { Asm, Basic(&'static str) }

fn lower(s: &str) -> String { s.to_ascii_lowercase() }
fn ends(l: &str, exts: &[&str]) -> bool { exts.iter().any(|e| l.ends_with(e)) }

fn doc_kind(doc: &str) -> Option<Kind> {
    let l = lower(doc);
    if ends(&l, &[".asm", ".inc", ".s"]) { Some(Kind::Asm) }
    else if l.ends_with(".bas") { Some(Kind::Basic(".bas")) }
    else if l.ends_with(".lst") { Some(Kind::Basic(".lst")) }
    else if l.ends_with(".tur") { Some(Kind::Basic(".tur")) }
    else { None }
}

/// Pfad bereinigen: `\` → `/`, `.` entfernen, `..` auflösen. `None`, wenn er über die Wurzel hinausführt oder absolut ist.
pub fn normalize(p: &str) -> Option<String> {
    let p = p.replace('\\', "/");
    let mut out: Vec<String> = vec![];
    for c in Path::new(&p).components() {
        match c {
            Component::Normal(s) => out.push(s.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => { out.pop()?; }
            _ => return None,
        }
    }
    Some(out.join("/"))
}

/// Pfad `to` (relativ zur Projektwurzel) von der Datei `doc` aus gesehen, mit `/`
pub fn relative_from_doc(doc: &str, to: &str) -> String {
    let dir: Vec<&str> = doc.split('/').collect::<Vec<_>>().split_last().map_or(vec![], |(_, d)| d.to_vec());
    let t: Vec<&str> = to.split('/').collect();
    let common = dir.iter().zip(&t).take_while(|(a, b)| a == b).count().min(t.len().saturating_sub(1));
    let mut parts: Vec<&str> = vec![".."; dir.len() - common];
    parts.extend(&t[common..]);
    parts.join("/")
}

/// Pfad einer Include-Zeile des Dokuments `doc` als Projektpfad
fn resolve_from_doc(doc: &str, p: &str) -> Option<String> {
    let dir = doc.rsplit_once('/').map_or("", |x| x.0);
    normalize(&if dir.is_empty() { p.to_string() } else { format!("{dir}/{}", p.replace('\\', "/")) })
}

fn check_name(rel: &str) -> Result<(), Reject> {
    if rel.contains('"') { return Err(Reject::BadName("Der Dateiname enthält ein Anführungszeichen – das kennt der Assembler in Include-Pfaden nicht. Bitte die Datei umbenennen.".into())); }
    if rel.chars().any(|c| c.is_control()) { return Err(Reject::BadName("Der Dateiname enthält Steuerzeichen – bitte umbenennen.".into())); }
    Ok(())
}

/// Zeile für das Einbinden von `src` (Pfad relativ zum Projekt) in das Dokument `doc` mit dem Text `text`.
/// `root`: Projektwurzel, nur für die Prüfung „ist eine Datei“; `None` überspringt sie (Tests).
pub fn plan(doc: &str, text: &str, src: &str, root: Option<&Path>) -> Result<Plan, Reject> {
    let Some(kind) = doc_kind(doc) else { return Err(Reject::TargetNotSource) };
    let src = normalize(src).filter(|s| !s.is_empty()).ok_or(Reject::Outside)?;
    if src == normalize(doc).unwrap_or_default() { return Err(Reject::SelfDrop); }
    if let Some(r) = root { if !r.join(&src).is_file() { return Err(Reject::NotAFile); } }
    let l = lower(&src);
    if l.ends_with(".vcprj") || vc_ide::diskimage::is_image_name(&l) {
        return Err(Reject::Unsupported("Projektdatei und Diskettenimages lassen sich nicht in Quelltext einbinden".into()));
    }
    let q = |p: &str| -> Result<String, Reject> { check_name(p)?; Ok(relative_from_doc(doc, p)) };
    // (Zeile, Alternative, aufgelöstes Ziel)
    let (line, alt, target): (String, Option<(String, String)>, String) = if vc_media::naming::is_data_file(&l) {
        let stem = vc_media::naming::stem(&src);
        match kind {
            Kind::Asm => { let t = format!("build/export/{stem}.asm"); (format!(".include \"{}\"", q(&t)?), None, t) }
            Kind::Basic(_) => { let t = format!("build/export/{stem}.data.bas"); (format!("#include \"{}\"", q(&t)?), None, t) }
        }
    } else if l.ends_with(".sbas") {
        return Err(Reject::Unsupported("Strukturiertes BASIC (.sbas) wird nicht unterstützt – bitte nur .bas einbinden".into()));
    } else if ends(&l, &[".bas", ".lst", ".tur"]) {
        match kind {
            Kind::Asm => return Err(Reject::Unsupported("BASIC-Quelltext lässt sich in Assembler nicht einbinden. Wandle ihn mit „Nach Assembler konvertieren …“ um oder binde ein Assembler-Programm ein.".into())),
            Kind::Basic(ext) if l.ends_with(ext) => (format!("#include \"{}\"", q(&src)?), None, src.clone()),
            Kind::Basic(_) => return Err(Reject::Unsupported("Die Datei ist ein anderer BASIC-Dialekt (andere Endung) als das Ziel – das passt nicht zusammen".into())),
        }
    } else if ends(&l, &[".asm", ".inc", ".s"]) {
        match kind {
            Kind::Asm => { let p = q(&src)?; (format!(".include \"{p}\""), Some(("#include (nur einmal)".into(), format!("#include \"{p}\""))), src.clone()) }
            Kind::Basic(_) => return Err(Reject::Unsupported("Assembler-Quelltext lässt sich in BASIC nicht einbinden. Maschinencode gehört per .incbin in ein Assembler-Programm.".into())),
        }
    } else {
        match kind {
            Kind::Asm => {
                let p = q(&src)?;
                let alt = l.ends_with(".prg").then(|| ("ohne Ladeadresse (, 2)".to_string(), format!(".incbin \"{p}\", 2")));
                (format!(".incbin \"{p}\""), alt, src.clone())
            }
            Kind::Basic(_) => return Err(Reject::Unsupported("BASIC kennt kein .incbin. Binäre Daten lassen sich nur über einen Assembler-Quelltext oder als Medium (DATA-Zeilen) einbinden.".into())),
        }
    };
    // schon eingebunden? (gleiches Ziel in einer Include-Zeile, egal mit welchem Schlüsselwort)
    for (i, ln) in text.split('\n').enumerate() {
        if let Some((p, _, _)) = crate::editor::include_span(ln) {
            if resolve_from_doc(doc, &p).as_deref() == Some(target.as_str()) { return Err(Reject::Already(i)); }
        }
    }
    Ok(Plan { line, alt, target })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(doc: &str, src: &str) -> Result<Plan, Reject> { plan(doc, "", src, None) }
    fn line(doc: &str, src: &str) -> String { p(doc, src).unwrap().line }

    #[test]
    fn source_and_binary_in_assembler() {
        assert_eq!(line("main.asm", "lib/music.asm"), ".include \"lib/music.asm\"");
        assert_eq!(line("main.asm", "data/level.bin"), ".incbin \"data/level.bin\"");
        assert_eq!(line("src/main.asm", "src/lib/x.inc"), ".include \"lib/x.inc\"");
        assert_eq!(line("src/main.asm", "data/x.bin"), ".incbin \"../data/x.bin\"");
        assert_eq!(line("a/b/main.asm", "a/c.bin"), ".incbin \"../c.bin\"");
        assert_eq!(line("main.asm", "x.PRG"), ".incbin \"x.PRG\"");
    }

    #[test]
    fn alternatives() {
        assert_eq!(p("main.asm", "x.prg").unwrap().alt.unwrap().1, ".incbin \"x.prg\", 2");
        assert_eq!(p("main.asm", "lib.asm").unwrap().alt.unwrap().1, "#include \"lib.asm\"");
        assert!(p("main.asm", "x.bin").unwrap().alt.is_none());
    }

    #[test]
    fn media_use_exports() {
        assert_eq!(line("main.asm", "gfx/hero.vcsprite"), ".include \"build/export/hero.asm\"");
        assert_eq!(line("src/main.asm", "hero.vcsprite"), ".include \"../build/export/hero.asm\"");
        assert_eq!(line("main.bas", "gfx/hero.vcsprite"), "#include \"build/export/hero.data.bas\"");
        assert_eq!(line("sub/main.bas", "hero.vcsprite"), "#include \"../build/export/hero.data.bas\"");
    }

    #[test]
    fn basic_rules() {
        assert_eq!(line("main.bas", "teile/ausgabe.bas"), "#include \"teile/ausgabe.bas\"");
        assert!(matches!(p("main.bas", "teile/x.tur"), Err(Reject::Unsupported(_))), "anderer Dialekt");
        assert!(matches!(p("main.bas", "x.asm"), Err(Reject::Unsupported(_))));
        assert!(matches!(p("main.bas", "x.bin"), Err(Reject::Unsupported(_))), "kein incbin in BASIC");
        assert!(matches!(p("main.asm", "x.bas"), Err(Reject::Unsupported(_))));
        assert!(matches!(p("main.asm", "x.sbas"), Err(Reject::Unsupported(_))));
    }

    #[test]
    fn special_characters() {
        assert_eq!(line("main.asm", "daten/mein level.bin"), ".incbin \"daten/mein level.bin\"");
        assert_eq!(line("main.asm", "grafik/Größe über.bin"), ".incbin \"grafik/Größe über.bin\"");
        assert_eq!(line("main.asm", "a\\b\\c.bin"), ".incbin \"a/b/c.bin\"", "Windows-Pfade mit /");
        assert_eq!(line("main.asm", "./x.bin"), ".incbin \"x.bin\"");
        assert!(matches!(p("main.asm", "a\"b.bin"), Err(Reject::BadName(_))));
    }

    #[test]
    fn rejections() {
        assert_eq!(p("main.asm", "main.asm"), Err(Reject::SelfDrop));
        assert_eq!(p("main.asm", "./main.asm"), Err(Reject::SelfDrop));
        assert_eq!(p("main.asm", "../x.bin"), Err(Reject::Outside));
        assert_eq!(p("main.asm", "/etc/passwd"), Err(Reject::Outside));
        assert_eq!(p("notes.txt", "x.bin"), Err(Reject::TargetNotSource));
        assert!(matches!(p("main.asm", "p.vcprj"), Err(Reject::Unsupported(_))));
        assert!(matches!(p("main.asm", "disk.d64"), Err(Reject::Unsupported(_))));
        let dir = std::env::temp_dir().join(format!("vc_dropinc_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("ordner")).unwrap();
        assert_eq!(plan("main.asm", "", "ordner", Some(&dir)), Err(Reject::NotAFile));
        assert_eq!(plan("main.asm", "", "fehlt.bin", Some(&dir)), Err(Reject::NotAFile));
        std::fs::write(dir.join("da.bin"), [1]).unwrap();
        assert!(plan("main.asm", "", "da.bin", Some(&dir)).is_ok());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn existing_include_is_not_duplicated() {
        let t = "start:\n        .include \"lib/m.asm\"\n        .incbin \"../d/x.bin\", 2\n; .include \"c.asm\"\n";
        assert_eq!(plan("main.asm", t, "lib/m.asm", None), Err(Reject::Already(1)));
        assert_eq!(plan("src/main.asm", t, "d/x.bin", None), Err(Reject::Already(2)), "gleiche Datei, anderer Offset");
        assert_eq!(plan("main.asm", "#include \"lib/./m.asm\"", "lib/m.asm", None), Err(Reject::Already(0)));
        assert!(plan("main.asm", t, "c.asm", None).is_ok(), "auskommentierte Zeile zählt nicht");
        assert_eq!(plan("main.asm", ".include \"build/export/hero.asm\"", "hero.vcsprite", None), Err(Reject::Already(0)));
        assert!(plan("main.asm", t, "other.bin", None).is_ok());
        assert_eq!(Reject::Already(1).to_string(), "Diese Datei ist schon eingebunden (Zeile 2) – nichts eingefügt");
    }

    #[test]
    fn relative_paths() {
        assert_eq!(relative_from_doc("main.asm", "a/b.bin"), "a/b.bin");
        assert_eq!(relative_from_doc("a/main.asm", "a/b.bin"), "b.bin");
        assert_eq!(relative_from_doc("a/b/main.asm", "c.bin"), "../../c.bin");
        assert_eq!(relative_from_doc("a/main.asm", "a/b/c/d.bin"), "b/c/d.bin");
        assert_eq!(normalize("a/../b/./c").as_deref(), Some("b/c"));
        assert_eq!(normalize("../a"), None);
    }
}
