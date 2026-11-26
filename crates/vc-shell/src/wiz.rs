// SPDX-License-Identifier: GPL-3.0-or-later
//! Einrichtungsassistent (E12.3): arbeitet auf einem Entwurf im Speicher (`vc_ide::setup::Draft`), zeigt die Änderungsliste und schreibt erst bei „Übernehmen“
//! (Sicherung `settings.json.bak`, danach „Rückgängig“). Die Seiten für Systeme, Emulatoren, Hardware und ROMs sind die der Einstellungen; der Assistent liefert nur Ablauf,
//! Entwurf, Änderungsliste und das Schreiben. Er installiert und lädt nichts herunter. Im Testlauf zeigen die Dateien in den (eigenen) Konfigurationsordner unter `HOME`.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;
use vc_ide::setup::{self, Applied, Draft, Kind, Mode, Part, Paths, Snapshot, Topic, ViewPrefs, WizardState};
use vc_ide::settings::Settings;

pub struct Wiz { pub paths: Paths, pub cur: Snapshot, pub draft: Draft, pub applied: Option<Applied>, pub mode: Mode, pub erste: bool, pub zustand: WizardState, /** Stand von `wizard.json` vor „Übernehmen“ (`None` = Datei gab es nicht): „Rückgängig“ stellt ihn wieder her */ pub zustand_vorher: Option<Option<WizardState>> }

fn mode_aus(s: &str) -> Mode { match s { "full" => Mode::Full, "roms" => Mode::Part(Part::Roms), "emulators" => Mode::Part(Part::Emulators), "hardware" => Mode::Part(Part::Hardware), "design" => Mode::Part(Part::DesignEditor), _ => Mode::Quick } }
fn mode_id(m: Mode) -> &'static str { match m { Mode::Quick => "quick", Mode::Full => "full", Mode::Part(Part::Roms) => "roms", Mode::Part(Part::Emulators) => "emulators", Mode::Part(Part::Hardware) => "hardware", Mode::Part(Part::DesignEditor) => "design" } }
fn fehler(e: String) -> Fehler { Fehler::mit("wiz-fehler", json!({ "detail": e })) }

/// Was beim Programmstart zu tun ist: `first` (Assistent öffnet sich), `offer` (einmal anbieten), `normal`
fn entdecke(kind: &str) -> Option<String> { vc_ide::launch::detect(kind) }
/// „Neues erkannt“ (E12.3): Emulator-Sorten, die jetzt gefunden werden und beim letzten Lauf nicht; nur Dateisystem und PATH, nichts wird gestartet
fn news(st: &WizardState) -> Vec<Value> {
    setup::news_since_last_run(st, &entdecke).into_iter().map(|k| json!({ "kind": k, "label": vc_ide::emulators::for_kind(&k).map(|e| e.labels.get(&k).cloned().unwrap_or_else(|| e.name.clone())).unwrap_or_else(|| k.clone()) })).collect()
}
/// „Neues erkannt“ beantworten: `seen` merkt die jetzt gefundenen Sorten als bekannt, `off` schaltet den Hinweis ab
pub fn news_antwort(modus: &str) -> Result<(), String> {
    let p = Paths::current().ok_or_else(|| vc_i18n::t!("err-wiz-kein-ordner").to_string())?; let mut st = WizardState::load(&p.wizard);
    match modus { "off" => st.no_news_hint = true, _ => st.known_programs = setup::found_kinds(&entdecke) }
    st.save(&p.wizard)
}
pub fn start_stand() -> Value {
    let Some(p) = Paths::current() else { return json!({ "start": "normal", "suppressed": true, "incomplete": false }) };
    let st = WizardState::load(&p.wizard);
    json!({ "start": match setup::start_state(&p) { setup::Start::FirstRun => "first", setup::Start::OfferOnce => "offer", setup::Start::Normal => "normal" }, "suppressed": setup::auto_start_suppressed(), "incomplete": st.incomplete(), "later": st.later, "news": news(&st) })
}

