// SPDX-License-Identifier: GPL-3.0-or-later
//! Emulator-Anbindung: einheitliche Schnittstelle (`Backend`) und Adapter für VICE (Binary Monitor) und atari800 (Text-Monitor).
mod atari800;
pub mod fujisan;
pub mod install;
mod mega65;
pub mod streams;
mod ultimate;
mod vice;

pub use atari800::Atari800;
pub use fujisan::Fujisan;
pub use mega65::Mega65;
pub use ultimate::{Info as UltimateInfo, Ultimate64, UltimateCaps};
pub use vice::Vice;

/// Startparameter für [`backend_for`]: was der Adapter zum Starten braucht (nicht jeder Adapter braucht alles)
#[derive(Debug, Clone, Default)]
pub struct Launch<'a> {
    /// Emulator-Sorte (`x64sc`, `x128` …); bei VICE der Programmname
    pub kind: &'a str,
    /// Programm, das geladen wird; `None` = leerer Emulator (Tests im Trap-Modus)
    pub file: Option<&'a str>,
    /// Pfad des Emulator-Programms (`None` = automatisch suchen)
    pub exe: Option<&'a str>,
    /// zusätzliche Programmargumente
    pub args: &'a [String],
    /// atari800: Betriebssystem-ROM
    pub os_rom: Option<&'a str>,
    /// Fujisan: TCP-Port (0 = Standard)
    pub port: u16,
    /// Xemu: MEGA65-ROM
    pub rom: Option<&'a str>,
}

/// Adapter zu einer Kennung aus dem Emulator-Katalog (Feld `backend` in `data/emulators/catalog.json`) starten. Die Zuordnung Kennung → Adapter
/// bleibt Rust, weil jeder Adapter eigenen Code hat; der Katalog nennt nur die Kennung.
pub fn backend_for(id: &str, l: &Launch) -> Result<Box<dyn Backend>, String> {
    match id {
        "vice-binmon" => Vice::launch_with(l.kind, l.file.unwrap_or(""), l.exe, l.args).map(|v| Box::new(v) as Box<dyn Backend>),
        "atari800-textmon" => match l.file {
            Some(f) => Atari800::launch_with(f, l.exe, l.os_rom, l.args),
            None => Atari800::launch_bare(l.exe, l.os_rom, l.args),
        }.map(|v| Box::new(v) as Box<dyn Backend>),
        "fujisan-tcp" => {
            let port = if l.port == 0 { fujisan::DEFAULT_PORT } else { l.port };
            match l.file { Some(f) => Fujisan::launch(f, l.exe, port), None => Fujisan::launch_bare(l.exe, port) }.map(|v| Box::new(v) as Box<dyn Backend>)
        }
        "xemu-monitor" => {
            let rom = l.rom.ok_or("MEGA65-ROM nicht gefunden")?;
            Mega65::launch_xemu_rom(l.exe, rom, l.args).map(|v| Box::new(v) as Box<dyn Backend>)
        }
        "" => Err("Diese Anbindung hat keinen Debugger- und Test-Adapter".into()),
        other => Err(format!("Adapter „{other}“ ist unbekannt")),
    }
}

/// Kennungen, die [`backend_for`] kennt (für den Test gegen den Katalog)
pub const BACKENDS: &[&str] = &["vice-binmon", "atari800-textmon", "fujisan-tcp", "xemu-monitor"];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Regs {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub flags: u8,
    /// Zusätzliche Werte des Emulators, z. B. `LIN`, `CYC`, `00`, `01` (VICE).
    pub extra: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState { Running, Stopped }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchKind { Read, Write, Access }

/// Was der Adapter kann (die Oberfläche blendet Fehlendes aus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps { pub registers: bool, pub step: bool, pub breakpoints: bool, pub watch: bool }

/// Zusätzliche Fähigkeiten für den Trap-Modus der Tests (getrennt von `Caps`, damit bestehende Adapter unverändert bleiben).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrapCaps {
    /// `set_regs` setzt alle Register samt Flags (sonst höchstens den PC; Flags dann über einen Vorspann im Programmcode)
    pub set_regs: bool,
    /// `cycles` liefert Zeile und Zyklus in der Zeile
    pub cycles: bool,
}

