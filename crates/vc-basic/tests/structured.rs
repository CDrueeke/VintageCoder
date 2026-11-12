// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_basic::{compile, parse_prg, to_classic_text, to_structured_text, tokenize_text, build_prg, Dialect, Options};

/// Ordner mit den (nicht im Repository liegenden) `.sbas`-Beispielen: Umgebungsvariable `VC_SBAS_EXAMPLES`; ohne sie überspringen sich die Tests
fn sbas_dir() -> PathBuf { std::env::var("VC_SBAS_EXAMPLES").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt")) }

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn dialect() -> Dialect { Dialect::load(&data(), "cbm_v2").unwrap() }
fn text(src: &str) -> String {
    let d = dialect();
    let c = compile(src, &d, &Options::new("t.sbas")).unwrap_or_else(|e| panic!("{e:?}"));
    to_classic_text(&parse_prg(&c.prg).unwrap(), &d)
}
fn codes(src: &str) -> Vec<&'static str> {
    compile(src, &dialect(), &Options::new("t.sbas")).err().unwrap().into_iter().map(|d| d.code).collect()
}
fn petcat() -> Option<PathBuf> { ["/Applications/vice-arm64-gtk3-3.10/bin/petcat", "/opt/homebrew/bin/petcat", "/usr/local/bin/petcat"].iter().map(PathBuf::from).find(|p| p.exists()) }

#[test]
fn labels_consts_and_numbering() {
    let src = "const BORDER = $d020\n@start:\n  print \"{clr}hello\"\n  for i = 0 to 15\n    poke BORDER, i\n  next\n  if peek(197) = 64 then goto @start\n  end\n";
    assert_eq!(text(src), "10 print\"{clr}hello\"\n20 fori=0to15\n30 poke53280,i\n40 next\n50 ifpeek(197)=64thengoto10\n60 end\n");
}

#[test]
fn block_if_else_while_repeat() {
    assert_eq!(text("if a = 1 then\n print \"one\"\nelse\n print \"other\"\nendif\nprint \"done\"\n"),
               "10 if(a=1)=0then40\n20 print\"one\"\n30 goto50\n40 print\"other\"\n50 print\"done\"\n");
    assert_eq!(text("i = 0\nwhile i < 3\n i = i + 1\nwend\nprint i\n"), "10 i=0\n20 if(i<3)=0then50\n30 i=i+1\n40 goto20\n50 printi\n");
    assert_eq!(text("repeat\n n = n + 1\nuntil n = 5\n"), "10 n=n+1\n20 if(n=5)=0then10\n");
    assert_eq!(text("if a then\n print 1\nendif\n"), "10 if(a)=0then30\n20 print1\n30 rem\n");
}

#[test]
fn long_variable_names_are_mapped_safely() {
    let d = dialect();
    let src = "score = 0\nlives = 3\ntotal = score + lives\ns$ = \"x\"\nname$ = s$ + \"y\"\ncount% = 5\nf = 1\norder = 2: format = 3: tomato = 4\nprint score; lives; total; name$; count%; order\n";
    let c = compile(src, &d, &Options::new("t.sbas")).unwrap();
    let map: std::collections::HashMap<_, _> = c.vars.iter().cloned().collect();
    let mut shorts: Vec<&String> = map.values().collect();
    shorts.sort();
    shorts.dedup();
    assert_eq!(shorts.len(), map.len(), "Kurznamen müssen eindeutig sein: {map:?}");
    assert_eq!(map["s$"], "S$");
    assert_eq!(map["f"], "F");
    for v in map.values() {
        let b = v.trim_end_matches(|c| c == '$' || c == '%');
        assert!(b.len() <= 2 && !["IF", "ON", "GO", "TO", "OR", "FN", "TI", "ST", "DS"].contains(&b), "gefährlicher Name {v}");
    }
    // Der Quelltext darf Schlüsselwörter in Namen enthalten (score ⊃ OR, total ⊃ TO, format ⊃ OR, tomato ⊃ TO)
    assert_eq!(map["score"].len(), 2);
}

