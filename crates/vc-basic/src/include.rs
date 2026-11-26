// SPDX-License-Identifier: GPL-3.0-or-later
//! Einbinden von Dateien und **Medien** im nativen BASIC (Konzept „Include von Medien“, Abschnitt 3).
//!
//! `#include "pfad"` liest die Datei relativ zur einbindenden Datei. Mit einem [`IncludeProvider`] (derselbe Haken wie beim Assembler,
//! `vc_asm::Options::include_provider`) wird zuerst der Provider gefragt: `Some(Ok(text))` heißt „Medium, hier ist der erzeugte Text“
//! (DATA-Zeilen mit `@name:`-Marken), `Some(Err(grund))` ist ein Fehler (`B193`), `None` bedeutet gewöhnliche Datei. `vc-basic` hängt
//! dadurch nicht von `vc-media` ab; Bau (`vc-ide`/`vc-cli`) setzt den Provider mit `vc_media::include::resolve_include`.
//!
//! * Das Medienstück trägt **keine Zeilennummern**; sie entstehen erst beim Nummerieren (`#numbering`/Projektwerte gelten also auch dort).
//! * Zeilenzuordnung ([`LineMap`]): Zeilen eines Mediums zeigen auf den eingebundenen Pfad mit den Zeilen 1…n des erzeugten Textes.
//! * Ein Medium wird je Programm **einmal** eingebunden; ein zweites `#include` desselben Mediums wird still übersprungen.
//! * Altpfade (`build/export/…`): der `legacy_hint` nennt das Medium dazu; `B194` (Warnung) mit der Quelle.
//! * Fehler: `B120`/`B121` wie bisher, `B193` Medium nicht gefunden/nicht als DATA exportierbar, `B194` Altpfad.
use crate::plain::{include_directive, LineMap};
use std::path::Path;
use std::sync::Arc;
use vc_core::{msg, Diagnostic};

/// Auflöser für Medien: `Some(Ok(text))` Medium, `Some(Err(grund))` Fehler, `None` gewöhnliche Datei
pub type IncludeProvider = Arc<dyn Fn(&Path) -> Option<Result<String, String>> + Send + Sync>;
/// Altpfad-Erkennung: nennt zu einem alten Pfad (`build/export/hero.data.bas`) das Medium, das ihn erzeugt (`graphics/hero.vcsprite`)
pub type LegacyHint = Arc<dyn Fn(&Path) -> Option<String> + Send + Sync>;

/// Haken für das Einbinden (beide optional; ohne beide verhält sich alles wie bisher)
#[derive(Clone, Default)]
pub struct IncludeHooks { pub provider: Option<IncludeProvider>, pub legacy_hint: Option<LegacyHint> }

impl std::fmt::Debug for IncludeHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "IncludeHooks(provider: {}, legacy_hint: {})", self.provider.is_some(), self.legacy_hint.is_some()) }
}

/// Ergebnis des Einbindens: Gesamttext, Herkunft je Zeile, Warnungen (Altpfade), eingebundene Medien (Pfad wie in der Zeilenzuordnung)
#[derive(Debug, Clone)]
pub struct Expanded { pub text: String, pub map: LineMap, pub warnings: Vec<Diagnostic>, pub media: Vec<String> }

/// Eine gelesene Einbindung: Text, ob Medium
struct Read { text: String, media: bool }

fn read_one(h: &IncludeHooks, path: &Path, name: &str, file: &str, line: usize, warnings: &mut Vec<Diagnostic>) -> Result<Read, Diagnostic> {
    if let Some(p) = &h.provider {
        match p(path) {
            Some(Ok(text)) => return Ok(Read { text, media: true }),
            Some(Err(why)) => return Err(Diagnostic::error("B193", file, line, msg!("diag-basic-b193-media-failed", name = name, reason = why.as_str()))),
            None => {}
        }
    }
    let text = vc_core::textfile::read_source(path).map_err(|_| Diagnostic::error("B120", file, line, msg!("diag-basic-b120-include-not-found", name = name)))?;
    if let Some(src) = h.legacy_hint.as_ref().and_then(|f| f(path)) {
        warnings.push(Diagnostic::warning("B194", file, line, msg!("diag-basic-b194-legacy-path", name = name, source = src.as_str())));
    }
    Ok(Read { text, media: false })
}

