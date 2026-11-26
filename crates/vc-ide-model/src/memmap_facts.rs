// SPDX-License-Identifier: GPL-3.0-or-later
//! Memory Map: Datenaufbereitung (Segmente aus `.vcdbg`, Belegung je Seite, Prüfungen, Seitenplan) – ohne Oberfläche.
use serde_json::Value;
use vc_target::{BankPreset, Page, PageCategory, PageKind};

/// Ein Segment des Programms (aus `.vcdbg`)
#[derive(Debug, Clone, PartialEq)]
pub struct Seg { pub start: u32, pub len: usize, pub bank: u32, pub name: String }

impl Seg {
    /// Letzte belegte Adresse (ein leeres Segment zählt als ein Byte)
    pub fn end(&self) -> u32 { self.start + self.len.max(1) as u32 - 1 }
}

/// Was die Matrix vom Programm weiß: Segmente und Symbole
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Facts { pub segs: Vec<Seg>, pub syms: Vec<(String, u32)> }

/// Liest Segmente und Symbole aus dem Text einer `.vcdbg`-Datei
pub fn parse_vcdbg(text: &str) -> Option<Facts> {
    let v: Value = serde_json::from_str(text).ok()?;
    let segs = v["segments"].as_array()?.iter().filter_map(|s| Some(Seg {
        start: s["start"].as_u64()? as u32, len: s["length"].as_u64()? as usize, bank: s["bank"].as_u64().unwrap_or(0) as u32, name: s["name"].as_str().unwrap_or("").into(),
    })).collect();
    let syms = v["symbols"].as_array().map(|a| a.iter().filter_map(|s| {
        let (n, val) = (s["name"].as_str()?, s["value"].as_i64()?);
        (0..0x10000).contains(&val).then(|| (n.to_string(), val as u32))
    }).collect()).unwrap_or_default();
    Some(Facts { segs, syms })
}

/// Belegte Bytes je Seite (nur Bank 0; überlappende Segmente zählen doppelt, höchstens 256)
pub fn page_usage(segs: &[Seg]) -> [u32; 256] {
    let mut u = [0u32; 256];
    for s in segs.iter().filter(|s| s.bank == 0) {
        let (a, b) = (s.start.min(0xFFFF), s.end().min(0xFFFF));
        for p in (a >> 8)..=(b >> 8) {
            let lo = a.max(p << 8);
            let hi = b.min((p << 8) + 255);
            u[p as usize] = (u[p as usize] + hi - lo + 1).min(256);
        }
    }
    u
}

/// Welche Bytes einer Seite das Programm belegt (Bank 0)
pub fn byte_usage(segs: &[Seg], page: u8) -> [bool; 256] {
    let mut u = [false; 256];
    let base = (page as u32) << 8;
    for s in segs.iter().filter(|s| s.bank == 0) {
        for b in s.start.max(base)..=s.end().min(base + 255) { if b >= base && b <= base + 255 { u[(b - base) as usize] = true; } }
    }
    u
}

/// Überlappungen von Segmenten derselben Bank: (Anfang, Ende) des gemeinsamen Bereichs
pub fn overlaps(segs: &[Seg]) -> Vec<(u32, u32)> {
    let mut v = vec![];
    for (i, a) in segs.iter().enumerate() {
        for b in &segs[i + 1..] {
            if a.bank != b.bank { continue; }
            let (lo, hi) = (a.start.max(b.start), a.end().min(b.end()));
            if lo <= hi { v.push((lo, hi)); }
        }
    }
    v
}

/// Quellstellen auf Seiten `first..=last`: (Datei, erste Zeile, letzte Zeile), je Datei ein Eintrag in Reihenfolge des Auftretens
pub fn page_sources(lines: &[(String, usize, u16, usize)], first: u8, last: u8) -> Vec<(String, usize, usize)> {
    let (lo, hi) = ((first as u32) << 8, ((last as u32) << 8) + 255);
    let mut out: Vec<(String, usize, usize)> = vec![];
    for (f, n, a, l) in lines {
        let (s, e) = (*a as u32, *a as u32 + (*l).max(1) as u32 - 1);
        if e < lo || s > hi { continue; }
        match out.iter_mut().find(|o| &o.0 == f) { Some(o) => { o.1 = o.1.min(*n); o.2 = o.2.max(*n); } None => out.push((f.clone(), *n, *n)) }
    }
    out
}

