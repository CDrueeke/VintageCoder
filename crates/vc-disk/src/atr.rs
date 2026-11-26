// SPDX-License-Identifier: GPL-3.0-or-later
//! ATR-Container und die Dateisysteme Atari DOS 2.0S, DOS 2.5 und MyDOS.
//!
//! Der Container kennt zwei Sektorgrößen (128 und 256 Byte) und zwei Anordnungen der Boot-Sektoren: Nach dem ATR-Standard sind die Sektoren 1–3 immer
//! 128 Byte lang (auch bei 256-Byte-Sektoren), manche Werkzeuge legen aber alle Sektoren gleich lang ab. Beim Lesen entscheidet die Dateilänge, beim
//! Schreiben bleibt die Anordnung des Abbilds erhalten (ein unverändert gespeichertes Abbild ist byte-gleich, auch der Kopf mit seinen Zusatzbytes).
//!
//! Layout einer DOS-2.0S-Diskette: Sektor 1–3 Boot, 360 VTOC, 361–368 Verzeichnis, Daten in 4–359 und 369–719.
//! DOS 2.5 (Enhanced Density, 1040 Sektoren) ergänzt eine zweite VTOC in Sektor 1024 für die Sektoren ab 720.
//! MyDOS belegt Sektor 720 mit, verwaltet große Abbilder mit mehreren VTOC-Sektoren (abwärts ab 359) und kennt Unterverzeichnisse (Flag `$10`, acht aufeinander folgende Sektoren).
//! Datensektor: Datenbytes, dann die letzten drei Bytes: (Dateinummer << 2) | Folgesektor hoch, Folgesektor niedrig, Anzahl Datenbytes.

pub const SECTORS: usize = 720;
pub const SECTOR_SIZE: usize = 128;
const VTOC: usize = 360;
const VTOC2: usize = 1024;
const DIR_FIRST: usize = 361;
const DIR_SECTORS: usize = 8;

/// Art des Dateisystems, an VTOC, Größe und Boot-Sektor erkannt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DosKind { Dos2, Dos25, MyDos, Keins }
impl DosKind {
    pub fn name(self) -> &'static str { match self { DosKind::Dos2 => "Atari DOS 2", DosKind::Dos25 => "Atari DOS 2.5", DosKind::MyDos => "Atari MyDOS", DosKind::Keins => "" } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atr {
    /// Kopf (16 Byte) wie gelesen; neue Abbilder bekommen einen erzeugten Kopf
    header: [u8; 16],
    ssz: usize,
    /// Sektoren 1–3 sind 128 Byte lang, die übrigen `ssz` (nur bei `ssz` > 128 von Bedeutung)
    boot128: bool,
    /// Sektoren 1..=n (Index 0 = Sektor 1)
    sectors: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dos2Entry { pub name: String, pub sectors: u16, pub start: u16, pub locked: bool, /** Unterverzeichnis (MyDOS) */ pub dir: bool }

struct Raw { idx: usize, flag: u8, count: usize, start: usize, name: String }

fn header_for(n: usize, ssz: usize, boot128: bool) -> [u8; 16] {
    let size = if boot128 && ssz > 128 { 3 * 128 + (n - 3) * ssz } else { n * ssz };
    let para = size / 16;
    let mut h = [0u8; 16];
    h[..7].copy_from_slice(&[0x96, 0x02, (para & 0xFF) as u8, ((para >> 8) & 0xFF) as u8, (ssz & 0xFF) as u8, (ssz >> 8) as u8, ((para >> 16) & 0xFF) as u8]);
    h
}

impl Atr {
    /// Leere, unformatierte 90K-Diskette (alles 0).
    pub fn blank() -> Atr { Atr::blank_with(SECTORS, SECTOR_SIZE, false) }
    /// Leeres Abbild mit gewählter Geometrie
    pub fn blank_with(n: usize, ssz: usize, boot128: bool) -> Atr {
        let boot128 = boot128 && ssz > 128;
        let sectors = (1..=n).map(|s| vec![0; if boot128 && s <= 3 { 128 } else { ssz }]).collect();
        Atr { header: header_for(n, ssz, boot128), ssz, boot128, sectors }
    }

