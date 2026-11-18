// SPDX-License-Identifier: GPL-3.0-or-later
//! Lizenzkennungen der eingebundenen Quelltexte: liest beim Übersetzen die Zeile `SPDX-License-Identifier:` (und die Urheberzeile)
//! aus dem Kopf jeder beitragenden Datei. Dateien ohne Kennung bleiben neutral; als Ersatz dient ein Ordnervermerk
//! (`LICENSE` oder `LICENSE.txt` im Ordner der Datei oder darüber, mit `SPDX-License-Identifier: …` oder `(SPDX: …)`).
//! Das ist nur so gut wie die Kopfzeilen; Binärdaten und Dateien ohne Kennung werden nicht erkannt.
use std::collections::HashMap;
use std::path::Path;

/// Woher die Kennung stammt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Zeile im Kopf der Datei
    Header,
    /// Ordnervermerk (Pfad der `LICENSE`-Datei)
    Folder(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseEntry {
    pub file: String,
    pub license: String,
    /// Urheberzeile (ohne „Copyright“/„(c)“-Vorspann), leer wenn unbekannt
    pub holder: String,
    pub origin: Origin,
}

const HEAD_LINES: usize = 25;
const FOLDER_LEVELS: usize = 6;

fn strip_comment_marks(s: &str) -> &str {
    s.trim().trim_start_matches(|c: char| matches!(c, ';' | '/' | '#' | '*' | '-' | '!' | '\'') || c.is_whitespace()).trim_end_matches(|c: char| matches!(c, '*' | '/') || c.is_whitespace()).trim_end_matches("-->").trim()
}

/// Wert hinter `key` in der Zeile (bis zum Zeilenende, ohne Kommentarabschluss)
fn after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let i = line.find(key)?;
    let v = strip_comment_marks(&line[i + key.len()..]);
    if v.is_empty() { None } else { Some(v) }
}

fn holder_of(line: &str) -> Option<String> {
    let t = strip_comment_marks(line);
    let low = t.to_ascii_lowercase();
    let rest = if let Some(r) = after(line, "SPDX-FileCopyrightText:") { r.to_string() }
        else if low.starts_with("(c)") { t[3..].trim().to_string() }
        else if low.starts_with("copyright") { t[9..].trim().trim_start_matches("(c)").trim().to_string() }
        else if t.starts_with('©') { t['©'.len_utf8()..].trim().to_string() }
        else { return None };
    let rest = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace()).to_string();
    if rest.is_empty() { None } else { Some(rest) }
}

/// Kennung und Urheberzeile aus dem Kopf einer Datei
pub fn parse_header(lines: &[String]) -> (Option<String>, Option<String>) {
    let (mut id, mut holder_spdx, mut holder_any) = (None, None, None);
    for l in lines.iter().take(HEAD_LINES) {
        if id.is_none() { id = after(l, "SPDX-License-Identifier:").map(str::to_string); }
        if l.contains("SPDX-FileCopyrightText:") { if holder_spdx.is_none() { holder_spdx = holder_of(l); } }
        else if holder_any.is_none() && !l.contains("SPDX-") { holder_any = holder_of(l); }
    }
    (id, holder_spdx.or(holder_any))
}

/// Kennung aus einer Vermerkdatei: `SPDX-License-Identifier: X` oder `(SPDX: X)`
fn parse_folder_notice(text: &str) -> Option<(String, String)> {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let (id, holder) = parse_header(&lines);
    let id = id.or_else(|| text.find("(SPDX:").map(|i| { let r = &text[i + 6..]; r[..r.find(')').unwrap_or(r.len())].trim().to_string() }))?;
    let holder = holder.or_else(|| lines.iter().find(|l| l.trim_start().to_ascii_lowercase().starts_with("copyright")).and_then(|l| holder_of(l))).unwrap_or_default();
    Some((id, holder))
}

fn folder_license(file: &str, cache: &mut HashMap<String, Option<(String, String, String)>>) -> Option<(String, String, String)> {
    let mut dir = Path::new(file).parent()?.to_path_buf();
    for _ in 0..FOLDER_LEVELS {
        let key = dir.to_string_lossy().into_owned();
        if let Some(c) = cache.get(&key) { if c.is_some() { return c.clone(); } }
        else {
            let found = ["LICENSE", "LICENSE.txt"].iter().find_map(|n| {
                let p = dir.join(n);
                let t = std::fs::read_to_string(&p).ok()?;
                let (id, h) = parse_folder_notice(&t)?;
                Some((id, h, p.to_string_lossy().into_owned()))
            });
            cache.insert(key, found.clone());
            if found.is_some() { return found; }
        }
        if !dir.pop() || dir.as_os_str().is_empty() { break; }
    }
    None
}

