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
fn diag_line(root: &Path, d: &Diagnostic) -> String { format!("{} {}{}  {}", if d.is_error() { "Fehler" } else { "Warnung" }, rel_of(root, &d.file), if d.line > 0 { format!(":{}", d.line) } else { String::new() }, d.message) }

fn fail(root: &Path, file: &str, target: &str, diags: &[Diagnostic], mut log: BuildLog) -> Value {
    log.fail(diags.iter().map(|d| diag_line(root, d)));
    json!({ "ok": false, "file": file, "target": target, "diagnostics": diags.iter().map(|d| diag_json(root, d)).collect::<Vec<_>>(), "log": log.finish() })
}

pub fn build(c: &Ctx, rel: &str, target: Option<&str>) -> Result<Value, String> { build_with(c, rel, target, None) }

/// Schreibt eine Zwischendatei nach `build/` und trägt sie im Protokoll ein; Schreibfehler werden als Hinweis vermerkt
fn side_file(log: &mut BuildLog, dir: &Path, name: &str, content: &str, role: &str) -> bool {
    match std::fs::write(dir.join(name), content) { Ok(_) => { log.file(&dir.join(name), role); true } Err(e) => { log.line(format!("{name}: {e}")); false } }
}

/// Lange Variablennamen (`#longnames`): Zuordnung ins Bauprotokoll und in `<name>.vars.txt`; liefert die Zeilen für `vars` und die Warnungen (auf die Quelldatei zurückgeführt)
fn long_names_report(log: &mut BuildLog, dir: &Path, stem: &str, names: &[vc_basic::longnames::NameEntry], warnings: &[Diagnostic], abs: &str, map: &vc_basic::LineMap) -> (Vec<String>, Vec<Diagnostic>) {
    let warns = vc_basic::remap_diagnostics(warnings, abs, map);
    if !names.is_empty() {
        log.line(format!("Lange Namen: {} umgesetzt", names.len()));
        for n in names { log.line(format!("  {}", n.line())); }
        side_file(log, dir, &format!("{stem}.vars.txt"), &vc_basic::longnames::table_text(names), "Zwischendatei: lange Variablennamen → Kurznamen");
    }
    for w in &warns { log.line(format!("Warnung {}", w.message)); }
    (names.iter().map(|n| format!("{} → {}", n.long, n.short)).collect(), warns)
}

fn list_files(log: &mut BuildLog, root: &Path, files: &[(String, usize)]) {
    log.line(format!("{} Datei(en), {} Zeilen", files.len(), files.iter().map(|f| f.1).sum::<usize>()));
    for (i, (f, n)) in files.iter().enumerate() { log.line(format!("  {}  ({n} Zeilen)", rel_of(root, f))); log.file(Path::new(f), if i == 0 { "Quelle" } else { "Include" }); }
}

