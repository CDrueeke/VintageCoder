// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc` – Kommandozeile der VintageCoder-Toolchain.
mod completion;
mod emulator;
mod help;
mod manpage;
mod markdown;
mod roms;
mod turbo;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vc_asm::{assemble, disassemble, listing, to_bin, to_car, to_crt, to_prg, to_rom, to_xex, vcdbg, vice_labels, Options, Policy};


fn find_data_dir(explicit: Option<String>) -> Option<PathBuf> {
    if let Some(p) = explicit.or_else(|| std::env::var("VC_DATA").ok()) { return Some(PathBuf::from(p)); }
    // 1. ab dem Ort des Programms (installiertes vcc, data\ liegt daneben; so gewinnt bei mehreren Installationen nicht eine fremde Datenbank
    // aus dem Arbeitsverzeichnis), 2. ab dem aktuellen Ordner nach oben (Arbeiten im Quellbaum, das Programm liegt dann im Zielordner von Cargo)
    std::env::current_exe().ok().and_then(|e| vc_ide::find_data_dir(&e))
        .or_else(|| std::env::current_dir().ok().and_then(|d| vc_ide::find_data_dir(&d)))
}

/// `--json` ist gesetzt: Meldungen auf der Fehlerausgabe erscheinen als JSON-Zeilen (`severity`, `code`, `key`, `args`, `message`), damit Werkzeuge nicht auf den Text angewiesen sind
static JSON_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn json_mode() -> bool { JSON_MODE.load(std::sync::atomic::Ordering::Relaxed) }

/// Meldung auf der Fehlerausgabe: Text in der gewählten Sprache; mit `--json` ein JSON-Objekt mit `code` (= Schlüssel der Meldung), `key`, `args` und `message`
fn emit(severity: &str, m: &vc_core::Msg) {
    if json_mode() { eprintln!("{}", serde_json::json!({ "severity": severity, "code": m.key, "key": m.key, "args": m.to_json()["args"], "message": m.text() })); }
    else { eprintln!("{}", m.text()); }
}
/// Fehlermeldung (ohne den Prozess zu beenden)
fn err_msg(m: vc_core::Msg) { emit("error", &m); }
/// Hinweis oder Warnung
fn note(m: vc_core::Msg) { emit("note", &m); }

/// Gibt Diagnosen mit Quellzeile aus (mit `--json`: als JSON-Zeile mit `code`, `key`, `args`, `range` und `message`).
fn print_diag(d: &vc_core::Diagnostic) {
    let line = std::fs::read_to_string(&d.file).ok().and_then(|t| t.lines().nth(d.line.wrapping_sub(1)).map(str::to_string));
    vc_ide::debuglog::log("fehler", &d.render(line.as_deref()));
    if json_mode() { eprintln!("{}", d.to_json(vc_i18n::language())); } else { eprint!("{}", d.render(line.as_deref())); }
}

/// Fehler mit Schlüssel und Parametern (Kennung = Schlüssel); Exit-Code 1
fn fail_msg(m: vc_core::Msg) -> ExitCode { vc_ide::debuglog::log("fehler", &m.text()); emit("error", &m); ExitCode::FAILURE }
/// Fehler, dessen Text schon fertig vorliegt (aus Bibliotheken); `code` ist `cli-error-text`, der Text steht im Parameter `text`
fn fail(msg: impl std::fmt::Display) -> ExitCode { fail_msg(vc_core::msg!("cli-error-text", text = msg.to_string())) }
/// Fehler beim Laden eines Targets: fehlt es in diesem Release, endet `vcc` mit Exit-Code 2
fn fail_target(msg: String) -> ExitCode { let missing = msg.contains("in diesem Release nicht enthalten"); err_msg(vc_core::msg!("cli-error-text", text = msg)); if missing { ExitCode::from(2) } else { ExitCode::FAILURE } }

/// Globale Option `--lang de|en` (oder `--lang=de`) aus den Argumenten nehmen. Gibt die übrigen Argumente und die Sprache zurück; ein ungültiger Wert ist ein Fehler
/// (Text in beiden Sprachen, weil noch keine gilt).
fn lang_option(args: Vec<String>) -> Result<(Vec<String>, Option<vc_i18n::Lang>), String> {
    let both = |m: vc_core::Msg| format!("{}\n{}", m.text_in(vc_i18n::Lang::De), m.text_in(vc_i18n::Lang::En));
    let (mut rest, mut lang, mut it) = (vec![], None, args.into_iter());
    while let Some(a) = it.next() {
        let value = if a == "--lang" { Some(it.next().ok_or_else(|| both(vc_core::msg!("cli-lang-missing-value")))?) } else { a.strip_prefix("--lang=").map(str::to_string) };
        match value {
            Some(v) => lang = Some(match v.trim().to_ascii_lowercase().as_str() { "de" => vc_i18n::Lang::De, "en" => vc_i18n::Lang::En, "xx" => vc_i18n::Lang::Xx, _ => return Err(both(vc_core::msg!("cli-lang-invalid", v = &v))) }),
            None => rest.push(a),
        }
    }
    Ok((rest, lang))
}

fn main() -> ExitCode {
    // Sprache: `--lang` für diesen Aufruf, sonst `VC_LANG`, sonst die Einstellung (Automatisch = Systemsprache); Unbekanntes ergibt Englisch
    let (args, lang) = match lang_option(std::env::args().skip(1).collect()) { Ok(x) => x, Err(e) => { eprintln!("{e}"); return ExitCode::from(2) } };
    match lang { Some(l) => vc_i18n::set_language(l), None => { vc_i18n::apply(vc_i18n::Choice::from_code(&vc_ide::settings::effective().language)); } }
    // `--json` gibt es nur bei einigen Befehlen; nur dort erscheinen Meldungen als JSON-Zeilen (sonst bleibt „unbekannte Option“ ein Text)
    let json_cmd = matches!(args.first().map(String::as_str), Some("basic" | "basic2asm" | "build" | "debuglog" | "feedback" | "update" | "roms" | "turbo"));
    JSON_MODE.store(json_cmd && args.iter().any(|a| a == "--json"), std::sync::atomic::Ordering::Relaxed);
    // Auswahl des Releases (`data/release.json` neben dem Programm, `VC_RELEASE` hat Vorrang); ohne Datei ist alles an
    if let Some(w) = vc_ide::features::init(find_data_dir(None).as_deref()) { note(vc_core::msg!("cli-warning", text = &w)); }
    if let Some(code) = help::dispatch(&args) { return code; }
    if vc_ide::debuglog::init(false) { vc_ide::debuglog::install_panic_hook(); vc_ide::debuglog::log("aufruf", &format!("vcc {}", args.join(" "))); }
    match args.first().map(String::as_str) {
        Some("--version") | Some("-V") | Some("version") => { println!("vcc {}", env!("CARGO_PKG_VERSION")); ExitCode::SUCCESS }
        Some("--license") => { println!("{}", vc_ide::license::notice_short()); ExitCode::SUCCESS }
        Some("asm") => asm(&args[1..]),
        Some("dis") => dis(&args[1..]),
        Some("inc") => inc(&args[1..]),
        Some("basic") if args.get(1).map(String::as_str) == Some("ast") => basic2asm(&args[2..], true),
        Some("basic2asm") => basic2asm(&args[1..], false),
        Some("basic") => basic(&args[1..]),
        Some("disk") => disk(&args[1..]),
        Some("build") => build_cmd(&args[1..]),
        Some("test") => test_cmd(&args[1..]),
        Some("streams") => match vc_ide::emulators::require_device("ultimate64") { Ok(()) => streams_cmd(&args[1..]), Err(e) => fail(format!("vcc streams: {e}")) },
        Some("media") | Some("asset") => media(&args[1..]),
        Some("import") => import_cmd(&args[1..]),
        Some("roms") => roms::run(&args[1..]),
        Some("turbo") => turbo::run(&args[1..]),
        Some("emulator") => emulator::run(&args[1..]),
        Some("migrate") => migrate_cmd(&args[1..]),
        Some("feedback") => feedback_cmd(&args[1..]),
        Some("update") => update_cmd(&args[1..]),
        Some("debuglog") => debuglog_cmd(&args[1..]),
        Some("new") => new_project(&args[1..]),
        Some("organize") => organize_cmd(&args[1..]),
        Some("add") => add_cmd(&args[1..]),
        Some(other) => help::unknown_command(other),
        None => ExitCode::from(2),
    }
}

fn asm(args: &[String]) -> ExitCode {
    let (mut input, mut output, mut target, mut cpu_id, mut illegals) = (None, None, String::new(), None, Policy::Warn);
    let (mut format, mut labels, mut debug, mut lst, mut data_dir, mut cart) = (None::<String>, None, None, None, None, None::<String>);
    let (mut dirs, mut defines) = (vec![], vec![]);
    let mut memory = None::<String>;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned();
        match a.as_str() {
            "-o" => output = val(), "--target" => target = val().unwrap_or(target.clone()), "--cpu" => cpu_id = val(),
            "--format" => format = val(), "--cart" => cart = val(), "--labels" => labels = val(), "--debug" => debug = val(),
            "--listing" => lst = val(), "--data-dir" => data_dir = val(), "--memory" => memory = val(), "-I" => dirs.extend(val().map(PathBuf::from)),
            "-D" => match val().and_then(|d| {
                let (n, v) = d.split_once('=').map(|(n, v)| (n.to_string(), v.to_string())).unwrap_or((d.clone(), "1".into()));
                vc_target::parse_num(&v).map(|v| (n, v))
            }) { Some(d) => defines.push(d), None => return fail(vc_i18n::t!("cli-define-expected")) },
            "--illegals" => match val().as_deref().and_then(Policy::parse) { Some(p) => illegals = p, None => return fail("--illegals: deny|warn|allow") },
            s if !s.starts_with('-') => input = Some(s.to_string()),
            s => { return help::unknown_option("asm", s); }
        }
    }
    let Some(input) = input else { return help::usage_error("asm") };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir-hint")) };
    if target.is_empty() { target = vc_target::default_target_or_c64(Some(&data)); }
    let mut tgt = match vc_target::load_target(&data, &target) { Ok(t) => t, Err(e) => return fail_target(e) };
    if let Err(e) = tgt.apply_memory(memory.as_deref()) { return fail(e); }
    let cpu = match vc_target::load_cpu(&data, cpu_id.as_deref().unwrap_or(&tgt.cpu)) { Ok(c) => c, Err(e) => return fail(e) };
    let src = match vc_core::textfile::read_source(&input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
    let mut opts = Options::new(input.clone(), illegals);
    opts.include_dirs = dirs;
    opts.include_dirs.push(data.join("sys"));
    // Medien als Include: `.include "graphics/hero.vcsprite"` wird beim Einbinden umgewandelt (Projektordner = nächster Ordner mit Projektdatei)
    {
        let abs_in = std::fs::canonicalize(&input).unwrap_or_else(|_| PathBuf::from(&input));
        let hooks = vc_media::include::asm_hooks(&vc_ide::launch::project_root_of(&abs_in), &data, &tgt.id);
        (opts.include_provider, opts.legacy_hint) = (Some(hooks.provider), Some(hooks.legacy));
    }
    opts.defines = defines;
    opts.atari_text = tgt.text_encoding == "atari";
    opts.basic_start = tgt.basic_load_addr;
    opts.predefined = match vc_target::chip_symbols(&data, &tgt) { Ok(v) => v, Err(e) => return fail(e) };
    opts.predefined.extend(tgt.memory_symbols());
    opts.predefined.extend(tgt.test_symbols());
    match assemble(&src, &cpu, &opts) {
        Ok(out) => {
            for w in &out.warnings { print_diag(w); }
            // ohne --format: Modul (`.cartridge` im Quelltext) → crt/car, sonst das Standardformat des Targets
            let format = format.unwrap_or_else(|| if out.cart.is_some() { if tgt.default_format == "xex" { "car".into() } else if tgt.id == "vic20" { "vic20cart".into() } else { "crt".into() } } else { tgt.default_format.clone() });
            // VC-20-Steckmodul: rohes Abbild, Dateiendung = Speicherblock (a0, 20, 40, 60)
            let mut ext_override: Option<&'static str> = None;
            let vcart = if matches!(format.as_str(), "vic20cart" | "a0" | "20" | "40" | "60") { Some(vc_asm::to_vic20_cart(&out, if format == "vic20cart" { None } else { Some(format.as_str()) })) } else { None };
            let bytes = if let Some(r) = vcart { r.map(|(e, b)| { ext_override = Some(e); b }) } else { match format.as_str() { "bin" => to_bin(&out), "xex" => to_xex(&out), "crt" => to_crt(&out, cart.as_deref()), "car" => to_car(&out, cart.as_deref()), "rom" => to_rom(&out, cart.as_deref()), "d81" | "d71" | "d64" => to_prg(&out), _ => to_prg(&out) } };
            let mut bytes = match bytes { Ok(b) => b, Err(e) => return fail(e.text()) };
            if matches!(format.as_str(), "d64" | "d71" | "d81") {     // PRG in ein Diskimage packen (Name = Dateiname ohne Endung, groß, höchstens 16 Zeichen)
                let stem: String = Path::new(&input).file_stem().map(|s| s.to_string_lossy().to_ascii_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).take(16).collect()).unwrap_or_default();
                let name = if stem.is_empty() { "PROGRAMM".to_string() } else { stem };
                bytes = match format.as_str() {
                    "d64" => vc_disk::D64::blank(&name, "VC").and_then(|mut d| d.add_prg(&name, &bytes).map(|_| d.to_bytes())),
                    f => vc_disk::Disk::blank(if f == "d71" { vc_disk::Kind::D71 } else { vc_disk::Kind::D81 }, &name, "VC").and_then(|mut d| d.add_prg(&name, &bytes).map(|_| d.to_bytes())),
                }.map_err(|e| e).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(1) });
            }
            let out_path = output.unwrap_or_else(|| Path::new(&input).with_extension(ext_override.unwrap_or(&format)).to_string_lossy().into_owned());
            if let Err(e) = std::fs::write(&out_path, &bytes) { return fail(format!("{out_path}: {e}")); }
            if let Some(l) = labels { let _ = std::fs::write(l, vice_labels(&out)); }
            if let Some(d) = debug { let _ = std::fs::write(d, vcdbg(&out)); }
            if let Some(l) = lst { let _ = std::fs::write(l, listing(&out)); }
            println!("{}", vc_i18n::t!("cli-asm-done", out_path = &out_path, bytes = bytes.len(), symbols = out.symbols.len(), warnings = out.warnings.len()));
            ExitCode::SUCCESS
        }
        Err(diags) => { for d in &diags { print_diag(d); } ExitCode::FAILURE }
    }
}

fn dis(args: &[String]) -> ExitCode {
    let (mut input, mut start, mut cpu_id, mut data_dir) = (None, None, "6510".to_string(), None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned();
        match a.as_str() {
            "--start" => start = val().and_then(|v| vc_target::parse_num(&v)),
            "--cpu" => cpu_id = val().unwrap_or(cpu_id.clone()),
            "--data-dir" => data_dir = val(),
            s if !s.starts_with('-') => input = Some(s.to_string()),
            s => { return help::unknown_option("dis", s); }
        }
    }
    let Some(input) = input else { return help::usage_error("dis") };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    let cpu = match vc_target::load_cpu(&data, &cpu_id) { Ok(c) => c, Err(e) => return fail(e) };
    let mut bytes = match std::fs::read(&input) { Ok(b) => b, Err(e) => return fail(format!("{input}: {e}")) };
    if vc_ide::diskimage::is_xex(&input, &bytes) && start.is_none() {
        let blocks = match vc_disk::xex_blocks(&bytes) { Ok(b) => b, Err(e) => return fail(format!("{input}: {e}")) };
        for (i, (addr, data)) in blocks.iter().enumerate() {
            println!("{}", vc_i18n::t!("cli-dis-segment", n = i + 1, addr = format!("{:04x}", addr), end = format!("{:04x}", *addr as usize + data.len() - 1), size = data.len()));
            for l in disassemble(&cpu, data, *addr) { print_dis(&l); }
        }
        return ExitCode::SUCCESS;
    }
    let addr = if input.ends_with(".prg") && bytes.len() >= 2 && start.is_none() {
        let a = bytes[0] as u16 | (bytes[1] as u16) << 8; bytes.drain(..2); a
    } else { start.unwrap_or(0) as u16 };
    for l in disassemble(&cpu, &bytes, addr) { print_dis(&l); }
    ExitCode::SUCCESS
}

fn print_dis(l: &vc_asm::DisLine) {
    let hex: Vec<String> = l.bytes.iter().map(|b| format!("{b:02x}")).collect();
    println!("{:04x}  {:<9} {}{}", l.addr, hex.join(" "), l.text, if l.illegal { format!("   {}", vc_i18n::t!("cli-dis-undocumented")) } else { String::new() });
}


