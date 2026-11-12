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

fn item_for(store: &RomStore, f: &Found, source_label: Option<String>) -> ImportItem {
    let mut it = ImportItem { src: f.path.clone(), system: None, role: None, label: None, size: f.size, crc32: f.crc32, sha1: f.sha1.clone(), dest: None, action: Action::Skip("nicht erkannt".into()), source_label, data: None };
    let Some((system, role)) = store.role_of(f) else {
        it.action = Action::Skip(match &f.ident {
            crate::Ident::Ambiguous { candidates, .. } => format!("Rolle unklar (passt zu {})", candidates.join(", ")),
            crate::Ident::WrongSize { expected, .. } => format!("falsche Größe (erwartet {} Byte)", expected.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" oder ")),
            crate::Ident::Unrecognized(r) => r.clone(),
            _ => "nicht erkannt".into(),
        });
        return it;
    };
    if vc_roms_system_unknown(&system) { it.system = Some(system); it.role = Some(role); it.action = Action::Skip("gehört zu einem System, das VintageCoder nicht unterstützt (z. B. C65)".into()); return it; }
    it.label = store.known_entry(f).map(|k| k.label.clone());
    it.system = Some(system.clone());
    it.role = Some(role);
    // schon im eigenen Ordner (Projekt, portabel, Benutzer)? Fremde Fundorte und Mitgeliefertes zählen nicht.
    let own = |g: &&Found| matches!(g.stage, Stage::Project | Stage::Portable | Stage::User | Stage::Legacy);
    if let Some(same) = store.found().iter().filter(own).find(|g| g.sha1.is_some() && g.sha1 == f.sha1) {
        it.action = Action::AlreadyPresent(same.path.clone());
        return it;
    }
    match store.env.install_dir() {
        Some(dir) => {
            if env::inside_app_bundle(&dir) { it.action = Action::Skip("Zielordner liegt in einem App-Bundle".into()); return it; }
            let name = f.file_name();
            it.dest = Some(unique_dest(&dir.join(&system), &name));
            it.action = Action::Copy;
        }
        None => it.action = Action::Skip("kein ROM-Ordner bekannt (kein Konfigurationsordner)".into()),
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

/// Plan für Dateien oder Ordner (Drag-and-drop). `system` ist ein Hinweis des Nutzers (gewählter Reiter), kein Zwang.
pub fn plan(store: &RomStore, sources: &[PathBuf], system: Option<&str>) -> Vec<ImportItem> {
    let mut files = vec![];
    for s in sources {
        if s.is_dir() {
            if let Ok(rd) = std::fs::read_dir(s) { let mut v: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect(); v.sort(); files.extend(v); }
        } else { files.push(s.clone()); }
    }
    let mut out = vec![];
    for p in &files {
        let name = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        if name.ends_with(".zip") { out.extend(plan_zip(store, p, system)); continue; }
        if let Some(f) = store.identify_file(p, system) { out.push(item_for(store, &f, None)); }
    }
    out
}

/// Inhalt eines ZIP-Archivs (Drag-and-drop): nur erkannte ROM-Dateien werden angeboten, alles andere im Archiv bleibt unberührt
fn plan_zip(store: &RomStore, zip: &Path, system: Option<&str>) -> Vec<ImportItem> {
    let skip = |why: String| ImportItem { src: zip.to_path_buf(), system: None, role: None, label: None, size: zip.metadata().map(|m| m.len()).unwrap_or(0), crc32: None, sha1: None, dest: None, action: Action::Skip(why), source_label: None, data: None };
    let bytes = match std::fs::read(zip) { Ok(b) => b, Err(e) => return vec![skip(format!("{}: {e}", zip.display()))] };
    let entries = match crate::zip::read(&bytes) { Ok(e) => e, Err(e) => return vec![skip(e)] };
    let mut out = vec![];
    for (name, data) in entries {
        let Some(f) = store.identify_bytes(&name, &data, system) else { continue };
        if store.role_of(&f).is_none() { continue; }
        let mut it = item_for(store, &f, Some(format!("Archiv {}", zip.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())));
        it.src = zip.to_path_buf();
        it.data = Some(data);
        out.push(it);
    }
    if out.is_empty() { out.push(skip("Archiv enthält keine erkannten ROM-Dateien".into())); }
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
            if env::inside_app_bundle(dest) { return Err("nie in ein App-Bundle schreiben".into()); }
            std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| format!("{}: {e}", dest.display()))?;
            match &item.data {
                Some(d) => std::fs::write(dest, d).map_err(|e| format!("{}: {e}", dest.display()))?,
                None => { std::fs::copy(&item.src, dest).map_err(|e| format!("{} → {}: {e}", item.src.display(), dest.display()))?; }
            }
            Ok(dest.clone())
        }
        (Action::AlreadyPresent(p), _) => Ok(p.clone()),
        (Action::Skip(why), _) => Err(why.clone()),
        _ => Err("kein Ziel".into()),
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
        let store = RomStore::with_known(RomEnv { vc_roms: Some(dst.clone()), ..Default::default() }, KnownTable { format: 1, roms: vec![k.clone()] });
        let items = plan(&store, &[src.clone()], None);
        assert_eq!(items.len(), 3);
        let kern = items.iter().find(|i| i.src.ends_with("k.bin")).unwrap();
        assert_eq!(kern.action, Action::Copy);
        assert_eq!(kern.system.as_deref(), Some("c64"));
        assert!(matches!(items.iter().find(|i| i.src.ends_with("zz.bin")).unwrap().action, Action::Skip(_)));
        assert!(matches!(items.iter().find(|i| i.src.ends_with("pack.zip")).unwrap().action, Action::Skip(_)));
        let out = execute(kern).unwrap();
        assert_eq!(out, dst.join("c64/k.bin"));
        assert_eq!(std::fs::read(&out).unwrap(), data);
        // zweiter Lauf: schon vorhanden
        let store = RomStore::with_known(RomEnv { vc_roms: Some(dst), ..Default::default() }, KnownTable { format: 1, roms: vec![k] });
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
        let store = RomStore::with_known(RomEnv { vc_roms: Some(base.join("roms")), ..Default::default() }, KnownTable { format: 1, roms: vec![k] });
        let items = plan(&store, &[base.join("pack.zip")], None);
        assert_eq!(items.len(), 1, "{items:?}");
        assert_eq!(items[0].action, Action::Copy);
        let out = execute(&items[0]).unwrap();
        assert_eq!(out, base.join("roms/c64/k.bin"));
        assert_eq!(std::fs::read(out).unwrap(), data);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn suche_in_fremden_orten_bricht_sofort_ab() {
        let store = RomStore::with_known(RomEnv { vc_roms: Some(std::env::temp_dir().join("vc-roms-nix")), ..Default::default() }, KnownTable { format: 1, roms: vec![] });
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
