// SPDX-License-Identifier: GPL-3.0-or-later
//! Panel „Zeichen“ (Inspektor): Modell ohne Oberfläche. Die Liste der Steuerzeichen mit Gruppe, Farbe und Beschreibung, und die
//! Regeln, in welcher Form ein Klick den Code an der Cursorstelle einfügt (`{clr}`, `"{clr}"`, `147`, `$93`).
//! Die Namen kommen noch aus den bestehenden Tabellen (Assembler: `vc_asm::tokens`, BASIC: Steuercodes des Dialekts);
//! eine gemeinsame Namensquelle ist Stufe 2 des Konzepts.
use crate::lang::Lang;
use vc_core::chars::{self, Family, Lang as CLang};

pub use vc_core::chars::Group;

/// Ein Steuerzeichen: Hauptname in der Sprache, weitere Namen, Code, Beschreibung
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub aliases: Vec<String>,
    pub code: u8,
    pub desc: String,
    pub group: Group,
    /// Farbnummer 0 bis 15 (C64-Reihenfolge) bei den Farbcodes
    pub color: Option<usize>,
}

/// Liste der Steuerzeichen aus der gemeinsamen Namensquelle (`data/chars/`). Hauptname ist der der Sprache
/// (BASIC: `orng`, Assembler: `orange`); alle anderen Schreibweisen stehen als Aliase dabei.
pub fn entries(atari: bool, lang: Lang) -> Vec<Entry> {
    let cl = if lang == Lang::Basic { CLang::Basic } else { CLang::Asm };
    let mut v: Vec<Entry> = chars::table(if atari { Family::Atascii } else { Family::Petscii }).iter().map(|c| {
        let names = c.names(cl);
        Entry { name: names[0].to_string(), aliases: names[1..].iter().map(|s| s.to_string()).collect(), code: c.code, desc: c.desc.clone(), group: c.group, color: c.color.map(|x| x as usize) }
    }).collect();
    v.sort_by_key(|e| (e.group, e.color.unwrap_or(99), e.code));
    v
}

/// Wo steht der Cursor? Bestimmt die Vorgabeform beim Einfügen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot { InString, AfterPrint, InChr, PokeValue, Rem, Other }

/// Stelle des Cursors in `line` (Text vor dem Cursor: `before`). `in_text_string` = Assembler: in der Zeichenkette einer Textdirektive.
pub fn spot(lang: Lang, before: &str, in_text_string: bool) -> Spot {
    if lang != Lang::Basic { return if in_text_string { Spot::InString } else { Spot::Other }; }
    // außerhalb von Anführungszeichen: letzte Anweisung (nach `:`) und ob REM oder `'` davor steht
    let (mut quote, mut stmt) = (false, String::new());
    for c in before.chars() {
        if c == '"' { quote = !quote; stmt.push(c); continue; }
        if quote { stmt.push(c); continue; }
        if c == ':' { stmt.clear(); continue; }
        stmt.push(c);
    }
    if quote { return Spot::InString; }
    let s = stmt.trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ').to_ascii_uppercase();
    if s.starts_with("REM") || s.starts_with('\'') { return Spot::Rem; }
    let compact: String = s.chars().filter(|c| *c != ' ').collect();
    if compact.ends_with("CHR$(") { return Spot::InChr; }
    // `POKE adresse,` : der Wert kommt als Nächstes (bei Farbcodes die Farbnummer)
    if compact.starts_with("POKE") && compact.contains(',') && !compact.contains(')') { return Spot::PokeValue; }
    if compact.starts_with("PRINT") || compact.starts_with('?') { return Spot::AfterPrint; }
    Spot::Other
}

/// Form, in der ein Code eingefügt wird
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// `{clr}`
    Name,
    /// `"{clr}"` (BASIC, nach PRINT)
    Quoted,
    /// `147`
    Dec,
    /// `$93`
    Hex,
    /// `CHR$(147)`
    Chr,
    /// Farbnummer 0 bis 15 (bei Farbcodes, für `POKE 646,2`)
    ColorNo,
}

