// SPDX-License-Identifier: GPL-3.0-or-later
//! man-Seiten (roff, Makros `man`) aus den Beschreibungen in `data/cli`: `vcc man [befehl]`, `vcc man --all <ordner>`.
use crate::help::{self, Cmd, Form, Opt};
use std::fmt::Write as _;
use std::process::ExitCode;

/// roff-Text: Backslash, Bindestrich und führender Punkt/Apostroph so schreiben, dass sie wörtlich erscheinen
fn esc(t: &str) -> String {
    let mut o = t.replace('\\', "\\e").replace('-', "\\-");
    if o.starts_with('.') || o.starts_with('\'') { o.insert_str(0, "\\&"); }
    o
}

fn th(out: &mut String, name: &str) {
    let _ = writeln!(out, ".\\\" Erzeugt von \"vcc man\"; nicht von Hand ändern (Quelle: data/cli/*.json).");
    let _ = writeln!(out, ".\\\" -*- coding: UTF-8 -*-");
    let _ = writeln!(out, ".TH {} 1 \"\" \"VintageCoder {}\" \"VintageCoder Handbuch\"", name.to_uppercase().replace('-', "\\-"), env!("CARGO_PKG_VERSION"));
}

fn option_line(o: &Opt) -> String {
    let mut l = String::new();
    if !o.short.is_empty() { let _ = write!(l, "\\fB{}\\fR, ", esc(&o.short)); }
    let _ = write!(l, "\\fB{}\\fR", esc(&o.flag));
    if !o.value.is_empty() { let _ = write!(l, " \\fI{}\\fR", esc(&o.value)); }
    l
}

fn option_text(o: &Opt) -> String {
    let mut t = o.help.clone();
    if !o.choices.is_empty() { let _ = write!(t, " Werte: {}.", o.choices.join(", ")); }
    if !o.default.is_empty() { let _ = write!(t, " Standard: {}.", o.default); } else if !o.default_text.is_empty() { let _ = write!(t, " Standard: {}.", o.default_text); }
    if o.repeat { t.push_str(" Mehrfach möglich."); }
    if o.required { t.push_str(" Erforderlich."); }
    t
}

fn form_body(out: &mut String, f: &Form, multi: bool) {
    if multi {
        let _ = writeln!(out, ".SS {}", esc(&if f.id.is_empty() { "Grundform".to_string() } else { format!("Form {}", f.id) }));
        if !f.summary.is_empty() { let _ = writeln!(out, "{}", esc(&f.summary)); let _ = writeln!(out, ".PP"); }
    }
    if !f.args.is_empty() {
        for a in &f.args {
            let _ = writeln!(out, ".TP\n\\fI{}\\fR", esc(&a.name));
            let mut t = a.help.clone();
            if !a.required { t.push_str(" (optional)"); }
            if a.repeat { t.push_str(" Mehrfach möglich."); }
            let _ = writeln!(out, "{}", esc(&t));
        }
    }
    let mut seen_groups: Vec<String> = vec![];
    for o in &f.options {
        let g = f.option_groups.iter().find(|g| g.0 == o.group).map(|g| g.1.clone()).unwrap_or_default();
        if !seen_groups.contains(&g) && (!g.is_empty() || !f.option_groups.is_empty()) { let _ = writeln!(out, ".PP\n\\fB{}\\fR", esc(&if g.is_empty() { "Weitere Optionen".to_string() } else { g.clone() })); }
        if !seen_groups.contains(&g) { seen_groups.push(g); }
        let _ = writeln!(out, ".TP\n{}\n{}", option_line(o), esc(&option_text(o)));
    }
}

fn examples(out: &mut String, forms: &[Form]) {
    if forms.iter().all(|f| f.examples.is_empty()) { return; }
    let _ = writeln!(out, ".SH BEISPIELE");
    for f in forms {
        for e in &f.examples {
            let _ = writeln!(out, "{}:\n.PP\n.RS 8\n.nf\n{}\n.fi\n.RE", esc(&e.title), esc(&e.cmd));
        }
    }
}

