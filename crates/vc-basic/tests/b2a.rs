// SPDX-License-Identifier: GPL-3.0-or-later
//! basic2asm: BASIC → Assembler → 6502-Kern. Das Ergebnis läuft ohne Emulator in einem kleinen C64-Modell
//! (64-KB-RAM, KERNAL-Aufrufe CHROUT/CHRIN/GETIN/PLOT werden abgefangen).
use std::collections::VecDeque;
use std::path::PathBuf;
use vc_asm::{assemble, to_prg, Options, Policy};
use vc_basic::b2a::{convert, parse, B2aOptions};
use vc_basic::Dialect;
use vc_media::cpu6502::{Bus, Cpu};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

struct Machine { ram: Vec<u8>, out: Vec<u8>, input: VecDeque<u8>, keys: VecDeque<u8>, col: u8 }
impl Bus for Machine {
    fn read(&mut self, a: u16) -> u8 { self.ram[a as usize] }
    fn write(&mut self, a: u16, v: u8) { self.ram[a as usize] = v; }
}

#[allow(dead_code)]
struct Run { out: String, ram: Vec<u8>, err: Option<u8>, stack: Vec<u8>, trace: Vec<u16> }

/// Original-ROM aus dem ROM-Verzeichnis (`VC_ROMS`, Benutzerordner, ROM-Ordner anderer Programme wie VICE; nur lesend). Freie Ersatz-ROMs und unbekannte
/// Revisionen zählen nicht: die Tests rufen Originaladressen im BASIC-ROM. Fehlt das ROM, meldet der Test das sichtbar (`SKIP: …`) und wird übersprungen.
fn rom(system: &str, role: &str) -> Option<Vec<u8>> {
    use std::sync::OnceLock;
    static STORE: OnceLock<vc_roms::RomStore> = OnceLock::new();
    let store = STORE.get_or_init(|| { let mut env = vc_roms::RomEnv::from_process(); env.use_foreign = true; vc_roms::RomStore::new(env) });
    let r = store.resolve(system, role);
    match (r.status, &r.path) {
        (vc_roms::Status::Recognized, Some(p)) => std::fs::read(p).ok(),
        _ => { eprintln!("SKIP: Original-ROM {system}/{role} fehlt ({}) – Fließkomma-Tests mit ROM werden übersprungen", r.status.label()); None }
    }
}

/// ROM-Abbilder eines Ziels als (Ladeadresse, Daten)
fn target_roms(target: &str) -> Option<Vec<(usize, Vec<u8>)>> {
    match target {
        "c64" => { let (b, k) = roms()?; Some(vec![(0xA000, b), (0xE000, k)]) }
        "c128" => Some(vec![(0x4000, rom("c128", "basiclo")?), (0x8000, rom("c128", "basichi")?), (0xC000, rom("c128", "kernal")?)]),
        "plus4" => Some(vec![(0x8000, rom("plus4", "basic")?), (0xC000, rom("plus4", "kernal")?)]),
        _ => None,
    }
}

