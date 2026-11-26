// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateien im Projektbaum anlegen, umbenennen und löschen (ohne GUI). Umbenennen und Löschen ziehen die Verweise der `.vcprj`
//! (Quellen, Startprogramm, Diskette, Bestandsliste) mit; gelöscht wird in den Papierkorb, wo das System einen kennt.
use crate::project::Project;
use std::path::Path;

/// Projektrelativen Pfad prüfen und vereinheitlichen (`/` als Trenner); verboten sind leere, absolute und aus dem Projekt führende Pfade
pub fn safe_rel(name: &str) -> Result<String, String> {
    let n = name.trim().replace('\\', "/");
    let n = n.trim_matches('/').to_string();
    if n.is_empty() { return Err(vc_i18n::t!("fileops-der-name-ist-leer").into()); }
    if name.trim().starts_with('/') || name.trim().starts_with('\\') || n.contains(':') { return Err(vc_i18n::t!("fileops-der-pfad-muss-innerhalb-des-projekts").into()); }
    if n.split('/').any(|p| p.is_empty() || p == "." || p == ".." ) { return Err(vc_i18n::t!("fileops-der-pfad-darf-kein-und-keine").into()); }
    if n.split('/').any(|p| p.starts_with('.')) { return Err(vc_i18n::t!("fileops-namen-mit-fuehrendem-punkt-sind-im").into()); }
    Ok(n)
}

/// Absoluter Pfad eines Projektpfads: wie [`safe_rel`], und zusätzlich bleibt der aufgelöste Pfad im Projektordner (kein Symlink nach draußen)
pub fn abs_im_projekt(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    let rel = safe_rel(rel)?;
    vc_ide::safepath::resolve(root, &rel)
}

/// Neue, leere Datei (Ordner werden bei Bedarf angelegt); eine vorhandene Datei bleibt unberührt
pub fn create_file(root: &Path, rel: &str) -> Result<String, String> {
    let rel = safe_rel(rel)?; let abs = abs_im_projekt(root, &rel)?;
    if abs.exists() { return Err(vc_i18n::t!("fileops-rel-gibt-es-schon", rel = &rel)); }
    if let Some(d) = abs.parent() { std::fs::create_dir_all(d).map_err(|e| vc_i18n::t!("fileops-rel-e", rel = &rel, e = &e))?; }
    std::fs::write(&abs, "").map_err(|e| vc_i18n::t!("fileops-rel-e", rel = &rel, e = &e))?;
    Ok(rel)
}

/// Neuer Ordner (auch verschachtelt); ein vorhandener Ordner oder eine gleichnamige Datei ist ein Fehler
pub fn create_dir(root: &Path, rel: &str) -> Result<String, String> {
    let rel = safe_rel(rel)?; let abs = abs_im_projekt(root, &rel)?;
    if abs.exists() { return Err(vc_i18n::t!("fileops-rel-gibt-es-schon", rel = &rel)); }
    std::fs::create_dir_all(&abs).map_err(|e| vc_i18n::t!("fileops-rel-e", rel = &rel, e = &e))?; Ok(rel)
}

/// Verweise der Projektdatei auf `old` ändern (`new`) oder entfernen (`None`); speichert und lädt neu, wenn sich etwas geändert hat
pub fn project_paths(p: &mut Project, old: &str, new: Option<&str>) -> Result<bool, String> {
    let (Some(file), Some(mut pf)) = (p.file.clone(), p.pf.clone()) else { return Ok(false) };
    let before = pf.clone();
    for s in &mut pf.sources { match new { Some(n) => for f in &mut s.files { if f == old { *f = n.to_string(); } }, None => s.files.retain(|f| f != old) } }
    pf.sources.retain(|s| !s.files.is_empty() || new.is_some());
    pf.rename_asset(old, new);
    if pf.main.as_deref() == Some(old) { pf.main = new.map(String::from); }
    if let Some(d) = &mut pf.disk { match new { Some(n) => for f in &mut d.files { if f == old { *f = n.to_string(); } }, None => d.files.retain(|f| f != old) } }
    if pf == before { return Ok(false); }
    pf.save(&p.root.join(&file))?; p.reload(); Ok(true)
}

