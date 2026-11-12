// SPDX-License-Identifier: GPL-3.0-or-later
//! Rückmeldung per E-Mail: stellt Betreff und Text einer Mail an die Entwicklung zusammen (Version, Commit, System, Installation,
//! Release-Auswahl, Zielsystem) und baut daraus einen `mailto:`-Link, den der Anwender im eigenen Mailprogramm öffnet. Gemeinsam genutzt von
//! der IDE (Dialog „Feedback senden“) und `vcc feedback`.
//!
//! **Datenschutz:** Es wird nichts gesendet und nichts ins Netz geschickt; die Angaben stehen vor dem Öffnen im Dialog (bzw. in der
//! Ausgabe von `vcc feedback`). Pfade im persönlichen Ordner werden zu `~` gekürzt, Projektnamen kommen nicht vor.
//! **Länge:** `mailto:`-Links sind begrenzt (Windows rund 2000 Zeichen). Passt die Mail nicht hinein, entfallen die entbehrlichen Angaben
//! (zuletzt die Release-Auswahl, dann Ordner) und der Text nennt, was fehlt; die Zwischenablage-Fassung ist immer vollständig.
use std::path::{Path, PathBuf};

/// Empfänger der Rückmeldungen
pub const ADDRESS: &str = "vintagecoder@drueeke.de";
/// Höchstlänge des `mailto:`-Links in Zeichen (Windows verarbeitet rund 2000; etwas Luft für den Aufruf)
pub const MAX_URL: usize = 1900;

/// Woher das laufende Programm stammt
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Package { WindowsInstaller, MacApp, LinuxPackage, Source, Unknown }

impl Package {
    /// Anzeigetext
    pub fn label(self) -> &'static str {
        match self { Package::WindowsInstaller => "Windows-Installer", Package::MacApp => "macOS-Programmpaket", Package::LinuxPackage => "Linux-Paket", Package::Source => "Quellbau (cargo)", Package::Unknown => "unbekannt" }
    }
    /// Maschinenlesbare Kennung (`--json`)
    pub fn key(self) -> &'static str {
        match self { Package::WindowsInstaller => "windows-installer", Package::MacApp => "macos-app", Package::LinuxPackage => "linux-package", Package::Source => "source", Package::Unknown => "unknown" }
    }
}

/// Alle Angaben, aus denen die Mail entsteht (Pfade noch ungekürzt)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub version: String,
    /// Kurzer Git-Commit des Baus (leer, wenn beim Bau unbekannt)
    pub commit: Option<String>,
    /// z. B. „macOS 27.2“
    pub os: String,
    pub arch: String,
    pub package: Package,
    pub install_dir: Option<String>,
    pub data_dir: Option<String>,
    pub config_dir: Option<String>,
    /// Ausgewählte Release-Bestandteile (`data/release.json`); leer bei vollständigem Release
    pub release: Vec<(String, String)>,
    /// Kein `release.json`: alles an
    pub release_all: bool,
    /// Zielsystem des offenen Projekts (kein Projektname)
    pub target: Option<String>,
}

impl Info {
    /// Angaben des laufenden Programms. `target`: Zielsystem des offenen Projekts, falls eines offen ist.
    pub fn detect(target: Option<&str>) -> Info {
        let exe = std::env::current_exe().ok().and_then(|e| e.canonicalize().ok().or(Some(e)));
        let exe_dir = exe.as_deref().and_then(Path::parent).map(Path::to_path_buf);
        let data = exe_dir.as_deref().and_then(crate::find_data_dir).or_else(|| std::env::current_dir().ok().and_then(|d| crate::find_data_dir(&d)));
        let mut starts: Vec<PathBuf> = vec![];
        if let Some(e) = &exe { starts.push(e.clone()); }
        if let Some(d) = &data { starts.push(d.clone()); }
        let package = detect_package(&starts, &|p| p.exists());
        let install = match package {
            Package::MacApp => exe.as_deref().and_then(|e| e.ancestors().find(|a| a.extension().map_or(false, |x| x == "app"))).map(Path::to_path_buf),
            _ => exe_dir,
        };
        let f = crate::features::features();
        Info {
            version: env!("CARGO_PKG_VERSION").to_string(),
            commit: Some(env!("VC_BUILD_COMMIT").to_string()).filter(|c| !c.is_empty()),
            os: os_label(),
            arch: arch_label(std::env::consts::ARCH).to_string(),
            package,
            install_dir: install.map(|p| p.to_string_lossy().into_owned()),
            data_dir: data.map(|p| p.to_string_lossy().into_owned()),
            config_dir: vc_core::userdirs::config_dir().map(|p| p.to_string_lossy().into_owned()),
            release: f.describe().into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            release_all: f.is_all(),
            target: target.map(str::to_string).filter(|t| !t.is_empty() && t != "auto"),
        }
    }
}

