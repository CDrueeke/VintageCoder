// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::PathBuf;
use std::process::Command;
use vc_disk::{sectors_in_track, D64};

fn c1541() -> Option<PathBuf> {
    ["/Applications/vice-arm64-gtk3-3.10/bin/c1541", "/opt/homebrew/bin/c1541", "/usr/local/bin/c1541", "/usr/bin/c1541"].iter().map(PathBuf::from).find(|p| p.exists())
}
fn prg(n: usize, seed: u8) -> Vec<u8> { let mut v = vec![0x01, 0x08]; v.extend((0..n).map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))); v }

#[test]
fn geometry_and_blank_disk() {
    assert_eq!((1..=35).map(sectors_in_track).sum::<usize>(), 683);
    let d = D64::blank("MY DISK", "01").unwrap();
    let b = d.to_bytes();
    assert_eq!(b.len(), 174_848);
    let bam = 357 * 256;                                   // Spur 18, Sektor 0
    assert_eq!(&b[bam..bam + 4], &[18, 1, b'A', 0]);
    assert_eq!(&b[bam + 144..bam + 151], b"MY DISK");
    assert_eq!(b[bam + 151], 0xA0);
    assert_eq!(&b[bam + 162..bam + 167], &[b'0', b'1', 0xA0, b'2', b'A']);
    assert_eq!(&b[bam + 4..bam + 8], &[21, 0xFF, 0xFF, 0x1F], "Spur 1: 21 Sektoren frei");
    assert_eq!(&b[bam + 72..bam + 76], &[17, 0xFC, 0xFF, 0x07], "Spur 18: Sektoren 0 und 1 belegt");
    assert_eq!(d.free_blocks(), 664);
    assert!(d.check().is_empty() && d.entries().is_empty());
    assert_eq!(D64::from_bytes(&b).unwrap(), d);
    assert!(D64::from_bytes(&b[..100]).is_err() && D64::blank("X", "1").is_err() && D64::blank("ZU LANGER NAME 123", "01").is_err());
}

#[test]
fn files_roundtrip_with_correct_chains() {
    let mut d = D64::blank("TEST", "AB").unwrap();
    let (a, b, big) = (prg(100, 1), prg(2000, 2), prg(40_000, 3));
    d.add_prg("hello", &a).unwrap();
    d.add_prg("MITTEL", &b).unwrap();
    d.add_prg("GROSS", &big).unwrap();
    assert_eq!(d.read("HELLO").unwrap(), a);
    assert_eq!(d.read("mittel").unwrap(), b);
    assert_eq!(d.read("GROSS").unwrap(), big);
    let e = d.entries();
    assert_eq!(e.iter().map(|x| (x.name.as_str(), x.blocks, x.kind)).collect::<Vec<_>>(), vec![("HELLO", 1, "PRG"), ("MITTEL", 8, "PRG"), ("GROSS", 158, "PRG")]);
    assert_eq!(d.free_blocks(), 664 - 1 - 8 - 158);
    assert!(d.check().is_empty(), "{:?}", d.check());
    // Der Startsektor der ersten Datei liegt auf Spur 17 (DOS beginnt mittig)
    assert_eq!((e[0].track, e[0].sector), (17, 0));
    assert_eq!(D64::from_bytes(&d.to_bytes()).unwrap().read("GROSS").unwrap(), big);
}

#[test]
fn limits_names_and_directory_growth() {
    let mut d = D64::blank("T", "01").unwrap();
    assert!(d.add_prg("A:B", b"x").is_err() && d.add_prg("", b"x").is_err() && d.add_prg("X*", b"x").is_err() && d.add_prg("EMPTY", b"").is_err());
    d.add_prg("A", b"xx").unwrap();
    assert!(d.add_prg("a", b"yy").unwrap_err().contains("existiert"));
    assert!(d.add_prg("HUGE", &vec![0; 665 * 254]).unwrap_err().contains("voll"));
    let mut many = D64::blank("MANY", "01").unwrap();
    for i in 0..20 { many.add_prg(&format!("F{i:02}"), &[1, 8, i as u8]).unwrap(); }     // > 8 Einträge: zweiter/dritter Verzeichnissektor
    assert_eq!(many.entries().len(), 20);
    assert_eq!(many.read("F19").unwrap(), vec![1, 8, 19]);
    assert!(many.check().is_empty(), "{:?}", many.check());
    assert!(d.read("NIX").is_err());
}

/// Abgleich mit dem echten `c1541` aus VICE (beide Richtungen).
#[test]
fn matches_c1541() {
    let Some(c) = c1541() else { eprintln!("c1541 nicht gefunden – übersprungen"); return };
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("c1541");
    std::fs::create_dir_all(&dir).unwrap();
    let run = |args: &[&str]| { let o = Command::new(&c).args(args).output().unwrap(); (String::from_utf8_lossy(&o.stdout).into_owned() + &String::from_utf8_lossy(&o.stderr)).lines().filter(|l| !l.contains("OPENCBM")).collect::<Vec<_>>().join("\n") };
    let (p1, p2) = (prg(3000, 7), prg(60, 9));
    // 1) unser Image → c1541 liest Verzeichnis und Dateiinhalt
    let mut d = D64::blank("OURS", "42").unwrap();
    d.add_prg("ALPHA", &p1).unwrap();
    d.add_prg("BETA", &p2).unwrap();
    let img = dir.join("ours.d64");
    std::fs::write(&img, d.to_bytes()).unwrap();
    let list = run(&["-attach", img.to_str().unwrap(), "-list"]);
    assert!(list.contains("0 \"ours            \" 42 2a") && list.contains("12   \"alpha\"") && list.contains("1    \"beta\"") && list.contains(&format!("{} blocks free", d.free_blocks())), "{list}");
    let out = dir.join("alpha.out");
    run(&["-attach", img.to_str().unwrap(), "-read", "alpha", out.to_str().unwrap()]);
    assert_eq!(std::fs::read(&out).unwrap(), p1);
    let val = run(&["-attach", img.to_str().unwrap(), "-validate"]);
    assert!(!val.to_lowercase().contains("error") && !val.to_lowercase().contains("fail"), "{val}");
    // 2) c1541-Image → unser Parser
    let host = dir.join("host.prg");
    std::fs::write(&host, &p1).unwrap();
    let img2 = dir.join("theirs.d64");
    let _ = std::fs::remove_file(&img2);
    run(&["-format", "theirs,77", "d64", img2.to_str().unwrap(), "-write", host.to_str().unwrap(), "one", "-write", host.to_str().unwrap(), "two"]);
    let t = D64::from_bytes(&std::fs::read(&img2).unwrap()).unwrap();
    assert_eq!(t.disk_name(), "THEIRS");
    assert_eq!(t.entries().iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["ONE", "TWO"]);
    assert_eq!(t.read("ONE").unwrap(), p1);
    assert_eq!(t.free_blocks(), 664 - 24);
    assert!(t.check().is_empty(), "{:?}", t.check());
}
