// SPDX-License-Identifier: GPL-3.0-or-later
//! Turbo BASIC XL in der Oberfläche (Konzept „Turbo BASIC XL starten“, Etappe E3): Zustand der Projektdiskette, Prüfung, Bereitstellen, Verkleinern,
//! Einstellungen (`turbo` der Projektdatei) und Bauen/Starten. Alles Fachliche steht in `vc_ide::turbo`; hier wird nur für die Oberfläche in JSON gepackt.
use serde_json::{json, Value};
use std::path::Path;
use vc_ide::projfile::{ProjectFile, TurboSpec};
use vc_ide::turbo::{self, Report};

/// Ist `rel` die Diskette des Anwenders im Projekt (`turbo/*.atr`, nicht tiefer)? Dafür zeigt der Projektbaum die Marke „Diskette (Anwender)“.
pub fn ist_anwenderdiskette(rel: &str) -> bool {
    let l = rel.to_ascii_lowercase();
    l.strip_prefix("turbo/").map_or(false, |r| r.ends_with(".atr") && !r.contains('/'))
}

/// Enthält das Projekt eine Diskette des Anwenders (`turbo/*.atr`)? Nach dem Kopieren eines Projekts folgt dann der Hinweis „nicht jeder darf sie weitergeben“.
pub fn enthaelt_diskette(root: &Path) -> bool {
    std::fs::read_dir(root.join(turbo::DIR)).map_or(false, |d| d.flatten().any(|e| ist_anwenderdiskette(&format!("{}/{}", turbo::DIR, e.file_name().to_string_lossy()))))
}

fn kb(sectors: usize) -> String { format!("{:.1}", sectors as f64 * 125.0 / 1000.0) }

/// Prüfergebnis für die Oberfläche: Felder von [`Report::to_json`] plus lesbare Zeilen, Namen, Größe und die Vorschau der Verkleinerung
pub fn report_json(r: &Report, bytes: &[u8]) -> Value {
    let mut v = r.to_json();
    v["lines"] = json!(r.lines());
    v["kb"] = json!(kb(r.free_sectors));
    v["name"] = json!(r.known.as_ref().map(|k| k.label.clone()).unwrap_or_else(|| if r.is_turbo { vc_i18n::t!("turboui-name-unbekannt").to_string() } else { String::new() }));
    v["entries"] = json!(r.entries.iter().map(|e| json!({ "name": e.name, "sectors": e.sectors, "locked": e.locked })).collect::<Vec<_>>());
    v["programLocked"] = json!(r.program_locked);
    v["reduce"] = turbo::reduce_preview(bytes).map_or(Value::Null, |(names, free)| json!({ "removes": names, "free": free, "kb": kb(free) }));
    v
}

fn laden(root: &Path) -> Result<(std::path::PathBuf, ProjectFile), String> {
    let pfile = vc_ide::projfile::find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let pf = ProjectFile::load(&pfile)?;
    Ok((pfile, pf))
}

/// Ist das geöffnete Projekt ein Turbo-BASIC-XL-Projekt? (für die Oberfläche: Karte, Banner, Debuggen ausgegraut)
pub fn ist_turbo_projekt(root: &Path) -> bool { laden(root).map_or(false, |(_, pf)| turbo::is_turbo_project(&pf)) }

/// Zustand für die Karte „Turbo BASIC XL“ und das Banner: `turbo` (ist es ein Turbo-Projekt), die Angaben der Projektdatei, ob die Diskette da ist,
/// ihre Prüfung und Probleme der Angaben (Namen) mit Warnungen
pub fn zustand(root: &Path) -> Result<Value, String> {
    let (_, pf) = laden(root)?;
    let is = turbo::is_turbo_project(&pf);
    let spec = pf.turbo.clone().unwrap_or_default();
    let program = if spec.program.is_empty() { turbo::PROGRAM.to_string() } else { spec.program.clone() };
    let mut out = json!({ "turbo": is, "disk": spec.disk, "present": false, "reduced": spec.reduced, "program": program, "programIsDefault": spec.program.is_empty(),
        "files": spec.files, "changed": false, "report": Value::Null, "errors": [], "warnings": [], "free": null });
    let (errs, warns) = turbo::spec_problems(&spec, root);
    out["errors"] = json!(errs); out["warnings"] = json!(warns);
    if !spec.disk.is_empty() {
        let path = root.join(&spec.disk);
        if let Ok(bytes) = std::fs::read(&path) {
            out["present"] = json!(true);
            if let Ok(r) = turbo::check_bytes(&bytes) {
                out["changed"] = json!(!spec.sha256.is_empty() && !r.sha256.eq_ignore_ascii_case(&spec.sha256));
                out["free"] = json!(r.free_sectors);
                out["report"] = report_json(&r, &bytes);
            }
        }
    }
    Ok(out)
}