/// Strahlposition des Emulators: Rasterzeile und Zyklus in der Zeile, dazu die Maße des Videomodus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cycles { pub line: u32, pub cycle: u32, pub per_line: u32, pub lines: u32 }

impl Cycles {
    /// Zyklen von `start` bis `self`. Die Position wiederholt sich mit jedem Bild (`per_line` × `lines` Zyklen): gültig ist der Wert
    /// nur, wenn weniger als ein Bild vergangen ist, das kann die Position allein nicht erkennen.
    pub fn since(&self, start: &Cycles) -> u64 {
        let dl = (self.line as i64 - start.line as i64).rem_euclid(self.lines as i64);
        let d = dl * self.per_line as i64 + (self.cycle as i64 - start.cycle as i64);
        d.rem_euclid(self.per_line as i64 * self.lines as i64) as u64
    }
    /// Zyklen je Bild
    pub fn frame(&self) -> u64 { self.per_line as u64 * self.lines as u64 }
}

/// Ursache des letzten Anhaltens, soweit der Adapter sie kennt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit { Exec(u16), Watch(u32) }

pub trait Backend: Send {
    fn caps(&self) -> Caps { Caps { registers: true, step: true, breakpoints: true, watch: false } }
    /// Grund, warum der Einzelschritt mit diesem Emulator nicht geht (bekannte Einschränkung); `None`: erlaubt
    fn step_blocked(&self) -> Option<String> { None }
    /// Datenhaltepunkt auf `start..=end`; liefert eine Kennung zum Entfernen.
    fn set_watch(&mut self, _start: u16, _end: u16, _kind: WatchKind) -> Result<u32, String> { Err("Datenhaltepunkte werden von diesem Adapter nicht unterstützt".into()) }
    fn clear_watch(&mut self, _id: u32) -> Result<(), String> { Ok(()) }
    fn last_hit(&mut self) -> Option<Hit> { None }
    /// Fähigkeiten für den Trap-Modus (Standard: keine)
    fn trap_caps(&self) -> TrapCaps { TrapCaps::default() }
    /// Setzt alle Register (PC, A, X, Y, SP, Flags). Der Emulator muss angehalten sein.
    fn set_regs(&mut self, _r: &Regs) -> Result<(), String> { Err("Register setzen wird von diesem Adapter nicht unterstützt".into()) }
    /// Strahlposition (Rasterzeile, Zyklus in der Zeile) für Zeitmessungen
    fn cycles(&mut self) -> Result<Cycles, String> { Err("Zyklenmessung wird von diesem Adapter nicht unterstützt".into()) }
    /// Läuft bis `addr` erreicht ist (Ausführungs-Haltepunkt). `Ok(true)`: angehalten an `addr`; `Ok(false)`: Zeitlimit, der Zustand ist
    /// danach unbestimmt (Emulator neu starten). Der Haltepunkt wird danach wieder entfernt.
    fn run_until(&mut self, addr: u16, timeout: std::time::Duration) -> Result<bool, String> {
        self.set_breakpoint(addr)?;
        let res = (|| {
            self.cont()?;
            let end = std::time::Instant::now() + timeout;
            while std::time::Instant::now() < end {
                if self.state() == RunState::Stopped { return Ok(self.registers()?.pc == addr); }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Ok(false)
        })();
        let _ = self.clear_breakpoint(addr);
        res
    }
    /// Debug-Register der Hardware (Ultimate 64: `$D7FF` über REST), falls der Adapter eines hat; `None`: gibt es nicht.
    /// `Some(Err(..))`: es gibt eines, es ließ sich aber nicht lesen (der Läufer fällt dann auf das RAM-Postfach zurück).
    fn debug_reg(&mut self) -> Option<Result<u8, String>> { None }
    fn name(&self) -> &str;
    /// Verarbeitet Ereignisse des Emulators (z. B. Haltepunkt erreicht) und liefert den aktuellen Zustand.
    fn state(&mut self) -> RunState;
    fn registers(&mut self) -> Result<Regs, String>;
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String>;
    /// Speicher in einer bestimmten Sicht lesen (`ram`, `rom`, `io`, `cpu`): unabhängig von der aktuellen Speicherumschaltung; Standard: Sicht der CPU
    fn read_space(&mut self, _space: &str, addr: u16, len: usize) -> Result<Vec<u8>, String> { self.read_mem(addr, len) }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String>;
    /// Einzelschritt (in Unterprogramme hinein).
    fn step(&mut self) -> Result<(), String>;
    fn cont(&mut self) -> Result<(), String>;
    fn can_pause(&self) -> bool;
    fn pause(&mut self) -> Result<(), String>;
    fn set_breakpoint(&mut self, addr: u16) -> Result<(), String>;
    fn clear_breakpoint(&mut self, addr: u16) -> Result<(), String>;
    fn breakpoints(&self) -> Vec<u16>;
    /// Kann der Adapter das Bild des Emulators liefern (`grab_frame`)? Standard: nein.
    fn can_grab_frame(&self) -> bool { false }
    /// Bildquellen des Emulators, z. B. `VIC-II` und `VDC` beim C128; leer: eine einzige Quelle
    fn frame_sources(&self) -> Vec<&'static str> { vec![] }
    /// Wählt die Bildquelle (Index in `frame_sources`)
    fn set_frame_source(&mut self, _index: usize) {}
    /// Holt das aktuelle Bild als RGBA. Ein laufender Emulator wird dafür kurz angehalten und danach fortgesetzt; ein stehender bleibt stehen.
    fn grab_frame(&mut self) -> Result<streams::video::Frame, String> { Err("Das Bild des Emulators ist mit diesem Adapter nicht abrufbar".into()) }
    /// Kann der Adapter Text über den Tastaturpuffer eingeben (`type_text`)? Standard: nein.
    fn can_type_text(&self) -> bool { false }
    /// Speist Zeichen (PETSCII) in den Tastaturpuffer des Emulators. Sie werden verarbeitet, sobald die Maschine läuft.
    fn type_text(&mut self, _petscii: &[u8]) -> Result<(), String> { Err("Texteingabe wird von diesem Adapter nicht unterstützt".into()) }
    /// Beendet den Emulator.
    fn stop(&mut self);
}

/// ASCII-Text für die Tastaturpuffer der Commodore-Rechner (PETSCII): Buchstaben beider Schreibweisen werden zu den Großbuchstaben
/// des Einschaltzustands, Zeilenumbruch zu RETURN; was es dort nicht gibt, entfällt. Höchstens 255 Zeichen (Grenze des Befehls).
pub fn ascii_to_petscii(text: &str) -> Vec<u8> {
    text.chars().filter_map(|c| match c {
        'a'..='z' => Some(c as u8 - 32),
        'A'..='Z' | ' '..='@' | '[' | ']' => Some(c as u8),
        '\n' => Some(13),
        _ => None,
    }).take(255).collect()
}

/// Freien lokalen TCP-Port finden.
pub fn free_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0)).and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(6502)
}

