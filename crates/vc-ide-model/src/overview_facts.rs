// SPDX-License-Identifier: GPL-3.0-or-later
//! Datenaufbereitung der Projekt-Übersicht (Kennzahlen, Quellen, Dateitabelle, Verbindungen) – ohne Oberfläche.
use crate::project::{Entry, Group};
use std::time::Instant;
use vc_ide::projfile::ProjectFile;

/// Deutsche Zahl mit einer Nachkommastelle: `14,8`
fn dec1(v: f64) -> String { format!("{v:.1}").replace('.', ",") }

/// Größe in Bytes als „812 B“ oder „14,8 KB“
pub fn fmt_size(bytes: usize) -> String {
    if bytes < 1024 { format!("{bytes} B") } else { format!("{} KB", dec1(bytes as f64 / 1024.0)) }
}

/// „1 Sprite“ / „2 Sprites“
pub fn count_label(n: usize, one: &str, many: &str) -> String { format!("{n} {}", if n == 1 { one } else { many }) }

/// „gerade eben“, „vor 42 s“, „vor 3 Min.“, „vor 2 Std.“
pub fn ago(secs: u64) -> String {
    match secs { 0..=1 => "gerade eben".into(), 2..=59 => format!("vor {secs} s"), 60..=3599 => format!("vor {} Min.", secs / 60), _ => format!("vor {} Std.", secs / 3600) }
}

/// Belegter Speicher aus den Segmenten der letzten Übersetzung (Start, Länge)
#[derive(Debug, PartialEq)]
pub struct MemFacts { pub used: usize, pub first: u32, pub last: u32, pub segments: usize }

pub fn memory_facts(segments: &[(u32, usize)]) -> Option<MemFacts> {
    let seg: Vec<&(u32, usize)> = segments.iter().filter(|s| s.1 > 0).collect();
    if seg.is_empty() { return None; }
    Some(MemFacts { used: seg.iter().map(|s| s.1).sum(), first: seg.iter().map(|s| s.0).min()?, last: seg.iter().map(|s| s.0 + s.1 as u32 - 1).max()?, segments: seg.len() })
}

/// Anzahl der Grafik- und Klangdateien je Art
#[derive(Debug, Default, PartialEq)]
pub struct MediaCounts { pub sprites: usize, pub charsets: usize, pub screens: usize, pub tiles: usize, pub songs: usize, pub palettes: usize }

impl MediaCounts {
    pub fn of(groups: impl Iterator<Item = Group>) -> MediaCounts {
        let mut c = MediaCounts::default();
        for g in groups { match g { Group::Sprite => c.sprites += 1, Group::Charset => c.charsets += 1, Group::Screen => c.screens += 1, Group::Tiles => c.tiles += 1, Group::Song | Group::Play => c.songs += 1, Group::Palette => c.palettes += 1, _ => {} } }
        c
    }
    pub fn total(&self) -> usize { self.sprites + self.charsets + self.screens + self.tiles + self.songs + self.palettes }
    /// Erste Zeile (Grafik) und zweite Zeile (Rest) der Kachel
    pub fn lines(&self) -> (String, String) {
        let mut a = vec![]; let mut b = vec![];
        if self.sprites > 0 { a.push(count_label(self.sprites, "Sprite", "Sprites")); }
        if self.charsets > 0 { a.push(count_label(self.charsets, "Zeichensatz", "Zeichensätze")); }
        if self.screens > 0 { b.push(count_label(self.screens, "Bildschirm", "Bildschirme")); }
        if self.tiles > 0 { b.push(count_label(self.tiles, "Kachelkarte", "Kachelkarten")); }
        if self.songs > 0 { b.push(count_label(self.songs, "Lied", "Lieder")); }
        if self.palettes > 0 { b.push(count_label(self.palettes, "Palette", "Paletten")); }
        (a.join(" · "), b.join(" · "))
    }
}

/// Zusammenfassung des letzten Baus
#[derive(Debug, PartialEq)]
pub struct BuildFacts { pub ok: bool, pub programs: usize, pub errors: usize, pub warnings: usize }

