// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc roms …`: ROM-Verzeichnis anzeigen, prüfen, importieren.
use std::path::PathBuf;
use std::process::ExitCode;
use vc_roms::{import, Portable, RomEnv, RomStore, Status};


fn build_store(foreign: bool, project: Option<String>, data_dir: Option<String>) -> RomStore {
    let mut env = RomEnv::from_process();
    env.use_foreign = foreign;
    env.project_dir = project.map(PathBuf::from);
    if env.data_dir.is_none() { env.data_dir = data_dir.map(PathBuf::from).or_else(|| crate::find_data_dir(None)); }
    RomStore::new(env)
}

fn size_text(r: &vc_roms::Resolved) -> String { r.expected.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("/") }

pub fn run(args: &[String]) -> ExitCode {
    let (mut pos, mut json, mut foreign, mut yes, mut project, mut system, mut data_dir, mut from_others) = (vec![], false, false, false, None::<String>, None::<String>, None::<String>, false);
    // emit-known hat eigene Optionen
    if args.first().map(String::as_str) == Some("emit-known") { return emit_known(&args[1..]); }
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true, "--foreign" => foreign = true, "--yes" => yes = true, "--from-others" => from_others = true,
            "--project" => project = it.next().cloned(), "--system" => system = it.next().cloned(), "--data-dir" => data_dir = it.next().cloned(),
            s if s.starts_with("--") => { return crate::help::unknown_option("roms", s); }
            s => pos.push(s.to_string()),
        }
    }
    let store = build_store(foreign, project, data_dir);
    match args.first().map(String::as_str) {
        Some("list") => list(&store, pos.first().map(String::as_str), json),
        Some("check") => check(&store, &pos),
        Some("path") => path(&store, pos.first().map(String::as_str)),
        Some("where") => where_cmd(&store),
        Some("import") => import_cmd(&store, &pos, system.as_deref(), from_others, yes),
        _ => { crate::help::usage_error("roms") }
    }
}

fn list(store: &RomStore, only: Option<&str>, json: bool) -> ExitCode {
    let rows: Vec<_> = store.status_all().into_iter().filter(|r| only.map_or(true, |s| r.system == s)).collect();
    if json {
        let v: Vec<_> = rows.iter().map(|r| serde_json::json!({
            "system": r.system, "role": r.role, "status": r.status.label(), "path": r.path, "origin": r.stage.map(|s| s.label()),
            "size": r.size, "crc32": r.crc32.map(|c| format!("{c:08X}")), "sha1": r.sha1, "label": r.label, "expected": r.expected,
            "others": r.others.iter().map(|f| f.path.to_string_lossy().into_owned()).collect::<Vec<_>>(),
        })).collect();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        return ExitCode::SUCCESS;
    }
    for r in &rows {
        let file = r.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
        println!("{:<7} {:<11} {:<20} {}", r.system, r.role, r.status.label_l(), if file.is_empty() { vc_i18n::t!("cli-roms-expected", size = size_text(r)) } else { format!("{file} [{}]", r.stage.map(|s| s.label()).unwrap_or("")) });
        if let Some(l) = &r.label { println!("{:<41} {l}", ""); }
        if !r.others.is_empty() { println!("{}", vc_i18n::t!("cli-roms-more-files", pad = format!("{:<41}", ""), count = r.others.len())); }
    }
    ExitCode::SUCCESS
}

