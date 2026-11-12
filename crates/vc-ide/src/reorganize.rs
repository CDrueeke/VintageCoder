// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateien neu ordnen: ein bestehendes Projekt nach `layout`, `rules` und den Kategorien in `assets` umsortieren.
//! Ablauf: **Plan** (Vorschlag je Datei) → **Anwenden** (Sicherung in `.vcbackup/<Zeit>/`, Dateien verschieben, `.include`/`.incbin`-Pfade
//! und die Projektdatei nachführen) → **Prüfbau** (gleiche Bytes wie vorher?) → bei Bedarf **Rückgängig**.
//! Quellprogramme aus `sources`, `main` und `disk`, die üblichen Kopfdateien (README …) und Dateien in auf ein Target beschränkten Ordnern (`folders`) bleiben, wo sie sind.
//! Studio-Dateien wandern nur, wenn `layout` einen Ordner für `studio` nennt; `.vctest`-Dateien nur in einen Ordner mit der Rolle „Tests“.
use crate::importfiles::{self, apply_rules, dir_for, Category, Plan};
use crate::projfile::ProjectFile;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Eine geplante Verschiebung
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: String,
    pub to: String,
    pub category: Category,
    /// Bezeichnung für die Anzeige (Kategorie oder „Test“)
    pub label: &'static str,
    /// Am Ziel liegt schon eine andere Datei (oder ein anderer Eintrag will dorthin): wird nicht verschoben
    pub conflict: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Opts {
    /// Auch Dateien in Unterordnern umordnen (sonst nur die im Projektordner)
    pub subdirs: bool,
}

/// Alle Dateien des Projekts (relativ, mit `/`); ohne versteckte Ordner, `build/` im Projektordner, `target`, `node_modules`
pub fn list_files(root: &Path) -> Vec<String> {
    fn go(root: &Path, dir: &Path, depth: usize, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
            let p = e.path();
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() { if !(depth == 0 && name == "build") && depth < 16 { go(root, &p, depth + 1, out); } }
            else if t.is_file() { if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")); } }
        }
    }
    let mut v = vec![]; go(root, root, 0, &mut v); v.sort(); v
}

fn dir_of(rel: &str) -> &str { rel.rsplit_once('/').map_or("", |x| x.0) }
fn name_of(rel: &str) -> &str { rel.rsplit_once('/').map_or(rel, |x| x.1) }

/// Bleibt die Datei, wo sie ist? (Programm, Studio-Datei, Test, Projektdatei, Kopfdatei)
fn pinned(pf: &ProjectFile, rel: &str) -> bool {
    let l = rel.to_ascii_lowercase();
    let n = name_of(&l);
    l.ends_with(".vcprj")
        || n.starts_with("readme") || n.starts_with("license") || n.starts_with("licence") || n.starts_with("changelog") || n.starts_with("makefile")
        || pf.source_files().iter().any(|s| s.0 == rel) || pf.main.as_deref() == Some(rel) || pf.disk.as_ref().map_or(false, |d| d.files.iter().any(|f| f == rel))
}

/// Kategorie und Zielordner einer Datei: Eintrag in `assets`, sonst erkannte Art; Ordner aus Regel, sonst `layout`
fn target_of(root: &Path, pf: &ProjectFile, rel: &str) -> Option<(Category, String)> {
    let mut plan = Plan::suggest_with(&root.join(rel), &pf.layout, &pf.rules);
    if let Some(c) = pf.asset(rel).and_then(|a| Category::from_id(&a.category)) {
        plan.category = c;
        plan.dir = apply_rules(&pf.rules, name_of(rel)).and_then(|r| r.1).unwrap_or_else(|| dir_for(&pf.layout, c));
    }
    Some((plan.category, plan.dir))
}

