// SPDX-License-Identifier: GPL-3.0-or-later
//! Manipulierte Abbilder: Verzeichniskette mit Selbstverweis, Kreis und Zeigern aus der Diskette heraus.
//! Früher: Endlosschleife in `check`/`add_prg` und Panic (Index außerhalb) bei Spur > 35. Jetzt: Fehlermeldung, nie hängen, nie abstürzen.
use std::sync::mpsc;
use std::time::Duration;
use vc_disk::{AnyDisk, Disk, Format, Kind, D64};

/// Führt `f` in einem Thread aus und scheitert, wenn es länger als 20 s braucht (Endlosschleife) oder abstürzt
fn begrenzt<T: Send + 'static>(name: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    let h = std::thread::spawn(move || { let _ = tx.send(f()); });
    match rx.recv_timeout(Duration::from_secs(20)) {
        Ok(v) => { h.join().unwrap(); v }
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("{name}: Endlosschleife (Zeitgrenze überschritten)"),
        Err(_) => { let _ = h.join(); panic!("{name}: Absturz (Panic)") }
    }
}

fn off(first_sector_of_track: usize, s: usize) -> usize { (first_sector_of_track + s) * 256 }
/// (Bytes-Offset des ersten Verzeichnissektors, eines zweiten, freien Sektors der Verzeichnisspur)
fn orte(kind: &str) -> (usize, usize) {
    match kind { "d81" => (off(39 * 40, 3), off(39 * 40, 6)), _ => (off(17 * 21, 1), off(17 * 21, 4)) }
}

#[derive(Clone, Copy, Debug)]
enum Fall { Selbst, Kreis, SpurZuGross, SektorZuGross, SpurNull }

fn manipuliere(kind: &str, bytes: &mut [u8], fall: Fall) {
    let (a, b) = orte(kind);
    let (ta, sa) = if kind == "d81" { (40u8, 3u8) } else { (18, 1) };
    let (tb, sb) = if kind == "d81" { (40u8, 6u8) } else { (18, 4) };
    match fall {
        Fall::Selbst => { bytes[a] = ta; bytes[a + 1] = sa; }
        Fall::Kreis => { bytes[a] = tb; bytes[a + 1] = sb; bytes[b] = ta; bytes[b + 1] = sa; }
        Fall::SpurZuGross => { bytes[a] = 99; bytes[a + 1] = 0; }
        Fall::SektorZuGross => { bytes[a] = ta; bytes[a + 1] = 250; }
        Fall::SpurNull => { bytes[a] = 0; bytes[a + 1] = 5; }   // Ende der Kette: regulär
    }
}

fn leer(kind: &str) -> Vec<u8> {
    match kind { "d64" => D64::blank("T", "01").unwrap().to_bytes(), "d71" => Disk::blank(Kind::D71, "T", "01").unwrap().to_bytes(), _ => Disk::blank(Kind::D81, "T", "01").unwrap().to_bytes() }
}
fn prg() -> Vec<u8> { let mut v = vec![1, 8]; v.extend(0..100u8); v }

#[test]
fn verzeichnisketten_schleifen_und_zeiger_werden_gemeldet() {
    for kind in ["d64", "d71", "d81"] {
        for fall in [Fall::Selbst, Fall::Kreis, Fall::SpurZuGross, Fall::SektorZuGross] {
            let mut b = leer(kind);
            manipuliere(kind, &mut b, fall);
            let name = format!("{kind} {fall:?}");
            let (b2, k2) = (b.clone(), kind.to_string());
            let probleme = begrenzt(&name, move || match k2.as_str() {
                "d64" => { let d = D64::from_bytes(&b2).unwrap(); let _ = d.entries(); d.check() }
                "d71" => { let d = Disk::from_bytes(Kind::D71, &b2).unwrap(); let _ = d.entries(); d.check() }
                _ => { let d = Disk::from_bytes(Kind::D81, &b2).unwrap(); let _ = d.entries(); d.check() }
            });
            assert!(!probleme.is_empty(), "{name}: check muss das melden");
            // add_prg: Fehler statt Hänger (Verzeichnis voll machen, damit der Weg über „letzter Sektor“ geht, ist nicht nötig: es darf nur nicht hängen/abstürzen)
            let k2 = kind.to_string();
            begrenzt(&format!("{name} add_prg"), move || {
                for i in 0..40 {
                    let n = format!("P{i}");
                    let r = match k2.as_str() {
                        "d64" => D64::from_bytes(&b).unwrap().add_prg(&n, &prg()),
                        "d71" => Disk::from_bytes(Kind::D71, &b).unwrap().add_prg(&n, &prg()),
                        _ => Disk::from_bytes(Kind::D81, &b).unwrap().add_prg(&n, &prg()),
                    };
                    let _ = r;
                }
            });
            // analyze (AnyDisk) war schon geschützt: darf ebenfalls nicht hängen
            let b3 = { let mut x = leer(kind); manipuliere(kind, &mut x, fall); x };
            let fmt = match kind { "d64" => "x.d64", "d71" => "x.d71", _ => "x.d81" };
            begrenzt(&format!("{name} analyze"), move || AnyDisk::from_file_bytes(fmt, &b3).unwrap().analyze());
        }
    }
}

