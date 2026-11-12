// SPDX-License-Identifier: GPL-3.0-or-later
//! Lange Variablennamen im nativen BASIC (`#longnames`): Umsetzung je Dialekt.
use std::path::PathBuf;
use vc_basic::longnames::{self, Target};
use vc_basic::{compile_plain_atari, compile_plain_cbm, Dialect, PlainOptions};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn opts() -> PlainOptions { PlainOptions::new("t.bas", 63999) }
fn cbm(id: &str, src: &str) -> vc_basic::PlainCbm { compile_plain_cbm(src, &Dialect::load(&data(), id).unwrap(), &opts(), 0x0801).unwrap_or_else(|e| panic!("{e:?}")) }
fn short<'a>(c: &'a vc_basic::PlainCbm, long: &str) -> &'a str { &c.names.iter().find(|n| n.long == long).unwrap_or_else(|| panic!("{long} fehlt: {:?}", c.names)).short }

const DIALECTS: [&str; 4] = ["cbm_v2", "basic35", "basic7", "basic65"];

#[test]
fn long_program_gives_the_same_bytes_as_the_hand_shortened_one() {
    let long = "#longnames\n\
        punkte = 0 : hero_x = 10 : zaehler% = 3 : name$ = \"anna\" : dim feld(5)\n\
        for i = 1 to 5 : feld(i) = i * hero_x : next i\n\
        punkte = punkte + feld(3) + zaehler%\n\
        print name$; punkte ; \"punkte\"\n\
        rem punkte bleibt im kommentar\n\
        data punkte, 2\n";
    let hand = "punkte = 0 : hero_x = 10 : zaehler% = 3 : name$ = \"anna\" : dim feld(5)\n";
    let _ = hand;
    for id in DIALECTS {
        let c = cbm(id, long);
        let (pu, he, za, na, fe) = (short(&c, "punkte"), short(&c, "hero_x"), short(&c, "zaehler%"), short(&c, "name$"), short(&c, "feld"));
        let by_hand = format!("{pu} = 0 : {he} = 10 : {za} = 3 : {na} = \"anna\" : dim {fe}(5)\n\
            for i = 1 to 5 : {fe}(i) = i * {he} : next i\n\
            {pu} = {pu} + {fe}(3) + {za}\n\
            print {na}; {pu} ; \"punkte\"\n\
            rem punkte bleibt im kommentar\n\
            data punkte, 2\n");
        let h = cbm(id, &by_hand);
        assert_eq!(c.prg, h.prg, "{id}: Bytes weichen ab");
        assert!(h.names.is_empty());
        // natürliche Namen: erste zwei Zeichen, wie im klassischen BASIC
        assert_eq!((pu, he, za, na, fe), ("PU", "HE", "ZA%", "NA$", "FE"), "{id}");
    }
}

#[test]
fn without_directive_nothing_changes() {
    let src = "print punkte + total\n";
    let c = cbm("cbm_v2", src);
    assert!(c.names.is_empty());
    assert_eq!(c.prg, cbm("cbm_v2", "10 print punkte + total\n").prg);
}

#[test]
fn token_traps_are_avoided() {
    // TOTAL → TO wäre das Schlüsselwort, FORM → FO ist frei, ORANGE → OR, IFFY → IF, GONE → GO, ONLINE → ON, FNORD ist FN + ORD
    let src = "#longnames\ntotal = 1 : score = 2 : form = 3 : orange = 4 : iffy = 5 : gone = 6 : online = 7 : status = 8 : timer = 9\n\
               for form = 1 to total : next form\nprint score\n";
    for id in DIALECTS {
        let c = cbm(id, src);
        let d = Dialect::load(&data(), id).unwrap();
        for n in &c.names {
            let s = n.short.trim_end_matches(['$', '%']);
            assert!(s.len() <= 2, "{id}: {} → {}", n.long, n.short);
            assert!(d.token_bytes(s).is_none(), "{id}: {} → {} ist ein Schlüsselwort", n.long, n.short);
            assert!(!d.is_reserved_var(s), "{id}: {} → {} ist reserviert", n.long, n.short);
        }
        assert_eq!(short(&c, "form"), "FO");
        assert_eq!(short(&c, "score"), "SC");
        assert_ne!(short(&c, "total"), "TO");
        assert_ne!(short(&c, "orange"), "OR");
        assert_ne!(short(&c, "status"), "ST");
        assert_ne!(short(&c, "timer"), "TI");
        // die Schlüsselwörter der Zeile bleiben Schlüsselwörter
        let listing = c.listing.to_ascii_uppercase();
        assert!(listing.contains("FOR FO = 1 TO "), "{id}: {listing}");
        // alle Kurznamen verschieden
        let mut all: Vec<&str> = c.names.iter().map(|n| n.short.as_str()).collect(); all.sort(); all.dedup();
        assert_eq!(all.len(), c.names.len(), "{id}");
    }
}

