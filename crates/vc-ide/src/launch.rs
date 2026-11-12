// SPDX-License-Identifier: GPL-3.0-or-later
//! Startet ein gebautes Programm im passenden externen Emulator (VICE, atari800, Xemu).
use std::path::Path;
use std::process::Child;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Es läuft höchstens ein per F5 gestarteter Emulator; ein neuer Start beendet den alten.
static RUNNING: Mutex<Option<Child>> = Mutex::new(None);
static GENERATION: AtomicU64 = AtomicU64::new(0);
/// Fehler aus Starts, die im Hintergrund laufen (MEGA65/Xemu: Programm wird erst nach dem Hochfahren geladen); die Oberfläche holt sie ab
static ASYNC_ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Fehlermeldungen aus Hintergrundstarts abholen (leer, wenn nichts schiefging)
pub fn take_errors() -> Vec<String> { std::mem::take(&mut *ASYNC_ERRORS.lock().unwrap_or_else(|e| e.into_inner())) }

fn kill(mut c: Child) { let _ = c.kill(); let _ = c.wait(); }

/// Beendet den zuletzt gestarteten Emulator (falls er noch läuft); liefert, ob einer beendet wurde.
pub fn stop_running() -> bool {
    GENERATION.fetch_add(1, Ordering::SeqCst);        // ein noch startender Xemu verwirft sich selbst
    let old = RUNNING.lock().ok().and_then(|mut g| g.take());
    match old { Some(c) => { kill(c); true } None => false }
}

fn register(c: Child) { if let Ok(mut g) = RUNNING.lock() { *g = Some(c); } else { kill(c); } }

fn which(name: &str, extra: &[&str]) -> Option<String> { vc_emu::find_program(name, extra) }

/// Altirra zum Starten von Atari-Programmen: Pfad aus den Einstellungen oder automatisch gefunden, sofern die Einstellung
/// `default_for_atari` nicht `atari800` verlangt; `None` heißt atari800.
pub fn altirra_choice() -> Option<String> {
    let em = crate::settings::effective().emulator_for("atari8");
    if em.kind != "altirra" || !crate::emulators::enabled("altirra") { return None; }
    em.path.or_else(|| detect("altirra"))
}

/// Programm eines aufgelösten Emulators: Pfad aus den Einstellungen, sonst Suche (PATH und übliche Ordner)
fn resolve(em: &crate::targets::ResolvedEmu, extra: &[&str]) -> Option<String> { em.path.clone().or_else(|| which(&em.kind, extra)) }

/// Pfad, den die automatische Suche für `kind` (`x64sc`, `x128`, `xplus4`, `xvic`, `atari800`, `xemu`) findet – ohne Einstellungen (für die Anzeige im Einstellungsdialog)
pub fn detect(kind: &str) -> Option<String> {
    // gewählte Version, dann System (PATH, Programmordner), zuletzt die von VintageCoder portabel installierte Version
    detect_in(kind, vc_emu::find_program)
}

/// Wie `detect`, aber nur, was das System selbst bietet (PATH, übliche Programmordner), ohne die portablen Installationen
pub fn detect_system(kind: &str) -> Option<String> {
    detect_in(kind, vc_emu::find_program_system)
}

fn detect_in(kind: &str, which: fn(&str, &[&str]) -> Option<String>) -> Option<String> {
    if !crate::emulators::enabled(kind) { return None; }     // Anbindung nicht im Release gewählt: nichts finden
    // Programmnamen, Suchordner und Umgebungsvariable stehen im Emulator-Katalog (`data/emulators/catalog.json`, Feld `detect`)
    crate::emulators::detect_by(kind, which)
}

/// Videonorm des Atari-Targets aus der Projektdatei im Ordner `root` (Standard: pal)
pub fn atari_variant(root: &Path) -> String {
    let found = std::fs::read_dir(root).ok().and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |e| e == "vcprj")));
    let meta = found.and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
    meta.and_then(|m| m["targets"].as_array().and_then(|t| t.iter().find(|x| x["system"] == "atari8").and_then(|x| x["variant"].as_str().map(String::from)))).unwrap_or_else(|| "pal".into())
}

