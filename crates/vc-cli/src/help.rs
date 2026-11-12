// SPDX-License-Identifier: GPL-3.0-or-later
//! Hilfe der Kommandozeile: `--help`, `vcc help`, Fehlerhinweise, man-Seiten und Vervollständigung.
//!
//! Alles entsteht aus den Beschreibungsdateien `data/cli/*.json` (Format: `docs/cli-hilfe.md`); sie sind in das Programm eingebettet.
//! Der Parser der Befehle (`main.rs`) bleibt unverändert; ein Test vergleicht seine Optionen mit der Beschreibung.
use serde_json::Value;
use std::fmt::Write as _;
use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::OnceLock;

/// Beschreibungsdateien (eine je Befehl); Reihenfolge ohne Bedeutung, die Übersicht ordnet nach Gruppen.
const FILES: &[(&str, &str)] = &[
    ("_vcc", include_str!("../../../data/cli/_vcc.json")),
    ("add", include_str!("../../../data/cli/add.json")),
    ("asm", include_str!("../../../data/cli/asm.json")),
    ("basic", include_str!("../../../data/cli/basic.json")),
    ("basic2asm", include_str!("../../../data/cli/basic2asm.json")),
    ("build", include_str!("../../../data/cli/build.json")),
    ("debuglog", include_str!("../../../data/cli/debuglog.json")),
    ("dis", include_str!("../../../data/cli/dis.json")),
    ("disk", include_str!("../../../data/cli/disk.json")),
    ("emulator", include_str!("../../../data/cli/emulator.json")),
    ("feedback", include_str!("../../../data/cli/feedback.json")),
    ("import", include_str!("../../../data/cli/import.json")),
    ("inc", include_str!("../../../data/cli/inc.json")),
    ("media", include_str!("../../../data/cli/media.json")),
    ("migrate", include_str!("../../../data/cli/migrate.json")),
    ("new", include_str!("../../../data/cli/new.json")),
    ("organize", include_str!("../../../data/cli/organize.json")),
    ("roms", include_str!("../../../data/cli/roms.json")),
    ("streams", include_str!("../../../data/cli/streams.json")),
    ("test", include_str!("../../../data/cli/test.json")),
];

#[derive(Debug, Clone, Default)]
pub struct Opt { pub flag: String, pub short: String, pub value: String, pub group: String, pub help: String, pub choices: Vec<String>, pub default: String, pub default_text: String, pub repeat: bool, pub required: bool }
#[derive(Debug, Clone, Default)]
pub struct Arg { pub name: String, pub required: bool, pub repeat: bool, pub help: String }
/// `run`: Beispiel darf der Test mit Beispieldateien ausführen (noch nicht ausgewertet; siehe docs/cli-hilfe.md)
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Example { pub title: String, pub cmd: String, pub run: bool }
#[derive(Debug, Clone, Default)]
pub struct Form {
    pub id: String, pub usage: String, pub summary: String, pub args: Vec<Arg>, pub option_groups: Vec<(String, String)>, pub options: Vec<Opt>,
    pub examples: Vec<Example>, pub notes: Vec<String>, pub exit: Vec<(String, String)>, pub see_also: Vec<String>,
}
#[derive(Debug, Clone, Default)]
pub struct Cmd { pub name: String, pub group: String, pub summary: String, pub description: Vec<String>, pub aliases: Vec<String>, pub forms: Vec<Form> }
#[derive(Debug, Clone, Default)]
pub struct Prog {
    pub summary: String, pub intro: String, pub groups: Vec<(String, String)>, pub overview_examples: Vec<Example>, pub env: Vec<(String, String)>,
    pub exit: Vec<(String, String)>, pub cmds: Vec<Cmd>,
}

