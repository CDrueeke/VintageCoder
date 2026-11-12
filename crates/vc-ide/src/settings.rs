// SPDX-License-Identifier: GPL-3.0-or-later
//! Persönliche Einstellungen (`~/.config/vintagecoder/settings.json`, Rechte 0600): Pfade und Parameter der Emulatoren sowie
//! Zugangsdaten zu echten Maschinen (Ultimate 64 / II+, MEGA65). Gelten für alle Projekte.
//!
//! Die Einstellungen sind prozessweit (`get`/`set`), damit Start (`launch`) und Debugger sie ohne Durchreichen nutzen können.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// VICE: Programme je System (leer = automatisch suchen), zusätzliche Kommandozeilenargumente
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vice { #[serde(default)] pub x64sc: String, #[serde(default)] pub x128: String, #[serde(default)] pub xplus4: String, #[serde(default)] pub xvic: String, #[serde(default)] pub args: String }

/// atari800: Programm, Betriebssystem-ROM (XL/XE, optional), zusätzliche Argumente
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Atari800 { #[serde(default)] pub path: String, #[serde(default)] pub os_rom: String, #[serde(default)] pub args: String }

/// Altirra (Atari 8 Bit, Windows): Programm und zusätzliche Argumente; `default_for_atari` wählt den Emulator zum Starten:
/// leer = Altirra, wenn gefunden, sonst atari800; `altirra` oder `atari800` erzwingt einen. Der Debugger wählt `Fujisan.debug_for_atari`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Altirra { #[serde(default)] pub path: String, #[serde(default)] pub args: String, #[serde(default)] pub default_for_atari: String }

/// Fujisan (Atari 8 Bit, alle Systeme): Programm (leer = automatisch suchen, auch die von VintageCoder geladene Version) und Wahl des Atari-Debuggers:
/// `debug_for_atari` leer = Fujisan, wenn gefunden, sonst atari800; `fujisan` oder `atari800` erzwingt einen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fujisan { #[serde(default)] pub path: String, #[serde(default)] pub debug_for_atari: String }

/// Xemu (MEGA65): Programm, Ordner mit `MEGA65.ROM`, zusätzliche Argumente
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Xemu { #[serde(default)] pub path: String, #[serde(default)] pub rom_dir: String, #[serde(default)] pub args: String }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Emulators { #[serde(default)] pub vice: Vice, #[serde(default)] pub atari800: Atari800, #[serde(default)] pub altirra: Altirra, #[serde(default)] pub fujisan: Fujisan, #[serde(default)] pub xemu: Xemu }

/// Eine echte Maschine. `kind`: `ultimate64` (auch Ultimate II+; Adresse + Netzwerkpasswort) oder `mega65` (serielle Schnittstelle)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub kind: String,
    /// Ultimate: Adresse (`192.168.1.50` oder `host:port`)
    #[serde(default)] pub host: String,
    /// Ultimate: Netzwerkpasswort (leer = keines)
    #[serde(default)] pub password: String,
    /// MEGA65: serielle Schnittstelle, z. B. `/dev/cu.usbserial-xxxx`
    #[serde(default)] pub device: String,
    /// MEGA65: Baudrate (leer = 2000000)
    #[serde(default)] pub baud: String,
    /// Zielsystem, zu dem die Maschine gehört (leer = aus dem Typ abgeleitet)
    #[serde(default)] pub target: String,
    /// Nach der Übertragung `RUN` ausführen (sonst nur laden)
    #[serde(default = "yes")] pub run_after: bool,
    /// Ultimate: vor der Übertragung die Maschine zurücksetzen
    #[serde(default)] pub reset_before: bool,
    /// MEGA65: Standardverbindung `usb` (leer = USB) oder `eth` (Ethernet); gilt für alle Aktionen ohne eigene Wahl.
    /// UNGEPRÜFT: der Ethernet-Weg ist nur gegen Attrappen-Befehle getestet (siehe `mega65net`).
    #[serde(default, skip_serializing_if = "String::is_empty")] pub link: String,
    /// MEGA65: eigene Verbindung für „Starten“ / „Kopieren“ / „Debuggen“ (`usb`, `eth`, leer = Standardverbindung)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub link_start: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub link_copy: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub link_debug: String,
    /// MEGA65, Ethernet: Netzwerkschnittstelle des PCs (z. B. `en0`)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub eth_iface: String,
    /// MEGA65, Ethernet: Pfade von `etherload` und `mega65_ftp` (leer = im PATH suchen)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub etherload: String,
    #[serde(default, skip_serializing_if = "String::is_empty")] pub ftp_tool: String,
    /// MEGA65, Ethernet: Ordner auf der SD-Karte, in den Disketten kopiert werden (leer = oberste Ebene)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub sd_dir: String,
}

fn yes() -> bool { true }

impl Default for Machine {
    fn default() -> Machine { Machine { id: String::new(), name: String::new(), kind: String::new(), host: String::new(), password: String::new(), device: String::new(), baud: String::new(), target: String::new(), run_after: true, reset_before: false, link: String::new(), link_start: String::new(), link_copy: String::new(), link_debug: String::new(), eth_iface: String::new(), etherload: String::new(), ftp_tool: String::new(), sd_dir: String::new() } }
}

impl Machine {
    /// Zielsystem der Maschine
    pub fn system(&self) -> &str {
        if !self.target.is_empty() { return &self.target; }
        match self.kind.as_str() { "mega65" => "mega65", _ => "c64" }
    }
    pub fn display_name(&self) -> &str { if self.name.trim().is_empty() { &self.id } else { self.name.trim() } }
}

/// Datenströme der Ultimate (Live-Bild, Hörprobe, Ablaufprotokoll)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StreamPrefs {
    /// Multicast statt gezielter Zustellung (nur wenn Unicast scheitert; manche Switches und WLAN-Zugangspunkte verteilen Multicast schlecht)
    #[serde(default)] pub multicast: bool,
    /// Fester erster UDP-Port (Bild; Ton +1, Ablaufprotokoll +2); 0 = freien Port wählen lassen
    #[serde(default)] pub base_port: u16,
    /// Eigene Adresse erzwingen (leer = vom Betriebssystem bestimmen lassen)
    #[serde(default)] pub interface: String,
}

/// ROM-Verzeichnis: Einstellungen (Orte und Suchreihenfolge siehe `vc_roms`)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RomPrefs {
    /// ROM-Ordner anderer Programme (VICE, Xemu, atari800) mitbenutzen; nur lesend, Standard: aus
    #[serde(default)] pub use_foreign: bool,
}

