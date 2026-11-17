// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektordner der Oberfläche: dünne Schicht über `vc-ide-model` (Projektbaum, Gruppen, Suche über alle Dateien im Hintergrund).
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use vc_ide_model::project::{Entry, Group, Project};
use vc_ide_model::projsearch::{self, Params};

#[derive(Clone, Debug, PartialEq)]
pub struct Datei { pub rel: String }
#[derive(Clone, Debug, PartialEq)]
pub struct Gruppe { pub name: &'static str, pub dateien: Vec<Datei> }
#[derive(Clone, Debug, PartialEq)]
pub struct Fund { pub rel: String, pub zeile: usize, pub text: String }

/// Erste `.vcprj` im Ordner
fn erste_projektdatei(root: &Path) -> Option<String> {
    let mut v: Vec<String> = std::fs::read_dir(root).ok()?.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".vcprj")).collect();
    v.sort(); v.into_iter().next()
}

/// Projekt laden (Ordner mit der ersten `.vcprj` darin, falls vorhanden); eine unlesbare Projektdatei ist ein Fehler, kein stiller Ersatz
pub fn pruefen(root: &Path, datei: Option<String>) -> Result<Project, String> {
    let datei = datei.or_else(|| erste_projektdatei(root));
    if let Some(f) = &datei { vc_ide::projfile::ProjectFile::load(&root.join(f)).map_err(|e| format!("Projekt konnte nicht geladen werden: {e}"))?; }
    Project::load(root.to_path_buf(), datei)
}

/// Wie `pruefen`, aber ohne Fehler: bei kaputter Projektdatei ein Projekt ohne Projektdaten (Ordner bleibt sichtbar)
pub fn laden(root: &Path) -> Project {
    pruefen(root, None).unwrap_or_else(|_| Project::load(root.to_path_buf(), None).unwrap_or_else(|_| Project { root: root.to_path_buf(), file: None, name: String::new(), systems: vec![], entries: vec![], dirs: vec![], pf: None }))
}

/// Was ein gewählter Pfad zum Öffnen bedeutet: Projektordner, ggf. Projektdatei, ggf. eine Datei darin (relativ)
#[derive(Debug, Clone, PartialEq)]
pub struct Ziel { pub root: PathBuf, pub vcprj: Option<String>, pub datei: Option<String> }

/// Ordner → Projekt dort; `.vcprj` → Projekt dieser Datei; sonstige Datei → Projekt der nächsten `.vcprj` darüber (sonst `None`: Einzeldatei)
pub fn ziel_fuer(p: &Path) -> Option<Ziel> {
    if p.is_dir() { return Some(Ziel { root: p.to_path_buf(), vcprj: None, datei: None }); }
    let ordner = p.parent()?;
    let name = p.file_name()?.to_string_lossy().into_owned();
    if name.ends_with(".vcprj") { return Some(Ziel { root: ordner.to_path_buf(), vcprj: Some(name), datei: None }); }
    for a in ordner.ancestors() {
        if let Some(f) = erste_projektdatei(a) { let rel = p.strip_prefix(a).ok()?.to_string_lossy().into_owned(); return Some(Ziel { root: a.to_path_buf(), vcprj: Some(f), datei: Some(rel) }); }
    }
    None
}

/// Startprogramm: `main` der Projektdatei, sonst die erste `.asm`
pub fn startprogramm(p: &Project) -> Option<String> {
    p.pf.as_ref().and_then(|pf| pf.main_file()).filter(|m| p.entries.iter().any(|e| &e.rel == m)).or_else(|| { let n = dateien(p); n.iter().find(|n| n.ends_with(".asm") && !n.contains('/')).or_else(|| n.iter().find(|n| n.ends_with(".asm"))).cloned() })
}

/// Alle Dateien relativ zum Projekt (Reihenfolge des Baums: nach Art, dann alphabetisch)
pub fn dateien(p: &Project) -> Vec<String> { p.entries.iter().map(|e| e.rel.clone()).collect() }

