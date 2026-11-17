// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprachdienst im selben Prozess: `vc_lsp::Server` läuft in einem Arbeits-Thread; die Oberfläche spricht über Kanäle mit ihm.
//! Diagnosen (Fehler und Warnungen beim Tippen), Vervollständigung und Hover.
use serde_json::{json, Value};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

pub enum Cmd { Open { uri: String, text: String }, Change { uri: String, text: String, version: i64 }, Close { uri: String },
    /// Vorschläge an der Stelle (Zeile, Zeichenspalte); `req` ordnet die Antwort zu
    Complete { uri: String, req: u64 }, Hover { uri: String, zeile: usize, spalte: usize, req: u64 },
    /// Definition des Wortes an der Stelle suchen
    Definition { uri: String, zeile: usize, spalte: usize, req: u64 } }
#[derive(Clone, Debug, PartialEq)]
pub struct Diag { pub zeile: usize, pub fehler: bool, pub hinweis: bool, pub text: String }
#[derive(Clone, Debug, PartialEq)]
pub struct Vorschlag { pub text: String, pub detail: String }
#[derive(Clone, Debug)]
pub enum Antwort { Diagnose { uri: String, diag: Vec<Diag> }, Vorschlaege { req: u64, uri: String, items: Vec<Vorschlag> }, Hover { req: u64, uri: String, zeile: usize, spalte: usize, text: Option<String>, bereich: Option<(usize, usize)> }, Definition { req: u64, ziel: Option<(String, usize)> } }

pub struct Lsp { tx: Sender<Cmd> }
pub fn uri(root: &Path, rel: &str) -> String { vc_core::paths::file_uri(&root.join(rel).to_string_lossy()) }

