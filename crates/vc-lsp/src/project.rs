// SPDX-License-Identifier: GPL-3.0-or-later
//! Include-Graph des Projekts: welche Datei bindet welche ein, und welche Datei ist die **Wurzel**, in deren Kontext eine
//! eingebundene Datei zu prüfen ist (eine Include-Datei ist allein nicht übersetzbar: Marken, Makros, `.target`, `.org` stehen in der einbindenden Datei).
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vc_asm::{include_names, norm_path, resolve_include};

/// Dateiendungen, die als Assemblerquelltext gelten (wie in der IDE)
const EXTS: &[&str] = &["asm", "inc", "s", "a65", "i65", "src"];
const SKIP_DIRS: &[&str] = &["target", "build", "node_modules", "dist"];
const MAX_FILES: usize = 4000;
const MAX_DEPTH: usize = 10;

/// Merkt je Datei auf der Platte die Include-Namen (Schlüssel: Änderungszeit und Größe), damit ein Neuaufbau des Graphen nur geänderte Dateien liest.
#[derive(Default)]
pub struct DiskCache { names: HashMap<String, (Option<SystemTime>, u64, Vec<String>)> }

impl DiskCache {
    fn names_of(&mut self, path: &str) -> Vec<String> {
        let meta = std::fs::metadata(path).ok();
        let stamp = (meta.as_ref().and_then(|m| m.modified().ok()), meta.as_ref().map_or(0, |m| m.len()));
        if let Some((t, l, v)) = self.names.get(path) { if (*t, *l) == stamp { return v.clone(); } }
        let v = vc_core::textfile::read_source(path).map(|t| include_names(&t)).unwrap_or_default();
        self.names.insert(path.to_string(), (stamp.0, stamp.1, v.clone()));
        v
    }
}

#[derive(Default)]
pub struct Project {
    /// `main` der Projektdatei (normalisierter absoluter Pfad), falls vorhanden
    pub main: Option<String>,
    /// Ziel je Quelldatei laut Projektdatei (normalisierter Pfad → Target-ID)
    pub targets: HashMap<String, String>,
    /// Quelldateien der Projektdatei (Pfad normalisiert)
    pub sources: HashSet<String>,
    edges: HashMap<String, Vec<String>>,
    rev: HashMap<String, BTreeSet<String>>,
}

fn is_source(p: &Path) -> bool { p.extension().and_then(|e| e.to_str()).map_or(false, |e| EXTS.contains(&e.to_ascii_lowercase().as_str())) }

fn walk(dir: &Path, depth: usize, out: &mut Vec<String>) {
    if depth > MAX_DEPTH || out.len() >= MAX_FILES { return; }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() { if !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()) { walk(&p, depth + 1, out); } }
        else if ft.is_file() && is_source(&p) { out.push(norm_path(&p.to_string_lossy())); }
    }
}

/// Liest `main` und die Quellen aus der `.vcprj` im Ordner (die erste in alphabetischer Reihenfolge, wie die IDE) – ohne Abhängigkeit von `vc-ide`.
fn read_vcprj(root: &Path, p: &mut Project) {
    let Ok(rd) = std::fs::read_dir(root) else { return };
    let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "vcprj")).collect();
    v.sort();
    let Some(file) = v.into_iter().next() else { return };
    let Ok(text) = vc_core::textfile::read_source(&file) else { return };
    let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) else { return };
    let abs = |rel: &str| norm_path(&root.join(rel).to_string_lossy());
    let first_target = j["targets"].as_array().and_then(|t| t.first()).and_then(|t| t["id"].as_str()).map(str::to_string);
    let mut first_src = None;
    for s in j["sources"].as_array().into_iter().flatten() {
        let tgt = s["targets"].as_array().and_then(|t| t.first()).and_then(|t| t.as_str()).map(str::to_string).or_else(|| first_target.clone());
        for f in s["files"].as_array().into_iter().flatten().filter_map(|f| f.as_str()) {
            let a = abs(f);
            if first_src.is_none() { first_src = Some(a.clone()); }
            p.sources.insert(a.clone());
            if let Some(t) = &tgt { p.targets.insert(a, t.clone()); }
        }
    }
    p.main = j["main"].as_str().filter(|m| !m.is_empty()).map(abs).or(first_src);
}

impl Project {
    /// Baut den Graphen: alle Quelldateien unter `root` (Platte) und alle offenen Dokumente (`open`: normalisierter Pfad → Text, hat Vorrang).
    pub fn build(root: Option<&Path>, include_dirs: &[PathBuf], open: &HashMap<String, String>, cache: &mut DiskCache) -> Project {
        let mut p = Project::default();
        let mut files = vec![];
        if let Some(r) = root { read_vcprj(r, &mut p); walk(r, 0, &mut files); }
        let mut set: BTreeSet<String> = files.into_iter().collect();
        set.extend(open.keys().cloned());
        for f in &set {
            let names = match open.get(f) { Some(t) => include_names(t), None => cache.names_of(f) };
            let mut to = vec![];
            for n in names {
                if let Some(r) = resolve_include(f, &n, include_dirs) {
                    let r = norm_path(&r.to_string_lossy());
                    if !to.contains(&r) { p.rev.entry(r.clone()).or_default().insert(f.clone()); to.push(r); }
                }
            }
            p.edges.insert(f.clone(), to);
        }
        p
    }

    /// Alle (auch mittelbar) von `root` eingebundenen Dateien, ohne `root` selbst (außer bei einem Zyklus über `root`)
    pub fn descendants(&self, root: &str) -> HashSet<String> {
        let (mut seen, mut stack) = (HashSet::new(), vec![root.to_string()]);
        while let Some(f) = stack.pop() {
            for c in self.edges.get(&f).into_iter().flatten() { if seen.insert(c.clone()) { stack.push(c.clone()); } }
        }
        seen
    }

    /// Wird `file` von mindestens einer anderen Projektdatei eingebunden?
    pub fn is_included(&self, file: &str) -> bool { self.rev.get(file).map_or(false, |s| s.iter().any(|x| x != file)) }

    /// Die Datei, in deren Kontext `file` zu prüfen ist.
    /// * `file` ist das Startprogramm (`main`) oder wird von nichts eingebunden → `file` selbst;
    /// * sonst `main`, wenn es `file` (auch über mehrere Ebenen) erreicht;
    /// * sonst unter den obersten Einbindern (die ihrerseits niemand einbindet) der mit der kürzesten Include-Kette, bei Gleichstand der erste im Pfad;
    /// * gibt es nur einen Zyklus ohne Einstieg von außen → `file` selbst (der Assembler meldet den Zyklus).
    pub fn root_of(&self, file: &str) -> String {
        if self.main.as_deref() == Some(file) { return file.to_string(); }
        let mut dist: HashMap<&str, usize> = HashMap::new();
        let mut q: VecDeque<&str> = VecDeque::new();
        q.push_back(file);
        while let Some(f) = q.pop_front() {
            let d = dist.get(f).copied().unwrap_or(0);
            for a in self.rev.get(f).into_iter().flatten() {
                if a == file || dist.contains_key(a.as_str()) { continue; }
                dist.insert(a.as_str(), d + 1);
                q.push_back(a.as_str());
            }
        }
        if let Some(m) = self.main.as_deref() { if dist.contains_key(m) { return m.to_string(); } }
        dist.iter().filter(|(a, _)| self.rev.get(**a).map_or(true, |r| r.is_empty()))
            .min_by(|x, y| x.1.cmp(y.1).then(x.0.cmp(y.0))).map_or_else(|| file.to_string(), |(a, _)| a.to_string())
    }
}
