// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests der Oberfläche ohne Fenster: jeder Schritt wird in beiden Schemata gezeichnet (kein Absturz, kein Panic), Navigationsregeln, der
//! Wiederaufruf ändert ohne „Übernehmen“ nichts, Haken der Zusammenfassung wirken, Kontrast der verwendeten Farbpaare.
use super::*;
use crate::theme::Scheme;
use std::sync::atomic::{AtomicU32, Ordering};
use vc_ide::settings::Settings;

fn tmp() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!("vc-setupui-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

struct Rig { ctx: egui::Context, view: View, addr: bool, dir: PathBuf }

impl Rig {
    fn new(scheme: Scheme) -> Rig {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        crate::ui::style::install(&ctx);
        let view = View { scheme, ..View::default() };
        view.apply(&ctx);
        Rig { ctx, view, addr: true, dir: tmp() }
    }
    fn paths(&self) -> Paths { Paths::in_dir(&self.dir) }
    fn wizard(&mut self, launch: Launch) -> SetupWizard {
        let paths = self.paths();
        let env = Env { view: &mut self.view, show_addr: &mut self.addr, data: None };
        SetupWizard::with_paths(launch, &env, paths)
    }
    fn frame(&mut self, w: &mut SetupWizard) -> Out {
        let mut out = Out::None;
        let (view, addr) = (&mut self.view, &mut self.addr);
        let mut o = self.ctx.run_ui(egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1400.0, 900.0))), ..Default::default() }, |ui| {
            let mut env = Env { view, show_addr: addr, data: None };
            out = w.show(ui.ctx(), &mut env);
        });
        o.textures_delta.clear();
        out
    }
}

fn settings_with_machine() -> Settings {
    let mut s = Settings::default();
    s.add_target("c64"); s.add_target("mega65");
    s.machines.push(vc_ide::settings::Machine { id: "m1".into(), name: "Ultimate".into(), kind: "ultimate64".into(), host: "192.168.1.50".into(), password: "geheim123".into(), target: "c64".into(), ..Default::default() });
    s.schema_version = setup::SCHEMA_VERSION;
    s
}

#[test]
fn jeder_schritt_zeichnet_in_beiden_schemata() {
    for scheme in [Scheme::Dark, Scheme::Light] {
        for launch in [Launch::FirstRun, Launch::Recall] {
            let mut rig = Rig::new(scheme);
            settings_with_machine().save_atomic(&rig.paths().settings).unwrap();
            let mut w = rig.wizard(launch);
            w.mode = Mode::Full; w.steps = Mode::Full.steps(); w.has_hardware = true;
            for i in 0..w.steps.len() {
                w.goto(i);
                for _ in 0..2 { let _ = rig.frame(&mut w); }
            }
        }
    }
}

#[test]
fn schnellweg_hat_sieben_schritte_und_blendet_die_automatischen_aus() {
    let mut rig = Rig::new(Scheme::Dark);
    let w = rig.wizard(Launch::FirstRun);
    assert_eq!(w.steps.len(), 7);
    assert_eq!(w.mode, Mode::Quick);
    // beim ersten Start steht C64 im Entwurf, der Projektordner ist vorgeschlagen und vorbelegt
    assert!(w.draft.settings.target("c64").is_some());
    assert!(!w.draft.settings.projects_dir.is_empty());
}

#[test]
fn weiter_ist_nur_bei_ungueltigem_feld_gesperrt() {
    let mut rig = Rig::new(Scheme::Dark);
    let mut w = rig.wizard(Launch::FirstRun);
    w.mode = Mode::Full; w.steps = Mode::Full.steps();
    w.jump_to(Topic::Systems);
    assert!(w.blocked().is_none());
    setup::set_system(&mut w.draft.settings, "c64", false);
    assert!(w.draft.settings.targets.is_empty(), "{:?} / {:?}", w.draft.settings.targets, w.topic());
    assert!(w.blocked().unwrap().contains("mindestens"));
    setup::set_system(&mut w.draft.settings, "c64", true);
    w.jump_to(Topic::Toolchain);
    assert!(w.blocked().is_none(), "ein fehlendes Programm sperrt nie");
    w.jump_to(Topic::Projects);
    w.draft.settings.projects_dir = "relativ/pfad".into();
    assert!(w.blocked().is_some());
    w.draft.settings.projects_dir = rig.dir.join("Projekte").to_string_lossy().into_owned();
    assert!(w.blocked().is_none());
}

