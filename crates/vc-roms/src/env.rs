// SPDX-License-Identifier: GPL-3.0-or-later
//! Orte, an denen ROM-Dateien liegen können: Suchstufen, Benutzerordner, portabler Ordner neben dem Programm, fremde Fundorte.
//!
//! # Portabler Modus
//! Neben dem Programm darf ein Ordner `roms/` liegen (gleiche Unterordner wie im Benutzerordner). Er ist **aktiv**, wenn
//! 1. der Ordner `roms/` neben dem Programm existiert, oder
//! 2. neben dem Programm eine Datei `portable` (auch `portable.txt`, `vintagecoder.portable`) liegt (der Ordner wird dann bei Bedarf angelegt).
//!
//! „Neben dem Programm“ heißt je nach Plattform:
//! - Windows, Linux: der Ordner der ausführbaren Datei. Als AppImage (`APPIMAGE`) der Ordner der `.AppImage`-Datei, denn das Abbild selbst ist schreibgeschützt.
//! - macOS: der Ordner, der die `.app` enthält (`Programme/VintageCoder.app` → `Programme/roms`). **Nie** im Bundle selbst (`Contents/…`):
//!   ein Schreibzugriff auf ein signiertes Bundle macht die Signatur ungültig (Gatekeeper: „beschädigt“), und notarisierte Apps dürfen ihr Bundle nicht ändern.
//!   Auch `Contents/Resources` ist deshalb ausgeschlossen, selbst wenn es schreibbar wäre.
//!   Läuft die App „App-Translocation“-isoliert (frisch aus dem Internet geladen, nicht verschoben), liegt sie in einem zufälligen schreibgeschützten Pfad;
//!   dort ist der portable Modus nicht verfügbar (Hinweis in der Diagnose: App einmal in „Programme“ verschieben).
//!
//! Schreiben (Import, Drag-and-drop) geht in den portablen Ordner, wenn er aktiv und beschreibbar ist, sonst in den Benutzerordner.
//! Gelesen wird immer in beiden (portabel zuerst).
use std::path::{Path, PathBuf};

/// Suchstufen, in der Reihenfolge ihres Vorrangs (kleiner = zuerst)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// ausdrücklich eingetragene Einzeldatei (Einstellungen, Altlast `Atari800.os_rom`, `Xemu.rom_dir`)
    Explicit,
    /// `<Projekt>/roms/`
    Project,
    /// `roms/` neben dem Programm
    Portable,
    /// `<config_dir>/roms/` (oder `VC_ROMS`)
    User,
    /// alte VintageCoder-Ablagen (`~/.config/vintagecoder/mega65/MEGA65.ROM`)
    Legacy,
    /// ROM-Ordner anderer Programme (nur auf ausdrücklichen Wunsch)
    Foreign,
    /// mitgelieferte freie Ersatz-ROMs (`data/roms-open`)
    Bundled,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Stage::Explicit => "Einstellung", Stage::Project => "Projekt", Stage::Portable => "Programmordner (portabel)", Stage::User => "Benutzerordner",
            Stage::Legacy => "Alte Ablage", Stage::Foreign => "Anderes Programm", Stage::Bundled => "Mitgeliefert (Ersatz)",
        }
    }
}

