// SPDX-License-Identifier: GPL-3.0-or-later
//! Textdateien lesen und schreiben: im Projekt gilt immer **UTF-8 ohne BOM mit LF** als Zeilenende.
//! Beim Lesen werden fremde Formen toleriert (BOM, CRLF/CR, Windows-1252) und gemeldet.

/// Ergebnis des Lesens: Text in Normalform und die Abweichungen, die dabei bereinigt wurden.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decoded {
    pub text: String,
    /// Die Datei hatte ein UTF-8-BOM (wurde entfernt).
    pub had_bom: bool,
    /// Die Datei hatte CRLF oder CR als Zeilenende (wurde zu LF).
    pub had_crlf: bool,
    /// Die Datei war kein gültiges UTF-8, sondern wurde als Windows-1252 gelesen.
    pub was_legacy: bool,
}

impl Decoded {
    /// Hinweise für die Statusleiste; leer, wenn die Datei schon in Normalform war.
    pub fn notes(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.had_bom { v.push("BOM entfernt"); }
        if self.was_legacy { v.push("als Windows-1252 gelesen, wird als UTF-8 gespeichert"); }
        if self.had_crlf { v.push("Zeilenenden CRLF → LF"); }
        v
    }
    pub fn changed(&self) -> bool { self.had_bom || self.had_crlf || self.was_legacy }
}

/// Windows-1252 für 0x80..=0x9F (nicht belegte Stellen bleiben Steuerzeichen).
const CP1252: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// Bytes → Text in Normalform. `None` bei Binärdaten (enthalten NUL-Bytes).
pub fn decode(bytes: &[u8]) -> Option<Decoded> {
    if bytes.contains(&0) { return None; }
    let mut d = Decoded::default();
    let body = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) { Some(b) => { d.had_bom = true; b } None => bytes };
    let raw = match std::str::from_utf8(body) {
        Ok(s) => s.to_string(),
        Err(_) => {
            d.was_legacy = true;
            body.iter().map(|&b| match b { 0x80..=0x9F => CP1252[(b - 0x80) as usize], _ => b as char }).collect()
        }
    };
    if raw.contains('\r') {
        d.had_crlf = true;
        d.text = raw.replace("\r\n", "\n").replace('\r', "\n");
    } else {
        d.text = raw;
    }
    Some(d)
}

/// Text für die Datei: UTF-8 ohne BOM, Zeilenende LF.
pub fn encode(text: &str) -> Vec<u8> {
    let t = text.strip_prefix('\u{feff}').unwrap_or(text);
    if t.contains('\r') { t.replace("\r\n", "\n").replace('\r', "\n").into_bytes() } else { t.as_bytes().to_vec() }
}

/// Quelltext aus einer Datei lesen (BOM und CRLF tolerant); Fehlertext wie `std::fs::read_to_string`.
pub fn read_source(path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
    let bytes = std::fs::read(path.as_ref())?;
    decode(&bytes).map(|d| d.text).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Binärdatei"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_utf8_is_unchanged() {
        let d = decode("Grüße\nZeile 2\n".as_bytes()).unwrap();
        assert_eq!(d.text, "Grüße\nZeile 2\n");
        assert!(!d.changed() && d.notes().is_empty());
    }
    #[test]
    fn bom_is_removed() {
        let d = decode(b"\xEF\xBB\xBFlda #1\n").unwrap();
        assert_eq!(d.text, "lda #1\n");
        assert!(d.had_bom && !d.had_crlf && !d.was_legacy);
        assert_eq!(d.notes(), vec!["BOM entfernt"]);
    }
    #[test]
    fn crlf_and_cr_become_lf() {
        let d = decode(b"a\r\nb\rc\n").unwrap();
        assert_eq!(d.text, "a\nb\nc\n");
        assert!(d.had_crlf);
    }
    #[test]
    fn windows_1252_is_read_as_text() {
        let d = decode(b"gr\xFC\xDFe \x80 \x93x\x94").unwrap();
        assert_eq!(d.text, "grüße € “x”");
        assert!(d.was_legacy);
    }
    #[test]
    fn binary_is_rejected() {
        assert!(decode(&[0x01, 0x08, 0x00, 0x20]).is_none());
    }
    #[test]
    fn encode_writes_utf8_without_bom_and_lf() {
        assert_eq!(encode("\u{feff}a\r\nb\n"), b"a\nb\n".to_vec());
        assert_eq!(encode("ä"), "ä".as_bytes().to_vec());
    }
    #[test]
    fn roundtrip_is_stable() {
        let first = decode(b"\xEF\xBB\xBFx\r\ny\xE4\r\n").unwrap();
        let again = decode(&encode(&first.text)).unwrap();
        assert_eq!(again.text, first.text);
        assert!(!again.changed());
    }
}
