// SPDX-License-Identifier: GPL-3.0-or-later
//! Statische Prüfungen ohne Emulator: Segmentgröße, verbotene Adressbereiche, Überlappung und Zyklenspanne.
//!
//! Grundlage sind die Zwischendateien des Bauens: `.vcdbg` (Symbole, Segmente, Zyklenblöcke, Zeilen mit Adresse und
//! Zyklen) und das Programmabbild (`.prg`, `.xex`) für die Opcodes. Die Zyklenspanne folgt allen Wegen eines Unterprogramms
//! von der Marke bis zu `rts`/`rti`. Bei Schleifen (Rücksprung), Unterprogrammaufrufen (`jsr`), `jmp (…)` und unbekannten
//! Befehlen meldet sie „nicht bestimmbar“ – sie rät keine Zahl. Gezählt werden die Zyklen der CPU; Abzüge durch DMA
//! (Bad Lines, Sprites), Interrupts und Wartezustände sind nicht enthalten. Seitenübertritte zählen im ungünstigsten Fall.
use super::model::{Level, Outcome, Report, State, StaticCheck, TestFile};
use crate::cond::{Ctx, Expr};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use vc_target::{Cpu, Mode};

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Hinweise (Zyklenbudget ohne eigene Angabe) werden zu Fehlern – für die CI
    pub strict: bool,
    /// Die Tests für den Emulator laufen anderswo (`runner`): `run_file` meldet sie dann nicht als „übersprungen“
    pub dynamic: bool,
}

#[derive(Debug, Clone)]
pub struct Seg { pub start: u32, pub len: u32, pub bank: u32, pub name: String }
#[derive(Debug, Clone)]
pub struct DbgLine { pub file: String, pub line: usize, pub addr: u16, pub len: usize, pub cycles: Option<(u32, u32)> }
#[derive(Debug, Clone)]
pub struct Block { pub name: String, pub file: String, pub line: usize, pub end_line: usize, pub min: u32, pub max: u32, pub budget: Option<u32> }

/// Inhalt einer `.vcdbg`
#[derive(Debug, Clone, Default)]
pub struct Dbg { pub symbols: HashMap<String, i64>, pub segments: Vec<Seg>, pub blocks: Vec<Block>, pub lines: Vec<DbgLine> }

impl Dbg {
    pub fn parse(text: &str) -> Result<Dbg, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("keine gültige .vcdbg ({e})"))?;
        let arr = |k: &str| v.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
        let str_of = |o: &Value, k: &str| o.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let num = |o: &Value, k: &str| o.get(k).and_then(Value::as_i64).unwrap_or(0);
        let mut d = Dbg::default();
        for o in arr("symbols") { d.symbols.insert(str_of(&o, "name"), num(&o, "value")); }
        for o in arr("segments") { d.segments.push(Seg { start: num(&o, "start") as u32, len: num(&o, "length") as u32, bank: num(&o, "bank") as u32, name: str_of(&o, "name") }); }
        for o in arr("cycleBlocks") {
            d.blocks.push(Block { name: str_of(&o, "name"), file: str_of(&o, "file"), line: num(&o, "line") as usize, end_line: num(&o, "endLine") as usize, min: num(&o, "min") as u32, max: num(&o, "max") as u32,
                budget: o.get("budget").and_then(Value::as_i64).map(|b| b as u32) });
        }
        for o in arr("lines") {
            let cycles = o.get("cycles").and_then(Value::as_array).and_then(|c| Some((c.first()?.as_u64()? as u32, c.get(1)?.as_u64()? as u32)));
            d.lines.push(DbgLine { file: str_of(&o, "file"), line: num(&o, "line") as usize, addr: num(&o, "addr") as u16, len: num(&o, "len") as usize, cycles });
        }
        Ok(d)
    }
}

/// Speicherabbild des Programms (Adresse → Byte); nur belegte Adressen sind bekannt
pub struct Image { mem: Vec<Option<u8>> }

