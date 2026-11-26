// SPDX-License-Identifier: GPL-3.0-or-later
//! Projekt kopieren: der ganze Projektordner wird unter einem neuen Namen angelegt; die Projektdatei (`*.vcprj`) bekommt den neuen
//! Namen (Dateiname und `project.name`). Standardmäßig bleiben der Bauordner `build/` (lässt sich neu erzeugen) und der Git-Ordner
//! `.git/` (die Kopie soll ein eigenes Projekt werden) zurück; beides lässt sich einschalten. Es wird zuerst in einen Zwischenordner
//! kopiert und erst am Ende umbenannt: schlägt etwas fehl, bleibt nichts Halbes zurück.
use std::path::{Path, PathBuf};

/// Was mitkopiert wird
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Bauordner `build/` mitkopieren
    pub with_build: bool,
    /// Git-Ordner `.git/` mitkopieren (die Kopie hat dann dieselbe Versionsgeschichte)
    pub with_git: bool,
}

/// Ergebnis: neuer Projektordner, neue Projektdatei, Zahl und Größe der kopierten Dateien, Übersprungenes (zum Anzeigen)
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub new_root: PathBuf,
    pub project_file: PathBuf,
    pub files: usize,
    pub bytes: u64,
    pub skipped: Vec<String>,
}

const RESERVED: [&str; 22] = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];

/// Prüft und bereinigt den Namen des neuen Projekts (zugleich Ordner- und Dateiname, auf allen Systemen gültig)
pub fn validate_name(name: &str) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() { return Err(vc_i18n::t!("copyproject-der-name-ist-leer").into()); }
    if n.chars().count() > 100 { return Err(vc_i18n::t!("copyproject-der-name-ist-zu-lang-hoechstens").into()); }
    if let Some(c) = n.chars().find(|c| c.is_control() || "/\\:*?\"<>|".contains(*c)) { return Err(vc_i18n::t!("copyproject-das-zeichen-c-ist-im-namen", c = &c)); }
    if n.starts_with('.') { return Err(vc_i18n::t!("copyproject-der-name-darf-nicht-mit-einem").into()); }
    if n.ends_with('.') || n.ends_with(' ') { return Err(vc_i18n::t!("copyproject-der-name-darf-nicht-mit-punkt").into()); }
    let stem = n.split('.').next().unwrap_or(n).to_ascii_uppercase();
    if RESERVED.contains(&stem.as_str()) { return Err(vc_i18n::t!("copyproject-n-ist-unter-windows-ein-reservierter", n = &n)); }
    Ok(n.to_string())
}

/// Die Projektdatei im Ordner: die einzige `*.vcprj`, bei mehreren die erste in alphabetischer Reihenfolge
pub fn project_file_of(root: &Path) -> Result<PathBuf, String> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?
        .filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file() && p.extension().map_or(false, |x| x == "vcprj")).collect();
    v.sort();
    v.into_iter().next().ok_or_else(|| vc_i18n::t!("copyproject-im-ordner-p1-liegt-keine-projektdatei", p1 = root.display()))
}

fn skipped_name(rel: &Path, is_top_dir: bool, name: &str, opts: &Options) -> Option<String> {
    let _ = rel;
    if name == ".DS_Store" || name == "Thumbs.db" { return Some(String::new()); }   // Systemmüll: stumm überspringen
    if is_top_dir && name == "build" && !opts.with_build { return Some(vc_i18n::t!("copyproject-build-bauordner-laesst-sich-neu-erzeugen").into()); }
    if is_top_dir && name == ".git" && !opts.with_git { return Some(format!(".git/ ({})", vc_i18n::t!("copyproject-versionsgeschichte-die-kopie-beginnt-ohne"))); }
    None
}

/// Zahl und Größe der Dateien, die kopiert würden (für die Anzeige im Dialog)
pub fn plan(root: &Path, opts: &Options) -> Result<(usize, u64), String> {
    let mut files = 0usize; let mut bytes = 0u64;
    walk(root, root, opts, &mut |_, size| { files += 1; bytes += size; Ok(()) }, &mut |_, _| Ok(()), &mut vec![])?;
    Ok((files, bytes))
}

