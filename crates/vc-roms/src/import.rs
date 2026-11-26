// SPDX-License-Identifier: GPL-3.0-or-later
//! ROM-Dateien in den ROM-Ordner übernehmen (Drag-and-drop, „ROMs aus VICE importieren“). Nichts wird ohne Bestätigung kopiert:
//! erst [`plan`] oder [`foreign_candidates`], dann [`execute`] für die bestätigten Einträge.
use crate::env::{self, Stage};
use crate::store::{Found, RomStore};

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// wird kopiert nach `dest`
    Copy,
    /// gleicher Inhalt liegt schon im ROM-Ordner
    AlreadyPresent(PathBuf),
    Skip(String),
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub src: PathBuf,
    pub system: Option<String>,
    pub role: Option<String>,
    /// Bezeichnung der erkannten Revision
    pub label: Option<String>,
    pub size: u64,
    pub crc32: Option<u32>,
    pub sha1: Option<String>,
    pub dest: Option<PathBuf>,
    pub action: Action,
    /// Herkunft („VICE 3.10“), nur bei fremden Fundorten
    pub source_label: Option<String>,
    /// Inhalt, wenn die Datei aus einem ZIP-Archiv stammt (`src` ist dann das Archiv)
    pub data: Option<Vec<u8>>,
}

/// Größere Archive werden nicht in den Speicher gelesen (ROM-Pakete sind klein, Core-Archive mit Bitstreams nicht)
const MAX_ZIP: u64 = 256 << 20;
/// Tiefe und Anzahl beim Einlesen von Ordnern (Ordner mit Unterordnern, z. B. ein VICE-Datenordner)
const MAX_DEPTH: usize = 2;
const MAX_FILES: usize = 2000;

fn collect_files(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut v: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
    v.sort();
    for p in v {
        if out.len() >= MAX_FILES { return; }
        if p.is_file() { out.push(p); } else if p.is_dir() && depth < MAX_DEPTH { collect_files(&p, depth + 1, out); }
    }
}

fn item_for(store: &RomStore, f: &Found, source_label: Option<String>) -> ImportItem {
    let mut it = ImportItem { src: f.path.clone(), system: None, role: None, label: None, size: f.size, crc32: f.crc32, sha1: f.sha1.clone(), dest: None, action: Action::Skip(vc_i18n::t!("romsio-import-not-recognized")), source_label, data: None };
    let Some((system, role)) = store.role_of(f) else {
        it.action = Action::Skip(match &f.ident {
            crate::Ident::Ambiguous { system, candidates } if system.is_empty() => vc_i18n::t!("romsio-import-checksum-unknown", candidates = candidates.join(", ")),
            crate::Ident::Ambiguous { candidates, .. } => vc_i18n::t!("romsio-import-role-ambiguous", candidates = candidates.join(", ")),
            crate::Ident::WrongSize { expected, .. } => vc_i18n::t!("romsio-import-wrong-size", sizes = expected.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(&vc_i18n::t!("romsio-import-or"))),
            crate::Ident::Unrecognized(r) => r.clone(),
            _ => vc_i18n::t!("romsio-import-not-recognized"),
        });
        return it;
    };
    if vc_roms_system_unknown(&system) { it.system = Some(system); it.role = Some(role); it.action = Action::Skip(vc_i18n::t!("romsio-import-unsupported-system")); return it; }
    it.label = store.known_entry(f).map(|k| k.label.clone());
    it.system = Some(system.clone());
    it.role = Some(role);
    // schon im eigenen Ordner (Projekt, portabel, Benutzer)? Fremde Fundorte und Mitgeliefertes zählen nicht.
    // und nur im Ordner desselben Systems: die C64-BASIC im C64-Ordner ersetzt nicht die im C128-Ordner
    let own = |g: &&Found| matches!(g.stage, Stage::Project | Stage::Portable | Stage::User | Stage::Legacy) && g.hint.as_deref().is_none_or(|h| h == system);
    if let Some(same) = store.found().iter().filter(own).find(|g| g.sha1.is_some() && g.sha1 == f.sha1) {
        it.action = Action::AlreadyPresent(same.path.clone());
        return it;
    }
    match store.env.install_dir() {
        Some(dir) => {
            if env::inside_app_bundle(&dir) { it.action = Action::Skip(vc_i18n::t!("romsio-import-in-app-bundle")); return it; }
            let name = f.file_name();
            it.dest = Some(unique_dest(&dir.join(&system), &name));
            it.action = Action::Copy;
        }
        None => it.action = Action::Skip(vc_i18n::t!("romsio-import-no-rom-dir")),
    }
    it
}

