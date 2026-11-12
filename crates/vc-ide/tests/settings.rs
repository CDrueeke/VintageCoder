// SPDX-License-Identifier: GPL-3.0-or-later
use std::io::{Read, Write};
use std::net::TcpListener;
use vc_ide::settings::*;

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("vc_settings_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}

#[test]
fn settings_roundtrip_keep_unknown_fields_and_use_private_file_mode() {
    let f = tmp("rt").join("sub/settings.json");
    assert_eq!(Settings::load_from(&f).unwrap(), Settings::default(), "fehlende Datei = Standardwerte");
    let mut s = Settings::default();
    s.emulators.vice.x64sc = "~/vice/x64sc".into(); s.emulators.vice.args = "-VICIIfilter 0".into();
    s.emulators.xemu.rom_dir = "/roms".into();
    s.machines.push(Machine { id: "u64".into(), name: "Ultimate 64".into(), kind: "ultimate64".into(), host: "192.168.1.50".into(), password: "geheim".into(), ..Default::default() });
    s.extra.insert("zukunft".into(), serde_json::json!({ "a": 1 }));
    s.save_to(&f).unwrap();
    let back = Settings::load_from(&f).unwrap();
    assert_eq!(back, s);
    assert_eq!(back.extra["zukunft"]["a"], 1);
    assert_eq!(back.machine("u64").unwrap().password, "geheim");
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(std::fs::metadata(&f).unwrap().permissions().mode() & 0o777, 0o600, "Passwörter nur für den Benutzer lesbar"); }
    // eine ältere Datei ohne Abschnitte bleibt lesbar
    std::fs::write(&f, r#"{"machines":[{"id":"x","kind":"mega65"}]}"#).unwrap();
    let old = Settings::load_from(&f).unwrap();
    assert_eq!(old.machines[0].device, ""); assert_eq!(old.emulators, Emulators::default());
    std::fs::write(&f, "kein json").unwrap();
    assert!(Settings::load_from(&f).is_err());
}

#[test]
#[cfg(not(windows))]
fn argument_lines_are_split_like_a_shell() {
    assert_eq!(split_args("-warp -VICIIfilter 0"), vec!["-warp", "-VICIIfilter", "0"]);
    assert_eq!(split_args(r#"-kernal "/Pfad mit Leerzeichen/k.rom" -x"#), vec!["-kernal", "/Pfad mit Leerzeichen/k.rom", "-x"]);
    assert_eq!(split_args("a 'b c' d\\ e \"\""), vec!["a", "b c", "d e", ""]);
    assert!(split_args("   ").is_empty());
}

/// Unter Windows ist `\` ein gewöhnliches Zeichen und maskiert nur vor Anführungszeichen
#[test]
#[cfg(windows)]
fn argument_lines_are_split_like_a_shell() {
    assert_eq!(split_args("-warp -VICIIfilter 0"), vec!["-warp", "-VICIIfilter", "0"]);
    assert_eq!(split_args("a 'b c' d\\ e \"\""), vec!["a", "b c", "d\\", "e", ""]);
    assert!(split_args("   ").is_empty());
}

/// Die Windows-Regel ist auf jedem System prüfbar (`split_args_os`); Messwerte aus der Windows-Sitzung
#[test]
fn windows_argument_lines_keep_backslashes() {
    let w = |s: &str| split_args_os(s, true);
    assert_eq!(w(r"-sdimg C:\Projekte\x.img"), vec!["-sdimg", r"C:\Projekte\x.img"]);
    assert_eq!(w(r#""C:\Program Files\x y\a.img" -x"#), vec![r"C:\Program Files\x y\a.img", "-x"]);
    assert_eq!(w(r"\\server\share\d.d64"), vec![r"\\server\share\d.d64"]);
    assert_eq!(w(r#"a\"b c\" d"#), vec!["a\"b", "c\"", "d"], "vor einem Anführungszeichen maskiert der Backslash");
    assert_eq!(w("d\\ e"), vec!["d\\", "e"]);
    // andere Systeme: unverändert
    assert_eq!(split_args_os("d\\ e", false), vec!["d e"]);
    assert_eq!(split_args_os(r"C:\x", false), vec!["C:x"]);
}

#[test]
fn program_check_reports_missing_and_non_executable_files() {
    assert!(check_program("/gibt/es/nicht").unwrap_err().contains("nicht gefunden"));
    let d = tmp("prog"); let f = d.join("x64sc"); std::fs::write(&f, "#!/bin/sh\n").unwrap();
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap(); assert!(check_program(f.to_str().unwrap()).unwrap_err().contains("nicht ausführbar")); std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o755)).unwrap(); }
    assert!(check_program(f.to_str().unwrap()).unwrap().starts_with("gefunden"));
    assert!(check_program(d.to_str().unwrap()).unwrap_err().contains("keine Datei"));   // ein Ordner (auch unter Windows vorhanden)
}

/// Kleiner HTTP-Server, der die Anfrage meldet und je nach Passwort antwortet
fn fake_ultimate(expect_pw: &'static str) -> (String, std::sync::mpsc::Receiver<String>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let addr = l.local_addr().unwrap().to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for s in l.incoming().take(4) {
            let mut s = s.unwrap(); let mut buf = [0u8; 2048]; let n = s.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).into_owned(); let ok = req.contains(&format!("X-Password: {expect_pw}"));
            let _ = tx.send(req.clone());
            let body = if req.starts_with("GET /v1/info") { r#"{"product":"Ultimate 64-II","firmware_version":"3.14d","hostname":"U64","errors":[]}"# } else { r#"{"version":"0.1","errors":[]}"# };
            let resp = if ok { format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()) } else { "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string() };
            let _ = s.write_all(resp.as_bytes());
        }
    });
    (addr, rx)
}

#[test]
fn ultimate_connection_test_sends_the_password_and_reports_wrong_ones() {
    let (addr, rx) = fake_ultimate("geheim");
    let ok = Machine { id: "u".into(), kind: "ultimate64".into(), host: addr.clone(), password: "geheim".into(), ..Default::default() };
    let r = test_machine(&ok).unwrap();
    assert_eq!(r, "verbunden: Ultimate 64-II, Firmware 3.14d, Host U64");
    let req = rx.recv().unwrap();
    assert!(req.starts_with("GET /v1/version"), "{req}"); assert!(req.contains("X-Password: geheim"));
    assert!(rx.recv().unwrap().starts_with("GET /v1/info"));
    let bad = Machine { password: "falsch".into(), ..ok.clone() };
    assert!(test_machine(&bad).unwrap_err().contains("403"), "falsches Passwort wird gemeldet");
    assert!(test_machine(&Machine { host: String::new(), ..ok.clone() }).unwrap_err().contains("Adresse"));
    assert!(test_machine(&Machine { kind: "toaster".into(), ..ok }).unwrap_err().contains("unbekannter Typ"));
}

#[test]
fn mega65_connection_spec_comes_from_device_and_baud() {
    let mut m = Machine { id: "m".into(), kind: "mega65".into(), device: "/dev/cu.usbserial-X".into(), ..Default::default() };
    assert_eq!(mega65_spec(&m).unwrap(), "serial:/dev/cu.usbserial-X");
    m.baud = "1000000".into(); assert_eq!(mega65_spec(&m).unwrap(), "serial:/dev/cu.usbserial-X@1000000");
    m.device.clear(); assert!(mega65_spec(&m).is_err());
    assert!(test_machine(&Machine { device: "/dev/gibt-es-nicht".into(), ..m }).is_err(), "nicht vorhandene Schnittstelle liefert einen Fehler");
}
