// SPDX-License-Identifier: GPL-3.0-or-later
//! Debug-Protokoll: Schaltbare Protokolldatei, mit der sich ein Fehler später nachvollziehen lässt. Sie wird nur geschrieben, wenn der Anwender
//! sie einschaltet (Einstellungen ▸ Allgemein, Einstellung `debug_log`, Umgebungsvariable `VC_DEBUG_LOG`), liegt lokal im Konfigurationsordner
//! (`logs/vintagecoder.log`) und wird weder gesendet noch angehängt: Der Anwender hängt sie selbst an seine Feedback-Mail (siehe [`crate::feedback`]).
//!
//! **Inhalt:** Kopf mit den Angaben der Feedback-Mail, Befehle und Meldungen der IDE (Bau- und Startaufrufe samt Ausgang), Fehler und Abstürze
//! (Panic mit Rückverfolgung). Pfade im persönlichen Ordner stehen als `~`, Passwörter, Tokens und Schlüssel werden durch `***` ersetzt.
//! Meldungen des Projekts (Dateinamen, Fehlermeldungen des Assemblers) können vorkommen; deshalb vor dem Weitergeben ansehen.
//! **Format:** JSON Lines, eine Zeile je Ereignis, damit sich die Datei automatisch auswerten lässt (`vcc debuglog <datei>` fasst sie zusammen,
//! `--json` maschinenlesbar): `{"ts":"2026-11-12 15:55:01.123","seq":7,"kind":"befehl","msg":"Build"}`; die erste Zeile (`kind":"start"`) trägt
//! zusätzlich `info` mit den Angaben der Feedback-Mail. Arten: `start`, `stop`, `befehl`, `ausgabe`, `warnung`, `fehler`, `aufruf` (vcc), `absturz`.
//! **Größe:** Überschreitet die Datei [`MAX_BYTES`], wird sie zu `vintagecoder.log.1` (eine Vorgängerdatei) und neu begonnen.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

/// Höchstgröße der Protokolldatei vor dem Wechsel auf eine neue (5 MB)
pub const MAX_BYTES: u64 = 5 * 1024 * 1024;

static ON: AtomicBool = AtomicBool::new(false);
static SEQ: AtomicU64 = AtomicU64::new(0);
static OVERRIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

/// `VC_DEBUG_LOG`: `1`, `true`, `on`, `ja` schalten ein (Standardpfad), `0`, `false`, `off`, `nein` oder leer lassen die Einstellung gelten,
/// alles andere ist der Pfad der Protokolldatei (und schaltet ein)
pub fn parse_env(v: Option<&str>) -> (Option<bool>, Option<PathBuf>) {
    let v = v.map(str::trim).unwrap_or("");
    match v.to_ascii_lowercase().as_str() {
        "" => (None, None),
        "1" | "true" | "on" | "ja" => (Some(true), None),
        "0" | "false" | "off" | "nein" => (Some(false), None),
        _ => (Some(true), Some(PathBuf::from(v))),
    }
}

/// Beim Programmstart: Schalter aus Einstellung und Umgebung (die Umgebung hat Vorrang) übernehmen. Gibt zurück, ob protokolliert wird.
pub fn init(setting: bool) -> bool {
    let (on, path) = parse_env(std::env::var("VC_DEBUG_LOG").ok().as_deref());
    if let Ok(mut g) = OVERRIDE.lock() { *g = path; }
    set_enabled(on.unwrap_or(setting));
    enabled()
}

pub fn enabled() -> bool { ON.load(Ordering::Relaxed) }

/// Ein- oder ausschalten; beim Einschalten steht ein Kopf mit den Angaben der Installation in der Datei
pub fn set_enabled(on: bool) {
    let was = ON.swap(on, Ordering::Relaxed);
    if on && !was { let (subject, info) = header(); write_entry("start", &subject, Some(info)); }
    if !on && was { write_line("stop", "Protokoll ausgeschaltet"); }
}

/// Ordner der Protokolle: `logs` im Konfigurationsordner
pub fn dir() -> Option<PathBuf> { Some(vc_core::userdirs::config_dir()?.join("logs")) }

