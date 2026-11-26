// SPDX-License-Identifier: GPL-3.0-or-later
//! Benennung der Direktiven (Konzept VCASM-Direktiven-Benennung): Äquivalenz alter und neuer Namen, Blöcke, neue Formen,
//! Hinweise bei unpassendem Ziel (W205–W207) und Konsistenz zwischen Tabelle und Assembler.
use std::collections::BTreeSet;
use std::path::PathBuf;
use vc_asm::{assemble, directives, to_xex, Options, Output, Policy};
use vc_target::{load_cpu, load_target};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }

/// Ergebnis eines Laufs: Segmente (Start, Bytes) und Codes aller Meldungen
type Run = Result<(Vec<(u16, Vec<u8>)>, Vec<String>), Vec<String>>;

fn run_with(src: &str, target: Option<&str>) -> Run {
    let (cpu_id, mut o) = match target {
        Some(t) => {
            let t = load_target(&data(), t).unwrap();
            let mut o = Options::new("t.asm", Policy::Allow);
            o.atari_text = t.text_encoding == "atari";
            o.basic_start = t.basic_load_addr;
            (t.cpu.clone(), o)
        }
        None => ("6510".to_string(), Options::new("t.asm", Policy::Allow)),
    };
    o.branch_long = false;
    let cpu = load_cpu(&data(), &cpu_id).unwrap();
    match assemble(src, &cpu, &o) {
        Ok(out) => Ok((out.segments.iter().map(|s| (s.start, s.bytes.clone())).collect(), out.warnings.iter().map(|w| w.code.to_string()).collect())),
        Err(e) => Err(e.iter().map(|d| d.code.to_string()).collect()),
    }
}
fn run(src: &str) -> Run { run_with(src, None) }
fn bytes(src: &str) -> Vec<u8> { run(src).unwrap_or_else(|e| panic!("{src:?}: {e:?}")).0.into_iter().flat_map(|s| s.1).collect() }
fn full(src: &str, target: &str) -> Output {
    let t = load_target(&data(), target).unwrap();
    let mut o = Options::new("t.asm", Policy::Allow);
    o.atari_text = t.text_encoding == "atari";
    o.basic_start = t.basic_load_addr;
    assemble(src, &load_cpu(&data(), &t.cpu).unwrap(), &o).unwrap_or_else(|e| panic!("{e:?}"))
}
fn warns(src: &str, target: &str) -> Vec<String> { full(src, target).warnings.iter().map(|w| w.code.to_string()).collect() }

/// alter und neuer Name: gleiche Segmente, gleiche Meldungen
fn same(old: &str, new: &str) {
    let (a, b) = (run(old), run(new));
    assert!(a.is_ok(), "{old:?}: {a:?}");
    assert_eq!(a, b, "{old:?} gegen {new:?}");
}

#[test]
fn every_alias_gives_bit_identical_output() {
    // A1 .relocate / .endrelocate gegen .pseudopc mit allen drei alten Enden
    let body = " jmp $c000\n{D} $c000\nhere: lda #1\n jmp here\n{E}\n nop\n";
    let new = body.replace("{D}", ".relocate").replace("{E}", ".endrelocate");
    for (d, e) in [(".pseudopc", ".endpseudopc"), (".pseudopc", ".realpc"), (".relocate", ".realpc"), (".pseudopc", ".endrelocate")] {
        same(&format!(".org $1000\n{}", body.replace("{D}", d).replace("{E}", e)), &format!(".org $1000\n{new}"));
    }
    same(".org $1000\n.rta $1234, 1\n", ".org $1000\n.retaddr $1234, 1\n");                      // A2
    same(".org $1000\n.lohifill 3, $0400 + 40 * i\n", ".org $1000\n.lohitable 3, $0400 + 40 * i\n");  // A3
    same(".org $1000\n.lohifill 3, $0400 + 40 * k, k\n", ".org $1000\n.lohitable 3, $0400 + 40 * k, k\n");
    same(".org $1000\n.screen \"Hallo\"\n", ".org $1000\n.screencode \"Hallo\"\n");                // A4
    same(".org $1000\n.null \"ab\", \"c\"\n", ".org $1000\n.textnull \"ab\", \"c\"\n");            // A5
    same(".org $1000\n.ptext \"ab\"\n", ".org $1000\n.textlen \"ab\"\n");
    same(".org $1000\n.shift \"ab\"\n", ".org $1000\n.textshift \"ab\"\n");
    same(".org $1000\n.encoding \"petscii\"\n.null \"Ab\"\n.ptext \"Ab\"\n.shift \"Ab\"\n", ".org $1000\n.encoding \"petscii\"\n.textnull \"Ab\"\n.textlen \"Ab\"\n.textshift \"Ab\"\n");
    same(".org $1000\n nop\n.page\n nop\n", ".org $1000\n nop\n.align 256, skip\n nop\n");             // A6
    same(".org $1000\n nop\n.page 64\n nop\n", ".org $1000\n nop\n.align 64, skip\n nop\n");
    same(".org $1000\n.dbyt $1234, 5\n", ".org $1000\n.be16 $1234, 5\n");                           // A8
    same(".org $1000\n.long $12345678\n", ".org $1000\n.dword $12345678\n");                       // A9
    same(".org $1000\n.db 1,2\n.dw 3\n.addr 4\n.equ A = 5\n.byte A\n", ".org $1000\n.byte 1,2\n.word 3\n.word 4\n.const A = 5\n.byte A\n");
    // Groß-/Kleinschreibung bleibt egal
    same(".org $1000\n.RTA $1234\n.Lohifill 2, i\n", ".org $1000\n.RETADDR $1234\n.LohiTable 2, i\n");
}

