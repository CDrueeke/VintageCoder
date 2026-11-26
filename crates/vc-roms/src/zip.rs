// SPDX-License-Identifier: GPL-3.0-or-later
//! Minimaler ZIP-Leser für abgelegte ROM-Archive (Drag-and-drop): Methoden „gespeichert“ und „deflate“, kein ZIP64, keine Verschlüsselung.
//! Gelesen wird nur in den Speicher; Dateinamen aus dem Archiv werden nie als Pfad benutzt (nur der letzte Namensteil, bereinigt).
use crate::hash;

/// Größte Datei im Archiv, die gelesen wird (ROM-Abbilder sind höchstens 128 KiB)
const MAX_ENTRY: usize = 1 << 20;
const MAX_ENTRIES: usize = 4000;
/// Summe aller ausgepackten Einträge je Archiv (Schutz vor vielen mittelgroßen Einträgen); darüber wird der Rest übergangen
const MAX_TOTAL: usize = 16 << 20;

fn u16le(b: &[u8], o: usize) -> Option<usize> { Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?) as usize) }
fn u32le(b: &[u8], o: usize) -> Option<usize> { Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?) as usize) }

/// Dateiname ohne Ordneranteile und ohne Zeichen, die in Dateinamen Ärger machen
pub fn safe_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let s: String = base.chars().map(|c| if c.is_control() || matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c }).collect();
    s.trim_matches('.').to_string()
}