/// Vorgabeform an der Cursorstelle; `None` = nicht einfügen (Kommentar: `{…}` wirkt dort nur beim LIST)
pub fn default_form(lang: Lang, spot: Spot) -> Option<Form> {
    match (lang == Lang::Basic, spot) {
        (_, Spot::InString) => Some(Form::Name),
        (true, Spot::AfterPrint) => Some(Form::Quoted),
        (true, Spot::InChr) | (true, Spot::PokeValue) => Some(Form::Dec),
        (true, Spot::Rem) => None,
        (true, Spot::Other) => Some(Form::Dec),
        (false, _) => Some(Form::Hex),
    }
}

/// Text für einen Code in der gewählten Form
pub fn render(form: Form, e: &Entry) -> String {
    match form {
        Form::Name => format!("{{{}}}", e.name),
        Form::Quoted => format!("\"{{{}}}\"", e.name),
        Form::Dec => e.code.to_string(),
        Form::Hex => format!("${:02x}", e.code),
        Form::Chr => format!("CHR$({})", e.code),
        Form::ColorNo => e.color.map_or_else(|| e.code.to_string(), |n| n.to_string()),
    }
}

/// Eintrag, dessen Name oder Alias `name` ist (klein, `_` und `-` wie Leerzeichen)
/// Treffer der Suche: ein Steuerzeichen der Liste (Index) oder ein Code der Zeichentabelle
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hit { Control(usize), Code(u8) }

/// Code aus einer Eingabe: `$c1`, `0xc1`, `%11000001` oder Dezimalzahl (0 bis 255)
pub fn parse_code(q: &str) -> Option<u8> {
    let q = q.trim();
    if let Some(h) = q.strip_prefix('$').or_else(|| q.strip_prefix("0x")).or_else(|| q.strip_prefix("0X")) { return u8::from_str_radix(h, 16).ok(); }
    if let Some(b) = q.strip_prefix('%') { return u8::from_str_radix(b, 2).ok(); }
    q.parse::<u8>().ok()
}

/// Suche nach Name, Beschreibung, Zahl (`$c1`, `193`) oder einem getippten Zeichen (`A`). Steuerzeichen stehen vor den Tabellencodes.
pub fn search(list: &[Entry], atari: bool, q: &str) -> Vec<Hit> {
    let q = q.trim().trim_matches(|c| c == '{' || c == '}').trim();
    if q.is_empty() { return vec![]; }
    let mut out: Vec<Hit> = vec![];
    let mut add = |h: Hit| if !out.contains(&h) { out.push(h); };
    let num = parse_code(q);
    if let Some(c) = num { if let Some(i) = list.iter().position(|e| e.code == c) { add(Hit::Control(i)); } }
    let (norm, low) = (chars::normalize(q), q.to_lowercase());
    // ein einzelnes Zeichen ist ein getipptes Zeichen (unten), kein Namensteil
    let single = q.chars().count() == 1;
    if !single {
        for (i, e) in list.iter().enumerate() {
            let names = std::iter::once(&e.name).chain(e.aliases.iter()).any(|n| chars::normalize(n).contains(&norm));
            if names || e.desc.to_lowercase().contains(&low) { add(Hit::Control(i)); }
        }
    }
    if let Some(c) = num { add(Hit::Code(c)); }
    let mut chars = q.chars();
    if let (Some(ch), None) = (chars.next(), chars.next()) {
        for code in 0..=255u8 { if crate::chartext::literal_of(atari, 0, code) == Some(ch) { add(Hit::Code(code)); } }
    }
    out
}

/// Text für einen Code der Zeichentabelle an der Cursorstelle; `None` = nicht einfügen (BASIC-Kommentar).
/// `tab` 0 = PETSCII/ATASCII, 1 = Bildschirmcode bzw. ANTIC-Code (dann nur als Wert: Dezimal im BASIC, `$xx` im Assembler).
/// `spot` `None` = keine Datei (der Text wird kopiert, dann wie in einer Zeichenkette).
pub fn code_text(atari: bool, tab: usize, lang: Lang, spot: Option<Spot>, code: u8) -> Option<String> { codes_text(atari, tab, lang, spot, &[code]) }

