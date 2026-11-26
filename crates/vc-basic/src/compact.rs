// SPDX-License-Identifier: GPL-3.0-or-later
//! Bauoption „kompakt“ (Konzept „BASIC-Editor und Dialekte“, 6.4, Entscheidung B16; Standard: aus): entfernt Leerzeichen innerhalb der Anweisungen,
//! **wo es nichts ändert**. Das Ergebnis spart Speicher und Laufzeit (Messung B0: 84 → 54 Bytes und rund 4 % Laufzeit bei sehr
//! großzügig gesetzten Leerzeichen).
//!
//! Sicherheit ist **durch Prüfung** gegeben, nicht durch Regeln: Ein Leerzeichen entfällt nur, wenn die Token der Zeile danach (ohne die
//! Leerzeichen außerhalb von Zeichenketten, `REM` und `DATA`, die der Interpreter ohnehin überliest) genau dieselben sind wie vorher.
//! Wo ein Schlüsselwort sonst mit einem Namen verklebt und anders gelesen würde (`GO TO` → `GOTO`), bleibt das Leerzeichen stehen.
//! Zeichenketten, `REM`-Reste und `DATA`-Werte bleiben unverändert. Nur Commodore: der Atari speichert Leerzeichen ohnehin nicht.
use crate::dialect::Dialect;

/// Token der Zeile ohne die Leerzeichen, die der Interpreter überliest (außerhalb von Zeichenketten, `REM`-Rest und `DATA`-Werten)
fn normalized(bytes: &[u8], d: &Dialect) -> Vec<u8> {
    let (rem, data) = (d.tok("REM"), d.tok("DATA"));
    let (mut out, mut quote, mut in_data, mut in_rem) = (vec![], false, false, false);
    for &b in bytes {
        if in_rem { out.push(b); continue; }
        if b == 0x22 { quote = !quote; out.push(b); continue; }
        if quote { out.push(b); continue; }
        if in_data { if b == b':' { in_data = false; } out.push(b); continue; }
        if b == rem { in_rem = true; }
        if b == data { in_data = true; }
        if b != 0x20 { out.push(b); }
    }
    out
}

/// Zeichenindizes der Leerzeichen, die außerhalb von Zeichenketten, `REM` und `DATA` stehen
fn candidates(rest: &str, d: &Dialect) -> Vec<usize> {
    let ch: Vec<char> = rest.chars().collect();
    let kws = crate::classic::keyword_spans(rest, d);
    let byte_of: Vec<usize> = rest.char_indices().map(|(b, _)| b).collect();
    let (rem, data) = (d.tok("REM"), d.tok("DATA"));
    let _ = (rem, data);
    let (mut out, mut quote, mut in_data) = (vec![], false, false);
    let mut i = 0;
    while i < ch.len() {
        let c = ch[i];
        if c == '{' { while i < ch.len() && ch[i] != '}' { i += 1; } i += 1; continue; }
        if c == '"' { quote = !quote; i += 1; continue; }
        if quote { i += 1; continue; }
        if in_data { if c == ':' { in_data = false; } i += 1; continue; }
        if let Some((s, e)) = kws.iter().find(|(s, _)| *s == byte_of[i]) {
            let w = rest[*s..*e].to_ascii_uppercase();
            if w == "REM" { break; }
            if w == "DATA" { in_data = true; }
        }
        if c == ' ' { out.push(i); }
        i += 1;
    }
    out
}

/// Kompakte Fassung des Teils einer Zeile nach der Zeilennummer
pub fn compact_line(rest: &str, d: &Dialect) -> String {
    let cands = candidates(rest, d);
    if cands.is_empty() { return rest.to_string(); }
    let Ok(orig) = crate::classic::tokenize_line(rest, d) else { return rest.to_string() };
    let want = normalized(&orig, d);
    let ch: Vec<char> = rest.chars().collect();
    let build = |skip: &std::collections::HashSet<usize>| -> String { ch.iter().enumerate().filter(|(i, _)| !skip.contains(i)).map(|(_, c)| *c).collect() };
    let same = |s: &str| crate::classic::tokenize_line(s, d).map_or(false, |b| normalized(&b, d) == want);
    let all: std::collections::HashSet<usize> = cands.iter().copied().collect();
    let t = build(&all);
    if same(&t) { return t; }
    // einzeln prüfen
    let mut skip = std::collections::HashSet::new();
    for c in cands { skip.insert(c); if !same(&build(&skip)) { skip.remove(&c); } }
    build(&skip)
}

/// Kompakter Text mit Zeilennummern (`NNN anweisung`, wie `number_lines` ihn liefert); Leerzeilen bleiben
pub fn compact_text(text: &str, d: &Dialect) -> String {
    let mut out: Vec<String> = text.lines().map(|l| {
        let t = l.trim_start();
        let digits: String = t.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() { return l.to_string(); }
        format!("{} {}", digits, compact_line(t[digits.len()..].trim_start(), d))
    }).collect();
    if text.ends_with('\n') { out.push(String::new()); }
    out.join("\n")
}

/// Sind zwei Tokenzeilen für den Interpreter gleich (ohne die überlesenen Leerzeichen)?
pub fn same_tokens(a: &[u8], b: &[u8], d: &Dialect) -> bool { normalized(a, d) == normalized(b, d) }
