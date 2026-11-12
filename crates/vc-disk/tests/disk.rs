// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use vc_disk::{boot_disk, xex_segments, Atr};

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }

#[test]
fn atr_header_and_blank_dos2_disk() {
    let a = Atr::new_dos2();
    let b = a.to_bytes();
    assert_eq!(&b[..7], &[0x96, 0x02, 0x80, 0x16, 0x80, 0x00, 0x00]);
    assert_eq!(b.len(), 16 + 720 * 128);
    assert_eq!(Atr::from_bytes(&b).unwrap(), a);
    let v = a.sector(360);
    assert_eq!((v[0], v[1], v[2], v[3], v[4]), (2, 0xC3, 0x02, 0xC3, 0x02));         // 707 Sektoren
    assert_eq!(&v[10..12], &[0x0F, 0xFF], "Sektoren 0-3 belegt, 4-7 frei");
    assert_eq!((v[10 + 45], v[10 + 46]), (0x00, 0x7F), "360-368 belegt, 369-375 frei");
    assert!(a.dos2_check().is_empty() && a.dos2_entries().is_empty());
    assert!(Atr::from_bytes(&b[..100]).is_err() && Atr::from_bytes(&[0; 20]).is_err());
}

#[test]
fn files_roundtrip_with_correct_sector_chains() {
    let mut a = Atr::new_dos2();
    let big: Vec<u8> = (0..1000u32).map(|i| (i * 7 % 256) as u8).collect();
    a.dos2_add("DATA.BIN", &big).unwrap();
    a.dos2_add("HELLO.TXT", b"hallo").unwrap();
    assert_eq!(a.dos2_read("DATA.BIN").unwrap(), big);
    assert_eq!(a.dos2_read("HELLO.TXT").unwrap(), b"hallo");
    assert_eq!(a.free_sectors(), 707 - 8 - 1);
    let e = a.dos2_entries();
    assert_eq!((e[0].name.as_str(), e[0].sectors, e[0].start), ("DATA.BIN", 8, 4));
    assert_eq!((e[1].name.as_str(), e[1].sectors, e[1].start), ("HELLO.TXT", 1, 12));
    // Rohstruktur: Verzeichniseintrag und erster Sektor
    let d = &a.sector(361)[..16];
    assert_eq!(d, &[0x42, 8, 0, 4, 0, b'D', b'A', b'T', b'A', b' ', b' ', b' ', b' ', b'B', b'I', b'N']);
    let s4 = a.sector(4);
    assert_eq!((s4[125], s4[126], s4[127]), (0 << 2, 5, 125));            // Datei 0, Folgesektor 5, 125 Bytes
    let s11 = a.sector(11);
    assert_eq!((s11[125], s11[126], s11[127]), (0, 0, (1000 - 7 * 125) as u8));   // letzter Sektor: Folgesektor 0
    assert_eq!(a.sector(12)[125] >> 2, 1, "zweite Datei hat Dateinummer 1");
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    let b = Atr::from_bytes(&a.to_bytes()).unwrap();
    assert_eq!(b.dos2_read("DATA.BIN").unwrap(), big);
}

#[test]
fn large_file_skips_the_directory_area() {
    let mut a = Atr::new_dos2();
    let data: Vec<u8> = (0..357 * 125).map(|i| (i % 251) as u8).collect();      // 357 Sektoren: 4..359 (356) + 369
    a.dos2_add("BIG.DAT", &data).unwrap();
    assert_eq!(a.dos2_read("BIG.DAT").unwrap(), data);
    assert_eq!(a.sector(359)[126] as usize | (a.sector(359)[125] as usize & 3) << 8, 369);
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
}

#[test]
fn limits_and_errors() {
    let mut a = Atr::new_dos2();
    assert!(a.dos2_add("lower.txt", b"x").is_err() && a.dos2_add("1ABC", b"x").is_err() && a.dos2_add("TOOLONGNAME.X", b"x").is_err() && a.dos2_add("A.LONG", b"x").is_err());
    assert!(a.dos2_add("EMPTY", b"").is_err());
    a.dos2_add("A", b"x").unwrap();
    assert!(a.dos2_add("A", b"y").unwrap_err().contains("existiert"));
    assert!(a.dos2_add("HUGE", &vec![0; 707 * 125]).unwrap_err().contains("voll"));
    let mut full = Atr::new_dos2();
    for i in 0..64 { full.dos2_add(&format!("F{i}"), b"x").unwrap(); }
    assert!(full.dos2_add("F64", b"x").unwrap_err().contains("Verzeichnis voll"));
    assert!(full.dos2_check().is_empty());
    assert!(a.dos2_read("MISSING").is_err());
}

#[test]
fn xex_segments_parse() {
    let x = [0xFF, 0xFF, 0x00, 0x20, 0x01, 0x20, 0xAA, 0xBB, 0xE0, 0x02, 0xE1, 0x02, 0x00, 0x20];
    assert_eq!(xex_segments(&x).unwrap(), vec![(0x2000, 0x2001), (0x02E0, 0x02E1)]);
    assert!(xex_segments(&x[..9]).is_err());
}

#[test]
fn bootable_xex_disk_layout() {
    let xex = std::fs::read(root().join("build/colorbars.xex")).unwrap_or_else(|_| vec![0xFF, 0xFF, 0x00, 0x20, 0x00, 0x20, 0x60, 0xE0, 0x02, 0xE1, 0x02, 0x00, 0x20]);
    let a = boot_disk(&xex, &root().join("data")).unwrap();
    let s1 = a.sector(1);
    assert_eq!(&s1[..6], &[0, 3, 0x00, 0x07, 0x06, 0x07], "Boot-Kopf: Flags, 3 Sektoren, Ladeadresse $0700, DOSINI $0706");
    assert_eq!(&a.sector(4)[..xex.len().min(128)], &xex[..xex.len().min(128)]);
    assert!(a.sector(3).iter().any(|b| *b != 0) || a.sector(2).iter().any(|b| *b != 0), "Loader belegt mehr als einen Sektor");
    // Überlappung mit dem Loader wird abgelehnt
    let bad = [0xFF, 0xFF, 0x00, 0x08, 0x00, 0x08, 0x60];
    assert!(boot_disk(&bad, &root().join("data")).unwrap_err().contains("überlappt"));
    assert!(boot_disk(&[1, 2, 3, 4, 5, 6], &root().join("data")).is_err());
}