/// Prüft eine beliebige Datei (vor dem Bereitstellen) und sagt, wohin sie kopiert würde und ob dort schon eine Kopie liegt
pub fn pruefen_datei(root: &Path, src: &Path) -> Result<Value, String> {
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let r = turbo::check_bytes(&bytes)?;
    let mut v = report_json(&r, &bytes);
    let name = src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let rel = format!("{}/{name}", turbo::DIR);
    v["dest"] = json!(rel);
    v["destExists"] = json!(root.join(&rel).exists());
    v["sameAsCopy"] = json!(matches!((std::fs::canonicalize(src), std::fs::canonicalize(root.join(&rel))), (Ok(a), Ok(b)) if a == b));
    v["file"] = json!(name);
    // Gibt es schon eine Projektkopie (anderer Name), nennt die Oberfläche sie beim Ersetzen
    v["current"] = json!(laden(root).ok().and_then(|(_, pf)| pf.turbo).map(|t| t.disk).filter(|d| !d.is_empty()));
    Ok(v)
}

/// Prüft die bereitgestellte Kopie des Projekts erneut; bei Fehlern der Prüfung die Meldungen, sonst `Ok` mit dem Zustand
pub fn pruefen_kopie(root: &Path) -> Result<Value, String> {
    let z = zustand(root)?;
    if z["disk"].as_str().unwrap_or("").is_empty() { return Err(vc_i18n::t!("turbo-diskette-fehlt").to_string()); }
    if z["present"] != true { return Err(vc_i18n::t!("turbo-diskette-datei-fehlt", disk = z["disk"].as_str().unwrap_or("")).to_string()); }
    Ok(z)
}

/// Speichert Programmname und Zusatzdateien in `turbo` der Projektdatei (leerer Programmname = Standard). Namen werden vorher geprüft.
pub fn einstellungen_setzen(root: &Path, program: &str, files: &[String]) -> Result<(), String> {
    let (pfile, mut pf) = laden(root)?;
    let mut t = pf.turbo.clone().unwrap_or_default();
    let prog = program.trim().to_ascii_uppercase();
    t.program = if prog == turbo::PROGRAM { String::new() } else { prog };
    t.files = files.iter().map(|f| f.trim().to_string()).filter(|f| !f.is_empty()).collect();
    let (errs, _) = turbo::spec_problems(&t, root);
    if let Some(e) = errs.into_iter().next() { return Err(e); }
    // Ein leeres `turbo` (nichts bereitgestellt, nichts eingestellt) bleibt aus der Datei heraus
    pf.turbo = if t == TurboSpec::default() { None } else { Some(t) };
    pf.save(&pfile)
}

/// Baut ohne zu starten (F7 im Turbo-Projekt, Testläufe ohne Emulator): `Ok` mit dem Bauergebnis im Aufbau des Kerns (`turbo.file`, `turbo.line`, …)
/// bzw. `Err((fehlt, Meldung))`; `fehlt` heißt: nur die Diskette fehlt (Hinweis, kein Fehler des Programms)
pub fn bauen(root: &Path, data: &Path) -> Result<Value, (bool, String)> {
    let (_, pf) = laden(root).map_err(|e| (false, e))?;
    if !turbo::is_turbo_project(&pf) { return Err((false, vc_i18n::t!("turbo-kein-turbo-projekt").to_string())); }
    let main = pf.main_file().unwrap_or_default();
    let r = vc_ide::projfile::build_project(&vc_ide::build::Ctx { root, data }, &pf, Some(&main));
    if r["ok"] != true {
        let msg = r["turbo_error"].as_str().map(String::from).or_else(|| r["results"][0]["diagnostics"][0]["message"].as_str().map(String::from));
        return Err((false, msg.unwrap_or_else(|| vc_i18n::t!("turbo-bauen-fehlgeschlagen").to_string())));
    }
    if let Some(m) = r["turbo_missing"].as_str() { return Err((true, m.to_string())); }
    Ok(r)
}

