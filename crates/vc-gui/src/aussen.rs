// SPDX-License-Identifier: GPL-3.0-or-later
//! Aufrufe nach außen (Dateimanager, …) laufen **nur** über diese Schnittstelle. Im Testlauf (und mit `VC_TEST_NO_SPAWN=1`) wird nie ein
//! Prozess gestartet: der Aufruf wird nur aufgezeichnet. Ein Test sorgt dafür, dass im übrigen GUI-Quelltext kein `Command` vorkommt.
use std::cell::RefCell;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Aufruf { AudioAktualisieren, ImOrdnerZeigen(PathBuf), Standardprogramm(PathBuf), DiskEinlegen(PathBuf), Mega65Kopieren(PathBuf), AudioStart { ord: usize, zeile: usize }, AudioStopp, AudioVorhoeren { note: u8 }, EmulatorTest(String), HardwareTest(String), EmulatorInstall(String), PortScan, Beenden, GitInit(PathBuf), UrlOeffnen(String), MailOeffnen, UpdatePruefen }

thread_local! { static AUFGEZEICHNET: RefCell<Vec<Aufruf>> = const { RefCell::new(Vec::new()) }; }

/// Gilt die Sicherung (Testlauf oder `VC_TEST_NO_SPAWN`)?
pub fn gesperrt() -> bool { cfg!(test) || std::env::var_os("VC_TEST_NO_SPAWN").is_some() }

/// Führt einen Aufruf aus; mit Sicherung nur aufzeichnen
pub fn ausfuehren(a: Aufruf) {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(a)); return }
    match a {
        Aufruf::AudioAktualisieren | Aufruf::DiskEinlegen(_) | Aufruf::Mega65Kopieren(_) | Aufruf::AudioStart { .. } | Aufruf::AudioStopp | Aufruf::AudioVorhoeren { .. } | Aufruf::EmulatorTest(_) | Aufruf::HardwareTest(_) | Aufruf::EmulatorInstall(_) | Aufruf::PortScan | Aufruf::Beenden | Aufruf::GitInit(_) | Aufruf::UrlOeffnen(_) | Aufruf::MailOeffnen | Aufruf::UpdatePruefen => {} // laufen über `disk_einlegen` und `mega65_kopieren` (mit Ergebnis)
        Aufruf::Standardprogramm(p) => {
            #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(&p).spawn(); }
            #[cfg(target_os = "windows")] { let _ = std::process::Command::new("cmd").args(["/C", "start", ""]).arg(&p).spawn(); }
            #[cfg(all(unix, not(target_os = "macos")))] { let _ = std::process::Command::new("xdg-open").arg(&p).spawn(); }
        }
        Aufruf::ImOrdnerZeigen(p) => {
            #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg("-R").arg(&p).spawn(); }
            #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn(); }
            #[cfg(all(unix, not(target_os = "macos")))] { if let Some(d) = p.parent() { let _ = std::process::Command::new("xdg-open").arg(d).spawn(); } }
        }
    }
}

/// Aufruf für den Test aufzeichnen (nur im Testlauf; im Betrieb passiert der Aufruf selbst, hier nur die Buchführung)
pub fn aufzeichnen(a: Aufruf) { if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(a)) } }

/// Diskette im Standard-Emulator des Systems einlegen (startet den Emulator); im Testlauf nur aufzeichnen
pub fn disk_einlegen(image: &std::path::Path, target: &str, variant: &str) -> Result<String, String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::DiskEinlegen(image.to_path_buf()))); return Ok(vc_i18n::t!("aussen-testlauf-nichts-gestartet").into()) }
    vc_ide::launch::insert_disk(image, target, variant)
}

/// Diskette per Ethernet auf einen MEGA65 kopieren und einbinden (kann dauern: nicht auf dem Oberflächenthread aufrufen); im Testlauf nur aufzeichnen
pub fn mega65_kopieren(image: &std::path::Path) -> Result<String, String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::Mega65Kopieren(image.to_path_buf()))); return Ok(vc_i18n::t!("aussen-testlauf-nichts-uebertragen").into()) }
    let st = vc_ide::settings::effective();
    let Some(m) = st.machines.iter().find(|m| m.kind == "mega65" && m.link_for(vc_ide::mega65net::Action::Copy) == vc_ide::mega65net::Link::Eth).cloned() else { return Err(vc_i18n::t!("aussen-keine-mega65-mit-ethernet-eingetragen-einstellungen").into()) };
    vc_ide::mega65net::copy_and_mount(&m, image)
}

/// Nur für Tests: bisher aufgezeichnete Aufrufe dieses Threads (leert die Liste)
#[cfg(test)]
pub fn aufgezeichnet() -> Vec<Aufruf> { AUFGEZEICHNET.with(|x| std::mem::take(&mut *x.borrow_mut())) }