/// Startet `path` (absoluter Pfad des Artefakts) für `target`; liefert die Kommandozeile
/// VICE-Argument für ein VC-20-Modulabbild (Endung `a0`, `20`, `40`, `60`, `b0` = Speicherblock), sonst `None`
pub fn vic20_cart_arg(path: &Path) -> Option<[String; 2]> {
    let flag = match path.extension()?.to_str()?.to_ascii_lowercase().as_str() { "a0" => "-cartA", "20" => "-cart2", "40" => "-cart4", "60" => "-cart6", "b0" => "-cartB", _ => return None };
    Some([flag.into(), path.to_string_lossy().into_owned()])
}

/// Emulator-Argumente, die vom Projekt abhängen: C16 → `-model c16`, VC-20 → `-memory <Ausbau>` (Einstellung `memory` des Targets in der Projektdatei)
pub fn project_args(data: &Path, root: &Path, target: &str) -> Vec<String> {
    if target == "c16" { return vec!["-model".into(), "c16".into()]; }
    if target != "vic20" { return vec![]; }
    let Ok(mut t) = vc_target::load_target(data, target) else { return vec![] };
    if t.apply_memory(crate::projfile::target_setting(root, target, "memory").as_deref()).is_err() { return vec![]; }
    let id = crate::projfile::target_setting(root, target, "memory").filter(|m| t.memory.iter().any(|c| &c.id == m)).or(t.memory_default.clone());
    t.memory.iter().find(|c| Some(&c.id) == id.as_ref()).filter(|c| !c.vice.is_empty()).map(|c| vec!["-memory".into(), c.vice.clone()]).unwrap_or_default()
}