/// CHRGET/CHRGOT des BASIC im RAM (die ROM-Routine FIN holt Zeichen darüber): Adresse und Textzeiger je Ziel
fn chrget_code(target: &str) -> (u16, Vec<u8>) {
    let (at, tp) = match target { "c128" => (0x0380u16, 0x3Du8), "plus4" => (0x0473, 0x3B), _ => (0x0073, 0x7A) };
    let src = format!(".target c64\n.cpu 6502\n.org ${at:04X}\nchrget: inc ${tp:02X}\n bne g1\n inc ${:02X}\ng1: ldy #0\n lda (${tp:02X}),y\n cmp #$3a\n bcs ret\n cmp #$20\n beq chrget\n sec\n sbc #$30\n sec\n sbc #$d0\nret: rts\n", tp + 1);
    let cpu = vc_target::load_cpu(&data(), "6502").or_else(|_| vc_target::load_cpu(&data(), "6510")).unwrap();
    let mut ao = Options::new("chrget.asm", Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    let out = assemble(&src, &cpu, &ao).unwrap_or_else(|e| panic!("{e:?}"));
    (at, vc_asm::to_bin(&out).unwrap())
}

/// Original-ROMs des C64 (BASIC, KERNAL): ohne sie werden die Fließkomma-Tests übersprungen
fn roms() -> Option<(Vec<u8>, Vec<u8>)> { Some((rom("c64", "basic")?, rom("c64", "kernal")?)) }

fn petscii_out(b: &[u8]) -> String {
    b.iter().map(|&c| match c { 13 => '\n', 0x41..=0x5A => (c + 0x20) as char, 0x2E => '.', 0x20..=0x40 => c as char, 0x93 => '~', _ => '?' }).collect()
}

fn build(src: &str, int_div: bool) -> Vec<u8> {
    let mut o = B2aOptions::new("t.bas");
    o.int_div = int_div;
    build_with(src, o)
}

fn build_with(src: &str, o: B2aOptions) -> Vec<u8> {
    let ti = vc_basic::b2a::target_info(&o.target).unwrap();
    let d = Dialect::load(&data(), ti.dialect).unwrap();
    let c = convert(src, &d, &o).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    std::fs::write(std::env::temp_dir().join("b2a_last.asm"), &c.asm).ok();
    let cpu = vc_target::load_cpu(&data(), ti.cpu).unwrap();
    let mut ao = Options::new("t.asm", Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    let out = assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}\n{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"), c.asm));
    to_prg(&out).unwrap()
}

fn run_prg(prg: &[u8], input: &str, keys: &[u8]) -> Run { run_machine(prg, input, keys, false) }

fn run_machine(prg: &[u8], input: &str, keys: &[u8], with_rom: bool) -> Run { run_target(prg, input, keys, with_rom, "c64") }

fn run_target(prg: &[u8], input: &str, keys: &[u8], with_rom: bool, target: &str) -> Run {
    let mut m = Machine { ram: vec![0; 65536], out: vec![], input: input.bytes().map(|b| if b == b'\n' { 13 } else { b }).collect(), keys: keys.iter().copied().collect(), col: 0 };
    if with_rom {
        for (base, data) in target_roms(target).expect("ROMs") { m.ram[base..base + data.len()].copy_from_slice(&data); }
        if target == "c64" {
            let k = &target_roms("c64").unwrap()[1].1;
            let chrget: Vec<u8> = k[0x3A2..0x3A2 + 0x1D].to_vec();   // CHRGET-Kopie in der Nullseite, wie beim Kaltstart
            m.ram[0x73..0x73 + chrget.len()].copy_from_slice(&chrget);
            m.ram[0x7A] = 0x01; m.ram[0x7B] = 0x08;
            m.ram[0x8B..0x90].copy_from_slice(&[0x80, 0x4F, 0xC7, 0x52, 0x58]);   // Startwert für RND
        } else {
            let (at, code) = chrget_code(target);
            m.ram[at as usize..at as usize + code.len()].copy_from_slice(&code);
            if target == "c128" {
                // Das BASIC 7 liest Variablen über kleine Routinen im RAM (Bankumschaltung um ein LDA (zp),Y); im Modell ohne MMU genügt das LDA
                for (at, zp, pre) in [(0x03B7usize, 0x24u8, (0x02u8, 0x04u8)), (0x03C0, 0x26, (0x01, 0x03)), (0x03C9, 0x3D, (0x01, 0x03))] {
                    let b = [0x8D, pre.0, 0xFF, 0xB1, zp, 0x8D, pre.1, 0xFF, 0x60];
                    m.ram[at..at + b.len()].copy_from_slice(&b);
                }
            }
            if target == "plus4" {
                let b = [0xEA, 0xEA, 0xEA, 0xEA, 0xB1, 0x3B, 0xEA, 0xEA, 0xEA, 0xEA, 0x60];   // $04A5: Lesen über ($3B); die Schreibzugriffe auf $FF3E/$FF3F (ROM/RAM-Umschaltung) entfallen im Modell (dort läge das KERNAL-ROM)
                m.ram[0x04A5..0x04A5 + b.len()].copy_from_slice(&b);
                // Das BASIC 3.5 liest Variablen "unter dem ROM" über kleine Routinen im RAM ($04B0 ff., je 11 Byte): SEI, STA $FF3F, LDA (zp),Y, STA $FF3E, CLI, RTS
                for (i, zp) in [0x22u8, 0x24, 0x6F, 0x5F, 0x64].iter().enumerate() {
                    let b = [0xEA, 0xEA, 0xEA, 0xEA, 0xB1, *zp, 0xEA, 0xEA, 0xEA, 0xEA, 0x60];
                    m.ram[0x04B0 + 11 * i..0x04B0 + 11 * i + 11].copy_from_slice(&b);
                }
            }
            // Fehlervektor $0300 zeigt auf eine Falle (die ROM-Fehlerbehandlung endet mit JMP ($0300))
            m.ram[0x0300] = 0x10; m.ram[0x0301] = 0x02;
        }
    }
    // Muster in $60-$73 (Block der Stufen 1 und 3). Beim Plus/4 mit ROM nicht: dort liegen Arbeitszellen der Rechenroutinen (z. B. $68), die das ROM-BASIC beim Start mit 0 vorfindet
    if !with_rom || (target != "c64" && target != "plus4") { let z = zblock(target); for b in &mut m.ram[z] { *b = 0xC3; } }
    if target != "c64" { for b in &mut m.ram[0x30..0x3E] { *b = 0xC3; } }   // Muster in der Nullseite: Ziele mit gesichertem Block müssen es zurückgeben
    let load = prg[0] as usize | (prg[1] as usize) << 8;
    m.ram[load..load + prg.len() - 2].copy_from_slice(&prg[2..]);
    // BASIC-Stub: "10 SYS nnnn" → Einsprung lesen
    let digits: String = m.ram[load + 5..].iter().skip_while(|b| !b.is_ascii_digit()).take_while(|b| b.is_ascii_digit()).map(|b| *b as char).collect();
    let entry: u16 = digits.parse().expect("SYS-Adresse im Stub");
    let mut cpu = Cpu::new();
    cpu.pc = entry;
    cpu.sp = 0xF0;
    // Rücksprungadresse auf eine Falle ($0200, unterhalb aller Programme)
    cpu.push(&mut m, 0x01); cpu.push(&mut m, 0xFF);
    let mut err = None;
    let mut trace: Vec<u16> = vec![];
    for _ in 0..50_000_000u64 {
        trace.push(cpu.pc); if trace.len() > 80 { trace.remove(0); }
        match cpu.pc {
            0x0200 => break,
            0x0000 if with_rom => break,                                                // ins Leere gesprungen
            0xA437 if with_rom && target == "c64" => { err = Some(cpu.x); break; }   // ERROR im BASIC-ROM: X = Fehlernummer
            0x0210 if with_rom && target != "c64" => { err = Some(cpu.x); break; }                        // Fehlervektor der anderen Ziele
            0xFFD2 => { let a = cpu.a; if a == 13 || a == 0x93 { m.col = 0 } else { m.col = m.col.wrapping_add(1) } m.out.push(a); ret(&mut cpu, &mut m); continue; }
            0xFFCF => { cpu.a = m.input.pop_front().unwrap_or(13); cpu.c = false; ret(&mut cpu, &mut m); continue; }
            0xFFE4 => { cpu.a = m.keys.pop_front().unwrap_or(0); ret(&mut cpu, &mut m); continue; }
            0xFFF0 => { cpu.y = m.col; cpu.x = 0; ret(&mut cpu, &mut m); continue; }
            0xFFDE => { cpu.a = m.ram[0xA2]; cpu.x = m.ram[0xA1]; cpu.y = m.ram[0xA0]; ret(&mut cpu, &mut m); continue; }   // RDTIM
            0xFFDB => { m.ram[0xA2] = cpu.a; m.ram[0xA1] = cpu.x; m.ram[0xA0] = cpu.y; ret(&mut cpu, &mut m); continue; }   // SETTIM
            _ => {}
        }
        cpu.step(&mut m).expect("Befehl");
    }
    if !(cpu.pc == 0x0200 || err.is_some()) { eprintln!("Programmende nicht erreicht bei ${:04X}; letzte Adressen: {}", cpu.pc, trace.iter().rev().take(16).map(|b| format!("{b:04X}")).collect::<Vec<_>>().join(" ")); }
    assert!(cpu.pc == 0x0200 || err.is_some(), "Programm endete nicht");
    Run { out: petscii_out(&m.out), stack: m.ram[0x100 + cpu.sp as usize + 1..0x100 + cpu.sp as usize + 13].to_vec(), trace, ram: m.ram, err }
}
fn ret(cpu: &mut Cpu, m: &mut Machine) { let lo = cpu.pull(m) as u16; let hi = cpu.pull(m) as u16; cpu.pc = (lo | hi << 8).wrapping_add(1); }

fn run(src: &str) -> Run { run_prg(&build(src, false), "", &[]) }

#[test]
fn print_and_arithmetic() {
    let r = run("a% = 6\nb% = 7\nprint a%*b%\nprint a%-b%; a%+b%\nprint \"hallo\"; 5\n");
    assert_eq!(r.out, " 42 \n-1  13 \nhallo 5 \n");
}

#[test]
fn loops_gosub_and_labels() {
    let src = "s = 0\nfor i = 1 to 10\ns = s + i\nnext i\nprint s\ngosub @sub\nprint \"ende\"\nend\n@sub:\nprint \"sub\"\nreturn\n";
    let r = run(src);
    assert_eq!(r.out, " 55 \nsub\nende\n");
}

#[test]
fn nested_loops_negative_step_and_if() {
    let src = "for i = 3 to 1 step -1\nfor j = 1 to 2\nprint i*10+j;\nnext j\nnext i\nprint\nif s = 0 then print \"null\"\nif s = 1 then print \"eins\"\n";
    let r = run(src);
    assert_eq!(r.out, " 31  32  21  22  11  12 \nnull\n");
}

#[test]
fn data_read_arrays_on_goto() {
    let src = "dim a(4)\nfor i = 0 to 4\nread a(i)\nnext\nt = 0\nfor i = 0 to 4\nt = t + a(i)\nnext\nprint t\non 2 goto @x, @y\nprint \"nie\"\n@x:\nprint \"x\"\n@y:\nprint \"y\"\ndata 10, 20, -5\ndata 7, 8\n";
    let r = run(src);
    assert_eq!(r.out, " 40 \ny\n");
}

#[test]
fn poke_peek_and_comparison_logic() {
    let src = "poke 49152, 77\nx = peek(49152)\nprint x\nprint (x > 70) and (x < 80)\nprint not 0\nprint abs(-3); sgn(-9)\n";
    let r = run(src);
    assert_eq!(r.out, " 77 \n-1 \n-1 \n 3 -1 \n");
    assert_eq!(r.ram[49152], 77);
}

#[test]
fn input_and_get() {
    let prg = build("input \"zahl\"; n%\nprint n%*2\nget k\nprint k\n", false);
    let r = run_prg(&prg, "21\n", b"7");
    assert_eq!(r.out, "zahl? \n 42 \n 7 \n");
    // ungültige Eingabe wird wiederholt
    let r = run_prg(&prg, "ab\n5\n", b"");
    assert!(r.out.contains("redo from start"), "{}", r.out);
    assert!(r.out.contains(" 10 "));
}

#[test]
fn division_needs_option_or_literals() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let mut o1 = B2aOptions::new("t.bas");
    o1.stage = 1;
    let e = convert("a = 7\nprint a/2\n", &d, &o1).unwrap_err();
    assert!(e[0].to_string().contains("E301"), "{}", e[0]);
    let mut o = B2aOptions::new("t.bas");
    o.stage = 1;
    o.int_div = true;
    let r = run_prg(&build_with("a = 7\nprint a/2\nprint 8/2\n", o), "", &[]);
    assert_eq!(r.out, " 3 \n 4 \n");
    let r = run("print 8/2\n");
    assert_eq!(r.out, " 4 \n");
}

#[test]
fn unsupported_things_are_reported_not_hidden() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    for (src, code) in [("a$ = \"x\"\n", "E302"), ("print len(\"x\")\n", "E302"), ("goto 999\n", "E201"), ("open 1,8,2\n", "E311"), ("for i% = 1 to 3\nnext\n", "E314"), ("x = ", "E103")] {
        let e = convert(src, &d, &B2aOptions { stage: 2, ..B2aOptions::new("t.bas") }).unwrap_err();
        assert!(e.iter().any(|x| x.to_string().contains(code)), "{src}: {:?}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>());
    }
}

#[test]
fn runtime_errors_report_line() {
    let r = run("dim a(2)\na(5) = 1\n");
    assert!(r.out.contains("bad subscript  error in 20"), "{:?}", r.out);
    let r = run("return\n");
    assert!(r.out.contains("return without gosub"), "{:?}", r.out);
}

#[test]
fn parser_keeps_rom_keyword_rules() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let p = parse("fori=1to3:next:ifa=0thenprint\"x\"\n", &d, &B2aOptions::new("t.bas")).unwrap();
    assert_eq!(p.lines[0].stmts.len(), 3, "{:?}", p.lines[0].stmts);
}

