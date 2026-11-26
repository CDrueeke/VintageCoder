// SPDX-License-Identifier: GPL-3.0-or-later
//! Prüfbrücke (nur Entwicklung, Feature `testbridge`): dieselben Befehle über eine WebSocket-Verbindung auf `127.0.0.1`.
//! Die echte Oberfläche läuft dann in einem gewöhnlichen Browser (Playwright, Handbuchbilder) gegen den echten Rust-Kern.
//! Nachricht rein: `{"id":1,"cmd":"ping","args":{…}}`; raus: `{"id":1,"ok":…}` oder `{"id":1,"err":{"code":…,"args":…}}`; Ereignisse: `{"event":"…","payload":…}`.
//! Gebunden wird nur an `127.0.0.1`; ohne `VC_TESTBRIDGE=<port>` startet sie nicht. `release.py check` verlangt, dass der Auslieferungsbau sie nicht enthält.
use serde_json::{json, Value};
use std::net::TcpListener;
use std::sync::{mpsc, Mutex, OnceLock};
use tauri::AppHandle;

fn hub() -> &'static Mutex<Vec<mpsc::Sender<String>>> { static H: OnceLock<Mutex<Vec<mpsc::Sender<String>>>> = OnceLock::new(); H.get_or_init(|| Mutex::new(vec![])) }

pub fn broadcast(name: &str, payload: &Value) {
    let s = json!({ "event": name, "payload": payload }).to_string();
    if let Ok(mut h) = hub().lock() { h.retain(|tx| tx.send(s.clone()).is_ok()); }
}

/// Die Prüfbrücke schaltet Sprache, Einstellungen und Projekte um: sie startet nur mit eigenem Konfigurationsordner (`VC_CONFIG_DIR`) oder einem `HOME` unter dem Temp-Ordner,
/// nie auf der echten Konfiguration des Benutzers (ein Ablauftest, der dort `setLanguage` aufriefe, überschriebe `settings.json`)
pub fn eigene_konfiguration(config_dir: Option<&str>, home: Option<&str>, temp: &str) -> bool {
    if config_dir.is_some_and(|d| !d.is_empty()) { return true }
    let Some(h) = home.filter(|h| !h.is_empty()) else { return false };
    let norm = |p: &str| p.trim_end_matches('/').to_string();
    let (h, t) = (norm(h), norm(temp));
    h.starts_with(&format!("{t}/")) || h.starts_with("/tmp/") || h.starts_with("/private/tmp/") || h.starts_with("/var/folders/") || h.starts_with("/private/var/folders/")
}

pub fn start(app: AppHandle) {
    let Some(port) = std::env::var("VC_TESTBRIDGE").ok().and_then(|p| p.parse::<u16>().ok()) else { return };
    if !eigene_konfiguration(std::env::var("VC_CONFIG_DIR").ok().as_deref(), std::env::var("HOME").ok().as_deref(), &std::env::temp_dir().to_string_lossy()) {
        eprintln!("testbridge: abgelehnt – ohne VC_CONFIG_DIR oder ein HOME unter dem Temp-Ordner würde sie die echte Konfiguration des Benutzers ändern");
        return;
    }
    let Ok(l) = TcpListener::bind(("127.0.0.1", port)) else { eprintln!("testbridge: Port {port} belegt"); return };
    eprintln!("testbridge: ws://127.0.0.1:{port}");
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let app = app.clone();
            std::thread::spawn(move || { let _ = verbindung(app, s); });
        }
    });
}

fn verbindung(app: AppHandle, s: std::net::TcpStream) -> Result<(), tungstenite::Error> {
    let mut ws = tungstenite::accept(s.try_clone().map_err(tungstenite::Error::Io)?).map_err(|_| tungstenite::Error::ConnectionClosed)?;
    // Windows: die Lesefrist gilt nur für das Socket-Handle, auf dem sie gesetzt wurde, nicht für das mit `try_clone` verdoppelte: sonst blockiert `read`
    // ohne Ende und Ereignisse bleiben in der Warteschlange, bis der Browser etwas sendet. Darum (nach dem Handschlag) auch auf dem Handle von tungstenite setzen.
    ws.get_ref().set_read_timeout(Some(std::time::Duration::from_millis(15))).ok();
    s.set_read_timeout(Some(std::time::Duration::from_millis(15))).ok();
    let (tx, rx) = mpsc::channel::<String>();
    hub().lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(tx);
    loop {
        for m in rx.try_iter() { ws.send(tungstenite::Message::text(m))?; }
        match ws.read() {
            Ok(tungstenite::Message::Text(t)) => {
                let v: Value = serde_json::from_str(t.as_str()).unwrap_or(Value::Null);
                let id = v["id"].clone();
                let antwort = match crate::cmds::dispatch(&app, v["cmd"].as_str().unwrap_or(""), v["args"].clone()) {
                    Ok(o) => json!({ "id": id, "ok": o }),
                    Err(e) => json!({ "id": id, "err": e }),
                };
                ws.send(tungstenite::Message::text(antwort.to_string()))?;
            }
            Ok(tungstenite::Message::Close(_)) => return Ok(()),
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests_konfig {
    use super::eigene_konfiguration as k;
    #[test] fn bruecke_nur_mit_eigener_konfiguration() {
        assert!(k(Some("/x/cfg"), Some("/srv/a"), "/var/folders/x/T/"), "VC_CONFIG_DIR genügt");
        assert!(k(None, Some("/tmp/vc-home"), "/var/folders/x/T/") && k(None, Some("/var/folders/x/T/h"), "/var/folders/x/T/"));
        assert!(!k(None, Some("/srv/konto"), "/var/folders/x/T/") && !k(None, None, "/tmp") && !k(Some(""), Some("/srv/u"), "/tmp"));
    }
}
