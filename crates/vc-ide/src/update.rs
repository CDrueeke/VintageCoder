// SPDX-License-Identifier: GPL-3.0-or-later
//! Update-Prüfung, erste Auslieferung: **nur suchen, Änderungen zeigen, Link anbieten.** Es wird nichts geladen und nichts installiert.
//!
//! **Datenschutz:** Ohne Zustimmung geht nie etwas ins Netz ([`Mode`]); der Standard ist „noch nicht gefragt“. Die einzige Verbindung ist eine Anfrage
//! nach der kleinen Versionsliste `update.json` (fester Release `update-feed` im öffentlichen Repository, `curl`, nur `https`). Gesendet werden Adresse, IP-Adresse
//! und der neutrale `User-Agent` [`USER_AGENT`] (ohne Versionsnummer), keine Kennung, kein Rechnername, keine Angaben zu Projekten. Jede Prüfung steht im
//! Prüfprotokoll (`updates/log.txt` im Konfigurationsordner). Pakete ohne `IDE-UPDATE` (siehe [`crate::features`]) haben weder Oberfläche noch Verbindung.
//!
//! **Sicherheit:** Die Liste ist in dieser Fassung nicht signiert (das Format bleibt nachrüstbar). Deshalb darf sie nichts bewirken außer einer Anzeige:
//! Der Link wird **nie** aus der Liste übernommen, sondern hier aus einer festen Basis und einer streng geprüften Versionsnummer gebildet ([`release_url`]);
//! die Änderungstexte werden als reiner Text ohne klickbare Links und ohne Bilder dargestellt ([`plain_notes`]); Größen und Anzahlen sind begrenzt.
//! Eine gefälschte Liste kann damit höchstens falschen Text oder eine falsche Versionsnummer zeigen.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Feste Adresse der Versionsliste (Anhang des Release `update-feed`, siehe `docs/update-feed.md`)
pub const FEED_URL: &str = "https://github.com/CDrueeke/VintageCoder/releases/download/update-feed/update.json";
/// Feste Basis des Links zur Releaseseite; angehängt wird die geprüfte Versionsnummer
pub const TAG_URL_BASE: &str = "https://github.com/CDrueeke/VintageCoder/releases/tag/v";
/// Rückfall, wenn keine gültige Versionsnummer vorliegt
pub const RELEASES_URL: &str = "https://github.com/CDrueeke/VintageCoder/releases";
/// Neutral, ohne Versionsnummer
pub const USER_AGENT: &str = "VintageCoder-update";
/// Größte gelesene Versionsliste (Bytes)
pub const MAX_FEED_BYTES: usize = 256 * 1024;
/// Höchstes bekanntes Format der Liste
pub const MANIFEST_FORMAT: u64 = 1;
/// Projektformat (`"format"` in den Projektdateien), das diese Version schreibt; die Liste warnt, wenn die neue Version ein höheres schreibt
pub const PROJECT_FORMAT: u32 = 1;
const MAX_RELEASES: usize = 50;
const MAX_NOTES: usize = 16 * 1024;
const MAX_NOTES_TOTAL: usize = 128 * 1024;
const MAX_TITLE: usize = 120;
const MAX_YANKED: usize = 200;
/// „Später erinnern“ blendet den Hinweis so lange aus (Sekunden)
pub const SNOOZE_SECS: u64 = 3 * 24 * 3600;

// ---------------------------------------------------------------- Version und Link

/// `MAJOR.MINOR.PATCH`, genau drei Zahlen
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version(pub u32, pub u32, pub u32);
impl From<Version> for vc_i18n::Arg { fn from(v: Version) -> vc_i18n::Arg { vc_i18n::Arg::Text(v.to_string()) } }

impl Version {
    /// Streng: drei Zahlen mit je 1–4 Ziffern, keine führenden Nullen (außer `0`), sonst nichts (kein `v`, keine Vorabkennung, keine Leerzeichen)
    pub fn parse(s: &str) -> Option<Version> {
        let mut it = s.split('.');
        let mut n = [0u32; 3];
        for slot in n.iter_mut() {
            let p = it.next()?;
            if p.is_empty() || p.len() > 4 || !p.bytes().all(|b| b.is_ascii_digit()) || (p.len() > 1 && p.starts_with('0')) { return None; }
            *slot = p.parse().ok()?;
        }
        if it.next().is_some() { return None; }
        Some(Version(n[0], n[1], n[2]))
    }
    /// Die Version dieses Programms (aus Cargo; ist sie unlesbar, 0.0.0)
    pub fn installed() -> Version { Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_default() }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}.{}.{}", self.0, self.1, self.2) }
}

/// Link zur Releaseseite: feste Basis plus Version; ohne Version die feste Seite aller Releases. Nie eine Adresse aus der Liste.
pub fn release_url(v: Option<Version>) -> String { match v { Some(v) => format!("{TAG_URL_BASE}{v}"), None => RELEASES_URL.to_string() } }

// ---------------------------------------------------------------- Liste (update.json)

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release { pub version: Version, pub title: String, pub notes: String }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub sequence: u64,
    pub published: String,
    pub latest: Version,
    pub project_format: Option<u32>,
    pub yanked: Vec<Version>,
    /// neueste zuerst
    pub releases: Vec<Release>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// `format` ist neuer, als dieses Programm kennt: es wird nichts angezeigt
    TooNew(u64),
    Invalid(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { ParseError::TooNew(n) => write!(f, "{}", vc_i18n::t!("update-die-versionsliste-hat-das-format-n", n = n, max = MANIFEST_FORMAT)), ParseError::Invalid(m) => write!(f, "{}", vc_i18n::t!("update-die-versionsliste-ist-ungueltig-m", m = m)) }
    }
}

