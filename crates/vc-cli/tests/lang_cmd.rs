// SPDX-License-Identifier: GPL-3.0-or-later
//! `vcc --lang` und `VC_LANG` (i18n I2): Reihenfolge `--lang` > `VC_LANG` > System, zweisprachige Fehlermeldung bei ungültigem Wert,
//! Meldungen in der gewählten Sprache und `--json` mit `code` und `args` (unabhängig vom Wortlaut).
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `vc_lang`: Wert für `VC_LANG` (None: Variable entfernen)
fn vcc_with(args: &[&str], vc_lang: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_vcc"));
    c.args(args).env("VC_DATA", Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"));
    match vc_lang { Some(l) => { c.env("VC_LANG", l); } None => { c.env_remove("VC_LANG"); } }
    c.output().unwrap()
}
fn vcc(args: &[&str]) -> Output { vcc_with(args, None) }
fn err(o: &Output) -> String { String::from_utf8_lossy(&o.stderr).into_owned() }

fn tmp(name: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("vcc-lang-{}-{name}", std::process::id())); let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d }

#[test]
fn lang_option_is_accepted_anywhere_and_invalid_values_fail_in_both_languages() {
    for a in [&["--lang", "en", "--version"][..], &["--version", "--lang", "de"], &["--lang=en", "version"]] {
        let o = vcc(a); assert!(o.status.success(), "{a:?}: {}", err(&o));
        assert!(String::from_utf8_lossy(&o.stdout).starts_with("vcc "));
    }
    // noch keine Sprache gewählt: die Meldung steht auf Deutsch und Englisch
    let o = vcc(&["--lang", "fr", "--version"]);
    assert_eq!(o.status.code(), Some(2));
    let e = err(&o); assert!(e.contains("ist keine unterstützte Sprache") && e.contains("is not a supported language") && e.contains("'fr'"), "{e}");
    let o = vcc(&["--lang"]);
    assert_eq!(o.status.code(), Some(2));
    let e = err(&o); assert!(e.contains("erwartet") && e.contains("expects"), "{e}");
}

#[test]
fn vc_lang_with_unknown_value_is_ignored() {
    for v in ["xx", "en", "de", "kauderwelsch"] {
        let o = vcc_with(&["--version"], Some(v));
        assert!(o.status.success(), "VC_LANG={v}");
    }
}

/// Fehlermeldung (unbekannte Option) in der Sprache: `--lang` vor `VC_LANG`, `VC_LANG` vor der Systemsprache
#[test]
fn messages_follow_lang_option_then_vc_lang() {
    for (args, vc_lang, english) in [
        (&["--lang", "en", "asm", "--foo"][..], Some("de"), true), (&["--lang=de", "asm", "--foo"], Some("en"), false),
        (&["asm", "--foo"], Some("en"), true), (&["asm", "--foo"], Some("de"), false),
    ] {
        let o = vcc_with(args, vc_lang);
        assert_eq!(o.status.code(), Some(2), "{args:?} VC_LANG={vc_lang:?}");
        let e = err(&o);
        if english { assert!(e.contains("unknown option") && e.contains("Help:") && !e.contains("unbekannte"), "{args:?} VC_LANG={vc_lang:?}: {e}"); }
        else { assert!(e.contains("unbekannte Option") && e.contains("Hilfe:") && !e.contains("unknown option"), "{args:?} VC_LANG={vc_lang:?}: {e}"); }
    }
    // Hilfe: Rahmentexte folgen der Sprache (der Text der Befehle aus data/cli bleibt bis I3 deutsch)
    let o = vcc(&["--lang", "en", "--help"]); let t = String::from_utf8_lossy(&o.stdout).into_owned() + &err(&o);
    assert!(t.contains("Usage:") && t.contains("General") && t.contains("Examples"), "{t}");
    let o = vcc(&["--lang", "de", "--help"]); let t = String::from_utf8_lossy(&o.stdout).into_owned() + &err(&o);
    assert!(t.contains("Aufruf:") && t.contains("Allgemein") && t.contains("Beispiele"), "{t}");
}

/// Ein Fehler mit `--lang en` auf der Fehlerausgabe: Text englisch, Dateiname und Option unverändert
#[test]
fn error_with_lang_en_is_english() {
    let d = tmp("err");
    let f = d.join("x.asm"); std::fs::write(&f, "lda #300\n").unwrap();
    let o = vcc(&["--lang", "en", "asm", f.to_str().unwrap()]);
    assert!(!o.status.success());
    let e = err(&o);
    assert!(e.contains("E203") && !e.contains("außerhalb") && !e.contains("Wert 300"), "{e}");
    let o = vcc(&["--lang", "de", "asm", f.to_str().unwrap()]);
    assert!(err(&o).contains("E203"), "Code bleibt in beiden Sprachen");
    let _ = std::fs::remove_dir_all(&d);
}

/// `--json`: Fehler ohne Diagnostic und Diagnosen erscheinen als JSON mit `code`, `key` und `args`; nur `message` ändert sich mit der Sprache
#[test]
fn json_output_has_code_and_args_in_both_languages() {
    let line = |o: &Output| -> serde_json::Value { serde_json::from_str(err(o).lines().next().unwrap()).unwrap() };
    let en = line(&vcc(&["--lang", "en", "build", "--json"]));
    let de = line(&vcc(&["--lang", "de", "build", "--json"]));
    assert_eq!((en["code"].as_str(), en["key"].as_str(), en["severity"].as_str()), (Some("cli-json-needs-license-report"), Some("cli-json-needs-license-report"), Some("error")), "{en}");
    assert_eq!((en["code"].clone(), en["args"].clone()), (de["code"].clone(), de["args"].clone()));
    assert!(en["args"].is_object());
    assert_ne!(en["message"], de["message"]);
    assert!(en["message"].as_str().unwrap().contains("only works"), "{en}");
    assert!(de["message"].as_str().unwrap().contains("gilt nur"), "{de}");
    // Fehler mit Parametern
    let d = tmp("json");
    let o = vcc(&["--lang", "en", "build", d.to_str().unwrap(), "--license-report", "--json"]);
    let j = line(&o);
    assert_eq!(j["code"].as_str(), Some("cli-no-project-file"), "{j}");
    assert!(j["args"]["path"].as_str().unwrap().contains("vcc-lang-") && j["message"].as_str().unwrap().contains("no .vcprj found"), "{j}");
    // Diagnose aus dem Übersetzer: Code des Assemblers/BASIC, Schlüssel und Parameter
    let f = d.join("bad.bas"); std::fs::write(&f, "10 PRINT \"A\"\n20 GOTO\n").unwrap();
    let o = vcc(&["--lang", "en", "basic", "ast", "--json", f.to_str().unwrap()]);
    assert!(!o.status.success());
    let j = line(&o);
    assert_eq!(j["code"].as_str(), Some("E103"), "{j}");
    assert!(j["key"].as_str().unwrap().starts_with("diag-basic-") && j["args"].is_object() && j["range"]["line"].is_number(), "{j}");
    assert!(j["message"].as_str().unwrap().contains("line number expected"), "{j}");
    let jd = line(&vcc(&["--lang", "de", "basic", "ast", "--json", f.to_str().unwrap()]));
    assert_eq!((jd["code"].clone(), jd["key"].clone(), jd["args"].clone()), (j["code"].clone(), j["key"].clone(), j["args"].clone()));
    let _ = std::fs::remove_dir_all(&d);
}
