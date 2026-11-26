// SPDX-License-Identifier: GPL-3.0-or-later
//! Brücke zu den Sprachdiensten (`vc-lsp`), im Prozess, ohne Rohr: Festlegung in `docs/text-dokumente.md`.
//! Der Server arbeitet auf dem Oberflächenstand. Je offenem Reiter hält die Brücke eine Spiegelung `{version, text}`.
//! Spalten der Oberfläche sind UTF-16-Einheiten (CodeMirror), die des Servers Zeichen; hier wird in beide Richtungen umgerechnet.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vc_lsp::Server;

struct Spiegel { version: u64, text: String }

pub struct Bruecke { server: Server, root: PathBuf, docs: HashMap<String, Spiegel> }

/// Spalte in Zeichen → UTF-16-Einheiten
fn zu16(line: &str, col: usize) -> usize { line.chars().take(col).map(char::len_utf16).sum() }
/// UTF-16-Einheiten → Spalte in Zeichen (mitten in einem Ersatzpaar: davor)
fn zu_zeichen(line: &str, col16: usize) -> usize {
    let mut n = 0;
    for (i, c) in line.chars().enumerate() { if n >= col16 { return i; } n += c.len_utf16(); }
    line.chars().count()
}
fn zeile(text: &str, n: usize) -> &str { text.split('\n').nth(n).unwrap_or("") }

/// Byte-Index im Text zu einem UTF-16-Versatz (ganzes Dokument, `\n`-getrennt)
fn byte_von16(text: &str, off16: usize) -> usize {
    let mut n = 0;
    for (b, c) in text.char_indices() { if n >= off16 { return b; } n += c.len_utf16(); }
    text.len()
}

/// Das Datenverzeichnis: `VC_DATA`, dann neben dem Programm (App-Bündel: `Resources/data`), dann im Arbeitsordner (Entwicklung)
pub fn daten_ordner() -> Option<PathBuf> {
    std::env::var_os("VC_DATA").map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok().and_then(|e| vc_ide::find_data_dir(&e)))
        .or_else(|| std::env::current_dir().ok().and_then(|d| vc_ide::find_data_dir(&d)))
}