fn inc(args: &[String]) -> ExitCode {
    let (mut target, mut output, mut data_dir) = (None, None, None);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--target" => target = it.next().cloned(), "-o" => output = it.next().cloned(), "--data-dir" => data_dir = it.next().cloned(),
            s => { return help::unknown_option("inc", s); }
        }
    }
    let Some(target) = target else { return help::usage_error("inc") };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    let tgt = match vc_target::load_target(&data, &target) { Ok(t) => t, Err(e) => return fail_target(e) };
    match vc_target::generate_include(&data, &tgt) {
        Ok(text) => match output {
            Some(o) => match std::fs::write(&o, text) { Ok(()) => { println!("{}", vc_i18n::t!("cli-inc-written", o = &o)); ExitCode::SUCCESS } Err(e) => fail_msg(vc_core::msg!("cli-file-error-o", o = &o, e = &e)) },
            None => { print!("{text}"); ExitCode::SUCCESS }
        },
        Err(e) => fail(e),
    }
}

/// Liest eine VICE-Labeldatei (`al C:080d .start`) als BASIC-Konstanten `asm.start`.
fn label_consts(path: &str, prefix: &str) -> Result<Vec<(String, i64)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    Ok(text.lines().filter_map(|l| {
        let mut w = l.split_whitespace();
        (w.next()? == "al").then_some(())?;
        let addr = i64::from_str_radix(w.next()?.trim_start_matches("C:"), 16).ok()?;
        let name = w.next()?.trim_start_matches('.');
        Some((format!("{prefix}.{name}"), addr))
    }).collect())
}

/// `vcc basic2asm` und `vcc basic ast`
fn basic2asm(args: &[String], ast_only: bool) -> ExitCode {
    let (mut input, mut output, mut target, mut map, mut data_dir) = (None::<String>, None::<String>, "c64".to_string(), None::<String>, None::<String>);
    let mut o = vc_basic::b2a::B2aOptions::new("");
    let (mut check, mut json, mut round) = (ast_only, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" => output = it.next().cloned(), "--target" => target = it.next().cloned().unwrap_or_default(), "--map" => map = it.next().cloned(),
            "--data-dir" => data_dir = it.next().cloned(),
            "--stage" => match it.next().and_then(|v| v.parse().ok()) { Some(n) => o.stage = n, None => return fail_msg(vc_core::msg!("cli-stage-needs-number")) },
            "--int-div" => o.int_div = true, "--float" => o.force_float = true, "--no-track-line" => o.track_line = false, "--no-rem" => o.no_rem = true, "--check" => check = true, "--json" => json = true, "--roundtrip" => round = true, "--turbo" => o.turbo = true,
            "--style" => match it.next().map(String::as_str) { Some("compact") => o.inline = false, Some("inline") => o.inline = true, _ => return fail_msg(vc_core::msg!("cli-style-invalid")) },
            "--runtime" => match it.next().map(String::as_str) { Some("include") => o.embed = false, Some("embed") => o.embed = true, _ => return fail_msg(vc_core::msg!("cli-runtime-invalid")) },
            s if !s.starts_with('-') => input = Some(s.to_string()),
            s => { return help::unknown_option("basic2asm", s); }
        }
    }
    let Some(input) = input else { return help::usage_error("basic2asm") };
    let Some(ti) = vc_basic::b2a::target_info(&target) else { return fail_msg(vc_core::msg!("cli-basic2asm-target-unsupported", target = &target, available = vc_basic::b2a::TARGETS.iter().map(|t| t.id).collect::<Vec<_>>().join(", "))) };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir-hint")) };
    let d = match vc_basic::Dialect::load(&data, ti.dialect) { Ok(d) => d, Err(e) => return fail(e) };
    let src = match vc_core::textfile::read_source(&input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
    o.file = input.clone();
    o.target = target.clone();
    o.data_dir = Some(data.clone());
    if ast_only && round {
        return match vc_basic::b2a::roundtrip(&src, &d, &o) { Ok(t) => { print!("{t}"); err_msg(vc_core::msg!("cli-basic-roundtrip-ok", input = &input)); ExitCode::SUCCESS } Err(e) => { e.iter().for_each(print_diag); ExitCode::FAILURE } };
    }
    if ast_only {
        if json { return match vc_basic::b2a::parse(&src, &d, &o) { Ok(p) => { println!("{}", serde_json::to_string_pretty(&vc_basic::b2a::print::to_json(&p)).unwrap_or_default()); ExitCode::SUCCESS } Err(e) => { e.iter().for_each(print_diag); ExitCode::FAILURE } }; }
        return match vc_basic::b2a::parse(&src, &d, &o) { Ok(p) => { println!("{:#?}", p.lines); ExitCode::SUCCESS } Err(e) => { e.iter().for_each(print_diag); ExitCode::FAILURE } };
    }
    match vc_basic::b2a::convert(&src, &d, &o) {
        Ok(c) => {
            for w in &c.warnings { print_diag(w); }
            if check { println!("{}", vc_i18n::t!("cli-basic2asm-check-ok", input = &input, stage = &c.stage, lines = c.map.len())); return ExitCode::SUCCESS; }
            let out_path = output.unwrap_or_else(|| Path::new(&input).with_extension("asm").to_string_lossy().into_owned());
            if let Err(e) = std::fs::write(&out_path, &c.asm) { return fail(format!("{out_path}: {e}")); }
            let map_path = map.unwrap_or_else(|| Path::new(&input).with_extension("b2a.json").to_string_lossy().into_owned());
            if let Err(e) = std::fs::write(&map_path, c.map_json(&input)) { return fail(format!("{map_path}: {e}")); }
            println!("{}", vc_i18n::t!("cli-basic2asm-done", out_path = &out_path, stage = &c.stage, lines = c.map.len(), modules = c.modules.join(", ")));
            ExitCode::SUCCESS
        }
        Err(e) => { e.iter().for_each(print_diag); ExitCode::FAILURE }
    }
}

