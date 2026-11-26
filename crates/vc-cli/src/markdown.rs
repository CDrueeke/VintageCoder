// SPDX-License-Identifier: GPL-3.0-or-later
//! Handbuch der Kommandozeile als Markdown (`vcc help --markdown`, Datei `docs/cli.md`), aus denselben Beschreibungen wie `--help`.
use crate::help::{self, Cmd, Form, Opt};
use std::fmt::Write as _;

fn cell(t: &str) -> String { t.replace('|', "\\|") }

fn option_row(o: &Opt) -> String {
    let mut flag = format!("`{}`", o.flag);
    if !o.short.is_empty() { flag = format!("`{}`, {flag}", o.short); }
    if !o.value.is_empty() { let _ = write!(flag, " `<{}>`", o.value); }
    let mut text = o.help.clone();
    if !o.choices.is_empty() { let _ = write!(text, " {}", vc_i18n::t!("cli-help-values", values = o.choices.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", "))); }
    if o.repeat { text.push_str(&format!(" {}", vc_i18n::t!("cli-help-repeatable"))); }
    if o.required { text.push_str(&format!(" {}", vc_i18n::t!("cli-help-required"))); }
    let default = if !o.default.is_empty() { format!("`{}`", o.default) } else { o.default_text.clone() };
    format!("| {} | {} | {} |", cell(&flag), cell(&text), cell(&default))
}

fn form(out: &mut String, f: &Form, multi: bool) {
    if multi {
        let _ = writeln!(out, "### {}\n", if f.id.is_empty() { vc_i18n::t!("cli-help-form-base") } else { vc_i18n::t!("cli-md-form-named", id = &f.id) });
        if !f.summary.is_empty() { let _ = writeln!(out, "{}\n", f.summary); }
    }
    let _ = writeln!(out, "```\n{}\n```\n", f.usage);
    if !f.args.is_empty() {
        let _ = writeln!(out, "{}", vc_i18n::t!("cli-md-table-arguments"));
        for a in &f.args {
            let mut t = a.help.clone();
            if !a.required { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-optional"))); }
            if a.repeat { t.push_str(&format!(" {}", vc_i18n::t!("cli-help-repeatable"))); }
            let _ = writeln!(out, "| `{}` | {} |", a.name, cell(&t));
        }
        let _ = writeln!(out);
    }
    if !f.options.is_empty() {
        let _ = writeln!(out, "{}", vc_i18n::t!("cli-md-table-options"));
        for o in &f.options { let _ = writeln!(out, "{}", option_row(o)); }
        let _ = writeln!(out);
    }
    if !f.examples.is_empty() {
        let _ = writeln!(out, "{}\n", vc_i18n::t!("cli-md-examples"));
        for e in &f.examples { let _ = writeln!(out, "```bash\n# {}\n{}\n```\n", e.title, e.cmd); }
    }
    for n in &f.notes { let _ = writeln!(out, "{}\n", vc_i18n::t!("cli-md-note", text = n)); }
    if !f.exit.is_empty() { let _ = writeln!(out, "{}\n", vc_i18n::t!("cli-md-exit", codes = f.exit.iter().map(|(k, v)| format!("`{k}` = {v}")).collect::<Vec<_>>().join(" · "))); }
}

fn command(out: &mut String, c: &Cmd) {
    let _ = writeln!(out, "## `vcc {}`\n\n{}\n", c.name, c.summary);
    if !c.aliases.is_empty() { let _ = writeln!(out, "{}\n", vc_i18n::t!("cli-md-aliases", names = c.aliases.iter().map(|a| format!("`vcc {a}`")).collect::<Vec<_>>().join(", "))); }
    for d in &c.description { let _ = writeln!(out, "{d}\n"); }
    for f in &c.forms { form(out, f, c.forms.len() > 1); }
}

/// Das ganze Handbuch
pub fn render() -> String {
    let p = help::prog();
    let mut out = String::new();
    let _ = writeln!(out, "{}\n{}\n\n{}\n", vc_i18n::t!("cli-md-generated-comment"), vc_i18n::t!("cli-md-title"), p.intro);
    let _ = writeln!(out, "{}\n", vc_i18n::t!("cli-md-help-line"));
    for (gid, glabel) in &p.groups {
        let cmds: Vec<&Cmd> = p.cmds.iter().filter(|c| &c.group == gid).collect();
        if cmds.is_empty() { continue; }
        let _ = writeln!(out, "**{glabel}:** {}\n", cmds.iter().map(|c| format!("[`{}`](#vcc-{})", c.name, c.name)).collect::<Vec<_>>().join(", "));
    }
    for c in &p.cmds { command(&mut out, c); }
    let _ = writeln!(out, "{}", vc_i18n::t!("cli-md-env-table"));
    for (n, h) in &p.env { let _ = writeln!(out, "| `{n}` | {} |", cell(h)); }
    let _ = writeln!(out, "\n{}", vc_i18n::t!("cli-md-exit-table"));
    for (c, h) in &p.exit { let _ = writeln!(out, "| `{c}` | {} |", cell(h)); }
    out
}
