// SPDX-License-Identifier: GPL-3.0-or-later
//! Media-API der IDE: Profile, neue Media, Export (ASM, BIN, PRG, DATA, INC).
use serde_json::{json, Value};
use std::path::Path;
use crate::*;

/// Einheitliche, git-freundliche Darstellung: ein Schlüssel je Zeile, Listen aus Objekten/Listen mit einem Element je Zeile.
pub fn format_media(v: &Value) -> String {
    let Some(o) = v.as_object() else { return v.to_string() };
    let items: Vec<String> = o.iter().map(|(k, val)| {
        let body = match val {
            Value::Array(a) if a.iter().any(|x| x.is_array() || x.is_object()) => format!("[\n{}\n  ]", a.iter().map(|x| format!("    {x}")).collect::<Vec<_>>().join(",\n")),
            other => other.to_string(),
        };
        format!("  {}: {body}", Value::String(k.clone()))
    }).collect();
    format!("{{\n{}\n}}\n", items.join(",\n"))
}

pub fn profiles(data: &Path) -> Result<Value, String> {
    let p = load_profiles(data)?;
    // gemessene Atari-Palette (PAL/NTSC), falls vorhanden
    let atari = crate::palette::file_json(data, "atari");
    let c64 = crate::palette::file_json(data, "c64");
    Ok(json!({ "profiles": p.iter().map(|x| x.raw.clone()).collect::<Vec<_>>(), "atari_palette": atari, "c64_palette": c64["colors"] }))
}

/// Zielsysteme aus der `.vcprj` im Projektordner (leer: keine Projektdatei, dann keine Einschränkung).
/// Dazu kommen die Systeme, die ein Zielsystem in seiner Beschreibung (`data/targets/<id>.target.json`, Feld `compatible`) als kompatibel nennt:
/// deren Media-Profile passen dann ebenfalls (z. B. C128: VIC-II und SID wie der C64).
pub fn project_systems(root: &Path, data: &Path) -> Vec<String> {
    let found = std::fs::read_dir(root).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |e| e == "vcprj")));
    let meta = found.and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let mut v: Vec<String> = meta.and_then(|m| m["targets"].as_array().map(|t| t.iter().filter_map(|x| x["system"].as_str().or(x["id"].as_str()).map(String::from)).collect())).unwrap_or_default();
    let mut i = 0;
    while i < v.len() {
        for c in compatible_systems(data, &v[i]) { if !v.contains(&c) { v.push(c); } }
        i += 1;
    }
    v
}

/// Systeme, deren Media-Profile auch für `system` gelten (Feld `compatible` der Zielbeschreibung)
pub fn compatible_systems(data: &Path, system: &str) -> Vec<String> {
    let path = data.join("targets").join(format!("{system}.target.json"));
    let Ok(text) = vc_core::read_static(&path) else { return vec![] };
    serde_json::from_str::<Value>(&text).ok().and_then(|v| v["target"]["compatible"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())).unwrap_or_default()
}

/// Neues Media aus einem Profil; ein neuer **Zeichensatz** beginnt mit dem Standardzeichensatz des Systems (Einschaltzustand), nicht leer.
pub fn create(root: &Path, data: &Path, name: &str, profile_id: &str, dir: &str) -> Result<Value, String> { create_with(root, data, name, profile_id, dir, false) }

/// Wie [`create`]; mit `blank` ist ein Zeichensatz leer.
pub fn create_with(root: &Path, data: &Path, name: &str, profile_id: &str, dir: &str, blank: bool) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let profiles = load_profiles(data)?;
    let p = profile(&profiles, profile_id)?;
    let allowed = project_systems(root, data);
    if !allowed.is_empty() && !allowed.contains(&p.target) { return Err(format!("Profil '{}' gehört zum System '{}', das Projekt hat nur: {}", p.id, p.target, allowed.join(", "))); }
    let rel = format!("{}/{name}{}", crate::naming::out_dir(dir, &p.kind), crate::naming::ext_for_kind(&p.kind));
    let path = root.join(&rel);
    if path.exists() { return Err(format!("{rel} existiert bereits")); }
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let mut media = new_media(p, name);
    if p.kind == "charset" && !blank {
        // neuer Zeichensatz = Verweis auf den Standardzeichensatz des Systems (Format 2); die Original-Glyphen stehen nicht in der Datei
        if let Some(reference) = default_std_charset(data, &p.id) {
            if let Ok(std) = std_charset_media(&p.id, &reference) { media["frames"] = std["frames"].clone(); media["base"] = json!(reference); media["format"] = json!(2); }
        }
        media = crate::romref::for_saving(&media);
    }
    std::fs::write(&path, format_media(&media)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": rel }))
}

fn parts(a: &Media, p: &Profile) -> Result<Vec<(String, Vec<u8>)>, String> {
    Ok(match p.kind.as_str() {
        "sprite" | "pmg" | "charset" => vec![(String::new(), encode(a, p)?)],
        "screen" => { let (c, at) = encode_screen(a, p)?; let mut v = vec![("chars".to_string(), c)]; if !at.is_empty() { v.push(("attrs".to_string(), at)); } v }
        "song" => crate::song::encode(&a.root, p)?,
        "palette" => crate::encode_palette(a)?,
        "tilemap" => { let (t, m, c) = encode_tilemap(a, p)?; vec![("tiles".to_string(), t), ("map".to_string(), m), ("colors".to_string(), c)] }
        k => return Err(format!("Export für '{k}' nicht unterstützt")),
    })
}

