// SPDX-License-Identifier: GPL-3.0-or-later
//! Daten (E11): Hex-Ansicht (Binärdateien nur lesen, seitenweise, Suche) und Speicherkarte (Belegung der 256 Seiten aus dem letzten Bau, Seitenplan je Bankumschaltung, Prüfungen).
//! Rechnen tut `vc_ide_model::memmap_facts`; hier steht nur die Aufbereitung als JSON.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use vc_ide_model::memmap_facts as mm;
use vc_target::{PageCategory, PageKind};

/// Größe einer Seite der Hex-Ansicht und höchste geladene Dateilänge (wie in der bisherigen Oberfläche)
pub const SEITE: usize = 4096;
pub const MAX: u64 = 4 * 1024 * 1024;

/// Dateien des Projekts, die in der Hex-Ansicht statt im Standardprogramm öffnen
pub fn hex_dateien(root: &Path, rels: &[String]) -> Vec<Value> {
    rels.iter().filter_map(|rel| {
        let p = root.join(rel); if !p.is_file() || !hex_statt_extern(rel) || !crate::dateien::ist_nichttext(&p) { return None }
        let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0); Some(json!({ "rel": rel, "size": size }))
    }).collect()
}

/// Gebaute Dateien im Ordner `build/` (Programme, Diskettenabbilder, Rohdaten; bis zu drei Ebenen tief), relativ zum Projekt
pub fn build_ergebnisse(root: &Path) -> Vec<String> {
    fn lauf(root: &Path, dir: &Path, tiefe: u32, out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        let mut v: Vec<_> = rd.flatten().collect(); v.sort_by_key(|e| e.file_name());
        for e in v {
            let p = e.path();
            if p.is_dir() { if tiefe < 3 { lauf(root, &p, tiefe + 1, out); } continue }
            let l = p.to_string_lossy().to_lowercase();
            if [".prg", ".d64", ".d71", ".d81", ".atr", ".xex", ".bin", ".crt", ".sid", ".rom", ".nes", ".a26", ".tap", ".t64"].iter().any(|x| l.ends_with(x)) {
                if let Ok(r) = p.strip_prefix(root) { out.push(r.to_string_lossy().replace('\\', "/")); }
            }
        }
    }
    let mut out = Vec::new(); lauf(root, &root.join("build"), 0, &mut out); out
}

/// Rohdaten, Programme, Module, unbekannte Binärformate; Bilder, PDF, Archive und Klang bleiben beim System
pub fn hex_statt_extern(rel: &str) -> bool {
    let l = rel.to_lowercase();
    ![".png", ".jpg", ".jpeg", ".gif", ".bmp", ".ico", ".webp", ".pdf", ".zip", ".gz", ".tar", ".7z", ".wav", ".mp3", ".ogg"].iter().any(|e| l.ends_with(e))
}

/// Ladeadresse einer PRG-Datei (erste zwei Bytes), sonst 0
fn ladeadresse(rel: &str, kopf: &[u8]) -> Option<u16> { (rel.to_lowercase().ends_with(".prg") && kopf.len() >= 2).then(|| u16::from_le_bytes([kopf[0], kopf[1]])) }

/// Eine Seite (4 KiB) der Datei: Zeilen zu `breite` Bytes (8 oder 16)
pub fn hex_seite(root: &Path, rel: &str, seite: usize, breite: usize) -> Result<Value, Fehler> {
    let p = crate::dateien::absolut(root, rel)?;
    let mut f = std::fs::File::open(&p).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": format!("{}: {e}", p.display()) })))?;
    let gesamt = f.metadata().map(|m| m.len()).unwrap_or(0); let geladen = gesamt.min(MAX) as usize;
    let seiten = geladen.div_ceil(SEITE).max(1); let seite = seite.min(seiten - 1);
    let (von, bis) = (seite * SEITE, ((seite + 1) * SEITE).min(geladen));
    let mut kopf = [0u8; 2]; let _ = f.read(&mut kopf);
    f.seek(SeekFrom::Start(von as u64)).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))?;
    let mut b = vec![0u8; bis.saturating_sub(von)]; f.read_exact(&mut b).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))?;
    let breite = if breite == 8 { 8 } else { 16 };
    let zeilen: Vec<Value> = b.chunks(breite).enumerate().map(|(i, c)| json!({ "off": von + i * breite, "bytes": c, "text": c.iter().map(|x| if (0x20..0x7F).contains(x) { *x as char } else { '·' }).collect::<String>() })).collect();
    Ok(json!({ "rel": rel, "total": gesamt, "loaded": geladen, "truncated": gesamt > MAX, "pages": seiten, "page": seite, "pageSize": SEITE, "width": breite, "rows": zeilen, "load": ladeadresse(rel, &kopf) }))
}

