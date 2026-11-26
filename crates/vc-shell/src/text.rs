// SPDX-License-Identifier: GPL-3.0-or-later
//! Textdateien für den Code-Editor (E1.1): Lesen und Schreiben ohne Verlust (Zeilenenden, Endzeile bleiben), Färbung und Faltung aus `vc-ide-model`.
//! Der Text selbst und sein Verlauf liegen im Editor der Oberfläche (CodeMirror); Rust liefert Inhalt, Token-Arten und Faltbereiche.
use crate::docstore::Fehler;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;
use vc_ide_model::lang::Lang;
use vc_ide_model::syntax::{self, Kind};

/// Befehlsnamen für die Färbung: alle Mnemonics der Prozessortabellen (`data/cpus/*.cpu.json`: NMOS 6502 samt illegalen Befehlen, 45GS02 samt Quad-Befehlen)
const CPUS: &[&str] = &[include_str!("../../../data/cpus/nmos6502.cpu.json"), include_str!("../../../data/cpus/45gs02.cpu.json")];
fn mnemonics() -> &'static HashSet<String> {
    static S: OnceLock<HashSet<String>> = OnceLock::new();
    S.get_or_init(|| {
        let mut s = HashSet::new();
        for text in CPUS {
            let Ok(v) = serde_json::from_str::<Value>(text) else { continue };
            for op in v["ops"].as_array().into_iter().flatten() { if let Some(m) = op.get(0).and_then(Value::as_str) { s.insert(m.to_ascii_lowercase()); } }
            for k in v["quadOps"].as_object().into_iter().flat_map(|o| o.keys()) { s.insert(k.to_ascii_lowercase()); }
        }
        s
    })
}

fn art(k: Kind) -> &'static str {
    match k { Kind::Comment => "comment", Kind::Str => "string", Kind::Num => "number", Kind::Hex => "hex", Kind::Bin => "bin", Kind::Bad => "bad", Kind::Directive => "directive",
        Kind::Mnemonic => "mnemonic", Kind::Label => "label", Kind::Symbol => "symbol", Kind::Keyword => "keyword", Kind::Register => "register", Kind::Plain => "plain" }
}

pub fn sprache(rel: &str) -> Lang { Lang::of(rel) }
fn sprach_name(l: Lang) -> &'static str { match l { Lang::Asm => "asm", Lang::Basic => "basic", Lang::Markdown => "markdown", Lang::Plain => "plain" } }
fn sprache_aus(s: &str) -> Lang { match s { "asm" => Lang::Asm, "basic" => Lang::Basic, "markdown" => Lang::Markdown, _ => Lang::Plain } }

fn io(rel: &str, e: &dyn std::fmt::Display) -> Fehler { Fehler::mit("datei-fehler", json!({ "rel": rel, "detail": e.to_string() })) }

/// Datei lesen. Zeilenenden werden erkannt (`lf`, `crlf`, `gemischt`) und im Text auf `\n` vereinheitlicht; `datei_schreiben` stellt sie wieder her.
pub fn lesen(root: &Path, rel: &str) -> Result<Value, Fehler> {
    let rel = vc_ide_model::fileops::safe_rel(rel).map_err(|e| Fehler::mit("pfad-ungueltig", json!({ "detail": e })))?;
    let bytes = std::fs::read(vc_ide::safepath::resolve(root, &rel).map_err(|e| Fehler::mit("pfad-ungueltig", json!({ "detail": e })))?).map_err(|e| io(&rel, &e))?;
    let s = String::from_utf8(bytes).map_err(|_| Fehler::mit("datei-kein-utf8", json!({ "rel": rel })))?;
    let (crlf, lf) = (s.matches("\r\n").count(), s.matches('\n').count());
    let eol = if crlf == 0 { "lf" } else if crlf == lf { "crlf" } else { "gemischt" };
    let text = if crlf > 0 { s.replace("\r\n", "\n") } else { s };
    Ok(json!({ "rel": rel, "text": text, "eol": eol, "lang": sprach_name(sprache(&rel)), "bytes": lf }))
}

/// Datei schreiben, atomar (erst Nachbardatei, dann umbenennen). `eol`: `crlf` stellt Windows-Zeilenenden her, sonst bleibt `\n`.
pub fn schreiben(root: &Path, rel: &str, text: &str, eol: &str) -> Result<(), Fehler> {
    let rel = vc_ide_model::fileops::safe_rel(rel).map_err(|e| Fehler::mit("pfad-ungueltig", json!({ "detail": e })))?;
    let ziel = vc_ide::safepath::resolve(root, &rel).map_err(|e| Fehler::mit("pfad-ungueltig", json!({ "detail": e })))?;
    let out = if eol == "crlf" { text.replace('\n', "\r\n") } else { text.to_string() };
    vc_core::atomar::schreiben(&ziel, out.as_bytes()).map_err(|e| io(&rel, &e))
}

