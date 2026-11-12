// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_asm::{assemble, disassemble, listing, to_bin, to_xex, Options, Policy};
use vc_target::{load_cpu, Cpu, Mode};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn cpu() -> Cpu { load_cpu(&data(), "6510").unwrap() }
fn asm(src: &str) -> Vec<u8> {
    let out = assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap_or_else(|e| panic!("{e:?}"));
    out.segments[0].bytes.clone()
}
fn code(src: &str) -> String {
    match assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)) { Err(e) => e[0].code.to_string(), Ok(_) => "ok".into() }
}

#[test]
fn conditional_assembly() {
    let src = ".const MODE = 2\n.org $1000\n.if MODE == 1\n lda #1\n.elseif MODE == 2\n lda #2\n.else\n lda #3\n.endif\n.ifdef MODE\n nop\n.endif\n.ifndef NOPE\n rts\n.endif\n";
    assert_eq!(asm(src), vec![0xA9, 0x02, 0xEA, 0x60]);
    assert_eq!(asm(".org $1000\n.if 1 && (2 > 1) || 0\n nop\n.endif\n.if !1\n rts\n.endif\n"), vec![0xEA]);
}

#[test]
fn macros_with_unique_local_labels() {
    let src = ".macro wait_line(line)\n lda #line\n@w: cmp $d012\n bne @w\n.endmacro\n.org $c000\nstart: wait_line $40\n wait_line($80)\n";
    assert_eq!(asm(src), vec![0xA9, 0x40, 0xCD, 0x12, 0xD0, 0xD0, 0xFB, 0xA9, 0x80, 0xCD, 0x12, 0xD0, 0xD0, 0xFB]);
    let two = ".macro twice(a, b)\n lda a\n sta b\n.endmacro\n.org $1000\n twice #1, $d020\n";
    assert_eq!(asm(two), vec![0xA9, 0x01, 0x8D, 0x20, 0xD0]);
}

#[test]
fn repeat_blocks() {
    assert_eq!(asm(".org $1000\n.repeat 4, i\n .byte i*2\n.endrepeat\n.repeat 3\n inx\n.endrepeat\n"), vec![0, 2, 4, 6, 0xE8, 0xE8, 0xE8]);
}

#[test]
fn petscii_and_screen_codes() {
    assert_eq!(asm(".org $1000\n.screen \"hello\"\n.petscii \"Ab\"\n.text \"A\"\n"), vec![8, 5, 12, 12, 15, 0xC1, 0x42, 0x41]);
}

#[test]
fn command_line_defines() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.defines = vec![("DEBUG".into(), 1)];
    let src = ".org $1000\n.if DEBUG\n nop\n.else\n rts\n.endif\n";
    assert_eq!(assemble(src, &cpu(), &o).unwrap().segments[0].bytes, vec![0xEA]);
}

#[test]
fn include_and_incbin() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("inc_test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib.inc"), ".const LIVES = 3\nhelper: rts\n").unwrap();
    std::fs::write(dir.join("data.bin"), [9u8, 8, 7, 6]).unwrap();
    let main = ".org $2000\n.include \"lib.inc\"\n lda #LIVES\n jsr helper\n.incbin \"data.bin\", 1, 2\n";
    let o = Options::new(dir.join("main.asm").to_string_lossy(), Policy::Allow);
    let out = assemble(main, &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0x60, 0xA9, 3, 0x20, 0x00, 0x20, 8, 7]);
    assert_eq!(code(".org $1000\n.include \"missing.inc\"\n"), "E212");
}

#[test]
fn disassembler_roundtrip_for_every_instruction() {
    let c = cpu();
    let mut src = String::from(".org $1000\n");
    for op in c.canonical_ops() {
        let operand = match op.mode {
            Mode::Imp => "", Mode::Acc => " a", Mode::Imm => " #$12", Mode::Zp => " $12", Mode::Zpx => " $12,x", Mode::Zpy => " $12,y",
            Mode::Abs => " $1234", Mode::Abx => " $1234,x", Mode::Aby => " $1234,y", Mode::Ind => " ($1234)",
            Mode::Izx => " ($12,x)", Mode::Izy => " ($12),y", Mode::Rel => " *+2", _ => unreachable!("nur 6502-Modi"),
        };
        src.push_str(&format!(" {}{}\n", op.mnemonic, operand));
    }
    let bytes = asm(&src);
    let lines = disassemble(&c, &bytes, 0x1000);
    let mut again = String::from(".org $1000\n");
    for l in &lines { again.push_str(&format!(" {}\n", l.text)); }
    assert_eq!(asm(&again), bytes, "Disassemblieren und erneutes Assemblieren muss identisch sein");
    assert_eq!(lines.iter().map(|l| l.bytes.len()).sum::<usize>(), bytes.len());
}

#[test]
fn listing_and_line_map() {
    let out = assemble(".org $1000\nstart: lda #2 ; Kommentar\n nop\n", &cpu(), &Options::new("t.asm", Policy::Warn)).unwrap();
    let l = listing(&out);
    assert!(l.contains("1000  a9 02") && l.contains("lda #2"), "{l}");
    assert!(out.lines.iter().any(|x| x.line == 2 && x.cycles == Some((2, 2))));
    assert_eq!(to_bin(&out).unwrap(), vec![0xA9, 2, 0xEA]);
}

#[test]
fn structural_errors() {
    assert_eq!(code(".org $1000\n.if 1\n nop\n"), "E210");
    assert_eq!(code(".org $1000\n.endif\n"), "E210");
    assert_eq!(code(".macro m(a)\n nop\n.endmacro\n.org $1000\n m 1, 2\n"), "E209");
    assert_eq!(code(".macro nop\n rts\n.endmacro\n"), "E209");
    assert_eq!(code(".org $1000\n.error \"kaputt\"\n"), "E207");
}

#[test]
fn pseudopc_assembles_for_another_address() {
    let src = ".org $1000\n jmp $c000\n.pseudopc $c000\nhere: lda #1\n jmp here\n.endpseudopc\n nop\n";
    assert_eq!(asm(src), vec![0x4C, 0x00, 0xC0, 0xA9, 0x01, 0x4C, 0x00, 0xC0, 0xEA]);
}

