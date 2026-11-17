// SPDX-License-Identifier: GPL-3.0-or-later
//! Die Namenstabellen der Hilfeseite „Text“ (`asm-text.md`) stimmen mit der gemeinsamen Namensquelle `data/chars/` überein.
use vc_core::chars::{self, Family, Lang};

fn rows(md: &str, from: &str, to: &str) -> Vec<(Vec<String>, Vec<u8>)> {
    let a = md.find(from).expect(from);
    let b = md[a..].find(to).map_or(md.len(), |i| a + i);
    md[a..b].lines().filter(|l| l.starts_with("| `{")).map(|l| {
        let cells: Vec<&str> = l.split('|').map(|c| c.trim()).collect();
        let names = cells[1].split('`').filter(|t| t.starts_with('{')).map(|t| t.trim_matches(|c| c == '{' || c == '}').to_string()).collect();
        let codes = cells[2].split('`').filter(|t| t.starts_with('$')).map(|t| u8::from_str_radix(&t[1..], 16).unwrap()).collect();
        (names, codes)
    }).collect()
}

fn check(md: &str, from: &str, to: &str, fam: Family) {
    let mut seen = vec![];
    for (names, codes) in rows(md, from, to) {
        assert_eq!(names.len(), codes.len(), "{names:?} {codes:?}");
        for (n, c) in names.iter().zip(&codes) {
            assert_eq!(chars::lookup(fam, n), Some(*c), "{{{n}}} in der Hilfe");
            seen.push(*c);
        }
    }
    // jedes Zeichen der Tabelle steht in der Hilfe (mit seinem Hauptnamen des Assemblers)
    for c in chars::table(fam) {
        assert!(seen.contains(&c.code), "{{{}}} (${:02X}) fehlt in der Hilfeseite", c.main(Lang::Asm), c.code);
    }
}

#[test]
fn help_tables_match_the_shared_names() {
    let md = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("help/asm-text.md")).unwrap();
    check(&md, "### Namen für PETSCII", "### Namen für ATASCII", Family::Petscii);
    check(&md, "### Namen für ATASCII", "Weiter mit:", Family::Atascii);
}