fn basic(args: &[String]) -> ExitCode {
    let (mut input, mut output, mut target, mut dialect_id, mut data_dir) = (None, None, String::new(), None, None);
    let (mut mode, mut structured, mut pack, mut map, mut labels, mut plain) = ("compile", false, false, None, None, false);
    let (mut start, mut step, mut dirs, mut defs) = (10u32, 10u32, vec![], vec![]);
    let mut memory = None::<String>;
    let (mut suggest, mut apply) = (false, None::<String>);
    let (mut convert, mut write_back, mut preview, mut range) = (None::<&str>, false, false, None::<(u32, u32)>);
    let (mut check_only, mut no_check, mut disabled, mut compact) = (false, false, Vec::<String>::new(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned();
        match a.as_str() {
            "--to-labels" => convert = Some("labels"), "--to-numbers" => convert = Some("numbers"), "--renumber" => convert = Some("renumber"),
            "--write" => write_back = true, "--preview" => preview = true,
            "--compact" => compact = true, "--check" => check_only = true, "--no-check" => no_check = true, "--disable" => disabled.extend(val().unwrap_or_default().split(',').map(|c| c.trim().to_ascii_uppercase()).filter(|c| !c.is_empty())),
            "--range" => match val().and_then(|r| r.split_once('-').and_then(|(a, b)| Some((a.trim().parse().ok()?, b.trim().parse().ok()?)))) { Some(r) => range = Some(r), None => return fail_msg(vc_core::msg!("cli-convert-range-expected")) },
            "--suggest-hex" => suggest = true, "--apply" => apply = val(),
            "-o" => output = val(), "--target" => target = val().unwrap_or(target.clone()), "--dialect" => dialect_id = val(), "--data-dir" => data_dir = val(), "--memory" => memory = val(),
            "--tokenize" => mode = "tokenize", "--detokenize" => mode = "detokenize", "--structured" => structured = true, "--pack" => pack = true, "--plain" => plain = true,
            "--map" => map = val(), "--labels" => labels = val(), "-I" => dirs.extend(val().map(PathBuf::from)),
            "--start" => start = val().and_then(|v| v.parse().ok()).unwrap_or(10), "--step" => step = val().and_then(|v| v.parse().ok()).unwrap_or(10),
            "-D" => match val().and_then(|d| d.split_once('=').and_then(|(n, v)| vc_target::parse_num(v).map(|v| (n.to_string(), v)))) { Some(d) => defs.push(d), None => return fail(vc_i18n::t!("cli-define-expected")) },
            s if !s.starts_with('-') => input = Some(s.to_string()),
            s => { return help::unknown_option("basic", s); }
        }
    }
    let Some(input) = input else { return help::usage_error("basic") };
    if mode == "compile" && input.ends_with(".sbas") { return fail_msg(vc_core::msg!("cli-basic-sbas-disabled")); }
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    if target.is_empty() { target = vc_target::default_target_or_c64(Some(&data)); }
    let mut tgt = match vc_target::load_target(&data, &target) { Ok(t) => t, Err(e) => return fail_target(e) };
    if let Err(e) = tgt.apply_memory(memory.as_deref()) { return fail(e); }
    let mut did = dialect_id.or(tgt.basic_dialect.clone()).unwrap_or_else(|| "cbm_v2".into());
    // `#dialect <id>` in der Datei überschreibt die Vorgabe (Projekt, Zielsystem, --dialect)
    if mode != "detokenize" && !suggest {
        if let Ok(text) = vc_core::textfile::read_source(&input) {
            match vc_basic::dialects::resolve(&text, &input, &did, Some(&tgt.id), &data) {
                Ok(r) => { for w in &r.warnings { print_diag(w); } did = r.id; }
                Err(es) => { for e in &es { print_diag(e); } return ExitCode::FAILURE; }
            }
        }
    }
    if suggest { return basic_suggest_hex(&input, output, &data, &did, &target, apply.as_deref()); }
    if let Some(kind) = convert { return basic_convert(kind, &input, output, write_back, preview, &data, &did, &tgt.id, (start, step), range); }
    // Natives BASIC (Endung .bas/.lst oder --plain): Zeilennummern und/oder @label, keine strukturierten Erweiterungen
    let plain = plain || (mode == "compile" && (input.ends_with(".bas") || input.ends_with(".lst") || input.ends_with(".tur")));
    if plain && mode == "compile" { return basic_plain(&input, output, &data, &did, &tgt, (start, step), vc_basic::check::CheckOptions { disabled, off: no_check }, check_only, &defs, compact); }
    if vc_basic::dialect_family(&data, &did).as_deref() == Ok("atari") {
        return basic_atari(mode, &input, output, &data, &did, (start, step), dirs, defs);
    }
    let d = match vc_basic::Dialect::load(&data, &did) { Ok(d) => d, Err(e) => return fail(e) };
    let load = tgt.basic_load_addr.unwrap_or(d.load_addr);
    let write = |path: Option<String>, bytes: &[u8], default_ext: &str| -> ExitCode {
        let p = path.unwrap_or_else(|| Path::new(&input).with_extension(default_ext).to_string_lossy().into_owned());
        match std::fs::write(&p, bytes) { Ok(()) => { println!("{p}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{p}: {e}")) }
    };
    match mode {
        "detokenize" => {
            let data = match std::fs::read(&input) { Ok(b) => b, Err(e) => return fail(format!("{input}: {e}")) };
            // Atari-BASIC-Dateien (SAVE-Format) beginnen mit zwei Nullbytes und gehören zu `--target atari8`; ohne Angabe gilt das CBM-PRG-Format
            let parsed = match vc_basic::parse_prg(&data) { Ok(p) => p, Err(e) => return fail_msg(if data.starts_with(&[0, 0]) { vc_core::msg!("cli-basic-looks-like-atari", input = &input, e = &e) } else { vc_core::msg!("cli-file-error", input = &input, e = &e) }) };
            let text = if structured { vc_basic::to_structured_text(&parsed, &d) } else { vc_basic::to_classic_text(&parsed, &d) };
            match output { Some(o) => write(Some(o), text.as_bytes(), "txt"), None => { print!("{text}"); ExitCode::SUCCESS } }
        }
        "tokenize" => {
            let src = match vc_core::textfile::read_source(&input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
            for w in vc_basic::numbers::classic_warnings(&src, &vc_basic::numbers::Lexicon::Cbm(&d), &input) { print_diag(&w); }
            match vc_basic::tokenize_text(&src, &d, &input) {
                Ok(lines) => write(output, &vc_basic::build_prg(&lines, load), "prg"),
                Err(ds) => { for x in &ds { print_diag(x); } ExitCode::FAILURE }
            }
        }
        _ => {
            let src = match vc_core::textfile::read_source(&input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
            let mut o = vc_basic::Options::new(input.clone());
            (o.start, o.step, o.pack, o.load_addr, o.include_dirs) = (start, step, pack, Some(load), dirs);
            o.consts = defs;
            o.system = Some(target.clone());
            if let Some(l) = labels { match label_consts(&l, "asm") { Ok(c) => o.consts.extend(c), Err(e) => return fail(e) } }
            let root = o.include_dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."));
            match vc_ide::blob::compile_with_blobs(&src, &d, &o, &root, &data, &tgt) {
                Ok(built) => {
                    let c = built.compiled; let full_prg = built.prg;
                    for b in &built.blobs { println!("{}", vc_i18n::t!("cli-basic-machine-code", file = &b.file, addr = format!("{:04X}", b.addr), size = &b.size)); }
                    for w in &c.warnings { print_diag(w); }
                    if let Some(m) = map {
                        let mut s = String::from("# Quellname -> BASIC-Name\n");
                        for (a, b) in &c.vars { s.push_str(&format!("{a} -> {b}\n")); }
                        for (n, l) in &c.labels { s.push_str(&format!("{}\n", vc_i18n::t!("cli-label-line", n = &n, l = &l))); }
                        let _ = std::fs::write(m, s);
                    }
                    write(output, &full_prg, "prg")
                }
                Err(ds) => { for x in &ds { print_diag(x); } ExitCode::FAILURE }
            }
        }
    }
}

/// Atari BASIC: `.sbas` (strukturiert) oder `.lst` (nummeriert) → tokenisierte `.BAS`-Datei (SAVE-Format); `--detokenize` → Listing.
fn basic_atari(mode: &str, input: &str, output: Option<String>, data: &Path, did: &str, (start, step): (u32, u32), dirs: Vec<PathBuf>, defs: Vec<(String, i64)>) -> ExitCode {
    let d = match vc_basic::atari::AtariDialect::load(data, did) { Ok(d) => d, Err(e) => return fail(e) };
    let write = |bytes: &[u8], ext: &str| -> ExitCode {
        let p = output.clone().unwrap_or_else(|| Path::new(input).with_extension(ext).to_string_lossy().into_owned());
        match std::fs::write(&p, bytes) { Ok(()) => { println!("{p}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{p}: {e}")) }
    };
    match mode {
        "detokenize" => {
            let bytes = match std::fs::read(input) { Ok(b) => b, Err(e) => return fail(format!("{input}: {e}")) };
            match vc_basic::atari::AtariProgram::parse_save(&bytes, &d) {
                Ok(p) => { let text = p.listing(&d); match &output { Some(o) => match std::fs::write(o, text) { Ok(()) => ExitCode::SUCCESS, Err(e) => fail(e) }, None => { print!("{text}"); ExitCode::SUCCESS } } }
                Err(e) => fail(format!("{input}: {e}")),
            }
        }
        "tokenize" => {
            let src = match vc_core::textfile::read_source(input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
            match vc_basic::atari::tokenize_text(&src, &d, input) {
                Ok(p) => write(&p.save_file(&d), "bas"),
                Err(ds) => { for x in &ds { print_diag(x); } ExitCode::FAILURE }
            }
        }
        _ => {
            let src = match vc_core::textfile::read_source(input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
            let mut o = vc_basic::AtariOptions::new(input);
            (o.start, o.step, o.include_dirs, o.consts) = (start, step, dirs, defs);
            let tgt = match vc_target::load_target(data, "atari8") { Ok(t) => t, Err(e) => return fail_target(e) };
            let root = o.include_dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."));
            match vc_ide::blob::compile_atari_with_blobs(&src, &d, &o, &root, data, &tgt) {
                Ok(b) => { for i in &b.blobs { println!("{}", vc_i18n::t!("cli-basic-machine-code-data", file = &i.file, addr = format!("{:04X}", i.addr), size = &i.size)); } write(&b.compiled.file, "bas") }
                Err(ds) => { for x in &ds { print_diag(x); } ExitCode::FAILURE }
            }
        }
    }
}

fn disk(args: &[String]) -> ExitCode {
    let Some(sub) = args.first().map(String::as_str) else { return help::usage_error("disk") };
    let (mut positional, mut output, mut boot, mut adds, mut data_dir) = (vec![], None, None, vec![], None);
    let (mut base, mut puts, mut deletes, mut force) = (None::<String>, Vec::<String>::new(), Vec::<String>::new(), false);
    let (mut dname, mut did) = ("VINTAGECODER".to_string(), "VC".to_string());
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" => output = it.next().cloned(), "--boot" => boot = it.next().cloned(), "--add" => adds.extend(it.next().cloned()), "--data-dir" => data_dir = it.next().cloned(),
            "--name" => dname = it.next().cloned().unwrap_or(dname), "--id" => did = it.next().cloned().unwrap_or(did),
            "--keep" => {}
            "--base" => base = it.next().cloned(), "--put" => puts.extend(it.next().cloned()), "--delete" => deletes.extend(it.next().cloned()), "--force" => force = true,
            s if !s.starts_with('-') => positional.push(s.to_string()),
            s => { return help::unknown_option("disk", s); }
        }
    }
    let read_atr = |p: &str| -> Result<vc_disk::Atr, String> { vc_disk::Atr::from_bytes(&std::fs::read(p).map_err(|e| format!("{p}: {e}"))?) };
    match sub {
        "atr" if base.is_some() => {
            // bestehendes ATR bearbeiten (Quelle bleibt unverändert): Dateien löschen, ersetzen oder anlegen
            let (Some(out), Some(base)) = (output, base) else { return help::usage_error("disk") };
            if boot.is_some() { return fail_msg(vc_core::msg!("turbo-cli-base-boot-konflikt")); }
            if let (Ok(a), Ok(b)) = (std::fs::canonicalize(&base), std::fs::canonicalize(&out)) { if a == b { return fail_msg(vc_core::msg!("turbo-cli-base-quelle-ist-ziel")); } }
            let mut a = match read_atr(&base) { Ok(a) => a, Err(e) => return fail(e) };
            for n in &deletes { if let Err(e) = a.dos2_remove(&n.to_ascii_uppercase(), force) { return fail(e); } }
            let split = |spec: &String| -> (String, String) { match spec.split_once(':') { Some((p, n)) => (p.to_string(), n.to_string()), None => (spec.clone(), Path::new(spec).file_name().map(|n| n.to_string_lossy().to_ascii_uppercase()).unwrap_or_default()) } };
            for spec in &puts {
                let (path, name) = split(spec);
                let bytes = match std::fs::read(&path) { Ok(b) => b, Err(e) => return fail(format!("{path}: {e}")) };
                if let Err(e) = a.dos2_put(&name.to_ascii_uppercase(), &bytes) { return fail(e); }
            }
            for spec in &adds {
                let (path, name) = split(spec);
                let bytes = match std::fs::read(&path) { Ok(b) => b, Err(e) => return fail(format!("{path}: {e}")) };
                if let Err(e) = a.dos2_add(&name.to_ascii_uppercase(), &bytes) { return fail(e); }
            }
            let problems = a.dos2_check_split().0;
            for m in &vc_disk::summarize_problems(&problems, 10) { err_msg(vc_core::msg!("cli-disk-problem", m = &m)); }
            if !problems.is_empty() { return ExitCode::FAILURE; }
            let bytes = a.to_bytes();
            match std::fs::write(&out, &bytes) { Ok(()) => { println!("{}", vc_i18n::t!("turbo-cli-disk-bearbeitet", out = &out, bytes = bytes.len(), free = a.free_sectors())); ExitCode::SUCCESS } Err(e) => fail(format!("{out}: {e}")) }
        }
        "atr" if !puts.is_empty() || !deletes.is_empty() || force => fail_msg(vc_core::msg!("turbo-cli-bearbeiten-braucht-base")),
        "atr" => {
            let Some(out) = output else { return help::usage_error("disk") };
            let atr = if let Some(x) = boot {
                if !adds.is_empty() { return fail_msg(vc_core::msg!("cli-disk-boot-add-conflict")); }
                let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
                let xex = match std::fs::read(&x) { Ok(b) => b, Err(e) => return fail(format!("{x}: {e}")) };
                match vc_disk::boot_disk(&xex, &data) { Ok(a) => a, Err(e) => return fail(e) }
            } else {
                let mut a = vc_disk::Atr::new_dos2();
                for spec in &adds {
                    let (path, name) = match spec.split_once(':') { Some((p, n)) => (p.to_string(), n.to_string()), None => (spec.clone(), Path::new(spec).file_name().map(|n| n.to_string_lossy().to_ascii_uppercase()).unwrap_or_default()) };
                    let bytes = match std::fs::read(&path) { Ok(b) => b, Err(e) => return fail(format!("{path}: {e}")) };
                    if let Err(e) = a.dos2_add(&name, &bytes) { return fail(e); }
                }
                a
            };
            match std::fs::write(&out, atr.to_bytes()) { Ok(()) => { println!("{out}: {} Bytes", atr.to_bytes().len()); ExitCode::SUCCESS } Err(e) => fail(format!("{out}: {e}")) }
        }
        "d64" => {
            let Some(out) = output else { return help::usage_error("disk") };
            let mut d = match vc_disk::D64::blank(&dname, &did) { Ok(d) => d, Err(e) => return fail(e) };
            for spec in &adds {
                let (path, name) = match spec.split_once(':') { Some((p, n)) => (p.to_string(), n.to_string()), None => (spec.clone(), Path::new(spec).file_stem().map(|n| n.to_string_lossy().to_ascii_uppercase()).unwrap_or_default()) };
                let bytes = match std::fs::read(&path) { Ok(b) => b, Err(e) => return fail(format!("{path}: {e}")) };
                if let Err(e) = d.add_prg(&name, &bytes) { return fail(e); }
            }
            match std::fs::write(&out, d.to_bytes()) { Ok(()) => { println!("{}", vc_i18n::t!("cli-disk-written", out = &out, bytes = d.to_bytes().len(), free = d.free_blocks())); ExitCode::SUCCESS } Err(e) => fail_msg(vc_core::msg!("cli-file-error-out", out = &out, e = &e)) }
        }
        "d71" | "d81" => {
            let Some(out) = output else { return help::usage_error("disk") };
            let kind = if sub == "d71" { vc_disk::Kind::D71 } else { vc_disk::Kind::D81 };
            let mut d = match vc_disk::Disk::blank(kind, &dname, &did) { Ok(d) => d, Err(e) => return fail(e) };
            for spec in &adds {
                let (path, name) = match spec.split_once(':') { Some((p, n)) => (p.to_string(), n.to_string()), None => (spec.clone(), Path::new(spec).file_stem().map(|n| n.to_string_lossy().to_ascii_uppercase()).unwrap_or_default()) };
                let bytes = match std::fs::read(&path) { Ok(b) => b, Err(e) => return fail(format!("{path}: {e}")) };
                if let Err(e) = d.add_prg(&name, &bytes) { return fail(e); }
            }
            match std::fs::write(&out, d.to_bytes()) { Ok(()) => { println!("{}", vc_i18n::t!("cli-disk-written", out = &out, bytes = d.to_bytes().len(), free = d.free_blocks())); ExitCode::SUCCESS } Err(e) => fail_msg(vc_core::msg!("cli-file-error-out", out = &out, e = &e)) }
        }
        "list" if positional.first().map_or(false, |p| vc_disk::Kind::from_extension(p).is_some()) => {
            let p = &positional[0];
            match std::fs::read(p).map_err(|e| format!("{p}: {e}")).and_then(|b| vc_disk::Disk::from_file_bytes(p, &b)) {
                Ok(d) => {
                    println!("0 \"{}\"", d.disk_name());
                    for e in d.entries() { println!("{:<4} \"{}\" {}", e.blocks, e.name, e.kind); }
                    println!("{}", vc_i18n::t!("cli-disk-blocks-free", blocks = d.free_blocks()));
                    let problems = d.check();
                    for m in &vc_disk::summarize_problems(&problems, 10) { err_msg(vc_core::msg!("cli-disk-problem", m = &m)); }
                    if problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
                }
                Err(e) => fail(e),
            }
        }
        "extract" if positional.first().map_or(false, |p| vc_disk::Kind::from_extension(p).is_some()) => {
            let (p, name) = (&positional[0], positional.get(1).cloned().unwrap_or_default());
            match std::fs::read(p).map_err(|e| format!("{p}: {e}")).and_then(|b| vc_disk::Disk::from_file_bytes(p, &b)).and_then(|d| d.read(&name)) {
                Ok(bytes) => { let o = output.unwrap_or_else(|| format!("{}.prg", name.to_ascii_lowercase())); match std::fs::write(&o, &bytes) { Ok(()) => { println!("{o}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{o}: {e}")) } }
                Err(e) => fail(e),
            }
        }
        "list" if positional.first().map_or(false, |p| p.to_ascii_lowercase().ends_with(".d64")) => {
            let p = &positional[0];
            match std::fs::read(p).map_err(|e| format!("{p}: {e}")).and_then(|b| vc_disk::D64::from_bytes(&b)) {
                Ok(d) => {
                    println!("0 \"{}\"", d.disk_name());
                    for e in d.entries() { println!("{:<4} \"{}\" {}", e.blocks, e.name, e.kind); }
                    println!("{}", vc_i18n::t!("cli-disk-blocks-free", blocks = d.free_blocks()));
                    let problems = d.check();
                    for m in &vc_disk::summarize_problems(&problems, 10) { err_msg(vc_core::msg!("cli-disk-problem", m = &m)); }
                    if problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
                }
                Err(e) => fail(e),
            }
        }
        "extract" if positional.first().map_or(false, |p| p.to_ascii_lowercase().ends_with(".d64")) => {
            let (p, name) = (&positional[0], positional.get(1).cloned().unwrap_or_default());
            match std::fs::read(p).map_err(|e| format!("{p}: {e}")).and_then(|b| vc_disk::D64::from_bytes(&b)).and_then(|d| d.read(&name)) {
                Ok(bytes) => { let o = output.unwrap_or_else(|| format!("{}.prg", name.to_ascii_lowercase())); match std::fs::write(&o, &bytes) { Ok(()) => { println!("{o}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{o}: {e}")) } }
                Err(e) => fail(e),
            }
        }
        "list" => {
            let Some(p) = positional.first() else { return help::usage_error("disk") };
            match read_atr(p) {
                Ok(a) if !a.has_dos2() => {
                    // bootbare Diskette ohne DOS-2-Verzeichnis (z. B. `vcc disk atr --boot x.xex`): eine Meldung statt Hunderter Einzelzeilen
                    let used = (1..=a.sector_count())   /* alle Sektoren des Abbilds */.filter(|&n| a.sector(n).iter().any(|&b| b != 0)).count();
                    println!("{}", vc_i18n::t!("cli-disk-boot-no-filesystem", used = &used));
                    ExitCode::SUCCESS
                }
                Ok(a) => {
                    for e in a.dos2_entries() { println!("{}", vc_i18n::t!("cli-disk-entry", name = format!("{:12}", e.name), sectors = format!("{:4}", e.sectors), start = &e.start, locked = if e.locked { "  *" } else { "" })); }
                    println!("{}", vc_i18n::t!("cli-disk-free-sectors", count = a.free_sectors()));
                    // Fehler (Exit-Code 1) und Hinweise auf Altlasten (z. B. nie abgeschlossene Einträge, Exit-Code 0)
                    let (problems, hints) = a.dos2_check_split();
                    for m in &vc_disk::summarize_problems(&problems, 10) { err_msg(vc_core::msg!("cli-disk-problem", m = &m)); }
                    for m in &vc_disk::summarize_problems(&hints, 10) { note(vc_core::msg!("cli-warning", text = &m)); }
                    if problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
                }
                Err(e) => fail(e),
            }
        }
        "extract" => {
            let (Some(p), Some(name)) = (positional.first(), positional.get(1)) else { return help::usage_error("disk") };
            match read_atr(p).and_then(|a| a.dos2_read(name)) {
                Ok(bytes) => { let o = output.unwrap_or_else(|| name.to_ascii_lowercase()); match std::fs::write(&o, &bytes) { Ok(()) => { println!("{o}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{o}: {e}")) } }
                Err(e) => fail(e),
            }
        }
        _ => help::usage_error("disk"),
    }
}

fn media(args: &[String]) -> ExitCode {
    let Some(sub) = args.first().map(String::as_str) else { return help::usage_error("media") };
    let (mut positional, mut profile, mut dir, mut format, mut label, mut address, mut data_dir) = (vec![], None, String::new(), "asm".to_string(), None, None, None);
    let (mut frame, mut system, mut auto, mut name, mut offset, mut invert) = (0usize, "pal".to_string(), true, None::<String>, 0usize, false);
    let (mut blank, mut copy) = (false, false);
    let (mut song_arg, mut seconds_arg) = (None::<u16>, None::<u32>);
    let (mut count_arg, mut mode_arg) = (0usize, None::<String>);
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--profile" => profile = it.next().cloned(), "--dir" => dir = it.next().cloned().unwrap_or(dir), "--format" => format = it.next().cloned().unwrap_or(format),
            "--label" => label = it.next().cloned(), "--address" => address = it.next().and_then(|v| vc_target::parse_num(v)).map(|v| v as u16), "--data-dir" => data_dir = it.next().cloned(),
            "--count" => count_arg = it.next().and_then(|v| v.parse().ok()).unwrap_or(0), "--mode" => mode_arg = it.next().cloned(),
            "--song" => song_arg = it.next().and_then(|v| v.parse().ok()), "--seconds" => seconds_arg = it.next().and_then(|v| v.parse().ok()),
            "--invert" => invert = true, "--blank" => blank = true, "--copy" => copy = true, "--name" => name = it.next().cloned(), "--offset" => offset = it.next().and_then(|v| vc_target::parse_num(v)).map(|v| v as usize).unwrap_or(0), "--frame" => frame = it.next().and_then(|v| v.parse().ok()).unwrap_or(0), "--system" => system = it.next().cloned().unwrap_or(system), "--keep-colors" => auto = false,
            s if !s.starts_with('-') => positional.push(s.to_string()),
            s => { return help::unknown_option("media", s); }
        }
    }
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    let root = std::env::current_dir().unwrap_or_default();
    match sub {
        "new" => {
            let (Some(name), Some(profile)) = (positional.first(), profile) else { return help::usage_error("media") };
            match vc_media::project::create_with(&root, &data, name, &profile, &dir, blank) { Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); ExitCode::SUCCESS } Err(e) => fail(e) }
        }
        "export" => {
            let Some(path) = positional.first() else { return help::usage_error("media") };
            match vc_media::project::export(&root, &data, path, &format, label.as_deref(), address, 1000, 10) {
                Ok(v) => { for f in v["files"].as_array().into_iter().flatten() { println!("{} ({} Bytes)", f["path"].as_str().unwrap_or(""), f["size"]); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "migrate-tiles" => {
            match vc_media::project::migrate_tiles(&root, &data, !args.iter().any(|a| a == "--keep")) { Ok(r) => { for l in r { println!("{l}"); } ExitCode::SUCCESS } Err(e) => fail(e) }
        }
        "import-sheet" => {
            let (Some(file), Some(name), Some(profile)) = (positional.first(), name.clone(), profile.clone()) else { return help::usage_error("media") };
            match vc_media::project::import_sheet(&root, &data, std::path::Path::new(file), &profile, invert, &name, &dir) { Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); ExitCode::SUCCESS } Err(e) => fail(e) }
        }
        "export-sheet" => {
            let (Some(path), Some(out)) = (positional.first(), positional.get(1)) else { return help::usage_error("media") };
            match vc_media::project::export_sheet(&root, &data, path, std::path::Path::new(out)) { Ok(()) => { println!("{out}"); ExitCode::SUCCESS } Err(e) => fail(e) }
        }
        "import-rom" => {
            let Some(file) = positional.first() else { return help::usage_error("media") };
            let len = std::fs::metadata(file).map(|m| m.len() as usize).unwrap_or(0);
            let rec = vc_media::project::recognize_rom(std::path::Path::new(file));
            let (Some(name), Some(profile)) = (name.clone(), profile.clone()) else {
                match &rec.label { Some(l) => err_msg(vc_core::msg!("cli-disk-detected", file = &file, len = &len, l = &l, status = rec.status.label_l())), None => err_msg(vc_core::msg!("cli-disk-not-detected", file = &file, len = &len)) }
                for (d, p, o) in &rec.sets { eprintln!("  --profile {p} --offset {o:#x}   {d}"); }
                return help::usage_error("media");
            };
            match vc_media::project::import_rom(&root, &data, std::path::Path::new(file), &profile, offset, &name, &dir) { Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); ExitCode::SUCCESS } Err(e) => fail(e) }
        }
        "import-rmt" => {
            let (Some(path), Some(name)) = (positional.first(), name) else { return help::usage_error("media") };
            match vc_media::project::import_rmt(&root, &data, path, &name, &dir) {
                Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); for w in v["warnings"].as_array().into_iter().flatten() { note(vc_core::msg!("cli-note", text = w.as_str().unwrap_or(""))); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "std-charset" => {
            // Zeichensatz aus einem ROM des Anwenders ins Projekt legen: `vcc media std-charset rom:c64/chargen#upper --profile c64.char.hires` (Verweis, Format 2);
            // --copy bettet die Glyphen ein (Format 1, nach graphics/standard/). Ohne Verweis: Liste der Quellen mit Status.
            let Some(reference) = positional.first() else {
                for c in vc_media::project::std_charset_list(&data) { println!("{}\t{}\t{}\t{}\t{}", c.reference, c.profile, c.system, c.status.label_l(), c.label); }
                return ExitCode::SUCCESS;
            };
            let Some(pid) = profile.clone().or_else(|| vc_media::romref::RomRef::parse(reference).ok().and_then(|r| vc_media::project::std_charset_list(&data).into_iter().find(|c| c.reference == r.to_text()).map(|c| c.profile))) else { return fail_msg(vc_core::msg!("cli-profile-missing")) };
            let target = if !copy { if dir.is_empty() || dir == "assets" { "graphics".to_string() } else { dir.clone() } } else { "graphics/standard".to_string() };
            match vc_media::project::add_std_charset(&root, reference, &pid, &target, copy) {
                Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); if let Some(w) = v["warning"].as_str() { note(vc_core::msg!("cli-note-w", text = &w)); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "import-sprite" => {
            // Sprites aus dem SPD-Format (.spd), Rohdaten (64-Byte-Blöcke) oder einem Sprite-Asset eines anderen Systems; --profile = Zielprofil (Standard c64.sprite.hires)
            let (Some(path), Some(name)) = (positional.first(), name) else { return help::usage_error("media") };
            let to = profile.unwrap_or_else(|| "c64.sprite.hires".into());
            let mc = match mode_arg.as_deref() { Some("multicolor") | Some("mc") => Some(true), Some("hires") => Some(false), None => None, Some(m) => return fail_msg(vc_core::msg!("cli-mode-invalid", m = &m)) };
            let opts = vc_media::spriteimport::SprOpts { offset, count: count_arg, mc, frame };
            match vc_media::project::import_sprite(&root, &data, std::path::Path::new(path), &to, &name, &dir, &opts) {
                Ok(v) => { println!("{}", vc_i18n::t!("cli-media-frames", path = v["path"].as_str().unwrap_or(""), count = v["count"].to_string())); for w in v["warnings"].as_array().into_iter().flatten() { note(vc_core::msg!("cli-note", text = w.as_str().unwrap_or(""))); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "import-song" => {
            // Song aus beliebiger Datei (GoatTracker-*.sng oder *.asset.json eines anderen Systems) ins Projekt; --profile = Zielprofil (Standard c64.sid.song)
            let (Some(path), Some(name)) = (positional.first(), name) else { return help::usage_error("media") };
            let to = profile.unwrap_or_else(|| "c64.sid.song".into());
            let opts = vc_media::sid::SidOpts { song: song_arg.unwrap_or(0), seconds: seconds_arg.unwrap_or(30) };
            match vc_media::project::import_song_with(&root, &data, std::path::Path::new(path), &to, &name, &dir, &opts) {
                Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); for w in v["warnings"].as_array().into_iter().flatten() { note(vc_core::msg!("cli-note", text = w.as_str().unwrap_or(""))); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "convert" => {
            let (Some(path), Some(profile), Some(name)) = (positional.first(), profile, name) else { return help::usage_error("media") };
            match vc_media::project::convert_song(&root, &data, path, &profile, &name, &dir) {
                Ok(v) => { println!("{}", v["path"].as_str().unwrap_or("")); for w in v["warnings"].as_array().into_iter().flatten() { note(vc_core::msg!("cli-note", text = w.as_str().unwrap_or(""))); } ExitCode::SUCCESS }
                Err(e) => fail(e),
            }
        }
        "import" => {
            let (Some(path), Some(png)) = (positional.first(), positional.get(1)) else { return help::usage_error("media") };
            let bytes = match std::fs::read(png) { Ok(b) => b, Err(e) => return fail(format!("{png}: {e}")) };
            match vc_media::project::import_png(&root, &data, path, &bytes, frame, auto, &system) {
                Ok(v) => {
                    let r = &v["report"]; println!("{}", vc_i18n::t!("cli-media-frame-imported", frame = r["frame"].to_string(), colors = r["imageColors"].to_string(), allowed = r["allowed"].to_string(), conflict = if r["conflict"] == true { format!(" {}", vc_i18n::t!("cli-media-conflict-suffix")) } else { String::new() }));
                    for l in r["reducedColors"].as_array().into_iter().flatten() { println!("  {}", vc_i18n::t!("cli-media-color-replaced", color = l["color"].to_string(), pixels = l["pixels"].to_string())); }
                    ExitCode::SUCCESS
                }
                Err(e) => fail(e),
            }
        }
        _ => help::usage_error("media"),
    }
}

/// `vcc add`: Dateien und Ordner wie per Drag and Drop ins Projekt übernehmen (Art erkennen, Kategorie und Ordner aus `layout`/`rules`,
/// Eintrag in `assets`). Ordner werden rekursiv übernommen; ihre Struktur bleibt erhalten (`--flat`: jede Datei in den Ordner ihrer Kategorie).
fn add_cmd(args: &[String]) -> ExitCode {
    use vc_ide::importfiles::{self, Category, Conflict, Plan};
    let (mut paths, mut into, mut cat, mut dir, mut desc, mut conflict, mut flat, mut dry) = (vec![], None::<String>, None::<String>, None::<String>, String::new(), Conflict::Rename, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--into" => into = it.next().cloned(), "--category" => cat = it.next().cloned(), "--dir" => dir = it.next().cloned(), "--description" => desc = it.next().cloned().unwrap_or_default(),
            "--conflict" => conflict = match it.next().map(String::as_str) { Some("rename") => Conflict::Rename, Some("replace") => Conflict::Replace, Some("skip") => Conflict::Skip, _ => return fail_msg(vc_core::msg!("cli-conflict-invalid")) },
            "--flat" => flat = true, "--dry-run" => dry = true,
            s if !s.starts_with('-') => paths.push(PathBuf::from(s)),
            s => { return help::unknown_option("add", s); }
        }
    }
    if paths.is_empty() { return help::usage_error("add"); }
    let category = match cat.as_deref() { None => None, Some(c) => match Category::from_id(c).filter(|c| *c != Category::Studio) { Some(c) => Some(c), None => return fail(vc_i18n::t!("cli-category-unknown", choices = Category::CHOICES.map(|c| c.id()).join(", "))) } };
    if let Some(d) = &dir { if !importfiles::valid_dir(d) { return fail_msg(vc_core::msg!("cli-dir-invalid")); } }
    let p = PathBuf::from(into.unwrap_or_else(|| ".".into()));
    let (root, file) = if p.is_dir() { match vc_ide::projfile::find(&p) { Some(f) => (p.clone(), f), None => return fail_msg(vc_core::msg!("cli-no-project-file", path = p.display().to_string())) } } else { (p.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")), p.clone()) };
    let mut pf = match vc_ide::projfile::ProjectFile::load(&file) { Ok(x) => x, Err(e) => return fail(e) };
    // Pläne: Datei → eigener Plan; Ordner → alle Dateien mit Unterordner
    let mut plans: Vec<Plan> = vec![];
    for src in &paths {
        let mut one = |f: &Path, sub: Option<String>| {
            let mut plan = Plan::suggest_with(f, &pf.layout, &pf.rules);
            if let Some(c) = category { plan.category = c; if dir.is_none() { plan.dir = importfiles::dir_for(&pf.layout, c); } }
            if let Some(d) = &dir { plan.dir = d.clone(); }
            if let (Some(s), false) = (sub, flat) { plan.dir = importfiles::join_rel(dir.as_deref().unwrap_or(""), &s); }
            plan.description = desc.clone(); plan.conflict = conflict;
            plans.push(plan);
        };
        if src.is_dir() {
            let (v, over) = importfiles::walk_folder(src, 5000);
            if over > 0 { note(vc_core::msg!("cli-add-over-limit", over = &over, dir = src.display().to_string())); }
            for (f, sub) in v { one(&f, Some(sub)); }
        } else if src.is_file() { one(src, None); } else { return fail_msg(vc_core::msg!("cli-not-found", path = src.display().to_string())); }
    }
    let (mut ok, mut n) = (true, 0);
    for plan in &plans {
        if dry { println!("{:<9} {} -> {}  ({}, {})", format!("{:?}", importfiles::target_state(&root, plan)), plan.src.display(), importfiles::join_rel(&plan.dir, &plan.name), plan.kind().label(), plan.category.label()); continue; }
        match importfiles::import(&root, Some(&mut pf), plan) {
            Ok(o) => { n += 1; println!("{} ({})", match &o { importfiles::Outcome::Copied(r) => vc_i18n::t!("cli-add-copied", r = &r), importfiles::Outcome::Replaced(r) => vc_i18n::t!("cli-add-replaced", r = &r), importfiles::Outcome::Existing(r) => vc_i18n::t!("cli-add-existing", r = &r), importfiles::Outcome::Skipped(r) => vc_i18n::t!("cli-add-skipped", r = &r) }, plan.category.label()); }
            Err(e) => { ok = false; eprintln!("{e}"); }
        }
    }
    if dry { println!("{}", vc_i18n::t!("cli-dry-run")); return ExitCode::SUCCESS; }
    if let Err(e) = pf.save(&file) { return fail(e); }
    println!("{}", vc_i18n::t!("cli-files-taken", n = &n));
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// `vcc organize`: Dateien nach `layout`, `rules` und `assets` der Projektdatei umsortieren. Pfade in `.include`/`.incbin` und die Projektdatei
/// werden nachgeführt, vorher entsteht eine Sicherung in `.vcbackup/`; mit Prüfbau (vorher/nachher gleiche Bytes) außer mit `--no-check`.
fn organize_cmd(args: &[String]) -> ExitCode {
    let (mut path, mut dry, mut subdirs, mut check, mut data_dir, mut yes, mut rewrite) = (None, false, false, true, None, false, true);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() { "--dry-run" => dry = true, "--subdirs" => subdirs = true, "--no-check" => check = false, "--yes" | "-y" => yes = true, "--no-rewrite" => rewrite = false, "--data-dir" => data_dir = it.next().cloned(), s if !s.starts_with('-') => path = Some(s.to_string()), s => { return help::unknown_option("organize", s); } }
    }
    let p = PathBuf::from(path.unwrap_or_else(|| ".".into()));
    let (root, file) = if p.is_dir() { match vc_ide::projfile::find(&p) { Some(f) => (p.clone(), f), None => return fail_msg(vc_core::msg!("cli-no-project-file", path = p.display().to_string())) } } else { (p.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")), p.clone()) };
    let mut pf = match vc_ide::projfile::ProjectFile::load(&file) { Ok(x) => x, Err(e) => return fail(e) };
    let moves = vc_ide::reorganize::plan(&root, &pf, &vc_ide::reorganize::Opts { subdirs });
    if moves.is_empty() { println!("{}", vc_i18n::t!("cli-organize-nothing")); return ExitCode::SUCCESS; }
    for m in &moves { println!("{} {} -> {}{}", if m.conflict { vc_i18n::t!("cli-organize-skipped-label") } else { format!("{} ", vc_i18n::t!("cli-organize-move-label")) }, m.from, m.to, if m.conflict { format!("  {}", vc_i18n::t!("cli-organize-target-taken")) } else { String::new() }); }
    let rw = vc_ide::reorganize::rewrites(&root, &moves.iter().filter(|m| !m.conflict).cloned().collect::<Vec<_>>());
    for (f, _, n) in &rw { println!("{}", vc_i18n::t!("cli-organize-adjust-paths", f = &f, n = &n)); }
    if dry { println!("{}", vc_i18n::t!("cli-dry-run")); return ExitCode::SUCCESS; }
    // Ohne --yes nachfragen (nur im Terminal); in Skripten muss --yes ausdrücklich dastehen
    if !yes {
        use std::io::{BufRead, IsTerminal, Write};
        if !std::io::stdin().is_terminal() { err_msg(vc_core::msg!("cli-organize-needs-yes")); return ExitCode::FAILURE; }
        let ask = |q: &str, default: bool| -> bool {
            print!("{q} [{}] ", if default { "J/n" } else { "j/N" }); let _ = std::io::stdout().flush();
            let mut l = String::new(); let _ = std::io::stdin().lock().read_line(&mut l);
            match l.trim().to_lowercase().as_str() { "j" | "ja" | "y" | "yes" => true, "n" | "nein" | "no" => false, _ => default }
        };
        if !ask(&vc_i18n::t!("cli-organize-confirm", count = moves.iter().filter(|m| !m.conflict).count()), false) { println!("{}", vc_i18n::t!("cli-cancelled")); return ExitCode::SUCCESS; }
        if !rw.is_empty() && rewrite { rewrite = ask(&format!("Pfade in {} Quelltext(en) anpassen (.include/.incbin)?", rw.len()), true); }
    }
    let data = if check { find_data_dir(data_dir) } else { None };
    if check && data.is_none() { note(vc_core::msg!("cli-organize-no-check-build")); }
    match vc_ide::reorganize::apply(&root, &file, &mut pf, &moves, data.as_deref(), rewrite, &|s| println!("{s}")) {
        Ok(r) => {
            println!("{}", vc_i18n::t!("cli-organize-done", count = r.moved.len(), backup = r.backup.display().to_string()));
            match r.check {
                vc_ide::reorganize::Check::Same(n) => { println!("{}", vc_i18n::t!("cli-organize-check-same", n = &n)); ExitCode::SUCCESS }
                vc_ide::reorganize::Check::Skipped(why) => { println!("{}", vc_i18n::t!("cli-organize-check-skipped", why = &why)); ExitCode::SUCCESS }
                vc_ide::reorganize::Check::Different(v) => { note(vc_core::msg!("cli-organize-check-different", files = v.join(", "))); note(vc_core::msg!("cli-organize-undo", backup = r.backup.display().to_string())); ExitCode::FAILURE }
            }
        }
        Err(e) => fail(e),
    }
}

/// `vcc new` – Projekt-Wizard: legt aus einer Vorlage Projektdatei und Quellen an.
fn new_project(args: &[String]) -> ExitCode {
    let (mut dir, mut name, mut targets, mut template, mut data_dir, mut list) = (None::<String>, None::<String>, String::new(), "asm-hello".to_string(), None, false);
    let mut variant: Option<String> = None; let mut structure: Option<String> = None; let mut charset = vc_media::wizard::Zeichensatz::Auto;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--charset" => match it.next().and_then(|v| vc_media::wizard::Zeichensatz::parse(v)) { Some(c) => charset = c, None => return fail_msg(vc_core::msg!("cli-charset-invalid")) },
            "--name" => name = it.next().cloned(), "--target" => targets = it.next().cloned().unwrap_or(targets), "--template" => template = it.next().cloned().unwrap_or(template),
            "--variant" => variant = it.next().cloned(), "--structure" => structure = it.next().cloned(),
            "--data-dir" => data_dir = it.next().cloned(), "--list" => list = true,
            s if !s.starts_with('-') => dir = Some(s.to_string()),
            s => { return help::unknown_option("new", s); }
        }
    }
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    if list {
        println!("{}", vc_i18n::t!("cli-structure-heading"));
        for s in vc_ide::structure::list_all(&data) { println!("  {:<12} {}", s.id, s.description); }
        match vc_media::wizard::templates(&data) {
            Ok(v) => {
                macro_rules! systems_of { ($t:expr) => { $t["systems"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).filter(|x| vc_target::target_available(&data, x)).collect::<Vec<_>>().join(",")).unwrap_or_default() }; }
                for t in v["templates"].as_array().into_iter().flatten() {
                    println!("{:<12} {} [{}]", t["id"].as_str().unwrap_or(""), t["description"].as_str().unwrap_or(""), systems_of!(t));
                    for va in t["variants"].as_array().into_iter().flatten() { println!("  --variant {:<8} {} [{}]", va["id"].as_str().unwrap_or(""), va["description"].as_str().unwrap_or(""), systems_of!(va)); }
                }
                return ExitCode::SUCCESS;
            }
            Err(e) => return fail(e),
        }
    }
    if targets.is_empty() { targets = vc_target::default_target_or_c64(Some(&data)); }
    let Some(dir) = dir else { return help::usage_error("new") };
    let path = PathBuf::from(&dir);
    let name = name.unwrap_or_else(|| path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
    let systems: Vec<String> = targets.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let created = match variant.as_deref() { Some(_) => vc_media::wizard::create_with(&data, &path, &name, &systems, &template, variant.as_deref()), None => vc_media::wizard::create(&data, &path, &name, &systems, &template) };
    // Projektstruktur: Ordner anlegen und layout in die Projektdatei eintragen
    let mut made: Vec<String> = vec![];
    if created.is_ok() {
        if let Some(id) = &structure {
            let Some(st) = vc_ide::structure::find(&vc_ide::structure::list_all(&data), id).cloned() else { return fail(vc_i18n::t!("cli-structure-unknown", id = id.as_str())) };
            let file = path.join(format!("{name}.vcprj"));
            let r = vc_ide::projfile::ProjectFile::load(&file).and_then(|mut pf| { let m = st.apply(&path, &mut pf, &systems, false)?; pf.save(&file)?; Ok(m) });
            match r { Ok(m) => made = m, Err(e) => return fail(e) }
        }
    }
    let zs = if created.is_ok() { match vc_media::wizard::zeichensatz_anlegen(&data, &path, &systems, charset) { Ok(f) => f, Err(e) => return fail(e) } } else { vec![] };
    match created {
        Ok(v) => { for d in &made { println!("{}/{}/", dir, d); } for f in v["files"].as_array().into_iter().flatten() { println!("{}/{}", dir, f.as_str().unwrap_or("")); } for f in &zs { println!("{}/{}", dir, f); } ExitCode::SUCCESS }
        Err(e) => fail(e),
    }
}

/// `vcc build`: alle Quellen eines Projekts bauen (Reihenfolge und Targets aus der `.vcprj`), danach die Diskette
/// `vcc test`: statische Prüfungen aus `*.vctest` (Stufe 1)
/// Pfad für die Ausgabe: immer mit `/`, auch unter Windows (dort mischt `join` sonst `/` und `\`)
fn show_path(p: &Path) -> String { vc_core::paths::slashes(&p.to_string_lossy()) }

fn test_cmd(args: &[String]) -> ExitCode {
    use vc_ide::tests::{find_tests, runner, run_file, Level, Options, Prepared, State, TestFile};
    let (mut path, mut strict, mut data_dir, mut backend, mut filter, mut prologue, mut poll) = (None, false, None, None::<String>, None::<String>, false, false);
    let (mut password, mut yes, mut junit) = (None::<String>, false, None::<String>);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() { "--strict" => strict = true, "--data-dir" => data_dir = it.next().cloned(), "--backend" => backend = it.next().cloned(), "--filter" => filter = it.next().cloned(), "--prologue" => prologue = true, "--poll" => poll = true, "--password" => password = it.next().cloned(), "--yes" => yes = true, "--junit" => junit = it.next().cloned(), s if !s.starts_with('-') => path = Some(s.to_string()), s => { return help::unknown_option("test", s); } }
    }
    // Anbindung nicht im Release gewählt: verständliche Meldung statt Startversuch
    if let Some(b) = backend.as_deref() {
        let r = if b.starts_with("u64") { vc_ide::emulators::require_device("ultimate64") } else if b.starts_with("m65") { vc_ide::emulators::require_device("mega65") } else { vc_ide::emulators::require(b.split('@').next().unwrap_or(b)) };
        if let Err(e) = r { return fail(format!("--backend {b}: {e}")); }
    }
    // `--backend vice@3.9`: eine bestimmte portabel installierte Version (`vcc emulator install vice@3.9`)
    if let Some(b) = backend.clone().filter(|b| b.contains('@') && !b.starts_with("u64") && !b.starts_with("m65")) {
        if let Err(e) = vc_ide::emulators::select(&b) { return fail(format!("--backend {b}: {e}")); }
        backend = Some(vc_ide::emulators::parse_spec(&b).0);
    }
    let p = PathBuf::from(path.unwrap_or_else(|| ".".into()));
    if !p.exists() { return fail_msg(vc_core::msg!("cli-not-found", path = show_path(&p))); }
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir-hint")) };
    let files = find_tests(&p);
    if files.is_empty() { println!("{}", vc_i18n::t!("cli-test-none-found", dir = show_path(&p))); return ExitCode::SUCCESS; }
    let hardware = match backend.as_deref() {
        Some(b) if b == "u64" || b == "m65" || b.starts_with("u64:") || b.starts_with("m65:") => match parse_hardware(b, password.as_deref()) { Ok(h) => Some(h), Err(e) => return fail(e) },
        Some(b) if !["vice", "atari800", "fujisan", "xemu"].contains(&b) => return fail_msg(vc_core::msg!("cli-test-backend-invalid", b = &b)),
        _ => None,
    };
    if let Some(h) = &hardware { if let Err(e) = confirm_hardware(&h.label(), yes) { return fail(e); } }
    let opts = Options { strict, dynamic: backend.is_some() };
    let ro = runner::RunOptions { strict, filter, force_prologue: prologue, ..Default::default() };
    let (mut passed, mut failed, mut hints, mut skipped, mut broken, mut timeouts) = (0, 0, 0, 0, 0, 0);
    let mut suites: Vec<vc_ide::tests::junit::SuiteResult> = vec![];
    for f in &files {
        let t0 = std::time::Instant::now();
        let tf = match TestFile::load(f) { Ok(t) => t, Err(e) => { println!("{}", vc_i18n::t!("cli-test-broken", error = vc_core::paths::slashes(&e))); broken += 1; suites.push(vc_ide::tests::junit::SuiteResult { name: show_path(f).trim_start_matches("./").to_string(), outcomes: vec![vc_ide::tests::Outcome::failed(vc_i18n::t!("cli-test-file"), Level::Error, e)], millis: 0 }); continue; } };
        let mut rep = run_file(&tf, &data, &opts);
        if backend.is_some() && !tf.tests.is_empty() {
            match Prepared::load(&tf, &data) {
                Ok(pre) if hardware.is_some() => {
                    let hw = hardware.clone().unwrap();
                    let dynamic = if tf.mode == vc_ide::tests::TestMode::Measure {
                        let mut m = vc_ide::tests::cycles::hardware_measurer(hw.clone(), tf.target.clone(), pre.program.clone(), vc_ide::streams::StreamOptions::from_settings(&vc_ide::settings::effective()));
                        vc_ide::tests::cycles::run_measure(&tf, &pre, &ro, &mut m)
                    } else if tf.mode == vc_ide::tests::TestMode::Mailbox {
                        let (target, hw2) = (tf.target.clone(), hw.clone());
                        vc_ide::tests::mailbox::run_mailbox(&tf, &pre, &ro, &mut |p: &Path| vc_ide::tests::suite::launch_hardware(&hw2, &target, p))
                    } else {
                        let why = vc_i18n::t!("cli-test-trap-unavailable");
                        tf.tests.iter().enumerate().map(|(i, t)| vc_ide::tests::Outcome::skipped(t.get("name").and_then(|n| n.as_str()).map(str::to_string).unwrap_or_else(|| vc_i18n::t!("cli-test-default-name", n = i + 1)), why.clone())).collect()
                    };
                    rep.outcomes.extend(dynamic);
                }
                Ok(_) if tf.mode == vc_ide::tests::TestMode::Measure => {
                    let why = vc_i18n::t!("cli-test-measure-needs-u64");
                    rep.outcomes.extend(tf.tests.iter().enumerate().map(|(i, t)| vc_ide::tests::Outcome::skipped(t.get("name").and_then(|n| n.as_str()).map(str::to_string).unwrap_or_else(|| vc_i18n::t!("cli-test-default-measure-name", n = i + 1)), why.clone())));
                }
                Ok(pre) => {
                    let prog = match tf.target.as_str() { "c64" => "x64sc", "c128" => "x128", "plus4" | "c16" => "xplus4", "vic20" => "xvic", _ => "" };
                    let which = backend.as_deref().unwrap_or("vice");
                    let mut launch = || -> Result<Box<dyn vc_emu::Backend>, String> {
                        match which {
                            "atari800" => {
                                if tf.target != "atari8" { return Err(vc_i18n::t!("cli-test-atari800-target", target = &tf.target)); }
                                let rp = vc_ide::roms::plan("atari800", &vc_ide::roms::store(None, None), &[]);
                                vc_ide::emulators::launch_backend("atari800", &vc_emu::Launch { kind: "atari800", args: &rp.args, ..Default::default() })
                            }
                            "fujisan" => {
                                if tf.target != "atari8" { return Err(vc_i18n::t!("cli-test-fujisan-target", target = &tf.target)); }
                                vc_ide::emulators::launch_backend("fujisan", &vc_emu::Launch { kind: "fujisan", ..Default::default() })
                            }
                            "xemu" => {
                                if tf.target != "mega65" { return Err(vc_i18n::t!("cli-test-xemu-target", target = &tf.target)); }
                                let rp = vc_ide::roms::plan("xemu", &vc_ide::roms::store(None, None), &[]);
                                if let Some(e) = rp.error { return Err(e); }
                                let rom = rp.xemu_rom.ok_or(vc_i18n::t!("cli-test-mega65-rom-missing"))?.to_string_lossy().into_owned();
                                vc_ide::emulators::launch_backend("xemu", &vc_emu::Launch { kind: "xemu", rom: Some(&rom), ..Default::default() })
                            }
                            _ => {
                                if prog.is_empty() { return Err(vc_i18n::t!("cli-test-vice-target", target = &tf.target)); }
                                let rp = vc_ide::roms::plan(prog, &vc_ide::roms::store(None, None), &[]);
                                let args = vice_test_args(&data, f, &tf.target, rp.args);
                                vc_ide::emulators::launch_backend(prog, &vc_emu::Launch { kind: prog, args: &args, ..Default::default() })
                            }
                        }
                    };
                    // atari800 steht nach dem Start im Monitor vor dem ersten Befehl, Xemu wartet schon beim Start auf das Hochfahren, Fujisan steht nach dem Laden am Einsprung: keine zusätzliche Pause
                    let ro = if which == "vice" { ro.clone() } else { runner::RunOptions { boot_wait: std::time::Duration::ZERO, ..ro.clone() } };
                    let dynamic = if tf.mode == vc_ide::tests::TestMode::Mailbox {
                        let ro = runner::RunOptions { poll, ..ro.clone() };
                        let (target, which) = (tf.target.clone(), which.to_string());
                        vc_ide::tests::mailbox::run_mailbox(&tf, &pre, &ro, &mut |p: &Path| vc_ide::tests::suite::launch_program(&which, &target, p))
                    } else { runner::run_tests(&tf, &pre, &ro, &mut launch) };
                    rep.outcomes.extend(dynamic);
                }
                Err(_) => {}      // der Fehler steht schon bei den statischen Ergebnissen
            }
        }
        println!("{}  ({}, {})", show_path(f), rep.target, rep.program);
        for o in &rep.outcomes {
            let (tag, n) = match (o.state, o.level) { (State::Passed, _) => (vc_i18n::t!("cli-test-tag-ok"), &mut passed), (State::Failed, Level::Error) => (vc_i18n::t!("cli-test-tag-error"), &mut failed), (State::Failed, Level::Hint) => (vc_i18n::t!("cli-test-tag-hint"), &mut hints), (State::TimedOut, _) => (vc_i18n::t!("cli-test-tag-timeout"), &mut timeouts), (State::Skipped, _) => (vc_i18n::t!("cli-test-skipped-label"), &mut skipped) };
            *n += 1;
            let loc = o.location.as_ref().map(|(f, l)| format!("  [{}:{l}]", Path::new(f).file_name().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default())).unwrap_or_default();
            println!("  {tag:<8} {}{}{loc}", o.name, o.measured.as_ref().map(|m| format!("  {m}")).unwrap_or_default());
            if !o.detail.is_empty() { println!("           {}", o.detail); }
        }
        suites.push(vc_ide::tests::junit::SuiteResult { name: show_path(f).trim_start_matches("./").to_string(), outcomes: rep.outcomes.clone(), millis: t0.elapsed().as_millis() as u64 });
    }
    if let Some(j) = &junit { match std::fs::write(j, vc_ide::tests::junit::render(&suites)) { Ok(()) => println!("{}", vc_i18n::t!("cli-test-junit-written", j = &j)), Err(e) => return fail_msg(vc_core::msg!("cli-file-error-j", j = &j, e = &e)) } }
    println!("\n{}", vc_i18n::t!("cli-test-summary", passed = &passed, failed = failed + broken, timeouts = &timeouts, hints = &hints, skipped = &skipped, strict = if strict { format!("  {}", vc_i18n::t!("cli-test-strict-mark")) } else { String::new() }));
    if failed + broken + timeouts > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// `u64:HOST[:PORT]` (Ultimate 64 / II+) oder `m65:SCHNITTSTELLE[@BAUD]` (MEGA65) → Hardware. Das Netzwerkpasswort kommt aus `--password`,
/// sonst aus der Maschine mit derselben Adresse in den Einstellungen. Ohne Adresse wird nichts angesprochen.
fn parse_hardware(b: &str, password: Option<&str>) -> Result<vc_ide::tests::suite::Hardware, String> {
    use vc_ide::tests::suite::Hardware;
    if let Some(host) = b.strip_prefix("u64:") {
        let host = host.trim();
        if host.is_empty() { return Err(vc_i18n::t!("cli-test-u64-address-missing").into()); }
        let st = vc_ide::settings::effective();
        let m = st.machines.iter().find(|m| m.kind == "ultimate64" && m.host.trim() == host);
        let pw = password.map(String::from).or_else(|| m.map(|m| m.password.clone())).filter(|p| !p.is_empty());
        return Ok(Hardware::Ultimate { host: host.to_string(), password: pw, name: m.map(|m| m.display_name().to_string()).unwrap_or_default() });
    }
    if let Some(dev) = b.strip_prefix("m65:") {
        if dev.trim().is_empty() { return Err(vc_i18n::t!("cli-test-m65-port-missing").into()); }
        return Ok(Hardware::Mega65 { spec: format!("serial:{}", dev.trim()), name: String::new() });
    }
    Err(vc_i18n::t!("cli-test-hw-backend-invalid", b = &b))
}

/// Sicherheitsabfrage vor jedem Lauf auf Hardware: `--yes` oder Rückfrage im Terminal; nicht interaktiv ohne `--yes` wird abgebrochen
fn confirm_hardware(label: &str, yes: bool) -> Result<(), String> {
    use std::io::{BufRead, IsTerminal, Write};
    err_msg(vc_core::msg!("cli-test-hw-warning", label = &label));
    if yes { return Ok(()); }
    if !std::io::stdin().is_terminal() { return Err(vc_i18n::t!("cli-test-hw-needs-yes").into()); }
    eprint!("{} ", vc_i18n::t!("cli-test-hw-ask")); let _ = std::io::stderr().flush();
    let mut l = String::new(); let _ = std::io::stdin().lock().read_line(&mut l);
    if matches!(l.trim().to_lowercase().as_str(), "j" | "ja" | "y" | "yes") { Ok(()) } else { Err(vc_i18n::t!("cli-test-hw-cancelled")) }
}

fn build_cmd(args: &[String]) -> ExitCode {
    let (mut path, mut only_main, mut data_dir) = (None, false, None);
    let (mut report, mut json, mut deny, mut strict) = (false, false, vec![], false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--main" => only_main = true, "--data-dir" => data_dir = it.next().cloned(),
            "--license-report" => report = true, "--json" => json = true, "--strict" => strict = true,
            "--deny-license" => match it.next() { Some(l) => deny.extend(l.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty())), None => return fail_msg(vc_core::msg!("cli-deny-license-invalid")) },
            s if !s.starts_with('-') => path = Some(s.to_string()),
            s => { return help::unknown_option("build", s); }
        }
    }
    if json && !report { return fail_msg(vc_core::msg!("cli-json-needs-license-report")); }
    if !deny.is_empty() { vc_ide::license::set_cli_policy(vc_ide::license::Policy { deny, allow: vec![] }); }
    let p = PathBuf::from(path.unwrap_or_else(|| ".".into()));
    let (root, file) = if p.is_dir() { match vc_ide::projfile::find(&p) { Some(f) => (p.clone(), f), None => return fail_msg(vc_core::msg!("cli-no-project-file", path = p.display().to_string())) } } else { (p.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")), p.clone()) };
    let pf = match vc_ide::projfile::ProjectFile::load(&file) { Ok(x) => x, Err(e) => return fail(e) };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    for w in pf.validate(&root) { note(vc_core::msg!("cli-warning", text = &w)); }
    if pf.source_files().is_empty() { return fail_msg(vc_core::msg!("cli-license-no-sources")); }
    // Zielsystem nicht im Release: eine klare Meldung, Exit-Code 2, Projektdatei bleibt unberührt
    let mut missing: Vec<String> = pf.targets.iter().map(|t| t.id.clone()).chain(pf.source_files().into_iter().filter_map(|s| s.2)).filter(|t| !vc_target::target_available(&data, t)).collect();
    missing.sort(); missing.dedup();
    if let Some(t) = missing.first() { return fail_target(vc_target::missing_message(t)); }
    let main = pf.main_file();
    let r = vc_ide::projfile::build_project(&vc_ide::build::Ctx { root: &root, data: &data }, &pf, if only_main { main.as_deref() } else { None });
    for x in r["results"].as_array().into_iter().flatten() {
        for d in x["diagnostics"].as_array().into_iter().flatten() {
            if d["severity"] == "info" { eprintln!("{}", d["message"].as_str().unwrap_or("")); continue; }
            // `--json`: jede Meldung als JSON-Zeile mit `code` und `args` (fehlen die Parameter in der Bauantwort, bleibt `args` leer)
            if json_mode() { let mut j = d.clone(); if j.get("args").is_none() { j["args"] = serde_json::json!({}); } eprintln!("{j}"); continue; }
            eprintln!("{}:{}: {} {}: {}", d["file"].as_str().unwrap_or(""), d["line"], d["severity"].as_str().unwrap_or(""), d["code"].as_str().unwrap_or(""), d["message"].as_str().unwrap_or(""));
        }
        let f = x["file"].as_str().unwrap_or("?");
        let line = if x["ok"] == true { vc_i18n::t!("cli-license-ok", f = &f, artifact = x["artifact"].as_str().unwrap_or("?"), size = x["size"].to_string(), mark = if main.as_deref() == Some(f) { format!("   {}", vc_i18n::t!("cli-license-main-mark")) } else { String::new() }) } else { vc_i18n::t!("cli-license-error", f = &f) };
        if json { eprintln!("{line}") } else { println!("{line}") }
    }
    if let Some(d) = r["disk"].as_object() { let l = format!("Disk    {} ({} Bytes): {}", d["file"].as_str().unwrap_or(""), d["size"], d["programs"].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default()); if json { eprintln!("{l}") } else { println!("{l}") } }
    if let Some(e) = r["disk_error"].as_str() { err_msg(vc_core::msg!("cli-build-disk-error", e = &e)); }
    // Turbo BASIC XL: Diskette (Kopie des Anwenders + AUTORUN.BAS); fehlt sie, ist das ein Hinweis (Exit-Code 0), mit --strict ein Fehler
    if let Some(l) = r["turbo"]["line"].as_str() { if json { eprintln!("{l}") } else { println!("{l}") } }
    for n in r["turbo"]["notes"].as_array().into_iter().flatten().filter_map(|n| n.as_str()) { note(vc_core::msg!("cli-warning", text = n)); }
    if let Some(e) = r["turbo_error"].as_str() { err_msg(vc_core::msg!("cli-error-text", text = e)); }
    let turbo_missing = r["turbo_missing"].as_str();
    if let Some(m) = turbo_missing { if strict { err_msg(vc_core::msg!("cli-error-text", text = m)); } else { note(vc_core::msg!("cli-warning", text = m)); } }
    if report { license_report(&root, &r, json); }
    if r["ok"] == true && !(strict && turbo_missing.is_some()) { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Kopf des Berichts (drei Zeilen) in der gewählten Sprache: `vc_ide::license::report_text` liefert ihn fest deutsch; die Zeilen darunter kommen schon aus dem Katalog.
/// Sobald `vc-ide` den Kopf selbst übersetzt, bleibt der Text unverändert (er beginnt dann nicht mehr mit dem deutschen Titel).
fn report_text_in_language(text: String) -> String {
    const DE_TITLE: &str = "Lizenzen der übersetzten Quelltexte";
    if !text.starts_with(DE_TITLE) { return text; }
    let rest: Vec<&str> = text.splitn(4, '\n').collect();
    let body = rest.get(3).copied().unwrap_or("");
    format!("{}\n{}\n{}\n{body}", vc_i18n::t!("cli-license-report-title"), vc_i18n::t!("cli-license-report-how"), vc_i18n::t!("cli-license-report-caveat"))
}

/// `vcc build --license-report [--json]`: Lizenzen aller übersetzten Quelltexte ausgeben und nach `build/LICENSES.txt` schreiben
fn license_report(root: &Path, r: &serde_json::Value, json: bool) {
    let progs: Vec<(String, String, Vec<vc_asm::license::LicenseEntry>, usize)> = r["results"].as_array().into_iter().flatten().filter(|x| x["licenses"].is_array()).map(|x| {
        let es = x["licenses"].as_array().into_iter().flatten().map(|e| vc_asm::license::LicenseEntry {
            file: e["file"].as_str().unwrap_or("").into(), license: e["license"].as_str().unwrap_or("").into(), holder: e["holder"].as_str().unwrap_or("").into(),
            origin: match e["notice"].as_str() { Some(f) => vc_asm::license::Origin::Folder(f.into()), None => vc_asm::license::Origin::Header } }).collect();
        (x["file"].as_str().unwrap_or("?").to_string(), x["target"].as_str().unwrap_or("?").to_string(), es, x["unlicensed"].as_u64().unwrap_or(0) as usize)
    }).collect();
    let show = |f: &str| f.to_string();
    let text = report_text_in_language(vc_ide::license::report_text(&progs, &show));
    let dir = root.join("build");
    let _ = std::fs::create_dir_all(&dir);
    match std::fs::write(dir.join("LICENSES.txt"), &text) { Ok(_) => note(vc_core::msg!("cli-license-report-written")), Err(e) => note(vc_core::msg!("cli-license-report-error", e = &e)) }
    if json { println!("{}", serde_json::to_string_pretty(&vc_ide::license::report_json(&progs, &show)).unwrap_or_default()); } else { print!("{text}"); }
}

/// `vcc basic --suggest-hex`: Chipregister-Adressen, die sich als Hex schreiben lassen, auflisten; mit `--apply` die gewählten in eine Ausgabedatei schreiben
fn basic_suggest_hex(input: &str, output: Option<String>, data: &Path, did: &str, target: &str, apply: Option<&str>) -> ExitCode {
    let text = match vc_core::textfile::read_source(input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
    let s = match vc_ide::hexhint::suggestions(data, did, target, &text) { Ok(s) => s, Err(e) => return fail(e) };
    let map = vc_ide::chipmap::ChipMap::load(data, target);
    let lines: Vec<&str> = text.split('\n').collect();
    for (i, x) in s.iter().enumerate() {
        let reg = map.lookup(x.value).map(|r| format!("  ({})", r.label())).unwrap_or_default();
        println!("{}", vc_i18n::t!("cli-suggest-line", number = format!("{:>3}", i + 1), line = x.line + 1, from = &x.from, to = &x.to, reg = &reg, text = lines.get(x.line).map_or("", |l| l.trim())));
    }
    let Some(sel) = apply else { println!("{}", vc_i18n::t!("cli-suggest-count", count = s.len())); return ExitCode::SUCCESS };
    let mut chosen = vec![false; s.len()];
    for part in sel.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        match part.parse::<usize>() { Ok(n) if n >= 1 && n <= s.len() => chosen[n - 1] = true, _ => return fail_msg(vc_core::msg!("cli-apply-invalid-number", part = &part, max = s.len())) }
    }
    let Some(out) = output else { return fail_msg(vc_core::msg!("cli-apply-needs-output")) };
    if out == input { return fail_msg(vc_core::msg!("cli-apply-would-overwrite", out = &out)); }
    let new = vc_ide::hexhint::apply_suggestions(&text, &s, &chosen);
    match std::fs::write(&out, new) { Ok(()) => { println!("{}", vc_i18n::t!("cli-apply-done", out = &out, count = chosen.iter().filter(|c| **c).count())); ExitCode::SUCCESS } Err(e) => fail_msg(vc_core::msg!("cli-file-error-out", out = &out, e = &e)) }
}

/// Haken für `#include` im nativen BASIC: Medien (`#include "graphics/hero.vcsprite"`) werden beim Einbinden als DATA umgewandelt, Altpfade `build/export/…`
/// erhalten den Hinweis `B194` (Projektordner = nächster Ordner mit Projektdatei, wie beim Assembler)
fn basic_include_hooks(input: &str, data: &Path, target: &str) -> vc_basic::include::IncludeHooks {
    use std::sync::Arc;
    let abs = std::fs::canonicalize(input).unwrap_or_else(|_| PathBuf::from(input));
    let (root, data, target) = (vc_ide::launch::project_root_of(&abs), data.to_path_buf(), target.to_string());
    let (r1, r2) = (root.clone(), root);
    let provider: vc_basic::include::IncludeProvider = Arc::new(move |p: &Path| {
        if !vc_media::include::is_media_path(&p.to_string_lossy()) { return None; }
        Some(vc_media::include::resolve_include(&r1, &data, p, vc_media::include::IncFormat::Data, Some(&target)).map(|x| x.text).map_err(|e| e.to_string()))
    });
    let legacy: vc_basic::include::LegacyHint = Arc::new(move |p: &Path| vc_media::include::legacy_media(&r2, p));
    vc_basic::include::IncludeHooks { provider: Some(provider), legacy_hint: Some(legacy) }
}

/// `vcc basic --to-labels|--to-numbers|--renumber`: Zeilennummern ↔ Labels, Neu-Nummerieren (Datei samt eingebundener Dateien)
fn basic_convert(kind: &str, input: &str, output: Option<String>, write_back: bool, preview: bool, data: &Path, did: &str, target: &str, (start, step): (u32, u32), range: Option<(u32, u32)>) -> ExitCode {
    use vc_basic::labels;
    let src = match vc_core::textfile::read_source(input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
    let (units, media) = match vc_basic::include::load_set_media(input, &src, &basic_include_hooks(input, data, target)) { Ok(u) => u, Err(ds) => { ds.iter().for_each(print_diag); return ExitCode::FAILURE; } };
    let (dialect, info) = match (vc_basic::dialects::AnyDialect::load(data, did), vc_basic::dialects::info(data, did)) { (Ok(d), Ok(i)) => (d, i), (Err(e), _) | (_, Err(e)) => return fail(e) };
    let num = match vc_basic::numbering::resolve(&src, input, vc_basic::numbering::Numbering::new(start, step), info.max_line_number) { Ok(n) => n, Err(ds) => { ds.iter().for_each(print_diag); return ExitCode::FAILURE; } };
    let cx = labels::Cx::new(&dialect, &info).numbering(num.start, num.step);
    let result = match kind { "labels" => labels::to_labels(&units, &cx), "numbers" => labels::to_numbers(&units, &cx, None), _ => labels::renumber(&units, &cx, range) };
    let c = match result { Ok(c) => c, Err(ds) => { ds.iter().for_each(print_diag); return ExitCode::FAILURE; } };
    c.warnings.iter().for_each(print_diag);
    note(vc_core::msg!("cli-convert-summary", changed = c.changes.len(), new = c.new_labels.len(), kept = c.kept_labels.len()));
    for l in &c.new_labels { note(vc_core::msg!("cli-convert-new-label", name = l.name.as_str(), file = l.file.as_str(), line = l.line)); }
    if preview {
        for ch in &c.changes { println!("{}", vc_i18n::t!("cli-convert-preview-line", file = ch.file.as_str(), line = ch.line, old = ch.old.trim(), new = ch.new.replace('\n', " ⏎ ").trim())); }
        return ExitCode::SUCCESS;
    }
    let others = c.units.iter().zip(&units).skip(1).filter(|(n, o)| n.text != o.text && !media.contains(&n.file)).count();
    if write_back {
        for (n, o) in c.units.iter().zip(&units) {
            if n.text == o.text || media.contains(&n.file) { continue; }          // Medien sind schreibgeschützt (der erzeugte Text gehört in keine Datei)
            if let Err(e) = std::fs::write(&n.file, &n.text) { return fail(format!("{}: {e}", n.file)); }
            println!("{}", vc_i18n::t!("cli-convert-written", file = n.file.as_str()));
        }
        return ExitCode::SUCCESS;
    }
    if others > 0 { note(vc_core::msg!("cli-convert-includes-hint", count = others)); }
    match output {
        Some(p) => { if p == input { return fail_msg(vc_core::msg!("cli-output-would-overwrite", p = &p)); } match std::fs::write(&p, &c.units[0].text) { Ok(()) => { println!("{}", vc_i18n::t!("cli-convert-written", file = p.as_str())); ExitCode::SUCCESS } Err(e) => fail(format!("{p}: {e}")) } }
        None => { print!("{}", c.units[0].text); ExitCode::SUCCESS }
    }
}

/// Natives BASIC (nummeriert und/oder mit `@label`) → `.prg` (CBM) bzw. `.bas` (Atari, SAVE-Format)
fn basic_plain(input: &str, output: Option<String>, data: &Path, did: &str, tgt: &vc_target::Target, (start, step): (u32, u32), check: vc_basic::check::CheckOptions, check_only: bool, defs: &[(String, i64)], compact: bool) -> ExitCode {
    let src = match vc_core::textfile::read_source(input) { Ok(s) => s, Err(e) => return fail(format!("{input}: {e}")) };
    // `#include` (Dateien und Medien) einbinden; Meldungen des Übersetzers zeigen danach auf Datei und Zeile der Quelle
    let exp = match vc_basic::expand_includes_with(&src, input, &basic_include_hooks(input, data, &tgt.id)) { Ok(e) => e, Err(ds) => { ds.iter().for_each(print_diag); return ExitCode::FAILURE; } };
    let (src, map, media_count) = (exp.text, exp.map, exp.media.len());
    for w in &exp.warnings { print_diag(w); }
    let write = |bytes: &[u8], ext: &str| -> ExitCode {
        let p = output.clone().unwrap_or_else(|| Path::new(input).with_extension(ext).to_string_lossy().into_owned());
        if p == input { return fail_msg(vc_core::msg!("cli-output-would-overwrite", p = &p)); }
        match std::fs::write(&p, bytes) { Ok(()) => { println!("{p}: {} Bytes", bytes.len()); ExitCode::SUCCESS } Err(e) => fail(format!("{p}: {e}")) }
    };
    let family = match vc_basic::dialect_family(data, did) { Ok(f) => f, Err(e) => return fail(e) };
    // Maschinencode (`#asm "datei.asm"`): der Code wird mit der Technik des strukturierten BASIC übersetzt und hinter den Text bzw. in DATA-Zeilen gelegt
    let has_blob = src.lines().any(vc_basic::blob::is_blob_directive);
    let infos: std::cell::RefCell<Vec<vc_ide::blob::BlobInfo>> = std::cell::RefCell::new(vec![]);
    let root = Path::new(input).parent().map(Path::to_path_buf).unwrap_or_default();
    let mut asm = |reqs: &[vc_basic::BlobReq], addr: usize, strict: bool| vc_ide::blob::assemble_blobs(reqs, addr, &tgt.id, &root, data, tgt, strict)
        .map(|(bytes, symbols, i, chunks)| { *infos.borrow_mut() = i; vc_basic::blob::Assembled { bytes, symbols, chunks } });
    let result = if has_blob && family == "atari" {
        let d = match vc_basic::atari::AtariDialect::load(data, did) { Ok(d) => d, Err(e) => return fail(e) };
        let mut o = vc_basic::PlainOptions::new(input, 32767); (o.start, o.step) = (start, step); o.check = check; o.compact = compact; o.system = Some(tgt.id.clone()); o.media = media_count; o.defines = defs.iter().map(|(n, v)| (n.clone(), v.to_string())).collect();
        vc_basic::blob::compile_plain_atari_blobs(&src, &d, &o, &mut asm).map(|r| (r.built.file, "bas", r.built.labels, r.built.names, r.built.warnings))
    } else if has_blob {
        let d = match vc_basic::Dialect::load(data, did) { Ok(d) => d, Err(e) => return fail(e) };
        let mut o = vc_basic::PlainOptions::new(input, 63999); (o.start, o.step) = (start, step); o.check = check; o.compact = compact; o.system = Some(tgt.id.clone()); o.media = media_count; o.defines = defs.iter().map(|(n, v)| (n.clone(), v.to_string())).collect();
        let place = if tgt.id == "mega65" { vc_basic::blob::Placement::Data(Some(vc_ide::blob::MEGA65_BLOB_BASE)) } else { vc_basic::blob::Placement::Behind };
        vc_basic::blob::compile_plain_cbm_blobs(&src, &d, &o, tgt.basic_load_addr.unwrap_or(d.load_addr), place, &mut asm).map(|r| (r.built.prg, "prg", r.built.labels, r.built.names, r.built.warnings))
    } else if family == "atari" {
        let d = match vc_basic::atari::AtariDialect::load(data, did) { Ok(d) => d, Err(e) => return fail(e) };
        let mut o = vc_basic::PlainOptions::new(input, 32767); (o.start, o.step) = (start, step); o.check = check; o.compact = compact; o.system = Some(tgt.id.clone()); o.media = media_count; o.defines = defs.iter().map(|(n, v)| (n.clone(), v.to_string())).collect();
        vc_basic::compile_plain_atari(&src, &d, &o).map(|r| (r.file, "bas", r.labels, r.names, r.warnings))
    } else {
        let d = match vc_basic::Dialect::load(data, did) { Ok(d) => d, Err(e) => return fail(e) };
        let mut o = vc_basic::PlainOptions::new(input, 63999); (o.start, o.step) = (start, step); o.check = check; o.compact = compact; o.system = Some(tgt.id.clone()); o.media = media_count; o.defines = defs.iter().map(|(n, v)| (n.clone(), v.to_string())).collect();
        vc_basic::compile_plain_cbm(&src, &d, &o, tgt.basic_load_addr.unwrap_or(d.load_addr)).map(|r| (r.prg, "prg", r.labels, r.names, r.warnings))
    };
    match result {
        Ok((bytes, ext, labels, names, warnings)) => {
            for w in &vc_basic::remap_diagnostics(&warnings, input, &map) { print_diag(w); }
            if check_only { return ExitCode::SUCCESS; }
            for b in infos.borrow().iter() { println!("{}", vc_i18n::t!("cli-basic-machine-code", file = &b.file, addr = format!("{:04X}", b.addr), size = &b.size)); }
            for (n, l) in &labels { println!("{}", vc_i18n::t!("cli-label-line", n = &n, l = &l)); }
            if !names.is_empty() { println!("{}", vc_i18n::t!("cli-migrate-long-names")); for n in &names { println!("  {}", n.line()); } }
            write(&bytes, ext)
        }
        Err(ds) => { for x in &vc_basic::remap_diagnostics(&ds, input, &map) { print_diag(x); } ExitCode::FAILURE }
    }
}

fn migrate_cmd(args: &[String]) -> ExitCode {
    if args.first().map(String::as_str) == Some("includes") { return migrate_includes_cmd(&args[1..]); }
    let root = std::path::PathBuf::from(args.first().map(String::as_str).unwrap_or("."));
    match vc_media::naming::migrate(&root) {
        Ok(m) if m.is_empty() => { println!("{}", vc_i18n::t!("cli-migrate-nothing")); ExitCode::SUCCESS }
        Ok(m) => { for (a, b) in &m { println!("{}", vc_i18n::t!("cli-migrate-pair", a = &a, b = &b)); } println!("{}", vc_i18n::t!("cli-migrate-done", count = m.len())); ExitCode::SUCCESS }
        Err(e) => fail(e),
    }
}

/// `vcc migrate includes [projektordner] [--apply]`: alte Include-Pfade `build/export/…` auf die Mediendatei umstellen (ohne `--apply` nur anzeigen)
fn migrate_includes_cmd(args: &[String]) -> ExitCode {
    let apply = args.iter().any(|a| a == "--apply");
    let root = std::path::PathBuf::from(args.iter().find(|a| !a.starts_with("--")).map(String::as_str).unwrap_or("."));
    match vc_media::include::migrate_includes(&root, apply) {
        Ok(c) if c.is_empty() => { println!("{}", vc_i18n::t!("cli-migrate-nothing")); ExitCode::SUCCESS }
        Ok(c) => {
            let (mut done, mut skipped) = (0, 0);
            for x in &c {
                match (&x.new, &x.skipped) {
                    (Some(n), _) => { done += 1; println!("{}", vc_i18n::t!("cli-migrate-includes-change", file = &x.file, line = x.line, old = &x.old, new = n)); }
                    (None, None) => { done += 1; println!("{}", vc_i18n::t!("cli-migrate-includes-removed", file = &x.file, line = x.line, old = &x.old)); }
                    (None, Some(why)) => { skipped += 1; println!("{}", vc_i18n::t!("cli-migrate-includes-skipped", file = &x.file, line = x.line, old = &x.old, why = why)); }
                }
            }
            if apply { println!("{}", vc_i18n::t!("cli-migrate-includes-done", count = done, skipped = skipped)); } else { println!("{}", vc_i18n::t!("cli-migrate-includes-dry", count = done, skipped = skipped)); }
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

/// `vcc debuglog [DATEI] [--json] [--tail N] [--path]`: Debug-Protokoll auswerten
fn debuglog_cmd(args: &[String]) -> ExitCode {
    let (mut json, mut tail, mut path_only, mut file) = (false, 20usize, false, None::<String>);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true, "--path" => path_only = true,
            "--tail" => match it.next().and_then(|v| v.parse().ok()) { Some(n) => tail = n, None => return help::usage_error("debuglog") },
            s if !s.starts_with("--") && file.is_none() => file = Some(s.to_string()),
            s => { return help::unknown_option("debuglog", s); }
        }
    }
    let Some(path) = file.map(PathBuf::from).or_else(vc_ide::debuglog::path) else { return fail_msg(vc_core::msg!("cli-debuglog-no-config-dir")) };
    if path_only { println!("{}", path.display()); return ExitCode::SUCCESS; }
    let text = match std::fs::read_to_string(&path) { Ok(t) => t, Err(e) => return fail(format!("vcc debuglog: {}: {e}", path.display())) };
    let s = vc_ide::debuglog::summarize(&text, tail);
    if json { println!("{}", serde_json::to_string_pretty(&s.to_json()).unwrap_or_default()); } else { print!("{}", s.to_text()); }
    ExitCode::SUCCESS
}

/// `vcc feedback [--open] [--json] [--target SYSTEM]`: Betreff und Text der Rückmeldungs-Mail ausgeben, auf Wunsch im Mailprogramm öffnen
fn feedback_cmd(args: &[String]) -> ExitCode {
    let (mut open, mut json, mut target) = (false, false, None::<String>);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--open" => open = true, "--json" => json = true, "--target" => target = it.next().cloned(),
            s => { return help::unknown_option("feedback", s); }
        }
    }
    let info = vc_ide::feedback::Info::detect(target.as_deref());
    let mail = vc_ide::feedback::build(&info, vc_core::userdirs::home_dir().as_deref().and_then(std::path::Path::to_str), vc_ide::feedback::MAX_URL);
    if json { println!("{}", serde_json::to_string_pretty(&mail.to_json(&info)).unwrap_or_default()); }
    else {
        println!("{}", vc_i18n::t!("cli-feedback-mail-preview", to = vc_ide::feedback::ADDRESS, subject = &mail.subject, body = &mail.body));
        if mail.too_long { note(vc_core::msg!("cli-feedback-link-too-long", max = vc_ide::feedback::MAX_URL)); }
    }
    if open {
        match vc_ide::feedback::open_mailto(&mail.url) { Ok(()) => { if !json { note(vc_core::msg!("cli-feedback-mail-opened")); } } Err(e) => return fail_msg(vc_core::msg!("cli-feedback-mail-failed", e = &e)) }
    }
    ExitCode::SUCCESS
}

/// `vcc update [--check] [--notes] [--json] [--offline]`: nachsehen, ob es eine neuere Version gibt. Zeigt nur an; lädt und installiert nichts.
/// Rückgabe: 0 aktuell, 10 Update verfügbar, 1 Fehler, 2 falscher Aufruf oder Release ohne Update-Prüfung.
fn update_cmd(args: &[String]) -> ExitCode {
    use vc_ide::update as up;
    let (mut notes, mut json, mut offline) = (false, false, false);
    for a in args {
        match a.as_str() {
            "--check" => {}, "--notes" => notes = true, "--json" => json = true, "--offline" => offline = true,
            s => { return help::unknown_option("update", s); }
        }
    }
    if !vc_ide::features::features().update() { err_msg(vc_core::msg!("cli-update-not-in-package")); return ExitCode::from(2); }
    let installed = up::Version::installed();
    let mut st = up::State::load();
    let (ev, source, at) = if offline {
        (up::cached(installed), "cache".to_string(), st.last_check)
    } else {
        let src = up::source();
        if !json { note(vc_core::msg!("cli-update-checking", source = if up::is_remote(&src) { "github.com/CDrueeke/VintageCoder" } else { src.as_str() })); }
        let rep = up::check(&src, installed, up::now_secs());
        up::record(&rep, &mut st);
        match rep.result { Ok(ev) => (Some(ev), src, rep.at), Err(e) => return fail(format!("vcc update: {e}")) }
    };
    let Some(ev) = ev else {
        if json { println!("{}", serde_json::json!({"installed": installed.to_string(), "checked": false, "update_available": false})); } else { println!("{}", vc_i18n::t!("cli-update-no-check-yet")) }
        return ExitCode::SUCCESS;
    };
    let code = if ev.update.is_some() { 10 } else { 0 };
    if json {
        let u = ev.update.as_ref();
        let o = serde_json::json!({"installed": installed.to_string(), "checked": true, "checked_at": up::format_time(at), "source": source, "latest": ev.latest.to_string(),
            "update_available": u.is_some(), "version": u.map(|u| u.version.to_string()), "url": u.map(|u| u.url.clone()), "published": u.map(|u| u.published.clone()),
            "project_format_warning": u.map_or(false, |u| u.project_format_warning), "installed_yanked": ev.installed_yanked, "notes": if notes { u.map(|u| u.notes.clone()) } else { None }});
        println!("{}", serde_json::to_string_pretty(&o).unwrap_or_default());
        return ExitCode::from(code);
    }
    println!("VintageCoder {installed} ({})", vc_ide::feedback::Info::detect(None).package.label());
    if ev.installed_yanked { println!("{}", vc_i18n::t!("cli-update-yanked", installed = &installed)); }
    match &ev.update {
        Some(u) => {
            println!("{}", vc_i18n::t!("cli-update-available", version = &u.version));
            if u.project_format_warning { println!("{}", vc_i18n::t!("cli-update-format-warning")); }
            println!("{}", vc_i18n::t!("cli-update-release-page", url = &u.url));
            if let Some(h) = up::package_hint(up::install_kind()) { println!("{h}"); }
            if notes { println!("\n{}", u.notes); } else { println!("{}", vc_i18n::t!("cli-update-show-notes")); }
        }
        None => println!("{}", vc_i18n::t!("cli-update-up-to-date", latest = &ev.latest)),
    }
    ExitCode::from(code)
}

/// `vcc import`: Dateien aus Diskettenimages bzw. einzelne PRG/XEX/BASIC-Dateien in ein Projekt übernehmen.
fn import_cmd(args: &[String]) -> ExitCode {
    use vc_ide::diskimage::{detect_basic, import_data, is_image_name, is_xex, Image, ImportOpts};
    let (mut input, mut names, mut all, mut into, mut data_dir) = (None::<String>, vec![], false, None::<String>, None);
    let mut o = ImportOpts::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--all" => all = true, "--asm" => o.asm = true, "--raw" => o.raw = true, "--no-basic" => o.basic = false,
            "--into" => into = it.next().cloned(), "--dir" => o.dir = it.next().cloned().unwrap_or(o.dir), "--cpu" => o.cpu = it.next().cloned(), "--data-dir" => data_dir = it.next().cloned(),
            s if !s.starts_with("--") => { if input.is_none() { input = Some(s.to_string()) } else { names.push(s.to_string()) } }
            s => { return help::unknown_option("import", s); }
        }
    }
    let Some(input) = input else { return help::usage_error("import") };
    let root = PathBuf::from(into.unwrap_or_else(|| ".".into()));
    let data = find_data_dir(data_dir);
    let report = |name: &str, r: Result<vc_ide::diskimage::Outcome, String>| -> bool {
        match r {
            Ok(out) => { println!("{name} → {}", out.files.join(", ")); for n in &out.notes { println!("  {n}"); } true }
            Err(m) => { eprintln!("{name}: {m}"); false }
        }
    };
    if is_image_name(&input) {
        let img = match Image::open(Path::new(&input)) { Ok(i) => i, Err(e) => return fail(e) };
        if names.is_empty() && !all {
            if img.atari { println!("{}  ({})", img.format, img.free); } else { println!("{} \"{}\"  ({})", img.format, img.title, img.free); }
            for e in &img.entries {
                let load = e.load_address().map(|a| format!("  ${a:04x}")).unwrap_or_default();
                let basic = match (&e.data, &data) { (Ok(d), Some(dd)) if e.kind == "PRG" || img.atari => detect_basic(dd, d, img.atari).map(|b| format!("  {}", vc_i18n::t!("cli-import-basic", dialect = &b.dialect, lines = &b.lines))).unwrap_or_default(), _ => String::new() };
                let err = e.data.as_ref().err().map(|m| format!("  {}", vc_i18n::t!("cli-import-error", m = &m))).unwrap_or_default();
                println!("{:<18} {:<4} {:>6} Bytes{load}{basic}{err}", e.name, e.kind, e.size());
            }
            println!("{}", vc_i18n::t!("cli-import-hint", input = &input));
            return ExitCode::SUCCESS;
        }
        let mut idx = vec![];
        if all { idx.extend(0..img.entries.len()); }
        for n in &names {
            match img.entries.iter().position(|e| e.name.eq_ignore_ascii_case(n)) { Some(i) => idx.push(i), None => return fail_msg(vc_core::msg!("cli-import-not-on-disk", n = &n, input = &input)) }
        }
        let (mut ok, mut n) = (true, 0);
        for i in idx {
            let e = &img.entries[i];
            let r = match &e.data { Ok(d) => import_data(&root, &img.suggested_name(e), d, e.kind == "PRG" || e.kind == "DOS", img.atari, data.as_deref(), &o), Err(m) => Err(m.clone()) };
            if report(&e.name, r) { n += 1 } else { ok = false }
        }
        println!("{}", vc_i18n::t!("cli-files-taken", n = &n));
        if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
    } else {
        let bytes = match std::fs::read(&input) { Ok(b) => b, Err(e) => return fail(format!("{input}: {e}")) };
        let fname = Path::new(&input).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "datei".into());
        let atari = is_xex(&fname, &bytes) || fname.to_ascii_lowercase().ends_with(".bas") && bytes.len() > 14 && bytes[0] == 0 && bytes[1] == 0;
        let program = atari || fname.to_ascii_lowercase().ends_with(".prg");
        if report(&input, import_data(&root, &fname, &bytes, program, atari, data.as_deref(), &o)) { ExitCode::SUCCESS } else { ExitCode::FAILURE }
    }
}

/// Emulator-Argumente für `vcc test` mit VICE: die ROM-Argumente plus die Einstellungen aus der Projektdatei, die zur Testdatei gehört
/// (VC-20: `-memory <Ausbau>` aus der Einstellung `memory`, genau wie beim Start in der IDE; ohne Einstellung gilt der Standardausbau des Ziels)
fn vice_test_args(data: &Path, test_file: &Path, target: &str, mut base: Vec<String>) -> Vec<String> {
    let abs = std::fs::canonicalize(test_file).unwrap_or_else(|_| test_file.to_path_buf());
    base.extend(vc_ide::launch::project_args(data, &vc_ide::launch::project_root_of(&abs), target));
    base
}

/// `vcc streams listen --host HOST LIED.vcsong [--secs N] [--wav DATEI] [--ntsc] [--keep]`: Hörprobe über den echten SID. Lädt das Lied als
/// Abspielprogramm auf die Ultimate (Reset!), empfängt den Audio-Strom, schreibt ihn bei `--wav` als WAV (Quellrate) und setzt die Ultimate danach
/// zurück (außer mit `--keep`). Ändert nichts an der Konfiguration, speichert nichts im Flash.
fn streams_listen(args: &[String]) -> ExitCode {
    use vc_emu::streams::audio::{AudioHub, RATE_NTSC, RATE_PAL};
    let (mut host, mut file, mut secs, mut password, mut multicast, mut wav, mut ntsc, mut keep) = (None::<String>, None::<String>, 6u64, None::<String>, false, None::<String>, false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--host" => host = it.next().cloned(), "--password" => password = it.next().cloned(), "--multicast" => multicast = true, "--wav" => wav = it.next().cloned(), "--ntsc" => ntsc = true, "--keep" => keep = true,
            "--secs" => match it.next().and_then(|v| v.parse::<u64>().ok()) { Some(n) => secs = n.clamp(2, 120), None => return fail_msg(vc_core::msg!("cli-secs-needs-number")) },
            s if !s.starts_with("--") && file.is_none() => file = Some(s.to_string()),
            s => { return help::unknown_option("streams", s); }
        }
    }
    let (Some(host), Some(file)) = (host.filter(|h| !h.trim().is_empty()), file) else { return fail_msg(vc_core::msg!("cli-streams-listen-usage")) };
    let Some(data) = vc_ide::find_data_dir(&std::env::current_dir().unwrap_or_default()).or_else(|| std::env::current_exe().ok().and_then(|e| vc_ide::find_data_dir(&e))) else { return fail_msg(vc_core::msg!("cli-no-data-dir")) };
    let text = match vc_core::textfile::read_source(&file) { Ok(t) => t, Err(e) => return fail_msg(vc_core::msg!("cli-file-error-file", file = &file, e = &e)) };
    let prg = match vc_ide::listen::build_player(&data, &text, ntsc) { Ok(p) => p, Err(e) => return fail(e) };
    println!("{}", vc_i18n::t!("cli-streams-player-size", size = prg.len()));
    let st = vc_ide::settings::effective();
    let pw = password.or_else(|| st.machines.iter().find(|m| m.kind == "ultimate64" && m.host.trim() == host).map(|m| m.password.clone())).filter(|p| !p.is_empty());
    let mut opts = vc_ide::streams::StreamOptions::from_settings(&st); opts.multicast |= multicast;
    let hub = std::sync::Arc::new(std::sync::Mutex::new(AudioHub::new(if ntsc { RATE_NTSC } else { RATE_PAL }, 48000.0)));
    if let Some(w) = &wav { match vc_emu::streams::wav::WavWriter::create(Path::new(w), (if ntsc { RATE_NTSC } else { RATE_PAL }).round() as u32) { Ok(r) => hub.lock().unwrap().rec = Some(r), Err(e) => return fail(e) } }
    let mut svc = match vc_ide::listen::start_hardware(&host, pw.as_deref(), opts, &prg, hub.clone()) { Ok(s) => s, Err(e) => return fail(e) };
    let t0 = std::time::Instant::now();
    while t0.elapsed() < std::time::Duration::from_secs(secs) { std::thread::sleep(std::time::Duration::from_millis(100)); svc.poll(); if matches!(svc.state(vc_emu::streams::StreamKind::Audio), vc_ide::streams::StreamState::Error(_)) { break; } }
    svc.poll();
    let s = svc.status(vc_emu::streams::StreamKind::Audio);
    println!("{}", vc_i18n::t!("cli-stream-audio-stats", state = format!("{:?}", s.state), packets = &s.packets, lost = &s.lost, percent = vc_i18n::num(s.loss_percent as f64, 2)));
    let bad = matches!(s.state, vc_ide::streams::StreamState::Error(_));
    if let vc_ide::streams::StreamState::Error(e) = &s.state { println!("{}", vc_i18n::t!("cli-streams-error", e = &e)); }
    if let Err(e) = vc_ide::listen::stop_hardware(&mut svc, !keep) { eprintln!("{e}"); }
    let mut h = hub.lock().unwrap();
    let ps = h.player.stats();
    println!("{}", vc_i18n::t!("cli-streams-buffer", fill = vc_i18n::num((ps.fill_ms) as f64, 0), filled = &ps.filled_packets, underruns = &ps.underruns, overruns = &ps.overruns));
    if let Some(mut w) = h.rec.take() { let _ = w.finish(); println!("{}", vc_i18n::t!("cli-streams-wav", seconds = vc_i18n::num((w.seconds()) as f64, 2), path = w.path().display())); }
    println!("{}", vc_i18n::t!("cli-streams-stopped", suffix = if keep { String::new() } else { format!(", {}", vc_i18n::t!("cli-stream-ultimate-reset")) }));
    if bad || s.packets == 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// `vcc streams probe --host HOST [--video|--audio|--debug] [--secs N] [--password PW] [--multicast] [--record ORDNER [--count N]]`:
/// startet die Ströme, zählt Pakete (2 bis 60 s), stoppt sie wieder und gibt Rate und Verluste aus. Ändert nichts an der Ultimate (kein Flash, keine Konfiguration).
fn streams_cmd(args: &[String]) -> ExitCode {
    use vc_emu::streams::StreamKind;
    use vc_ide::streams::{StreamOptions, StreamService, StreamState};
    if args.first().map(String::as_str) == Some("shot") { return streams_shot(&args[1..]); }
    if args.first().map(String::as_str) == Some("listen") { return streams_listen(&args[1..]); }
    if args.first().map(String::as_str) == Some("trace") { return streams_trace(&args[1..]); }
    if args.first().map(String::as_str) != Some("probe") { err_msg(vc_core::msg!("cli-streams-usage")); return ExitCode::from(2); }
    let (mut host, mut kinds, mut secs, mut password, mut multicast, mut record, mut count) = (None::<String>, vec![], 5u64, None::<String>, false, None::<String>, 400usize);
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--host" => host = it.next().cloned(), "--password" => password = it.next().cloned(), "--multicast" => multicast = true, "--record" => record = it.next().cloned(),
            "--video" => kinds.push(StreamKind::Video), "--audio" => kinds.push(StreamKind::Audio), "--debug" => kinds.push(StreamKind::Debug),
            "--secs" => match it.next().and_then(|v| v.parse::<u64>().ok()) { Some(n) => secs = n.clamp(2, 60), None => return fail_msg(vc_core::msg!("cli-secs-needs-number")) },
            "--count" => match it.next().and_then(|v| v.parse::<usize>().ok()) { Some(n) => count = n, None => return fail_msg(vc_core::msg!("cli-count-needs-number")) },
            s => { return help::unknown_option("streams", s); }
        }
    }
    let Some(host) = host.filter(|h| !h.trim().is_empty()) else { return fail_msg(vc_core::msg!("cli-host-missing")) };
    if kinds.is_empty() { kinds = vec![StreamKind::Video, StreamKind::Audio]; }
    if kinds.contains(&StreamKind::Video) && kinds.contains(&StreamKind::Debug) { return fail_msg(vc_core::msg!("cli-streams-video-debug-conflict")); }
    let st = vc_ide::settings::effective();
    let pw = password.or_else(|| st.machines.iter().find(|m| m.kind == "ultimate64" && m.host.trim() == host).map(|m| m.password.clone())).filter(|p| !p.is_empty());
    let mut opts = StreamOptions::from_settings(&st); opts.multicast |= multicast;
    let mut svc = StreamService::new(&host, pw.as_deref(), opts);
    let recs: std::sync::Arc<std::sync::Mutex<Vec<(StreamKind, Vec<u8>, usize)>>> = Default::default();
    for &k in &kinds {
        let r = recs.clone(); let rec = record.is_some();
        let sink: vc_emu::streams::Sink = std::sync::Arc::new(move |d: &[u8], _| {
            if !rec { return; }
            if let Ok(mut g) = r.lock() { let n = match g.iter_mut().find(|x| x.0 == k) { Some(x) => x, None => { g.push((k, vec![], 0)); g.last_mut().unwrap() } }; if n.2 < count { n.1.extend_from_slice(&(d.len() as u16).to_le_bytes()); n.1.extend_from_slice(d); n.2 += 1; } }
        });
        if let Err(e) = svc.start(k, sink) { svc.stop_all(); return fail(format!("{}: {e}", k.label())); }
        if let Some(t) = svc.status(k).target { println!("{}", vc_i18n::t!("cli-streams-target", label = k.label(), target = t.to_string())); }
    }
    let t0 = std::time::Instant::now();
    while t0.elapsed() < std::time::Duration::from_secs(secs) {
        std::thread::sleep(std::time::Duration::from_millis(100)); svc.poll();
        if kinds.iter().all(|&k| matches!(svc.state(k), StreamState::Error(_))) { break; }
    }
    svc.poll();
    let mut bad = false;
    for &k in &kinds {
        let s = svc.status(k);
        let secs_f = t0.elapsed().as_secs_f64().max(0.001);
        match &s.state {
            StreamState::Error(e) => { bad = true; println!("{}", vc_i18n::t!("cli-streams-kind-error", label = k.label(), e = &e)); }
            state => println!("{}", vc_i18n::t!("cli-stream-stats", label = k.label(), state = format!("{state:?}"), packets = &s.packets, rate = vc_i18n::num(s.packets as f64 / secs_f, 0), mbit = vc_i18n::num(s.mbit_per_s as f64, 2), lost = &s.lost, percent = vc_i18n::num(s.loss_percent as f64, 2), rejected = &s.rejected, restarts = &s.restarts)),
        }
    }
    svc.stop_all();
    if let Some(dir) = record {
        if let Err(e) = std::fs::create_dir_all(&dir) { return fail_msg(vc_core::msg!("cli-dir-error", dir = &dir, e = &e)); }
        for (k, bytes, n) in recs.lock().map(|g| g.clone()).unwrap_or_default() { let f = Path::new(&dir).join(format!("{}.bin", k.rest_name())); match std::fs::write(&f, bytes) { Ok(()) => println!("{}", vc_i18n::t!("cli-streams-recorded", n = &n, path = f.display())), Err(e) => return fail(format!("{}: {e}", f.display())) } }
    }
    println!("{}", vc_i18n::t!("cli-streams-all-stopped"));
    if bad { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// `vcc streams shot --host HOST --out DATEI.png [--password PW] [--multicast]`: ein Bild der Ultimate als PNG (Strom wird gestartet und wieder gestoppt)
fn streams_shot(args: &[String]) -> ExitCode {
    use vc_ide::streams::{StreamOptions};
    let (mut host, mut out, mut password, mut multicast) = (None::<String>, None::<String>, None::<String>, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() { "--host" => host = it.next().cloned(), "--out" => out = it.next().cloned(), "--password" => password = it.next().cloned(), "--multicast" => multicast = true,
            s => { return help::unknown_option("streams", s); } }
    }
    let (Some(host), Some(out)) = (host.filter(|h| !h.trim().is_empty()), out.filter(|o| !o.trim().is_empty())) else { return fail_msg(vc_core::msg!("cli-streams-shot-usage")) };
    let st = vc_ide::settings::effective();
    let pw = password.or_else(|| st.machines.iter().find(|m| m.kind == "ultimate64" && m.host.trim() == host).map(|m| m.password.clone())).filter(|p| !p.is_empty());
    let mut opts = StreamOptions::from_settings(&st); opts.multicast |= multicast;
    match vc_ide::livebild::grab_frame(&host, pw.as_deref(), opts, std::time::Duration::from_secs(5)) {
        Ok(f) => match vc_ide::livebild::save_png(&f, Path::new(&out)) { Ok(()) => { println!("{}×{} → {out}", f.width, f.height); ExitCode::SUCCESS } Err(e) => fail(e) },
        Err(e) => fail(e),
    }
}

/// Adresse als `$1234`, `0x1234`, Dezimalzahl oder Symbol der `.vcdbg`
fn parse_addr(s: &str, syms: &std::collections::HashMap<String, i64>) -> Result<u16, String> {
    let t = s.trim();
    let v = if let Some(h) = t.strip_prefix('$').or_else(|| t.strip_prefix("0x")) { i64::from_str_radix(h, 16).map_err(|_| vc_i18n::t!("cli-address-invalid", s = &s))? }
        else if let Ok(n) = t.parse::<i64>() { n }
        else { *syms.get(t).ok_or_else(|| vc_i18n::t!("cli-address-unknown", s = &s))? };
    u16::try_from(v).map_err(|_| vc_i18n::t!("cli-address-out-of-range", s = &s))
}

/// `vcc streams trace --host HOST [--secs N] ...`: Debug-Strom (Modus „6510 Only“) auswerten und zusammenfassen.
fn streams_trace(args: &[String]) -> ExitCode {
    use vc_emu::streams::trace::{Analyzer, MeasureSpec};
    use vc_ide::streams::{StreamOptions, StreamState};
    use vc_ide::trace::{format_record, op_table, LineMap, TraceSession, TraceWriter};
    let (mut host, mut secs, mut password, mut multicast, mut dbg, mut out, mut tail, mut cpu_id, mut data_dir) = (None::<String>, 5u64, None::<String>, false, None::<String>, None::<String>, 10usize, "6510".to_string(), None::<String>);
    let mut measures: Vec<String> = vec![];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--host" => host = it.next().cloned(), "--password" => password = it.next().cloned(), "--multicast" => multicast = true, "--debug" => dbg = it.next().cloned(), "--out" => out = it.next().cloned(),
            "--cpu" => match it.next() { Some(c) => cpu_id = c.clone(), None => return fail_msg(vc_core::msg!("cli-cpu-needs-id")) }, "--data-dir" => data_dir = it.next().cloned(),
            "--measure" => match it.next() { Some(m) => measures.push(m.clone()), None => return fail_msg(vc_core::msg!("cli-measure-needs-pair")) },
            "--secs" => match it.next().and_then(|v| v.parse::<u64>().ok()) { Some(n) => secs = n.clamp(1, 300), None => return fail_msg(vc_core::msg!("cli-secs-needs-number")) },
            "--tail" => match it.next().and_then(|v| v.parse::<usize>().ok()) { Some(n) => tail = n, None => return fail_msg(vc_core::msg!("cli-tail-needs-number")) },
            s => { return help::unknown_option("streams", s); }
        }
    }
    let Some(host) = host.filter(|h| !h.trim().is_empty()) else { return fail_msg(vc_core::msg!("cli-host-missing")) };
    let Some(data) = find_data_dir(data_dir) else { return fail_msg(vc_core::msg!("cli-no-data-dir-short")) };
    let cpu = match vc_target::load_cpu(&data, &cpu_id) { Ok(c) => c, Err(e) => return fail(format!("CPU {cpu_id}: {e}")) };
    if cpu_id == "45gs02" { return fail_msg(vc_core::msg!("cli-trace-45gs02")); }
    let dbg = match dbg.map(|f| std::fs::read_to_string(&f).map_err(|e| vc_i18n::t!("cli-file-error-f", f = &f, e = &e)).and_then(|t| vc_ide::tests::static_checks::Dbg::parse(&t))) { Some(Ok(d)) => Some(d), Some(Err(e)) => return fail(e), None => None };
    let map = dbg.as_ref().map(LineMap::from_dbg);
    let syms = dbg.as_ref().map(|d| d.symbols.clone()).unwrap_or_default();
    let mut an = Analyzer::new(op_table(&cpu));
    if let Some(m) = &map { an.set_entry_points(m.entry_points()); }
    let mut ids = vec![];
    for m in &measures {
        let Some((a, b)) = m.split_once(':') else { return fail_msg(vc_core::msg!("cli-measure-invalid", m = &m)) };
        match (parse_addr(a, &syms), parse_addr(b, &syms)) {
            (Ok(from), Ok(to)) => ids.push(an.add_measure(MeasureSpec { name: m.clone(), from, to })),
            (Err(e), _) | (_, Err(e)) => return fail(e),
        }
    }
    let writer = match &out { Some(f) => match TraceWriter::create(Path::new(f), map.clone()) { Ok(w) => { an.set_recorder(Some(w.tx.clone())); Some(w) } Err(e) => return fail(e) }, None => None };
    let st = vc_ide::settings::effective();
    let pw = password.or_else(|| st.machines.iter().find(|m| m.kind == "ultimate64" && m.host.trim() == host).map(|m| m.password.clone())).filter(|p| !p.is_empty());
    let mut opts = StreamOptions::from_settings(&st); opts.multicast |= multicast;
    let mut sess = match TraceSession::start(&host, pw.as_deref(), opts, an) { Ok(s) => s, Err(e) => return fail_msg(vc_core::msg!("cli-trace-failed", e = &e)) };
    println!("{}", vc_i18n::t!("cli-trace-started", secs = &secs));
    let t0 = std::time::Instant::now();
    let mut err = None;
    while t0.elapsed() < std::time::Duration::from_secs(secs) {
        std::thread::sleep(std::time::Duration::from_millis(100)); sess.poll();
        if let StreamState::Error(e) = sess.service().state(vc_emu::streams::StreamKind::Debug) { err = Some(e.clone()); break; }
    }
    let secs_f = t0.elapsed().as_secs_f64();
    let an = sess.analyzer().clone();
    let st = sess.stop();
    if let Some(e) = err { err_msg(vc_core::msg!("cli-streams-error", e = &e)); if let Some(w) = writer { let _ = w.finish(); } return ExitCode::FAILURE; }
    let (recent, results, io, cov) = { let g = an.lock().unwrap(); (g.recent(tail), ids.iter().filter_map(|&i| g.measure(i)).collect::<Vec<_>>(), g.recent_io(8), g.coverage()) };
    if let Some(u) = &st.unsupported { eprintln!("{u}"); if let Some(w) = writer { let _ = w.finish(); } return ExitCode::FAILURE; }
    println!("{}", vc_i18n::t!("cli-trace-heading", secs = vc_i18n::num((secs_f) as f64, 1)));
    println!("  {}", vc_i18n::t!("cli-trace-cycles", cycles = &st.cycles, stalls = &st.stall_cycles, rate = vc_i18n::num((st.cycles as f64 / secs_f / 1e6) as f64, 2), insns = &st.insns, irqs = &st.irqs, nmis = &st.nmis, resets = &st.resets));
    println!("  {}", vc_i18n::t!("cli-trace-sync", state = if st.synced { vc_i18n::t!("cli-trace-synced") } else { vc_i18n::t!("cli-trace-not-synced") }, desyncs = &st.desyncs, syncs = &st.syncs));
    println!("  {}", vc_i18n::t!("cli-trace-packets", packets = &st.packets, lost = &st.lost_packets, percent = vc_i18n::num((st.loss_percent()) as f64, 2), epochs = &st.epochs));
    println!("  {}", vc_i18n::t!("cli-trace-coverage", covered = &st.covered, io_writes = &st.io_writes));
    if let Some(m) = &map {
        let un = m.uncovered(|a| cov[(a >> 6) as usize] >> (a & 63) & 1 != 0);
        println!("  {}", vc_i18n::t!("cli-trace-unreached", count = un.len(), hint = if un.is_empty() { String::new() } else { format!(" {}", vc_i18n::t!("cli-trace-examples-below")) }));
        for (f, l, a) in un.iter().take(10) { println!("    {}:{l}  ${a:04X}", Path::new(f).file_name().map_or(f.clone(), |n| n.to_string_lossy().into_owned())); }
    }
    for r in &results { println!("  {}", r.hint()); }
    if !io.is_empty() { println!("  Letzte SID/VIC-Schreibzugriffe (Zyklus: Adresse=Wert):"); for w in &io { println!("    {:>10}: ${:04X}=${:02X}", w.cycle, w.addr, w.value); } }
    if !recent.is_empty() { println!("  {}", vc_i18n::t!("cli-trace-last-instructions", count = recent.len())); for r in &recent { println!("    {}", format_record(r, map.as_ref())); } }
    if let Some(w) = writer { match w.finish() { Ok(n) => println!("  {}", vc_i18n::t!("cli-trace-recorded", n = &n, path = out.as_deref().unwrap_or(""))), Err(e) => return fail(e) } if st.recorder_dropped > 0 { println!("  {}", vc_i18n::t!("cli-trace-dropped", count = &st.recorder_dropped)); } }
    println!("{}", vc_i18n::t!("cli-streams-all-stopped"));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vice_test_args_follow_project_memory_setting() {
        let data = find_data_dir(None).expect("data/");
        let proj = data.parent().unwrap().join("examples/vic20/prallball/prallball.asm");
        if !proj.exists() { return; }
        let a = vice_test_args(&data, &proj, "vic20", vec!["-x".into()]);
        assert_eq!(a, vec!["-x".to_string(), "-memory".into(), "8k".into()]);
        // anderes Ziel: unverändert
        assert_eq!(vice_test_args(&data, &proj, "c64", vec!["-x".into()]), vec!["-x".to_string()]);
    }

    #[test]
    fn lang_option_wird_aus_den_argumenten_genommen() {
        let v = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(lang_option(v(&["build", "--lang", "en", "x"])).unwrap(), (v(&["build", "x"]), Some(vc_i18n::Lang::En)));
        assert_eq!(lang_option(v(&["--lang=de", "help"])).unwrap(), (v(&["help"]), Some(vc_i18n::Lang::De)));
        assert_eq!(lang_option(v(&["asm", "a.asm"])).unwrap(), (v(&["asm", "a.asm"]), None));
        assert!(lang_option(v(&["--lang"])).is_err() && lang_option(v(&["--lang", "fr"])).is_err());
    }

    #[test]
    fn paths_are_shown_with_forward_slashes() {
        assert_eq!(show_path(Path::new(r"examples/c64/tests_demo\tests_demo.vctest")), "examples/c64/tests_demo/tests_demo.vctest");
        assert_eq!(show_path(Path::new(r"C:\proj\a.vctest")), "C:/proj/a.vctest");
        assert_eq!(show_path(Path::new("a/b")), "a/b");
    }
}
