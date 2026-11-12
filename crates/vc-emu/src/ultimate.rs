// SPDX-License-Identifier: GPL-3.0-or-later
//! Adapter für Ultimate 64 / Ultimate II+ über die REST-API (Firmware ≥ 3.15, Port 80, `X-Password`).
//! Die Hardware bietet Speicherzugriff (DMA, auch bei laufendem Rechner), Anhalten/Fortsetzen, Reset und das Starten von PRG-Dateien –
//! aber keine CPU-Register, keinen Einzelschritt und keine Haltepunkte.
use crate::streams::StreamKind;
use crate::{Backend, Caps, Regs, RunState};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug)]
pub struct Ultimate64 { host: String, password: Option<String>, paused: bool, info: Option<Option<Info>> }

/// Antwort von `GET /v1/info`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Info { pub product: String, pub firmware_version: String, pub hostname: String, pub core_version: String }

/// Was das Gerät laut Produkt kann. Bei unbekanntem Produkt ist alles `false` (nichts annehmen).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UltimateCaps {
    /// Datenströme (`streams/…:start`)
    pub streams: bool,
    /// `machine:input` (Tastatureingabe)
    pub machine_input: bool,
    /// `machine:debugreg` (Debug-Register `$D7FF`)
    pub debugreg: bool,
    /// `machine:poweroff`
    pub poweroff: bool,
}

impl Info {
    /// Fähigkeiten aus dem Produktnamen: nur Geräte der Familie „Ultimate 64“ (auch „Ultimate 64-II“) haben diese Funktionen,
    /// die Ultimate II+ nicht. Ein unbekanntes oder leeres Produkt ergibt keine Fähigkeit.
    pub fn caps(&self) -> UltimateCaps {
        let p: String = self.product.chars().filter(|c| !c.is_whitespace() && *c != '-' && *c != '_').collect::<String>().to_ascii_lowercase();
        let u64 = p.starts_with("ultimate64");
        UltimateCaps { streams: u64, machine_input: u64, debugreg: u64, poweroff: u64 }
    }
}

/// Text zu `"key"` aus einem flachen JSON-Objekt (Zeichenketten; einfache Maskierung), sonst leer
fn json_str(body: &str, key: &str) -> String {
    let Some(p) = body.find(&format!("\"{key}\"")) else { return String::new() };
    let rest = body[p + key.len() + 2..].trim_start();
    let Some(rest) = rest.strip_prefix(':') else { return String::new() };
    let Some(rest) = rest.trim_start().strip_prefix('"') else { return String::new() };
    let (mut out, mut esc) = (String::new(), false);
    for c in rest.chars() {
        if esc { out.push(c); esc = false } else if c == '\\' { esc = true } else if c == '"' { break } else { out.push(c) }
    }
    out
}

/// Wert des Debug-Registers aus der Antwort: `{"value": "2A"}` (Text = hexadezimal), `{"value": 42}` (Zahl = dezimal) oder bloßer Hex-Text.
/// Das genaue Format der echten Firmware ist nicht geprüft; was nicht passt, ergibt einen Fehler und der Läufer nimmt das RAM-Postfach.
pub(crate) fn parse_debugreg(body: &[u8]) -> Result<u8, String> {
    let t = String::from_utf8_lossy(body); let t = t.trim();
    let bad = || format!("Debug-Register: Antwort nicht lesbar: {}", t.chars().take(60).collect::<String>());
    if let Some(p) = t.find("\"value\"") {
        let rest = t[p + 7..].trim_start().strip_prefix(':').ok_or_else(bad)?.trim_start();
        if let Some(s) = rest.strip_prefix('"') { let h = s.split('"').next().unwrap_or(""); return u8::from_str_radix(h.trim_start_matches('$').trim_start_matches("0x"), 16).map_err(|_| bad()); }
        let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        return n.parse::<u8>().map_err(|_| bad());
    }
    u8::from_str_radix(t.trim_start_matches('$').trim_start_matches("0x"), 16).map_err(|_| bad())
}

