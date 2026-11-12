// SPDX-License-Identifier: GPL-3.0-or-later
//! Statische Prüfungen (`vcc test`, Testkonzept Stufe 1): Größe, Zeropage, Überlappung, Zyklenspanne aus der `.vcdbg`.
use std::path::{Path, PathBuf};
use vc_ide::build::{build, Ctx};
use vc_ide::tests::{find_tests, run_file, Level, Options, Outcome, State, TestFile};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

const DEMO_ASM: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.asm");
const DEMO_TEST: &str = include_str!("../../../examples/c64/tests_demo/tests_demo.vctest");

/// Baut `asm` als Projekt in einem frischen Ordner und liefert den Ordner
fn project(tag: &str, target: &str, asm: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-stat-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("p.asm"), asm).unwrap();
    let out = build(&Ctx { root: &dir, data: &data() }, "p.asm", Some(target)).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    dir
}

fn run(dir: &Path, json: &str, strict: bool) -> Vec<Outcome> {
    let p = dir.join("t.vctest");
    std::fs::write(&p, json).unwrap();
    let tf = TestFile::load(&p).unwrap();
    run_file(&tf, &data(), &Options { strict, ..Default::default() }).outcomes
}

fn by<'a>(o: &'a [Outcome], needle: &str) -> &'a Outcome { o.iter().find(|x| x.name.contains(needle)).unwrap_or_else(|| panic!("kein Ergebnis mit „{needle}“ in {:?}", o.iter().map(|x| &x.name).collect::<Vec<_>>())) }

