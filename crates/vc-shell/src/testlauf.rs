// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests (E1.3): Läufe aus `*.vctest` im Hintergrund über `vc_ide::tests::suite` (statische Prüfungen und Emulator-Tests), Ergebnisse als Ereignis `tests:update`
//! (ganzer Stand, klein). Immer nur ein Lauf; Abbrechen wartet nicht (der Läufer beendet den Emulator selbst).
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vc_ide::tests::model::{Level, State};
use vc_ide::tests::suite::{self, Event, Only, RunRequest, Suite};

#[derive(Default)]
pub struct Tests { pub suite: Arc<Mutex<Suite>>, cancel: Mutex<Option<Arc<AtomicBool>>> }

fn zustand(s: State) -> &'static str { match s { State::Passed => "passed", State::Failed => "failed", State::TimedOut => "timeout", State::Skipped => "skipped" } }

/// Stand für die Oberfläche; Dateipfade relativ zum Projekt
pub fn json(root: &Path, s: &Suite) -> Value {
    let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/");
    let c = s.counts();
    json!({ "running": s.running, "error": s.error, "millis": s.last_millis, "counts": { "passed": c.passed, "failed": c.failed, "timedOut": c.timed_out, "skipped": c.skipped, "hints": c.hints },
        "groups": s.groups.iter().map(|g| json!({ "title": g.title, "file": g.file.as_deref().map(&rel), "backend": g.backend, "mailbox": g.mailbox, "millis": g.millis, "expected": g.expected,
            "items": g.items.iter().map(|i| { let o = &i.outcome; json!({ "name": o.name, "state": zustand(o.state), "level": if o.level == Level::Hint { "hint" } else { "error" }, "detail": o.detail, "measured": o.measured,
                "location": o.location.as_ref().map(|(f, l)| json!([f, l])), "regs": o.regs.map(|r| json!({ "pc": r.pc, "a": r.a, "x": r.x, "y": r.y, "sp": r.sp, "flags": r.flags })), "file": rel(&i.file) }) }).collect::<Vec<_>>() })).collect::<Vec<_>>() })
}

impl Tests {
    pub fn laeuft(&self) -> bool { self.suite.lock().map_or(false, |s| s.running) }
    pub fn abbrechen(&self) { if let Some(c) = self.cancel.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref() { c.store(true, Ordering::SeqCst); } }
    /// Lauf starten; `senden` bekommt nach jedem Ereignis den Stand. `Err`, wenn schon einer läuft.
    pub fn starten(&self, root: PathBuf, data: PathBuf, backend: Option<String>, only: Only, senden: impl Fn(Value) + Send + 'static) -> Result<(), ()> {
        if self.laeuft() { return Err(()); }
        let cancel = Arc::new(AtomicBool::new(false)); *self.cancel.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cancel.clone());
        { let mut s = self.suite.lock().unwrap_or_else(std::sync::PoisonError::into_inner); s.begin(&only); }
        let req = RunRequest { root: root.clone(), data, backend, strict: false, only, hardware: None };
        let (tx, rx) = std::sync::mpsc::channel::<Event>();
        let suite_ref = self.suite.clone();
        std::thread::spawn(move || suite::run(&req, cancel, &tx));
        std::thread::spawn(move || {
            while let Ok(e) = rx.recv() {
                let ende = matches!(e, Event::Done { .. } | Event::Error(_));
                { let mut s = suite_ref.lock().unwrap_or_else(std::sync::PoisonError::into_inner); s.apply(e); senden(json(&root, &s)); }
                if ende { break; }
            }
            // Läufer ohne `Done` (Absturz): nicht hängen bleiben
            let mut s = suite_ref.lock().unwrap_or_else(std::sync::PoisonError::into_inner); if s.running { s.running = false; s.error = Some(vc_i18n::t!("testlauf-der-testlauf-wurde-unerwartet-beendet").to_string()); senden(json(&root, &s)); }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn statische_pruefungen_laufen_und_melden_den_stand() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data");
        let tmp = std::env::temp_dir().join(format!("vc_testlauf_{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("main.asm"), ".target c64\n.org $c000\nstart: lda #1\n  sta $d020\n  rts\n").unwrap();
        let b = vc_ide::build::build(&vc_ide::build::Ctx { root: &tmp, data: &data }, "main.asm", None).unwrap(); assert_eq!(b["ok"], true);
        std::fs::write(tmp.join("main.vctest"), r#"{ "kind": "test", "target": "c64", "program": "build/main.prg", "static": [ { "check": "overlap" }, { "check": "size", "max": 100 }, { "check": "size", "max": 1 } ] }"#).unwrap();
        let t = Tests::default(); let (tx, rx) = std::sync::mpsc::channel();
        t.starten(tmp.clone(), data, None, Only::All, move |v| { let _ = tx.send(v); }).unwrap();
        let mut last = None; let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 30 { match rx.recv_timeout(std::time::Duration::from_millis(200)) { Ok(v) => { let fertig = v["running"] == false; last = Some(v); if fertig { break; } } Err(_) => {} } }
        let v = last.expect("Stand"); assert_eq!(v["running"], false, "{v}");
        assert!(v["counts"]["passed"].as_u64().unwrap() >= 2 && v["counts"]["failed"].as_u64().unwrap() == 1, "{}", v["counts"]);
        assert_eq!(v["groups"][0]["items"][0]["file"], "main.vctest");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
