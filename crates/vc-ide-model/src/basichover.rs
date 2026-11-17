// SPDX-License-Identifier: GPL-3.0-or-later
//! Hover im nativen BASIC (`.bas`, `.lst`, `.tur`) **ohne Sprachdienst und ohne Oberfläche**: Zahlen in allen Darstellungen (Zahlenpass),
//! das Chipregister zur Adresse (Name, Zugriff, Text, Bitfelder) und die **Zerlegung des Wertes** hinter `poke` bzw. `peek(a) or m`.
//! Konzept `Konzept-POKE-Designer.md`, Stufe 0 und 7.3. Die Karte gehört zur Stelle unter dem **Mauszeiger** (der Aufrufer liefert Zeile
//! und Spalte aus der Mausposition, nie vom Textcursor).
use crate::editor_names::{chips, name_dialect, scan_basic, select_chips, select_name_dialect, NameDialect};
use std::path::Path;
use vc_ide::chipmap::{RegInfo, BitInfo};

/// Hover-Karte: Zeichenbereich der Stelle (Spaltenindizes, Ende ausgeschlossen) und Markdown
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card { pub start: usize, pub end: usize, pub md: String }

/// Zugriff im Klartext (nie nur das Kürzel)
pub fn access_text(a: &str) -> &'static str {
    match a { "rw" => "lesen und schreiben (rw)", "r" => "nur lesen (r)", "w" => "nur schreiben (w)", _ => "Zugriff unbekannt" }
}

/// Wert eines Feldes `lo..lo+width` aus `v`
pub fn field_value(b: &BitInfo, v: u32) -> u32 { (v >> b.lo) & ((1u32 << b.width.min(8)) - 1) }
fn field_mask(b: &BitInfo) -> u32 { (((1u32 << b.width.min(8)) - 1) << b.lo) & 0xFF }
fn bits_text(b: &BitInfo) -> String { if b.width <= 1 { format!("{}", b.lo) } else { format!("{}–{}", b.lo + b.width - 1, b.lo) } }

// ---------------- kleiner Ausdrucksauswerter (nur Konstantes) ----------------

fn parse_num(s: &str) -> Option<i64> {
    let s: String = s.chars().filter(|c| *c != '_').collect();
    if let Some(h) = s.strip_prefix('$') { return i64::from_str_radix(h, 16).ok(); }
    if let Some(b) = s.strip_prefix('%') { return i64::from_str_radix(b, 2).ok(); }
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) { return s.parse().ok(); }
    None
}

