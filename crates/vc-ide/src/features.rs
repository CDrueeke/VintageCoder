// SPDX-License-Identifier: GPL-3.0-or-later
//! Auswahl eines Releases zur Laufzeit: liest `data/release.json` (vom Paketierungsskript `tools/release.py` geschrieben, Aufbau siehe
//! Konzept „Release-Auswahl“, Anhang B). **Regeln:** Datei fehlt → alles an (Entwicklung, `cargo test`). Schlüssel fehlt → alles an.
//! Schlüssel vorhanden → nur die genannten Einträge. Ein unbekanntes `format` (größer als [`FORMAT`]) gilt als Mindestauswahl
//! (nichts Wählbares an) und erzeugt eine Warnung. Die Umgebungsvariable `VC_RELEASE` überschreibt: `all` = alles an,
//! sonst Pfad einer anderen `release.json`.
//!
//! Die Kennungen sind die Kurzformen der Registry (`sprite`, `tracker`, `vice`, `atari8`), nicht die Kennungen aus `FEATURES.md`;
//! [`Features::has`] versteht beides (`STU-SPRITE`, `SYS-ATARI`).
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Höchstes bekanntes Format von `release.json`
pub const FORMAT: u64 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Features {
    /// IDE (`vintagecoder`) gewählt; `None` = alles an
    ide: Option<bool>,
    studios: Option<BTreeSet<String>>,
    links: Option<BTreeSet<String>>,
    targets: Option<BTreeSet<String>>,
    examples: Option<BTreeSet<String>>,
    /// Warnung beim Einlesen (unbekanntes Format, kaputte Datei)
    pub warning: Option<String>,
}

fn set_of(v: &serde_json::Value, key: &str) -> Option<BTreeSet<String>> {
    v.get(key)?.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
}

/// Kennung aus `FEATURES.md` (`SYS-ATARI`) → Kurzform der Registry (`atari8`) für Systeme
fn system_id(k: &str) -> Option<&'static str> {
    Some(match k { "SYS-C64" => "c64", "SYS-C128" => "c128", "SYS-PLUS4" => "plus4", "SYS-C16" => "c16", "SYS-VC20" => "vic20", "SYS-ATARI" => "atari8", "SYS-MEGA65" => "mega65", "SYS-PET" => "pet", "SYS-CBM2" => "cbm2", _ => return None })
}

impl Features {
    /// Alles an (Entwicklung, Tests)
    pub fn all() -> Features { Features::default() }

    /// Aus dem Text einer `release.json`; ein Fehler ergibt eine Mindestauswahl mit Warnung (nie „alles an“: ein kaputtes Release soll nicht mehr zeigen)
    pub fn from_json(text: &str) -> Features {
        let minimal = |w: String| Features { ide: Some(false), studios: Some(BTreeSet::new()), links: Some(BTreeSet::new()), targets: Some(BTreeSet::new()), examples: Some(BTreeSet::new()), warning: Some(w) };
        let v: serde_json::Value = match serde_json::from_str(text) { Ok(v) => v, Err(e) => return minimal(format!("release.json: {e}")) };
        let format = v["format"].as_u64().unwrap_or(1);
        if format > FORMAT { return minimal(format!("release.json: Format {format} ist neuer als bekannt ({FORMAT}); es gilt die Mindestauswahl")); }
        Features { ide: v.get("ide").and_then(|x| x.as_bool()), studios: set_of(&v, "studios"), links: set_of(&v, "links"), targets: set_of(&v, "targets"), examples: set_of(&v, "examples"), warning: None }
    }

    pub fn load(path: &Path) -> Features {
        match std::fs::read_to_string(path) {
            Ok(t) => Features::from_json(&t),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Features::all(),
            Err(e) => Features { warning: Some(format!("{}: {e}", path.display())), ..Features::all() },
        }
    }

    /// Die Auswahl in lesbarer Form (Bezeichnung, Werte) für Hinweise und Rückmeldungen; leer, wenn alles an ist.
    pub fn describe(&self) -> Vec<(&'static str, String)> {
        let list = |s: &Option<BTreeSet<String>>| s.as_ref().map(|s| if s.is_empty() { "keine".to_string() } else { s.iter().cloned().collect::<Vec<_>>().join(", ") });
        let mut v = vec![];
        if let Some(i) = self.ide { v.push(("IDE", (if i { "ja" } else { "nein" }).to_string())); }
        for (k, s) in [("Studios", &self.studios), ("Anbindungen", &self.links), ("Zielsysteme", &self.targets), ("Beispiele", &self.examples)] { if let Some(t) = list(s) { v.push((k, t)); } }
        v
    }