/// Byte-Offset in einer Zeile → UTF-16-Einheiten (CodeMirror zählt so)
fn u16_at(line: &str, byte: usize) -> usize { line[..byte.min(line.len())].encode_utf16().count() }

/// Färbung: je Zeile Liste `[von, bis, art]` in UTF-16-Einheiten; Zeilen ohne Färbung fehlen. Rückgabe: `[[zeile, [[von, bis, art], …]], …]` (Zeilen ab 0).
pub fn faerbung(text: &str, lang: &str) -> Value { faerbung_q(text, lang, true) }

/// Wie [`faerbung`]; `quote`: `'` ist im BASIC ein Kommentar (Commodore: nein)
pub fn faerbung_q(text: &str, lang: &str, quote: bool) -> Value {
    if lang == "markdown" { return md_faerbung(text); }
    let l = sprache_aus(lang);
    if l.is_text() { return json!([]); }
    let mut out = vec![];
    for (i, line) in text.split('\n').enumerate() {
        let toks: Vec<Value> = syntax::tokens_q(line, l, mnemonics(), None, quote).into_iter().filter(|t| t.2 != Kind::Plain).map(|(a, b, k)| json!([u16_at(line, a), u16_at(line, b), art(k)])).collect();
        if !toks.is_empty() { out.push(json!([i, toks])); }
    }
    Value::Array(out)
}

/// Markdown: Farben je Zeile mit Zustand über die Zeilen (Zaunblöcke, Front Matter); Arten mit Vorsatz `md-`
fn md_faerbung(text: &str) -> Value {
    use vc_md::lex::Kind as K;
    let name = |k: K| match k { K::Marker => "md-marker", K::Heading => "md-heading", K::Bold => "md-bold", K::Italic => "md-italic", K::Strike => "md-strike", K::Code => "md-code", K::CodeBlock => "md-codeblock", K::Fence => "md-fence",
        K::LinkText => "md-linktext", K::Url => "md-url", K::ListMark => "md-listmark", K::Quote => "md-quote", K::Rule => "md-rule", K::Html => "md-html", K::Front => "md-front", K::Table => "md-table" };
    let mut st = vc_md::lex::State::new(text); let mut out = vec![];
    for (i, line) in text.split('\n').enumerate() {
        let toks: Vec<Value> = vc_md::lex::line(line, i, &mut st).into_iter().map(|(a, b, k)| json!([u16_at(line, a), u16_at(line, b), name(k)])).collect();
        if !toks.is_empty() { out.push(json!([i, toks])); }
    }
    Value::Array(out)
}

/// Faltbereiche `[erste Zeile, letzte Zeile]` (ab 0)
pub fn faltung(text: &str, lang: &str, quote: bool) -> Value { Value::Array(vc_ide_model::fold::ranges_q(text, sprache_aus(lang), quote).into_iter().map(|(a, b)| json!([a, b])).collect()) }

/// Spalte in UTF-16-Einheiten → Spalte in Zeichen (mitten in einem Ersatzpaar: davor)
pub fn spalte_zeichen(line: &str, col16: usize) -> usize {
    let mut n = 0;
    for (i, c) in line.chars().enumerate() { if n >= col16 { return i; } n += c.len_utf16(); }
    line.chars().count()
}
/// Spalte in Zeichen → UTF-16-Einheiten
pub fn spalte_16(line: &str, col: usize) -> usize { line.chars().take(col).map(char::len_utf16).sum() }

/// Karte für die Stelle (`line` ab 0, `col16` in UTF-16-Einheiten): `{name}`-Zeichenangabe in einer Zeichenkette, sonst im nativen BASIC Zahlen, Register und Wertzerlegung.
/// Antwort `{ markdown, line, endLine, from, to }` oder null. `sys` ist das Zielsystem der Datei (`c64` …), `dialect` der BASIC-Dialekt des Ziels.
pub fn karte(data: &Path, sys: &str, dialect: &str, rel: &str, lang: &str, text: &str, line: usize, col16: usize) -> Value {
    let l = sprache_aus(lang);
    let Some(src) = text.split('\n').nth(line) else { return Value::Null };
    let col = spalte_zeichen(src, col16);
    let byte = src.char_indices().nth(col).map_or(src.len(), |x| x.0);
    if l == Lang::Basic { vc_ide_model::editor_names::select_name_dialect(data, dialect); }
    for sp in vc_ide_model::charesc::spans(src, l) {
        if byte >= sp.a && byte < sp.z {
            if let Some(md) = vc_ide_model::charesc::hover(l, src, &src[sp.a + 1..sp.z - 1]) {
                return json!({ "markdown": md, "line": line, "endLine": line, "from": u16_at(src, sp.a), "to": u16_at(src, sp.z) });
            }
        }
    }
    let nativ = { let r = rel.to_ascii_lowercase(); r.ends_with(".bas") || r.ends_with(".lst") || r.ends_with(".tur") };
    if l == Lang::Basic && nativ {
        if let Some(k) = vc_ide_model::basichover::hover(data, sys, dialect, text, line, col) {
            return json!({ "markdown": k.md, "line": line, "endLine": line, "from": spalte_16(src, k.start), "to": spalte_16(src, k.end.max(k.start + 1)) });
        }
    }
    Value::Null
}

