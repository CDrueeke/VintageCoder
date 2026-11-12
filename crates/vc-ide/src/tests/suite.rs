// SPDX-License-Identifier: GPL-3.0-or-later
//! Testlauf für die Oberfläche (Testkonzept T3): Ergebnismodell mit Gruppen, Zählern, Filter und Zeilenmarken sowie der Lauf über alle
//! `*.vctest` eines Projekts in einem Arbeits-Thread. Die Oberfläche schickt eine `RunRequest`, empfängt `Event`s und pflegt damit eine `Suite`.
//! Ohne Oberfläche testbar (`crates/vc-ide/tests/suite.rs`).
use super::{cycles, mailbox};
use super::model::{find_tests, Level, Outcome, State, TestFile, TestMode};
use super::runner::{self, arm_test, ArmInfo, RunOptions};
use super::static_checks::{run_file, Options, Prepared};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vc_emu::Backend;

/// Ein Ergebnis mit der Testdatei, aus der es stammt
#[derive(Debug, Clone)]
pub struct Item { pub file: PathBuf, pub outcome: Outcome }

/// Gruppe der Anzeige: die statischen Prüfungen aller Dateien oder die Emulator-Tests einer Datei
#[derive(Debug, Clone, Default)]
pub struct Group {
    pub title: String,
    /// Testdatei der Gruppe (`None` bei den statischen Prüfungen)
    pub file: Option<PathBuf>,
    /// Emulator, auf dem die Tests liefen (z. B. „VICE x64sc“)
    pub backend: Option<String>,
    pub items: Vec<Item>,
    pub millis: u64,
    /// Postfach-Modus (das Programm prüft sich selbst): kein Debuggen mit Falle, keine Setup-Zeile
    pub mailbox: bool,
    /// Anzahl der erwarteten Ergebnisse, solange der Lauf noch nicht fertig ist (Fortschritt)
    pub expected: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts { pub passed: usize, pub failed: usize, pub timed_out: usize, pub skipped: usize, pub hints: usize }

impl Counts {
    pub fn total(&self) -> usize { self.passed + self.failed + self.timed_out + self.skipped + self.hints }
}

/// Marke für eine Quelltextzeile (Randmarke und Zeilenhinweis im Editor)
#[derive(Debug, Clone, PartialEq)]
pub struct LineMark { pub line: usize, pub state: State, pub level: Level, pub text: String }

#[derive(Debug, Clone, Default)]
pub struct Suite { pub groups: Vec<Group>, pub running: bool, pub last_millis: Option<u64>, pub error: Option<String> }

/// Was die Oberfläche anstößt
#[derive(Debug, Clone, PartialEq)]
pub enum Only {
    /// alles
    All,
    /// nur diese Testdatei
    File(PathBuf),
    /// ein einzelner Emulator-Test
    Test(PathBuf, String),
    /// die bisher fehlgeschlagenen Tests (Datei, Name)
    Failed(Vec<(PathBuf, String)>),
}

/// Echte Hardware als Testgerät (nur Postfach-Modus: die Hardware liefert keine Register und keine Haltepunkte)
#[derive(Debug, Clone, PartialEq)]
pub enum Hardware {
    /// Ultimate 64 / Ultimate II+ über REST: `host` (`192.168.1.50` oder `host:port`), Netzwerkpasswort, Anzeigename
    Ultimate { host: String, password: Option<String>, name: String },
    /// MEGA65 über die serielle Schnittstelle (`serial:/dev/…[@baud]`): vorbereitet, ungeprüft (Gerät offline)
    Mega65 { spec: String, name: String },
}

impl Hardware {
    pub fn label(&self) -> String {
        match self { Hardware::Ultimate { host, name, .. } => format!("Ultimate 64 {} ({host})", if name.is_empty() { String::new() } else { format!("„{name}“") }).replace("  ", " "), Hardware::Mega65 { spec, name } => format!("MEGA65 {} ({spec})", if name.is_empty() { String::new() } else { format!("„{name}“") }).replace("  ", " ") }
    }
}

#[derive(Debug, Clone)]
pub struct RunRequest {
    pub root: PathBuf,
    pub data: PathBuf,
    /// `Some("vice")`: Tests im Emulator; `None`: nur die statischen Prüfungen
    pub backend: Option<String>,
    pub strict: bool,
    pub only: Only,
    /// Echte Hardware statt eines Emulators (hat Vorrang vor `backend`)
    pub hardware: Option<Hardware>,
}

#[derive(Debug, Clone)]
pub enum Event {
    /// Die statischen Prüfungen aller Dateien (ersetzt die Gruppe der vorigen Läufe)
    Static(Group),
    /// Eine Testdatei beginnt (Gruppe mit Titel, Emulator und erwarteter Anzahl)
    FileStarted { file: PathBuf, title: String, backend: Option<String>, expected: usize, mailbox: bool },
    Outcome { file: PathBuf, outcome: Outcome },
    FileDone { file: PathBuf, millis: u64 },
    Done { millis: u64 },
    Error(String),
}

pub const STATIC_TITLE: &str = "Statische Prüfungen";

/// Emulatoren, die der Läufer ansteuern kann und die installiert sind: (Kennung, Anzeigename)
pub fn available_backends() -> Vec<(String, String)> {
    let mut v = vec![];
    if crate::emulators::enabled("x64sc") && vc_emu::find_program("x64sc", &["/Applications/vice-arm64-gtk3-3.10/bin", "/Applications/vice-arm64-gtk3-3.9/bin", "/opt/homebrew/bin", "/usr/local/bin"]).is_some() { v.push(("vice".to_string(), "VICE x64sc".to_string())); }
    v
}

/// VICE-Programm für ein Ziel
pub fn vice_program(target: &str) -> Option<&'static str> { match target { "c64" => Some("x64sc"), "c128" => Some("x128"), "plus4" | "c16" => Some("xplus4"), "vic20" => Some("xvic"), _ => None } }

/// Startet `backend` (`vice`, `atari800`) mit dem Testprogramm für den Postfach-Modus
/// Startet das Testprogramm auf echter Hardware und liefert die Verbindung zum Lesen des Postfachs. Die Ultimate lädt und startet das
/// Programm (Reset, Laden, RUN) und liest den Speicher bei laufendem Rechner; `stop` setzt das Gerät fort.
pub fn launch_hardware(hw: &Hardware, target: &str, program: &Path) -> Result<Box<dyn Backend>, String> {
    crate::emulators::require_device(match hw { Hardware::Ultimate { .. } => "ultimate64", Hardware::Mega65 { .. } => "mega65" })?;
    let prg = std::fs::read(program).map_err(|e| format!("{}: {e}", program.display()))?;
    match hw {
        Hardware::Ultimate { host, password, .. } => {
            if target != "c64" { return Err("Die Ultimate-Hardware spielt nur C64-Programme".into()); }
            if !program.extension().map_or(false, |e| e.eq_ignore_ascii_case("prg")) { return Err("Auf der Ultimate lassen sich nur .prg-Dateien starten".into()); }
            let mut u = vc_emu::Ultimate64::connect(host.trim(), password.as_deref())?;
            u.run_prg(&prg)?;
            Ok(Box::new(u))
        }
        Hardware::Mega65 { spec, .. } => {
            if target != "mega65" { return Err("Die MEGA65-Hardware spielt nur MEGA65-Programme".into()); }
            let mut m = vc_emu::Mega65::connect(spec)?;
            let load = prg.first().copied().unwrap_or(1) as u16 | (prg.get(1).copied().unwrap_or(0x20) as u16) << 8;
            let entry = crate::debug::sys_target(&prg).unwrap_or(load);
            m.load_prg(&prg, entry)?; m.cont()?;
            Ok(Box::new(m))
        }
    }
}

pub fn launch_program(backend: &str, target: &str, program: &Path) -> Result<Box<dyn Backend>, String> {
    let file = program.to_string_lossy().into_owned();
    crate::emulators::require(backend)?;
    match backend {
        "vice" => { let p = vice_program(target).ok_or_else(|| format!("VICE kennt das Ziel {target} nicht"))?; let args: Vec<String> = if target == "c16" { vec!["-model".into(), "c16".into()] } else { vec![] }; crate::emulators::launch_backend(p, &vc_emu::Launch { kind: p, file: Some(&file), args: &args, ..Default::default() }) }
        "atari800" => { if target != "atari8" { return Err(format!("atari800 kennt nur das Ziel atari8, nicht {target}")); } crate::emulators::launch_backend("atari800", &vc_emu::Launch { kind: "atari800", file: Some(&file), ..Default::default() }) }
        "fujisan" => {
            if target != "atari8" { return Err(format!("Fujisan kennt nur das Ziel atari8, nicht {target}")); }
            let mut f = crate::emulators::launch_backend("fujisan", &vc_emu::Launch { kind: "fujisan", file: Some(&file), ..Default::default() })?;
            f.cont()?;
            Ok(f)
        }
        "xemu" => Err("Xemu lädt Programme über den Monitor: der Postfach-Modus ist dafür noch nicht eingerichtet".into()),
        x => Err(format!("Emulator „{x}“ ist unbekannt")),
    }
}

fn backend_label(id: &str, target: &str) -> String { match id { "vice" => format!("VICE {}", vice_program(target).unwrap_or("?")), x => x.to_string() } }

impl Counts {
    fn add(&mut self, o: &Outcome) {
        match (o.state, o.level) {
            (State::Passed, _) => self.passed += 1,
            (State::Failed, Level::Hint) => self.hints += 1,
            (State::Failed, Level::Error) => self.failed += 1,
            (State::TimedOut, _) => self.timed_out += 1,
            (State::Skipped, _) => self.skipped += 1,
        }
    }
}

impl Group {
    pub fn counts(&self) -> Counts { let mut c = Counts::default(); for i in &self.items { c.add(&i.outcome); } c }
}

impl Suite {
    pub fn counts(&self) -> Counts { let mut c = Counts::default(); for g in &self.groups { for i in &g.items { c.add(&i.outcome); } } c }
    /// Anzahl der Fehlschläge, die den Lauf scheitern lassen (fehlgeschlagen und Zeitüberschreitung)
    pub fn failed_count(&self) -> usize { let c = self.counts(); c.failed + c.timed_out }
    pub fn is_empty(&self) -> bool { self.groups.iter().all(|g| g.items.is_empty()) }
    /// „Tests 13/16“ für die Statusleiste: bestanden von allen Ergebnissen; `None`, solange es keine gibt
    pub fn status_text(&self) -> Option<String> {
        if self.is_empty() { return None; }
        let c = self.counts();
        Some(format!("Tests {}/{}", c.passed, c.total()))
    }