/// Hilfe: Handbuchbilder werden online nachgeladen (nie ohne Zustimmung)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HelpPrefs {
    /// `""` = beim ersten Öffnen eines Kapitels mit Bildern fragen, `always` = immer laden, `never` = nie laden
    #[serde(default, skip_serializing_if = "String::is_empty")] pub images: String,
    /// Ordner oder Adresse der Bilder (`<Basis>/<Kennung>.png`); leer = Standardadresse (siehe `help_images`). `VC_HELP_IMAGES` hat Vorrang.
    #[serde(default, skip_serializing_if = "String::is_empty")] pub images_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Version des Dateiformats (fehlt = 0 = vor dem Konfigurationsassistenten); siehe [`crate::setup::SCHEMA_VERSION`]. Neuere Versionen bleiben beim Speichern erhalten.
    #[serde(default, skip_serializing_if = "is_zero")] pub schema_version: u32,
    /// Basisverzeichnis der Projekte (leer = Voreinstellung `~/VintageCoder`); Voreinstellung für „Ort“ im Projekt-Assistenten
    #[serde(default, skip_serializing_if = "String::is_empty")] pub projects_dir: String,
    /// Im Konfigurationsassistenten abgewählte Zielsysteme: ausgeblendet, ihre Einträge bleiben erhalten (Wiedereinblenden stellt sie her)
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub hidden_targets: Vec<crate::targets::TargetCfg>,
    /// Debug-Protokoll in eine lokale Datei schreiben (siehe [`crate::debuglog`]); `VC_DEBUG_LOG` hat Vorrang
    #[serde(default, skip_serializing_if = "is_default")] pub debug_log: bool,
    /// Hilfe (Handbuchbilder online laden)
    #[serde(default, skip_serializing_if = "is_default")] pub help: HelpPrefs,
    /// ROM-Verzeichnis
    #[serde(default, skip_serializing_if = "is_default")] pub roms: RomPrefs,
    /// Datenströme der Ultimate
    #[serde(default, skip_serializing_if = "is_default")] pub streams: StreamPrefs,
    /// Ältere, nach Programmen gegliederte Felder; bei neuen Dateien leer (siehe `targets`, [`Settings::migrate_legacy`])
    #[serde(default, skip_serializing_if = "is_default")] pub emulators: Emulators,
    /// Zielsysteme mit ihren Emulatoren und dem Standard-Startziel
    #[serde(default, skip_serializing_if = "Vec::is_empty")] pub targets: Vec<crate::targets::TargetCfg>,
    #[serde(default)] pub machines: Vec<Machine>,
    /// Einstellungen des Assemblers beim Bauen in der IDE (Katalog `vcc-asm`)
    #[serde(default, skip_serializing_if = "is_default")] pub assembler: AsmCfg,
    #[serde(flatten)] pub extra: Map<String, Value>,
}

