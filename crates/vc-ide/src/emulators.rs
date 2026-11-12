// SPDX-License-Identifier: GPL-3.0-or-later
//! Katalog der Emulatoren (`data/emulators/catalog.json`, eingebettet): Homepage, Lizenz, Rolle in VintageCoder und die portablen Pakete, die
//! `vcc emulator install` und die Einstellungen unterhalb von VintageCoder installieren können. Die Programme selbst werden nie mitgeliefert.
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use vc_emu::install::{self, Format};

const CATALOG: &str = include_str!("../../../data/emulators/catalog.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Pkg {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub url: String,
    #[serde(default)] pub sha256: String,
    pub format: String,
    #[serde(default)] pub size: u64,
    /// Programmnamen dieses Pakets je Sorte, wenn sie vom Standard des Emulators abweichen
    #[serde(default)] pub exe: BTreeMap<String, String>,
}

/// Fähigkeiten einer Anbindung (Konzept „Release-Auswahl“ 6.2): die Oberfläche entscheidet daraus, nicht aus Programmnamen
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Caps {
    /// Programme starten (F5)
    #[serde(default)] pub run: bool,
    /// Debugger: `full`, `limited` oder leer (keiner)
    #[serde(default)] pub debug: String,
    /// Tests über die Anbindung (`vcc test --backend`)
    #[serde(default)] pub test: bool,
    #[serde(default)] pub image_on_halt: bool,
    #[serde(default)] pub live_image: bool,
}

impl Caps { pub fn can_debug(&self) -> bool { !self.debug.is_empty() } }

/// Erkennung des Programms: Programmnamen je Sorte (`*` = alle Sorten; fehlt der Eintrag, gilt der Programmname der Sorte aus `exe`), zusätzliche
/// Suchordner je Betriebssystem (`windows`, `macos`, `linux`; `*` = alle) und eine Umgebungsvariable, die Vorrang hat
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Detect {
    #[serde(default)] pub names: BTreeMap<String, Vec<String>>,
    #[serde(default)] pub dirs: BTreeMap<String, Vec<String>>,
    #[serde(default)] pub env: String,
}

/// Feld für ein ROM in der Einstellungskarte (atari800: Datei, Xemu: Ordner)
#[derive(Debug, Clone, Deserialize)]
pub struct RomField { pub label: String, #[serde(default)] pub hint: String, #[serde(default)] pub folder: bool }

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    /// `emulator` (Programm, Standard) oder `device` (echte Maschine ohne Programm: Ultimate, MEGA65)
    #[serde(default)] pub kind: String,
    /// Kennung in `FEATURES.md` (`EMU-VICE`); die Anbindung heißt in `release.json` wie `id`
    #[serde(default)] pub feature: String,
    /// Geräte: Maschinentyp der Einstellungen (`ultimate64`, `mega65`)
    #[serde(default)] pub machine: String,
    /// Emulator-Sorten aus den Einstellungen (`x64sc`, `atari800` …); leer bei Fujisan (kein Starter, nur Debugger)
    #[serde(default)] pub kinds: Vec<String>,
    /// Anzeigename je Sorte (Auswahl in den Einstellungen); fehlt er, gilt der Name des Eintrags
    #[serde(default)] pub labels: BTreeMap<String, String>,
    /// Systeme je Sorte, wenn sie nicht alle Systeme des Eintrags bedienen (VICE: `x128` nur `c128`)
    #[serde(default)] pub systems_of: BTreeMap<String, Vec<String>>,
    /// Reihenfolge der Einträge in der Auswahl eines Systems (kleiner = früher, Standard 10); der erste ist der Standard-Emulator neuer Systeme
    #[serde(default)] pub rank: Option<u32>,
    #[serde(default)] pub exe: BTreeMap<String, String>,
    /// Erkennung (Suchordner, Namen, Umgebungsvariable)
    #[serde(default)] pub detect: Detect,
    /// Parameterkatalog je Sorte (Programmkennung in `data/args`)
    #[serde(default)] pub args: BTreeMap<String, String>,
    /// Adapter in `vc_emu::backend_for` (leer = nur Starter)
    #[serde(default)] pub backend: String,
    #[serde(default)] pub caps: Caps,
    #[serde(default)] pub rom: Option<RomField>,
    #[serde(default)] pub homepage: String,
    #[serde(default)] pub download_page: String,
    #[serde(default)] pub license: String,
    pub systems: Vec<String>,
    pub role: String,
    pub summary: String,
    #[serde(default)] pub notes: Vec<String>,
    #[serde(default)] pub packages: Vec<Pkg>,
    /// Hinweis je Betriebssystem (`windows`, `macos`, `linux`), wenn es dort kein automatisches Paket gibt
    #[serde(default)] pub manual: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Catalog { emulators: Vec<Entry> }

fn all_entries() -> &'static [Entry] {
    static C: OnceLock<Vec<Entry>> = OnceLock::new();
    C.get_or_init(|| serde_json::from_str::<Catalog>(CATALOG).expect("data/emulators/catalog.json").emulators)
}

