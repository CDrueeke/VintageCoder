// SPDX-License-Identifier: GPL-3.0-or-later
//! Domänenmodell der Zielsysteme: je System 1..n Emulatoren (genau einer davon Standard), 0..n echte Maschinen und das
//! Standard-Startziel (Emulator oder eine Maschine). Reine Daten und Regeln – ohne Oberfläche und ohne Betriebssystemzugriffe
//! (Prozesse, Sockets, serielle Ports: siehe `platform`).
use std::collections::BTreeMap;
use std::sync::OnceLock;
use crate::settings::{expand_home, split_args, Machine, Settings};
use serde::{Deserialize, Serialize};

/// Eine Emulator-Sorte, die für ein System in Frage kommt
#[derive(Clone, Copy, Debug)]
pub struct EmuKind { pub id: &'static str, pub label: &'static str }

/// Ein unterstütztes 8-Bit-System
#[derive(Clone, Copy, Debug)]
pub struct SystemInfo {
    pub id: &'static str,
    pub name: &'static str,
    /// Gruppe in der Navigation (Hersteller bzw. Kategorie)
    pub group: &'static str,
    /// Kürzel für das Symbol
    pub badge: &'static str,
    pub cpu: &'static str,
}

impl SystemInfo {
    /// Emulator-Sorten, die für dieses System in Frage kommen: aus dem Emulator-Katalog (`systems`, `systems_of`, `rank`, `labels`)
    pub fn kinds(&self) -> &'static [EmuKind] {
        static K: OnceLock<BTreeMap<&'static str, Vec<EmuKind>>> = OnceLock::new();
        let m = K.get_or_init(|| SYSTEMS.iter().map(|s| (s.id, crate::emulators::kinds_of(s.id).into_iter().map(|(id, label)| EmuKind { id, label }).collect())).collect());
        m.get(self.id).map_or(&[], Vec::as_slice)
    }
    /// Sorten echter Maschinen, die sich für dieses System anbinden lassen (`ultimate64`, `mega65`): aus dem Katalog
    pub fn hardware(&self) -> &'static [&'static str] {
        static H: OnceLock<BTreeMap<&'static str, Vec<&'static str>>> = OnceLock::new();
        let m = H.get_or_init(|| SYSTEMS.iter().map(|s| (s.id, crate::emulators::hardware_of(s.id))).collect());
        m.get(self.id).map_or(&[], Vec::as_slice)
    }
}


/// Alle bekannten Systeme in der Reihenfolge der Navigation
pub const SYSTEMS: &[SystemInfo] = &[
    SystemInfo { id: "c64", name: "Commodore 64", group: "Commodore 8 Bit", badge: "C64", cpu: "MOS 6510 · VIC-II · SID" },
    SystemInfo { id: "c128", name: "Commodore 128", group: "Commodore 8 Bit", badge: "128", cpu: "MOS 8502 / Z80 · VDC" },
    SystemInfo { id: "plus4", name: "Commodore Plus/4", group: "Commodore 8 Bit", badge: "+4", cpu: "MOS 7501/8501 · TED" },
    SystemInfo { id: "c16", name: "Commodore 16 / 116", group: "Commodore 8 Bit", badge: "C16", cpu: "MOS 7501 · TED · 16 KB" },
    SystemInfo { id: "vic20", name: "VC-20", group: "Commodore 8 Bit", badge: "V20", cpu: "MOS 6502 · VIC" },
    SystemInfo { id: "atari8", name: "Atari 800 XL / XE", group: "Atari 8 Bit", badge: "800", cpu: "MOS 6502C · ANTIC · GTIA · POKEY" },
    SystemInfo { id: "mega65", name: "MEGA65", group: "Next-Gen 8 Bit", badge: "M65", cpu: "45GS02 · VIC-IV · 40 MHz" },
];

pub fn system(id: &str) -> Option<&'static SystemInfo> { SYSTEMS.iter().find(|s| s.id == id) }

/// Die im Release gelieferten Systeme (Target-Datei vorhanden), in der Reihenfolge der Navigation. Ohne Datenordner: alle.
pub fn installed_systems(data: Option<&std::path::Path>) -> Vec<&'static SystemInfo> {
    let f = crate::features::features();
    match data { Some(d) => SYSTEMS.iter().filter(|s| vc_target::target_available(d, s.id) && f.target(s.id)).collect(), None => SYSTEMS.iter().filter(|s| f.target(s.id)).collect() }
}