    /// Wendet ein Ereignis des Läufers an. `fresh`: ein vollständiger Lauf beginnt (vorherige Ergebnisse verwerfen)
    pub fn apply(&mut self, ev: Event) {
        match ev {
            Event::Static(g) => { match self.groups.iter_mut().find(|x| x.file.is_none()) { Some(old) => *old = g, None => self.groups.insert(0, g) } }
            Event::FileStarted { file, title, backend, expected, mailbox } => {
                match self.groups.iter_mut().find(|x| x.file.as_deref() == Some(&file)) {
                    Some(g) => { g.title = title; g.backend = backend; g.expected = expected; g.mailbox = mailbox; }
                    None => self.groups.push(Group { title, file: Some(file), backend, items: vec![], millis: 0, expected, mailbox }),
                }
            }
            Event::Outcome { file, outcome } => {
                let Some(g) = self.groups.iter_mut().find(|x| x.file.as_deref() == Some(&file)) else { return };
                match g.items.iter_mut().find(|i| i.outcome.name == outcome.name) { Some(i) => i.outcome = outcome, None => g.items.push(Item { file, outcome }) }
            }
            Event::FileDone { file, millis } => { if let Some(g) = self.groups.iter_mut().find(|x| x.file.as_deref() == Some(&file)) { g.millis = millis; g.expected = 0; } }
            Event::Done { millis } => { self.running = false; self.last_millis = Some(millis); for g in &mut self.groups { g.expected = 0; } }
            Event::Error(e) => { self.running = false; self.error = Some(e); }
        }
    }

