// SPDX-License-Identifier: GPL-3.0-or-later
//! Vervollständigung für `vcc` (PowerShell, bash, zsh, fish) aus den Beschreibungen in `data/cli`: `vcc completion <shell>`.
use crate::help::{self, Cmd, Opt};
use std::fmt::Write as _;
use std::process::ExitCode;

fn q(t: &str) -> String { t.replace(['\'', '"', '`', '$', '\\'], " ") }

/// Alle Optionen eines Befehls über alle Formen (gleiche Schreibweise nur einmal; Werte werden zusammengeführt)
fn opts_of(c: &Cmd) -> Vec<Opt> {
    let mut v: Vec<Opt> = vec![];
    for f in &c.forms { for o in &f.options {
        if let Some(e) = v.iter_mut().find(|x| x.flag == o.flag) { for ch in &o.choices { if !e.choices.contains(ch) { e.choices.push(ch.clone()); } } }
        else { v.push(o.clone()); }
    } }
    v
}

/// Formen mit eigener Kennung gelten als zweites Wort (`vcc basic ast`) oder als Schalter (`--tokenize`): beides anbieten
fn form_words(c: &Cmd) -> Vec<String> { c.forms.iter().filter(|f| !f.id.is_empty()).map(|f| f.id.clone()).collect() }

fn cmd_names() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = vec![];
    for c in &help::prog().cmds { v.push((c.name.clone(), c.summary.clone())); for a in &c.aliases { v.push((a.clone(), c.summary.clone())); } }
    v.sort();
    v
}

fn bash() -> String {
    let mut o = String::from("# Vervollständigung für vcc (bash). Einmalig: eval \"$(vcc completion bash)\"\n_vcc() {\n  local cur prev cmd opts\n  COMPREPLY=()\n  cur=\"${COMP_WORDS[COMP_CWORD]}\"\n  prev=\"${COMP_WORDS[COMP_CWORD-1]}\"\n  cmd=\"${COMP_WORDS[1]}\"\n");
    let names: Vec<String> = cmd_names().into_iter().map(|c| c.0).chain(["help", "man", "completion", "version"].map(String::from)).collect();
    let _ = writeln!(o, "  if [ \"$COMP_CWORD\" -eq 1 ]; then COMPREPLY=( $(compgen -W \"{}\" -- \"$cur\") ); return; fi", names.join(" "));
    o.push_str("  case \"$cmd\" in\n");
    for c in &help::prog().cmds {
        let mut pats = vec![c.name.clone()]; pats.extend(c.aliases.iter().cloned());
        let _ = writeln!(o, "    {})", pats.join("|"));
        let mut words: Vec<String> = opts_of(c).iter().flat_map(|x| [x.flag.clone(), x.short.clone()]).filter(|x| !x.is_empty()).collect();
        words.extend(form_words(c));
        let _ = writeln!(o, "      opts=\"{}\"", words.join(" "));
        let with_choices: Vec<Opt> = opts_of(c).into_iter().filter(|x| !x.choices.is_empty()).collect();
        if !with_choices.is_empty() {
            o.push_str("      case \"$prev\" in\n");
            for x in with_choices {
                let flags: Vec<String> = [x.flag.clone(), x.short.clone()].into_iter().filter(|f| !f.is_empty()).collect();
                let _ = writeln!(o, "        {}) COMPREPLY=( $(compgen -W \"{}\" -- \"$cur\") ); return ;;", flags.join("|"), x.choices.join(" "));
            }
            o.push_str("      esac ;;\n");
        } else { o.push_str("      ;;\n"); }
    }
    o.push_str("    help) opts=\"");
    o.push_str(&help::prog().cmds.iter().map(|c| c.name.clone()).chain(["env", "exit-codes", "--all"].map(String::from)).collect::<Vec<_>>().join(" "));
    o.push_str("\" ;;\n    completion) opts=\"powershell bash zsh fish\" ;;\n  esac\n  COMPREPLY=( $(compgen -W \"$opts\" -- \"$cur\") )\n  [ ${#COMPREPLY[@]} -eq 0 ] && COMPREPLY=( $(compgen -f -- \"$cur\") )\n}\ncomplete -o default -F _vcc vcc\n");
    o
}

