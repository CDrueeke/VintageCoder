// SPDX-License-Identifier: GPL-3.0-or-later
//! Bauen einer Quelldatei (Assembler oder strukturiertes BASIC) für die IDE.
/// Das plattformübergreifende strukturierte BASIC (`.sbas`) ist vorerst abgeschaltet; der Übersetzer in `vc-basic` bleibt erhalten.
pub const SBAS_ENABLED: bool = false;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_asm::{assemble, line_parts, listing, to_bin, to_car, to_crt, to_prg, to_xex, vcdbg, vice_labels, Options, PartKind, Policy};
use vc_core::Diagnostic;
use crate::buildlog::{BuildLog, Status};

pub struct Ctx<'a> { pub root: &'a Path, pub data: &'a Path }

fn diag_json(root: &Path, d: &Diagnostic) -> Value {
    let rel = rel_of(root, &d.file);
    json!({ "file": rel, "line": d.line, "severity": if d.is_error() { "error" } else { "warning" }, "code": d.code, "message": d.message })
}

pub fn detect_target(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (_, p) = line_parts(l);
        p.filter(|p| p.kind == PartKind::Directive && p.name == ".target").and_then(|p| p.args.split_whitespace().next().map(str::to_string))
    })
}

fn rel_of(root: &Path, f: &str) -> String { match Path::new(f).strip_prefix(root) { Ok(p) => vc_core::paths::slashes(&p.to_string_lossy()), Err(_) => f.to_string() } }
fn diag_line(root: &Path, d: &Diagnostic) -> String { format!("{} {}{}  {}", if d.is_error() { vc_i18n::t!("build-fehler") } else { vc_i18n::t!("build-warnung") }, rel_of(root, &d.file), if d.line > 0 { format!(":{}", d.line) } else { String::new() }, d.message) }

fn fail(root: &Path, file: &str, target: &str, diags: &[Diagnostic], mut log: BuildLog) -> Value {
    log.fail(diags.iter().map(|d| diag_line(root, d)));
    json!({ "ok": false, "file": file, "target": target, "diagnostics": diags.iter().map(|d| diag_json(root, d)).collect::<Vec<_>>(), "log": log.finish() })
}

pub fn build(c: &Ctx, rel: &str, target: Option<&str>) -> Result<Value, String> { build_with(c, rel, target, None) }

/// Schreibt eine Zwischendatei nach `build/` und trägt sie im Protokoll ein; Schreibfehler werden als Hinweis vermerkt
fn side_file(log: &mut BuildLog, dir: &Path, name: &str, content: &str, role: &str) -> bool {
    match std::fs::write(dir.join(name), content) { Ok(_) => { log.file(&dir.join(name), role); true } Err(e) => { log.line(vc_i18n::t!("build-name-e", name = &name, e = &e)); false } }
}

/// Lange Variablennamen (`#longnames`): Zuordnung ins Bauprotokoll und in `<name>.vars.txt`; liefert die Zeilen für `vars` und die Warnungen (auf die Quelldatei zurückgeführt)
fn long_names_report(log: &mut BuildLog, dir: &Path, stem: &str, names: &[vc_basic::longnames::NameEntry], warnings: &[Diagnostic], abs: &str, map: &vc_basic::LineMap) -> (Vec<String>, Vec<Diagnostic>) {
    let warns = vc_basic::remap_diagnostics(warnings, abs, map);
    if !names.is_empty() {
        log.line(vc_i18n::t!("build-lange-namen-p1-umgesetzt", p1 = names.len()));
        for n in names { log.line(format!("  {}", n.line())); }
        side_file(log, dir, &format!("{stem}.vars.txt"), &vc_basic::longnames::table_text(names), &vc_i18n::t!("build-zwischendatei-lange-variablennamen-kurznamen"));
    }
    for w in &warns { log.line(vc_i18n::t!("build-warnung-p1", p1 = &w.message)); }
    (names.iter().map(|n| format!("{} → {}", n.long, n.short)).collect(), warns)
}

/// Übersetzungsoptionen für natives BASIC aus dem Projekt: Nummerierung (`basic.start/step`), „kompakt“, Syntaxprüfung,
/// Zielsystem (für `#if SYSTEM_…`) und Konstanten des Targets (`defines`, für `#define`-Ersetzung und `#if`).
fn plain_options(c: &Ctx, abs: &str, max_line: u32, tid: &str, media: usize, log: &mut BuildLog) -> vc_basic::PlainOptions {
    let bs = crate::projfile::basic_settings(c.root);
    let mut o = vc_basic::PlainOptions::new(abs, max_line);
    (o.start, o.step) = (bs.numbering.start, bs.numbering.step);
    o.check = bs.check_options();
    o.compact = bs.compact;
    o.system = Some(tid.to_string());
    o.media = media;
    o.defines = vc_target::project_defines(c.root, tid).into_iter().map(|(n, v)| (n, v.to_string())).collect();
    let d = vc_basic::numbering::Numbering::default();
    if bs.numbering != d { log.line(vc_i18n::t!("build-nummerierung-start-p1-schritt-p2-projekt", p1 = bs.numbering.start, p2 = bs.numbering.step)); }
    if bs.compact { log.line(vc_i18n::t!("build-bauoption-kompakt-aus-der-projektdatei")); }
    if !o.defines.is_empty() { log.line(vc_i18n::t!("build-konstanten-aus-der-projektdatei-p1", p1 = o.defines.iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join(", "))); }
    o
}