pub fn export(root: &Path, data: &Path, rel: &str, format: &str, label: Option<&str>, address: Option<u16>, _start: u32, _step: u32) -> Result<Value, String> {
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    let a = parse_media(&text)?;
    let profiles = load_profiles(data)?;
    let p = profile(&profiles, &a.profile)?;
    let label = label.map(String::from).unwrap_or_else(|| a.name.replace('-', "_"));
    if p.kind == "song" && p.raw["chip"] == "pokey" { return export_rmt(root, &a, p, &label, format, address); }
    if p.kind == "play" {
        if format != "bas" && format != "data" { return Err("PLAY-Song: Exportformat 'bas' (BASIC-Programm) verwenden".into()); }
        let txt = crate::play::to_basic(&a.root, &label)?;
        let dir = root.join("build/export"); std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = format!("{}.bas", a.name); std::fs::write(dir.join(&file), txt.as_bytes()).map_err(|e| e.to_string())?;
        let preview: String = txt.lines().take(40).collect::<Vec<_>>().join("\n");
        return Ok(json!({ "ok": true, "files": [{ "path": format!("build/export/{file}"), "size": txt.len() }], "preview": preview }));
    }
    let tiles_out = if p.kind == "tileset" || crate::tiles::is_bank(&a.root) { Some(crate::tiles::export_parts(root, &a.root, &profiles, &label)?) } else { None };
    let ps = match &tiles_out { Some(t) => t.parts.clone(), None => parts(&a, p)? };
    let dir = root.join("build/export");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut files = vec![];
    let preview: String;
    let mut write = |name: String, bytes: &[u8]| -> Result<(), String> { std::fs::write(dir.join(&name), bytes).map_err(|e| e.to_string())?; files.push(json!({ "path": format!("build/export/{name}"), "size": bytes.len() })); Ok(()) };
    let lab = |suffix: &str| if suffix.is_empty() { label.clone() } else { format!("{label}_{suffix}") };
    match format {
        "asm" => {
            let mut s = String::new();
            for (suf, b) in &ps { s.push_str(&export_asm(&lab(suf), b, 8)); s.push('\n'); }
            if let Some(t) = &tiles_out { s.push_str(&t.tail_asm); s.push_str(&t.consts); } else {
            s.push_str(&export_colors(&label, &a.root));
            s.push_str(&export_dims(&label, &a.root, p, false));
            s.push_str(&crate::sprite::export_consts(&label, p, &a.root, false)?);
            if p.kind == "song" { s.push_str(&crate::song::consts(&label, &a.root, p, false)); } }
            write(format!("{}.asm", a.name), s.as_bytes())?;
            // Sprites mit Animationen oder gemischten Modi: Tabellen in eine zweite Datei (Daten, die hinter die Sprite-Daten gehören)
            if let Some(t) = crate::sprite::export_anim_asm(&label, p, &a.root)? { write(format!("{}.anim.asm", a.name), t.as_bytes())?; }
            preview = s;
        }
        "inc" => {
            let mut s = String::new();
            let mut addr = address;
            for (suf, b) in &ps { s.push_str(&export_inc(&lab(suf), b.len(), addr)); if let Some(x) = addr { addr = Some(x.wrapping_add(b.len() as u16)); } }
            write(format!("{}.inc", a.name), s.as_bytes())?;
            preview = s;
        }
        "bin" => { for (suf, b) in &ps { write(if suf.is_empty() { format!("{}.bin", a.name) } else { format!("{}.{suf}.bin", a.name) }, b)?; } preview = format!("{} Datei(en)", files.len()); }
        "prg" => {
            let mut addr = address.ok_or("PRG-Export braucht eine Adresse")?;
            for (suf, b) in &ps { write(if suf.is_empty() { format!("{}.prg", a.name) } else { format!("{}.{suf}.prg", a.name) }, &export_prg(addr, b))?; addr = addr.wrapping_add(b.len() as u16); }
            preview = format!("{} Datei(en), Ladeadresse ${:04X}", files.len(), address.unwrap());
        }
        "sbas" => {
            let mut s = format!("' erzeugt von VintageCoder – Daten für strukturiertes BASIC (Reihenfolge: {})\n", ps.iter().map(|(n, _)| if n.is_empty() { "daten" } else { n.as_str() }).collect::<Vec<_>>().join(", "));
            for (suf, b) in &ps { s.push_str(&format!("@{}_data:\n{}", lab(suf), export_sbas_data(b))); }
            if let Some(t) = &tiles_out { s.push_str(&t.consts_sbas); } else {
            s.push_str(&export_colors_sbas(&label, &a.root));
            s.push_str(&export_dims(&label, &a.root, p, true));
            s.push_str(&crate::sprite::export_consts(&label, p, &a.root, true)?);
            if p.kind == "song" { s.push_str(&crate::song::consts(&label, &a.root, p, true)); } }
            write(format!("{}.data.sbas", a.name), s.as_bytes())?;
            preview = s;
        }
        "data" => {
            // BASIC: `@label:` + DATA-Zeilen ohne Nummern (zum Einbinden mit `#include`); mehrere Teile (Zeichen/Attribute) mit Suffix
            let mut s = String::new();
            for (suf, b) in &ps { s.push_str(&export_data_labeled(&lab(suf), b)); }
            write(format!("{}.data.bas", a.name), s.as_bytes())?;
            preview = s;
        }
        f => return Err(format!("unbekanntes Exportformat '{f}' (asm, inc, bin, prg, data, sbas)")),
    }
    let preview: String = preview.lines().take(40).collect::<Vec<_>>().join("\n");
    Ok(json!({ "ok": true, "files": files, "preview": preview }))
}

