// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_ide::build::Ctx;
use vc_ide::projfile::*;

/// Ordner mit den (nicht im Repository liegenden) `.sbas`-Beispielen: Umgebungsvariable `VC_SBAS_EXAMPLES`; ohne sie überspringen sich die Tests
fn sbas_dir() -> PathBuf { std::env::var("VC_SBAS_EXAMPLES").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/nicht-gesetzt")) }

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc_projfile_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}
fn data() -> PathBuf { vc_ide::find_data_dir(&PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap() }
const PRG: &str = ".target c64\n.org $c000\nstart: lda #1\n sta $d020\n rts\n";

#[test]
fn roundtrip_keeps_unknown_fields() {
    let t = r#"{"format":1,"project":{"name":"X","autor":"ich"},"targets":[{"id":"c64","system":"c64","extra":5}],"sources":[],"main":"a.asm","zusatz":{"a":1}}"#;
    let pf: ProjectFile = serde_json::from_str(t).unwrap();
    let back: serde_json::Value = serde_json::to_value(&pf).unwrap();
    assert_eq!(back["zusatz"]["a"], 1); assert_eq!(back["project"]["autor"], "ich"); assert_eq!(back["targets"][0]["extra"], 5);
}

#[test]
fn main_defaults_to_first_source_and_validate_reports_problems() {
    let root = tmp("val");
    std::fs::write(root.join("a.asm"), PRG).unwrap();
    let mut pf = ProjectFile { targets: vec![TargetDef { id: "c64".into(), ..Default::default() }], sources: vec![Source { targets: vec!["c64".into()], kind: "asm".into(), files: vec!["a.asm".into(), "fehlt.asm".into()] , ..Default::default() }], ..Default::default() };
    assert_eq!(pf.main_file().as_deref(), Some("a.asm"));
    assert!(pf.validate(&root).iter().any(|p| p.contains("fehlt.asm")));
    pf.main = Some("x.asm".into()); pf.sources[0].targets = vec!["atari8".into()]; pf.disk = Some(DiskSpec { format: "zip".into(), ..Default::default() });
    let v = pf.validate(&root).join("\n");
    assert!(v.contains("main:") && v.contains("atari8") && v.contains("Format"), "{v}");
}

#[test]
fn builds_all_sources_continues_after_error_and_makes_disk() {
    let root = tmp("all");
    std::fs::write(root.join("a.asm"), PRG).unwrap(); std::fs::write(root.join("b.asm"), PRG).unwrap(); std::fs::write(root.join("kaputt.asm"), ".target c64\n lda #\n").unwrap();
    let mut pf = ProjectFile { targets: vec![TargetDef { id: "c64".into(), ..Default::default() }], main: Some("b.asm".into()),
        sources: vec![Source { targets: vec!["c64".into()], kind: "asm".into(), files: vec!["a.asm".into(), "kaputt.asm".into(), "b.asm".into()] , ..Default::default() }],
        disk: Some(DiskSpec { format: "d64".into(), name: "TEST".into(), ..Default::default() }), ..Default::default() };
    let data = data(); let c = Ctx { root: &root, data: &data };
    let r = build_project(&c, &pf, None);
    assert_eq!(r["ok"], false);
    let res = r["results"].as_array().unwrap();
    assert_eq!(res.len(), 3, "ein Fehler hält die übrigen Quellen nicht auf");
    assert_eq!(res[2]["ok"], true); assert_eq!(r["main_artifact"], "build/b.prg");
    assert!(r.get("disk").is_none(), "bei Fehlern keine Diskette");
    pf.sources[0].files.remove(1);
    let r = build_project(&c, &pf, None);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["disk"]["programs"], serde_json::json!(["A", "B"]));
    assert!(root.join("build/all.d64").is_file() || root.join(r["disk"]["file"].as_str().unwrap()).is_file());
    // nur das Startprogramm
    let only = build_project(&c, &pf, pf.main_file().as_deref());
    assert_eq!(only["results"].as_array().unwrap().len(), 1); assert_eq!(only["main_artifact"], "build/b.prg");
    let atr = make_disk(&root, &pf, &DiskSpec { format: "atr".into(), files: vec!["a.asm".into()], file: "build/t.atr".into(), ..Default::default() }, &[("a.asm".into(), "build/a.prg".into())]).unwrap();
    assert_eq!(atr["programs"], serde_json::json!(["A.PRG"]));
}

