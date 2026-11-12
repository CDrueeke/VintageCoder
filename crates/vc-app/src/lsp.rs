// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprachdienst im selben Prozess: `vc_lsp::Server` läuft in einem Arbeits-Thread, die Oberfläche spricht über Kanäle mit ihm.
use eframe::egui;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Cmd { Open { uri: String, text: String }, Change { uri: String, text: String, version: i64 }, Close { uri: String }, Complete { uri: String, line: usize, col: usize, req: u64 }, Definition { uri: String, line: usize, col: usize, req: u64 }, Hover { uri: String, line: usize, col: usize, req: u64 } }

#[derive(Clone, Debug)]
pub struct Item { pub label: String, #[allow(dead_code)] pub kind: u64, pub detail: String }
#[derive(Clone, Debug)]
pub struct Diag { pub line: usize, pub error: bool, pub code: String, pub msg: String }

pub enum Reply { Diagnostics { uri: String, items: Vec<Diag> }, Completion { req: u64, items: Vec<Item> }, Addresses { uri: String, lines: Vec<(usize, u16, usize)>, source: Vec<String> }, Definition { uri: String, line: usize }, NoDefinition, Hover { req: u64, text: String } }

pub struct Lsp { tx: Sender<Cmd>, pub rx: Receiver<Reply> }

pub fn uri(root: &Path, rel: &str) -> String { vc_core::paths::file_uri(&root.join(rel).to_string_lossy()) }

impl Lsp {
    pub fn start(root: &Path, data: &Path, ctx: egui::Context) -> Lsp {
        let (tx, crx) = channel::<Cmd>(); let (rtx, rx) = channel::<Reply>();
        let (root, data) = (root.to_path_buf(), data.to_path_buf());
        std::thread::spawn(move || {
            let mut srv = vc_lsp::Server::new();
            srv.handle(json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "processId": null, "rootUri": vc_core::paths::file_uri(&root.to_string_lossy()), "capabilities": {}, "initializationOptions": { "dataDir": data.to_string_lossy() } } }));
            srv.handle(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
            while let Ok(first) = crx.recv() {
                // Änderungen zusammenfassen: nur die letzte je Datei auswerten
                let mut batch = vec![first]; while let Ok(c) = crx.try_recv() { batch.push(c); }
                let mut last_change: std::collections::HashMap<String, usize> = Default::default();
                for (i, c) in batch.iter().enumerate() { if let Cmd::Change { uri, .. } = c { last_change.insert(uri.clone(), i); } }
                for (i, c) in batch.into_iter().enumerate() {
                    let is_def = matches!(&c, Cmd::Definition { .. });
                    let (msgs, req) = match c {
                        Cmd::Open { uri, text } => (srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": uri, "languageId": "vcasm", "version": 1, "text": text } } })), None),
                        Cmd::Change { uri, text, version } => { if last_change.get(&uri) != Some(&i) { continue; } (srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "text": text }] } })), None) }
                        Cmd::Close { uri } => (srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didClose", "params": { "textDocument": { "uri": uri } } })), None),
                        Cmd::Definition { uri, line, col, req } => (srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/definition", "params": { "textDocument": { "uri": uri }, "position": { "line": line, "character": col } } })), Some(req)),
                        Cmd::Hover { uri, line, col, req } => (srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/hover", "params": { "textDocument": { "uri": uri }, "position": { "line": line, "character": col } } })), Some(req)),
                        Cmd::Complete { uri, line, col, req } => (srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/completion", "params": { "textDocument": { "uri": uri }, "position": { "line": line, "character": col } } })), Some(req)),
                    };
                    for m in msgs {
                        if let (Some(_), Some(u)) = (req, m["result"]["uri"].as_str()) {
                            let _ = rtx.send(Reply::Definition { uri: u.into(), line: m["result"]["range"]["start"]["line"].as_u64().unwrap_or(0) as usize });
                        } else if is_def && m["result"].is_null() && m.get("result").is_some() {
                            let _ = rtx.send(Reply::NoDefinition);   // das Ziel ist unbekannt: die Oberfläche meldet es
                        } else if m["method"] == "vintagecoder/addresses" {
                            let lines = m["params"]["lines"].as_array().into_iter().flatten().filter_map(|l| Some((l[0].as_u64()? as usize, l[1].as_u64()? as u16, l[2].as_u64()? as usize))).collect();
                            let source = m["params"]["source"].as_array().into_iter().flatten().map(|s| s.as_str().unwrap_or("").to_string()).collect();
                            let _ = rtx.send(Reply::Addresses { uri: m["params"]["uri"].as_str().unwrap_or("").into(), lines, source });
                        } else if m["method"] == "textDocument/publishDiagnostics" {
                            let items = m["params"]["diagnostics"].as_array().into_iter().flatten().map(|d| Diag { line: d["range"]["start"]["line"].as_u64().unwrap_or(0) as usize, error: d["severity"].as_u64().unwrap_or(1) == 1, code: d["code"].as_str().unwrap_or("").into(), msg: d["message"].as_str().unwrap_or("").into() }).collect();
                            let _ = rtx.send(Reply::Diagnostics { uri: m["params"]["uri"].as_str().unwrap_or("").into(), items });
                        } else if let (Some(req), Some(t)) = (req, m["result"]["contents"]["value"].as_str()) {
                            let _ = rtx.send(Reply::Hover { req, text: t.to_string() });
                        } else if let (Some(req), Some(arr)) = (req, m["result"].as_array().or_else(|| m["result"]["items"].as_array())) {
                            let items = arr.iter().map(|i: &Value| Item { label: i["label"].as_str().unwrap_or("").into(), kind: i["kind"].as_u64().unwrap_or(0), detail: i["detail"].as_str().unwrap_or("").into() }).collect();
                            let _ = rtx.send(Reply::Completion { req, items });
                        }
                    }
                    ctx.request_repaint();
                }
            }
        });
        Lsp { tx, rx }
    }
    pub fn send(&self, c: Cmd) { let _ = self.tx.send(c); }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait(l: &Lsp, pred: impl Fn(&Reply) -> bool) -> Option<Reply> {
        let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 20 { if let Ok(r) = l.rx.recv_timeout(std::time::Duration::from_millis(200)) { if pred(&r) { return Some(r); } } }
        None
    }

    #[test]
    fn definition_of_unknown_symbol_is_reported_and_known_symbol_is_found() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root = std::env::temp_dir().join(format!("vc_lsp_def_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let l = Lsp::start(&root, &data, egui::Context::default());
        let u = uri(&root, "a.asm");
        l.send(Cmd::Open { uri: u.clone(), text: ".target c64\n.org $0801\nstart: lda #1\n jmp start\n jmp nirgendwo\n".into() });
        // „start“ in Zeile 4 (0-basiert 3), Spalte 6
        l.send(Cmd::Definition { uri: u.clone(), line: 3, col: 6, req: 1 });
        assert!(matches!(wait(&l, |r| matches!(r, Reply::Definition { .. } | Reply::NoDefinition)), Some(Reply::Definition { line: 2, .. })), "bekanntes Label wird gefunden");
        // Ein unbekannter Name: Meldung statt Schweigen
        l.send(Cmd::Definition { uri: u, line: 4, col: 8, req: 2 });
        assert!(matches!(wait(&l, |r| matches!(r, Reply::Definition { .. } | Reply::NoDefinition)), Some(Reply::NoDefinition)));
        let _ = std::fs::remove_dir_all(&root);
    }
}
