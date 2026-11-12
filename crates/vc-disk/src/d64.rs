// SPDX-License-Identifier: GPL-3.0-or-later
//! D64-Diskimage (1541, 35 Spuren, 174 848 Bytes): BAM, Verzeichnis, PRG-Dateien.
//!
//! Spur 18 enthält BAM (18/0) und Verzeichnis (18/1…). Datensektoren: Bytes 0–1 = nächste Spur/Sektor, 254 Datenbytes;
//! im letzten Sektor ist die Spur 0 und Byte 1 der Index des letzten benutzten Bytes.

pub const TRACKS: usize = 35;
pub const SIZE: usize = 174_848;
const DIR_TRACK: usize = 18;

pub fn sectors_in_track(t: usize) -> usize { match t { 1..=17 => 21, 18..=24 => 19, 25..=30 => 18, _ => 17 } }
fn offset(t: usize, s: usize) -> usize { ((1..t).map(sectors_in_track).sum::<usize>() + s) * 256 }

#[derive(Debug, Clone, PartialEq)]
pub struct D64Entry { pub name: String, pub blocks: u16, pub track: u8, pub sector: u8, pub kind: &'static str }

#[derive(Debug, Clone, PartialEq)]
pub struct D64 { data: Vec<u8> }

pub(crate) fn petscii_name(s: &str) -> Result<[u8; 16], String> {
    if s.is_empty() || s.chars().count() > 16 { return Err(format!("Name '{s}': 1–16 Zeichen erforderlich")); }
    let mut b = [0xA0u8; 16];
    for (i, c) in s.chars().enumerate() {
        if !(' '..='~').contains(&c) || c == '"' || c == '*' || c == '?' || c == ',' || c == ':' { return Err(format!("Name '{s}': Zeichen '{c}' nicht erlaubt")); }
        b[i] = c.to_ascii_uppercase() as u8;
    }
    Ok(b)
}
pub(crate) fn name_text(b: &[u8]) -> String { b.iter().take_while(|c| **c != 0xA0).map(|c| *c as char).collect() }

impl D64 {
    /// Neu formatierte Diskette mit Namen und 2-Zeichen-ID.
    pub fn blank(name: &str, id: &str) -> Result<D64, String> {
        if id.chars().count() != 2 || !id.is_ascii() { return Err("ID muss aus genau 2 ASCII-Zeichen bestehen".into()); }
        let mut d = D64 { data: vec![0; SIZE] };
        for t in 1..=TRACKS { for s in 0..sectors_in_track(t) { d.set_free(t, s, true); } }
        d.set_free(DIR_TRACK, 0, false);
        d.set_free(DIR_TRACK, 1, false);
        let o = offset(DIR_TRACK, 0);
        d.data[o] = DIR_TRACK as u8; d.data[o + 1] = 1; d.data[o + 2] = b'A';
        d.data[o + 144..o + 160].copy_from_slice(&petscii_name(name)?);
        d.data[o + 160] = 0xA0; d.data[o + 161] = 0xA0;
        d.data[o + 162] = id.as_bytes()[0].to_ascii_uppercase(); d.data[o + 163] = id.as_bytes()[1].to_ascii_uppercase();
        d.data[o + 164] = 0xA0; d.data[o + 165] = b'2'; d.data[o + 166] = b'A';
        for i in 167..171 { d.data[o + i] = 0xA0; }
        let o = offset(DIR_TRACK, 1);
        d.data[o] = 0; d.data[o + 1] = 0xFF;
        Ok(d)
    }

    fn bam(&self, t: usize) -> usize { offset(DIR_TRACK, 0) + 4 * t }
    fn is_free(&self, t: usize, s: usize) -> bool { self.data[self.bam(t) + 1 + s / 8] & (1 << (s % 8)) != 0 }
    fn set_free(&mut self, t: usize, s: usize, free: bool) {
        let (b, was) = (self.bam(t), self.is_free(t, s));
        if free { self.data[b + 1 + s / 8] |= 1 << (s % 8); } else { self.data[b + 1 + s / 8] &= !(1 << (s % 8)); }
        match (was, free) { (false, true) => self.data[b] += 1, (true, false) => self.data[b] -= 1, _ => {} }
    }
    /// Roher Sektor (256 Bytes); `None` außerhalb der Diskette
    pub fn raw_sector(&self, t: usize, s: usize) -> Option<&[u8]> { if t == 0 || t > TRACKS || s >= sectors_in_track(t) { None } else { Some(self.sec(t, s)) } }
    /// Ist der Sektor laut BAM frei?
    pub fn sector_free(&self, t: usize, s: usize) -> bool { t >= 1 && t <= TRACKS && s < sectors_in_track(t) && self.is_free(t, s) }
    fn sec(&self, t: usize, s: usize) -> &[u8] { let o = offset(t, s); &self.data[o..o + 256] }
    fn sec_mut(&mut self, t: usize, s: usize) -> &mut [u8] { let o = offset(t, s); &mut self.data[o..o + 256] }