/// Suche ab `von` (Byte-Versatz): Bytes wie `$A9 00` / `a9 00` oder Text; Antwort Versatz und Länge des Treffers oder null
pub fn hex_suche(root: &Path, rel: &str, anfrage: &str, von: usize) -> Result<Value, Fehler> {
    let p = crate::dateien::absolut(root, rel)?;
    let mut b = Vec::new(); std::fs::File::open(&p).and_then(|f| f.take(MAX).read_to_end(&mut b)).map_err(|e| Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))?;
    let q = anfrage.trim(); if q.is_empty() { return Ok(Value::Null) }
    let toks: Vec<&str> = q.split_whitespace().collect();
    let hexish = !toks.is_empty() && toks.iter().all(|t| { let h = t.trim_start_matches('$').trim_start_matches("0x"); !h.is_empty() && h.len() <= 2 && h.chars().all(|c| c.is_ascii_hexdigit()) });
    let nadel: Vec<u8> = if hexish { toks.iter().map(|t| u8::from_str_radix(t.trim_start_matches('$').trim_start_matches("0x"), 16).unwrap_or(0)).collect() } else { q.as_bytes().to_vec() };
    let ab = von.min(b.len());
    let treffer = b[ab..].windows(nadel.len()).position(|w| w == nadel.as_slice()).map(|i| ab + i).or_else(|| b[..ab.min(b.len())].windows(nadel.len()).position(|w| w == nadel.as_slice()));
    Ok(treffer.map_or(Value::Null, |o| json!({ "off": o, "len": nadel.len() })))
}

fn art_schluessel(k: PageKind) -> &'static str { match k { PageKind::Ram => "ram", PageKind::Rom => "rom", PageKind::BasicRom => "basicrom", PageKind::KernalRom => "kernalrom", PageKind::CharRom => "charrom", PageKind::Io => "io", PageKind::Module => "module" } }
fn kat_schluessel(c: PageCategory) -> &'static str { match c { PageCategory::Zeropage => "zeropage", PageCategory::Stack => "stack", PageCategory::Screen => "screen", PageCategory::Sprites => "sprites", PageCategory::Charset => "charset", PageCategory::Music => "music", PageCategory::Io => "io", PageCategory::Rom => "rom", PageCategory::Basic => "basic", PageCategory::Free => "free" } }

