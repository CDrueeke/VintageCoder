// SPDX-License-Identifier: GPL-3.0-or-later
//! Die Beispiele der Hilfeseiten zu den Direktiven bauen mit dem Assembler und erzeugen die in `;=`-Zeilen genannten Bytes.
//! Kennzeichen am Anfang des Blocks: `asm [roh] [fehler|warnung] [target=…] [fmt=…]`; `;=*` (nur Segmentanfang) wird nicht verglichen.
use std::path::PathBuf;
use vc_asm::{assemble, Options, Policy};
use vc_target::{load_cpu, load_target};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn blocks(md: &str) -> Vec<(String, String)> {
    let mut v = vec![];
    let mut cur: Option<(String, String)> = None;
    for l in md.lines() {
        match (&mut cur, l.strip_prefix("```")) {
            (None, Some(tag)) if tag.starts_with("asm") => cur = Some((tag.to_string(), String::new())),
            (None, _) => {}
            (Some(_), Some("")) => v.push(cur.take().unwrap()),
            (Some(c), _) => { c.1.push_str(l); c.1.push('\n'); }
        }
    }
    v
}

fn check(page: &str) -> usize {
    let md = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vc-ide/help").join(page)).unwrap();
    let mut n = 0; let mut bad = 0;
    for (tag, src) in blocks(&md) {
        let flags: Vec<&str> = tag.split_whitespace().collect();
        // Beispiele, die Zusatzdateien oder besondere Einstellungen brauchen, prüft diese Probe nicht
        if flags.iter().any(|f| f.starts_with("illegals=")) || src.contains(".incbin") || src.contains(".tbl") || src.contains(".include") || src.contains("#include") { continue; }
        let target = flags.iter().find_map(|f| f.strip_prefix("target=")).unwrap_or("c64");
        let t = load_target(&data(), target).unwrap();
        let mut o = Options::new("hilfe.asm", Policy::Allow);
        o.atari_text = t.text_encoding == "atari";
        o.basic_start = t.basic_load_addr;
        for f in &flags { if let Some(d) = f.strip_prefix("D=") { let (n, v) = d.split_once('=').unwrap_or((d, "1")); o.defines.push((n.to_string(), v.parse().unwrap())); } }
        let cpu = load_cpu(&data(), &t.cpu).unwrap();
        // ohne `roh` setzt die Hilfe vor das Beispiel `*= $c000` (die Adressen in den Texten gehen davon aus)
        let full = if flags.contains(&"roh") { src.clone() } else { format!("*= $c000\n{src}") };
        let res = assemble(&full, &cpu, &o);
        let ctx = format!("{page}: Beispiel ```{tag}\n{src}");
        if flags.contains(&"fehler") { assert!(res.is_err(), "{ctx}soll fehlschlagen"); n += 1; continue; }
        let out = res.unwrap_or_else(|e| panic!("{ctx}{:?}", e.iter().map(|d| d.code).collect::<Vec<_>>()));
        if flags.contains(&"warnung") { assert!(!out.warnings.is_empty(), "{ctx}soll warnen"); }
        let want: Vec<u8> = src.lines().filter_map(|l| l.strip_prefix(";= ")).flat_map(|h| h.split_whitespace().map(|b| u8::from_str_radix(b, 16).unwrap()).collect::<Vec<_>>()).collect();
        if !want.is_empty() && !src.contains(";=*") {
            let got: Vec<u8> = match flags.iter().find_map(|f| f.strip_prefix("fmt=")) {
                Some("prg") => vc_asm::to_prg(&out).unwrap(),
                Some("xex") => vc_asm::to_xex(&out).unwrap(),
                _ => out.segments.iter().flat_map(|s| s.bytes.clone()).collect(),
            };
            if got != want { eprintln!("ABWEICHUNG {ctx} {got:02x?}"); bad += 1; }
        }
        n += 1;
    }
    assert_eq!(bad, 0, "{page}");
    n
}

#[test]
fn directive_pages_examples_build_and_give_the_stated_bytes() {
    let n = check("asm-directives.md") + check("asm-text.md");
    assert!(n > 40, "{n} Beispiele");
}