fn s(v: &Value, k: &str) -> String { v[k].as_str().unwrap_or("").to_string() }
fn b(v: &Value, k: &str) -> bool { v[k].as_bool().unwrap_or(false) }
fn strs(v: &Value, k: &str) -> Vec<String> { v[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default() }
fn pairs(v: &Value, k1: &str, k2: &str) -> Vec<(String, String)> { v.as_array().map(|a| a.iter().map(|x| (s(x, k1), s(x, k2))).collect()).unwrap_or_default() }

fn parse_example(x: &Value) -> Example { Example { title: s(x, "title"), cmd: s(x, "cmd"), run: b(x, "run") } }

fn parse_form(f: &Value) -> Form {
    Form {
        id: s(f, "id"), usage: s(f, "usage"), summary: s(f, "summary"),
        args: f["args"].as_array().map(|a| a.iter().map(|x| Arg { name: s(x, "name"), required: b(x, "required"), repeat: b(x, "repeat"), help: s(x, "help") }).collect()).unwrap_or_default(),
        option_groups: pairs(&f["option_groups"], "id", "label"),
        options: f["options"].as_array().map(|a| a.iter().map(|x| Opt {
            flag: s(x, "flag"), short: s(x, "short"), value: s(x, "value"), group: s(x, "group"), help: s(x, "help"), choices: strs(x, "choices"),
            default: s(x, "default"), default_text: s(x, "default_text"), repeat: b(x, "repeat"), required: b(x, "required") }).collect()).unwrap_or_default(),
        examples: f["examples"].as_array().map(|a| a.iter().map(parse_example).collect()).unwrap_or_default(),
        notes: strs(f, "notes"),
        exit: f["exit"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()).unwrap_or_default(),
        see_also: strs(f, "see_also"),
    }
}

/// Lädt die eingebetteten Beschreibungen (einmal); ein Fehler in einer Datei ist ein Programmierfehler und wird beim Test gefunden.
pub fn prog() -> &'static Prog {
    static P: OnceLock<Prog> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = Prog::default();
        for (name, text) in FILES {
            let v: Value = match serde_json::from_str(text) { Ok(v) => v, Err(e) => { eprintln!("Hilfe: data/cli/{name}.json ist ungültig: {e}"); continue; } };
            if *name == "_vcc" {
                p.summary = s(&v, "summary"); p.intro = s(&v, "intro"); p.groups = pairs(&v["groups"], "id", "label");
                p.overview_examples = v["overview_examples"].as_array().map(|a| a.iter().map(parse_example).collect()).unwrap_or_default();
                p.env = pairs(&v["env"], "name", "help"); p.exit = pairs(&v["exit"], "code", "help");
            } else {
                p.cmds.push(Cmd { name: s(&v, "command"), group: s(&v, "group"), summary: s(&v, "summary"), description: strs(&v, "description"), aliases: strs(&v, "aliases"),
                    forms: v["forms"].as_array().map(|a| a.iter().map(parse_form).collect()).unwrap_or_default() });
            }
        }
        p
    })
}

pub fn find(name: &str) -> Option<&'static Cmd> { prog().cmds.iter().find(|c| c.name == name || c.aliases.iter().any(|a| a == name)) }

// ---------------------------------------------------------------------------------------------------------------------
// Darstellung
// ---------------------------------------------------------------------------------------------------------------------
/// Breite für den Umbruch: `COLUMNS`, höchstens 100, mindestens 50
fn width() -> usize { std::env::var("COLUMNS").ok().and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(100).clamp(50, 100) }

/// Farbe nur auf einem Terminal, ohne `NO_COLOR`; unter Windows nur im Windows Terminal (`WT_SESSION`)
fn color() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() && (cfg!(not(windows)) || std::env::var_os("WT_SESSION").is_some())
}

struct Style { on: bool }
impl Style {
    fn bold(&self, t: &str) -> String { if self.on { format!("\x1b[1m{t}\x1b[0m") } else { t.to_string() } }
    fn dim(&self, t: &str) -> String { if self.on { format!("\x1b[2m{t}\x1b[0m") } else { t.to_string() } }
}

/// Bricht `text` auf `w` Zeichen um; die erste Zeile beginnt nach `first` Zeichen Einrückung, die folgenden nach `indent`.
fn wrap(text: &str, w: usize, first: usize, indent: usize) -> Vec<String> {
    let (mut lines, mut cur) = (vec![], String::new());
    let mut avail = w.saturating_sub(first).max(20);
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > avail { lines.push(std::mem::take(&mut cur)); avail = w.saturating_sub(indent).max(20); }
        if !cur.is_empty() { cur.push(' '); }
        cur.push_str(word);
    }
    if !cur.is_empty() || lines.is_empty() { lines.push(cur); }
    lines
}

