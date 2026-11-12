// SPDX-License-Identifier: GPL-3.0-or-later
//! Turbo BASIC XL: Abgleich mit dem Referenz-Tokenizer `basicParser` (github.com/dmsc/tbxl-parser, nur als Prüfwerkzeug – nicht Teil dieses Projekts).
//! Pfad über die Umgebungsvariable `VC_TBXL_PARSER`; ohne sie werden die Abgleich-Tests übersprungen.
use std::path::PathBuf;
use std::process::Command;
use vc_basic::atari::{tokenize_text, AtariDialect, AtariProgram};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn turbo() -> AtariDialect { AtariDialect::load(&data(), "turbo_basic_xl").unwrap() }
fn atari() -> AtariDialect { AtariDialect::load(&data(), "atari_basic").unwrap() }
fn oracle() -> Option<PathBuf> { std::env::var("VC_TBXL_PARSER").ok().map(PathBuf::from).filter(|p| p.exists()) }

/// Programme mit ausdrücklichen Zeilennummern (eine Anweisungszeile je Zeile, damit die Zeilen 1:1 übereinstimmen).
const PROGRAMS: &[(&str, &str)] = &[
    ("labels", "10 EXEC hello\n20 PROC hello\n30 PRINT \"HI\"\n40 ENDPROC\n50 GO# there\n60 # there\n70 ON A EXEC hello,hello\n80 ON A GO# there,there\n90 TRAP #there:RESTORE #there:TRAP 100:RESTORE\n"),
    ("operators", "10 X=$FF:Y=%2+%0+%3+%1\n20 Z=A DIV 3 MOD 2 EXOR 5 & 3 ! 4\n30 T$=TIME$:A=RND:B=RAND(3):C=DPEEK(512):D=TIME:E=ERR:F=ERL\n40 A=USR(1536,1,2):B=FRAC(2.5)+TRUNC(2.5)\n50 A=INSTR(A$,\"x\"):B=INSTR(A$,\"x\",2):C=DEC(\"FF\"):D$=HEX$(255):E$=INKEY$:G=UINSTR(A$,\"x\")\n"),
    ("control", "100 IF A\n110 PRINT 1\n120 ELSE\n130 PRINT 2\n140 ENDIF\n150 WHILE A<3\n160 A=A+1\n170 WEND\n180 REPEAT:UNTIL A>3\n190 DO:EXIT:LOOP\n200 IF A=1 THEN PRINT \"X\"\n210 IF A THEN 100\n"),
    ("statements", "10 DPOKE 512,1234:MOVE 1,2,3:-MOVE 1,2,3\n20 BPUT #1,A,B:BGET #1,A,B\n30 PAUSE 5:FILLTO 5,6:CIRCLE 1,2,3:CIRCLE 1,2,3,4\n40 *F-:*B+:*L:*F\n50 CLS:CLS #6:TEXT 1,2,\"A\"\n60 DIR:DIR \"D:*.*\":DUMP:DSOUND:DSOUND 1,2,3,4\n70 RENUM 1,2,3:DEL 1,2:TRACE:BLOAD \"X\":BRUN \"Y\"\n80 PAINT 1,2:FCOLOR 1:%PUT 1,2\n90 LOCK \"X\":UNLOCK \"X\":RENAME \"A,B\":DELETE \"X\"\n100 TIME$=\"12:34:56\"\n110 -- kommentar\n120 REM text\n130 DATA 1,2,3\n"),
    ("turbo_forms", "10 CLOSE:SOUND:INPUT \"name\";N$:PUT 65\n20 INPUT \"a\",A,B\n"),
    ("classic", "10 DIM A$(10),B(5):A$=\"HI\":B(1)=2:A$(1,2)=\"X\"\n20 FOR I=1 TO 10 STEP 2:NEXT I\n30 PRINT \"A\";I,\"B\";:PRINT #6;\"X\"\n40 OPEN #1,4,0,\"D:X\":CLOSE #1\n50 GET #1,A:PUT #16,65:SOUND 0,1,2,3\n60 INPUT N$:INPUT #1,A,B\n70 ON A GOTO 10,20:ON A GOSUB 10,20\n80 POKE 82,0:GRAPHICS 0:PLOT 1,2:DRAWTO 3,4:POSITION 5,6:SETCOLOR 1,2,3\n90 A=A+1:IF A<10 THEN 30\n100 A$=\"x\":IF A$=\"x\" THEN PRINT \"EQ\":IF A$<>\"y\" THEN B=1\n110 X=-A:Y=+A:Z=NOT A:W=A^2+SQR(A)*ABS(-1)\n"),
    ("names", "10 a=1:Hello=2:b$=\"x\":Lab=3\n20 exec Lab\n30 proc LAB\n40 endproc\n50 dim cc(3)\n60 print cc(1);A;HELLO;B$\n"),
    ("abbrev", "10 G. 10\n20 PR. \"A\";:? \"B\"\n30 FOR I=1 TO 3:N. I\n40 GOS. 100\n50 POK. 1,2:RET.\n60 . kommentar\n70 ' auch\n"),
];

