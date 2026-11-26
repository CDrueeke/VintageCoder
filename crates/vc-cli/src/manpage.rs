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
    let _ = writeln!(out, "{}", vc_i18n::t!("cli-man-generated-comment"));
    let _ = writeln!(out, ".\\\" -*- coding: UTF-8 -*-");
    let _ = writeln!(out, ".TH {} 1 \"\" \"VintageCoder {}\" \"{}\"", name.to_uppercase().replace('-', "\\-"), env!("CARGO_PKG_VERSION"), vc_i18n::t!("cli-man-manual"));
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
    if !o.choices.is_empty() { let _ = write!(t, " {}", vc_i18n::t!("cli-help-values", values = o.choices.join(", "))); }
    if !o.default.is_empty() { let _ = write!(t, " {}", vc_i18n::t!("cli-man-default", value = &o.default)); } else if !o.default_text.is_empty() { let _ = write!(t, " {}", vc_i18n::t!("cli-man-default", value = &o.default_text)); }
    if o.repeat { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-repeatable"))); }
    if o.required { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-required"))); }
    t
}

fn form_body(out: &mut String, f: &Form, multi: bool) {
    if multi {
        let _ = writeln!(out, ".SS {}", esc(&if f.id.is_empty() { vc_i18n::t!("cli-help-form-base") } else { vc_i18n::t!("cli-man-form-named", id = &f.id) }));
        if !f.summary.is_empty() { let _ = writeln!(out, "{}", esc(&f.summary)); let _ = writeln!(out, ".PP"); }
    }
    if !f.args.is_empty() {
        for a in &f.args {
            let _ = writeln!(out, ".TP\n\\fI{}\\fR", esc(&a.name));
            let mut t = a.help.clone();
            if !a.required { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-optional"))); }
            if a.repeat { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-repeatable"))); }
            let _ = writeln!(out, "{}", esc(&t));
        }
    }
    let mut seen_groups: Vec<String> = vec![];
    for o in &f.options {
        let g = f.option_groups.iter().find(|g| g.0 == o.group).map(|g| g.1.clone()).unwrap_or_default();
        if !seen_groups.contains(&g) && (!g.is_empty() || !f.option_groups.is_empty()) { let _ = writeln!(out, ".PP\n\\fB{}\\fR", esc(&if g.is_empty() { vc_i18n::t!("cli-help-more-options") } else { g.clone() })); }
        if !seen_groups.contains(&g) { seen_groups.push(g); }
        let _ = writeln!(out, ".TP\n{}\n{}", option_line(o), esc(&option_text(o)));
    }
}

fn examples(out: &mut String, forms: &[Form]) {
    if forms.iter().all(|f| f.examples.is_empty()) { return; }
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-examples"));
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
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-synopsis"));
    for f in &c.forms { let _ = writeln!(out, ".B {}\n.br", esc(&f.usage)); }
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-description"));
    let _ = writeln!(out, "{}", esc(&c.summary));
    for d in &c.description { let _ = writeln!(out, ".PP\n{}", esc(d)); }
    if !c.aliases.is_empty() { let _ = writeln!(out, ".PP\n{}", vc_i18n::t!("cli-man-aliases", names = esc(&c.aliases.iter().map(|a| format!("vcc {a}")).collect::<Vec<_>>().join(", ")))); }
    let has_opts = c.forms.iter().any(|f| !f.options.is_empty() || !f.args.is_empty());
    if has_opts {
        let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-options"));
        let multi = c.forms.len() > 1;
        for f in &c.forms { form_body(&mut out, f, multi); }
    }
    examples(&mut out, &c.forms);
    let notes: Vec<&String> = c.forms.iter().flat_map(|f| f.notes.iter()).collect();
    if !notes.is_empty() { let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-notes")); for n in notes { let _ = writeln!(out, "{}\n.PP", esc(n)); } }
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-exit"));
    let exit = c.forms.iter().find(|f| !f.exit.is_empty()).map(|f| f.exit.clone()).unwrap_or_else(|| help::prog().exit.clone());
    let _ = writeln!(out, "{}", esc(&exit.iter().map(|(k, v)| format!("{k} = {v}")).collect::<Vec<_>>().join(", ")));
    let mut see: Vec<String> = vec!["vcc(1)".into()];
    for f in &c.forms { for s in &f.see_also { let x = format!("vcc\\-{}(1)", esc(s)); if !see.contains(&x) { see.push(x); } } }
    let _ = writeln!(out, ".SH {}\n{}", vc_i18n::t!("cli-man-sh-see-also"), see.join(", "));
    out
}

pub fn overview_page() -> String {
    let p = help::prog();
    let mut out = String::new();
    th(&mut out, "vcc");
    let _ = writeln!(out, ".SH NAME\nvcc \\- {}", esc(&p.summary));
    let (arg_cmd, arg_opts) = (vc_i18n::t!("cli-man-arg-command"), vc_i18n::t!("cli-man-arg-options"));
    let _ = writeln!(out, ".SH {}\n.B vcc\n.I {arg_cmd}\n[\\c\n.IR {arg_opts} ]\n.br\n.B vcc help\n[\\c\n.IR {arg_cmd} ]", vc_i18n::t!("cli-man-sh-synopsis"));
    let _ = writeln!(out, ".SH {}\n{}", vc_i18n::t!("cli-man-sh-description"), esc(&p.intro));
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-commands"));
    for (gid, glabel) in &p.groups {
        let cmds: Vec<&Cmd> = p.cmds.iter().filter(|c| &c.group == gid).collect();
        if cmds.is_empty() { continue; }
        let _ = writeln!(out, ".SS {}", esc(glabel));
        for c in cmds { let _ = writeln!(out, ".TP\n\\fBvcc {}\\fR\n{}", esc(&c.name), vc_i18n::t!("cli-man-command-line", summary = esc(&c.summary), name = esc(&c.name))); }
    }
    let _ = writeln!(out, ".SS {}\n.TP\n\\fBvcc help\\fR [\\fI{arg_cmd}\\fR]\n{}\n.TP\n\\fBvcc man\\fR [\\fI{arg_cmd}\\fR]\n{}\n.TP\n\\fBvcc completion\\fR \\fIshell\\fR\n{}", vc_i18n::t!("cli-help-general"),
        esc(&vc_i18n::t!("cli-man-general-help")), esc(&vc_i18n::t!("cli-man-general-man")), esc(&vc_i18n::t!("cli-man-general-completion")));
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-examples"));
    for e in &p.overview_examples { let _ = writeln!(out, "{}:\n.PP\n.RS 8\n.nf\n{}\n.fi\n.RE", esc(&e.title), esc(&e.cmd)); }
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-environment"));
    for (n, h) in &p.env { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(n), esc(h)); }
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-exit"));
    for (c, h) in &p.exit { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(c), esc(h)); }
    let _ = writeln!(out, ".SH {}\n{}", vc_i18n::t!("cli-man-sh-see-also"), p.cmds.iter().map(|c| format!("vcc\\-{}(1)", esc(&c.name))).collect::<Vec<_>>().join(", "));
    out
}

/// Seiten für die Programme ohne Befehle (`vintagecoder`, `vc-lsp`); Text wie in deren `--help`
fn simple_page(name: &str, summary: &str, synopsis: &str, desc: &[String], opts: &[(&str, String)], env: &[(&str, String)], examples: &[(String, &str)], exit: &str) -> String {
    let mut out = String::new();
    th(&mut out, name);
    let _ = writeln!(out, ".SH NAME\n{} \\- {}", esc(name), esc(summary));
    let _ = writeln!(out, ".SH {}\n.B {}", vc_i18n::t!("cli-man-sh-synopsis"), esc(synopsis));
    let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-description"));
    for (i, d) in desc.iter().enumerate() { if i > 0 { let _ = writeln!(out, ".PP"); } let _ = writeln!(out, "{}", esc(d)); }
    if !opts.is_empty() { let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-options")); for (o, h) in opts { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(o), esc(h)); } }
    if !env.is_empty() { let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-environment")); for (o, h) in env { let _ = writeln!(out, ".TP\n\\fB{}\\fR\n{}", esc(o), esc(h)); } }
    if !examples.is_empty() { let _ = writeln!(out, ".SH {}", vc_i18n::t!("cli-man-sh-examples")); for (t, c) in examples { let _ = writeln!(out, "{}:\n.PP\n.RS 8\n.nf\n{}\n.fi\n.RE", esc(t), esc(c)); } }
    let _ = writeln!(out, ".SH {}\n{}", vc_i18n::t!("cli-man-sh-exit"), esc(exit));
    let _ = writeln!(out, ".SH {}\nvcc(1)", vc_i18n::t!("cli-man-sh-see-also"));
    out
}

pub fn extra_pages() -> Vec<(String, String)> {
    let env: &[(&str, String)] = &[("VC_DATA", vc_i18n::t!("cli-man-env-data")), ("VC_ROMS", vc_i18n::t!("cli-man-env-roms")), ("VC_XEMU", vc_i18n::t!("cli-man-env-xemu")), ("VC_XEMU_SD", vc_i18n::t!("cli-man-env-xemu-sd")), ("VC_WIN", vc_i18n::t!("cli-man-env-win"))];
    let opts: &[(&str, String)] = &[("-h, --help", vc_i18n::t!("cli-man-opt-help")), ("-V, --version", vc_i18n::t!("cli-man-opt-version"))];
    vec![
        ("vintagecoder.1".to_string(), simple_page("vintagecoder", &vc_i18n::t!("cli-man-ide-summary"), &vc_i18n::t!("cli-man-ide-synopsis"),
            &[vc_i18n::t!("cli-man-ide-description")], opts, env,
            &[(vc_i18n::t!("cli-man-ide-example-start"), "vintagecoder"), (vc_i18n::t!("cli-man-ide-example-open-project"), "vintagecoder spiel"), (vc_i18n::t!("cli-man-ide-example-open-source"), "vintagecoder prog.asm")], &vc_i18n::t!("cli-man-ide-exit"))),
        ("vc-lsp.1".to_string(), simple_page("vc-lsp", &vc_i18n::t!("cli-man-lsp-summary"), "vc-lsp [--help] [--version]",
            &[vc_i18n::t!("cli-man-lsp-description-1"), vc_i18n::t!("cli-man-lsp-description-2")],
            opts, &[], &[(vc_i18n::t!("cli-man-opt-version-short"), "vc-lsp --version")], &vc_i18n::t!("cli-man-lsp-exit"))),
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
            let Some(dir) = args.get(1) else { crate::err_msg(vc_core::msg!("cli-man-all-missing-dir")); return ExitCode::from(2); };
            if let Err(e) = std::fs::create_dir_all(dir) { eprintln!("vcc man: {dir}: {e}"); return ExitCode::FAILURE; }
            for (name, text) in all_pages() {
                if let Err(e) = std::fs::write(std::path::Path::new(dir).join(&name), text) { eprintln!("vcc man: {name}: {e}"); return ExitCode::FAILURE; }
            }
            println!("{}", vc_i18n::t!("cli-man-all-written", count = all_pages().len(), dir = dir));
            ExitCode::SUCCESS
        }
        Some("--help") | Some("-h") => {
            println!("{}", vc_i18n::t!("cli-man-help"));
            ExitCode::SUCCESS
        }
        Some(name) => match help::find(name) {
            Some(c) => { print!("{}", command_page(c)); ExitCode::SUCCESS }
            None => help::unknown_command(name),
        },
    }
}
