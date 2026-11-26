// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprachdienste für natives BASIC (Punkt „Sprachdienste B7“): Dialekt der Datei, `#define`-Namen, durch `#if` ausgeschlossene Zeilen, Marken (Gliederung)
//! und die Hover-Karte für `#define`-Namen und Direktiven. Rechnet aus dem Text des Editors (auch ungespeichert); schreibt nichts.
use serde_json::{json, Value};
use std::path::Path;

pub struct Req<'a> { pub root: &'a Path, pub data: &'a Path, pub rel: &'a str, pub text: &'a str }

/// Zielsystem und aufgelöster Dialekt der Datei (Projekt, Zielsystem, `#dialect` der Datei)
fn context(r: &Req) -> (String, Option<vc_basic::dialects::DialectInfo>) {
    let pf = crate::projfile::find(r.root).and_then(|f| crate::projfile::ProjectFile::load(&f).ok());
    let tid = pf.as_ref().and_then(|p| p.target_of(r.rel)).or_else(|| crate::build::detect_target(r.text)).unwrap_or_else(|| vc_target::default_target_or_c64(Some(r.data)));
    let tdialect = vc_target::load_target(r.data, &tid).ok().and_then(|t| t.basic_dialect);
    let did = pf.as_ref().and_then(|p| p.dialect_of(r.rel)).or(tdialect).unwrap_or_else(|| "cbm_v2".into());
    let abs = r.root.join(r.rel).to_string_lossy().into_owned();
    let id = vc_basic::dialects::resolve(r.text, &abs, &did, Some(&tid), r.data).map(|x| x.id).unwrap_or(did);
    (tid, vc_basic::dialects::info(r.data, &id).ok())
}

/// Leitet `'` im Dialekt `id` einen Kommentar ein? (Commodore: nein, Warnung `B167`; Atari: ja)
pub fn quote_comment(data: &Path, id: &str) -> bool { vc_basic::dialects::info(data, id).map_or(true, |i| i.quote_comment) }

/// Marken `@name:` am Zeilenanfang (nach einer Zeilennummer): (Name, Zeile ab 0)
fn labels(text: &str) -> Vec<(String, usize)> {
    let mut out = vec![];
    for (i, l) in text.lines().enumerate() {
        let mut t = l.trim_start();
        t = t.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start();
        while let Some(rest) = t.strip_prefix('@') {
            let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
            let (name, after) = (&rest[..end], rest[end..].trim_start());
            if name.is_empty() || !after.starts_with(':') { break; }
            out.push((name.to_ascii_lowercase(), i));
            t = after[1..].trim_start();
        }
    }
    out
}

fn pre(r: &Req, tid: &str) -> Option<vc_basic::preprocess::Preprocessed> {
    let mut o = vc_basic::preprocess::PreOptions::new(r.root.join(r.rel).to_string_lossy());
    o.system = Some(tid.to_string());
    o.defines = vc_target::project_defines(r.root, tid).into_iter().map(|(n, v)| (n, v.to_string())).collect();
    // `#asm`/`#blob` meldet der Vorverarbeitungspass als Fehler (er läuft ohne Assembler); für die Anzeige werden die Zeilen leer gesetzt (gleiche Zeilenzahl)
    let text = r.text.lines().map(|l| if vc_basic::blob::is_blob_directive(l) || vc_basic::blob::is_old_blob_directive(l) { "" } else { l }).collect::<Vec<_>>().join("\n");
    vc_basic::preprocess::run(&text, &o).ok()
}