/// Projektordner zu einer Datei: der nächste Ordner darüber mit einer Projektdatei, sonst der Ordner der Datei
pub fn project_root_of(path: &Path) -> std::path::PathBuf {
    let mut d = path.parent();
    while let Some(x) = d { if crate::projfile::find(x).is_some() { return x.to_path_buf(); } d = x.parent(); }
    path.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Startet `path` (absoluter Pfad des Artefakts) für `target`; `extra` = Argumente aus [`project_args`]; liefert die Kommandozeile
pub fn spawn(path: &Path, target: &str, atari_variant: &str, extra: &[String]) -> Result<String, String> {
    stop_running();
    let generation = GENERATION.load(Ordering::SeqCst);
    // Windows: der Artefaktpfad kommt als `<Projekt>\build/x.xex`; Altirra liest ein "/" als Schalter ("Cannot open file …\build"), daher einheitlich mit "\"
    #[cfg(windows)]
    let path_native: std::path::PathBuf = path.components().collect();
    #[cfg(windows)]
    let path = path_native.as_path();
    let rel = path.to_string_lossy().into_owned();
        // Atari: Altirra (Windows) ist der Standard, wenn gefunden; sonst atari800 (Einstellung `default_for_atari` kann das ändern)
        let settings = crate::settings::effective();
        let ctx = crate::args::ArgCtx::from_launch(&project_root_of(path), path, target, atari_variant);
        let altirra = if target == "atari8" { altirra_choice() } else { None };
        let vice = settings.vice_for(target);
        // abgewählte Anbindung: kein Start (Altirra ist hier schon ausgeschlossen, siehe `altirra_choice`)
        if altirra.is_none() { crate::emulators::require(match target { "atari8" => "atari800", "mega65" => "xemu", _ => vice.kind.as_str() })?; }
        let (cmd, args): (String, Vec<String>) = match target {
            "atari8" if altirra.is_some() => {
                let pal = if atari_variant == "ntsc" { "/ntsc" } else { "/pal" };
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
                let mut a: Vec<String> = vec!["/tempprofile".into(), "/defprofile:xl".into(), pal.into()];
                match ext.as_str() {
                    "car" | "rom" => a.extend(["/nobasic".into(), "/cart".into(), rel.clone()]),
                    "bas" => a.extend(["/basic".into(), "/runbas".into(), rel.clone()]),
                    _ => a.extend(["/nobasic".into(), "/run".into(), rel.clone()]),
                }
                (altirra.clone().unwrap_or_default(), a)
            }
            "atari8" => (resolve(&settings.emulator_of_kind("atari8", "atari800"), &[]).ok_or_else(|| format!("atari800 nicht gefunden ({} oder Pfad unter Einstellungen eintragen)", vc_emu::install_hint("atari800")))?,
                         if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("car") || e.eq_ignore_ascii_case("rom")) { vec!["-xl".into(), (if atari_variant.to_string() == "ntsc" { "-ntsc" } else { "-pal" }).into(), "-nobasic".into(), "-cart".into(), path.to_string_lossy().into_owned()] } else { vec!["-xl".into(), (if atari_variant.to_string() == "ntsc" { "-ntsc" } else { "-pal" }).into(), "-nobasic".into(), "-run".into(), path.to_string_lossy().into_owned()] }),
            "mega65" => {
                // Open-ROMs-BASIC versteht `RUN"*"` nicht (Autoload von Diskette scheitert): Xemu starten, das Programm über den Monitor laden
                // und am SYS-Ziel losrennen lassen; der Monitor-Thread beendet sich danach, Xemu läuft weiter.
                // unter Windows nur zur Prüfung gelesen (Xemu lädt das Programm selbst)
                #[cfg_attr(windows, allow(unused_variables))]
                let prg = std::fs::read(path).map_err(|e| format!("{rel}: {e}"))?;
                let xemu = settings.emulator_for("mega65");
                let rplan = crate::roms::plan("xemu", &crate::roms::store(Some(&project_root_of(path)), Some(&xemu)), &[]);
                if let Some(e) = rplan.error.clone() { return Err(e); }
                let rom = rplan.xemu_rom.clone().map(|p| p.to_string_lossy().into_owned()).ok_or("MEGA65-ROM nicht gefunden")?;
                let rom_lines = rplan.lines();
                // Windows: Das Laden über den Monitor startet mit dem Original-ROM nicht (bei $2000-$7FFF liegt das ROM-BASIC darüber); Xemu lädt das PRG selbst und startet es (`-prg`, C65-Modus)
                #[cfg(windows)]
                let xargs = { let mut a = xemu.launch_args(&ctx)?; a.extend(["-prg".to_string(), path.to_string_lossy().into_owned(), "-prgmode".to_string(), "65".to_string()]); a };
                #[cfg(not(windows))]
                let xargs = xemu.launch_args(&ctx)?;
                let xexe = xemu.path.clone();
                // Einstiegsadresse nur dort, wo das Programm über den Monitor geladen wird (unter Windows lädt Xemu selbst)
                #[cfg(not(windows))]
                let load = prg.get(0).copied().unwrap_or(1) as u16 | (prg.get(1).copied().unwrap_or(0x20) as u16) << 8;
                #[cfg(not(windows))]
                let entry = crate::debug::sys_target(&prg).unwrap_or(load);
                std::thread::spawn(move || {
                    let r = vc_emu::Mega65::launch_xemu_rom(xexe.as_deref(), &rom, &xargs).and_then(|mut m| { #[cfg(not(windows))] { use vc_emu::Backend; m.load_prg(&prg, entry)?; m.cont()?; } let child = m.take_child(); std::mem::forget(m); Ok(child) });   // forget: Drop würde den Monitor-Dialog stören
                    match r {
                        // Wurde inzwischen ein anderer Emulator gestartet oder beendet, verfällt dieser Xemu
                        Ok(Some(c)) if GENERATION.load(Ordering::SeqCst) != generation => kill(c),
                        Ok(Some(c)) => register(c),
                        Ok(None) => {}
                        Err(e) => { eprintln!("MEGA65-Start: {e}"); ASYNC_ERRORS.lock().unwrap_or_else(|x| x.into_inner()).push(format!("MEGA65-Start fehlgeschlagen: {e}")); }
                    }
                });
                #[cfg(windows)]
                return Ok(with_lines("Xemu (Programm wird mit -prg im C65-Modus geladen und gestartet)".to_string(), &rom_lines));
                #[cfg(not(windows))]
                return Ok(with_lines(format!("Xemu (Programm wird über den Monitor geladen, Start bei ${entry:04X}; dauert ca. 15 s)"), &rom_lines));
            }
            _ => { let prog = vice.kind.as_str(); (resolve(&vice, &crate::emulators::search_dirs(&vice.kind).iter().map(String::as_str).collect::<Vec<_>>()).ok_or(format!("{prog} (VICE) nicht gefunden"))?,
                  if let Some(a) = vic20_cart_arg(path).filter(|_| target == "vic20") { a.to_vec() } else if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("crt")) { vec!["-cartcrt".into(), path.to_string_lossy().into_owned()] } else { vec!["-autostart".into(), path.to_string_lossy().into_owned()] }) }
        };
        // zusätzliche Argumente und ROM aus den Einstellungen
        let mut args = args;
        let mut rom_lines: Vec<String> = vec![];
        if altirra.is_some() { rom_lines = crate::roms::plan("altirra", &crate::roms::store(None, None), &[]).lines(); args.extend(settings.emulator_for("atari8").launch_args(&ctx)?); }
        else if target == "atari8" {
            let a8 = settings.emulator_of_kind("atari8", "atari800");
            let user = a8.launch_args(&ctx)?;
            let rp = crate::roms::plan("atari800", &crate::roms::store(Some(&project_root_of(path)), Some(&a8)), &user);
            rom_lines = rp.lines();
            args.extend(rp.args); args.extend(user);
        } else {
            let user = vice.launch_args(&ctx)?;
            let rp = crate::roms::plan(&vice.kind, &crate::roms::store(Some(&project_root_of(path)), Some(&vice)), &user);
            rom_lines = rp.lines();
            args.extend(rp.args); args.extend(extra.iter().cloned()); args.extend(user);
        }
        let child = std::process::Command::new(&cmd).args(&args).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn().map_err(|e| format!("{cmd}: {e}"))?;
        register(child);
    Ok(with_lines(format!("{cmd} {}", args.join(" ")), &rom_lines))
}

/// Meldung des Starts um die ROM-Zeilen (gewählte ROMs mit Herkunft, Warnungen) ergänzen
fn with_lines(head: String, lines: &[String]) -> String {
    if lines.is_empty() { head } else { format!("{head}\n{}", lines.join("\n")) }
}

/// Legt ein Diskettenimage im Standard-Emulator des Systems ein (Laufwerk 8 bei VICE, Laufwerk 1 bei atari800/Altirra) und startet ihn; ein
/// früher gestarteter Emulator wird beendet. Es wird nichts geladen: bei Commodore tippt man `LOAD"*",8,1`, der Atari bootet die Diskette selbst.
pub fn insert_disk(image: &Path, target: &str, atari_variant: &str) -> Result<String, String> {
    if target == "mega65" { return Err("MEGA65: Einlegen im Emulator ist noch nicht umgesetzt".into()); }
    stop_running();
    let settings = crate::settings::effective();
    let img = image.to_string_lossy().into_owned();
    let ctx = crate::args::ArgCtx::from_launch(&project_root_of(image), image, target, atari_variant);
    let (cmd, mut args): (String, Vec<String>) = if target == "atari8" {
        if let Some(a) = altirra_choice() {
            let mut v = vec!["/tempprofile".into(), "/defprofile:xl".into(), (if atari_variant == "ntsc" { "/ntsc" } else { "/pal" }).into(), "/nobasic".into(), "/disk".into(), img];
            v.extend(settings.emulator_for("atari8").launch_args(&ctx)?);
            (a, v)
        } else {
            crate::emulators::require("atari800")?;
            let a8 = settings.emulator_of_kind("atari8", "atari800");
            let prog = resolve(&a8, &[]).ok_or_else(|| format!("atari800 nicht gefunden ({} oder Pfad unter Einstellungen eintragen)", vc_emu::install_hint("atari800")))?;
            let mut v: Vec<String> = vec!["-xl".into(), (if atari_variant == "ntsc" { "-ntsc" } else { "-pal" }).into(), "-nobasic".into()];
            let user = a8.launch_args(&ctx)?;
            v.extend(crate::roms::plan("atari800", &crate::roms::store(Some(&project_root_of(image)), Some(&a8)), &user).args);
            v.extend(user);
            v.push(img);       // ein Imagename ohne Schalter wird in Laufwerk 1 gelegt und gebootet
            (prog, v)
        }
    } else {
        let vice = settings.vice_for(target);
        crate::emulators::require(&vice.kind)?;
        let prog = resolve(&vice, &crate::emulators::search_dirs(&vice.kind).iter().map(String::as_str).collect::<Vec<_>>()).ok_or(format!("{} (VICE) nicht gefunden", vice.kind))?;
        let mut v: Vec<String> = vec!["-8".into(), img];
        let user = vice.launch_args(&ctx)?;
        v.extend(crate::roms::plan(&vice.kind, &crate::roms::store(Some(&project_root_of(image)), Some(&vice)), &user).args);
        v.extend(user);
        (prog, v)
    };
    args.retain(|a| !a.is_empty());
    let child = std::process::Command::new(&cmd).args(&args).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| format!("{cmd}: {e}"))?;
    register(child);
    Ok(format!("{cmd} {}", args.join(" ")))
}