/// Symbole auf den Seiten `first..=last`, nach Adresse sortiert. Nur Symbole innerhalb eines Segments (Marken, keine Konstanten wie Chip-Register).
pub fn page_symbols(facts: &Facts, first: u8, last: u8) -> Vec<(String, u32)> {
    let (lo, hi) = ((first as u32) << 8, ((last as u32) << 8) + 255);
    let mut v: Vec<(String, u32)> = facts.syms.iter()
        .filter(|(n, a)| *a >= lo && *a <= hi && !n.contains(".__m") && facts.segs.iter().any(|s| s.bank == 0 && *a >= s.start && *a <= s.end()))
        .cloned().collect();
    v.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    v
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level { Ok, Info, Warn, Err }

#[derive(Debug, Clone, PartialEq)]
pub struct Check { pub level: Level, pub text: String }

pub fn range_text(a: u32, b: u32) -> String { format!("${a:04X}–${b:04X}") }

/// Prüfungen: Überlappung (Fehler), Stack (Warnung), Zeropage und Seiten unter ROM/E/A, geteilte Seiten (Hinweise)
pub fn checks(facts: &Facts, plan: Option<&[Page; 256]>) -> Vec<Check> {
    let mut v = vec![];
    let ov = overlaps(&facts.segs);
    if ov.is_empty() { v.push(Check { level: Level::Ok, text: vc_i18n::t!("memmap-facts-0-ueberlappungen").into() }); }
    else { v.push(Check { level: Level::Err, text: vc_i18n::t!("memmap-facts-p1-ueberlappung-en-p2", p1 = ov.len(), p2 = ov.iter().take(3).map(|(a, b)| range_text(*a, *b)).collect::<Vec<_>>().join(", ")) }); }
    let usage = page_usage(&facts.segs);
    if usage[1] > 0 { v.push(Check { level: Level::Warn, text: vc_i18n::t!("memmap-facts-programm-belegt-p1-byte-im-stack", p1 = usage[1]) }); }
    else { v.push(Check { level: Level::Ok, text: vc_i18n::t!("memmap-facts-stack-frei-seite-01").into() }); }
    if usage[0] > 0 { v.push(Check { level: Level::Info, text: vc_i18n::t!("memmap-facts-zeropage-p1-byte-belegt", p1 = usage[0]) }); }
    let shared: Vec<u8> = (0..=255u8).filter(|&p| facts.segs.iter().filter(|s| s.bank == 0 && (s.start >> 8) <= p as u32 && (s.end().min(0xFFFF) >> 8) >= p as u32).count() > 1).collect();
    if !shared.is_empty() { v.push(Check { level: Level::Info, text: vc_i18n::t!("memmap-facts-p1-seite-n-von-mehreren-segmenten", p1 = shared.len(), p2 = format!("{:02X}", shared[0])) }); }
    if let Some(plan) = plan {
        let hidden: Vec<u8> = (0..=255u8).filter(|&p| usage[p as usize] > 0 && plan[p as usize].kind != PageKind::Ram).collect();
        if !hidden.is_empty() { v.push(Check { level: Level::Info, text: vc_i18n::t!("memmap-facts-p1-seite-n-des-programms-liegen", p1 = hidden.len(), p2 = format!("{:02X}", hidden[0])) }); }
    }
    v
}

/// Ansicht der Matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View { Pages, Blocks, Bytes }

impl View {
    pub fn parse(s: &str) -> View { match s { "blocks" => View::Blocks, "bytes" => View::Bytes, _ => View::Pages } }
}

/// Filter nach Art
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter { All, Ram, Rom, Io, Program }

impl Filter {
    pub fn parse(s: &str) -> Filter { match s { "ram" => Filter::Ram, "rom" => Filter::Rom, "io" => Filter::Io, "prog" => Filter::Program, _ => Filter::All } }
    /// Passt die Seite (Art, belegte Bytes) zum Filter?
    pub fn matches(self, kind: PageKind, used: u32) -> bool {
        match self {
            Filter::All => true,
            Filter::Ram => kind == PageKind::Ram,
            Filter::Rom => matches!(kind, PageKind::Rom | PageKind::BasicRom | PageKind::KernalRom | PageKind::CharRom | PageKind::Module),
            Filter::Io => kind == PageKind::Io,
            Filter::Program => used > 0,
        }
    }
}

/// Seitenplan, wenn es keine Presets gibt: alles RAM, Seite 0 und 1 als Zeropage und Stack
pub fn plain_plan() -> [Page; 256] {
    let mut p = [Page { kind: PageKind::Ram, category: PageCategory::Free }; 256];
    for (i, x) in p.iter_mut().enumerate() { x.category = PageCategory::default_for(PageKind::Ram, i as u8); }
    p
}

pub fn kind_label(k: PageKind) -> String {
    match k { PageKind::Ram => "RAM".into(), PageKind::Rom => "ROM".into(), PageKind::BasicRom => "BASIC-ROM".into(), PageKind::KernalRom => vc_i18n::t!("memmap-facts-system-rom-kernal-os"), PageKind::CharRom => vc_i18n::t!("memmap-facts-zeichen-rom"), PageKind::Io => "E/A".into(), PageKind::Module => vc_i18n::t!("memmap-facts-modul") }
}

pub fn category_label(c: PageCategory) -> String {
    match c {
        PageCategory::Zeropage => vc_i18n::t!("memmap-facts-zeropage"), PageCategory::Stack => vc_i18n::t!("memmap-facts-stack"), PageCategory::Screen => vc_i18n::t!("memmap-facts-bildschirm"), PageCategory::Sprites => vc_i18n::t!("memmap-facts-sprites"), PageCategory::Charset => vc_i18n::t!("memmap-facts-zeichensatz"),
        PageCategory::Music => vc_i18n::t!("memmap-facts-musik"), PageCategory::Io => "E/A".into(), PageCategory::Rom => "ROM".into(), PageCategory::Basic => "BASIC".into(), PageCategory::Free => vc_i18n::t!("memmap-facts-frei-ram"),
    }
}

/// Kategorien, die im Seitenplan vorkommen, in fester Reihenfolge (für die Legende)
pub fn legend(plan: &[Page; 256]) -> Vec<PageCategory> {
    use PageCategory::*;
    [Zeropage, Stack, Screen, Sprites, Charset, Music, Basic, Rom, Io, Free].into_iter().filter(|c| plan.iter().any(|p| p.category == *c)).collect()
}

/// Presets der Liste als (Kennung, Beschriftung); Presets nur für ein Modell tragen „nur 130XE“ o. ä. im Namen
pub fn preset_label(p: &BankPreset) -> String {
    let m = if p.models.len() == 1 { format!(" {}", vc_i18n::t!("memmap-facts-nur-p1", p1 = p.models[0].to_uppercase())) } else { String::new() };
    format!("${:02X}  {}{}", p.value, p.name, m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn seg(start: u32, len: usize, bank: u32) -> Seg { Seg { start, len, bank, name: "Main".into() } }

    #[test]
    fn vcdbg_wird_gelesen() {
        let t = r#"{"format":1,"symbols":[{"name":"start","value":2064},{"name":"neg","value":-1},{"name":"hoch","value":70000}],
            "segments":[{"start":2049,"length":12,"bank":0,"name":"Main"},{"start":32768,"length":4,"bank":2,"name":"Rom"}],"lines":[]}"#;
        let f = parse_vcdbg(t).unwrap();
        assert_eq!(f.segs.len(), 2);
        assert_eq!(f.segs[1], Seg { start: 32768, len: 4, bank: 2, name: "Rom".into() });
        assert_eq!(f.syms, vec![("start".to_string(), 2064)]);
        assert!(parse_vcdbg("kein json").is_none());
    }

    #[test]
    fn belegung_je_seite() {
        // $07FE–$0802 liegt auf zwei Seiten: 2 Byte auf $07, 3 Byte auf $08
        let u = page_usage(&[seg(0x07FE, 5, 0)]);
        assert_eq!((u[7], u[8], u[6], u[9]), (2, 3, 0, 0));
        // Bank 1 zählt nicht, ein volles Segment füllt die Seiten ganz
        let u = page_usage(&[seg(0x2000, 0x200, 0), seg(0x4000, 100, 1)]);
        assert_eq!((u[0x20], u[0x21], u[0x22], u[0x40]), (256, 256, 0, 0));
        // Segment bis $FFFF
        assert_eq!(page_usage(&[seg(0xFFF0, 16, 0)])[0xFF], 16);
    }

    #[test]
    fn belegung_je_byte() {
        let b = byte_usage(&[seg(0x0805, 3, 0)], 8);
        assert!(!b[4] && b[5] && b[6] && b[7] && !b[8]);
        assert!(byte_usage(&[seg(0x0805, 3, 0)], 9).iter().all(|x| !x));
    }

    #[test]
    fn ueberlappung_nur_in_derselben_bank() {
        assert_eq!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x10F0, 0x20, 0)]), vec![(0x10F0, 0x10FF)]);
        assert!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x10F0, 0x20, 1)]).is_empty());
        assert!(overlaps(&[seg(0x1000, 0x100, 0), seg(0x1100, 0x20, 0)]).is_empty());
    }

    #[test]
    fn quellen_und_symbole_der_seite() {
        let lines = vec![("a.asm".to_string(), 3, 0x0800u16, 3), ("a.asm".to_string(), 4, 0x0803, 2), ("b.asm".to_string(), 9, 0x0900, 1), ("c.asm".to_string(), 1, 0x2000, 1)];
        assert_eq!(page_sources(&lines, 8, 8), vec![("a.asm".to_string(), 3, 4)]);
        assert_eq!(page_sources(&lines, 8, 9).len(), 2);
        assert!(page_sources(&lines, 0x30, 0x30).is_empty());
        let f = Facts { segs: vec![seg(0x0801, 0x20, 0)], syms: vec![("main".into(), 0x0810), ("SPRITE_PTRS".into(), 0x07F8), ("VIC.__m1".into(), 0x0812), ("init".into(), 0x0805)] };
        // Konstante außerhalb des Segments und interne Makro-Marken fallen weg
        assert_eq!(page_symbols(&f, 8, 8), vec![("init".to_string(), 0x0805), ("main".to_string(), 0x0810)]);
    }

    #[test]
    fn pruefungen() {
        let ok = Facts { segs: vec![seg(0x0801, 0x20, 0)], syms: vec![] };
        let c = checks(&ok, None);
        assert_eq!(c[0], Check { level: Level::Ok, text: "0 Überlappungen".into() });
        assert!(c.iter().any(|x| x.level == Level::Ok && x.text.contains("Stack frei")));
        let bad = Facts { segs: vec![seg(0x0100, 0x20, 0), seg(0x0110, 0x20, 0), seg(0x0010, 4, 0)], syms: vec![] };
        let c = checks(&bad, None);
        assert_eq!(c[0].level, Level::Err);
        assert!(c[0].text.contains("$0110–$011F"));
        assert!(c.iter().any(|x| x.level == Level::Warn && x.text.contains("Stack")));
        assert!(c.iter().any(|x| x.level == Level::Info && x.text.contains("Zeropage: 4")));
        assert!(c.iter().any(|x| x.text.contains("mehreren Segmenten")));
    }

    #[test]
    fn pruefung_unter_rom_nutzt_den_seitenplan() {
        let mut plan = plain_plan();
        for p in 0xA0..=0xBF { plan[p] = Page { kind: PageKind::BasicRom, category: PageCategory::Basic }; }
        let f = Facts { segs: vec![seg(0xA000, 0x100, 0), seg(0x0801, 4, 0)], syms: vec![] };
        let c = checks(&f, Some(&plan));
        assert!(c.iter().any(|x| x.text.contains("unter ROM oder E/A, z. B. $A0")));
        assert!(!checks(&f, None).iter().any(|x| x.text.contains("unter ROM")));
    }

    #[test]
    fn filter_nach_art() {
        assert!(Filter::All.matches(PageKind::Io, 0));
        assert!(Filter::Ram.matches(PageKind::Ram, 0) && !Filter::Ram.matches(PageKind::Io, 0));
        assert!(Filter::Rom.matches(PageKind::CharRom, 0) && Filter::Rom.matches(PageKind::BasicRom, 0) && !Filter::Rom.matches(PageKind::Ram, 0));
        assert!(Filter::Io.matches(PageKind::Io, 0));
        assert!(Filter::Program.matches(PageKind::Ram, 1) && !Filter::Program.matches(PageKind::Ram, 0));
        assert_eq!(Filter::parse("rom"), Filter::Rom);
        assert_eq!(View::parse("bytes"), View::Bytes);
    }

    fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn legende_und_plan_aus_den_zieldaten() {
        let t = vc_target::load_target(&data(), "c64").unwrap();
        let b = t.banking.as_ref().unwrap();
        let plan = b.page_plan(b.default_preset().unwrap());
        assert_eq!(b.default_preset().unwrap().value, 0x37);
        assert_eq!(plan[0xA0].kind, PageKind::BasicRom);
        assert_eq!(plan[0xD0].kind, PageKind::Io);
        assert_eq!(legend(&plan), vec![PageCategory::Zeropage, PageCategory::Stack, PageCategory::Basic, PageCategory::Rom, PageCategory::Io, PageCategory::Free]);
        // Alles RAM: weder BASIC noch E/A in der Legende
        let ram = b.page_plan(b.preset("30").unwrap());
        assert_eq!(legend(&ram), vec![PageCategory::Zeropage, PageCategory::Stack, PageCategory::Free]);
        // Umschalten eines Bits findet das Preset zum Wert
        assert_eq!(b.preset_for_value(0x37 ^ 1).unwrap().id, "36");
        assert!(preset_label(b.preset("37").unwrap()).starts_with("$37  Standard"));
        // Ziele ohne Presets
        assert!(vc_target::load_target(&data(), "vic20").unwrap().banking.is_none());
        let atari = vc_target::load_target(&data(), "atari8").unwrap();
        assert!(preset_label(atari.banking.as_ref().unwrap().preset("e3").unwrap()).ends_with("nur 130XE"));
    }
}