fn oracle_bytes(o: &PathBuf, atari_mode: bool, src: &str, name: &str) -> Vec<u8> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tbxl");
    std::fs::create_dir_all(&dir).unwrap();
    let (lst, out) = (dir.join(format!("{name}.lst")), dir.join(format!("{name}.{}.bas", if atari_mode { "a" } else { "t" })));
    std::fs::write(&lst, src).unwrap();
    let mut c = Command::new(o);
    c.args(["-f", "-k", "-q"]);
    if atari_mode { c.arg("-A"); }
    let st = c.arg("-o").arg(&out).arg(&lst).output().unwrap();
    assert!(st.status.success(), "{name}: Referenzparser: {}{}", String::from_utf8_lossy(&st.stdout), String::from_utf8_lossy(&st.stderr));
    std::fs::read(&out).unwrap()
}

fn show(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ") }

#[test]
fn turbo_output_matches_reference_tokenizer_byte_for_byte() {
    let Some(o) = oracle() else { eprintln!("VC_TBXL_PARSER nicht gesetzt – übersprungen"); return };
    let d = turbo();
    for (name, src) in PROGRAMS {
        let ours = tokenize_text(src, &d, &format!("{name}.lst")).unwrap_or_else(|e| panic!("{name}: {e:?}")).save_file(&d);
        let theirs = oracle_bytes(&o, false, src, name);
        if ours != theirs {
            let n = ours.iter().zip(&theirs).position(|(a, b)| a != b).unwrap_or(ours.len().min(theirs.len()));
            panic!("{name}: weicht vom Referenz-Tokenizer ab (Byte {n}; unsere Länge {}, Referenz {}):\nunser    …{}\nReferenz …{}", ours.len(), theirs.len(), show(&ours[n.saturating_sub(6)..(n + 24).min(ours.len())]), show(&theirs[n.saturating_sub(6)..(n + 24).min(theirs.len())]));
        }
    }
}

#[test]
fn atari_mode_still_matches_reference_for_plain_atari_basic() {
    let Some(o) = oracle() else { return };
    let d = atari();
    for (name, src) in PROGRAMS.iter().filter(|(n, _)| matches!(*n, "classic")) {
        let ours = tokenize_text(src, &d, "x.lst").unwrap_or_else(|e| panic!("{name}: {e:?}")).save_file(&d);
        let theirs = oracle_bytes(&o, true, src, name);
        // Direktzeile am Ende: END (ROM-Abbild) gegen CSAVE-Token (Referenz) – nur der Rest zählt
        assert_eq!(&ours[..ours.len() - 2], &theirs[..theirs.len() - 2], "{name}");
    }
}

#[test]
fn variables_beyond_128_use_the_two_byte_form() {
    let d = turbo();
    let src: String = (0..140).map(|i| format!("{} V{i}={i}\n", 10 + i)).collect();
    let p = tokenize_text(&src, &d, "v.lst").unwrap();
    assert_eq!(p.vars.len(), 140);
    // Variable 130: Token 00 02
    let line = &p.lines[130].1;
    assert_eq!(&line[4..8], &[0x36, 0x00, 0x02, 0x2D], "{}", show(line));
    assert!(tokenize_text(&src, &atari(), "v.lst").is_err(), "Atari BASIC kennt nur 128 Variablen");
}

#[test]
fn turbo_listing_roundtrips_to_the_same_program() {
    let d = turbo();
    for (name, src) in PROGRAMS.iter().filter(|(n, _)| *n != "abbrev") {
        let p = tokenize_text(src, &d, "x.lst").unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let text = p.listing(&d);
        let again = tokenize_text(&text, &d, "l.lst").unwrap_or_else(|e| panic!("{name}: Listing nicht lesbar: {e:?}\n{text}"));
        assert_eq!(again.save_file(&d), p.save_file(&d), "{name}: Listing → Tokenizer ändert das Programm:\n{text}");
        let saved = p.save_file(&d);
        assert_eq!(AtariProgram::parse_save(&saved, &d).unwrap(), p, "{name}: SAVE-Datei wird nicht zurückgelesen");
    }
}

#[test]
fn parameters_for_proc_are_rejected_with_a_clear_message() {
    let e = tokenize_text("10 EXEC a,1\n20 PROC a\n30 ENDPROC\n", &turbo(), "x.lst").unwrap_err();
    assert!(e[0].message.contains("Parameter"), "{}", e[0].message);
}

#[test]
fn example_project_matches_reference_tokenizer() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/atari/turbo_basic/balken.tur");
    let src = std::fs::read_to_string(&path).unwrap();
    let numbered = vc_basic::number_lines(&src, &vc_basic::PlainOptions::new("balken.tur", 32767)).unwrap();
    let d = turbo();
    let ours = tokenize_text(&numbered.text, &d, "balken.tur").unwrap_or_else(|e| panic!("{e:?}")).save_file(&d);
    assert!(ours.len() < 2000);
    let Some(o) = oracle() else { return };
    assert_eq!(ours, oracle_bytes(&o, false, &numbered.text, "balken"), "Beispielprojekt weicht vom Referenz-Tokenizer ab");
}
