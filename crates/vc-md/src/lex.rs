// SPDX-License-Identifier: GPL-3.0-or-later
//! Farben für eine Markdown-Zeile (Zustandsautomat über die Zeilen). Die Spannen decken den Text nie doppelt ab; nicht genannte Teile sind
//! gewöhnlicher Text. Markierungszeichen (`#`, `**`, `` ` ``, `[`, `](` …) bleiben im Text und bekommen die Art `Marker`: nichts wird ausgeblendet.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Marker, Heading, Bold, Italic, Strike, Code, CodeBlock, Fence, LinkText, Url, ListMark, Quote, Rule, Html, Front, Table }

/// Zustand zwischen den Zeilen: offener Zaunblock (Zeichen, Länge) und Ende des Front Matters
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct State { fence: Option<(u8, usize)>, front_end: Option<usize> }

impl State {
    /// Zustand für eine Datei (ermittelt das Front Matter einmal)
    pub fn new(text: &str) -> State { State { fence: None, front_end: crate::front_matter_end(text) } }
    /// Steht die nächste Zeile in einem offenen Zaunblock?
    pub fn in_fence(&self) -> bool { self.fence.is_some() }
    /// Liegt Zeile `n` im Front Matter?
    pub fn in_front(&self, n: usize) -> bool { self.front_end.map_or(false, |e| n <= e) }
}

type Span = (usize, usize, Kind);

fn lead(s: &str) -> usize { s.len() - s.trim_start_matches(' ').len() }

/// Farben der Zeile `n` (0-basiert), Byte-Bereiche, aufsteigend
pub fn line(text: &str, n: usize, st: &mut State) -> Vec<Span> {
    if st.front_end.map_or(false, |e| n <= e) { return if text.is_empty() { vec![] } else { vec![(0, text.len(), Kind::Front)] }; }
    let t = text.trim_end_matches('\r');
    let ind = lead(t); let body = &t[ind..];
    if let Some((ch, len)) = st.fence {
        let run = body.bytes().take_while(|c| *c == ch).count();
        if ind < 4 && run >= len && body[run..].trim().is_empty() { st.fence = None; return vec![(0, t.len(), Kind::Fence)]; }
        return if t.is_empty() { vec![] } else { vec![(0, t.len(), Kind::CodeBlock)] };
    }
    if ind < 4 {
        for ch in [b'`', b'~'] {
            let run = body.bytes().take_while(|c| *c == ch).count();
            if run >= 3 && !(ch == b'`' && body[run..].contains('`')) { st.fence = Some((ch, run)); return vec![(0, t.len(), Kind::Fence)]; }
        }
    }
    if ind >= 4 && !is_list_start(body) { return vec![]; }
    let mut out: Vec<Span> = vec![];
    // Überschrift
    let hashes = body.bytes().take_while(|c| *c == b'#').count();
    if ind < 4 && (1..=6).contains(&hashes) && (body.len() == hashes || body.as_bytes()[hashes] == b' ' || body.as_bytes()[hashes] == b'\t') {
        out.push((ind, ind + hashes, Kind::Marker));
        let rest = &body[hashes..]; let core = rest.trim_end(); let no_hash = core.trim_end_matches('#');
        let closing = if core.len() > no_hash.len() && (no_hash.is_empty() || no_hash.ends_with(' ')) { core.len() - no_hash.len() } else { 0 };
        let end = ind + hashes + core.len() - closing;
        if end > ind + hashes { out.push((ind + hashes, end, Kind::Heading)); }
        if closing > 0 { out.push((end, end + closing, Kind::Marker)); }
        return out;
    }
    // Setext-Unterstreichung und Trennlinie
    let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    if ind < 4 && compact.len() >= 3 {
        let c0 = compact.as_bytes()[0];
        if matches!(c0, b'-' | b'*' | b'_') && compact.bytes().all(|c| c == c0) { return vec![(ind, t.trim_end().len(), Kind::Rule)]; }
        if c0 == b'=' && compact.bytes().all(|c| c == b'=') { return vec![(ind, t.trim_end().len(), Kind::Marker)]; }
    }
    // HTML-Zeile
    if ind < 4 && body.starts_with('<') && body[1..].chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '/' || c == '!') && !body.starts_with("<http") { return vec![(ind, t.trim_end().len(), Kind::Html)]; }
    let mut pos = ind; let mut base = None::<Kind>;
    // Zitat
    if ind < 4 && body.starts_with('>') {
        let mut p = ind;
        loop { let r = &t[p..]; if let Some(r2) = r.strip_prefix('>') { out.push((p, p + 1, Kind::Marker)); p += 1; let _ = r2; let sp = t[p..].len() - t[p..].trim_start_matches(' ').len(); p += sp.min(1); } else { break; } 
               let sp2 = lead(&t[p..]); if t[p + sp2..].starts_with('>') { p += sp2; } else { break; } }
        pos = p; base = Some(Kind::Quote);
    }
    // Listenzeichen und Aufgabenkästchen
    let rest_ind = lead(&t[pos..]); let b2 = &t[pos + rest_ind..];
    if let Some(m) = list_marker(b2) {
        let (a, z) = (pos + rest_ind, pos + rest_ind + m);
        out.push((a, z, Kind::ListMark)); pos = z + 1;
        let r = &t[pos.min(t.len())..];
        if r.len() >= 3 && r.starts_with('[') && matches!(r.as_bytes()[1], b' ' | b'x' | b'X') && r.as_bytes()[2] == b']' && (r.len() == 3 || r.as_bytes()[3] == b' ') { out.push((pos, pos + 3, Kind::ListMark)); pos += 3; }
        base = base.or(None);
    }
    let pos = pos.min(t.len());
    // Tabellenzeile: `|` als Zeichen, Trennzeile ganz gedämpft
    let tr = t[pos..].trim();
    if tr.starts_with('|') || (tr.contains('|') && is_table_sep(tr)) {
        if is_table_sep(tr) { out.push((pos + lead(&t[pos..]), t.trim_end().len(), Kind::Marker)); return out; }
        inline(t, pos, t.len(), Kind::Table, &mut out, true);
        return out;
    }
    inline(t, pos, t.len(), base.unwrap_or(Kind::Marker), &mut out, false);
    if base == Some(Kind::Quote) { fill_quote(t, pos, &mut out); }
    out
}