impl Image {
    /// Abbild aus belegten Bereichen (Anfangsadresse, Bytes), z. B. für Tests
    pub fn from_runs(runs: &[(u16, &[u8])]) -> Image {
        let mut mem = vec![None; 0x10000];
        for (s, b) in runs { for (i, x) in b.iter().enumerate() { if *s as usize + i <= 0xFFFF { mem[*s as usize + i] = Some(*x); } } }
        Image { mem }
    }
    pub fn get(&self, a: u16) -> Option<u8> { self.mem[a as usize] }
    /// Zusammenhängende belegte Bereiche (Anfangsadresse, Bytes)
    pub fn runs(&self) -> Vec<(u16, Vec<u8>)> {
        let mut out: Vec<(u16, Vec<u8>)> = vec![];
        for (a, b) in self.mem.iter().enumerate() {
            let Some(b) = b else { continue };
            match out.last_mut() { Some((s, v)) if *s as usize + v.len() == a => v.push(*b), _ => out.push((a as u16, vec![*b])) }
        }
        out
    }
    /// `.prg` (Ladeadresse + Daten) und `.xex` (Blöcke mit Kopf `$FFFF`); andere Formate haben keine lesbaren Adressen
    pub fn load(path: &Path) -> Result<Image, String> {
        let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut mem = vec![None; 0x10000];
        match path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).as_deref() {
            Some("prg") => {
                if b.len() < 2 { return Err("PRG ist zu kurz".into()); }
                let start = u16::from_le_bytes([b[0], b[1]]) as usize;
                for (i, x) in b[2..].iter().enumerate() { if start + i <= 0xFFFF { mem[start + i] = Some(*x); } }
            }
            Some("xex") => {
                let (mut p, mut first) = (0usize, true);
                while p + 2 <= b.len() {
                    if b[p] == 0xFF && b[p + 1] == 0xFF { p += 2; if p + 2 > b.len() { break; } } else if first { return Err("XEX beginnt nicht mit $FFFF".into()); }
                    first = false;
                    if p + 4 > b.len() { return Err("XEX: Blockkopf abgeschnitten".into()); }
                    let (s, e) = (u16::from_le_bytes([b[p], b[p + 1]]) as usize, u16::from_le_bytes([b[p + 2], b[p + 3]]) as usize);
                    p += 4;
                    if e < s || p + (e - s + 1) > b.len() { return Err("XEX: Block ragt über das Dateiende".into()); }
                    for (i, x) in b[p..p + (e - s + 1)].iter().enumerate() { mem[s + i] = Some(*x); }
                    p += e - s + 1;
                }
            }
            other => return Err(format!("Programmformat „{}“ ist nicht lesbar (unterstützt: prg, xex)", other.unwrap_or("?"))),
        }
        Ok(Image { mem })
    }
}

struct SCtx<'a> { dbg: &'a Dbg, image: Option<&'a Image> }

impl SCtx<'_> {
    fn segs(&self, name: Option<&str>) -> Vec<&Seg> { self.dbg.segments.iter().filter(|s| name.map_or(true, |n| s.name == n)).collect() }
    fn segment_names(&self) -> String {
        let mut n: Vec<String> = self.dbg.segments.iter().map(|s| if s.name.is_empty() { "(ohne Namen)".to_string() } else { s.name.clone() }).collect();
        n.sort(); n.dedup(); n.join(", ")
    }
}

impl Ctx for SCtx<'_> {
    fn reg(&self, _name: &str) -> Option<i64> { None }
    fn peek(&mut self, addr: u16) -> Result<u8, String> {
        self.image.ok_or("Programmabbild nicht lesbar")?.get(addr).ok_or_else(|| format!("${addr:04X} gehört nicht zum Programm"))
    }
    fn size(&self, segment: &str) -> Result<i64, String> {
        let s = self.segs(Some(segment));
        if s.is_empty() { return Err(format!("Segment „{segment}“ nicht gefunden (vorhanden: {})", self.segment_names())); }
        Ok(s.iter().map(|s| s.len as i64).sum())
    }
}

fn eval(src: &str, ctx: &mut SCtx) -> Result<i64, String> { Expr::parse(src, &ctx.dbg.symbols)?.eval(ctx) }