pub fn command_page(c: &Cmd) -> String {
    let mut out = String::new();
    th(&mut out, &format!("vcc-{}", c.name));
    let _ = writeln!(out, ".SH NAME\nvcc\\-{} \\- {}", esc(&c.name), esc(&c.summary));
    let _ = writeln!(out, ".SH ÜBERSICHT");
    for f in &c.forms { let _ = writeln!(out, ".B {}\n.br", esc(&f.usage)); }
    let _ = writeln!(out, ".SH BESCHREIBUNG");
    let _ = writeln!(out, "{}", esc(&c.summary));
    for d in &c.description { let _ = writeln!(out, ".PP\n{}", esc(d)); }
    if !c.aliases.is_empty() { let _ = writeln!(out, ".PP\nAuch als: {}.", esc(&c.aliases.iter().map(|a| format!("vcc {a}")).collect::<Vec<_>>().join(", "))); }
    let has_opts = c.forms.iter().any(|f| !f.options.is_empty() || !f.args.is_empty());
    if has_opts {
        let _ = writeln!(out, ".SH OPTIONEN");
        let multi = c.forms.len() > 1;
        for f in &c.forms { form_body(&mut out, f, multi); }
    }
    examples(&mut out, &c.forms);
    let notes: Vec<&String> = c.forms.iter().flat_map(|f| f.notes.iter()).collect();
    if !notes.is_empty() { let _ = writeln!(out, ".SH HINWEISE"); for n in notes { let _ = writeln!(out, "{}\n.PP", esc(n)); } }
    let _ = writeln!(out, ".SH RÜCKGABEWERT");
    let exit = c.forms.iter().find(|f| !f.exit.is_empty()).map(|f| f.exit.clone()).unwrap_or_else(|| help::prog().exit.clone());
    let _ = writeln!(out, "{}", esc(&exit.iter().map(|(k, v)| format!("{k} = {v}")).collect::<Vec<_>>().join(", ")));
    let mut see: Vec<String> = vec!["vcc(1)".into()];
    for f in &c.forms { for s in &f.see_also { let x = format!("vcc\\-{}(1)", esc(s)); if !see.contains(&x) { see.push(x); } } }
    let _ = writeln!(out, ".SH SIEHE AUCH\n{}", see.join(", "));
    out
}

pub fn overview_page() -> String {
    let p = help::prog();
    let mut out = String::new();
    th(&mut out, "vcc");
    let _ = writeln!(out, ".SH NAME\nvcc \\- {}", esc(&p.summary));
    let _ = writeln!(out, ".SH ÜBERSICHT\n.B vcc\n.I befehl\n[\\c\n.IR optionen ]\n.br\n.B vcc help\n[\\c\n.IR befehl ]");
    let _ = writeln!(out, ".SH BESCHREIBUNG\n{}", esc(&p.intro));
    let _ = writeln!(out, ".SH BEFEHLE");
    for (gid, glabel) in &p.groups {
        let cmds: Vec<&Cmd> = p.cmds.iter().filter(|c| &c.group == gid).collect();
        if cmds.is_empty() { continue; }
        let _ = writeln!(out, ".SS {}", esc(glabel));
        for c in cmds { let _ = writeln!(out, ".TP\n\\fBvcc {}\\fR\n{} (siehe \\fBvcc\\-{}\\fR(1))", esc(&c.name), esc(&c.summary), esc(&c.name)); }
    }
    let _ = writeln!(out, ".SS Allgemein\n.TP\n\\fBvcc help\\fR [\\fIbefehl\\fR]\nHilfe zu einem Befehl oder Thema (env, exit\\-codes).\n.TP\n\\fBvcc man\\fR [\\fIbefehl\\fR]\nman\\-Seite ausgeben.\n.TP\n\\fBvcc completion\\fR \\fIshell\\fR\nVervollständigung für powershell, bash, zsh oder fish ausgeben.");
    let _ = writeln!(out, ".SH BEISPIELE");
    for e in &p.overview_examples { let _ = writeln!(out, "{}:\n.PP\n.RS 8\n.nf\n{}\n.fi\n.RE", esc(&e.title), esc(&e.cmd)); }
    let _ = writeln!(out, ".SH UMGEBUNG");
    for (n, h) in &p.env { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(n), esc(h)); }
    let _ = writeln!(out, ".SH RÜCKGABEWERT");
    for (c, h) in &p.exit { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(c), esc(h)); }
    let _ = writeln!(out, ".SH SIEHE AUCH\n{}", p.cmds.iter().map(|c| format!("vcc\\-{}(1)", esc(&c.name))).collect::<Vec<_>>().join(", "));
    out
}

