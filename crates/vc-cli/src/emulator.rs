// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc emulator list|status|install`: Emulatoren aus dem Katalog anzeigen, finden und auf ausdrücklichen Aufruf portabel installieren.
use std::process::ExitCode;
use vc_ide::emulators::{self, Entry};

pub fn run(args: &[String]) -> ExitCode {
    let mut dry = false;
    let mut pos: Vec<&str> = vec![];
    for a in args.iter().skip(1) {
        match a.as_str() {
            "--dry-run" => dry = true,
            s if s.starts_with('-') => return crate::help::unknown_option("emulator", s),
            s => pos.push(s),
        }
    }
    match (args.first().map(String::as_str), pos.as_slice()) {
        (Some("list"), []) if !dry => list(),
        (Some("status"), []) if !dry => { for e in emulators::available() { status(e); } for d in emulators::devices().iter().filter(|d| vc_ide::features::features().link(&d.id)) { device(d); } ExitCode::SUCCESS }
        (Some("status"), [id]) if !dry && emulators::devices().iter().any(|d| d.id == *id) => {
            if !vc_ide::features::features().link(id) { eprintln!("vcc emulator: {}", emulators::not_in_release(id)); return ExitCode::from(1); }
            device(emulators::devices().iter().find(|d| d.id == *id).unwrap()); ExitCode::SUCCESS
        }
        (Some("status"), [id]) if !dry => match lookup(&emulators::parse_spec(id).0) { Ok(e) => { status(e); ExitCode::SUCCESS } Err(c) => c },
        (Some("install"), [spec]) => { let (id, ver) = emulators::parse_spec(spec); match lookup(&id) { Ok(e) => install(e, ver.as_deref(), dry), Err(c) => c } }
        _ => crate::help::usage_error("emulator"),
    }
}

fn lookup(id: &str) -> Result<&'static Entry, ExitCode> {
    // bekannt, aber in diesem Release nicht gewählt: eigene Meldung (Rückgabewert 1 wie bei anderen Fehlern)
    if let Some(e) = emulators::find(id).or_else(|| emulators::for_kind(id)) {
        if let Err(m) = emulators::require(&e.id) { eprintln!("vcc emulator: {m}"); return Err(ExitCode::from(1)); }
        return Ok(e);
    }
    Err({
        let ids: Vec<&str> = emulators::available().iter().map(|e| e.id.as_str()).collect();
        crate::err_msg(vc_core::msg!("cli-emu-unknown", id = id, known = ids.join(", ")));
        ExitCode::from(2)
    })
}

/// Zeile `  Beschriftung  Wert` der Statusanzeige (Beschriftung auf 14 Zeichen gefüllt)
fn row(label: String, value: impl std::fmt::Display) { println!("  {:<13} {value}", label); }

fn mb(b: u64) -> String { format!("{:.1} MB", b as f64 / 1_048_576.0) }

/// Echte Maschine (Gerät): kein Programm zu finden oder zu installieren, nur Rolle und Einstellungsort
fn device(d: &Entry) {
    println!("{} ({})", d.name, d.id);
    row(vc_i18n::t!("cli-emu-label-device"), vc_i18n::t!("cli-emu-device-yes"));
    row(vc_i18n::t!("cli-emu-label-systems"), d.systems.join(", "));
    row(vc_i18n::t!("cli-emu-label-role"), &d.role);
}

fn list() -> ExitCode {
    for e in emulators::available() {
        let vers: Vec<String> = e.packages_here().iter().map(|p| p.version.clone()).collect();
        let auto = if vers.is_empty() { vc_i18n::t!("cli-emu-no-package").to_string() } else { vc_i18n::t!("cli-emu-portable-versions", versions = vers.join(", ")) };
        println!("{:<9} {}  [{}]\n          {}\n          {}\n          {}", e.id, e.name, e.systems.join(", "), e.homepage, e.role, auto);
    }
    for d in emulators::devices().iter().filter(|d| vc_ide::features::features().link(&d.id)) {
        println!("{:<9} {}  [{}]\n          {}\n          {}", d.id, d.name, d.systems.join(", "), vc_i18n::t!("cli-emu-device-list"), d.role);
    }
    ExitCode::SUCCESS
}

fn status(e: &Entry) {
    println!("{} ({})", e.name, e.id);
    row(vc_i18n::t!("cli-emu-label-homepage"), &e.homepage);
    row(vc_i18n::t!("cli-emu-label-license"), &e.license);
    let kind = e.main_kind();
    let src = emulators::sources(kind);
    if src.is_empty() { row(vc_i18n::t!("cli-emu-label-found"), vc_i18n::t!("cli-emu-found-none")); }
    for s in &src { row(vc_i18n::t!("cli-emu-label-found"), vc_i18n::t!("cli-emu-found", label = format!("{:<14}", s.label), path = &s.path)); }
    if let Some(p) = vc_ide::launch::detect(kind) { row(vc_i18n::t!("cli-emu-label-auto"), p); }
    let vers = e.packages_here();
    if vers.is_empty() { row(vc_i18n::t!("cli-emu-label-installable"), vc_i18n::t!("cli-emu-installable-no", hint = e.manual_hint().map(|h| format!(" – {h}")).unwrap_or_default())); }
    for (i, p) in vers.iter().enumerate() { row(vc_i18n::t!("cli-emu-label-installable"), vc_i18n::t!("cli-emu-installable-yes", version = &p.version, size = mb(p.size), default = if i == 0 { format!(" {}", vc_i18n::t!("cli-emu-default-mark")) } else { String::new() }, id = &e.id)); }
}

fn install(e: &Entry, version: Option<&str>, dry: bool) -> ExitCode {
    let pkg = match version { Some(v) => e.package_version(v), None => e.package() };
    if let (Some(v), None) = (version, pkg) {
        let have: Vec<&str> = e.packages_here().iter().map(|p| p.version.as_str()).collect();
        crate::err_msg(vc_core::msg!("cli-emu-version-unknown", name = &e.name, v = &v, available = if have.is_empty() { String::new() } else { format!(" {}", vc_i18n::t!("cli-emu-available-list", versions = have.join(", "))) }));
        return ExitCode::from(1);
    }
    let Some(p) = pkg else {
        crate::err_msg(vc_core::msg!("cli-emu-no-package-for", name = &e.name, hint = e.manual_hint().map(|h| format!(" {h}")).unwrap_or_default(), page = &e.download_page));
        return ExitCode::from(1);
    };
    if dry {
        println!("{}", vc_i18n::t!("cli-emu-install-plan", name = &e.name, version = &p.version, url = &p.url, sha = &p.sha256, size = mb(p.size), dir = emulators::install_dir(&e.id).map(|d| d.join(&p.version).display().to_string()).unwrap_or_default()));
        return ExitCode::SUCCESS;
    }
    match emulators::install(&format!("{}@{}", e.id, p.version), &mut |m| println!("{m}")) {
        Ok(path) => { println!("{}", vc_i18n::t!("cli-emu-program", path = &path)); ExitCode::SUCCESS }
        Err(err) => { crate::err_msg(vc_core::msg!("cli-emu-error", err = &err)); ExitCode::from(1) }
    }
}
