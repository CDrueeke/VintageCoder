// SPDX-License-Identifier: GPL-3.0-or-later
//! basic2asm gegen das echte BASIC V2: dasselbe Programm einmal als BASIC-PRG und einmal als konvertiertes Assembler-PRG
//! in VICE; der Bildschirminhalt muss gleich sein. Nur mit VC_EMU_TESTS=1 (öffnet Emulatorfenster).
use std::path::PathBuf;
use std::time::Duration;
use vc_basic::b2a::{convert, B2aOptions};
use vc_basic::{compile_plain_cbm, Dialect, PlainOptions};
use vc_emu::{Backend, Vice};

const SRC: &str = "print \"{clr}test\"
s% = 0
for i = 1 to 12
s% = s% + i * i
next i
print s%
a% = 5
b% = a% - 9
print a%; b%, a% * b%
if s% > 100 then print \"gross\"
if s% < 100 then print \"klein\"
gosub @unter
print \"fertig\"
end
@unter:
print \"unterprogramm\"; 7
return
";

const SRC_FLOAT: &str = "print \"{clr}float\"
for x = 0 to 1 step 0.25
print x;
next x
print
a% = 7
b = a% / 2
print b; a% * 2
c% = -3.7
print c%
d% = 3.7
print d%
dim f(3), g%(2)
f(2) = 1.5
g%(1) = 9
print f(2) * 2; g%(1)
read p, q
print p + q, 1 / 3
print int(-1.5); abs(-2); sgn(-7); sqr(16); 2 ^ 10
print sin(0); cos(0); exp(1); log(10); atn(1) * 4
print 1e10; 1.5e-5; 123456789012; -0.000001
r = 10
if r / 4 = 2.5 then print \"ok\"
print (r > 5) and (r < 20); not 0
data 1.5, 2.25
";

const SRC_STR: &str = "a$ = \"hallo\"
b$ = \" welt\"
c$ = a$ + b$
print c$; len(c$)
print left$(c$, 2); right$(c$, 3); mid$(c$, 3, 4); mid$(c$, 8)
print asc(\"a\"); chr$(66); str$(42); str$(-1.5); val(\"12abc\") + 1; val(\" -3.25\")
if a$ = \"hallo\" then print \"gleich\"
if a$ < \"hallp\" and \"ab\" < \"abc\" and b$ > \"\" then print \"kleiner\"
dim s$(3)
s$(1) = \"x\"
s$(2) = s$(1) + \"y\"
print s$(2); len(s$(0)); s$(3); \"/\"
read d$, n, e$, m$
print d$; n; e$; m$
x = 7
n$ = \"x=\" + str$(x / 2)
print n$
k$ = \"\"
for i = 1 to 40
k$ = k$ + chr$(64 + i)
next
print k$
print left$(k$, 5) + right$(k$, 3)
ti$ = \"123456\"
t$ = ti$
print left$(t$, 4); len(t$)
x = 3
def fna(x) = x * x + 1
def fnb(t) = int(t * 100 + .5) / 100
print fna(4); x; fnb(3.14159); fna(fna(1))
data hello, 5, \"a,b\", 3.5
";

fn screen_once(prg: &[u8], name: &str, emu: &str, scr: u16) -> String {
    let dir = std::env::temp_dir().join("vc_b2a_live");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("{name}.prg"));
    std::fs::write(&f, prg).unwrap();
    let mut v = Vice::launch_system(emu, f.to_str().unwrap(), None).expect("VICE starten");
    std::thread::sleep(Duration::from_secs(20));
    let scr = match v.pause().and_then(|_| v.read_mem(scr, 1000)) { Ok(s) => s, Err(e) => { eprintln!("VICE-Monitor: {e} (Wiederholung)"); v.stop(); return String::new(); } };
    v.stop();
    scr.chunks(40).map(|l| l.iter().map(|b| { let c = b & 0x7f; if c < 32 { (c + 64) as char } else { c as char } }).collect::<String>().trim_end().to_string() + "\n").collect()
}