impl Ultimate64 {
    /// Verbindet sich (`host` oder `host:port`) und prüft die Erreichbarkeit über `/v1/version`.
    pub fn connect(host: &str, password: Option<&str>) -> Result<Ultimate64, String> {
        let u = Ultimate64 { host: host.to_string(), password: password.filter(|p| !p.is_empty()).map(String::from), paused: false, info: None };
        u.request("GET", "/v1/version", "", None).map_err(|e| format!("Ultimate 64 unter {host}: {e}"))?;
        Ok(u)
    }
    /// Adresse der Ultimate (`host` oder `host:port`)
    pub fn host(&self) -> &str { &self.host }
    pub fn version(&self) -> Result<String, String> { Ok(String::from_utf8_lossy(&self.request("GET", "/v1/version", "", None)?).into_owned()) }
    /// PRG-Datei laden und starten (Reset, Laden, RUN).
    pub fn run_prg(&mut self, data: &[u8]) -> Result<(), String> { self.request("POST", "/v1/runners:run_prg", "", Some(data))?; self.paused = false; Ok(()) }
    /// Cartridge-Abbild (`.crt`) starten (Reset, Modul einlegen)
    pub fn run_crt(&mut self, data: &[u8]) -> Result<(), String> { self.request("POST", "/v1/runners:run_crt", "", Some(data))?; self.paused = false; Ok(()) }
    /// Produkt, Firmware, Hostname und Core (`GET /v1/info`). Fehler, wenn die Antwort kein Produkt und keine Firmware nennt.
    pub fn info(&self) -> Result<Info, String> {
        let raw = self.request("GET", "/v1/info", "", None)?; let body = String::from_utf8_lossy(&raw);
        let i = Info { product: json_str(&body, "product"), firmware_version: json_str(&body, "firmware_version"), hostname: json_str(&body, "hostname"), core_version: json_str(&body, "core_version") };
        if i.product.is_empty() && i.firmware_version.is_empty() { return Err("keine Geräteinformation in der Antwort".into()); }
        Ok(i)
    }
    /// Fähigkeiten laut Produkt; fragt `/v1/info` einmal ab und merkt sich das Ergebnis. Ohne Antwort: alles `false`.
    pub fn ultimate_caps(&mut self) -> UltimateCaps {
        if self.info.is_none() { self.info = Some(self.info().ok()); }
        self.info.as_ref().and_then(|i| i.as_ref()).map(Info::caps).unwrap_or_default()
    }
    /// Debug-Register (`$D7FF`) lesen: `GET /v1/machine:debugreg`, nur Ultimate 64
    pub fn debugreg(&self) -> Result<u8, String> { parse_debugreg(&self.request("GET", "/v1/machine:debugreg", "", None)?) }
    /// PRG-Datei nur in den Speicher laden (ohne RUN)
    pub fn load_prg(&mut self, data: &[u8]) -> Result<(), String> { self.request("POST", "/v1/runners:load_prg", "", Some(data))?; self.paused = false; Ok(()) }
    pub fn reset(&mut self) -> Result<(), String> { self.request("PUT", "/v1/machine:reset", "", None)?; self.paused = false; Ok(()) }

    fn request(&self, method: &str, path: &str, query: &str, body: Option<&[u8]>) -> Result<Vec<u8>, String> { parse_response(&self.exchange(method, path, query, body)?) }

    /// Datenstrom starten: `PUT /v1/streams/<art>:start?ip=<ip>:<port>`. Die Ultimate sendet ab sofort dorthin. Setzt nur das Ziel für diese Sitzung:
    /// keine Konfigurationsaufrufe und kein Speichern im Flash. Fehlertexte der Ultimate sind übersetzt („Kein LAN-Link“).
    pub fn stream_start(&self, kind: StreamKind, ip: Ipv4Addr, port: u16) -> Result<(), String> {
        let raw = self.exchange("PUT", &format!("/v1/streams/{}:start", kind.rest_name()), &format!("ip={ip}:{port}"), None)?;
        stream_result(&raw, kind, "starten")
    }
    /// Datenstrom beenden: `PUT /v1/streams/<art>:stop`
    pub fn stream_stop(&self, kind: StreamKind) -> Result<(), String> {
        let raw = self.exchange("PUT", &format!("/v1/streams/{}:stop", kind.rest_name()), "", None)?;
        stream_result(&raw, kind, "stoppen")
    }

    fn exchange(&self, method: &str, path: &str, query: &str, body: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let addr = if self.host.contains(':') { self.host.clone() } else { format!("{}:80", self.host) };
        let sa = addr.to_socket_addrs().map_err(|e| format!("{addr}: {e}"))?.next().ok_or("Adresse nicht auflösbar")?;
        let mut s = TcpStream::connect_timeout(&sa, Duration::from_secs(5)).map_err(|e| format!("keine Verbindung: {e}"))?;
        s.set_read_timeout(Some(Duration::from_secs(10))).ok(); s.set_write_timeout(Some(Duration::from_secs(10))).ok();
        let target = if query.is_empty() { path.to_string() } else { format!("{path}?{query}") };
        let mut req = format!("{method} {target} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n", self.host);
        if let Some(p) = &self.password { req.push_str(&format!("X-Password: {p}\r\n")); }
        if let Some(b) = body { req.push_str(&format!("Content-Type: application/octet-stream\r\nContent-Length: {}\r\n", b.len())); }
        req.push_str("\r\n");
        s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        if let Some(b) = body { s.write_all(b).map_err(|e| e.to_string())?; }
        let mut raw = vec![]; s.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        Ok(raw)
    }
}

