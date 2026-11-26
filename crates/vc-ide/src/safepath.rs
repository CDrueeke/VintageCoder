// SPDX-License-Identifier: GPL-3.0-or-later
//! Pfade aus Projektdateien (`.vcprj`) sind fremde Eingaben: Eine heruntergeladene Projektdatei darf beim Bauen nichts außerhalb des
//! Projektordners lesen oder überschreiben. Jeder Pfad, der aus der Projektdatei stammt, geht vor dem Zugriff durch [`resolve`].
//!
//! Erlaubt sind nur relative Pfade ohne `..`, ohne Laufwerk (`C:`), ohne UNC-Form (`\\server\share`) und ohne führenden Trenner.
//! Beide Trenner (`/` und `\`) gelten auf allen Systemen. Zusätzlich muss der aufgelöste Pfad im Projektordner bleiben,
//! auch über Symlinks (der tiefste vorhandene Elternordner wird mit `canonicalize` aufgelöst).
use std::path::{Path, PathBuf};

/// Syntaktische Prüfung eines Projektpfads; `Err` nennt den Grund (nur für Protokolle, die Meldung für Anwender ist [`resolve`])
pub fn check_rel(rel: &str) -> Result<(), &'static str> {
    if rel.is_empty() { return Err("leer"); }
    if rel.contains('\0') { return Err("NUL"); }
    if rel.starts_with(['/', '\\']) { return Err("absolut"); }
    let b = rel.as_bytes();
    if b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' { return Err("Laufwerk"); }
    if rel.split(['/', '\\']).any(|c| c == "..") { return Err("..") }
    Ok(())
}

/// Ist `rel` ein zulässiger relativer Projektpfad? (nur Syntax)
pub fn is_safe_rel(rel: &str) -> bool { check_rel(rel).is_ok() }

/// Löst `rel` unterhalb von `root` auf. Fehler: Meldung für den Anwender (Katalog `projfile-pfad-unzulaessig`).
pub fn resolve(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let bad = || vc_i18n::t!("projfile-pfad-unzulaessig", pfad = rel).to_string();
    check_rel(rel).map_err(|_| bad())?;
    let joined = root.join(rel);
    // Projektordner gibt es (noch) nicht: dann kann auch kein Symlink hinausführen; die Syntaxprüfung genügt
    let Ok(canon_root) = std::fs::canonicalize(root) else { return Ok(joined) };
    // tiefster vorhandener Vorfahre (bei vorhandener Datei sie selbst): folgt Symlinks, auch den letzten
    let mut probe: &Path = &joined;
    let real = loop {
        match std::fs::canonicalize(probe) {
            Ok(p) => break p,
            Err(_) => match probe.parent() { Some(p) if p != probe => probe = p, _ => return Err(bad()) },
        }
    };
    // ein hängender Symlink existiert nicht für canonicalize, zeigt aber nach draußen: Linkziel prüfen
    if let Ok(md) = std::fs::symlink_metadata(&joined) {
        if md.file_type().is_symlink() && std::fs::canonicalize(&joined).is_err() { return Err(bad()); }
    }
    if real.starts_with(&canon_root) { Ok(joined) } else { Err(bad()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc-safepath-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn syntax_lehnt_gefaehrliche_formen_ab() {
        for p in ["/Users/x/.zshrc", "../x", "a/../../x", "a/..", "..\\x", "a\\..\\..\\x", "C:\\Windows\\x", "c:/x", "\\\\server\\share\\f", "//server/share", "\\x", "", "a\0b"] {
            assert!(!is_safe_rel(p), "{p:?} muss abgelehnt werden");
        }
        for p in ["build/a.d64", "a.prg", "src/lib/x.asm", "a b/c.d", "..x/y", "x..y", "dir\\file.prg"] {
            assert!(is_safe_rel(p), "{p:?} muss erlaubt sein");
        }
    }

    #[test]
    fn unterordner_bleibt_erlaubt_auch_wenn_neu() {
        let d = tmp("ok");
        assert!(resolve(&d, "build/neu/x.d64").is_ok());
        std::fs::create_dir_all(d.join("build")).unwrap();
        assert_eq!(resolve(&d, "build/x.d64").unwrap(), d.join("build/x.d64"));
    }

    #[test]
    fn absolut_und_hoch_werden_abgelehnt() {
        let d = tmp("abs");
        assert!(resolve(&d, "/etc/passwd").is_err());
        assert!(resolve(&d, "../draussen.d64").is_err());
        let m = resolve(&d, "../x").unwrap_err();
        assert!(m.contains("../x"), "Meldung nennt den Pfad: {m}");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_aus_dem_projekt_heraus_wird_abgelehnt() {
        let d = tmp("sym");
        let out = tmp("sym-aussen");
        std::os::unix::fs::symlink(&out, d.join("link")).unwrap();
        assert!(resolve(&d, "link/x.d64").is_err(), "Ordner-Symlink nach draußen");
        std::fs::write(out.join("f"), b"x").unwrap();
        std::os::unix::fs::symlink(out.join("f"), d.join("datei")).unwrap();
        assert!(resolve(&d, "datei").is_err(), "Datei-Symlink nach draußen");
        std::os::unix::fs::symlink(out.join("gibt-es-nicht"), d.join("haengt")).unwrap();
        assert!(resolve(&d, "haengt").is_err(), "hängender Symlink nach draußen");
        std::fs::create_dir_all(d.join("innen")).unwrap();
        std::os::unix::fs::symlink(d.join("innen"), d.join("kurz")).unwrap();
        assert!(resolve(&d, "kurz/x.d64").is_ok(), "Symlink innerhalb des Projekts bleibt erlaubt");
    }
}
