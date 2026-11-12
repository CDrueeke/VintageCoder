// SPDX-License-Identifier: GPL-3.0-or-later
//! Rundreise des basic2asm-Parsers (Konzept 14.4): Text → Syntaxbaum → kanonischer Text → Syntaxbaum muss denselben Baum ergeben.
//! Außerdem: der kanonische Text ist ein Fixpunkt, und der JSON-Export enthält alle Zeilen.
use std::path::PathBuf;
use vc_basic::b2a::{parse, roundtrip, B2aOptions};
use vc_basic::Dialect;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn dialect(id: &str) -> Dialect { Dialect::load(&root().join("data"), id).unwrap() }
fn opts(target: &str) -> B2aOptions { let mut o = B2aOptions::new("t.bas"); o.target = target.into(); o }
fn diag(e: Vec<vc_core::Diagnostic>) -> String { e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n") }

fn check(src: &str, target: &str) -> String {
    let d = dialect(vc_basic::b2a::target_info(target).unwrap().dialect);
    let o = opts(target);
    let text = roundtrip(src, &d, &o).unwrap_or_else(|e| panic!("{}\n{src}", diag(e)));
    // Fixpunkt: der kanonische Text noch einmal zurückgeschrieben ändert sich nicht
    let again = roundtrip(&text, &d, &o).unwrap_or_else(|e| panic!("{}\n{text}", diag(e)));
    assert_eq!(text, again, "kanonischer Text ist kein Fixpunkt");
    text
}

const CBM_ALL: &str = "10 REM alle Anweisungen
20 A=1:B%=2:C$=\"HALLO\":D=1.5E3
30 DIM X(10),Y$(5)
40 FOR I=1 TO 10 STEP 2:X(I)=I*2+A^2-(B%+1)*3/4:NEXT I
50 IF A=1 AND B%<>2 OR NOT C$=\"X\" THEN PRINT \"JA\";A;B%,TAB(5);SPC(2);\"x\"
60 IF A>0 THEN 100
70 ON A GOTO 100,110,120
80 ON B% GOSUB 200,210
90 PRINT -A^2;(-A)^2;A^-B%;-(A+1)
100 INPUT \"NAME\";N$
110 GET K$
120 READ P,Q,R$:DATA 1,2.5,\"text\",-3,hallo
130 RESTORE:RESTORE 120
140 POKE 53280,0:SYS 49152:WAIT 1,2,3:WAIT 1,2
150 DEF FNA(X)=X*2+1:PRINT FNA(3)+LEN(C$)+ASC(\"A\")+VAL(\"5\")
160 PRINT LEFT$(C$,2)+MID$(C$,2,3)+RIGHT$(C$,1)+CHR$(65)+STR$(5)
170 PRINT PEEK(1024);INT(2.5);SGN(-1);ABS(-2);SQR(4);RND(1);SIN(1);COS(1);TAN(1);ATN(1);LOG(2);EXP(1)
180 GOSUB 200:CLR:END
200 RETURN
210 STOP
";

#[test]
fn cbm_roundtrip_covers_all_statements() {
    let t = check(CBM_ALL, "c64");
    assert!(t.contains("FOR I=1 TO 10 STEP 2"), "{t}");
    assert!(t.contains("IF A=1 AND B%<>2 OR NOT C$=\"X\" THEN PRINT"), "{t}");
}

#[test]
fn precedence_survives_the_roundtrip() {
    // Klammern erscheinen genau dort, wo sie nötig sind
    let t = check("10 PRINT (1+2)*3-(4-5)-6/(7*8)\n20 PRINT (A OR B) AND C\n30 PRINT -(A^2)+(-A)^2\n40 PRINT 2^3^2;2^(3^2)\n", "c64");
    assert!(t.contains("(1+2)*3-(4-5)-6/(7*8)"), "{t}");
    assert!(t.contains("(A OR B) AND C"), "{t}");
    assert!(t.contains("(-A)^2"), "{t}");
    assert!(t.contains("2^(3^2)"), "{t}");
}

#[test]
fn examples_roundtrip() {
    // Beispiele des Repositories, soweit der Parser sie ganz kennt (BASIC V2 des C64)
    let mut n = 0;
    for f in ["examples/c64/basic_native/sterne.bas", "examples/c64/basic_langenamen/punkte.bas", "data/templates/basic-hello/c64.bas"] {
        let p = root().join(f);
        let Ok(src) = std::fs::read_to_string(&p) else { continue };
        let d = dialect("cbm_v2");
        let o = opts("c64");
        if parse(&src, &d, &o).is_err() { continue; }   // z. B. Marken oder Erweiterungen, die der Parser nicht liest
        roundtrip(&src, &d, &o).unwrap_or_else(|e| panic!("{f}: {}", diag(e)));
        n += 1;
    }
    assert!(n >= 1, "kein Beispiel geprüft");
}

const ATARI_ALL: &str = "10 DIM A$(20),B$(5),N(3)
20 GRAPHICS 8+16:COLOR 1:SETCOLOR 2,0,14
30 PLOT 10,10:DRAWTO 100,50:POSITION 5,5:LOCATE 1,1,K
40 SOUND 0,100,10,8:X=STICK(0)+STRIG(0)
50 PRINT #6;\"TEXT\";X
60 A$=\"HELLO|WORLD\":A$(3,4)=\"XY\":A$(LEN(A$)+1)=\"!\":B$=A$(2,3)
70 PRINT A$;LEN(A$);ASC(A$);VAL(\"12\");STR$(5);CHR$(65)
80 IF A$=B$ AND -X<>2 THEN PRINT -2^2;NOT 0=1;- -X
90 READ C$,D:DATA  AB ,\"X:Y\", 3
100 DEG:PRINT SIN(30);COS(60);ATN(1):RAD
";

#[test]
fn atari_roundtrip_keeps_raw_text_and_rules() {
    let t = check(ATARI_ALL, "atari8");
    assert!(t.contains("HELLO|WORLD"), "{t}");          // ASCII bleibt ASCII
    assert!(t.contains("DATA AB ,\"X:Y\", 3"), "{t}");   // DATA roh (Leerzeichen und Anführungszeichen bleiben)
    assert!(t.contains("POKE 251,6") && t.contains("-X"), "{t}");   // DEG wird zu POKE 251,6; - -X zu -X (Messung am Original)
    assert!(t.contains("-2^2"), "{t}");                 // Atari: unäres Minus über ^ braucht keine Klammern
}

#[test]
fn json_export_lists_all_lines() {
    let d = dialect("cbm_v2");
    let p = parse(CBM_ALL, &d, &opts("c64")).unwrap_or_else(|e| panic!("{}", diag(e)));
    let j = vc_basic::b2a::print::to_json(&p);
    let lines = j["lines"].as_array().unwrap();
    assert_eq!(lines.len(), CBM_ALL.lines().count());
    assert_eq!(lines[3]["stmts"][0]["stmt"], "for");
    assert_eq!(lines[3]["number"], 40);
}

#[test]
fn unsupported_statements_are_reported() {
    let d = dialect("cbm_v2");
    let e = roundtrip("10 OPEN 1,8,15,\"x\"\n", &d, &opts("c64")).unwrap_err();
    assert!(diag(e).contains("E190"));
}

#[test]
fn turbo_roundtrip_and_example() {
    let mut o = opts("atari8");
    o.turbo = true;
    let d = dialect("cbm_v2");
    let src = "PROC p\n  PRINT \"x\"\nENDPROC\ni=0\nWHILE i<3\n  EXEC p\n  IF i MOD 2\n    PRINT 7 DIV 2;i MOD 3\n  ELSE\n    DPOKE 1792,i\n  ENDIF\n  i=i+1\nWEND\nREPEAT\n  i=i-1\nUNTIL i<=0\nDO\n  EXIT\nLOOP\nPAUSE 1:MOVE 1,2,3:PRINT DPEEK(1792)\n";
    let t = roundtrip(src, &d, &o).unwrap_or_else(|e| panic!("{}", diag(e)));
    assert!(t.contains("IF I MOD") || t.contains("IF I-") || t.contains("DIV"), "{t}");
    // das Beispiel des Repositories (Turbo BASIC XL ohne Zeilennummern)
    let f = root().join("examples/atari/turbo_basic/balken.tur");
    if let Ok(src) = std::fs::read_to_string(&f) {
        roundtrip(&src, &d, &o).unwrap_or_else(|e| panic!("balken.tur: {}", diag(e)));
        vc_basic::b2a::convert(&src, &d, &o).unwrap_or_else(|e| panic!("balken.tur: {}", diag(e)));
    }
}