/// Verbotene Bereiche: `$02-$1f` oder eine einzelne Adresse
fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let num = |t: &str| vc_target::parse_num(t.trim()).filter(|v| (0..=0xFFFF).contains(v)).map(|v| v as u32).ok_or_else(|| format!("Adresse „{}“ nicht verständlich (0..65535)", t.trim()));
    match s.split_once('-') { Some((a, b)) => { let (a, b) = (num(a)?, num(b)?); if a > b { Err(format!("Bereich „{s}“: Anfang liegt hinter dem Ende")) } else { Ok((a, b)) } } None => { let a = num(s)?; Ok((a, a)) } }
}

fn hex(v: i64) -> String { if v < 0x100 { format!("${v:02X}") } else { format!("${v:04X}") } }

/// Sucht `rel` im Ordner der Testdatei und in dessen übergeordneten Ordnern (Testdateien liegen oft in `tests/`)
fn resolve(test: &Path, rel: &str) -> Option<PathBuf> {
    let r = Path::new(rel);
    if r.is_absolute() { return r.exists().then(|| r.to_path_buf()); }
    let mut d = test.parent().map(Path::to_path_buf).unwrap_or_default();
    if d.as_os_str().is_empty() { d = PathBuf::from("."); }
    loop {
        if d.join(r).exists() { return Some(d.join(r)); }
        if !d.pop() || d.as_os_str().is_empty() { return None; }
    }
}

/// Geladene Grundlagen einer Testdatei: Ziel, CPU, Programmdatei, Debuginformation und Abbild
pub struct Prepared {
    pub target: vc_target::Target,
    pub cpu: Cpu,
    pub program: PathBuf,
    pub dbg: Dbg,
    pub image: Result<Image, String>,
}

impl Prepared {
    pub fn load(tf: &TestFile, data: &Path) -> Result<Prepared, String> {
        let target = vc_target::load_target(data, &tf.target).map_err(|e| format!("Ziel „{}“: {e}", tf.target))?;
        let cpu = vc_target::load_cpu(data, &target.cpu)?;
        let program = resolve(&tf.path, &tf.program).ok_or_else(|| format!("Programm „{}“ nicht gefunden – zuerst bauen (vcc build)", tf.program))?;
        let dbg_path = program.with_extension("vcdbg");
        let dbg = std::fs::read_to_string(&dbg_path).map_err(|e| format!("{}: {e} – zuerst bauen (vcc build)", dbg_path.display())).and_then(|t| Dbg::parse(&t))?;
        let image = Image::load(&program);
        Ok(Prepared { target, cpu, program, dbg, image })
    }
}

/// Führt alle statischen Prüfungen einer Testdatei aus. Tests für den Emulator werden gezählt und als „übersprungen“ gemeldet,
/// sofern `opts.dynamic` nicht gesetzt ist.
pub fn run_file(tf: &TestFile, data: &Path, opts: &Options) -> Report {
    let mut rep = Report { file: tf.path.clone(), target: tf.target.clone(), program: tf.program.clone(), outcomes: vec![] };
    let pre = match Prepared::load(tf, data) { Ok(p) => p, Err(e) => { rep.outcomes.push(Outcome::failed("Testdatei", Level::Error, e)); return rep; } };
    let (cpu, dbg) = (&pre.cpu, &pre.dbg);
    let image_err = pre.image.as_ref().err().cloned();
    let image = pre.image.as_ref().ok();
    let mut ctx = SCtx { dbg, image };
    for c in &tf.statics {
        let o = match c {
            StaticCheck::Size { segment, min, max, max_end } => check_size(&mut ctx, segment.as_deref(), min.as_deref(), max.as_deref(), max_end.as_deref()),
            StaticCheck::Zeropage { forbid } => check_zeropage(&mut ctx, cpu, forbid, image_err.as_deref()),
            StaticCheck::Overlap => check_overlap(&ctx),
            StaticCheck::Cycles { label, block, max, level } => check_cycles(&mut ctx, cpu, label.as_deref(), block.as_deref(), max.as_deref(), *level, opts.strict, image_err.as_deref()),
            StaticCheck::Expr { expr, name } => {
                let n = name.clone().unwrap_or_else(|| expr.clone());
                match eval(expr, &mut ctx) { Ok(v) if v != 0 => Outcome::passed(n, None), Ok(v) => Outcome::failed(n, Level::Error, format!("{expr} ergibt {v}, erwartet: wahr")), Err(e) => Outcome::failed(n, Level::Error, format!("Ausdruck nicht auswertbar: {e}")) }
            }
        };
        rep.outcomes.push(o);
    }
    if !opts.dynamic { for (i, t) in tf.tests.iter().enumerate() {
        let name = t.get("name").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| format!("Test {}", i + 1));
        rep.outcomes.push(Outcome::skipped(name, match tf.mode { super::model::TestMode::Mailbox => "braucht den Emulator (Postfach-Modus): `vcc test --backend vice`", super::model::TestMode::Measure => "braucht die Ultimate 64 (Zyklenmessung): `vcc test --backend u64:HOST`", super::model::TestMode::Trap => "braucht den Emulator (Trap-Modus): `vcc test --backend vice`" }));
    } }
    rep
}

