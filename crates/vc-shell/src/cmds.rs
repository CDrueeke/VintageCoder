// SPDX-License-Identifier: GPL-3.0-or-later
//! Die Befehle der Oberfläche. Ein Befehl je Absicht, JSON rein und raus; fertig formatierte Texte gibt es nicht (Fehler: `{code, args}`).
//! `dispatch` ist der einzige Einstieg: Tauri (`vc`) und die Prüfbrücke rufen ihn gleich auf.
use crate::docstore::{DocStore, Fehler};
use crate::menu::{self, Kontext};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Wry};
use tauri::menu::MenuItem;
use vc_i18n::Lang;

/// Alle Befehle (Quelle der Wahrheit; `ui/src/lib/api.ts` führt dieselbe Liste, ein Test vergleicht)
pub const BEFEHLE: &[&str] = &[
    "i18n_ftl", "dom_info", "zahlen_werte", "lang_get", "lang_set", "commands_table", "registry_info", "features_get", "menu_context", "recent_list", "project_pick", "project_open", "project_close",
    "doc_list", "ping", "raster_apply", "raster_reset", "app_info", "app_quit",
    "file_read", "file_write", "file_create", "dir_create", "file_rename", "dir_rename", "file_delete", "syntax_color", "fold_ranges",
    "tree_move", "tree_undo", "tree_reload", "abs_path", "file_reveal", "file_external", "file_stats", "file_pick", "tree_can_trash",
    "doc_open", "doc_change", "doc_close", "lsp_hover", "lsp_complete", "lsp_definition", "lsp_outline", "lsp_fixes", "format_lines", "search_start", "search_cancel", "text_hover", "char_names",
    "build_start", "run_start", "run_stop", "build_info", "run_targets", "run_target_set", "build_cycles",
    "tests_start", "tests_stop", "tests_state",
    "projekt_info", "projekt_set", "basic_get", "basic_set", "turbo_state", "turbo_pick", "turbo_check", "turbo_provide", "turbo_reduce", "turbo_set",
    "recent_pin", "recent_remove", "recent_reveal", "examples_list", "tools_list", "vcprj_pick", "copy_suggest", "copy_check", "copy_plan", "copy_run",
    "disk_pick", "disk_open", "disk_view", "disk_apply", "disk_close", "disk_import", "disk_add_pick", "disk_save_pick", "disk_new",
    "palette_open", "kacheln_open", "karten_open", "bildschirm_open", "tracker_open", "audio_poll", "help_topics", "help_search", "help_open", "help_image", "wiz_stand", "wiz_start", "wiz_set", "wiz_system", "wiz_apply", "wiz_undo", "wiz_later", "wiz_news", "wiz_end", "info_about", "info_update_state", "info_update_check", "info_update_later", "info_update_skip", "info_feedback", "info_mail", "info_open", "settings_get", "args_info", "args_migrate", "settings_op", "ziele_info", "ziele_probe", "roms_info", "roms_open", "md_render", "md_edit", "md_url_open", "chips_info", "emu_pakete", "emu_install", "roms_pick", "roms_import_plan", "roms_import_run", "roms_pin", "settings_save", "settings_export", "settings_import_pick", "settings_import_apply", "settings_reset", "settings_reveal", "hex_list", "hex_page", "hex_find", "speicher_info", "debug_start", "debug_start_test", "debug_state", "debug_cmd", "debug_frame", "debug_stop", "song_quelle_pick", "song_import_run", "pixel_open", "pixel_view", "pixel_apply", "pixel_close", "export_info", "export_preview", "export_run", "png_pick", "media_profiles", "media_new",
    "drop_plan", "drop_copy", "include_resolve", "start_args", "newproj_info", "newproj_system", "newproj_preview", "newproj_create", "dir_pick",
    "session_get", "session_set", "restore_tabs_set", "scratch_get", "scratch_set", "scratch_pref",
    "basic_convert", "basic_info", "basic_codes",
];

#[derive(Default)]
pub struct Shell {
    pub docs: Mutex<DocStore>,
    pub raster: Mutex<crate::bench::Raster>,
    pub kontext: Mutex<Kontext>,
    pub eintraege: Mutex<Vec<(String, MenuItem<Wry>)>>,
    pub projekt: Mutex<Option<PathBuf>>,
    pub lsp: Mutex<Option<crate::lsp::Bruecke>>,
    pub suche: crate::suche::Suche,
    pub bau: crate::bau::Bau,
    pub tests: crate::testlauf::Tests,
    pub debug: Mutex<Option<crate::dbgsitzung::Sitzung>>,
    pub wizard: Mutex<Option<crate::wiz::Wiz>>,
}

/// Ist `'` in dieser Datei ein Kommentar? Nur im nativen BASIC mit einem Dialekt, der es nicht vorsieht (Commodore: `quoteComment` aus), ist es keiner (Warnung `B167`);
/// strukturiertes BASIC, Assembler und Reiter ohne Datei behalten die bisherige Regel.
fn apostroph_ist_kommentar(sh: &Shell, rel: &str, lang: &str) -> bool {
    let nativ = { let r = rel.to_ascii_lowercase(); r.ends_with(".bas") || r.ends_with(".lst") || r.ends_with(".tur") };
    if lang != "basic" || !nativ { return true; }
    let (Ok(p), Some(data)) = (projekt_laden(sh), crate::lsp::daten_ordner()) else { return true };
    let sys = system_von(&p, rel);
    let dialekt = p.pf.as_ref().and_then(|pf| pf.dialect_of(rel)).or_else(|| vc_target::load_target(&data, &sys).ok().and_then(|t| t.basic_dialect)).unwrap_or_else(|| "cbm_v2".into());
    vc_ide::basicinfo::quote_comment(&data, &dialekt)
}

/// Abgewählte Einträge der Änderungsliste aus den Argumenten
fn wiz_abgewaehlt(a: &Value) -> std::collections::BTreeSet<String> { a["deselected"].as_array().map(|x| x.iter().filter_map(|k| k.as_str().map(String::from)).collect()).unwrap_or_default() }

/// Ziel für die Hilfe: erstes System des Projekts, sonst c64
fn hilfe_ziel(sh: &Shell, a: &Value) -> String { a["target"].as_str().filter(|t| !t.is_empty()).map(|t| t.to_lowercase()).or_else(|| sh.bau.info.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|i| i.target.clone())).unwrap_or_else(|| "c64".into()) }

fn arg_str<'a>(a: &'a Value, k: &str) -> Result<&'a str, Fehler> { a[k].as_str().ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": k }))) }

/// Alle `.ftl`-Quellen einer Sprache, zu einem Text verbunden (Reihenfolge wie `CATALOGS`)
fn ftl(en: bool) -> String { vc_i18n::CATALOGS.iter().map(|(_, de, e)| if en { *e } else { *de }).collect::<Vec<_>>().join("\n") }

pub fn lang_code(l: Lang) -> &'static str { l.code() }

fn gespeicherte_wahl() -> vc_i18n::Choice { vc_i18n::Choice::from_code(&vc_ide::settings::get().language) }

/// Beim Start: gespeicherte Sprachwahl (oder `VC_LANG`) anwenden
pub fn sprache_beim_start() -> Lang { vc_i18n::apply(gespeicherte_wahl()) }

pub fn menue_neu(app: &AppHandle) -> Result<(), Fehler> {
    let sh = app.state::<Shell>();
    let k = *sh.kontext.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (m, items) = menu::bauen(app, k).map_err(|e| Fehler::mit("menue-fehler", json!({ "detail": e.to_string() })))?;
    app.set_menu(m).map_err(|e| Fehler::mit("menue-fehler", json!({ "detail": e.to_string() })))?;
    *sh.eintraege.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = items;
    Ok(())
}

fn baum(p: &vc_ide_model::project::Project) -> Value {
    // Verschachtelung aus den relativen Pfaden (`a/b/c.asm`); Ordner zuerst, dann Dateien, je alphabetisch ohne Groß/Klein
    #[derive(Default)] struct K { ordner: std::collections::BTreeMap<String, K>, dateien: Vec<(String, String, String)> }
    let mut wurzel = K::default();
    for d in &p.dirs { let mut k = &mut wurzel; for t in d.split('/') { k = k.ordner.entry(t.to_string()).or_default(); } }
    for e in &p.entries {
        let teile: Vec<&str> = e.rel.split('/').collect();
        let mut k = &mut wurzel;
        for t in &teile[..teile.len() - 1] { k = k.ordner.entry(t.to_string()).or_default(); }
        k.dateien.push((teile[teile.len() - 1].to_string(), e.rel.clone(), e.group.tag().to_string()));
    }
    fn wandle(k: &K, pfad: &str) -> Vec<Value> {
        let mut v: Vec<Value> = k.ordner.iter().map(|(n, s)| { let rel = if pfad.is_empty() { n.clone() } else { format!("{pfad}/{n}") }; json!({ "kind": "dir", "name": n, "rel": rel, "children": wandle(s, &rel) }) }).collect();
        let mut d = k.dateien.clone(); d.sort_by_key(|x| x.0.to_lowercase());
        v.extend(d.into_iter().map(|(n, rel, tag)| { let mut o = json!({ "kind": "file", "name": n, "rel": rel, "tag": tag }); if crate::turbo::ist_anwenderdiskette(&rel) { o["mark"] = json!("turbo-disk"); } o }));
        v
    }
    Value::Array(wandle(&wurzel, ""))
}