/// Anzeigename einer Emulator-Sorte
pub fn kind_label(kind: &str) -> &'static str { crate::emulators::kind_label(kind).unwrap_or("Emulator") }

/// Welche Art Hardware ist es? (Anzeige im Dialog)
pub fn hardware_label(kind: &str) -> &'static str {
    match kind { "ultimate64" => "Ultimate 64 / II+ (Netzwerk)", "mega65" => "MEGA65 (USB oder Ethernet)", _ => "Unbekannt" }
}

/// Ein Emulator für ein System
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EmulatorCfg {
    pub id: String,
    #[serde(default)] pub name: String,
    /// Sorte, siehe [`SystemInfo::kinds`]
    #[serde(default)] pub kind: String,
    /// Programm (leer = automatisch suchen); `~` wird aufgelöst
    #[serde(default)] pub path: String,
    /// Zusätzliche Startargumente, wie in der Shell geschrieben
    #[serde(default)] pub args: String,
    /// Ausgewählte Parameter aus dem Katalog (`data/args`), Kennung → Wert; `args` bleibt das freie Feld „weitere Parameter“
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")] pub opts: crate::args::Selection,
    /// atari800: Betriebssystem-ROM; Xemu: Ordner mit `MEGA65.ROM`
    #[serde(default)] pub rom: String,
    /// Standard-Emulator des Systems (höchstens einer je System)
    #[serde(default)] pub standard: bool,
}

/// Ein Zielsystem mit seinen Emulatoren und dem Standard-Startziel
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetCfg {
    pub system: String,
    #[serde(default)] pub emulators: Vec<EmulatorCfg>,
    /// Startziel bei F5: leer = Standard-Emulator, sonst die Kennung einer echten Maschine
    #[serde(default)] pub start_on: String,
}

/// Aufgelöster Emulator zum Starten: Sorte, Programm (`None` = automatisch suchen), Argumente und ROM
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedEmu {
    pub kind: String, pub path: Option<String>,
    /// freies Feld „weitere Parameter“, in Wörter zerlegt (Platzhalter noch nicht ersetzt); die fertige Liste liefert [`ResolvedEmu::launch_args`]
    pub args: Vec<String>,
    pub rom: Option<String>,
    /// Auswahl aus dem Katalog und das freie Feld im Wortlaut
    pub opts: crate::args::Selection, pub free: String,
}

impl ResolvedEmu {
    /// Wie [`ResolvedEmu::launch_args`] für den Debug-Start: ohne die Schalter, die der Debug-Start selbst für die Verbindung setzt
    pub fn debug_args(&self, ctx: &crate::args::ArgCtx) -> Result<Vec<String>, String> {
        let cat = crate::args::catalog_of_kind(&self.kind);
        Ok(crate::args::drop_debug_reserved(cat.as_deref(), self.launch_args(ctx)?))
    }
    /// Fertige Zusatzargumente für den Start: Katalogauswahl, dann das freie Feld, Platzhalter aus `ctx` ersetzt (Fehler als ein Text)
    pub fn launch_args(&self, ctx: &crate::args::ArgCtx) -> Result<Vec<String>, String> {
        let cat = crate::args::catalog_of_kind(&self.kind);
        crate::args::build(cat.as_deref(), &self.opts, &self.free, ctx).map_err(|e| format!("Emulatorparameter: {}", e.join("; ")))
    }
}

fn non_empty(s: &str) -> Option<String> { let s = s.trim(); if s.is_empty() { None } else { Some(expand_home(s)) } }

/// Überschreibung der globalen Einstellungen durch ein Projekt, für ein System (siehe [`Settings::with_overrides`])
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectOverride {
    pub system: String,
    /// Kennung eines Emulators des Systems, der für dieses Projekt Standard ist; leer = keine Überschreibung
    pub default_emulator: String,
    /// `@emulator` oder Kennung einer Maschine; leer = keine Überschreibung
    pub start_on: String,
}