#[test]
#[ignore = ".sbas ist vorerst abgeschaltet"]
fn full_demo_example_builds_to_disk_with_menu_first() {
    // Beispielprojekt: BASIC-Menü (Startprogramm) + Assembler-Player, gebündelt auf eine Diskette; MENU muss zuerst stehen (Autostart)
    let src = sbas_dir().join("examples/c64/full_demo");
    let root = tmp("komplett");
    fn copy(a: &std::path::Path, b: &std::path::Path) { std::fs::create_dir_all(b).unwrap(); for e in std::fs::read_dir(a).unwrap().flatten() { let (p, q) = (e.path(), b.join(e.file_name())); if e.file_name() == "build" { continue; } if p.is_dir() { copy(&p, &q) } else { std::fs::copy(&p, &q).unwrap(); } } }
    copy(&src, &root);
    let pf = ProjectFile::load(&find(&root).unwrap()).unwrap();
    assert!(pf.validate(&root).is_empty(), "{:?}", pf.validate(&root));
    assert_eq!(pf.main_file().as_deref(), Some("menu.sbas"));
    assert!(pf.run_disk(&root).unwrap().ends_with("build/full_demo.d64"), "F5 startet von der Diskette");
    let data = data();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["disk"]["programs"], serde_json::json!(["MENU", "PLAYER"]));
    let prg = std::fs::read(root.join("build/player.prg")).unwrap();
    assert_eq!(&prg[..2], &[0x00, 0x20], "Player lädt nach $2000");
    assert_eq!(prg[2], 0x4c, "erstes Byte ist JMP – daran erkennt menu.sbas, dass der Player geladen ist");
    assert!(prg.len() < 0x1000, "Player muss unter $3000 bleiben");
}

#[test]
fn run_disk_only_when_main_is_first_program_on_a_cbm_disk() {
    let mut pf = ProjectFile { targets: vec![TargetDef { id: "c64".into(), ..Default::default() }],
        sources: vec![Source { targets: vec!["c64".into()], kind: "asm".into(), files: vec!["a.asm".into(), "b.asm".into()] , ..Default::default() }],
        disk: Some(DiskSpec { format: "d64".into(), ..Default::default() }), main: Some("a.asm".into()), ..Default::default() };
    let root = PathBuf::from("/x/proj");
    assert_eq!(pf.run_disk(&root), Some(PathBuf::from("/x/proj/build/proj.d64")));
    pf.main = Some("b.asm".into());
    assert_eq!(pf.run_disk(&root), None, "Autostart lädt den ersten Eintrag – dann das einzelne Programm starten");
    pf.main = Some("a.asm".into()); pf.disk.as_mut().unwrap().format = "atr".into();
    assert_eq!(pf.run_disk(&root), None);
}

#[test]
fn native_bas_sources_build_in_a_project_and_dialect_is_per_group() {
    let root = tmp("nativ");
    std::fs::write(root.join("a.bas"), "print \"hi\"\n@l:\na=a+1\ngoto @l\n").unwrap();
    std::fs::write(root.join("b.bas"), "10 print \"nummeriert\"\n20 goto 10\n").unwrap();
    let pf = ProjectFile { targets: vec![TargetDef { id: "c64".into(), ..Default::default() }], main: Some("a.bas".into()),
        sources: vec![Source { targets: vec!["c64".into()], kind: "basic".into(), files: vec!["a.bas".into()], dialect: "cbm_v2".into(), ..Default::default() }, Source { targets: vec!["c64".into()], kind: "basic".into(), files: vec!["b.bas".into()], ..Default::default() }], ..Default::default() };
    assert_eq!(pf.dialect_of("a.bas").as_deref(), Some("cbm_v2"));
    assert_eq!(pf.dialect_of("b.bas"), None);
    let data = data();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["results"][0]["labels"], serde_json::json!([{ "name": "l", "line": 20 }]));
    let prg = std::fs::read(root.join("build/a.prg")).unwrap();
    assert_eq!(&prg[..2], &[0x01, 0x08]);
    // Fehler im Label: Zeile der Quelle wird gemeldet
    std::fs::write(root.join("a.bas"), "print 1\ngoto @weg\n").unwrap();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], false);
    assert_eq!(r["results"][0]["diagnostics"][0]["line"], 2);
}

#[test]
fn turbo_basic_xl_dialect_builds_from_a_project() {
    let root = tmp("turbo");
    std::fs::write(root.join("t.bas"), "proc p\nendproc\nexec p\nwhile a<3\na=a+1\nwend\n").unwrap();
    let pf = ProjectFile { targets: vec![TargetDef { id: "atari8".into(), ..Default::default() }], main: Some("t.bas".into()),
        sources: vec![Source { targets: vec!["atari8".into()], kind: "basic".into(), files: vec!["t.bas".into()], dialect: "turbo_basic_xl".into(), ..Default::default() }], ..Default::default() };
    let data = data();
    let r = build_project(&Ctx { root: &root, data: &data }, &pf, None);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["results"][0]["dialect"], "turbo_basic_xl");
    let bas = std::fs::read(root.join("build/t.bas")).unwrap();
    assert!(bas.windows(2).any(|w| w == [0x4F, 0x80]), "PROC-Token ($4F) mit Label fehlt");
    // Ohne den Turbo-Dialekt ist PROC unbekannt
    let mut plain = pf.clone(); plain.sources[0].dialect = String::new();
    let r = build_project(&Ctx { root: &root, data: &data }, &plain, None);
    assert_eq!(r["ok"], false);
}