/// PNG (Bytes) in einen Frame des Media `rel` importieren und die Media-Datei schreiben.
pub fn import_png(root: &Path, data: &Path, rel: &str, png: &[u8], frame: usize, auto: bool, sys: &str) -> Result<Value, String> {
    let path = root.join(rel);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
    let mut v: Value = serde_json::from_str(&text).map_err(|e| format!("{rel}: {e}"))?;
    let profiles = load_profiles(data)?; let p = profile(&profiles, v["profile"].as_str().ok_or("profile fehlt")?)?;
    let report = crate::import::import_png(png, &mut v, p, data, frame, auto, sys)?;
    std::fs::write(&path, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "report": report, "media": v }))
}

/// Base64 (Standard-Alphabet) dekodieren; Zeilenumbrüche/Leerzeichen und ein `data:`-Präfix werden toleriert.
pub fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.split_once(',').filter(|(h, _)| h.starts_with("data:")).map(|(_, b)| b).unwrap_or(s);
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0u32);
    for c in s.bytes() {
        let v = match c { b'A'..=b'Z' => c - b'A', b'a'..=b'z' => c - b'a' + 26, b'0'..=b'9' => c - b'0' + 52, b'+' => 62, b'/' => 63, b'=' => break, b' ' | b'\n' | b'\r' | b'\t' => continue, _ => return Err("Base64: ungültiges Zeichen".into()) } as u32;
        acc = (acc << 6) | v; bits += 6; if bits >= 8 { bits -= 8; out.push((acc >> bits) as u8); acc &= (1 << bits) - 1; }
    }
    Ok(out)
}

fn find_media(dir: &Path, depth: usize, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path(); let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() { if depth < 6 && !name.starts_with('.') && name != "build" && name != "target" && name != "node_modules" { find_media(&p, depth + 1, out); } }
        else if crate::naming::is_data_file(&name) { out.push(p); }
    }
}

/// Media als Build-Eingabe: alle veralteten ``<name>.vc<art>`` des Projekts nach `build/export/<name>.asm` exportieren.
/// Abschaltbar per `.vcprj`: `"export": { "autoExport": false }`. Liefert (exportiert, Fehlermeldungen je Media).
pub fn export_all(root: &Path, data: &Path) -> (Vec<String>, Vec<(String, String)>) {
    let proj_off = std::fs::read_dir(root).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |e| e == "vcprj")))
        .and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()).map_or(false, |v| v["export"]["autoExport"] == false || v["assets"]["autoExport"] == false);
    let (mut done, mut errs) = (vec![], vec![]);
    if proj_off { return (done, errs); }
    let mut files = vec![]; find_media(root, 0, &mut files); files.sort();
    for f in files {
        let rel = vc_core::paths::rel_path(root, &f);
        let name = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v["name"].as_str().map(String::from)).unwrap_or_else(|| rel.clone());
        let out = root.join("build/export").join(format!("{name}.asm"));
        let fresh = match (std::fs::metadata(&f).and_then(|m| m.modified()), std::fs::metadata(&out).and_then(|m| m.modified())) { (Ok(a), Ok(b)) => b >= a, _ => false };
        if fresh {
            // ältere Projekte: BASIC-DATA nachliefern, falls noch nicht vorhanden oder veraltet
            let dout = root.join("build/export").join(format!("{name}.data.bas"));
            let dfresh = match (std::fs::metadata(&f).and_then(|m| m.modified()), std::fs::metadata(&dout).and_then(|m| m.modified())) { (Ok(a), Ok(b)) => b >= a, _ => false };
            if !dfresh { let _ = export(root, data, &rel, "data", None, None, 1000, 10); }
            continue;
        }
        match export(root, data, &rel, "asm", None, None, 1000, 10) { Ok(_) => done.push(rel.clone()), Err(e) => errs.push((rel.clone(), e)) }
        // zusätzlich als BASIC-DATA (build/export/<name>.data.bas) für natives BASIC; nicht möglich für alle Arten (PLAY, RMT …) – dann still ohne
        let _ = export(root, data, &rel, "data", None, None, 1000, 10);
    }
    (done, errs)
}

/// `.const <label>_<farbe> = n` für alle Einträge von `colors` (z. B. `title_bg`, `hero_mc1`)
fn export_colors(label: &str, root: &Value) -> String {
    root["colors"].as_object().map(|o| o.iter().filter_map(|(k, v)| v.as_u64().map(|n| format!(".const {label}_{} = {n}\n", k.to_lowercase()))).collect()).unwrap_or_default()
}
fn export_colors_sbas(label: &str, root: &Value) -> String {
    root["colors"].as_object().map(|o| o.iter().filter_map(|(k, v)| v.as_u64().map(|n| format!("const {label}_{} = {n}\n", k.to_lowercase()))).collect()).unwrap_or_default()
}
/// Unnummerierte DATA-Zeilen für strukturiertes BASIC (16 Werte je Zeile)
fn export_sbas_data(bytes: &[u8]) -> String {
    bytes.chunks(16).map(|c| format!("data {}\n", c.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","))).collect()
}

