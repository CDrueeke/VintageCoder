// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

fn tmp() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!("vc-setup-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn all(_: &str) -> bool { true }

fn machine_with_password() -> Machine { Machine { id: "m1".into(), name: "Ultimate".into(), kind: "ultimate64".into(), host: "192.168.1.50".into(), password: "geheim123".into(), target: "c64".into(), ..Default::default() } }

fn written(paths: &Paths, s: &Settings) { s.save_atomic(&paths.settings).unwrap(); }

#[test]
fn schritte_der_wege() {
    assert_eq!(Mode::Quick.steps().len(), 7);
    assert_eq!(Mode::Full.steps().len(), 12);
    // Schnelleinrichtung blendet 5 bis 9 aus und behält Anfang und Ende
    let q = Mode::Quick.steps();
    for t in [Topic::Emulators, Topic::Roms, Topic::Hardware, Topic::StartTarget, Topic::Editor] { assert!(!q.contains(&t)); }
    assert_eq!(q.first(), Some(&Topic::Welcome)); assert_eq!(q.last(), Some(&Topic::Done));
    // Teil-Assistenten enden in der Zusammenfassung
    for p in [Part::Roms, Part::Emulators, Part::Hardware, Part::DesignEditor] { assert_eq!(Mode::Part(p).steps().last(), Some(&Topic::Summary)); }
    for t in Topic::ALL { assert_eq!(Topic::from_id(t.id()), Some(t)); }
}

#[test]
fn erststart_erkennung() {
    let d = tmp(); let p = Paths::in_dir(&d);
    assert_eq!(start_state(&p), Start::FirstRun);
    written(&p, &Settings::default());
    assert_eq!(start_state(&p), Start::OfferOnce);
    WizardState::default().save(&p.wizard).unwrap();
    assert_eq!(start_state(&p), Start::Normal);
}

#[test]
fn wizard_json_ohne_zugangsdaten_und_mit_englischen_schluesseln() {
    let d = tmp(); let p = Paths::in_dir(&d);
    let mut w = WizardState { first_run_done: true, mode: "quick".into(), ..Default::default() };
    w.mark_later(Topic::Hardware); w.dismiss("emu:c64:x64sc"); w.last_run = "2026-11-12".into();
    w.save(&p.wizard).unwrap();
    let t = std::fs::read_to_string(&p.wizard).unwrap();
    for k in ["\"version\"", "\"first_run_done\"", "\"last_run\"", "\"mode\"", "\"later\"", "\"dismissed\"", "\"examples_copied\""] { assert!(t.contains(k), "{k} fehlt in {t}"); }
    assert!(t.contains("\"hardware\""));
    assert!(!t.to_lowercase().contains("password") && !t.contains('/'), "keine Pfade, keine Zugangsdaten: {t}");
    assert_eq!(WizardState::load(&p.wizard), w.clone().tap(|w| w.version = WIZARD_VERSION));
    assert!(w.incomplete());
    w.dismiss("banner"); assert!(!w.incomplete());
}

trait Tap: Sized { fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self { f(&mut self); self } }
impl Tap for WizardState {}

#[test]
fn datum_aus_sekunden() {
    assert_eq!(date_text(0), "1970-01-01");
    assert_eq!(date_text(1_000_000_000), "2001-09-09");
    assert_eq!(date_text(1_794_441_600), "2026-11-12");
    assert_eq!(date_text(1_709_164_800), "2024-02-29");
}

#[test]
fn migration_hebt_alte_dateien_an_und_stuft_neuere_nicht_herab() {
    let mut s = Settings::default(); s.emulators.atari800.path = "/x/atari800".into();
    migrate(&mut s);
    assert_eq!(s.schema_version, 2);
    assert!(s.target("atari8").unwrap().emulators.iter().any(|e| e.path == "/x/atari800"));
    // neuere Version bleibt erhalten, auch nach merge
    let mut n = Settings::default(); n.schema_version = 7; n.add_target("c64");
    migrate(&mut n); assert_eq!(n.schema_version, 7);
    let cur = Snapshot { settings: n.clone(), view: ViewPrefs::default(), disk_hash: 0 };
    let (m, _, _) = merge(&cur, &Draft::from_snapshot(&cur), &all);
    assert_eq!(m.schema_version, 7);
}

#[test]
fn diff_neu_geaendert_entfernt_unveraendert() {
    let mut s = Settings::default(); s.add_target("c64"); s.add_target("c128"); s.machines.push(machine_with_password());
    let cur = Snapshot { settings: s.clone(), view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur);
    set_system(&mut d.settings, "c128", false);          // ausblenden = Removed
    set_system(&mut d.settings, "vic20", true);          // New
    d.settings.target_mut("c64").unwrap().emulators[0].path = "/neu/x64sc".into();   // Changed
    d.view.scheme = "light".into();
    let ch = diff(&cur, &d);
    let kind = |k: &str| ch.iter().find(|c| c.key == k).unwrap().kind;
    assert_eq!(kind("target:vic20"), Kind::New);
    assert_eq!(kind("target:c128"), Kind::Removed);
    assert_eq!(kind("target:c64"), Kind::Same);
    assert_eq!(kind(&format!("emu:c64:{}", s.target("c64").unwrap().emulators[0].id)), Kind::Changed);
    assert_eq!(kind("view:scheme"), Kind::Changed);
    assert_eq!(kind("view:zoom"), Kind::Same);
    assert!(ch.iter().find(|c| c.key == "target:c128").unwrap().note.contains("bleiben erhalten"));
    // Voreinstellung der Auswahl: nur Änderungen sind angehakt
    assert!(ch.iter().filter(|c| c.kind == Kind::Same).all(|c| !c.selected));
    assert!(ch.iter().filter(|c| c.kind != Kind::Same).all(|c| c.selected));
}

#[test]
fn passwoerter_stehen_nie_in_der_aenderungsliste() {
    let mut s = Settings::default(); s.add_target("c64"); s.machines.push(machine_with_password());
    let cur = Snapshot { settings: s, view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur);
    d.settings.machines[0].password = "ganz-anderes-geheimnis".into();
    d.settings.machines.push(Machine { id: "m2".into(), name: "Zweite".into(), kind: "ultimate64".into(), host: "h".into(), password: "noch-eins".into(), ..Default::default() });
    for c in diff(&cur, &d) {
        let all = format!("{} {} {} {}", c.label, c.old, c.new, c.note);
        for secret in ["geheim123", "ganz-anderes-geheimnis", "noch-eins"] { assert!(!all.contains(secret), "Passwort in {all}"); }
    }
    let m1 = diff(&cur, &d).into_iter().find(|c| c.key == "machine:m1").unwrap();
    assert_eq!(m1.kind, Kind::Changed); assert!(m1.sensitive);
    assert!(m1.new.contains("Passwort geändert") || m1.old != m1.new);
}

#[test]
fn merge_wendet_nur_gewaehlte_aenderungen_an() {
    let mut s = Settings::default(); s.add_target("c64");
    let cur = Snapshot { settings: s, view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur);
    set_system(&mut d.settings, "mega65", true);
    d.settings.projects_dir = "/home/x/P".into();
    d.view.font = 20.0; d.view.scheme = "light".into();
    let (m, v, _) = merge(&cur, &d, &|k| k != "projects_dir" && k != "view:font");
    assert!(m.target("mega65").is_some());
    assert!(m.projects_dir.is_empty());
    assert_eq!(v.font, 13.0); assert_eq!(v.scheme, "light");
    // alles gewählt: Entwurf wird vollständig übernommen
    let (m, v, _) = merge(&cur, &d, &all);
    let mut want = d.settings.clone(); want.schema_version = SCHEMA_VERSION;
    assert_eq!(m, want); assert_eq!(v, d.view);
}

#[test]
fn abhaengigkeit_startziel_auf_abgewaehlte_maschine() {
    let mut s = Settings::default(); s.add_target("c64");
    let cur = Snapshot { settings: s, view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur);
    d.settings.machines.push(machine_with_password());
    d.settings.target_mut("c64").unwrap().start_on = "m1".into();
    let (m, _, notes) = merge(&cur, &d, &|k| k != "machine:m1");
    assert_eq!(m.target("c64").unwrap().start_on, "");
    assert!(notes.iter().any(|n| n.contains("Startziel")), "{notes:?}");
}

#[test]
fn ausblenden_erhaelt_eintraege_und_einblenden_stellt_sie_her() {
    let mut s = Settings::default(); set_system(&mut s, "c64", true);
    let id = s.target("c64").unwrap().emulators[0].id.clone();
    s.target_mut("c64").unwrap().emulators[0].path = "/mein/vice".into();
    set_system(&mut s, "c64", false);
    assert!(s.target("c64").is_none());
    assert_eq!(s.hidden_targets[0].emulators[0].path, "/mein/vice");
    set_system(&mut s, "c64", true);
    assert_eq!(s.target("c64").unwrap().emulators[0].id, id);
    assert_eq!(s.target("c64").unwrap().emulators[0].path, "/mein/vice");
    assert!(s.hidden_targets.is_empty());
}

#[test]
fn altirra_wird_ausserhalb_von_windows_nicht_angeboten() {
    assert!(!kind_offered_on("altirra", false)); assert!(kind_offered_on("altirra", true)); assert!(kind_offered_on("atari800", false));
    let mut s = Settings::default(); set_system(&mut s, "atari8", true);
    let k = &s.target("atari8").unwrap().emulators[0].kind;
    if cfg!(windows) { assert_eq!(k, "altirra"); } else { assert_eq!(k, "atari800"); }
    let rows = toolchain_rows(&["atari8".into()]);
    assert_eq!(rows.iter().any(|r| r.kind == "altirra"), cfg!(windows));
    assert!(rows[0].builtin);
}

#[test]
fn apply_schreibt_atomar_mit_sicherung_und_undo() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let mut s = Settings::default(); s.add_target("c64"); written(&p, &s);
    let before = std::fs::read(&p.settings).unwrap();
    let cur = load_snapshot(&p).unwrap();
    let mut d = Draft::from_snapshot(&cur);
    set_system(&mut d.settings, "vic20", true); d.view.font = 18.0;
    let a = apply_to(&p, &cur, &d, &all).unwrap();
    assert_eq!(std::fs::read(a.backup.as_ref().unwrap()).unwrap(), before, ".bak enthält den alten Stand");
    assert!(p.backup().to_string_lossy().ends_with("settings.json.bak"));
    let now = load_snapshot(&p).unwrap();
    assert!(now.settings.target("vic20").is_some()); assert_eq!(now.view.font, 18.0); assert_eq!(now.settings.schema_version, SCHEMA_VERSION);
    // keine Reste der temporären Datei
    assert!(std::fs::read_dir(&dir).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().ends_with(".tmp")));
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(std::fs::metadata(&p.settings).unwrap().permissions().mode() & 0o777, 0o600); }
    undo_to(&p, &a).unwrap();
    assert_eq!(std::fs::read(&p.settings).unwrap(), before);
    assert!(!p.view.exists(), "view.json existierte vorher nicht");
}