/// Wie ein Ort aufgebaut ist
#[derive(Debug, Clone, PartialEq)]
pub enum Layout {
    /// `<pfad>/<system>/…` mit den Ordnernamen der Systeme (`c64`, `atari8`, …)
    Tree,
    /// `<pfad>/<Ordner>/…`, Ordnername → System (VICE: `C64` → `c64`)
    Mapped(Vec<(String, String)>),
    /// der ganze Ordner gehört einem System
    Dir(String),
    /// eine einzelne Datei für eine Rolle
    File { system: String, role: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub stage: Stage,
    /// Anzeigename (z. B. „VICE 3.10“)
    pub label: String,
    pub path: PathBuf,
    pub layout: Layout,
}

/// Zustand des portablen Modus
#[derive(Debug, Clone, PartialEq)]
pub enum Portable {
    /// kein Ordner, keine Markierung
    Off { base: Option<PathBuf> },
    Active { dir: PathBuf, by_marker: bool },
    /// nicht verfügbar, mit Begründung (z. B. macOS-Translocation)
    Unavailable(String),
}

const MARKERS: &[&str] = &["portable", "portable.txt", "vintagecoder.portable"];

/// Ordner, neben dem der portable Ordner `roms/` läge. `None`, wenn die Plattformregeln keinen erlauben (Bundle-Inneres, Translocation).
pub fn portable_base(exe: &Path, appimage: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(a) = appimage.filter(|a| a.is_absolute()) { if let Some(p) = a.parent() { return Ok(p.to_path_buf()); } }
    let comps: Vec<String> = exe.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    // macOS-Bundle: …/<Name>.app/Contents/MacOS/<programm>
    if let Some(i) = comps.iter().position(|c| c.ends_with(".app")) {
        if comps.iter().any(|c| c == "AppTranslocation") {
            return Err("App-Translocation: die App läuft aus einem schreibgeschützten Zufallspfad; zum Aktivieren in den Ordner „Programme“ verschieben und neu starten".into());
        }
        let mut base = PathBuf::new();
        for c in exe.components().take(i) { base.push(c.as_os_str()); }
        return if base.as_os_str().is_empty() { Err("Bundle ohne übergeordneten Ordner".into()) } else { Ok(base) };
    }
    exe.parent().map(Path::to_path_buf).ok_or_else(|| "kein Programmordner".to_string())
}

pub fn detect_portable(exe: Option<&Path>, appimage: Option<&Path>) -> Portable {
    let Some(exe) = exe else { return Portable::Off { base: None } };
    let base = match portable_base(exe, appimage) { Ok(b) => b, Err(e) => return Portable::Unavailable(e) };
    let dir = base.join("roms");
    if dir.is_dir() { return Portable::Active { dir, by_marker: false }; }
    if MARKERS.iter().any(|m| base.join(m).is_file()) { return Portable::Active { dir, by_marker: true }; }
    Portable::Off { base: Some(base) }
}

/// Liegt der Pfad im Inneren eines `.app`-Bundles? Dorthin wird nie geschrieben.
pub fn inside_app_bundle(p: &Path) -> bool {
    let comps: Vec<_> = p.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    comps.iter().position(|c| c.ends_with(".app")).is_some_and(|i| i + 1 < comps.len())
}

/// Kann in diesen Ordner geschrieben werden (er existiert, oder sein nächster vorhandener Elternordner ist beschreibbar)? Probiert eine Datei an.
pub fn is_writable(dir: &Path) -> bool {
    if inside_app_bundle(dir) { return false; }
    let mut d = dir;
    while !d.exists() { match d.parent() { Some(p) => d = p, None => return false } }
    if !d.is_dir() { return false; }
    let probe = d.join(format!(".vc-write-test-{}", std::process::id()));
    match std::fs::File::create(&probe) { Ok(_) => { let _ = std::fs::remove_file(&probe); true } Err(_) => false }
}

/// Alles, was die Suche über die Umgebung wissen muss. Wird in Tests von Hand gebaut, im Programm mit [`RomEnv::from_process`].
#[derive(Debug, Clone, Default)]
pub struct RomEnv {
    pub project_dir: Option<PathBuf>,
    /// Ersetzt Benutzer- und portablen Ordner (Tests, feste Ablage)
    pub vc_roms: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub portable: Option<Portable>,
    pub data_dir: Option<PathBuf>,
    /// ROM-Ordner anderer Programme mitbenutzen (Standard: aus)
    pub use_foreign: bool,
    /// ausdrücklich eingetragene Einzeldateien: (System, Rolle, Pfad)
    pub explicit: Vec<(String, String, PathBuf)>,
    /// weitere ROM-Ordner anderer Programme, z. B. aus dem eingestellten VICE-Programm (siehe [`vice_places_from_exe`])
    pub extra_foreign: Vec<Place>,
}

impl RomEnv {
    pub fn from_process() -> RomEnv {
        let exe = std::env::current_exe().ok();
        let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
        RomEnv {
            vc_roms: std::env::var_os("VC_ROMS").filter(|v| !v.is_empty()).map(PathBuf::from),
            config_dir: vc_core::userdirs::config_dir(),
            portable: Some(detect_portable(exe.as_deref(), appimage.as_deref())),
            data_dir: std::env::var_os("VC_DATA").map(PathBuf::from).filter(|p| p.is_dir()),
            ..Default::default()
        }
    }

    /// Der Benutzerordner (oder `VC_ROMS`)
    pub fn user_dir(&self) -> Option<PathBuf> { self.vc_roms.clone().or_else(|| self.config_dir.as_ref().map(|c| c.join("roms"))) }

    /// Portabler Ordner, falls aktiv (mit `VC_ROMS` nie)
    pub fn portable_dir(&self) -> Option<PathBuf> {
        if self.vc_roms.is_some() { return None; }
        match &self.portable { Some(Portable::Active { dir, .. }) => Some(dir.clone()), _ => None }
    }