/// Läuft durch den Ordner: `on_file(relativer Pfad, Größe)`, `on_dir(relativer Pfad)`; Übersprungenes landet in `skipped`
fn walk(root: &Path, dir: &Path, opts: &Options, on_file: &mut dyn FnMut(&Path, u64) -> Result<(), String>, on_dir: &mut dyn FnMut(&Path, bool) -> Result<(), String>, skipped: &mut Vec<String>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let name = e.file_name().to_string_lossy().into_owned();
        let top = dir == root;
        if let Some(why) = skipped_name(&rel, top, &name, opts) { if !why.is_empty() { skipped.push(why); } continue; }
        let ft = e.file_type().map_err(|e| format!("{}: {e}", path.display()))?;
        if ft.is_symlink() { skipped.push(vc_i18n::t!("copyproject-p1-symbolische-verknuepfung", p1 = rel.display())); continue; }
        if ft.is_dir() { on_dir(&rel, true)?; walk(root, &path, opts, on_file, on_dir, skipped)?; }
        else if ft.is_file() { let size = e.metadata().map(|m| m.len()).unwrap_or(0); on_file(&rel, size)?; }
        else { skipped.push(vc_i18n::t!("copyproject-p1-keine-gewoehnliche-datei", p1 = rel.display())); }
    }
    Ok(())
}

