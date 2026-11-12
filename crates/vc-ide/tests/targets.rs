// SPDX-License-Identifier: GPL-3.0-or-later
use vc_ide::settings::*;
use vc_ide::targets::*;

#[test]
fn legacy_settings_migrate_into_targets_once() {
    let mut s = Settings::default();
    s.emulators.vice.x64sc = "~/vice/x64sc".into(); s.emulators.vice.args = "-VICIIfilter 0".into();
    s.emulators.atari800.path = "/opt/a800".into(); s.emulators.atari800.os_rom = "/roms/xl.rom".into(); s.emulators.altirra.default_for_atari = "atari800".into();
    s.emulators.xemu.rom_dir = "/roms".into();
    s.machines.push(Machine { id: "m1".into(), kind: "ultimate64".into(), host: "1.2.3.4".into(), ..Default::default() });
    s.machines.push(Machine { id: "m2".into(), kind: "mega65".into(), device: "/dev/x".into(), ..Default::default() });
    assert!(s.migrate_legacy());
    assert_eq!(s.emulators, Emulators::default(), "alte Felder sind leer");
    assert_eq!(s.targets.len(), SYSTEMS.len());
    let c64 = s.target("c64").unwrap();
    assert_eq!(c64.emulators.len(), 1);
    assert_eq!(c64.standard_emulator().unwrap().args, "-VICIIfilter 0");
    let a8 = s.target("atari8").unwrap();
    assert_eq!(a8.emulators.len(), 2);
    assert_eq!(a8.standard_emulator().unwrap().kind, "atari800", "default_for_atari=atari800 bleibt erhalten");
    assert_eq!(a8.emulators.iter().filter(|e| e.standard).count(), 1);
    assert_eq!(s.machine("m1").unwrap().system(), "c64");
    assert_eq!(s.machine("m2").unwrap().target, "mega65");
    assert!(!s.migrate_legacy(), "zweites Mal ändert nichts");
    let e = s.emulator_for("c64");
    assert_eq!(e.kind, "x64sc"); assert_eq!(e.args, vec!["-VICIIfilter", "0"]);
    assert_eq!(s.emulator_for("mega65").rom.as_deref(), Some("/roms"));
    assert_eq!(s.emulator_of_kind("atari8", "atari800").rom.as_deref(), Some("/roms/xl.rom"));
}

#[test]
fn exactly_one_standard_emulator_and_removal_reassigns() {
    let mut s = Settings::default();
    s.add_target("c64");
    let t = s.target_mut("c64").unwrap();
    let a = t.emulators[0].id.clone();
    let b = t.add_emulator("x64");
    assert_eq!(t.emulators.iter().filter(|e| e.standard).count(), 1);
    t.set_standard(&b);
    assert_eq!(t.standard_emulator().unwrap().id, b);
    t.remove_emulator(&b);
    assert_eq!(t.standard_emulator().unwrap().id, a, "nach dem Löschen des Standards rückt der nächste nach");
    assert_eq!(t.emulators.iter().filter(|e| e.standard).count(), 1);
    let again = t.add_emulator("x64sc"); assert_ne!(again, a, "Kennungen bleiben eindeutig");
}

#[test]
fn start_target_resolution_and_machine_removal() {
    let mut s = Settings::default();
    s.add_target("c64");
    let id = s.add_machine("c64", "ultimate64");
    assert!(s.start_machine("", "c64").is_none(), "Standard: Emulator");
    s.target_mut("c64").unwrap().start_on = id.clone();
    assert_eq!(s.start_machine("", "c64").unwrap().id, id, "Standardziel Hardware");
    assert!(s.start_machine("@emulator", "c64").is_none(), "Werkzeugleiste kann den Emulator erzwingen");
    assert_eq!(s.start_machine(&id, "plus4").unwrap().id, id, "ausdrückliche Wahl gilt immer");
    s.remove_machine(&id);
    assert!(s.target("c64").unwrap().start_on.is_empty(), "gelöschte Maschine ist kein Startziel mehr");
    assert!(s.start_machine("", "c64").is_none());
}

#[test]
fn targets_roundtrip_and_old_files_still_load() {
    let mut s = Settings::default();
    s.add_target("c64"); s.add_target("atari8"); s.add_target("mega65");
    assert_eq!(s.targets.iter().map(|t| t.system.as_str()).collect::<Vec<_>>(), ["c64", "atari8", "mega65"], "Reihenfolge der Navigation");
    let text = serde_json::to_string(&s).unwrap();
    assert!(!text.contains("\"emulators\":{"), "leere alte Felder werden nicht mehr geschrieben");
    assert_eq!(serde_json::from_str::<Settings>(&text).unwrap(), s);
    let old: Settings = serde_json::from_str(r#"{"machines":[{"id":"x","kind":"ultimate64"}]}"#).unwrap();
    assert!(old.machines[0].run_after && !old.machines[0].reset_before && old.targets.is_empty());
}