    /// Cache für aus ROMs herausgeschnittene Zeichensätze (`<config_dir>/cache/roms`); außerhalb jedes Projekts
    pub fn cache_dir(&self) -> Option<PathBuf> { self.config_dir.as_ref().map(|c| c.join("cache").join("roms")) }

    /// Wohin neue ROM-Dateien kopiert werden: portabler Ordner (aktiv und beschreibbar), sonst Benutzerordner
    pub fn install_dir(&self) -> Option<PathBuf> {
        match self.portable_dir() { Some(p) if is_writable(&p) => Some(p), _ => self.user_dir() }
    }

    /// Die Suchorte in Rangfolge. Nur die Pfade; ob es sie gibt, prüft der Scan.
    pub fn places(&self) -> Vec<Place> {
        let mut v = vec![];
        for (s, r, p) in &self.explicit { v.push(Place { stage: Stage::Explicit, label: format!("Einstellung {s}/{r}"), path: p.clone(), layout: Layout::File { system: s.clone(), role: r.clone() } }); }
        if let Some(p) = &self.project_dir { v.push(Place { stage: Stage::Project, label: "Projekt".into(), path: p.join("roms"), layout: Layout::Tree }); }
        if let Some(p) = self.portable_dir() { v.push(Place { stage: Stage::Portable, label: Stage::Portable.label().into(), path: p, layout: Layout::Tree }); }
        if let Some(p) = self.user_dir() {
            let label = if self.vc_roms.is_some() { "VC_ROMS".to_string() } else { Stage::User.label().to_string() };
            v.push(Place { stage: Stage::User, label, path: p, layout: Layout::Tree });
        }
        if self.vc_roms.is_none() {
            if let Some(c) = &self.config_dir { v.push(Place { stage: Stage::Legacy, label: "Alte Ablage (mega65)".into(), path: c.join("mega65"), layout: Layout::Dir("mega65".into()) }); }
            // Windows: Xemu legt `MEGA65.ROM` selbst unter %APPDATA%\xemu-lgb\mega65 ab; so wie bisher immer mitgesucht (frühere Fassungen fanden sie dort ohne Einstellung)
            #[cfg(windows)]
            if let Some(a) = std::env::var_os("APPDATA") { v.push(Place { stage: Stage::Legacy, label: "Xemu (Programmdaten)".into(), path: PathBuf::from(a).join("xemu-lgb").join("mega65"), layout: Layout::Dir("mega65".into()) }); }
        }
        if self.use_foreign { v.extend(self.extra_foreign.iter().cloned()); v.extend(foreign_places()); }
        if let Some(d) = &self.data_dir { v.push(Place { stage: Stage::Bundled, label: Stage::Bundled.label().into(), path: d.join("roms-open"), layout: Layout::Tree }); }
        v
    }
}

/// VICE-Datenordner zu einem VICE-Programm: Windows liegen die ROM-Ordner (`C64\`, `C128\` …) im Installationsordner neben der `.exe`,
/// sonst unter `../share/vice` (Linux, Homebrew) bzw. `../Resources/share/vice` und `../../Resources/share/vice` (macOS-App). Nur Ordner mit Unterordner `C64`.
pub fn vice_places_from_exe(exe: &Path) -> Vec<Place> {
    let Some(dir) = exe.parent() else { return vec![] };
    let mut v: Vec<Place> = vec![];
    for c in [dir.to_path_buf(), dir.join("../share/vice"), dir.join("../Resources/share/vice"), dir.join("../../Resources/share/vice")] {
        if c.join("C64").is_dir() && !v.iter().any(|p| p.path.canonicalize().ok() == c.canonicalize().ok()) {
            v.push(Place { stage: Stage::Foreign, label: "VICE (eingestellt)".into(), path: c, layout: vice_map() });
        }
    }
    v
}

fn vice_map() -> Layout {
    let m = [("C64", "c64"), ("C128", "c128"), ("VIC20", "vic20"), ("PLUS4", "plus4"), ("DRIVES", "c64")];
    Layout::Mapped(m.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect())
}

/// Bekannte ROM-Ordner anderer Programme (nur wenn es sie gibt). Nur lesend benutzt; nichts wird kopiert, außer der Nutzer importiert.
pub fn foreign_places() -> Vec<Place> {
    let home = vc_core::userdirs::home_dir();
    let mut v = vec![];
    let mut vice_roots: Vec<(String, PathBuf)> = vec![];
    // macOS: /Applications/vice*/VICE.app/Contents/Resources/share/vice, /Applications/VICE.app/…, ~/Applications
    #[cfg(target_os = "macos")]
    {
        let mut apps = vec![PathBuf::from("/Applications")];
        if let Some(h) = &home { apps.push(h.join("Applications")); }
        for a in apps {
            vice_roots.push(("VICE".into(), a.join("VICE.app/Contents/Resources/share/vice")));
            if let Ok(rd) = std::fs::read_dir(&a) {
                let mut names: Vec<_> = rd.flatten().filter(|e| e.file_name().to_string_lossy().to_lowercase().starts_with("vice")).map(|e| e.path()).collect();
                names.sort();
                for n in names { vice_roots.push((n.file_name().unwrap().to_string_lossy().into_owned(), n.join("VICE.app/Contents/Resources/share/vice"))); }
            }
        }
        vice_roots.push(("VICE (Homebrew)".into(), PathBuf::from("/opt/homebrew/share/vice")));
        vice_roots.push(("VICE (Homebrew)".into(), PathBuf::from("/usr/local/share/vice")));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for p in ["/usr/share/vice", "/usr/lib/vice", "/usr/local/share/vice", "/usr/local/lib/vice"] { vice_roots.push(("VICE".into(), PathBuf::from(p))); }
        if let Some(h) = &home { vice_roots.push(("VICE".into(), h.join(".local/share/vice"))); }
    }
    #[cfg(windows)]
    {
        for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            let Some(pf) = std::env::var_os(var).map(PathBuf::from) else { continue };
            if let Ok(rd) = std::fs::read_dir(&pf) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    if n.contains("vice") { vice_roots.push((e.file_name().to_string_lossy().into_owned(), e.path())); }
                }
            }
        }
    }
    for (label, p) in vice_roots { if p.join("C64").is_dir() { v.push(Place { stage: Stage::Foreign, label, path: p, layout: vice_map() }); } }
    // Altirra (Windows): Sammlung `roms\` neben der .exe (Programme\…Altirra…\roms); Altirra braucht sie nicht, atari800 kann sie nutzen
    #[cfg(windows)]
    for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        let Some(pf) = std::env::var_os(var).map(PathBuf::from) else { continue };
        if let Ok(rd) = std::fs::read_dir(&pf) {
            for e in rd.flatten() {
                if e.file_name().to_string_lossy().to_lowercase().contains("altirra") { v.push(Place { stage: Stage::Foreign, label: e.file_name().to_string_lossy().into_owned(), path: e.path().join("roms"), layout: Layout::Dir("atari8".into()) }); }
            }
        }
    }
    // Xemu legt MEGA65.ROM unter ~/.local/share/xemu-lgb/mega65 ab (Linux/macOS; Windows nicht geprüft)
    #[cfg(unix)]
    if let Some(h) = &home { v.push(Place { stage: Stage::Foreign, label: "Xemu".into(), path: h.join(".local/share/xemu-lgb/mega65"), layout: Layout::Dir("mega65".into()) }); }
    // atari800: Pfade stehen in ~/.atari800.cfg
    if let Some(h) = &home {
        if let Ok(cfg) = std::fs::read_to_string(h.join(".atari800.cfg")) {
            for (s, r, p) in parse_atari800_cfg(&cfg) { v.push(Place { stage: Stage::Foreign, label: "atari800".into(), path: p, layout: Layout::File { system: s, role: r } }); }
        }
    }
    v.retain(|p| p.path.exists());
    v
}