/// `{ dialect: {id, name, family}, system, defines: [{name, value, line}], excluded: [Zeile ab 0], outline: [{name, line, kind}] }`
pub fn info(r: &Req) -> Value {
    let (tid, di) = context(r);
    let p = pre(r, &tid);
    let defines: Vec<Value> = p.as_ref().map(|p| p.defines.iter().map(|d| json!({ "name": d.name, "value": d.value, "line": d.line.saturating_sub(1) })).collect()).unwrap_or_default();
    // `excluded` enthält auch die Direktivenzeilen selbst (`#define`, `#if`, `#else`, `#endif`); grau werden nur die ausgeschlossenen Programmzeilen
    let lines: Vec<&str> = r.text.lines().collect();
    let is_pre = |l: &str| { let t = l.trim().to_ascii_lowercase(); ["#define", "#if", "#else", "#endif"].iter().any(|d| t.strip_prefix(d).map_or(false, |r| r.is_empty() || r.starts_with(|c: char| c.is_whitespace()))) };
    let excluded: Vec<usize> = p.as_ref().map(|p| p.excluded.iter().map(|l| l.saturating_sub(1)).filter(|i| lines.get(*i).map_or(false, |l| !is_pre(l))).collect()).unwrap_or_default();
    let mut outline: Vec<Value> = labels(r.text).into_iter().map(|(n, l)| json!({ "name": format!("@{n}"), "line": l, "kind": "label" })).collect();
    outline.extend(defines.iter().map(|d| json!({ "name": d["name"], "line": d["line"], "kind": "const" })));
    outline.sort_by_key(|o| o["line"].as_u64().unwrap_or(0));
    json!({ "dialect": di.map(|d| json!({ "id": d.id, "name": d.name, "family": d.family })), "system": tid, "defines": defines, "excluded": excluded, "outline": outline })
}

/// Karte aus Kopf und Erklärung
fn card(head: String, body: String) -> String { format!("{head}\n\n{body}") }

/// Karte einer Direktive (Wort klein, ohne `#`)
fn directive_card(w: &str) -> Option<String> {
    Some(match w {
        "define" => card(vc_i18n::t!("basicinfo-dir-define"), vc_i18n::t!("basicinfo-dir-define-text")),
        "if" => card(vc_i18n::t!("basicinfo-dir-if"), vc_i18n::t!("basicinfo-dir-if-text")),
        "else" => card(vc_i18n::t!("basicinfo-dir-else"), vc_i18n::t!("basicinfo-dir-else-text")),
        "endif" => card(vc_i18n::t!("basicinfo-dir-endif"), vc_i18n::t!("basicinfo-dir-endif-text")),
        "asm" => card(vc_i18n::t!("basicinfo-dir-asm"), vc_i18n::t!("basicinfo-dir-asm-text")),
        "blob" => vc_i18n::t!("basicinfo-dir-blob").to_string(),
        "include" => card(vc_i18n::t!("basicinfo-dir-include"), vc_i18n::t!("basicinfo-dir-include-text")),
        "numbering" => card(vc_i18n::t!("basicinfo-dir-numbering"), vc_i18n::t!("basicinfo-dir-numbering-text")),
        "dialect" => card(vc_i18n::t!("basicinfo-dir-dialect"), vc_i18n::t!("basicinfo-dir-dialect-text")),
        "longnames" => card(vc_i18n::t!("basicinfo-dir-longnames"), vc_i18n::t!("basicinfo-dir-longnames-text")),
        _ => return None,
    })
}

/// Erklärtext zu einem BASIC-Diagnosecode (B150 bis B196); `None` für andere Codes. Reiner Text, Befehle in Rückwärtsstrichen.
pub fn code_text(code: &str) -> Option<String> {
    Some(match code {
        "B150" => vc_i18n::t!("basiccode-b150").to_string(),
        "B151" => vc_i18n::t!("basiccode-b151").to_string(),
        "B152" => vc_i18n::t!("basiccode-b152").to_string(),
        "B153" => vc_i18n::t!("basiccode-b153").to_string(),
        "B154" => vc_i18n::t!("basiccode-b154").to_string(),
        "B155" => vc_i18n::t!("basiccode-b155").to_string(),
        "B156" => vc_i18n::t!("basiccode-b156").to_string(),
        "B157" => vc_i18n::t!("basiccode-b157").to_string(),
        "B158" => vc_i18n::t!("basiccode-b158").to_string(),
        "B159" => vc_i18n::t!("basiccode-b159").to_string(),
        "B160" => vc_i18n::t!("basiccode-b160").to_string(),
        "B161" => vc_i18n::t!("basiccode-b161").to_string(),
        "B162" => vc_i18n::t!("basiccode-b162").to_string(),
        "B163" => vc_i18n::t!("basiccode-b163").to_string(),
        "B164" => vc_i18n::t!("basiccode-b164").to_string(),
        "B165" => vc_i18n::t!("basiccode-b165").to_string(),
        "B166" => vc_i18n::t!("basiccode-b166").to_string(),
        "B167" => vc_i18n::t!("basiccode-b167").to_string(),
        "B168" => vc_i18n::t!("basiccode-b168").to_string(),
        "B170" => vc_i18n::t!("basiccode-b170").to_string(),
        "B171" => vc_i18n::t!("basiccode-b171").to_string(),
        "B172" => vc_i18n::t!("basiccode-b172").to_string(),
        "B173" => vc_i18n::t!("basiccode-b173").to_string(),
        "B174" => vc_i18n::t!("basiccode-b174").to_string(),
        "B175" => vc_i18n::t!("basiccode-b175").to_string(),
        "B176" => vc_i18n::t!("basiccode-b176").to_string(),
        "B177" => vc_i18n::t!("basiccode-b177").to_string(),
        "B178" => vc_i18n::t!("basiccode-b178").to_string(),
        "B180" => vc_i18n::t!("basiccode-b180").to_string(),
        "B181" => vc_i18n::t!("basiccode-b181").to_string(),
        "B182" => vc_i18n::t!("basiccode-b182").to_string(),
        "B183" => vc_i18n::t!("basiccode-b183").to_string(),
        "B184" => vc_i18n::t!("basiccode-b184").to_string(),
        "B185" => vc_i18n::t!("basiccode-b185").to_string(),
        "B190" => vc_i18n::t!("basiccode-b190").to_string(),
        "B191" => vc_i18n::t!("basiccode-b191").to_string(),
        "B192" => vc_i18n::t!("basiccode-b192").to_string(),
        "B193" => vc_i18n::t!("basiccode-b193").to_string(),
        "B194" => vc_i18n::t!("basiccode-b194").to_string(),
        "B195" => vc_i18n::t!("basiccode-b195").to_string(),
        "B196" => vc_i18n::t!("basiccode-b196").to_string(),
        _ => return None,
    })
}