/// Größenkonstanten für Bildschirme und Kachelkarten: `<label>_width/_height`, bei Kacheln `_tile_count`, `_tile_width/_height`
fn export_dims(label: &str, root: &Value, p: &Profile, sbas: bool) -> String {
    let c = if sbas { "const" } else { ".const" };
    match p.kind.as_str() {
        "screen" => format!("{c} {label}_width = {}\n{c} {label}_height = {}\n", p.width, p.height),
        "tilemap" => format!("{c} {label}_width = {}\n{c} {label}_height = {}\n{c} {label}_tile_width = {}\n{c} {label}_tile_height = {}\n{c} {label}_tile_count = {}\n", p.width, p.height,
                             p.raw["tileWidth"].as_u64().unwrap_or(2), p.raw["tileHeight"].as_u64().unwrap_or(2), root["tiles"].as_array().map_or(0, |t| t.len())),
        _ => String::new(),
    }
}

/// POKEY-Songs: Export als RMT-Modul (ASM mit verschiebbaren Zeigern oder `.rmt`-Datei für eine feste Adresse)
fn export_rmt(root: &Path, a: &Media, p: &Profile, label: &str, format: &str, address: Option<u16>) -> Result<Value, String> {
    let m = crate::song::rmt_build(&a.root, p)?;
    let dir = root.join("build/export"); std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let (name, bytes, preview): (String, Vec<u8>, String) = match format {
        "asm" => { let mut t = m.to_asm(&format!("{label}_rmt")); t.push_str(&crate::song::consts(label, &a.root, p, false)); (format!("{}.asm", a.name), t.clone().into_bytes(), t) }
        "rmt" => { let f = m.to_file(address.unwrap_or(0x4000)); let n = f.len(); (format!("{}.rmt", a.name), f, format!("RMT-Modul, {n} Bytes, Ladeadresse ${:04X}", address.unwrap_or(0x4000))) }
        f => return Err(format!("POKEY-Songs werden als RMT-Modul exportiert: Format 'asm' oder 'rmt' (nicht '{f}')")),
    };
    std::fs::write(dir.join(&name), &bytes).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "files": [{ "path": format!("build/export/{name}"), "size": bytes.len() }], "preview": preview }))
}

/// Song in das Profil `to_profile` (anderer Chip) konvertieren und als `<dir>/<name>.vc<art>` schreiben
pub fn convert_song(root: &Path, data: &Path, rel: &str, to_profile: &str, name: &str, dir: &str) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    let a = parse_media(&text)?; let profiles = load_profiles(data)?;
    let (from, to) = (profile(&profiles, &a.profile)?, profile(&profiles, to_profile)?);
    let (mut v, warnings) = crate::song::convert(&a.root, from, to)?;
    v["name"] = json!(name);
    let out_rel = format!("{}/{name}{}", crate::naming::out_dir(dir, &to.kind), crate::naming::ext_for_kind(&to.kind)); let out = root.join(&out_rel);
    if out.exists() { return Err(format!("{out_rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": out_rel, "warnings": warnings }))
}

/// Song aus einer beliebigen Datei (z. B. aus einem Projekt für ein anderes System oder ein GoatTracker-`*.sng`) ins Projekt holen:
/// wird in das Profil `to_profile` umgewandelt (anderer Chip) oder, bei gleichem Chip, übernommen, und als `<dir>/<name>.asset.json` angelegt.
/// Liefert Pfad, Hinweise und die Quelle (Profil bzw. `goattracker`).
pub fn import_song(root: &Path, data: &Path, src: &Path, to_profile: &str, name: &str, dir: &str) -> Result<Value, String> {
    import_song_with(root, data, src, to_profile, name, dir, &crate::sid::SidOpts::default())
}

/// Wie [`import_song`]; `sid` gilt für `*.sid`-Dateien (Subtune und aufzunehmende Sekunden)
pub fn import_song_with(root: &Path, data: &Path, src: &Path, to_profile: &str, name: &str, dir: &str, sid: &crate::sid::SidOpts) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let profiles = load_profiles(data)?;
    let to = profile(&profiles, to_profile)?;
    let is_sng = src.extension().map_or(false, |e| e.eq_ignore_ascii_case("sng"));
    let is_sid = src.extension().map_or(false, |e| e.eq_ignore_ascii_case("sid"));
    let (from, src_root, mut warnings, from_name) = if is_sid {
        // SID-Datei: Aufnahme der Spielroutine als C64-SID-Song
        let p = profile(&profiles, "c64.sid.song")?;
        let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
        let (v, w) = crate::sid::import(&bytes, p, name, sid)?;
        (p, v, w, "sid".to_string())
    } else if is_sng {
        // GoatTracker: immer ein C64-SID-Song, danach wie jeder andere Song weiterbehandelt
        let sid = profile(&profiles, "c64.sid.song")?;
        let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
        let (v, w) = crate::goat::import(&bytes, sid, name)?;
        (sid, v, w, "goattracker".to_string())
    } else {
        let text = std::fs::read_to_string(src).map_err(|e| format!("{}: {e}", src.display()))?;
        let a = parse_media(&text)?;
        let from = profile(&profiles, &a.profile)?;
        if from.kind != "song" { return Err(format!("{}: kein Song (Profil {})", src.display(), a.profile)); }
        (from, a.root, vec![], a.profile.clone())
    };
    let mut v = if from.raw["chip"] == to.raw["chip"] {
        // gleicher Chip (z. B. C64-SID in den MEGA65 mit vier SIDs): Stimmen auffüllen, wenn das Ziel mehr hat
        let (fc, tc) = (from.raw["channels"].as_u64().unwrap_or(3) as usize, to.raw["channels"].as_u64().unwrap_or(3) as usize);
        if fc > tc { return Err(format!("Der Song hat {fc} Stimmen, das Ziel nur {tc}")); }
        let mut v = src_root; v["profile"] = json!(to.id);
        if fc < tc { for p in v["patterns"].as_array_mut().into_iter().flatten() { for row in p["rows"].as_array_mut().into_iter().flatten() { if let Some(r) = row.as_array_mut() { while r.len() < tc { r.push(json!("--- --")); } } } } }
        if fc < tc { warnings.push(format!("Gleicher Chip: {} leere Stimme(n) ergänzt", tc - fc)); } else if !is_sng && !is_sid { warnings.push("Gleicher Chip: der Song wurde unverändert übernommen".into()); }
        v
    } else { let (v, w) = crate::song::convert(&src_root, from, to)?; warnings.extend(w); v };
    v["name"] = json!(name);
    let out_rel = format!("{}/{name}{}", crate::naming::out_dir(dir, &to.kind), crate::naming::ext_for_kind(&to.kind)); let out = root.join(&out_rel);
    if out.exists() { return Err(format!("{out_rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": out_rel, "warnings": warnings, "from": from_name }))
}

