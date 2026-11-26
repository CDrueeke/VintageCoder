// SPDX-License-Identifier: GPL-3.0-or-later
//! Turbo BASIC XL starten (Konzept „Turbo BASIC XL starten“, Etappen E0–E2): Turbo BASIC XL läuft auf dem Atari von einer Diskette, die der
//! Anwender selbst besitzt (wir liefern keine aus). Das Projekt bekommt eine **Kopie** dieser Diskette (`turbo/<Originalname>.atr`); beim Bauen
//! entsteht daraus `build/<name>.atr` mit dem übersetzten Programm als `AUTORUN.BAS`, das Turbo BASIC XL nach dem Boot selbst lädt.
//!
//! * [`check_bytes`]/[`check_file`]: Prüfung eines Abbilds („ist das eine Turbo-Diskette?“, Version, freie Sektoren, SHA-256 gegen die Liste bekannter Abbilder).
//! * [`provide`]: Original in `turbo/` kopieren (das Original bleibt unberührt), `.gitignore`, Eintrag in der Projektdatei, auf Wunsch [`reduce`].
//! * [`reduce`]: Kopie auf `DOS.SYS`, `DUP.SYS`, `AUTORUN.SYS` verkleinern (T2: 482 statt 103 freie Sektoren) – nur in der Projektkopie.
//! * [`make_turbo_disk`]: Bau der Diskette (Kopie + Programm + `files`), mit Platzprüfung; [`start`]: Bauen und im Emulator booten.
//!
//! Alle Meldungen stehen im Katalog (`catalog/*/turbo.ftl`).
use crate::projfile::{ProjectFile, TurboSpec};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_disk::{Atr, Dos2Entry};

/// Name des Programms auf der Diskette, den Turbo BASIC XL nach dem Boot selbst lädt
pub const PROGRAM: &str = "AUTORUN.BAS";
/// Dateien, die zum Starten genügen (Messung Anhang D des Konzepts); `AUTORUN.SYS` enthält Turbo BASIC XL
pub const SYSTEM_FILES: [&str; 3] = ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS"];
/// Ordner der Kopie im Projekt und Eintrag der `.gitignore`
pub const DIR: &str = "turbo";
pub const GITIGNORE_LINE: &str = "turbo/*.atr";
const SIGNATURE: &[u8] = b"TURBO-BASIC XL";

/// Bekanntes Abbild (Liste `data/turbo/turbo-basic-known.json`, eingebettet)
#[derive(Clone, Debug, PartialEq)]
pub struct Known { pub id: String, pub label: String, pub version: String, pub size: usize, pub sha256: String }

/// Die Liste der bekannten Abbilder
pub fn known_list() -> Vec<Known> {
    let v: Value = serde_json::from_str(include_str!("../../../data/turbo/turbo-basic-known.json")).unwrap_or(Value::Null);
    v["disks"].as_array().into_iter().flatten().map(|d| Known {
        id: d["id"].as_str().unwrap_or("").into(), label: d["label"].as_str().unwrap_or("").into(), version: d["version"].as_str().unwrap_or("").into(),
        size: d["size"].as_u64().unwrap_or(0) as usize, sha256: d["sha256"].as_str().unwrap_or("").to_ascii_lowercase(),
    }).collect()
}

pub fn sha256_hex(data: &[u8]) -> String { vc_emu::install::sha256_hex(data) }

/// Ergebnis der Prüfung eines Abbilds (alles auf den Bytes, ohne Emulator)
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub sha256: String,
    pub size: usize,
    pub known: Option<Known>,
    /// DOS-2-Dateisystem vorhanden
    pub dos2: bool,
    /// `AUTORUN.SYS` mit der Kennung „TURBO-BASIC XL“
    pub is_turbo: bool,
    /// Version aus dem Text von `AUTORUN.SYS` (z. B. `1.5`)
    pub version: Option<String>,
    pub has_program: bool,
    pub program_locked: bool,
    pub free_sectors: usize,
    pub entries: Vec<Dos2Entry>,
    /// Fehler der Prüfung (kaputte Ketten, VTOC)
    pub problems: Vec<String>,
    /// Hinweise auf Altlasten, die nicht stören (nie abgeschlossene Einträge, Sektoren ohne Datei)
    pub hints: Vec<String>,
}

