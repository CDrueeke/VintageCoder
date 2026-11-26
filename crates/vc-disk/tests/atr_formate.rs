// SPDX-License-Identifier: GPL-3.0-or-later
//! Atari-Dateisysteme über DOS 2 hinaus: DOS 2.5 (1040 Sektoren), MyDOS (256-Byte-Sektoren, Unterverzeichnisse). Die Abbilder baut der Test selbst
//! (nach der Beschreibung der Formate); echte Abbilder werden zusätzlich geprüft, wenn sie unter `VC_ATR_ECHT=<Ordner>` liegen (MYDOS45D.ATR u. a.).
use vc_disk::{AnyDisk, Atr, Cell, DosKind};

/// Datei in ein Abbild legen: `sectors` sind die Sektoren der Kette, `dir_first`/`slot` der Verzeichnisplatz
fn lege(a: &mut Atr, dir_first: usize, slot: usize, name: &str, sectors: &[usize], data: &[u8]) {
    let ssz = a.sector_size(); let per = ssz - 3;
    for (k, &s) in sectors.iter().enumerate() {
        let chunk = &data[k * per..((k + 1) * per).min(data.len())]; let next = sectors.get(k + 1).copied().unwrap_or(0);
        let sec = a.sector_mut(s); sec.fill(0); sec[..chunk.len()].copy_from_slice(chunk);
        sec[ssz - 3] = ((slot % 64) as u8) << 2 | (next >> 8) as u8; sec[ssz - 2] = (next & 0xFF) as u8; sec[ssz - 1] = chunk.len() as u8;
    }
    let per_dir = ssz / 16;
    let e = &mut a.sector_mut(dir_first + slot / per_dir)[(slot % per_dir) * 16..(slot % per_dir) * 16 + 16];
    e[0] = 0x42; e[1] = (sectors.len() & 0xFF) as u8; e[2] = (sectors.len() >> 8) as u8; e[3] = (sectors[0] & 0xFF) as u8; e[4] = (sectors[0] >> 8) as u8;
    let (b, x) = name.split_once('.').unwrap_or((name, "")); e[5..16].fill(b' '); e[5..5 + b.len()].copy_from_slice(b.as_bytes()); e[13..13 + x.len()].copy_from_slice(x.as_bytes());
}
fn setze(a: &mut Atr, vtoc: usize, byte0: usize, first: usize, s: usize, frei: bool) { let (by, bit) = (byte0 + (s - first) / 8, 7 - (s - first) % 8); let v = a.sector_mut(vtoc); if frei { v[by] |= 1 << bit } else { v[by] &= !(1 << bit) } }

/// MyDOS auf 720 × 256 Byte (Boot-Sektoren wahlweise 128 Byte), Sektor 1 mit der Kennung `M`
fn mydos(boot128: bool) -> Atr {
    let mut a = Atr::blank_with(720, 256, boot128); a.sector_mut(1)[0] = 0x4D;
    a.sector_mut(360)[0] = 2;
    let belegt: Vec<usize> = (1..=3).chain(360..=368).collect();
    for s in 1..=720 { if !belegt.contains(&s) { setze(&mut a, 360, 10, 0, s, true); } }
    let frei = (1..=720).filter(|s| !belegt.contains(s)).count(); let v = a.sector_mut(360); v[1] = (frei & 0xFF) as u8; v[2] = (frei >> 8) as u8; v[3] = v[1]; v[4] = v[2];
    a
}
fn belege(a: &mut Atr, vtoc: usize, byte0: usize, first: usize, secs: &[usize]) { for &s in secs { setze(a, vtoc, byte0, first, s, false); } }
fn zaehler_minus(a: &mut Atr, n: usize) { let v = a.sector_mut(360); let f = (v[3] as usize | (v[4] as usize) << 8) - n; v[3] = (f & 0xFF) as u8; v[4] = (f >> 8) as u8; }

#[test]
fn mydos_256_byte_sektoren_lesen_pruefen_und_rundlauf() {
    for boot128 in [false, true] {
        let mut a = mydos(boot128);
        let daten: Vec<u8> = (0..600u32).map(|i| (i % 251) as u8).collect();
        let secs = [4usize, 5, 6]; lege(&mut a, 361, 0, "HALLO.TXT", &secs, &daten); belege(&mut a, 360, 10, 0, &secs); zaehler_minus(&mut a, 3);
        assert_eq!(a.kind(), DosKind::MyDos);
        assert_eq!(a.dos2_read("HALLO.TXT").unwrap(), daten);
        assert_eq!(a.dos2_check(), Vec::<String>::new());
        let b = a.to_bytes(); let r = Atr::from_bytes(&b).unwrap(); assert_eq!(r.boot_128(), boot128); assert_eq!(r.to_bytes(), b, "Rundlauf byte-gleich");
        let d = AnyDisk::Atr(r); assert_eq!(d.sector_size(), 256); assert_eq!(d.files().len(), 1);
        assert!(d.analyze().problems.is_empty(), "{:?}", d.analyze().problems);
    }
}