fn oeffne(app: &AppHandle, pfad: &str) -> Result<Value, Fehler> {
    let p = PathBuf::from(pfad);
    let (root, file) = if p.is_dir() { (p.clone(), vc_ide::projfile::find(&p).and_then(|f| f.file_name().map(|n| n.to_string_lossy().into_owned()))) }
        else if p.is_file() { (p.parent().map(Path::to_path_buf).unwrap_or_default(), p.file_name().map(|n| n.to_string_lossy().into_owned())) }
        else { return Err(Fehler::mit("projekt-fehlt", json!({ "path": pfad }))) };
    let pr = vc_ide_model::project::Project::load(root.clone(), file).map_err(|e| Fehler::mit("projekt-fehler", json!({ "detail": e })))?;
    let mut rec = vc_ide_model::recent::load(None);
    vc_ide_model::recent::touch(&mut rec, &root, &pr.name, None);
    let sh = app.state::<Shell>();
    *sh.projekt.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(root.clone());
    let ziel = pr.systems.first().map(|x| x.to_lowercase()).filter(|x| crate::lsp::daten_ordner().map_or(false, |d| d.join("targets").join(format!("{x}.target.json")).is_file()));
    *sh.lsp.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(crate::lsp::Bruecke::neu(&root, ziel.as_deref(), vc_i18n::language()));
    sh.kontext.lock().unwrap_or_else(std::sync::PoisonError::into_inner).projekt = true;
    let _ = menue_neu(app);
    let out = json!({ "root": root.to_string_lossy(), "pretty": vc_ide_model::recent::pretty(&root), "name": pr.name, "turbo": crate::turbo::ist_turbo_projekt(&root), "systems": pr.systems.iter().map(|x| vc_ide_model::recent::short_name(x)).collect::<Vec<_>>(), "tree": baum(&pr) });
    crate::events::emit(app, "project:changed", json!({ "name": pr.name }));
    Ok(out)
}

fn projekt_wurzel(sh: &Shell) -> Result<PathBuf, Fehler> { sh.projekt.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone().ok_or_else(|| Fehler::neu("kein-projekt")) }
fn export_quelle(sh: &Shell, a: &Value) -> Result<crate::docstore::ExportQuelle, Fehler> {
    sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(a["docId"].as_u64().unwrap_or(0))?.export_quelle().ok_or_else(|| Fehler::neu("op-ungueltig"))
}
fn projekt_laden(sh: &Shell) -> Result<vc_ide_model::project::Project, Fehler> {
    let root = projekt_wurzel(sh)?;
    let file = vc_ide::projfile::find(&root).and_then(|f| f.file_name().map(|n| n.to_string_lossy().into_owned()));
    vc_ide_model::project::Project::load(root, file).map_err(|e| Fehler::mit("projekt-fehler", json!({ "detail": e })))
}
/// Fehler der Dateiverwaltung (fertiger Text aus dem Katalog der Modellschicht) als Befehlsfehler
/// Fehler von Turbo BASIC XL (fertiger Text aus dem Katalog des Kerns) als Befehlsfehler
fn turbo_fehler(e: String) -> Fehler { Fehler::mit("turbo-fehler", json!({ "detail": e })) }
/// Datei löschen; scheitert der gewünschte Papierkorb, kommt der Code `fileops-papierkorb-fehlgeschlagen` (mit `rel`, `detail`) unverändert zur Oberfläche, sonst `datei-fehler`
fn datei_loeschen(p: &mut vc_ide_model::project::Project, rel: &str, papierkorb: bool) -> Result<bool, Fehler> {
    use vc_ide_model::fileops::DeleteFehler;
    vc_ide_model::fileops::delete_file_typed(p, rel, papierkorb).map_err(|e| match e {
        DeleteFehler::PapierkorbFehlgeschlagen { rel, text } => Fehler::mit("fileops-papierkorb-fehlgeschlagen", json!({ "rel": rel, "detail": text })),
        DeleteFehler::Anders(t) => datei_fehler(t),
    })
}
fn datei_fehler(e: String) -> Fehler { Fehler::mit("datei-fehler", json!({ "detail": e })) }

/// Startordner der Dateidialoge: das geöffnete Projekt, sonst der Benutzerordner (ohne Angabe beginnt der GTK-Dialog unter Linux in „Zuletzt verwendet“)
fn start_ordner(sh: &Shell) -> Option<PathBuf> { sh.projekt.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone().or_else(vc_core::userdirs::home_dir).filter(|p| p.is_dir()) }
fn mit_start<R: tauri::Runtime>(b: tauri_plugin_dialog::FileDialogBuilder<R>, sh: &Shell) -> tauri_plugin_dialog::FileDialogBuilder<R> { match start_ordner(sh) { Some(d) => b.set_directory(d), None => b } }

fn pick_file(app: &AppHandle, root: &Path) -> Option<PathBuf> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog().file().set_title(vc_i18n::t!("editor-datei-oeffnen")).set_directory(root).blocking_pick_file().and_then(|f| f.into_path().ok())
}

/// Zielsystem der Datei (aus der Projektdatei), sonst des ersten Ziels, sonst C64
fn system_von(p: &vc_ide_model::project::Project, rel: &str) -> String {
    let Some(pf) = &p.pf else { return "c64".into() };
    let id = pf.target_of(rel).or_else(|| pf.targets.first().map(|t| t.id.clone()));
    id.and_then(|id| pf.targets.iter().find(|t| t.id == id)).map(|t| if t.system.is_empty() { t.id.clone() } else { t.system.clone() }).unwrap_or_else(|| "c64".into())
}
/// Starten im Hintergrund: Ereignis `run:started` mit der Kommandozeile oder `run:failed` mit Text(en); frühes Ende des Emulators wird gemeldet
fn run_thread(app: &AppHandle, root: PathBuf, data: PathBuf, info: crate::bau::Info, ziel: String) {
    let h = app.clone();
    std::thread::spawn(move || {
        let sh = h.state::<Shell>();
        let _nacheinander = sh.bau.starten.lock().unwrap_or_else(|e| e.into_inner());   // zwei Starts gleichzeitig würden einen Emulator ohne Handle zurücklassen
        match crate::bau::starten(&root, &data, &info, &ziel) {
            Ok(line) => {
                crate::events::emit(&h, "run:started", json!({ "line": line, "target": info.target }));
                if !crate::bau::gesperrt() { let fehler = vc_ide::launch::wait_early_exit(std::time::Duration::from_millis(1500)); if !fehler.is_empty() { crate::events::emit(&h, "run:failed", json!({ "errors": fehler })); return; } }
                ueberwachen(&h);
            }
            Err(e) => crate::events::emit(&h, "run:failed", json!({ "errors": [e] })),
        }
    });
}

/// Bauen (und starten) eines Turbo-BASIC-XL-Projekts (Konzept 9): Diskette bauen und booten über `vc_ide::turbo::start`; ohne Prozess (Testlauf) nur bauen.
/// Meldet wie der normale Bau `build:started`/`build:done`; fehlt nur die Diskette, trägt `build:done` `turbo: "missing"` (Banner statt Fehler).
fn turbo_bauen_start(app: &AppHandle, sh: &Shell, root: PathBuf, data: PathBuf, rel: String, starten: bool) -> Result<Value, Fehler> {
    if !sh.bau.beginnen() { return Err(Fehler::neu("bau-laeuft")); }
    crate::events::emit(app, "build:started", json!({ "rel": rel }));
    let h = app.clone(); let r2 = rel.clone();
    std::thread::spawn(move || {
        let sh = h.state::<Shell>();
        let fehler = sh.bau.geschuetzt(|| {
        let _nacheinander = sh.bau.starten.lock().unwrap_or_else(|e| e.into_inner());
        // Ergebnis als Zeilen der Stufe „Turbo BASIC XL“; `Err((fehlt, text))`
        let res: Result<(Vec<String>, Option<PathBuf>, String), (bool, String)> = if starten && !crate::bau::gesperrt() {
            vc_ide::turbo::start(&root, &data).map(|s| (s.lines, Some(s.disk), s.command)).map_err(|m| (crate::turbo::diskette_fehlt(&root), m))
        } else {
            crate::turbo::bauen(&root, &data).map(|r| {
                let mut l: Vec<String> = vec![]; if let Some(x) = r["turbo"]["line"].as_str() { l.push(x.to_string()); }
                for n in r["turbo"]["notes"].as_array().into_iter().flatten().filter_map(|n| n.as_str()) { l.push(n.to_string()); }
                if starten { l.push(vc_i18n::t!("gui-testlauf-kein-emulator-gestartet").to_string()); }
                (l, r["turbo"]["file"].as_str().map(|f| root.join(f)), String::new())
            })
        };
        sh.bau.ende();
        match res {
            Ok((lines, disk, command)) => {
                let size = disk.as_ref().and_then(|d| std::fs::metadata(d).ok()).map_or(0, |m| m.len());
                let art = disk.as_ref().and_then(|d| d.strip_prefix(&root).ok().map(|x| x.to_string_lossy().replace('\\', "/"))).unwrap_or_default();
                let result = json!({ "ok": true, "file": r2, "target": "atari8", "artifact": art, "size": size, "diagnostics": [],
                    "log": { "ms": 0, "status": "ok", "stages": [{ "title": vc_i18n::t!("turboui-stufe-titel"), "status": "ok", "ms": 0, "lines": lines }] } });
                crate::events::emit(&h, "build:done", json!({ "ok": true, "rel": r2, "result": result, "then": if starten { "run" } else { "none" }, "turbo": "ok" }));
                if starten {
                    let zeile = if crate::bau::gesperrt() { vc_i18n::t!("gui-testlauf-kein-emulator-gestartet").to_string() } else { command };
                    crate::events::emit(&h, "run:started", json!({ "line": zeile, "target": "atari8" }));
                    if !crate::bau::gesperrt() {
                        let fehler = vc_ide::launch::wait_early_exit(std::time::Duration::from_millis(1500));
                        if !fehler.is_empty() { crate::events::emit(&h, "run:failed", json!({ "errors": fehler })); return; }
                        ueberwachen(&h);
                    }
                }
            }
            Err((fehlt, text)) => {
                let result = json!({ "ok": false, "file": r2, "diagnostics": [], "error": text });
                crate::events::emit(&h, "build:done", json!({ "ok": false, "rel": r2, "result": result, "then": "none", "turbo": if fehlt { "missing" } else { "error" } }));
            }
        }
        });
        if let Some(m) = fehler { crate::events::emit(&h, "build:done", json!({ "ok": false, "rel": r2, "result": { "ok": false, "file": r2, "diagnostics": [], "error": m }, "then": "none", "turbo": "error" })); }
    });
    Ok(json!({ "rel": rel }))
}