    pub fn sector_size(&self) -> usize { self.ssz }
    pub fn sector_count(&self) -> usize { self.sectors.len() }
    /// Sind die Boot-Sektoren 1–3 kürzer (128 Byte) als die übrigen Sektoren?
    pub fn boot_128(&self) -> bool { self.boot128 }

    pub fn sector(&self, n: usize) -> &[u8] { &self.sectors[n - 1] }
    pub fn sector_mut(&mut self, n: usize) -> &mut [u8] { &mut self.sectors[n - 1] }
    fn has(&self, n: usize) -> bool { n >= 1 && n <= self.sectors.len() }

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

    fn set_free(vtoc: &mut [u8], s: usize, free: bool) {
        let (byte, bit) = (10 + s / 8, 7 - s % 8);
        if free { vtoc[byte] |= 1 << bit } else { vtoc[byte] &= !(1 << bit) }
    }

    /// Dateisystem des Abbilds (`Keins`: keine VTOC, z. B. eine Boot-Diskette)
    pub fn kind(&self) -> DosKind {
        if !self.has(VTOC) || !matches!(self.sector(VTOC)[0], 2 | 3) { return DosKind::Keins; }
        let n = self.sectors.len();
        if self.ssz > 128 || self.sector(1)[0] == 0x4D || (n > 720 && n != 1040) { DosKind::MyDos }
        else if n == 1040 && self.has(VTOC2) { DosKind::Dos25 }
        else { DosKind::Dos2 }
    }

    /// Ort des Bitmap-Bits von Sektor `s`: (Sektor der VTOC, Byte)
    fn bit_pos(&self, s: usize) -> Option<(usize, usize)> {
        match self.kind() {
            DosKind::Keins => None,
            DosKind::Dos2 => (s >= 1 && s < SECTORS).then(|| (VTOC, 10 + s / 8)),
            DosKind::Dos25 => if s < 1 { None } else if s < SECTORS { Some((VTOC, 10 + s / 8)) } else if s < 1024 { Some((VTOC2, 84 + (s - SECTORS) / 8)) } else { None },
            DosKind::MyDos => {
                if s < 1 { return None; }
                let cap0 = (self.ssz - 10) * 8;
                if s < cap0 { Some((VTOC, 10 + s / 8)) } else { let off = s - cap0; let k = 1 + off / (self.ssz * 8); (VTOC > k).then(|| (VTOC - k, (off % (self.ssz * 8)) / 8)) }
            }
        }
    }
    /// Höchste Sektornummer, für die die VTOC ein Bit führt
    fn last_mapped(&self) -> usize {
        match self.kind() { DosKind::Keins => 0, DosKind::Dos2 => SECTORS - 1, DosKind::Dos25 => 1023.min(self.sectors.len()), DosKind::MyDos => self.sectors.len() }
    }
    /// Ist Sektor `n` laut VTOC frei? (Sektoren ohne Bitmap-Bit, etwa Sektor 720 bei DOS 2, sind nie frei.)
    pub fn sector_free(&self, n: usize) -> bool { self.is_free(n) }
    fn is_free(&self, s: usize) -> bool {
        if !self.has(s) { return false; }
        match self.bit_pos(s) { Some((sec, byte)) if self.has(sec) && byte < self.sector(sec).len() => self.sector(sec)[byte] & (1 << (7 - s % 8)) != 0, _ => false }
    }

    /// Freie Sektoren: bei DOS 2 und MyDOS der Zähler der VTOC, bei DOS 2.5 aus der Bitmap gezählt (der Zähler steht dort an zwei Stellen)
    pub fn free_sectors(&self) -> usize {
        match self.kind() {
            DosKind::Dos25 => (1..=self.last_mapped()).filter(|&s| self.is_free(s)).count(),
            DosKind::Keins => 0,
            _ => self.sector(VTOC)[3] as usize | (self.sector(VTOC)[4] as usize) << 8,
        }
    }