/// Kopiert das Projekt in `parent/name`. Fehler, wenn das Ziel schon belegt ist oder im Quellordner läge.
pub fn copy_project(root: &Path, parent: &Path, name: &str, opts: &Options) -> Result<Report, String> {
    let name = validate_name(name)?;
    let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
    let old_file = project_file_of(&root)?;
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let parent = parent.canonicalize().map_err(|e| format!("{}: {e}", parent.display()))?;
    let dest = parent.join(&name);
    if dest == root || dest.starts_with(&root) { return Err(vc_i18n::t!("copyproject-das-ziel-liegt-im-quellordner-des").into()); }
    if root.starts_with(&dest) { return Err(vc_i18n::t!("copyproject-das-ziel-enthaelt-das-quellprojekt-waehle").into()); }
    if dest.exists() && !(dest.is_dir() && std::fs::read_dir(&dest).map_or(false, |mut d| d.next().is_none())) {
        return Err(vc_i18n::t!("copyproject-p1-gibt-es-schon-und-ist", p1 = dest.display()));
    }
    let tmp = parent.join(format!(".{name}.kopie-tmp"));
    if tmp.exists() { let _ = std::fs::remove_dir_all(&tmp); }
    std::fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let result = (|| -> Result<Report, String> {
        let (mut files, mut bytes) = (0usize, 0u64);
        let mut skipped = vec![];
        walk(&root, &root, opts, &mut |rel, size| {
            let to = tmp.join(rel);
            if let Some(d) = to.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
            std::fs::copy(root.join(rel), &to).map_err(|e| format!("{}: {e}", rel.display()))?;
            files += 1; bytes += size; Ok(())
        }, &mut |rel, _| std::fs::create_dir_all(tmp.join(rel)).map_err(|e| format!("{}: {e}", rel.display())), &mut skipped)?;
        // Projektdatei: neuer Name (Datei und project.name); übrige Felder bleiben unverändert
        let old_rel = old_file.strip_prefix(&root).map_err(|e| e.to_string())?.to_path_buf();
        let copied_old = tmp.join(&old_rel);
        let new_file_tmp = tmp.join(format!("{name}.vcprj"));
        let text = std::fs::read_to_string(&copied_old).map_err(|e| format!("{}: {e}", old_rel.display()))?;
        let mut v: serde_json::Value = serde_json::from_str(&text).map_err(|e| vc_i18n::t!("copyproject-p1-keine-gueltige-projektdatei-e", p1 = old_rel.display(), e = e.to_string()))?;
        if !v.get("project").map_or(false, |p| p.is_object()) { v["project"] = serde_json::json!({}); }
        v["project"]["name"] = serde_json::Value::String(name.clone());
        let mut out = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?; out.push('\n');
        if copied_old != new_file_tmp { std::fs::remove_file(&copied_old).map_err(|e| e.to_string())?; }
        std::fs::write(&new_file_tmp, out).map_err(|e| format!("{}: {e}", new_file_tmp.display()))?;
        if dest.exists() { std::fs::remove_dir(&dest).map_err(|e| format!("{}: {e}", dest.display()))?; }
        std::fs::rename(&tmp, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        Ok(Report { project_file: dest.join(format!("{name}.vcprj")), new_root: dest.clone(), files, bytes, skipped })
    })();
    if result.is_err() { let _ = std::fs::remove_dir_all(&tmp); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc_copy_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
    }

    /// Beispielprojekt mit Quelltext, Medium, Bauordner, Git-Ordner und Systemmüll
    fn sample(root: &Path) {
        std::fs::create_dir_all(root.join("src/lib")).unwrap(); std::fs::create_dir_all(root.join("build")).unwrap(); std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join("Alt.vcprj"), "{\n  \"format\": 1,\n  \"project\": {\"name\": \"Alt\", \"description\": \"Beschreibung\", \"eigen\": 7},\n  \"main\": \"src/a.asm\"\n}\n").unwrap();
        std::fs::write(root.join("src/a.asm"), "lda #1\n").unwrap(); std::fs::write(root.join("src/lib/b.inc"), "; b\n").unwrap();
        std::fs::write(root.join("hero.vcsprite"), b"\x00\x01\x02").unwrap();
        std::fs::write(root.join("build/a.prg"), b"prg").unwrap(); std::fs::write(root.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(root.join(".DS_Store"), b"x").unwrap();
    }

    #[test]
    fn namen_werden_geprueft() {
        assert_eq!(validate_name("  Mein Spiel 2 ").unwrap(), "Mein Spiel 2");
        for bad in ["", "   ", "a/b", "a\\b", "a:b", "x*", "x?", "<x>", "pipe|", ".versteckt", "ende.", "CON", "con.txt", "Lpt3", "zu\u{0007}lang"] { assert!(validate_name(bad).is_err(), "{bad:?} muss abgelehnt werden"); }
        assert_eq!(validate_name("ende ").unwrap(), "ende", "Leerzeichen am Rand werden entfernt");
        assert!(validate_name(&"x".repeat(101)).is_err());
        assert!(validate_name("Über Größe ä").is_ok(), "Umlaute sind erlaubt");
    }

    #[test]
    fn kopie_ohne_bauordner_und_git_neuer_projektname() {
        let base = tmp("a"); let src = base.join("Alt"); std::fs::create_dir_all(&src).unwrap(); sample(&src);
        let (n, b) = plan(&src, &Options::default()).unwrap(); assert_eq!(n, 4, "Projektdatei, a.asm, b.inc, hero.vcsprite"); assert!(b > 0);
        let r = copy_project(&src, &base.join("Ziel"), "Neu", &Options::default()).unwrap();
        assert_eq!(r.new_root, base.join("Ziel").canonicalize().unwrap().join("Neu"));
        assert_eq!(r.files, 4);
        assert!(r.project_file.is_file() && r.project_file.file_name().unwrap() == "Neu.vcprj");
        assert!(!r.new_root.join("Alt.vcprj").exists(), "alter Dateiname entfällt");
        for rel in ["src/a.asm", "src/lib/b.inc", "hero.vcsprite"] { assert!(r.new_root.join(rel).is_file(), "{rel}"); }
        assert_eq!(std::fs::read(r.new_root.join("hero.vcsprite")).unwrap(), b"\x00\x01\x02", "Binärdaten unverändert");
        assert!(!r.new_root.join("build").exists() && !r.new_root.join(".git").exists() && !r.new_root.join(".DS_Store").exists());
        assert!(r.skipped.iter().any(|s| s.starts_with("build/")) && r.skipped.iter().any(|s| s.starts_with(".git/")), "{:?}", r.skipped);
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&r.project_file).unwrap()).unwrap();
        assert_eq!(v["project"]["name"], "Neu"); assert_eq!(v["project"]["description"], "Beschreibung", "andere Felder bleiben");
        assert_eq!(v["project"]["eigen"], 7); assert_eq!(v["main"], "src/a.asm");
        assert!(src.join("Alt.vcprj").is_file() && src.join("build/a.prg").is_file(), "das Original bleibt unberührt");
        assert!(!base.join("Ziel").join(".Neu.kopie-tmp").exists(), "kein Zwischenordner zurück");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn mit_bauordner_und_git_auf_wunsch() {
        let base = tmp("b"); let src = base.join("Alt"); std::fs::create_dir_all(&src).unwrap(); sample(&src);
        let r = copy_project(&src, &base, "Voll", &Options { with_build: true, with_git: true }).unwrap();
        assert!(r.new_root.join("build/a.prg").is_file() && r.new_root.join(".git/HEAD").is_file());
        assert_eq!(r.files, 6, "vier Projektdateien plus build/a.prg und .git/HEAD");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn ziel_belegt_oder_im_quellordner_wird_abgelehnt_ohne_reste() {
        let base = tmp("c"); let src = base.join("Alt"); std::fs::create_dir_all(&src).unwrap(); sample(&src);
        std::fs::create_dir_all(base.join("Belegt")).unwrap(); std::fs::write(base.join("Belegt/x.txt"), "x").unwrap();
        let e = copy_project(&src, &base, "Belegt", &Options::default()).unwrap_err(); assert!(e.contains("nicht leer"), "{e}");
        let e = copy_project(&src, &src, "Innen", &Options::default()).unwrap_err(); assert!(e.contains("im Quellordner"), "{e}");
        let e = copy_project(&src, &base, "Alt", &Options::default()).unwrap_err(); assert!(e.contains("im Quellordner") || e.contains("nicht leer"), "{e}");
        assert!(copy_project(&src, &base, "a/b", &Options::default()).is_err());
        assert!(!src.join("Innen").exists(), "nichts im Quellordner angelegt");
        // leeres Ziel ist erlaubt
        std::fs::create_dir_all(base.join("Leer")).unwrap();
        assert!(copy_project(&src, &base, "Leer", &Options::default()).is_ok());
        // ohne Projektdatei
        let nop = base.join("OhneProjekt"); std::fs::create_dir_all(&nop).unwrap(); std::fs::write(nop.join("a.txt"), "a").unwrap();
        assert!(copy_project(&nop, &base, "X", &Options::default()).unwrap_err().contains("keine Projektdatei"));
        let entries: Vec<String> = std::fs::read_dir(&base).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert!(!entries.iter().any(|n| n.contains("kopie-tmp")), "keine Zwischenordner: {entries:?}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn symbolische_verknuepfungen_werden_uebersprungen_und_gemeldet() {
        let base = tmp("d"); let src = base.join("Alt"); std::fs::create_dir_all(&src).unwrap(); sample(&src);
        std::os::unix::fs::symlink("/etc", src.join("etc-link")).unwrap();
        let r = copy_project(&src, &base, "Kopie", &Options::default()).unwrap();
        assert!(!r.new_root.join("etc-link").exists());
        assert!(r.skipped.iter().any(|s| s.contains("etc-link") && s.contains("Verknüpfung")), "{:?}", r.skipped);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn kaputte_projektdatei_laesst_keinen_zwischenordner() {
        let base = tmp("e"); let src = base.join("Alt"); std::fs::create_dir_all(&src).unwrap(); sample(&src);
        std::fs::write(src.join("Alt.vcprj"), "kein json").unwrap();
        let e = copy_project(&src, &base, "Neu", &Options::default()).unwrap_err(); assert!(e.contains("keine gültige Projektdatei"), "{e}");
        assert!(!base.join("Neu").exists());
        assert!(std::fs::read_dir(&base).unwrap().filter_map(|e| e.ok()).all(|e| !e.file_name().to_string_lossy().contains("kopie-tmp")));
        let _ = std::fs::remove_dir_all(&base);
    }
}