#[test]
fn names_that_collide_are_separated_and_warned() {
    let c = cbm("cbm_v2", "#longnames\npunkte = 1 : punkt = 2 : pu = 3 : print punkte + punkt + pu\n");
    let (a, b) = (short(&c, "punkte"), short(&c, "punkt"));
    assert_ne!(a, b);
    assert_ne!(a, "PU", "PU hat der Programmierer selbst geschrieben");
    assert_ne!(b, "PU");
    assert!(c.warnings.iter().any(|w| w.code == "W313" && w.message.contains("'punkte'") && w.message.contains("'punkt'")), "{:?}", c.warnings);
    // PU bleibt PU
    assert!(c.listing.to_ascii_uppercase().contains("PU = 3"), "{}", c.listing);
}

#[test]
fn suffixes_and_arrays_share_the_base_but_stay_separate_variables() {
    let c = cbm("cbm_v2", "#longnames\nwert = 1 : wert% = 2 : wert$ = \"x\" : dim wert(4), wert$(2)\nprint wert(1) + wert\n");
    assert_eq!(short(&c, "wert"), "WE");
    assert_eq!(short(&c, "wert%"), "WE%");
    assert_eq!(short(&c, "wert$"), "WE$");
    let e = c.names.iter().find(|n| n.long == "wert").unwrap();
    assert!(e.scalar && e.array);
    let l = c.listing.to_ascii_uppercase();
    assert!(l.contains("DIM WE(4), WE$(2)"), "{l}");
}

#[test]
fn def_fn_names_have_their_own_namespace() {
    let src = "#longnames\nquad = 5 : def fn quad(wert) = wert * wert\nprint fn quad(quad) + fnquad(2)\n";
    let c = cbm("cbm_v2", src);
    let f = c.names.iter().find(|n| n.kind == "Funktion").unwrap();
    assert_eq!(f.short, "QU");
    assert_eq!(short(&c, "quad"), "QU");   // Variable und Funktion dürfen denselben Kurznamen tragen
    assert_eq!(short(&c, "wert"), "WE");
    let by_hand = cbm("cbm_v2", "qu = 5 : def fn qu(we) = we * we\nprint fn qu(qu) + fnqu(2)\n");
    assert_eq!(c.prg, by_hand.prg);
    // Funktionsname, der ein Schlüsselwort ergäbe
    let c = cbm("cbm_v2", "#longnames\ndef fn total(x) = x\nprint fn total(1)\n");
    let f = c.names.iter().find(|n| n.kind == "Funktion").unwrap();
    assert_ne!(f.short, "TO");
}

#[test]
fn strings_comments_data_numbers_and_labels_stay() {
    let c = cbm("cbm_v2", "#longnames\n@anfang:\nanzahl = 1e3 + $ff + .5 : print \"anzahl {clr}\"\nrem anzahl\ndata anzahl : goto @anfang\n");
    let l = c.listing.to_ascii_uppercase();
    assert!(l.contains("AN = 1E3"), "{l}");
    assert!(l.contains("PRINT \"ANZAHL"), "{l}");
    assert!(l.contains("REM ANZAHL"), "{l}");
    assert!(l.contains("DATA ANZAHL"), "{l}");
    assert!(l.contains("GOTO 10"), "{l}");
}

#[test]
fn many_names_get_unique_short_names() {
    let mut src = String::from("#longnames\n");
    for i in 0..300 { src.push_str(&format!("wert_{i} = {i}\n")); }
    let c = cbm("cbm_v2", &src);
    let mut all: Vec<&str> = c.names.iter().map(|n| n.short.as_str()).collect(); all.sort(); all.dedup();
    assert_eq!(all.len(), 300);
}

#[test]
fn no_free_short_name_is_an_error() {
    let mut src = String::from("#longnames\n");
    for i in 0..1100 { src.push_str(&format!("v_{i} = 1\n")); }
    let e = compile_plain_cbm(&src, &Dialect::load(&data(), "cbm_v2").unwrap(), &opts(), 0x0801).err().unwrap();
    assert_eq!(e[0].code, "B131");
}