/// Startet `path` auf einer echten Maschine aus den Einstellungen: Ultimate 64 (REST: Reset, Laden, RUN) bzw. MEGA65 (Monitor über die serielle Schnittstelle).
pub fn run_on_machine(m: &crate::settings::Machine, path: &Path, target: &str) -> Result<String, String> {
    crate::emulators::require_device(&m.kind)?;
    stop_running();
    let prg = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    match m.kind.as_str() {
        "ultimate64" => {
            if target != "c64" { return Err("Die Ultimate-Hardware spielt nur C64-Programme".into()); }
            let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if ext != "prg" && ext != "crt" { return Err("Auf der Ultimate lassen sich nur .prg-Programme und .crt-Module starten (keine Diskette)".into()); }
            let mut u = vc_emu::Ultimate64::connect(m.host.trim(), Some(m.password.as_str()))?;
            let who = if m.name.is_empty() { &m.host } else { &m.name };
            if ext == "crt" {
                // Ein Modul startet immer mit Reset und läuft sofort (run_crt); „nur laden“ gibt es dafür nicht
                u.run_crt(&prg)?;
                return Ok(format!("{who}: Modul gestartet ({} Bytes)", prg.len()));
            }
            if m.reset_before { u.reset()?; }
            if m.run_after { u.run_prg(&prg)?; } else { u.load_prg(&prg)?; }
            Ok(format!("{who}: Programm gestartet ({} Bytes)", prg.len()))
        }
        "mega65" => {
            if target != "mega65" { return Err("Die MEGA65-Hardware spielt nur MEGA65-Programme".into()); }
            // Ethernet (etherload / mega65_ftp), UNGEPRÜFT: eine Diskette wird kopiert und eingebunden, ein Programm geladen und gestartet
            if m.link_for(crate::mega65net::Action::Start) == crate::mega65net::Link::Eth {
                let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
                return if ext == "d81" { crate::mega65net::copy_and_mount(m, path) } else { crate::mega65net::start(m, path) };
            }
            let spec = crate::settings::mega65_spec(m)?;
            let mut mm = vc_emu::Mega65::connect(&spec)?;
            let load = prg.get(0).copied().unwrap_or(1) as u16 | (prg.get(1).copied().unwrap_or(0x20) as u16) << 8;
            let entry = crate::debug::sys_target(&prg).unwrap_or(load);
            { use vc_emu::Backend; mm.load_prg(&prg, entry)?; if m.run_after { mm.cont()?; } }
            std::mem::forget(mm);   // wie beim Xemu: Drop würde den Monitor-Dialog stören
            Ok(format!("{}: Programm gestartet, Start bei ${entry:04X}", if m.name.is_empty() { &m.device } else { &m.name }))
        }
        k => Err(format!("unbekannter Maschinentyp '{k}'")),
    }
}