/// Antwort auf einen Strom-Aufruf auswerten: Status 200 ohne Eintrag in `errors`, sonst übersetzter Fehler (auch aus dem Rumpf nicht-200-Antworten).
pub(crate) fn stream_result(raw: &[u8], kind: StreamKind, what: &str) -> Result<(), String> {
    let (status, body) = parse_http(raw)?;
    let text = String::from_utf8_lossy(&body);
    let errs = json_errors(&text);
    if status == 200 && errs.is_empty() { return Ok(()); }
    let detail = if !errs.is_empty() { errs.join("; ") } else if status == 403 { return Err("Netzwerkpasswort der Ultimate fehlt oder ist falsch (403)".into()) } else if status == 501 { return Err("Datenströme gibt es nur am Ultimate 64 (501)".into()) } else if text.trim().is_empty() { format!("HTTP {status}") } else { text.trim().chars().take(200).collect() };
    Err(translate_stream_error(&detail, kind, what))
}

/// Einträge aus `"errors": [ "…", … ]` (nur Zeichenketten)
fn json_errors(body: &str) -> Vec<String> {
    let Some(p) = body.find("\"errors\"") else { return vec![] };
    let rest = &body[p + 8..];
    let Some(a) = rest.find('[') else { return vec![] };
    let Some(b) = rest[a..].find(']') else { return vec![] };
    let (mut out, mut cur, mut inside, mut esc) = (vec![], String::new(), false, false);
    for c in rest[a + 1..a + b].chars() {
        if inside { if esc { cur.push(c); esc = false } else if c == '\\' { esc = true } else if c == '"' { out.push(std::mem::take(&mut cur)); inside = false } else { cur.push(c) } }
        else if c == '"' { inside = true }
    }
    out.into_iter().filter(|e| !e.trim().is_empty()).collect()
}

/// Fehlertext der Ultimate für Menschen: bekannte Meldungen übersetzen, sonst den Originaltext anhängen
pub(crate) fn translate_stream_error(raw: &str, kind: StreamKind, what: &str) -> String {
    let l = raw.to_ascii_lowercase();
    if l.contains("no operational network interface") || l.contains("no (valid) link") {
        return "Kein LAN-Link: Die Ultimate hat kein LAN-Kabel. Die Ströme laufen nur über den LAN-Anschluss, nicht über WLAN (Meldung der Ultimate: „No Operational Network Interface“).".into();
    }
    format!("Strom „{}“ lässt sich nicht {what}: {raw}", kind.rest_name())
}

/// HTTP-Antwort zerlegen: Statuscode prüfen, Rumpf (Content-Length, chunked oder bis Verbindungsende) liefern.
pub(crate) fn parse_response(raw: &[u8]) -> Result<Vec<u8>, String> {
    let (status, body) = parse_http(raw)?;
    if status != 200 { return Err(match status { 403 => "403: Netzwerkpasswort fehlt oder falsch".into(), 501 => "501: Funktion nur am Ultimate 64".into(), c => format!("HTTP {c}") }); }
    Ok(body)
}

/// Statuscode und Rumpf (Content-Length, chunked oder bis Verbindungsende)
pub(crate) fn parse_http(raw: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or("ungültige HTTP-Antwort")?;
    let head = String::from_utf8_lossy(&raw[..split]).into_owned(); let body = &raw[split + 4..];
    let status: u16 = head.lines().next().and_then(|l| l.split_whitespace().nth(1)).and_then(|c| c.parse().ok()).ok_or("ungültige Statuszeile")?;
    let lower = head.to_ascii_lowercase();
    if lower.contains("transfer-encoding: chunked") {
        let (mut out, mut p) = (vec![], 0usize);
        loop {
            let eol = body.get(p..).ok_or("chunked: abgeschnitten")?.windows(2).position(|w| w == b"\r\n").ok_or("chunked: Länge fehlt")?;
            let n = usize::from_str_radix(String::from_utf8_lossy(&body[p..p + eol]).split(';').next().unwrap_or("").trim(), 16).map_err(|_| "chunked: ungültige Länge")?;
            p += eol + 2; if n == 0 { break; }
            if p + n > body.len() { return Err("chunked: abgeschnitten".into()); }
            out.extend_from_slice(&body[p..p + n]); p += n + 2;
        }
        return Ok((status, out));
    }
    if let Some(l) = lower.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()) { if body.len() < l { return Err("Antwort abgeschnitten".into()); } return Ok((status, body[..l].to_vec())); }
    Ok((status, body.to_vec()))
}