/// Sprites aus einer beliebigen Datei ins Projekt holen: SPD-Format (`.spd`), Rohdaten in 64-Byte-Blöcken (`.bin`, `.spr`, `.prg`, …)
/// oder ein Sprite-Asset eines anderen Systems (C64-Sprite ↔ Atari-PMG). Das Ergebnis passt zum Profil `to_profile` und
/// liegt als `<dir>/<name>.vc<art>` im Projekt. Liefert Pfad, Hinweise, Quelle und Zahl der Sprites.
pub fn import_sprite(root: &Path, data: &Path, src: &Path, to_profile: &str, name: &str, dir: &str, o: &crate::spriteimport::SprOpts) -> Result<Value, String> {
    use crate::spriteimport::{self as si, C64Spr};
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let profiles = load_profiles(data)?;
    let to = profile(&profiles, to_profile)?;
    if !matches!(to.kind.as_str(), "sprite" | "pmg") { return Err(format!("Profil '{}' ist kein Sprite-Profil", to.id)); }
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let mut warn: Vec<String> = vec![];
    enum Source { C64(Vec<C64Spr>, [u8; 3]), Pmg(Value) }
    let dflt = |k: &str, d: u8| to.raw["defaultColors"][k].as_u64().map_or(d, |v| v as u8);
    let default_colors = [dflt("sprite", 1), dflt("mc1", 2), dflt("mc2", 6)];
    let (source, from_name) = match ext.as_str() {
        "spd" => { let (v, c, w) = si::read_spd(&bytes)?; warn.extend(w); (Source::C64(v, c), "spd".to_string()) }
        "vcsprite" | "vcpmg" | "json" => {
            let m = parse_media(&String::from_utf8_lossy(&bytes))?;
            let fp = profile(&profiles, &m.profile)?;
            match (fp.kind.as_str(), fp.target.as_str()) {
                ("sprite", "c64") => { let c = &m.root["colors"]; let col = [c["sprite"].as_u64().unwrap_or(1) as u8, c["mc1"].as_u64().unwrap_or(2) as u8, c["mc2"].as_u64().unwrap_or(6) as u8]; (Source::C64(si::read_c64_media(&m.root, fp)?, col), m.profile.clone()) }
                ("pmg", _) => (Source::Pmg(m.root.clone()), m.profile.clone()),
                (k, _) => return Err(format!("{}: kein Sprite (Art '{k}', Profil {})", src.display(), m.profile)),
            }
        }
        _ => {
            // Rohdaten; bei `.prg` die zwei Bytes Ladeadresse überspringen
            let mut oo = o.clone(); if ext == "prg" { oo.offset += 2; }
            let v = si::read_raw(&bytes, &oo, to.kind == "sprite" && to.bpp == 2)?;
            if o.mc.is_none() && to.kind == "sprite" { warn.push(format!("Rohdaten ohne Modusangabe: gelesen als {} (Modus des Zielprofils)", if to.bpp == 2 { "Multicolor" } else { "Hires" })); }
            (Source::C64(v, default_colors), "rohdaten".to_string())
        }
    };
    let media = match (to.kind.as_str(), source) {
        ("sprite", _) if to.target != "c64" => return Err(format!("Sprite-Import in '{}' wird nicht unterstützt", to.id)),
        ("sprite", Source::C64(v, col)) => si::c64_media(&v, col, to, name),
        ("sprite", Source::Pmg(m)) => { let s = si::pmg_to_c64(&m, &mut warn)?; si::c64_media(&[s], default_colors, to, name) }
        ("pmg", Source::C64(v, _)) => {
            if o.frame >= v.len() { return Err(format!("Sprite {} gibt es nicht (die Quelle hat {})", o.frame, v.len())); }
            if v.len() > 1 { warn.push(format!("Die Quelle hat {} Sprites; übernommen wird Sprite {} (ein PMG-Asset hat einen Satz aus 4 Playern)", v.len(), o.frame)); }
            si::c64_to_pmg(&v[o.frame], to, name, &mut warn)
        }
        ("pmg", Source::Pmg(mut m)) => { m["profile"] = json!(to.id); m["name"] = json!(name); m }
        _ => return Err("Import für dieses Profil nicht unterstützt".into()),
    };
    let n = media["frames"].as_array().map_or(0, |f| f.len());
    let rel = format!("{}/{name}{}", crate::naming::out_dir(dir, &to.kind), crate::naming::ext_for_kind(&to.kind)); let out = root.join(&rel);
    if out.exists() { return Err(format!("{rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&media)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": rel, "warnings": warn, "from": from_name, "count": n }))
}