/// Summe/Differenz aus Zahlen und Variablen, die weiter oben als `name = konstante` stehen
fn eval(text: &str, line: usize, expr: &str, depth: u8) -> Option<i64> {
    let ch: Vec<char> = expr.chars().collect();
    let (mut total, mut sign, mut i, mut any) = (0i64, 1i64, 0usize, false);
    while i < ch.len() {
        let c = ch[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == '+' { sign = 1; i += 1; continue; }
        if c == '-' { sign = -1; i += 1; continue; }
        let st = i;
        while i < ch.len() && (ch[i].is_ascii_alphanumeric() || matches!(ch[i], '$' | '%' | '_' | '.')) { i += 1; }
        if st == i { return None; }
        let tok: String = ch[st..i].iter().collect();
        let v = match parse_num(&tok) {
            Some(v) => v,
            None => { if depth > 2 { return None; } resolve_var(text, line, &tok, depth + 1)? }
        };
        total += sign * v; sign = 1; any = true;
    }
    any.then_some(total)
}

/// Letzte Zuweisung `name = konstante` vor Zeile `line` (nur geradlinig, kein Sprung-/Schleifenmodell)
fn resolve_var(text: &str, line: usize, name: &str, depth: u8) -> Option<i64> {
    let lname = name.to_ascii_lowercase();
    if lname.chars().next()?.is_ascii_digit() { return None; }
    let lines: Vec<&str> = text.lines().collect();
    for l in (0..line.min(lines.len())).rev() {
        for stmt in split_statements(lines[l]).into_iter().rev() {
            let s = strip_line_number(stmt.2.trim()).trim();
            let s = s.strip_prefix("let ").or_else(|| s.strip_prefix("LET ")).unwrap_or(s);
            let Some((lhs, rhs)) = s.split_once('=') else { continue };
            if lhs.trim().eq_ignore_ascii_case(&lname) { return eval(text, l, rhs, depth); }
        }
    }
    None
}

fn strip_line_number(s: &str) -> &str { s.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start() }

/// Anweisungen einer Zeile (an `:` außerhalb von Anführungszeichen): (Anfang, Ende, Text), Zeichenindizes
fn split_statements(line: &str) -> Vec<(usize, usize, String)> {
    let ch: Vec<char> = line.chars().collect();
    let (mut out, mut st, mut q) = (vec![], 0usize, false);
    for (i, c) in ch.iter().enumerate() {
        if *c == '"' { q = !q }
        if *c == ':' && !q { out.push((st, i, ch[st..i].iter().collect())); st = i + 1; }
    }
    out.push((st, ch.len(), ch[st..].iter().collect()));
    out
}

/// `poke a, w` in einer Anweisung: Zeichenbereiche von Adresse und Wert (ohne Randleerzeichen)
struct PokeStmt { a: (usize, usize), w: (usize, usize) }

fn find_poke(line: &str, stmt: (usize, usize)) -> Option<PokeStmt> {
    let ch: Vec<char> = line.chars().collect();
    let mut i = stmt.0;
    while i < stmt.1 && (ch[i].is_whitespace() || ch[i].is_ascii_digit()) { i += 1; }
    // Zeilennummer kann nur am Zeilenanfang stehen; ein Stück Ziffern direkt vor `poke` ist dort die Zeilennummer
    if stmt.1 - i < 4 || !ch[i..i + 4].iter().collect::<String>().eq_ignore_ascii_case("poke") { return None; }
    i += 4;
    let (mut depth, mut q, mut comma) = (0i32, false, None);
    for j in i..stmt.1 {
        match ch[j] { '"' => q = !q, '(' if !q => depth += 1, ')' if !q => depth -= 1, ',' if !q && depth == 0 => { comma = Some(j); break } _ => {} }
    }
    let comma = comma?;
    let trim = |mut s: usize, mut e: usize| { while s < e && ch[s].is_whitespace() { s += 1 } while e > s && ch[e - 1].is_whitespace() { e -= 1 } (s, e) };
    Some(PokeStmt { a: trim(i, comma), w: trim(comma + 1, stmt.1) })
}

/// Wert hinter `poke`: absolut oder `peek(a) or m` / `peek(a) and m`
enum Val { Abs(i64), Or(i64), And(i64) }

fn eval_value(text: &str, line: usize, w: &str) -> Option<Val> {
    let lw = w.trim();
    let low = lw.to_ascii_lowercase();
    if low.starts_with("peek(") {
        let close = lw.find(')')?;
        let rest = lw[close + 1..].trim();
        let r = rest.to_ascii_lowercase();
        if let Some(m) = r.strip_prefix("or") { return eval(text, line, &rest[rest.len() - m.len()..], 0).map(Val::Or); }
        if let Some(m) = r.strip_prefix("and") { return eval(text, line, &rest[rest.len() - m.len()..], 0).map(Val::And); }
        return None;
    }
    eval(text, line, lw, 0).map(Val::Abs)
}

// ---------------- Karten ----------------

fn register_block(r: &RegInfo, show_bits: bool) -> String {
    let doc = if r.doc.is_empty() { String::new() } else { format!(" – {}", r.doc) };
    let mut md = format!("**Register `{}`** (`${:04X}`), {}{}\n", r.label(), r.addr, access_text(&r.access), doc);
    if show_bits && !r.bits.is_empty() {
        md.push_str("\n| Feld | Bits | Bedeutung |\n|---|---|---|\n");
        for b in &r.bits { md.push_str(&format!("| {} | {} | {} |\n", b.name, bits_text(b), b.doc)); }
    }
    md
}

fn decomposition(r: &RegInfo, v: u32, kind: &Val) -> String {
    if r.bits.is_empty() { return String::new(); }
    let mut md = String::from("\n| Feld | Bits | Wert | Bedeutung |\n|---|---|---|---|\n");
    let mut changed: Vec<String> = vec![];
    for b in &r.bits {
        let m = field_mask(b);
        let involved = match kind { Val::Abs(_) => true, Val::Or(_) => v & m != 0, Val::And(_) => v & m != m };
        if !involved { continue; }
        let fv = field_value(b, v);
        md.push_str(&format!("| {} | {} | {} | {} |\n", b.name, bits_text(b), fv, b.doc));
        changed.push(if b.width <= 1 { format!("{}={}", b.name, fv) } else { format!("{}={}", b.name, fv) });
    }
    if changed.is_empty() { return String::new(); }
    let lead = match kind { Val::Abs(_) => format!("Dieser Wert ({v}) setzt: "), Val::Or(_) => format!("`or {v}` setzt Bits der Felder: "), Val::And(_) => format!("`and {v}` löscht Bits der Felder: ") };
    md.push_str(&format!("\n{}{}.\n", lead, changed.join(", ")));
    md
}

/// Karte für die Stelle (`line`, `col`: Zeilen- und Zeichenindex, null-basiert) im BASIC-Text; `None`, wo es nichts zu sagen gibt
/// (Kommentar, Zeichenkette, Namen, Schlüsselwörter). `data`, `target` (z. B. `c64`) und `dialect` (z. B. `cbm_v2`) bestimmen
/// Zahlenpass und Chipregister.
pub fn hover(data: &Path, target: &str, dialect: &str, text: &str, line: usize, col: usize) -> Option<Card> {
    select_name_dialect(data, dialect);
    select_chips(data, target);
    let nd = name_dialect()?;
    let map = chips();
    let line_text = text.lines().nth(line)?;
    let t = line_text.trim_start();
    if t.starts_with(';') { return None; }
    let long = vc_basic::longnames::has_directive(text);
    let sc = scan_basic(&nd, line_text, long);
    let ch: Vec<char> = line_text.chars().collect();
    let hit = |s: usize, e: usize| col >= s && col < e;
    let slice = |s: usize, e: usize| -> String { ch[s.min(ch.len())..e.min(ch.len())].iter().collect() };
    let o = vc_basic::numbers::NumberOptions { whole_words: long, ..vc_basic::numbers::defaults() };
    let lookup = |a: i64| -> Option<&RegInfo> { if (0..=0xFFFF).contains(&a) { map.as_deref().and_then(|m| m.lookup(a as u32)) } else { None } };
    // Zeile mit POKE?
    let poke = split_statements(line_text).into_iter().find_map(|(s, e, _)| find_poke(line_text, (s, e)));

    // 1. Stelle liegt in der Adresse eines POKE (Zahl oder `v + 21`)
    if let Some(p) = &poke {
        if hit(p.a.0, p.a.1) {
            let expr = slice(p.a.0, p.a.1);
            let addr = eval(text, line, &expr, 0)?;
            let mut md = if parse_num(expr.trim()).is_some() { vc_lsp::value_hover(expr.trim(), addr) } else { format!("**{}** = `{}` = `${:04X}`\n", expr.trim(), addr, addr.rem_euclid(0x10000)) };
            if (0..=0xFFFF).contains(&addr) { if let Some(r) = lookup(addr) { md.push('\n'); md.push_str(&register_block(r, true)); } }
            if let Some(n) = sc.notes.iter().find(|n| hit(n.start, n.end)) { md.push_str(&format!("\n{}: {}\n", n.code, n.message)); }
            return Some(Card { start: p.a.0, end: p.a.1, md: md });
        }
        // 2. Stelle liegt im Wert eines POKE
        if hit(p.w.0, p.w.1) {
            let addr = eval(text, line, &slice(p.a.0, p.a.1), 0);
            let wtxt = slice(p.w.0, p.w.1);
            let kind = eval_value(text, line, &wtxt);
            let (v, label_span) = match &kind { Some(Val::Abs(v)) | Some(Val::Or(v)) | Some(Val::And(v)) => (*v, p.w), None => return number_card(&sc, &ch, col, &o, &nd, None) };
            let mut md = String::new();
            let num_span = sc.nums.iter().find(|n| hit(n.start, n.end)).map(|n| (n.start, n.end));
            md.push_str(&vc_lsp::value_hover(&match num_span { Some((s, e)) => slice(s, e), None => wtxt.clone() }, v));
            if !(0..=255).contains(&v) { md.push_str("\n`POKE` verlangt 0–255 (sonst `?ILLEGAL QUANTITY ERROR`).\n"); }
            if let (Some(a), Some(k)) = (addr, &kind) {
                if let Some(r) = lookup(a) {
                    md.push('\n'); md.push_str(&register_block(r, false));
                    if (0..=255).contains(&v) { md.push_str(&decomposition(r, v as u32, k)); }
                }
            }
            let _ = label_span;
            let (s, e) = num_span.unwrap_or(p.w);
            return Some(Card { start: s, end: e, md });
        }
    }
    number_card(&sc, &ch, col, &o, &nd, map.as_deref())
}

/// Allgemeine Zahlenkarte (Zahlenpass) mit Registerzeile bei Adressen
fn number_card(sc: &vc_basic::numbers::Scan, ch: &[char], col: usize, o: &vc_basic::numbers::NumberOptions, nd: &NameDialect, map: Option<&vc_ide::chipmap::ChipMap>) -> Option<Card> {
    use vc_basic::numbers::{replacement, Lexicon};
    let hit = |s: usize, e: usize| col >= s && col < e;
    let slice = |s: usize, e: usize| -> String { ch[s.min(ch.len())..e.min(ch.len())].iter().collect() };
    if let Some(l) = sc.lits.iter().find(|l| hit(l.start, l.end)) {
        let tx = slice(l.start, l.end);
        let notes: Vec<&vc_basic::numbers::Note> = sc.notes.iter().filter(|n| hit(n.start, n.end) || (n.start >= l.start && n.end <= l.end)).collect();
        let Some(v) = l.value else {
            let mut md = format!("**{tx}** – ungültige Zahl\n\n");
            for n in &notes { md.push_str(&format!("- {}: {}\n", n.code, n.message)); }
            return Some(Card { start: l.start, end: l.end, md });
        };
        let mut md = vc_lsp::value_hover(&tx, v as i64);
        let bits = if l.radix == 16 { l.digits * 4 } else { l.digits };
        md.push_str(&format!("| Breite | {bits} Bit ({} Ziffern) |\n", l.digits));
        if let (Some(m), Some(n)) = (map, sc.nums.iter().find(|n| n.start == l.start)) {
            if n.role == vc_basic::numbers::Role::Address { if let Some(r) = m.lookup(n.value) { md.push('\n'); md.push_str(&register_block(r, true)); } }
        }
        let new = match nd { NameDialect::Cbm(d) => replacement(l, &Lexicon::Cbm(d), o), NameDialect::Atari(d) => replacement(l, &Lexicon::Atari(d), o) };
        if let Some(n) = new { if n == l.text { md.push_str(&format!("\n| Erzeugt | `{n}` (der Dialekt liest es selbst) |\n")); } else { md.push_str(&format!("\n| Erzeugt | `{n}` (Dezimalzahl) |\n")); } }
        if l.in_data { md.push_str("\n`DATA`-Element: wird beim Bauen umgewandelt (Text in Anführungszeichen bleibt).\n"); }
        for n in notes.iter().filter(|n| !n.error) { md.push_str(&format!("\n{}: {}\n", n.code, n.message)); }
        return Some(Card { start: l.start, end: l.end, md });
    }
    if let Some(n) = sc.nums.iter().find(|n| n.radix == 10 && hit(n.start, n.end)) {
        let tx = slice(n.start, n.end);
        let mut md = vc_lsp::value_hover(&tx, n.value as i64);
        if n.role == vc_basic::numbers::Role::Address { if let Some(r) = map.and_then(|m| m.lookup(n.value)) { md.push('\n'); md.push_str(&register_block(r, true)); } }
        return Some(Card { start: n.start, end: n.end, md });
    }
    let n = sc.notes.iter().find(|n| hit(n.start, n.end))?;
    Some(Card { start: n.start, end: n.end, md: format!("**{}** {}: {}", n.code, if n.error { "Fehler" } else { "Hinweis" }, n.message) })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
    fn h(text: &str, line: usize, col: usize) -> Option<Card> { hover(&data(), "c64", "cbm_v2", text, line, col) }

    #[test]
    fn address_in_poke_shows_register_and_bit_fields() {
        let c = h("poke 53265, 59\n", 0, 7).unwrap();
        assert_eq!((c.start, c.end), (5, 10));
        for s in ["vic2.ctrl1", "$D011", "lesen und schreiben (rw)", "| BMM | 5 | Bitmap-Modus |", "YSCROLL"] { assert!(c.md.contains(s), "{s}: {}", c.md); }
    }

    #[test]
    fn value_in_poke_is_decomposed_into_fields() {
        let c = h("poke 53265, 59\n", 0, 13).unwrap();
        assert_eq!((c.start, c.end), (12, 14));
        for s in ["| BMM | 5 | 1 | Bitmap-Modus |", "| DEN | 4 | 1 |", "| YSCROLL | 2–0 | 3 |", "| ECM | 6 | 0 |", "setzt:"] { assert!(c.md.contains(s), "{s}: {}", c.md); }
    }

    #[test]
    fn or_mask_lists_only_the_touched_fields() {
        let c = h("poke 53265, peek(53265) or 32\n", 0, 28).unwrap();
        assert!(c.md.contains("| BMM | 5 | 1 |") && !c.md.contains("| DEN |") && c.md.contains("`or 32` setzt"), "{}", c.md);
        let c = h("poke 53265, peek(53265) and 223\n", 0, 29).unwrap();
        assert!(c.md.contains("`and 223` löscht") && c.md.contains("| BMM | 5 | 0 |") && !c.md.contains("| DEN |"), "{}", c.md);
    }

    #[test]
    fn base_variable_is_resolved_when_it_stands_above() {
        let t = "v = 53248\npoke v + 21, 7\n";
        let c = h(t, 1, 8).unwrap();
        assert!(c.md.contains("53269") && c.md.contains("$D015") && c.md.contains("vic2.spena"), "{}", c.md);
        // unbekannte Variable: keine Karte (nie raten)
        assert!(h("poke q + 21, 7\n", 0, 7).is_none());
    }

    #[test]
    fn hex_and_binary_literals_show_the_register_too() {
        let c = h("poke $d020, %0110\n", 0, 7).unwrap();
        assert!(c.md.contains("vic2.border") && c.md.contains("Rahmenfarbe"), "{}", c.md);
        let c = h("poke $d020, %0110\n", 0, 14).unwrap();
        assert!(c.md.contains("`%0110`") || c.md.contains("%0110"), "{}", c.md);
    }

    #[test]
    fn nothing_in_comments_strings_or_names() {
        assert!(h("; poke 53265, 59\n", 0, 8).is_none());
        assert!(h("print \"53265\"\n", 0, 8).is_none());
        assert!(h("poke 53265, 59\n", 0, 2).is_none());
    }

    #[test]
    fn plain_peek_address_shows_register() {
        let c = h("x = peek(53280)\n", 0, 11).unwrap();
        assert!(c.md.contains("vic2.border"), "{}", c.md);
    }

    #[test]
    fn value_out_of_range_warns() {
        let c = h("poke 53280, 300\n", 0, 13).unwrap();
        assert!(c.md.contains("0–255"), "{}", c.md);
    }

    #[test]
    fn line_numbers_do_not_confuse_the_poke_parser() {
        let c = h("10 poke 53265, 59\n", 0, 11).unwrap();
        assert!(c.md.contains("vic2.ctrl1"), "{}", c.md);
        let c = h("10 poke 53265, 59 : poke 53272, 21\n", 0, 33).unwrap();
        assert!(c.md.contains("21"), "{}", c.md);
    }
}