fn kb(sectors: usize) -> String { format!("{:.1}", sectors as f64 * 125.0 / 1000.0) }

impl Report {
    /// Lesbare Zeilen für Kommandozeile und Protokoll (Katalog)
    pub fn lines(&self) -> Vec<String> {
        let mut v = vec![];
        match (&self.known, self.is_turbo) {
            (Some(k), _) => v.push(vc_i18n::t!("turbo-erkannt-label", label = &k.label)),
            (None, true) => v.push(vc_i18n::t!("turbo-unbekanntes-abbild-version", version = self.version.as_deref().unwrap_or("?"))),
            (None, false) => v.push(vc_i18n::t!("turbo-nicht-erkannt")),
        }
        v.push(vc_i18n::t!("turbo-sektoren-frei", count = self.free_sectors, kb = kb(self.free_sectors)));
        v.push(vc_i18n::t!("turbo-pruefsumme", sha = &self.sha256));
        if self.dos2 {
            let names: Vec<String> = self.entries.iter().map(|e| if e.locked { format!("{}*", e.name) } else { e.name.clone() }).collect();
            v.push(vc_i18n::t!("turbo-dateien", count = self.entries.len(), names = names.join(", ")));
            if self.has_program { v.push(if self.program_locked { vc_i18n::t!("turbo-programm-vorhanden-gesperrt", name = PROGRAM) } else { vc_i18n::t!("turbo-programm-vorhanden", name = PROGRAM) }); }
        } else { v.push(vc_i18n::t!("turbo-kein-dos2")); }
        for m in vc_disk::summarize_problems(&self.problems, 10) { v.push(vc_i18n::t!("turbo-problem", m = &m)); }
        for m in vc_disk::summarize_problems(&self.hints, 10) { v.push(vc_i18n::t!("turbo-hinweis", m = &m)); }
        v
    }
    /// Für `--json`
    pub fn to_json(&self) -> Value {
        json!({ "sha256": self.sha256, "size": self.size, "known": self.known.as_ref().map(|k| k.id.clone()), "label": self.known.as_ref().map(|k| k.label.clone()),
                "dos2": self.dos2, "turbo": self.is_turbo, "version": self.version, "program_present": self.has_program, "free_sectors": self.free_sectors,
                "files": self.entries.iter().map(|e| json!({ "name": e.name, "sectors": e.sectors, "locked": e.locked })).collect::<Vec<_>>(), "problems": self.problems, "hints": self.hints })
    }
}

/// Version hinter der Kennung in `AUTORUN.SYS` („TURBO-BASIC XL 1.5“)
fn version_in(data: &[u8]) -> Option<String> {
    let at = data.windows(SIGNATURE.len()).position(|w| w == SIGNATURE)? + SIGNATURE.len();
    let v: String = data[at..].iter().skip_while(|b| **b == b' ').take_while(|b| b.is_ascii_digit() || **b == b'.').map(|b| *b as char).collect();
    let v = v.trim_end_matches('.').to_string();
    (!v.is_empty()).then_some(v)
}

