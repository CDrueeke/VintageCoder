// SPDX-License-Identifier: GPL-3.0-or-later
//! Konsistenz der Direktivennamen: Tabelle (`data/asm/directives.json`), Sprachserver (Beschreibungen in beiden Sprachen,
//! Vervollständigung, Hover, Quick-Fix) und Hilfeseiten stehen gegenseitig vollständig.
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;
use vc_asm::directives;
use vc_i18n::Lang;
use vc_lsp::{directive_doc_in, directive_names, quickfix::alias_fixes, Server};

fn names() -> BTreeSet<&'static str> { directives::canonical_names().into_iter().collect() }

#[test]
fn language_server_knows_every_name_of_the_table_in_both_languages() {
    assert_eq!(directive_names().into_iter().collect::<BTreeSet<_>>(), names());
    for n in names() {
        for lang in [Lang::De, Lang::En] {
            let d = directive_doc_in(lang, n).unwrap_or_else(|| panic!("keine Beschreibung für {n} ({lang:?})"));
            assert!(!d.trim().is_empty(), "{n}");
        }
    }
    // Alias liefern die Beschreibung des kanonischen Namens
    for (a, c) in directives::aliases() { assert_eq!(directive_doc_in(Lang::De, a), directive_doc_in(Lang::De, c), "{a}"); }
    // jeder Katalogschlüssel der Tabelle (summary_id) existiert; Blockenden heißen `lsp-directive-<end>`
    let cat = |l: &str| std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../vc-i18n/catalog/{l}/lsp.ftl"))).unwrap();
    for l in ["de", "en"] {
        let c = cat(l);
        for d in directives::table() {
            for k in std::iter::once(d.summary_id.clone()).chain(d.end.iter().map(|e| format!("lsp-directive-{}", &e[1..]))) {
                assert!(c.lines().any(|x| x.starts_with(&format!("{k} ="))), "{l}: Schlüssel {k} fehlt");
            }
        }
        // und kein Eintrag ohne Tabellenzeile
        for line in c.lines().filter(|x| x.starts_with("lsp-directive-")) {
            let key = line.split(" =").next().unwrap();
            assert!(names().contains(format!(".{}", &key["lsp-directive-".len()..]).as_str()), "{l}: {key} steht nicht in der Tabelle");
        }
    }
}

#[test]
fn every_name_of_the_table_has_a_help_page_entry() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vc-ide/help");
    let mut all = String::new();
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map_or(false, |x| x == "md") && p.file_name().unwrap().to_string_lossy().starts_with("asm-") || p.file_name().map_or(false, |n| n == "cartridges.md") { all += &std::fs::read_to_string(p).unwrap(); }
    }
    for n in names() {
        assert!(all.contains(&format!("`{n}`")) || all.contains(&format!("`{n} ")), "Hilfe: {n} fehlt");
    }
    // alle alten Namen stehen in der Tabelle „Alte Namen“
    for (a, _) in directives::aliases() { assert!(all.contains(&format!("`{a}`")), "Hilfe: alter Name {a} fehlt"); }
}

fn req(s: &mut Server, id: u64, method: &str, params: Value) -> Value {
    let out = s.handle(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
    out.into_iter().find(|m| m["id"] == json!(id)).map(|m| m["result"].clone()).unwrap_or(Value::Null)
}

fn server_with(text: &str) -> (Server, String) {
    let mut s = Server::new();
    s.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "locale": "de" } }));
    let uri = "file:///tmp/vc_directive_names.asm".to_string();
    s.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": uri, "text": text } } }));
    (s, uri)
}

#[test]
fn completion_lists_canonical_names_first_and_old_names_after() {
    let (mut s, uri) = server_with(".org $1000\n");
    let r = req(&mut s, 2, "textDocument/completion", json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 0 } }));
    let items = r["items"].as_array().unwrap();
    let sort = |label: &str| items.iter().find(|i| i["label"] == label).unwrap_or_else(|| panic!("{label}"))["sortText"].as_str().unwrap().to_string();
    for (c, a) in [(".relocate", ".pseudopc"), (".retaddr", ".rta"), (".lohitable", ".lohifill"), (".screencode", ".screen"), (".textnull", ".null"), (".endfor", ".next"), (".dword", ".long")] {
        assert!(sort(c) < sort(a), "{c} muss vor {a} stehen");
        let detail = items.iter().find(|i| i["label"] == a).unwrap()["detail"].as_str().unwrap();
        assert!(detail.contains(c), "{a}: {detail}");
    }
    assert!(items.iter().any(|i| i["label"] == ".be32"));
}