#[test]
fn mydos_unterverzeichnis_und_pfad() {
    let mut a = mydos(false);
    // Unterverzeichnis „SPIEL“: acht aufeinander folgende Sektoren ab 100, im Wurzelverzeichnis als Eintrag mit Flag $10
    let sub: Vec<usize> = (100..108).collect();
    { let e = &mut a.sector_mut(361)[0..16]; e[0] = 0x10 | 0x40; e[1] = 8; e[2] = 0; e[3] = 100; e[4] = 0; e[5..16].fill(b' '); e[5..10].copy_from_slice(b"SPIEL"); }
    belege(&mut a, 360, 10, 0, &sub); zaehler_minus(&mut a, 8);
    let daten = vec![0xFFu8, 0xFF, 1, 2, 3, 4, 5, 6, 7, 8]; let secs = [200usize];
    lege(&mut a, 100, 0, "LOS.XEX", &secs, &daten); belege(&mut a, 360, 10, 0, &secs); zaehler_minus(&mut a, 1);
    let e = a.dos2_entries(); assert_eq!(e.iter().map(|x| (x.name.as_str(), x.dir)).collect::<Vec<_>>(), vec![("SPIEL", true), ("SPIEL/LOS.XEX", false)]);
    assert_eq!(a.dos2_read("SPIEL/LOS.XEX").unwrap(), daten);
    assert!(a.dos2_read("SPIEL").is_err(), "ein Verzeichnis ist keine Datei");
    assert_eq!(a.dos2_check(), Vec::<String>::new());
    let d = AnyDisk::Atr(a); let an = d.analyze();
    assert!(an.problems.is_empty(), "{:?}", an.problems);
    assert_eq!(an.cell(AnyDisk::atr_addr(100)), Some(Cell::Dir), "Unterverzeichnis steht in der Karte als Verzeichnis");
    assert!(matches!(an.cell(AnyDisk::atr_addr(200)), Some(Cell::File(_))));
}

/// DOS 2.5: 1040 Sektoren zu 128 Byte, zweite VTOC in Sektor 1024 (Sektoren 720–1023 ab Byte 84)
fn dos25() -> Atr {
    let mut a = Atr::blank_with(1040, 128, false); a.sector_mut(360)[0] = 2;
    let belegt: Vec<usize> = (1..=3).chain(360..=368).chain([1024]).collect();
    for s in 1..=1023 { if !belegt.contains(&s) { if s < 720 { setze(&mut a, 360, 10, 0, s, true); } else { setze(&mut a, 1024, 84, 720, s, true); } } }
    a
}
#[test]
fn dos25_lesen_mit_sektoren_ueber_720() {
    let mut a = dos25(); assert_eq!(a.kind(), DosKind::Dos25);
    let daten: Vec<u8> = (0..300u32).map(|i| (i * 7 % 256) as u8).collect(); let secs = [719usize, 720, 1000];   // Kette über die Grenze 720 und bis 1000
    lege(&mut a, 361, 0, "GROSS.DAT", &secs, &daten);
    for &s in &secs { if s < 720 { setze(&mut a, 360, 10, 0, s, false) } else { setze(&mut a, 1024, 84, 720, s, false) } }
    assert_eq!(a.dos2_read("GROSS.DAT").unwrap(), daten);
    assert_eq!(a.dos2_check(), Vec::<String>::new());
    assert_eq!(a.free_sectors(), 1023 - 3 - 9 - 3, "freie Sektoren aus beiden VTOC gezählt: {}", a.free_sectors());
    assert!(!a.sector_free(1000) && a.sector_free(999) && !a.sector_free(1024) && !a.sector_free(1030));
    let b = a.to_bytes(); assert_eq!(b.len(), 16 + 1040 * 128); assert_eq!(Atr::from_bytes(&b).unwrap().to_bytes(), b);
    let d = AnyDisk::Atr(a); assert_eq!(d.tracks(), 58); assert_eq!(d.sectors_in_track(58), 1040 - 57 * 18);
    assert!(d.analyze().problems.is_empty(), "{:?}", d.analyze().problems);
}

