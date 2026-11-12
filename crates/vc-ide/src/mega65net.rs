// SPDX-License-Identifier: GPL-3.0-or-later
//! Programme und Disketten über **Ethernet** auf einen echten MEGA65 bringen (`etherload`, `mega65_ftp`), Konzept
//! `Konzept-MEGA65-Netzwerk.md`, Stufen 1b und 2.
//!
//! **UNGEPRÜFT:** Der ganze Ethernet-Weg ist nur gegen Attrappen-Befehle (Skripte, die Argumente prüfen) getestet, nicht an einem
//! MEGA65. Die Aufrufzeilen stehen in `data/mega65/tools.json` (eingebaut; ein Ersatz liegt als `mega65-tools.json` im
//! Konfigurationsordner) und sind aus Wiki und GitHub übernommen. Beim ersten Test am Gerät gegen `--help` der installierten Werkzeuge prüfen.
//!
//! Die Verbindung wird je Maschine als Standard (`Machine::link`) festgelegt und je Aktion (Starten, Kopieren, Debuggen) überschrieben.
//! USB (eigener Monitor, siehe `vc_emu::Mega65`) bleibt der Weg für Starten und Debuggen; Dateien auf die SD-Karte kopiert nur Ethernet
//! (eine eigene USB-Übertragung ist zurückgestellt).
use crate::settings::{expand_home, Machine};
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Kennzeichnung für Oberfläche, Hilfe und Meldungen
pub const UNCHECKED: &str = "ungeprüft: noch nicht an einem MEGA65 getestet";

/// Verbindung zum Gerät
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link { Usb, Eth }

/// Was mit dem Gerät geschehen soll
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action { Start, Copy, Debug }

impl Link {
    pub fn parse(s: &str) -> Option<Link> { match s.trim() { "usb" => Some(Link::Usb), "eth" => Some(Link::Eth), _ => None } }
    pub fn id(self) -> &'static str { match self { Link::Usb => "usb", Link::Eth => "eth" } }
    pub fn label(self) -> &'static str { match self { Link::Usb => "USB", Link::Eth => "Ethernet" } }
}