    /// Alles an? (keine `release.json`)
    pub fn is_all(&self) -> bool { *self == Features::all() }
    pub fn ide(&self) -> bool { self.ide.unwrap_or(true) }
    /// Studio gewählt? (`sprite`, `char`, `screen`, `tiles`, `palette`, `tracker`, `play`, `disk`, `memmap`, `extract`). Ohne IDE gibt es keine Studios.
    pub fn studio(&self, id: &str) -> bool { self.ide() && self.studios.as_ref().map_or(true, |s| s.contains(id)) }
    /// Anbindung (Emulator, Gerät) gewählt? (`vice`, `atari800`, `altirra`, `fujisan`, `xemu`, `ultimate`, `mega65`)
    pub fn link(&self, id: &str) -> bool { self.links.as_ref().map_or(true, |s| s.contains(id)) }
    /// Ist mindestens eine Anbindung gewählt, mit der sich debuggen lässt? (`caps.debug` im Emulator-Katalog; Altirra startet nur)
    pub fn any_debugger(&self) -> bool { crate::emulators::any_debug_in(self) }
    /// Ist mindestens eine Anbindung mit Echtzeit-Bild gewählt (Reiter „Live-Bild“)? (`caps.live_image`: die Ultimate)
    pub fn any_live_image(&self) -> bool { crate::emulators::any_live_in(self) }
    /// Zielsystem gewählt? (`c64`, `atari8` …)
    pub fn target(&self, id: &str) -> bool { self.targets.as_ref().map_or(true, |s| s.contains(id)) }
    /// Beispielpaket gewählt? (`EX-HELLO` …)
    pub fn examples(&self, pkg: &str) -> bool { self.examples.as_ref().map_or(true, |s| s.contains(pkg)) }
    /// Gibt es überhaupt Beispielpakete?
    pub fn any_examples(&self) -> bool { self.examples.as_ref().map_or(true, |s| !s.is_empty()) }
    /// Ist mindestens ein Studio gewählt?
    pub fn any_studio(&self, ids: &[&str]) -> bool { ids.iter().any(|i| self.studio(i)) }

    /// Kennung wie in `FEATURES.md` (`STU-TILES`, `SYS-ATARI`, `IDE-APP`) oder Kurzform mit Art (`studio:tiles`)
    pub fn has(&self, kennung: &str) -> bool {
        if let Some(id) = kennung.strip_prefix("STU-") { return self.studio(&id.to_ascii_lowercase()); }
        if let Some(id) = system_id(kennung) { return self.target(id); }
        if kennung == "IDE-APP" { return self.ide(); }
        if let Some(id) = kennung.strip_prefix("EMU-").or_else(|| kennung.strip_prefix("HW-")) { return self.link(&id.to_ascii_lowercase()); }
        true
    }
    /// Mindestens eine der Kennungen an; eine leere Liste gilt als „immer sichtbar“
    pub fn has_any(&self, kennungen: &[&str]) -> bool { kennungen.is_empty() || kennungen.iter().any(|k| self.has(k)) }
}

static CURRENT: RwLock<Option<Arc<Features>>> = RwLock::new(None);

/// Die Auswahl dieses Programmlaufs. Vor [`init`] (und ohne `VC_RELEASE`) gilt „alles an“.
pub fn features() -> Arc<Features> {
    if let Some(f) = CURRENT.read().ok().and_then(|g| g.clone()) { return f; }
    Arc::new(from_env().unwrap_or_default())
}

fn from_env() -> Option<Features> {
    let v = std::env::var("VC_RELEASE").ok()?;
    let v = v.trim();
    if v.is_empty() { return None; }
    Some(if v.eq_ignore_ascii_case("all") { Features::all() } else { Features::load(Path::new(v)) })
}

/// Auswahl festlegen: `VC_RELEASE` (`all` oder Pfad) hat Vorrang, sonst `<data>/release.json` (fehlt sie: alles an). Gibt die Warnung zurück.
pub fn init(data: Option<&Path>) -> Option<String> {
    let f = from_env().unwrap_or_else(|| data.map(|d| Features::load(&d.join("release.json"))).unwrap_or_default());
    let w = f.warning.clone();
    if let Ok(mut g) = CURRENT.write() { *g = Some(Arc::new(f)); }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datei_fehlt_alles_an() {
        let f = Features::load(Path::new("/nicht/vorhanden/release.json"));
        assert!(f.is_all() && f.studio("sprite") && f.target("c64") && f.link("vice") && f.examples("EX-HELLO") && f.ide());
    }
    #[test]
    fn schluessel_fehlt_alles_an_liste_nur_genannte() {
        let f = Features::from_json(r#"{"format":1,"studios":["sprite"],"targets":["atari8"]}"#);
        assert!(f.studio("sprite") && !f.studio("tracker"));
        assert!(f.target("atari8") && !f.target("c64"));
        assert!(f.link("vice") && f.examples("EX-DEMOS"), "fehlende Schlüssel: alles an");
        assert!(f.has("STU-SPRITE") && !f.has("STU-TILES") && f.has("SYS-ATARI") && !f.has("SYS-C64"));
        assert!(f.has_any(&[]) && f.has_any(&["STU-TILES", "STU-SPRITE"]) && !f.has_any(&["STU-TILES"]));
    }
    #[test]
    fn anbindungen_nur_die_gewaehlten() {
        let f = Features::from_json(r#"{"format":1,"links":["vice"]}"#);
        assert!(f.link("vice") && !f.link("atari800") && !f.link("ultimate") && f.any_debugger());
        assert!(f.has("EMU-VICE") && !f.has("EMU-XEMU") && !f.has("HW-ULTIMATE") && !f.has("HW-MEGA65"));
        let g = Features::from_json(r#"{"format":1,"links":["altirra"]}"#);
        assert!(!g.any_debugger(), "Altirra startet nur");
        assert!(Features::all().any_debugger());
    }
    #[test]
    fn ohne_ide_keine_studios() {
        let f = Features::from_json(r#"{"format":1,"ide":false,"studios":["sprite"]}"#);
        assert!(!f.ide() && !f.studio("sprite"));
    }
    #[test]
    fn unbekanntes_format_und_kaputte_datei_sind_mindestauswahl() {
        for t in [r#"{"format":99,"studios":["sprite"]}"#, "{kaputt"] {
            let f = Features::from_json(t);
            assert!(f.warning.is_some() && !f.studio("sprite") && !f.target("c64") && !f.link("vice") && !f.any_examples(), "{t}");
        }
    }
}