/// Alle Erklärtexte auf einmal: `{ "B150": { kind, disable, text }, … }`; `kind` ist `error`, `warning` oder `hint`, `disable`: lässt sich je Projekt abschalten
pub fn code_texts() -> Value {
    use vc_basic::codes::{can_disable, Kind, CODES};
    let mut m = serde_json::Map::new();
    for c in CODES {
        let Some(text) = code_text(c.code) else { continue };
        let kind = match c.kind { Kind::Error => "error", Kind::Warning => "warning", Kind::Hint => "hint" };
        m.insert(c.code.to_string(), json!({ "kind": kind, "disable": can_disable(c.code), "text": text }));
    }
    Value::Object(m)
}

/// Karte für die Stelle (`line` ab 0, `col` in Zeichen): ein `#define`-Name oder ein Direktivenwort. `(Markdown, Start, Ende)` in Zeichen der Zeile.
pub fn hover(r: &Req, line: usize, col: usize) -> Option<(String, usize, usize)> {
    let src = r.text.split('\n').nth(line)?.trim_end_matches('\r');
    let ch: Vec<char> = src.chars().collect();
    if col >= ch.len() { return None; }
    let is_w = |c: char| c.is_ascii_alphanumeric() || c == '_';
    // Zeichenkette: nichts erklären
    if ch[..col].iter().filter(|c| **c == '"').count() % 2 == 1 { return None; }
    if !is_w(ch[col]) && ch[col] != '#' { return None; }
    let (mut a, mut z) = (col, col);
    while a > 0 && is_w(ch[a - 1]) { a -= 1; }
    while z < ch.len() && is_w(ch[z]) { z += 1; }
    if ch[col] == '#' { a = col; z = col + 1; while z < ch.len() && ch[z].is_ascii_alphabetic() { z += 1; } }
    let hash = a > 0 && ch[a - 1] == '#' || ch[a] == '#';
    let word: String = ch[a..z].iter().collect::<String>().trim_start_matches('#').to_string();
    if word.is_empty() { return None; }
    let (start, end) = (if a > 0 && ch[a - 1] == '#' { a - 1 } else { a }, z);
    // Direktive am Zeilenanfang
    if hash && src.trim_start().starts_with('#') {
        let w = word.to_ascii_lowercase();
        if let Some(md) = directive_card(&w) { return Some((md, start, end)); }
    }
    // Name aus `#define`
    let (tid, _) = context(r);
    let p = pre(r, &tid)?;
    let d = p.defines.iter().find(|d| d.name == word)?;
    let md = card(vc_i18n::t!("basicinfo-define", name = d.name.as_str(), value = d.value.as_str(), line = d.line), vc_i18n::t!("basicinfo-define-text"));
    Some((md, start, end))
}