impl Machine {
    /// Standardverbindung einer MEGA65 (leer oder unbekannt = USB)
    pub fn link_default(&self) -> Link { Link::parse(&self.link).unwrap_or(Link::Usb) }
    /// Verbindung für eine Aktion: eigene Wahl, sonst die Standardverbindung
    pub fn link_for(&self, a: Action) -> Link {
        let own = match a { Action::Start => &self.link_start, Action::Copy => &self.link_copy, Action::Debug => &self.link_debug };
        Link::parse(own).unwrap_or_else(|| self.link_default())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Etherload { pub exe: String, pub iface_args: Vec<String>, pub run: Vec<String>, pub load: Vec<String>, pub mount: Vec<String>, pub mount_run: Vec<String>, pub timeout_s: u64 }
#[derive(Clone, Debug, Deserialize)]
pub struct Ftp { pub exe: String, pub iface_args: Vec<String>, pub base: Vec<String>, pub mkdir: String, pub put: Vec<String>, pub put_root: Vec<String>, pub dir: Vec<String>, pub timeout_s: u64, pub test_timeout_s: u64 }
#[derive(Clone, Debug, Deserialize)]
pub struct Hints { pub nicht_erreichbar: String, pub schnittstelle: String }

/// Aufrufzeilen der Fremdwerkzeuge (Datendatei `data/mega65/tools.json`)
#[derive(Clone, Debug, Deserialize)]
pub struct Tools { pub etherload: Etherload, pub mega65_ftp: Ftp, pub hinweise: Hints }

const BUILTIN: &str = include_str!("../../../data/mega65/tools.json");

impl Tools {
    pub fn builtin() -> Tools { serde_json::from_str(BUILTIN).expect("data/mega65/tools.json") }
    /// Ersatzdatei `mega65-tools.json` im Konfigurationsordner, sonst die eingebauten Vorlagen
    pub fn load() -> Tools {
        vc_core::userdirs::config_dir().map(|d| d.join("mega65-tools.json")).and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(Tools::builtin)
    }
}

/// Vorlage ausfüllen: `{iface_args}` wird durch die Schnittstellenargumente ersetzt (oder entfällt), `{name}` usw. durch `vars`
pub fn expand(tpl: &[String], vars: &[(&str, &str)], iface_args: &[String]) -> Vec<String> {
    let mut out = vec![];
    for t in tpl {
        if t == "{iface_args}" { out.extend(iface_args.iter().cloned()); continue; }
        out.push(fill(t, vars));
    }
    out
}

fn fill(t: &str, vars: &[(&str, &str)]) -> String {
    let mut s = t.to_string();
    for (k, v) in vars { s = s.replace(&format!("{{{k}}}"), v); }
    s
}

fn iface_args(tpl: &[String], m: &Machine) -> Vec<String> {
    let i = m.eth_iface.trim();
    if i.is_empty() { vec![] } else { tpl.iter().map(|a| fill(a, &[("iface", i)])).collect() }
}

/// Programm der Werkzeuge: eingetragener Pfad, sonst Suche (PATH und übliche Ordner)
fn find_tool(configured: &str, exe: &str) -> Result<String, String> {
    let c = configured.trim();
    if !c.is_empty() {
        let p = expand_home(c);
        return if Path::new(&p).is_file() { Ok(p) } else { Err(format!("{p}: nicht gefunden (Pfad unter Einstellungen → Hardware prüfen)")) };
    }
    vc_emu::find_program(exe, &["/opt/homebrew/bin", "/usr/local/bin"])
        .ok_or_else(|| format!("{exe} nicht gefunden – Pfad unter Einstellungen → Hardware eintragen (Werkzeug aus dem Paket mega65-tools, Quelle: github.com/MEGA65/mega65-tools; die IDE liefert es nicht mit)"))
}

struct Out { code: Option<i32>, text: String }

/// Startet ein Werkzeug und sammelt Ausgabe (stdout und stderr); nach `limit` wird es beendet
fn run_tool(exe: &str, args: &[String], limit: Duration) -> Result<Out, String> {
    let mut child = Command::new(exe).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("{exe}: {e}"))?;
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let n = [child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>), child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>)].into_iter().flatten()
        .map(|mut r| { let tx = tx.clone(); std::thread::spawn(move || { let mut b = vec![]; let _ = r.read_to_end(&mut b); let _ = tx.send(String::from_utf8_lossy(&b).into_owned()); }); }).count();
    drop(tx);
    let t0 = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().map_err(|e| e.to_string())? { break Some(s); }
        if t0.elapsed() > limit { let _ = child.kill(); let _ = child.wait(); break None; }
        std::thread::sleep(Duration::from_millis(20));
    };
    // Nach einem Abbruch kann ein Kindprozess die Pipes noch offen halten: nur kurz auf die Ausgabe warten
    let wait = if status.is_some() { Duration::from_secs(5) } else { Duration::from_millis(300) };
    let text = (0..n).filter_map(|_| rx.recv_timeout(wait).ok()).collect::<Vec<_>>().join("\n");
    match status { Some(s) => Ok(Out { code: s.code(), text }), None => Err(format!("keine Antwort innerhalb von {} s; Ausgabe: {}", limit.as_secs(), tail(&text))) }
}

fn tail(t: &str) -> String {
    let l: Vec<&str> = t.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let n = l.len().saturating_sub(3);
    l[n..].join(" / ")
}

/// Fehlerhinweis passend zur Ausgabe
fn hint(tools: &Tools, text: &str) -> String {
    let t = text.to_lowercase();
    if t.contains("no such device") || t.contains("interface") || t.contains("unreachable") || t.contains("ipv6") { tools.hinweise.schnittstelle.clone() } else { tools.hinweise.nicht_erreichbar.clone() }
}

fn check(tools: &Tools, what: &str, r: Result<Out, String>) -> Result<String, String> {
    match r {
        Ok(o) if o.code == Some(0) => Ok(tail(&o.text)),
        Ok(o) => Err(format!("{what} endete mit {} ({}) [{UNCHECKED}]\n{}", o.code.map_or("Signal".to_string(), |c| format!("Code {c}")), tail(&o.text), hint(tools, &o.text))),
        Err(e) => Err(format!("{what}: {e}\n{} [{UNCHECKED}]", tools.hinweise.nicht_erreichbar)),
    }
}