impl Entry { pub fn is_device(&self) -> bool { self.kind == "device" } }

/// Startet den Debugger-/Test-Adapter der Sorte `kind` (die Kennung `backend` steht im Katalog, `vc_emu::backend_for` kennt den Adapter)
pub fn launch_backend(kind: &str, l: &vc_emu::Launch) -> Result<Box<dyn vc_emu::Backend>, String> {
    require(kind)?;
    let e = for_kind(kind).ok_or_else(|| format!("Emulator „{kind}“ ist nicht im Katalog"))?;
    vc_emu::backend_for(&e.backend, l).map_err(|m| if e.backend.is_empty() { format!("{}: {m}", e.name) } else { m })
}

/// Die Emulatoren (Programme) des Katalogs
pub fn catalog() -> &'static [Entry] {
    static C: OnceLock<Vec<Entry>> = OnceLock::new();
    C.get_or_init(|| all_entries().iter().filter(|e| !e.is_device()).cloned().collect())
}

/// Die echten Maschinen des Katalogs (Ultimate, MEGA65): Anbindungen ohne Programm
pub fn devices() -> &'static [Entry] {
    static C: OnceLock<Vec<Entry>> = OnceLock::new();
    C.get_or_init(|| all_entries().iter().filter(|e| e.is_device()).cloned().collect())
}

pub fn find(id: &str) -> Option<&'static Entry> { catalog().iter().find(|e| e.id == id) }

/// Eintrag einer Maschinenart der Einstellungen (`ultimate64`, `mega65`)
pub fn device_for(machine_kind: &str) -> Option<&'static Entry> { devices().iter().find(|e| e.machine == machine_kind) }

/// Eintrag zu einer Emulator-Sorte der Einstellungen (`x64sc`, `atari800`, `xemu` …) oder einer Kennung (`fujisan`)
pub fn for_kind(kind: &str) -> Option<&'static Entry> { catalog().iter().find(|e| e.id == kind || e.kinds.iter().any(|k| k == kind) || e.args.contains_key(kind)) }

/// Kennung der Anbindung (Registry/`release.json`, z. B. `vice`) zu einer Emulator-Sorte (`x64sc`, `xpet`, `atari800` …) oder Kennung; `None` = unbekannt
pub fn link_id(kind: &str) -> Option<&'static str> { for_kind(kind).map(|e| e.id.as_str()) }

/// Ist die Anbindung zu dieser Emulator-Sorte im Release gewählt? Unbekannte Sorten gelten als gewählt (kein Grund, sie zu sperren).
pub fn enabled(kind: &str) -> bool { enabled_in(&crate::features::features(), kind) }
/// Wie [`enabled`] für eine bestimmte Auswahl
pub fn enabled_in(f: &crate::features::Features, kind: &str) -> bool { link_id(kind).map_or(true, |id| f.link(id)) }

/// Ist die Anbindung einer echten Maschine (`ultimate64`, `mega65`) im Release gewählt?
pub fn device_enabled(machine_kind: &str) -> bool { device_enabled_in(&crate::features::features(), machine_kind) }
/// Wie [`device_enabled`] für eine bestimmte Auswahl
pub fn device_enabled_in(f: &crate::features::Features, machine_kind: &str) -> bool { f.link(match machine_kind { "ultimate64" => "ultimate", other => other }) }