/// Seiten für die Programme ohne Befehle (`vintagecoder`, `vc-lsp`); Text wie in deren `--help`
fn simple_page(name: &str, summary: &str, synopsis: &str, desc: &[&str], opts: &[(&str, &str)], env: &[(&str, &str)], examples: &[(&str, &str)], exit: &str) -> String {
    let mut out = String::new();
    th(&mut out, name);
    let _ = writeln!(out, ".SH NAME\n{} \\- {}", esc(name), esc(summary));
    let _ = writeln!(out, ".SH ÜBERSICHT\n.B {}", esc(synopsis));
    let _ = writeln!(out, ".SH BESCHREIBUNG");
    for (i, d) in desc.iter().enumerate() { if i > 0 { let _ = writeln!(out, ".PP"); } let _ = writeln!(out, "{}", esc(d)); }
    if !opts.is_empty() { let _ = writeln!(out, ".SH OPTIONEN"); for (o, h) in opts { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(o), esc(h)); } }
    if !env.is_empty() { let _ = writeln!(out, ".SH UMGEBUNG"); for (o, h) in env { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(o), esc(h)); } }
    if !examples.is_empty() { let _ = writeln!(out, ".SH BEISPIELE"); for (t, c) in examples { let _ = writeln!(out, "{}:\n.PP\n.RS 8\n.nf\n{}\n.fi\n.RE", esc(t), esc(c)); } }
    let _ = writeln!(out, ".SH RÜCKGABEWERT\n{}", esc(exit));
    let _ = writeln!(out, ".SH SIEHE AUCH\nvcc(1)");
    out
}

pub fn extra_pages() -> Vec<(String, String)> {
    let env: &[(&str, &str)] = &[("VC_DATA", "Ordner der Hardware-Datenbank (data)."), ("VC_ROMS", "ROM-Ordner des Benutzers."), ("VC_XEMU", "Pfad zum Xemu-Programm."), ("VC_XEMU_SD", "Ordner mit MEGA65.ROM für Xemu."), ("VC_WIN", "Fenstergröße, z. B. 1400x860.")];
    vec![
        ("vintagecoder.1".to_string(), simple_page("vintagecoder", "grafische Entwicklungsumgebung für Retro-Systeme", "vintagecoder [projekt-ordner | datei.vcprj | datei.asm | datei.bas]",
            &["Öffnet die IDE. Ohne Argument erscheint der Startbildschirm, mit einem Pfad das Projekt oder die Datei."],
            &[("-h, --help", "Hilfe anzeigen."), ("-V, --version", "Version anzeigen.")], env,
            &[("Startbildschirm", "vintagecoder"), ("Projekt öffnen", "vintagecoder spiel"), ("Quelltext öffnen", "vintagecoder prog.asm")], "0 nach normalem Beenden.")),
        ("vc-lsp.1".to_string(), simple_page("vc-lsp", "Language Server für VintageCoder-Assembler", "vc-lsp [--help] [--version]",
            &["Spricht das Language Server Protocol über Standardeingabe und Standardausgabe. Er wird normalerweise vom Editor gestartet, nicht von Hand: ohne Eingabe wartet er auf Nachrichten und gibt nichts aus.", "Im Editor als Server mit dem Befehl vc-lsp ohne Argumente für Dateien mit der Endung .asm einbinden."],
            &[("-h, --help", "Hilfe anzeigen."), ("-V, --version", "Version anzeigen.")], &[], &[("Version anzeigen", "vc-lsp --version")], "0 nach geordnetem Beenden (shutdown, exit), sonst 1; 2 bei unbekannter Option.")),
    ]
}

/// Alle Seiten: (Dateiname, Inhalt)
pub fn all_pages() -> Vec<(String, String)> {
    let mut v = vec![("vcc.1".to_string(), overview_page())];
    let mut cmds: Vec<&Cmd> = help::prog().cmds.iter().collect();
    cmds.sort_by(|a, b| a.name.cmp(&b.name));
    for c in cmds { v.push((format!("vcc-{}.1", c.name), command_page(c))); }
    v.extend(extra_pages());
    v
}

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => { print!("{}", overview_page()); ExitCode::SUCCESS }
        Some("--all") => {
            let Some(dir) = args.get(1) else { eprintln!("vcc man --all: Zielordner fehlt\nAufruf:  vcc man --all <ordner>"); return ExitCode::from(2); };
            if let Err(e) = std::fs::create_dir_all(dir) { eprintln!("vcc man: {dir}: {e}"); return ExitCode::FAILURE; }
            for (name, text) in all_pages() {
                if let Err(e) = std::fs::write(std::path::Path::new(dir).join(&name), text) { eprintln!("vcc man: {name}: {e}"); return ExitCode::FAILURE; }
            }
            println!("{} Seiten nach {dir} geschrieben.", all_pages().len());
            ExitCode::SUCCESS
        }
        Some("--help") | Some("-h") => {
            println!("vcc man - man-Seite (roff) ausgeben\n\nAufruf:  vcc man [befehl]\n         vcc man --all <ordner>\n\nBeispiele:\n  vcc man asm | man -l -\n  vcc man --all installer/linux/man");
            ExitCode::SUCCESS
        }
        Some(name) => match help::find(name) {
            Some(c) => { print!("{}", command_page(c)); ExitCode::SUCCESS }
            None => help::unknown_command(name),
        },
    }
}