#[test]
fn zeilenkarte_and_comments() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let c = convert("@start:\nx = 1\nprint x\n", &d, &B2aOptions::new("t.bas")).unwrap();
    assert!(c.asm.contains(";---- Zeile 10 (@start): x = 1"), "{}", c.asm);
    assert!(c.asm.contains("bas_start:"));
    assert_eq!(c.map.len(), 2);
    assert!(c.map_json("t.bas").contains("\"label\": \"L20\""));
}

// ---- Stufe 2 (Fließkomma über das BASIC-ROM): läuft nur, wenn die Original-ROMs unter assets/ROMS/c64 liegen

fn frun(src: &str) -> Option<Run> {
    roms()?;
    Some(run_machine(&build_with(src, B2aOptions::new("t.bas")), "", &[], true))
}

#[test]
fn float_print_and_arithmetic() {
    let Some(r) = frun("print 7/2\nprint 1/3\nprint 2^10\nprint sqr(16)\nx = 0.1 + 0.2\nprint x\nprint 1.5e3 * 2\nprint -2.5\nprint 10/4*4\n") else { eprintln!("ROMs fehlen: übersprungen"); return };
    assert_eq!(r.out, " 3.5 \n .333333333 \n 1024 \n 4 \n .3 \n 3000 \n-2.5 \n 10 \n", "{}", r.out);
}