    /// Beginn eines Laufs: bei `Only::All` frisch, sonst bleiben die übrigen Ergebnisse stehen
    pub fn begin(&mut self, only: &Only) {
        self.running = true; self.error = None;
        if *only == Only::All { self.groups.clear(); }
    }

    /// Einträge nach Filtertext (Name oder Grund, ohne Beachtung der Schreibweise) und optional nur Fehlgeschlagene: (Gruppe, Eintrag)
    pub fn visible(&self, filter: &str, only_failed: bool) -> Vec<(usize, usize)> {
        let f = filter.trim().to_lowercase();
        let mut v = vec![];
        for (gi, g) in self.groups.iter().enumerate() {
            for (ii, i) in g.items.iter().enumerate() {
                if only_failed && !i.outcome.is_error() { continue; }
                if !f.is_empty() && !i.outcome.name.to_lowercase().contains(&f) && !i.outcome.detail.to_lowercase().contains(&f) { continue; }
                v.push((gi, ii));
            }
        }
        v
    }

    /// Die fehlgeschlagenen Emulator-Tests (Datei, Name) für „Fehlgeschlagene ausführen“
    pub fn failed_tests(&self) -> Vec<(PathBuf, String)> {
        self.groups.iter().filter(|g| g.file.is_some()).flat_map(|g| g.items.iter().filter(|i| i.outcome.is_error()).map(|i| (i.file.clone(), i.outcome.name.clone()))).collect()
    }