fn pad(t: &str, n: usize) -> String { let l = t.chars().count(); format!("{t}{}", " ".repeat(n.saturating_sub(l))) }

fn para(out: &mut String, text: &str, ind: usize, w: usize) {
    for l in wrap(text, w, ind, ind) { let _ = writeln!(out, "{}{l}", " ".repeat(ind)); }
}

/// Zeile der Übersicht: Name und Kurztext
fn overview_rows(out: &mut String, st: &Style, w: usize, short: bool) {
    let p = prog();
    let namew = p.cmds.iter().map(|c| c.name.chars().count()).max().unwrap_or(8) + 2;
    for (gid, glabel) in &p.groups {
        let cmds: Vec<&Cmd> = p.cmds.iter().filter(|c| &c.group == gid).collect();
        if cmds.is_empty() { continue; }
        let _ = writeln!(out, "\n{}", st.bold(glabel));
        for c in cmds {
            let lines = wrap(&c.summary, w, 2 + namew, 2 + namew);
            let _ = writeln!(out, "  {}{}", pad(&c.name, namew), lines[0]);
            for l in &lines[1..] { let _ = writeln!(out, "  {}{l}", " ".repeat(namew)); }
        }
    }
    let _ = short;
}

/// `vcc --help` (kurz = auch ohne Argumente)
pub fn overview(short: bool) -> String {
    let (st, w, p) = (Style { on: color() }, width(), prog());
    let mut out = String::new();
    let _ = writeln!(out, "{} {} - {}", st.bold("vcc"), env!("CARGO_PKG_VERSION"), p.summary);
    let _ = writeln!(out, "\n{}  vcc <befehl> [optionen]      {}", st.bold("Aufruf:"), st.dim("Hilfe zu einem Befehl: vcc help <befehl>  oder  vcc <befehl> --help"));
    overview_rows(&mut out, &st, w, short);
    let _ = writeln!(out, "\n{}", st.bold("Allgemein"));
    let _ = writeln!(out, "  {}Hilfe zu einem Befehl oder Thema (env, exit-codes)", pad("help", 14));
    let _ = writeln!(out, "  {}man-Seite eines Befehls (roff) ausgeben", pad("man", 14));
    let _ = writeln!(out, "  {}Vervollständigung für powershell, bash, zsh oder fish ausgeben", pad("completion", 14));
    let _ = writeln!(out, "  {}Version anzeigen", pad("version", 14));
    let _ = writeln!(out, "\n{}", st.bold("Beispiele"));
    let cw = p.overview_examples.iter().map(|e| e.cmd.chars().count()).max().unwrap_or(0) + 2;
    for e in &p.overview_examples { let _ = writeln!(out, "  {}{}", pad(&e.cmd, cw), st.dim(&format!("# {}", e.title))); }
    let _ = writeln!(out, "\nAusführlicher: vcc help --all   ·   Umgebungsvariablen: vcc help env   ·   Rückgabewerte: vcc help exit-codes");
    out
}

fn option_label(o: &Opt) -> String {
    let mut l = String::new();
    if !o.short.is_empty() { l.push_str(&o.short); l.push_str(", "); } else { l.push_str("    "); }
    l.push_str(&o.flag);
    if !o.value.is_empty() { let _ = write!(l, " <{}>", o.value); }
    l
}

fn option_text(o: &Opt) -> String {
    let mut t = o.help.clone();
    if !o.choices.is_empty() { let _ = write!(t, " Werte: {}.", o.choices.join(", ")); }
    if !o.default.is_empty() { let _ = write!(t, " (Standard: {})", o.default); } else if !o.default_text.is_empty() { let _ = write!(t, " (Standard: {})", o.default_text); }
    if o.repeat { t.push_str(" Mehrfach möglich."); }
    if o.required { t.push_str(" Erforderlich."); }
    t
}