pub fn starten(data: &std::path::Path, modus: &str) -> Result<Wiz, Fehler> {
    let paths = Paths::current().ok_or_else(|| Fehler::neu("wiz-kein-ordner"))?;
    let cur = setup::load_snapshot(&paths).map_err(fehler)?;
    let erste = matches!(setup::start_state(&paths), setup::Start::FirstRun);
    let mut draft = if erste { Draft::for_first_run(&cur) } else { Draft::from_snapshot(&cur) };
    draft.examples.source = data.parent().map(|p| p.join("examples")).filter(|p| p.is_dir());
    draft.examples.copy = erste && draft.examples.source.is_some();
    Ok(Wiz { paths: paths.clone(), cur, draft, applied: None, mode: mode_aus(modus), erste, zustand: WizardState::load(&paths.wizard), zustand_vorher: None })
}

fn kind_text(k: Kind) -> &'static str { match k { Kind::New => "new", Kind::Changed => "changed", Kind::Removed => "removed", Kind::Same => "same" } }

/// Entwurf, Schritte, Änderungsliste und Probleme für die Oberfläche
pub fn json(w: &Wiz, abgewaehlt: &BTreeSet<String>) -> Value {
    let changes = setup::diff(&w.cur, &w.draft);
    let items = w.draft.examples.source.as_ref().map(|r| setup::list_examples(r, &w.draft.settings.targets.iter().map(|t| t.system.clone()).collect::<Vec<_>>())).unwrap_or_default();
    json!({
        "mode": mode_id(w.mode), "firstRun": w.erste, "applied": w.applied.is_some(),
        "steps": w.mode.steps().into_iter().map(|t| json!({ "id": t.id(), "title": t.title(), "group": t.group() })).collect::<Vec<_>>(),
        "settings": serde_json::to_value(&w.draft.settings).unwrap_or(Value::Null),
        "view": { "scheme": w.draft.view.scheme, "zoom": w.draft.view.zoom, "font": w.draft.view.font },
        "examples": { "copy": w.draft.examples.copy, "excluded": w.draft.examples.excluded.iter().collect::<Vec<_>>(), "source": w.draft.examples.source.as_ref().map(|p| p.to_string_lossy().into_owned()), "dest": w.draft.examples.dest,
            "items": items.iter().map(|i| json!({ "id": i.id(), "system": i.system, "name": i.name })).collect::<Vec<_>>(),
            "destEffective": setup::examples_dest_in(&w.draft.examples.dest, &if w.draft.settings.projects_dir.trim().is_empty() { setup::effective_projects_dir(&w.draft.settings) } else { w.draft.settings.projects_dir.clone() }).to_string_lossy() },
        "projects": setup::effective_projects_dir(&w.draft.settings),
        "changes": changes.iter().map(|c| json!({ "key": c.key, "area": c.area, "kind": kind_text(c.kind), "label": c.label, "old": c.old, "new": c.new, "selected": c.selected && !abgewaehlt.contains(&c.key), "sensitive": c.sensitive, "note": c.note, "topic": c.topic.id() })).collect::<Vec<_>>(),
        "problems": setup::problems(&w.draft.settings).into_iter().map(|p| json!({ "topic": p.topic.id(), "text": p.text })).collect::<Vec<_>>(),
        "later": w.zustand.later,
    })
}

/// Entwurf aus der Oberfläche übernehmen (nur die genannten Teile)
pub fn setzen(w: &mut Wiz, a: &Value) -> Result<(), Fehler> {
    if let Some(s) = a.get("settings").filter(|s| !s.is_null()) { let mut s = serde_json::from_value::<Settings>(s.clone()).map_err(|e| Fehler::mit("einst-ungueltig", json!({ "detail": e.to_string() })))?; s.migrate_legacy(); w.draft.settings = s; }
    if let Some(v) = a.get("view").filter(|v| v.is_object()) {
        if let Some(s) = v["scheme"].as_str() { w.draft.view.scheme = if matches!(s, "light" | "system") { s.into() } else { "dark".into() }; }
        if let Some(z) = v["zoom"].as_f64() { w.draft.view.zoom = (z as f32).clamp(0.7, 2.5); }
        if let Some(f) = v["font"].as_f64() { w.draft.view.font = (f as f32).clamp(9.0, 36.0); }
    }
    if let Some(e) = a.get("examples").filter(|e| e.is_object()) {
        if let Some(c) = e["copy"].as_bool() { w.draft.examples.copy = c; }
        if let Some(x) = e["excluded"].as_array() { w.draft.examples.excluded = x.iter().filter_map(|i| i.as_str().map(String::from)).collect(); }
        if let Some(d) = e["dest"].as_str() { w.draft.examples.dest = d.to_string(); }
    }
    Ok(())
}
pub fn system_schalten(w: &mut Wiz, id: &str, an: bool) { setup::set_system(&mut w.draft.settings, id, an); }