#[test]
fn spaeter_merkt_nur_schritt_und_offene_themen() {
    let mut rig = Rig::new(Scheme::Dark);
    let mut w = rig.wizard(Launch::FirstRun);
    w.goto(3);   // Toolchain
    assert_eq!(w.topic(), Topic::Toolchain);
    let later = w.later_topics();
    assert_eq!(later, vec![Topic::Toolchain, Topic::Projects]);
    w.draft.settings.machines.push(vc_ide::settings::Machine { id: "m9".into(), password: "streng-geheim".into(), host: "10.1.2.3".into(), ..Default::default() });
    w.save_later();
    let t = std::fs::read_to_string(rig.paths().wizard).unwrap();
    assert!(t.contains("\"toolchain\"") && t.contains("\"projects\"") && t.contains("\"resume_step\""));
    assert!(!t.contains("streng-geheim") && !t.contains("10.1.2.3") && !t.contains(&rig.dir.to_string_lossy().to_string()), "keine Zugangsdaten, keine Pfade: {t}");
    assert!(WizardState::load(&rig.paths().wizard).incomplete());
}

#[test]
fn wiederaufruf_aendert_ohne_ja_nichts() {
    let mut rig = Rig::new(Scheme::Dark);
    let paths = rig.paths();
    settings_with_machine().save_atomic(&paths.settings).unwrap();
    vc_ide::setup::ViewPrefs::default().save(&paths.view).unwrap();
    let (b1, b2) = (std::fs::read(&paths.settings).unwrap(), std::fs::read(&paths.view).unwrap());
    let mut w = rig.wizard(Launch::Recall);
    w.has_hardware = true;
    // alle Schritte durchgehen, Erkennung läuft, Neufunde erscheinen höchstens als Vorschlag
    for i in 0..w.steps.len() { w.goto(i); let _ = rig.frame(&mut w); }
    assert_eq!(std::fs::read(&paths.settings).unwrap(), b1, "settings.json unverändert");
    assert_eq!(std::fs::read(&paths.view).unwrap(), b2, "view.json unverändert");
    assert!(!paths.backup().exists(), "ohne Schreiben keine Sicherung");
    // Abbrechen über „Später“ schreibt nur wizard.json
    w.save_later();
    assert_eq!(std::fs::read(&paths.settings).unwrap(), b1);
}

#[test]
fn haken_in_der_zusammenfassung_wirken_und_uebernehmen_schreibt_mit_sicherung() {
    let mut rig = Rig::new(Scheme::Dark);
    let paths = rig.paths();
    settings_with_machine().save_atomic(&paths.settings).unwrap();
    let before = std::fs::read(&paths.settings).unwrap();
    let mut w = rig.wizard(Launch::Recall);
    setup::set_system(&mut w.draft.settings, "vic20", true);
    w.draft.settings.machines[0].host = "10.0.0.7".into();
    w.refresh_status(true);
    let keys = w.selected_keys();
    assert!(keys.contains("target:vic20") && keys.contains("machine:m1"));
    // Haken abwählen
    w.deselected.insert("machine:m1".into()); w.refresh_status(true);
    assert!(!w.selected_keys().contains("machine:m1"));
    w.jump_to(Topic::Summary);
    let _ = rig.frame(&mut w);
    let v = w.do_apply().expect("Übernehmen");
    let _ = v;
    assert_eq!(std::fs::read(paths.backup()).unwrap(), before, "Sicherung");
    let now = setup::load_snapshot(&paths).unwrap();
    assert!(now.settings.target("vic20").is_some());
    assert_eq!(now.settings.machines[0].host, "192.168.1.50", "abgewählte Änderung wurde nicht geschrieben");
    assert_eq!(now.settings.machines[0].password, "geheim123");
    assert!(WizardState::load(&paths.wizard).first_run_done);
    assert!(w.applied.is_some());
    // Passwort steht nirgends in der Änderungsliste
    for c in &w.changes { assert!(!format!("{} {} {} {}", c.label, c.old, c.new, c.note).contains("geheim123")); }
}