/// `log`: (Datei, gelungen) je Programm; `problems`: (Datei, ist Fehler). `None`, solange es kein Bauprotokoll gibt.
pub fn build_facts(log: &[(String, bool)], problems: &[(String, bool)]) -> Option<BuildFacts> {
    if log.is_empty() { return None; }
    let errors = problems.iter().filter(|p| p.1).count();
    Some(BuildFacts { ok: log.iter().all(|l| l.1) && errors == 0, programs: log.len(), errors, warnings: problems.len() - errors })
}

/// Zustand einer Quelldatei in der Quellenliste
#[derive(Clone, Debug, PartialEq)]
pub enum SrcStatus { Missing, NotBuilt, Built, Warnings(usize), Errors(usize) }

pub struct SrcRow { pub file: String, pub kind: String, pub dialect: String, pub converted: bool, pub target: String, pub main: bool, pub status: SrcStatus }

/// Quellen in Baureihenfolge aus der `.vcprj`. `exists`: gibt es die Datei im Projekt?
pub fn source_rows(pf: &ProjectFile, exists: impl Fn(&str) -> bool, log: &[(String, bool)], problems: &[(String, bool)]) -> Vec<SrcRow> {
    let main = pf.main_file();
    let mut rows = vec![];
    for s in &pf.sources { for f in &s.files {
        let errs = problems.iter().filter(|p| &p.0 == f && p.1).count();
        let warns = problems.iter().filter(|p| &p.0 == f && !p.1).count();
        let built = log.iter().find(|l| &l.0 == f).map(|l| l.1);
        let status = if !exists(f) { SrcStatus::Missing } else if errs > 0 || built == Some(false) { SrcStatus::Errors(errs) }
            else if warns > 0 { SrcStatus::Warnings(warns) } else if built == Some(true) { SrcStatus::Built } else { SrcStatus::NotBuilt };
        rows.push(SrcRow { file: f.clone(), kind: s.kind.clone(), dialect: s.dialect.clone(), converted: s.convert.is_some(), target: s.targets.first().cloned().unwrap_or_default(), main: main.as_deref() == Some(f.as_str()), status });
    } }
    rows
}

/// Zeile der Dateitabelle: jede am Projekt beteiligte Datei
#[derive(Debug, PartialEq)]
pub struct FileRow { pub rel: String, pub group: Group, pub tag: String, pub size: Option<u64>, pub main: bool, pub code: bool, pub status: Option<SrcStatus> }

/// Marke der Art wie in der Seitenleiste; sonstige Dateien zeigen ihre Endung
pub fn file_tag(group: Group, rel: &str) -> String {
    let t = group.tag();
    if !t.is_empty() { return t.to_string(); }
    match rel.rsplit_once('.') { Some((_, e)) if !e.is_empty() && e.len() <= 4 => e.to_uppercase(), _ => "DAT".into() }
}

/// Alle Dateien des Projekts: zuerst der Code (Startprogramm ganz oben, danach alphabetisch), dann der Rest nach Art gruppiert,
/// je Art alphabetisch. Quellen, die die `.vcprj` nennt und die fehlen, stehen mit Status „Datei fehlt“ dabei.
pub fn file_rows(root: &std::path::Path, entries: &[Entry], pf: Option<&ProjectFile>, log: &[(String, bool)], problems: &[(String, bool)]) -> Vec<FileRow> {
    let src: Vec<SrcRow> = pf.map(|pf| source_rows(pf, |f| root.join(f).is_file(), log, problems)).unwrap_or_default();
    let main = pf.and_then(|pf| pf.main_file());
    let mut rows: Vec<FileRow> = vec![];
    let mut add = |rel: &str, group: Group| {
        if rows.iter().any(|r| r.rel == rel) { return; }
        let status = src.iter().find(|s| s.file == rel).map(|s| s.status.clone());
        let in_src = status.is_some();
        rows.push(FileRow { rel: rel.to_string(), group, tag: file_tag(group, rel), size: std::fs::metadata(root.join(rel)).ok().filter(|m| m.is_file()).map(|m| m.len()), main: main.as_deref() == Some(rel), code: matches!(group, Group::Asm | Group::Basic) || in_src, status });
    };
    for e in entries { add(&e.rel, e.group); }
    for s in &src { add(&s.file, if s.kind == "basic" { Group::Basic } else { Group::Asm }); }
    rows.sort_by_key(|r| (!r.code, !r.main, if r.code { 0 } else { r.group as u8 }, r.rel.to_lowercase()));
    rows
}