fn is_mega65(m: &Machine) -> Result<(), String> { if m.kind == "mega65" { Ok(()) } else { Err("Ethernet-Übertragung gibt es nur für den MEGA65".into()) } }

fn plain_name(p: &Path) -> Result<String, String> {
    let n = p.file_name().map(|n| n.to_string_lossy().into_owned()).ok_or_else(|| format!("{}: kein Dateiname", p.display()))?;
    if n.contains(char::is_whitespace) || n.contains('"') { return Err(format!("{n}: Dateiname mit Leerzeichen oder Anführungszeichen lässt sich nicht an mega65_ftp übergeben – bitte umbenennen")); }
    Ok(n)
}

/// Ordner auf der SD-Karte ohne führende und folgende Schrägstriche (leer = oberste Ebene)
pub fn sd_folder(m: &Machine) -> String { m.sd_dir.trim().trim_matches('/').to_string() }

/// Pfad einer kopierten Datei auf der SD-Karte
pub fn sd_path(m: &Machine, name: &str) -> String { let d = sd_folder(m); if d.is_empty() { format!("/{name}") } else { format!("/{d}/{name}") } }

fn who(m: &Machine) -> &str { m.display_name() }

/// Programm per `etherload` laden (und starten, wenn `run_after`). UNGEPRÜFT.
pub fn start(m: &Machine, prg: &Path) -> Result<String, String> { start_with(&Tools::load(), m, prg) }
pub fn start_with(tools: &Tools, m: &Machine, prg: &Path) -> Result<String, String> {
    is_mega65(m)?;
    let exe = find_tool(&m.etherload, &tools.etherload.exe)?;
    let tpl = if m.run_after { &tools.etherload.run } else { &tools.etherload.load };
    let args = expand(tpl, &[("file", &prg.to_string_lossy())], &iface_args(&tools.etherload.iface_args, m));
    let out = check(tools, "etherload", run_tool(&exe, &args, Duration::from_secs(tools.etherload.timeout_s)))?;
    Ok(format!("{}: Programm über Ethernet {} ({UNCHECKED})\n{exe} {}{}", who(m), if m.run_after { "gestartet" } else { "geladen" }, args.join(" "), if out.is_empty() { String::new() } else { format!("\n{out}") }))
}

/// Diskimage per `mega65_ftp` auf die SD-Karte kopieren; liefert den Pfad auf der Karte. UNGEPRÜFT.
pub fn copy(m: &Machine, image: &Path) -> Result<String, String> { copy_with(&Tools::load(), m, image) }
pub fn copy_with(tools: &Tools, m: &Machine, image: &Path) -> Result<String, String> {
    is_mega65(m)?;
    if m.link_for(Action::Copy) != Link::Eth {
        return Err("Kopieren auf die SD-Karte geht nur über Ethernet (eine eigene USB-Übertragung ist zurückgestellt). Unter Einstellungen → Hardware die Verbindung „Kopieren“ auf Ethernet stellen.".into());
    }
    if !image.is_file() { return Err(format!("{}: Datei nicht gefunden (erst bauen)", image.display())); }
    let name = plain_name(image)?;
    let local = image.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| ".".into());
    let exe = find_tool(&m.ftp_tool, &tools.mega65_ftp.exe)?;
    let ia = iface_args(&tools.mega65_ftp.iface_args, m);
    let dir = sd_folder(m);
    if dir.contains(char::is_whitespace) { return Err(format!("SD-Ordner „{dir}“ enthält Leerzeichen – bitte ändern")); }
    let call = |cmds: &[String], vars: &[(&str, &str)]| -> Vec<String> {
        let mut a = expand(&tools.mega65_ftp.base, vars, &ia);
        for c in cmds { a.push("-c".into()); a.push(fill(c, vars)); }
        a
    };
    let vars = [("dir", dir.as_str()), ("localdir", local.as_str()), ("name", name.as_str())];
    let limit = Duration::from_secs(tools.mega65_ftp.timeout_s);
    // Ordner anlegen (gibt es ihn schon, meldet das Werkzeug womöglich einen Fehler: ignorieren)
    if !dir.is_empty() {
        let _ = run_tool(&exe, &call(&[tools.mega65_ftp.mkdir.clone(), "quit".into()], &vars), Duration::from_secs(tools.mega65_ftp.test_timeout_s));
    }
    let put = if dir.is_empty() { &tools.mega65_ftp.put_root } else { &tools.mega65_ftp.put };
    let args = call(put, &vars);
    check(tools, "mega65_ftp", run_tool(&exe, &args, limit))?;
    Ok(sd_path(m, &name))
}