/// Nennt der Quelltext (samt der Dateien, die er einbindet) noch den Build-Ordner `build/export`? Dann braucht der Bau die alte Export-Stufe (Übergang).
/// Nennt das BASIC-Programm (oder eine eingebundene Datei, verschachtelt bis acht Ebenen) noch einen Pfad unter `build/export`? Nur dann läuft die alte Exportstufe;
/// Medien, die über ihre Datei eingebunden sind, brauchen sie nicht.
fn basic_uses_legacy_export(text: &str, file: &str) -> bool {
    fn go(text: &str, file: &str, depth: usize) -> bool {
        text.lines().filter_map(vc_basic::include_directive).any(|name| {
            if name.replace('\\', "/").contains("build/export/") { return true; }
            if depth >= 8 { return false; }
            let p = Path::new(file).parent().map(Path::to_path_buf).unwrap_or_default().join(name);
            vc_core::textfile::read_source(&p).map_or(false, |t| go(&t, &p.to_string_lossy(), depth + 1))
        })
    }
    go(text, file, 0)
}

fn uses_legacy_export(root: &Path, text: &str, file: &str, dirs: &[PathBuf]) -> bool {
    fn walk(root: &Path, text: &str, file: &str, dirs: &[PathBuf], seen: &mut std::collections::HashSet<PathBuf>, depth: usize) -> bool {
        // Kommentare zählen nicht (`;` und `//`): nur Include- und Incbin-Zeilen brauchen die alte Export-Stufe
        if text.lines().any(|l| { let code = l.split(';').next().unwrap_or(""); let code = code.split("//").next().unwrap_or(""); code.contains("build/export") || code.contains("build\\export") }) { return true; }
        if depth > 16 { return false; }
        for n in vc_asm::include_names(text) {
            let Some(p) = vc_asm::resolve_include(file, &n, dirs) else { continue };
            if vc_media::include::is_media_path(&n) || !seen.insert(p.clone()) { continue; }
            if let Ok(t) = vc_core::textfile::read_source(&p) { if walk(root, &t, &p.to_string_lossy(), dirs, seen, depth + 1) { return true; } }
        }
        false
    }
    walk(root, text, file, dirs, &mut Default::default(), 0)
}

fn list_files(log: &mut BuildLog, root: &Path, files: &[(String, usize)]) {
    log.line(vc_i18n::t!("build-p1-datei-en-p2-zeilen", p1 = files.len(), p2 = files.iter().map(|f| f.1).sum::<usize>()));
    for (i, (f, n)) in files.iter().enumerate() { log.line(format!("  {}", vc_i18n::t!("build-p1-n-zeilen", p1 = rel_of(root, f), n = &n))); log.file(Path::new(f), &if i == 0 { vc_i18n::t!("build-quelle") } else { vc_i18n::t!("build-include") }); }
}