/// Vorschlag, wohin welche Datei soll. Dateien, die schon im Zielordner (oder darunter) liegen, bleiben.
pub fn plan(root: &Path, pf: &ProjectFile, opts: &Opts) -> Vec<Move> {
    let files = list_files(root);
    let existing: HashSet<&str> = files.iter().map(String::as_str).collect();
    let mut out: Vec<Move> = vec![];
    for rel in &files {
        if pinned(pf, rel) || pf.in_target_folder(rel) { continue; }
        let dir = dir_of(rel);
        if !opts.subdirs && !dir.is_empty() { continue; }
        let l = rel.to_ascii_lowercase();
        let (category, tdir, label) = if l.ends_with(".vctest") {
            // Tests gehören in den Ordner mit der Rolle „Tests“; gibt es keinen, bleiben sie
            let Some(td) = pf.folders.iter().find(|f| f.role == "tests" && f.targets.is_empty()).map(|f| f.dir.clone()) else { continue };
            (Category::Text, td, "Test")
        } else {
            let Some((c, d)) = target_of(root, pf, rel) else { continue };
            // Studio-Dateien nur, wenn layout einen Ordner nennt (sonst blieben sie in der Wurzel, nicht dahin zurück)
            if c == Category::Studio && d.is_empty() { continue; }
            (c, d, c.label())
        };
        if !importfiles::valid_dir(&tdir) { continue; }
        if dir == tdir || (!tdir.is_empty() && dir.starts_with(&format!("{tdir}/"))) { continue; }
        let to = importfiles::join_rel(&tdir, name_of(rel));
        let conflict = existing.contains(to.as_str());
        out.push(Move { from: rel.clone(), to, category, label, conflict });
    }
    // zwei Dateien mit demselben Ziel: die zweite (und jede weitere) ist ein Konflikt
    let mut seen: HashSet<String> = HashSet::new();
    for m in &mut out { if !seen.insert(m.to.clone()) { m.conflict = true; } }
    out
}

fn normalize(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = vec![];
    for c in path.split('/') { match c { "" | "." => {} ".." => { parts.pop()?; } s => parts.push(s) } }
    Some(parts.join("/"))
}

/// Relativer Pfad von Ordner `from_dir` zur Datei `to`
pub fn rel_path(from_dir: &str, to: &str) -> String {
    let a: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let b: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<&str> = vec![".."; a.len() - common];
    parts.extend(&b[common..]);
    parts.join("/")
}

/// Schreibt die Pfade in Zeichenketten (`"…"`) um, die auf eine vorhandene Projektdatei zeigen. `old_dir` ist der Ordner, in dem der Text
/// bisher lag (dort sind die Pfade aufzulösen), `new_dir` der Ordner danach; `mapped` liefert den neuen Ort einer Datei.
/// Liefert den neuen Text und die Zahl geänderter Pfade.
pub fn rewrite_paths(text: &str, old_dir: &str, new_dir: &str, exists: &dyn Fn(&str) -> bool, mapped: &dyn Fn(&str) -> String) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut n = 0;
    let mut rest = text;
    while let Some(i) = rest.find('"') {
        out.push_str(&rest[..=i]);
        rest = &rest[i + 1..];
        let end = rest.find(['"', '\n']).filter(|&e| rest.as_bytes()[e] == b'"');
        let Some(e) = end else { continue };
        let lit = &rest[..e];
        let mut repl = None;
        if !lit.is_empty() && !lit.starts_with('/') && !lit.contains('\\') && !lit.contains('{') {
            if let Some(target) = normalize(&importfiles::join_rel(old_dir, lit)) {
                if exists(&target) {
                    let new = rel_path(new_dir, &mapped(&target));
                    if new != lit { repl = Some(new); }
                }
            }
        }
        match repl { Some(r) => { out.push_str(&r); n += 1; } None => out.push_str(lit) }
        out.push('"');
        rest = &rest[e + 1..];
    }
    out.push_str(rest);
    (out, n)
}

fn is_code(rel: &str) -> bool {
    let l = rel.to_ascii_lowercase();
    [".asm", ".inc", ".s", ".a65", ".i65", ".src", ".bas", ".lst", ".sbas"].iter().any(|e| l.ends_with(e))
}

