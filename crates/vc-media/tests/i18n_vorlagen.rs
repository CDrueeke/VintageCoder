// SPDX-License-Identifier: GPL-3.0-or-later
//! Englisch: Vorlagenliste, Profile und neu angelegte Projekte kommen in englischer Sprache (Etappe I3).
use std::path::PathBuf;
use vc_i18n::Lang;
use vc_media::wizard;

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

fn deutsch(t: &str) -> bool { t.chars().any(|c| "äöüÄÖÜß".contains(c)) }

#[test]
fn vorlagenliste_und_projektdateien_folgen_der_sprache() {
    vc_i18n::set_thread_language(Some(Lang::En));
    let list = wizard::templates(&data()).unwrap();
    let s = list.to_string();
    assert!(!deutsch(&s), "deutsche Reste in der Vorlagenliste: {s}");
    assert!(s.contains("cartridge") || s.contains("Cartridge"), "{s}");
    for t in list["templates"].as_array().unwrap() {
        for sys in t["systems"].as_array().unwrap() {
            let (tid, sid) = (t["id"].as_str().unwrap(), sys.as_str().unwrap());
            let dir = std::env::temp_dir().join(format!("vc-wiz-en-{}-{tid}-{sid}", std::process::id()));
            let r = wizard::create(&data(), &dir, "demo", &[sid.to_string()], tid).unwrap();
            for f in r["files"].as_array().unwrap() {
                let text = std::fs::read_to_string(dir.join(f.as_str().unwrap())).unwrap();
                assert!(!deutsch(&text), "{tid}/{sid}: deutsche Reste in {f}");
            }
            std::fs::remove_dir_all(&dir).ok();
        }
    }
    vc_i18n::set_thread_language(Some(Lang::De));
    let de = wizard::templates(&data()).unwrap().to_string();
    assert!(deutsch(&de) || de.contains("Assembler"), "Deutsch bleibt Deutsch");
}

#[test]
fn profile_haben_englische_namen() {
    vc_i18n::set_thread_language(Some(Lang::En));
    let p = vc_media::load_profiles(&data()).unwrap();
    assert!(!p.is_empty());
    assert!(p.iter().all(|x| !deutsch(&x.name)), "{:?}", p.iter().map(|x| &x.name).collect::<Vec<_>>());
}