/// Lizenzen aller eingebundenen Dateien (Reihenfolge wie `Output::sources`); Dateien ohne erkennbare Kennung fehlen in der Liste
pub fn scan(sources: &[(String, Vec<String>)]) -> Vec<LicenseEntry> {
    let mut cache = HashMap::new();
    let mut v = vec![];
    for (file, lines) in sources {
        let (id, holder) = parse_header(lines);
        match id {
            Some(license) => {
                // Kopfzeile ohne Urheberangabe: die Urheberzeile des Ordnervermerks (falls es einen gibt) ersetzt sie
                let holder = holder.or_else(|| folder_license(file, &mut cache).map(|f| f.1)).unwrap_or_default();
                v.push(LicenseEntry { file: file.clone(), license, holder, origin: Origin::Header })
            }
            None => if let Some((license, h, from)) = folder_license(file, &mut cache) {
                v.push(LicenseEntry { file: file.clone(), license, holder: holder.unwrap_or(h), origin: Origin::Folder(from) });
            },
        }
    }
    v
}

fn single_copyleft(id: &str) -> bool {
    let u = id.trim().trim_matches(|c| c == '(' || c == ')').to_ascii_uppercase();
    let base = u.split_whitespace().next().unwrap_or("");
    base.starts_with("GPL-") || base.starts_with("LGPL-") || base.starts_with("AGPL-") || base.starts_with("EUPL-") || base.starts_with("MPL-")
}

/// Copyleft nach der SPDX-Kennung (GPL-*, LGPL-*, AGPL-*, EUPL-*, MPL-*). Bei `A OR B` nur, wenn jede Wahl Copyleft ist; bei `A AND B`, wenn eine es ist.
pub fn is_copyleft(license: &str) -> bool {
    let u = license.to_string();
    let alts: Vec<&str> = u.split(" OR ").collect();
    alts.iter().all(|a| a.split(" AND ").any(single_copyleft))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn l(s: &str) -> Vec<String> { s.lines().map(str::to_string).collect() }

    #[test]
    fn header_lines_are_read() {
        let (id, h) = parse_header(&l("; SPDX-License-Identifier: GPL-3.0-only\n; SPDX-FileCopyrightText: 2002-2009 Radek Sterba\n; rest"));
        assert_eq!(id.as_deref(), Some("GPL-3.0-only"));
        assert_eq!(h.as_deref(), Some("2002-2009 Radek Sterba"));
        let (id, h) = parse_header(&l("// SPDX-License-Identifier: MIT-0 */\n// (c) Erika Muster 2020"));
        assert_eq!(id.as_deref(), Some("MIT-0"));
        assert_eq!(h.as_deref(), Some("Erika Muster 2020"));
        assert_eq!(parse_header(&l("lda #1")), (None, None));
    }

    #[test]
    fn copyleft_detection() {
        for s in ["GPL-3.0-only", "GPL-3.0-or-later", "LGPL-2.1-only", "AGPL-3.0-or-later", "GPL-2.0-only WITH Classpath-exception-2.0"] { assert!(is_copyleft(s), "{s}"); }
        for s in ["MIT-0", "CC0-1.0", "MIT", "Apache-2.0", "MIT OR GPL-2.0-only", "LicenseRef-Eigen"] { assert!(!is_copyleft(s), "{s}"); }
        assert!(is_copyleft("MIT AND GPL-3.0-only"));
    }

    #[test]
    fn folder_notice_is_found() {
        let d = std::env::temp_dir().join(format!("vc-lic-{}", std::process::id()));
        std::fs::create_dir_all(d.join("sub")).unwrap();
        std::fs::write(d.join("LICENSE"), "Diese Dateien stehen unter MIT-0 (SPDX: MIT-0).\n\nCopyright 2026 Erika\n").unwrap();
        let f = d.join("sub").join("a.asm").to_string_lossy().into_owned();
        let v = scan(&[(f.clone(), l("lda #1"))]);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].license, "MIT-0");
        assert_eq!(v[0].holder, "2026 Erika");
        assert!(matches!(v[0].origin, Origin::Folder(_)));
        // Kopfzeile geht vor
        let v = scan(&[(f, l("; SPDX-License-Identifier: GPL-3.0-only\nlda #1"))]);
        assert_eq!(v[0].license, "GPL-3.0-only");
        std::fs::remove_dir_all(&d).ok();
    }
}