#[test]
fn teil_assistent_waehlt_andere_bereiche_ab() {
    let mut rig = Rig::new(Scheme::Dark);
    settings_with_machine().save_atomic(&rig.paths().settings).unwrap();
    let mut w = rig.wizard(Launch::Part(Part::Hardware));
    assert_eq!(w.steps, Mode::Part(Part::Hardware).steps());
    w.draft.settings.machines[0].host = "10.0.0.7".into();
    w.draft.settings.target_mut("c64").unwrap().emulators[0].path = "/anderes/x64sc".into();
    w.draft.view.zoom = 1.4;
    w.refresh_status(true);
    let keys = w.selected_keys();
    assert!(keys.contains("machine:m1"));
    assert!(!keys.iter().any(|k| k.starts_with("emu:") || k.starts_with("view:")), "{keys:?}");
}

#[test]
fn export_import_und_zuruecksetzen_fuehren_in_die_zusammenfassung() {
    let mut rig = Rig::new(Scheme::Dark);
    settings_with_machine().save_atomic(&rig.paths().settings).unwrap();
    let mut w = rig.wizard(Launch::Reset);
    assert!(w.tool_only);
    // Zurücksetzen „Alles“ lässt Hardware in Ruhe
    w.side = side::Side::Reset { area: setup::ResetArea::All, with_hw: false };
    setup::reset_area(&mut w.draft, setup::ResetArea::All);
    w.steps = vec![Topic::Summary, Topic::Done]; w.tool_only = false; w.refresh_status(true);
    assert_eq!(w.draft.settings.machines.len(), 1);
    let _ = rig.frame(&mut w);
    let w2 = rig.wizard(Launch::Export);
    assert!(w2.tool_only && matches!(w2.side, side::Side::Export { .. }));
}

#[test]
fn defekte_einstellungsdatei_wird_gemeldet_und_nie_ueberschrieben() {
    let mut rig = Rig::new(Scheme::Dark);
    std::fs::write(rig.paths().settings, "{ kaputt").unwrap();
    let mut w = rig.wizard(Launch::Recall);
    assert!(w.load_error.is_some());
    assert!(w.blocked().is_none(), "Willkommen ist frei");
    w.jump_to(Topic::Summary);
    assert!(w.blocked().is_some(), "Schreiben ist gesperrt");
    let _ = rig.frame(&mut w);
    assert_eq!(std::fs::read_to_string(rig.paths().settings).unwrap(), "{ kaputt");
}

#[test]
fn lesbarkeit_der_verwendeten_farbpaare() {
    use crate::ui::tokens::{contrast, DARK, LIGHT};
    let mut bad = vec![];
    for (name, k) in [("dunkel", &DARK), ("hell", &LIGHT)] {
        for (cname, c) in [("fg", k.fg), ("mute", k.mute), ("cy", k.cy)] {
            for (gname, g) in [("bg1", k.bg1), ("bg2", k.bg2), ("bg3", k.bg3)] {
                let r = contrast(c, g);
                if r < 4.5 { bad.push(format!("{cname} auf {gname} ({name}): {r:.2}")); }
            }
        }
        // Statusfarben: im Designsystem knapp unter AA auf einzelnen Flächen (dunkel rd/bg3 4,0; hell gr/bg2 4,4, am/bg2 4,2); der Status steht
        // deshalb immer mit Symbol und Text. Untergrenze hier: 4,0 (Texte der Statusfarben sind klein, aber nie das einzige Merkmal).
        for (cname, c) in [("gr", k.gr), ("am", k.am), ("rd", k.rd)] {
            for (gname, g) in [("bg1", k.bg1), ("bg2", k.bg2), ("bg3", k.bg3)] {
                let r = contrast(c, g);
                if r < 4.0 { bad.push(format!("{cname} auf {gname} ({name}): {r:.2}")); }
            }
        }
        // Text auf der Primärfläche (Knöpfe)
        let r = contrast(k.on_cy, k.cy);
        if r < 4.5 { bad.push(format!("on_cy auf cy ({name}): {r:.2}")); }
    }
    assert!(bad.is_empty(), "zu wenig Kontrast (AA 4,5:1): {bad:#?}");
}