/// Datei umbenennen oder verschieben; das Ziel darf nicht existieren. Liefert den neuen Pfad
pub fn rename_file(p: &mut Project, old: &str, new: &str) -> Result<String, String> {
    let new = safe_rel(new)?; if new == old { return Ok(new); }
    let (from, to) = (abs_im_projekt(&p.root, old)?, abs_im_projekt(&p.root, &new)?);
    if !from.is_file() { return Err(vc_i18n::t!("fileops-old-gibt-es-nicht", old = &old)); }
    if to.exists() { return Err(vc_i18n::t!("fileops-new-gibt-es-schon", new = &new)); }
    if let Some(d) = to.parent() { std::fs::create_dir_all(d).map_err(|e| vc_i18n::t!("fileops-new-e", new = &new, e = &e))?; }
    std::fs::rename(&from, &to).map_err(|e| vc_i18n::t!("fileops-old-e", old = &old, e = &e))?;
    project_paths(p, old, Some(&new))?; p.rescan(); Ok(new)
}

/// Ordner umbenennen oder verschieben; alle Verweise der Projektdatei auf Dateien darin ziehen mit. Liefert den neuen Pfad und die Paare (alt, neu) der enthaltenen Dateien
pub fn rename_dir(p: &mut Project, old: &str, new: &str) -> Result<(String, Vec<(String, String)>), String> {
    let new = safe_rel(new)?; if new == old { return Ok((new, vec![])); }
    let (from, to) = (abs_im_projekt(&p.root, old)?, abs_im_projekt(&p.root, &new)?);
    if !from.is_dir() { return Err(vc_i18n::t!("fileops-old-gibt-es-nicht", old = &old)); }
    if new.starts_with(&format!("{old}/")) { return Err(vc_i18n::t!("fileops-ein-ordner-laesst-sich-nicht-in").into()); }
    if to.exists() { return Err(vc_i18n::t!("fileops-new-gibt-es-schon", new = &new)); }
    let mut dateien = vec![]; fn sammeln(d: &Path, rel: &str, out: &mut Vec<String>) { if let Ok(rd) = std::fs::read_dir(d) { for e in rd.flatten() { let n = e.file_name().to_string_lossy().to_string(); let r = format!("{rel}/{n}"); let Ok(t) = e.file_type() else { continue }; if t.is_symlink() { continue } if t.is_dir() { sammeln(&e.path(), &r, out) } else { out.push(r) } } } }
    sammeln(&from, old, &mut dateien);
    if let Some(d) = to.parent() { std::fs::create_dir_all(d).map_err(|e| vc_i18n::t!("fileops-new-e", new = &new, e = &e))?; }
    std::fs::rename(&from, &to).map_err(|e| vc_i18n::t!("fileops-old-e", old = &old, e = &e))?;
    let paare: Vec<(String, String)> = dateien.into_iter().map(|a| { let n = format!("{new}{}", &a[old.len()..]); (a, n) }).collect();
    for (a, n) in &paare { project_paths(p, a, Some(n))?; }
    p.rescan(); Ok((new, paare))
}

/// Ob der Papierkorb benutzt werden kann: macOS (Finder), Linux mit `gio trash` oder `trash-put`
pub fn can_trash() -> bool {
    if kein_prozess() { return false }
    if cfg!(target_os = "macos") || cfg!(windows) { return true; }   // Windows: Papierkorb über PowerShell (Microsoft.VisualBasic.FileIO)
    cfg!(target_os = "linux") && (has("gio") || has("trash-put"))
}
fn has(prog: &str) -> bool { std::process::Command::new(prog).arg("--version").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok() }