impl Lsp {
    pub fn start(root: &Path, data: &Path) -> (Lsp, Receiver<Antwort>) {
        let (tx, crx) = channel::<Cmd>(); let (rtx, rx) = channel::<Antwort>();
        let (root, data) = (root.to_path_buf(), data.to_path_buf());
        std::thread::spawn(move || {
            let mut srv = vc_lsp::Server::new();
            srv.handle(json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": { "processId": null, "rootUri": vc_core::paths::file_uri(&root.to_string_lossy()), "capabilities": {}, "initializationOptions": { "dataDir": data.to_string_lossy() } } }));
            srv.handle(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
            while let Ok(first) = crx.recv() {
                // Änderungen zusammenfassen: nur die letzte je Datei auswerten
                let mut batch = vec![first]; while let Ok(c) = crx.try_recv() { batch.push(c) }
                let mut letzte: std::collections::HashMap<String, usize> = Default::default();
                for (i, c) in batch.iter().enumerate() { if let Cmd::Change { uri, .. } = c { letzte.insert(uri.clone(), i); } }
                for (i, c) in batch.into_iter().enumerate() {
                    let msgs: Vec<Value> = match c {
                        Cmd::Complete { uri, req } => { let r = srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/completion", "params": { "textDocument": { "uri": uri }, "position": { "line": 0, "character": 0 } } })); let items = r.iter().find(|m| m["id"] == json!(req)).map(|m| { let l = if m["result"].is_array() { &m["result"] } else { &m["result"]["items"] }; l.as_array().into_iter().flatten().map(|i| Vorschlag { text: i["label"].as_str().unwrap_or("").into(), detail: i["detail"].as_str().unwrap_or("").into() }).collect() }).unwrap_or_default(); let _ = rtx.send(Antwort::Vorschlaege { req, uri, items }); continue }
                        Cmd::Definition { uri, zeile, spalte, req } => { let r = srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/definition", "params": { "textDocument": { "uri": uri }, "position": { "line": zeile, "character": spalte } } })); let ziel = r.iter().find(|m| m["id"] == json!(req)).and_then(|m| { let e = if m["result"].is_array() { &m["result"][0] } else { &m["result"] }; Some((e["uri"].as_str()?.to_string(), e["range"]["start"]["line"].as_u64()? as usize)) }); let _ = rtx.send(Antwort::Definition { req, ziel }); continue }
                        Cmd::Hover { uri, zeile, spalte, req } => { let r = srv.handle(json!({ "jsonrpc": "2.0", "id": req, "method": "textDocument/hover", "params": { "textDocument": { "uri": uri }, "position": { "line": zeile, "character": spalte } } })); let erg = r.iter().find(|m| m["id"] == json!(req)); let text = erg.and_then(|m| m["result"]["contents"]["value"].as_str().map(String::from)); let bereich = erg.and_then(|m| Some((m["result"]["range"]["start"]["character"].as_u64()? as usize, m["result"]["range"]["end"]["character"].as_u64()? as usize))); let _ = rtx.send(Antwort::Hover { req, uri, zeile, spalte, text, bereich }); continue }
                        Cmd::Open { uri, text } => srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": uri, "languageId": "vcasm", "version": 1, "text": text } } })),
                        Cmd::Change { uri, text, version } => { if letzte.get(&uri) != Some(&i) { continue } srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "text": text }] } })) }
                        Cmd::Close { uri } => srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didClose", "params": { "textDocument": { "uri": uri } } })),
                    };
                    for m in msgs { if m["method"] == "textDocument/publishDiagnostics" {
                        let diag = m["params"]["diagnostics"].as_array().into_iter().flatten().map(|d| Diag { zeile: d["range"]["start"]["line"].as_u64().unwrap_or(0) as usize, fehler: d["severity"].as_u64().unwrap_or(1) == 1, hinweis: d["severity"].as_u64().unwrap_or(1) >= 3, text: d["message"].as_str().unwrap_or("").into() }).collect();
                        let _ = rtx.send(Antwort::Diagnose { uri: m["params"]["uri"].as_str().unwrap_or("").into(), diag });
                    } }
                }
            }
        });
        (Lsp { tx }, rx)
    }
    pub fn senden(&self, c: Cmd) { let _ = self.tx.send(c); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn unbekanntes_symbol_wird_beim_tippen_gemeldet() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root = std::env::temp_dir().join(format!("vcgui_lsp_{}", std::process::id())); std::fs::create_dir_all(&root).unwrap();
        let (l, rx) = Lsp::start(&root, &data); let u = uri(&root, "a.asm");
        l.senden(Cmd::Open { uri: u.clone(), text: ".target c64\n.org $0801\nstart: lda #1\n jmp nirgendwo\n".into() });
        let t0 = std::time::Instant::now(); let mut gefunden = None;
        while t0.elapsed().as_secs() < 20 { if let Ok(a) = rx.recv_timeout(std::time::Duration::from_millis(200)) { if let Antwort::Diagnose { uri: au, diag } = &a { if *au == u && !diag.is_empty() { gefunden = Some(diag.clone()); break } } } }
        let a = gefunden.expect("Diagnose erwartet"); assert!(a.iter().any(|d| d.fehler && d.zeile == 3), "{:?}", a);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test] fn definition_einer_marke_wird_gefunden() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root = std::env::temp_dir().join(format!("vcgui_lsp3_{}", std::process::id())); std::fs::create_dir_all(&root).unwrap();
        let (l, rx) = Lsp::start(&root, &data); let u = uri(&root, "d.asm");
        l.senden(Cmd::Open { uri: u.clone(), text: ".target c64\n.org $0801\nstart: lda #1\n jmp start\n".into() });
        l.senden(Cmd::Definition { uri: u.clone(), zeile: 3, spalte: 6, req: 5 });
        let t0 = std::time::Instant::now(); let mut z = None;
        while t0.elapsed().as_secs() < 20 && z.is_none() { if let Ok(Antwort::Definition { req: 5, ziel }) = rx.recv_timeout(std::time::Duration::from_millis(200)) { z = Some(ziel) } }
        assert_eq!(z.expect("Antwort erwartet").map(|(_, zeile)| zeile), Some(2), "start steht in Zeile 3 (0-basiert 2)");
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test] fn vorschlaege_und_hover_kommen_aus_dem_sprachdienst() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root = std::env::temp_dir().join(format!("vcgui_lsp2_{}", std::process::id())); std::fs::create_dir_all(&root).unwrap();
        let (l, rx) = Lsp::start(&root, &data); let u = uri(&root, "b.asm");
        l.senden(Cmd::Open { uri: u.clone(), text: ".target c64\n.org $0801\nstart: lda #$20\n jmp start\n".into() });
        l.senden(Cmd::Complete { uri: u.clone(), req: 7 }); l.senden(Cmd::Hover { uri: u.clone(), zeile: 2, spalte: 12, req: 8 });
        let (mut v, mut h) = (None, None); let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 20 && (v.is_none() || h.is_none()) { if let Ok(a) = rx.recv_timeout(std::time::Duration::from_millis(200)) { match a { Antwort::Vorschlaege { req: 7, items, .. } => v = Some(items), Antwort::Hover { req: 8, text, .. } => h = Some(text), _ => {} } } }
        let v = v.expect("Vorschläge erwartet"); assert!(v.iter().any(|i| i.text == "lda") && v.iter().any(|i| i.text == "start"), "{} Einträge", v.len());
        assert!(h.expect("Hover-Antwort erwartet").is_some_and(|t| t.contains("32")), "Hover auf $20 nennt den Wert");
        let _ = std::fs::remove_dir_all(&root);
    }
    #[test] fn waisen_datei_meldet_hinweis_und_include_diagnosen_kommen_fuer_beide_dateien() {
        let data = vc_ide::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let root = std::env::temp_dir().join(format!("vcgui_lsp4_{}", std::process::id())); let _ = std::fs::remove_dir_all(&root); std::fs::create_dir_all(&root).unwrap();
        let lib = "wert: .byte 1\n lda fehlt\n";
        std::fs::write(root.join("lib.asm"), lib).unwrap();
        let (l, rx) = Lsp::start(&root, &data); let u = uri(&root, "lib.asm");
        // Waise (keine Wurzel, kein .org/.target): fehlendes Symbol ist nur ein Hinweis (Schweregrad 3), kein Fehler
        l.senden(Cmd::Open { uri: u.clone(), text: lib.into() });
        let t0 = std::time::Instant::now(); let mut gefunden = None;
        while t0.elapsed().as_secs() < 20 { if let Ok(Antwort::Diagnose { uri: au, diag }) = rx.recv_timeout(std::time::Duration::from_millis(200)) { if au == u && !diag.is_empty() { gefunden = Some(diag); break } } }
        let a = gefunden.expect("Meldung zur Waisen-Datei erwartet"); assert!(a.iter().any(|d| d.hinweis && !d.fehler && d.zeile == 1), "{a:?}");
        // Ungespeicherter Text geht in den Lauf: Meldungen beziehen sich auf den neuen Pufferstand (eine Zeile mehr)
        l.senden(Cmd::Change { uri: u.clone(), text: "wert: .byte 1\nfehlt: .byte 2\n lda fehlt\n".into(), version: 2 });
        let mut leer = false; let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 20 && !leer { if let Ok(Antwort::Diagnose { uri: au, diag }) = rx.recv_timeout(std::time::Duration::from_millis(200)) { if au == u && diag.iter().any(|d| d.zeile == 2 && d.hinweis) && diag.iter().all(|d| d.zeile != 1) { leer = true } } }
        assert!(leer, "der Kontextlauf nutzt den ungespeicherten Text (Hinweis wandert von Zeile 1 nach 2)");
        let _ = std::fs::remove_dir_all(&root);
    }
}
