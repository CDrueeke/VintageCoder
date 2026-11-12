// SPDX-License-Identifier: GPL-3.0-or-later
//! Pfade im Projekt und `file://`-Adressen, plattformunabhängig: Projektpfade (`rel`) tragen immer `/`, auch unter Windows
//! (wo `Path` mit `\` rechnet). Die Funktionen arbeiten rein auf Text und sind ohne Windows testbar.
use std::path::Path;

/// Trennzeichen vereinheitlichen: `\` wird `/`
pub fn slashes(s: &str) -> String { s.replace('\\', "/") }

/// Pfad relativ zum Projektordner, mit `/`. Liegt `path` nicht unter `root`, kommt der Pfad selbst (mit `/`) zurück.
pub fn rel_path(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(p) => slashes(&p.to_string_lossy()),
        Err(_) => rel_str(&root.to_string_lossy(), &path.to_string_lossy()).unwrap_or_else(|| slashes(&path.to_string_lossy())),
    }
}

/// Wie [`rel_path`] auf Text: `Some(rel)`, wenn `path` unter `root` liegt (Laufwerksbuchstaben ohne Beachtung der Groß-/Kleinschreibung)
pub fn rel_str(root: &str, path: &str) -> Option<String> {
    let (r, p) = (slashes(root), slashes(path));
    let r = r.trim_end_matches('/');
    let same = |a: &str, b: &str| if is_drive(r) { a.eq_ignore_ascii_case(b) } else { a == b };
    if p.len() > r.len() && p.is_char_boundary(r.len()) && same(&p[..r.len()], r) && p.as_bytes()[r.len()] == b'/' { Some(p[r.len() + 1..].to_string()) } else { None }
}

fn is_drive(s: &str) -> bool { let b = s.as_bytes(); b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' }

/// `file://`-Adresse eines Pfads: `/proj/x.asm` → `file:///proj/x.asm`, `C:\proj\x.asm` → `file:///C:/proj/x.asm`
pub fn file_uri(path: &str) -> String {
    let p = slashes(path).replace('%', "%25").replace(' ', "%20");
    if p.starts_with('/') { format!("file://{p}") } else { format!("file:///{p}") }
}

pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { out.push(v); i += 3; continue; }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Pfad aus einer `file://`-Adresse (auch der alten Form `file://C:\…`); `file:///C:/x` wird zu `C:/x`
pub fn uri_to_path(uri: &str) -> String {
    let p = percent_decode(uri.strip_prefix("file://").unwrap_or(uri));
    match p.strip_prefix('/') { Some(rest) if is_drive(rest) => rest.to_string(), _ => p }
}

/// Projektpfad (`rel`, mit `/`) einer `file://`-Adresse, wenn sie unter `root` liegt
pub fn uri_to_rel(root: &Path, uri: &str) -> Option<String> { rel_str(&root.to_string_lossy(), &uri_to_path(uri)) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_pfade_als_text() {
        assert_eq!(slashes(r"sub\x.asm"), "sub/x.asm");
        assert_eq!(rel_str(r"C:\proj", r"C:\proj\sub\x.asm").as_deref(), Some("sub/x.asm"));
        assert_eq!(rel_str(r"C:\proj\", r"c:\PROJ/sub/x.asm").as_deref(), Some("sub/x.asm"));   // Laufwerk: Groß-/Kleinschreibung egal
        assert_eq!(rel_str(r"C:\proj", r"C:\projekt\x.asm"), None);   // kein Teilwort-Treffer
        assert_eq!(rel_str(r"C:\proj", r"D:\proj\x.asm"), None);
        assert_eq!(rel_str("/home/u/p", "/home/u/p/a/b.asm").as_deref(), Some("a/b.asm"));
        assert_eq!(rel_str("/home/u/p", "/home/u/P/a.asm"), None);   // Unix: Groß-/Kleinschreibung zählt
    }

    #[test]
    fn rel_path_mit_echten_pfaden() {
        assert_eq!(rel_path(Path::new("/a/b"), Path::new("/a/b/c/d.asm")), "c/d.asm");
        assert_eq!(rel_path(Path::new("/a/b"), Path::new("/x/y.asm")), "/x/y.asm");
    }

    #[test]
    fn adressen_hin_und_zurueck() {
        assert_eq!(file_uri("/home/u/my proj/x.asm"), "file:///home/u/my%20proj/x.asm");
        assert_eq!(file_uri(r"C:\proj\sub\x.asm"), "file:///C:/proj/sub/x.asm");
        assert_eq!(file_uri(r"C:\proj/sub\x.asm"), "file:///C:/proj/sub/x.asm");   // gemischt, wie root.join("sub/x.asm") unter Windows
        assert_eq!(uri_to_path("file:///C:/proj/sub/x.asm"), "C:/proj/sub/x.asm");
        assert_eq!(uri_to_path(r"file://C:\proj\x.asm"), r"C:\proj\x.asm");   // alte Form bleibt lesbar
        assert_eq!(uri_to_path("file:///home/u/my%20proj/x.asm"), "/home/u/my proj/x.asm");
        assert_eq!(uri_to_path(&file_uri("/a/100%/b c.asm")), "/a/100%/b c.asm");
    }

    #[test]
    fn leerzeichen_und_umlaute() {
        let u = file_uri(r"C:\Spiele\Größe & Spaß\x.asm");
        assert_eq!(u, "file:///C:/Spiele/Größe%20&%20Spaß/x.asm");
        assert_eq!(uri_to_path(&u), "C:/Spiele/Größe & Spaß/x.asm");
        assert_eq!(uri_to_rel(Path::new(r"C:\Spiele\Größe & Spaß"), &u).as_deref(), Some("x.asm"));
        assert_eq!(uri_to_path("file:///C:/Sp%C3%A4%C3%9Fe/x.asm"), "C:/Späße/x.asm");   // von außen kodierte Adresse
    }

    #[test]
    fn definition_liegt_im_projekt() {
        let root = Path::new(r"C:\proj");
        let uri = file_uri(r"C:\proj\sub\x.asm");
        assert_eq!(uri_to_rel(root, &uri).as_deref(), Some("sub/x.asm"));
        assert_eq!(uri_to_rel(root, "file:///C:/anderes/x.asm"), None);
        // dieselbe Adresse aus Projektordner und rel, wie sie die Oberfläche baut
        assert_eq!(file_uri(&format!("{}/{}", r"C:\proj", "sub/x.asm")), uri);
        assert_eq!(uri_to_rel(Path::new("/p"), "file:///p/a%20b.asm").as_deref(), Some("a b.asm"));
    }
}
