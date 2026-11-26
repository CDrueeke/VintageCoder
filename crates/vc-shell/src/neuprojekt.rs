// SPDX-License-Identifier: GPL-3.0-or-later
//! Neues Projekt (E1.4): Vorlagen, Optionen je Zielsystem, Vorschau der Dateistruktur und Anlegen. Dieselbe Logik wie in der alten Oberfläche
//! (`vc-gui/src/neuprojekt.rs`), aber ohne Oberfläche: die Webseite hält die Eingaben und fragt hier nach Optionen, Vorschau und dem Anlegen.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use vc_ide::targets;
use vc_ide_model::recent::{dialect_label, target_dialect};
use vc_media::wizard::Zeichensatz;

fn strs(v: &Value) -> Vec<String> { v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }
fn json_lesen(p: &Path) -> Option<Value> { serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok() }
/// Diskettenformate, die zu einem System passen
pub fn disk_formate(system: &str) -> &'static [&'static str] { match system { "c128" => &["d64", "d71", "d81"], "atari8" => &["atr"], "mega65" => &["d81"], _ => &["d64"] } }

#[derive(Clone, Debug)]
pub struct Karte { pub vorlage: String, pub variante: Option<String>, pub name: String, pub beschreibung: String, pub basic: bool, pub systeme: Vec<String>, pub dateien: Vec<String> }

/// Vorlagenkarten aus `templates/index.json` (jede Vorlage und jede ihrer Varianten)
pub fn karten(data: &Path) -> Option<Vec<Karte>> {
    let idx = json_lesen(&data.join("templates/index.json"))?; let mut out = vec![];
    for t in idx["templates"].as_array()? {
        let mut add = |name: String, beschr: String, id: Option<String>, node: &Value| {
            let basic = node["sources"].as_array().and_then(|s| s.first()).map_or("asm", |s| s["kind"].as_str().unwrap_or("asm")) == "basic";
            let dateien = node["files"].as_array().map(|f| f.iter().filter_map(|x| x["to"].as_str().map(String::from)).collect()).unwrap_or_default();
            out.push(Karte { vorlage: t["id"].as_str().unwrap_or("").into(), variante: id, name, beschreibung: beschr, basic, systeme: strs(&node["systems"]), dateien });
        };
        add(t["name"].as_str().unwrap_or("").into(), t["description"].as_str().unwrap_or("").into(), None, t);
        for v in t["variants"].as_array().into_iter().flatten() { add(v["name"].as_str().unwrap_or("").into(), v["description"].as_str().unwrap_or("").into(), v["id"].as_str().map(String::from), v); }
    }
    Some(out)
}

fn vic20_speicher(data: &Path) -> Vec<(String, String)> {
    let Some(t) = json_lesen(&data.join("targets/vic20.target.json")) else { return vec![] };
    let vorgabe = t["memory"]["default"].as_str().unwrap_or("").to_string();
    let mut v: Vec<(String, String)> = t["memory"]["configs"].as_object().map(|o| o.iter().map(|(k, c)| (k.clone(), c["label"].as_str().unwrap_or(k).to_string())).collect()).unwrap_or_default();
    if let Some(i) = v.iter().position(|x| x.0 == vorgabe) { let d = v.remove(i); v.insert(0, d); } v
}
fn dialekte(data: &Path, system: &str) -> Vec<String> { let mut v: Vec<String> = target_dialect(data, system).into_iter().collect(); if system == "atari8" { v.push("turbo_basic_xl".into()); } v }