/// Prüft die Bytes eines ATR-Abbilds. Fehler nur, wenn es gar kein lesbares 90-KB-ATR ist (Meldung des Containers, z. B. „noch nicht unterstützt“).
pub fn check_bytes(bytes: &[u8]) -> Result<Report, String> {
    let atr = Atr::from_bytes(bytes)?;
    let sha = sha256_hex(bytes);
    let known = known_list().into_iter().find(|k| k.sha256 == sha);
    let dos2 = atr.has_dos2();
    let (errors, hints) = atr.dos2_check_split();
    let entries = if dos2 { atr.dos2_entries() } else { vec![] };
    let autorun = if dos2 { atr.dos2_read("AUTORUN.SYS").ok() } else { None };
    let is_turbo = autorun.as_ref().map_or(false, |a| a.windows(SIGNATURE.len()).any(|w| w == SIGNATURE));
    let version = autorun.as_deref().filter(|_| is_turbo).and_then(version_in).or_else(|| known.as_ref().map(|k| k.version.clone()));
    Ok(Report { sha256: sha, size: bytes.len(), known, dos2, is_turbo, version, has_program: atr.dos2_is_locked(PROGRAM).is_some(),
                program_locked: atr.dos2_is_locked(PROGRAM) == Some(true), free_sectors: if dos2 { atr.free_sectors() } else { 0 }, entries, problems: errors, hints })
}

pub fn check_file(path: &Path) -> Result<Report, String> {
    check_bytes(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
}

/// Dateien, die [`reduce`] entfernen würde
pub fn reducible(atr: &Atr) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for e in atr.dos2_entries() { if !SYSTEM_FILES.contains(&e.name.as_str()) && !v.contains(&e.name) { v.push(e.name); } }
    v
}

/// Verkleinert die Kopie auf `DOS.SYS`, `DUP.SYS`, `AUTORUN.SYS` (auch geschützte Dateien werden entfernt). Fehlt eine der drei, wird nichts getan.
/// Gibt die entfernten Namen zurück.
pub fn reduce(atr: &mut Atr) -> Result<Vec<String>, String> {
    if !atr.has_dos2() { return Err(vc_i18n::t!("turbo-kein-dos2").into()); }
    for f in SYSTEM_FILES { if atr.dos2_is_locked(f).is_none() { return Err(vc_i18n::t!("turbo-reduzieren-systemdatei-fehlt", name = f)); } }
    let mut work = atr.clone();
    let names = reducible(&work);
    for n in &names { work.dos2_remove(n, true)?; }
    let p = work.dos2_check_split().0;
    if !p.is_empty() { return Err(vc_i18n::t!("turbo-reduzieren-abbild-fehlerhaft", m = p[0].as_str())); }
    *atr = work;
    Ok(names)
}

/// Vorschau für die Oberfläche (Bestätigung vor [`reduce`]): die Namen, die entfallen würden, und die freien Sektoren danach (`None`, wenn sich das
/// Abbild nicht verkleinern lässt, z. B. weil eine Systemdatei fehlt). Verändert nichts.
pub fn reduce_preview(bytes: &[u8]) -> Option<(Vec<String>, usize)> {
    let mut atr = Atr::from_bytes(bytes).ok()?;
    let names = reduce(&mut atr).ok()?;
    Some((names, atr.free_sectors()))
}

/// Eingaben von [`provide`]
#[derive(Clone, Debug, Default)]
pub struct ProvideOpts {
    /// Kopie auf die drei Systemdateien verkleinern (vorher vom Anwender bestätigt, T2)
    pub reduce: bool,
    /// Auch ein Abbild ohne Turbo-Kennung oder mit Fehlern verwenden (T4: auf eigene Verantwortung)
    pub anyway: bool,
    /// Eine vorhandene Kopie gleichen Namens überschreiben
    pub replace: bool,
}

#[derive(Clone, Debug)]
pub struct Provided {
    /// Pfad der Kopie relativ zum Projekt (`turbo/<Name>.atr`)
    pub disk: String,
    /// Prüfung des Originals
    pub report: Report,
    /// Bei [`ProvideOpts::reduce`]: entfernte Dateien
    pub removed: Vec<String>,
    pub free_sectors: usize,
    pub sha256: String,
    pub gitignore_added: bool,
}