fn vc_roms_system_unknown(system: &str) -> bool { crate::systems::system(system).is_none() }

/// Zielname im Ordner: gleicher Name bleibt, wenn frei; sonst `name-2.ext` usw.
fn unique_dest(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    if !p.exists() { return p; }
    let (stem, ext) = match name.rsplit_once('.') { Some((s, e)) => (s.to_string(), format!(".{e}")), None => (name.to_string(), String::new()) };
    (2..).map(|i| dir.join(format!("{stem}-{i}{ext}"))).find(|p| !p.exists()).unwrap()
}

/// Der Hinweis der Oberfläche (aktiver Reiter) ist kein Zwang: kennt die Tabelle die Datei, aber nicht für dieses System (PET-Chip bei aktivem C64-Reiter),
/// gilt ihre erste Verwendung. Bei Ordnern (`roms/c64/…`) bleibt der Hinweis streng, siehe `RomStore::role_of`.
fn hint_nur_wenn_passend(store: &RomStore, mut f: Found) -> Found {
    if let (crate::Ident::Known(i), Some(h)) = (&f.ident, f.hint.as_deref()) {
        if !store.known.roms[*i].uses.iter().any(|u| u.system == h) { f.hint = None; }
    }
    f
}

/// Plan für Dateien oder Ordner (Drag-and-drop). `system` ist ein Hinweis des Nutzers (gewählter Reiter), kein Zwang.
pub fn plan(store: &RomStore, sources: &[PathBuf], system: Option<&str>) -> Vec<ImportItem> {
    let mut files = vec![];
    for s in sources {
        if s.is_dir() { collect_files(s, 0, &mut files); } else { files.push(s.clone()); }
    }
    let mut out = vec![];
    for p in &files {
        let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        if name.ends_with(".zip") { out.extend(plan_zip(store, p, system)); continue; }
        if p.metadata().is_ok_and(|m| store.known.combined.iter().any(|c| c.size as u64 == m.len())) {
            if let Some(items) = std::fs::read(p).ok().and_then(|d| combined_items(store, p, &name, &d, None)) { out.extend(items); continue; }
        }
        if let Some(f) = store.identify_file(p, system).map(|f| hint_nur_wenn_passend(store, f)) { out.push(item_for(store, &f, None)); }
    }
    dedupe(out)
}

