// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateien ins Projekt übernehmen (Drag and Drop aus Finder/Explorer): Art erkennen, Kategorie und Zielordner vorschlagen,
//! Namenskonflikte lösen, ins Projektverzeichnis kopieren und in `assets` der `.vcprj` eintragen.
//!
//! Konzept: Eine abgelegte Datei hat eine **Art** (was sie ist: Quelltext, Text, Binärdaten, PRG, Zeichensatz-Rohdaten, Bild …),
//! erkannt an der Endung und, wo die nicht reicht, am Inhalt. Die **Kategorie** (Quelltext, Text, Binärdaten, Grafik, Zeichensatz)
//! legt der Nutzer fest (Vorschlag aus der Art); sie steht mit Beschreibung in `assets` und bestimmt die Marke im Projektbaum.
//! Dateien werden immer ins Projekt **kopiert** (das Projekt bleibt in sich geschlossen und lässt sich verschieben); ein Pfad auf
//! eine Datei außerhalb würde `vcc` und den Build brüchig machen. Liegt die Datei schon im Projektordner, wird sie nur eingetragen.
use crate::projfile::{Asset, ProjectFile, Rule};
use std::path::{Component, Path, PathBuf};

/// Was die Datei ist
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Asm, Basic, Text, Binary, Prg, Charset, Image, RawBitmap, DiskImage, Studio }

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Asm => "Assembler-Quelltext", Kind::Basic => "BASIC-Quelltext", Kind::Text => "Text", Kind::Binary => "Binärdaten",
            Kind::Prg => "PRG (Ladeadresse + Daten)", Kind::Charset => "Zeichensatz-Rohdaten", Kind::Image => "Bild",
            Kind::RawBitmap => "Grafik-Rohdaten (Bitmap)", Kind::DiskImage => "Diskettenimage", Kind::Studio => "Studio-Datei (Sprite, Song …)",
        }
    }
    /// Vorgeschlagene Kategorie
    pub fn category(self) -> Category {
        match self {
            Kind::Asm | Kind::Basic => Category::Source, Kind::Text => Category::Text, Kind::Charset => Category::Charset,
            Kind::Image | Kind::RawBitmap => Category::Graphics, Kind::Studio => Category::Studio,
            Kind::Binary | Kind::Prg | Kind::DiskImage => Category::Binary,
        }
    }
}

/// Kategorie im Projekt (Eintrag `category` in `assets`)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category { Source, Text, Binary, Graphics, Charset, /** Studio-Dateien kennt der Projektbaum schon; kein Eintrag in `assets` */ Studio }

impl Category {
    pub const CHOICES: [Category; 5] = [Category::Source, Category::Text, Category::Binary, Category::Graphics, Category::Charset];
    pub fn id(self) -> &'static str {
        match self { Category::Source => "source", Category::Text => "text", Category::Binary => "binary", Category::Graphics => "graphics", Category::Charset => "charset", Category::Studio => "studio" }
    }
    pub fn from_id(s: &str) -> Option<Category> { [Category::Source, Category::Text, Category::Binary, Category::Graphics, Category::Charset, Category::Studio].into_iter().find(|c| c.id() == s) }
    pub fn label(self) -> &'static str {
        match self { Category::Source => "Quelltext", Category::Text => "Text / Doku", Category::Binary => "Binärdaten", Category::Graphics => "Grafikrohdaten", Category::Charset => "Zeichensatz (roh)", Category::Studio => "Studio-Datei" }
    }
    /// Vorgeschlagener Zielordner im Projekt (leer = Projektordner)
    pub fn dir(self) -> &'static str {
        match self { Category::Source => "lib", Category::Text => "doc", Category::Binary | Category::Charset => "data", Category::Graphics => "graphics", Category::Studio => "" }
    }
}

fn ext(name: &str) -> String { Path::new(name).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default() }

