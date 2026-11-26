// SPDX-License-Identifier: GPL-3.0-or-later
//! Einstellungen (E12): die Einstellungsdatei der IDE (`vc_ide::settings::Settings`) als JSON zur Oberfläche und zurück; Sicherung (Export ohne Passwörter, Import mit Rückfrage,
//! Zurücksetzen). Die Oberfläche bearbeitet einen Entwurf und schickt ihn ganz zum Speichern; der Kern prüft ihn über die Typen von `Settings`.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::settings::{self, Settings};

/// Dateiname mit Endung: fehlt jede Endung, wird `endung` ergänzt
fn mit_endung(name: &str, endung: &str) -> String { let datei = name.rsplit(['/', '\\']).next().unwrap_or(name); if name.is_empty() || datei.trim_start_matches('.').contains('.') { name.to_string() } else { format!("{name}.{endung}") } }

fn aus_json(v: &Value) -> Result<Settings, Fehler> { serde_json::from_value::<Settings>(v.clone()).map_err(|e| Fehler::mit("einst-ungueltig", json!({ "detail": e.to_string() }))) }

/// Die gespeicherten Einstellungen, der Ort der Datei und der persönliche Ordner (für Kurzschreibweise `~`)
pub fn lesen() -> Value {
    let mut g = settings::get(); g.migrate_legacy();   // wie die bisherige Oberfläche: aus den alten Feldern bzw. der Vorgabe entstehen die Zielsysteme
    json!({ "settings": serde_json::to_value(&g).unwrap_or(Value::Null), "file": settings::file().map(|p| p.to_string_lossy().into_owned()), "home": vc_core::userdirs::home_dir().map(|p| p.to_string_lossy().into_owned()),
        "effectiveProjects": vc_ide::setup::effective_projects_dir(&settings::get()), "mac": cfg!(target_os = "macos") })
}
/// Entwurf speichern; Antwort: der gespeicherte Stand
pub fn speichern(v: &Value) -> Result<Value, Fehler> {
    let mut s = aus_json(v)?; s.migrate_legacy(); settings::set(s).map_err(|e| Fehler::mit("einst-schreiben", json!({ "detail": e })))?; Ok(lesen())
}
/// Passwörter der Maschinen leer (Weitergabe)
pub fn fuer_export(s: &Settings) -> Settings { let mut c = s.clone(); for m in &mut c.machines { m.password.clear() } c }
pub fn exportieren(ziel: &Path) -> Result<(), Fehler> {
    let ziel = PathBuf::from(mit_endung(&ziel.to_string_lossy(), "json"));
    fuer_export(&settings::get()).save_to(&ziel).map_err(|e| Fehler::mit("einst-schreiben", json!({ "detail": e })))
}
/// Zusammenfassung einer Datei für die Rückfrage vor dem Übernehmen
pub fn pruefen(p: &Path) -> Result<Value, Fehler> {
    let mut s = Settings::load_from(p).map_err(|e| Fehler::mit("einst-import", json!({ "detail": e })))?; s.migrate_legacy();
    Ok(json!({ "path": p.to_string_lossy(), "targets": s.targets.len(), "machines": s.machines.len(), "projectsDir": s.projects_dir }))
}
/// Übernommene Einstellungen: Passwörter gleicher Maschinen bleiben, wenn die Datei keine enthält
pub fn uebernehmen_aus(aktuell: &Settings, neu: Settings) -> Settings {
    let mut n = neu; for m in &mut n.machines { if m.password.is_empty() { if let Some(a) = aktuell.machine(&m.id) { m.password = a.password.clone() } } } n
}
pub fn importieren(p: &Path) -> Result<Value, Fehler> {
    let mut s = Settings::load_from(p).map_err(|e| Fehler::mit("einst-import", json!({ "detail": e })))?; s.migrate_legacy();
    let neu = uebernehmen_aus(&settings::get(), s); settings::set(neu).map_err(|e| Fehler::mit("einst-schreiben", json!({ "detail": e })))?; Ok(lesen())
}
pub fn zuruecksetzen() -> Result<Value, Fehler> { settings::set(Settings::default()).map_err(|e| Fehler::mit("einst-schreiben", json!({ "detail": e })))?; Ok(lesen()) }

/// BASIC-Dialekt je installiertem Zielsystem (Anzeige der Toolchain-Seite)
pub fn dialekte(data: &Path) -> Value {
    json!(vc_ide::targets::installed_systems(Some(data)).into_iter().map(|s| json!({ "id": s.id, "name": s.name(), "dialect": vc_ide_model::recent::target_dialect(data, s.id).map(|x| vc_ide_model::recent::dialect_label(&x).to_string()) })).collect::<Vec<_>>())
}