#[test]
fn apply_ohne_auswahl_aendert_die_datei_nicht() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let mut s = Settings::default(); s.add_target("c64"); s.schema_version = SCHEMA_VERSION; written(&p, &s);
    ViewPrefs::default().save(&p.view).unwrap();
    let (b1, b2) = (std::fs::read(&p.settings).unwrap(), std::fs::read(&p.view).unwrap());
    let cur = load_snapshot(&p).unwrap();
    let mut d = Draft::from_snapshot(&cur); set_system(&mut d.settings, "mega65", true); d.view.scheme = "light".into(); d.settings.projects_dir = "/x/y".into();
    apply_to(&p, &cur, &d, &|_| false).unwrap();
    assert_eq!(std::fs::read(&p.settings).unwrap(), b1); assert_eq!(std::fs::read(&p.view).unwrap(), b2);
}

#[test]
fn teil_assistent_aendert_nur_seinen_bereich() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let mut s = Settings::default(); s.add_target("c64"); s.machines.push(machine_with_password()); s.schema_version = SCHEMA_VERSION; written(&p, &s);
    let cur = load_snapshot(&p).unwrap();
    let mut d = Draft::from_snapshot(&cur);
    d.settings.machines[0].host = "10.0.0.9".into();                 // gehört zu „Hardware“
    d.settings.target_mut("c64").unwrap().emulators[0].path = "/andere/x64sc".into();   // gehört zu „Emulatoren“
    d.view.zoom = 1.5;                                               // gehört zu „Design“
    let mut ch = diff(&cur, &d);
    only_topics(&mut ch, &[Topic::Hardware, Topic::StartTarget, Topic::Summary]);
    let sel: BTreeSet<String> = ch.iter().filter(|c| c.selected && c.kind != Kind::Same).map(|c| c.key.clone()).collect();
    apply_to(&p, &cur, &d, &|k| sel.contains(k)).unwrap();
    let now = load_snapshot(&p).unwrap();
    assert_eq!(now.settings.machines[0].host, "10.0.0.9");
    assert_eq!(now.settings.target("c64").unwrap().emulators[0].path, cur.settings.target("c64").unwrap().emulators[0].path, "Emulator unverändert");
    assert_eq!(now.view, cur.view, "Design unverändert");
}

