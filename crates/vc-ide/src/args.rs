// SPDX-License-Identifier: GPL-3.0-or-later
//! Katalog der Emulator-Kommandozeilenparameter (`data/args/<programm>.json`) und der reine Kern dahinter: Auswahl → Argumentliste,
//! Platzhalter ersetzen, Prüfung auf unbekannte, doppelte und widersprüchliche Parameter, Übernahme von Freitext in die Auswahl.
//! Ohne Oberfläche und ohne Prozessstart; `launch` und die Einstellungen benutzen dieselben Funktionen.
//!
//! Die Auswahl eines Emulators ist eine Abbildung Parameterkennung → Wert (siehe [`Selection`]): bei Schaltern `on`/`off`, sonst der Wert.
//! Parameter, die nicht ausgewählt sind, kommen nicht in die Kommandozeile (der Emulator nimmt seinen Standard). Was der Katalog nicht kennt,
//! steht weiter im freien Feld „weitere Parameter“ (`EmulatorCfg::args`), das nach den Katalogparametern angehängt wird.
//!
//! Platzhalter stehen in geschweiften Klammern: `{project}`, `{project_dir}`, `{output}`, `{target}`, `{variant}`, `{start}`, `{symbols}`, `{disk}`
//! (Liste und Bedeutung: [`PLACEHOLDERS`]). `{{` und `}}` ergeben je eine geschweifte Klammer.
use crate::settings::split_args;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Auswahl der Katalogparameter eines Emulators: Kennung → Wert (`on`/`off` bei Schaltern)
pub type Selection = BTreeMap<String, String>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType { #[default] Switch, Choice, Text, Number, File }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Choice { pub value: String, pub label: String }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Group { pub id: String, pub label: String }

/// Ein Parameter des Katalogs
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub id: String,
    #[serde(default)] pub group: String,
    #[serde(rename = "type", default)] pub ty: ParamType,
    /// Schalter, der den Parameter einschaltet bzw. setzt (z. B. `-warp`, `-VICIIfilter`)
    pub flag: String,
    /// Wert steht im selben Wort wie der Schalter, getrennt durch dieses Zeichen (Altirra: `/kernel:xl`); sonst ist er das nächste Wort
    #[serde(default)] pub join: Option<String>,
    /// Gegenschalter (z. B. `+warp`); nur bei Schaltern
    #[serde(default)] pub off_flag: Option<String>,
    /// Kurzbeschreibung (Beschriftung)
    #[serde(default)] pub label: String,
    /// Erklärung für den Hover-Text
    #[serde(default)] pub hint: String,
    #[serde(default)] pub choices: Vec<Choice>,
    /// Standard des Emulators (nur Anzeige: was gilt, wenn der Parameter nicht gesetzt ist)
    #[serde(default)] pub default: Option<String>,
    #[serde(default)] pub example: Option<String>,
    #[serde(default)] pub min: Option<f64>,
    #[serde(default)] pub max: Option<f64>,
    /// Kennungen von Parametern, die nicht zugleich gesetzt sein sollen
    #[serde(default)] pub conflicts: Vec<String>,
    /// Kennungen von Parametern, ohne die dieser keine Wirkung hat
    #[serde(default)] pub requires: Vec<String>,
}

/// Katalog eines Programms
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub program: String,
    #[serde(default)] pub label: String,
    /// Emulator-Sorten (`EmulatorCfg::kind`), für die der Katalog gilt
    #[serde(default)] pub kinds: Vec<String>,
    /// Womit die Einträge geprüft wurden
    #[serde(default)] pub verified: String,
    #[serde(default)] pub groups: Vec<Group>,
    /// Schalter, die VintageCoder selbst setzt (Start von Datei, Debugger); im freien Feld bringen sie eine Warnung
    #[serde(default)] pub reserved: Vec<String>,
    /// Schalter, die der Debug-Start selbst braucht (Monitor-Anbindung); Angaben des Benutzers dazu werden beim Debuggen weggelassen.
    /// Hat ein Schalter einen Wert, steht er mit dazu: `-binarymonitoraddress <value>`
    #[serde(default)] pub debug_reserved: Vec<String>,
    /// Unbekannte Schalter im freien Feld sind Fehler (statt Hinweis): für Kataloge, deren Schalter ein Programm von VintageCoder selbst auswertet
    #[serde(default)] pub strict: bool,
    #[serde(default)] pub params: Vec<Param>,
}

impl Catalog {
    pub fn param(&self, id: &str) -> Option<&Param> { self.params.iter().find(|p| p.id == id) }
    pub fn in_group<'a>(&'a self, group: &'a str) -> impl Iterator<Item = &'a Param> + 'a { self.params.iter().filter(move |p| p.group == group) }
    /// Parameter, zu dem ein Schalter gehört (Einschalt- oder Gegenschalter) und ob es der Gegenschalter ist
    pub fn by_flag(&self, flag: &str) -> Option<(&Param, bool)> {
        self.params.iter().find_map(|p| if p.flag == flag { Some((p, false)) } else if p.off_flag.as_deref() == Some(flag) { Some((p, true)) } else { None })
    }
}