fn is_table_sep(s: &str) -> bool { let s = s.trim(); s.contains('-') && s.contains('|') && s.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) }
fn is_list_start(b: &str) -> bool { list_marker(b).is_some() }
/// Länge des Listenzeichens (`-`, `*`, `+`, `12.`, `3)`) am Anfang, wenn ein Leerzeichen folgt
fn list_marker(b: &str) -> Option<usize> {
    let by = b.as_bytes();
    if by.len() >= 2 && matches!(by[0], b'-' | b'*' | b'+') && by[1] == b' ' { return Some(1); }
    let d = by.iter().take_while(|c| c.is_ascii_digit()).count();
    if (1..=9).contains(&d) && by.len() > d + 1 && matches!(by[d], b'.' | b')') && by[d + 1] == b' ' { return Some(d + 1); }
    None
}

/// Im Zitat bekommt gewöhnlicher Text die Art `Quote`
fn fill_quote(t: &str, from: usize, out: &mut Vec<Span>) {
    let mut res: Vec<Span> = vec![]; let mut p = from;
    let mut sorted: Vec<Span> = out.iter().copied().filter(|s| s.0 >= from).collect(); sorted.sort_by_key(|s| s.0);
    let before: Vec<Span> = out.iter().copied().filter(|s| s.0 < from).collect();
    for s in sorted { if s.0 > p { res.push((p, s.0, Kind::Quote)); } res.push(s); p = s.1; }
    if p < t.trim_end().len() { res.push((p, t.trim_end().len(), Kind::Quote)); }
    *out = before; out.extend(res);
}