/// Startet ein Programm von einer Diskette auf einer echten MEGA65 über Ethernet: Diskette auf die SD-Karte kopieren, einbinden, `prg` starten.
/// UNGEPRÜFT (nur gegen Attrappen-Befehle getestet). Über USB gibt es das nicht.
pub fn run_disk_on_machine(m: &crate::settings::Machine, disk: &Path, prg: &Path, target: &str) -> Result<String, String> {
    crate::emulators::require_device(&m.kind)?;
    if m.kind != "mega65" || target != "mega65" { return Err("Diskette auf Hardware starten gibt es nur für den MEGA65".into()); }
    if m.link_for(crate::mega65net::Action::Start) != crate::mega65net::Link::Eth { return Err("Eine Diskette lässt sich nur über Ethernet auf den MEGA65 bringen (Einstellungen → Hardware, Verbindung „Starten“)".into()); }
    stop_running();
    crate::mega65net::start_disk(m, disk, prg)
}

/// Kann die Maschine das Programm von einer Diskette starten (MEGA65 mit Ethernet als Startverbindung)?
pub fn machine_runs_disks(m: &crate::settings::Machine) -> bool {
    m.kind == "mega65" && m.link_for(crate::mega65net::Action::Start) == crate::mega65net::Link::Eth && m.link_for(crate::mega65net::Action::Copy) == crate::mega65net::Link::Eth
}
