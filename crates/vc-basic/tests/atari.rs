// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_basic::atari::{bcd_encode, bcd_text, tokenize_text, AtariDialect, AtariProgram};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn dialect() -> AtariDialect { AtariDialect::load(&root().join("data"), "atari_basic").unwrap() }
fn fixture(name: &str) -> Vec<u8> { std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/atari").join(name)).unwrap() }
fn ours(listing: &str) -> AtariProgram { tokenize_text(listing, &dialect(), "t.lst").unwrap_or_else(|e| panic!("{e:?}")) }
fn listing_text(bytes: &[u8]) -> String { bytes.split(|b| *b == 0x9B).filter(|l| !l.is_empty()).map(|l| format!("{}\n", String::from_utf8_lossy(l))).collect() }

#[test]
fn bcd_numbers_match_the_rom() {
    let hex = |b: [u8; 6]| b.iter().map(|x| format!("{x:02X}")).collect::<String>();
    for (lit, want) in [("42", "404200000000"), ("1536", "411536000000"), ("53279", "420532790000"), ("710", "410710000000"), ("0", "000000000000"),
                        ("0.5", "3F5000000000"), ("3.5", "400350000000"), ("100", "410100000000"), ("65535", "420655350000"), ("1E3", "411000000000")] {
        assert_eq!(hex(bcd_encode(lit).unwrap()), want, "{lit}");
    }
    assert_eq!(bcd_text(&bcd_encode("1536").unwrap()), "1536");
    assert_eq!(bcd_text(&bcd_encode("0.25").unwrap()), ".25");
    assert!(bcd_encode("1E99").is_err());
}

/// Das vom echten BASIC-ROM erzeugte Abbild (aus atari800 gelesen) ist die Referenz.
#[test]
fn image_matches_rom_output_for_diverse_statements() {
    let lst = listing_text(&fixture("multi.lst"));
    let p = ours(&lst);
    let (img, _vntd, _stmtab, direct) = p.image();
    let rom = fixture("multi.bin");
    // Direktzeile: ROM hat `RUN`, wir `END` – nur den Rest vergleichen
    assert_eq!(&img[..direct], &rom[..rom.len() - 6], "Abbild weicht vom ROM ab");
    assert_eq!(rom.len() - 6, direct);
}

#[test]
fn save_file_header_matches_the_rom_example() {
    // `SAVE`-Datei, die das ROM für 10 X=42 / 20 Y=X*2 / 30 PRINT "HI";Y / 35 SAVE "H1:T.BAS" / 40 A=USR(1536) geschrieben hat (Kopf + Abbild bis STARP)
    let d = dialect();
    let p = ours("10 X=42\n20 Y=X*2\n30 PRINT \"HI\";Y\n35 SAVE \"H1:T.BAS\"\n40 A=USR(1536)\n");
    let f = p.save_file(&d);
    assert_eq!(&f[..14], &[0, 0, 0x00, 0x01, 0x03, 0x01, 0x04, 0x01, 0x1C, 0x01, 0x6A, 0x01, 0x70, 0x01]);
    // Namen- und Statement-Teil (ohne Werte der VVT, die im ROM-Beispiel zur Laufzeit gefüllt waren)
    assert_eq!(&f[14..18], &[0xD8, 0xD9, 0xC1, 0x00]);
    let stmts = &f[14 + 4 + 24..f.len() - 6];
    assert_eq!(stmts, &[0x0A, 0x00, 0x0F, 0x0F, 0x36, 0x80, 0x2D, 0x0E, 0x40, 0x42, 0, 0, 0, 0, 0x16, 0x14, 0x00, 0x11, 0x11, 0x36, 0x81, 0x2D, 0x80, 0x24, 0x0E, 0x40, 0x02, 0, 0, 0, 0, 0x16,
                        0x1E, 0x00, 0x0C, 0x0C, 0x20, 0x0F, 0x02, 0x48, 0x49, 0x15, 0x81, 0x16, 0x23, 0x00, 0x10, 0x10, 0x19, 0x0F, 0x08, 0x48, 0x31, 0x3A, 0x54, 0x2E, 0x42, 0x41, 0x53, 0x16,
                        0x28, 0x00, 0x12, 0x12, 0x36, 0x82, 0x2D, 0x3F, 0x3A, 0x0E, 0x41, 0x15, 0x36, 0, 0, 0, 0x2C, 0x16]);
    let back = AtariProgram::parse_save(&f, &d).unwrap();
    assert_eq!(back, p);
}

#[test]
fn listing_and_roundtrip() {
    let d = dialect();
    let lst = listing_text(&fixture("multi.lst"));
    let p = ours(&lst);
    let text = p.listing(&d);
    let again = ours(&text);
    assert_eq!(again.image(), p.image(), "Listing → Tokenizer muss dasselbe Abbild ergeben:\n{text}");
    assert!(text.contains("10 DIM A$(10),B(5):A$=\"HI\":B(1)=2:A$(1,2)=\"X\"") , "{text}");
    assert!(text.contains("30 IF A=1 THEN PRINT 2:GOTO 40") && text.contains("50 ON A GOTO 10,20"), "{text}");
    let rom_vars: Vec<&str> = p.vars.iter().map(String::as_str).collect();
    assert_eq!(rom_vars, vec!["A", "A$", "B(", "I", "X", "Y$", "B$", "Z", "Q"]);
}

#[test]
fn errors() {
    let d = dialect();
    for bad in ["10 PRINT (1\n", "10 FOR I=1\n", "10 X=\n", "10 IF A THEN\n", "99999 PRINT 1\n", "10 PRINT \"é\"\n", "10 X=PRINT\n", "PRINT 1\n"] {
        assert!(tokenize_text(bad, &d, "t").is_err(), "{bad}");
    }
}

/// Optional: ROM-Orakel live (VC_LIVE_ORACLE=1 und VC_ORACLE_SCRIPT=<Skript>; benötigt atari800 und python3; dauert etwa eine Minute).
#[test]
fn live_oracle_agrees_on_extra_programs() {
    if std::env::var("VC_LIVE_ORACLE").is_err() || Command::new("atari800").arg("-help").output().is_err() { eprintln!("Live-Orakel übersprungen (VC_LIVE_ORACLE=1 und atari800 nötig)"); return; }
    // Das Orakel-Skript liegt nicht im Repository: Pfad in VC_ORACLE_SCRIPT
    let oracle = std::env::var("VC_ORACLE_SCRIPT").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt"));
    if !oracle.is_file() { eprintln!("Live-Orakel übersprungen ({} fehlt)", oracle.display()); return; }
    let progs = [
        "5 A=USR(1536)\n10 DIM B$(20),C(3,4)\n20 C(1,2)=B+1:PRINT C(1,2)\n30 FOR J=10 TO 1 STEP -1:NEXT J\n40 IF A>1 OR B<=2 THEN GOTO 100\n50 PRINT \"A\";:PRINT\n60 X=INT(RND(0)*10)+SQR(4):Y=-X\n70 PLOT 1,2:DRAWTO 3,4:SETCOLOR 0,1,2\n80 LET Z=1E5+.25\n",
        "5 POKE 0,USR(1536)\n10 PRINT 1\n20 GOSUB 10\n30 GO TO 20\n40 RETURN\n",
    ];
    let d = dialect();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("atari_oracle");
    std::fs::create_dir_all(&dir).unwrap();
    for (i, p) in progs.iter().enumerate() {
        let (lst, bin) = (dir.join(format!("p{i}.lst")), dir.join(format!("p{i}.bin")));
        std::fs::write(&lst, p.lines().flat_map(|l| l.bytes().chain([0x9B])).collect::<Vec<u8>>()).unwrap();
        let out = Command::new("python3").arg(&oracle).arg(&lst).arg(&bin).output().unwrap();
        assert!(out.status.success(), "Orakel: {}", String::from_utf8_lossy(&out.stderr));
        let rom = std::fs::read(&bin).unwrap();
        let (img, _, _, direct) = ours(p).image();
        assert_eq!(&img[..direct], &rom[..rom.len() - 6], "Programm {i} weicht vom ROM ab:\n{p}");
    }
    let _ = d;
}