impl EmulatorCfg {
    pub fn new(system: &str, kind: &str, existing: &[EmulatorCfg]) -> EmulatorCfg {
        let base = format!("{system}-{kind}");
        let id = std::iter::once(base.clone()).chain((2..).map(|n| format!("{base}-{n}"))).find(|id| !existing.iter().any(|e| &e.id == id)).unwrap_or(base);
        EmulatorCfg { id, name: kind_label(kind).into(), kind: kind.into(), standard: existing.is_empty(), ..Default::default() }
    }
    pub fn display_name(&self) -> &str { if self.name.trim().is_empty() { kind_label(&self.kind) } else { self.name.trim() } }
    pub fn resolved(&self) -> ResolvedEmu {
        ResolvedEmu { kind: self.kind.clone(), path: non_empty(&self.path), args: split_args(&self.args), rom: non_empty(&self.rom), opts: self.opts.clone(), free: self.args.clone() }
    }
}

impl TargetCfg {
    pub fn standard_emulator(&self) -> Option<&EmulatorCfg> { self.emulators.iter().find(|e| e.standard).or(self.emulators.first()) }
    /// Genau ein Standard-Emulator, solange es Emulatoren gibt
    pub fn normalize(&mut self) {
        let pos = self.emulators.iter().position(|e| e.standard).unwrap_or(0);
        for (i, e) in self.emulators.iter_mut().enumerate() { e.standard = i == pos; }
    }
    pub fn set_standard(&mut self, id: &str) { if self.emulators.iter().any(|e| e.id == id) { for e in &mut self.emulators { e.standard = e.id == id; } } }
    pub fn add_emulator(&mut self, kind: &str) -> String { let e = EmulatorCfg::new(&self.system, kind, &self.emulators); let id = e.id.clone(); self.emulators.push(e); self.normalize(); id }
    pub fn remove_emulator(&mut self, id: &str) { self.emulators.retain(|e| e.id != id); self.normalize(); }
}

impl Settings {
    pub fn target(&self, system: &str) -> Option<&TargetCfg> { self.targets.iter().find(|t| t.system == system) }
    pub fn target_mut(&mut self, system: &str) -> Option<&mut TargetCfg> { self.targets.iter_mut().find(|t| t.system == system) }

    /// Nimmt ein System auf (mit seinem Standard-Emulator); die Reihenfolge folgt [`SYSTEMS`]
    pub fn add_target(&mut self, system: &str) {
        let Some(info) = system_info(system) else { return };
        if self.target(system).is_some() { return; }
        let mut t = TargetCfg { system: info.id.into(), ..Default::default() };
        if let Some(k) = info.kinds().first() { t.add_emulator(k.id); }
        self.targets.push(t);
        self.targets.sort_by_key(|t| SYSTEMS.iter().position(|s| s.id == t.system).unwrap_or(usize::MAX));
    }
    /// Entfernt ein System; seine Maschinen bleiben erhalten, werden aber keinem System mehr angezeigt und sind beim Start weiter wählbar
    pub fn remove_target(&mut self, system: &str) { self.targets.retain(|t| t.system != system); }

    pub fn machines_for<'a>(&'a self, system: &'a str) -> impl Iterator<Item = &'a Machine> + 'a { self.machines.iter().filter(move |m| m.system() == system) }

    /// Neue Kennung für eine Maschine
    pub fn next_machine_id(&self) -> String { (1..).map(|n| format!("m{n}")).find(|id| self.machine(id).is_none()).unwrap_or_default() }

    /// Legt eine Maschine für `system` an (Typ aus [`SystemInfo::hardware`]); liefert ihre Kennung
    pub fn add_machine(&mut self, system: &str, kind: &str) -> String {
        let id = self.next_machine_id();
        let name = match kind { "mega65" => "MEGA65", _ => "Ultimate 64" };
        self.machines.push(Machine { id: id.clone(), name: name.into(), kind: kind.into(), target: system.into(), ..Default::default() });
        id
    }
    /// Entfernt eine Maschine; war sie Startziel, gilt wieder der Emulator
    pub fn remove_machine(&mut self, id: &str) {
        self.machines.retain(|m| m.id != id);
        for t in &mut self.targets { if t.start_on == id { t.start_on.clear(); } }
    }