/// Test-Start eines Emulators (startet kurz einen Prozess und beendet ihn); im Testlauf nur aufzeichnen
pub fn emulator_test(e: &vc_ide::targets::EmulatorCfg) -> Result<String, String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::EmulatorTest(e.id.clone()))); return Ok("(Testlauf: nichts gestartet)".into()) }
    vc_ide::platform::test_launch(e)
}
/// Verbindungstest zu einer Maschine (Netzwerk bzw. serielle Schnittstelle); im Testlauf nur aufzeichnen
pub fn hardware_test(m: &vc_ide::settings::Machine, link: Option<vc_ide::mega65net::Link>) -> Result<String, String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::HardwareTest(m.id.clone()))); return Ok("(Testlauf: nichts verbunden)".into()) }
    let limit = std::time::Duration::from_secs(12);
    match link { Some(l) => vc_ide::platform::test_hardware_link(m, l, limit), None => vc_ide::platform::test_hardware(m, limit).map(|(t, d)| format!("{t} ({} ms)", d.as_millis().max(1))) }
}
/// Serielle Schnittstellen auflisten; im Testlauf eine feste Attrappe
pub fn ports_suchen() -> Vec<String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::PortScan)); return vec!["/dev/cu.test-port".into()] }
    vc_ide::platform::scan_serial_ports()
}
/// Emulator laden und portabel installieren (Netz, nur nach Bestätigung in der Oberfläche); im Testlauf nur aufzeichnen
pub fn emulator_installieren(spec: &str) -> Result<String, String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::EmulatorInstall(spec.to_string()))); return Ok("(Testlauf: nichts geladen)".into()) }
    vc_ide::emulators::install(spec, &mut |_| {})
}

/// `git init` im Projektordner (Prozess); im Testlauf nur aufzeichnen und „gelungen“ melden
pub fn git_init(dir: &std::path::Path) -> bool {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::GitInit(dir.to_path_buf()))); return true }
    std::process::Command::new("git").arg("init").current_dir(dir).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}

/// Adresse im Browser öffnen (nur feste Adressen des Programms, nie Adressen aus Dateien oder Netz); im Testlauf nur aufzeichnen
pub fn url_oeffnen(url: &str) {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::UrlOeffnen(url.to_string()))); return }
    let (prog, args) = vc_ide::feedback::open_command(std::env::consts::OS, url);
    let _ = std::process::Command::new(prog).args(args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
}
/// Mailprogramm mit dem vorbereiteten `mailto:`-Link öffnen (gesendet wird dort); im Testlauf nur aufzeichnen
pub fn mail_oeffnen(url: &str) -> Result<(), String> {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::MailOeffnen)); return Ok(()) }
    vc_ide::feedback::open_mailto(url)
}
thread_local! { static UPDATE_ANTWORT: RefCell<Option<vc_ide::update::Report>> = const { RefCell::new(None) }; }
/// Nur Tests: die Antwort der nächsten Update-Prüfung vorgeben (sonst „offline“); es geht nie etwas ins Netz
pub fn test_update_antwort(r: vc_ide::update::Report) { UPDATE_ANTWORT.with(|x| *x.borrow_mut() = Some(r)) }
/// Versionsliste abfragen und auswerten (Netz); im Testlauf eine vorgegebene Antwort
pub fn update_pruefen(src: &str, installiert: vc_ide::update::Version) -> vc_ide::update::Report {
    if gesperrt() { AUFGEZEICHNET.with(|x| x.borrow_mut().push(Aufruf::UpdatePruefen)); return UPDATE_ANTWORT.with(|x| x.borrow_mut().take()).unwrap_or(vc_ide::update::Report { at: 0, source: src.to_string(), result: Err("offline (Testlauf)".into()), raw: None }) }
    vc_ide::update::check(src, installiert, vc_ide::update::now_secs())
}

/// Text der Systemabfrage nach dem Tastaturlayout (macOS: Eingabequelle, Linux: setxkbmap); `None`, wenn nicht abfragbar. Im Test immer `None`.
pub fn tastaturlayout_text() -> Option<String> {
    if cfg!(test) { return None }
    let aus = |prog: &str, args: &[&str]| -> Option<String> { let o = std::process::Command::new(prog).args(args).output().ok()?; o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned()) };
    if cfg!(target_os = "macos") { aus("defaults", &["read", "com.apple.HIToolbox", "AppleCurrentKeyboardLayoutInputSourceID"]) } else if cfg!(target_os = "linux") { aus("setxkbmap", &["-query"]) } else { None }
}
