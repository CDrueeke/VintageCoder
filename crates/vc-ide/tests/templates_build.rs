// SPDX-License-Identifier: GPL-3.0-or-later
//! Jede Vorlage des Wizards (und jede ihrer Varianten) muss für jedes ihrer Systeme bauen (Assembler und natives BASIC).
use std::path::PathBuf;
use vc_media::wizard;
use vc_ide::build::{build, Ctx};

#[test]
fn every_template_builds_for_every_system() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let list = wizard::templates(&data).unwrap();
    let mut built = 0;
    for t in list["templates"].as_array().unwrap() {
        // die Vorlage selbst (None) und jede Variante (z. B. Steckmodul bei asm-hello)
        let mut entries: Vec<(Option<String>, &serde_json::Value)> = vec![(None, t)];
        for v in t["variants"].as_array().into_iter().flatten() { entries.push((Some(v["id"].as_str().unwrap().to_string()), v)); }
        for (variant, e) in entries {
            for s in e["systems"].as_array().unwrap() {
                let (tid, sid) = (t["id"].as_str().unwrap(), s.as_str().unwrap());
                let tag = format!("{tid}{}", variant.as_deref().map(|v| format!("-{v}")).unwrap_or_default());
                let dir = std::env::temp_dir().join(format!("vc-tb-{}-{tag}-{sid}", std::process::id()));
                let r = wizard::create_with(&data, &dir, "probe", &[sid.to_string()], tid, variant.as_deref()).unwrap();
                for f in r["files"].as_array().unwrap().iter().filter_map(|f| f.as_str()).filter(|f| f.ends_with(".asm") || f.ends_with(".bas")) {
                    let out = build(&Ctx { root: &dir, data: &data }, f, Some(sid)).unwrap_or_else(|e| panic!("{tag}/{sid}/{f}: {e}"));
                    assert_eq!(out["ok"], true, "{tag}/{sid}/{f}: {out}");
                    built += 1;
                }
                std::fs::remove_dir_all(&dir).ok();
            }
        }
    }
    assert!(built >= 12, "nur {built} Quellen gebaut");
}

/// Das Steckmodul ist nur eine Variante von `asm-hello`: gleiche Projektdatei (Quellart `asm`), nur der Startquelltext unterscheidet sich.
/// `cartridge` bleibt als Kurzform erhalten.
#[test]
fn cartridge_is_a_variant_of_asm_hello() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let list = wizard::templates(&data).unwrap();
    assert!(!list["templates"].as_array().unwrap().iter().any(|t| t["id"] == "cartridge"), "kein eigener Eintrag mehr");
    let sid = "c64".to_string();
    let (d1, d2) = (std::env::temp_dir().join(format!("vc-cv1-{}", std::process::id())), std::env::temp_dir().join(format!("vc-cv2-{}", std::process::id())));
    let a = wizard::create(&data, &d1, "probe", &[sid.clone()], "cartridge").unwrap();
    let b = wizard::create_with(&data, &d2, "probe", &[sid.clone()], "asm-hello", Some("cartridge")).unwrap();
    assert_eq!(a["files"], b["files"]);
    assert!(std::fs::read_to_string(d1.join("src/c64/module.asm")).unwrap().contains(".cartridge"));
    let (p1, p2): (serde_json::Value, serde_json::Value) = (serde_json::from_str(&std::fs::read_to_string(d1.join("probe.vcprj")).unwrap()).unwrap(), serde_json::from_str(&std::fs::read_to_string(d2.join("probe.vcprj")).unwrap()).unwrap());
    assert_eq!(p1["sources"][0]["kind"], "asm"); assert_eq!(p1["sources"], p2["sources"]);
    // nicht für jedes System, unbekannte Variante
    let d3 = std::env::temp_dir().join(format!("vc-cv3-{}", std::process::id()));
    assert!(wizard::create_with(&data, &d3, "probe", &["mega65".to_string()], "asm-hello", Some("cartridge")).unwrap_err().contains("gibt es nicht für 'mega65'"));
    assert!(wizard::create_with(&data, &d3, "probe", &[sid], "asm-hello", Some("gibtsnicht")).unwrap_err().contains("keine Variante"));
    for d in [d1, d2, d3] { std::fs::remove_dir_all(&d).ok(); }
}