/// Erkennt das Ende des Emulators (vom Nutzer beendet, abgestürzt) und meldet `run:ended { code }`; ein Beenden über „Beenden“ meldet `run:stopped` und beendet die Überwachung still
fn ueberwachen(h: &AppHandle) {
    let sh = h.state::<Shell>();
    if sh.bau.wacht.swap(true, std::sync::atomic::Ordering::SeqCst) { return; }
    let h = h.clone();
    std::thread::spawn(move || {
        let t0 = std::time::Instant::now();
        // Xemu meldet sich erst nach dem Hochfahren an: bis dahin warten
        while !vc_ide::launch::is_running() && t0.elapsed() < std::time::Duration::from_secs(15) { std::thread::sleep(std::time::Duration::from_millis(200)); }
        loop {
            if let Some(code) = vc_ide::launch::poll_exit() { crate::events::emit(&h, "run:ended", json!({ "code": code })); break; }
            if !vc_ide::launch::is_running() { break; }
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
        h.state::<Shell>().bau.wacht.store(false, std::sync::atomic::Ordering::SeqCst);
    });
}
fn zahl(a: &Value, k: &str) -> usize { a[k].as_u64().unwrap_or(0) as usize }
/// Brücke zu den Sprachdiensten benutzen; ohne Projekt gibt es keine
fn with_lsp<T>(sh: &Shell, f: impl FnOnce(&mut crate::lsp::Bruecke) -> Result<T, Fehler>) -> Result<T, Fehler> {
    let mut g = sh.lsp.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(g.as_mut().ok_or_else(|| Fehler::neu("kein-projekt"))?)
}
fn senden(app: &AppHandle, ev: Vec<(&'static str, Value)>) { for (n, p) in ev { crate::events::emit(app, n, p); } }

pub fn dispatch(app: &AppHandle, cmd: &str, a: Value) -> Result<Value, Fehler> {
    let sh = app.state::<Shell>();
    match cmd {
        "i18n_ftl" => {
            let l = Lang::from_code(arg_str(&a, "lang")?).ok_or_else(|| Fehler::mit("sprache-unbekannt", json!({ "lang": a["lang"] })))?;
            // Deutsch ist immer dabei (Rückfall); Englisch nur auf Wunsch. Die Pseudo-Sprache `xx` entsteht in der Oberfläche aus dem Deutschen.
            Ok(json!({ "lang": l.code(), "de": ftl(false), "en": if l == Lang::En { Value::String(ftl(true)) } else { Value::Null } }))
        }
        // Diagnose für Prüfer: Mit VC_DOM_DUMP=<Datei> schreibt die Oberfläche beim Laden und bei jedem Sprachwechsel, was die Webview als Seitensprache sieht (lang, Meta, Systemsprache der Webview)
        "dom_info" => { if let Some(f) = std::env::var_os("VC_DOM_DUMP") { use std::io::Write; if let Ok(mut d) = std::fs::OpenOptions::new().create(true).append(true).open(f) { let _ = writeln!(d, "{}", a["info"]); } } Ok(Value::Null) }
        // Zahlenwerkzeuge: Text lesen und alle Schreibweisen liefern; mit `toggle` zuvor Bit n umschalten (Text kommt dann als Hexzahl zurück)
        "zahlen_werte" => {
            let bits = a["bits"].as_u64().unwrap_or(8) as u8; let mut text = arg_str(&a, "text")?.to_string();
            if let Some(n) = a["toggle"].as_u64() { text = vc_ide_model::zahlen::bit_text(&text, n as u8, bits); }
            let mut v = vc_ide_model::zahlen::alle(&text, bits); v["text"] = json!(text); Ok(v)
        }
        "lang_get" => Ok(json!({ "choice": gespeicherte_wahl().code(), "lang": vc_i18n::language().code(), "available": ["de", "en", "xx"] })),
        "lang_set" => {
            let c = arg_str(&a, "choice")?;
            let lang = if c == "xx" { vc_i18n::set_language(Lang::Xx); Lang::Xx } else {
                // gespeichert wird nur Automatisch, Deutsch oder Englisch; „xx“ ist nur zum Prüfen und bleibt unverändert in der Datei
                let ch = vc_i18n::Choice::from_code(c);
                let mut s = vc_ide::settings::get(); s.language = if ch == vc_i18n::Choice::Auto { String::new() } else { ch.code().into() };
                if std::env::var_os("VC_NO_SAVE").is_none() { let _ = vc_ide::settings::set(s); }
                vc_i18n::apply(ch)
            };
            if let Some(b) = sh.lsp.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_mut() { b.sprache(lang); }
            menue_neu(app)?;
            crate::events::emit(app, "lang:changed", json!({ "lang": lang.code() }));
            Ok(json!({ "lang": lang.code() }))
        }
        "commands_table" => {
            let mac = cfg!(target_os = "macos"); let lang = vc_i18n::language().code();
            Ok(Value::Array(crate::erweiterungen::zeilen_sichtbar(lang).iter().map(|z| json!({
                "id": z.id, "menu": z.menue, "submenu": z.unter, "keys": vc_commands::tasten_fuer(&z.id, &z.tasten, mac),
                "keysDisplay": vc_commands::fuer_anzeige(vc_commands::tasten_fuer(&z.id, &z.tasten, mac), mac),
                "needs": match z.braucht { vc_commands::Braucht::Nichts => "none", vc_commands::Braucht::Projekt => "project", vc_commands::Braucht::Editor => "editor" },
                "source": z.herkunft, "title": if z.dynamisch { Some(z.name.clone()) } else { None },
            })).collect()))
        }
        "registry_info" => Ok(crate::erweiterungen::info_json()),
        "features_get" => Ok(crate::erweiterungen::features_json()),
        "menu_context" => {
            let k = Kontext { projekt: a["project"].as_bool().unwrap_or(false), editor: a["editor"].as_bool().unwrap_or(false), turbo: a["turbo"].as_bool().unwrap_or(false) };
            *sh.kontext.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = k;
            let zeilen = vc_commands::dynamisch::zeilen(vc_i18n::language().code());
            for (id, it) in sh.eintraege.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter() {
                if let Some(z) = zeilen.iter().find(|z| z.id == *id) { let _ = it.set_enabled(k.erlaubt_id(&z.id, z.braucht)); }
            }
            Ok(Value::Null)
        }
        "recent_list" => Ok(crate::start::recent_json()),
        "recent_pin" => Ok(crate::start::pin(arg_str(&a, "path")?)),
        "recent_remove" => Ok(crate::start::remove(arg_str(&a, "path")?)),
        "recent_reveal" => { crate::dateien::im_ordner_zeigen(std::path::Path::new(arg_str(&a, "path")?))?; Ok(Value::Null) }
        "examples_list" => Ok(crate::start::beispiele(&crate::erweiterungen::registry())),
        "tools_list" => Ok(crate::lsp::daten_ordner().map_or(json!([]), |d| crate::start::werkzeuge(&d))),
        "vcprj_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_PICK_FILE") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("startui-projekt-oeffnen")).add_filter("VintageCoder", &["vcprj"]).blocking_pick_file();
            Ok(match p { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "disk_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_DISK") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("disk-oeffnen")).add_filter(vc_i18n::t!("disk-abbilder"), &["d64", "d71", "d81", "atr"]).blocking_pick_file();
            Ok(match p { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "disk_open" => {
            // `rel` liegt im Projekt, `path` ist absolut (Auswahldialog); schon geöffnete Abbilder werden wiederverwendet
            let p = match a["rel"].as_str() { Some(rel) => crate::dateien::absolut(&projekt_wurzel(&sh)?, rel)?, None => PathBuf::from(arg_str(&a, "path")?) };
            let dok = crate::disk::DiskDok::oeffnen(&p, crate::lsp::daten_ordner())?;
            let mut docs = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner); let view = dok.doc_view(); let id = docs.insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "disk", "view": view }))
        }
        "disk_add_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_ADD") { return Ok(json!(p.split(':').collect::<Vec<_>>())); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("diskstudio-dateien-auf-die-diskette-legen")).blocking_pick_files();
            Ok(match p { Some(v) => json!(v.iter().map(|x| x.to_string()).collect::<Vec<_>>()), None => json!([]) })
        }
        "disk_save_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_SAVEAS") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let mut b = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("diskstudio-image-speichern-unter")).add_filter(vc_i18n::t!("disk-abbilder"), &["d64", "d71", "d81", "atr"]);
            if let Some(n) = a["name"].as_str() { b = b.set_file_name(n); }
            Ok(match b.blocking_save_file() { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "disk_new" => {
            let fmt = vc_ide_model::diskdoc::format_aus_kennung(arg_str(&a, "format")?);
            let doc = vc_ide_model::diskdoc::DiskDoc::neu(fmt, arg_str(&a, "name")?, arg_str(&a, "id")?, crate::lsp::daten_ordner()).map_err(|e| Fehler::mit("disk-neu", json!({ "detail": e })))?;
            let dok = crate::disk::DiskDok { doc, uebernahme: None }; let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "disk", "view": view }))
        }
        "disk_import" => {
            // Vorgemerkte Dateien in den Projektordner übernehmen; die Meldungen (Erfolg, Text) kommen zurück, der Dateibaum wird neu gelesen
            let root = projekt_wurzel(&sh)?;
            let r = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(a["docId"].as_u64().unwrap_or(0))?.apply(&json!({ "kind": "import", "root": root.to_string_lossy() }))?; Ok(r)
        }
        "disk_view" => Ok(sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(a["docId"].as_u64().unwrap_or(0))?.view()),
        "disk_apply" => { let id = a["docId"].as_u64().unwrap_or(0); let r = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(id)?.apply(&a["op"])?; Ok(r) }
        "palette_open" => {
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?;
            let dok = crate::palette::PaletteDok::oeffnen(&root, rel)?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "palette", "view": view }))
        }
        "audio_poll" => Ok(crate::audio::abfragen()),
        "settings_get" => { let mut v = crate::einst::lesen(); if let Some(d) = crate::lsp::daten_ordner() { v["dialects"] = crate::einst::dialekte(&d) } Ok(v) }
        "help_topics" => Ok(crate::hilfe::themen()),
        "help_search" => { let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; Ok(crate::hilfe::suchen(&data, &hilfe_ziel(&sh, &a), arg_str(&a, "q")?)) }
        "help_open" => { let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; Ok(crate::hilfe::oeffnen(&data, &hilfe_ziel(&sh, &a), a["lang"].as_str().unwrap_or("asm"), arg_str(&a, "key")?)) }
        "help_image" => crate::hilfe::bild(arg_str(&a, "id")?, a["fetch"].as_bool().unwrap_or(false)),
        "wiz_stand" => Ok(crate::wiz::start_stand()),
        "wiz_start" => { let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; let w = crate::wiz::starten(&data, a["mode"].as_str().unwrap_or("quick"))?; let v = crate::wiz::json(&w, &Default::default()); *sh.wizard.lock().unwrap_or_else(|e| e.into_inner()) = Some(w); Ok(v) }
        "wiz_set" => { let mut g = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()); let w = g.as_mut().ok_or_else(|| Fehler::neu("wiz-keine-sitzung"))?; crate::wiz::setzen(w, &a)?; Ok(crate::wiz::json(w, &wiz_abgewaehlt(&a))) }
        "wiz_system" => { let mut g = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()); let w = g.as_mut().ok_or_else(|| Fehler::neu("wiz-keine-sitzung"))?; crate::wiz::system_schalten(w, arg_str(&a, "id")?, a["on"].as_bool().unwrap_or(true)); Ok(crate::wiz::json(w, &wiz_abgewaehlt(&a))) }
        "wiz_apply" => { let mut g = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()); let w = g.as_mut().ok_or_else(|| Fehler::neu("wiz-keine-sitzung"))?; let r = crate::wiz::uebernehmen(w, &wiz_abgewaehlt(&a))?; let mut v = crate::wiz::json(w, &wiz_abgewaehlt(&a)); v["report"] = r; Ok(v) }
        "wiz_undo" => { let mut g = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()); let w = g.as_mut().ok_or_else(|| Fehler::neu("wiz-keine-sitzung"))?; crate::wiz::rueckgaengig(w)?; Ok(crate::wiz::json(w, &wiz_abgewaehlt(&a))) }
        "wiz_later" => { if let Some(w) = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()).as_mut() { crate::wiz::spaeter(w, a["topic"].as_str().unwrap_or("")); } Ok(Value::Null) }
        "wiz_news" => crate::wiz::news_antwort(a["mode"].as_str().unwrap_or("seen")).map(|_| Value::Null).map_err(|e| Fehler::mit("wiz-fehler", json!({ "detail": e }))),
        "wiz_end" => { if let Some(mut w) = sh.wizard.lock().unwrap_or_else(|e| e.into_inner()).take() { crate::wiz::beenden(&mut w); } Ok(Value::Null) }
        "info_about" => Ok(crate::info::ueber()),
        "info_update_state" => Ok(crate::info::update_stand()),
        "info_update_check" => Ok(crate::info::update_pruefen(a["auto"].as_bool().unwrap_or(false))),
        "info_update_later" => Ok(crate::info::update_spaeter()),
        "info_update_skip" => crate::info::update_ueberspringen(arg_str(&a, "version")?),
        "info_feedback" => Ok(crate::info::feedback(a["system"].as_str())),
        "info_mail" => { crate::info::mail_oeffnen(arg_str(&a, "url")?)?; Ok(Value::Null) }
        "info_open" => { let w = arg_str(&a, "which")?; if w == "credit" { crate::info::danksagung_oeffnen(a["id"].as_str().unwrap_or(""), a["index"].as_u64().unwrap_or(0) as usize)?; } else { crate::info::adresse_oeffnen(w, a["version"].as_str())?; } Ok(Value::Null) }
        "args_info" => Ok(crate::einst::args_info(&a)),
        "args_migrate" => Ok(crate::einst::args_migrieren(&a)),
        "settings_op" => Ok(json!({ "settings": crate::einst::op(&a["settings"], &a["op"])? })),
        "ziele_info" => { let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; Ok(crate::einst::ziele_info(&data)) }
        "ziele_probe" => crate::einst::probe(&a),
        "roms_info" => Ok(crate::einst::roms_info(a["useForeign"].as_bool().unwrap_or_else(|| vc_ide::settings::get().roms.use_foreign))),
        "md_render" => { let root = projekt_wurzel(&sh)?; Ok(crate::markdown::render(&root, a["rel"].as_str().unwrap_or(""), a["text"].as_str().unwrap_or(""), a["from"].as_u64().unwrap_or(0) as usize, a["count"].as_u64().unwrap_or(200) as usize)) }
        "md_url_open" => crate::markdown::adresse_oeffnen(arg_str(&a, "url")?).map(|_| Value::Null),
        "md_edit" => Ok(crate::markdown::bearbeiten(arg_str(&a, "op")?, a["text"].as_str().unwrap_or(""), a["a"].as_u64().unwrap_or(0) as usize, a["b"].as_u64().unwrap_or(0) as usize, a["arg"].as_str().unwrap_or(""))),
        "chips_info" => { let d = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; Ok(crate::einst::chips_info(&d, arg_str(&a, "target")?)) }
        "emu_pakete" => Ok(crate::einst::emu_pakete(arg_str(&a, "kind")?)),
        "emu_install" => crate::einst::emu_install(arg_str(&a, "kind")?, arg_str(&a, "version")?).map(|p| json!(p)).map_err(|e| Fehler::mit("emu-install-fehler", json!({ "detail": e }))),
        "roms_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_ROM_PICK") { return Ok(json!(p.split('|').filter(|x| !x.is_empty()).collect::<Vec<_>>())); }
            use tauri_plugin_dialog::DialogExt;
            let v = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("roms-datei-waehlen")).blocking_pick_files().unwrap_or_default();
            Ok(json!(v.into_iter().filter_map(|f| f.into_path().ok()).map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>()))
        }
        "roms_import_plan" | "roms_import_run" => {
            let paths: Vec<PathBuf> = a["paths"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(PathBuf::from)).collect()).unwrap_or_default();
            let sys = a["system"].as_str().filter(|x| !x.is_empty()); let fremd = a["foreign"].as_bool().unwrap_or(false);
            if cmd == "roms_import_plan" { Ok(crate::einst::roms_import_plan(&paths, sys, fremd)) }
            else { let pick: Vec<usize> = a["pick"].as_array().map(|v| v.iter().filter_map(|x| x.as_u64().map(|n| n as usize)).collect()).unwrap_or_default(); Ok(crate::einst::roms_import_run(&paths, sys, fremd, &pick)) }
        }
        "roms_pin" => crate::einst::roms_pin(arg_str(&a, "system")?, arg_str(&a, "role")?, a["file"].as_str().filter(|x| !x.is_empty())).map(|_| Value::Null).map_err(|e| Fehler::mit("roms-fehler", json!({ "detail": e }))),
        "roms_open" => { let env = vc_ide::roms::env(None, None); if let Some(d) = env.vc_roms.clone() { let _ = std::fs::create_dir_all(&d); crate::dateien::im_ordner_zeigen(&d)?; } Ok(Value::Null) }
        "settings_save" => crate::einst::speichern(&a["settings"]),
        "settings_export" => {
            #[cfg(feature = "testbridge")]
            let gewaehlt = std::env::var("VC_TEST_KONFIG_OUT").ok().map(PathBuf::from);
            #[cfg(not(feature = "testbridge"))]
            let gewaehlt: Option<PathBuf> = None;
            let gewaehlt = match gewaehlt { Some(p) => Some(p), None => { use tauri_plugin_dialog::DialogExt; mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("konfig-konfiguration-exportieren")).set_file_name("vintagecoder-einstellungen.json").add_filter("JSON", &["json"]).blocking_save_file().and_then(|f| f.into_path().ok()) } };
            match gewaehlt { None => Ok(Value::Null), Some(p) => { crate::einst::exportieren(&p)?; Ok(json!({ "path": p.to_string_lossy() })) } }
        }
        "settings_import_pick" => {
            #[cfg(feature = "testbridge")]
            let gewaehlt = std::env::var("VC_TEST_KONFIG_IN").ok().map(PathBuf::from);
            #[cfg(not(feature = "testbridge"))]
            let gewaehlt: Option<PathBuf> = None;
            let gewaehlt = match gewaehlt { Some(p) => Some(p), None => { use tauri_plugin_dialog::DialogExt; mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("konfig-konfiguration-importieren")).add_filter("JSON", &["json"]).blocking_pick_file().and_then(|f| f.into_path().ok()) } };
            match gewaehlt { None => Ok(Value::Null), Some(p) => crate::einst::pruefen(&p) }
        }
        "settings_import_apply" => crate::einst::importieren(Path::new(arg_str(&a, "path")?)),
        "settings_reset" => crate::einst::zuruecksetzen(),
        "settings_reveal" => { if let Some(p) = vc_ide::settings::file() { crate::dateien::im_ordner_zeigen(&p)?; } Ok(Value::Null) }
        "hex_list" => { let root = projekt_wurzel(&sh)?; let p = projekt_laden(&sh)?; let mut rels: Vec<String> = p.entries.iter().map(|e| e.rel.clone()).collect(); for b in crate::daten::build_ergebnisse(&root) { if !rels.contains(&b) { rels.push(b); } } Ok(json!(crate::daten::hex_dateien(&root, &rels))) }
        "hex_page" => crate::daten::hex_seite(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?, a["page"].as_u64().unwrap_or(0) as usize, a["width"].as_u64().unwrap_or(16) as usize),
        "hex_find" => crate::daten::hex_suche(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?, arg_str(&a, "query")?, a["from"].as_u64().unwrap_or(0) as usize),
        "speicher_info" => { let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; let info = sh.bau.info.lock().unwrap_or_else(|e| e.into_inner()).clone(); Ok(crate::daten::speicher(&data, info.as_ref(), a["preset"].as_str(), a["page"].as_u64().map(|x| x as u8))) }
        "debug_start_test" => { let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?); crate::dbgsitzung::starten_test(&sh, root, data, &a) }
        "debug_start" => { let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?); crate::dbgsitzung::starten(&sh, root, data, &a) }
        "debug_state" => Ok(crate::dbgsitzung::zustand(&sh)),
        "debug_cmd" => crate::dbgsitzung::befehl(&sh, &a),
        "debug_frame" => Ok(crate::dbgsitzung::bild(&sh, a["seq"].as_u64().unwrap_or(0))),
        "debug_stop" => Ok(json!(crate::dbgsitzung::beenden(&sh))),
        "song_quelle_pick" => {
            // Quelle für „Song aus anderem System übernehmen“ bzw. „RMT-Modul importieren“ wählen; Antwort null (abgebrochen) oder die Beschreibung der Quelle
            let rmt = a["rmt"].as_bool().unwrap_or(false);
            #[cfg(feature = "testbridge")]
            let gewaehlt = std::env::var("VC_TEST_SONG_SRC").ok().map(PathBuf::from);
            #[cfg(not(feature = "testbridge"))]
            let gewaehlt: Option<PathBuf> = None;
            let gewaehlt = match gewaehlt { Some(p) => Some(p), None => {
                use tauri_plugin_dialog::DialogExt;
                let d = mit_start(app.dialog().file(), &sh).set_title(if rmt { vc_i18n::t!("songstudio-rmt-modul-importieren") } else { vc_i18n::t!("songstudio-song-aus-anderem-system-uebernehmen") });
                let d = if rmt { d.add_filter("RMT", &["rmt"]) } else { d.add_filter(vc_i18n::t!("tracker-quelle-filter"), &["sid", "sng", "vcsong", "vcplay"]) };
                d.blocking_pick_file().and_then(|f| f.into_path().ok()) } };
            let Some(p) = gewaehlt else { return Ok(Value::Null) };
            let q = if rmt { Ok(vc_ide_model::songimport::Quelle { art: vc_i18n::t!("songstudio-rmt-modul-raster-music-tracker").into(), name: vc_media::naming::stem(&p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()).replace(' ', "_"), sid: None }) } else { vc_ide_model::songimport::quelle_pruefen(&p) };
            Ok(match q {
                Ok(q) => json!({ "path": p.to_string_lossy(), "file": p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), "art": q.art, "name": q.name, "sid": q.sid.map(|(d, n, start)| json!({ "info": d, "songs": n, "start": start })) }),
                Err(e) => json!({ "error": e, "file": p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default() }),
            })
        }
        "song_import_run" => {
            let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?);
            let (path, name, ordner) = (PathBuf::from(arg_str(&a, "path")?), arg_str(&a, "name")?, arg_str(&a, "ordner")?);
            let r = if a["rmt"].as_bool().unwrap_or(false) { vc_ide_model::songimport::rmt_importieren(&root, &data, &path, name, ordner) }
                else { vc_ide_model::songimport::konvertieren(&root, &data, &path, arg_str(&a, "ziel")?, name, ordner, a["subtune"].as_u64().unwrap_or(0) as u16, a["sekunden"].as_u64().unwrap_or(30) as u32) };
            let (rel, warnungen) = r.map_err(|e| Fehler::mit("song-import-fehler", json!({ "detail": e })))?;
            Ok(json!({ "rel": rel, "warnings": warnungen }))
        }
        "tracker_open" => {
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let dok = crate::tracker::TrackerDok::oeffnen(&root, rel, &data)?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(|e| e.into_inner()).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "song", "view": view }))
        }
        "bildschirm_open" => {
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            if let Ok(m) = std::fs::read_to_string(root.join(rel)).map_err(|_| ()).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|_| ())) {
                if m["kind"] == "screen" && !vc_ide_model::screendoc::ist_textbildschirm(&m) { return Err(Fehler::mit("bildschirm-kachel", json!({ "rel": rel }))); }   // Kachel-Bildschirm: die Oberfläche erklärt und bietet „Als Text öffnen“
            }
            let dok = crate::bildschirm::BildschirmDok::oeffnen(&root, rel, &data)?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(|e| e.into_inner()).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "screen", "view": view }))
        }
        "karten_open" => {
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let dok = crate::karten::KartenDok::oeffnen(&root, rel, &data)?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(|e| e.into_inner()).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "bank", "view": view }))
        }
        "kacheln_open" => {
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let dok = crate::kacheln::KachelDok::oeffnen(&root, rel, &data)?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(|e| e.into_inner()).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "tileset", "view": view }))
        }
        "pixel_open" => {
            // `rel` liegt im Projekt; die Datei wird jedes Mal neu gelesen (der Aufrufer schließt ein altes Dokument)
            let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?;
            let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let dok = crate::pixel::PixelDok::oeffnen(&root, rel, &data, Some(&root))?;
            let view = dok.doc_view(); let id = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(Box::new(dok));
            Ok(json!({ "docId": id, "kind": "pixel", "view": view }))
        }
        "media_profiles" => {
            // Profile, aus denen sich im Projekt eine neue Mediendatei anlegen lässt (nur Systeme des Projekts)
            let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?);
            let kinds: Vec<String> = a["kinds"].as_array().map(|k| k.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            let profiles = vc_media::load_profiles(&data).map_err(|e| Fehler::mit("media-fehler", json!({ "detail": e })))?; let erlaubt = vc_media::project::project_systems(&root, &data);
            Ok(json!(profiles.iter().filter(|p| (kinds.is_empty() || kinds.contains(&p.kind)) && (erlaubt.is_empty() || erlaubt.contains(&p.target)))
                .map(|p| json!({ "id": p.id, "name": p.name, "kind": p.kind, "target": p.target })).collect::<Vec<_>>()))
        }
        "media_new" => {
            let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?);
            let r = vc_media::project::create(&root, &data, arg_str(&a, "name")?, arg_str(&a, "profile")?, a["dir"].as_str().unwrap_or("")).map_err(|e| Fehler::mit("media-fehler", json!({ "detail": e })))?;
            Ok(json!({ "path": r["path"] }))
        }
        "png_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_PNG") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("sprite-png-laden")).add_filter("PNG", &["png"]).blocking_pick_file();
            Ok(match p { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "export_info" => { let q = export_quelle(&sh, &a)?; Ok(crate::export::info(&q)) }
        "export_preview" => {
            let q = export_quelle(&sh, &a)?; let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?);
            Ok(crate::export::vorschau(&root, &data, &q, arg_str(&a, "format")?, arg_str(&a, "name")?, a["addr"].as_str().unwrap_or(""), a["routine"].as_bool().unwrap_or(false)))
        }
        "export_run" => {
            let q = export_quelle(&sh, &a)?; let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?);
            let (name, addr) = (arg_str(&a, "name")?, a["addr"].as_str().unwrap_or(""));
            let id = a["docId"].as_u64().unwrap_or(0);
            match arg_str(&a, "action")? {
                "copy" => Ok(json!({ "text": crate::export::kopiertext(&root, &data, &q, a["targetBasic"].as_bool().unwrap_or(false), name, addr, a["routine"].as_bool().unwrap_or(false))? })),
                "save" => { sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(id)?.save()?; Ok(json!({ "files": crate::export::speichern(&root, &data, &q.rel, arg_str(&a, "format")?, name, addr)? })) }
                "include" => { sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(id)?.save()?; let (line, files) = crate::export::einbinden(&root, &data, &q.rel, arg_str(&a, "target")?, a["text"].as_str().unwrap_or(""), name)?; Ok(json!({ "line": line, "files": files })) }
                _ => Err(Fehler::neu("op-ungueltig")),
            }
        }
        "pixel_view" => Ok(sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(a["docId"].as_u64().unwrap_or(0))?.view()),
        "pixel_apply" => { let id = a["docId"].as_u64().unwrap_or(0); let r = sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(id)?.apply(&a["op"])?; Ok(r) }
        "pixel_close" => Ok(json!(sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).close(a["docId"].as_u64().unwrap_or(0)))),
        "disk_close" => Ok(json!(sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).close(a["docId"].as_u64().unwrap_or(0)))),
        "copy_suggest" => Ok(json!(crate::start::kopie_vorschlag(Path::new(arg_str(&a, "src")?), Path::new(arg_str(&a, "dir")?)))),
        "copy_check" => Ok(crate::start::kopie_problem(Path::new(arg_str(&a, "src")?), Path::new(arg_str(&a, "dir")?), arg_str(&a, "name")?).map_or(Value::Null, |p| json!(p))),
        "copy_plan" => { let o = vc_ide::copyproject::Options { with_build: a["withBuild"].as_bool().unwrap_or(false), with_git: a["withGit"].as_bool().unwrap_or(false) }; Ok(match vc_ide::copyproject::plan(Path::new(arg_str(&a, "src")?), &o) { Ok((f, b)) => json!({ "files": f, "bytes": b }), Err(_) => Value::Null }) }
        "copy_run" => {
            let o = vc_ide::copyproject::Options { with_build: a["withBuild"].as_bool().unwrap_or(false), with_git: a["withGit"].as_bool().unwrap_or(false) };
            let r = vc_ide::copyproject::copy_project(Path::new(arg_str(&a, "src")?), Path::new(arg_str(&a, "dir")?), arg_str(&a, "name")?.trim(), &o).map_err(|e| Fehler::mit("kopieren-fehler", json!({ "detail": e })))?;
            if vc_i18n::language() == vc_i18n::Lang::En { crate::start::spiegel_ueberlagern(Path::new(arg_str(&a, "src")?), &r.new_root); }
            if let Some(sys) = a["system"].as_str().filter(|x| !x.is_empty()) { crate::projekt::normalziel_setzen(&r.new_root, &crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?, sys).map_err(|e| Fehler::mit("kopieren-fehler", json!({ "detail": e })))?; }   // Englisch: Dateien des englischen Spiegels über die Kopie legen
            Ok(json!({ "root": r.new_root.to_string_lossy(), "files": r.files, "bytes": r.bytes, "skipped": r.skipped, "turboDisk": crate::turbo::enthaelt_diskette(&r.new_root) }))
        }
        "include_resolve" => { let root = projekt_wurzel(&sh)?; Ok(crate::dateien::include_aufloesen(&root, arg_str(&a, "from")?, arg_str(&a, "path")?)) }
        "project_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_PICK") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("startui-projekt-oeffnen")).blocking_pick_folder();
            Ok(match p { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "start_args" => Ok(crate::startargs::abholen()),
        "drop_plan" => { let paths: Vec<String> = a["paths"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(); let root = sh.projekt.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(); Ok(crate::ablegen::plan(root.as_deref(), &paths)) }
        "drop_copy" => { let root = projekt_wurzel(&sh)?; Ok(json!(crate::ablegen::kopieren(&root, Path::new(&arg_str(&a, "path")?))?)) }
        "project_open" => oeffne(app, arg_str(&a, "path")?),
        "project_close" => {
            *sh.projekt.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            *sh.lsp.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            sh.kontext.lock().unwrap_or_else(std::sync::PoisonError::into_inner).projekt = false;
            let _ = menue_neu(app);
            crate::events::emit(app, "project:changed", Value::Null);
            Ok(Value::Null)
        }
        "doc_list" => Ok(Value::Array(sh.docs.lock().unwrap_or_else(std::sync::PoisonError::into_inner).list())),
        "ping" => Ok(a),
        "raster_apply" => { let z = a["cells"].as_array().cloned().unwrap_or_default(); Ok(sh.raster.lock().unwrap_or_else(std::sync::PoisonError::into_inner).strich(&z)) }
        "raster_reset" => { sh.raster.lock().unwrap_or_else(std::sync::PoisonError::into_inner).leeren(); Ok(Value::Null) }
        "app_info" => Ok(json!({ "pid": std::process::id(), "os": std::env::consts::OS, "arch": std::env::consts::ARCH, "version": env!("CARGO_PKG_VERSION"),
            "testbridge": cfg!(feature = "testbridge"), "debug": cfg!(debug_assertions) })),
        #[cfg(feature = "selftest")]
        "selftest_config" => Ok(match std::env::var("VC_SELFTEST_OUT") { Ok(p) => json!({ "out": p, "project": std::env::var("VC_SELFTEST_PROJECT").ok(), "ms_since_start": crate::START.get().map(|t| t.elapsed().as_secs_f64() * 1000.0) }), Err(_) => Value::Null }),
        #[cfg(feature = "selftest")]
        "selftest_report" => {
            let p = std::env::var("VC_SELFTEST_OUT").map_err(|_| Fehler::neu("selftest-aus"))?;
            let mut a = a; a["rust_ms_since_start"] = json!(crate::START.get().map(|t| t.elapsed().as_secs_f64() * 1000.0));
            std::fs::write(p, serde_json::to_string_pretty(&a).unwrap_or_default()).map_err(|e| Fehler::mit("selftest-schreiben", json!({ "detail": e.to_string() })))?;
            app.exit(0); Ok(Value::Null)
        }
        "file_read" => { let root = projekt_wurzel(&sh)?; let rel = arg_str(&a, "rel")?; if let Ok(p) = crate::dateien::absolut(&root, rel) { if p.is_file() && crate::dateien::ist_nichttext(&p) { return Err(Fehler::mit("datei-nichttext", json!({ "rel": rel }))); } } crate::text::lesen(&root, rel) }
        "file_write" => { crate::text::schreiben(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?, arg_str(&a, "text")?, a["eol"].as_str().unwrap_or("lf"))?; Ok(Value::Null) }
        "file_create" => { let root = projekt_wurzel(&sh)?; let rel = vc_ide_model::fileops::create_file(&root, arg_str(&a, "rel")?).map_err(datei_fehler)?; Ok(json!({ "rel": rel, "tree": baum(&projekt_laden(&sh)?) })) }
        "dir_create" => { let root = projekt_wurzel(&sh)?; let rel = vc_ide_model::fileops::create_dir(&root, arg_str(&a, "rel")?).map_err(datei_fehler)?; Ok(json!({ "rel": rel, "tree": baum(&projekt_laden(&sh)?) })) }
        "file_rename" | "dir_rename" => {
            let (old, new) = (arg_str(&a, "old")?, arg_str(&a, "new")?); let root = projekt_wurzel(&sh)?;
            match crate::dateien::umbenennen(&root, old, new)? {
                Some((rel, o)) => Ok(json!({ "rel": rel, "moved": o["moved"], "rewritten": o["rewritten"], "tree": baum(&projekt_laden(&sh)?) })),
                None if cmd == "file_rename" => { let mut p = projekt_laden(&sh)?; let rel = vc_ide_model::fileops::rename_file(&mut p, old, new).map_err(datei_fehler)?; Ok(json!({ "rel": rel, "moved": [[old, rel]], "rewritten": [], "tree": baum(&p) })) }
                None => { let mut p = projekt_laden(&sh)?; let (rel, paare) = vc_ide_model::fileops::rename_dir(&mut p, old, new).map_err(datei_fehler)?; Ok(json!({ "rel": rel, "moved": paare, "rewritten": [], "tree": baum(&p) })) }
            }
        }
        "file_delete" => { let mut p = projekt_laden(&sh)?; let korb = datei_loeschen(&mut p, arg_str(&a, "rel")?, a["trash"].as_bool().unwrap_or(true))?; Ok(json!({ "trashed": korb, "tree": baum(&p) })) }
        "tree_move" => { let root = projekt_wurzel(&sh)?; let mut o = crate::dateien::verschieben(&root, arg_str(&a, "from")?, arg_str(&a, "folder")?)?; o["tree"] = baum(&projekt_laden(&sh)?); Ok(o) }
        "tree_undo" => { let root = projekt_wurzel(&sh)?; let mut o = crate::dateien::rueckgaengig(&root)?; o["tree"] = baum(&projekt_laden(&sh)?); Ok(o) }
        "tree_reload" => Ok(json!({ "tree": baum(&projekt_laden(&sh)?) })),
        "abs_path" => Ok(json!(crate::dateien::absolut(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?)?.to_string_lossy())),
        "file_reveal" => { crate::dateien::im_ordner_zeigen(&crate::dateien::absolut(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?)?)?; Ok(Value::Null) }
        "file_external" => {
            let p = crate::dateien::absolut(&projekt_wurzel(&sh)?, arg_str(&a, "rel")?)?;
            // ausführbare Dateien nur nach ausdrücklicher Bestätigung (`confirmed: true`), sonst Fehler `extern-ausfuehrbar` für die Rückfrage der Oberfläche
            if crate::dateien::ist_ausfuehrbar(&p) && a["confirmed"].as_bool() != Some(true) { return Err(Fehler::mit("extern-ausfuehrbar", json!({ "rel": arg_str(&a, "rel")? }))); }
            crate::dateien::standardprogramm(&p)?; Ok(Value::Null)
        }
        "file_stats" => { let rels: Vec<String> = a["rels"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(); Ok(crate::dateien::staende(&projekt_wurzel(&sh)?, &rels)) }
        "tree_can_trash" => Ok(json!(vc_ide_model::fileops::can_trash())),
        "file_pick" => {
            // Datei wählen (⌘O): Antwort `{ rel }` (im Projekt) oder `{ outside: Pfad }` oder null (abgebrochen)
            let root = projekt_wurzel(&sh)?;
            #[cfg(feature = "testbridge")]
            let gewaehlt = std::env::var("VC_TEST_PICK_FILE").ok().map(PathBuf::from).or_else(|| pick_file(app, &root));
            #[cfg(not(feature = "testbridge"))]
            let gewaehlt = pick_file(app, &root);
            Ok(match gewaehlt { None => Value::Null, Some(p) => match p.strip_prefix(&root) { Ok(r) => json!({ "rel": r.to_string_lossy().replace('\\', "/") }), Err(_) => json!({ "outside": p.to_string_lossy() }) } })
        }
        "syntax_color" => {
            // nummeriertes BASIC: der Dialekt des Ziels der Datei bestimmt, wo Schlüsselwörter ohne Leerzeichen stehen (`fori=1to10`)
            if arg_str(&a, "lang")? == "basic" {
                if let (Ok(p), Some(data)) = (projekt_laden(&sh), crate::lsp::daten_ordner()) {
                    let sys = system_von(&p, a["rel"].as_str().unwrap_or(""));
                    let dialekt = vc_target::load_target(&data, &sys).ok().and_then(|t| t.basic_dialect).unwrap_or_else(|| "cbm_v2".into());
                    vc_ide_model::editor_names::select_name_dialect(&data, &dialekt);
                }
            }
            let quote = apostroph_ist_kommentar(&sh, a["rel"].as_str().unwrap_or(""), arg_str(&a, "lang")?);
            Ok(crate::text::faerbung_q(arg_str(&a, "text")?, arg_str(&a, "lang")?, quote))
        }
        "fold_ranges" => { let quote = apostroph_ist_kommentar(&sh, a["rel"].as_str().unwrap_or(""), arg_str(&a, "lang")?); Ok(crate::text::faltung(arg_str(&a, "text")?, arg_str(&a, "lang")?, quote)) }
        "doc_open" => { let ev = with_lsp(&sh, |b| Ok(b.oeffnen(arg_str(&a, "rel")?, arg_str(&a, "text")?)))?; senden(app, ev); Ok(Value::Null) }
        "doc_change" => {
            let ch = a["changes"].as_array().cloned().unwrap_or_default();
            let ev = with_lsp(&sh, |b| b.aendern(arg_str(&a, "rel")?, a["version"].as_u64().unwrap_or(0), &ch))?; senden(app, ev); Ok(Value::Null)
        }
        "doc_close" => { let ev = with_lsp(&sh, |b| Ok(b.schliessen(arg_str(&a, "rel")?)))?; senden(app, ev); Ok(Value::Null) }
        "lsp_hover" => with_lsp(&sh, |b| b.hover(arg_str(&a, "rel")?, zahl(&a, "line"), zahl(&a, "col"))),
        "lsp_complete" => with_lsp(&sh, |b| b.vervollstaendigen(arg_str(&a, "rel")?, zahl(&a, "line"), zahl(&a, "col"))),
        "lsp_definition" => with_lsp(&sh, |b| b.definition(arg_str(&a, "rel")?, zahl(&a, "line"), zahl(&a, "col"))),
        "lsp_outline" => with_lsp(&sh, |b| b.gliederung(arg_str(&a, "rel")?)),
        "lsp_fixes" => with_lsp(&sh, |b| b.korrekturen(arg_str(&a, "rel")?, zahl(&a, "from"), zahl(&a, "to"))),
        "format_lines" => Ok(crate::text::formatieren(arg_str(&a, "text")?, arg_str(&a, "rel")?, arg_str(&a, "lang")?, zahl(&a, "from"), zahl(&a, "to"))),
        "search_start" => {
            let root = projekt_wurzel(&sh)?; let p = projekt_laden(&sh)?;
            let dateien = vc_ide_model::projsearch::candidate_files(&p.entries.iter().map(|e| (e.rel.clone(), e.group)).collect::<Vec<_>>());
            let ov: std::collections::HashMap<String, String> = a["overrides"].as_object().map(|o| o.iter().filter_map(|(k, v)| v.as_str().map(|t| (k.clone(), t.to_string()))).collect()).unwrap_or_default();
            let h = app.clone();
            let id = sh.suche.starten(crate::suche::parameter(root, dateien, ov, arg_str(&a, "text")?.to_string(), a["case"].as_bool().unwrap_or(false)), move |n, v| crate::events::emit(&h, n, v));
            Ok(json!({ "id": id }))
        }
        "search_cancel" => { sh.suche.abbrechen(); Ok(Value::Null) }
        "text_hover" | "char_names" => {
            let p = projekt_laden(&sh)?; let rel = a["rel"].as_str().unwrap_or("");
            let Some(data) = crate::lsp::daten_ordner() else { return Err(Fehler::neu("daten-fehlen")) };
            let sys = system_von(&p, rel);
            let dialekt = vc_target::load_target(&data, &sys).ok().and_then(|t| t.basic_dialect).unwrap_or_else(|| "cbm_v2".into());
            if cmd == "text_hover" {
                // natives BASIC: Karten für `#define`-Namen und Direktiven gehen vor den Zahlen- und Registerkarten
                let nativ = { let r = rel.to_ascii_lowercase(); r.ends_with(".bas") || r.ends_with(".lst") || r.ends_with(".tur") };
                if nativ && a["lang"].as_str() == Some("basic") {
                    let (root, text) = (projekt_wurzel(&sh)?, arg_str(&a, "text")?);
                    let (line, col16) = (zahl(&a, "line"), zahl(&a, "col"));
                    if let Some(src) = text.split('\n').nth(line) {
                        let col = crate::text::spalte_zeichen(src, col16);
                        if let Some((md, s0, e0)) = vc_ide::basicinfo::hover(&vc_ide::basicinfo::Req { root: &root, data: &data, rel, text }, line, col) {
                            let src = src.trim_end_matches('\r');
                            return Ok(json!({ "markdown": md, "line": line, "endLine": line, "from": crate::text::spalte_16(src, s0), "to": crate::text::spalte_16(src, e0) }));
                        }
                    }
                }
                Ok(crate::text::karte(&data, &sys, &dialekt, rel, arg_str(&a, "lang")?, arg_str(&a, "text")?, zahl(&a, "line"), zahl(&a, "col")))
            }
            else { Ok(crate::text::zeichennamen(&data, &dialekt, arg_str(&a, "lang")?, arg_str(&a, "line_text")?, zahl(&a, "col"))) }
        }
        "build_start" => {
            let root = projekt_wurzel(&sh)?; let p = projekt_laden(&sh)?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            if p.pf.as_ref().map_or(false, vc_ide::turbo::is_turbo_project) { return turbo_bauen_start(app, &sh, root, data, p.pf.as_ref().and_then(|x| x.main_file()).unwrap_or_default(), a["then"].as_str() == Some("run")); }
            let dateien: Vec<String> = p.entries.iter().map(|e| e.rel.clone()).collect();
            let rel = crate::bau::quelle(p.pf.as_ref(), &dateien, a["active"].as_str(), a["onlyActive"].as_bool().unwrap_or(false)).ok_or_else(|| Fehler::neu("keine-quelle"))?;
            if !sh.bau.beginnen() { return Err(Fehler::neu("bau-laeuft")); }
            let danach = a["then"].as_str().unwrap_or("none").to_string(); let pf = p.pf.clone(); let h = app.clone(); let r2 = rel.clone();
            crate::events::emit(app, "build:started", json!({ "rel": rel }));
            std::thread::spawn(move || {
                let sh = h.state::<Shell>();
                let fehler = sh.bau.geschuetzt(|| {
                let res = vc_ide::projfile::build_gui(&vc_ide::build::Ctx { root: &root, data: &data }, pf.as_ref(), &r2);
                let (ok, v) = match res { Ok(v) => (v["ok"] == true, v), Err(e) => (false, json!({ "ok": false, "file": r2, "diagnostics": [], "error": e })) };
                let info = crate::bau::Info::aus(&root, &v);
                if let Some(i) = &info { *sh.bau.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(i.clone()); }
                sh.bau.ende();
                crate::events::emit(&h, "build:done", json!({ "ok": ok, "rel": r2, "result": v, "then": danach }));
                if ok && danach == "run" { if let Some(i) = info { let ziel = sh.bau.ziel.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(); run_thread(&h, root, data, i, ziel); } }
                });
                if let Some(m) = fehler { crate::events::emit(&h, "build:done", json!({ "ok": false, "rel": r2, "result": { "ok": false, "file": r2, "diagnostics": [], "error": m }, "then": "none" })); }
            });
            Ok(json!({ "rel": rel }))
        }
        "run_start" => {
            let root = projekt_wurzel(&sh)?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let info = sh.bau.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone().ok_or_else(|| Fehler::neu("nichts-zu-starten"))?;
            let ziel = sh.bau.ziel.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(); run_thread(app, root, data, info, ziel); Ok(Value::Null)
        }
        "run_stop" => { let was = vc_ide::launch::stop_running(); crate::events::emit(app, "run:stopped", json!({ "was": was })); Ok(json!(was)) }
        "build_info" => Ok(sh.bau.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref().map_or(Value::Null, |i| i.json())),
        "build_cycles" => Ok(sh.bau.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref().map_or(json!([]), |i| i.zyklen_json(arg_str(&a, "rel").unwrap_or("")))),
        "tests_start" => {
            let root = projekt_wurzel(&sh)?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            // Emulator-Tests nur außerhalb von Testläufen der Prüfbrücke (kein Prozess)
            let backend = if crate::bau::gesperrt() { None } else { a["backend"].as_str().map(String::from) };
            let only = match a["only"].as_str() {
                Some("failed") => vc_ide::tests::suite::Only::Failed(sh.tests.suite.lock().unwrap_or_else(std::sync::PoisonError::into_inner).failed_tests()),
                Some("file") => vc_ide::tests::suite::Only::File(root.join(arg_str(&a, "file")?)),
                _ => vc_ide::tests::suite::Only::All,
            };
            let h = app.clone(); let r2 = root.clone();
            sh.tests.starten(root, data, backend, only, move |v| { let _ = &r2; crate::events::emit(&h, "tests:update", v); }).map_err(|_| Fehler::neu("tests-laufen"))?;
            Ok(Value::Null)
        }
        "tests_stop" => { sh.tests.abbrechen(); Ok(Value::Null) }
        "tests_state" => {
            let root = projekt_wurzel(&sh)?; let mut v = crate::testlauf::json(&root, &sh.tests.suite.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            v["backends"] = json!(vc_ide::tests::suite::available_backends().into_iter().map(|(i, n)| json!({ "id": i, "name": n })).collect::<Vec<_>>());
            v["files"] = json!(vc_ide::tests::model::find_tests(&root).iter().map(|p| p.strip_prefix(&root).unwrap_or(p).to_string_lossy().replace('\\', "/")).collect::<Vec<_>>());
            Ok(v)
        }
        "run_targets" => {
            let p = projekt_laden(&sh)?; let sys = sh.bau.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref().map(|i| i.target.clone()).unwrap_or_else(|| system_von(&p, a["rel"].as_str().unwrap_or("")));
            let wahl = sh.bau.ziel.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(); Ok(crate::bau::ziele(&sys, &wahl))
        }
        "run_target_set" => { *sh.bau.ziel.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = arg_str(&a, "id")?.to_string(); Ok(Value::Null) }
        "basic_get" => { let root = projekt_wurzel(&sh)?; Ok(crate::basic::zustand(&root)) }
        "basic_set" => { let root = projekt_wurzel(&sh)?; crate::basic::setzen(&root, &a).map_err(|e| Fehler::mit("projekt-fehler", json!({ "detail": e }))) }
        "projekt_info" => { let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?); crate::projekt::info(&root, &data).map_err(datei_fehler) }
        "projekt_set" => { let (root, data) = (projekt_wurzel(&sh)?, crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?); crate::projekt::setzen(&root, &data, &a).map_err(datei_fehler) }
        "turbo_state" => { let root = projekt_wurzel(&sh)?; crate::turbo::zustand(&root).map_err(turbo_fehler) }
        "turbo_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_PICK_ATR") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let p = mit_start(app.dialog().file(), &sh).set_title(vc_i18n::t!("turboui-datei-waehlen-titel")).add_filter(vc_i18n::t!("turboui-filter-diskabbild"), &["atr"]).blocking_pick_file();
            Ok(match p { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "turbo_check" => {
            let root = projekt_wurzel(&sh)?;
            match a["path"].as_str() { Some(p) => crate::turbo::pruefen_datei(&root, Path::new(p)).map_err(turbo_fehler), None => crate::turbo::pruefen_kopie(&root).map_err(turbo_fehler) }
        }
        "turbo_provide" => {
            let root = projekt_wurzel(&sh)?;
            let o = vc_ide::turbo::ProvideOpts { reduce: a["reduce"].as_bool().unwrap_or(false), anyway: a["anyway"].as_bool().unwrap_or(false), replace: a["replace"].as_bool().unwrap_or(false) };
            let r = vc_ide::turbo::provide(&root, Path::new(arg_str(&a, "src")?), &o).map_err(turbo_fehler)?;
            Ok(json!({ "disk": r.disk, "removed": r.removed, "free": r.free_sectors, "sha256": r.sha256, "gitignoreAdded": r.gitignore_added, "lines": r.report.lines(),
                       "state": crate::turbo::zustand(&root).map_err(turbo_fehler)?, "tree": baum(&projekt_laden(&sh)?) }))
        }
        "turbo_reduce" => {
            let root = projekt_wurzel(&sh)?;
            let removed = vc_ide::turbo::reduce_project_copy(&root).map_err(turbo_fehler)?;
            Ok(json!({ "removed": removed, "state": crate::turbo::zustand(&root).map_err(turbo_fehler)? }))
        }
        "turbo_set" => {
            let root = projekt_wurzel(&sh)?;
            let files: Vec<String> = a["files"].as_array().map(|v| v.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            crate::turbo::einstellungen_setzen(&root, a["program"].as_str().unwrap_or(""), &files).map_err(turbo_fehler)?;
            crate::turbo::zustand(&root).map_err(turbo_fehler)
        }
        "newproj_info" => { let d = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; crate::neuprojekt::info(&d, &vc_ide::settings::get()).ok_or_else(|| Fehler::mit("neuprojekt-fehler", json!({ "detail": vc_i18n::t!("neuprojekt-neues-projekt-data-templates-index-json") }))) }
        "newproj_system" => { let d = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; Ok(crate::neuprojekt::system_optionen(&d, &vc_ide::settings::get(), arg_str(&a, "system")?)) }
        "newproj_preview" => { let d = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?; let f = crate::neuprojekt::Form::aus_json(&a).map_err(|e| Fehler::mit("neuprojekt-fehler", json!({ "detail": e })))?; Ok(json!(crate::neuprojekt::vorschau(&d, &f))) }
        "newproj_create" => {
            let d = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let f = crate::neuprojekt::Form::aus_json(&a).map_err(|e| Fehler::mit("neuprojekt-fehler", json!({ "detail": e })))?;
            let (dir, notizen) = crate::neuprojekt::anlegen(&d, &vc_ide::settings::get(), &f).map_err(|e| Fehler::mit("neuprojekt-fehler", json!({ "detail": e })))?;
            let mut out = oeffne(app, &dir.to_string_lossy())?; out["notes"] = json!(notizen); Ok(out)
        }
        "dir_pick" => {
            #[cfg(feature = "testbridge")]
            if let Ok(p) = std::env::var("VC_TEST_DIR") { return Ok(json!(p)); }
            use tauri_plugin_dialog::DialogExt;
            let mut b = app.dialog().file().set_title(vc_i18n::t!("neuprojekt-speicherort"));
            if let Some(s) = a["start"].as_str().filter(|s| Path::new(s).is_dir()) { b = b.set_directory(s); }
            Ok(match b.blocking_pick_folder() { Some(p) => json!(p.to_string()), None => Value::Null })
        }
        "session_get" => { let r = projekt_wurzel(&sh)?; Ok(crate::sitzung::sitzungsdatei().map_or(json!({ "enabled": crate::sitzung::wiederherstellen_an(&r), "open": [], "active": null, "marks": {} }), |f| crate::sitzung::lesen(&r, &f))) }
        "session_set" => { let r = projekt_wurzel(&sh)?; if let Some(f) = crate::sitzung::sitzungsdatei() { crate::sitzung::speichern(&r, &f, &a["open"], &a["active"], &a["marks"]); } Ok(Value::Null) }
        "restore_tabs_set" => { let r = projekt_wurzel(&sh)?; crate::sitzung::wiederherstellen_setzen(&r, a["on"].as_bool().unwrap_or(false)).map_err(|e| Fehler::mit("projekt-fehler", json!({ "detail": e })))?; Ok(Value::Null) }
        "scratch_get" => Ok(json!({ "enabled": vc_ide_model::scratchsave::enabled(), "entries": crate::sitzung::zwischenkopiedatei().map_or(json!([]), |f| crate::sitzung::zwischenkopie_lesen(&f, vc_ide_model::scratchsave::now_secs())) })),
        "scratch_set" => { if vc_ide_model::scratchsave::enabled() { if let Some(f) = crate::sitzung::zwischenkopiedatei() { crate::sitzung::zwischenkopie_schreiben(&f, &a["entries"], vc_ide_model::scratchsave::now_secs()); } } Ok(Value::Null) }
        "scratch_pref" => { if let Some(on) = a["on"].as_bool() { vc_ide_model::scratchsave::set_enabled(on); } Ok(json!(vc_ide_model::scratchsave::enabled())) }
        "basic_convert" => {
            let root = projekt_wurzel(&sh)?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            let kind = vc_ide::basicconv::Kind::parse(arg_str(&a, "kind")?).ok_or_else(|| Fehler::mit("arg-fehlt", json!({ "name": "kind" })))?;
            let range = match (a["from"].as_u64(), a["to"].as_u64()) { (Some(f), Some(t)) => Some((f as u32, t as u32)), _ => None };
            let r = vc_ide::basicconv::Req { root: &root, data: &data, rel: arg_str(&a, "rel")?, text: arg_str(&a, "text")?, kind, range, with_includes: a["includes"].as_bool().unwrap_or(true) };
            Ok(if a["apply"].as_bool() == Some(true) { vc_ide::basicconv::apply(&r) } else { vc_ide::basicconv::run(&r) })
        }
        "basic_info" => {
            let root = projekt_wurzel(&sh)?; let data = crate::lsp::daten_ordner().ok_or_else(|| Fehler::neu("daten-fehlen"))?;
            Ok(vc_ide::basicinfo::info(&vc_ide::basicinfo::Req { root: &root, data: &data, rel: arg_str(&a, "rel")?, text: arg_str(&a, "text")? }))
        }
        "basic_codes" => Ok(vc_ide::basicinfo::code_texts()),
        "app_quit" => { app.exit(0); Ok(Value::Null) }
        _ => Err(Fehler::mit("befehl-unbekannt", json!({ "cmd": cmd }))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Jeder Befehl der Befehlstabelle hat entweder einen Zweig in `ui/src/lib/commands.ts` (oder gehört zu den Markdown- und Namensraum-Befehlen) oder steht in
    /// `NOCH_NICHT_GEBAUT` (dann zeigen Menü und Palette ihn nicht): so zeigt keine Menüzeile nur „noch nicht gebaut“.
    #[test] fn jeder_sichtbare_befehl_hat_eine_funktion() {
        let c = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ui/src/lib/commands.ts")).unwrap();
        let md = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ui/src/lib/markdown.svelte.ts")).unwrap();
        let ohne: Vec<&str> = vc_commands::BEFEHLE.iter().map(|i| i.id).filter(|id| !c.contains(&format!("case \"{id}\"")) && !md.contains(&format!("{id}:")) && !vc_commands::NOCH_NICHT_GEBAUT.contains(id)).collect();
        assert!(ohne.is_empty(), "Befehle ohne Funktion, die Menü und Palette zeigen: {ohne:?}");
        for id in vc_commands::NOCH_NICHT_GEBAUT { assert!(vc_commands::nach_id(id).is_some(), "{id} steht nicht in der Befehlstabelle"); assert!(!c.contains(&format!("case \"{id}\"")), "{id} hat inzwischen einen Zweig: aus NOCH_NICHT_GEBAUT nehmen"); }
    }
    #[test] fn befehlsliste_hat_keine_doppelten() { let mut v = BEFEHLE.to_vec(); v.sort(); let n = v.len(); v.dedup(); assert_eq!(n, v.len()); }
    #[test] fn ftl_deutsch_und_englisch_enthalten_den_katalog() {
        assert!(ftl(false).contains("menu-file")); assert!(ftl(true).contains("menu-file"));
    }
    #[test] fn jede_ereignis_und_befehlsliste_steht_in_api_ts() {
        let api = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../ui/src/lib/api.ts")).expect("ui/src/lib/api.ts");
        for b in BEFEHLE { assert!(api.contains(&format!("\"{b}\"")), "Befehl {b} fehlt in api.ts") }
        for e in crate::events::EREIGNISSE { assert!(api.contains(&format!("\"{e}\"")), "Ereignis {e} fehlt in api.ts") }
    }
    #[test] fn papierkorb_fehler_kommt_als_eigener_code_zur_oberflaeche() {
        let r = std::env::temp_dir().join(format!("vc_papierkorb_code_{}", std::process::id())); let _ = std::fs::remove_dir_all(&r); std::fs::create_dir_all(&r).unwrap();
        std::fs::write(r.join("a.asm"), "nop").unwrap();
        std::fs::write(r.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"main":"a.asm","sources":[{"name":"p","files":["a.asm"]}]}"#).unwrap();
        let mut p = vc_ide_model::project::Project::load(r.clone(), Some("p.vcprj".into())).unwrap();
        // im Test gibt es keinen Papierkorb: gewünschter Papierkorb scheitert mit dem eigenen Code, die Datei bleibt
        let e = datei_loeschen(&mut p, "a.asm", true).unwrap_err();
        assert_eq!(e.code, "fileops-papierkorb-fehlgeschlagen");
        assert_eq!(e.args["rel"], "a.asm"); assert!(e.args["detail"].as_str().unwrap().contains("a.asm"));
        assert!(r.join("a.asm").exists());
        // andere Fehler bleiben `datei-fehler`
        assert_eq!(datei_loeschen(&mut p, "gibts-nicht.asm", false).unwrap_err().code, "datei-fehler");
        // endgültig löschen geht mit trash:false
        assert_eq!(datei_loeschen(&mut p, "a.asm", false).unwrap(), false); assert!(!r.join("a.asm").exists());
        let _ = std::fs::remove_dir_all(&r);
    }
}