/// ROM-Pfade aus einer `atari800.cfg`: `OS/XL/XE_ROM=…`, `OS/B_ROM=…`, `OS/A_ROM=…`, `BASIC_ROM=…`
pub fn parse_atari800_cfg(text: &str) -> Vec<(String, String, PathBuf)> {
    text.lines().filter_map(|l| {
        let (k, v) = l.split_once('=')?;
        let role = match k.trim().to_ascii_uppercase().as_str() { "OS/XL/XE_ROM" => "os-xl", "OS/B_ROM" => "os-b", "OS/A_ROM" => "os-a", "BASIC_ROM" => "basic", _ => return None };
        let v = v.trim();
        if v.is_empty() { return None; }
        Some(("atari8".to_string(), role.to_string(), PathBuf::from(v)))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_neben_exe_und_neben_der_app() {
        assert_eq!(portable_base(Path::new("/opt/vc/vintagecoder"), None).unwrap(), PathBuf::from("/opt/vc"));
        assert_eq!(portable_base(Path::new("/Applications/VintageCoder.app/Contents/MacOS/vintagecoder"), None).unwrap(), PathBuf::from("/Applications"));
        // Windows: absoluter Pfad mit Laufwerk
        let (img, dir) = if cfg!(windows) { ("C:/home/u/Apps/VC.AppImage", "C:/home/u/Apps") } else { ("/home/u/Apps/VC.AppImage", "/home/u/Apps") };
        assert_eq!(portable_base(Path::new("/x/y"), Some(Path::new(img))).unwrap(), PathBuf::from(dir));
        assert!(portable_base(Path::new("/private/var/folders/ab/T/AppTranslocation/UUID/d/VintageCoder.app/Contents/MacOS/vc"), None).is_err());
    }

    #[test]
    fn nie_im_bundle_schreiben() {
        assert!(inside_app_bundle(Path::new("/Applications/VintageCoder.app/Contents/Resources/roms")));
        assert!(!inside_app_bundle(Path::new("/Applications/roms")));
        assert!(!is_writable(Path::new("/Applications/VintageCoder.app/Contents/Resources/roms")));
    }

    #[test]
    fn portabler_modus_per_ordner_oder_marke() {
        let t = std::env::temp_dir().join(format!("vc-roms-portable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        std::fs::create_dir_all(&t).unwrap();
        let exe = t.join("vintagecoder");
        assert_eq!(detect_portable(Some(&exe), None), Portable::Off { base: Some(t.clone()) });
        std::fs::write(t.join("portable"), "").unwrap();
        assert_eq!(detect_portable(Some(&exe), None), Portable::Active { dir: t.join("roms"), by_marker: true });
        std::fs::create_dir(t.join("roms")).unwrap();
        assert_eq!(detect_portable(Some(&exe), None), Portable::Active { dir: t.join("roms"), by_marker: false });
        assert!(is_writable(&t.join("roms")));
        assert!(is_writable(&t.join("noch/nicht/da")));
        let _ = std::fs::remove_dir_all(&t);
    }

    #[test]
    fn suchorte_in_rangfolge_und_vc_roms_ersetzt() {
        let env = RomEnv {
            project_dir: Some("/p".into()), config_dir: Some("/c".into()), data_dir: Some("/d".into()),
            portable: Some(Portable::Active { dir: "/exe/roms".into(), by_marker: false }),
            explicit: vec![("atari8".into(), "os-xl".into(), "/x/os.rom".into())],
            ..Default::default()
        };
        let st: Vec<Stage> = env.places().iter().map(|p| p.stage).collect();
        // Windows kennt zusätzlich die Xemu-Ablage unter %APPDATA% (zweite Stufe "Legacy"), sofern APPDATA gesetzt ist
        let mut want = vec![Stage::Explicit, Stage::Project, Stage::Portable, Stage::User, Stage::Legacy, Stage::Bundled];
        #[cfg(windows)]
        if std::env::var_os("APPDATA").is_some() { want.insert(5, Stage::Legacy); }
        assert_eq!(st, want);
        let env2 = RomEnv { vc_roms: Some("/t".into()), ..env };
        let p = env2.places();
        assert!(p.iter().all(|x| x.stage != Stage::Portable && x.stage != Stage::Legacy));
        assert_eq!(p.iter().find(|x| x.stage == Stage::User).unwrap().path, PathBuf::from("/t"));
    }

    #[test]
    fn vice_ordner_neben_der_exe_und_unter_share() {
        let t = std::env::temp_dir().join(format!("vc-roms-vice-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        std::fs::create_dir_all(t.join("win/C64")).unwrap();
        std::fs::create_dir_all(t.join("unix/bin")).unwrap();
        std::fs::create_dir_all(t.join("unix/share/vice/C64")).unwrap();
        assert_eq!(vice_places_from_exe(&t.join("win/x64sc.exe")).len(), 1);
        let u = vice_places_from_exe(&t.join("unix/bin/x64sc"));
        assert_eq!(u.len(), 1);
        assert!(u[0].path.ends_with("share/vice"));
        assert!(vice_places_from_exe(&t.join("nix/x64sc")).is_empty());
        let _ = std::fs::remove_dir_all(&t);
    }

    #[test]
    fn atari800_cfg() {
        let r = parse_atari800_cfg("OS/XL/XE_ROM=/a/xl.rom\nBASIC_ROM = /a/b.rom\nOS/B_ROM=\nANDERES=1\n");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].1, "os-xl");
        assert_eq!(r[1].2, PathBuf::from("/a/b.rom"));
    }
}