    fn valid_name(name: &str) -> Result<([u8; 8], [u8; 3]), String> {
        let (base, ext) = name.split_once('.').unwrap_or((name, ""));
        let ok = |s: &str, max: usize| s.len() <= max && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if base.is_empty() || !ok(base, 8) || !ok(ext, 3) || !base.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(vc_i18n::t!("disk-ungueltiger-dateiname-name-1-8-zeichen", name = &name));
        }
        let mut b = [b' '; 8];
        let mut e = [b' '; 3];
        b[..base.len()].copy_from_slice(base.as_bytes());
        e[..ext.len()].copy_from_slice(ext.as_bytes());
        Ok((b, e))
    }

    /// Verzeichnis ab Sektor `first` (`count` aufeinander folgende Sektoren): belegte, nicht gelöschte Einträge mit ihrer Nummer im Verzeichnis
    fn read_dir(&self, first: usize, count: usize) -> Vec<Raw> {
        let mut v = vec![];
        let per = self.ssz / 16;
        for k in 0..count {
            let s = first + k; if !self.has(s) { break; }
            for i in 0..per {
                let e = &self.sector(s)[i * 16..i * 16 + 16];
                if e[0] & 0x80 != 0 || (e[0] & 0x40 == 0 && e[0] & 0x10 == 0) { continue; }
                let base = String::from_utf8_lossy(&e[5..13]).trim_end().to_string();
                let ext = String::from_utf8_lossy(&e[13..16]).trim_end().to_string();
                v.push(Raw { idx: k * per + i, flag: e[0], count: e[1] as usize | (e[2] as usize) << 8, start: e[3] as usize | (e[4] as usize) << 8, name: if ext.is_empty() { base } else { format!("{base}.{ext}") } });
            }
        }
        v
    }
    /// Alle Einträge samt Unterverzeichnissen (MyDOS) in der Reihenfolge des Verzeichnisses; Pfade mit `/`, Unterverzeichnisse selbst zuerst
    fn baum(&self) -> Vec<(String, Raw)> {
        fn rec(a: &Atr, first: usize, prefix: &str, depth: usize, seen: &mut Vec<usize>, out: &mut Vec<(String, Raw)>) {
            for r in a.read_dir(first, DIR_SECTORS) {
                let path = if prefix.is_empty() { r.name.clone() } else { format!("{prefix}/{}", r.name) };
                let (start, sub) = (r.start, r.flag & 0x10 != 0);
                out.push((path.clone(), r));
                if sub && depth < 8 && a.kind() == DosKind::MyDos && a.has(start) && !seen.contains(&start) { seen.push(start); rec(a, start, &path, depth + 1, seen, out); }
            }
        }
        let mut out = vec![]; if self.kind() == DosKind::Keins || !self.has(DIR_FIRST) { return out; }
        rec(self, DIR_FIRST, "", 0, &mut vec![DIR_FIRST], &mut out); out
    }

    /// Verzeichniseinträge (nicht gelöschte), bei MyDOS einschließlich der Unterverzeichnisse (Pfad mit `/`).
    pub fn dos2_entries(&self) -> Vec<Dos2Entry> {
        self.baum().into_iter().map(|(name, r)| Dos2Entry { name, sectors: r.count as u16, start: r.start as u16, locked: r.flag & 0x20 != 0, dir: r.flag & 0x10 != 0 }).collect()
    }
    /// Sektoren, die zu Unterverzeichnissen gehören (je acht ab dem Startsektor)
    pub fn subdir_sectors(&self) -> Vec<usize> { self.baum().into_iter().filter(|(_, r)| r.flag & 0x10 != 0 && self.has(r.start)).flat_map(|(_, r)| r.start..r.start + DIR_SECTORS).filter(|&s| self.has(s)).collect() }
    /// Sektoren des Dateisystems selbst: Boot, VTOC(s), Verzeichnis, Unterverzeichnisse
    pub fn system_sectors(&self) -> Vec<usize> {
        if self.kind() == DosKind::Keins { return vec![]; }
        let mut v: Vec<usize> = (1..=3).chain([VTOC]).chain(DIR_FIRST..DIR_FIRST + DIR_SECTORS).collect();
        match self.kind() {
            DosKind::Dos2 => v.push(SECTORS),
            DosKind::Dos25 => v.push(VTOC2),
            DosKind::MyDos => { let cap0 = (self.ssz - 10) * 8; let mut k = 1; while cap0 + (k - 1) * self.ssz * 8 < self.sectors.len() && VTOC > k { v.push(VTOC - k); k += 1; } }
            DosKind::Keins => {}
        }
        v.extend(self.subdir_sectors()); v.retain(|&s| self.has(s)); v
    }
    /// Trailer eines Datensektors: (Dateinummer, Folgesektor, Anzahl Datenbytes)
    pub fn link(&self, s: usize) -> (usize, usize, usize) {
        let sec = self.sector(s); let l = sec.len();
        ((sec[l - 3] >> 2) as usize, ((sec[l - 3] & 3) as usize) << 8 | sec[l - 2] as usize, sec[l - 1] as usize)
    }
    /// Reichen die 10-Bit-Sektorzeiger für dieses Abbild? Größere MyDOS-Abbilder (über 1023 Sektoren) lesen wir noch nicht.
    fn chains_ok(&self) -> bool { self.sectors.len() <= 1023 || self.kind() == DosKind::Dos25 }

