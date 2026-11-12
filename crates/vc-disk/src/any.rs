// SPDX-License-Identifier: GPL-3.0-or-later
//! Einheitliche, lesende Sicht auf D64, D71, D81 und ATR (DOS 2) für das Disk-Studio: Geometrie, rohe Sektoren, Kettenzeiger,
//! Belegung je Sektor und Verzeichnis mit Startsektor. Schreibend gibt es nur das Anlegen von Dateien.
//!
//! Adressen: Commodore (Spur, Sektor ab 0); ATR wird als 40 Spuren zu 18 Sektoren dargestellt, die Sektornummer (1…720) ist
//! `(Spur − 1) · 18 + Sektor + 1`.
use crate::{Atr, Disk, Kind, D64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Addr { pub track: usize, pub sector: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format { D64, D71, D81, Atr }

impl Format {
    pub fn label(self) -> &'static str { match self { Format::D64 => "D64", Format::D71 => "D71", Format::D81 => "D81", Format::Atr => "ATR" } }
    pub fn is_atari(self) -> bool { self == Format::Atr }
}

/// Nutzung eines Sektors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Free,
    /// Boot-Sektoren, BAM/VTOC, Diskettenkopf
    System,
    Dir,
    /// gehört zur Datei mit diesem Index (Reihenfolge von [`AnyDisk::files`])
    File(usize),
    /// laut BAM belegt, aber von keiner Kette erreicht
    Orphan,
    /// von mehreren Ketten erreicht oder Kette verläuft in Verzeichnis/System
    Conflict,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    /// Name als Bytes (PETSCII bzw. ATASCII), ohne Auffüllung
    pub name: Vec<u8>,
    /// PRG, SEQ, USR, REL, DEL; bei ATR XEX, BAS oder DAT
    pub kind: &'static str,
    pub start: Addr,
    pub blocks: u16,
    pub locked: bool,
    /// Startet automatisch (erste PRG-Datei bei `LOAD"*",8`; AUTORUN.SYS beim Atari)
    pub autostart: bool,
}

/// Belegungsplan samt Statistik
#[derive(Debug, Clone)]
pub struct Analysis {
    /// [Spur − 1][Sektor]
    pub cells: Vec<Vec<Cell>>,
    pub free: usize,
    /// Blöcke der Dateien
    pub used: usize,
    /// Meldungen der Prüfung (BAM, Ketten)
    pub problems: Vec<String>,
    /// häufigster Abstand aufeinanderfolgender Sektoren einer Kette innerhalb einer Spur
    pub interleave: Option<usize>,
}

impl Analysis {
    pub fn cell(&self, a: Addr) -> Option<Cell> { self.cells.get(a.track.wrapping_sub(1))?.get(a.sector).copied() }
}

#[derive(Debug, Clone)]
pub enum AnyDisk { D64(D64), Cbm(Disk), Atr(Atr) }

const ATR_PER_TRACK: usize = 18;

impl AnyDisk {
    pub fn from_file_bytes(name: &str, b: &[u8]) -> Result<AnyDisk, String> {
        let n = name.to_ascii_lowercase();
        if n.ends_with(".atr") { Atr::from_bytes(b).map(AnyDisk::Atr) }
        else if n.ends_with(".d64") { D64::from_bytes(b).map(AnyDisk::D64) }
        else if let Some(k) = Kind::from_extension(&n) { Disk::from_bytes(k, b).map(AnyDisk::Cbm) }
        else { Err(format!("{name}: unbekanntes Imageformat (erwartet .d64, .d71, .d81 oder .atr)")) }
    }

    /// Leere, formatierte Diskette
    pub fn blank(f: Format, name: &str, id: &str) -> Result<AnyDisk, String> {
        match f {
            Format::D64 => D64::blank(name, id).map(AnyDisk::D64),
            Format::D71 => Disk::blank(Kind::D71, name, id).map(AnyDisk::Cbm),
            Format::D81 => Disk::blank(Kind::D81, name, id).map(AnyDisk::Cbm),
            Format::Atr => Ok(AnyDisk::Atr(Atr::new_dos2())),
        }
    }

    pub fn format(&self) -> Format {
        match self { AnyDisk::D64(_) => Format::D64, AnyDisk::Cbm(d) => if d.kind() == Kind::D71 { Format::D71 } else { Format::D81 }, AnyDisk::Atr(_) => Format::Atr }
    }
    pub fn to_bytes(&self) -> Vec<u8> { match self { AnyDisk::D64(d) => d.to_bytes(), AnyDisk::Cbm(d) => d.to_bytes(), AnyDisk::Atr(a) => a.to_bytes() } }
    pub fn tracks(&self) -> usize { match self { AnyDisk::D64(_) => 35, AnyDisk::Cbm(d) => d.kind().tracks(), AnyDisk::Atr(_) => crate::atr::SECTORS / ATR_PER_TRACK } }
    pub fn sectors_in_track(&self, t: usize) -> usize {
        match self { AnyDisk::D64(_) => crate::sectors_in_track(t), AnyDisk::Cbm(d) => d.kind().sectors(t), AnyDisk::Atr(_) => ATR_PER_TRACK }
    }
    pub fn sector_size(&self) -> usize { if self.format().is_atari() { crate::atr::SECTOR_SIZE } else { 256 } }
    pub fn total_sectors(&self) -> usize { (1..=self.tracks()).map(|t| self.sectors_in_track(t)).sum() }
    /// Disketten-Name (leer beim Atari)
    pub fn title(&self) -> Vec<u8> {
        match self {
            AnyDisk::D64(d) => d.raw_sector(18, 0).map(|s| trim_name(&s[144..160])).unwrap_or_default(),
            AnyDisk::Cbm(d) => if d.kind() == Kind::D71 { d.raw_sector(18, 0).map(|s| trim_name(&s[144..160])).unwrap_or_default() } else { d.raw_sector(40, 0).map(|s| trim_name(&s[4..20])).unwrap_or_default() },
            AnyDisk::Atr(_) => vec![],
        }
    }
    /// Freie Blöcke bzw. Sektoren wie von DOS gemeldet
    pub fn free_blocks(&self) -> usize { match self { AnyDisk::D64(d) => d.free_blocks(), AnyDisk::Cbm(d) => d.free_blocks(), AnyDisk::Atr(a) => a.free_sectors() } }

    /// Nummer eines ATR-Sektors (1…720)
    pub fn atr_number(a: Addr) -> usize { (a.track - 1) * ATR_PER_TRACK + a.sector + 1 }
    pub fn atr_addr(n: usize) -> Addr { Addr { track: (n - 1) / ATR_PER_TRACK + 1, sector: (n - 1) % ATR_PER_TRACK } }

    pub fn sector(&self, a: Addr) -> Option<Vec<u8>> {
        match self {
            AnyDisk::D64(d) => d.raw_sector(a.track, a.sector).map(|s| s.to_vec()),
            AnyDisk::Cbm(d) => d.raw_sector(a.track, a.sector).map(|s| s.to_vec()),
            AnyDisk::Atr(x) => { if a.track == 0 || a.track > self.tracks() || a.sector >= ATR_PER_TRACK { None } else { Some(x.sector(Self::atr_number(a)).to_vec()) } }
        }
    }

    fn sector_free(&self, a: Addr) -> bool {
        match self { AnyDisk::D64(d) => d.sector_free(a.track, a.sector), AnyDisk::Cbm(d) => d.sector_free(a.track, a.sector), AnyDisk::Atr(x) => x.sector_free(Self::atr_number(a)) }
    }

    fn valid(&self, a: Addr) -> bool { a.track >= 1 && a.track <= self.tracks() && a.sector < self.sectors_in_track(a.track) }

    /// Kettenzeiger des Sektors: Commodore = Bytes 0/1 (Spur 0 beendet), ATR = Bytes 125/126 eines Dateisektors
    pub fn next(&self, a: Addr, an: &Analysis) -> Option<Addr> {
        let s = self.sector(a)?;
        if self.format().is_atari() {
            if !matches!(an.cell(a)?, Cell::File(_) | Cell::Conflict) { return None; }
            let n = ((s[125] & 3) as usize) << 8 | s[126] as usize;
            return if n == 0 || n > crate::atr::SECTORS { None } else { Some(Self::atr_addr(n)) };
        }
        let n = Addr { track: s[0] as usize, sector: s[1] as usize };
        if n.track == 0 || !self.valid(n) { None } else { Some(n) }
    }

    fn dir_start(&self) -> Option<Addr> {
        match self { AnyDisk::D64(_) => Some(Addr { track: 18, sector: 1 }), AnyDisk::Cbm(d) => Some(if d.kind() == Kind::D71 { Addr { track: 18, sector: 1 } } else { Addr { track: 40, sector: 3 } }), AnyDisk::Atr(_) => None }
    }

    pub fn files(&self) -> Vec<FileInfo> {
        match self {
            AnyDisk::D64(d) => cbm_files(d.entries()),
            AnyDisk::Cbm(d) => cbm_files(d.entries()),
            AnyDisk::Atr(a) => a.dos2_entries().into_iter().map(|e| {
                let kind = match a.dos2_read(&e.name) {
                    Ok(d) if d.len() >= 2 && d[0] == 0xFF && d[1] == 0xFF => "XEX",
                    _ if e.name.ends_with(".BAS") => "BAS",
                    _ => "DAT",
                };
                FileInfo { autostart: e.name == "AUTORUN.SYS", name: e.name.into_bytes(), kind, start: Self::atr_addr((e.start as usize).clamp(1, crate::atr::SECTORS)), blocks: e.sectors, locked: e.locked }
            }).collect(),
        }
    }

    /// Prüfmeldungen des Dateisystems (leer = in Ordnung)
    /// Hat das Image ein Dateisystem mit Verzeichnis? Nur eine ATR-Boot-Diskette ohne DOS-2-VTOC hat keines.
    pub fn has_filesystem(&self) -> bool { match self { AnyDisk::Atr(a) => a.has_dos2(), _ => true } }
    pub fn check(&self) -> Vec<String> { match self { AnyDisk::D64(d) => d.check(), AnyDisk::Cbm(d) => d.check(), AnyDisk::Atr(a) => a.dos2_check() } }

    /// Legt eine Datei an: Commodore als PRG (Name bis 16 Zeichen), Atari nach DOS-2-Regeln (Name wird bereinigt: Großbuchstaben, 8.3)
    pub fn add_file(&mut self, name: &str, data: &[u8]) -> Result<String, String> {
        match self {
            AnyDisk::D64(d) => { d.add_prg(name, data)?; Ok(name.to_ascii_uppercase()) }
            AnyDisk::Cbm(d) => { d.add_prg(name, data)?; Ok(name.to_ascii_uppercase()) }
            AnyDisk::Atr(a) => { let n = dos2_name(name); a.dos2_add(&n, data)?; Ok(n) }
        }
    }

    pub fn analyze(&self) -> Analysis {
        let files = self.files();
        let mut cells: Vec<Vec<Option<Cell>>> = (1..=self.tracks()).map(|t| vec![None; self.sectors_in_track(t)]).collect();
        let set = |cells: &mut Vec<Vec<Option<Cell>>>, a: Addr, c: Cell| {
            let slot = &mut cells[a.track - 1][a.sector];
            *slot = Some(match (*slot, c) { (None, c) => c, (Some(Cell::Free), c) => c, _ => Cell::Conflict });
        };
        let mut problems = vec![];
        // Boot-Diskette ohne Dateisystem: belegt sind genau die Sektoren mit Inhalt, es gibt kein Verzeichnis
        let boot_only = !self.has_filesystem();
        let system: Vec<Addr> = if boot_only { (1..=crate::atr::SECTORS).map(Self::atr_addr).filter(|a| self.sector(*a).map_or(false, |s| s.iter().any(|&b| b != 0))).collect() } else { match self {
            AnyDisk::D64(_) => vec![Addr { track: 18, sector: 0 }],
            AnyDisk::Cbm(d) if d.kind() == Kind::D71 => (0..19).map(|s| Addr { track: 53, sector: s }).chain([Addr { track: 18, sector: 0 }]).collect(),
            AnyDisk::Cbm(_) => (0..3).map(|s| Addr { track: 40, sector: s }).collect(),
            AnyDisk::Atr(_) => (1..=3).chain([360, 720]).map(Self::atr_addr).collect(),
        } };
        for a in system { set(&mut cells, a, Cell::System); }
        if let Some(mut a) = self.dir_start() {
            let mut n = 0;
            loop {
                set(&mut cells, a, Cell::Dir);
                n += 1;
                let Some(s) = self.sector(a) else { break };
                let nx = Addr { track: s[0] as usize, sector: s[1] as usize };
                if nx.track == 0 || !self.valid(nx) || n > 400 { break; }
                if cells[nx.track - 1][nx.sector] == Some(Cell::Dir) { problems.push("Verzeichniskette ist ein Kreis".into()); break; }
                a = nx;
            }
        } else if !boot_only {
            for n in 361..=368 { set(&mut cells, Self::atr_addr(n), Cell::Dir); }
        }
        let mut link_deltas: Vec<usize> = vec![];
        for (i, f) in files.iter().enumerate() {
            let mut a = f.start;
            let mut steps = 0;
            while self.valid(a) {
                set(&mut cells, a, Cell::File(i));
                steps += 1;
                let Some(s) = self.sector(a) else { break };
                let nx = if self.format().is_atari() {
                    let n = ((s[125] & 3) as usize) << 8 | s[126] as usize;
                    if n == 0 || n > crate::atr::SECTORS { break } else { Self::atr_addr(n) }
                } else {
                    let nx = Addr { track: s[0] as usize, sector: s[1] as usize };
                    if nx.track == 0 { break } else { nx }
                };
                if !self.valid(nx) || steps > self.total_sectors() { break; }
                if nx.track == a.track { let n = self.sectors_in_track(a.track); link_deltas.push((nx.sector + n - a.sector) % n); }
                if matches!(cells[nx.track - 1][nx.sector], Some(Cell::File(j)) if j == i) { break; }   // Kreis: schon besucht
                a = nx;
            }
        }
        let (mut free, mut used) = (0, 0);
        let cells: Vec<Vec<Cell>> = cells.into_iter().enumerate().map(|(ti, row)| row.into_iter().enumerate().map(|(s, c)| {
            let a = Addr { track: ti + 1, sector: s };
            let c = match c { Some(c) => c, None => if boot_only || self.sector_free(a) { Cell::Free } else { Cell::Orphan } };
            match c { Cell::Free => free += 1, Cell::File(_) | Cell::Conflict => used += 1, _ => {} }
            c
        }).collect()).collect();
        problems.extend(self.check());
        let interleave = {
            let mut counts = std::collections::BTreeMap::new();
            for d in link_deltas { *counts.entry(d).or_insert(0usize) += 1; }
            counts.into_iter().max_by_key(|(_, c)| *c).map(|(d, _)| d)
        };
        Analysis { cells, free, used, problems, interleave }
    }
}

fn trim_name(b: &[u8]) -> Vec<u8> { b.iter().take_while(|c| **c != 0xA0).copied().collect() }

fn cbm_files(entries: Vec<crate::D64Entry>) -> Vec<FileInfo> {
    let first_prg = entries.iter().position(|e| e.kind == "PRG");
    entries.into_iter().enumerate().map(|(i, e)| FileInfo {
        name: e.name.chars().map(|c| c as u32 as u8).collect(), kind: e.kind, start: Addr { track: e.track as usize, sector: e.sector as usize }, blocks: e.blocks, locked: false, autostart: Some(i) == first_prg,
    }).collect()
}

/// Dateiname nach DOS-2-Regeln: Großbuchstaben, Ziffern, `_`; erstes Zeichen ein Buchstabe, Name bis 8, Erweiterung bis 3 Zeichen
pub fn dos2_name(name: &str) -> String {
    let clean = |s: &str, max: usize| -> String { s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }).take(max).collect() };
    let (base, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    let mut b = clean(base, 8);
    if !b.chars().next().map_or(false, |c| c.is_ascii_uppercase()) { b = format!("F{b}").chars().take(8).collect(); }
    let e = clean(ext, 3);
    if e.is_empty() { b } else { format!("{b}.{e}") }
}
