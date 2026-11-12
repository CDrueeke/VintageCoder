// SPDX-License-Identifier: GPL-3.0-or-later
//! D71 (1571, doppelseitig, 70 Spuren, 349 696 Bytes) und D81 (1581, 80 Spuren à 40 Sektoren, 819 200 Bytes): BAM, Verzeichnis, PRG-Dateien.
//!
//! D71: Spur 18 wie bei der D64 (BAM Spuren 1–35 in 18/0, Byte 3 = $80 für „doppelseitig“); Spuren 36–70: Freizähler in 18/0 ab $DD,
//! Bitmaps (3 Bytes je Spur) in 53/0; Spur 53 ist vollständig belegt. D81: Kopf 40/0, BAM 40/1 (Spuren 1–40) und 40/2 (41–80,
//! je 6 Bytes: Freizähler + 5 Bytes Bitmap ab Offset 16), Verzeichnis ab 40/3. Dateiketten und Verzeichniseinträge wie bei der D64.
use crate::d64::{name_text, petscii_name, D64Entry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind { D71, D81 }

#[derive(Debug, Clone, PartialEq)]
pub struct Disk { kind: Kind, data: Vec<u8> }

fn zone(t: usize) -> usize { match t { 1..=17 => 21, 18..=24 => 19, 25..=30 => 18, _ => 17 } }

impl Kind {
    pub fn size(self) -> usize { match self { Kind::D71 => 349_696, Kind::D81 => 819_200 } }
    pub fn tracks(self) -> usize { match self { Kind::D71 => 70, Kind::D81 => 80 } }
    pub fn sectors(self, t: usize) -> usize { match self { Kind::D71 => zone(if t > 35 { t - 35 } else { t }), Kind::D81 => 40 } }
    fn dir_track(self) -> usize { match self { Kind::D71 => 18, Kind::D81 => 40 } }
    fn first_dir(self) -> usize { match self { Kind::D71 => 1, Kind::D81 => 3 } }
    fn interleave(self) -> usize { match self { Kind::D71 => 6, Kind::D81 => 1 } }
    /// Spuren, die keine Dateidaten aufnehmen
    fn reserved(self, t: usize) -> bool { t == self.dir_track() || (self == Kind::D71 && t == 53) }
    pub fn from_extension(name: &str) -> Option<Kind> { let n = name.to_ascii_lowercase(); if n.ends_with(".d71") { Some(Kind::D71) } else if n.ends_with(".d81") { Some(Kind::D81) } else { None } }
}

impl Disk {
    pub fn kind(&self) -> Kind { self.kind }
    fn offset(&self, t: usize, s: usize) -> usize { ((1..t).map(|x| self.kind.sectors(x)).sum::<usize>() + s) * 256 }
    /// Roher Sektor (256 Bytes); `None` außerhalb der Diskette
    pub fn raw_sector(&self, t: usize, s: usize) -> Option<&[u8]> { if t == 0 || t > self.kind.tracks() || s >= self.kind.sectors(t) { None } else { Some(self.sec(t, s)) } }
    /// Ist der Sektor laut BAM frei?
    pub fn sector_free(&self, t: usize, s: usize) -> bool { t >= 1 && t <= self.kind.tracks() && s < self.kind.sectors(t) && self.is_free(t, s) }
    fn sec(&self, t: usize, s: usize) -> &[u8] { let o = self.offset(t, s); &self.data[o..o + 256] }
    fn sec_mut(&mut self, t: usize, s: usize) -> &mut [u8] { let o = self.offset(t, s); &mut self.data[o..o + 256] }
    /// (Position des Freizählers, Position des Bitmap-Bytes 0) der Spur
    fn bam(&self, t: usize) -> (usize, usize) {
        match self.kind {
            Kind::D71 if t <= 35 => { let b = self.offset(18, 0) + 4 * t; (b, b + 1) }
            Kind::D71 => (self.offset(18, 0) + 0xDD + (t - 36), self.offset(53, 0) + 3 * (t - 36)),
            Kind::D81 => { let b = self.offset(40, if t <= 40 { 1 } else { 2 }) + 16 + 6 * ((t - 1) % 40); (b, b + 1) }
        }
    }
    fn is_free(&self, t: usize, s: usize) -> bool { self.data[self.bam(t).1 + s / 8] & (1 << (s % 8)) != 0 }
    fn set_free(&mut self, t: usize, s: usize, free: bool) {
        let (c, b) = self.bam(t); let was = self.is_free(t, s);
        if free { self.data[b + s / 8] |= 1 << (s % 8); } else { self.data[b + s / 8] &= !(1 << (s % 8)); }
        match (was, free) { (false, true) => self.data[c] += 1, (true, false) => self.data[c] -= 1, _ => {} }
    }

    pub fn blank(kind: Kind, name: &str, id: &str) -> Result<Disk, String> {
        if id.chars().count() != 2 || !id.is_ascii() { return Err("ID muss aus genau 2 ASCII-Zeichen bestehen".into()); }
        let mut d = Disk { kind, data: vec![0; kind.size()] };
        let (n, id) = (petscii_name(name)?, [id.as_bytes()[0].to_ascii_uppercase(), id.as_bytes()[1].to_ascii_uppercase()]);
        for t in 1..=kind.tracks() { for s in 0..kind.sectors(t) { d.set_free(t, s, true); } }
        let dt = kind.dir_track();
        match kind {
            Kind::D71 => {
                { let o = d.offset(18, 0); let h = &mut d.data[o..o + 256]; h[0] = 18; h[1] = 1; h[2] = b'A'; h[3] = 0x80;
                  h[144..160].copy_from_slice(&n); h[160] = 0xA0; h[161] = 0xA0; h[162] = id[0]; h[163] = id[1]; h[164] = 0xA0; h[165] = b'2'; h[166] = b'A'; for i in 167..171 { h[i] = 0xA0; } }
                for s in 0..zone(18) { if s < 2 { d.set_free(18, s, false); } }
                for s in 0..d.kind.sectors(53) { d.set_free(53, s, false); }
            }
            Kind::D81 => {
                { let o = d.offset(40, 0); let h = &mut d.data[o..o + 256]; h[0] = 40; h[1] = 3; h[2] = b'D'; h[3] = 0;
                  h[4..20].copy_from_slice(&n); h[20] = 0xA0; h[21] = 0xA0; h[22] = id[0]; h[23] = id[1]; h[24] = 0xA0; h[25] = b'3'; h[26] = b'D'; h[27] = 0xA0; h[28] = 0xA0; }
                for (s, (nt, ns)) in [(1usize, (40u8, 2u8)), (2, (0, 0xFF))] {
                    let o = d.offset(40, s); let h = &mut d.data[o..o + 256];
                    h[0] = nt; h[1] = ns; h[2] = b'D'; h[3] = 0xBB; h[4] = id[0]; h[5] = id[1]; h[6] = 0xC0;
                }
                for s in 0..4 { d.set_free(40, s, false); }
            }
        }
        if kind == Kind::D71 { d.set_free(dt, 1, false); }
        let o = d.offset(dt, kind.first_dir()); d.data[o] = 0; d.data[o + 1] = 0xFF;
        Ok(d)
    }

    pub fn disk_name(&self) -> String {
        match self.kind { Kind::D71 => { let o = self.offset(18, 0); name_text(&self.data[o + 144..o + 160]) } Kind::D81 => { let o = self.offset(40, 0); name_text(&self.data[o + 4..o + 20]) } }
    }
    /// Freie Blöcke wie in „1328 BLOCKS FREE“ / „3160 BLOCKS FREE“ (Verzeichnisspur und bei der D71 Spur 53 zählen nicht mit).
    pub fn free_blocks(&self) -> usize { (1..=self.kind.tracks()).filter(|t| !self.kind.reserved(*t)).map(|t| self.data[self.bam(t).0] as usize).sum() }

    fn alloc_order(&self) -> Vec<usize> {
        match self.kind { Kind::D71 => (1..=17).rev().chain(19..=35).chain((36..=52).rev()).chain(54..=70).collect(), Kind::D81 => (1..=39).rev().chain(41..=80).collect() }
    }
    fn alloc_data(&mut self, prev: Option<(usize, usize)>) -> Option<(usize, usize)> {
        let order = self.alloc_order(); let mut tracks = order.clone();
        if let Some((pt, _)) = prev { if let Some(i) = order.iter().position(|t| *t == pt) { tracks = order[i..].iter().chain(order[..i].iter()).copied().collect(); } }
        for t in tracks {
            let n = self.kind.sectors(t);
            let start = match prev { Some((pt, ps)) if pt == t => (ps + self.kind.interleave().max(1)) % n, _ => 0 };
            for k in 0..n { let s = (start + k) % n; if self.is_free(t, s) { self.set_free(t, s, false); return Some((t, s)); } }
        }
        None
    }
    fn entry_slots(&self) -> Vec<usize> {
        let mut v = vec![]; let (mut t, mut s) = (self.kind.dir_track(), self.kind.first_dir());
        loop {
            for i in 0..8 { v.push(self.offset(t, s) + i * 32); }
            let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize);
            if nt == 0 || nt > self.kind.tracks() || ns >= self.kind.sectors(nt) { break; }
            t = nt; s = ns; if v.len() > 8 * 400 { break; }
        }
        v
    }
    pub fn entries(&self) -> Vec<D64Entry> {
        self.entry_slots().into_iter().filter_map(|o| {
            let e = &self.data[o..o + 32]; if e[2] == 0 { return None; }
            let kind = match e[2] & 7 { 0 => "DEL", 1 => "SEQ", 2 => "PRG", 3 => "USR", 4 => "REL", _ => "???" };
            Some(D64Entry { name: name_text(&e[5..21]), blocks: e[30] as u16 | (e[31] as u16) << 8, track: e[3], sector: e[4], kind })
        }).collect()
    }

    pub fn add_prg(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let n = petscii_name(name)?;
        if data.is_empty() { return Err(format!("'{name}': leere Dateien werden nicht unterstützt")); }
        if self.entries().iter().any(|e| e.name == name_text(&n)) { return Err(format!("'{name}' existiert bereits")); }
        let need = data.len().div_ceil(254);
        if need > self.free_blocks() { return Err(format!("Diskette voll: '{name}' braucht {need} Blöcke, frei sind {}", self.free_blocks())); }
        let mut slot = self.entry_slots().into_iter().find(|o| self.data[o + 2] == 0);
        if slot.is_none() {
            let dt = self.kind.dir_track(); let mut last = (dt, self.kind.first_dir());
            while self.sec(last.0, last.1)[0] != 0 { last = (self.sec(last.0, last.1)[0] as usize, self.sec(last.0, last.1)[1] as usize); }
            let n_s = self.kind.sectors(dt); let il = if self.kind == Kind::D71 { 3 } else { 1 };
            let s = (0..n_s).map(|k| (last.1 + il + k) % n_s).find(|s| self.is_free(dt, *s)).ok_or("Verzeichnis voll")?;
            self.set_free(dt, s, false);
            let sec = self.sec_mut(last.0, last.1); sec[0] = dt as u8; sec[1] = s as u8;
            let sec = self.sec_mut(dt, s); sec.fill(0); sec[1] = 0xFF;
            slot = Some(self.offset(dt, s));
        }
        let slot = slot.unwrap();
        let (mut chain, mut prev) = (vec![], None);
        for _ in 0..need { let s = self.alloc_data(prev).ok_or("keine freien Sektoren")?; chain.push(s); prev = Some(s); }
        for (k, &(t, s)) in chain.iter().enumerate() {
            let chunk = &data[k * 254..((k + 1) * 254).min(data.len())];
            let sec = self.sec_mut(t, s); sec.fill(0); sec[2..2 + chunk.len()].copy_from_slice(chunk);
            match chain.get(k + 1) { Some(&(nt, ns)) => { sec[0] = nt as u8; sec[1] = ns as u8; } None => { sec[0] = 0; sec[1] = (chunk.len() + 1) as u8; } }
        }
        let e = &mut self.data[slot..slot + 32]; let (k0, k1) = (e[0], e[1]);
        e.fill(0); e[0] = k0; e[1] = k1; e[2] = 0x82; e[3] = chain[0].0 as u8; e[4] = chain[0].1 as u8; e[5..21].copy_from_slice(&n);
        e[30] = (need & 0xFF) as u8; e[31] = (need >> 8) as u8;
        Ok(())
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        let e = self.entries().into_iter().find(|e| e.name == name.to_ascii_uppercase()).ok_or(format!("'{name}' nicht gefunden"))?;
        let (mut t, mut s, mut out, mut n) = (e.track as usize, e.sector as usize, vec![], 0);
        loop {
            if t == 0 || t > self.kind.tracks() || s >= self.kind.sectors(t) { return Err(format!("ungültiger Sektor {t}/{s}")); }
            let sec = self.sec(t, s); n += 1;
            if n > 3200 { return Err("Sektorkette zu lang (Schleife?)".into()); }
            if sec[0] == 0 { let last = sec[1] as usize; if !(1..=255).contains(&last) { return Err(format!("ungültiges Ende in {t}/{s}")); } out.extend_from_slice(&sec[2..=last]); break; }
            out.extend_from_slice(&sec[2..]); t = sec[0] as usize; s = sec[1] as usize;
        }
        if n != e.blocks as usize { return Err(format!("Kette hat {n} Blöcke, Verzeichnis meldet {}", e.blocks)); }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> { self.data.clone() }
    pub fn from_bytes(kind: Kind, b: &[u8]) -> Result<Disk, String> {
        if b.len() != kind.size() { return Err(format!("{:?} ohne Fehlerinfo hat {} Bytes, Datei hat {}", kind, kind.size(), b.len())); }
        Ok(Disk { kind, data: b.to_vec() })
    }
    pub fn from_file_bytes(name: &str, b: &[u8]) -> Result<Disk, String> { Disk::from_bytes(Kind::from_extension(name).ok_or("Endung .d71 oder .d81 erwartet")?, b) }

    /// Prüft BAM-Zähler, Bitmaps und Dateiketten
    pub fn check(&self) -> Vec<String> {
        let k = self.kind; let mut problems = vec![]; let mut used = vec![vec![false; 41]; k.tracks() + 1];
        let dt = k.dir_track();
        match k { Kind::D71 => { used[18][0] = true; for s in 0..zone(18) { used[53][s] = true; } } Kind::D81 => { for s in 0..3 { used[40][s] = true; } } }
        let (mut t, mut s) = (dt, k.first_dir());
        loop { used[t][s] = true; let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize); if nt == 0 || nt > k.tracks() || ns >= k.sectors(nt) { break; } t = nt; s = ns; }
        for e in self.entries() {
            let (mut t, mut s) = (e.track as usize, e.sector as usize);
            loop {
                if t == 0 || t > k.tracks() || s >= k.sectors(t) { problems.push(format!("{}: ungültiger Sektor {t}/{s}", e.name)); break; }
                if used[t][s] { problems.push(format!("{}: Sektor {t}/{s} doppelt belegt", e.name)); break; }
                used[t][s] = true; let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize); if nt == 0 { break; } t = nt; s = ns;
            }
            if let Err(m) = self.read(&e.name) { problems.push(format!("{}: {m}", e.name)); }
        }
        for t in 1..=k.tracks() {
            let mut free = 0;
            for s in 0..k.sectors(t) {
                if self.is_free(t, s) { free += 1; if used[t][s] { problems.push(format!("{t}/{s}: belegt, aber in der BAM frei")); } }
                else if !used[t][s] { problems.push(format!("{t}/{s}: in der BAM belegt, gehört aber zu keiner Datei")); }
            }
            if free != self.data[self.bam(t).0] as usize { problems.push(format!("Spur {t}: BAM-Zähler {} ≠ Bitmap {free}", self.data[self.bam(t).0])); }
        }
        problems
    }
}
