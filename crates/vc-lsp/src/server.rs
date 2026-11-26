// SPDX-License-Identifier: GPL-3.0-or-later
//! LSP-Server: Dokumentverwaltung, Analyse und Antworten auf Editor-Anfragen.
use crate::docs::{directive_doc_in, directive_names, mnemonic_doc_in};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use crate::project::{DiskCache, Project};
use vc_asm::{assemble, line_parts, norm_path, outline, OutlineKind, Options, Output, PartKind, Policy};
use vc_core::{msg, Diagnostic, Msg};
use vc_i18n::{Choice, Lang};
use vc_target::{load_chip, load_cpu, load_target, Chip, Cpu, Target};

struct Ctx { cpu: Cpu, #[allow(dead_code)] target: Target, chips: Vec<Chip>, predefined: Vec<(String, i64)> }

/// Ein offenes Dokument. `root` ist die Datei, in deren Kontext es geprüft wird (ohne Einbinder: es selbst).
/// Was der Assembler für die Zeile unter dem Mauszeiger gewählt hat
enum LineMode { Chosen { mode: vc_target::Mode, opcode: u8, bytes: Vec<u8>, cycles: u8 }, Open(Msg), Ambiguous }

struct Doc { text: String, path: String, includes: Vec<String>, root: String }

/// Ergebnis eines Assemblerlaufs über eine Wurzeldatei samt allem, was sie einbindet.
struct Run { /// letzte erfolgreiche Ausgabe (bleibt bei Fehlern für Hover/Definition stehen)
    out: Option<Output>, /// Diagnosen je Datei (normalisierter Pfad), jede Datei kommt vor, auch ohne Meldungen
    diags: BTreeMap<String, Vec<Value>>, target: String, /// Lauf war fehlerfrei: Adresszeilen sind frisch
    fresh: bool }

pub struct Server {
    data_dir: Option<PathBuf>,
    root: Option<PathBuf>,
    default_target: String,
    docs: HashMap<String, Doc>,
    /// Läufe je Wurzeldatei (Schlüssel: normalisierter Pfad); es gibt genau die Läufe, die ein offenes Dokument braucht
    runs: HashMap<String, Run>,
    project: Option<Project>,
    disk: DiskCache,
    /// Dateien (Pfad), für die zuletzt Diagnosen gesendet wurden
    sent: HashSet<String>,
    ctxs: HashMap<String, Ctx>,
    /// Sprache der Texte dieser Serverinstanz (Anfrage `initialize` → `locale`, sonst `VC_LANG`/Systemsprache); nie global gesetzt
    lang: Lang,
    pub shutdown: bool,
    pub exit: bool,
}

/// Sprache aus der `locale` des Editors (`de`, `en-US`, `de_DE.UTF-8`); leer oder `C` → `None`; Unbekanntes → Englisch
fn lang_of_locale(s: &str) -> Option<Lang> {
    vc_i18n::parse_locale(s).map(|c| match c.as_str() { "de" => Lang::De, "xx" => Lang::Xx, _ => Lang::En })
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

/// Trägt die Datei Merkmale eines eigenständigen Programms (`.org`, `* =`, `.target`, `.basicstub`)?
fn has_program_marks(text: &str) -> bool {
    text.lines().any(|l| {
        let t = l.trim_start();
        if t.starts_with('*') { return t[1..].trim_start().starts_with('='); }
        let (_, p) = line_parts(l);
        p.map_or(false, |p| p.kind == PartKind::Directive && matches!(p.name.as_str(), ".org" | ".target" | ".basicstub"))
    })
}

/// Schlüssel der Meldung oder einer darin verschachtelten Meldung (z. B. `diag-asm-e202-expression` mit `inner`)
fn has_key(key: &str, args: &[(&'static str, vc_core::MsgArg)], want: &str) -> bool {
    key == want || args.iter().any(|(_, a)| matches!(a, vc_core::MsgArg::Msg(m) if has_key(m.key, &m.args, want)))
}

/// Fehler, die in einer allein geprüften Datei nur deshalb auftreten, weil Symbole, Makros oder `.org` in der einbindenden Datei stehen.
/// Erkannt an Code und Schlüssel (`diag-asm-e202-unknown-symbol`, auch verschachtelt), nicht am Wortlaut.
fn is_context_error(d: &Diagnostic) -> bool {
    d.code == "E300" || d.code == "E205" || (d.code == "E202" && has_key(d.key, &d.args, "diag-asm-e202-unknown-symbol"))
}

fn resp(id: Option<Value>, result: Value) -> Value { json!({ "jsonrpc": "2.0", "id": id, "result": result }) }
pub(crate) fn range(line: usize, s: usize, e: usize) -> Value { json!({ "start": { "line": line, "character": s }, "end": { "line": line, "character": e } }) }

impl Server {
    pub fn new() -> Self {
        Server { data_dir: None, root: None, default_target: "c64".into(), docs: HashMap::new(), runs: HashMap::new(), project: None, disk: DiskCache::default(), sent: HashSet::new(), ctxs: HashMap::new(), lang: vc_i18n::resolve(Choice::Auto), shutdown: false, exit: false }
    }

    /// Sprache der Serverinstanz festlegen (z. B. wenn der Server in der Oberfläche läuft und nicht über `initialize` angesprochen wird)
    pub fn set_language(&mut self, lang: Lang) { self.lang = lang; }
    pub fn language(&self) -> Lang { self.lang }

    fn context(&mut self, target: &str) -> Result<&Ctx, Msg> {
        if !self.ctxs.contains_key(target) {
            let data = self.data_dir.clone().ok_or_else(|| msg!("diag-lsp-e000-no-data"))?;
            let load = |e: String| msg!("diag-lsp-e000-load", detail = e);
            let t = load_target(&data, target).map_err(load)?;
            let cpu = load_cpu(&data, &t.cpu).map_err(load)?;
            let chips = t.chips.iter().filter_map(|c| load_chip(&data, c).ok()).collect();
            let predefined = vc_target::chip_symbols(&data, &t).map_err(load)?;
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
                if let Some(l) = p["locale"].as_str().and_then(lang_of_locale) { self.lang = l; }
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
                        "completionProvider": { "triggerCharacters": [".", "$"] }, "codeActionProvider": { "codeActionKinds": ["quickfix"] }
                    },
                    "serverInfo": { "name": "vc-lsp", "version": env!("CARGO_PKG_VERSION") }
                })));
            }
            "initialized" | "$/cancelRequest" | "workspace/didChangeConfiguration" => {}
            "shutdown" => { self.shutdown = true; out.push(resp(id, Value::Null)); }
            "exit" => self.exit = true,
            "textDocument/didOpen" => {
                let text = p["textDocument"]["text"].as_str().unwrap_or("").to_string();
                let path = norm_path(&uri_to_path(&uri));
                self.docs.insert(uri.clone(), Doc { text, root: path.clone(), path, includes: vec![] });
                self.project = None;   // neues Dokument: Include-Beziehungen neu bestimmen
                out.extend(self.analyze(&uri));
            }
            "textDocument/didChange" => {
                if let (Some(c), Some(d)) = (p["contentChanges"].as_array().and_then(|a| a.last()), self.docs.get_mut(&uri)) {
                    d.text = c["text"].as_str().unwrap_or("").to_string();
                }
                out.extend(self.analyze(&uri));
            }
            "textDocument/didSave" => self.project = None,   // die Platte hat sich geändert: beim nächsten Lauf neu einlesen
            "textDocument/didClose" => {
                if self.docs.remove(&uri).is_some() {
                    self.project = None;
                    out.extend(self.close_cleanup());
                }
            }
            "textDocument/hover" => out.push(resp(id, self.hover(&uri, line, ch))),
            "textDocument/completion" => out.push(resp(id, self.completion(&uri))),
            "textDocument/definition" => out.push(resp(id, self.definition(&uri, line, ch))),
            "textDocument/documentSymbol" => out.push(resp(id, self.document_symbols(&uri))),
            "textDocument/codeAction" => out.push(resp(id, self.code_actions(&uri, &p["range"]))),
            _ => if id.is_some() { out.push(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": msg!("lsp-error-method-unsupported", method = method.as_str()).text_in(self.lang) } })); }
        }
        out
    }

