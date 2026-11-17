// SPDX-License-Identifier: GPL-3.0-or-later
//! Steuerzeichen von PETSCII und ATASCII: eine Namensquelle (`data/chars/*.json`) für Assembler, BASIC und das Zeichen-Panel.
//! Die Dateien sind ins Programm eingebettet (der Assembler braucht die Namen ohne Datenordner); `data/chars/` bleibt die Quelle.
//! Jede Sprache hat ihre Hauptnamen (`orange` im Assembler, `orng` im BASIC); alle Namen gelten in beiden Sprachen.
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family { Petscii, Atascii }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang { Asm, Basic }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Group { Color, Cursor, Edit, Charset, FKey, Other }

impl Group {
    pub const ALL: [Group; 6] = [Group::Color, Group::Cursor, Group::Edit, Group::Charset, Group::FKey, Group::Other];
    pub fn title(self) -> &'static str {
        match self { Group::Color => "Farben", Group::Cursor => "Cursor", Group::Edit => "Bildschirm und Bearbeiten", Group::Charset => "Zeichensatz", Group::FKey => "Funktionstasten", Group::Other => "Weitere" }
    }
    fn parse(s: &str) -> Group {
        match s { "color" => Group::Color, "cursor" => Group::Cursor, "edit" => Group::Edit, "charset" => Group::Charset, "fkey" => Group::FKey, _ => Group::Other }
    }
}

/// Ein Steuerzeichen
#[derive(Clone, Debug, PartialEq)]
pub struct Char {
    pub code: u8,
    /// Hauptname im Assembler
    pub name: String,
    /// Hauptname im BASIC, wenn er vom Assemblernamen abweicht
    pub basic: Option<String>,
    pub aliases: Vec<String>,
    pub group: Group,
    /// Farbnummer 0 bis 15 (C64-Reihenfolge) bei den Farbcodes
    pub color: Option<u8>,
    pub desc: String,
}

impl Char {
    /// Hauptname in der Sprache
    pub fn main(&self, lang: Lang) -> &str { match lang { Lang::Asm => &self.name, Lang::Basic => self.basic.as_deref().unwrap_or(&self.name) } }
    /// Alle Namen: Hauptname der Sprache zuerst, dann die übrigen
    pub fn names(&self, lang: Lang) -> Vec<&str> {
        let main = self.main(lang);
        let mut v = vec![main];
        for n in std::iter::once(self.name.as_str()).chain(self.basic.as_deref()).chain(self.aliases.iter().map(|s| s.as_str())) { if !v.contains(&n) { v.push(n); } }
        v
    }
}

fn parse(text: &str, what: &str) -> Vec<Char> {
    let v: Value = serde_json::from_str(text).unwrap_or_else(|e| panic!("data/chars/{what}.json: {e}"));
    v["entries"].as_array().unwrap_or_else(|| panic!("data/chars/{what}.json: entries fehlt")).iter().map(|e| {
        let s = |k: &str| e[k].as_str().map(|s| s.to_string());
        let code = e["code"].as_str().and_then(|c| c.strip_prefix('$')).and_then(|h| u8::from_str_radix(h, 16).ok()).unwrap_or_else(|| panic!("data/chars/{what}.json: code ungültig in {e}"));
        Char {
            code,
            name: s("name").unwrap_or_else(|| panic!("data/chars/{what}.json: name fehlt bei ${code:02x}")),
            basic: s("basic"),
            aliases: e["aliases"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
            group: Group::parse(e["group"].as_str().unwrap_or("other")),
            color: e["color"].as_u64().map(|c| c as u8),
            desc: s("desc").unwrap_or_default(),
        }
    }).collect()
}

/// Tabelle der Steuerzeichen, nach Code sortiert
pub fn table(f: Family) -> &'static [Char] {
    static PETSCII: OnceLock<Vec<Char>> = OnceLock::new();
    static ATASCII: OnceLock<Vec<Char>> = OnceLock::new();
    match f {
        Family::Petscii => PETSCII.get_or_init(|| parse(include_str!("../../../data/chars/petscii.json"), "petscii")),
        Family::Atascii => ATASCII.get_or_init(|| parse(include_str!("../../../data/chars/atascii.json"), "atascii")),
    }
}

/// Name vereinheitlichen: klein, `_` und `-` wie Leerzeichen, mehrfache Leerzeichen zusammenfassen
pub fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['_', '-'], " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Eintrag zu einem Namen (Hauptname oder Alias in beiden Sprachen)
pub fn find(f: Family, name: &str) -> Option<&'static Char> {
    let n = normalize(name);
    table(f).iter().find(|c| c.names(Lang::Asm).iter().any(|x| normalize(x) == n))
}

pub fn lookup(f: Family, name: &str) -> Option<u8> { find(f, name).map(|c| c.code) }