/// Meldung für eine abgewählte Anbindung (`vcc`, Start, Debugger)
pub fn not_in_release(name: &str) -> String { format!("{name}: nicht im Paket enthalten (diese Anbindung ist in diesem Release nicht gewählt)") }

/// Prüft [`enabled`] und liefert sonst die Meldung „nicht im Paket enthalten“
pub fn require(kind: &str) -> Result<(), String> {
    if enabled(kind) { Ok(()) } else { Err(not_in_release(for_kind(kind).map_or(kind, |e| e.name.as_str()))) }
}

/// Wie [`require`] für eine echte Maschine
pub fn require_device(machine_kind: &str) -> Result<(), String> {
    if device_enabled(machine_kind) { Ok(()) } else { Err(not_in_release(match machine_kind { "ultimate64" => "Ultimate", "mega65" => "MEGA65 (Gerät)", k => k })) }
}

/// Die Katalogeinträge, die in diesem Release gewählt sind (ohne `release.json`: alle)
pub fn available() -> Vec<&'static Entry> { available_in(&crate::features::features()) }
/// Wie [`available`] für eine bestimmte Auswahl
pub fn available_in(f: &crate::features::Features) -> Vec<&'static Entry> { catalog().iter().filter(|e| f.link(&e.id)).collect() }

/// Hat mindestens eine gewählte Anbindung (Emulator oder Gerät) einen Debugger (`caps.debug`)?
pub fn any_debug_in(f: &crate::features::Features) -> bool { all_entries().iter().any(|e| f.link(&e.id) && e.caps.can_debug()) }

/// Hat mindestens eine gewählte Anbindung ein Echtzeit-Bild (`caps.live_image`, Reiter „Live-Bild“)?
pub fn any_live_in(f: &crate::features::Features) -> bool { all_entries().iter().any(|e| f.link(&e.id) && e.caps.live_image) }

/// Bedient mindestens ein gewählter Eintrag dieses System (`c64`, `atari8` …)?
pub fn any_for_system(system: &str) -> bool { available().iter().any(|e| e.systems.iter().any(|s| s == system)) }

/// Sorten, die dieser Eintrag für `system` bedient (VICE: je System genau eine bzw. zwei), in der Reihenfolge des Eintrags
fn kinds_for_system<'a>(e: &'a Entry, system: &str) -> Vec<&'a str> {
    e.kinds.iter().map(String::as_str).filter(|k| e.systems_of.get(*k).map_or(e.systems.iter().any(|s| s == system), |v| v.iter().any(|s| s == system))).collect()
}

/// Anzeigename einer Sorte (aus dem Katalog), sonst der Name des Eintrags
pub fn kind_label(kind: &str) -> Option<&'static str> { let e = for_kind(kind)?; Some(e.labels.get(kind).map_or(e.name.as_str(), String::as_str)) }

