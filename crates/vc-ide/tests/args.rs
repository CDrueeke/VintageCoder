// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_ide::args::*;

fn data_args() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/args") }
fn cat(kind: &str) -> Catalog {
    let (all, errs) = load_dirs(&[data_args()]);
    assert!(errs.is_empty(), "{errs:?}");
    all.into_iter().find(|c| c.kinds.iter().any(|k| k == kind)).unwrap_or_else(|| panic!("kein Katalog für {kind}"))
}
fn sel(v: &[(&str, &str)]) -> Selection { v.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect() }
fn ctx() -> ArgCtx { ArgCtx::example("c64") }

#[test]
fn shipped_catalogs_load_and_are_consistent() {
    let (all, errs) = load_dirs(&[data_args()]);
    assert!(errs.is_empty(), "{errs:?}");
    for kind in ["x64sc", "x128", "xplus4", "xvic", "atari800", "altirra", "xemu"] { assert!(all.iter().any(|c| c.kinds.contains(&kind.to_string())), "{kind}"); }
    for c in &all {
        for p in &c.params {
            assert!(c.groups.iter().any(|g| g.id == p.group), "{}: Gruppe {} unbekannt", c.program, p.group);
            for x in p.conflicts.iter().chain(&p.requires) { assert!(c.param(x).is_some(), "{}: {} verweist auf unbekannt {x}", c.program, p.id); }
            assert!(!p.label.is_empty() && !p.hint.is_empty(), "{}: {} ohne Beschriftung/Hover", c.program, p.id);
            if let Some(d) = &p.default { let _ = d; }
            for ph in p.example.iter().flat_map(|e| placeholders_in(e)) { assert!(PLACEHOLDERS.iter().any(|x| x.name == ph), "{}: Beispiel mit {ph}", c.program); }
        }
        // jeder Flag kommt nur einmal als Einschalt-Schalter vor
        let mut flags: Vec<&str> = c.params.iter().map(|p| p.flag.as_str()).collect(); flags.sort(); let n = flags.len(); flags.dedup();
        assert_eq!(n, flags.len(), "{}: doppelter Schalter", c.program);
    }
}

#[test]
fn selection_becomes_args_in_catalog_order() {
    let c = cat("x64sc");
    let s = sel(&[("filter", "1"), ("warp", "on"), ("sound", "off"), ("speed", "")]);
    let a = build(Some(&c), &s, "-VICIIvsync", &ctx()).unwrap();
    // Reihenfolge des Katalogs, danach das freie Feld; leere Werte entfallen
    let pos = |x: &str| a.iter().position(|t| t == x).unwrap_or_else(|| panic!("{x} fehlt in {a:?}"));
    assert!(pos("-warp") < pos("-VICIIfilter") || pos("-VICIIfilter") < pos("-warp"));
    assert_eq!(a[pos("-VICIIfilter") + 1], "1");
    assert!(a.contains(&"+sound".to_string()));
    assert!(!a.contains(&"-speed".to_string()));
    assert_eq!(a.last().unwrap(), "-VICIIvsync");
}

#[test]
fn free_field_alone_matches_old_behaviour() {
    let a = build(None, &Selection::new(), "-VICIIfilter 0 \"a b\"", &ctx()).unwrap();
    assert_eq!(a, vec!["-VICIIfilter", "0", "a b"]);
}