// ───────────────────────── Laden ─────────────────────────

/// Ordner `data/args` in der Reihenfolge der übrigen Datenbank: `VC_DATA`, dann neben dem Programm bzw. im Arbeitsordner (wie `find_data_dir`),
/// zuletzt der persönliche Ordner `~/.config/vintagecoder/args` (dort lassen sich Kataloge ergänzen oder überschreiben)
pub fn search_dirs(data: Option<&Path>) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = vec![];
    let mut add = |d: PathBuf| { let d = d.join("args"); if d.is_dir() && !v.contains(&d) { v.push(d); } };
    if let Some(d) = data { add(d.to_path_buf()); }
    if let Ok(d) = std::env::var("VC_DATA") { add(PathBuf::from(d)); }
    if let Some(d) = std::env::current_exe().ok().and_then(|e| crate::find_data_dir(&e)) { add(d); }
    if let Some(d) = std::env::current_dir().ok().and_then(|e| crate::find_data_dir(&e)) { add(d); }
    // der persönliche Ordner steht immer am Ende (auch ohne bestehenden Ordner entsteht daraus kein Fehler)
    v
}

fn user_dir() -> Option<PathBuf> { vc_core::userdirs::config_dir().map(|d| d.join("args")).filter(|d| d.is_dir()) }

/// Katalogdatei lesen; Fehler nennen die Datei
pub fn parse(text: &str, origin: &str) -> Result<Catalog, String> {
    let c: Catalog = serde_json::from_str(text).map_err(|e| format!("{origin}: {e}"))?;
    if c.program.trim().is_empty() { return Err(format!("{origin}: Feld program fehlt")); }
    let mut seen = std::collections::HashSet::new();
    for p in &c.params {
        if p.id.is_empty() || p.flag.is_empty() { return Err(format!("{origin}: Parameter ohne id oder flag")); }
        if !seen.insert(p.id.as_str()) { return Err(format!("{origin}: Parameter {} kommt doppelt vor", p.id)); }
        if p.ty == ParamType::Choice && p.choices.is_empty() { return Err(format!("{origin}: Auswahl {} ohne choices", p.id)); }
    }
    Ok(c)
}

/// Katalog zusammenführen: Parameter des Zusatzes mit gleicher Kennung ersetzen die vorhandenen, neue werden angehängt
pub fn merge(base: &mut Catalog, extra: Catalog) {
    for g in extra.groups { if let Some(x) = base.groups.iter_mut().find(|x| x.id == g.id) { *x = g; } else { base.groups.push(g); } }
    for p in extra.params { if let Some(x) = base.params.iter_mut().find(|x| x.id == p.id) { *x = p; } else { base.params.push(p); } }
    for r in extra.reserved { if !base.reserved.contains(&r) { base.reserved.push(r); } }
    for r in extra.debug_reserved { if !base.debug_reserved.contains(&r) { base.debug_reserved.push(r); } }
    for k in extra.kinds { if !base.kinds.contains(&k) { base.kinds.push(k); } }
    if !extra.label.is_empty() { base.label = extra.label; }
}

/// Alle Kataloge aus den Ordnern lesen (gleiche Programmkennung → zusammengeführt, spätere Ordner ergänzen); Fehler werden gesammelt
pub fn load_dirs(dirs: &[PathBuf]) -> (Vec<Catalog>, Vec<String>) {
    let (mut out, mut errs): (Vec<Catalog>, Vec<String>) = (vec![], vec![]);
    for d in dirs {
        let Ok(rd) = std::fs::read_dir(d) else { continue };
        let mut files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "json")).collect();
        files.sort();
        for f in files {
            let origin = f.display().to_string();
            match std::fs::read_to_string(&f).map_err(|e| format!("{origin}: {e}")).and_then(|t| parse(&t, &origin)) {
                // Der erste gefundene Katalog eines Programms ist der Grundstock; derselbe Programmname aus dem persönlichen Ordner ergänzt ihn
                Ok(c) => match out.iter_mut().find(|x| x.program == c.program) { Some(x) => { if Some(d) == user_dir().as_ref() { merge(x, c) } } None => out.push(c) },
                Err(e) => errs.push(e),
            }
        }
    }
    (out, errs)
}

