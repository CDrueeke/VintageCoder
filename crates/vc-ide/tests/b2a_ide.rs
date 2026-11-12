// SPDX-License-Identifier: GPL-3.0-or-later
//! BASIC → Assembler in der IDE: Konvertieren, Überschreibschutz, Eintrag in die Projektdatei, Bauen des Ergebnisses.
use std::path::PathBuf;
use vc_ide::b2a::{convert_file, default_out, is_generated, Req, MARK};
use vc_ide::build::{build, Ctx};
use vc_ide::projfile::ProjectFile;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-b2a-ide-{tag}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(dir.join("basic")).unwrap();
    std::fs::write(dir.join("basic/zaehler.bas"), "@start:\ni% = 0\n@loop:\nprint i%\ni% = i% + 1\nif i% < 3 then @loop\nend\n").unwrap();
    std::fs::write(dir.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"basic","files":["basic/zaehler.bas"]}],"main":"basic/zaehler.bas"}"#).unwrap();
    dir
}

#[test]
fn converts_registers_and_builds() {
    let dir = project("ok");
    let r = convert_file(&dir, &data(), &Req::new("basic/zaehler.bas")).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(r.out, "basic/zaehler_asm.asm");
    assert!(r.registered && r.stage == 1 && r.lines == 5, "{r:?}");
    let asm = std::fs::read_to_string(dir.join(&r.out)).unwrap();
    assert!(asm.starts_with(MARK) && asm.contains("basic/zaehler.bas"));
    assert!(dir.join("basic/zaehler_asm.b2a.json").is_file());
    // Projektdatei: BASIC bleibt Quelle, Ergebnis kommt als Assembler dazu
    let pf = ProjectFile::load(&dir.join("p.vcprj")).unwrap();
    let files = pf.source_files();
    assert!(files.iter().any(|f| f.0 == "basic/zaehler.bas" && f.1 == "basic"));
    assert!(files.iter().any(|f| f.0 == "basic/zaehler_asm.asm" && f.1 == "asm"));
    assert!(pf.validate(&dir).is_empty(), "{:?}", pf.validate(&dir));
    // das Ergebnis baut
    let out = build(&Ctx { root: &dir, data: &data() }, "basic/zaehler_asm.asm", Some("c64")).unwrap();
    assert_eq!(out["ok"], true, "{out}");
    // erneutes Konvertieren ersetzt die erzeugte Datei und trägt nichts doppelt ein
    let r2 = convert_file(&dir, &data(), &Req::new("basic/zaehler.bas")).unwrap();
    assert!(!r2.registered);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn never_overwrites_hand_written_files() {
    let dir = project("keep");
    std::fs::write(dir.join("basic/zaehler_asm.asm"), "; meine Datei\n.target c64\n").unwrap();
    assert!(!is_generated(&dir.join("basic/zaehler_asm.asm")));
    assert_eq!(default_out(&dir, "basic/zaehler.bas"), "basic/zaehler_asm2.asm");
    let mut req = Req::new("basic/zaehler.bas");
    req.out = "basic/zaehler_asm.asm".into();
    let e = convert_file(&dir, &data(), &req).unwrap_err();
    assert!(e[0].message.contains("nicht überschrieben"), "{e:?}");
    assert_eq!(std::fs::read_to_string(dir.join("basic/zaehler_asm.asm")).unwrap(), "; meine Datei\n.target c64\n");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn reports_errors_and_unsupported_targets() {
    let dir = project("err");
    std::fs::write(dir.join("basic/kaputt.bas"), "goto 999\n").unwrap();
    let e = convert_file(&dir, &data(), &Req::new("basic/kaputt.bas")).unwrap_err();
    assert!(e.iter().any(|d| d.code == "E201"), "{e:?}");
    assert!(!dir.join("basic/kaputt_asm.asm").exists());
    let mut r = Req::new("basic/zaehler.bas");
    r.target = "vic20".into();
    assert!(convert_file(&dir, &data(), &r).is_err());
    r.target = "atari8".into();
    let rep = convert_file(&dir, &data(), &r).expect("atari8");
    assert!(std::fs::read_to_string(dir.join(&rep.out)).unwrap().contains(".target atari8"));
    r.target = "plus4".into();
    let rep = convert_file(&dir, &data(), &r).expect("plus4");
    assert!(std::fs::read_to_string(dir.join(&rep.out)).unwrap().contains(".target plus4"));
    assert!(convert_file(&dir, &data(), &Req::new("p.vcprj")).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn line_map_translates_between_basic_lines_and_addresses() {
    use vc_ide::b2a::load_maps;
    let dir = project("map");
    let r = convert_file(&dir, &data(), &Req::new("basic/zaehler.bas")).unwrap();
    let pf = ProjectFile::load(&dir.join("p.vcprj")).unwrap();
    let maps = load_maps(&dir, &pf);
    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].source, "basic/zaehler.bas");
    assert_eq!(maps[0].asm, r.out);
    // Symbole des Assemblers
    let cpu = vc_target::load_cpu(&data(), "6510").unwrap();
    let mut ao = vc_asm::Options::new("t.asm", vc_asm::Policy::Allow);
    ao.include_dirs.push(data().join("sys"));
    let asm = std::fs::read_to_string(dir.join(&r.out)).unwrap();
    let out = vc_asm::assemble(&asm, &cpu, &ao).unwrap_or_else(|e| panic!("{e:?}"));
    let sym = |n: &str| out.symbols.get(n).copied();
    // Quelle: Zeile 1 = "@start:", Zeile 2 = "i% = 0" (BASIC-Zeile 10), Zeile 4 = "print i%" (Zeile 20)
    let a10 = maps[0].addr_of(&sym, 2).expect("Zeile 2");
    let a20 = maps[0].addr_of(&sym, 4).expect("Zeile 4");
    assert!(a20 > a10);
    assert_eq!(maps[0].addr_of(&sym, 1), None, "Labelzeile ohne Anweisung hat keine Adresse");
    assert_eq!(maps[0].line_at(&sym, a10), Some((10, 2)));
    assert_eq!(maps[0].line_at(&sym, a20 + 3), Some((20, 4)));
    assert_eq!(maps[0].line_at(&sym, a20), Some((20, 4)));
    let fall = sym("b2a_fall").unwrap() as u16;
    assert_eq!(maps[0].line_at(&sym, fall), None, "Laufzeit hat keine BASIC-Zeile");
    assert_eq!(maps[0].line_at(&sym, 0x0801), None, "vor dem Programm");
    // nächste BASIC-Zeile: hinter Zeile 10 beginnt eine spätere Zeile, hinter der letzten das Ende des Programmteils, in der Laufzeit nichts
    let n10 = maps[0].next_line_addr(&sym, a10).expect("nach Zeile 10");
    assert!(n10 > a10 && n10 <= a20, "{n10:04X} zwischen {a10:04X} und {a20:04X}");
    assert_eq!(maps[0].line_at(&sym, n10).map(|l| l.0 > 10), Some(true));
    assert_eq!(maps[0].next_line_addr(&sym, a20 + 1).map(|a| a > a20), Some(true));
    assert_eq!(maps[0].next_line_addr(&sym, fall), None);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn project_build_converts_a_basic_group_with_convert() {
    let dir = project("conv");
    // Gruppe mit `convert`: gebaut wird das Ergebnis der Konvertierung, die .bas bleibt die Quelle
    std::fs::write(dir.join("p.vcprj"), r#"{"format":1,"project":{"name":"p"},"targets":[{"id":"c64","system":"c64"}],"sources":[{"targets":["c64"],"kind":"basic","files":["basic/zaehler.bas"],"convert":{"target":"c64","out":"build/gen/zaehler.asm"}}],"main":"basic/zaehler.bas"}"#).unwrap();
    let pf = ProjectFile::load(&dir.join("p.vcprj")).unwrap();
    assert_eq!(pf.convert_of("basic/zaehler.bas").unwrap().out, "build/gen/zaehler.asm");
    let out = vc_ide::projfile::build_project(&Ctx { root: &dir, data: &data() }, &pf, None);
    assert_eq!(out["ok"], true, "{out}");
    assert_eq!(out["results"][0]["converted"], "build/gen/zaehler.asm");
    assert!(is_generated(&dir.join("build/gen/zaehler.asm")) && dir.join("build/gen/zaehler.b2a.json").is_file());
    let art = out["main_artifact"].as_str().unwrap();
    assert!(dir.join(art).is_file() && art.contains("zaehler"), "{art}");
    // die Einstellung bleibt beim Speichern erhalten; ohne `convert` wird das BASIC selbst gebaut
    pf.save(&dir.join("p2.vcprj")).unwrap();
    assert!(std::fs::read_to_string(dir.join("p2.vcprj")).unwrap().contains("\"convert\""));
    let mut plain = pf.clone(); plain.sources[0].convert = None;
    let out2 = vc_ide::projfile::build_project(&Ctx { root: &dir, data: &data() }, &plain, None);
    assert_eq!(out2["ok"], true, "{out2}"); assert!(out2["results"][0]["converted"].is_null());
    // Fehler der Konvertierung (nicht übersetzbar) landen in den Diagnosen
    std::fs::write(dir.join("basic/zaehler.bas"), "10 open 1,8,15\n").unwrap();
    let bad = vc_ide::projfile::build_project(&Ctx { root: &dir, data: &data() }, &pf, None);
    assert_eq!(bad["ok"], false, "{bad}");
    assert!(bad["results"][0]["diagnostics"].as_array().map_or(false, |d| !d.is_empty()));
    std::fs::remove_dir_all(&dir).ok();
}
