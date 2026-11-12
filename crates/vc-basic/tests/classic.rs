// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_basic::{build_prg, parse_prg, to_classic_text, tokenize_text, Dialect};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn dialect() -> Dialect { Dialect::load(&data(), "cbm_v2").unwrap() }
fn petcat() -> Option<PathBuf> {
    ["/Applications/vice-arm64-gtk3-3.10/bin/petcat", "/usr/local/bin/petcat", "/opt/homebrew/bin/petcat", "/usr/bin/petcat"].iter().map(PathBuf::from).find(|p| p.exists())
}

const PROGRAMS: &[&str] = &[
    "10 print\"{clr}Hello, World! abc ABC\"\n20 for i=1 to 10:poke 53280,i:next\n30 if a<=3 then goto 10\n",
    "10 rem Test 'quote' text\n20 data 1,2,hello,\"x y\",3\n30 print#1,tab(5);spc(3);a$;x\n40 on x goto 10,20\n",
    "10 a=3.5e3:b%=-2:c=a^2<>b\n20 input\"name\";n$:print left$(n$,3);mid$(n$,2,1);chr$(65)\n30 get a$:if a$=\"\" then 30\n",
    "10 total=5:score=total+1:for t=1 to 3:next t\n20 print \"{rvon}{red}hi{rvof}{wht}\":goto 20\n",
    "100 dim a(10),b$(3):def fn f(x)=x*2+sin(x)\n110 poke 646,1:sys 49152:open 1,8,15,\"i\":close 1\n120 print fre(0)-peek(56320) and 255 or not 3\n",
    "10 print \"a\"; : rem comment\n20 gosub 100 : return\n30 data \"a:b\",c : data d\n100 end\n",
];

#[test]
fn tokenizer_matches_petcat_byte_for_byte() {
    let Some(p) = petcat() else { eprintln!("petcat nicht gefunden – übersprungen"); return };
    let d = dialect();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("petcat");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, src) in PROGRAMS.iter().enumerate() {
        let (txt, prg) = (dir.join(format!("p{i}.bas")), dir.join(format!("p{i}.prg")));
        std::fs::write(&txt, src).unwrap();
        let st = Command::new(&p).args(["-w2", "-o"]).arg(&prg).arg("--").arg(&txt).output().unwrap();
        assert!(st.status.success(), "petcat: {}", String::from_utf8_lossy(&st.stderr));
        let ours = build_prg(&tokenize_text(src, &d, "t.bas").unwrap_or_else(|e| panic!("{e:?}")), 0x0801);
        assert_eq!(ours, std::fs::read(&prg).unwrap(), "Programm {i} weicht von petcat ab:\n{src}");
    }
}

#[test]
fn detokenize_then_tokenize_is_identity() {
    let d = dialect();
    for src in PROGRAMS {
        let prg = build_prg(&tokenize_text(src, &d, "t.bas").unwrap(), 0x0801);
        let parsed = parse_prg(&prg).unwrap();
        assert_eq!(parsed.load_addr, 0x0801);
        let text = to_classic_text(&parsed, &d);
        let again = build_prg(&tokenize_text(&text, &d, "t2.bas").unwrap_or_else(|e| panic!("{e:?}\n{text}")), 0x0801);
        assert_eq!(again, prg, "Round-Trip fehlgeschlagen:\n{text}");
    }
}

#[test]
fn listing_of_known_tokens() {
    let d = dialect();
    // 10 PRINT "HI" : 20 GOTO 10  (Tokens $99 und $89)
    let lines = tokenize_text("10 print\"hi\"\n20 goto10\n", &d, "t").unwrap();
    assert_eq!(lines[0].1, vec![0x99, 0x22, 0x48, 0x49, 0x22]);
    assert_eq!(lines[1].1, vec![0x89, 0x31, 0x30]);
    let prg = build_prg(&lines, 0x0801);
    assert_eq!(prg, vec![0x01, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x99, 0x22, 0x48, 0x49, 0x22, 0x00, 0x13, 0x08, 0x14, 0x00, 0x89, 0x31, 0x30, 0x00, 0x00, 0x00]);
}

#[test]
fn errors() {
    let d = dialect();
    assert!(tokenize_text("print 1\n", &d, "t").is_err());
    assert!(tokenize_text("70000 print 1\n", &d, "t").is_err());
    assert!(tokenize_text("10 print \"€\"\n", &d, "t").is_err());
    assert!(parse_prg(&[1, 8, 9, 9, 0, 0]).is_err());
}
