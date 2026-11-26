// SPDX-License-Identifier: GPL-3.0-or-later
//! Ereignisbus: der Kern meldet Zeitliches an die Oberfläche (`lang:changed`, `menu:cmd`, später `build:*`, `debug:*`, `fs:changed`).
//! Dieselben Ereignisse gehen bei eingeschalteter Prüfbrücke an deren Verbindungen.
use serde_json::Value;
use tauri::{AppHandle, Emitter};

/// Namen aller Ereignisse (Quelle der Wahrheit; `ui/src/lib/api.ts` führt dieselbe Liste, ein Test vergleicht)
pub const EREIGNISSE: &[&str] = &["app:oeffnen", "lang:changed", "menu:cmd", "project:changed", "lsp:diagnostics", "search:hits", "search:done", "build:started", "build:done", "run:started", "run:failed", "run:stopped", "run:ended", "tests:update"];

pub fn emit(app: &AppHandle, name: &str, payload: Value) {
    debug_assert!(EREIGNISSE.contains(&name), "Ereignis {name} nicht in EREIGNISSE");
    #[cfg(feature = "testbridge")]
    crate::testbridge::broadcast(name, &payload);
    let _ = app.emit(name, payload);
}
