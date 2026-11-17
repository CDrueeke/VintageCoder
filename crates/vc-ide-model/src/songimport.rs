// SPDX-License-Identifier: GPL-3.0-or-later
//! „Konvertieren von …“ und „RMT importieren …“ ohne Oberfläche: Quelle prüfen und beschreiben (SID-Datei, GoatTracker `.sng`, Song eines anderen Chips,
//! RMT-Modul), Song in das Profil des offenen Trackers umwandeln bzw. importieren und als neue Datei im Projekt anlegen. Die Umwandlung selbst steckt in `vc_media`.
use serde_json::Value;
use std::path::Path;

/// Was die gewählte Quelldatei ist (für den Dialog)
#[derive(Clone, Debug, PartialEq)]
pub struct Quelle {
    /// Anzeige „SID-Datei (Aufnahme der Spielroutine)“, „GoatTracker (SID)“, „POKEY (Atari)“, …
    pub art: String,
    /// Vorschlag für den neuen Namen (Dateiname ohne Endung, Leerzeichen → _)
    pub name: String,
    /// bei SID-Dateien: Beschreibung, Zahl der Subtunes, Startsong
    pub sid: Option<(String, u16, u16)>,
}

/// Quelle untersuchen; Fehler (unlesbare SID-Datei u. a.) sind ein Text für den Anwender
pub fn quelle_pruefen(f: &Path) -> Result<Quelle, String> {
    let fname = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(); let low = fname.to_lowercase();
    let name = vc_media::naming::stem(&fname).trim_end_matches(".sng").trim_end_matches(".sid").replace(' ', "_");
    if low.ends_with(".sid") {
        let i = std::fs::read(f).map_err(|e| format!("{fname}: {e}")).and_then(|b| vc_media::sid::info(&b).map(|x| x.0).map_err(|e| format!("{fname}: {e}")))?;
        return Ok(Quelle { art: "SID-Datei (Aufnahme der Spielroutine)".into(), name, sid: Some((format!("{} – {} ({} Subtunes)", i.title, i.author, i.songs), i.songs, i.start_song)) });
    }
    let art = if low.ends_with(".sng") { "GoatTracker (SID)".to_string() } else {
        match std::fs::read_to_string(f).ok().and_then(|t| vc_media::parse_media(&t).ok()).map(|a| a.profile) { Some(p) if p.contains("pokey") => "POKEY (Atari)".into(), Some(p) if p.contains("sid") => "SID (C64)".into(), Some(p) => p, None => "unbekannt".into() }
    };
    Ok(Quelle { art, name, sid: None })
}

/// Ergebnis: Pfad der neuen Datei (relativ zum Projekt) und Hinweise der Umwandlung
pub type Ergebnis = Result<(String, Vec<String>), String>;

fn ergebnis(v: &Value) -> (String, Vec<String>) {
    (v["path"].as_str().unwrap_or("").to_string(), v["warnings"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default())
}

/// Song, SID-Datei oder GoatTracker-Datei in das Profil `ziel_profil` umwandeln und als `<ordner>/<name>.vcsong` anlegen (bei SID: `subtune` 1-basiert, 0 = Startsong, `sekunden` Aufnahmelänge)
pub fn konvertieren(root: &Path, data: &Path, quelle: &Path, ziel_profil: &str, name: &str, ordner: &str, subtune: u16, sekunden: u32) -> Ergebnis {
    let opts = vc_media::sid::SidOpts { song: subtune, seconds: sekunden };
    vc_media::project::import_song_with(root, data, quelle, ziel_profil, name, ordner, &opts).map(|v| ergebnis(&v))
}

/// RMT-Modul als POKEY-Song (`atari.pokey.song`) importieren
pub fn rmt_importieren(root: &Path, data: &Path, datei: &Path, name: &str, ordner: &str) -> Ergebnis {
    let bytes = std::fs::read(datei).map_err(|e| format!("{}: {e}", datei.display()))?;
    let profiles = vc_media::load_profiles(data)?; let p = vc_media::profile(&profiles, "atari.pokey.song")?;
    let (v, w) = vc_media::song::rmt_import(&bytes, p, name)?;
    let rel = format!("{ordner}/{name}.vcsong"); let out = root.join(&rel);
    if out.exists() { return Err(format!("{rel} existiert bereits")) }
    std::fs::create_dir_all(out.parent().ok_or("Pfad")?).map_err(|e| e.to_string())?; std::fs::write(&out, vc_media::project::format_media(&v)).map_err(|e| e.to_string())?; Ok((rel, w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
    fn projekt(tag: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-songimport-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("music")).unwrap(); d }

    #[test]
    fn songs_zwischen_systemen_umwandeln_und_nicht_ueberschreiben() {
        let (data, dir) = (repo().join("data"), projekt("a")); std::fs::copy(repo().join("examples/c64/music_demo/music/tune.vcsong"), dir.join("music/tune.vcsong")).unwrap();
        // SID → POKEY im eigenen Projekt
        let r = konvertieren(&dir, &data, &dir.join("music/tune.vcsong"), "atari.pokey.song", "tune_p", "music", 0, 30).unwrap(); assert_eq!(r.0, "music/tune_p.vcsong");
        let v: Value = serde_json::from_str(&std::fs::read_to_string(dir.join(&r.0)).unwrap()).unwrap(); assert_eq!(v["profile"], "atari.pokey.song");
        // Quelle außerhalb des Projekts (anderes System), Ziel = SID-Profil
        let fremd = std::env::temp_dir().join(format!("vc-songimport-fremd-{}", std::process::id())); std::fs::create_dir_all(&fremd).unwrap(); std::fs::copy(dir.join(&r.0), fremd.join("aus atari.vcsong")).unwrap();
        let q = quelle_pruefen(&fremd.join("aus atari.vcsong")).unwrap(); assert_eq!((q.art.as_str(), q.name.as_str(), q.sid.is_none()), ("POKEY (Atari)", "aus_atari", true));
        let i = konvertieren(&dir, &data, &fremd.join("aus atari.vcsong"), "c64.sid.song", "aus_atari", "music", 0, 30).unwrap(); assert_eq!(i.0, "music/aus_atari.vcsong");
        assert!(konvertieren(&dir, &data, &fremd.join("aus atari.vcsong"), "c64.sid.song", "aus_atari", "music", 0, 30).is_err(), "vorhandenes wird nicht überschrieben");
        assert!(konvertieren(&dir, &data, &fremd.join("aus atari.vcsong"), "c64.sid.song", "falsch name!", "music", 0, 30).is_err(), "Name wird geprüft");
        let _ = (std::fs::remove_dir_all(&dir), std::fs::remove_dir_all(&fremd));
    }

    #[test]
    fn quellen_erkennen_und_fehler_melden() {
        let dir = projekt("b"); std::fs::write(dir.join("kaputt.sid"), b"nicht wirklich").unwrap(); std::fs::write(dir.join("lied.sng"), b"GTS5").unwrap();
        assert!(quelle_pruefen(&dir.join("kaputt.sid")).is_err(), "unlesbare SID-Datei: verständlicher Fehler");
        let q = quelle_pruefen(&dir.join("lied.sng")).unwrap(); assert_eq!((q.art.as_str(), q.name.as_str()), ("GoatTracker (SID)", "lied"));
        assert!(rmt_importieren(&dir, &repo().join("data"), &dir.join("lied.sng"), "x", "music").is_err(), "kein RMT-Modul: Fehler statt Datei");
        assert!(!dir.join("music/x.vcsong").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
