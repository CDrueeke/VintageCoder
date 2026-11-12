// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_disk::{Disk, Kind};

fn c1541() -> Option<PathBuf> {
    ["/Applications/vice-arm64-gtk3-3.10/bin/c1541", "/opt/homebrew/bin/c1541", "/usr/local/bin/c1541", "/usr/bin/c1541"].iter().map(PathBuf::from).find(|p| p.exists())
}
fn prg(n: usize, seed: u8) -> Vec<u8> { let mut v = vec![0x01, 0x08]; v.extend((0..n).map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))); v }

#[test]
fn geometry_and_blank_disks() {
    assert_eq!((1..=70).map(|t| Kind::D71.sectors(t)).sum::<usize>() * 256, Kind::D71.size());
    assert_eq!(80 * 40 * 256, Kind::D81.size());
    for (kind, free, dir) in [(Kind::D71, 1328, "MY DISK"), (Kind::D81, 3160, "MY DISK")] {
        let d = Disk::blank(kind, "my disk", "01").unwrap();
        assert_eq!(d.to_bytes().len(), kind.size());
        assert_eq!(d.free_blocks(), free);
        assert_eq!(d.disk_name(), dir);
        assert!(d.check().is_empty(), "{kind:?}: {:?}", d.check());
        assert_eq!(Disk::from_bytes(kind, &d.to_bytes()).unwrap(), d);
        assert!(Disk::from_bytes(kind, &[0; 100]).is_err());
    }
    assert!(Disk::blank(Kind::D71, "X", "1").is_err());
    assert_eq!(Disk::from_file_bytes("a.D81", &Disk::blank(Kind::D81, "X", "01").unwrap().to_bytes()).unwrap().kind(), Kind::D81);
}

#[test]
fn files_roundtrip_on_both_formats() {
    for kind in [Kind::D71, Kind::D81] {
        let mut d = Disk::blank(kind, "TEST", "AB").unwrap();
        let free0 = d.free_blocks();
        let (a, b, big) = (prg(100, 1), prg(2000, 2), prg(150_000, 3));
        d.add_prg("hello", &a).unwrap(); d.add_prg("MITTEL", &b).unwrap(); d.add_prg("GROSS", &big).unwrap();
        assert_eq!((d.read("HELLO").unwrap(), d.read("mittel").unwrap(), d.read("GROSS").unwrap()), (a, b, big.clone()));
        assert_eq!(d.entries().iter().map(|x| (x.name.as_str(), x.blocks)).collect::<Vec<_>>(), vec![("HELLO", 1), ("MITTEL", 8), ("GROSS", 591)]);
        assert_eq!(d.free_blocks(), free0 - 1 - 8 - 591);
        assert!(d.check().is_empty(), "{kind:?}: {:?}", d.check());
        // viele Einträge → weitere Verzeichnissektoren
        for i in 0..20 { d.add_prg(&format!("F{i:02}"), &[1, 8, i as u8]).unwrap(); }
        assert_eq!(d.entries().len(), 23); assert_eq!(d.read("F19").unwrap(), vec![1, 8, 19]);
        assert!(d.check().is_empty(), "{kind:?}: {:?}", d.check());
        assert!(d.add_prg("hello", b"x").unwrap_err().contains("existiert") && d.add_prg("A:B", b"x").is_err());
        let full = free0 * 254;
        assert!(Disk::blank(kind, "V", "01").unwrap().add_prg("ZUVIEL", &vec![0; full + 300]).unwrap_err().contains("voll"));
    }
}

/// Abgleich mit dem echten `c1541` aus VICE (beide Richtungen, beide Formate).
#[test]
fn matches_c1541() {
    let Some(c) = c1541() else { eprintln!("c1541 nicht gefunden – übersprungen"); return };
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("c1541_cbm");
    std::fs::create_dir_all(&dir).unwrap();
    let run = |args: &[&str]| { let o = Command::new(&c).args(args).output().unwrap(); (String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr)).lines().filter(|l| !l.contains("OPENCBM")).collect::<Vec<_>>().join("\n") };
    for (kind, ext, free) in [(Kind::D71, "d71", 1328), (Kind::D81, "d81", 3160)] {
        let (p1, p2) = (prg(3000, 7), prg(60, 9));
        // leere Diskette: unser Image == frisch von c1541 formatiert
        let fresh = dir.join(format!("fresh.{ext}")); let _ = std::fs::remove_file(&fresh);
        run(&["-format", "ours,42", ext, fresh.to_str().unwrap()]);
        let ours_blank = Disk::blank(kind, "OURS", "42").unwrap().to_bytes();
        let theirs_blank = std::fs::read(&fresh).unwrap();
        assert_eq!(ours_blank.len(), theirs_blank.len());
        let diff: Vec<usize> = (0..ours_blank.len()).filter(|&i| ours_blank[i] != theirs_blank[i]).collect();
        assert!(diff.is_empty(), "{ext}: {} abweichende Bytes, erste bei {:?}", diff.len(), &diff[..diff.len().min(8)]);
        // 1) unser Image → c1541 liest Verzeichnis, Dateien, validiert
        let mut d = Disk::blank(kind, "OURS", "42").unwrap();
        d.add_prg("ALPHA", &p1).unwrap(); d.add_prg("BETA", &p2).unwrap();
        let img = dir.join(format!("ours.{ext}")); std::fs::write(&img, d.to_bytes()).unwrap();
        let list = run(&["-attach", img.to_str().unwrap(), "-list"]);
        assert!(list.contains("\"ours") && list.contains("\"alpha\"") && list.contains("\"beta\"") && list.contains(&format!("{} blocks free", d.free_blocks())), "{ext}: {list}");
        let out = dir.join("alpha.out"); let _ = std::fs::remove_file(&out);
        run(&["-attach", img.to_str().unwrap(), "-read", "alpha", out.to_str().unwrap()]);
        assert_eq!(std::fs::read(&out).unwrap(), p1, "{ext}");
        let val = run(&["-attach", img.to_str().unwrap(), "-validate"]);
        assert!(!val.to_lowercase().contains("error") && !val.to_lowercase().contains("fail"), "{ext}: {val}");
        // 2) c1541-Image → unser Parser
        let host = dir.join("host.prg"); std::fs::write(&host, &p1).unwrap();
        let img2 = dir.join(format!("theirs.{ext}")); let _ = std::fs::remove_file(&img2);
        run(&["-format", "theirs,77", ext, img2.to_str().unwrap(), "-write", host.to_str().unwrap(), "one", "-write", host.to_str().unwrap(), "two"]);
        let t = Disk::from_bytes(kind, &std::fs::read(&img2).unwrap()).unwrap();
        assert_eq!(t.disk_name(), "THEIRS");
        assert_eq!(t.entries().iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["ONE", "TWO"]);
        assert_eq!(t.read("ONE").unwrap(), p1);
        assert_eq!(t.free_blocks(), free - 24);
        assert!(t.check().is_empty(), "{ext}: {:?}", t.check());
    }
}
