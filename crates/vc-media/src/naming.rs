// SPDX-License-Identifier: GPL-3.0-or-later
//! Dateinamen der Grafik- und Song-Dateien: `<name>.vc<art>` (z. B. `font.vcchar`, Inhalt JSON) in `graphics/` bzw. `music/`.
//! Das alte Schema (`*.asset.json` in `assets/`, Export nach `build/assets/`) wird weiter gelesen und lässt sich per `migrate` umstellen.
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Endung des alten Schemas
pub const LEGACY_EXT: &str = ".asset.json";
/// Ausgabeordner der Exporte
pub const EXPORT_DIR: &str = "build/export";
pub const LEGACY_EXPORT_DIR: &str = "build/assets";
/// Arten, die im Dateinamen vorkommen (Endung `.vc<kürzel>`, siehe [`ext_tag`])
pub const TAGS: [&str; 8] = ["sprite", "pmg", "charset", "screen", "tiles", "palette", "song", "play"];

/// Kurzname der Art im Dateinamen (`tilemap` heißt dort `tiles`)
pub fn tag(kind: &str) -> &str { if kind == "tilemap" || kind == "tileset" { "tiles" } else { kind } }
pub fn kind_of_tag(tag: &str) -> &str { if tag == "tiles" { "tilemap" } else { tag } }
/// Kürzel in der Endung (`.vc<kürzel>`): `charset` → `char`, `palette` → `pal`, sonst der Kurzname der Art
pub fn ext_tag(tag: &str) -> &str { match tag { "charset" => "char", "palette" => "pal", t => t } }
pub fn tag_of_ext_tag(x: &str) -> &str { match x { "char" => "charset", "pal" => "palette", t => t } }
/// Dateiendung der Art mit Punkt (`charset` → `.vcchar`, `tilemap` → `.vctiles`)
pub fn ext_for_kind(kind: &str) -> String { format!(".vc{}", ext_tag(tag(kind))) }
/// Standardordner: Songs in `music/`, alles andere in `graphics/`
pub fn default_dir(kind: &str) -> &'static str { if kind == "song" || kind == "play" { "music" } else { "graphics" } }
/// `dir` leer oder das alte `assets` bedeutet „Standardordner der Art“
pub fn out_dir(dir: &str, kind: &str) -> String {
    if dir.is_empty() || dir == "assets" || dir == "." { default_dir(kind).to_string() }
    else if let Some(rest) = dir.strip_prefix("assets/") { format!("{}/{rest}", default_dir(kind)) }
    else { dir.to_string() }
}

/// Kurzname der Art aus der Endung (`font.vcchar` → `charset`), `None` für fremde Endungen
fn tag_from_ext(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('/').next()?.rsplit_once('.')?.1.strip_prefix("vc")?;
    let t = tag_of_ext_tag(ext);
    TAGS.iter().copied().find(|x| *x == t)
}
/// Art laut Dateiname (`font.vcchar` → `charset`); alte Namen (`*.asset.json`) liefern `Some("")`
pub fn kind_from_name(name: &str) -> Option<String> {
    if name.ends_with(LEGACY_EXT) { return Some(String::new()); }
    tag_from_ext(name).map(|t| kind_of_tag(t).to_string())
}
pub fn is_data_file(name: &str) -> bool { kind_from_name(name).is_some() }
/// Name ohne Endung (`font.vcchar` / `font.asset.json` → `font`)
pub fn stem(name: &str) -> &str {
    let base = name.rsplit('/').next().unwrap_or(name);
    if let Some(s) = base.strip_suffix(LEGACY_EXT) { return s; }
    if tag_from_ext(base).is_some() { return base.rsplit_once('.').map_or(base, |x| x.0); }
    base
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path(); let n = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() { if depth < 6 && !n.starts_with('.') && n != "target" && n != "node_modules" { walk(&p, depth + 1, out); } } else { out.push(p); }
    }
}

/// Gibt es im Projekt noch Dateien im alten Schema (`*.asset.json`)?
pub fn has_legacy(root: &Path) -> bool { let mut all = vec![]; walk(root, 0, &mut all); all.iter().any(|p| p.strip_prefix(root).map_or(false, |r| !r.starts_with("build")) && p.to_string_lossy().ends_with(LEGACY_EXT)) }

