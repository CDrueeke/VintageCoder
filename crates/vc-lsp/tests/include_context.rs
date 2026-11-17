// SPDX-License-Identifier: GPL-3.0-or-later
//! Eingebundene Dateien werden im Kontext der einbindenden Wurzel geprüft (Beispielprojekt: Wurzel, Include, Unter-Include, Waise, Zyklus).
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_lsp::Server;

fn data_dir() -> String { format!("{}/../../data", env!("CARGO_MANIFEST_DIR")) }

struct Proj { dir: PathBuf, srv: Server }

impl Proj {
    /// Legt ein Projekt im Temp-Ordner an (`files`: relativer Pfad → Inhalt) und startet einen Sprachdienst darauf.
    fn new(name: &str, files: &[(&str, &str)]) -> Proj {
        let dir = std::env::temp_dir().join(format!("vc_inc_ctx_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (f, t) in files {
            let p = dir.join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, t).unwrap();
        }
        let mut srv = Server::new();
        srv.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "rootUri": uri(&dir), "capabilities": {}, "initializationOptions": { "dataDir": data_dir() } } }));
        Proj { dir, srv }
    }
    fn uri(&self, rel: &str) -> String { uri(&self.dir.join(rel)) }
    fn disk_text(&self, rel: &str) -> String { std::fs::read_to_string(self.dir.join(rel)).unwrap() }
    fn open(&mut self, rel: &str) -> Vec<Value> { let t = self.disk_text(rel); self.open_text(rel, &t) }
    fn open_text(&mut self, rel: &str, text: &str) -> Vec<Value> {
        let u = self.uri(rel);
        self.srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": u, "languageId": "vcasm", "version": 1, "text": text } } }))
    }
    fn change(&mut self, rel: &str, text: &str) -> Vec<Value> {
        let u = self.uri(rel);
        self.srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": u, "version": 2 }, "contentChanges": [{ "text": text }] } }))
    }
    fn close(&mut self, rel: &str) -> Vec<Value> {
        let u = self.uri(rel);
        self.srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didClose", "params": { "textDocument": { "uri": u } } }))
    }
    fn request(&mut self, method: &str, rel: &str, line: u64, ch: u64) -> Value {
        let u = self.uri(rel);
        let r = self.srv.handle(json!({ "jsonrpc": "2.0", "id": 7, "method": method, "params": { "textDocument": { "uri": u }, "position": { "line": line, "character": ch } } }));
        r.into_iter().find(|m| m["id"] == json!(7)).map(|m| m["result"].clone()).unwrap_or(Value::Null)
    }
}

impl Drop for Proj { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.dir); } }

fn uri(p: &Path) -> String { vc_core::paths::file_uri(&p.to_string_lossy()) }

/// Diagnosen (Code, Schweregrad, Zeile 0-basiert) der letzten Veröffentlichung für `rel`; `None`, wenn nichts für die Datei gesendet wurde.
fn diags(p: &Proj, msgs: &[Value], rel: &str) -> Option<Vec<(String, i64, i64)>> {
    let u = p.uri(rel);
    msgs.iter().filter(|m| m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"] == json!(u)).last().map(|m| {
        m["params"]["diagnostics"].as_array().unwrap().iter().map(|d| (d["code"].as_str().unwrap().to_string(), d["severity"].as_i64().unwrap(), d["range"]["start"]["line"].as_i64().unwrap())).collect()
    })
}

fn addresses(p: &Proj, msgs: &[Value], rel: &str) -> Option<Vec<(i64, i64, i64)>> {
    let u = p.uri(rel);
    msgs.iter().find(|m| m["method"] == "vintagecoder/addresses" && m["params"]["uri"] == json!(u))
        .map(|m| m["params"]["lines"].as_array().unwrap().iter().map(|l| (l[0].as_i64().unwrap(), l[1].as_i64().unwrap(), l[2].as_i64().unwrap())).collect())
}