/// Dateien des Archivs (Name, Inhalt); zu große, verschlüsselte oder anders gepackte Einträge werden übergangen
pub fn read(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let start = data.len().saturating_sub(65_557);
    let eocd = (start..data.len().saturating_sub(21)).rev().find(|&i| data[i..].starts_with(&[0x50, 0x4B, 0x05, 0x06])).ok_or_else(|| vc_i18n::t!("romsio-zip-not-zip"))?;
    let n = u16le(data, eocd + 10).ok_or_else(|| vc_i18n::t!("romsio-zip-dir-damaged"))?;
    let mut p = u32le(data, eocd + 16).ok_or_else(|| vc_i18n::t!("romsio-zip-dir-damaged"))?;
    if n > MAX_ENTRIES { return Err(vc_i18n::t!("romsio-zip-too-many", n = n)); }
    let mut out = vec![];
    let mut total = 0usize;
    for _ in 0..n {
        if data.get(p..p + 4) != Some(&[0x50, 0x4B, 0x01, 0x02]) { return Err(vc_i18n::t!("romsio-zip-dir-damaged")); }
        let (flags, method, crc, csize, usize_, nlen, elen, clen, lh) = (u16le(data, p + 8).ok_or("ZIP")?, u16le(data, p + 10).ok_or("ZIP")?, u32le(data, p + 16).ok_or("ZIP")?, u32le(data, p + 20).ok_or("ZIP")?, u32le(data, p + 24).ok_or("ZIP")?, u16le(data, p + 28).ok_or("ZIP")?, u16le(data, p + 30).ok_or("ZIP")?, u16le(data, p + 32).ok_or("ZIP")?, u32le(data, p + 42).ok_or("ZIP")?);
        let name = String::from_utf8_lossy(data.get(p + 46..p + 46 + nlen).ok_or_else(|| vc_i18n::t!("romsio-zip-name-missing"))?).into_owned();
        p += 46 + nlen + elen + clen;
        if name.ends_with('/') || flags & 1 != 0 || usize_ > MAX_ENTRY || !(method == 0 || method == 8) { continue; }
        if data.get(lh..lh + 4) != Some(&[0x50, 0x4B, 0x03, 0x04]) { continue; }
        let (ln, le) = (u16le(data, lh + 26).ok_or("ZIP")?, u16le(data, lh + 28).ok_or("ZIP")?);
        if total + usize_ > MAX_TOTAL { continue; }
        let raw = data.get(lh + 30 + ln + le..lh + 30 + ln + le + csize).ok_or_else(|| vc_i18n::t!("romsio-zip-data-missing"))?;
        let bytes = if method == 0 { raw.to_vec() } else { match miniz_oxide::inflate::decompress_to_vec_with_limit(raw, MAX_ENTRY) { Ok(v) => v, Err(_) => continue } };
        if bytes.len() != usize_ || hash::crc32(&bytes) as usize != crc { continue; }
        total += bytes.len();
        let safe = safe_name(&name);
        if !safe.is_empty() { out.push((safe, bytes)); }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// kleines ZIP von Hand: eine gespeicherte und eine deflate-Datei
    fn zip_of(files: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let (mut out, mut central) = (vec![], vec![]);
        for (name, data, deflate) in files {
            let comp = if *deflate { miniz_oxide::deflate::compress_to_vec(data, 6) } else { data.to_vec() };
            let off = out.len() as u32;
            let (crc, method) = (hash::crc32(data), if *deflate { 8u16 } else { 0 });
            let mut lh = vec![0x50, 0x4B, 3, 4, 20, 0, 0, 0]; lh.extend(method.to_le_bytes()); lh.extend([0, 0, 0, 0]); lh.extend(crc.to_le_bytes());
            lh.extend((comp.len() as u32).to_le_bytes()); lh.extend((data.len() as u32).to_le_bytes()); lh.extend((name.len() as u16).to_le_bytes()); lh.extend([0, 0]); lh.extend(name.as_bytes()); lh.extend(&comp);
            out.extend(lh);
            let mut ch = vec![0x50, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0]; ch.extend(method.to_le_bytes()); ch.extend([0, 0, 0, 0]); ch.extend(crc.to_le_bytes());
            ch.extend((comp.len() as u32).to_le_bytes()); ch.extend((data.len() as u32).to_le_bytes()); ch.extend((name.len() as u16).to_le_bytes()); ch.extend([0; 12]); ch.extend(off.to_le_bytes()); ch.extend(name.as_bytes());
            central.extend(ch);
        }
        let cd = out.len() as u32;
        out.extend(&central);
        out.extend([0x50, 0x4B, 5, 6, 0, 0, 0, 0]); out.extend((files.len() as u16).to_le_bytes()); out.extend((files.len() as u16).to_le_bytes());
        out.extend((central.len() as u32).to_le_bytes()); out.extend(cd.to_le_bytes()); out.extend([0, 0]);
        out
    }

    #[test]
    fn liest_gespeicherte_und_gepackte_dateien_und_bereinigt_namen() {
        let a: Vec<u8> = (0..5000).map(|i| (i % 7) as u8).collect();
        let z = zip_of(&[("ordner/../kernal.bin", &a, true), ("basic.bin", b"abc", false)]);
        let v = read(&z).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!((v[0].0.as_str(), &v[0].1), ("kernal.bin", &a));
        assert_eq!(v[1].0, "basic.bin");
        assert!(read(b"kein zip").is_err());
    }

    #[test]
    fn grosse_eintraege_und_gesamtmenge_sind_begrenzt() {
        let big = vec![7u8; 2 << 20];   // über MAX_ENTRY: wird gar nicht erst entpackt
        let mid: Vec<u8> = (0..(900 << 10)).map(|i| (i % 253) as u8).collect();
        let mut files: Vec<(String, &[u8], bool)> = vec![("riesig.bin".into(), &big, true)];
        let names: Vec<String> = (0..30).map(|i| format!("m{i}.bin")).collect();
        for n in &names { files.push((n.clone(), &mid, true)); }
        let refs: Vec<(&str, &[u8], bool)> = files.iter().map(|(n, d, c)| (n.as_str(), *d, *c)).collect();
        let v = read(&zip_of(&refs)).unwrap();
        assert!(v.iter().all(|(n, _)| n != "riesig.bin"));
        assert!(v.len() < 30 && v.iter().map(|(_, d)| d.len()).sum::<usize>() <= MAX_TOTAL);
    }

    #[test]
    fn namen_sind_nie_pfade() {
        assert_eq!(safe_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_name(r"C:\x\y.bin"), "y.bin");
        assert_eq!(safe_name("a:b*c.bin"), "a_b_c.bin");
        assert_eq!(safe_name(".."), "");
    }

    #[test]
    fn meldungen_englisch() {
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::En));
        let e = read(b"kein zip").unwrap_err();
        vc_i18n::set_thread_language(Some(vc_i18n::Lang::De));
        assert_eq!(e, "not a ZIP archive");
        assert_eq!(read(b"kein zip").unwrap_err(), "kein ZIP-Archiv");
    }
}