/// Stellt ein Projekt vom alten Schema um: `*.asset.json` → `<name>.vc<art>`, `assets/` → `graphics/` bzw. `music/`,
/// `build/assets/` → `build/export/`; Verweise in Quelltexten, Includes und Assets werden mitgezogen. Liefert die Liste der Umbenennungen.
pub fn migrate(root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut all = vec![]; walk(root, 0, &mut all);
    let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/");
    let mut moves: Vec<(String, String)> = vec![];
    for p in &all {
        let r = rel(p);
        if r.starts_with("build/") || !r.ends_with(LEGACY_EXT) { continue; }
        let text = std::fs::read_to_string(p).map_err(|e| format!("{r}: {e}"))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("{r}: {e}"))?;
        let kind = v["kind"].as_str().ok_or_else(|| format!("{r}: kind fehlt"))?;
        let st = stem(&r).to_string();
        let (dir, file) = match r.rsplit_once('/') { Some((d, _)) => (d.to_string(), st.clone()), None => (String::new(), st.clone()) };
        let new_dir = if dir == "assets" { default_dir(kind).to_string() } else if let Some(rest) = dir.strip_prefix("assets/") { format!("{}/{}", default_dir(kind), rest) } else { dir };
        let to = if new_dir.is_empty() { format!("{file}{}", ext_for_kind(kind)) } else { format!("{new_dir}/{file}{}", ext_for_kind(kind)) };
        moves.push((r, to));
    }
    // Zielkonflikte vermeiden
    let mut seen = std::collections::HashSet::new();
    for (_, t) in &moves { if !seen.insert(t.clone()) || root.join(t).exists() { return Err(format!("Umstellung abgebrochen: {t} existiert bereits oder kommt doppelt vor")); } }
    // Verweise in Textdateien (Quelltexte, Includes, Projektdatei, Daten) ersetzen
    let exts = ["asm", "inc", "sbas", "bas", "json", "vcprj", "vcchar", "vcsprite", "vcpmg", "vcscreen", "vctiles", "vcsong", "vcpal", "vcplay", "vctest", "md", "txt", "vs"];
    for p in &all {
        let r = rel(p);
        if r.starts_with("build/") || p.extension().map_or(true, |e| !exts.contains(&e.to_string_lossy().as_ref())) { continue; }
        let Ok(text) = std::fs::read_to_string(p) else { continue };
        let mut t = text.clone();
        for (from, to) in &moves { t = t.replace(from.as_str(), to.as_str()); }
        t = t.replace("build/assets/", "build/export/");
        if t.contains("\"assets\"") && r.ends_with(".vcprj") { t = t.replace("\"assets\"", "\"export\""); }
        if t != text { std::fs::write(p, t).map_err(|e| format!("{r}: {e}"))?; }
    }
    for (from, to) in &moves {
        let dst = root.join(to); if let Some(d) = dst.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
        std::fs::rename(root.join(from), &dst).map_err(|e| format!("{from}: {e}"))?;
    }
    // restliche Dateien aus assets/ (ROM-Abbilder, Rohdaten) nach graphics/ verschieben, leere Ordner entfernen
    let mut rest = vec![]; walk(&root.join("assets"), 0, &mut rest);
    for p in rest { let r = rel(&p); let to = r.replacen("assets/", "graphics/", 1); let dst = root.join(&to); if dst.exists() { continue; } if let Some(d) = dst.parent() { let _ = std::fs::create_dir_all(d); } let _ = std::fs::rename(&p, &dst); }
    fn prune(d: &Path) { if let Ok(rd) = std::fs::read_dir(d) { for e in rd.flatten() { if e.path().is_dir() { prune(&e.path()); } } } let _ = std::fs::remove_dir(d); }
    prune(&root.join("assets"));
    if root.join("build/assets").is_dir() { let _ = std::fs::remove_dir_all(root.join("build/assets")); }
    Ok(moves)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names() {
        assert_eq!(kind_from_name("font.vcchar").as_deref(), Some("charset"));
        assert_eq!(kind_from_name("level.vctiles").as_deref(), Some("tilemap"));
        assert_eq!(kind_from_name("x.asset.json").as_deref(), Some(""));
        assert_eq!(kind_from_name("tune.vcprj"), None); assert_eq!(kind_from_name("layout.json"), None); assert_eq!(kind_from_name("a.b.json"), None);
        assert_eq!(kind_from_name("x.vcpal").as_deref(), Some("palette")); assert_eq!(kind_from_name("x.vcplay").as_deref(), Some("play")); assert_eq!(kind_from_name("x.vctest"), None); assert_eq!(kind_from_name("x.vcfoo"), None);
        assert_eq!(ext_for_kind("charset"), ".vcchar"); assert_eq!(ext_for_kind("tilemap"), ".vctiles"); assert_eq!(ext_for_kind("palette"), ".vcpal");
        assert_eq!(stem("graphics/font.vcchar"), "font"); assert_eq!(stem("assets/hero.asset.json"), "hero");
        assert_eq!(out_dir("assets", "song"), "music"); assert_eq!(out_dir("mein/ordner", "song"), "mein/ordner");
    }
}
