// SPDX-License-Identifier: GPL-3.0-or-later
//! Markdown für den Editor: CommonMark plus Tabellen, Aufgabenlisten und Durchgestrichen.
//!
//! * [`parse`] liefert einen Blockbaum mit **Quellzeilen** je Block (Vorschau, Scrollsynchronisation, Gliederung),
//! * [`lex`] färbt eine Zeile (Zustandsautomat, Zeichen für Zeichen ohne Verlust),
//! * [`link`] prüft Links und Bildpfade (nur innerhalb des Projekts),
//! * [`edit`] sind die Schreibhilfen als reine Textfunktionen.
//!
//! Rohes HTML wird nie ausgeführt: es erscheint als Text (`Block::Html`). Front Matter am Dateianfang wird ausgeblendet.
pub mod edit;
pub mod lex;
pub mod link;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag};

/// Ausrichtung einer Tabellenspalte
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align { None, Left, Center, Right }

#[derive(Clone, PartialEq, Debug)]
pub enum Inline {
    Text(String), Code(String), Emph(Vec<Inline>), Strong(Vec<Inline>), Strike(Vec<Inline>),
    Link { url: String, title: String, kids: Vec<Inline> },
    Image { url: String, alt: String },
    SoftBreak, HardBreak,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Item { pub task: Option<bool>, pub nodes: Vec<Node> }

#[derive(Clone, PartialEq, Debug)]
pub enum Block {
    Heading { level: u8, text: Vec<Inline>, anchor: String },
    Para(Vec<Inline>),
    Quote(Vec<Node>),
    List { start: Option<u64>, items: Vec<Item> },
    Code { lang: String, text: String },
    Table { align: Vec<Align>, head: Vec<Vec<Inline>>, rows: Vec<Vec<Vec<Inline>>> },
    Rule,
    /// Rohes HTML: wird als Text gezeigt
    Html(String),
}

/// Block mit Quellzeilen (0-basiert, `lines.0` ..= `lines.1`)
#[derive(Clone, PartialEq, Debug)]
pub struct Node { pub block: Block, pub lines: (usize, usize) }

/// Ergebnis von [`parse`]
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Doc {
    pub nodes: Vec<Node>,
    /// Zeilen des Front Matters (0-basiert, einschließlich der Trennzeilen), falls vorhanden
    pub front: Option<(usize, usize)>,
}

/// Eintrag der Gliederung
#[derive(Clone, PartialEq, Debug)]
pub struct Heading { pub level: u8, pub text: String, pub line: usize, pub anchor: String }

/// Letzte Zeile des Front Matters (`---` in Zeile 1, schließendes `---` oder `...` später), sonst `None`
pub fn front_matter_end(text: &str) -> Option<usize> {
    let mut it = text.split('\n');
    if it.next()?.trim_end() != "---" { return None; }
    for (i, l) in it.enumerate() { let t = l.trim_end(); if t == "---" || t == "..." { return Some(i + 1); } }
    None
}

fn options() -> Options { Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS }

/// Text mit durch Leerzeilen ersetztem Front Matter: gleiche Zeilenzahl, also gleiche Zeilennummern
fn without_front(text: &str, end: Option<usize>) -> String {
    let Some(e) = end else { return text.to_string() };
    let mut s = String::with_capacity(text.len());
    for (i, l) in text.split('\n').enumerate() { if i > 0 { s.push('\n'); } if i > e { s.push_str(l); } }
    s
}

struct Builder<'a> { ev: Vec<(Event<'a>, std::ops::Range<usize>)>, i: usize, starts: Vec<usize>, anchors: std::collections::HashMap<String, usize> }

