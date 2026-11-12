// SPDX-License-Identifier: GPL-3.0-or-later
//! Projektsuche (Seitenleiste „Suche“): Textsuche in den Textdateien des Projekts, im Hintergrund und abbrechbar.
//!
//! Der Kern (`find_in_line`, `run`) ist synchron und frei von Oberfläche, damit er sich testen lässt; `SearchState` startet ihn in einem
//! Arbeits-Thread und sammelt die Treffer ab. Je Zeile zählt ein Treffer (der erste in der Zeile). Ungespeicherte Änderungen offener
//! Dateien werden mit durchsucht (`Params::overrides`).
use crate::project::Group;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

/// So viele Trefferzeilen höchstens; danach bricht die Suche ab und meldet „mehr als …“
pub const MAX_HITS: usize = 500;
/// Größere Dateien werden übersprungen (Speicherabzüge, Exporte)
const MAX_FILE: u64 = 2 * 1024 * 1024;
/// Endungen sonstiger Textdateien, die mit durchsucht werden (Quelltext und BASIC kommen über die Art)
const TEXT_EXT: [&str; 12] = ["txt", "md", "vctest", "vcprj", "json", "cfg", "ini", "toml", "yaml", "yml", "csv", "xml"];

/// Eine Trefferzeile
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub rel: String,
    /// 1-basiert
    pub line: usize,
    /// Die Zeile ohne führende Leerzeichen, bei langen Zeilen um den Treffer herum gekürzt
    pub text: String,
    /// Bytebereich des Treffers in `text`
    pub start: usize,
    pub end: usize,
}

/// Was durchsucht wird
#[derive(Clone, Debug)]
pub struct Params {
    pub root: PathBuf,
    /// Projektrelative Dateien in der Reihenfolge der Ausgabe
    pub files: Vec<String>,
    /// Stand ungespeicherter Dokumente (relativer Pfad → Text); er geht der Datei auf der Platte vor
    pub overrides: HashMap<String, String>,
    pub text: String,
    pub case: bool,
    pub limit: usize,
}

/// Wie die Suche endete
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary { pub files: usize, pub truncated: bool, pub cancelled: bool }

/// Welche Einträge des Projektbaums durchsucht werden: Quelltext, BASIC und sonstige Textdateien (Mediendateien nicht)
pub fn candidate_files(entries: &[(String, Group)]) -> Vec<String> {
    entries.iter().filter(|(rel, g)| match g {
        Group::Asm | Group::Basic | Group::Text => true,
        Group::Other => Path::new(rel).extension().and_then(|e| e.to_str()).map_or(false, |e| TEXT_EXT.contains(&e.to_ascii_lowercase().as_str())),
        _ => false,
    }).map(|(r, _)| r.clone()).collect()
}

/// Alle (nicht überlappenden) Fundstellen von `needle` in `line` als Bytebereiche. Ohne `case` zählt Groß-/Kleinschreibung nicht.
pub fn find_in_line(line: &str, needle: &str, case: bool) -> Vec<(usize, usize)> {
    if needle.is_empty() { return vec![]; }
    let fold = |c: char| if case { c } else { c.to_lowercase().next().unwrap_or(c) };
    let n: Vec<char> = needle.chars().map(fold).collect();
    let hay: Vec<(usize, char)> = line.char_indices().collect();
    let mut out = vec![];
    let mut i = 0;
    while i + n.len() <= hay.len() {
        if (0..n.len()).all(|j| fold(hay[i + j].1) == n[j]) {
            let end = hay.get(i + n.len()).map_or(line.len(), |x| x.0);
            out.push((hay[i].0, end));
            i += n.len();
        } else { i += 1; }
    }
    out
}