/// Fehlerart eines Starts aus `vc_ide::turbo::start`: fehlt nur die Diskette? (Diskette nicht eingetragen oder Datei fehlt)
pub fn diskette_fehlt(root: &Path) -> bool {
    match laden(root) {
        Ok((_, pf)) => match pf.turbo { Some(t) if !t.disk.is_empty() => !root.join(&t.disk).is_file(), _ => true },
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use vc_disk::Atr;

    fn tmp(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc_shell_turbo_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
    fn bytes(n: usize, seed: u8) -> Vec<u8> { (0..n).map(|i| (i as u8).wrapping_mul(13).wrapping_add(seed)).collect() }
    pub fn fake_disk(sig: bool) -> Vec<u8> {
        let mut a = Atr::new_dos2();
        a.dos2_add("DOS.SYS", &bytes(4625, 1)).unwrap(); a.dos2_add("DUP.SYS", &bytes(5126, 2)).unwrap();
        let mut auto = bytes(6000, 3); if sig { auto[100..133].copy_from_slice(b"TURBO-BASIC XL 1.5 (c) 1985 Frank"); }
        a.dos2_add("AUTORUN.SYS", &auto).unwrap(); a.dos2_add("MAGIC.TUR", &bytes(125 * 50, 5)).unwrap();
        for n in ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS", "MAGIC.TUR"] { a.dos2_set_locked(n, true).unwrap(); }
        a.to_bytes()
    }
    fn projekt(n: &str) -> PathBuf {
        let root = tmp(n);
        std::fs::write(root.join("prog.vcprj"), r#"{ "format": 1, "project": {"name": "T"},
          "targets": [{ "id": "atari8", "system": "atari8", "model": "800xl", "variant": "pal", "cpu": "sally" }],
          "sources": [{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["prog.tur"] }], "main": "prog.tur" }"#).unwrap();
        std::fs::write(root.join("prog.tur"), "PRINT \"HALLO\"\n").unwrap();
        root
    }

    #[test] fn marke_nur_fuer_turbo_atr() {
        assert!(ist_anwenderdiskette("turbo/Turbo_Basic_XL.atr") && ist_anwenderdiskette("turbo/X.ATR"));
        assert!(!ist_anwenderdiskette("build/prog.atr") && !ist_anwenderdiskette("turbo/sub/x.atr") && !ist_anwenderdiskette("turbo/notiz.txt"));
    }
    #[test] fn diskette_im_projekt_wird_erkannt() {
        let root = projekt("enthaelt"); assert!(!enthaelt_diskette(&root));
        std::fs::create_dir_all(root.join("turbo")).unwrap(); std::fs::write(root.join("turbo/notiz.txt"), "x").unwrap(); assert!(!enthaelt_diskette(&root));
        std::fs::write(root.join("turbo/Eigene.ATR"), fake_disk(true)).unwrap(); assert!(enthaelt_diskette(&root));
    }
    #[test] fn zustand_ohne_und_mit_diskette() {
        let root = projekt("zustand");
        let z = zustand(&root).unwrap();
        assert_eq!((z["turbo"].as_bool(), z["present"].as_bool(), z["program"].as_str()), (Some(true), Some(false), Some("AUTORUN.BAS")));
        assert!(diskette_fehlt(&root));
        let src = tmp("zustand_src").join("Meine_Turbo.atr"); std::fs::write(&src, fake_disk(true)).unwrap();
        let p = pruefen_datei(&root, &src).unwrap();
        assert_eq!((p["turbo"].as_bool(), p["destExists"].as_bool(), p["dest"].as_str(), p["version"].as_str()), (Some(true), Some(false), Some("turbo/Meine_Turbo.atr"), Some("1.5")));
        assert!(p["reduce"]["removes"].as_array().unwrap().iter().any(|n| n == "MAGIC.TUR") && p["reduce"]["free"].as_u64().unwrap() > p["free_sectors"].as_u64().unwrap());
        let r = turbo::provide(&root, &src, &turbo::ProvideOpts::default()).unwrap();
        assert_eq!(r.disk, "turbo/Meine_Turbo.atr");
        let z = zustand(&root).unwrap();
        assert_eq!((z["present"].as_bool(), z["reduced"].as_bool(), z["changed"].as_bool(), z["report"]["turbo"].as_bool()), (Some(true), Some(false), Some(false), Some(true)));
        assert!(!diskette_fehlt(&root));
        assert!(std::fs::read_to_string(root.join(".gitignore")).unwrap().contains("turbo/*.atr"));
        let free = z["free"].as_u64().unwrap();
        turbo::reduce_project_copy(&root).unwrap();
        let z = zustand(&root).unwrap();
        assert_eq!(z["reduced"].as_bool(), Some(true)); assert!(z["free"].as_u64().unwrap() > free);
        // Kopie von Hand verändert: „changed“
        let mut b = std::fs::read(root.join("turbo/Meine_Turbo.atr")).unwrap(); let n = b.len(); b[n - 1] ^= 1; std::fs::write(root.join("turbo/Meine_Turbo.atr"), b).unwrap();
        assert_eq!(zustand(&root).unwrap()["changed"].as_bool(), Some(true));
    }
    #[test] fn nicht_erkannte_diskette_wird_gemeldet_und_nicht_ohne_wahl_bereitgestellt() {
        let root = projekt("fremd");
        let src = tmp("fremd_src").join("x.atr"); std::fs::write(&src, fake_disk(false)).unwrap();
        let p = pruefen_datei(&root, &src).unwrap();
        assert_eq!(p["turbo"].as_bool(), Some(false));
        assert!(turbo::provide(&root, &src, &turbo::ProvideOpts::default()).is_err());
        assert!(turbo::provide(&root, &src, &turbo::ProvideOpts { anyway: true, ..Default::default() }).is_ok());
    }
    #[test] fn einstellungen_pruefen_namen_und_bleiben_leer_ohne_angaben() {
        let root = projekt("einst");
        assert!(einstellungen_setzen(&root, "zu_langer_name.bas", &[]).is_err(), "Namen werden geprüft");
        einstellungen_setzen(&root, "start.bas", &["daten/a.dat:A.DAT".into()]).unwrap();
        let z = zustand(&root).unwrap();
        assert_eq!((z["program"].as_str(), z["programIsDefault"].as_bool(), z["files"][0].as_str()), (Some("START.BAS"), Some(false), Some("daten/a.dat:A.DAT")));
        assert!(!z["warnings"].as_array().unwrap().is_empty(), "fehlende Zusatzdatei wird gemeldet");
        einstellungen_setzen(&root, "AUTORUN.BAS", &[]).unwrap();
        assert!(!std::fs::read_to_string(root.join("prog.vcprj")).unwrap().contains("\"turbo\""), "nichts Besonderes: kein Feld");
    }
    #[test] fn bauen_ohne_diskette_meldet_fehlt() {
        let root = projekt("bauen");
        let data = crate::lsp::daten_ordner().expect("Datenordner");
        match bauen(&root, &data) { Err((true, m)) => assert!(m.to_lowercase().contains("turbo"), "{m}"), x => panic!("{x:?}") }
        let src = tmp("bauen_src").join("T.atr"); std::fs::write(&src, fake_disk(true)).unwrap();
        turbo::provide(&root, &src, &turbo::ProvideOpts::default()).unwrap();
        let r = bauen(&root, &data).expect("gebaut");
        assert!(root.join(r["turbo"]["file"].as_str().unwrap()).is_file());
    }
}