impl<'a> Builder<'a> {
    fn line(&self, byte: usize) -> usize { match self.starts.binary_search(&byte) { Ok(i) => i, Err(i) => i.saturating_sub(1) } }
    fn lines(&self, r: &std::ops::Range<usize>) -> (usize, usize) { (self.line(r.start), self.line(r.end.saturating_sub(1).max(r.start))) }
    fn is_inline_start(e: &Event) -> bool {
        match e {
            Event::Text(_) | Event::Code(_) | Event::SoftBreak | Event::HardBreak | Event::InlineHtml(_) => true,
            Event::Start(t) => matches!(t, Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. }),
            _ => false,
        }
    }
    /// Inline-Inhalt bis zum Ende des umgebenden Elements (`stop`: nächstes `End`, wird nicht verbraucht)
    fn inlines(&mut self, until_end: bool) -> Vec<Inline> {
        let mut out: Vec<Inline> = vec![];
        let push_text = |out: &mut Vec<Inline>, s: &str| { if let Some(Inline::Text(t)) = out.last_mut() { t.push_str(s); } else { out.push(Inline::Text(s.to_string())); } };
        while self.i < self.ev.len() {
            let e = self.ev[self.i].0.clone();
            match e {
                Event::End(_) if until_end => break,
                Event::Text(s) | Event::InlineHtml(s) | Event::Html(s) => { push_text(&mut out, &s); self.i += 1; }
                Event::Code(s) => { out.push(Inline::Code(s.to_string())); self.i += 1; }
                Event::SoftBreak => { out.push(Inline::SoftBreak); self.i += 1; }
                Event::HardBreak => { out.push(Inline::HardBreak); self.i += 1; }
                Event::Start(Tag::Emphasis) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Inline::Emph(k)); }
                Event::Start(Tag::Strong) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Inline::Strong(k)); }
                Event::Start(Tag::Strikethrough) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Inline::Strike(k)); }
                Event::Start(Tag::Link { dest_url, title, .. }) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Inline::Link { url: dest_url.to_string(), title: title.to_string(), kids: k }); }
                Event::Start(Tag::Image { dest_url, .. }) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Inline::Image { url: dest_url.to_string(), alt: plain(&k) }); }
                _ if !until_end && !Self::is_inline_start(&e) => break,
                _ => { self.i += 1; }
            }
        }
        out
    }
    fn blocks(&mut self) -> Vec<Node> {
        let mut out = vec![];
        while self.i < self.ev.len() {
            let (e, r) = (self.ev[self.i].0.clone(), self.ev[self.i].1.clone());
            match e {
                Event::End(_) => break,
                Event::Start(Tag::Paragraph) => { self.i += 1; let k = self.inlines(true); self.i += 1; out.push(Node { block: Block::Para(k), lines: self.lines(&r) }); }
                Event::Start(Tag::Heading { level, .. }) => {
                    self.i += 1; let k = self.inlines(true); self.i += 1;
                    let anchor = self.anchor(&plain(&k));
                    out.push(Node { block: Block::Heading { level: level as u8, text: k, anchor }, lines: self.lines(&r) });
                }
                Event::Start(Tag::BlockQuote(_)) => { self.i += 1; let k = self.blocks(); self.i += 1; out.push(Node { block: Block::Quote(k), lines: self.lines(&r) }); }
                Event::Start(Tag::CodeBlock(kind)) => {
                    self.i += 1; let mut t = String::new();
                    while self.i < self.ev.len() { match &self.ev[self.i].0 { Event::End(_) => break, Event::Text(s) => t.push_str(s), _ => {} } self.i += 1; }
                    self.i += 1;
                    let lang = match kind { CodeBlockKind::Fenced(l) => l.split_whitespace().next().unwrap_or("").to_string(), _ => String::new() };
                    out.push(Node { block: Block::Code { lang, text: t.trim_end_matches('\n').to_string() }, lines: self.lines(&r) });
                }
                Event::Start(Tag::HtmlBlock) => {
                    self.i += 1; let mut t = String::new();
                    while self.i < self.ev.len() { match &self.ev[self.i].0 { Event::End(_) => break, Event::Html(s) | Event::Text(s) => t.push_str(s), _ => {} } self.i += 1; }
                    self.i += 1;
                    out.push(Node { block: Block::Html(t.trim_end_matches('\n').to_string()), lines: self.lines(&r) });
                }
                Event::Start(Tag::List(start)) => {
                    self.i += 1; let mut items = vec![];
                    while self.i < self.ev.len() {
                        match &self.ev[self.i].0 {
                            Event::Start(Tag::Item) => {
                                let ir = self.ev[self.i].1.clone(); self.i += 1;
                                let task = if let Some((Event::TaskListMarker(b), _)) = self.ev.get(self.i) { let b = *b; self.i += 1; Some(b) } else { None };
                                let mut nodes = self.blocks(); self.i += 1;
                                if let Some(f) = nodes.first_mut() { f.lines.0 = f.lines.0.min(self.line(ir.start)); }
                                items.push(Item { task, nodes });
                            }
                            Event::End(_) => break,
                            _ => self.i += 1,
                        }
                    }
                    self.i += 1; out.push(Node { block: Block::List { start, items }, lines: self.lines(&r) });
                }
                Event::Start(Tag::Table(al)) => {
                    self.i += 1;
                    let align = al.iter().map(|a| match a { Alignment::Left => Align::Left, Alignment::Center => Align::Center, Alignment::Right => Align::Right, Alignment::None => Align::None }).collect();
                    let (mut head, mut rows) = (vec![], vec![]);
                    while self.i < self.ev.len() {
                        match &self.ev[self.i].0 {
                            Event::Start(Tag::TableHead) => { self.i += 1; head = self.cells(); self.i += 1; }
                            Event::Start(Tag::TableRow) => { self.i += 1; rows.push(self.cells()); self.i += 1; }
                            Event::End(_) => break,
                            _ => self.i += 1,
                        }
                    }
                    self.i += 1; out.push(Node { block: Block::Table { align, head, rows }, lines: self.lines(&r) });
                }
                Event::Rule => { self.i += 1; out.push(Node { block: Block::Rule, lines: self.lines(&r) }); }
                ev if Self::is_inline_start(&ev) => {
                    // lose Listenpunkte: Inline-Inhalt ohne Absatz
                    let start = r.start; let k = self.inlines(false);
                    let end = self.ev.get(self.i.wrapping_sub(1)).map_or(start, |x| x.1.end);
                    out.push(Node { block: Block::Para(k), lines: self.lines(&(start..end.max(start + 1))) });
                }
                _ => self.i += 1,
            }
        }
        out
    }
    fn cells(&mut self) -> Vec<Vec<Inline>> {
        let mut cells = vec![];
        while self.i < self.ev.len() {
            match &self.ev[self.i].0 {
                Event::Start(Tag::TableCell) => { self.i += 1; cells.push(self.inlines(true)); self.i += 1; }
                Event::End(_) => break,
                _ => self.i += 1,
            }
        }
        cells
    }
    fn anchor(&mut self, text: &str) -> String {
        let base = slug(text); let n = self.anchors.entry(base.clone()).or_insert(0);
        let a = if *n == 0 { base } else { format!("{base}-{n}") }; *n += 1; a
    }
}

