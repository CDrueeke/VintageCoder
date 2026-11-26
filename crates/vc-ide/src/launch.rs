// SPDX-License-Identifier: GPL-3.0-or-later
//! Startet ein gebautes Programm im passenden externen Emulator (VICE, atari800, Xemu).
use std::path::Path;
use std::process::Child;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Es läuft höchstens ein per F5 gestarteter Emulator; ein neuer Start beendet den alten.
static RUNNING: Mutex<Option<Run>> = Mutex::new(None);
static GENERATION: AtomicU64 = AtomicU64::new(0);
/// Fehler aus Starts, die im Hintergrund laufen (MEGA65/Xemu: Programm wird erst nach dem Hochfahren geladen); die Oberfläche holt sie ab
static ASYNC_ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Fehlermeldungen aus Hintergrundstarts abholen (leer, wenn nichts schiefging)
pub fn take_errors() -> Vec<String> { std::mem::take(&mut *ASYNC_ERRORS.lock().unwrap_or_else(|e| e.into_inner())) }

/// Windows: Pfade mit dem Präfix `\\?\` (aus `canonicalize`) versteht kein Emulator (VICE: „Cannot open file“), deshalb ohne Präfix übergeben.
fn ohne_praefix(p: &Path) -> std::path::PathBuf { let t = p.to_string_lossy(); match t.strip_prefix(r"\\?\") { Some(r) if !r.starts_with("UNC\\") => std::path::PathBuf::from(r), _ => p.to_path_buf() } }

/// Ein gestarteter Emulator samt seiner Prozessgruppe. Unter macOS/Linux ist `x64sc` oft eine Kette von Shell-Wrappern (`x64sc` → `VICE.sh` → das eigentliche VICE);
/// beendet man nur den ersten Prozess, bleibt VICE offen. Darum gehört zum Emulator der ganze Nachkommenbaum.
struct Run { child: Child, /** Prozessgruppe, wenn der Emulator ihr Anführer ist (siehe [`isolate`]) */ grp: Option<i32>, done: Option<std::process::ExitStatus> }

impl Run {
    fn new(child: Child) -> Run { let grp = own_group(child.id()); Run { child, grp, done: None } }
    fn id(&self) -> u32 { self.child.id() }
    /// Wie `Child::try_wait`, meldet das Ende aber erst, wenn auch alle Nachkommen weg sind (ein Wrapper darf nicht vor dem Emulator „enden“)
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        if self.done.is_none() { self.done = self.child.try_wait()?; }
        match self.done { Some(st) if !self.group_alive() => Ok(Some(st)), _ => Ok(None) }
    }
    fn group_alive(&self) -> bool { self.grp.map_or(false, group_alive) }
}

/// Setzt den Prozess beim Start in eine eigene Prozessgruppe (Unix), damit sich der ganze Baum gezielt beenden lässt, ohne die Gruppe der IDE zu treffen
pub fn isolate(cmd: &mut std::process::Command) -> &mut std::process::Command {
    #[cfg(unix)] { use std::os::unix::process::CommandExt; cmd.process_group(0); }
    cmd
}

#[cfg(unix)]
fn own_group(pid: u32) -> Option<i32> { let p = pid as i32; (unsafe { libc::getpgid(p) } == p).then_some(p) }
#[cfg(not(unix))]
fn own_group(_pid: u32) -> Option<i32> { None }

#[cfg(unix)]
fn group_alive(g: i32) -> bool {
    // kill(-g, 0) gelingt, solange irgendein Mitglied (auch ein Zombie) da ist; Zombies zählen nicht als lebendig
    if unsafe { libc::kill(-g, 0) } != 0 { return false; }
    members(g).iter().any(|(_, st)| !st.starts_with('Z'))
}
#[cfg(not(unix))]
fn group_alive(_g: i32) -> bool { false }

/// `(pid, ppid, pgid, state)` aller Prozesse (`ps`)
#[cfg(unix)]
fn ps_all() -> Vec<(i32, i32, i32, String)> {
    let Ok(o) = std::process::Command::new("ps").args(["-A", "-o", "pid=,ppid=,pgid=,stat="]).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).output() else { return vec![] };
    String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| { let mut f = l.split_whitespace(); Some((f.next()?.parse().ok()?, f.next()?.parse().ok()?, f.next()?.parse().ok()?, f.next()?.to_string())) }).collect()
}
#[cfg(unix)]
fn members(g: i32) -> Vec<(i32, String)> { ps_all().into_iter().filter(|p| p.2 == g).map(|p| (p.0, p.3)).collect() }