#[test]
fn apply_bricht_ab_wenn_die_datei_von_aussen_geaendert_wurde() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let mut s = Settings::default(); s.add_target("c64"); written(&p, &s);
    let cur = load_snapshot(&p).unwrap();
    let mut other = s.clone(); other.add_target("c128"); written(&p, &other);
    let d = Draft::from_snapshot(&cur);
    let e = apply_to(&p, &cur, &d, &all).unwrap_err();
    assert!(e.contains("geändert"), "{e}");
    assert!(load_snapshot(&p).unwrap().settings.target("c128").is_some(), "fremde Änderung bleibt");
}

#[test]
fn apply_baut_bei_schreibfehler_zurueck() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let mut s = Settings::default(); s.add_target("c64"); written(&p, &s);
    let before = std::fs::read(&p.settings).unwrap();
    let cur = load_snapshot(&p).unwrap();
    // view.json lässt sich nicht schreiben: Ort ist ein Ordner → Fehler im dritten Schritt
    let mut bad = p.clone(); bad.view = dir.join("view-ist-ein-ordner"); std::fs::create_dir_all(&bad.view).unwrap();
    let mut d = Draft::from_snapshot(&cur); set_system(&mut d.settings, "vic20", true); d.view.font = 22.0;
    let e = apply_to(&bad, &cur, &d, &all).unwrap_err();
    assert!(e.contains("view-ist-ein-ordner"), "Fehler nennt den Pfad: {e}");
    assert_eq!(std::fs::read(&p.settings).unwrap(), before, "settings.json zurückgebaut");
}