#[test]
fn schreiben_auf_andere_systeme_als_dos2_meldet_klar() {
    let mut a = mydos(false);
    assert!(a.dos2_add("X.BIN", b"x").unwrap_err().contains("MyDOS"));
    let mut b = dos25(); assert!(b.dos2_add("X.BIN", b"x").unwrap_err().contains("2.5"));
}

#[test]
fn fehler_in_der_bitmap_und_ketten_werden_gemeldet() {
    let mut a = mydos(false); let secs = [4usize, 5]; lege(&mut a, 361, 0, "A.TXT", &secs, &vec![1u8; 400]); belege(&mut a, 360, 10, 0, &[4]);   // Sektor 5 bleibt als frei markiert
    zaehler_minus(&mut a, 1);
    let p = a.dos2_check(); assert!(p.iter().any(|m| m.contains('5') && m.contains("frei")), "{p:?}");
}

/// Echte Abbilder, wenn vorhanden (nicht im Repository: MyDOS ist fremde Software)
#[test]
fn echtes_mydos_abbild_wenn_vorhanden() {
    let Ok(dir) = std::env::var("VC_ATR_ECHT") else { return };
    let Ok(b) = std::fs::read(std::path::Path::new(&dir).join("MYDOS45D.ATR")) else { return };
    let a = Atr::from_bytes(&b).unwrap();
    assert_eq!((a.kind(), a.sector_size(), a.sector_count(), a.boot_128()), (DosKind::MyDos, 256, 720, false));
    let namen: Vec<String> = a.dos2_entries().into_iter().map(|e| e.name).collect(); assert!(namen.contains(&"DOS.SYS".to_string()) && namen.contains(&"DUP.SYS".to_string()), "{namen:?}");
    for n in &namen { assert!(a.dos2_read(n).is_ok(), "{n}"); }
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check()); assert_eq!(a.to_bytes(), b);
}

/// Erzeugt die Testabbilder für die Handprüfung (nicht Teil der normalen Läufe): `VC_ATR_OUT=<Ordner> cargo test -p vc-disk --test atr_formate -- --ignored erzeuge`
#[test]
#[ignore]
fn erzeuge_testabbilder() {
    let Ok(out) = std::env::var("VC_ATR_OUT") else { return };
    let out = std::path::Path::new(&out); std::fs::create_dir_all(out).unwrap();
    let text = |n: usize| -> Vec<u8> { (0..n).map(|i| b"VINTAGECODER TESTDATEI "[i % 23]).collect() };
    // DOS 2.5, 1040 Sektoren: eine Datei im Bereich bis 719, eine Kette über die Grenze 720 bis 1000
    let mut a = dos25(); let k1 = [4usize, 5]; let k2 = [718usize, 719, 720, 721, 1000];
    lege(&mut a, 361, 0, "KLEIN.TXT", &k1, &text(200)); lege(&mut a, 361, 1, "GROSS.DAT", &k2, &text(600));
    for &s in k1.iter().chain(&k2) { if s < 720 { setze(&mut a, 360, 10, 0, s, false) } else { setze(&mut a, 1024, 84, 720, s, false) } }
    std::fs::write(out.join("dos25_1040.atr"), a.to_bytes()).unwrap();
    // MyDOS 720 × 256 mit Unterverzeichnis SPIEL (Sektoren 100–107) und einer Datei darin, dazu eine im Hauptverzeichnis
    let mut m = mydos(false);
    { let e = &mut m.sector_mut(361)[16..32]; e[0] = 0x50; e[1] = 8; e[3] = 100; e[5..16].fill(b' '); e[5..10].copy_from_slice(b"SPIEL"); }
    belege(&mut m, 360, 10, 0, &(100..108).collect::<Vec<_>>()); zaehler_minus(&mut m, 8);
    lege(&mut m, 361, 0, "HALLO.TXT", &[4, 5], &text(400)); belege(&mut m, 360, 10, 0, &[4, 5]); zaehler_minus(&mut m, 2);
    lege(&mut m, 100, 0, "LOS.TXT", &[200], &text(100)); belege(&mut m, 360, 10, 0, &[200]); zaehler_minus(&mut m, 1);
    std::fs::write(out.join("mydos_ordner_256.atr"), m.to_bytes()).unwrap();
    // MyDOS mit durchgehend 256 Byte und mit 128-Byte-Boot-Sektoren (ATR-Standard)
    let mut b = mydos(true); lege(&mut b, 361, 0, "STANDARD.TXT", &[4, 5, 6], &text(700)); belege(&mut b, 360, 10, 0, &[4, 5, 6]); zaehler_minus(&mut b, 3);
    std::fs::write(out.join("mydos_boot128.atr"), b.to_bytes()).unwrap();
}