/// Art aus Dateiname und Anfang der Daten (`head` genügt; `len` ist die Gesamtgröße)
pub fn detect(name: &str, head: &[u8], len: u64) -> Kind {
    let l = name.to_ascii_lowercase();
    if vc_media::naming::is_data_file(&l) { return Kind::Studio; }
    if crate::diskimage::is_image_name(&l) { return Kind::DiskImage; }
    match ext(&l).as_str() {
        "asm" | "inc" | "s" | "a65" | "i65" | "src" => return Kind::Asm,
        "bas" | "lst" | "sbas" => return Kind::Basic,
        "txt" | "md" | "json" | "csv" | "xml" | "cfg" | "ini" | "html" | "htm" | "yml" | "yaml" | "toml" | "log" | "tsv" => return Kind::Text,
        "prg" | "p00" => return Kind::Prg,
        "chr" | "fnt" | "font" | "cset" | "set" => return Kind::Charset,
        "png" | "jpg" | "jpeg" | "gif" | "bmp" => return Kind::Image,
        "koa" | "kla" | "gg" | "art" | "pic" | "dd" | "ddl" | "fli" | "afli" | "mpic" | "spr" => return Kind::RawBitmap,
        _ => {}
    }
    // 1, 2 oder 4 KiB ohne eindeutige Endung (bin/rom/dat/keine): sehr wahrscheinlich ein Zeichensatz
    if matches!(ext(&l).as_str(), "bin" | "rom" | "dat" | "") && matches!(len, 1024 | 2048 | 4096) && !is_text(head) { return Kind::Charset; }
    if is_text(head) { Kind::Text } else { Kind::Binary }
}

/// Sieht der Anfang nach Text aus (gültiges UTF-8, keine NUL-Bytes)?
pub fn is_text(head: &[u8]) -> bool {
    if head.contains(&0) { return false; }
    match std::str::from_utf8(head) {
        Ok(_) => true,
        // ein am Ende der Probe abgeschnittenes Zeichen ist kein Fehler
        Err(e) => e.error_len().is_none() && e.valid_up_to() + 4 > head.len(),
    }
}

/// Für Projektbaum und Editor: ist das eine Binärdatei, die nicht als Text geöffnet werden kann?
pub fn looks_binary(bytes: &[u8]) -> bool { std::str::from_utf8(bytes).is_err() || bytes.contains(&0) }

/// Marke im Projektbaum, die schon die Endung nahelegt (ohne Eintrag in `assets`): `"binary"` oder `"text"`
pub fn tree_hint(name: &str) -> Option<&'static str> {
    match ext(name).as_str() {
        "bin" | "dat" | "chr" | "fnt" | "rom" | "prg" | "koa" | "kla" | "p00" | "cset" => Some("binary"),
        "txt" | "md" | "cfg" | "ini" | "csv" => Some("text"),
        _ => None,
    }
}

/// Zielordner je Kategorie aus dem Abschnitt `layout` der Projektdatei; fehlende Kategorien fallen auf [`Category::dir`] zurück
pub fn dir_for(layout: &std::collections::BTreeMap<String, String>, c: Category) -> String {
    match layout.get(c.id()).map(|d| d.trim_matches('/')) { Some(d) if valid_dir(d) => d.to_string(), _ => c.dir().to_string() }
}

/// Glob mit `*` (beliebig viele Zeichen) und `?` (ein Zeichen), ohne Groß-/Kleinschreibung
pub fn glob_match(pat: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pat.to_lowercase().chars().collect(), name.to_lowercase().chars().collect());
    fn go(p: &[char], n: &[char]) -> bool {
        match p.first() {
            None => n.is_empty(),
            Some('*') => (0..=n.len()).any(|i| go(&p[1..], &n[i..])),
            Some('?') => !n.is_empty() && go(&p[1..], &n[1..]),
            Some(c) => n.first() == Some(c) && go(&p[1..], &n[1..]),
        }
    }
    go(&p, &n)
}

/// Erste passende Regel: Kategorie (leer = Vorschlag aus der Art) und Ordner (leer = Ordner der Kategorie aus `layout`)
pub fn apply_rules(rules: &[Rule], name: &str) -> Option<(Option<Category>, Option<String>)> {
    let r = rules.iter().find(|r| !r.pattern.trim().is_empty() && glob_match(r.pattern.trim(), name))?;
    let dir = (!r.dir.is_empty() && valid_dir(&r.dir)).then(|| r.dir.trim_matches('/').to_string());
    Some((Category::from_id(&r.category), dir))
}