/// Inline-Spannen in `t[a..z]`. `plain_kind`: Art für Text ohne Auszeichnung (nur bei Tabellen gefärbt); `bars`: `|` als Markierung
fn inline(t: &str, a: usize, z: usize, plain_kind: Kind, out: &mut Vec<Span>, bars: bool) {
    let b = t.as_bytes(); let mut i = a;
    let put = |s: usize, e: usize, k: Kind, out: &mut Vec<Span>| { if e > s { out.push((s, e, k)); } };
    let _ = plain_kind;
    while i < z {
        let c = b[i];
        match c {
            b'\\' => { i += 2; }
            b'|' if bars => { put(i, i + 1, Kind::Marker, out); i += 1; }
            b'`' => {
                let run = b[i..z].iter().take_while(|c| **c == b'`').count();
                if let Some(close) = find_run(b, i + run, z, b'`', run) { put(i, i + run, Kind::Marker, out); put(i + run, close, Kind::Code, out); put(close, close + run, Kind::Marker, out); i = close + run; } else { i += run; }
            }
            b'*' | b'_' | b'~' => {
                let run = b[i..z].iter().take_while(|x| **x == c).count();
                let (want, kind) = match (c, run) { (b'~', 2..) => (2, Kind::Strike), (b'~', _) => { i += run; continue; } (_, 2..) => (2, Kind::Bold), _ => (1, Kind::Italic) };
                let after = b.get(i + want).copied(); let prev = if i > a { Some(b[i - 1]) } else { None };
                let opens = after.map_or(false, |x| !x.is_ascii_whitespace()) && (c != b'_' || prev.map_or(true, |p| !p.is_ascii_alphanumeric()));
                if opens && run >= want {
                    if let Some(close) = find_close(b, i + want, z, c, want) {
                        put(i, i + want, Kind::Marker, out); put(i + want, close, kind, out); put(close, close + want, Kind::Marker, out); i = close + want; continue;
                    }
                }
                i += run;
            }
            b'[' | b'!' => {
                let img = c == b'!'; if img && b.get(i + 1) != Some(&b'[') { i += 1; continue; }
                let o = if img { i + 1 } else { i };
                if let Some(cl) = find_bracket(b, o, z) { if b.get(cl + 1) == Some(&b'(') { if let Some(pe) = find_paren(b, cl + 1, z) {
                    put(i, o + 1, Kind::Marker, out); put(o + 1, cl, Kind::LinkText, out); put(cl, cl + 2, Kind::Marker, out);
                    let url_end = url_len(&t[cl + 2..pe]) + cl + 2;
                    put(cl + 2, url_end, Kind::Url, out); put(url_end, pe, Kind::Marker, out); put(pe, pe + 1, Kind::Marker, out); i = pe + 1; continue;
                } } }
                i += 1;
            }
            b'<' => {
                if let Some(e) = t[i..z].find('>') { let inner = &t[i + 1..i + e]; if (inner.starts_with("http://") || inner.starts_with("https://") || inner.starts_with("mailto:")) && !inner.contains(' ') { put(i, i + 1, Kind::Marker, out); put(i + 1, i + e, Kind::Url, out); put(i + e, i + e + 1, Kind::Marker, out); i += e + 1; continue; } }
                i += 1;
            }
            _ => { i += 1; }
        }
    }
}

