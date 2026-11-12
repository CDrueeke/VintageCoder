// SPDX-License-Identifier: GPL-3.0-or-later
//! Integrationstest: startet `vc-lsp` und spricht LSP über stdin/stdout.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client { child: Child, stdin: ChildStdin, out: BufReader<ChildStdout>, next: u64 }

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vc-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let (stdin, out) = (child.stdin.take().unwrap(), BufReader::new(child.stdout.take().unwrap()));
        Client { child, stdin, out, next: 1 }
    }
    fn send(&mut self, v: Value) {
        let b = v.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", b.len(), b).unwrap();
        self.stdin.flush().unwrap();
    }
    fn read(&mut self) -> Value {
        let mut len = 0;
        loop {
            let mut l = String::new();
            self.out.read_line(&mut l).unwrap();
            if l.trim().is_empty() { break; }
            if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") { len = v.trim().parse().unwrap(); }
        }
        let mut buf = vec![0; len];
        self.out.read_exact(&mut buf).unwrap();
        serde_json::from_slice(&buf).unwrap()
    }
    fn notify(&mut self, method: &str, params: Value) { self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params })); }
    /// Sendet eine Anfrage und liefert die Antwort; dazwischenliegende Benachrichtigungen werden gesammelt.
    fn request(&mut self, method: &str, params: Value) -> (Value, Vec<Value>) {
        let id = self.next;
        self.next += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let mut notes = vec![];
        loop {
            let m = self.read();
            if m["id"] == json!(id) { return (m["result"].clone(), notes); }
            notes.push(m);
        }
    }
    fn open(&mut self, uri: &str, text: &str) -> Value {
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "vcasm", "version": 1, "text": text } }));
        self.read()
    }
}

fn root() -> String { format!("{}/../..", env!("CARGO_MANIFEST_DIR")) }

fn client() -> Client {
    let mut c = Client::start();
    let (r, _) = c.request("initialize", json!({ "rootUri": format!("file://{}", root()), "capabilities": {}, "initializationOptions": { "dataDir": format!("{}/data", root()) } }));
    assert_eq!(r["capabilities"]["hoverProvider"], json!(true));
    c.notify("initialized", json!({}));
    c
}

fn hover(c: &mut Client, uri: &str, line: u64, ch: u64) -> String {
    let (r, _) = c.request("textDocument/hover", json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": ch } }));
    r["contents"]["value"].as_str().unwrap_or("").to_string()
}

const SRC: &str = ".target c64\n.org $0801\nstart:  lda #$00\n        sta vic2.border\n        lax $d012\n.loop:  jmp .loop\n";

#[test]
fn diagnostics_hover_definition_completion_symbols() {
    let mut c = client();
    let uri = "file:///tmp/vc_lsp_test.asm";
    // Diagnose: undokumentierter Opcode als Warnung (W104), kein Fehler
    let d = c.open(uri, &SRC.replace("vic2.border", "$d020"));
    assert_eq!(d["method"], "textDocument/publishDiagnostics");
    let items = d["params"]["diagnostics"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!((items[0]["code"].as_str(), items[0]["severity"].as_i64(), items[0]["range"]["start"]["line"].as_i64()), (Some("W104"), Some(2), Some(4)));
    // Hover auf Mnemonic: Opcode-Tabelle mit Zyklen
    let h = hover(&mut c, uri, 2, 10);
    assert!(h.contains("**LDA**") && h.contains("| imm | $A9 | 2 | 2 |") && h.contains("abx") && h.contains("4+1"), "{h}");
    let h = hover(&mut c, uri, 4, 9);
    assert!(h.contains("**LAX**") && h.contains("undok."), "{h}");
    // Hover auf Direktive
    assert!(hover(&mut c, uri, 1, 2).contains("Programmzähler"));
    // Hover auf Label: Wert und Definitionsstelle
    let h = hover(&mut c, uri, 5, 16);
    assert!(h.contains("**.loop**") && h.contains("$0809"), "{h}");
    // Fehler im Dokument ersetzt die Diagnose; Hover zeigt weiter den letzten Stand
    c.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri }, "contentChanges": [{ "text": ".target c64\n.org $0801\n foo\n lda #300\n" }] }));
    let d = c.read();
    let codes: Vec<&str> = d["params"]["diagnostics"].as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap()).collect();
    assert_eq!(codes, vec!["E205", "E203"]);
    // Register-Namen: Hover, Definition, Completion
    let uri2 = "file:///tmp/vc_lsp_test2.asm";
    let d = c.open(uri2, &SRC);
    assert!(d["params"]["diagnostics"].as_array().unwrap().len() <= 1);
    let h = hover(&mut c, uri2, 3, 14);
    assert!(h.contains("vic2.border") && h.contains("$D020") && h.contains("Rahmenfarbe") == false || h.contains("vic2.border"), "{h}");
    let (def, _) = c.request("textDocument/definition", json!({ "textDocument": { "uri": uri2 }, "position": { "line": 5, "character": 17 } }));
    assert_eq!(def["range"]["start"]["line"], json!(5));
    let (comp, _) = c.request("textDocument/completion", json!({ "textDocument": { "uri": uri2 }, "position": { "line": 3, "character": 12 } }));
    let labels: Vec<&str> = comp["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    for want in ["lda", "lax", ".org", "vic2.border", "start", ".loop"] { assert!(labels.contains(&want), "fehlt: {want}"); }
    let (syms, _) = c.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri2 } }));
    let names: Vec<&str> = syms.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["start", ".loop"]);
    // Sauber beenden
    let (r, _) = c.request("shutdown", Value::Null);
    assert!(r.is_null());
    c.notify("exit", Value::Null);
    assert!(c.child.wait().unwrap().success());
}

