// SPDX-License-Identifier: GPL-3.0-or-later
//! Orakel: ein Atari-BASIC-Programm läuft im echten BASIC-Modul (ATARIBAS.ROM) in atari800; gelesen wird der Bildschirmspeicher.
//! Nur mit `VC_EMU_TESTS=1` (und atari800 samt ROMs in ~/.atari800.cfg), immer mit `--test-threads=1` (je Test startet ein Emulator; parallel stören sie sich).
//! Die Turbo-BASIC-XL-Tests brauchen das Diskettenabbild unter assets/disks/atari8/Programmiersprachen (VC_TURBO_ATR); die Datei-Tests (D:) nehmen daraus DOS.SYS. Die Tests hier messen Regeln von Atari-BASIC, die basic2asm nachbildet.
use std::path::PathBuf;
use std::time::Duration;
use vc_basic::atari::{tokenize_text, AtariDialect};
use vc_emu::{Atari800, Backend};

const STOP: u16 = 0xE45F;   // SYSVBV: das Programm endet mit A=USR(58463), der Haltepunkt liegt dort

fn screen_lines(mem: &[u8]) -> Vec<String> {
    mem.chunks(40).map(|l| l.iter().map(|&b| match b & 0x7F { c @ 0x00..=0x3F => (c + 0x20) as char, c @ 0x40..=0x5F => (c - 0x40) as char, c @ 0x60..=0x7F => c as char, _ => '?' }).collect::<String>().trim_end().to_string()).collect()
}

fn gfx_state_mode(a: &mut Atari800, mode: u8) -> Vec<u8> {
    // Hauptbildschirm aus der Anzeigeliste (SDLSTL $0230): drei Leerzeilen, dann die Modus-Zeile mit LMS-Adresse (SAVMSC zeigt nach INPUT auf das Textfenster)
    let dl = a.read_mem(0x0230, 2).unwrap();
    let d = dl[0] as u16 | (dl[1] as u16) << 8;
    let sav = a.read_mem(d + 4, 2).unwrap();
    let size = match mode { 1 => 480, 2 | 3 => 240, 4 => 480, 5 => 960, 6 => 1920, 7 => 3840, 8..=11 => 7680, _ => 960 };
    let mut v = a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, size).unwrap();
    v.extend(a.read_mem(0x02C4, 5).unwrap());
    v
}

/// Grafikzustand nach dem Lauf: Bildschirmspeicher (Größe nach Modus, ab SAVMSC), Farbschatten COLOR0-4 ($02C4-$02C8) und Cursor/Modus ($54-$5C)
fn gfx_state(a: &mut Atari800) -> Vec<u8> {
    let sav = a.read_mem(0x58, 2).unwrap();
    let mode = a.read_mem(0x57, 1).unwrap()[0] & 0x0F;   // DINDEX
    let size = match mode { 1 => 480, 2 | 3 => 240, 4 => 480, 5 => 960, 6 => 1920, 7 => 3840, 8..=11 => 7680, _ => 960 };   // Bytes des Hauptbildschirms (ohne Textfenster)
    let mut v = a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, size).unwrap();
    v.extend(a.read_mem(0x02C4, 5).unwrap());
    v.extend(a.read_mem(0x54, 9).unwrap());
    v
}

/// Führt die Zeilen (mit Zeilennummern) im echten Atari-BASIC aus; Fehler fängt `TRAP` und druckt `ERR<Code> LINE<Zeile>`.
pub fn run_basic(body: &str) -> Vec<String> { run_basic_full(body).0 }

pub fn run_basic_full(body: &str) -> (Vec<String>, Vec<u8>) {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = AtariDialect::load(&data, "atari_basic").unwrap();
    let src = format!("1 TRAP 31000\n{body}\n30999 A=USR(58463)\n31000 ? \"ERR\";PEEK(195);\" LINE\";PEEK(187)*256+PEEK(186)\n31001 A=USR(58463)\n");
    let prog = tokenize_text(&src, &d, "oracle.lst").unwrap_or_else(|e| panic!("{e:?}"));
    let dir = std::env::temp_dir().join("vc_atari_oracle");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("ORACLE.BAS");
    std::fs::write(&f, prog.save_file(&d)).unwrap();
    let mut a = Atari800::launch(f.to_str().unwrap(), None).expect("atari800 starten");
    a.set_breakpoint(STOP).unwrap();
    let hit = a.run_until(STOP, Duration::from_secs(40)).expect("run_until");
    let (lines, gfx) = if hit {
        let sav = a.read_mem(0x58, 2).unwrap();
        (screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap()), gfx_state(&mut a))
    } else { (vec!["<Haltepunkt nicht erreicht>".into()], vec![]) };
    a.stop();
    (lines.into_iter().filter(|l| !l.is_empty()).collect(), gfx)
}

fn enabled() -> bool { std::env::var("VC_EMU_TESTS").is_ok() }

#[test]
fn oracle_smoke() {
    if !enabled() { return; }
    let out = run_basic("10 PRINT 1/3\n20 PRINT \"HALLO\"");
    eprintln!("{out:#?}");
    assert!(out.iter().any(|l| l.contains("HALLO")), "{out:?}");
}

/// Dasselbe Programm (Zeilen mit Nummern) nach basic2asm übersetzt, in atari800 mit dem echten OS gestartet; Bildschirmzeilen ohne linken Rand
fn run_converted(body: &str) -> Vec<String> { run_converted_full(body).0 }

