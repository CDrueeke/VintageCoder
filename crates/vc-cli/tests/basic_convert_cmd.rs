// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc basic --to-labels|--to-numbers|--renumber` und `#dialect`: Umwandlung, Vorschau, Schreiben samt eingebundener Dateien, Bedeutungstreue.
use std::path::PathBuf;
use std::process::Command;

fn vcc() -> Command { let mut c = Command::new(env!("CARGO_BIN_EXE_vcc")); c.env("VC_LANG", "de").env("VC_DATA", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")); c }
fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-conv-{name}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }
const LOOP: &str = "10 REM HAUPTSCHLEIFE\n20 FOR I=1 TO 10\n30 PRINT I\n40 NEXT I\n50 GOTO 20\n";

#[test]
fn to_labels_prints_text_and_preview() {
    let d = tmp("l"); let f = d.join("a.bas"); std::fs::write(&f, LOOP).unwrap();
    let o = vcc().args(["basic", "--to-labels", f.to_str().unwrap()]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8_lossy(&o.stdout), "REM HAUPTSCHLEIFE\n@hauptschleife: FOR I=1 TO 10\nPRINT I\nNEXT I\nGOTO @hauptschleife\n");
    assert!(String::from_utf8_lossy(&o.stderr).contains("1 Labels neu"));
    let o = vcc().args(["basic", "--to-labels", f.to_str().unwrap(), "--preview"]).output().unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("20 FOR I=1 TO 10 → @hauptschleife: FOR I=1 TO 10") && std::fs::read_to_string(&f).unwrap() == LOOP, "{out}");
}