/// Katalog für eine Emulator-Sorte (`x64sc`, `atari800`, …), mit Zwischenspeicher (drei Sekunden), damit die Oberfläche nicht in jedem Bild liest
pub fn catalog_for(kind: &str) -> Option<Arc<Catalog>> {
    if !crate::emulators::enabled(kind) { return None; }     // Anbindung nicht im Release gewählt: kein Parameterkatalog
    type Cache = Option<(Instant, Vec<Arc<Catalog>>)>;
    static CACHE: Mutex<Cache> = Mutex::new(None);
    let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if g.as_ref().map_or(true, |(t, _)| t.elapsed() > Duration::from_secs(3)) {
        let mut dirs = search_dirs(None); dirs.extend(user_dir());
        *g = Some((Instant::now(), load_dirs(&dirs).0.into_iter().map(Arc::new).collect()));
    }
    // Programmkennung laut Emulator-Katalog (Feld `args`), sonst nach den Sorten im Katalog der Parameter
    let named = crate::emulators::for_kind(kind).and_then(|e| e.args.get(kind));
    g.as_ref().and_then(|(_, v)| named.and_then(|n| v.iter().find(|c| &c.program == n)).or_else(|| v.iter().find(|c| c.kinds.iter().any(|k| k == kind))).cloned())
}

/// Katalog zu einer Sorte ohne Zwischenspeicher, mit Datenordner `data` an erster Stelle der Suche (für den Build)
pub fn catalog_with(data: Option<&Path>, kind: &str) -> Option<Catalog> {
    if !crate::emulators::enabled(kind) { return None; }
    let mut dirs = search_dirs(data); dirs.extend(user_dir());
    load_dirs(&dirs).0.into_iter().find(|c| c.kinds.iter().any(|k| k == kind))
}

/// Katalog für die Sorte; `x64` (altes VICE) benutzt den des x64sc
pub fn catalog_of_kind(kind: &str) -> Option<Arc<Catalog>> { catalog_for(if kind == "x64" { "x64sc" } else { kind }) }

// ───────────────────────── Platzhalter ─────────────────────────

/// Ein Platzhalter für die Oberfläche: Name (ohne Klammern), Beschriftung, Erklärung
pub struct Placeholder { pub name: &'static str, pub label: &'static str, pub hint: &'static str }

pub const PLACEHOLDERS: &[Placeholder] = &[
    Placeholder { name: "project", label: "Projektname", hint: "Name des Projekts (Ordnername)" },
    Placeholder { name: "project_dir", label: "Projektordner", hint: "Absoluter Pfad des Projektordners" },
    Placeholder { name: "output", label: "Ausgabedatei", hint: "Das gebaute Programm (PRG, XEX, CRT, …), das gestartet wird" },
    Placeholder { name: "target", label: "Zielsystem", hint: "Kennung des Zielsystems: c64, c128, plus4, vic20, atari8, mega65" },
    Placeholder { name: "variant", label: "Videonorm", hint: "pal oder ntsc (Atari: aus der Projektdatei, sonst pal)" },
    Placeholder { name: "start", label: "Startadresse", hint: "Einsprung aus dem BASIC-Starter (SYS) als vier Hexziffern, z. B. 080D; nur wenn das Programm einen hat" },
    Placeholder { name: "symbols", label: "Symboldatei", hint: "Labels für den Monitor (VICE: .vs neben der Ausgabedatei); nur nach einem Build" },
    Placeholder { name: "disk", label: "Diskimage", hint: "Das Diskimage des Projekts (Feld disk der Projektdatei); nur wenn das Projekt eines hat" },
];

/// Werte der Platzhalter für einen Start; `None` heißt: hier nicht verfügbar
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArgCtx {
    pub project: String,
    pub root: PathBuf,
    pub output: PathBuf,
    pub target: String,
    pub variant: String,
    pub start: Option<u16>,
    pub symbols: Option<PathBuf>,
    pub disk: Option<PathBuf>,
}

impl ArgCtx {
    /// Werte aus einem echten Start: Projektordner, gebaute Datei, Zielsystem und Videonorm
    pub fn from_launch(root: &Path, output: &Path, target: &str, variant: &str) -> ArgCtx {
        let start = std::fs::read(output).ok().and_then(|b| crate::debug::sys_target(&b));
        let vs = output.with_extension("vs");
        let disk = crate::projfile::find(root).and_then(|f| crate::projfile::ProjectFile::load(&f).ok()).and_then(|pf| pf.run_disk(root)).filter(|p| p.exists());
        ArgCtx {
            project: root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            root: root.to_path_buf(), output: output.to_path_buf(), target: target.into(), variant: variant.into(),
            start, symbols: Some(vs).filter(|p| p.exists()), disk,
        }
    }
    /// Beispielwerte für die Vorschau in den Einstellungen (kein echtes Projekt); alle Platzhalter sind belegt
    pub fn example(target: &str) -> ArgCtx {
        let (ext, start) = match target { "atari8" => ("xex", None), "mega65" => ("prg", Some(0x2014)), _ => ("prg", Some(0x080D)) };
        let root = PathBuf::from("/Users/ich/Projekte/demo");
        let dext = if target == "atari8" { "atr" } else if target == "mega65" { "d81" } else { "d64" };
        ArgCtx {
            project: "demo".into(), output: root.join("build").join(format!("demo.{ext}")), symbols: Some(root.join("build").join("demo.vs")), disk: Some(root.join("build").join(format!("demo.{dext}"))),
            root, target: target.into(), variant: "pal".into(), start,
        }
    }
    fn value(&self, name: &str) -> Result<String, String> {
        let p = |x: &Path| x.to_string_lossy().into_owned();
        match name {
            "project" => Ok(self.project.clone()),
            "project_dir" => Ok(p(&self.root)),
            "output" => if self.output.as_os_str().is_empty() { Err("{output}: hier noch keine Ausgabedatei".into()) } else { Ok(p(&self.output)) },
            "target" => Ok(self.target.clone()),
            "variant" => Ok(if self.variant.is_empty() { "pal".into() } else { self.variant.clone() }),
            "start" => self.start.map(|a| format!("{a:04X}")).ok_or_else(|| "{start}: das Programm hat keinen BASIC-Starter mit SYS".into()),
            "symbols" => self.symbols.as_deref().map(p).ok_or_else(|| "{symbols}: keine Symboldatei (erst bauen)".into()),
            "disk" => self.disk.as_deref().map(p).ok_or_else(|| "{disk}: das Projekt hat kein Diskimage".into()),
            _ => Err(format!("{{{name}}}: unbekannter Platzhalter (bekannt: {})", PLACEHOLDERS.iter().map(|p| format!("{{{}}}", p.name)).collect::<Vec<_>>().join(", "))),
        }
    }
}

/// Namen aller Platzhalter in `text` (ohne `{{`/`}}`), in der Reihenfolge ihres Vorkommens
pub fn placeholders_in(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '{' if it.peek() == Some(&'{') => { it.next(); }
            '{' => { let mut n = String::new(); for d in it.by_ref() { if d == '}' { break; } n.push(d); } out.push(n); }
            _ => {}
        }
    }
    out
}