/// Pfad der Protokolldatei (`VC_DEBUG_LOG` mit Pfad, sonst `logs/vintagecoder.log`)
pub fn path() -> Option<PathBuf> {
    if let Some(p) = OVERRIDE.lock().ok().and_then(|g| g.clone()) { return Some(p); }
    Some(dir()?.join("vintagecoder.log"))
}

/// Betreff der Feedback-Mail und die Angaben der Installation (Pfade gekürzt)
fn header() -> (String, serde_json::Value) {
    let i = crate::feedback::Info::detect(None);
    let home = vc_core::userdirs::home_dir();
    let home = home.as_deref().and_then(Path::to_str);
    let m = crate::feedback::build(&i, home, usize::MAX);
    let mut info = m.to_json(&i)["info"].clone();
    redact_value(&mut info, home);
    (m.subject, info)
}

fn redact_value(v: &mut serde_json::Value, home: Option<&str>) {
    match v {
        serde_json::Value::String(s) => *s = redact(s, home),
        serde_json::Value::Array(a) => a.iter_mut().for_each(|x| redact_value(x, home)),
        serde_json::Value::Object(o) => o.values_mut().for_each(|x| redact_value(x, home)),
        _ => {}
    }
}

/// Zeile ins Protokoll (`kind`: Art, z. B. `befehl`, `bau`, `fehler`); tut nichts, solange das Protokoll aus ist. Fehler beim Schreiben werden verschluckt.
pub fn log(kind: &str, msg: &str) { if enabled() { write_line(kind, msg); } }

fn write_line(kind: &str, msg: &str) { write_entry(kind, msg, None); }

fn write_entry(kind: &str, msg: &str, info: Option<serde_json::Value>) {
    let Some(p) = path() else { return };
    let home = vc_core::userdirs::home_dir();
    let text = format_entry(&timestamp(), SEQ.fetch_add(1, Ordering::Relaxed), kind, &redact(msg, home.as_deref().and_then(Path::to_str)), info);
    let _ = append(&p, &text, MAX_BYTES);
}

/// Eintrag als eine JSON-Zeile (mit Zeilenende): `ts`, `seq`, `kind`, `msg`, bei `start` auch `info`
pub fn format_entry(ts: &str, seq: u64, kind: &str, msg: &str, info: Option<serde_json::Value>) -> String {
    let mut o = serde_json::json!({ "ts": ts, "seq": seq, "kind": kind, "msg": msg });
    if let Some(i) = info { o["info"] = i; }
    format!("{o}\n")
}

/// Ein ausgewerteter Eintrag
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry { pub ts: String, pub seq: u64, pub kind: String, pub msg: String }

/// Zusammenfassung einer Protokolldatei für die automatische Auswertung
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub entries: usize,
    /// Zeilen, die kein gültiger Eintrag waren (abgeschnittene oder fremde Zeilen)
    pub malformed: usize,
    pub by_kind: std::collections::BTreeMap<String, usize>,
    /// Subject und `info` des letzten `start`-Eintrags (Version, System, Installation)
    pub start: Option<(String, serde_json::Value)>,
    /// Fehler, Warnungen und Abstürze in Reihenfolge
    pub problems: Vec<Entry>,
    /// die letzten Ereignisse vor Ende der Datei (Weg zum Fehler)
    pub tail: Vec<Entry>,
    pub first_ts: String,
    pub last_ts: String,
}

/// Protokolltext auswerten; `tail`: wie viele letzte Ereignisse behalten werden
pub fn summarize(text: &str, tail: usize) -> Summary {
    let mut s = Summary::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { s.malformed += 1; continue };
        let (Some(ts), Some(kind)) = (v["ts"].as_str(), v["kind"].as_str()) else { s.malformed += 1; continue };
        let e = Entry { ts: ts.to_string(), seq: v["seq"].as_u64().unwrap_or(0), kind: kind.to_string(), msg: v["msg"].as_str().unwrap_or("").to_string() };
        s.entries += 1;
        *s.by_kind.entry(e.kind.clone()).or_default() += 1;
        if s.first_ts.is_empty() { s.first_ts = e.ts.clone(); }
        s.last_ts = e.ts.clone();
        if e.kind == "start" { s.start = Some((e.msg.clone(), v["info"].clone())); }
        if matches!(e.kind.as_str(), "fehler" | "warnung" | "absturz") { s.problems.push(e.clone()); }
        s.tail.push(e);
        if s.tail.len() > tail { s.tail.remove(0); }
    }
    s
}

