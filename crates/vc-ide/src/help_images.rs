// SPDX-License-Identifier: GPL-3.0-or-later
//! Bilder des Handbuchs: nicht in der Programmdatei, sondern online nachgeladen (`<Basis>/<Kennung>.png`) und im Zwischenspeicher abgelegt.
//!
//! Datenschutz: Es geht nie etwas ohne Zustimmung ins Netz (siehe [`Consent`]); gesendet wird nur die Adresse des Bildes, nichts vom Benutzer.
//! Die Basis ist `VC_HELP_IMAGES` (Vorrang), sonst `help.images_url` aus den Einstellungen, sonst [`DEFAULT_BASE`]. Ist die Basis ein Ordner
//! (kein `http://`/`https://`), wird gelesen statt geladen (lokale Tests auf `docs/handbuch/bilder`); das braucht keine Zustimmung.
//! Geladen wird mit `curl` (wie bei den Emulatorpaketen, `vc_emu::install`); kein eigener HTTP-Client in der Programmdatei.
use std::path::{Path, PathBuf};
use std::process::Command;

/// Standardadresse der Bilder (die Seite ist noch nicht veröffentlicht; bis dahin meldet das Laden einen Fehler und die Hilfe zeigt Platzhalter)
pub const DEFAULT_BASE: &str = "https://vintagecoder.drueeke.de/handbuch/bilder";
/// Größtes erlaubtes Bild (Bytes)
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Wahl des Benutzers zum Nachladen aus dem Internet
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consent { Ask, Always, Never }

impl Consent {
    pub fn from_setting(s: &str) -> Consent { match s { "always" => Consent::Always, "never" => Consent::Never, _ => Consent::Ask } }
    pub fn setting(self) -> &'static str { match self { Consent::Ask => "", Consent::Always => "always", Consent::Never => "never" } }
}

/// Wirksame Basis: Umgebung, Einstellung, Standard (ohne Schrägstrich am Ende)
pub fn base() -> String { resolve_base(std::env::var("VC_HELP_IMAGES").ok().as_deref(), &crate::settings::get().help.images_url) }

pub fn resolve_base(env: Option<&str>, setting: &str) -> String {
    let b = env.map(str::trim).filter(|b| !b.is_empty()).unwrap_or_else(|| { let s = setting.trim(); if s.is_empty() { DEFAULT_BASE } else { s } });
    b.trim_end_matches('/').to_string()
}

/// Kommt das Bild aus dem Netz (`http://`/`https://`) und braucht deshalb Zustimmung?
pub fn is_remote(base: &str) -> bool { base.starts_with("https://") || base.starts_with("http://") }

/// Nur Kennungen aus Kleinbuchstaben, Ziffern, `-` und `_` (kein Pfad, keine Adressteile)
pub fn valid_id(id: &str) -> bool { !id.is_empty() && id.len() <= 80 && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') }

/// Zwischenspeicher: `VC_HELP_CACHE`, sonst `cache/handbuch` im Konfigurationsordner
pub fn cache_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VC_HELP_CACHE").filter(|d| !d.is_empty()) { return Some(d.into()); }
    Some(vc_core::userdirs::config_dir()?.join("cache").join("handbuch"))
}

/// Adresse bzw. Dateipfad des Bildes
pub fn image_location(base: &str, id: &str) -> String { format!("{base}/{id}.png") }

fn is_png(b: &[u8]) -> bool { b.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) }

fn curl() -> PathBuf {
    if cfg!(windows) { if let Some(root) = std::env::var_os("SystemRoot") { let p = Path::new(&root).join("System32").join("curl.exe"); if p.is_file() { return p; } } }
    PathBuf::from("curl")
}

/// Bild liegt schon im Zwischenspeicher (oder lokal)? Ohne Netz.
pub fn cached(id: &str) -> Option<Vec<u8>> {
    if !valid_id(id) { return None; }
    let b = base();
    if !is_remote(&b) { return read_local(&b, id).ok(); }
    std::fs::read(cache_dir()?.join(format!("{id}.png"))).ok().filter(|d| is_png(d))
}

/// Wie [`cached`], ohne die Datei zu lesen (billig genug für jedes Bild der Oberfläche)
pub fn is_cached(id: &str) -> bool {
    if !valid_id(id) { return false; }
    let b = base();
    if !is_remote(&b) { return Path::new(b.strip_prefix("file://").unwrap_or(&b)).join(format!("{id}.png")).is_file(); }
    cache_dir().map_or(false, |d| d.join(format!("{id}.png")).is_file())
}

fn read_local(base: &str, id: &str) -> Result<Vec<u8>, String> {
    let dir = base.strip_prefix("file://").unwrap_or(base);
    let p = Path::new(dir).join(format!("{id}.png"));
    let d = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    if is_png(&d) { Ok(d) } else { Err(format!("{}: keine PNG-Datei", p.display())) }
}

/// Holt die PNG-Daten des Bildes: Zwischenspeicher, sonst lokaler Ordner bzw. Download (blockiert; im Hintergrund-Thread aufrufen).
/// Aufrufer prüfen vorher die Zustimmung ([`is_remote`]). Fehler sind Text, nie ein Absturz.
pub fn fetch(id: &str) -> Result<Vec<u8>, String> { fetch_from(&base(), cache_dir().as_deref(), id) }