#[test]
fn placeholders_are_replaced_and_unknown_ones_fail() {
    let c = cat("x64sc");
    let s = sel(&[("moncommands", "{symbols}"), ("initbreak", "0x{start}")]);
    let a = build(Some(&c), &s, "-8 {disk} -keybuf \"{{x}}\"", &ctx()).unwrap();
    // plattformneutral: gleicher Aufbau wie ArgCtx::example (unter Windows mischt join die Trennzeichen)
    let demo = |f: &str| PathBuf::from("/Users/ich/Projekte/demo").join("build").join(f).to_string_lossy().into_owned();
    assert!(a.contains(&demo("demo.vs")));
    assert!(a.contains(&"0x080D".to_string()));
    assert!(a.contains(&demo("demo.d64")));
    assert!(a.contains(&"{x}".to_string()), "{{{{ ergibt eine Klammer");
    let e = build(Some(&c), &sel(&[("moncommands", "{nope}")]), "{auch} {project}", &ctx()).unwrap_err();
    assert_eq!(e.len(), 2, "alle Fehler, nicht nur der erste: {e:?}");
    assert!(e[0].contains("{nope}") && e[1].contains("{auch}"));
    // nicht verfügbare Werte
    let none = ArgCtx { project: "p".into(), ..Default::default() };
    assert!(expand("{start}", &none).unwrap_err()[0].contains("SYS"));
    assert!(expand("{symbols}", &none).is_err() && expand("{disk}", &none).is_err());
    assert!(expand("{offen", &ctx()).is_err());
    assert_eq!(expand("{project}/{target}-{variant}", &ctx()).unwrap(), "demo/c64-pal");
    assert_eq!(placeholders_in("a {x} {{y}} {z}"), vec!["x", "z"]);
}

#[test]
fn every_placeholder_resolves_in_the_example() {
    for t in ["c64", "c128", "plus4", "vic20", "mega65"] { for p in PLACEHOLDERS { assert!(expand(&format!("{{{}}}", p.name), &ArgCtx::example(t)).is_ok(), "{t} {}", p.name); } }
    // Atari-Programme haben keinen SYS-Starter
    assert!(expand("{start}", &ArgCtx::example("atari8")).is_err());
}

#[test]
fn check_finds_conflicts_duplicates_and_bad_values() {
    let c = cat("x64sc");
    let texts = |i: Vec<Issue>| i.into_iter().map(|x| x.text).collect::<Vec<_>>().join(" | ");
    // widersprüchlich: Katalog-Konflikt, Gegenschalter, Auswahl gegen Freitext
    let t = texts(check(Some(&c), &sel(&[("warp", "on"), ("speed", "50")]), "", None)); assert!(t.contains("widersprechen"), "{t}");
    let t = texts(check(Some(&c), &Selection::new(), "-warp +warp", None)); assert!(t.contains("widersprechen"), "{t}");
    let t = texts(check(Some(&c), &sel(&[("warp", "on")]), "+warp", None)); assert!(t.contains("+warp"), "{t}");
    // doppelt
    let t = texts(check(Some(&c), &Selection::new(), "-VICIIfilter 1 -VICIIfilter 2", None)); assert!(t.contains("mehrfach"), "{t}");
    let t = texts(check(Some(&c), &sel(&[("filter", "1")]), "-VICIIfilter 2", None)); assert!(t.contains("zusätzlich"), "{t}");
    // unbekannt: nur ein Hinweis
    let i = check(Some(&c), &Selection::new(), "-gibtsnicht", None);
    assert_eq!(i.len(), 1); assert_eq!(i[0].level, Level::Info);
    // der Wert eines Schalters wird nicht als Schalter gelesen (-speed -30)
    assert!(check(Some(&c), &Selection::new(), "-speed -30", None).is_empty());
    // vom Programm selbst gesetzt
    let t = texts(check(Some(&c), &Selection::new(), "-autostart x.prg", None)); assert!(t.contains("selbst"), "{t}");
    // ungültige Werte
    let t = texts(check(Some(&c), &sel(&[("filter", "9"), ("speed", "abc"), ("soundrate", "5")]), "", None));
    assert!(t.contains("keine bekannte Auswahl") && t.contains("keine Zahl") && t.contains("außerhalb"), "{t}");
    // fehlende Voraussetzung, fehlender Wert, unbekannte Kennung, unbekannter Platzhalter
    let t = texts(check(Some(&c), &sel(&[("reusize", "512"), ("alt", "on")]), "-speed", None)); assert!(t.contains("wirkt nur zusammen") && t.contains("verlangt einen Wert") && t.contains("nicht (mehr)"), "{t}");
    let t = texts(check(Some(&c), &Selection::new(), "-moncommands {x}", None)); assert!(t.contains("unbekannter Platzhalter"), "{t}");
    // mit Kontext: nicht verfügbare Werte werden gemeldet; Fehler stehen vor Warnungen
    let none = ArgCtx::default();
    let i = check(Some(&c), &Selection::new(), "-initbreak {start}", Some(&none)); assert_eq!(i[0].level, Level::Error);
    // eine saubere Auswahl hat keine Meldung
    assert!(check(Some(&c), &sel(&[("filter", "1"), ("warp", "on")]), "-VICIIvsync", Some(&ctx())).is_empty());
}