impl Summary {
    /// Lesbare Fassung (Text für das Terminal)
    pub fn to_text(&self) -> String {
        let mut o = String::new();
        if let Some((subj, _)) = &self.start { o.push_str(&format!("Programm: {subj}\n")); }
        o.push_str(&format!("Einträge: {} ({} bis {}){}\n", self.entries, self.first_ts, self.last_ts, if self.malformed > 0 { format!(", {} unlesbare Zeilen", self.malformed) } else { String::new() }));
        o.push_str(&format!("Arten: {}\n", self.by_kind.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")));
        let crashes = self.problems.iter().filter(|e| e.kind == "absturz").count();
        o.push_str(&format!("Abstürze: {crashes}, Fehler: {}, Warnungen: {}\n", self.by_kind.get("fehler").copied().unwrap_or(0), self.by_kind.get("warnung").copied().unwrap_or(0)));
        if !self.problems.is_empty() { o.push_str("\nProbleme:\n"); for e in &self.problems { o.push_str(&format!("  #{} {} [{}] {}\n", e.seq, e.ts, e.kind, e.msg.lines().next().unwrap_or(""))); } }
        o.push_str("\nLetzte Ereignisse:\n");
        for e in &self.tail { o.push_str(&format!("  #{} {} [{}] {}\n", e.seq, e.ts, e.kind, e.msg.lines().next().unwrap_or(""))); }
        o
    }
    /// Maschinenlesbare Fassung
    pub fn to_json(&self) -> serde_json::Value {
        let ent = |e: &Entry| serde_json::json!({ "ts": e.ts, "seq": e.seq, "kind": e.kind, "msg": e.msg });
        serde_json::json!({
            "entries": self.entries, "malformed": self.malformed, "first": self.first_ts, "last": self.last_ts, "by_kind": self.by_kind,
            "start": self.start.as_ref().map(|(s, i)| serde_json::json!({ "subject": s, "info": i })),
            "problems": self.problems.iter().map(ent).collect::<Vec<_>>(), "tail": self.tail.iter().map(ent).collect::<Vec<_>>(),
        })
    }
}

/// Anhängen mit Wechsel: Ist die Datei mit dem Eintrag größer als `max`, wird sie zu `<name>.1` (die alte `.1` entfällt) und neu begonnen
pub fn append(path: &Path, text: &str, max: u64) -> std::io::Result<()> {
    if let Some(d) = path.parent() { std::fs::create_dir_all(d)?; }
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size > 0 && size + text.len() as u64 > max {
        let mut old = path.as_os_str().to_owned(); old.push(".1");
        let _ = std::fs::remove_file(&old);
        std::fs::rename(path, &old)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(text.as_bytes())
}

/// Zeitstempel in UTC: `2026-11-12 15:55:01.123`
pub fn timestamp() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    format_time(d.as_secs(), d.subsec_millis())
}

/// Sekunden seit 1970 (UTC) als Datum und Uhrzeit
pub fn format_time(secs: u64, ms: u32) -> String {
    let (days, rem) = (secs / 86400, secs % 86400);
    // bürgerliches Datum aus Tagen seit 1970 (Verfahren von H. Hinnant)
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}.{ms:03}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Schlüsselwörter, deren Wert (danach) geschwärzt wird
const SECRETS: [&str; 7] = ["password", "passwort", "passwd", "token", "secret", "apikey", "api-key"];

/// Text für das Protokoll bereinigen: persönlicher Ordner und Benutzerordner → `~`, Werte hinter Passwort-, Token- und Schlüsselwörtern → `***`
pub fn redact(text: &str, home: Option<&str>) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 { out.push('\n'); }
        out.push_str(&redact_secrets(&redact_paths(line, home)));
    }
    out
}