/// Vom Export erzeugte Dateien (`build/export/…`): der Bauordner ist im Baum sonst ausgeblendet, diese Dateien bindet man aber per `.include` ein
pub fn exportierte(root: &Path) -> Vec<String> {
    fn lauf(root: &Path, dir: &Path, tiefe: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() { let p = e.path(); if e.file_name().to_string_lossy().starts_with('.') { continue } if p.is_dir() { if tiefe < 3 { lauf(root, &p, tiefe + 1, out) } } else if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")) } }
    }
    let mut v = vec![]; lauf(root, &root.join("build/export"), 0, &mut v); v.sort(); v
}

/// Nach Arten gruppiert (Reihenfolge wie `Group::ALL`, leere Gruppen fehlen)
pub fn gruppiere(entries: &[Entry]) -> Vec<Gruppe> {
    Group::ALL.iter().filter_map(|g| { let d: Vec<Datei> = entries.iter().filter(|e| e.group == *g).map(|e| Datei { rel: e.rel.clone() }).collect(); (!d.is_empty()).then(|| Gruppe { name: g.title(), dateien: d }) }).collect()
}

/// Eine Zeile des Dateibaums: Ordner (auf-/zuklappbar) oder Datei, eingerückt nach Tiefe
#[derive(Clone, Debug, PartialEq)]
pub struct BaumZeile { pub rel: String, pub name: String, pub tiefe: usize, pub ordner: bool, pub offen: bool, pub gruppe: Option<Group> }

/// Verzeichnisbaum aus Dateien und Ordnern des Projekts (`export`: Dateien aus `build/export`, die der Baum sonst ausblendet).
/// Ordner stehen vor Dateien, jeweils alphabetisch ohne Rücksicht auf Groß-/Kleinschreibung. `zu`: zugeklappte Ordner. Mit `filter` bleiben
/// nur passende Dateien (Teil des Pfades) und ihre Ordner, alle aufgeklappt.
pub fn baum_zeilen(p: &Project, zu: &std::collections::BTreeSet<String>, filter: &str, export: &[String]) -> Vec<BaumZeile> {
    use std::collections::BTreeMap;
    let f = filter.trim().to_lowercase();
    let mut dateien: Vec<(String, Option<Group>)> = p.entries.iter().map(|e| (e.rel.clone(), Some(e.group))).collect();
    let mut ordner: std::collections::BTreeSet<String> = p.dirs.iter().cloned().collect();
    for x in export { dateien.push((x.clone(), None)); let mut d = x.as_str(); while let Some((o, _)) = d.rsplit_once('/') { ordner.insert(o.to_string()); d = o } }
    if !f.is_empty() {
        dateien.retain(|(r, _)| r.to_lowercase().contains(&f));
        let mit: std::collections::BTreeSet<String> = dateien.iter().flat_map(|(r, _)| { let mut v = vec![]; let mut d = r.as_str(); while let Some((o, _)) = d.rsplit_once('/') { v.push(o.to_string()); d = o } v }).collect();
        ordner.retain(|o| mit.contains(o));
    }
    // Kinder je Ordner ("" = Wurzel): (Name, rel, ist Ordner, Gruppe)
    let mut kinder: BTreeMap<String, Vec<(String, String, bool, Option<Group>)>> = BTreeMap::new();
    let eltern = |rel: &str| rel.rsplit_once('/').map_or(String::new(), |(o, _)| o.to_string());
    let name_von = |rel: &str| rel.rsplit_once('/').map_or(rel.to_string(), |(_, n)| n.to_string());
    for o in &ordner { kinder.entry(eltern(o)).or_default().push((name_von(o), o.clone(), true, None)) }
    for (r, g) in &dateien { kinder.entry(eltern(r)).or_default().push((name_von(r), r.clone(), false, *g)) }
    for v in kinder.values_mut() { v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.to_lowercase().cmp(&b.0.to_lowercase())).then(a.0.cmp(&b.0))); v.dedup_by(|a, b| a.1 == b.1) }
    fn lauf(kinder: &BTreeMap<String, Vec<(String, String, bool, Option<Group>)>>, dir: &str, tiefe: usize, zu: &std::collections::BTreeSet<String>, filtern: bool, out: &mut Vec<BaumZeile>) {
        for (name, rel, ist_ordner, g) in kinder.get(dir).into_iter().flatten() {
            let offen = *ist_ordner && (filtern || !zu.contains(rel));
            out.push(BaumZeile { rel: rel.clone(), name: name.clone(), tiefe, ordner: *ist_ordner, offen, gruppe: *g });
            if offen { lauf(kinder, rel, tiefe + 1, zu, filtern, out) }
        }
    }
    let mut out = vec![]; lauf(&kinder, "", 0, zu, !f.is_empty(), &mut out); out
}