impl Backend for Ultimate64 {
    fn debug_reg(&mut self) -> Option<Result<u8, String>> { if self.ultimate_caps().debugreg { Some(self.debugreg()) } else { None } }
    fn name(&self) -> &str { "Ultimate 64" }
    fn caps(&self) -> Caps { Caps { registers: false, step: false, breakpoints: false, watch: false } }
    fn state(&mut self) -> RunState { if self.paused { RunState::Stopped } else { RunState::Running } }
    fn registers(&mut self) -> Result<Regs, String> { Err("Die Ultimate-Hardware liefert keine CPU-Register".into()) }
    fn read_mem(&mut self, addr: u16, len: usize) -> Result<Vec<u8>, String> {
        let (mut out, mut a) = (Vec::with_capacity(len), addr as usize);
        while out.len() < len && a <= 0xFFFF {
            let n = (len - out.len()).min(0x10000 - a).min(4096);
            let b = self.request("GET", "/v1/machine:readmem", &format!("address={a:04X}&length={n}"), None)?;
            if b.len() != n { return Err(format!("readmem lieferte {} statt {n} Bytes", b.len())); }
            out.extend_from_slice(&b); a += n;
        }
        Ok(out)
    }
    fn write_mem(&mut self, addr: u16, data: &[u8]) -> Result<(), String> {
        let mut a = addr as usize;
        for chunk in data.chunks(4096) {
            if a + chunk.len() > 0x10000 { return Err("Schreibzugriff über $FFFF hinaus".into()); }
            if chunk.len() <= 128 { let hex: String = chunk.iter().map(|b| format!("{b:02X}")).collect(); self.request("PUT", "/v1/machine:writemem", &format!("address={a:04X}&data={hex}"), None)?; }
            else { self.request("POST", "/v1/machine:writemem", &format!("address={a:04X}"), Some(chunk))?; }
            a += chunk.len();
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> { Err("Einzelschritt gibt es auf der Ultimate-Hardware nicht".into()) }
    fn cont(&mut self) -> Result<(), String> { self.request("PUT", "/v1/machine:resume", "", None)?; self.paused = false; Ok(()) }
    fn can_pause(&self) -> bool { true }
    fn pause(&mut self) -> Result<(), String> { self.request("PUT", "/v1/machine:pause", "", None)?; self.paused = true; Ok(()) }
    fn set_breakpoint(&mut self, _addr: u16) -> Result<(), String> { Err("Haltepunkte gibt es auf der Ultimate-Hardware nicht".into()) }
    fn clear_breakpoint(&mut self, _addr: u16) -> Result<(), String> { Ok(()) }
    fn breakpoints(&self) -> Vec<u16> { vec![] }
    fn stop(&mut self) { let _ = self.request("PUT", "/v1/machine:resume", "", None); }       // Rechner nicht angehalten zurücklassen
}

#[cfg(test)]
mod stream_tests {
    use super::*;
    fn http(status: &str, body: &str) -> Vec<u8> { format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n{body}", body.len()).into_bytes() }
    #[test]
    fn stream_result_translates_errors() {
        let k = StreamKind::Video;
        assert_eq!(stream_result(&http("200 OK", "{ \n  \"errors\" : [  ]\n}"), k, "starten"), Ok(()));
        let e = stream_result(&http("200 OK", "{\"errors\":[\"No Operational Network Interface\"]}"), k, "starten").unwrap_err();
        assert!(e.starts_with("Kein LAN-Link"), "{e}");
        let e = stream_result(&http("400 Bad Request", "{\"errors\":[\"No Operational Network Interface\"]}"), k, "starten").unwrap_err();
        assert!(e.starts_with("Kein LAN-Link"), "{e}");
        let e = stream_result(&http("400 Bad Request", "{\"errors\":[\"Irgendwas anderes\"]}"), k, "starten").unwrap_err();
        assert!(e.contains("video") && e.contains("Irgendwas anderes") && e.contains("starten"), "{e}");
        assert!(stream_result(&http("403 Forbidden", ""), k, "stoppen").unwrap_err().contains("403"));
        assert!(stream_result(&http("500 Error", ""), k, "stoppen").unwrap_err().contains("HTTP 500"));
        assert!(stream_result(b"Mist", k, "stoppen").is_err());
    }
    #[test]
    fn json_errors_reads_strings() {
        assert_eq!(json_errors("{\"errors\": [\"a\", \"b \\\"c\\\"\"]}"), vec!["a".to_string(), "b \"c\"".to_string()]);
        assert!(json_errors("{\"errors\": []}").is_empty() && json_errors("kein json").is_empty() && json_errors("{\"errors\": [").is_empty());
    }
    #[test]
    fn chunked_truncated_does_not_panic() { assert!(parse_http(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nab").is_err()); assert!(parse_http(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1\r\na").is_err()); }
}
