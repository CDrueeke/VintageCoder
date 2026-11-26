// SPDX-License-Identifier: GPL-3.0-or-later
//! Integrationstest: startet `vc-lsp` und spricht LSP über stdin/stdout.
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client { child: Child, stdin: ChildStdin, out: BufReader<ChildStdout>, next: u64 }

impl Client {
    /// `vc_lang`: Wert für `VC_LANG` des Serverprozesses (None: Variable entfernen, es gilt die Systemsprache)
    fn start_with_lang(vc_lang: Option<&str>) -> Client {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_vc-lsp"));
        match vc_lang { Some(l) => { cmd.env("VC_LANG", l); } None => { cmd.env_remove("VC_LANG"); } }
        let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
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

fn client() -> Client { client_in(Some("de")) }

/// Startet den Server; `locale` kommt wie von einem Editor in der Anfrage `initialize` (None: keine Angabe)
fn client_in(locale: Option<&str>) -> Client {
    let mut c = Client::start_with_lang(None);
    let mut params = json!({ "rootUri": format!("file://{}", root()), "capabilities": {}, "initializationOptions": { "dataDir": format!("{}/data", root()) } });
    if let Some(l) = locale { params["locale"] = json!(l); }
    let (r, _) = c.request("initialize", params);
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
    assert_eq!((items[0]["data"]["key"].as_str(), items[0]["data"]["args"]["mn"].as_str()), (Some("diag-asm-w104"), Some("LAX")), "{}", items[0]);
    // Hover auf Mnemonic: Opcode-Tabelle mit Zyklen
    let h = hover(&mut c, uri, 2, 10);
    assert!(h.contains("**LDA**") && h.contains("Sofortwert (imm) | $A9 | 2 | 2 |") && h.contains("Absolut, X-indiziert (abx)") && h.contains("4+1"), "{h}");
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
    let d = c.open(uri, ".target atari8\n.org $2000\nstart: sta gtia.colbk\n jmp start\n.run start\n");
    assert!(d["params"]["diagnostics"].as_array().unwrap().is_empty(), "{d}");
    let h = hover(&mut c, uri, 2, 14);
    assert!(h.contains("gtia.colbk") && h.contains("$D01A") && h.contains("schreiben"), "{h}");
    let d = c.open("file:///tmp/vc_lsp_bad.asm", ".target nirgendwo\n nop\n");
    let first = &d["params"]["diagnostics"][0];
    let msg = first["message"].as_str().unwrap().to_string();
    assert!(msg.contains("nirgendwo"), "{msg}");
    // maschinenlesbar: Code und Schlüssel mit Parametern (`data`), unabhängig vom Wortlaut
    assert_eq!((first["code"].as_str(), first["data"]["key"].as_str()), (Some("E000"), Some("diag-lsp-e000-load")), "{first}");
    assert!(first["data"]["args"]["detail"].as_str().unwrap().contains("nirgendwo"), "{first}");
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

/// Hover über dem Mnemonic: der Assembler hat den Modus gewählt; Antwort liefert `vintagecoder.lineMode` und hebt die Tabellenzeile mit „▶“ hervor
fn hover_modus(c: &mut Client, uri: &str, line: u64, ch: u64) -> (String, Value) {
    let (r, _) = c.request("textDocument/hover", json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": ch } }));
    (r["contents"]["value"].as_str().unwrap_or("").to_string(), r["vintagecoder"]["lineMode"].clone())
}

#[test]
fn hover_hebt_die_gewaehlte_adressierungsart_der_zeile_hervor() {
    let mut c = client();
    let uri = "file:///tmp/vc_lsp_modus.asm";
    let src = ".target c64\n.org $0801\nlabel:\n  lda #$01\n  lda $10\n  lda $10,x\n  lda ($10),y\n  lda $1000\n  jmp ($1000)\n  bne label\n  lda zz\nzz = $2000\n";
    let d = c.open(uri, src); assert!(d["params"]["diagnostics"].as_array().unwrap().is_empty(), "{d}");
    // (Zeile, erwartetes Kürzel, Zyklen)
    for (zeile, kuerzel, zyklen) in [(3u64, "imm", 2u64), (4, "zp", 3), (5, "zpx", 4), (6, "izy", 5), (7, "abs", 4), (8, "ind", 5), (9, "rel", 2), (10, "abs", 4)] {
        let (md, m) = hover_modus(&mut c, uri, zeile, 3);
        assert_eq!((m["state"].as_str(), m["mode"].as_str(), m["cycles"].as_u64(), m["chosenBy"].as_str()), (Some("chosen"), Some(kuerzel), Some(zyklen), Some("assembler")), "Zeile {zeile}: {md}");
        let name = vc_target::Mode::parse(kuerzel).unwrap().label();
        assert!(md.contains(&format!("| ▶ {name} |")), "hervorgehobene Zeile fehlt: {md}");
        assert_eq!(md.matches("▶").count(), 1, "genau eine Zeile hervorgehoben: {md}");
        assert!(md.contains("vom Assembler gewählt") && md.contains(&format!("{zyklen} Zyklen")), "{md}");
    }
    // Zeropage gegen Absolut entscheidet der Assembler nach dem Wert: dasselbe Mnemonic, andere Zeile
    let (_, m4) = hover_modus(&mut c, uri, 4, 3); let (_, m7) = hover_modus(&mut c, uri, 7, 3); assert_ne!(m4["mode"], m7["mode"]);
}

#[test]
fn hover_hebt_nichts_hervor_bei_fehler_oder_ohne_lauf() {
    let mut c = client();
    let uri = "file:///tmp/vc_lsp_modus_fehler.asm";
    let d = c.open(uri, ".target c64\n.org $0801\n  lda #$01\n  lda unbekannt\n");
    assert!(!d["params"]["diagnostics"].as_array().unwrap().is_empty());
    let (md, m) = hover_modus(&mut c, uri, 2, 3);
    assert_eq!(m["state"], "open", "{md}"); assert!(!md.contains('▶') && md.contains("Modus noch offen"), "bei Fehlern wird nie geraten: {md}");
}

/// Diagnose und Hover in der Sprache der Anfrage `initialize` (`locale`); `code`, `data.key` und `data.args` bleiben in beiden Sprachen gleich
#[test]
fn language_follows_the_initialize_locale() {
    let src = ".target c64\n.org $0801\n lda #$01\n lax $d012\n";
    let uri = "file:///tmp/vc_lsp_lang.asm";
    for (locale, hover_word, diag_word, diag_other) in [("en", "Load accumulator", "undocumented opcode", "undokumentierter"), ("en-US", "Load accumulator", "undocumented opcode", "undokumentierter"),
                                                         ("de", "Akkumulator laden", "undokumentierter", "undocumented opcode"), ("de_DE.UTF-8", "Akkumulator laden", "undokumentierter", "undocumented opcode")] {
        let mut c = client_in(Some(locale));
        let d = c.open(uri, src);
        let item = &d["params"]["diagnostics"][0];
        assert_eq!((item["code"].as_str(), item["data"]["key"].as_str(), item["data"]["args"]["mn"].as_str()), (Some("W104"), Some("diag-asm-w104"), Some("LAX")), "{locale}: {item}");
        let msg = item["message"].as_str().unwrap();
        assert!(msg.contains(diag_word) && !msg.contains(diag_other), "{locale}: {msg}");
        let h = hover(&mut c, uri, 2, 3);
        assert!(h.contains(hover_word), "{locale}: {h}");
        if locale.starts_with("en") { assert!(h.contains("| Mode | Opcode | Bytes | Cycles |"), "{locale}: {h}"); } else { assert!(h.contains("| Modus | Opcode | Bytes | Zyklen |"), "{locale}: {h}"); }
        let (r, _) = c.request("shutdown", Value::Null);
        assert!(r.is_null());
        c.notify("exit", Value::Null);
        let _ = c.child.wait();
    }
}

/// Ohne `locale` gilt `VC_LANG` des Serverprozesses; eine unbekannte Sprache ergibt Englisch
#[test]
fn language_falls_back_to_vc_lang_and_unknown_means_english() {
    for (vc_lang, locale, want, table) in [(Some("en"), None, "Load accumulator", "| Mode |"), (Some("de"), None, "Akkumulator laden", "| Modus |"), (None, Some("fr-FR"), "Load accumulator", "| Mode |"), (Some("de"), Some("en"), "Load accumulator", "| Mode |")] {
        let mut c = Client::start_with_lang(vc_lang);
        let mut params = json!({ "rootUri": format!("file://{}", root()), "capabilities": {}, "initializationOptions": { "dataDir": format!("{}/data", root()) } });
        if let Some(l) = locale { params["locale"] = json!(l); }
        c.request("initialize", params);
        c.notify("initialized", json!({}));
        let uri = "file:///tmp/vc_lsp_lang2.asm";
        c.open(uri, ".target c64\n.org $0801\n lda #$01\n");
        let h = hover(&mut c, uri, 2, 3);
        assert!(h.contains(want) && h.contains(table), "VC_LANG={vc_lang:?} locale={locale:?}: {h}");
        let (_, _) = c.request("shutdown", Value::Null);
        c.notify("exit", Value::Null);
        let _ = c.child.wait();
    }
}
