// SPDX-License-Identifier: GPL-3.0-or-later
//! Löschen, Ersetzen und Schreibschutz in Atari-DOS-2-Abbildern (Turbo-BASIC-Konzept, Etappe E0).
//! Die Rundlauftests nehmen ein selbst erzeugtes Abbild; zusätzlich läuft derselbe Ablauf auf einer KOPIE der Diskette des Inhabers,
//! wenn sie da ist (`VC_TURBO_ATR` oder `assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr`; nicht im Repository, das Original wird nie geschrieben).
use std::path::PathBuf;
use vc_disk::Atr;

fn data(n: usize, seed: u8) -> Vec<u8> { (0..n).map(|i| (i as u8).wrapping_mul(7).wrapping_add(seed)).collect() }

fn sample() -> Atr {
    let mut a = Atr::new_dos2();
    a.dos2_add("DOS.SYS", &data(4625, 1)).unwrap();
    a.dos2_add("DUP.SYS", &data(5126, 2)).unwrap();
    a.dos2_add("MITTE.TXT", &data(300, 3)).unwrap();
    a.dos2_add("AUTORUN.SYS", &data(9000, 4)).unwrap();
    a.dos2_add("ENDE.DAT", &data(125, 5)).unwrap();
    a
}

fn content(a: &Atr) -> Vec<(String, Vec<u8>)> { a.dos2_entries().into_iter().map(|e| (e.name.clone(), a.dos2_read(&e.name).unwrap())).collect() }

#[test]
fn entfernen_gibt_sektoren_frei_und_laesst_die_anderen_dateien_unberuehrt() {
    let mut a = sample();
    let before = a.free_sectors();
    let others: Vec<_> = content(&a).into_iter().filter(|(n, _)| n != "MITTE.TXT").collect();
    let freed = a.dos2_remove("MITTE.TXT", false).unwrap();
    assert_eq!(freed, 3);
    assert_eq!(a.free_sectors(), before + 3);
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    assert_eq!(content(&a), others, "andere Dateien bit-gleich");
    assert!(a.dos2_read("MITTE.TXT").is_err());
    assert!(a.dos2_remove("MITTE.TXT", false).is_err(), "zweites Löschen: nicht gefunden");
}

#[test]
fn neue_datei_nutzt_den_freigewordenen_platz_und_die_dateinummer() {
    let mut a = sample();
    a.dos2_remove("MITTE.TXT", false).unwrap();
    a.dos2_add("NEU.TXT", &data(250, 9)).unwrap();
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    assert_eq!(a.dos2_read("NEU.TXT").unwrap(), data(250, 9));
    assert_eq!(a.dos2_read("ENDE.DAT").unwrap(), data(125, 5));
}

#[test]
fn ersetzen_behaelt_den_namen_und_prueft_nach() {
    let mut a = sample();
    let before = content(&a);
    a.dos2_replace("AUTORUN.SYS", &data(500, 77)).unwrap();
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    assert_eq!(a.dos2_read("AUTORUN.SYS").unwrap(), data(500, 77));
    for (n, d) in before.iter().filter(|(n, _)| n != "AUTORUN.SYS") { assert_eq!(&a.dos2_read(n).unwrap(), d, "{n}"); }
    assert!(a.dos2_replace("GIBTS.NIX", &[1]).is_err(), "replace braucht eine vorhandene Datei");
    a.dos2_put("GIBTS.NIX", &data(10, 1)).unwrap();                     // put legt an
    a.dos2_put("GIBTS.NIX", &data(20, 2)).unwrap();                     // und ersetzt
    assert_eq!(a.dos2_read("GIBTS.NIX").unwrap(), data(20, 2));
    assert_eq!(a.dos2_entries().iter().filter(|e| e.name == "GIBTS.NIX").count(), 1);
}

#[test]
fn geschuetzte_dateien_werden_nicht_geloescht_oder_ersetzt_ausser_mit_force() {
    let mut a = sample();
    assert_eq!(a.dos2_is_locked("DOS.SYS"), Some(false));
    a.dos2_set_locked("DOS.SYS", true).unwrap();
    assert_eq!(a.dos2_is_locked("DOS.SYS"), Some(true));
    assert!(a.dos2_entries().iter().find(|e| e.name == "DOS.SYS").unwrap().locked);
    let snapshot = a.clone();
    assert!(a.dos2_remove("DOS.SYS", false).unwrap_err().contains("DOS.SYS"));
    assert!(a.dos2_replace("DOS.SYS", &[1, 2, 3]).is_err());
    assert!(a.dos2_put("DOS.SYS", &[1, 2, 3]).is_err());
    assert_eq!(a, snapshot, "bei einem Fehler bleibt das Abbild unverändert");
    a.dos2_remove("DOS.SYS", true).unwrap();
    assert!(a.dos2_check().is_empty());
    assert_eq!(a.dos2_is_locked("DOS.SYS"), None);
    assert!(a.dos2_set_locked("DOS.SYS", false).is_err());
}

#[test]
fn zu_wenig_platz_laesst_das_abbild_unveraendert() {
    let mut a = sample();
    let snapshot = a.clone();
    assert!(a.dos2_put("RIESIG.DAT", &data(125 * 5000, 1)).unwrap_err().contains("RIESIG.DAT"));
    assert!(a.dos2_replace("AUTORUN.SYS", &data(125 * 5000, 1)).is_err());
    assert_eq!(a, snapshot);
    // Ersetzen einer großen Datei durch eine, die nur dank der freiwerdenden Sektoren passt
    let mut b = Atr::new_dos2();
    let big = 125 * (b.free_sectors() - 10);
    b.dos2_add("GROSS.DAT", &data(big, 1)).unwrap();
    b.dos2_replace("GROSS.DAT", &data(big, 2)).unwrap();
    assert_eq!(b.dos2_read("GROSS.DAT").unwrap(), data(big, 2));
    assert!(b.dos2_check().is_empty());
}