/// Zielsysteme des Programms (installierte), mit Sorten der Emulatoren und Hardware; dazu die Namen
pub fn ziele_info(data: &Path) -> Value {
    json!({ "systems": vc_ide::targets::installed_systems(Some(data)).into_iter().map(|s| json!({
        "id": s.id, "name": s.name(), "group": s.group(), "cpu": s.cpu(),
        "kinds": s.kinds().iter().map(|k| json!({ "id": k.id, "label": k.label })).collect::<Vec<_>>(),
        "hardware": s.hardware().iter().map(|h| json!({ "id": h, "label": vc_ide::targets::hardware_label(h) })).collect::<Vec<_>>(),
    })).collect::<Vec<_>>() })
}

/// Änderung am Entwurf, die der Kern erledigt (Kennungen vergeben, Standard setzen …); Antwort: der neue Entwurf
pub fn op(entwurf: &Value, op: &Value) -> Result<Value, Fehler> {
    let mut s = aus_json(entwurf)?; s.migrate_legacy();
    let t = |k: &str| op[k].as_str().unwrap_or("");
    match t("kind") {
        "add_target" => s.add_target(t("system")),
        "remove_target" => s.remove_target(t("system")),
        "add_emulator" => { if let Some(x) = s.target_mut(t("system")) { x.add_emulator(t("ekind")); } }
        "remove_emulator" => { if let Some(x) = s.target_mut(t("system")) { x.remove_emulator(t("id")); } }
        "set_standard" => { if let Some(x) = s.target_mut(t("system")) { x.set_standard(t("id")); } }
        "add_machine" => { s.add_machine(t("system"), t("mkind")); }
        "remove_machine" => s.remove_machine(t("id")),
        _ => return Err(Fehler::neu("op-ungueltig")),
    }
    serde_json::to_value(&s).map_err(|e| Fehler::mit("einst-ungueltig", json!({ "detail": e.to_string() })))
}

/// Proben: Programm eines Emulators finden / automatisch suchen / Test-Start; Verbindung einer Maschine; serielle Schnittstellen. Im Testlauf (`gesperrt`) startet nichts.
pub fn probe(a: &Value) -> Result<Value, Fehler> {
    let emu = || serde_json::from_value::<vc_ide::targets::EmulatorCfg>(a["emulator"].clone()).map_err(|e| Fehler::mit("einst-ungueltig", json!({ "detail": e.to_string() })));
    let mach = || serde_json::from_value::<settings::Machine>(a["machine"].clone()).map_err(|e| Fehler::mit("einst-ungueltig", json!({ "detail": e.to_string() })));
    Ok(match a["kind"].as_str().unwrap_or("") {
        "presence" => { let p = vc_ide::platform::presence(&emu()?); json!({ "found": p.found, "auto": p.auto, "text": p.text }) }
        "discover" => json!(vc_ide::platform::discover(a["ekind"].as_str().unwrap_or(""))),
        "test" => { if crate::bau::gesperrt() { json!({ "ok": true, "text": vc_i18n::t!("gui-testlauf-kein-emulator-gestartet").to_string() }) } else { match vc_ide::platform::test_launch(&emu()?) { Ok(t) => json!({ "ok": true, "text": t }), Err(e) => json!({ "ok": false, "text": e }) } } }
        "hardware" => {
            if crate::bau::gesperrt() { json!({ "ok": true, "text": vc_i18n::t!("gui-testlauf-kein-emulator-gestartet").to_string() }) } else {
                let m = mach()?;
                let r = match a["link"].as_str() { Some(l) => vc_ide::platform::test_hardware_link(&m, if l == "eth" { vc_ide::mega65net::Link::Eth } else { vc_ide::mega65net::Link::Usb }, std::time::Duration::from_secs(6)), None => vc_ide::platform::test_hardware(&m, std::time::Duration::from_secs(6)).map(|(t, d)| format!("{t} ({} ms)", d.as_millis())) };
                match r { Ok(t) => json!({ "ok": true, "text": t }), Err(e) => json!({ "ok": false, "text": e }) }
            }
        }
        "ports" => json!(if crate::bau::gesperrt() { vec![] } else { vc_ide::platform::scan_serial_ports() }),
        _ => return Err(Fehler::neu("op-ungueltig")),
    })
}

