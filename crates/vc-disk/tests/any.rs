// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_disk::{Addr, AnyDisk, Cell, Format};

fn c1541() -> Option<PathBuf> {
    ["/Applications/vice-arm64-gtk3-3.10/bin/c1541", "/opt/homebrew/bin/c1541", "/usr/local/bin/c1541", "/usr/bin/c1541"].iter().map(PathBuf::from).find(|p| p.exists())
}
fn prg(n: usize, seed: u8) -> Vec<u8> { let mut v = vec![0x01, 0x08]; v.extend((0..n).map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))); v }

#[test]
fn geometry_of_all_formats() {
    for (f, tracks, total, ssz) in [(Format::D64, 35, 683, 256), (Format::D71, 70, 1366, 256), (Format::D81, 80, 3200, 256), (Format::Atr, 40, 720, 128)] {
        let d = AnyDisk::blank(f, "T", "01").unwrap();
        assert_eq!((d.tracks(), d.total_sectors(), d.sector_size()), (tracks, total, ssz), "{f:?}");
        let an = d.analyze();
        assert!(an.problems.is_empty(), "{f:?}: {:?}", an.problems);
        assert_eq!(an.cells.iter().map(|r| r.len()).sum::<usize>(), total);
        assert!(!an.cells.iter().flatten().any(|c| matches!(c, Cell::Orphan | Cell::Conflict)), "{f:?}");
        assert_eq!(AnyDisk::from_file_bytes(&format!("x.{}", f.label().to_lowercase()), &d.to_bytes()).unwrap().format(), f);
    }
}

#[test]
fn analysis_follows_chains_and_pointers() {
    let mut d = AnyDisk::blank(Format::D64, "TEST", "01").unwrap();
    d.add_file("A", &prg(600, 1)).unwrap();
    d.add_file("B", &prg(100, 2)).unwrap();
    let (files, an) = (d.files(), d.analyze());
    assert_eq!(files.iter().map(|f| (f.blocks, f.autostart)).collect::<Vec<_>>(), vec![(3, true), (1, false)]);
    assert_eq!(an.used, 4);
    assert_eq!(an.free, 683 - 2 - 4);
    assert_eq!(an.interleave, Some(10));
    // Folgen: Datei A hat 3 Sektoren, danach endet die Kette
    let mut a = files[0].start;
    assert_eq!(a, Addr { track: 17, sector: 0 });
    let mut n = 1;
    while let Some(nx) = d.next(a, &an) { assert_eq!(an.cell(nx), Some(Cell::File(0))); a = nx; n += 1; }
    assert_eq!(n, 3);
    // BAM zeigt auf das Verzeichnis
    assert_eq!(d.next(Addr { track: 18, sector: 0 }, &an), Some(Addr { track: 18, sector: 1 }));
    assert_eq!(d.sector(Addr { track: 18, sector: 0 }).unwrap().len(), 256);
    assert!(d.sector(Addr { track: 36, sector: 0 }).is_none());
    // zerstörte BAM: Sektor als frei markiert, aber belegt → Prüfung meldet es
    let mut bytes = d.to_bytes();
    bytes[357 * 256 + 4 * 17 + 1] |= 1;
    let bad = AnyDisk::from_file_bytes("x.d64", &bytes).unwrap();
    assert!(!bad.analyze().problems.is_empty());
}

#[test]
fn atr_sector_chain() {
    let mut d = AnyDisk::blank(Format::Atr, "", "").unwrap();
    d.add_file("autorun.sys", &[0xFF, 0xFF, 0, 0x20, 0xFF, 0x20, 1, 2, 3]).unwrap();
    let data: Vec<u8> = (0..300).map(|i| i as u8).collect();
    d.add_file("daten.dat", &data).unwrap();
    let (files, an) = (d.files(), d.analyze());
    assert_eq!(files.iter().map(|f| (String::from_utf8_lossy(&f.name).into_owned(), f.kind, f.autostart)).collect::<Vec<_>>(),
        vec![("AUTORUN.SYS".into(), "XEX", true), ("DATEN.DAT".into(), "DAT", false)]);
    assert!(an.problems.is_empty(), "{:?}", an.problems);
    let mut a = files[1].start;
    assert_eq!(AnyDisk::atr_number(a), 5);
    let mut n = 1;
    while let Some(nx) = d.next(a, &an) { a = nx; n += 1; }
    assert_eq!(n, 3);
    assert_eq!(vc_disk::AnyDisk::atr_addr(360), Addr { track: 20, sector: 17 });
    assert_eq!(an.cell(AnyDisk::atr_addr(360)), Some(Cell::System));
}

#[test]
fn matches_c1541() {
    let Some(c) = c1541() else { eprintln!("c1541 nicht gefunden – übersprungen"); return };
    let dir = std::env::temp_dir().join(format!("vc_any_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir); std::fs::create_dir_all(&dir).unwrap();
    for (ext, fmt, tracks) in [("d64", "d64", 35), ("d71", "d71", 70), ("d81", "d81", 80)] {
        std::fs::write(dir.join("a.prg"), prg(2000, 3)).unwrap();
        std::fs::write(dir.join("b.prg"), prg(300, 4)).unwrap();
        let img = dir.join(format!("t.{ext}"));
        let st = Command::new(&c).args(["-format", "disk,01", fmt, img.to_str().unwrap(), "-write", dir.join("a.prg").to_str().unwrap(), "a", "-write", dir.join("b.prg").to_str().unwrap(), "b"]).output().unwrap();
        assert!(st.status.success(), "{}", String::from_utf8_lossy(&st.stderr));
        let d = AnyDisk::from_file_bytes(img.to_str().unwrap(), &std::fs::read(&img).unwrap()).unwrap();
        assert_eq!(d.tracks(), tracks);
        let (files, an) = (d.files(), d.analyze());
        assert_eq!(files.iter().map(|f| f.blocks).collect::<Vec<_>>(), vec![8, 2], "{ext}");
        assert!(an.problems.is_empty(), "{ext}: {:?}", an.problems);
        assert_eq!(an.used, 10, "{ext}");
        assert!(!an.cells.iter().flatten().any(|c| matches!(c, Cell::Orphan | Cell::Conflict)), "{ext}");
        // jede Kette endet; Länge entspricht den Blöcken
        for (i, f) in files.iter().enumerate() {
            let (mut a, mut n) = (f.start, 1);
            while let Some(nx) = d.next(a, &an) { assert_eq!(an.cell(nx), Some(Cell::File(i))); a = nx; n += 1; }
            assert_eq!(n, f.blocks as usize, "{ext} {i}");
        }
        // unsere Diskette ist für c1541 gültig und umgekehrt
        let mut ours = AnyDisk::blank(match ext { "d64" => Format::D64, "d71" => Format::D71, _ => Format::D81 }, "disk", "01").unwrap();
        ours.add_file("a", &prg(2000, 3)).unwrap();
        let o = dir.join(format!("o.{ext}")); std::fs::write(&o, ours.to_bytes()).unwrap();
        let v = Command::new(&c).args(["-attach", o.to_str().unwrap(), "-validate", "-list"]).output().unwrap();
        assert!(v.status.success() && String::from_utf8_lossy(&v.stdout).contains("\"a\""), "{}", String::from_utf8_lossy(&v.stdout));
    }
    let _ = std::fs::remove_dir_all(&dir);
}