    /// Übernimmt die älteren, nach Programmen gegliederten Felder (`emulators`) in die Zielsysteme. Danach sind die alten Felder leer.
    /// Liefert, ob etwas geändert wurde.
    pub fn migrate_legacy(&mut self) -> bool {
        let mut changed = false;
        if self.targets.is_empty() {
            for info in SYSTEMS {
                let mut t = TargetCfg { system: info.id.into(), ..Default::default() };
                for k in info.kinds() {
                    // der schnelle VICE x64 hat in den alten Einstellungen keinen Platz; er kommt nur auf Wunsch dazu
                    if k.id == "x64" { continue; }
                    let id = t.add_emulator(k.id);
                    let (path, args, rom) = self.legacy_raw(k.id);
                    if let Some(e) = t.emulators.iter_mut().find(|e| e.id == id) { e.path = path; e.args = args; e.rom = rom; }
                }
                // Standard: der früher gewählte bzw. automatisch bevorzugte Emulator
                let want = if info.id == "atari8" { self.legacy_kind("atari8") } else { info.kinds()[0].id };
                let want = if info.id == "atari8" && want == "altirra" && self.emulators.altirra.path.trim().is_empty() && crate::launch::detect("altirra").is_none() { "atari800" } else { want };
                if let Some(e) = t.emulators.iter().find(|e| e.kind == want).map(|e| e.id.clone()) { t.set_standard(&e); }
                self.targets.push(t);
            }
            self.emulators = Default::default();
            changed = true;
        }
        for m in &mut self.machines { if m.target.is_empty() { m.target = m.system().into(); changed = true; } }
        for t in &mut self.targets { t.normalize(); }
        changed
    }