#[test]
fn compiled_output_survives_listing_and_retokenizing() {
    let d = dialect();
    let src = "const MAX = 20\nscore = 0\nlives = 3\nprint \"{clr}{wht}score:\"; score\nfor i = 1 to MAX\n  total = total + i * 2\n  if total > 100 then goto @done\nnext i\n@done:\nprint left$(\"abc\", 2); chr$(65); tab(5); spc(2)\ndata 1, 2, hello, \"x y\"\non lives goto @done, @done\nrem Kommentar\n";
    let c = compile(src, &d, &Options::new("t.sbas")).unwrap();
    let text = to_classic_text(&parse_prg(&c.prg).unwrap(), &d);
    let again = build_prg(&tokenize_text(&text, &d, "x").unwrap(), 0x0801);
    assert_eq!(again, c.prg, "eigener Round-Trip:\n{text}");
    if let Some(p) = petcat() {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("petcat2");
        std::fs::create_dir_all(&dir).unwrap();
        let (prg, listing, re) = (dir.join("a.prg"), dir.join("a.txt"), dir.join("b.prg"));
        std::fs::write(&prg, &c.prg).unwrap();
        let l = Command::new(&p).arg("-2").arg(&prg).output().unwrap();
        let only_lines: String = String::from_utf8_lossy(&l.stdout).lines().filter(|x| x.trim_start().starts_with(|c: char| c.is_ascii_digit())).map(|x| format!("{x}\n")).collect();
        std::fs::write(&listing, only_lines).unwrap();
        assert!(Command::new(&p).args(["-w2", "-o"]).arg(&re).arg("--").arg(&listing).output().unwrap().status.success());
        assert_eq!(std::fs::read(&re).unwrap(), c.prg, "petcat-Listing und Neu-Tokenisierung weichen ab:\n{}", String::from_utf8_lossy(&l.stdout));
    }
}

#[test]
fn packing_merges_safe_lines_only() {
    let d = dialect();
    let mut o = Options::new("t.sbas");
    o.pack = true;
    let c = compile("a = 1\nb = 2\nif a = 1 then print \"x\"\nc = 3\n@lbl:\nd = 4\ngoto @lbl\nrem hi\ne = 5\n", &d, &o).unwrap();
    let t = to_classic_text(&parse_prg(&c.prg).unwrap(), &d);
    // if-Zeile darf angehängt werden, nimmt aber nichts mehr auf; Label-Zeile beginnt neu; nach rem kein Anhängen
    assert_eq!(t, "10 a=1:b=2:ifa=1thenprint\"x\"\n20 c=3\n30 d=4:goto30:rem hi\n40 e=5\n");
}

#[test]
fn structured_roundtrip_via_detokenizer() {
    let d = dialect();
    let src = "@start:\nprint \"hi\"\nfor i = 1 to 3\nnext\nif i = 4 then goto @start\non i goto @start, @end\n@end:\nend\n";
    let c = compile(src, &d, &Options::new("t.sbas")).unwrap();
    let s = to_structured_text(&parse_prg(&c.prg).unwrap(), &d);
    assert!(s.contains("@L10:") && s.contains("goto @L10") && s.contains("on i goto @L10,@L"));
    let c2 = compile(&s, &d, &Options::new("t2.sbas")).unwrap();
    assert_eq!(c2.prg, c.prg, "Struktur-Text → PRG muss identisch sein:\n{s}");
}

#[test]
fn external_constants_and_includes() {
    let d = dialect();
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("basic_inc");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib.sbas"), "const LIVES = 3\n@helper:\nreturn\n").unwrap();
    let mut o = Options::new(dir.join("main.sbas").to_string_lossy());
    o.consts = vec![("asm.start".into(), 2061)];
    let c = compile("include \"lib.sbas\"\ngosub @helper\nsys asm.start\nprint LIVES\nend\n", &d, &o).unwrap();
    assert_eq!(to_classic_text(&parse_prg(&c.prg).unwrap(), &d), "10 return\n20 gosub10\n30 sys2061\n40 print3\n50 end\n");
}

#[test]
fn errors_are_reported() {
    assert_eq!(codes("goto @nowhere\n"), vec!["B107"]);
    assert_eq!(codes("if a then\nprint 1\n"), vec!["B106"]);
    assert_eq!(codes("@a:\nprint 1\n@a:\nprint 2\n"), vec!["B104"]);
    assert_eq!(codes("else\n"), vec!["B106"]);
    assert_eq!(codes("print 1 { 2\n"), vec!["B100"]);
    assert_eq!(codes("print \"{nix}\"\n"), vec!["B100"]);
    assert_eq!(codes("const X = a + b\n"), vec!["B108"]);
    assert_eq!(codes("print a.b\n"), vec!["B100"]);
}

fn text_for(id: &str, src: &str) -> String {
    let d = Dialect::load(&data(), id).unwrap();
    let c = compile(src, &d, &Options::new("t.sbas")).unwrap_or_else(|e| panic!("{id}: {e:?}"));
    to_classic_text(&parse_prg(&c.prg).unwrap(), &d)
}

#[test]
fn elseif_chain() {
    let t = text("if a = 1 then\n print 1\nelseif a = 2 then\n print 2\nelse\n print 3\nendif\nprint 9\n");
    assert_eq!(t, "10 if(a=1)=0then40\n20 print1\n30 goto80\n40 if(a=2)=0then70\n50 print2\n60 goto80\n70 print3\n80 print9\n");
}