/// Die Emulator-Sorten, die für `system` in Frage kommen, in Reihenfolge der Auswahl: nach `rank`, dann Katalogreihenfolge (aus dem Katalog,
/// nicht nach Auswahl des Releases gefiltert: die Datei behält alles; die Oberfläche filtert mit [`enabled`])
pub fn kinds_of(system: &str) -> Vec<(&'static str, &'static str)> {
    let mut es: Vec<(u32, usize, &'static Entry)> = catalog().iter().enumerate().map(|(i, e)| (e.rank.unwrap_or(10), i, e)).collect();
    es.sort_by_key(|(r, i, _)| (*r, *i));
    es.into_iter().flat_map(|(_, _, e)| kinds_for_system(e, system).into_iter().map(move |k| (k, e.labels.get(k).map_or(e.name.as_str(), String::as_str)))).collect()
}

/// Maschinenarten (`ultimate64`, `mega65`), die für `system` anbindbar sind (aus dem Katalog)
pub fn hardware_of(system: &str) -> Vec<&'static str> { devices().iter().filter(|d| d.systems.iter().any(|s| s == system)).map(|d| d.machine.as_str()).collect() }

/// Suchordner für diese Sorte laut Katalog (`*` und der Eintrag des aktuellen Betriebssystems)
pub fn search_dirs(kind: &str) -> Vec<String> {
    let Some(e) = for_kind(kind) else { return vec![] };
    ["*", current_os()].iter().filter_map(|k| e.detect.dirs.get(*k)).flatten().cloned().collect()
}

/// Generische Erkennung aus dem Katalog: erst die Umgebungsvariable des Eintrags, dann die Programmnamen in PATH und Suchordnern. `which` sucht
/// einen Namen (mit den zusätzlichen Ordnern); `None` für unbekannte Sorten.
pub fn detect_by(kind: &str, which: fn(&str, &[&str]) -> Option<String>) -> Option<String> {
    let e = for_kind(kind)?;
    if !e.detect.env.is_empty() { if let Ok(v) = std::env::var(&e.detect.env) { return Some(v); } }
    let names: Vec<String> = e.detect.names.get(kind).or_else(|| e.detect.names.get("*")).cloned().or_else(|| e.exe.get(kind).map(|n| vec![n.clone()])).unwrap_or_else(|| vec![if e.exe.contains_key(&e.id) && e.kinds.is_empty() { e.exe[&e.id].clone() } else { kind.to_string() }]);
    let dirs = search_dirs(kind);
    let dirs: Vec<&str> = dirs.iter().map(String::as_str).collect();
    names.iter().find_map(|n| which(n, &dirs))
}

pub fn current_os() -> &'static str { if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" } }
pub fn current_arch() -> &'static str { std::env::consts::ARCH }

impl Entry {
    /// Paket für dieses System (Betriebssystem und Architektur), mit Prüfsumme
    pub fn package(&self) -> Option<&Pkg> { self.package_for(current_os(), current_arch()) }
    pub fn package_for(&self, os: &str, arch: &str) -> Option<&Pkg> { self.packages_for(os, arch).into_iter().next() }
    /// Alle Pakete für dieses System (neueste Version zuerst, so wie im Katalog aufgeführt)
    pub fn packages_here(&self) -> Vec<&Pkg> { self.packages_for(current_os(), current_arch()) }
    pub fn packages_for(&self, os: &str, arch: &str) -> Vec<&Pkg> { self.packages.iter().filter(|p| p.os == os && p.arch == arch && p.sha256.len() == 64).collect() }
    /// Paket einer bestimmten Version für dieses System
    pub fn package_version(&self, version: &str) -> Option<&Pkg> { self.packages_here().into_iter().find(|p| p.version == version) }
    /// Hinweis für dieses System, wenn es kein automatisches Paket gibt
    pub fn manual_hint(&self) -> Option<&str> { self.manual.get(current_os()).map(String::as_str).filter(|s| !s.is_empty()) }
    /// Programmname der Sorte (ohne `.exe`)
    pub fn exe_name(&self, kind: &str, pkg: Option<&Pkg>) -> Option<String> {
        pkg.and_then(|p| p.exe.get(kind)).or_else(|| self.exe.get(kind)).cloned()
    }
    /// Kennung der Hauptsorte (für Anzeige und Suche): die erste Sorte, bei Fujisan die Kennung
    pub fn main_kind(&self) -> &str { self.kinds.first().map(String::as_str).unwrap_or(&self.id) }

    fn install_package(&self, p: &Pkg) -> Result<install::Package, String> {
        Ok(install::Package { id: self.id.clone(), version: p.version.clone(), url: p.url.clone(), sha256: p.sha256.to_ascii_lowercase(), format: Format::parse(&p.format).ok_or_else(|| format!("Paketform „{}“ unbekannt", p.format))?, size: p.size })
    }
}

/// Eine Quelle für ein Emulator-Programm: eine portabel installierte Version oder das, was das System bietet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Anzeigetext, z. B. „portabel 3.9“ oder „System“
    pub label: String,
    pub path: String,
    /// Version bei portablen Installationen (Ordnername)
    pub version: Option<String>,
}

