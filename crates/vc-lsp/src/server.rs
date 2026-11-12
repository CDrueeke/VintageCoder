// SPDX-License-Identifier: GPL-3.0-or-later
//! LSP-Server: Dokumentverwaltung, Analyse und Antworten auf Editor-Anfragen.
use crate::docs::{directive_doc, mnemonic_doc, DIRECTIVES};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use vc_asm::{assemble, line_parts, outline, OutlineKind, Options, Output, PartKind, Policy};
use vc_core::Diagnostic;
use vc_target::{load_chip, load_cpu, load_target, Chip, Cpu, Target};

struct Ctx { cpu: Cpu, #[allow(dead_code)] target: Target, chips: Vec<Chip>, predefined: Vec<(String, i64)> }

struct Doc { text: String, out: Option<Output>, published: HashSet<String>, target: String }

pub struct Server {
    data_dir: Option<PathBuf>,
    root: Option<PathBuf>,
    default_target: String,
    docs: HashMap<String, Doc>,
    ctxs: HashMap<String, Ctx>,
    pub shutdown: bool,
    pub exit: bool,
}

fn uri_to_path(uri: &str) -> String { vc_core::paths::uri_to_path(uri) }
fn path_to_uri(p: &str) -> String { vc_core::paths::file_uri(p) }

fn find_data_dir(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("data/cpus").is_dir() { return Some(dir.join("data")); }
        // macOS-App-Bundle: Contents/MacOS/<programm>, Daten in Contents/Resources/data
        if dir.join("Resources/data/cpus").is_dir() { return Some(dir.join("Resources/data")); }
        // Linux-Pakete (FHS): <prefix>/bin/vc-lsp, Daten in <prefix>/share/vintagecoder/data
        if dir.join("share/vintagecoder/data/cpus").is_dir() { return Some(dir.join("share/vintagecoder/data")); }
        if !dir.pop() { return None; }
    }
}

fn is_ident(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '.' }

fn word_at(chars: &[char], ch: usize) -> Option<(usize, usize)> {
    let mut c = ch.min(chars.len());
    if (c >= chars.len() || !is_ident(chars[c])) && c > 0 && is_ident(chars[c - 1]) { c -= 1; }
    if c >= chars.len() || !is_ident(chars[c]) { return None; }
    let (mut s, mut e) = (c, c + 1);
    while s > 0 && is_ident(chars[s - 1]) { s -= 1; }
    while e < chars.len() && is_ident(chars[e]) { e += 1; }
    Some((s, e))
}

fn detect_target(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (_, p) = line_parts(l);
        p.filter(|p| p.kind == PartKind::Directive && p.name == ".target").and_then(|p| p.args.split_whitespace().next().map(str::to_string))
    })
}

fn resp(id: Option<Value>, result: Value) -> Value { json!({ "jsonrpc": "2.0", "id": id, "result": result }) }
fn range(line: usize, s: usize, e: usize) -> Value { json!({ "start": { "line": line, "character": s }, "end": { "line": line, "character": e } }) }

impl Server {
    pub fn new() -> Self {
        Server { data_dir: None, root: None, default_target: "c64".into(), docs: HashMap::new(), ctxs: HashMap::new(), shutdown: false, exit: false }
    }

    fn context(&mut self, target: &str) -> Result<&Ctx, String> {
        if !self.ctxs.contains_key(target) {
            let data = self.data_dir.clone().ok_or("Hardware-Datenbank (data/) nicht gefunden – initializationOptions.dataDir oder VC_DATA setzen")?;
            let t = load_target(&data, target)?;
            let cpu = load_cpu(&data, &t.cpu)?;
            let chips = t.chips.iter().filter_map(|c| load_chip(&data, c).ok()).collect();
            let predefined = vc_target::chip_symbols(&data, &t)?;
            self.ctxs.insert(target.to_string(), Ctx { cpu, target: t, chips, predefined });
        }
        Ok(&self.ctxs[target])
    }