const VCPRJ: &str = r#"{ "format": 1, "project": {"name": "t"}, "targets": [{"id": "c64", "system": "c64"}],
  "sources": [{"targets": ["c64"], "kind": "asm", "files": ["main.asm", "other.asm"]}], "main": "main.asm" }"#;

/// Wurzel `main.asm` definiert Konstante, Marke und Makro; `lib/a.asm` nutzt sie und bindet `lib/sub/b.asm` ein.
const MAIN: &str = ".target c64\n.org $0801\nSTART = $c000\n.macro WAIT(n)\n  ldx #n\n.endmacro\nmain:\n  jsr aufruf\n  jmp ende\n.include \"lib/a.asm\"\nende:\n  rts\n";
const A: &str = "aufruf:\n  WAIT(3)\n  lda #<START\n  jsr tief\n  jmp main\n#include \"sub/b.asm\"\n";
const B: &str = "tief:\n  lda #<START\n  sta $d020\n  rts\n";

fn standard(name: &str) -> Proj {
    Proj::new(name, &[("p.vcprj", VCPRJ), ("main.asm", MAIN), ("lib/a.asm", A), ("lib/sub/b.asm", B)])
}

#[test]
fn include_file_alone_has_no_follow_up_errors_in_context() {
    let mut p = standard("kontext");
    // ohne Kontext wäre `aufruf` voller Fehler (START, main, WAIT, .org); im Kontext der Wurzel ist alles gut
    let m = p.open("lib/a.asm");
    assert_eq!(diags(&p, &m, "lib/a.asm"), Some(vec![]), "{m:?}");
    // die Wurzel ist nicht offen, bekommt aber ihre (leere) Veröffentlichung
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![]));
    let m = p.open("lib/sub/b.asm");
    assert_eq!(diags(&p, &m, "lib/sub/b.asm"), Some(vec![]), "Unter-Include über zwei Ebenen: {m:?}");
}

#[test]
fn unsaved_text_of_the_include_file_is_used_and_errors_land_in_their_own_file() {
    let mut p = standard("ungespeichert");
    p.open("main.asm");
    p.open("lib/a.asm");
    // Tippfehler in a.asm (nicht gespeichert): Fehler in a.asm, nicht in main.asm
    let m = p.change("lib/a.asm", &A.replace("jsr tief", "jsr tieef"));
    assert_eq!(diags(&p, &m, "lib/a.asm"), Some(vec![("E202".into(), 1, 3)]), "{m:?}");
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![]));
    // Behoben: weg
    let m = p.change("lib/a.asm", A);
    assert_eq!(diags(&p, &m, "lib/a.asm"), Some(vec![]));
    // Fehler der Wurzel (Include-Datei fehlt) erscheint in der Wurzel, auch wenn nur die Include-Datei offen ist
    let m = p.change("main.asm", &MAIN.replace("lib/a.asm", "lib/fehlt.asm"));
    assert_eq!(diags(&p, &m, "main.asm").unwrap().iter().map(|d| d.0.as_str()).collect::<Vec<_>>().first(), Some(&"E212"));
}

#[test]
fn root_error_shows_up_in_its_file_while_only_the_include_is_open() {
    let mut p = Proj::new("wurzelfehler", &[("p.vcprj", VCPRJ), ("main.asm", &MAIN.replace("jmp ende", "jmp nirgends")), ("lib/a.asm", A), ("lib/sub/b.asm", B)]);
    let m = p.open("lib/a.asm");
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![("E202".into(), 1, 8)]), "{m:?}");
    assert_eq!(diags(&p, &m, "lib/a.asm"), Some(vec![]));
    // Datei schließen: Veröffentlichungen verschwinden
    let m = p.close("lib/a.asm");
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![]), "{m:?}");
}