    /// Freie Blöcke wie in „664 BLOCKS FREE“ (Spur 18 zählt nicht mit).
    pub fn free_blocks(&self) -> usize { (1..=TRACKS).filter(|t| *t != DIR_TRACK).map(|t| self.data[self.bam(t)] as usize).sum() }
    pub fn disk_name(&self) -> String { name_text(&self.data[offset(DIR_TRACK, 0) + 144..offset(DIR_TRACK, 0) + 160]) }

    /// Nächster freier Sektor: Datenspuren in der Reihenfolge 17…1, 19…35, Interleave 10 innerhalb der Spur.
    fn alloc_data(&mut self, prev: Option<(usize, usize)>) -> Option<(usize, usize)> {
        let order: Vec<usize> = (1..=17).rev().chain(19..=TRACKS).collect();
        let mut tracks: Vec<usize> = order.clone();
        if let Some((pt, _)) = prev { if let Some(i) = order.iter().position(|t| *t == pt) { tracks = order[i..].iter().chain(order[..i].iter()).copied().collect(); } }
        for t in tracks {
            let n = sectors_in_track(t);
            let start = match prev { Some((pt, ps)) if pt == t => (ps + 10) % n, _ => 0 };
            for k in 0..n { let s = (start + k) % n; if self.is_free(t, s) { self.set_free(t, s, false); return Some((t, s)); } }
        }
        None
    }

    fn entry_slots(&self) -> Vec<(usize, usize)> {
        let mut v = vec![];
        let (mut t, mut s) = (DIR_TRACK, 1);
        loop {
            for i in 0..8 { v.push((offset(t, s) + i * 32, 0)); }
            let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize);
            if nt == 0 { break; }
            t = nt; s = ns;
            if v.len() > 8 * 40 { break; }
        }
        v
    }

    pub fn entries(&self) -> Vec<D64Entry> {
        self.entry_slots().into_iter().filter_map(|(o, _)| {
            let e = &self.data[o..o + 32];
            if e[2] == 0 { return None; }
            let kind = match e[2] & 7 { 0 => "DEL", 1 => "SEQ", 2 => "PRG", 3 => "USR", 4 => "REL", _ => "???" };
            Some(D64Entry { name: name_text(&e[5..21]), blocks: e[30] as u16 | (e[31] as u16) << 8, track: e[3], sector: e[4], kind })
        }).collect()
    }

