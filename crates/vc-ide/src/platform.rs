// SPDX-License-Identifier: GPL-3.0-or-later
//! Zugriffe aufs Betriebssystem für die Einstellungen: Emulatoren suchen und probeweise starten, Verbindungen zu echten Maschinen
//! prüfen (Netzwerk, Seriell) und serielle Ports auflisten. Ohne Oberfläche; alle Aufrufe können blockieren und gehören in einen
//! Hintergrund-Thread.
use crate::settings::{check_program, program_exe, Machine};
use crate::targets::EmulatorCfg;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Ergebnis der Suche nach dem Programm eines Emulators
#[derive(Clone, Debug, PartialEq)]
pub struct Presence {
    pub found: bool,
    /// Der Pfad stammt aus der automatischen Suche (Pfadfeld leer)
    pub auto: bool,
    /// Gefundener Pfad bzw. Fehlertext
    pub text: String,
}

/// Findet das Programm des Emulators: eingetragener Pfad, sonst automatische Suche (PATH und übliche Ordner)
pub fn presence(e: &EmulatorCfg) -> Presence {
    if !e.path.trim().is_empty() {
        return match check_program(&e.path) { Ok(t) => Presence { found: true, auto: false, text: t }, Err(t) => Presence { found: false, auto: false, text: t } };
    }
    match discover(&e.kind) { Some(p) => Presence { found: true, auto: true, text: p }, None => Presence { found: false, auto: true, text: "nicht gefunden – Pfad eintragen".into() } }
}

/// Pfad, den die automatische Suche für die Emulator-Sorte findet
pub fn discover(kind: &str) -> Option<String> { crate::launch::detect(kind) }

/// Probestart: startet den Emulator mit seinen Argumenten (ohne Programm) und prüft, dass er läuft; beendet ihn danach wieder.
/// Ein früher gestarteter Emulator wird zuvor beendet (es läuft immer nur einer).
pub fn test_launch(e: &EmulatorCfg) -> Result<String, String> {
    let exe = if e.path.trim().is_empty() { discover(&e.kind).ok_or("Programm nicht gefunden – Pfad eintragen")? } else { check_program(&e.path)?; program_exe(&e.path).to_string_lossy().into_owned() };
    // ohne Projekt gibt es keine Platzhalterwerte: Parameter mit Platzhaltern entfallen beim Test-Start
    let mut args = crate::args::build_plain(crate::args::catalog_of_kind(&e.kind).as_deref(), &e.opts, &e.args);
    // ROMs aus dem ROM-Verzeichnis (bei atari800 auch die alte Eintragung „OS-ROM“); Xemu bekommt seine ROM beim Start über `Mega65`
    if matches!(e.kind.as_str(), "x64sc" | "x64" | "x128" | "xvic" | "xplus4" | "atari800") {
        let r = e.resolved();
        args.extend(crate::roms::plan(&e.kind, &crate::roms::store(None, Some(&r)), &args.clone()).args);
    }
    crate::launch::stop_running();
    let mut child = Command::new(&exe).args(&args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().map_err(|er| format!("{exe}: {er}"))?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_millis(2500) {
        if let Some(status) = child.try_wait().map_err(|er| er.to_string())? {
            let mut err = String::new();
            if let Some(mut s) = child.stderr.take() { let _ = s.read_to_string(&mut err); }
            let tail: Vec<&str> = err.lines().rev().filter(|l| !l.trim().is_empty()).take(2).collect();
            let tail = tail.into_iter().rev().collect::<Vec<_>>().join(" / ");
            return Err(if status.success() { "Emulator hat sich sofort beendet (Argumente prüfen)".into() } else { format!("Emulator beendet sich mit {}{}", status_text(&status), if tail.is_empty() { String::new() } else { format!(": {tail}") }) });
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill(); let _ = child.wait();
    Ok(format!("läuft ({exe})"))
}

/// Exit-Code lesbar: Windows meldet -1 als `0xffffffff`, daher als vorzeichenbehaftete Zahl („Code -1“)
fn status_text(s: &std::process::ExitStatus) -> String {
    match s.code() { Some(c) => format!("Code {c}"), None => s.to_string() }
}

/// Verbindungstest einer echten Maschine mit Zeitlimit; liefert Meldung und Dauer
pub fn test_hardware(m: &Machine, limit: Duration) -> Result<(String, Duration), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let m = m.clone();
    std::thread::spawn(move || { let t0 = Instant::now(); let r = crate::settings::test_machine(&m); let _ = tx.send(r.map(|t| (t, t0.elapsed()))); });
    rx.recv_timeout(limit).map_err(|_| format!("keine Antwort innerhalb von {} s", limit.as_secs()))?
}

/// Verbindungstest einer MEGA65 über eine bestimmte Verbindung mit Zeitlimit (Ethernet: UNGEPRÜFT)
pub fn test_hardware_link(m: &Machine, link: crate::mega65net::Link, limit: Duration) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let m = m.clone();
    std::thread::spawn(move || { let _ = tx.send(crate::settings::test_machine_link(&m, link)); });
    rx.recv_timeout(limit).map_err(|_| format!("keine Antwort innerhalb von {} s", limit.as_secs()))?
}

/// Serielle Ports des Rechners (macOS/Linux: USB-Seriell-Geräte, Windows: COM-Ports)
pub fn scan_serial_ports() -> Vec<String> { crate::settings::serial_ports() }

/// Übliche Baudraten für die serielle Verbindung
pub const BAUD_RATES: &[&str] = &["115200", "230400", "460800", "921600", "1000000", "2000000", "4000000"];