#[test]
fn write_converts_included_files_and_the_program_stays_identical() {
    let d = tmp("w");
    std::fs::write(d.join("lib.bas"), "@unter: PRINT \"U\"\nRETURN\n").unwrap();
    let f = d.join("m.bas");
    std::fs::write(&f, "GOSUB @unter\nEND\n#include \"lib.bas\"\n").unwrap();
    let o = vcc().args(["basic", "--to-numbers", f.to_str().unwrap(), "--write"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(std::fs::read_to_string(&f).unwrap(), "10 GOSUB 30\n20 END\n#include \"lib.bas\"\n");
    assert_eq!(std::fs::read_to_string(d.join("lib.bas")).unwrap(), ";@unter\n30 PRINT \"U\"\n40 RETURN\n");
}

#[test]
fn converted_program_is_byte_identical() {
    let d = tmp("b");
    let f = d.join("a.bas"); std::fs::write(&f, LOOP).unwrap();
    let before = d.join("before.prg");
    assert!(vcc().args(["basic", f.to_str().unwrap(), "-o", before.to_str().unwrap()]).output().unwrap().status.success());
    for kind in ["--to-labels", "--to-numbers"] {
        assert!(vcc().args(["basic", kind, f.to_str().unwrap(), "--write"]).output().unwrap().status.success());
        let after = d.join("after.prg");
        assert!(vcc().args(["basic", f.to_str().unwrap(), "-o", after.to_str().unwrap()]).output().unwrap().status.success());
        assert_eq!(std::fs::read(&before).unwrap(), std::fs::read(&after).unwrap(), "{kind}");
    }
}

#[test]
fn computed_target_fails_with_b162_and_renumber_works() {
    let d = tmp("c"); let f = d.join("a.bas"); std::fs::write(&f, "10 PRINT\n20 GOTO A*10\n").unwrap();
    let o = vcc().args(["basic", "--to-labels", f.to_str().unwrap()]).output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success() && err.contains("B161") && err.contains("B162"), "{err}");
    let f2 = d.join("b.bas"); std::fs::write(&f2, "5 PRINT\n9 GOTO 5\n").unwrap();
    let o = vcc().args(["basic", "--renumber", f2.to_str().unwrap(), "--start", "100", "--step", "5"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "100 PRINT\n105 GOTO 100\n");
}

#[test]
fn dialect_directive_selects_the_dialect() {
    let d = tmp("d");
    let f = d.join("a.bas"); std::fs::write(&f, "#dialect basic65\n10 TRAP 20\n20 END\n").unwrap();
    let o = vcc().args(["basic", "--to-labels", f.to_str().unwrap(), "--target", "mega65"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("TRAP @z20"));
    let f = d.join("b.bas"); std::fs::write(&f, "#dialect gibtsnicht\n10 END\n").unwrap();
    let o = vcc().args(["basic", f.to_str().unwrap(), "--target", "c64"]).output().unwrap();
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("B178"));
}

#[test]
fn define_and_system_condition_work_in_the_plain_build() {
    let d = tmp("def");
    let f = d.join("a.bas"); std::fs::write(&f, "#define SCR 1024\n#if SYSTEM_C64\nPOKE SCR,LEVEL\n#endif\n").unwrap();
    let o1 = d.join("o1.prg"); let o2 = d.join("o2.prg");
    assert!(vcc().args(["basic", f.to_str().unwrap(), "--target", "c64", "-D", "LEVEL=7", "-o", o1.to_str().unwrap()]).output().unwrap().status.success());
    let g = d.join("b.bas"); std::fs::write(&g, "POKE 1024,7\n").unwrap();
    assert!(vcc().args(["basic", g.to_str().unwrap(), "--target", "c64", "-o", o2.to_str().unwrap()]).output().unwrap().status.success());
    assert_eq!(std::fs::read(&o1).unwrap(), std::fs::read(&o2).unwrap());
    let h = d.join("c.bas"); std::fs::write(&h, "#define PRINT 1\n").unwrap();
    let o = vcc().args(["basic", h.to_str().unwrap(), "-o", d.join("c.prg").to_str().unwrap()]).output().unwrap();
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("B172"));
}

#[test]
fn blob_directive_builds_machine_code_behind_the_text() {
    let d = tmp("blob");
    std::fs::write(d.join("rahmen.asm"), ".org asm_base\nstart:\n    lda #2\n    sta $d020\n    rts\n").unwrap();
    let f = d.join("t.bas"); std::fs::write(&f, "#asm \"rahmen.asm\"\nsys asm.start\n").unwrap();
    let out = d.join("t.prg");
    let o = vcc().args(["basic", f.to_str().unwrap(), "--target", "c64", "-o", out.to_str().unwrap()]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("Maschinencode: rahmen.asm bei $"));
    let prg = std::fs::read(&out).unwrap();
    assert_eq!(&prg[prg.len() - 6..], &[0xA9, 0x02, 0x8D, 0x20, 0xD0, 0x60]);
}

#[test]
fn numbering_start_step_directive_and_overflow() {
    let d = tmp("num"); let f = d.join("a.bas"); std::fs::write(&f, "PRINT 1\nPRINT 2\n").unwrap();
    let o = vcc().args(["basic", "--to-numbers", f.to_str().unwrap(), "--start", "100", "--step", "5"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "100 PRINT 1\n105 PRINT 2\n");
    // die Datei schlägt die Kommandozeile
    let g = d.join("b.bas"); std::fs::write(&g, "#numbering start=1 step=1\nPRINT 1\nPRINT 2\n").unwrap();
    let o = vcc().args(["basic", "--to-numbers", g.to_str().unwrap(), "--start", "100", "--step", "5"]).output().unwrap();
    assert!(String::from_utf8_lossy(&o.stdout).contains("1 PRINT 1\n2 PRINT 2"));
    // Überlauf mit Vorschlag
    let o = vcc().args(["basic", "--to-numbers", f.to_str().unwrap(), "--start", "63999", "--step", "10"]).output().unwrap();
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("B192"));
    let o = vcc().args(["basic", "--renumber", f.to_str().unwrap(), "--step", "0"]).output().unwrap();
    assert!(String::from_utf8_lossy(&o.stderr).contains("B191") || o.status.success());
}