#[test]
fn old_and_new_names_give_the_same_diagnostics() {
    for (old, new) in [(".rta 70000", ".retaddr 70000"), (".lohifill 2, 70000 + i", ".lohitable 2, 70000 + i"), (".lohifill 2", ".lohitable 2"),
                       (".ptext 5", ".textlen 5"), (".shift \"\"", ".textshift \"\""), (".page 0", ".align 0, skip"), (".dbyt 70000", ".be16 70000"), (".pseudopc", ".relocate")] {
        let (a, b) = (run(&format!(".org $1000\n{old}\n")), run(&format!(".org $1000\n{new}\n")));
        assert!(a.is_err(), "{old}: {a:?}");
        assert_eq!(a, b, "{old} gegen {new}");
    }
}

#[test]
fn labels_with_the_name_of_a_directive_stay_labels() {
    // Marken brauchen den Doppelpunkt: `.textnull:` ist eine lokale Marke, `.textnull "x"` die Direktive
    let src = ".org $1000\nstart: nop\n.textnull: nop\n bne .textnull\n.screencode \"a\"\n.endrelocate_not: nop\n";
    assert_eq!(bytes(src), vec![0xEA, 0xEA, 0xD0, 0xFD, 0x01, 0xEA]);
    assert_eq!(run(".org $1000\n.screen: nop\n beq .screen\n.screen \"a\"\n").unwrap().0[0].1, vec![0xEA, 0xF0, 0xFD, 0x01]);
    assert_eq!(bytes(".org $1000\n.page: nop\n.next_x: nop\n"), vec![0xEA, 0xEA]);
}

#[test]
fn for_blocks_end_with_endfor_or_next() {
    let want = vec![0, 2, 4, 6];
    for end in [".next", ".endfor", ".NEXT", ".EndFor"] {
        assert_eq!(bytes(&format!(".org $1000\n.for i = 0, i < 4, i += 1\n .byte i * 2\n{end}\n")), want, "{end}");
        assert_eq!(bytes(&format!(".org $1000\n.for i in range(4)\n .byte i * 2\n{end}\n")), want, "{end}");
    }
    // verschachtelt, gemischte Paare
    assert_eq!(bytes(".org $1000\n.for a = 0, a < 2, a += 1\n.for b = 0, b < 2, b += 1\n .byte a * 2 + b\n.endfor\n.next\n"), vec![0, 1, 2, 3]);
    // fehlendes Ende und überzähliges Ende
    assert_eq!(run(".org $1000\n.for i = 0, i < 2, i += 1\n nop\n"), Err(vec!["E210".to_string()]));
    assert_eq!(run(".org $1000\n.endfor\n"), Err(vec!["E210".to_string()]));
    assert_eq!(run(".org $1000\n.next\n"), Err(vec!["E210".to_string()]));
}