/// Bezeichnung der Art: `vcasm` bzw. BASIC mit ausdrücklich gewähltem Dialekt
pub fn kind_chip(r: &SrcRow) -> String {
    match r.kind.as_str() { "asm" => "vcasm".into(), "basic" if r.dialect.is_empty() => "BASIC".into(), "basic" => format!("BASIC · {}", r.dialect), k => k.to_string() }
}

/// Zustand der Diskette des Projekts
#[derive(Debug, PartialEq)]
pub enum DiskFacts {
    /// Die Projektdatei beschreibt keine Diskette
    None,
    /// Beschrieben, aber noch nicht geschrieben
    NotBuilt { file: String, format: String },
    Image { file: String, format: String, free: String, files: usize },
    Broken { file: String, error: String },
}

pub fn disk_facts(root: &std::path::Path, pf: &ProjectFile) -> DiskFacts {
    let Some(spec) = &pf.disk else { return DiskFacts::None };
    let rel = vc_ide::projfile::disk_file(root, spec);
    let path = root.join(&rel);
    let file = rel.rsplit('/').next().unwrap_or(&rel).to_string();
    if !path.is_file() { return DiskFacts::NotBuilt { file, format: spec.format.to_uppercase() }; }
    match vc_ide::diskimage::Image::open(&path) {
        Ok(img) => DiskFacts::Image { file, format: spec.format.to_uppercase(), free: img.free, files: img.entries.len() },
        Err(error) => DiskFacts::Broken { file, error },
    }
}

/// Ampelzustand einer Verbindung
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dot { Ready, Idle, Bad }

/// Ergebnis des Verbindungstests einer Maschine
#[derive(Clone, Debug, PartialEq)]
pub enum Probe { Running, Ok(String), Err(String) }

/// Zeile des Verbindungsdiagramms
#[derive(Debug, PartialEq)]
pub struct Backend { pub name: &'static str, pub protocol: &'static str, pub dot: Dot }

/// Welches Protokoll spricht diese Emulator-Sorte? `None`: wird nur gestartet, ohne Monitor
pub fn protocol_of(kind: &str) -> Option<&'static str> {
    match kind { "x64sc" | "x64" | "x128" | "xplus4" | "xvic" => Some("vice"), "atari800" => Some("atari800"), _ => None }
}