/// ROM-Stand je System und Rolle (Status, Datei, Prüfsummen), durchsuchte Orte
pub fn roms_info(use_foreign: bool) -> Value {
    use vc_roms::{systems, Status};
    let mut env = vc_ide::roms::env(None, None); env.use_foreign = use_foreign; let store = vc_roms::RomStore::new(env);
    let st = |s: Status| match s { Status::Recognized => "recognized", Status::UnknownRevision => "unknown", Status::Replacement => "replacement", Status::Missing => "missing", Status::Unusable => "unusable" };
    let system_json = |sd: &vc_roms::systems::SystemDef| json!({
        "id": sd.id, "label": sd.label_l(), "missingRequired": store.missing_required(sd.id),
        "roles": sd.roles.iter().map(|rd| { let r = store.resolve(sd.id, rd.id); json!({ "id": rd.id, "label": rd.label_l(), "required": rd.required, "file": rd.file, "status": st(r.status), "statusLabel": r.status.label_l(), "path": r.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "size": r.size, "crc32": r.crc32.map(|c| format!("{c:08X}")), "sha1": r.sha1, "revision": r.label, "place": r.place_label, "expected": r.expected, "others": r.others.iter().map(|f| f.file_name()).collect::<Vec<_>>() }) }).collect::<Vec<_>>() });
    json!({ "systems": systems::SYSTEMS.iter().map(system_json).collect::<Vec<_>>(), "searched": store.searched().iter().map(|s| json!({ "stage": s.stage.label(), "path": s.path.to_string_lossy(), "exists": s.exists, "files": s.files })).collect::<Vec<_>>(), "useForeign": use_foreign })
}

fn rom_plan_items(paths: &[std::path::PathBuf], system: Option<&str>, foreign: bool) -> Vec<vc_roms::import::ImportItem> {
    let env = vc_ide::roms::env(None, None); let store = vc_roms::RomStore::new(env);
    if foreign { vc_roms::import::foreign_candidates(&store) } else { vc_roms::import::plan(&store, paths, system) }
}
/// ROM-Import (E12): Plan für gewählte Dateien/Ordner/Zip oder für ROMs anderer Programme; es wird nichts kopiert.
pub fn roms_import_plan(paths: &[std::path::PathBuf], system: Option<&str>, foreign: bool) -> Value {
    use vc_roms::import::Action;
    let items: Vec<Value> = rom_plan_items(paths, system, foreign).iter().enumerate().map(|(i, it)| {
        let (aktion, grund) = match &it.action { Action::Copy => ("copy", String::new()), Action::AlreadyPresent(p) => ("present", p.to_string_lossy().into_owned()), Action::Skip(w) => ("skip", w.clone()) };
        json!({ "i": i, "name": it.src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), "src": it.src.to_string_lossy(), "system": it.system, "role": it.role, "label": it.label, "size": it.size,
            "crc32": it.crc32.map(|c| format!("{c:08X}")), "action": aktion, "reason": grund, "dest": it.dest.as_ref().map(|d| d.to_string_lossy().into_owned()), "source": it.source_label })
    }).collect();
    json!({ "items": items })
}
/// Bestätigte Einträge (Nummern aus dem Plan) kopieren; der Plan wird dazu neu gebildet (gleiche Eingaben)
pub fn roms_import_run(paths: &[std::path::PathBuf], system: Option<&str>, foreign: bool, pick: &[usize]) -> Value {
    let items = rom_plan_items(paths, system, foreign); let mut ok = vec![]; let mut fehler = vec![];
    for &i in pick { let Some(it) = items.get(i) else { continue };
        match vc_roms::import::execute(it) { Ok(d) => ok.push(d.to_string_lossy().into_owned()), Err(e) => fehler.push(e) } }
    json!({ "copied": ok, "errors": fehler })
}
/// Festnageln: für Rolle eines Systems eine Datei wählen (leer = automatisch); gespeichert in roms.json des ROM-Ordners
pub fn roms_pin(system: &str, role: &str, file: Option<&str>) -> Result<(), String> {
    let env = vc_ide::roms::env(None, None);
    let root = env.install_dir().ok_or_else(|| vc_i18n::t!("romui-kein-rom-ordner-verfuegbar").to_string())?;
    vc_roms::store::pin(&root, system, role, file)
}