    /// Legt eine PRG-Datei an (Datei mit Ladeadresse als erste zwei Bytes).
    pub fn add_prg(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let n = petscii_name(name)?;
        if data.is_empty() { return Err(format!("'{name}': leere Dateien werden nicht unterstützt")); }
        if self.entries().iter().any(|e| e.name == name_text(&n)) { return Err(format!("'{name}' existiert bereits")); }
        let need = data.len().div_ceil(254);
        if need > self.free_blocks() { return Err(format!("Diskette voll: '{name}' braucht {need} Blöcke, frei sind {}", self.free_blocks())); }
        // Verzeichnisplatz suchen (bei Bedarf neuen Verzeichnissektor mit Interleave 3 anlegen)
        let mut slot = self.entry_slots().into_iter().map(|(o, _)| o).find(|o| self.data[o + 2] == 0);
        if slot.is_none() {
            let mut last = (DIR_TRACK, 1);
            while self.sec(last.0, last.1)[0] != 0 { last = (self.sec(last.0, last.1)[0] as usize, self.sec(last.0, last.1)[1] as usize); }
            let n_s = sectors_in_track(DIR_TRACK);
            let s = (0..n_s).map(|k| (last.1 + 3 + k) % n_s).find(|s| self.is_free(DIR_TRACK, *s)).ok_or("Verzeichnis voll")?;
            self.set_free(DIR_TRACK, s, false);
            let sec = self.sec_mut(last.0, last.1); sec[0] = DIR_TRACK as u8; sec[1] = s as u8;
            let sec = self.sec_mut(DIR_TRACK, s); sec.fill(0); sec[1] = 0xFF;
            slot = Some(offset(DIR_TRACK, s));
        }
        let slot = slot.unwrap();
        let mut chain = vec![];
        let mut prev = None;
        for _ in 0..need { let s = self.alloc_data(prev).ok_or("keine freien Sektoren")?; chain.push(s); prev = Some(s); }
        for (k, &(t, s)) in chain.iter().enumerate() {
            let chunk = &data[k * 254..((k + 1) * 254).min(data.len())];
            let sec = self.sec_mut(t, s);
            sec.fill(0);
            sec[2..2 + chunk.len()].copy_from_slice(chunk);
            match chain.get(k + 1) { Some(&(nt, ns)) => { sec[0] = nt as u8; sec[1] = ns as u8; } None => { sec[0] = 0; sec[1] = (chunk.len() + 1) as u8; } }
        }
        let e = &mut self.data[slot..slot + 32];
        let (keep0, keep1) = (e[0], e[1]);
        e.fill(0);
        e[0] = keep0; e[1] = keep1;
        e[2] = 0x82;
        e[3] = chain[0].0 as u8; e[4] = chain[0].1 as u8;
        e[5..21].copy_from_slice(&n);
        e[30] = (need & 0xFF) as u8; e[31] = (need >> 8) as u8;
        Ok(())
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, String> {
        let e = self.entries().into_iter().find(|e| e.name == name.to_ascii_uppercase()).ok_or(format!("'{name}' nicht gefunden"))?;
        let (mut t, mut s, mut out, mut n) = (e.track as usize, e.sector as usize, vec![], 0);
        loop {
            if t == 0 || t > TRACKS || s >= sectors_in_track(t) { return Err(format!("ungültiger Sektor {t}/{s}")); }
            let sec = self.sec(t, s);
            n += 1;
            if n > 700 { return Err("Sektorkette zu lang (Schleife?)".into()); }
            if sec[0] == 0 { let last = sec[1] as usize; if !(1..=255).contains(&last) { return Err(format!("ungültiges Ende in {t}/{s}")); } out.extend_from_slice(&sec[2..=last]); break; }
            out.extend_from_slice(&sec[2..]);
            t = sec[0] as usize; s = sec[1] as usize;
        }
        if n != e.blocks as usize { return Err(format!("Kette hat {n} Blöcke, Verzeichnis meldet {}", e.blocks)); }
        Ok(out)
    }

    pub fn to_bytes(&self) -> Vec<u8> { self.data.clone() }
    pub fn from_bytes(b: &[u8]) -> Result<D64, String> {
        if b.len() != SIZE { return Err(format!("nur D64 mit 35 Spuren ohne Fehlerinfo ({SIZE} Bytes) unterstützt, Datei hat {}", b.len())); }
        Ok(D64 { data: b.to_vec() })
    }

    /// Prüft BAM-Zähler, Bitmaps und Dateiketten (Probleme als Liste).
    pub fn check(&self) -> Vec<String> {
        let mut problems = vec![];
        let mut used = vec![vec![false; 22]; TRACKS + 1];
        used[DIR_TRACK][0] = true;
        let (mut t, mut s) = (DIR_TRACK, 1);
        loop { used[t][s] = true; let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize); if nt == 0 { break; } t = nt; s = ns; }
        for e in self.entries() {
            let (mut t, mut s) = (e.track as usize, e.sector as usize);
            loop {
                if t == 0 || t > TRACKS || s >= sectors_in_track(t) { problems.push(format!("{}: ungültiger Sektor {t}/{s}", e.name)); break; }
                if used[t][s] { problems.push(format!("{}: Sektor {t}/{s} doppelt belegt", e.name)); break; }
                used[t][s] = true;
                let (nt, ns) = (self.sec(t, s)[0] as usize, self.sec(t, s)[1] as usize);
                if nt == 0 { break; }
                t = nt; s = ns;
            }
            if let Err(m) = self.read(&e.name) { problems.push(format!("{}: {m}", e.name)); }
        }
        for t in 1..=TRACKS {
            let mut free = 0;
            for s in 0..sectors_in_track(t) {
                if self.is_free(t, s) { free += 1; if used[t][s] { problems.push(format!("{t}/{s}: belegt, aber in der BAM frei")); } }
                else if !used[t][s] { problems.push(format!("{t}/{s}: in der BAM belegt, gehört aber zu keiner Datei")); }
            }
            if free != self.data[self.bam(t)] as usize { problems.push(format!("Spur {t}: BAM-Zähler {} ≠ Bitmap {free}", self.data[self.bam(t)])); }
        }
        problems
    }
}