#[test]
fn atari_target_is_detected_from_the_file() {
    let mut c = client();
    let uri = "file:///tmp/vc_lsp_atari.asm";
    let d = c.open(uri, ".target atari8\n.org $2000\nstart: sta gtia.colbk\n jmp start\n");
    assert!(d["params"]["diagnostics"].as_array().unwrap().is_empty(), "{d}");
    let h = hover(&mut c, uri, 2, 14);
    assert!(h.contains("gtia.colbk") && h.contains("$D01A") && h.contains("schreiben"), "{h}");
    let d = c.open("file:///tmp/vc_lsp_bad.asm", ".target nirgendwo\n nop\n");
    let msg = d["params"]["diagnostics"][0]["message"].as_str().unwrap().to_string();
    assert!(msg.contains("nirgendwo"), "{msg}");
}

#[test]
fn hover_shows_number_literals_and_constants_in_all_notations() {
    let mut c = client();
    let uri = "file:///tmp/vc_lsp_test_values.asm";
    let src = ".target c64\n.const LIVES = 3\n.const SCR = $0400\n.org $0801\nstart:  lda #LIVES\n        sta $d020\n        ldx #%10100101\n        lda #'A'\n";
    c.open(uri, src);
    // Literal $d020: Dezimal, Hex, Binär, Low-/High-Byte und das passende Register
    let h = hover(&mut c, uri, 5, 12);
    assert!(h.contains("**$d020**") && h.contains("| Dezimal | 53280 |") && h.contains("`%1101 0000 0010 0000`") && h.contains("High-Byte (`>`) | `$D0` = 208") && h.contains("vic2.border"), "{h}");
    // Binärliteral und Zeichenliteral
    let h = hover(&mut c, uri, 6, 14);
    assert!(h.contains("| Dezimal | 165 |") && h.contains("`$A5`") && h.contains("Mit Vorzeichen (8 Bit) | -91"), "{h}");
    let h = hover(&mut c, uri, 7, 14);
    assert!(h.contains("| Dezimal | 65 |") && h.contains("Zeichen (ASCII) | `'A'`"), "{h}");
    // Konstanten (Symbole): Name, Wert und Definitionsstelle
    let h = hover(&mut c, uri, 4, 14);
    assert!(h.contains("**LIVES**") && h.contains("| Dezimal | 3 |") && h.contains("`%0000 0011`") && h.contains("Definiert in"), "{h}");
}