/// Alles, was die Oberfläche beim Öffnen des Dialogs braucht: Zielsysteme, Vorlagenkarten, Strukturen, Standardort
pub fn info(data: &Path, einst: &vc_ide::settings::Settings) -> Option<Value> {
    let k = karten(data)?;
    let ort = { let p = einst.projects_dir.trim(); if p.is_empty() { vc_core::userdirs::home_dir().map(|h| h.join("VintageCoder")).unwrap_or_else(|| PathBuf::from(".")) } else { PathBuf::from(vc_ide::settings::expand_home(p)) } };
    Some(json!({
        "systems": targets::installed_systems(Some(data)).into_iter().map(|i| json!({ "id": i.id, "name": i.name(), "cpu": i.cpu() })).collect::<Vec<_>>(),
        "default": vc_target::default_target_or_c64(Some(data)),
        "cards": k.iter().enumerate().map(|(i, c)| json!({ "i": i, "name": c.name, "description": c.beschreibung, "basic": c.basic, "systems": c.systeme, "files": c.dateien })).collect::<Vec<_>>(),
        "structures": vc_ide::structure::list_all(data).iter().map(|s| json!({ "id": s.id, "name": s.name, "description": s.description })).collect::<Vec<_>>(),
        "dir": ort.to_string_lossy(), "dirPretty": vc_ide_model::recent::pretty(&ort),
    }))
}

/// Optionen eines Zielsystems: gültiges Hardware-Profil, BASIC-Dialekte, Disketten, Emulatoren, Hardware, VIC-20-Speicher
pub fn system_optionen(data: &Path, einst: &vc_ide::settings::Settings, system: &str) -> Value {
    let t = vc_target::load_target(data, system);
    let cfg = einst.target(system);
    let globaler_emu = cfg.and_then(|t| t.standard_emulator()).map(|e| e.id.clone()).unwrap_or_default();
    let globales_ziel = cfg.map(|t| t.start_on.clone()).filter(|id| einst.machine(id).is_some()).unwrap_or_default();
    let emus: Vec<Value> = match cfg.filter(|t| !t.emulators.is_empty()) {
        Some(t) => t.emulators.iter().map(|e| json!({ "id": e.id, "name": e.display_name(), "global": e.id == globaler_emu })).collect(),
        None => vec![],
    };
    json!({
        "valid": t.is_ok(), "cpu": t.as_ref().map(|t| t.cpu.clone()).unwrap_or_default(), "chips": t.as_ref().map(|t| t.chips.join(", ")).unwrap_or_default(),
        "dialects": dialekte(data, system).iter().map(|d| json!({ "id": d, "label": dialect_label(d) })).collect::<Vec<_>>(),
        "disks": disk_formate(system), "norm": system != "mega65", "memory": if system == "vic20" { vic20_speicher(data).into_iter().map(|(k, l)| json!({ "id": k, "label": l })).collect::<Vec<_>>() } else { vec![] },
        "emulators": emus, "globalEmulatorKind": targets::kind_label(&einst.emulator_for(system).kind), "globalEmulator": globaler_emu,
        "machines": einst.machines_for(system).map(|m| json!({ "id": m.id, "name": m.display_name() })).collect::<Vec<_>>(), "globalStart": globales_ziel,
    })
}