#[test]
fn defekte_datei_wird_nicht_ueberschrieben_und_laesst_sich_sichern() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    std::fs::write(&p.settings, "{ kaputt").unwrap();
    assert!(load_snapshot(&p).is_err());
    let bak = backup_broken(&p).unwrap();
    assert_eq!(std::fs::read_to_string(bak).unwrap(), "{ kaputt");
}

#[test]
fn beispiele_kopieren_ueberschreibt_nie() {
    let root = tmp().join("examples"); let proj = root.join("c64/demo"); std::fs::create_dir_all(proj.join("sub")).unwrap();
    std::fs::write(proj.join("demo.vcprj"), "{}").unwrap(); std::fs::write(proj.join("demo.asm"), "neu").unwrap(); std::fs::write(proj.join("sub/x.txt"), "x").unwrap();
    std::fs::create_dir_all(root.join("c64/lib")).unwrap(); std::fs::write(root.join("c64/lib/a.asm"), "").unwrap();   // kein Projekt (keine .vcprj)
    let items = list_examples(&root, &["c64".into(), "mega65".into()]);
    assert_eq!(items.len(), 1); assert_eq!(items[0].id(), "c64/demo");
    let dest = tmp().join("Beispiele");
    let r = copy_examples(&items, &dest).unwrap();
    assert_eq!(r.copied, vec!["c64/demo"]); assert_eq!(r.files_created.len(), 3);
    std::fs::write(dest.join("c64/demo/demo.asm"), "MEINE ÄNDERUNG").unwrap();
    let r2 = copy_examples(&items, &dest).unwrap();
    assert_eq!(r2.files_skipped, 3); assert!(r2.copied.is_empty());
    assert_eq!(std::fs::read_to_string(dest.join("c64/demo/demo.asm")).unwrap(), "MEINE ÄNDERUNG");
}