/// Reiner Text eines Inline-Inhalts
pub fn plain(k: &[Inline]) -> String {
    let mut s = String::new();
    for i in k { match i {
        Inline::Text(t) | Inline::Code(t) => s.push_str(t),
        Inline::Emph(k) | Inline::Strong(k) | Inline::Strike(k) | Inline::Link { kids: k, .. } => s.push_str(&plain(k)),
        Inline::Image { alt, .. } => s.push_str(alt),
        Inline::SoftBreak | Inline::HardBreak => s.push(' '),
    } }
    s
}

/// Ankername einer Überschrift (wie auf GitHub: klein, ohne Satzzeichen, Leerzeichen zu `-`, Umlaute bleiben)
pub fn slug(text: &str) -> String {
    text.trim().chars().filter_map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { Some(c.to_lowercase().next().unwrap_or(c)) } else if c == ' ' { Some('-') } else { None }).collect()
}

/// Markdown zerlegen
pub fn parse(text: &str) -> Doc {
    let front_end = front_matter_end(text);
    let body = without_front(text, front_end);
    let mut starts = vec![0usize];
    for (i, b) in body.bytes().enumerate() { if b == b'\n' { starts.push(i + 1); } }
    let ev: Vec<_> = Parser::new_ext(&body, options()).into_offset_iter().collect();
    let mut b = Builder { ev, i: 0, starts, anchors: Default::default() };
    let mut nodes = vec![];
    while b.i < b.ev.len() { let mut part = b.blocks(); nodes.append(&mut part); if b.i < b.ev.len() { b.i += 1; } }
    Doc { nodes, front: front_end.map(|e| (0, e)) }
}

