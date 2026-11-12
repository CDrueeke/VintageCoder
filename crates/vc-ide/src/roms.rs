// SPDX-License-Identifier: GPL-3.0-or-later
//! ROM-Anbindung der Emulatoren: aus dem ROM-Verzeichnis (`vc_roms`) entstehen die Kommandozeilenargumente je Emulator.
//! Eine Funktion ([`plan`]) für alle Aufrufer (Start, Debugger, Probestart); testbar ohne Emulator.
use crate::targets::ResolvedEmu;
use std::path::{Path, PathBuf};
use vc_roms::{RomEnv, RomStore, Stage, Status};

/// ROM-Dienst für den Start: Prozessumgebung, Einstellung „ROMs anderer Programme mitbenutzen“, Projektordner und ausdrückliche Einträge
/// (Altlast `Atari800.os_rom`, `Xemu.rom_dir`, `VC_XEMU_SD`; sie stehen in der Suche ganz oben)
pub fn store(project_root: Option<&Path>, emu: Option<&ResolvedEmu>) -> RomStore { RomStore::new(env(project_root, emu)) }

/// Umgebung des ROM-Dienstes (Suchorte und Schalter), siehe [`store`]
pub fn env(project_root: Option<&Path>, emu: Option<&ResolvedEmu>) -> RomEnv {
    let mut env = RomEnv::from_process();
    env.use_foreign = crate::settings::get().roms.use_foreign;
    env.project_dir = project_root.map(Path::to_path_buf);
    if env.data_dir.is_none() {
        env.data_dir = std::env::current_exe().ok().and_then(|e| crate::find_data_dir(&e)).or_else(|| std::env::current_dir().ok().and_then(|d| crate::find_data_dir(&d)));
    }
    if let Some(e) = emu { env.explicit.extend(explicit_of(e)); }
    // VICE-Datenordner aus dem eingestellten bzw. gefundenen VICE-Programm (nur genutzt, wenn „ROMs anderer Programme mitbenutzen“ an ist)
    if env.use_foreign {
        for sys in ["c64", "c128", "vic20", "plus4"] {
            let v = crate::settings::effective().vice_for(sys);
            let exe = v.path.clone().or_else(|| crate::launch::detect(&v.kind));
            if let Some(exe) = exe { for p in vc_roms::env::vice_places_from_exe(Path::new(&exe)) { if !env.extra_foreign.iter().any(|x| x.path == p.path) { env.extra_foreign.push(p); } } }
        }
    }
    // Xemu: Ordner aus den Einstellungen (alt) und VC_XEMU_SD
    if let Some(d) = crate::settings::xemu_rom_dir() { env.explicit.extend(rom_in_dir(&d)); }
    if let Ok(d) = std::env::var("VC_XEMU_SD") { env.explicit.extend(rom_in_dir(&d)); }
    env
}

/// ROM-Dienst für Zeichensätze (`vc_media::romref`) auf Projekt und Einstellungen abstimmen: beim Programmstart, beim Öffnen eines Projekts
/// und nach einer Änderung von „ROMs anderer Programme mitbenutzen“
pub fn configure_media(project_root: Option<&Path>) {
    *MEDIA_ROOT.lock().unwrap_or_else(|e| e.into_inner()) = project_root.map(Path::to_path_buf);
    vc_media::romref::configure_async(env(project_root, None));
}

/// Wie [`configure_media`] mit dem zuletzt gemeldeten Projekt (nach geänderten Einstellungen)
pub fn reconfigure_media() { let root = MEDIA_ROOT.lock().unwrap_or_else(|e| e.into_inner()).clone(); vc_media::romref::configure_async(env(root.as_deref(), None)); }

static MEDIA_ROOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn rom_in_dir(dir: &str) -> Option<(String, String, PathBuf)> { vc_emu::Mega65::rom_in(dir).map(|p| ("mega65".to_string(), "rom".to_string(), PathBuf::from(p))) }

/// Ausdrücklich eingetragene ROMs eines Emulators: atari800 → Datei des XL/XE-Betriebssystems, Xemu → Ordner mit `MEGA65.ROM`
pub fn explicit_of(e: &ResolvedEmu) -> Vec<(String, String, PathBuf)> {
    let Some(rom) = e.rom.as_deref().filter(|r| !r.is_empty()) else { return vec![] };
    match e.kind.as_str() {
        "atari800" => vec![("atari8".into(), "os-xl".into(), PathBuf::from(rom))],
        "xemu" => rom_in_dir(rom).into_iter().collect(),
        _ => vec![],
    }
}