/// Speicherkarte aus dem letzten Bau: Seitenplan (nach `preset`), Belegung, Legende, Prüfungen; `page`: Einzelheiten dieser Seite
pub fn speicher(data: &Path, info: Option<&crate::bau::Info>, preset: Option<&str>, page: Option<u8>) -> Value {
    let Some(info) = info else { return json!({ "built": false }) };
    let banking = vc_target::load_target(data, &info.target).ok().and_then(|t| t.banking);
    let facts = std::fs::read_to_string(info.artifact.with_extension("vcdbg")).ok().and_then(|t| mm::parse_vcdbg(&t)).unwrap_or_else(|| mm::Facts {
        segs: info.segmente.iter().map(|(a, l)| mm::Seg { start: *a, len: *l, bank: 0, name: String::new() }).collect(),
        syms: info.symbols.iter().filter(|(_, v)| (0..0x10000).contains(*v)).map(|(n, v)| (n.clone(), *v as u32)).collect() });
    let gewaehlt = banking.as_ref().and_then(|b| preset.and_then(|i| b.preset(i)).or_else(|| b.default_preset()));
    let plan = match (&banking, gewaehlt) { (Some(b), Some(p)) => b.page_plan(p), _ => mm::plain_plan() };
    let usage = mm::page_usage(&facts.segs);
    let seiten: Vec<Value> = (0..256).map(|i| json!([art_schluessel(plan[i].kind), kat_schluessel(plan[i].category), usage[i]])).collect();
    let sel = page.unwrap_or_else(|| usage.iter().position(|u| *u > 0).unwrap_or(0) as u8);
    let pg = plan[sel as usize];
    let programm: u32 = usage.iter().sum();
    let frei = plan.iter().zip(usage.iter()).filter(|(p, u)| p.kind == PageKind::Ram && **u == 0).count() * 256;
    json!({
        "built": true, "target": info.target, "pages": seiten, "sel": sel,
        "legend": mm::legend(&plan).into_iter().map(|c| json!({ "key": kat_schluessel(c), "label": mm::category_label(c) })).collect::<Vec<_>>(),
        "banking": banking.as_ref().map(|b| json!({ "register": { "name": b.register.name, "addr": b.register.addr }, "presets": b.presets.iter().map(|p| json!({ "id": p.id, "label": mm::preset_label(p), "note": p.note })).collect::<Vec<_>>(), "selected": gewaehlt.map(|p| p.id.clone()) })),
        "detail": { "page": sel, "from": (sel as u32) << 8, "to": ((sel as u32) << 8) + 255, "kind": mm::kind_label(pg.kind), "category": mm::category_label(pg.category), "used": usage[sel as usize],
            "symbols": mm::page_symbols(&facts, sel, sel).into_iter().take(12).map(|(n, a)| json!([n, a])).collect::<Vec<_>>(),
            "sources": mm::page_sources(&info.zeilen, sel, sel).into_iter().take(6).map(|(f, a, b)| json!([f, a, b])).collect::<Vec<_>>() },
        "checks": mm::checks(&facts, Some(&plan)).into_iter().map(|c| json!({ "level": match c.level { mm::Level::Ok => "ok", mm::Level::Info => "info", mm::Level::Warn => "warn", mm::Level::Err => "err" }, "text": c.text })).collect::<Vec<_>>(),
        "program": programm, "free": frei, "segments": facts.segs.iter().map(|s| json!({ "start": s.start, "len": s.len, "bank": s.bank, "name": s.name })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp(tag: &str, bytes: &[u8]) -> (std::path::PathBuf, String) { let d = std::env::temp_dir().join(format!("vc-daten-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); std::fs::write(d.join("x.prg"), bytes).unwrap(); (d, "x.prg".into()) }
    #[test] fn seiten_zeilen_und_ladeadresse() {
        let mut b = vec![0x01u8, 0x08]; b.extend((0..5000u32).map(|i| (i % 251) as u8)); let (d, rel) = temp("a", &b);
        let v = hex_seite(&d, &rel, 0, 16).unwrap(); assert_eq!((v["pages"].as_u64(), v["load"].as_u64(), v["rows"].as_array().unwrap().len()), (Some(2), Some(0x801), 256));
        let v = hex_seite(&d, &rel, 9, 8).unwrap(); assert_eq!((v["page"].as_u64(), v["rows"].as_array().unwrap().len(), v["width"].as_u64()), (Some(1), (b.len() - SEITE).div_ceil(8), Some(8)));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn suche_bytes_und_text_mit_umbruch() {
        let (d, rel) = temp("b", b"\x00\x01ABC\xA9\x00\x20xyz"); let v = hex_suche(&d, &rel, "$A9 00", 0).unwrap(); assert_eq!((v["off"].as_u64(), v["len"].as_u64()), (Some(5), Some(2)));
        assert_eq!(hex_suche(&d, &rel, "xyz", 0).unwrap()["off"], 8); assert_eq!(hex_suche(&d, &rel, "ABC", 6).unwrap()["off"], 2, "Suche umbrechen"); assert!(hex_suche(&d, &rel, "nope", 0).unwrap().is_null());
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn gebaute_dateien_werden_aufgelistet() {
        let d = std::env::temp_dir().join(format!("vc-hexbuild-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("build/c64")).unwrap();
        std::fs::write(d.join("build/c64/x.prg"), [1, 8, 0]).unwrap(); std::fs::write(d.join("build/notiz.txt"), "a").unwrap();
        assert_eq!(build_ergebnisse(&d), vec!["build/c64/x.prg".to_string()]); let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn binaerdateien_werden_erkannt() { assert!(hex_statt_extern("a.prg") && !hex_statt_extern("b.PNG")); }
}