/// Zeile für die Anzeige: führende Leerzeichen weg, lange Zeilen um den Treffer gekürzt. Liefert Text und den Treffer darin.
pub fn snippet(line: &str, start: usize, end: usize) -> (String, usize, usize) {
    const KEEP: usize = 120;
    let trimmed = line.trim_start();
    let cut = line.len() - trimmed.len();
    let (mut s, mut e) = (start.saturating_sub(cut), end.saturating_sub(cut));
    let mut text = trimmed.trim_end().to_string();
    if e > text.len() { e = text.len(); s = s.min(e); }
    let chars_before = text[..s].chars().count();
    if text.chars().count() > KEEP {
        // Fenster um den Treffer: höchstens 30 Zeichen davor
        let skip = if chars_before > KEEP - 40 { chars_before - 30 } else { 0 };
        let from = text.char_indices().nth(skip).map_or(0, |x| x.0);
        let mut tail: String = text[from..].chars().take(KEEP).collect();
        let lost = text[from..].chars().count() > KEEP;
        let prefix = if skip > 0 { "…" } else { "" };
        if lost { tail.push('…'); }
        let (ns, ne) = (s.saturating_sub(from) + prefix.len(), e.saturating_sub(from) + prefix.len());
        text = format!("{prefix}{tail}");
        s = ns.min(text.len()); e = ne.min(text.len());
    }
    (text, s, e)
}

/// Durchsucht die Dateien der Reihe nach und meldet jede Trefferzeile an `emit`. Prüft `cancel` je Datei und alle 2000 Zeilen.
pub fn run(p: &Params, cancel: &AtomicBool, emit: &mut dyn FnMut(Hit)) -> Summary {
    let mut sum = Summary::default();
    let mut count = 0usize;
    if p.text.is_empty() { return sum; }
    for rel in &p.files {
        if cancel.load(Ordering::Relaxed) { sum.cancelled = true; return sum; }
        let text = match p.overrides.get(rel) {
            Some(t) => t.clone(),
            None => {
                let path = p.root.join(rel);
                if std::fs::metadata(&path).map_or(true, |m| m.len() > MAX_FILE) { continue; }
                match std::fs::read(&path).ok().and_then(|b| if b.contains(&0) { None } else { String::from_utf8(b).ok() }) { Some(t) => t, None => continue }
            }
        };
        sum.files += 1;
        for (i, line) in text.lines().enumerate() {
            if i % 2000 == 1999 && cancel.load(Ordering::Relaxed) { sum.cancelled = true; return sum; }
            let Some(&(s, e)) = find_in_line(line, &p.text, p.case).first() else { continue };
            if count >= p.limit { sum.truncated = true; return sum; }
            count += 1;
            let (t, ts, te) = snippet(line, s, e);
            emit(Hit { rel: rel.clone(), line: i + 1, text: t, start: ts, end: te });
        }
    }
    sum
}

enum Msg { Hit(Hit), Done(Summary) }

/// Zustand der Seitenleiste „Suche“: Eingabe, laufende Suche, bisherige Treffer
#[derive(Default)]
pub struct SearchState {
    pub text: String,
    pub case: bool,
    /// Eingabefeld beim nächsten Bild fokussieren
    pub focus: bool,
    pub hits: Vec<Hit>,
    pub summary: Summary,
    pub running: bool,
    /// Für diese Eingabe (Text, Groß/Klein) laufen oder liegen Treffer vor
    pub for_query: Option<(String, bool)>,
    /// Eingeklappte Dateien
    pub collapsed: std::collections::HashSet<String>,
    rx: Option<Receiver<Msg>>,
    cancel: Option<Arc<AtomicBool>>,
}

impl SearchState {
    /// Laufende Suche abbrechen (Treffer bleiben stehen)
    pub fn cancel(&mut self) {
        if let Some(c) = &self.cancel { c.store(true, Ordering::Relaxed); }
        self.rx = None; self.cancel = None;
        if self.running { self.running = false; self.summary.cancelled = true; }
    }

    /// Neue Suche starten; eine laufende wird abgebrochen. Leerer Text löscht die Treffer.
    pub fn start(&mut self, p: Params, ctx: &eframe::egui::Context) {
        self.cancel();
        self.hits.clear(); self.summary = Summary::default();
        self.for_query = Some((p.text.clone(), p.case));
        if p.text.is_empty() { return; }
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel();
        let (c2, ctx2) = (cancel.clone(), ctx.clone());
        std::thread::spawn(move || {
            let tx2 = tx.clone();
            let sum = run(&p, &c2, &mut |h| { let _ = tx2.send(Msg::Hit(h)); });
            let _ = tx.send(Msg::Done(sum));
            ctx2.request_repaint();
        });
        self.rx = Some(rx); self.cancel = Some(cancel); self.running = true;
    }