#[test]
fn exit_and_do_loop_are_lowered_without_native_support() {
    // C64 (CBM V2) hat kein DO/LOOP/EXIT: es entstehen Sprünge
    assert_eq!(text("do\n n = n + 1\n if n > 5 then goto @x\nloop\n@x:\nprint n\n"), "10 n=n+1\n20 ifn>5thengoto40\n30 goto10\n40 printn\n");
    assert_eq!(text("do while n < 3\n n = n + 1\nloop\nprint n\n"), "10 if(n<3)=0then40\n20 n=n+1\n30 goto10\n40 printn\n");
    assert_eq!(text("do\n n = n + 1\nloop until n = 3\nprint n\n"), "10 n=n+1\n20 if(n=3)=0then10\n30 printn\n");
    assert_eq!(text("do\n n = n + 1\nloop while n < 3\n"), "10 n=n+1\n20 ifn<3then10\n");
    assert_eq!(text("while 1\n n = n + 1\n if n > 3 then\n  exit\n endif\nwend\nprint n\n"),
               "10 if(1)=0then60\n20 n=n+1\n30 if(n>3)=0then50\n40 goto60\n50 goto10\n60 printn\n");
    assert_eq!(text("repeat\n n = n + 1\n if n = 2 then\n  exit\n endif\nuntil n = 9\nprint n\n"), "10 n=n+1\n20 if(n=2)=0then40\n30 goto50\n40 if(n=9)=0then10\n50 printn\n");
    assert_eq!(codes("exit\n"), vec!["B106"]);
    assert_eq!(codes("loop\n"), vec!["B106"]);
    assert_eq!(codes("do\nprint 1\n"), vec!["B106"]);
}

#[test]
fn do_loop_exit_are_native_on_basic7_basic35_basic65() {
    for id in ["basic7", "basic35", "basic65"] {
        let t = text_for(id, "do while n < 3\n n = n + 1\n if n = 2 then\n  exit\n endif\nloop\ndo\n n = n - 1\nloop until n = 0\nprint n\n");
        assert!(t.contains("dowhilen<3") && t.contains("loopuntiln=0"), "{id}: {t}");
        // exit im nativen do ist das Wort EXIT, kein Sprung
        assert!(t.contains("exit") && !t.contains("goto"), "{id}: {t}");
    }
}

#[test]
fn exit_in_while_inside_native_do_jumps_out_of_while_only() {
    let t = text_for("basic7", "do\n while 1\n  exit\n wend\n exit\nloop\n");
    assert!(t.contains("goto") && t.matches("exit").count() == 1, "{t}");
}

#[test]
fn procedures_with_parameters() {
    let src = "call plot(3, a + 1)\nend\nproc plot(px, py)\n poke 1024 + px, py\nendproc\ncall later\nproc later\n print \"x\"\nendproc\n";
    let t = text(src);
    assert_eq!(t, "10 px=3:py=a+1:gosub40\n20 end\n30 goto60\n40 poke1024+px,py\n50 return\n60 gosub80\n70 goto100\n80 print\"x\"\n90 return\n100 rem\n");
}

#[test]
fn example_structured_basic_compiles_for_every_commodore_system() {
    let example = sbas_dir().join("examples/c128/structured_basic/structured.sbas");
    if !example.is_file() { eprintln!("übersprungen: Beispiel nicht vorhanden"); return; }
    let src = std::fs::read_to_string(&example).unwrap();
    for id in ["cbm_v2", "basic7", "basic35", "basic65"] {
        let t = text_for(id, &src);
        assert_eq!(t.contains("do\n"), id != "cbm_v2", "{id}: natives do/loop nur auf C128, Plus/4, MEGA65\n{t}");
        assert!(t.contains("gosub") && t.contains("return"), "{id}");
    }
}

fn text_sys(id: &str, system: &str, src: &str) -> String {
    let d = Dialect::load(&data(), id).unwrap();
    let mut o = Options::new("t.sbas");
    o.system = Some(system.into());
    let c = compile(src, &d, &o).unwrap_or_else(|e| panic!("{system}: {e:?}"));
    to_classic_text(&parse_prg(&c.prg).unwrap(), &d)
}
fn codes_sys(system: &str, src: &str) -> Vec<&'static str> {
    let mut o = Options::new("t.sbas");
    o.system = Some(system.into());
    compile(src, &dialect(), &o).err().unwrap().into_iter().map(|d| d.code).collect()
}