/// Programmnamen der Sorte in einer installierten Version (die des Pakets dieser Version, sonst der Standard des Emulators)
fn names_for(e: &Entry, key: &str, version: &str) -> Vec<String> {
    let mut v: Vec<String> = e.packages.iter().filter(|p| p.version == version && p.os == current_os()).filter_map(|p| e.exe_name(key, Some(p))).collect();
    if let Some(n) = e.exe_name(key, None) { v.push(n); }
    v.dedup();
    v
}

fn key_of<'a>(e: &'a Entry, kind: &'a str) -> &'a str { if e.exe.contains_key(kind) { kind } else { e.main_kind() } }

/// Alle Quellen für diese Sorte: erst die portabel installierten Versionen (neueste zuerst), dann der Fund im System (PATH, Programmordner)
pub fn sources(kind: &str) -> Vec<Source> {
    let Some(e) = for_kind(kind) else { return vec![] };
    if !enabled(kind) { return vec![]; }
    let key = key_of(e, kind);
    let mut out: Vec<Source> = install::versions(&e.id).into_iter().filter_map(|(ver, dir)| {
        let exe = names_for(e, key, &ver).iter().find_map(|n| install::find_in(&dir, n, 4))?;
        Some(Source { label: format!("portabel {ver}"), path: exe.to_string_lossy().into_owned(), version: Some(ver) })
    }).collect();
    if let Some(p) = crate::launch::detect_system(kind) { out.push(Source { label: "System".into(), path: p, version: None }); }
    out
}

/// Pfad des von VintageCoder installierten Programms dieser Sorte (neueste Version), falls vorhanden
pub fn installed_exe(kind: &str) -> Option<String> { sources(kind).into_iter().find(|s| s.version.is_some()).map(|s| s.path) }

/// Zerlegt `vice@3.9` in Emulator und Version
pub fn parse_spec(s: &str) -> (String, Option<String>) {
    match s.split_once('@') { Some((a, v)) if !v.is_empty() => (a.to_string(), Some(v.to_string())), _ => (s.trim_end_matches('@').to_string(), None) }
}

/// Wählt für diesen Prozess eine bestimmte, schon installierte Version (`vice@3.9`); ohne Version bleibt die automatische Wahl
pub fn select(spec: &str) -> Result<(), String> {
    let (id, ver) = parse_spec(spec);
    let e = for_kind(&id).ok_or_else(|| format!("Emulator „{id}“ ist nicht im Katalog"))?;
    require(&e.id)?;
    let Some(v) = ver else { return Ok(()) };
    let key = e.main_kind().to_string();
    let found = install::versions(&e.id).into_iter().any(|(ver, dir)| ver == v && names_for(e, &key, &ver).iter().any(|n| install::find_in(&dir, n, 4).is_some()));
    if !found { return Err(format!("{} {v} ist nicht installiert (vcc emulator install {}@{v})", e.name, e.id)); }
    install::prefer(&e.id, &v);
    Ok(())
}

/// Ordner der installierten Versionen eines Emulators
pub fn install_dir(id: &str) -> Option<PathBuf> { install::dir_of(id) }