/// Sucht ein Programm (gewählte Version, PATH und zusätzliche Verzeichnisse, zuletzt die eigenen Installationen unter `install`).
pub fn find_program(name: &str, extra: &[&str]) -> Option<String> {
    // ausdrücklich gewählte Version (`name@version`), sonst PATH und übliche Ordner, zuletzt die von VintageCoder portabel installierten
    if let Some(p) = install::find_preferred(name) { return Some(p.to_string_lossy().into_owned()); }
    find_program_system(name, extra).or_else(|| install::find_any(name).map(|p| p.to_string_lossy().into_owned()))
}

/// Wie `find_program`, aber nur PATH und übliche Ordner des Systems (ohne die portablen Installationen von VintageCoder)
pub fn find_program_system(name: &str, extra: &[&str]) -> Option<String> {
    let mut dirs: Vec<std::path::PathBuf> = extra.iter().map(std::path::PathBuf::from).collect();
    if let Some(p) = std::env::var_os("PATH") { dirs.extend(std::env::split_paths(&p)); }
    #[cfg(windows)]
    dirs.extend(windows_dirs(name));
    #[cfg(target_os = "linux")]
    dirs.extend(linux_dirs());
    // Windows: Programme heißen name.exe
    let names: Vec<String> = if cfg!(windows) && !name.ends_with(".exe") { vec![format!("{name}.exe"), name.to_string()] } else { vec![name.to_string()] };
    dirs.iter().flat_map(|d| names.iter().map(move |n| d.join(n))).find(|p| p.is_file()).map(|p| p.to_string_lossy().into_owned())
}