#[test]
fn conditional_compilation_by_system() {
    let src = "#if SYSTEM_C64 or SYSTEM_C128\nprint \"cbm\"\n#elseif SYSTEM_PLUS4\nprint \"plus4\"\n#else\nprint \"anderes\"\n#endif\nprint \"ende\"\n";
    assert_eq!(text_sys("cbm_v2", "c64", src), "10 print\"cbm\"\n20 print\"ende\"\n");
    assert_eq!(text_sys("basic7", "c128", src), "10 print\"cbm\"\n20 print\"ende\"\n");
    assert_eq!(text_sys("basic35", "plus4", src), "10 print\"plus4\"\n20 print\"ende\"\n");
    assert_eq!(text_sys("basic65", "mega65", src), "10 print\"anderes\"\n20 print\"ende\"\n");
}

#[test]
fn inactive_branches_are_not_checked_and_may_nest() {
    // Der C64 kennt `sprite` nicht: im toten Zweig stört das nicht, auch nicht Unsinn oder ein fehlendes Include
    let src = "#if not SYSTEM_C64\n sprite 1,1\n include \"gibt_es_nicht.sbas\"\n @@@ ???\n#else\n print \"c64\"\n #if SYSTEM_C64 and not SYSTEM_PLUS4\n  print \"innen\"\n #endif\n#endif\n";
    assert_eq!(text_sys("cbm_v2", "c64", src), "10 print\"c64\"\n20 print\"innen\"\n");
    // Ein inaktiver äußerer Block macht auch den inneren inaktiv, selbst wenn dessen Bedingung stimmt
    assert_eq!(text_sys("basic35", "plus4", "#if SYSTEM_C64\n#if SYSTEM_PLUS4 or 1\nprint 1\n#else\nprint 2\n#endif\n#endif\nprint 3\n"), "10 print3\n");
}

#[test]
fn conditions_support_comparison_and_defined_constants() {
    let d = dialect();
    let mut o = Options::new("t.sbas");
    o.system = Some("c64".into());
    o.consts = vec![("DEBUG".into(), 1), ("LEVEL".into(), 3)];
    let c = compile("#if DEBUG and LEVEL = 3\nprint 1\n#endif\n#if LEVEL <> 3\nprint 2\n#endif\n", &d, &o).unwrap();
    assert_eq!(to_classic_text(&parse_prg(&c.prg).unwrap(), &d), "10 print1\n");
}

#[test]
fn conditional_errors() {
    assert_eq!(codes_sys("c64", "#if SYSTEM_GIBTS_NICHT\nprint 1\n#endif\n"), vec!["B111"]);
    assert_eq!(codes_sys("c64", "#if SYSTEM_C64\nprint 1\n"), vec!["B111"]);
    assert_eq!(codes_sys("c64", "#endif\n"), vec!["B111"]);
    assert_eq!(codes_sys("c64", "#if 1\n#else\n#else\n#endif\n"), vec!["B111"]);
    assert_eq!(codes_sys("c64", "#if 1 and\n#endif\n"), vec!["B111"]);
}

#[test]
fn include_with_system_placeholder() {
    let dir = std::env::temp_dir().join("vc_sbas_hal");
    std::fs::create_dir_all(dir.join("hal")).unwrap();
    std::fs::write(dir.join("hal/c64.sbas"), "print \"hal c64\"\n").unwrap();
    std::fs::write(dir.join("hal/plus4.sbas"), "print \"hal plus4\"\n").unwrap();
    for (id, sys, want) in [("cbm_v2", "c64", "10 print\"hal c64\"\n"), ("basic35", "plus4", "10 print\"hal plus4\"\n")] {
        let d = Dialect::load(&data(), id).unwrap();
        let mut o = Options::new(dir.join("main.sbas").to_string_lossy());
        o.system = Some(sys.into());
        let c = compile("include \"hal/{system}.sbas\"\n", &d, &o).unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(to_classic_text(&parse_prg(&c.prg).unwrap(), &d), want);
    }
    // Datei für das Ziel fehlt, oder das Ziel ist unbekannt
    let mut o = Options::new(dir.join("main.sbas").to_string_lossy());
    o.system = Some("mega65".into());
    let e = compile("include \"hal/{system}.sbas\"\n", &dialect(), &o).err().unwrap();
    assert_eq!(e[0].code, "B110");
    let e = compile("include \"hal/{system}.sbas\"\n", &dialect(), &Options::new(dir.join("main.sbas").to_string_lossy())).err().unwrap();
    assert_eq!(e[0].code, "B111");
}

#[test]
fn restore_with_label_is_rejected_on_basic_v2_only() {
    assert_eq!(codes("@d:\ndata 1,2\nrestore @d\n"), vec!["B112"]);
    assert!(text_for("basic7", "@d:\ndata 1,2\nrestore @d\n").contains("restore"));
    assert!(text("data 1,2\nrestore\n").contains("restore"));
}