    /// Roh gespeicherte Werte der alten Felder: (Pfad, Argumente, ROM)
    fn legacy_raw(&self, kind: &str) -> (String, String, String) {
        let e = &self.emulators;
        match kind {
            "x64sc" => (e.vice.x64sc.clone(), e.vice.args.clone(), String::new()),
            "x128" => (e.vice.x128.clone(), e.vice.args.clone(), String::new()),
            "xplus4" => (e.vice.xplus4.clone(), e.vice.args.clone(), String::new()),
            "xvic" => (e.vice.xvic.clone(), e.vice.args.clone(), String::new()),
            "x64" => (String::new(), e.vice.args.clone(), String::new()),
            "atari800" => (e.atari800.path.clone(), e.atari800.args.clone(), e.atari800.os_rom.clone()),
            "altirra" => (e.altirra.path.clone(), e.altirra.args.clone(), String::new()),
            "xemu" => (e.xemu.path.clone(), e.xemu.args.clone(), e.xemu.rom_dir.clone()),
            _ => Default::default(),
        }
    }
    fn legacy(&self, kind: &str) -> ResolvedEmu {
        let (p, a, r) = self.legacy_raw(kind);
        ResolvedEmu { kind: kind.into(), path: non_empty(&p), args: split_args(&a), rom: non_empty(&r), opts: Default::default(), free: a }
    }
    /// Sorte, die ohne Zielsystem-Einstellungen für `system` gilt
    fn legacy_kind(&self, system: &str) -> &'static str {
        match system { "c128" => "x128", "plus4" | "c16" => "xplus4", "vic20" => "xvic", "mega65" => "xemu",
            "atari8" => if self.emulators.altirra.default_for_atari.trim() == "atari800" { "atari800" } else { "altirra" },
            _ => "x64sc" }
    }

    /// Die Einstellungen, wie sie für ein Projekt gelten: Standard-Emulator und Startziel der genannten Systeme sind überschrieben.
    /// Die Datei `settings.json` bleibt die Quelle der Wahrheit: unbekannte Kennungen (Emulator oder Maschine gelöscht) werden ignoriert,
    /// dann gilt der globale Wert. Zugangsdaten und Pfade kommen immer aus den globalen Einstellungen.
    pub fn with_overrides(&self, ov: &[ProjectOverride]) -> Settings {
        let mut s = self.clone();
        for o in ov {
            let machines: Vec<String> = s.machines.iter().map(|m| m.id.clone()).collect();
            let Some(t) = s.target_mut(&o.system) else { continue };
            if !o.default_emulator.is_empty() { t.set_standard(&o.default_emulator); }
            if o.start_on == "@emulator" { t.start_on.clear(); }
            else if !o.start_on.is_empty() && machines.contains(&o.start_on) { t.start_on = o.start_on.clone(); }
        }
        s
    }

    /// Die Einstellungen, wie sie in diesem Release wirken: Emulatoren und Maschinen abgewählter Anbindungen fehlen. Nur eine Sicht
    /// für Start, Debugger und Auswahlen: die Datei und die Entwürfe der Einstellungsseite behalten alles (ein volles Release findet es wieder).
    pub fn released(&self) -> Settings { self.released_in(&crate::features::features()) }
    /// Wie [`Settings::released`] für eine bestimmte Auswahl
    pub fn released_in(&self, f: &crate::features::Features) -> Settings {
        if f.is_all() { return self.clone(); }
        let mut s = self.clone();
        for t in &mut s.targets {
            t.emulators.retain(|e| crate::emulators::enabled_in(f, &e.kind));
            t.normalize();
            if t.start_on != "@emulator" && !t.start_on.is_empty() && !self.machines.iter().any(|m| m.id == t.start_on && crate::emulators::device_enabled_in(f, &m.kind)) { t.start_on.clear(); }
        }
        s.machines.retain(|m| crate::emulators::device_enabled_in(f, &m.kind));
        s
    }

    /// Emulator, mit dem `system` gestartet wird: der Standard-Emulator, sonst die alten Einstellungen
    pub fn emulator_for(&self, system: &str) -> ResolvedEmu {
        match self.target(system).and_then(|t| t.standard_emulator()) {
            Some(e) => e.resolved(),
            None => self.legacy(self.legacy_kind(system)),
        }
    }
    /// Emulator einer bestimmten Sorte für `system` (z. B. atari800 für den Debugger), der Standard bevorzugt
    pub fn emulator_of_kind(&self, system: &str, kind: &str) -> ResolvedEmu {
        let t = self.target(system);
        let found = t.and_then(|t| t.emulators.iter().filter(|e| e.kind == kind).max_by_key(|e| e.standard));
        match found { Some(e) => e.resolved(), None => self.legacy(kind) }
    }

    /// VICE-Emulator für `system`: der Standard-Emulator, wenn er zu VICE gehört, sonst der VICE des Systems
    pub fn vice_for(&self, system: &str) -> ResolvedEmu {
        let e = self.emulator_for(system);
        if matches!(e.kind.as_str(), "x64sc" | "x64" | "x128" | "xplus4" | "xvic") { return e; }
        self.emulator_of_kind(system, match system { "c128" => "x128", "plus4" | "c16" => "xplus4", "vic20" => "xvic", _ => "x64sc" })
    }

    /// Echte Maschine für den Start: `run_on` ist die Wahl in der Werkzeugleiste (leer = Standard des Systems, `@emulator` = Emulator,
    /// sonst Kennung einer Maschine). `None` heißt: im Emulator starten.
    pub fn start_machine(&self, run_on: &str, system: &str) -> Option<&Machine> {
        match run_on {
            "@emulator" => None,
            "" => self.target(system).filter(|t| !t.start_on.is_empty()).and_then(|t| self.machine(&t.start_on)),
            id => self.machine(id),
        }
    }
}

fn system_info(id: &str) -> Option<&'static SystemInfo> { system(id) }

#[cfg(test)]
mod override_tests {
    use super::*;

    fn settings() -> Settings {
        let mut s = Settings::default();
        s.add_target("atari8");
        let t = s.target_mut("atari8").unwrap();
        let second = t.add_emulator("atari800");
        assert_eq!(t.standard_emulator().unwrap().kind, "altirra");
        s.add_machine("atari8", "ultimate64");
        let _ = second;
        s
    }

    #[test]
    fn ohne_ueberschreibung_gilt_global() {
        let s = settings();
        assert_eq!(s.with_overrides(&[]), s);
        let o = ProjectOverride { system: "c64".into(), default_emulator: "x".into(), start_on: "m1".into() };
        assert_eq!(s.with_overrides(&[o]), s);   // System nicht eingerichtet: nichts zu überschreiben
    }