fn check_size(ctx: &mut SCtx, segment: Option<&str>, min: Option<&str>, max: Option<&str>, max_end: Option<&str>) -> Outcome {
    let label = segment.map_or("alle Segmente".to_string(), |s| if s.is_empty() { "(ohne Namen)".into() } else { s.to_string() });
    let segs = ctx.segs(segment);
    if segs.is_empty() { return Outcome::failed(format!("Größe {label}"), Level::Error, format!("Segment „{}“ nicht gefunden (vorhanden: {})", segment.unwrap_or(""), ctx.segment_names())); }
    let size: i64 = segs.iter().map(|s| s.len as i64).sum();
    let end: i64 = segs.iter().map(|s| (s.start + s.len) as i64).max().unwrap_or(0);
    let mut name = format!("Größe {label}");
    if let Some(m) = max { name.push_str(&format!(" ≤ {m}")); }
    if let Some(m) = min { name.push_str(&format!(" ≥ {m}")); }
    if let Some(m) = max_end { name.push_str(&format!(" endet vor {m}")); }
    let mut problems = vec![];
    for (src, kind) in [(max, 0), (min, 1), (max_end, 2)] {
        let Some(src) = src else { continue };
        let lim = match eval(src, ctx) { Ok(v) => v, Err(e) => return Outcome::failed(name, Level::Error, format!("„{src}“ nicht auswertbar: {e}")) };
        match kind {
            0 if size > lim => problems.push(format!("{size} Bytes ({}) sind {} Bytes zu viel, erlaubt: höchstens {lim} ({})", hex(size), size - lim, hex(lim))),
            1 if size < lim => problems.push(format!("{size} Bytes sind {} Bytes zu wenig, erwartet: mindestens {lim}", lim - size)),
            2 if end > lim => problems.push(format!("das Segment endet bei {} (erste freie Adresse), erlaubt ist {} – {} Bytes zu weit", hex(end), hex(lim), end - lim)),
            _ => {}
        }
    }
    if problems.is_empty() { Outcome::passed(name, Some(format!("{} ({size} Bytes)", hex(size)))) } else { Outcome::failed(name, Level::Error, problems.join("; ")) }
}

fn check_overlap(ctx: &SCtx) -> Outcome {
    let name = "Keine Segment-Überlappung";
    let segs: Vec<&Seg> = ctx.dbg.segments.iter().filter(|s| s.len > 0).collect();
    let mut hits = vec![];
    for (i, a) in segs.iter().enumerate() {
        for b in &segs[i + 1..] {
            if a.bank == b.bank && a.start < b.start + b.len && b.start < a.start + a.len {
                let n = |s: &Seg| if s.name.is_empty() { "(ohne Namen)".to_string() } else { s.name.clone() };
                hits.push(format!("{} ${:04X}–${:04X} überlappt {} ${:04X}–${:04X}", n(a), a.start, a.start + a.len - 1, n(b), b.start, b.start + b.len - 1));
            }
        }
    }
    if hits.is_empty() { Outcome::passed(name, Some(format!("{} Segmente", segs.len()))) } else { Outcome::failed(name, Level::Error, hits.join("; ")) }
}

