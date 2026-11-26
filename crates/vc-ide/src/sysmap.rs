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
    // Anzeigetexte (Abschnittsüberschriften und Kurzbeschreibungen) als Zeilenwerte `L<n>`; die englische Überlagerung `sys/<datei>` ersetzt sie
    // (tools/check_i18n_data.py erzeugt dieselben Zeiger mit derselben Regel)
    let mut lines = serde_json::Map::new();
    for (i, l) in text.lines().enumerate() {
        let l = l.trim();
        if let Some(h) = l.strip_prefix("// ----") { lines.insert(format!("L{}", i + 1), h.trim().into()); continue; }
        let Some(rest) = l.strip_prefix(".const ") else { continue };
        if let Some((_, doc)) = rest.split_once("//") { lines.insert(format!("L{}", i + 1), doc.trim().into()); }
    }
    let mut lines = serde_json::Value::Object(lines);
    vc_i18n::localize_json(&mut lines, &format!("sys/{rel}.json"));
    let shown = |i: usize| lines[format!("L{}", i + 1)].as_str().unwrap_or("").to_string();
    let mut section = String::new();
    for (i, l) in text.lines().enumerate() {
        let l = l.trim();
        if l.starts_with("// ----") { section = shown(i); continue; }
        if let Some(inc) = l.strip_prefix("#include <") { if let Some(p) = inc.split('>').next() { parse_file(data, p, seen, out); } continue; }
        let Some(rest) = l.strip_prefix(".const ") else { continue };
        let code = rest.split_once("//").map_or(rest, |(c, _)| c);
        let Some((name, val)) = code.split_once('=') else { continue };
        let val = val.trim();
        let Some(addr) = val.strip_prefix('$').and_then(|h| u32::from_str_radix(h, 16).ok()) else { continue };
        out.push(Sym { name: name.trim().to_string(), addr, doc: shown(i), section: section.clone(), file: rel.to_string() });
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
    let mut md = format!("# {}\n\n{}", s.name, vc_i18n::t!("help-sym-address", addr = format!("${:04X}", s.addr)));
    if !s.section.is_empty() { md.push_str(&format!(" · {}", s.section)); }
    md.push_str("\n\n");
    if !s.doc.is_empty() { md.push_str(&s.doc); md.push_str("\n\n"); }
    let others: Vec<&str> = at(syms, s.addr).into_iter().filter(|o| o.name != s.name).map(|o| o.name.as_str()).collect();
    if !others.is_empty() { md.push_str(&format!("{}\n\n", vc_i18n::t!("help-sym-same-address", names = others.join(", ")))); }
    md.push_str(&vc_i18n::t!("help-sym-include", file = s.file.as_str()));
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

    /// Englisch: Kurzbeschreibungen und Abschnitte kommen aus der Überlagerung `data/i18n/en/sys`; Adressen und Namen bleiben
    #[test]
    fn englische_beschreibungen_der_systembibliothek() {
        let umlaut = |t: &str| t.chars().any(|c| "äöüÄÖÜß".contains(c));
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        let de = symbols(&data(), "c64");
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        let en = symbols(&data(), "c64");
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(de.len(), en.len());
        let mut geaendert = 0;
        for (a, b) in de.iter().zip(&en) {
            assert_eq!((&a.name, a.addr, &a.file), (&b.name, b.addr, &b.file));
            assert!(!umlaut(&b.doc) && !umlaut(&b.section), "{}: deutsche Zeichen {:?} / {:?}", b.name, b.doc, b.section);
            if a.doc != b.doc { geaendert += 1; }
        }
        assert!(geaendert > 50, "{geaendert} übersetzte Beschreibungen");
    }
}
