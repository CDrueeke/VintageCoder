// SPDX-License-Identifier: GPL-3.0-or-later
//! Medien als Include (Konzept „Include von Medien“, Teil B): Provider-Hook, virtuelle Datei, E219, W208, `#include` einmalig, `as`.
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use vc_asm::{assemble, Options, Policy};
use vc_target::{load_cpu, Cpu};

fn data() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") }
fn cpu() -> Cpu { load_cpu(&data(), "6510").unwrap() }
fn dir(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("media_inc_{tag}"));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(d.join("graphics")).unwrap(); d
}
/// Provider, der `*.vcsprite` als Text mit Marke `hero` liefert und `*.vcbad` ablehnt; zählt die Aufrufe
fn provider(calls: Arc<AtomicUsize>) -> vc_asm::IncludeProvider {
    Arc::new(move |p: &Path| {
        let n = p.to_string_lossy();
        if n.ends_with(".vcsprite") { calls.fetch_add(1, Ordering::SeqCst); Some(Ok("hero: .byte 1, 2, 3\n.const hero_count = 3\n".to_string())) }
        else if n.ends_with(".vcbad") { Some(Err("Medium graphics/x.vcbad ist für atari8, das Ziel des Baus ist c64".to_string())) }
        else { None }
    })
}
fn opts(d: &Path, calls: Arc<AtomicUsize>) -> Options {
    let mut o = Options::new(d.join("main.asm").to_string_lossy(), Policy::Allow);
    o.include_provider = Some(provider(calls)); o
}

#[test]
fn medium_wird_ueber_den_provider_eingebunden() {
    let d = dir("ok");
    std::fs::write(d.join("graphics/hero.vcsprite"), "{}").unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let src = ".org $1000\n lda #hero_count\n ldx hero\nstart: rts\n.include \"graphics/hero.vcsprite\"\n";
    let out = assemble(src, &cpu(), &opts(&d, calls.clone())).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(&out.segments[0].bytes[..4], &[0xA9, 3, 0xAE, 0x06], "hero liegt hinter rts: $1006");
    assert_eq!(&out.segments[0].bytes[6..], &[1, 2, 3]);
    // das Medium ist eine virtuelle Datei: Quelle, Listing und Zeilenzuordnung nennen den Medienpfad
    assert!(out.sources.iter().any(|(f, l)| f.ends_with("graphics/hero.vcsprite") && l.iter().any(|x| x.starts_with("hero:"))), "{:?}", out.sources.iter().map(|s| &s.0).collect::<Vec<_>>());
    assert!(out.lines.iter().any(|l| l.file.ends_with("graphics/hero.vcsprite") && l.line >= 1));
    assert!(out.defs.get("hero").map_or(false, |(f, _)| f.ends_with("graphics/hero.vcsprite")), "Definition zeigt auf das Medium");
}

#[test]
fn ohne_provider_ist_das_medium_eine_gewoehnliche_datei() {
    let d = dir("ohne");
    std::fs::write(d.join("graphics/hero.vcsprite"), "hero: .byte 9\n").unwrap();
    let o = Options::new(d.join("main.asm").to_string_lossy(), Policy::Allow);
    let out = assemble(".org $1000\n.include \"graphics/hero.vcsprite\"\n", &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![9]);
}

#[test]
fn medium_nicht_exportierbar_ist_e219_in_der_include_zeile() {
    let d = dir("e219");
    std::fs::write(d.join("graphics/x.vcbad"), "{}").unwrap();
    let e = assemble(".org $1000\n nop\n.include \"graphics/x.vcbad\"\n", &cpu(), &opts(&d, Arc::new(AtomicUsize::new(0)))).unwrap_err();
    assert_eq!(e[0].code, "E219");
    assert_eq!(e[0].line, 3);
    assert!(e[0].message.contains("atari8"), "{}", e[0].message);
}

#[test]
fn fehlendes_medium_bleibt_e212() {
    let d = dir("e212");
    let e = assemble(".org $1000\n.include \"graphics/fehlt.vcsprite\"\n", &cpu(), &opts(&d, Arc::new(AtomicUsize::new(0)))).unwrap_err();
    assert_eq!(e[0].code, "E212");
}

#[test]
fn ungespeicherter_editortext_hat_vorrang() {
    let d = dir("override");
    std::fs::write(d.join("graphics/hero.vcsprite"), "{}").unwrap();
    let mut o = opts(&d, Arc::new(AtomicUsize::new(0)));
    o.overrides.insert(vc_asm::norm_path(&d.join("graphics/hero.vcsprite").to_string_lossy()), "hero: .byte 7\n".into());
    let out = assemble(".org $1000\n.include \"graphics/hero.vcsprite\"\n", &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![7]);
}

#[test]
fn hash_include_bindet_ein_medium_nur_einmal_ein() {
    let d = dir("once");
    std::fs::write(d.join("graphics/hero.vcsprite"), "{}").unwrap();
    let src = ".org $1000\n#include \"graphics/hero.vcsprite\"\n#include \"graphics/hero.vcsprite\"\n";
    let calls = Arc::new(AtomicUsize::new(0));
    let out = assemble(src, &cpu(), &opts(&d, calls.clone())).unwrap();
    assert_eq!(out.segments[0].bytes, vec![1, 2, 3], "zweites #include überspringt");
    assert_eq!(calls.load(Ordering::SeqCst), 1, "der Provider wird nur einmal gefragt");
    // .include darf mehrfach, auch mit eigenem Namensraum
    let src = ".org $1000\n.include \"graphics/hero.vcsprite\" as a\n.include \"graphics/hero.vcsprite\" as b\n ldx a.hero\n ldy b.hero\n";
    let out = assemble(src, &cpu(), &opts(&d, Arc::new(AtomicUsize::new(0)))).unwrap();
    assert_eq!(out.segments[0].bytes, vec![1, 2, 3, 1, 2, 3, 0xAE, 0x00, 0x10, 0xAC, 0x03, 0x10], "a.hero = $1000, b.hero = $1003");
}

#[test]
fn alter_exportpfad_gibt_hinweis_w208_mit_medium() {
    let d = dir("w208");
    std::fs::create_dir_all(d.join("build/export")).unwrap();
    std::fs::write(d.join("build/export/hero.asm"), "hero: .byte 5\n").unwrap();
    let mut o = opts(&d, Arc::new(AtomicUsize::new(0)));
    o.legacy_hint = Some(Arc::new(|p: &Path| p.to_string_lossy().contains("build/export/hero.asm").then(|| "graphics/hero.vcsprite".to_string())));
    let out = assemble(".org $1000\n nop\n.include \"build/export/hero.asm\"\n", &cpu(), &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0xEA, 5], "der alte Pfad funktioniert weiter");
    let w: Vec<_> = out.warnings.iter().filter(|w| w.code == "W208").collect();
    assert_eq!(w.len(), 1, "{:?}", out.warnings);
    assert_eq!(w[0].line, 3);
    assert!(w[0].message.contains("graphics/hero.vcsprite") && w[0].message.contains("build/export/hero.asm"), "{}", w[0].message);
    // ohne Hinweisgeber keine Warnung
    let o2 = opts(&d, Arc::new(AtomicUsize::new(0)));
    let out = assemble(".org $1000\n.include \"build/export/hero.asm\"\n", &cpu(), &o2).unwrap();
    assert!(out.warnings.iter().all(|w| w.code != "W208"));
}