#[test]
fn atari_beispiele_liegen_im_ordner_atari() {
    assert_eq!(example_dir_name("atari8"), "atari"); assert_eq!(example_dir_name("c64"), "c64");
    // die echten Beispiele der Installation
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let v = list_examples(&root, &["atari8".into(), "c64".into()]);
    assert!(v.iter().any(|i| i.system == "atari8" && i.name == "jumprun"), "{:?}", v.iter().map(|i| i.id()).collect::<Vec<_>>());
    assert!(v.iter().any(|i| i.id() == "c64/jumprun"));
}

#[test]
fn apply_kopiert_beispiele_und_undo_entfernt_sie() {
    let dir = tmp(); let p = Paths::in_dir(&dir);
    let root = tmp().join("examples"); let proj = root.join("c64/demo"); std::fs::create_dir_all(&proj).unwrap(); std::fs::write(proj.join("demo.vcprj"), "{}").unwrap();
    let mut s = Settings::default(); s.add_target("c64"); written(&p, &s);
    let cur = load_snapshot(&p).unwrap();
    let base = tmp().join("Projekte");
    let mut d = Draft::from_snapshot(&cur); d.settings.projects_dir = base.to_string_lossy().into_owned(); d.examples = ExamplesCfg { copy: true, excluded: Default::default(), source: Some(root), dest: String::new() };
    let ch = diff(&cur, &d);
    assert!(ch.iter().any(|c| c.key == "examples"), "Beispiele erscheinen in der Änderungsliste");
    assert!(!base.exists(), "vor Übernehmen wird nichts kopiert");
    let a = apply_to(&p, &cur, &d, &all).unwrap();
    assert!(base.join("Beispiele/c64/demo/demo.vcprj").is_file());
    assert_eq!(a.copy.copied, vec!["c64/demo"]);
    undo_to(&p, &a).unwrap();
    assert!(!base.join("Beispiele/c64/demo/demo.vcprj").exists());
}

#[test]
fn projektordner_regeln() {
    assert!(check_projects_dir("", None).is_err());
    assert!(check_projects_dir("relativ/pfad", None).is_err());
    assert!(check_projects_dir("/", None).is_err());
    assert!(check_projects_dir("/usr/local/projekte", None).is_err());
    let inst = tmp().join("Install");
    std::fs::create_dir_all(&inst).unwrap();
    assert!(check_projects_dir(inst.join("x").to_str().unwrap(), Some(&inst)).unwrap_err().contains("Installation"));
    let ok = tmp(); assert_eq!(check_projects_dir(ok.to_str().unwrap(), None), Ok(ProjectsDirState::Writable));
    assert_eq!(check_projects_dir(ok.join("neu/ordner").to_str().unwrap(), None), Ok(ProjectsDirState::WillCreate));
    // Trennzeichen sind betriebssystemeigen (unter Windows `\`): zum Vergleich vereinheitlichen
    let n = |p: String| p.replace('\\', "/");
    assert_eq!(n(default_projects_dir(None, Some(Path::new("/home/a")), false)), "/home/a/VintageCoder");
    assert!(n(default_projects_dir(None, Some(Path::new("C:/Users/a")), true)).ends_with("Documents/VintageCoder"));
    assert!(n(default_projects_dir(Some(Path::new("/usb/VC")), Some(Path::new("/home/a")), false)).starts_with("/usb/VC"));
}