/// `aarch64` heißt in der Mail `arm64` (wie in macOS-Meldungen üblich)
pub fn arch_label(arch: &str) -> &str { if arch == "aarch64" { "arm64" } else { arch } }

/// Quellpaket nach der Heuristik: Inno-Installer (`unins000.exe` neben dem Programm) → Windows-Installer; innerhalb eines `.app`-Bündels →
/// macOS-Programmpaket; unter `/usr`, `/opt` → Linux-Paket; `Cargo.toml` und `.git` oberhalb → Quellbau; sonst unbekannt.
/// `starts`: Programmdatei und Datenordner; `exists`: Dateiprüfung (für Tests austauschbar).
pub fn detect_package(starts: &[PathBuf], exists: &dyn Fn(&Path) -> bool) -> Package {
    let Some(first) = starts.first() else { return Package::Unknown };
    if first.parent().map_or(false, |d| exists(&d.join("unins000.exe"))) { return Package::WindowsInstaller; }
    if first.ancestors().any(|a| a.extension().map_or(false, |x| x == "app") && a.file_name().is_some()) { return Package::MacApp; }
    let posix = first.to_string_lossy().replace('\\', "/");
    if cfg!(not(windows)) && (posix.starts_with("/usr/bin/") || posix.starts_with("/usr/lib/") || posix.starts_with("/usr/share/") || posix.starts_with("/opt/")) { return Package::LinuxPackage; }
    for s in starts { if s.ancestors().skip(1).any(|a| exists(&a.join("Cargo.toml")) && exists(&a.join(".git"))) { return Package::Source; } }
    Package::Unknown
}

/// Bezeichnung des Betriebssystems mit Version, z. B. „macOS 27.2“, „Windows 10.0.22631“, „Fedora Linux 43“
pub fn os_label() -> String {
    #[cfg(target_os = "macos")]
    { if let Some(v) = run_text("sw_vers", &["-productVersion"]) { return format!("macOS {v}"); } return "macOS".into(); }
    #[cfg(windows)]
    { if let Some(v) = run_text("cmd", &["/C", "ver"]).and_then(|t| parse_windows_ver(&t)) { return format!("Windows {v}"); } return "Windows".into(); }
    #[cfg(all(unix, not(target_os = "macos")))]
    { return std::fs::read_to_string("/etc/os-release").ok().and_then(|t| parse_os_release(&t)).unwrap_or_else(|| "Linux".into()); }
    #[allow(unreachable_code)]
    std::env::consts::OS.to_string()
}

#[allow(dead_code)]
fn run_text(prog: &str, args: &[&str]) -> Option<String> {
    let o = std::process::Command::new(prog).args(args).output().ok()?;
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    (o.status.success() && !s.is_empty()).then_some(s)
}

/// `PRETTY_NAME` aus `/etc/os-release`
pub fn parse_os_release(text: &str) -> Option<String> {
    let line = text.lines().find_map(|l| l.strip_prefix("PRETTY_NAME="))?;
    let v = line.trim().trim_matches('"').trim_matches('\'').trim();
    (!v.is_empty()).then(|| v.to_string())
}

/// Version aus der Ausgabe von `ver` („Microsoft Windows [Version 10.0.22631.4602]“ → „10.0.22631“)
pub fn parse_windows_ver(text: &str) -> Option<String> {
    let rest = text.split("Version ").nth(1)?;
    let v = rest.split(']').next()?.trim();
    let parts: Vec<&str> = v.split('.').collect();
    (parts.len() >= 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))).then(|| parts[..3].join("."))
}