/// Inhalt eines ZIP-Archivs (Drag-and-drop): nur erkannte ROM-Dateien werden angeboten, alles andere im Archiv bleibt unberührt
fn plan_zip(store: &RomStore, zip: &Path, system: Option<&str>) -> Vec<ImportItem> {
    let skip = |why: String| ImportItem { src: zip.to_path_buf(), system: None, role: None, label: None, size: zip.metadata().map(|m| m.len()).unwrap_or(0), crc32: None, sha1: None, dest: None, action: Action::Skip(why), source_label: None, data: None };
    let len = zip.metadata().map(|m| m.len()).unwrap_or(0);
    if len > MAX_ZIP { return vec![skip(vc_i18n::t!("romsio-import-zip-too-large", mb = len / (1 << 20)))]; }
    let bytes = match std::fs::read(zip) { Ok(b) => b, Err(e) => return vec![skip(format!("{}: {e}", zip.display()))] };
    let entries = match crate::zip::read(&bytes) { Ok(e) => e, Err(e) => return vec![skip(e)] };
    let mut out = vec![];
    let label = vc_i18n::t!("romsio-import-archive-label", name = zip.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
    for (name, data) in entries {
        if let Some(items) = combined_items(store, zip, &name, &data, Some(label.clone())) { out.extend(items); continue; }
        let Some(f) = store.identify_bytes(&name, &data, system).map(|f| hint_nur_wenn_passend(store, f)) else { continue };
        if store.role_of(&f).is_none() { continue; }
        let mut it = item_for(store, &f, Some(vc_i18n::t!("romsio-import-archive-label", name = zip.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())));
        it.src = zip.to_path_buf();
        it.data = Some(data);
        out.push(it);
    }
    if out.is_empty() { out.push(skip(vc_i18n::t!("romsio-import-zip-no-roms"))); }
    out
}

/// Kombiniertes Abbild (mehrere ROM-Chips in einer Datei, Tabelle `combined`): ein Eintrag je Teil. Die Teile stehen als Daten im Speicher
/// (wie Dateien aus einem ZIP-Archiv) und werden von [`execute`] als eigene Dateien geschrieben. Teile, die im Katalog stehen, tragen ihre Revision;
/// alle anderen gelten als unbekannte Revision der Rolle, die der Katalog für den Teil vorgibt.
fn combined_items(store: &RomStore, src: &Path, name: &str, data: &[u8], source_label: Option<String>) -> Option<Vec<ImportItem>> {
    let c = store.known.find_combined(data.len(), crate::hash::crc32(data), &crate::hash::sha1_hex(data))?;
    let base = Path::new(name).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string());
    let mut out = vec![];
    for part in &c.parts {
        let bytes = data.get(part.offset..part.offset + part.size)?;
        let sha1 = crate::hash::sha1_hex(bytes);
        let crc = crate::hash::crc32(bytes);
        let known = store.known.find(bytes.len(), crc, &sha1).and_then(|k| store.known.roms.iter().position(|x| std::ptr::eq(x, k)));
        let known = known.filter(|&i| store.known.roms[i].uses.iter().any(|u| u.system == part.system));
        let role_label = crate::systems::system(&part.system).and_then(|s| s.roles.iter().find(|r| r.id == part.role)).map(|r| r.label_l()).unwrap_or_else(|| part.role.clone());
        let found = Found { path: PathBuf::from(format!("{}-{base}", part.role)), stage: Stage::Explicit, place: 0, size: part.size as u64, crc32: Some(crc), sha1: Some(sha1), hint: Some(part.system.clone()),
            ident: match known { Some(i) => crate::Ident::Known(i), None => crate::Ident::UnknownRevision { system: part.system.clone(), role: part.role.clone() } }, pinned: false };
        let mut it = item_for(store, &found, source_label.clone());
        if known.is_none() && it.system.is_some() { it.label = Some(vc_i18n::t!("romsio-import-part-label", label = &c.label, role = role_label)); }
        it.src = src.to_path_buf();
        it.data = Some(bytes.to_vec());
        out.push(it);
    }
    Some(out)
}

/// Gleicher Inhalt zweimal im selben Plan (z. B. zwei Dateinamen für dieselbe ROM): nur der erste wird kopiert, der zweite gilt als vorhanden
fn dedupe(items: Vec<ImportItem>) -> Vec<ImportItem> {
    let mut out: Vec<ImportItem> = vec![];
    for mut it in items {
        if it.action == Action::Copy {
            if let Some(first) = out.iter().find(|o| o.action == Action::Copy && o.sha1 == it.sha1 && o.system == it.system) {
                it.action = Action::AlreadyPresent(first.dest.clone().unwrap_or_else(|| first.src.clone()));
                it.dest = None;
            }
        }
        out.push(it);
    }
    out
}

/// Erkannte ROMs (Prüfsumme in der Tabelle) in ROM-Ordnern anderer Programme (VICE …), unabhängig vom Schalter „ROMs anderer Programme mitbenutzen“.
/// Es wird nur gelesen; kopiert wird erst nach Bestätigung mit [`execute`]. Nicht erkannte Dateien erscheinen nicht.
pub fn foreign_candidates(store: &RomStore) -> Vec<ImportItem> { foreign_candidates_until(store, &|| false) }

