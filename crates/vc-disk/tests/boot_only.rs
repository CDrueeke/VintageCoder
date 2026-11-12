// SPDX-License-Identifier: GPL-3.0-or-later
use vc_disk::{summarize_problems, AnyDisk, Atr, Cell};

fn boot_only_atr() -> Atr {
    let mut a = Atr::blank();
    for s in 1..=5 { a.sector_mut(s)[0] = 0x55; a.sector_mut(s)[1] = s as u8; }   // Boot-Programm ohne DOS-Verzeichnis
    a
}

#[test]
fn boot_disk_without_dos2_has_no_filesystem_and_no_problem_flood() {
    let a = boot_only_atr();
    assert!(!a.has_dos2());
    assert!(a.dos2_check().is_empty(), "keine 707 Einzelmeldungen");
    let d = AnyDisk::Atr(a);
    assert!(!d.has_filesystem());
    assert!(d.files().is_empty());
    let an = d.analyze();
    assert!(an.problems.is_empty(), "{:?}", an.problems);
    let system = an.cells.iter().flatten().filter(|c| **c == Cell::System).count();
    assert_eq!(system, 5, "belegt sind genau die Sektoren mit Inhalt");
    assert!(!an.cells.iter().flatten().any(|c| matches!(c, Cell::Dir | Cell::Orphan)));
}

#[test]
fn dos2_disk_still_has_a_filesystem() {
    let a = Atr::new_dos2();
    assert!(a.has_dos2() && AnyDisk::Atr(a).has_filesystem());
}

#[test]
fn long_problem_lists_are_summarized() {
    let p: Vec<String> = (0..707).map(|i| format!("Sektor {i}")).collect();
    let s = summarize_problems(&p, 10);
    assert_eq!(s.len(), 11);
    assert_eq!(s[10], "… und 697 weitere Probleme");
    assert_eq!(summarize_problems(&p[..3], 10).len(), 3);
}