/// Platzhalter in `text` ersetzen. Fehler (alle, nicht nur der erste): unbekannte Platzhalter, nicht verfügbare Werte, nicht geschlossene Klammer.
pub fn expand(text: &str, ctx: &ArgCtx) -> Result<String, Vec<String>> {
    let (mut out, mut errs) = (String::new(), vec![]);
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '{' if it.peek() == Some(&'{') => { it.next(); out.push('{'); }
            '}' if it.peek() == Some(&'}') => { it.next(); out.push('}'); }
            '{' => {
                let mut name = String::new(); let mut closed = false;
                for d in it.by_ref() { if d == '}' { closed = true; break; } name.push(d); }
                if !closed { errs.push(format!("Klammer „{{{}“ nicht geschlossen", name.chars().take(12).collect::<String>())); continue; }
                match ctx.value(name.trim()) { Ok(v) => out.push_str(&v), Err(e) => errs.push(e) }
            }
            c => out.push(c),
        }
    }
    if errs.is_empty() { Ok(out) } else { errs.dedup(); Err(errs) }
}

// ───────────────────────── Auswahl → Argumente ─────────────────────────

/// Eine Meldung der Prüfung
#[derive(Clone, Debug, PartialEq)]
pub struct Issue { pub level: Level, pub text: String }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level { /// Hinweis, kein Fehler
    Info, /// kann so nicht gewollt sein, wird aber gestartet
    Warn, /// Start würde scheitern oder der Wert ist ungültig
    Error }

fn issue(level: Level, text: impl Into<String>) -> Issue { Issue { level, text: text.into() } }

/// Argumente der Auswahl in Katalogreihenfolge (ohne Freitext). Platzhalter sind noch nicht ersetzt. Unbekannte Kennungen werden übersprungen
/// (die Prüfung meldet sie); Auswahlen mit leerem Wert zählen als nicht gesetzt.
pub fn selected_args(cat: &Catalog, sel: &Selection) -> Vec<String> {
    let mut out = vec![];
    for p in &cat.params {
        let Some(v) = sel.get(&p.id) else { continue };
        let v = v.trim();
        match p.ty {
            ParamType::Switch => match v { "on" => out.push(p.flag.clone()), "off" => if let Some(o) = &p.off_flag { out.push(o.clone()) }, _ => {} },
            _ => if !v.is_empty() { match &p.join { Some(j) => out.push(format!("{}{j}{v}", p.flag)), None => { out.push(p.flag.clone()); out.push(v.to_string()); } } },
        }
    }
    out
}