impl Bruecke {
    pub fn neu(root: &Path, ziel: Option<&str>, lang: vc_i18n::Lang) -> Bruecke {
        let mut server = Server::new();
        server.set_language(lang);
        let mut opts = json!({});
        if let Some(d) = daten_ordner() { opts["dataDir"] = json!(d.to_string_lossy()); }
        if let Some(z) = ziel { opts["target"] = json!(z); }
        let root_s = root.to_string_lossy().replace('\\', "/");
        let _ = server.handle(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "rootUri": vc_core::paths::file_uri(&root_s), "initializationOptions": opts } }));
        Bruecke { server, root: root.to_path_buf(), docs: HashMap::new() }
    }

    pub fn sprache(&mut self, lang: vc_i18n::Lang) { self.server.set_language(lang); }

    fn uri(&self, rel: &str) -> String { vc_core::paths::file_uri(&self.root.join(rel).to_string_lossy().replace('\\', "/")) }

    /// Öffnen (auch erneut, um nach einem Versionssprung den ganzen Text zu liefern). Ergebnis: Ereignisse `(Name, Nutzlast)`
    pub fn oeffnen(&mut self, rel: &str, text: &str) -> Vec<(&'static str, Value)> {
        self.docs.insert(rel.to_string(), Spiegel { version: 0, text: text.to_string() });
        let m = self.server.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": { "textDocument": { "uri": self.uri(rel), "text": text, "version": 0 } } }));
        self.ereignisse(m)
    }

    /// Änderungen eines Bündels (Spannen im Text **vor** dem Bündel, UTF-16, nicht überlappend). `version` muss die Vorgängerversion + 1 sein.
    pub fn aendern(&mut self, rel: &str, version: u64, changes: &[Value]) -> Result<Vec<(&'static str, Value)>, Fehler> {
        let sp = self.docs.get_mut(rel).ok_or_else(|| Fehler::mit("doc-unbekannt", json!({ "rel": rel })))?;
        if version != sp.version + 1 { return Err(Fehler::mit("doc-version", json!({ "rel": rel, "erwartet": sp.version + 1, "erhalten": version }))); }
        let mut c: Vec<(usize, usize, String)> = changes.iter().map(|x| (x["from"].as_u64().unwrap_or(0) as usize, x["to"].as_u64().unwrap_or(0) as usize, x["insert"].as_str().unwrap_or("").to_string())).collect();
        c.sort_by(|a, b| b.0.cmp(&a.0));   // von hinten nach vorn, damit die Spannen gültig bleiben
        let mut text = sp.text.clone();
        for (f, t, ins) in c { let (bf, bt) = (byte_von16(&text, f), byte_von16(&text, t.max(f))); text.replace_range(bf..bt, &ins); }
        sp.version = version; sp.text = text.clone();
        let m = self.server.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": { "textDocument": { "uri": self.uri(rel), "version": version }, "contentChanges": [{ "text": text }] } }));
        Ok(self.ereignisse(m))
    }

    pub fn schliessen(&mut self, rel: &str) -> Vec<(&'static str, Value)> {
        if self.docs.remove(rel).is_none() { return vec![]; }
        let m = self.server.handle(json!({ "jsonrpc": "2.0", "method": "textDocument/didClose", "params": { "textDocument": { "uri": self.uri(rel) } } }));
        self.ereignisse(m)
    }

    pub fn text(&self, rel: &str) -> Option<&str> { self.docs.get(rel).map(|s| s.text.as_str()) }
    pub fn version(&self, rel: &str) -> Option<u64> { self.docs.get(rel).map(|s| s.version) }

    /// Nachrichten des Servers → Ereignisse der Oberfläche (`lsp:diagnostics`); Zeilenadressen sind Sache von E1.3
    fn ereignisse(&self, msgs: Vec<Value>) -> Vec<(&'static str, Value)> {
        let mut out = vec![];
        for m in msgs {
            if m["method"] != "textDocument/publishDiagnostics" { continue; }
            let uri = m["params"]["uri"].as_str().unwrap_or("");
            let Some(rel) = vc_core::paths::uri_to_rel(&self.root, uri) else { continue };
            let sp = self.docs.get(&rel);
            let items: Vec<Value> = m["params"]["diagnostics"].as_array().map(|a| a.iter().map(|d| self.diagnose(sp.map(|s| s.text.as_str()), d)).collect()).unwrap_or_default();
            out.push(("lsp:diagnostics", json!({ "rel": rel, "version": sp.map(|s| s.version), "items": items })));
        }
        out
    }

    fn diagnose(&self, text: Option<&str>, d: &Value) -> Value {
        let l = d["range"]["start"]["line"].as_u64().unwrap_or(0) as usize;
        let le = d["range"]["end"]["line"].as_u64().unwrap_or(l as u64) as usize;
        let src = text.map(|t| zeile(t, l)).unwrap_or("");
        let (cs, ce) = (d["range"]["start"]["character"].as_u64().unwrap_or(0) as usize, d["range"]["end"]["character"].as_u64().unwrap_or(0) as usize);
        let data = &d["data"];
        // Spalte 0 und Ende am Zeilenende = ganze Zeile: die Oberfläche unterstreicht dann den Code der Zeile
        let mut out = json!({ "line": l, "endLine": le, "from": zu16(src, cs), "to": zu16(if le == l { src } else { text.map(|t| zeile(t, le)).unwrap_or("") }, ce),
                "severity": d["severity"], "code": d["code"], "key": data["key"], "args": data["args"], "message": d["message"] });
        // Include einer Datei aus `build/export/`, die es noch nicht gibt, deren Quelle (Sprite, Zeichensatz …) aber im Projekt liegt: Hinweis „entsteht beim Bauen“ statt Fehler
        if d["code"] == "E212" { if let Some(q) = export_quelle(&self.root, data["args"]["name"].as_str().unwrap_or("")) {
            out["severity"] = json!(3); out["code"] = json!("H212"); out["key"] = json!("diag-export-wird-beim-bauen-erzeugt");
            out["args"] = json!({ "name": data["args"]["name"], "quelle": q });
            out["message"] = json!(vc_i18n::t!("diag-export-wird-beim-bauen-erzeugt", name = data["args"]["name"].as_str().unwrap_or(""), quelle = q.as_str()).to_string());
        } }
        out
    }



    fn pos(&self, rel: &str, line: usize, col16: usize) -> Result<Value, Fehler> {
        let sp = self.docs.get(rel).ok_or_else(|| Fehler::mit("doc-unbekannt", json!({ "rel": rel })))?;
        Ok(json!({ "textDocument": { "uri": self.uri(rel) }, "position": { "line": line, "character": zu_zeichen(zeile(&sp.text, line), col16) } }))
    }

    fn bereich(&self, text: &str, r: &Value) -> Value {
        let (l, le) = (r["start"]["line"].as_u64().unwrap_or(0) as usize, r["end"]["line"].as_u64().unwrap_or(0) as usize);
        json!({ "line": l, "endLine": le, "from": zu16(zeile(text, l), r["start"]["character"].as_u64().unwrap_or(0) as usize), "to": zu16(zeile(text, le), r["end"]["character"].as_u64().unwrap_or(0) as usize) })
    }

    fn antwort(&mut self, m: Value) -> Value { self.server.handle(m).into_iter().find(|x| x.get("id").is_some()).map(|x| x["result"].clone()).unwrap_or(Value::Null) }

    /// Hover: `{ markdown, line, from, to, lineMode? }` oder null
    pub fn hover(&mut self, rel: &str, line: usize, col16: usize) -> Result<Value, Fehler> {
        let p = self.pos(rel, line, col16)?;
        let r = self.antwort(json!({ "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover", "params": p }));
        if r.is_null() { return Ok(Value::Null); }
        let text = self.text(rel).unwrap_or("").to_string();
        let mut o = self.bereich(&text, &r["range"]);
        o["markdown"] = r["contents"]["value"].clone();
        if !r["vintagecoder"]["lineMode"].is_null() { o["lineMode"] = r["vintagecoder"]["lineMode"].clone(); }
        Ok(o)
    }

    /// Vervollständigung: `[ { label, kind, detail, documentation, sortText } ]`; das Filtern nach dem getippten Wort macht die Oberfläche
    pub fn vervollstaendigen(&mut self, rel: &str, line: usize, col16: usize) -> Result<Value, Fehler> {
        let p = self.pos(rel, line, col16)?;
        let r = self.antwort(json!({ "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion", "params": p }));
        Ok(r["items"].clone())
    }

    /// Definition: `{ rel, line, col }` (Datei im Projekt) oder null
    pub fn definition(&mut self, rel: &str, line: usize, col16: usize) -> Result<Value, Fehler> {
        let p = self.pos(rel, line, col16)?;
        let r = self.antwort(json!({ "jsonrpc": "2.0", "id": 4, "method": "textDocument/definition", "params": p }));
        if r.is_null() { return Ok(Value::Null); }
        let Some(ziel) = r["uri"].as_str().and_then(|u| vc_core::paths::uri_to_rel(&self.root, u)) else { return Ok(Value::Null) };
        Ok(json!({ "rel": ziel, "line": r["range"]["start"]["line"], "col": 0 }))
    }

    /// Gliederung: `[ { name, kind, line } ]` (`kind`: label, const, macro)
    pub fn gliederung(&mut self, rel: &str) -> Result<Value, Fehler> {
        if !self.docs.contains_key(rel) { return Err(Fehler::mit("doc-unbekannt", json!({ "rel": rel }))); }
        let r = self.antwort(json!({ "jsonrpc": "2.0", "id": 5, "method": "textDocument/documentSymbol", "params": { "textDocument": { "uri": self.uri(rel) } } }));
        Ok(Value::Array(r.as_array().cloned().unwrap_or_default().into_iter().map(|s| json!({
            "name": s["name"], "line": s["location"]["range"]["start"]["line"],
            "kind": match s["kind"].as_u64() { Some(14) => "const", Some(6) => "macro", _ => "label" } })).collect()))
    }

    /// Quick-Fixes für Zeilen `von..=bis`: `[ { title, edits: [ { line, from, to, text } ] } ]`
    pub fn korrekturen(&mut self, rel: &str, von: usize, bis: usize) -> Result<Value, Fehler> {
        if !self.docs.contains_key(rel) { return Err(Fehler::mit("doc-unbekannt", json!({ "rel": rel }))); }
        let uri = self.uri(rel);
        let r = self.antwort(json!({ "jsonrpc": "2.0", "id": 6, "method": "textDocument/codeAction", "params": { "textDocument": { "uri": uri }, "range": { "start": { "line": von, "character": 0 }, "end": { "line": bis, "character": 0 } } } }));
        let text = self.text(rel).unwrap_or("").to_string();
        Ok(Value::Array(r.as_array().cloned().unwrap_or_default().into_iter().map(|a| {
            let edits: Vec<Value> = a["edit"]["changes"][&uri].as_array().cloned().unwrap_or_default().iter().map(|e| {
                let mut b = self.bereich(&text, &e["range"]); b["text"] = e["newText"].clone(); b }).collect();
            json!({ "title": a["title"], "edits": edits })
        }).collect()))
    }
}

