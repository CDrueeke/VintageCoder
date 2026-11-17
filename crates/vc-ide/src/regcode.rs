// SPDX-License-Identifier: GPL-3.0-or-later
//! Codegenerator für Registerwerte als **BASIC-Zeilen** (Konzept `Konzept-POKE-Designer.md`, Abschnitt 5): reine Funktion ohne
//! Oberfläche, ohne Dateizugriff beim Erzeugen. Eingabe: Wertmodell ([`RegWrite`]) + Dialekt ([`Dialect`]) + Optionen
//! ([`Options`]); Ausgabe: Zeilen, Warnungen, gesperrte Register. Die Ausgabe ist natives BASIC (`.bas`): Kommentare mit `;`
//! (kostenlos) oder `rem`, Zeilennummern nur auf Wunsch. Hex und Binär sind nur eine Schreibweise; der Zahlenpass des Übersetzers macht
//! beim Bauen Dezimalzahlen daraus.
//!
//! Regeln (Auszug): Dezimal als Standard; das **ganze Byte bestimmt** oder ein **Schreibregister** → Absolutwert; sonst bei lesbarem
//! Register `PEEK … OR/AND` (CBM), `! / &` (Turbo-BASIC XL) bzw. Arithmetik (Atari BASIC, kein bitweises AND/OR); Atari schreibt
//! standardmäßig den **Schatten** des Betriebssystems; Variablen für Basisadressen ab einer Schwelle; mehrere `POKE` je Zeile bis zur
//! Zeilenlänge; gesperrte Register nur mit Bestätigung.
use crate::chipmap::{ChipMap, RegInfo};
use std::path::Path;

/// Zugriff eines Registers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access { R, W, Rw }
impl Access {
    pub fn parse(s: &str) -> Access { match s { "r" => Access::R, "w" => Access::W, _ => Access::Rw } }
    fn readable(self) -> bool { self != Access::W }
}

/// Ein Registerwert des Modells: welche Bits bestimmt sind (`mask`) und welchen Wert sie haben (`value`, nur unter `mask` zählt)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegWrite {
    /// `vic2.ctrl1`
    pub label: String,
    pub chip: String,
    pub addr: u16,
    /// Basisadresse des Chips für Variablen (`$D000` beim VIC-II)
    pub base: u16,
    pub value: u8,
    pub mask: u8,
    pub access: Access,
    /// Registertext (`Steuerregister 1`) für den Kommentar
    pub doc: String,
    /// geänderte Felder (`BMM`, 1) für den Kommentar
    pub fields: Vec<(String, u32)>,
    /// Atari: Schatten (Adresse, `os.color4`)
    pub shadow: Option<(u16, String)>,
    /// gesperrtes Register (MMU, ROM/RAM-Schalter): Grund; wird nur mit [`Options::allow_locked`] erzeugt
    pub lock: Option<String>,
    /// weitere Hinweise (z. B. Adresse im ROM-Bereich)
    pub notes: Vec<String>,
}

impl RegWrite {
    /// Alle Bits bestimmt, Wert `v` (Absolutwert)
    pub fn full(info: &RegInfo, base: u16, v: u8) -> RegWrite {
        RegWrite { label: info.label(), chip: info.chip.clone(), addr: info.addr, base, value: v, mask: 0xFF, access: Access::parse(&info.access), doc: info.doc.clone(), fields: vec![], shadow: None, lock: None, notes: vec![] }
    }
    /// Nur die genannten Felder (Name, Wert) bestimmt; unbekannte Namen sind ein Fehler
    pub fn fields(info: &RegInfo, base: u16, assign: &[(&str, u32)]) -> Result<RegWrite, String> {
        let (mut value, mut mask, mut fl) = (0u32, 0u32, vec![]);
        for (n, v) in assign {
            let b = info.bits.iter().find(|b| b.name.eq_ignore_ascii_case(n)).ok_or_else(|| format!("{}: kein Feld {n}", info.label()))?;
            let w = b.width.clamp(1, 8) as u32;
            if *v >> w != 0 { return Err(format!("{}: Wert {v} passt nicht in {n} ({w} Bit)", info.label())); }
            let m = ((1u32 << w) - 1) << b.lo;
            mask |= m; value = (value & !m) | (v << b.lo); fl.push((b.name.clone(), *v));
        }
        let mut r = RegWrite::full(info, base, value as u8);
        r.mask = mask as u8; r.fields = fl;
        Ok(r)
    }
    /// Schatten dieses Hardwareregisters aus den Daten (`shadow` am Register) nachtragen
    pub fn with_shadow(mut self, map: &ChipMap, info: &RegInfo) -> RegWrite {
        if let Some(s) = info.shadow.as_deref().and_then(|l| map.by_label(l)) { self.shadow = Some((s.addr, s.label())); }
        self
    }
}