/// Fertige Argumentliste: Katalogparameter, danach das freie Feld; Platzhalter durch `ctx` ersetzt. Bei Fehlern in Platzhaltern kommen alle zurück.
pub fn build(cat: Option<&Catalog>, sel: &Selection, free: &str, ctx: &ArgCtx) -> Result<Vec<String>, Vec<String>> {
    let mut args = cat.map(|c| selected_args(c, sel)).unwrap_or_default();
    args.extend(split_args(free));
    let (mut out, mut errs) = (vec![], vec![]);
    for a in args { match expand(&a, ctx) { Ok(v) => out.push(v), Err(e) => errs.extend(e) } }
    errs.dedup();
    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

/// Argumente ohne alles, was Platzhalter enthält (Test-Start in den Einstellungen: dort gibt es kein Projekt). Ein Parameter mit Platzhalter
/// entfällt samt seinem Schalter; im freien Feld entfällt das Wort und ein davorstehender Schalter.
pub fn build_plain(cat: Option<&Catalog>, sel: &Selection, free: &str) -> Vec<String> {
    let has = |t: &str| !placeholders_in(t).is_empty();
    let mut args = vec![];
    if let Some(c) = cat { let plain: Selection = sel.iter().filter(|(_, v)| !has(v)).map(|(k, v)| (k.clone(), v.clone())).collect(); args = selected_args(c, &plain); }
    let tokens = split_args(free);
    for (i, t) in tokens.iter().enumerate() {
        if has(t) { if args.last().map_or(false, |l| i > 0 && &tokens[i - 1] == l && is_flag_like(l)) { args.pop(); } continue; }
        args.push(t.clone());
    }
    args
}

/// Wie [`build`], aber für die Anzeige: nicht auflösbare Platzhalter bleiben stehen; liefert zusätzlich deren Meldungen
pub fn preview(cat: Option<&Catalog>, sel: &Selection, free: &str, ctx: &ArgCtx) -> (Vec<String>, Vec<String>) {
    let mut args = cat.map(|c| selected_args(c, sel)).unwrap_or_default();
    args.extend(split_args(free));
    let (mut out, mut errs) = (vec![], vec![]);
    for a in args { match expand(&a, ctx) { Ok(v) => out.push(v), Err(e) => { errs.extend(e); out.push(a) } } }
    errs.dedup();
    (out, errs)
}

/// Kommandozeile zum Kopieren: Argumente mit Leerzeichen oder Anführungszeichen werden in `"` gesetzt
pub fn shell_line(program: &str, args: &[String]) -> String { shell_line_os(program, args, cfg!(windows)) }

/// Wie [`shell_line`] für ein Betriebssystem. Windows (cmd, PowerShell): Backslashes bleiben unverändert (`C:\Program Files\x`), nur ein
/// Backslash-Rest vor dem schließenden `"` wird verdoppelt und ein `"` im Wort mit `\` maskiert; sonst wird `\` verdoppelt.
pub fn shell_line_os(program: &str, args: &[String], windows: bool) -> String {
    let q = |a: &str| {
        if !(a.is_empty() || a.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || (!windows && c == '\\'))) { return a.to_string(); }
        if !windows { return format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\"")); }
        let trail = a.len() - a.trim_end_matches('\\').len();
        format!("\"{}{}\"", a.replace('"', "\\\""), "\\".repeat(trail))
    };
    std::iter::once(program).chain(args.iter().map(String::as_str)).map(q).collect::<Vec<_>>().join(" ")
}

// ───────────────────────── Prüfung ─────────────────────────

fn is_flag_like(t: &str) -> bool { t.len() > 1 && (t.starts_with('-') || t.starts_with('+') || (t.starts_with('/') && !t[1..].contains('/'))) }

/// Schalter im freien Feld, ohne ihre Werte: ein Katalogschalter mit Wert (`-speed -30`) überspringt das nächste Wort, ein Wort ohne
/// `-`/`+` davor ist ein Wert. Ein Pfad (`/Users/x`) gilt nicht als Schalter, `/f` und `/disk:x` (Altirra) schon.
fn free_flags<'a>(cat: Option<&Catalog>, tokens: &'a [String]) -> Vec<&'a str> {
    let (mut out, mut i) = (vec![], 0);
    while i < tokens.len() {
        let t = head(cat, tokens[i].as_str()).0;
        if is_flag_like(t) {
            out.push(t);
            if cat.and_then(|c| takes_value(c, t)) == Some(true) { i += 1; }
        }
        i += 1;
    }
    out
}

/// Schalter ohne den im selben Wort angehängten Wert (`/kernel:xl` → `/kernel`), soweit ein Katalogparameter das so schreibt
fn head<'a>(cat: Option<&Catalog>, t: &'a str) -> (&'a str, Option<&'a str>) {
    if let Some(c) = cat { for p in &c.params { if let Some(j) = p.join.as_deref() { if let Some(v) = t.strip_prefix(p.flag.as_str()).and_then(|r| r.strip_prefix(j)) { return (&t[..p.flag.len()], Some(v)); } } } }
    (t, None)
}

/// Katalogschalter (ohne Wert) und ob das nächste Wort sein Wert ist
fn takes_value(cat: &Catalog, flag: &str) -> Option<bool> { cat.by_flag(flag).map(|(p, off)| p.ty != ParamType::Switch && !off && p.join.is_none()) }

