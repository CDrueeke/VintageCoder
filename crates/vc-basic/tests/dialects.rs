// SPDX-License-Identifier: GPL-3.0-or-later
//! Dialektmechanismus: Metadaten, Vererbung (`base`), gemeinsame Schnittstelle, Auswahl je Projekt/Datei (`#dialect`).
use std::path::{Path, PathBuf};
use vc_basic::dialects::{self, AnyDialect};
use vc_basic::{number_lines, Dialect, PlainOptions};

fn data() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn all_shipped_dialects_load_with_metadata() {
    let all = dialects::list(&data());
    let ids: Vec<&str> = all.iter().map(|d| d.id.as_str()).collect();
    for want in ["cbm_v2", "vic_basic_2", "basic35", "basic7", "basic65", "atari_basic", "turbo_basic_xl"] { assert!(ids.contains(&want), "{want} fehlt in {ids:?}"); }
    for d in &all {
        assert!(!d.name.is_empty() && !d.jumps.is_empty(), "{}", d.id);
        assert!(d.jumps.iter().any(|j| j == "GOTO"), "{}", d.id);
        AnyDialect::load(&data(), &d.id).unwrap_or_else(|e| panic!("{}: {e}", d.id));
        assert!(!d.provenance_source.is_empty() && !d.provenance_checked.is_empty(), "{}: Herkunft fehlt", d.id);
    }
}

#[test]
fn vic_basic_2_inherits_from_v2() {
    let v2 = Dialect::load(&data(), "cbm_v2").unwrap();
    let vic = Dialect::load(&data(), "vic_basic_2").unwrap();
    assert_eq!(vic.id, "vic_basic_2");
    assert_eq!(vic.tokens, v2.tokens, "Wörter und Codes wie V2");
    assert_eq!(vic.max_line_len, 88);
    assert_eq!(v2.max_line_len, 80);
    assert_eq!(vic.var_sig_chars, 2);
    let i = dialects::info(&data(), "vic_basic_2").unwrap();
    assert_eq!(i.base.as_deref(), Some("cbm_v2"));
    assert_eq!(i.systems, ["vic20"]);
    assert_eq!(i.family, "cbm");
}

#[test]
fn own_tokens_come_before_inherited_ones_and_capabilities_merge() {
    let dir = std::env::temp_dir().join(format!("vc_dia_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("basic")).unwrap();
    std::fs::copy(data().join("basic/cbm_v2.dialect.json"), dir.join("basic/cbm_v2.dialect.json")).unwrap();
    std::fs::write(dir.join("basic/ext.dialect.json"), r#"{"id":"ext","name":"Ext","family":"cbm","base":"cbm_v2","tokens":[["XYZ","$CC"]],"capabilities":{"graphic":true},"maxLineLen":90}"#).unwrap();
    let d = Dialect::load(&dir, "ext").unwrap();
    assert_eq!(d.tokens[0].0, "XYZ");
    assert_eq!(d.token("XYZ"), Some(0xCC));
    assert_eq!(d.token("PRINT"), Some(0x99));
    assert_eq!(d.max_line_len, 90);
    let v = dialects::load_json(&dir, "ext").unwrap();
    assert_eq!(v["capabilities"]["graphic"], true);
    assert_eq!(v["capabilities"]["trap"], false, "Fähigkeiten der Basis bleiben");
    // Zyklus
    std::fs::write(dir.join("basic/a.dialect.json"), r#"{"id":"a","base":"b","tokens":[]}"#).unwrap();
    std::fs::write(dir.join("basic/b.dialect.json"), r#"{"id":"b","base":"a","tokens":[]}"#).unwrap();
    assert!(dialects::load_json(&dir, "a").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn any_dialect_covers_both_families() {
    let c = AnyDialect::load(&data(), "basic65").unwrap();
    let a = AnyDialect::load(&data(), "turbo_basic_xl").unwrap();
    assert!(!c.is_atari() && a.is_atari());
    assert!(c.keywords().iter().any(|k| k == "GRAPHIC") && a.keywords().iter().any(|k| k == "EXEC"));
    assert_eq!(c.id(), "basic65");
    assert!(dialects::info(&data(), "atari_basic").unwrap().quote_comment);
    assert!(!dialects::info(&data(), "cbm_v2").unwrap().quote_comment);
    assert_eq!(vc_basic::dialect_family(&data(), "vic_basic_2").unwrap(), "cbm");
}

#[test]
fn directive_overrides_project_choice() {
    let src = "; Kopf\n#dialect basic65\n10 PRINT\n";
    assert_eq!(dialects::dialect_directive(src), Some(("basic65".to_string(), 2)));
    assert_eq!(dialects::dialect_directive("#DIALECT  vic_basic_2 \n").unwrap().0, "vic_basic_2");
    assert_eq!(dialects::dialect_directive("#dialects x\n"), None);
    assert_eq!(dialects::dialect_directive("PRINT \"#dialect x\"\n"), None);
    let r = dialects::resolve(src, "a.bas", "cbm_v2", Some("c64"), &data()).unwrap();
    assert!(r.from_file && r.id == "basic65");
    assert_eq!(r.warnings[0].code, "B170", "BASIC 65 passt nicht zum C64");
    assert_eq!(r.warnings[0].arg_text("system"), "c64");
    let r = dialects::resolve("10 PRINT\n", "a.bas", "cbm_v2", Some("c64"), &data()).unwrap();
    assert!(!r.from_file && r.id == "cbm_v2" && r.warnings.is_empty());
    let r = dialects::resolve("10 PRINT\n", "a.bas", "vic_basic_2", Some("vic20"), &data()).unwrap();
    assert!(r.warnings.is_empty());
    let e = dialects::resolve("#dialect basic99\n", "a.bas", "cbm_v2", None, &data()).unwrap_err();
    assert_eq!(e[0].code, "B178");
    assert_eq!(e[0].line, 1);
    assert!(e[0].arg_text("known").contains("basic65"));
}

#[test]
fn directive_line_is_not_a_program_line() {
    let n = number_lines("#dialect vic_basic_2\nPRINT 1\nPRINT 2\n", &PlainOptions::new("a.bas", 63999)).unwrap();
    assert_eq!(n.text, "\n10 PRINT 1\n20 PRINT 2\n");
}

#[test]
fn vic20_target_uses_vic_basic_2_and_builds_like_v2() {
    let t = vc_target::load_target(&data(), "vic20").unwrap();
    assert_eq!(t.basic_dialect.as_deref(), Some("vic_basic_2"));
    let src = "10 PRINT \"HALLO\":FOR I=1 TO 3:POKE 36879,I:NEXT\n20 GOTO 10\n";
    let prg = |id: &str| { let d = Dialect::load(&data(), id).unwrap(); vc_basic::compile_plain_cbm(src, &d, &PlainOptions::new("t.bas", 63999), 0x1001).unwrap().prg };
    assert_eq!(prg("vic_basic_2"), prg("cbm_v2"), "gleiche Wörter und Codes: byte-gleich");
}