/// RMT-Modul (`rel` = Datei im Projekt) in einen POKEY-Song `<dir>/<name>.vc<art>` importieren
pub fn import_rmt(root: &Path, data: &Path, rel: &str, name: &str, dir: &str) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let bytes = std::fs::read(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    let profiles = load_profiles(data)?; let p = profile(&profiles, "atari.pokey.song")?;
    let (v, warnings) = crate::song::rmt_import(&bytes, p, name)?;
    let out_rel = format!("{}/{name}.vcsong", crate::naming::out_dir(dir, "song")); let out = root.join(&out_rel);
    if out.exists() { return Err(format!("{out_rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": out_rel, "warnings": warnings }))
}

/// ROM-Abbild (Datei irgendwo auf der Platte) als Zeichensatz-Media `<dir>/<name>.vc<art>` ins Projekt übernehmen
pub fn import_rom(root: &Path, data: &Path, file: &Path, profile_id: &str, offset: usize, name: &str, dir: &str) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let profiles = load_profiles(data)?; let p = profile(&profiles, profile_id)?;
    let v = crate::import_rom(p, &bytes, offset, name)?;
    let rel = format!("{}/{name}.vcchar", crate::naming::out_dir(dir, "charset")); let out = root.join(&rel);
    if out.exists() { return Err(format!("{rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": rel }))
}

/// Eine Standardzeichensatz-Quelle: ein Satz in einem ROM-Abbild des Anwenders (Verweis `rom:system/rolle#satz`)
#[derive(Debug, Clone, PartialEq)]
pub struct StdCharset {
    /// Verweis, so wie er in `base` der Zeichensatzdatei steht
    pub reference: String,
    pub profile: String,
    /// Zielsystem des Profils
    pub system: String,
    pub label: String,
    /// `Missing`: das ROM liegt nicht vor (der Verweis lässt sich trotzdem anlegen, das Studio zeigt dann ein leeres Raster mit Hinweis)
    pub status: vc_roms::Status,
}

/// Zeichensatzquellen aller Zeichensatzprofile aus dem ROM-Verzeichnis (siehe [`crate::romref::choices`]); der Standard des Systems steht je Profil vorn
pub fn std_charset_list(data: &Path) -> Vec<StdCharset> {
    let profiles = load_profiles(data).unwrap_or_default();
    let mut out = vec![];
    crate::romref::with_store(|store| {
        for p in profiles.iter().filter(|p| p.kind == "charset") {
            let mut v: Vec<StdCharset> = crate::romref::choices(store, &p.id).into_iter().map(|c| StdCharset { reference: c.reference, profile: p.id.clone(), system: p.target.clone(), label: c.label, status: c.status }).collect();
            if let Some(d) = default_std_charset(data, &p.id) { if let Some(i) = v.iter().position(|x| x.reference == d) { let x = v.remove(i); v.insert(0, x); } }
            out.extend(v);
        }
    });
    out
}

/// Zeichensatzquellen, die zum Profil passen (nur vorhandene ROMs, wenn `only_present`)
pub fn std_charsets_for_profile(data: &Path, profile_id: &str, only_present: bool) -> Vec<StdCharset> {
    std_charset_list(data).into_iter().filter(|c| c.profile == profile_id && (!only_present || c.status != vc_roms::Status::Missing)).collect()
}

/// Der Zeichensatz, den das System beim Einschalten zeigt, als Verweis (unabhängig davon, ob das ROM vorliegt, damit Projektdateien auf jedem Rechner gleich sind).
/// MEGA65-Vollfarbe leitet sich vom C64-Satz ab, wie bisher.
pub fn default_std_charset(_data: &Path, profile_id: &str) -> Option<String> {
    Some(match profile_id {
        "c64.char.hires" | "mega65.char.fcm" => "rom:c64/chargen#upper",
        "c128.char.hires" => "rom:c128/chargen#upper",
        "c128.char.vdc" => "rom:c128/chargen#vdc",
        "vic20.char.hires" => "rom:vic20/chargen#upper",
        "plus4.char.hires" => "rom:plus4/kernal#standard",
        "atari.char.antic2" => "rom:atari8/os#standard",
        _ => return None,
    }.to_string())
}

/// Inhalt eines Zeichensatz-Verweises für ein Profil (Media-Wert mit vollständigen `frames`, `base` bleibt; fehlt das ROM: leeres Raster und `rom_missing`)
pub fn std_charset_media(profile_id: &str, reference: &str) -> Result<Value, String> {
    crate::romref::resolve_media(&crate::romref::new_reference(profile_id, "vorlage", reference, &Value::Null))
}