/// Die vier Wege von vc-ide zum Ziel. `found`: Sorten der Standard-Emulatoren der Projekt-Systeme, die gefunden wurden;
/// `hw`: Maschinen der Projekt-Systeme (Sorte, Testergebnis).
pub fn backends(found: &[String], hw: &[(String, Option<Probe>)]) -> Vec<Backend> {
    let emu = |proto: &str| if found.iter().any(|k| protocol_of(k) == Some(proto)) { Dot::Ready } else { Dot::Idle };
    let machine = |kind: &str| {
        let mine: Vec<&Option<Probe>> = hw.iter().filter(|h| h.0 == kind).map(|h| &h.1).collect();
        if mine.iter().any(|p| matches!(p, Some(Probe::Ok(_)))) { Dot::Ready } else if mine.iter().any(|p| matches!(p, Some(Probe::Err(_)))) { Dot::Bad } else { Dot::Idle }
    };
    vec![
        Backend { name: "VICE", protocol: "Binärmonitor", dot: emu("vice") },
        Backend { name: "atari800", protocol: "Textmonitor", dot: emu("atari800") },
        Backend { name: "Ultimate", protocol: "REST", dot: machine("ultimate64") },
        Backend { name: "MEGA65", protocol: "UART", dot: machine("mega65") },
    ]
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind { Info, Ok, Warn, Err }

pub struct Event { pub at: Instant, pub kind: Kind, pub text: String }

#[cfg(test)]
mod tests {
    use super::*;

    fn pf(json: &str) -> ProjectFile { serde_json::from_str(json).unwrap() }

    #[test]
    fn groessen_und_zahlwoerter() {
        assert_eq!(fmt_size(812), "812 B");
        assert_eq!(fmt_size(15155), "14,8 KB");
        assert_eq!(count_label(1, "Sprite", "Sprites"), "1 Sprite");
        assert_eq!(count_label(2, "Sprite", "Sprites"), "2 Sprites");
        assert_eq!(ago(0), "gerade eben"); assert_eq!(ago(42), "vor 42 s"); assert_eq!(ago(180), "vor 3 Min."); assert_eq!(ago(7300), "vor 2 Std.");
    }

    #[test]
    fn speicher_aus_segmenten() {
        assert_eq!(memory_facts(&[]), None);
        assert_eq!(memory_facts(&[(0x0801, 0)]), None);
        assert_eq!(memory_facts(&[(0x0801, 0x100), (0xC000, 0x20)]), Some(MemFacts { used: 0x120, first: 0x0801, last: 0xC01F, segments: 2 }));
    }

    #[test]
    fn medien_zaehlen() {
        let c = MediaCounts::of([Group::Sprite, Group::Sprite, Group::Charset, Group::Song, Group::Asm, Group::Other].into_iter());
        assert_eq!((c.sprites, c.charsets, c.songs, c.total()), (2, 1, 1, 4));
        assert_eq!(c.lines(), ("2 Sprites · 1 Zeichensatz".to_string(), "1 Lied".to_string()));
        assert_eq!(MediaCounts::default().total(), 0);
    }

    #[test]
    fn build_ohne_protokoll_ist_none() {
        assert_eq!(build_facts(&[], &[]), None);
        let b = build_facts(&[("a.asm".into(), true)], &[("a.asm".into(), false)]).unwrap();
        assert_eq!(b, BuildFacts { ok: true, programs: 1, errors: 0, warnings: 1 });
        let b = build_facts(&[("a.asm".into(), false)], &[("a.asm".into(), true)]).unwrap();
        assert!(!b.ok && b.errors == 1);
    }

    #[test]
    fn quellenliste_mit_status() {
        let p = pf(r#"{"targets":[{"id":"c64","system":"c64"}],"sources":[
            {"targets":["c64"],"kind":"basic","dialect":"turbo","files":["menu.bas"]},
            {"targets":["c64"],"kind":"asm","files":["player.asm","sfx.asm","weg.asm"]}],"main":"menu.bas"}"#);
        let log = vec![("menu.bas".to_string(), true), ("player.asm".to_string(), true), ("sfx.asm".to_string(), true)];
        let problems = vec![("sfx.asm".to_string(), false)];
        let rows = source_rows(&p, |f| f != "weg.asm", &log, &problems);
        assert_eq!(rows.iter().map(|r| &r.status).collect::<Vec<_>>(), vec![&SrcStatus::Built, &SrcStatus::Built, &SrcStatus::Warnings(1), &SrcStatus::Missing]);
        assert!(rows[0].main && !rows[1].main);
        assert_eq!(kind_chip(&rows[0]), "BASIC · turbo"); assert_eq!(kind_chip(&rows[1]), "vcasm");
        // ohne Bau: nichts als gebaut ausgeben
        assert!(source_rows(&p, |_| true, &[], &[]).iter().all(|r| r.status == SrcStatus::NotBuilt));
        // Fehler schlagen Warnungen und Erfolg
        let rows = source_rows(&p, |_| true, &log, &[("player.asm".into(), true)]);
        assert_eq!(rows[1].status, SrcStatus::Errors(1));
    }

    #[test]
    fn startprogramm_ohne_main_ist_die_erste_quelle() {
        let p = pf(r#"{"targets":[{"id":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm","b.asm"]}]}"#);
        let rows = source_rows(&p, |_| true, &[], &[]);
        assert!(rows[0].main && !rows[1].main);
    }

    #[test]
    fn diskette_nicht_beschrieben_nicht_gebaut_und_fertig() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(tmp.join("build")).unwrap();
        assert_eq!(disk_facts(&tmp, &pf(r#"{"targets":[{"id":"c64"}]}"#)), DiskFacts::None);
        let p = pf(r#"{"targets":[{"id":"c64"}],"disk":{"format":"d64","file":"build/x.d64"}}"#);
        assert_eq!(disk_facts(&tmp, &p), DiskFacts::NotBuilt { file: "x.d64".into(), format: "D64".into() });
        let mut d = vc_disk::D64::blank("TEST", "01").unwrap();
        d.add_prg("A", &[0x01, 0x08, 0x60]).unwrap();
        std::fs::write(tmp.join("build/x.d64"), d.to_bytes()).unwrap();
        match disk_facts(&tmp, &p) { DiskFacts::Image { file, format, free, files } => { assert_eq!((file.as_str(), format.as_str(), files), ("x.d64", "D64", 1)); assert!(free.ends_with("Blöcke frei"), "{free}"); } o => panic!("{o:?}") }
        std::fs::write(tmp.join("build/x.d64"), b"kaputt").unwrap();
        assert!(matches!(disk_facts(&tmp, &p), DiskFacts::Broken { .. }));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn verbindungen_gruen_grau_rot() {
        let b = backends(&[], &[]);
        assert!(b.iter().all(|x| x.dot == Dot::Idle));
        let b = backends(&["x64sc".into(), "altirra".into()], &[("ultimate64".into(), None), ("mega65".into(), Some(Probe::Err("x".into())))]);
        assert_eq!(b.iter().map(|x| x.dot).collect::<Vec<_>>(), vec![Dot::Ready, Dot::Idle, Dot::Idle, Dot::Bad]);
        let b = backends(&["atari800".into()], &[("ultimate64".into(), Some(Probe::Ok("ok".into())))]);
        assert_eq!(b.iter().map(|x| x.dot).collect::<Vec<_>>(), vec![Dot::Idle, Dot::Ready, Dot::Ready, Dot::Idle]);
        assert_eq!(protocol_of("xemu"), None);
    }

    #[test]
    fn dateitabelle_code_zuerst_main_oben_dann_nach_art() {
        let tmp = std::env::temp_dir().join(format!("vc_overview_files_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        for f in ["b.asm", "a.asm", "main.asm", "z.chr.json", "a.spr.json", "song.trk.json", "x.d64"] { std::fs::write(tmp.join(f), "12345").unwrap(); }
        let e = |rel: &str, group| Entry { rel: rel.into(), group };
        let entries = vec![e("a.spr.json", Group::Sprite), e("b.asm", Group::Asm), e("a.asm", Group::Asm), e("main.asm", Group::Asm), e("song.trk.json", Group::Song), e("x.d64", Group::Other), e("z.chr.json", Group::Charset)];
        let p = pf(r#"{"targets":[{"id":"c64"}],"sources":[{"targets":["c64"],"kind":"asm","files":["a.asm","b.asm","main.asm","fehlt.asm"]}],"main":"main.asm"}"#);
        let rows = file_rows(&tmp, &entries, Some(&p), &[], &[]);
        assert_eq!(rows.iter().map(|r| r.rel.as_str()).collect::<Vec<_>>(), vec!["main.asm", "a.asm", "b.asm", "fehlt.asm", "song.trk.json", "a.spr.json", "z.chr.json", "x.d64"]);
        assert_eq!(rows.iter().map(|r| r.tag.as_str()).collect::<Vec<_>>(), vec!["ASM", "ASM", "ASM", "ASM", "TRK", "SPR", "CHR", "D64"]);
        assert!(rows[0].main && rows[0].code && rows[0].size == Some(5));
        assert_eq!(rows[3].status, Some(SrcStatus::Missing)); assert_eq!(rows[3].size, None);
        assert!(rows[5].status.is_none() && !rows[5].code);
        // keine Projektdatei: trotzdem alle Dateien, ohne Startprogramm
        assert!(file_rows(&tmp, &entries, None, &[], &[]).iter().all(|r| !r.main));
        assert_eq!(file_tag(Group::Other, "notes.txt"), "TXT"); assert_eq!(file_tag(Group::Other, "LIESMICH"), "DAT");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