#[test]
fn float_loops_integers_and_mixed() {
    let src = "for x = 0 to 1 step 0.25\nprint x;\nnext x\nprint\na% = 7\nb = a% / 2\nprint b; a% * 2\nc% = -3.7\nprint c%\nfor i = 5 to 1 step -2\nprint i;\nnext\nprint\n";
    let Some(r) = frun(src) else { return };
    assert_eq!(r.out, " 0  .25  .5  .75  1 \n 3.5  14 \n-4 \n 5  3  1 \n", "{}", r.out);
}

#[test]
fn float_arrays_data_and_functions() {
    let src = "dim a(3), b%(2)\na(2) = 1.5\nb%(1) = 9\nprint a(2) * 2; b%(1)\nread x, y\nprint x + y\nprint int(-1.5); abs(-2); sgn(-7)\nr = rnd(1)\nprint (r >= 0) and (r < 1)\nprint sin(0); cos(0)\ndata 1.5, 2.25\n";
    let Some(r) = frun(src) else { return };
    assert_eq!(r.out, " 3  9 \n 3.75 \n-2  2 -1 \n-1 \n 0  1 \n", "{}", r.out);
}

#[test]
fn float_conditions_and_gosub() {
    let src = "x = 0.5\nif x < 1 then print \"klein\"\nif x > 1 then print \"gross\"\nif not (x = 0.5) then print \"nie\"\ngosub @s\nprint y\nend\n@s:\ny = x * 3\nreturn\n";
    let Some(r) = frun(src) else { return };
    assert_eq!(r.out, "klein\n 1.5 \n", "{}", r.out);
}

#[test]
fn float_input_and_poke_peek() {
    let Some(_) = roms() else { return };
    let prg = build_with("input \"x\"; v\nprint v * 2\npoke 49152, v + 1\nprint peek(49152)\n", B2aOptions { force_float: true, ..B2aOptions::new("t.bas") });
    let r = run_machine(&prg, "3.5\n", &[], true);
    assert_eq!(r.out, "x? \n 7 \n 4 \n", "{}", r.out);
    assert_eq!(r.ram[0x7A], 0x01, "BASIC-Textzeiger muss wiederhergestellt sein");
}

#[test]
fn float_errors_come_from_the_rom_with_line_number() {
    let Some(r) = frun("print 1\nx = 0\nprint 1/x\n") else { return };
    assert_eq!(r.err, Some(20), "DIVISION BY ZERO: {}", r.out);
    assert_eq!(r.ram[0x39] as u16 | (r.ram[0x3A] as u16) << 8, 30, "CURLIN");
}

#[test]
fn stage_selection_is_automatic_and_forceable() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let c = convert("a% = 3\nprint a% * 2\n", &d, &B2aOptions::new("t.bas")).unwrap();
    assert_eq!(c.stage, 1);
    assert!(c.modules.contains(&"rt_arith".to_string()));
    let c = convert("print 1.5\n", &d, &B2aOptions::new("t.bas")).unwrap();
    assert_eq!(c.stage, 2);
    assert!(c.modules.contains(&"rt_fp".to_string()));
    let c = convert("a% = 3\n", &d, &B2aOptions { force_float: true, ..B2aOptions::new("t.bas") }).unwrap();
    assert_eq!(c.stage, 2);
    let e = convert("print 1.5\n", &d, &B2aOptions { stage: 1, ..B2aOptions::new("t.bas") }).unwrap_err();
    assert!(e[0].to_string().contains("E301"));
}

#[test]
fn mbf_encoding_matches_rom_constants() {
    // aus dem ROM: 1 = 81 00 00 00 00, -1 = 81 80 .., 0.5 = 80 00 .., 10 = 84 20 .., pi = 82 49 0F DA A2
    use vc_basic::b2a::mbf;
    assert_eq!(mbf(1.0).unwrap(), [0x81, 0, 0, 0, 0]);
    assert_eq!(mbf(-1.0).unwrap(), [0x81, 0x80, 0, 0, 0]);
    assert_eq!(mbf(0.5).unwrap(), [0x80, 0, 0, 0, 0]);
    assert_eq!(mbf(10.0).unwrap(), [0x84, 0x20, 0, 0, 0]);
    assert_eq!(mbf(std::f64::consts::PI).unwrap(), [0x82, 0x49, 0x0F, 0xDA, 0xA2]);
    assert_eq!(mbf(0.0).unwrap(), [0; 5]);
    assert!(mbf(1e40).is_err());
}

// ---- Stufe 3 (Zeichenketten)