/// Datei in den Papierkorb legen; `false`, wenn das nicht geht
/// Sicherung für Tests: mit `VC_TEST_NO_SPAWN` (oder im eigenen Testlauf) startet dieses Modul nie einen Prozess (Finder, `gio`, `osascript`)
pub fn kein_prozess() -> bool { cfg!(test) || std::env::var_os("VC_TEST_NO_SPAWN").is_some() }

pub fn move_to_trash(abs: &Path) -> bool {
    if kein_prozess() { return false }
    use std::process::{Command, Stdio};
    let quiet = |mut c: Command| c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success());
    if cfg!(target_os = "macos") {
        // Der Pfad geht als Argument (`argv`) an das Skript, nie als Text in den Skriptcode: Anführungszeichen, Rückwärtsschrägstriche und Zeilenumbrüche im Namen können keinen Code einschleusen
        return Command::new("osascript").args(["-e", "on run argv", "-e", "tell application \"Finder\" to delete (POSIX file (item 1 of argv))", "-e", "end run"]).arg(abs).stdin(Stdio::null()).output().map_or(false, |o| o.status.success());
    }
    #[cfg(windows)]
    {
        // Pfad über die Umgebung, damit Anführungszeichen und Leerzeichen nichts durcheinanderbringen; kein Konsolenfenster
        use std::os::windows::process::CommandExt;
        let ps = "Add-Type -AssemblyName Microsoft.VisualBasic; $p = $env:VC_TRASH_PATH; if (Test-Path -LiteralPath $p -PathType Container) { [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory($p, 'OnlyErrorDialogs', 'SendToRecycleBin') } else { [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($p, 'OnlyErrorDialogs', 'SendToRecycleBin') }";
        let mut c = Command::new("powershell"); c.args(["-NoProfile", "-NonInteractive", "-Command", ps]).env("VC_TRASH_PATH", abs).creation_flags(0x0800_0000);
        return quiet(c);
    }
    if cfg!(target_os = "linux") {
        if has("gio") { let mut c = Command::new("gio"); c.arg("trash").arg("--").arg(abs); if quiet(c) { return true; } }
        if has("trash-put") { let mut c = Command::new("trash-put"); c.arg("--").arg(abs); if quiet(c) { return true; } }
    }
    false
}

/// Datei löschen: mit `papierkorb` in den Papierkorb (scheitert das, bleibt die Datei und es gibt einen Fehler), sonst endgültig. Liefert, ob sie im Papierkorb liegt
pub fn delete_file(p: &mut Project, rel: &str, papierkorb: bool) -> Result<bool, String> {
    delete_file_typed(p, rel, papierkorb).map_err(|e| e.text())
}

/// Fehler von [`delete_file_typed`]: der Papierkorb-Fall hat einen eigenen Fall, damit die Oberfläche ihn am Code erkennt (nicht am Text)
#[derive(Debug, Clone, PartialEq)]
pub enum DeleteFehler {
    /// Papierkorb gewünscht, aber nicht möglich; die Datei ist unverändert. Nach Rückfrage geht endgültiges Löschen mit `papierkorb = false`
    PapierkorbFehlgeschlagen { rel: String, text: String },
    Anders(String),
}
impl DeleteFehler {
    /// Meldungstext (übersetzt)
    pub fn text(&self) -> String { match self { DeleteFehler::PapierkorbFehlgeschlagen { text, .. } | DeleteFehler::Anders(text) => text.clone() } }
}