fn check_zeropage(ctx: &mut SCtx, cpu: &Cpu, forbid: &[String], image_err: Option<&str>) -> Outcome {
    let name = format!("Zeropage frei {}", forbid.join(", "));
    let mut ranges = vec![];
    for f in forbid { match parse_range(f) { Ok(r) => ranges.push(r), Err(e) => return Outcome::failed(name, Level::Error, e) } }
    let inside = |a: u32| ranges.iter().any(|(lo, hi)| a >= *lo && a <= *hi);
    let mut hits: Vec<String> = vec![];
    let mut first: Option<(String, usize)> = None;
    for s in ctx.dbg.segments.iter().filter(|s| s.len > 0) {
        if ranges.iter().any(|(lo, hi)| s.start <= *hi && s.start + s.len > *lo) {
            hits.push(format!("Segment {} liegt auf ${:04X}–${:04X}", if s.name.is_empty() { "(ohne Namen)" } else { &s.name }, s.start, s.start + s.len - 1));
        }
    }
    let Some(img) = ctx.image else {
        let why = image_err.unwrap_or("Programmabbild fehlt");
        return if hits.is_empty() { Outcome::skipped(name, format!("Zugriffe der Befehle nicht prüfbar: {why}")) } else { Outcome::failed(name, Level::Error, hits.join("; ")) };
    };
    let mut accesses = 0;
    for l in ctx.dbg.lines.iter().filter(|l| l.cycles.is_some()) {
        let Some(op) = img.get(l.addr).and_then(|c| cpu.by_opcode(c)) else { continue };
        if op.bytes as usize != l.len { continue; }
        let b1 = img.get(l.addr.wrapping_add(1));
        let target = match op.mode {
            Mode::Zp | Mode::Zpx | Mode::Zpy | Mode::Izx | Mode::Izy | Mode::Zpr | Mode::Izz | Mode::Isy => b1.map(u32::from),
            Mode::Abs | Mode::Abx | Mode::Aby | Mode::Ind => match (b1, img.get(l.addr.wrapping_add(2))) { (Some(lo), Some(0)) => Some(lo as u32), _ => None },
            _ => None,
        };
        if let Some(t) = target.filter(|t| inside(*t)) {
            accesses += 1;
            if first.is_none() { first = Some((l.file.clone(), l.line)); }
            if hits.len() < 6 { hits.push(format!("{}:{}: {} spricht ${t:02X} an (Adresse ${:04X})", short(&l.file), l.line, op.mnemonic.to_lowercase(), l.addr)); }
        }
    }
    if hits.is_empty() { return Outcome::passed(name, Some("kein Zugriff".to_string())); }
    let mut o = Outcome::failed(name, Level::Error, format!("{}{}", hits.join("; "), if accesses > 5 { format!("; … insgesamt {accesses} Zugriffe") } else { String::new() }));
    if let Some((f, l)) = first { o = o.at(&f, l); }
    o
}

fn short(p: &str) -> String { Path::new(p).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_else(|| p.to_string()) }

// ---------- Zyklenspanne ----------

struct Walker<'a> {
    by_addr: HashMap<u16, &'a DbgLine>,
    img: &'a Image,
    cpu: &'a Cpu,
    /// Bereich des Zyklenblocks `[von, bis)`: wer ihn verlässt, hat den Block beendet
    region: Option<(u16, u16)>,
    memo: HashMap<u16, Result<(u32, u32), String>>,
    stack: HashSet<u16>,
}