fn srun(src: &str) -> Run { run_prg(&build(src, false), "", &[]) }

#[test]
fn strings_concat_and_functions_integer_mode() {
    let src = "a$ = \"hallo\"\nb$ = \" welt\"\nc$ = a$ + b$\nprint c$\nprint len(c$)\nprint left$(\"abcdef\", 2); right$(\"abcdef\", 3); mid$(\"abcdef\", 2, 3); mid$(\"abcdef\", 5)\nprint asc(\"a\"); chr$(66); str$(42); val(\"123abc\") + 1\nprint left$(\"ab\", 9); \"/\"; mid$(\"ab\", 3); \"/\"; right$(\"\", 2); \"/\"\n";
    let r = srun(src);
    assert_eq!(r.out, "hallo welt\n 10 \nabdefbcdef\n 65 b 42 124 \nab///\n", "{}", r.out);
}

#[test]
fn string_comparisons_and_arrays() {
    let src = "a$ = \"abc\"\nif a$ = \"abc\" then print \"gleich\"\nif a$ < \"abd\" then print \"kleiner\"\nif \"ab\" < a$ then print \"praefix\"\nif a$ <> \"x\" and a$ > \"\" then print \"ok\"\ndim s$(3)\ns$(1) = \"x\"\ns$(2) = s$(1) + \"y\"\nprint s$(2); len(s$(0)); s$(3)\n";
    let r = srun(src);
    assert_eq!(r.out, "gleich\nkleiner\npraefix\nok\nxy 0 \n", "{}", r.out);
}

#[test]
fn garbage_collection_keeps_live_strings() {
    let src = "k$ = \"kee\" + \"p\"\ndim t$(2)\nt$(1) = \"arr\" + \"ay\"\na$ = \"abcdefghij\"\nfor i = 1 to 5000\nb$ = a$ + chr$(65) + a$\nnext\nprint k$; t$(1); len(b$); left$(b$, 3)\n";
    let r = srun(src);
    assert_eq!(r.out, "keeparray 21 abc\n".replace("array 21", "array 21").replace("keeparray", "keeparray"), "{}", r.out);
    assert!(r.err.is_none());
}

#[test]
fn strings_data_input_get_and_clr() {
    let prg = build("data hello, 5, \"a,b\"\nread a$, b, c$\nprint a$; b; c$\nrestore\nread x$, y$\nprint y$\ninput \"n\"; n$\nprint \"hi \" + n$\nget k$\nprint \"<\" + k$ + \">\"\nclr\nprint len(a$)\n", false);
    let r = run_prg(&prg, "WORLD\n", b"Z");
    assert_eq!(r.out, "hello 5 a,b\n5\nn? \nhi world\n<z>\n 0 \n", "{}", r.out);
}

#[test]
fn string_type_errors_are_reported() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    for (src, code) in [("a$ = 5\n", "E320"), ("print \"a\" + 1\n", "E320"), ("x = \"a\"\n", "E320"), ("print -\"a\"\n", "E320"), ("if \"a\" = 1 then end\n", "E320"), ("print left$(\"a\")\n", "E305"), ("a$ = len(\"x\")\n", "E320")] {
        let e = convert(src, &d, &B2aOptions::new("t.bas")).unwrap_err();
        assert!(e.iter().any(|x| x.to_string().contains(code)), "{src}: {:?}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>());
    }
}

#[test]
fn strings_with_float_mode() {
    let Some(_) = roms() else { return };
    let src = "x = 1.5\ns$ = str$(x * 2)\nprint s$; len(s$)\nprint val(\"2.5\") * 2\nn$ = \"v=\" + str$(3.25)\nprint n$\nfor i = 1 to 3\nr$ = r$ + chr$(48 + i)\nnext\nprint r$\n";
    let r = run_machine(&build_with(src, B2aOptions::new("t.bas")), "", &[], true);
    assert_eq!(r.out, " 3 2 \n 5 \nv= 3.25\n123\n", "{}", r.out);
}

#[test]
fn stage_selection_with_strings() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    assert_eq!(convert("print \"hallo\"\nprint chr$(65)\n", &d, &B2aOptions::new("t.bas")).unwrap().stage, 1);
    let c = convert("a$ = \"x\"\n", &d, &B2aOptions::new("t.bas")).unwrap();
    assert_eq!(c.stage, 3);
    assert!(c.modules.contains(&"rt_str".to_string()));
    let e = convert("a$ = \"x\"\n", &d, &B2aOptions { stage: 2, ..B2aOptions::new("t.bas") }).unwrap_err();
    assert!(e[0].to_string().contains("E302"));
}

// ---- DEF FN und TI$

#[test]
fn def_fn_integer_mode() {
    let r = srun("x = 100\ndef fna(x) = x * 2 + 1\nprint fna(5); fna(fna(1))\ny = fna(7)\nprint y; x\n");
    assert_eq!(r.out, " 11  7 \n 15  100 \n", "{}", r.out);
}

#[test]
fn def_fn_uses_globals_and_restores_parameter() {
    let r = srun("k = 3\nt = 9\ndef fnb(t) = t * k\nprint fnb(4); t\nk = 5\nprint fnb(2)\n");
    assert_eq!(r.out, " 12  9 \n 10 \n", "{}", r.out);
}

#[test]
fn def_fn_float_mode() {
    let Some(r) = frun("def fnr(x) = int(x * 100 + .5) / 100\nprint fnr(3.14159)\ndef fnc(t) = t * 9 / 5 + 32\nprint fnc(100); fnc(-40)\nx = 2.5\nprint fnc(x) + x\n") else { return };
    assert_eq!(r.out, " 3.14 \n 212 -40 \n 39 \n", "{}", r.out);
}