    fn include_dirs(&self) -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = self.root.iter().cloned().collect();
        if let Some(d) = &self.data_dir { v.push(d.join("sys")); }
        v
    }

    /// Texte aller offenen Dokumente (Pfad → Text): sie ersetzen beim Einbinden den Inhalt der Platte.
    fn open_texts(&self) -> HashMap<String, String> { self.docs.values().map(|d| (d.path.clone(), d.text.clone())).collect() }

    /// Include-Graph bei Bedarf (neu) aufbauen und die Wurzeln der offenen Dokumente bestimmen; `true`, wenn neu gebaut wurde.
    fn ensure_project(&mut self) -> bool {
        if self.project.is_some() { return false; }
        let dirs = self.include_dirs();
        let texts = self.open_texts();
        let p = Project::build(self.root.as_deref(), &dirs, &texts, &mut self.disk);
        for d in self.docs.values_mut() { d.root = p.root_of(&d.path); }
        self.project = Some(p);
        true
    }

    /// Ein Assemblerlauf über `root` (Text des offenen Dokuments, sonst von der Platte) mit dem ungespeicherten Text aller offenen Dokumente.
    fn run_root(&mut self, root: &str, hint_text: &str) -> Run {
        let texts = self.open_texts();
        let root_text = texts.get(root).cloned().or_else(|| vc_core::textfile::read_source(root).ok()).unwrap_or_default();
        let proj_target = self.project.as_ref().and_then(|p| p.targets.get(root).cloned());
        let tid = detect_target(&root_text).or_else(|| detect_target(hint_text)).or(proj_target).unwrap_or_else(|| self.default_target.clone());
        let (is_main, in_sources, included) = self.project.as_ref().map_or((false, false, false), |p| (p.main.as_deref() == Some(root), p.sources.contains(root), p.is_included(root)));
        // Waise ohne Programmmerkmale (weder .org noch .target noch .basicstub), nicht im Projekt als Quelle geführt: vermutlich eine Include-Datei, die nur im Kontext übersetzbar ist
        let lib_like = !is_main && !in_sources && !included && !has_program_marks(&root_text);
        let dirs = self.include_dirs();
        let defines_root = self.root.clone();
        let media_ctx = self.root.clone().zip(self.data_dir.clone());
        let (diags, output): (Vec<Diagnostic>, Option<Output>) = match self.context(&tid) {
            Err(e) => (vec![Diagnostic::error("E000", root, 1, e)], None),
            Ok(ctx) => {
                let mut o = Options::new(root.to_string(), Policy::Warn);
                if let Some(r) = &defines_root { o.defines = vc_target::project_defines(r, &tid); }
                o.include_dirs = dirs;
                // Medien als Include: wie beim Bau umwandeln (sonst läse der Assembler die Mediendatei als Text)
                if let Some((r, d)) = &media_ctx {
                    let hooks = vc_media::include::asm_hooks_with(r, d, &tid, vc_media::include::CacheMode::Memory);
                    (o.include_provider, o.legacy_hint) = (Some(hooks.provider), Some(hooks.legacy));
                }
                o.predefined = ctx.predefined.clone();
                o.atari_text = ctx.target.text_encoding == "atari";
                o.basic_start = ctx.target.basic_load_addr;
                o.overrides = texts.iter().filter(|(k, _)| k.as_str() != root).map(|(k, v)| (k.clone(), v.clone())).collect();
                match assemble(&root_text, &ctx.cpu, &o) { Ok(out) => (out.warnings.clone(), Some(out)), Err(d) => (d, None) }
            }
        };
        let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        by_file.entry(root.to_string()).or_default();
        for d in self.docs.values().filter(|d| d.root == root) { by_file.entry(d.path.clone()).or_default(); }
        let mut seen_org = false;
        for d in &diags {
            let file = if d.file.is_empty() { root.to_string() } else { norm_path(&d.file) };
            // Folgefehler einer Datei, die allein steht, aber nur im Kontext einer einbindenden Datei übersetzbar ist: abstufen bzw. unterdrücken
            let mut severity = if d.is_error() { 1 } else { 2 };
            let mut message = d.text_in(self.lang);
            if lib_like && file == root && d.is_error() && is_context_error(d) {
                if d.code == "E300" { if seen_org { continue; } seen_org = true; }
                severity = 3;
                message = msg!("lsp-diag-context-hint", message = message).text_in(self.lang);
            }
            let l = d.line.saturating_sub(1);
            let src = texts.get(&file).cloned().or_else(|| vc_core::textfile::read_source(&file).ok());
            let line_text: Vec<char> = src.as_deref().and_then(|s| s.lines().nth(l)).unwrap_or("").chars().collect();
            let start = line_text.iter().position(|c| !c.is_whitespace()).unwrap_or(0);
            by_file.entry(file).or_default().push(json!({
                "range": if d.range.col > 0 { range(d.range.line.saturating_sub(1), d.range.col - 1, if d.range.end_col > d.range.col { d.range.end_col - 1 } else { line_text.len() }) } else { range(l, start, line_text.len()) }, "severity": severity,
                "code": d.code, "source": "vcc", "message": message, "data": d.msg().to_json()
            }));
        }
        let fresh = output.is_some();
        let old = self.runs.remove(root).and_then(|r| r.out);
        Run { out: output.or(old), diags: by_file, target: tid, fresh }
    }

    fn analyze(&mut self, uri: &str) -> Vec<Value> {
        let Some(doc) = self.docs.get_mut(uri) else { return vec![] };
        // Ändert sich die Liste der Includes dieser Datei, gelten die Wurzeln neu
        let names = vc_asm::include_names(&doc.text);
        if names != doc.includes { doc.includes = names; self.project = None; }
        let (path, text) = (doc.path.clone(), doc.text.clone());
        self.ensure_project();
        let root = self.project.as_ref().map(|p| p.root_of(&path)).unwrap_or_else(|| path.clone());
        if let Some(d) = self.docs.get_mut(uri) { d.root = root.clone(); }
        let needed: BTreeSet<String> = self.docs.values().map(|d| d.root.clone()).collect();
        self.runs.retain(|r, _| needed.contains(r));
        // neu zu rechnen: die Wurzel dieses Dokuments, jede Wurzel, die dieses Dokument einbindet, und Wurzeln ohne Lauf
        let mut touch: BTreeSet<String> = BTreeSet::new();
        touch.insert(root.clone());
        for r in &needed {
            if !self.runs.contains_key(r) || self.project.as_ref().map_or(false, |p| p.descendants(r).contains(&path)) { touch.insert(r.clone()); }
        }
        for r in &touch { let run = self.run_root(r, &text); self.runs.insert(r.clone(), run); }
        let mut msgs = self.publish(Some(&path), &touch);
        // Zeilenadressen für die Anzeige im Editor (nur Zeilen der jeweiligen Datei, die Bytes erzeugen); bei Fehlern bleibt der letzte Stand stehen
        let mut docs: Vec<(&String, &Doc)> = self.docs.iter().filter(|(_, d)| touch.contains(&d.root)).collect();
        docs.sort_by_key(|(u, _)| (u.as_str() != uri, (*u).clone()));
        for (u, d) in docs {
            let Some(run) = self.runs.get(&d.root).filter(|r| r.fresh) else { continue };
            let Some(out) = &run.out else { continue };
            let lines: Vec<Value> = out.lines.iter().filter(|l| !l.bytes.is_empty() && norm_path(&l.file) == d.path).map(|l| json!([l.line.saturating_sub(1), l.addr, l.bytes.len()])).collect();
            let source: Vec<&str> = d.text.split('\n').collect();
            msgs.push(json!({ "jsonrpc": "2.0", "method": "vintagecoder/addresses", "params": { "uri": u, "lines": lines, "source": source } }));
        }
        msgs
    }

    /// `publishDiagnostics` je Datei: die Vereinigung der Meldungen aller Läufe; Dateien, die kein Lauf mehr betrifft, werden geleert.
    /// Gesendet wird für `own` (zuerst) und alle Dateien der Läufe in `touched`.
    fn publish(&mut self, own: Option<&str>, touched: &BTreeSet<String>) -> Vec<Value> {
        let mut all: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for run in self.runs.values() {
            for (f, items) in &run.diags {
                let e = all.entry(f.clone()).or_default();
                for i in items { if !e.contains(i) { e.push(i.clone()); } }
            }
        }
        let mut send: Vec<String> = vec![];
        if let Some(o) = own { send.push(o.to_string()); }
        for r in touched { if let Some(run) = self.runs.get(r) { for f in run.diags.keys() { if !send.contains(f) { send.push(f.clone()); } } } }
        let mut msgs = vec![];
        for f in send {
            let items = all.get(&f).cloned().unwrap_or_default();
            msgs.push(json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": path_to_uri(&f), "diagnostics": items } }));
        }
        for old in self.sent.difference(&all.keys().cloned().collect()) {
            msgs.push(json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": path_to_uri(old), "diagnostics": [] } }));
        }
        self.sent = all.into_keys().collect();
        msgs
    }

    /// Nach dem Schließen eines Dokuments: nicht mehr gebrauchte Läufe entfernen, Diagnosen verwaister Dateien leeren.
    fn close_cleanup(&mut self) -> Vec<Value> {
        self.ensure_project();
        let needed: BTreeSet<String> = self.docs.values().map(|d| d.root.clone()).collect();
        self.runs.retain(|r, _| needed.contains(r));
        self.publish(None, &BTreeSet::new())
    }

    fn run_of(&self, doc: &Doc) -> Option<&Run> { self.runs.get(&doc.root) }

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
        let ctx = self.run_of(doc).and_then(|r| self.ctxs.get(&r.target));
        let lang = self.lang;
        // Zahlenliteral ($d020, %1010, 255, 'A'): Wert in mehreren Schreibweisen, bei Adressen auch das passende Register
        if let Some((ls, le, v)) = crate::literal_at(l, ch) {
            let text: String = chars[ls..le].iter().collect();
            let mut md = crate::value_hover_in(lang, &text, v);
            if let Some(ctx) = ctx { if let Some((chip, r)) = ctx.chips.iter().find_map(|chip| chip.regs.iter().find(|r| r.addr as i64 == v).map(|r| (chip, r))) {
                md.push_str(&format!("\n{}", msg!("lsp-hover-register-short", chip = chip.id.as_str(), reg = r.name.as_str(), doc = r.doc.as_str()).text_in(lang)));
            } }
            return json!({ "contents": { "kind": "markdown", "value": md }, "range": range(line, ls, le) });
        }
        let Some((s, e)) = word_at(&chars, ch) else { return Value::Null };
        let word: String = chars[s..e].iter().collect();
        let (_, part) = line_parts(l);
        let mut md = String::new(); let mut mode_json: Option<Value> = None;
        if let Some(p) = &part {
            if p.kind == PartKind::Instruction && p.name.eq_ignore_ascii_case(&word) {
                if let Some(ctx) = ctx {
                    let mut ops: Vec<_> = ctx.cpu.ops.iter().filter(|o| o.mnemonic == p.name).collect();
                    ops.sort_by_key(|o| o.opcode);
                    if !ops.is_empty() {
                        md.push_str(&format!("**{}** – {}\n\n", p.name, mnemonic_doc_in(lang, &p.name).unwrap_or_default()));
                        if !ops[0].affects.is_empty() { md.push_str(&format!("{}\n\n", msg!("lsp-hover-flags", flags = ops[0].affects.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")).text_in(lang))); }
                        // Welche Adressierungsart der Assembler für diese Zeile gewählt hat (aus dem letzten fehlerfreien Lauf); nie raten
                        let chosen = self.line_mode(doc, line, &p.name, &ctx.cpu);
                        md.push_str(&msg!("lsp-hover-table-head").text_in(lang)); md.push('\n');
                        for o in ops {
                            let cy = format!("{}{}{}", o.cycles, if o.page_cross { "+1" } else { "" }, if o.branch { "+1/+2" } else { "" });
                            let tag = if o.group == "official" { String::new() } else if o.group == "illegal.unstable" { format!(" {}", msg!("lsp-hover-tag-unstable").text_in(lang)) } else { format!(" {}", msg!("lsp-hover-tag-undocumented").text_in(lang)) };
                            let hit = matches!(&chosen, LineMode::Chosen { opcode, .. } if *opcode == o.opcode);
                            md.push_str(&format!("| {}{} | ${:02X} | {} | {}{} |\n", if hit { "▶ " } else { "" }, o.mode.label(), o.opcode, o.bytes, cy, tag));
                            if hit { mode_json = Some(json!({ "mode": o.mode.kuerzel(), "name": o.mode.name(), "state": "chosen", "chosenBy": "assembler", "opcode": o.opcode, "bytes": o.bytes, "cycles": o.cycles, "pageCross": o.page_cross, "branch": o.branch })); }
                        }
                        match &chosen {
                            LineMode::Chosen { mode, bytes, cycles, .. } => md.push_str(&format!("\n{}\n", msg!("lsp-hover-line-chosen", mode = mode.label(), bytes = bytes.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" "), cycles = *cycles).text_in(lang))),
                            LineMode::Open(why) => { let why_text = why.text_in(lang); md.push_str(&format!("\n{}\n", msg!("lsp-hover-line-open", why = why_text.as_str()).text_in(lang))); mode_json = Some(json!({ "state": "open", "reason": why_text, "reasonKey": why.key })); }
                            LineMode::Ambiguous => { md.push_str(&format!("\n{}\n", msg!("lsp-hover-line-ambiguous").text_in(lang))); mode_json = Some(json!({ "state": "ambiguous" })); }
                        }
                    }
                }
            } else if p.kind == PartKind::Directive && vc_asm::directives::normalize(&word.to_ascii_lowercase(), "").0 == p.name {
                // ein Alias zeigt dieselbe Seite wie der kanonische Name, darunter „Auch: …“
                if let Some(d) = directive_doc_in(lang, &p.name) {
                    md = format!("**{}**\n\n{}", p.name, d);
                    if let Some(e) = vc_asm::directives::entry(&p.name) {
                        let names: Vec<String> = if e.name == p.name { e.aliases.iter().filter(|a| a.starts_with('.')).cloned().collect() } else { e.end_aliases.iter().filter(|a| a.starts_with('.')).cloned().collect() };
                        if !names.is_empty() { md.push_str(&format!("\n\n{}", msg!("lsp-hover-also", names = names.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")).text_in(lang))); }
                    }
                }
            }
        }
        if md.is_empty() {
            if let Some(ctx) = ctx {
                for chip in &ctx.chips {
                    if let Some(r) = chip.regs.iter().find(|r| word == format!("{}.{}", chip.id, r.name)) {
                        let acc = match r.access.as_str() { "r" => msg!("lsp-hover-access-read"), "w" => msg!("lsp-hover-access-write"), _ => msg!("lsp-hover-access-rw") }.text_in(lang);
                        md = msg!("lsp-hover-register", chip = chip.id.as_str(), reg = r.name.as_str(), addr = format!("${:04X}", r.addr), access = acc, doc = r.doc.as_str(), name = chip.name.as_str()).text_in(lang);
                    }
                }
            }
        }
        if md.is_empty() {
            if let Some(out) = self.run_of(doc).and_then(|r| r.out.as_ref()) {
                let key = if word.starts_with('.') { format!("{}{}", Self::scope_at(&doc.text, line), word) } else { word.clone() };
                if let Some(v) = out.symbols.get(&key) {
                    md = crate::value_hover_in(lang, &word, *v);
                    if let Some((f, ln)) = out.defs.get(&key) {
                        md.push_str(&format!("\n\n{}", msg!("lsp-hover-defined-in", file = Path::new(f).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), line = ln.to_string()).text_in(lang)));
                    }
                }
            }
        }
        if md.is_empty() { return Value::Null; }
        let mut r = json!({ "contents": { "kind": "markdown", "value": md }, "range": range(line, s, e) });
        if let Some(m) = mode_json { r["vintagecoder"] = json!({ "lineMode": m }); }
        r
    }

    /// Adressierungsart der Quellzeile `line` (0-basiert) laut letztem fehlerfreien Assemblerlauf
    fn line_mode(&self, doc: &Doc, line: usize, mnemonic: &str, cpu: &vc_target::Cpu) -> LineMode {
        let Some(run) = self.runs.get(&doc.root) else { return LineMode::Open(msg!("lsp-hover-why-no-run")) };
        if !run.fresh { return LineMode::Open(msg!("lsp-hover-why-errors")) }
        let Some(out) = &run.out else { return LineMode::Open(msg!("lsp-hover-why-no-run")) };
        let hits: Vec<&vc_asm::LineInfo> = out.lines.iter().filter(|l| l.line == line + 1 && !l.bytes.is_empty() && norm_path(&l.file) == doc.path).collect();
        if hits.is_empty() { return LineMode::Open(msg!("lsp-hover-why-no-code")) }
        let mut found: Vec<(vc_target::Mode, u8, &[u8], u8)> = vec![];
        for h in hits {
            // 45GS02: Präfixe $42 $42 (Q) und $EA (32-Bit-Zeiger) stehen vor dem eigentlichen Opcode
            let skip = if h.bytes.len() > 2 && h.bytes[0] == 0x42 && h.bytes[1] == 0x42 { 2 } else if h.bytes.len() > 1 && h.bytes[0] == 0xEA { 1 } else { 0 };
            if let Some(o) = cpu.by_opcode(h.bytes[skip]).filter(|o| o.mnemonic == mnemonic) { found.push((o.mode, o.opcode, &h.bytes[..], o.cycles)) }
        }
        found.dedup_by_key(|f| (f.1, f.2.to_vec()));
        match found.as_slice() {
            [] => LineMode::Open(msg!("lsp-hover-why-ambiguous")),
            [(mode, opcode, bytes, cycles)] => LineMode::Chosen { mode: *mode, opcode: *opcode, bytes: bytes.to_vec(), cycles: *cycles },
            _ => LineMode::Ambiguous,
        }
    }


    /// Quick-Fixes für den Bereich `range`: alte Direktivennamen durch den kanonischen ersetzen (je Zeile und für die ganze Datei)
    fn code_actions(&self, uri: &str, range: &Value) -> Value {
        let Some(doc) = self.docs.get(uri) else { return json!([]) };
        let (l0, l1) = (range["start"]["line"].as_u64().unwrap_or(0) as usize, range["end"]["line"].as_u64().unwrap_or(0) as usize);
        let edit = |f: &crate::quickfix::AliasFix| json!({ "range": crate::server::range(f.line, f.start, f.end), "newText": f.new_text });
        let here = crate::quickfix::alias_fixes(&doc.text, Some(l0..=l1.max(l0)));
        let mut actions = vec![];
        for f in &here {
            actions.push(json!({ "title": msg!("lsp-codeaction-canonical", old = f.old.as_str(), new = f.new_text.as_str()).text_in(self.lang), "kind": "quickfix",
                                 "edit": { "changes": { uri: [edit(f)] } } }));
        }
        let all = crate::quickfix::alias_fixes(&doc.text, None);
        if !all.is_empty() && (all.len() > here.len() || here.len() > 1) {
            actions.push(json!({ "title": msg!("lsp-codeaction-canonical-all", count = all.len()).text_in(self.lang), "kind": "quickfix",
                                 "edit": { "changes": { uri: all.iter().map(edit).collect::<Vec<_>>() } } }));
        }
        Value::Array(actions)
    }

    fn completion(&self, uri: &str) -> Value {
        let Some(doc) = self.docs.get(uri) else { return json!({ "isIncomplete": false, "items": [] }) };
        let mut items = vec![];
        if let Some(ctx) = self.run_of(doc).and_then(|r| self.ctxs.get(&r.target)) {
            let mut seen = HashSet::new();
            for o in &ctx.cpu.ops {
                if seen.insert(o.mnemonic.clone()) {
                    items.push(json!({ "label": o.mnemonic.to_lowercase(), "kind": 14, "detail": mnemonic_doc_in(self.lang, &o.mnemonic).unwrap_or_default(), "sortText": format!("2{}", o.mnemonic) }));
                }
            }
            for chip in &ctx.chips {
                for r in &chip.regs {
                    items.push(json!({ "label": format!("{}.{}", chip.id, r.name), "kind": 5, "detail": format!("${:04X} · {}", r.addr, r.access), "documentation": r.doc, "sortText": format!("3{}", r.name) }));
                }
            }
        }
        // kanonische Namen zuerst; alte Namen erscheinen danach (der Editor zeigt sie nur, wenn man sie schon tippt) mit Hinweis auf den kanonischen
        for d in directive_names() { items.push(json!({ "label": d, "kind": 14, "detail": msg!("lsp-completion-directive").text_in(self.lang), "documentation": directive_doc_in(self.lang, d).unwrap_or_default(), "sortText": format!("1{d}") })); }
        for (a, c) in vc_asm::directives::aliases() {
            items.push(json!({ "label": a, "kind": 14, "detail": msg!("lsp-completion-alias", name = c).text_in(self.lang), "documentation": directive_doc_in(self.lang, c).unwrap_or_default(), "sortText": format!("9{a}") }));
        }
        let mut names = HashSet::new();
        for it in outline(&doc.text) {
            if names.insert(it.name.clone()) {
                let kind = match it.kind { OutlineKind::Label => 6, OutlineKind::Const => 21, OutlineKind::Macro => 3 };
                items.push(json!({ "label": it.name, "kind": kind, "detail": msg!("lsp-completion-line", line = it.line.to_string()).text_in(self.lang), "sortText": format!("0{}", it.name) }));
            }
        }
        if let Some(out) = self.run_of(doc).and_then(|r| r.out.as_ref()) {
            for (n, params) in &out.macros {
                let n = &n.to_lowercase();
                if names.insert(n.clone()) { items.push(json!({ "label": n, "kind": 3, "detail": msg!("lsp-completion-macro", params = params.join(", ")).text_in(self.lang), "sortText": format!("0{n}") })); }
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
        if let Some((f, ln)) = self.run_of(doc).and_then(|r| r.out.as_ref()).and_then(|o| o.defs.get(&key)) {
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
