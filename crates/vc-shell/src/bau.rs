// SPDX-License-Identifier: GPL-3.0-or-later
//! Bauen und Starten (E1.3): Bauen im Hintergrund über `vc_ide::projfile::build_gui` (Ereignisse `build:started`, `build:done`),
//! Starten des Ergebnisses im Emulator oder auf einer eingerichteten Maschine (`run:started`, `run:failed`), Beenden (`run_stop`).
//! Das Ergebnis des letzten Baus (Ziel, Artefakt, Segmente, Zeilenadressen, Marken) bleibt für Statuszeile, „Ab hier starten“ und Debug erhalten.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Was vom letzten gelungenen Bau gebraucht wird
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub rel: String,
    pub target: String,
    pub artifact: PathBuf,
    pub size: u64,
    pub run: Option<u16>,
    pub symbols: HashMap<String, i64>,
    /// (Datei, Zeile ab 1, Adresse, Länge)
    pub zeilen: Vec<(String, usize, u16, usize)>,
    /// (Start, Länge)
    pub segmente: Vec<(u32, usize)>,
    /// (Datei, Zeile ab 1, kleinste, größte Zyklenzahl der Anweisung)
    pub zyklen: Vec<(String, usize, u8, u8)>,
}

impl Info {
    /// Aus dem Bauergebnis (`ok: true`); `None` bei Fehlschlag
    pub fn aus(root: &Path, v: &Value) -> Option<Info> {
        if v["ok"] != true { return None; }
        let art = v["artifact"].as_str()?;
        Some(Info {
            rel: v["file"].as_str().unwrap_or("").to_string(), target: v["target"].as_str().unwrap_or("c64").to_string(), artifact: root.join(art), size: v["size"].as_u64().unwrap_or(0),
            run: v["run"].as_u64().map(|x| x as u16),
            symbols: v["symbols"].as_array().map(|a| a.iter().filter_map(|s| Some((s["name"].as_str()?.to_string(), s["value"].as_i64()?))).collect()).unwrap_or_default(),
            zeilen: v["lines"].as_array().map(|a| a.iter().filter_map(|l| Some((l["file"].as_str()?.to_string(), l["line"].as_u64()? as usize, l["addr"].as_u64()? as u16, l["len"].as_u64()? as usize))).collect()).unwrap_or_default(),
            zyklen: v["lines"].as_array().map(|a| a.iter().filter_map(|l| Some((l["file"].as_str()?.to_string(), l["line"].as_u64()? as usize, l["cyc"][0].as_u64()? as u8, l["cyc"][1].as_u64()? as u8))).collect()).unwrap_or_default(),
            segmente: v["segments"].as_array().map(|a| a.iter().filter_map(|s| Some((s["start"].as_u64()? as u32, s["length"].as_u64()? as usize))).collect()).unwrap_or_default(),
        })
    }
    /// Zyklenzahl je Zeile einer Datei: `[ [zeile, min, max] ]` (Zeile ab 1); Zeilen mit mehreren Anweisungen (Makros) summieren
    pub fn zyklen_json(&self, rel: &str) -> Value {
        let mut m: std::collections::BTreeMap<usize, (u32, u32)> = Default::default();
        for (f, z, a, b) in &self.zyklen { if f == rel { let e = m.entry(*z).or_default(); e.0 += *a as u32; e.1 += *b as u32; } }
        Value::Array(m.into_iter().map(|(z, (a, b))| json!([z, a, b])).collect())
    }
    /// Quelldatei und Zeile (ab 1) zu einer Adresse
    pub fn zeile_bei(&self, pc: u16) -> Option<(&str, usize)> { self.zeilen.iter().find(|(_, _, a, l)| pc >= *a && (pc as usize) < *a as usize + (*l).max(1)).map(|(f, n, _, _)| (f.as_str(), *n)) }
    /// Adresse zu Datei und Zeile (ab 1)
    pub fn adresse_von(&self, datei: &str, zeile: usize) -> Option<u16> { self.zeilen.iter().find(|(f, n, _, _)| f == datei && *n == zeile).map(|(_, _, a, _)| *a) }
    /// Marke (ohne Punkt, bei mehreren die alphabetisch erste) zu einer Adresse
    pub fn marke_bei(&self, a: u16) -> Option<&str> { self.symbols.iter().filter(|(n, v)| **v == a as i64 && !n.contains('.')).map(|(n, _)| n.as_str()).min() }
    /// Zusammenfassung für die Oberfläche
    pub fn json(&self) -> Value {
        json!({ "rel": self.rel, "target": self.target, "artifact": self.artifact.to_string_lossy(), "size": self.size, "run": self.run, "segments": self.segmente.iter().map(|(s, l)| json!([s, l])).collect::<Vec<_>>() })
    }
}

#[derive(Default)]
pub struct Bau { pub laeuft: AtomicBool, pub info: Mutex<Option<Info>>, pub ziel: Mutex<String>, /** Es läuft eine Starterkennung (nur eine) */ pub wacht: AtomicBool, /** Starten nacheinander */ pub starten: Mutex<()> }