/// Lädt und installiert den Emulator `id` (feste Version, Prüfsumme geprüft) unterhalb von VintageCoder. Gibt den Pfad des Programms zurück.
/// Nur auf ausdrücklichen Aufruf; `say` erhält Fortschrittsmeldungen.
pub fn install(spec: &str, say: &mut dyn FnMut(&str)) -> Result<String, String> {
    let (id, version) = parse_spec(spec);
    let e = for_kind(&id).ok_or_else(|| format!("Emulator „{id}“ ist nicht im Katalog"))?;
    require(&e.id)?;
    let no_pkg = || match e.manual_hint() { Some(h) => format!("Für dieses System gibt es kein automatisches Paket. {h}"), None => "Für dieses System gibt es kein automatisches Paket.".to_string() };
    let p = match &version {
        Some(v) => e.package_version(v).ok_or_else(|| { let have: Vec<&str> = e.packages_here().iter().map(|p| p.version.as_str()).collect(); if have.is_empty() { no_pkg() } else { format!("{} {v} gibt es nicht im Katalog (verfügbar: {})", e.name, have.join(", ")) } })?,
        None => e.package().ok_or_else(no_pkg)?,
    };
    let key = e.main_kind().to_string();
    let name = e.exe_name(&key, Some(p)).ok_or("Programmname fehlt im Katalog")?;
    let pkg = e.install_package(p)?;
    let have = |dir: &std::path::Path| install::find_in(dir, &name, 4).is_some();
    let dir = install::install(&pkg, &have, say)?;
    let exe = install::find_in(&dir, &name, 4).ok_or_else(|| format!("Nach dem Auspacken fehlt {name} in {}", dir.display()))?;
    Ok(exe.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn katalog_ist_vollstaendig() {
        let c = catalog();
        assert!(c.len() >= 5);
        for e in c {
            assert!(e.homepage.starts_with("https://") && e.download_page.starts_with("https://"), "{}: Adressen", e.id);
            assert!(!e.summary.is_empty() && !e.license.is_empty() && !e.role.is_empty() && !e.systems.is_empty(), "{}", e.id);
            for k in &e.kinds { assert!(e.exe.contains_key(k), "{}: Programmname der Sorte {k} fehlt", e.id); }
            if e.kinds.is_empty() { assert!(e.exe.contains_key(&e.id), "{}", e.id); }
            for p in &e.packages {
                assert!(p.url.starts_with("https://"), "{}: {}", e.id, p.url);
                assert_eq!(p.sha256.len(), 64, "{}: Prüfsumme von {}", e.id, p.url);
                assert!(p.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()), "{}", e.id);
                assert!(Format::parse(&p.format).is_some(), "{}", e.id);
                assert!(["windows", "linux", "macos"].contains(&p.os.as_str()) && ["x86_64", "aarch64"].contains(&p.arch.as_str()), "{}", e.id);
            }
        }
        // jede Sorte der Einstellungen hat einen Eintrag
        for s in crate::targets::SYSTEMS { assert!(!s.kinds().is_empty(), "System {} ohne Emulator-Sorte", s.id); for k in s.kinds() { assert!(for_kind(k.id).is_some(), "Sorte {} fehlt im Katalog", k.id); } }
        // jede Sorte des Katalogs bedient mindestens ein System
        for e in c { for k in &e.kinds { assert!(crate::targets::SYSTEMS.iter().any(|s| s.kinds().iter().any(|x| x.id == k)), "Sorte {k} bedient kein System"); } }
    }

    #[test]
    fn spezifikation_und_mehrere_versionen() {
        assert_eq!(parse_spec("vice@3.9"), ("vice".to_string(), Some("3.9".to_string())));
        assert_eq!(parse_spec("vice"), ("vice".to_string(), None));
        let v = find("vice").unwrap();
        let ps = v.packages_for("windows", "x86_64");
        assert_eq!(ps.iter().map(|p| p.version.as_str()).collect::<Vec<_>>(), ["3.10", "3.9"], "neueste zuerst (Vorgabe)");
        assert_eq!(v.package_for("windows", "x86_64").unwrap().version, "3.10");
        // Katalogreihenfolge: je System ist das erste Paket die höchste Version
        for e in catalog() {
            for (os, arch) in [("windows", "x86_64"), ("windows", "aarch64"), ("linux", "x86_64"), ("linux", "aarch64")] {
                let vs: Vec<&str> = e.packages_for(os, arch).iter().map(|p| p.version.as_str()).collect();
                let mut sorted = vs.clone();
                sorted.sort_by(|a, b| vc_emu::install::version_cmp(b, a));
                assert_eq!(vs, sorted, "{} {os} {arch}: Versionen nicht neueste zuerst", e.id);
            }
        }
        assert!(select("vice@0.1").unwrap_err().contains("nicht installiert"));
        assert!(select("nichtda@1").is_err());
        assert!(select("vice").is_ok());
    }

    #[test]
    fn suche_nach_sorte_und_paket() {
        assert_eq!(for_kind("x64sc").unwrap().id, "vice");
        assert_eq!(for_kind("fujisan").unwrap().id, "fujisan");
        let w = find("atari800").unwrap().package_for("windows", "x86_64").unwrap();
        assert_eq!(w.version, "7.2.1");
        let l = find("atari800").unwrap().package_for("linux", "x86_64").unwrap();
        assert_eq!(find("atari800").unwrap().exe_name("atari800", Some(l)).unwrap(), "Atari800-7.2.1-x86_64.AppImage");
        assert!(find("altirra").unwrap().package_for("linux", "x86_64").is_none());
        assert_eq!(find("altirra").unwrap().exe_name("altirra", find("altirra").unwrap().package_for("windows", "aarch64")).unwrap(), "AltirraARM64");
    }

    #[test]
    fn anbindungen_nach_auswahl() {
        use crate::features::Features;
        let f = Features::from_json(r#"{"format":1,"links":["vice"]}"#);
        assert_eq!(available_in(&f).iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["vice"]);
        for k in ["x64sc", "x64", "x128", "xplus4", "xvic", "xpet", "xcbm2"] { assert!(enabled_in(&f, k), "{k}"); }
        for k in ["atari800", "altirra", "fujisan", "xemu"] { assert!(!enabled_in(&f, k), "{k}"); }
        assert!(enabled_in(&f, "vcc-asm") && enabled_in(&f, "unbekannt"), "Assembler-Katalog und Unbekanntes bleiben");
        assert!(!device_enabled_in(&f, "ultimate64") && !device_enabled_in(&f, "mega65"));
        assert_eq!(available_in(&Features::all()).len(), catalog().len());
        // ohne Auswahl (Tests laufen ohne release.json): alles an, Meldungen leer
        assert!(require("atari800").is_ok() && require_device("ultimate64").is_ok());
        assert!(not_in_release("Xemu").contains("nicht im Paket enthalten"));
    }

    /// Der feste `match` von vorher (Erkennung): Vergleich mit der Erkennung aus dem Katalog, für alle bisherigen Sorten
    fn legacy_detect(kind: &str, which: fn(&str, &[&str]) -> Option<String>) -> Option<String> {
        let vice = ["/Applications/vice-arm64-gtk3-3.10/bin", "/Applications/vice-arm64-gtk3-3.9/bin", "/opt/homebrew/bin", "/usr/local/bin"];
        match kind {
            "x64sc" | "x64" | "x128" | "xplus4" | "xvic" => which(kind, &vice),
            "atari800" => which("atari800", &["/opt/homebrew/bin", "/usr/local/bin"]),
            "altirra" => which("Altirra64", &[]).or_else(|| which("Altirra", &[])),
            "fujisan" => which("Fujisan", &["/opt/fujisan", "/opt/fujisan/bin", "/Applications/Fujisan.app/Contents/MacOS"]).or_else(|| which("fujisan", &[])),
            "xemu" => std::env::var("VC_XEMU").ok().or_else(|| which("xmega65", &["/opt/homebrew/bin", "/usr/local/bin", "/Applications/xmega65.app/Contents/MacOS", "/Applications/Xemu"])),
            _ => None,
        }
    }

    #[test]
    fn erkennung_aus_dem_katalog_wie_der_alte_match() {
        let kinds = ["x64sc", "x64", "x128", "xplus4", "xvic", "atari800", "altirra", "fujisan", "xemu", "unbekannt"];
        for k in kinds {
            assert_eq!(detect_by(k, vc_emu::find_program), legacy_detect(k, vc_emu::find_program), "{k} (mit portablen Installationen)");
            assert_eq!(detect_by(k, vc_emu::find_program_system), legacy_detect(k, vc_emu::find_program_system), "{k} (nur System)");
        }
        // Namen und Ordner kommen aus dem Katalog
        fn probe(name: &str, dirs: &[&str]) -> Option<String> { Some(format!("{name}|{}", dirs.join(","))) }
        assert_eq!(detect_by("altirra", probe).unwrap(), "Altirra64|");
        assert_eq!(detect_by("x128", probe).unwrap(), "x128|/Applications/vice-arm64-gtk3-3.10/bin,/Applications/vice-arm64-gtk3-3.9/bin,/opt/homebrew/bin,/usr/local/bin");
        assert!(detect_by("atari800", probe).unwrap().starts_with("atari800|/opt/homebrew/bin"));
        assert_eq!(detect_by("nichtda", probe), None);
        // neu gegenüber dem festen `match`: die übrigen VICE-Sorten mit Parameterkatalog (xpet, xcbm2) werden ebenfalls gefunden
        assert!(detect_by("xpet", probe).unwrap().starts_with("xpet|"));
    }

    #[test]
    fn sorten_und_geraete_wie_die_feste_tabelle() {
        let kinds = |s: &str| kinds_of(s).iter().map(|(k, _)| *k).collect::<Vec<_>>();
        assert_eq!(kinds("c64"), ["x64sc", "x64"]);
        assert_eq!(kinds("c128"), ["x128"]);
        assert_eq!(kinds("plus4"), ["xplus4"]);
        assert_eq!(kinds("c16"), ["xplus4"]);
        assert_eq!(kinds("vic20"), ["xvic"]);
        assert_eq!(kinds("atari8"), ["altirra", "atari800"], "Altirra zuerst: Standard neuer Atari-Systeme");
        assert_eq!(kinds("mega65"), ["xemu"]);
        assert_eq!(kind_label("x64sc"), Some("VICE x64sc (taktgenau)"));
        assert_eq!(kind_label("x64"), Some("VICE x64 (schnell)"));
        assert_eq!(kind_label("altirra"), Some("Altirra"));
        assert_eq!(kind_label("xemu"), Some("Xemu (xmega65)"));
        assert_eq!(kind_label("atari800"), Some("atari800"));
        assert_eq!(hardware_of("c64"), ["ultimate64"]);
        assert_eq!(hardware_of("mega65"), ["mega65"]);
        assert!(hardware_of("atari8").is_empty() && hardware_of("c128").is_empty());
    }

    #[test]
    fn katalogfelder_sind_stimmig() {
        // Adapter-Kennungen kennt vc_emu; Debugger braucht einen Adapter; Parameterkataloge gibt es in data/args
        let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/args");
        let (cats, errs) = crate::args::load_dirs(&[data]);
        assert!(errs.is_empty(), "{errs:?}");
        for e in all_entries() {
            assert!(e.backend.is_empty() || vc_emu::BACKENDS.contains(&e.backend.as_str()) || e.is_device(), "{}: Adapter {} unbekannt", e.id, e.backend);
            if e.caps.can_debug() && !e.is_device() { assert!(!e.backend.is_empty(), "{}: caps.debug ohne backend", e.id); }
            for (k, prog) in &e.args { assert!(cats.iter().any(|c| &c.program == prog), "{}: Parameterkatalog {prog} für {k} fehlt", e.id); }
            assert!(e.feature.starts_with(if e.is_device() { "HW-" } else { "EMU-" }), "{}: feature", e.id);
            assert!(e.is_device() == !e.machine.is_empty(), "{}: machine nur bei Geräten", e.id);
        }
        // Kennungen wie in der Registry (release/registry.json)
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../release/registry.json")).expect("registry.json");
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        for e in all_entries() {
            let f = &v["features"][e.feature.as_str()];
            assert_eq!(f["kind"], "link", "{}: {} fehlt in der Registry", e.id, e.feature);
            assert_eq!(f["id"], e.id.as_str(), "{}: Kennung in der Registry", e.feature);
        }
        for (key, f) in v["features"].as_object().unwrap() { if f["kind"] == "link" { assert!(all_entries().iter().any(|e| &e.feature == key), "{key} fehlt im Emulator-Katalog"); } }
    }

    #[test]
    fn fahigkeiten_steuern_debugger_und_live() {
        use crate::features::Features;
        let only = |l: &str| Features::from_json(&format!(r#"{{"format":1,"links":["{l}"]}}"#));
        for (l, dbg, live) in [("vice", true, false), ("atari800", true, false), ("altirra", false, false), ("fujisan", true, false), ("xemu", true, false), ("ultimate", true, true), ("mega65", true, false)] {
            assert_eq!(only(l).any_debugger(), dbg, "{l} Debugger");
            assert_eq!(only(l).any_live_image(), live, "{l} Live");
        }
        assert!(Features::all().any_debugger() && Features::all().any_live_image());
        assert!(!Features::from_json(r#"{"format":1,"links":[]}"#).any_debugger());
    }
}