/// Wie [`code_text`] für mehrere Codes hintereinander (Sequenzen): in einer Zeichenkette `{clr}{wht}`, nach PRINT `"{clr}{wht}"`,
/// sonst im BASIC `CHR$(147)+CHR$(5)` bzw. im Assembler `$93, $05`; in `CHR$(` verbindet `)+CHR$(` die Codes.
pub fn codes_text(atari: bool, tab: usize, lang: Lang, spot: Option<Spot>, codes: &[u8]) -> Option<String> {
    if codes.is_empty() || spot == Some(Spot::Rem) { return None; }
    let is_basic = lang == Lang::Basic;
    let nums = |sep: &str| codes.iter().map(|c| if is_basic { c.to_string() } else { format!("${c:02x}") }).collect::<Vec<_>>().join(sep);
    if tab == 1 { return Some(nums(", ")); }
    let fam = if atari { Family::Atascii } else { Family::Petscii };
    let in_string = crate::chartext::compose_ex(atari, 0, crate::chartext::Mode::InString, codes, is_basic);
    Some(match spot {
        None | Some(Spot::InString) => in_string,
        Some(Spot::AfterPrint) => format!("\"{in_string}\""),
        Some(Spot::InChr) if is_basic => nums(")+CHR$("),
        Some(Spot::PokeValue) => codes.iter().map(|c| chars::by_code(fam, *c).and_then(|x| x.color).map_or_else(|| c.to_string(), |n| n.to_string())).collect::<Vec<_>>().join(", "),
        Some(_) if is_basic && codes.len() > 1 => codes.iter().map(|c| format!("CHR$({c})")).collect::<Vec<_>>().join("+"),
        _ => nums(", "),
    })
}

/// Was aus der Liste per Ziehen in den Editor fällt: Codes der Familie; der Editor entscheidet an der Ablagestelle über die Form
#[derive(Clone, Debug, PartialEq)]
pub struct CharDrag { pub atari: bool, pub tab: usize, pub codes: Vec<u8> }

/// Text für eine abgelegte Zeichenfolge an Spalte `col` (Zeichenindex) der Zeile `line`
pub fn drop_text(lang: Lang, line: &str, col: usize, d: &CharDrag) -> Option<String> {
    let before: String = line.chars().take(col).collect();
    let in_text = lang == Lang::Asm && crate::charesc::asm_text_line(line) && before.matches('"').count() % 2 == 1;
    codes_text(d.atari, d.tab, lang, Some(spot(lang, &before, in_text)), &d.codes)
}