impl Bau {
    /// Beginnt einen Bau; `false`, wenn schon einer läuft
    pub fn beginnen(&self) -> bool { !self.laeuft.swap(true, Ordering::SeqCst) }
    pub fn ende(&self) { self.laeuft.store(false, Ordering::SeqCst); }
    /// Führt den Bau-Thread-Rumpf mit Panikschutz aus: bei einer Panik endet der Bau (sonst bliebe er für immer „läuft“); `Some(Meldung)` für `build:done`
    pub fn geschuetzt(&self, f: impl FnOnce()) -> Option<String> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(()) => None,
            Err(p) => {
                self.ende();
                let was = p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned()).unwrap_or_default();
                Some(vc_i18n::t!("bau-abgestuerzt", was = was).to_string())
            }
        }
    }
}

/// Quelldatei zum Bauen: die gewünschte (aktive), sonst die Hauptquelle des Projekts; `None`, wenn keine in Frage kommt
pub fn quelle(pf: Option<&vc_ide::projfile::ProjectFile>, dateien: &[String], aktiv: Option<&str>, nur_aktiv: bool) -> Option<String> {
    // baubar: Assembler und natives BASIC (`.bas`, `.lst`, `.tur`); `.sbas` ist abgeschaltet
    let baubar = |s: &String| { let l = s.to_ascii_lowercase(); [".asm", ".bas", ".lst", ".tur"].iter().any(|e| l.ends_with(e)) };
    let asm = |s: &String| s.to_ascii_lowercase().ends_with(".asm");
    let aktiv = aktiv.map(String::from).filter(baubar);
    if nur_aktiv { return aktiv; }
    // natives BASIC: jede Datei ist ein eigenes Programm, gebaut wird die aktive (Assembler: Hauptquelle, die aktive Datei kann ein Include sein)
    if let Some(a) = aktiv.as_ref().filter(|a| !asm(a)) { return Some(a.clone()); }
    pf.and_then(|p| p.main.clone()).filter(|m| baubar(m) && dateien.contains(m)).or(aktiv)
        .or_else(|| dateien.iter().find(|d| asm(d) && !d.contains('/')).cloned())
        .or_else(|| dateien.iter().find(|d| baubar(d) && !d.contains('/')).cloned())
}

/// Kein Prozess starten (Tests, Prüfbrücke): `VC_TEST_NO_SPAWN`
pub fn gesperrt() -> bool { cfg!(test) || std::env::var_os("VC_TEST_NO_SPAWN").is_some() }

/// Artefakt im Ziel starten (Emulator des Systems oder eingerichtete Maschine `ziel`); liefert die Kommandozeile bzw. einen Hinweis
pub fn starten(root: &Path, data: &Path, info: &Info, ziel: &str) -> Result<String, String> {
    if gesperrt() {
        // Prüfbrücke: kein Emulator; mit `VC_TEST_FAKE_EMU=<Sekunden>` läuft stattdessen ein Platzhalterprozess, damit Ende und Beenden prüfbar sind
        #[cfg(windows)]
        if let Some(n) = std::env::var("VC_TEST_FAKE_EMU").ok().filter(|v| v.parse::<u32>().is_ok()) {
            vc_ide::launch::stop_running();
            let n: u32 = n.parse().unwrap_or(1);
            let c = std::process::Command::new("ping").args(["-n", &(n + 1).to_string(), "127.0.0.1"]).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).spawn().map_err(|e| e.to_string())?; vc_ide::launch::adopt(c);
        }
        #[cfg(unix)]
        if let Some(n) = std::env::var("VC_TEST_FAKE_EMU").ok().filter(|v| v.parse::<u32>().is_ok()) {
            vc_ide::launch::stop_running();
            let c = vc_ide::launch::isolate(std::process::Command::new("sleep").arg(n)).spawn().map_err(|e| e.to_string())?; vc_ide::launch::adopt(c);
        }
        return Ok(vc_i18n::t!("gui-testlauf-kein-emulator-gestartet").to_string());
    }
    let maschine = vc_ide::settings::effective().start_machine(ziel, &info.target).cloned();
    match maschine {
        Some(m) => vc_ide::launch::run_on_machine(&m, &info.artifact, &info.target),
        None => vc_ide::launch::spawn(&info.artifact, &info.target, &vc_ide::launch::atari_variant(root), &vc_ide::launch::project_args(data, root, &info.target)),
    }
}