/// Wie [`build`], mit ausdrücklich gewähltem BASIC-Dialekt (z. B. `turbo_basic_xl`, `basic7`); sonst gilt der Standarddialekt des Targets.
pub fn build_with(c: &Ctx, rel: &str, target: Option<&str>, dialect: Option<&str>) -> Result<Value, String> {
    let path: PathBuf = crate::safepath::resolve(c.root, rel)?;
    let text = vc_core::textfile::read_source(&path).map_err(|e| vc_i18n::t!("build-rel-e", rel = &rel, e = &e))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out").to_string();
    let mut log = BuildLog::new(c.root);
    log.stage(&vc_i18n::t!("build-vorbereitung"));
    let tid = target.map(String::from).or_else(|| detect_target(&text)).unwrap_or_else(|| vc_target::default_target_or_c64(Some(c.data)));
    let mut tgt = vc_target::load_target(c.data, &tid)?;
    tgt.apply_memory(crate::projfile::target_setting(c.root, &tid, "memory").as_deref())?;
    let out_dir = c.root.join("build");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let abs = path.to_string_lossy().into_owned();
    log.line(vc_i18n::t!("build-quelle-rel-p2-zeilen-p3-bytes", rel = &rel, p2 = text.lines().count(), p3 = text.len()));
    log.line(vc_i18n::t!("build-zielsystem-tid-p2-cpu-p3", tid = &tid, p2 = if target.is_some() { format!(" {}", vc_i18n::t!("build-gewaehlt")) } else if detect_target(&text).is_some() { format!(" {}", vc_i18n::t!("build-aus-target")) } else { format!(" {}", vc_i18n::t!("build-standard")) }, p3 = &tgt.cpu));
    log.line(vc_i18n::t!("build-ausgabeordner-build"));
    log.file(&path, &vc_i18n::t!("build-quelle"));
    match ext.as_str() {
        "asm" => {
            log.line(vc_i18n::t!("build-include-suche-projektordner-dann-systembibliothek-p1", p1 = c.data.display()));
            // Sprites, Zeichensätze, Songs als Build-Eingabe: veraltete Dateien nach build/export/*.asm exportieren (vor dem Assemblieren, damit .include greift)
            log.stage(&vc_i18n::t!("build-medien-exportieren"));
            // Medien, die der Quelltext über ihre Datei einbindet (`.include "graphics/hero.vcsprite"`), werden beim Einbinden umgewandelt; die alte
            // Stufe nach build/export läuft nur noch, solange ein Include oder .incbin den Build-Ordner nennt (Übergang)
            let legacy_export = uses_legacy_export(c.root, &text, &abs, &[c.root.to_path_buf(), c.data.join("sys")]);
            let (exported, media_errs) = if legacy_export { vc_media::project::export_all(c.root, c.data) } else { (vec![], vec![]) };
            if exported.is_empty() && media_errs.is_empty() { log.line(vc_i18n::t!("build-alle-exportierten-dateien-sind-aktuell-oder")); log.done(Status::Skipped); }
            else {
                for f in &exported { log.line(vc_i18n::t!("build-exportiert-f", f = &f)); log.file(Path::new(f), &vc_i18n::t!("build-zwischendatei-export")); }
                for (f, e) in &media_errs { log.line(vc_i18n::t!("build-warnung-f-e", f = &f, e = &e)); }
                log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
            }
            let media_diags: Vec<Value> = media_errs.iter().map(|(f, e)| json!({ "file": f, "line": 0, "severity": "warning", "code": "EXPORT", "message": e })).collect();
            log.stage(&vc_i18n::t!("build-assemblieren"));
            let cpu = vc_target::load_cpu(c.data, &tgt.cpu)?;
            let mut o = Options::new(abs.clone(), Policy::Warn);
            o.include_dirs.push(c.root.to_path_buf());
            o.include_dirs.push(c.data.join("sys"));
            let hooks = vc_media::include::asm_hooks(c.root, c.data, &tid);
            (o.include_provider, o.legacy_hint) = (Some(hooks.provider.clone()), Some(hooks.legacy.clone()));
            o.atari_text = tgt.text_encoding == "atari";
            o.basic_start = tgt.basic_load_addr;
            o.predefined = vc_target::chip_symbols(c.data, &tgt)?;
            // Speicherausbau (VC-20): MEM_SCREEN_RAM, MEM_COLOR_RAM, MEM_BASIC_START für die Systemdateien
            o.predefined.extend(tgt.memory_symbols());
            o.predefined.extend(tgt.test_symbols());
            // Konstanten des Projekts (`"defines"` im Target der .vcprj), z. B. DEBUG für `.if DEBUG`
            o.defines = vc_target::project_defines(c.root, &tid);
            if !o.defines.is_empty() { log.line(vc_i18n::t!("build-konstanten-aus-der-projektdatei-p1", p1 = o.defines.iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join(", "))); }
            // Assembler-Optionen aus den Einstellungen (Toolchain): Illegale Opcodes, Include-Ordner, Konstanten
            {
                let cfg = crate::settings::get().assembler;
                if !cfg.opts.is_empty() || !cfg.args.trim().is_empty() {
                    let cat = crate::args::catalog_with(Some(c.data), "vcc-asm");
                    let ctx = crate::args::ArgCtx { project: c.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), root: c.root.to_path_buf(), target: tid.clone(), ..Default::default() };
                    let a = crate::args::asm_args(cat.as_ref(), &cfg, &ctx).map_err(|e| vc_i18n::t!("build-assembler-optionen-p1", p1 = e.join("; ")))?;
                    let dirs = o.include_dirs.len();
                    for n in crate::args::apply_asm(&a, &mut o).map_err(|e| vc_i18n::t!("build-assembler-optionen-e", e = &e))? { log.line(vc_i18n::t!("build-einstellung-n", n = &n)); }
                    // Systemdateien zuletzt: zusätzliche Ordner stehen hinter dem Projektordner, aber vor `data/sys`
                    if o.include_dirs.len() > dirs { let sys = o.include_dirs.remove(1); o.include_dirs.push(sys); }
                }
            }
            if let Some(m) = tgt.memory_config() { log.line(vc_i18n::t!("build-speicherausbau-p1-p2", p1 = &m.label, p2 = &m.id)); }
            log.line(vc_i18n::t!("build-chip-symbole-des-ziels-p1-z", p1 = o.predefined.len()));
            match assemble(&text, &cpu, &o) {
                Err(d) => Ok(fail(c.root, rel, &tid, &d, log)),
                Ok(out) => {
                    let nlines: usize = out.sources.iter().map(|s| s.1.len()).sum();
                    log.line(vc_i18n::t!("build-quelltext-p1-datei-en-nlines-zeilen", p1 = out.sources.len(), nlines = &nlines, p3 = out.expanded.len()));
                    for (i, (f, ls)) in out.sources.iter().enumerate() { log.line(format!("  {}", vc_i18n::t!("build-p1-p2-zeilen", p1 = rel_of(c.root, f), p2 = ls.len()))); if i > 0 || Path::new(f) != path { log.file(Path::new(f), &if i == 0 { vc_i18n::t!("build-quelle") } else { vc_i18n::t!("build-include") }); } }
                    // Medien, die beim Einbinden umgewandelt wurden (oder aus dem Zwischenspeicher kamen)
                    for u in hooks.used.lock().unwrap().iter() { log.line(if u.cached { vc_i18n::t!("build-medium-aus-zwischenspeicher-p1", p1 = &u.rel) } else { vc_i18n::t!("build-medium-umgewandelt-p1", p1 = &u.rel) }); }
                    let macro_names: Vec<&str> = out.macros.iter().map(|m| m.0.as_str()).collect();
                    log.line(vc_i18n::t!("build-makros-p1-p2", p1 = out.macros.len(), p2 = if macro_names.is_empty() { String::new() } else { format!(" ({})", macro_names.iter().take(12).cloned().collect::<Vec<_>>().join(", ") + if macro_names.len() > 12 { ", …" } else { "" }) }));
                    log.line(vc_i18n::t!("build-durchlaeufe-p1-symbolaufloesung-bis-zur-stabilitaet", p1 = &out.passes));
                    log.line(vc_i18n::t!("build-symbole-p1", p1 = out.symbols.len()));
                    for s in &out.segments { log.line(vc_i18n::t!("build-segment-p1-p2-p3-bytes-p4", p1 = format!("{:04X}", s.start), p2 = format!("{:04X}", s.start as usize + s.bytes.len().max(1) - 1), p3 = s.bytes.len(), p4 = if s.bank != 0 { format!("  {}", vc_i18n::t!("build-bank-p1", p1 = &s.bank)) } else { String::new() })); }
                    if let Some(r) = out.run { log.line(vc_i18n::t!("build-startadresse-r", r = format!("{:04X}", r))); }
                    for b in &out.cycles { log.line(vc_i18n::t!("build-zyklen-p1-p2-p3-p4", p1 = &b.name, p2 = &b.min, p3 = &b.max, p4 = b.budget.map(|x| format!(" {}", vc_i18n::t!("build-budget-x", x = &x))).unwrap_or_default())); }
                    for w in &out.warnings { log.line(diag_line(c.root, w)); }
                    // Lizenzen der eingebundenen Quelltexte: Hinweis bei Copyleft (kein Fehler), Sperre aus der Projektdatei oder von `--deny-license`
                    let licenses = vc_asm::license::scan(&out.sources);
                    let unlicensed = out.sources.len() - licenses.len();
                    let (policy, show) = (crate::license::policy_for(c.root), |f: &str| crate::license::show_path(c.root, c.data, f));
                    let bans = crate::license::denied(&licenses, &policy, &show);
                    if !bans.is_empty() {
                        log.done(Status::Ok);
                        let diags: Vec<Diagnostic> = bans.iter().map(|m| Diagnostic::error("L002", &abs, 0, vc_core::Msg::raw(m.clone()))).collect();
                        return Ok(fail(c.root, rel, &tid, &diags, log));
                    }
                    let license_notes = crate::license::copyleft_notes(&licenses, &policy, &show);
                    for n in &license_notes { log.line(vc_i18n::t!("build-hinweis-l001-n", n = &n)); }
                    log.done(if out.warnings.is_empty() { Status::Ok } else { Status::Warn });
                    // `.cartridge` im Quelltext → Modulformat (crt für C64, car für Atari)
                    log.stage(&vc_i18n::t!("build-ausgabe-schreiben"));
                    let fmt = if out.cart.is_some() { if tgt.default_format == "xex" { "car" } else if tid == "vic20" { "vic20cart" } else { "crt" } } else { tgt.default_format.as_str() };
                    let mut ext = fmt.to_string();
                    let bytes = match fmt {
                        "xex" => to_xex(&out)?, "bin" => to_bin(&out)?, "crt" => to_crt(&out, None)?, "car" => to_car(&out, None)?,
                        // VC-20-Steckmodul: rohes Abbild, Endung = Speicherblock (.a0, .20, .40, .60)
                        "vic20cart" => { let (e, b) = vc_asm::to_vic20_cart(&out, None)?; ext = e.to_string(); b }
                        _ => to_prg(&out)?,
                    };
                    log.line(vc_i18n::t!("build-format-p1", p1 = if fmt == "vic20cart" { vc_i18n::t!("build-vc-20-modul-ext", ext = &ext) } else { fmt.to_string() }));
                    let art = out_dir.join(format!("{stem}.{ext}"));
                    std::fs::write(&art, &bytes).map_err(|e| e.to_string())?;
                    log.line(vc_i18n::t!("build-p1-bytes", p1 = bytes.len())); log.file(&art, &vc_i18n::t!("build-ergebnis"));
                    // Zwischendateien: nützlich zum Nachvollziehen und für den Debugger
                    log.stage(&vc_i18n::t!("build-zwischendateien"));
                    let mut ok = true;
                    let mut ex = format!("{}\n", vc_i18n::t!("build-expanded-head"));
                    let mut cur = String::new();
                    for (f, _, t) in &out.expanded { if *f != cur { ex.push_str(&format!("; ===== {} =====\n", rel_of(c.root, f))); cur = f.clone(); } ex.push_str(t); ex.push('\n'); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.expanded.asm"), &ex, &vc_i18n::t!("build-zwischendatei-quelltext-nach-includes"));
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.lst"), &listing(&out), &vc_i18n::t!("build-zwischendatei-listing-adresse-bytes-zeile"));
                    let mut sym = format!("{}\n", vc_i18n::t!("build-sym-head"));
                    for (n, v) in &out.symbols { sym.push_str(&format!("{n:<32} ${:<6X} {v}\n", if *v < 0 { 0 } else { *v })); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.sym.txt"), &sym, &vc_i18n::t!("build-zwischendatei-symboltabelle"));
                    let mut map = format!("{}\n", vc_i18n::t!("build-programm-rel-ziel-tid-format-ext", rel = &rel, tid = &tid, ext = &ext, p4 = bytes.len()));
                    if let Some(r) = out.run { map.push_str(&format!("{}\n", vc_i18n::t!("build-start-r", r = format!("{:04X}", r)))); }
                    map.push_str("\nSegmente:\n"); for s in &out.segments { map.push_str(&format!("  {}\n", vc_i18n::t!("build-p1-p2-p3-bytes-p4", p1 = format!("{:04X}", s.start), p2 = format!("{:04X}", s.start as usize + s.bytes.len().max(1) - 1), p3 = format!("{:>6}", s.bytes.len()), p4 = if s.bank != 0 { format!("  {}", vc_i18n::t!("build-bank-p1-2", p1 = &s.bank)) } else { String::new() }))); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.map.txt"), &map, &vc_i18n::t!("build-zwischendatei-speicherbelegung"));
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.vs"), &vice_labels(&out), &vc_i18n::t!("build-zwischendatei-labels-fuer-vice-monitor"));
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.vcdbg"), &vcdbg(&out), &vc_i18n::t!("build-zwischendatei-debuginformation-zeile-adresse"));
                    log.done(if ok { Status::Ok } else { Status::Warn });
                    let symbols: Vec<Value> = out.symbols.iter().take(500).map(|(n, v)| json!({ "name": n, "value": v })).collect();
                    Ok(json!({
                        "ok": true, "file": rel, "target": tid, "kind": "asm", "artifact": format!("build/{stem}.{ext}"), "size": bytes.len(),
                        "diagnostics": out.warnings.iter().map(|d| diag_json(c.root, d)).chain(media_diags.iter().cloned())
                            .chain(license_notes.iter().map(|n| json!({ "file": rel, "line": 0, "severity": "info", "code": "L001", "message": vc_i18n::t!("build-hinweis-l001-n", n = &n) }))).collect::<Vec<_>>(),
                        "licenses": licenses.iter().map(|e| crate::license::entry_json(e, &show)).collect::<Vec<_>>(), "unlicensed": unlicensed,
                        "export": exported,
                        "segments": out.segments.iter().map(|s| json!({ "start": s.start, "length": s.bytes.len() })).collect::<Vec<_>>(),
                        "symbols": symbols,
                        "lines": out.lines.iter().filter(|l| !l.bytes.is_empty()).map(|l| json!({ "file": rel_of(c.root, &l.file), "line": l.line, "addr": l.addr, "len": l.bytes.len(), "cyc": l.cycles.map(|(a, b)| json!([a, b])) })).collect::<Vec<_>>(),
                        "cycles": out.cycles.iter().map(|b| json!({ "name": b.name, "min": b.min, "max": b.max, "budget": b.budget })).collect::<Vec<_>>(),
                        "run": out.run,
                        "log": log.finish(),
                    }))
                }
            }
        }
        "sbas" if !SBAS_ENABLED => Err(vc_i18n::t!("build-rel-das-plattformuebergreifende-strukturierte-basic-sbas", rel = &rel)),
        "sbas" => {
            // Songs/Sprites/… als Build-Eingabe: veraltete Dateien nach build/export/ exportieren (für `include` und `blob … with`)
            log.stage(&vc_i18n::t!("build-medien-exportieren"));
            let (exported, media_errs) = vc_media::project::export_all(c.root, c.data);
            if exported.is_empty() && media_errs.is_empty() { log.line(vc_i18n::t!("build-alle-exportierten-dateien-sind-aktuell-oder")); log.done(Status::Skipped); }
            else {
                for f in &exported { log.line(vc_i18n::t!("build-exportiert-f", f = &f)); log.file(Path::new(f), &vc_i18n::t!("build-zwischendatei-export")); }
                for (f, e) in &media_errs { log.line(vc_i18n::t!("build-warnung-f-e", f = &f, e = &e)); }
                log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
            }
            let did = dialect.map(String::from).or(tgt.basic_dialect.clone()).unwrap_or_else(|| "cbm_v2".into());
            let fam = vc_basic::dialect_family(c.data, &did)?;
            log.line(vc_i18n::t!("build-basic-dialekt-did-p2", did = &did, p2 = if fam == "atari" { "Atari" } else { "Commodore" }));
            // Stufe 1: Includes einsetzen (der Text wird als Zwischendatei abgelegt)
            log.stage(&vc_i18n::t!("build-quelltext-und-includes"));
            match vc_basic::expanded_source(&text, &abs, &[c.root.to_path_buf()], Some(&tid)) {
                Err(ds) => return Ok(fail(c.root, rel, &tid, &ds, log)),
                Ok((ex, files)) => {
                    list_files(&mut log, c.root, &files);
                    log.line(vc_i18n::t!("build-ergebnis-p1-zeilen", p1 = ex.lines().count()));
                    side_file(&mut log, &out_dir, &format!("{stem}.expanded.sbas"), &ex, &vc_i18n::t!("build-zwischendatei-quelltext-nach-includes"));
                }
            }
            log.stage(&vc_i18n::t!("build-uebersetzen"));
            log.line(vc_i18n::t!("build-strukturen-if-while-repeat-do-proc"));
            if fam == "atari" {
                let d = vc_basic::atari::AtariDialect::load(c.data, &did)?;
                let mut o = vc_basic::AtariOptions::new(abs);
                o.include_dirs.push(c.root.to_path_buf());
                o.system = Some(tid.clone());
                match crate::blob::compile_atari_with_blobs(&text, &d, &o, c.root, c.data, &tgt) {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(built) => {
                        let r = built.compiled;
                        for b in &built.blobs { log.line(vc_i18n::t!("build-maschinencode-als-data-zeilen-p1-bei", p1 = &b.file, p2 = format!("{:04X}", b.addr), p3 = &b.size)); }
                        log.line(vc_i18n::t!("build-zeilen-p1", p1 = r.listing.lines().count())); log.line(vc_i18n::t!("build-variablen-p1", p1 = r.program.vars.len())); log.line(vc_i18n::t!("build-labels-p1", p1 = r.labels.len()));
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, &vc_i18n::t!("build-zwischendatei-basic-mit-zeilennummern"));
                        let vars = format!("{}\n", r.program.vars.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join("\n"));
                        side_file(&mut log, &out_dir, &format!("{stem}.vars.txt"), &vars, &vc_i18n::t!("build-zwischendatei-variablen"));
                        log.stage(&vc_i18n::t!("build-ausgabe-schreiben"));
                        let art = out_dir.join(format!("{stem}.bas"));
                        std::fs::write(&art, &r.file).map_err(|e| e.to_string())?;
                        log.line(vc_i18n::t!("build-p1-bytes-atari-basic-datei-wie", p1 = r.file.len())); log.file(&art, &vc_i18n::t!("build-ergebnis"));
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "artifact": format!("build/{stem}.bas"), "size": r.file.len(),
                                   "diagnostics": [], "listing": r.listing, "vars": r.program.vars, "log": log.finish() }))
                    }
                }
            } else {
                let d = vc_basic::Dialect::load(c.data, &did)?;
                let mut o = vc_basic::Options::new(abs);
                o.include_dirs.push(c.root.to_path_buf());
                o.system = Some(tid.clone());
                o.load_addr = tgt.basic_load_addr;
                match crate::blob::compile_with_blobs(&text, &d, &o, c.root, c.data, &tgt) {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(built) => {
                        let (r, full_prg, blob_info) = (built.compiled, built.prg, built.blobs);
                        for b in &blob_info { log.line(vc_i18n::t!("build-maschinencode-hinter-dem-basic-text-p1", p1 = &b.file, p2 = format!("{:04X}", b.addr), p3 = &b.size, p4 = &b.symbols)); }
                        let parsed = vc_basic::parse_prg(&r.prg)?;
                        let listing = vc_basic::to_classic_text(&parsed, &d);
                        log.line(vc_i18n::t!("build-zeilen-p1", p1 = r.lines.len())); log.line(vc_i18n::t!("build-variablen-p1", p1 = r.vars.len())); log.line(vc_i18n::t!("build-labels-p1", p1 = r.labels.len()));
                        for w in &r.warnings { log.line(diag_line(c.root, w)); }
                        let st = if r.warnings.is_empty() { Status::Ok } else { Status::Warn }; log.done(st);
                        log.stage(&vc_i18n::t!("build-zwischendateien"));
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &listing, &vc_i18n::t!("build-zwischendatei-basic-mit-zeilennummern-zurueckuebersetzt-aus"));
                        let vars = r.vars.iter().map(|(a, b)| vc_i18n::t!("build-a-b", a = &a, b = &b)).collect::<Vec<_>>().join("\n") + "\n";
                        side_file(&mut log, &out_dir, &format!("{stem}.vars.txt"), &vars, &vc_i18n::t!("build-zwischendatei-variablennamen-quelle-kurzer-name"));
                        let labels = r.labels.iter().map(|(n, l)| vc_i18n::t!("build-n-zeile-l", n = format!("{:<24}", n), l = &l)).collect::<Vec<_>>().join("\n") + "\n";
                        side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels, &vc_i18n::t!("build-zwischendatei-labels-zeilennummern"));
                        log.done(Status::Ok);
                        log.stage(&vc_i18n::t!("build-ausgabe-schreiben"));
                        let art = out_dir.join(format!("{stem}.prg"));
                        std::fs::write(&art, &full_prg).map_err(|e| e.to_string())?;
                        log.line(vc_i18n::t!("build-p1-bytes-prg-ladeadresse-p2", p1 = full_prg.len(), p2 = format!("{:04X}", o.load_addr.unwrap_or(d.load_addr)))); log.file(&art, &vc_i18n::t!("build-ergebnis"));
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "artifact": format!("build/{stem}.prg"), "size": full_prg.len(),
                                   "diagnostics": r.warnings.iter().map(|w| diag_json(c.root, w)).collect::<Vec<_>>(),
                                   "listing": listing, "vars": r.vars.iter().map(|(a, b)| format!("{a} → {b}")).collect::<Vec<_>>(), "log": log.finish() }))
                    }
                }
            }
        }
        "bas" | "lst" | "tur" => {
            // Natives BASIC des Systems: Zeilennummern und/oder `@label`; `#include "datei"` bindet Dateien ein (z. B. exportierte DATA-Zeilen)
            // nur wenn das Programm etwas einbindet: veraltete Medien nach build/export/ exportieren (liefert z. B. <name>.data.bas)
            if basic_uses_legacy_export(&text, &abs) {
                log.stage(&vc_i18n::t!("build-medien-exportieren"));
                let (exported, media_errs) = vc_media::project::export_all(c.root, c.data);
                if exported.is_empty() && media_errs.is_empty() { log.line(vc_i18n::t!("build-alle-exportierten-dateien-sind-aktuell-oder")); log.done(Status::Skipped); }
                else {
                    for f in &exported { log.line(vc_i18n::t!("build-exportiert-f", f = &f)); log.file(Path::new(f), &vc_i18n::t!("build-zwischendatei-export")); }
                    for (f, e) in &media_errs { log.line(vc_i18n::t!("build-warnung-f-e", f = &f, e = &e)); }
                    log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
                }
            }
            let (text, line_map, inc_warns, media) = match vc_basic::expand_includes_with(&text, &abs, &crate::basicmedia::hooks(c.root, c.data, &tid)) {
                Ok(x) => (x.text, x.map, x.warnings, x.media),
                Err(ds) => return Ok(fail(c.root, rel, &tid, &ds, log)),
            };
            for m in &media { log.line(vc_i18n::t!("build-medium-als-data-zeilen-eingebunden-p1", p1 = m)); }
            let fail = |root: &Path, rel: &str, tid: &str, ds: &[Diagnostic], log: BuildLog| fail(root, rel, tid, &vc_basic::remap_diagnostics(ds, &abs, &line_map), log);
            let did = dialect.map(String::from).or(tgt.basic_dialect.clone()).unwrap_or_else(|| "cbm_v2".into());
            let labels_json = |l: &[(String, u32)]| -> Vec<Value> { l.iter().map(|(n, ln)| json!({ "name": n, "line": ln })).collect() };
            let labels_txt = |l: &[(String, u32)]| -> String { l.iter().map(|(n, ln)| vc_i18n::t!("build-n-zeile-ln", n = format!("{:<24}", n), ln = &ln)).collect::<Vec<_>>().join("\n") + "\n" };
            let fam = vc_basic::dialect_family(c.data, &did)?;
            log.line(vc_i18n::t!("build-basic-dialekt-did-p2", did = &did, p2 = if fam == "atari" { "Atari" } else { "Commodore" }));
            log.stage(&vc_i18n::t!("build-zeilennummern-und-labels"));
            // Maschinencode (`#asm "datei.asm"`): `#include` ist schon aufgelöst; der Assembler läuft als Rückruf des Kerns
            let has_blob = text.lines().any(vc_basic::blob::is_blob_directive);
            let infos: std::cell::RefCell<Vec<crate::blob::BlobInfo>> = std::cell::RefCell::new(vec![]);
            let mut asm = |reqs: &[vc_basic::BlobReq], addr: usize, strict: bool| crate::blob::assemble_blobs(reqs, addr, &tid, c.root, c.data, &tgt, strict)
                .map(|(bytes, symbols, i, chunks)| { *infos.borrow_mut() = i; vc_basic::blob::Assembled { bytes, symbols, chunks } });
            let blob_lines = |log: &mut BuildLog, atari: bool| for b in infos.borrow().iter() {
                if atari { log.line(vc_i18n::t!("build-maschinencode-als-data-zeilen-p1-bei", p1 = &b.file, p2 = format!("{:04X}", b.addr), p3 = &b.size)); }
                else { log.line(vc_i18n::t!("build-maschinencode-hinter-dem-basic-text-p1", p1 = &b.file, p2 = format!("{:04X}", b.addr), p3 = &b.size, p4 = &b.symbols)); }
            };
            if fam == "atari" {
                let d = vc_basic::atari::AtariDialect::load(c.data, &did)?;
                let o = plain_options(c, &abs, 32767, &tid, media.len(), &mut log);
                let res = if has_blob { vc_basic::blob::compile_plain_atari_blobs(&text, &d, &o, &mut asm).map(|r| r.built) } else { vc_basic::compile_plain_atari(&text, &d, &o) };
                match res {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(r) => {
                        blob_lines(&mut log, true);
                        log.line(vc_i18n::t!("build-zeilen-p1", p1 = r.listing.lines().count())); log.line(vc_i18n::t!("build-labels-p1", p1 = r.labels.len()));
                        if r.numbers_converted > 0 { log.line(vc_i18n::t!("build-zahlen-umgewandelt-p1-hex-binaer", p1 = &r.numbers_converted)); }
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, &vc_i18n::t!("build-zwischendatei-basic-mit-zeilennummern"));
                        if !r.labels.is_empty() { side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels_txt(&r.labels), &vc_i18n::t!("build-zwischendatei-labels-zeilennummern")); }
                        let (vars, warns) = long_names_report(&mut log, &out_dir, &stem, &r.names, &r.warnings, &abs, &line_map);
                        log.stage(&vc_i18n::t!("build-ausgabe-schreiben"));
                        let art = out_dir.join(format!("{stem}.bas"));
                        std::fs::write(&art, &r.file).map_err(|e| e.to_string())?;
                        log.line(vc_i18n::t!("build-p1-bytes", p1 = r.file.len())); log.file(&art, &vc_i18n::t!("build-ergebnis"));
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "dialect": did, "artifact": format!("build/{stem}.bas"), "size": r.file.len(),
                                   "diagnostics": warns.iter().chain(inc_warns.iter()).map(|d| diag_json(c.root, d)).collect::<Vec<_>>(), "listing": r.listing, "labels": labels_json(&r.labels), "vars": vars, "log": log.finish() }))
                    }
                }
            } else {
                let d = vc_basic::Dialect::load(c.data, &did)?;
                let load = tgt.basic_load_addr.unwrap_or(d.load_addr);
                let o = plain_options(c, &abs, 63999, &tid, media.len(), &mut log);
                let res = if has_blob {
                    let place = if tid == "mega65" { vc_basic::blob::Placement::Data(Some(crate::blob::MEGA65_BLOB_BASE)) } else { vc_basic::blob::Placement::Behind };
                    vc_basic::blob::compile_plain_cbm_blobs(&text, &d, &o, load, place, &mut asm).map(|r| r.built)
                } else { vc_basic::compile_plain_cbm(&text, &d, &o, load) };
                match res {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(r) => {
                        blob_lines(&mut log, false);
                        log.line(vc_i18n::t!("build-zeilen-p1", p1 = r.listing.lines().count())); log.line(vc_i18n::t!("build-labels-p1", p1 = r.labels.len()));
                        if r.numbers_converted > 0 { log.line(vc_i18n::t!("build-zahlen-umgewandelt-p1-hex-binaer", p1 = &r.numbers_converted)); }
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, &vc_i18n::t!("build-zwischendatei-basic-mit-zeilennummern"));
                        if !r.labels.is_empty() { side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels_txt(&r.labels), &vc_i18n::t!("build-zwischendatei-labels-zeilennummern")); }
                        let (vars, warns) = long_names_report(&mut log, &out_dir, &stem, &r.names, &r.warnings, &abs, &line_map);
                        log.stage(&vc_i18n::t!("build-tokenisieren-und-ausgabe-schreiben"));
                        let art = out_dir.join(format!("{stem}.prg"));
                        std::fs::write(&art, &r.prg).map_err(|e| e.to_string())?;
                        log.line(vc_i18n::t!("build-p1-bytes-prg-ladeadresse-load", p1 = r.prg.len(), load = format!("{:04X}", load))); log.file(&art, &vc_i18n::t!("build-ergebnis"));
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "dialect": did, "artifact": format!("build/{stem}.prg"), "size": r.prg.len(),
                                   "diagnostics": warns.iter().chain(inc_warns.iter()).map(|d| diag_json(c.root, d)).collect::<Vec<_>>(), "listing": r.listing, "labels": labels_json(&r.labels), "vars": vars, "log": log.finish() }))
                    }
                }
            }
        }
        _ => Err(vc_i18n::t!("build-rel-dateityp-ext-ist-nicht-baubar", rel = &rel, ext = &ext)),
    }
}