/// Namen für die Vervollständigung nach `{` in einer Zeichenkette: `[ [name, beschreibung] ]`, leer, wenn der Cursor nicht hinter einer offenen `{` steht
pub fn zeichennamen(data: &Path, dialect: &str, lang: &str, line_text: &str, col16: usize) -> Value {
    let l = sprache_aus(lang);
    let col = spalte_zeichen(line_text, col16);
    let byte = line_text.char_indices().nth(col).map_or(line_text.len(), |x| x.0);
    let before = &line_text[..byte];
    if vc_ide_model::charesc::open_brace(before).is_none() { return json!([]); }
    // innerhalb einer Zeichenkette? (Anführungszeichen vor dem Cursor ungerade)
    if before.matches('"').count() % 2 == 0 { return json!([]); }
    if l == Lang::Basic { vc_ide_model::editor_names::select_name_dialect(data, dialect); }
    let fam = vc_ide_model::charesc::family(l, line_text);
    Value::Array(vc_ide_model::charesc::completions(l, fam).into_iter().map(|(n, d)| json!([n, d])).collect())
}

/// Prettyprint der Zeilen `von..=bis` (ab 0): `[[zeile, neuer Text], …]` nur für Zeilen, die sich ändern. Assembler: Spalten für Marke, Befehl und Kommentar;
/// strukturiertes BASIC rückt nach Blocktiefe der Zeilen davor ein (Kontext), natives BASIC (`.bas`, `.lst`, `.tur`) fasst Leerzeichen zusammen. Andere Arten bleiben unberührt.
pub fn formatieren(text: &str, rel: &str, lang: &str, von: usize, bis: usize) -> Value {
    use vc_ide_model::pretty;
    let l = sprache_aus(lang);
    if !matches!(l, Lang::Asm | Lang::Basic) { return json!([]); }
    let lines: Vec<&str> = text.split('\n').collect();
    let nativ = { let r = rel.to_ascii_lowercase(); r.ends_with(".bas") || r.ends_with(".lst") || r.ends_with(".tur") };
    let mut out = vec![];
    for i in von..=bis.min(lines.len().saturating_sub(1)) {
        let neu = match l { Lang::Asm => pretty::asm_line(lines[i], mnemonics()), _ if nativ => pretty::native_basic_line(lines[i]), _ => pretty::basic_line(&lines, i) };
        if neu != lines[i] { out.push(json!([i, neu])); }
    }
    Value::Array(out)
}

