// SPDX-License-Identifier: GPL-3.0-or-later
//! Lesen und Schreiben von Textdateien mit Erhalt von Kodierung und Zeilenende (Stufe 4c). Der Editor arbeitet immer mit UTF-8 und `\n`;
//! beim Öffnen wird erkannt, wie die Datei vorliegt (UTF-8, UTF-8 mit BOM, sonst ISO-8859-1; LF oder CRLF), und beim Speichern genau so geschrieben,
//! damit ein Öffnen und Speichern eine fremde Datei nicht still umwandelt. Beides lässt sich in der Statusleiste umschalten.
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meta { pub crlf: bool, pub latin1: bool, pub bom: bool }
/// Neue Dokumente: UTF-8 ohne BOM, Zeilenende der Plattform (Windows CRLF, sonst LF)
impl Default for Meta { fn default() -> Self { Meta { crlf: cfg!(windows), latin1: false, bom: false } } }
impl Meta {
    pub fn kodierung(&self) -> &'static str { if self.latin1 { "ISO-8859-1" } else if self.bom { "UTF-8 mit BOM" } else { "UTF-8" } }
    pub fn zeilenende(&self) -> &'static str { if self.crlf { "CRLF" } else { "LF" } }
}

/// Bytes lesen: (Text mit `\n`, Meta). UTF-8 (auch mit BOM) sonst ISO-8859-1 (jedes Byte ein Zeichen, verlustfrei)
pub fn dekodieren(bytes: &[u8]) -> (String, Meta) { dekodieren_fuer(bytes, cfg!(windows)) }
/// Wie [`dekodieren`], mit der Plattform-Vorgabe für das Zeilenende, wenn der Text keinen Zeilenumbruch enthält (leere und neue Dateien: Windows CRLF, sonst LF)
pub fn dekodieren_fuer(bytes: &[u8], plattform_crlf: bool) -> (String, Meta) {
    let (bom, rest) = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) { Some(r) => (true, r), None => (false, bytes) };
    let (text, latin1) = match std::str::from_utf8(rest) { Ok(s) => (s.to_string(), false), Err(_) => (bytes.iter().map(|b| *b as char).collect(), true) };
    let (bom, text) = if latin1 { (false, text) } else { (bom, text) };
    // keine Umbrüche (leer, neu, eine Zeile): Vorgabe der Plattform; sonst entscheidet die Mehrheit der Umbrüche (eine fremde Datei bleibt, wie sie ist)
    let crlf = if !text.contains('\n') && !text.contains('\r') { plattform_crlf } else { text.contains("\r\n") && text.matches("\r\n").count() * 2 >= text.matches('\n').count() };
    (if crlf || text.contains('\r') { text.replace("\r\n", "\n").replace('\r', "\n") } else { text }, Meta { crlf, latin1, bom })
}

pub fn lesen(path: &Path) -> Result<(String, Meta), String> { std::fs::read(path).map(|b| dekodieren(&b)).map_err(|e| format!("{}: {e}", path.display())) }

/// Text so kodieren, wie `meta` es verlangt; Zeichen außerhalb von ISO-8859-1 gehen dort nicht: Fehler
pub fn kodieren(text: &str, meta: &Meta) -> Result<Vec<u8>, String> {
    let t = if meta.crlf { text.replace('\n', "\r\n") } else { text.to_string() };
    if meta.latin1 {
        let mut out = Vec::with_capacity(t.len());
        for c in t.chars() { if (c as u32) < 256 { out.push(c as u32 as u8) } else { return Err(format!("Zeichen „{c}“ gibt es in ISO-8859-1 nicht: Kodierung auf UTF-8 stellen")) } }
        return Ok(out)
    }
    let mut out = if meta.bom { vec![0xEF, 0xBB, 0xBF] } else { vec![] }; out.extend_from_slice(t.as_bytes()); Ok(out)
}

pub fn schreiben(path: &Path, text: &str, meta: &Meta) -> Result<(), String> { std::fs::write(path, kodieren(text, meta)?).map_err(|e| format!("{}: {e}", path.display())) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn utf8_lf_wird_unveraendert_gelesen_und_geschrieben() { let b = "a\nb ä\n".as_bytes(); let (t, m) = dekodieren(b); assert_eq!((t.as_str(), m), ("a\nb ä\n", Meta::default())); assert_eq!(kodieren(&t, &m).unwrap(), b); }
    #[test] fn crlf_und_bom_bleiben_erhalten() { let mut b = vec![0xEF, 0xBB, 0xBF]; b.extend_from_slice(b"x\r\ny\r\n"); let (t, m) = dekodieren(&b); assert_eq!(t, "x\ny\n"); assert!(m.crlf && m.bom && !m.latin1); assert_eq!(kodieren(&t, &m).unwrap(), b); }
    #[test] fn latin1_wird_verlustfrei_gelesen_und_geschrieben() { let b = vec![b'a', 0xE4, 0xFC, b'\n']; let (t, m) = dekodieren(&b); assert!(m.latin1); assert_eq!(t, "aäü\n"); assert_eq!(kodieren(&t, &m).unwrap(), b); assert!(kodieren("€", &m).is_err(), "Euro gibt es in Latin-1 nicht"); }
    #[test] fn umschalten_der_kodierung_schreibt_neu() { let m = Meta { latin1: false, ..Default::default() }; assert_eq!(kodieren("ä", &m).unwrap(), "ä".as_bytes()); assert_eq!(kodieren("ä", &Meta { latin1: true, ..m }).unwrap(), vec![0xE4]); }
    #[test] fn leere_und_neue_dateien_bekommen_das_zeilenende_der_plattform_fremde_bleiben() {
        assert!(!dekodieren_fuer(b"", false).1.crlf && dekodieren_fuer(b"", true).1.crlf, "leer: Windows CRLF, sonst LF"); assert!(dekodieren_fuer(b"nur eine Zeile", true).1.crlf);
        assert!(!dekodieren_fuer(b"a\nb\n", true).1.crlf, "vorhandene LF-Datei bleibt LF, auch unter Windows"); assert!(dekodieren_fuer(b"a\r\nb\r\n", false).1.crlf, "vorhandene CRLF-Datei bleibt CRLF");
        let (_, m) = dekodieren_fuer(b"", false); assert!(!m.bom && !m.latin1, "neu: UTF-8 ohne BOM");
    }
    #[test] fn gemischtes_zeilenende_zaehlt_nach_mehrheit() { let (_, m) = dekodieren(b"a\r\nb\nc\n"); assert!(!m.crlf); let (_, m) = dekodieren(b"a\r\nb\r\nc\n"); assert!(m.crlf); }
}