#[test]
fn export_ohne_passwort_mit_relativen_pfaden_und_import() {
    let base = tmp(); let mut s = Settings::default(); s.projects_dir = base.to_string_lossy().into_owned();
    s.add_target("c64"); s.machines.push(machine_with_password());
    s.target_mut("c64").unwrap().emulators[0].path = base.join("tools/x64sc").to_string_lossy().into_owned();
    s.target_mut("c64").unwrap().emulators[0].rom = "/ganz/woanders/rom".into();
    let v = ViewPrefs { scheme: "light".into(), zoom: 1.2, font: 15.0 };
    // Voreinstellung: kein Passwort, Pfade außerhalb leer
    let j = export_json(&s, &v, ExportOpts::default());
    let text = serde_json::to_string_pretty(&j).unwrap();
    assert!(!text.contains("geheim123"), "Passwort im Export: {text}");
    assert!(text.contains("{projects_dir}/tools/x64sc"));
    assert!(!text.contains("/ganz/woanders"));
    assert_eq!(j["kind"], EXPORT_KIND); assert_eq!(j["schema_version"], SCHEMA_VERSION);
    assert_eq!(outside_paths(&s), vec!["/ganz/woanders/rom".to_string()]);
    // mit Wunsch: Passwort und Pfade
    let j2 = export_json(&s, &v, ExportOpts { passwords: true, outside_paths: true });
    let t2 = serde_json::to_string(&j2).unwrap(); assert!(t2.contains("geheim123") && t2.contains("/ganz/woanders/rom"));
    // Import auf einem Rechner mit eigenem Passwort: leeres Passwort löscht nichts
    let mut mine = Settings::default(); mine.machines.push(machine_with_password()); mine.projects_dir = tmp().to_string_lossy().into_owned();
    let cur = Snapshot { settings: mine.clone(), view: ViewPrefs::default(), disk_hash: 0 };
    let imp = import_json(&text, &cur).unwrap();
    assert_eq!(imp.draft.settings.machines[0].password, "geheim123");
    assert_eq!(imp.draft.view.scheme, "light");
    assert!(imp.draft.settings.target("c64").is_some());
    assert!(imp.path_notes.iter().any(|n| n.contains("existiert hier nicht")), "{:?}", imp.path_notes);
    // dasselbe Diff wie bei jeder Änderung
    assert!(diff(&cur, &imp.draft).iter().any(|c| c.key == "target:c64" && c.kind == Kind::New));
    assert!(import_json("{}", &cur).is_err());
    assert!(import_json("kein json", &cur).is_err());
    let neuer = text.replace(&format!("\"schema_version\": {SCHEMA_VERSION}"), "\"schema_version\": 99");
    assert!(import_json(&neuer, &cur).unwrap_err().contains("neueren"));
}

#[test]
fn zuruecksetzen_je_bereich() {
    let mut s = Settings::default(); s.add_target("c64"); s.machines.push(machine_with_password());
    s.target_mut("c64").unwrap().start_on = "m1".into();
    s.target_mut("c64").unwrap().emulators[0].path = "/x".into();
    s.projects_dir = "/p".into(); s.roms.use_foreign = true;
    let cur = Snapshot { settings: s.clone(), view: ViewPrefs { scheme: "light".into(), zoom: 2.0, font: 20.0 }, disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur);
    reset_area(&mut d, ResetArea::Design); assert_eq!(d.view, ViewPrefs::default());
    reset_area(&mut d, ResetArea::Roms); assert!(!d.settings.roms.use_foreign);
    reset_area(&mut d, ResetArea::All);
    assert_eq!(d.settings.target("c64").unwrap().emulators[0].path, "");
    assert_eq!(d.settings.machines.len(), 1, "Alles lässt Hardware und Passwörter in Ruhe");
    assert_eq!(d.settings.target("c64").unwrap().start_on, "m1");
    assert!(d.settings.projects_dir.is_empty());
    reset_area(&mut d, ResetArea::Hardware);
    assert!(d.settings.machines.is_empty() && d.settings.target("c64").unwrap().start_on.is_empty());
}

#[test]
fn badges_und_probleme() {
    let mut s = Settings::default(); s.add_target("c64");
    s.target_mut("c64").unwrap().emulators[0].path = "/gibt/es/nicht/x64sc".into();
    let cur = Snapshot { settings: s.clone(), view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur); d.view.font = 20.0; set_system(&mut d.settings, "vic20", true);
    let pr = problems(&s);
    assert!(pr.iter().any(|p| p.topic == Topic::Emulators && p.text.contains("existiert nicht mehr")));
    let b = badges(&diff(&cur, &d), &pr, &[], &BTreeSet::from([Topic::Hardware]));
    assert_eq!(b[&Topic::Emulators], Badge::Problem);
    assert_eq!(b[&Topic::Design], Badge::Changed);
    assert_eq!(b[&Topic::Systems], Badge::New);
    assert_eq!(b[&Topic::Hardware], Badge::Skipped);
    assert_eq!(b[&Topic::Editor], Badge::Unchanged);
}