fn render_form(out: &mut String, st: &Style, w: usize, f: &Form, multi: bool) {
    if multi { let _ = writeln!(out, "\n{}", st.bold(&format!("{}", if f.id.is_empty() { "Grundform".to_string() } else { format!("Form „{}“", f.id) }))); if !f.summary.is_empty() { para(out, &f.summary, 2, w); } }
    let _ = writeln!(out, "\n{}  {}", st.bold("Aufruf:"), f.usage);
    if !f.args.is_empty() {
        let _ = writeln!(out, "\n{}", st.bold("Argumente"));
        let lw = f.args.iter().map(|a| a.name.chars().count()).max().unwrap_or(0) + 2;
        for a in &f.args {
            let mut t = a.help.clone();
            if !a.required { t.push_str(" (optional)"); }
            if a.repeat { t.push_str(" Mehrfach möglich."); }
            let ls = wrap(&t, w, 2 + lw, 2 + lw);
            let _ = writeln!(out, "  {}{}", pad(&a.name, lw), ls[0]);
            for l in &ls[1..] { let _ = writeln!(out, "  {}{l}", " ".repeat(lw)); }
        }
    }
    if !f.options.is_empty() {
        let lw = f.options.iter().map(|o| option_label(o).chars().count()).max().unwrap_or(0).min(34) + 2;
        let mut groups: Vec<(String, String)> = f.option_groups.clone();
        if f.options.iter().any(|o| o.group.is_empty() || !groups.iter().any(|g| g.0 == o.group)) { groups.push((String::new(), if groups.is_empty() { "Optionen".into() } else { "Weitere Optionen".into() })); }
        for (gid, glabel) in &groups {
            let opts: Vec<&Opt> = f.options.iter().filter(|o| if gid.is_empty() { o.group.is_empty() || !f.option_groups.iter().any(|g| g.0 == o.group) } else { &o.group == gid }).collect();
            if opts.is_empty() { continue; }
            let _ = writeln!(out, "\n{}", st.bold(glabel));
            for o in opts {
                let lab = option_label(o);
                let ls = wrap(&option_text(o), w, 2 + lw, 2 + lw);
                if lab.chars().count() + 2 > lw { let _ = writeln!(out, "  {lab}"); for l in &ls { let _ = writeln!(out, "  {}{l}", " ".repeat(lw)); } }
                else { let _ = writeln!(out, "  {}{}", pad(&lab, lw), ls[0]); for l in &ls[1..] { let _ = writeln!(out, "  {}{l}", " ".repeat(lw)); } }
            }
        }
    }
    if !f.examples.is_empty() {
        let _ = writeln!(out, "\n{}", st.bold("Beispiele"));
        for e in &f.examples { let _ = writeln!(out, "  {}", st.dim(&format!("# {}", e.title))); let _ = writeln!(out, "  {}\n", e.cmd); }
        while out.ends_with("\n\n") { out.pop(); }
    }
    for n in &f.notes { let _ = writeln!(out); para(out, &format!("Hinweis: {n}"), 0, w); }
    let exit = if f.exit.is_empty() { prog().exit.clone() } else { f.exit.clone() };
    if !f.exit.is_empty() { let _ = writeln!(out, "\n{}  {}", st.bold("Rückgabe:"), exit.iter().map(|(k, v)| format!("{k} = {v}")).collect::<Vec<_>>().join(" · ")); }
    if !f.see_also.is_empty() { let _ = writeln!(out, "\n{}  {}", st.bold("Siehe auch:"), f.see_also.iter().map(|x| format!("vcc help {x}")).collect::<Vec<_>>().join(" · ")); }
}

/// Hilfe eines Befehls; `form` wählt eine Form (leer: alle)
pub fn command_help(c: &Cmd, form: Option<&str>) -> String {
    let (st, w) = (Style { on: color() }, width());
    let mut out = String::new();
    let _ = writeln!(out, "{} - {}", st.bold(&format!("vcc {}", c.name)), c.summary);
    if !c.aliases.is_empty() { let _ = writeln!(out, "Auch als: {}", c.aliases.iter().map(|a| format!("vcc {a}")).collect::<Vec<_>>().join(", ")); }
    for d in &c.description { let _ = writeln!(out); para(&mut out, d, 0, w); }
    let forms: Vec<&Form> = match form { Some(id) => c.forms.iter().filter(|f| f.id == id).collect(), None => c.forms.iter().collect() };
    let multi = c.forms.len() > 1 && form.is_none();
    for f in forms { render_form(&mut out, &st, w, f, multi); }
    out
}