/// Startziele des Systems: Emulator des Systems und eingerichtete Maschinen: `[ { id, name } ]`; `current` ist die Wahl
pub fn ziele(system: &str, wahl: &str) -> Value {
    let einst = vc_ide::settings::effective();
    let emu = { let k = einst.emulator_for(system).kind; if k.is_empty() { vc_i18n::t!("gui-emulator").to_string() } else { vc_ide::targets::kind_label(&k).to_string() } };
    let mut v = vec![json!({ "id": "@emulator", "name": emu })];
    for m in einst.machines_for(system) { v.push(json!({ "id": m.id, "name": m.display_name() })); }
    let name = einst.start_machine(wahl, system).map(|m| m.display_name().to_string()).unwrap_or_else(|| emu.clone());
    json!({ "system": vc_ide::targets::system(system).map(|s| s.name()).unwrap_or_else(|| system.to_uppercase()), "current": if wahl.is_empty() { "@emulator" } else { wahl }, "name": name, "targets": v })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn quelle_nimmt_aktive_oder_haupt_oder_erste_asm() {
        let d: Vec<String> = ["lib/x.inc", "a.asm", "b.asm", "n.txt"].iter().map(|s| s.to_string()).collect();
        assert_eq!(quelle(None, &d, Some("b.asm"), false).as_deref(), Some("b.asm"));
        assert_eq!(quelle(None, &d, Some("n.txt"), false).as_deref(), Some("a.asm"), "keine Quelle aktiv: erste im Projekt");
        assert_eq!(quelle(None, &d, Some("n.txt"), true), None, "aktive Datei verlangt, aber keine Quelle");
        assert_eq!(quelle(None, &[], None, false), None);
        // natives BASIC ist baubar; Assembler hat bei der Vorgabe ohne Hauptquelle Vorrang
        let b: Vec<String> = ["x.txt", "sterne.bas", "a.asm"].iter().map(|s| s.to_string()).collect();
        assert_eq!(quelle(None, &b, Some("sterne.bas"), true).as_deref(), Some("sterne.bas"));
        assert_eq!(quelle(None, &b, None, false).as_deref(), Some("a.asm"));
        assert_eq!(quelle(None, &["sterne.bas".to_string()], None, false).as_deref(), Some("sterne.bas"));
        // BASIC: auch ohne „nur aktive“ gewinnt die aktive Datei vor der Hauptquelle (F5/F7 bauen, was offen ist)
        let mut pf = vc_ide::projfile::ProjectFile::default(); pf.main = Some("sterne.bas".into());
        let d2: Vec<String> = ["sterne.bas", "pruefung.bas", "a.asm"].iter().map(|s| s.to_string()).collect();
        assert_eq!(quelle(Some(&pf), &d2, Some("pruefung.bas"), false).as_deref(), Some("pruefung.bas"));
        assert_eq!(quelle(Some(&pf), &d2, Some("a.asm"), false).as_deref(), Some("sterne.bas"));
    }
    #[test] fn info_aus_dem_bauergebnis() {
        let v = json!({ "ok": true, "file": "a.asm", "target": "c64", "artifact": "build/a.prg", "size": 12, "run": 2064, "symbols": [{ "name": "start", "value": 2064 }],
                        "lines": [{ "file": "a.asm", "line": 3, "addr": 2064, "len": 2 }], "segments": [{ "start": 2049, "length": 12 }] });
        let i = Info::aus(Path::new("/p"), &v).unwrap();
        assert_eq!((i.target.as_str(), i.run, i.symbols["start"], i.zeilen[0].1, i.segmente[0].1), ("c64", Some(2064), 2064, 3, 12)); assert_eq!(i.artifact, PathBuf::from("/p/build/a.prg"));
        assert!(Info::aus(Path::new("/p"), &json!({ "ok": false })).is_none());
    }
    #[test] fn zyklen_je_zeile_summieren_bei_makros() {
        let v = json!({ "ok": true, "file": "a.asm", "artifact": "build/a.prg", "lines": [
            { "file": "a.asm", "line": 3, "addr": 2064, "len": 2, "cyc": [2, 2] }, { "file": "a.asm", "line": 4, "addr": 2066, "len": 3, "cyc": [4, 5] }, { "file": "a.asm", "line": 4, "addr": 2069, "len": 1, "cyc": [2, 2] }, { "file": "a.asm", "line": 5, "addr": 2070, "len": 1, "cyc": null }] });
        let i = Info::aus(Path::new("/p"), &v).unwrap();
        assert_eq!(i.zyklen_json("a.asm"), json!([[3, 2, 2], [4, 6, 7]])); assert_eq!(i.zyklen_json("b.asm"), json!([]));
    }
    #[test] fn bauen_laeuft_nur_einmal() { let b = Bau::default(); assert!(b.beginnen()); assert!(!b.beginnen()); b.ende(); assert!(b.beginnen()); }
    #[test] fn panik_im_bau_beendet_den_bau_und_meldet_fehler() {
        let b = Bau::default(); assert!(b.beginnen());
        let m = b.geschuetzt(|| panic!("kaputte Daten")); // Panikmeldung im Testprotokoll ist erwartet
        assert!(m.unwrap().contains("kaputte Daten"));
        assert!(b.beginnen(), "nach der Panik kann wieder gebaut werden");
        assert!(b.geschuetzt(|| {}).is_none());
    }
    #[test] fn vergifteter_mutex_blockiert_nicht_fuer_immer() {
        let b = std::sync::Arc::new(Bau::default());
        let b2 = b.clone();
        let _ = std::thread::spawn(move || { let _g = b2.info.lock().unwrap(); panic!("im gesperrten Zustand"); }).join();
        assert!(b.info.is_poisoned());
        assert!(b.info.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_none());
    }
}