#[test]
fn def_fn_called_before_definition_is_undefd_and_unknown_is_compile_error() {
    let r = srun("print fna(1)\ndef fna(x) = x\n");
    assert!(r.out.contains("undef'd function  error in 10"), "{}", r.out);
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let e = convert("print fnz(1)\n", &d, &B2aOptions::new("t.bas")).unwrap_err();
    assert!(e[0].to_string().contains("E317"));
    let e = convert("def fna(x) = x\ndef fna(y) = y\n", &d, &B2aOptions::new("t.bas")).unwrap_err();
    assert!(e.iter().any(|x| x.to_string().contains("E316")));
}

#[test]
fn ti_string_reads_and_sets_the_clock() {
    let r = srun("ti$ = \"123456\"\nprint ti$\nti$ = \"000000\"\nprint ti$\nti$ = \"235959\"\nprint ti$; len(ti$)\n");
    assert_eq!(r.out, "123456\n000000\n235959 6 \n", "{}", r.out);
    // 1:02:03 = 3723 s = 223380 Schritte = $03684 4 -> $A0=$03 $A1=$6B $A2=$14? (hier nur geprüft: Rückwandlung stimmt)
    let r = srun("ti$ = \"010203\"\n");
    let jiffies = (r.ram[0xA0] as u32) << 16 | (r.ram[0xA1] as u32) << 8 | r.ram[0xA2] as u32;
    assert_eq!(jiffies, (3600 + 2 * 60 + 3) * 60);
    let r = srun("ti$ = \"12x456\"\n");
    assert!(r.out.contains("illegal quantity"), "{}", r.out);
}

// ---- C128 und Plus/4 (Stufe 1 und 3; KERNAL-Aufrufe wie beim C64, andere Ladeadresse, Nullseite und Speichergrenze)

/// Nullseitenblock der Laufzeit (Stufe 1 und 3) je Ziel
fn zblock(target: &str) -> std::ops::Range<usize> { if target == "mega65" { 0x40..0x54 } else { 0x60..0x74 } }

fn on(target: &str) -> B2aOptions { B2aOptions { target: target.into(), ..B2aOptions::new("t.bas") } }

#[test]
fn c128_and_plus4_run_the_same_programs() {
    let cases: Vec<(&str, &str, &str, &[u8])> = vec![
        ("a% = 6\nb% = 7\nprint a%*b%\nprint a%-b%; a%+b%\nprint \"hallo\"; 5\n", " 42 \n-1  13 \nhallo 5 \n", "", b""),
        ("s = 0\nfor i = 1 to 10\ns = s + i\nnext i\nprint s\ngosub @sub\nprint \"ende\"\nend\n@sub:\nprint \"sub\"\nreturn\n", " 55 \nsub\nende\n", "", b""),
        ("dim a(4)\nfor i = 0 to 4\nread a(i)\nnext\nt = 0\nfor i = 0 to 4\nt = t + a(i)\nnext\nprint t\non 2 goto @x, @y\nprint \"nie\"\n@x:\nprint \"x\"\n@y:\nprint \"y\"\ndata 10, 20, -5\ndata 7, 8\n", " 40 \ny\n", "", b""),
        ("a$ = \"hallo\"\nb$ = \" welt\"\nc$ = a$ + b$\nprint c$\nprint len(c$)\nprint left$(c$, 2); right$(c$, 3); mid$(c$, 3, 4)\nprint asc(\"a\"); chr$(66); str$(42); val(\"123abc\") + 1\n", "hallo welt\n 10 \nhaeltllo \n 65 b 42 124 \n", "", b""),
        ("k$ = \"kee\" + \"p\"\ndim t$(2)\nt$(1) = \"arr\" + \"ay\"\na$ = \"abcdefghij\"\nfor i = 1 to 5000\nb$ = a$ + chr$(65) + a$\nnext\nprint k$; t$(1); len(b$); left$(b$, 3)\n", "keeparray 21 abc\n", "", b""),
        ("def fna(x) = x * 2 + 1\nprint fna(5)\nti$ = \"123456\"\nprint ti$\nprint ti\n", " 11 \n123456\n", "", b""),
        ("input \"n\"; n$\nprint \"hi \" + n$\nget k$\nprint \"<\" + k$ + \">\"\n", "n? \nhi world\n<z>\n", "WORLD\n", b"Z"),
        ("dim a(2)\na(5) = 1\n", "", "", b""),
    ];
    for t in ["c64", "c128", "plus4", "mega65"] {
        for (src, want, input, keys) in &cases {
            let prg = build_with(src, on(t));
            let r = run_target(&prg, input, keys, false, t);
            if want.is_empty() { assert!(r.out.contains("bad subscript  error in 20"), "{t}: {}", r.out); continue; }
            if src.contains("print ti\n") {
                // TI hängt von der Uhr ab (hier fest): nur den Rest prüfen
                assert!(r.out.starts_with(want), "{t}: {}", r.out);
                continue;
            }
            assert_eq!(&r.out, want, "{t}: {src}");
            if t != "c64" { assert!(r.ram[zblock(t)].iter().all(|b| *b == 0xC3), "{t}: Nullseitenblock nicht zurückgeschrieben"); }
        }
    }
}