/// Alle Nachkommen von `root` (ohne `root`) und, falls `grp` gesetzt ist, alle Mitglieder der Gruppe
#[cfg(unix)]
fn tree(root: i32, grp: Option<i32>) -> Vec<i32> {
    let all = ps_all();
    let mut out: Vec<i32> = vec![root];
    let mut i = 0;
    while i < out.len() { let p = out[i]; for q in &all { if q.1 == p && !out.contains(&q.0) { out.push(q.0); } } i += 1; }
    if let Some(g) = grp { for q in &all { if q.2 == g && !out.contains(&q.0) { out.push(q.0); } } }
    out
}

/// Beendet den Emulator samt aller Nachkommen: erst SIGTERM, nach kurzer Frist SIGKILL
fn kill(mut r: Run) {
    #[cfg(unix)]
    {
        // Baum merken, solange die Eltern-Kind-Kette noch besteht (ein beendeter Wrapper hängt seine Kinder an Prozess 1)
        let pids = tree(r.child.id() as i32, r.grp);
        for p in &pids { unsafe { libc::kill(*p, libc::SIGTERM); } }
        let t0 = std::time::Instant::now();
        while t0.elapsed() < std::time::Duration::from_millis(1500) {
            let _ = r.child.try_wait();
            if pids.iter().all(|p| unsafe { libc::kill(*p, 0) } != 0 || zombie(*p)) { break; }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        for p in &pids { unsafe { libc::kill(*p, libc::SIGKILL); } }
        if let Some(g) = r.grp { unsafe { libc::kill(-g, libc::SIGKILL); } }
    }
    #[cfg(windows)]
    { let _ = std::process::Command::new("taskkill").args(["/T", "/F", "/PID", &r.child.id().to_string()]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status(); }
    let _ = r.child.kill(); let _ = r.child.wait();
}
#[cfg(unix)]
fn zombie(p: i32) -> bool { ps_all().iter().any(|q| q.0 == p && q.3.starts_with('Z')) }

/// Beendet den zuletzt gestarteten Emulator (falls er noch läuft); liefert, ob einer beendet wurde.
pub fn stop_running() -> bool {
    GENERATION.fetch_add(1, Ordering::SeqCst);        // ein noch startender Xemu verwirft sich selbst
    let old = RUNNING.lock().ok().and_then(|mut g| g.take());
    match old { Some(c) => { kill(c); true } None => false }
}

/// Wie lange nach dem Start ein Ende des Emulators als „sofort beendet“ gilt
const EARLY_EXIT: std::time::Duration = std::time::Duration::from_secs(5);
/// So viele Ausgabezeilen des Emulators bleiben für die Meldung erhalten
const TAIL_LINES: usize = 12;

/// Liest die Ausgabe des Emulators (Standardausgabe und Fehlerausgabe) laufend mit und behält nur die letzten Zeilen
fn pump(mut r: impl std::io::Read + Send + 'static, tail: std::sync::Arc<Mutex<std::collections::VecDeque<String>>>) {
    std::thread::spawn(move || {
        let mut buf = vec![];
        let mut chunk = [0u8; 4096];
        loop {
            match r.read(&mut chunk) { Ok(0) | Err(_) => break, Ok(n) => buf.extend_from_slice(&chunk[..n]) }
            while let Some(i) = buf.iter().position(|b| *b == b'\n') {
                let line = String::from_utf8_lossy(&buf[..i]).trim_end_matches('\r').trim().to_string();
                buf.drain(..=i);
                if line.is_empty() { continue; }
                if let Ok(mut t) = tail.lock() { t.push_back(line); while t.len() > TAIL_LINES { t.pop_front(); } }
            }
            if buf.len() > 64 * 1024 { buf.clear(); }
        }
        let rest = String::from_utf8_lossy(&buf).trim().to_string();
        if !rest.is_empty() { if let Ok(mut t) = tail.lock() { t.push_back(rest); while t.len() > TAIL_LINES { t.pop_front(); } } }
    });
}

/// Startet den Emulator, merkt sich seine Ausgabe und meldet ein Ende kurz nach dem Start (Rückgabewert ungleich 0, z. B. VICE ohne ROM-Dateien):
/// die Meldung (letzte Ausgabezeilen und ein Hinweis) holt [`take_errors`] bzw. [`wait_early_exit`] ab. Ein später Abbruch oder ein vom Nutzer beendeter Emulator bleibt still.
fn spawn_watched(cmd: &str, args: &[String]) -> Result<(), String> { spawn_watched_in(cmd, args, EARLY_EXIT) }

fn spawn_watched_in(cmd: &str, args: &[String], window: std::time::Duration) -> Result<(), String> {
    let mut child = isolate(std::process::Command::new(cmd).args(args).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()))
        .spawn().map_err(|e| vc_i18n::t!("launch-cmd-e", cmd = &cmd, e = &e))?;
    let tail = std::sync::Arc::new(Mutex::new(std::collections::VecDeque::new()));
    if let Some(o) = child.stdout.take() { pump(o, tail.clone()); }
    if let Some(e) = child.stderr.take() { pump(e, tail.clone()); }
    let pid = child.id();
    register(child);
    std::thread::spawn(move || {
        let t0 = std::time::Instant::now();
        while t0.elapsed() < window {
            let status = {
                let mut g = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
                match g.as_mut() { Some(c) if c.id() == pid => c.try_wait(), _ => return }     // beendet oder ersetzt: nichts melden
            };
            match status {
                Ok(Some(st)) => {
                    if st.success() { return; }
                    std::thread::sleep(std::time::Duration::from_millis(200));      // die Lesefäden die letzten Zeilen holen lassen
                    let code = st.code().map(|c| vc_i18n::t!("launch-rueckgabewert-c", c = &c)).unwrap_or_default();
                    let mut m = vc_i18n::t!("launch-der-emulator-hat-sich-sofort-beendet", code = &code);
                    let lines: Vec<String> = tail.lock().map(|t| t.iter().cloned().collect()).unwrap_or_default();
                    if !lines.is_empty() { m.push_str(&format!("\n{}", vc_i18n::t!("launch-ausgabe-des-emulators"))); for l in lines { m.push_str(&format!("\n  {}", l.chars().take(300).collect::<String>())); } }
                    ASYNC_ERRORS.lock().unwrap_or_else(|e| e.into_inner()).push(m);
                    return;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(_) => return,
            }
        }
    });
    Ok(())
}

/// Wartet bis zu `timeout` auf Meldungen aus Hintergrundstarts (Emulator sofort beendet, Xemu-Start fehlgeschlagen); leer, wenn nichts schiefging
pub fn wait_early_exit(timeout: std::time::Duration) -> Vec<String> {
    let t0 = std::time::Instant::now();
    loop {
        let v = take_errors();
        if !v.is_empty() || t0.elapsed() >= timeout { return v; }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Merkt sich den gestarteten Emulator; ein noch laufender vorheriger wird beendet (sonst bliebe er ohne Handle zurück und ließe sich nicht mehr beenden)
fn register(c: Child) { let r = Run::new(c); if let Ok(mut g) = RUNNING.lock() { if let Some(old) = g.take() { kill(old); } *g = Some(r); } else { kill(r); } }

/// Für Tests und Prüfbrücke: einen fremden Prozess als „laufenden Emulator“ anmelden
pub fn adopt(c: Child) { register(c); }

/// Läuft der zuletzt gestartete Emulator noch?
pub fn is_running() -> bool {
    let Ok(mut g) = RUNNING.lock() else { return false };
    match g.as_mut() { Some(c) => matches!(c.try_wait(), Ok(None)), None => false }
}

/// Ist der zuletzt gestartete Emulator von selbst zu Ende gegangen (vom Nutzer beendet, abgestürzt)? Dann wird er abgemeldet und der Rückgabewert geliefert
/// (`Some(Some(code))`, `Some(None)` bei Beendigung durch ein Signal); `None`: läuft noch oder es gibt keinen.
pub fn poll_exit() -> Option<Option<i32>> {
    let mut g = RUNNING.lock().ok()?;
    let st = match g.as_mut()?.try_wait() { Ok(Some(st)) => st, _ => return None };
    *g = None;
    Some(st.code())
}

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
    let path = &ohne_praefix(path);
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
            "atari8" => (resolve(&settings.emulator_of_kind("atari8", "atari800"), &[]).ok_or_else(|| vc_i18n::t!("launch-atari800-nicht-gefunden-p1-oder-pfad", p1 = vc_emu::install_hint("atari800")))?,
                         if path.extension().map_or(false, |e| e.eq_ignore_ascii_case("car") || e.eq_ignore_ascii_case("rom")) { vec!["-xl".into(), (if atari_variant.to_string() == "ntsc" { "-ntsc" } else { "-pal" }).into(), "-nobasic".into(), "-cart".into(), path.to_string_lossy().into_owned()] } else { vec!["-xl".into(), (if atari_variant.to_string() == "ntsc" { "-ntsc" } else { "-pal" }).into(), "-nobasic".into(), "-run".into(), path.to_string_lossy().into_owned()] }),
            "mega65" => {
                // Open-ROMs-BASIC versteht `RUN"*"` nicht (Autoload von Diskette scheitert): Xemu starten, das Programm über den Monitor laden
                // und am SYS-Ziel losrennen lassen; der Monitor-Thread beendet sich danach, Xemu läuft weiter.
                // unter Windows nur zur Prüfung gelesen (Xemu lädt das Programm selbst)
                #[cfg_attr(windows, allow(unused_variables))]
                let prg = std::fs::read(path).map_err(|e| vc_i18n::t!("launch-rel-e", rel = &rel, e = &e))?;
                let xemu = settings.emulator_for("mega65");
                let rplan = crate::roms::plan("xemu", &crate::roms::store(Some(&project_root_of(path)), Some(&xemu)), &[]);
                if let Some(e) = rplan.error.clone() { return Err(e); }
                let rom = rplan.xemu_rom.clone().map(|p| p.to_string_lossy().into_owned()).ok_or(vc_i18n::t!("launch-mega65-rom-nicht-gefunden"))?;
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
                        Ok(Some(c)) if GENERATION.load(Ordering::SeqCst) != generation => kill(Run::new(c)),
                        Ok(Some(c)) => register(c),
                        Ok(None) => {}
                        Err(e) => { eprintln!("MEGA65-Start: {e}"); ASYNC_ERRORS.lock().unwrap_or_else(|x| x.into_inner()).push(vc_i18n::t!("launch-mega65-start-fehlgeschlagen-e", e = &e)); }
                    }
                });
                #[cfg(windows)]
                return Ok(with_lines(vc_i18n::t!("launch-xemu-programm-wird-mit-prg-im").to_string(), &rom_lines));
                #[cfg(not(windows))]
                return Ok(with_lines(vc_i18n::t!("launch-xemu-programm-wird-ueber-den-monitor", entry = format!("{:04X}", entry)), &rom_lines));
            }
            _ => { let prog = vice.kind.as_str(); (resolve(&vice, &crate::emulators::search_dirs(&vice.kind).iter().map(String::as_str).collect::<Vec<_>>()).ok_or(vc_i18n::t!("launch-prog-vice-nicht-gefunden", prog = &prog))?,
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
        spawn_watched(&cmd, &args)?;
    Ok(with_lines(vc_i18n::t!("launch-cmd-p2", cmd = &cmd, p2 = args.join(" ")), &rom_lines))
}

/// Meldung des Starts um die ROM-Zeilen (gewählte ROMs mit Herkunft, Warnungen) ergänzen
fn with_lines(head: String, lines: &[String]) -> String {
    if lines.is_empty() { head } else { format!("{head}\n{}", lines.join("\n")) }
}

/// Legt ein Diskettenimage im Standard-Emulator des Systems ein (Laufwerk 8 bei VICE, Laufwerk 1 bei atari800/Altirra) und startet ihn; ein
/// früher gestarteter Emulator wird beendet. Es wird nichts geladen: bei Commodore tippt man `LOAD"*",8,1`, der Atari bootet die Diskette selbst.
pub fn insert_disk(image: &Path, target: &str, atari_variant: &str) -> Result<String, String> {
    if target == "mega65" { return Err(vc_i18n::t!("launch-mega65-einlegen-im-emulator-ist-noch").into()); }
    let (cmd, args) = insert_disk_command(image, target, atari_variant)?;
    stop_running();
    spawn_watched(&cmd, &args)?;
    Ok(vc_i18n::t!("launch-cmd-p2", cmd = &cmd, p2 = args.join(" ")))
}

/// Programm und Argumente, mit denen [`insert_disk`] den Emulator startet (ohne ihn zu starten; für Tests und Anzeige).
/// Atari: atari800 mit `-xl -pal|-ntsc -nobasic` und dem Abbild als Laufwerk 1; Altirra (Windows, wenn gewählt): `/nobasic /disk <Abbild>`.
/// Der Altirra-Start mit einer Turbo-BASIC-Diskette ist noch nicht gemessen (zu prüfen in einer Windows-Sitzung).
pub fn insert_disk_command(image: &Path, target: &str, atari_variant: &str) -> Result<(String, Vec<String>), String> {
    let image = &ohne_praefix(image);
    if target == "mega65" { return Err(vc_i18n::t!("launch-mega65-einlegen-im-emulator-ist-noch").into()); }
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
            let prog = resolve(&a8, &[]).ok_or_else(|| vc_i18n::t!("launch-atari800-nicht-gefunden-p1-oder-pfad", p1 = vc_emu::install_hint("atari800")))?;
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
        let prog = resolve(&vice, &crate::emulators::search_dirs(&vice.kind).iter().map(String::as_str).collect::<Vec<_>>()).ok_or(vc_i18n::t!("launch-p1-vice-nicht-gefunden", p1 = &vice.kind))?;
        let mut v: Vec<String> = vec!["-8".into(), img];
        let user = vice.launch_args(&ctx)?;
        v.extend(crate::roms::plan(&vice.kind, &crate::roms::store(Some(&project_root_of(image)), Some(&vice)), &user).args);
        v.extend(user);
        (prog, v)
    };
    args.retain(|a| !a.is_empty());
    Ok((cmd, args))
}

/// Startet `path` auf einer echten Maschine aus den Einstellungen: Ultimate 64 (REST: Reset, Laden, RUN) bzw. MEGA65 (Monitor über die serielle Schnittstelle).
pub fn run_on_machine(m: &crate::settings::Machine, path: &Path, target: &str) -> Result<String, String> {
    let path = &ohne_praefix(path);
    crate::emulators::require_device(&m.kind)?;
    stop_running();
    let prg = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    match m.kind.as_str() {
        "ultimate64" => {
            if target != "c64" { return Err(vc_i18n::t!("launch-die-ultimate-hardware-spielt-nur-c64").into()); }
            let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
            if ext != "prg" && ext != "crt" { return Err(vc_i18n::t!("launch-auf-der-ultimate-lassen-sich-nur").into()); }
            let mut u = vc_emu::Ultimate64::connect(m.host.trim(), Some(m.password.as_str()))?;
            let who = if m.name.is_empty() { &m.host } else { &m.name };
            if ext == "crt" {
                // Ein Modul startet immer mit Reset und läuft sofort (run_crt); „nur laden“ gibt es dafür nicht
                u.run_crt(&prg)?;
                return Ok(vc_i18n::t!("launch-who-modul-gestartet-p2-bytes", who = &who, p2 = prg.len()));
            }
            if m.reset_before { u.reset()?; }
            if m.run_after { u.run_prg(&prg)?; } else { u.load_prg(&prg)?; }
            Ok(vc_i18n::t!("launch-who-programm-gestartet-p2-bytes", who = &who, p2 = prg.len()))
        }
        "mega65" => {
            if target != "mega65" { return Err(vc_i18n::t!("launch-die-mega65-hardware-spielt-nur-mega65").into()); }
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
            Ok(vc_i18n::t!("launch-p1-programm-gestartet-start-bei-entry", p1 = if m.name.is_empty() { &m.device } else { &m.name }, entry = format!("{:04X}", entry)))
        }
        k => Err(vc_i18n::t!("launch-unbekannter-maschinentyp-k", k = &k)),
    }
}

/// Startet ein Programm von einer Diskette auf einer echten MEGA65 über Ethernet: Diskette auf die SD-Karte kopieren, einbinden, `prg` starten.
/// UNGEPRÜFT (nur gegen Attrappen-Befehle getestet). Über USB gibt es das nicht.
pub fn run_disk_on_machine(m: &crate::settings::Machine, disk: &Path, prg: &Path, target: &str) -> Result<String, String> {
    let (disk, prg) = (&ohne_praefix(disk), &ohne_praefix(prg));
    crate::emulators::require_device(&m.kind)?;
    if m.kind != "mega65" || target != "mega65" { return Err(vc_i18n::t!("launch-diskette-auf-hardware-starten-gibt-es").into()); }
    if m.link_for(crate::mega65net::Action::Start) != crate::mega65net::Link::Eth { return Err(vc_i18n::t!("launch-eine-diskette-laesst-sich-nur-ueber").into()); }
    stop_running();
    crate::mega65net::start_disk(m, disk, prg)
}

/// Kann die Maschine das Programm von einer Diskette starten (MEGA65 mit Ethernet als Startverbindung)?
pub fn machine_runs_disks(m: &crate::settings::Machine) -> bool {
    m.kind == "mega65" && m.link_for(crate::mega65net::Action::Start) == crate::mega65net::Link::Eth && m.link_for(crate::mega65net::Action::Copy) == crate::mega65net::Link::Eth
}

#[cfg(test)]
mod praefix_tests { use super::*; #[test] fn praefix_wird_entfernt() { assert_eq!(ohne_praefix(Path::new(r"\\?\C:\a\b.prg")), std::path::PathBuf::from(r"C:\a\b.prg")); assert_eq!(ohne_praefix(Path::new(r"\\?\UNC\s\f")), std::path::PathBuf::from(r"\\?\UNC\s\f")); assert_eq!(ohne_praefix(Path::new("/tmp/x")), std::path::PathBuf::from("/tmp/x")); } }

#[cfg(all(test, unix))]
mod early_exit_tests {
    use super::*;
    use std::time::Duration;
    fn sh(script: &str) -> Vec<String> { vec!["-c".into(), script.into()] }
    /// Die Tests teilen die Meldungsliste und das laufende Programm; deshalb nacheinander
    #[test]
    fn immediate_exit_reports_last_output_and_hint_others_stay_silent() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        take_errors();
        spawn_watched_in("sh", &sh("echo \"Couldn't load kernal ROM\" >&2; echo zweite zeile; exit 1"), Duration::from_secs(3)).unwrap();
        let v = wait_early_exit(Duration::from_secs(6));
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("sofort beendet") && v[0].contains("Rückgabewert 1") && v[0].contains("Einstellungen ▸ ROMs"), "{}", v[0]);
        assert!(v[0].contains("Couldn't load kernal ROM") && v[0].contains("zweite zeile"), "{}", v[0]);
        // Rückgabewert 0: kein Fehler
        spawn_watched_in("sh", &sh("echo fertig; exit 0"), Duration::from_secs(2)).unwrap();
        assert!(wait_early_exit(Duration::from_millis(2500)).is_empty());
        // läuft weiter: nichts gemeldet; stop_running beendet ihn still
        spawn_watched_in("sh", &sh("sleep 30"), Duration::from_secs(1)).unwrap();
        assert!(wait_early_exit(Duration::from_millis(1500)).is_empty());
        assert!(stop_running());
        // später Abbruch (nach dem Fenster): still
        spawn_watched_in("sh", &sh("sleep 1.5; exit 3"), Duration::from_millis(500)).unwrap();
        assert!(wait_early_exit(Duration::from_millis(2500)).is_empty());
    }
}

