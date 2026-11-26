// SPDX-License-Identifier: GPL-3.0-or-later
//! `#define` und `#if`/`#else`/`#endif` im nativen BASIC: Ersetzung vor der Nummerierung, Zeilen bleiben erhalten, Bedeutungstreue.
use std::path::{Path, PathBuf};
use vc_basic::preprocess::{run, PreOptions};
use vc_basic::{compile_plain_atari, compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn pre(src: &str) -> String { run(src, &PreOptions::new("t.bas")).unwrap().text }
fn err(src: &str, sys: Option<&str>) -> Vec<&'static str> {
    let mut o = PreOptions::new("t.bas"); o.system = sys.map(String::from); o.keywords = ["PRINT", "POKE"].iter().map(|s| s.to_string()).collect();
    run(src, &o).err().unwrap().iter().map(|d| d.code).collect()
}
fn prg(src: &str, sys: &str) -> Vec<u8> {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let mut o = PlainOptions::new("t.bas", 63999); o.system = Some(sys.into());
    compile_plain_cbm(src, &d, &o, d.load_addr).unwrap_or_else(|e| panic!("{:?}", e.iter().map(|x| x.message.clone()).collect::<Vec<_>>())).prg
}

#[test]
fn define_replaces_whole_words_only() {
    let t = pre("#define SCR 1024\nPOKE SCR,1\nPOKE SCR+40,2\nPRINT \"SCR\" : REM SCR\nA=SCREEN+SCR1+SCR\nDATA SCR,SCR:POKE SCR,3\nGOTO @SCR\nX$=\"A\":PRINT SCR$\n");
    assert_eq!(t, "\nPOKE 1024,1\nPOKE 1024+40,2\nPRINT \"SCR\" : REM SCR\nA=SCREEN+SCR1+1024\nDATA SCR,SCR:POKE 1024,3\nGOTO @SCR\nX$=\"A\":PRINT SCR$\n");
    assert_eq!(t.lines().count(), 8, "gleiche Zeilenzahl");
}

#[test]
fn defines_nest_and_apply_only_after_the_definition() {
    assert_eq!(pre("PRINT W\n#define W 40\n#define H W*25\nPRINT H\n"), "PRINT W\n\n\nPRINT 40*25\n");
    let e = err("#define A B\n#define B A\nPRINT A\n", None);
    assert_eq!(e, ["B176"]);
    // Namen in Zeichenketten, `$FF` und Zahlen bleiben
    assert_eq!(pre("#define FF 5\nPOKE $FF,1E3\nPRINT \"FF\"\n"), "\nPOKE $FF,1E3\nPRINT \"FF\"\n");
}

#[test]
fn define_errors() {
    assert_eq!(err("#define PRINT 5\n", None), ["B172"]);
    assert_eq!(err("#define print 5\n", None), ["B172"], "Groß-/Kleinschreibung egal bei Schlüsselwörtern");
    assert_eq!(err("#define 1X 5\n", None), ["B174"]);
    assert_eq!(err("#define\n", None), ["B174"]);
    assert_eq!(err("#define A 1\n#define A 2\n", None), ["B174"]);
}

#[test]
fn if_else_endif_by_system_and_name() {
    let src = "#if SYSTEM_C64\nPOKE 53280,0\n#else\nPOKE 712,0\n#endif\nEND\n";
    let mut o = PreOptions::new("t.bas"); o.system = Some("c64".into());
    assert_eq!(run(src, &o).unwrap().text, "\nPOKE 53280,0\n\n\n\nEND\n");
    o.system = Some("atari8".into());
    let r = run(src, &o).unwrap();
    assert_eq!(r.text, "\n\n\nPOKE 712,0\n\nEND\n");
    assert_eq!(r.excluded, [1, 2, 3, 5]);
    // eigene Namen, Verneinung, Verschachtelung; `#define` im ausgeschlossenen Zweig gilt nicht
    assert_eq!(pre("#define DEBUG 1\n#if DEBUG\nA\n#endif\n#if !DEBUG\nB\n#endif\n#if NIE\n#define X 1\n#endif\nX\n"), "\n\nA\n\n\n\n\n\n\n\nX\n");
    assert_eq!(pre("#define D 0\n#if D\nA\n#else\nB\n#endif\n"), "\n\n\n\nB\n\n");
    assert_eq!(pre("#if SYSTEM_C64\n#if DEBUG\nA\n#else\nB\n#endif\n#endif\n"), "\n\n\n\n\n\n\n");
}

#[test]
fn if_structure_errors() {
    assert_eq!(err("#if SYSTEM_C64\nA\n", Some("c64")), ["B175"]);
    assert_eq!(err("#else\n", None), ["B175"]);
    assert_eq!(err("#endif\n", None), ["B175"]);
    assert_eq!(err("#if X\n#else\n#else\n#endif\n", None), ["B175"]);
    assert_eq!(err("#if\nA\n#endif\n", None), ["B175"]);
    assert_eq!(err("#if X-Y\nA\n#endif\n", None), ["B175"]);
}

#[test]
fn error_lines_stay_those_of_the_source() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let e = compile_plain_cbm("#define A 1\n#if SYSTEM_C64\nPRINT 1\n#endif\nGOTO @nirgends\n", &d, &PlainOptions::new("t.bas", 63999), d.load_addr).err().unwrap();
    assert_eq!((e[0].code, e[0].line), ("B110", 5));
}

#[test]
fn build_is_identical_to_the_hand_written_program() {
    let with = "#define SCR 1024\n#define COLOR_RAM 55296\n#if SYSTEM_C64\n#define BORDER 53280\n#else\n#define BORDER 53281\n#endif\n@loop: POKE SCR,1\nPOKE COLOR_RAM,2\nPOKE BORDER,3\nGOTO @loop\n";
    let by_hand = "@loop: POKE 1024,1\nPOKE 55296,2\nPOKE 53280,3\nGOTO @loop\n";
    assert_eq!(prg(with, "c64"), prg(by_hand, "c64"));
}


#[test]
fn atari_uses_the_same_pass() {
    let d = vc_basic::atari::AtariDialect::load(&data(), "atari_basic").unwrap();
    let mut o = PlainOptions::new("t.bas", 32767); o.system = Some("atari8".into());
    let a = compile_plain_atari("#define COLPF2 710\n#if SYSTEM_ATARI8\nPOKE COLPF2,0\n#endif\n", &d, &o).unwrap().file;
    let b = compile_plain_atari("POKE 710,0\n", &d, &PlainOptions::new("t.bas", 32767)).unwrap().file;
    assert_eq!(a, b);
}

#[test]
fn command_line_style_defines_apply() {
    let mut o = PreOptions::new("t.bas"); o.defines = vec![("LEVEL".into(), "3".into())];
    assert_eq!(run("PRINT LEVEL\n#if LEVEL\nX\n#endif\n", &o).unwrap().text, "PRINT 3\n\nX\n\n");
}