/// Prüft Auswahl und freies Feld. Meldungen: ungültige Werte, widersprüchliche und fehlende Voraussetzungen, unbekannte Kennungen der Auswahl,
/// doppelte Schalter (auch zwischen Auswahl und Freitext), vom Programm selbst gesetzte Schalter, unbekannte Schalter im Freitext (Hinweis),
/// unbekannte Platzhalter. `ctx` ist optional: mit ihm werden auch nicht verfügbare Platzhalter gemeldet.
pub fn check(cat: Option<&Catalog>, sel: &Selection, free: &str, ctx: Option<&ArgCtx>) -> Vec<Issue> {
    let mut out: Vec<Issue> = vec![];
    let tokens = split_args(free);
    if let Some(cat) = cat {
        for (id, v) in sel {
            let Some(p) = cat.param(id) else { out.push(issue(Level::Warn, format!("„{id}“ steht in der Auswahl, ist aber im Katalog nicht (mehr) vorhanden"))); continue };
            let v = v.trim();
            match p.ty {
                ParamType::Switch => if v != "on" && v != "off" { out.push(issue(Level::Error, format!("{}: Schalterwert muss on oder off sein", p.label))) } else if v == "off" && p.off_flag.is_none() { out.push(issue(Level::Warn, format!("{}: hat keinen Gegenschalter, „aus“ bewirkt nichts", p.label))) },
                ParamType::Choice => if !v.is_empty() && !p.choices.iter().any(|c| c.value == v) && placeholders_in(v).is_empty() { out.push(issue(Level::Error, format!("{}: „{v}“ ist keine bekannte Auswahl", p.label))) },
                ParamType::Number => if !v.is_empty() && placeholders_in(v).is_empty() {
                    match v.replace(',', ".").parse::<f64>() {
                        Err(_) => out.push(issue(Level::Error, format!("{}: „{v}“ ist keine Zahl", p.label))),
                        Ok(n) => { if p.min.map_or(false, |m| n < m) || p.max.map_or(false, |m| n > m) { out.push(issue(Level::Error, format!("{}: {v} liegt außerhalb von {} bis {}", p.label, p.min.map_or("−∞".into(), |m| m.to_string()), p.max.map_or("∞".into(), |m| m.to_string())))) } }
                    }
                },
                ParamType::Text | ParamType::File => {}
            }
            if !active(p, v) { continue; }
            for c in &p.conflicts { if let Some(q) = cat.param(c) { if sel.get(c).map_or(false, |x| active(q, x.trim())) && id.as_str() < c.as_str() { out.push(issue(Level::Warn, format!("{} und {} widersprechen sich", p.label, q.label))); } } }
            for r in &p.requires { if let Some(q) = cat.param(r) { if !sel.get(r).map_or(false, |x| active(q, x.trim())) { out.push(issue(Level::Warn, format!("{} wirkt nur zusammen mit „{}“", p.label, q.label))); } } }
        }
        // Freitext gegen Katalog
        let flags = free_flags(Some(cat), &tokens);
        let mut seen: Vec<&str> = vec![];
        for f in &flags {
            if seen.contains(f) { out.push(issue(Level::Warn, format!("{f} steht mehrfach im freien Feld"))); continue; }
            seen.push(f);
            if cat.debug_reserved.iter().any(|r| r.split_whitespace().next() == Some(*f)) { out.push(issue(Level::Warn, format!("{f} wird beim Debuggen weggelassen: der Debug-Start braucht es für die Verbindung zum Emulator"))); }
            if cat.reserved.iter().any(|r| r == f) { out.push(issue(Level::Warn, format!("{f} setzt VintageCoder beim Start selbst; eine eigene Angabe kann das stören"))); }
            match cat.by_flag(f) {
                Some((p, off)) => {
                    // derselbe Schalter in Auswahl und Freitext (der Gegenschalter nur, wenn die Auswahl „aus“ sagt)
                    let same = sel.get(&p.id).map_or(false, |v| { let v = v.trim(); active(p, v) && (v == "off") == off });
                    if same { out.push(issue(Level::Warn, format!("{f} ({}) ist schon in der Auswahl gesetzt und steht zusätzlich im freien Feld", p.label))); }
                    if !off { for c in &p.conflicts { if let Some(q) = cat.param(c) { if sel.get(c).map_or(false, |x| active(q, x.trim())) || flags.iter().any(|g| *g == q.flag) { out.push(issue(Level::Warn, format!("{f} ({}) widerspricht „{}“", p.label, q.label))); } } } }
                    // Schalter gegen seinen eigenen Gegenschalter
                    if let Some(o) = p.off_flag.as_deref() { if f == &p.flag && flags.iter().any(|g| *g == o) { out.push(issue(Level::Warn, format!("{f} und {o} widersprechen sich"))); } }
                }
                None if cat.strict => out.push(issue(Level::Error, format!("{f} gibt es hier nicht (erlaubt: {})", cat.params.iter().map(|p| p.flag.as_str()).collect::<Vec<_>>().join(", ")))),
                None => if !f.starts_with('/') || flags.len() == tokens.len() { out.push(issue(Level::Info, format!("{f} kennt der Katalog nicht; es wird unverändert übergeben"))); },
            }
        }
        // Auswahl gegen Gegenschalter im Freitext (z. B. Schalter an, aber `+warp` im Freitext)
        for p in &cat.params { if let (Some(v), Some(o)) = (sel.get(&p.id), p.off_flag.as_deref()) { if v.trim() == "on" && flags.iter().any(|g| *g == o) { out.push(issue(Level::Warn, format!("{} ist an, aber {o} steht im freien Feld", p.label))); } } }
        // Wert fehlt hinter einem Schalter, der einen verlangt
        if let Some(last) = tokens.last() { if takes_value(cat, last) == Some(true) { out.push(issue(Level::Error, format!("{last} verlangt einen Wert"))); } }
    } else {
        let mut seen: Vec<&str> = vec![];
        for f in free_flags(None, &tokens) { if seen.contains(&f) { out.push(issue(Level::Warn, format!("{f} steht mehrfach im freien Feld"))); } else { seen.push(f); } }
    }
    // Platzhalter
    let mut texts: Vec<String> = sel.values().cloned().collect(); texts.push(free.to_string());
    for t in &texts {
        match ctx {
            Some(c) => if let Err(es) = expand(t, c) { for e in es { if !out.iter().any(|i| i.text == e) { out.push(issue(Level::Error, e)); } } },
            None => for n in placeholders_in(t) { if !PLACEHOLDERS.iter().any(|p| p.name == n.trim()) { let e = format!("{{{n}}}: unbekannter Platzhalter"); if !out.iter().any(|i| i.text == e) { out.push(issue(Level::Error, e)); } } },
        }
    }
    out.sort_by(|a, b| b.level.cmp(&a.level));
    out
}