    /// Marken für den Editor: alle Ergebnisse mit Quelltextstelle in der Datei `rel` (Pfad relativ zum Projekt). Je Zeile gewinnt das schlechteste Ergebnis.
    pub fn marks_for(&self, root: &Path, rel: &str) -> Vec<LineMark> {
        let mut by_line: Vec<LineMark> = vec![];
        for g in &self.groups {
            for i in &g.items {
                let Some((f, line)) = &i.outcome.location else { continue };
                if *line == 0 || !same_file(root, f, rel) { continue; }
                let o = &i.outcome;
                let m = LineMark { line: line - 1, state: o.state, level: o.level, text: mark_text(o) };
                match by_line.iter_mut().find(|x| x.line == m.line) {
                    Some(x) => { if rank(&m) > rank(x) { *x = m; } }
                    None => by_line.push(m),
                }
            }
        }
        by_line.sort_by_key(|m| m.line);
        by_line
    }
}

fn rank(m: &LineMark) -> u8 { match (m.state, m.level) { (State::Failed, Level::Error) | (State::TimedOut, _) => 4, (State::Failed, Level::Hint) => 3, (State::Skipped, _) => 2, (State::Passed, _) => 1 } }

/// Kurzer Hinweistext hinter der Zeile
fn mark_text(o: &Outcome) -> String {
    let m = o.measured.as_deref().map(|m| format!(" {m}")).unwrap_or_default();
    match (o.state, o.level) {
        (State::Passed, _) => format!("OK{m}"),
        (State::Failed, Level::Hint) => format!("Hinweis{m}: {}", o.name),
        (State::Failed, Level::Error) => format!("Fehler: {}{m}", o.name),
        (State::TimedOut, _) => format!("Zeitüberschreitung: {}", o.name),
        (State::Skipped, _) => format!("übersprungen: {}", o.name),
    }
}

/// Gehört der (absolute oder relative) Pfad `f` aus einem Ergebnis zur Projektdatei `rel`?
pub fn same_file(root: &Path, f: &str, rel: &str) -> bool {
    let p = Path::new(f);
    let stripped = p.strip_prefix(root).ok().map(|x| x.to_string_lossy().replace('\\', "/"));
    match stripped { Some(s) => s == rel, None => f.replace('\\', "/") == rel || (!p.is_absolute() && f.replace('\\', "/").trim_start_matches("./") == rel) }
}

/// Pfad relativ zum Projektordner (mit `/`), falls er darunter liegt; sonst unverändert
pub fn rel_to(root: &Path, f: &str) -> String { Path::new(f).strip_prefix(root).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| f.to_string()) }

fn rel_title(root: &Path, f: &Path) -> String { f.strip_prefix(root).unwrap_or(f).to_string_lossy().replace('\\', "/") }