/// Dateien eines Ordners (rekursiv), mit Ordner relativ zum übergeordneten Ordner (`bilder/sub`). Versteckte Dateien und Ordner
/// (`.git`, `.DS_Store`), `target` und `node_modules` bleiben draußen; Symlinks auf Ordner werden nicht verfolgt.
/// Liefert höchstens `max` Dateien und die Zahl der übrigen.
pub fn walk_folder(folder: &Path, max: usize) -> (Vec<(PathBuf, String)>, usize) {
    fn go(base: &str, dir: &Path, depth: usize, out: &mut Vec<(PathBuf, String)>, over: &mut usize, max: usize) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut es: Vec<_> = rd.flatten().collect();
        es.sort_by_key(|e| e.file_name());
        for e in es {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() { if depth < 16 { go(&join_rel(base, &name), &e.path(), depth + 1, out, over, max); } }
            else if t.is_file() { if out.len() < max { out.push((e.path(), base.to_string())); } else { *over += 1; } }
        }
    }
    let name = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let (mut out, mut over) = (vec![], 0);
    go(&name, folder, 0, &mut out, &mut over, max);
    (out, over)
}

/// Was bei einem Namenskonflikt geschieht
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Conflict { #[default] Rename, Replace, Skip }

/// Eine abgelegte Datei und wohin sie soll
#[derive(Clone, Debug)]
pub struct Plan {
    pub src: PathBuf,
    /// Dateiname im Projekt
    pub name: String,
    /// Zielordner relativ zum Projekt (leer = Projektordner)
    pub dir: String,
    pub category: Category,
    pub description: String,
    pub conflict: Conflict,
}

impl Plan {
    /// Vorschlag für eine Datei: Art erkennen, Kategorie und Zielordner daraus ableiten
    pub fn suggest(src: &Path) -> Plan { Self::suggest_in(src, &Default::default()) }
    /// Wie [`Plan::suggest`], mit den Zielordnern der Projektstruktur (`layout`)
    pub fn suggest_in(src: &Path, layout: &std::collections::BTreeMap<String, String>) -> Plan { Self::suggest_with(src, layout, &[]) }
    /// Wie [`Plan::suggest_in`], zusätzlich mit Regeln nach Dateimuster (`rules`), die vor `layout` gelten
    pub fn suggest_with(src: &Path, layout: &std::collections::BTreeMap<String, String>, rules: &[Rule]) -> Plan {
        let name = src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let (head, len) = read_head(src);
        let kind = detect(&name, &head, len);
        let (rc, rd) = apply_rules(rules, &name).unwrap_or((None, None));
        let category = rc.unwrap_or_else(|| kind.category());
        let dir = rd.unwrap_or_else(|| dir_for(layout, category));
        Plan { src: src.to_path_buf(), name, dir, category, description: String::new(), conflict: Conflict::Rename }
    }
    pub fn kind(&self) -> Kind { let (h, l) = read_head(&self.src); detect(&self.name, &h, l) }
}

fn read_head(p: &Path) -> (Vec<u8>, u64) {
    use std::io::Read;
    let len = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let mut buf = vec![0u8; 4096.min(len as usize)];
    if let Ok(mut f) = std::fs::File::open(p) { let n = f.read(&mut buf).unwrap_or(0); buf.truncate(n); } else { buf.clear(); }
    (buf, len)
}

/// Ist `dir` ein zulässiger Ordner im Projekt (relativ, ohne `..`, ohne Laufwerk)?
pub fn valid_dir(dir: &str) -> bool {
    Path::new(dir).components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)) && !dir.starts_with('/') && !dir.contains('\\')
}
/// Ist `name` ein zulässiger Dateiname (nicht leer, kein Pfad, nicht versteckt)?
pub fn valid_name(name: &str) -> bool { !name.is_empty() && !name.contains(['/', '\\', ':']) && !name.starts_with('.') && name != ".." }

/// Pfad im Projekt (immer mit `/`)
pub fn join_rel(dir: &str, name: &str) -> String {
    let d = dir.trim_matches('/').replace('\\', "/");
    if d.is_empty() { name.to_string() } else { format!("{d}/{name}") }
}

/// Erster freier Name: `font.bin`, `font_2.bin`, `font_3.bin` …
pub fn unique_name(root: &Path, dir: &str, name: &str) -> String {
    if !root.join(join_rel(dir, name)).exists() { return name.to_string(); }
    let (stem, e) = match name.rsplit_once('.') { Some((a, b)) if !a.is_empty() => (a.to_string(), format!(".{b}")), _ => (name.to_string(), String::new()) };
    (2..10_000).map(|n| format!("{stem}_{n}{e}")).find(|c| !root.join(join_rel(dir, c)).exists()).unwrap_or_else(|| name.to_string())
}