/// Ergebnis: Argumente für den Emulator und Zeilen für das Startprotokoll
#[derive(Debug, Default, Clone)]
pub struct Plan {
    /// Argumente (nur ROM-Optionen)
    pub args: Vec<String>,
    /// Xemu: ROM-Datei
    pub xemu_rom: Option<PathBuf>,
    /// Zeilen für die Konsole: gewählte ROMs mit Herkunft und Status
    pub log: Vec<String>,
    /// Warnungen (Unbekannte Revision, Ersatz, unbrauchbare Datei)
    pub warnings: Vec<String>,
    /// Harter Fehler (nur Xemu: ohne ROM startet er nicht)
    pub error: Option<String>,
}

impl Plan {
    /// Zeilen für das Startprotokoll: Auswahl und Warnungen
    pub fn lines(&self) -> Vec<String> { self.log.iter().cloned().chain(self.warnings.iter().map(|w| format!("Warnung: {w}"))).collect() }
}

/// Rollen und Schalter der VICE-Programme (System, [(Rolle, Schalter)])
fn vice_roles(kind: &str) -> (&'static str, &'static [(&'static str, &'static str)]) {
    match kind {
        "x128" => ("c128", &[("kernal", "-kernal"), ("basiclo", "-basiclo"), ("basichi", "-basichi"), ("chargen", "-chargen")]),
        "xvic" => ("vic20", &[("kernal", "-kernal"), ("basic", "-basic"), ("chargen", "-chargen")]),
        "xplus4" => ("plus4", &[("kernal", "-kernal"), ("basic", "-basic"), ("functionlo", "-functionlo"), ("functionhi", "-functionhi")]),
        _ => ("c64", &[("kernal", "-kernal"), ("basic", "-basic"), ("chargen", "-chargen")]),
    }
}

/// Meldung „ROM fehlt“ mit den gesuchten Orten und dem Weg zur Abhilfe
pub fn missing_message(store: &RomStore, system: &str, role: &str) -> String {
    let rd = vc_roms::systems::role(system, role);
    let size = rd.map(|r| r.sizes.iter().map(|s| format!("{s} Byte")).collect::<Vec<_>>().join(" oder ")).unwrap_or_default();
    let file = rd.map(|r| r.file).unwrap_or("");
    let target = store.env.install_dir().map(|d| d.join(system).display().to_string()).unwrap_or_else(|| format!("roms/{system}"));
    let places: Vec<String> = store.searched().iter().map(|s| format!("{} ({})", s.path.display(), s.stage.label())).collect();
    format!("ROM {system}/{role} nicht gefunden (erwartet {size}). Datei nach {target}{}{file} legen (Einstellungen → ROMs oder `vcc roms import`). Gesuchte Orte: {}",
        std::path::MAIN_SEPARATOR, places.join("; "))
}

fn describe(r: &vc_roms::Resolved) -> String { r.describe() }