/// Assembler-Optionen der IDE: Auswahl aus dem Katalog und freies Feld (wie bei den Emulatoren)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AsmCfg {
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")] pub opts: crate::args::Selection,
    /// weitere Schalter, wie in der Shell geschrieben (z. B. mehrere `-I`)
    #[serde(default, skip_serializing_if = "String::is_empty")] pub args: String,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool { *v == T::default() }
fn is_zero(v: &u32) -> bool { *v == 0 }

/// Ort der Einstellungsdatei: `VC_SETTINGS`, sonst `settings.json` im Konfigurationsordner (`~/.config/vintagecoder`, Windows `%APPDATA%\vintagecoder`)
pub fn file() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VC_SETTINGS") { return Some(PathBuf::from(p)); }
    Some(vc_core::userdirs::config_dir()?.join("settings.json"))
}

impl Settings {
    pub fn load_from(path: &Path) -> Result<Settings, String> {
        let t = match std::fs::read_to_string(path) { Ok(t) => t, Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Settings::default()), Err(e) => return Err(format!("{}: {e}", path.display())) };
        serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))
    }
    /// Schreibt die Datei; sie enthält Passwörter und ist nur für den Benutzer lesbar (0600)
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(d) = path.parent() { std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?; }
        let mut t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?; t.push('\n');
        // erst mit engen Rechten anlegen, dann schreiben (kein Zeitfenster mit offenen Rechten)
        #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; use std::io::Write;
            let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).map_err(|e| format!("{}: {e}", path.display()))?;
            f.write_all(t.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
            use std::os::unix::fs::PermissionsExt; let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
            return Ok(());
        }
        #[allow(unreachable_code)] std::fs::write(path, t).map_err(|e| format!("{}: {e}", path.display()))
    }
    /// Schreibt die Datei atomar: erst in eine Nachbardatei mit Rechten 0600, dann `fsync`, dann Umbenennen. Eine Unterbrechung lässt die alte Datei heil.
    pub fn save_atomic(&self, path: &Path) -> Result<(), String> {
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?; t.push('\n');
        write_atomic(path, t.as_bytes())
    }
    pub fn machine(&self, id: &str) -> Option<&Machine> { self.machines.iter().find(|m| m.id == id) }
}

/// Schreibt `bytes` atomar nach `path` (Nachbardatei, Rechte 0600 vor dem Inhalt, `fsync`, Umbenennen)
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "datei".into());
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let r = (|| -> std::io::Result<()> {
        #[cfg(unix)] let mut f = { use std::os::unix::fs::OpenOptionsExt; std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)? };
        #[cfg(not(unix))] let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&tmp)?;
        f.write_all(bytes)?; f.sync_all()?; drop(f);
        std::fs::rename(&tmp, path)
    })();
    if let Err(e) = r { let _ = std::fs::remove_file(&tmp); return Err(format!("{}: {e}", path.display())); }
    Ok(())
}

/// Übernimmt die Einstellungen nur im Prozess (ohne Datei); der Konfigurationsassistent schreibt die Datei selbst atomar
pub fn replace_in_memory(s: Settings) { if let Ok(mut g) = cell().write() { *g = s; } }

fn cell() -> &'static RwLock<Settings> { static S: OnceLock<RwLock<Settings>> = OnceLock::new(); S.get_or_init(|| RwLock::new(file().and_then(|f| Settings::load_from(&f).ok()).unwrap_or_default())) }

/// Aktuelle Einstellungen (beim ersten Zugriff aus der Datei gelesen)
pub fn get() -> Settings { cell().read().map(|s| s.clone()).unwrap_or_default() }

fn project_cell() -> &'static RwLock<Vec<crate::targets::ProjectOverride>> { static P: OnceLock<RwLock<Vec<crate::targets::ProjectOverride>>> = OnceLock::new(); P.get_or_init(|| RwLock::new(Vec::new())) }