/// Trägt `turbo/*.atr` in die `.gitignore` des Projekts ein (legt sie bei Bedarf an); liefert, ob etwas geschrieben wurde
pub fn ensure_gitignore(root: &Path) -> Result<bool, String> {
    let f = root.join(".gitignore");
    let old = std::fs::read_to_string(&f).unwrap_or_default();
    if old.lines().any(|l| l.trim() == GITIGNORE_LINE || l.trim() == format!("/{GITIGNORE_LINE}")) { return Ok(false); }
    let mut t = old;
    if !t.is_empty() && !t.ends_with('\n') { t.push('\n'); }
    t.push_str(&format!("# {}\n{GITIGNORE_LINE}\n", vc_i18n::t!("turbo-gitignore-kommentar")));
    vc_core::atomar::schreiben(&f, t).map_err(|e| format!("{}: {e}", f.display()))?;
    Ok(true)
}

/// Stellt die Diskette des Anwenders bereit: prüft sie, kopiert sie nach `turbo/` (Name wie das Original), trägt `turbo` in der Projektdatei und
/// `turbo/*.atr` in der `.gitignore` ein. Das Original wird nur gelesen. Mit `reduce` ist die Kopie danach auf die Systemdateien verkleinert.
pub fn provide(root: &Path, src: &Path, opts: &ProvideOpts) -> Result<Provided, String> {
    let pfile = crate::projfile::find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let mut pf = ProjectFile::load(&pfile)?;
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let report = check_bytes(&bytes)?;
    if !opts.anyway {
        if !report.dos2 || !report.is_turbo { return Err(vc_i18n::t!("turbo-nicht-erkannt-trotzdem").into()); }
        if let Some(p) = report.problems.first() { return Err(vc_i18n::t!("turbo-bereitstellen-fehler-in-der-diskette", m = p.as_str())); }
    }
    let name = src.file_name().map(|n| n.to_string_lossy().into_owned()).ok_or_else(|| vc_i18n::t!("turbo-ungueltiger-pfad", path = src.display().to_string()))?;
    let rel = format!("{DIR}/{name}");
    let dest = root.join(&rel);
    if let (Ok(a), Ok(b)) = (std::fs::canonicalize(src), std::fs::canonicalize(&dest)) { if a == b { return Err(vc_i18n::t!("turbo-quelle-ist-die-kopie", disk = &rel).into()); } }
    if dest.exists() && !opts.replace { return Err(vc_i18n::t!("turbo-kopie-existiert-bereits", disk = &rel)); }
    let mut out = bytes.clone();
    let mut removed = vec![];
    if opts.reduce {
        let mut atr = Atr::from_bytes(&bytes)?;
        removed = reduce(&mut atr)?;
        out = atr.to_bytes();
    }
    std::fs::create_dir_all(root.join(DIR)).map_err(|e| e.to_string())?;
    vc_core::atomar::schreiben(&dest, &out).map_err(|e| format!("{}: {e}", dest.display()))?;
    let sha = sha256_hex(&out);
    let free = Atr::from_bytes(&out)?.free_sectors();
    let t = pf.turbo.get_or_insert_with(TurboSpec::default);
    t.disk = rel.clone(); t.sha256 = sha.clone(); t.reduced = opts.reduce;
    pf.save(&pfile)?;
    let gitignore_added = ensure_gitignore(root)?;
    Ok(Provided { disk: rel, report, removed, free_sectors: free, sha256: sha, gitignore_added })
}