    /// Treffer des Threads übernehmen; liefert, ob sich etwas geändert hat
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.rx else { return false };
        let mut changed = false;
        let mut done = false;
        for m in rx.try_iter().take(2000) {
            changed = true;
            match m { Msg::Hit(h) => self.hits.push(h), Msg::Done(s) => { self.summary = s; self.running = false; done = true; } }
        }
        if done { self.rx = None; self.cancel = None; }
        changed
    }

    /// Wurde Text oder Groß-/Kleinschreibung seit dem letzten Start geändert?
    pub fn stale(&self) -> bool { self.for_query.as_ref().map_or(!self.text.is_empty(), |(t, c)| *t != self.text || *c != self.case) }
}

impl Drop for SearchState { fn drop(&mut self) { if let Some(c) = &self.cancel { c.store(true, Ordering::Relaxed); } } }

/// Treffer nach Datei gruppiert (Reihenfolge des Auftretens): (Datei, Treffer dieser Datei)
pub fn grouped(hits: &[Hit]) -> Vec<(&str, Vec<&Hit>)> {
    let mut out: Vec<(&str, Vec<&Hit>)> = vec![];
    for h in hits {
        match out.last_mut() { Some((r, v)) if *r == h.rel => v.push(h), _ => out.push((&h.rel, vec![h])) }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(dir: &Path, files: &[&str], text: &str, case: bool, limit: usize) -> Params {
        Params { root: dir.to_path_buf(), files: files.iter().map(|s| s.to_string()).collect(), overrides: HashMap::new(), text: text.into(), case, limit }
    }
    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc_psearch_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
    }
    fn collect(p: &Params, cancel: &AtomicBool) -> (Vec<Hit>, Summary) {
        let mut v = vec![]; let s = run(p, cancel, &mut |h| v.push(h)); (v, s)
    }

    #[test]
    fn treffer_in_der_zeile_mit_bytebereich() {
        assert_eq!(find_in_line("lda #$00 ; LDA", "lda", false), vec![(0, 3), (11, 14)]);
        assert_eq!(find_in_line("lda #$00 ; LDA", "lda", true), vec![(0, 3)]);
        assert_eq!(find_in_line("aaaa", "aa", true), vec![(0, 2), (2, 4)], "ohne Überlappung");
        assert!(find_in_line("abc", "", true).is_empty());
        // Umlaute: Bytebereiche liegen auf Zeichengrenzen
        let l = "größe = ÄÖ";
        let r = find_in_line(l, "äö", false);
        assert_eq!(r.len(), 1);
        assert_eq!(&l[r[0].0..r[0].1], "ÄÖ");
    }

    #[test]
    fn suche_findet_zeilen_in_mehreren_dateien_gruppiert() {
        let d = tmp("a");
        std::fs::write(d.join("a.asm"), "start:\n  jsr Init\n  rts\n").unwrap();
        std::fs::write(d.join("b.asm"), "init:\n  rts\nnoch init\n").unwrap();
        let (hits, s) = collect(&params(&d, &["a.asm", "b.asm"], "init", false, 100), &AtomicBool::new(false));
        assert_eq!(hits.iter().map(|h| (h.rel.as_str(), h.line)).collect::<Vec<_>>(), vec![("a.asm", 2), ("b.asm", 1), ("b.asm", 3)]);
        assert_eq!(hits[0].text, "jsr Init");
        assert_eq!(&hits[0].text[hits[0].start..hits[0].end], "Init");
        assert_eq!((s.files, s.truncated, s.cancelled), (2, false, false));
        let g = grouped(&hits);
        assert_eq!(g.iter().map(|(r, v)| (*r, v.len())).collect::<Vec<_>>(), vec![("a.asm", 1), ("b.asm", 2)]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn gross_und_kleinschreibung_ist_wahlfrei() {
        let d = tmp("b");
        std::fs::write(d.join("a.asm"), "Init\ninit\nINIT\n").unwrap();
        let alle = collect(&params(&d, &["a.asm"], "init", false, 100), &AtomicBool::new(false)).0;
        let genau = collect(&params(&d, &["a.asm"], "init", true, 100), &AtomicBool::new(false)).0;
        assert_eq!(alle.len(), 3);
        assert_eq!(genau.iter().map(|h| h.line).collect::<Vec<_>>(), vec![2]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn trefferzahl_ist_begrenzt() {
        let d = tmp("c");
        std::fs::write(d.join("a.asm"), "x\n".repeat(50)).unwrap();
        let (hits, s) = collect(&params(&d, &["a.asm"], "x", false, 10), &AtomicBool::new(false));
        assert_eq!(hits.len(), 10);
        assert!(s.truncated);
        // genau das Limit: nicht abgeschnitten
        let (hits, s) = collect(&params(&d, &["a.asm"], "x", false, 50), &AtomicBool::new(false));
        assert_eq!((hits.len(), s.truncated), (50, false));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn abbruch_beendet_die_suche_sofort() {
        let d = tmp("d");
        std::fs::write(d.join("a.asm"), "x\n").unwrap();
        let cancel = AtomicBool::new(true);
        let (hits, s) = collect(&params(&d, &["a.asm"], "x", false, 10), &cancel);
        assert!(hits.is_empty() && s.cancelled);
        // mitten in der Suche: nach dem ersten Treffer gesetzt
        let d2 = tmp("d2");
        for n in ["a.asm", "b.asm", "c.asm"] { std::fs::write(d2.join(n), "x\n").unwrap(); }
        let c = AtomicBool::new(false);
        let mut n = 0;
        let p = params(&d2, &["a.asm", "b.asm", "c.asm"], "x", false, 10);
        let s = run(&p, &c, &mut |_| { n += 1; c.store(true, Ordering::Relaxed); });
        assert_eq!(n, 1);
        assert!(s.cancelled);
        let _ = (std::fs::remove_dir_all(&d), std::fs::remove_dir_all(&d2));
    }

    #[test]
    fn hintergrundsuche_liefert_treffer_und_laesst_sich_abbrechen() {
        let d = tmp("e");
        std::fs::write(d.join("a.asm"), "foo\nbar\nfoo\n").unwrap();
        let ctx = eframe::egui::Context::default();
        let mut s = SearchState::default();
        s.text = "foo".into();
        s.start(params(&d, &["a.asm"], "foo", false, 10), &ctx);
        assert!(s.running && !s.stale());
        for _ in 0..200 { s.poll(); if !s.running { break; } std::thread::sleep(std::time::Duration::from_millis(10)); }
        assert!(!s.running);
        assert_eq!(s.hits.len(), 2);
        s.text = "bar".into();
        assert!(s.stale(), "geänderte Eingabe verlangt eine neue Suche");
        s.start(params(&d, &["a.asm"], "bar", false, 10), &ctx);
        s.cancel();
        assert!(!s.running);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn ungespeicherte_aenderungen_und_binaerdateien() {
        let d = tmp("f");
        std::fs::write(d.join("a.asm"), "alt\n").unwrap();
        std::fs::write(d.join("b.bin"), [0u8, 1, 2, b'x']).unwrap();
        let mut p = params(&d, &["a.asm", "b.bin", "fehlt.asm"], "neu", false, 10);
        p.overrides.insert("a.asm".into(), "alt\nneu hier\n".into());
        let (hits, s) = collect(&p, &AtomicBool::new(false));
        assert_eq!(hits.iter().map(|h| (h.rel.as_str(), h.line)).collect::<Vec<_>>(), vec![("a.asm", 2)]);
        assert_eq!(s.files, 1, "Binärdatei und fehlende Datei zählen nicht");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn auswahl_der_dateien_und_kuerzen_langer_zeilen() {
        let e = vec![("a.asm".to_string(), Group::Asm), ("b.bas".into(), Group::Basic), ("c.spr.json".into(), Group::Sprite), ("d.vctest".into(), Group::Other), ("e.prg".into(), Group::Other), ("README.md".into(), Group::Other)];
        assert_eq!(candidate_files(&e), vec!["a.asm", "b.bas", "d.vctest", "README.md"]);
        let (t, s, en) = snippet("    lda #1", 4, 7);
        assert_eq!((t.as_str(), &t[s..en]), ("lda #1", "lda"));
        let long = format!("{}ZIEL{}", "a".repeat(300), "b".repeat(300));
        let st = long.find("ZIEL").unwrap();
        let (t, s, en) = snippet(&long, st, st + 4);
        assert!(t.chars().count() < 200, "{}", t.len());
        assert_eq!(&t[s..en], "ZIEL");
    }
}