impl Walker<'_> {
    fn at(&self, a: u16) -> String { match self.by_addr.get(&a) { Some(l) => format!("${a:04X} ({}:{})", short(&l.file), l.line), None => format!("${a:04X}") } }
    fn walk(&mut self, addr: u16) -> Result<(u32, u32), String> {
        if let Some((lo, hi)) = self.region { if addr < lo || addr >= hi { return Ok((0, 0)); } }
        if let Some(r) = self.memo.get(&addr) { return r.clone(); }
        if self.stack.contains(&addr) { return Err(format!("Schleife bei {}", self.at(addr))); }
        if self.stack.len() > 3000 { return Err("Code zu verschachtelt".into()); }
        self.stack.insert(addr);
        let r = self.step(addr);
        self.stack.remove(&addr);
        self.memo.insert(addr, r.clone());
        r
    }
    fn step(&mut self, addr: u16) -> Result<(u32, u32), String> {
        let line = *self.by_addr.get(&addr).ok_or_else(|| format!("bei ${addr:04X} steht kein Befehl (Daten oder unbekannter Code)"))?;
        let (cmin, cmax) = line.cycles.ok_or_else(|| format!("bei {} stehen Daten statt eines Befehls", self.at(addr)))?;
        let op = self.img.get(addr).and_then(|c| self.cpu.by_opcode(c)).ok_or_else(|| format!("Opcode bei {} unbekannt", self.at(addr)))?;
        if op.bytes as usize != line.len { return Err(format!("Befehl bei {} nicht auswertbar (Präfix oder Sonderform)", self.at(addr))); }
        let next = addr.wrapping_add(line.len as u16);
        let b = |i: u16| self.img.get(addr.wrapping_add(i)).ok_or_else(|| format!("Operand bei {} fehlt im Programmabbild", self.at(addr)));
        let add = |(a, c): (u32, u32), (x, y): (u32, u32)| (a + x, c + y);
        match op.mnemonic.as_str() {
            "RTS" | "RTI" => return Ok((cmin, cmax)),
            "JSR" => return Err(format!("Unterprogrammaufruf (jsr) bei {}", self.at(addr))),
            "BRK" | "JAM" | "KIL" | "HLT" | "STP" => return Err(format!("{} bei {} beendet den Ablauf nicht bestimmbar", op.mnemonic.to_lowercase(), self.at(addr))),
            "JMP" => {
                if op.mode != Mode::Abs { return Err(format!("indirekter Sprung (jmp) bei {}", self.at(addr))); }
                let t = u16::from_le_bytes([b(1)?, b(2)?]);
                return Ok(add((cmin, cmax), self.walk(t)?));
            }
            _ => {}
        }
        match op.mode {
            Mode::Rel => {
                let t = next.wrapping_add(b(1)? as i8 as i16 as u16);
                let taken = if (t >> 8) == (next >> 8) { 3 } else { 4 };      // Sprung genommen: 3 Zyklen, über eine Seitengrenze 4
                let go = self.walk(t)?;
                if op.mnemonic == "BRA" { return Ok(add((taken, taken), go)); }
                let stay = self.walk(next)?;
                let (a, c) = (add((cmin, cmin), stay), add((taken, taken), go));
                Ok((a.0.min(c.0), a.1.max(c.1)))
            }
            Mode::Rl | Mode::Zpr => Err(format!("Sprungbefehl {} bei {} wird nicht ausgewertet", op.mnemonic.to_lowercase(), self.at(addr))),
            _ => Ok(add((cmin, cmax), self.walk(next)?)),
        }
    }
}

/// Spanne (min, max) in Zyklen ab `start`; `region` begrenzt sie auf einen Zyklenblock
fn span(ctx: &SCtx, cpu: &Cpu, start: u16, region: Option<(u16, u16)>) -> Result<(u32, u32), String> {
    let img = ctx.image.ok_or("Programmabbild nicht lesbar")?;
    let by_addr = ctx.dbg.lines.iter().map(|l| (l.addr, l)).collect();
    Walker { by_addr, img, cpu, region, memo: HashMap::new(), stack: HashSet::new() }.walk(start)
}