#[test]
fn demo_project_results() {
    let dir = project("demo", "c64", DEMO_ASM);
    let o = run(&dir, &DEMO_TEST.replace("build/tests_demo.prg", "build/p.prg"), false);
    let irq = by(&o, "Zyklen irq ≤");
    assert_eq!((irq.state, irq.measured.as_deref()), (State::Passed, Some("20")), "{irq:?}");                // pha 3 + lda zp 3 + sta abs 4 + pla 4 + rti 6
    let nc = by(&o, "Zyklen next_color ≤ 20");
    assert_eq!((nc.state, nc.measured.as_deref()), (State::Passed, Some("18 bis 19")), "{nc:?}");        // Verzweigung nach vorn: kürzester und längster Weg
    let hint = by(&o, "next_color ≤ 18");
    assert_eq!((hint.state, hint.level), (State::Failed, Level::Hint));
    assert!(!hint.is_error(), "Zyklenbudget ist standardmäßig nur ein Hinweis");
    assert!(by(&o, "clear_text").detail.contains("Schleife"));
    assert_eq!(by(&o, "clear_text").state, State::Skipped);
    assert!(by(&o, "Zyklen start").detail.contains("jsr"));
    assert_eq!(by(&o, "Zeropage").state, State::Passed);
    assert_eq!(by(&o, "Überlappung").state, State::Passed);
    assert_eq!(by(&o, "next_color: 15").state, State::Skipped, "Emulator-Tests laufen noch nicht");
    assert_eq!(o.iter().filter(|x| x.is_error()).count(), 0);
    // --strict: derselbe Hinweis ist ein Fehler
    let o = run(&dir, &DEMO_TEST.replace("build/tests_demo.prg", "build/p.prg"), true);
    assert!(by(&o, "next_color ≤ 18").is_error());
    assert_eq!(o.iter().filter(|x| x.is_error()).count(), 1);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn size_and_expression_limits() {
    let dir = project("size", "c64", DEMO_ASM);
    let o = run(&dir, r#"{ "target": "c64", "program": "build/p.prg", "static": [
        { "check": "size", "max": 61 }, { "check": "size", "max": "$3E" }, { "check": "size", "min": 100 },
        { "check": "size", "segment": "Gibt es nicht", "max": 10 }, { "check": "size", "maxEnd": "$0820" },
        { "check": "expr", "expr": "size(\"\") == 62" }, { "check": "expr", "expr": "size(\"\") == 63" }, { "check": "expr", "expr": "a == 1" }
    ] }"#, false);
    assert!(o[0].is_error() && o[0].detail.contains("1 Bytes zu viel"), "{:?}", o[0]);
    assert_eq!(o[1].state, State::Passed);
    assert!(o[2].is_error() && o[2].detail.contains("zu wenig"), "{:?}", o[2]);
    assert!(o[3].is_error() && o[3].detail.contains("nicht gefunden"));
    assert!(o[4].is_error() && o[4].detail.contains("zu weit"), "{:?}", o[4]);
    assert_eq!(o[5].state, State::Passed);
    assert!(o[6].is_error() && o[6].detail.contains("ergibt 0"));
    assert!(o[7].is_error() && o[7].detail.contains("weder Register noch Symbol"), "{:?}", o[7]);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn zeropage_segments_and_accesses() {
    let dir = project("zp", "c64", DEMO_ASM);
    let o = run(&dir, r#"{ "target": "c64", "program": "build/p.prg", "static": [
        { "check": "zeropage", "forbid": "$fb" }, { "check": "zeropage", "forbid": ["$02-$1f", "$d000-$dfff"] }, { "check": "zeropage", "forbid": "$0800-$0810" }, { "check": "zeropage", "forbid": "$zz" }
    ] }"#, false);
    assert!(o[0].is_error() && o[0].detail.contains("lda spricht $FB an"), "{:?}", o[0]);
    assert!(o[0].location.as_ref().is_some_and(|(f, l)| f.ends_with("p.asm") && *l == 52), "Fundstelle: lda COLOR in irq_handler, {:?}", o[0].location);
    assert_eq!(o[1].state, State::Passed);      // $D020 steht nur im Operanden von sta, Abs-Zugriffe oberhalb der Zeropage zählen nur im Bereich selbst
    assert!(o[2].is_error() && o[2].detail.contains("Segment"), "Segment auf dem verbotenen Bereich: {:?}", o[2]);
    assert!(o[3].is_error() && o[3].detail.contains("nicht verständlich"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn zeropage_range_hits_absolute_data_access() {
    // sta $d020 liegt im verbotenen Bereich $d000-$dfff: nur Zeropage-Zugriffe werden erkannt, Absolutzugriffe nicht (Hinweis im Konzept)
    let dir = project("zpabs", "c64", ".target c64\n.org $0801\n.basicstub s\ns: lda $0010\n lda $10,x\n sta ($20),y\n rts\n");
    let o = run(&dir, r#"{ "target": "c64", "program": "build/p.prg", "static": [ { "check": "zeropage", "forbid": "$10" }, { "check": "zeropage", "forbid": "$20" }, { "check": "zeropage", "forbid": "$11-$1f" } ] }"#, false);
    assert!(o[0].is_error() && o[0].detail.contains("lda spricht $10 an") && o[0].detail.contains("$10 an"), "{:?}", o[0]);
    assert!(o[1].is_error() && o[1].detail.contains("sta"), "{:?}", o[1]);
    assert_eq!(o[2].state, State::Passed);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn overlapping_segments_are_found() {
    // Der Assembler lehnt überlappende Segmente selbst ab (E301); die Prüfung fängt veraltete oder von Hand erzeugte Debugdateien ab
    let dir = project("ovl", "c64", ".target c64\n.org $0801\ns: rts\n");
    let seg = |s: u32, l: u32, bank: u32, n: &str| format!(r#"{{ "start": {s}, "length": {l}, "bank": {bank}, "name": "{n}" }}"#);
    let vcdbg = |segs: &[String]| format!(r#"{{ "format": 1, "symbols": [], "segments": [ {} ], "cycleBlocks": [], "lines": [] }}"#, segs.join(", "));
    let json = r#"{ "target": "c64", "program": "build/p.prg", "static": [ { "check": "overlap" } ] }"#;
    std::fs::write(dir.join("build/p.vcdbg"), vcdbg(&[seg(0x801, 8, 0, "A"), seg(0x805, 3, 0, "B"), seg(0x900, 1, 0, "C")])).unwrap();
    let o = run(&dir, json, false);
    assert!(o[0].is_error() && o[0].detail.contains("A $0801–$0808 überlappt B $0805–$0807") && !o[0].detail.contains('C'), "{:?}", o[0]);
    // andere Bank oder Segment ohne Bytes: keine Überlappung
    std::fs::write(dir.join("build/p.vcdbg"), vcdbg(&[seg(0x801, 8, 0, "A"), seg(0x805, 3, 1, "B"), seg(0x802, 0, 0, "")])).unwrap();
    assert_eq!(run(&dir, json, false)[0].state, State::Passed);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn cycle_blocks_and_branches() {
    let asm = "\
.target c64
.org $0801
.basicstub s
s: rts
.cycles up max=10
fwd:    lda #1          ; 2
        beq .skip       ; 2 / 3
        sta $10         ; 3
.skip:  rts             ; 6
.endcycles
.cycles lp
lp1:    dex
        bne lp1
.endcycles
far:    lda $10,x       ; Zeropage indiziert: 4
        bcc .t          ; 2 / 3
.t:     rts
tail:   nop
        jmp far
ind:    jmp ($0314)
";
    let dir = project("blk", "c64", asm);
    let o = run(&dir, r#"{ "target": "c64", "program": "build/p.prg", "static": [
        { "check": "cycles", "block": "up" }, { "check": "cycles", "block": "up", "max": 10 }, { "check": "cycles", "block": "lp" }, { "check": "cycles", "block": "nope" },
        { "check": "cycles", "label": "fwd" }, { "check": "cycles", "label": "far" }, { "check": "cycles", "label": "tail" }, { "check": "cycles", "label": "ind" },
        { "check": "cycles", "label": "unbekannt" }
    ] }"#, false);
    // Block „up“: Budget aus .cycles (max=10); lda# 2 + (beq genommen 3 | beq 2 + sta 3) + rts 6 → 11 bis 13, also über dem Budget
    assert_eq!(by(&o, "Zyklen up ≤ 10").measured.as_deref(), Some("11 bis 13"));
    assert!(o[0].state == State::Failed && o[0].level == Level::Hint, "{:?}", o[0]);
    assert!(o[1].detail.contains("13 Zyklen") && o[1].detail.contains("günstigster Fall 11"), "{:?}", o[1]);
    assert_eq!(o[2].state, State::Skipped); assert!(o[2].detail.contains("Schleife"), "{:?}", o[2]);
    assert!(o[3].is_error() && o[3].detail.contains("nicht gefunden") && o[3].detail.contains("up"));
    assert_eq!(o[4].measured.as_deref(), Some("11 bis 13"));
    assert_eq!(o[5].measured.as_deref(), Some("12 bis 13"), "{:?}", o[5]);      // lda zp,x 4 + bcc (2 nicht genommen | 3 genommen) + rts 6
    assert_eq!(o[6].measured.as_deref(), Some("17 bis 18"), "{:?}", o[6]);      // nop 2 + jmp 3 + far
    assert_eq!(o[7].state, State::Skipped); assert!(o[7].detail.contains("indirekt"), "{:?}", o[7]);
    assert!(o[8].is_error() && o[8].detail.contains("Marke"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn atari_xex_cycles() {
    let dir = project("atari", "atari8", ".target atari8\n.org $2000\nrt: lda #1\n clc\n adc $80\n sta $81\n rts\n.run rt\n");
    let o = run(&dir, r#"{ "target": "atari8", "program": "build/p.xex", "static": [ { "check": "cycles", "label": "rt", "max": 20, "level": "error" }, { "check": "zeropage", "forbid": "$80-$81" }, { "check": "size", "max": "$20" } ] }"#, false);
    assert_eq!(o[0].measured.as_deref(), Some("16"), "{:?}", o[0]);      // 2 + 2 + 3 + 3 + 6
    assert!(o[1].is_error(), "{:?}", o[1]);
    assert_eq!(o[2].state, State::Passed);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn setup_problems_are_errors_not_passes() {
    let dir = project("setup", "c64", ".target c64\n.org $0801\ns: rts\n");
    let t = |json: &str| run(&dir, json, false);
    let o = t(r#"{ "target": "c64", "program": "build/fehlt.prg", "static": [ { "check": "overlap" } ] }"#);
    assert!(o.len() == 1 && o[0].is_error() && o[0].detail.contains("zuerst bauen"), "{o:?}");
    let o = t(r#"{ "target": "c99", "program": "build/p.prg", "static": [ { "check": "overlap" } ] }"#);
    assert!(o.len() == 1 && o[0].is_error(), "{o:?}");
    // Programm ohne lesbare Opcodes (rohes Binärformat): Zugriffe und Zyklen sind „übersprungen“, nie „bestanden“
    std::fs::copy(dir.join("build/p.prg"), dir.join("build/p.bin")).unwrap();
    std::fs::copy(dir.join("build/p.vcdbg"), dir.join("build/p.vcdbg.bak")).unwrap();
    std::fs::rename(dir.join("build/p.vcdbg.bak"), dir.join("build/p.vcdbg")).unwrap();
    let o = t(r#"{ "target": "c64", "program": "build/p.bin", "static": [ { "check": "zeropage", "forbid": "$fb" }, { "check": "cycles", "label": "s" } ] }"#);
    assert_eq!((o[0].state, o[1].state), (State::Skipped, State::Skipped), "{o:?}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_file_format_errors() {
    let p = Path::new("x.vctest");
    let e = |s: &str| TestFile::parse(p, s).unwrap_err();
    assert!(e("{").contains("JSON"));
    assert!(e(r#"{"program":"a.prg"}"#).contains("target"));
    assert!(e(r#"{"target":"c64"}"#).contains("program"));
    assert!(e(r#"{"kind":"x","target":"c64","program":"a"}"#).contains("kind"));
    assert!(e(r#"{"target":"c64","program":"a","static":[{"check":"kaputt"}]}"#).contains("static[0]") && e(r#"{"target":"c64","program":"a","static":[{"check":"kaputt"}]}"#).contains("kaputt"));
    assert!(e(r#"{"target":"c64","program":"a","static":[{"check":"cycles","label":"a","block":"b"}]}"#).contains("genau eines"));
    assert!(e(r#"{"target":"c64","program":"a","static":[{"check":"cycles","label":"a","level":"laut"}]}"#).contains("level"));
    assert!(e(r#"{"target":"c64","program":"a","static":[{"check":"zeropage"}]}"#).contains("forbid"));
    assert!(TestFile::parse(p, r#"{"target":"c64","program":"a","tests":[{"name":"x"}]}"#).unwrap().tests.len() == 1);
}

#[test]
fn find_tests_skips_build_and_hidden_folders() {
    let dir = std::env::temp_dir().join(format!("vc-stat-find-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for d in ["tests", "build", ".git", "sub/deep"] { std::fs::create_dir_all(dir.join(d)).unwrap(); }
    for f in ["tests/b.vctest", "a.vctest", "build/x.vctest", ".git/y.vctest", "sub/deep/c.vctest", "readme.json"] { std::fs::write(dir.join(f), "{}").unwrap(); }
    let names: Vec<String> = find_tests(&dir).iter().map(|p| p.strip_prefix(&dir).unwrap().to_string_lossy().replace('\\', "/")).collect();
    assert_eq!(names, vec!["a.vctest", "sub/deep/c.vctest", "tests/b.vctest"]);
    assert_eq!(find_tests(&dir.join("a.vctest")).len(), 1);
    std::fs::remove_dir_all(&dir).ok();
}