/// Geänderte Quelltexte: (Datei am alten Ort, neuer Text, Zahl geänderter Pfade)
pub fn rewrites(root: &Path, moves: &[Move]) -> Vec<(String, String, usize)> {
    let files = list_files(root);
    let existing: HashSet<&str> = files.iter().map(String::as_str).collect();
    let map: HashMap<&str, &str> = moves.iter().map(|m| (m.from.as_str(), m.to.as_str())).collect();
    let mapped = |p: &str| map.get(p).map_or_else(|| p.to_string(), |t| t.to_string());
    let mut out = vec![];
    for f in files.iter().filter(|f| is_code(f)) {
        let Ok(text) = std::fs::read_to_string(root.join(f)) else { continue };
        let new_dir = map.get(f.as_str()).map_or(dir_of(f), |t| dir_of(t)).to_string();
        let (nt, n) = rewrite_paths(&text, dir_of(f), &new_dir, &|p| existing.contains(p), &mapped);
        if n > 0 { out.push((f.clone(), nt, n)); }
    }
    out
}

/// Ergebnis des Prüfbaus
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check { Skipped(String), Same(usize), Different(Vec<String>) }

fn build_hashes(root: &Path, data: &Path, pf: &ProjectFile) -> Result<BTreeMap<String, u64>, String> {
    use std::hash::{Hash, Hasher};
    let v = crate::projfile::build_project(&crate::build::Ctx { root, data }, pf, None);
    let mut out = BTreeMap::new();
    for r in v["results"].as_array().into_iter().flatten() {
        if r["ok"] != true { return Err(format!("{} lässt sich nicht bauen", r["file"].as_str().unwrap_or("?"))); }
        if let Some(a) = r["artifact"].as_str() {
            let bytes = std::fs::read(root.join(a)).map_err(|e| format!("{a}: {e}"))?;
            let mut h = std::collections::hash_map::DefaultHasher::new(); bytes.hash(&mut h);
            out.insert(a.to_string(), h.finish());
        }
    }
    if out.is_empty() { return Err("keine Ergebnisdateien".into()); }
    Ok(out)
}

/// Bericht einer Umordnung
#[derive(Clone, Debug)]
pub struct Report { pub moved: Vec<(String, String)>, pub rewritten: Vec<(String, usize)>, pub backup: PathBuf, pub check: Check }

fn stamp() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{t}")
}