fn bad<T>(m: impl Into<String>) -> Result<T, ParseError> { Err(ParseError::Invalid(m.into())) }

/// Ein Text aus der Liste: Steuerzeichen (außer Zeilenumbruch und Tabulator) fallen weg, Zeilenenden werden `\n`
fn clean(s: &str) -> String { s.replace("\r\n", "\n").chars().filter(|c| *c == '\n' || *c == '\t' || !c.is_control()).collect() }

impl Manifest {
    /// Liest und prüft die Liste streng. Unbekannte Felder (z. B. eine spätere Signatur) werden ignoriert; Adressen werden nie gelesen.
    pub fn parse(text: &str) -> Result<Manifest, ParseError> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| ParseError::Invalid(vc_i18n::t!("update-kein-gueltiges-json-e", e = e.to_string())))?;
        let Some(o) = v.as_object() else { return bad(vc_i18n::t!("update-kein-objekt")) };
        let format = o.get("format").and_then(|x| x.as_u64()).ok_or_else(|| ParseError::Invalid(vc_i18n::t!("update-format-fehlt").into()))?;
        if format > MANIFEST_FORMAT { return Err(ParseError::TooNew(format)); }
        if format == 0 { return bad(vc_i18n::t!("update-format-0")); }
        if o.get("channel").and_then(|x| x.as_str()) != Some("stable") { return bad(vc_i18n::t!("update-channel-ist-nicht-stable")); }
        let sequence = o.get("sequence").and_then(|x| x.as_u64()).unwrap_or(0);
        let published = clean(o.get("published").and_then(|x| x.as_str()).unwrap_or("")).chars().take(40).collect::<String>();
        let latest = o.get("latest").and_then(|x| x.as_str()).and_then(Version::parse).ok_or_else(|| ParseError::Invalid(vc_i18n::t!("update-latest-fehlt-oder-ist-keine-version").into()))?;
        let project_format = match o.get("project_format") { None | Some(serde_json::Value::Null) => None, Some(x) => match x.as_u64() { Some(n) if n <= 1000 => Some(n as u32), _ => return bad("project_format") } };
        let mut yanked = vec![];
        if let Some(a) = o.get("yanked") {
            let Some(a) = a.as_array() else { return bad(vc_i18n::t!("update-yanked-ist-keine-liste")) };
            if a.len() > MAX_YANKED { return bad(vc_i18n::t!("update-yanked-ist-zu-lang")); }
            for x in a { yanked.push(x.as_str().and_then(Version::parse).ok_or_else(|| ParseError::Invalid(vc_i18n::t!("update-yanked-ungueltige-version").into()))?); }
        }
        let Some(arr) = o.get("releases").and_then(|x| x.as_array()) else { return bad(vc_i18n::t!("update-releases-fehlt")) };
        if arr.is_empty() || arr.len() > MAX_RELEASES { return bad(vc_i18n::t!("update-releases-1-bis-max-releases-eintraege", MAX_RELEASES = MAX_RELEASES)); }
        let mut releases = vec![]; let mut total = 0usize;
        for r in arr {
            let Some(r) = r.as_object() else { return bad(vc_i18n::t!("update-releases-eintrag-ist-kein-objekt")) };
            let version = r.get("version").and_then(|x| x.as_str()).and_then(Version::parse).ok_or_else(|| ParseError::Invalid(vc_i18n::t!("update-releases-ungueltige-version").into()))?;
            let notes = clean(r.get("notes").and_then(|x| x.as_str()).unwrap_or(""));
            if notes.len() > MAX_NOTES { return bad(vc_i18n::t!("update-text-zu-version-ist-zu-lang", version = &version)); }
            total += notes.len();
            if total > MAX_NOTES_TOTAL { return bad(vc_i18n::t!("update-texte-insgesamt-zu-lang")); }
            let title: String = clean(r.get("title").and_then(|x| x.as_str()).unwrap_or("")).lines().next().unwrap_or("").chars().take(MAX_TITLE).collect();
            if releases.iter().any(|x: &Release| x.version == version) { return bad(vc_i18n::t!("update-version-version-kommt-doppelt-vor", version = &version)); }
            releases.push(Release { version, title, notes });
        }
        if !releases.iter().any(|r| r.version == latest) { return bad(vc_i18n::t!("update-latest-steht-nicht-in-releases")); }
        releases.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(Manifest { sequence, published, latest, project_format, yanked, releases })
    }
}

// ---------------------------------------------------------------- Auswertung

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Update {
    pub version: Version,
    /// Link zur Releaseseite (aus fester Basis, siehe [`release_url`])
    pub url: String,
    /// Änderungen aller neueren Versionen als Markdown, bereinigt ([`plain_notes`])
    pub notes: String,
    /// Die neue Version schreibt ein höheres Projektformat: ältere Versionen können die Projekte danach nicht mehr lesen
    pub project_format_warning: bool,
    pub published: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub update: Option<Update>,
    /// Die installierte Version wurde zurückgezogen
    pub installed_yanked: bool,
    pub latest: Version,
    pub sequence: u64,
}

/// Wertet die Liste für die installierte Version aus. Angeboten wird die höchste nicht zurückgezogene Version, wenn sie **größer** ist als die installierte
/// (nie ein Downgrade).
pub fn evaluate(m: &Manifest, installed: Version) -> Evaluation {
    let ok = |v: &Version| !m.yanked.contains(v);
    let cand = if ok(&m.latest) { Some(m.latest) } else { m.releases.iter().map(|r| r.version).filter(ok).max() };
    let update = cand.filter(|c| *c > installed).map(|c| {
        let mut notes = String::new();
        for r in m.releases.iter().filter(|r| r.version > installed && r.version <= c) {
            let rest = r.title.strip_prefix(&r.version.to_string()).unwrap_or(&r.title).trim();
            notes.push_str(&format!("## {}\n\n{}\n\n", vc_i18n::t!("update-notes-kopf", p1 = &r.version, p2 = if rest.is_empty() { String::new() } else { format!(" {rest}") }), plain_notes(&r.notes)));
        }
        Update { version: c, url: release_url(Some(c)), notes: notes.trim_end().to_string(), project_format_warning: m.project_format.map_or(false, |f| f > PROJECT_FORMAT), published: m.published.clone() }
    });
    Evaluation { update, installed_yanked: m.yanked.contains(&installed), latest: m.latest, sequence: m.sequence }
}

