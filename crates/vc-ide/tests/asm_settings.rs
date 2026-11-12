// SPDX-License-Identifier: GPL-3.0-or-later
//! Assembler-Optionen aus den Einstellungen (Toolchain) wirken beim Bauen in der IDE: Konstante, Include-Ordner, Illegale Opcodes.
use std::path::PathBuf;
use vc_ide::build::{build, Ctx};
use vc_ide::settings::{set, AsmCfg, Settings};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn settings_reach_the_assembler() {
    let dir = std::env::temp_dir().join(format!("vc_asm_set_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (root, extra) = (dir.join("proj"), dir.join("shared"));
    std::fs::create_dir_all(&root).unwrap(); std::fs::create_dir_all(&extra).unwrap();
    std::env::set_var("VC_SETTINGS", dir.join("settings.json"));
    std::env::set_var("VC_DATA", data());
    std::fs::write(extra.join("lib.inc"), ".const LIVES = 3\n").unwrap();
    std::fs::write(root.join("main.asm"), ".target c64\n.org $0801\n.basicstub start\n.include \"lib.inc\"\nstart:\n.if DEBUG\n lda #LIVES\n.endif\n jmp *\n").unwrap();
    let run = || build(&Ctx { root: &root, data: &data() }, "main.asm", None);

    // ohne Einstellung: lib.inc ist nicht zu finden
    assert_eq!(run().unwrap()["ok"], false);
    // mit Include-Ordner und Konstante aus Auswahl und freiem Feld
    let mut s = Settings::default();
    s.assembler = AsmCfg { opts: [("include".to_string(), extra.to_string_lossy().into_owned())].into(), args: "-D DEBUG=1".into() };
    set(s.clone()).unwrap();
    let v = run().unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let log = v["log"].to_string();
    assert!(log.contains("Einstellung: Konstante: DEBUG=1") && log.contains("Include-Ordner"), "{log}");
    // illegale Opcodes verbieten: ein SLO-Befehl scheitert
    std::fs::write(root.join("main.asm"), ".target c64\n.org $0801\n.basicstub start\nstart: slo $10\n jmp *\n").unwrap();
    s.assembler = AsmCfg::default(); set(s.clone()).unwrap();
    assert_eq!(run().unwrap()["ok"], true, "Standard: warnen");
    s.assembler.opts.insert("illegals".into(), "deny".into()); set(s.clone()).unwrap();
    assert_eq!(run().unwrap()["ok"], false, "deny: Fehler");
    // ungültige Einstellung: Fehler mit Erklärung statt stillem Ignorieren
    s.assembler = AsmCfg { opts: Default::default(), args: "--bogus".into() }; set(s).unwrap();
    let e = run().unwrap_err(); assert!(e.contains("Assembler-Optionen"), "{e}");
    std::fs::remove_dir_all(&dir).ok();
}