#[test]
fn altirra_slash_flags_and_paths() {
    let c = cat("altirra");
    assert!(check(Some(&c), &sel(&[("fullscreen", "on")]), "", None).is_empty());
    let t: Vec<_> = check(Some(&c), &Selection::new(), "/f /f", None).into_iter().map(|i| i.text).collect(); assert!(t[0].contains("mehrfach"));
    // ein Pfad ist kein Schalter
    assert!(check(Some(&c), &Selection::new(), "/disk x /Users/a/b.atr", None).iter().all(|i| !i.text.contains("/Users")));
}

#[test]
fn migration_moves_known_flags_and_keeps_the_rest() {
    let c = cat("x64sc");
    let (s, rest) = migrate_free(&c, &Selection::new(), "-VICIIfilter 1 -warp -frei \"a b\" +sound -speed 200 -VICIIvsync");
    assert_eq!(s.get("filter").map(String::as_str), Some("1"));
    assert_eq!(s.get("sound").map(String::as_str), Some("off"));
    assert_eq!(s.get("vsync").map(String::as_str), Some("on"));
    // -warp und -speed widersprechen sich: beide bleiben unverändert im Freitext, ebenso Unbekanntes
    assert!(!s.contains_key("warp") && !s.contains_key("speed"));
    assert_eq!(split_args_of(&rest), vec!["-warp", "-frei", "a b", "-speed", "200"]);
    // zweimal vorkommender Schalter bleibt, ebenso ein Wert außerhalb der Liste
    let (s, rest) = migrate_free(&c, &Selection::new(), "-VICIIfilter 1 -VICIIfilter 2 -jamaction 9");
    assert!(s.is_empty() && rest == "-VICIIfilter 1 -VICIIfilter 2 -jamaction 9", "{rest}");
    // bereits gesetzte Auswahl wird nicht überschrieben
    let (s, rest) = migrate_free(&c, &sel(&[("filter", "2")]), "-VICIIfilter 1");
    assert_eq!(s["filter"], "2"); assert_eq!(rest, "-VICIIfilter 1");
    // Ergebnis ist gleichwertig: gleiche Argumente, nur anders eingeteilt
    let before = "-VICIIfilter 1 +sound -frei";
    let (s, rest) = migrate_free(&c, &Selection::new(), before);
    let mut a = build(Some(&c), &s, &rest, &ctx()).unwrap(); a.sort();
    let mut b = build(None, &Selection::new(), before, &ctx()).unwrap(); b.sort();
    assert_eq!(a, b);
    assert_eq!(migratable(&c, &Selection::new(), before), 2);
}
fn split_args_of(s: &str) -> Vec<String> { vc_ide::settings::split_args(s) }

#[test]
fn reset_clears_everything_and_counts() {
    let c = cat("atari800");
    let mut s = sel(&[("turbo", "on"), ("volume", "50"), ("unbekannt", "on")]);
    assert_eq!(changed_count(&c, &s), 2);
    let mut free = "-foo".to_string();
    reset(&mut s, &mut free);
    assert!(s.is_empty() && free.is_empty());
    assert!(build(Some(&c), &s, &free, &ctx()).unwrap().is_empty());
}

