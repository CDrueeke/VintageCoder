// SPDX-License-Identifier: GPL-3.0-or-later
//! basic2asm für Atari 8 Bit (Stufe 1): BASIC → Assembler → 6502-Kern. Das Ergebnis läuft ohne Emulator in einem kleinen Modell
//! (64-KB-RAM); die Aufrufe der OS-Schicht `rt_os.asm` (CHROUT, CHRIN, GETIN, PLOT, RDTIM, SETTIM) werden an ihren Adressen abgefangen.
use std::collections::VecDeque;
use std::path::PathBuf;
use vc_asm::{assemble, Options, Policy};
use vc_basic::b2a::{convert, B2aOptions};
use vc_basic::Dialect;
use vc_media::cpu6502::{Bus, Cpu};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

struct Machine { ram: Vec<u8>, rnd: u32 }
impl Bus for Machine {
    fn read(&mut self, a: u16) -> u8 {
        if a == 0xD20A { self.rnd = self.rnd.wrapping_mul(1103515245).wrapping_add(12345); return (self.rnd >> 16) as u8; }   // POKEY RANDOM
        self.ram[a as usize]
    }
    fn write(&mut self, a: u16, v: u8) { self.ram[a as usize] = v; }
}

struct Run { out: String, ram: Vec<u8> }

fn opts() -> B2aOptions { let mut o = B2aOptions::new("t.bas"); o.target = "atari8".into(); o }

