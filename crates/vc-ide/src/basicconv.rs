// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC im Editor umwandeln: „In Labels umwandeln“, „In Nummern umwandeln“, „Neu nummerieren“ (Konzept „BASIC-Editor und Dialekte“, B1–B3).
//! Das Fachliche steht in `vc_basic::labels`; hier werden Dialekt, Nummerierung (Projekt/Datei) und Einbindungen für die IDE bestimmt und das Ergebnis
//! als JSON für die Oberfläche aufbereitet. Der Text der Quelldatei kommt aus dem Editor (auch ungespeichert); eingebundene Dateien liest `load_set` von der Platte.
use serde_json::{json, Value};
use std::path::Path;
use vc_basic::labels::{self, Conversion, Mode};
use vc_core::Diagnostic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Labels, Numbers, Renumber, Mode }
impl Kind {
    pub fn parse(s: &str) -> Option<Kind> { Some(match s { "labels" => Kind::Labels, "numbers" => Kind::Numbers, "renumber" => Kind::Renumber, "mode" => Kind::Mode, _ => return None }) }
}

pub struct Req<'a> {
    pub root: &'a Path,
    pub data: &'a Path,
    /// Quelldatei relativ zum Projekt
    pub rel: &'a str,
    /// aktueller Text der Quelldatei (Editor)
    pub text: &'a str,
    pub kind: Kind,
    /// nur Zeilen mit Nummern in diesem Bereich (nur „Neu nummerieren“)
    pub range: Option<(u32, u32)>,
    /// eingebundene Dateien mitumwandeln (sonst nur die Datei selbst)
    pub with_includes: bool,
}

fn diag_json(root: &Path, d: &Diagnostic) -> Value {
    let rel = match Path::new(&d.file).strip_prefix(root) { Ok(p) => vc_core::paths::slashes(&p.to_string_lossy()), Err(_) => d.file.clone() };
    json!({ "file": rel, "line": d.line, "severity": if d.is_error() { "error" } else { "warning" }, "code": d.code, "message": d.message, "key": d.key,
            "args": d.args.iter().map(|(k, v)| (k.to_string(), match v { vc_core::MsgArg::Num(n) => json!(n), vc_core::MsgArg::Text(t) => json!(t), vc_core::MsgArg::Msg(m) => json!(m.to_string()) })).collect::<serde_json::Map<_, _>>() })
}
fn mode_name(m: Mode) -> &'static str { match m { Mode::Numbers => "numbers", Mode::Labels => "labels", Mode::Mixed => "mixed", Mode::Undetermined => "undetermined" } }
fn rel_of(root: &Path, f: &str) -> String { match Path::new(f).strip_prefix(root) { Ok(p) => vc_core::paths::slashes(&p.to_string_lossy()), Err(_) => f.to_string() } }

/// Dialekt und Nummerierung der Datei: Projekt (Quellgruppe, Target), `#dialect`/`#numbering` der Datei
struct Setup { target: String, dialect: vc_basic::dialects::AnyDialect, info: vc_basic::dialects::DialectInfo, start: u32, step: u32, notes: Vec<Diagnostic> }
fn setup(r: &Req, abs: &str) -> Result<Setup, Vec<Diagnostic>> {
    let pf = crate::projfile::find(r.root).and_then(|f| crate::projfile::ProjectFile::load(&f).ok());
    let tid = pf.as_ref().and_then(|p| p.target_of(r.rel)).or_else(|| crate::build::detect_target(r.text)).unwrap_or_else(|| vc_target::default_target_or_c64(Some(r.data)));
    let tgt_dialect = vc_target::load_target(r.data, &tid).ok().and_then(|t| t.basic_dialect);
    let did = pf.as_ref().and_then(|p| p.dialect_of(r.rel)).or(tgt_dialect).unwrap_or_else(|| "cbm_v2".into());
    let res = vc_basic::dialects::resolve(r.text, abs, &did, Some(&tid), r.data)?;
    let (dialect, info) = match (vc_basic::dialects::AnyDialect::load(r.data, &res.id), vc_basic::dialects::info(r.data, &res.id)) {
        (Ok(d), Ok(i)) => (d, i),
        (Err(e), _) | (_, Err(e)) => return Err(vec![Diagnostic::error("B178", abs, 1, vc_core::Msg::raw(e))]),
    };
    let proj = crate::projfile::basic_settings(r.root).numbering;
    let n = vc_basic::numbering::resolve(r.text, abs, proj, info.max_line_number)?;
    Ok(Setup { target: tid, dialect, info, start: n.start, step: n.step, notes: res.warnings })
}