#[test]
fn relocate_blocks_work_with_org_and_segments() {
    // Zähler läuft auf der neuen Adresse, Bytes landen an der alten Stelle; nach dem Ende wieder die echte Adresse
    let (seg, _) = run(".org $1000\n.relocate $0300\nl: lda $1c00\n jmp l\n.endrelocate\nafter: nop\n.byte <after, >after\n").unwrap();
    assert_eq!(seg, vec![(0x1000, vec![0xAD, 0x00, 0x1C, 0x4C, 0x00, 0x03, 0xEA, 0x06, 0x10])]);
    // .relocate vor .org: wie früher E300
    assert_eq!(run(".relocate $0300\n nop\n.endrelocate\n").unwrap_err()[0], "E300");
    // neuer Abschnitt mit .org beendet die Wirkung; zwei Blöcke hintereinander
    let (seg, _) = run(".org $1000\n.relocate $2000\n nop\n.endrelocate\n.relocate $3000\nx: jmp x\n.endrelocate\n.org $1100\n nop\n").unwrap();
    assert_eq!(seg, vec![(0x1000, vec![0xEA, 0x4C, 0x00, 0x30]), (0x1100, vec![0xEA])]);
}

#[test]
fn align_fills_skips_and_pads_with_a_byte() {
    assert_eq!(run(".org $1001\n.align 4\n nop\n").unwrap().0, vec![(0x1001, vec![0, 0, 0, 0xEA])]);
    // Füllbyte (A6, V7): $ea als NOP-Polster, auch als Ausdruck
    assert_eq!(run(".org $1001\n.align 4, $ea\n rts\n").unwrap().0, vec![(0x1001, vec![0xEA, 0xEA, 0xEA, 0x60])]);
    assert_eq!(run(".org $1001\n.align 4, 1+1\n rts\n").unwrap().0, vec![(0x1001, vec![2, 2, 2, 0x60])]);
    // schon ausgerichtet: nichts
    assert_eq!(run(".org $1000\n.align 4, $ea\n rts\n").unwrap().0, vec![(0x1000, vec![0x60])]);
    // skip: keine Füllbytes, neues Segment
    assert_eq!(run(".org $10f0\n nop\n.align 256, skip\n nop\n").unwrap().0, vec![(0x10f0, vec![0xEA]), (0x1100, vec![0xEA])]);
    assert_eq!(run(".org $1000\n nop\n.align 16, SKIP\n nop\n").unwrap().0, vec![(0x1000, vec![0xEA]), (0x1010, vec![0xEA])]);
    // Fehler: Größe unter 1 bei skip (E203), Füllbyte außerhalb des Bereichs (E203)
    assert_eq!(run(".org $1000\n.align 0, skip\n"), Err(vec!["E203".to_string()]));
    assert_eq!(run(".org $1000\n nop\n.align 4, 300\n"), Err(vec!["E203".to_string()]));
}

#[test]
fn be16_be32_dword_widths() {
    assert_eq!(bytes(".org $1000\n.be16 $1234\n"), vec![0x12, 0x34]);
    assert_eq!(bytes(".org $1000\n.be32 $12345678, 1\n"), vec![0x12, 0x34, 0x56, 0x78, 0, 0, 0, 1]);
    assert_eq!(bytes(".org $1000\n.dword $12345678\n"), vec![0x78, 0x56, 0x34, 0x12]);
    assert_eq!(bytes(".org $1000\n.long 1\n"), vec![1, 0, 0, 0], ".long legt vier Byte ab");
    assert_eq!(run(".org $1000\n.be32 $100000000\n"), Err(vec!["E203".to_string()]));
    assert_eq!(bytes(".org $1000\n.be32 -1\n"), vec![0xFF; 4]);
}

#[test]
fn text_family_matches_old_forms() {
    assert_eq!(bytes(".org $1000\n.textnull \"ab\"\n"), vec![0x61, 0x62, 0]);
    assert_eq!(bytes(".org $1000\n.textlen \"ab\"\n"), vec![2, 0x61, 0x62]);
    assert_eq!(bytes(".org $1000\n.textshift \"ab\"\n"), vec![0x61, 0xE2]);
    assert_eq!(bytes(".org $1000\n.screencode \"a\"\n"), vec![1]);
}

#[test]
fn folding_and_blocks_come_from_the_table() {
    let (opens, closes) = directives::block_words();
    for w in [".relocate", ".pseudopc", ".for", ".if", ".macro", ".cycles", ".scope", ".struct", ".enum", ".repeat"] { assert!(opens.contains(&w), "{w}"); }
    for w in [".endrelocate", ".realpc", ".endpseudopc", ".endfor", ".next", ".endif", ".endmacro", ".endscope", ".endcycles"] { assert!(closes.contains(&w), "{w}"); }
}