/// Diskimage von der SD-Karte einbinden (`etherload -m`). UNGEPRÜFT.
pub fn mount(m: &Machine, sd_image: &str) -> Result<String, String> { mount_with(&Tools::load(), m, sd_image) }
pub fn mount_with(tools: &Tools, m: &Machine, sd_image: &str) -> Result<String, String> {
    is_mega65(m)?;
    let exe = find_tool(&m.etherload, &tools.etherload.exe)?;
    let args = expand(&tools.etherload.mount, &[("image", sd_image)], &iface_args(&tools.etherload.iface_args, m));
    check(tools, "etherload", run_tool(&exe, &args, Duration::from_secs(tools.etherload.timeout_s)))?;
    Ok(format!("{}: {sd_image} eingebunden ({UNCHECKED})", who(m)))
}

/// Diskette kopieren und einbinden. UNGEPRÜFT.
pub fn copy_and_mount(m: &Machine, image: &Path) -> Result<String, String> { copy_and_mount_with(&Tools::load(), m, image) }
pub fn copy_and_mount_with(tools: &Tools, m: &Machine, image: &Path) -> Result<String, String> {
    let sd = copy_with(tools, m, image)?;
    let r = mount_with(tools, m, &sd)?;
    Ok(format!("{}: {} kopiert nach {sd}; {r}", who(m), image.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()))
}

/// Diskette kopieren, einbinden und das Programm `prg` starten. UNGEPRÜFT.
pub fn start_disk(m: &Machine, image: &Path, prg: &Path) -> Result<String, String> { start_disk_with(&Tools::load(), m, image, prg) }
pub fn start_disk_with(tools: &Tools, m: &Machine, image: &Path, prg: &Path) -> Result<String, String> {
    let sd = copy_with(tools, m, image)?;
    let exe = find_tool(&m.etherload, &tools.etherload.exe)?;
    let args = expand(&tools.etherload.mount_run, &[("image", &sd), ("file", &prg.to_string_lossy())], &iface_args(&tools.etherload.iface_args, m));
    check(tools, "etherload", run_tool(&exe, &args, Duration::from_secs(tools.etherload.timeout_s)))?;
    Ok(format!("{}: Diskette nach {sd} kopiert, eingebunden und Programm gestartet ({UNCHECKED})", who(m)))
}

/// Verbindungstest über Ethernet: beide Werkzeuge vorhanden, `mega65_ftp dir` antwortet. UNGEPRÜFT.
pub fn test(m: &Machine) -> Result<String, String> { test_with(&Tools::load(), m) }
pub fn test_with(tools: &Tools, m: &Machine) -> Result<String, String> {
    is_mega65(m)?;
    let el = find_tool(&m.etherload, &tools.etherload.exe)?;
    let ftp = find_tool(&m.ftp_tool, &tools.mega65_ftp.exe)?;
    let mut args = expand(&tools.mega65_ftp.base, &[], &iface_args(&tools.mega65_ftp.iface_args, m));
    for c in &tools.mega65_ftp.dir { args.push("-c".into()); args.push(c.clone()); }
    check(tools, "mega65_ftp", run_tool(&ftp, &args, Duration::from_secs(tools.mega65_ftp.test_timeout_s)))?;
    Ok(format!("Ethernet: Werkzeuge gefunden ({el}, {ftp}), MEGA65 antwortet ({UNCHECKED})"))
}