/// Pfad für die Mail kürzen: der persönliche Ordner wird zu `~`; liegt der Pfad sonst unter einem Benutzerordner (`/Users/x`, `/home/x`,
/// `C:\Users\x`), ebenfalls (damit kein Benutzername in der Mail steht).
pub fn shorten_home(path: &str, home: Option<&str>) -> String {
    if let Some(h) = home.map(|h| h.trim_end_matches(['/', '\\'])).filter(|h| !h.is_empty()) {
        if let Some(rest) = path.strip_prefix(h) { if rest.is_empty() || rest.starts_with(['/', '\\']) { return format!("~{rest}"); } }
    }
    let p = path.replace('\\', "/");
    let segs: Vec<&str> = p.split('/').collect();
    let at = |i: usize| segs.get(i).copied().unwrap_or("");
    let skip = if at(0).is_empty() && matches!(at(1), "Users" | "home") && !at(2).is_empty() { Some(3) }
        else if at(0).len() == 2 && at(0).ends_with(':') && at(1).eq_ignore_ascii_case("users") && !at(2).is_empty() { Some(3) } else { None };
    match skip {
        Some(n) => { let sep = if path.contains('\\') { "\\" } else { "/" }; let rest = segs[n..].join(sep); if rest.is_empty() { "~".into() } else { format!("~{sep}{rest}") } }
        None => path.to_string(),
    }
}