/// Standardzeichensatz als `<dir>/<name>.vcchar` ins Projekt legen. Standard: nur der Verweis (Format 2), die Original-Glyphen bleiben im ROM des Anwenders.
/// Mit `copy` werden die Glyphen eingebettet (Format 1); dann ist der Anwender für die Weitergabe verantwortlich.
pub fn add_std_charset(root: &Path, reference: &str, profile_id: &str, dir: &str, copy: bool) -> Result<Value, String> {
    let r = crate::romref::RomRef::parse(reference)?;
    let name = format!("{}_{}", r.system, r.set);
    let rel = format!("{}/{name}.vcchar", crate::naming::out_dir(dir, "charset")); let out = root.join(&rel);
    if out.exists() { return Err(format!("{rel} existiert bereits")); }
    let mut v = std_charset_media(profile_id, reference)?;
    v["name"] = json!(name);
    let warning = v["rom_missing"].as_str().map(String::from);
    if copy {
        for k in ["base", "rom_status", "rom_note", "rom_missing"] { v.as_object_mut().unwrap().remove(k); }
        v["format"] = json!(1);
    } else { v = crate::romref::for_saving(&v); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": rel, "warning": warning }))
}

/// Was die Erkennung über eine ROM-Datei sagt (Import-Dialog): Bezeichnung, Status und die Zeichensätze darin
#[derive(Debug, Clone)]
pub struct RomRecognition {
    /// erkannte Revision („C64 Zeichensatz 901225-01“) oder `None` (nur nach der Größe geraten)
    pub label: Option<String>,
    pub status: vc_roms::Status,
    /// (Beschreibung, Profil, Offset); bei erkannten Dateien aus der Tabelle, sonst aus [`rom_candidates`]
    pub sets: Vec<(String, String, usize)>,
}

/// Datei erkennen (Größe + Prüfsumme gegen `known.json`); nicht erkannte Dateien bekommen die Vorschläge nach Dateigröße
pub fn recognize_rom(file: &Path) -> RomRecognition {
    let guess = |len: usize| rom_candidates(len).into_iter().map(|(d, p, o)| (d.to_string(), p.to_string(), o)).collect::<Vec<_>>();
    let len = std::fs::metadata(file).map(|m| m.len() as usize).unwrap_or(0);
    crate::romref::with_store(|store| {
        let Some(f) = store.identify_file(file, None) else { return RomRecognition { label: None, status: vc_roms::Status::UnknownRevision, sets: guess(len) } };
        match (store.known_entry(&f), store.role_of(&f)) {
            (Some(k), Some((sys, role))) => {
                let sets: Vec<_> = k.char_sets(&sys, &role).into_iter().map(|s| (s.label.clone(), s.profile.clone(), s.offset)).collect();
                if sets.is_empty() { RomRecognition { label: Some(k.label.clone()), status: vc_roms::Status::Recognized, sets: vec![] } }
                else { RomRecognition { label: Some(k.label.clone()), status: if k.free { vc_roms::Status::Replacement } else { vc_roms::Status::Recognized }, sets } }
            }
            _ => RomRecognition { label: None, status: vc_roms::Status::UnknownRevision, sets: guess(len) },
        }
    })
}

/// Bekannte ROM-Abbilder mit Zeichensatz: (Beschreibung, passendes Profil, Offset) – nach Dateigröße
pub fn rom_candidates(len: usize) -> Vec<(&'static str, &'static str, usize)> {
    match len {
        4096 => vec![("C64: Großbuchstaben und Grafik (Satz 1)", "c64.char.hires", 0), ("C64: Groß-/Kleinschreibung (Satz 2)", "c64.char.hires", 0x800), ("C128: Zeichensatz Satz 1", "c128.char.hires", 0), ("C128: Zeichensatz Satz 2", "c128.char.hires", 0x800), ("MEGA65 (Vollfarbe): Satz 1", "mega65.char.fcm", 0), ("MEGA65 (Vollfarbe): Satz 2", "mega65.char.fcm", 0x800), ("VC-20: Großbuchstaben und Grafik (Satz 1)", "vic20.char.hires", 0), ("VC-20: Groß-/Kleinschreibung (Satz 2)", "vic20.char.hires", 0x800)],
        2048 => vec![("C64: Zeichensatz", "c64.char.hires", 0), ("MEGA65 (Vollfarbe)", "mega65.char.fcm", 0)],
        16384 => vec![("Atari XL/XE-Betriebssystem: Zeichensatz bei $E000", "atari.char.antic2", 0x2000), ("Atari XL/XE-Betriebssystem: internationaler Zeichensatz bei $CC00", "atari.char.antic2", 0x0C00),
                      ("Plus/4 (KERNAL-ROM): Zeichensatz bei $D000", "plus4.char.hires", 0x1000)],
        10240 => vec![("Atari 400/800 (OS-B): Zeichensatz bei $E000", "atari.char.antic2", 0x800)],
        8192 => vec![("Atari: Zeichensatz bei $E000 (ROM $C000–$FFFF → Offset $2000)", "atari.char.antic2", 0x2000), ("C128 (Zeichen-ROM): 80-Spalten-Zeichensatz (VDC 8×16)", "c128.char.vdc", 0x1000),
                     ("C128 (Zeichen-ROM): 40-Spalten-Zeichensatz Satz 1", "c128.char.hires", 0), ("C128 (Zeichen-ROM): 40-Spalten-Zeichensatz Satz 2", "c128.char.hires", 0x800)],
        1024 => vec![("Atari: Zeichensatz", "atari.char.antic2", 0)],
        _ => vec![],
    }
}

/// PNG-Raster als Zeichensatz-Media `<dir>/<name>.vc<art>` ins Projekt übernehmen
pub fn import_sheet(root: &Path, data: &Path, file: &Path, profile_id: &str, invert: bool, name: &str, dir: &str) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err("Name: nur Buchstaben, Ziffern, _ und -".into()); }
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let profiles = load_profiles(data)?; let p = profile(&profiles, profile_id)?;
    let v = crate::import_sheet(p, &bytes, name, invert)?;
    let rel = format!("{}/{name}.vcchar", crate::naming::out_dir(dir, "charset")); let out = root.join(&rel);
    if out.exists() { return Err(format!("{rel} existiert bereits")); }
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&out, format_media(&v)).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "path": rel }))
}