    /// Legt eine Datei an (Sektoren aufsteigend ab 4, Verzeichnis ab Eintrag 0). Zurzeit nur für Atari DOS 2 (90K, 128-Byte-Sektoren).
    /// Datei der Chain eines Eintrags: die Sektoren ab `start`, geprüft gegen die Dateinummer `idx % 64`
    fn kette_sektoren(&self, start: usize, idx: usize) -> Result<Vec<usize>, String> {
        let (mut s, mut v) = (start, vec![]);
        while s != 0 {
            if !self.has(s) || v.len() > self.sectors.len() { return Err(vc_i18n::t!("disk-verschieben-kette-defekt").into()); }
            let (fno, next, _) = self.link(s);
            if fno != idx % 64 { return Err(vc_i18n::t!("disk-verschieben-kette-defekt").into()); }
            v.push(s); s = next;
        }
        Ok(v)
    }
    /// Eintrag `von` an die Stelle `nach` setzen (Rang unter den benutzten Einträgen des Hauptverzeichnisses). Die Dateinummer steht in jedem Datensektor und
    /// ist der Platz im Verzeichnis: sie wird bei verschobenen Dateien mitgeführt. Nur für 128-Byte-Sektoren (DOS 2 und 2.5); sonst Fehler, ohne etwas zu ändern.
    pub fn dos2_move(&mut self, from: usize, to: usize) -> Result<(), String> {
        if self.ssz != 128 || self.kind() == DosKind::Keins { return Err(vc_i18n::t!("disk-verschieben-nur-dos2").into()); }
        let raws = self.read_dir(DIR_FIRST, DIR_SECTORS); let n = raws.len();
        if from >= n || to >= n { return Err(vc_i18n::t!("disk-eintrag-nummer-ungueltig").into()); }
        let slots: Vec<usize> = raws.iter().map(|r| r.idx).collect();
        let mut order: Vec<usize> = (0..n).collect(); let x = order.remove(from); order.insert(to, x);
        let mut ketten: Vec<(Vec<usize>, usize)> = vec![];
        for (k, &alt) in order.iter().enumerate() {
            let r = &raws[alt];
            if slots[k] != r.idx && r.flag & 0x10 == 0 { ketten.push((self.kette_sektoren(r.start, r.idx)?, slots[k] % 64)); }
        }
        let inhalt: Vec<Vec<u8>> = raws.iter().map(|r| self.entry(r.idx).to_vec()).collect();
        for (k, &alt) in order.iter().enumerate() { self.entry_mut(slots[k]).copy_from_slice(&inhalt[alt]); }
        for (secs, fno) in ketten { for s in secs { let sec = self.sector_mut(s); let l = sec.len(); sec[l - 3] = ((fno as u8) << 2) | (sec[l - 3] & 3); } }
        Ok(())
    }