/// Änderungstext als reinen Text darstellbar machen: Bilder und Links verlieren ihre Verweise (`[Text](Adresse)` wird zu `Text`, bei Webadressen zu
/// `Text (Adresse)` als bloßer Text), rohes HTML bleibt Text. Der Markdown-Renderer der Hilfe macht daraus keine klickbaren Elemente.
pub fn plain_notes(md: &str) -> String {
    let mut s = clean(md);
    for _ in 0..4 {
        let t = strip_links(&s);
        if t == s { break; }
        s = t;
    }
    // übrig gebliebene Bildmarken und Verweisformen entschärfen
    s.replace("](", "] (").replace("<http", "‹http").replace("<mailto", "‹mailto")
}

fn strip_links(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let (mut out, mut i) = (String::new(), 0);
    while i < b.len() {
        let img = b[i] == '!' && b.get(i + 1) == Some(&'[');
        if b[i] == '[' || img {
            let start = if img { i + 1 } else { i };
            // passende ] mit höchstens einer Verschachtelung innerhalb einer Zeile suchen
            let mut depth = 0; let mut j = start; let mut close = None;
            while j < b.len() && b[j] != '\n' {
                if b[j] == '[' { depth += 1; } else if b[j] == ']' { depth -= 1; if depth == 0 { close = Some(j); break; } }
                j += 1;
            }
            if let Some(c) = close { if b.get(c + 1) == Some(&'(') {
                let mut k = c + 2; let mut d = 1; let mut end = None;
                while k < b.len() && b[k] != '\n' { if b[k] == '(' { d += 1; } else if b[k] == ')' { d -= 1; if d == 0 { end = Some(k); break; } } k += 1; }
                if let Some(e) = end {
                    let text: String = b[start + 1..c].iter().collect();
                    let url: String = b[c + 2..e].iter().collect::<String>().trim().to_string();
                    let shown = url.split_whitespace().next().unwrap_or("").to_string();
                    out.push_str(&text);
                    if !img && (shown.starts_with("http://") || shown.starts_with("https://")) { out.push_str(&format!(" {}", vc_i18n::t!("update-shown", shown = &shown))); }
                    i = e + 1;
                    continue;
                }
            } }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

// ---------------------------------------------------------------- Einstellungen, Zustimmung, Zeitplan

/// Wahl des Anwenders zur automatischen Suche
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// noch nicht gefragt: es wird nichts gesendet (nur „Jetzt prüfen“)
    Ask,
    /// nur auf Wunsch
    Never,
    Daily,
    Weekly,
}

impl Mode {
    pub fn from_setting(s: &str) -> Mode { match s { "daily" => Mode::Daily, "weekly" => Mode::Weekly, "never" => Mode::Never, _ => Mode::Ask } }
    pub fn setting(self) -> &'static str { match self { Mode::Ask => "", Mode::Never => "never", Mode::Daily => "daily", Mode::Weekly => "weekly" } }
    fn interval(self) -> Option<u64> { match self { Mode::Daily => Some(24 * 3600), Mode::Weekly => Some(7 * 24 * 3600), _ => None } }
}

/// Wartezeit nach `fails` Fehlversuchen in Folge: 6 Stunden, verdoppelt, höchstens 7 Tage
pub fn backoff_secs(fails: u32) -> u64 { if fails == 0 { 0 } else { (6 * 3600u64).saturating_mul(1u64 << (fails - 1).min(10)).min(7 * 24 * 3600) } }

/// Zustand zwischen den Läufen (`updates/state.json`; getrennt von den Einstellungen, damit nur Entscheidungen in `settings.json` stehen)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// letzte erfolgreiche Prüfung (Unix-Sekunden, 0 = nie)
    #[serde(default)] pub last_check: u64,
    /// letzter Versuch, auch ein fehlgeschlagener
    #[serde(default)] pub last_attempt: u64,
    /// Fehlversuche in Folge
    #[serde(default)] pub fails: u32,
    /// neueste gemeldete Version (nur Anzeige)
    #[serde(default)] pub last_seen: String,
    /// bis wann der Hinweis nach „Später erinnern“ ruht
    #[serde(default)] pub snooze_until: u64,
    /// Ergebnis des letzten Versuchs in Worten (Über-Dialog)
    #[serde(default)] pub last_result: String,
}

/// Ist eine automatische Prüfung fällig? Nur bei Daily/Weekly; `Ask` und `Never` nie.
pub fn is_due(mode: Mode, st: &State, now: u64) -> bool {
    let Some(iv) = mode.interval() else { return false };
    now >= st.last_check.saturating_add(iv) && now >= st.last_attempt.saturating_add(backoff_secs(st.fails))
}

/// Darf das Programm beim Start von selbst nachsehen? Release mit Update-Prüfung, Zustimmung erteilt, fällig; nie bei `VC_SHOT` (Bildschirmfotos).
pub fn auto_check_allowed(features: &crate::features::Features, prefs: &crate::settings::UpdatePrefs, st: &State, now: u64) -> bool {
    features.update() && std::env::var_os("VC_SHOT").is_none() && is_due(Mode::from_setting(&prefs.check), st, now)
}

/// Ort von Zustand und Protokoll: `VC_UPDATE_DIR`, sonst `updates` im Konfigurationsordner
pub fn dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("VC_UPDATE_DIR").filter(|d| !d.is_empty()) { return Some(d.into()); }
    Some(vc_core::userdirs::config_dir()?.join("updates"))
}