/// Führt die Verschiebungen aus (Konflikte werden übersprungen). `file`: Pfad der `.vcprj`. Mit `data` (Hardware-Datenbank) wird vorher und
/// nachher gebaut und verglichen. Bei einem Fehler mittendrin wird alles zurückgenommen.
pub fn apply(root: &Path, file: &Path, pf: &mut ProjectFile, moves: &[Move], data: Option<&Path>, rewrite: bool, progress: &dyn Fn(&str)) -> Result<Report, String> {
    let moves: Vec<Move> = moves.iter().filter(|m| !m.conflict && m.from != m.to).cloned().collect();
    if moves.is_empty() { return Err("nichts zu verschieben".into()); }
    for m in &moves {
        if !root.join(&m.from).is_file() { return Err(format!("{}: Datei fehlt", m.from)); }
        if root.join(&m.to).exists() { return Err(format!("{}: Ziel existiert schon", m.to)); }
        if !importfiles::valid_dir(dir_of(&m.to)) { return Err(format!("{}: ungültiger Zielordner", m.to)); }
    }
    let rw = if rewrite { rewrites(root, &moves) } else { vec![] };
    // Vorher bauen (nur zum Vergleich; scheitert es schon vorher, entfällt die Prüfung)
    if data.is_some() { progress("Vorher bauen …"); }
    let before = data.map(|d| build_hashes(root, d, pf));
    progress("Sicherung anlegen …");
    // Sicherung
    let backup = root.join(".vcbackup").join(stamp());
    let mut n = 0;
    let mut backup_dir = backup.clone();
    while backup_dir.exists() { n += 1; backup_dir = root.join(".vcbackup").join(format!("{}-{n}", stamp())); }
    let backup = backup_dir;
    let vcprj_rel = file.strip_prefix(root).unwrap_or(file).to_string_lossy().replace('\\', "/");
    let save = |rel: &str| -> Result<(), String> {
        let dest = backup.join("files").join(rel);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::copy(root.join(rel), &dest).map(|_| ()).map_err(|e| format!("{rel}: {e}"))
    };
    for (f, _, _) in &rw { save(f)?; }
    save(&vcprj_rel)?;
    std::fs::write(backup.join("moves.txt"), moves.iter().map(|m| format!("{}\t{}\n", m.from, m.to)).collect::<String>()).map_err(|e| e.to_string())?;
    progress("Dateien verschieben …");
    // Ausführen: erst Texte (am alten Ort), dann Dateien verschieben
    let mut done: Vec<&Move> = vec![];
    let result: Result<(), String> = (|| {
        for (f, text, _) in &rw { std::fs::write(root.join(f), text).map_err(|e| format!("{f}: {e}"))?; }
        for m in &moves {
            let dest = root.join(&m.to);
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::rename(root.join(&m.from), &dest).map_err(|e| format!("{}: {e}", m.from))?;
            done.push(m);
        }
        for m in &moves { pf.rename_path(&m.from, &m.to); }
        pf.save(file)
    })();
    if let Err(e) = result {
        for m in done.iter().rev() { let _ = std::fs::rename(root.join(&m.to), root.join(&m.from)); }
        let _ = restore(root, &backup);
        return Err(e);
    }
    if data.is_some() { progress("Nachher bauen und vergleichen …"); }
    let check = match (before, data) {
        (Some(Ok(b)), Some(d)) => match build_hashes(root, d, pf) {
            Ok(a) => { let diff: Vec<String> = b.iter().filter(|(k, v)| a.get(*k) != Some(v)).map(|(k, _)| k.clone()).collect(); if diff.is_empty() { Check::Same(b.len()) } else { Check::Different(diff) } }
            Err(e) => Check::Different(vec![e]),
        },
        (Some(Err(e)), _) => Check::Skipped(format!("vorher schon nicht baubar: {e}")),
        _ => Check::Skipped("nicht angefordert".into()),
    };
    prune_backups(root, KEEP_BACKUPS, &backup);
    Ok(Report { moved: moves.iter().map(|m| (m.from.clone(), m.to.clone())).collect(), rewritten: rw.iter().map(|(f, _, n)| (f.clone(), *n)).collect(), backup, check })
}

/// So viele Sicherungen bleiben in `.vcbackup/` (die neueste und die davor)
pub const KEEP_BACKUPS: usize = 5;

/// Löscht die ältesten Sicherungen, bis höchstens `keep` bleiben; `current` wird nie gelöscht
pub fn prune_backups(root: &Path, keep: usize, current: &Path) {
    let Ok(rd) = std::fs::read_dir(root.join(".vcbackup")) else { return };
    let mut v: Vec<(u64, u64, PathBuf)> = rd.flatten().filter(|e| e.path().is_dir()).map(|e| {
        let n = e.file_name().to_string_lossy().into_owned();
        let (a, b) = n.split_once('-').unwrap_or((n.as_str(), "0"));
        (a.parse().unwrap_or(0), b.parse().unwrap_or(0), e.path())
    }).collect();
    v.sort();
    let excess = v.len().saturating_sub(keep);
    for (_, _, p) in v.into_iter().take(excess) { if p != current { let _ = std::fs::remove_dir_all(p); } }
}

/// Schreibt die Quelltexte aus [`rewrites`] zurück (am bisherigen Ort), z. B. vor dem Umbenennen einer eingebundenen Datei
pub fn write_rewrites(root: &Path, rw: &[(String, String, usize)]) -> Result<(), String> {
    for (f, text, _) in rw { std::fs::write(root.join(f), text).map_err(|e| format!("{f}: {e}"))?; }
    Ok(())
}

/// Gesicherte Dateien zurückschreiben (an ihren ursprünglichen Ort)
fn restore(root: &Path, backup: &Path) -> Result<(), String> {
    let base = backup.join("files");
    fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) { if let Ok(rd) = std::fs::read_dir(dir) { for e in rd.flatten() { let p = e.path(); if p.is_dir() { walk(base, &p, out); } else { out.push(p); } } } }
    let mut files = vec![]; walk(&base, &base, &mut files);
    for f in files {
        let rel = f.strip_prefix(&base).map_err(|e| e.to_string())?;
        let dest = root.join(rel);
        if let Some(p) = dest.parent() { let _ = std::fs::create_dir_all(p); }
        std::fs::copy(&f, &dest).map_err(|e| format!("{}: {e}", rel.display()))?;
    }
    Ok(())
}