/// ROM-Argumente für `kind` (`x64sc`, `x64`, `x128`, `xvic`, `xplus4`, `atari800`, `altirra`, `xemu`). `existing` sind schon vorhandene
/// Argumente (Katalog, freies Feld): Schalter, die der Nutzer selbst gesetzt hat, werden nicht überschrieben.
pub fn plan(kind: &str, store: &RomStore, existing: &[String]) -> Plan {
    let mut p = Plan::default();
    let has = |flag: &str| existing.iter().any(|a| a == flag);
    // Eigene Auswahl des Nutzers (Stufen Einstellung bis Fremdprogramm) wird übergeben; Mitgeliefertes nur, wo der Emulator sonst nicht startet
    let own = |s: Option<Stage>| matches!(s, Some(Stage::Explicit | Stage::Project | Stage::Portable | Stage::User | Stage::Legacy | Stage::Foreign));
    match kind {
        "x64sc" | "x64" | "x128" | "xvic" | "xplus4" => {
            let (sys, roles) = vice_roles(kind);
            for (role, flag) in roles {
                let r = store.resolve(sys, role);
                if has(flag) { p.log.push(format!("{sys}/{role}: {flag} aus den Emulatorparametern")); continue; }
                match r.status {
                    Status::Missing => if vc_roms::systems::role(sys, role).is_some_and(|d| d.required) { p.log.push(format!("{sys}/{role}: nicht im ROM-Verzeichnis – VICE verwendet seine eigenen ROMs")) },
                    Status::Unusable => p.warnings.push(format!("{}: Datei hat die falsche Größe (erwartet {}) und wird nicht benutzt", describe(&r), r.expected.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" oder "))),
                    _ if !own(r.stage) => p.log.push(format!("{} – nicht übergeben", describe(&r))),
                    s => {
                        p.args.push((*flag).into());
                        p.args.push(r.path.as_ref().unwrap().to_string_lossy().into_owned());
                        p.log.push(describe(&r));
                        match s { Status::UnknownRevision => p.warnings.push(format!("{sys}/{role}: unbekannte Revision, Symbole und Offsets können abweichen")), Status::Replacement => p.warnings.push(format!("{sys}/{role}: freie Ersatz-ROM, kein Original")), _ => {} }
                    }
                }
            }
        }
        "atari800" => {
            for (role, flag) in [("os-xl", "-xlxe_rom"), ("os-b", "-osb_rom"), ("os-a", "-osa_rom"), ("basic", "-basic_rom")] {
                let r = store.resolve("atari8", role);
                if has(flag) { p.log.push(format!("atari8/{role}: {flag} aus den Emulatorparametern")); continue; }
                match r.status {
                    Status::Missing => if role == "os-xl" { p.log.push("atari8/os-xl: nicht im ROM-Verzeichnis – atari800 verwendet seine Einstellung bzw. das eingebaute Altirra-OS (Ersatz)".into()) },
                    Status::Unusable => p.warnings.push(format!("{}: Datei hat die falsche Größe und wird nicht benutzt", describe(&r))),
                    _ if !own(r.stage) => p.log.push(format!("{} – nicht übergeben", describe(&r))),
                    s => {
                        p.args.push(flag.into());
                        p.args.push(r.path.as_ref().unwrap().to_string_lossy().into_owned());
                        p.log.push(describe(&r));
                        match s { Status::UnknownRevision => p.warnings.push(format!("atari8/{role}: unbekannte Revision")), Status::Replacement => p.warnings.push(format!("atari8/{role}: freie Ersatz-ROM, kein Original")), _ => {} }
                    }
                }
            }
        }
        "xemu" => {
            let r = store.resolve("mega65", "rom");
            if r.status.usable() && r.path.is_some() {
                let path = r.path.clone().unwrap();
                p.args = vec!["-rom".into(), path.to_string_lossy().into_owned()];
                p.xemu_rom = Some(path);
                p.log.push(describe(&r));
                match r.status { Status::UnknownRevision => p.warnings.push("mega65/rom: unbekannte Revision, Symbole und Offsets können abweichen".into()), Status::Replacement => p.warnings.push("mega65/rom: freie Ersatz-ROM (Open-ROMs), kein Original".into()), _ => {} }
            } else {
                p.error = Some(if r.status == Status::Unusable { format!("MEGA65-ROM {} hat {} Byte, erwartet 131072", r.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(), r.size.unwrap_or(0)) } else { missing_message(store, "mega65", "rom") });
            }
        }
        "altirra" => p.log.push("Altirra verwaltet seine Firmware selbst (Einstellungen von Altirra); VintageCoder übergibt keine ROMs".into()),
        _ => {}
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tmp(PathBuf);
    impl Tmp {
        fn new(n: &str) -> Tmp { let p = std::env::temp_dir().join(format!("vc-ide-roms-{n}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&p); std::fs::create_dir_all(&p).unwrap(); Tmp(p) }
        fn put(&self, rel: &str, n: usize) { let p = self.0.join(rel); std::fs::create_dir_all(p.parent().unwrap()).unwrap(); std::fs::write(p, vec![0xEAu8; n]).unwrap(); }
    }
    impl Drop for Tmp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    fn store_of(t: &Tmp) -> RomStore { RomStore::new(RomEnv { vc_roms: Some(t.0.clone()), ..Default::default() }) }

    #[test]
    fn vice_c64_bekommt_die_schalter_aus_dem_rom_ordner() {
        let t = Tmp::new("vice");
        t.put("c64/kernal.bin", 8192); t.put("c64/basic.bin", 8192); t.put("c64/chargen.bin", 4096);
        let p = plan("x64sc", &store_of(&t), &[]);
        let flags: Vec<&str> = p.args.iter().step_by(2).map(String::as_str).collect();
        assert_eq!(flags, ["-kernal", "-basic", "-chargen"]);
        assert!(p.args[1].ends_with("kernal.bin"));
        assert_eq!(p.warnings.len(), 3);   // unbekannte Revisionen (synthetische Daten)
    }

    /// Fehler aus dem Windows-Test: VICE darf nichts aus dem Ordner von Xemu bekommen (dort lag `CHARROM.M65` = C64-Zeichensatz)
    #[test]
    fn vice_bekommt_nichts_aus_dem_mega65_ordner() {
        use vc_roms::known::{KnownRom, KnownTable, Use};
        use vc_roms::hash;
        let t = Tmp::new("xemu-legacy");
        let data = vec![5u8; 4096];
        t.put("mega65/CHARROM.M65", 4096);   // Inhalt 0xEA, nur Größe zählt für die nächste Prüfung
        std::fs::write(t.0.join("mega65/CHARROM.M65"), &data).unwrap();
        let k = KnownRom { id: "c64.chargen.t".into(), label: "t".into(), size: 4096, crc32: hash::crc32_hex(&data), sha1: hash::sha1_hex(&data), uses: vec![Use { system: "c64".into(), role: "chargen".into() }], default: true, free: false, note: String::new(), sets: Default::default() };
        let known = KnownTable { format: 1, roms: vec![k] };
        // der Xemu-Ordner als alte Ablage `<config>/mega65` (auf Windows zusätzlich `%APPDATA%\\xemu-lgb\\mega65`)
        let env = RomEnv { config_dir: Some(t.0.clone()), ..Default::default() };
        let store = RomStore::with_known(env, known);
        let r = store.resolve("c64", "chargen");
        assert!(!r.is_usable(), "{r:?}");
        let p = plan("x64sc", &store, &[]);
        assert!(!p.args.iter().any(|a| a == "-chargen"), "{:?}", p.args);
    }

    #[test]
    fn eigene_schalter_des_nutzers_bleiben() {
        let t = Tmp::new("vice-own");
        t.put("c64/kernal.bin", 8192);
        let p = plan("x64sc", &store_of(&t), &["-kernal".to_string(), "/meins.bin".to_string()]);
        assert!(p.args.is_empty());
    }

    #[test]
    fn vice_ohne_roms_laesst_vice_seine_eigenen_nehmen() {
        let t = Tmp::new("vice-leer");
        let p = plan("x128", &store_of(&t), &[]);
        assert!(p.args.is_empty());
        assert!(p.error.is_none());
        assert!(p.log.iter().any(|l| l.contains("VICE verwendet seine eigenen ROMs")));
    }

    #[test]
    fn x128_und_xplus4_haben_ihre_eigenen_rollen() {
        let t = Tmp::new("sys");
        t.put("c128/basiclo.bin", 16384); t.put("c128/basichi.bin", 16384);
        t.put("plus4/functionlo.bin", 16384);
        let s = store_of(&t);
        let a: Vec<String> = plan("x128", &s, &[]).args.into_iter().step_by(2).collect();
        assert_eq!(a, ["-basiclo", "-basichi"]);
        let a: Vec<String> = plan("xplus4", &s, &[]).args.into_iter().step_by(2).collect();
        assert_eq!(a, ["-functionlo"]);
    }

    #[test]
    fn atari800_os_rollen() {
        let t = Tmp::new("atari");
        t.put("atari8/os-xl.rom", 16384); t.put("atari8/os-b.rom", 10240);
        let p = plan("atari800", &store_of(&t), &[]);
        assert_eq!(p.args.iter().step_by(2).map(String::as_str).collect::<Vec<_>>(), ["-xlxe_rom", "-osb_rom"]);
        let leer = plan("atari800", &store_of(&Tmp::new("atari-leer")), &[]);
        assert!(leer.args.is_empty() && leer.error.is_none());
        assert!(leer.log[0].contains("Altirra"));
    }

    #[test]
    fn xemu_braucht_die_rom_und_nennt_gesuchte_orte() {
        let t = Tmp::new("xemu");
        let fehlt = plan("xemu", &store_of(&t), &[]);
        let e = fehlt.error.unwrap();
        assert!(e.contains("mega65/rom") && e.contains("131072") && e.contains(&t.0.display().to_string()), "{e}");
        t.put("mega65/mega65.rom", 131072);
        let p = plan("xemu", &store_of(&t), &[]);
        assert!(p.error.is_none());
        assert_eq!(p.args[0], "-rom");
        assert!(p.xemu_rom.unwrap().ends_with("mega65.rom"));
        // falsche Größe
        let t2 = Tmp::new("xemu2");
        t2.put("mega65/mega65.rom", 1000);
        assert!(plan("xemu", &store_of(&t2), &[]).error.unwrap().contains("1000"));
    }

    #[test]
    fn altlast_eintraege_stehen_ganz_oben() {
        let t = Tmp::new("alt");
        t.put("mega65/mega65.rom", 131072);
        let alt = Tmp::new("alt-old");
        alt.put("MEGA65.ROM", 131072);
        let emu = ResolvedEmu { kind: "xemu".into(), path: None, args: vec![], rom: Some(alt.0.to_string_lossy().into_owned()), opts: Default::default(), free: String::new() };
        let mut env = RomEnv { vc_roms: Some(t.0.clone()), ..Default::default() };
        env.explicit.extend(explicit_of(&emu));
        let p = plan("xemu", &RomStore::new(env), &[]);
        assert!(p.xemu_rom.unwrap().starts_with(&alt.0));
    }
}
