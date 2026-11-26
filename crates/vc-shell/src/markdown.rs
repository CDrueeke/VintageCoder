// SPDX-License-Identifier: GPL-3.0-or-later
//! Markdown (E13): Vorschau als Blockbaum mit Quellzeilen und die Schreibhilfen als reine Textfunktionen (`vc_md`).
//! Rohes HTML wird nie ausgeführt (es erscheint als Text); Links und Bilder zeigen nur ins Projekt, ins Web (Browser des Systems) oder auf Anker.
use serde_json::{json, Value};
use std::path::Path;
use vc_md::{edit, link, Align, Block, Inline, Node};

/// Bildadresse für eine Projektdatei (PNG/JPEG/GIF/WebP bis 600 KB) als Datenadresse; sonst `null` (die Oberfläche zeigt den Alt-Text)
fn bild_daten(root: &Path, rel: &str) -> Value {
    let Ok(p) = crate::dateien::absolut(root, rel) else { return Value::Null };
    let Ok(m) = std::fs::metadata(&p) else { return Value::Null }; if !m.is_file() || m.len() > 600_000 { return Value::Null }
    let Ok(b) = std::fs::read(&p) else { return Value::Null };
    let typ = if b.starts_with(&[0x89, b'P', b'N', b'G']) { "image/png" } else if b.starts_with(&[0xFF, 0xD8]) { "image/jpeg" } else if b.starts_with(b"GIF8") { "image/gif" } else if b.len() > 12 && &b[8..12] == b"WEBP" { "image/webp" } else { return Value::Null };
    json!(format!("data:{typ};base64,{}", crate::hilfe::base64(&b)))
}

fn ziel(base: &str, url: &str) -> Value {
    match link::resolve(base, url) {
        link::Link::Web(u) => json!({ "kind": "web", "url": u }),
        link::Link::Anchor(a) => json!({ "kind": "anchor", "anchor": a }),
        link::Link::File { rel, line } => json!({ "kind": "file", "rel": rel, "line": line }),
        link::Link::Blocked(why) => json!({ "kind": "blocked", "why": why }),
    }
}

fn inlines(root: &Path, base: &str, k: &[Inline]) -> Vec<Value> {
    k.iter().map(|i| match i {
        Inline::Text(s) => json!({ "t": "text", "s": s }),
        Inline::Code(s) => json!({ "t": "code", "s": s }),
        Inline::Emph(v) => json!({ "t": "em", "k": inlines(root, base, v) }),
        Inline::Strong(v) => json!({ "t": "strong", "k": inlines(root, base, v) }),
        Inline::Strike(v) => json!({ "t": "strike", "k": inlines(root, base, v) }),
        Inline::Link { url, title, kids } => json!({ "t": "link", "title": title, "to": ziel(base, url), "k": inlines(root, base, kids) }),
        Inline::Image { url, alt } => { let to = ziel(base, url); let img = if to["kind"] == "file" { bild_daten(root, to["rel"].as_str().unwrap_or("")) } else { Value::Null }; json!({ "t": "image", "alt": alt, "to": to, "img": img }) }
        Inline::SoftBreak => json!({ "t": "soft" }),
        Inline::HardBreak => json!({ "t": "br" }),
    }).collect()
}

fn knoten(root: &Path, base: &str, n: &Node) -> Value {
    let l = json!([n.lines.0, n.lines.1]);
    match &n.block {
        Block::Heading { level, text, anchor } => json!({ "b": "heading", "level": level, "anchor": anchor, "k": inlines(root, base, text), "lines": l }),
        Block::Para(k) => json!({ "b": "para", "k": inlines(root, base, k), "lines": l }),
        Block::Quote(v) => json!({ "b": "quote", "nodes": v.iter().map(|x| knoten(root, base, x)).collect::<Vec<_>>(), "lines": l }),
        Block::List { start, items } => json!({ "b": "list", "start": start, "lines": l, "items": items.iter().map(|it| json!({ "task": it.task, "nodes": it.nodes.iter().map(|x| knoten(root, base, x)).collect::<Vec<_>>() })).collect::<Vec<_>>() }),
        Block::Code { lang, text } => json!({ "b": "code", "lang": lang, "text": text, "lines": l }),
        Block::Table { align, head, rows } => json!({ "b": "table", "lines": l,
            "align": align.iter().map(|a| match a { Align::Left => "left", Align::Center => "center", Align::Right => "right", Align::None => "" }).collect::<Vec<_>>(),
            "head": head.iter().map(|c| inlines(root, base, c)).collect::<Vec<_>>(), "rows": rows.iter().map(|r| r.iter().map(|c| inlines(root, base, c)).collect::<Vec<_>>()).collect::<Vec<_>>() }),
        Block::Rule => json!({ "b": "rule", "lines": l }),
        Block::Html(s) => json!({ "b": "html", "text": s, "lines": l }),
    }
}

/// Vorschau: Blöcke ab `von` (höchstens `anzahl`), dazu die Gesamtzahl (lange Dokumente erscheinen blockweise) und die Gliederung
pub fn render(root: &Path, rel: &str, text: &str, von: usize, anzahl: usize) -> Value {
    let doc = vc_md::parse(text);
    let nodes: Vec<Value> = doc.nodes.iter().skip(von).take(anzahl).map(|n| knoten(root, rel, n)).collect();
    json!({ "nodes": nodes, "total": doc.nodes.len(), "front": doc.front.map(|f| json!([f.0, f.1])),
        "outline": vc_md::headings(text).iter().map(|h| json!({ "level": h.level, "text": h.text, "line": h.line, "anchor": h.anchor })).collect::<Vec<_>>() })
}