/// Mediendatei im Projekt, aus der `build/export/<name>.…` beim Bauen entsteht (gleicher Name ohne Endungen); `None`, wenn es keine gibt
pub fn export_quelle(root: &Path, name: &str) -> Option<String> {
    let rest = name.replace('\\', "/"); let rest = rest.split("build/export/").nth(1)?;
    let stem = rest.split('.').next().filter(|s| !s.is_empty())?;
    fn suche(root: &std::path::Path, dir: &std::path::Path, stem: &str, tiefe: usize) -> Option<String> {
        for e in std::fs::read_dir(dir).ok()?.flatten() {
            let (p, n) = (e.path(), e.file_name().to_string_lossy().into_owned());
            if p.is_dir() { if tiefe < 6 && !n.starts_with('.') && !matches!(n.as_str(), "build" | "target" | "node_modules") { if let Some(r) = suche(root, &p, stem, tiefe + 1) { return Some(r); } } }
            else if vc_media::naming::is_data_file(&n) && vc_media::naming::stem(&n) == stem { return p.strip_prefix(root).ok().map(|r| r.to_string_lossy().replace('\\', "/")); }
        }
        None
    }
    suche(root, root, stem, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn projekt(n: &str, dateien: &[(&str, &str)]) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vc_lsp_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap();
        for (r, t) in dateien { std::fs::write(d.join(r), t).unwrap(); }
        d
    }
    fn daten() -> Option<PathBuf> { daten_ordner() }

    #[test] fn umrechnung_utf16_und_zeichen() {
        let l = "a😀b"; assert_eq!(zu16(l, 2), 3); assert_eq!(zu_zeichen(l, 3), 2); assert_eq!(zu_zeichen(l, 1), 1); assert_eq!(zu_zeichen(l, 99), 3);
        assert_eq!(byte_von16("a😀b", 3), 5);
    }
    #[test] fn aendern_wendet_buendel_von_hinten_an_und_prueft_version() {
        let r = projekt("aendern", &[("a.asm", "lda #1\n")]);
        let mut b = Bruecke::neu(&r, Some("c64"), vc_i18n::Lang::De);
        b.oeffnen("a.asm", "lda #1\nrts\n");
        b.aendern("a.asm", 1, &[json!({ "from": 5, "to": 6, "insert": "2" }), json!({ "from": 7, "to": 10, "insert": "nop" })]).unwrap();
        assert_eq!(b.text("a.asm"), Some("lda #2\nnop\n"));
        assert_eq!(b.aendern("a.asm", 5, &[]).unwrap_err().code, "doc-version");
        assert_eq!(b.aendern("x.asm", 1, &[]).unwrap_err().code, "doc-unbekannt");
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn include_aus_build_export_ohne_datei_hat_eine_quelle() {
        let dir = std::env::temp_dir().join(format!("vc_lsp_exp_{}", std::process::id())); let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(dir.join("graphics")).unwrap();
        std::fs::write(dir.join("graphics/hg.vcsprite"), "{}").unwrap();
        assert_eq!(export_quelle(&dir, "build/export/hg.asm").as_deref(), Some("graphics/hg.vcsprite"));
        assert_eq!(export_quelle(&dir, "build/export/hg.anim.asm").as_deref(), Some("graphics/hg.vcsprite"));
        assert_eq!(export_quelle(&dir, "build/export/gibtsnicht.asm"), None);
        assert_eq!(export_quelle(&dir, "lib/hg.asm"), None, "nur Dateien aus build/export");
        std::fs::remove_dir_all(dir).ok();
    }
    #[test] fn diagnose_traegt_schluessel_args_und_utf16_spalten() {
        if daten().is_none() { eprintln!("kein data-Ordner: übersprungen"); return; }
        let r = projekt("diag", &[("a.asm", "")]);
        let mut b = Bruecke::neu(&r, Some("c64"), vc_i18n::Lang::De);
        let ev = b.oeffnen("a.asm", ".org $0801\nstart: lda #$00\n  lda unbekannt ; 😀\n  rts\n");
        let d = ev.iter().find(|e| e.0 == "lsp:diagnostics" && e.1["rel"] == "a.asm").expect("Diagnose-Ereignis");
        assert_eq!(d.1["version"], 0);
        let items = d.1["items"].as_array().unwrap(); assert!(!items.is_empty(), "unbekanntes Symbol muss gemeldet werden: {:?}", d.1);
        assert!(items[0]["key"].as_str().unwrap().starts_with("diag-"), "{:?}", items[0]);
        assert_eq!(items[0]["line"], 2);
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn hover_vervollstaendigung_definition_gliederung_korrektur() {
        if daten().is_none() { eprintln!("kein data-Ordner: übersprungen"); return; }
        let r = projekt("anfr", &[("a.asm", "")]);
        let mut b = Bruecke::neu(&r, Some("c64"), vc_i18n::Lang::De);
        b.oeffnen("a.asm", ".org $0801\nstart: lda #$00\n  jmp start\n.pseudopc $c000\n.endpseudopc\n");
        let h = b.hover("a.asm", 1, 8).unwrap(); assert!(h["markdown"].as_str().unwrap().contains("LDA"), "{h}");
        let m = b.hover("a.asm", 2, 8).unwrap(); assert!(m["markdown"].as_str().unwrap().contains("Definiert"), "Marke nennt die Fundstelle: {m}");
        let v = b.vervollstaendigen("a.asm", 2, 4).unwrap(); let labels: Vec<&str> = v.as_array().unwrap().iter().filter_map(|i| i["label"].as_str()).collect();
        assert!(labels.contains(&"lda") && labels.contains(&"start"), "{labels:?}");
        let d = b.definition("a.asm", 2, 7).unwrap(); assert_eq!(d["line"], 1); assert_eq!(d["rel"], "a.asm");
        let g = b.gliederung("a.asm").unwrap(); assert!(g.as_array().unwrap().iter().any(|s| s["name"] == "start" && s["kind"] == "label"));
        let k = b.korrekturen("a.asm", 3, 3).unwrap(); assert!(!k.as_array().unwrap().is_empty(), "alter Direktivenname braucht Quick-Fix"); assert!(k[0]["edits"][0]["text"].is_string());
        let _ = std::fs::remove_dir_all(&r);
    }
}