#[cfg(test)]
mod md_tests {
    #[test] fn markdown_wird_gefaerbt() {
        let v = super::faerbung("# Titel\n\nText **fett** `code`\n```\nx\n```\n", "markdown");
        let arten: Vec<String> = v.as_array().unwrap().iter().flat_map(|z| z[1].as_array().unwrap().iter().map(|t| t[2].as_str().unwrap().to_string()).collect::<Vec<_>>()).collect();
        for a in ["md-heading", "md-bold", "md-code", "md-fence", "md-codeblock"] { assert!(arten.iter().any(|x| x == a), "{a} fehlt in {arten:?}"); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tmp(n: &str) -> std::path::PathBuf { let d = std::env::temp_dir().join(format!("vc_text_{n}_{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
    #[test] fn rundlauf_ist_byte_gleich_fuer_lf_crlf_und_ohne_endzeile() {
        let r = tmp("rund");
        for (n, inhalt) in [("a.asm", "lda #$00\nrts\n"), ("b.asm", "lda #$00\r\nrts\r\n"), ("c.asm", "rts"), ("d.asm", "\n\n"), ("e.asm", "ä ö ü ß\n")] {
            std::fs::write(r.join(n), inhalt).unwrap();
            let v = lesen(&r, n).unwrap();
            schreiben(&r, n, v["text"].as_str().unwrap(), v["eol"].as_str().unwrap()).unwrap();
            assert_eq!(std::fs::read(r.join(n)).unwrap(), inhalt.as_bytes(), "{n}");
        }
        assert_eq!(lesen(&r, "b.asm").unwrap()["eol"], "crlf"); assert_eq!(lesen(&r, "a.asm").unwrap()["eol"], "lf");
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn kein_utf8_und_pfad_ausserhalb_werden_gemeldet() {
        let r = tmp("fehler"); std::fs::write(r.join("x.bin"), [0xffu8, 0xfe]).unwrap();
        assert_eq!(lesen(&r, "x.bin").unwrap_err().code, "datei-kein-utf8");
        assert_eq!(lesen(&r, "../x").unwrap_err().code, "pfad-ungueltig");
        assert_eq!(lesen(&r, "fehlt.asm").unwrap_err().code, "datei-fehler");
        let _ = std::fs::remove_dir_all(&r);
    }
    #[test] fn faerbung_zaehlt_in_utf16_und_markiert_arten() {
        let v = faerbung("start: lda #$00 ; Ärger\n  .text \"hällo 😀\"", "asm");
        let z0 = &v[0][1]; assert_eq!(z0[0][2], "label"); assert!(z0.as_array().unwrap().iter().any(|t| t[2] == "mnemonic")); assert!(z0.as_array().unwrap().iter().any(|t| t[2] == "comment"));
        let z1 = &v[1][1]; let s = z1.as_array().unwrap().iter().find(|t| t[2] == "string").unwrap(); assert_eq!(s[1].as_u64().unwrap() - s[0].as_u64().unwrap(), 10, "Emoji zählt als zwei Einheiten");
        assert_eq!(faerbung("x", "plain"), json!([]));
    }
    #[test] fn mnemonics_kommen_aus_den_cpu_tabellen() {
        let m = mnemonics();
        for w in ["lda", "rts", "bra", "stz", "inz", "lbra", "ldq", "bbr0", "rmb7", "slo", "lax"] { assert!(m.contains(w), "{w} fehlt (Stand: {} Namen)", m.len()); }
        assert!(!m.contains("hallo")); assert!(m.len() > 56);
    }
    #[test] fn formatieren_aendert_nur_geaenderte_zeilen_und_ist_idempotent() {
        let t = "start:   lda   #1    ; wert\nrts\n; Kommentar";
        let v = formatieren(t, "a.asm", "asm", 0, 2);
        assert_eq!(v.as_array().unwrap().len(), 2, "{v}");
        assert_eq!(v[1][0], 1); assert_eq!(v[1][1], "        rts");
        let neu: Vec<String> = t.split('\n').enumerate().map(|(i, z)| v.as_array().unwrap().iter().find(|e| e[0] == i).map_or(z.to_string(), |e| e[1].as_str().unwrap().to_string())).collect();
        assert_eq!(formatieren(&neu.join("\n"), "a.asm", "asm", 0, 2), json!([]), "zweites Mal nichts");
        assert_eq!(formatieren(t, "a.asm", "asm", 1, 1).as_array().unwrap().len(), 1, "nur die gewünschte Zeile");
        assert_eq!(formatieren("a    b", "a.txt", "plain", 0, 0), json!([]));
    }
    #[test] fn formatieren_basic_nach_endung_und_blocktiefe() {
        assert_eq!(formatieren("10   print   \"a  b\"", "x.bas", "basic", 0, 0)[0][1], "10 print \"a  b\"");
        let v = formatieren("for i = 1 to 3\nprint i\nnext", "x.sbas", "basic", 0, 2);
        assert_eq!(v[0][1], "  print i", "Einrückung folgt der Blocktiefe: {v}");
    }
    #[test] fn karte_zeigt_zeichenangabe_und_basic_register() {
        let Some(data) = crate::lsp::daten_ordner() else { return };
        let k = karte(&data, "c64", "cbm_v2", "a.asm", "asm", ".text \"x{clr}y\"", 0, 10);
        assert!(k["markdown"].as_str().unwrap_or("").contains("clr"), "{k}"); assert_eq!(k["from"], 8);
        assert_eq!(karte(&data, "c64", "cbm_v2", "a.asm", "asm", "lda #1", 0, 1), Value::Null);
        let b = karte(&data, "c64", "cbm_v2", "a.bas", "basic", "10 poke 53280,0", 0, 10);
        assert!(b["markdown"].as_str().unwrap_or("").contains("D020") || b["markdown"].as_str().unwrap_or("").contains("d020"), "{b}");
        let n = zeichennamen(&data, "cbm_v2", "asm", ".text \"x{c", 0 + 11);
        assert!(n.as_array().unwrap().iter().any(|e| e[0] == "clr"), "{n}");
        assert_eq!(zeichennamen(&data, "cbm_v2", "asm", "lda {c", 5), json!([]), "außerhalb einer Zeichenkette keine Namen");
    }
    #[test] fn faltung_findet_makro_block() { let v = faltung(".macro m\n nop\n.endmacro\n", "asm", true); assert_eq!(v, json!([[0, 2]])); }
}