#[test]
fn addresses_of_the_include_file_come_from_the_context_run() {
    let mut p = standard("adressen");
    let m = p.open("lib/a.asm");
    let a = addresses(&p, &m, "lib/a.asm").expect("Adressen der Include-Datei");
    // main: jsr aufruf (3) + jmp ende (3) → a.asm beginnt bei $0801+6, das Makro WAIT(3) belegt 2 Bytes
    assert_eq!(a[0], (2, 0x0809, 2), "{a:?}");   // `lda #<START` (WAIT expandiert in main.asm)
    assert!(a.iter().all(|l| l.0 >= 1));
    let m = p.open("lib/sub/b.asm");
    let b = addresses(&p, &m, "lib/sub/b.asm").unwrap();
    assert_eq!(b[0].0, 1);
    assert!(b[0].1 > 0x0809);
}

#[test]
fn hover_definition_and_completion_know_the_context_symbols() {
    let mut p = standard("symbole");
    p.open("lib/a.asm");
    // Hover auf START (Zeile 2 `lda #<START`) und auf die Marke `main` (Zeile 4)
    let h = p.request("textDocument/hover", "lib/a.asm", 2, 10);
    assert!(h["contents"]["value"].as_str().unwrap_or("").contains("C000"), "{h}");
    let d = p.request("textDocument/definition", "lib/a.asm", 4, 8);
    assert_eq!(d["uri"], json!(p.uri("main.asm")));
    assert_eq!(d["range"]["start"]["line"], json!(6));
    let d = p.request("textDocument/definition", "lib/a.asm", 2, 10);
    assert_eq!(d["range"]["start"]["line"], json!(2));
    let c = p.request("textDocument/completion", "lib/a.asm", 0, 0);
    let labels: Vec<&str> = c["items"].as_array().unwrap().iter().filter_map(|i| i["label"].as_str()).collect();
    for want in ["START", "main", "ende", "wait"] { assert!(labels.contains(&want), "{want} fehlt: {labels:?}"); }
}

#[test]
fn root_is_main_when_it_reaches_the_file_otherwise_shortest_chain_then_path() {
    // other.asm bindet a.asm ebenfalls ein (kürzere Kette zu b.asm wäre über other.asm: other → sub/b.asm direkt)
    let other = ".target c64\n.org $0801\nSTART = $c000\n  nop\n.include \"lib/sub/b.asm\"\n";
    let mut p = Proj::new("mehrere", &[("p.vcprj", VCPRJ), ("main.asm", MAIN), ("other.asm", other), ("lib/a.asm", A), ("lib/sub/b.asm", B)]);
    // b.asm: main erreicht es (über a.asm) und ist das Startprogramm → Kontext main (tief/aufruf kommen von dort)
    let m = p.open("lib/sub/b.asm");
    assert_eq!(diags(&p, &m, "lib/sub/b.asm"), Some(vec![]));
    assert!(diags(&p, &m, "main.asm").is_some() && diags(&p, &m, "other.asm").is_none(), "Wurzel ist main");
    // ohne Projektdatei: kürzeste Kette (other → b) gewinnt vor main → a → b
    let mut q = Proj::new("mehrere_ohne", &[("main.asm", MAIN), ("other.asm", other), ("lib/a.asm", A), ("lib/sub/b.asm", B)]);
    let m = q.open("lib/sub/b.asm");
    assert!(diags(&q, &m, "other.asm").is_some() && diags(&q, &m, "main.asm").is_none(), "kürzeste Kette");
    assert_eq!(diags(&q, &m, "lib/sub/b.asm"), Some(vec![]));
}

#[test]
fn startprogram_and_unincluded_files_stay_isolated() {
    let mut p = standard("isoliert");
    // main.asm: wie bisher allein geprüft
    let m = p.open("main.asm");
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![]));
    // other.asm: Quelle des Projekts, von nichts eingebunden: echte Fehler bleiben Fehler
    let m = p.open_text("other.asm", ".target c64\n.org $0801\n jmp unbekannt\n");
    assert_eq!(diags(&p, &m, "other.asm"), Some(vec![("E202".into(), 1, 2)]));
}