fn url_len(inside: &str) -> usize { inside.find(" \"").or_else(|| inside.find(" '")).unwrap_or(inside.len()) }
fn find_run(b: &[u8], from: usize, z: usize, ch: u8, len: usize) -> Option<usize> {
    let mut i = from;
    while i < z { if b[i] == ch { let r = b[i..z].iter().take_while(|x| **x == ch).count(); if r == len { return Some(i); } i += r; } else { i += 1; } }
    None
}
fn find_close(b: &[u8], from: usize, z: usize, ch: u8, want: usize) -> Option<usize> {
    let mut i = from;
    while i < z {
        if b[i] == b'\\' { i += 2; continue; }
        if b[i] == b'`' { let r = b[i..z].iter().take_while(|x| **x == b'`').count(); match find_run(b, i + r, z, b'`', r) { Some(c) => { i = c + r; continue; } None => { i += r; continue; } } }
        if b[i] == ch { let r = b[i..z].iter().take_while(|x| **x == ch).count(); if r >= want && i > from && !b[i - 1].is_ascii_whitespace() && (ch != b'_' || b.get(i + want).map_or(true, |n| !n.is_ascii_alphanumeric())) { return Some(i); } i += r; } else { i += 1; }
    }
    None
}
fn find_bracket(b: &[u8], open: usize, z: usize) -> Option<usize> {
    let mut d = 0; let mut i = open;
    while i < z { match b[i] { b'\\' => i += 1, b'[' => d += 1, b']' => { d -= 1; if d == 0 { return Some(i); } } _ => {} } i += 1; }
    None
}
fn find_paren(b: &[u8], open: usize, z: usize) -> Option<usize> {
    let mut d = 0; let mut i = open;
    while i < z { match b[i] { b'\\' => i += 1, b'(' => d += 1, b')' => { d -= 1; if d == 0 { return Some(i); } } _ => {} } i += 1; }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spans(text: &str) -> Vec<(String, Kind)> {
        let mut st = State::new(text);
        text.split('\n').enumerate().flat_map(|(n, l)| { let v = line(l, n, &mut st); v.into_iter().map(|(a, z, k)| (l[a..z].to_string(), k)).collect::<Vec<_>>() }).collect()
    }
    fn has(text: &str, s: &str, k: Kind) -> bool { spans(text).iter().any(|(t, x)| t == s && *x == k) }

    #[test]
    fn headings_keep_their_marks() {
        assert!(has("## Aufgaben", "##", Kind::Marker) && has("## Aufgaben", " Aufgaben", Kind::Heading));
        assert!(has("# Titel #", " Titel ", Kind::Heading));
        assert!(!has("#kein Titel", "#", Kind::Marker) || !has("#kein Titel", "kein", Kind::Heading));
        assert!(!spans("####### sieben").iter().any(|(_, k)| *k == Kind::Heading));
    }

    #[test]
    fn inline_marks_and_content() {
        assert!(has("a **fett** b", "fett", Kind::Bold) && has("a **fett** b", "**", Kind::Marker));
        assert!(has("a *kursiv* b", "kursiv", Kind::Italic));
        assert!(has("a ~~weg~~ b", "weg", Kind::Strike));
        assert!(has("ein `code *x*` hier", "code *x*", Kind::Code) && !has("ein `code *x*` hier", "x", Kind::Italic));
        assert!(!has("a * b * c", " b ", Kind::Italic));
        assert!(!has("snake_case_name", "case", Kind::Italic));
    }

    #[test]
    fn links_images_autolinks() {
        let t = "[Text](src/a.asm \"Titel\") ![Bild](p.png) <https://x.de>";
        assert!(has(t, "Text", Kind::LinkText) && has(t, "src/a.asm", Kind::Url) && has(t, "Bild", Kind::LinkText) && has(t, "p.png", Kind::Url) && has(t, "https://x.de", Kind::Url));
        assert!(!has("[nur] Klammern", "nur", Kind::LinkText));
        assert!(has("[a](b(c)d)", "b(c)d", Kind::Url));
    }

    #[test]
    fn lists_tasks_quotes_rules_tables() {
        assert!(has("- [x] fertig", "-", Kind::ListMark) && has("- [x] fertig", "[x]", Kind::ListMark));
        assert!(has("  12. zwölf", "12.", Kind::ListMark));
        assert!(has("> Zitat mit `c`", "Zitat mit ", Kind::Quote) && has("> Zitat mit `c`", ">", Kind::Marker) && has("> Zitat mit `c`", "c", Kind::Code));
        assert!(has("> > tief", ">", Kind::Marker));
        assert!(has("---", "---", Kind::Rule) && has("* * *", "* * *", Kind::Rule));
        assert!(has("| a | b |", "|", Kind::Marker));
        assert!(has("|:--|--:|", "|:--|--:|", Kind::Marker));
    }

    #[test]
    fn fences_are_stateful() {
        let t = "text\n```asm\n# kein Titel\n**nichts**\n```\n# Titel\n~~~~\n```\n~~~~\nrest";
        let s = spans(t);
        assert!(s.contains(&("# kein Titel".into(), Kind::CodeBlock)) && s.contains(&("**nichts**".into(), Kind::CodeBlock)));
        assert!(s.contains(&(" Titel".into(), Kind::Heading)));
        assert!(s.contains(&("```".into(), Kind::CodeBlock)), "kürzerer Zaun schließt ~~~~ nicht: {s:?}");
        assert!(!s.iter().any(|(t, k)| t == "rest" && *k == Kind::CodeBlock));
    }

    #[test]
    fn front_matter_is_grey_and_html_is_marked() {
        let t = "---\ntitel: x\n---\n# H\n<div>x</div>";
        let s = spans(t);
        assert!(s.contains(&("titel: x".into(), Kind::Front)) && s.contains(&(" H".into(), Kind::Heading)) && s.contains(&("<div>x</div>".into(), Kind::Html)));
    }

    #[test]
    fn spans_are_ordered_and_inside_the_line() {
        let texts = ["", "# ä ö ü **x** `y` [é](ü)", "> - [ ] **a *b* c** ~~d~~ <https://x>", "|a|b|\n|-|-|\n|ä|`|`|", "\\*nicht\\* *doch*", "**offen", "`offen", "[offen](", "![", "```\n```x", "    - eingerückt"];
        for t in texts { let mut st = State::new(t); for (n, l) in t.split('\n').enumerate() { let mut last = 0; for (a, z, _) in line(l, n, &mut st) { assert!(a >= last && z > a && z <= l.len() && l.is_char_boundary(a) && l.is_char_boundary(z), "{l:?}"); last = z; } } }
    }
}