/// Zustand des Ziels einer Datei
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target { /** Name ist frei */ Free, /** Datei mit anderem Inhalt vorhanden */ Clash, /** Datei mit gleichem Inhalt vorhanden */ Same, /** Quelle liegt schon dort */ InPlace }

/// Prüft das Ziel von `plan` (für die Anzeige im Dialog)
pub fn target_state(root: &Path, plan: &Plan) -> Target {
    let dest = root.join(join_rel(&plan.dir, &plan.name));
    if !dest.exists() { return Target::Free; }
    if same_file(&plan.src, &dest) { return Target::InPlace; }
    if std::fs::read(&plan.src).ok() == std::fs::read(&dest).ok() { Target::Same } else { Target::Clash }
}

fn same_file(a: &Path, b: &Path) -> bool { matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y) }

/// Ergebnis einer Übernahme
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome { Copied(String), Replaced(String), /** gleicher Inhalt war schon da */ Existing(String), Skipped(String) }

impl Outcome {
    /// Pfad der Datei im Projekt (nicht bei `Skipped`)
    pub fn rel(&self) -> Option<&str> { match self { Outcome::Copied(r) | Outcome::Replaced(r) | Outcome::Existing(r) => Some(r), Outcome::Skipped(_) => None } }
}

/// Übernimmt eine Datei: kopiert sie nach `<root>/<dir>/<name>` (Konflikt nach `plan.conflict`) und trägt sie in `pf` ein
/// (außer Studio-Dateien). Liegt die Quelle schon im Projektordner, wird nichts kopiert.
pub fn import(root: &Path, pf: Option<&mut ProjectFile>, plan: &Plan) -> Result<Outcome, String> {
    if !valid_name(&plan.name) { return Err(format!("{}: ungültiger Dateiname", plan.name)); }
    if !valid_dir(&plan.dir) { return Err(format!("{}: ungültiger Zielordner", plan.dir)); }
    if plan.src.is_dir() { return Err(format!("{}: Ordner werden nicht übernommen", plan.src.display())); }
    // Quelle liegt schon im Projekt: bleibt, wo sie ist
    let inside = plan.src.canonicalize().ok().zip(root.canonicalize().ok()).and_then(|(s, r)| s.strip_prefix(&r).ok().map(|p| p.to_string_lossy().replace('\\', "/")));
    let (rel, out) = if let Some(rel) = inside { (rel.clone(), Outcome::Existing(rel)) } else {
        let state = target_state(root, plan);
        let (name, replaced) = match (state, plan.conflict) {
            (Target::Free, _) => (plan.name.clone(), false),
            (Target::Same | Target::InPlace, _) => (plan.name.clone(), false),
            (Target::Clash, Conflict::Skip) => return Ok(Outcome::Skipped(join_rel(&plan.dir, &plan.name))),
            (Target::Clash, Conflict::Rename) => (unique_name(root, &plan.dir, &plan.name), false),
            (Target::Clash, Conflict::Replace) => (plan.name.clone(), true),
        };
        let rel = join_rel(&plan.dir, &name);
        let dest = root.join(&rel);
        if state == Target::Same || state == Target::InPlace { (rel.clone(), Outcome::Existing(rel)) } else {
            if let Some(p) = dest.parent() { std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?; }
            std::fs::copy(&plan.src, &dest).map_err(|e| format!("{}: {e}", plan.src.display()))?;
            (rel.clone(), if replaced { Outcome::Replaced(rel) } else { Outcome::Copied(rel) })
        }
    };
    if plan.category != Category::Studio {
        if let Some(pf) = pf { pf.set_asset(Asset { file: rel, category: plan.category.id().to_string(), description: plan.description.clone() }); }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc_import_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
    }

    #[test]
    fn art_aus_endung_und_inhalt() {
        assert_eq!(detect("lib.asm", b"", 0), Kind::Asm);
        assert_eq!(detect("a.BAS", b"", 0), Kind::Basic);
        assert_eq!(detect("notiz.txt", b"hallo", 5), Kind::Text);
        assert_eq!(detect("spiel.prg", &[1, 8, 0], 3), Kind::Prg);
        assert_eq!(detect("bild.png", b"\x89PNG", 4), Kind::Image);
        assert_eq!(detect("bild.koa", &[0, 0x60], 10003), Kind::RawBitmap);
        assert_eq!(detect("x.d64", b"", 174848), Kind::DiskImage);
        assert_eq!(detect("held.vcsprite", b"{}", 2), Kind::Studio);
        // Größe und Endung: 2 KiB ohne Textinhalt = Zeichensatz, sonst Binärdaten
        assert_eq!(detect("font.bin", &[0, 1, 2, 3], 2048), Kind::Charset);
        assert_eq!(detect("tab.bin", &[0, 1, 2, 3], 777), Kind::Binary);
        assert_eq!(detect("unbekannt", b"nur text", 8), Kind::Text);
        assert_eq!(detect("unbekannt.xyz", &[0xff, 0xfe, 0, 1], 4), Kind::Binary);
        assert!(is_text("Grüße".as_bytes()) && !is_text(&[0xff, 0xfe]) && !is_text(&[b'a', 0, b'b']));
    }

    #[test]
    fn kategorie_und_zielordner() {
        assert_eq!((Kind::Asm.category(), Category::Source.dir()), (Category::Source, "lib"));
        assert_eq!(Kind::Charset.category().id(), "charset");
        assert_eq!(Kind::Prg.category().dir(), "data");
        assert_eq!(Kind::Image.category().dir(), "graphics");
        for c in Category::CHOICES { assert_eq!(Category::from_id(c.id()), Some(c)); }
        assert_eq!(tree_hint("x.BIN"), Some("binary"));
        assert_eq!(tree_hint("x.txt"), Some("text"));
        assert_eq!(tree_hint("x.asm"), None);
    }

    #[test]
    fn layout_bestimmt_den_vorschlag_und_ordner_werden_durchlaufen() {
        let mut l = std::collections::BTreeMap::new();
        l.insert("binary".to_string(), "res/bin/".to_string());
        l.insert("text".to_string(), "../raus".to_string());
        assert_eq!(dir_for(&l, Category::Binary), "res/bin");
        assert_eq!(dir_for(&l, Category::Text), "doc", "ungültiger Eintrag: Vorschlag der IDE");
        assert_eq!(dir_for(&l, Category::Graphics), "graphics");
        let d = tmp("walk");
        std::fs::create_dir_all(d.join("pack/sub/tief")).unwrap(); std::fs::create_dir_all(d.join("pack/.git")).unwrap();
        for f in ["pack/a.bin", "pack/sub/b.txt", "pack/sub/tief/c.asm", "pack/.git/x", "pack/.DS_Store"] { std::fs::write(d.join(f), [1]).unwrap(); }
        let (v, over) = walk_folder(&d.join("pack"), 100);
        let mut got: Vec<(String, String)> = v.iter().map(|(p, b)| (p.file_name().unwrap().to_string_lossy().into_owned(), b.clone())).collect();
        got.sort();
        assert_eq!(got, vec![("a.bin".into(), "pack".into()), ("b.txt".into(), "pack/sub".into()), ("c.asm".into(), "pack/sub/tief".into())]);
        assert_eq!(over, 0);
        let (v, over) = walk_folder(&d.join("pack"), 2);
        assert_eq!((v.len(), over), (2, 1));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn regeln_gehen_dem_layout_vor() {
        assert!(glob_match("*.SID", "intro.sid") && glob_match("level?.bin", "level1.bin") && !glob_match("level?.bin", "level10.bin") && glob_match("*", "x") && !glob_match("*.sid", "a.sid.txt"));
        let rules = vec![
            Rule { pattern: "*.sid".into(), category: "binary".into(), dir: "res/music".into() },
            Rule { pattern: "level*.bin".into(), category: String::new(), dir: "res/levels".into() },
            Rule { pattern: "*.koa".into(), category: "graphics".into(), dir: "../raus".into() },
        ];
        let d = tmp("rules");
        for n in ["intro.sid", "level3.bin", "pic.koa", "tab.bin"] { std::fs::write(d.join(n), [0u8, 1, 2]).unwrap(); }
        let mut lay = std::collections::BTreeMap::new(); lay.insert("binary".to_string(), "res/bin".to_string()); lay.insert("graphics".to_string(), "res/gfx".to_string());
        let p = |n: &str| Plan::suggest_with(&d.join(n), &lay, &rules);
        assert_eq!((p("intro.sid").category, p("intro.sid").dir.as_str()), (Category::Binary, "res/music"));
        assert_eq!(p("level3.bin").dir, "res/levels", "Regel ohne Kategorie: Art bleibt");
        assert_eq!((p("pic.koa").category, p("pic.koa").dir.as_str()), (Category::Graphics, "res/gfx"), "ungültiger Regelordner: layout");
        assert_eq!(p("tab.bin").dir, "res/bin");
        let mut pf = ProjectFile::default();
        pf.rules = rules; pf.layout.insert("nix".into(), "x".into());
        let v = pf.validate(&d);
        assert!(v.iter().any(|m| m.contains("nix")) && v.iter().any(|m| m.contains("../raus")));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn namen_und_ordner_pruefen() {
        assert!(valid_dir("") && valid_dir("data/fonts") && !valid_dir("../x") && !valid_dir("/abs") && !valid_dir("a/../b"));
        assert!(valid_name("a.bin") && !valid_name("") && !valid_name("a/b") && !valid_name(".versteckt"));
        assert_eq!(join_rel("", "a"), "a");
        assert_eq!(join_rel("data/", "a.bin"), "data/a.bin");
    }

    #[test]
    fn kopieren_eintragen_und_namenskonflikte() {
        let root = tmp("copy"); let ext = tmp("ext");
        let src = ext.join("font.bin"); std::fs::write(&src, { let mut v = vec![7u8; 2048]; v[0] = 0; v }).unwrap();
        let mut pf = ProjectFile::default();
        let mut plan = Plan::suggest(&src);
        assert_eq!((plan.category, plan.dir.as_str()), (Category::Charset, "data"));
        plan.description = "Zeichensatz".into();
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Copied("data/font.bin".into()));
        assert_eq!(std::fs::read(root.join("data/font.bin")).unwrap().len(), 2048);
        assert_eq!(pf.asset("data/font.bin").map(|a| (a.category.as_str(), a.description.as_str())), Some(("charset", "Zeichensatz")));
        // gleicher Inhalt: nichts kopiert
        assert_eq!(target_state(&root, &plan), Target::Same);
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Existing("data/font.bin".into()));
        // anderer Inhalt: Umbenennen / Ersetzen / Überspringen
        std::fs::write(&src, { let mut v = vec![9u8; 2048]; v[1] = 0; v }).unwrap();
        assert_eq!(target_state(&root, &plan), Target::Clash);
        plan.conflict = Conflict::Skip;
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Skipped("data/font.bin".into()));
        assert_eq!(std::fs::read(root.join("data/font.bin")).unwrap()[2], 7);
        plan.conflict = Conflict::Rename;
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Copied("data/font_2.bin".into()));
        assert_eq!(std::fs::read(root.join("data/font.bin")).unwrap()[2], 7);
        plan.conflict = Conflict::Replace;
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Replaced("data/font.bin".into()));
        assert_eq!(std::fs::read(root.join("data/font.bin")).unwrap()[2], 9);
        assert_eq!(pf.assets.len(), 2, "pro Datei ein Eintrag");
        // Ungültiges Ziel
        plan.dir = "../draussen".into();
        assert!(import(&root, Some(&mut pf), &plan).is_err());
        let _ = std::fs::remove_dir_all(&root); let _ = std::fs::remove_dir_all(&ext);
    }

    #[test]
    fn datei_im_projekt_wird_nur_eingetragen_studio_nie() {
        let root = tmp("inplace");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("sub/t.txt"), "hallo").unwrap();
        let mut pf = ProjectFile::default();
        let plan = Plan::suggest(&root.join("sub/t.txt"));
        assert_eq!(import(&root, Some(&mut pf), &plan).unwrap(), Outcome::Existing("sub/t.txt".into()));
        assert_eq!(pf.asset("sub/t.txt").map(|a| a.category.as_str()), Some("text"));
        assert!(!root.join("doc").exists(), "nichts kopiert");
        std::fs::write(root.join("held.vcsprite"), "{}").unwrap();
        let p2 = Plan::suggest(&root.join("held.vcsprite"));
        assert_eq!(p2.category, Category::Studio);
        import(&root, Some(&mut pf), &p2).unwrap();
        assert_eq!(pf.assets.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn vcprj_runde_mit_assets_und_ohne() {
        let alt = r#"{ "format": 1, "project": {"name": "x"}, "sources": [] }"#;
        let mut pf: ProjectFile = serde_json::from_str(alt).unwrap();
        assert!(pf.assets.is_empty());
        assert!(!serde_json::to_string(&pf).unwrap().contains("assets"), "leer bleibt unsichtbar");
        pf.set_asset(Asset { file: "data/a.bin".into(), category: "binary".into(), description: "Tabelle".into() });
        pf.set_asset(Asset { file: "data/a.bin".into(), category: "charset".into(), description: String::new() });
        assert_eq!(pf.assets.len(), 1);
        let t = serde_json::to_string_pretty(&pf).unwrap();
        let back: ProjectFile = serde_json::from_str(&t).unwrap();
        assert_eq!(back, pf);
        assert!(!t.contains("description"), "leere Beschreibung entfällt");
        // Umbenennen und Entfernen
        assert!(pf.rename_asset("data/a.bin", Some("data/b.bin")));
        assert!(pf.asset("data/b.bin").is_some() && pf.asset("data/a.bin").is_none());
        assert!(pf.rename_asset("data/b.bin", None) && pf.assets.is_empty());
        assert!(!pf.rename_asset("gibt/es/nicht", None));
        // Regeln: Feldname `match`
        let r: ProjectFile = serde_json::from_str(r#"{ "rules": [{"match": "*.sid", "category": "binary", "dir": "res/music"}] }"#).unwrap();
        assert_eq!(r.rules[0].pattern, "*.sid");
        assert!(serde_json::to_string(&r).unwrap().contains("\"match\":\"*.sid\""));
        // unbekannte Felder bleiben erhalten
        let mit: ProjectFile = serde_json::from_str(r#"{ "format": 1, "zukunft": 5, "assets": [{"file": "x"}] }"#).unwrap();
        assert_eq!(mit.assets[0].category, "");
        assert!(serde_json::to_string(&mit).unwrap().contains("zukunft"));
    }

    #[test]
    fn validate_meldet_fehlende_assets() {
        let root = tmp("val");
        let mut pf = ProjectFile::default();
        pf.set_asset(Asset { file: "fehlt.bin".into(), category: "binary".into(), description: String::new() });
        assert!(pf.validate(&root).iter().any(|m| m.contains("fehlt.bin")));
        std::fs::write(root.join("fehlt.bin"), [1]).unwrap();
        assert!(pf.validate(&root).iter().all(|m| !m.contains("fehlt.bin")));
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod folder_tests {
    use super::*;

    #[test]
    fn rollen_setzen_entfernen_und_pruefen() {
        let mut pf = ProjectFile::default();
        assert!(pf.set_role("src", Some("source")) && !pf.set_role("src", Some("source")));
        assert_eq!(pf.role_of("src"), Some("source"));
        assert!(pf.set_role("src", Some("tests")) && pf.role_of("src") == Some("tests") && pf.folders.len() == 1);
        assert!(pf.set_role("src", None) && pf.role_of("src").is_none() && !pf.set_role("src", None));
        // Ziel-Beschränkung bleibt beim Entfernen der Rolle, die Rolle beim Entfernen des Ziels
        pf.set_role("src/c64", Some("source")); assert!(pf.set_folder_target("src/c64", Some("c64")) && pf.targets_of("src/c64") == ["c64"]);
        assert!(pf.set_role("src/c64", None) && pf.targets_of("src/c64") == ["c64"] && pf.role_of("src/c64").is_none());
        assert!(pf.set_folder_target("src/c64", None) && pf.folders.is_empty());
        pf.set_role("a", Some("docs")); pf.set_folder_target("a", Some("c64")); pf.set_folder_target("a", None);
        assert_eq!(pf.role_of("a"), Some("docs"));
        pf.set_role("a", None);
        pf.targets.clear(); pf.set_folder_target("b", Some("nix"));
        assert!(pf.validate(std::path::Path::new(".")).iter().any(|m| m.contains("Target 'nix'")));
        pf.set_folder_target("b", None);
        pf.set_role("x", Some("quatsch")); pf.folders.push(crate::projfile::FolderDef { dir: "../y".into(), role: "docs".into(), targets: vec![] });
        let v = pf.validate(std::path::Path::new("."));
        assert!(v.iter().any(|m| m.contains("quatsch")) && v.iter().any(|m| m.contains("../y")));
        assert!(pf.set_layout_dir("binary", "res/bin") && !pf.set_layout_dir("binary", "res/bin") && pf.set_layout_dir("binary", "data"));
        let back: ProjectFile = serde_json::from_str(&serde_json::to_string(&pf).unwrap()).unwrap();
        assert_eq!(back, pf);
    }
}