// ---- Hinweise bei unpassendem Ziel (B1–B3)

#[test]
fn w205_basicstub_on_atari() {
    let src = ".org $2000\n.basicstub start\nstart: rts\n.run start\n";
    assert_eq!(warns(src, "atari8"), vec!["W205"]);
    // Ausgabe bleibt wie bisher: BASIC-Zeile im XEX
    let out = full(src, "atari8");
    assert_eq!(out.segments[0].bytes.len(), 13, "BASIC-Zeile und rts bleiben");
    assert!(out.warnings[0].line == 2);
    // C64 und andere BASIC-Ziele: keine Meldung
    for t in ["c64", "mega65", "plus4", "c128", "vic20"] { assert_eq!(warns(".basicstub start\nstart: rts\n", t), Vec::<String>::new(), "{t}"); }
}

#[test]
fn w206_run_without_start_vector() {
    let src = ".org $c000\n.run start\nstart: rts\n";
    for t in ["c64", "mega65", "plus4", "c128", "vic20"] {
        let w = full(src, t).warnings;
        assert_eq!(w.iter().map(|d| d.code).collect::<Vec<_>>(), vec!["W206"], "{t}");
        assert_eq!(w[0].line, 2);
    }
    // Ausgabe unverändert: .run wird wie bisher verworfen
    let out = full(src, "c64");
    assert_eq!((out.segments[0].start, out.segments[0].bytes.clone()), (0xc000, vec![0x60]));
    // Atari: .run gültig, ohne Meldung
    assert_eq!(warns(".org $2000\n.run start\nstart: rts\n", "atari8"), Vec::<String>::new());
    // zweimal .run: eine Meldung
    assert_eq!(warns(".org $c000\n.run a\na: .run b\nb: rts\n", "c64"), vec!["W206"]);
    // Module haben einen eigenen Start: keine Meldung
    assert_eq!(warns(".cartridge \"normal\"\n.org $8000\n.run start\nstart: rts\n", "c64"), Vec::<String>::new());
}

#[test]
fn w207_atari_program_without_start() {
    assert_eq!(warns(".org $2000\n nop\n rts\n", "atari8"), vec!["W207"]);
    let w = full(".org $2000\n\n nop\n", "atari8").warnings;
    assert_eq!((w[0].code, w[0].line), ("W207", 3), "Meldung an der ersten Befehlszeile");
    // Ausgabe unverändert: XEX ohne Startvektor
    let out = full(".org $2000\n nop\n", "atari8");
    assert_eq!(to_xex(&out).unwrap(), vec![0xFF, 0xFF, 0x00, 0x20, 0x00, 0x20, 0xEA]);
    // mit .run: keine Meldung; Befehle in Makros und Schleifen zählen mit
    assert_eq!(warns(".org $2000\n.run s\ns: nop\n", "atari8"), Vec::<String>::new());
    assert_eq!(warns(".macro m\n nop\n.endmacro\n.org $2000\n m\n.run $2000\n", "atari8"), Vec::<String>::new());
    assert_eq!(warns(".macro m\n nop\n.endmacro\n.org $2000\n m\n", "atari8"), vec!["W207"]);
}

#[test]
fn no_false_alarms_for_modules_and_data_files() {
    // reine Datendateien (Atari und C64): keine Befehle, keine Meldung
    for t in ["atari8", "c64"] {
        assert_eq!(warns(".org $2000\n.byte 1,2,3\n.word $1234\n.textnull \"x\"\n", t), Vec::<String>::new(), "{t}");
    }
    // Atari-Modul: eigener Start, keine Meldung auch ohne .run
    assert_eq!(warns(".cartridge \"standard\"\n.org $a000\n nop\n", "atari8"), Vec::<String>::new());
    // Module auf dem C64 und Dateien ohne Zielangabe (Options::new): keine Meldung
    assert_eq!(warns(".cartridge \"normal\"\n.org $8000\n nop\n", "c64"), Vec::<String>::new());
    assert_eq!(run(".org $1000\n.run $1000\n nop\n").map(|r| r.1), Ok(vec![]));
    // Bibliotheksdatei nur mit Marken und Konstanten
    assert_eq!(warns("X = 5\n.const Y = 6\n", "atari8"), Vec::<String>::new());
}