/// Der Lauf selbst (blockiert; die Oberfläche ruft ihn in einem Arbeits-Thread). `cancel` wird zwischen den Tests geprüft.
pub fn run(req: &RunRequest, cancel: Arc<AtomicBool>, tx: &Sender<Event>) {
    let t0 = Instant::now();
    let send = |e: Event| { let _ = tx.send(e); };
    let mut files = find_tests(&req.root);
    match &req.only {
        Only::File(f) | Only::Test(f, _) => files.retain(|x| x == f),
        Only::Failed(v) => files.retain(|x| v.iter().any(|(f, _)| f == x)),
        Only::All => {}
    }
    // Statische Prüfungen: immer alle Dateien (schnell), eine Gruppe
    let mut stat = Group { title: STATIC_TITLE.into(), ..Default::default() };
    let opts = Options { strict: req.strict, dynamic: true };
    let all_files = find_tests(&req.root);
    let many = all_files.len() > 1;
    let loaded: Vec<(PathBuf, Result<TestFile, String>)> = all_files.iter().map(|f| (f.clone(), TestFile::load(f))).collect();
    let mut prepared: Vec<(PathBuf, TestFile, Option<Prepared>)> = vec![];
    for (f, tf) in loaded {
        match tf {
            Err(e) => stat.items.push(Item { file: f.clone(), outcome: Outcome::failed(rel_title(&req.root, &f), Level::Error, e) }),
            Ok(tf) => {
                let rep = run_file(&tf, &req.data, &opts);
                let stem = f.file_name().map(|n| n.to_string_lossy().trim_end_matches(".vctest").to_string()).unwrap_or_default();
                for mut o in rep.outcomes { if many { o.name = format!("{stem}: {}", o.name); } stat.items.push(Item { file: f.clone(), outcome: o }); }
                let pre = Prepared::load(&tf, &req.data).ok();
                prepared.push((f, tf, pre));
            }
        }
    }
    send(Event::Static(stat));
    // Tests im Emulator, je Datei eine Gruppe
    for (f, tf, pre) in prepared {
        if cancel.load(Ordering::SeqCst) { break; }
        if tf.tests.is_empty() || !files.contains(&f) { continue; }
        let title = rel_title(&req.root, &f);
        let suffix = match tf.mode { TestMode::Mailbox => " · Postfach", TestMode::Measure => " · Zyklen", TestMode::Trap => "" };
        let label = match &req.hardware { Some(h) => Some(format!("{}{suffix}", h.label())), None => req.backend.as_deref().map(|b| format!("{}{suffix}", backend_label(b, &tf.target))) };
        let names: Option<Vec<String>> = match &req.only {
            Only::Test(_, n) => Some(vec![n.clone()]),
            Only::Failed(v) => Some(v.iter().filter(|(x, _)| *x == f).map(|(_, n)| n.clone()).collect()),
            _ => None,
        };
        if names.as_ref().map_or(false, |n| n.is_empty()) { continue; }
        let expected = names.as_ref().map_or(tf.tests.len(), |n| n.len());
        send(Event::FileStarted { file: f.clone(), title, backend: label, expected, mailbox: tf.mode != TestMode::Trap });
        let file_t0 = Instant::now();
        let skip_all = |why: &str| {
            for (i, t) in tf.tests.iter().enumerate() {
                let name = t.get("name").and_then(|n| n.as_str()).map(str::to_string).unwrap_or_else(|| format!("Test {}", i + 1));
                if names.as_ref().map_or(true, |n| n.contains(&name)) { send(Event::Outcome { file: f.clone(), outcome: Outcome::skipped(name, why) }); }
            }
        };
        if let Some(hw) = &req.hardware {
            match &pre {
                None => skip_all("das Programm ist nicht gebaut oder nicht lesbar (Bauen, dann erneut ausführen)"),
                Some(pre) if tf.mode == TestMode::Measure => {
                    let tx2 = tx.clone(); let f2 = f.clone();
                    let ro = RunOptions {
                        strict: req.strict, filter: None, only_names: names.clone(), cancel: Some(cancel.clone()), boot_wait: Duration::ZERO,
                        on_outcome: Some(Arc::new(move |o: &Outcome| { let _ = tx2.send(Event::Outcome { file: f2.clone(), outcome: o.clone() }); })),
                        ..Default::default()
                    };
                    let mut m = cycles::hardware_measurer(hw.clone(), tf.target.clone(), pre.program.clone(), crate::streams::StreamOptions::from_settings(&crate::settings::effective()));
                    let _ = cycles::run_measure(&tf, pre, &ro, &mut m);
                }
                Some(_) if tf.mode == TestMode::Trap => skip_all("Trap-Modus nicht verfügbar: die Hardware liefert keine CPU-Register und keine Haltepunkte. Als Postfach-Test (\"mode\": \"mailbox\", test.inc) neu bauen"),
                Some(pre) => {
                    let tx2 = tx.clone(); let f2 = f.clone();
                    let ro = RunOptions {
                        strict: req.strict, filter: None, only_names: names.clone(), cancel: Some(cancel.clone()), boot_wait: Duration::ZERO,
                        on_outcome: Some(Arc::new(move |o: &Outcome| { let _ = tx2.send(Event::Outcome { file: f2.clone(), outcome: o.clone() }); })),
                        ..Default::default()
                    };
                    let (hw, target) = (hw.clone(), tf.target.clone());
                    let mut launch = move |p: &Path| launch_hardware(&hw, &target, p);
                    let _ = mailbox::run_mailbox(&tf, pre, &ro, &mut launch);
                }
            }
            send(Event::FileDone { file: f, millis: file_t0.elapsed().as_millis() as u64 });
            continue;
        }
        if tf.mode == TestMode::Measure && req.hardware.is_none() {
            skip_all("Zyklenmessung (mode measure) braucht die Ultimate 64: „Läuft auf“ auf das Gerät stellen bzw. vcc test --backend u64:HOST");
            send(Event::FileDone { file: f, millis: file_t0.elapsed().as_millis() as u64 });
            continue;
        }
        match (&req.backend, &pre) {
            (None, _) => skip_all("kein Emulator gewählt: „Läuft auf“ in der Kopfleiste"),
            (_, None) => skip_all("das Programm ist nicht gebaut oder nicht lesbar (Bauen, dann erneut ausführen)"),
            (Some(b), Some(pre)) if tf.mode == TestMode::Mailbox => {
                let tx2 = tx.clone(); let f2 = f.clone();
                let ro = RunOptions {
                    strict: req.strict, filter: None, only_names: names.clone(), cancel: Some(cancel.clone()),
                    on_outcome: Some(Arc::new(move |o: &Outcome| { let _ = tx2.send(Event::Outcome { file: f2.clone(), outcome: o.clone() }); })),
                    ..Default::default()
                };
                let b = b.clone(); let target = tf.target.clone();
                let mut launch = move |p: &Path| launch_program(&b, &target, p);
                let _ = mailbox::run_mailbox(&tf, pre, &ro, &mut launch);
            }
            (Some(b), Some(pre)) if b == "vice" => {
                let prog = vice_program(&tf.target);
                let tx2 = tx.clone(); let f2 = f.clone();
                let ro = RunOptions {
                    strict: req.strict, filter: None, only_names: names.clone(), cancel: Some(cancel.clone()),
                    on_outcome: Some(Arc::new(move |o: &Outcome| { let _ = tx2.send(Event::Outcome { file: f2.clone(), outcome: o.clone() }); })),
                    ..Default::default()
                };
                let mut launch = || -> Result<Box<dyn Backend>, String> {
                    let p = prog.ok_or_else(|| format!("VICE kennt das Ziel {} nicht", tf.target))?;
                    vc_emu::Vice::launch_with(p, "", None, &[]).map(|v| Box::new(v) as Box<dyn Backend>)
                };
                let _ = runner::run_tests(&tf, pre, &ro, &mut launch);
            }
            (Some(b), _) => skip_all(&format!("Emulator „{b}“ wird vom Läufer noch nicht unterstützt")),
        }
        send(Event::FileDone { file: f, millis: file_t0.elapsed().as_millis() as u64 });
    }
    send(Event::Done { millis: t0.elapsed().as_millis() as u64 });
}