/// Die vier Bitoperationen, die ein Dialekt kennt
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bitwise { /// `AND`/`OR`
    Words, /// Turbo-BASIC XL `&`, `!`
    Symbols, /// Atari BASIC: nur Arithmetik
    None }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialect { pub id: String, pub atari: bool, pub bitwise: Bitwise, pub max_line_len: usize, pub sig_chars: usize }

impl Dialect {
    /// Aus `data/basic/<id>.dialect.json` (nur Lesen)
    pub fn load(data: &Path, id: &str) -> Result<Dialect, String> {
        let t = std::fs::read_to_string(data.join("basic").join(format!("{id}.dialect.json"))).map_err(|e| format!("Dialekt {id}: {e}"))?;
        let v: serde_json::Value = serde_json::from_str(&t).map_err(|e| format!("Dialekt {id}: {e}"))?;
        let atari = v["family"].as_str() == Some("atari");
        let bitwise = if !atari { Bitwise::Words } else if id == "turbo_basic_xl" { Bitwise::Symbols } else { Bitwise::None };
        Ok(Dialect { id: id.into(), atari, bitwise, max_line_len: v["maxLineLen"].as_u64().unwrap_or(80) as usize, sig_chars: v["varSigChars"].as_u64().unwrap_or(2) as usize })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comment { Off, Semicolon, Rem }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Numbers { Decimal, Hex }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vars { Never, FromLines(usize), Always }

#[derive(Clone, Debug)]
pub struct Options {
    pub comments: Comment,
    pub numbers: Numbers,
    pub vars: Vars,
    /// mehrere `POKE` je Zeile (bis `max_len`)
    pub per_line: bool,
    /// Höchstlänge einer Zeile (Standard 80, nie mehr als der Dialekt erlaubt)
    pub max_len: usize,
    /// andere Bits erhalten (`PEEK … OR/AND`) statt Absolutwert, wo das Register lesbar ist
    pub keep_others: bool,
    /// `; vc:poke <name> begin` … `; vc:poke end` um den Block
    pub markers: bool,
    pub block_name: String,
    /// Zeilennummern (Start, Schritt); dann sind Kommentare immer `rem`
    pub line_numbers: Option<(u32, u32)>,
    /// `#longnames` in der Datei: lesbare Variablennamen
    pub long_names: bool,
    /// bereits benutzte Namen der Datei (Variablen), nie doppelt vergeben
    pub taken: Vec<String>,
    pub allow_locked: bool,
    /// Atari: Schatten schreiben (Standard) statt Hardwareregister
    pub atari_shadow: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options { comments: Comment::Semicolon, numbers: Numbers::Decimal, vars: Vars::FromLines(5), per_line: true, max_len: 80, keep_others: true, markers: true, block_name: "regs".into(), line_numbers: None, long_names: false, taken: vec![], allow_locked: false, atari_shadow: true }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Generated { pub lines: Vec<String>, pub warnings: Vec<String>, /// nicht erzeugt: (Register, Grund)
    pub locked: Vec<(String, String)> }

impl Generated { pub fn text(&self) -> String { let mut s = self.lines.join("\n"); s.push('\n'); s } }

/// Wörter, die in kurzen CBM-Namen nicht vorkommen dürfen (Schlüsselwörter und reservierte Namen, `crates/vc-basic/src/longnames.rs`)
const BAD_IN_SHORT: &[&str] = &["TO", "OR", "IF", "GO", "FN", "ON", "ST", "TI", "DS", "ER", "EL", "AS", "AT"];

fn short_default(chip: &str) -> &'static str {
    match chip { "vic2" | "vic1" | "vic4" | "vic2e" => "v", "sid" | "sid2" | "sid3" | "sid4" => "s", "cia1" => "c1", "cia2" => "c2", "ted" => "t", "antic" => "a", "gtia" => "g", "pokey" => "k", _ => "r" }
}

fn name_ok(n: &str, d: &Dialect, taken: &[String]) -> bool {
    let sig: String = n.chars().take(d.sig_chars.max(1)).collect::<String>().to_ascii_uppercase();
    if taken.iter().any(|t| t.chars().take(d.sig_chars.max(1)).collect::<String>().eq_ignore_ascii_case(&sig)) { return false; }
    if !d.atari && BAD_IN_SHORT.iter().any(|b| sig.contains(b)) { return false; }
    true
}

fn pick_name(preferred_short: &str, chip: &str, d: &Dialect, o: &Options, taken: &mut Vec<String>) -> String {
    let mut cands: Vec<String> = vec![];
    if o.long_names || d.atari { cands.push(chip.to_string()); }
    cands.push(preferred_short.to_string());
    for i in 1..=9 { cands.push(format!("{}{}", &preferred_short[..1], i)); }
    for c in 'a'..='z' { cands.push(format!("{}{}", &preferred_short[..1], c)); }
    let n = cands.into_iter().find(|c| name_ok(c, d, taken)).unwrap_or_else(|| preferred_short.to_string());
    taken.push(n.clone());
    n
}

fn num(v: u32, hex: bool, digits: usize) -> String { if hex { format!("${:0w$X}", v, w = digits) } else { v.to_string() } }

/// Eine Zeile Absolut-/Bitform für ein Register; `a` ist der Adressausdruck (Zahl oder `v + 17`)
fn cbm_forms(w: &RegWrite, a: &str, hexn: bool, d: &Dialect, o: &Options, warn: &mut Vec<String>, readable: bool) -> String {
    let n8 = |v: u32| num(v, hexn, 2);
    let set = (w.value & w.mask) as u32;
    let clr = (w.mask & !w.value) as u32;
    let partial = w.mask != 0xFF;
    if !partial || !readable || !o.keep_others {
        if partial && !readable { warn.push(format!("{}: Schreibregister, der nicht gelesene Rest wird 0 gesetzt", w.label)); }
        return format!("poke {a}, {}", n8(set));
    }
    // lesbar, nur einzelne Bits bestimmt
    let (or_, and_) = match d.bitwise { Bitwise::Symbols => ("!", "&"), _ => ("or", "and") };
    let keep = 255 - w.mask as u32;
    if clr == 0 { format!("poke {a}, peek({a}) {or_} {}", n8(set)) }
    else if set == 0 { format!("poke {a}, peek({a}) {and_} {}", n8(255 - clr)) }
    else { format!("poke {a}, (peek({a}) {and_} {}) {or_} {}", n8(keep), n8(set)) }
}

/// Atari BASIC: Arithmetik statt AND/OR; `p` ist die Hilfsvariable
fn atari_arith(w: &RegWrite, a: &str, p: &str, hexn: bool) -> Vec<String> {
    let n = |v: u32| num(v, hexn, 2);
    let mut out = vec![format!("{p} = peek({a})")];
    // zusammenhängende Läufe bestimmter Bits
    let mut b = 0u32;
    while b < 8 {
        if w.mask >> b & 1 == 0 { b += 1; continue; }
        let mut e = b; while e < 8 && w.mask >> e & 1 == 1 { e += 1; }
        let width = e - b;
        let v = ((w.value as u32) >> b) & ((1 << width) - 1);
        let f = 1u32 << b;
        if width == 1 {
            let test = if f == 1 { format!("{p} - 2 * int({p} / 2)") } else { format!("int({p} / {}) - 2 * int({p} / {})", n(f), n(2 * f)) };
            if v == 1 { out.push(format!("if {test} = 0 then {p} = {p} + {}", n(f))); } else { out.push(format!("if {test} = 1 then {p} = {p} - {}", n(f))); }
        } else {
            let m = 1u32 << width;
            if f == 1 { out.push(format!("{p} = {p} + {v} - ({p} - {} * int({p} / {}))", n(m), n(m))); }
            else {
                let cur = format!("int({p} / {}) - {} * int({p} / {})", n(f), n(m), n(f * m));
                out.push(format!("{p} = {p} + ({v} - ({cur})) * {}", n(f)));
            }
        }
        b = e;
    }
    out.push(format!("poke {a}, {p}"));
    out
}

fn comment_line(c: Comment, numbered: bool, text: &str) -> Option<String> {
    match (c, numbered) { (Comment::Off, _) => None, (Comment::Semicolon, false) => Some(format!("; {text}")), _ => Some(format!("rem {text}")) }
}

/// BASIC-Zeilen für die Registerwerte `ws` (in der gegebenen Reihenfolge)
pub fn generate(ws: &[RegWrite], d: &Dialect, o: &Options) -> Generated {
    let mut g = Generated::default();
    let hexn = o.numbers == Numbers::Hex;
    let numbered = o.line_numbers.is_some();
    let mut taken = o.taken.clone();
    let max_len = o.max_len.min(d.max_line_len).max(20);
    // gesperrte und leere Schreibvorgänge aussortieren
    let mut active: Vec<&RegWrite> = vec![];
    for w in ws {
        if let Some(why) = &w.lock { if !o.allow_locked { g.locked.push((w.label.clone(), why.clone())); continue; } g.warnings.push(format!("{}: gesperrtes Register ({why}), nur nach Bestätigung erzeugt", w.label)); }
        if w.mask == 0 { continue; }
        for n in &w.notes { g.warnings.push(format!("{}: {n}", w.label)); }
        active.push(w);
    }
    if active.is_empty() { return g; }
    // Schatten (Atari)
    let target = |w: &RegWrite| -> (u16, bool, Option<String>) {
        match (&w.shadow, o.atari_shadow && d.atari) { (Some((a, l)), true) => (*a, true, Some(l.clone())), _ => (w.addr, false, None) }
    };
    // Variablen je Chip
    let mut var_of: Vec<(String, String, u16)> = vec![];   // (Chip, Name, Basis)
    if !d.atari || d.bitwise != Bitwise::None || true {
        let chips: Vec<String> = { let mut c: Vec<String> = vec![]; for w in &active { if target(w).1 { continue; } if !c.contains(&w.chip) { c.push(w.chip.clone()); } } c };
        for chip in chips {
            let count = active.iter().filter(|w| w.chip == chip && !target(w).1).count();
            let use_var = match o.vars { Vars::Never => false, Vars::Always => true, Vars::FromLines(n) => count >= n };
            if use_var {
                let base = active.iter().find(|w| w.chip == chip).map(|w| w.base).unwrap_or(0);
                let name = pick_name(short_default(&chip), &chip, d, o, &mut taken);
                var_of.push((chip, name, base));
            }
        }
    }
    let mut head: Vec<String> = vec![];    // Kommentare
    let mut defs: Vec<String> = vec![];    // v = 53248
    let mut stmts: Vec<Vec<String>> = vec![]; // je Register: eine oder mehrere Anweisungen
    let mut atari_p: Option<String> = None;
    for w in &active {
        let (addr, via_shadow, shadow_label) = target(w);
        let a = match var_of.iter().find(|v| v.0 == w.chip && !via_shadow) {
            Some((_, name, base)) => { let off = addr - base; if off == 0 { name.clone() } else { format!("{name} + {}", num(off as u32, hexn, 2)) } }
            None => num(addr as u32, hexn, 4),
        };
        // Kommentar
        let mut c = format!("${:04X} {}", w.addr, if w.doc.is_empty() { w.label.clone() } else { w.doc.clone() });
        if !w.fields.is_empty() { c.push_str(": "); c.push_str(&w.fields.iter().map(|(n, v)| format!("{n}={v}")).collect::<Vec<_>>().join(" ")); }
        else if w.mask == 0xFF { c.push_str(&format!(" = {}", w.value)); }
        if let Some(l) = shadow_label { c.push_str(&format!(" (Schatten {l} ${addr:04X})")); }
        if let Some(l) = comment_line(o.comments, numbered, &c) { head.push(l); }
        let readable = w.access.readable() || via_shadow;
        if d.bitwise == Bitwise::None && w.mask != 0xFF && readable {
            let p = atari_p.get_or_insert_with(|| pick_name("p", "p", d, o, &mut taken)).clone();
            stmts.push(atari_arith(w, &a, &p, hexn));
        } else {
            stmts.push(vec![cbm_forms(w, &a, hexn, d, o, &mut g.warnings, readable)]);
        }
    }
    for (_, n, b) in &var_of { defs.push(format!("{n} = {}", num(*b as u32, hexn, 4))); }
    // Zeilen zusammensetzen
    let mut code: Vec<String> = defs;
    let prefix = |i: usize| -> usize { o.line_numbers.map_or(0, |(s, st)| (s + st * i as u32).to_string().len() + 1) };
    let mut cur: Option<String> = None;
    for group in stmts {
        if group.len() > 1 { if let Some(c) = cur.take() { code.push(c); } code.extend(group); continue; }
        let s = group.into_iter().next().unwrap();
        match cur.take() {
            Some(c) if o.per_line && c.len() + 3 + s.len() + prefix(code.len()) <= max_len => cur = Some(format!("{c} : {s}")),
            Some(c) => { code.push(c); cur = Some(s); }
            None => cur = Some(s),
        }
    }
    if let Some(c) = cur { code.push(c); }
    for l in &code { if l.len() + prefix(0) > d.max_line_len.max(max_len) { g.warnings.push(format!("Zeile länger als {} Zeichen: {l}", max_len)); } }
    // Rahmen und Nummern
    let mut all: Vec<String> = vec![];
    if o.markers { if let Some(l) = comment_line(if o.comments == Comment::Off { Comment::Semicolon } else { o.comments }, numbered, &format!("vc:poke {} begin", o.block_name)) { all.push(l); } }
    all.extend(head); all.extend(code);
    if o.markers { if let Some(l) = comment_line(if o.comments == Comment::Off { Comment::Semicolon } else { o.comments }, numbered, "vc:poke end") { all.push(l); } }
    g.lines = match o.line_numbers { Some((s, st)) => all.into_iter().enumerate().map(|(i, l)| format!("{} {l}", s + st * i as u32)).collect(), None => all };
    g
}

/// Hilfe für Aufrufer: Register `label` aus der Chipkarte mit Feldwerten; Basis und Schatten aus den Daten
pub fn write_from_map(map: &ChipMap, label: &str, assign: &[(&str, u32)], full: Option<u8>) -> Result<RegWrite, String> {
    let info = map.by_label(label).ok_or_else(|| format!("Register {label} unbekannt"))?;
    let base = map.chip_base(&info.chip).unwrap_or(info.addr);
    let w = match full { Some(v) => RegWrite::full(info, base, v), None => RegWrite::fields(info, base, assign)? };
    Ok(w.with_shadow(map, info))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> std::path::PathBuf { std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

    #[test]
    fn partial_mask_on_readable_register_uses_peek_or() {
        let m = ChipMap::load(&data(), "c64");
        let w = write_from_map(&m, "vic2.ctrl1", &[("BMM", 1)], None).unwrap();
        let g = generate(&[w], &Dialect::load(&data(), "cbm_v2").unwrap(), &Options { markers: false, comments: Comment::Off, ..Default::default() });
        assert_eq!(g.lines, vec!["poke 53265, peek(53265) or 32"]);
    }

    #[test]
    fn full_byte_is_absolute_and_both_directions_use_and_or() {
        let m = ChipMap::load(&data(), "c64");
        let o = Options { markers: false, comments: Comment::Off, ..Default::default() };
        let d = Dialect::load(&data(), "cbm_v2").unwrap();
        assert_eq!(generate(&[write_from_map(&m, "vic2.ctrl1", &[], Some(59)).unwrap()], &d, &o).lines, vec!["poke 53265, 59"]);
        let w = write_from_map(&m, "vic2.ctrl1", &[("BMM", 0), ("DEN", 1)], None).unwrap();
        assert_eq!(generate(&[w], &d, &o).lines, vec!["poke 53265, (peek(53265) and 207) or 16"]);
        let w = write_from_map(&m, "vic2.ctrl1", &[("BMM", 0)], None).unwrap();
        assert_eq!(generate(&[w], &d, &o).lines, vec!["poke 53265, peek(53265) and 223"]);
    }

    #[test]
    fn locked_registers_are_only_generated_with_confirmation() {
        let m = ChipMap::load(&data(), "c64");
        let mut w = write_from_map(&m, "vic2.ctrl1", &[], Some(1)).unwrap(); w.lock = Some("Bankumschaltung".into());
        let d = Dialect::load(&data(), "cbm_v2").unwrap();
        let g = generate(&[w.clone()], &d, &Options::default());
        assert!(g.lines.is_empty() && g.locked.len() == 1);
        let g = generate(&[w], &d, &Options { allow_locked: true, markers: false, comments: Comment::Off, ..Default::default() });
        assert_eq!(g.lines.len(), 1); assert!(g.warnings[0].contains("gesperrt"));
    }

    // ---- Arithmetik des Atari BASIC gegen eine Referenz: alle Masken und alle Werte ----
    struct Ev<'a> { s: Vec<char>, i: usize, p: f64, _m: std::marker::PhantomData<&'a ()> }
    impl<'a> Ev<'a> {
        fn ws(&mut self) { while self.i < self.s.len() && self.s[self.i] == ' ' { self.i += 1 } }
        fn expr(&mut self) -> f64 { let mut v = self.term(); loop { self.ws(); match self.s.get(self.i) { Some('+') => { self.i += 1; v += self.term() } Some('-') => { self.i += 1; v -= self.term() } _ => return v } } }
        fn term(&mut self) -> f64 { let mut v = self.atom(); loop { self.ws(); match self.s.get(self.i) { Some('*') => { self.i += 1; v *= self.atom() } Some('/') => { self.i += 1; v /= self.atom() } _ => return v } } }
        fn atom(&mut self) -> f64 {
            self.ws();
            if self.s[self.i] == '(' { self.i += 1; let v = self.expr(); self.ws(); self.i += 1; return v; }
            if self.s[self.i..].starts_with(&['i', 'n', 't', '(']) { self.i += 4; let v = self.expr(); self.ws(); self.i += 1; return v.floor(); }
            if self.s[self.i] == 'p' { self.i += 1; return self.p; }
            let st = self.i; while self.i < self.s.len() && self.s[self.i].is_ascii_digit() { self.i += 1 }
            self.s[st..self.i].iter().collect::<String>().parse().unwrap()
        }
    }
    fn ev(e: &str, p: f64) -> (f64, usize) { let mut x = Ev { s: e.chars().collect(), i: 0, p, _m: std::marker::PhantomData }; let v = x.expr(); (v, x.i) }
    fn run_atari(lines: &[String], start: f64) -> f64 {
        let mut p = start;
        for l in lines {
            if let Some(r) = l.strip_prefix("p = peek(") { let _ = r; p = start; }
            else if let Some(r) = l.strip_prefix("if ") {
                let (lhs, rest) = r.split_once(" then ").unwrap();
                let (expr, want) = lhs.rsplit_once(" = ").unwrap();
                let (v, _) = ev(expr, p);
                if v == want.trim().parse::<f64>().unwrap() { let (_, rhs) = rest.split_once(" = ").unwrap(); p = ev(rhs, p).0; }
            }
            else if let Some(r) = l.strip_prefix("p = ") { p = ev(r, p).0; }
            else if l.starts_with("poke ") { return p; }
        }
        panic!("kein poke")
    }

    #[test]
    fn atari_arithmetic_matches_the_bit_reference_for_every_mask_and_value() {
        let d = Dialect { id: "atari_basic".into(), atari: true, bitwise: Bitwise::None, max_line_len: 255, sig_chars: 120 };
        let o = Options { comments: Comment::Off, markers: false, ..Default::default() };
        for mask in 1u32..255 {
            for value in [0u32, 0xFF, 0x55, 0xAA, 0x0F, 0xF0, 0x33, 0x81] {
                let w = RegWrite { label: "x.y".into(), chip: "x".into(), addr: 559, base: 559, value: (value & mask) as u8, mask: mask as u8, access: Access::Rw, doc: String::new(), fields: vec![], shadow: None, lock: None, notes: vec![] };
                let g = generate(&[w], &d, &o);
                for start in 0u32..256 {
                    let got = run_atari(&g.lines, start as f64) as u32;
                    let want = (start & !mask) | (value & mask);
                    assert_eq!(got, want, "Maske {mask:#04x} Wert {value:#04x} Start {start}: {:?}", g.lines);
                }
            }
        }
    }
}
