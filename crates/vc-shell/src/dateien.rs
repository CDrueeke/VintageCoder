// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateiaktionen des Projektbaums über die Fachschicht: Verschieben mit Anpassen der Include-Pfade und Sicherung (und Rückgängig), Pfade, im Ordner zeigen,
//! Standardprogramm, Änderungsstand. Gleiches Verhalten wie im Iced-Baum (`baum_verschieben`, `baum_rueckgaengig`).
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::reorganize as ro;

/// Kein Prozess starten (Tests, Prüfbrücke): `VC_TEST_NO_SPAWN`
fn gesperrt() -> bool { cfg!(test) || std::env::var_os("VC_TEST_NO_SPAWN").is_some() }

fn fehler(code: &str, e: impl std::fmt::Display) -> Fehler { Fehler::mit(code, json!({ "detail": e.to_string() })) }

/// Datei, die nicht als Text geöffnet wird (Bilder, Binärdaten, Archive); unbekannte Endung: Inhalt entscheidet (NUL-Byte)
pub fn ist_nichttext(p: &Path) -> bool {
    let l = p.to_string_lossy().to_lowercase();
    if [".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".pdf", ".zip", ".gz", ".tar", ".7z", ".bin", ".dat", ".prg", ".crt", ".rom", ".sid", ".wav", ".mp3", ".ogg"].iter().any(|e| l.ends_with(e)) { return true; }
    // nur den Anfang lesen, nicht die ganze Datei
    use std::io::Read;
    let mut buf = Vec::with_capacity(4096);
    std::fs::File::open(p).and_then(|f| f.take(4096).read_to_end(&mut buf)).map(|_| buf.contains(&0)).unwrap_or(false)
}

/// Leere Ordner entfernen (ohne Symlinks zu verfolgen)
fn loesche_leere_ordner(p: &Path) {
    if let Ok(rd) = std::fs::read_dir(p) { for e in rd.flatten() { if e.file_type().map_or(false, |t| t.is_dir()) { loesche_leere_ordner(&e.path()); } } }
    let _ = std::fs::remove_dir(p);
}

/// Datei oder Ordner in `folder` verschieben (`""` = Wurzel). Include-Pfade in Quelltexten und Projektverweise ziehen mit; vorher wird gesichert.
/// Antwort: `{ moved: [[alt, neu]], rewritten: [[datei, n]] }` (Dateinamen nach dem Verschieben)
pub fn verschieben(root: &Path, from: &str, folder: &str) -> Result<Value, Fehler> {
    let from = vc_ide_model::fileops::safe_rel(from).map_err(|e| fehler("pfad-ungueltig", e))?;
    let folder = if folder.is_empty() { String::new() } else { vc_ide_model::fileops::safe_rel(folder).map_err(|e| fehler("pfad-ungueltig", e))? };
    let name = from.rsplit('/').next().unwrap_or(&from).to_string();
    let neu = if folder.is_empty() { name } else { format!("{folder}/{name}") };
    if neu == from { return Ok(json!({ "moved": [], "rewritten": [] })); }
    if folder == from || folder.starts_with(&format!("{from}/")) { return Err(Fehler::neu("ordner-in-sich")); }
    umziehen(root, &from, &neu)
}

/// Gemeinsamer Teil von Verschieben und Umbenennen: Datei oder Ordner `from` nach `neu` bringen; Include-Pfade und Projektverweise ziehen mit, vorher wird gesichert.
fn umziehen(root: &Path, from: &str, neu: &str) -> Result<Value, Fehler> {
    let ist_ordner = vc_ide::safepath::resolve(root, from).map_err(|e| fehler("pfad-ungueltig", e))?.is_dir();
    let paare: Vec<(String, String)> = if ist_ordner {
        ro::list_files(root).into_iter().filter_map(|f| f.strip_prefix(&format!("{from}/")).map(|r| (f.clone(), format!("{neu}/{r}")))).collect()
    } else { vec![(from.to_string(), neu.to_string())] };
    if paare.is_empty() { return Err(Fehler::mit("nichts-zu-verschieben", json!({ "rel": from }))); }
    let moves: Vec<ro::Move> = paare.iter().map(|(a, b)| ro::Move { from: a.clone(), to: b.clone(), category: vc_ide::importfiles::Category::Source, label: String::new(), conflict: false }).collect();
    let file = vc_ide::projfile::find(root).ok_or_else(|| Fehler::neu("verschieben-braucht-projektdatei"))?;
    let mut pf = vc_ide::projfile::ProjectFile::load(&file).map_err(|e| fehler("projekt-fehler", e))?;
    let rep = ro::apply(root, &file, &mut pf, &moves, None, true, &|_| {}).map_err(|e| fehler("verschieben-fehlgeschlagen", e))?;
    if ist_ordner { loesche_leere_ordner(&root.join(from)); }
    let wohin = |f: &str| paare.iter().find(|(a, _)| a == f).map_or(f.to_string(), |(_, b)| b.clone());
    Ok(json!({ "moved": paare, "rewritten": rep.rewritten.iter().map(|(f, n)| json!([wohin(f), n])).collect::<Vec<_>>() }))
}

/// Datei oder Ordner umbenennen (auch in einen anderen Pfad). Wie beim Verschieben ziehen die Include-Pfade in Quelltexten und die Verweise der Projektdatei mit
/// (vorher wird gesichert, `tree_undo` nimmt es zurück). Ohne Projektdatei bleibt es beim einfachen Umbenennen ohne Anpassen (`Ok(None)`).
pub fn umbenennen(root: &Path, old: &str, new: &str) -> Result<Option<(String, Value)>, Fehler> {
    let old = vc_ide_model::fileops::safe_rel(old).map_err(|e| fehler("pfad-ungueltig", e))?;
    let new = vc_ide_model::fileops::safe_rel(new).map_err(|e| fehler("pfad-ungueltig", e))?;
    if new == old { return Ok(Some((new, json!({ "moved": [], "rewritten": [] })))); }
    if vc_ide::projfile::find(root).is_none() { return Ok(None); }
    if !root.join(&old).exists() { return Err(fehler("datei-fehler", vc_i18n::t!("fileops-old-gibt-es-nicht", old = &old))); }
    if root.join(&new).exists() { return Err(fehler("datei-fehler", vc_i18n::t!("fileops-new-gibt-es-schon", new = &new))); }
    if root.join(&old).is_dir() && new.starts_with(&format!("{old}/")) { return Err(Fehler::neu("ordner-in-sich")); }
    if let Some(d) = root.join(&new).parent() { std::fs::create_dir_all(d).map_err(|e| fehler("datei-fehler", e))?; }
    let out = umziehen(root, &old, &new)?;
    Ok(Some((new, out)))
}

/// Letztes Verschieben zurücknehmen (Sicherung unter `.vcbackup/`); überschreibt nichts, wenn sich Dateien seitdem geändert haben.
/// Antwort: `{ moves: [[alt, neu]] (die Bewegung, die zurückgenommen wurde), n }`
pub fn rueckgaengig(root: &Path) -> Result<Value, Fehler> {
    let l = ro::letzte(root).ok_or_else(|| Fehler::neu("nichts-rueckgaengig"))?;
    let n = ro::undo_geprueft(root, &l).map_err(|e| fehler("rueckgaengig-fehlgeschlagen", e))?;
    let _ = std::fs::remove_dir_all(&l.backup);
    let alte: std::collections::BTreeSet<String> = l.moves.iter().filter_map(|(_, b)| b.rsplit_once('/').map(|(o, _)| o.to_string())).collect();
    for o in alte { loesche_leere_ordner(&root.join(o)); }
    Ok(json!({ "moves": l.moves, "n": n }))
}

pub fn absolut(root: &Path, rel: &str) -> Result<PathBuf, Fehler> {
    let rel = vc_ide_model::fileops::safe_rel(rel).map_err(|e| fehler("pfad-ungueltig", e))?; vc_ide::safepath::resolve(root, &rel).map_err(|e| fehler("pfad-ungueltig", e))
}

/// Include-Absprung (Konzept Include-von-Medien 4.5): löst den Pfad einer eingebundenen Datei auf – zuerst neben der einbindenden Datei `vom`, dann ab der
/// Projektwurzel. Ergebnis: `status` = `ok` (mit `rel` und `medium`), `legacy` (Altpfad `build/export/…`, Datei fehlt) oder `missing`.
pub fn include_aufloesen(root: &Path, vom: &str, pfad: &str) -> Value {
    let pfad = pfad.trim().replace('\\', "/"); let pfad = pfad.trim_start_matches("./");
    let mut kandidaten: Vec<String> = vec![];
    if let Some((dir, _)) = vom.rsplit_once('/') { kandidaten.push(format!("{dir}/{pfad}")); }
    kandidaten.push(pfad.to_string());
    for k in kandidaten {
        let Ok(rel) = vc_ide_model::fileops::safe_rel(&k) else { continue };
        if root.join(&rel).is_file() {
            let rel = rel.replace('\\', "/");
            let medium = [".vcsprite", ".vcpmg", ".vcchar", ".vctiles", ".vcscreen", ".vcsong", ".vcplay", ".vcpal"].iter().any(|e| rel.to_ascii_lowercase().ends_with(e));
            return json!({ "status": "ok", "rel": rel, "medium": medium });
        }
    }
    json!({ "status": if pfad.starts_with("build/export/") { "legacy" } else { "missing" } })
}

fn starte(mut c: std::process::Command) -> Result<(), Fehler> {
    if gesperrt() { return Ok(()); }
    c.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ()).map_err(|e| fehler("extern-fehlgeschlagen", e))
}