impl State {
    pub fn load() -> State { dir().and_then(|d| std::fs::read_to_string(d.join("state.json")).ok()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default() }
    pub fn save(&self) -> Result<(), String> {
        let d = dir().ok_or(vc_i18n::t!("update-kein-ordner-fuer-die-update-pruefung"))?;
        std::fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        let mut t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?; t.push('\n');
        crate::settings::write_atomic(&d.join("state.json"), t.as_bytes())
    }
}

/// Aktuelle Zeit in Unix-Sekunden
pub fn now_secs() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

/// Unix-Sekunden als `JJJJ-MM-TT hh:mm` in UTC (ohne Zeitzonenbibliothek)
pub fn format_time(secs: u64) -> String {
    let days = (secs / 86400) as i64; let rem = secs % 86400;
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153; let d = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    vc_i18n::t!("update-y-m-d-p4-p5", y = format!("{:04}", y), m = format!("{:02}", m), d = format!("{:02}", d), p4 = format!("{:02}", rem / 3600), p5 = format!("{:02}", (rem % 3600) / 60))
}

// ---------------------------------------------------------------- Abruf

/// Warum ein Abruf scheiterte
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchError { Offline, Timeout, Http(u16), Tls, NoCurl, Insecure, TooBig, Other(String) }

impl FetchError {
    /// Kurzer Text für Anwender
    pub fn message(&self) -> String {
        match self {
            FetchError::Offline => vc_i18n::t!("update-keine-verbindung-kein-netz-dns-oder").into(),
            FetchError::Timeout => vc_i18n::t!("update-zeitueberschreitung").into(),
            FetchError::Http(404) => vc_i18n::t!("update-keine-versionsliste-unter-dieser-adresse-404").into(),
            FetchError::Http(403) | FetchError::Http(429) => vc_i18n::t!("update-github-lehnt-die-anfrage-voruebergehend-ab", p1 = match self { FetchError::Http(c) => *c, _ => 0 }),
            FetchError::Http(c) => vc_i18n::t!("update-der-server-antwortet-mit-fehler-c", c = &c),
            FetchError::Tls => vc_i18n::t!("update-sichere-verbindung-nicht-moeglich-zertifikat-oder").into(),
            FetchError::NoCurl => vc_i18n::t!("update-das-programm-curl-fehlt").into(),
            FetchError::Insecure => vc_i18n::t!("update-nur-https-ist-erlaubt").into(),
            FetchError::TooBig => vc_i18n::t!("update-die-versionsliste-ist-zu-gross").into(),
            FetchError::Other(m) => m.clone(),
        }
    }
}

/// Ist die Quelle eine Webadresse (braucht Netz und Zustimmung) und nicht ein Ordner oder eine Datei?
pub fn is_remote(src: &str) -> bool { src.starts_with("https://") || src.starts_with("http://") }

/// Wirksame Quelle der Liste: `VC_UPDATE_URL`, dann die Einstellung, dann [`FEED_URL`]
pub fn source() -> String { resolve_source(std::env::var("VC_UPDATE_URL").ok().as_deref(), &crate::settings::get().updates.manifest_url) }

pub fn resolve_source(env: Option<&str>, setting: &str) -> String {
    env.map(str::trim).filter(|e| !e.is_empty()).map(String::from).unwrap_or_else(|| { let s = setting.trim(); if s.is_empty() { FEED_URL.to_string() } else { s.to_string() } })
}

fn read_local(src: &str) -> Result<String, FetchError> {
    let p = Path::new(src);
    let f = if p.is_dir() { p.join("update.json") } else { p.to_path_buf() };
    let d = std::fs::read(&f).map_err(|e| FetchError::Other(format!("{}: {e}", f.display())))?;
    if d.len() > MAX_FEED_BYTES { return Err(FetchError::TooBig); }
    String::from_utf8(d).map_err(|_| FetchError::Other(vc_i18n::t!("update-keine-gueltige-utf-8-datei").into()))
}

/// Liest die Liste: Ordner oder Datei lokal, Webadresse mit `curl`. `curl -q` ignoriert eine eigene `.curlrc`; nur `https` (auch bei Weiterleitungen),
/// begrenzte Größe und Zeit, keine Cookies, keine Zugangsdaten, nur der neutrale `User-Agent`.
pub fn fetch_text(src: &str) -> Result<String, FetchError> {
    if !is_remote(src) { return read_local(src); }
    if !src.starts_with("https://") { return Err(FetchError::Insecure); }
    let out = Command::new(crate::help_images::curl())
        .args(["-q", "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--proto-redir", "=https", "--max-filesize", &MAX_FEED_BYTES.to_string(),
               "--max-time", "15", "--connect-timeout", "8", "--user-agent", USER_AGENT, "--output", "-", "--", src])
        .stdin(Stdio::null()).output().map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { FetchError::NoCurl } else { FetchError::Other(vc_i18n::t!("update-curl-nicht-ausfuehrbar-e", e = &e)) })?;
    if !out.status.success() { return Err(curl_error(out.status.code(), &String::from_utf8_lossy(&out.stderr))); }
    if out.stdout.len() > MAX_FEED_BYTES { return Err(FetchError::TooBig); }
    String::from_utf8(out.stdout).map_err(|_| FetchError::Other(vc_i18n::t!("update-keine-gueltige-utf-8-datei").into()))
}