pub fn help_all() -> String {
    let mut out = overview(false);
    for c in &prog().cmds { let _ = writeln!(out, "\n{}\n", "=".repeat(width().min(60))); out.push_str(&command_help(c, None)); }
    out
}

pub fn env_help() -> String {
    let st = Style { on: color() };
    let mut out = format!("{}\n", st.bold("Umgebungsvariablen"));
    let lw = prog().env.iter().map(|e| e.0.len()).max().unwrap_or(8) + 2;
    for (n, h) in &prog().env { let ls = wrap(h, width(), 2 + lw, 2 + lw); let _ = writeln!(out, "  {}{}", pad(n, lw), ls[0]); for l in &ls[1..] { let _ = writeln!(out, "  {}{l}", " ".repeat(lw)); } }
    out
}

pub fn exit_help() -> String {
    let st = Style { on: color() };
    let mut out = format!("{}\n", st.bold("Rückgabewerte"));
    for (c, h) in &prog().exit { let _ = writeln!(out, "  {}  {h}", pad(c, 2)); }
    out
}

// ---------------------------------------------------------------------------------------------------------------------
// Fehler mit Vorschlag
// ---------------------------------------------------------------------------------------------------------------------
/// Editierabstand (Einfügen, Löschen, Ersetzen und Vertauschen benachbarter Zeichen zählen je 1)
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..=a.len() { d[i][0] = i; }
    for j in 0..=b.len() { d[0][j] = j; }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] { d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1); }
        }
    }
    d[a.len()][b.len()]
}

/// Bester Vorschlag unter `cands` (Abstand höchstens 2 bzw. ein Drittel der Länge, oder Wort ist Anfang des Kandidaten)
pub fn suggest<'a>(word: &str, cands: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let w = word.trim_start_matches('-');
    let mut best: Option<(usize, String)> = None;
    for c in cands {
        let cc = c.trim_start_matches('-');
        if cc.is_empty() { continue; }
        let d = if w.len() >= 2 && cc.starts_with(w) { 0 } else { distance(w, cc) };
        if d <= 2.max(cc.len() / 3).min(3) && best.as_ref().map_or(true, |(bd, _)| d < *bd) { best = Some((d, c.to_string())); }
    }
    best.map(|(_, c)| c)
}

/// Aufruf unvollständig oder ungültig (fehlendes Argument, unbekannter Unterbefehl): Aufrufzeilen und Hinweis; Exit-Code 2
pub fn usage_error(cmd: &str) -> ExitCode {
    eprintln!("vcc {cmd}: Aufruf unvollständig oder ungültig");
    if let Some(c) = find(cmd) { eprintln!(); for f in &c.forms { eprintln!("  {}", f.usage); } }
    eprintln!("\nHilfe:   vcc {cmd} --help");
    ExitCode::from(2)
}

pub fn unknown_command(word: &str) -> ExitCode {
    eprintln!("vcc: unbekannter Befehl „{word}“");
    let names: Vec<&str> = prog().cmds.iter().flat_map(|c| std::iter::once(c.name.as_str()).chain(c.aliases.iter().map(String::as_str))).chain(["help", "man", "completion", "version"]).collect();
    if let Some(s) = suggest(word, names) { eprintln!("  Meinten Sie {s}?"); }
    eprintln!("\nAlle Befehle: vcc --help");
    ExitCode::from(2)
}

/// Unbekannte Option bei Befehl `cmd`: eine Zeile, Vorschlag, Hinweis auf die Hilfe; Exit-Code 2
pub fn unknown_option(cmd: &str, opt: &str) -> ExitCode {
    eprintln!("vcc {cmd}: unbekannte Option „{opt}“");
    if let Some(c) = find(cmd) {
        let flags: Vec<&str> = c.forms.iter().flat_map(|f| f.options.iter().flat_map(|o| [o.flag.as_str(), o.short.as_str()])).filter(|f| !f.is_empty()).collect();
        if let Some(s) = suggest(opt, flags) { eprintln!("  Meinten Sie {s}?"); }
        if let Some(f) = c.forms.first() { eprintln!("\nAufruf:  {}", f.usage); }
    }
    eprintln!("Hilfe:   vcc {cmd} --help");
    ExitCode::from(2)
}