/// Meldet die Überschreibungen des geöffneten Projekts (leer = keine). Wirkt auf [`effective`]; ändert die Einstellungsdatei nicht.
pub fn set_project_overrides(ov: Vec<crate::targets::ProjectOverride>) {
    if project_cell().read().map_or(false, |g| *g == ov) { return; }
    if let Ok(mut g) = project_cell().write() { *g = ov; }
}

/// Einstellungen, wie sie für das geöffnete Projekt wirken: die globalen mit den Überschreibungen des Projekts.
/// Start und Debugger lesen Emulator und Startziel nur hierüber (nicht über [`get`]). Anbindungen, die das Release nicht enthält, fehlen hier
/// (siehe [`Settings::released`]).
pub fn effective() -> Settings {
    let ov = project_cell().read().map(|g| g.clone()).unwrap_or_default();
    get().with_overrides(&ov).released()
}

/// Übernimmt die Einstellungen im Prozess und speichert sie in die Datei
pub fn set(s: Settings) -> Result<(), String> {
    if let Ok(mut g) = cell().write() { *g = s.clone(); }
    match file() { Some(f) => s.save_to(&f), None => Err("Kein Ordner für die Einstellungen (HOME bzw. unter Windows APPDATA fehlt)".into()) }
}

/// Zerlegt eine Argumentzeile wie die Shell (Leerzeichen trennen; `"…"` und `'…'` fassen zusammen; `\` maskiert)
pub fn split_args(line: &str) -> Vec<String> { split_args_os(line, cfg!(windows)) }

/// Wie [`split_args`] für ein bestimmtes Betriebssystem. Windows: `\` ist ein gewöhnliches Zeichen (Pfade wie `C:\Users\x.img` und
/// `\\server\share`) und maskiert nur unmittelbar vor `"` oder `'`; sonst maskiert es jedes folgende Zeichen.
pub fn split_args_os(line: &str, windows: bool) -> Vec<String> {
    let (mut out, mut cur, mut quote, mut has) = (vec![], String::new(), None::<char>, false);
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        match (c, quote) {
            ('\\', _) if windows => { match it.peek() { Some(&n) if n == '"' || n == '\'' => { it.next(); cur.push(n); } _ => cur.push('\\') } has = true; }
            ('\\', _) if quote != Some('\'') => { if let Some(n) = it.next() { cur.push(n); has = true; } }
            ('"', None) | ('\'', None) => { quote = Some(c); has = true; }
            (q, Some(open)) if q == open => quote = None,
            (c, None) if c.is_whitespace() => { if has { out.push(std::mem::take(&mut cur)); has = false; } }
            (c, _) => { cur.push(c); has = true; }
        }
    }
    if has { out.push(cur); }
    out
}

/// Eingestellter Pfad für ein Programm (`x64sc`, `x128`, `xplus4`, `xvic`, `atari800`, `xemu`), falls gesetzt; `~` wird aufgelöst
pub fn program_path(kind: &str) -> Option<String> {
    let s = get();
    let p = match kind { "x64sc" => s.emulators.vice.x64sc, "x128" => s.emulators.vice.x128, "xplus4" => s.emulators.vice.xplus4, "xvic" => s.emulators.vice.xvic, "atari800" => s.emulators.atari800.path, "altirra" => s.emulators.altirra.path, "fujisan" => s.emulators.fujisan.path, "xemu" => s.emulators.xemu.path, _ => String::new() };
    let p = p.trim().to_string();
    if p.is_empty() { None } else { Some(expand_home(&p)) }
}

/// Zusätzliche Argumente für ein Programm (`vice`, `atari800`, `xemu`)
pub fn extra_args(kind: &str) -> Vec<String> {
    let s = get();
    split_args(match kind { "vice" => &s.emulators.vice.args, "atari800" => &s.emulators.atari800.args, "altirra" => &s.emulators.altirra.args, "xemu" => &s.emulators.xemu.args, _ => "" })
}

pub fn expand_home(p: &str) -> String {
    vc_core::userdirs::expand_home(p)
}

/// Ordner mit `MEGA65.ROM` für Xemu aus den Einstellungen
pub fn xemu_rom_dir() -> Option<String> { let d = get().emulators.xemu.rom_dir; let d = d.trim(); if d.is_empty() { None } else { Some(expand_home(d)) } }

