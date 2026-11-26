// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateien, die aus dem Dateimanager in das Fenster gezogen werden: was soll mit jeder geschehen (Plan), und Kopieren ins Projekt.
//! Die Oberfläche führt den Plan aus (Projekt öffnen, Datei öffnen, Rückfrage vor dem Kopieren).
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const DISKS: [&str; 4] = ["d64", "d71", "d81", "atr"];
/// Eigene Medien: kommen beim Kopieren in den passenden Ordner des Projekts
const BILDER: [&str; 4] = ["vcsprite", "vcpmg", "vcchar", "vctiles"];
const MUSIK: [&str; 1] = ["vcsong"];
const MEDIEN_SONST: [&str; 5] = ["vcscreen", "vcpal", "vcplay", "vctest", "vs"];
const TEXTE: [&str; 9] = ["asm", "inc", "sbas", "bas", "txt", "md", "json", "s", "a65"];
const MAX_KOPIE: u64 = 32 * 1024 * 1024;

fn endung(p: &Path) -> String { p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default() }

/// Plan je Pfad: `action` ist `project` (Projekt öffnen), `open` (Datei im Projekt öffnen, mit `rel`), `disk` (Diskettenabbild öffnen), `copy` (erst ins Projekt kopieren)
/// oder `none` (mit `reason`: `no-project`, `type`, `folder`, `missing`)
pub fn plan(root: Option<&Path>, paths: &[String]) -> Value {
    let root_c = root.and_then(|r| r.canonicalize().ok());
    Value::Array(paths.iter().map(|s| {
        let p = PathBuf::from(s); let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| s.clone());
        let item = |action: &str, extra: Value| { let mut o = json!({ "path": s, "name": name, "action": action }); if let (Some(m), Some(e)) = (o.as_object_mut(), extra.as_object()) { for (k, v) in e { m.insert(k.clone(), v.clone()); } } o };
        if !p.exists() { return item("none", json!({ "reason": "missing" })); }
        let ext = endung(&p);
        if p.is_dir() { return if vc_ide::projfile::find(&p).is_some() { item("project", json!({})) } else { item("none", json!({ "reason": "folder" })) }; }
        if ext == "vcprj" { return item("project", json!({})); }
        if DISKS.contains(&ext.as_str()) { return item("disk", json!({})); }
        // liegt die Datei schon im Projekt?
        if let (Some(rc), Ok(pc)) = (&root_c, p.canonicalize()) {
            if let Ok(rel) = pc.strip_prefix(rc) { return item("open", json!({ "rel": rel.to_string_lossy().replace('\\', "/") })); }
        }
        let kopierbar = BILDER.contains(&ext.as_str()) || MUSIK.contains(&ext.as_str()) || MEDIEN_SONST.contains(&ext.as_str()) || TEXTE.contains(&ext.as_str());
        if !kopierbar { return item("none", json!({ "reason": "type" })); }
        if root.is_none() { return item("none", json!({ "reason": "no-project" })); }
        item("copy", json!({}))
    }).collect())
}

/// Kopiert eine Datei in den passenden Ordner des Projekts (Bilder nach `graphics`, Musik nach `music`, falls es sie gibt; sonst ins Projektverzeichnis);
/// ein vorhandener Name wird nie überschrieben (`name_2.ext`). Ergebnis: Pfad relativ zum Projekt.
pub fn kopieren(root: &Path, src: &Path) -> Result<String, Fehler> {
    let fehler = |e: String| Fehler::mit("datei-fehler", json!({ "detail": e }));
    let md = std::fs::metadata(src).map_err(|e| fehler(format!("{}: {e}", src.display())))?;
    if !md.is_file() { return Err(Fehler::mit("ablegen-keine-datei", json!({ "name": src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }))); }
    if md.len() > MAX_KOPIE { return Err(Fehler::mit("ablegen-zu-gross", json!({ "name": src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }))); }
    let ext = endung(src);
    let ordner = if BILDER.contains(&ext.as_str()) { "graphics" } else if MUSIK.contains(&ext.as_str()) { "music" } else { "" };
    let dir = if !ordner.is_empty() && root.join(ordner).is_dir() { root.join(ordner) } else { root.to_path_buf() };
    let stem = src.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "datei".into());
    let mut k = 1; let mut ziel;
    loop {
        let name = if k == 1 { format!("{stem}.{ext}") } else { format!("{stem}_{k}.{ext}") };
        ziel = dir.join(if ext.is_empty() { name.trim_end_matches('.').to_string() } else { name });
        if !ziel.exists() { break; } k += 1;
    }
    std::fs::copy(src, &ziel).map_err(|e| fehler(format!("{}: {e}", ziel.display())))?;
    Ok(ziel.strip_prefix(root).unwrap_or(&ziel).to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plan_nach_endung_und_ort_und_kopieren_ohne_ueberschreiben() {
        let t = std::env::temp_dir().join(format!("vc_ablegen_{}", std::process::id())); let _ = std::fs::remove_dir_all(&t);
        let (root, ext) = (t.join("proj"), t.join("extern")); std::fs::create_dir_all(root.join("graphics")).unwrap(); std::fs::create_dir_all(&ext).unwrap();
        std::fs::write(root.join("a.asm"), "nop").unwrap(); for f in ["held.vcsprite", "x.d64", "y.xyz", "z.vcprj", "text.asm"] { std::fs::write(ext.join(f), "{}").unwrap(); }
        let paths: Vec<String> = [root.join("a.asm"), ext.join("held.vcsprite"), ext.join("x.d64"), ext.join("y.xyz"), ext.join("z.vcprj"), ext.join("text.asm"), ext.join("fehlt.txt"), root.join("graphics")].iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let a: Vec<(String, Option<String>)> = plan(Some(&root), &paths).as_array().unwrap().iter().map(|v| (v["action"].as_str().unwrap().into(), v["reason"].as_str().map(String::from).or(v["rel"].as_str().map(String::from)))).collect();
        assert_eq!(a, vec![("open".into(), Some("a.asm".into())), ("copy".into(), None), ("disk".into(), None), ("none".into(), Some("type".into())), ("project".into(), None), ("copy".into(), None), ("none".into(), Some("missing".into())), ("none".into(), Some("folder".into()))]);
        let ohne = plan(None, &[ext.join("held.vcsprite").to_string_lossy().into_owned()]); assert_eq!(ohne[0]["reason"], "no-project");
        assert_eq!(kopieren(&root, &ext.join("held.vcsprite")).unwrap(), "graphics/held.vcsprite");
        assert_eq!(kopieren(&root, &ext.join("held.vcsprite")).unwrap(), "graphics/held_2.vcsprite", "nichts wird überschrieben");
        assert_eq!(kopieren(&root, &ext.join("text.asm")).unwrap(), "text.asm", "Quelltexte ins Projektverzeichnis");
        assert!(kopieren(&root, &ext).is_err());
        std::fs::remove_dir_all(t).ok();
    }
}