#[test]
fn orphan_library_file_gets_hints_instead_of_follow_up_errors() {
    let mut p = Proj::new("waise", &[("p.vcprj", VCPRJ), ("main.asm", MAIN), ("lib/frei.asm", "")]);
    // keine .org/.target/.basicstub, nicht eingebunden, nicht in der Projektdatei: Symbole fehlen vermutlich nur, weil die einbindende Datei fehlt
    let m = p.open_text("lib/frei.asm", "frei:\n  lda #<UNBEKANNT\n  jsr makro_x\n  nop\n  rts\n");
    let d = diags(&p, &m, "lib/frei.asm").unwrap();
    assert!(!d.is_empty() && d.iter().all(|x| x.1 == 3), "nur Hinweise: {d:?}");
    // .org vorhanden: eigenständiges Programm, echte Fehler
    let m = p.change("lib/frei.asm", ".org $c000\n  jmp unbekannt\n");
    assert_eq!(diags(&p, &m, "lib/frei.asm"), Some(vec![("E202".into(), 1, 1)]));
    // Waise mit eigenem Syntaxfehler: bleibt Fehler (nur Kontextfehler werden abgestuft)
    let m = p.change("lib/frei.asm", "frei:\n  lda #1\n  lda\n");
    assert!(diags(&p, &m, "lib/frei.asm").unwrap().iter().any(|x| x.1 == 1));
}

#[test]
fn cycles_without_entry_and_through_the_root_are_defined() {
    // Zyklus ohne Einstieg: x ↔ y, beide eingebunden, keine Wurzel → keine Endlosschleife, Meldung des Assemblers
    let mut p = Proj::new("zyklus", &[("x.asm", ".target c64\n.org $0801\n.include \"y.asm\"\n"), ("y.asm", ".include \"x.asm\"\n nop\n")]);
    let m = p.open("y.asm");
    let d = diags(&p, &m, "y.asm").unwrap();
    assert!(d.iter().any(|x| x.0 == "E211"), "Zyklus gemeldet: {d:?}");
    // Zyklus über die Wurzel mit #include (einmal): kein Fehler, a gilt im Kontext von main
    let mut q = Proj::new("zyklus2", &[("main.asm", ".target c64\n.org $0801\n#include \"a.asm\"\n  rts\n"), ("a.asm", "#include \"b.asm\"\nla: nop\n"), ("b.asm", "#include \"a.asm\"\n  jmp la\n")]);
    let m = q.open("b.asm");
    assert_eq!(diags(&q, &m, "b.asm"), Some(vec![]), "{m:?}");
    // mit .include (mehrfach): Zyklus als Fehler, nicht als Absturz oder Hängen
    let mut r = Proj::new("zyklus3", &[("main.asm", ".target c64\n.org $0801\n.include \"a.asm\"\n"), ("a.asm", ".include \"b.asm\"\n"), ("b.asm", ".include \"a.asm\"\n")]);
    let m = r.open("b.asm");
    assert!(diags(&r, &m, "b.asm").unwrap().iter().chain(diags(&r, &m, "a.asm").unwrap_or_default().iter()).any(|x| x.0 == "E211"), "{m:?}");
}

#[test]
fn unresolvable_include_is_reported_in_the_including_file() {
    let mut p = Proj::new("fehlt", &[("main.asm", ".target c64\n.org $0801\n.include \"weg.asm\"\n nop\n")]);
    let m = p.open("main.asm");
    assert_eq!(diags(&p, &m, "main.asm"), Some(vec![("E212".into(), 1, 2)]));
}