/// Text für `mailto:`: alles außer `A–Z a–z 0–9 - . _ ~` als `%XX` (UTF-8), Zeilenumbrüche als `%0D%0A`, Leerzeichen als `%20`
pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    let s = s.replace("\r\n", "\n");
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            b'\n' => out.push_str("%0D%0A"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `mailto:`-Link aus Betreff und Text
pub fn mailto_url(subject: &str, body: &str) -> String { format!("mailto:{ADDRESS}?subject={}&body={}", percent_encode(subject), percent_encode(body)) }

/// Die fertige Mail
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mail {
    pub subject: String,
    /// Text, wie er im `mailto:`-Link steht (bei Platzmangel gekürzt, mit Hinweis)
    pub body: String,
    /// Vollständiger Text (Zwischenablage)
    pub body_full: String,
    pub url: String,
    /// Angaben, die wegen der Längengrenze fehlen (Bezeichnungen)
    pub dropped: Vec<&'static str>,
    /// Auch ohne die entbehrlichen Angaben länger als die Grenze
    pub too_long: bool,
}

/// Betreff: „VintageCoder 0.1.0 (Commit abc1234) · macOS 27.2 arm64 · Paket: Quellbau (cargo)“
pub fn subject(i: &Info) -> String {
    let commit = i.commit.as_deref().map(|c| format!(" (Commit {c})")).unwrap_or_default();
    format!("VintageCoder {}{commit} · {} {} · Paket: {}", i.version, i.os, i.arch, i.package.label())
}

/// Entbehrliche Angaben, in der Reihenfolge, in der sie bei Platzmangel entfallen (zuerst die erste)
const DROP_ORDER: [&str; 4] = ["Release-Auswahl", "Konfigurationsordner", "Datenordner", "Installationsordner"];

fn lines(i: &Info, home: Option<&str>) -> Vec<(&'static str, Option<String>)> {
    let p = |s: &Option<String>| s.as_deref().map(|x| shorten_home(x, home));
    let release = if i.release_all { "vollständig (keine Auswahl)".to_string() } else if i.release.is_empty() { "unbekannt".into() } else { i.release.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join("; ") };
    vec![
        ("Version", Some(i.version.clone())),
        ("Commit", Some(i.commit.clone().unwrap_or_else(|| "unbekannt".into()))),
        ("System", Some(format!("{} {}", i.os, i.arch))),
        ("Installationsart", Some(i.package.label().to_string())),
        ("Installationsordner", p(&i.install_dir)),
        ("Datenordner", p(&i.data_dir)),
        ("Konfigurationsordner", p(&i.config_dir)),
        ("Zielsystem", Some(i.target.clone().unwrap_or_else(|| "kein Projekt offen".into()))),
        ("Release-Auswahl", Some(release)),
    ]
}

fn body_text(ls: &[(&'static str, Option<String>)], dropped: &[&'static str]) -> String {
    let mut b = String::from("\n\nAngaben zur Installation (automatisch eingetragen, bitte stehen lassen)\n----------------------------------------------------------------\n");
    for (k, v) in ls { if let (false, Some(v)) = (dropped.contains(k), v) { b.push_str(&format!("{k}: {v}\n")); } }
    if !dropped.is_empty() { b.push_str(&format!("Hinweis: wegen der Längengrenze von Mail-Links fehlen: {}.\n", dropped.join(", "))); }
    b
}

/// Mail aus den Angaben bilden. `home`: persönlicher Ordner (für `~`), `limit`: Höchstlänge des Links ([`MAX_URL`]).
pub fn build(i: &Info, home: Option<&str>, limit: usize) -> Mail {
    let subject = subject(i);
    let ls = lines(i, home);
    let body_full = body_text(&ls, &[]);
    let mut dropped: Vec<&'static str> = vec![];
    let mut body = body_full.clone();
    let mut url = mailto_url(&subject, &body);
    for d in DROP_ORDER {
        if url.len() <= limit { break; }
        dropped.push(d);
        body = body_text(&ls, &dropped);
        url = mailto_url(&subject, &body);
    }
    let too_long = url.len() > limit;
    Mail { subject, body, body_full, url, dropped, too_long }
}

/// Mail für das laufende Programm
pub fn current(target: Option<&str>) -> Mail { build(&Info::detect(target), vc_core::userdirs::home_dir().as_deref().and_then(Path::to_str), MAX_URL) }

impl Mail {
    /// Für `vcc feedback --json`
    pub fn to_json(&self, i: &Info) -> serde_json::Value {
        serde_json::json!({
            "to": ADDRESS, "subject": self.subject, "body": self.body, "body_full": self.body_full, "url": self.url, "url_length": self.url.len(),
            "shortened": self.dropped, "too_long": self.too_long,
            "info": { "version": i.version, "commit": i.commit, "os": i.os, "arch": i.arch, "package": i.package.key(), "install_dir": i.install_dir, "data_dir": i.data_dir,
                "config_dir": i.config_dir, "release_all": i.release_all, "release": i.release.iter().map(|(k, v)| serde_json::json!({ k: v })).collect::<Vec<_>>(), "target": i.target },
        })
    }
    /// Text für die Zwischenablage: Empfänger, Betreff, vollständiger Text
    pub fn clipboard(&self) -> String { format!("An: {ADDRESS}\nBetreff: {}\n{}", self.subject, self.body_full) }
}

/// Programm und Argumente, mit denen das Betriebssystem den Link im Standard-Mailprogramm öffnet.
/// Windows: `rundll32 url.dll,FileProtocolHandler` (kein `cmd /c start`: dort zerlegt `&` im Link den Aufruf).
pub fn open_command(os: &str, url: &str) -> (&'static str, Vec<String>) {
    match os {
        "macos" => ("open", vec![url.into()]),
        "windows" => ("rundll32", vec!["url.dll,FileProtocolHandler".into(), url.into()]),
        _ => ("xdg-open", vec![url.into()]),
    }
}

/// Link im Standard-Mailprogramm öffnen (startet nur das Programm, wartet nicht)
pub fn open_mailto(url: &str) -> Result<(), String> {
    let (prog, args) = open_command(std::env::consts::OS, url);
    std::process::Command::new(prog).args(&args).spawn().map(|_| ()).map_err(|e| format!("{prog}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> Info {
        Info { version: "0.1.0".into(), commit: Some("abc1234".into()), os: "macOS 27.2".into(), arch: "arm64".into(), package: Package::Source,
            install_dir: Some("/Users/anna/Development/VintageCoder/target/debug".into()), data_dir: Some("/Users/anna/Development/VintageCoder/data".into()),
            config_dir: Some("/Users/anna/.config/vintagecoder".into()), release: vec![], release_all: true, target: Some("c64".into()) }
    }

    #[test]
    fn betreff_ist_kompakt_und_lesbar() {
        assert_eq!(subject(&info()), "VintageCoder 0.1.0 (Commit abc1234) · macOS 27.2 arm64 · Paket: Quellbau (cargo)");
        let mut i = info(); i.commit = None;
        assert_eq!(subject(&i), "VintageCoder 0.1.0 · macOS 27.2 arm64 · Paket: Quellbau (cargo)");
    }

    #[test]
    fn text_hat_leere_zeile_oben_und_kuerzt_pfade() {
        let m = build(&info(), Some("/Users/anna"), MAX_URL);
        assert!(m.body.starts_with("\n\nAngaben zur Installation"));
        assert!(m.body.contains("Installationsordner: ~/Development/VintageCoder/target/debug\n"));
        assert!(m.body.contains("Konfigurationsordner: ~/.config/vintagecoder\n"));
        assert!(m.body.contains("Zielsystem: c64\n") && m.body.contains("Release-Auswahl: vollständig"));
        assert!(!m.body.contains("anna") && !m.url.contains("anna"), "kein Benutzername in der Mail");
        assert!(m.dropped.is_empty() && !m.too_long && m.body == m.body_full);
    }

    #[test]
    fn ohne_projekt_und_mit_release_auswahl() {
        let mut i = info(); i.target = None; i.release_all = false; i.release = vec![("Studios".into(), "sprite, tracker".into()), ("Zielsysteme".into(), "c64".into())];
        let m = build(&i, None, MAX_URL);
        assert!(m.body.contains("Zielsystem: kein Projekt offen") && m.body.contains("Release-Auswahl: Studios: sprite, tracker; Zielsysteme: c64"));
    }

    #[test]
    fn pfadkuerzung() {
        assert_eq!(shorten_home("/Users/anna/x/y", Some("/Users/anna")), "~/x/y");
        assert_eq!(shorten_home("/Users/anna", Some("/Users/anna/")), "~");
        assert_eq!(shorten_home("/Users/annabelle/x", Some("/Users/anna")), "~/x", "anderer Benutzerordner wird trotzdem gekürzt");
        assert_eq!(shorten_home("/home/bob/.config/vintagecoder", None), "~/.config/vintagecoder");
        assert_eq!(shorten_home(r"C:\Users\Bob\AppData\Roaming\vintagecoder", None), r"~\AppData\Roaming\vintagecoder");
        assert_eq!(shorten_home(r"C:\Users\Bob", Some(r"C:\Users\Bob")), "~");
        assert_eq!(shorten_home("/Applications/VintageCoder.app", Some("/Users/anna")), "/Applications/VintageCoder.app");
        assert_eq!(shorten_home("/opt/vintagecoder", None), "/opt/vintagecoder");
        assert_eq!(shorten_home(r"C:\Program Files\VintageCoder", Some(r"C:\Users\Bob")), r"C:\Program Files\VintageCoder");
    }

    #[test]
    fn url_kodierung() {
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("Größe ü"), "Gr%C3%B6%C3%9Fe%20%C3%BC");
        assert_eq!(percent_encode("x\ny\r\nz"), "x%0D%0Ay%0D%0Az");
        assert_eq!(percent_encode("A-z_0.9~"), "A-z_0.9~");
        assert_eq!(percent_encode("a&b=c?d#e+f%g"), "a%26b%3Dc%3Fd%23e%2Bf%25g");
        assert_eq!(percent_encode("·"), "%C2%B7");
        let u = mailto_url("Hi ü", "eins\nzwei");
        assert_eq!(u, "mailto:vintagecoder@drueeke.de?subject=Hi%20%C3%BC&body=eins%0D%0Azwei");
    }

    #[test]
    fn laengenbegrenzung_laesst_nicht_still_weg() {
        let mut i = info(); i.release_all = false;
        i.release = vec![("Studios".into(), "x".repeat(400))];
        let voll = build(&i, None, usize::MAX);
        assert!(voll.dropped.is_empty());
        let knapp = build(&i, None, voll.url.len() - 1);
        assert_eq!(knapp.dropped, vec!["Release-Auswahl"]);
        assert!(knapp.url.len() < voll.url.len() && !knapp.too_long);
        assert!(knapp.body.contains("Hinweis: wegen der Längengrenze von Mail-Links fehlen: Release-Auswahl."));
        assert!(knapp.body_full.contains(&"x".repeat(400)), "Zwischenablage bleibt vollständig");
        assert!(!knapp.body.contains("Release-Auswahl:"));
        // noch knapper: weitere Ordner entfallen der Reihe nach, Version/Commit/System/Art bleiben
        let winzig = build(&i, None, 10);
        assert_eq!(winzig.dropped, DROP_ORDER.to_vec());
        assert!(winzig.too_long && winzig.body.contains("Version: 0.1.0") && winzig.body.contains("Installationsart: Quellbau"));
        assert!(winzig.body.contains("Hinweis:") && winzig.body.contains("Installationsordner"));
        // Standardgrenze: eine normale Mail passt
        assert!(build(&info(), Some("/Users/anna"), MAX_URL).url.len() < MAX_URL);
    }

    #[test]
    fn quellpaket_heuristik() {
        let has = |set: &'static [&'static str]| move |p: &Path| set.iter().any(|s| p == Path::new(s));
        let p = |s: &str| vec![PathBuf::from(s)];
        assert_eq!(detect_package(&p(r"C:/Program Files/VintageCoder/vintagecoder.exe"), &has(&["C:/Program Files/VintageCoder/unins000.exe"])), Package::WindowsInstaller);
        assert_eq!(detect_package(&p("/Applications/VintageCoder.app/Contents/MacOS/vintagecoder"), &has(&[])), Package::MacApp);
        assert_eq!(detect_package(&p("/Users/a/dev/VintageCoder/target/debug/vintagecoder"), &has(&["/Users/a/dev/VintageCoder/Cargo.toml", "/Users/a/dev/VintageCoder/.git"])), Package::Source);
        assert_eq!(detect_package(&p("/Users/a/dev/VintageCoder/target/debug/vintagecoder"), &has(&["/Users/a/dev/VintageCoder/Cargo.toml"])), Package::Unknown, "ohne .git kein Quellbau");
        // Ziel-Ordner woanders: der Datenordner führt zum Quellbaum
        let both = vec![PathBuf::from("/cache/target/debug/vintagecoder"), PathBuf::from("/w/vc/data")];
        assert_eq!(detect_package(&both, &has(&["/w/vc/Cargo.toml", "/w/vc/.git"])), Package::Source);
        assert_eq!(detect_package(&p("/irgendwo/vintagecoder"), &has(&[])), Package::Unknown);
        assert_eq!(detect_package(&[], &has(&[])), Package::Unknown);
        #[cfg(not(windows))]
        assert_eq!(detect_package(&p("/usr/bin/vintagecoder"), &has(&[])), Package::LinuxPackage);
    }

    #[test]
    fn betriebssystem_angaben() {
        assert_eq!(parse_os_release("NAME=\"Fedora Linux\"\nPRETTY_NAME=\"Fedora Linux 43 (Workstation Edition)\"\nID=fedora\n").as_deref(), Some("Fedora Linux 43 (Workstation Edition)"));
        assert_eq!(parse_os_release("ID=x\n"), None);
        assert_eq!(parse_windows_ver("\r\nMicrosoft Windows [Version 10.0.22631.4602]\r\n").as_deref(), Some("10.0.22631"));
        assert_eq!(parse_windows_ver("Microsoft Windows [Version 10.0.19045]").as_deref(), Some("10.0.19045"));
        assert_eq!(parse_windows_ver("nichts"), None);
        assert_eq!(arch_label("aarch64"), "arm64");
        assert_eq!(arch_label("x86_64"), "x86_64");
    }

    #[test]
    fn mailprogramm_aufruf() {
        assert_eq!(open_command("macos", "mailto:x").0, "open");
        let (p, a) = open_command("windows", "mailto:x?a=1&b=2");
        assert_eq!(p, "rundll32"); assert_eq!(a, vec!["url.dll,FileProtocolHandler", "mailto:x?a=1&b=2"]);
        assert_eq!(open_command("linux", "mailto:x").0, "xdg-open");
    }

    #[test]
    fn laufendes_programm_liefert_angaben() {
        let i = Info::detect(Some("auto"));
        assert_eq!(i.version, env!("CARGO_PKG_VERSION"));
        assert!(i.target.is_none() && !i.os.is_empty() && !i.arch.is_empty());
        let m = current(Some("c64"));
        assert!(m.url.starts_with("mailto:vintagecoder@drueeke.de?subject=") && m.subject.starts_with("VintageCoder "));
        let j = m.to_json(&i);
        assert_eq!(j["to"], ADDRESS);
        assert!(m.clipboard().starts_with("An: vintagecoder@drueeke.de\nBetreff: VintageCoder "));
    }
}
