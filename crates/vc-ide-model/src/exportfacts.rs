// SPDX-License-Identifier: GPL-3.0-or-later
//! Export der Studios ohne Oberfläche: Formate, Adresse, Hexauszug und Vorschau im Wegwerf-Ordner.
use std::path::Path;

/// Formate des Dialogs: (Kennung für `vc_media::project::export`, Beschriftung)
/// Bezeichnung eines Exportformats in der gewählten Sprache
pub fn format_label(id: &str) -> String {
    match id {
        "asm" => "Assembler".to_string(), "inc" => vc_i18n::t!("exportfacts-konstanten"), "bin" => vc_i18n::t!("exportfacts-binaer"), "prg" => "PRG".to_string(), "data" => "BASIC DATA".to_string(),
        _ => id.to_string(),
    }
}

pub const FORMATS: &[(&str, &str)] = &[("asm", "Assembler"), ("inc", "Konstanten"), ("bin", "Binär"), ("prg", "PRG"), ("data", "BASIC DATA")];

/// Adresse in `$C000`, `0xC000` oder dezimaler Schreibweise
pub fn parse_addr(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('$').or_else(|| s.strip_prefix("0x")).or_else(|| s.strip_prefix("0X")) { return u16::from_str_radix(h, 16).ok(); }
    s.parse().ok()
}

/// Vorschlag für die Adresse je Art
pub fn default_addr(kind: &str) -> &'static str { match kind { "screen" => "$0400", "tilemap" | "tileset" => "$4000", "charset" => "$2000", "palette" => "$C000", _ => "$2000" } }

/// Hexauszug (16 Bytes je Zeile mit Adresse und ASCII-Spalte), höchstens `max` Bytes
pub fn hex_dump(bytes: &[u8], base: usize, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in bytes.chunks(16).take(max.div_ceil(16)).enumerate() {
        let hex: Vec<String> = ch.iter().map(|b| format!("{b:02x}")).collect();
        let asc: String = ch.iter().map(|b| if (0x20..0x7f).contains(b) { *b as char } else { '.' }).collect();
        out.push_str(&format!("{:04x}  {:<47}  {asc}\n", base + i * 16, hex.join(" ")));
    }
    if bytes.len() > max { out.push_str(&format!("{}\n", vc_i18n::t!("exportfacts-p1-weitere-bytes", p1 = bytes.len() - max))); }
    out
}

/// Ergebnis einer Probe-Ausgabe
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preview { pub files: Vec<(String, usize)>, pub text: String, pub hex: String, pub error: Option<String>, /** erzeugte `tiles_use`-Zeile (Screen-Banken), sonst leer */ pub tiles_use: String }

/// Rechnet den Export von `media_text` (Inhalt der Mediendatei `rel`) in einem Wegwerf-Ordner aus.
/// Eigener Temp-Ordner je Aufruf (Prozess und Zähler), damit gleichzeitige Vorschauen
/// (mehrere Dialoge, parallele Tests) einander nicht löschen
fn unique_tmp(prefix: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
}

pub fn preview(data: &Path, rel: &str, media_text: &str, format: &str, label: &str, addr: Option<u16>) -> Preview { preview_im_projekt(None, data, rel, media_text, format, label, addr) }

/// Dateien, auf die ein Medium verweist (Zeichensätze, Kachelsätze samt deren Zeichensätzen): Kachelsatz- und Bildschirm-Exporte lesen sie
fn abhaengige(root: &Path, media_text: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let Ok(v) = serde_json::from_str::<serde_json::Value>(media_text) else { return out };
    let sammle = |v: &serde_json::Value, out: &mut Vec<String>| {
        if let Some(c) = v["charset"].as_str() { out.push(c.to_string()) }
        for k in ["charsets", "tilesets"] { for x in v[k].as_array().into_iter().flatten() { if let Some(s) = x.as_str() { out.push(s.to_string()) } } }
    };
    sammle(&v, &mut out);
    for t in out.clone() { if t.ends_with(".vctiles") { if let Some(w) = std::fs::read_to_string(root.join(&t)).ok().and_then(|x| serde_json::from_str::<serde_json::Value>(&x).ok()) { sammle(&w, &mut out) } } }
    out.sort(); out.dedup(); out
}
fn abhaengige_kopieren(root: Option<&Path>, tmp: &Path, media_text: &str) {
    let Some(root) = root else { return };
    for rel in abhaengige(root, media_text) { let (q, z) = (root.join(&rel), tmp.join(&rel)); if let Some(p) = z.parent() { let _ = std::fs::create_dir_all(p); } let _ = std::fs::copy(q, z); }
}