#[test]
fn shared_include_gets_the_union_of_root_diagnostics_and_changing_includes_moves_the_root() {
    let lib = "gemeinsam:\n  nop\n  jmp nur_in_eins\n";
    let r1 = ".target c64\n.org $0801\nnur_in_eins:\n.include \"lib.asm\"\n";
    let r2 = ".target c64\n.org $0801\n.include \"lib.asm\"\n";
    let mut p = Proj::new("gemeinsam", &[("r1.asm", r1), ("r2.asm", r2), ("lib.asm", lib)]);
    // beide Wurzeln offen: r1 (ohne Fehler) und r2 (Fehler in lib.asm); lib.asm zeigt den Fehler von r2
    p.open("r1.asm");
    let m = p.open("r2.asm");
    assert_eq!(diags(&p, &m, "lib.asm"), Some(vec![("E202".into(), 1, 2)]), "{m:?}");
    // r2 bindet lib nicht mehr ein: der Fehler verschwindet
    let m = p.change("r2.asm", ".target c64\n.org $0801\n nop\n");
    assert_eq!(diags(&p, &m, "lib.asm"), Some(vec![]), "{m:?}");
    // r1 öffnen/ändern berührt weiter nur den Lauf von r1
    let m = p.open("lib.asm");
    assert_eq!(diags(&p, &m, "lib.asm"), Some(vec![]));
}

#[test]
fn context_change_costs_little() {
    // Messung: Änderungen in einer Include-Datei des Beispielprojekts prallball (rund 4000 Zeilen, Wurzel + 7 Includes)
    let root = PathBuf::from(format!("{}/../../examples/c128/prallball", env!("CARGO_MANIFEST_DIR")));
    if !root.join("prallball.asm").is_file() { return; }
    let mut srv = Server::new();
    srv.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "rootUri": uri(&root), "capabilities": {}, "initializationOptions": { "dataDir": data_dir() } } }));
    let lib = root.join("lib/sound.asm");
    let text = std::fs::read_to_string(&lib).unwrap();
    let u = uri(&lib);
    let t0 = std::time::Instant::now();
    let m = srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": u, "languageId": "vcasm", "version": 1, "text": text } } }));
    let first = t0.elapsed();
    let errs = |m: &[Value]| m.iter().flat_map(|x| x["params"]["diagnostics"].as_array().cloned().unwrap_or_default()).filter(|d| d["severity"] == json!(1)).count();
    assert_eq!(errs(&m), 0, "{m:?}");
    assert!(m.iter().any(|x| x["method"] == "vintagecoder/addresses" && x["params"]["lines"].as_array().map_or(false, |l| !l.is_empty())), "Adressen der Include-Datei");
    let t0 = std::time::Instant::now();
    for i in 0..5 {
        let m = srv.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": u, "version": 2 + i }, "contentChanges": [{ "text": format!("{text}\n; {i}") }] } }));
        assert_eq!(errs(&m), 0);
    }
    let per = t0.elapsed() / 5;
    eprintln!("Kontextlauf prallball (Debug-Build): Öffnen {first:?}, danach {per:?} je Änderung");
    assert!(per.as_secs() < 5, "{per:?}");
}

#[test]
fn hover_im_include_zeigt_den_modus_aus_dem_kontextlauf_und_bei_waisen_nichts() {
    let mut p = Proj::new("modus", &[("main.asm", ".target c64\n.org $0801\n.include \"lib.asm\"\n"), ("lib.asm", "  lda $10,x\n  lda $1000\n"), ("waise.asm", "  lda $10,x\n")]);
    p.open("main.asm"); p.open("lib.asm"); p.open("waise.asm");
    let h = p.request("textDocument/hover", "lib.asm", 0, 3);
    assert_eq!((h["vintagecoder"]["lineMode"]["state"].as_str(), h["vintagecoder"]["lineMode"]["mode"].as_str()), (Some("chosen"), Some("zpx")), "{h}");
    assert!(h["contents"]["value"].as_str().unwrap().contains("▶ Zeropage, X-indiziert (zpx)"));
    let h = p.request("textDocument/hover", "lib.asm", 1, 3);
    assert_eq!(h["vintagecoder"]["lineMode"]["mode"].as_str(), Some("abs"));
    // Waise (keine .org/.target, keine Wurzel): kein Modus geraten
    let h = p.request("textDocument/hover", "waise.asm", 0, 3);
    assert!(h["vintagecoder"]["lineMode"]["state"] != json!("chosen") || h["vintagecoder"]["lineMode"]["mode"] == json!("zpx"), "{h}");
}