/// Datei, die nicht als Text geöffnet wird (Bilder, Binärdaten, Archive): öffnet im Standardprogramm des Systems
pub fn ist_nichttext(p: &Path) -> bool {
    let l = p.to_string_lossy().to_lowercase();
    if [".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".pdf", ".zip", ".gz", ".tar", ".7z", ".bin", ".dat", ".prg", ".crt", ".rom", ".sid", ".wav", ".mp3", ".ogg"].iter().any(|e| l.ends_with(e)) { return true }
    // unbekannte Endung: Inhalt entscheidet (NUL-Byte oder kein UTF-8/Latin-1-Text)
    std::fs::read(p).map(|b| { let kopf = &b[..b.len().min(4096)]; kopf.contains(&0) }).unwrap_or(false)
}

/// Textsuche über alle durchsuchbaren Dateien (Quelltext, BASIC, Textdateien); `ueberschreibt`: ungespeicherter Stand je Datei; Groß/Klein egal
pub fn suche(root: &Path, entries: &[Entry], ueberschreibt: HashMap<String, String>, frage: &str, max: usize) -> Vec<Fund> {
    let liste: Vec<(String, Group)> = entries.iter().map(|e| (e.rel.clone(), e.group)).collect();
    let p = Params { root: root.to_path_buf(), files: projsearch::candidate_files(&liste), overrides: ueberschreibt, text: frage.to_string(), case: false, value: None, limit: max };
    let mut out = vec![];
    projsearch::run(&p, &AtomicBool::new(false), &mut |h| out.push(Fund { rel: h.rel, zeile: h.line, text: h.text }));
    out
}

pub fn pfad(root: &Path, rel: &str) -> PathBuf { root.join(rel) }

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(name: &str) -> PathBuf { let p = std::env::temp_dir().join(format!("vcgui-test-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&p); std::fs::create_dir_all(&p).unwrap(); p }
    #[test] fn dateien_gruppiert_und_ohne_ausgabeordner() {
        let r = tmp("g"); std::fs::create_dir_all(r.join("lib")).unwrap(); std::fs::create_dir_all(r.join("build")).unwrap();
        for f in ["main.asm", "lib/rand.asm", "ideen.md", "build/x.prg", ".versteckt"] { std::fs::write(r.join(f), "x").unwrap() }
        let p = laden(&r); let d = dateien(&p); assert!(d.contains(&"lib/rand.asm".to_string()) && !d.iter().any(|x| x.starts_with("build") || x.starts_with('.')));
        let g = gruppiere(&p.entries); let namen: Vec<_> = g.iter().map(|x| x.name).collect(); assert_eq!(namen, [Group::Asm.title(), Group::Note.title()]);
        assert_eq!(g[0].dateien.len(), 2); let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn suche_findet_zeilen_und_beachtet_grenze_und_ungespeichertes() {
        let r = tmp("s"); std::fs::write(r.join("a.asm"), "lda #1\n  STA $d020\nsta $d021\n").unwrap(); std::fs::write(r.join("b.png"), "sta").unwrap();
        let p = laden(&r);
        let f = suche(&r, &p.entries, HashMap::new(), "sta", 10); assert_eq!(f.len(), 2); assert_eq!(f[0], Fund { rel: "a.asm".into(), zeile: 2, text: "STA $d020".into() });
        assert_eq!(suche(&r, &p.entries, HashMap::new(), "sta", 1).len(), 1); assert!(suche(&r, &p.entries, HashMap::new(), "", 5).is_empty());
        let mut u = HashMap::new(); u.insert("a.asm".to_string(), "nur neu\n".to_string()); assert!(suche(&r, &p.entries, u, "sta", 10).is_empty(), "ungespeicherter Stand geht vor"); let _ = std::fs::remove_dir_all(&r);
    }
}

#[cfg(test)]
mod baum_tests {
    use super::*;
    use std::collections::BTreeSet;
    fn proj(tag: &str) -> (Project, PathBuf) {
        let root = std::env::temp_dir().join(format!("vcgui_baum_{tag}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&root);
        for d in ["lib", "gfx/sub", "leer", "build/export"] { std::fs::create_dir_all(root.join(d)).unwrap() }
        for f in ["main.asm", "Zeta.txt", "alpha.bin", "projektbild.png", "lib/music.inc", "gfx/sub/x.vcchar", "build/export/a.asm", "p.vcprj", ".versteckt"] { std::fs::write(root.join(f), if f.ends_with("vcprj") { "{\"format\":1,\"project\":{\"name\":\"p\"}}" } else { "x" }).unwrap() }
        (Project::load(root.clone(), Some("p.vcprj".into())).unwrap(), root)
    }
    fn namen(z: &[BaumZeile]) -> Vec<String> { z.iter().map(|z| format!("{}{}{}", "  ".repeat(z.tiefe), z.name, if z.ordner { "/" } else { "" })).collect() }
    #[test]
    fn baum_zeigt_alle_dateien_ordner_vor_dateien_alphabetisch_und_klappt_zu() {
        let (p, root) = proj("alle"); let z = baum_zeilen(&p, &BTreeSet::new(), "", &[]);
        assert_eq!(namen(&z), ["gfx/", "  sub/", "    x.vcchar", "leer/", "lib/", "  music.inc", "alpha.bin", "main.asm", "p.vcprj", "projektbild.png", "Zeta.txt"], "alle Dateien (auch .png/.bin/.txt), versteckte und build/ nicht");
        assert!(z.iter().find(|x| x.name == "projektbild.png").is_some_and(|x| !x.ordner && x.gruppe.is_some()));
        let zu: BTreeSet<String> = ["gfx".to_string()].into_iter().collect(); let z = baum_zeilen(&p, &zu, "", &[]);
        assert_eq!(namen(&z)[..3], ["gfx/", "leer/", "lib/"], "zugeklappt: Kinder weg"); assert!(!z[0].offen);
        let z = baum_zeilen(&p, &BTreeSet::new(), "", &["build/export/a.asm".to_string()]); assert!(namen(&z).iter().any(|n| n == "build/") && namen(&z).iter().any(|n| n == "    a.asm"), "{:?}", namen(&z));
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn baum_filter_zeigt_treffer_mit_ihren_ordnern_aufgeklappt() {
        let (p, root) = proj("filter"); let zu: BTreeSet<String> = ["lib".to_string()].into_iter().collect();
        assert_eq!(namen(&baum_zeilen(&p, &zu, "MUSIC", &[])), ["lib/", "  music.inc"], "Filter ignoriert Groß/Klein und klappt Ordner auf");
        assert!(baum_zeilen(&p, &BTreeSet::new(), "gibtsnicht", &[]).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test]
    fn nichttext_erkennt_bilder_binaerdaten_und_laesst_text_im_editor() {
        let t = std::env::temp_dir().join(format!("vcgui_nt_{}", std::process::id())); std::fs::create_dir_all(&t).unwrap();
        std::fs::write(t.join("a.xyz"), [1u8, 0, 2]).unwrap(); std::fs::write(t.join("b.xyz"), "hallo").unwrap(); std::fs::write(t.join("c.asm"), "nop").unwrap();
        assert!(ist_nichttext(Path::new("bild.PNG")) && ist_nichttext(&t.join("a.xyz")) && !ist_nichttext(&t.join("b.xyz")) && !ist_nichttext(&t.join("c.asm")));
        let _ = std::fs::remove_dir_all(&t);
    }
}
