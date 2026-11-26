// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien im nativen BASIC (`#include "graphics/hero.vcsprite"`, Konzept „Include von Medien“): die Haken für `vc_basic::include`.
//! Der Provider wandelt ein Medium des Projekts (Sprite, Zeichensatz, Song …) in DATA-Zeilen (Format `Data`) um; gewöhnliche Dateien liest der Kern selbst.
//! Der Hinweis auf Altpfade (`build/export/<name>.data.bas`) nennt das Medium, das stattdessen einzubinden ist.
use std::path::Path;
use std::sync::Arc;
use vc_basic::include::IncludeHooks;
use vc_media::include::{is_media_path, legacy_media, resolve_include, IncFormat};

/// Haken für Bau und Umwandlung; `target`: Zielsystem (Profile anderer Systeme sind ein Fehler)
pub fn hooks(root: &Path, data: &Path, target: &str) -> IncludeHooks {
    let (r, d, t) = (root.to_path_buf(), data.to_path_buf(), target.to_string());
    let provider = Arc::new(move |p: &Path| -> Option<Result<String, String>> {
        if !is_media_path(&p.to_string_lossy()) { return None; }
        Some(resolve_include(&r, &d, p, IncFormat::Data, Some(&t)).map(|x| x.text).map_err(|e| e.to_string()))
    });
    let r2 = root.to_path_buf();
    let legacy = Arc::new(move |p: &Path| legacy_media(&r2, p));
    IncludeHooks { provider: Some(provider), legacy_hint: Some(legacy) }
}
