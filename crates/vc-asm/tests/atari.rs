// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_asm::{assemble, to_xex, Options, Policy};
use vc_target::{generate_include, load_chip, load_cpu, load_target};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }
fn data() -> PathBuf { root().join("data") }

#[test]
fn atari_target_and_cpu_load() {
    let t = load_target(&data(), "atari8").unwrap();
    assert_eq!((t.cpu.as_str(), t.default_format.as_str()), ("sally", "xex"));
    assert_eq!(t.chips, ["antic", "gtia", "pokey", "pia", "os"]);
    let sally = load_cpu(&data(), "sally").unwrap();
    assert_eq!(sally.ops.len(), 256);
    assert!(sally.has_group("illegal.stable") && sally.features.contains(&"halt".to_string()));
    assert_eq!(load_target(&data(), "c64").unwrap().default_format, "prg");
}

#[test]
fn chip_register_addresses() {
    let find = |chip: &str, name: &str| load_chip(&data(), chip).unwrap().regs.into_iter().find(|r| r.name == name).unwrap().addr;
    assert_eq!(find("gtia", "colbk"), 0xD01A);
    assert_eq!(find("antic", "wsync"), 0xD40A);
    assert_eq!(find("antic", "vcount"), 0xD40B);
    assert_eq!(find("pokey", "audctl"), 0xD208);
    assert_eq!(find("pia", "portb"), 0xD301);
    assert_eq!(find("vic2", "border"), 0xD020);
    assert_eq!(find("sid", "mode_vol"), 0xD418);
    assert_eq!(find("cia2", "ddra"), 0xDD02);
}

#[test]
fn chip_files_have_no_duplicate_names() {
    for target in ["atari8", "c64"] {
        let t = load_target(&data(), target).unwrap();
        for c in &t.chips {
            let chip = load_chip(&data(), c).unwrap();
            let mut names: Vec<_> = chip.regs.iter().map(|r| r.name.clone()).collect();
            names.sort();
            let n = names.len();
            names.dedup();
            assert_eq!(names.len(), n, "{c}: doppelte Registernamen");
        }
    }
}

#[test]
fn committed_includes_are_up_to_date() {
    for (target, file) in [("atari8", "examples/atari/regs_test/lib/atari.inc"), ("c64", "examples/c64/lib/c64.inc")] {
        let fresh = generate_include(&data(), &load_target(&data(), target).unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(root().join(file)).unwrap(), fresh, "{file} veraltet: `vcc inc --target {target} -o {file}`");
    }
}

#[test]
fn colorbars_example_assembles_to_known_xex() {
    let path = root().join("examples/atari/colorbars/colorbars.asm");
    let src = std::fs::read_to_string(&path).unwrap();
    let cpu = load_cpu(&data(), "sally").unwrap();
    let mut o = Options::new(path.to_string_lossy(), Policy::Deny);
    o.predefined = vc_target::chip_symbols(&data(), &load_target(&data(), "atari8").unwrap()).unwrap();
    let out = assemble(&src, &cpu, &o).unwrap();
    assert_eq!(to_xex(&out).unwrap(), vec![0xFF, 0xFF, 0x00, 0x20, 0x10, 0x20, 0xA9, 0x00, 0x8D, 0x0E, 0xD4, 0xAE, 0x0B, 0xD4, 0x8E, 0x0A, 0xD4,
                                          0x8E, 0x1A, 0xD0, 0x4C, 0x05, 0x20, 0xE0, 0x02, 0xE1, 0x02, 0x00, 0x20]);
}

#[test]
fn chip_registers_are_predefined_and_not_exported() {
    let t = load_target(&data(), "atari8").unwrap();
    let cpu = load_cpu(&data(), "sally").unwrap();
    let mut o = Options::new("t.asm", Policy::Allow);
    o.predefined = vc_target::chip_symbols(&data(), &t).unwrap();
    let out = assemble(".org $2000\nstart: sta gtia.colbk\n sta antic.wsync\n", &cpu, &o).unwrap();
    assert_eq!(out.segments[0].bytes, vec![0x8D, 0x1A, 0xD0, 0x8D, 0x0A, 0xD4]);
    assert!(out.symbols.keys().all(|k| !k.contains("gtia.")), "{:?}", out.symbols.keys().collect::<Vec<_>>());
    // Zusätzliches Einbinden der erzeugten Include-Datei bleibt erlaubt (gleiche Werte)
    let inc = generate_include(&data(), &t).unwrap();
    assert!(assemble(&format!(".org $2000\n{inc}\n sta gtia.colbk\n"), &cpu, &o).is_ok());
}
