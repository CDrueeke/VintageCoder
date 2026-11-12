// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_basic::atari::{AtariDialect, AtariProgram};
use vc_basic::{compile_atari, AtariOptions};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn d() -> AtariDialect { AtariDialect::load(&data(), "atari_basic").unwrap() }
fn listing(src: &str) -> String { compile_atari(src, &d(), &AtariOptions::new("t.sbas")).unwrap_or_else(|e| panic!("{e:?}")).listing }
fn codes(src: &str) -> Vec<&'static str> { compile_atari(src, &d(), &AtariOptions::new("t.sbas")).err().unwrap().into_iter().map(|x| x.code).collect() }

#[test]
fn structured_program_lowers_to_numbered_listing() {
    let src = "const LIMIT = 10\ntotal = 0\ncounter = 1\nwhile counter <= LIMIT\n  total = total + counter\n  counter = counter + 1\nwend\nprint \"sum=\";total\nif total / 2 = int(total / 2) then\n  print \"even\"\nelse\n  print \"odd\"\nendif\n@end:\nx = usr($0600)\n";
    assert_eq!(listing(src), "10 TOTAL=0\n20 COUNTER=1\n30 IF NOT (COUNTER<=10) THEN 70\n40 TOTAL=TOTAL+COUNTER\n50 COUNTER=COUNTER+1\n60 GOTO 30\n70 PRINT \"sum=\";TOTAL\n80 IF NOT (TOTAL/2=INT(TOTAL/2)) THEN 110\n90 PRINT \"even\"\n100 GOTO 120\n110 PRINT \"odd\"\n120 X=USR(1536)\n");
}

#[test]
fn labels_goto_on_and_repeat() {
    let src = "@start:\nn = n + 1\nrepeat\n  print n\nuntil n > 5\non n goto @start, @done\n@done:\nend\n";
    assert_eq!(listing(src), "10 N=N+1\n20 PRINT N\n30 IF NOT (N>5) THEN 20\n40 ON N GOTO 10,50\n50 END\n");
}

#[test]
fn long_names_consts_and_roundtrip() {
    let src = "const TITLE = \"hello\"\nplayerscore = 100\nname$ = TITLE\nrem $d020 bleibt Kommentar\nposition 2, 3: print name$; playerscore\n";
    let c = compile_atari(src, &d(), &AtariOptions::new("t.sbas")).unwrap();
    assert_eq!(c.listing, "10 PLAYERSCORE=100\n20 NAME$=\"hello\"\n30 REM $d020 bleibt Kommentar\n40 POSITION 2,3:PRINT NAME$;PLAYERSCORE\n");
    let back = AtariProgram::parse_save(&c.file, &d()).unwrap();
    assert_eq!(back, c.program);
}

#[test]
fn errors_are_reported_with_source_lines() {
    assert_eq!(codes("goto @nowhere\n"), vec!["B107"]);
    assert_eq!(codes("if a then\nprint 1\n"), vec!["B106"]);
    assert_eq!(codes("print (1\n"), vec!["B100"]);
    assert_eq!(codes("const X = a + b\n"), vec!["B108"]);
    let e = compile_atari("print 1\nprint (2\n", &d(), &AtariOptions::new("t.sbas")).err().unwrap();
    assert_eq!((e[0].file.as_str(), e[0].line), ("t.sbas", 2));
}

#[test]
fn elseif_do_loop_exit_and_procedures() {
    assert_eq!(listing("if a = 1 then\n print 1\nelseif a = 2 then\n print 2\nelse\n print 3\nendif\nprint 9\n"),
               "10 IF NOT (A=1) THEN 40\n20 PRINT 1\n30 GOTO 80\n40 IF NOT (A=2) THEN 70\n50 PRINT 2\n60 GOTO 80\n70 PRINT 3\n80 PRINT 9\n");
    assert_eq!(listing("do while n < 3\n n = n + 1\nloop\nprint n\n"), "10 IF NOT (N<3) THEN 40\n20 N=N+1\n30 GOTO 10\n40 PRINT N\n");
    assert_eq!(listing("do\n n = n + 1\n if n = 4 then\n  exit\n endif\nloop until n = 9\nprint n\n"),
               "10 N=N+1\n20 IF NOT (N=4) THEN 40\n30 GOTO 50\n40 IF NOT (N=9) THEN 10\n50 PRINT N\n".replace("40 IF NOT (N=9) THEN 10\n50 PRINT N\n", "40 IF NOT (N=9) THEN 10\n50 PRINT N\n"));
    assert_eq!(listing("call plot(3, a + 1)\nend\nproc plot(px, py)\n poke 1024 + px, py\nendproc\n"),
               "10 PX=3:PY=A+1:GOSUB 40\n20 END\n30 GOTO 60\n40 POKE 1024+PX,PY\n50 RETURN\n60 REM \n");
    assert_eq!(codes("exit\n"), vec!["B106"]);
    assert_eq!(codes("call nix\n"), vec!["B109"]);
}

#[test]
fn conditional_compilation_on_atari() {
    assert_eq!(listing("#if SYSTEM_ATARI\nprint 1\n#else\nprint 2\n#endif\n"), "10 PRINT 1\n");
    let mut o = AtariOptions::new("t.sbas");
    o.system = Some("c64".into());
    assert_eq!(compile_atari("#if SYSTEM_ATARI\nprint 1\n#else\nprint 2\n#endif\n", &d(), &o).unwrap().listing, "10 PRINT 2\n");
}