/// Zählt der Parameter mit diesem Wert als gesetzt?
fn active(p: &Param, v: &str) -> bool { if p.ty == ParamType::Switch { v == "on" || (v == "off" && p.off_flag.is_some()) } else { !v.is_empty() } }

// ───────────────────────── Migration und Zurücksetzen ─────────────────────────

/// Bisherige Freitext-Parameter in die Auswahl überführen: Was der Katalog kennt (Schalter und Schalter mit Wert, jeweils einmal und nicht
/// schon in `sel` gesetzt, Auswahlwerte aus der Liste), wandert in die Auswahl; alles andere bleibt im Freitext, in der alten Reihenfolge.
/// Wiederholt vorkommende Schalter bleiben im Freitext (bei VICE gewinnt der letzte; die Reihenfolge soll sich nicht ändern).
pub fn migrate_free(cat: &Catalog, sel: &Selection, free: &str) -> (Selection, String) {
    let tokens = split_args(free);
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &tokens { *count.entry(t.as_str()).or_default() += 1; }
    let mut out = sel.clone();
    let (mut rest, mut i): (Vec<String>, usize) = (vec![], 0);
    while i < tokens.len() {
        let (t, joined) = { let (h, v) = head(Some(cat), &tokens[i]); (&tokens[i].as_str()[..h.len()], v) };
        let t = &t.to_string();
        if let (Some(v), Some((p, false))) = (joined, cat.by_flag(t)) {
            let free_of_dup = count.iter().filter(|(k, _)| head(Some(cat), k).0 == t.as_str()).map(|(_, n)| *n).sum::<usize>() == 1;
            if free_of_dup && !out.contains_key(&p.id) && (p.ty != ParamType::Choice || p.choices.iter().any(|c| c.value == v)) { out.insert(p.id.clone(), v.to_string()); i += 1; continue; }
            rest.push(tokens[i].clone()); i += 1; continue;
        }
        let known = cat.by_flag(t).filter(|(p, _)| count.get(t.as_str()) == Some(&1) && !out.contains_key(&p.id) && !p.conflicts.iter().any(|c| tokens.iter().any(|x| cat.by_flag(x).map_or(false, |(q, _)| &q.id == c))));
        match known {
            Some((p, off)) if p.ty == ParamType::Switch => { out.insert(p.id.clone(), if off { "off" } else { "on" }.into()); i += 1; }
            Some((p, false)) if i + 1 < tokens.len() && !is_flag_like(&tokens[i + 1]) && (p.ty != ParamType::Choice || p.choices.iter().any(|c| c.value == tokens[i + 1])) => { out.insert(p.id.clone(), tokens[i + 1].clone()); i += 2; }
            _ => { rest.push(t.clone()); i += 1; }
        }
    }
    (out, rest.iter().map(|a| quote_free(a)).collect::<Vec<_>>().join(" "))
}