pub fn by_code(f: Family, code: u8) -> Option<&'static Char> { table(f).iter().find(|c| c.code == code) }

/// Das Innere einer `{…}`-Angabe in Zeichenketten: `name`, `$hex`, `nnn` (Dezimal), je mit `*n` (1 bis 255).
/// `extra` löst Namen zuerst auf (BASIC: Steuercodes des Dialekts); danach gilt die gemeinsame Tabelle.
/// Ergebnis: (Code, Wiederholungen).
pub fn parse_ref(f: Family, inner: &str, extra: &dyn Fn(&str) -> Option<u8>) -> Result<(u8, usize), String> {
    let (base, count) = match inner.rsplit_once('*') {
        Some((a, n)) => (a, n.trim().parse::<usize>().ok().filter(|n| (1..=255).contains(n)).ok_or(format!("ungültige Anzahl in {{{inner}}}"))?),
        None => (inner, 1),
    };
    let base = base.trim();
    let code = if let Some(h) = base.strip_prefix('$') {
        u8::from_str_radix(h, 16).map_err(|_| format!("ungültiger Code {{{inner}}}"))?
    } else if !base.is_empty() && base.bytes().all(|c| c.is_ascii_digit()) {
        base.parse::<u8>().map_err(|_| format!("ungültiger Code {{{inner}}}"))?
    } else {
        let n = normalize(base);
        extra(&n).or_else(|| lookup(f, &n)).ok_or_else(|| format!("unbekannter Steuercode {{{base}}}"))?
    };
    Ok((code, count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_load_sorted_and_without_duplicate_codes_or_names() {
        for f in [Family::Petscii, Family::Atascii] {
            let t = table(f);
            assert!(t.len() > 10);
            assert!(t.windows(2).all(|w| w[0].code < w[1].code), "{f:?}: nach Code sortiert, jeder Code einmal");
            let mut names: Vec<String> = t.iter().flat_map(|c| c.names(Lang::Asm).into_iter().map(normalize)).collect();
            let n = names.len();
            names.sort(); names.dedup();
            assert_eq!(names.len(), n, "{f:?}: jeder Name gehört genau einem Zeichen");
        }
    }

    #[test]
    fn every_name_in_either_language_finds_the_code() {
        assert_eq!(lookup(Family::Petscii, "orange"), Some(0x81));
        assert_eq!(lookup(Family::Petscii, "orng"), Some(0x81));
        assert_eq!(lookup(Family::Petscii, "Rvs_On"), Some(0x12));
        assert_eq!(lookup(Family::Petscii, "rvon"), Some(0x12));
        assert_eq!(lookup(Family::Petscii, "shift-return"), Some(0x8D));
        assert_eq!(lookup(Family::Petscii, "  shift   ret "), Some(0x8D));
        assert_eq!(lookup(Family::Petscii, "pound"), Some(0x5C));
        assert_eq!(lookup(Family::Atascii, "bs"), Some(0x7E));
        assert_eq!(lookup(Family::Atascii, "del"), Some(0x7E));
        assert_eq!(lookup(Family::Atascii, "wht"), None, "PETSCII-Namen gelten nicht im ATASCII");
        let o = find(Family::Petscii, "orange").unwrap();
        assert_eq!((o.main(Lang::Asm), o.main(Lang::Basic), o.color), ("orange", "orng", Some(8)));
        assert_eq!(o.names(Lang::Basic), ["orng", "orange"]);
    }

    #[test]
    fn colors_are_the_sixteen_c64_colors_once() {
        let mut c: Vec<u8> = table(Family::Petscii).iter().filter_map(|c| c.color).collect();
        c.sort();
        assert_eq!(c, (0..16).collect::<Vec<u8>>());
        assert!(table(Family::Petscii).iter().all(|c| (c.group == Group::Color) == c.color.is_some()));
    }

    #[test]
    fn reference_bodies() {
        let no = |_: &str| None;
        assert_eq!(parse_ref(Family::Petscii, "clr", &no), Ok((0x93, 1)));
        assert_eq!(parse_ref(Family::Petscii, "down*3", &no), Ok((0x11, 3)));
        assert_eq!(parse_ref(Family::Petscii, "$c1", &no), Ok((0xC1, 1)));
        assert_eq!(parse_ref(Family::Petscii, "147", &no), Ok((147, 1)));
        assert_eq!(parse_ref(Family::Petscii, " Rvs_On ", &no), Ok((0x12, 1)));
        assert_eq!(parse_ref(Family::Petscii, "x", &|n| (n == "x").then_some(7)), Ok((7, 1)), "Dialektname zuerst");
        assert!(parse_ref(Family::Petscii, "300", &no).is_err());
        assert!(parse_ref(Family::Petscii, "down*0", &no).is_err());
        assert!(parse_ref(Family::Petscii, "nix", &no).unwrap_err().contains("unbekannter Steuercode"));
    }
}