/// Verkleinert die bereitgestellte Kopie des Projekts (T2, nur in der Projektkopie); liefert die entfernten Dateien
pub fn reduce_project_copy(root: &Path) -> Result<Vec<String>, String> {
    let pfile = crate::projfile::find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let mut pf = ProjectFile::load(&pfile)?;
    let t = pf.turbo.as_mut().filter(|t| !t.disk.is_empty()).ok_or_else(|| vc_i18n::t!("turbo-diskette-fehlt").to_string())?;
    let path = crate::safepath::resolve(root, &t.disk)?;
    let mut atr = Atr::from_bytes(&std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?)?;
    let removed = reduce(&mut atr)?;
    let bytes = atr.to_bytes();
    vc_core::atomar::schreiben(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    t.sha256 = sha256_hex(&bytes); t.reduced = true;
    pf.save(&pfile)?;
    Ok(removed)
}

/// Ist das Hauptprogramm des Projekts ein Turbo-BASIC-XL-Programm für den Atari?
pub fn is_turbo_project(pf: &ProjectFile) -> bool {
    let Some(main) = pf.main_file() else { return false };
    let turbo = pf.dialect_of(&main).as_deref() == Some("turbo_basic_xl") || (pf.dialect_of(&main).is_none() && main.to_ascii_lowercase().ends_with(".tur"));
    turbo && pf.target_of(&main).map_or(true, |t| t == "atari8")
}

/// Grund, warum keine Turbo-Diskette gebaut werden konnte
#[derive(Clone, Debug, PartialEq)]
pub enum TurboErr {
    /// Die Diskette fehlt (Hinweis, kein Fehler des Programms; `vcc build --strict` macht daraus einen Fehler)
    Missing(String),
    Fail(String),
}
impl TurboErr { pub fn message(&self) -> &str { match self { TurboErr::Missing(m) | TurboErr::Fail(m) => m } } }

/// Ergebnis von [`make_turbo_disk`]
#[derive(Clone, Debug, PartialEq)]
pub struct Built {
    /// `build/<name>.atr`, relativ zum Projekt
    pub file: String,
    pub program: String,
    pub program_size: usize,
    pub version: Option<String>,
    pub free_sectors: usize,
    /// Hinweise (z. B. die Kopie wurde seit dem Bereitstellen verändert)
    pub notes: Vec<String>,
}
impl Built {
    pub fn to_json(&self) -> Value { json!({ "file": self.file, "program": self.program, "size": self.program_size, "version": self.version, "free_sectors": self.free_sectors, "notes": self.notes }) }
    pub fn line(&self) -> String {
        vc_i18n::t!("turbo-bau-diskette", file = &self.file, version = self.version.as_deref().unwrap_or("?"), program = &self.program, size = self.program_size, free = self.free_sectors)
    }
}

/// Zusatzdatei `Quelle[:NAME]` → (Quelle, Name auf der Diskette)
/// Pfadteil einer `files`-Angabe (`pfad` oder `pfad:NAME`)
pub fn spec_path(spec: &str) -> &str {
    // `C:\…` ist keine Namensangabe: ein Doppelpunkt zählt nur, wenn dahinter ein Name ohne Pfadtrenner steht
    match spec.rsplit_once(':') { Some((p, n)) if !n.is_empty() && !n.contains(['/', '\\']) && p.len() > 1 => p, _ => spec }
}

fn split_spec(spec: &str, root: &Path) -> Result<(PathBuf, String), String> {
    let p = spec_path(spec);
    let n = spec.get(p.len() + 1..).filter(|n| spec.len() > p.len() && !n.is_empty());
    let name = n.map(str::to_string).unwrap_or_else(|| Path::new(p).file_name().map(|f| f.to_string_lossy().to_ascii_uppercase()).unwrap_or_default());
    Ok((crate::safepath::resolve(root, p)?, name.to_ascii_uppercase()))
}

/// Prüft die Namen von `program` und `files` einer `turbo`-Angabe (für `validate` der Projektdatei)
pub fn spec_problems(t: &TurboSpec, root: &Path) -> (Vec<String>, Vec<String>) {
    let (mut errs, mut warns) = (vec![], vec![]);
    let prog = if t.program.is_empty() { PROGRAM.to_string() } else { t.program.to_ascii_uppercase() };
    let mut names = vec![prog.clone()];
    if !valid_name(&prog) { errs.push(vc_i18n::t!("turbo-ungueltiger-name", name = &prog)); }
    for f in &t.files {
        let (src, name) = match split_spec(f, root) { Ok(x) => x, Err(e) => { errs.push(e); continue; } };
        if !valid_name(&name) { errs.push(vc_i18n::t!("turbo-ungueltiger-name", name = &name)); }
        if names.contains(&name) { errs.push(vc_i18n::t!("turbo-name-doppelt", name = &name)); }
        names.push(name);
        if !src.is_file() { warns.push(vc_i18n::t!("turbo-zusatzdatei-fehlt", file = f.as_str())); }
    }
    (errs, warns)
}

/// Ist `name` ein gültiger DOS-2-Name (8.3, Großbuchstaben)?
pub fn valid_name(name: &str) -> bool { Atr::valid_dos2_name(name) }

/// Baut `build/<name>.atr`: Kopie der Diskette des Projekts, `AUTORUN.BAS` (bzw. `turbo.program`) ersetzt oder neu, danach die Dateien aus `turbo.files`.
/// `artifact`: das übersetzte Programm relativ zum Projekt (SAVE-Format); `source`: Quelldatei (bestimmt den Namen der Ausgabe).
pub fn make_turbo_disk(root: &Path, pf: &ProjectFile, source: &str, artifact: &str) -> Result<Built, TurboErr> {
    let spec = pf.turbo.clone().unwrap_or_default();
    if spec.disk.is_empty() { return Err(TurboErr::Missing(vc_i18n::t!("turbo-diskette-fehlt").to_string())); }
    let disk_path = crate::safepath::resolve(root, &spec.disk).map_err(TurboErr::Fail)?;
    if !disk_path.is_file() { return Err(TurboErr::Missing(vc_i18n::t!("turbo-diskette-datei-fehlt", disk = &spec.disk))); }
    let bytes = std::fs::read(&disk_path).map_err(|e| TurboErr::Fail(format!("{}: {e}", disk_path.display())))?;
    let mut atr = Atr::from_bytes(&bytes).map_err(TurboErr::Fail)?;
    let mut notes = vec![];
    if !spec.sha256.is_empty() && !sha256_hex(&bytes).eq_ignore_ascii_case(&spec.sha256) { notes.push(vc_i18n::t!("turbo-kopie-veraendert", disk = &spec.disk)); }
    let report = check_bytes(&bytes).map_err(TurboErr::Fail)?;
    if !report.is_turbo { notes.push(vc_i18n::t!("turbo-hinweis-nicht-erkannt", disk = &spec.disk)); }
    if let Some(p) = report.problems.first() { return Err(TurboErr::Fail(vc_i18n::t!("turbo-bauen-fehler-in-der-diskette", disk = &spec.disk, m = p.as_str()))); }
    let (errs, _) = spec_problems(&spec, root);
    if let Some(e) = errs.into_iter().next() { return Err(TurboErr::Fail(e)); }
    let program = if spec.program.is_empty() { PROGRAM.to_string() } else { spec.program.to_ascii_uppercase() };
    let prog_bytes = std::fs::read(crate::safepath::resolve(root, artifact).map_err(TurboErr::Fail)?).map_err(|e| TurboErr::Fail(format!("{artifact}: {e}")))?;
    let mut items: Vec<(String, Vec<u8>)> = vec![(program.clone(), prog_bytes.clone())];
    for f in &spec.files {
        let (src, name) = split_spec(f, root).map_err(TurboErr::Fail)?;
        items.push((name, std::fs::read(&src).map_err(|e| TurboErr::Fail(format!("{}: {e}", src.display())))?));
    }
    // Platz: neue Sektoren gegen freie plus die der Dateien, die ersetzt werden
    let need: usize = items.iter().map(|(_, d)| Atr::sectors_for(d.len())).sum();
    let reclaim: usize = items.iter().map(|(n, _)| atr.dos2_entries().iter().filter(|e| &e.name == n).map(|e| e.sectors as usize).sum::<usize>()).sum();
    for (n, _) in &items { if atr.dos2_is_locked(n) == Some(true) { return Err(TurboErr::Fail(vc_i18n::t!("turbo-datei-name-ist-schreibgeschuetzt", name = n.as_str()))); } }
    if need > atr.free_sectors() + reclaim {
        let have = atr.free_sectors() + reclaim;
        let mut m = vc_i18n::t!("turbo-platz", need = need, free = have);
        let mut small = atr.clone();
        let hint = if !spec.reduced && reduce(&mut small).is_ok() && need <= small.free_sectors() + items.iter().map(|(n, _)| small.dos2_entries().iter().filter(|e| &e.name == n).map(|e| e.sectors as usize).sum::<usize>()).sum::<usize>() {
            vc_i18n::t!("turbo-platz-hinweis-reduzieren", free = small.free_sectors())
        } else { vc_i18n::t!("turbo-platz-hinweis-groesseres-abbild") };
        m.push(' '); m.push_str(&hint);
        return Err(TurboErr::Fail(m));
    }
    for (n, d) in &items { atr.dos2_put(n, d).map_err(TurboErr::Fail)?; }
    let p = atr.dos2_check_split().0;
    if let Some(m) = p.first() { return Err(TurboErr::Fail(vc_i18n::t!("turbo-bauen-fehler-in-der-diskette", disk = &spec.disk, m = m.as_str()))); }
    let stem = Path::new(source).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "turbo".into());
    let file = format!("build/{stem}.atr");
    let out = root.join(&file);
    if let Some(d) = out.parent() { std::fs::create_dir_all(d).map_err(|e| TurboErr::Fail(e.to_string()))?; }
    vc_core::atomar::schreiben(&out, atr.to_bytes()).map_err(|e| TurboErr::Fail(format!("{file}: {e}")))?;
    Ok(Built { file, program, program_size: prog_bytes.len(), version: report.version, free_sectors: atr.free_sectors(), notes })
}