fn check(store: &RomStore, systems: &[String]) -> ExitCode {
    let ids: Vec<String> = if systems.is_empty() { vc_roms::systems::SYSTEMS.iter().map(|s| s.id.to_string()).collect() } else { systems.to_vec() };
    let mut bad = false;
    for id in ids {
        if vc_roms::systems::system(&id).is_none() { crate::err_msg(vc_core::msg!("cli-roms-unknown-system", id = &id)); return ExitCode::from(2); }
        let missing = store.missing_required(&id);
        if missing.is_empty() { println!("{id}: ok"); } else {
            bad = true;
            println!("{}", vc_i18n::t!("cli-roms-missing", id = &id, roles = missing.join(", ")));
            if !store.env.use_foreign {
                let n = import::foreign_candidates(store).iter().filter(|i| i.system.as_deref() == Some(id.as_str()) && matches!(i.action, import::Action::Copy)).count();
                if n > 0 { println!("  {}", vc_i18n::t!("cli-roms-others-found", n = &n)); }
            }
        }
        for r in store.status_all().iter().filter(|r| r.system == id && r.is_usable()) {
            if r.status != Status::Recognized { println!("  {}: {}{}", r.role, r.status.label_l(), if r.status == Status::UnknownRevision { format!(" {}", vc_i18n::t!("cli-roms-unknown-revision")) } else { String::new() }); }
        }
    }
    if bad { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn path(store: &RomStore, spec: Option<&str>) -> ExitCode {
    let Some((s, r)) = spec.and_then(|s| s.split_once('/')) else { crate::err_msg(vc_core::msg!("cli-roms-path-usage")); return ExitCode::from(2) };
    let res = store.resolve(s, r);
    match (res.is_usable(), &res.path) { (true, Some(p)) => { println!("{}", p.display()); ExitCode::SUCCESS } _ => { eprintln!("{}", res.describe()); ExitCode::FAILURE } }
}

fn where_cmd(store: &RomStore) -> ExitCode {
    let env = &store.env;
    let unknown = || vc_i18n::t!("cli-roms-unknown");
    println!("{}", vc_i18n::t!("cli-roms-install-dir", dir = env.install_dir().map(|p| p.display().to_string()).unwrap_or_else(unknown)));
    println!("{}", vc_i18n::t!("cli-roms-user-dir", dir = env.user_dir().map(|p| p.display().to_string()).unwrap_or_else(unknown)));
    match &env.portable {
        Some(Portable::Active { dir, by_marker }) => println!("{}", vc_i18n::t!("cli-roms-portable-active", dir = dir.display().to_string(), how = if *by_marker { vc_i18n::t!("cli-roms-portable-by-marker") } else { vc_i18n::t!("cli-roms-portable-by-folder") })),
        Some(Portable::Unavailable(why)) => println!("{}", vc_i18n::t!("cli-roms-portable-unavailable", why = &why)),
        Some(Portable::Off { base: Some(b) }) => println!("{}", vc_i18n::t!("cli-roms-portable-off-hint", dir = b.display().to_string())),
        _ => println!("{}", vc_i18n::t!("cli-roms-portable-off")),
    }
    if env.vc_roms.is_some() { println!("{}", vc_i18n::t!("cli-roms-env-set")); }
    println!("{}", vc_i18n::t!("cli-roms-cache-dir", dir = env.cache_dir().map(|p| p.display().to_string()).unwrap_or_else(unknown)));
    println!("\n{}", vc_i18n::t!("cli-roms-searched"));
    for s in store.searched() { println!("  {:<26} {:<48} {}", s.stage.label_l(), s.path.display(), if s.exists { vc_i18n::t!("cli-roms-files", count = &s.files) } else { vc_i18n::t!("cli-roms-absent").into() }); }
    if !env.use_foreign { println!("  {}", vc_i18n::t!("cli-roms-foreign-off")); }
    ExitCode::SUCCESS
}

fn import_cmd(store: &RomStore, pos: &[String], system: Option<&str>, from_others: bool, yes: bool) -> ExitCode {
    let items = if from_others { import::foreign_candidates(store) } else {
        if pos.is_empty() { return crate::help::usage_error("roms"); }
        import::plan(store, &pos.iter().map(PathBuf::from).collect::<Vec<_>>(), system)
    };
    if items.is_empty() { println!("{}", vc_i18n::t!("cli-roms-none")); return ExitCode::SUCCESS; }
    let mut failed = false;
    for it in &items {
        let what = format!("{}/{}", it.system.as_deref().unwrap_or("?"), it.role.as_deref().unwrap_or("?"));
        let name = it.src.display().to_string();
        let src = it.source_label.as_deref().map(|l| format!(" [{l}]")).unwrap_or_default();
        match &it.action {
            import::Action::Copy if yes => match import::execute(it) { Ok(d) => println!("{}", vc_i18n::t!("cli-roms-copied", what = format!("{:<18}", what), name = &name, src = &src, dest = d.display().to_string())), Err(e) => { failed = true; crate::err_msg(vc_core::msg!("cli-roms-copy-error", name = &name, e = &e)) } },
            import::Action::Copy => println!("{}", vc_i18n::t!("cli-roms-would-copy", what = format!("{:<12}", what), name = &name, src = &src, dest = it.dest.as_ref().unwrap().display().to_string(), label = it.label.as_deref().map(str::to_string).unwrap_or_else(|| vc_i18n::t!("cli-roms-unknown-revision-label")))),
            import::Action::AlreadyPresent(p) => println!("{}", vc_i18n::t!("cli-roms-already", what = format!("{what:<17}"), name = &name, path = p.display().to_string())),
            import::Action::Skip(why) => println!("{}", vc_i18n::t!("cli-roms-skipped", name = &name, why = &why)),
        }
    }
    if !yes && items.iter().any(|i| i.action == import::Action::Copy) { println!("\n{}", vc_i18n::t!("cli-roms-nothing-copied")); }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn emit_known(args: &[String]) -> ExitCode {
    let (mut into, mut vice, mut merge_json) = (None::<String>, None::<String>, Vec::<String>::new());
    struct Item { file: String, as_: Option<String>, rev: String, free: bool, default: bool, note: String, sets: Vec<String> }
    let mut items: Vec<Item> = vec![];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--into" => into = it.next().cloned(), "--vice" => vice = it.next().cloned(), "--merge-json" => merge_json.extend(it.next().cloned()),
            "--file" => items.push(Item { file: it.next().cloned().unwrap_or_default(), as_: None, rev: String::new(), free: false, default: false, note: String::new(), sets: vec![] }),
            "--as" | "--rev" | "--note" | "--free" | "--default" | "--set" => {
                let Some(cur) = items.last_mut() else { crate::err_msg(vc_core::msg!("cli-roms-needs-file", a = &a)); return ExitCode::from(2) };
                match a.as_str() { "--as" => cur.as_ = it.next().cloned(), "--rev" => cur.rev = it.next().cloned().unwrap_or_default(), "--note" => cur.note = it.next().cloned().unwrap_or_default(), "--set" => cur.sets.extend(it.next().cloned()), "--free" => cur.free = true, _ => cur.default = true }
            }
            s => { return crate::help::unknown_option("roms", s); }
        }
    }
    let mut table = match &into { Some(f) if std::path::Path::new(f).exists() => match vc_roms::KnownTable::parse(&std::fs::read_to_string(f).unwrap_or_default()) { Ok(t) => t, Err(e) => return crate::fail(e) }, _ => vc_roms::KnownTable { format: 1, combined: vec![], roms: vec![] } };
    let mut errors = 0;
    if let Some(v) = vice { for r in vc_roms::emit::from_vice(std::path::Path::new(&v)) { match r { Ok(k) => table.merge(k), Err(e) => { errors += 1; eprintln!("{e}") } } } }
    // Einträge, die anderswo gemessen wurden (JSON-Liste von Einträgen, wie in known.json)
    for f in merge_json {
        match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Vec<vc_roms::KnownRom>>(&t).map_err(|e| e.to_string())) {
            Ok(v) => for k in v { table.merge(k) },
            Err(e) => { errors += 1; eprintln!("{f}: {e}") }
        }
    }
    for i in items {
        let Some((s, r)) = i.as_.as_deref().and_then(|a| a.split_once('/')) else { crate::err_msg(vc_core::msg!("cli-roms-as-missing", file = &i.file)); errors += 1; continue };
        match vc_roms::emit::entry_with_sets(std::path::Path::new(&i.file), s, r, &i.rev, i.free, i.default, &i.note, &i.sets) { Ok(k) => table.merge(k), Err(e) => { errors += 1; eprintln!("{e}") } }
    }
    let out = table.to_json();
    match into { Some(f) => { if let Err(e) = std::fs::write(&f, out) { return crate::fail_msg(vc_core::msg!("cli-file-error-f", f = &f, e = &e)); } println!("{}", vc_i18n::t!("cli-roms-table-written", count = table.roms.len(), f = &f)); } None => print!("{out}") }
    if errors > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