#[test]
fn hover_on_an_old_name_shows_the_page_of_the_canonical_name() {
    let (mut s, uri) = server_with(".org $1000\n.pseudopc $0300\n nop\n.endpseudopc\n.relocate $0300\n.endrelocate\n");
    let hover = |s: &mut Server, line: u64| req(s, 3, "textDocument/hover", json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": 3 } }))["contents"]["value"].as_str().unwrap_or("").to_string();
    let old = hover(&mut s, 1);
    let new = hover(&mut s, 4);
    assert!(old.starts_with("**.relocate**") && old.contains("Auch:") && old.contains("`.pseudopc`"), "{old}");
    assert_eq!(old, new);
    assert!(hover(&mut s, 3).contains("`.realpc`"), "Ende: alte Namen des Endes");
}

#[test]
fn quick_fix_replaces_old_names_and_changes_nothing_else() {
    let src = "main: .pseudopc $0300 ; Kommentar .rta\n nop\n.endpseudopc\n.rta a, b\n.page\n.PAGE 64 ; neue Seite\n.screen \"x .null\"\n.for i = 0, i < 2, i += 1\n.next\n.relocate $10\n.endrelocate\n.textnull \"ok\"\nl: .shift \"a\"\n";
    let fx = alias_fixes(src, None);
    let got: Vec<(usize, &str, &str)> = fx.iter().map(|f| (f.line, f.old.as_str(), f.new_text.as_str())).collect();
    assert_eq!(got, vec![(0, ".pseudopc", ".relocate"), (2, ".endpseudopc", ".endrelocate"), (3, ".rta", ".retaddr"), (4, ".page", ".align 256, skip"), (5, ".PAGE", ".align 64, skip"),
                         (6, ".screen", ".screencode"), (8, ".next", ".endfor"), (12, ".shift", ".textshift")]);
    // angewendet ergibt das Ergebnis denselben Quelltext in neuer Schreibweise (Zeilenweise von hinten ersetzen)
    let mut lines: Vec<String> = src.lines().map(String::from).collect();
    for f in fx.iter().rev() {
        let c: Vec<char> = lines[f.line].chars().collect();
        lines[f.line] = format!("{}{}{}", c[..f.start].iter().collect::<String>(), f.new_text, c[f.end..].iter().collect::<String>());
    }
    assert_eq!(lines[0], "main: .relocate $0300 ; Kommentar .rta");
    assert_eq!(lines[5], ".align 64, skip ; neue Seite");
    assert_eq!(lines[6], ".screencode \"x .null\"");
    // über den Server: Aktionen für eine Zeile und für die ganze Datei
    let (mut s, uri) = server_with(src);
    let r = req(&mut s, 4, "textDocument/codeAction", json!({ "textDocument": { "uri": uri }, "range": { "start": { "line": 3, "character": 0 }, "end": { "line": 3, "character": 0 } }, "context": { "diagnostics": [] } }));
    let acts = r.as_array().unwrap();
    assert_eq!(acts.len(), 2, "{r}");
    assert!(acts[0]["title"].as_str().unwrap().contains(".rta") && acts[0]["title"].as_str().unwrap().contains(".retaddr"), "{r}");
    assert_eq!(acts[0]["edit"]["changes"][&uri][0]["newText"], ".retaddr");
    assert_eq!(acts[1]["edit"]["changes"][&uri].as_array().unwrap().len(), 8);
    // keine alten Namen: keine Aktion
    let (mut s, uri) = server_with(".org $1000\n.relocate $20\n.endrelocate\n");
    let r = req(&mut s, 4, "textDocument/codeAction", json!({ "textDocument": { "uri": uri }, "range": { "start": { "line": 1, "character": 0 }, "end": { "line": 2, "character": 0 } }, "context": { "diagnostics": [] } }));
    assert!(r.as_array().unwrap().is_empty());
}