/// Web-Link der Vorschau im Browser des Systems öffnen (nur auf Klick; nur http, https und mailto)
pub fn adresse_oeffnen(url: &str) -> Result<(), crate::docstore::Fehler> {
    if !matches!(link::resolve("", url), link::Link::Web(_)) { return Err(crate::docstore::Fehler::neu("op-unguelt")); }
    if crate::bau::gesperrt() { return Ok(()); }
    let (prog, args) = vc_ide::feedback::open_command(std::env::consts::OS, url);
    std::process::Command::new(prog).args(args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ()).map_err(|e| crate::docstore::Fehler::mit("datei-fehler", json!({ "detail": e.to_string() })))
}

/// Schreibhilfe: `op` auf `text` an der Auswahl `a..b` (Zeichenindizes); Antwort: neuer Text und neue Auswahl, oder `null`, wenn die Hilfe hier nichts tut
pub fn bearbeiten(op: &str, text: &str, a: usize, b: usize, arg: &str) -> Value {
    let e = match op {
        "bold" => Some(edit::toggle_wrap(text, a, b, "**")), "italic" => Some(edit::toggle_wrap(text, a, b, "*")), "strike" => Some(edit::toggle_wrap(text, a, b, "~~")),
        "code" => Some(edit::code_toggle(text, a, b)), "link" => Some(edit::link_insert(text, a, b)), "image" => Some(edit::image_insert(text, a, b, if arg.is_empty() { "bild.png" } else { arg })),
        "deeper" => Some(edit::heading_shift(text, a, b, 1)), "higher" => Some(edit::heading_shift(text, a, b, -1)),
        "bullet" => Some(edit::list_toggle(text, a, b, edit::ListKind::Bullet)), "numbered" => Some(edit::list_toggle(text, a, b, edit::ListKind::Numbered)),
        "tasks" => Some(edit::list_toggle(text, a, b, edit::ListKind::Task)), "quote" => Some(edit::list_toggle(text, a, b, edit::ListKind::Quote)),
        "task" => edit::toggle_task(text, a),
        "table" => { let blk = edit::table_new(3, 2, true, 0); Some(edit::insert_block(text, a, &blk, 2)) }
        "align" => edit::table_align(text, a),
        "list_continue" => edit::continue_list(text, a),
        "indent" => edit::indent(text, a, b, false), "outdent" => edit::indent(text, a, b, true),
        "cell_next" => edit::table_cell(text, a, false), "cell_back" => edit::table_cell(text, a, true),
        _ => None,
    };
    match e { Some(e) => json!({ "text": e.text, "a": e.a, "b": e.b }), None => Value::Null }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn vorschau_und_schreibhilfen() {
        let d = std::env::temp_dir().join(format!("vc-md-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("img")).unwrap();
        std::fs::write(d.join("img/a.png"), [0x89, b'P', b'N', b'G', 1, 2, 3]).unwrap();
        let t = "# Titel\n\nText **fett** und `code` mit [Web](https://example.org), [Datei](x.asm:3) und ![B](img/a.png) ![C](../../außen.png)\n\n- [ ] eins\n- [x] zwei\n\n| a | b |\n|---|--:|\n| 1 | 2 |\n";
        let v = render(&d, "doc.md", t, 0, 50);
        assert_eq!(v["total"], 4); assert_eq!(v["nodes"][0]["b"], "heading"); assert_eq!(v["nodes"][1]["b"], "para");
        let k = v["nodes"][1]["k"].as_array().unwrap();
        assert!(k.iter().any(|x| x["t"] == "link" && x["to"]["kind"] == "web"));
        assert!(k.iter().any(|x| x["t"] == "link" && x["to"]["kind"] == "file" && x["to"]["rel"] == "x.asm" && x["to"]["line"] == 3));
        assert!(k.iter().any(|x| x["t"] == "image" && x["img"].as_str().is_some_and(|s| s.starts_with("data:image/png;base64,"))));
        assert!(k.iter().any(|x| x["t"] == "image" && x["to"]["kind"] == "blocked" && x["img"].is_null()), "außerhalb des Projekts bleibt gesperrt");
        assert_eq!(v["nodes"][2]["items"][1]["task"], true); assert_eq!(v["nodes"][3]["align"][1], "right");
        assert_eq!(render(&d, "doc.md", t, 2, 1)["nodes"].as_array().unwrap().len(), 1, "blockweise");
        assert_eq!(bearbeiten("bold", "welt", 0, 4, "")["text"], "**welt**");
        assert_eq!(bearbeiten("deeper", "x", 0, 1, "")["text"], "# x");
        assert!(bearbeiten("task", "text", 0, 0, "").is_null(), "keine Listenzeile");
        assert_eq!(bearbeiten("task", "- eins", 2, 2, "")["text"], "- [ ] eins");
        assert!(bearbeiten("unbekannt", "x", 0, 0, "").is_null());
        assert!(adresse_oeffnen("file:///etc/passwd").is_err() && adresse_oeffnen("javascript:alert(1)").is_err() && adresse_oeffnen("https://example.org").is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }
}
