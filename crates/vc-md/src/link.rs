// SPDX-License-Identifier: GPL-3.0-or-later
//! Ziele von Links und Bildern: Web, Anker, Projektdatei (nur innerhalb des Projekts) oder gesperrt.

#[derive(Clone, PartialEq, Debug)]
pub enum Link {
    /// `https://…`, `http://…`, `mailto:…`: im Browser des Betriebssystems
    Web(String),
    /// `#abschnitt`: Sprung in derselben Datei
    Anchor(String),
    /// Datei im Projekt (Pfad relativ zur Projektwurzel, mit `/`), optional Zeile (`datei.asm:14`, `datei.asm#L14`)
    File { rel: String, line: Option<usize> },
    /// Nicht erlaubt; der Text nennt den Grund (für den Tooltip)
    Blocked(&'static str),
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes(); let mut out = Vec::with_capacity(b.len()); let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() { if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) { out.push(v); i += 3; continue; } }
        out.push(b[i]); i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Pfad `rel` (Projektwurzel-relativ, mit `/`) lexikalisch zusammensetzen: `..` über die Wurzel hinaus ist ein Fehler
pub fn join(base_dir: &str, path: &str) -> Option<String> {
    let mut parts: Vec<&str> = base_dir.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
    for seg in path.split('/') {
        match seg { "" | "." => {}, ".." => { parts.pop()?; } s => parts.push(s) }
    }
    Some(parts.join("/"))
}

/// Verzeichnis einer Datei (`a/b/c.md` → `a/b`)
pub fn dir_of(rel: &str) -> &str { rel.rsplit_once('/').map_or("", |x| x.0) }

/// Ziel eines Links oder Bildes in der Markdown-Datei `base` (Projektwurzel-relativ)
pub fn resolve(base: &str, url: &str) -> Link {
    let url = url.trim();
    if url.is_empty() { return Link::Blocked("leeres Ziel"); }
    if let Some(a) = url.strip_prefix('#') { return Link::Anchor(percent_decode(a).to_lowercase()); }
    // Schema: Buchstabe, dann Buchstaben/Ziffern/+/-/. bis ':'; „datei.asm:14“ ist ein Zeilenverweis, kein Schema
    if let Some(colon) = url.find(':') {
        let scheme = &url[..colon]; let rest = &url[colon + 1..];
        let is_scheme = scheme.len() >= 2 && scheme.as_bytes()[0].is_ascii_alphabetic() && scheme.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.'));
        let is_line = !rest.is_empty() && rest.bytes().all(|c| c.is_ascii_digit());
        if is_scheme && !is_line {
            return match scheme.to_ascii_lowercase().as_str() { "http" | "https" | "mailto" => Link::Web(url.to_string()), _ => Link::Blocked("Dieses Linkschema ist nicht erlaubt") };
        }
    }
    let (mut path, mut line) = (url, None);
    if let Some((p, f)) = path.split_once('#') { path = p; if let Some(n) = f.strip_prefix('L').and_then(|n| n.parse().ok()) { line = Some(n); } }
    if let Some((p, n)) = path.rsplit_once(':') { if !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()) { path = p; line = n.parse().ok(); } }
    let path = percent_decode(path);
    if path.is_empty() { return Link::Anchor(String::new()); }
    if path.starts_with('/') || path.starts_with('\\') || path.contains('\\') || path.as_bytes().get(1) == Some(&b':') { return Link::Blocked("Nur Pfade innerhalb des Projekts sind erlaubt"); }
    match join(dir_of(base), &path) { Some(rel) if !rel.is_empty() => Link::File { rel, line }, Some(_) => Link::Blocked("leeres Ziel"), None => Link::Blocked("Das Ziel liegt außerhalb des Projekts") }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(rel: &str, line: Option<usize>) -> Link { Link::File { rel: rel.into(), line } }

    #[test]
    fn web_anchor_and_files() {
        assert_eq!(resolve("README.md", "https://example.org/x"), Link::Web("https://example.org/x".into()));
        assert_eq!(resolve("README.md", "mailto:vintagecoder@drueeke.de"), Link::Web("mailto:vintagecoder@drueeke.de".into()));
        assert_eq!(resolve("README.md", "#Größe-1"), Link::Anchor("größe-1".into()));
        assert_eq!(resolve("README.md", "src/irq.asm"), f("src/irq.asm", None));
        assert_eq!(resolve("doc/a.md", "../src/irq.asm"), f("src/irq.asm", None));
        assert_eq!(resolve("doc/a.md", "./b%20c.md"), f("doc/b c.md", None));
    }

    #[test]
    fn line_numbers_in_links() {
        assert_eq!(resolve("README.md", "src/irq.asm:14"), f("src/irq.asm", Some(14)));
        assert_eq!(resolve("README.md", "src/irq.asm#L14"), f("src/irq.asm", Some(14)));
        assert_eq!(resolve("README.md", "irq.asm#abschnitt"), f("irq.asm", None));
    }

    #[test]
    fn everything_outside_the_project_is_blocked() {
        for u in ["../x.txt", "a/../../x.txt", "/etc/passwd", "C:/x.txt", "..\\x.txt", "a\\b.txt", "file:///etc/passwd", "javascript:alert(1)", "data:text/html,x", "vbscript:x", "ftp://x", ""] {
            assert!(matches!(resolve("README.md", u), Link::Blocked(_)), "{u}");
        }
        assert!(matches!(resolve("a/b.md", "../../x"), Link::Blocked(_)));
        assert_eq!(resolve("a/b.md", "../x"), f("x", None));
    }

    #[test]
    fn join_is_lexical() {
        assert_eq!(join("a/b", "../c/./d.png").as_deref(), Some("a/c/d.png"));
        assert_eq!(join("", "../x"), None);
        assert_eq!(dir_of("a/b/c.md"), "a/b"); assert_eq!(dir_of("c.md"), "");
    }
}
