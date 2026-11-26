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
    pub label: String,
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
            (Category::Text, td, vc_i18n::t!("reorganize-test"))
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
        if r["ok"] != true { return Err(vc_i18n::t!("reorganize-p1-laesst-sich-nicht-bauen", p1 = r["file"].as_str().unwrap_or("?"))); }
        if let Some(a) = r["artifact"].as_str() {
            let bytes = std::fs::read(root.join(a)).map_err(|e| vc_i18n::t!("reorganize-a-e", a = &a, e = &e))?;
            let mut h = std::collections::hash_map::DefaultHasher::new(); bytes.hash(&mut h);
            out.insert(a.to_string(), h.finish());
        }
    }
    if out.is_empty() { return Err(vc_i18n::t!("reorganize-keine-ergebnisdateien").into()); }
    Ok(out)
}

/// Bericht einer Umordnung
#[derive(Clone, Debug)]
pub struct Report { pub moved: Vec<(String, String)>, pub rewritten: Vec<(String, usize)>, pub backup: PathBuf, pub check: Check }

fn hash_bytes(b: &[u8]) -> u64 { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); b.hash(&mut h); h.finish() }

/// Letzte Umordnung, die noch zurückgenommen werden kann
#[derive(Clone, Debug)]
pub struct Letzte { pub backup: PathBuf, pub zeit: u64, pub moves: Vec<(String, String)> }

/// Neueste Sicherung mit `moves.txt` unter `.vcbackup/`, deren verschobene Dateien noch an den neuen Orten liegen
pub fn letzte(root: &Path) -> Option<Letzte> {
    let rd = std::fs::read_dir(root.join(".vcbackup")).ok()?;
    let mut v: Vec<(u64, PathBuf)> = rd.flatten().filter_map(|e| { let n = e.file_name().to_string_lossy().into_owned(); let zeit = n.split('-').next()?.parse::<u64>().ok()?; let sub = n.split_once('-').and_then(|(_, r)| r.parse::<u64>().ok()).unwrap_or(0); e.path().join("moves.txt").is_file().then_some((zeit * 1000 + sub, e.path())) }).collect();
    v.sort();
    let (key, backup) = v.pop()?;
    let moves = lies_moves(root, &backup).ok()?;
    if moves.is_empty() || !moves.iter().any(|(_, to)| root.join(to).is_file()) { return None }
    Some(Letzte { backup, zeit: key / 1000, moves })
}

/// Gibt es eine neueste Sicherung, die nicht von diesem Programm stammt? (Dann „Sicherung stammt nicht von VintageCoder“ anzeigen, nichts tun.)
pub fn fremde_sicherung(root: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(root.join(".vcbackup")) else { return false };
    rd.flatten().any(|e| e.path().join("moves.txt").is_file() && !ist_echt(&e.path()))
}

/// Gründe, aus denen „Rückgängig“ nichts überschreiben darf: Datei fehlt oder wurde geändert, alter Ort belegt
pub fn undo_konflikte(root: &Path, l: &Letzte) -> Vec<String> {
    let mut out = vec![];
    for (from, to) in &l.moves {
        if !root.join(to).is_file() { out.push(vc_i18n::t!("reorganize-to-fehlt-geloescht-oder-verschoben", to = &to)) }
        else if root.join(from).exists() { out.push(vc_i18n::t!("reorganize-from-ist-am-alten-ort-schon", from = &from)) }
    }
    if let Ok(t) = std::fs::read_to_string(l.backup.join("after.txt")) {
        for z in t.lines() { let Some((h, rel)) = z.split_once('\t') else { continue };
            let Ok(pfad) = crate::safepath::resolve(root, rel) else { continue };
            if let Ok(b) = std::fs::read(pfad) { if hash_bytes(&b).to_string() != h { out.push(vc_i18n::t!("reorganize-rel-seit-dem-umordnen-geaendert", rel = &rel)) } } }
    }
    out.sort(); out.dedup(); out
}

/// Nimmt die letzte Umordnung zurück, aber nur ohne Konflikte; sonst Fehler mit der Liste
pub fn undo_geprueft(root: &Path, l: &Letzte) -> Result<usize, String> {
    let k = undo_konflikte(root, l); if !k.is_empty() { return Err(vc_i18n::t!("reorganize-nicht-rueckgaengig-gemacht-damit-nichts-ueberschrieben", p1 = k.join("\n"))) }
    undo(root, &l.backup)
}

