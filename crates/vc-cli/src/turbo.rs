// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc turbo …`: Turbo-BASIC-XL-Diskette des Anwenders prüfen, ins Projekt bereitstellen, verkleinern und das Projekt starten.
//! Bauen geschieht mit `vcc build`, Dateien in einem bestehenden ATR ändern mit `vcc disk atr --base`.
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vc_ide::turbo;

/// Rückfrage auf der Fehlerausgabe; `--yes` überspringt sie. Ohne Antwort (kein Terminal) gilt „nein“.
fn confirm(question: &str, yes: bool) -> bool {
    if yes { return true; }
    eprint!("{question} "); let _ = std::io::stderr().flush();
    let mut l = String::new(); let _ = std::io::stdin().lock().read_line(&mut l);
    matches!(l.trim().to_lowercase().as_str(), "j" | "ja" | "y" | "yes")
}

fn project_dir(p: Option<&String>) -> PathBuf { PathBuf::from(p.map(String::as_str).unwrap_or(".")) }

pub fn run(args: &[String]) -> ExitCode {
    let (mut pos, mut json, mut yes, mut reduce, mut anyway, mut replace, mut data_dir) = (vec![], false, false, false, false, false, None::<String>);
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true, "--yes" => yes = true, "--reduce" => reduce = true, "--anyway" => anyway = true, "--replace" => replace = true,
            "--data-dir" => data_dir = it.next().cloned(),
            s if s.starts_with("--") => { return crate::help::unknown_option("turbo", s); }
            s => pos.push(s.to_string()),
        }
    }
    match args.first().map(String::as_str) {
        Some("check") => check(&pos, json),
        Some("provide") => provide(&pos, reduce, anyway, replace, yes),
        Some("reduce") => reduce_cmd(&pos, yes),
        Some("run") => run_cmd(&pos, data_dir),
        _ => crate::help::usage_error("turbo"),
    }
}

fn check(pos: &[String], json: bool) -> ExitCode {
    let Some(p) = pos.first() else { return crate::help::usage_error("turbo") };
    let r = match turbo::check_file(Path::new(p)) { Ok(r) => r, Err(e) => return crate::fail(e) };
    if json { println!("{}", serde_json::to_string_pretty(&r.to_json()).unwrap_or_default()); }
    else { for l in r.lines() { println!("{l}"); } }
    if r.dos2 && r.is_turbo && r.problems.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Anzahl der Dateien, die das Verkleinern entfernen würde
fn reducible_count(bytes: &[u8]) -> usize { vc_disk::Atr::from_bytes(bytes).map(|a| turbo::reducible(&a).len()).unwrap_or(0) }

fn provide(pos: &[String], reduce: bool, anyway: bool, replace: bool, yes: bool) -> ExitCode {
    let Some(src) = pos.first() else { return crate::help::usage_error("turbo") };
    let root = project_dir(pos.get(1));
    if reduce {
        let n = std::fs::read(src).map(|b| reducible_count(&b)).unwrap_or(0);
        let disk = Path::new(src).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        if !confirm(&vc_i18n::t!("turbo-cli-bestaetigen-reduzieren", disk = format!("{}/{disk}", turbo::DIR), count = n), yes) { return crate::fail_msg(vc_core::msg!("turbo-cli-abgebrochen")); }
    }
    match turbo::provide(&root, Path::new(src), &turbo::ProvideOpts { reduce, anyway, replace }) {
        Ok(p) => {
            for l in p.report.lines() { println!("{l}"); }
            if !p.removed.is_empty() { println!("{}", vc_i18n::t!("turbo-cli-verkleinert", count = p.removed.len(), names = p.removed.join(", "), free = p.free_sectors)); }
            println!("{}", vc_i18n::t!("turbo-cli-bereitgestellt", disk = &p.disk, free = p.free_sectors));
            if p.gitignore_added { println!("{}", vc_i18n::t!("turbo-cli-gitignore")); }
            ExitCode::SUCCESS
        }
        Err(e) => crate::fail(e),
    }
}

fn reduce_cmd(pos: &[String], yes: bool) -> ExitCode {
    let root = project_dir(pos.first());
    let Some(pf) = vc_ide::projfile::find(&root).and_then(|f| vc_ide::projfile::ProjectFile::load(&f).ok()) else { return crate::fail_msg(vc_core::msg!("turbo-keine-projektdatei", path = root.display().to_string())) };
    let Some(disk) = pf.turbo.as_ref().map(|t| t.disk.clone()).filter(|d| !d.is_empty()) else { return crate::fail(vc_i18n::t!("turbo-diskette-fehlt")) };
    let n = std::fs::read(root.join(&disk)).map(|b| reducible_count(&b)).unwrap_or(0);
    if n == 0 { println!("{}", vc_i18n::t!("turbo-cli-nichts-zu-entfernen")); return ExitCode::SUCCESS; }
    if !confirm(&vc_i18n::t!("turbo-cli-bestaetigen-reduzieren", disk = &disk, count = n), yes) { return crate::fail_msg(vc_core::msg!("turbo-cli-abgebrochen")); }
    match turbo::reduce_project_copy(&root) {
        Ok(removed) => {
            let free = std::fs::read(root.join(&disk)).ok().and_then(|b| vc_disk::Atr::from_bytes(&b).ok()).map_or(0, |a| a.free_sectors());
            println!("{}", vc_i18n::t!("turbo-cli-verkleinert", count = removed.len(), names = removed.join(", "), free = free));
            ExitCode::SUCCESS
        }
        Err(e) => crate::fail(e),
    }
}

fn run_cmd(pos: &[String], data_dir: Option<String>) -> ExitCode {
    let root = project_dir(pos.first());
    let Some(data) = crate::find_data_dir(data_dir) else { return crate::fail_msg(vc_core::msg!("cli-no-data-dir")) };
    match turbo::start(&root, &data) {
        Ok(s) => { for l in &s.lines { println!("{l}"); } println!("{}", vc_i18n::t!("turbo-cli-gestartet", command = &s.command)); ExitCode::SUCCESS }
        Err(e) => crate::fail(e),
    }
}