    pub fn handle(&mut self, msg: Value) -> Vec<Value> {
        let method = msg["method"].as_str().unwrap_or("").to_string();
        let id = msg.get("id").cloned().filter(|v| !v.is_null());
        let p = &msg["params"];
        let mut out = vec![];
        let uri = p["textDocument"]["uri"].as_str().unwrap_or("").to_string();
        let (line, ch) = (p["position"]["line"].as_u64().unwrap_or(0) as usize, p["position"]["character"].as_u64().unwrap_or(0) as usize);
        match method.as_str() {
            "initialize" => {
                self.root = p["rootUri"].as_str().map(|u| PathBuf::from(uri_to_path(u))).or_else(|| p["rootPath"].as_str().map(PathBuf::from));
                let opts = &p["initializationOptions"];
                if let Some(t) = opts["target"].as_str() { self.default_target = t.to_string(); }
                self.data_dir = opts["dataDir"].as_str().map(PathBuf::from)
                    .or_else(|| std::env::var("VC_DATA").ok().map(PathBuf::from))
                    .or_else(|| self.root.as_deref().and_then(find_data_dir))
                    .or_else(|| std::env::current_dir().ok().and_then(|d| find_data_dir(&d)))
                    .or_else(|| std::env::current_exe().ok().and_then(|e| find_data_dir(&e)));      // installiertes Paket: neben dem Programm bzw. <prefix>/share/vintagecoder/data
                out.push(resp(id, json!({
                    "capabilities": {
                        "textDocumentSync": 1, "hoverProvider": true, "definitionProvider": true, "documentSymbolProvider": true,
                        "completionProvider": { "triggerCharacters": [".", "$"] }
                    },
                    "serverInfo": { "name": "vc-lsp", "version": "0.1.0" }
                })));
            }
            "initialized" | "$/cancelRequest" | "workspace/didChangeConfiguration" => {}
            "shutdown" => { self.shutdown = true; out.push(resp(id, Value::Null)); }
            "exit" => self.exit = true,
            "textDocument/didOpen" => {
                let text = p["textDocument"]["text"].as_str().unwrap_or("").to_string();
                self.docs.insert(uri.clone(), Doc { text, out: None, published: HashSet::new(), target: self.default_target.clone() });
                out.extend(self.analyze(&uri));
            }
            "textDocument/didChange" => {
                if let (Some(c), Some(d)) = (p["contentChanges"].as_array().and_then(|a| a.last()), self.docs.get_mut(&uri)) {
                    d.text = c["text"].as_str().unwrap_or("").to_string();
                }
                out.extend(self.analyze(&uri));
            }
            "textDocument/didClose" => {
                if let Some(d) = self.docs.remove(&uri) {
                    for u in d.published { out.push(json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": u, "diagnostics": [] } })); }
                }
            }
            "textDocument/hover" => out.push(resp(id, self.hover(&uri, line, ch))),
            "textDocument/completion" => out.push(resp(id, self.completion(&uri))),
            "textDocument/definition" => out.push(resp(id, self.definition(&uri, line, ch))),
            "textDocument/documentSymbol" => out.push(resp(id, self.document_symbols(&uri))),
            _ => if id.is_some() { out.push(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("nicht unterstützt: {method}") } })); }
        }
        out
    }

    fn analyze(&mut self, uri: &str) -> Vec<Value> {
        let Some(doc) = self.docs.get(uri) else { return vec![] };
        let text = doc.text.clone();
        let tid = detect_target(&text).unwrap_or_else(|| self.default_target.clone());
        let path = uri_to_path(uri);
        let root = self.root.clone();
        let sys_dir = self.data_dir.as_ref().map(|d| d.join("sys"));
        let (diags, output): (Vec<Diagnostic>, Option<Output>) = match self.context(&tid) {
            Err(e) => (vec![Diagnostic::error("E000", &path, 1, e)], None),
            Ok(ctx) => {
                let mut o = Options::new(path.clone(), Policy::Warn);
                if let Some(r) = &root { o.defines = vc_target::project_defines(r, &tid); }
                o.include_dirs.extend(root);
                if let Some(d) = &sys_dir { o.include_dirs.push(d.clone()); }
                o.predefined = ctx.predefined.clone();
                o.atari_text = ctx.target.text_encoding == "atari";
                o.basic_start = ctx.target.basic_load_addr;
                match assemble(&text, &ctx.cpu, &o) { Ok(out) => (out.warnings.clone(), Some(out)), Err(d) => (d, None) }
            }
        };
        let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        by_file.entry(path.clone()).or_default();
        for d in &diags {
            let file = if d.file.is_empty() { path.clone() } else { d.file.clone() };
            let l = d.line.saturating_sub(1);
            let src = if file == path { Some(text.clone()) } else { std::fs::read_to_string(&file).ok() };
            let line_text: Vec<char> = src.as_deref().and_then(|s| s.lines().nth(l)).unwrap_or("").chars().collect();
            let start = line_text.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
            by_file.entry(file).or_default().push(json!({
                "range": range(l, start, line_text.len()), "severity": if d.is_error() { 1 } else { 2 },
                "code": d.code, "source": "vcc", "message": d.message
            }));
        }
        let mut msgs = vec![];
        let mut now = HashSet::new();
        for (file, items) in by_file {
            let u = path_to_uri(&file);
            now.insert(u.clone());
            msgs.push(json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": u, "diagnostics": items } }));
        }
        let doc = self.docs.get_mut(uri).unwrap();
        for old in doc.published.difference(&now) {
            msgs.push(json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": old, "diagnostics": [] } }));
        }
        doc.published = now;
        doc.target = tid;
        // Zeilenadressen für die Anzeige im Editor (nur Zeilen dieser Datei, die Bytes erzeugen); bei Fehlern bleibt der letzte Stand stehen
        if let Some(out) = &output {
            let lines: Vec<Value> = out.lines.iter().filter(|l| l.file == path && !l.bytes.is_empty()).map(|l| json!([l.line.saturating_sub(1), l.addr, l.bytes.len()])).collect();
            let source: Vec<&str> = text.split('\n').collect();
            msgs.push(json!({ "jsonrpc": "2.0", "method": "vintagecoder/addresses", "params": { "uri": uri, "lines": lines, "source": source } }));
        }
        if output.is_some() { doc.out = output; }
        msgs
    }

    /// Letztes globales Label vor/auf der Zeile (Gültigkeitsbereich lokaler `.labels`).
    fn scope_at(text: &str, line: usize) -> String {
        let mut scope = String::new();
        for l in text.lines().take(line + 1) {
            if let Some(n) = line_parts(l).0 { if !n.starts_with('.') { scope = n; } }
        }
        scope
    }

    fn hover(&self, uri: &str, line: usize, ch: usize) -> Value {
        let Some(doc) = self.docs.get(uri) else { return Value::Null };
        let Some(l) = doc.text.lines().nth(line) else { return Value::Null };
        let chars: Vec<char> = l.chars().collect();
        let ctx = self.ctxs.get(&doc.target);
        // Zahlenliteral ($d020, %1010, 255, 'A'): Wert in mehreren Schreibweisen, bei Adressen auch das passende Register
        if let Some((ls, le, v)) = crate::literal_at(l, ch) {
            let text: String = chars[ls..le].iter().collect();
            let mut md = crate::value_hover(&text, v);
            if let Some(ctx) = ctx { if let Some((chip, r)) = ctx.chips.iter().find_map(|chip| chip.regs.iter().find(|r| r.addr as i64 == v).map(|r| (chip, r))) {
                md.push_str(&format!("\nRegister **{}.{}** – {}", chip.id, r.name, r.doc));
            } }
            return json!({ "contents": { "kind": "markdown", "value": md }, "range": range(line, ls, le) });
        }
        let Some((s, e)) = word_at(&chars, ch) else { return Value::Null };
        let word: String = chars[s..e].iter().collect();
        let (_, part) = line_parts(l);
        let mut md = String::new();
        if let Some(p) = &part {
            if p.kind == PartKind::Instruction && p.name.eq_ignore_ascii_case(&word) {
                if let Some(ctx) = ctx {
                    let mut ops: Vec<_> = ctx.cpu.ops.iter().filter(|o| o.mnemonic == p.name).collect();
                    ops.sort_by_key(|o| o.opcode);
                    if !ops.is_empty() {
                        md.push_str(&format!("**{}** – {}\n\n", p.name, mnemonic_doc(&p.name).unwrap_or("")));
                        if !ops[0].affects.is_empty() { md.push_str(&format!("Flags: `{}`\n\n", ops[0].affects.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" "))); }
                        md.push_str("| Modus | Opcode | Bytes | Zyklen |\n|---|---|---|---|\n");
                        for o in ops {
                            let cy = format!("{}{}{}", o.cycles, if o.page_cross { "+1" } else { "" }, if o.branch { "+1/+2" } else { "" });
                            let tag = if o.group == "official" { "" } else if o.group == "illegal.unstable" { " ⚠ instabil" } else { " ⚠ undok." };
                            md.push_str(&format!("| {} | ${:02X} | {} | {}{} |\n", format!("{:?}", o.mode).to_lowercase(), o.opcode, o.bytes, cy, tag));
                        }
                    }
                }
            } else if p.kind == PartKind::Directive && p.name == word.to_ascii_lowercase() {
                if let Some(d) = directive_doc(&p.name) { md = format!("**{}**\n\n{}", p.name, d); }
            }
        }
        if md.is_empty() {
            if let Some(ctx) = ctx {
                for chip in &ctx.chips {
                    if let Some(r) = chip.regs.iter().find(|r| word == format!("{}.{}", chip.id, r.name)) {
                        let acc = match r.access.as_str() { "r" => "lesen", "w" => "schreiben", _ => "lesen/schreiben" };
                        md = format!("**{}.{}** — `${:04X}` · {}\n\n{}\n\n*{}*", chip.id, r.name, r.addr, acc, r.doc, chip.name);
                    }
                }
            }
        }
        if md.is_empty() {
            if let Some(out) = &doc.out {
                let key = if word.starts_with('.') { format!("{}{}", Self::scope_at(&doc.text, line), word) } else { word.clone() };
                if let Some(v) = out.symbols.get(&key) {
                    md = crate::value_hover(&word, *v);
                    if let Some((f, ln)) = out.defs.get(&key) {
                        md.push_str(&format!("\n\nDefiniert in `{}`:{}", Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), ln));
                    }
                }
            }
        }
        if md.is_empty() { return Value::Null; }
        json!({ "contents": { "kind": "markdown", "value": md }, "range": range(line, s, e) })
    }

    fn completion(&self, uri: &str) -> Value {
        let Some(doc) = self.docs.get(uri) else { return json!({ "isIncomplete": false, "items": [] }) };
        let mut items = vec![];
        if let Some(ctx) = self.ctxs.get(&doc.target) {
            let mut seen = HashSet::new();
            for o in &ctx.cpu.ops {
                if seen.insert(o.mnemonic.clone()) {
                    items.push(json!({ "label": o.mnemonic.to_lowercase(), "kind": 14, "detail": mnemonic_doc(&o.mnemonic).unwrap_or(""), "sortText": format!("2{}", o.mnemonic) }));
                }
            }
            for chip in &ctx.chips {
                for r in &chip.regs {
                    items.push(json!({ "label": format!("{}.{}", chip.id, r.name), "kind": 5, "detail": format!("${:04X} · {}", r.addr, r.access), "documentation": r.doc, "sortText": format!("3{}", r.name) }));
                }
            }
        }
        for (d, doc_) in DIRECTIVES { items.push(json!({ "label": d, "kind": 14, "detail": "Direktive", "documentation": doc_, "sortText": format!("1{d}") })); }
        let mut names = HashSet::new();
        for it in outline(&doc.text) {
            if names.insert(it.name.clone()) {
                let kind = match it.kind { OutlineKind::Label => 6, OutlineKind::Const => 21, OutlineKind::Macro => 3 };
                items.push(json!({ "label": it.name, "kind": kind, "detail": format!("Zeile {}", it.line), "sortText": format!("0{}", it.name) }));
            }
        }
        if let Some(out) = &doc.out {
            for (n, params) in &out.macros {
                let n = &n.to_lowercase();
                if names.insert(n.clone()) { items.push(json!({ "label": n, "kind": 3, "detail": format!("Makro({})", params.join(", ")), "sortText": format!("0{n}") })); }
            }
            for (n, v) in &out.symbols {
                if !n.contains(".__m") && names.insert(n.clone()) { items.push(json!({ "label": n, "kind": 21, "detail": format!("${:04X}", *v as u32 & 0xFFFF), "sortText": format!("0{n}") })); }
            }
        }
        json!({ "isIncomplete": false, "items": items })
    }

    fn definition(&self, uri: &str, line: usize, ch: usize) -> Value {
        let Some(doc) = self.docs.get(uri) else { return Value::Null };
        let Some(l) = doc.text.lines().nth(line) else { return Value::Null };
        let chars: Vec<char> = l.chars().collect();
        let Some((s, e)) = word_at(&chars, ch) else { return Value::Null };
        let word: String = chars[s..e].iter().collect();
        let key = if word.starts_with('.') { format!("{}{}", Self::scope_at(&doc.text, line), word) } else { word.clone() };
        if let Some((f, ln)) = doc.out.as_ref().and_then(|o| o.defs.get(&key)) {
            let uri_f = if Path::new(f).is_absolute() || f.contains('/') { path_to_uri(f) } else { uri.to_string() };
            return json!({ "uri": uri_f, "range": range(ln.saturating_sub(1), 0, 0) });
        }
        outline(&doc.text).into_iter().find(|i| i.name == word)
            .map_or(Value::Null, |i| json!({ "uri": uri, "range": range(i.line - 1, 0, 0) }))
    }

    fn document_symbols(&self, uri: &str) -> Value {
        let Some(doc) = self.docs.get(uri) else { return json!([]) };
        let list: Vec<Value> = outline(&doc.text).into_iter().map(|i| {
            let kind = match i.kind { OutlineKind::Label => 12, OutlineKind::Const => 14, OutlineKind::Macro => 6 };
            json!({ "name": i.name, "kind": kind, "location": { "uri": uri, "range": range(i.line - 1, 0, 0) } })
        }).collect();
        json!(list)
    }
}

#[cfg(test)]
mod pfad_tests {
    use super::*;

    #[test]
    fn adressen_beider_richtungen_stimmen_ueberein() {
        for p in [r"C:\proj\sub\x.asm", "/home/u/my proj/Größe.asm", r"C:\Spiele\Späße 100%\x.asm"] {
            let u = path_to_uri(p);
            assert!(u.starts_with("file:///"), "{u}");
            assert_eq!(uri_to_path(&u), p.replace('\\', "/"));
        }
        assert_eq!(uri_to_path("file:///C:/proj/x.asm"), "C:/proj/x.asm");
    }
}