fn powershell() -> String {
    let mut o = String::from("# Vervollständigung für vcc (PowerShell). Einmalig: vcc completion powershell | Out-String | Invoke-Expression\n$vccCommands = @{\n");
    for (n, s) in cmd_names() { let _ = writeln!(o, "  '{n}' = '{}'", q(&s)); }
    o.push_str("  'help' = 'Hilfe zu einem Befehl'\n  'man' = 'man-Seite ausgeben'\n  'completion' = 'Vervollstaendigung ausgeben'\n  'version' = 'Version anzeigen'\n}\n$vccOptions = @{\n");
    for c in &help::prog().cmds {
        let mut words: Vec<String> = opts_of(c).iter().flat_map(|x| [x.flag.clone(), x.short.clone()]).filter(|x| !x.is_empty()).collect();
        words.extend(form_words(c));
        let line = format!("@({})", words.iter().map(|w| format!("'{w}'")).collect::<Vec<_>>().join(","));
        let _ = writeln!(o, "  '{}' = {line}", c.name);
        for a in &c.aliases { let _ = writeln!(o, "  '{a}' = {line}"); }
    }
    o.push_str("}\n$vccChoices = @{\n");
    for c in &help::prog().cmds { for x in opts_of(c).into_iter().filter(|x| !x.choices.is_empty()) {
        let line = format!("@({})", x.choices.iter().map(|w| format!("'{w}'")).collect::<Vec<_>>().join(","));
        for f in [x.flag.clone(), x.short.clone()].into_iter().filter(|f| !f.is_empty()) { let _ = writeln!(o, "  '{}|{f}' = {line}", c.name); }
    } }
    o.push_str("}\nRegister-ArgumentCompleter -Native -CommandName vcc -ScriptBlock {\n  param($wordToComplete, $commandAst, $cursorPosition)\n  $words = @($commandAst.CommandElements | ForEach-Object { $_.ToString() })\n  $items = @()\n  if ($words.Count -le 1 -or ($words.Count -eq 2 -and $wordToComplete -ne '')) { $items = $vccCommands.Keys | Sort-Object }\n  else {\n    $cmd = $words[1]\n    $prev = if ($wordToComplete -eq '') { $words[-1] } else { $words[-2] }\n    if ($vccChoices.ContainsKey(\"$cmd|$prev\")) { $items = $vccChoices[\"$cmd|$prev\"] }\n    elseif ($vccOptions.ContainsKey($cmd)) { $items = $vccOptions[$cmd] }\n    elseif ($cmd -eq 'help') { $items = @($vccOptions.Keys) + @('env','exit-codes','--all') }\n    elseif ($cmd -eq 'completion') { $items = @('powershell','bash','zsh','fish') }\n  }\n  $items | Where-Object { $_ -like \"$wordToComplete*\" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }\n}\n");
    o
}

fn zsh() -> String {
    let mut o = String::from("#compdef vcc\n# Vervollständigung für vcc (zsh). Datei als _vcc in einen Ordner des fpath legen oder: eval \"$(vcc completion zsh)\"\n_vcc() {\n  local -a cmds\n  cmds=(\n");
    for (n, s) in cmd_names() { let _ = writeln!(o, "    '{n}:{}'", q(&s).replace(':', " ")); }
    o.push_str("    'help:Hilfe zu einem Befehl'\n    'man:man-Seite ausgeben'\n    'completion:Vervollstaendigung ausgeben'\n    'version:Version anzeigen'\n  )\n  if (( CURRENT == 2 )); then _describe 'befehl' cmds; return; fi\n  case $words[2] in\n");
    for c in &help::prog().cmds {
        let mut pats = vec![c.name.clone()]; pats.extend(c.aliases.iter().cloned());
        let _ = writeln!(o, "    {})", pats.join("|"));
        let mut args: Vec<String> = vec![];
        for x in opts_of(c) {
            let desc = q(&x.help);
            let action = if !x.choices.is_empty() { format!(":{}:({})", if x.value.is_empty() { "wert" } else { &x.value }, x.choices.join(" ")) } else if !x.value.is_empty() { format!(":{}:_files", x.value) } else { String::new() };
            for f in [x.flag.clone(), x.short.clone()].into_iter().filter(|f| !f.is_empty()) { args.push(format!("'{f}[{desc}]{action}'")); }
        }
        args.push("'*:datei:_files'".into());
        let _ = writeln!(o, "      _arguments {} ;;", args.join(" "));
    }
    o.push_str("    help) compadd -- ");
    o.push_str(&help::prog().cmds.iter().map(|c| c.name.clone()).chain(["env", "exit-codes", "--all"].map(String::from)).collect::<Vec<_>>().join(" "));
    o.push_str(" ;;\n    completion) compadd -- powershell bash zsh fish ;;\n  esac\n}\n_vcc \"$@\"\n");
    o
}