#[test]
fn plain_build_drops_placeholder_parameters() {
    let c = cat("x64sc");
    let a = build_plain(Some(&c), &sel(&[("moncommands", "{symbols}"), ("filter", "1")]), "-8 {disk} -verbose");
    assert_eq!(a, vec!["-VICIIfilter", "1", "-verbose"]);
}

#[test]
fn shell_line_quotes_only_when_needed() {
    assert_eq!(shell_line("x64sc", &["-warp".into(), "a b".into(), "".into()]), "x64sc -warp \"a b\" \"\"");
}

#[test]
fn catalogs_are_extensible_from_a_second_directory() {
    let dir = std::env::temp_dir().join(format!("vc_args_ext_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("meins.json"), r#"{"program":"eigen","kinds":["eigen"],"groups":[{"id":"a","label":"A"}],"params":[{"id":"x","group":"a","type":"switch","flag":"--x","label":"X","hint":"h"}]}"#).unwrap();
    std::fs::write(dir.join("kaputt.json"), "{").unwrap();
    let (all, errs) = load_dirs(&[data_args(), dir.clone()]);
    assert!(all.iter().any(|c| c.program == "eigen") && all.iter().any(|c| c.program == "atari800"));
    assert_eq!(errs.len(), 1, "{errs:?}");
    // merge: gleiche Kennung ersetzt, neue wird angehängt
    let mut base = cat("atari800");
    let n = base.params.len();
    let mut extra = base.clone(); extra.params = vec![Param { id: "turbo".into(), flag: "-turbo".into(), label: "Neu".into(), hint: "h".into(), group: "lauf".into(), ..Default::default() }, Param { id: "neu".into(), flag: "-neu".into(), label: "N".into(), hint: "h".into(), group: "lauf".into(), ..Default::default() }];
    merge(&mut base, extra);
    assert_eq!(base.params.len(), n + 1); assert_eq!(base.param("turbo").unwrap().label, "Neu");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn bad_catalog_files_are_reported() {
    assert!(parse(r#"{"program":"x","params":[{"id":"a","flag":"-a"},{"id":"a","flag":"-b"}]}"#, "t").unwrap_err().contains("doppelt"));
    assert!(parse(r#"{"program":"x","params":[{"id":"a","flag":"-a","type":"choice"}]}"#, "t").unwrap_err().contains("choices"));
    assert!(parse(r#"{"params":[]}"#, "t").is_err());
}

#[test]
fn emulator_cfg_carries_options_and_resolves_args() {
    use vc_ide::targets::EmulatorCfg;
    let mut e = EmulatorCfg { id: "a".into(), kind: "x64sc".into(), args: "-VICIIvsync".into(), ..Default::default() };
    e.opts = sel(&[("warp", "on")]);
    let r = e.resolved();
    assert_eq!(r.args, vec!["-VICIIvsync"], "args bleibt das freie Feld");
    std::env::set_var("VC_DATA", data_args().parent().unwrap());
    let a = r.launch_args(&ctx()).unwrap();
    assert_eq!(a, vec!["-warp", "-VICIIvsync"]);
    // alte Einstellungsdatei ohne opts lädt weiter; leere opts werden nicht geschrieben
    let old: EmulatorCfg = serde_json::from_str(r#"{"id":"b","kind":"x64sc","args":"-warp"}"#).unwrap();
    assert!(old.opts.is_empty());
    assert!(!serde_json::to_string(&old).unwrap().contains("opts"));
}

#[test]
fn debug_start_drops_what_it_sets_itself() {
    let c = cat("x64sc");
    let a: Vec<String> = ["-warp", "-binarymonitoraddress", "ip4://x:1", "+confirmonexit", "-VICIIfilter", "1", "-autostart", "a.prg", "-binarymonitor"].iter().map(|s| s.to_string()).collect();
    assert_eq!(drop_debug_reserved(Some(&c), a), vec!["-warp", "-VICIIfilter", "1"]);
    // ohne Katalog bleibt alles
    assert_eq!(drop_debug_reserved(None, vec!["-x".into()]), vec!["-x"]);
    // die Einstellungen warnen davor
    let t: Vec<_> = check(Some(&c), &Selection::new(), "-binarymonitor", None).into_iter().map(|i| i.text).collect();
    assert!(t.iter().any(|x| x.contains("beim Debuggen weggelassen")), "{t:?}");
    let a8 = cat("atari800");
    assert_eq!(drop_debug_reserved(Some(&a8), vec!["-monitor".into(), "-nosound".into(), "-run".into(), "x.xex".into()]), vec!["-nosound"]);
    let x = cat("xemu");
    assert_eq!(drop_debug_reserved(Some(&x), vec!["-uartmon".into(), "s".into(), "-fullscreen".into()]), vec!["-fullscreen"]);
}

#[test]
fn assembler_catalog_applies_to_options() {
    use vc_asm::{Options, Policy};
    let c = cat("vcc-asm");
    let cfg = vc_ide::settings::AsmCfg { opts: sel(&[("illegals", "deny"), ("include", "{project_dir}/inc"), ("define", "DEBUG=1")]), args: "-I /x/y -D LEVEL=0x10 -D DEBUG=2".into() };
    let a = asm_args(Some(&c), &cfg, &ctx()).unwrap();
    let mut o = Options::new("a.asm", Policy::Warn);
    let notes = apply_asm(&a, &mut o).unwrap();
    assert_eq!(o.illegals, Policy::Deny);
    assert_eq!(o.include_dirs, vec![PathBuf::from("/Users/ich/Projekte/demo/inc"), PathBuf::from("/x/y")]);
    assert_eq!(o.defines, vec![("LEVEL".to_string(), 16), ("DEBUG".to_string(), 2)], "spätere Angabe ersetzt, Zahlen auch hexadezimal");
    assert_eq!(notes.len(), 6);
    // Fehler: unbekannter Schalter, fehlender Wert, falsche Werte
    for bad in [vec!["--listing", "x"], vec!["-I"], vec!["--illegals", "vielleicht"], vec!["-D", "A=zwei"], vec!["-D", "=1"]] {
        assert!(apply_asm(&bad.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &mut Options::new("a", Policy::Warn)).is_err(), "{bad:?}");
    }
    // `-D NAME` ohne Wert heißt 1
    let mut o = Options::new("a", Policy::Warn); apply_asm(&["-D".into(), "X".into()], &mut o).unwrap(); assert_eq!(o.defines, vec![("X".to_string(), 1)]);
    // Prüfung: im strengen Katalog sind unbekannte Schalter Fehler; {output} gibt es beim Assemblieren nicht
    let i = check(Some(&c), &Selection::new(), "--listing x", None); assert_eq!(i[0].level, Level::Error);
    let none = ArgCtx { project: "p".into(), ..Default::default() };
    assert!(expand("{output}", &none).is_err());
    assert!(check(Some(&c), &sel(&[("illegals", "allow")]), "-I a -D B=1", None).is_empty());
    // Einstellungsdatei: leer wird nicht geschrieben, alte Dateien laden
    let s = vc_ide::settings::Settings::default();
    assert!(!serde_json::to_string(&s).unwrap().contains("assembler"));
    let mut s2 = s.clone(); s2.assembler = cfg.clone();
    let back: vc_ide::settings::Settings = serde_json::from_str(&serde_json::to_string(&s2).unwrap()).unwrap();
    assert_eq!(back.assembler, cfg);
}

#[test]
fn altirra_joined_values_build_check_and_migrate() {
    let c = cat("altirra");
    assert!(c.verified.contains("4.01.200"));
    // Wert im selben Wort (/kernel:xl), sonst nächstes Wort (/cartmapper 1)
    let a = build(Some(&c), &sel(&[("kernel", "xl"), ("memsize", "64K"), ("cartmapper", "1"), ("fullscreen", "on"), ("vsync", "off")]), "/type RUN~", &ctx()).unwrap();
    assert_eq!(a, vec!["/f", "/novsync", "/kernel:xl", "/memsize:64K", "/cartmapper", "1", "/type", "RUN~"].into_iter().map(String::from).collect::<Vec<_>>());
    // Prüfung im Freitext: doppelt, widersprüchlich, mit Auswahl überschneidend, Wert im nächsten Wort wird nicht als Schalter gelesen
    let t = |s: &str, f: &str| check(Some(&c), &sel_of(s), f, None).into_iter().map(|i| i.text).collect::<Vec<_>>().join(" | ");
    assert!(t("", "/kernel:xl /kernel:osb").contains("mehrfach"));
    assert!(t("kernel=xl", "/kernel:osa").contains("zusätzlich"));
    assert!(t("", "/f /w").contains("widerspricht"));
    assert!(check(Some(&c), &Selection::new(), "/type ~RUN~ /cartmapper 1", None).is_empty());
    // von VintageCoder selbst gesetzt
    assert!(t("", "/tempprofile").contains("selbst"));
    // Übernahme aus dem Freitext
    let (s, rest) = migrate_free(&c, &Selection::new(), "/kernel:xl /memsize:99K /type RUN~ /f");
    assert_eq!(s.get("kernel").map(String::as_str), Some("xl"));
    assert_eq!(s.get("fullscreen").map(String::as_str), Some("on"));
    assert!(!s.contains_key("memsize"), "Wert nicht in der Liste bleibt im Freitext");
    assert_eq!(s.get("type").map(String::as_str), Some("RUN~"), "Schalter mit Wort als Wert wird übernommen");
    assert_eq!(split_args_of(&rest), vec!["/memsize:99K"]);
}
fn sel_of(s: &str) -> Selection { s.split(',').filter(|x| !x.is_empty()).filter_map(|p| p.split_once('=')).map(|(a, b)| (a.to_string(), b.to_string())).collect() }

#[test]
fn windows_rules_for_free_field_and_copy_line_work_on_every_system() {
    // Kopierzeile: Backslashes bleiben, nur ein Rest vor dem schließenden Anführungszeichen wird verdoppelt
    let args = vec!["-sdimg".to_string(), r"C:\Program Files\x y\".to_string(), r"C:\ohne\leer.img".to_string(), "a\"b".to_string()];
    assert_eq!(shell_line_os(r"C:\Program Files\xemu\xmega65.exe", &args, true),
        r#""C:\Program Files\xemu\xmega65.exe" -sdimg "C:\Program Files\x y\\" C:\ohne\leer.img "a\"b""#);
    // andere Systeme: Backslash wird verdoppelt
    assert_eq!(shell_line_os("x", &[r"a\b".to_string()], false), r#"x "a\\b""#);
    // freies Feld: Rückschreiben und Wiederlesen ergibt dasselbe
    for (os, w) in [(true, r"C:\Pfad mit Leer\x.img"), (true, r"\\server\share\d.d64"), (false, r"a b\c")] {
        let q = quote_free_os(w, os);
        assert_eq!(vc_ide::settings::split_args_os(&q, os), vec![w.to_string()], "{q}");
    }
}

/// Auf Windows selbst: die Standardfunktionen folgen der Windows-Regel
#[test]
#[cfg(windows)]
fn windows_backslashes_stay_literal_in_free_field_and_copy_line() {
    assert_eq!(vc_ide::settings::split_args(r"-sdimg C:\Projekte\x.img"), vec!["-sdimg", r"C:\Projekte\x.img"]);
    assert_eq!(shell_line(r"C:\Program Files\xemu\xmega65.exe", &[]), r#""C:\Program Files\xemu\xmega65.exe""#);
}