#[cfg(test)]
pub fn find<'a>(list: &'a [Entry], name: &str) -> Option<&'a Entry> {
    let n = chars::normalize(name);
    list.iter().find(|e| chars::normalize(&e.name) == n || e.aliases.iter().any(|a| chars::normalize(a) == n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembler_list_has_every_name_once_and_groups() {
        let v = entries(false, Lang::Asm);
        assert_eq!(v.len(), chars::table(Family::Petscii).len());
        let clr = find(&v, "clr").unwrap();
        assert_eq!((clr.code, clr.group, clr.color), (0x93, Group::Edit, None));
        assert_eq!(find(&v, "red").unwrap().color, Some(2));
        assert_eq!(find(&v, "f5").unwrap().group, Group::FKey);
        // Farben stehen in Palettenreihenfolge am Anfang
        let colors: Vec<usize> = v.iter().take_while(|e| e.group == Group::Color).map(|e| e.color.unwrap()).collect();
        assert_eq!(colors, (0..16).collect::<Vec<_>>());
    }

    #[test]
    fn basic_list_uses_the_basic_main_names_and_keeps_all_spellings() {
        let v = entries(false, Lang::Basic);
        let o = find(&v, "orng").unwrap();                       // BASIC schreibt `orng`, der Assembler `orange`
        assert_eq!((o.name.as_str(), o.code, o.color), ("orng", 0x81, Some(8)));
        assert!(o.aliases.contains(&"orange".to_string()));
        assert_eq!(find(&v, "orange").unwrap().code, 0x81, "auch die Assembler-Schreibweise findet den Eintrag");
        let rv = find(&v, "rvon").unwrap();                      // Alias zu `rvs on`
        assert_eq!((rv.name.as_str(), rv.code), ("rvs on", 0x12));
        assert_eq!(v.iter().filter(|e| e.code == 0x12).count(), 1);
        assert_eq!(find(&v, "stop").unwrap().desc, "STOP-Taste");
        assert_eq!(entries(false, Lang::Asm).len(), v.len(), "beide Sprachen zeigen dieselben Zeichen");
    }

    #[test]
    fn atari_list() {
        let v = entries(true, Lang::Basic);
        assert_eq!(v.len(), chars::table(Family::Atascii).len());
        assert!(v.iter().all(|e| e.color.is_none()));
        assert_eq!(find(&v, "clr").unwrap().group, Group::Edit);
        assert_eq!(find(&v, "up").unwrap().group, Group::Cursor);
        assert_eq!(find(&v, "eol").unwrap().desc, "Zeilenende");
        assert_eq!(find(&v, "del").unwrap().code, 0x7E, "Name des Dialekts bleibt gültig");
    }

    #[test]
    fn spot_in_basic_lines() {
        let b = |s: &str| spot(Lang::Basic, s, false);
        assert_eq!(b("10 PRINT \"HALLO"), Spot::InString);
        assert_eq!(b("10 PRINT "), Spot::AfterPrint);
        assert_eq!(b("10 ? "), Spot::AfterPrint);
        assert_eq!(b("10 PRINT\""), Spot::InString);
        assert_eq!(b("10 PRINT \"A\";"), Spot::AfterPrint);
        assert_eq!(b("10 A$=CHR$("), Spot::InChr);
        assert_eq!(b("10 A$=CHR$ ("), Spot::InChr);
        assert_eq!(b("10 REM "), Spot::Rem);
        assert_eq!(b("10 REM TEXT \"X"), Spot::InString);   // Anführungszeichen im Kommentar zählen mit (das Ergebnis ist dort ohnehin unkritisch)
        assert_eq!(b("10 GOTO 20:PRINT "), Spot::AfterPrint);
        assert_eq!(b("10 PRINT \"A\":X="), Spot::Other);
        assert_eq!(b("10 POKE 646,"), Spot::PokeValue);
        assert_eq!(b(""), Spot::Other);
    }

    #[test]
    fn spot_in_assembler() {
        assert_eq!(spot(Lang::Asm, "  .petscii \"", true), Spot::InString);
        assert_eq!(spot(Lang::Asm, "  lda #", false), Spot::Other);
    }

    #[test]
    fn default_forms() {
        use Form::*;
        assert_eq!(default_form(Lang::Basic, Spot::InString), Some(Name));
        assert_eq!(default_form(Lang::Basic, Spot::AfterPrint), Some(Quoted));
        assert_eq!(default_form(Lang::Basic, Spot::InChr), Some(Dec));
        assert_eq!(default_form(Lang::Basic, Spot::Rem), None);
        assert_eq!(default_form(Lang::Basic, Spot::Other), Some(Dec));
        assert_eq!(default_form(Lang::Asm, Spot::InString), Some(Name));
        assert_eq!(default_form(Lang::Asm, Spot::Other), Some(Hex));
    }

    #[test]
    fn search_by_name_number_and_character() {
        let v = entries(false, Lang::Asm);
        let names = |q: &str| -> Vec<String> { search(&v, false, q).into_iter().map(|h| match h { Hit::Control(i) => format!("{{{}}}", v[i].name), Hit::Code(c) => format!("${c:02x}") }).collect() };
        assert_eq!(names("clr"), ["{clr}"]);
        assert_eq!(names("{CLR}"), ["{clr}"], "Klammern und Großschreibung stören nicht");
        assert_eq!(names("rvs"), ["{rvs on}", "{rvs off}"]);
        assert_eq!(names("$93"), ["{clr}", "$93"], "Zahl: das Steuerzeichen und der Code der Tabelle");
        assert_eq!(names("147"), ["{clr}", "$93"]);
        assert_eq!(names("0x93"), ["{clr}", "$93"]);
        assert_eq!(names("türkis"), ["{cyn}"], "Beschreibung");
        assert_eq!(names("A"), ["$c1"], "getipptes Zeichen: A mit Umschalt");
        assert_eq!(names("a"), ["$41"]);
        assert_eq!(names("1"), ["$01", "$31"], "Zahl und Zeichen");
        assert!(names("").is_empty() && names("   ").is_empty() && names("zzzz").is_empty());
        // Atari: Namen der ATASCII-Tabelle
        let a = entries(true, Lang::Asm);
        assert!(search(&a, true, "eol").contains(&Hit::Control(a.iter().position(|e| e.name == "eol").unwrap())));
    }

    #[test]
    fn table_codes_follow_the_cursor_spot() {
        let b = |tab, spot, code| code_text(false, tab, Lang::Basic, spot, code);
        assert_eq!(b(0, Some(Spot::InString), 0xC1).as_deref(), Some("A"));
        assert_eq!(b(0, Some(Spot::InString), 0x93).as_deref(), Some("{clr}"));
        assert_eq!(b(0, Some(Spot::InString), 0xA1).as_deref(), Some("{$a1}"));
        assert_eq!(b(0, Some(Spot::AfterPrint), 0xC1).as_deref(), Some("\"A\""));
        assert_eq!(b(0, Some(Spot::AfterPrint), 0x93).as_deref(), Some("\"{clr}\""));
        assert_eq!(b(0, Some(Spot::InChr), 0x93).as_deref(), Some("147"));
        assert_eq!(b(0, Some(Spot::Other), 0xC1).as_deref(), Some("193"));
        assert_eq!(b(0, Some(Spot::Rem), 0xC1), None);
        assert_eq!(b(1, Some(Spot::InString), 0x01).as_deref(), Some("1"), "Bildschirmcode: nur der Wert (POKE)");
        assert_eq!(b(0, None, 0x93).as_deref(), Some("{clr}"), "ohne Datei: wie in einer Zeichenkette (Zwischenablage)");
        assert_eq!(code_text(false, 0, Lang::Asm, Some(Spot::Other), 0xC1).as_deref(), Some("$c1"));
        assert_eq!(code_text(false, 1, Lang::Asm, Some(Spot::Other), 0x01).as_deref(), Some("$01"));
        assert_eq!(code_text(false, 0, Lang::Asm, Some(Spot::InString), 0x93).as_deref(), Some("{clr}"));
        assert_eq!(code_text(true, 0, Lang::Basic, Some(Spot::AfterPrint), 0x7D).as_deref(), Some("\"{clr}\""));
    }

    #[test]
    fn poke_values_use_the_colour_number() {
        assert_eq!(spot(Lang::Basic, "10 POKE 646,", false), Spot::PokeValue);
        assert_eq!(spot(Lang::Basic, "10 POKE 53280,", false), Spot::PokeValue);
        assert_eq!(spot(Lang::Basic, "10 POKE 646", false), Spot::Other, "noch kein Komma");
        assert_eq!(spot(Lang::Basic, "10 POKE PEEK(1),", false), Spot::Other, "Klammer in der Adresse: keine Annahme");
        assert_eq!(code_text(false, 0, Lang::Basic, Some(Spot::PokeValue), 0x1C).as_deref(), Some("2"), "red = Farbe 2");
        assert_eq!(code_text(false, 0, Lang::Basic, Some(Spot::PokeValue), 0x81).as_deref(), Some("8"), "orng = Farbe 8");
        assert_eq!(code_text(false, 0, Lang::Basic, Some(Spot::PokeValue), 0x93).as_deref(), Some("147"), "kein Farbcode: Code");
        let e = Entry { name: "red".into(), aliases: vec![], code: 0x1C, desc: String::new(), group: Group::Color, color: Some(2) };
        assert_eq!(render(Form::ColorNo, &e), "2");
    }

    #[test]
    fn sequences_follow_the_cursor_spot() {
        let c = |sp: Option<Spot>, l: Lang| codes_text(false, 0, l, sp, &[0x93, 0x05, 0x41]);
        assert_eq!(c(Some(Spot::InString), Lang::Basic).as_deref(), Some("{clr}{wht}a"));
        assert_eq!(c(Some(Spot::AfterPrint), Lang::Basic).as_deref(), Some("\"{clr}{wht}a\""));
        assert_eq!(c(Some(Spot::Other), Lang::Basic).as_deref(), Some("CHR$(147)+CHR$(5)+CHR$(65)"));
        assert_eq!(c(Some(Spot::InChr), Lang::Basic).as_deref(), Some("147)+CHR$(5)+CHR$(65"));
        assert_eq!(c(Some(Spot::Other), Lang::Asm).as_deref(), Some("$93, $05, $41"));
        assert_eq!(c(None, Lang::Basic).as_deref(), Some("{clr}{wht}a"), "ohne Datei wie in einer Zeichenkette");
        assert_eq!(c(Some(Spot::Rem), Lang::Basic), None);
        assert_eq!(codes_text(false, 0, Lang::Basic, Some(Spot::Other), &[]), None);
        assert_eq!(codes_text(false, 1, Lang::Basic, Some(Spot::Other), &[1, 2]).as_deref(), Some("1, 2"));
    }

    #[test]
    fn dropped_codes_take_the_form_of_the_drop_position() {
        let d = CharDrag { atari: false, tab: 0, codes: vec![0x93] };
        let line = "10 PRINT \"AB\"";
        assert_eq!(drop_text(Lang::Basic, line, 11, &d).as_deref(), Some("{clr}"), "zwischen den Anführungszeichen");
        assert_eq!(drop_text(Lang::Basic, line, 9, &d).as_deref(), Some("\"{clr}\""), "nach PRINT, vor der Zeichenkette");
        assert_eq!(drop_text(Lang::Basic, "10 REM x", 8, &d), None, "Kommentar");
        assert_eq!(drop_text(Lang::Asm, "  .petscii \"ab\"", 13, &d).as_deref(), Some("{clr}"));
        assert_eq!(drop_text(Lang::Asm, "  lda #0", 8, &d).as_deref(), Some("$93"));
    }

    #[test]
    fn rendered_texts() {
        let e = Entry { name: "clr".into(), aliases: vec![], code: 0x93, desc: String::new(), group: Group::Edit, color: None };
        assert_eq!(render(Form::Name, &e), "{clr}");
        assert_eq!(render(Form::Quoted, &e), "\"{clr}\"");
        assert_eq!(render(Form::Dec, &e), "147");
        assert_eq!(render(Form::Hex, &e), "$93");
        assert_eq!(render(Form::Chr, &e), "CHR$(147)");
    }

    /// Goldene Tabelle: jeder Name der Liste wird von BASIC (alle Dialekte) und Assembler wieder in denselben Code übersetzt
    #[test]
    fn every_name_is_understood_by_both_languages() {
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        for id in ["cbm_v2", "basic7", "basic35", "basic65"] {
            let d = vc_basic::Dialect::load(&data, id).unwrap();
            for lang in [Lang::Basic, Lang::Asm] {
                for e in entries(false, lang) {
                    for n in std::iter::once(&e.name).chain(e.aliases.iter()) {
                        assert_eq!(vc_basic::petscii::from_text(&format!("{{{n}}}"), &d).unwrap(), vec![e.code], "BASIC {id} {{{n}}}");
                        assert_eq!(vc_asm::tokens::lookup(vc_asm::tokens::Names::Petscii, n), Some(e.code), "Assembler {{{n}}}");
                    }
                }
            }
        }
        for e in entries(true, Lang::Asm) {
            for n in std::iter::once(&e.name).chain(e.aliases.iter()) {
                assert_eq!(vc_asm::tokens::lookup(vc_asm::tokens::Names::Atascii, n), Some(e.code), "ATASCII {{{n}}}");
            }
        }
    }
}