/// Zeichensatz-Media (Pfad relativ zum Projekt) als PNG-Raster schreiben
pub fn export_sheet(root: &Path, data: &Path, rel: &str, out: &Path) -> Result<(), String> {
    let a = parse_media(&std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"))?)?;
    let profiles = load_profiles(data)?; let p = profile(&profiles, &a.profile)?;
    std::fs::write(out, crate::export_sheet(p, &a.root)?).map_err(|e| format!("{}: {e}", out.display()))
}

/// Wandelt alte Kachelkarten (`kind: tilemap`) im Projekt um: Dateien eines Ordners mit **gleichem Kachelsatz** ergeben einen Kachelsatz
/// (`<präfix>_tiles.vctiles`) und eine Screen-Bank (`<präfix>_screens.vcscreen`, ein Screen je alter Karte). Einzelne Karten: Satz `<name>_tiles`, Bank `<name>`.
/// Die alten Dateien werden entfernt, wenn `remove_old` gesetzt ist. Liefert die Zeilen des Berichts.
pub fn migrate_tiles(root: &Path, data: &Path, remove_old: bool) -> Result<Vec<String>, String> {
    let profiles = load_profiles(data)?;
    let mut files = vec![]; find_media(root, 0, &mut files); files.sort();
    let mut groups: Vec<(std::path::PathBuf, Vec<(std::path::PathBuf, Value)>)> = vec![];
    for f in files {
        if f.extension().map_or(true, |e| e != "vctiles") { continue; }
        let Ok(v) = serde_json::from_str::<Value>(&std::fs::read_to_string(&f).map_err(|e| e.to_string())?) else { continue };
        if v["kind"] != "tilemap" { continue; }
        let dir = f.parent().unwrap().to_path_buf();
        match groups.iter_mut().find(|(d, g)| *d == dir && g[0].1["tiles"] == v["tiles"] && g[0].1["charset"] == v["charset"] && g[0].1["profile"] == v["profile"]) {
            Some((_, g)) => g.push((f, v)), None => groups.push((dir, vec![(f, v)])),
        }
    }
    let mut report = vec![];
    for (dir, g) in groups {
        let stems: Vec<String> = g.iter().map(|(f, _)| f.file_stem().unwrap().to_string_lossy().into_owned()).collect();
        let (set_name, bank_name) = if g.len() == 1 { (format!("{}_tiles", stems[0]), stems[0].clone()) } else {
            let mut pre = stems[0].clone(); for s in &stems[1..] { while !s.starts_with(&pre) { pre.pop(); } }
            let pre = pre.trim_end_matches(|c: char| c.is_ascii_digit() || c == '_' || c == '-').to_string(); let pre = if pre.is_empty() { "level".to_string() } else { pre };
            (format!("{pre}_tiles"), format!("{pre}_screens"))
        };
        let pid = g[0].1["profile"].as_str().unwrap_or("");
        let p = profile(&profiles, pid)?;
        let (tw, th, cram) = (p.raw["tileWidth"].as_u64().unwrap_or(2) as usize, p.raw["tileHeight"].as_u64().unwrap_or(2) as usize, p.raw["colorRam"].as_bool().unwrap_or(false));
        let (mut set, mut screens) = (Value::Null, vec![]);
        for (_, v) in &g { let (s, sc) = crate::tiles::migrate_tilemap(v, tw, th, cram, &set_name)?; if set.is_null() { set = s; } screens.push(sc); }
        let rel = |p: &Path| vc_core::paths::rel_path(root, p);
        let set_path = dir.join(format!("{set_name}.vctiles")); let bank_path = dir.join(format!("{bank_name}.vcscreen"));
        if set_path.exists() || bank_path.exists() { return Err(format!("{} oder {} existiert bereits", rel(&set_path), rel(&bank_path))); }
        let tp = profiles.iter().find(|x| x.kind == "screen" && x.target == p.target).map(|x| x.id.clone()).unwrap_or_else(|| "c64.screen.text".into());
        let bank = json!({ "format": 1, "kind": "screen", "profile": tp, "name": bank_name, "tilesets": [rel(&set_path)], "screens": screens });
        std::fs::write(&set_path, format_media(&set)).map_err(|e| e.to_string())?; std::fs::write(&bank_path, format_media(&bank)).map_err(|e| e.to_string())?;
        report.push(format!("{} Karte(n) → {} + {}", g.len(), rel(&set_path), rel(&bank_path)));
        if remove_old { for (f, _) in &g { std::fs::remove_file(f).map_err(|e| e.to_string())?; report.push(format!("  entfernt: {}", rel(f))); } }
    }
    if report.is_empty() { report.push("keine alten Kachelkarten gefunden".into()); }
    Ok(report)
}