pub fn fetch_from(base: &str, cache: Option<&Path>, id: &str) -> Result<Vec<u8>, String> {
    if !valid_id(id) { return Err(format!("ungültige Bildkennung „{id}“")); }
    if !is_remote(base) { return read_local(base, id); }
    let file = cache.map(|c| c.join(format!("{id}.png")));
    if let Some(f) = &file { if let Ok(d) = std::fs::read(f) { if is_png(&d) { return Ok(d); } } }
    let url = image_location(base, id);
    if let Some(c) = cache { std::fs::create_dir_all(c).map_err(|e| format!("{}: {e}", c.display()))?; }
    let out = match cache { Some(c) => c.join(format!("{id}.png.part")), None => std::env::temp_dir().join(format!("vc-help-{}-{id}.png.part", std::process::id())) };
    // nur http(s), Weiterleitungen nur auf https, begrenzte Größe und Zeit; keine Cookies, keine Zugangsdaten
    let res = Command::new(curl()).args(["--fail", "--location", "--silent", "--show-error", "--proto", "=https,http", "--proto-redir", "=https", "--max-filesize", &MAX_BYTES.to_string(), "--max-time", "60", "--connect-timeout", "15", "--output"]).arg(&out).arg(&url).output()
        .map_err(|e| format!("curl nicht ausführbar: {e}"))?;
    if !res.status.success() {
        let _ = std::fs::remove_file(&out);
        let msg = String::from_utf8_lossy(&res.stderr).lines().last().unwrap_or("").trim().to_string();
        return Err(format!("{url}: {}", if msg.is_empty() { "Download fehlgeschlagen".to_string() } else { msg }));
    }
    let d = std::fs::read(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    if !is_png(&d) { let _ = std::fs::remove_file(&out); return Err(format!("{url}: keine PNG-Datei")); }
    match &file { Some(f) => { std::fs::rename(&out, f).map_err(|e| format!("{}: {e}", f.display()))?; } None => { let _ = std::fs::remove_file(&out); } }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bilder() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/handbuch/bilder") }

    #[test]
    fn basis_reihenfolge_umgebung_einstellung_standard() {
        assert_eq!(resolve_base(None, ""), DEFAULT_BASE);
        assert_eq!(resolve_base(None, " https://x.example/b/ "), "https://x.example/b");
        assert_eq!(resolve_base(Some("/tmp/bilder/"), "https://x.example/b"), "/tmp/bilder");
        assert_eq!(resolve_base(Some("  "), ""), DEFAULT_BASE);
        assert!(is_remote(DEFAULT_BASE) && !is_remote("/tmp/bilder") && !is_remote("file:///tmp"));
    }

    #[test]
    fn kennungen_sind_nur_einfache_namen() {
        for ok in ["ui-gesamt", "a_b-1"] { assert!(valid_id(ok), "{ok}"); }
        for bad in ["", "../x", "a/b", "A", "x.png", "a b", "a?b=1"] { assert!(!valid_id(bad), "{bad}"); }
        assert!(fetch_from("https://example.invalid", None, "../etc/passwd").is_err());
    }

    #[test]
    fn lokaler_ordner_liefert_png_ohne_netz() {
        let d = fetch_from(bilder().to_str().unwrap(), None, "ui-gesamt").unwrap();
        assert!(is_png(&d) && d.len() > 1000);
        assert!(fetch_from(&format!("file://{}", bilder().display()), None, "ui-gesamt").is_ok());
    }

    #[test]
    fn fehlendes_bild_ist_ein_fehler_kein_absturz() {
        let e = fetch_from(bilder().to_str().unwrap(), None, "gibt-es-nicht").unwrap_err();
        assert!(e.contains("gibt-es-nicht"), "{e}");
        // Adresse nicht erreichbar (reservierte Domain .invalid): Fehlertext, kein Absturz, nichts im Zwischenspeicher
        let tmp = std::env::temp_dir().join(format!("vc-help-test-{}", std::process::id()));
        let e = fetch_from("https://nicht-erreichbar.invalid/bilder", Some(&tmp), "ui-gesamt");
        assert!(e.is_err());
        assert!(!tmp.join("ui-gesamt.png").exists() && !tmp.join("ui-gesamt.png.part").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn zwischenspeicher_hat_vorrang_vor_dem_netz() {
        let tmp = std::env::temp_dir().join(format!("vc-help-test-cache-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::copy(bilder().join("ui-gesamt.png"), tmp.join("ui-gesamt.png")).unwrap();
        assert!(fetch_from("https://nicht-erreichbar.invalid/bilder", Some(&tmp), "ui-gesamt").is_ok());
        std::fs::write(tmp.join("kaputt.png"), b"kein png").unwrap();
        assert!(fetch_from("https://nicht-erreichbar.invalid/bilder", Some(&tmp), "kaputt").is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn einwilligung_aus_der_einstellung() {
        assert_eq!(Consent::from_setting(""), Consent::Ask);
        assert_eq!(Consent::from_setting("always"), Consent::Always);
        assert_eq!(Consent::from_setting("never"), Consent::Never);
        for c in [Consent::Ask, Consent::Always, Consent::Never] { assert_eq!(Consent::from_setting(c.setting()), c); }
    }
}
