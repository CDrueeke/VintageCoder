// SPDX-License-Identifier: GPL-3.0-or-later
//! ATR-Container (Single Density, 128-Byte-Sektoren) und Atari DOS 2.0S.
//!
//! Layout einer DOS-2.0S-Diskette: Sektor 1–3 Boot, 360 VTOC, 361–368 Verzeichnis, Daten in 4–359 und 369–719.
//! Datensektor: 125 Datenbytes, dann Byte 125 = (Dateinummer << 2) | Folgesektor hoch, 126 = Folgesektor niedrig, 127 = Anzahl Datenbytes.

pub const SECTORS: usize = 720;
pub const SECTOR_SIZE: usize = 128;
const VTOC: usize = 360;
const DIR_FIRST: usize = 361;
const DIR_SECTORS: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Atr {
    /// Sektoren 1..=720 (Index 0 = Sektor 1).
    sectors: Vec<[u8; SECTOR_SIZE]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dos2Entry { pub name: String, pub sectors: u16, pub start: u16, pub locked: bool }

impl Atr {
    /// Leere, unformatierte 90K-Diskette (alles 0).
    pub fn blank() -> Atr { Atr { sectors: vec![[0; SECTOR_SIZE]; SECTORS] } }

    pub fn sector(&self, n: usize) -> &[u8; SECTOR_SIZE] { &self.sectors[n - 1] }
    pub fn sector_mut(&mut self, n: usize) -> &mut [u8; SECTOR_SIZE] { &mut self.sectors[n - 1] }

    /// Leere DOS-2.0S-Diskette: VTOC und Verzeichnis angelegt.
    pub fn new_dos2() -> Atr {
        let mut a = Atr::blank();
        let v = a.sector_mut(VTOC);
        v[0] = 2;                                           // DOS-Code
        for s in 4..SECTORS { if !(VTOC..=DIR_FIRST + DIR_SECTORS - 1).contains(&s) { Self::set_free(v, s, true); } }
        let free = (4..SECTORS).filter(|s| !(VTOC..=DIR_FIRST + DIR_SECTORS - 1).contains(s)).count() as u16;
        v[1] = (free & 0xFF) as u8; v[2] = (free >> 8) as u8;                // verfügbare Sektoren
        v[3] = v[1]; v[4] = v[2];                                            // freie Sektoren
        a
    }

    fn set_free(vtoc: &mut [u8; SECTOR_SIZE], s: usize, free: bool) {
        let (byte, bit) = (10 + s / 8, 7 - s % 8);
        if free { vtoc[byte] |= 1 << bit } else { vtoc[byte] &= !(1 << bit) }
    }
    /// Ist Sektor `n` (1…720) laut VTOC frei? (Sektor 720 kennt die VTOC von DOS 2 nicht: nie frei.)
    pub fn sector_free(&self, n: usize) -> bool { n >= 1 && n < SECTORS && self.is_free(n) }
    fn is_free(&self, s: usize) -> bool { self.sector(VTOC)[10 + s / 8] & (1 << (7 - s % 8)) != 0 }

    pub fn free_sectors(&self) -> usize { self.sector(VTOC)[3] as usize | (self.sector(VTOC)[4] as usize) << 8 }

    fn valid_name(name: &str) -> Result<([u8; 8], [u8; 3]), String> {
        let (base, ext) = name.split_once('.').unwrap_or((name, ""));
        let ok = |s: &str, max: usize| s.len() <= max && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if base.is_empty() || !ok(base, 8) || !ok(ext, 3) || !base.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(format!("ungültiger Dateiname '{name}' (1–8 Zeichen A–Z/0–9/_, erstes ein Buchstabe, Erweiterung bis 3 Zeichen)"));
        }
        let mut b = [b' '; 8];
        let mut e = [b' '; 3];
        b[..base.len()].copy_from_slice(base.as_bytes());
        e[..ext.len()].copy_from_slice(ext.as_bytes());
        Ok((b, e))
    }

    /// Verzeichniseinträge (nicht gelöschte).
    pub fn dos2_entries(&self) -> Vec<Dos2Entry> {
        let mut v = vec![];
        for s in DIR_FIRST..DIR_FIRST + DIR_SECTORS {
            for i in 0..8 {
                let e = &self.sector(s)[i * 16..i * 16 + 16];
                if e[0] & 0x80 != 0 || e[0] & 0x40 == 0 { continue; }
                let base = String::from_utf8_lossy(&e[5..13]).trim_end().to_string();
                let ext = String::from_utf8_lossy(&e[13..16]).trim_end().to_string();
                v.push(Dos2Entry { name: if ext.is_empty() { base } else { format!("{base}.{ext}") }, sectors: e[1] as u16 | (e[2] as u16) << 8,
                                   start: e[3] as u16 | (e[4] as u16) << 8, locked: e[0] & 0x20 != 0 });
            }
        }
        v
    }

    /// Legt eine Datei an (Sektoren aufsteigend ab 4, Verzeichnis ab Eintrag 0).
    pub fn dos2_add(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let (base, ext) = Self::valid_name(name)?;
        if data.is_empty() { return Err(format!("'{name}': leere Dateien werden nicht unterstützt")); }
        if self.dos2_entries().iter().any(|e| e.name == name) { return Err(format!("'{name}' existiert bereits")); }
        let need = data.len().div_ceil(125);
        if need > self.free_sectors() { return Err(format!("Diskette voll: '{name}' braucht {need} Sektoren, frei sind {}", self.free_sectors())); }
        let slot = (0..DIR_SECTORS * 8).find(|i| { let e = self.sector(DIR_FIRST + i / 8)[(i % 8) * 16]; e & 0x40 == 0 || e & 0x80 != 0 }).ok_or("Verzeichnis voll (64 Dateien)")?;
        let secs: Vec<usize> = (4..SECTORS).filter(|s| self.is_free(*s)).take(need).collect();
        for (k, &s) in secs.iter().enumerate() {
            let chunk = &data[k * 125..((k + 1) * 125).min(data.len())];
            let next = secs.get(k + 1).copied().unwrap_or(0);
            let sec = self.sector_mut(s);
            *sec = [0; SECTOR_SIZE];
            sec[..chunk.len()].copy_from_slice(chunk);
            sec[125] = ((slot as u8) << 2) | (next >> 8) as u8;
            sec[126] = (next & 0xFF) as u8;
            sec[127] = chunk.len() as u8;
            Self::set_free(self.sector_mut(VTOC), s, false);
        }
        let free = self.free_sectors() - need;
        let v = self.sector_mut(VTOC);
        v[3] = (free & 0xFF) as u8; v[4] = (free >> 8) as u8;
        let e = &mut self.sector_mut(DIR_FIRST + slot / 8)[(slot % 8) * 16..(slot % 8) * 16 + 16];
        e[0] = 0x42;
        e[1] = (need & 0xFF) as u8; e[2] = (need >> 8) as u8;
        e[3] = (secs[0] & 0xFF) as u8; e[4] = (secs[0] >> 8) as u8;
        e[5..13].copy_from_slice(&base);
        e[13..16].copy_from_slice(&ext);
        Ok(())
    }

    /// Liest eine Datei über die Sektorkette; prüft Dateinummern und Zähler.
    pub fn dos2_read(&self, name: &str) -> Result<Vec<u8>, String> {
        let idx = (0..DIR_SECTORS * 8).find(|i| {
            let e = &self.sector(DIR_FIRST + i / 8)[(i % 8) * 16..(i % 8) * 16 + 16];
            e[0] & 0x40 != 0 && e[0] & 0x80 == 0 && Self::valid_name(name).map_or(false, |(b, x)| e[5..13] == b && e[13..16] == x)
        }).ok_or(format!("'{name}' nicht gefunden"))?;
        let e = &self.sector(DIR_FIRST + idx / 8)[(idx % 8) * 16..(idx % 8) * 16 + 16];
        let (mut s, expect) = (e[3] as usize | (e[4] as usize) << 8, e[1] as usize | (e[2] as usize) << 8);
        let (mut out, mut count) = (vec![], 0);
        while s != 0 {
            if s > SECTORS { return Err(format!("Sektor {s} außerhalb der Diskette")); }
            let sec = self.sector(s);
            if (sec[125] >> 2) as usize != idx { return Err(format!("Sektor {s} gehört nicht zu '{name}' (Dateinummer {})", sec[125] >> 2)); }
            if sec[127] > 125 { return Err(format!("Sektor {s}: ungültige Byteanzahl {}", sec[127])); }
            out.extend_from_slice(&sec[..sec[127] as usize]);
            s = ((sec[125] & 3) as usize) << 8 | sec[126] as usize;
            count += 1;
            if count > expect { return Err("Sektorkette länger als im Verzeichnis".into()); }
        }
        if count != expect { return Err(format!("Sektorkette hat {count} statt {expect} Sektoren")); }
        Ok(out)
    }

    /// ATR-Datei: 16-Byte-Kopf + 720 × 128 Byte.
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = SECTORS * SECTOR_SIZE;
        let para = size / 16;
        let mut v = vec![0x96, 0x02, (para & 0xFF) as u8, ((para >> 8) & 0xFF) as u8, SECTOR_SIZE as u8, 0, ((para >> 16) & 0xFF) as u8];
        v.resize(16, 0);
        for s in &self.sectors { v.extend_from_slice(s); }
        v
    }

    pub fn from_bytes(b: &[u8]) -> Result<Atr, String> {
        if b.len() < 16 || b[0] != 0x96 || b[1] != 0x02 { return Err("kein ATR (Kennung $0296 fehlt)".into()); }
        let size = ((b[2] as usize) | (b[3] as usize) << 8 | (b[6] as usize) << 16) * 16;
        let ssz = b[4] as usize | (b[5] as usize) << 8;
        if ssz != SECTOR_SIZE || size != SECTORS * SECTOR_SIZE || b.len() != 16 + size { return Err(format!("nur 90K-Images mit 128-Byte-Sektoren unterstützt (Sektorgröße {ssz}, Größe {size})")); }
        Ok(Atr { sectors: b[16..].chunks(SECTOR_SIZE).map(|c| c.try_into().unwrap()).collect() })
    }

    /// Hat die Diskette ein DOS-2-Dateisystem? Eine bootbare Diskette ohne DOS (z. B. aus einer XEX-Datei) hat keine VTOC (DOS-Code ≠ 2).
    pub fn has_dos2(&self) -> bool { self.sector(VTOC)[0] == 2 }

    /// Prüft Konsistenz von VTOC, Verzeichnis und Sektorketten (Rückgabe: Liste von Problemen).
    /// Eine Boot-Diskette ohne Dateisystem hat nichts zu prüfen (kein Problem, sondern ein anderer Typ von Diskette).
    pub fn dos2_check(&self) -> Vec<String> {
        if !self.has_dos2() { return vec![]; }
        let mut problems = vec![];
        let mut used = vec![false; SECTORS + 1];
        for e in self.dos2_entries() {
            match self.dos2_read(&e.name) {
                Err(m) => problems.push(format!("{}: {m}", e.name)),
                Ok(_) => { let mut s = e.start as usize; while s != 0 { if used[s] { problems.push(format!("{}: Sektor {s} doppelt belegt", e.name)); } used[s] = true; s = ((self.sector(s)[125] & 3) as usize) << 8 | self.sector(s)[126] as usize; } }
            }
        }
        let mut free = 0;
        for s in 4..SECTORS {
            let reserved = (VTOC..DIR_FIRST + DIR_SECTORS).contains(&s);
            if reserved { if self.is_free(s) { problems.push(format!("Sektor {s} ist reserviert, aber als frei markiert")); } continue; }
            if self.is_free(s) { free += 1; if used[s] { problems.push(format!("Sektor {s} ist belegt, aber als frei markiert")); } }
            else if !used[s] { problems.push(format!("Sektor {s} ist als belegt markiert, gehört aber zu keiner Datei")); }
        }
        if free != self.free_sectors() { problems.push(format!("VTOC-Zähler {} stimmt nicht mit der Bitmap ({free}) überein", self.free_sectors())); }
        problems
    }
}
