// SPDX-License-Identifier: GPL-3.0-or-later
//! CBM-II (Ziel `cbm2`) wurde entfernt: alte Projekte und der Aufruf mit `--target cbm2` geben eine klare Meldung aus, nie einen Absturz.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn vcc(dir: &Path, lang: &str, args: &[&str]) -> Output { Command::new(env!("CARGO_BIN_EXE_vcc")).args(args).env("VC_LANG", lang).env("VC_DATA", data()).current_dir(dir).output().unwrap() }
fn text(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr) }

#[test]
fn altes_cbm2_projekt_und_target_cbm2_melden_klar() {
    let d = std::env::temp_dir().join(format!("vc-cbm2-alt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("hello.bas"), "10 print \"hallo\"\n").unwrap();
    std::fs::write(d.join("alt.vcprj"), r#"{"format":1,"project":{"name":"Alt"},"targets":[{"id":"cbm2","system":"cbm2","variant":"pal","cpu":"nmos6502","illegals":"deny","emulator":"vice"}],"sources":[{"targets":["cbm2"],"kind":"basic","files":["hello.bas"]}],"main":"hello.bas"}"#).unwrap();
    for (lang, wort) in [("de", "gibt es nicht mehr"), ("en", "no longer exists")] {
        for args in [vec!["build", d.to_str().unwrap()], vec!["basic", d.join("hello.bas").to_str().unwrap(), "--target", "cbm2", "-o", d.join("o.prg").to_str().unwrap()], vec!["asm", "x.asm", "--target", "cbm2"]] {
            let o = vcc(&d, lang, &args);
            let t = text(&o);
            assert!(!o.status.success(), "{lang} {args:?}: {t}");
            assert!(t.contains("cbm2") && t.contains(wort), "{lang} {args:?}: {t}");
            assert!(!t.contains("panicked"), "{lang} {args:?}: {t}");
        }
    }
    assert!(!d.join("o.prg").exists());
    let _ = std::fs::remove_dir_all(&d);
}