fn stamp() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{t}")
}

/// Führt die Verschiebungen aus (Konflikte werden übersprungen). `file`: Pfad der `.vcprj`. Mit `data` (Hardware-Datenbank) wird vorher und
/// nachher gebaut und verglichen. Bei einem Fehler mittendrin wird alles zurückgenommen.
pub fn apply(root: &Path, file: &Path, pf: &mut ProjectFile, moves: &[Move], data: Option<&Path>, rewrite: bool, progress: &dyn Fn(&str)) -> Result<Report, String> {
    let moves: Vec<Move> = moves.iter().filter(|m| !m.conflict && m.from != m.to).cloned().collect();
    if moves.is_empty() { return Err(vc_i18n::t!("reorganize-nichts-zu-verschieben").into()); }
    for m in &moves {
        crate::safepath::resolve(root, &m.from)?; crate::safepath::resolve(root, &m.to)?;
        if !root.join(&m.from).is_file() { return Err(vc_i18n::t!("reorganize-p1-datei-fehlt", p1 = &m.from)); }
        if root.join(&m.to).exists() { return Err(vc_i18n::t!("reorganize-p1-ziel-existiert-schon", p1 = &m.to)); }
        if !importfiles::valid_dir(dir_of(&m.to)) { return Err(vc_i18n::t!("reorganize-p1-ungueltiger-zielordner", p1 = &m.to)); }
    }
    let rw = if rewrite { rewrites(root, &moves) } else { vec![] };
    // Vorher bauen (nur zum Vergleich; scheitert es schon vorher, entfällt die Prüfung)
    if data.is_some() { progress(&vc_i18n::t!("reorganize-vorher-bauen")); }
    let before = data.map(|d| build_hashes(root, d, pf));
    progress(&vc_i18n::t!("reorganize-sicherung-anlegen"));
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
        std::fs::copy(root.join(rel), &dest).map(|_| ()).map_err(|e| vc_i18n::t!("reorganize-rel-e", rel = &rel, e = &e))
    };
    for (f, _, _) in &rw { save(f)?; }
    save(&vcprj_rel)?;
    vc_core::atomar::schreiben(backup.join("moves.txt"), moves.iter().map(|m| format!("{}\t{}\n", m.from, m.to)).collect::<String>()).map_err(|e| e.to_string())?;
    progress(&vc_i18n::t!("reorganize-dateien-verschieben"));
    // Ausführen: erst Texte (am alten Ort), dann Dateien verschieben
    let mut done: Vec<&Move> = vec![];
    let result: Result<(), String> = (|| {
        for (f, text, _) in &rw { vc_core::atomar::schreiben(root.join(f), text).map_err(|e| vc_i18n::t!("reorganize-f-e", f = &f, e = &e))?; }
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
    if data.is_some() { progress(&vc_i18n::t!("reorganize-nachher-bauen-und-vergleichen")); }
    let check = match (before, data) {
        (Some(Ok(b)), Some(d)) => match build_hashes(root, d, pf) {
            Ok(a) => { let diff: Vec<String> = b.iter().filter(|(k, v)| a.get(*k) != Some(v)).map(|(k, _)| k.clone()).collect(); if diff.is_empty() { Check::Same(b.len()) } else { Check::Different(diff) } }
            Err(e) => Check::Different(vec![e]),
        },
        (Some(Err(e)), _) => Check::Skipped(vc_i18n::t!("reorganize-vorher-schon-nicht-baubar-e", e = &e)),
        _ => Check::Skipped(vc_i18n::t!("reorganize-nicht-angefordert").into()),
    };
    // Stand nach dem Umordnen festhalten: „Rückgängig“ erkennt später geänderte Dateien und überschreibt sie nicht
    { let mut zeilen = String::new(); let mut merke = |rel: &str| { if let Ok(b) = std::fs::read(root.join(rel)) { zeilen.push_str(&format!("{}\t{rel}\n", hash_bytes(&b))) } };
      for m in &moves { merke(&m.to) } for (f, _, _) in &rw { merke(f) } merke(&vcprj_rel); let _ = vc_core::atomar::schreiben(backup.join("after.txt"), zeilen); }
    beglaubigen(&backup);
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
    for (f, text, _) in rw { vc_core::atomar::schreiben(root.join(f), text).map_err(|e| vc_i18n::t!("reorganize-f-e", f = &f, e = &e))?; }
    Ok(())
}

/// Gesicherte Dateien zurückschreiben (an ihren ursprünglichen Ort)
fn restore(root: &Path, backup: &Path) -> Result<(), String> {
    let base = backup.join("files");
    let mut files = vec![]; sicherungsdateien(&base, &base, &mut files);
    for (rel, f) in files {
        let dest = crate::safepath::resolve(root, &rel)?;
        if let Some(p) = dest.parent() { let _ = std::fs::create_dir_all(p); }
        std::fs::copy(&f, &dest).map_err(|e| format!("{rel}: {e}"))?;
    }
    Ok(())
}

/// Macht eine Umordnung rückgängig: Dateien zurückverschieben, gesicherte Texte und Projektdatei wiederherstellen
pub fn undo(root: &Path, backup: &Path) -> Result<usize, String> {
    let moves = lies_moves(root, backup)?;
    let mut n = 0;
    for (from, to) in moves.iter().rev() {
        let (pf, pt) = (crate::safepath::resolve(root, from)?, crate::safepath::resolve(root, to)?);
        if !pt.is_file() { continue; }
        if pf.exists() { return Err(vc_i18n::t!("reorganize-from-existiert-schon-nicht-zurueckverschoben", from = &from)); }
        if let Some(p) = pf.parent() { let _ = std::fs::create_dir_all(p); }
        std::fs::rename(&pt, &pf).map_err(|e| vc_i18n::t!("reorganize-to-e", to = &to, e = &e))?;
        n += 1;
    }
    restore(root, backup)?;
    Ok(n)
}

// ---- Echtheit der Sicherung: `.vcbackup/` liegt im Projektordner und kann aus einem fremden Projekt (ZIP, Git) stammen.
// Eine Sicherung zählt nur, wenn ihr Fingerabdruck in der Liste im Konfigurationsordner dieses Programms steht;
// jeder Pfad daraus muss außerdem im Projektordner bleiben (siehe `safepath`).

fn sicherungsliste() -> Option<PathBuf> {
    #[cfg(test)]
    let d = { let d = std::env::temp_dir().join(format!("vc_reorg_liste_{}", std::process::id())); let _ = std::fs::create_dir_all(&d); d };   // Tests nie in den echten Konfigurationsordner
    #[cfg(not(test))]
    // `VC_CONFIG_DIR` ersetzt den Konfigurationsordner (Tests anderer Pakete und Prozesse, die `apply` aufrufen)
    let d = match std::env::var_os("VC_CONFIG_DIR").filter(|d| !d.is_empty()) { Some(d) => PathBuf::from(d), None => vc_core::userdirs::config_dir()? };
    Some(d.join("reorganize-sicherungen.txt"))
}

/// Fingerabdruck einer Sicherung: Ort, `moves.txt`, `after.txt` und Inhalt von `files/`
fn fingerabdruck(backup: &Path) -> String {
    let mut t = format!("{}\n", std::fs::canonicalize(backup).unwrap_or_else(|_| backup.to_path_buf()).display());
    for n in ["moves.txt", "after.txt"] { t.push_str(&format!("{n}\n")); t.push_str(&std::fs::read_to_string(backup.join(n)).unwrap_or_default()); }
    let mut fs = vec![]; sicherungsdateien(&backup.join("files"), &backup.join("files"), &mut fs);
    fs.sort();
    for (rel, p) in fs { t.push_str(&format!("{rel}\t{}\n", crate::turbo::sha256_hex(&std::fs::read(p).unwrap_or_default()))); }
    crate::turbo::sha256_hex(t.as_bytes())
}

/// Dateien unter `dir` (relativ zu `base`); Symlinks werden weder gelesen noch kopiert
fn sicherungsdateien(base: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(md) = std::fs::symlink_metadata(&p) else { continue };
        if md.file_type().is_symlink() { continue; }
        if md.is_dir() { sicherungsdateien(base, &p, out); }
        else if let Ok(r) = p.strip_prefix(base) { out.push((r.to_string_lossy().replace('\\', "/"), p)); }
    }
}