/// Linux: Ordner, die Emulatoren üblicherweise belegen, aber nicht immer im `PATH` der Anwendung (Desktop-Starter, Menü) liegen:
/// `/usr/games` (Debian/Ubuntu: atari800), `/usr/local/games`, `~/.local/bin`, `~/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`.
/// Flatpak-Programme heißen nach ihrer Kennung (nicht `x64sc`) und werden nicht automatisch gefunden: Pfad unter Einstellungen eintragen.
#[cfg(target_os = "linux")]
fn linux_dirs() -> Vec<std::path::PathBuf> {
    let mut v: Vec<std::path::PathBuf> = ["/usr/games", "/usr/local/games", "/usr/local/bin", "/usr/bin", "/snap/bin", "/opt/vice/bin", "/opt/xemu", "/opt/xemu/bin"].iter().map(std::path::PathBuf::from).collect();
    if let Some(h) = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(std::path::PathBuf::from) { v.push(h.join(".local/bin")); v.push(h.join("bin")); v.push(h.join("xemu")); v.push(h.join("Applications")); }
    v.extend(login_shell_path());
    v
}

/// Linux: `PATH` der Login-Shell. Aus dem Desktop-Starter gestartete Programme erben oft nur einen kurzen `PATH`, ohne die Ordner,
/// die `~/.profile` oder `~/.bashrc` ergänzen. Einmal ermittelt (mit Zeitlimit) und gemerkt; bei Fehlern leer.
#[cfg(target_os = "linux")]
fn login_shell_path() -> Vec<std::path::PathBuf> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<std::path::PathBuf>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let out = std::process::Command::new("/bin/sh").args(["-lc", "printf %s \"$PATH\""]).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).output();
            let _ = tx.send(out.ok().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()));
        });
        match rx.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(Some(p)) => std::env::split_paths(&p).filter(|d| d.is_absolute()).collect(),
            _ => vec![],
        }
    }).clone()
}

/// Hinweis zur Installation eines Emulators für die Meldung „nicht gefunden“ (`pkg`: `vice`, `atari800`), je nach Betriebssystem
pub fn install_hint(pkg: &str) -> String {
    if cfg!(target_os = "macos") { format!("brew install {pkg}") }
    else if cfg!(target_os = "linux") { format!("Paket „{pkg}“ der Distribution installieren, z. B. sudo apt install {pkg} / sudo dnf install {pkg} / sudo zypper install {pkg}") }
    else { format!("{pkg} installieren") }
}

/// Windows: übliche Installationsordner unter `Programme`, deren Name zum Emulator passt (Xemu: „xemu“, VICE: „vice“, atari800: „atari800“), je auch mit `bin`
#[cfg(windows)]
fn windows_dirs(name: &str) -> Vec<std::path::PathBuf> {
    let key = match name { "xmega65" => "xemu", "x64sc" | "x128" | "xplus4" | "xvic" => "vice", "atari800" => "atari800", "Fujisan" | "fujisan" => "fujisan", "Altirra64" | "Altirra" => "altirra", _ => return vec![] };
    let mut v = vec![];
    for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        let Some(base) = std::env::var_os(var).map(std::path::PathBuf::from) else { continue };
        let Ok(rd) = std::fs::read_dir(&base) else { continue };
        // absteigend nach Ordnername: bei mehreren Versionen (z. B. „… 4.01“ vor „… 3.90“) gewinnt die neuere
        let mut found: Vec<std::path::PathBuf> = rd.flatten().filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().to_ascii_lowercase().contains(key)).map(|e| e.path()).collect();
        found.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
        for d in found { v.push(d.join("bin")); v.push(d); }
    }
    v
}