/// Wie [`preview`]; mit `root` stehen die Dateien, auf die das Medium verweist (Zeichensätze, Kachelsätze), für den Export bereit
pub fn preview_im_projekt(root: Option<&Path>, data: &Path, rel: &str, media_text: &str, format: &str, label: &str, addr: Option<u16>) -> Preview {
    let tmp = unique_tmp("vc_export_prev");
    let _ = std::fs::remove_dir_all(&tmp);
    let run = || -> Result<Preview, String> {
        let file = tmp.join(rel);
        std::fs::create_dir_all(file.parent().ok_or(vc_i18n::t!("exportfacts-pfad"))?).map_err(|e| e.to_string())?;
        std::fs::write(&file, media_text).map_err(|e| e.to_string())?; abhaengige_kopieren(root, &tmp, media_text);
        let v = vc_media::project::export(&tmp, data, rel, format, Some(label).filter(|l| !l.is_empty()), addr, 10, 10)?;
        let files: Vec<(String, usize)> = v["files"].as_array().map(|a| a.iter().filter_map(|f| Some((f["path"].as_str()?.rsplit('/').next()?.to_string(), f["size"].as_u64()? as usize))).collect()).unwrap_or_default();
        let first = v["files"][0]["path"].as_str().map(|p| tmp.join(p));
        let bytes = first.and_then(|p| std::fs::read(p).ok()).unwrap_or_default();
        let is_text = matches!(format, "asm" | "inc" | "data" | "bas");
        let text = if is_text { v["preview"].as_str().unwrap_or("").to_string() } else { vc_i18n::t!("exportfacts-p1-binaerdaten-reiter-hex", p1 = v["preview"].as_str().unwrap_or("")) };
        Ok(Preview { files, text, hex: hex_dump(&bytes, addr.unwrap_or(0) as usize, 512), error: None, tiles_use: v["tilesUse"].as_str().unwrap_or("").to_string() })
    };
    let r = run().unwrap_or_else(|e| Preview { error: Some(e), ..Default::default() });
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

/// Vollständiger Text des ersten erzeugten Textdatei-Exports (asm, inc, data), ohne die Kürzung der Vorschau; für „direkt in den Quelltext kopieren“
pub fn volltext_im_projekt(root: Option<&Path>, data: &Path, rel: &str, media_text: &str, format: &str, label: &str, addr: Option<u16>) -> Result<String, String> {
    let tmp = unique_tmp("vc_export_full");
    let _ = std::fs::remove_dir_all(&tmp);
    let r = (|| -> Result<String, String> {
        let file = tmp.join(rel);
        std::fs::create_dir_all(file.parent().ok_or(vc_i18n::t!("exportfacts-pfad"))?).map_err(|e| e.to_string())?;
        std::fs::write(&file, media_text).map_err(|e| e.to_string())?; abhaengige_kopieren(root, &tmp, media_text);
        let v = vc_media::project::export(&tmp, data, rel, format, Some(label).filter(|l| !l.is_empty()), addr, 10, 10)?;
        let p = v["files"][0]["path"].as_str().ok_or_else(|| vc_i18n::t!("exportdlg-der-export-hat-keine-datei-erzeugt").to_string())?;
        std::fs::read_to_string(tmp.join(p)).map_err(|e| e.to_string())
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

/// BASIC-DATA-Zeilen (Format `data`) zur Vorschau
pub fn preview_data(data: &Path, rel: &str, media_text: &str, label: &str) -> Result<String, String> { preview_data_im_projekt(None, data, rel, media_text, label) }
pub fn preview_data_im_projekt(root: Option<&Path>, data: &Path, rel: &str, media_text: &str, label: &str) -> Result<String, String> {
    let tmp = unique_tmp("vc_export_data");
    let _ = std::fs::remove_dir_all(&tmp);
    let r = (|| -> Result<String, String> {
        let file = tmp.join(rel);
        std::fs::create_dir_all(file.parent().ok_or(vc_i18n::t!("exportfacts-pfad"))?).map_err(|e| e.to_string())?;
        std::fs::write(&file, media_text).map_err(|e| e.to_string())?; abhaengige_kopieren(root, &tmp, media_text);
        let v = vc_media::project::export(&tmp, data, rel, "data", Some(label).filter(|l| !l.is_empty()), None, 10, 10)?;
        Ok(v["preview"].as_str().unwrap_or("").to_string())
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adressen_lesen() {
        assert_eq!(parse_addr("$C000"), Some(0xC000));
        assert_eq!(parse_addr(" 0x0400 "), Some(0x400));
        assert_eq!(parse_addr("8192"), Some(8192));
        assert_eq!(parse_addr("$zz"), None);
        assert_eq!(parse_addr("70000"), None);
        assert_eq!(parse_addr(""), None);
    }

    #[test]
    fn hexauszug() {
        let d: Vec<u8> = (0..20u8).map(|i| 0x40 + i).collect();
        let h = hex_dump(&d, 0x2000, 16);
        assert!(h.starts_with("2000  40 41 42"), "{h}");
        assert!(h.contains("@ABCDEFGHIJKLMNO"));
        assert!(h.contains("… 4 weitere Bytes"));
    }

    #[test]
    fn vorschau_gleichzeitig() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let med = vc_media::project::format_media(&serde_json::json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "font", "frames": (0..256).map(|_| serde_json::json!({ "rows": vec!["00000000"; 8] })).collect::<Vec<_>>() }));
        let hs: Vec<_> = (0..8).map(|_| { let (d, m) = (data.clone(), med.clone()); std::thread::spawn(move || (0..20).map(|_| preview(&d, "gfx/font.vcchar", &m, "prg", "font", Some(0x2000))).all(|p| p.error.is_none() && p.files.len() == 1)) }).collect();
        for h in hs { assert!(h.join().unwrap(), "gleichzeitige Vorschau schlug fehl"); }
    }

    #[test]
    fn formate() { assert_eq!(FORMATS.len(), 5); assert!(FORMATS.iter().all(|f| !f.1.is_empty())); }

    #[test]
    fn vorschau_ohne_projektordner() {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        let med = vc_media::project::format_media(&serde_json::json!({ "format": 1, "kind": "charset", "profile": "c64.char.hires", "name": "font", "frames": (0..256).map(|_| serde_json::json!({ "rows": vec!["00000000"; 8] })).collect::<Vec<_>>() }));
        let p = preview(&data, "gfx/font.vcchar", &med, "asm", "font", None);
        assert!(p.error.is_none(), "{:?}", p.error);
        assert!(p.files.iter().any(|f| f.0.ends_with(".asm")));
        assert!(p.text.contains("font"), "{}", p.text);
        let p = preview(&data, "gfx/font.vcchar", &med, "prg", "font", Some(0x2000));
        assert!(p.error.is_none(), "{:?}", p.error);
        assert_eq!(p.files[0].1, 2048 + 2);
        assert!(p.hex.starts_with("2000"));
        let p = preview(&data, "gfx/font.vcchar", &med, "prg", "font", None);
        assert!(p.error.is_some());
        assert!(preview_data(&data, "gfx/font.vcchar", &med, "font").is_ok());
    }
}

// ---------------------------------------------------------------------------------------------- Kachel-Layout (K2)

/// Speicherbedarf eines Kachel-Exports in Bytes, nach Teilen (ohne die kleinen Zeigertabellen `_screens_lo/_hi`)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KachelSpeicher { pub kacheln: usize, pub farben: usize, pub eigenschaften: usize, pub zeichensatz: usize, pub karten: usize, pub zellen: usize }
impl KachelSpeicher { pub fn summe(&self) -> usize { self.kacheln + self.farben + self.eigenschaften + self.zeichensatz + self.karten + self.zellen } }

/// Mediendatei mit eingetragenem Layout (für Vorschau und Rechnung; die Datei im Projekt bleibt unberührt)
pub fn mit_layout(media_text: &str, lay: vc_media::tiles::TilesLayout) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(media_text) else { return media_text.to_string() };
    match lay.json() { Some(j) => v["layout"] = j, None => { if let Some(o) = v.as_object_mut() { o.remove("layout"); } } }
    vc_media::project::format_media(&v)
}

/// Speicherbedarf des Exports von Kachelsatz oder Screen-Bank mit dem Layout `lay` (die echte Ausgabe wird in einem Wegwerfordner erzeugt und gewogen)
pub fn kachel_speicher(root: Option<&Path>, data: &Path, rel: &str, media_text: &str, lay: vc_media::tiles::TilesLayout) -> Result<KachelSpeicher, String> {
    let p = preview_im_projekt(root, data, rel, &mit_layout(media_text, lay), "bin", "m", None);
    if let Some(e) = p.error { return Err(e) }
    let mut k = KachelSpeicher::default();
    for (name, size) in &p.files {
        let teil = name.split('.').nth(1).unwrap_or("");
        match teil {
            "chars" => k.zeichensatz += size,
            "props" => k.eigenschaften += size,
            t if t.starts_with("col") => k.farben += size,
            t if t.starts_with("map") => k.karten += size,
            t if t.starts_with("cells") => k.zellen += size,
            _ => k.kacheln += size,
        }
    }
    Ok(k)
}

#[cfg(test)]
mod kachel_tests {
    use super::*;
    use vc_media::tiles::TilesLayout;
    #[test] fn speicherbedarf_der_jumprun_bank_16_bit_verdoppelt_nur_die_karten() {
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."); let root = repo.join("examples/c64/jumprun");
        let text = std::fs::read_to_string(root.join("graphics/levels.vcscreen")).unwrap();
        let a = kachel_speicher(Some(&root), &repo.join("data"), "graphics/levels.vcscreen", &text, TilesLayout::default()).unwrap();
        let b = kachel_speicher(Some(&root), &repo.join("data"), "graphics/levels.vcscreen", &text, TilesLayout { index16: true, ..Default::default() }).unwrap();
        assert!(a.karten > 0 && a.zeichensatz > 0, "{a:?}"); assert_eq!(b.karten, a.karten * 2, "Hochbyte-Ebene dazu"); assert_eq!(b.zeichensatz, a.zeichensatz);
        assert_eq!(b.summe(), a.summe() + a.karten);
        let t = mit_layout(&text, TilesLayout { map_columns: true, ..Default::default() }); assert!(t.contains("\"columns\""), "Layout im Text"); assert!(!mit_layout(&t, TilesLayout::default()).contains("\"layout\""), "Vorgabe entfernt den Eintrag wieder");
    }
}