/// Abbildung der Rückgabewerte von `curl` auf [`FetchError`]
pub fn curl_error(code: Option<i32>, stderr: &str) -> FetchError {
    match code {
        Some(5) | Some(6) | Some(7) | Some(52) | Some(55) | Some(56) => FetchError::Offline,
        Some(28) => FetchError::Timeout,
        Some(35) | Some(51) | Some(58) | Some(59) | Some(60) | Some(77) | Some(83) | Some(91) => FetchError::Tls,
        Some(63) => FetchError::TooBig,
        Some(22) => match stderr.rsplit("error: ").next().and_then(|t| t.trim().split(|c: char| !c.is_ascii_digit()).next()).and_then(|n| n.parse::<u16>().ok()) { Some(c) => FetchError::Http(c), None => FetchError::Other(vc_i18n::t!("update-http-fehler").into()) },
        _ => FetchError::Other(stderr.lines().last().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).unwrap_or_else(|| vc_i18n::t!("update-abruf-fehlgeschlagen").into())),
    }
}

// ---------------------------------------------------------------- Prüfung

/// Ergebnis einer Prüfung
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report { pub at: u64, pub source: String, pub result: Result<Evaluation, String>, pub raw: Option<String> }

/// Ruft die Liste ab und wertet sie aus. Schreibt nichts (siehe [`record`]).
pub fn check(src: &str, installed: Version, now: u64) -> Report {
    let result = fetch_text(src).map_err(|e| e.message()).and_then(|t| Manifest::parse(&t).map(|m| (m, t)).map_err(|e| e.to_string()));
    match result {
        Ok((m, raw)) => Report { at: now, source: src.to_string(), result: Ok(evaluate(&m, installed)), raw: Some(raw) },
        Err(e) => Report { at: now, source: src.to_string(), result: Err(e), raw: None },
    }
}

/// Prüfung im Hintergrund; das Ergebnis kommt über den Kanal
pub fn spawn_check(src: String, installed: Version) -> std::sync::mpsc::Receiver<Report> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(check(&src, installed, now_secs())); });
    rx
}

/// Zustand fortschreiben, die geprüfte Liste für den Neustart merken und eine Zeile ins Prüfprotokoll schreiben. Fehler beim Schreiben bleiben still
/// (die Prüfung selbst ist dann trotzdem gelungen).
pub fn record(report: &Report, st: &mut State) {
    st.last_attempt = report.at;
    match &report.result {
        Ok(ev) => {
            st.last_check = report.at; st.fails = 0; st.last_seen = ev.latest.to_string();
            st.last_result = match &ev.update { Some(u) => vc_i18n::t!("update-version-p1-verfuegbar", p1 = &u.version), None => vc_i18n::t!("update-aktuell") };
            if let (Some(d), Some(raw)) = (dir(), &report.raw) { let _ = std::fs::create_dir_all(&d); let _ = crate::settings::write_atomic(&d.join("last-manifest.json"), raw.as_bytes()); }
        }
        Err(e) => { st.fails = st.fails.saturating_add(1); st.last_result = vc_i18n::t!("update-fehler-e", e = &e); }
    }
    let _ = st.save();
    log_line(report.at, &report.source, &st.last_result);
}

/// Das zuletzt erfolgreich gelesene Ergebnis, neu ausgewertet für die installierte Version (so verschwindet der Hinweis nach dem Update von selbst)
pub fn cached(installed: Version) -> Option<Evaluation> {
    let t = std::fs::read_to_string(dir()?.join("last-manifest.json")).ok()?;
    Manifest::parse(&t).ok().map(|m| evaluate(&m, installed))
}

/// Soll der Hinweis in der Statusleiste erscheinen? Nicht, wenn die Version übersprungen wurde oder der Hinweis ruht.
pub fn badge_visible(ev: &Evaluation, prefs: &crate::settings::UpdatePrefs, st: &State, now: u64) -> bool {
    match &ev.update { Some(u) => prefs.skipped != u.version.to_string() && now >= st.snooze_until, None => false }
}

const LOG_LINES: usize = 20;

/// Eine Zeile an das Prüfprotokoll hängen (höchstens die letzten 20); es enthält Zeit, Quelle und Ergebnis, sonst nichts
pub fn log_line(at: u64, source: &str, result: &str) {
    let Some(d) = dir() else { return };
    let f = d.join("log.txt");
    let mut lines: Vec<String> = std::fs::read_to_string(&f).map(|t| t.lines().map(String::from).collect()).unwrap_or_default();
    lines.push(vc_i18n::t!("update-p1-utc-source-p3", p1 = format_time(at), source = &source, p3 = result.replace('\n', " ")));
    let skip = lines.len().saturating_sub(LOG_LINES);
    let body = lines[skip..].join("\n") + "\n";
    let _ = std::fs::create_dir_all(&d);
    let _ = crate::settings::write_atomic(&f, body.as_bytes());
}

/// Das Prüfprotokoll (neueste zuletzt)
pub fn log_text() -> String { dir().and_then(|d| std::fs::read_to_string(d.join("log.txt")).ok()).unwrap_or_default() }

/// Prüfprotokoll löschen
pub fn clear_log() { if let Some(d) = dir() { let _ = std::fs::remove_file(d.join("log.txt")); } }

/// Hinweis zur Installationsart: Pakete der Paketverwaltung aktualisiert diese; die Releaseseite bleibt der Link
pub fn package_hint(pkg: crate::feedback::Package) -> Option<String> {
    match pkg {
        crate::feedback::Package::LinuxPackage => Some(vc_i18n::t!("update-diese-installation-stammt-aus-einem-linux")),
        crate::feedback::Package::Source => Some(vc_i18n::t!("update-diese-installation-wurde-aus-dem-quelltext")),
        _ => None,
    }
}