// ---- Konsistenz Tabelle ↔ Assembler

fn match_arms() -> BTreeSet<String> {
    let src = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    let a = src.find("fn directive(").unwrap();
    let b = src[a..].find("fn instruction(").unwrap() + a;
    let mut v = BTreeSet::new();
    for l in src[a..b].lines() {
        let t = l.trim_start();
        // Zeilen vom Typ `".a" | ".b" => …` auf der Ebene des match (12 Leerzeichen)
        if l.starts_with("            \"") && t.contains("=>") {
            let head = t.split("=>").next().unwrap();
            for part in head.split('|') { let p = part.trim().trim_matches('"'); if p.starts_with('.') { v.insert(p.to_string()); } }
        }
    }
    v
}

#[test]
fn table_and_assembler_know_the_same_names() {
    // an anderer Stelle behandelt: Blöcke und Namensräume im Baum (`tree.rs`), Einbinden beim Laden, Makros
    let elsewhere: BTreeSet<String> = [".macro", ".endmacro", ".if", ".ifdef", ".ifndef", ".elseif", ".else", ".endif", ".repeat", ".endrepeat", ".for", ".endfor",
                                       ".struct", ".enum", ".endstruct", ".endenum", ".scope", ".endscope", ".namespace", ".include"].iter().map(|s| s.to_string()).collect();
    let arms = match_arms();
    assert!(arms.len() > 30, "{arms:?}");
    let mut known: BTreeSet<String> = arms.union(&elsewhere).cloned().collect();
    let table: BTreeSet<String> = directives::canonical_names().into_iter().map(String::from).collect();
    // im match darf nur Kanonisches stehen (keine Alias), und alles Kanonische steht dort oder an der genannten Stelle
    for a in &arms { assert_eq!(directives::canonical(a), Some(a.as_str()), "{a} steht im match, ist aber kein kanonischer Name"); }
    known.retain(|n| !n.is_empty());
    assert_eq!(known, table, "nur im Assembler: {:?}, nur in der Tabelle: {:?}", known.difference(&table).collect::<Vec<_>>(), table.difference(&known).collect::<Vec<_>>());
}

#[test]
fn table_is_well_formed() {
    let mut seen = BTreeSet::new();
    for d in directives::table() {
        assert!(d.name.starts_with('.') && d.name == d.name.to_lowercase() && !d.name.contains(' '), "{}", d.name);
        assert!(d.end.is_some() || d.end_aliases.is_empty(), "{}", d.name);
        assert_eq!(d.summary_id, format!("lsp-directive-{}", &d.name[1..]), "{}", d.name);
        assert!(!d.group.is_empty() && !d.since.is_empty());
        for n in std::iter::once(&d.name).chain(&d.aliases) { assert!(seen.insert(n.clone()), "{n} doppelt"); }
    }
    // jede Alias-Form führt zum kanonischen Namen und wird von normalize() aufgelöst
    for (alias, canon) in directives::aliases() {
        assert_eq!(directives::canonical(alias), Some(canon));
        assert!(directives::is_alias(alias) && !directives::is_alias(canon));
        assert_ne!(directives::normalize(alias, "x").0, alias);
    }
    // die neun Namen aus dem Konzept
    for (old, new) in [(".pseudopc", ".relocate"), (".realpc", ".endrelocate"), (".endpseudopc", ".endrelocate"), (".rta", ".retaddr"), (".lohifill", ".lohitable"),
                       (".screen", ".screencode"), (".null", ".textnull"), (".ptext", ".textlen"), (".shift", ".textshift"), (".page", ".align"),
                       (".next", ".endfor"), (".dbyt", ".be16"), (".long", ".dword")] {
        assert_eq!(directives::canonical(old), Some(new), "{old}");
    }
    assert_eq!(directives::canonical(".be32"), Some(".be32"));
    assert_eq!(directives::canonical(".entry"), None, ".entry entfällt");
    assert_eq!(directives::canonical(".unbekannt"), None);
}

#[test]
fn unknown_directives_still_report_e208() {
    assert_eq!(run(".org $1000\n.entry start\n"), Err(vec!["E208".to_string()]));
    assert_eq!(run(".org $1000\n.pseudo $10\n"), Err(vec!["E208".to_string()]));
}