#[allow(clippy::too_many_arguments)]
fn check_cycles(ctx: &mut SCtx, cpu: &Cpu, label: Option<&str>, block: Option<&str>, max: Option<&str>, level: Option<Level>, strict: bool, image_err: Option<&str>) -> Outcome {
    let what = label.or(block).unwrap_or("?").to_string();
    let mut name = format!("Zyklen {what}");
    let mut limit_src = max.map(str::to_string);
    let (start, region, loc): (u16, Option<(u16, u16)>, Option<(String, usize)>);
    if let Some(l) = label {
        let Some(&a) = ctx.dbg.symbols.get(l) else { return Outcome::failed(name, Level::Error, format!("Marke „{l}“ nicht gefunden")) };
        if !(0..=0xFFFF).contains(&a) { return Outcome::failed(name, Level::Error, format!("„{l}“ ist keine Adresse ({a})")); }
        start = a as u16; region = None;
        loc = ctx.dbg.lines.iter().find(|x| x.addr == start && x.cycles.is_some()).map(|x| (x.file.clone(), x.line));
    } else {
        let bn = block.unwrap_or("");
        let Some(b) = ctx.dbg.blocks.iter().find(|b| b.name == bn) else {
            let names: Vec<&str> = ctx.dbg.blocks.iter().map(|b| b.name.as_str()).collect();
            return Outcome::failed(name, Level::Error, format!("Zyklenblock „{bn}“ nicht gefunden (vorhanden: {}). Im Quelltext: .cycles {bn} … .endcycles", if names.is_empty() { "keine".to_string() } else { names.join(", ") }));
        };
        if b.end_line == 0 { return Outcome::skipped(name, "Die .vcdbg stammt aus einer älteren Übersetzung (ohne Blockende) – neu bauen"); }
        let inst: Vec<&DbgLine> = ctx.dbg.lines.iter().filter(|l| l.cycles.is_some() && l.file == b.file && l.line > b.line && l.line < b.end_line).collect();
        let (Some(lo), Some(hi)) = (inst.iter().map(|l| l.addr).min(), inst.iter().map(|l| l.addr as u32 + l.len as u32).max()) else { return Outcome::failed(name, Level::Error, format!("Der Zyklenblock „{bn}“ enthält keine Befehle")) };
        start = lo; region = Some((lo, hi.min(0xFFFF) as u16));
        loc = Some((b.file.clone(), b.line));
        if limit_src.is_none() { limit_src = b.budget.map(|x| x.to_string()); }
    }
    if let Some(l) = &limit_src { name.push_str(&format!(" ≤ {l}")); }
    let finish = |o: Outcome| match &loc { Some((f, l)) => o.at(f, *l), None => o };
    if ctx.image.is_none() { return finish(Outcome::skipped(name, format!("Opcodes nicht lesbar: {}", image_err.unwrap_or("Programmabbild fehlt")))); }
    let (lo, hi) = match span(ctx, cpu, start, region) { Ok(s) => s, Err(why) => return finish(Outcome::skipped(name, format!("nicht bestimmbar: {why}"))) };
    let shown = if lo == hi { format!("{lo}") } else { format!("{lo} bis {hi}") };
    let Some(src) = limit_src else { return finish(Outcome::passed(name, Some(shown))) };
    let lim = match eval(&src, ctx) { Ok(v) => v, Err(e) => return finish(Outcome::failed(name, Level::Error, format!("„{src}“ nicht auswertbar: {e}"))) };
    if hi as i64 <= lim { return finish(Outcome::passed(name, Some(shown))); }
    let lvl = if strict { Level::Error } else { level.unwrap_or(Level::Hint) };
    finish(Outcome::failed(name, lvl, format!("im ungünstigsten Fall {hi} Zyklen (günstigster Fall {lo}), erlaubt: höchstens {lim} – {} zu viel", hi as i64 - lim)).with_measured(shown))
}

impl Report {
    /// Alle Ergebnisse in einer Liste von Zuständen – für knappe Zusammenfassungen
    pub fn summary(&self) -> (usize, usize, usize) { (self.count(State::Passed), self.count(State::Failed), self.count(State::Skipped)) }
}