    pub fn dos2_add(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        if self.kind() != DosKind::Dos2 { return Err(vc_i18n::t!("disk-schreiben-nur-dos2", dos = self.kind().name())); }
        let (base, ext) = Self::valid_name(name)?;
        if data.is_empty() { return Err(vc_i18n::t!("disk-name-leere-dateien-werden-nicht-unterstuetzt", name = &name)); }
        if self.dos2_entries().iter().any(|e| e.name == name) { return Err(vc_i18n::t!("disk-name-existiert-bereits", name = &name)); }
        let need = data.len().div_ceil(125);
        if need > self.free_sectors() { return Err(vc_i18n::t!("disk-diskette-voll-name-braucht-need-sektoren", name = &name, need = &need, p3 = self.free_sectors())); }
        let slot = (0..DIR_SECTORS * 8).find(|i| { let e = self.sector(DIR_FIRST + i / 8)[(i % 8) * 16]; e & 0x40 == 0 || e & 0x80 != 0 }).ok_or(vc_i18n::t!("disk-verzeichnis-voll-64-dateien"))?;
        let secs: Vec<usize> = (4..SECTORS).filter(|s| self.is_free(*s)).take(need).collect();
        for (k, &s) in secs.iter().enumerate() {
            let chunk = &data[k * 125..((k + 1) * 125).min(data.len())];
            let next = secs.get(k + 1).copied().unwrap_or(0);
            let sec = self.sector_mut(s);
            sec.fill(0);
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

    /// Ist `name` ein gültiger DOS-2-Dateiname (Name 1–8 Zeichen A–Z/0–9/_, erstes ein Buchstabe, Erweiterung bis 3 Zeichen, Großbuchstaben)?
    pub fn valid_dos2_name(name: &str) -> bool { Self::valid_name(name).is_ok() }

    /// Eintrag des Hauptverzeichnisses mit diesem Namen (Pfade mit `/` suchen im Unterverzeichnis, nur MyDOS). Gibt es den Namen mehrmals (ein nie abgeschlossener Rest,
    /// Kennzeichen `$43` mit „offen zum Schreiben“, neben der echten Datei), gewinnt der abgeschlossene Eintrag.
    fn find_raw(&self, name: &str) -> Option<Raw> {
        let hits: Vec<Raw> = self.baum().into_iter().filter(|(p, r)| p == name && r.flag & 0x10 == 0).map(|(_, r)| r).collect();
        let i = hits.iter().position(|r| r.flag & 0x01 == 0).unwrap_or(0);
        hits.into_iter().nth(i)
    }
    /// Index im Hauptverzeichnis (nur DOS 2: 0…63)
    fn find_entry(&self, name: &str) -> Option<usize> { if name.contains('/') { return None; } self.find_raw(name).map(|r| r.idx) }

    fn entry(&self, i: usize) -> &[u8] { let per = self.ssz / 16; &self.sector(DIR_FIRST + i / per)[(i % per) * 16..(i % per) * 16 + 16] }
    fn entry_mut(&mut self, i: usize) -> &mut [u8] { let per = self.ssz / 16; &mut self.sector_mut(DIR_FIRST + i / per)[(i % per) * 16..(i % per) * 16 + 16] }

    /// Läuft die Sektorkette eines Eintrags ab (Dateinummer, Byteanzahl, Schleifenschutz): Daten und Sektoren. Der Zähler im Verzeichnis wird hier nicht geprüft.
    /// Eine nie abgeschlossene Datei (offen zum Schreiben) darf im zuletzt vergebenen Sektor noch keine Dateinummer tragen; der Sektor gehört trotzdem dazu.
    fn kette(&self, r: &Raw, name: &str) -> Result<(Vec<u8>, Vec<usize>), String> {
        if !self.chains_ok() { return Err(vc_i18n::t!("disk-atr-ketten-ueber-1023-sektoren").into()); }
        let max = self.ssz - 3; let offen = r.flag & 0x01 != 0;
        let (mut s, mut out, mut secs) = (r.start, vec![], vec![]);
        while s != 0 {
            if !self.has(s) { return Err(vc_i18n::t!("disk-sektor-s-ausserhalb-der-diskette", s = &s)); }
            let (fno, next, bytes) = self.link(s);
            if fno != r.idx % 64 && offen { secs.push(s); break; }
            if fno != r.idx % 64 { return Err(vc_i18n::t!("disk-sektor-s-gehoert-nicht-zu-name", s = &s, name = &name, p3 = fno)); }
            if bytes > max { return Err(vc_i18n::t!("disk-sektor-s-ungueltige-byteanzahl-p2", s = &s, p2 = bytes)); }
            out.extend_from_slice(&self.sector(s)[..bytes]);
            secs.push(s);
            if secs.len() > self.sectors.len() { return Err(vc_i18n::t!("disk-sektorkette-laenger-als-im-verzeichnis").into()); }
            s = next;
        }
        Ok((out, secs))
    }

    /// Ist die Datei schreibgeschützt (Bit `$20` im Kennzeichen)? `None`: es gibt sie nicht.
    pub fn dos2_is_locked(&self, name: &str) -> Option<bool> { self.find_raw(name).map(|r| r.flag & 0x20 != 0) }

    /// Setzt oder löscht den Schreibschutz einer Datei (Hauptverzeichnis)
    pub fn dos2_set_locked(&mut self, name: &str, locked: bool) -> Result<(), String> {
        let i = self.find_entry(name).ok_or_else(|| vc_i18n::t!("disk-name-nicht-gefunden", name = &name))?;
        let f = &mut self.entry_mut(i)[0];
        if locked { *f |= 0x20 } else { *f &= !0x20 }
        Ok(())
    }

    /// Löscht eine Datei (nur Atari DOS 2): ihre Sektoren werden in der VTOC frei, der Zähler wird nachgeführt, der Eintrag erhält das Kennzeichen „gelöscht“
    /// (`$80`, wie in DOS 2). Hat der Name mehrere Einträge (unvollständige Reste), gehen alle weg. Geschützte Dateien bleiben mit
    /// `force = false` stehen (Fehler); mit `force` werden sie ebenfalls gelöscht. Gibt die Zahl der freigegebenen Sektoren zurück.
    /// Bei einem Fehler bleibt das Abbild unverändert.
    pub fn dos2_remove(&mut self, name: &str, force: bool) -> Result<usize, String> {
        if self.kind() != DosKind::Dos2 { return Err(vc_i18n::t!("disk-schreiben-nur-dos2", dos = self.kind().name())); }
        if self.find_entry(name).is_none() { return Err(vc_i18n::t!("disk-name-nicht-gefunden", name = &name)); }
        if !force && self.dos2_is_locked(name) == Some(true) { return Err(vc_i18n::t!("turbo-datei-name-ist-schreibgeschuetzt", name = &name)); }
        let mut work = self.clone();
        let mut freed = 0;
        while let Some(r) = work.find_raw(name) {
            // der Zähler im Verzeichnis zählt nicht (ein nie abgeschlossener Eintrag hat 0, aber eine Kette); maßgeblich ist die Kette
            let (_, secs) = work.kette(&r, name)?;
            for s in secs {
                if s < 4 || (VTOC..DIR_FIRST + DIR_SECTORS).contains(&s) || s >= SECTORS { return Err(vc_i18n::t!("disk-sektor-s-ausserhalb-der-diskette", s = &s)); }
                if !work.is_free(s) { Self::set_free(work.sector_mut(VTOC), s, true); freed += 1; }
            }
            work.entry_mut(r.idx)[0] = 0x80;
        }
        let free = work.free_sectors() + freed;
        let v = work.sector_mut(VTOC);
        v[3] = (free & 0xFF) as u8; v[4] = (free >> 8) as u8;
        *self = work;
        Ok(freed)
    }

    /// Ersetzt eine Datei (Inhalt neu, Name bleibt): die alte wird gelöscht, die neue angelegt. Ist die alte schreibgeschützt, ist das ein
    /// Fehler. Reicht der Platz nicht, bleibt das Abbild unverändert. Die Datei sperrt nicht von selbst.
    pub fn dos2_replace(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        if self.find_entry(name).is_none() { return Err(vc_i18n::t!("disk-name-nicht-gefunden", name = &name)); }
        self.dos2_put(name, data)
    }

    /// Legt die Datei an oder ersetzt sie, wenn es sie schon gibt (geschützte Dateien werden nicht überschrieben; bei Fehler bleibt das Abbild unverändert)
    pub fn dos2_put(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let mut work = self.clone();
        if work.find_entry(name).is_some() { work.dos2_remove(name, false)?; }
        work.dos2_add(name, data)?;
        *self = work;
        Ok(())
    }

    /// Sektoren, die eine Datei der Länge `len` belegt (125 Datenbytes je Sektor, DOS 2)
    pub fn sectors_for(len: usize) -> usize { len.div_ceil(125) }

    /// Liest eine Datei über die Sektorkette; prüft Dateinummern und Zähler. Bei MyDOS auch mit Pfad (`ORDNER/NAME.EXT`).
    pub fn dos2_read(&self, name: &str) -> Result<Vec<u8>, String> {
        let r = self.find_raw(name).ok_or(vc_i18n::t!("disk-name-nicht-gefunden", name = &name))?;
        let (out, secs) = self.kette(&r, name)?;
        if secs.len() > r.count { return Err(vc_i18n::t!("disk-sektorkette-laenger-als-im-verzeichnis").into()); }
        if secs.len() != r.count { return Err(vc_i18n::t!("disk-sektorkette-hat-count-statt-expect-sektoren", count = &secs.len(), expect = &r.count)); }
        Ok(out)
    }

    /// ATR-Datei: 16-Byte-Kopf + Sektoren in der Anordnung des Abbilds.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(16 + self.sectors.iter().map(Vec::len).sum::<usize>());
        v.extend_from_slice(&self.header);
        for s in &self.sectors { v.extend_from_slice(s); }
        v
    }

    pub fn from_bytes(b: &[u8]) -> Result<Atr, String> {
        if b.len() < 16 || b[0] != 0x96 || b[1] != 0x02 { return Err(vc_i18n::t!("disk-kein-atr-kennung-0296-fehlt").into()); }
        let size = ((b[2] as usize) | (b[3] as usize) << 8 | (b[6] as usize) << 16) * 16;
        let ssz = b[4] as usize | (b[5] as usize) << 8;
        if ssz != 128 && ssz != 256 { return Err(vc_i18n::t!("disk-atr-sektorgroesse-unbekannt", ssz = &ssz)); }
        let passt = || vc_i18n::t!("disk-atr-groesse-passt-nicht", size = &size, len = &(b.len() - 16));
        if b.len() != 16 + size { return Err(passt().into()); }
        let (n, boot128) = if ssz == 128 { if size % 128 != 0 { return Err(passt().into()); } (size / 128, false) }
            else if size % 256 == 0 { (size / 256, false) } else if size % 256 == 128 && size >= 384 { ((size - 384) / 256 + 3, true) }
            else { return Err(passt().into()); };
        if n < 3 { return Err(passt().into()); }
        let mut sectors = Vec::with_capacity(n); let mut p = 16;
        for s in 1..=n { let l = if boot128 && s <= 3 { 128 } else { ssz }; sectors.push(b[p..p + l].to_vec()); p += l; }
        let mut header = [0u8; 16]; header.copy_from_slice(&b[..16]);
        Ok(Atr { header, ssz, boot128, sectors })
    }

    /// Hat die Diskette ein Dateisystem mit VTOC (DOS 2, 2.5 oder MyDOS)? Eine bootbare Diskette ohne DOS (z. B. aus einer XEX-Datei) hat keine VTOC (DOS-Code ≠ 2).
    pub fn has_dos2(&self) -> bool { self.kind() != DosKind::Keins }

    /// Prüft Konsistenz von VTOC, Verzeichnis und Sektorketten (Rückgabe: Liste von Problemen, Fehler und Hinweise zusammen, siehe [`Atr::dos2_check_split`]).
    /// Eine Boot-Diskette ohne Dateisystem hat nichts zu prüfen (kein Problem, sondern ein anderer Typ von Diskette).
    pub fn dos2_check(&self) -> Vec<String> { let (mut e, w) = self.dos2_check_split(); e.extend(w); e }

    /// Wie [`Atr::dos2_check`], getrennt in (Fehler, Hinweise). Hinweise sind Altlasten, die weder Dateien noch Platzbedarf verfälschen und beim
    /// Schreiben nicht stören: ein nie abgeschlossener Eintrag (Kennzeichen „offen zum Schreiben“, wie auf der Diskette von Turbo BASIC XL) und
    /// Sektoren, die als belegt markiert sind, aber zu keiner Datei gehören. Kaputte Ketten, doppelt belegte oder fälschlich freie Sektoren
    /// und ein falscher Zähler sind Fehler. Jeder Verzeichniseintrag wird für sich geprüft (auch bei doppelten Namen).
    pub fn dos2_check_split(&self) -> (Vec<String>, Vec<String>) {
        if !self.has_dos2() { return (vec![], vec![]); }
        let n = self.sectors.len();
        let (mut errors, mut warnings) = (vec![], vec![]);
        let mut used = vec![false; n + 1];
        for (name, r) in self.baum() {
            if r.flag & 0x10 != 0 { continue; }
            match self.kette(&r, &name) {
                Err(m) => errors.push(format!("{name}: {m}")),
                Ok((_, secs)) => {
                    for &s in &secs { if used[s] { errors.push(vc_i18n::t!("disk-p1-sektor-s-doppelt-belegt", p1 = &name, s = &s)); } used[s] = true; }
                    if r.flag & 0x01 != 0 { warnings.push(vc_i18n::t!("turbo-warnung-eintrag-nicht-abgeschlossen", name = &name, count = secs.len())); }
                    else if secs.len() > r.count { errors.push(format!("{name}: {}", vc_i18n::t!("disk-sektorkette-laenger-als-im-verzeichnis"))); }
                    else if secs.len() != r.count { errors.push(format!("{name}: {}", vc_i18n::t!("disk-sektorkette-hat-count-statt-expect-sektoren", count = &secs.len(), expect = &r.count))); }
                }
            }
        }
        if !self.chains_ok() { return (errors, warnings); }   // ohne lesbare Ketten kein Vergleich mit der Bitmap
        let reserved: std::collections::HashSet<usize> = self.system_sectors().into_iter().collect();
        let mut free = 0;
        for s in 4..=self.last_mapped().min(n) {
            if reserved.contains(&s) { if self.is_free(s) { errors.push(vc_i18n::t!("disk-sektor-s-ist-reserviert-aber-als", s = &s)); } continue; }
            if self.is_free(s) { free += 1; if used[s] { errors.push(vc_i18n::t!("disk-sektor-s-ist-belegt-aber-als", s = &s)); } }
            else if !used[s] { warnings.push(vc_i18n::t!("disk-sektor-s-ist-als-belegt-markiert", s = &s)); }
        }
        if self.kind() != DosKind::Dos25 && free != self.free_sectors() { errors.push(vc_i18n::t!("disk-vtoc-zaehler-p1-stimmt-nicht-mit", p1 = self.free_sectors(), free = &free)); }
        (errors, warnings)
    }
}

#[cfg(test)]
mod tests_geometrie {
    use super::*;
    #[test]
    fn atr_mit_256_byte_sektoren_beide_anordnungen() {
        for boot128 in [false, true] {
            let mut a = Atr::blank_with(720, 256, boot128); a.sector_mut(5)[3] = 0xAB;
            let b = a.to_bytes(); assert_eq!(b.len(), 16 + if boot128 { 384 + 717 * 256 } else { 720 * 256 });
            let r = Atr::from_bytes(&b).unwrap(); assert_eq!((r.sector_size(), r.sector_count(), r.boot_128()), (256, 720, boot128)); assert_eq!(r.sector(5)[3], 0xAB);
            assert_eq!(r.sector(1).len(), if boot128 { 128 } else { 256 }); assert_eq!(r.to_bytes(), b, "Rundlauf byte-gleich");
        }
    }
    #[test]
    fn unbekannte_sektorgroesse_und_falsche_laenge_werden_klar_abgelehnt() {
        let mut b = vec![0x96, 0x02, 0x00, 0x2d, 0x00, 0x02, 0x00]; b.resize(16 + 720 * 256, 0);   // Sektorgröße 512
        assert!(Atr::from_bytes(&b).unwrap_err().contains("512"));
        let mut c = vec![0x96, 0x02, 0x00, 0x2d, 0x00, 0x01, 0x00]; c.resize(16 + 1000, 0);
        assert!(Atr::from_bytes(&c).is_err());
    }
}