/// Umwandlung (oder nur Modus) für die Vorschau; schreibt nichts. Ergebnis (`ok`): `mode`, `numbering`, `changes`, `newLabels`, `kept`, `warnings`, `files`, `text` (neuer Text der Quelldatei).
fn compute(r: &Req) -> Value {
    let abs = r.root.join(r.rel).to_string_lossy().into_owned();
    let fail = |ds: &[Diagnostic]| json!({ "ok": false, "diagnostics": ds.iter().map(|d| diag_json(r.root, d)).collect::<Vec<_>>() });
    let s = match setup(r, &abs) { Ok(s) => s, Err(ds) => return fail(&ds) };
    let hooks = crate::basicmedia::hooks(r.root, r.data, &s.target);
    let (mut units, media) = match vc_basic::include::load_set_media(&abs, r.text, &hooks) { Ok(u) => u, Err(ds) => return fail(&ds) };
    // Medien-Einheiten sind schreibgeschützt: sie bleiben in der Menge (für die Sprungziele), werden aber nie zurückgeschrieben
    let n_inc = units.iter().skip(1).filter(|u| !media.contains(&u.file)).count();
    if !r.with_includes { units.truncate(1); }
    let cx = labels::Cx::new(&s.dialect, &s.info).numbering(s.start, s.step);
    let mode = labels::detect_mode(&units, &cx);
    let base = json!({ "mode": mode_name(mode), "numbering": { "start": s.start, "step": s.step }, "includes": n_inc, "maxLine": s.info.max_line_number });
    let mut out = base.clone();
    if r.kind == Kind::Mode {
        let ws = labels::check_modes(&units, &cx);
        out["ok"] = json!(true);
        out["warnings"] = json!(s.notes.iter().chain(ws.iter()).map(|d| diag_json(r.root, d)).collect::<Vec<_>>());
        return out;
    }
    let res: Result<Conversion, Vec<Diagnostic>> = match r.kind { Kind::Labels => labels::to_labels(&units, &cx), Kind::Numbers => labels::to_numbers(&units, &cx, None), _ => labels::renumber(&units, &cx, r.range), Kind::Mode => unreachable!() };
    match res {
        Err(ds) => { let mut v = fail(&ds); v["mode"] = base["mode"].clone(); v }
        Ok(c) => {
            out["ok"] = json!(true);
            out["changes"] = json!(c.changes.iter().map(|ch| json!({ "file": rel_of(r.root, &ch.file), "line": ch.line, "old": ch.old, "new": ch.new })).collect::<Vec<_>>());
            out["newLabels"] = json!(c.new_labels.iter().map(|l| json!({ "name": l.name, "file": rel_of(r.root, &l.file), "line": l.line })).collect::<Vec<_>>());
            out["kept"] = json!(c.kept_labels.len());
            // Modus-Warnungen (B160 gemischt, B165/B166/B168) gehören zur Anzeige des Programms, auch wenn die Umwandlung selbst keine meldet
            let mut all: Vec<Diagnostic> = s.notes.iter().cloned().chain(c.warnings.iter().cloned()).collect();
            for w in labels::check_modes(&units, &cx) { if !all.iter().any(|x| x.code == w.code && x.file == w.file && x.line == w.line) { all.push(w); } }
            out["warnings"] = json!(all.iter().map(|d| diag_json(r.root, d)).collect::<Vec<_>>());
            out["media"] = json!(media.len());
            out["files"] = json!(c.units.iter().zip(&units).filter(|(n, _)| !media.contains(&n.file)).map(|(n, o)| json!({ "file": rel_of(r.root, &n.file), "changed": n.text != o.text })).collect::<Vec<_>>());
            out["text"] = json!(c.units[0].text);
            // Texte der übrigen Dateien nur für `apply`
            out["_others"] = json!(c.units.iter().zip(&units).skip(1).filter(|(n, o)| n.text != o.text && !media.contains(&n.file)).map(|(n, _)| json!({ "file": rel_of(r.root, &n.file), "abs": n.file, "text": n.text })).collect::<Vec<_>>());
            out
        }
    }
}

/// Vorschau (schreibt nichts)
pub fn run(r: &Req) -> Value { let mut v = compute(r); v.as_object_mut().map(|m| m.remove("_others")); v }

/// Anwenden: wie [`run`]; die geänderten eingebundenen Dateien werden geschrieben (`written`), der neue Text der Quelldatei wird zurückgegeben (der Editor setzt ihn
/// als eine Änderung ein, damit „Rückgängig“ genügt); die Quelldatei selbst schreibt der Aufrufer nicht.
pub fn apply(r: &Req) -> Value {
    let mut v = compute(r);
    if v["ok"] != true || r.kind == Kind::Mode { return v; }
    let others = v["_others"].as_array().cloned().unwrap_or_default();
    let mut written = vec![]; let mut errors = vec![];
    for o in &others {
        let (rel, abs, text) = (o["file"].as_str().unwrap_or(""), o["abs"].as_str().unwrap_or(""), o["text"].as_str().unwrap_or(""));
        match vc_core::atomar::schreiben(abs, text) { Ok(()) => written.push(rel.to_string()), Err(e) => errors.push(format!("{rel}: {e}")) }
    }
    v["written"] = json!(written);
    if !errors.is_empty() { v["ok"] = json!(false); v["error"] = json!(errors.join("; ")); }
    v.as_object_mut().map(|m| m.remove("_others"));
    v
}