#[test]
fn cycle_budgets() {
    let body = ".org $1000\n.cycles irq max=LIMIT\n lda #1\n sta $d020\n bne *+2\n.endcycles\n";
    let ok = assemble(&body.replace("LIMIT", "10"), &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    assert!(ok.warnings.is_empty());
    assert_eq!((ok.cycles[0].min, ok.cycles[0].max, ok.cycles[0].budget), (8, 10, Some(10)));
    let over = assemble(&body.replace("LIMIT", "9"), &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    assert_eq!(over.warnings[0].code, "W201");
    assert_eq!(code(".org $1000\n.cycles x\n nop\n"), "E210");
}

#[test]
fn xex_output_with_run_address() {
    let out = assemble(".org $2000\nstart: rts\n.run start\n", &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    assert_eq!(to_xex(&out).unwrap(), vec![0xFF, 0xFF, 0x00, 0x20, 0x00, 0x20, 0x60, 0xE0, 0x02, 0xE1, 0x02, 0x00, 0x20]);
}

#[test]
fn label_before_block_directive_is_kept() {
    assert_eq!(asm(".org $1000\ntab: .repeat 2\n .byte 1\n.endrepeat\n lda tab\n"), vec![1, 1, 0xAD, 0x00, 0x10]);
}

#[test]
fn diagnostics_render_with_source_excerpt() {
    let e = assemble(".org $1000\n foo\n", &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap_err();
    let r = e[0].render(Some(" foo"));
    assert!(r.contains("error[E205]") && r.contains("--> t.asm:2") && r.contains("2 |  foo"), "{r}");
}

#[test]
fn system_library_and_slash_comments() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    let src = ".org $0801\n#include <c64/hardware.asm>\n#include <c64/kernal.asm>\n lda #15 // grau\n sta VIC_BORDERCOLOR\n jsr CHROUT\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(out.segments[0].bytes, vec![0xA9, 0x0F, 0x8D, 0x20, 0xD0, 0x20, 0xD2, 0xFF]);
    // Atari-Bibliothek: eigene Namen, gleiche Adressen wie die Chip-Daten
    let src = ".org $2000\n#include <atari8bit/gtia.asm>\n#include <atari8bit/os.asm>\n sta COLBK\n sta COLOR4\n jsr CIOV\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(out.segments[0].bytes, vec![0x8D, 0x1A, 0xD0, 0x8D, 0xC8, 0x02, 0x20, 0x56, 0xE4]);
    assert_eq!(code(".org $1000\n#include <c64/gibts_nicht.asm>\n"), "E212");
    assert_eq!(code(".org $1000\n#include foo\n"), "E208");
}

#[test]
fn scopes_blocks_and_named() {
    // gleiche Labelnamen in getrennten Scopes; Zugriff von außen über name.label; Zugriff nach außen ohne Präfix
    let src = ".org $1000\nVAL = 7\ninit: {\n        ldx #VAL\nloop:   dex\n        bne loop\n        rts\n}\nmain: {\n        jsr init\n        jmp init.loop\nloop:   nop\n        jmp loop\n}\n";
    // init=$1000, init.loop=$1002, main=$1006, main.loop=$100C
    assert_eq!(asm(src), vec![0xA2, 7, 0xCA, 0xD0, 0xFD, 0x60, 0x20, 0x00, 0x10, 0x4C, 0x02, 0x10, 0xEA, 0x4C, 0x0C, 0x10]);
}

#[test]
fn scopes_nested_anonymous_and_constants() {
    let src = ".org $2000\n{\n  X = 1\n  {\n    X = 2\n    lda #X\n  }\n  lda #X\n}\nX = 9\nlda #X\n";
    assert_eq!(asm(src), vec![0xA9, 2, 0xA9, 1, 0xA9, 9]);
    // äußere Scopes sind von innen sichtbar, innere von außen nur über den Namen
    let src = ".org $2000\nouter: {\n A = 5\n inner: {\n  lda #A\n  B = 6\n }\n lda #inner.B\n}\nlda #outer.A\nlda #outer.inner.B\n";
    assert_eq!(asm(src), vec![0xA9, 5, 0xA9, 6, 0xA9, 5, 0xA9, 6]);
    assert_eq!(code(".org $1000\n{\n Y = 1\n}\nlda #Y\n"), "E202");
}

#[test]
fn scopes_locals_namespace_and_errors() {
    // lokale `.labels` gelten je Scope getrennt
    let src = ".org $1000\na: {\nf: ldx #1\n.l: dex\n bne .l\n}\nb: {\ng: ldx #2\n.l: dex\n bne .l\n}\n";
    assert_eq!(asm(src), vec![0xA2, 1, 0xCA, 0xD0, 0xFD, 0xA2, 2, 0xCA, 0xD0, 0xFD]);
    // .namespace definiert kein Label
    assert_eq!(asm(".org $1000\n.namespace lib {\nrts_: rts\n}\n jmp lib.rts_\n"), vec![0x60, 0x4C, 0x00, 0x10]);
    // Fehler: offener und überzähliger Scope, doppelt im selben Scope
    assert_eq!(code(".org $1000\n{\n nop\n"), "E213");
    assert_eq!(code(".org $1000\n nop\n}\n"), "E213");
    assert_eq!(code(".org $1000\n{\nz: nop\nz: nop\n}\n"), "E204");
    assert_eq!(code(".org $1000\n{\nz: nop\n}\n{\nz: nop\n}\n"), "ok");
    // Makros: Scopes im Makrorumpf lösen die Kollision beim mehrfachen Aufruf
    let m = ".macro delay(n)\n{\n ldx #n\nl: dex\n bne l\n}\n.endmacro\n.org $1000\n delay 3\n delay 4\n";
    assert_eq!(asm(m), vec![0xA2, 3, 0xCA, 0xD0, 0xFD, 0xA2, 4, 0xCA, 0xD0, 0xFD]);
}

#[test]
fn include_as_namespace() {
    let dir = std::env::temp_dir().join(format!("vc_inc_ns_{}", std::process::id())); std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("lib.inc"), "SPEED = 3\nplay: lda #SPEED\n rts\n").unwrap();
    let mut o = Options::new(dir.join("main.asm").to_str().unwrap(), Policy::Allow);
    o.include_dirs = vec![dir.clone()];
    let src = ".org $1000\n.include \"lib.inc\" as music\nstart: jsr music.play\n lda #music.SPEED\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(out.segments[0].bytes, vec![0xA9, 3, 0x60, 0x20, 0x00, 0x10, 0xA9, 3]);
    // ohne Namensraum bleibt alles global
    let out = assemble(".org $1000\n.include \"lib.inc\"\n jsr play\n", &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0xA9, 3, 0x60, 0x20, 0x00, 0x10]);
    // zwei Kopien derselben Datei kollidieren nicht
    let out = assemble(".org $1000\n.include \"lib.inc\" as a\n.include \"lib.inc\" as b\n jsr b.play\n", &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes.len(), 9);
    let e = assemble(".org $1000\n.include \"lib.inc\" as 9x\n", &cpu(), &o).unwrap_err(); assert_eq!(e[0].code, "E208");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Jede Systembibliothek muss für sich assemblieren; Namen verschiedener Systeme dürfen sich überschneiden (daher getrennt je System).
#[test]
fn every_system_library_assembles() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    let groups: &[(&str, &[&str])] = &[
        ("c64", &["c64/hardware.asm", "c64/kernal.asm", "c64/basic.asm"]),
        ("c128", &["c128/hardware.asm", "c128/kernal.asm"]),
        ("plus4", &["plus4/hardware.asm", "plus4/kernal.asm"]),
        ("mega65 geschlossen", &["mega65/hardware.asm", "mega65/gs4502.asm", "mega65/rom_920418.asm"]),
        ("mega65 open", &["mega65/hardware.asm", "mega65/rom_open.asm"]),
        ("atari", &["atari8bit/antic.asm", "atari8bit/gtia.asm", "atari8bit/pokey.asm", "atari8bit/os.asm"]),
        ("vcs", &["vcs2600/tia.asm"]),
        ("common", &["common/types.asm", "common/math.asm"]),
    ];
    for (name, files) in groups {
        let mut src: String = files.iter().map(|f| format!("#include <{f}>\n")).collect();
        src.push_str(".org $2000\n nop\n");
        // doppeltes Einbinden (auch über andere Bibliotheken) ist erlaubt
        src.push_str(&files.iter().map(|f| format!("#include <{f}>\n")).collect::<String>());
        if let Err(e) = assemble(&src, &cpu(), &o) { panic!("{name}: {e:?}"); }
    }
    // zwei MEGA65-ROM-Varianten schließen sich aus
    assert_eq!(code(".org $1000\n#include <mega65/rom_open.asm>\n"), "E212");   // ohne sys-Pfad nicht auffindbar
    let src = "#include <mega65/rom_open.asm>\n#include <mega65/rom_920418.asm>\n";
    assert!(assemble(src, &cpu(), &o).is_err());
    // C128 und MEGA65 benutzen gleiche Namen mit verschiedenen Adressen: gemeinsames Einbinden ist ein Fehler
    assert!(assemble("#include <c128/kernal.asm>\n#include <mega65/kernal.asm>\n", &cpu(), &o).is_err());
}

#[test]
fn system_macros_and_math() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    let src = "#include <common/math.asm>\n.org $2000\n inc16 $fb\n set_bits $d020, BIT3\n ld16 $fd, $1234\n add16i $fb, 5\n cmp16 $fb, $fd\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(&out.segments[0].bytes[..9], &[0xE6, 0xFB, 0xD0, 0x02, 0xE6, 0xFC, 0xAD, 0x20, 0xD0]);
    // MATH_ZP muss in die Nullseite passen
    assert!(assemble(".const MATH_ZP = $ff\n#include <common/math.asm>\n", &cpu(), &o).is_err());
}