/// Ergebnis von [`start`]
#[derive(Clone, Debug)]
pub struct Started { pub disk: PathBuf, pub command: String, pub lines: Vec<String> }

/// Baut das Hauptprogramm des Projekts mit der Turbo-Diskette und bootet `build/<name>.atr` im Atari-Emulator (Laufwerk 1, ohne eingebautes BASIC;
/// atari800, unter Windows auf Wunsch Altirra). Fehlt die Diskette, kommt die Meldung „bereitstellen“.
pub fn start(root: &Path, data: &Path) -> Result<Started, String> {
    let root = &root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let pfile = crate::projfile::find(root).ok_or_else(|| vc_i18n::t!("turbo-keine-projektdatei", path = root.display().to_string()))?;
    let pf = ProjectFile::load(&pfile)?;
    if !is_turbo_project(&pf) { return Err(vc_i18n::t!("turbo-kein-turbo-projekt").into()); }
    let main = pf.main_file().unwrap_or_default();
    let r = crate::projfile::build_project(&crate::build::Ctx { root, data }, &pf, Some(&main));
    if r["ok"] != true {
        let msg = r["turbo_error"].as_str().map(String::from).or_else(|| r["results"][0]["diagnostics"][0]["message"].as_str().map(String::from));
        return Err(msg.unwrap_or_else(|| vc_i18n::t!("turbo-bauen-fehlgeschlagen").to_string()));
    }
    if let Some(m) = r["turbo_missing"].as_str() { return Err(m.to_string()); }
    let file = r["turbo"]["file"].as_str().ok_or_else(|| vc_i18n::t!("turbo-bauen-fehlgeschlagen").to_string())?.to_string();
    let disk = root.join(&file);
    let mut lines = vec![];
    if let Some(l) = r["turbo"]["line"].as_str() { lines.push(l.to_string()); }
    for n in r["turbo"]["notes"].as_array().into_iter().flatten().filter_map(|n| n.as_str()) { lines.push(n.to_string()); }
    let command = crate::launch::insert_disk(&disk, "atari8", &crate::launch::atari_variant(root))?;
    lines.push(vc_i18n::t!("turbo-programmende-hinweis").to_string());
    Ok(Started { disk, command, lines })
}
