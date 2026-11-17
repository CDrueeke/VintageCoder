// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungen der Oberfläche (Farbschema, Zoom, Editor-Schrift, Umbruch); als JSON im Konfigurationsordner, ohne Netz und ohne Kennung.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Schema { Dunkel, Hell, System, Kontrast }

/// Tastaturlayout für die Beschriftung der Klaviertasten im Tracker (die Belegung selbst gilt nach der physischen Tastenposition): Automatisch = vom System erkannt
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Tastaturlayout { #[default] Auto, Qwertz, Qwerty, Azerty }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Einstellungen { pub schema: Schema, pub zoom: f32, pub editor_schrift: f32, pub umbruch: bool, pub seitenleiste: bool, pub schublade: bool, pub panel: bool, pub panel_reiter: u8,
    /// Höhe der Debug-Fläche (Anteil der Fensterhöhe zwischen Kopfzeile und Statusleiste), per Trennlinie verstellbar und gemerkt
    pub debug_hoehe: f32,
    /// Einzeltasten der Studios (Buchstaben/Ziffern: Werkzeuge, Noten, Zeichen) an oder aus; Cmd-/Alt-Kürzel gelten immer
    #[serde(default = "wahr")] pub studio_einzeltasten: bool,
    /// Layout für die Beschriftung der Klaviertasten und die Hinweise (Auto = erkennen) und ob die Klaviatur-Karte die Computertaste auf jeder Taste zeigt
    pub tastaturlayout: Tastaturlayout, #[serde(default = "wahr")] pub tasten_auf_klaviatur: bool,
    /// Takt, Norm, Zeilen und Zyklen des Zielsystems in der Statusleiste (nur Anzeige, rein lokal; Standard aus)
    pub takt_anzeige: bool,
    /// Startbild beim Programmstart (kurz, mit Klick oder Taste zu schließen)
    #[serde(default = "wahr")] pub splash: bool,
    /// Kürzel-Diagnose: zeigt im Protokoll je Tastenereignis und Menüauslösung Taste, Umschalttasten und gefundenen Befehl (nur lokal, keine Übertragung)
    #[serde(default)] pub kuerzel_diagnose: bool,
    /// Prettyprint: eine bearbeitete Zeile beim Verlassen formatieren (Assembler, BASIC)
    #[serde(default = "wahr")] pub pretty_zeile: bool,
    /// Menüleiste im Fenster (Windows und Linux; unter macOS gibt es die Menüleiste des Systems): Vorgabe an
    #[serde(default = "wahr")] pub menueleiste: bool }
fn wahr() -> bool { true }
impl Default for Einstellungen { fn default() -> Self { Einstellungen { schema: Schema::System, zoom: 1.0, editor_schrift: 13.0, umbruch: false, seitenleiste: true, schublade: true, panel: false, panel_reiter: 0, debug_hoehe: 0.38, studio_einzeltasten: true, tastaturlayout: Tastaturlayout::Auto, tasten_auf_klaviatur: true, takt_anzeige: false, splash: true, kuerzel_diagnose: false, pretty_zeile: true, menueleiste: true } } }

#[cfg(test)] thread_local! { static TEST_KONF: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) }; }
/// Nur für Tests: Konfigurationsordner dieses Test-Threads
#[cfg(test)] pub fn test_konfig(d: impl Into<PathBuf>) { TEST_KONF.with(|t| *t.borrow_mut() = Some(d.into())) }

/// Ordner der Oberflächen-Einstellungen: Windows `%APPDATA%\VintageCoder-neu` (ohne `APPDATA` unter dem persönlichen Ordner), macOS `~/Library/Application Support/VintageCoder-neu`,
/// sonst `~/.config/vintagecoder-neu`. Der persönliche Ordner kommt aus `HOME`, unter Windows sonst `USERPROFILE` (`vc_core::userdirs::home_dir`): `HOME` allein fehlt dort meist.
pub(crate) fn ordner_aus(home: Option<PathBuf>, appdata: Option<PathBuf>, windows: bool, macos: bool) -> Option<PathBuf> {
    if windows { if let Some(a) = appdata { return Some(a.join("VintageCoder-neu")) } }
    let h = home?;
    Some(if macos { h.join("Library/Application Support/VintageCoder-neu") } else if windows { h.join("AppData/Roaming/VintageCoder-neu") } else { h.join(".config/vintagecoder-neu") })
}

impl Einstellungen {
    pub fn begrenze(&mut self) { self.debug_hoehe = self.debug_hoehe.clamp(0.2, 0.7); self.zoom = self.zoom.clamp(0.7, 2.0); self.editor_schrift = self.editor_schrift.clamp(9.0, 28.0) }
    pub fn pfad() -> Option<PathBuf> {
        // Tests: je Test-Thread eigener Ordner (Umgebungsvariablen sind prozessweit und lassen parallele Tests einander stören); ohne Vorgabe wird nichts gespeichert
        #[cfg(test)] { return TEST_KONF.with(|t| t.borrow().clone()).map(|d| d.join("oberflaeche.json")) }
        #[cfg(not(test))] if let Ok(d) = std::env::var("VC_KONFIG") { return Some(PathBuf::from(d).join("oberflaeche.json")) }
        ordner_aus(vc_core::userdirs::home_dir(), std::env::var_os("APPDATA").filter(|a| !a.is_empty()).map(PathBuf::from), cfg!(windows), cfg!(target_os = "macos")).map(|d| d.join("oberflaeche.json"))
    }
    pub fn laden() -> Self { Self::laden_von(Self::pfad(), Self::alte_ansicht_pfad()) }
    /// `view.json` der bisherigen Oberfläche (Farbschema, Zoom, Schrift); in Tests nur im Testordner, nie die echte Datei
    fn alte_ansicht_pfad() -> Option<PathBuf> {
        #[cfg(test)] { return TEST_KONF.with(|t| t.borrow().clone()).map(|d| d.join("view.json")) }
        #[cfg(not(test))] { vc_core::userdirs::config_dir().map(|d| d.join("view.json")) }
    }
    /// Laden mit ausdrücklichen Pfaden. Gibt es `oberflaeche.json` nicht (allererster Start), wird das Aussehen aus der `view.json` der bisherigen Oberfläche
    /// übernommen (nur Lesen, die alte Datei bleibt unverändert) und gleich als `oberflaeche.json` gespeichert, damit es nur einmal geschieht.
    pub(crate) fn laden_von(ziel: Option<PathBuf>, alt: Option<PathBuf>) -> Self {
        if let Some(p) = ziel.as_ref().filter(|p| p.exists()) {
            let mut e: Einstellungen = std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default(); e.begrenze(); return e
        }
        let Some(mut e) = alt.as_deref().and_then(Self::aus_alter_ansicht) else { return Self::default() };
        e.begrenze(); if let Some(p) = ziel { e.speichern_in(&p) } e
    }
    /// Aussehen aus einer `view.json` der bisherigen Oberfläche (`scheme`: dark, light oder system; `zoom`; `font`); fehlt die Datei oder ist sie kaputt: `None`
    fn aus_alter_ansicht(p: &std::path::Path) -> Option<Einstellungen> {
        if !p.is_file() { return None }
        let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
        if !j.is_object() { return None }
        let v = vc_ide::setup::ViewPrefs::load(p);
        Some(Einstellungen { schema: match v.scheme.as_str() { "light" => Schema::Hell, "system" => Schema::System, _ => Schema::Dunkel }, zoom: v.zoom, editor_schrift: v.font, ..Default::default() })
    }
    pub fn speichern(&self) { if let Some(p) = Self::pfad() { self.speichern_in(&p) } }
    fn speichern_in(&self, p: &std::path::Path) { if let Some(d) = p.parent() { let _ = std::fs::create_dir_all(d); } let _ = std::fs::write(p, serde_json::to_string_pretty(self).unwrap_or_default()); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ordner_unter_windows_ohne_home_und_unveraendert_auf_macos_und_linux() {
        let p = |s: &str| Some(PathBuf::from(s));
        // Windows: APPDATA zählt, HOME ist nicht nötig
        assert_eq!(ordner_aus(None, p("C:\\Users\\a\\AppData\\Roaming"), true, false), p("C:\\Users\\a\\AppData\\Roaming").map(|d| d.join("VintageCoder-neu")));
        // Windows ohne APPDATA, aber mit persönlichem Ordner (USERPROFILE): trotzdem ein Pfad; ganz ohne beides: keiner
        assert_eq!(ordner_aus(p("C:\\Users\\a"), None, true, false), p("C:\\Users\\a").map(|d| d.join("AppData/Roaming/VintageCoder-neu")));
        assert_eq!(ordner_aus(None, None, true, false), None);
        // macOS und Linux wie bisher
        assert_eq!(ordner_aus(p("/Users/a"), p("/egal"), false, true), p("/Users/a/Library/Application Support/VintageCoder-neu"));
        assert_eq!(ordner_aus(p("/home/a"), None, false, false), p("/home/a/.config/vintagecoder-neu"));
        assert_eq!(ordner_aus(None, None, false, false), None);
    }
    #[test] fn aussehen_wird_beim_ersten_start_aus_der_alten_view_json_uebernommen() {
        let d = std::env::temp_dir().join(format!("vcgui-view-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap();
        let (ziel, alt) = (d.join("oberflaeche.json"), d.join("view.json"));
        // alte view.json vorhanden, oberflaeche.json fehlt: Werte übernommen, gespeichert, alte Datei unverändert
        let text = "{\"scheme\":\"light\",\"zoom\":1.5,\"font\":16}\n"; std::fs::write(&alt, text).unwrap();
        let e = Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone()));
        assert_eq!((e.schema, e.zoom, e.editor_schrift), (Schema::Hell, 1.5, 16.0)); assert!(ziel.exists(), "gleich als oberflaeche.json gespeichert"); assert_eq!(std::fs::read_to_string(&alt).unwrap(), text, "alte Datei bleibt unverändert");
        // oberflaeche.json vorhanden: keine Übernahme, auch wenn sich view.json ändert
        std::fs::write(&alt, "{\"scheme\":\"dark\",\"zoom\":1.2,\"font\":20}").unwrap();
        let e = Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone())); assert_eq!((e.schema, e.zoom, e.editor_schrift), (Schema::Hell, 1.5, 16.0));
        // system und dark, Grenzen (zoom höchstens 2,0, Schrift höchstens 28)
        let _ = std::fs::remove_file(&ziel); std::fs::write(&alt, "{\"scheme\":\"system\",\"zoom\":2.4,\"font\":40}").unwrap();
        let e = Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone())); assert_eq!((e.schema, e.zoom, e.editor_schrift), (Schema::System, 2.0, 28.0));
        let _ = std::fs::remove_file(&ziel); std::fs::write(&alt, "{\"scheme\":\"dark\"}").unwrap();
        let e = Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone())); assert_eq!((e.schema, e.zoom, e.editor_schrift), (Schema::Dunkel, 1.0, 13.0), "fehlende Felder: Vorgaben der alten Oberfläche");
        // fehlende oder kaputte view.json: Vorgaben der neuen Oberfläche, kein Absturz, nichts geschrieben
        let _ = std::fs::remove_file(&ziel);
        assert_eq!(Einstellungen::laden_von(Some(ziel.clone()), Some(d.join("gibt-es-nicht.json"))).schema, Einstellungen::default().schema); assert!(!ziel.exists());
        std::fs::write(&alt, "{kaputt").unwrap(); assert_eq!(Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone())).zoom, 1.0); assert!(!ziel.exists());
        std::fs::write(&alt, "[1,2]").unwrap(); assert_eq!(Einstellungen::laden_von(Some(ziel.clone()), Some(alt.clone())).zoom, 1.0); assert!(!ziel.exists());
        assert_eq!(Einstellungen::laden_von(None, None).zoom, 1.0);
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn rundlauf_und_begrenzung() {
        let d = std::env::temp_dir().join(format!("vcgui-konf-{}", std::process::id())); test_konfig(&d);
        let mut e = Einstellungen::default(); e.schema = Schema::Hell; e.zoom = 5.0; e.begrenze(); assert_eq!(e.zoom, 2.0); e.speichern();
        let g = Einstellungen::laden(); assert_eq!(g.schema, Schema::Hell); assert_eq!(g.zoom, 2.0); let _ = std::fs::remove_dir_all(&d);
    }
    #[test] fn panel_ist_vorgabe_eingeklappt_und_reiter_wird_gemerkt() { let e = Einstellungen::default(); assert!(!e.panel && e.panel_reiter == 0); let j: Einstellungen = serde_json::from_str("{\"panel_reiter\":2}").unwrap(); assert_eq!(j.panel_reiter, 2); assert!(!j.panel) }
    #[test] fn fehlende_felder_bekommen_vorgaben() { let e: Einstellungen = serde_json::from_str("{\"schema\":\"Dunkel\"}").unwrap(); assert_eq!(e.schema, Schema::Dunkel); assert!(e.seitenleiste); assert_eq!(e.zoom, 1.0) }
}