#[test]
fn neufunde_werden_nur_vorgeschlagen_und_ignorierte_kommen_nicht_wieder() {
    let mut s = Settings::default(); s.add_target("c64");
    let disc = |k: &str| if k == "x64sc" || k == "xemu" { Some("/irgendwo".to_string()) } else { None };
    let w = WizardState::default();
    // x64sc ist eingetragen, xemu gehört zu mega65 (nicht eingerichtet) → kein Fund
    assert!(findings(&s, &w, &disc).is_empty());
    set_system(&mut s, "mega65", true); s.target_mut("mega65").unwrap().emulators.clear();
    let f = findings(&s, &w, &disc);
    assert_eq!(f.len(), 1); assert_eq!(f[0].id, "emu:mega65:xemu");
    let mut w2 = w.clone(); w2.dismiss(&f[0].id);
    assert!(findings(&s, &w2, &disc).is_empty());
    let cur = Snapshot { settings: s, view: ViewPrefs::default(), disk_hash: 0 };
    let mut d = Draft::from_snapshot(&cur); accept_finding(&mut d, &f[0]);
    assert_eq!(d.settings.target("mega65").unwrap().emulators[0].kind, "xemu");
    assert!(diff(&cur, &d).iter().any(|c| c.kind == Kind::New && c.key.starts_with("emu:mega65:")), "Annehmen erscheint im Diff, geschrieben wird erst beim Übernehmen");
}

#[test]
fn neues_beim_programmstart() {
    let disc = |k: &str| if k == "atari800" { Some("/x".to_string()) } else { None };
    let mut w = WizardState { first_run_done: true, ..Default::default() };
    assert_eq!(news_since_last_run(&w, &disc), vec!["atari800"]);
    w.known_programs = found_kinds(&disc); assert!(news_since_last_run(&w, &disc).is_empty());
    let off = WizardState { first_run_done: true, no_news_hint: true, ..Default::default() };
    assert!(news_since_last_run(&off, &disc).is_empty());
    assert!(news_since_last_run(&WizardState::default(), &disc).is_empty(), "vor dem ersten Lauf kein Hinweis");
}

#[test]
fn linux_paketmanager_aus_os_release() {
    assert_eq!(package_manager_from_os_release("ID=ubuntu\nID_LIKE=debian\n"), Some("apt"));
    assert_eq!(package_manager_from_os_release("ID=debian\n"), Some("apt"));
    assert_eq!(package_manager_from_os_release("ID=fedora\n"), Some("dnf"));
    assert_eq!(package_manager_from_os_release("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n"), Some("zypper"));
    assert_eq!(package_manager_from_os_release("ID=arch\n"), None);
}

#[test]
fn hinweise_aus_den_installationsdaten() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let h = load_hints(&data);
    assert!(!h.is_empty(), "data/setup/hints.json lesbar");
    for kind in ["x64sc", "atari800", "xemu"] { for os in ["macos", "windows", "linux"] { assert!(!hints_for(&h, kind, os).is_empty(), "Hinweis für {kind} auf {os}"); } }
    assert!(!hints_for(&h, "altirra", "windows").is_empty());
    for x in &h { assert!(["macos", "windows", "linux"].contains(&x.os.as_str())); assert!(!x.command.contains("sudo rm") ); }
    assert_eq!(hint_program("x128"), "vice");
}

#[test]
fn suchorte_nennen_path_zuerst() { assert_eq!(search_places("atari800")[0], "PATH"); assert!(search_places("x64sc").len() > 3); }

#[test]
fn neue_installation_bekommt_nicht_alle_systeme() {
    let d = tmp(); let p = Paths::in_dir(&d);
    let s = load_snapshot(&p).unwrap();
    assert!(s.settings.targets.is_empty(), "ohne Datei keine Migration");
    assert_eq!(s.disk_hash, 0);
    let draft = Draft::for_first_run(&s);
    assert_eq!(draft.settings.targets.len(), 1);
    assert_eq!(draft.settings.targets[0].system, "c64");
}