#[test]
fn atari_keeps_long_names_and_only_drops_underscores() {
    for id in ["atari_basic", "turbo_basic_xl"] {
        let d = vc_basic::atari::AtariDialect::load(&data(), id).unwrap();
        let o = PlainOptions::new("t.bas", 32767);
        let a = compile_plain_atari("#longnames\npunkte = 1 : hero_x = punkte + 1 : herox = 2 : name$ = \"a_b\" : ? hero_x, herox, name$\n", &d, &o).unwrap();
        let b = compile_plain_atari("punkte = 1 : herox2 = punkte + 1 : herox = 2 : name$ = \"a_b\" : ? herox2, herox, name$\n", &d, &o).unwrap();
        assert_eq!(a.file, b.file, "{id}");
        assert_eq!(a.names.len(), 1);
        assert_eq!((a.names[0].long.as_str(), a.names[0].short.as_str()), ("hero_x", "herox2"));
        // ohne Unterstrich bleibt alles, wie es ist
        let c = compile_plain_atari("#longnames\ntotal = 1 : score = total\n", &d, &o).unwrap();
        assert!(c.names.is_empty());
        assert_eq!(c.file, compile_plain_atari("total = 1 : score = total\n", &d, &o).unwrap().file);
    }
}

#[test]
fn lookup_and_table_text() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let src = "#longnames\nfeld(3) = 1 : punkte = 2 : punkte$ = \"a\"\n";
    let e = longnames::lookup(src, Target::Cbm(&d), "punkte");
    assert_eq!(e.iter().map(|n| n.short.as_str()).collect::<Vec<_>>(), vec!["PU", "PU$"]);
    assert!(longnames::lookup("feld = 1\n", Target::Cbm(&d), "feld").is_empty(), "ohne #longnames keine Umsetzung");
    let t = longnames::table_text(&cbm("cbm_v2", src).names);
    assert!(t.contains("feld(…)") && t.contains("FE(…)") && t.contains("punkte$"), "{t}");
}

#[test]
fn basic2asm_accepts_long_names() {
    let d = Dialect::load(&data(), "cbm_v2").unwrap();
    let o = vc_basic::b2a::B2aOptions::new("t.bas");
    let long = vc_basic::b2a::convert("#longnames\nzaehler = 0\n@l:\nzaehler = zaehler + 1\nif zaehler < 10 then @l\n", &d, &o).unwrap();
    let hand = vc_basic::b2a::convert("za = 0\n@l:\nza = za + 1\nif za < 10 then @l\n", &d, &o).unwrap();
    assert_eq!(long.asm.to_ascii_uppercase(), hand.asm.to_ascii_uppercase());   // Kommentarzeilen zeigen den Quelltext
    assert_eq!(long.names.len(), 1);
    assert!(long.map_json("t.bas").contains("\"long\": \"zaehler\""));
}

/// Beispiele aus der Hilfe und das Beispielprojekt müssen bauen
#[test]
fn help_and_example_programs_build() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // Die Seite `basic-names.md`: jeder ```basic-Block mit `#longnames` (außer Fehler-, Atari- und Hilfsdateien) muss bauen
    let help = std::fs::read_to_string(root.join("crates/vc-ide/help/basic-names.md")).unwrap();
    let mut with_names = 0;
    for b in help.split("```").skip(1).step_by(2) {
        let (info, body) = b.split_once('\n').unwrap_or((b, ""));
        if !info.starts_with("basic") || info.contains("fehler") || info.contains("system=") || info.contains("datei=") || !body.contains("#longnames") { continue; }
        if !cbm("cbm_v2", body).names.is_empty() { with_names += 1; }
    }
    assert!(with_names >= 4, "zu wenige Beispiele mit langen Namen: {with_names}");
    let ex = std::fs::read_to_string(root.join("examples/c64/basic_langenamen/punkte.bas")).unwrap();
    for id in DIALECTS { assert!(cbm(id, &ex).names.len() >= 7, "{id}"); }
}

#[test]
fn different_long_names_never_share_a_short_name() {
    let src = "#longnames\npositionx = 1 : positiony = 2 : positionz% = 3 : position$ = \"a\"\nprint positionx + positiony\n";
    for id in DIALECTS {
        let c = cbm(id, src);
        let (x, y) = (short(&c, "positionx"), short(&c, "positiony"));
        assert_ne!(x, y, "{id}");
        assert_eq!(x, "PO", "{id}");
        // verschiedene Grundnamen → verschiedene Kurzbasen (Typzeichen abgeschnitten)
        let mut bases: Vec<String> = ["positionx", "positiony", "positionz%", "position$"].iter().map(|n| short(&c, n).trim_end_matches(['$', '%']).to_string()).collect();
        bases.sort(); bases.dedup();
        assert_eq!(bases.len(), 4, "{id}: {bases:?}");
    }
}