/// Alles für „Diesen Test debuggen“: ein gestarteter Emulator mit vorbereiteter Falle, angehalten am Anfang der Falle
pub struct DebugPlan { pub backend: Box<dyn Backend>, pub info: ArmInfo, pub target: String, pub cpu: vc_target::Cpu, pub symbols: std::collections::HashMap<String, i64> }

pub fn prepare_debug(file: &Path, data: &Path, test: &str) -> Result<DebugPlan, String> {
    let tf = TestFile::load(file)?;
    if tf.mode == TestMode::Mailbox { return Err("Im Postfach-Modus gibt es keine Falle: das Programm lässt sich normal debuggen (F6)".into()); }
    let pre = Prepared::load(&tf, data)?;
    let prog = vice_program(&tf.target).ok_or_else(|| format!("Debuggen im Trap-Modus gibt es für das Ziel {} noch nicht", tf.target))?;
    let mut emu: Box<dyn Backend> = Box::new(vc_emu::Vice::launch_with(prog, "", None, &[])?);
    std::thread::sleep(RunOptions::default().boot_wait);
    let info = match arm_test(emu.as_mut(), &tf, &pre, test, &RunOptions::default()) { Ok(i) => i, Err(e) => { emu.stop(); return Err(e); } };
    Ok(DebugPlan { backend: emu, info, target: tf.target.clone(), cpu: pre.cpu.clone(), symbols: pre.dbg.symbols.clone() })
}