fn redact_paths(line: &str, home: Option<&str>) -> String {
    let mut s = line.to_string();
    if let Some(h) = home.map(|h| h.trim_end_matches(['/', '\\'])).filter(|h| h.len() > 1) { s = s.replace(h, "~"); }
    // übrige Benutzerordner: /Users/x, /home/x, C:\Users\x
    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    'outer: while !rest.is_empty() {
        for pre in ["/Users/", "/home/", "\\Users\\", "\\users\\"] {
            if let Some(tail) = rest.strip_prefix(pre) {
                let n = tail.find(['/', '\\', ' ', '"', '\'']).unwrap_or(tail.len());
                if n > 0 {
                    // nur am Anfang eines Pfades (nach Leerraum, Anführungszeichen, `=`, Laufwerk) ersetzen
                    let prev_ok = out.chars().last().map_or(true, |c| c.is_whitespace() || matches!(c, '"' | '\'' | '=' | ':' | '(' | '[' | ','));
                    if prev_ok || pre.starts_with('\\') { out.push('~'); rest = &tail[n..]; continue 'outer; }
                }
            }
        }
        let c = rest.chars().next().unwrap();
        out.push(c); rest = &rest[c.len_utf8()..];
    }
    // `C:` vor `\Users\x` bleibt sonst stehen: „C:~“ → „~“
    out.replace("C:~", "~").replace("c:~", "~")
}

fn redact_secrets(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let mut out = String::new();
    let mut i = 0;
    while i < line.len() {
        let hit = SECRETS.iter().find(|k| lower[i..].starts_with(*k));
        if let Some(k) = hit {
            let key_end = i + k.len();
            out.push_str(&line[i..key_end]);
            let b = line.as_bytes();
            let mut j = key_end;
            while j < b.len() && matches!(b[j], b' ' | b'=' | b':' | b'"' | b'\'') { j += 1; }
            if j > key_end && j < b.len() {
                out.push_str(&line[key_end..j]);
                let mut e = j;
                while e < b.len() && !matches!(b[e], b' ' | b'"' | b'\'' | b',' | b'}' | b'&') { e += 1; }
                out.push_str("***");
                i = e;
                continue;
            }
            i = key_end;
            continue;
        }
        let c = line[i..].chars().next().unwrap();
        out.push(c); i += c.len_utf8();
    }
    out
}