fn run_converted_full(body: &str) -> (Vec<String>, Vec<u8>) { run_converted_opts(body, false) }

fn run_converted_opts(body: &str, turbo: bool) -> (Vec<String>, Vec<u8>) {
    use vc_basic::b2a::{convert, B2aOptions};
    use vc_basic::Dialect;
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = Dialect::load(&data, "cbm_v2").unwrap();
    let mut o = B2aOptions::new("oracle.bas");
    o.target = "atari8".into();
    o.turbo = turbo;
    let c = convert(body, &d, &o).unwrap_or_else(|e| panic!("{}\n{body}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let cpu = vc_target::load_cpu(&data, "sally").unwrap();
    let mut ao = vc_asm::Options::new("oracle.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data.join("sys"));
    let out = vc_asm::assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let dir = std::env::temp_dir().join("vc_atari_oracle");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("CONV.XEX");
    std::fs::write(&f, vc_asm::to_xex(&out).unwrap()).unwrap();
    let done = out.symbols["rt_end"] as u16;
    let mut a = Atari800::launch(f.to_str().unwrap(), None).expect("atari800 starten");
    let hit = a.run_until(done, Duration::from_secs(40)).expect("run_until");
    assert!(hit, "Programmende nicht erreicht:\n{body}");
    let sav = a.read_mem(0x58, 2).unwrap();
    let lines = screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap());
    let gfx = gfx_state(&mut a);
    a.stop();
    (lines.into_iter().filter(|l| !l.is_empty()).collect(), gfx)
}

/// Programme, deren Ausgabe das echte Atari-BASIC und das übersetzte Programm gleich liefern müssen
const SAME: &[(&str, &str)] = &[
    ("zahlen", "10 PRINT 1/3\n20 PRINT 2^10;\" \";10/4;\" \";-7/2;\" \";.5\n30 PRINT 1E10;\" \";1E-6;\" \";123456789012;\" \";.000123\n40 PRINT SQR(16);\" \";SQR(2);\" \";INT(-2.5);\" \";ABS(-3.5);\" \";SGN(-9)\n50 PRINT 0.1+0.2=0.3\n60 PRINT 3*1.1;\" \";7/3;\" \";100/7"),
    ("funktionen", "10 PRINT EXP(1);\" \";LOG(10);\" \";2^0.5;\" \";3^3;\" \";(-2)^3;\" \";2^-2\n20 PRINT INT(7.9);INT(-0.5);INT(1E10+.5)"),
    ("schleife", "10 FOR X=0 TO 1 STEP .25\n20 PRINT X;\",\";\n30 NEXT X\n40 PRINT\n50 FOR I=3 TO 1 STEP -1:PRINT I*1.5;:NEXT I"),
    ("felder", "10 DIM A(3)\n20 FOR I=0 TO 3:A(I)=I/4:NEXT I\n30 PRINT A(1);A(2);A(3);A(0)\n40 READ P,Q:PRINT P*Q;P+Q\n50 DATA 1.5, 4"),
    ("strings", "10 DIM A$(10),B$(10)\n20 A$=\"ABCDEF\":PRINT A$(1,1);\"|\";A$(2);\"|\";A$(2,3);\"|\";A$(6,6)\n30 A$(3,4)=\"XYZ\":PRINT A$;LEN(A$)\n40 A$(3)=\"XY\":PRINT A$;LEN(A$)\n50 A$(LEN(A$)+1)=\"!!\":PRINT A$;LEN(A$)\n60 B$=\"HELLO WORLD\":PRINT B$;LEN(B$)\n70 A$(7)=\"123\":PRINT A$"),
    ("strfun", "10 DIM A$(20),B$(10)\n20 A$=STR$(12.5):PRINT A$\n30 PRINT VAL(\"12.5E1\");VAL(\"-3\");ASC(\"A\");CHR$(66);LEN(STR$(100))\n40 PRINT VAL(\"12AB\");VAL(\" 12\")\n50 A$=\"ABC\":B$=\"ABD\"\n60 PRINT A$<B$;A$=B$;A$<>B$;A$>B$;A$<=\"ABC\"\n70 PRINT STR$(1/3);\"|\";STR$(-2);\"|\";STR$(1E10)"),
    ("strdata", "10 DIM A$(10),B$(10),C$(10)\n20 READ A$,B$,C$,X\n30 PRINT \"[\";A$;\"][\";B$;\"][\";C$;\"]\";X\n40 DATA \" AB \", CD ,EF:G, 7"),
    ("logik", "10 PRINT 5 AND 3;\" \";5 AND 0;\" \";0 OR 0;\" \";5 OR 0;\" \";2 OR 4;\" \";NOT 0;\" \";NOT 5\n20 PRINT (5>3)+(2>1);\" \";5>3;\" \";5<3;\" \";-(5>3)\n30 X=5:IF X THEN PRINT \"T\"\n40 PRINT 6 AND 3;\" \";-1 AND 5;\" \";1.5 AND 1;\" \";3=3.0;\" \";1<2<3"),
    ("ganz", "10 X=5:Y=3\n20 PRINT X>Y;X AND Y;X OR 0;NOT X;(X<Y)+1;-4;5\n30 PRINT X*Y;X-Y*10;X+Y"),
    ("komma", "10 PRINT 1,2,3\n20 PRINT \"A\",\"B\"\n30 PRINT 1;2;3\n40 PRINT 1.5,\"X\",-2"),
    ("fluss", "10 FOR I=1 TO 3:GOSUB 100:NEXT I\n20 ON 2 GOTO 50,60\n50 PRINT \"X\"\n60 PRINT \"Y\"\n70 GOTO 200\n100 PRINT I*I;\n110 RETURN\n200 PRINT \"ENDE\""),
    ("rang", "10 PRINT NOT 0=1;\" \";NOT 0+1;\" \";NOT 0 AND 0;\" \";NOT 1 OR 1;\" \";-2^-2\n15 X=2:PRINT -X^2;\" \";2^-X;\" \";-X*-X;\" \";1+NOT 0;\" \";2*NOT 0;\" \";3- -2\n20 PRINT -2^2;\" \";2^3^2;\" \";1-2-3;\" \";2*-3;\" \";-3*2;\" \";2+3*4^2\n30 PRINT NOT 1=2;\" \";1+2>2;\" \";1 OR 0 AND 0;\" \";(1 OR 0) AND 0\n40 PRINT 7-2*3+1;\" \";10/4/5;\" \";-(2+3)^2"),
    ("trig", "10 PRINT SIN(1);\" \";COS(1);\" \";ATN(1)\n20 PRINT SIN(-.5);\" \";COS(0);\" \";SIN(0);\" \";ATN(10);\" \";ATN(-.3);\" \";ATN(.5)\n30 PRINT SIN(100);\" \";COS(3.14159265);\" \";SIN(3.14159265/2);\" \";ATN(1E10);\" \";COS(-2)\n40 PRINT SIN(.1);\" \";SIN(2);\" \";SIN(3);\" \";SIN(4);\" \";SIN(5);\" \";SIN(6);\" \";COS(.7);\" \";COS(2.5)"),
    ("grad", "10 DEG\n20 PRINT SIN(30);\" \";COS(60);\" \";ATN(1);\" \";SIN(90);\" \";COS(180);\" \";SIN(45);\" \";ATN(-2)\n30 RAD\n40 PRINT SIN(1)"),
    ("stick", "10 PRINT STICK(0);STRIG(0);STICK(1);STRIG(1)\n20 IF STICK(0)=15 AND STRIG(0)=1 THEN PRINT \"RUHE\""),
    ("vorzeichen", "10 PRINT - -2;\" \";- - 2;\" \";-(-2);\" \";3- -2;\" \";3--2;\" \";- -2*3\n20 X=2:PRINT - -X;\" \";-(-X);\" \";- - -X;\" \";+ -X;\" \";- +X;\" \";--X;\" \";NOT (-1)"),
    ("runden", "10 DIM A(4):A(1.6)=7:A(2.4)=9:PRINT A(1);A(2);A(3)\n20 PRINT CHR$(65.5);CHR$(65.4);\n30 X=1.5:PRINT A(X);A(2.5)"),
    ("kanaele", "10 OPEN #1,8,0,\"E:\"\n20 PUT #1,72:PUT #1,73:PUT #1,155\n30 PRINT #1;\"ABC\";1+2\n40 CLOSE #1\n50 PRINT \"OK\""),
    ("lesen", "10 GRAPHICS 3+16:COLOR 1:PLOT 0,0:COLOR 2:PLOT 1,0:COLOR 3:PLOT 2,0\n20 POSITION 0,0:GET #6,A:GET #6,B:GET #6,C\n30 GRAPHICS 0:PRINT A;B;C"),
    ("cmp", "10 DIM A$(5),B$(5)\n20 A$=\"a\":B$=\"A\"\n30 IF A$>B$ THEN PRINT \"gross\"\n40 IF A$=\"a\" AND B$=\"A\" THEN PRINT \"beide\"\n50 IF A$=\"\" THEN PRINT \"leer\"\n60 B$=\"\":IF B$=\"\" THEN PRINT \"leer\""),
];

#[test]
fn converted_programs_match_real_atari_basic() {
    if !enabled() { return; }
    for (name, body) in SAME {
        let real = run_basic(body);
        let conv = run_converted(body);
        let tail = &conv[conv.len().saturating_sub(real.len())..];
        assert_eq!(tail, &real[..], "Programm '{name}' weicht ab\n{body}\nreal: {real:#?}\nconv: {conv:#?}");
        eprintln!("{name}: gleich ({} Zeilen)", real.len());
    }
}

/// Grafikprogramme: Bildschirmspeicher, Farbschatten und Cursorzellen müssen nach dem Lauf gleich sein
const GFX: &[(&str, &str)] = &[
    ("gr8", "10 GRAPHICS 8+16\n20 SETCOLOR 2,0,0:SETCOLOR 1,0,14\n30 COLOR 1\n40 PLOT 10,10:DRAWTO 100,50:DRAWTO 20,80\n50 POSITION 5,5:DRAWTO 150,5"),
    ("gr3", "10 GRAPHICS 3+16\n20 COLOR 2:PLOT 3,3:DRAWTO 30,3:DRAWTO 30,15\n30 COLOR 1:PLOT 0,0:DRAWTO 39,23"),
    ("gr1text", "10 GRAPHICS 1\n20 POSITION 2,3:PRINT #6;\"HALLO\"\n30 SETCOLOR 0,4,6\n40 PRINT #6;\"WELT\""),
    ("gr5", "10 GRAPHICS 5+16:SETCOLOR 0,2,6\n20 COLOR 1:PLOT 0,0:DRAWTO 79,39\n30 COLOR 2:PLOT 79,0:DRAWTO 0,39\n40 COLOR 3:POSITION 40,0:DRAWTO 40,39"),
    ("gr0pos", "10 GRAPHICS 0\n20 POSITION 5,3:PRINT \"X\"\n30 POSITION 10,7:PRINT \"ABC\";1.5"),
    ("gr2zahl", "10 GRAPHICS 2:X=7\n20 POSITION 1,1:PRINT #6;\"N=\";X;\" \";X*1.5"),
    ("locate", "10 GRAPHICS 3+16:COLOR 3:PLOT 5,5\n20 LOCATE 5,5,A:LOCATE 6,5,B\n30 GRAPHICS 0:PRINT A;B"),
];

#[test]
fn converted_graphics_match_real_atari_basic() {
    if !enabled() { return; }
    for (name, body) in GFX {
        let (rt, rg) = run_basic_full(body);
        let (ct, cg) = run_converted_full(body);
        assert!(!rg.is_empty(), "{name}: Orakel lief nicht: {rt:?}");
        // SAVMSC ($58/$59) liegt je nach RAM-Obergrenze woanders (mit BASIC-Modul tiefer): nicht vergleichen
        let skip = rg.len() - 9 + 4;
        if let Some(i) = rg.iter().zip(&cg).enumerate().position(|(i, (a, b))| a != b && i != skip && i != skip + 1) { panic!("Programm '{name}': Grafikzustand weicht ab bei Byte {i} (real {:02X}, übersetzt {:02X}); Text real {rt:?} / übersetzt {ct:?}", rg[i], cg[i]); }
        if rg[rg.len() - 9 + 3] & 0x0F == 0 {   // Textmodus: auch die Bildschirmzeilen vergleichen
            let tail = &ct[ct.len().saturating_sub(rt.len())..];
            assert_eq!(tail, &rt[..], "Programm '{name}': Text weicht ab");
        }
        eprintln!("{name}: gleich");
    }
}

/// Turbo BASIC XL (Quelle ohne Zeilennummern) gegen das gleichwertige klassische Atari-BASIC mit Sprüngen: das übersetzte Turbo-Programm muss dasselbe zeigen.
/// (Ein Turbo-BASIC-ROM liegt nicht vor; geprüft werden die Blockanweisungen als Sprungstrukturen und MOD/DIV für nichtnegative Zahlen.)
const TURBO: &[(&str, &str, &str)] = &[
    ("schleifen", "i=1
WHILE i<=4
  EXEC quad
  i=i+1
WEND
PRINT
j=10
REPEAT
  j=j-3
  PRINT j;\" \";
UNTIL j<3
PRINT
k=0
DO
  k=k+2
  IF k>7
    EXIT
  ELSE
    PRINT k;\" \";
  ENDIF
LOOP
PRINT
PRINT 17 MOD 5;\" \";17 DIV 5;\" \";100 MOD 7;\" \";100 DIV 7;\" \";8.5 DIV 2
END
PROC quad
  PRINT i*i;\" \";
ENDPROC
",
     "10 GOTO 50
20 PRINT I*I;\" \";
30 RETURN
50 I=1
60 IF I>4 THEN 100
70 GOSUB 20
80 I=I+1
90 GOTO 60
100 PRINT
110 J=10
120 J=J-3
130 PRINT J;\" \";
140 IF NOT (J<3) THEN 120
150 PRINT
160 K=0
170 K=K+2
180 IF K>7 THEN 210
190 PRINT K;\" \";
200 GOTO 170
210 PRINT
220 PRINT 17-5*INT(17/5);\" \";INT(17/5);\" \";100-7*INT(100/7);\" \";INT(100/7);\" \";INT(8.5/2)"),
    ("blockif", "FOR n=1 TO 6
  IF n MOD 2
    IF n>3
      PRINT \"G\";
    ELSE
      PRINT \"u\";
    ENDIF
  ELSE
    PRINT \"e\";
  ENDIF
NEXT n
PRINT
x=5
WHILE x>0
  x=x-2
  IF x<0 THEN PRINT \"neg\";
WEND
PRINT x
",
     "10 FOR N=1 TO 6
20 IF N-2*INT(N/2)=0 THEN 70
30 IF N>3 THEN 60
40 PRINT \"u\";
50 GOTO 80
60 PRINT \"G\";
65 GOTO 80
70 PRINT \"e\";
80 NEXT N
90 PRINT
100 X=5
110 IF X<=0 THEN 150
120 X=X-2
130 IF X<0 THEN PRINT \"neg\";
140 GOTO 110
150 PRINT X"),
];

// ---- Turbo BASIC XL im Original: Diskettenabbild (assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr, nicht in Git) --------------------

fn turbo_atr() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VC_TURBO_ATR") { return Some(PathBuf::from(p)); }
    let rel = "assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr";
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().map(|a| a.join(rel)).find(|p| p.exists())
}

/// Tastencode (Atari-Tastaturcode, wie in CH $02FC) für ein Zeichen; Großschreibung ist an (CAPS)
fn atari_key(c: char) -> u8 {
    match c.to_ascii_lowercase() {
        l @ 'a'..='z' => b"\x3f\x15\x12\x3a\x2a\x38\x3d\x39\x0d\x01\x05\x00\x25\x23\x08\x0a\x2f\x28\x3e\x2d\x0b\x10\x2e\x16\x2b\x17"[(l as u8 - b'a') as usize],
        d @ '0'..='9' => [0x32, 0x1f, 0x1e, 0x1a, 0x18, 0x1d, 0x1b, 0x33, 0x35, 0x30][(d as u8 - b'0') as usize],
        ' ' => 0x21, '\n' => 0x0c, ',' => 0x20, '.' => 0x22, '/' => 0x26, ';' => 0x02, '-' => 0x0e, '=' => 0x0f, '+' => 0x06, '*' => 0x07, '<' => 0x36, '>' => 0x37,
        '"' => 0x5e, '!' => 0x5f, '#' => 0x5a, '$' => 0x58, '%' => 0x5d, '&' => 0x5b, '(' => 0x70, ')' => 0x72, ':' => 0x42, '?' => 0x66, '^' => 0x47, '@' => 0x75,
        o => panic!("Taste {o:?} fehlt in atari_key"),
    }
}

/// Führt ein Turbo-BASIC-XL-Programm (mit Zeilennummern) im echten Turbo BASIC XL aus: das Abbild bootet in atari800, die Zeilen werden getippt
/// (Taste für Taste in CH, immer wenn der Tastaturtreiber nach einem Zeichen fragt), danach `RUN`. Der Bildschirm wird vorher gelöscht (Zeile 0);
/// zurück kommen die Bildschirmzeilen ohne das abschließende `READY`.
pub fn run_turbo_real(body: &str) -> Option<Vec<String>> { run_turbo_real_full(body, None).map(|r| r.0) }

/// `mode`: Grafikmodus für den Bildschirmspeicher (bei Textfenster zeigt DINDEX auf das Fenster)
pub fn run_turbo_real_full(body: &str, mode: Option<u8>) -> Option<(Vec<String>, Vec<u8>)> {
    let atr = turbo_atr()?;
    let mut a = Atari800::launch(atr.to_str().unwrap(), None).expect("atari800 starten");
    let t = a.read_mem(0xE424, 2).unwrap();
    let kget = (t[0] as u16 | (t[1] as u16) << 8).wrapping_add(1);   // Einsprung „Zeichen lesen“ des K:-Treibers
    let text = format!("0 PRINT CHR$(125)\n{}\nRUN\n", body.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n"));
    let mut in_str = false;
    for c in text.chars() {
        if c == '"' { in_str = !in_str; }
        // Umschaltsperre (SHFLOK $02BE): außerhalb von Zeichenketten immer Großbuchstaben, darin so, wie es im Quelltext steht
        let upper = !(in_str && c.is_ascii_lowercase());
        assert!(a.run_until(kget, Duration::from_secs(60)).expect("run_until"), "Turbo BASIC XL fragt nicht nach Tasten");
        a.write_mem(0x02BE, &[if upper { 0x40 } else { 0x00 }]).unwrap();
        a.write_mem(0x02FC, &[atari_key(c)]).unwrap();
    }
    // Ende des Programms: der Editor fragt wieder nach Tasten (nach READY); längere Läufe brauchen mehrere Anläufe
    let hit = a.run_until(kget, Duration::from_secs(60)).expect("run_until");
    let sav = a.read_mem(0x58, 2).unwrap();
    let lines = screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap());
    let gfx = match mode { None => gfx_state(&mut a), Some(m) => gfx_state_mode(&mut a, m) };
    a.stop();
    if !hit { return Some((vec!["<Ende nicht erreicht>".into()], gfx)); }
    let mut v: Vec<String> = lines.into_iter().filter(|l| !l.is_empty()).collect();
    if v.last().map_or(false, |l| l.trim() == "READY") { v.pop(); }
    Some((v, gfx))
}

/// Nummeriert Turbo-Quelltext ohne Zeilennummern (10, 20, …)
fn numbered(src: &str) -> String {
    use vc_basic::{number_lines, PlainOptions};
    number_lines(src, &PlainOptions::new("t.bas", 63999)).unwrap_or_else(|e| panic!("{e:?}")).text
}

#[test]
fn turbo_original_smoke() {
    if !enabled() { return; }
    let Some(out) = run_turbo_real("10 PRINT 1/3\n20 PRINT \"HI\"") else { eprintln!("kein Turbo-ATR"); return };
    eprintln!("{out:#?}");
    assert!(out.iter().any(|l| l.contains("HI")), "{out:?}");
}

#[test]
fn converted_turbo_programs_match_classic_atari_basic() {
    if !enabled() { return; }
    for (name, turbo, classic) in TURBO {
        let real = run_basic(classic);
        let conv = run_converted_opts(turbo, true).0;
        let tail = &conv[conv.len().saturating_sub(real.len())..];
        assert_eq!(tail, &real[..], "Turbo-Programm '{name}' weicht ab\n{turbo}\nreal: {real:#?}\nconv: {conv:#?}");
        eprintln!("{name}: gleich ({} Zeilen)", real.len());
    }
}

#[test]
fn converted_turbo_programs_match_real_turbo_basic() {
    if !enabled() { return; }
    let mut bad = vec![];
    for (name, turbo, _) in TURBO {
        let Some(real) = run_turbo_real(&numbered(turbo)) else { eprintln!("kein Turbo-ATR"); return };
        let conv = run_converted_opts(turbo, true).0;
        let tail = &conv[conv.len().saturating_sub(real.len())..];
        if tail != &real[..] { bad.push(format!("'{name}'\nTurbo: {real:#?}\nconv:  {conv:#?}")); } else { eprintln!("{name}: gleich ({} Zeilen)", real.len()); }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn converted_turbo_example_draws_its_title() {
    if !enabled() { return; }
    let src = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/atari/turbo_basic/balken.tur")).unwrap();
    use vc_basic::b2a::{convert, B2aOptions};
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = vc_basic::Dialect::load(&data, "cbm_v2").unwrap();
    let mut o = B2aOptions::new("balken.tur");
    o.target = "atari8".into();
    o.turbo = true;
    let c = convert(&src, &d, &o).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let cpu = vc_target::load_cpu(&data, "sally").unwrap();
    let mut ao = vc_asm::Options::new("balken.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data.join("sys"));
    let out = vc_asm::assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let f = std::env::temp_dir().join("vc_atari_oracle").join("BALKEN.XEX");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, vc_asm::to_xex(&out).unwrap()).unwrap();
    let mut a = Atari800::launch(f.to_str().unwrap(), None).expect("atari800 starten");
    // Das Programm wartet am Ende auf eine Taste (UNTIL PEEK(764)<>255): am Label dieser Zeile anhalten
    let until_line = src.lines().position(|l| l.trim_start().starts_with("UNTIL")).unwrap() + 1;
    let label = &c.map.iter().find(|m| m.src_line == until_line).expect("Zeilenkarte").label;
    let hit = a.run_until(out.symbols[label.as_str()] as u16, Duration::from_secs(30)).expect("run_until");
    assert!(hit, "UNTIL-Zeile nicht erreicht");
    let sav = a.read_mem(0x58, 2).unwrap();
    let lines = screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap());
    a.stop();
    let shown: Vec<String> = lines.into_iter().filter(|l| !l.is_empty()).collect();
    assert!(shown.iter().any(|l| l.contains("TURBO BASIC XL")), "{shown:?}");
    assert!(shown.iter().any(|l| l.contains("PROC, EXEC, WHILE, REPEAT")), "{shown:?}");
    assert!(shown.iter().any(|l| l.contains("TASTE DRUECKEN")), "{shown:?}");
    assert_eq!(shown.iter().map(|l| l.matches('#').count()).sum::<usize>(), 16, "{shown:?}");   // 16 Spalten mit je einem # in einer Zeile
}

/// Programme, die das echte Turbo BASIC XL und das übersetzte Programm gleich ausgeben müssen (Quelle ohne Zeilennummern)
const TURBO_REAL: &[(&str, &str)] = &[
    ("zahlen", "PRINT 1/3\nPRINT 2/3;\" \";100/7;\" \";1E10;\" \";.000123;\" \";SQR(2)\nPRINT 3*1.1;\" \";1234567890;\" \";0.1+0.2;\" \";EXP(1)\n"),
    ("moddiv", "PRINT -7 MOD 3;\" \";7 MOD -3;\" \";-7 MOD -3;\" \";-7 DIV 2;\" \";7 DIV -2;\" \";-7 DIV -2\nPRINT -8.5 DIV 2;\" \";8.5 MOD 2;\" \";5 MOD 0.5\nPRINT 7.5 MOD 2;\" \";-7.5 MOD 2;\" \";1E10 MOD 7;\" \";3 MOD 5;\" \";0 MOD 3;\" \";2.5 DIV 0.5\n"),
    ("hexdec", "PRINT HEX$(0);\" \";HEX$(255);\" \";HEX$(256);\" \";HEX$(65535);\" \";HEX$(10);HEX$(1.5);HEX$(2.5)\nPRINT DEC(\"FF\");\" \";DEC(\"ffff\");\" \";DEC(\"FFFF\");\" \";DEC(\"10000\");\" \";DEC(\"1G\")\nPRINT DEC(\"\");\" \";DEC(\" 1\");\" \";DEC(\"0A\");\" \";DEC(\"A0\");\" \";DEC(\"FFFF1\")\n"),
    ("instr", "DIM A$(20),B$(10)\nA$=\"HELLO WORLD\":B$=\"LO\"\nPRINT INSTR(A$,\"WOR\");\" \";INSTR(A$,\"X\");\" \";INSTR(A$,\"L\");\" \";INSTR(A$,\"L\",5)\nPRINT INSTR(A$,B$);\" \";INSTR(\"\",\"A\");\" \";INSTR(A$,\"\")\nPRINT INSTR(\"ABCD\",\"D\",3);INSTR(\"ABCD\",\"D\",4);INSTR(\"ABCD\",\"C\",2)\nPRINT INSTR(\"ABCD\",\"CD\",2);INSTR(\"ABCD\",\"CD\",3);INSTR(\"ABCD\",\"B\",1)\nPRINT INSTR(\"AAAA\",\"AA\",2);INSTR(\"AAAA\",\"AA\",3);INSTR(\"AAAA\",\"A\",3);INSTR(\"AAAA\",\"A\",4);INSTR(\"ABCABC\",\"BC\",3)\n"),
    ("bewegen", "POKE 1792,1:POKE 1793,2:POKE 1794,3\n-MOVE 1792,1793,2\nPRINT PEEK(1792);PEEK(1793);PEEK(1794)\nMOVE 1793,1792,2\nPRINT PEEK(1792);PEEK(1793);PEEK(1794)\nDPOKE 1796,4660\nPRINT DPEEK(1796);\" \";PEEK(1796)\n"),
    ("zufall", "n=0\nFOR i=1 TO 50\n  r=RAND(6)\n  IF r<0 OR r>5 THEN n=n+1\nNEXT i\nPRINT n;\" \";RAND(1);\" \";RAND(0)\n"),
    ("cls", "PRINT \"A\"\nCLS\nPRINT \"B\"\n"),
    ("zeit", "POKE 18,0:POKE 19,2:POKE 20,3\nPRINT TIME$\nPOKE 18,1:POKE 19,255:POKE 20,0\nPRINT TIME$\nPOKE 18,254:POKE 19,255:POKE 20,0\nPRINT TIME$;\" \";LEN(TIME$)\n"),
    ("onexec", "FOR i=0 TO 3\n  ON i EXEC a,b\nNEXT i\nPRINT\nEND\nPROC a\n  PRINT \"A\";\nENDPROC\nPROC b\n  PRINT \"B\";\nENDPROC"),
    ("bput", "POKE 1792,72:POKE 1793,69:POKE 1794,76:POKE 1795,76:POKE 1796,79\nOPEN #1,8,0,\"E:\"\nBPUT #1,1792,5\nPUT #1,33:PUT #1,155\nCLOSE #1\nPRINT \"ENDE\""),
    ("bget", "GRAPHICS 3+16\nCOLOR 1:PLOT 0,0:COLOR 2:PLOT 1,0:COLOR 3:PLOT 2,0:COLOR 1:PLOT 3,0\nPOSITION 0,0\nBGET #6,1792,4\nGRAPHICS 0\nPRINT PEEK(1792);PEEK(1793);PEEK(1794);PEEK(1795)"),
    ("taste", "PRINT LEN(INKEY$);\" \";INKEY$=\"\"\n"),
];

#[test]
fn converted_turbo_words_match_real_turbo_basic() {
    if !enabled() { return; }
    let mut bad = vec![];
    for (name, turbo) in TURBO_REAL {
        let Some(real) = run_turbo_real(&numbered(turbo)) else { eprintln!("kein Turbo-ATR"); return };
        let conv = run_converted_opts(turbo, true).0;
        let real: Vec<String> = if *name == "cls" { real } else { real };
        let tail = &conv[conv.len().saturating_sub(real.len())..];
        if tail != &real[..] { bad.push(format!("'{name}'\nTurbo: {real:#?}\nconv:  {conv:#?}")); } else { eprintln!("{name}: gleich ({} Zeilen)", real.len()); }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Grafikprogramme in Turbo BASIC XL: Bildschirmspeicher, Farbschatten und Cursor müssen im Original und im übersetzten Programm gleich sein
const TURBO_GFX: &[(&str, &str)] = &[
    ("fuellen", "GRAPHICS 3\nCOLOR 1\nPLOT 2,2:DRAWTO 12,2:DRAWTO 12,12:DRAWTO 2,12:DRAWTO 2,2\nFCOLOR 2\nPLOT 4,4\nFILLTO 10,4\nFILLTO 10,8"),
    ("fuellen2", "GRAPHICS 5\nCOLOR 1\nPLOT 10,5:DRAWTO 60,5:DRAWTO 60,30:DRAWTO 10,30:DRAWTO 10,5\nCOLOR 3\nFCOLOR 2\nPLOT 20,10\nFILLTO 50,10\nPOSITION 20,12\nFILLTO 40,25"),
];

#[test]
fn converted_turbo_graphics_match_real_turbo_basic() {
    if !enabled() { return; }
    for (name, body) in TURBO_GFX {
        let mode: u8 = body.trim_start_matches("GRAPHICS ").split('\n').next().unwrap().parse().unwrap();
        // Turbo BASIC XL schaltet am Programmende auf Text zurück: ein INPUT am Ende hält das Programm im Grafikmodus an (Textfenster)
        let Some((_, rg)) = run_turbo_real_full(&numbered(&format!("{body}\nINPUT a$")), Some(mode)) else { eprintln!("kein Turbo-ATR"); return };
        let (_, cg) = run_converted_opts(body, true);
        assert!(!rg.is_empty(), "{name}: Orakel lief nicht");
        // verglichen werden Bildschirmspeicher und Farbschatten (ohne Cursorzellen und SAVMSC, die letzten neun Bytes)
        let n = rg.len();
        if let Some(i) = rg[..n].iter().zip(&cg[..n]).position(|(a, b)| a != b) { panic!("Turbo-Programm '{name}': Grafikzustand weicht ab bei Byte {i} (real {:02X}, übersetzt {:02X})", rg[i], cg[i]); }
        eprintln!("{name}: gleich");
    }
}


// ---- Dateien auf D: (Atari-DOS aus dem Turbo-BASIC-Abbild: DOS.SYS aus Turbo_Basic_XL.atr, das übersetzte Programm als AUTORUN.SYS) ----------------

/// Bootbare DOS-Diskette (Boot-Sektoren und DOS.SYS/DUP.SYS aus dem Turbo-Abbild) mit dem Programm als AUTORUN.SYS; gibt den Pfad einer frischen Kopie zurück
fn dos_disk(xex: &[u8], name: &str) -> Option<PathBuf> {
    let src = vc_disk::Atr::from_bytes(&std::fs::read(turbo_atr()?).ok()?).ok()?;
    let mut d = vc_disk::Atr::new_dos2();
    for n in 1..=3 { *d.sector_mut(n) = *src.sector(n); }
    for f in ["DOS.SYS", "DUP.SYS"] { d.dos2_add(f, &src.dos2_read(f).ok()?).ok()?; }
    d.dos2_add("AUTORUN.SYS", xex).ok()?;
    let dir = std::env::temp_dir().join("vc_atari_oracle"); std::fs::create_dir_all(&dir).ok()?;
    let f = dir.join(name);
    std::fs::write(&f, d.to_bytes()).ok()?;
    Some(f)
}

/// Wie `run_converted_opts`, aber von einer DOS-Diskette gestartet (D: ist vorhanden)
fn run_converted_on_disk(body: &str, turbo: bool) -> Option<Vec<String>> {
    use vc_basic::b2a::{convert, B2aOptions};
    use vc_basic::Dialect;
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let d = Dialect::load(&data, "cbm_v2").unwrap();
    let mut o = B2aOptions::new("oracle.bas");
    o.target = "atari8".into();
    o.turbo = turbo;
    let c = convert(body, &d, &o).unwrap_or_else(|e| panic!("{}\n{body}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let cpu = vc_target::load_cpu(&data, "sally").unwrap();
    let mut ao = vc_asm::Options::new("oracle.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data.join("sys"));
    let out = vc_asm::assemble(&c.asm, &cpu, &ao).unwrap_or_else(|e| panic!("{}", e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n")));
    let disk = dos_disk(&vc_asm::to_xex(&out).unwrap(), "CONVDOS.ATR")?;
    let done = out.symbols["rt_end"] as u16;
    let mut a = Atari800::launch(disk.to_str().unwrap(), None).expect("atari800 starten");
    let hit = a.run_until(done, Duration::from_secs(60)).expect("run_until");
    let sav = a.read_mem(0x58, 2).unwrap();
    let lines = screen_lines(&a.read_mem(sav[0] as u16 | (sav[1] as u16) << 8, 960).unwrap());
    a.stop();
    assert!(hit, "Programmende nicht erreicht:\n{body}\n{lines:?}");
    Some(lines.into_iter().filter(|l| !l.is_empty()).collect())
}

/// Dasselbe im echten Turbo BASIC XL, auf einer frischen Kopie des Abbilds (Schreibzugriffe ändern das Abbild)
fn run_turbo_real_on_copy(body: &str) -> Option<Vec<String>> {
    let atr = turbo_atr()?;
    let dir = std::env::temp_dir().join("vc_atari_oracle"); std::fs::create_dir_all(&dir).ok()?;
    let copy = dir.join("TURBOCOPY.ATR");
    std::fs::copy(&atr, &copy).ok()?;
    std::env::set_var("VC_TURBO_ATR", &copy);
    let r = run_turbo_real(body);
    std::env::remove_var("VC_TURBO_ATR");
    r
}

const DISK: &[(&str, &str)] = &[
    ("datei", "OPEN #1,8,0,\"D:T.DAT\"\nPUT #1,65:PUT #1,66:PUT #1,67\nPRINT #1;\"HALLO\"\nCLOSE #1\nOPEN #1,4,0,\"D:T.DAT\"\nGET #1,A:GET #1,B:GET #1,C\nCLOSE #1\nPRINT A;B;C"),
    ("block", "FOR i=0 TO 9:POKE 1792+i,100+i:NEXT i\nOPEN #2,8,0,\"D:B.BIN\"\nBPUT #2,1792,10\nCLOSE #2\nFOR i=0 TO 9:POKE 1792+i,0:NEXT i\nOPEN #2,4,0,\"D:B.BIN\"\nBGET #2,1792,10\nCLOSE #2\nPRINT PEEK(1792);\" \";PEEK(1795);\" \";PEEK(1801)"),
];

#[test]
fn converted_disk_io_matches_real_turbo_basic() {
    if !enabled() { return; }
    for (name, body) in DISK {
        let Some(real) = run_turbo_real_on_copy(&numbered(body)) else { eprintln!("kein Turbo-ATR"); return };
        let Some(conv) = run_converted_on_disk(body, true) else { eprintln!("keine DOS-Diskette"); return };
        let tail = &conv[conv.len().saturating_sub(real.len())..];
        assert_eq!(tail, &real[..], "'{name}': Turbo {real:#?} / übersetzt {conv:#?}");
        eprintln!("{name}: gleich ({} Zeilen): {real:?}", real.len());
    }
}

#[test]
fn converted_disk_errors_carry_the_atari_error_number() {
    if !enabled() { return; }
    // Datei fehlt: das Original meldet Fehler 170, das übersetzte Programm "?DEVICE ERROR 170 IN 20"
    let body = "PRINT \"VOR\"\nOPEN #1,4,0,\"D:FEHLT.DAT\"\nPRINT \"NACH\"";
    let Some(real) = run_turbo_real_on_copy(&numbered(body)) else { return };
    let Some(conv) = run_converted_on_disk(body, true) else { return };
    eprintln!("Turbo: {real:?}\nübersetzt: {conv:?}");
    assert!(real.iter().any(|l| l.contains("170")), "{real:?}");
    assert!(conv.iter().any(|l| l.to_lowercase().contains("device error 170")), "{conv:?}");
    assert!(!conv.iter().any(|l| l.contains("NACH")), "nach dem Fehler darf nichts mehr laufen: {conv:?}");
}