/// Übernehmen: schreibt die gewählten Änderungen (Sicherung, Beispiele kopieren), macht sie im Prozess wirksam und merkt den Stand des Assistenten
pub fn uebernehmen(w: &mut Wiz, abgewaehlt: &BTreeSet<String>) -> Result<Value, Fehler> {
    let sel = |k: &str| !abgewaehlt.contains(k);
    let a = setup::apply_to(&w.paths, &w.cur, &w.draft, &sel).map_err(fehler)?;
    setup::apply_replace_memory(&a);
    let kopiert = a.copy.copied.clone(); let notes = a.notes.clone(); let backup = a.backup.as_ref().map(|p| p.to_string_lossy().into_owned());
    w.zustand_vorher = Some(if w.paths.wizard.exists() { Some(w.zustand.clone()) } else { None });
    w.zustand.known_programs = setup::found_kinds(&entdecke); w.zustand.first_run_done = true; w.zustand.mode = w.mode.id().to_string(); w.zustand.later.clear(); for k in &kopiert { if !w.zustand.examples_copied.contains(k) { w.zustand.examples_copied.push(k.clone()) } }
    let _ = w.zustand.save(&w.paths.wizard);
    w.cur = setup::load_snapshot(&w.paths).map_err(fehler)?; w.applied = Some(a);
    Ok(json!({ "notes": notes, "copied": kopiert, "backup": backup }))
}
/// Rückgängig: Einstellungen, Darstellung und kopierte Beispiele wie vorher
pub fn rueckgaengig(w: &mut Wiz) -> Result<(), Fehler> {
    let Some(a) = w.applied.take() else { return Ok(()) };
    setup::undo_to(&w.paths, &a).map_err(fehler)?;
    // auch der Stand des Assistenten (first_run_done, kopierte Beispiele …): beim Erststart gibt es die Datei danach wieder nicht, der Assistent kommt beim nächsten Start
    match w.zustand_vorher.take() {
        Some(None) => { let _ = std::fs::remove_file(&w.paths.wizard); w.zustand = WizardState::default(); }
        Some(Some(z)) => { w.zustand = z; let _ = w.zustand.save(&w.paths.wizard); }
        None => {}
    }
    w.cur = setup::load_snapshot(&w.paths).map_err(fehler)?; setup::apply_replace_memory_settings(w.cur.settings.clone());
    Ok(())
}
pub fn spaeter(w: &mut Wiz, thema: &str) { if let Some(t) = Topic::from_id(thema) { w.zustand.mark_later(t) } let _ = w.zustand.save(&w.paths.wizard); }
pub fn beenden(w: &mut Wiz) { if !w.zustand.first_run_done && w.erste { w.zustand.first_run_done = true; } let _ = w.zustand.save(&w.paths.wizard); }
pub fn ansicht_prefs(v: &ViewPrefs) -> Value { json!({ "scheme": v.scheme, "zoom": v.zoom, "font": v.font }) }
pub fn _pfad(p: &PathBuf) -> String { p.to_string_lossy().into_owned() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn modi_und_schritte() { assert_eq!(mode_aus("full").steps().len(), 12); assert_eq!(mode_aus("quick").steps().len(), 7); assert_eq!(mode_id(mode_aus("hardware")), "hardware"); }
}