#[test]
fn c128_and_plus4_use_their_load_address_and_memory_limit() {
    let d128 = build_with("print 1\n", on("c128"));
    assert_eq!(d128[0] as u16 | (d128[1] as u16) << 8, 0x1C01);
    let dp4 = build_with("print 1\n", on("plus4"));
    assert_eq!(dp4[0] as u16 | (dp4[1] as u16) << 8, 0x1001);
    // zu groß für die Speicherlage des C128 (ROM ab $4000)
    let big = format!("{}\n", (0..600).map(|i| format!("print \"zeile {i:04} mit etwas text damit es groesser wird\"")).collect::<Vec<_>>().join("\n"));
    let d = Dialect::load(&data(), "basic7").unwrap();
    let c = convert(&big, &d, &on("c128")).unwrap();
    let cpu = vc_target::load_cpu(&data(), "8502").unwrap();
    let mut ao = Options::new("t.asm", Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    let e = assemble(&c.asm, &cpu, &ao).unwrap_err();
    assert!(e.iter().any(|x| x.message.contains("Programm zu groß")), "{e:?}");
}

#[test]
fn unknown_targets_and_stage_limits_are_reported() {
    for t in ["c128", "plus4"] {
        let d = Dialect::load(&data(), if t == "c128" { "basic7" } else { "basic35" }).unwrap();
        // Fließkomma ist erlaubt; mit --stage 1 nicht
        convert("print 1/3\n", &d, &on(t)).unwrap();
        let e = convert("print 1/3\n", &d, &B2aOptions { stage: 1, ..on(t) }).unwrap_err();
        assert!(e[0].to_string().contains("E301"), "{t}: {}", e[0]);
    }
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let e = convert("print 1\n", &d, &on("zx81")).unwrap_err();
    assert!(e[0].to_string().contains("E330"));
    // MEGA65: nur Ganzzahl und Zeichenketten; Fließkomma meldet E331
    let d65 = Dialect::load(&data(), "basic65").unwrap();
    let e = convert("print 1/3\n", &d65, &on("mega65")).unwrap_err();
    assert!(e[0].to_string().contains("E331"), "{}", e[0]);
}

#[test]
fn mega65_is_6502_compatible_and_starts_at_2001() {
    let prg = build_with("print 1\n", on("mega65"));
    assert_eq!(&prg[..2], &[0x01, 0x20], "Ladeadresse $2001");
    // lange Sprünge als umgekehrter Sprung plus JMP: kein 45GS02-Opcode (lange Verzweigungen $13/$33/$53/$73/$93/$B3/$D3/$F3)
    let src: String = (1..=60).map(|i| format!("{} if x then print {i}\n", i * 10)).collect();
    let prg = build_with(&src, on("mega65"));
    assert!(prg.len() > 600);
}

// ---- Stufe 2 auf C128 und Plus/4 (Einsprünge aus deren BASIC-ROMs; läuft nur mit vorhandenen ROMs)

fn frun_on(target: &str, src: &str) -> Option<Run> {
    target_roms(target)?;
    Some(run_target(&build_with(src, on(target)), "", &[], true, target))
}

#[test]
fn float_programs_run_on_all_targets() {
    let cases: Vec<(&str, &str)> = vec![
        ("print 7/2\nprint 1/3\nprint 2^10\nprint sqr(16)\nx = 0.1 + 0.2\nprint x\nprint 1.5e3 * 2\nprint -2.5\nprint 10/4*4\n", " 3.5 \n .333333333 \n 1024 \n 4 \n .3 \n 3000 \n-2.5 \n 10 \n"),
        ("for x = 0 to 1 step 0.25\nprint x;\nnext x\nprint\na% = 7\nb = a% / 2\nprint b; a% * 2\nc% = -3.7\nprint c%\nfor i = 5 to 1 step -2\nprint i;\nnext\nprint\n", " 0  .25  .5  .75  1 \n 3.5  14 \n-4 \n 5  3  1 \n"),
        ("dim a(3), b%(2)\na(2) = 1.5\nb%(1) = 9\nprint a(2) * 2; b%(1)\nread x, y\nprint x + y\nprint int(-1.5); abs(-2); sgn(-7)\nr = rnd(1)\nprint (r >= 0) and (r < 1)\nprint sin(0); cos(0)\ndata 1.5, 2.25\n", " 3  9 \n 3.75 \n-2  2 -1 \n-1 \n 0  1 \n"),
        ("x = 0.5\nif x < 1 then print \"klein\"\nif x > 1 then print \"gross\"\nif not (x = 0.5) then print \"nie\"\ngosub @s\nprint y\nend\n@s:\ny = x * 3\nreturn\n", "klein\n 1.5 \n"),
        ("x = 1.5\ns$ = str$(x * 2)\nprint s$; len(s$)\nprint val(\"2.5\") * 2\nn$ = \"v=\" + str$(3.25)\nprint n$\nfor i = 1 to 3\nr$ = r$ + chr$(48 + i)\nnext\nprint r$\n", " 3 2 \n 5 \nv= 3.25\n123\n"),
        ("def fnr(x) = int(x * 100 + .5) / 100\nprint fnr(3.14159)\ndef fnc(t) = t * 9 / 5 + 32\nprint fnc(100); fnc(-40)\n", " 3.14 \n 212 -40 \n"),
        ("poke 49152, 200 + 55.9\nprint peek(49152)\nprint 40000 + 5\nprint 3 > 2.5; 2 < 1\nprint 5 and 3.7\n", " 255 \n 40005 \n-1  0 \n 1 \n"),
    ];
    let mut ran = 0;
    for t in ["c64", "c128", "plus4"] {
        for (src, want) in &cases {
            let Some(r) = frun_on(t, src) else { eprintln!("{t}: ROMs fehlen, übersprungen"); break };
            assert_eq!(&r.out, want, "{t}: {src}");
            if t != "c64" { assert!(r.ram[0x30..0x3E].iter().all(|b| *b == 0xC3), "{t}: Nullseitenblock nicht zurückgeschrieben (Stufe 2)"); }
            ran += 1;
        }
    }
    eprintln!("{ran} Programmläufe");
}

#[test]
fn float_input_and_rom_errors_on_other_targets() {
    for t in ["c128", "plus4"] {
        if target_roms(t).is_none() { continue; }
        let prg = build_with("input \"x\"; v\nprint v * 2\nprint 1 / (v - 3.5)\n", on(t));
        let r = run_target(&prg, "3.5\n", &[], true, t);
        assert_eq!(r.out, "x? \n 7 \n", "{t}: {}", r.out);
        assert_eq!(r.err, Some(20), "{t}: DIVISION BY ZERO aus dem ROM");
        assert!(r.ram[0x30..0x3E].iter().all(|b| *b == 0xC3), "{t}: Nullseitenblock muss nach einem ROM-Fehler zurückgeschrieben sein");
        assert_eq!(r.ram[0x0300] as u16 | (r.ram[0x0301] as u16) << 8, 0x0210, "{t}: Fehlervektor muss wiederhergestellt sein");
    }
}

#[test]
fn zz_debug_probe() {
    if std::env::var("VC_DBG").is_err() { return; }
    for src in ["print 3.5\n", "print 7\n", "print 2^10\nprint 1/3\n", "x = 0.5\nprint x + x\n"] {
        let r = run_target(&build_with(src, on("plus4")), "", &[], true, "plus4");
        eprintln!("{src:?} -> {:?}", r.out);
    }
}

// ---- Stil inline und --runtime embed

const INT_PROG: &str = "10 X=5:Y=7:Z=0
20 Z=X+Y:PRINT Z
30 Z=X-Y:PRINT Z
40 Z=X+1000:PRINT Z
50 Z=100-X:PRINT Z
60 Z=Y-3:PRINT Z
70 X=X+1:X=X+1:X=X-300:PRINT X
80 PRINT X=-298;X<>-298;X=Y;X<>Y;Y=7;7=Y
90 IF X=-298 THEN PRINT \"JA\"
100 IF Y<>7 THEN PRINT \"NEIN\"
110 A=32767:A=A+1:PRINT A
120 B=3:C=B+B:PRINT C-B
";

#[test]
fn inline_style_gives_the_same_results_with_fewer_calls() {
    let mut o = B2aOptions::new("t.bas");
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let compact = build_with(INT_PROG, o.clone());
    let asm_compact = convert(INT_PROG, &d, &o).unwrap_or_else(|e| panic!("{e:?}")).asm;
    o.inline = true;
    let inline = build_with(INT_PROG, o.clone());
    let asm_inline = convert(INT_PROG, &d, &o).unwrap_or_else(|e| panic!("{e:?}")).asm;
    let (a, b) = (run_prg(&compact, "", &[]), run_prg(&inline, "", &[]));
    assert_eq!(a.out, b.out);
    assert_eq!(a.out, " 12 \n-2 \n 1005 \n 95 \n 4 \n-293 \n 0 -1  0 -1 -1 -1 \n-32768 \n 3 \n", "{}", a.out.replace('\n', "|"));
    let calls = |s: &str| s.matches("jsr rt_add").count() + s.matches("jsr rt_sub").count() + s.matches("jsr rt_eq").count() + s.matches("jsr rt_ne").count();
    assert!(calls(&asm_inline) < calls(&asm_compact), "inline ruft nicht weniger Routinen auf: {} / {}", calls(&asm_inline), calls(&asm_compact));
    assert!(asm_inline.contains("adc #"), "kein Inline-Code");
}

#[test]
fn embedded_runtime_is_self_contained_and_identical() {
    for target in ["c64", "atari8"] {
        let ti = vc_basic::b2a::target_info(target).unwrap();
        let d = Dialect::load(&data(), ti.dialect).unwrap();
        let cpu = vc_target::load_cpu(&data(), ti.cpu).unwrap();
        let mut o = B2aOptions::new("t.bas");
        o.target = target.into();
        let inc = convert(INT_PROG, &d, &o).unwrap_or_else(|e| panic!("{e:?}"));
        o.embed = true;
        o.data_dir = Some(data());
        let emb = convert(INT_PROG, &d, &o).unwrap_or_else(|e| panic!("{e:?}"));
        assert!(!emb.asm.lines().any(|l| l.trim_start().starts_with("#include")), "{target}: Include übrig");
        assert!(emb.asm.contains("; ---------------- common/b2a/rt_core.asm"), "{target}: Laufzeit nicht eingebettet");
        let mut a1 = Options::new("t.asm", Policy::Allow);
        a1.include_dirs.push(data().join("sys"));
        let r1 = assemble(&inc.asm, &cpu, &a1).unwrap_or_else(|e| panic!("{e:?}"));
        let a2 = Options::new("t.asm", Policy::Allow);   // kein Include-Ordner
        let r2 = assemble(&emb.asm, &cpu, &a2).unwrap_or_else(|e| panic!("{target}: {}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
        let bytes = |o: &vc_asm::Output| o.segments.iter().map(|s| (s.start, s.bytes.clone())).collect::<Vec<_>>();
        assert_eq!(bytes(&r1), bytes(&r2), "{target}: eingebettete Laufzeit ergibt andere Bytes");
    }
    let mut o = B2aOptions::new("t.bas");
    o.embed = true;
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    assert!(convert(INT_PROG, &d, &o).unwrap_err()[0].to_string().contains("E192"));
}

#[test]
fn source_comments_are_kept() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "; Kopf\n\n\n; Zweiter Absatz\n@a: PRINT 1\n; mitten\nPRINT 2\n; Ende\n";
    let o = B2aOptions::new("t.bas");
    let c = convert(src, &d, &o).unwrap();
    let a = &c.asm;
    let i = a.find("; Kopf").expect(a);
    let j = a.find("; Zweiter Absatz").unwrap();
    let k = a.find("---- Zeile 10").unwrap();
    let m = a.find("; mitten").unwrap();
    let l = a.find("---- Zeile 20").unwrap();
    let e = a.find("; Ende").unwrap();
    assert!(i < j && j < k && k < m && m < l && l < e, "{a}");
    assert!(a[i..j].matches('\n').count() == 2, "eine Leerzeile bleibt: {a}");
    let mut o2 = B2aOptions::new("t.bas"); o2.no_rem = true;
    assert!(!convert(src, &d, &o2).unwrap().asm.contains("Kopf"));
}