/// Chip-Fenster (E6): die Chips des Zielsystems mit ihren Registern (Adresse, Name, Zugriff, Erklärung) und dem Fenster, das beim Anhalten gelesen wird
pub fn chips_info(data: &Path, target: &str) -> Value {
    let Ok(t) = vc_target::load_target(data, target) else { return json!({ "chips": [] }) };
    let chips: Vec<Value> = t.chips.iter().filter_map(|id| vc_target::load_chip(data, id).ok()).filter(|c| !c.regs.is_empty()).map(|c| {
        let lo = c.regs.iter().map(|r| r.addr).min().unwrap_or(0); let hi = c.regs.iter().map(|r| r.addr).max().unwrap_or(0);
        json!({ "id": c.id, "name": c.name, "start": lo, "len": (hi - lo + 1).min(0x400), "regs": c.regs.iter().filter(|r| r.addr < 0x10000).map(|r| json!({ "name": r.name, "addr": r.addr, "access": r.access, "doc": r.doc })).collect::<Vec<_>>() })
    }).collect();
    json!({ "chips": chips })
}

/// Test-Attrappe (nur mit Prüfbrücke): `VC_TEST_EMU_PKG=<id>|<version>|<url>|<sha256>|<format>|<programm>` ersetzt die Pakete des Katalogs durch ein Paket aus einer lokalen Datei
#[cfg(feature = "testbridge")]
fn attrappe(id: &str) -> Option<(String, String, String, String, String)> {
    let v = std::env::var("VC_TEST_EMU_PKG").ok()?; let p: Vec<&str> = v.split('|').collect();
    if p.len() == 6 && p[0] == id { Some((p[1].into(), p[2].into(), p[3].into(), p[4].into(), p[5].into())) } else { None }
}
/// Portabel installierbare Emulator-Pakete (E12): Angaben für die Bestätigung vor dem Laden. Es wird nichts geladen.
pub fn emu_pakete(kind: &str) -> Value {
    use vc_ide::emulators as em;
    let Some(e) = em::for_kind(kind) else { return json!({ "known": false }) };
    let have: Vec<String> = vc_emu::install::versions(&e.id).into_iter().map(|(v, _)| v).collect();
    #[allow(unused_mut)]
    let mut pk: Vec<Value> = e.packages_here().iter().map(|p| json!({ "version": p.version, "sizeMb": p.size as f64 / 1_000_000.0, "host": p.url.split('/').nth(2).unwrap_or(""), "installed": have.contains(&p.version),
        "to": em::install_dir(&e.id).map(|d| d.join(&p.version).to_string_lossy().into_owned()) })).collect();
    #[cfg(feature = "testbridge")]
    if let Some((ver, url, _, _, _)) = attrappe(&e.id) { pk.push(json!({ "version": ver, "sizeMb": 0.0, "host": url.split('/').nth(2).filter(|h| !h.is_empty()).unwrap_or("lokal"), "installed": have.contains(&ver), "to": em::install_dir(&e.id).map(|d| d.join(&ver).to_string_lossy().into_owned()) })); }
    json!({ "known": true, "id": e.id, "name": e.name, "homepage": e.homepage, "license": e.license, "packages": pk, "manual": e.manual_hint(), "installed": have })
}
/// Lädt und installiert ein Paket (nur auf ausdrücklichen Aufruf nach der Bestätigung): Antwort = Pfad des Programms
pub fn emu_install(kind: &str, version: &str) -> Result<String, String> {
    let e = vc_ide::emulators::for_kind(kind).ok_or_else(|| vc_i18n::t!("emulators-emulator-id-ist-nicht-im-katalog", id = kind).to_string())?;
    #[cfg(feature = "testbridge")]
    if let Some((ver, url, sha, fmt, exe)) = attrappe(&e.id) { if ver == version {
        let pkg = vc_emu::install::Package { id: e.id.clone(), version: ver, url, sha256: sha, format: vc_emu::install::Format::parse(&fmt).ok_or("format")?, size: 0 };
        let dir = vc_emu::install::install(&pkg, &|d| vc_emu::install::find_in(d, &exe, 4).is_some(), &mut |_| {})?;
        return vc_emu::install::find_in(&dir, &exe, 4).map(|p| p.to_string_lossy().into_owned()).ok_or_else(|| "Programm fehlt nach dem Auspacken".to_string());
    } }
    vc_ide::emulators::install(&format!("{}@{}", e.id, version), &mut |_| {})
}

/// Anzeigetext eines Platzhalters des Argumenteditors: Katalog (`rest-platzhalter-<name>-label|hint`), sonst der Text aus dem Quelltext
fn platzhalter_text(name: &str, teil: &str, rueckfall: &str) -> String {
    let key = format!("rest-platzhalter-{}-{}", name.replace('_', "-"), teil);
    if vc_i18n::has_key(&key) { vc_i18n::translate(&key, &[]) } else { rueckfall.to_string() }
}

