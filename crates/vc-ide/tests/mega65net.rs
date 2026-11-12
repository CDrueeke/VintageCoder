// SPDX-License-Identifier: GPL-3.0-or-later
//! MEGA65 über Ethernet (`etherload`, `mega65_ftp`) gegen **Attrappen-Befehle**: Skripte, die ihre Argumente aufzeichnen und eine
//! vorgegebene Ausgabe liefern. Kein Gerät beteiligt – alles hier ist UNGEPRÜFT im Sinne von „nicht am MEGA65 getestet“.
#![cfg(unix)]
use std::fs;
use std::path::{Path, PathBuf};
use vc_ide::launch;
use vc_ide::mega65net::{self as net, Action, Link, Tools, UNCHECKED};
use vc_ide::settings::{Machine, Settings};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-m65net-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d); fs::create_dir_all(&d).unwrap(); d
}

/// Attrappe: hängt `<Name> <Argumente einzeln in eckigen Klammern>` an `log`, gibt `out` aus, beendet sich mit `code`
fn fake(d: &Path, name: &str, log: &Path, out: &str, code: i32, sleep: u32) -> String {
    let p = d.join(name);
    let script = format!("#!/bin/sh\nprintf '%s' '{name}' >> '{}'\nfor a in \"$@\"; do printf ' [%s]' \"$a\" >> '{}'; done\nprintf '\\n' >> '{}'\n{}echo '{out}'\nexit {code}\n",
        log.display(), log.display(), log.display(), if sleep > 0 { format!("sleep {sleep}\n") } else { String::new() });
    fs::write(&p, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    p.to_string_lossy().into_owned()
}

fn machine(el: &str, ftp: &str) -> Machine {
    Machine { id: "m1".into(), name: "MEGA65".into(), kind: "mega65".into(), target: "mega65".into(), link: "eth".into(), eth_iface: "en0".into(), etherload: el.into(), ftp_tool: ftp.into(), ..Default::default() }
}

fn lines(log: &Path) -> Vec<String> { fs::read_to_string(log).unwrap_or_default().lines().map(String::from).collect() }

#[test]
fn verbindung_je_aktion_ueberschreibt_den_standard() {
    let mut m = Machine { kind: "mega65".into(), ..Default::default() };
    assert_eq!(m.link_default(), Link::Usb, "ohne Angabe: USB wie bisher");
    for a in [Action::Start, Action::Copy, Action::Debug] { assert_eq!(m.link_for(a), Link::Usb); }
    m.link = "eth".into();
    for a in [Action::Start, Action::Copy, Action::Debug] { assert_eq!(m.link_for(a), Link::Eth); }
    m.link_debug = "usb".into();
    assert_eq!(m.link_for(Action::Debug), Link::Usb);
    assert_eq!(m.link_for(Action::Start), Link::Eth);
    m.link = "usb".into(); m.link_copy = "eth".into();
    assert_eq!((m.link_for(Action::Start), m.link_for(Action::Copy)), (Link::Usb, Link::Eth));
    m.link_start = "unsinn".into();
    assert_eq!(m.link_for(Action::Start), Link::Usb, "unbekannter Wert: Standard");
}

#[test]
fn eingebaute_aufrufzeilen_sind_lesbar_und_als_ungeprueft_gekennzeichnet() {
    let t = Tools::builtin();
    assert_eq!(t.etherload.exe, "etherload");
    assert!(t.etherload.run.contains(&"-r".to_string()));
    assert!(include_str!("../../../data/mega65/tools.json").contains("UNGEPRÜFT"));
}

#[test]
fn einstellungen_alt_und_neu() {
    // alte Datei ohne die neuen Felder lädt; leere Felder werden nicht geschrieben
    let s: Settings = serde_json::from_str(r#"{"machines":[{"id":"m1","kind":"mega65","device":"/dev/cu.usbserial-1"}]}"#).unwrap();
    assert_eq!(s.machines[0].link_default(), Link::Usb);
    let t = serde_json::to_string(&s).unwrap();
    assert!(!t.contains("eth_iface") && !t.contains("link_start"), "{t}");
    let mut m = machine("/x/etherload", "/x/mega65_ftp"); m.link_copy = "eth".into(); m.sd_dir = "vc".into();
    let s = Settings { machines: vec![m.clone()], ..Default::default() };
    let back: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back.machines[0], m);
}

#[test]
fn start_ruft_etherload_mit_schnittstelle_und_datei() {
    let d = dir("start"); let log = d.join("log");
    let el = fake(&d, "etherload", &log, "geladen", 0, 0);
    let prg = d.join("hallo.prg"); fs::write(&prg, [1, 0x20]).unwrap();
    let mut m = machine(&el, "");
    let r = net::start(&m, &prg).unwrap();
    assert_eq!(lines(&log), vec![format!("etherload [-i] [en0] [-r] [{}]", prg.display())]);
    assert!(r.contains("gestartet") && r.contains(UNCHECKED) && r.contains("geladen"), "{r}");
    // nur laden, ohne Schnittstelle
    fs::remove_file(&log).unwrap();
    m.run_after = false; m.eth_iface.clear();
    net::start(&m, &prg).unwrap();
    assert_eq!(lines(&log), vec![format!("etherload [{}]", prg.display())]);
}

#[test]
fn start_ueber_launch_nimmt_den_ethernetweg_nur_bei_ethernet() {
    let d = dir("launch"); let log = d.join("log");
    let el = fake(&d, "etherload", &log, "ok", 0, 0);
    let prg = d.join("a.prg"); fs::write(&prg, [1, 0x20, 0, 0]).unwrap();
    let mut m = machine(&el, "");
    let r = launch::run_on_machine(&m, &prg, "mega65").unwrap();
    assert!(r.contains("Ethernet"), "{r}");
    assert_eq!(lines(&log).len(), 1);
    // falsches Zielsystem
    assert!(launch::run_on_machine(&m, &prg, "c64").is_err());
    // nur das Starten über USB: kein Aufruf des Fremdwerkzeugs (der Monitor wird versucht und scheitert ohne Schnittstelle)
    m.link_start = "usb".into();
    let e = launch::run_on_machine(&m, &prg, "mega65").unwrap_err();
    assert!(e.contains("Schnittstelle"), "{e}");
    assert_eq!(lines(&log).len(), 1, "etherload darf nicht erneut laufen");
}

#[test]
fn diskette_kopieren_legt_ordner_an_und_uebertraegt() {
    let d = dir("copy"); let log = d.join("log");
    let ftp = fake(&d, "mega65_ftp", &log, "Transfer fertig", 0, 0);
    let img = d.join("demo.d81"); fs::write(&img, vec![0u8; 16]).unwrap();
    let mut m = machine("", &ftp); m.sd_dir = "/VintageCoder/".into();
    let sd = net::copy(&m, &img).unwrap();
    assert_eq!(sd, "/VintageCoder/demo.d81");
    let l = lines(&log);
    assert_eq!(l.len(), 2, "{l:?}");
    assert_eq!(l[0], "mega65_ftp [-e] [-i] [en0] [-c] [mkdir VintageCoder] [-c] [quit]");
    assert_eq!(l[1], format!("mega65_ftp [-e] [-i] [en0] [-c] [cd VintageCoder] [-c] [lcd {}] [-c] [put demo.d81] [-c] [quit]", d.display()));
    // oberste Ebene: kein mkdir, kein cd
    fs::remove_file(&log).unwrap();
    m.sd_dir.clear();
    assert_eq!(net::copy(&m, &img).unwrap(), "/demo.d81");
    assert_eq!(lines(&log), vec![format!("mega65_ftp [-e] [-i] [en0] [-c] [lcd {}] [-c] [put demo.d81] [-c] [quit]", d.display())]);
}

#[test]
fn kopieren_verweigert_usb_fehlende_datei_und_leerzeichen() {
    let d = dir("refuse"); let log = d.join("log");
    let ftp = fake(&d, "mega65_ftp", &log, "", 0, 0);
    let img = d.join("demo.d81"); fs::write(&img, [0]).unwrap();
    let mut m = machine("", &ftp);
    m.link = "usb".into();
    assert!(net::copy(&m, &img).unwrap_err().contains("Ethernet"), "USB-Kopieren ist zurückgestellt");
    m.link_copy = "eth".into();                       // nur das Kopieren über Ethernet, Standard USB
    assert!(net::copy(&m, &img).is_ok());
    assert!(net::copy(&m, &d.join("fehlt.d81")).unwrap_err().contains("nicht gefunden"));
    let sp = d.join("mein spiel.d81"); fs::write(&sp, [0]).unwrap();
    assert!(net::copy(&m, &sp).unwrap_err().contains("Leerzeichen"));
    m.kind = "ultimate64".into();
    assert!(net::copy(&m, &img).is_err());
    assert_eq!(lines(&log).len(), 1, "nur der eine gültige Aufruf");
}

#[test]
fn kopieren_einbinden_und_von_der_diskette_starten() {
    let d = dir("disk"); let log = d.join("log");
    let el = fake(&d, "etherload", &log, "ok", 0, 0);
    let ftp = fake(&d, "mega65_ftp", &log, "ok", 0, 0);
    let img = d.join("demo.d81"); fs::write(&img, [0]).unwrap();
    let prg = d.join("demo.prg"); fs::write(&prg, [1, 0x20]).unwrap();
    let m = machine(&el, &ftp);
    let r = net::copy_and_mount(&m, &img).unwrap();
    assert!(r.contains("/demo.d81") && r.contains(UNCHECKED), "{r}");
    let l = lines(&log);
    assert_eq!(l.len(), 2);
    assert!(l[0].starts_with("mega65_ftp") && l[1] == "etherload [-i] [en0] [-m] [/demo.d81]", "{l:?}");
    fs::remove_file(&log).unwrap();
    let r = launch::run_disk_on_machine(&m, &img, &prg, "mega65").unwrap();
    assert!(r.contains("kopiert, eingebunden"), "{r}");
    let l = lines(&log);
    assert_eq!(l.len(), 2);
    assert_eq!(l[1], format!("etherload [-i] [en0] [-m] [/demo.d81] [-r] [{}]", prg.display()));
    assert!(launch::machine_runs_disks(&m));
    // über USB gibt es das nicht
    let mut u = m.clone(); u.link = "usb".into(); u.link_copy.clear();
    assert!(!launch::machine_runs_disks(&u));
    assert!(launch::run_disk_on_machine(&u, &img, &prg, "mega65").is_err());
}

#[test]
fn fehler_der_werkzeuge_kommen_mit_hinweis_und_kennzeichnung() {
    let d = dir("fail"); let log = d.join("log");
    let prg = d.join("a.prg"); fs::write(&prg, [1, 0x20]).unwrap();
    let el = fake(&d, "etherload", &log, "Error: no response from device", 1, 0);
    let e = net::start(&machine(&el, ""), &prg).unwrap_err();
    assert!(e.contains("Code 1") && e.contains("no response") && e.contains("Shift+£") && e.contains(UNCHECKED), "{e}");
    let el = fake(&d, "etherload", &log, "Network is unreachable", 2, 0);
    let e = net::start(&machine(&el, ""), &prg).unwrap_err();
    assert!(e.contains("Netzwerkschnittstelle"), "{e}");
}

#[test]
fn werkzeug_fehlt_oder_haengt() {
    let d = dir("missing"); let log = d.join("log");
    let prg = d.join("a.prg"); fs::write(&prg, [1, 0x20]).unwrap();
    let e = net::start(&machine("/gibt/es/nicht/etherload", ""), &prg).unwrap_err();
    assert!(e.contains("nicht gefunden"), "{e}");
    // ohne Pfad und ohne Werkzeug im PATH
    let mut t = Tools::builtin(); t.etherload.exe = "vc-gibt-es-bestimmt-nicht".into();
    let e = net::start_with(&t, &machine("", ""), &prg).unwrap_err();
    assert!(e.contains("mega65-tools"), "{e}");
    // hängt: Zeitlimit
    let el = fake(&d, "etherload", &log, "", 0, 5);
    let mut t = Tools::builtin(); t.etherload.timeout_s = 1;
    let t0 = std::time::Instant::now();
    let e = net::start_with(&t, &machine(&el, ""), &prg).unwrap_err();
    assert!(e.contains("keine Antwort innerhalb von 1 s"), "{e}");
    assert!(t0.elapsed().as_secs() < 4);
}

#[test]
fn verbindungstest_ethernet() {
    let d = dir("test"); let log = d.join("log");
    let el = fake(&d, "etherload", &log, "", 0, 0);
    let ftp = fake(&d, "mega65_ftp", &log, "Verzeichnis", 0, 0);
    let m = machine(&el, &ftp);
    let r = net::test(&m).unwrap();
    assert!(r.contains("antwortet") && r.contains(UNCHECKED), "{r}");
    assert_eq!(lines(&log), vec!["mega65_ftp [-e] [-i] [en0] [-c] [dir] [-c] [quit]".to_string()]);
    // über die Einstellungen: die Standardverbindung entscheidet
    assert!(vc_ide::settings::test_machine(&m).unwrap().contains("Ethernet"));
    let bad = fake(&d, "mega65_ftp", &log, "timeout", 3, 0);
    assert!(net::test(&machine(&el, &bad)).is_err());
}