/// Eingaben des Dialogs
#[derive(Clone, Debug)]
pub struct Form {
    pub system: String, pub extra: Vec<String>, pub basic: bool, pub karte: usize, pub name: String, pub ort: PathBuf, pub dialekt: String, pub norm: String, pub speicher: String,
    pub emulator: String, pub start_on: String, pub disk: String, pub zeichensatz: Zeichensatz, pub struktur: usize, pub readme: bool, pub git: bool,
}
impl Form {
    pub fn aus_json(a: &Value) -> Result<Form, String> {
        let s = |k: &str| a[k].as_str().unwrap_or("").to_string();
        Ok(Form {
            system: s("system"), extra: strs(&a["extra"]), basic: a["basic"].as_bool().unwrap_or(false), karte: a["card"].as_u64().ok_or(vc_i18n::t!("neuprojekt-keine-vorlage-gewaehlt"))? as usize,
            name: s("name"), ort: PathBuf::from(s("dir")), dialekt: s("dialect"), norm: if s("norm").is_empty() { "pal".into() } else { s("norm") }, speicher: s("memory"), emulator: s("emulator"), start_on: s("startOn"), disk: s("disk"),
            zeichensatz: Zeichensatz::parse(&s("charset")).unwrap_or(Zeichensatz::Auto), struktur: a["structure"].as_u64().unwrap_or(0) as usize, readme: a["readme"].as_bool().unwrap_or(false), git: a["git"].as_bool().unwrap_or(false),
        })
    }
    pub fn name_ok(&self) -> bool { !self.name.is_empty() && self.name.len() <= 48 && self.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') }
    fn systeme(&self) -> Vec<String> { let mut v = vec![self.system.clone()]; v.extend(self.extra.iter().filter(|e| **e != self.system).cloned()); v }
}

/// Dateistruktur-Vorschau (Pfade; Ordner enden auf `/`)
pub fn vorschau(data: &Path, f: &Form) -> Vec<String> {
    let k = karten(data).unwrap_or_default(); let st = vc_ide::structure::list_all(data);
    let mut v = vec![format!("{}.vcprj", f.name)];
    if let Some(c) = k.get(f.karte) { for s in f.systeme() { for d in &c.dateien { v.push(d.replace("{system}", &s)); } } }
    if let Some(s) = st.get(f.struktur) { for d in s.dirs_for(&f.systeme()) { if !v.iter().any(|x| x.starts_with(&format!("{d}/"))) { v.push(format!("{d}/")); } } }
    for z in vc_media::wizard::zeichensatz_vorschau(data, &f.systeme(), f.zeichensatz) { v.push(z); }
    if f.readme { v.push("README.md".into()); } if f.git { v.push(".gitignore".into()); } v.push("build/".into()); v
}

/// `git init` im Projektordner; im Testlauf (`VC_TEST_NO_SPAWN`) wird kein Prozess gestartet
fn git_init(dir: &Path) -> bool {
    if crate::bau::gesperrt() { return true; }
    std::process::Command::new("git").arg("init").arg("-q").current_dir(dir).stdin(std::process::Stdio::null()).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Projekt anlegen und die Projektdatei ergänzen; liefert den Projektordner und Hinweise (z. B. Git fehlt)
pub fn anlegen(data: &Path, einst: &vc_ide::settings::Settings, f: &Form) -> Result<(PathBuf, Vec<String>), String> {
    let k = karten(data).ok_or(vc_i18n::t!("neuprojekt-neues-projekt-data-templates-index-json"))?;
    let c = k.get(f.karte).ok_or(vc_i18n::t!("neuprojekt-keine-vorlage-gewaehlt"))?;
    if !f.name_ok() { return Err(vc_i18n::t!("neuprojekt-nur-buchstaben-ziffern-und")); }
    let dir = f.ort.join(&f.name); let systeme = f.systeme();
    vc_media::wizard::create_with(data, &dir, &f.name, &systeme, &c.vorlage, c.variante.as_deref())?;
    let mut notizen = vec![]; let datei = dir.join(format!("{}.vcprj", f.name));
    let mut pf = vc_ide::projfile::ProjectFile::load(&datei)?;
    for t in &mut pf.targets {
        let sys = if t.system.is_empty() { t.id.clone() } else { t.system.clone() };
        if sys != "mega65" { t.variant = f.norm.clone(); }
        if sys == "vic20" && !f.speicher.is_empty() { t.extra.insert("memory".into(), json!(f.speicher)); }
    }
    if f.basic { for s in &mut pf.sources { if s.kind != "basic" { continue; } let sys = s.targets.first().cloned().unwrap_or_else(|| f.system.clone()); if target_dialect(data, &sys).as_deref() != Some(f.dialekt.as_str()) && sys == f.system { s.dialect = f.dialekt.clone(); } } }
    pf.set_overrides_if_differs(&f.system, einst, &f.emulator, &f.start_on);
    if !f.disk.is_empty() { pf.disk = Some(vc_ide::projfile::DiskSpec { format: f.disk.clone(), ..Default::default() }); }
    if let Some(st) = vc_ide::structure::list_all(data).get(f.struktur) { st.apply(&dir, &mut pf, &systeme, f.git)?; }
    pf.save(&datei)?;
    if let Err(e) = vc_media::wizard::zeichensatz_anlegen(data, &dir, &systeme, f.zeichensatz) { notizen.push(vc_i18n::t!("neuprojekt-zeichensatz-nicht-angelegt-e", e = &e)); }
    if f.readme {
        let sys = targets::system(&f.system).map_or(f.system.clone(), |s| s.name());
        std::fs::write(dir.join("README.md"), format!("# {}\n\nProjekt für {} ({}), angelegt mit VintageCoder.\n\n- Bauen: F7 (oder `vcc build .`)\n- Starten: F5\n- Debuggen: F6\n", f.name, sys, c.name)).map_err(|e| e.to_string())?;
    }
    if f.git {
        std::fs::write(dir.join(".gitignore"), "build/\n.vcbackup/\n").map_err(|e| e.to_string())?;
        if !git_init(&dir) { notizen.push(vc_i18n::t!("neuprojekt-git-init-ist-fehlgeschlagen-ist-git").to_string()); }
    }
    Ok((dir, notizen))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn daten() -> Option<PathBuf> { crate::lsp::daten_ordner() }
    fn form(system: &str, card: usize, name: &str, ort: PathBuf) -> Form {
        Form { system: system.into(), extra: vec![], basic: false, karte: card, name: name.into(), ort, dialekt: String::new(), norm: "pal".into(), speicher: String::new(), emulator: String::new(), start_on: String::new(), disk: String::new(), zeichensatz: Zeichensatz::Keine, struktur: 0, readme: false, git: false }
    }
    #[test] fn namen_werden_geprueft() {
        let mut f = form("c64", 0, "gut_1-x", PathBuf::new()); assert!(f.name_ok());
        for n in ["", "mit leerzeichen", "ümlaut", &"x".repeat(49)] { f.name = n.into(); assert!(!f.name_ok(), "{n}"); }
    }
    #[test] fn vorlagen_und_optionen_kommen_aus_den_datendateien() {
        let Some(d) = daten() else { return };
        let k = karten(&d).expect("templates/index.json"); assert!(!k.is_empty());
        assert!(k.iter().any(|c| c.systeme.iter().any(|s| s == "c64") && !c.basic), "eine Assembler-Vorlage für C64");
        let o = system_optionen(&d, &vc_ide::settings::Settings::default(), "c64"); assert_eq!(o["valid"], json!(true)); assert!(o["dialects"].as_array().unwrap().len() >= 1);
        assert_eq!(system_optionen(&d, &vc_ide::settings::Settings::default(), "mega65")["norm"], json!(false), "MEGA65 hat keine Videonorm");
    }
    #[test] fn anlegen_schreibt_projekt_readme_und_gitignore_und_meldet_fehler() {
        let Some(d) = daten() else { return };
        let tmp = std::env::temp_dir().join(format!("vc_np_{}", std::process::id())); let _ = std::fs::remove_dir_all(&tmp); std::fs::create_dir_all(&tmp).unwrap();
        let k = karten(&d).unwrap(); let c = k.iter().position(|c| c.systeme.iter().any(|s| s == "c64") && !c.basic).unwrap();
        let mut f = form("c64", c, "probe", tmp.clone()); f.readme = true; f.git = true;
        let v = vorschau(&d, &f); assert!(v.contains(&"probe.vcprj".to_string()) && v.contains(&"README.md".to_string()) && v.contains(&".gitignore".to_string()) && v.contains(&"build/".to_string()), "{v:?}");
        let (dir, _n) = anlegen(&d, &vc_ide::settings::Settings::default(), &f).expect("anlegen");
        assert!(dir.join("probe.vcprj").is_file() && dir.join("README.md").is_file() && dir.join(".gitignore").is_file());
        assert!(anlegen(&d, &vc_ide::settings::Settings::default(), &f).is_err(), "ein zweites Mal im selben Ordner schlägt sichtbar fehl");
        f.name = "ungültig!".into(); assert!(anlegen(&d, &vc_ide::settings::Settings::default(), &f).is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