/// Wie [`delete_file`], mit unterscheidbarem Papierkorb-Fehler
pub fn delete_file_typed(p: &mut Project, rel: &str, papierkorb: bool) -> Result<bool, DeleteFehler> {
    let a = DeleteFehler::Anders;
    let rel = safe_rel(rel).map_err(a)?; let abs = abs_im_projekt(&p.root, &rel).map_err(DeleteFehler::Anders)?;
    if !abs.is_file() { return Err(DeleteFehler::Anders(vc_i18n::t!("fileops-rel-gibt-es-nicht", rel = &rel).to_string())); }
    // Papierkorb gewünscht, aber nicht möglich: nichts löschen, sondern melden (die Oberfläche kann nachfragen und mit `papierkorb = false` endgültig löschen)
    let im_korb = papierkorb && move_to_trash(&abs);
    if papierkorb && !im_korb { return Err(DeleteFehler::PapierkorbFehlgeschlagen { text: vc_i18n::t!("fileops-papierkorb-fehlgeschlagen", rel = &rel).to_string(), rel }); }
    if !im_korb { std::fs::remove_file(&abs).map_err(|e| DeleteFehler::Anders(vc_i18n::t!("fileops-rel-e", rel = &rel, e = &e).to_string()))?; }
    project_paths(p, &rel, None).map_err(DeleteFehler::Anders)?; p.rescan(); Ok(im_korb)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(n: &str) -> std::path::PathBuf { let d = std::env::temp_dir().join(format!("vc_fileops_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test]
    fn pfade_werden_geprueft() {
        assert_eq!(safe_rel("  lib\\rand.asm ").unwrap(), "lib/rand.asm");
        for bad in ["", "  ", "/etc/x", "../x", "a/../../x", "a//b", ".versteckt", "a/.git/x", "C:\\x"] { assert!(safe_rel(bad).is_err(), "{bad:?}"); }
    }
    #[test]
    fn anlegen_legt_ordner_an_und_ueberschreibt_nichts() {
        let r = tmp("neu");
        assert_eq!(create_file(&r, "lib/neu.asm").unwrap(), "lib/neu.asm"); assert!(r.join("lib/neu.asm").is_file());
        std::fs::write(r.join("a.asm"), "x").unwrap(); assert!(create_file(&r, "a.asm").is_err()); assert_eq!(std::fs::read_to_string(r.join("a.asm")).unwrap(), "x");
        assert!(create_file(&r, "../draussen.asm").is_err()); let _ = std::fs::remove_dir_all(&r);
    }
    #[test]
    fn ordner_anlegen() {
        let r = tmp("dir"); assert_eq!(create_dir(&r, "a/b").unwrap(), "a/b"); assert!(r.join("a/b").is_dir()); assert!(create_dir(&r, "a/b").is_err()); assert!(create_dir(&r, "../x").is_err()); let _ = std::fs::remove_dir_all(&r);
    }
    #[test]
    fn umbenennen_und_loeschen_ziehen_die_projektdatei_mit() {
        let r = tmp("ren");
        std::fs::write(r.join("main.asm"), "nop").unwrap(); std::fs::write(r.join("lib.asm"), "nop").unwrap();
        std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"main.asm","sources":[{"name":"p","files":["main.asm","lib.asm"]}]}"#).unwrap();
        let mut p = Project::load(r.clone(), Some("p.vcprj".into())).unwrap();
        assert_eq!(rename_file(&mut p, "main.asm", "start.asm").unwrap(), "start.asm");
        assert!(r.join("start.asm").is_file() && !r.join("main.asm").exists());
        let pf = p.pf.clone().unwrap(); assert_eq!(pf.main.as_deref(), Some("start.asm")); assert!(pf.sources[0].files.contains(&"start.asm".to_string()));
        assert!(rename_file(&mut p, "start.asm", "lib.asm").is_err(), "Ziel vorhanden");
        assert_eq!(delete_file(&mut p, "lib.asm", false).unwrap(), false); assert!(!r.join("lib.asm").exists());
        let pf = p.pf.clone().unwrap(); assert_eq!(pf.sources[0].files, vec!["start.asm".to_string()]);
        assert!(p.entries.iter().all(|e| e.rel != "lib.asm") && p.entries.iter().any(|e| e.rel == "start.asm"));
        assert!(delete_file(&mut p, "gibts-nicht.asm", false).is_err()); let _ = std::fs::remove_dir_all(&r);
    }
    #[test]
    fn ordner_umbenennen_zieht_projektverweise_nach() {
        let r = tmp("rendir"); std::fs::create_dir_all(r.join("lib/tief")).unwrap();
        std::fs::write(r.join("main.asm"), "nop").unwrap(); std::fs::write(r.join("lib/a.asm"), "nop").unwrap(); std::fs::write(r.join("lib/tief/b.asm"), "nop").unwrap();
        std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"main.asm","sources":[{"name":"p","files":["main.asm","lib/a.asm","lib/tief/b.asm"]}]}"#).unwrap();
        let mut p = Project::load(r.clone(), Some("p.vcprj".into())).unwrap();
        let (neu, paare) = rename_dir(&mut p, "lib", "bib").unwrap(); assert_eq!(neu, "bib"); assert_eq!(paare.len(), 2);
        assert!(r.join("bib/tief/b.asm").is_file() && !r.join("lib").exists());
        assert_eq!(p.pf.clone().unwrap().sources[0].files, vec!["main.asm".to_string(), "bib/a.asm".into(), "bib/tief/b.asm".into()]);
        assert!(rename_dir(&mut p, "bib", "bib/innen").is_err(), "nicht in sich selbst"); std::fs::create_dir_all(r.join("x")).unwrap(); assert!(rename_dir(&mut p, "bib", "x").is_err(), "Ziel vorhanden");
        let _ = std::fs::remove_dir_all(&r);
    }

    #[test]
    fn papierkorb_der_scheitert_loescht_nicht_still_endgueltig() {
        let r = tmp("korb");   // im Test gibt es keinen Papierkorb (kein_prozess)
        std::fs::write(r.join("a.asm"), "nop").unwrap(); std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"a.asm","sources":[{"name":"p","files":["a.asm"]}]}"#).unwrap();
        let mut p = Project::load(r.clone(), Some("p.vcprj".into())).unwrap();
        let e = delete_file(&mut p, "a.asm", true).unwrap_err(); assert!(e.contains("a.asm"), "{e}");
        assert!(r.join("a.asm").exists(), "Datei bleibt erhalten");
        assert_eq!(delete_file(&mut p, "a.asm", false).unwrap(), false); assert!(!r.join("a.asm").exists());
        let _ = std::fs::remove_dir_all(&r);
    }
    #[cfg(unix)]
    #[test]
    fn symlink_nach_draussen_wird_nicht_gefolgt() {
        let r = tmp("sym"); let aussen = tmp("sym-aussen");
        std::fs::write(aussen.join("geheim.txt"), "x").unwrap();
        std::os::unix::fs::symlink(&aussen, r.join("link")).unwrap();
        assert!(abs_im_projekt(&r, "link/geheim.txt").is_err());
        assert!(create_file(&r, "link/neu.asm").is_err()); assert!(!aussen.join("neu.asm").exists());
        assert!(create_dir(&r, "link/ordner").is_err()); assert!(!aussen.join("ordner").exists());
        std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"main.asm","sources":[{"name":"p","files":["main.asm"]}]}"#).unwrap(); std::fs::write(r.join("main.asm"), "nop").unwrap();
        let mut p = Project::load(r.clone(), Some("p.vcprj".into())).unwrap();
        assert!(delete_file(&mut p, "link/geheim.txt", false).is_err()); assert!(aussen.join("geheim.txt").exists());
        assert!(rename_file(&mut p, "link/geheim.txt", "x.txt").is_err()); assert!(aussen.join("geheim.txt").exists());
        std::fs::create_dir_all(r.join("d")).unwrap(); std::os::unix::fs::symlink(&aussen, r.join("d/l")).unwrap();
        let (_, dateien) = rename_dir(&mut p, "d", "e").unwrap(); assert!(dateien.is_empty(), "Symlink zählt nicht als Inhalt: {dateien:?}");
        assert!(aussen.join("geheim.txt").exists());
        let _ = std::fs::remove_dir_all(&r); let _ = std::fs::remove_dir_all(&aussen);
    }
}