fn child(file: &str, name: &str) -> (std::path::PathBuf, String) {
    let path = Path::new(file).parent().map(|p| p.to_path_buf()).unwrap_or_default().join(name);
    let key = path.to_string_lossy().into_owned();
    (path, key)
}

/// Ersetzt `#include "datei"`-Zeilen (Dateien und Medien) durch den Inhalt, auch verschachtelt
pub fn expand(src: &str, file: &str, h: &IncludeHooks) -> Result<Expanded, Vec<Diagnostic>> {
    struct St { out: Vec<String>, map: LineMap, warnings: Vec<Diagnostic>, media: Vec<String> }
    fn go(src: &str, file: &str, h: &IncludeHooks, stack: &mut Vec<String>, st: &mut St) -> Result<(), Diagnostic> {
        for (n, raw) in src.lines().enumerate() {
            let Some(name) = include_directive(raw) else { st.out.push(raw.to_string()); st.map.push((file.to_string(), n + 1)); continue };
            let (path, key) = child(file, name);
            if stack.len() >= 8 || stack.contains(&key) { return Err(Diagnostic::error("B121", file, n + 1, msg!("diag-basic-b121-include-nesting", name = name))); }
            if st.media.contains(&key) { continue; }            // Medium schon eingebunden: still überspringen
            let r = read_one(h, &path, name, file, n + 1, &mut st.warnings)?;
            if r.media { st.media.push(key.clone()); }
            stack.push(key.clone());
            go(&r.text, &key, h, stack, st)?;
            stack.pop();
        }
        Ok(())
    }
    let mut st = St { out: vec![], map: vec![], warnings: vec![], media: vec![] };
    go(src, file, h, &mut vec![file.to_string()], &mut st).map_err(|d| vec![d])?;
    let mut text = st.out.join("\n"); text.push('\n');
    Ok(Expanded { text, map: st.map, warnings: st.warnings, media: st.media })
}

/// Datei samt Einbindungen für die Umwandlung (`labels`): Medien stehen als eigene Einheit mit dem Medienpfad
pub fn load_set(path: &str, text: &str, h: &IncludeHooks) -> Result<Vec<crate::labels::Unit>, Vec<Diagnostic>> { load_set_media(path, text, h).map(|(u, _)| u) }

/// Wie [`load_set`], liefert zusätzlich die Pfade der Einheiten, die **Medien** sind. Diese Einheiten sind schreibgeschützt: wer eine Umwandlung
/// (`labels::to_numbers` …) zurückschreibt, darf sie nicht schreiben (der erzeugte Text gehört nicht in eine Datei).
pub fn load_set_media(path: &str, text: &str, h: &IncludeHooks) -> Result<(Vec<crate::labels::Unit>, Vec<String>), Vec<Diagnostic>> {
    use crate::labels::Unit;
    struct St { out: Vec<Unit>, warns: Vec<Diagnostic>, media: Vec<String> }
    fn go(file: &str, text: &str, h: &IncludeHooks, stack: &mut Vec<String>, st: &mut St) -> Result<(), Diagnostic> {
        if !st.out.iter().any(|u| u.file == file) { st.out.push(Unit { file: file.to_string(), text: text.to_string() }); }
        for (n, raw) in text.lines().enumerate() {
            let Some(name) = include_directive(raw) else { continue };
            let (path, key) = child(file, name);
            if stack.len() >= 8 || stack.contains(&key) { return Err(Diagnostic::error("B121", file, n + 1, msg!("diag-basic-b121-include-nesting", name = name))); }
            let r = read_one(h, &path, name, file, n + 1, &mut st.warns)?;
            if r.media && !st.media.contains(&key) { st.media.push(key.clone()); }
            stack.push(key.clone());
            go(&key, &r.text, h, stack, st)?;
            stack.pop();
        }
        Ok(())
    }
    let mut st = St { out: vec![], warns: vec![], media: vec![] };
    go(path, text, h, &mut vec![path.to_string()], &mut st).map_err(|d| vec![d])?;
    Ok((st.out, st.media))
}