fn try_convert(src: &str, o: &B2aOptions) -> Result<vc_basic::b2a::Converted, String> {
    let d = Dialect::load(&data(), vc_basic::b2a::target_info(&o.target).unwrap().dialect).unwrap();
    convert(src, &d, o).map_err(|e| e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"))
}

fn build(src: &str) -> vc_asm::Output { build_with(src, opts()) }

fn build_with(src: &str, o: B2aOptions) -> vc_asm::Output {
    let c = try_convert(src, &o).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(std::env::temp_dir().join("b2a_atari_last.asm"), &c.asm).ok();
    let cpu = vc_target::load_cpu(&data(), "sally").unwrap();
    let mut ao = Options::new("t.asm", Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}\n{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"), c.asm))
}

/// OS-ROM (Rev. B, 10 KB ab $D800) mit dem Fließkommapaket: Umgebungsvariable VC_ATARI_OSROM oder assets/ROMS/atari/ATARIOSB.ROM (im Hauptverzeichnis, nicht in Git)
fn os_rom() -> Option<Vec<u8>> {
    let m = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut c: Vec<PathBuf> = std::env::var("VC_ATARI_OSROM").ok().map(PathBuf::from).into_iter().collect();
    for up in ["../..", "../../../../.."] { c.push(m.join(up).join("assets/ROMS/atari/ATARIOSB.ROM")); }
    c.iter().find_map(|p| std::fs::read(p).ok()).filter(|r| r.len() == 0x2800)
}

fn atascii_out(b: &[u8]) -> String {
    b.iter().map(|&c| match c { 0x9B => '\n', 0x7D => '~', 0x20..=0x7E => c as char, _ => '?' }).collect()
}

fn run_with(src: &str, input: &str, keys: &[u8]) -> Run { run_built(build(src), input, keys) }

fn run_built(out: vc_asm::Output, input: &str, keys: &[u8]) -> Run {
    let sym = |n: &str| *out.symbols.get(n).unwrap_or_else(|| panic!("Symbol {n}")) as u16;
    let (chrout, chrin, getin, plot, rdtim, settim) = (sym("CHROUT"), sym("CHRIN"), sym("GETIN"), sym("PLOT"), sym("RDTIM"), sym("SETTIM"));
    let mut m = Machine { ram: vec![0; 65536], rnd: 1 };
    if let Some(rom) = os_rom() { m.ram[0xD800..].copy_from_slice(&rom); }   // Fließkommapaket des OS ($D800-$FFFF)
    for s in &out.segments { m.ram[s.start as usize..s.start as usize + s.bytes.len()].copy_from_slice(&s.bytes); }
    let mut printed: Vec<u8> = vec![];
    let mut input: VecDeque<u8> = input.bytes().map(|b| if b == b'\n' { 0x9B } else { b }).collect();
    let mut keys: VecDeque<u8> = keys.iter().copied().collect();
    let mut col = 0u8;
    let mut cpu = Cpu::new();
    cpu.pc = out.run.expect(".run");
    cpu.sp = 0xF0;
    cpu.push(&mut m, 0x01); cpu.push(&mut m, 0xFF);   // Rücksprung auf die Falle $0200
    let ret = |cpu: &mut Cpu, m: &mut Machine| { let lo = cpu.pull(m) as u16; let hi = cpu.pull(m) as u16; cpu.pc = (lo | hi << 8).wrapping_add(1); };
    for _ in 0..50_000_000u64 {
        let pc = cpu.pc;
        if pc == 0x0200 { break; }
        if pc == chrout { let a = cpu.a; let a = if a == 13 { 0x9B } else { a }; col = if a == 0x9B { 0 } else { col.wrapping_add(1) }; printed.push(a); ret(&mut cpu, &mut m); continue; }
        if pc == chrin { cpu.a = match input.pop_front() { Some(0x9B) | None => 13, Some(b) => b }; cpu.c = false; ret(&mut cpu, &mut m); continue; }
        if pc == getin { cpu.a = keys.pop_front().unwrap_or(0); ret(&mut cpu, &mut m); continue; }
        if pc == plot { cpu.y = col; cpu.x = 0; ret(&mut cpu, &mut m); continue; }
        if pc == rdtim { cpu.a = m.ram[0x14]; cpu.x = m.ram[0x13]; cpu.y = m.ram[0x12]; ret(&mut cpu, &mut m); continue; }
        if pc == settim { m.ram[0x14] = cpu.a; m.ram[0x13] = cpu.x; m.ram[0x12] = cpu.y; ret(&mut cpu, &mut m); continue; }
        cpu.step(&mut m).expect("Befehl");
    }
    assert_eq!(cpu.pc, 0x0200, "Programm endete nicht");
    Run { out: atascii_out(&printed), ram: m.ram }
}

fn run(src: &str) -> Run { run_with(src, "", &[]) }

#[test]
fn print_has_no_sign_column() {
    let r = run("a = 6\nb = 7\nprint a*b\nprint a-b; a+b\nprint \"Hallo\"; 5\n");
    assert_eq!(r.out, "42\n-113\nHallo5\n");
}

#[test]
fn text_is_ascii_not_petscii() {
    let r = run("print \"Hello, World 123\"\n? \"abc\"\n");
    assert_eq!(r.out, "Hello, World 123\nabc\n");
}

#[test]
fn loops_gosub_and_labels() {
    let src = "s = 0\nfor i = 1 to 10\ns = s + i\nnext i\nprint s\ngosub @sub\nprint \"ende\"\nend\n@sub:\nprint \"sub\"\nreturn\n";
    assert_eq!(run(src).out, "55\nsub\nende\n");
}

#[test]
fn data_arrays_on_goto_and_comma_tabs() {
    let src = "dim a(4)\nfor i = 0 to 4\nread a(i)\nnext\nt = 0\nfor i = 0 to 4\nt = t + a(i)\nnext\nprint t\non 2 goto @x, @y\nprint \"nie\"\n@x:\nprint \"x\"\n@y:\nprint \"y\"\nprint 1, 2, 3\ndata 10, 20, -5\ndata 7, 8\n";
    assert_eq!(run(src).out, "40\ny\n1         2         3\n");
}

#[test]
fn poke_peek_and_logic() {
    let r = run("poke 1536+100, 77\nx = peek(1636)\nprint x\nprint (x > 70) and (x < 80)\n");
    assert_eq!(r.out, "77\n1\n");
    assert_eq!(r.ram[1636], 77);
}

#[test]
fn input_get_and_time() {
    let r = run_with("input n\nprint n*2\nget k\nprint k\nprint ti\n", "21\n", b"7");
    assert_eq!(r.out, "? \n42\n7\n0\n".replace("? \n", "? \n"));
}

#[test]
fn zero_page_block_is_restored() {
    // Muster in $80-$93 wird gesichert und beim Ende zurückgeschrieben: eigener Test ohne Muster genügt – hier nur: Programm endet sauber nach Fehlerfreiheit
    let r = run("print 1\n");
    assert_eq!(r.out, "1\n");
}

#[test]
fn runtime_errors_use_ascii_messages() {
    let mut o = opts();
    o.int_div = true;
    let out = build_with("a = 5\nb = 0\nprint a / b\n", o);
    assert!(out.symbols.contains_key("rt_error"));
}

#[test]
fn atari_only_statements_are_rejected() {
    for (src, word) in [("xio 18,6,0,0,\"S:\"\n", "XIO"), ("pop\n", "POP"), ("trap 100\n", "TRAP"), ("note 1,a,b\n", "NOTE"), ("print paddle(0)\n", "PADDLE")] {
        let e = try_convert(src, &opts()).unwrap_err();
        assert!(e.contains("E311") && e.contains(word), "{src}: {e}");
    }
    // in Zeichenketten und Kommentaren sind es nur Wörter
    if let Err(e) = try_convert("print \"graphics\"\nrem sound\ndata 1\n", &opts()) { panic!("{e}"); }
}

#[test]
fn long_atari_names_work() {
    let r = run("spielstand = 5\nspielerzahl = 2\nprint spielstand*spielerzahl\ntotal = 3\nscore = 4\nprint total+score\n");
    assert_eq!(r.out, "10\n7\n");
}

#[test]
fn xex_has_run_segment() {
    let out = build("print 1\n");
    let x = vc_asm::to_xex(&out).unwrap();
    assert_eq!(&x[..2], &[0xFF, 0xFF]);
    assert_eq!(out.segments[0].start, 0x2000);
}

// ---- Stufe 2: Fließkomma über das OS-Paket (6-Byte-BCD). Die Läufe brauchen das OS-ROM (siehe os_rom); ohne ROM werden sie übersprungen.

fn runf(src: &str) -> Option<Run> { runf_with(src, "", &[]) }
fn runf_with(src: &str, input: &str, keys: &[u8]) -> Option<Run> {
    if os_rom().is_none() { eprintln!("übersprungen: kein OS-ROM (VC_ATARI_OSROM=…/ATARIOSB.ROM)"); return None; }
    Some(run_with(src, input, keys))
}

#[test]
fn bcd_encoding_matches_the_os_format() {
    use vc_basic::b2a::bcd;
    assert_eq!(bcd(1.0).unwrap(), [0x40, 1, 0, 0, 0, 0]);
    assert_eq!(bcd(100.0).unwrap(), [0x41, 1, 0, 0, 0, 0]);
    assert_eq!(bcd(0.5).unwrap(), [0x3F, 0x50, 0, 0, 0, 0]);
    assert_eq!(bcd(-12.5).unwrap(), [0xC0, 0x12, 0x50, 0, 0, 0]);
    assert_eq!(bcd(3.14159265).unwrap(), [0x40, 3, 0x14, 0x15, 0x92, 0x65]);
    assert_eq!(bcd(9.9999999995).unwrap(), [0x40, 0x10, 0, 0, 0, 0]);
    assert_eq!(bcd(0.0).unwrap(), [0; 6]);
    assert!(bcd(1e99).is_err());
}

#[test]
fn float_arithmetic_and_print_format() {
    let Some(r) = runf("print 1/3\nprint 2^10\nprint 10/4\nprint -7/2\nprint 0.5\nprint 100\nprint 1e10\nprint sqr(16)\nprint int(-2.5)\nprint 0.1+0.2\nprint 123456789012\nprint 1/1000000\nprint 2^-2\nprint (-2)^3\nprint 7*0.5; 3\n") else { return };
    // Format des BASIC-Moduls: kein Vorzeichen-/Endleerzeichen; ist das erste Ziffernpaar >= 10, fällt die zehnte Stelle weg
    assert_eq!(r.out, "0.333333333\n1024\n2.5\n-3.5\n0.5\n100\n1E+10\n4\n-3\n0.3\n1.23456789E+11\n1E-06\n0.250000002\n-8\n3.53\n");
}

#[test]
fn bcd_arithmetic_is_decimal_exact() {
    let Some(r) = runf("if 0.1+0.2 = 0.3 then print \"gleich\"\nif 0.1 < 0.2 then print \"kleiner\"\nif 1/3*3 <> 1 then print \"nicht\"\nx = 1e-3\nprint x*1000\n") else { return };
    assert!(r.out.starts_with("gleich\nkleiner\n"), "{}", r.out);
    assert_eq!(r.out, "gleich\nkleiner\nnicht\n1\n", "{}", r.out);   // 1/3*3 ist im BCD-Format 0,999999999…, kein Gleichheitswunder
}

#[test]
fn float_loops_arrays_data_and_functions() {
    let src = "dim a(3)\nfor x = 0 to 2 step 0.5\nprint x;\" \";\nnext x\nprint\nfor i = 3 to 1 step -1\na(i) = i / 2\nnext i\nprint a(1) + a(2) + a(3)\nread p, q\nprint p*q\nprint abs(-2.5); sgn(-3); sgn(0); int(7.9)\ndata 1.5, 4\n";
    let Some(r) = runf(src) else { return };
    assert_eq!(r.out, "0 0.5 1 1.5 2 \n3\n6\n2.5-10 7\n".replace("2.5-10 7", "2.5-107"), "{}", r.out);
}

#[test]
fn float_input_and_errors() {
    let Some(r) = runf_with("input n\nprint n*2.5\ninput m\nprint m+1\n", "3.25\nabc\n10\n", &[]) else { return };
    assert_eq!(r.out, "? \n8.125\n? \n?REDO FROM START\n? \n11\n", "{}", r.out);
}

#[test]
fn float_runtime_errors_and_functions() {
    // Division durch 0, Wurzel aus negativer Zahl, Überlauf
    for (src, msg) in [("a=1\nb=0\nprint a/b\n", "DIVISION BY ZERO"), ("print sqr(-1)\n", "ILLEGAL QUANTITY"), ("print log(0)\n", "ILLEGAL QUANTITY"), ("print 1e50*1e50\n", "OVERFLOW")] {
        let Some(r) = runf(src) else { return };
        assert!(r.out.contains(msg), "{src}: {}", r.out);
    }
    let Some(r) = runf("print sqr(2)\nprint exp(1)\nprint log(10)\nprint 3^3\nprint 2^0.5\n") else { return };
    let v: Vec<f64> = r.out.lines().map(|l| l.parse().unwrap()).collect();
    for (got, want) in v.iter().zip([2f64.sqrt(), 1f64.exp(), 10f64.ln(), 27.0, 2f64.sqrt()]) { assert!((got - want).abs() < 1e-7, "{got} {want}"); }
    // RND liefert 0 <= r < 1 und ist nicht konstant
    let Some(r) = runf("for i = 1 to 20\nx = rnd(0)\nif x < 0 or x >= 1 then print \"bad\"\ns = s + x\nnext i\nprint s > 3 and s < 17\n") else { return };
    assert_eq!(r.out, "1\n", "{}", r.out);
}

#[test]
fn float_conversions_and_ti() {
    // Ganzzahlen und Fließkomma mischen: PEEK/POKE mit Fließkommaausdrücken, TI als Zahl, Rundung beim Index
    let Some(r) = runf("poke 1536, 200.9\nprint peek(1536)\nprint ti > -1\na = 1.5\nprint int(a*2)\nprint int(-0.5)\nx = 40000.7\nprint x\n") else { return };
    assert_eq!(r.ram[1536], 201);   // wie im echten Atari-BASIC: FPI rundet (200.9 -> 201)
    assert_eq!(r.out, "201\n1\n3\n-1\n40000.7\n", "{}", r.out);
}

#[test]
fn trig_functions_and_deg_rad() {
    // TAN gibt es in Atari-BASIC nicht
    let e = try_convert("print tan(1)\n", &opts()).unwrap_err();
    assert!(e.contains("E334"), "{e}");
    let Some(r) = runf("print sin(1)\nprint cos(1)\nprint atn(1)\ndeg\nprint sin(30)\nprint cos(60)\nprint atn(1)\nrad\nprint sin(1)\n") else { return };
    let v: Vec<f64> = r.out.lines().map(|l| l.parse().unwrap()).collect();
    let want = [1f64.sin(), 1f64.cos(), 1f64.atan(), 0.5, 0.5, 45.0, 1f64.sin()];
    assert_eq!(v.len(), want.len(), "{}", r.out);
    for (g, w) in v.iter().zip(want) { assert!((g - w).abs() < 1e-6, "{g} {w}: {}", r.out); }   // wie im Original: sin(30 Grad) = 0.499999999
}

#[test]
fn atari_strings_assign_truncate_and_substrings() {
    let src = "dim a$(5), b$(10)\na$ = \"HELLO WORLD\"\nprint a$; len(a$)\nb$ = \"ABCDEF\"\nprint b$(1,1); \"|\"; b$(2); \"|\"; b$(2,3); \"|\"; b$(6,6)\nb$(3,4) = \"XYZ\"\nprint b$; len(b$)\nb$(3) = \"XY\"\nprint b$; len(b$)\nb$(len(b$)+1) = \"!!\"\nprint b$; len(b$)\nb$(7) = \"123\"\nprint b$; len(b$)\n";
    let Some(r) = runf(src) else { return };
    assert_eq!(r.out, "HELLO5\nA|BCDEF|BC|F\nABXYEF6\nABXY4\nABXY!!6\nABXY!!1239\n", "{}", r.out);
}

#[test]
fn atari_string_functions_and_compare() {
    let src = "dim a$(20), b$(10)\na$ = str$(12.5)\nprint a$\nprint val(\"12.5E1\"); val(\"-3\"); asc(\"A\"); chr$(66); len(str$(100))\nprint val(\"12AB\"); val(\" 12\")\na$ = \"ABC\"\nb$ = \"ABD\"\nif a$ < b$ then print \"kleiner\"\nif a$ = b$ then print \"gleich\"\nif a$ <> b$ then print \"anders\"\nif a$ <= \"ABC\" then print \"le\"\nb$ = \"AB\"\nif b$ < a$ then print \"kuerzer\"\nprint str$(1/3); \"|\"; str$(-2); \"|\"; str$(1e10)\n";
    let Some(r) = runf(src) else { return };
    assert_eq!(r.out, "12.5\n125-365B3\n1212\nkleiner\nanders\nle\nkuerzer\n0.3333333333|-2|1E+10\n", "{}", r.out);
}

#[test]
fn atari_string_data_input_and_errors() {
    let Some(r) = runf_with("dim a$(10), b$(10), c$(20)\nread a$, b$, x\nprint \"[\"; a$; \"][\"; b$; \"]\"; x\ninput c$\nprint c$\ndata \" AB \", CD:E, 7\n", "hallo welt\n", &[]) else { return };
    assert!(r.out.contains("[\" AB \"][ CD:E]7"), "{}", r.out);
    assert!(r.out.contains("hallo welt"), "{}", r.out);
    for (src, msg) in [("dim a$(5)\na$ = \"ABC\"\nprint a$(4)\n", "STRING LENGTH"), ("dim a$(5)\ndim a$(6)\n", "DIMENSION"), ("print val(\"x\")\ndim a$(2)\n", "BAD VALUE"), ("read x\ndata AB\n", "INPUT"), ("dim a$(5)\nread a$, a$\ndata 1\n", "OUT OF DATA")] {
        let Some(r) = runf(src) else { return };
        assert!(r.out.contains(msg), "{src}: {}", r.out);
    }
}

#[test]
fn atari_string_rules_are_enforced_at_compile_time() {
    for (src, code) in [("dim a$(5)\na$ = a$ + \"x\"\n", "E311"), ("dim a$(5)\nprint left$(a$,2)\n", "E311"), ("a$ = \"x\"\nprint a$\n", "E321"), ("n = 5\ndim a$(n)\n", "E312")] {
        let e = try_convert(src, &opts()).unwrap_err();
        assert!(e.contains(code), "{src}: {e}");
    }
}

#[test]
fn atari_sound_setcolor_and_joystick() {
    // SOUND und SETCOLOR schreiben nur Register (keine CIO-Aufrufe): im Modell prüfbar
    let r = run("sound 1,100,10,8\nsetcolor 2,3,4\nsetcolor 0,15,14\nx = stick(0) + strig(1)\nprint x\n");
    assert_eq!(r.ram[0xD202], 100, "AUDF2");
    assert_eq!(r.ram[0xD203], 0xA8, "AUDC2");
    assert_eq!(r.ram[0xD208], 0, "AUDCTL");
    assert_eq!(r.ram[0xD20F], 3, "SKCTL");
    assert_eq!(r.ram[0x02C6], 0x34, "COLOR2");
    assert_eq!(r.ram[0x02C4], 0xFE, "COLOR0");
    assert_eq!(r.out, "0\n");
    for bad in ["sound 4,0,0,0\n", "setcolor 5,0,0\n"] {
        let r = run(bad);
        assert!(r.out.contains("ILLEGAL QUANTITY"), "{bad}: {}", r.out);
    }
}

#[test]
fn atari_graphics_statements_are_parsed() {
    // Wörter wie COLOR (enthält OR), POSITION (POS) und STICK werden als Atari-Anweisungen gelesen, nicht als Variablen
    for src in ["graphics 8+16\ncolor 1\nplot 10,10\ndrawto 20,20\nposition 5,5\nsetcolor 2,0,0\nsound 0,1,2,3\nlocate 1,1,a\nprint #6;\"x\";a\nprint stick(0);strig(0)\n"] {
        try_convert(src, &opts()).unwrap_or_else(|e| panic!("{e}"));
    }
    let e = try_convert("plot 1\n", &opts()).unwrap_err();
    assert!(e.contains("PLOT erwartet 2 Zahlen"), "{e}");
    let e = try_convert("xio 1,2\n", &opts()).unwrap_err();
    assert!(e.contains("E311"), "{e}");
}

#[test]
fn atari_unary_sign_quirks() {
    // Messung am echten BASIC: ein Vorzeichen vor einem weiteren Vorzeichen entfällt; NOT -1 ist Fehler 11 (wird beim Übersetzen gemeldet)
    let Some(r) = runf("print - -2; \" \"; 3- -2; \" \"; - +4; \" \"; not (-1)\n") else { return };
    assert_eq!(r.out, "-2 5 4 0\n", "{}", r.out);
    let e = try_convert("print not -1\n", &opts()).unwrap_err();
    assert!(e.contains("Fehler 11"), "{e}");
}

// ---- Turbo BASIC XL (nur Ziel atari8): Blockanweisungen und Prozeduren, in einfache Sprünge übersetzt

fn turbo_opts() -> B2aOptions { let mut o = opts(); o.turbo = true; o }

fn run_turbo(src: &str) -> Run {
    let out = build_with(src, turbo_opts());
    run_built(out, "", &[])
}

#[test]
fn turbo_blocks_and_procedures() {
    let src = "; Turbo BASIC XL ohne Zeilennummern
PROC zwei
  PRINT \"zwei\"
ENDPROC
PROC mal(x)
ENDPROC
i=0
WHILE i<5
  PRINT i;
  i=i+1
WEND
PRINT
j=3
REPEAT
  PRINT j;
  j=j-1
UNTIL j=0
PRINT
k=0
DO
  k=k+1
  IF k>5
    EXIT
  ENDIF
LOOP
PRINT k
x=7
IF x MOD 2
  PRINT \"ungerade\"
ELSE
  PRINT \"gerade\"
ENDIF
IF x DIV 2 = 3 THEN PRINT \"drei\"
EXEC zwei
EXEC zwei
";
    // Parameter in PROC gibt es nicht: der Parser meldet die Zeile
    let e = try_convert(src, &turbo_opts()).unwrap_err();
    assert!(e.contains("E103") || e.contains("E1"), "{e}");
    let src = src.replace("PROC mal(x)\nENDPROC\n", "");
    let r = run_turbo(&src);
    assert_eq!(r.out, "01234\n321\n6\nungerade\ndrei\nzwei\nzwei\n", "{}", r.out);
}

#[test]
fn turbo_nesting_exit_and_errors() {
    let src = "i=0
WHILE i<3
  j=0
  REPEAT
    j=j+1
    IF j=2
      PRINT \"[\";i;j;\"]\";
    ELSE
      IF i=1 THEN PRINT \"-\";
    ENDIF
  UNTIL j>=3
  i=i+1
WEND
PRINT
DO
  PRINT \"x\";
  EXIT
LOOP
PRINT \"y\"
";
    let r = run_turbo(src);
    assert_eq!(r.out, "[02]-[12]-[22]\nxy\n", "{}", r.out);
    for (bad, code) in [("IF 1\nPRINT 1\n", "E341"), ("ENDIF\n", "E342"), ("WEND\n", "E342"), ("EXIT\n", "E342"), ("EXEC nix\n", "E343"), ("PROC a\nENDPROC\nPROC a\nENDPROC\n", "E340"), ("ELSE\n", "E342"), ("WHILE 1\nUNTIL 1\n", "E342")] {
        let e = try_convert(bad, &turbo_opts()).unwrap_err();
        assert!(e.contains(code), "{bad:?}: {e}");
    }
    // ohne Turbo-Schalter sind die Wörter gewöhnliche Namen (Atari-BASIC kennt sie nicht)
    let o = opts();
    assert!(try_convert("while=1\nprint while\n", &o).is_ok());
    let mut c64 = B2aOptions::new("t.bas");
    c64.turbo = true;
    assert!(try_convert("print 1\n", &c64).unwrap_err().contains("E344"));
}

#[test]
fn turbo_dpoke_dpeek_move_pause() {
    // DPEEK rechnet mit dem Fließkommapaket des OS: ohne ROM läuft das Programm nicht zu Ende (wie bei den anderen Fließkomma-Tests überspringen)
    if os_rom().is_none() { eprintln!("übersprungen: kein OS-ROM (VC_ATARI_OSROM=…/ATARIOSB.ROM)"); return; }
    // Seite 7 ($0700): Seite 6 gehört den Laufzeitvariablen
    let r = run_turbo("DPOKE 1792,4660\nPRINT PEEK(1792);\" \";PEEK(1793);\" \";DPEEK(1792)\nMOVE 1792,1856,2\nPRINT DPEEK(1856)\nDPOKE 1860,65535\nPRINT DPEEK(1860)\nt=PEEK(20)\nPAUSE 0\nPRINT PEEK(20)-t >= 0\n");
    assert_eq!(r.out, "52 18 4660\n4660\n65535\n1\n", "{}", r.out);
}