/// Nur die Programmausgabe: bis zum letzten READY., ohne Ladeprotokoll vor "RUN". None = der Start ist misslungen (Autostart-Aussetzer)
fn program_output(screen: &str) -> Option<String> {
    let lines: Vec<&str> = screen.lines().collect();
    let end = lines.iter().rposition(|l| l.trim() == "READY.")?;
    let body = &lines[..end];
    let start = match body.iter().rposition(|l| l.trim() == "RUN") { Some(i) => i + 1, None => { if body.iter().any(|l| l.contains("COMMODORE 64 BASIC") || l.starts_with("LOAD")) { return None; } 0 } };
    let out: Vec<&str> = body[start..].iter().copied().collect();
    if out.iter().all(|l| l.trim().is_empty()) { return None; }
    if out.iter().any(|l| l.contains("BYTES FREE") || l.contains("BASIC V")) { return None; }   // nur der Startbanner: Autostart ist nicht gelaufen
    Some(out.join("\n").trim().to_string())
}

fn screen(prg: &[u8], name: &str, emu: &str, scr: u16) -> String {
    for _ in 0..3 {
        if let Some(o) = program_output(&screen_once(prg, name, emu, scr)) { return o; }
    }
    panic!("{name}: kein Programmergebnis auf dem Bildschirm (VICE-Start misslungen)");
}

#[test]
fn converted_program_shows_same_screen_as_basic() { compare(SRC, "int"); }

#[test]
fn converted_float_program_shows_same_screen_as_basic() { compare(SRC_FLOAT, "float"); }

#[test]
fn converted_string_program_shows_same_screen_as_basic() { compare(SRC_STR, "str"); }

fn compare(src: &str, tag: &str) { compare_on(src, tag, "c64"); }

