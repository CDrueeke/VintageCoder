// SPDX-License-Identifier: GPL-3.0-or-later
//! Systembibliothek (`data/sys/**.asm`) als Symboltabelle: Namen, Adressen, Abschnitte und Kurzbeschreibungen für Memory Map und Referenz.
use std::collections::HashSet;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Sym { pub name: String, pub addr: u32, pub doc: String, pub section: String, pub file: String }

/// Einstiegsdateien je Ziel; `#include <…>` darin wird mitgelesen.
pub fn roots(target: &str) -> &'static [&'static str] {
    match target {
        "c128" => &["c128/kernal.asm"],
        "plus4" | "c16" => &["plus4/kernal.asm"],
        "vic20" => &["vic20/kernal.asm"],
        "atari8" => &["atari8bit/antic.asm", "atari8bit/gtia.asm", "atari8bit/pokey.asm", "atari8bit/os.asm"],
        "mega65" => &["mega65/hardware.asm", "mega65/kernal.asm", "mega65/rom_920418.asm"],
        _ => &["c64/hardware.asm", "c64/kernal.asm", "c64/basic.asm"],
    }
}

fn parse_file(data: &Path, rel: &str, seen: &mut HashSet<String>, out: &mut Vec<Sym>) {
    if !seen.insert(rel.to_string()) { return; }
    let Ok(text) = std::fs::read_to_string(data.join("sys").join(rel)) else { return };
    let mut section = String::new();
    for l in text.lines() {
        let l = l.trim();
        if let Some(h) = l.strip_prefix("// ----") { section = h.trim().to_string(); continue; }
        if let Some(inc) = l.strip_prefix("#include <") { if let Some(p) = inc.split('>').next() { parse_file(data, p, seen, out); } continue; }
        let Some(rest) = l.strip_prefix(".const ") else { continue };
        let (code, doc) = rest.split_once("//").map_or((rest, ""), |(c, d)| (c, d.trim()));
        let Some((name, val)) = code.split_once('=') else { continue };
        let val = val.trim();
        let Some(addr) = val.strip_prefix('$').and_then(|h| u32::from_str_radix(h, 16).ok()) else { continue };
        out.push(Sym { name: name.trim().to_string(), addr, doc: doc.to_string(), section: section.clone(), file: rel.to_string() });
    }
}

/// Alle Symbole der Bibliothek des Ziels (in Dateireihenfolge)
pub fn symbols(data: &Path, target: &str) -> Vec<Sym> {
    let (mut out, mut seen) = (vec![], HashSet::new());
    for r in roots(target) { parse_file(data, r, &mut seen, &mut out); }
    out
}

/// Ein Symbol nach Namen (ohne Beachtung der Groß-/Kleinschreibung)
pub fn find<'a>(syms: &'a [Sym], name: &str) -> Option<&'a Sym> { syms.iter().find(|s| s.name.eq_ignore_ascii_case(name)) }

/// Symbole, die genau an `addr` liegen
pub fn at(syms: &[Sym], addr: u32) -> Vec<&Sym> { syms.iter().filter(|s| s.addr == addr).collect() }

/// Markdown-Seite zu einem Symbol (für Referenz und Hilfe)
pub fn page(syms: &[Sym], s: &Sym) -> String {
    let mut md = format!("# {}\n\nAdresse `${:04X}`", s.name, s.addr);
    if !s.section.is_empty() { md.push_str(&format!(" · {}", s.section)); }
    md.push_str("\n\n");
    if !s.doc.is_empty() { md.push_str(&s.doc); md.push_str("\n\n"); }
    let others: Vec<&str> = at(syms, s.addr).into_iter().filter(|o| o.name != s.name).map(|o| o.name.as_str()).collect();
    if !others.is_empty() { md.push_str(&format!("Gleiche Adresse: {}\n\n", others.join(", "))); }
    md.push_str(&format!("Einbinden: `#include <{}>`", s.file));
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
    #[test]
    fn reads_library_and_follows_includes() {
        let c64 = symbols(&data(), "c64");
        let s = find(&c64, "vic_bordercolor").unwrap();
        assert_eq!((s.addr, s.file.as_str()), (0xD020, "c64/hardware.asm"));
        assert_eq!(find(&c64, "CHROUT").unwrap().addr, 0xFFD2);
        // C128: hardware.asm kommt über #include dazu, C64-Hardware ebenfalls
        let c128 = symbols(&data(), "c128");
        assert_eq!(find(&c128, "MMU_CR").unwrap().addr, 0xD500);
        assert_eq!(find(&c128, "VIC_BORDERCOLOR").unwrap().addr, 0xD020);
        assert_eq!(find(&symbols(&data(), "mega65"), "SPIN_SPOUT").unwrap().addr, 0xFF4D);
        assert_eq!(find(&c128, "SPIN_SPOUT").unwrap().addr, 0xFF47);
        assert!(find(&symbols(&data(), "atari8"), "COLPF0").is_some());
        assert!(page(&c64, s).contains("#include <c64/hardware.asm>"));
        // Referenz und Suche finden Bibliotheksnamen
        let v = crate::help::lookup(&data(), "c64", "chrout", "asm").unwrap();
        assert_eq!(v["kind"], "symbol");
        assert!(v["markdown"].as_str().unwrap().contains("$FFD2"));
        let r = crate::help::search(&data(), "atari8", "colpf").unwrap();
        assert!(r["results"].as_array().unwrap().iter().any(|x| x["kind"] == "symbol" && x["title"] == "COLPF0"));
    }
}