/// Die Tests teilen das laufende Programm und die Meldungsliste und laufen deshalb nacheinander
#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(all(test, unix))]
mod ende_tests {
    use super::*;
    #[test] fn poll_exit_meldet_das_ende_eines_selbst_beendeten_prozesses_und_register_beendet_den_alten() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        stop_running();
        adopt(std::process::Command::new("sh").args(["-c", "exit 3"]).spawn().unwrap());
        let t0 = std::time::Instant::now(); let mut r = None;
        while t0.elapsed().as_secs() < 5 { r = poll_exit(); if r.is_some() { break; } std::thread::sleep(std::time::Duration::from_millis(50)); }
        assert_eq!(r, Some(Some(3)), "Rückgabewert kommt an"); assert!(!is_running() && poll_exit().is_none(), "danach abgemeldet");
        // zweiter Start ersetzt den ersten: der erste wird beendet, nicht vergessen
        let a = std::process::Command::new("sleep").arg("30").spawn().unwrap(); let pid_a = a.id(); adopt(a);
        adopt(std::process::Command::new("sleep").arg("30").spawn().unwrap());
        std::thread::sleep(std::time::Duration::from_millis(200));
        let lebt = std::process::Command::new("kill").args(["-0", &pid_a.to_string()]).status().map(|s| s.success()).unwrap_or(false);
        assert!(!lebt, "der ersetzte Prozess läuft nicht mehr");
        assert!(is_running() && stop_running() && !is_running());
    }

    fn lebt(pid: i32) -> bool { let a = unsafe { libc::kill(pid, 0) } == 0; a && !zombie(pid) }
    fn pid_aus(c: &mut std::process::Child) -> i32 {
        // die Prozess-ID des innersten `sleep` aus der Ausgabe des Platzhalters lesen
        use std::io::BufRead; let mut l = String::new(); std::io::BufReader::new(c.stdout.take().unwrap()).read_line(&mut l).unwrap(); l.trim().parse().unwrap()
    }
    fn kette() -> std::process::Child {
        // wie VICE unter macOS: Wrapper → Wrapper → eigentlicher Prozess; der innerste meldet seine Nummer
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "sh -c 'sh -c \"echo \\$\\$; sleep 30; true\"; true'; true"]).stdout(std::process::Stdio::piped()).stdin(std::process::Stdio::null());
        isolate(&mut cmd).spawn().unwrap()
    }

    #[test] fn beenden_trifft_die_ganze_wrapper_kette() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        stop_running();
        let mut c = kette(); let innen = pid_aus(&mut c); adopt(c);
        assert!(lebt(innen) && is_running());
        assert!(stop_running());
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(!lebt(innen), "der innerste Prozess ist beendet");
        assert!(!is_running());
    }

    #[test] fn ein_neuer_start_beendet_die_ganze_alte_kette() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        stop_running();
        let mut a = kette(); let innen = pid_aus(&mut a); adopt(a);
        adopt(std::process::Command::new("sleep").arg("30").spawn().unwrap());
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(!lebt(innen), "der alte innerste Prozess ist beendet");
        assert!(stop_running());
    }

    #[test] fn ohne_eigene_gruppe_wird_der_baum_ueber_die_eltern_kette_beendet() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        stop_running();
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "sh -c \"echo \\$\\$; sleep 30; true\"; true"]).stdout(std::process::Stdio::piped()).stdin(std::process::Stdio::null());
        let mut c = cmd.spawn().unwrap(); let innen = pid_aus(&mut c); adopt(c);
        assert!(stop_running());
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(!lebt(innen));
    }

    #[test] fn ende_wird_erst_gemeldet_wenn_auch_die_nachkommen_weg_sind() {
        let _l = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        stop_running();
        // der Wrapper endet sofort, der eigentliche Prozess läuft weiter
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "sleep 1.5 & exit 0"]).stdin(std::process::Stdio::null());
        adopt(isolate(&mut cmd).spawn().unwrap());
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(poll_exit().is_none() && is_running(), "läuft, solange ein Nachkomme lebt");
        let t0 = std::time::Instant::now(); let mut r = None;
        while t0.elapsed().as_secs() < 6 { r = poll_exit(); if r.is_some() { break; } std::thread::sleep(std::time::Duration::from_millis(100)); }
        assert_eq!(r, Some(Some(0)));
    }
}