#[test]
fn mehrere_eintraege_gleichen_namens_gehen_zusammen_weg_und_ungueltige_namen_werden_gemeldet() {
    let mut a = sample();
    // unvollständiger Rest wie auf der Diskette des Inhabers: zweiter Eintrag `$43` mit 0 Sektoren
    let slot = (0..64).find(|i| a.sector(361 + i / 8)[(i % 8) * 16] == 0).unwrap();
    let e = &mut a.sector_mut(361 + slot / 8)[(slot % 8) * 16..(slot % 8) * 16 + 16];
    e[0] = 0x43; e[5..13].copy_from_slice(b"ENDE    "); e[13..16].copy_from_slice(b"DAT");
    assert_eq!(a.dos2_entries().iter().filter(|x| x.name == "ENDE.DAT").count(), 2);
    a.dos2_remove("ENDE.DAT", false).unwrap();
    assert!(a.dos2_entries().iter().all(|x| x.name != "ENDE.DAT"));
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    assert!(a.dos2_remove("kleinbuchstaben", false).is_err());
    assert!(Atr::valid_dos2_name("AUTORUN.BAS") && !Atr::valid_dos2_name("autorun.bas") && !Atr::valid_dos2_name("ZU_LANGER_NAME.BAS"));
}

#[test]
fn verzeichnis_voll_nach_loeschen_wieder_frei() {
    let mut a = Atr::new_dos2();
    for i in 0..64 { a.dos2_add(&format!("F{i}.DAT"), &[i as u8 + 1]).unwrap(); }
    assert!(a.dos2_add("X.DAT", &[1]).is_err());
    a.dos2_remove("F10.DAT", false).unwrap();
    a.dos2_add("X.DAT", &[9]).unwrap();
    assert!(a.dos2_check().is_empty(), "{:?}", a.dos2_check());
    assert_eq!(a.dos2_read("X.DAT").unwrap(), vec![9]);
}

fn owner_disk() -> Option<Vec<u8>> {
    let p = std::env::var("VC_TURBO_ATR").map(PathBuf::from).ok().or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().map(|a| a.join("assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr")).find(|p| p.exists())
    })?;
    std::fs::read(p).ok()
}

/// Messung des Konzepts (Anhang A und D) an der Diskette des Inhabers (nur Lesen, im Speicher)
#[test]
fn diskette_des_inhabers_anhang_a_und_d() {
    let Some(bytes) = owner_disk() else { eprintln!("keine Turbo-Diskette: übersprungen"); return };
    let mut a = Atr::from_bytes(&bytes).unwrap();
    let (errors, warnings) = a.dos2_check_split();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(warnings.len(), 2, "Altlasten: nie abgeschlossener Eintrag PUTGET.TUR und Sektor 621 ohne Datei: {warnings:?}");
    assert_eq!(a.free_sectors(), 103);
    let keep: Vec<(String, Vec<u8>)> = ["DOS.SYS", "DUP.SYS", "AUTORUN.SYS"].iter().map(|n| (n.to_string(), a.dos2_read(n).unwrap())).collect();
    assert_eq!(keep[2].1.len(), 18104);
    assert!(a.dos2_entries().iter().filter(|e| e.name != "PUTGET.TUR").all(|e| e.locked), "Kennzeichen $62: schreibgeschützt");
    // Anhang D: auf die drei Systemdateien reduzieren → 482 freie Sektoren
    let names: Vec<String> = a.dos2_entries().into_iter().map(|e| e.name).filter(|n| !keep.iter().any(|k| &k.0 == n)).collect();
    assert!(names.contains(&"COMPILER.COM".to_string()));
    for n in &names { assert!(a.dos2_remove(n, false).is_err() || a.dos2_is_locked(n).is_none(), "geschützt: nur mit force"); }
    for n in &names { if a.dos2_is_locked(n).is_some() { a.dos2_remove(n, true).unwrap(); } }
    assert!(a.dos2_check_split().0.is_empty(), "{:?}", a.dos2_check_split());
    assert_eq!(a.free_sectors(), 482, "Anhang D des Konzepts");
    for (n, d) in &keep { assert_eq!(&a.dos2_read(n).unwrap(), d, "{n} bit-gleich"); }
    // Programm als AUTORUN.BAS schreiben, ersetzen
    a.dos2_put("AUTORUN.BAS", &data(32956, 1)).unwrap();        // das große Programm aus Anhang D (264 Sektoren) passt
    assert!(a.dos2_check_split().0.is_empty());
    a.dos2_replace("AUTORUN.BAS", &data(1284, 2)).unwrap();
    assert!(a.dos2_check_split().0.is_empty());
    assert_eq!(a.dos2_read("AUTORUN.BAS").unwrap(), data(1284, 2));
    // auf dem Original (103 frei) passt das große Programm nicht
    let mut orig = Atr::from_bytes(&bytes).unwrap();
    assert!(orig.dos2_put("AUTORUN.BAS", &data(32956, 1)).is_err());
    // Original-Bytes unverändert (nur im Speicher gearbeitet)
    assert_eq!(Atr::from_bytes(&bytes).unwrap().free_sectors(), 103);
}