/// Wie [`foreign_candidates`], bricht ab, sobald `stop()` wahr wird (Abbrechen, Zeitgrenze); bis dahin Gefundenes wird geliefert.
/// Gesucht wird nur in der kurzen Liste bekannter Orte ([`env::foreign_places`]), nie über ganze Laufwerke; je Ort höchstens eine Ordnerebene tief.
pub fn foreign_candidates_until(store: &RomStore, stop: &dyn Fn() -> bool) -> Vec<ImportItem> {
    let places: Vec<_> = env::foreign_places();
    let found = crate::store::scan_places_until(&store.known, &places, stop);
    let mut out = vec![];
    for f in &found {
        // nur Dateien mit bekannter Prüfsumme: Namen fremder Ordner (Landesvarianten, Prototypen) sind kein Beleg
        // Ausnahme MEGA65: genau eine Rolle und eine eigene Größe (131072 Byte), im Ordner „mega65“ (Xemu): `MEGA65.ROM` in allen Fassungen anbieten
        let mega65_rom = matches!(&f.ident, crate::Ident::UnknownRevision { system, .. } if system == "mega65");
        if (store.known_entry(f).is_none() && !mega65_rom) || store.role_of(f).is_none() { continue; }
        let it = item_for(store, f, Some(places[f.place].label.clone()));
        // doppelte Inhalte aus mehreren Fundorten nur einmal anbieten
        if it.sha1.is_some() && out.iter().any(|o: &ImportItem| o.sha1 == it.sha1 && o.system == it.system) { continue; }
        out.push(it);
    }
    out
}