/// Wie [`build`], mit ausdrücklich gewähltem BASIC-Dialekt (z. B. `turbo_basic_xl`, `basic7`); sonst gilt der Standarddialekt des Targets.
pub fn build_with(c: &Ctx, rel: &str, target: Option<&str>, dialect: Option<&str>) -> Result<Value, String> {
    let path: PathBuf = c.root.join(rel);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{rel}: {e}"))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out").to_string();
    let mut log = BuildLog::new(c.root);
    log.stage("Vorbereitung");
    let tid = target.map(String::from).or_else(|| detect_target(&text)).unwrap_or_else(|| vc_target::default_target_or_c64(Some(c.data)));
    let mut tgt = vc_target::load_target(c.data, &tid)?;
    tgt.apply_memory(crate::projfile::target_setting(c.root, &tid, "memory").as_deref())?;
    let out_dir = c.root.join("build");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let abs = path.to_string_lossy().into_owned();
    log.line(format!("Quelle: {rel} ({} Zeilen, {} Bytes)", text.lines().count(), text.len()));
    log.line(format!("Zielsystem: {tid}{} · CPU {}", if target.is_some() { " (gewählt)" } else if detect_target(&text).is_some() { " (aus .target)" } else { " (Standard)" }, tgt.cpu));
    log.line(format!("Ausgabeordner: build/"));
    log.file(&path, "Quelle");
    match ext.as_str() {
        "asm" => {
            log.line(format!("Include-Suche: Projektordner, dann Systembibliothek ({}/sys)", c.data.display()));
            // Sprites, Zeichensätze, Songs als Build-Eingabe: veraltete Dateien nach build/export/*.asm exportieren (vor dem Assemblieren, damit .include greift)
            log.stage("Medien exportieren");
            let (exported, media_errs) = vc_media::project::export_all(c.root, c.data);
            if exported.is_empty() && media_errs.is_empty() { log.line("Alle exportierten Dateien sind aktuell (oder es gibt keine Medien)."); log.done(Status::Skipped); }
            else {
                for f in &exported { log.line(format!("exportiert: {f}")); log.file(Path::new(f), "Zwischendatei (Export)"); }
                for (f, e) in &media_errs { log.line(format!("Warnung {f}: {e}")); }
                log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
            }
            let media_diags: Vec<Value> = media_errs.iter().map(|(f, e)| json!({ "file": f, "line": 0, "severity": "warning", "code": "EXPORT", "message": e })).collect();
            log.stage("Assemblieren");
            let cpu = vc_target::load_cpu(c.data, &tgt.cpu)?;
            let mut o = Options::new(abs.clone(), Policy::Warn);
            o.include_dirs.push(c.root.to_path_buf());
            o.include_dirs.push(c.data.join("sys"));
            o.atari_text = tgt.text_encoding == "atari";
            o.basic_start = tgt.basic_load_addr;
            o.predefined = vc_target::chip_symbols(c.data, &tgt)?;
            // Speicherausbau (VC-20): MEM_SCREEN_RAM, MEM_COLOR_RAM, MEM_BASIC_START für die Systemdateien
            o.predefined.extend(tgt.memory_symbols());
            o.predefined.extend(tgt.test_symbols());
            // Konstanten des Projekts (`"defines"` im Target der .vcprj), z. B. DEBUG für `.if DEBUG`
            o.defines = vc_target::project_defines(c.root, &tid);
            if !o.defines.is_empty() { log.line(format!("Konstanten aus der Projektdatei: {}", o.defines.iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join(", "))); }
            // Assembler-Optionen aus den Einstellungen (Toolchain): Illegale Opcodes, Include-Ordner, Konstanten
            {
                let cfg = crate::settings::get().assembler;
                if !cfg.opts.is_empty() || !cfg.args.trim().is_empty() {
                    let cat = crate::args::catalog_with(Some(c.data), "vcc-asm");
                    let ctx = crate::args::ArgCtx { project: c.root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), root: c.root.to_path_buf(), target: tid.clone(), ..Default::default() };
                    let a = crate::args::asm_args(cat.as_ref(), &cfg, &ctx).map_err(|e| format!("Assembler-Optionen: {}", e.join("; ")))?;
                    let dirs = o.include_dirs.len();
                    for n in crate::args::apply_asm(&a, &mut o).map_err(|e| format!("Assembler-Optionen: {e}"))? { log.line(format!("Einstellung: {n}")); }
                    // Systemdateien zuletzt: zusätzliche Ordner stehen hinter dem Projektordner, aber vor `data/sys`
                    if o.include_dirs.len() > dirs { let sys = o.include_dirs.remove(1); o.include_dirs.push(sys); }
                }
            }
            if let Some(m) = tgt.memory_config() { log.line(format!("Speicherausbau: {} ({})", m.label, m.id)); }
            log.line(format!("Chip-Symbole des Ziels: {} (z. B. vic2.border)", o.predefined.len()));
            match assemble(&text, &cpu, &o) {
                Err(d) => Ok(fail(c.root, rel, &tid, &d, log)),
                Ok(out) => {
                    let nlines: usize = out.sources.iter().map(|s| s.1.len()).sum();
                    log.line(format!("Quelltext: {} Datei(en), {nlines} Zeilen; nach Include-Auflösung {} Zeilen", out.sources.len(), out.expanded.len()));
                    for (i, (f, ls)) in out.sources.iter().enumerate() { log.line(format!("  {}  ({} Zeilen)", rel_of(c.root, f), ls.len())); if i > 0 || Path::new(f) != path { log.file(Path::new(f), if i == 0 { "Quelle" } else { "Include" }); } }
                    let macro_names: Vec<&str> = out.macros.iter().map(|m| m.0.as_str()).collect();
                    log.line(format!("Makros: {}{}", out.macros.len(), if macro_names.is_empty() { String::new() } else { format!(" ({})", macro_names.iter().take(12).cloned().collect::<Vec<_>>().join(", ") + if macro_names.len() > 12 { ", …" } else { "" }) }));
                    log.line(format!("Durchläufe: {} (Symbolauflösung bis zur Stabilität, dann der endgültige Durchlauf)", out.passes));
                    log.line(format!("Symbole: {}", out.symbols.len()));
                    for s in &out.segments { log.line(format!("Segment ${:04X}–${:04X}  {} Bytes{}", s.start, s.start as usize + s.bytes.len().max(1) - 1, s.bytes.len(), if s.bank != 0 { format!("  (Bank {})", s.bank) } else { String::new() })); }
                    if let Some(r) = out.run { log.line(format!("Startadresse: ${r:04X}")); }
                    for b in &out.cycles { log.line(format!("Zyklen „{}“: {}–{}{}", b.name, b.min, b.max, b.budget.map(|x| format!(" (Budget {x})")).unwrap_or_default())); }
                    for w in &out.warnings { log.line(diag_line(c.root, w)); }
                    log.done(if out.warnings.is_empty() { Status::Ok } else { Status::Warn });
                    // `.cartridge` im Quelltext → Modulformat (crt für C64, car für Atari)
                    log.stage("Ausgabe schreiben");
                    let fmt = if out.cart.is_some() { if tgt.default_format == "xex" { "car" } else if tid == "vic20" { "vic20cart" } else { "crt" } } else { tgt.default_format.as_str() };
                    let mut ext = fmt.to_string();
                    let bytes = match fmt {
                        "xex" => to_xex(&out)?, "bin" => to_bin(&out)?, "crt" => to_crt(&out, None)?, "car" => to_car(&out, None)?,
                        // VC-20-Steckmodul: rohes Abbild, Endung = Speicherblock (.a0, .20, .40, .60)
                        "vic20cart" => { let (e, b) = vc_asm::to_vic20_cart(&out, None)?; ext = e.to_string(); b }
                        _ => to_prg(&out)?,
                    };
                    log.line(format!("Format: {}", if fmt == "vic20cart" { format!("VC-20-Modul (.{ext})") } else { fmt.to_string() }));
                    let art = out_dir.join(format!("{stem}.{ext}"));
                    std::fs::write(&art, &bytes).map_err(|e| e.to_string())?;
                    log.line(format!("{} Bytes", bytes.len())); log.file(&art, "Ergebnis");
                    // Zwischendateien: nützlich zum Nachvollziehen und für den Debugger
                    log.stage("Zwischendateien");
                    let mut ok = true;
                    let mut ex = String::from("; Quelltext nach dem Einsetzen der Includes (erzeugt von VintageCoder, nur zum Nachlesen)\n");
                    let mut cur = String::new();
                    for (f, _, t) in &out.expanded { if *f != cur { ex.push_str(&format!("; ===== {} =====\n", rel_of(c.root, f))); cur = f.clone(); } ex.push_str(t); ex.push('\n'); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.expanded.asm"), &ex, "Zwischendatei: Quelltext nach Includes");
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.lst"), &listing(&out), "Zwischendatei: Listing (Adresse, Bytes, Zeile)");
                    let mut sym = String::from("; Symbole (Name, Wert hexadezimal, dezimal)\n");
                    for (n, v) in &out.symbols { sym.push_str(&format!("{n:<32} ${:<6X} {v}\n", if *v < 0 { 0 } else { *v })); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.sym.txt"), &sym, "Zwischendatei: Symboltabelle");
                    let mut map = format!("Programm: {rel}\nZiel: {tid}\nFormat: {ext} ({} Bytes)\n", bytes.len());
                    if let Some(r) = out.run { map.push_str(&format!("Start: ${r:04X}\n")); }
                    map.push_str("\nSegmente:\n"); for s in &out.segments { map.push_str(&format!("  ${:04X}–${:04X}  {:>6} Bytes{}\n", s.start, s.start as usize + s.bytes.len().max(1) - 1, s.bytes.len(), if s.bank != 0 { format!("  Bank {}", s.bank) } else { String::new() })); }
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.map.txt"), &map, "Zwischendatei: Speicherbelegung");
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.vs"), &vice_labels(&out), "Zwischendatei: Labels für VICE-Monitor");
                    ok &= side_file(&mut log, &out_dir, &format!("{stem}.vcdbg"), &vcdbg(&out), "Zwischendatei: Debuginformation (Zeile ↔ Adresse)");
                    log.done(if ok { Status::Ok } else { Status::Warn });
                    let symbols: Vec<Value> = out.symbols.iter().take(500).map(|(n, v)| json!({ "name": n, "value": v })).collect();
                    Ok(json!({
                        "ok": true, "file": rel, "target": tid, "kind": "asm", "artifact": format!("build/{stem}.{ext}"), "size": bytes.len(),
                        "diagnostics": out.warnings.iter().map(|d| diag_json(c.root, d)).chain(media_diags.iter().cloned()).collect::<Vec<_>>(),
                        "export": exported,
                        "segments": out.segments.iter().map(|s| json!({ "start": s.start, "length": s.bytes.len() })).collect::<Vec<_>>(),
                        "symbols": symbols,
                        "lines": out.lines.iter().filter(|l| !l.bytes.is_empty()).map(|l| json!({ "file": rel_of(c.root, &l.file), "line": l.line, "addr": l.addr, "len": l.bytes.len() })).collect::<Vec<_>>(),
                        "cycles": out.cycles.iter().map(|b| json!({ "name": b.name, "min": b.min, "max": b.max, "budget": b.budget })).collect::<Vec<_>>(),
                        "run": out.run,
                        "log": log.finish(),
                    }))
                }
            }
        }
        "sbas" if !SBAS_ENABLED => Err(format!("{rel}: Das plattformübergreifende strukturierte BASIC (.sbas) ist vorerst abgeschaltet. Bitte natives BASIC des Systems verwenden (.bas: mit oder ohne Zeilennummern, Sprungmarken @name).")),
        "sbas" => {
            // Songs/Sprites/… als Build-Eingabe: veraltete Dateien nach build/export/ exportieren (für `include` und `blob … with`)
            log.stage("Medien exportieren");
            let (exported, media_errs) = vc_media::project::export_all(c.root, c.data);
            if exported.is_empty() && media_errs.is_empty() { log.line("Alle exportierten Dateien sind aktuell (oder es gibt keine Medien)."); log.done(Status::Skipped); }
            else {
                for f in &exported { log.line(format!("exportiert: {f}")); log.file(Path::new(f), "Zwischendatei (Export)"); }
                for (f, e) in &media_errs { log.line(format!("Warnung {f}: {e}")); }
                log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
            }
            let did = dialect.map(String::from).or(tgt.basic_dialect.clone()).unwrap_or_else(|| "cbm_v2".into());
            let fam = vc_basic::dialect_family(c.data, &did)?;
            log.line(format!("BASIC-Dialekt: {did} ({})", if fam == "atari" { "Atari" } else { "Commodore" }));
            // Stufe 1: Includes einsetzen (der Text wird als Zwischendatei abgelegt)
            log.stage("Quelltext und Includes");
            match vc_basic::expanded_source(&text, &abs, &[c.root.to_path_buf()], Some(&tid)) {
                Err(ds) => return Ok(fail(c.root, rel, &tid, &ds, log)),
                Ok((ex, files)) => {
                    list_files(&mut log, c.root, &files);
                    log.line(format!("Ergebnis: {} Zeilen", ex.lines().count()));
                    side_file(&mut log, &out_dir, &format!("{stem}.expanded.sbas"), &ex, "Zwischendatei: Quelltext nach Includes");
                }
            }
            log.stage("Übersetzen");
            log.line("Strukturen (if/while/repeat/do, proc/call, Labels, Konstanten) auflösen, Zeilennummern vergeben, Variablen auf kurze Namen abbilden, tokenisieren");
            if fam == "atari" {
                let d = vc_basic::atari::AtariDialect::load(c.data, &did)?;
                let mut o = vc_basic::AtariOptions::new(abs);
                o.include_dirs.push(c.root.to_path_buf());
                o.system = Some(tid.clone());
                match crate::blob::compile_atari_with_blobs(&text, &d, &o, c.root, c.data, &tgt) {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(built) => {
                        let r = built.compiled;
                        for b in &built.blobs { log.line(format!("Maschinencode als DATA-Zeilen: {} bei ${:04X}, {} Bytes", b.file, b.addr, b.size)); }
                        log.line(format!("Zeilen: {}", r.listing.lines().count())); log.line(format!("Variablen: {}", r.program.vars.len())); log.line(format!("Labels: {}", r.labels.len()));
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, "Zwischendatei: BASIC mit Zeilennummern");
                        let vars = format!("{}\n", r.program.vars.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join("\n"));
                        side_file(&mut log, &out_dir, &format!("{stem}.vars.txt"), &vars, "Zwischendatei: Variablen");
                        log.stage("Ausgabe schreiben");
                        let art = out_dir.join(format!("{stem}.bas"));
                        std::fs::write(&art, &r.file).map_err(|e| e.to_string())?;
                        log.line(format!("{} Bytes (Atari-BASIC-Datei, wie von SAVE geschrieben)", r.file.len())); log.file(&art, "Ergebnis");
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
                        for b in &blob_info { log.line(format!("Maschinencode hinter dem BASIC-Text: {} bei ${:04X}, {} Bytes, {} Symbole als Konstanten", b.file, b.addr, b.size, b.symbols)); }
                        let parsed = vc_basic::parse_prg(&r.prg)?;
                        let listing = vc_basic::to_classic_text(&parsed, &d);
                        log.line(format!("Zeilen: {}", r.lines.len())); log.line(format!("Variablen: {}", r.vars.len())); log.line(format!("Labels: {}", r.labels.len()));
                        for w in &r.warnings { log.line(diag_line(c.root, w)); }
                        let st = if r.warnings.is_empty() { Status::Ok } else { Status::Warn }; log.done(st);
                        log.stage("Zwischendateien");
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &listing, "Zwischendatei: BASIC mit Zeilennummern (zurückübersetzt aus dem Programm)");
                        let vars = r.vars.iter().map(|(a, b)| format!("{a} → {b}")).collect::<Vec<_>>().join("\n") + "\n";
                        side_file(&mut log, &out_dir, &format!("{stem}.vars.txt"), &vars, "Zwischendatei: Variablennamen (Quelle → kurzer Name)");
                        let labels = r.labels.iter().map(|(n, l)| format!("{n:<24} Zeile {l}")).collect::<Vec<_>>().join("\n") + "\n";
                        side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels, "Zwischendatei: Labels → Zeilennummern");
                        log.done(Status::Ok);
                        log.stage("Ausgabe schreiben");
                        let art = out_dir.join(format!("{stem}.prg"));
                        std::fs::write(&art, &full_prg).map_err(|e| e.to_string())?;
                        log.line(format!("{} Bytes (PRG, Ladeadresse ${:04X})", full_prg.len(), o.load_addr.unwrap_or(d.load_addr))); log.file(&art, "Ergebnis");
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
            if text.lines().any(|l| vc_basic::include_directive(l).is_some()) {
                log.stage("Medien exportieren");
                let (exported, media_errs) = vc_media::project::export_all(c.root, c.data);
                if exported.is_empty() && media_errs.is_empty() { log.line("Alle exportierten Dateien sind aktuell (oder es gibt keine Medien)."); log.done(Status::Skipped); }
                else {
                    for f in &exported { log.line(format!("exportiert: {f}")); log.file(Path::new(f), "Zwischendatei (Export)"); }
                    for (f, e) in &media_errs { log.line(format!("Warnung {f}: {e}")); }
                    log.done(if media_errs.is_empty() { Status::Ok } else { Status::Warn });
                }
            }
            let (text, line_map) = match vc_basic::expand_includes(&text, &abs) { Ok(x) => x, Err(ds) => return Ok(fail(c.root, rel, &tid, &ds, log)) };
            let fail = |root: &Path, rel: &str, tid: &str, ds: &[Diagnostic], log: BuildLog| fail(root, rel, tid, &vc_basic::remap_diagnostics(ds, &abs, &line_map), log);
            let did = dialect.map(String::from).or(tgt.basic_dialect.clone()).unwrap_or_else(|| "cbm_v2".into());
            let labels_json = |l: &[(String, u32)]| -> Vec<Value> { l.iter().map(|(n, ln)| json!({ "name": n, "line": ln })).collect() };
            let labels_txt = |l: &[(String, u32)]| -> String { l.iter().map(|(n, ln)| format!("{n:<24} Zeile {ln}")).collect::<Vec<_>>().join("\n") + "\n" };
            let fam = vc_basic::dialect_family(c.data, &did)?;
            log.line(format!("BASIC-Dialekt: {did} ({})", if fam == "atari" { "Atari" } else { "Commodore" }));
            log.stage("Zeilennummern und Labels");
            if fam == "atari" {
                let d = vc_basic::atari::AtariDialect::load(c.data, &did)?;
                let o = vc_basic::PlainOptions::new(abs.clone(), 32767);
                match vc_basic::compile_plain_atari(&text, &d, &o) {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(r) => {
                        log.line(format!("Zeilen: {}", r.listing.lines().count())); log.line(format!("Labels: {}", r.labels.len()));
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, "Zwischendatei: BASIC mit Zeilennummern");
                        if !r.labels.is_empty() { side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels_txt(&r.labels), "Zwischendatei: Labels → Zeilennummern"); }
                        let (vars, warns) = long_names_report(&mut log, &out_dir, &stem, &r.names, &r.warnings, &abs, &line_map);
                        log.stage("Ausgabe schreiben");
                        let art = out_dir.join(format!("{stem}.bas"));
                        std::fs::write(&art, &r.file).map_err(|e| e.to_string())?;
                        log.line(format!("{} Bytes", r.file.len())); log.file(&art, "Ergebnis");
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "dialect": did, "artifact": format!("build/{stem}.bas"), "size": r.file.len(),
                                   "diagnostics": warns.iter().map(|d| diag_json(c.root, d)).collect::<Vec<_>>(), "listing": r.listing, "labels": labels_json(&r.labels), "vars": vars, "log": log.finish() }))
                    }
                }
            } else {
                let d = vc_basic::Dialect::load(c.data, &did)?;
                let load = tgt.basic_load_addr.unwrap_or(d.load_addr);
                let o = vc_basic::PlainOptions::new(abs.clone(), 63999);
                match vc_basic::compile_plain_cbm(&text, &d, &o, load) {
                    Err(ds) => Ok(fail(c.root, rel, &tid, &ds, log)),
                    Ok(r) => {
                        log.line(format!("Zeilen: {}", r.listing.lines().count())); log.line(format!("Labels: {}", r.labels.len()));
                        side_file(&mut log, &out_dir, &format!("{stem}.classic.txt"), &r.listing, "Zwischendatei: BASIC mit Zeilennummern");
                        if !r.labels.is_empty() { side_file(&mut log, &out_dir, &format!("{stem}.labels.txt"), &labels_txt(&r.labels), "Zwischendatei: Labels → Zeilennummern"); }
                        let (vars, warns) = long_names_report(&mut log, &out_dir, &stem, &r.names, &r.warnings, &abs, &line_map);
                        log.stage("Tokenisieren und Ausgabe schreiben");
                        let art = out_dir.join(format!("{stem}.prg"));
                        std::fs::write(&art, &r.prg).map_err(|e| e.to_string())?;
                        log.line(format!("{} Bytes (PRG, Ladeadresse ${load:04X})", r.prg.len())); log.file(&art, "Ergebnis");
                        Ok(json!({ "ok": true, "file": rel, "target": tid, "kind": "basic", "dialect": did, "artifact": format!("build/{stem}.prg"), "size": r.prg.len(),
                                   "diagnostics": warns.iter().map(|d| diag_json(c.root, d)).collect::<Vec<_>>(), "listing": r.listing, "labels": labels_json(&r.labels), "vars": vars, "log": log.finish() }))
                    }
                }
            }
        }
        _ => Err(format!("{rel}: Dateityp '.{ext}' ist nicht baubar (unterstützt: .asm, .bas, .lst)")),
    }
}