/// Im Dateimanager zeigen (Finder, Explorer, Ordner des Dateimanagers)
pub fn im_ordner_zeigen(p: &Path) -> Result<(), Fehler> {
    #[cfg(target_os = "macos")] { let mut c = std::process::Command::new("open"); c.arg("-R").arg(p); starte(c) }
    #[cfg(target_os = "windows")] { use std::os::windows::process::CommandExt; let mut c = std::process::Command::new("explorer"); c.raw_arg(format!("/select,\"{}\"", p.display())); starte(c) }   // in Anführungszeichen, sonst öffnet „Dokumente“ bei Leerzeichen im Pfad
    #[cfg(all(unix, not(target_os = "macos")))] { let mut c = std::process::Command::new("xdg-open"); c.arg(if p.is_dir() { p } else { p.parent().unwrap_or(p) }); starte(c) }
}
/// Endungen, bei denen „Öffnen“ ein Programm ausführt (Windows, macOS, Linux)
const AUSFUEHRBAR: &[&str] = &["exe", "bat", "cmd", "com", "lnk", "scr", "ps1", "vbs", "vbe", "js", "jse", "wsf", "msi", "msc", "reg", "pif", "command", "app", "sh", "desktop", "jar", "workflow", "pkg", "dmg", "run", "appimage", "scpt", "terminal"];
/// Würde das Öffnen der Datei im Standardprogramm ein Programm ausführen? (nach Endung; Ordner mit `.app` zählen ebenso)
pub fn ist_ausfuehrbar(p: &Path) -> bool {
    p.extension().map_or(false, |e| { let e = e.to_string_lossy().to_ascii_lowercase(); AUSFUEHRBAR.contains(&e.as_str()) })
}
/// Im Standardprogramm des Systems öffnen
pub fn standardprogramm(p: &Path) -> Result<(), Fehler> {
    #[cfg(target_os = "macos")] { let mut c = std::process::Command::new("open"); c.arg(p); starte(c) }
    // Windows ohne `cmd`: `&`, `^` und `%` im Dateinamen werden so nicht ausgewertet
    #[cfg(target_os = "windows")] { use std::os::windows::process::CommandExt; let mut c = std::process::Command::new("rundll32"); c.arg("url.dll,FileProtocolHandler").arg(p).creation_flags(0x0800_0000); starte(c) }
    #[cfg(all(unix, not(target_os = "macos")))] { let mut c = std::process::Command::new("xdg-open"); c.arg(p); starte(c) }
}