/// Absturzbericht ins Protokoll: Meldung, Ort und Rückverfolgung; ruft danach den bisherigen Haken auf. Ohne eingeschaltetes Protokoll geschieht nichts Zusätzliches.
pub fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if enabled() {
            let bt = std::backtrace::Backtrace::force_capture();
            write_line("absturz", &format!("{info}\n{bt}"));
        }
        prev(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-debuglog-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); d }

    #[test]
    fn umgebungsvariable() {
        assert_eq!(parse_env(None), (None, None));
        assert_eq!(parse_env(Some("")), (None, None));
        assert_eq!(parse_env(Some("1")), (Some(true), None));
        assert_eq!(parse_env(Some("OFF")), (Some(false), None));
        assert_eq!(parse_env(Some("/tmp/x.log")), (Some(true), Some(PathBuf::from("/tmp/x.log"))));
    }

    #[test]
    fn zeitstempel_und_eintrag() {
        assert_eq!(format_time(0, 0), "1970-01-01 00:00:00.000");
        assert_eq!(format_time(1_781_192_101, 123), "2026-06-11 15:35:01.123");
        assert_eq!(format_time(951_782_400, 5), "2000-02-29 00:00:00.005", "Schaltjahr");
        let l = format_entry("T", 3, "befehl", "eins\nzwei \"x\"", None);
        assert!(l.ends_with('\n') && l.matches('\n').count() == 1, "genau eine Zeile");
        let v: serde_json::Value = serde_json::from_str(l.trim()).unwrap();
        assert_eq!((v["ts"].as_str(), v["seq"].as_u64(), v["kind"].as_str(), v["msg"].as_str()), (Some("T"), Some(3), Some("befehl"), Some("eins\nzwei \"x\"")));
    }

    #[test]
    fn pfade_werden_gekuerzt() {
        assert_eq!(redact("lade /Users/anna/proj/a.asm", Some("/Users/anna")), "lade ~/proj/a.asm");
        assert_eq!(redact("x=/home/bob/.config/vintagecoder y", None), "x=~/.config/vintagecoder y");
        assert_eq!(redact(r"C:\Users\Bob\AppData\x", None), r"~\AppData\x");
        assert_eq!(redact("/opt/vice/bin/x64sc /Applications/V.app", Some("/Users/anna")), "/opt/vice/bin/x64sc /Applications/V.app");
        assert_eq!(redact("kein /Usersx/y", None), "kein /Usersx/y");
        assert_eq!(redact("Zeile 1\n/Users/a/b", None), "Zeile 1\n~/b");
    }

    #[test]
    fn zugangsdaten_werden_geschwaerzt() {
        assert_eq!(redact("--password geheim --host u64", None), "--password *** --host u64");
        assert_eq!(redact(r#"{"password":"geheim","host":"x"}"#, None), r#"{"password":"***","host":"x"}"#);
        assert_eq!(redact("Token=abc123&x=1", None), "Token=***&x=1");
        assert_eq!(redact("api-key: XYZ", None), "api-key: ***");
        assert_eq!(redact("Passwort leer", None), "Passwort ***", "ein Wort nach dem Schlüsselwort gilt als Wert");
        assert_eq!(redact("passwords", None), "passwords");
    }

    #[test]
    fn rotation() {
        let d = tmp("rot"); let p = d.join("logs").join("v.log");
        for i in 0..10 { append(&p, &format!("zeile {i:02}\n"), 30).unwrap(); }
        let alt = d.join("logs").join("v.log.1");
        let (neu, vor) = (std::fs::read_to_string(&p).unwrap(), std::fs::read_to_string(&alt).unwrap());
        assert!(neu.len() <= 30 && vor.len() <= 30 && !neu.is_empty() && !vor.is_empty());
        assert!(neu.contains("zeile 09") && !d.join("logs").join("v.log.2").exists(), "nur eine Vorgängerdatei");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn schalter_schreibt_kopf_und_zeilen() {
        let d = tmp("on"); let p = d.join("l.log");
        std::env::set_var("VC_DEBUG_LOG", p.to_str().unwrap());
        assert!(init(false) && enabled());
        log("befehl", "Build");
        set_enabled(false);
        log("befehl", "wird nicht geschrieben");
        std::env::remove_var("VC_DEBUG_LOG");
        let t = std::fs::read_to_string(&p).unwrap();
        assert!(!t.contains("wird nicht geschrieben"));
        let s = summarize(&t, 10);
        assert_eq!(s.malformed, 0);
        let (subj, info) = s.start.clone().expect("Kopf");
        assert!(subj.starts_with("VintageCoder 0.1.0") && info["version"] == "0.1.0" && info["package"].is_string());
        assert_eq!(s.by_kind.get("befehl"), Some(&1));
        assert!(s.tail.last().unwrap().kind == "stop" && s.tail[1].msg == "Build");
        if let Ok(mut g) = OVERRIDE.lock() { *g = None; }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn auswertung_findet_probleme_und_ende() {
        let mut t = String::new();
        for (i, (k, m)) in [("start", "VintageCoder 0.1.0"), ("befehl", "Build"), ("ausgabe", "gebaut"), ("fehler", "Zeile 5: unbekannter Befehl\nzweite Zeile"), ("befehl", "Run"), ("absturz", "panicked at x.rs:1")].iter().enumerate() {
            t.push_str(&format_entry(&format_time(1_781_192_101 + i as u64, 0), i as u64, k, m, None));
        }
        t.push_str("{abgeschnitten\n");
        let s = summarize(&t, 3);
        assert_eq!((s.entries, s.malformed), (6, 1));
        assert_eq!(s.problems.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(), ["fehler", "absturz"]);
        assert_eq!(s.tail.iter().map(|e| e.msg.as_str()).collect::<Vec<_>>(), ["Zeile 5: unbekannter Befehl\nzweite Zeile", "Run", "panicked at x.rs:1"]);
        let txt = s.to_text();
        assert!(txt.contains("Abstürze: 1, Fehler: 1") && txt.contains("1 unlesbare Zeilen") && txt.contains("#3 ") && txt.contains("[absturz]"));
        let j = s.to_json();
        assert_eq!(j["by_kind"]["befehl"], 2); assert_eq!(j["problems"][1]["kind"], "absturz");
    }
}
