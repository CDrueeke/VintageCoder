// SPDX-License-Identifier: GPL-3.0-or-later
//! Orte persönlicher Dateien (Einstellungen, Fensteranordnung, Ansicht): unter Windows `%APPDATA%\vintagecoder`,
//! sonst `~/.config/vintagecoder`.
use std::path::PathBuf;

/// Persönlicher Ordner des Benutzers: `HOME`, unter Windows sonst `USERPROFILE`
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").filter(|h| !h.is_empty()).or_else(|| std::env::var_os("USERPROFILE").filter(|h| !h.is_empty())).map(PathBuf::from)
}

/// Läuft dieser Prozess als Testprogramm von `cargo test` (Programmdatei unter `target/<profil>/deps`, Cargo-Umgebung gesetzt)?
fn laeuft_als_cargo_test() -> bool {
    std::env::var_os("CARGO_MANIFEST_DIR").is_some() && std::env::current_exe().ok().is_some_and(|e| e.parent().and_then(|p| p.file_name()).is_some_and(|n| n == "deps"))
}

/// Ordner für Einstellungen und Layout (wird beim Schreiben bei Bedarf angelegt).
/// `VC_CONFIG_DIR` ersetzt ihn (Tests, Prüfpakete); in einem Testprogramm von `cargo test` ohne diese Variable liefert er einen eigenen Temp-Ordner,
/// **nie** die echte Konfiguration des Benutzers (Tests dürfen dort nichts lesen oder überschreiben).
pub fn config_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VC_CONFIG_DIR").filter(|d| !d.is_empty()) { return Some(PathBuf::from(d)); }
    if laeuft_als_cargo_test() { return Some(std::env::temp_dir().join(format!("vc-test-config-{}", std::process::id()))); }
    #[cfg(windows)]
    if let Some(a) = std::env::var_os("APPDATA").filter(|a| !a.is_empty()) { return Some(PathBuf::from(a).join("vintagecoder")); }
    unix_config_dir()
}

#[cfg(target_os = "linux")]
fn unix_config_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from);
    linux_config_dir(xdg.as_deref(), home_dir().as_deref())
}

#[cfg(not(target_os = "linux"))]
fn unix_config_dir() -> Option<PathBuf> { Some(home_dir()?.join(".config").join("vintagecoder")) }

/// Linux: `$XDG_CONFIG_HOME/vintagecoder` (nur absolute Pfade, wie in der XDG-Spezifikation), sonst `~/.config/vintagecoder`.
/// Gab es bisher nur den alten Ordner unter `~/.config` und im XDG-Ordner noch keinen, bleibt es dabei (keine stille Verlagerung der Einstellungen).
pub fn linux_config_dir(xdg: Option<&std::path::Path>, home: Option<&std::path::Path>) -> Option<PathBuf> {
    let legacy = home.map(|h| h.join(".config").join("vintagecoder"));
    match xdg.filter(|x| x.is_absolute()) {
        Some(x) => {
            let new = x.join("vintagecoder");
            match &legacy { Some(l) if !new.exists() && l.exists() => legacy, _ => Some(new) }
        }
        None => legacy,
    }
}

/// `~/` am Anfang eines Pfades durch den persönlichen Ordner ersetzen
pub fn expand_home(p: &str) -> String {
    match (p.strip_prefix("~/"), home_dir()) { (Some(rest), Some(h)) => h.join(rest).to_string_lossy().into_owned(), _ => p.to_string() }
}

#[cfg(test)]
mod tests {
    #[test] fn tests_beruehren_nie_die_echte_konfiguration() {
        if std::env::var_os("VC_CONFIG_DIR").is_some() { return; }
        let d = config_dir().unwrap(); assert!(d.starts_with(std::env::temp_dir()), "{}", d.display());
    }
    use super::*;

    #[test]
    fn expands_home_only_at_the_start() {
        let h = home_dir().expect("HOME oder USERPROFILE gesetzt");
        assert!(expand_home("~/x/y").starts_with(h.to_string_lossy().as_ref()));
        assert!(expand_home("~/x/y").ends_with("y"));
        assert_eq!(expand_home("/a/~/b"), "/a/~/b");
        assert_eq!(expand_home("ohne"), "ohne");
    }

    #[test]
    fn linux_xdg_regeln() {
        // Windows: ein absoluter Pfad braucht ein Laufwerk (sonst gilt "/x" als nicht absolut)
        let (h, x) = if cfg!(windows) { (std::path::Path::new("C:/nonexistent-h"), std::path::Path::new("C:/nonexistent-x")) } else { (std::path::Path::new("/nonexistent-h"), std::path::Path::new("/nonexistent-x")) };
        assert_eq!(linux_config_dir(None, Some(h)).unwrap(), h.join(".config/vintagecoder"));
        assert_eq!(linux_config_dir(Some(x), Some(h)).unwrap(), x.join("vintagecoder"));
        assert_eq!(linux_config_dir(Some(std::path::Path::new("relativ")), Some(h)).unwrap(), h.join(".config/vintagecoder"));
    }

    /// Hermetisch: die Umgebungsvariable wird im Test gesetzt und entfernt (nicht vom Aufrufer geerbt) und danach wiederhergestellt;
    /// beide Werte liegen im Temp-Ordner, damit der Test `tests_beruehren_nie_die_echte_konfiguration` nebenher nichts anderes sieht.
    #[test]
    fn config_dir_is_below_a_user_folder() {
        let alt = std::env::var_os("VC_CONFIG_DIR");
        let eigen = std::env::temp_dir().join(format!("vc-test-config-explizit-{}", std::process::id()));
        std::env::set_var("VC_CONFIG_DIR", &eigen);
        assert_eq!(config_dir().unwrap(), eigen, "VC_CONFIG_DIR gilt");
        std::env::remove_var("VC_CONFIG_DIR");
        let d = config_dir().expect("Konfigurationsordner");
        assert!(d.to_string_lossy().contains("vc-test-config"), "{}", d.display());
        match alt { Some(v) => std::env::set_var("VC_CONFIG_DIR", v), None => {} }
    }
}
