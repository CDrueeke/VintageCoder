// SPDX-License-Identifier: GPL-3.0-or-later
//! Sprachdienste für natives BASIC: Dialekt, #define, ausgeschlossene Zeilen, Gliederung, Hover.
use std::path::PathBuf;
use vc_ide::basicinfo::{code_text, code_texts, hover, info, Req};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn tmp(tag: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vc-bi-{tag}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

const SRC: &str = "#define STERNE 5\n#if SYSTEM_C128\nprint \"c128\"\n#else\nprint \"c64\"\n#endif\n@start: for i=1 to STERNE\nnext i\n#asm \"code.asm\"\n";

#[test]
fn info_nennt_dialekt_defines_ausgeschlossene_zeilen_und_marken() {
    let (d, dt) = (tmp("info"), data());
    let v = info(&Req { root: &d, data: &dt, rel: "a.bas", text: SRC });
    assert_eq!(v["dialect"]["id"], "cbm_v2", "{v}");
    assert_eq!(v["system"], "c64");
    assert_eq!(v["defines"][0]["name"], "STERNE");
    assert_eq!(v["defines"][0]["value"], "5");
    assert_eq!(v["defines"][0]["line"], 0);
    // C64: der C128-Zweig (Zeile 3, ab 0: 2) ist ausgeschlossen
    let ex: Vec<u64> = v["excluded"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect();
    assert_eq!(ex, vec![2], "nur die ausgeschlossene Programmzeile, nicht die Direktiven: {v}");
    assert!(!ex.contains(&4), "{v}");
    let names: Vec<&str> = v["outline"].as_array().unwrap().iter().map(|o| o["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"@start") && names.contains(&"STERNE"), "{names:?}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn hover_erklaert_define_und_direktive_aber_nicht_in_zeichenketten() {
    let (d, dt) = (tmp("hover"), data());
    let r = Req { root: &d, data: &dt, rel: "a.bas", text: SRC };
    // Verwendung des Namens in Zeile 7 („for i=1 to STERNE“)
    let col = SRC.lines().nth(6).unwrap().find("STERNE").unwrap();
    let (md, a, z) = hover(&r, 6, col + 2).expect("Karte für STERNE");
    assert!(md.contains("STERNE") && md.contains("`5`"), "{md}");
    assert_eq!((a, z), (col, col + 6));
    // Direktive
    let (md, ..) = hover(&r, 1, 2).expect("Karte für #if");
    assert!(md.contains("#if"), "{md}");
    // nichts in einer Zeichenkette und für unbekannte Wörter
    let q = Req { root: &d, data: &dt, rel: "a.bas", text: "#define X 1\nprint \"X\"\nprint Y\n" };
    assert!(hover(&q, 1, 7).is_none());
    assert!(hover(&q, 2, 6).is_none());
    // der alte Name #blob erklärt seinen Ersatz
    let b = Req { root: &d, data: &dt, rel: "a.bas", text: "#blob \"x.asm\"\n" };
    assert!(hover(&b, 0, 2).unwrap().0.contains("#asm"));
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn jeder_basic_code_hat_einen_erklaertext() {
    let m = code_texts();
    for c in vc_basic::codes::CODES {
        let e = &m[c.code];
        assert!(e["text"].as_str().map_or(false, |t| t.len() > 20), "Erklärtext zu {} fehlt", c.code);
    }
    assert_eq!(m["B152"]["kind"], "warning");
    assert_eq!(m["B152"]["disable"], true);
    assert_eq!(m["B162"]["disable"], false);
    assert!(code_text("E100").is_none());
}