/// Installationsart dieses Programms nach der Heuristik der Feedback-Funktion
pub fn install_kind() -> crate::feedback::Package {
    let exe = std::env::current_exe().ok();
    crate::feedback::detect_package(&exe.into_iter().collect::<Vec<_>>(), &|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(latest: &str, rels: &[(&str, &str)], extra: &str) -> String {
        let r: Vec<String> = rels.iter().map(|(v, n)| format!(r#"{{"version":"{v}","title":"{v} (Test)","notes":{}}}"#, serde_json::to_string(n).unwrap())).collect();
        format!(r#"{{"format":1,"sequence":4,"channel":"stable","published":"2026-12-02T09:00:00Z","latest":"{latest}"{extra},"releases":[{}]}}"#, r.join(","))
    }
    fn v(s: &str) -> Version { Version::parse(s).unwrap() }
    fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-update-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

    #[test]
    fn versionen_streng() {
        assert_eq!(Version::parse("0.10.2"), Some(Version(0, 10, 2)));
        for bad in ["", "1", "1.2", "1.2.3.4", "01.2.3", "1.02.3", "v1.2.3", "1.2.3-beta", "1.2.x", " 1.2.3", "1.2.3 ", "12345.0.0", "1..3", "-1.0.0", "1.2.3/../x", "1.2.3\n"] { assert_eq!(Version::parse(bad), None, "{bad:?}"); }
        assert!(v("0.10.0") > v("0.9.9") && v("1.0.0") > v("0.99.99"));
        assert_eq!(Version::installed().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn link_kommt_nur_aus_fester_basis_und_geprueftem_version() {
        assert_eq!(release_url(Some(v("0.2.0"))), "https://github.com/CDrueeke/VintageCoder/releases/tag/v0.2.0");
        assert_eq!(release_url(None), "https://github.com/CDrueeke/VintageCoder/releases");
        // eine Adresse in der Liste wird nie gelesen
        let t = feed("0.2.0", &[("0.2.0", "x")], r#","url":"https://boese.example/","notes_url":"https://boese.example/""#).replace(r#""notes":"x""#, r#""notes":"x","url":"https://boese.example/x""#);
        let ev = evaluate(&Manifest::parse(&t).unwrap(), v("0.1.0"));
        assert!(ev.update.unwrap().url.starts_with(TAG_URL_BASE));
        // eine Version mit Zusätzen wird abgelehnt, bevor sie in einen Link gelangen kann
        assert!(Manifest::parse(&feed("0.2.0/../../evil", &[("0.2.0/../../evil", "x")], "")).is_err());
        assert!(Manifest::parse(&feed("0.2.0?x=1", &[("0.2.0?x=1", "x")], "")).is_err());
    }

    #[test]
    fn liste_wird_streng_geprueft() {
        assert!(Manifest::parse(&feed("0.2.0", &[("0.2.0", "a"), ("0.1.0", "b")], "")).is_ok());
        assert_eq!(Manifest::parse(r#"{"format":2,"channel":"stable"}"#), Err(ParseError::TooNew(2)));
        for bad in ["", "kein json", "[]", r#"{"format":1}"#, r#"{"format":0,"channel":"stable","latest":"1.0.0","releases":[]}"#,
                    r#"{"format":1,"channel":"beta","latest":"1.0.0","releases":[{"version":"1.0.0"}]}"#, r#"{"format":1,"channel":"stable","latest":"1.0","releases":[]}"#] {
            assert!(matches!(Manifest::parse(bad), Err(ParseError::Invalid(_))), "{bad}");
        }
        assert!(Manifest::parse(&feed("0.3.0", &[("0.2.0", "a")], "")).is_err(), "latest muss in releases stehen");
        assert!(Manifest::parse(&feed("0.2.0", &[("0.2.0", "a"), ("0.2.0", "b")], "")).is_err(), "doppelte Version");
        let lang = "x".repeat(MAX_NOTES + 1);
        assert!(Manifest::parse(&feed("0.2.0", &[("0.2.0", &lang)], "")).is_err());
        let viele: Vec<(String, String)> = (0..=MAX_RELEASES as u32).map(|i| (format!("0.{i}.0"), "x".into())).collect();
        let refs: Vec<(&str, &str)> = viele.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        assert!(Manifest::parse(&feed("0.50.0", &refs, "")).is_err(), "zu viele Einträge");
    }

    #[test]
    fn unbekannte_felder_und_spaetere_signatur_stoeren_nicht() {
        let t = feed("0.2.0", &[("0.2.0", "a")], r#","signature":"abc","expires":"2030-01-01T00:00:00Z","zukunft":{"x":1}"#);
        assert!(Manifest::parse(&t).is_ok());
    }

    #[test]
    fn auswertung_nur_neuere_versionen_nie_downgrade() {
        let m = Manifest::parse(&feed("0.2.0", &[("0.2.0", "Neu 0.2"), ("0.1.1", "Fix"), ("0.1.0", "Erste")], "")).unwrap();
        let e = evaluate(&m, v("0.1.0")); let u = e.update.unwrap();
        assert_eq!(u.version, v("0.2.0"));
        assert!(u.notes.contains("## Version 0.2.0 (Test)") && u.notes.contains("Neu 0.2") && u.notes.contains("## Version 0.1.1") && !u.notes.contains("Erste"), "{}", u.notes);
        assert!(u.notes.find("0.2.0").unwrap() < u.notes.find("0.1.1").unwrap(), "neueste zuerst");
        assert_eq!(evaluate(&m, v("0.2.0")).update, None, "gleiche Version: aktuell");
        assert_eq!(evaluate(&m, v("0.3.0")).update, None, "installiert neuer als die Liste: kein Downgrade");
        assert!(!evaluate(&m, v("0.1.0")).installed_yanked);
    }

    #[test]
    fn zurueckgezogene_versionen() {
        let m = Manifest::parse(&feed("0.2.0", &[("0.2.0", "a"), ("0.1.1", "b"), ("0.1.0", "c")], r#","yanked":["0.2.0","0.1.0"]"#)).unwrap();
        // latest ist zurückgezogen: die höchste nicht zurückgezogene wird angeboten, falls sie neuer ist
        assert_eq!(evaluate(&m, v("0.1.0")).update.unwrap().version, v("0.1.1"));
        let e = evaluate(&m, v("0.1.1")); assert!(e.update.is_none() && !e.installed_yanked);
        let e = evaluate(&m, v("0.2.0")); assert!(e.installed_yanked && e.update.is_none(), "zurückgezogen und nichts Neueres: nur Hinweis");
        assert!(evaluate(&m, v("0.1.0")).installed_yanked);
    }

    #[test]
    fn projektformat_warnung() {
        let m = Manifest::parse(&feed("0.2.0", &[("0.2.0", "a")], &format!(r#","project_format":{}"#, PROJECT_FORMAT + 1))).unwrap();
        assert!(evaluate(&m, v("0.1.0")).update.unwrap().project_format_warning);
        let m = Manifest::parse(&feed("0.2.0", &[("0.2.0", "a")], &format!(r#","project_format":{PROJECT_FORMAT}"#))).unwrap();
        assert!(!evaluate(&m, v("0.1.0")).update.unwrap().project_format_warning);
    }

    #[test]
    fn aenderungstext_ohne_links_und_bilder() {
        let t = plain_notes("Siehe [STATUS](STATUS.md) und [Seite](https://x.example/a) sowie ![Bild](https://x.example/b.png) und ![](bild:ui).\n[![Abzeichen](https://b/x.svg)](https://ziel.example/)");
        assert!(!t.contains("](") && !t.contains("!["), "{t}");
        assert!(t.contains("Siehe STATUS und Seite (https://x.example/a) sowie Bild"), "{t}");
        assert!(t.contains("Abzeichen (https://ziel.example/)"), "{t}");
        assert_eq!(plain_notes("a\u{7}b\r\nc"), "ab\nc");
        assert_eq!(plain_notes("<https://x.example>"), "‹https://x.example>");
        assert_eq!(plain_notes("[offen](unvollständig"), "[offen] (unvollständig");
    }

    #[test]
    fn zeitplan_und_zustimmung() {
        let mut st = State::default();
        let day = 24 * 3600;
        assert!(!is_due(Mode::Ask, &st, 10 * day) && !is_due(Mode::Never, &st, 10 * day), "ohne Zustimmung nie");
        assert!(is_due(Mode::Daily, &st, 10 * day), "nie geprüft: fällig");
        st.last_check = 10 * day; st.last_attempt = 10 * day;
        assert!(!is_due(Mode::Daily, &st, 10 * day + day - 1) && is_due(Mode::Daily, &st, 11 * day));
        assert!(!is_due(Mode::Weekly, &st, 16 * day) && is_due(Mode::Weekly, &st, 17 * day));
        st.fails = 1; st.last_attempt = 12 * day;
        assert!(!is_due(Mode::Daily, &st, 12 * day + 5 * 3600), "nach Fehler erst nach 6 Stunden");
        assert!(is_due(Mode::Daily, &st, 12 * day + 6 * 3600));
        assert_eq!([backoff_secs(0), backoff_secs(1), backoff_secs(2), backoff_secs(3)], [0, 6 * 3600, 12 * 3600, 24 * 3600]);
        assert_eq!(backoff_secs(50), 7 * 24 * 3600);
        assert_eq!(Mode::from_setting(""), Mode::Ask); assert_eq!(Mode::from_setting("unbekannt"), Mode::Ask);
        for m in [Mode::Never, Mode::Daily, Mode::Weekly] { assert_eq!(Mode::from_setting(m.setting()), m); }
    }

    #[test]
    fn selbstaendige_pruefung_braucht_zustimmung_und_release() {
        let on = crate::features::Features::all(); let off = crate::features::Features::from_json(r#"{"format":1,"update":false}"#);
        let mut p = crate::settings::UpdatePrefs::default(); let st = State::default();
        if std::env::var_os("VC_SHOT").is_some() { return; }
        assert!(!auto_check_allowed(&on, &p, &st, 1_000_000), "noch nicht gefragt");
        p.check = "never".into(); assert!(!auto_check_allowed(&on, &p, &st, 1_000_000));
        p.check = "daily".into(); assert!(auto_check_allowed(&on, &p, &st, 1_000_000));
        assert!(!auto_check_allowed(&off, &p, &st, 1_000_000), "Release ohne IDE-UPDATE: nie");
    }

    #[test]
    fn zeit_formatieren() {
        assert_eq!(format_time(0), "1970-01-01 00:00");
        assert_eq!(format_time(1_700_000_000), "2023-11-14 22:13");
        assert_eq!(format_time(1_782_000_000 + 59), "2026-06-21 00:00");
    }

    #[test]
    fn quelle_reihenfolge() {
        assert_eq!(resolve_source(None, ""), FEED_URL);
        assert_eq!(resolve_source(None, " /tmp/feed "), "/tmp/feed");
        assert_eq!(resolve_source(Some("/x"), "https://y"), "/x");
        assert_eq!(resolve_source(Some(" "), ""), FEED_URL);
        assert!(is_remote(FEED_URL) && !is_remote("/tmp/feed") && !is_remote("file:///tmp"));
        assert_eq!(fetch_text("http://example.invalid/update.json"), Err(FetchError::Insecure), "kein http");
    }

    #[test]
    fn curl_fehler_zuordnen() {
        assert_eq!(curl_error(Some(6), ""), FetchError::Offline);
        assert_eq!(curl_error(Some(28), ""), FetchError::Timeout);
        assert_eq!(curl_error(Some(60), ""), FetchError::Tls);
        assert_eq!(curl_error(Some(63), ""), FetchError::TooBig);
        assert_eq!(curl_error(Some(22), "curl: (22) The requested URL returned error: 404"), FetchError::Http(404));
        assert_eq!(curl_error(Some(22), "curl: (22) The requested URL returned error: 429\n"), FetchError::Http(429));
        assert!(matches!(curl_error(Some(1), "curl: (1) irgendwas"), FetchError::Other(m) if m.contains("irgendwas")));
        assert!(FetchError::Http(403).message().contains("vorübergehend"));
    }

    #[test]
    fn abruf_aus_ordner_und_datei_ohne_netz() {
        let d = tmp("abruf");
        std::fs::write(d.join("update.json"), feed("0.2.0", &[("0.2.0", "Neu"), ("0.1.0", "Alt")], "")).unwrap();
        let r = check(d.to_str().unwrap(), v("0.1.0"), 5);
        let ev = r.result.clone().unwrap();
        assert_eq!(ev.update.unwrap().version, v("0.2.0"));
        let r2 = check(d.join("update.json").to_str().unwrap(), v("0.2.0"), 6);
        assert!(r2.result.unwrap().update.is_none());
        // fehlende Datei, kaputte Datei, zu große Datei
        assert!(check(d.join("gibtsnicht").to_str().unwrap(), v("0.1.0"), 7).result.is_err());
        std::fs::write(d.join("kaputt.json"), "{").unwrap();
        assert!(check(d.join("kaputt.json").to_str().unwrap(), v("0.1.0"), 8).result.unwrap_err().contains("ungültig"));
        std::fs::write(d.join("gross.json"), vec![b' '; MAX_FEED_BYTES + 1]).unwrap();
        assert!(check(d.join("gross.json").to_str().unwrap(), v("0.1.0"), 9).result.unwrap_err().contains("zu groß"));
        std::fs::write(d.join("neu.json"), r#"{"format":9,"channel":"stable"}"#).unwrap();
        assert!(check(d.join("neu.json").to_str().unwrap(), v("0.1.0"), 10).result.unwrap_err().contains("Format 9"));
    }

    #[test]
    fn protokoll_zustand_und_neustart() {
        let d = tmp("state");
        std::env::set_var("VC_UPDATE_DIR", &d);
        let src = d.join("feed"); std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("update.json"), feed("0.2.0", &[("0.2.0", "Neu")], "")).unwrap();
        let mut st = State::default();
        let ok = check(src.to_str().unwrap(), v("0.1.0"), 1_700_000_000);
        record(&ok, &mut st);
        assert_eq!((st.last_check, st.fails, st.last_seen.as_str(), st.last_result.as_str()), (1_700_000_000, 0, "0.2.0", "Version 0.2.0 verfügbar"));
        assert_eq!(State::load(), st);
        // nach einem Neustart: Hinweis aus der gemerkten Liste, ohne Netz; nach dem Update verschwindet er
        assert_eq!(cached(v("0.1.0")).unwrap().update.unwrap().version, v("0.2.0"));
        assert!(cached(v("0.2.0")).unwrap().update.is_none());
        // Fehlversuch: Zähler steigt, letzte Liste bleibt
        let bad = check(src.join("nix").to_str().unwrap(), v("0.1.0"), 1_700_000_100);
        record(&bad, &mut st);
        assert_eq!((st.last_check, st.fails), (1_700_000_000, 1)); assert_eq!(st.last_attempt, 1_700_000_100);
        assert!(cached(v("0.1.0")).is_some());
        let log = log_text();
        assert_eq!(log.lines().count(), 2, "{log}");
        assert!(log.contains("2023-11-14 22:13 UTC") && log.contains("Version 0.2.0 verfügbar") && log.contains("Fehler:"));
        for i in 0..30 { log_line(i, "q", "x"); }
        assert_eq!(log_text().lines().count(), LOG_LINES);
        clear_log(); assert_eq!(log_text(), "");
        std::env::remove_var("VC_UPDATE_DIR");
    }

    #[test]
    fn hinweis_nur_wenn_nicht_uebersprungen_und_nicht_ruhend() {
        let m = Manifest::parse(&feed("0.2.0", &[("0.2.0", "a")], "")).unwrap();
        let ev = evaluate(&m, v("0.1.0"));
        let mut p = crate::settings::UpdatePrefs::default(); let mut st = State::default();
        assert!(badge_visible(&ev, &p, &st, 100));
        p.skipped = "0.2.0".into(); assert!(!badge_visible(&ev, &p, &st, 100), "übersprungen");
        p.skipped = "0.1.9".into(); assert!(badge_visible(&ev, &p, &st, 100), "andere Nummer übersprungen");
        p.skipped.clear(); st.snooze_until = 100 + SNOOZE_SECS; assert!(!badge_visible(&ev, &p, &st, 100) && badge_visible(&ev, &p, &st, 100 + SNOOZE_SECS));
        assert!(!badge_visible(&evaluate(&m, v("0.2.0")), &p, &State::default(), 0));
    }

    #[test]
    fn curl_gegen_geschlossenen_lokalen_port_meldet_keine_verbindung() {
        // Port 1 auf dem eigenen Rechner: sofort „Verbindung abgelehnt“, kein echtes Netz. Fehlt curl, ist das auch ein sauberer Fehler.
        match fetch_text("https://127.0.0.1:1/update.json") { Err(FetchError::Offline) | Err(FetchError::NoCurl) | Err(FetchError::Tls) | Err(FetchError::Timeout) => {} other => panic!("{other:?}") }
    }

    #[test]
    fn pakethinweis() {
        assert!(package_hint(crate::feedback::Package::LinuxPackage).unwrap().contains("Paketverwaltung"));
        assert!(package_hint(crate::feedback::Package::MacApp).is_none());
    }
}