/// Einen bestätigten Eintrag ausführen. Gibt den Zielpfad zurück.
pub fn execute(item: &ImportItem) -> Result<PathBuf, String> {
    match (&item.action, &item.dest) {
        (Action::Copy, Some(dest)) => {
            if env::inside_app_bundle(dest) { return Err(vc_i18n::t!("romsio-import-never-bundle")); }
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| format!("{}: {e}", dest.display()))?;
            match &item.data {
                Some(d) => std::fs::write(dest, d).map_err(|e| format!("{}: {e}", dest.display()))?,
                None => { std::fs::copy(&item.src, dest).map_err(|e| format!("{} → {}: {e}", item.src.display(), dest.display()))?; }
            }
            Ok(dest.clone())
        }
        (Action::AlreadyPresent(p), _) => Ok(p.clone()),
        (Action::Skip(why), _) => Err(why.clone()),
        _ => Err(vc_i18n::t!("romsio-import-no-target")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hash, known::*, RomEnv};

    #[test]
    fn plan_und_kopieren_ohne_ueberschreiben() {
        let base = std::env::temp_dir().join(format!("vc-roms-imp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (src, dst) = (base.join("src"), base.join("roms"));
        std::fs::create_dir_all(&src).unwrap();
        let data: Vec<u8> = (0..8192).map(|i| (i % 251) as u8).collect();
        let other: Vec<u8> = (0..8192).map(|i| (i % 241) as u8).collect();
        std::fs::write(src.join("k.bin"), &data).unwrap();
        std::fs::write(src.join("zz.bin"), &other).unwrap();   // nicht in der Tabelle
        std::fs::write(src.join("pack.zip"), b"PK").unwrap();
        let k = KnownRom { id: "c64.kernal.t".into(), label: "Test-KERNAL".into(), size: 8192, crc32: hash::crc32_hex(&data), sha1: hash::sha1_hex(&data), uses: vec![Use { system: "c64".into(), role: "kernal".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(dst.clone()), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![k.clone()] });
        let items = plan(&store, &[src.clone()], None);
        assert_eq!(items.len(), 3);
        let kern = items.iter().find(|i| i.src.ends_with("k.bin")).unwrap();
        assert_eq!(kern.action, Action::Copy);
        assert_eq!(kern.system.as_deref(), Some("c64"));
        let zz = items.iter().find(|i| i.src.ends_with("zz.bin")).unwrap();
        assert!(matches!(&zz.action, Action::Skip(w) if w.contains("Prüfsumme unbekannt")), "{:?}", zz.action);
        assert!(matches!(items.iter().find(|i| i.src.ends_with("pack.zip")).unwrap().action, Action::Skip(_)));
        let out = execute(kern).unwrap();
        assert_eq!(out, dst.join("c64/k.bin"));
        assert_eq!(std::fs::read(&out).unwrap(), data);
        // zweiter Lauf: schon vorhanden
        let store = RomStore::with_known(RomEnv { vc_roms: Some(dst), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![k] });
        let again = plan(&store, &[src.join("k.bin")], None);
        assert!(matches!(again[0].action, Action::AlreadyPresent(_)));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn zip_archiv_wird_ausgepackt_nur_erkanntes() {
        let base = std::env::temp_dir().join(format!("vc-roms-impzip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let data: Vec<u8> = (0..8192).map(|i| (i % 249) as u8).collect();
        // ZIP mit einer gespeicherten Datei „sub/k.bin“ und einer fremden Datei
        let mut z = vec![]; let mut c = vec![];
        for (name, d) in [("sub/k.bin", &data[..]), ("liesmich.txt", &b"hallo"[..])] {
            let off = z.len() as u32; let crc = hash::crc32(d);
            z.extend([0x50u8, 0x4B, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); z.extend(crc.to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((name.len() as u16).to_le_bytes()); z.extend([0, 0]); z.extend(name.as_bytes()); z.extend(d);
            c.extend([0x50u8, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); c.extend(crc.to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((name.len() as u16).to_le_bytes()); c.extend([0; 12]); c.extend(off.to_le_bytes()); c.extend(name.as_bytes());
        }
        let cd = z.len() as u32; z.extend(&c); z.extend([0x50u8, 0x4B, 5, 6, 0, 0, 0, 0, 2, 0, 2, 0]); z.extend((c.len() as u32).to_le_bytes()); z.extend(cd.to_le_bytes()); z.extend([0, 0]);
        std::fs::write(base.join("pack.zip"), &z).unwrap();
        let k = KnownRom { id: "c64.kernal.t".into(), label: "Test-KERNAL".into(), size: 8192, crc32: hash::crc32_hex(&data), sha1: hash::sha1_hex(&data), uses: vec![Use { system: "c64".into(), role: "kernal".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![k] });
        let items = plan(&store, &[base.join("pack.zip")], None);
        assert_eq!(items.len(), 1, "{items:?}");
        assert_eq!(items[0].action, Action::Copy);
        let out = execute(&items[0]).unwrap();
        assert_eq!(out, base.join("roms/c64/k.bin"));
        assert_eq!(std::fs::read(out).unwrap(), data);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn unterordner_dubletten_und_riesige_dateien() {
        let base = std::env::temp_dir().join(format!("vc-roms-imp3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let sub = base.join("src/Plus:4 sammlung/tief");
        std::fs::create_dir_all(&sub).unwrap();
        let data: Vec<u8> = (0..8192).map(|i| (i % 247) as u8).collect();
        std::fs::write(sub.join("kernal.a..bin"), &data).unwrap();   // doppelter Punkt im Namen
        std::fs::write(base.join("src/Plus:4 sammlung/kernal.b.bin"), &data).unwrap();   // gleicher Inhalt, anderer Name
        std::fs::write(base.join("src/.versteckt.bin"), &data).unwrap();
        let big = std::fs::File::create(base.join("src/riesig.7z")).unwrap();
        big.set_len(5 << 20).unwrap();
        let k = KnownRom { id: "c64.kernal.t".into(), label: "Test-KERNAL".into(), size: 8192, crc32: hash::crc32_hex(&data), sha1: hash::sha1_hex(&data), uses: vec![Use { system: "c64".into(), role: "kernal".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![k] });
        let items = plan(&store, &[base.join("src")], None);
        let copies: Vec<_> = items.iter().filter(|i| i.action == Action::Copy).collect();
        assert_eq!(copies.len(), 1, "{items:?}");
        assert_eq!(items.iter().filter(|i| matches!(i.action, Action::AlreadyPresent(_))).count(), 1);
        assert!(items.iter().all(|i| !i.src.to_string_lossy().contains(".versteckt")));
        assert!(items.iter().any(|i| i.src.ends_with("riesig.7z") && matches!(i.action, Action::Skip(_))));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// ZIP mit gespeicherten (unkomprimierten) Einträgen
    fn stored_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut z = vec![]; let mut c = vec![];
        for (name, d) in files.iter().copied() {
            let off = z.len() as u32; let crc = hash::crc32(d);
            z.extend([0x50u8, 0x4B, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); z.extend(crc.to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((name.len() as u16).to_le_bytes()); z.extend([0, 0]); z.extend(name.as_bytes()); z.extend(d);
            c.extend([0x50u8, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); c.extend(crc.to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((name.len() as u16).to_le_bytes()); c.extend([0; 12]); c.extend(off.to_le_bytes()); c.extend(name.as_bytes());
        }
        let n = files.len() as u16;
        let cd = z.len() as u32; z.extend(&c); z.extend([0x50u8, 0x4B, 5, 6, 0, 0, 0, 0]); z.extend(n.to_le_bytes()); z.extend(n.to_le_bytes()); z.extend((c.len() as u32).to_le_bytes()); z.extend(cd.to_le_bytes()); z.extend([0, 0]);
        z
    }

    /// Der Reiter der Oberfläche (z. B. C64) ist nur ein Hinweis: eine bekannte PET-Datei wird auch bei aktivem C64-Reiter erkannt, einzeln und im ZIP.
    #[test]
    fn systemhinweis_der_oberflaeche_verwirft_bekannte_dateien_anderer_systeme_nicht() {
        let base = std::env::temp_dir().join(format!("vc-roms-imp-hint-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        let ed: Vec<u8> = (0..2048).map(|i| (i % 241) as u8).collect();
        let kn: Vec<u8> = (0..8192).map(|i| (i % 239) as u8).collect();
        std::fs::write(base.join("src/edit-x.bin"), &ed).unwrap();
        std::fs::write(base.join("src/kernal-x.bin"), &kn).unwrap();
        let known = |id: &str, d: &[u8], sys: &str, role: &str| KnownRom { id: id.into(), label: id.into(), size: d.len(), crc32: hash::crc32_hex(d), sha1: hash::sha1_hex(d), uses: vec![Use { system: sys.into(), role: role.into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![known("pet.editor.t", &ed, "pet", "editor"), known("c64.kernal.t", &kn, "c64", "kernal")] });
        for hint in [None, Some("c64"), Some("vic20"), Some("pet")] {
            let items = plan(&store, &[base.join("src/edit-x.bin")], hint);
            assert_eq!(items.len(), 1, "{hint:?}");
            assert_eq!((items[0].system.as_deref(), items[0].role.as_deref(), &items[0].action), (Some("pet"), Some("editor"), &Action::Copy), "{hint:?}: {:?}", items[0]);
        }
        // passt der Hinweis zu einer Verwendung, bleibt es dabei
        let items = plan(&store, &[base.join("src/kernal-x.bin")], Some("c64"));
        assert_eq!(items[0].system.as_deref(), Some("c64"));
        // ZIP: gleiche Regel
        let zip = stored_zip(&[("edit-x.bin", &ed[..])]);
        std::fs::write(base.join("src/s.zip"), &zip).unwrap();
        let items = plan(&store, &[base.join("src/s.zip")], Some("c64"));
        assert!(items.iter().any(|i| i.system.as_deref() == Some("pet") && i.action == Action::Copy), "{items:?}");
        let _ = std::fs::remove_dir_all(&base);
    }

    fn fake(n: usize, seed: u8) -> Vec<u8> { (0..n).map(|i| (i as u8).wrapping_mul(37).wrapping_add(seed).wrapping_add((i >> 8) as u8)).collect() }

    fn combined_of(id: &str, data: &[u8], parts: Vec<(usize, usize, &str, &str)>) -> Combined {
        Combined { id: id.into(), label: "Test-Abbild".into(), size: data.len(), crc32: hash::crc32_hex(data), sha1: hash::sha1_hex(data), note: String::new(),
            parts: parts.into_iter().map(|(offset, size, s, r)| Part { offset, size, system: s.into(), role: r.into() }).collect() }
    }

    #[test]
    fn kombiniertes_abbild_wird_in_teile_geteilt() {
        let base = std::env::temp_dir().join(format!("vc-roms-comb-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        let (basic, kernal) = (fake(8192, 1), fake(8192, 2));
        let both: Vec<u8> = basic.iter().chain(&kernal).copied().collect();
        std::fs::write(base.join("src/c64c.bin"), &both).unwrap();
        // gleiche Größe, anderer Inhalt: nicht im Katalog
        std::fs::write(base.join("src/fremd.bin"), fake(16384, 9)).unwrap();
        // KERNAL-Teil steht schon als Revision im Katalog, BASIC nicht
        let k = KnownRom { id: "c64.kernal.t".into(), label: "Test-KERNAL".into(), size: 8192, crc32: hash::crc32_hex(&kernal), sha1: hash::sha1_hex(&kernal), uses: vec![Use { system: "c64".into(), role: "kernal".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let table = KnownTable { format: 1, roms: vec![k], combined: vec![combined_of("combined.t", &both, vec![(0, 8192, "c64", "basic"), (8192, 8192, "c64", "kernal")])] };
        let env = || RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() };
        let store = RomStore::with_known(env(), table.clone());
        let items = plan(&store, &[base.join("src/c64c.bin")], None);
        assert_eq!(items.len(), 2, "{items:?}");
        assert_eq!((items[0].role.as_deref(), items[0].size, items[0].action.clone()), (Some("basic"), 8192, Action::Copy));
        assert_eq!(items[0].label.as_deref(), Some("Test-Abbild: BASIC"));
        assert_eq!((items[1].role.as_deref(), items[1].label.as_deref()), (Some("kernal"), Some("Test-KERNAL")));
        assert_eq!(items[1].sha1.as_deref(), Some(hash::sha1_hex(&kernal).as_str()));
        let out0 = execute(&items[0]).unwrap();
        let out1 = execute(&items[1]).unwrap();
        assert_eq!(out0, base.join("roms/c64/basic-c64c.bin"));
        assert_eq!(std::fs::read(&out0).unwrap(), basic);
        assert_eq!(std::fs::read(&out1).unwrap(), kernal);
        // fremde Datei gleicher Größe bleibt unerkannt
        let other = plan(&store, &[base.join("src/fremd.bin")], None);
        assert!(matches!(&other[0].action, Action::Skip(_)), "{other:?}");
        // zweiter Lauf: beide Teile liegen schon im ROM-Ordner
        let store = RomStore::with_known(env(), table);
        let again = plan(&store, &[base.join("src/c64c.bin")], None);
        assert!(again.iter().all(|i| matches!(i.action, Action::AlreadyPresent(_))), "{again:?}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn teil_im_ordner_eines_anderen_systems_zaehlt_nicht_als_vorhanden() {
        let base = std::env::temp_dir().join(format!("vc-roms-comb2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        std::fs::create_dir_all(base.join("roms/c64")).unwrap();
        let (basic, kernal) = (fake(8192, 3), fake(8192, 4));
        let both: Vec<u8> = basic.iter().chain(&kernal).copied().collect();
        std::fs::write(base.join("src/c128teil.bin"), &both).unwrap();
        std::fs::write(base.join("roms/c64/basic.bin"), &basic).unwrap();   // dieselbe BASIC, aber im C64-Ordner
        let b = KnownRom { id: "c64.basic.t".into(), label: "Test-BASIC".into(), size: 8192, crc32: hash::crc32_hex(&basic), sha1: hash::sha1_hex(&basic), uses: vec![Use { system: "c64".into(), role: "basic".into() }, Use { system: "c128".into(), role: "basic64".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let table = KnownTable { format: 1, roms: vec![b], combined: vec![combined_of("combined.c128", &both, vec![(0, 8192, "c128", "basic64"), (8192, 8192, "c128", "kernal64")])] };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, table);
        let items = plan(&store, &[base.join("src/c128teil.bin")], None);
        assert_eq!(items.len(), 2, "{items:?}");
        assert_eq!((items[0].system.as_deref(), items[0].role.as_deref(), items[0].action.clone()), (Some("c128"), Some("basic64"), Action::Copy));
        assert_eq!(items[0].label.as_deref(), Some("Test-BASIC"));
        assert_eq!(execute(&items[1]).unwrap(), base.join("roms/c128/kernal64-c128teil.bin"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn kombiniertes_abbild_im_zip_und_dublette_im_plan() {
        let base = std::env::temp_dir().join(format!("vc-roms-comb3-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("src")).unwrap();
        let (lo, hi) = (fake(16384, 5), fake(16384, 6));
        let both: Vec<u8> = lo.iter().chain(&hi).copied().collect();
        let mut z = vec![]; let mut c = vec![];
        let (name, d) = ("sub/basic.bin", &both[..]);
        let off = z.len() as u32; let crc = hash::crc32(d);
        z.extend([0x50u8, 0x4B, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); z.extend(crc.to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((d.len() as u32).to_le_bytes()); z.extend((name.len() as u16).to_le_bytes()); z.extend([0, 0]); z.extend(name.as_bytes()); z.extend(d);
        c.extend([0x50u8, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]); c.extend(crc.to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((d.len() as u32).to_le_bytes()); c.extend((name.len() as u16).to_le_bytes()); c.extend([0; 12]); c.extend(off.to_le_bytes()); c.extend(name.as_bytes());
        let cd = z.len() as u32; z.extend(&c); z.extend([0x50u8, 0x4B, 5, 6, 0, 0, 0, 0, 1, 0, 1, 0]); z.extend((c.len() as u32).to_le_bytes()); z.extend(cd.to_le_bytes()); z.extend([0, 0]);
        std::fs::write(base.join("src/pack.zip"), &z).unwrap();
        // dieselbe Datei noch einmal lose unter anderem Namen
        std::fs::write(base.join("src/kopie.bin"), &both).unwrap();
        let table = KnownTable { format: 1, roms: vec![], combined: vec![combined_of("combined.p4", &both, vec![(0, 16384, "plus4", "functionlo"), (16384, 16384, "plus4", "functionhi")])] };
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, table);
        let items = plan(&store, &[base.join("src")], None);
        assert_eq!(items.len(), 4, "{items:?}");
        // lose Datei (kopie.bin) zuerst, dann ZIP: nur die ersten beiden werden kopiert
        assert_eq!(items.iter().filter(|i| i.action == Action::Copy).count(), 2, "{items:?}");
        assert_eq!(items.iter().filter(|i| matches!(i.action, Action::AlreadyPresent(_))).count(), 2);
        assert!(items.iter().any(|i| i.role.as_deref() == Some("functionhi") && i.data.as_deref() == Some(&hi[..])));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn eingebauter_katalog_teile_liegen_im_abbild_und_haben_gueltige_rollen() {
        let t = KnownTable::builtin();
        assert!(t.combined.len() >= 20);
        for c in &t.combined {
            for p in &c.parts {
                let role = crate::systems::system(&p.system).and_then(|s| s.roles.iter().find(|r| r.id == p.role)).unwrap_or_else(|| panic!("{}: {}/{}", c.id, p.system, p.role));
                assert!(role.sizes.contains(&p.size), "{}: Größe {} passt nicht zu {}/{}", c.id, p.size, p.system, p.role);
            }
            // Teile überlappen nicht
            let mut v: Vec<_> = c.parts.iter().map(|p| (p.offset, p.offset + p.size)).collect();
            v.sort();
            assert!(v.windows(2).all(|w| w[0].1 <= w[1].0), "{}", c.id);
        }
    }

    #[test]
    fn suche_in_fremden_orten_bricht_sofort_ab() {
        let store = RomStore::with_known(RomEnv { vc_roms: Some(std::env::temp_dir().join("vc-roms-nix")), ..Default::default() }, KnownTable { format: 1, combined: vec![], roms: vec![] });
        assert!(foreign_candidates_until(&store, &|| true).is_empty());
    }

    #[test]
    fn zielname_wird_nie_ueberschrieben() {
        let d = std::env::temp_dir().join(format!("vc-roms-uniq-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("a.bin"), b"x").unwrap();
        assert_eq!(unique_dest(&d, "a.bin"), d.join("a-2.bin"));
        assert_eq!(unique_dest(&d, "b.bin"), d.join("b.bin"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