/// Macht eine Umordnung rückgängig: Dateien zurückverschieben, gesicherte Texte und Projektdatei wiederherstellen
pub fn undo(root: &Path, backup: &Path) -> Result<usize, String> {
    let list = std::fs::read_to_string(backup.join("moves.txt")).map_err(|e| format!("moves.txt: {e}"))?;
    let mut n = 0;
    for l in list.lines().rev() {
        let Some((from, to)) = l.split_once('\t') else { continue };
        if !root.join(to).is_file() { continue; }
        if root.join(from).exists() { return Err(format!("{from}: existiert schon, nicht zurückverschoben")); }
        if let Some(p) = root.join(from).parent() { let _ = std::fs::create_dir_all(p); }
        std::fs::rename(root.join(to), root.join(from)).map_err(|e| format!("{to}: {e}"))?;
        n += 1;
    }
    restore(root, backup)?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc_reorg_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
    }
    fn data() -> PathBuf { crate::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data/") }

    #[test]
    fn pfade_umschreiben_relativ_zum_neuen_ort() {
        let ex = |p: &str| ["font.bin", "lib/t.asm", "data/x.bin"].contains(&p);
        let map = |p: &str| match p { "font.bin" => "res/fonts/font.bin".to_string(), "lib/t.asm" => "src/lib/t.asm".into(), o => o.into() };
        // main.asm bleibt im Projektordner
        let (t, n) = rewrite_paths("  .incbin \"font.bin\", 2\n  .include \"lib/t.asm\"\n  .text \"hallo\"\n", "", "", &ex, &map);
        assert_eq!(t, "  .incbin \"res/fonts/font.bin\", 2\n  .include \"src/lib/t.asm\"\n  .text \"hallo\"\n");
        assert_eq!(n, 2);
        // t.asm zieht von lib/ nach src/lib/ um: Verweis auf font.bin ändert sich mit
        let (t, n) = rewrite_paths(".incbin \"../font.bin\"", "lib", "src/lib", &ex, &map);
        assert_eq!((t.as_str(), n), (".incbin \"../../res/fonts/font.bin\"", 1));
        // Datei bleibt, Ziel bleibt: nichts ändern (auch nicht bei vorhandener Datei)
        let (t, n) = rewrite_paths(".incbin \"x.bin\" ; \"data/x.bin\"", "data", "data", &|p| p == "data/x.bin", &|p| p.to_string());
        assert_eq!(n, 0); assert!(t.contains("\"x.bin\""));
        // nicht auflösbar / nur ein Anführungszeichen: unberührt
        assert_eq!(rewrite_paths("a \"nix.bin\" b \"offen", "", "", &ex, &map), ("a \"nix.bin\" b \"offen".to_string(), 0));
        assert_eq!(rel_path("src/lib", "res/a.bin"), "../../res/a.bin");
        assert_eq!(rel_path("", "a/b.bin"), "a/b.bin");
    }

    #[test]
    fn plan_laesst_programme_und_passende_dateien_stehen() {
        let root = tmp("plan");
        std::fs::create_dir_all(root.join("data")).unwrap(); std::fs::create_dir_all(root.join("sub")).unwrap();
        for (f, c) in [("main.asm", "x"), ("tab.asm", "x"), ("font.bin", "\0\0\0"), ("notiz.txt", "hi"), ("README.md", "r"), ("held.vcsprite", "{}"), ("data/ok.bin", "\0"), ("sub/versteckt.bin", "\0"), ("a.vcprj", "{}")] { std::fs::write(root.join(f), c).unwrap(); }
        let mut pf: ProjectFile = serde_json::from_str(r#"{"sources":[{"targets":["c64"],"kind":"asm","files":["main.asm"]}],"layout":{"binary":"data","text":"doc","source":"src/lib"}}"#).unwrap();
        let p = plan(&root, &pf, &Opts::default());
        let got: Vec<(&str, &str)> = p.iter().map(|m| (m.from.as_str(), m.to.as_str())).collect();
        assert_eq!(got, vec![("font.bin", "data/font.bin"), ("notiz.txt", "doc/notiz.txt"), ("tab.asm", "src/lib/tab.asm")], "Programm, README, Studio, Projektdatei, Unterordner bleiben");
        assert!(plan(&root, &pf, &Opts { subdirs: true }).iter().any(|m| m.from == "sub/versteckt.bin" && m.to == "data/versteckt.bin"));
        // Dateien in einem Ordner für ein bestimmtes Target bleiben
        std::fs::create_dir_all(root.join("sub/c64")).unwrap(); std::fs::write(root.join("sub/c64/x.bin"), "\0").unwrap();
        let mut pf2 = pf.clone(); pf2.set_folder_target("sub/c64", Some("c64"));
        assert!(plan(&root, &pf2, &Opts { subdirs: true }).iter().all(|m| m.from != "sub/c64/x.bin"));
        assert!(plan(&root, &pf, &Opts { subdirs: true }).iter().any(|m| m.from == "sub/c64/x.bin"));
        // Konflikt: Ziel belegt
        std::fs::write(root.join("data/font.bin"), "\0anders").unwrap();
        assert!(plan(&root, &pf, &Opts::default()).iter().find(|m| m.from == "font.bin").unwrap().conflict);
        // Kategorie aus assets geht vor der Erkennung
        pf.assets.push(crate::projfile::Asset { file: "notiz.txt".into(), category: "binary".into(), description: String::new() });
        assert_eq!(plan(&root, &pf, &Opts::default()).iter().find(|m| m.from == "notiz.txt").unwrap().to, "data/notiz.txt");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn umordnen_verschiebt_schreibt_pfade_um_baut_gleich_und_macht_rueckgaengig() {
        let root = tmp("apply");
        vc_media::wizard::create(&data(), &root, "reorg", &["c64".to_string()], "asm-hello").unwrap();
        let main = root.join("src/c64/main.asm");
        // Quelltext in src/c64/, Daten und Tabelle im Projektordner (alter Stil), Verweise relativ zur Quelle
        std::fs::write(root.join("font.bin"), [0u8, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        std::fs::write(root.join("tab.asm"), "tab: .byte 1,2,3\n").unwrap();
        let mut t = std::fs::read_to_string(&main).unwrap();
        t.push_str("\nfont: .incbin \"../../font.bin\"\n.include \"../../tab.asm\"\n");
        std::fs::write(&main, &t).unwrap();
        let file = root.join("reorg.vcprj");
        let mut pf = ProjectFile::load(&file).unwrap();
        pf.layout.insert("binary".into(), "res/bin".into()); pf.layout.insert("source".into(), "src/lib".into());
        pf.assets.push(crate::projfile::Asset { file: "font.bin".into(), category: "binary".into(), description: "Zeichensatz".into() });
        pf.save(&file).unwrap();
        let moves = plan(&root, &pf, &Opts::default());
        assert_eq!(moves.len(), 2, "{moves:?}");
        let pre = rewrites(&root, &moves);
        assert_eq!(pre.len(), 1, "nur main.asm ändert sich");
        let rep = apply(&root, &file, &mut pf, &moves, Some(&data()), true, &|_| {}).unwrap();
        assert!(root.join("res/bin/font.bin").is_file() && root.join("src/lib/tab.asm").is_file() && !root.join("font.bin").exists());
        let nt = std::fs::read_to_string(&main).unwrap();
        assert!(nt.contains("\"../../res/bin/font.bin\"") && nt.contains("\"../lib/tab.asm\""), "{nt}");
        assert_eq!(pf.asset("res/bin/font.bin").map(|a| a.description.as_str()), Some("Zeichensatz"), "assets nachgeführt");
        assert_eq!(ProjectFile::load(&file).unwrap(), pf, "Projektdatei gespeichert");
        assert!(matches!(rep.check, Check::Same(n) if n >= 1), "{:?}", rep.check);
        assert_eq!(rep.rewritten.len(), 1);
        // Rückgängig
        assert_eq!(undo(&root, &rep.backup).unwrap(), 2);
        assert!(root.join("font.bin").is_file() && root.join("tab.asm").is_file() && !root.join("res/bin/font.bin").exists());
        assert_eq!(std::fs::read_to_string(&main).unwrap(), t);
        assert!(ProjectFile::load(&file).unwrap().asset("font.bin").is_some());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn fehlerhafte_oder_leere_umordnung_wird_abgelehnt() {
        let root = tmp("bad");
        std::fs::write(root.join("a.bin"), "\0").unwrap();
        let file = root.join("p.vcprj");
        let mut pf = ProjectFile::default(); pf.save(&file).unwrap();
        assert!(apply(&root, &file, &mut pf, &[], None, true, &|_| {}).is_err());
        let bad = Move { from: "a.bin".into(), to: "../x/a.bin".into(), category: Category::Binary, label: "x", conflict: false };
        assert!(apply(&root, &file, &mut pf, &[bad], None, true, &|_| {}).is_err());
        assert!(root.join("a.bin").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn studio_und_tests_nur_mit_ziel_sicherungen_werden_gekuerzt() {
        let root = tmp("studio");
        std::fs::create_dir_all(root.join("tests")).unwrap();
        for f in ["held.vcsprite", "mathe.vctest", "x.bin"] { std::fs::write(root.join(f), if f == "x.bin" { "\0" } else { "{}" }).unwrap(); }
        let mut pf = ProjectFile::default();
        let p = plan(&root, &pf, &Opts::default());
        assert!(p.iter().all(|m| m.from != "held.vcsprite" && m.from != "mathe.vctest"), "ohne layout/Rolle bleiben beide");
        pf.layout.insert("studio".into(), "gfx".into());
        pf.set_role("tests", Some("tests"));
        let p = plan(&root, &pf, &Opts::default());
        assert_eq!(p.iter().find(|m| m.from == "held.vcsprite").map(|m| (m.to.as_str(), m.label)), Some(("gfx/held.vcsprite", "Studio-Datei")));
        assert_eq!(p.iter().find(|m| m.from == "mathe.vctest").map(|m| (m.to.as_str(), m.label)), Some(("tests/mathe.vctest", "Test")));
        // Sicherungen: nur die letzten KEEP_BACKUPS bleiben, die aktuelle nie weg
        for i in 0..8u64 { std::fs::create_dir_all(root.join(".vcbackup").join(format!("{}", 100 + i))).unwrap(); }
        std::fs::create_dir_all(root.join(".vcbackup/107-1")).unwrap();
        prune_backups(&root, 5, &root.join(".vcbackup/100"));
        let mut left: Vec<String> = std::fs::read_dir(root.join(".vcbackup")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(left.len(), 6, "{left:?}");
        assert!(left.contains(&"100".to_string()) && left.contains(&"107-1".to_string()) && !left.contains(&"101".to_string()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ohne_nachfuehren_bleiben_die_quelltexte_unberuehrt() {
        let root = tmp("norw");
        vc_media::wizard::create(&data(), &root, "norw", &["c64".to_string()], "asm-hello").unwrap();
        std::fs::write(root.join("tab.asm"), "t: .byte 1\n").unwrap();
        let main = root.join("src/c64/main.asm");
        let mut t = std::fs::read_to_string(&main).unwrap(); t.push_str("\n.include \"../../tab.asm\"\n"); std::fs::write(&main, &t).unwrap();
        let file = root.join("norw.vcprj");
        let mut pf = ProjectFile::load(&file).unwrap();
        let moves = plan(&root, &pf, &Opts::default());
        let rep = apply(&root, &file, &mut pf, &moves, Some(&data()), false, &|_| {}).unwrap();
        assert_eq!(std::fs::read_to_string(&main).unwrap(), t, "Text unverändert");
        assert!(matches!(rep.check, Check::Different(_)), "Prüfbau merkt, dass der Include nun ins Leere zeigt: {:?}", rep.check);
        let _ = std::fs::remove_dir_all(&root);
    }
}