/// Parametereditor (E12.2b): Katalog zur Sorte (`asm` = Assembler), Platzhalter mit Beispielwerten, Prüfmeldungen und die fertige Kommandozeile
pub fn args_info(a: &Value) -> Value {
    use vc_ide::args as ar;
    let art = a["kind"].as_str().unwrap_or(""); let ziel = a["target"].as_str().filter(|t| !t.is_empty()).unwrap_or("c64");
    let kat = if art == "asm" { ar::catalog_for("vcc-asm") } else { ar::catalog_of_kind(art) };
    let opts: ar::Selection = a["opts"].as_object().map(|o| o.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect()).unwrap_or_default();
    let frei = a["free"].as_str().unwrap_or("").to_string(); let ctx = ar::ArgCtx::example(ziel);
    let programm = if art == "asm" { "vcc asm".to_string() } else { vc_ide::targets::kind_label(art).to_string().to_lowercase().replace(' ', "") };
    let (aufgeloest, _) = ar::preview(kat.as_deref(), &opts, &frei, &ctx);
    json!({
        "catalog": kat.as_deref().map(|k| serde_json::to_value(k).unwrap_or(Value::Null)),
        "placeholders": ar::PLACEHOLDERS.iter().map(|p| json!({ "name": p.name, "label": platzhalter_text(p.name, "label", p.label), "hint": platzhalter_text(p.name, "hint", p.hint), "example": ar::expand(&format!("{{{}}}", p.name), &ctx).ok() })).collect::<Vec<_>>(),
        "issues": ar::check(kat.as_deref(), &opts, &frei, Some(&ctx)).into_iter().map(|i| json!({ "level": match i.level { ar::Level::Error => "error", ar::Level::Warn => "warn", ar::Level::Info => "info" }, "text": i.text })).collect::<Vec<_>>(),
        "preview": ar::shell_line(&programm, &aufgeloest), "changed": kat.as_deref().map_or(0, |k| ar::changed_count(k, &opts)), "migratable": kat.as_deref().map_or(0, |k| ar::migratable(k, &opts, &frei)),
    })
}
/// Parameter aus dem freien Feld in die Auswahl übernehmen
pub fn args_migrieren(a: &Value) -> Value {
    use vc_ide::args as ar;
    let art = a["kind"].as_str().unwrap_or(""); let kat = if art == "asm" { ar::catalog_for("vcc-asm") } else { ar::catalog_of_kind(art) };
    let opts: ar::Selection = a["opts"].as_object().map(|o| o.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect()).unwrap_or_default();
    let frei = a["free"].as_str().unwrap_or("").to_string();
    match kat { Some(k) => { let (s, rest) = ar::migrate_free(&k, &opts, &frei); json!({ "opts": s, "free": rest }) } None => json!({ "opts": opts, "free": frei }) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn export_ohne_passwoerter_import_behaelt_bekannte() {
        let mut a = Settings::default(); a.machines.push(vc_ide::settings::Machine { id: "m1".into(), password: "pw1".into(), host: "h".into(), ..Default::default() });
        let e = fuer_export(&a); assert!(e.machines[0].password.is_empty() && e.machines[0].host == "h"); assert_eq!(a.machines[0].password, "pw1");
        assert_eq!(uebernehmen_aus(&a, e.clone()).machines[0].password, "pw1");
        let mut f = e; f.machines[0].id = "neu".into(); assert!(uebernehmen_aus(&a, f).machines[0].password.is_empty());
    }
    #[test] fn parameter_katalog_fuer_vice_und_assembler() {
        let v = args_info(&json!({ "kind": "x64sc", "opts": {}, "free": "" })); assert!(v["catalog"]["params"].as_array().is_some_and(|p| p.len() > 5), "{v}"); assert!(v["placeholders"].as_array().unwrap().len() >= 6);
        let a = args_info(&json!({ "kind": "asm", "opts": {}, "free": "-I x" })); assert!(a["catalog"].is_object() && a["preview"].as_str().unwrap().contains("-I"), "{a}");
    }
    #[test] fn ungueltiger_entwurf_wird_abgelehnt() { assert!(aus_json(&json!({ "language": 5 })).is_err()); assert!(aus_json(&json!({ "language": "de", "debug_log": true })).is_ok()); }
}