/// Alle Überschriften (auch in Zitaten und Listen) mit Zeile und Ankername; Überschriften in Codeblöcken zählen nicht
pub fn headings(text: &str) -> Vec<Heading> {
    fn walk(nodes: &[Node], out: &mut Vec<Heading>) {
        for n in nodes { match &n.block {
            Block::Heading { level, text, anchor } => out.push(Heading { level: *level, text: plain(text), line: n.lines.0, anchor: anchor.clone() }),
            Block::Quote(k) => walk(k, out),
            Block::List { items, .. } => for it in items { walk(&it.nodes, out) },
            _ => {}
        } }
    }
    let mut out = vec![]; walk(&parse(text).nodes, &mut out); out
}

/// Zeilenverweis im Fließtext wie `datei.asm:14`
#[derive(Clone, PartialEq, Debug)]
pub struct LineRef { pub start: usize, pub end: usize, pub path: String, pub line: usize }

/// Findet `pfad/datei.ext:zeile` in einem Text (Byte-Bereiche); Webadressen und Uhrzeiten zählen nicht
pub fn find_line_refs(s: &str) -> Vec<LineRef> {
    let mut out = vec![]; let b = s.as_bytes(); let mut i = 0;
    let pc = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'.' | b'/' | b'-');
    while i < b.len() {
        if !pc(b[i]) || (i > 0 && (pc(b[i - 1]) || b[i - 1] == b':')) { i += 1; continue; }
        let mut j = i; while j < b.len() && pc(b[j]) { j += 1; }
        if j < b.len() && b[j] == b':' {
            let mut k = j + 1; while k < b.len() && b[k].is_ascii_digit() { k += 1; }
            let path = &s[i..j]; let digits = &s[j + 1..k];
            let ext = path.rsplit_once('.').map(|x| x.1).unwrap_or("");
            let ok = !digits.is_empty() && digits.len() <= 7 && !ext.is_empty() && ext.len() <= 6 && ext.bytes().all(|c| c.is_ascii_alphanumeric()) && ext.bytes().any(|c| c.is_ascii_alphabetic())
                && !path.starts_with('.') && !path.ends_with('.') && !(k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_'));
            if ok { out.push(LineRef { start: i, end: k, path: path.to_string(), line: digits.parse().unwrap_or(0) }); i = k; continue; }
        }
        i = j.max(i + 1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn blocks(t: &str) -> Vec<Block> { parse(t).nodes.into_iter().map(|n| n.block).collect() }

    #[test]
    fn headings_paragraphs_and_lines() {
        let d = parse("# Titel\n\nText eins\nzweite Zeile\n\n## Unter\n");
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.nodes[0].lines, (0, 0));
        assert_eq!(d.nodes[1].lines, (2, 3));
        assert!(matches!(&d.nodes[2].block, Block::Heading { level: 2, .. }));
        assert_eq!(d.nodes[2].lines.0, 5);
    }

    #[test]
    fn inline_formats_and_links() {
        let Block::Para(k) = &blocks("a **fett** *kursiv* ~~weg~~ `c` [t](u \"ti\") ![alt](p.png)")[0] else { panic!() };
        assert!(k.iter().any(|i| matches!(i, Inline::Strong(x) if plain(x) == "fett")));
        assert!(k.iter().any(|i| matches!(i, Inline::Emph(x) if plain(x) == "kursiv")));
        assert!(k.iter().any(|i| matches!(i, Inline::Strike(x) if plain(x) == "weg")));
        assert!(k.iter().any(|i| matches!(i, Inline::Code(c) if c == "c")));
        assert!(k.iter().any(|i| matches!(i, Inline::Link { url, title, .. } if url == "u" && title == "ti")));
        assert!(k.iter().any(|i| matches!(i, Inline::Image { url, alt } if url == "p.png" && alt == "alt")));
    }

    #[test]
    fn nested_and_task_lists_keep_lines() {
        let d = parse("- [x] eins\n- [ ] zwei\n  - tief\n1. a\n2. b\n");
        let Block::List { items, start: None } = &d.nodes[0].block else { panic!("{:?}", d.nodes) };
        assert_eq!(items[0].task, Some(true)); assert_eq!(items[1].task, Some(false));
        assert!(items[1].nodes.iter().any(|n| matches!(&n.block, Block::List { .. })));
        assert_eq!(d.nodes[0].lines, (0, 2));
        assert!(matches!(&d.nodes[1].block, Block::List { start: Some(1), .. }));
    }

    #[test]
    fn code_tables_quotes_rules_and_html() {
        let t = "```asm\nlda #1\n```\n\n| a | b |\n|:--|--:|\n| 1 | 2 |\n\n> zitat\n\n---\n\n<div>roh</div>\n";
        let b = blocks(t);
        assert!(matches!(&b[0], Block::Code { lang, text } if lang == "asm" && text == "lda #1"));
        let Block::Table { align, head, rows } = &b[1] else { panic!("{b:?}") };
        assert_eq!(align, &vec![Align::Left, Align::Right]); assert_eq!(head.len(), 2); assert_eq!(rows.len(), 1);
        assert!(matches!(&b[2], Block::Quote(_)) && matches!(&b[3], Block::Rule));
        assert!(matches!(&b[4], Block::Html(h) if h.contains("<div>")), "{b:?}");
    }

    #[test]
    fn front_matter_is_hidden_but_lines_stay() {
        let t = "---\ntitel: x\n---\n# Ü\n";
        assert_eq!(front_matter_end(t), Some(2));
        let d = parse(t);
        assert_eq!(d.front, Some((0, 2)));
        assert!(matches!(&d.nodes[0].block, Block::Heading { .. }) && d.nodes[0].lines.0 == 3, "{:?}", d.nodes);
        assert_eq!(front_matter_end("text\n---\n"), None);
        assert_eq!(front_matter_end("---\nohne ende\n"), None);
    }

    #[test]
    fn heading_list_skips_code_and_makes_unique_anchors() {
        let h = headings("# Raster IRQ\n```\n# kein Titel\n```\n## Größe\n## Größe\n#kein\n");
        assert_eq!(h.iter().map(|x| (x.level, x.text.as_str(), x.line)).collect::<Vec<_>>(), vec![(1, "Raster IRQ", 0), (2, "Größe", 4), (2, "Größe", 5)]);
        assert_eq!(h[0].anchor, "raster-irq"); assert_eq!(h[1].anchor, "größe"); assert_eq!(h[2].anchor, "größe-1");
    }

    #[test]
    fn line_refs_in_text() {
        let r = find_line_refs("siehe src/irq.asm:14, auch main.bas:20. Zeit 12:30 und http://x.de:80/a.html");
        assert_eq!(r.iter().map(|x| (x.path.as_str(), x.line)).collect::<Vec<_>>(), vec![("src/irq.asm", 14), ("main.bas", 20)]);
        assert_eq!(&"siehe src/irq.asm:14, x"[r[0].start..r[0].end], "src/irq.asm:14");
        assert!(find_line_refs("v1.2:3 und a.b:").is_empty());
    }

    #[test]
    fn robust_against_odd_input() {
        for t in ["", ">", "|", "|||\n|-|", "- ", "1.", "```", "~~~~\n", "![", "[a](", "\n\n\n", "# ", "---", "<", "* * *", "- [ ]", "> - > 1. ```"] { let _ = parse(t); let _ = headings(t); }
        let big = "- a\n".repeat(5000) + &"> ".repeat(100) + "x\n";
        let _ = parse(&big);
    }
}