fn fish() -> String {
    let mut o = String::from("# Vervollständigung für vcc (fish). Einmalig: vcc completion fish | source\ncomplete -c vcc -f\n");
    for (n, s) in cmd_names() { let _ = writeln!(o, "complete -c vcc -n '__fish_use_subcommand' -a '{n}' -d '{}'", q(&s)); }
    for (n, d) in [("help", "Hilfe zu einem Befehl"), ("man", "man-Seite ausgeben"), ("completion", "Vervollstaendigung ausgeben"), ("version", "Version anzeigen")] { let _ = writeln!(o, "complete -c vcc -n '__fish_use_subcommand' -a '{n}' -d '{d}'"); }
    for c in &help::prog().cmds {
        let mut names = vec![c.name.clone()]; names.extend(c.aliases.iter().cloned());
        let cond = format!("__fish_seen_subcommand_from {}", names.join(" "));
        for x in opts_of(c) {
            let mut l = format!("complete -c vcc -n '{cond}'");
            if let Some(s) = x.flag.strip_prefix("--") { let _ = write!(l, " -l {s}"); } else if let Some(s) = x.flag.strip_prefix('-') { let _ = write!(l, " -s {s}"); }
            if let Some(s) = x.short.strip_prefix('-') { if !x.short.starts_with("--") { let _ = write!(l, " -s {s}"); } }
            if !x.choices.is_empty() { let _ = write!(l, " -x -a '{}'", x.choices.join(" ")); } else if !x.value.is_empty() { l.push_str(" -r"); }
            let _ = writeln!(o, "{l} -d '{}'", q(&x.help));
        }
        for w in form_words(c) { let _ = writeln!(o, "complete -c vcc -n '{cond}' -a '{w}' -d 'Form'"); }
    }
    let _ = writeln!(o, "complete -c vcc -n '__fish_seen_subcommand_from completion' -a 'powershell bash zsh fish'");
    o
}

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("powershell") | Some("pwsh") => { print!("{}", powershell()); ExitCode::SUCCESS }
        Some("bash") => { print!("{}", bash()); ExitCode::SUCCESS }
        Some("zsh") => { print!("{}", zsh()); ExitCode::SUCCESS }
        Some("fish") => { print!("{}", fish()); ExitCode::SUCCESS }
        Some("--help") | Some("-h") => {
            println!("vcc completion - Vervollständigung (Tab) für die Shell ausgeben\n\nAufruf:  vcc completion <shell>\n         shell: powershell | bash | zsh | fish\n\nBeispiele:\n  vcc completion powershell | Out-String | Invoke-Expression\n  eval \"$(vcc completion bash)\"\n  vcc completion fish | source");
            ExitCode::SUCCESS
        }
        Some(other) => { eprintln!("vcc completion: unbekannte Shell „{other}“\nErlaubt: powershell, bash, zsh, fish"); ExitCode::from(2) }
        None => { eprintln!("vcc completion: Shell fehlt\nAufruf:  vcc completion <powershell|bash|zsh|fish>"); ExitCode::from(2) }
    }
}