/// Trägt die Sicherung als echt ein (nach dem Schreiben von `moves.txt` und `after.txt`)
fn beglaubigen(backup: &Path) {
    let Some(f) = sicherungsliste() else { return };
    if let Some(d) = f.parent() { let _ = std::fs::create_dir_all(d); }
    use std::io::Write;
    // eine Zeile mit einem einzigen Schreibaufruf: gleichzeitige Schreiber (mehrere Fenster, Tests) vermischen sonst ihre Zeilen
    let zeile = format!("{}\n", fingerabdruck(backup));
    if let Ok(mut h) = std::fs::OpenOptions::new().create(true).append(true).open(&f) { let _ = h.write_all(zeile.as_bytes()); }
}

/// Stammt die Sicherung von diesem Programm (und ist unverändert)?
pub fn ist_echt(backup: &Path) -> bool {
    let Some(f) = sicherungsliste() else { return false };
    let fp = fingerabdruck(backup);
    std::fs::read_to_string(f).map_or(false, |t| t.lines().any(|l| l == fp))
}

/// Liest `moves.txt` einer Sicherung: nur wenn sie echt ist, und nur mit Pfaden, die im Projektordner bleiben
fn lies_moves(root: &Path, backup: &Path) -> Result<Vec<(String, String)>, String> {
    let list = std::fs::read_to_string(backup.join("moves.txt")).map_err(|e| vc_i18n::t!("reorganize-moves-txt-e", e = &e))?;
    if !ist_echt(backup) { return Err(vc_i18n::t!("reorganize-sicherung-nicht-von-vintagecoder").into()); }
    let moves: Vec<(String, String)> = list.lines().filter_map(|l| l.split_once('\t')).map(|(a, b)| (a.to_string(), b.to_string())).collect();
    for (a, b) in &moves { crate::safepath::resolve(root, a)?; crate::safepath::resolve(root, b)?; }
    Ok(moves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letzte_umordnung_spaeter_rueckgaengig_mit_konfliktpruefung() {
        sicher();
        let root = tmp("spaeter"); std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::write(root.join("p.vcprj"), "{\"project\":{\"name\":\"t\"}}").unwrap();
        std::fs::write(root.join("a.inc"), b"\x01\x02 inc").unwrap(); std::fs::write(root.join("m.asm"), "  .include \"a.inc\"\n").unwrap();
        let mut pf = ProjectFile::load(&root.join("p.vcprj")).unwrap();
        let moves = vec![Move { from: "a.inc".into(), to: "lib/a.inc".into(), category: Category::Text, label: "Test".into(), conflict: false }];
        let vorher: Vec<(String, Vec<u8>)> = ["a.inc", "m.asm", "p.vcprj"].iter().map(|f| (f.to_string(), std::fs::read(root.join(f)).unwrap())).collect();
        assert!(letzte(&root).is_none());
        let rep = apply(&root, &root.join("p.vcprj"), &mut pf, &moves, None, true, &|_| {}).unwrap(); assert!(root.join("lib/a.inc").is_file());
        // „Dialog schließen und neu öffnen“: nur der Platteninhalt zählt
        let l = letzte(&root).expect("Sicherung gefunden"); assert_eq!(l.backup, rep.backup); assert_eq!(l.moves, vec![("a.inc".to_string(), "lib/a.inc".to_string())]);
        // Konflikt: Datei geändert → nichts passiert
        std::fs::write(root.join("lib/a.inc"), b"anders").unwrap();
        let e = undo_geprueft(&root, &l).unwrap_err(); assert!(e.contains("geändert"), "{e}"); assert!(root.join("lib/a.inc").is_file() && !root.join("a.inc").exists());
        std::fs::write(root.join("lib/a.inc"), b"\x01\x02 inc").unwrap();
        // Konflikt: gelöscht
        std::fs::remove_file(root.join("lib/a.inc")).unwrap(); assert!(letzte(&root).is_none(), "ohne Dateien an den neuen Orten nicht anbieten"); std::fs::write(root.join("lib/a.inc"), b"\x01\x02 inc").unwrap();
        let l = letzte(&root).unwrap(); assert_eq!(undo_geprueft(&root, &l).unwrap(), 1);
        for (f, b) in vorher { assert_eq!(std::fs::read(root.join(&f)).unwrap(), b, "{f} wie vorher"); }
        let _ = std::fs::remove_dir_all(&root);
    }

    fn sicher() {}

    #[test]
    fn praeparierte_sicherung_bewegt_nichts() {
        sicher();
        let aussen = tmp("fremd-aussen");
        let root = tmp("fremd");
        std::fs::write(root.join("boese.asm"), "x").unwrap();
        let b = root.join(".vcbackup/1");
        std::fs::create_dir_all(&b).unwrap();
        let ziel = aussen.join("x.plist");
        let abs = ziel.to_string_lossy().into_owned();
        for pfad in ["../../../Library/LaunchAgents/x.plist", abs.as_str(), "C:\\Windows\\x", "\\\\server\\share\\x", "..\\x"] {
            std::fs::write(b.join("moves.txt"), format!("{pfad}\tboese.asm\n")).unwrap();
            // 1. nicht von VintageCoder: keine Aktion
            assert!(letzte(&root).is_none(), "{pfad}");
            assert!(fremde_sicherung(&root));
            let e = undo(&root, &b).unwrap_err(); assert!(e.contains("nicht von VintageCoder"), "{e}");
            // 2. selbst mit gültiger Beglaubigung: Pfadprüfung
            beglaubigen(&b);
            assert!(letzte(&root).is_none(), "{pfad}: ungültiger Pfad verworfen");
            let e = undo(&root, &b).unwrap_err(); assert!(e.contains("Unzul") || e.contains("Invalid"), "{e}");
            assert!(root.join("boese.asm").is_file() && !ziel.exists(), "{pfad}");
        }
        // after.txt mit fremdem Pfad wird nur ignoriert, nie gelesen
        std::fs::write(b.join("moves.txt"), "a.inc\tboese.asm\n").unwrap();
        std::fs::write(b.join("after.txt"), "1\t../../etc/passwd\n").unwrap();
        beglaubigen(&b);
        let l = letzte(&root).unwrap();
        assert!(undo_konflikte(&root, &l).iter().all(|k| !k.contains("passwd")));
        // files/ mit Symlink nach draußen wird nicht zurückgespielt
        #[cfg(unix)]
        {
            std::fs::create_dir_all(b.join("files")).unwrap();
            std::fs::write(aussen.join("geheim.txt"), "geheim").unwrap();
            std::os::unix::fs::symlink(aussen.join("geheim.txt"), b.join("files/geheim.txt")).unwrap();
            beglaubigen(&b);
            let _ = undo(&root, &b);
            assert!(!root.join("geheim.txt").exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlink_im_projekt_nach_draussen_wird_abgelehnt() {
        sicher();
        let aussen = tmp("sym2-aussen");
        let root = tmp("sym2");
        std::fs::write(root.join("neu.asm"), "x").unwrap();
        std::os::unix::fs::symlink(&aussen, root.join("link")).unwrap();
        let b = root.join(".vcbackup/1"); std::fs::create_dir_all(&b).unwrap();
        std::fs::write(b.join("moves.txt"), "link/alt.asm\tneu.asm\n").unwrap();
        beglaubigen(&b);
        assert!(undo(&root, &b).is_err());
        assert!(!aussen.join("alt.asm").exists() && root.join("neu.asm").is_file());
    }

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
        sicher();
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
        let bad = Move { from: "a.bin".into(), to: "../x/a.bin".into(), category: Category::Binary, label: "x".into(), conflict: false };
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
        assert_eq!(p.iter().find(|m| m.from == "held.vcsprite").map(|m| (m.to.as_str(), m.label.as_str())), Some(("gfx/held.vcsprite", "Editor-Datei")));
        assert_eq!(p.iter().find(|m| m.from == "mathe.vctest").map(|m| (m.to.as_str(), m.label.as_str())), Some(("tests/mathe.vctest", "Test")));
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