    #[test]
    fn standard_emulator_wird_ueberschrieben() {
        let s = settings();
        let id = s.target("atari8").unwrap().emulators.iter().find(|e| e.kind == "atari800").unwrap().id.clone();
        let eff = s.with_overrides(&[ProjectOverride { system: "atari8".into(), default_emulator: id, start_on: String::new() }]);
        assert_eq!(eff.emulator_for("atari8").kind, "atari800");
        assert_eq!(s.emulator_for("atari8").kind, "altirra");   // global unverändert
        // unbekannte Kennung: globaler Standard bleibt
        let eff = s.with_overrides(&[ProjectOverride { system: "atari8".into(), default_emulator: "gibt-es-nicht".into(), start_on: String::new() }]);
        assert_eq!(eff.emulator_for("atari8").kind, "altirra");
    }

    #[test]
    fn startziel_wird_ueberschrieben() {
        let mut s = settings();
        s.target_mut("atari8").unwrap().start_on = "m1".into();
        assert_eq!(s.start_machine("", "atari8").map(|m| m.id.as_str()), Some("m1"));
        // Projekt: lieber im Emulator
        let eff = s.with_overrides(&[ProjectOverride { system: "atari8".into(), default_emulator: String::new(), start_on: "@emulator".into() }]);
        assert!(eff.start_machine("", "atari8").is_none());
        // global Emulator, Projekt: Maschine
        s.target_mut("atari8").unwrap().start_on.clear();
        let eff = s.with_overrides(&[ProjectOverride { system: "atari8".into(), default_emulator: String::new(), start_on: "m1".into() }]);
        assert_eq!(eff.start_machine("", "atari8").map(|m| m.id.as_str()), Some("m1"));
        // gelöschte Maschine: globaler Wert
        let eff = s.with_overrides(&[ProjectOverride { system: "atari8".into(), default_emulator: String::new(), start_on: "m9".into() }]);
        assert!(eff.start_machine("", "atari8").is_none());
        // die Wahl in der Werkzeugleiste (run_on) schlägt weiterhin alles
        assert!(eff.start_machine("@emulator", "atari8").is_none());
    }
}

#[cfg(test)]
mod release_tests {
    use super::*;
    use crate::features::Features;

    fn full() -> Settings {
        let mut s = Settings::default();
        s.add_target("atari8"); s.add_target("c64"); s.add_target("mega65");
        s.add_machine("c64", "ultimate64"); s.add_machine("mega65", "mega65");
        s.target_mut("c64").unwrap().start_on = "m1".into();
        s
    }

    #[test]
    fn ohne_auswahl_bleibt_alles() {
        let s = full();
        assert_eq!(s.released_in(&Features::all()), s);
    }

    #[test]
    fn nur_vice_blendet_alles_andere_aus_und_laesst_die_datei_unberuehrt() {
        let s = full();
        let f = Features::from_json(r#"{"format":1,"links":["vice"]}"#);
        let r = s.released_in(&f);
        assert!(r.target("atari8").unwrap().emulators.is_empty(), "weder Altirra noch atari800");
        assert!(r.target("mega65").unwrap().emulators.is_empty(), "kein Xemu");
        assert_eq!(r.target("c64").unwrap().emulators.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(), ["x64sc"]);
        assert!(r.machines.is_empty(), "keine Ultimate, kein MEGA65-Gerät");
        assert_eq!(r.target("c64").unwrap().start_on, "", "Startziel auf eine abgewählte Maschine fällt auf den Emulator zurück");
        // die Sicht ändert das Original nicht: ein volles Release findet alles wieder
        assert_eq!(s.machines.len(), 2);
        assert_eq!(s.target("atari8").unwrap().emulators.len(), 1);
        assert_eq!(s.target("c64").unwrap().start_on, "m1");
        // Standard rückt auf den ersten sichtbaren Emulator
        let mut s2 = full();
        s2.target_mut("atari8").unwrap().add_emulator("atari800");
        let only800 = Features::from_json(r#"{"format":1,"links":["atari800"]}"#);
        let r2 = s2.released_in(&only800);
        assert_eq!(r2.emulator_for("atari8").kind, "atari800");
        assert!(r2.target("atari8").unwrap().emulators[0].standard);
    }
}