#[test]
fn volles_verzeichnis_mit_kaputter_kette_meldet_fehler_beim_anlegen() {
    // alle Einträge der ersten Verzeichnissektoren belegt, letzter Sektor zeigt auf sich: neuer Sektor wäre nötig → Fehler, kein Hängen
    let mut d = D64::blank("T", "01").unwrap();
    for i in 0..8 { d.add_prg(&format!("F{i}"), &prg()).unwrap(); }
    let mut b = d.to_bytes();
    let (a, _) = orte("d64");
    b[a] = 18; b[a + 1] = 1;
    let r = begrenzt("voll", move || D64::from_bytes(&b).unwrap().add_prg("NEU", &prg()));
    assert!(r.is_err());
}

#[test]
fn regulaeres_ende_und_normale_diskette_bleiben_unveraendert() {
    for kind in ["d64", "d71", "d81"] {
        let mut b = leer(kind);
        manipuliere(kind, &mut b, Fall::SpurNull);
        let probleme = match kind {
            "d64" => D64::from_bytes(&b).unwrap().check(),
            "d71" => Disk::from_bytes(Kind::D71, &b).unwrap().check(),
            _ => Disk::from_bytes(Kind::D81, &b).unwrap().check(),
        };
        assert!(probleme.is_empty(), "{kind}: {probleme:?}");
    }
    let _ = Format::D64;
}

/// Zufällig veränderte Abbilder (Verzeichnisspur, BAM, Dateisektoren): alle Lese- und Prüfwege liefern, ohne zu hängen oder abzustürzen
#[test]
fn zufaellig_veraenderte_abbilder_haengen_und_stuerzen_nicht() {
    for kind in ["d64", "d71", "d81"] {
        let mut basis = match kind { "d64" => AnyDisk::blank(Format::D64, "T", "01"), "d71" => AnyDisk::blank(Format::D71, "T", "01"), _ => AnyDisk::blank(Format::D81, "T", "01") }.unwrap();
        for i in 0..12 { basis.add_file(&format!("F{i}"), &{ let mut v = prg(); v.resize(300 + i * 700, 7); v }).unwrap(); }
        let bytes = basis.to_bytes();
        let ext = kind.to_string();
        begrenzt(kind, move || {
            let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
            let mut rnd = move || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
            for _ in 0..1500 {
                let mut b = bytes.clone();
                for _ in 0..(1 + rnd() % 6) {
                    // bevorzugt die ersten Bytes jedes Sektors (Kettenzeiger) und die Verzeichnisspur
                    let sektor = (rnd() as usize) % (b.len() / 256);
                    let pos = if rnd() % 2 == 0 { sektor * 256 + (rnd() as usize % 2) } else { (rnd() as usize) % b.len() };
                    b[pos] = rnd() as u8;
                }
                let Ok(mut d) = AnyDisk::from_file_bytes(&format!("x.{ext}"), &b) else { continue };
                let an = d.analyze();
                for f in d.files() { let _ = d.sector(f.start); }
                for t in 1..=d.tracks() { for s in 0..d.sectors_in_track(t) { let a = vc_disk::Addr { track: t, sector: s }; let _ = d.next(a, &an); } }
                let _ = d.check();
                let _ = d.add_file("NEU", &prg());
            }
        });
    }
}