// ---------------------------------------------------------------------------------------------------------------------
// Einstieg aus main
// ---------------------------------------------------------------------------------------------------------------------
fn out(text: &str) -> ExitCode { print!("{text}"); ExitCode::SUCCESS }

fn is_help(a: &str) -> bool { a == "--help" || a == "-h" }

fn form_of(c: &Cmd, rest: &[String]) -> Option<String> {
    let first = rest.iter().find(|a| !is_help(a))?;
    let id = first.trim_start_matches('-');
    c.forms.iter().find(|f| !f.id.is_empty() && f.id == id).map(|f| f.id.clone())
}

/// Behandelt Hilfe-Aufrufe (`vcc`, `--help`, `help …`, `<befehl> --help`, `man`, `completion`); `None`: kein Hilfe-Aufruf, der Befehl läuft normal.
pub fn dispatch(args: &[String]) -> Option<ExitCode> {
    let first = match args.first() { None => { eprint!("{}", overview(true)); return Some(ExitCode::from(2)); } Some(f) => f.as_str() };
    if is_help(first) { return Some(out(&overview(false))); }
    match first {
        "help" => {
            let rest = &args[1..];
            return Some(match rest.first().map(String::as_str) {
                None => out(&overview(false)),
                Some("--all") | Some("all") => out(&help_all()),
                Some("--markdown") => out(&crate::markdown::render()),
                Some("env") => out(&env_help()),
                Some("exit-codes") => out(&exit_help()),
                Some(name) => match find(name) {
                    Some(c) => out(&command_help(c, form_of(c, &rest[1..]).as_deref())),
                    None => unknown_command(name),
                },
            });
        }
        "man" => return Some(crate::manpage::run(&args[1..])),
        "completion" => return Some(crate::completion::run(&args[1..])),
        _ => {}
    }
    if let Some(c) = find(first) {
        let rest = &args[1..];
        if rest.iter().any(|a| is_help(a)) { return Some(out(&command_help(c, form_of(c, rest).as_deref()))); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const MAIN: &str = include_str!("main.rs");
    const ROMS: &str = include_str!("roms.rs");
    const EMULATOR: &str = include_str!("emulator.rs");

    /// Befehle, die `main` verteilt (`Some("asm") =>`)
    fn dispatched() -> BTreeSet<String> {
        let mut v = BTreeSet::new();
        let body = MAIN.split("fn main()").nth(1).unwrap_or("").split("\nfn ").next().unwrap_or("");
        for part in body.split("Some(\"").skip(1) {
            if let Some(n) = part.split('"').next() {
                if part[n.len()..].starts_with("\")") && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) { v.insert(n.to_string()); }
            }
        }
        for n in ["version", "help", "man", "completion"] { v.remove(n); }
        v
    }

    fn is_flag(l: &str) -> bool {
        (l.starts_with("--") && l.len() > 2 && l[2..].chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            || (l.len() == 2 && l.starts_with('-') && l.as_bytes()[1].is_ascii_alphabetic())
    }

    /// Alle Zeichenfolgen eines Quelltextes (ohne die Zeichen-Literale `'"'` und Lebensdauern durcheinanderzubringen)
    fn string_literals(src: &str) -> Vec<String> {
        let b: Vec<char> = src.chars().collect();
        let (mut i, mut out) = (0, vec![]);
        while i < b.len() {
            match b[i] {
                '"' => {
                    let mut j = i + 1;
                    let mut lit = String::new();
                    while j < b.len() && b[j] != '"' { if b[j] == '\\' && j + 1 < b.len() { lit.push(b[j]); j += 1; } lit.push(b[j]); j += 1; }
                    out.push(lit);
                    i = j + 1;
                }
                '\'' => {
                    if i + 2 < b.len() && b[i + 1] == '\\' { let mut j = i + 2; while j < b.len() && b[j] != '\'' { j += 1; } i = j + 1; }
                    else if i + 2 < b.len() && b[i + 2] == '\'' { i += 3; }
                    else { i += 1; }
                }
                _ => i += 1,
            }
        }
        out
    }

    /// Quelltext ohne das Test-Modul (`#[cfg(test)] mod tests { ... }`), dessen Beispiele keine Optionen des Programms sind
    fn without_tests(src: &str) -> String {
        let Some(i) = src.find("#[cfg(test)]") else { return src.to_string() };
        let Some(open) = src[i..].find('{').map(|x| x + i) else { return src.to_string() };
        let (mut depth, mut end) = (0i32, src.len());
        for (k, c) in src[open..].char_indices() {
            if c == '{' { depth += 1; } else if c == '}' { depth -= 1; if depth == 0 { end = open + k + 1; break; } }
        }
        format!("{}{}", &src[..i], &src[end..])
    }

    /// Alle Optionen, die der Quelltext der Befehle als Zeichenfolge kennt (`"--target"`, `"-D"`)
    fn flags_in_source() -> BTreeSet<String> {
        let mut v = BTreeSet::new();
        for src in [MAIN, ROMS, EMULATOR] {
            for lit in string_literals(&without_tests(src)) { if is_flag(&lit) { v.insert(lit); } }
        }
        v
    }

    #[test]
    fn beschreibungen_sind_gueltig_und_vollstaendig() {
        let p = prog();
        assert!(!p.summary.is_empty() && !p.groups.is_empty());
        let names: BTreeSet<String> = p.cmds.iter().map(|c| c.name.clone()).collect();
        let disp: BTreeSet<String> = dispatched().into_iter().filter(|n| n != "asset" && n != "ast").collect();
        assert_eq!(names, disp, "Befehle der Beschreibung und von main weichen ab");
        let mut errs = vec![];
        for c in &p.cmds {
            if c.summary.is_empty() || c.summary.chars().count() > 70 || c.summary.ends_with('.') { errs.push(format!("{}: summary leer, über 70 Zeichen oder mit Punkt", c.name)); }
            if !p.groups.iter().any(|g| g.0 == c.group) { errs.push(format!("{}: unbekannte Gruppe {}", c.name, c.group)); }
            if c.forms.is_empty() { errs.push(format!("{}: keine Form", c.name)); }
            if c.forms.iter().filter(|f| f.id.is_empty()).count() > 1 { errs.push(format!("{}: mehrere Formen ohne id", c.name)); }
            for f in &c.forms {
                let id = format!("{} {}", c.name, f.id);
                if !f.usage.starts_with("vcc ") { errs.push(format!("{id}: usage beginnt nicht mit vcc")); }
                if c.forms.len() > 1 && f.summary.is_empty() { errs.push(format!("{id}: Form ohne summary")); }
                if f.examples.is_empty() { errs.push(format!("{id}: kein Beispiel")); }
                for o in &f.options {
                    if o.flag.is_empty() || o.help.is_empty() { errs.push(format!("{id}: Option {:?} ohne flag oder help", o.flag)); }
                    if !o.choices.is_empty() && o.value.is_empty() { errs.push(format!("{id}: {} hat Werte, aber keinen value", o.flag)); }
                    if !o.group.is_empty() && !f.option_groups.iter().any(|g| g.0 == o.group) { errs.push(format!("{id}: {} nennt unbekannte Gruppe {}", o.flag, o.group)); }
                }
                for a in &f.args { if a.name.is_empty() || a.help.is_empty() { errs.push(format!("{id}: Argument ohne name oder help")); } }
                for x in &f.see_also { if find(x).is_none() { errs.push(format!("{id}: siehe auch unbekannter Befehl {x}")); } }
            }
        }
        assert!(errs.is_empty(), "{}", errs.join("\n"));
    }

    #[test]
    fn optionen_stimmen_mit_dem_quelltext_ueberein() {
        let src = flags_in_source();
        let mut table: BTreeSet<String> = BTreeSet::new();
        for c in &prog().cmds { for f in &c.forms { for o in &f.options { table.insert(o.flag.clone()); if !o.short.is_empty() { table.insert(o.short.clone()); } } } }
        // Schalter, die nur die Hilfe selbst betreffen
        let own: BTreeSet<&str> = ["--help", "-h", "--version", "-V", "--all"].into_iter().collect();
        let missing: Vec<&String> = src.iter().filter(|f| !table.contains(*f) && !own.contains(f.as_str())).collect();
        let stale: Vec<&String> = table.iter().filter(|f| !src.contains(*f)).collect();
        assert!(missing.is_empty(), "Optionen im Quelltext, aber nicht in data/cli: {missing:?}");
        assert!(stale.is_empty(), "Optionen in data/cli, aber nicht im Quelltext: {stale:?}");
    }

    #[test]
    fn beispiele_benutzen_bekannte_befehle_und_optionen() {
        let mut errs = vec![];
        let mut all: Vec<(&str, &Example)> = vec![];
        for e in &prog().overview_examples { all.push(("Übersicht", e)); }
        for c in &prog().cmds { for f in &c.forms { for e in &f.examples { all.push((c.name.as_str(), e)); } } }
        for (who, e) in all {
            let toks: Vec<&str> = e.cmd.split_whitespace().collect();
            if toks.first() != Some(&"vcc") { errs.push(format!("{who}: Beispiel beginnt nicht mit vcc: {}", e.cmd)); continue; }
            let Some(name) = toks.get(1) else { continue };
            let Some(c) = find(name) else { errs.push(format!("{who}: Beispiel mit unbekanntem Befehl: {}", e.cmd)); continue };
            for t in &toks[2..] {
                if !t.starts_with('-') || t.len() < 2 || (t.starts_with("-D") && t.len() > 2) { continue; }
                let flag = t.split('=').next().unwrap();
                let known = c.forms.iter().any(|f| f.options.iter().any(|o| o.flag == flag || o.short == flag)) || c.forms.iter().any(|f| f.id == flag.trim_start_matches('-'));
                if !known { errs.push(format!("{who}: Beispiel benutzt unbekannte Option {flag}: {}", e.cmd)); }
            }
        }
        assert!(errs.is_empty(), "{}", errs.join("\n"));
    }

    #[test]
    fn hilfe_und_fehlertexte() {
        assert!(overview(false).contains("Programme bauen"));
        assert!(help_all().contains("vcc asm"));
        let h = command_help(find("asm").unwrap(), None);
        assert!(h.contains("Aufruf:") && h.contains("Beispiele"));
        assert_eq!(suggest("--targt", ["--target", "--format"]), Some("--target".to_string()));
        assert_eq!(suggest("tset", ["test", "asm"]), Some("test".to_string()));
        assert_eq!(suggest("zzzzzz", ["test", "asm"]), None);
        assert!(find("asset").is_some(), "Alias asset -> media");
    }

    #[test]
    fn man_und_vervollstaendigung() {
        let pages = crate::manpage::all_pages();
        assert_eq!(pages.len(), prog().cmds.len() + 3, "vcc, alle Befehle, vintagecoder, vc-lsp");
        for (n, t) in &pages {
            assert!(t.contains(".TH ") && t.contains(".SH NAME"), "{n}");
            assert!(!t.lines().any(|l| l.starts_with("..")), "{n}");
        }
        assert!(pages.iter().any(|(n, _)| n == "vcc-asm.1"));
    }

    #[test]
    fn man_seiten_im_quellbaum_sind_aktuell() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../installer/linux/man");
        let mut stale = vec![];
        for (name, text) in crate::manpage::all_pages() {
            let on_disk = std::fs::read_to_string(dir.join(&name)).unwrap_or_default().replace("\r\n", "\n");
            if on_disk != text { stale.push(name); }
        }
        assert!(stale.is_empty(), "veraltet oder fehlt unter installer/linux/man: {stale:?}; neu erzeugen mit: vcc man --all installer/linux/man");
    }

    #[test]
    fn handbuch_im_quellbaum_ist_aktuell() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/cli.md");
        let on_disk = std::fs::read_to_string(path).unwrap_or_default().replace("\r\n", "\n");
        assert_eq!(on_disk, crate::markdown::render(), "docs/cli.md ist veraltet; neu erzeugen mit: vcc help --markdown > docs/cli.md");
    }
}
