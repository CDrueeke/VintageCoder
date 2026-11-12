// SPDX-License-Identifier: GPL-3.0-or-later
//! Bauprotokoll: der Ablauf eines Builds als Folge von Stufen (mit Dauer, Meldungen und den beteiligten Dateien, auch Zwischendateien).
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub enum Status { Ok, Warn, Error, Skipped }
impl Status { fn s(self) -> &'static str { match self { Status::Ok => "ok", Status::Warn => "warn", Status::Error => "error", Status::Skipped => "skip" } } }

struct Stage { title: String, status: Status, ms: u128, lines: Vec<String>, files: Vec<Value> }

pub struct BuildLog<'a> { root: &'a Path, t0: Instant, stages: Vec<Stage>, cur: Option<(String, Instant, Vec<String>, Vec<Value>)> }

const TEXT_EXT: &[&str] = &["asm", "inc", "lst", "vs", "vcdbg", "txt", "sbas", "bas", "tur", "json", "vcprj", "vcchar", "vcsprite", "vcpmg", "vcscreen", "vctiles", "vcsong", "vcpal", "vcplay", "vctest", "map", "sym", "md"];

impl<'a> BuildLog<'a> {
    pub fn new(root: &'a Path) -> Self { BuildLog { root, t0: Instant::now(), stages: vec![], cur: None } }
    /// Neue Stufe beginnen (eine noch offene wird als „ok“ abgeschlossen)
    pub fn stage(&mut self, title: &str) { self.done(Status::Ok); self.cur = Some((title.to_string(), Instant::now(), vec![], vec![])); }
    pub fn line(&mut self, text: impl Into<String>) { if let Some(c) = &mut self.cur { c.2.push(text.into()); } }
    /// Datei zur laufenden Stufe: `path` absolut oder relativ zum Projekt, `role` z. B. „Zwischendatei“, „Ergebnis“, „Quelle“
    pub fn file(&mut self, path: &Path, role: &str) {
        let abs = if path.is_absolute() || path.starts_with(self.root) { path.to_path_buf() } else { self.root.join(path) };
        // Projektpfade im Protokoll immer mit `/` (auch unter Windows), wie in Projektdateien und Quelltexten
        let rel = abs.strip_prefix(self.root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| abs.to_string_lossy().into_owned());
        let size = std::fs::metadata(&abs).map(|m| m.len()).ok();
        let ext = abs.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let text = TEXT_EXT.contains(&ext.as_str());
        let inside = abs.starts_with(self.root);
        if let Some(c) = &mut self.cur { c.3.push(json!({ "path": rel, "role": role, "size": size, "text": text, "inside": inside, "abs": abs.to_string_lossy() })); }
    }
    pub fn done(&mut self, status: Status) {
        if let Some((title, t, lines, files)) = self.cur.take() { self.stages.push(Stage { title, status, ms: t.elapsed().as_millis(), lines, files }); }
    }
    /// Laufende Stufe mit Fehlermeldungen abschließen
    pub fn fail(&mut self, lines: impl IntoIterator<Item = String>) { for l in lines { self.line(l); } self.done(Status::Error); }
    pub fn finish(mut self) -> Value {
        self.done(Status::Ok);
        let worst = self.stages.iter().fold(Status::Ok, |w, s| match (w, s.status) { (Status::Error, _) | (_, Status::Error) => Status::Error, (Status::Warn, _) | (_, Status::Warn) => Status::Warn, _ => Status::Ok });
        json!({ "ms": self.t0.elapsed().as_millis() as u64, "status": worst.s(),
                "stages": self.stages.iter().map(|s| json!({ "title": s.title, "status": s.status.s(), "ms": s.ms as u64, "lines": s.lines, "files": s.files })).collect::<Vec<_>>() })
    }
}