#[test]
fn system_libraries_in_namespaces() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    // gleiche Namen bei verschiedenen Systemen sind mit Namensraum nebeneinander möglich
    let src = "#include <c128/kernal.asm> as c128\n#include <mega65/kernal.asm> as m65\n#include <c64/hardware.asm> as c64\n.org $2000\n jsr c128.SPIN_SPOUT\n jsr m65.SPIN_SPOUT\n sta c64.VIC_BORDERCOLOR\n sta c128.VIC_BORDERCOLOR\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(out.segments[0].bytes, vec![0x20, 0x47, 0xFF, 0x20, 0x4D, 0xFF, 0x8D, 0x20, 0xD0, 0x8D, 0x20, 0xD0]);
}

#[test]
fn star_equals_is_org() {
    assert_eq!(asm("*= $c000\n nop\n* = $c010\nl: jmp l\n"), asm(".org $c000\n nop\n.org $c010\nl: jmp l\n"));
    let out = assemble("*=$1000\n nop\n*=$1010\n nop\n", &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    assert_eq!(out.segments.iter().map(|s| s.start).collect::<Vec<_>>(), vec![0x1000, 0x1010]);
    // `*` als Programmzähler im Ausdruck bleibt unberührt
    assert_eq!(asm(".org $1000\nx: .word *\n"), vec![0x00, 0x10]);
}

#[test]
fn page_and_next_address() {
    // .page: nächste 256-Byte-Seite ohne Füllbytes (neues Segment); .org next(n): nächste durch n teilbare Adresse
    let out = assemble(".org $1010\n nop\n.page\nl: nop\n.org next(1024)\nm: nop\n .word l, m\n", &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    let starts: Vec<u16> = out.segments.iter().map(|s| s.start).collect();
    assert_eq!(starts, vec![0x1010, 0x1100, 0x1400]);
    assert_eq!(out.segments[2].bytes, vec![0xEA, 0x00, 0x11, 0x00, 0x14]);
    // schon ausgerichtet: bleibt stehen; .page 1024 und align() als Funktion
    let out = assemble(".org $2000\n.page\na: nop\n.page 1024\nb: nop\n.org align(256) + 16\nc: nop\n", &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    let syms = &out.symbols;
    assert_eq!((syms["a"], syms["b"], syms["c"]), (0x2000, 0x2400, 0x2510));
    // `next` als Label bleibt ein Label
    assert_eq!(asm(".org $1000\nnext: jmp next\n jmp (next)\n"), vec![0x4C, 0x00, 0x10, 0x6C, 0x00, 0x10]);
}

#[test]
fn anonymous_labels_backward_and_forward() {
    // Schleife rückwärts (`-`), Sprung vorwärts (`+`)
    let src = ".org $1000\n ldx #8\n- dex\n bne -\n beq +\n nop\n+ rts\n";
    // $1000 ldx; $1002 dex; $1003 bne -> $1002 (FD); $1005 beq + -> $1008 (01); $1007 nop; $1008 rts
    assert_eq!(asm(src), vec![0xA2, 8, 0xCA, 0xD0, 0xFD, 0xF0, 0x01, 0xEA, 0x60]);
    // mehrere Ebenen: `--` springt zum vorletzten, `++` zum übernächsten Label
    let src = ".org $1000\n- nop\n- nop\n bne --\n bne +\n bne ++\n+ nop\n+ nop\n";
    // $1000 nop, $1001 nop, $1002 bne -- -> $1000 (FC), $1004 bne + -> $1008 (02), $1006 bne ++ -> $1009 (01), $1008 nop, $1009 nop
    assert_eq!(asm(src), vec![0xEA, 0xEA, 0xD0, 0xFC, 0xD0, 0x02, 0xD0, 0x01, 0xEA, 0xEA]);
    // auch in Makros und mit jmp
    let src = ".macro skip()\n beq +\n nop\n+ nop\n.endmacro\n.org $1000\n skip\n skip\n- nop\n jmp -\n";
    assert_eq!(code(".org $1000\n bne -\n"), "E214");
    assert_eq!(code(".org $1000\n bne +\n"), "E214");
    assert_eq!(asm(src)[0..3], [0xF0, 0x01, 0xEA]);
}

#[test]
fn anonymous_labels_do_not_appear_in_symbols() {
    let out = vc_asm::assemble(".org $1000\nstart:\n- dex\n bne -\n", &cpu(), &vc_asm::Options::new("t.asm", vc_asm::Policy::Deny)).unwrap();
    assert!(out.symbols.keys().all(|k| !k.contains('#')), "{:?}", out.symbols);
    assert!(out.symbols.contains_key("start"));
}

#[test]
fn forced_address_mode() {
    // ohne Suffix: kleiner Wert → Zeropage; `.a` erzwingt absolut, `.z` Zeropage auch bei Vorwärtsreferenz
    assert_eq!(asm(".org $1000\n lda $10\n"), vec![0xA5, 0x10]);
    assert_eq!(asm(".org $1000\n lda.a $10\n"), vec![0xAD, 0x10, 0x00]);
    assert_eq!(asm(".org $1000\n lda.abs $10,x\n lda.ABS $10,y\n"), vec![0xBD, 0x10, 0x00, 0xB9, 0x10, 0x00]);
    assert_eq!(asm(".org $1000\n lda.z zp\nzp = $20\n"), vec![0xA5, 0x20]);
    assert_eq!(asm(".org $1000\n lda.zp $20,x\n ldx.z $30,y\n"), vec![0xB5, 0x20, 0xB6, 0x30]);
    // Fehler: keine solche Form, Sprungbefehl, Wert zu groß, andere Operandenart
    assert_eq!(code(".org $1000\n jmp.z $20\n"), "E215");
    assert_eq!(code(".org $1000\n bne.a $1000\n"), "E215");
    assert_eq!(code(".org $1000\n lda.z #1\n"), "E215");
    assert_eq!(code(".org $1000\n lda.z $1234\n"), "E203");
}

#[test]
fn encoding_state_and_block() {
    // Standard: unverändert; Zustand gilt bis zur nächsten Umschaltung
    assert_eq!(asm(".org $1000\n .text \"aA\"\n"), vec![0x61, 0x41]);
    assert_eq!(asm(".org $1000\n .encoding \"petscii\"\n .text \"aA\"\n .encoding \"ascii\"\n .text \"a\"\n"), vec![0x41, 0xC1, 0x61]);
    assert_eq!(asm(".org $1000\n .encoding \"screen\"\n .text \"a@\"\n"), vec![0x01, 0x00]);
    // Block: danach gilt wieder die vorige Kodierung
    assert_eq!(asm(".org $1000\n .encoding \"screen\" {\n .text \"a\"\n }\n .text \"a\"\n"), vec![0x01, 0x61]);
    // Block verändert keine Scopes: Label bleibt außerhalb sichtbar
    assert_eq!(asm(".org $1000\n .encoding \"screen\" {\nx: .text \"a\"\n }\n .word x\n"), vec![0x01, 0x00, 0x10]);
    // Atari: ATASCII (Zeilenende) und interner Code
    assert_eq!(asm(".org $1000
 .encoding \"atascii\"
 .text \"Ab\"
"), vec![0x41, 0x62]);
    assert_eq!(code(".org $1000\n .encoding \"gibtsnicht\"\n"), "E208");
    assert_eq!(code(".org $1000\n .encoding screen\n"), "E208");
}

#[test]
fn data_directives() {
    assert_eq!(asm(".org $1000\n .null \"AB\"\n"), vec![0x41, 0x42, 0x00]);
    assert_eq!(asm(".org $1000\n .ptext \"AB\"\n"), vec![2, 0x41, 0x42]);
    assert_eq!(asm(".org $1000\n .shift \"END\"\n"), vec![0x45, 0x4E, 0xC4]);
    assert_eq!(code(".org $1000\n .shift \"\"\n"), "E203");
    assert_eq!(asm(".org $1000\n .rta $1234, 1\n"), vec![0x33, 0x12, 0x00, 0x00]);
    assert_eq!(asm(".org $1000\n .be16 $1234\n .dbyt 1\n"), vec![0x12, 0x34, 0x00, 0x01]);
    assert_eq!(asm(".org $1000\n .dword $12345678, -1\n"), vec![0x78, 0x56, 0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(asm(".org $1000\n .hex \"aa bb0C\"\n"), vec![0xAA, 0xBB, 0x0C]);
    assert_eq!(code(".org $1000\n .hex \"abc\"\n"), "E208");
    // Kodierung wirkt auf .null/.ptext/.shift
    assert_eq!(asm(".org $1000\n .encoding \"screen\"\n .null \"a\"\n"), vec![0x01, 0x00]);
}

#[test]
fn atascii_directive() {
    assert_eq!(asm(".org $1000\n .atascii \"Ab\"\n"), vec![0x41, 0x62]);
    assert_eq!(asm(".org $1000\n .encoding \"petscii\"\n .atascii \"a\"\n .text \"a\"\n"), vec![0x61, 0x41]);
}

#[test]
fn screen_follows_target_text_encoding() {
    let run = |src: &str, atari: bool| {
        let mut o = Options::new("t.asm", Policy::Allow); o.atari_text = atari;
        assemble(src, &cpu(), &o).unwrap().segments[0].bytes.clone()
    };
    // C64: kleine Buchstaben → 1..; Atari (ANTIC): $20–$5F → −$20, $00–$1F → +$40
    assert_eq!(run(".org $1000\n .screen \"a\"\n", false), vec![0x01]);
    assert_eq!(run(".org $1000\n .screen \"A 0\"\n", true), vec![0x21, 0x00, 0x10]);
    assert_eq!(run(".org $1000\n .encoding \"screen\"\n .text \"A 0\"\n", true), vec![0x21, 0x00, 0x10]);
    assert_eq!(run(".org $1000\n .encoding \"screen\"\n .null \"A\"\n", true), vec![0x21, 0x00]);
}

#[test]
fn byte_macros() {
    let mut o = Options::new("t.asm", Policy::Allow);
    o.include_dirs.push(data().join("sys"));
    let src = "#include <common/types.asm>\n.org $2000\n mov8 $d020, #1\n movx8 $fb, $fc\n movy8 $fd, #2\n add8 $fb, #5\n sub8 $fb, $fc\n";
    let out = assemble(src, &cpu(), &o).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(out.segments[0].bytes, vec![
        0xA9, 1, 0x8D, 0x20, 0xD0,          // mov8
        0xA6, 0xFC, 0x86, 0xFB,             // movx8
        0xA0, 2, 0x84, 0xFD,                // movy8
        0x18, 0xA5, 0xFB, 0x69, 5, 0x85, 0xFB,   // add8
        0x38, 0xA5, 0xFB, 0xE5, 0xFC, 0x85, 0xFB, // sub8
    ]);
}

#[test]
fn macro_default_parameters() {
    let src = ".macro setc(v, reg=a)\n .if reg == 1\n ldx #v\n .else\n lda #v\n .endif\n.endmacro\n.org $1000\n setc 5\n setc 6, 1\n";
    assert_eq!(asm(&src.replace("reg=a", "reg=0")), vec![0xA9, 5, 0xA2, 6]);
    // mehrere Standardwerte, Argumente von links
    let src = ".macro m(a, b=2, c=3)\n .byte a, b, c\n.endmacro\n.org $1000\n m 1\n m 1, 9\n m 1, 9, 8\n";
    assert_eq!(asm(src), vec![1, 2, 3, 1, 9, 3, 1, 9, 8]);
    // Standardwert darf Klammern enthalten
    assert_eq!(asm(".macro m(a=(1+1))\n .byte a\n.endmacro\n.org $1000\n m\n"), vec![2]);
    // Fehler: zu wenige / zu viele Argumente, Pflichtparameter hinter Standardwert
    assert_eq!(code(".macro m(a, b=2)\n.endmacro\n.org $1000\n m\n"), "E209");
    assert_eq!(code(".macro m(a, b=2)\n.endmacro\n.org $1000\n m 1,2,3\n"), "E209");
    assert_eq!(code(".macro m(a=1, b)\n.endmacro\n.org $1000\n"), "E210");
}

#[test]
fn exitmacro_stops_the_macro() {
    let src = ".macro m(n)\n .byte 1\n .if n == 0\n .exitmacro\n .endif\n .byte 2\n.endmacro\n.org $1000\n m 0\n m 1\n";
    assert_eq!(asm(src), vec![1, 1, 2]);
    // aus einer Wiederholung heraus; verschachteltes Makro beendet nur sich selbst
    let src = ".macro inner\n .byte 7\n .exitmacro\n .byte 8\n.endmacro\n.macro outer\n inner\n .repeat 3\n .byte 9\n .exitmacro\n .endrepeat\n .byte 10\n.endmacro\n.org $1000\n outer\n .byte 11\n";
    assert_eq!(asm(src), vec![7, 9, 11]);
    assert_eq!(code(".org $1000\n .exitmacro\n"), "E210");
}

fn warns(src: &str) -> Vec<String> {
    match assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)) { Ok(o) => o.warnings.iter().map(|w| w.code.to_string()).collect(), Err(e) => vec![format!("ERR {}", e[0].code)] }
}

#[test]
fn variables_can_be_reassigned() {
    let src = ".org $1000\n.var x = 1\n .byte x\nx = x + 1\n .byte x\n.set x = x * 5\n .byte x\nx += 3\n x -= 1\n .byte x\n x <<= 1\n .byte x\n";
    assert_eq!(asm(src), vec![1, 2, 10, 12, 24]);
    // Konstante bleibt Konstante: Neuzuweisung ist weiter ein Fehler
    assert_eq!(code(".org $1000\nk = 1\nk = 2\n"), "E204");
    assert_eq!(code(".org $1000\n.const k = 1\n.set k = 2\n"), "E216");
    // Variable vor der Definition ist nicht lesbar
    assert_eq!(code(".org $1000\n .byte y\n.var y = 1\n"), "E202");
    // Wert eines Labels in einer Variablen (Vorwärtsreferenz wird über die Durchläufe aufgelöst)
    assert_eq!(asm(".org $1000\n.var p = ziel\n .word p\nziel: nop\n"), vec![0x02, 0x10, 0xEA]);
    // Variablen leben in Scopes
    assert_eq!(asm(".org $1000\n.var v = 7\n{\n.var v = 9\n .byte v\n}\n .byte v\n"), vec![9, 7]);
}

#[test]
fn math_functions_and_float_values() {
    assert_eq!(asm(".org $1000\n .byte int(sqrt(16)), abs(-3), min(5, 2, 9), max(5, 2, 9), sign(-4), round(2.5), floor(2.9), ceil(2.1)\n"),
               vec![4, 3, 2, 9, 255, 3, 2, 3]);
    assert_eq!(asm(".org $1000\n .byte round(128 + 127 * sin(pi / 2)), int(cos(0) * 10), pow(2, 8) - 1 - 255, round(log2(64))\n"), vec![255, 10, 0, 6]);
    // / bleibt ganzzahlig, mit Kommazahl wird gerechnet
    assert_eq!(asm(".org $1000\n .byte 7 / 2, int(7 / 2.0 * 10), 3e1 / 10\n"), vec![3, 35, 3]);
    assert_eq!(asm(".org $1000\n .byte int(atan2(1, 1) * 4 * 50)\n"), vec![157]);
    // Variablen halten Kommazahlen
    assert_eq!(asm(".org $1000\n.var f = 0.5\nf = f * 3\n .byte int(f * 10)\n"), vec![15]);
    // Funktionsnamen sind nur mit Klammer Funktionen, sonst normale Symbole
    assert_eq!(asm(".org $1000\nmax = 3\n .byte max, max(1, 2)\n"), vec![3, 2]);
    // Fehler
    assert_eq!(code(".org $1000\n .byte sin(1, 2)\n"), "E202");
    assert_eq!(code(".org $1000\n .byte 1.5 & 1\n"), "E202");
    assert_eq!(code(".org $1000\n .byte sqrt(0-1)\n"), "E202");
    assert_eq!(code(".org $1000\n .byte 1.5 % 2\n"), "E202");
}

#[test]
fn fractions_warn_when_truncated() {
    assert_eq!(warns(".org $1000\n .byte 2.5\n"), vec!["W202"]);
    assert_eq!(asm(".org $1000\n .byte 2.5\n"), vec![2]);
    assert!(warns(".org $1000\n .byte int(2.5), 2.0, round(2.5)\n.var f = 2.5\n").is_empty());
    assert_eq!(warns(".org $1000\n.var f = 1.5\n .word f\n"), vec!["W202"]);
}

#[test]
fn sine_table_with_repeat() {
    let b = asm(".org $1000\nsinus:\n.repeat 256, i\n .byte round(128 + 127 * sin(i * 2 * pi / 256))\n.endrepeat\n");
    assert_eq!(b.len(), 256);
    assert_eq!((b[0], b[64], b[128], b[192]), (128, 255, 128, 1));
}

#[test]
fn for_loops() {
    // C-Form mit Schrittweite
    assert_eq!(asm(".org $1000\n.for i = 0, i < 10, i += 3\n .byte i\n.next\n"), vec![0, 3, 6, 9]);
    // range: bis ausschließlich, mit Anfang, mit Schritt (auch rückwärts)
    assert_eq!(asm(".org $1000\n.for i in range(3)\n .byte i\n.next\n"), vec![0, 1, 2]);
    assert_eq!(asm(".org $1000\n.for i in range(2, 5)\n .byte i\n.next\n"), vec![2, 3, 4]);
    assert_eq!(asm(".org $1000\n.for i in range(10, 0, -4)\n .byte i\n.next\n"), vec![10, 6, 2]);
    // verschachtelt; die Laufvariable ist danach wieder weg bzw. hat den alten Wert
    assert_eq!(asm(".org $1000\n.var i = 99\n.for i in range(2)\n.for j in range(2)\n .byte i * 10 + j\n.next\n.next\n .byte i\n"), vec![0, 1, 10, 11, 99]);
    assert_eq!(code(".org $1000\n.for i in range(2)\n.next\n .byte i\n"), "E202");
    // break / continue (auch in .repeat und aus .if heraus)
    assert_eq!(asm(".org $1000\n.for i in range(10)\n .if i == 2\n  .continue\n .endif\n .if i == 5\n  .break\n .endif\n .byte i\n.next\n"), vec![0, 1, 3, 4]);
    assert_eq!(asm(".org $1000\n.repeat 5, k\n .if k == 3\n  .break\n .endif\n .byte k\n.endrepeat\n"), vec![0, 1, 2]);
    assert_eq!(asm(".org $1000\n.for i in range(3)\n.repeat 5, k\n .if k == 1\n  .break\n .endif\n .byte i\n.endrepeat\n.next\n"), vec![0, 1, 2]);
    // Fehler
    assert_eq!(code(".org $1000\n .break\n"), "E210");
    assert_eq!(code(".org $1000\n.for i = 0, i < 1, i += 1\n"), "E210");
    assert_eq!(code(".org $1000\n.next\n"), "E210");
    assert_eq!(code(".org $1000\n.for i = 0, i >= 0, i += 1\n.next\n"), "E217");
    assert_eq!(code(".org $1000\n.for i in range(0, 5, 0)\n.next\n"), "E202");
    assert_eq!(code(".org $1000\n.for i = 0\n.next\n"), "E208");
    // Makro in einer Schleife: .break im Makro betrifft nicht die Schleife außen
    assert_eq!(code(".macro m\n .break\n.endmacro\n.org $1000\n.for i in range(2)\n m\n.next\n"), "E210");
}

#[test]
fn user_functions() {
    assert_eq!(asm(".org $1000\n.function lerp(a, b, t) = a + (b - a) * t / 255\n .byte lerp(0, 200, 255), lerp(10, 20, 0)\n"), vec![200, 10]);
    // Funktionen rufen Funktionen und eingebaute Funktionen; sehen Konstanten und Variablen
    let src = ".org $1000\nK = 3\n.var s = 2\n.function sq(x) = x * x\n.function f(x) = sq(x) + K + s + int(sqrt(16))\n .byte f(2)\n";
    assert_eq!(asm(src), vec![4 + 3 + 2 + 4]);
    // Parameter verdecken gleichnamige Variablen
    assert_eq!(asm(".org $1000\n.var x = 100\n.function id(x) = x\n .byte id(7), x\n"), vec![7, 100]);
    // Kommazahlen laufen durch
    assert_eq!(asm(".org $1000\n.function half(x) = x / 2.0\n .byte int(half(5) * 2)\n"), vec![5]);
    // Fehler: falsche Argumentzahl, Rekursion, doppelte Definition, eingebauter Name, vor der Definition benutzt
    assert_eq!(code(".org $1000\n.function f(a) = a\n .byte f(1, 2)\n"), "E202");
    assert_eq!(code(".org $1000\n.function f(a) = f(a)\n .byte f(1)\n"), "E202");
    assert_eq!(code(".org $1000\n.function f(a) = a\n.function f(a) = a\n"), "E204");
    assert_eq!(code(".org $1000\n.function sin(a) = a\n"), "E204");
    assert_eq!(code(".org $1000\n .byte g(1)\n.function g(a) = a\n"), "E202");
    assert_eq!(code(".org $1000\n.function g = 1\n"), "E208");
}

#[test]
fn lists() {
    // Literal, Ausgabe der ganzen Liste, Elementzugriff, Länge
    assert_eq!(asm(".org $1000\n.var t = [1, 2, 3]\n .byte t\n .byte t[1], len(t), t[-1]\n"), vec![1, 2, 3, 2, 3, 3]);
    assert_eq!(asm(".org $1000\n .byte [4, 5], 6\n .word [$102, 3]\n"), vec![4, 5, 6, 2, 1, 3, 0]);
    // Ausschnitte
    assert_eq!(asm(".org $1000\n.var t = [10, 20, 30, 40]\n .byte t[1:3], t[:1], t[2:], t[3:9]\n"), vec![20, 30, 10, 30, 40, 40]);
    // Aufbau: leere Liste, + hängt an, * wiederholt
    assert_eq!(asm(".org $1000\n.var t = []\n.for i in range(4)\n t += [i * i]\n.next\n .byte t, len(t)\n"), vec![0, 1, 4, 9, 4]);
    assert_eq!(asm(".org $1000\n .byte [7] * 3, 2 * [1, 2]\n"), vec![7, 7, 7, 1, 2, 1, 2]);
    // verschachtelt wird flach ausgegeben
    assert_eq!(asm(".org $1000\n .byte [[1, 2], [3]]\n"), vec![1, 2, 3]);
    // range als Liste, in .for über eine Liste, Elemente mit Berechnung
    assert_eq!(asm(".org $1000\n .byte range(3), range(5, 1, -2), len(range(2, 10, 3))\n"), vec![0, 1, 2, 5, 3, 3]);
    assert_eq!(asm(".org $1000\n.for c in [2, 5, 7]\n .byte c * 2\n.next\n"), vec![4, 10, 14]);
    // Listen in Funktionen
    assert_eq!(asm(".org $1000\n.function last(l) = l[len(l) - 1]\n .byte last([3, 4, 5])\n"), vec![5]);
    // Tabelle mit Kommazahlen: jedes Element wird einzeln gekürzt und gemeldet
    assert_eq!(warns(".org $1000\n .byte [1.5, 2]\n"), vec!["W202"]);
    assert!(warns(".org $1000\n .byte [int(1.5), 2]\n").is_empty());
    // Fehler: Liste als Zahl, Index außerhalb, Rechnen mit Listen, Index auf Zahl, len von Zahl
    assert_eq!(code(".org $1000\n.var t = [1]\n lda #t\n"), "E202");
    assert_eq!(code(".org $1000\n.var t = [1]\n .byte t[1]\n"), "E202");
    assert_eq!(code(".org $1000\n.var t = [1]\n .byte t - 1\n"), "E202");
    assert_eq!(code(".org $1000\n .byte 5[0]\n"), "E202");
    assert_eq!(code(".org $1000\n .byte len(5)\n"), "E202");
    assert_eq!(code(".org $1000\n.for c in 5\n.next\n"), "E202");
    assert_eq!(code(".org $1000\n .byte range(100000)\n"), "E202");
}

#[test]
fn long_branches() {
    let far = format!(".org $1000\n .branchlong auto\n beq far\n .fill 200, $ea\nfar: nop\n");
    let out = assemble(&far, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    let b = &out.segments[0].bytes;
    assert_eq!(&b[..5], &[0xD0, 0x03, 0x4C, 0xCD, 0x10]);   // BNE *+5 ; JMP far
    assert_eq!(b.len(), 5 + 200 + 1);
    assert!(out.warnings.iter().any(|w| w.code == "W203"));
    // rückwärts; gleiche Länge der Folgeadressen (Label hinter dem Sprung stimmt)
    let src = ".org $1000\n .branchlong auto\nback: nop\n .fill 200, $ea\n bne back\nend: nop\n";
    let out = assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    let b = &out.segments[0].bytes;
    assert_eq!(&b[201..206], &[0xF0, 0x03, 0x4C, 0x00, 0x10]);   // BEQ *+5 ; JMP back
    assert_eq!(out.symbols["end"], 0x1000 + 201 + 5);
    // Sprung in Reichweite bleibt kurz (kein W203)
    let src = ".org $1000\n .branchlong auto\n bne x\n nop\nx: nop\n";
    let out = assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0xD0, 0x01, 0xEA, 0xEA]);
    assert!(out.warnings.is_empty());
    // mehrere weite Sprünge verschieben sich gegenseitig, bis alles stimmt
    let src = ".org $1000\n .branchlong auto\n beq la\n bne lb\n .fill 126, $ea\nla: .fill 4, $ea\nlb: nop\n";
    let out = assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap();
    let sym = |n: &str| out.symbols[n];
    let b = &out.segments[0].bytes;
    assert_eq!(b[0], 0xD0);                                          // BNE *+5 ; JMP a
    assert_eq!(b[3] as i64 | (b[4] as i64) << 8, sym("la"), "JMP-Ziel von beq a");
    assert_eq!(b[5], 0xF0);                                          // BEQ *+5 ; JMP b
    assert_eq!(b[8] as i64 | (b[9] as i64) << 8, sym("lb"), "JMP-Ziel von bne b");
    // ohne Option: weiter der Fehler
    assert_eq!(code(".org $1000\n beq far\n .fill 200, $ea\nfar: nop\n"), "E203");
    assert_eq!(code(".org $1000\n .branchlong auto\n .branchlong off\n beq far\n .fill 200, $ea\nfar: nop\n"), "E203");
    assert_eq!(code(".org $1000\n .branchlong vielleicht\n"), "E208");
    // Options::branch_long als Voreinstellung
    let mut o = Options::new("t.asm", Policy::Allow); o.branch_long = true;
    assert!(assemble(".org $1000\n beq far\n .fill 200, $ea\nfar: nop\n", &cpu(), &o).is_ok());
    // 45GS02: eingebaute lange Branches (3 Bytes)
    let c = vc_target::load_cpu(&data(), "45gs02").unwrap();
    let out = assemble(&far, &c, &Options::new("t.asm", Policy::Allow)).unwrap();
    assert_eq!(out.segments[0].bytes.len(), 3 + 200 + 1);
}

#[test]
fn fill_with_expression_and_lohifill() {
    assert_eq!(asm(".org $1000\n .fill 3, $ea\n"), vec![0xEA; 3]);
    assert_eq!(asm(".org $1000\n .fill 4, i * 2\n"), vec![0, 2, 4, 6]);
    assert_eq!(asm(".org $1000\n .fill 3, k + 10, k\n"), vec![10, 11, 12]);
    assert_eq!(asm(".org $1000\n .fill 3\n"), vec![0, 0, 0]);
    // Laufvariable ist nur im Ausdruck sichtbar
    assert_eq!(code(".org $1000\n .fill 2, i\n .byte i\n"), "E202");
    // Zeilenadressen: erst Low-, dann High-Bytes
    assert_eq!(asm(".org $1000\n .lohifill 3, $0400 + 40 * i\n"), vec![0x00, 0x28, 0x50, 0x04, 0x04, 0x04]);
    assert_eq!(code(".org $1000\n .lohifill 2, 70000 + i\n"), "E203");
    assert_eq!(code(".org $1000\n .lohifill 2\n"), "E208");
}

#[test]
fn structs_and_enums() {
    let src = ".struct Sprite\n x .byte\n y .byte\n ptr .word\n anim .byte 4\n pos .dword\n.endstruct\n.org $1000\n .byte Sprite.x, Sprite.y, Sprite.ptr, Sprite.anim, Sprite.pos, Sprite.size\n";
    assert_eq!(asm(src), vec![0, 1, 2, 4, 8, 12]);
    // Kommentare und Leerzeilen im Rumpf; Anzahl als Ausdruck; Instanzen mit .fill reservieren
    let src = "N = 3\n.struct S ; Kopf\n\n a .word N ; drei Wörter\n b .byte\n.endstruct\n.org $1000\nobj: .fill S.size, 0\n lda obj + S.b\n";
    assert_eq!(asm(src), vec![0, 0, 0, 0, 0, 0, 0, 0xAD, 0x06, 0x10]);
    // verschachtelt: Offsets der Unterfelder und Feldfelder (Array von Strukturen)
    let src = ".struct Pt\n x .byte\n y .byte\n.endstruct\n.struct Obj\n id .byte\n pos Pt\n trail Pt 3\n.endstruct\n.org $1000\n .byte Obj.pos, Obj.pos.x, Obj.pos.y, Obj.trail, Obj.trail.y, Obj.size, Pt.size\n";
    assert_eq!(asm(src), vec![1, 1, 2, 3, 4, 9, 2]);
    // Vorwärtsbenutzung (vor der Definition) funktioniert wie bei Konstanten
    assert_eq!(asm(".org $1000\n .byte T.size\n.struct T\n a .word\n.endstruct\n"), vec![2]);
    // Aufzählung
    let src = ".enum Color\n black\n white\n red = 5\n green, blue\n.endenum\n.org $1000\n .byte Color.black, Color.white, Color.red, Color.green, Color.blue\n";
    assert_eq!(asm(src), vec![0, 1, 5, 6, 7]);
    // Fehler
    assert_eq!(code(".struct S\n a .float\n.endstruct\n"), "E208");
    assert_eq!(code(".struct S\n a .byte\n a .byte\n.endstruct\n"), "E204");
    assert_eq!(code(".struct S\n.endstruct\n.struct S\n.endstruct\n"), "E204");
    assert_eq!(code(".struct S\n a .byte\n"), "E210");
    assert_eq!(code(".enum E\n a\n"), "E210");
    assert_eq!(code(".struct S\n a\n.endstruct\n"), "E208");
    assert_eq!(code(".enum E\n x\n.endenum\n.struct S\n a E\n.endstruct\n"), "E208");
    assert_eq!(code(".endstruct\n"), "E210");
}

#[test]
fn struct_fields_accept_aligned_columns() {
    // Spalten mit mehreren Leerzeichen und Tabulatoren (wie in der Hilfe)
    let src = ".struct Sprite\n x      .byte\n y\t\t.byte\n ptr    .word\n anim   .byte   4\n.endstruct\n.org $1000\n .byte Sprite.x, Sprite.y, Sprite.ptr, Sprite.anim, Sprite.size\n";
    assert_eq!(asm(src), vec![0, 1, 2, 4, 8]);
    assert_eq!(asm(".struct P\n a .byte\n.endstruct\n.struct O\n  p      P      2\n.endstruct\n.org $1000\n .byte O.size\n"), vec![2]);
}

#[test]
fn special_characters_in_strings() {
    // PETSCII: gewöhnliche Zeichen werden umgewandelt, {…} ist das fertige Byte
    assert_eq!(asm(".org $1000\n .petscii \"hi{clr}{down*2}{$c1}{147}\"\n"), vec![0x48, 0x49, 0x93, 0x11, 0x11, 0xC1, 0x93]);
    // .text folgt der Kodierung; bei ascii bleiben Klammern gewöhnliche Zeichen
    assert_eq!(asm(".org $1000\n .encoding \"petscii\"\n .text \"a{rvs on}b\"\n"), vec![0x41, 0x12, 0x42]);
    assert_eq!(asm(".org $1000\n .text \"a{down}\"\n"), b"a{down}".to_vec());
    assert_eq!(asm(".org $1000\n .encoding \"petscii\"\n .null \"{{x}\"\n"), vec![b'{', 0x58, b'}', 0]);
    // Atari und Bildschirmcode
    assert_eq!(asm(".org $1000\n .atascii \"a{clr}{eol}\"\n"), vec![0x61, 0x7D, 0x9B]);
    assert_eq!(asm(".org $1000\n .screen \"a{$41}\"\n"), vec![0x01, 0x41]);
    assert_eq!(asm(".org $1000\n .encoding \"screen\"\n .text \"a{$20}\"\n"), vec![0x01, 0x20]);
    // Fehler: unbekannter Name, offene Klammer, Name im Bildschirmcode, Steuerzeichen des falschen Systems
    assert_eq!(code(".org $1000\n .petscii \"{nix}\"\n"), "E208");
    assert_eq!(code(".org $1000\n .petscii \"{down\"\n"), "E208");
    assert_eq!(code(".org $1000\n .screen \"{down}\"\n"), "E208");
    assert_eq!(code(".org $1000\n .atascii \"{wht}\"\n"), "E208");
    assert_eq!(code(".org $1000\n .petscii \"{$100}\"\n"), "E208");
}

#[test]
fn basicstub_zeile_und_rem() {
    // 1000 SYS2068:REM TEST – Zeile $03E8, REM-Token $8F, Text als PETSCII-Großbuchstaben, Programm beginnt hinter dem Stub ($0814 = 2068)
    let b = asm(".org $0801\n.basicstub start, 1000, \"test\"\nstart: rts\n");
    let mut want = vec![0x12, 0x08, 0xE8, 0x03, 0x9E];
    want.extend(b"2068");
    want.extend([b':', 0x8F, b' ', b'T', b'E', b'S', b'T', 0x00, 0x00, 0x00, 0x60]);
    assert_eq!(b, want);
    // nur Zeilennummer, nur Kommentar, Standard
    assert_eq!(asm(".org $0801\n.basicstub start, 5\nstart: rts\n")[2..4], [5, 0]);
    let c = asm(".org $0801\n.basicstub start, \"x\"\nstart: rts\n");
    assert_eq!(c[2..4], [10, 0]); assert!(c.windows(3).any(|w| w == [0x8F, b' ', b'X']));
    let d = asm(".org $0801\n.basicstub start\nstart: rts\n");
    assert_eq!(&d[4..], &[&[0x9E][..], b"2061", &[0, 0, 0, 0x60]].concat()[..]);
    // Zeile außerhalb des Bereichs und Sonderzeichen im Kommentar
    assert_eq!(code(".org $0801\n.basicstub start, 64000\nstart: rts\n"), "E203");
    assert_eq!(code(&format!(".org $0801\n.basicstub start, 10, \"{}\"\nstart: rts\n", "x".repeat(200))), "E203");
    let e = asm(".org $0801\n.basicstub start, 10, \"a{$41}\"\nstart: rts\n");
    assert!(e.windows(2).any(|w| w == [b'A', 0x41]));
}

// ---- .basicstub ohne .org: Beginn an der BASIC-Startadresse des Zielsystems
fn stub_prg(target: &str, memory: Option<&str>, src: &str) -> Result<(Vec<u8>, Vec<String>), Vec<String>> {
    let mut t = vc_target::load_target(&data(), target).unwrap();
    t.apply_memory(memory).unwrap();
    let cpu = load_cpu(&data(), &t.cpu).unwrap();
    let mut o = Options::new("t.asm", Policy::Allow);
    o.basic_start = t.basic_load_addr;
    match assemble(src, &cpu, &o) {
        Ok(out) => Ok((vc_asm::to_prg(&out).unwrap(), out.warnings.iter().map(|w| w.code.to_string()).collect())),
        Err(e) => Err(e.iter().map(|d| d.code.to_string()).collect()),
    }
}

#[test]
fn basicstub_ohne_org_nutzt_basic_start_des_ziels() {
    let src = ".basicstub start\nstart: rts\n";
    // (Ziel, Speicherausbau, Ladeadresse, SYS-Ziel)
    for (t, m, load, sys) in [("c64", None, 0x0801u16, "2061"), ("c128", None, 0x1C01, "7181"), ("plus4", None, 0x1001, "4109"),
                              ("mega65", None, 0x2001, "8205"), ("vic20", None, 0x1001, "4109"), ("vic20", Some("8k"), 0x1201, "4621"), ("vic20", Some("3k"), 0x0401, "1037")] {
        let (prg, w) = stub_prg(t, m, src).unwrap_or_else(|e| panic!("{t}: {e:?}"));
        assert_eq!(&prg[..2], &load.to_le_bytes(), "{t} {m:?} Ladeadresse");
        assert_eq!(&prg[2..4], &(load + 10).to_le_bytes(), "{t} {m:?} Zeiger auf die nächste Zeile");
        assert_eq!(&prg[7..7 + sys.len()], sys.as_bytes(), "{t} {m:?} SYS-Ziel");
        assert!(w.is_empty(), "{t}: {w:?}");
    }
    // ohne Zieldaten (Options::new): Rückfall $0801
    let prg = vc_asm::to_prg(&assemble(src, &cpu(), &Options::new("t.asm", Policy::Allow)).unwrap()).unwrap();
    assert_eq!(&prg[..2], &[0x01, 0x08]);
}

#[test]
fn basicstub_mit_org_warnt_bei_falscher_adresse() {
    // passende Adresse: still; falsche: W204; Datei bleibt trotzdem gleich wie bisher
    assert_eq!(stub_prg("c128", None, ".org $1c01\n.basicstub start\nstart: rts\n").unwrap().1, Vec::<String>::new());
    let (prg, w) = stub_prg("c128", None, ".org $0801\n.basicstub start\nstart: rts\n").unwrap();
    assert_eq!(w, vec!["W204".to_string()]);
    assert_eq!(&prg[..2], &[0x01, 0x08]);
    // Ausnahmen: Modul und Pseudo-PC
    assert!(stub_prg("c64", None, ".cartridge \"normal\"\n.org $8000\n.basicstub start\nstart: rts\n").unwrap().1.is_empty());
    assert!(stub_prg("c64", None, ".org $c000\n.pseudopc $0801\n.basicstub start\nstart: rts\n").unwrap().1.is_empty());
}

#[test]
fn basicstub_nach_ausgabe_ist_fehler() {
    assert_eq!(stub_prg("c64", None, ".org $0801\n nop\n.basicstub start\nstart: rts\n").unwrap_err(), vec!["E218".to_string()]);
    assert_eq!(stub_prg("c64", None, " nop\n.basicstub start\nstart: rts\n").unwrap_err()[0], "E300");
}
