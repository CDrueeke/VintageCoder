// SPDX-License-Identifier: GPL-3.0-or-later
use vc_ide::diskimage::{is_image_name, Image};

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("vc_diskimage_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d); std::fs::create_dir_all(&d).unwrap(); d
}

#[test]
fn d64_verzeichnis_und_herausloesen() {
    let mut d = vc_disk::D64::blank("TESTDISK", "TD").unwrap();
    let prg = [0x01, 0x08, 0xA9, 0x00, 0x60];
    d.add_prg("HALLO WELT", &prg).unwrap();
    d.add_prg("MUSIK", &[0x00, 0xC0, 1, 2, 3]).unwrap();
    let img = Image::from_bytes("x.d64", &d.to_bytes()).unwrap();
    assert_eq!(img.format, "D64"); assert_eq!(img.title, "TESTDISK"); assert!(!img.atari);
    assert_eq!(img.entries.len(), 2);
    assert_eq!(img.entries[0].kind, "PRG"); assert_eq!(img.entries[0].load_address(), Some(0x0801));
    assert_eq!(img.entries[1].load_address(), Some(0xC000));
    assert_eq!(img.suggested_name(&img.entries[0]), "hallo_welt.prg");
    let root = tmp("d64");
    assert_eq!(img.extract(0, &root, "import", "hallo_welt.prg").unwrap(), "import/hallo_welt.prg");
    assert_eq!(std::fs::read(root.join("import/hallo_welt.prg")).unwrap(), prg);
    // zweites Mal: Name wird eindeutig gemacht, nichts überschrieben
    assert_eq!(img.extract(0, &root, "import", "hallo_welt.prg").unwrap(), "import/hallo_welt_2.prg");
    assert_eq!(img.extract(0, &root, "", "a.prg").unwrap(), "a.prg");
    assert!(img.extract(0, &root, "../x", "a.prg").is_err());
    assert!(img.extract(0, &root, "", "../a.prg").is_err());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn d71_d81_und_atr() {
    for (ext, kind) in [("d71", vc_disk::Kind::D71), ("d81", vc_disk::Kind::D81)] {
        let mut d = vc_disk::Disk::blank(kind, "DISK", "01").unwrap();
        d.add_prg("PROG", &[0x00, 0x20, 0xEA]).unwrap();
        let img = Image::from_bytes(&format!("x.{ext}"), &d.to_bytes()).unwrap();
        assert_eq!(img.entries.len(), 1); assert_eq!(img.entries[0].data.as_ref().unwrap(), &vec![0x00, 0x20, 0xEA]);
    }
    let mut a = vc_disk::Atr::new_dos2();
    let xex = [0xFF, 0xFF, 0x00, 0x20, 0x02, 0x20, 0xEA, 0xEA, 0xEA];
    a.dos2_add("SPIEL.XEX", &xex).unwrap();
    a.dos2_add("LIES.TXT", b"hallo").unwrap();
    let img = Image::from_bytes("x.atr", &a.to_bytes()).unwrap();
    assert!(img.atari); assert_eq!(img.entries.len(), 2);
    assert_eq!(img.entries[0].load_address(), Some(0x2000)); assert_eq!(img.entries[1].load_address(), None);
    assert_eq!(img.suggested_name(&img.entries[0]), "spiel.xex");
    let root = tmp("atr");
    img.extract(1, &root, "", "lies.txt").unwrap();
    assert_eq!(std::fs::read(root.join("lies.txt")).unwrap(), b"hallo");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn erkennung_und_fehler() {
    assert!(is_image_name("A.D64") && is_image_name("b.atr") && !is_image_name("c.prg"));
    assert!(Image::from_bytes("x.bin", &[0; 10]).is_err());
    assert!(Image::from_bytes("x.d64", &[0; 10]).is_err());
}

#[test]
fn xex_bloecke() {
    let x = [0xFF, 0xFF, 0x00, 0x20, 0x01, 0x20, 1, 2, 0x10, 0x30, 0x10, 0x30, 9];
    let b = vc_disk::xex_blocks(&x).unwrap();
    assert_eq!(b, vec![(0x2000, &[1u8, 2][..]), (0x3010, &[9u8][..])]);
    assert!(vc_disk::xex_blocks(&x[..9]).is_err());
}

fn data_dir() -> std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data") }

#[test]
fn basic_mit_maschinencode_anhang() {
    use vc_ide::diskimage::{detect_basic, import_data, ImportOpts};
    // 10 SYS2061 + 3 Bytes Maschinencode dahinter
    let prg = [0x01, 0x08, 0x0B, 0x08, 0x0A, 0x00, 0x9E, b'2', b'0', b'6', b'1', 0x00, 0x00, 0x00, 0xA9, 0x00, 0x60];
    let info = detect_basic(&data_dir(), &prg, false).expect("BASIC erkannt");
    assert_eq!((info.dialect.as_str(), info.system, info.lines, info.tail), ("cbm_v2", "c64", 1, 3));
    let root = tmp("basic");
    let o = import_data(&root, "stub.prg", &prg, true, false, Some(&data_dir()), &ImportOpts::default()).unwrap();
    assert_eq!(o.files, vec!["import/stub.bas", "import/stub.prg"]);   // Original bleibt wegen des Anhangs
    assert!(std::fs::read_to_string(root.join("import/stub.bas")).unwrap().starts_with("10 sys"));
    // ohne BASIC-Erkennung nur kopieren
    let o = import_data(&root, "stub.prg", &prg, true, false, Some(&data_dir()), &ImportOpts { basic: false, dir: "roh".into(), ..Default::default() }).unwrap();
    assert_eq!(o.files, vec!["roh/stub.prg"]);
    // Maschinenprogramm ist kein BASIC
    assert!(detect_basic(&data_dir(), &[0x00, 0xC0, 0xEE, 0x20, 0xD0, 0x60], false).is_none());
    assert!(detect_basic(&data_dir(), &[0xFF, 0xFF, 0x00, 0x20, 0x01, 0x20, 1, 2], true).is_none());
    let _ = std::fs::remove_dir_all(&root);
}