/// Zielsystem → (VICE-Programm, Anfang des Bildschirmspeichers, Ladeadresse des BASIC-Programms)
fn machine(target: &str) -> (&'static str, u16, u16) { match target { "c128" => ("x128", 0x0400, 0x1C01), "plus4" => ("xplus4", 0x0C00, 0x1001), _ => ("x64sc", 0x0400, 0x0801) } }

fn compare_on(src: &str, tag: &str, target: &str) {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let src_text = src;
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let ti = vc_basic::b2a::target_info(target).unwrap();
    let (emu, scr, load) = machine(target);
    let d = Dialect::load(&data, ti.dialect).unwrap();
    let basic = compile_plain_cbm(src_text, &d, &PlainOptions::new("t.bas", 63999), load).unwrap().prg;
    let c = convert(src_text, &d, &B2aOptions { target: target.into(), ..B2aOptions::new("t.bas") }).unwrap();
    let cpu = vc_target::load_cpu(&data, ti.cpu).unwrap();
    let mut ao = vc_asm::Options::new("t.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data.join("sys"));
    let asm = vc_asm::assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{e:?}"));
    let conv = vc_asm::to_prg(&asm).unwrap();
    let (a, b) = (screen(&basic, &format!("original_{tag}_{target}"), emu, scr), screen(&conv, &format!("converted_{tag}_{target}"), emu, scr));
    eprintln!("--- Original ---\n{a}\n--- Konvertiert ---\n{b}");
    assert_eq!(a, b);
}

/// Ganzzahl- und Zeichenkettenprogramm ohne Fließkomma (für C128 und Plus/4)
const SRC_INT_STR: &str = "a$ = \"hallo\"
b$ = \" welt\"
c$ = a$ + b$
print c$; len(c$)
print left$(c$, 2); right$(c$, 3); mid$(c$, 3, 4); mid$(c$, 8)
print asc(\"a\"); chr$(66); str$(42); val(\"12abc\") + 1; val(\" -325\")
if a$ = \"hallo\" then print \"gleich\"
if a$ < \"hallp\" and \"ab\" < \"abc\" and b$ > \"\" then print \"kleiner\"
dim s$(3), n%(2)
s$(1) = \"x\"
s$(2) = s$(1) + \"y\"
n%(1) = 9
print s$(2); len(s$(0)); n%(1); \"/\"
read d$, n, e$, m$
print d$; n; e$; m$
k$ = \"\"
for i = 1 to 40
k$ = k$ + chr$(64 + i)
next
print k$
print left$(k$, 5) + right$(k$, 3)
ti$ = \"123456\"
t$ = ti$
print left$(t$, 4); len(t$)
x = 3
def fna(x) = x * x + 1
print fna(4); x; fna(fna(1))
on x goto @a, @b, @c
@a:
print \"eins\"
@b:
print \"zwei\"
@c:
print \"drei\"
gosub @s
print \"ende\"
end
@s:
print \"sub\"
return
data hello, 5, \"a,b\", 7
";

#[test]
fn converted_programs_match_basic_on_c128() { compare_on(SRC, "int", "c128"); compare_on(SRC_INT_STR, "str", "c128"); }

#[test]
fn converted_programs_match_basic_on_plus4() { compare_on(SRC, "int", "plus4"); compare_on(SRC_INT_STR, "str", "plus4"); }

#[test]
fn converted_float_program_matches_basic_on_c128() { compare_on(SRC_FLOAT, "float", "c128"); }

#[test]
fn converted_float_program_matches_basic_on_plus4() { compare_on(SRC_FLOAT, "float", "plus4"); }

// ---- Atari 8 Bit: das konvertierte Programm läuft als XEX in atari800 (mit dem echten Betriebssystem); der Bildschirmspeicher wird gelesen

const SRC_ATARI: &str = "print \"HALLO ATARI\"
s = 0
for i = 1 to 12
s = s + i * i
next i
print s
a = 5
b = a - 9
print a; b, a * b
if s > 100 then print \"gross\"
if s < 100 then print \"klein\"
gosub @unter
print \"fertig\"
end
@unter:
print \"unterprogramm\"; 7
return
";

/// Bildschirmcodes der Textzeilen (GRAPHICS 0, 40 Spalten) als Text
fn atari_screen_text(mem: &[u8]) -> Vec<String> {
    mem.chunks(40).map(|l| l.iter().map(|&b| match b & 0x7F { c @ 0x00..=0x3F => (c + 0x20) as char, c @ 0x40..=0x5F => (c - 0x40) as char, c @ 0x60..=0x7F => c as char, _ => '?' }).collect::<String>().trim_end().to_string()).collect()
}

/// Konvertiert ein Atari-Programm, startet es in atari800 (echtes OS) und liefert die Bildschirmzeilen ohne linken Rand
fn run_atari(src: &str) -> Vec<String> {
    use vc_emu::{Atari800, Backend};
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = Dialect::load(&data, "cbm_v2").unwrap();
    let mut o = B2aOptions::new("atari.bas");
    o.target = "atari8".into();
    let c = convert(src, &d, &o).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let cpu = vc_target::load_cpu(&data, "sally").unwrap();
    let mut ao = vc_asm::Options::new("atari.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data.join("sys"));
    let out = vc_asm::assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let dir = std::env::temp_dir().join("vc_b2a_live");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("atari.xex");
    std::fs::write(&f, vc_asm::to_xex(&out).unwrap()).unwrap();
    let done = out.symbols["rt_end"] as u16;
    let mut a = Atari800::launch(f.to_str().unwrap(), None).expect("atari800 starten");
    let hit = a.run_until(done, Duration::from_secs(60)).expect("run_until");
    assert!(hit, "Programmende nicht erreicht");
    let sav = a.read_mem(0x58, 2).unwrap();
    let scr = a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap();
    a.stop();
    let lines = atari_screen_text(&scr);
    lines.iter().map(|l| l.get(2..).unwrap_or("").to_string()).filter(|l| !l.is_empty()).collect()   // 2 Spalten linker Rand
}

#[test]
fn converted_atari_program_runs_on_the_real_os() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let shown = run_atari(SRC_ATARI);
    // Ausgabe von Atari-BASIC: Zahlen ohne Vorzeichenstelle, Komma = nächste Zehnerspalte
    let want = ["HALLO ATARI", "650", "5-4       -20", "gross", "unterprogramm7", "fertig"];
    let at = shown.iter().position(|l| l == want[0]).unwrap_or_else(|| panic!("Anfang fehlt: {shown:?}"));
    assert_eq!(&shown[at..at + want.len()], &want, "{shown:?}");
}

const SRC_ATARI_FLOAT: &str = "print \"FLOAT\"
print 1/3
print 2^10
print sqr(16)
for x = 0 to 1 step 0.5
print x; \"/\";
next x
print
dim a(2)
a(1) = 2.5
print a(1) * 4
read p
print p + 0.25
print 10 / 4; 7 * 1.5
data 1.5
";

#[test]
fn converted_atari_float_program_runs_on_the_real_os() {
    if std::env::var("VC_EMU_TESTS").is_err() { eprintln!("übersprungen (VC_EMU_TESTS=1 setzen)"); return; }
    let shown = run_atari(SRC_ATARI_FLOAT);
    let want = ["FLOAT", "0.333333333", "1024", "4", "0/0.5/1/", "10", "1.75", "2.510.5"];
    let at = shown.iter().position(|l| l == want[0]).unwrap_or_else(|| panic!("Anfang fehlt: {shown:?}"));
    assert_eq!(&shown[at..at + want.len()], &want, "{shown:?}");
}