/// Schalter im Freitext, die [`migrate_free`] übernehmen würde (für den Hinweis in der Oberfläche)
pub fn migratable(cat: &Catalog, sel: &Selection, free: &str) -> usize {
    let (s, _) = migrate_free(cat, sel, free);
    s.len() - sel.len()
}

/// Argumente für den Debug-Start: ohne die Schalter (und Werte), die der Debug-Start selbst setzt ([`Catalog::debug_reserved`])
pub fn drop_debug_reserved(cat: Option<&Catalog>, args: Vec<String>) -> Vec<String> {
    let Some(cat) = cat else { return args };
    let mut out = vec![]; let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match cat.debug_reserved.iter().find(|r| r.split_whitespace().next() == Some(a.as_str())) {
            Some(r) => { if r.split_whitespace().nth(1).is_some() { it.next(); } }
            None => out.push(a),
        }
    }
    out
}

/// Ein Wort für das freie Feld so schreiben, dass `split_args` es wieder als ein Wort liest. Unter Windows bleiben Backslashes unverändert
/// (Ausnahme: ein Backslash am Ende eines Wortes mit Leerzeichen lässt sich dort nicht in Anführungszeichen schreiben und bleibt ohne)
pub fn quote_free(a: &str) -> String { quote_free_os(a, cfg!(windows)) }

pub fn quote_free_os(a: &str, windows: bool) -> String {
    if !windows { return if a.is_empty() || a.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'' || c == '\\') { format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\"")) } else { a.to_string() }; }
    if a.is_empty() || a.chars().any(|c| c.is_whitespace() || c == '"' || c == '\'') { format!("\"{}\"", a.replace('"', "\\\"")) } else { a.to_string() }
}

/// Zurücksetzen auf Standard: nichts ausgewählt, freies Feld leer (der Emulator läuft mit seinen eigenen Standardwerten)
pub fn reset(sel: &mut Selection, free: &mut String) { sel.clear(); free.clear(); }

/// Parameter, die vom Standard des Emulators abweichen (Anzahl gesetzter Katalogparameter, ohne unbekannte Kennungen)
pub fn changed_count(cat: &Catalog, sel: &Selection) -> usize { sel.iter().filter(|(id, v)| cat.param(id).map_or(false, |p| active(p, v.trim()))).count() }

// ───────────────────────── Assembler ─────────────────────────

/// Einstellungen des Assemblers aus Katalogauswahl und freiem Feld (Einstellungen → Toolchain)
pub fn asm_args(cat: Option<&Catalog>, cfg: &crate::settings::AsmCfg, ctx: &ArgCtx) -> Result<Vec<String>, Vec<String>> { build(cat, &cfg.opts, &cfg.args, ctx) }

/// Wendet fertige Assembler-Argumente (`--illegals X`, `-I Ordner`, `-D NAME=Zahl`) auf `o` an; liefert Meldungen fürs Bauprotokoll.
/// Alles andere ist ein Fehler. Include-Ordner kommen nach dem Projektordner und vor die Systemdateien (letzteres ordnet der Aufrufer).
pub fn apply_asm(args: &[String], o: &mut vc_asm::Options) -> Result<Vec<String>, String> {
    let (mut notes, mut i) = (vec![], 0);
    let val = |i: &mut usize, flag: &str| -> Result<String, String> { *i += 1; args.get(*i).cloned().ok_or_else(|| format!("{flag} verlangt einen Wert")) };
    while i < args.len() {
        let f = args[i].clone();
        match f.as_str() {
            "--illegals" => { let v = val(&mut i, &f)?; o.illegals = vc_asm::Policy::parse(&v).ok_or_else(|| format!("--illegals: „{v}“ ist nicht deny, warn oder allow"))?; notes.push(format!("Illegale Opcodes: {v}")); }
            "-I" => { let v = val(&mut i, &f)?; o.include_dirs.push(PathBuf::from(&v)); notes.push(format!("Include-Ordner: {v}")); }
            "-D" => {
                let v = val(&mut i, &f)?;
                let (n, x) = v.split_once('=').map(|(n, x)| (n.trim().to_string(), x.trim().to_string())).unwrap_or((v.trim().to_string(), "1".into()));
                let num = vc_target::parse_num(&x).ok_or_else(|| format!("-D {v}: erwartet NAME=Zahl"))?;
                if n.is_empty() { return Err(format!("-D {v}: Name fehlt")); }
                o.defines.retain(|(m, _)| *m != n); o.defines.push((n.clone(), num)); notes.push(format!("Konstante: {n}={num}"));
            }
            other => return Err(format!("{other}: unbekannter Assembler-Schalter (erlaubt: --illegals, -I, -D)")),
        }
        i += 1;
    }
    Ok(notes)
}