/// Ist `path` eine ausführbare Datei (bei `.app`-Bündeln: das Programm darin)?
/// Ausführbare Datei zu einem Pfad (bei `.app`-Bündeln: das Programm darin)
pub fn program_exe(path: &str) -> PathBuf {
    let p = PathBuf::from(expand_home(path.trim()));
    if p.extension().map_or(false, |e| e == "app") { let n = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(); p.join("Contents/MacOS").join(n) } else { p }
}

pub fn check_program(path: &str) -> Result<String, String> {
    let p = program_exe(path);
    let md = std::fs::metadata(&p).map_err(|_| format!("{}: nicht gefunden", p.display()))?;
    if !md.is_file() { return Err(format!("{}: keine Datei", p.display())); }
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; if md.permissions().mode() & 0o111 == 0 { return Err(format!("{}: nicht ausführbar", p.display())); } }
    Ok(format!("gefunden: {}", p.display()))
}

/// Verbindungstest einer echten Maschine: Ultimate (REST-Version) bzw. MEGA65 (Monitor antwortet auf der seriellen Schnittstelle)
pub fn test_machine(m: &Machine) -> Result<String, String> {
    match m.kind.as_str() {
        "ultimate64" => {
            if m.host.trim().is_empty() { return Err("Adresse fehlt".into()); }
            let u = vc_emu::Ultimate64::connect(m.host.trim(), Some(m.password.as_str()))?;
            // Produkt, Firmware und Hostname aus /v1/info; fehlt die Angabe (ältere Firmware), bleibt die Versionsantwort
            match u.info() {
                Ok(i) if !i.product.is_empty() => Ok(describe_info(&i)),
                _ => { let v = u.version().unwrap_or_default(); Ok(format!("verbunden, Firmware-Antwort: {}", v.trim().chars().take(80).collect::<String>())) }
            }
        }
        "mega65" => test_machine_link(m, m.link_for(crate::mega65net::Action::Start)),
        k => Err(format!("unbekannter Typ '{k}'")),
    }
}

/// Verbindungstest einer MEGA65 über eine bestimmte Verbindung: USB (Monitor antwortet) oder Ethernet (Werkzeuge gefunden, `mega65_ftp dir`; UNGEPRÜFT)
pub fn test_machine_link(m: &Machine, link: crate::mega65net::Link) -> Result<String, String> {
    match link {
        crate::mega65net::Link::Usb => { let spec = mega65_spec(m)?; vc_emu::Mega65::connect(&spec).map(|_| "USB: Monitor antwortet".to_string()) }
        crate::mega65net::Link::Eth => crate::mega65net::test(m),
    }
}

/// Text des Verbindungstests: „verbunden: Ultimate 64-II, Firmware 3.14d, Host …“ (nur Angaben, die das Gerät gemeldet hat)
pub fn describe_info(i: &vc_emu::UltimateInfo) -> String {
    let mut parts = vec![format!("verbunden: {}", i.product)];
    if !i.firmware_version.is_empty() { parts.push(format!("Firmware {}", i.firmware_version)); }
    if !i.hostname.is_empty() { parts.push(format!("Host {}", i.hostname)); }
    parts.join(", ")
}

/// Verbindungsangabe für `vc_emu::Mega65::connect`
pub fn mega65_spec(m: &Machine) -> Result<String, String> {
    if m.device.trim().is_empty() { return Err("Serielle Schnittstelle fehlt (z. B. /dev/cu.usbserial-…)".into()); }
    let baud = m.baud.trim();
    Ok(if baud.is_empty() { format!("serial:{}", m.device.trim()) } else { format!("serial:{}@{baud}", m.device.trim()) })
}

/// Serielle Schnittstellen des Rechners (für die Auswahl beim MEGA65)
pub fn serial_ports() -> Vec<String> {
    #[allow(unused_mut)]
    let mut v: Vec<String> = std::fs::read_dir("/dev").into_iter().flatten().flatten().map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("cu.usb") || n.starts_with("cu.wch") || n.starts_with("ttyUSB") || n.starts_with("ttyACM")).map(|n| format!("/dev/{n}")).collect();
    // Windows: COM1…COM32 durch Öffnen prüfen (ein belegter Port meldet „Zugriff verweigert“, existiert aber)
    #[cfg(windows)]
    for n in 1..=32 {
        let open = std::fs::OpenOptions::new().read(true).write(true).open(format!(r"\\.\COM{n}"));
        if open.is_ok() || open.err().map_or(false, |e| e.kind() == std::io::ErrorKind::PermissionDenied) { v.push(format!("COM{n}")); }
    }
    v.sort(); v
}