/// Änderungsstand (Größe, Änderungszeit in ms) je Datei; fehlende Datei: `null`
pub fn staende(root: &Path, rels: &[String]) -> Value {
    Value::Array(rels.iter().map(|r| {
        let m = vc_ide_model::fileops::safe_rel(r).ok().and_then(|r| vc_ide::safepath::resolve(root, &r).ok()).and_then(|a| std::fs::metadata(a).ok());
        match m { Some(m) => json!([m.len(), m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_millis() as u64)]), None => Value::Null }
    }).collect())
}

#[cfg(test)]
mod include_tests {
    use super::*;
    #[test] fn include_pfad_wird_neben_der_datei_dann_ab_der_wurzel_gesucht() {
        let d = std::env::temp_dir().join(format!("vc-inc-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src")).unwrap(); std::fs::create_dir_all(d.join("graphics")).unwrap();
        std::fs::write(d.join("src/lib.inc"), "").unwrap(); std::fs::write(d.join("graphics/hero.vcsprite"), "").unwrap();
        assert_eq!(include_aufloesen(&d, "src/main.asm", "lib.inc"), json!({ "status": "ok", "rel": "src/lib.inc", "medium": false }));
        assert_eq!(include_aufloesen(&d, "src/main.asm", "graphics/hero.vcsprite"), json!({ "status": "ok", "rel": "graphics/hero.vcsprite", "medium": true }));
        assert_eq!(include_aufloesen(&d, "src/main.asm", "nix.inc")["status"], "missing");
        assert_eq!(include_aufloesen(&d, "src/main.asm", "build/export/hero.inc")["status"], "legacy");
        assert_eq!(include_aufloesen(&d, "src/main.asm", "../../etc/passwd")["status"], "missing");
        let _ = std::fs::remove_dir_all(&d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_dateien_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
    fn projekt(r: &Path) {
        std::fs::create_dir_all(r.join("lib")).unwrap();
        std::fs::write(r.join("main.asm"), ".include \"lib/a.inc\"\n lda #1\n").unwrap(); std::fs::write(r.join("lib/a.inc"), "nop\n").unwrap();
        std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"main.asm","sources":[{"name":"p","files":["main.asm"]}]}"#).unwrap();
    }
    #[test] fn nichttext_nach_endung_und_inhalt() {
        let t = tmp("nt"); std::fs::write(t.join("a.xyz"), [1u8, 0, 2]).unwrap(); std::fs::write(t.join("b.xyz"), "hallo").unwrap(); std::fs::write(t.join("c.asm"), "nop").unwrap();
        assert!(ist_nichttext(Path::new("bild.PNG")) && ist_nichttext(&t.join("a.xyz")) && !ist_nichttext(&t.join("b.xyz")) && !ist_nichttext(&t.join("c.asm"))); let _ = std::fs::remove_dir_all(&t);
    }
    #[test] fn verschieben_zieht_include_pfade_mit_und_rueckgaengig_stellt_her() {
        let r = tmp("mv"); projekt(&r);
        let out = verschieben(&r, "lib/a.inc", "").unwrap();
        assert_eq!(out["moved"], json!([["lib/a.inc", "a.inc"]]));
        assert!(r.join("a.inc").is_file() && !r.join("lib/a.inc").exists());
        let neu = std::fs::read_to_string(r.join("main.asm")).unwrap(); assert!(neu.contains("\"a.inc\""), "{neu}");
        let rw = out["rewritten"].as_array().unwrap(); assert!(rw.iter().any(|x| x[0] == "main.asm"), "{rw:?}");
        let u = rueckgaengig(&r).unwrap(); assert_eq!(u["moves"], json!([["lib/a.inc", "a.inc"]]));
        assert!(r.join("lib/a.inc").is_file() && std::fs::read_to_string(r.join("main.asm")).unwrap().contains("lib/a.inc"));
        assert_eq!(rueckgaengig(&r).unwrap_err().code, "nichts-rueckgaengig"); let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn umbenennen_passt_include_und_projektdatei_an_und_rueckgaengig_stellt_her() {
        let r = tmp("ren"); projekt(&r);
        let (neu, out) = umbenennen(&r, "lib/a.inc", "lib/b.inc").unwrap().unwrap();
        assert_eq!(neu, "lib/b.inc"); assert!(r.join("lib/b.inc").is_file() && !r.join("lib/a.inc").exists());
        let m = std::fs::read_to_string(r.join("main.asm")).unwrap(); assert!(m.contains("lib/b.inc") && !m.contains("lib/a.inc"), "{m}");
        assert!(out["rewritten"].as_array().unwrap().iter().any(|x| x[0] == "main.asm"));
        // Hauptdatei umbenennen: Projektdatei zieht mit
        let (_, _) = umbenennen(&r, "main.asm", "haupt.asm").unwrap().unwrap();
        let p = std::fs::read_to_string(r.join("p.vcprj")).unwrap(); assert!(p.contains("haupt.asm") && !p.contains("\"main.asm\""), "{p}");
        rueckgaengig(&r).unwrap(); assert!(r.join("main.asm").is_file());
        // Ordner umbenennen
        umbenennen(&r, "lib", "inc").unwrap().unwrap(); assert!(std::fs::read_to_string(r.join("main.asm")).unwrap().contains("inc/b.inc"));
        // vorhandener Name wird gemeldet, ohne Projektdatei kein Anpassen
        assert_eq!(umbenennen(&r, "main.asm", "inc/b.inc").unwrap_err().code, "datei-fehler");
        let k = tmp("ren2"); std::fs::write(k.join("x.asm"), "nop").unwrap(); assert!(umbenennen(&k, "x.asm", "y.asm").unwrap().is_none());
        let _ = std::fs::remove_dir_all(&r); let _ = std::fs::remove_dir_all(&k);
    }
    #[test] fn ordner_nicht_in_sich_und_fehlende_projektdatei() {
        let r = tmp("self"); projekt(&r);
        assert_eq!(verschieben(&r, "lib", "lib").unwrap_err().code, "ordner-in-sich");
        std::fs::create_dir_all(r.join("lib/sub")).unwrap(); assert_eq!(verschieben(&r, "lib", "lib/sub").unwrap_err().code, "ordner-in-sich");
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn staende_melden_groesse_und_fehlende() {
        let r = tmp("st"); std::fs::write(r.join("a.asm"), "abc").unwrap(); let s = staende(&r, &["a.asm".into(), "fehlt.asm".into()]);
        assert_eq!(s[0][0], 3); assert!(s[1].is_null()); let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn ausfuehrbare_endungen_werden_erkannt() {
        for n in ["a.exe", "B.BAT", "x.command", "Spiel.app", "s.sh", "z.JAR", "a&calc.exe", "k.desktop"] { assert!(ist_ausfuehrbar(Path::new(n)), "{n}"); }
        for n in ["a.asm", "b.png", "c.d64", "exe", "noext", "d.txt"] { assert!(!ist_ausfuehrbar(Path::new(n)), "{n}"); }
    }
}
